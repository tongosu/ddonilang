import { createWasmLoader, applyWasmLogicAndDispatchState } from "./wasm_page_common.js";
import { createLessonCanonHydrator, buildFlatPlanView } from "./runtime/index.js";
import { BrowseScreen } from "./screens/browse.js";
import { EditorScreen, saveDdnToFile } from "./screens/editor.js";
import { BlockEditorScreen } from "./screens/block_editor.js";
import { RunScreen } from "./screens/run.js";
import { normalizeViewFamilyList } from "./view_family_contract.js";
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
  registerFormulaInput,
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

const PROJECT_PREFIX = "solutions/seamgrim_ui_mvp/";
const SIM_CORE_POLICY_CLASS = "policy-sim-core";
const OVERLAY_SESSION_STORAGE_KEY = "seamgrim.overlay_session.v1";
const INPUT_REGISTRY_STORAGE_KEY = "seamgrim.input_registry.v0";
const SCENE_SUMMARY_STORAGE_KEY = "seamgrim.scene_summary.v0";
const SNAPSHOT_V0_STORAGE_KEY = "seamgrim.snapshot.v0";
const SESSION_V0_STORAGE_KEY = "seamgrim.session.v0";
const BROWSE_PRESET_QUERY_KEY = "browsePreset";
const WASM_CANON_RUNTIME_URL = "./runtime/../wasm/ddonirang_tool.js";

