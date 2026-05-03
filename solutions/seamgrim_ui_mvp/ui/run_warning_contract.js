export function normalizeWarningCode(raw) {
  return String(raw ?? "").trim();
}

export function classifyToUserCategory(code = "") {
  const token = String(code ?? "").trim().toUpperCase();
  if (!token) return "입력";
  if (token.startsWith("E_PARSE") || token.startsWith("W_PARSE") || token.startsWith("W_BLOCK_HEADER")) {
    return "문법";
  }
  if (token.startsWith("E_RUNTIME") || token.startsWith("E_WASM")) {
    return "입력";
  }
  return "논리";
}

export function resolveUserWarningCause(warning = null) {
  const row = warning && typeof warning === "object" ? warning : {};
  const code = String(row?.code ?? row?.technical_code ?? "").trim();
  const userMessage = String(row?.user_message ?? row?.userMessage ?? "").trim();
  if (userMessage) return userMessage;
  const message = String(row?.message ?? "").trim();
  if (message) return message;
  const technicalMessage = String(row?.technical_message ?? row?.technicalMessage ?? "").trim();
  return mapParseWarningToUserMessage(code, technicalMessage);
}

function mapParseTechnicalMessageToUserMessage(technicalMessage) {
  const message = String(technicalMessage ?? "").trim();
  if (!message) return "문법/실행 입력을 점검해 주세요.";
  const lower = message.toLowerCase();
  if (lower.includes("expected '.' or newline")) {
    return "문장 끝에 '.' 또는 줄바꿈이 필요합니다.";
  }
  if (lower.includes("unexpected token")) {
    return "예상하지 못한 기호가 있습니다. 문장/기호 배치를 확인해 주세요.";
  }
  if (message.includes("닫는 괄호")) {
    return "닫는 괄호 ')'가 필요합니다.";
  }
  if (message.includes("닫는 중괄호")) {
    return "닫는 중괄호 '}'가 필요합니다.";
  }
  if (message.includes("블록 헤더") && message.includes(":")) {
    return "블록 헤더에는 ':'를 쓰지 않습니다. 예: 채비 {}";
  }
  return "문법/실행 입력을 점검해 주세요.";
}

