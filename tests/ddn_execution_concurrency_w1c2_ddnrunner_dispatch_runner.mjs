import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const baseline = "af7ad5224c4645c81316fe72fb092ac337374347";
const ledgerPath = path.join(
  root,
  "pack",
  "ddn_execution_concurrency_w1c2_ddnrunner_dispatch",
  "expected.jsonl",
);
const targetDir = process.env.DDN_W1C2_I2_CARGO_TARGET_DIR;
if (!targetDir) {
  throw new Error("DDN_W1C2_I2_CARGO_TARGET_DIR is required (fail closed)");
}

const rows = fs.readFileSync(ledgerPath, "utf8")
  .trim()
  .split(/\r?\n/u)
  .map((line) => JSON.parse(line));
if (rows.length !== 15) throw new Error(`expected 15 integration rows, got ${rows.length}`);
const caseIds = new Set(rows.map((row) => row.case_id));
const testIds = new Set(rows.map((row) => row.test_id));
if (caseIds.size !== 15 || testIds.size !== 15) throw new Error("duplicate I2 evidence identity");

function cargo(args) {
  const result = spawnSync("cargo", args, {
    cwd: root,
    env: { ...process.env, CARGO_TARGET_DIR: targetDir },
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;
  if (result.status !== 0) {
    throw new Error(`cargo ${args.join(" ")} failed (${result.status})\n${output}`);
  }
  return output;
}

const testModule = "ddn_runtime::w1c2_i2_ddnrunner_dispatch_tests::";
const listed = cargo(["test", "-p", "ddonirang-tool", "--lib", "w1c2_i2_case_", "--", "--list"]);
const compiled = new Set(
  listed.split(/\r?\n/u)
    .map((line) => line.match(new RegExp(`^${testModule}(w1c2_i2_case_[a-zA-Z0-9_]+): test$`, "u"))?.[1])
    .filter(Boolean),
);
for (const testId of testIds) {
  if (!compiled.has(testId)) throw new Error(`ledger test is not compiled: ${testId}`);
}
if (compiled.size !== testIds.size) {
  throw new Error(`compiled/ledger mismatch ${compiled.size}/${testIds.size}`);
}
const execution = cargo(["test", "-p", "ddonirang-tool", "--lib", "w1c2_i2_case_", "--", "--nocapture"]);
if (!/test result: ok\. 15 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out/u.test(execution)) {
  throw new Error("actual DdnRunner internal integration module did not execute exact 15/15");
}

const runtimePath = path.join(root, "tool/src/ddn_runtime.rs");
const runtime = fs.readFileSync(runtimePath, "utf8");
for (const [label, pattern] of [
  ["shared prepare", /self\s*\.\s*internal_dispatch\s*\.\s*runtime\s*\.\s*prepare\s*\(/u],
  ["shared commit", /self\s*\.\s*internal_dispatch\s*\.\s*runtime\s*\.\s*commit_prepared\s*\(/u],
  ["outer commit event", /ddn_route_frame_committed/u],
  ["default empty registry", /ReactionProviderRegistry::default\(\)/u],
  ["public ingress count", /pub fn public_provider_source_ingress_count\(&self\) -> usize/u],
]) {
  if (!pattern.test(runtime)) throw new Error(`DdnRunner shared dispatch binding missing: ${label}`);
}
if (!/pub fn public_provider_source_ingress_count\(&self\) -> usize \{\s*0\s*\}/u.test(runtime)) {
  throw new Error("DdnRunner public provider source reachability is not exact zero");
}
const injectionMethods = [
  "with_internal_reaction_provider_registry",
  "with_internal_provider_reachability",
  "with_internal_reaction_runtime",
];
for (const method of injectionMethods) {
  const cratePrivate = new RegExp(`pub\\(crate\\)\\s+fn\\s+${method}\\b`, "u");
  const externallyPublic = new RegExp(`(?<!pub\\(crate\\)\\s)pub\\s+fn\\s+${method}\\b`, "u");
  if (!cratePrivate.test(runtime) || externallyPublic.test(runtime)) {
    throw new Error(`internal injection visibility escaped crate boundary: ${method}`);
  }
}
const crateRoot = fs.readFileSync(path.join(root, "tool/src/lib.rs"), "utf8");
if (/pub\s+use\s+ddn_runtime::(?:\{[^}]*\bDdnRunner\b[^}]*\}|DdnRunner)\s*;/u.test(crateRoot)) {
  throw new Error("DdnRunner crate-root re-export is forbidden for internal injection custody");
}

const publicIngressPaths = [
  "lang/src/ast.rs",
  "lang/src/parser.rs",
  "lang/src/normalizer.rs",
  "lang/src/currentline.rs",
  "tool/src/preprocess.rs",
  "tool/src/wasm_api.rs",
  "tools/teul-cli/src/cli",
  "tools/teul-cli/src/main.rs",
  "solutions/seamgrim_ui_mvp/ui",
].map((entry) => path.join(root, entry));
const forbiddenTokens = [
  "반응제공자",
  "반응하면",
  "--reaction-provider",
  "DDN_REACTION_PROVIDER",
  "providerRegistry",
];
for (const target of publicIngressPaths) {
  const stat = fs.statSync(target);
  const files = stat.isDirectory()
    ? fs.readdirSync(target, { recursive: true })
        .filter((entry) => /\.(?:rs|js|ts|html)$/u.test(entry))
        .map((entry) => path.join(target, entry))
    : [target];
  for (const file of files) {
    const text = fs.readFileSync(file, "utf8");
    for (const token of forbiddenTokens) {
      if (text.includes(token)) {
        throw new Error(`public provider ingress token ${token} found in ${path.relative(root, file)}`);
      }
    }
  }
}

const publicDiff = spawnSync(
  "git",
  [
    "diff", "--name-only", baseline, "--",
    "lang/src/ast.rs", "lang/src/parser.rs", "lang/src/normalizer.rs",
    "lang/src/currentline.rs", "tool/src/preprocess.rs", "tool/src/wasm_api.rs",
    "tools/teul-cli/src/cli", "tools/teul-cli/src/main.rs",
    "solutions/seamgrim_ui_mvp/ui",
    ":(exclude)solutions/seamgrim_ui_mvp/ui/wasm/**",
  ],
  { cwd: root, encoding: "utf8" },
);
if (publicDiff.status !== 0 || publicDiff.stdout.trim() !== "") {
  throw new Error(`public provider ingress surface changed: ${publicDiff.stdout}`);
}

const i1Ledger = fs.readFileSync(
  path.join(root, "pack/ddn_execution_concurrency_w1c2_internal_typed/expected.jsonl"),
  "utf8",
);
if (i1Ledger.includes("22_teul_legacy_provider_fallback_zero") ||
    !i1Ledger.includes("22_tool_ddnrunner_public_provider_ingress_zero")) {
  throw new Error("case 22 evidence label accuracy correction missing");
}
const teulEval = fs.readFileSync(path.join(root, "tools/teul-cli/src/runtime/eval.rs"), "utf8");
if (!teulEval.includes("plan_delivery_occurrence(") ||
    !teulEval.includes("ReactionProviderRegistry::<()>::default()")) {
  throw new Error("teul-cli empty-registry static scan failed");
}

console.log(JSON.stringify({
  status: "PASS",
  actual_ddnrunner_integration_cases: 15,
  external_rust_provider_injection_api_count: 0,
  public_provider_syntax_token_count: 0,
  public_provider_source_ingress_count: 0,
  public_provider_cli_option_count: 0,
  public_provider_env_count: 0,
  public_provider_wasm_export_count: 0,
  public_provider_ui_control_count: 0,
  case22_tool_ddnrunner_label_corrected: true,
  teul_cli_empty_registry_static_scan: "PASS",
}));
