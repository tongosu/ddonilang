#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { extractCanonicalViewAuthorityInputsFromState } from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  CANONICAL_VIEW_PAYLOAD_SCHEMA,
  CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
  SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
  buildCanonicalCurriculumSemanticSceneViewPayload,
  buildCanonicalSceneSummaryViewPayload,
  buildCanonicalViewPayloadSet,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";
import {
  buildSceneSummarySnapshot,
  restoreSceneSummarySession,
  serializeSceneSummarySession,
} from "../solutions/seamgrim_ui_mvp/ui/scene_summary_contract.js";
import { buildCurriculumSemanticSceneV0 } from "../solutions/seamgrim_ui_mvp/ui/curriculum_semantic_scene.js";
import { buildProjectSnapshotV0 } from "../solutions/seamgrim_ui_mvp/ui/project_snapshot.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function outcomeSummary(payload) {
  return {
    status: String(payload?.status ?? ""),
    diagnostic_code: payload?.diagnostic?.code ?? null,
  };
}

function assertStatusMatrix(family, expectedCases, actualCases) {
  for (const [caseId, expected] of Object.entries(expectedCases)) {
    const actual = actualCases[caseId];
    assert(actual, `${family} ${caseId}: actual case missing`);
    assert(actual.status === expected.status, `${family} ${caseId}: status ${actual.status} != ${expected.status}`);
    assert(
      actual.diagnostic_code === expected.diagnostic_code,
      `${family} ${caseId}: diagnostic ${actual.diagnostic_code} != ${expected.diagnostic_code}`,
    );
  }
}

const root = process.cwd();
const packRoot = path.join(root, "pack/seamgrim_scene_curriculum_canonical_view_payload_v1");
const state = JSON.parse(await fs.readFile(path.join(packRoot, "fixtures/authoritative_state.detjson"), "utf8"));
const caseMatrix = JSON.parse(await fs.readFile(path.join(packRoot, "case_matrix.detjson"), "utf8"));
assert(
  caseMatrix.schema === "seamgrim.scene_curriculum_canonical_view_payload.case_matrix.v1",
  "T5G case matrix schema mismatch",
);
const stateBefore = stableSemanticJson(state);
const canonicalViewPayloads = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state));

assert(canonicalViewPayloads.graph.status === "available", "fixture canonical graph unavailable");
assert(canonicalViewPayloads.table.status === "available", "fixture canonical table unavailable");
assert(canonicalViewPayloads.text.status === "available", "fixture canonical text unavailable");

const scene = buildSceneSummarySnapshot({
  timestamp: "2026-08-25T00:00:00.000Z",
  lessonId: "t5g_scene",
  lessonTitle: "T5G scene",
  requiredViews: ["graph", "table", "text"],
  primaryView: "graph",
  canonicalViewPayloads,
  runtimeDerived: { canonicalViewPayloads },
  view: { kind: "view-graph", config: { zoom: 9, browser_only: true } },
  time: { t_min: 0, t_max: 2, now: 1, frame_count: 3, frame_sample: [0, 1, 2] },
  overlayRuns: [{ id: "browser-layer", graph: { series: [] } }],
});
const sceneCanonical = scene.canonical_view_payload;
assert(sceneCanonical.schema === CANONICAL_VIEW_PAYLOAD_SCHEMA, "scene canonical envelope missing");
assert(sceneCanonical.family === "scene-summary" && sceneCanonical.status === "available", "scene canonical unavailable");
assert(sceneCanonical.content.scene_summary.primary_family === "graph", "scene explicit primary drift");
assert(
  stableSemanticJson(sceneCanonical.content.scene_summary.primary_payload) === stableSemanticJson(canonicalViewPayloads.graph),
  "scene nested canonical graph drift",
);
const sceneBytes = stableSemanticJson(sceneCanonical);
for (const forbidden of ["browser-layer", "browser_only", "2026-08-25T00:00:00.000Z", "frame_sample", "bogae_scene"]) {
  assert(!sceneBytes.includes(forbidden), `scene presentation/session token leaked: ${forbidden}`);
}
const restoredScene = restoreSceneSummarySession(serializeSceneSummarySession(scene));
assert(stableSemanticJson(restoredScene.canonical_view_payload) === sceneBytes, "scene session restore parity failed");

const ambiguousScene = buildSceneSummarySnapshot({
  lessonId: "ambiguous",
  requiredViews: ["graph", "table"],
  canonicalViewPayloads,
  runtimeDerived: { canonicalViewPayloads },
}).canonical_view_payload;
assert(ambiguousScene.status === "ambiguous", "scene equal candidates must be ambiguous");
assert(ambiguousScene.diagnostic?.code === "E_CANONICAL_SCENE_SUMMARY_AMBIGUOUS_CANDIDATE", "scene ambiguity diagnostic drift");

