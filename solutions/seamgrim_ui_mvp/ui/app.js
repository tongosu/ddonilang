import { createWasmLoader, applyWasmLogicAndDispatchState } from "./wasm_page_common.js";
import { createLessonCanonHydrator, buildFlatPlanView } from "./runtime/index.js";
import { BrowseScreen } from "./screens/browse.js";
import { mountLearningSpineIndex } from "./learning_spine_index.js?v=20260710-first-delight-w6a";
import { EditorScreen, saveDdnToFile } from "./screens/editor.js";
import { RunScreen, applyLegacyAutofixToDdn, hasLegacyAutofixCandidate } from "./screens/run.js?v=20260710-first-delight-w5a";
import { normalizeViewFamilyList } from "./view_family_contract.js";
import {
  mergeCurriculumCardsIntoLessons,
  validateCurriculumCardBundle,
} from "./curriculum_cards.js";
import {
  buildAutofixResultContract,
  buildStudioEditorReadinessModel,
  STUDIO_READINESS_STAGE,
} from "./studio_edit_run_contract.js";
import {
  parseTomlMeta,
  parseLessonDdnMetaHeader,
  resolveLessonDisplayMeta,
} from "./lesson_loader_contract.js";
import {
  parseLessonPlanToml,
  serializeLessonPlanToml,
} from "./lesson_plan_v0.js";

const LESSON_PLAN_DRAFT_KEY = "seamgrim.lesson_plan.v0.draft";
const PROJECT_RECENT_STORAGE_KEY = "seamgrim.project.recent.v0";

function lessonPlanDraftKey(lessonId = "") {
  const id = String(lessonId ?? "").trim();
  return id ? `${LESSON_PLAN_DRAFT_KEY}:${id}` : LESSON_PLAN_DRAFT_KEY;
}

function readLessonPlanDraft(lessonId = "") {
  if (typeof window === "undefined") return "";
  const id = String(lessonId ?? "").trim();
  try {
    const candidates = [
      window.localStorage?.getItem?.(lessonPlanDraftKey(id)) ?? "",
      window.localStorage?.getItem?.(LESSON_PLAN_DRAFT_KEY) ?? "",
    ];
    for (const candidate of candidates) {
      const text = String(candidate ?? "");
      if (!text.trim()) continue;
      const parsed = parseLessonPlanToml(text, { fallbackLessonId: id });
      if (!id || String(parsed.lesson_id ?? "").trim() === id) return text;
    }
  } catch (_) {
    // Browser storage may be unavailable in restricted contexts.
  }
  return "";
}
import {
  buildLessonLibraryCurationSnapshot,
  formatLessonLibraryCurationText,
} from "./lesson_library_curation.js";
import {
  buildNumericTrackIndexSnapshot,
  buildNumericTrackReportExport,
  formatNumericTrackIndexText,
  formatNumericTrackReportExportText,
} from "./numeric_curriculum_track.js";
import {
  buildStudioLocalPackageManifest,
  buildStudioLocalPackagePayload,
  formatStudioLocalPackageError,
  formatStudentSubmissionExchangeError,
  importStudioLocalPackagePayload,
  validateStudentSubmissionExchangeV0,
} from "./studio_local_share_package.js";
import {
  buildProjectSnapshotArtifactV0,
  buildProjectSnapshotInputFromEditorState,
} from "./project_snapshot.js";
import {
  PROJECT_API_BASE,
  PROJECT_CAPABILITY_SESSION_KEY,
  buildProjectCapabilityFragment,
  captureProjectCapabilityFromLocation,
  readProjectCapabilitySession,
  requestProjectApi,
} from "./project_service_client.js";
import {
  DEFAULT_SEULGI_PROPOSAL_UI_ROWS,
  buildSeulgiProposalUi,
  formatSeulgiProposalUiText,
  renderSeulgiProposalUi,
} from "./seulgi_proposal_ui.js";
import {
  DEFAULT_SEULGI_REPLAY_SAFE_ROWS,
  buildSeulgiReplaySafeWorkflow,
  formatSeulgiReplaySafeWorkflowText,
  renderSeulgiReplaySafeWorkflow,
} from "./seulgi_replay_safe_workflow.js";
import {
  resolveAvailableFeaturedSeedIds,
  pickNextFeaturedSeedLaunch,
  shouldTriggerFeaturedSeedQuickLaunch,
  shouldTriggerFeaturedSeedQuickPreset,
} from "./featured_seed_quick_launch.js";
import { FEATURED_SEED_IDS } from "./featured_seed_catalog.js";
import { buildLegacyGuideDraftText } from "./legacy_warning_guide.js";
import {
  buildOverlaySessionRunsPayload,
  buildOverlayCompareSessionPayload,
  resolveOverlayCompareFromSession,
  buildSessionViewComboPayload,
  resolveSessionViewComboFromPayload,
} from "./overlay_session_contract.js";
import {
  createInputRegistryState,
  registerDdnInput,
  registerLessonInput,
  restoreInputRegistrySession,
  serializeInputRegistrySession,
} from "./input_registry.js";
import {
  buildSceneSummarySnapshot,
  restoreSceneSummarySession,
  serializeSceneSummarySession,
} from "./scene_summary_contract.js";
import { buildRuntimeSnapshotBundleV0 } from "./snapshot_session_contract.js";
import {
  CatalogKind,
  ObjectKind,
  PublishPolicy,
  PublicationPolicy,
  RouteSlotPolicy,
  RevisionPolicy,
  ShareKind,
  SourceManagementPolicy,
  Visibility,
} from "./platform_contract.js";
import {
  buildMockInstallPackagePayload,
  buildMockPublishAdapterPayload,
  buildMockRestoreRevisionPayload,
  buildMockSaveAdapterPayload,
  buildMockShareAdapterPayload,
  buildMockSwitchCatalogPayload,
} from "./platform_mock_adapter_contract.js";
import {
  PlatformServerAdapterOp,
  PlatformServerAdapterUiAction,
  buildServerAdapterErrorResponse,
  buildServerAdapterRequest,
  mapServerOpToNotReadyErrorCode,
  resolveServerErrorActionRail,
} from "./platform_server_adapter_contract.js";

const PROJECT_PREFIX = "solutions/seamgrim_ui_mvp/";
const DATA_ROOT = normalizeDataRoot(globalThis?.SEAMGRIM_DATA_ROOT ?? "");
const ACTIVE_ALLOWLIST_PATH = dataPath("lessons/active_allowlist.detjson");
const CATALOG_MODE_REPS_ONLY = "reps_only";
const CATALOG_MODE_FULL = "full";
const SIM_CORE_POLICY_CLASS = "policy-sim-core";
const OVERLAY_SESSION_STORAGE_KEY = "seamgrim.overlay_session.v1";
const INPUT_REGISTRY_STORAGE_KEY = "seamgrim.input_registry.v0";
const SCENE_SUMMARY_STORAGE_KEY = "seamgrim.scene_summary.v0";
const SNAPSHOT_V0_STORAGE_KEY = "seamgrim.snapshot.v0";
const SESSION_V0_STORAGE_KEY = "seamgrim.session.v0";
const BROWSE_PRESET_QUERY_KEY = "browsePreset";
const FIRST_DELIGHT_QUERY_KEY = "first_delight";
const LEARNING_SPINE_QUERY_KEY = "learning_spine";
const FIRST_DELIGHT_PROJECTILE_ID = "rep_phys_projectile_xy_v1";
const FIRST_DELIGHT_PENDULUM_ID = "rep_phys_pendulum_wave1_v1";
const FOUNDATION12_LESSON_IDS = Object.freeze([
  "elem_math_number_line",
  "elem_math_fraction_line",
  "elem_math_rectangle_area",
  "elem_science_daily_temperature",
  "elem_econ_allowance_budget",
  "elem_econ_store_stock",
  "rep_math_function_line_v1",
  "rep_phys_pendulum_wave1_v1",
  "rep_phys_projectile_xy_v1",
  "rep_econ_supply_demand_tax_v1",
  "rep_phys_damped_oscillation_v1",
  "high_math_derivative",
]);
const FIRST_DELIGHT_LESSON_BY_ALIAS = Object.freeze({
  "1": FIRST_DELIGHT_PROJECTILE_ID,
  true: FIRST_DELIGHT_PROJECTILE_ID,
  projectile: FIRST_DELIGHT_PROJECTILE_ID,
  physics_projectile: FIRST_DELIGHT_PROJECTILE_ID,
  [FIRST_DELIGHT_PROJECTILE_ID]: FIRST_DELIGHT_PROJECTILE_ID,
  pendulum: FIRST_DELIGHT_PENDULUM_ID,
  physics_pendulum: FIRST_DELIGHT_PENDULUM_ID,
  "wave1-pendulum": FIRST_DELIGHT_PENDULUM_ID,
  [FIRST_DELIGHT_PENDULUM_ID]: FIRST_DELIGHT_PENDULUM_ID,
  line: "rep_math_function_line_v1",
  "linear-function": "rep_math_function_line_v1",
  math_function_line: "rep_math_function_line_v1",
  rep_math_function_line_v1: "rep_math_function_line_v1",
  economy: "rep_econ_supply_demand_tax_v1",
  "market-tax": "rep_econ_supply_demand_tax_v1",
  supply_demand_tax: "rep_econ_supply_demand_tax_v1",
  econ_supply_demand_tax: "rep_econ_supply_demand_tax_v1",
  rep_econ_supply_demand_tax_v1: "rep_econ_supply_demand_tax_v1",
});
const BROWSE_UI_PREFS_STORAGE_KEY = "seamgrim.ui.browse_prefs.v1";
const DEV_SURFACES_QUERY_KEY = "devSurfaces";
// lesson canon runtime(./runtime/*)에서 동적 import하므로 경로 기준은 runtime 디렉터리다.
const WASM_CANON_RUNTIME_URL = "../wasm/ddonirang_tool.js";
const PLATFORM_UI_ACTION_EVENT = "seamgrim:platform-ui-action";
const PLATFORM_REVIEW_ACTION_EVENT = "seamgrim:platform-review-action";

function isLocalDevHost() {
  const host = String(globalThis?.location?.hostname ?? "").trim().toLowerCase();
  return !host || host === "localhost" || host === "127.0.0.1" || host === "::1";
}

function shouldEnableDevSurfaces() {
  if (!isLocalDevHost()) return false;
  try {
    const url = new URL(String(globalThis?.location?.href ?? ""));
    const queryValue = String(url.searchParams.get(DEV_SURFACES_QUERY_KEY) ?? "").trim().toLowerCase();
    if (queryValue === "1" || queryValue === "true" || queryValue === "yes") return true;
    if (queryValue === "0" || queryValue === "false" || queryValue === "no") return false;
  } catch (_) {
    // fall through to other local dev signals
  }
  return globalThis?.SEAMGRIM_DEV_SURFACES === true;
}
const DEV_SURFACES_ENABLED = shouldEnableDevSurfaces();
try {
  document?.body?.classList?.toggle("dev-surfaces-enabled", DEV_SURFACES_ENABLED);
} catch (_) {
  // Body can be unavailable in narrow test imports.
}
const MAIN_TAB_BROWSE = "browse";
const MAIN_TAB_STUDIO = "studio";
const MAIN_TAB_PUBLISH = "publish";
const STUDIO_DRAFT_LESSON_ID = "studio_draft";
const STUDIO_DRAFT_DDN_TEMPLATE = `설정 {
  제목: 새_교과.
  설명: "채비를 조절하면서 결과를 확인하세요.".
}.

채비 {
  계수:수 <- 1.
  프레임수:수 <- 0.
  t:수 <- 0.
  y:수 <- 0.
}.

(매마디)마다 {
  t <- 프레임수.
  y <- (계수 * t).
  t 보여주기.
  y 보여주기.
  프레임수 <- (프레임수 + 1).
}.`;

const appState = {
  catalogMode: CATALOG_MODE_REPS_ONLY,
  currentLesson: null,
  currentScreen: "browse",
  shell: {
    activeSpace: "learn",
    authSession: null,
    currentWorkId: null,
    currentProjectId: null,
    currentRevisionId: null,
    currentPublicationId: null,
    shareMode: null,
    activeCatalog: CatalogKind.LESSON,
    reviewStatus: "pending",
  },
  wasm: {
    enabled: true,
    loader: null,
    client: null,
    parseWarnings: [],
    fpsLimit: 30,
    dtMax: 0.1,
    langMode: "strict",
  },
  studio: {
    sourceKind: "scratch",
    sourceLabel: "새 작업",
    engineStatus: "idle",
    primaryViewFamily: "sim",
    activeSubpanelTab: "graph",
    localSaveStatus: "저장 대기",
  },
  lessonsById: new Map(),
  lessonLibraryCuration: null,
  numericTrackIndex: null,
  numericTrackReportExport: null,
  numericResultReportStage: null,
  productizationStageClosure: null,
  postSuperLongRebase: null,
  publicReleaseApprovalRecheck: null,
  localReleaseRehearsalCheck: null,
  publicationArtifactDryRun: null,
  teacherFeedbackLoopSeed: null,
  classroomOperationsTriage: null,
  benchmarkBaselinePrepDryRun: null,
  nextRoadmapV2CoordinateLock: null,
  ma3NextDevelopmentQueueRebase: null,
  teacherFeedbackSurfacePreview: null,
  classroomOperationsPanelPreview: null,
  screenListeners: new Set(),
  quickLaunch: {
    featuredSeedCursor: -1,
  },
  overlaySession: {
    runs: [],
    compare: {
      enabled: false,
      baselineId: null,
      variantId: null,
    },
    viewCombo: {
      enabled: false,
      layout: "horizontal",
      overlayOrder: "graph",
    },
  },
  inputRegistry: createInputRegistryState(),
  sceneSummary: null,
  runtimeSnapshotV0: null,
  runtimeSessionV0: null,
};

const lessonCanonHydrator = createLessonCanonHydrator({
  wasmUrl: WASM_CANON_RUNTIME_URL,
  cacheBust: 0,
});
let editorCanonSummaryTicket = 0;

function normalizeBrowsePresetId(raw) {
  const presetId = String(raw ?? "").trim().toLowerCase();
  if (!presetId) return "";
  if (presetId === "featured_seed_quick_recent") return "featured_seed_quick_recent";
  return "";
}

function readBrowsePresetFromLocation() {
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (!href) return "";
    const url = new URL(href);
    return normalizeBrowsePresetId(url.searchParams.get(BROWSE_PRESET_QUERY_KEY));
  } catch (_) {
    return "";
  }
}

function normalizeStringList(value) {
  return Array.isArray(value) ? value.map((item) => String(item ?? "").trim()).filter(Boolean) : [];
}

function buildDefaultLessonPlanTextForLesson(lesson) {
  const source = lesson && typeof lesson === "object" ? lesson : {};
  const card = source.curriculumCard && typeof source.curriculumCard === "object" ? source.curriculumCard : {};
  const lessonId = String(source.id ?? card.lesson_id ?? "custom").trim() || "custom";
  const title = String(source.title ?? card.display?.title ?? lessonId).trim() || lessonId;
  const allowedControls = normalizeStringList(card.allowed_controls);
  const observations = normalizeStringList(card.allowed_observations);
  const primaryView = String(card.primary_view ?? source.primaryViewContract?.primary_view ?? "").trim();
  const secondaryViews = normalizeStringList(card.secondary_views);
  const predictionQuestion = String(card.prediction_prompt ?? "").trim() || "실행하기 전에 결과를 예상하세요.";
  const inquiryQuestion = String(card.inquiry_prompt ?? "").trim() || "조절값 하나를 바꾸고 관찰값 변화를 비교하세요.";
  const explanationQuestion = observations.length
    ? `${observations.slice(0, 3).join(", ")} 관찰값을 근거로 결과를 설명하세요.`
    : "실행 근거를 들어 결과를 설명하세요.";
  const firstControl = allowedControls[0] ?? "";
  const firstObservation = observations[0] ?? "";
  return serializeLessonPlanToml({
    lesson_id: lessonId,
    title: `${title} 수업 구성`,
    lesson_question: inquiryQuestion,
    prediction_question: predictionQuestion,
    explanation_question: explanationQuestion,
    misconception_refs: normalizeStringList(card.misconception_refs),
    reading_check: firstObservation ? `${firstObservation} 관찰값의 단위와 의미를 한 문장으로 쓰세요.` : "관찰값 하나의 의미를 한 문장으로 쓰세요.",
    rubric: "예상 1점, 조절 실험 1점, 관찰 근거 1점, 설명 1점",
    deployment_profile_lock: "locked",
    allowed_controls: allowedControls,
    control_ranges: allowedControls.map((control) => `${control}:card`),
    primary_view: primaryView,
    secondary_views: secondaryViews,
    activities: [
      {
        activity_id: "observe-1",
        kind: "observe",
        title: "질문 확인",
        prompt: predictionQuestion,
        scene_ref: "scene.main",
        primary_view: primaryView,
        allowed_controls: [],
        question: predictionQuestion,
        view_plan_ref: "view_plan.v1",
        anchor_refs: observations.slice(0, 3).map((name) => `obs.${name}`),
      },
      {
        activity_id: "change-1",
        kind: "change",
        title: "하나만 바꾸기",
        prompt: firstControl ? `${firstControl} 하나만 바꾸고 다시 실행하세요.` : "조절값 하나만 바꾸고 다시 실행하세요.",
        scene_ref: "scene.main",
        primary_view: primaryView,
        allowed_controls: firstControl ? [firstControl] : allowedControls.slice(0, 1),
        question: inquiryQuestion,
        view_plan_ref: "view_plan.v1",
      },
      {
        activity_id: "compare-1",
        kind: "compare",
        title: "근거 비교",
        prompt: observations.length ? `${observations.slice(0, 3).join(", ")} 변화와 주 보기를 비교하세요.` : "주 보기와 관찰값 변화를 비교하세요.",
        scene_ref: "scene.main",
        primary_view: primaryView,
        allowed_controls: allowedControls.slice(0, 2),
        anchor_refs: observations.slice(0, 4).map((name) => `obs.${name}`),
        view_plan_ref: "view_plan.v1",
      },
      {
        activity_id: "explain-1",
        kind: "explain",
        title: "설명 쓰기",
        prompt: explanationQuestion,
        scene_ref: "scene.main",
        primary_view: primaryView,
        question: explanationQuestion,
        check_ref: "rubric.local_teacher_review",
      },
    ],
  });
}

function syncBrowsePresetToLocation(presetId = "") {
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (!href || !window?.history?.replaceState) return;
    const url = new URL(href);
    const normalized = normalizeBrowsePresetId(presetId);
    if (normalized) {
      url.searchParams.set(BROWSE_PRESET_QUERY_KEY, normalized);
    } else {
      url.searchParams.delete(BROWSE_PRESET_QUERY_KEY);
    }
    const next = `${url.pathname}${url.search}${url.hash}`;
    const current = `${window.location.pathname}${window.location.search}${window.location.hash}`;
    if (next === current) return;
    window.history.replaceState(null, "", next);
  } catch (_) {
    // ignore url sync errors
  }
}

function byId(id) {
  return document.getElementById(id);
}

function setSeulgiProductPayload(dataKey, textKey, formatter, row) {
  try {
    window[dataKey] = row;
    window[textKey] = row && formatter ? formatter(row) : "";
  } catch (_) {
    // ignore browser instrumentation errors
  }
}

function mountSeulgiProductSurfaces() {
  const proposal = buildSeulgiProposalUi({ rows: DEFAULT_SEULGI_PROPOSAL_UI_ROWS });
  setSeulgiProductPayload(
    "__SEULGI_PROPOSAL_UI__",
    "__SEULGI_PROPOSAL_UI_TEXT__",
    formatSeulgiProposalUiText,
    proposal,
  );
  renderSeulgiProposalUi(byId("seulgi-proposal-ui"), proposal);

  const replayWorkflow = buildSeulgiReplaySafeWorkflow({ rows: DEFAULT_SEULGI_REPLAY_SAFE_ROWS });
  setSeulgiProductPayload(
    "__SEULGI_REPLAY_SAFE_WORKFLOW__",
    "__SEULGI_REPLAY_SAFE_WORKFLOW_TEXT__",
    formatSeulgiReplaySafeWorkflowText,
    replayWorkflow,
  );
  renderSeulgiReplaySafeWorkflow(byId("seulgi-replay-safe-workflow"), replayWorkflow);
}

function readWindowStringArray(key, fallback = []) {
  try {
    const value = window?.[key];
    if (!Array.isArray(value)) return fallback;
    const out = [];
    const seen = new Set();
    value.forEach((item) => {
      const row = String(item ?? "").trim();
      if (!row || seen.has(row)) return;
      seen.add(row);
      out.push(row);
    });
    return out;
  } catch (_) {
    return fallback;
  }
}

function readWindowBoolean(key, fallback = false) {
  try {
    const value = window?.[key];
    if (typeof value === "boolean") return value;
    const text = String(value ?? "").trim().toLowerCase();
    if (!text) return fallback;
    if (text === "1" || text === "true" || text === "yes" || text === "on") return true;
    if (text === "0" || text === "false" || text === "no" || text === "off") return false;
    return fallback;
  } catch (_) {
    return fallback;
  }
}

function readQueryBoolean(key, fallback = false) {
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (!href) return fallback;
    const url = new URL(href);
    const raw = String(url.searchParams.get(key) ?? "").trim().toLowerCase();
    if (!raw) return fallback;
    if (raw === "1" || raw === "true" || raw === "yes" || raw === "on") return true;
    if (raw === "0" || raw === "false" || raw === "no" || raw === "off") return false;
    return fallback;
  } catch (_) {
    return fallback;
  }
}

function normalizeMainTabTarget(raw, fallback = MAIN_TAB_BROWSE) {
  const value = String(raw ?? "").trim().toLowerCase();
  if (value === MAIN_TAB_STUDIO) return MAIN_TAB_STUDIO;
  if (value === MAIN_TAB_PUBLISH) return MAIN_TAB_PUBLISH;
  if (value === MAIN_TAB_BROWSE || value === "lesson") return MAIN_TAB_BROWSE;
  return fallback;
}

function decodeBase64Utf8(raw) {
  const text = String(raw ?? "").trim();
  if (!text) return "";
  try {
    const normalized = text.replace(/-/g, "+").replace(/_/g, "/");
    const padLen = normalized.length % 4;
    const padded = padLen === 0 ? normalized : normalized + "=".repeat(4 - padLen);
    const bytes = Uint8Array.from(atob(padded), (char) => char.charCodeAt(0));
    return new TextDecoder("utf-8", { fatal: false }).decode(bytes);
  } catch (_) {
    return "";
  }
}

function normalizeLessonIdFromRoute(raw) {
  const value = String(raw ?? "").trim();
  if (!value) return "";
  if (/^https?:\/\//i.test(value)) return "";
  const direct = value.replace(/^\/+/, "").replace(/\/+$/, "");
  if (!direct) return "";
  if (direct.includes("/")) {
    const matched = direct.match(/^lessons\/([^/]+)\/lesson\.ddn$/i);
    if (matched && matched[1]) return String(matched[1]).trim();
  }
  const aliasHit = FIRST_DELIGHT_LESSON_BY_ALIAS[direct.toLowerCase()];
  return aliasHit || direct;
}

function readFirstDelightFromLocation() {
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (!href) return "";
    const url = new URL(href);
    const requested = String(url.searchParams.get(FIRST_DELIGHT_QUERY_KEY) ?? "").trim().toLowerCase();
    return FIRST_DELIGHT_LESSON_BY_ALIAS[requested] || "";
  } catch (_) {
    // ignore invalid route input
  }
  return "";
}

function readStudioRouteRequestFromLocation() {
  const fallback = {
    tab: MAIN_TAB_BROWSE,
    lessonId: "",
    ddnText: "",
    hasStudioRequest: false,
    fromLegacyPlayRedirect: false,
    hasLessonOrDdnRequest: false,
  };
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (!href) return fallback;
    const url = new URL(href);
    const tab = normalizeMainTabTarget(url.searchParams.get("tab"), MAIN_TAB_BROWSE);
    const lessonId = normalizeLessonIdFromRoute(url.searchParams.get("lesson"));
    const ddnText = decodeBase64Utf8(url.searchParams.get("ddn"));
    const notice = String(url.searchParams.get("_notice") ?? "").trim().toLowerCase();
    const fromLegacyPlayRedirect =
      readQueryBoolean("legacy_play_redirect", false) || notice === "play_unified";
    const hasLessonOrDdnRequest = Boolean(lessonId) || Boolean(ddnText);
    return {
      tab,
      lessonId,
      ddnText,
      hasStudioRequest: tab === MAIN_TAB_STUDIO || hasLessonOrDdnRequest,
      fromLegacyPlayRedirect,
      hasLessonOrDdnRequest,
    };
  } catch (_) {
    return fallback;
  }
}

function syncMainTabRoute({ tab = MAIN_TAB_BROWSE, lessonId = "" } = {}) {
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (!href || !window?.history?.replaceState) return false;
    const url = new URL(href);
    const normalizedTab = normalizeMainTabTarget(tab, MAIN_TAB_BROWSE);
    if (normalizedTab === MAIN_TAB_STUDIO) {
      url.searchParams.set("tab", MAIN_TAB_STUDIO);
      const normalizedLessonId = normalizeLessonIdFromRoute(lessonId);
      if (normalizedLessonId) {
        url.searchParams.set("lesson", normalizedLessonId);
      } else {
        url.searchParams.delete("lesson");
      }
    } else if (normalizedTab === MAIN_TAB_PUBLISH) {
      url.searchParams.set("tab", MAIN_TAB_PUBLISH);
      url.searchParams.delete("lesson");
      url.searchParams.delete("ddn");
    } else {
      url.searchParams.set("tab", MAIN_TAB_BROWSE);
      url.searchParams.delete("lesson");
      url.searchParams.delete("ddn");
    }
    const next = `${url.pathname}${url.search}${url.hash}`;
    const current = `${window.location.pathname}${window.location.search}${window.location.hash}`;
    if (next === current) return false;
    window.history.replaceState(null, "", next);
    return true;
  } catch (_) {
    return false;
  }
}

function consumeLegacyPlayRedirectParam() {
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (!href || !window?.history?.replaceState) return false;
    const url = new URL(href);
    const hasLegacy = url.searchParams.has("legacy_play_redirect");
    const hasNotice = url.searchParams.has("_notice");
    if (!hasLegacy && !hasNotice) return false;
    url.searchParams.delete("legacy_play_redirect");
    url.searchParams.delete("_notice");
    window.history.replaceState(null, "", `${url.pathname}${url.search}${url.hash}`);
    return true;
  } catch (_) {
    return false;
  }
}

