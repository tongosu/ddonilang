#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function exactJson(value) {
  return JSON.stringify(value);
}

const root = path.resolve(import.meta.dirname, "..");
const packRoot = path.join(root, "pack/seamgrim_canonical_view_t5h_readiness_audit_v1");
const matrixPath = String(process.env.DDN_T5H_SURFACE_EVIDENCE_MATRIX ?? "").trim()
  || path.join(packRoot, "surface_evidence_matrix.detjson");
const [contract, matrix] = await Promise.all([
  fs.readFile(path.join(packRoot, "contract.detjson"), "utf8").then(JSON.parse),
  fs.readFile(matrixPath, "utf8").then(JSON.parse),
]);

assert(matrix.schema === "seamgrim.canonical_view.t5h_surface_evidence_matrix.v1", "surface evidence matrix schema mismatch");
assert(matrix.canonical_owner === contract.canonical_owner, "surface evidence canonical owner drift");
assert(exactJson(matrix.rows.map((row) => row.key)) === exactJson(contract.active_keys), "surface evidence key inventory drift");
assert(exactJson(matrix.rows.map((row) => row.family)) === exactJson(contract.active_families), "surface evidence family inventory drift");

const executables = new Map(matrix.executables.map((entry) => [entry.id, entry]));
assert(executables.size === matrix.executables.length, "duplicate surface evidence executable id");
const allowed = new Set(matrix.allowed_classifications);
const counts = Object.fromEntries(matrix.allowed_classifications.map((classification) => [classification, 0]));
let cellCount = 0;

for (const row of matrix.rows) {
  const expectedSurfaces = contract.consumer_matrix[row.key];
  assert(Array.isArray(expectedSurfaces), `${row.key}: missing contract consumer row`);
  assert(exactJson(row.evidence.map((entry) => entry.surface)) === exactJson(expectedSurfaces), `${row.key}: evidence surface inventory drift`);
  for (const evidence of row.evidence) {
    cellCount += 1;
    assert(allowed.has(evidence.classification), `${row.key}/${evidence.surface}: unknown evidence classification`);
    counts[evidence.classification] += 1;
    const executable = executables.get(evidence.executable_id);
    assert(executable, `${row.key}/${evidence.surface}: missing evidence executable`);
    const source = await fs.readFile(path.join(root, executable.runner), "utf8");
    assert(source.includes(evidence.required_marker), `${row.key}/${evidence.surface}: evidence marker missing`);
    assert(evidence.surface !== "playground", `${row.key}: legacy Playground leaked into current-product parity`);
  }
}

assert(cellCount === matrix.claim_boundaries.matrix_cell_count, "surface evidence cell count drift");
assert(exactJson(counts) === exactJson(matrix.expected_classification_counts), "surface evidence classification count drift");
assert(matrix.claim_boundaries.direct_or_serialization_executable_cells === 23, "direct/serialization cell claim drift");
assert(matrix.claim_boundaries.presentation_port_cells === counts.PRESENTATION_PORT_PARITY, "presentation-port count claim drift");
assert(matrix.claim_boundaries.t5h_actual_surface_parity_closed === true, "T5H actual-surface parity not closed");
assert(matrix.claim_boundaries.t5g_plain_pass_bound === true, "T5G plain-PASS binding lost");
assert(matrix.claim_boundaries.playground_option_b_bound === true, "Playground OPTION_B binding lost");

assert(matrix.compatibility_evidence.surface === contract.legacy_compatibility_matrix.surface, "compatibility surface drift");
assert(matrix.compatibility_evidence.classification === "LEGACY_COMPATIBILITY_ONLY", "compatibility classification drift");
assert(matrix.compatibility_evidence.owner_decision === "OPTION_B_PLAYGROUND_IS_LEGACY_COMPATIBILITY_ONLY", "compatibility Owner decision drift");
assert(matrix.compatibility_evidence.full_product_surface_identity === false, "compatibility surface silently promoted");
assert(exactJson(matrix.compatibility_evidence.keys) === exactJson(contract.legacy_compatibility_matrix.keys), "compatibility key inventory drift");
assert(matrix.claim_boundaries.legacy_compatibility_cells === matrix.compatibility_evidence.keys.length, "compatibility cell count drift");

assert(matrix.non_consumer_evidence.surface === "run", "non-consumer surface drift");
assert(exactJson(matrix.non_consumer_evidence.keys) === exactJson(contract.non_consumer_matrix.run), "Run non-consumer inventory drift");
const nonConsumerSource = await fs.readFile(path.join(root, matrix.non_consumer_evidence.source), "utf8");
for (const marker of matrix.non_consumer_evidence.forbidden_family_specific_consumer_markers) {
  assert(!nonConsumerSource.includes(marker), `Run family-specific consumer unexpectedly exists: ${marker}`);
}
assert(matrix.claim_boundaries.excluded_non_consumer_cells === matrix.non_consumer_evidence.keys.length, "excluded non-consumer count drift");

const validateOnly = String(process.env.DDN_T5H_SURFACE_AUDIT_VALIDATE_ONLY ?? "") === "1";
const executed = [];
if (!validateOnly) {
  const sourceWasmDir = String(process.env.DDN_T5F_SOURCE_WASM_DIR ?? "").trim();
  const playwrightModules = String(process.env.DDN_LOCAL_PLAYWRIGHT_NODE_MODULES ?? "").trim();
  for (const executable of matrix.executables) {
    if (executable.requires_source_wasm) assert(sourceWasmDir, `${executable.id}: DDN_T5F_SOURCE_WASM_DIR is required`);
    if (executable.browser) assert(playwrightModules, `${executable.id}: DDN_LOCAL_PLAYWRIGHT_NODE_MODULES is required`);
    const args = executable.browser
      ? ["--experimental-loader", "./tests/local_playwright_module_loader.mjs", executable.runner]
      : [executable.runner];
    const run = spawnSync(process.execPath, args, {
      cwd: root,
      env: { ...process.env, DDN_T5F_SOURCE_WASM_DIR: sourceWasmDir, DDN_LOCAL_PLAYWRIGHT_NODE_MODULES: playwrightModules },
      encoding: "utf8",
      windowsHide: true,
      timeout: 180_000,
      maxBuffer: 16 * 1024 * 1024,
    });
    assert(run.status === 0, `${executable.id} failed with ${run.status}\n${run.stdout ?? ""}\n${run.stderr ?? ""}`);
    executed.push(executable.id);
  }
}

console.log(JSON.stringify({
  status: "PASS",
  audit_result: "T5H_ACTUAL_SURFACE_PARITY_AND_OPTION_B_BOUNDARY_CLOSED",
  matrix_cell_count: cellCount,
  classification_counts: counts,
  executed_runner_count: executed.length,
  executed_runner_ids: executed,
  validate_only: validateOnly,
  legacy_compatibility_cell_count: matrix.compatibility_evidence.keys.length,
  excluded_non_consumer_cell_count: matrix.non_consumer_evidence.keys.length,
  t5h_actual_surface_parity_closed: true,
  t5g_plain_pass_bound: true,
  product_change: 1,
}));
