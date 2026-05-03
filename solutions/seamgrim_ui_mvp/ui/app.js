import { createWasmLoader, applyWasmLogicAndDispatchState } from "./wasm_page_common.js";
import { createLessonCanonHydrator, buildFlatPlanView } from "./runtime/index.js";
import { BrowseScreen } from "./screens/browse.js";
import { EditorScreen, saveDdnToFile } from "./screens/editor.js";
import { BlockEditorScreen } from "./screens/block_editor.js";
import { RunScreen, applyLegacyAutofixToDdn, hasLegacyAutofixCandidate } from "./screens/run.js";
import { normalizeViewFamilyList } from "./view_family_contract.js";
import {
  buildAutofixResultContract,
  buildStudioEditorReadinessModel,
  STUDIO_READINESS_STAGE,
} from "./studio_edit_run_contract.js";
import {
  parseLessonDdnMetaHeader,
  resolveLessonDisplayMeta,
} from "./lesson_loader_contract.js";
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
const ACTIVE_ALLOWLIST_PATH = "solutions/seamgrim_ui_mvp/lessons/active_allowlist.detjson";
const CATALOG_MODE_REPS_ONLY = "reps_only";
const CATALOG_MODE_FULL = "full";
const SIM_CORE_POLICY_CLASS = "policy-sim-core";
const OVERLAY_SESSION_STORAGE_KEY = "seamgrim.overlay_session.v1";
const INPUT_REGISTRY_STORAGE_KEY = "seamgrim.input_registry.v0";
const SCENE_SUMMARY_STORAGE_KEY = "seamgrim.scene_summary.v0";
const SNAPSHOT_V0_STORAGE_KEY = "seamgrim.snapshot.v0";
const SESSION_V0_STORAGE_KEY = "seamgrim.session.v0";
const BROWSE_PRESET_QUERY_KEY = "browsePreset";
const BROWSE_UI_PREFS_STORAGE_KEY = "seamgrim.ui.browse_prefs.v1";
// lesson canon runtime(./runtime/*)에서 동적 import하므로 경로 기준은 runtime 디렉터리다.
const WASM_CANON_RUNTIME_URL = "../wasm/ddonirang_tool.js";
const PLATFORM_UI_ACTION_EVENT = "seamgrim:platform-ui-action";
const PLATFORM_REVIEW_ACTION_EVENT = "seamgrim:platform-review-action";
const MAIN_TAB_BROWSE = "browse";
const MAIN_TAB_STUDIO = "studio";
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
  },
  lessonsById: new Map(),
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
  return direct;
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

function setScreen(name) {
  ["browse", "editor", "block_editor", "run"].forEach((screenName) => {
    const node = byId(`screen-${screenName}`);
    if (!node) return;
    node.classList.toggle("hidden", screenName !== name);
  });
  appState.currentScreen = name;
  const mainTabTarget = name === "browse" ? MAIN_TAB_BROWSE : MAIN_TAB_STUDIO;
  if (mainTabTarget === MAIN_TAB_BROWSE) {
    syncMainTabRoute({ tab: MAIN_TAB_BROWSE });
  }
  try {
    const tabButtons = document?.querySelectorAll?.(".main-shell-tab[data-main-tab-target]") ?? [];
    tabButtons.forEach((button) => {
      const target = normalizeMainTabTarget(button?.dataset?.mainTabTarget, MAIN_TAB_BROWSE);
      button?.classList?.toggle?.("active", target === mainTabTarget);
      try {
        button?.setAttribute?.("aria-pressed", target === mainTabTarget ? "true" : "false");
      } catch (_) {
        // ignore aria update errors
      }
    });
  } catch (_) {
    // ignore main tab update errors
  }
  appState.screenListeners.forEach((listener) => {
    try {
      listener(name);
    } catch (_) {
      // ignore screen listener errors
    }
  });
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

function parseTomlMeta(text) {
  if (!text) return {};
  const out = {};
  const lines = String(text).replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  lines.forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) return;
    const match = trimmed.match(/^([A-Za-z0-9_]+)\s*=\s*(.+)$/);
    if (!match) return;
    const key = match[1].trim();
    const rawValue = match[2].trim();
    if (rawValue.startsWith("[") && rawValue.endsWith("]")) {
      const inner = rawValue.slice(1, -1).trim();
      const values = inner
        ? inner.split(",").map((item) => String(item ?? "").trim().replace(/^"(.*)"$/, "$1")).filter(Boolean)
        : [];
      out[key] = key === "required_views" ? normalizeViewFamilyList(values) : values;
      return;
    }
    if (rawValue.startsWith('"') && rawValue.endsWith('"')) {
      out[key] = rawValue.slice(1, -1);
      return;
    }
    out[key] = rawValue;
  });
  return out;
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
  if (normalized === "seed") return "solutions/seamgrim_ui_mvp/seed_lessons_v1";
  if (normalized === "rewrite") return "solutions/seamgrim_ui_mvp/lessons_rewrite_v1";
  return "solutions/seamgrim_ui_mvp/lessons";
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
    requiredViews: selection.requiredViews ?? selection.required_views ?? [],
    ddnCandidates: ddnCandidates.length ? ddnCandidates : [fallback.ddn],
    maegimControlCandidates: deriveMaegimControlCandidates(
      ddnCandidates.length ? ddnCandidates : [fallback.ddn],
      maegimControlCandidates,
    ),
    textCandidates: textCandidates.length ? textCandidates : [fallback.text],
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
        requiredViews: row?.requiredViews ?? row?.required_views ?? [],
        ddnCandidates: ddnCandidates.length ? ddnCandidates : [fallback.ddn],
        maegimControlCandidates: deriveMaegimControlCandidates(
          ddnCandidates.length ? ddnCandidates : [fallback.ddn],
          maegimControlCandidates,
        ),
        textCandidates: textCandidates.length ? textCandidates : [fallback.text],
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
  rows.forEach((raw) => {
    const lessonId = String(raw ?? "").trim();
    if (lessonId) out.add(lessonId);
  });
  return out;
}

