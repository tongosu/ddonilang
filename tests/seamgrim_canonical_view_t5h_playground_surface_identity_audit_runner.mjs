#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function exactJson(value) {
  return JSON.stringify(value);
}

async function collectFiles(root, extensions) {
  const result = [];
  async function walk(directory) {
    const entries = await fs.readdir(directory, { withFileTypes: true });
    for (const entry of entries) {
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) await walk(absolute);
      else if (extensions.has(path.extname(entry.name).toLowerCase())) result.push(absolute);
    }
  }
  await walk(root);
  return result.sort();
}

const root = path.resolve(import.meta.dirname, "..");
const packRoot = path.join(root, "pack/seamgrim_canonical_view_t5h_readiness_audit_v1");
const contractPath = String(process.env.DDN_T5H_CONTRACT_PATH ?? "").trim()
  || path.join(packRoot, "contract.detjson");
const auditPath = String(process.env.DDN_T5H_PLAYGROUND_AUDIT_PATH ?? "").trim()
  || path.join(packRoot, "playground_surface_identity_audit.detjson");
const [contract, audit] = await Promise.all([
  fs.readFile(contractPath, "utf8").then(JSON.parse),
  fs.readFile(auditPath, "utf8").then(JSON.parse),
]);
assert(audit.schema === "seamgrim.canonical_view.t5h_playground_surface_identity_audit.v1", "Playground identity audit schema mismatch");

const playHtmlPath = String(process.env.DDN_T5H_PLAY_HTML_PATH ?? "").trim()
  || path.join(root, audit.legacy_entrypoint);
const legacyModulePath = String(process.env.DDN_T5H_LEGACY_PLAYGROUND_MODULE_PATH ?? "").trim()
  || path.join(root, audit.legacy_module);
const [playHtml, appSource, legacyModuleSource] = await Promise.all([
  fs.readFile(playHtmlPath, "utf8"),
  fs.readFile(path.join(root, audit.current_shell_runtime), "utf8"),
  fs.readFile(legacyModulePath, "utf8"),
]);
for (const marker of audit.required_redirect_markers) {
  assert(playHtml.includes(marker), `legacy Playground redirect marker missing: ${marker}`);
}
for (const marker of audit.required_shell_markers) {
  assert(appSource.includes(marker), `Workbench Studio redirect-custody marker missing: ${marker}`);
}
for (const marker of audit.required_legacy_module_authority_markers) {
  assert(legacyModuleSource.includes(marker), `legacy compatibility authority marker missing: ${marker}`);
}

const uiRoot = path.join(root, "solutions/seamgrim_ui_mvp/ui");
const uiFiles = await collectFiles(uiRoot, new Set([".html", ".js"]));
const legacyModuleAbsolute = path.join(root, audit.legacy_module);
const activeImporters = [];
for (const file of uiFiles) {
  if (path.resolve(file) === path.resolve(legacyModuleAbsolute)) continue;
  const source = await fs.readFile(file, "utf8");
  if (source.includes("playground.js")) activeImporters.push(path.relative(root, file).replaceAll("\\", "/"));
}
assert(
  exactJson(activeImporters) === exactJson(audit.expected_active_entrypoint_importers),
  `Playground active importer inventory drift: ${exactJson(activeImporters)}`,
);

const playgroundConsumerKeys = contract.active_keys.filter(
  (key) => Array.isArray(contract.consumer_matrix[key]) && contract.consumer_matrix[key].includes("playground"),
);
assert(
  exactJson(playgroundConsumerKeys) === exactJson(audit.expected_consumer_matrix_playground_keys),
  `Playground consumer-matrix inventory drift: ${exactJson(playgroundConsumerKeys)}`,
);
assert(
  exactJson(contract.legacy_compatibility_matrix.keys) === exactJson(audit.expected_legacy_compatibility_playground_keys),
  "Playground legacy-compatibility inventory drift",
);
assert(contract.legacy_compatibility_matrix.surface === "playground", "Playground compatibility surface drift");
assert(contract.legacy_compatibility_matrix.identity === "legacy_compatibility_only", "Playground compatibility identity drift");
assert(contract.legacy_compatibility_matrix.full_product_surface_identity === false, "Playground full-surface identity silently promoted");

const projectionModes = {};
for (const evidence of audit.existing_projection_evidence) {
  const source = await fs.readFile(path.join(root, evidence.runner), "utf8");
  if (evidence.mode === "NO_EXPLICIT_PLAYGROUND_PROJECTION") {
    assert(!source.toLowerCase().includes(evidence.marker), `${evidence.phase}: unexpected explicit Playground projection appeared`);
  } else {
    assert(source.includes(evidence.marker), `${evidence.phase}: recorded Playground alias marker missing`);
  }
  projectionModes[evidence.phase] = evidence.mode;
}

assert(audit.claim_boundaries.owner_decision === "OPTION_B_PLAYGROUND_IS_LEGACY_COMPATIBILITY_ONLY", "Playground Owner decision drift");
assert(audit.claim_boundaries.standalone_playground_current_product_surface === "NO_LEGACY_COMPATIBILITY_ONLY", "standalone Playground claim boundary drift");
assert(audit.claim_boundaries.workbench_studio_is_playground_identity === "NO_DISTINCT_PRODUCT_SURFACE_IDENTITY", "Workbench/Playground identity drift");
assert(audit.claim_boundaries.compatibility_policy_change === "OWNER_APPROVED", "compatibility policy authority drift");

console.log(JSON.stringify({
  status: "PASS",
  audit_result: audit.expected_audit_result,
  legacy_entrypoint_redirects_to: audit.current_redirect_target,
  active_entrypoint_importer_count: activeImporters.length,
  consumer_matrix_playground_key_count: playgroundConsumerKeys.length,
  consumer_matrix_playground_keys: playgroundConsumerKeys,
  existing_projection_modes: projectionModes,
  legacy_compatibility_key_count: contract.legacy_compatibility_matrix.keys.length,
  owner_decision: audit.claim_boundaries.owner_decision,
  standalone_playground_current_product_surface: audit.claim_boundaries.standalone_playground_current_product_surface,
  workbench_studio_is_playground_identity: audit.claim_boundaries.workbench_studio_is_playground_identity,
  compatibility_policy_change: audit.claim_boundaries.compatibility_policy_change,
  product_semantic_change: 0,
  compatibility_presentation_change: 1,
}));
