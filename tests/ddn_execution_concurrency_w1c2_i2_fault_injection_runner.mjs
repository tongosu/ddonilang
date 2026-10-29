import { createHash } from "node:crypto";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const targetDir = process.env.DDN_W1C2_I2_CARGO_TARGET_DIR;
const externalTargetDir = process.env.DDN_W1C2_I2_EXTERNAL_CARGO_TARGET_DIR;
const externalTempRoot = process.env.DDN_W1C2_I2_EXTERNAL_CONSUMER_TEMP_ROOT;
if (!targetDir || !externalTargetDir || !externalTempRoot) {
  throw new Error(
    "DDN_W1C2_I2_CARGO_TARGET_DIR, DDN_W1C2_I2_EXTERNAL_CARGO_TARGET_DIR, and DDN_W1C2_I2_EXTERNAL_CONSUMER_TEMP_ROOT are required (fail closed)",
  );
}

const toolPath = resolve(root, "tool/src/ddn_runtime.rs");
const libPath = resolve(root, "tool/src/lib.rs");
const manifestPath = resolve(root, "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.manifest.detjson");
const wasmPath = resolve(root, "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm");
const membershipProbePath = resolve(root, "tool/src/w1c2_i2_fault_membership_probe.rs");
if (existsSync(membershipProbePath)) {
  throw new Error(`${membershipProbePath}: task-owned membership probe already exists`);
}

const protectedPaths = [toolPath, libPath, manifestPath, wasmPath];
const originals = new Map(protectedPaths.map((path) => [path, readFileSync(path)]));
const sha256 = (data) => createHash("sha256").update(data).digest("hex");
const textOf = (path) => originals.get(path).toString("utf8").replace(/\r\n/g, "\n");

function replaceExact(source, before, after, label) {
  const first = source.indexOf(before);
  if (first < 0 || source.indexOf(before, first + before.length) >= 0) {
    throw new Error(`${label}: mutation anchor must occur exactly once`);
  }
  return source.slice(0, first) + after + source.slice(first + before.length);
}

function restoreAll() {
  for (const [path, original] of originals) writeFileSync(path, original);
  rmSync(membershipProbePath, { force: true });
}

function runExact(testName, expectedGreen) {
  const result = spawnSync(
    "cargo",
    ["test", "-p", "ddonirang-tool", "--lib", `ddn_runtime::w1c2_i2_ddnrunner_dispatch_tests::${testName}`, "--", "--exact", "--nocapture"],
    { cwd: root, env: { ...process.env, CARGO_TARGET_DIR: targetDir }, encoding: "utf8", windowsHide: true },
  );
  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
  if (!/running 1 test/u.test(output)) throw new Error(`${testName}: zero/multi-match filter is forbidden\n${output}`);
  if (expectedGreen) {
    if (result.status !== 0 || !/1 passed; 0 failed/u.test(output)) throw new Error(`${testName}: expected GREEN\n${output}`);
  } else if (result.status === 0 || !/test result: FAILED/u.test(output) || !/0 passed; 1 failed/u.test(output) || /could not compile/u.test(output)) {
    throw new Error(`${testName}: mutation must produce an assertion RED, not compile RED\n${output}`);
  }
}

function runBoundaryGate(expectedGreen) {
  const env = {
    ...process.env,
    DDN_W1C2_I2_CARGO_TARGET_DIR: targetDir,
    DDN_W1C2_I2_EXTERNAL_CARGO_TARGET_DIR: externalTargetDir,
    DDN_W1C2_I2_EXTERNAL_CONSUMER_TEMP_ROOT: externalTempRoot,
  };
  const visibility = spawnSync("node", ["tests/ddn_execution_concurrency_w1c2_ddnrunner_dispatch_runner.mjs"], {
    cwd: root, env, encoding: "utf8", windowsHide: true,
  });
  const consumer = spawnSync("node", ["tests/ddn_execution_concurrency_w1c2_i2_external_consumer_compile_fail_runner.mjs"], {
    cwd: root, env, encoding: "utf8", windowsHide: true,
  });
  const visibilityOutput = `${visibility.stdout ?? ""}\n${visibility.stderr ?? ""}`;
  const consumerOutput = `${consumer.stdout ?? ""}\n${consumer.stderr ?? ""}`;
  if (expectedGreen) {
    if (visibility.status !== 0 || consumer.status !== 0 || !/"external_rust_provider_injection_api_count":0/u.test(visibilityOutput) || !/"external_consumer_compile_fail":true/u.test(consumerOutput)) {
      throw new Error(`boundary gate expected GREEN\n${visibilityOutput}\n${consumerOutput}`);
    }
  } else if (visibility.status === 0 || consumer.status === 0) {
    throw new Error(`boundary mutation must make both visibility and external-consumer gates RED\n${visibilityOutput}\n${consumerOutput}`);
  }
}