function filterCatalogByAllowlist(merged, allowlist) {
  if (!(allowlist instanceof Set) || allowlist.size === 0) return;
  Array.from(merged.keys()).forEach((lessonId) => {
    if (!allowlist.has(lessonId)) {
      merged.delete(lessonId);
    }
  });
}

async function loadCatalogLessons() {
  const catalogMode = resolveCatalogMode();
  appState.catalogMode = catalogMode;
  const repsOnly = catalogMode !== CATALOG_MODE_FULL;
  const activeAllowlist = await loadActiveLessonAllowlist();
  const merged = new Map();

  const inventoryApi = await fetchFirstOk(["/api/lessons/inventory", "/api/lesson-inventory"], "json");
  if (inventoryApi.ok) {
    mergeCatalogFromInventoryPayload(merged, inventoryApi.data);
    if (repsOnly) {
      filterCatalogByAllowlist(merged, activeAllowlist);
    }
  }

  if (merged.size === 0) {
    const indexJson = await fetchJson("solutions/seamgrim_ui_mvp/lessons/index.json");
    const indexLessons = Array.isArray(indexJson?.lessons) ? indexJson.lessons : [];
    indexLessons.forEach((row) => {
      const id = String(row.id ?? "").trim();
      if (!id) return;
      if (repsOnly && activeAllowlist.size > 0 && !activeAllowlist.has(id)) return;
      const paths = lessonPathsFromId("solutions/seamgrim_ui_mvp/lessons", id);
      const rowMetaCandidates = resolveSelectionCandidates(row, ["metaCandidates", "meta_path"]);
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
          requiredViews: row.requiredViews ?? row.required_views ?? [],
          ddnCandidates: [paths.ddn],
          maegimControlCandidates: [paths.maegimControl],
          textCandidates: [paths.text],
          graphCandidates: [paths.graph],
          tableCandidates: [paths.table],
          space2dCandidates: [paths.space2d],
          structureCandidates: [paths.structure],
          metaCandidates: rowMetaCandidates,
        }),
      );
    });

    if (!repsOnly) {
      const seedManifest = await fetchJson("solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson");
      const seeds = Array.isArray(seedManifest?.seeds) ? seedManifest.seeds : [];
      seeds.forEach((seed) => {
        const id = String(seed.seed_id ?? "").trim();
        if (!id) return;
        const fallback = lessonPathsFromId("solutions/seamgrim_ui_mvp/seed_lessons_v1", id);
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

      const rewriteManifest = await fetchJson("solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson");
      const generated = Array.isArray(rewriteManifest?.generated) ? rewriteManifest.generated : [];
      generated.forEach((row) => {
        const id = String(row.lesson_id ?? "").trim();
        if (!id) return;
        const fallback = lessonPathsFromId("solutions/seamgrim_ui_mvp/lessons_rewrite_v1", id);
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

  const lessons = Array.from(merged.values()).sort((a, b) => String(a.title).localeCompare(String(b.title), "ko"));
  if (lessons.length === 0) {
    throw new Error(
      "교과 카탈로그를 찾지 못했습니다. ddn_exec_server.py(8787)로 실행했는지 확인하세요.",
    );
  }
  lessons.forEach((lesson) => {
    appState.lessonsById.set(lesson.id, lesson);
  });
  return lessons;
}

async function loadLessonById(lessonId) {
  const base = appState.lessonsById.get(lessonId);
  if (!base) throw new Error(`교과를 찾지 못했습니다: ${lessonId}`);

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

  const maegimControlCandidates = deriveMaegimControlCandidates(base.ddnCandidates, base.maegimControlCandidates);
  const maegimControlJson = (await fetchText(maegimControlCandidates)) ?? "";
  const textMd = (await fetchText(base.textCandidates)) ?? "";
  const metaRaw = await fetchText(base.metaCandidates);
  const meta = parseTomlMeta(metaRaw);
  const ddnMetaHeader = parseLessonDdnMetaHeader(ddnText);
  const displayMeta = resolveLessonDisplayMeta({
    baseTitle: base.title,
    baseDescription: base.description,
    tomlMeta: meta,
    ddnMetaHeader,
  });

  const lesson = await lessonCanonHydrator.hydrateLessonCanon({
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
  });

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
  const lesson = await lessonCanonHydrator.hydrateLessonCanon({
    id: String(id ?? STUDIO_DRAFT_LESSON_ID).trim() || STUDIO_DRAFT_LESSON_ID,
    title: displayMeta.title || title,
    description: displayMeta.description || description,
    subject: normalizeSubject(appState.currentLesson?.subject ?? ""),
    grade: appState.currentLesson?.grade ?? "",
    quality: appState.currentLesson?.quality ?? "experimental",
    requiredViews: normalizeViewFamilyList(
      baseMeta.required_views ??
      ddnMetaHeader.requiredViews ??
      ddnMetaHeader.required_views ??
      appState.currentLesson?.requiredViews ??
      [],
    ),
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

function createAdvancedMenu({ onSmoke }) {
  const menu = byId("advanced-menu");
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
  const allowShapeFallback = readWindowBoolean("SEAMGRIM_ENABLE_SHAPE_FALLBACK", true);
  const allowServerFallback =
    readWindowBoolean("SEAMGRIM_ENABLE_SERVER_FALLBACK", false) ||
    readQueryBoolean("server_fallback", false);
  const federatedFileCandidates = allowFederatedFileFallback
    ? readWindowStringArray("SEAMGRIM_FEDERATED_FILE_CANDIDATES", [])
    : [];
  const browsePresetFromLocation = readBrowsePresetFromLocation();
  const studioRouteRequest = readStudioRouteRequestFromLocation();
  const platformRouteSlots = readPlatformRouteSlotsFromLocation();
  const catalogMode = resolveCatalogMode();
  appState.catalogMode = catalogMode;
  const featuredSeedEnabled = catalogMode === CATALOG_MODE_FULL;
  const featuredSeedButton = byId("btn-preset-featured-seed-quick-recent");
  let runScreen = null;
  let editorScreen = null;
  let blockEditorScreen = null;
  let lastEditorReadinessModel = buildStudioEditorReadinessModel({
    sourceText: "",
    canonDiagCode: "",
    canonDiagMessage: "",
    autofixAvailable: false,
  });

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
    return appState.studio;
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
    setScreen("run");
    void refreshEditorCanonSummary(lesson?.ddnText ?? "");
    updateFeaturedSeedQuickAction();
    return queuedRunRequest;
  };

  const openRunWithLesson = (lesson, { launchKind = "manual", autoExecute = false } = {}) => {
    openRunWithLessonWithSource(lesson, { launchKind, autoExecute });
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
      fallbackToBlankOnLessonError = true,
    } = {},
  ) => {
    const normalizedTarget = normalizeMainTabTarget(target, MAIN_TAB_BROWSE);
    if (normalizedTarget === MAIN_TAB_BROWSE) {
      setScreen("browse");
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

  const browseScreen = new BrowseScreen({
    root: byId("screen-browse"),
    federatedApiCandidates,
    federatedFileCandidates,
    featuredSeedEnabled,
    onLessonSelect: async (selection, { autoExecute = true } = {}) => {
      const lessonId = ensureLessonEntryFromSelection(selection);
      if (!lessonId) {
        throw new Error("교과를 찾지 못했습니다: invalid lesson selection");
      }
      const onboardingProfile = normalizeRunOnboardingProfile(
        selection?.launchProfile ?? selection?.onboardingProfile ?? readBrowseLaunchProfile(),
      );
      const launchKind = onboardingProfile ? `browse_select_${onboardingProfile}` : "browse_select";
      await switchMainTab(MAIN_TAB_STUDIO, {
        lessonId,
        launchKind,
        autoExecute,
        onboardingProfile,
      });
    },
    onCreate: () => {
      void switchMainTab(MAIN_TAB_STUDIO, { launchKind: "manual" });
    },
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
        maegimControlJson: appState.currentLesson?.maegimControlJson ?? "",
        textMd: appState.currentLesson?.textMd ?? "",
        meta: baseMeta,
        ddnMetaHeader,
      };
      appState.currentLesson = baseLesson;
      const queuedRunRequest = openRunWithLessonWithSource(baseLesson, {
        launchKind: "editor_run",
        runRequest: { id: `editor:${Date.now()}`, sourceText: effectiveDdnText, launchKind: "editor_run", sourceType: "ddn", createdAtMs: Date.now() },
        sourceId: `ddn:editor:${baseLesson.id}`,
        sourceType: "ddn",
        derivedFrom: sourceLessonId,
      });
      void lessonCanonHydrator.hydrateLessonCanon(baseLesson)
        .then((hydratedLesson) => {
          if (hydratedLesson && appState.currentLesson === baseLesson && String(hydratedLesson.ddnText ?? "") === queuedRunRequest?.sourceText) appState.currentLesson = hydratedLesson;
        })
        .catch((err) => console.warn(`[seamgrim] editor run hydration skipped: ${String(err?.message ?? err)}`));
    },
    onSave: (ddnText) => {
      const ok = saveCurrentWork("local", { ddnText });
      editorScreen.setSmokeResult(ok ? "DDN 파일 저장 완료" : "저장 실패");
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
    onOpenBlock: async (ddnText, { title = "블록 편집" } = {}) => {
      await blockEditorScreen.loadSource(ddnText, { title, mode: "" });
      setScreen("block_editor");
    },
  });

  blockEditorScreen = new BlockEditorScreen({
    root: byId("screen-block_editor"),
    onBack: () => {
      setScreen("editor");
    },
    onTextMode: (ddnText, { title = "DDN 편집" } = {}) => {
      editorScreen.loadLesson(ddnText, {
        title,
        readOnly: false,
      });
      setScreen("editor");
    },
    onRun: async (ddnText, { title = "사용자 DDN" } = {}) => {
      const baseMeta = appState.currentLesson?.meta ?? {};
      const ddnMetaHeader = parseLessonDdnMetaHeader(ddnText);
      const displayMeta = resolveLessonDisplayMeta({
        baseTitle: title,
        baseDescription: appState.currentLesson?.description ?? "",
        tomlMeta: baseMeta,
        ddnMetaHeader,
      });
      const lesson = await lessonCanonHydrator.hydrateLessonCanon({
        id: appState.currentLesson?.id ?? "custom",
        title: displayMeta.title || title,
        description: displayMeta.description || appState.currentLesson?.description || "",
        subject: appState.currentLesson?.subject ?? "",
        grade: appState.currentLesson?.grade ?? "",
        quality: appState.currentLesson?.quality ?? "experimental",
        requiredViews: normalizeViewFamilyList(
          baseMeta.required_views ??
            ddnMetaHeader.requiredViews ??
            ddnMetaHeader.required_views ??
            appState.currentLesson?.requiredViews ??
            [],
        ),
        ddnText,
        maegimControlJson: appState.currentLesson?.maegimControlJson ?? "",
        textMd: appState.currentLesson?.textMd ?? "",
        meta: baseMeta,
        ddnMetaHeader,
      });
      appState.currentLesson = lesson;
      openRunWithLessonWithSource(lesson, {
        launchKind: "editor_run",
        autoExecute: true,
        sourceId: `ddn:block:${lesson.id}`,
        sourceType: "ddn",
        derivedFrom: String(appState.currentLesson?.id ?? ""),
      });
    },
    onOpenAdvanced: () => {
      advanced.toggle();
    },
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
    onSelectLesson: async (lessonId) => {
      const targetId = String(lessonId ?? "").trim();
      if (!targetId) return;
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
      editorScreen.loadLesson(ddnText, { title, readOnly: true });
      setScreen("editor");
    },
    onOpenAdvanced: () => {
      advanced.toggle();
    },
  });

  const advanced = createAdvancedMenu({
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
  });

  browseScreen.init();
  editorScreen.init();
  blockEditorScreen.init();
  runScreen.init();
  document.querySelectorAll(".main-shell-tab[data-main-tab-target]").forEach((button) => {
    button.addEventListener("click", () => {
      const target = normalizeMainTabTarget(button?.dataset?.mainTabTarget, MAIN_TAB_BROWSE);
      void switchMainTab(target, { launchKind: "manual" });
    });
  });
  featuredSeedButton?.addEventListener("click", () => {
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
  if (shouldApplyPlatformRouteFallback(platformRouteSlots)) {
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
    if (studioRouteRequest.lessonId) {
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
    }
  }

  if (!initialScreenApplied) {
    setScreen("browse");
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