function resolvePlatformMockMenuEnabled() {
  return (
    readWindowBoolean("SEAMGRIM_ENABLE_PLATFORM_MOCK_MENU", false) ||
    readQueryBoolean("platform_mock_menu", false) ||
    readQueryBoolean("platform_mock", false)
  );
}

function resolvePlatformServerAdapterEnabled() {
  return (
    readWindowBoolean("SEAMGRIM_ENABLE_PLATFORM_SERVER_ADAPTER", false) ||
    readQueryBoolean("platform_server_adapter", false)
  );
}

function applyPlatformMockMenuState(buttons = [], enabled = false) {
  const active = Boolean(enabled);
  const rows = Array.isArray(buttons) ? buttons : [];
  rows.forEach((button) => {
    if (!button || typeof button !== "object") return;
    try {
      button.disabled = !active;
    } catch (_) {
      // ignore disabled toggle errors
    }
    try {
      button.title = active ? "플랫폼 mock 표면 활성화" : "준비 중";
    } catch (_) {
      // ignore title update errors
    }
    try {
      button.setAttribute?.("aria-disabled", active ? "false" : "true");
    } catch (_) {
      // ignore aria-disabled errors
    }
  });
  return active;
}

function emitPlatformServerAdapterExchange(request = null, response = null) {
  if (typeof window === "undefined") return;
  try {
    window.__SEAMGRIM_PLATFORM_SERVER_LAST_REQUEST__ = request && typeof request === "object" ? request : null;
  } catch (_) {
    // ignore assignment errors
  }
  try {
    window.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__ = response && typeof response === "object" ? response : null;
  } catch (_) {
    // ignore assignment errors
  }
  const errorCode = String(response?.error?.code ?? "").trim();
  const actionRail = resolveServerErrorActionRail(errorCode);
  try {
    window.__SEAMGRIM_PLATFORM_SERVER_LAST_ACTION_RAIL__ = Array.isArray(actionRail) ? actionRail.slice() : [];
  } catch (_) {
    // ignore assignment errors
  }
  try {
    if (typeof window?.dispatchEvent === "function" && typeof CustomEvent === "function") {
      window.dispatchEvent(
        new CustomEvent("seamgrim:platform-server-adapter-exchange", {
          detail: {
            request: request && typeof request === "object" ? request : null,
            response: response && typeof response === "object" ? response : null,
            action_rail: Array.isArray(actionRail) ? actionRail.slice() : [],
          },
        }),
      );
    }
  } catch (_) {
    // ignore dispatch errors
  }
}

function maybeEmitPlatformServerAdapterForOp(op, payload = {}, fallbackMessage = "플랫폼 server adapter 준비 중입니다.") {
  if (!resolvePlatformServerAdapterEnabled()) return null;
  const request = buildServerAdapterRequest({
    op: String(op ?? "").trim(),
    requestId: `mock-${Date.now()}`,
    authSessionId: appState.shell?.authSession?.id ?? null,
    userId: appState.shell?.authSession?.userId ?? null,
    context: {
      work_id: appState.shell.currentWorkId,
      project_id: appState.shell.currentProjectId,
      revision_id: appState.shell.currentRevisionId,
      publication_id: appState.shell.currentPublicationId,
      active_catalog: appState.shell.activeCatalog,
      lesson_id: appState.currentLesson?.id ?? null,
      share_mode: appState.shell.shareMode,
    },
    payload: payload && typeof payload === "object" ? payload : {},
  });
  const response = buildServerAdapterErrorResponse({
    request,
    code: mapServerOpToNotReadyErrorCode(op),
    message: String(fallbackMessage ?? "").trim() || "플랫폼 server adapter 준비 중입니다.",
    retryable: false,
  });
  emitPlatformServerAdapterExchange(request, response);
  return response;
}

function normalizeCatalogMode(raw, fallback = CATALOG_MODE_REPS_ONLY) {
  const mode = String(raw ?? "").trim().toLowerCase();
  if (mode === CATALOG_MODE_FULL) return CATALOG_MODE_FULL;
  if (mode === CATALOG_MODE_REPS_ONLY) return CATALOG_MODE_REPS_ONLY;
  return fallback;
}

function normalizeRunOnboardingProfile(raw) {
  const profile = String(raw ?? "").trim().toLowerCase();
  if (profile === "student") return "student";
  if (profile === "teacher") return "teacher";
  return "";
}

function readBrowseLaunchProfile() {
  try {
    const raw = window?.localStorage?.getItem(BROWSE_UI_PREFS_STORAGE_KEY);
    if (!raw) return "";
    const parsed = JSON.parse(raw);
    return normalizeRunOnboardingProfile(parsed?.launchProfile ?? "");
  } catch (_) {
    return "";
  }
}

function resolveCatalogMode() {
  const windowMode = normalizeCatalogMode(window?.SEAMGRIM_LESSON_CATALOG_MODE, CATALOG_MODE_REPS_ONLY);
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (href) {
      const url = new URL(href);
      const queryMode = normalizeCatalogMode(url.searchParams.get("catalog_mode"), "");
      if (queryMode === CATALOG_MODE_FULL || queryMode === CATALOG_MODE_REPS_ONLY) {
        return queryMode;
      }
    }
  } catch (_) {
    // ignore query parse errors
  }
  return windowMode;
}

function applySimCorePolicy() {
  const enabled = readWindowBoolean("SEAMGRIM_SIM_CORE_POLICY", true);
  try {
    document?.body?.classList?.toggle(SIM_CORE_POLICY_CLASS, enabled);
  } catch (_) {
    // ignore class toggle errors
  }
  return enabled;
}

function buildOverlaySessionSnapshot() {
  const overlay = appState.overlaySession ?? {};
  const runs = Array.isArray(overlay.runs) ? overlay.runs : [];
  const resolvedCompare = resolveOverlayCompareFromSession({
    runs,
    compare: overlay.compare ?? {},
  });
  return {
    runs: buildOverlaySessionRunsPayload(runs),
    compare: buildOverlayCompareSessionPayload(resolvedCompare),
    view_combo: buildSessionViewComboPayload({
      enabled: Boolean(overlay.viewCombo?.enabled),
      layout: String(overlay.viewCombo?.layout ?? "horizontal"),
      overlayOrder: String(overlay.viewCombo?.overlayOrder ?? "graph"),
    }),
  };
}

function restoreOverlaySessionSnapshot() {
  try {
    const raw = window?.localStorage?.getItem(OVERLAY_SESSION_STORAGE_KEY);
    if (!raw) return;
    const payload = JSON.parse(raw);
    const snapshot = payload?.overlay_session && typeof payload.overlay_session === "object" ? payload.overlay_session : {};
    const runs = Array.isArray(snapshot.runs) ? snapshot.runs : [];
    const compare = resolveOverlayCompareFromSession({
      runs,
      compare: snapshot.compare ?? {},
    });
    const viewCombo = resolveSessionViewComboFromPayload(snapshot.view_combo ?? {});
    appState.overlaySession = {
      runs,
      compare,
      viewCombo,
    };
  } catch (_) {
    // ignore restore errors
  }
}

function persistOverlaySessionSnapshot() {
  try {
    const payload = {
      schema: "seamgrim.overlay_session.v1",
      overlay_session: buildOverlaySessionSnapshot(),
    };
    window?.localStorage?.setItem(OVERLAY_SESSION_STORAGE_KEY, JSON.stringify(payload));
  } catch (_) {
    // ignore save errors
  }
}

function restoreInputRegistrySnapshot() {
  try {
    const raw = window?.localStorage?.getItem(INPUT_REGISTRY_STORAGE_KEY);
    if (!raw) {
      appState.inputRegistry = createInputRegistryState();
      return;
    }
    const payload = JSON.parse(raw);
    appState.inputRegistry = restoreInputRegistrySession(payload);
  } catch (_) {
    appState.inputRegistry = createInputRegistryState();
  }
}

function persistInputRegistrySnapshot() {
  try {
    const payload = serializeInputRegistrySession(appState.inputRegistry);
    window?.localStorage?.setItem(INPUT_REGISTRY_STORAGE_KEY, JSON.stringify(payload));
  } catch (_) {
    // ignore save errors
  }
}

function setSceneSummarySnapshot(summary) {
  appState.sceneSummary = summary && typeof summary === "object" ? summary : null;
  if (typeof window !== "undefined") {
    window.__SEAMGRIM_SCENE_SUMMARY__ = appState.sceneSummary;
  }
}

function restoreSceneSummarySnapshot() {
  try {
    const raw = window?.localStorage?.getItem(SCENE_SUMMARY_STORAGE_KEY);
    if (!raw) {
      setSceneSummarySnapshot(null);
      return;
    }
    const payload = JSON.parse(raw);
    const restored = restoreSceneSummarySession(payload);
    setSceneSummarySnapshot(restored);
  } catch (_) {
    setSceneSummarySnapshot(null);
  }
}

function buildSceneSummaryForApp(runScreen = null) {
  const lesson = runScreen?.lesson ?? appState.currentLesson;
  const hasLesson = Boolean(lesson && (lesson.id || lesson.title));
  const hasRuntime = Boolean(runScreen?.lastRuntimeDerived);
  const hasInputSelection = Boolean(appState.inputRegistry?.selectedId);
  if (!hasLesson && !hasRuntime && !hasInputSelection) {
    return null;
  }
  return buildSceneSummarySnapshot({
    timestamp: new Date().toISOString(),
    lessonId: String(lesson?.id ?? ""),
    lessonTitle: String(lesson?.title ?? ""),
    requiredViews: lesson?.requiredViews ?? [],
    inputRegistryState: appState.inputRegistry,
    overlayRuns: appState.overlaySession?.runs ?? [],
    runtimeDerived: runScreen?.lastRuntimeDerived ?? null,
    runtimeHash: runScreen?.lastRuntimeHash ?? "",
    primaryView: String(
      lesson?.primaryViewContract?.primary_view
        ?? lesson?.curriculumCard?.primary_view
        ?? "",
    ).trim(),
    canonicalViewPayloads: runScreen?.lastRuntimeDerived?.canonicalViewPayloads ?? null,
    bogaeScene: null,
  });
}

function persistSceneSummarySnapshot(runScreen = null) {
  try {
    const scene = buildSceneSummaryForApp(runScreen);
    if (!scene) return;
    setSceneSummarySnapshot(scene);
    const payload = serializeSceneSummarySession(scene);
    window?.localStorage?.setItem(SCENE_SUMMARY_STORAGE_KEY, JSON.stringify(payload));
  } catch (_) {
    // ignore save errors
  }
}

function setRuntimeSnapshotBundleV0({ snapshot = null, session = null } = {}) {
  appState.runtimeSnapshotV0 = snapshot && typeof snapshot === "object" ? snapshot : null;
  appState.runtimeSessionV0 = session && typeof session === "object" ? session : null;
  if (typeof window !== "undefined") {
    window.__SEAMGRIM_SNAPSHOT_V0__ = appState.runtimeSnapshotV0;
    window.__SEAMGRIM_SESSION_V0__ = appState.runtimeSessionV0;
  }
  updateShellStatusRail();
}

function restoreRuntimeSnapshotBundleV0() {
  try {
    const rawSnapshot = window?.localStorage?.getItem(SNAPSHOT_V0_STORAGE_KEY);
    const rawSession = window?.localStorage?.getItem(SESSION_V0_STORAGE_KEY);
    const snapshot = rawSnapshot ? JSON.parse(rawSnapshot) : null;
    const session = rawSession ? JSON.parse(rawSession) : null;
    setRuntimeSnapshotBundleV0({ snapshot, session });
  } catch (_) {
    setRuntimeSnapshotBundleV0({ snapshot: null, session: null });
  }
}

function toFiniteNumber(raw) {
  const num = Number(raw);
  return Number.isFinite(num) ? num : null;
}

function normalizeGraphSample(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  const varName = String(row.var ?? row.variable ?? "").trim();
  const xMin = toFiniteNumber(row.x_min ?? row.xMin);
  const xMax = toFiniteNumber(row.x_max ?? row.xMax);
  const step = toFiniteNumber(row.step);
  if (!varName || xMin === null || xMax === null || step === null || xMax < xMin || step <= 0) {
    return null;
  }
  const tick = toFiniteNumber(row.tick);
  return {
    var: varName,
    x_min: xMin,
    x_max: xMax,
    step,
    ...(tick === null ? {} : { tick }),
  };
}

function normalizeGraphView(raw, { auto = false, panX = 0, panY = 0, zoom = 1 } = {}) {
  const row = raw && typeof raw === "object" ? raw : {};
  const xMin = toFiniteNumber(row.x_min ?? row.xMin);
  const xMax = toFiniteNumber(row.x_max ?? row.xMax);
  const yMin = toFiniteNumber(row.y_min ?? row.yMin);
  const yMax = toFiniteNumber(row.y_max ?? row.yMax);
  if ([xMin, xMax, yMin, yMax].some((value) => value === null) || xMax <= xMin || yMax <= yMin) {
    return null;
  }
  return {
    auto: Boolean(auto),
    x_min: xMin,
    x_max: xMax,
    y_min: yMin,
    y_max: yMax,
    pan_x: toFiniteNumber(panX) ?? 0,
    pan_y: toFiniteNumber(panY) ?? 0,
    zoom: toFiniteNumber(zoom) ?? 1,
  };
}

function normalizeRuntimeGraphForPersistence(runtimeGraph = null, runtimeSessionState = null) {
  const graph = runtimeGraph && typeof runtimeGraph === "object" ? JSON.parse(JSON.stringify(runtimeGraph)) : null;
  if (!graph) return null;
  const state = runtimeSessionState && typeof runtimeSessionState === "object" ? runtimeSessionState : {};
  const stateView = state?.view && typeof state.view === "object" ? state.view : {};
  const graphView = stateView?.graph && typeof stateView.graph === "object" ? stateView.graph : {};
  const sample = normalizeGraphSample(graph.sample) ?? normalizeGraphSample(state.sample);
  const view = normalizeGraphView(
    graph.view ?? graphView.axis ?? graphView.range ?? null,
    {
      auto: Boolean(graphView.auto_fit ?? graphView.autoFit ?? false),
      panX: stateView.panX ?? stateView.pan_x ?? 0,
      panY: stateView.panY ?? stateView.pan_y ?? 0,
      zoom: stateView.zoom ?? 1,
    },
  );
  if (sample) graph.sample = sample;
  if (view) graph.view = view;
  return graph;
}

function buildRuntimeSnapshotBundleForApp(runScreen = null) {
  const lesson = runScreen?.lesson ?? appState.currentLesson;
  const runtimeGraphRaw = runScreen?.lastRuntimeDerived?.views?.graph ?? null;
  const runtimeSessionState =
    runScreen && typeof runScreen.exportRuntimeSessionState === "function"
      ? runScreen.exportRuntimeSessionState()
      : {};
  const runtimeGraph = normalizeRuntimeGraphForPersistence(runtimeGraphRaw, runtimeSessionState);
  const runtimeTime = runtimeSessionState?.time && typeof runtimeSessionState.time === "object"
    ? runtimeSessionState.time
    : {};
  const activeRunId = String(runtimeSessionState?.active_run_id ?? "").trim();
  const cursorTick = Math.max(0, Number(runtimeTime.tick) || Number(runScreen?.runtimeTickCounter) || 0);
  const cursorTime = Number.isFinite(Number(runtimeTime.now)) ? Number(runtimeTime.now) : 0;
  const hasLesson = Boolean(lesson && (lesson.id || lesson.title));
  const hasGraph = Boolean(runtimeGraph && typeof runtimeGraph === "object");
  const hasOverlayRuns = Array.isArray(appState.overlaySession?.runs) && appState.overlaySession.runs.length > 0;
  if (!hasLesson && !hasGraph && !hasOverlayRuns) {
    return null;
  }

  return buildRuntimeSnapshotBundleV0({
    timestamp: new Date().toISOString(),
    lessonId: String(lesson?.id ?? ""),
    lessonTitle: String(lesson?.title ?? ""),
    ddnText: String(lesson?.ddnText ?? ""),
    requiredViews: lesson?.requiredViews ?? [],
    inputRegistryState: appState.inputRegistry,
    overlayRuns: appState.overlaySession?.runs ?? [],
    runtimeGraph,
    runtimeHash: runScreen?.lastRuntimeHash ?? "",
    layoutPreset: "auto",
    cursor: {
      id: activeRunId || String(lesson?.id ?? ""),
      t: cursorTime,
      tick: cursorTick,
    },
    graphRef: "",
    sceneRef: "",
    runtimeSessionState,
    overlayCompare: appState.overlaySession?.compare ?? {},
    overlayViewCombo: appState.overlaySession?.viewCombo ?? {},
    activeRunId,
  });
}

function persistRuntimeSnapshotV0(runScreen = null, bundle = null) {
  try {
    const resolvedBundle = bundle ?? buildRuntimeSnapshotBundleForApp(runScreen);
    const snapshot = resolvedBundle?.snapshot ?? null;
    if (!snapshot) return false;
    setRuntimeSnapshotBundleV0({
      snapshot,
      session: appState.runtimeSessionV0 ?? null,
    });
    window?.localStorage?.setItem(SNAPSHOT_V0_STORAGE_KEY, JSON.stringify(snapshot));
    return true;
  } catch (_) {
    return false;
  }
}

function persistRuntimeSessionV0(runScreen = null, bundle = null) {
  try {
    const resolvedBundle = bundle ?? buildRuntimeSnapshotBundleForApp(runScreen);
    const session = resolvedBundle?.session ?? null;
    if (!session) return false;
    setRuntimeSnapshotBundleV0({
      snapshot: appState.runtimeSnapshotV0 ?? null,
      session,
    });
    window?.localStorage?.setItem(SESSION_V0_STORAGE_KEY, JSON.stringify(session));
    return true;
  } catch (_) {
    return false;
  }
}

function persistRuntimeSnapshotBundleV0(runScreen = null) {
  try {
    const bundle = buildRuntimeSnapshotBundleForApp(runScreen);
    if (!bundle) return false;
    const snapshotSaved = persistRuntimeSnapshotV0(runScreen, bundle);
    const sessionSaved = persistRuntimeSessionV0(runScreen, bundle);
    return snapshotSaved || sessionSaved;
  } catch (_) {
    return false;
  }
}

function trackLessonInputSource(lesson, { sourceId = "" } = {}) {
  const lessonId = String(lesson?.id ?? "").trim();
  if (!lessonId) return;
  appState.inputRegistry = registerLessonInput(appState.inputRegistry, {
    id: String(sourceId ?? "").trim() || `lesson:${lessonId}`,
    lessonId,
    label: String(lesson?.title ?? lessonId).trim() || lessonId,
    requiredViews: lesson?.requiredViews ?? [],
    ddnText: lesson?.ddnText ?? "",
  });
}

function trackDdnInputSource({
  sourceId = "",
  label = "",
  ddnText = "",
  derivedFrom = "",
} = {}) {
  appState.inputRegistry = registerDdnInput(appState.inputRegistry, {
    id: String(sourceId ?? "").trim() || "ddn:custom",
    label: String(label ?? "").trim() || "사용자 DDN",
    ddnText: String(ddnText ?? ""),
    derivedFrom: String(derivedFrom ?? "").trim(),
  });
}

function resolveShellScreenLabel(screenName = appState.currentScreen) {
  const screen = String(screenName ?? "").trim();
  if (screen === "browse") return "교과";
  if (screen === "editor") return "편집";
  if (screen === "run") return appState.shell.activeSpace === "learn" ? "수업" : "실험실";
  return "교과";
}

function resolveShellSessionStatus() {
  if (appState.runtimeSessionV0) return "세션 복원됨";
  if (appState.runtimeSnapshotV0) return "결과 복원됨";
  return "세션 대기";
}

function buildShellStatusModel() {
  const sourceLabel = String(appState.studio?.sourceLabel ?? "").trim() || "새 작업";
  const saveStatus = String(appState.studio?.localSaveStatus ?? "").trim() || "저장 대기";
  const sessionStatus = resolveShellSessionStatus();
  return {
    screenLabel: resolveShellScreenLabel(),
    sourceLabel,
    saveStatus,
    sessionStatus,
  };
}

function setShellChipText(rail, selector, text, status = "") {
  const node = rail?.querySelector?.(selector);
  if (!node) return;
  const value = String(text ?? "").trim();
  node.textContent = value;
  node.title = value;
  if (status) {
    node.dataset.status = status;
  } else {
    delete node.dataset.status;
  }
}

function updateShellStatusRail() {
  if (typeof document === "undefined") return;
  const rails = document.querySelectorAll?.("[data-shell-status-rail]") ?? [];
  if (!rails.length) return;
  const status = buildShellStatusModel();
  rails.forEach((rail) => {
    setShellChipText(rail, "[data-shell-current-screen]", status.screenLabel, "active");
    setShellChipText(rail, "[data-shell-source-label]", status.sourceLabel);
    setShellChipText(
      rail,
      "[data-shell-save-status]",
      status.saveStatus,
      status.saveStatus === "저장 대기" ? "" : "saved",
    );
    setShellChipText(
      rail,
      "[data-shell-session-status]",
      status.sessionStatus,
      status.sessionStatus === "세션 대기" ? "" : "restored",
    );
  });
}

function setScreen(name, { shellSpace = "" } = {}) {
  const requestedShellSpace = String(shellSpace ?? "").trim();
  if (["learn", "lab", "create", "publish"].includes(requestedShellSpace)) {
    appState.shell.activeSpace = requestedShellSpace;
  } else if (name === "browse") {
    appState.shell.activeSpace = "learn";
  } else if (name === "editor") {
    appState.shell.activeSpace = "create";
  } else if (name === "run" && !["learn", "lab"].includes(appState.shell.activeSpace)) {
    appState.shell.activeSpace = "lab";
  }
  ["browse", "publish", "editor", "run"].forEach((screenName) => {
    const node = byId(`screen-${screenName}`);
    if (!node) return;
    node.classList.toggle("hidden", screenName !== name);
  });
  appState.currentScreen = name;
  const mainTabTarget = name === "browse" ? MAIN_TAB_BROWSE : name === "publish" ? MAIN_TAB_PUBLISH : MAIN_TAB_STUDIO;
  if (mainTabTarget === MAIN_TAB_BROWSE || mainTabTarget === MAIN_TAB_PUBLISH) {
    syncMainTabRoute({ tab: mainTabTarget });
  }
  try {
    const tabButtons = document?.querySelectorAll?.(".main-shell-tab[data-main-tab-target]") ?? [];
    const activeShellSpace = name === "browse" ? appState.shell.activeSpace : name === "publish" ? "publish" : name === "editor" ? "create" : appState.shell.activeSpace;
    tabButtons.forEach((button) => {
      const target = normalizeMainTabTarget(button?.dataset?.mainTabTarget, MAIN_TAB_BROWSE);
      const shellSpace = String(button?.dataset?.shellSpace ?? "").trim();
      const active = shellSpace ? shellSpace === activeShellSpace : target === mainTabTarget;
      button?.classList?.toggle?.("active", active);
      try {
        button?.setAttribute?.("aria-pressed", active ? "true" : "false");
      } catch (_) {
        // ignore aria update errors
      }
    });
  } catch (_) {
    // ignore main tab update errors
  }
  const activeScreen = byId(`screen-${name}`);
  if (activeScreen?.dataset) {
    activeScreen.dataset.shellSpace = name === "run" ? appState.shell.activeSpace : name === "editor" ? "create" : name === "publish" ? "publish" : appState.shell.activeSpace;
  }
  appState.screenListeners.forEach((listener) => {
    try {
      listener(name);
    } catch (_) {
      // ignore screen listener errors
    }
  });
  updateShellStatusRail();
}

function onScreenChange(listener) {
  if (typeof listener !== "function") return;
  appState.screenListeners.add(listener);
}

function isEditableKeyboardTarget(target) {
  if (!(target instanceof Element)) return false;
  if (target instanceof HTMLInputElement) return true;
  if (target instanceof HTMLTextAreaElement) return true;
  if (target instanceof HTMLSelectElement) return true;
  if (target.closest("input, textarea, select, [contenteditable='true']")) return true;
  return target.isContentEditable;
}

function showPlatformToast(message) {
  const text = String(message ?? "").trim();
  if (!text) return;
  try {
    window?.dispatchEvent?.(
      new CustomEvent("seamgrim:platform-toast", {
        detail: { message: text },
      }),
    );
  } catch (_) {
    // ignore dispatch errors
  }
  console.info(`[seamgrim-platform] ${text}`);
}

function setLocalPackageImportStatus(state, message) {
  const node = byId("local-package-import-status");
  if (!node) return;
  const normalizedState = String(state ?? "").trim() || "idle";
  const text = String(message ?? "").trim();
  node.dataset.state = normalizedState;
  node.textContent = text;
  node.classList.toggle("hidden", !text);
}

function safeLocalPackageToken(value, fallback) {
  const token = String(value ?? "")
    .trim()
    .replace(/[^a-zA-Z0-9._-]+/g, "_")
    .replace(/^_+|_+$/g, "");
  return token || fallback;
}

function normalizePlatformUiAction(raw) {
  const token = String(raw ?? "").trim();
  if (
    token === PlatformServerAdapterUiAction.LOGIN ||
    token === PlatformServerAdapterUiAction.REQUEST_ACCESS ||
    token === PlatformServerAdapterUiAction.FIX_INPUT ||
    token === PlatformServerAdapterUiAction.RETRY_LATER ||
    token === PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE
  ) {
    return token;
  }
  return "";
}

function resolvePlatformUiActionDdnText(detail = {}, resolveDdnText = null) {
  const direct = String(detail?.ddnText ?? "");
  if (direct.trim()) return direct;
  if (typeof resolveDdnText === "function") {
    const resolved = String(resolveDdnText() ?? "");
    if (resolved.trim()) return resolved;
  }
  return "";
}

function handlePlatformUiActionRequest(detail = {}, { resolveDdnText = null } = {}) {
  const action = normalizePlatformUiAction(detail?.action ?? "");
  if (!action) return false;
  if (action === PlatformServerAdapterUiAction.LOGIN) {
    showPlatformToast("로그인 기능은 준비 중입니다. 우선 로컬 저장으로 계속할 수 있습니다.");
    return true;
  }
  if (action === PlatformServerAdapterUiAction.REQUEST_ACCESS) {
    showPlatformToast("권한 요청 기능은 준비 중입니다. 우선 로컬 저장으로 계속할 수 있습니다.");
    return true;
  }
  if (action === PlatformServerAdapterUiAction.FIX_INPUT) {
    showPlatformToast("입력 점검이 필요합니다. DDN/거울 탭에서 원인을 확인하세요.");
    return true;
  }
  if (action === PlatformServerAdapterUiAction.RETRY_LATER) {
    showPlatformToast("잠시 후 다시 시도하세요.");
    return true;
  }
  if (action === PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE) {
    const ddnText = resolvePlatformUiActionDdnText(detail, resolveDdnText);
    const ok = saveCurrentWork("local", { ddnText });
    showPlatformToast(ok ? "로컬 저장을 실행했습니다." : "로컬 저장에 실패했습니다.");
    return ok;
  }
  return false;
}

