import { createInputRegistryState, serializeInputRegistrySession } from "./input_registry.js";
import {
  buildOverlayCompareSessionPayload,
  buildOverlaySessionRunsPayload,
  buildSessionUiLayoutPayload,
  buildSessionViewComboPayload,
  resolveOverlayCompareFromSession,
  resolveSessionUiLayoutFromPayload,
  resolveSessionViewComboFromPayload,
} from "./overlay_session_contract.js";
import {
  buildSessionLayerUpdateTick,
  buildSnapshotRunUpdateTick,
  sortSessionLayers,
} from "./update_tick_contract.js";
import { normalizeViewFamilyList } from "./view_family_contract.js";

export const SNAPSHOT_V0_SCHEMA = "seamgrim.snapshot.v0";
export const SESSION_V0_SCHEMA = "seamgrim.session.v0";

function toText(raw) {
  return String(raw ?? "").trim();
}

function toFiniteNumber(raw) {
  const num = Number(raw);
  return Number.isFinite(num) ? num : null;
}

function toIsoTimestamp(raw) {
  const text = toText(raw);
  if (!text) return new Date().toISOString();
  const parsed = Date.parse(text);
  if (!Number.isFinite(parsed)) return new Date().toISOString();
  return new Date(parsed).toISOString();
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function toObject(raw, fallback = {}) {
  return raw && typeof raw === "object" ? raw : fallback;
}

function stableHashHex(raw) {
  const text = String(raw ?? "");
  let hash = 0x811c9dc5;
  for (let i = 0; i < text.length; i += 1) {
    hash ^= text.charCodeAt(i);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function stableStringify(value) {
  if (value === null || value === undefined) return "";
  if (typeof value === "number" || typeof value === "boolean") return JSON.stringify(value);
  if (typeof value === "string") return JSON.stringify(value);
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableStringify(item)).join(",")}]`;
  }
  if (typeof value === "object") {
    const keys = Object.keys(value).sort((a, b) => a.localeCompare(b, "ko"));
    const pairs = keys.map((key) => `${JSON.stringify(key)}:${stableStringify(value[key])}`);
    return `{${pairs.join(",")}}`;
  }
  return JSON.stringify(String(value));
}

function normalizeRange(raw) {
  const row = toObject(raw, {});
  const xMin = toFiniteNumber(row.x_min ?? row.xMin);
  const xMax = toFiniteNumber(row.x_max ?? row.xMax);
  const yMin = toFiniteNumber(row.y_min ?? row.yMin);
  const yMax = toFiniteNumber(row.y_max ?? row.yMax);
  if ([xMin, xMax, yMin, yMax].some((value) => value === null)) return null;
  if (xMax <= xMin || yMax <= yMin) return null;
  return {
    xMin,
    xMax,
    yMin,
    yMax,
  };
}

function normalizeSessionSample(raw, fallbackGraph = null) {
  const source = toObject(raw, {});
  const fallback = toObject(fallbackGraph?.sample, {});
  const row = Object.keys(source).length ? source : fallback;
  const varName = toText(row.var ?? row.variable);
  const xMin = toFiniteNumber(row.x_min ?? row.xMin);
  const xMax = toFiniteNumber(row.x_max ?? row.xMax);
  const step = toFiniteNumber(row.step);
  if (!varName || xMin === null || xMax === null || step === null || xMax < xMin || step <= 0) {
    return null;
  }
  return {
    var: varName,
    x_min: xMin,
    x_max: xMax,
    step,
  };
}

function normalizeGraphSample(raw) {
  const row = toObject(raw, {});
  const varName = toText(row.var ?? row.variable);
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

function normalizeGraphView(raw) {
  const row = toObject(raw, {});
  const xMin = toFiniteNumber(row.x_min ?? row.xMin);
  const xMax = toFiniteNumber(row.x_max ?? row.xMax);
  const yMin = toFiniteNumber(row.y_min ?? row.yMin);
  const yMax = toFiniteNumber(row.y_max ?? row.yMax);
  if ([xMin, xMax, yMin, yMax].some((value) => value === null) || xMax <= xMin || yMax <= yMin) {
    return null;
  }
  return {
    auto: Boolean(row.auto),
    x_min: xMin,
    x_max: xMax,
    y_min: yMin,
    y_max: yMax,
    pan_x: toFiniteNumber(row.pan_x ?? row.panX) ?? 0,
    pan_y: toFiniteNumber(row.pan_y ?? row.panY) ?? 0,
    zoom: toFiniteNumber(row.zoom) ?? 1,
  };
}

function resolveRuntimeSessionGraphSample(runtimeSessionState = null) {
  const state = toObject(runtimeSessionState, {});
  return normalizeGraphSample(state.sample);
}

function resolveRuntimeSessionGraphView(runtimeSessionState = null) {
  const state = toObject(runtimeSessionState, {});
  const view = toObject(state.view, {});
  const graph = toObject(view.graph, {});
  const axis = graph.axis ?? graph.range ?? null;
  if (!axis) return null;
  return normalizeGraphView({
    auto: graph.auto_fit ?? graph.autoFit ?? false,
    x_min: axis.x_min ?? axis.xMin,
    x_max: axis.x_max ?? axis.xMax,
    y_min: axis.y_min ?? axis.yMin,
    y_max: axis.y_max ?? axis.yMax,
    pan_x: view.panX ?? view.pan_x ?? 0,
    pan_y: view.panY ?? view.pan_y ?? 0,
    zoom: view.zoom ?? 1,
  });
}

function applyRuntimeSessionGraphHints(graph, runtimeSessionState = null) {
  const source = graph && typeof graph === "object" ? cloneJson(graph) : null;
  if (!source) return null;
  const sample = normalizeGraphSample(source.sample) ?? resolveRuntimeSessionGraphSample(runtimeSessionState);
  const view = normalizeGraphView(source.view) ?? resolveRuntimeSessionGraphView(runtimeSessionState);
  if (sample) source.sample = sample;
  if (view) source.view = view;
  return source;
}

function buildDerivedInputHash({
  ddnText = "",
  runtimeSessionState = null,
  graph = null,
} = {}) {
  const state = toObject(runtimeSessionState, {});
  const controls = toObject(state.controls, {});
  const sample = normalizeGraphSample(graph?.sample) ?? resolveRuntimeSessionGraphSample(state);
  const seed = [
    toText(ddnText),
    stableStringify(controls),
    stableStringify(sample ?? {}),
  ].join("||");
  return stableHashHex(seed);
}

function normalizeSessionTime(raw) {
  const row = toObject(raw, {});
  const tMin = toFiniteNumber(row.t_min ?? row.tMin) ?? 0;
  const tMaxRaw = toFiniteNumber(row.t_max ?? row.tMax);
  const tMax = tMaxRaw !== null && tMaxRaw >= tMin ? tMaxRaw : tMin + 1;
  const stepRaw = toFiniteNumber(row.step);
  const step = stepRaw !== null && stepRaw > 0 ? stepRaw : 1;
  const nowRaw = toFiniteNumber(row.now);
  const now = nowRaw !== null ? nowRaw : tMin;
  const intervalRaw = toFiniteNumber(row.interval);
  const interval = intervalRaw !== null && intervalRaw > 0 ? Math.trunc(intervalRaw) : 300;
  const tickRaw = toFiniteNumber(row.tick);
  const tick = tickRaw !== null && tickRaw >= 0 ? Math.trunc(tickRaw) : 0;
  const cursorRaw = toFiniteNumber(row.cursor);
  const cursor = cursorRaw !== null && cursorRaw >= 0 ? Math.trunc(cursorRaw) : tick;
  const speedRaw = toFiniteNumber(row.speed);
  const speed = speedRaw !== null && speedRaw > 0 ? speedRaw : 1;
  const paused = typeof row.paused === "boolean" ? row.paused : null;
  const playing = typeof row.playing === "boolean" ? row.playing : null;
  return {
    enabled: Boolean(row.enabled),
    t_min: tMin,
    t_max: tMax,
    step,
    now,
    interval,
    loop: row.loop !== false,
    tick,
    cursor,
    speed,
    paused: paused ?? (playing === null ? false : !playing),
    playing: playing ?? !(paused ?? false),
  };
}

function normalizeSessionView(raw) {
  const row = toObject(raw, {});
  const graph = toObject(row.graph, {});
  const space2d = toObject(row.space2d, {});
  const dock = toObject(row.dock, {});
  const graphGuides = toObject(graph.guides, {});
  const space2dGuides = toObject(space2d.guides, {});
  const selectedAxes = toObject(graph.selected_axes ?? graph.selectedAxes, {});
  const rootRange = normalizeRange(row.range);
  const graphRange = normalizeRange(graph.axis ?? graph.range ?? row.graph_range);
  const space2dRange = normalizeRange(space2d.range ?? space2d.axis ?? row.space2d_range);
  return {
    auto: Boolean(row.auto),
    panX: toFiniteNumber(row.panX ?? row.pan_x) ?? 0,
    panY: toFiniteNumber(row.panY ?? row.pan_y) ?? 0,
    zoom: toFiniteNumber(row.zoom) ?? 1,
    range: rootRange,
    showGrid: Boolean(row.showGrid ?? row.show_grid ?? graphGuides.showGrid ?? space2dGuides.showGrid),
    showAxis: Boolean(row.showAxis ?? row.show_axis ?? graphGuides.showAxis ?? space2dGuides.showAxis),
    graph: {
      auto_fit: Boolean(graph.auto_fit ?? graph.autoFit),
      axis: graphRange,
      guides: {
        showGrid: Boolean(graphGuides.showGrid),
        showAxis: Boolean(graphGuides.showAxis),
      },
      selected_axes: {
        x: toText(selectedAxes.x ?? selectedAxes.xKey),
        y: toText(selectedAxes.y ?? selectedAxes.yKey),
      },
    },
    space2d: {
      auto_fit: Boolean(space2d.auto_fit ?? space2d.autoFit),
      range: space2dRange,
      guides: {
        showGrid: Boolean(space2dGuides.showGrid),
        showAxis: Boolean(space2dGuides.showAxis),
      },
    },
    dock: {
      target: toText(dock.target) || "space2d",
      highlight: Boolean(dock.highlight),
    },
  };
}

function cloneOptional(raw) {
  if (raw === null || raw === undefined) return null;
  if (typeof raw === "string") return toText(raw) || null;
  if (typeof raw === "number" || typeof raw === "boolean") return raw;
  if (Array.isArray(raw) || (raw && typeof raw === "object")) return cloneJson(raw);
  return null;
}

function normalizeSessionSpace2dView(raw, view = null) {
  const source = toObject(raw, {});
  const fallback = toObject(view?.space2d, {});
  const row = Object.keys(source).length ? source : fallback;
  return {
    auto_fit: Boolean(row.auto_fit ?? row.autoFit),
    range: normalizeRange(row.range ?? row.axis),
    guides: {
      showGrid: Boolean(row?.guides?.showGrid),
      showAxis: Boolean(row?.guides?.showAxis),
    },
  };
}

function normalizeCursor(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  return {
    id: toText(row.id),
    t: toFiniteNumber(row.t) ?? 0,
    tick: toFiniteNumber(row.tick) ?? 0,
  };
}

function normalizeOverlayRuns(overlayRuns) {
  const source = Array.isArray(overlayRuns) ? overlayRuns : [];
  return source.filter((row) => row && typeof row === "object");
}

function buildFallbackRun({
  lessonId = "",
  lessonTitle = "",
  ddnText = "",
  graph = null,
  runtimeHash = "",
  runtimeSessionState = null,
} = {}) {
  if (!graph || typeof graph !== "object") return null;
  const id = lessonId ? `run:${lessonId}` : "run:runtime";
  const label = toText(lessonTitle) || toText(lessonId) || "runtime";
  const graphWithHints = applyRuntimeSessionGraphHints(graph, runtimeSessionState);
  const graphMeta = graphWithHints?.meta && typeof graphWithHints.meta === "object" ? graphWithHints.meta : {};
  const inputHash = toText(graphMeta.source_input_hash) || buildDerivedInputHash({
    ddnText,
    runtimeSessionState,
    graph: graphWithHints,
  });
  return {
    id,
    label,
    visible: true,
    order: 0,
    source: {
      kind: "ddn",
      text: toText(ddnText),
    },
    inputs: {},
    graph: graphWithHints,
    hash: {
      input: inputHash,
      result: toText(graphMeta.result_hash ?? runtimeHash),
    },
  };
}

function ensureRunsForSessionAndSnapshot({
  lessonId = "",
  lessonTitle = "",
  ddnText = "",
  overlayRuns = [],
  runtimeGraph = null,
  runtimeHash = "",
  runtimeSessionState = null,
} = {}) {
  const runs = normalizeOverlayRuns(overlayRuns);
  if (runs.length > 0) return runs;
  const fallback = buildFallbackRun({
    lessonId,
    lessonTitle,
    ddnText,
    graph: runtimeGraph,
    runtimeHash,
    runtimeSessionState,
  });
  return fallback ? [fallback] : [];
}

export function buildSnapshotV0({
  timestamp = "",
  note = "",
  run = null,
} = {}) {
  const sourceRun = run && typeof run === "object" ? run : {};
  const normalizedRun = buildSnapshotRunUpdateTick(sourceRun);
  return {
    schema: SNAPSHOT_V0_SCHEMA,
    ts: toIsoTimestamp(timestamp),
    note: toText(note),
    run: normalizedRun,
  };
}

export function buildSessionV0({
  timestamp = "",
  lessonId = "",
  ddnText = "",
  requiredViews = [],
  layoutPreset = "",
  inputRegistryState = null,
  overlayRuns = [],
  runtimeGraph = null,
  runtimeHash = "",
  activeRunId = "",
  cursor = null,
  graphRef = "",
  sceneRef = "",
  runtimeSessionState = null,
  overlayCompare = null,
  overlayViewCombo = null,
} = {}) {
  const sessionState = toObject(runtimeSessionState, {});
  const runs = ensureRunsForSessionAndSnapshot({
    lessonId,
    lessonTitle: "",
    ddnText,
    overlayRuns,
    runtimeGraph,
    runtimeHash,
    runtimeSessionState,
  });
  const runsPayload = buildOverlaySessionRunsPayload(runs);
  const compare = resolveOverlayCompareFromSession({
    runs: runsPayload,
    compare: sessionState.compare ?? overlayCompare ?? {},
  });
  const viewCombo = resolveSessionViewComboFromPayload(sessionState.view_combo ?? sessionState.viewCombo ?? overlayViewCombo ?? {});
  const uiLayout = resolveSessionUiLayoutFromPayload(sessionState.ui_layout ?? sessionState.uiLayout ?? {});
  const sessionTime = normalizeSessionTime(sessionState.time);
  const sessionView = normalizeSessionView(sessionState.view);
  const space2dView = normalizeSessionSpace2dView(sessionState.space2d_view ?? sessionState.space2dView, sessionView);
  const sessionSample = normalizeSessionSample(sessionState.sample, runtimeGraph);
  const selectedActiveRunId = toText(sessionState.active_run_id ?? activeRunId) || (runsPayload[0]?.id ?? "");
  const normalizedCursor = normalizeCursor(
    cursor ?? sessionState.cursor ?? {
      id: selectedActiveRunId || toText(lessonId),
      t: sessionTime.now,
      tick: sessionTime.tick,
    },
  );
  const layers = sortSessionLayers(runs.map((run, index) => buildSessionLayerUpdateTick(run, index)));
  const inputRegistry = createInputRegistryState(inputRegistryState);
  const serializedInputs = serializeInputRegistrySession(inputRegistry);
  return {
    schema: SESSION_V0_SCHEMA,
    ts: toIsoTimestamp(timestamp),
    lesson: toText(lessonId),
    ddn_text: toText(ddnText),
    inputs: serializedInputs.inputs,
    layers,
    required_views: normalizeViewFamilyList(requiredViews),
    layout_preset: toText(layoutPreset),
    graph_ref: toText(graphRef),
    scene_ref: toText(sceneRef),
    cursor: normalizedCursor,
    active_run_id: selectedActiveRunId || (layers[0]?.id ?? ""),
    formula_text: toText(sessionState.formula_text ?? sessionState.formulaText),
    controls: cloneOptional(sessionState.controls) ?? {},
    sample: sessionSample,
    time: sessionTime,
    view: sessionView,
    ui_layout: buildSessionUiLayoutPayload(uiLayout),
    view_combo: buildSessionViewComboPayload(viewCombo),
    runs: runsPayload,
    compare: buildOverlayCompareSessionPayload(compare),
    text_doc: cloneOptional(sessionState.text_doc ?? sessionState.textDoc),
    space2d: cloneOptional(sessionState.space2d),
    space2d_view: space2dView,
    table_view: cloneOptional(sessionState.table_view ?? sessionState.tableView),
    structure_view: cloneOptional(sessionState.structure_view ?? sessionState.structureView),
    view_meta: cloneOptional(sessionState.view_meta ?? sessionState.viewMeta),
    last_state_hash: toText(sessionState.last_state_hash ?? runtimeHash),
    last_view_hash: toText(sessionState.last_view_hash ?? sessionState.lastViewHash),
  };
}

export function buildRuntimeSnapshotBundleV0({
  timestamp = "",
  lessonId = "",
  lessonTitle = "",
  ddnText = "",
  requiredViews = [],
  inputRegistryState = null,
  overlayRuns = [],
  runtimeGraph = null,
  runtimeHash = "",
  layoutPreset = "",
  cursor = null,
  graphRef = "",
  sceneRef = "",
  runtimeSessionState = null,
  overlayCompare = null,
  overlayViewCombo = null,
  activeRunId = "",
} = {}) {
  const runs = ensureRunsForSessionAndSnapshot({
    lessonId,
    lessonTitle,
    ddnText,
    overlayRuns,
    runtimeGraph,
    runtimeHash,
    runtimeSessionState,
  });
  const runtimeSessionRow = toObject(runtimeSessionState, {});
  const activeId = toText(activeRunId ?? runtimeSessionState?.active_run_id);
  const hintedRuns = runs.map((run, index) => {
    const row = run && typeof run === "object" ? run : {};
    const runId = toText(row.id);
    const useRuntimeHints = !runId || !activeId || runId === activeId || (index === 0 && !activeId);
    if (!useRuntimeHints) return row;
    return {
      ...row,
      graph: applyRuntimeSessionGraphHints(row.graph ?? runtimeGraph, runtimeSessionRow),
    };
  });
  const primaryRun = (
    hintedRuns.find((run) => toText(run?.id) === activeId) ??
    hintedRuns[0] ??
    {}
  );
  const snapshot = buildSnapshotV0({
    timestamp,
    note: toText(lessonTitle) || toText(lessonId),
    run: primaryRun,
  });
  const session = buildSessionV0({
    timestamp,
    lessonId,
    ddnText,
    requiredViews,
    layoutPreset,
    inputRegistryState,
    overlayRuns: hintedRuns,
    runtimeGraph,
    runtimeHash,
    activeRunId: activeId || toText(primaryRun.id),
    cursor,
    graphRef,
    sceneRef,
    runtimeSessionState,
    overlayCompare,
    overlayViewCombo,
  });
  return { snapshot, session };
}
