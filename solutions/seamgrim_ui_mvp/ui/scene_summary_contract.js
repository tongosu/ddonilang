import { createInputRegistryState, getSelectedInputRegistryItem } from "./input_registry.js";
import { resolveGraphUpdateTick } from "./update_tick_contract.js";
import { normalizeViewFamilyList } from "./view_family_contract.js";

export const SCENE_SUMMARY_SCHEMA = "seamgrim.scene.v0";

function toFiniteNumber(raw) {
  const num = Number(raw);
  return Number.isFinite(num) ? num : null;
}

function toInteger(raw, fallback = 0) {
  const num = Number(raw);
  if (!Number.isFinite(num)) return fallback;
  return Math.trunc(num);
}

function toText(raw) {
  return String(raw ?? "").trim();
}

function normalizeTimestamp(raw) {
  const text = toText(raw);
  if (!text) return new Date().toISOString();
  const parsed = Date.parse(text);
  if (!Number.isFinite(parsed)) return new Date().toISOString();
  return new Date(parsed).toISOString();
}

function readObservationTimeNow(observation) {
  const row = observation && typeof observation === "object" ? observation : {};
  const values = row.all_values && typeof row.all_values === "object"
    ? row.all_values
    : row.values && typeof row.values === "object"
      ? row.values
      : {};
  const keys = ["t", "time", "tick", "프레임수", "시간"];
  for (const key of keys) {
    const hit = toFiniteNumber(values[key]);
    if (hit !== null) return hit;
  }
  return null;
}

function readGraphSeries(graph) {
  const row = graph && typeof graph === "object" ? graph : {};
  return Array.isArray(row.series) ? row.series : [];
}

function readPrimarySeries(graph) {
  const series = readGraphSeries(graph);
  if (!series.length) return null;
  return (
    series.find((entry) => Array.isArray(entry?.points) && entry.points.length > 0) ??
    series[0] ??
    null
  );
}

function readSeriesId(series) {
  return toText(series?.id ?? series?.name ?? series?.label);
}

function readSeriesPoints(series) {
  const points = Array.isArray(series?.points) ? series.points : [];
  return points
    .map((row) => ({ x: toFiniteNumber(row?.x), y: toFiniteNumber(row?.y) }))
    .filter((row) => row.x !== null && row.y !== null);
}

function inferTimelineFromRuntime(runtimeDerived) {
  const graph = runtimeDerived?.views?.graph ?? null;
  const primarySeries = readPrimarySeries(graph);
  const points = readSeriesPoints(primarySeries);
  if (!points.length) return null;

  const xValues = points.map((row) => Number(row.x));
  const tMin = Math.min(...xValues);
  const tMax = Math.max(...xValues);

  let step = null;
  for (let i = 1; i < xValues.length; i += 1) {
    const diff = Math.abs(xValues[i] - xValues[i - 1]);
    if (diff > 0) {
      step = diff;
      break;
    }
  }

  const now = readObservationTimeNow(runtimeDerived?.observation);
  return {
    t_min: tMin,
    t_max: tMax,
    step,
    now: now === null ? tMax : now,
    playing: false,
    frame_count: xValues.length,
    frame_sample: xValues.slice(0, 5),
  };
}

function normalizeTimeSummary(raw) {
  if (!raw || typeof raw !== "object") return null;
  const tMin = toFiniteNumber(raw.t_min);
  const tMax = toFiniteNumber(raw.t_max);
  if (tMin === null || tMax === null) return null;
  return {
    t_min: tMin,
    t_max: tMax,
    step: toFiniteNumber(raw.step),
    now: toFiniteNumber(raw.now),
    playing: Boolean(raw.playing),
    frame_count: Math.max(0, toInteger(raw.frame_count, 0)),
    frame_sample: Array.isArray(raw.frame_sample)
      ? raw.frame_sample.map((value) => toFiniteNumber(value)).filter((value) => value !== null)
      : [],
  };
}

function summarizeInputRegistry(inputRegistryState) {
  const state = createInputRegistryState(inputRegistryState);
  const selected = getSelectedInputRegistryItem(state);
  const registrySummary = state.registry
    .map((entry) => ({
      id: toText(entry?.id),
      type: toText(entry?.type),
      label: toText(entry?.label),
    }))
    .filter((entry) => entry.id)
    .sort((a, b) => a.id.localeCompare(b.id, "ko"));

  const payload = selected?.payload && typeof selected.payload === "object" ? selected.payload : {};
  return {
    selected_id: toText(state.selectedId),
    registry_summary: registrySummary,
    kind: toText(selected?.type),
    label: toText(selected?.label),
    input_hash: toText(payload.input_hash ?? payload.source_input_hash),
    result_hash: toText(payload.result_hash),
  };
}

function summarizeLayer(raw, index = 0) {
  const row = raw && typeof raw === "object" ? raw : {};
  const graph = row.graph && typeof row.graph === "object" ? row.graph : {};
  const updateTick = resolveGraphUpdateTick(graph);
  const primarySeries = readPrimarySeries(graph);
  const points = readSeriesPoints(primarySeries);
  return {
    id: toText(row.id) || `run-${index + 1}`,
    label: toText(row.label) || toText(row.id) || `run-${index + 1}`,
    visible: row.visible !== false,
    order: toInteger(row.order ?? row.layer_index ?? row.layerIndex, index),
    opacity: toFiniteNumber(row.opacity) ?? 1,
    update: updateTick.update,
    tick: updateTick.tick,
    ticks: updateTick.ticks,
    series_id: readSeriesId(primarySeries) || toText(row.series_id),
    points: points.length,
  };
}

