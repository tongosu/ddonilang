import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

const root = path.resolve(import.meta.dirname, "..");
const wasmDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm");
const wasm = await import(pathToFileURL(path.join(wasmDir, "ddonirang_tool.js")));
await wasm.default(fs.readFileSync(path.join(wasmDir, "ddonirang_tool_bg.wasm")));

const fixturePath = path.join(root, "pack", "gaji_creator_wave1", "g2_extraction_request.detjson");
const fixtureText = fs.readFileSync(fixturePath, "utf8");
const outRoot = path.join(os.tmpdir(), `ddn-gaji-output-${process.pid}`);
const cargoTargetRoot = path.join(outRoot, "cargo-target");

function cargoRun(workspace, args) {
  try {
    return execFileSync("cargo", args, {
      cwd: root,
      encoding: "utf8",
      // teul-cli is a nested Cargo workspace with its own lock graph.  Keep
      // its compiled native libraries separate from the official workspace,
      // otherwise a later teul invocation can link two bzip2-sys copies.
      env: { ...process.env, CARGO_TARGET_DIR: path.join(cargoTargetRoot, workspace) },
    });
  } catch (error) {
    fs.rmSync(outRoot, { recursive: true, force: true });
    throw error;
  }
}

const wasmExtraction = wasm.wasm_gaji_extraction_plan_json(fixtureText);
const cliExtraction = cargoRun("teul", [
  "run", "--quiet", "--manifest-path", "tools/teul-cli/Cargo.toml", "--",
  "gaji", "plan", "--input", fixturePath,
]);
assert.equal(wasmExtraction, cliExtraction, "CLI/WASM extraction canonical JSON differs");
const extractionOut = path.join(outRoot, "extraction", "nested", "plan.detjson");
cargoRun("teul", [
  "run", "--quiet", "--manifest-path", "tools/teul-cli/Cargo.toml", "--",
  "gaji", "plan", "--input", fixturePath, "--out", extractionOut,
]);
assert.equal(
  fs.readFileSync(extractionOut, "utf8"),
  wasmExtraction,
  "CLI extraction --out differs from shared WASM authority",
);

const fixture = JSON.parse(fixtureText);
const extractionArtifact = JSON.parse(wasm.wasm_local_gaji_artifact_json(fixtureText));
assert.equal(extractionArtifact.gaji_id, "운동");
const runtimeFixture = structuredClone(fixture);
runtimeFixture.source.files.find((file) => file.path === "lib/운동.ddn").utf8_text =
  '쓰임 { 수치: "./lib/수치". }\n드러냄 { 속도. }\n속도 <- 수치.기본값.';
