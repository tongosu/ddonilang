#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function exactJson(value) {
  return JSON.stringify(value);
}

function sha256(text) {
  return crypto.createHash("sha256").update(text, "utf8").digest("hex");
}

function runNode(root, relativePath, { browser = false, env = {} } = {}) {
  const loader = "./tests/local_playwright_module_loader.mjs";
  const args = browser
    ? ["--experimental-loader", loader, relativePath]
    : [relativePath];
  const run = spawnSync(process.execPath, args, {
    cwd: root,
    env: { ...process.env, ...env },
    encoding: "utf8",
    windowsHide: true,
    timeout: 180_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  const stdout = String(run.stdout ?? "");
  const stderr = String(run.stderr ?? "");
  assert(
    run.status === 0,
    `${relativePath} failed with ${run.status}\nSTDOUT:\n${stdout}\nSTDERR:\n${stderr}`,
  );
  return {
    runner: relativePath,
    exit: run.status,
    stdout_sha256: sha256(stdout),
    stdout_bytes: Buffer.byteLength(stdout, "utf8"),
    stderr_bytes: Buffer.byteLength(stderr, "utf8"),
  };
}

const root = path.resolve(import.meta.dirname, "..");
const packRoot = path.join(root, "pack/seamgrim_canonical_view_t5h_readiness_audit_v1");
const [contract, matrix] = await Promise.all([
  fs.readFile(path.join(packRoot, "contract.detjson"), "utf8").then(JSON.parse),
  fs.readFile(path.join(packRoot, "browser_fallback_matrix.detjson"), "utf8").then(JSON.parse),
]);

assert(contract.schema === "seamgrim.canonical_view.t5h_readiness_audit.v1", "T5H base contract schema mismatch");
assert(matrix.schema === "seamgrim.canonical_view.t5h_browser_fallback_readiness.v1", "T5H browser/fallback matrix schema mismatch");
assert(matrix.canonical_owner === contract.canonical_owner, "T5H canonical owner drift");

const rows = Array.isArray(matrix.families) ? matrix.families : [];
assert(exactJson(rows.map((row) => row.key)) === exactJson(contract.active_keys), "T5H browser/fallback key inventory drift");
assert(exactJson(rows.map((row) => row.family)) === exactJson(contract.active_families), "T5H browser/fallback family inventory drift");

const browserGroups = new Map(matrix.browser_groups.map((group) => [group.id, group]));
const fallbackGroups = new Map(matrix.fallback_groups.map((group) => [group.id, group]));
assert(browserGroups.size === matrix.browser_groups.length, "duplicate browser group id");
assert(fallbackGroups.size === matrix.fallback_groups.length, "duplicate fallback group id");

const browserFamilies = matrix.browser_groups.flatMap((group) => group.families);
const fallbackFamilies = matrix.fallback_groups.flatMap((group) => group.families);
assert(browserFamilies.length === contract.active_families.length, "browser family coverage count drift");
assert(fallbackFamilies.length === contract.active_families.length, "fallback family coverage count drift");
assert(new Set(browserFamilies).size === contract.active_families.length, "browser family coverage duplicates or omissions");
assert(new Set(fallbackFamilies).size === contract.active_families.length, "fallback family coverage duplicates or omissions");
assert(contract.active_families.every((family) => browserFamilies.includes(family)), "browser family coverage is not total");
assert(contract.active_families.every((family) => fallbackFamilies.includes(family)), "fallback family coverage is not total");

let notApplicableCellCount = 0;
let compatibilityCellCount = 0;
for (const row of rows) {
  const browserGroup = browserGroups.get(row.browser_group);
  const fallbackGroup = fallbackGroups.get(row.fallback_group);
  assert(browserGroup?.families.includes(row.family), `${row.family}: browser group mismatch`);
  assert(fallbackGroup?.families.includes(row.family), `${row.family}: fallback group mismatch`);
  assert(
    exactJson(row.applicable_surfaces) === exactJson(contract.consumer_matrix[row.key]),
    `${row.family}: applicable surface inventory drift`,
  );
  const expectedNotApplicable = matrix.surface_universe.filter((surface) => !row.applicable_surfaces.includes(surface));
  assert(
    exactJson(row.not_applicable_surfaces) === exactJson(expectedNotApplicable),
    `${row.family}: NOT_APPLICABLE surface inventory drift`,
  );
  const expectedCompatibility = contract.legacy_compatibility_matrix.keys.includes(row.key)
    ? [contract.legacy_compatibility_matrix.surface]
    : [];
  assert(exactJson(row.compatibility_surfaces) === exactJson(expectedCompatibility), `${row.family}: compatibility surface drift`);
  assert(row.compatibility_surfaces.every((surface) => !row.applicable_surfaces.includes(surface)), `${row.family}: compatibility promoted to current product`);
  notApplicableCellCount += row.not_applicable_surfaces.length;
  compatibilityCellCount += row.compatibility_surfaces.length;
}

for (const group of matrix.fallback_groups) {
  assert(
    group.aggregate_runner_ids.every((id) => contract.aggregate_runner_ids.includes(id)),
    `${group.id}: fallback aggregate runner id missing`,
  );
  const source = await fs.readFile(path.join(root, group.evidence_runner), "utf8");
  assert(source.includes(group.required_source_marker), `${group.id}: fallback evidence marker missing`);
}

const sourceWasmDir = String(process.env.DDN_T5F_SOURCE_WASM_DIR ?? "").trim();
const playwrightModules = String(process.env.DDN_LOCAL_PLAYWRIGHT_NODE_MODULES ?? "").trim();
assert(sourceWasmDir, "DDN_T5F_SOURCE_WASM_DIR is required");
assert(playwrightModules, "DDN_LOCAL_PLAYWRIGHT_NODE_MODULES is required");
assert(await fs.stat(path.join(sourceWasmDir, "ddonirang_tool.js")).then((row) => row.isFile()).catch(() => false), "source-built WASM JS missing");
assert(await fs.stat(path.join(sourceWasmDir, "ddonirang_tool_bg.wasm")).then((row) => row.isFile()).catch(() => false), "source-built WASM binary missing");
assert(await fs.stat(path.join(playwrightModules, "playwright", "index.mjs")).then((row) => row.isFile()).catch(() => false), "local playwright package missing");

const aggregateResult = runNode(root, "tests/seamgrim_canonical_view_t5h_cross_family_readiness_runner.mjs", {
  env: { DDN_T5F_SOURCE_WASM_DIR: sourceWasmDir },
});
const browserResults = matrix.browser_groups.map((group) => ({
  id: group.id,
  families: group.families,
  ...runNode(root, group.runner, {
    browser: true,
    env: {
      DDN_LOCAL_PLAYWRIGHT_NODE_MODULES: playwrightModules,
      DDN_T5F_SOURCE_WASM_DIR: sourceWasmDir,
    },
  }),
}));
const supplementalBrowserResults = matrix.supplemental_browser_runners.map((group) => ({
  id: group.id,
  ...runNode(root, group.runner, {
    browser: true,
    env: { DDN_LOCAL_PLAYWRIGHT_NODE_MODULES: playwrightModules },
  }),
}));

console.log(JSON.stringify({
  status: "PASS",
  audit_result: "T5H_BROWSER_FALLBACK_AND_ACTUAL_SURFACE_READY",
  active_family_count: rows.length,
  browser_family_probe_count: new Set(browserFamilies).size,
  browser_group_count: browserResults.length,
  supplemental_browser_runner_count: supplementalBrowserResults.length,
  semantic_fallback_family_evidence_count: new Set(fallbackFamilies).size,
  semantic_fallback_group_count: fallbackGroups.size,
  explicit_not_applicable_surface_cell_count: notApplicableCellCount,
  compatibility_surface_cell_count: compatibilityCellCount,
  actual_product_surface_parity: matrix.claim_boundaries.actual_product_surface_parity,
  playground_compatibility_boundary: matrix.claim_boundaries.playground_compatibility_boundary,
  t5h_status_totality: matrix.claim_boundaries.t5h_status_totality,
  t5g_independent_pass: matrix.claim_boundaries.t5g_independent_pass,
  network_install: 0,
  product_change: 1,
  aggregate_result: aggregateResult,
  browser_results: browserResults,
  supplemental_browser_results: supplementalBrowserResults,
}));