function runFreshness(expectedGreen) {
  const result = spawnSync("python", ["tests/run_ddn_cli_wasm_freshness_check.py"], {
    cwd: root, encoding: "utf8", windowsHide: true,
  });
  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
  if (expectedGreen) {
    if (result.status !== 0 || !/\[ddn-cli-wasm-freshness\] PASS/u.test(output)) throw new Error(`freshness gate expected GREEN\n${output}`);
  } else if (result.status === 0 || !/\[ddn-cli-wasm-freshness\] fail:/u.test(output)) {
    throw new Error(`freshness mutation must produce checker RED\n${output}`);
  }
}

const preparedRouteAnchor = "            let route_index = route_stmt_indices";
const preparePrematureEffect = `            if let Some(effect) = prepared.group.effect_intents.first() {
                self.internal_dispatch.effect_sink_observations.borrow_mut().push(InternalEffectSinkObservation {
                    identity: effect.identity.clone(),
                    committed_world: self.internal_dispatch.runtime.world().clone(),
                    committed_revision: self.internal_dispatch.runtime.revision(),
                    route_frame_committed: false,
                });
            }
            let route_index = route_stmt_indices`;
const precommitStatePublish = `            self.internal_dispatch.runtime.world.insert(
                "fault-precommit-state".to_string(),
                "published-before-route-commit".to_string(),
            );
            let route_index = route_stmt_indices`;
const failureBranchAnchor = `        } else {
            self.async_outbox.abort_frame();`;
const providerFailureEffect = `        } else {
            self.internal_dispatch.effect_sink_observations.borrow_mut().push(InternalEffectSinkObservation {
                identity: "fault-provider-failure-release".to_string(),
                committed_world: self.internal_dispatch.runtime.world().clone(),
                committed_revision: self.internal_dispatch.runtime.revision(),
                route_frame_committed: false,
            });
            self.async_outbox.abort_frame();`;
const mergeFailureEffect = `        } else {
            self.internal_dispatch.effect_sink_observations.borrow_mut().push(InternalEffectSinkObservation {
                identity: "fault-merge-failure-release".to_string(),
                committed_world: self.internal_dispatch.runtime.world().clone(),
                committed_revision: self.internal_dispatch.runtime.revision(),
                route_frame_committed: false,
            });
            self.async_outbox.abort_frame();`;
const publishAfterAbort = `        } else {
            let mut fault_leak_source = self.async_outbox.clone();
            self.async_outbox.abort_frame();
            let fault_published_after_abort = fault_leak_source.commit_frame();
            self.admit_async_envelopes(fault_published_after_abort);
            self.internal_dispatch.effect_sink_observations.borrow_mut().push(InternalEffectSinkObservation {
                identity: "fault-outbox-published-after-abort".to_string(),
                committed_world: self.internal_dispatch.runtime.world().clone(),
                committed_revision: self.internal_dispatch.runtime.revision(),
                route_frame_committed: false,
            });`;
const observationCommitAnchor = `            self.internal_dispatch
                .effect_sink_observations
                .borrow_mut()
                .extend(effect_sink_observations);`;
const duplicateEffectAfterCommit = `            let duplicate_effect_sink_observations = effect_sink_observations.clone();
            self.internal_dispatch
                .effect_sink_observations
                .borrow_mut()
                .extend(effect_sink_observations);
            self.internal_dispatch
                .effect_sink_observations
                .borrow_mut()
                .extend(duplicate_effect_sink_observations);`;

const test09 = "w1c2_i2_case_09_ddnrunner_provider_failure_aborts_route_publish";
const test10 = "w1c2_i2_case_10_ddnrunner_provider_conflict_is_deterministic_failure";
const test11 = "w1c2_i2_case_11_ddnrunner_effect_intent_releases_after_outer_commit";