function emitPlatformMockAdapterPayload(payload) {
  if (!payload || typeof payload !== "object") return;
  if (typeof window === "undefined") return;
  try {
    window.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__ = payload;
  } catch (_) {
    // ignore assignment errors
  }
}

function emitPlatformReviewAction(action = "", detail = {}) {
  const normalizedAction = String(action ?? "").trim();
  if (!normalizedAction || typeof window === "undefined") return;
  const payload = {
    action: normalizedAction,
    workId: appState.shell.currentWorkId,
    revisionId: appState.shell.currentRevisionId,
    publicationId: appState.shell.currentPublicationId,
    reviewStatus: appState.shell.reviewStatus,
    ...((detail && typeof detail === "object") ? detail : {}),
  };
  try {
    window.__SEAMGRIM_PLATFORM_REVIEW_LAST_ACTION__ = payload;
  } catch (_) {
    // ignore assignment errors
  }
  try {
    if (typeof window.dispatchEvent === "function" && typeof CustomEvent === "function") {
      window.dispatchEvent(
        new CustomEvent(PLATFORM_REVIEW_ACTION_EVENT, {
          detail: payload,
        }),
      );
    }
  } catch (_) {
    // ignore dispatch errors
  }
}

function readPlatformRouteSlotsFromLocation() {
  const fallback = {
    workId: "",
    revisionId: "",
    publicationId: "",
    projectId: "",
    hasPlatformSlots: false,
    hasLegacySlots: false,
  };
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (!href) return fallback;
    const url = new URL(href);
    const routePrecedence = Array.isArray(RouteSlotPolicy?.PLATFORM_ROUTE_PRECEDENCE)
      ? RouteSlotPolicy.PLATFORM_ROUTE_PRECEDENCE
      : ["work", "revision", "publication", "project"];
    const legacyKeys = Array.isArray(RouteSlotPolicy?.LEGACY_FALLBACK_KEYS)
      ? RouteSlotPolicy.LEGACY_FALLBACK_KEYS
      : ["lesson", "ddn"];
    const values = Object.create(null);
    routePrecedence.forEach((key) => {
      values[key] = String(url.searchParams.get(key) ?? "").trim();
    });
    const legacyValues = Object.create(null);
    legacyKeys.forEach((key) => {
      legacyValues[key] = String(url.searchParams.get(key) ?? "").trim();
    });
    return {
      workId: String(values.work ?? "").trim(),
      revisionId: String(values.revision ?? "").trim(),
      publicationId: String(values.publication ?? "").trim(),
      projectId: String(values.project ?? "").trim(),
      hasPlatformSlots: routePrecedence.some((key) => Boolean(values[key])),
      hasLegacySlots: legacyKeys.some((key) => Boolean(legacyValues[key])),
    };
  } catch (_) {
    return fallback;
  }
}

function applyPlatformRouteFallback(slots = {}) {
  const route = slots && typeof slots === "object" ? slots : {};
  appState.shell.currentWorkId = String(route.workId ?? "").trim() || null;
  appState.shell.currentRevisionId = String(route.revisionId ?? "").trim() || null;
  appState.shell.currentPublicationId = String(route.publicationId ?? "").trim() || null;
  appState.shell.currentProjectId = String(route.projectId ?? "").trim() || null;
  const routeLabel = Array.isArray(RouteSlotPolicy?.PLATFORM_ROUTE_PRECEDENCE)
    ? RouteSlotPolicy.PLATFORM_ROUTE_PRECEDENCE.join("/")
    : "work/revision/publication/project";
  const targetKinds = [
    [ObjectKind.WORKSPACE, appState.shell.currentWorkId],
    [ObjectKind.REVISION, appState.shell.currentRevisionId],
    [ObjectKind.ARTIFACT, appState.shell.currentPublicationId],
    [ObjectKind.PROJECT, appState.shell.currentProjectId],
  ]
    .filter(([, value]) => Boolean(value))
    .map(([kind]) => kind);
  if (targetKinds.length > 0) {
    showPlatformToast(
      `서버 저장 기능은 준비 중입니다. (${routeLabel}, ${targetKinds.join(", ")}) 경로는 교과 탭으로 이동합니다.`,
    );
  } else {
    showPlatformToast("서버 저장 기능은 준비 중입니다. 교과 탭으로 이동합니다.");
  }
}

function shouldApplyPlatformRouteFallback(slots = {}) {
  const route = slots && typeof slots === "object" ? slots : {};
  return Boolean(route.hasPlatformSlots);
}

function switchCatalog(kind) {
  const next = String(kind ?? "").trim();
  emitPlatformMockAdapterPayload(
    buildMockSwitchCatalogPayload({
      catalogKind: next,
    }),
  );
  if (next === CatalogKind.PACKAGE) {
    if (typeof maybeEmitPlatformServerAdapterForOp === "function") {
      maybeEmitPlatformServerAdapterForOp(
        PlatformServerAdapterOp.SWITCH_CATALOG,
        { catalog_kind: next },
        "꾸러미 카탈로그는 준비 중입니다.",
      );
    }
    showPlatformToast("꾸러미 카탈로그는 준비 중입니다.");
    return;
  }
  if (next === CatalogKind.LESSON || next === CatalogKind.PROJECT) {
    appState.shell.activeCatalog = next;
  }
}

function saveCurrentWork(target = "local", { ddnText = "" } = {}) {
  const mode = String(target ?? "").trim().toLowerCase();
  emitPlatformMockAdapterPayload(
    buildMockSaveAdapterPayload({
      target: mode,
      ddnText: String(ddnText ?? ""),
      workId: appState.shell.currentWorkId,
      projectId: appState.shell.currentProjectId,
      revisionId: appState.shell.currentRevisionId,
      publicationId: appState.shell.currentPublicationId,
    }),
  );
  if (mode === "local") {
    saveDdnToFile(String(ddnText ?? ""), "lesson.ddn");
    if (!appState.studio || typeof appState.studio !== "object") {
      appState.studio = {};
    }
    appState.studio.localSaveStatus = "로컬 저장됨";
    if (typeof updateShellStatusRail === "function") {
      updateShellStatusRail();
    }
    return true;
  }
  if (mode === "server") {
    if (typeof maybeEmitPlatformServerAdapterForOp === "function") {
      maybeEmitPlatformServerAdapterForOp(
        PlatformServerAdapterOp.SAVE,
        {
          target: mode,
          ddn_text: String(ddnText ?? ""),
        },
        "서버 저장은 준비 중입니다.",
      );
    }
    showPlatformToast("서버 저장은 준비 중입니다.");
    return false;
  }
  if (mode === "share") {
    if (typeof maybeEmitPlatformServerAdapterForOp === "function") {
      maybeEmitPlatformServerAdapterForOp(
        PlatformServerAdapterOp.SHARE,
        {
          target: mode,
          ddn_text: String(ddnText ?? ""),
        },
        "공유 링크 생성은 준비 중입니다.",
      );
    }
    showPlatformToast("공유 링크 생성은 준비 중입니다.");
    return false;
  }
  showPlatformToast(`지원하지 않는 저장 대상입니다: ${mode || "-"}`);
  return false;
}

function restoreRevision(revisionId = "") {
  const sourceRevisionId = String(revisionId ?? "").trim() || appState.shell.currentRevisionId || "";
  emitPlatformMockAdapterPayload(
    buildMockRestoreRevisionPayload({
      sourceRevisionId,
      restoreMode: RevisionPolicy.RESTORE_MODE,
    }),
  );
  if (
    SourceManagementPolicy.REVISION_APPEND_ONLY !== true ||
    SourceManagementPolicy.RESTORE_CREATES_NEW_REVISION !== true ||
    SourceManagementPolicy.OVERWRITE_FORBIDDEN !== true
  ) {
    showPlatformToast("소스관리 정책이 잘못되었습니다. (append-only/new-revision/overwrite-forbidden)");
    return false;
  }
  if (!sourceRevisionId) {
    showPlatformToast("복원할 리비전이 없습니다.");
    return false;
  }
  if (typeof maybeEmitPlatformServerAdapterForOp === "function") {
    maybeEmitPlatformServerAdapterForOp(
      PlatformServerAdapterOp.RESTORE_REVISION,
      { source_revision_id: sourceRevisionId },
      "리비전 복원은 준비 중입니다. (새 revision으로 복원 예정)",
    );
  }
  if (RevisionPolicy.RESTORE_MODE !== "new_revision") {
    showPlatformToast("리비전 복원 정책이 잘못되었습니다.");
    return false;
  }
  showPlatformToast("리비전 복원은 준비 중입니다. (새 revision으로 복원 예정)");
  return false;
}

function openRevisionHistory() {
  showPlatformToast("리비전 기록 화면은 준비 중입니다.");
  return false;
}

function compareRevisionWithHead() {
  const sourceRevisionId = String(appState.shell.currentRevisionId ?? "").trim();
  if (!sourceRevisionId) {
    showPlatformToast("비교할 리비전이 없습니다.");
    return false;
  }
  showPlatformToast(`리비전 비교(${sourceRevisionId} vs HEAD)는 준비 중입니다.`);
  return false;
}

function duplicateCurrentWork() {
  const sourceWorkId = String(appState.shell.currentWorkId ?? "").trim();
  if (!sourceWorkId) {
    showPlatformToast("복제할 작업이 없습니다.");
    return false;
  }
  showPlatformToast(`작업 복제(${sourceWorkId})는 준비 중입니다.`);
  return false;
}

function shareCurrent(kind = ShareKind.LINK) {
  const normalized = String(kind ?? "").trim().toLowerCase();
  emitPlatformMockAdapterPayload(
    buildMockShareAdapterPayload({
      kind: normalized,
      objectKind: appState.currentLesson ? ObjectKind.LESSON : ObjectKind.PROJECT,
      objectId:
        (appState.currentLesson && String(appState.currentLesson.id ?? "").trim()) ||
        appState.shell.currentProjectId ||
        appState.shell.currentWorkId ||
        "",
      visibility: Visibility.PRIVATE,
      sourceRevisionId: appState.shell.currentRevisionId,
      linkTarget: PublicationPolicy.PUBLIC_LINK_TARGET_DEFAULT,
    }),
  );
  if (typeof maybeEmitPlatformServerAdapterForOp === "function") {
    maybeEmitPlatformServerAdapterForOp(
      PlatformServerAdapterOp.SHARE,
      {
        kind: normalized,
        lesson_id: appState.currentLesson?.id ?? null,
      },
      normalized === ShareKind.CLONE ? "복제 공유는 준비 중입니다." : "링크 공유는 준비 중입니다.",
    );
  }
  if (normalized === ShareKind.LINK) {
    showPlatformToast(`링크 공유(${PublicationPolicy.PUBLIC_LINK_TARGET_DEFAULT})는 준비 중입니다.`);
    return false;
  }
  if (normalized === ShareKind.CLONE) {
    showPlatformToast("복제 공유는 준비 중입니다.");
    return false;
  }
  if (normalized === ShareKind.PACKAGE) {
    showPlatformToast("꾸러미 공유는 준비 중입니다.");
    return false;
  }
  showPlatformToast(`지원하지 않는 공유 방식입니다: ${normalized || "-"}`);
  return false;
}

function publishCurrent() {
  emitPlatformMockAdapterPayload(
    buildMockPublishAdapterPayload({
      projectId: appState.shell.currentProjectId,
      sourceRevisionId: appState.shell.currentRevisionId,
      publicationId: appState.shell.currentPublicationId,
      visibility: Visibility.PRIVATE,
    }),
  );
  if (typeof maybeEmitPlatformServerAdapterForOp === "function") {
    maybeEmitPlatformServerAdapterForOp(
      PlatformServerAdapterOp.PUBLISH,
      {
        project_id: appState.shell.currentProjectId,
        source_revision_id: appState.shell.currentRevisionId,
      },
      "게시 기능은 준비 중입니다.",
    );
  }
  if (!RevisionPolicy.SOURCE_REVISION_ID_REQUIRED) {
    showPlatformToast("게시 정책이 잘못되었습니다.");
    return false;
  }
  if (PublishPolicy.ARTIFACT_TRACKS_DRAFT !== false) {
    showPlatformToast("게시 정책이 잘못되었습니다. (artifact draft 추적 금지)");
    return false;
  }
  if (
    PublicationPolicy.SNAPSHOT_IMMUTABLE !== true ||
    PublicationPolicy.PINNED_REVISION_REQUIRED !== true ||
    PublicationPolicy.REPUBLISH_APPEND_ONLY !== true
  ) {
    showPlatformToast("게시 스냅샷 정책이 잘못되었습니다.");
    return false;
  }
  if (PublicationPolicy.PINNED_REVISION_REQUIRED && !String(appState.shell.currentRevisionId ?? "").trim()) {
    showPlatformToast("게시 실패: source_revision_id가 필요합니다.");
    return false;
  }
  showPlatformToast("게시 기능은 준비 중입니다.");
  return false;
}

function republishCurrent() {
  const sourcePublicationId = String(appState.shell.currentPublicationId ?? "").trim();
  if (PublicationPolicy.REPUBLISH_APPEND_ONLY !== true) {
    showPlatformToast("재게시 정책이 잘못되었습니다.");
    return false;
  }
  if (!sourcePublicationId) {
    showPlatformToast("재게시할 publication이 없습니다.");
    return false;
  }
  showPlatformToast(`재게시(${sourcePublicationId})는 준비 중입니다. (새 publication record append-only)`);
  return false;
}

function requestReview() {
  appState.shell.reviewStatus = "pending";
  emitPlatformReviewAction("request", {
    reviewStatus: appState.shell.reviewStatus,
  });
  showPlatformToast("검토 요청은 준비 중입니다.");
  return false;
}

function approvePublication() {
  appState.shell.reviewStatus = "approved";
  emitPlatformReviewAction("approve", {
    reviewStatus: appState.shell.reviewStatus,
  });
  showPlatformToast("검토 승인 처리는 준비 중입니다.");
  return false;
}

function rejectPublication() {
  appState.shell.reviewStatus = "rejected";
  emitPlatformReviewAction("reject", {
    reviewStatus: appState.shell.reviewStatus,
  });
  showPlatformToast("검토 반려 처리는 준비 중입니다.");
  return false;
}

function installPackage(packageId = "", version = "") {
  const normalizedId = String(packageId ?? "").trim() || "-";
  const normalizedVersion = String(version ?? "").trim() || "latest";
  emitPlatformMockAdapterPayload(
    buildMockInstallPackagePayload({
      packageId: normalizedId,
      version: normalizedVersion,
      catalogKind: appState.shell.activeCatalog,
    }),
  );
  if (typeof maybeEmitPlatformServerAdapterForOp === "function") {
    maybeEmitPlatformServerAdapterForOp(
      PlatformServerAdapterOp.INSTALL_PACKAGE,
      {
        package_id: normalizedId,
        version: normalizedVersion,
        catalog_kind: appState.shell.activeCatalog,
      },
      `꾸러미 설치(${normalizedId}@${normalizedVersion})는 준비 중입니다.`,
    );
  }
  showPlatformToast(`꾸러미 설치(${normalizedId}@${normalizedVersion})는 준비 중입니다.`);
  return false;
}

