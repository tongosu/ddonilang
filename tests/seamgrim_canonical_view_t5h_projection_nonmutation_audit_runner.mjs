#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function deepFreeze(value, seen = new Set()) {
  if (!value || typeof value !== "object" || seen.has(value)) return value;
  seen.add(value);
  Object.values(value).forEach((child) => deepFreeze(child, seen));
  return Object.freeze(value);
}

function collectRefs(value, refs = new Set()) {
  if (!value || typeof value !== "object" || refs.has(value)) return refs;
  refs.add(value);
  Object.values(value).forEach((child) => collectRefs(child, refs));
  return refs;
}

function outputAliasesInput(value, inputRefs, seen = new Set()) {
  if (!value || typeof value !== "object" || seen.has(value)) return false;
  if (inputRefs.has(value)) return true;
  seen.add(value);
  return Object.values(value).some((child) => outputAliasesInput(child, inputRefs, seen));
}

const root = path.resolve(import.meta.dirname, "..");
const contract = JSON.parse(await fs.readFile(
  path.join(root, "pack/seamgrim_canonical_view_t5h_projection_nonmutation_audit_v1/contract.detjson"),
  "utf8",
));
assert(contract.schema === "seamgrim.canonical_view.t5h_projection_nonmutation_audit.v1", "projection nonmutation contract schema mismatch");

const modulePath = String(process.env.DDN_T5H_CANONICAL_OWNER_MODULE_PATH ?? "").trim()
  || path.join(root, contract.canonical_owner);
const mod = await import(`${pathToFileURL(modulePath).href}?audit=${Date.now()}`);

const identity = {
  state_hash: "sha256:t5h-projection-nonmutation",
  source_hash: "sha256:t5h-projection-source",
  result_identity: "result:t5h-projection-nonmutation",
  observation_projection_identity: "observation:t5h-projection-nonmutation",
  view_hash: "sha256:t5h-projection-view",
  tick_id: 7,
};
const provenance = {
  status: "available",
  source: "t5h_projection_nonmutation_fixture",
  provenance: "native_authoritative_structured_input",
  classification: "AUTHORITATIVE_STRUCTURED",
};

const graphProjection = {
  ...provenance,
  graph: { schema: "seamgrim.graph.v0", series: [{ id: "position.x", points: [{ x: 0, y: 0 }, { x: 1, y: 8 }] }] },
  series_identities: ["position.x"],
};
const graphPayload = mod.buildCanonicalGraphViewPayload({ identity, projection: graphProjection });
assert(graphPayload.status === "available", "graph prerequisite unavailable");

const consoleProjection = {
  ...provenance,
  entries: [
    { tick: 7, line_no: 1, kind: "output", text: "준비", output_identity: "output:1", provenance: "runtime" },
    { tick: 7, line_no: 2, kind: "output", text: "위치 8", output_identity: "output:2", provenance: "runtime" },
  ],
};
const consolePayload = mod.buildCanonicalConsoleViewPayload({ identity, projection: consoleProjection });
assert(consolePayload.status === "available", "console prerequisite unavailable");

const resultSchemas = {
  line: "seamgrim.workbench.line_result.v1_candidate",
  pendulum: "seamgrim.workbench.pendulum_result.v1_candidate",
  projectile: "seamgrim.workbench.projectile_result.v1_candidate",
  grid: "seamgrim.workbench.grid_result.v1_candidate",
};
const workbenchContent = {
  line: { current: { x: 1, y: 2 }, equation: { a: 1, b: 1 } },
  pendulum: { current: { theta: 0, omega: 0, length: 1, energy: null, kinetic_energy: null, potential_energy: null, physical_bob: { x: 0, y: -1 }, normalized_bob: { x: 0, y: -1 } } },
  projectile: { current: { t: 0, x: 0, y: 0, vx: 1, vy: 1 } },
  grid: { position: { x: 0, y: 0 }, dimensions: { columns: 2, rows: 2 }, space2d: { schema: "seamgrim.space2d.v0", drawlist: [{ kind: "point", x: 0, y: 0 }] } },
};
function workbenchProjection(family) {
  return {
    ...provenance,
    source: "ddonirang_tool.wasm_runtime",
    result_set: {
      schema: "seamgrim.workbench.authoritative_result_set.v1_candidate",
      producer: "ddonirang_tool.wasm_runtime",
      provenance: "native_authoritative_structured_result",
      results: {
        [family]: {
          schema: resultSchemas[family],
          family,
          status: "available",
          producer: "ddonirang_tool.wasm_runtime",
          provenance: "native_authoritative_structured_result",
          content: workbenchContent[family],
        },
      },
    },
  };
}

const sceneProjection = {
  schema: mod.SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
  status: "available",
  source: "scene_summary_semantic_projection",
  provenance: "canonical_view_payload_projection",
  primary_family: "graph",
  required_families: ["graph"],
  primary_payload: graphPayload,
};
const curriculumProjection = {
  schema: mod.CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
  status: "available",
  source: "curriculum_semantic_scene_projection",
  provenance: "validated_curriculum_card_and_committed_bindings",
  scene_id: "scene:t5h",
  lesson_id: "lesson:t5h",
  world_id: "world:t5h",
  primary_view: "graph",
  primary_view_family: "graph",
  primary_view_payload: graphPayload,
  concept_labels: ["위치"],
  anchors: [
    { anchor_id: "control.speed", kind: "control", name: "속도", binding: { source: "slider_panel", resolved: true } },
    { anchor_id: "observation.position", kind: "observation", name: "위치", binding: { source: "committed_observation_payload", resolved: true } },
  ],
};