function summarizeFallbackLayer(runtimeDerived, { lessonId = "", lessonTitle = "" } = {}) {
  const graph = runtimeDerived?.views?.graph ?? null;
  const primarySeries = readPrimarySeries(graph);
  if (!primarySeries) return [];
  const updateTick = resolveGraphUpdateTick(graph);
  const points = readSeriesPoints(primarySeries);
  const id = lessonId ? `run:${lessonId}` : "run:runtime";
  const label = toText(lessonTitle) || toText(lessonId) || "runtime";
  return [
    {
      id,
      label,
      visible: true,
      order: 0,
      opacity: 1,
      update: updateTick.update,
      tick: updateTick.tick,
      ticks: updateTick.ticks,
      series_id: readSeriesId(primarySeries),
      points: points.length,
    },
  ];
}

function summarizeLayers(overlayRuns, runtimeDerived, lessonInfo) {
  const source = Array.isArray(overlayRuns) ? overlayRuns : [];
  const rows = source.map((run, index) => summarizeLayer(run, index)).filter((row) => row.id);
  const picked = rows.length ? rows : summarizeFallbackLayer(runtimeDerived, lessonInfo);
  return picked.sort((a, b) => {
    if (a.order !== b.order) return a.order - b.order;
    return a.id.localeCompare(b.id, "ko");
  });
}

function selectViewKind(requiredViews, runtimeDerived) {
  const runtimeViews = runtimeDerived?.views && typeof runtimeDerived.views === "object"
    ? runtimeDerived.views
    : {};
  const hasGraph = Array.isArray(runtimeViews?.graph?.series) && runtimeViews.graph.series.length > 0;
  if (hasGraph) return "view-graph";
  if (runtimeViews.space2d) return "view-2d";
  if (runtimeViews.table) return "view-table";
  if (runtimeViews.text || runtimeViews.structure) return "view-text";

  const families = normalizeViewFamilyList(requiredViews);
  if (families.includes("graph")) return "view-graph";
  if (families.some((family) => family === "space2d" || family === "grid2d")) return "view-2d";
  if (families.includes("table")) return "view-table";
  if (families.includes("text") || families.includes("structure")) return "view-text";
  return "view-graph";
}

function buildViewConfig(runtimeDerived) {
  const graphAxis = runtimeDerived?.views?.graph?.axis ?? {};
  return {
    range: {
      x_min: toFiniteNumber(graphAxis.x_min),
      x_max: toFiniteNumber(graphAxis.x_max),
      y_min: toFiniteNumber(graphAxis.y_min),
      y_max: toFiniteNumber(graphAxis.y_max),
    },
    pan_x: 0,
    pan_y: 0,
    zoom: 1,
    grid: true,
    axis: true,
  };
}

function resolveHashes({ runtimeDerived, runtimeHash, inputSummary }) {
  const graphMeta = runtimeDerived?.views?.graph?.meta ?? {};
  const inputHash = toText(
    graphMeta.source_input_hash ??
      inputSummary.input_hash,
  );
  const resultHash = toText(
    graphMeta.result_hash ??
      runtimeHash ??
      inputSummary.result_hash,
  );
  return {
    input_hash: inputHash,
    result_hash: resultHash,
  };
}

function normalizeBogaeScene(raw) {
  if (raw === null || raw === undefined) return null;
  if (!Array.isArray(raw)) return null;
  return raw.map((row) => (row && typeof row === "object" ? { ...row } : row));
}

export function buildSceneSummarySnapshot({
  timestamp = "",
  lessonId = "",
  lessonTitle = "",
  requiredViews = [],
  inputRegistryState = null,
  overlayRuns = [],
  runtimeDerived = null,
  runtimeHash = "",
  view = null,
  time = undefined,
  bogaeScene = null,
} = {}) {
  const normalizedRequiredViews = normalizeViewFamilyList(requiredViews);
  const inputSummary = summarizeInputRegistry(inputRegistryState);
  const hashes = resolveHashes({
    runtimeDerived,
    runtimeHash,
    inputSummary,
  });
  const layers = summarizeLayers(overlayRuns, runtimeDerived, { lessonId, lessonTitle });
  const viewSummary = view && typeof view === "object"
    ? {
      kind: toText(view.kind) || selectViewKind(normalizedRequiredViews, runtimeDerived),
      config: view.config && typeof view.config === "object" ? { ...view.config } : buildViewConfig(runtimeDerived),
    }
    : {
      kind: selectViewKind(normalizedRequiredViews, runtimeDerived),
      config: buildViewConfig(runtimeDerived),
    };
  const timeSummary = time === undefined
    ? inferTimelineFromRuntime(runtimeDerived)
    : normalizeTimeSummary(time);

  return {
    schema: SCENE_SUMMARY_SCHEMA,
    ts: normalizeTimestamp(timestamp),
    view: viewSummary,
    inputs: {
      selected_id: inputSummary.selected_id,
      registry_summary: inputSummary.registry_summary,
      kind: inputSummary.kind,
      label: inputSummary.label,
      input_hash: hashes.input_hash,
      result_hash: hashes.result_hash,
    },
    required_views: normalizedRequiredViews,
    layers,
    time: timeSummary,
    hashes,
    bogae_scene: normalizeBogaeScene(bogaeScene),
  };
}

export function serializeSceneSummarySession(summary) {
  return {
    schema: SCENE_SUMMARY_SCHEMA,
    scene: summary && typeof summary === "object"
      ? JSON.parse(JSON.stringify(summary))
      : null,
  };
}

export function restoreSceneSummarySession(payload) {
  const row = payload && typeof payload === "object" ? payload : {};
  const scene = row.scene && typeof row.scene === "object" ? row.scene : row;
  if (toText(scene.schema) !== SCENE_SUMMARY_SCHEMA) return null;
  return JSON.parse(JSON.stringify(scene));
}
