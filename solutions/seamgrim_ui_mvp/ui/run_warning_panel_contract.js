import { buildWarningShortcutHint, resolveWarningGuideTab } from "./run_warning_contract.js";
import { buildUserDiagnosticModel, normalizeUserCategory } from "./studio_edit_run_contract.js";

const PLATFORM_UI_ACTION_FIX_INPUT = "fix_input";
const PLATFORM_UI_ACTION_LOGIN = "login";
const PLATFORM_UI_ACTION_REQUEST_ACCESS = "request_access";
const PLATFORM_UI_ACTION_OPEN_LOCAL_SAVE = "open_local_save";

function normalizeWarningList(warnings = []) {
  if (!Array.isArray(warnings)) return [];
  return warnings.filter((warning) => warning && typeof warning === "object");
}

function normalizePlatformErrorCode(raw) {
  return String(raw ?? "").trim();
}

function normalizePlatformActionRail(raw) {
  if (!Array.isArray(raw)) return [];
  const out = [];
  const seen = new Set();
  raw.forEach((item) => {
    const token = String(item ?? "").trim();
    if (!token || seen.has(token)) return;
    seen.add(token);
    out.push(token);
  });
  return out;
}

function buildPlatformUiActionModel(actionRail = [], token = "", title = "") {
  const normalizedToken = String(token ?? "").trim();
  const visible = normalizedToken ? actionRail.includes(normalizedToken) : false;
  const preferred = visible && actionRail[0] === normalizedToken;
  return {
    hidden: !visible,
    disabled: !visible,
    recommended: preferred,
    title: String(title ?? "").trim(),
  };
}

function resolvePlatformGuideTab(actionRail = []) {
  return actionRail.includes(PLATFORM_UI_ACTION_FIX_INPUT) ? "ddn" : "inspector";
}

function resolvePlatformShortcutsText(actionRail = []) {
  if (resolvePlatformGuideTab(actionRail) === "ddn") {
    return "단축키: Alt+D DDN · Alt+I 검증 (권장: Alt+D)";
  }
  return "단축키: Alt+D DDN · Alt+I 검증 (권장: Alt+I)";
}

function mapPlatformErrorCodeToUserSummary(code = "") {
  const normalized = normalizePlatformErrorCode(code);
  if (normalized === "E_PLATFORM_AUTH_REQUIRED") {
    return "서버 연동을 위해 로그인이 필요합니다. 우선 로컬 저장으로 계속할 수 있습니다.";
  }
  if (normalized === "E_PLATFORM_PERMISSION_DENIED") {
    return "서버 저장 권한이 없습니다. 권한 요청 또는 로컬 저장으로 계속하세요.";
  }
  if (normalized === "E_PLATFORM_VALIDATION_FAILED" || normalized === "E_PLATFORM_INVALID_REQUEST") {
    return "서버 요청 입력이 올바르지 않습니다. DDN/입력값을 점검하세요.";
  }
  if (normalized) {
    return "서버 연동이 준비 중이거나 일시적으로 사용할 수 없습니다.";
  }
  return "";
}

function formatWarningLocation(span) {
  if (!span || typeof span !== "object") return "";
  const line = Number(span.line ?? span.row ?? span.start_line ?? span.startLine);
  const column = Number(span.column ?? span.col ?? span.start_col ?? span.startCol);
  if (Number.isFinite(line) && Number.isFinite(column)) {
    return `L${Math.max(1, Math.trunc(line))}:C${Math.max(1, Math.trunc(column))}`;
  }
  if (Number.isFinite(line)) {
    return `L${Math.max(1, Math.trunc(line))}`;
  }
  return "";
}

function toWarningSignature(warnings = [], platformErrorCode = "", platformActionRail = []) {
  const warningSignature = warnings
    .map((warning) => {
      const code = String(warning?.technical_code ?? warning?.code ?? "").trim();
      const message = String(warning?.message ?? "").trim();
      const technical = String(warning?.technical_message ?? "").trim();
      return `${code}|${message}|${technical}`;
    })
    .join("||");
  const platformCode = normalizePlatformErrorCode(platformErrorCode);
  const platformRail = normalizePlatformActionRail(platformActionRail).join(",");
  return `${warningSignature}##platform:${platformCode}##rail:${platformRail}`;
}

