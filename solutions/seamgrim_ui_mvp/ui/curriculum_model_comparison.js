export const CURRICULUM_MODEL_COMPARISON_SCHEMA = "seamgrim.curriculum_model_comparison.v0";

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

export function buildCurriculumModelComparisonV0(lesson = null) {
  const row = asObject(lesson);
  const pair = asObject(row.representationPair);
  const card = asObject(row.curriculumCard);
  const validPair = pair.schema === "ddn.seamgrim.representation_pair.v1";
  const left = asObject(pair.left);
  const right = asObject(pair.right);
  const lessonId = asText(row.id, asText(card.lesson_id, "custom"));
  return sortDeep({
    schema: CURRICULUM_MODEL_COMPARISON_SCHEMA,
    lesson_id: lessonId,
    world_id: asText(card.world_id, lessonId),
    explicit_model: {
      available: true,
      representation_kind: asText(left.representation_kind, "explicit_tick"),
      title: asText(left.title, "마디별 계산"),
      runtime_execution: asText(left.runtime_execution, "landed"),
      source_path: asText(left.source_path, `solutions/seamgrim_ui_mvp/lessons/${lessonId}/lesson.ddn`),
      teaching_snippet: listText(left.teaching_snippet),
    },
    declarative_model: {
      available: validPair,
      representation_kind: asText(right.representation_kind, validPair ? "declarative_seum" : "candidate_not_attached"),
      title: asText(right.title, "세움 선언"),
      runtime_execution: asText(right.runtime_execution, "not_landed"),
      source_path: asText(right.source_path, ""),
      teaching_snippet: listText(right.teaching_snippet),
      solveplan_path: asText(right.solveplan_path, ""),
    },
    teaching_map: sortDeep(asArray(pair.teaching_map)),
    symbol_map: asArray(pair.symbol_map),
    comparison_prompt: validPair
      ? "왼쪽 마디별 계산과 오른쪽 세움 선언이 같은 뜻을 어디까지 공유하는지 비교하세요."
      : "이 수업은 현재 실행되는 마디별 계산을 기준으로 배우고, 세움 선언 비교는 후속 후보로 남깁니다.",
    status: {
      explicit_runtime_landed: true,
      declarative_runtime_landed: right.runtime_execution === "landed",
      declarative_teaching_pair_available: validPair,
      observable_equivalence_verified: pair.same_observables_claim === true,
    },
    boundaries: {
      declarative_runtime_completion_claim: false,
      observable_equivalence_claim: pair.same_observables_claim === true,
      automatic_promotion_forbidden: true,
      state_hash_participation: false,
    },
  });
}

export function summarizeCurriculumModelComparison(model = null) {
  const row = asObject(model);
  const status = asObject(row.status);
  const declarative = status.declarative_teaching_pair_available
    ? "선언형 비교 있음"
    : "선언형 비교 후보";
  const runtime = status.declarative_runtime_landed
    ? "선언형 실행됨"
    : "선언형 실행 미주장";
  return `모델 비교: 명령형 실행됨 · ${declarative} · ${runtime}`;
}
