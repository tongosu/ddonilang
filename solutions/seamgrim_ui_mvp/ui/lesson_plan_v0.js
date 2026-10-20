export const LESSON_PLAN_V0_SCHEMA = "seamgrim.lesson_plan.v0";

export const LESSON_PLAN_ACTIVITY_KINDS = Object.freeze([
  "observe",
  "change",
  "compare",
  "explain",
]);

const ACTIVITY_KIND_LABELS = Object.freeze({
  observe: "관찰",
  change: "바꾸기",
  compare: "비교",
  explain: "설명",
});

const ARRAY_FIELDS = Object.freeze(["allowed_controls", "anchor_refs", "measurement_components", "reaction_rule_refs"]);
const ROOT_ARRAY_FIELDS = Object.freeze(["allowed_controls", "control_ranges", "secondary_views", "misconception_refs"]);
const ROOT_FIELDS = Object.freeze([
  "schema",
  "lesson_id",
  "title",
  "lesson_question",
  "prediction_question",
  "explanation_question",
  "misconception_refs",
  "reading_check",
  "rubric",
  "deployment_profile_lock",
  "allowed_controls",
  "control_ranges",
  "primary_view",
  "secondary_views",
]);
const ACTIVITY_FIELDS = Object.freeze([
  "activity_id",
  "kind",
  "title",
  "prompt",
  "scene_ref",
  "primary_view",
  "preset_ref",
  "camera_policy",
  "measurement_components",
  "reaction_rule_refs",
  "view_authoring_hash",
  "allowed_controls",
  "question",
  "check_ref",
  "view_plan_ref",
  "anchor_refs",
]);

function normalizeString(value) {
  return String(value ?? "").trim();
}