export function mapParseWarningToUserMessage(code, technicalMessage) {
  const normalizedCode = String(code ?? "").trim();
  const normalizedTechnicalMessage = String(technicalMessage ?? "").trim().toLowerCase();
  if (normalizedCode === "E_WASM_CANON_JSON_PARSE_FAILED") {
    if (normalizedTechnicalMessage.includes("flat canonical")) {
      return "구성 해석에 실패했습니다. 실행은 계속할 수 있습니다.";
    }
    if (normalizedTechnicalMessage.includes("maegim canonical")) {
      return "매김 계획 해석에 실패했습니다. 실행은 계속할 수 있습니다.";
    }
    if (normalizedTechnicalMessage.includes("alrim canonical")) {
      return "알림 계획 해석에 실패했습니다. 실행은 계속할 수 있습니다.";
    }
    if (normalizedTechnicalMessage.includes("block_editor canonical")) {
      return "블록 편집 계획 해석에 실패했습니다. 실행은 계속할 수 있습니다.";
    }
    return "구성 해석에 실패했습니다. 실행은 계속할 수 있습니다.";
  }
  if (normalizedCode === "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED") {
    return "WASM 직접 실행 전용 모드로 서버 보조 실행이 차단되었습니다. 진단 모드로 전환하거나 입력을 점검하세요.";
  }
  if (normalizedCode === "E_RUNTIME_EXEC_FAILED") {
    return "실행에 실패했습니다. 문법과 입력값을 점검한 뒤 다시 시도해 주세요.";
  }
  if (normalizedCode === "E_BLOCK_HEADER_COLON_FORBIDDEN") {
    return "블록 헤더에는 ':'를 쓰지 않습니다. 예: 채비 {}";
  }
  if (normalizedCode === "W_BLOCK_HEADER_COLON_DEPRECATED") {
    return "블록 헤더의 ':' 표기는 구식입니다. 예: `채비 { ... }` 형태로 바꿔 주세요.";
  }
  if (normalizedCode === "E_BLOCK_HEADER_HASH_FORBIDDEN") {
    return "active 실행선에서는 # 헤더를 사용할 수 없습니다. 설정 {}와 매김 {}를 사용하세요.";
  }
  if (normalizedCode === "E_SEAMGRIM_RETIRED_LINE_BLOCKED") {
    return "이 자료는 현재 실행선에서 비활성화되었습니다. 대표 교과 목록에서 다시 선택해 주세요.";
  }
  if (normalizedCode === "E_LANG_COMPAT_MATIC_ENTRY_DISABLED") {
    return "레거시 마틱 진입 표면은 비활성화되었습니다. 현재 표면으로 수정해 주세요.";
  }
  if (normalizedCode === "E_EVENT_SURFACE_ALIAS_FORBIDDEN") {
    return "이벤트 별칭 표면은 허용되지 않습니다. canonical 이벤트 표면을 사용하세요.";
  }
  if (normalizedCode === "E_EFFECT_SURFACE_ALIAS_FORBIDDEN") {
    return "효과 별칭 표면은 허용되지 않습니다. canonical 효과 표면을 사용하세요.";
  }
  if (normalizedCode === "E_IMPORT_ALIAS_DUPLICATE") {
    return "import 별칭이 중복되었습니다. 별칭 이름을 고유하게 바꿔 주세요.";
  }
  if (normalizedCode === "E_IMPORT_ALIAS_RESERVED") {
    return "예약된 이름은 import 별칭으로 사용할 수 없습니다.";
  }
  if (normalizedCode === "E_IMPORT_PATH_INVALID") {
    return "import 경로 형식이 올바르지 않습니다. 경로 표기를 확인해 주세요.";
  }
  if (normalizedCode === "E_IMPORT_VERSION_CONFLICT") {
    return "동일 패키지의 import 버전이 충돌합니다. 버전을 하나로 맞춰 주세요.";
  }
  if (normalizedCode === "E_EXPORT_BLOCK_DUPLICATE") {
    return "export 블록이 중복되었습니다. 하나만 유지해 주세요.";
  }
  if (normalizedCode === "E_RECEIVE_OUTSIDE_IMJA") {
    return "받기(수신) 구문은 임자 블록 안에서만 사용할 수 있습니다.";
  }
  if (normalizedCode === "W_CANON_FALLBACK_LANG_NORMALIZER" || normalizedCode === "W_CANON_PASSTHROUGH") {
    return "정본화 경로가 strict 모드로 처리되지 못했습니다. 입력 표면을 점검하세요.";
  }
  if (normalizedCode === "E_PARSE_UNEXPECTED_TOKEN") {
    return "예상하지 못한 기호가 있습니다. 문장/기호 배치를 확인해 주세요.";
  }
  if (normalizedCode === "E_PARSE_EXPECTED_EXPR") {
    return "식이 필요한 위치입니다. 값을 넣거나 수식을 완성해 주세요.";
  }
  if (normalizedCode === "E_PARSE_EXPECTED_TARGET") {
    return "대입 대상이 필요합니다. 왼쪽에 변수/경로를 지정해 주세요.";
  }
  if (normalizedCode === "E_PARSE_ROOT_HIDE_UNDECLARED") {
    return "숨김 처리할 경로가 선언되어 있지 않습니다. 경로를 먼저 선언해 주세요.";
  }
  if (normalizedCode === "E_PARSE_UNSUPPORTED_COMPOUND_TARGET") {
    return "복합 대입 대상은 지원되지 않습니다. 단일 경로로 작성해 주세요.";
  }
  if (normalizedCode === "E_PARSE_EXPECTED_PATH") {
    return "경로가 필요한 위치입니다. 대상 경로를 지정해 주세요.";
  }
  if (normalizedCode === "E_PARSE_EXPECTED_UNIT") {
    return "단위 표기가 필요합니다. 예: m, s 같은 단위를 확인해 주세요.";
  }
  if (normalizedCode === "E_PARSE_TENSOR_SHAPE") {
    return "텐서 모양(shape) 표기가 올바르지 않습니다. 차원/형식을 점검해 주세요.";
  }
  if (normalizedCode === "E_PARSE_EXPECTED_RPAREN") {
    return "닫는 괄호 ')'가 필요합니다.";
  }
  if (normalizedCode === "E_PARSE_EXPECTED_RBRACE") {
    return "닫는 중괄호 '}'가 필요합니다.";
  }
  if (normalizedCode === "E_PARSE_CALL_JOSA_AMBIGUOUS") {
    return "조사(josa) 해석이 모호합니다. 호출 표면을 명확히 적어 주세요.";
  }
  if (normalizedCode === "E_PARSE_CALL_PIN_DUPLICATE") {
    return "동일한 핀(pin)이 중복되었습니다. 핀 이름을 고유하게 맞춰 주세요.";
  }
  if (normalizedCode === "E_PARSE_CASE_COMPLETION_REQUIRED") {
    return "경우 나눔 구문이 완결되지 않았습니다. 모든 경우를 채워 주세요.";
  }
  if (normalizedCode === "E_PARSE_CASE_ELSE_NOT_LAST") {
    return "case의 else는 마지막에 와야 합니다.";
  }
  if (normalizedCode === "E_PARSE_DEFERRED_ASSIGN_OUTSIDE_BEAT") {
    return "지연 대입은 (매마디)마다 블록 안에서만 사용할 수 있습니다.";
  }
  if (normalizedCode === "E_PARSE_COMPAT_EQUAL_DISABLED") {
    return "호환 '=' 대입은 비활성화되었습니다. '<-'를 사용하세요.";
  }
  if (normalizedCode.startsWith("E_PARSE_QUANTIFIER_")) {
    return "양화 구문에서 허용되지 않은 조작입니다. 표현식을 점검해 주세요.";
  }
  if (normalizedCode.startsWith("E_PARSE_IMMEDIATE_PROOF_")) {
    return "즉시 증명 블록에서 허용되지 않은 구문입니다. 블록 규칙을 확인해 주세요.";
  }
  if (normalizedCode.startsWith("E_PARSE_LIFECYCLE_")) {
    return "생애주기 이름/선언이 중복되었거나 올바르지 않습니다.";
  }
  if (normalizedCode.startsWith("E_PARSE_MAEGIM_")) {
    return "매김 {} 문법이 올바르지 않습니다. 묶음/구조를 점검해 주세요.";
  }
  if (normalizedCode.startsWith("E_PARSE_HOOK_EVERY_N_MADI_")) {
    return "(N마디)마다 표면이 올바르지 않습니다. 간격/단위를 점검해 주세요.";
  }
  if (normalizedCode.startsWith("E_PARSE_")) {
    return mapParseTechnicalMessageToUserMessage(technicalMessage);
  }
  if (normalizedCode) {
    return "문법/실행 입력을 점검해 주세요.";
  }
  return mapParseTechnicalMessageToUserMessage(technicalMessage);
}

