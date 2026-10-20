export const SEAMGRIM_FIRST_RUN_PATH_TEXT = "첫 인사 -> 움직임 -> 매김 조절 -> 되돌려보기/거울";
export const SEAMGRIM_FIRST_RUN_ESTIMATED_MINUTES = 3;

export const SEAMGRIM_FIRST_RUN_STEPS = Object.freeze([
  Object.freeze({
    id: "hello",
    title: "첫 인사",
    targetKind: "sample",
    targetId: "06_console_grid_scalar_show",
  }),
  Object.freeze({
    id: "movement",
    title: "움직임",
    targetKind: "sample",
    targetId: "09_moyang_pendulum_working",
  }),
  Object.freeze({
    id: "maegim",
    title: "매김 조절",
    targetKind: "lesson",
    targetId: "elem_math_multiplication_table",
  }),
  Object.freeze({
    id: "replay_geoul",
    title: "되돌려보기/거울",
    targetKind: "lesson",
    targetId: "elem_econ_timeline",
  }),
]);

export function normalizeFirstRunPath(raw) {
  const text = String(raw ?? "").trim().toLowerCase();
  return SEAMGRIM_FIRST_RUN_STEPS.some((step) => step.id === text) ? text : "";
}

export function resolveFirstRunStepByPath(stepId = "") {
  const normalized = normalizeFirstRunPath(stepId);
  return SEAMGRIM_FIRST_RUN_STEPS.find((step) => step.id === normalized) ?? null;
}

export function resolveFirstRunStepByTarget({
  id = "",
  source = "",
  firstRunPath = "",
} = {}) {
  const direct = resolveFirstRunStepByPath(firstRunPath);
  if (direct) return direct;
  const normalizedId = String(id ?? "").trim();
  const normalizedSource = String(source ?? "").trim().toLowerCase();
  if (!normalizedId) return null;
  return (
    SEAMGRIM_FIRST_RUN_STEPS.find((step) => {
      if (step.targetId !== normalizedId) return false;
      if (step.targetKind === "sample") return normalizedSource === "sample";
      if (step.targetKind === "lesson") return normalizedSource !== "sample";
      return false;
    }) ?? null
  );
}

export function buildFirstRunBadgeLabel(step) {
  if (!step) return "";
  return `첫실행 ${step.title}`;
}

export function buildFirstRunHintText(step) {
  if (!step) return "";
  return `첫 시작 ${step.title} · ${SEAMGRIM_FIRST_RUN_PATH_TEXT}`;
}