const artifact = JSON.parse(wasm.wasm_local_gaji_artifact_json(JSON.stringify(runtimeFixture)));
const sourceHash = wasm.wasm_project_source_sha256_json(JSON.stringify(runtimeFixture.source));
const lifecycleRequest = {
  schema: "ddn.gaji_lifecycle_plan.v1",
  source: runtimeFixture.source,
  expected_source_sha256: sourceHash,
  expected_lock_sha256: null,
  operation: { kind: "install", alias: "재사용힘", artifact },
};
const temp = path.join(os.tmpdir(), `ddn-gaji-lifecycle-${process.pid}.json`);
const runTemp = path.join(os.tmpdir(), `ddn-gaji-run-${process.pid}.json`);
function cliRun(source) {
  fs.writeFileSync(runTemp, `${JSON.stringify(source)}\n`, "utf8");
  return cargoRun("official", ["run", "--quiet", "-p", "ddonirang-tool", "--bin", "ddonirang", "--",
    "project-run", "--virtual", runTemp, "--target", "앱", "--inspect-graph", "--inspect-lock"],
  );
}
fs.writeFileSync(temp, `${JSON.stringify(lifecycleRequest)}\n`, "utf8");
try {
  const wasmLifecycle = wasm.wasm_gaji_lifecycle_plan_json(JSON.stringify(lifecycleRequest));
  const cliLifecycle = cargoRun("teul", [
    "run", "--quiet", "--manifest-path", "tools/teul-cli/Cargo.toml", "--",
    "gaji", "lifecycle-plan", "--input", temp,
  ]);
  assert.equal(wasmLifecycle, cliLifecycle, "CLI/WASM lifecycle canonical JSON differs");
  const lifecycleOut = path.join(outRoot, "lifecycle", "nested", "plan.detjson");
  cargoRun("teul", [
    "run", "--quiet", "--manifest-path", "tools/teul-cli/Cargo.toml", "--",
    "gaji", "lifecycle-plan", "--input", temp, "--out", lifecycleOut,
  ]);
  assert.equal(
    fs.readFileSync(lifecycleOut, "utf8"),
    wasmLifecycle,
    "CLI lifecycle --out differs from shared WASM authority",
  );
  const plan = JSON.parse(wasmLifecycle);
  assert.ok(plan.after_lock_sha256);
  assert.equal(plan.project_source.files.some((file) => file.path === "ddn.lock"), true);
  plan.project_source.files.find((file) => file.path === "app/main.ddn").utf8_text =
    '쓰임 { 운동: "./vendor/운동/ddn/lib/운동". }\n운동.속도 보여주기.';
  plan.project_source.files = plan.project_source.files.filter((file) => file.path !== "ddn.lock");
  const lockV1 = JSON.parse(wasm.wasm_project_gaji_lock_json(JSON.stringify(plan.project_source), false));
  plan.project_source.files.push({ path: "ddn.lock", role: "lock", utf8_text: `${JSON.stringify(lockV1)}\n` });
  const wasmRunV1 = wasm.wasm_project_run_summary_json(JSON.stringify(plan.project_source), "앱", true, true);
  assert.equal(wasmRunV1, cliRun(plan.project_source), "installed V1 CLI/WASM run differs");

  const fixtureV2 = structuredClone(runtimeFixture);
  fixtureV2.version = "2.0.0";
  fixtureV2.source.files.find((file) => file.path === "lib/운동.ddn").utf8_text = "드러냄 { 속도. }\n속도 <- 8.";
  const artifactV2 = JSON.parse(wasm.wasm_local_gaji_artifact_json(JSON.stringify(fixtureV2)));
  const updateV2 = JSON.parse(wasm.wasm_gaji_lifecycle_plan_json(JSON.stringify({
    schema: "ddn.gaji_lifecycle_plan.v1", source: plan.project_source,
    expected_source_sha256: wasm.wasm_project_source_sha256_json(JSON.stringify(plan.project_source)), expected_lock_sha256: null,
    operation: { kind: "update", alias: "재사용힘", artifact: artifactV2 },
  })));
  const wasmRunV2 = wasm.wasm_project_run_summary_json(JSON.stringify(updateV2.project_source), "앱", true, true);
  assert.equal(wasmRunV2, cliRun(updateV2.project_source), "installed V2 CLI/WASM run differs");
  assert.notDeepEqual(JSON.parse(wasmRunV2).output_rows, JSON.parse(wasmRunV1).output_rows, "V2 physics observation did not change");

  const rollback = JSON.parse(wasm.wasm_gaji_lifecycle_plan_json(JSON.stringify({
    schema: "ddn.gaji_lifecycle_plan.v1", source: updateV2.project_source,
    expected_source_sha256: wasm.wasm_project_source_sha256_json(JSON.stringify(updateV2.project_source)), expected_lock_sha256: null,
    operation: { kind: "update", alias: "재사용힘", artifact },
  })));
  const wasmRollback = wasm.wasm_project_run_summary_json(JSON.stringify(rollback.project_source), "앱", true, true);
  assert.equal(wasmRollback, cliRun(rollback.project_source), "rollback CLI/WASM run differs");
  assert.deepEqual(JSON.parse(wasmRollback).output_rows, JSON.parse(wasmRunV1).output_rows, "rollback output did not reproduce V1");
  assert.equal(JSON.parse(wasmRollback).runtime_state_hash, JSON.parse(wasmRunV1).runtime_state_hash, "rollback state hash did not reproduce V1");
  const mismatch = structuredClone(lifecycleRequest);
  mismatch.expected_source_sha256 = "0".repeat(64);
  assert.throws(() => wasm.wasm_gaji_lifecycle_plan_json(JSON.stringify(mismatch)), /E_GAJI_LIFECYCLE_SOURCE_STALE/);
} finally {
  fs.rmSync(temp, { force: true });
  fs.rmSync(runTemp, { force: true });
  fs.rmSync(outRoot, { recursive: true, force: true });
}

console.log("gaji_creator_wasm_parity: ok");