export function resolveWarningGuideTab(warnings = []) {
  const codes = Array.isArray(warnings)
    ? warnings.map((warning) => normalizeWarningCode(warning?.code)).filter(Boolean)
    : [];
  if (!codes.length) return "inspector";
  if (
    codes.some((code) => [
      "E_BLOCK_HEADER_COLON_FORBIDDEN",
      "E_BLOCK_HEADER_HASH_FORBIDDEN",
      "E_RUNTIME_EXEC_FAILED",
      "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED",
    ].includes(code))
  ) {
    return "ddn";
  }
  return "inspector";
}

export function buildWarningActionHint(warnings = []) {
  const guideTab = resolveWarningGuideTab(warnings);
  if (guideTab === "ddn") {
    return "다음: DDN 탭에서 문법을 수정하세요.";
  }
  if (Array.isArray(warnings) && warnings.length > 0) {
    return "다음: 검증/인스펙터 탭에서 상세를 확인하세요.";
  }
  return "";
}

export function buildWarningShortcutHint(warnings = []) {
  const guideTab = resolveWarningGuideTab(warnings);
  if (guideTab === "ddn") {
    return "단축키: Alt+D DDN · Alt+I 검증 (권장: Alt+D)";
  }
  return "단축키: Alt+D DDN · Alt+I 검증 (권장: Alt+I)";
}

export function buildWarningPrimaryUserMessage(warnings = []) {
  const list = Array.isArray(warnings) ? warnings : [];
  for (const warning of list) {
    const user = resolveUserWarningCause(warning);
    if (user) return user;
  }
  return "";
}

export function resolveRuntimeGuideText({ warnings = [], observeGuideText = "" } = {}) {
  const warningMessage = buildWarningPrimaryUserMessage(warnings);
  if (warningMessage) return `문제: ${warningMessage}`;
  const warningGuide = buildWarningActionHint(warnings);
  if (warningGuide) return warningGuide;
  return String(observeGuideText ?? "").trim();
}
