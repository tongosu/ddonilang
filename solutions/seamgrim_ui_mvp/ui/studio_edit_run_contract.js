const STUDIO_STAGE_READY = "ready";
const STUDIO_STAGE_AUTOFIX = "autofix";
const STUDIO_STAGE_BLOCKED = "blocked";

const USER_CATEGORY_GRAMMAR = "문법";
const USER_CATEGORY_INPUT = "실행입력";
const USER_CATEGORY_LOGIC = "모델논리";

function normalizeText(raw) {
  return String(raw ?? "").trim();
}

function normalizeCode(raw) {
  return normalizeText(raw).toUpperCase();
}

function includesAny(text, needles = []) {
  const source = normalizeText(text);
  if (!source) return false;
  return needles.some((needle) => source.includes(String(needle ?? "")));
}

export function classifyWarningCategory(code = "") {
  const token = normalizeCode(code);
  if (!token) return USER_CATEGORY_INPUT;
  if (token.startsWith("E_PARSE") || token.startsWith("W_PARSE") || token.startsWith("W_BLOCK_HEADER")) {
    return USER_CATEGORY_GRAMMAR;
  }
  if (token.startsWith("E_RUNTIME") || token.startsWith("E_WASM")) {
    return USER_CATEGORY_INPUT;
  }
  return USER_CATEGORY_LOGIC;
}

export function normalizeUserCategory(raw = "") {
  const text = normalizeText(raw);
  if (!text) return USER_CATEGORY_INPUT;
  if (text === USER_CATEGORY_GRAMMAR || text === USER_CATEGORY_INPUT || text === USER_CATEGORY_LOGIC) {
    return text;
  }
  return USER_CATEGORY_INPUT;
}

function resolvePrimaryActionForCategory(category, { autofixAvailable = false } = {}) {
  const normalized = normalizeUserCategory(category);
  if (autofixAvailable) {
    return {
      kind: "autofix",
      label: "자동 수정 적용",
      detail: "구식 표면을 정본 형태로 변환",
    };
  }
  if (normalized === USER_CATEGORY_GRAMMAR) {
    return {
      kind: "open_ddn",
      label: "DDN 바로 수정",
      detail: "문법 표면을 정본 형태로 맞춤",
    };
  }
  if (normalized === USER_CATEGORY_LOGIC) {
    return {
      kind: "open_inspector",
      label: "거울에서 원인 확인",
      detail: "모델 규칙/계약 조건 확인",
    };
  }
  return {
    kind: "retry",
    label: "수정 후 다시 실행",
    detail: "입력값/실행 환경 재점검 후 재시도",
  };
}

export function buildUserDiagnosticModel({
  code = "",
  userMessage = "",
  technicalMessage = "",
  autofixAvailable = false,
} = {}) {
  const normalizedCode = normalizeText(code);
  const category = classifyWarningCategory(normalizedCode);
  const cause = normalizeText(userMessage) || normalizeText(technicalMessage) || "원인을 확인해 주세요.";
  const primaryAction = resolvePrimaryActionForCategory(category, { autofixAvailable });
  return {
    user_category: category,
    user_cause: cause,
    primary_action: primaryAction,
    autofix_available: Boolean(autofixAvailable),
    technical_code: normalizedCode,
  };
}

function detectLegacyBlockingReason(sourceText = "") {
  const source = String(sourceText ?? "");
  if (!source.trim()) {
    return {
      matched: true,
      user_category: USER_CATEGORY_INPUT,
      user_cause: "DDN 내용이 비어 있습니다.",
      manual_example: "기본 초안을 넣고 다시 실행하세요. 예: 채비 { 값:수 <- 0. }.",
    };
  }
  if (includesAny(source, ["보개장면"])) {
    return {
      matched: true,
      user_category: USER_CATEGORY_GRAMMAR,
      user_cause: "`보개장면`은 작업실 실행선에서 허용되지 않습니다.",
      manual_example: "`보개마당 { ... }` 표면으로 바꿔 주세요.",
    };
  }
  if (includesAny(source, ["붙박이마련", "그릇채비", "붙박이채비"])) {
    return {
      matched: true,
      user_category: USER_CATEGORY_GRAMMAR,
      user_cause: "레거시 선언 블록 표면은 작업실에서 허용되지 않습니다.",
      manual_example: "`채비 { ... }` 블록으로 통일해 주세요.",
    };
  }
  return {
    matched: false,
    user_category: USER_CATEGORY_INPUT,
    user_cause: "",
    manual_example: "",
  };
}

export function buildStudioEditorReadinessModel({
  sourceText = "",
  canonDiagCode = "",
  canonDiagMessage = "",
  autofixAvailable = false,
} = {}) {
  const blocking = detectLegacyBlockingReason(sourceText);
  if (blocking.matched && !autofixAvailable) {
    return {
      stage: STUDIO_STAGE_BLOCKED,
      user_category: blocking.user_category,
      user_cause: blocking.user_cause,
      primary_action: {
        kind: "manual_fix_example",
        label: "수정 예시 보기",
        detail: blocking.manual_example,
      },
      autofix_available: false,
      blocking_remaining: true,
      manual_example: blocking.manual_example,
    };
  }

  if (autofixAvailable) {
    return {
      stage: STUDIO_STAGE_AUTOFIX,
      user_category: USER_CATEGORY_GRAMMAR,
      user_cause: "구식 표면이 감지되었습니다. 실행 전에 정본으로 전환하세요.",
      primary_action: {
        kind: "autofix",
        label: "자동 수정 적용",
        detail: "변환 후 바로 실행",
      },
      autofix_available: true,
      blocking_remaining: false,
      manual_example: "",
    };
  }

  const diagCode = normalizeText(canonDiagCode);
  const diagMessage = normalizeText(canonDiagMessage);
  if (diagCode) {
    const diag = buildUserDiagnosticModel({
      code: diagCode,
      userMessage: "",
      technicalMessage: diagMessage,
      autofixAvailable: false,
    });
    return {
      stage: STUDIO_STAGE_BLOCKED,
      user_category: diag.user_category,
      user_cause: diag.user_cause,
      primary_action: diag.primary_action,
      autofix_available: false,
      blocking_remaining: true,
      manual_example: "",
    };
  }

  return {
    stage: STUDIO_STAGE_READY,
    user_category: USER_CATEGORY_INPUT,
    user_cause: "입력 준비됨",
    primary_action: {
      kind: "run",
      label: "바로 실행",
      detail: "현재 입력으로 실행",
    },
    autofix_available: false,
    blocking_remaining: false,
    manual_example: "",
  };
}

export function buildAutofixResultContract(rawResult = {}, { sourceTextAfter = "" } = {}) {
  const result = rawResult && typeof rawResult === "object" ? rawResult : {};
  const changed = Boolean(result.changed);
  const stats = result.stats && typeof result.stats === "object" ? result.stats : {};
  const appliedRules = Object.entries(stats)
    .filter(([, value]) => Number(value) > 0)
    .map(([name, value]) => `${name}:${Number(value)}`);
  const remaining = detectLegacyBlockingReason(sourceTextAfter);
  return {
    changed,
    blocking_remaining: Boolean(remaining.matched),
    applied_rules: appliedRules,
    next_action: changed ? "수정 후 다시 실행" : "수동 수정 필요",
  };
}

export const STUDIO_READINESS_STAGE = Object.freeze({
  READY: STUDIO_STAGE_READY,
  AUTOFIX: STUDIO_STAGE_AUTOFIX,
  BLOCKED: STUDIO_STAGE_BLOCKED,
});