const lesson = {
  id: "t5g_curriculum",
  curriculumCard: {
    lesson_id: "t5g_curriculum",
    world_id: "world.t5g",
    primary_view: "graph",
    allowed_controls: ["속도"],
    allowed_observations: ["위치"],
    prediction_prompt: "어디로 움직일까요?",
    inquiry_prompt: "그래프를 설명하세요.",
    display: { core_concepts: ["위치", "속도"] },
  },
};
const curriculum = buildCurriculumSemanticSceneV0(lesson, {
  controlValues: { "속도": 12 },
  observationRows: [{ id: "위치", value: 8, unit: "m", role: "state", source_ref: "runtime.position" }],
  primaryViewMode: "graph",
  canonicalViewPayloads,
});
const curriculumCanonical = curriculum.canonical_view_payload;
assert(curriculumCanonical.family === "curriculum-semantic-scene", "curriculum canonical family drift");
assert(curriculumCanonical.status === "available", "curriculum canonical unavailable");
assert(curriculum.runtime_binding.complete === true, "curriculum wrapper runtime binding incomplete");
assert(curriculumCanonical.content.curriculum_semantic_scene.runtime_binding.complete === true, "canonical curriculum binding incomplete");
assert(curriculumCanonical.content.curriculum_semantic_scene.anchors.length === 2, "canonical curriculum anchors drift");
const curriculumBytes = stableSemanticJson(curriculumCanonical);
for (const forbidden of ["target_selector", "a11y_label", "reading_order", "prediction_prompt", "#run-main-graph-host"]) {
  assert(!curriculumBytes.includes(forbidden), `curriculum presentation token leaked: ${forbidden}`);
}
const projectSnapshot = buildProjectSnapshotV0({
  projectName: "T5G",
  ddnText: "가:수 <- 1.",
  sourceLessonId: lesson.id,
  curriculumSemanticScene: curriculum,
});
assert(
  stableSemanticJson(projectSnapshot.curriculum_semantic_scene.canonical_view_payload) === curriculumBytes,
  "curriculum project snapshot parity failed",
);

const mutatedPresentation = structuredClone(curriculum);
mutatedPresentation.reading_order.reverse();
mutatedPresentation.prompts.prediction = "browser-only prompt change";
mutatedPresentation.anchors[0].target_selector = "#different-browser-target";
assert(
  stableSemanticJson(mutatedPresentation.canonical_view_payload) === curriculumBytes,
  "curriculum presentation mutation changed canonical bytes",
);

const identity = canonicalViewPayloads.graph.identity;
const unsupportedScene = buildCanonicalSceneSummaryViewPayload({
  identity,
  projection: { schema: "seamgrim.scene_summary_semantic_projection.v999", status: "available" },
});
assert(unsupportedScene.status === "unsupported", "unknown scene projection schema must fail closed");
const missingIdentityScene = buildCanonicalSceneSummaryViewPayload({
  identity: {},
  projection: {
    schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
    status: "available",
    primary_family: "graph",
    required_families: ["graph"],
    primary_payload: canonicalViewPayloads.graph,
  },
});
assert(missingIdentityScene.status === "unavailable", "scene missing identity must fail closed");

const duplicateAnchorCurriculum = buildCanonicalCurriculumSemanticSceneViewPayload({
  identity,
  projection: {
    schema: CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
    status: "available",
    scene_id: "semantic_scene.bad",
    lesson_id: "bad",
    world_id: "bad",
    primary_view: "graph",
    primary_view_family: "graph",
    primary_view_payload: canonicalViewPayloads.graph,
    anchors: [
      { anchor_id: "same", kind: "control", name: "a", binding: { source: "slider_panel", resolved: true, value: 1 } },
      { anchor_id: "same", kind: "observation", name: "b", binding: { source: "committed_observation_payload", resolved: true, value: 2 } },
    ],
  },
});
assert(duplicateAnchorCurriculum.status === "invalid", "duplicate curriculum anchor must fail closed");

const exactSceneProjection = {
  schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
  status: "available",
  primary_family: "graph",
  required_families: ["graph", "table", "text"],
  primary_payload: canonicalViewPayloads.graph,
  source: "case_matrix",
  provenance: "canonical_view_payload_projection",
};
const exactCurriculumContent = curriculumCanonical.content.curriculum_semantic_scene;
const exactCurriculumProjection = {
  schema: CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
  status: "available",
  scene_id: exactCurriculumContent.scene_id,
  lesson_id: exactCurriculumContent.lesson_id,
  world_id: exactCurriculumContent.world_id,
  primary_view: exactCurriculumContent.primary_view,
  primary_view_family: exactCurriculumContent.primary_view_family,
  primary_view_payload: exactCurriculumContent.primary_view_payload,
  concept_labels: exactCurriculumContent.concept_labels,
  anchors: exactCurriculumContent.anchors,
  source: "case_matrix",
  provenance: "validated_curriculum_card_and_committed_bindings",
};

