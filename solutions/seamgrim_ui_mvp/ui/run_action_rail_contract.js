import {
  buildWarningPrimaryUserMessage,
  resolveWarningGuideTab,
} from "./run_warning_contract.js";
import { SEAMGRIM_FIRST_RUN_PATH_TEXT } from "./first_run_catalog.js";

const PLATFORM_UI_ACTION_FIX_INPUT = "fix_input";
const DEFAULT_ONBOARDING_STATUS_TEXT = `온보딩: ${SEAMGRIM_FIRST_RUN_PATH_TEXT} 순서로 보세요.`;

function normalizeProfile(raw) {
  const text = String(raw ?? "").trim().toLowerCase();
  if (text === "student" || text === "teacher") return text;
  return "";
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

function hasCode(warnings = [], targetCode = "") {
  const list = Array.isArray(warnings) ? warnings : [];
  const target = String(targetCode ?? "").trim();
  if (!target) return false;
  return list.some((warning) => {
    const code = String(warning?.technical_code ?? warning?.code ?? "").trim();
    return code === target;
  });
}

function mapPlatformErrorCodeToStatusText(code = "") {
  const normalized = normalizePlatformErrorCode(code);
  if (normalized === "E_PLATFORM_AUTH_REQUIRED") {
    return "점검 필요: 서버 연동 로그인 필요";
  }
  if (normalized === "E_PLATFORM_PERMISSION_DENIED") {
    return "점검 필요: 서버 저장 권한 필요";
  }
  if (normalized === "E_PLATFORM_VALIDATION_FAILED" || normalized === "E_PLATFORM_INVALID_REQUEST") {
    return "점검 필요: 서버 요청 입력 점검 필요";
  }
  if (normalized) {
    return "점검 필요: 서버 연동 상태 확인 필요";
  }
  return "";
}

export function buildRunActionRailViewModel({
  warnings = [],
  onboardingProfile = "",
  onboardingStatusText = "",
  platformErrorCode = "",
  platformActionRail = [],
} = {}) {
  const list = Array.isArray(warnings) ? warnings : [];
  const profile = normalizeProfile(onboardingProfile);
  const warningMessage = buildWarningPrimaryUserMessage(list);
  const warningGuideTab = resolveWarningGuideTab(list);
  const normalizedPlatformCode = normalizePlatformErrorCode(platformErrorCode);
  const normalizedPlatformActionRail = normalizePlatformActionRail(platformActionRail);
  const hasPlatformWarning = Boolean(normalizedPlatformCode);
  const platformWantsInputFix = normalizedPlatformActionRail.includes(PLATFORM_UI_ACTION_FIX_INPUT);

  let statusText = String(onboardingStatusText ?? "").trim() || DEFAULT_ONBOARDING_STATUS_TEXT;
  let statusLevel = "idle";
  const hasDirectOnlyWarning = hasCode(list, "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED");
  if (warningMessage) {
    statusText = hasDirectOnlyWarning
      ? "점검 필요: WASM 실행 환경을 확인해 주세요."
      : `점검 필요: ${warningMessage}`;
    statusLevel = "warn";
  } else if (hasPlatformWarning) {
    statusText = mapPlatformErrorCodeToStatusText(normalizedPlatformCode) || "점검 필요: 서버 연동 상태 확인 필요";
    statusLevel = "warn";
  } else if (profile) {
    statusText = profile === "student"
      ? "학생 시작 적용됨: 교과 중심 동선"
      : "교사 시작 적용됨: 관찰 중심 동선";
    statusLevel = "ok";
  }

  const recommendStudent = !warningMessage && !hasPlatformWarning && !profile;
  const recommendTeacher = !warningMessage && !hasPlatformWarning && profile === "student";
  const recommendDdn = Boolean(warningMessage && warningGuideTab === "ddn");
  const recommendInspector = Boolean(warningMessage && warningGuideTab === "inspector");
  const platformRecommendDdn = Boolean(!warningMessage && hasPlatformWarning && platformWantsInputFix);
  const platformRecommendInspector = Boolean(!warningMessage && hasPlatformWarning && !platformWantsInputFix);
  const showOnboardingActions = !warningMessage && !hasPlatformWarning;

  return {
    statusText,
    statusLevel,
    actions: {
      onboardStudent: {
        visible: showOnboardingActions,
        recommended: recommendStudent,
      },
      onboardTeacher: {
        visible: showOnboardingActions,
        recommended: recommendTeacher,
      },
      openDdn: {
        visible: true,
        recommended: recommendDdn || platformRecommendDdn,
      },
      openInspector: {
        visible: true,
        recommended: recommendInspector || platformRecommendInspector,
      },
    },
  };
}