function normalizePath(path) {
  return String(path ?? "")
    .replace(/\\/g, "/")
    .replace(/^\.\//, "")
    .replace(/^\//, "")
    .trim();
}

function normalizeDataRoot(raw) {
  return normalizePath(raw).replace(/\/+$/, "");
}

function dataPath(path) {
  const normalized = normalizePath(path);
  if (!normalized) return DATA_ROOT;
  return DATA_ROOT ? `${DATA_ROOT}/${normalized}` : normalized;
}

function isProjectPrefixedHost() {
  try {
    const pathname = String(window?.location?.pathname ?? "").trim();
    if (!pathname) return false;
    return pathname.includes(`/${PROJECT_PREFIX}`) || pathname.startsWith(`/${PROJECT_PREFIX.slice(0, -1)}`);
  } catch (_) {
    return false;
  }
}

function normalizeSubject(raw) {
  const subject = String(raw ?? "").trim().toLowerCase();
  if (subject === "economy") return "econ";
  return subject;
}

function buildPathCandidates(path) {
  const normalized = normalizePath(path);
  if (!normalized) return [];
  if (/^https?:\/\//i.test(normalized)) return [normalized];

  const stripped = normalized.startsWith(PROJECT_PREFIX)
    ? normalized.slice(PROJECT_PREFIX.length)
    : normalized;
  const prefixed = normalized.startsWith(PROJECT_PREFIX)
    ? normalized
    : `${PROJECT_PREFIX}${normalized}`;

  // 404 노이즈를 줄이기 위해 절대 경로 후보만 최소 집합으로 유지한다.
  const primary = `/${stripped}`;
  const secondary = `/${prefixed}`;
  if (primary === secondary) return [primary];
  return isProjectPrefixedHost() ? [secondary, primary] : [primary, secondary];
}

async function fetchFirstOk(urls, parseAs = "text") {
  const tried = [];
  for (const url of urls) {
    tried.push(url);
    try {
      const response = await fetch(url, { cache: "no-cache" });
      if (!response.ok) continue;
      if (parseAs === "json") {
        return { ok: true, url, data: await response.json() };
      }
      return { ok: true, url, data: await response.text() };
    } catch (_) {
      // continue
    }
  }
  return { ok: false, url: tried[tried.length - 1] ?? "", data: null };
}

async function fetchJson(path) {
  const result = await fetchFirstOk(buildPathCandidates(path), "json");
  return result.ok ? result.data : null;
}

async function fetchText(pathCandidates) {
  const list = Array.isArray(pathCandidates)
    ? pathCandidates.flatMap((item) => buildPathCandidates(item))
    : buildPathCandidates(pathCandidates);
  const result = await fetchFirstOk(Array.from(new Set(list)), "text");
  return result.ok ? result.data : null;
}

function stableObjectString(value) {
  if (Array.isArray(value)) return `[${value.map((item) => stableObjectString(item)).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableObjectString(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function fnv1a32Text(text = "") {
  let hash = 0x811c9dc5;
  const source = String(text ?? "");
  for (let index = 0; index < source.length; index += 1) {
    hash ^= source.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function buildLessonDeclarationHash({ lessonId = "", meta = null, requiredViews = [] } = {}) {
  return `candidate:fnv1a32:lesson_declaration:${fnv1a32Text(stableObjectString({
    lesson_id: String(lessonId ?? "").trim(),
    primary_view: String(meta?.primary_view ?? "").trim(),
    required_views: Array.isArray(requiredViews) ? requiredViews.map((item) => String(item ?? "").trim()).filter(Boolean) : [],
    view_roles: Array.isArray(meta?.view_roles) ? meta.view_roles.map((item) => String(item ?? "").trim()).filter(Boolean) : [],
  }))}`;
}

function buildPrimaryViewContract({ lessonId = "", meta = null, ruleset = null, requiredViews = [] } = {}) {
  const declaredPrimary = String(meta?.primary_view ?? "").trim();
  const rules = Array.isArray(ruleset?.rules) ? ruleset.rules : [];
  const matched = rules.find((rule) => String(rule?.lesson_id ?? "").trim() === String(lessonId ?? "").trim()) ?? null;
  const primaryView = String(matched?.primary_view ?? declaredPrimary ?? "").trim();
  if (!primaryView) return null;
  return {
    schema_id: String(ruleset?.schema_id ?? "seamgrim.primary_view_rules").trim(),
    schema_version: String(ruleset?.schema_version ?? "v0").trim(),
    ruleset_hash: String(ruleset?.ruleset_hash ?? "").trim(),
    lesson_declaration_hash: buildLessonDeclarationHash({ lessonId, meta, requiredViews }),
    lesson_id: String(lessonId ?? "").trim(),
    primary_view: primaryView,
    secondary_views: Array.isArray(matched?.secondary_views)
      ? matched.secondary_views.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [],
    reason: String(matched?.reason ?? "lesson_meta").trim(),
    heuristic_selection: false,
  };
}

function shouldProbeLessonInventoryApiForCatalog() {
  return (
    readWindowBoolean("SEAMGRIM_ENABLE_SERVER_FALLBACK", false) ||
    readQueryBoolean("server_fallback", false) ||
    globalThis?.SEAMGRIM_FEDERATED_API_CANDIDATES !== undefined
  );
}

async function fetchLessonInventoryApiForCatalog() {
  const inventoryApi = await fetchFirstOk(["/api/lessons/inventory", "/api/lesson-inventory"], "json");
  return inventoryApi;
}

function metaStringList(value) {
  return Array.isArray(value) ? value.map((item) => String(item ?? "").trim()).filter(Boolean) : [];
}

function buildCurriculumMeta(meta) {
  if (!meta || typeof meta !== "object") return null;
  if (String(meta.schema ?? "").trim() !== "CurriculumMetaV1") return null;
  return {
    schema: "CurriculumMetaV1",
    lessonId: String(meta.lesson_id ?? "").trim(),
    title: String(meta.title ?? "").trim(),
    subject: String(meta.subject ?? "").trim(),
    gradeBand: String(meta.grade_band ?? meta.grade ?? "").trim(),
    unit: String(meta.unit ?? "").trim(),
    lesson: String(meta.lesson ?? "").trim(),
    difficulty: String(meta.difficulty ?? "").trim(),
    learningGoals: metaStringList(meta.learning_goals),
    coreConcepts: metaStringList(meta.core_concepts),
    prerequisites: metaStringList(meta.prerequisites),
    misconceptions: metaStringList(meta.misconceptions),
    allowedControls: metaStringList(meta.allowed_controls),
    requiredViews: normalizeViewFamilyList(meta.required_views ?? []),
    evidence: metaStringList(meta.evidence),
    teacherNotesRef: String(meta.teacher_notes_ref ?? "").trim(),
    studentSheetRef: String(meta.student_sheet_ref ?? "").trim(),
  };
}

function mergeLessonWithCurriculumMeta(lesson, meta) {
  const curriculumMeta = buildCurriculumMeta(meta);
  if (!curriculumMeta) {
    const goals = metaStringList(meta?.goals ?? meta?.learning_goals);
    const missions = metaStringList(meta?.missions);
    if (goals.length === 0 && missions.length === 0) return lesson;
    return {
      ...lesson,
      goals: goals.length ? goals : lesson?.goals ?? [],
      missions: missions.length ? missions : lesson?.missions ?? [],
    };
  }
  const requiredViews = curriculumMeta.requiredViews.length
    ? curriculumMeta.requiredViews
    : normalizeViewFamilyList(lesson?.requiredViews ?? []);
  const tags = Array.from(new Set([...(lesson?.tags ?? []), ...curriculumMeta.coreConcepts]));
  return {
    ...lesson,
    title: curriculumMeta.title || lesson.title,
    description:
      String(meta.description ?? "").trim() ||
      curriculumMeta.learningGoals.slice(0, 2).join(" / ") ||
      lesson.description,
    grade: curriculumMeta.gradeBand || lesson.grade,
    subject: normalizeSubject(curriculumMeta.subject || lesson.subject),
    requiredViews,
    tags,
    goals: curriculumMeta.learningGoals.length ? curriculumMeta.learningGoals : lesson?.goals ?? [],
    curriculumMeta,
  };
}

async function enrichCatalogLessonsWithMeta(lessons) {
  const rows = Array.isArray(lessons) ? lessons : [];
  const enriched = [];
  for (const lesson of rows) {
    const metaRaw = await fetchText(lesson?.metaCandidates ?? []);
    if (!metaRaw) {
      enriched.push(lesson);
      continue;
    }
    enriched.push(mergeLessonWithCurriculumMeta(lesson, parseTomlMeta(metaRaw)));
  }
  return enriched;
}

async function enrichCatalogLessonsWithCurriculumCards(lessons) {
  const rows = Array.isArray(lessons) ? lessons : [];
  const cardsBundle =
    (await fetchJson(dataPath("lessons/lesson_cards.v0.detjson"))) ??
    (await fetchJson("solutions/seamgrim_ui_mvp/lessons/lesson_cards.v0.detjson"));
  const crosswalk =
    (await fetchJson(dataPath("lessons/curriculum_crosswalk.v0.detjson"))) ??
    (await fetchJson("solutions/seamgrim_ui_mvp/lessons/curriculum_crosswalk.v0.detjson"));
  if (!cardsBundle || !crosswalk) {
    const validation = validateCurriculumCardBundle({ cardsBundle, crosswalk });
    try {
      window.__SEAMGRIM_CURRICULUM_CARD_VALIDATION__ = validation;
    } catch (_) {
      // ignore instrumentation write errors
    }
    return rows;
  }
  const merged = mergeCurriculumCardsIntoLessons(rows, { cardsBundle, crosswalk });
  try {
    window.__SEAMGRIM_CURRICULUM_CARD_VALIDATION__ = merged.validation;
    window.__SEAMGRIM_CURRICULUM_CARD_COUNT__ = merged.validation.card_count;
  } catch (_) {
    // ignore instrumentation write errors
  }
  return merged.lessons;
}

function prioritizeLessonCandidates(candidates, source) {
  const list = Array.isArray(candidates) ? candidates.filter(Boolean).map((item) => String(item)) : [];
  if (list.length <= 1) return list;
  const mode = String(source ?? "").trim().toLowerCase();
  const rankOf = (path) => {
    const normalized = normalizePath(path);
    if (mode === "rewrite") {
      if (normalized.includes("lessons_rewrite_v1/")) return 0;
      if (normalized.includes("lessons/")) return 1;
      return 2;
    }
    if (mode === "seed") {
      if (normalized.includes("seed_lessons_v1/")) return 0;
      if (normalized.includes("lessons/")) return 1;
      return 2;
    }
    return 0;
  };
  return [...list].sort((a, b) => rankOf(a) - rankOf(b));
}

function toLessonEntry(base) {
  const id = String(base.id ?? "").trim();
  if (!id) return null;
  return {
    id,
    title: String(base.title ?? id),
    description: String(base.description ?? ""),
    grade: String(base.grade ?? ""),
    subject: normalizeSubject(base.subject),
    quality: String(base.quality ?? "experimental"),
    source: String(base.source ?? "official"),
    firstRunPath: String(base.firstRunPath ?? base.first_run_path ?? "").trim(),
    tags: Array.isArray(base.tags) ? base.tags.filter(Boolean).map((item) => String(item)) : [],
    goals: Array.isArray(base.goals) ? base.goals.filter(Boolean).map((item) => String(item)) : [],
    missions: Array.isArray(base.missions) ? base.missions.filter(Boolean).map((item) => String(item)) : [],
    requiredViews: normalizeViewFamilyList(base.requiredViews ?? base.required_views ?? []),
    ddnCandidates: Array.isArray(base.ddnCandidates) ? base.ddnCandidates.filter(Boolean) : [],
    maegimControlCandidates: Array.isArray(base.maegimControlCandidates) ? base.maegimControlCandidates.filter(Boolean) : [],
    textCandidates: Array.isArray(base.textCandidates) ? base.textCandidates.filter(Boolean) : [],
    graphCandidates: Array.isArray(base.graphCandidates) ? base.graphCandidates.filter(Boolean) : [],
    tableCandidates: Array.isArray(base.tableCandidates) ? base.tableCandidates.filter(Boolean) : [],
    space2dCandidates: Array.isArray(base.space2dCandidates) ? base.space2dCandidates.filter(Boolean) : [],
    structureCandidates: Array.isArray(base.structureCandidates) ? base.structureCandidates.filter(Boolean) : [],
    metaCandidates: Array.isArray(base.metaCandidates) ? base.metaCandidates.filter(Boolean) : [],
    maegimControlWarningCount: Math.max(0, Math.trunc(Number(base.maegimControlWarningCount) || 0)),
    maegimControlWarningCodes: Array.isArray(base.maegimControlWarningCodes)
      ? base.maegimControlWarningCodes.filter(Boolean).map((item) => String(item))
      : [],
    maegimControlWarningNames: Array.isArray(base.maegimControlWarningNames)
      ? base.maegimControlWarningNames.filter(Boolean).map((item) => String(item))
      : [],
    maegimControlWarningExamples: Array.isArray(base.maegimControlWarningExamples)
      ? base.maegimControlWarningExamples.filter(Boolean).map((item) => String(item))
      : [],
    maegimControlWarningSource: String(base.maegimControlWarningSource ?? "").trim(),
  };
}

function lessonSourcePriority(source) {
  const normalized = String(source ?? "").trim().toLowerCase();
  if (normalized === "official") return 40;
  if (normalized === "federated") return 35;
  if (normalized === "seed") return 30;
  if (normalized === "rewrite") return 10;
  return 20;
}

function mergeLessonEntry(map, nextEntry) {
  if (!nextEntry) return;
  const existing = map.get(nextEntry.id);
  if (!existing) {
    map.set(nextEntry.id, nextEntry);
    return;
  }

  const keepExistingMeta = lessonSourcePriority(existing.source) >= lessonSourcePriority(nextEntry.source);
  const mergedSource = keepExistingMeta ? existing.source : nextEntry.source;
  const mergedTitle = keepExistingMeta ? existing.title : nextEntry.title;
  const mergedDescription = keepExistingMeta ? existing.description : nextEntry.description;
  const mergedGrade = keepExistingMeta ? existing.grade : nextEntry.grade;
  const mergedSubject = keepExistingMeta ? existing.subject : nextEntry.subject;
  const mergedQuality = keepExistingMeta ? existing.quality : nextEntry.quality;

  const merged = {
    ...existing,
    ...nextEntry,
    source: mergedSource,
    title: mergedTitle,
    description: mergedDescription,
    grade: mergedGrade,
    subject: mergedSubject,
    quality: mergedQuality,
    firstRunPath: String(existing.firstRunPath ?? "").trim() || String(nextEntry.firstRunPath ?? "").trim(),
    tags: Array.from(new Set([...(existing.tags ?? []), ...(nextEntry.tags ?? [])])),
    goals: Array.from(new Set([...(existing.goals ?? []), ...(nextEntry.goals ?? [])])),
    missions: Array.from(new Set([...(existing.missions ?? []), ...(nextEntry.missions ?? [])])),
    requiredViews: Array.from(new Set([...(existing.requiredViews ?? []), ...(nextEntry.requiredViews ?? [])])),
    ddnCandidates: Array.from(new Set([...(existing.ddnCandidates ?? []), ...(nextEntry.ddnCandidates ?? [])])),
    maegimControlCandidates: Array.from(
      new Set([...(existing.maegimControlCandidates ?? []), ...(nextEntry.maegimControlCandidates ?? [])]),
    ),
    textCandidates: Array.from(new Set([...(existing.textCandidates ?? []), ...(nextEntry.textCandidates ?? [])])),
    graphCandidates: Array.from(new Set([...(existing.graphCandidates ?? []), ...(nextEntry.graphCandidates ?? [])])),
    tableCandidates: Array.from(new Set([...(existing.tableCandidates ?? []), ...(nextEntry.tableCandidates ?? [])])),
    space2dCandidates: Array.from(new Set([...(existing.space2dCandidates ?? []), ...(nextEntry.space2dCandidates ?? [])])),
    structureCandidates: Array.from(new Set([...(existing.structureCandidates ?? []), ...(nextEntry.structureCandidates ?? [])])),
    metaCandidates: Array.from(new Set([...(existing.metaCandidates ?? []), ...(nextEntry.metaCandidates ?? [])])),
    maegimControlWarningCount: Math.max(
      Number(existing.maegimControlWarningCount) || 0,
      Number(nextEntry.maegimControlWarningCount) || 0,
    ),
    maegimControlWarningCodes: Array.from(
      new Set([...(existing.maegimControlWarningCodes ?? []), ...(nextEntry.maegimControlWarningCodes ?? [])]),
    ),
    maegimControlWarningNames: Array.from(
      new Set([...(existing.maegimControlWarningNames ?? []), ...(nextEntry.maegimControlWarningNames ?? [])]),
    ),
    maegimControlWarningExamples: Array.from(
      new Set([...(existing.maegimControlWarningExamples ?? []), ...(nextEntry.maegimControlWarningExamples ?? [])]),
    ),
    maegimControlWarningSource:
      String(nextEntry.maegimControlWarningSource ?? "").trim() ||
      String(existing.maegimControlWarningSource ?? "").trim(),
  };
  map.set(nextEntry.id, merged);
}

function lessonPathsFromId(prefix, lessonId) {
  return {
    ddn: `${prefix}/${lessonId}/lesson.ddn`,
    maegimControl: `${prefix}/${lessonId}/maegim_control.json`,
    text: `${prefix}/${lessonId}/text.md`,
    graph: `${prefix}/${lessonId}/graph.json`,
    table: `${prefix}/${lessonId}/table.json`,
    space2d: `${prefix}/${lessonId}/space2d.json`,
    structure: `${prefix}/${lessonId}/structure.json`,
    meta: `${prefix}/${lessonId}/meta.toml`,
  };
}

function sourceToLessonPrefix(source) {
  const normalized = String(source ?? "").trim().toLowerCase();
  if (normalized === "seed") return dataPath("seed_lessons_v1");
  if (normalized === "rewrite") return dataPath("lessons_rewrite_v1");
  return dataPath("lessons");
}

function toStringArray(value) {
  if (Array.isArray(value)) {
    return value.map((item) => String(item ?? "").trim()).filter(Boolean);
  }
  const single = String(value ?? "").trim();
  return single ? [single] : [];
}

function resolveSelectionCandidates(selection, keys) {
  const out = [];
  keys.forEach((key) => {
    out.push(...toStringArray(selection?.[key]));
  });
  return Array.from(new Set(out));
}

function deriveMaegimControlCandidates(ddnCandidates, explicitCandidates = []) {
  const explicit = Array.isArray(explicitCandidates) ? explicitCandidates.filter(Boolean).map((item) => String(item)) : [];
  if (explicit.length) return Array.from(new Set(explicit));
  const inferred = [];
  const ddnList = Array.isArray(ddnCandidates) ? ddnCandidates : [];
  ddnList.forEach((candidate) => {
    const normalized = normalizePath(candidate);
    if (!normalized) return;
    if (normalized.endsWith("/lesson.ddn")) {
      inferred.push(`${normalized.slice(0, -"/lesson.ddn".length)}/maegim_control.json`);
    }
  });
  return Array.from(new Set(inferred));
}

function ensureLessonEntryFromSelection(selection) {
  if (typeof selection === "string") {
    return String(selection).trim();
  }
  if (!selection || typeof selection !== "object") {
    return "";
  }
  const id = String(selection.id ?? selection.lesson_id ?? "").trim();
  if (!id) return "";

  const source = String(selection.source ?? "federated");
  const fallback = lessonPathsFromId(sourceToLessonPrefix(source), id);
  const ddnCandidates = resolveSelectionCandidates(selection, ["ddnCandidates", "ddn_path", "lesson_ddn_path"]);
  const maegimControlCandidates = resolveSelectionCandidates(selection, ["maegimControlCandidates", "maegim_control_path"]);
  const textCandidates = resolveSelectionCandidates(selection, ["textCandidates", "text_path", "text_md_path"]);
  const graphCandidates = resolveSelectionCandidates(selection, ["graphCandidates", "graph_path", "graph_json_path"]);
  const tableCandidates = resolveSelectionCandidates(selection, ["tableCandidates", "table_path", "table_json_path", "table_csv_path"]);
  const space2dCandidates = resolveSelectionCandidates(selection, ["space2dCandidates", "space2d_path", "space2d_json_path", "2d_path"]);
  const structureCandidates = resolveSelectionCandidates(selection, ["structureCandidates", "structure_path"]);
  const metaCandidates = resolveSelectionCandidates(selection, ["metaCandidates", "meta_path"]);
  const requiredViews = normalizeViewFamilyList(selection.requiredViews ?? selection.required_views ?? []);
  const needsTextCandidate = requiredViews.some((view) => view === "text" || view === "structure");

  const nextEntry = toLessonEntry({
    id,
    title: selection.title,
    description: selection.description,
    grade: selection.grade,
    subject: selection.subject,
    quality: selection.quality,
    source,
    firstRunPath: selection.firstRunPath ?? selection.first_run_path ?? "",
    tags: selection.tags ?? [],
    requiredViews,
    ddnCandidates: ddnCandidates.length ? ddnCandidates : [fallback.ddn],
    maegimControlCandidates,
    textCandidates: textCandidates.length || needsTextCandidate
      ? (textCandidates.length ? textCandidates : [fallback.text])
      : [],
    graphCandidates: graphCandidates.length ? graphCandidates : [fallback.graph],
    tableCandidates: tableCandidates.length ? tableCandidates : [fallback.table],
    space2dCandidates: space2dCandidates.length ? space2dCandidates : [fallback.space2d],
    structureCandidates: structureCandidates.length ? structureCandidates : [fallback.structure],
    // meta.toml is optional; do not synthesize fallback paths that trigger noisy 404s.
    metaCandidates,
    maegimControlWarningCount: selection.maegim_control_warning_count ?? selection.maegimControlWarningCount ?? 0,
    maegimControlWarningCodes: selection.maegim_control_warning_codes ?? selection.maegimControlWarningCodes ?? [],
    maegimControlWarningNames: selection.maegim_control_warning_names ?? selection.maegimControlWarningNames ?? [],
    maegimControlWarningExamples:
      selection.maegim_control_warning_examples ?? selection.maegimControlWarningExamples ?? [],
    maegimControlWarningSource: selection.maegim_control_warning_source ?? selection.maegimControlWarningSource ?? "",
  });
  mergeLessonEntry(appState.lessonsById, nextEntry);
  return id;
}

function mergeCatalogFromInventoryPayload(merged, payload) {
  const rows = Array.isArray(payload?.lessons) ? payload.lessons : [];
  rows.forEach((row) => {
    const id = String(row?.id ?? row?.lesson_id ?? "").trim();
    if (!id) return;
    const source = String(row?.source ?? "official").trim() || "official";
    const fallback = lessonPathsFromId(sourceToLessonPrefix(source), id);
    const ddnCandidates = resolveSelectionCandidates(row, ["ddnCandidates", "ddn_path", "lesson_ddn_path"]);
    const maegimControlCandidates = resolveSelectionCandidates(row, ["maegimControlCandidates", "maegim_control_path"]);
    const textCandidates = resolveSelectionCandidates(row, ["textCandidates", "text_path", "text_md_path"]);
    const graphCandidates = resolveSelectionCandidates(row, ["graphCandidates", "graph_path", "graph_json_path"]);
    const tableCandidates = resolveSelectionCandidates(row, ["tableCandidates", "table_path", "table_json_path", "table_csv_path"]);
    const space2dCandidates = resolveSelectionCandidates(row, ["space2dCandidates", "space2d_path", "space2d_json_path", "2d_path"]);
    const structureCandidates = resolveSelectionCandidates(row, ["structureCandidates", "structure_path"]);
    const metaCandidates = resolveSelectionCandidates(row, ["metaCandidates", "meta_path"]);
    const requiredViews = normalizeViewFamilyList(row?.requiredViews ?? row?.required_views ?? []);
    const needsTextCandidate = requiredViews.some((view) => view === "text" || view === "structure");
    mergeLessonEntry(
      merged,
      toLessonEntry({
        id,
        title: row?.title ?? row?.name ?? id,
        description: row?.description ?? "",
        grade: row?.grade ?? "all",
        subject: row?.subject ?? "",
        quality: row?.quality ?? "experimental",
        source,
        firstRunPath: row?.firstRunPath ?? row?.first_run_path ?? "",
        tags: row?.tags ?? [],
        requiredViews,
        ddnCandidates: ddnCandidates.length ? ddnCandidates : [fallback.ddn],
        maegimControlCandidates,
        textCandidates: textCandidates.length || needsTextCandidate
          ? (textCandidates.length ? textCandidates : [fallback.text])
          : [],
        graphCandidates: graphCandidates.length ? graphCandidates : [fallback.graph],
        tableCandidates: tableCandidates.length ? tableCandidates : [fallback.table],
        space2dCandidates: space2dCandidates.length ? space2dCandidates : [fallback.space2d],
        structureCandidates: structureCandidates.length ? structureCandidates : [fallback.structure],
        metaCandidates,
        maegimControlWarningCount: row?.maegim_control_warning_count ?? row?.maegimControlWarningCount ?? 0,
        maegimControlWarningCodes: row?.maegim_control_warning_codes ?? row?.maegimControlWarningCodes ?? [],
        maegimControlWarningNames: row?.maegim_control_warning_names ?? row?.maegimControlWarningNames ?? [],
        maegimControlWarningExamples: row?.maegim_control_warning_examples ?? row?.maegimControlWarningExamples ?? [],
        maegimControlWarningSource: row?.maegim_control_warning_source ?? row?.maegimControlWarningSource ?? "",
      }),
    );
  });
}

async function loadActiveLessonAllowlist() {
  const payload = await fetchJson(ACTIVE_ALLOWLIST_PATH);
  const rows = Array.isArray(payload?.lesson_ids) ? payload.lesson_ids : [];
  const out = new Set();
  const rawIds = [];
  const duplicateIds = [];
  const seen = new Set();
  rows.forEach((raw) => {
    const lessonId = String(raw ?? "").trim();
    if (!lessonId) return;
    rawIds.push(lessonId);
    if (seen.has(lessonId) && !duplicateIds.includes(lessonId)) {
      duplicateIds.push(lessonId);
    }
    seen.add(lessonId);
    out.add(lessonId);
  });
  out.schema = String(payload?.schema ?? "").trim();
  out.mode = String(payload?.mode ?? "").trim();
  out.rawIds = rawIds;
  out.duplicateIds = duplicateIds;
  return out;
}

function publishLessonLibraryCurationSnapshot(snapshot) {
  const row = snapshot && typeof snapshot === "object" ? snapshot : null;
  appState.lessonLibraryCuration = row;
  try {
    window.__SEAMGRIM_LESSON_LIBRARY_CURATION__ = row;
    window.__SEAMGRIM_LESSON_LIBRARY_CURATION_TEXT__ = row
      ? formatLessonLibraryCurationText(row)
      : "";
  } catch (_) {
    // ignore browser instrumentation errors
  }
}

function publishNumericTrackIndexSnapshot(snapshot) {
  const row = snapshot && typeof snapshot === "object" ? snapshot : null;
  appState.numericTrackIndex = row;
  try {
    window.__SEAMGRIM_NUMERIC_TRACK_INDEX__ = row;
    window.__SEAMGRIM_NUMERIC_TRACK_INDEX_TEXT__ = row
      ? formatNumericTrackIndexText(row)
      : "";
  } catch (_) {
    // ignore browser instrumentation errors
  }
}

function publishNumericTrackReportExport(report) {
  const row = report && typeof report === "object" ? report : null;
  appState.numericTrackReportExport = row;
  try {
    window.__SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT__ = row;
    window.__SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT_TEXT__ = row
      ? formatNumericTrackReportExportText(row)
      : "";
  } catch (_) {
    // ignore browser instrumentation errors
  }
}

function filterCatalogByAllowlist(merged, allowlist) {
  if (!(allowlist instanceof Set) || allowlist.size === 0) return;
  Array.from(merged.keys()).forEach((lessonId) => {
    if (!allowlist.has(lessonId)) {
      merged.delete(lessonId);
    }
  });
}

async function mergeSeedLessonsIntoCatalog(merged, { featuredOnly = false } = {}) {
  const seedManifest =
    (await fetchJson(dataPath("seed_lessons_v1/seed_manifest.detjson"))) ??
    (await fetchJson("solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson"));
  const seeds = Array.isArray(seedManifest?.seeds) ? seedManifest.seeds : [];
  seeds.forEach((seed) => {
    const id = String(seed.seed_id ?? "").trim();
    if (!id) return;
    if (featuredOnly && !FEATURED_SEED_IDS.includes(id)) return;
    const fallback = lessonPathsFromId(dataPath("seed_lessons_v1"), id);
    mergeLessonEntry(
      merged,
      toLessonEntry({
        id,
        title: id,
        description: "Seed lesson",
        grade: "all",
        subject: seed.subject,
        quality: "recommended",
        source: "seed",
        ddnCandidates: [seed.lesson_ddn, fallback.ddn],
        maegimControlCandidates: deriveMaegimControlCandidates(
          [seed.lesson_ddn, fallback.ddn],
          resolveSelectionCandidates(seed, ["maegimControlCandidates", "maegim_control_path"]),
        ),
        textCandidates: [seed.text_md, fallback.text],
        graphCandidates: [fallback.graph],
        tableCandidates: [fallback.table],
        space2dCandidates: [fallback.space2d],
        structureCandidates: [fallback.structure],
        metaCandidates: resolveSelectionCandidates(seed, ["metaCandidates", "meta_path", "meta_toml"]),
      }),
    );
  });
}

async function loadCatalogLessons() {
  const catalogMode = resolveCatalogMode();
  appState.catalogMode = catalogMode;
  const repsOnly = catalogMode !== CATALOG_MODE_FULL;
  const activeAllowlist = await loadActiveLessonAllowlist();
  const merged = new Map();

  const inventoryApi = shouldProbeLessonInventoryApiForCatalog()
    ? await fetchLessonInventoryApiForCatalog()
    : { ok: false, url: "", data: null };
  if (inventoryApi.ok) {
    mergeCatalogFromInventoryPayload(merged, inventoryApi.data);
    if (repsOnly) {
      filterCatalogByAllowlist(merged, activeAllowlist);
    }
  }

  if (merged.size === 0) {
    const indexJson =
      (await fetchJson(dataPath("lessons/index.json"))) ??
      (await fetchJson("solutions/seamgrim_ui_mvp/lessons/index.json"));
    const indexLessons = Array.isArray(indexJson?.lessons) ? indexJson.lessons : [];
    indexLessons.forEach((row) => {
      const id = String(row.id ?? "").trim();
      if (!id) return;
      if (repsOnly && activeAllowlist.size > 0 && !activeAllowlist.has(id)) return;
      const paths = lessonPathsFromId(dataPath("lessons"), id);
      const rowMetaCandidates = resolveSelectionCandidates(row, ["metaCandidates", "meta_path"]);
      const rowMaegimControlCandidates = resolveSelectionCandidates(row, ["maegimControlCandidates", "maegim_control_path"]);
      const requiredViews = normalizeViewFamilyList(row.requiredViews ?? row.required_views ?? []);
      const needsTextCandidate = requiredViews.some((view) => view === "text" || view === "structure");
      mergeLessonEntry(
        merged,
        toLessonEntry({
          id,
          title: row.title,
          description: row.description,
          grade: row.grade,
          subject: row.subject,
          quality: "experimental",
          source: "official",
          goals: row.goals ?? [],
          missions: row.missions ?? [],
          requiredViews,
          ddnCandidates: [paths.ddn],
          maegimControlCandidates: rowMaegimControlCandidates,
          textCandidates: needsTextCandidate ? [paths.text] : [],
          graphCandidates: [paths.graph],
          tableCandidates: [paths.table],
          space2dCandidates: [paths.space2d],
          structureCandidates: [paths.structure],
          metaCandidates: rowMetaCandidates,
        }),
      );
    });

    if (!repsOnly) {
      await mergeSeedLessonsIntoCatalog(merged);

      const rewriteManifest =
        (await fetchJson(dataPath("lessons_rewrite_v1/rewrite_manifest.detjson"))) ??
        (await fetchJson("solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson"));
      const generated = Array.isArray(rewriteManifest?.generated) ? rewriteManifest.generated : [];
      generated.forEach((row) => {
        const id = String(row.lesson_id ?? "").trim();
        if (!id) return;
        const fallback = lessonPathsFromId(dataPath("lessons_rewrite_v1"), id);
        mergeLessonEntry(
          merged,
          toLessonEntry({
            id,
            title: id,
            description: "Rewrite v1",
            grade: "all",
            subject: row.subject,
            quality: "reviewed",
            source: "rewrite",
            ddnCandidates: [row.generated_lesson_ddn, fallback.ddn],
            maegimControlCandidates: deriveMaegimControlCandidates(
              [row.generated_lesson_ddn, fallback.ddn],
              resolveSelectionCandidates(row, ["maegimControlCandidates", "maegim_control_path"]),
            ),
            textCandidates: [row.generated_text_md, fallback.text],
            graphCandidates: [fallback.graph],
            tableCandidates: [fallback.table],
            space2dCandidates: [fallback.space2d],
            structureCandidates: [fallback.structure],
            metaCandidates: resolveSelectionCandidates(row, ["metaCandidates", "meta_path", "meta_toml"]),
          }),
        );
      });
    }
  }

  let lessons = Array.from(merged.values()).sort((a, b) => String(a.title).localeCompare(String(b.title), "ko"));
  lessons = await enrichCatalogLessonsWithMeta(lessons);
  lessons = await enrichCatalogLessonsWithCurriculumCards(lessons);
  if (lessons.length === 0) {
    throw new Error(
      "교과 카탈로그를 찾지 못했습니다. ddn_exec_server.py(8787)로 실행했는지 확인하세요.",
    );
  }
  lessons.forEach((lesson) => {
    appState.lessonsById.set(lesson.id, lesson);
  });
  publishLessonLibraryCurationSnapshot(
    buildLessonLibraryCurationSnapshot({
      lessons,
      allowlist: activeAllowlist,
      catalogMode,
    }),
  );
  publishNumericTrackIndexSnapshot(buildNumericTrackIndexSnapshot(lessons));
  publishNumericTrackReportExport(buildNumericTrackReportExport(lessons));
  return lessons;
}

async function loadLessonById(lessonId) {
  const base = appState.lessonsById.get(lessonId);
  if (!base) throw new Error(`교과를 찾지 못했습니다: ${lessonId}`);

  const directDdnText = String(base.ddnText ?? "").trim();
  if (directDdnText) {
    const lesson = await lessonCanonHydrator.hydrateLessonCanon({
      ...base,
      ddnText: String(base.ddnText ?? ""),
      requiredViews: normalizeViewFamilyList(base.requiredViews ?? base.required_views ?? []),
    });
    appState.currentLesson = lesson;
    appState.lessonsById.set(lesson.id, lesson);
    const packageId = String(lesson.localPackageId ?? "").trim();
    const originalLessonId = String(lesson.localPackageLessonId ?? lesson.id ?? "").trim();
    trackLessonInputSource(lesson, {
      sourceId: packageId
        ? `local_package:${safeLocalPackageToken(packageId, "local_package")}:${safeLocalPackageToken(originalLessonId, "local_package_lesson")}`
        : `lesson:${lesson.id}`,
    });
    persistInputRegistrySnapshot();
    return lesson;
  }

  base.ddnCandidates = prioritizeLessonCandidates(base.ddnCandidates, base.source);
  base.textCandidates = prioritizeLessonCandidates(base.textCandidates, base.source);
  base.graphCandidates = prioritizeLessonCandidates(base.graphCandidates, base.source);
  base.tableCandidates = prioritizeLessonCandidates(base.tableCandidates, base.source);
  base.space2dCandidates = prioritizeLessonCandidates(base.space2dCandidates, base.source);
  base.structureCandidates = prioritizeLessonCandidates(base.structureCandidates, base.source);
  base.metaCandidates = prioritizeLessonCandidates(base.metaCandidates, base.source);

  const ddnText = await fetchText(base.ddnCandidates);
  if (!ddnText) {
    throw new Error(`lesson.ddn 로드 실패: ${lessonId}`);
  }

  const maegimControlCandidates = Array.isArray(base.maegimControlCandidates) ? base.maegimControlCandidates : [];
  const maegimControlJson = (await fetchText(maegimControlCandidates)) ?? "";
  const textMd = (await fetchText(base.textCandidates)) ?? "";
  const metaRaw = await fetchText(base.metaCandidates);
  const meta = parseTomlMeta(metaRaw);
  const representationPairPath = String(meta.representation_pair_path ?? "").trim();
  const representationPair = representationPairPath ? await fetchJson(representationPairPath) : null;
  const worldDiffPath = String(meta.world_diff_path ?? "").trim();
  const worldDiff = worldDiffPath ? await fetchJson(worldDiffPath) : null;
  const viewPlanPath = String(meta.view_plan_path ?? "").trim();
  const viewPlan = viewPlanPath ? await fetchJson(viewPlanPath) : null;
  const sealedTraceReactionsPath = String(meta.sealed_trace_reactions_path ?? "").trim();
  const sealedTraceReactions = sealedTraceReactionsPath ? await fetchJson(sealedTraceReactionsPath) : null;
  const primaryViewRulesPath = String(meta.primary_view_rules_path ?? "").trim();
  const primaryViewRules = primaryViewRulesPath ? await fetchJson(primaryViewRulesPath) : null;
  const ddnMetaHeader = parseLessonDdnMetaHeader(ddnText);
  const displayMeta = resolveLessonDisplayMeta({
    baseTitle: base.title,
    baseDescription: base.description,
    tomlMeta: meta,
    ddnMetaHeader,
  });
  const lessonPlanPath = String(meta.lesson_plan_path ?? "").trim();
  const bundledLessonPlanText = lessonPlanPath ? (await fetchText([lessonPlanPath]) ?? "") : "";
  const generatedLessonPlanText = buildDefaultLessonPlanTextForLesson({
    ...base,
    title: displayMeta.title || base.title,
  });
  const lessonPlanText = readLessonPlanDraft(base.id) || bundledLessonPlanText || generatedLessonPlanText;
  const lessonPlan = lessonPlanText
    ? parseLessonPlanToml(lessonPlanText, {
      fallbackLessonId: base.id,
      fallbackTitle: displayMeta.title || base.title,
    })
    : null;

  const lessonBase = mergeLessonWithCurriculumMeta({
    ...base,
    title: displayMeta.title || base.title,
    description: displayMeta.description || base.description,
    grade: meta.grade || base.grade,
    subject: normalizeSubject(meta.subject || base.subject),
    quality: String(meta.quality ?? base.quality ?? "experimental"),
    requiredViews: normalizeViewFamilyList(
      meta.required_views ?? ddnMetaHeader.requiredViews ?? ddnMetaHeader.required_views ?? base.requiredViews ?? [],
    ),
    ddnText,
    maegimControlJson,
    textMd,
    graphCandidates: base.graphCandidates,
    tableCandidates: base.tableCandidates,
    space2dCandidates: base.space2dCandidates,
    structureCandidates: base.structureCandidates,
    ddnMetaHeader,
    meta,
    representationPair,
    worldDiff,
    viewPlan,
    sealedTraceReactions,
    lessonPlanPath,
    lessonPlanText,
    lessonPlan,
  }, meta);

  const lesson = await lessonCanonHydrator.hydrateLessonCanon(lessonBase);
  if (lessonPlan) lesson.lessonPlan = lessonPlan;
  if (lessonPlanText) lesson.lessonPlanText = lessonPlanText;
  if (lessonPlanPath) lesson.lessonPlanPath = lessonPlanPath;
  const primaryViewContract = buildPrimaryViewContract({
    lessonId: lesson?.id ?? lessonBase.id,
    meta,
    ruleset: primaryViewRules,
    requiredViews: lesson?.requiredViews ?? lessonBase.requiredViews ?? [],
  });
  if (primaryViewRules) lesson.primaryViewRules = primaryViewRules;
  if (primaryViewContract) lesson.primaryViewContract = primaryViewContract;

  appState.currentLesson = lesson;
  appState.lessonsById.set(lesson.id, lesson);
  trackLessonInputSource(lesson, { sourceId: `lesson:${lesson.id}` });
  persistInputRegistrySnapshot();
  return lesson;
}

async function buildStudioDraftLessonFromDdn(ddnText, {
  id = STUDIO_DRAFT_LESSON_ID,
  title = "작업실 DDN",
  description = "작업실에서 직접 작성한 DDN",
  subject = "",
  grade = "",
  requiredViews = [],
  goals = [],
  missions = [],
} = {}) {
  const sourceText = String(ddnText ?? "");
  const baseMeta = {};
  const ddnMetaHeader = parseLessonDdnMetaHeader(sourceText);
  const displayMeta = resolveLessonDisplayMeta({
    baseTitle: title,
    baseDescription: description,
    tomlMeta: baseMeta,
    ddnMetaHeader,
  });
  const importedRequiredViews = Array.isArray(requiredViews)
    ? requiredViews
    : [];
  const fallbackRequiredViews =
    baseMeta.required_views ??
    ddnMetaHeader.requiredViews ??
    ddnMetaHeader.required_views ??
    appState.currentLesson?.requiredViews ??
    [];
  const lesson = await lessonCanonHydrator.hydrateLessonCanon({
    id: String(id ?? STUDIO_DRAFT_LESSON_ID).trim() || STUDIO_DRAFT_LESSON_ID,
    title: displayMeta.title || title,
    description: displayMeta.description || description,
    subject: normalizeSubject(subject || appState.currentLesson?.subject || ""),
    grade: String(grade || appState.currentLesson?.grade || "").trim(),
    quality: appState.currentLesson?.quality ?? "experimental",
    requiredViews: normalizeViewFamilyList(
      importedRequiredViews.length ? importedRequiredViews : fallbackRequiredViews,
    ),
    goals: Array.isArray(goals) ? goals.map((item) => String(item ?? "").trim()).filter(Boolean) : [],
    missions: Array.isArray(missions) ? missions.map((item) => String(item ?? "").trim()).filter(Boolean) : [],
    ddnText: sourceText,
    maegimControlJson: "",
    textMd: "",
    meta: baseMeta,
    ddnMetaHeader,
  });
  appState.currentLesson = lesson;
  appState.lessonsById.set(lesson.id, lesson);
  return lesson;
}

function createAdvancedMenuButton({ id = "", text = "", className = "", disabled = false, title = "" } = {}) {
  const button = document.createElement("button");
  button.type = "button";
  if (id) button.id = id;
  if (className) button.className = className;
  button.disabled = Boolean(disabled);
  if (title) button.title = title;
  button.textContent = text;
  return button;
}

function ensureAdvancedMenuRoot() {
  const existing = byId("advanced-menu");
  if (existing) return existing;
  if (!DEV_SURFACES_ENABLED) return null;
  const root = document.querySelector(".app-root") || document.body;
  if (!root) return null;
  const menu = document.createElement("div");
  menu.id = "advanced-menu";
  menu.className = "advanced-menu hidden";
  menu.style.position = "fixed";
  menu.style.top = "56px";
  menu.style.right = "12px";
  menu.style.width = "220px";
  menu.style.border = "1px solid var(--line)";
  menu.style.borderRadius = "12px";
  menu.style.background = "#fff";
  menu.style.boxShadow = "0 10px 30px rgba(15, 23, 42, 0.16)";
  menu.style.padding = "8px";
  menu.style.display = "flex";
  menu.style.flexDirection = "column";
  menu.style.gap = "6px";
  menu.style.zIndex = "50";
  [
    { id: "advanced-smoke", text: "Smoke 검증" },
    { id: "btn-save-server", text: "서버에 저장 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-share-link", text: "공유 링크 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-revision-history", text: "리비전 기록 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-revision-compare", text: "리비전 비교 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-work-duplicate", text: "작업 복제 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-share-clone", text: "복제해서 사용 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-publish", text: "게시 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-republish", text: "재게시 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-publication-history", text: "게시 이력 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-package-catalog", text: "꾸러미 카탈로그 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-package-deps", text: "의존 꾸러미 보기 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-review-request", text: "검토 요청 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-review-approve", text: "검토 승인 (준비 중)", disabled: true, title: "준비 중" },
    { id: "btn-review-reject", text: "검토 반려 (준비 중)", disabled: true, title: "준비 중" },
    { className: "age-slot", text: "A 실시간 입력", disabled: true },
    { className: "age-slot", text: "B 꾸러미 브라우저", disabled: true },
    { className: "age-slot", text: "C 3D 렌더", disabled: true },
  ].forEach((item) => {
    const button = createAdvancedMenuButton(item);
    button.style.textAlign = "left";
    menu.appendChild(button);
  });
  root.appendChild(menu);
  return menu;
}

function createNoopAdvancedMenu() {
  return {
    toggle() {},
    close() {},
  };
}

function createAdvancedMenu({ onSmoke }) {
  const menu = ensureAdvancedMenuRoot();
  const smokeBtn = byId("advanced-smoke");
  const saveServerBtn = byId("btn-save-server");
  const shareLinkBtn = byId("btn-share-link");
  const revisionHistoryBtn = byId("btn-revision-history");
  const revisionCompareBtn = byId("btn-revision-compare");
  const workDuplicateBtn = byId("btn-work-duplicate");
  const shareCloneBtn = byId("btn-share-clone");
  const publishBtn = byId("btn-publish");
  const republishBtn = byId("btn-republish");
  const publicationHistoryBtn = byId("btn-publication-history");
  const packageCatalogBtn = byId("btn-package-catalog");
  const packageDepsBtn = byId("btn-package-deps");
  const reviewRequestBtn = byId("btn-review-request");
  const reviewApproveBtn = byId("btn-review-approve");
  const reviewRejectBtn = byId("btn-review-reject");
  const platformMenuButtons = [
    saveServerBtn,
    shareLinkBtn,
    revisionHistoryBtn,
    revisionCompareBtn,
    workDuplicateBtn,
    shareCloneBtn,
    publishBtn,
    republishBtn,
    publicationHistoryBtn,
    packageCatalogBtn,
    packageDepsBtn,
    reviewRequestBtn,
    reviewApproveBtn,
    reviewRejectBtn,
  ];
  const platformMockMenuEnabled = applyPlatformMockMenuState(platformMenuButtons, resolvePlatformMockMenuEnabled());
  if (platformMockMenuEnabled) {
    showPlatformToast("플랫폼 mock 메뉴가 활성화되었습니다. 서버/공유 기능은 mock payload만 기록됩니다.");
  }
  if (resolvePlatformServerAdapterEnabled()) {
    showPlatformToast("플랫폼 server adapter가 활성화되었습니다. 요청/응답 스냅샷을 기록합니다.");
  }

  smokeBtn?.addEventListener("click", async () => {
    menu?.classList.add("hidden");
    if (typeof onSmoke === "function") {
      await onSmoke();
    }
  });
  saveServerBtn?.addEventListener("click", () => {
    saveCurrentWork("server");
  });
  shareLinkBtn?.addEventListener("click", () => {
    saveCurrentWork("share");
  });
  revisionHistoryBtn?.addEventListener("click", () => {
    openRevisionHistory();
  });
  revisionCompareBtn?.addEventListener("click", () => {
    compareRevisionWithHead();
  });
  workDuplicateBtn?.addEventListener("click", () => {
    duplicateCurrentWork();
  });
  shareCloneBtn?.addEventListener("click", () => {
    shareCurrent(ShareKind.CLONE);
  });
  publishBtn?.addEventListener("click", () => {
    publishCurrent();
  });
  republishBtn?.addEventListener("click", () => {
    republishCurrent();
  });
  publicationHistoryBtn?.addEventListener("click", () => {
    showPlatformToast("게시 이력 화면은 준비 중입니다.");
  });
  packageCatalogBtn?.addEventListener("click", () => {
    switchCatalog(CatalogKind.PACKAGE);
  });
  packageDepsBtn?.addEventListener("click", () => {
    installPackage("표준/물리/중력", "0.0.0");
  });
  reviewRequestBtn?.addEventListener("click", () => {
    requestReview();
  });
  reviewApproveBtn?.addEventListener("click", () => {
    approvePublication();
  });
  reviewRejectBtn?.addEventListener("click", () => {
    rejectPublication();
  });

  window.addEventListener("click", (event) => {
    if (!menu || menu.classList.contains("hidden")) return;
    const target = event.target;
    if (!(target instanceof Element)) return;
    if (menu.contains(target)) return;
    const buttonIds = new Set([
      "btn-advanced-browse",
      "btn-advanced-editor",
      "btn-advanced-block-editor",
      "btn-advanced-run",
    ]);
    if (buttonIds.has(target.id)) return;
    menu.classList.add("hidden");
  });

  return {
    toggle() {
      menu?.classList.toggle("hidden");
    },
    close() {
      menu?.classList.add("hidden");
    },
  };
}

function getFoundation12LessonChoices() {
  return FOUNDATION12_LESSON_IDS.map((lessonId) => {
    const lesson = appState.lessonsById.get(lessonId) ?? {};
    return {
      lesson_id: lessonId,
      title: String(lesson.title ?? lessonId).trim() || lessonId,
      grade: String(lesson.grade ?? "").trim(),
      subject: String(lesson.subject ?? "").trim(),
    };
  });
}

async function main() {
  applySimCorePolicy();
  restoreOverlaySessionSnapshot();
  restoreInputRegistrySnapshot();
  restoreSceneSummarySnapshot();
  restoreRuntimeSnapshotBundleV0();

  appState.wasm.loader = createWasmLoader({
    cacheBust: Date.now(),
    modulePath: "./wasm/ddonirang_tool.js",
    wrapperPath: "./wasm_ddn_wrapper.js",
    setStatus: () => {},
    clearStatusError: () => {},
  });

  const federatedApiCandidates = readWindowStringArray("SEAMGRIM_FEDERATED_API_CANDIDATES", [
    "/api/lessons/inventory",
  ]);
  const allowFederatedFileFallback = readWindowBoolean("SEAMGRIM_ENABLE_FEDERATED_FILE_FALLBACK", false);
  const allowShapeFallback = readWindowBoolean("SEAMGRIM_ENABLE_SHAPE_FALLBACK", false);
  const allowServerFallback =
    readWindowBoolean("SEAMGRIM_ENABLE_SERVER_FALLBACK", false) ||
    readQueryBoolean("server_fallback", false);
  const federatedFileCandidates = allowFederatedFileFallback
    ? readWindowStringArray("SEAMGRIM_FEDERATED_FILE_CANDIDATES", [])
    : [];
  const browsePresetFromLocation = readBrowsePresetFromLocation();
  const firstDelightLessonId = readFirstDelightFromLocation();
  const learningSpineFromLocation = readQueryBoolean(LEARNING_SPINE_QUERY_KEY, false);
  const studioRouteRequest = readStudioRouteRequestFromLocation();
  const platformRouteSlots = readPlatformRouteSlotsFromLocation();
  const catalogMode = resolveCatalogMode();
  appState.catalogMode = catalogMode;
  const featuredSeedEnabled = catalogMode === CATALOG_MODE_FULL;
  const getFeaturedSeedButton = () => byId("btn-preset-featured-seed-quick-recent");
  let runScreen = null;
  let editorScreen = null;
  let lastEditorReadinessModel = buildStudioEditorReadinessModel({
    sourceText: "",
    canonDiagCode: "",
    canonDiagMessage: "",
    autofixAvailable: false,
  });

  const buildCurrentProjectLocalPackagePayload = ({ projectName = "", deploymentProfile = null } = {}) => {
    const lesson = appState.currentLesson && typeof appState.currentLesson === "object"
      ? appState.currentLesson
      : {};
    const editorDdnText = appState.currentScreen === "editor" && typeof editorScreen?.getDdn === "function"
      ? String(editorScreen.getDdn() ?? "")
      : "";
    const ddnText = editorDdnText.trim() ? editorDdnText : String(lesson.ddnText ?? "");
    const editorLessonPlanText = appState.currentScreen === "editor" && typeof editorScreen?.getLessonPlanText === "function"
      ? String(editorScreen.getLessonPlanText() ?? "")
      : "";
    const lessonPlanText = editorLessonPlanText.trim() ? editorLessonPlanText : String(lesson.lessonPlanText ?? "");
    const title = String(projectName ?? "").trim() || String(lesson.title ?? "").trim() || "이름 없는 셈그림";
    const lessonRow = {
        lesson_id: String(lesson.id ?? "custom").trim() || "custom",
        title,
        description: String(lesson.description ?? "").trim(),
        grade: String(lesson.grade ?? "").trim(),
        subject: String(lesson.subject ?? "").trim(),
        required_views: normalizeViewFamilyList(lesson.requiredViews ?? lesson.required_views ?? []),
        goals: Array.isArray(lesson.goals) ? lesson.goals : [],
        missions: Array.isArray(lesson.missions) ? lesson.missions : [],
        curriculum_card: lesson.curriculumCard && typeof lesson.curriculumCard === "object"
          ? lesson.curriculumCard
          : null,
        path: `lessons/${String(lesson.id ?? "custom").trim() || "custom"}.ddn`,
        source_text: ddnText,
      };
    const reports = lessonPlanText
        ? [{
          report_id: "lesson_plan_v0",
          title: "lesson_plan.v0",
          path: "reports/lesson_plan.v0.toml",
          mime: "text/plain; charset=utf-8",
          text: lessonPlanText,
        }]
        : [];
    const manifest = buildStudioLocalPackageManifest({
      packageId: `classroom.${String(lesson.id ?? "custom").trim() || "custom"}.lesson_bundle.v0`,
      title,
      version: "0.1.0",
      lessons: [lessonRow],
      reports,
      deploymentProfile,
    });
    return buildStudioLocalPackagePayload({
      manifest,
      lessons: [lessonRow],
      reports,
      assets: [],
    });
  };

  const sha256HexUtf8 = async (text = "") => {
    const subtle = globalThis?.crypto?.subtle;
    if (!subtle || typeof subtle.digest !== "function") {
      throw new Error("SHA-256 digest를 사용할 수 없습니다.");
    }
    const digest = await subtle.digest("SHA-256", new TextEncoder().encode(String(text ?? "")));
    return Array.from(new Uint8Array(digest))
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join("");
  };

  const buildProjectRunLineageV0 = (summary = {}, {
    parentRunId = "",
    branchMadi = null,
    branchStateHash = "",
    createdRevision = "",
  } = {}) => {
    const request = summary.execution_preparation?.runtime_request && typeof summary.execution_preparation.runtime_request === "object"
      ? summary.execution_preparation.runtime_request
      : {
        frontdoor: String(summary.frontdoor ?? "wasm_project_run_summary_json"),
        requested_ticks: Number(summary.runtime_execution_envelope?.requested_ticks ?? 1),
        target: String(summary.target ?? "앱"),
        target_path: String(summary.target_path ?? "app/main.ddn"),
      };
    const runRequestHash = `candidate:fnv1a32:project_run_request:${fnv1a32Text(stableObjectString(request))}`;
    const lineage = {
      schema: "seamgrim.local_project_run_lineage.v0",
      project_graph_hash: String(summary.project_graph_sha256 ?? ""),
      lock_hash: String(summary.lock_sha256 ?? ""),
      linked_source_hash: String(
        summary.linked_execution_source_sha256
          ?? summary.execution_preparation?.linked_source?.source_sha256
          ?? "",
      ),
      run_request_hash: runRequestHash,
      parent_run_id: String(parentRunId ?? ""),
      branch_madi: branchMadi === null || branchMadi === undefined ? null : Number(branchMadi),
      branch_state_hash: String(branchStateHash ?? ""),
      final_state_hash: String(summary.runtime_state_hash ?? ""),
      view_artifact_hash: "",
      created_revision: String(createdRevision ?? "").trim()
        || `local:${String(summary.summary_sha256 ?? summary.bounded_state_sha256 ?? "run").slice(0, 12)}`,
    };
    return {
      ...lineage,
      lineage_hash: `candidate:fnv1a32:project_run_lineage:${fnv1a32Text(stableObjectString(lineage))}`,
    };
  };

  const preservedProjectRunLineage = (summary = {}, loadedPayload = null) => {
    const lineage = loadedPayload?.run_lineage && typeof loadedPayload.run_lineage === "object"
      ? loadedPayload.run_lineage
      : null;
    if (!lineage) return null;
    const sameProjectGraph = String(loadedPayload.project_graph_sha256 ?? "") === String(summary.project_graph_sha256 ?? "");
    const sameRuntimeState = String(loadedPayload.runtime_state_hash ?? "") === String(summary.runtime_state_hash ?? "");
    const sameSummary = String(loadedPayload.summary_sha256 ?? "") === String(summary.summary_sha256 ?? "");
    const sameBounded = String(loadedPayload.bounded_state_sha256 ?? "") === String(summary.bounded_state_sha256 ?? "");
    if (!sameProjectGraph || !sameRuntimeState || !sameSummary || !sameBounded) return null;
    return JSON.parse(JSON.stringify(lineage));
  };

  const normalizeProjectSourceWithWasm = async (projectSource) => {
    const source = projectSource && typeof projectSource === "object" ? projectSource : {};
    const client = await appState.wasm.loader.ensure("1 보여주기.");
    const wasmModule = client?.wasmModule;
    if (!wasmModule || typeof wasmModule.wasm_normalize_project_source_json !== "function") {
      throw new Error("wasm_normalize_project_source_json export 누락");
    }
    const raw = wasmModule.wasm_normalize_project_source_json(JSON.stringify(source));
    return JSON.parse(String(raw ?? "{}"));
  };

  const buildCurrentProjectSnapshotLocalPackagePayload = async ({ projectName = "" } = {}) => {
    if (
      appState.currentScreen !== "editor" ||
      typeof editorScreen?.isProjectModeActive !== "function" ||
      typeof editorScreen?.buildVirtualProjectSource !== "function" ||
      !editorScreen.isProjectModeActive()
    ) {
      return buildCurrentProjectLocalPackagePayload({ projectName });
    }
    const source = editorScreen.buildVirtualProjectSource();
    const normalized = await normalizeProjectSourceWithWasm(source);
    const summary = await runLocalProjectSummaryFromWasm(source, { target: "앱" });
    const loadedPayload = typeof editorScreen?.getLoadedProjectPayload === "function"
      ? editorScreen.getLoadedProjectPayload()
      : null;
    const sourceFilesByPath = new Map(
      (Array.isArray(source.files) ? source.files : [])
        .map((file) => [String(file?.path ?? "").trim(), file]),
    );
    const files = [];
    for (const file of Array.isArray(normalized.files) ? normalized.files : []) {
      const pathValue = String(file?.path ?? "").trim();
      const sourceFile = sourceFilesByPath.get(pathValue) ?? {};
      const text = String(sourceFile?.utf8_text ?? sourceFile?.text ?? "");
      files.push({
        path: pathValue,
        role: String(file?.role ?? "ddn_source").trim() || "ddn_source",
        utf8_text: text,
        byte_length: Number(file?.byte_length ?? new TextEncoder().encode(text).length),
        sha256: String(file?.sha256 ?? "") || await sha256HexUtf8(text),
      });
    }
    return {
      schema: "seamgrim.local_project_tree_payload.v0",
      kind: "seamgrim.local_project_tree.v0",
      root_path: String(source.root_path ?? normalized.root_path ?? "."),
      entry_path: String(source.entry_path ?? normalized.entry_path ?? "app/main.ddn"),
      files,
      project_graph_sha256: String(summary.project_graph_sha256 ?? normalized.project_graph_sha256 ?? ""),
      lock_sha256: String(summary.lock_sha256 ?? normalized.lock_sha256 ?? ""),
      canonical_manifest_sha256: String(summary.canonical_manifest_sha256 ?? normalized.canonical_manifest_sha256 ?? ""),
      symbol_boundary_sha256: String(summary.symbol_boundary_sha256 ?? ""),
      bounded_state_sha256: String(summary.bounded_state_sha256 ?? ""),
      runtime_state_hash: String(summary.runtime_state_hash ?? ""),
      summary_sha256: String(summary.summary_sha256 ?? ""),
      output_rows: Array.isArray(summary.output_rows) ? summary.output_rows.map((row) => String(row ?? "")) : [],
      project_observation_trace: summary.project_observation_trace && typeof summary.project_observation_trace === "object"
        ? summary.project_observation_trace
        : null,
      execution_preparation: summary.execution_preparation && typeof summary.execution_preparation === "object"
        ? summary.execution_preparation
        : null,
      run_lineage: preservedProjectRunLineage(summary, loadedPayload) ?? buildProjectRunLineageV0(summary, {
        createdRevision: appState.shell.currentRevisionId || "",
      }),
      limits: summary.limits && typeof summary.limits === "object" ? summary.limits : {},
      frontdoor: "wasm_project_run_summary_json",
      state_hash_formula_change: false,
      project_name: String(projectName ?? "").trim() || String(appState.currentLesson?.title ?? "로컬 프로젝트").trim() || "로컬 프로젝트",
    };
  };

  const buildCurrentProjectSnapshotArtifact = async ({ projectName = "" } = {}) => {
    const localPackagePayload = await buildCurrentProjectSnapshotLocalPackagePayload({ projectName });
    const curriculumInquiryArtifact =
      runScreen && typeof runScreen.exportCurriculumInquiryArtifact === "function"
        ? runScreen.exportCurriculumInquiryArtifact()
        : appState.currentLesson?.curriculumInquiryArtifact ?? null;
    const curriculumArtifactBundle =
      runScreen && typeof runScreen.exportCurriculumArtifactBundle === "function"
        ? runScreen.exportCurriculumArtifactBundle()
        : appState.currentLesson?.curriculumArtifactBundle ?? null;
    const curriculumModelComparison =
      runScreen && typeof runScreen.exportCurriculumModelComparison === "function"
        ? runScreen.exportCurriculumModelComparison()
        : appState.currentLesson?.curriculumModelComparison ?? null;
    const curriculumSemanticScene =
      runScreen && typeof runScreen.exportCurriculumSemanticScene === "function"
        ? runScreen.exportCurriculumSemanticScene()
        : appState.currentLesson?.curriculumSemanticScene ?? null;
    const input = buildProjectSnapshotInputFromEditorState({
      editorScreen,
      appState,
      projectName,
      localPackagePayload,
      curriculumInquiryArtifact,
      curriculumArtifactBundle,
      curriculumModelComparison,
      curriculumSemanticScene,
      lineage: appState.shell.currentProjectLineage ?? null,
    });
    return buildProjectSnapshotArtifactV0(input);
  };

  try {
    window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__ = buildCurrentProjectSnapshotArtifact;
    window.__SEAMGRIM_PROJECT_API__ = {
      buildProjectCapabilityFragment,
      readProjectCapabilitySession,
      requestProjectApi,
    };
  } catch (_) {
    // Non-browser imports can ignore the product helper export.
  }
  const initialProjectCapability = captureProjectCapabilityFromLocation();
  if (initialProjectCapability?.project_id) {
    appState.shell.currentProjectId = initialProjectCapability.project_id;
    appState.shell.shareMode = initialProjectCapability.role;
  }

  const projectLinkForCapability = ({ projectId = "", role = "edit", capability = "" } = {}) => {
    const url = new URL(window.location.href);
    url.search = "";
    url.hash = buildProjectCapabilityFragment({ projectId, role, capability }).slice(1);
    return url.href;
  };

  const escapeProjectText = (value = "") => String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  const escapeProjectAttribute = (value = "") => escapeProjectText(value).replace(/"/g, "&quot;");

  const readRecentProjects = () => {
    try {
      const parsed = JSON.parse(window.localStorage?.getItem?.(PROJECT_RECENT_STORAGE_KEY) ?? "[]");
      return Array.isArray(parsed) ? parsed.filter((item) => item && typeof item === "object") : [];
    } catch (_) {
      return [];
    }
  };

  const writeRecentProjects = (rows = []) => {
    try {
      window.localStorage?.setItem?.(PROJECT_RECENT_STORAGE_KEY, JSON.stringify(rows.slice(0, 12)));
    } catch (_) {
      // Local recent work is convenience state only.
    }
  };

  const rememberRecentProject = (row = {}) => {
    const projectId = String(row.project_id ?? "").trim();
    if (!projectId) return;
    const next = {
      project_id: projectId,
      title: String(row.title ?? "").trim() || "이름 없는 셈그림",
      head_revision_id: String(row.head_revision_id ?? "").trim(),
      content_sha256: String(row.content_sha256 ?? "").trim(),
      edit_capability: String(row.edit_capability ?? "").trim(),
      view_capability: String(row.view_capability ?? "").trim(),
      edit_link: String(row.edit_link ?? "").trim(),
      view_link: String(row.view_link ?? "").trim(),
      updated_at_ms: Date.now(),
    };
    const rows = readRecentProjects().filter((item) => String(item.project_id ?? "").trim() !== projectId);
    writeRecentProjects([next, ...rows]);
  };

  const setProjectSaveStatus = (state = "idle", message = "") => {
    const node = byId("project-save-status");
    if (!node) return;
    node.dataset.state = String(state ?? "idle").trim() || "idle";
    node.textContent = String(message ?? "").trim() || "서버 저장 대기";
  };

  const renderRecentProjects = () => {
    const list = byId("project-recent-list");
    if (!list) return;
    const rows = readRecentProjects();
    if (!rows.length) {
      list.textContent = "저장된 서버 작업 없음";
      return;
    }
    list.innerHTML = rows
      .map((row) => {
        const title = String(row.title ?? row.project_id ?? "작업").trim();
        const link = String(row.edit_link ?? "#").replace(/"/g, "&quot;");
        const revision = String(row.head_revision_id ?? "").trim();
        return `<a class="project-recent-item" href="${link}" data-project-recent-item data-project-id="${String(row.project_id ?? "").replace(/"/g, "&quot;")}"><span>${title}</span><small>${revision || "-"}</small></a>`;
      })
      .join("");
  };

  const updateProjectEditLink = ({ projectId = "", editCapability = "", title = "" } = {}) => {
    const link = byId("project-edit-link");
    if (!link) return "";
    if (!projectId || !editCapability) {
      link.href = "#";
      link.textContent = "편집 링크 없음";
      link.setAttribute("aria-disabled", "true");
      return "";
    }
    const href = projectLinkForCapability({ projectId, role: "edit", capability: editCapability });
    link.href = href;
    link.textContent = title ? `${title} 편집 링크` : "편집 링크";
    link.removeAttribute("aria-disabled");
    return href;
  };

  const updateProjectViewLink = ({ projectId = "", viewCapability = "", title = "" } = {}) => {
    const link = byId("project-view-link");
    if (!link) return "";
    if (!projectId || !viewCapability) {
      link.href = "#";
      link.textContent = "보기 링크 없음";
      link.setAttribute("aria-disabled", "true");
      return "";
    }
    const href = projectLinkForCapability({ projectId, role: "view", capability: viewCapability });
    link.href = href;
    link.textContent = title ? `${title} 보기 링크` : "보기 링크";
    link.removeAttribute("aria-disabled");
    return href;
  };

  const updateProjectRoleControls = (role = "edit") => {
    const normalizedRole = role === "view" ? "view" : "edit";
    const session = readProjectCapabilitySession();
    const saveButton = byId("btn-project-save");
    if (saveButton) {
      saveButton.disabled = normalizedRole === "view";
      saveButton.title = normalizedRole === "view" ? "보기 링크에서는 원본을 저장할 수 없습니다." : "";
    }
    const cloneButton = byId("btn-project-clone");
    if (cloneButton) {
      cloneButton.disabled = !session?.project_id || !session?.capability;
      cloneButton.textContent = normalizedRole === "view" ? "내 버전 만들기" : "복제본 만들기";
      cloneButton.title = session?.project_id ? "현재 프로젝트를 새 프로젝트로 복제합니다." : "저장된 프로젝트를 먼저 여세요.";
    }
  };

  const setProjectConflictPanel = ({
    visible = false,
    message = "",
    serverRevision = "",
    serverContentSha = "",
    localRevision = "",
    localPackagePayload = null,
  } = {}) => {
    const panel = byId("project-conflict-panel");
    if (!panel) return;
    panel.classList.toggle("hidden", !visible);
    byId("project-conflict-message").textContent = String(message || "다른 창에서 먼저 저장되었습니다.");
    byId("project-conflict-server").textContent = serverRevision
      ? `${serverRevision} · ${String(serverContentSha ?? "").slice(0, 16)}`
      : "-";
    byId("project-conflict-local").textContent = localRevision || appState.shell.currentRevisionId || "-";
    const text = localPackagePayload ? JSON.stringify(localPackagePayload, null, 2) : "";
    const textarea = byId("project-conflict-local-package");
    if (textarea) textarea.value = visible ? text : "";
    try {
      window.__SEAMGRIM_LAST_CONFLICT_LOCAL_PACKAGE__ = localPackagePayload ?? null;
    } catch (_) {
      // Browser instrumentation is optional.
    }
  };

  const renderProjectRevisions = (rows = []) => {
    const list = byId("project-revision-list");
    if (!list) return;
    if (!rows.length) {
      list.textContent = "저장 후 리비전을 볼 수 있습니다.";
      return;
    }
    list.innerHTML = rows
      .map((row, index) => {
        const revisionId = String(row.revision_id ?? "").trim();
        const parentId = String(row.parent_revision_id ?? "").trim();
        const contentSha = String(row.content_sha256 ?? "").trim();
        const packageHash = String(row.package_hash ?? "").trim();
        const restoreOf = String(row.restore_of_revision_id ?? "").trim();
        const isHead = revisionId === String(appState.shell.currentRevisionId ?? "").trim();
        return `<div class="project-revision-item" data-project-revision-id="${escapeProjectAttribute(revisionId)}">
          <div class="project-revision-meta">
            <strong>${isHead ? "현재판" : `이전판 ${index + 1}`}</strong>
            <code>revision: ${escapeProjectText(revisionId || "-")}</code>
            <code>parent: ${escapeProjectText(parentId || "-")}</code>
            <code>content: ${escapeProjectText(contentSha || "-")}</code>
            <code>package: ${escapeProjectText(packageHash || "-")}</code>
            ${restoreOf ? `<code>restore_of: ${escapeProjectText(restoreOf)}</code>` : ""}
          </div>
          <button class="ghost" type="button" data-project-restore-revision="${escapeProjectAttribute(revisionId)}"${isHead ? " disabled" : ""}>복원</button>
        </div>`;
      })
      .join("");
  };

  const loadProjectRevisions = async () => {
    const session = readProjectCapabilitySession();
    if (!session?.project_id || !session?.capability) {
      renderProjectRevisions([]);
      return null;
    }
    try {
      const payload = await requestProjectApi(`${PROJECT_API_BASE}/${session.project_id}/revisions`, {
        method: "GET",
        capability: session.capability,
      });
      const rows = Array.isArray(payload.revisions) ? payload.revisions : [];
      renderProjectRevisions(rows);
      return rows;
    } catch (error) {
      const list = byId("project-revision-list");
      if (list) list.textContent = String(error?.message ?? "리비전을 불러오지 못했습니다.");
      return null;
    }
  };

  const localProjectTreePayloadFromSnapshot = (snapshot = {}) => {
    const payload = snapshot?.local_package_payload;
    if (
      payload &&
      typeof payload === "object" &&
      payload.kind === "seamgrim.local_project_tree.v0" &&
      Array.isArray(payload.files) &&
      payload.files.length > 0
    ) {
      return payload;
    }
    return null;
  };

  const loadSnapshotIntoEditor = (
    snapshot = {},
    {
      title = "저장된 셈그림",
      readOnly = false,
      lessonPlanText = "",
      lessonPlan = null,
      lessonId = "saved_project",
      lessonChoices = null,
    } = {},
  ) => {
    const projectTreePayload = localProjectTreePayloadFromSnapshot(snapshot);
    if (projectTreePayload && typeof editorScreen?.loadVirtualProjectSource === "function") {
      return editorScreen.loadVirtualProjectSource(projectTreePayload, {
        title,
        readOnly,
        lessonPlanText,
        lessonPlan,
        lessonId,
        lessonChoices,
      });
    }
    editorScreen.loadLesson(String(snapshot.ddn_text ?? ""), {
      title,
      readOnly,
      lessonPlanText,
      lessonPlan,
      lessonId,
      lessonChoices,
    });
    return false;
  };

  const restoreProjectRevision = async (revisionId = "") => {
    const session = readProjectCapabilitySession();
    const sourceRevisionId = String(revisionId ?? "").trim();
    if (!session?.project_id || !session?.capability || session.role !== "edit" || !sourceRevisionId) {
      setProjectSaveStatus("error", "복원할 편집 권한 또는 리비전이 없습니다.");
      return null;
    }
    try {
      setProjectSaveStatus("saving", "이전판을 새 리비전으로 복원하는 중...");
      const response = await requestProjectApi(`${PROJECT_API_BASE}/${session.project_id}/restore`, {
        method: "POST",
        capability: session.capability,
        body: {
          source_revision_id: sourceRevisionId,
          expected_parent_revision_id: appState.shell.currentRevisionId,
        },
      });
      const snapshot = response.project_snapshot && typeof response.project_snapshot === "object"
        ? response.project_snapshot
        : {};
      const title = String(snapshot.project_name ?? byId("project-name-input")?.value ?? "저장된 셈그림").trim() || "저장된 셈그림";
      appState.shell.currentRevisionId = String(response.revision_id ?? response.head_revision_id ?? "");
      appState.shell.currentProjectLineage = snapshot.lineage ?? appState.shell.currentProjectLineage ?? null;
      appState.studio.localSaveStatus = `복원됨 ${appState.shell.currentRevisionId || ""}`.trim();
      const restoredCatalogLesson = appState.lessonsById.get(String(snapshot.source_lesson_id ?? appState.currentLesson?.id ?? "restored_project").trim()) ?? {};
      appState.currentLesson = {
        id: String(snapshot.source_lesson_id ?? appState.currentLesson?.id ?? "restored_project").trim() || "restored_project",
        title,
        description: "복원된 서버 프로젝트",
        subject: String(restoredCatalogLesson.subject ?? ""),
        grade: String(restoredCatalogLesson.grade ?? ""),
        quality: "saved_project_restore",
        requiredViews: restoredCatalogLesson.requiredViews ?? [],
        curriculumCard: restoredCatalogLesson.curriculumCard ?? null,
        curriculumMeta: restoredCatalogLesson.curriculumMeta ?? null,
        ddnText: String(snapshot.ddn_text ?? ""),
        lessonPlanText: String(snapshot.lesson_plan_text ?? ""),
        curriculumInquiryArtifact: snapshot.curriculum_inquiry_artifact ?? null,
        curriculumArtifactBundle: snapshot.curriculum_artifact_bundle ?? null,
        curriculumModelComparison: snapshot.curriculum_model_comparison ?? null,
        curriculumSemanticScene: snapshot.curriculum_semantic_scene ?? null,
      };
      appState.lessonsById.set(appState.currentLesson.id, appState.currentLesson);
      loadSnapshotIntoEditor(snapshot, {
        title,
        readOnly: false,
        lessonPlanText: appState.currentLesson.lessonPlanText,
        lessonId: appState.currentLesson.id,
        lessonChoices: getFoundation12LessonChoices(),
      });
      const nameInput = byId("project-name-input");
      if (nameInput) nameInput.value = title;
      updateShellStatusRail();
      setProjectSaveStatus("saved", `복원됨 · ${appState.shell.currentRevisionId || sourceRevisionId}`);
      await loadProjectRevisions();
      return response;
    } catch (error) {
      const state = Number(error?.status) === 409 ? "conflict" : "error";
      setProjectSaveStatus(state, String(error?.message ?? "리비전을 복원하지 못했습니다."));
      return null;
    }
  };

  const persistProjectCapabilitySession = ({ projectId = "", role = "edit", capability = "" } = {}) => {
    try {
      window.sessionStorage?.setItem?.(
        PROJECT_CAPABILITY_SESSION_KEY,
        JSON.stringify({ project_id: projectId, role, capability }),
      );
    } catch (_) {
      // Session capability is page-local convenience state.
    }
  };

  const saveProjectFromEditor = async () => {
    const nameInput = byId("project-name-input");
    const saveButton = byId("btn-project-save");
    const projectName = String(nameInput?.value ?? "").trim() || "이름 없는 셈그림";
    const session = readProjectCapabilitySession();
    if (session?.role === "view") {
      setProjectSaveStatus("error", "보기 링크에서는 원본을 저장할 수 없습니다. 내 버전 만들기를 사용하세요.");
      updateProjectRoleControls("view");
      return null;
    }
    let artifact = null;
    try {
      if (saveButton) saveButton.disabled = true;
      setProjectSaveStatus("saving", "서버에 저장하는 중...");
      artifact = await buildCurrentProjectSnapshotArtifact({ projectName });
      let response = null;
      if (session?.project_id && session.role === "edit" && appState.shell.currentRevisionId) {
        response = await requestProjectApi(`${PROJECT_API_BASE}/${session.project_id}/revisions`, {
          method: "POST",
          capability: session.capability,
          body: {
            expected_parent_revision_id: appState.shell.currentRevisionId,
            project_snapshot_artifact: artifact,
          },
        });
      } else {
        response = await requestProjectApi(PROJECT_API_BASE, {
          method: "POST",
          body: {
            title: projectName,
            project_snapshot_artifact: artifact,
          },
        });
        persistProjectCapabilitySession({
          projectId: response.project_id,
          role: "edit",
          capability: response.edit_capability,
        });
      }
      const projectId = String(response.project_id ?? session?.project_id ?? "").trim();
      const revisionId = String(response.revision_id ?? response.head_revision_id ?? "").trim();
      const editCapability = String(response.edit_capability ?? session?.capability ?? "").trim();
      const viewCapability = String(response.view_capability ?? appState.shell.currentViewCapability ?? "").trim();
      appState.shell.currentProjectId = projectId;
      appState.shell.currentRevisionId = revisionId;
      appState.shell.currentViewCapability = viewCapability;
      appState.shell.currentProjectLineage = response.project_snapshot?.lineage ?? artifact.project_snapshot?.lineage ?? null;
      appState.studio.localSaveStatus = revisionId ? `서버 저장됨 ${revisionId}` : "서버 저장됨";
      appState.studio.sourceLabel = projectName;
      const editLink = updateProjectEditLink({ projectId, editCapability, title: projectName });
      const viewLink = updateProjectViewLink({ projectId, viewCapability, title: projectName });
      rememberRecentProject({
        project_id: projectId,
        title: projectName,
        head_revision_id: revisionId,
        content_sha256: response.content_sha256,
        edit_capability: editCapability,
        view_capability: viewCapability,
        edit_link: editLink,
        view_link: viewLink,
      });
      renderRecentProjects();
      updateShellStatusRail();
      setProjectSaveStatus("saved", `저장됨 · ${revisionId || projectId} · ${String(response.content_sha256 ?? "").slice(0, 12)}`);
      setProjectConflictPanel({ visible: false });
      updateProjectRoleControls("edit");
      await loadProjectRevisions();
      return response;
    } catch (error) {
      const state = Number(error?.status) === 409 ? "conflict" : "error";
      setProjectSaveStatus(state, String(error?.message ?? "서버 저장에 실패했습니다."));
      if (state === "conflict" && session?.project_id && session?.capability) {
        let serverProject = null;
        try {
          serverProject = await requestProjectApi(`${PROJECT_API_BASE}/${session.project_id}`, {
            method: "GET",
            capability: session.capability,
          });
        } catch (_) {
          serverProject = null;
        }
        setProjectConflictPanel({
          visible: true,
          message: String(error?.message ?? "다른 창에서 먼저 저장되었습니다."),
          serverRevision: serverProject?.head_revision_id ?? "",
          serverContentSha: serverProject?.content_sha256 ?? "",
          localRevision: appState.shell.currentRevisionId ?? "",
          localPackagePayload: artifact?.project_snapshot?.local_package_payload ?? null,
        });
        await loadProjectRevisions();
      }
      return null;
    } finally {
      updateProjectRoleControls(readProjectCapabilitySession()?.role ?? "edit");
    }
  };

  const openProjectFromCapabilitySession = async () => {
    const session = readProjectCapabilitySession();
    if (!session?.project_id || !session?.capability) return false;
    try {
      setProjectSaveStatus("saving", "저장된 프로젝트를 여는 중...");
      const payload = await requestProjectApi(`${PROJECT_API_BASE}/${session.project_id}`, {
        method: "GET",
        capability: session.capability,
      });
      const snapshot = payload.project_snapshot && typeof payload.project_snapshot === "object"
        ? payload.project_snapshot
        : {};
      const title = String(snapshot.project_name ?? "저장된 셈그림").trim() || "저장된 셈그림";
      const ddnText = String(snapshot.ddn_text ?? "");
      const lessonPlanText = String(snapshot.lesson_plan_text ?? "");
      const lessonId = String(snapshot.source_lesson_id ?? "saved_project").trim() || "saved_project";
      const savedCatalogLesson = appState.lessonsById.get(lessonId) ?? {};
      appState.currentLesson = {
        id: lessonId,
        title,
        description: "서버 저장 프로젝트",
        subject: String(savedCatalogLesson.subject ?? ""),
        grade: String(savedCatalogLesson.grade ?? ""),
        quality: "saved_project",
        requiredViews: savedCatalogLesson.requiredViews ?? [],
        curriculumCard: savedCatalogLesson.curriculumCard ?? null,
        curriculumMeta: savedCatalogLesson.curriculumMeta ?? null,
        ddnText,
        lessonPlanText,
        curriculumInquiryArtifact: snapshot.curriculum_inquiry_artifact ?? null,
        curriculumArtifactBundle: snapshot.curriculum_artifact_bundle ?? null,
        curriculumModelComparison: snapshot.curriculum_model_comparison ?? null,
        curriculumSemanticScene: snapshot.curriculum_semantic_scene ?? null,
      };
      appState.lessonsById.set(lessonId, appState.currentLesson);
      appState.shell.currentProjectId = String(payload.project_id ?? session.project_id);
      appState.shell.currentRevisionId = String(payload.head_revision_id ?? "");
      appState.shell.shareMode = String(payload.capability_role ?? session.role);
      appState.shell.currentProjectLineage = snapshot.lineage ?? null;
      const recent = readRecentProjects().find((item) => String(item.project_id ?? "").trim() === session.project_id);
      const viewCapability = session.role === "view"
        ? session.capability
        : String(recent?.view_capability ?? appState.shell.currentViewCapability ?? "").trim();
      appState.shell.currentViewCapability = viewCapability;
      appState.studio.sourceLabel = title;
      appState.studio.localSaveStatus = `서버에서 열림 ${appState.shell.currentRevisionId || ""}`.trim();
      loadSnapshotIntoEditor(snapshot, {
        title,
        readOnly: session.role !== "edit",
        lessonPlanText,
        lessonId,
        lessonChoices: getFoundation12LessonChoices(),
      });
      const nameInput = byId("project-name-input");
      if (nameInput) nameInput.value = title;
      if (session.role === "edit") {
        const editLink = updateProjectEditLink({ projectId: session.project_id, editCapability: session.capability, title });
        const viewLink = updateProjectViewLink({ projectId: session.project_id, viewCapability, title });
        rememberRecentProject({
          project_id: session.project_id,
          title,
          head_revision_id: appState.shell.currentRevisionId,
          content_sha256: payload.content_sha256,
          edit_capability: session.capability,
          view_capability: viewCapability,
          edit_link: editLink,
          view_link: viewLink,
        });
      } else {
        updateProjectEditLink({});
        updateProjectViewLink({ projectId: session.project_id, viewCapability: session.capability, title });
      }
      renderRecentProjects();
      updateShellStatusRail();
      setScreen("editor", { shellSpace: "create" });
      setProjectSaveStatus("saved", `다시 열림 · ${appState.shell.currentRevisionId || session.project_id}`);
      updateProjectRoleControls(session.role);
      await loadProjectRevisions();
      return true;
    } catch (error) {
      setProjectSaveStatus("error", String(error?.message ?? "저장된 프로젝트를 열지 못했습니다."));
      return false;
    }
  };

  const cloneProjectFromSession = async () => {
    const session = readProjectCapabilitySession();
    const cloneButton = byId("btn-project-clone");
    if (!session?.project_id || !session?.capability) {
      setProjectSaveStatus("error", "복제할 저장 프로젝트가 없습니다.");
      return null;
    }
    try {
      if (cloneButton) cloneButton.disabled = true;
      setProjectSaveStatus("saving", "내 버전을 만드는 중...");
      const baseTitle = String(byId("project-name-input")?.value ?? "저장된 셈그림").trim() || "저장된 셈그림";
      const response = await requestProjectApi(`${PROJECT_API_BASE}/${session.project_id}/clone`, {
        method: "POST",
        capability: session.capability,
        body: { title: `${baseTitle} 복제본` },
      });
      persistProjectCapabilitySession({
        projectId: response.project_id,
        role: "edit",
        capability: response.edit_capability,
      });
      const snapshot = response.project_snapshot && typeof response.project_snapshot === "object"
        ? response.project_snapshot
        : {};
      const title = String(snapshot.project_name ?? `${baseTitle} 복제본`).trim() || `${baseTitle} 복제본`;
      const ddnText = String(snapshot.ddn_text ?? "");
      const lessonPlanText = String(snapshot.lesson_plan_text ?? "");
      const lessonId = String(snapshot.source_lesson_id ?? "cloned_project").trim() || "cloned_project";
      const clonedCatalogLesson = appState.lessonsById.get(lessonId) ?? {};
      appState.currentLesson = {
        id: lessonId,
        title,
        description: "복제된 서버 프로젝트",
        subject: String(clonedCatalogLesson.subject ?? ""),
        grade: String(clonedCatalogLesson.grade ?? ""),
        quality: "saved_project_clone",
        requiredViews: clonedCatalogLesson.requiredViews ?? [],
        curriculumCard: clonedCatalogLesson.curriculumCard ?? null,
        curriculumMeta: clonedCatalogLesson.curriculumMeta ?? null,
        ddnText,
        lessonPlanText,
        curriculumInquiryArtifact: snapshot.curriculum_inquiry_artifact ?? null,
        curriculumArtifactBundle: snapshot.curriculum_artifact_bundle ?? null,
        curriculumModelComparison: snapshot.curriculum_model_comparison ?? null,
        curriculumSemanticScene: snapshot.curriculum_semantic_scene ?? null,
      };
      appState.lessonsById.set(lessonId, appState.currentLesson);
      appState.shell.currentProjectId = String(response.project_id ?? "");
      appState.shell.currentRevisionId = String(response.revision_id ?? response.head_revision_id ?? "");
      appState.shell.currentViewCapability = String(response.view_capability ?? "");
      appState.shell.currentProjectLineage = snapshot.lineage ?? null;
      appState.shell.shareMode = "edit";
      appState.studio.sourceLabel = title;
      appState.studio.localSaveStatus = `내 버전 생성됨 ${appState.shell.currentRevisionId || ""}`.trim();
      loadSnapshotIntoEditor(snapshot, {
        title,
        readOnly: false,
        lessonPlanText,
        lessonId,
        lessonChoices: getFoundation12LessonChoices(),
      });
      const nameInput = byId("project-name-input");
      if (nameInput) nameInput.value = title;
      const editLink = updateProjectEditLink({ projectId: response.project_id, editCapability: response.edit_capability, title });
      const viewLink = updateProjectViewLink({ projectId: response.project_id, viewCapability: response.view_capability, title });
      rememberRecentProject({
        project_id: response.project_id,
        title,
        head_revision_id: appState.shell.currentRevisionId,
        content_sha256: response.content_sha256,
        edit_capability: response.edit_capability,
        view_capability: response.view_capability,
        edit_link: editLink,
        view_link: viewLink,
      });
      renderRecentProjects();
      updateShellStatusRail();
      setScreen("editor", { shellSpace: "create" });
      setProjectSaveStatus("saved", `내 버전 생성됨 · ${appState.shell.currentRevisionId || response.project_id}`);
      updateProjectRoleControls("edit");
      await loadProjectRevisions();
      try {
        window.__SEAMGRIM_LAST_PROJECT_CLONE__ = response;
      } catch (_) {
        // Browser instrumentation is optional.
      }
      return response;
    } catch (error) {
      setProjectSaveStatus("error", String(error?.message ?? "내 버전을 만들지 못했습니다."));
      return null;
    } finally {
      updateProjectRoleControls(readProjectCapabilitySession()?.role ?? "edit");
    }
  };

  const emitStudioMetric = (name, payload = {}) => {
    const metricName = String(name ?? "").trim();
    if (!metricName || typeof window === "undefined") return;
    const record = {
      name: metricName,
      ts: Date.now(),
      ...((payload && typeof payload === "object") ? payload : {}),
    };
    try {
      if (!Array.isArray(window.__SEAMGRIM_STUDIO_METRICS__)) {
        window.__SEAMGRIM_STUDIO_METRICS__ = [];
      }
      window.__SEAMGRIM_STUDIO_METRICS__.push(record);
    } catch (_) {
      // ignore metric buffer failures
    }
    try {
      if (typeof window.dispatchEvent === "function" && typeof CustomEvent === "function") {
        window.dispatchEvent(new CustomEvent("seamgrim:studio-metric", { detail: record }));
      }
    } catch (_) {
      // ignore metric event failures
    }
  };

  const setStudioState = (patch = {}) => {
    const next = patch && typeof patch === "object" ? patch : {};
    appState.studio = {
      ...appState.studio,
      ...next,
    };
    updateShellStatusRail();
    return appState.studio;
  };

  const buildEditorFixitPreview = (sourceText) => {
    const source = String(sourceText ?? "");
    if (!hasLegacyAutofixCandidate(source)) return null;
    const rawResult = applyLegacyAutofixToDdn(source);
    const textAfter = String(rawResult?.text ?? source);
    const contract = buildAutofixResultContract(rawResult, { sourceTextAfter: textAfter });
    const appliedRules = Array.isArray(contract.applied_rules) ? contract.applied_rules : [];
    const totalChanges = Number(rawResult?.total_changes ?? appliedRules.length);
    const fixitCount = Math.max(0, Number.isFinite(totalChanges) ? totalChanges : appliedRules.length);
    if (!rawResult?.changed || fixitCount <= 0) return null;
    return {
      schema: "seamgrim.editor_fixit_preview.v1",
      preview_text: textAfter,
      diagnostic_count: appliedRules.length || 1,
      fixit_count: fixitCount,
      applied_rules: appliedRules,
      blocking_remaining: Boolean(contract.blocking_remaining),
      next_action: contract.next_action,
    };
  };

  const updateEditorReadinessModel = (sourceText, { canonDiagCode = "", canonDiagMessage = "" } = {}) => {
    const source = String(sourceText ?? "");
    const model = buildStudioEditorReadinessModel({
      sourceText: source,
      canonDiagCode: String(canonDiagCode ?? "").trim(),
      canonDiagMessage: String(canonDiagMessage ?? "").trim(),
      autofixAvailable: hasLegacyAutofixCandidate(source),
    });
    lastEditorReadinessModel = model;
    editorScreen?.setStudioReadinessModel?.(model);
    editorScreen?.setFixitModel?.(buildEditorFixitPreview(source));
    runScreen?.setStudioReadinessModel?.(model);
    return model;
  };

  const resolveCanonDiagUiModel = (diag = null) => {
    const row = diag && typeof diag === "object" ? diag : {};
    const code = String(row?.code ?? "").trim();
    const rawMessage = String(row?.message ?? row?.detail ?? "").trim();
    const normalized = rawMessage.toLowerCase();
    if (code === "E_WASM_CANON_JSON_PARSE_FAILED") {
      if (normalized.includes("flat canonical")) {
        return {
          blocking: false,
          readinessCode: "",
          readinessMessage: "",
          summaryText: "구성 해석 실패: 실행은 계속할 수 있습니다.",
        };
      }
      if (normalized.includes("maegim canonical")) {
        return {
          blocking: false,
          readinessCode: "",
          readinessMessage: "",
          summaryText: "매김 계획 해석 실패: 실행은 계속할 수 있습니다.",
        };
      }
      return {
        blocking: false,
        readinessCode: "",
        readinessMessage: "",
        summaryText: "구성 해석 실패: 실행은 계속할 수 있습니다.",
      };
    }
    return {
      blocking: Boolean(code),
      readinessCode: code,
      readinessMessage: rawMessage,
      summaryText: code ? `구성 해석 실패 (${code})` : "구성 해석 실패",
    };
  };

  const refreshEditorCanonSummary = async (ddnText) => {
    if (!editorScreen && !runScreen) return;
    const sourceText = String(ddnText ?? "");
    const ticket = editorCanonSummaryTicket + 1;
    editorCanonSummaryTicket = ticket;
    if (!sourceText.trim()) {
      editorScreen?.setCanonFlatView?.(null);
      updateEditorReadinessModel(sourceText);
      return;
    }
    updateEditorReadinessModel(sourceText);
    editorScreen?.setCanonFlatView?.({
      summaryText: "구성: WASM canon 계산 중...",
      topoOrder: [],
      instances: [],
      links: [],
    });
    const flat = await lessonCanonHydrator.deriveFlatJson(sourceText, { quiet: true });
    if (ticket !== editorCanonSummaryTicket) return;
    if (!flat) {
      const canonDiag = Array.isArray(lessonCanonHydrator.getCanonDiags?.())
        ? lessonCanonHydrator.getCanonDiags()
        : [];
      const runtimeDiag = Array.isArray(lessonCanonHydrator.getRuntimeDiags?.())
        ? lessonCanonHydrator.getRuntimeDiags()
        : [];
      const topDiag = canonDiag[0] ?? runtimeDiag[0] ?? null;
      if (topDiag && topDiag.code) {
        const uiDiag = resolveCanonDiagUiModel(topDiag);
        updateEditorReadinessModel(sourceText, {
          canonDiagCode: uiDiag.readinessCode,
          canonDiagMessage: uiDiag.readinessMessage,
        });
        editorScreen?.setCanonFlatView?.({
          summaryText: uiDiag.summaryText,
          topoOrder: [],
          instances: [],
          links: [],
        });
        return;
      }
    }
    editorScreen?.setCanonFlatView?.(buildFlatPlanView(flat));
    updateEditorReadinessModel(sourceText);
  };

  const resolveFeaturedSeedIds = () =>
    featuredSeedEnabled
      ? resolveAvailableFeaturedSeedIds(FEATURED_SEED_IDS, appState.lessonsById)
      : [];

  const updateFeaturedSeedQuickAction = () => {
    const featuredSeedButton = getFeaturedSeedButton();
    if (!featuredSeedButton) return;
    if (!featuredSeedEnabled) {
      featuredSeedButton.disabled = true;
      featuredSeedButton.classList.add("hidden");
      featuredSeedButton.title = "release reps-only 모드에서는 비활성화됩니다.";
      return;
    }
    const ids = resolveFeaturedSeedIds();
    if (!ids.length) {
      featuredSeedButton.disabled = true;
      featuredSeedButton.textContent = "신규 seed 없음";
      featuredSeedButton.title = "seed_manifest에서 신규 seed를 찾지 못했습니다.";
      return;
    }
    const currentCursor = Number.isInteger(appState.quickLaunch?.featuredSeedCursor)
      ? appState.quickLaunch.featuredSeedCursor
      : -1;
    const nextIndex = currentCursor >= 0 && currentCursor < ids.length
      ? (currentCursor + 1) % ids.length
      : 0;
    const nextId = ids[nextIndex];
    const nextTitle = String(appState.lessonsById.get(nextId)?.title ?? nextId);
    featuredSeedButton.disabled = false;
    featuredSeedButton.textContent = `신규 seed 실행 (${ids.length}) · Alt+6`;
    featuredSeedButton.title = `다음 실행 후보: ${nextTitle}`;
  };

  const openRunWithLessonWithSource = (
    lesson,
    {
      launchKind = "manual",
      autoExecute = false,
      sourceId = "",
      sourceType = "lesson",
      derivedFrom = "",
      onboardingProfile = "",
      runRequest = null,
      shellSpace = "lab",
    } = {},
  ) => {
    if (!runScreen) return;
    const sourceKind = String(sourceType ?? "").trim().toLowerCase();
    const routeLessonId = sourceKind === "lesson" ? String(lesson?.id ?? "").trim() : "";
    const sourceLabel = String(
      sourceKind === "lesson"
        ? (lesson?.title ?? lesson?.id ?? "교과")
        : (lesson?.title ?? "새 작업"),
    ).trim() || "새 작업";
    syncMainTabRoute({
      tab: MAIN_TAB_STUDIO,
      lessonId: routeLessonId,
    });
    if (sourceKind === "ddn") {
      trackDdnInputSource({
        sourceId,
        label: String(lesson?.title ?? lesson?.id ?? "사용자 DDN"),
        ddnText: lesson?.ddnText ?? "",
        derivedFrom,
      });
    } else {
      trackLessonInputSource(lesson, { sourceId });
    }
    persistInputRegistrySnapshot();
    setStudioState({
      sourceKind: sourceKind === "lesson" ? "lesson" : "scratch",
      sourceLabel,
      engineStatus: "idle",
      primaryViewFamily: "sim",
      activeSubpanelTab: "graph",
    });
    runScreen.setLessonOptions(Array.from(appState.lessonsById.values()));
    runScreen.loadLesson(lesson, {
      launchKind,
      sourceKind: sourceKind === "lesson" ? "lesson" : "scratch",
      sourceLabel,
    });
    const queuedRunRequest = (runRequest || autoExecute)
      ? runScreen.enqueueRunRequest(runRequest ?? {
        sourceText: lesson?.ddnText ?? "",
        launchKind,
        sourceType: sourceKind === "lesson" ? "lesson" : "ddn",
      })
      : null;
    const normalizedOnboardingProfile = normalizeRunOnboardingProfile(onboardingProfile);
    if (normalizedOnboardingProfile) {
      runScreen.applyRunOnboardingProfile(normalizedOnboardingProfile);
    }
    persistSceneSummarySnapshot(runScreen);
    persistRuntimeSnapshotBundleV0(runScreen);
    setScreen("run", { shellSpace });
    if (String(launchKind ?? "").trim().toLowerCase() === "local_package_import") {
      runScreen.resetRuntimeForExplicitRun?.({ execPathHint: "받은 수업 실행 대기" });
      window.requestAnimationFrame?.(() => {
        document.querySelector("#btn-run")?.focus?.();
      });
    }
    void refreshEditorCanonSummary(lesson?.ddnText ?? "");
    updateFeaturedSeedQuickAction();
    return queuedRunRequest;
  };

  const openRunWithLesson = (lesson, { launchKind = "manual", autoExecute = false, shellSpace = "lab" } = {}) => {
    openRunWithLessonWithSource(lesson, { launchKind, autoExecute, shellSpace });
  };

  const openStudioWithDdnText = async (
    ddnText,
    {
      launchKind = "manual",
      autoExecute = false,
      sourceId = "ddn:studio:manual",
      title = "작업실 DDN",
      description = "작업실에서 직접 작성한 DDN",
    } = {},
  ) => {
    const source = String(ddnText ?? "");
    if (!source.trim()) {
      showPlatformToast("빈 DDN은 실행할 수 없습니다.");
      return false;
    }
    const lesson = await buildStudioDraftLessonFromDdn(source, {
      id: STUDIO_DRAFT_LESSON_ID,
      title,
      description,
    });
    openRunWithLessonWithSource(lesson, {
      launchKind,
      autoExecute,
      sourceId,
      sourceType: "ddn",
      derivedFrom: appState.currentLesson?.id ?? "",
    });
    return true;
  };

  const openStudioWithBlankDraft = async () =>
    openStudioWithDdnText(STUDIO_DRAFT_DDN_TEMPLATE, {
      launchKind: "manual",
      sourceId: "ddn:studio:draft",
      title: "새 작업",
      description: "작업실 기본 초안",
    });

  const switchMainTab = async (
    target,
    {
      lessonId = "",
      ddnText = "",
      launchKind = "manual",
      autoExecute = false,
      onboardingProfile = "",
      shellSpace = "lab",
      fallbackToBlankOnLessonError = true,
    } = {},
  ) => {
    const normalizedTarget = normalizeMainTabTarget(target, MAIN_TAB_BROWSE);
    if (normalizedTarget === MAIN_TAB_BROWSE) {
      setScreen("browse", { shellSpace });
      return true;
    }
    if (normalizedTarget === MAIN_TAB_PUBLISH) {
      setScreen("publish", { shellSpace: "publish" });
      return true;
    }

    const requestedLessonId = String(lessonId ?? "").trim();
    if (requestedLessonId) {
      try {
        const lesson = await loadLessonById(requestedLessonId);
        openRunWithLessonWithSource(lesson, {
          launchKind,
          autoExecute,
          sourceId: `lesson:${lesson.id}`,
          onboardingProfile,
          shellSpace,
        });
        return true;
      } catch (err) {
        showPlatformToast(`작업실 로드 실패: ${String(err?.message ?? err)}`);
        if (!fallbackToBlankOnLessonError) {
          return false;
        }
      }
    }

    const directDdn = String(ddnText ?? "");
    if (directDdn.trim()) {
      const ok = await openStudioWithDdnText(directDdn, {
        launchKind,
        autoExecute,
        sourceId: "ddn:studio:route",
      });
      if (ok) return true;
    }

    return openStudioWithBlankDraft();
  };

  const pickNextFeaturedSeedId = () => {
    if (!featuredSeedEnabled) return "";
    const picked = pickNextFeaturedSeedLaunch({
      featuredSeedIds: FEATURED_SEED_IDS,
      lessonsById: appState.lessonsById,
      currentLessonId: String(appState.currentLesson?.id ?? ""),
      cursor: Number.isInteger(appState.quickLaunch?.featuredSeedCursor)
        ? appState.quickLaunch.featuredSeedCursor
        : -1,
    });
    appState.quickLaunch.featuredSeedCursor = Number.isInteger(picked.nextCursor) ? picked.nextCursor : -1;
    return String(picked.nextId ?? "");
  };

  const runNextFeaturedSeed = async () => {
    if (!featuredSeedEnabled) {
      updateFeaturedSeedQuickAction();
      return false;
    }
    const targetId = pickNextFeaturedSeedId();
    if (!targetId) {
      updateFeaturedSeedQuickAction();
      return false;
    }
    const lesson = await loadLessonById(targetId);
    openRunWithLesson(lesson, { launchKind: "featured_seed_quick", autoExecute: true });
    return true;
  };

  const openLocalPackageFile = async (file) => {
    const fileName = String(file?.name ?? "").trim();
    setLocalPackageImportStatus("loading", `배포 파일을 확인하는 중입니다: ${fileName || "선택한 파일"}`);
    try {
      const text = await file.text();
      let payload = null;
      try {
        payload = JSON.parse(text);
      } catch (_) {
        throw new Error("studio_local_package_invalid_json");
      }
      if (
        payload?.kind === "student_submission"
        || (payload?.schema === "seamgrim.classroom_exchange.v0" && payload?.__종류 !== "studio_local_package_payload")
      ) {
        const validation = validateStudentSubmissionExchangeV0(payload);
        if (!validation.valid) {
          throw new Error(formatStudentSubmissionExchangeError(new Error(validation.error)));
        }
        const snapshot = payload.student_project_snapshot && typeof payload.student_project_snapshot === "object"
          ? payload.student_project_snapshot
          : {};
        const sourceText = String(snapshot.ddn_text ?? "").trim();
        if (!sourceText) {
          throw new Error("학생 제출에 실행할 교과 원문이 없습니다.");
        }
        const lessonId = String(payload.lesson_id ?? snapshot.lesson_id ?? "student_submission_lesson").trim() || "student_submission_lesson";
        const safeLessonId = safeLocalPackageToken(lessonId, "student_submission_lesson");
        const safeSubmissionId = safeLocalPackageToken(payload.exchange_id ?? payload.payload_hash ?? "student_submission", "student_submission");
        const importedLesson = await buildStudioDraftLessonFromDdn(sourceText, {
          id: `student_submission_${safeSubmissionId}_${safeLessonId}`,
          title: String(snapshot.title ?? payload.local_label ?? lessonId).trim() || lessonId,
          description: "학생 제출에서 다시 연 수업",
          subject: snapshot.subject,
          grade: snapshot.grade,
          requiredViews: snapshot.required_views,
          goals: snapshot.goals,
          missions: snapshot.missions,
        });
        if (snapshot.curriculum_card && typeof snapshot.curriculum_card === "object") {
          importedLesson.curriculumCard = snapshot.curriculum_card;
        }
        importedLesson.curriculumInquiryArtifact = {
          schema: "seamgrim.curriculum_inquiry_artifact.v0",
          lesson_id: lessonId,
          response: {
            prediction_text: String(payload.response?.prediction_text ?? ""),
            explanation_text: String(payload.response?.explanation_text ?? ""),
          },
          run_evidence: {
            runtime_hash: String(payload.run_evidence?.runtime_hash ?? "").trim(),
            engine_status: String(payload.run_evidence?.engine_status ?? "").trim(),
          },
        };
        importedLesson.localPackageId = String(payload.origin_payload_hash ?? payload.origin_hash ?? "").trim() || "student_submission";
        importedLesson.localPackageLessonId = lessonId;
        importedLesson.localPackageTitle = "학생 제출";
        importedLesson.localPackagePayload = snapshot.origin_bundle_payload ?? null;
        importedLesson.localPackageClassroomExchange = payload;
        importedLesson.localPackageClassroomExchangeHash = String(payload.payload_hash ?? "").trim();
        importedLesson.localPackageDeploymentProfileHash = String(payload.deployment_profile_hash ?? "").trim();
        importedLesson.localPackageHash = String(payload.origin_payload_hash ?? "").trim();
        appState.lessonsById.set(importedLesson.id, importedLesson);
        try {
          window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__ = {
            schema: "seamgrim.student_submission_import_action.v1",
            imported: true,
            file_name: fileName,
            exchange_id: String(payload.exchange_id ?? "").trim(),
            lesson_id: lessonId,
            payload_hash: String(payload.payload_hash ?? "").trim(),
            origin_payload_hash: String(payload.origin_payload_hash ?? "").trim(),
            local_label: String(payload.local_label ?? "").trim(),
            opened_lesson_id: importedLesson.id,
          };
        } catch (_) {
          // ignore browser instrumentation errors
        }
        setLocalPackageImportStatus("ok", `학생 제출을 열었습니다: ${String(payload.local_label ?? lessonId).trim() || lessonId}`);
        showPlatformToast("학생 제출을 열었습니다.");
        openRunWithLessonWithSource(importedLesson, {
          launchKind: "local_package_import",
          autoExecute: false,
          sourceId: `student_submission:${safeSubmissionId}`,
          sourceType: "ddn",
          onboardingProfile: "student",
        });
        return true;
      }
      const imported = importStudioLocalPackagePayload(payload);
      const packageId = String(imported.manifest?.package_id ?? "local_package").trim() || "local_package";
      const safePackageId = safeLocalPackageToken(packageId, "local_package");
      const packageTitle = String(imported.manifest?.title ?? packageId).trim() || packageId;
      const packageSessionLabel = String(imported.manifest?.session_label ?? "").trim();
      const deploymentProfile = imported.deployment_profile ?? imported.manifest?.deployment_profile ?? payload.deployment_profile ?? null;
      const deploymentProfileHash = String(imported.manifest?.deployment_profile_hash ?? payload.deployment_profile_hash ?? deploymentProfile?.profile_hash ?? "").trim();
      const classroomExchange = imported.classroom_exchange ?? payload.classroom_exchange ?? null;
      const classroomExchangeHash = String(imported.classroom_exchange_hash ?? payload.classroom_exchange_hash ?? payload.payload_hash ?? classroomExchange?.payload_hash ?? "").trim();
      const packageHash = String(imported.manifest?.package_hash ?? payload.package_hash ?? "").trim();
      const sourceLessons = Array.isArray(imported.lessons) ? imported.lessons : [];
      if (!sourceLessons.length) {
        throw new Error("배포 파일에 실행할 교과 원문이 없습니다.");
      }
      const importedLessons = [];
      for (const [index, lesson] of sourceLessons.entries()) {
        const sourceText = String(lesson?.source_text ?? "").trim();
        if (!sourceText) {
          throw new Error("배포 파일에 실행할 교과 원문이 없습니다.");
        }
        const lessonId = String(lesson.lesson_id ?? `local_package_lesson_${index + 1}`).trim() || `local_package_lesson_${index + 1}`;
        const safeLessonId = safeLocalPackageToken(lessonId, `local_package_lesson_${index + 1}`);
        const importedLessonTitle = String(lesson.title ?? lessonId).trim() || lessonId;
        const importedLesson = await buildStudioDraftLessonFromDdn(sourceText, {
          id: `local_package_${safePackageId}_${safeLessonId}`,
          title: importedLessonTitle,
          description: String(lesson.description ?? "").trim()
            || `배포 파일: ${packageTitle}`,
          subject: lesson.subject,
          grade: lesson.grade,
          requiredViews: lesson.required_views,
          goals: lesson.goals,
          missions: lesson.missions,
        });
        if (lesson.curriculum_card && typeof lesson.curriculum_card === "object") {
          importedLesson.curriculumCard = lesson.curriculum_card;
        }
        importedLesson.localPackageId = packageId;
        importedLesson.localPackageLessonId = lessonId;
        importedLesson.localPackageTitle = packageTitle;
        importedLesson.localPackageSessionLabel = packageSessionLabel;
        importedLesson.localPackageLessonIndex = index + 1;
        importedLesson.localPackageLessonCount = sourceLessons.length;
        importedLessons.push(importedLesson);
      }
      const importedLesson = importedLessons[0];
      const lessonId = String(importedLesson?.localPackageLessonId ?? importedLesson?.id ?? "local_package_lesson").trim() || "local_package_lesson";
      const safeLessonId = safeLocalPackageToken(lessonId, "local_package_lesson");
      const importedLessonTitle = String(importedLesson?.title ?? lessonId).trim() || lessonId;
      const studentInstructions = Array.isArray(imported.manifest?.student_instructions)
        ? imported.manifest.student_instructions.map((item) => String(item ?? "").trim()).filter(Boolean)
        : [];
      const materialSummary = Array.isArray(imported.manifest?.materials_summary)
        ? imported.manifest.materials_summary.map((item) => String(item ?? "").trim()).filter(Boolean)
        : [];
      const studentMaterialSummary = Array.isArray(imported.manifest?.student_materials_summary)
        ? imported.manifest.student_materials_summary.map((item) => String(item ?? "").trim()).filter(Boolean)
        : Array.isArray(imported.student_materials_summary)
          ? imported.student_materials_summary.map((item) => String(item ?? "").trim()).filter(Boolean)
          : [];
      const packageReports = Array.isArray(imported.reports) ? imported.reports : [];
      const packageAssets = Array.isArray(imported.assets) ? imported.assets : [];
      importedLessons.forEach((row) => {
        row.localPackageStudentInstructions = studentInstructions;
        row.localPackageMaterialsSummary = materialSummary;
        row.localPackageStudentMaterialsSummary = studentMaterialSummary;
        row.localPackageManifest = imported.manifest;
        row.localPackagePayload = payload;
        row.localPackageReports = packageReports;
        row.localPackageAssets = packageAssets;
        row.localPackageDeploymentProfile = deploymentProfile;
        row.localPackageDeploymentProfileHash = deploymentProfileHash;
        row.localPackageClassroomExchange = classroomExchange;
        row.localPackageClassroomExchangeHash = classroomExchangeHash;
        row.localPackageHash = packageHash;
        appState.lessonsById.set(row.id, row);
      });
      try {
        window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ = {
          schema: "seamgrim.local_package_import_action.v1",
          imported: true,
          file_name: fileName,
          package_id: packageId,
          session_label: packageSessionLabel,
          lesson_id: lessonId,
          lesson_title: importedLessonTitle,
          imported_lesson_ids: importedLessons.map((row) => String(row.id ?? "").trim()).filter(Boolean),
          lesson_count: imported.lesson_count,
          report_count: imported.report_count,
          report_titles: packageReports.map((report) => String(report?.title ?? report?.report_id ?? "").trim()).filter(Boolean),
          materials_summary: materialSummary,
          student_materials_summary: studentMaterialSummary,
          deployment_profile: deploymentProfile,
          deployment_profile_hash: deploymentProfileHash,
          classroom_exchange: classroomExchange,
          classroom_exchange_hash: classroomExchangeHash,
          package_hash: packageHash,
          account_required: false,
          cloud_sync: false,
          public_registry: false,
        };
      } catch (_) {
        // ignore browser instrumentation errors
      }
      const lessonCount = Math.max(0, Number(imported.lesson_count) || 0);
      const reportCount = Math.max(0, Number(imported.report_count) || 0);
      const reportSummary = materialSummary.find((item) => item.startsWith("교사용 자료 "));
      const successParts = [
        `배포 파일을 열었습니다: ${String(imported.manifest?.title ?? packageId).trim() || packageId}`,
        `실행할 수업 ${importedLessonTitle}`,
        `교과 ${lessonCount}개`,
      ];
      const studentReportSummary = studentMaterialSummary.find((item) => item.startsWith("학생 자료 "));
      if (studentReportSummary) {
        successParts.push(studentReportSummary);
      } else if (reportSummary) {
        successParts.push(reportSummary.replace(/^교사용 자료/u, "자료"));
      } else if (reportCount > 0) {
        successParts.push(`자료 ${reportCount}개`);
      }
      const successMessage = successParts.join(" · ");
      setLocalPackageImportStatus("ok", successMessage);
      showPlatformToast("배포 파일을 열었습니다.");
      openRunWithLessonWithSource(importedLesson, {
        launchKind: "local_package_import",
        autoExecute: false,
        sourceId: `local_package:${safePackageId}:${safeLessonId}`,
        sourceType: "ddn",
        onboardingProfile: "student",
      });
      return true;
    } catch (error) {
      const errorCode = String(error?.message ?? error);
      const userMessage = formatStudioLocalPackageError(error);
      try {
        window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ = {
          schema: "seamgrim.local_package_import_action.v1",
          imported: false,
          file_name: fileName,
          error: errorCode,
          error_message: userMessage,
        };
      } catch (_) {
        // ignore browser instrumentation errors
      }
      setLocalPackageImportStatus("error", `배포 파일 열기 실패: ${userMessage}`);
      showPlatformToast(`배포 파일 열기 실패: ${userMessage}`);
      return false;
    }
  };

  window.addEventListener("seamgrim:student-submission-reopen", async (event) => {
    const submission = event?.detail?.submission;
    if (!submission || typeof submission !== "object") {
      return;
    }
    const payloadHash = String(submission.payload_hash ?? "student_submission").trim() || "student_submission";
    try {
      const opened = await openLocalPackageFile({
        name: `${safeLocalPackageToken(payloadHash, "student_submission")}.json`,
        text: async () => JSON.stringify(submission, null, 2),
      });
      window.__SEAMGRIM_STUDENT_SUBMISSION_INBOX_REOPEN_RESULT__ = {
        schema: "seamgrim.student_submission_inbox_reopen_result.v0",
        opened: opened === true,
        payload_hash: payloadHash,
      };
    } catch (error) {
      window.__SEAMGRIM_STUDENT_SUBMISSION_INBOX_REOPEN_RESULT__ = {
        schema: "seamgrim.student_submission_inbox_reopen_result.v0",
        opened: false,
        payload_hash: payloadHash,
        error: String(error?.message ?? error),
      };
    }
  });

  const browseScreen = new BrowseScreen({
    root: byId("screen-browse"),
    publishRoot: byId("screen-publish"),
    federatedApiCandidates,
    federatedFileCandidates,
    featuredSeedEnabled,
    onLessonSelect: async (selection, { autoExecute = true } = {}) => {
      const lessonId = ensureLessonEntryFromSelection(selection);
      if (!lessonId) {
        throw new Error("교과를 찾지 못했습니다: invalid lesson selection");
      }
      const onboardingProfile = normalizeRunOnboardingProfile(selection?.launchProfile ?? "");
      const launchKind = onboardingProfile === "teacher" ? "browse_select_teacher" : "browse_select";
      const shellSpace = String(selection?.launchAction ?? "learn").trim() === "lab" ? "lab" : "learn";
      await switchMainTab(MAIN_TAB_STUDIO, {
        lessonId,
        launchKind,
        autoExecute,
        onboardingProfile,
        shellSpace,
      });
    },
    onCreate: () => {
      setScreen("editor");
    },
    onOpenLocalPackageFile: openLocalPackageFile,
    onBuildCurrentLessonPackagePayload: ({ deploymentProfile = null } = {}) =>
      buildCurrentProjectLocalPackagePayload({
        deploymentProfile,
      }),
    onOpenLegacyGuideExample: async ({ lesson, example, examples, warningNames }) => {
      const title = String(lesson?.title ?? lesson?.id ?? "교과").trim() || "교과";
      let loadedLesson = null;
      const lessonId = String(lesson?.id ?? "").trim();
      try {
        loadedLesson =
          lessonId && appState.lessonsById.get(lessonId)?.ddnText
            ? appState.lessonsById.get(lessonId)
            : lessonId
              ? await loadLessonById(lessonId)
              : null;
      } catch (_) {
        loadedLesson = null;
      }
      const draftText = loadedLesson?.ddnText
        ? buildLegacyGuideDraftText({
            title,
            ddnText: loadedLesson.ddnText,
            warningNames,
            warningExamples: Array.isArray(examples) && examples.length > 0 ? examples : [String(example ?? "")],
          })
        : String(example ?? "");
      const focusText = Array.isArray(examples) && examples.length > 0
        ? `// 매김 전환 후보: ${examples[0]}`
        : "";
      const focusTexts = Array.isArray(examples)
        ? examples
            .filter(Boolean)
            .map((row) => `// 매김 전환 후보: ${String(row)}`)
        : [];
      if (loadedLesson && typeof loadedLesson === "object") {
        appState.currentLesson = {
          ...loadedLesson,
          ddnText: draftText,
        };
      }
      editorScreen.loadLesson(draftText, {
        title: `${title} · 매김 전환 초안`,
        readOnly: false,
        focusText,
        focusTexts,
      });
      editorScreen.setSmokeResult("구식 범위주석 전환 초안 편집 모드");
      setScreen("editor");
    },
    onOpenAdvanced: () => {
      advanced.toggle();
    },
  });

  const runEditorAutofix = (ddnText, { applyToEditor = false, source = "editor" } = {}) => {
    const sourceText = String(ddnText ?? "");
    const rawResult = applyLegacyAutofixToDdn(sourceText);
    const textAfter = String(rawResult?.text ?? sourceText);
    const contract = buildAutofixResultContract(rawResult, { sourceTextAfter: textAfter });
    const appliedRuleCount = Array.isArray(contract.applied_rules) ? contract.applied_rules.length : 0;
    emitStudioMetric("studio.autofix", {
      source,
      changed: Boolean(contract.changed),
      blocking_remaining: Boolean(contract.blocking_remaining),
      applied_rule_count: appliedRuleCount,
    });
    if (applyToEditor && rawResult?.changed) {
      editorScreen?.replaceDdn?.(textAfter, { emitSourceChange: true });
    }
    return {
      ...(rawResult && typeof rawResult === "object" ? rawResult : {}),
      text: textAfter,
      contract,
    };
  };

  const runLocalProjectSummaryFromWasm = async (projectSource, { target = "앱" } = {}) => {
    const source = projectSource && typeof projectSource === "object" ? projectSource : {};
    const client = await appState.wasm.loader.ensure("1 보여주기.");
    const wasmModule = client?.wasmModule;
    if (!wasmModule || typeof wasmModule.wasm_project_run_summary_json !== "function") {
      throw new Error("wasm_project_run_summary_json export 누락");
    }
    const raw = wasmModule.wasm_project_run_summary_json(
      JSON.stringify(source),
      String(target ?? "앱"),
      true,
      true,
    );
    return JSON.parse(String(raw ?? "{}"));
  };

  editorScreen = new EditorScreen({
    root: byId("screen-editor"),
    onBack: () => {
      setScreen("browse");
    },
    onRun: async (ddnText) => {
      editorScreen.setSmokeResult("WASM 매김 계획 계산 중...");
      const sourceText = String(ddnText ?? "");
      const sourceLessonId = String(appState.currentLesson?.id ?? "").trim();
      const model = buildStudioEditorReadinessModel({
        sourceText,
        autofixAvailable: hasLegacyAutofixCandidate(sourceText),
      });
      lastEditorReadinessModel = model;
      editorScreen?.setStudioReadinessModel?.(model);
      const stage = String(model?.stage ?? "").trim().toLowerCase();

      if (stage === STUDIO_READINESS_STAGE.BLOCKED) {
        const cause = String(model?.user_cause ?? "").trim() || "실행 전 수정이 필요합니다.";
        const fixHint = String(model?.manual_example ?? model?.primary_action?.detail ?? "").trim();
        editorScreen.setSmokeResult(fixHint ? `실행 차단: ${cause}\n수정 가이드: ${fixHint}` : `실행 차단: ${cause}`);
        emitStudioMetric("studio.run.blocked", {
          stage,
          cause,
        });
        return;
      }

      let effectiveDdnText = sourceText;
      let autofixContract = null;
      if (stage === STUDIO_READINESS_STAGE.AUTOFIX || hasLegacyAutofixCandidate(sourceText)) {
        const autofix = runEditorAutofix(sourceText, {
          applyToEditor: false,
          source: "editor_run_preflight",
        });
        autofixContract = autofix.contract ?? null;
        effectiveDdnText = String(autofix.text ?? sourceText);
        if (autofixContract?.blocking_remaining) {
          const reason = String(model?.user_cause ?? "").trim() || "자동 수정으로 해결할 수 없는 항목이 남았습니다.";
          const fixHint = String(model?.manual_example ?? "").trim();
          editorScreen.setSmokeResult(
            fixHint
              ? `실행 차단: ${reason}\n수정 가이드: ${fixHint}`
              : `실행 차단: ${reason}`,
          );
          emitStudioMetric("studio.run.blocked_after_autofix", {
            stage,
            changed: Boolean(autofixContract?.changed),
          });
          return;
        }
        if (autofixContract?.changed) {
          editorScreen.setSmokeResult("자동 수정 변환본으로 실행합니다. 원문 반영은 '자동 수정 적용' 버튼에서 선택할 수 있습니다.");
        }
      }

      emitStudioMetric("studio.run.attempt", {
        stage: stage || "ready",
        autofix_changed: Boolean(autofixContract?.changed),
        source_lesson_id: sourceLessonId || null,
      });
      editorScreen.setSmokeResult("WASM 매김 계획 계산 중...");
      const baseMeta = appState.currentLesson?.meta ?? {};
      const sourceChangedFromLesson = String(effectiveDdnText ?? "").replace(/\r\n/g, "\n").trim()
        !== String(appState.currentLesson?.ddnText ?? "").replace(/\r\n/g, "\n").trim();
      const ddnMetaHeader = parseLessonDdnMetaHeader(effectiveDdnText);
      const displayMeta = resolveLessonDisplayMeta({
        baseTitle: appState.currentLesson?.title ?? "사용자 DDN",
        baseDescription: appState.currentLesson?.description ?? "",
        tomlMeta: baseMeta,
        ddnMetaHeader,
      });
      const baseLesson = {
        id: appState.currentLesson?.id ?? "custom",
        title: displayMeta.title || appState.currentLesson?.title || "사용자 DDN",
        description: displayMeta.description || appState.currentLesson?.description || "",
        subject: appState.currentLesson?.subject ?? "",
        grade: appState.currentLesson?.grade ?? "",
        quality: appState.currentLesson?.quality ?? "experimental",
        requiredViews: normalizeViewFamilyList(baseMeta.required_views ?? ddnMetaHeader.requiredViews ?? ddnMetaHeader.required_views ?? appState.currentLesson?.requiredViews ?? []),
        ddnText: effectiveDdnText,
        maegimControlJson: sourceChangedFromLesson ? "" : (appState.currentLesson?.maegimControlJson ?? ""),
        textMd: appState.currentLesson?.textMd ?? "",
        meta: baseMeta,
        ddnMetaHeader,
      };
      const hydratedLesson = await lessonCanonHydrator.hydrateLessonCanon(baseLesson);
      appState.currentLesson = hydratedLesson;
      openRunWithLessonWithSource(hydratedLesson, {
        launchKind: "editor_run",
        runRequest: { id: `editor:${Date.now()}`, sourceText: effectiveDdnText, launchKind: "editor_run", sourceType: "ddn", createdAtMs: Date.now() },
        sourceId: `ddn:editor:${hydratedLesson.id}`,
        sourceType: "ddn",
        derivedFrom: sourceLessonId,
      });
    },
    onSave: (ddnText) => {
      const ok = saveCurrentWork("local", { ddnText });
      editorScreen.setSmokeResult(ok ? "DDN 파일 저장 완료" : "저장 실패");
    },
    onSaveLessonPlan: (lessonPlanText, { model = null } = {}) => {
      const text = serializeLessonPlanToml(model ?? parseLessonPlanToml(lessonPlanText, {
        fallbackLessonId: appState.currentLesson?.id ?? "custom",
        fallbackTitle: appState.currentLesson?.title ?? "수업 구성",
      }));
      const parsed = parseLessonPlanToml(text, {
        fallbackLessonId: appState.currentLesson?.id ?? "custom",
        fallbackTitle: appState.currentLesson?.title ?? "수업 구성",
      });
      if (appState.currentLesson && typeof appState.currentLesson === "object") {
        appState.currentLesson.lessonPlanText = text;
        appState.currentLesson.lessonPlan = parsed;
        const currentId = String(appState.currentLesson.id ?? "").trim();
        if (currentId) {
          appState.lessonsById.set(currentId, appState.currentLesson);
        }
      }
      try {
        const lessonId = String(parsed.lesson_id ?? appState.currentLesson?.id ?? "").trim();
        window.localStorage?.setItem?.(LESSON_PLAN_DRAFT_KEY, text);
        if (lessonId) window.localStorage?.setItem?.(lessonPlanDraftKey(lessonId), text);
      } catch (_) {
        // ignore localStorage failures
      }
      editorScreen.loadLessonPlan(text, {
        lessonPlan: parsed,
        lessonId: appState.currentLesson?.id ?? "custom",
        title: appState.currentLesson?.title ?? "수업 구성",
        lessonChoices: getFoundation12LessonChoices(),
      });
      editorScreen.setSmokeResult(parsed.diagnostics.length ? `수업 구성 저장 · 진단 ${parsed.diagnostics.length}건` : "수업 구성 저장 완료");
    },
    onSelectLesson: async (lessonId) => {
      const targetId = String(lessonId ?? "").trim();
      if (!targetId) return;
      const lesson = await loadLessonById(targetId);
      editorScreen.loadLesson(lesson.ddnText, {
        title: lesson.title ?? lesson.id ?? "수업 구성",
        readOnly: false,
        lessonPlanText: lesson.lessonPlanText ?? "",
        lessonPlan: lesson.lessonPlan ?? null,
        lessonId: lesson.id ?? "",
        lessonChoices: getFoundation12LessonChoices(),
      });
      editorScreen.setEditorMode("lesson-plan");
      setScreen("editor", { shellSpace: "create" });
    },
    onOpenAdvanced: () => {
      advanced.toggle();
    },
    onSourceChange: (ddnText) => {
      void refreshEditorCanonSummary(ddnText);
    },
    onAutofix: async (ddnText, { source = "editor_readiness_card" } = {}) => {
      const result = runEditorAutofix(ddnText, {
        applyToEditor: false,
        source,
      });
      if (result?.changed) {
        editorScreen.replaceDdn(result.text, { emitSourceChange: true });
        const rules = Array.isArray(result.contract?.applied_rules) ? result.contract.applied_rules.join(", ") : "";
        editorScreen.setSmokeResult(rules ? `자동 수정 적용: ${rules}` : "자동 수정을 적용했습니다.");
      } else if (result?.contract?.blocking_remaining) {
        editorScreen.setSmokeResult("자동 수정으로 해결되지 않는 항목이 있습니다. 수동 수정 가이드를 확인하세요.");
      } else {
        editorScreen.setSmokeResult("자동 수정 대상이 없습니다.");
      }
      updateEditorReadinessModel(String(result?.text ?? ddnText));
      return result;
    },
    onApplyFixit: async (_preview, { sourceText = "" } = {}) => {
      const result = runEditorAutofix(sourceText, {
        applyToEditor: true,
        source: "editor_fixit_card",
      });
      if (result?.changed) {
        const rules = Array.isArray(result.contract?.applied_rules) ? result.contract.applied_rules.join(", ") : "";
        editorScreen.setSmokeResult(rules ? `수정 적용: ${rules}` : "수정 후보를 적용했습니다.");
      } else {
        editorScreen.setSmokeResult("적용할 수정 후보가 없습니다.");
      }
      updateEditorReadinessModel(String(result?.text ?? sourceText));
      editorScreen?.setFixitModel?.(buildEditorFixitPreview(String(result?.text ?? sourceText)));
      return result;
    },
    onProjectRunSummary: runLocalProjectSummaryFromWasm,
  });

  runScreen = new RunScreen({
    root: byId("screen-run"),
    wasmState: appState.wasm,
    allowShapeFallback,
    allowServerFallback,
    getOverlaySession: () => appState.overlaySession,
    getRuntimeSessionV0: () => appState.runtimeSessionV0,
    onOverlaySessionChange: (sessionLike) => {
      const row = sessionLike && typeof sessionLike === "object" ? sessionLike : {};
      const runs = Array.isArray(row.runs) ? row.runs : [];
      const compare = resolveOverlayCompareFromSession({
        runs,
        compare: row.compare ?? {},
      });
      const viewCombo = resolveSessionViewComboFromPayload(row.viewCombo ?? row.view_combo ?? {});
      appState.overlaySession = {
        runs,
        compare,
        viewCombo,
      };
      persistOverlaySessionSnapshot();
      persistSceneSummarySnapshot(runScreen);
      persistRuntimeSnapshotBundleV0(runScreen);
    },
    onSourceChange: (ddnText) => {
      void refreshEditorCanonSummary(ddnText);
    },
    onStudioStateChange: (patch) => {
      setStudioState(patch);
    },
    onSaveDdn: (ddnText) => saveCurrentWork("local", { ddnText }),
    onSaveSnapshot: () => persistRuntimeSnapshotV0(runScreen),
    onSaveSession: () => persistRuntimeSessionV0(runScreen),
    resolveControlPlan: (sourceText) => lessonCanonHydrator.deriveMaegimControlJson(sourceText),
    onSelectLesson: async (lessonId) => {
      const targetId = String(lessonId ?? "").trim();
      if (!targetId) return;
      const localPackageLesson = appState.lessonsById.get(targetId);
      if (
        localPackageLesson
        && String(localPackageLesson.localPackageId ?? "").trim()
        && String(localPackageLesson.ddnText ?? "").trim()
      ) {
        const packageId = String(localPackageLesson.localPackageId ?? "").trim();
        const originalLessonId = String(localPackageLesson.localPackageLessonId ?? localPackageLesson.id ?? "").trim();
        openRunWithLessonWithSource(localPackageLesson, {
          launchKind: "local_package_import",
          autoExecute: false,
          sourceId: `local_package:${safeLocalPackageToken(packageId, "local_package")}:${safeLocalPackageToken(originalLessonId, "local_package_lesson")}`,
          sourceType: "ddn",
          onboardingProfile: "student",
        });
        return;
      }
      await switchMainTab(MAIN_TAB_STUDIO, {
        lessonId: targetId,
        launchKind: "browse_select",
        autoExecute: true,
      });
    },
    onGetInspectorContext: () => ({
      sceneSummary: appState.sceneSummary,
      snapshotV0: appState.runtimeSnapshotV0,
      sessionV0: appState.runtimeSessionV0,
    }),
    onBack: () => {
      runScreen?.flushOverlaySession?.();
      persistSceneSummarySnapshot(runScreen);
      persistRuntimeSnapshotBundleV0(runScreen);
      setScreen("browse");
    },
    onEditDdn: ({ ddnText, title }) => {
      editorScreen.loadLesson(ddnText, {
        title,
        readOnly: true,
        lessonPlanText: appState.currentLesson?.lessonPlanText ?? "",
        lessonPlan: appState.currentLesson?.lessonPlan ?? null,
        lessonId: appState.currentLesson?.id ?? "",
        lessonChoices: getFoundation12LessonChoices(),
      });
      setScreen("editor");
    },
    onOpenAdvanced: () => {
      advanced.toggle();
    },
  });

  const advanced = DEV_SURFACES_ENABLED ? createAdvancedMenu({
    onSmoke: async () => {
      const source =
        appState.currentScreen === "editor"
          ? editorScreen.getDdn()
          : appState.currentLesson?.ddnText ?? "";
      if (!String(source).trim()) {
        editorScreen.setSmokeResult("Smoke: 검사할 DDN이 없습니다.");
        setScreen("editor");
        return;
      }
      try {
        const ensureWasm = (text) => appState.wasm.loader.ensure(text);
        const result = await applyWasmLogicAndDispatchState({
          sourceText: source,
          ensureWasm,
          mode: appState.wasm.langMode,
        });
        const hash = typeof result.client?.getStateHash === "function" ? result.client.getStateHash() : "-";
        const parseWarningCount = Array.isArray(result.parseWarnings) ? result.parseWarnings.length : 0;
        const parseWarningText = parseWarningCount > 0 ? ` · parse_warning=${parseWarningCount}` : "";
        editorScreen.setSmokeResult(`Smoke: 성공 · state_hash=${hash}${parseWarningText}`);
        if (appState.currentScreen !== "editor") {
          setScreen("editor");
        }
      } catch (err) {
        editorScreen.setSmokeResult(`Smoke: 실패 · ${String(err?.message ?? err)}`);
        if (appState.currentScreen !== "editor") {
          setScreen("editor");
        }
      }
    },
  }) : createNoopAdvancedMenu();

  browseScreen.init();
  editorScreen.init();
  runScreen.init();
  byId("btn-project-save")?.addEventListener("click", () => {
    void saveProjectFromEditor();
  });
  byId("btn-project-clone")?.addEventListener("click", () => {
    void cloneProjectFromSession();
  });
  byId("btn-project-revisions-refresh")?.addEventListener("click", () => {
    void loadProjectRevisions();
  });
  byId("project-revision-list")?.addEventListener("click", (event) => {
    const target = event.target;
    if (!(target instanceof Element)) return;
    const button = target.closest("[data-project-restore-revision]");
    if (!button) return;
    void restoreProjectRevision(button.getAttribute("data-project-restore-revision") ?? "");
  });
  byId("btn-project-conflict-copy-local")?.addEventListener("click", () => {
    const textarea = byId("project-conflict-local-package");
    const text = String(textarea?.value ?? "");
    if (!text) {
      setProjectSaveStatus("conflict", "보존할 로컬 꾸러미가 없습니다.");
      return;
    }
    try {
      navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional; the textarea remains the durable local copy surface.
    }
    setProjectSaveStatus("conflict", "로컬 꾸러미 보존됨");
  });
  renderRecentProjects();
  updateProjectRoleControls(readProjectCapabilitySession()?.role ?? "edit");
  const learningSpineButton = byId("btn-learning-spine");
  const learningSpinePanel = byId("learning-spine-panel");
  const learningSpineIndex =
    (await fetchJson(`${PROJECT_PREFIX}learning_spine_index.detjson`)) ??
    (await fetchJson("learning_spine_index.detjson"));
  let learningSpine = null;
  const setLearningSpineBrowseMode = (open) => {
    const active = Boolean(open);
    learningSpineButton?.classList.toggle("active", active);
    learningSpineButton?.setAttribute("aria-expanded", active ? "true" : "false");
    byId("screen-browse")?.classList.toggle("learning-spine-open", active);
  };
  if (learningSpineIndex && learningSpinePanel) {
    learningSpine = mountLearningSpineIndex({
      root: learningSpinePanel,
      index: learningSpineIndex,
      opener: learningSpineButton,
      onOpenLesson: async (lessonId) => {
        await switchMainTab(MAIN_TAB_STUDIO, {
          lessonId,
          launchKind: "learning_spine",
          autoExecute: true,
          fallbackToBlankOnLessonError: false,
        });
      },
      onOpenChange: setLearningSpineBrowseMode,
    });
    learningSpineButton?.addEventListener("click", () => learningSpine?.toggle());
    document.querySelectorAll(".browse-tab[data-tab]").forEach((button) => {
      button.addEventListener("click", () => learningSpine?.close());
    });
    try {
      window.__SEAMGRIM_LEARNING_SPINE_INDEX__ = learningSpineIndex;
    } catch (_) {
      // Browser instrumentation is optional.
    }
  } else if (learningSpineButton) {
    learningSpineButton.disabled = true;
  }
  if (!DEV_SURFACES_ENABLED) {
    mountSeulgiProductSurfaces();
  }
  // TODO(retire-dev-surfaces): this is a temporary opt-in mount path; remove it with dev_surfaces.js.
  if (DEV_SURFACES_ENABLED) {
    void import("./dev_surfaces.js")
      .then(({ mountDevSurfaces }) => {
        mountDevSurfaces({ openStudioWithDdnText });
      })
      .catch((err) => {
        console.warn("dev surfaces failed to load", err);
      });
  }
  document.querySelectorAll(".main-shell-tab[data-main-tab-target]").forEach((button) => {
    button.addEventListener("click", () => {
      const shellSpace = String(button?.dataset?.shellSpace ?? "").trim();
      if (shellSpace === "create") {
        const current = appState.currentLesson;
        if (current?.ddnText) {
          editorScreen.loadLesson(current.ddnText, {
            title: current.title ?? current.id ?? "수업 만들기",
            readOnly: false,
            lessonPlanText: current.lessonPlanText ?? "",
            lessonPlan: current.lessonPlan ?? null,
            lessonId: current.id ?? "",
            lessonChoices: getFoundation12LessonChoices(),
          });
        }
        setScreen("editor");
        return;
      }
      if (shellSpace === "publish") {
        setScreen("publish", { shellSpace: "publish" });
        return;
      }
      if (shellSpace === "lab" && appState.currentScreen === "editor" && appState.currentLesson?.ddnText) {
        openRunWithLessonWithSource(appState.currentLesson, {
          launchKind: "manual",
          sourceId: `lesson:${appState.currentLesson.id ?? "current"}`,
          sourceType: "lesson",
          shellSpace: "lab",
        });
        return;
      }
      const target = normalizeMainTabTarget(button?.dataset?.mainTabTarget, MAIN_TAB_BROWSE);
      void switchMainTab(target, { launchKind: "manual", shellSpace: shellSpace || "lab" });
    });
  });
  getFeaturedSeedButton()?.addEventListener("click", () => {
    void runNextFeaturedSeed();
  });
  if (typeof window !== "undefined" && window?.addEventListener) {
    window.addEventListener("seamgrim:browse-preset-changed", (event) => {
      const presetId = normalizeBrowsePresetId(event?.detail?.presetId ?? "");
      syncBrowsePresetToLocation(presetId);
    });
    window.addEventListener(PLATFORM_UI_ACTION_EVENT, (event) => {
      handlePlatformUiActionRequest(event?.detail ?? {}, {
        resolveDdnText: () => {
          if (appState.currentScreen === "editor") {
            return String(editorScreen?.getDdn?.() ?? "");
          }
          return String(appState.currentLesson?.ddnText ?? "");
        },
      });
    });
    window.addEventListener("keydown", (event) => {
      if (!featuredSeedEnabled) return;
      if (
        shouldTriggerFeaturedSeedQuickPreset(event, {
          isEditableTarget: isEditableKeyboardTarget(event?.target),
          isBrowseScreen: appState.currentScreen === "browse",
        })
      ) {
        event.preventDefault();
        browseScreen.applyFeaturedSeedQuickRecentPreset();
        return;
      }
      if (
        !shouldTriggerFeaturedSeedQuickLaunch(event, {
          isEditableTarget: isEditableKeyboardTarget(event?.target),
        })
      ) {
        return;
      }
      event.preventDefault();
      void runNextFeaturedSeed();
    });
  }
  onScreenChange((screenName) => {
    runScreen.setScreenVisible(screenName === "run");
    if (screenName !== "run") {
      runScreen?.flushOverlaySession?.();
      persistSceneSummarySnapshot(runScreen);
      persistRuntimeSnapshotBundleV0(runScreen);
    }
  });

  try {
    const lessons = await loadCatalogLessons();
    browseScreen.setLessons(lessons);
    runScreen.setLessonOptions(lessons);
    if (browsePresetFromLocation) {
      browseScreen.applyBrowsePreset(browsePresetFromLocation);
    }
    updateFeaturedSeedQuickAction();
  } catch (err) {
    console.error(err);
    updateFeaturedSeedQuickAction();
  }

  let initialScreenApplied = false;
  if (initialProjectCapability?.project_id) {
    initialScreenApplied = await openProjectFromCapabilitySession();
  }
  if (!initialScreenApplied && shouldApplyPlatformRouteFallback(platformRouteSlots)) {
    if (!appState.shell.shareMode) {
      appState.shell.shareMode = Visibility.PRIVATE;
    }
    applyPlatformRouteFallback(platformRouteSlots);
    setScreen("browse");
    initialScreenApplied = true;
  } else {
    if (studioRouteRequest.fromLegacyPlayRedirect && consumeLegacyPlayRedirectParam()) {
      showPlatformToast("예전 Playground 주소를 작업실로 옮겼습니다. 이제 작업실에서 편집하고 실행하세요.");
    }
    if (firstDelightLessonId) {
      const opened = await switchMainTab(MAIN_TAB_STUDIO, {
        lessonId: firstDelightLessonId,
        launchKind: "first_delight",
        autoExecute: true,
        fallbackToBlankOnLessonError: false,
      });
      if (!opened) {
        showPlatformToast("첫 세계를 열지 못했습니다.");
        setScreen("browse");
      }
      initialScreenApplied = true;
    } else if (studioRouteRequest.lessonId) {
      const opened = await switchMainTab(MAIN_TAB_STUDIO, {
        lessonId: studioRouteRequest.lessonId,
        launchKind: "browse_select",
        autoExecute: true,
        fallbackToBlankOnLessonError: false,
      });
      if (!opened) {
        showPlatformToast(`교과를 찾지 못했습니다: ${studioRouteRequest.lessonId}`);
        setScreen("browse");
      }
      initialScreenApplied = true;
    } else if (studioRouteRequest.ddnText) {
      const opened = await openStudioWithDdnText(studioRouteRequest.ddnText, {
        launchKind: "manual",
        sourceId: "ddn:studio:query",
      });
      initialScreenApplied = Boolean(opened);
    } else if (studioRouteRequest.tab === MAIN_TAB_STUDIO) {
      const opened = await switchMainTab(MAIN_TAB_STUDIO, { launchKind: "manual" });
      initialScreenApplied = Boolean(opened);
    } else if (studioRouteRequest.tab === MAIN_TAB_PUBLISH) {
      const opened = await switchMainTab(MAIN_TAB_PUBLISH, { launchKind: "manual" });
      initialScreenApplied = Boolean(opened);
    }
  }

  if (!initialScreenApplied) {
    setScreen("browse");
  }
  if (learningSpineFromLocation && !firstDelightLessonId && !studioRouteRequest.hasStudioRequest) {
    setScreen("browse");
    learningSpine?.open();
  }
  window.addEventListener("beforeunload", () => {
    runScreen?.flushOverlaySession?.();
    persistSceneSummarySnapshot(runScreen);
    persistRuntimeSnapshotBundleV0(runScreen);
    persistOverlaySessionSnapshot();
    persistInputRegistrySnapshot();
  });
}

void main();
