export const CURRICULUM_INQUIRY_ARTIFACT_SCHEMA = "seamgrim.curriculum_inquiry_artifact.v0";

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

function sortDeep(value) {
  if (Array.isArray(value)) {
    return value.map((item) => sortDeep(item));
  }
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

function listText(values = []) {
  return asArray(values).map((item) => String(item ?? "").trim()).filter(Boolean);
}

export function hasCurriculumInquiryCard(lesson = null) {
  return Boolean(asObject(lesson).curriculumCard);
}

export function buildCurriculumInquiryArtifactV0({
  lesson = null,
  predictionText = "",
  explanationText = "",
  runtimeHash = "",
  runtimeStatus = "",
  primaryView = "",
  completedActivityIds = [],
  observationSummary = "",
} = {}) {
  const row = asObject(lesson);
  const card = asObject(row.curriculumCard);
  const display = asObject(card.display);
  const lessonId = asText(card.lesson_id, asText(row.id, "custom"));
  const worldId = asText(card.world_id, lessonId);
  const normalizedRuntimeHash = asText(runtimeHash, "");
  return sortDeep({
    schema: CURRICULUM_INQUIRY_ARTIFACT_SCHEMA,
    lesson_id: lessonId,
    world_id: worldId,
    grade_band: asText(card.grade_band, asText(row.grade, "")),
    subject_refs: listText(card.subject_refs),
    curriculum_refs: listText(card.curriculum_refs),
    concept_spine_refs: listText(card.concept_spine_refs),
    stem_practice_refs: listText(card.stem_practice_refs),
    crosscutting_refs: listText(card.crosscutting_refs),
    primary_view: asText(primaryView, asText(card.primary_view, "")),
    secondary_views: listText(card.secondary_views),
    allowed_controls: listText(card.allowed_controls),
    allowed_observations: listText(card.allowed_observations),
    prompt: {
      prediction: asText(card.prediction_prompt, ""),
      inquiry: asText(card.inquiry_prompt, ""),
    },
    response: {
      prediction_text: String(predictionText ?? "").replace(/\r\n/g, "\n"),
      explanation_text: String(explanationText ?? "").replace(/\r\n/g, "\n"),
    },
    run_evidence: {
      runtime_hash: normalizedRuntimeHash,
      runtime_status: asText(runtimeStatus, normalizedRuntimeHash ? "done" : "idle"),
      observation_summary: asText(observationSummary, ""),
      completed_activity_ids: listText(completedActivityIds),
    },
    display: {
      grade_band: asText(display.grade_band, asText(row.grade, "")),
      core_concepts: listText(display.core_concepts),
      inquiry_actions: listText(display.inquiry_actions),
    },
    boundaries: {
      canonical_curriculum_alignment: false,
      external_student_success_claim: false,
      state_hash_participation: false,
      runtime_role_toggle: false,
    },
  });
}

export function summarizeCurriculumInquiryArtifact(artifact = null) {
  const row = asObject(artifact);
  const prediction = asText(row.response?.prediction_text, "");
  const explanation = asText(row.response?.explanation_text, "");
  const runtimeHash = asText(row.run_evidence?.runtime_hash, "");
  const completed = listText(row.run_evidence?.completed_activity_ids);
  const parts = [
    prediction ? "예상 기록됨" : "예상 대기",
    runtimeHash ? `실행 ${runtimeHash.slice(0, 18)}` : "실행 대기",
    explanation ? "설명 기록됨" : "설명 대기",
    completed.length ? `활동 ${completed.length}개 완료` : "",
  ].filter(Boolean);
  return parts.join(" · ");
}
