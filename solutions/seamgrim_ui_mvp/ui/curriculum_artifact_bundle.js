export const CURRICULUM_ARTIFACT_BUNDLE_SCHEMA = "seamgrim.curriculum_artifact_bundle.v0";

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function asText(value, fallback = "") {
  const text = String(value ?? "").replace(/\r\n/g, "\n").trim();
  return text || fallback;
}

function listText(values = []) {
  return asArray(values).map((item) => String(item ?? "").trim()).filter(Boolean);
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

function artifactBase(inquiry = {}) {
  return {
    lesson_id: asText(inquiry.lesson_id, ""),
    world_id: asText(inquiry.world_id, ""),
    grade_band: asText(inquiry.grade_band, ""),
    primary_view: asText(inquiry.primary_view, ""),
    curriculum_refs: listText(inquiry.curriculum_refs),
    concept_spine_refs: listText(inquiry.concept_spine_refs),
  };
}

export function buildCurriculumArtifactBundleV0(inquiryArtifact = null) {
  const inquiry = asObject(inquiryArtifact);
  if (!inquiry.schema) return null;
  const base = artifactBase(inquiry);
  const response = asObject(inquiry.response);
  const prompt = asObject(inquiry.prompt);
  const runEvidence = asObject(inquiry.run_evidence);
  const boundaries = {
    canonical_curriculum_alignment: false,
    external_student_success_claim: false,
    state_hash_participation: false,
    runtime_role_toggle: false,
  };
  return sortDeep({
    schema: CURRICULUM_ARTIFACT_BUNDLE_SCHEMA,
    lesson_id: base.lesson_id,
    world_id: base.world_id,
    artifacts: [
      {
        ...base,
        artifact_kind: "prediction",
        prompt: asText(prompt.prediction, ""),
        student_text: String(response.prediction_text ?? "").replace(/\r\n/g, "\n"),
        ready: Boolean(asText(response.prediction_text, "")),
        boundaries,
      },
      {
        ...base,
        artifact_kind: "experiment",
        allowed_controls: listText(inquiry.allowed_controls),
        allowed_observations: listText(inquiry.allowed_observations),
        runtime_hash: asText(runEvidence.runtime_hash, ""),
        runtime_status: asText(runEvidence.runtime_status, ""),
        observation_summary: asText(runEvidence.observation_summary, ""),
        completed_activity_ids: listText(runEvidence.completed_activity_ids),
        ready: Boolean(asText(runEvidence.runtime_hash, "")),
        boundaries,
      },
      {
        ...base,
        artifact_kind: "explanation",
        prompt: asText(prompt.inquiry, ""),
        student_text: String(response.explanation_text ?? "").replace(/\r\n/g, "\n"),
        evidence_runtime_hash: asText(runEvidence.runtime_hash, ""),
        ready: Boolean(asText(response.explanation_text, "")),
        boundaries,
      },
    ],
    status: {
      prediction_ready: Boolean(asText(response.prediction_text, "")),
      experiment_ready: Boolean(asText(runEvidence.runtime_hash, "")),
      explanation_ready: Boolean(asText(response.explanation_text, "")),
    },
    boundaries,
  });
}

export function summarizeCurriculumArtifactBundle(bundle = null) {
  const row = asObject(bundle);
  const status = asObject(row.status);
  const ready = [
    status.prediction_ready ? "예측" : "",
    status.experiment_ready ? "실험" : "",
    status.explanation_ready ? "설명" : "",
  ].filter(Boolean);
  return ready.length ? `산출물 ${ready.join(", ")} 준비됨` : "산출물 대기";
}