function buildFallbackPrimaryAction(guideTab = "inspector") {
  if (guideTab === "ddn") {
    return {
      kind: "open_ddn",
      label: "DDN 바로 수정",
      detail: "문법 표면을 정본 형태로 맞춤",
    };
  }
  return {
    kind: "open_inspector",
    label: "거울에서 원인 확인",
    detail: "모델 규칙/계약 조건 확인",
  };
}

function buildDefaultViewModel(lastWarningSignature = "") {
  return {
    hasWarnings: false,
    panelLevel: "none",
    userSummary: "",
    userCategory: "실행입력",
    userCause: "",
    primaryAction: {
      kind: "retry",
      label: "수정 후 다시 실행",
      detail: "입력값을 확인한 뒤 재시도",
    },
    retryAction: {
      kind: "retry",
      label: "재실행",
      detail: "현재 입력으로 재시도",
      disabled: false,
    },
    autofixAvailable: false,
    codes: [],
    techSummaryText: "기술 상세",
    techBodyText: "",
    guideTab: "inspector",
    shortcutsText: "단축키: Alt+D DDN · Alt+I 검증",
    ddnAction: {
      disabled: false,
      recommended: false,
      title: "DDN 탭 열기 (Alt+D)",
    },
    inspectorAction: {
      disabled: false,
      recommended: false,
      title: "검증/인스펙터 열기 (Alt+I)",
    },
    platformLoginAction: buildPlatformUiActionModel([], PLATFORM_UI_ACTION_LOGIN, "로그인 (준비 중)"),
    platformRequestAccessAction: buildPlatformUiActionModel([], PLATFORM_UI_ACTION_REQUEST_ACCESS, "권한 요청 (준비 중)"),
    platformOpenLocalSaveAction: buildPlatformUiActionModel([], PLATFORM_UI_ACTION_OPEN_LOCAL_SAVE, "로컬 저장 열기"),
    warningSignature: toWarningSignature([], "", []),
    signatureChanged: String(lastWarningSignature ?? "").trim().length > 0,
  };
}