function stripTomlString(raw) {
  const text = String(raw ?? "").trim();
  if (text.length >= 2 && text.startsWith('"') && text.endsWith('"')) {
    return text.slice(1, -1).replace(/\\"/g, '"').replace(/\\\\/g, "\\");
  }
  return text;
}

function escapeTomlString(raw) {
  return String(raw ?? "").replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function splitTomlItems(raw) {
  const text = String(raw ?? "");
  const out = [];
  let current = "";
  let inString = false;
  let escaped = false;
  for (const ch of text) {
    if (escaped) {
      current += ch;
      escaped = false;
      continue;
    }
    if (ch === "\\") {
      current += ch;
      escaped = true;
      continue;
    }
    if (ch === '"') {
      inString = !inString;
      current += ch;
      continue;
    }
    if (ch === "," && !inString) {
      const item = current.trim();
      if (item) out.push(item);
      current = "";
      continue;
    }
    current += ch;
  }
  const tail = current.trim();
  if (tail) out.push(tail);
  return out;
}

function parseTomlScalar(raw) {
  const text = String(raw ?? "").trim();
  if (text.startsWith("[") && text.endsWith("]")) {
    const inner = text.slice(1, -1).trim();
    return inner ? splitTomlItems(inner).map(stripTomlString).filter((item) => String(item).trim()) : [];
  }
  return stripTomlString(text);
}

function parseAssignment(line) {
  const match = String(line ?? "").match(/^([A-Za-z0-9_가-힣.-]+)\s*=\s*(.+)$/u);
  if (!match) return null;
  return {
    key: normalizeString(match[1]),
    value: parseTomlScalar(match[2]),
  };
}

function makeDiagnostic(code, message, { where = "lesson_plan", fix = "" } = {}) {
  return {
    code,
    severity: code.startsWith("E_") ? "error" : "warning",
    message,
    what: message,
    where,
    fix: fix || "수업 구성 TOML의 필수 필드와 활동 값을 확인하세요.",
  };
}

function normalizeArray(value) {
  if (Array.isArray(value)) return value.map(normalizeString).filter(Boolean);
  const text = normalizeString(value);
  return text ? text.split(",").map(normalizeString).filter(Boolean) : [];
}

function normalizeKind(raw) {
  const kind = normalizeString(raw).toLowerCase();
  return LESSON_PLAN_ACTIVITY_KINDS.includes(kind) ? kind : "observe";
}

function normalizeActivity(raw, index, diagnostics) {
  const source = raw && typeof raw === "object" ? raw : {};
  const kindRaw = normalizeString(source.kind);
  const kind = normalizeKind(kindRaw);
  if (kindRaw && kindRaw !== kind) {
    diagnostics.push(makeDiagnostic(
      "E_LESSON_PLAN_ACTIVITY_KIND",
      `알 수 없는 활동 종류입니다: ${kindRaw}`,
      { where: `activities[${index}].kind`, fix: "observe/change/compare/explain 중 하나를 사용하세요." },
    ));
  }
  const activityId = normalizeString(source.activity_id) || `${kind}-${index + 1}`;
  const title = normalizeString(source.title) || `${ACTIVITY_KIND_LABELS[kind]} ${index + 1}`;
  const prompt = normalizeString(source.prompt);
  if (!prompt) {
    diagnostics.push(makeDiagnostic(
      "E_LESSON_PLAN_ACTIVITY_PROMPT",
      `${activityId} 활동의 prompt가 비어 있습니다.`,
      { where: `activities[${index}].prompt`, fix: "학생에게 보일 짧은 안내문을 넣으세요." },
    ));
  }
  return {
    activity_id: activityId,
    kind,
    title,
    prompt,
    scene_ref: normalizeString(source.scene_ref),
    primary_view: normalizeString(source.primary_view),
    preset_ref: normalizeString(source.preset_ref),
    camera_policy: normalizeString(source.camera_policy),
    measurement_components: normalizeArray(source.measurement_components),
    reaction_rule_refs: normalizeArray(source.reaction_rule_refs),
    view_authoring_hash: normalizeString(source.view_authoring_hash),
    allowed_controls: normalizeArray(source.allowed_controls),
    question: normalizeString(source.question),
    check_ref: normalizeString(source.check_ref),
    view_plan_ref: normalizeString(source.view_plan_ref),
    anchor_refs: normalizeArray(source.anchor_refs),
  };
}

export function parseLessonPlanToml(text, { fallbackLessonId = "", fallbackTitle = "" } = {}) {
  const root = {};
  const activities = [];
  const diagnostics = [];
  let current = root;
  const lines = String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  lines.forEach((rawLine, index) => {
    const line = String(rawLine ?? "").trim();
    if (!line || line.startsWith("#")) return;
    if (line === "[[activities]]") {
      const row = {};
      activities.push(row);
      current = row;
      return;
    }
    const parsed = parseAssignment(line);
    if (!parsed) {
      diagnostics.push(makeDiagnostic(
        "E_LESSON_PLAN_TOML_LINE",
        `읽을 수 없는 TOML 행입니다: ${line}`,
        { where: `line:${index + 1}`, fix: "key = value 형식으로 고치세요." },
      ));
      return;
    }
    current[parsed.key] = parsed.value;
  });
  const schema = normalizeString(root.schema) || LESSON_PLAN_V0_SCHEMA;
  if (schema !== LESSON_PLAN_V0_SCHEMA) {
    diagnostics.push(makeDiagnostic(
      "E_LESSON_PLAN_SCHEMA",
      `지원하지 않는 수업 구성 schema입니다: ${schema}`,
      { where: "schema", fix: LESSON_PLAN_V0_SCHEMA },
    ));
  }
  const normalizedActivities = activities.map((row, index) => normalizeActivity(row, index, diagnostics));
  if (!normalizedActivities.length) {
    diagnostics.push(makeDiagnostic(
      "E_LESSON_PLAN_NO_ACTIVITY",
      "활동이 하나도 없습니다.",
      { where: "activities", fix: "[[activities]] 블록을 하나 이상 추가하세요." },
    ));
  }
  return {
    schema: LESSON_PLAN_V0_SCHEMA,
    lesson_id: normalizeString(root.lesson_id) || normalizeString(fallbackLessonId),
    lesson_question: normalizeString(root.lesson_question),
    prediction_question: normalizeString(root.prediction_question),
    explanation_question: normalizeString(root.explanation_question),
    misconception_refs: normalizeArray(root.misconception_refs),
    reading_check: normalizeString(root.reading_check),
    rubric: normalizeString(root.rubric),
    deployment_profile_lock: normalizeString(root.deployment_profile_lock),
    allowed_controls: normalizeArray(root.allowed_controls),
    control_ranges: normalizeArray(root.control_ranges),
    primary_view: normalizeString(root.primary_view),
    secondary_views: normalizeArray(root.secondary_views),
    title: normalizeString(root.title) || normalizeString(fallbackTitle) || "수업 구성",
    activities: normalizedActivities,
    diagnostics,
  };
}

function formatTomlArray(values) {
  const rows = normalizeArray(values);
  return `[${rows.map((item) => `"${escapeTomlString(item)}"`).join(", ")}]`;
}

function formatTomlString(value) {
  return `"${escapeTomlString(value)}"`;
}

export function serializeLessonPlanToml(plan) {
  const model = normalizeLessonPlanModel(plan);
  const lines = [];
  ROOT_FIELDS.forEach((field) => {
    const value = field === "schema" ? LESSON_PLAN_V0_SCHEMA : model[field];
    if (ROOT_ARRAY_FIELDS.includes(field)) {
      lines.push(`${field} = ${formatTomlArray(value)}`);
      return;
    }
    if (field === "schema" || field === "lesson_id" || field === "title" || normalizeString(value)) {
      lines.push(`${field} = ${formatTomlString(value)}`);
    }
  });
  lines.push("");
  model.activities.forEach((activity, index) => {
    if (index > 0) lines.push("");
    lines.push("[[activities]]");
    ACTIVITY_FIELDS.forEach((field) => {
      const value = activity[field];
      if (ARRAY_FIELDS.includes(field)) {
        lines.push(`${field} = ${formatTomlArray(value)}`);
        return;
      }
      if (field === "activity_id" || field === "kind" || normalizeString(value)) {
        lines.push(`${field} = ${formatTomlString(value)}`);
      }
    });
  });
  return `${lines.join("\n").replace(/\n{3,}/g, "\n\n")}\n`;
}

export function normalizeLessonPlanModel(rawPlan = {}) {
  const diagnostics = [];
  const source = rawPlan && typeof rawPlan === "object" ? rawPlan : {};
  const activities = Array.isArray(source.activities) ? source.activities : [];
  const normalizedActivities = activities.map((row, index) => normalizeActivity(row, index, diagnostics));
  return {
    schema: LESSON_PLAN_V0_SCHEMA,
    lesson_id: normalizeString(source.lesson_id),
    lesson_question: normalizeString(source.lesson_question),
    prediction_question: normalizeString(source.prediction_question),
    explanation_question: normalizeString(source.explanation_question),
    misconception_refs: normalizeArray(source.misconception_refs),
    reading_check: normalizeString(source.reading_check),
    rubric: normalizeString(source.rubric),
    deployment_profile_lock: normalizeString(source.deployment_profile_lock),
    allowed_controls: normalizeArray(source.allowed_controls),
    control_ranges: normalizeArray(source.control_ranges),
    primary_view: normalizeString(source.primary_view),
    secondary_views: normalizeArray(source.secondary_views),
    title: normalizeString(source.title) || "수업 구성",
    activities: normalizedActivities,
    diagnostics: [
      ...(Array.isArray(source.diagnostics) ? source.diagnostics : []),
      ...diagnostics,
    ],
  };
}

export function roundtripLessonPlanToml(text, options = {}) {
  const first = parseLessonPlanToml(text, options);
  const canonical_text = serializeLessonPlanToml(first);
  const second = parseLessonPlanToml(canonical_text, options);
  return {
    schema: "seamgrim.lesson_plan_roundtrip.v1",
    canonical_text,
    first,
    second,
    byte_stable: canonical_text === serializeLessonPlanToml(second),
    semantic_equal: JSON.stringify(normalizeLessonPlanModel(first)) === JSON.stringify(normalizeLessonPlanModel(second)),
    diagnostics: [...first.diagnostics, ...second.diagnostics],
  };
}

export function buildBlankLessonPlanToml({ lessonId = "custom", title = "새 수업 구성" } = {}) {
  return serializeLessonPlanToml({
    lesson_id: lessonId,
    title,
    lesson_question: "What should students investigate?",
    prediction_question: "What do you predict before running?",
    explanation_question: "What evidence explains the result?",
    misconception_refs: [],
    reading_check: "Name one observation you used.",
    rubric: "Prediction, experiment, evidence, explanation.",
    deployment_profile_lock: "locked",
    allowed_controls: [],
    control_ranges: [],
    primary_view: "",
    secondary_views: [],
    activities: [
      {
        activity_id: "observe-1",
        kind: "observe",
        title: "첫 관찰",
        prompt: "결과를 먼저 관찰하세요.",
        anchor_refs: [],
        allowed_controls: [],
      },
    ],
  });
}

export function getActivityKindLabel(kind) {
  return ACTIVITY_KIND_LABELS[normalizeKind(kind)] || "활동";
}
