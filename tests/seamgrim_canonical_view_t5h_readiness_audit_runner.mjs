#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function exactJson(value) {
  return JSON.stringify(value);
}

const root = path.resolve(import.meta.dirname, "..");
const canonicalOwnerPath = String(process.env.DDN_T5H_CANONICAL_OWNER_MODULE_PATH ?? "").trim()
  || path.join(root, "solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js");
const {
  CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
  SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
  buildCanonicalViewPayloadSet,
} = await import(pathToFileURL(canonicalOwnerPath).href);
const contract = JSON.parse(await fs.readFile(
  path.join(root, "pack/seamgrim_canonical_view_t5h_readiness_audit_v1/contract.detjson"),
  "utf8",
));
assert(contract.schema === "seamgrim.canonical_view.t5h_readiness_audit.v1", "T5H audit contract schema mismatch");

function payloadsForStatus(status) {
  return buildCanonicalViewPayloadSet({
    identity: { state_hash: "sha256:t5h-readiness-audit" },
    console: { status },
    graph: { status },
    space2d: { status },
    table: { status },
    text: { status },
    structure: { status },
    grid2d: { status },
    workbench: { status },
    sceneSummary: {
      schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
      status,
    },
    curriculumSemanticScene: {
      schema: CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
      status,
    },
  });
}

const unknownStatus = String(contract.unknown_status_probe);
const payloads = payloadsForStatus(unknownStatus);

const actualKeys = Object.keys(payloads);
const actualFamilies = Object.values(payloads).map((payload) => String(payload.family));
assert(exactJson(actualKeys) === exactJson(contract.active_keys), "14-key canonical inventory drift");
assert(exactJson(actualFamilies) === exactJson(contract.active_families), "14-family canonical inventory drift");
assert(Object.keys(contract.consumer_matrix).length === contract.active_keys.length, "consumer matrix is not total");
assert(
  contract.active_keys.every((key) => Array.isArray(contract.consumer_matrix[key]) && contract.consumer_matrix[key].length > 0),
  "consumer matrix has an empty or missing row",
);

const statuses = Object.fromEntries(Object.entries(payloads).map(([key, payload]) => [key, String(payload.status)]));
const expectedUnknownOutcome = String(contract.expected_unknown_status_outcome);
const failOpenKeys = actualKeys.filter((key) => statuses[key] !== expectedUnknownOutcome);
const failClosedKeys = actualKeys.filter((key) => statuses[key] === expectedUnknownOutcome);
assert(
  exactJson(failOpenKeys) === exactJson(contract.known_unknown_status_fail_open_keys),
  `unknown-status fail-open inventory drift: ${exactJson(failOpenKeys)}`,
);
assert(
  exactJson(failClosedKeys) === exactJson(contract.known_unknown_status_fail_closed_keys),
  `unknown-status fail-closed inventory drift: ${exactJson(failClosedKeys)}`,
);
assert(
  failOpenKeys.every((key) => !contract.allowed_statuses.includes(statuses[key])),
  "recorded fail-open status unexpectedly belongs to the canonical status model",
);
assert(failOpenKeys.length === 0, "T5H status totality remains open");
assert(
  actualKeys.every((key) => payloads[key]?.diagnostic?.code === contract.unknown_status_diagnostic_codes[key]),
  "unknown-status candidate diagnostic inventory drift",
);

for (const expectedStatus of contract.propagated_failure_statuses) {
  const knownPayloads = payloadsForStatus(expectedStatus);
  assert(
    actualKeys.every((key) => knownPayloads[key]?.status === expectedStatus),
    `${expectedStatus}: accepted failure status propagation drift`,
  );
}

console.log(JSON.stringify({
  status: "PASS",
  audit_result: "T5H_STATUS_TOTALITY_CLOSED",
  active_key_count: actualKeys.length,
  registry_only_count: contract.registry_only_families.length,
  consumer_matrix_rows: Object.keys(contract.consumer_matrix).length,
  unknown_status_probe: unknownStatus,
  fail_open_count: failOpenKeys.length,
  fail_open_keys: failOpenKeys,
  fail_closed_count: failClosedKeys.length,
  fail_closed_keys: failClosedKeys,
  propagated_failure_status_count: contract.propagated_failure_statuses.length,
  product_change: 1,
}));
