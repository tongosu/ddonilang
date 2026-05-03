export function normalizeDiagnosticSeverity(warning) {
  const raw = String(warning?.severity ?? warning?.level ?? "").trim().toLowerCase();
  if (raw === "error" || raw === "err" || raw === "fatal") return "error";
  if (raw === "warn" || raw === "warning") return "warn";
  if (raw === "info" || raw === "information") return "info";
  const code = String(
    warning?.technical_code ?? warning?.code ?? warning?.kind ?? "",
  ).trim().toUpperCase();
  if (code.startsWith("E_")) return "error";
  if (code.startsWith("W_")) return "warn";
  return "info";
}

export function mapPlayDiagnosticToUserMessage(code, technicalMessage) {
  const normalizedCode = String(code ?? "").trim();
  const technical = String(technicalMessage ?? "").trim();
  if (normalizedCode === "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED") {
    return "WASM 직접 실행 전용 모드에서 실패했습니다. 입력을 점검해 주세요.";
  }
  if (normalizedCode === "E_RUNTIME_EXEC_FAILED" || normalizedCode === "E_RUNTIME") {
    return "실행 중 오류가 발생했습니다. 문법/입력값을 확인해 주세요.";
  }
  if (normalizedCode === "E_BLOCK_HEADER_COLON_FORBIDDEN") {
    return "블록 헤더에는 ':'를 쓰지 않습니다. 예: 채비 {}";
  }
  if (normalizedCode === "E_BLOCK_HEADER_HASH_FORBIDDEN") {
    return "active 실행선에서는 # 헤더를 사용할 수 없습니다. 설정 {}와 매김 {}를 사용하세요.";
  }
  if (normalizedCode === "E_IMPORT_ALIAS_DUPLICATE") {
    return "import 별칭이 중복되었습니다. 별칭 이름을 고유하게 바꿔 주세요.";
  }
  if (normalizedCode === "E_RECEIVE_OUTSIDE_IMJA") {
    return "받기(수신) 구문은 임자 블록 안에서만 사용할 수 있습니다.";
  }
  const lower = technical.toLowerCase();
  if (lower.includes("expected '.' or newline")) {
    return "문장 끝에 '.' 또는 줄바꿈이 필요합니다.";
  }
  if (lower.includes("unexpected token")) {
    return "예상하지 못한 기호가 있습니다. 문장/기호 배치를 확인해 주세요.";
  }
  if (technical) return technical;
  return "문법/실행 입력을 점검해 주세요.";
}

export function normalizeDiagnosticItem(warning) {
  if (!warning || typeof warning !== "object") return null;
  const technicalCode = String(warning.technical_code ?? warning.code ?? warning.kind ?? "").trim();
  const technicalMessage = String(
    warning.technical_message ?? warning.message ?? warning.error ?? warning.msg ?? "",
  ).trim();
  const userMessageRaw = String(warning.user_message ?? warning.userMessage ?? "").trim();
  const userMessage = userMessageRaw || mapPlayDiagnosticToUserMessage(technicalCode, technicalMessage);
  const severity = normalizeDiagnosticSeverity({ ...warning, code: technicalCode });
  return {
    ...warning,
    code: technicalCode,
    technical_code: technicalCode,
    technical_message: technicalMessage,
    message: userMessage,
    severity,
  };
}