const mutations = [
  {
    axis: "01_builder_visibility_pub", group: "F1_API_CONFINEMENT", verify: runBoundaryGate,
    mutate: () => writeFileSync(toolPath, replaceExact(textOf(toolPath), "    pub(crate) fn with_internal_reaction_provider_registry(", "    pub fn with_internal_reaction_provider_registry(", "axis01"), "utf8"),
  },
  {
    axis: "02_crate_root_injection_api_reexport", group: "F1_API_CONFINEMENT", verify: runBoundaryGate,
    mutate: () => {
      writeFileSync(toolPath, replaceExact(textOf(toolPath), "    pub(crate) fn with_internal_reaction_provider_registry(", "    pub fn with_internal_reaction_provider_registry(", "axis02-builder"), "utf8");
      writeFileSync(libPath, replaceExact(textOf(libPath), "pub mod ddn_runtime;", "pub mod ddn_runtime;\npub use ddn_runtime::DdnRunner;", "axis02-reexport"), "utf8");
    },
  },
  {
    axis: "03_prepare_effect_immediate", group: "F2_EFFECT_ATOMICITY", verify: (green) => runExact(test11, green),
    mutate: () => writeFileSync(toolPath, replaceExact(textOf(toolPath), preparedRouteAnchor, preparePrematureEffect, "axis03"), "utf8"),
  },
  {
    axis: "04_provider_failure_effect", group: "F2_EFFECT_ATOMICITY", verify: (green) => runExact(test09, green),
    mutate: () => writeFileSync(toolPath, replaceExact(textOf(toolPath), failureBranchAnchor, providerFailureEffect, "axis04"), "utf8"),
  },
  {
    axis: "05_merge_failure_partial_effect", group: "F2_EFFECT_ATOMICITY", verify: (green) => runExact(test10, green),
    mutate: () => writeFileSync(toolPath, replaceExact(textOf(toolPath), failureBranchAnchor, mergeFailureEffect, "axis05"), "utf8"),
  },
  {
    axis: "06_state_publish_before_commit", group: "F2_EFFECT_ATOMICITY", verify: (green) => runExact(test11, green),
    mutate: () => writeFileSync(toolPath, replaceExact(textOf(toolPath), preparedRouteAnchor, precommitStatePublish, "axis06"), "utf8"),
  },
  {
    axis: "07_outbox_publish_after_abort", group: "F2_EFFECT_ATOMICITY", verify: (green) => runExact(test09, green),
    mutate: () => writeFileSync(toolPath, replaceExact(textOf(toolPath), failureBranchAnchor, publishAfterAbort, "axis07"), "utf8"),
  },
  {
    axis: "08_duplicate_effect_after_commit", group: "F2_EFFECT_ATOMICITY", verify: (green) => runExact(test11, green),
    mutate: () => writeFileSync(toolPath, replaceExact(textOf(toolPath), observationCommitAnchor, duplicateEffectAfterCommit, "axis08"), "utf8"),
  },
  {
    axis: "09_wrong_profile_and_toolchain_accepted", group: "F3_WASM_REPRODUCIBILITY", verify: runFreshness,
    mutate: () => {
      const manifest = JSON.parse(textOf(manifestPath));
      manifest.profile = "fault-debug";
      manifest.rustc_version = "rustc fault-toolchain";
      writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
    },
  },
  {
    axis: "10_stale_target_accepted", group: "F3_WASM_REPRODUCIBILITY", verify: runFreshness,
    mutate: () => writeFileSync(wasmPath, Buffer.concat([originals.get(wasmPath), Buffer.from([0])])),
  },
  {
    axis: "11_wrong_wasm_selected_by_manifest", group: "F3_WASM_REPRODUCIBILITY", verify: runFreshness,
    mutate: () => {
      const manifest = JSON.parse(textOf(manifestPath));
      manifest.wasm_path = "solutions/seamgrim_ui_mvp/ui/wasm/not-the-product-artifact.wasm";
      writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
    },
  },
  {
    axis: "12_source_membership_one_file_change_missed", group: "F3_WASM_REPRODUCIBILITY", verify: runFreshness,
    mutate: () => writeFileSync(membershipProbePath, "// Fault-injection-only source membership probe.\npub(crate) const MEMBERSHIP_PROBE: u8 = 1;\n", "utf8"),
  },
];

const results = [];
try {
  for (const mutation of mutations) {
    restoreAll();
    mutation.verify(true);
    const before = new Map(protectedPaths.map((path) => [path, sha256(readFileSync(path))]));
    mutation.mutate();
    const changed = protectedPaths.some((path) => sha256(readFileSync(path)) !== before.get(path)) || existsSync(membershipProbePath);
    if (!changed) throw new Error(`${mutation.axis}: mutation was inert`);
    mutation.verify(false);
    restoreAll();
    mutation.verify(true);
    results.push({ axis: mutation.axis, group: mutation.group, original_green: true, mutation_red: true, restored_green: true });
  }
} finally {
  restoreAll();
}

for (const [path, original] of originals) {
  if (sha256(readFileSync(path)) !== sha256(original)) throw new Error(`${path}: final restoration hash mismatch`);
}
if (existsSync(membershipProbePath)) throw new Error(`${membershipProbePath}: final task-owned residue remains`);

const counts = Object.fromEntries(
  ["F1_API_CONFINEMENT", "F2_EFFECT_ATOMICITY", "F3_WASM_REPRODUCIBILITY"].map((group) => [group, results.filter((result) => result.group === group).length]),
);
console.log(JSON.stringify({ status: "PASS", required_fault_axes: results.length, required_fault_axes_expected: 12, group_counts: counts, results }));