export function buildWarningPanelViewModel({
  warnings = [],
  lastWarningSignature = "",
  platformErrorCode = "",
  platformActionRail = [],
} = {}) {
  const normalized = normalizeWarningList(warnings);
  const normalizedPlatformCode = normalizePlatformErrorCode(platformErrorCode);
  const normalizedPlatformActionRail = normalizePlatformActionRail(platformActionRail);
  const hasPlatformWarning = Boolean(normalizedPlatformCode);
  if (!normalized.length && !hasPlatformWarning) {
    return buildDefaultViewModel(lastWarningSignature);
  }

  let userSummary = "";
  let userCategory = "실행입력";
  let userCause = "";
  let primaryAction = buildFallbackPrimaryAction("inspector");
  let autofixAvailable = false;
  let codes = [];
  let techSummaryText = "기술 상세";
  let techBodyText = "";
  let guideTab = "inspector";
  let panelLevel = "warn";

  if (normalized.length > 0) {
    const userMessages = [...new Set(
      normalized
        .map((warning) => String(warning?.message ?? "").trim())
        .filter(Boolean),
    )];
    userSummary = userMessages.length <= 1
      ? (userMessages[0] ?? "입력 점검이 필요합니다.")
      : `${userMessages[0]} 외 ${userMessages.length - 1}건`;
    codes = [...new Set(
      normalized
        .map((warning) => String(warning?.technical_code ?? warning?.code ?? "").trim())
        .filter(Boolean),
    )];
    if (hasPlatformWarning) {
      codes = [...new Set([...codes, normalizedPlatformCode])];
    }
    techSummaryText = codes.length
      ? `기술 상세 (코드 ${codes.length}개)`
      : "기술 상세";
    techBodyText = normalized
      .map((warning, index) => {
        const code = String(warning?.technical_code ?? warning?.code ?? "").trim() || `W${index + 1}`;
        const technicalMessage = String(warning?.technical_message ?? "").trim();
        const userMessage = String(warning?.message ?? "").trim();
        const location = formatWarningLocation(warning?.span);
        const message = technicalMessage || userMessage || "세부 정보 없음";
        return location ? `[${code}] (${location}) ${message}` : `[${code}] ${message}`;
      })
      .join("\n");
    if (hasPlatformWarning) {
      const platformLine = `[${normalizedPlatformCode}] action_rail=${normalizedPlatformActionRail.join(",") || "-"}`;
      techBodyText = techBodyText ? `${techBodyText}\n${platformLine}` : platformLine;
    }
    guideTab = resolveWarningGuideTab(normalized);
    panelLevel = normalized.some(
      (warning) => String(warning?.technical_code ?? warning?.code ?? "").startsWith("E_"),
    )
      ? "error"
      : "warn";

    const first = normalized[0] ?? {};
    const autofixOnWarnings = normalized.some((warning) => Boolean(warning?.autofix_available));
    const diag = buildUserDiagnosticModel({
      code: String(first?.technical_code ?? first?.code ?? "").trim(),
      userMessage: String(first?.message ?? first?.user_message ?? "").trim(),
      technicalMessage: String(first?.technical_message ?? "").trim(),
      autofixAvailable: autofixOnWarnings,
    });
    userCategory = normalizeUserCategory(diag?.user_category ?? "실행입력");
    userCause = String(diag?.user_cause ?? userSummary).trim() || userSummary;
    primaryAction = diag?.primary_action && typeof diag.primary_action === "object"
      ? diag.primary_action
      : buildFallbackPrimaryAction(guideTab);
    autofixAvailable = Boolean(diag?.autofix_available);
  } else {
    userSummary = mapPlatformErrorCodeToUserSummary(normalizedPlatformCode);
    codes = normalizedPlatformCode ? [normalizedPlatformCode] : [];
    techSummaryText = codes.length ? `기술 상세 (코드 ${codes.length}개)` : "기술 상세";
    techBodyText = normalizedPlatformCode
      ? `[${normalizedPlatformCode}] action_rail=${normalizedPlatformActionRail.join(",") || "-"}`
      : "";
    guideTab = resolvePlatformGuideTab(normalizedPlatformActionRail);
    userCategory = normalizeUserCategory(guideTab === "ddn" ? "문법" : "실행입력");
    userCause = userSummary;
    primaryAction = buildFallbackPrimaryAction(guideTab);
    panelLevel = "warn";
  }

  const warningSignature = toWarningSignature(normalized, normalizedPlatformCode, normalizedPlatformActionRail);
  const signatureChanged = warningSignature !== String(lastWarningSignature ?? "");
  const primaryKind = String(primaryAction?.kind ?? "").trim().toLowerCase();
  const primaryOnDdn = primaryKind === "open_ddn" || primaryKind === "autofix" || guideTab === "ddn";
  const primaryOnInspector = primaryKind === "open_inspector" || guideTab === "inspector";

  return {
    hasWarnings: true,
    panelLevel,
    userSummary,
    userCategory,
    userCause,
    primaryAction,
    retryAction: {
      kind: "retry",
      label: "재실행",
      detail: "수정 후 다시 실행",
      disabled: false,
    },
    autofixAvailable,
    codes,
    techSummaryText,
    techBodyText,
    guideTab,
    shortcutsText: normalized.length > 0
      ? buildWarningShortcutHint(normalized)
      : resolvePlatformShortcutsText(normalizedPlatformActionRail),
    ddnAction: {
      disabled: false,
      recommended: primaryOnDdn,
      title: "DDN 탭 열기 (Alt+D)",
    },
    inspectorAction: {
      disabled: false,
      recommended: primaryOnInspector,
      title: "검증/인스펙터 열기 (Alt+I)",
    },
    platformLoginAction: buildPlatformUiActionModel(
      normalizedPlatformActionRail,
      PLATFORM_UI_ACTION_LOGIN,
      "로그인 (준비 중)",
    ),
    platformRequestAccessAction: buildPlatformUiActionModel(
      normalizedPlatformActionRail,
      PLATFORM_UI_ACTION_REQUEST_ACCESS,
      "권한 요청 (준비 중)",
    ),
    platformOpenLocalSaveAction: buildPlatformUiActionModel(
      normalizedPlatformActionRail,
      PLATFORM_UI_ACTION_OPEN_LOCAL_SAVE,
      "로컬 저장 열기",
    ),
    warningSignature,
    signatureChanged,
  };
}
