#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { extractCanonicalViewAuthorityInputsFromState } from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
  SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
  buildCanonicalCurriculumSemanticSceneViewPayload,
  buildCanonicalSceneSummaryViewPayload,
  buildCanonicalViewPayloadSet,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";
import { buildSceneSummarySnapshot } from "../solutions/seamgrim_ui_mvp/ui/scene_summary_contract.js";
import { buildCurriculumSemanticSceneV0 } from "../solutions/seamgrim_ui_mvp/ui/curriculum_semantic_scene.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function prove(label, greenValue, mutate, invariant) {
  assert(invariant(greenValue), `${label}: initial GREEN failed`);
  const mutant = mutate(structuredClone(greenValue));
  assert(!invariant(mutant), `${label}: mutation did not turn RED`);
  assert(invariant(greenValue), `${label}: restored GREEN failed`);
  return { label, initial: "GREEN", mutated: "RED", restored: "GREEN" };
}

const root = process.cwd();
const state = JSON.parse(await fs.readFile(
  path.join(root, "pack/seamgrim_scene_curriculum_canonical_view_payload_v1/fixtures/authoritative_state.detjson"),
  "utf8",
));
const payloads = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state));
const identity = payloads.graph.identity;
const scene = buildSceneSummarySnapshot({
  lessonId: "mutation",
  requiredViews: ["graph", "table"],
  primaryView: "graph",
  canonicalViewPayloads: payloads,
  runtimeDerived: { canonicalViewPayloads: payloads },
});
const curriculum = buildCurriculumSemanticSceneV0({
  id: "mutation",
  curriculumCard: {
    lesson_id: "mutation",
    world_id: "world.mutation",
    primary_view: "graph",
    allowed_controls: ["a"],
    allowed_observations: ["b"],
    display: { core_concepts: ["mutation"] },
  },
}, {
  controlValues: { a: 1 },
  observationRows: [{ id: "b", value: 2 }],
  primaryViewMode: "graph",
  canonicalViewPayloads: payloads,
});

const results = [];
results.push(prove(
  "M1_SCENE_PRESENTATION_LEAKS_CANONICAL_IDENTITY",
  { canonical: scene.canonical_view_payload, presentation: scene.presentation_session },
  (value) => {
    value.canonical.content.scene_summary.browser_view_config = value.presentation.view;
    return value;
  },
  (value) => !stableSemanticJson(value.canonical).includes("browser_view_config"),
));

const ambiguous = buildSceneSummarySnapshot({
  lessonId: "ambiguous",
  requiredViews: ["graph", "table"],
  canonicalViewPayloads: payloads,
  runtimeDerived: { canonicalViewPayloads: payloads },
}).canonical_view_payload;
results.push(prove(
  "M2_SCENE_AMBIGUITY_PICK_FIRST",
  ambiguous,
  (value) => {
    value.status = "available";
    value.content = { scene_summary: { primary_family: "graph", primary_payload: payloads.graph } };
    return value;
  },
  (value) => value.status === "ambiguous" && value.content === null,
));

results.push(prove(
  "M3_CURRICULUM_DOM_SELECTOR_LEAKS_CANONICAL_IDENTITY",
  curriculum.canonical_view_payload,
  (value) => {
    value.content.curriculum_semantic_scene.anchors[0].target_selector = "#runtime-dom";
    return value;
  },
  (value) => !stableSemanticJson(value).includes("target_selector"),
));

const sceneProjection = {
  schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
  status: "available",
  primary_family: "graph",
  required_families: ["graph"],
  primary_payload: payloads.graph,
  source: "mutation",
};
results.push(prove(
  "M4_SCENE_PRIMARY_IDENTITY_MISMATCH",
  sceneProjection,
  (value) => {
    value.primary_payload.identity.state_hash = "sha256:renderer-reinvented";
    return value;
  },
  (value) => buildCanonicalSceneSummaryViewPayload({ identity, projection: value }).status === "available",
));

const curriculumProjection = {
  schema: CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
  status: "available",
  scene_id: "semantic_scene.mutation",
  lesson_id: "mutation",
  world_id: "world.mutation",
  primary_view: "graph",
  primary_view_family: "graph",
  primary_view_payload: payloads.graph,
  anchors: [{
    anchor_id: "control.a",
    kind: "control",
    name: "a",
    binding: { source: "slider_panel", resolved: true, value: 1 },
  }],
};
results.push(prove(
  "M5_CURRICULUM_PROJECTION_PREFIX_SCHEMA_ACCEPTANCE",
  curriculumProjection,
  (value) => {
    value.schema = `${CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA}.future`;
    return value;
  },
  (value) => buildCanonicalCurriculumSemanticSceneViewPayload({ identity, projection: value }).status === "available",
));

console.log(JSON.stringify({ status: "PASS", mutation_count: results.length, mutation_residue: 0, results }));
