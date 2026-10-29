import {
  CANONICAL_VIEW_PAYLOAD_SCHEMA,
  CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
  buildCanonicalCurriculumSemanticSceneViewPayload,
} from "./workbench/semantic_view_payload.js";

export const CURRICULUM_SEMANTIC_SCENE_SCHEMA = "seamgrim.curriculum_semantic_scene.v0";

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function asText(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function listText(values = []) {
  return asArray(values).map((item) => String(item ?? "").trim()).filter(Boolean);
}

function selectorValue(value) {
  return String(value ?? "").replaceAll("\\", "\\\\").replaceAll('"', '\\"');
}

function sortDeep(value) {
  if (Array.isArray(value)) return value.map((item) => sortDeep(item));
  if (value && typeof value === "object") {
    const out = {};
    Object.keys(value)
      .filter((key) => value[key] !== undefined && typeof value[key] !== "function")
      .sort()
      .forEach((key) => {
        out[key] = sortDeep(value[key]);
      });
    return out;
  }
  return value;
}

function controlAnchorRows(values = [], controlValues = {}) {
  const current = asObject(controlValues);
  return listText(values).map((name) => {
    const resolved = Object.prototype.hasOwnProperty.call(current, name);
    return {
      anchor_id: `control.${name}`,
      kind: "control",
      name,
      a11y_label: `조절값 ${name}`,
      target_selector: `[data-control-name="${selectorValue(name)}"]`,
      binding: {
        source: "slider_panel",
        resolved,
        value: resolved ? current[name] : null,
      },
      state_hash_participation: false,
    };
  });
}

function observationAnchorRows(values = [], observationRows = []) {
  const rows = asArray(observationRows).filter((row) => row && typeof row === "object");
  const byId = new Map(rows.map((row) => [asText(row.id), row]));
  return listText(values).map((name) => {
    const row = byId.get(name) ?? null;
    return {
      anchor_id: `observation.${name}`,
      kind: "observation",
      name,
      a11y_label: `관찰값 ${name}`,
      target_selector: `[data-observation-id="${selectorValue(name)}"]`,
      binding: {
        source: "committed_observation_payload",
        resolved: Boolean(row),
        value: row?.value ?? null,
        unit: asText(row?.unit),
        delta: row?.delta ?? null,
        role: asText(row?.role),
        source_ref: asText(row?.source_ref),
      },
      state_hash_participation: false,
    };
  });
}

function primaryViewBinding(primaryView, primaryViewMode) {
  const requested = asText(primaryView);
  const mode = asText(primaryViewMode);
  const selector = mode === "graph"
    ? "#run-main-graph-host"
    : (mode === "space2d" || mode === "debug-fallback")
      ? "#canvas-bogae"
      : "";
  return {
    requested,
    mode,
    target_selector: selector,
    resolved: Boolean(requested && mode && selector),
  };
}

const CANONICAL_VIEW_KEY_BY_PRIMARY = Object.freeze({
  console: "console",
  "console-grid": "console_grid",
  graph: "graph",
  space2d: "space2d",
  table: "table",
  text: "text",
  structure: "structure",
  grid2d: "grid2d",
  line: "workbench_line",
  pendulum: "workbench_pendulum",
  projectile: "workbench_projectile",
  grid: "workbench_grid",
});

function canonicalPrimaryFamily(primaryView) {
  const value = asText(primaryView).toLowerCase();
  if (["line", "pendulum", "projectile", "grid"].includes(value)) return `workbench-${value}`;
  return value;
}

function canonicalPrimaryPayload(primaryView, canonicalViewPayloads) {
  const key = CANONICAL_VIEW_KEY_BY_PRIMARY[asText(primaryView).toLowerCase()] ?? "";
  if (!key) return null;
  const payload = canonicalViewPayloads?.[key];
  return payload && typeof payload === "object" && !Array.isArray(payload) ? payload : null;
}

function canonicalAnchorRows(anchors) {
  return asArray(anchors).map((anchor) => ({
    anchor_id: asText(anchor?.anchor_id),
    kind: asText(anchor?.kind),
    name: asText(anchor?.name),
    binding: sortDeep(asObject(anchor?.binding)),
  }));
}

export function buildCurriculumSemanticSceneV0(
  lesson = null,
  {
    controlValues = null,
    observationRows = null,
    primaryViewMode = "",
    canonicalViewPayloads = null,
  } = {},
) {
  const row = asObject(lesson);
  const card = asObject(row.curriculumCard);
  const display = asObject(card.display);
  const lessonId = asText(row.id, asText(card.lesson_id, "custom"));
  const controls = listText(card.allowed_controls);
  const observations = listText(card.allowed_observations);
  const concepts = listText(display.core_concepts);
  const anchors = [
    ...controlAnchorRows(controls, controlValues),
    ...observationAnchorRows(observations, observationRows),
  ];
  const resolvedCount = anchors.filter((anchor) => anchor.binding?.resolved === true).length;
  const viewBinding = primaryViewBinding(card.primary_view, primaryViewMode);
  const primaryView = asText(card.primary_view, "");
  const primaryFamily = canonicalPrimaryFamily(primaryView);
  const primaryPayload = canonicalPrimaryPayload(primaryView, canonicalViewPayloads);
  const primaryPayloadSchema = asText(primaryPayload?.schema);
  const projectionStatus = !primaryView
    ? "no_view"
    : !primaryPayload
      ? "no_view"
      : primaryPayloadSchema !== CANONICAL_VIEW_PAYLOAD_SCHEMA
        ? (primaryPayloadSchema ? "unsupported" : "invalid")
        : asText(primaryPayload.status, "invalid");
  const semanticProjection = {
    schema: CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA,
    status: projectionStatus,
    scene_id: `semantic_scene.${lessonId}`,
    lesson_id: lessonId,
    world_id: asText(card.world_id, lessonId),
    primary_view: primaryView,
    primary_view_family: primaryFamily,
    primary_view_payload: projectionStatus === "available" ? primaryPayload : null,
    concept_labels: concepts,
    anchors: canonicalAnchorRows(anchors),
    reason: projectionStatus === "available" ? "" : "authoritative_primary_view_payload_not_available",
    source: "validated_curriculum_card_and_runtime_bindings",
    provenance: "validated_curriculum_card_and_committed_bindings",
  };
  const canonicalViewPayload = buildCanonicalCurriculumSemanticSceneViewPayload({
    identity: primaryPayload?.identity ?? {},
    projection: semanticProjection,
  });
  const canonicalAvailable = canonicalViewPayload.status === "available";
  const readingOrder = [
    "prediction_prompt",
    "control_anchors",
    "primary_view",
    "observation_anchors",
    "explanation_prompt",
  ];
  const prompts = {
    prediction: asText(card.prediction_prompt, ""),
    explanation: asText(card.inquiry_prompt, ""),
  };
  return sortDeep({
    schema: CURRICULUM_SEMANTIC_SCENE_SCHEMA,
    canonical_view_payload: canonicalViewPayload,
    scene_id: `semantic_scene.${lessonId}`,
    lesson_id: lessonId,
    world_id: asText(card.world_id, lessonId),
    primary_view: primaryView,
    semantic_scene_claim: "teaching_overlay_with_runtime_bindings",
    concept_labels: concepts,
    anchors,
    primary_view_binding: {
      ...viewBinding,
      canonical_status: canonicalViewPayload.status,
      canonical_payload_hash: asText(canonicalViewPayload.candidate_payload_hash),
    },
    runtime_binding: {
      expected_count: anchors.length,
      resolved_count: resolvedCount,
      complete: anchors.length > 0 && resolvedCount === anchors.length && viewBinding.resolved,
      canonical_binding_complete: canonicalAvailable,
      observation_source: "seamgrim.observation_payload.v1",
      control_source: "slider_panel",
    },
    presentation_session: {
      reading_order: readingOrder,
      prompts,
      anchor_targets: anchors.map((anchor) => ({
        anchor_id: anchor.anchor_id,
        a11y_label: anchor.a11y_label,
        target_selector: anchor.target_selector,
      })),
      primary_view_target_selector: viewBinding.target_selector,
    },
    reading_order: readingOrder,
    prompts,
    boundaries: {
      state_hash_participation: false,
      world_state_storage: false,
      semantic_runtime_completion_claim: false,
      control_observation_binding_claim: true,
      accessibility_name_claim: true,
      canonical_owner: "semantic_view_payload",
      top_level_scene_fields_are_educational_presentation_compatibility: true,
      canonical_identity_excludes: [
        "target_selector",
        "a11y_label",
        "reading_order",
        "prompts",
        "browser_view_mode",
      ],
    },
  });
}

export function summarizeCurriculumSemanticScene(scene = null) {
  const row = asObject(scene);
  const anchors = asArray(row.anchors);
  const controlCount = anchors.filter((anchor) => anchor.kind === "control").length;
  const observationCount = anchors.filter((anchor) => anchor.kind === "observation").length;
  const resolved = Number(row?.runtime_binding?.resolved_count ?? 0);
  const expected = Number(row?.runtime_binding?.expected_count ?? anchors.length);
  return `의미 장면: 조절 ${controlCount} · 관찰 ${observationCount} · 연결 ${resolved}/${expected}`;
}
