#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function sha256(text) {
  return crypto.createHash("sha256").update(text, "utf8").digest("hex");
}

const root = path.resolve(import.meta.dirname, "..");
const contract = JSON.parse(await fs.readFile(
  path.join(root, "pack/seamgrim_canonical_view_t5h_readiness_audit_v1/contract.detjson"),
  "utf8",
));
const sourceWasmDir = String(process.env.DDN_T5F_SOURCE_WASM_DIR ?? "").trim();
assert(sourceWasmDir, "DDN_T5F_SOURCE_WASM_DIR is required");

const runners = [
  ["T5B_GOLDEN", "tests/seamgrim_console_graph_canonical_view_payload_runner.mjs"],
  ["T5B_MUTATION", "tests/seamgrim_console_graph_canonical_view_payload_mutation_runner.mjs"],
  ["T5C_GOLDEN", "tests/seamgrim_space2d_canonical_view_payload_runner.mjs"],
  ["T5C_MUTATION", "tests/seamgrim_space2d_canonical_view_payload_mutation_runner.mjs"],
  ["T5D_GOLDEN", "tests/seamgrim_table_text_structure_canonical_view_payload_runner.mjs"],
  ["T5D_MUTATION", "tests/seamgrim_table_text_structure_canonical_view_payload_mutation_runner.mjs"],
  ["T5E_GOLDEN", "tests/seamgrim_console_grid_grid2d_canonical_view_payload_runner.mjs"],
  ["T5E_MUTATION", "tests/seamgrim_console_grid_grid2d_canonical_view_payload_mutation_runner.mjs"],
  ["T5F_GOLDEN", "tests/seamgrim_workbench_four_family_canonical_view_payload_runner.mjs"],
  ["T5F_MUTATION", "tests/seamgrim_workbench_four_family_canonical_view_payload_mutation_runner.mjs"],
  ["T5G_GOLDEN", "tests/seamgrim_scene_curriculum_canonical_view_payload_runner.mjs"],
  ["T5G_MUTATION", "tests/seamgrim_scene_curriculum_canonical_view_payload_mutation_runner.mjs"],
  ["T5H_STATUS_READINESS", "tests/seamgrim_canonical_view_t5h_readiness_audit_runner.mjs"],
  ["T5H_STATUS_MUTATION", "tests/seamgrim_canonical_view_t5h_status_totality_mutation_runner.mjs"],
];
assert(
  JSON.stringify(runners.map(([id]) => id)) === JSON.stringify(contract.aggregate_runner_ids),
  "aggregate runner inventory drift",
);

const results = [];
for (const [id, relativePath] of runners) {
  const run = spawnSync(process.execPath, [relativePath], {
    cwd: root,
    env: { ...process.env, DDN_T5F_SOURCE_WASM_DIR: sourceWasmDir },
    encoding: "utf8",
    windowsHide: true,
  });
  const stdout = String(run.stdout ?? "");
  const stderr = String(run.stderr ?? "");
  assert(
    run.status === 0,
    `${id} failed with ${run.status}\nSTDOUT:\n${stdout}\nSTDERR:\n${stderr}`,
  );
  results.push({
    id,
    runner: relativePath,
    exit: run.status,
    stdout_sha256: sha256(stdout),
    stderr_bytes: Buffer.byteLength(stderr, "utf8"),
  });
}

console.log(JSON.stringify({
  status: "PASS",
  audit_result: "T5H_CROSS_FAMILY_EXECUTABLE_READINESS_BASELINE",
  runner_count: results.length,
  golden_runner_count: results.filter((row) => row.id.endsWith("_GOLDEN")).length,
  mutation_runner_count: results.filter((row) => row.id.endsWith("_MUTATION")).length,
  status_readiness_runner_count: results.filter((row) => row.id.startsWith("T5H_STATUS_")).length,
  browser_claim: "NOT_RUN_BY_THIS_AGGREGATE",
  product_change: 1,
  results,
}));
