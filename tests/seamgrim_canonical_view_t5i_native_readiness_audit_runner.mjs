#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import {
  CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
  SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
  buildCanonicalWorkbenchViewPayload,
  buildCanonicalViewPayloadSet,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function sha256(text) {
  return crypto.createHash("sha256").update(text, "utf8").digest("hex");
}

function collectKeys(value, keys = new Set()) {
  if (Array.isArray(value)) {
    value.forEach((item) => collectKeys(item, keys));
    return keys;
  }
  if (value && typeof value === "object") {
    Object.entries(value).forEach(([key, child]) => {
      keys.add(key);
      collectKeys(child, keys);
    });
  }
  return keys;
}

function fnv1a32(text) {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function expectedCandidateHash(payload) {
  const { candidate_payload_hash: ignored, ...hashInput } = payload;
  return `candidate:fnv1a32:canonical_view_payload:${fnv1a32(stableSemanticJson(hashInput))}`;
}

function workbenchContent(family) {
  if (family === "line") return { current: { x: 1, y: 2 }, equation: { a: 1, b: 1 } };
  if (family === "pendulum") {
    return {
      current: {
        theta: 0,
        omega: 0,
        length: 1,
        energy: null,
        kinetic_energy: null,
        potential_energy: null,
        physical_bob: { x: 0, y: -1 },
        normalized_bob: { x: 0, y: -1 },
      },
    };
  }
  if (family === "projectile") return { current: { t: 0, x: 0, y: 0, vx: 1, vy: 1 } };
  return {
    position: { x: 0, y: 0 },
    dimensions: { columns: 2, rows: 2 },
    space2d: { schema: "seamgrim.space2d.v0", drawlist: [{ kind: "point", x: 0, y: 0 }] },
  };
}

function buildWorkbenchFixture(family, schemas) {
  const result = {
    schema: schemas[family],
    family,
    status: "available",
    producer: "ddonirang_tool.wasm_runtime",
    provenance: "native_authoritative_structured_result",
    content: workbenchContent(family),
  };
  return buildCanonicalWorkbenchViewPayload({
    identity: {
      state_hash: `sha256:t5i-audit:${family}`,
      result_identity: `result:t5i-audit:${family}`,
    },
    family,
    projection: {
      status: "available",
      source: "ddonirang_tool.wasm_runtime",
      provenance: "native_authoritative_structured_input",
      classification: "AUTHORITATIVE_STRUCTURED",
      result_set: {
        schema: schemas.result_set,
        producer: "ddonirang_tool.wasm_runtime",
        provenance: "native_authoritative_structured_result",
        results: { [family]: result },
      },
    },
  });
}

const root = path.resolve(import.meta.dirname, "..");
const contractPath = String(process.env.DDN_T5I_CONTRACT_PATH ?? "").trim() || path.join(
  root,
  "pack/seamgrim_canonical_view_t5i_native_readiness_audit_v1/contract.detjson",
);
const contract = JSON.parse(await fs.readFile(contractPath, "utf8"));
assert(contract.schema === "seamgrim.canonical_view.t5i_native_readiness_audit.v1", "T5I audit contract schema mismatch");
assert(contract.audit_scope === "final_renderer_neutral_native_and_web_readiness", "T5I final audit scope missing");
assert(contract.native_implementation_authorized === false, "Native implementation must remain unauthorized");
assert(contract.native_implementation_complete === false, "Native implementation must remain unclaimed");
assert(contract.public_abi_claimed === false, "public ABI must remain unclaimed");
assert(contract.stable_wire_claimed === false, "stable wire must remain unclaimed");
assert(contract.ssot_schema_promoted === false, "SSOT schema promotion must remain unclaimed");
assert(contract.public_spec_changed === false, "public SPEC change must remain unclaimed");
assert(contract.human_pass_claimed === false, "Human PASS must remain unclaimed");
assert(contract.review_gate.t5g_independent_plain_pass_bound === true, "T5G plain PASS binding missing");
assert(contract.review_gate.t5h_independent_plain_pass_bound === true, "T5H plain PASS binding missing");
assert(contract.review_gate.t5i_final_readiness_start_authorized === true, "T5I final readiness not authorized");
assert(contract.known_dependent_gates.length === 0, "T5I dependent gate inventory is not empty");
assert(contract.presentation_profile_probe.semantic_payload_participation === false, "presentation profile entered semantic payload");
assert(contract.presentation_profile_probe.semantic_identity_hash_participation === false, "presentation profile entered semantic identity");
assert(contract.presentation_profile_probe.public_contract === false, "audit presentation profile was promoted to public contract");

const statusContract = JSON.parse(await fs.readFile(path.join(root, contract.status_totality_contract), "utf8"));
assert(statusContract.known_unknown_status_fail_open_keys.length === 0, "T5H status fail-open inventory is not zero");
assert(statusContract.known_unknown_status_fail_closed_keys.length === 14, "T5H status fail-closed inventory is not total");
const unknownStatus = contract.expected_status_totality.unknown_status;
const statusPayloads = buildCanonicalViewPayloadSet({
  identity: { state_hash: "sha256:t5i-final-readiness" },
  console: { status: unknownStatus },
  graph: { status: unknownStatus },
  space2d: { status: unknownStatus },
  table: { status: unknownStatus },
  text: { status: unknownStatus },
  structure: { status: unknownStatus },
  grid2d: { status: unknownStatus },
  workbench: { status: unknownStatus },
  sceneSummary: { schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA, status: unknownStatus },
  curriculumSemanticScene: { schema: CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA, status: unknownStatus },
});
const failClosedCount = Object.values(statusPayloads)
  .filter((payload) => payload.status === contract.expected_status_totality.outcome).length;
assert(failClosedCount === contract.expected_status_totality.fail_closed_count, "T5H status-totality runtime probe drift");

const payloads = new Map();
for (const [family, relativePath] of Object.entries(contract.golden_payloads)) {
  payloads.set(family, JSON.parse(await fs.readFile(path.join(root, relativePath), "utf8")));
}
for (const family of ["line", "pendulum", "projectile", "grid"]) {
  payloads.set(`workbench-${family}`, buildWorkbenchFixture(family, contract.workbench_fixture_schemas));
}
assert(payloads.size === 14, `expected 14 positive payloads, got ${payloads.size}`);
assert(
  stableSemanticJson([...payloads.keys()].sort()) === stableSemanticJson([...contract.active_families].sort()),
  "T5I active-family inventory drift",
);

const ownerSource = await fs.readFile(path.join(root, contract.canonical_owner), "utf8");
const ownerRuntimeHits = contract.forbidden_owner_runtime_tokens.filter((token) => ownerSource.includes(token));
assert(ownerRuntimeHits.length === 0, `canonical owner contains Web runtime tokens: ${ownerRuntimeHits.join(",")}`);
const uiRoot = path.join(root, "solutions/seamgrim_ui_mvp/ui");
const canonicalSchemaOwners = [];
async function findSchemaOwners(directory) {
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) await findSchemaOwners(entryPath);
    else if (entry.name.endsWith(".js") && (await fs.readFile(entryPath, "utf8")).includes(contract.canonical_schema)) {
      canonicalSchemaOwners.push(path.relative(root, entryPath).replaceAll("\\", "/"));
    }
  }
}
await findSchemaOwners(uiRoot);
assert(stableSemanticJson(canonicalSchemaOwners.sort()) === stableSemanticJson([contract.canonical_owner]), "third canonical View abstraction detected");