const cases = [
  { family: "console", input: { identity, projection: consoleProjection }, invoke: (input) => mod.buildCanonicalConsoleViewPayload(input) },
  { family: "console-grid", input: { identity, consolePayload }, invoke: (input) => mod.buildCanonicalConsoleGridViewPayload(input) },
  { family: "graph", input: { identity, projection: graphProjection }, invoke: (input) => mod.buildCanonicalGraphViewPayload(input) },
  { family: "space2d", input: { identity, projection: { ...provenance, space2d: { schema: "seamgrim.space2d.v0", points: [{ id: "p", x: 0, y: 0 }], shapes: [] } } }, invoke: (input) => mod.buildCanonicalSpace2dViewPayload(input) },
  { family: "table", input: { identity, projection: { ...provenance, table: { schema: "seamgrim.table.v0", columns: [{ key: "x", label: "x", type: "number" }], rows: [{ x: 1 }] } } }, invoke: (input) => mod.buildCanonicalTableViewPayload(input) },
  { family: "text", input: { identity, projection: { ...provenance, text: { schema: "seamgrim.text.v0", content: "설명", format: "plain" } } }, invoke: (input) => mod.buildCanonicalTextViewPayload(input) },
  { family: "structure", input: { identity, projection: { ...provenance, structure: { schema: "seamgrim.structure.v0", nodes: [{ id: "n1", label: "N" }], edges: [] } } }, invoke: (input) => mod.buildCanonicalStructureViewPayload(input) },
  { family: "grid2d", input: { identity, projection: { ...provenance, grid2d: { schema: "seamgrim.nurimaker.grid_state.v1", gridW: 2, gridH: 2, tiles: [[0, 1], [1, 0]] }, actors: [{ id: "player", row: 1, col: 0, label: "P" }] } }, invoke: (input) => mod.buildCanonicalGrid2dViewPayload(input) },
  ...["line", "pendulum", "projectile", "grid"].map((family) => ({
    family: `workbench-${family}`,
    input: { identity, projection: workbenchProjection(family), family },
    invoke: (input) => mod.buildCanonicalWorkbenchViewPayload(input),
  })),
  { family: "scene-summary", input: { identity, projection: sceneProjection }, invoke: (input) => mod.buildCanonicalSceneSummaryViewPayload(input) },
  { family: "curriculum-semantic-scene", input: { identity, projection: curriculumProjection }, invoke: (input) => mod.buildCanonicalCurriculumSemanticSceneViewPayload(input) },
];
assert(JSON.stringify(cases.map((entry) => entry.family)) === JSON.stringify(contract.active_families), "projection nonmutation family inventory drift");

let storageReads = 0;
let storageWrites = 0;
let networkCalls = 0;
const priorDescriptors = new Map();
for (const key of ["localStorage", "sessionStorage", "fetch"]) priorDescriptors.set(key, Object.getOwnPropertyDescriptor(globalThis, key));
const storageProbe = Object.freeze({
  getItem() { storageReads += 1; return null; },
  setItem() { storageWrites += 1; },
  removeItem() { storageWrites += 1; },
  clear() { storageWrites += 1; },
});
Object.defineProperty(globalThis, "localStorage", { configurable: true, value: storageProbe });
Object.defineProperty(globalThis, "sessionStorage", { configurable: true, value: storageProbe });
Object.defineProperty(globalThis, "fetch", { configurable: true, value: () => { networkCalls += 1; throw new Error("network forbidden"); } });

const rows = [];
try {
  for (const entry of cases) {
    const before = mod.stableSemanticJson(entry.input);
    const inputRefs = collectRefs(entry.input);
    deepFreeze(entry.input);
    const output = entry.invoke(entry.input);
    assert(output?.status === "available", `${entry.family}: fixture unavailable (${output?.status})`);
    assert(output?.identity?.state_hash === identity.state_hash, `${entry.family}: state hash identity drift`);
    assert(mod.stableSemanticJson(entry.input) === before, `${entry.family}: input byte drift`);
    assert(!outputAliasesInput(output, inputRefs), `${entry.family}: output aliases authoritative input`);
    rows.push({ family: entry.family, status: output.status, input_byte_drift: 0, input_alias_count: 0, state_hash_identity_drift: 0 });
  }
} finally {
  for (const [key, descriptor] of priorDescriptors) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor);
    else delete globalThis[key];
  }
}

assert(storageReads === 0, "canonical projection read storage");
assert(storageWrites === 0, "canonical projection wrote storage");
assert(networkCalls === 0, "canonical projection called network");

const validateOnly = String(process.env.DDN_T5H_PROJECTION_NONMUTATION_VALIDATE_ONLY ?? "") === "1";
let existingRunnerCount = 0;
if (!validateOnly) {
  const run = spawnSync(process.execPath, [contract.required_existing_runner], { cwd: root, env: process.env, encoding: "utf8", windowsHide: true, timeout: 120_000 });
  assert(run.status === 0, `${contract.required_existing_runner} failed with ${run.status}\n${run.stdout ?? ""}\n${run.stderr ?? ""}`);
  existingRunnerCount = 1;
}

console.log(JSON.stringify({
  status: "PASS",
  audit_result: "T5H_CANONICAL_PROJECTION_NONMUTATION_REPRODUCED",
  family_count: rows.length,
  available_count: rows.length,
  input_byte_drift: 0,
  input_object_aliases_in_output: 0,
  state_hash_identity_drift: 0,
  storage_reads: storageReads,
  storage_writes: storageWrites,
  network_calls: networkCalls,
  existing_runner_count: existingRunnerCount,
  validate_only: validateOnly,
  product_change: 0,
}));
