const CARD_SCHEMA = "seamgrim.lesson_card.v0";
const CROSSWALK_SCHEMA = "seamgrim.curriculum_crosswalk.v0";

function normalizeString(value) {
  return String(value ?? "").trim();
}

function normalizeStringList(value) {
  return Array.isArray(value) ? value.map(normalizeString).filter(Boolean) : [];
}

function makeDiagnostic(code, message, cardId = "") {
  return {
    code,
    message,
    card_id: normalizeString(cardId),
  };
}

function refSetFromCrosswalk(crosswalk) {
  const refs = new Set(Object.keys(crosswalk?.refs ?? {}));
  Object.keys(crosswalk?.concept_spine ?? {}).forEach((key) => refs.add(key));
  Object.keys(crosswalk?.crosscutting ?? {}).forEach((key) => refs.add(key));
  Object.keys(crosswalk?.misconceptions ?? {}).forEach((key) => refs.add(key));
  return refs;
}

export function validateCurriculumCardBundle({ cardsBundle = null, crosswalk = null } = {}) {
  const diagnostics = [];
  if (normalizeString(cardsBundle?.schema) !== CARD_SCHEMA) {
    diagnostics.push(makeDiagnostic("E_CURRICULUM_CARD_SCHEMA", "교과 카드 묶음 형식이 다릅니다."));
  }
  if (normalizeString(crosswalk?.schema) !== CROSSWALK_SCHEMA) {
    diagnostics.push(makeDiagnostic("E_CURRICULUM_CROSSWALK_SCHEMA", "교과 연결표 형식이 다릅니다."));
  }
  const knownRefs = refSetFromCrosswalk(crosswalk);
  const cards = Array.isArray(cardsBundle?.cards) ? cardsBundle.cards : [];
  for (const card of cards) {
    const cardId = normalizeString(card?.lesson_id || card?.world_id);
    for (const field of [
      "lesson_id",
      "world_id",
      "grade_band",
      "subject_refs",
      "curriculum_refs",
      "concept_spine_refs",
      "stem_practice_refs",
      "crosscutting_refs",
      "primary_view",
      "allowed_controls",
      "allowed_observations",
      "prediction_prompt",
      "inquiry_prompt",
      "misconception_refs",
      "lesson_refs",
      "view_refs",
      "deployment_refs",
      "lineage",
      "evidence",
      "status_axes",
    ]) {
      const value = card?.[field];
      const missing = Array.isArray(value) ? value.length === 0 : value === undefined || value === null || normalizeString(value) === "";
      if (missing && field !== "view_refs") {
        diagnostics.push(makeDiagnostic("E_CURRICULUM_CARD_MISSING_FIELD", `교과 카드에 ${field} 항목이 없습니다.`, cardId));
      }
    }
    const refFields = ["curriculum_refs", "concept_spine_refs", "stem_practice_refs", "crosscutting_refs", "misconception_refs"];
    for (const field of refFields) {
      for (const ref of normalizeStringList(card?.[field])) {
        if (!knownRefs.has(ref)) {
          diagnostics.push(makeDiagnostic("E_CURRICULUM_CARD_UNKNOWN_REF", `알 수 없는 교과 연결 ref입니다: ${ref}`, cardId));
        }
      }
    }
  }
  return {
    ok: diagnostics.length === 0,
    diagnostics,
    card_count: cards.length,
  };
}

export function mergeCurriculumCardsIntoLessons(lessons = [], { cardsBundle = null, crosswalk = null } = {}) {
  const validation = validateCurriculumCardBundle({ cardsBundle, crosswalk });
  const cards = Array.isArray(cardsBundle?.cards) ? cardsBundle.cards : [];
  const cardByLesson = new Map(cards.map((card) => [normalizeString(card?.lesson_id), card]));
  const refs = crosswalk?.refs ?? {};
  const concepts = crosswalk?.concept_spine ?? {};
  const crosscutting = crosswalk?.crosscutting ?? {};
  const misconceptions = crosswalk?.misconceptions ?? {};
  const enriched = (Array.isArray(lessons) ? lessons : []).map((lesson) => {
    const card = cardByLesson.get(normalizeString(lesson?.id));
    if (!card) return lesson;
    const display = card.display && typeof card.display === "object" ? card.display : {};
    const curriculumCard = {
      ...card,
      display: {
        grade_band: normalizeString(display.grade_band || card.grade_band),
        core_concepts: normalizeStringList(display.core_concepts).length
          ? normalizeStringList(display.core_concepts)
          : normalizeStringList(card.concept_spine_refs).map((ref) => concepts[ref] || ref),
        inquiry_actions: normalizeStringList(display.inquiry_actions),
        curriculum_labels: normalizeStringList(card.curriculum_refs).map((ref) => normalizeString(refs[ref]?.label || ref)),
        practice_labels: normalizeStringList(card.stem_practice_refs).map((ref) => normalizeString(refs[ref]?.label || ref)),
        crosscutting_labels: normalizeStringList(card.crosscutting_refs).map((ref) => normalizeString(crosscutting[ref] || ref)),
        misconception_labels: normalizeStringList(card.misconception_refs).map((ref) => normalizeString(misconceptions[ref] || ref)),
      },
    };
    return {
      ...lesson,
      curriculumCard,
      tags: Array.from(new Set([...(lesson?.tags ?? []), ...curriculumCard.display.core_concepts])),
      keywords: Array.from(new Set([...(lesson?.keywords ?? []), ...curriculumCard.display.core_concepts])),
    };
  });
  return {
    lessons: enriched,
    validation,
  };
}
