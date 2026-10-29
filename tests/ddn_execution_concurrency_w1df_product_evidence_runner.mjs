import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const oraclePath = path.resolve(
  process.env.DDN_W1DF_ORACLE_PATH
    ?? path.join(repoRoot, "pack", "future_grammar_atlas_owner_closed_delta_r2_w1df", "logical_oracle.jsonl"),
);
const productFixedHead = "d8a4b2ab5b7c5fe68b1738f955d2d3d5b2b86ad1";
const cargoTargetDir = path.resolve(String(process.env.DDN_W1DF_CARGO_TARGET_DIR ?? "").trim());
const teulCargoTargetDir = path.resolve(
  String(process.env.DDN_W1DF_TEUL_CARGO_TARGET_DIR ?? "").trim(),
);

assert(String(process.env.DDN_W1DF_CARGO_TARGET_DIR ?? "").trim(),
  "DDN_W1DF_CARGO_TARGET_DIR must be an isolated external target directory");
assert(String(process.env.DDN_W1DF_TEUL_CARGO_TARGET_DIR ?? "").trim(),
  "DDN_W1DF_TEUL_CARGO_TARGET_DIR must isolate teul-cli's standalone lockfile");

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, CARGO_TARGET_DIR: cargoTargetDir },
    ...options,
  });
  const stdout = String(result.stdout ?? "").replaceAll("\r\n", "\n");
  const stderr = String(result.stderr ?? "").replaceAll("\r\n", "\n");
  assert.equal(result.status, 0, `${command} ${args.join(" ")}\n${stdout}\n${stderr}`);
  return { stdout, stderr };
}

const rows = fs.readFileSync(oraclePath, "utf8")
  .trim()
  .split(/\r?\n/u)
  .map((line) => JSON.parse(line));

assert.equal(rows.length, 65, "logical oracle cardinality");
for (const row of rows) {
  assert.equal(row.logical_oracle_class, "LOGICAL_ORACLE_ONLY", `${row.case_id}: logical class`);
  assert.equal(Object.hasOwn(row, "product_status"), false, `${row.case_id}: stale product_status`);
  assert.equal(JSON.stringify(row).includes("PRODUCT_PASS"), false, `${row.case_id}: stale PRODUCT_PASS label`);
}

const publicRows = rows.filter((row) => row.evidence_class === "PUBLIC_SOURCE_PRODUCT_CASE");
const internalRows = rows.filter((row) => row.evidence_class === "INTERNAL_TYPED_PRODUCT_CASE");
assert.deepEqual(publicRows.map((row) => row.case_id).sort(), [
  "FGA2-N10", "FGA2-P09", "FGA2-P10", "FGA2-X20",
]);
assert.deepEqual(internalRows.map((row) => row.case_id).sort(), [
  "FGA2-N11", "FGA2-P11", "FGA2-X14",
]);
assert.equal(rows.filter((row) => row.evidence_class === "PRODUCT_UNSUPPORTED").length, 58);

for (const row of [...publicRows, ...internalRows]) {
  assert.equal(row.evidence_fixed_head, productFixedHead, `${row.case_id}: product fixed HEAD`);
  assert.equal(row.evidence_test_ids.length > 0, true, `${row.case_id}: evidence test IDs`);
  assert.notEqual(row.evidence_frontdoor, "NONE", `${row.case_id}: evidence frontdoor`);
  assert.notEqual(row.evidence_scope, "NO_CURRENT_PRODUCT_EVIDENCE", `${row.case_id}: evidence scope`);
}

run("git", ["cat-file", "-e", `${productFixedHead}^{commit}`]);
run("git", ["merge-base", "--is-ancestor", productFixedHead, "HEAD"]);

const w1c1ExpectedPath = path.join(
  repoRoot,
  "pack",
  "ddn_execution_concurrency_w1c1_sender_route_provider",
  "expected.jsonl",
);
const w1c1Cases = new Set(
  fs.readFileSync(w1c1ExpectedPath, "utf8")
    .trim()
    .split(/\r?\n/u)
    .map((line) => JSON.parse(line).case),
);
const mappedW1c1Cases = new Set();
for (const row of publicRows) {
  for (const testId of row.evidence_test_ids) {
    if (!testId.startsWith("w1c1:")) continue;
    const caseId = testId.slice("w1c1:".length);
    assert.equal(w1c1Cases.has(caseId), true, `${row.case_id}: missing W1-C1 case ${caseId}`);
    assert.equal(
      fs.existsSync(path.join(repoRoot, "pack", "ddn_execution_concurrency_w1c1_sender_route_provider", caseId, "input.ddn")),
      true,
      `${row.case_id}: missing W1-C1 input ${caseId}`,
    );
    mappedW1c1Cases.add(caseId);
  }
}
assert.equal(mappedW1c1Cases.size > 0, true, "public evidence must map official W1-C1 cases");

const official = run(process.execPath, [
  path.join(repoRoot, "tests", "ddn_execution_concurrency_w1c1_sender_route_provider_runner.mjs"),
]);
const officialLines = official.stdout.trim().split(/\r?\n/u);
const officialResult = JSON.parse(officialLines.at(-1));
assert.equal(officialResult.schema, "ddn.execution_concurrency.w1c1.sender_route_provider.parity.v1");
assert.equal(officialResult.cases, 10);
assert.equal(officialResult.source_built_wasm, true);
assert.equal(officialResult.tracked_workbench_wasm, true);

const cargoResults = new Map();
function runCargoEvidence(testId) {
  if (cargoResults.has(testId)) return cargoResults.get(testId);
  const [, owner, testName] = testId.split(":");
  assert(owner && testName, `invalid cargo evidence ID: ${testId}`);
  const args = owner === "teul-cli"
    ? ["test", "--manifest-path", "tools/teul-cli/Cargo.toml", testName, "--", "--nocapture"]
    : owner === "ddonirang-tool-integration"
      ? ["test", "-p", "ddonirang-tool", "--test", "ddn_execution_concurrency_w1df_capacity", testName, "--", "--nocapture"]
      : ["test", "-p", owner, "--lib", testName, "--", "--nocapture"];
  const result = run("cargo", args, owner === "teul-cli"
    ? { env: { ...process.env, CARGO_TARGET_DIR: teulCargoTargetDir } }
    : {});
  const output = `${result.stdout}\n${result.stderr}`;
  assert.match(output, /test result: ok\. [1-9][0-9]* passed; 0 failed; 0 ignored/u,
    `${testId}: named test must execute and must not be ignored`);
  cargoResults.set(testId, { owner, test_name: testName, outcome: "PASS" });
  return cargoResults.get(testId);
}

for (const row of [...publicRows, ...internalRows]) {
  for (const testId of row.evidence_test_ids) {
    if (testId.startsWith("cargo:")) runCargoEvidence(testId);
  }
}

console.log(JSON.stringify({
  schema: "ddn.execution_concurrency.w1df.product_evidence.v1",
  product_fixed_head: productFixedHead,
  logical_cases: rows.length,
  public_source_product_cases: publicRows.length,
  internal_typed_product_cases: internalRows.length,
  product_unsupported_cases: 58,
  official_w1c1_cases_executed: officialResult.cases,
  mapped_w1c1_case_ids: [...mappedW1c1Cases].sort(),
  cargo_test_ids_executed: [...cargoResults.keys()].sort(),
  public_capacity_override: 0,
  wasm_capacity_override: 0,
  workbench_capacity_override: 0,
  bounded_wait_public_frontdoor: "OPEN",
}));