const sceneStatusCases = {
  POSITIVE: outcomeSummary(sceneCanonical),
  MISSING_NO_VIEW: outcomeSummary(buildCanonicalSceneSummaryViewPayload({ identity, projection: null })),
  INVALID: outcomeSummary(buildCanonicalSceneSummaryViewPayload({
    identity,
    projection: { ...exactSceneProjection, primary_family: "" },
  })),
  UNSUPPORTED: outcomeSummary(unsupportedScene),
  AMBIGUOUS: outcomeSummary(ambiguousScene),
  IDENTITY: outcomeSummary(missingIdentityScene),
  UNKNOWN_STATUS: outcomeSummary(buildCanonicalSceneSummaryViewPayload({
    identity,
    projection: { ...exactSceneProjection, status: "renderer_ready" },
  })),
};
const curriculumStatusCases = {
  POSITIVE: outcomeSummary(curriculumCanonical),
  MISSING_NO_VIEW: outcomeSummary(buildCanonicalCurriculumSemanticSceneViewPayload({ identity, projection: null })),
  INVALID: outcomeSummary(duplicateAnchorCurriculum),
  UNSUPPORTED: outcomeSummary(buildCanonicalCurriculumSemanticSceneViewPayload({
    identity,
    projection: { ...exactCurriculumProjection, schema: "seamgrim.curriculum_semantic_scene_projection.v999" },
  })),
  AMBIGUOUS: outcomeSummary(buildCanonicalCurriculumSemanticSceneViewPayload({
    identity,
    projection: {
      ...exactCurriculumProjection,
      status: "ambiguous",
      reason: "multiple_equal_primary_view_candidates",
    },
  })),
  IDENTITY: outcomeSummary(buildCanonicalCurriculumSemanticSceneViewPayload({
    identity: {},
    projection: exactCurriculumProjection,
  })),
  UNKNOWN_STATUS: outcomeSummary(buildCanonicalCurriculumSemanticSceneViewPayload({
    identity,
    projection: { ...exactCurriculumProjection, status: "dom_bound" },
  })),
};
assertStatusMatrix(
  "scene-summary",
  caseMatrix.families["scene-summary"].status_cases,
  sceneStatusCases,
);
assertStatusMatrix(
  "curriculum-semantic-scene",
  caseMatrix.families["curriculum-semantic-scene"].status_cases,
  curriculumStatusCases,
);
assert(
  stableSemanticJson(sceneCanonical.content.scene_summary.required_families)
    === stableSemanticJson(caseMatrix.families["scene-summary"].ordering.required_families),
  "scene required-family ordering drift",
);
assert(
  stableSemanticJson(exactCurriculumContent.anchors.map((anchor) => anchor.anchor_id))
    === stableSemanticJson(caseMatrix.families["curriculum-semantic-scene"].ordering.anchor_ids),
  "curriculum anchor ordering drift",
);
assert(
  stableSemanticJson(exactCurriculumContent.concept_labels)
    === stableSemanticJson(caseMatrix.families["curriculum-semantic-scene"].ordering.concept_labels),
  "curriculum concept ordering drift",
);
assert(
  caseMatrix.families["scene-summary"].cross_surface.scene_session_byte_parity === true,
  "scene cross-surface oracle missing",
);
assert(
  caseMatrix.families["curriculum-semantic-scene"].cross_surface.project_snapshot_byte_parity === true,
  "curriculum cross-surface oracle missing",
);

assert(stableSemanticJson(state) === stateBefore, "T5G projection mutated authoritative state");

if (process.argv.includes("--emit")) {
  console.log(`SCENE=${JSON.stringify(JSON.parse(sceneBytes), null, 2)}`);
  console.log(`CURRICULUM=${JSON.stringify(JSON.parse(curriculumBytes), null, 2)}`);
} else {
  for (const [name, bytes] of [["scene_summary", sceneBytes], ["curriculum_semantic_scene", curriculumBytes]]) {
    const expected = JSON.parse(await fs.readFile(path.join(packRoot, `expected/${name}.payload.detjson`), "utf8"));
    assert(bytes === stableSemanticJson(expected), `${name} exact golden mismatch`);
  }
}

console.log(JSON.stringify({
  status: "PASS",
  families: ["scene-summary", "curriculum-semantic-scene"],
  scene_session_byte_parity: true,
  curriculum_project_snapshot_byte_parity: true,
  matrix_status_cases: Object.keys(sceneStatusCases).length + Object.keys(curriculumStatusCases).length,
  ordering_cases: 3,
  presentation_metadata_in_canonical_identity: 0,
  graph_default_semantic_success: 0,
  state_hash_drift: 0,
  storage_drift: 0,
}));