const appState = {
  currentLesson: null,
  currentScreen: "browse",
  wasm: {
    enabled: true,
    loader: null,
    client: null,
    parseWarnings: [],
    fpsLimit: 30,
    dtMax: 0.1,
    langMode: "strict",
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

function trackFormulaInputSource({
  lessonId = "",
  lessonTitle = "",
  formulaText = "",
  derivedDdn = "",
} = {}) {
  const normalizedLessonId = String(lessonId ?? "").trim();
  const normalizedId = normalizedLessonId ? `formula:${normalizedLessonId}` : "formula:main";
  const normalizedTitle = String(lessonTitle ?? "").trim();
  appState.inputRegistry = registerFormulaInput(appState.inputRegistry, {
    id: normalizedId,
    label: normalizedTitle ? `수식 · ${normalizedTitle}` : "수식",
    formulaText: String(formulaText ?? ""),
    derivedDdn: String(derivedDdn ?? ""),
  });
}

function setScreen(name) {
  ["browse", "editor", "block_editor", "run"].forEach((screenName) => {
    const node = byId(`screen-${screenName}`);
    if (!node) return;
    node.classList.toggle("hidden", screenName !== name);
  });
  appState.currentScreen = name;
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

function mergeLessonEntry(map, nextEntry) {
  if (!nextEntry) return;
  const existing = map.get(nextEntry.id);
  if (!existing) {
    map.set(nextEntry.id, nextEntry);
    return;
  }

  const merged = {
    ...existing,
    ...nextEntry,
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

async function loadCatalogLessons() {
  const merged = new Map();

  const inventoryApi = await fetchFirstOk(["/api/lessons/inventory", "/api/lesson-inventory"], "json");
  if (inventoryApi.ok) {
    mergeCatalogFromInventoryPayload(merged, inventoryApi.data);
  }

  if (merged.size === 0) {
    const indexJson = await fetchJson("solutions/seamgrim_ui_mvp/lessons/index.json");
    const indexLessons = Array.isArray(indexJson?.lessons) ? indexJson.lessons : [];
    indexLessons.forEach((row) => {
      const id = String(row.id ?? "").trim();
      if (!id) return;
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

function createAdvancedMenu({ onSmoke }) {
  const menu = byId("advanced-menu");
  const smokeBtn = byId("advanced-smoke");

  smokeBtn?.addEventListener("click", async () => {
    menu?.classList.add("hidden");
    if (typeof onSmoke === "function") {
      await onSmoke();
    }
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
  const allowShapeFallback = readWindowBoolean("SEAMGRIM_ENABLE_SHAPE_FALLBACK", false);
  const federatedFileCandidates = allowFederatedFileFallback
    ? readWindowStringArray("SEAMGRIM_FEDERATED_FILE_CANDIDATES", [])
    : [];
  const browsePresetFromLocation = readBrowsePresetFromLocation();
  const featuredSeedButton = byId("btn-run-featured-seed");
  let runScreen = null;
  let editorScreen = null;
  let blockEditorScreen = null;

  const refreshEditorCanonSummary = async (ddnText) => {
    if (!editorScreen) return;
    const sourceText = String(ddnText ?? "");
    const ticket = editorCanonSummaryTicket + 1;
    editorCanonSummaryTicket = ticket;
    if (!sourceText.trim()) {
      editorScreen.setCanonFlatView(null);
      return;
    }
    editorScreen.setCanonFlatView({
      summaryText: "구성: WASM canon 계산 중...",
      topoOrder: [],
      instances: [],
      links: [],
    });
    const flat = await lessonCanonHydrator.deriveFlatJson(sourceText, { quiet: true });
    if (ticket !== editorCanonSummaryTicket || !editorScreen) return;
    if (!flat) {
      const canonDiag = Array.isArray(lessonCanonHydrator.getCanonDiags?.())
        ? lessonCanonHydrator.getCanonDiags()
        : [];
      const runtimeDiag = Array.isArray(lessonCanonHydrator.getRuntimeDiags?.())
        ? lessonCanonHydrator.getRuntimeDiags()
        : [];
      const topDiag = canonDiag[0] ?? runtimeDiag[0] ?? null;
      if (topDiag && topDiag.code) {
        editorScreen.setCanonFlatView({
          summaryText: `구성: WASM canon 실패 (${String(topDiag.code)})`,
          topoOrder: [],
          instances: [],
          links: [],
        });
        return;
      }
    }
    editorScreen.setCanonFlatView(buildFlatPlanView(flat));
  };

  const resolveFeaturedSeedIds = () =>
    resolveAvailableFeaturedSeedIds(FEATURED_SEED_IDS, appState.lessonsById);

  const updateFeaturedSeedQuickAction = () => {
    if (!featuredSeedButton) return;
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
    { launchKind = "manual", sourceId = "", sourceType = "lesson", derivedFrom = "" } = {},
  ) => {
    if (!runScreen) return;
    if (String(sourceType) === "ddn") {
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
    runScreen.setLessonOptions(Array.from(appState.lessonsById.values()));
    runScreen.loadLesson(lesson, { launchKind });
    persistSceneSummarySnapshot(runScreen);
    persistRuntimeSnapshotBundleV0(runScreen);
    setScreen("run");
    updateFeaturedSeedQuickAction();
  };

  const openRunWithLesson = (lesson, { launchKind = "manual" } = {}) => {
    openRunWithLessonWithSource(lesson, { launchKind });
  };

  const pickNextFeaturedSeedId = () => {
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
    const targetId = pickNextFeaturedSeedId();
    if (!targetId) {
      updateFeaturedSeedQuickAction();
      return false;
    }
    const lesson = await loadLessonById(targetId);
    openRunWithLesson(lesson, { launchKind: "featured_seed_quick" });
    return true;
  };

  const browseScreen = new BrowseScreen({
    root: byId("screen-browse"),
    federatedApiCandidates,
    federatedFileCandidates,
    onLessonSelect: async (selection) => {
      const lessonId = ensureLessonEntryFromSelection(selection);
      if (!lessonId) {
        throw new Error("교과를 찾지 못했습니다: invalid lesson selection");
      }
      const lesson = await loadLessonById(lessonId);
      openRunWithLessonWithSource(lesson, { launchKind: "browse_select", sourceId: `lesson:${lesson.id}` });
    },
    onCreate: () => {
      editorScreen.loadBlank();
      setScreen("editor");
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

  editorScreen = new EditorScreen({
    root: byId("screen-editor"),
    onBack: () => {
      setScreen("browse");
    },
    onRun: async (ddnText) => {
      editorScreen.setSmokeResult("WASM 매김 계획 계산 중...");
      const baseMeta = appState.currentLesson?.meta ?? {};
      const ddnMetaHeader = parseLessonDdnMetaHeader(ddnText);
      const displayMeta = resolveLessonDisplayMeta({
        baseTitle: appState.currentLesson?.title ?? "사용자 DDN",
        baseDescription: appState.currentLesson?.description ?? "",
        tomlMeta: baseMeta,
        ddnMetaHeader,
      });
      const lesson = await lessonCanonHydrator.hydrateLessonCanon({
        id: appState.currentLesson?.id ?? "custom",
        title: displayMeta.title || appState.currentLesson?.title || "사용자 DDN",
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
        sourceId: `ddn:editor:${lesson.id}`,
        sourceType: "ddn",
        derivedFrom: String(appState.currentLesson?.id ?? ""),
      });
    },
    onSave: (ddnText) => {
      saveDdnToFile(ddnText, "lesson.ddn");
      editorScreen.setSmokeResult("DDN 파일 저장 완료");
    },
    onOpenAdvanced: () => {
      advanced.toggle();
    },
    onSourceChange: (ddnText) => {
      void refreshEditorCanonSummary(ddnText);
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
    onSaveSnapshot: () => persistRuntimeSnapshotV0(runScreen),
    onSaveSession: () => persistRuntimeSessionV0(runScreen),
    onFormulaApplied: ({ formulaText = "", derivedDdn = "" } = {}) => {
      const lessonId = String(runScreen?.lesson?.id ?? appState.currentLesson?.id ?? "").trim();
      const lessonTitle = String(runScreen?.lesson?.title ?? appState.currentLesson?.title ?? lessonId).trim();
      trackFormulaInputSource({
        lessonId,
        lessonTitle,
        formulaText,
        derivedDdn,
      });
      persistInputRegistrySnapshot();
      persistSceneSummarySnapshot(runScreen);
      persistRuntimeSnapshotBundleV0(runScreen);
    },
    onSelectLesson: async (lessonId) => {
      const targetId = String(lessonId ?? "").trim();
      if (!targetId) return;
      const lesson = await loadLessonById(targetId);
      openRunWithLessonWithSource(lesson, { launchKind: "browse_select", sourceId: `lesson:${lesson.id}` });
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
  featuredSeedButton?.addEventListener("click", () => {
    void runNextFeaturedSeed();
  });
  if (typeof window !== "undefined" && window?.addEventListener) {
    window.addEventListener("seamgrim:browse-preset-changed", (event) => {
      const presetId = normalizeBrowsePresetId(event?.detail?.presetId ?? "");
      syncBrowsePresetToLocation(presetId);
    });
    window.addEventListener("keydown", (event) => {
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

  setScreen("browse");
  window.addEventListener("beforeunload", () => {
    runScreen?.flushOverlaySession?.();
    persistSceneSummarySnapshot(runScreen);
    persistRuntimeSnapshotBundleV0(runScreen);
    persistOverlaySessionSnapshot();
    persistInputRegistrySnapshot();
  });
}

void main();