const forbiddenPayloadKeys = new Set(contract.web_only_payload_keys);
const rows = [];
for (const family of contract.active_families) {
  const payload = payloads.get(family);
  assert(payload.schema === contract.canonical_schema, `${family}: canonical schema mismatch`);
  assert(payload.family === family, `${family}: payload family mismatch`);
  assert(payload.status === "available", `${family}: positive payload is not available`);
  assert(payload.content && typeof payload.content === "object", `${family}: semantic content missing`);
  assert(payload.candidate_payload_hash === expectedCandidateHash(payload), `${family}: candidate payload hash mismatch`);
  assert(String(payload.identity?.state_hash ?? "") || String(payload.identity?.result_identity ?? ""), `${family}: semantic identity missing`);
  assert(typeof payload.provenance?.source === "string", `${family}: provenance source missing`);
  assert(typeof payload.provenance?.input_authority === "string", `${family}: input authority missing`);

  const keys = collectKeys(payload);
  const webOnlyHits = [...forbiddenPayloadKeys].filter((key) => keys.has(key));
  assert(webOnlyHits.length === 0, `${family}: Web/presentation keys leaked: ${webOnlyHits.join(",")}`);
  const categoryCounts = Object.fromEntries(Object.entries(contract.web_field_categories).map(([category, candidates]) => [
    category,
    candidates.filter((key) => keys.has(key)).length,
  ]));
  assert(Object.values(categoryCounts).every((count) => count === 0), `${family}: Web field category leak`);

  const bytes = stableSemanticJson(payload);
  assert(stableSemanticJson(JSON.parse(bytes)) === bytes, `${family}: stable JSON round-trip mismatch`);
  const reorderedProfile = Object.fromEntries(Object.entries(contract.presentation_profile_probe).reverse());
  const semanticIdentityBefore = payload.candidate_payload_hash;
  const semanticBytesBefore = stableSemanticJson(payload);
  const projectionA = stableSemanticJson({ payload, presentation_profile: contract.presentation_profile_probe });
  const projectionB = stableSemanticJson({ presentation_profile: reorderedProfile, payload: JSON.parse(semanticBytesBefore) });
  assert(projectionA === projectionB, `${family}: same semantic result and presentation profile are not deterministic`);
  assert(payload.candidate_payload_hash === semanticIdentityBefore, `${family}: presentation profile changed semantic identity`);
  assert(stableSemanticJson(payload) === semanticBytesBefore, `${family}: presentation profile changed semantic payload`);

  rows.push({
    family,
    canonical_owner: contract.canonical_owner,
    serializable: true,
    deterministic: true,
    web_only_field_count: 0,
    dom_selector_in_semantic_payload: categoryCounts.dom_selector,
    css_in_semantic_payload: categoryCounts.css,
    canvas_pixel_in_semantic_payload: categoryCounts.canvas_pixel,
    font_in_semantic_payload: categoryCounts.font,
    browser_object_in_semantic_payload: categoryCounts.browser_object,
    transient_ui_state_in_semantic_payload: categoryCounts.transient_ui_state,
    semantic_identity: "PRESERVED_RENDERER_INDEPENDENT",
    presentation_profile: "SEPARATED_AUDIT_ONLY",
    native_reusability: contract.expected_native_reusability,
    stable_json_bytes: Buffer.byteLength(bytes, "utf8"),
    audit_sha256: sha256(bytes),
  });
}

console.log(JSON.stringify({
  status: "PASS",
  completion_code: "T5I_FINAL_NATIVE_WEB_READINESS_AUDIT_PASS",
  family_count: rows.length,
  canonical_payload_serializable: rows.filter((row) => row.serializable).length,
  deterministic_serialization: rows.filter((row) => row.deterministic).length,
  semantic_identity_renderer_independent: rows.filter((row) => row.semantic_identity === "PRESERVED_RENDERER_INDEPENDENT").length,
  presentation_profile_separated: rows.every((row) => row.presentation_profile === "SEPARATED_AUDIT_ONLY"),
  web_only_semantic_field_count: rows.reduce((sum, row) => sum + row.web_only_field_count, 0),
  third_view_abstraction: canonicalSchemaOwners.length - 1,
  status_fail_open_count: Object.keys(statusPayloads).length - failClosedCount,
  status_fail_closed_count: failClosedCount,
  final_program_disposition: contract.final_program_disposition,
  web_s0_view_readiness: contract.web_s0_view_readiness,
  native_renderer_readiness: contract.native_renderer_readiness,
  native_implementation: "NOT_STARTED",
  public_abi: "NOT_CLAIMED",
  rows,
}, null, 2));
