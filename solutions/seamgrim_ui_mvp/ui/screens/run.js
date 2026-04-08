import {
  applyWasmLogicAndDispatchState,
  createManagedRafStepLoop,
  formatObservationCellValue,
  readWasmClientParseWarnings,
  stepWasmClientParsed,
} from "../wasm_page_common.js";
import {
  extractObservationChannelsFromState,
  extractStructuredViewsFromState,
} from "../seamgrim_runtime_state.js";
import { applyControlValuesToDdnText } from "../components/control_parser.js";
import { Bogae } from "../components/bogae.js";
import { DotbogiPanel } from "../components/dotbogi.js";
import {
  buildGraphPreviewHtml,
  buildGraphSummaryMarkdown,
} from "../components/graph_preview.js";
import {
  buildStructurePreviewHtml,
  buildStructureSummaryMarkdown,
} from "../components/structure_preview.js";
import {
  hasSpatialViewFamily,
  normalizeViewFamilyList,
  resolveRunDockPanelOrderFromFamilies,
} from "../view_family_contract.js";
import {
  buildFamilyPreviewResult,
  buildPreviewResultCollection,
} from "../preview_result_contract.js";
import {
  applyPreviewViewModelMetadata,
  buildPreviewViewModel,
} from "../preview_view_model.js";
import { SliderPanel } from "../components/slider_panel.js";
import { OverlayDescription } from "../components/overlay.js";
import { markdownToHtml } from "../components/markdown.js";
import { preprocessDdnText } from "../runtime/ddn_preprocess.js";
import { buildInspectorReport, formatInspectorReportText } from "../inspector_contract.js";
import {
  applyFormulaDdnToSource,
  buildFormulaSugarDdn,
  parseFormulaSugarDraft,
} from "../formula_sugar.js";

export {
  applyFormulaDdnToSource,
  buildFormulaSugarDdn,
  parseFormulaSugarDraft,
} from "../formula_sugar.js";

const RUN_UI_PREFS_STORAGE_KEY = "seamgrim.ui.run_prefs.v1";
const RUNTIME_TABLE_CELL_MAX_CHARS = 24;
const RUNTIME_TABLE_CELL_MIN_CHARS = 12;
const RUNTIME_TABLE_CELL_MAX_CHARS_LIMIT = 48;
const RUNTIME_TABLE_CELL_ESTIMATED_PX_PER_CHAR = 7;
const RUNTIME_TABLE_CELL_WIDTH_PADDING_PX = 80;
const RUNTIME_INPUT_MASK_LIMIT = (1 << 9) - 1;
const RUNTIME_INPUT_BITS = Object.freeze({
  ArrowUp: 1 << 0,
  ArrowLeft: 1 << 1,
  ArrowDown: 1 << 2,
  ArrowRight: 1 << 3,
  Space: 1 << 4,
  Enter: 1 << 5,
  Escape: 1 << 6,
  KeyZ: 1 << 7,
  KeyX: 1 << 8,
});
const RUN_TAB_IDS = Object.freeze(["lesson", "ddn", "formula", "inspector"]);
const RUN_DOCK_TARGETS = Object.freeze(["space2d", "graph"]);
const RUN_DOCK_SPEEDS = Object.freeze([0.5, 1, 1.5, 2]);
const RUN_MANAGER_MAX_RUNS = 12;

function normalizeRuntimeInputToken(raw = "") {
  const text = String(raw ?? "").trim();
  if (!text) return "";
  return text;
}

function runtimeInputBitFromToken(token) {
  const normalized = normalizeRuntimeInputToken(token);
  if (!normalized) return 0;
  return Number(RUNTIME_INPUT_BITS[normalized] ?? 0);
}

function runtimeInputTokenFromKeyboardEvent(event) {
  const code = normalizeRuntimeInputToken(event?.code ?? "");
  if (code && Object.prototype.hasOwnProperty.call(RUNTIME_INPUT_BITS, code)) {
    return code;
  }
  const key = String(event?.key ?? "").trim();
  if (!key) return "";
  const lower = key.toLowerCase();
  if (lower === "arrowup" || lower === "up") return "ArrowUp";
  if (lower === "arrowleft" || lower === "left") return "ArrowLeft";
  if (lower === "arrowdown" || lower === "down") return "ArrowDown";
  if (lower === "arrowright" || lower === "right") return "ArrowRight";
  if (lower === " " || lower === "space" || lower === "spacebar") return "Space";
  if (lower === "enter") return "Enter";
  if (lower === "escape" || lower === "esc") return "Escape";
  if (lower === "z") return "KeyZ";
  if (lower === "x") return "KeyX";
  return "";
}

function readStorageJson(key, fallback) {
  try {
    if (typeof window === "undefined" || !window.localStorage) return fallback;
    const raw = window.localStorage.getItem(key);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed : fallback;
  } catch (_) {
    return fallback;
  }
}

function writeStorageJson(key, value) {
  try {
    if (typeof window === "undefined" || !window.localStorage) return;
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch (_) {
    // ignore storage write errors
  }
}

function normalizeRunTab(raw) {
  const tab = String(raw ?? "").trim().toLowerCase();
  if (RUN_TAB_IDS.includes(tab)) return tab;
  return RUN_TAB_IDS[0];
}

function normalizeDockTarget(raw) {
  const target = String(raw ?? "").trim().toLowerCase();
  if (RUN_DOCK_TARGETS.includes(target)) return target;
  return "space2d";
}

function normalizeDockSpeed(raw) {
  const value = Number(raw);
  if (Number.isFinite(value) && RUN_DOCK_SPEEDS.includes(value)) return value;
  return 1;
}

function toPlainObject(raw, fallback = {}) {
  return raw && typeof raw === "object" ? raw : fallback;
}

function escapeHtml(raw) {
  return String(raw ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function shortHash(raw, max = 8) {
  const text = String(raw ?? "").trim();
  if (!text) return "-";
  return text.length > max ? text.slice(0, max) : text;
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

function stableColorHue(label, hash) {
  const seed = `${String(label ?? "").trim()}|${String(hash ?? "").trim()}`;
  const hashed = Number.parseInt(stableHashHex(seed), 16);
  if (!Number.isFinite(hashed)) return 200;
  return hashed % 360;
}

function buildRunColor(hue, alpha = 1) {
  const h = Math.max(0, Math.min(359, Number(hue) || 0));
  const a = Number(alpha);
  if (!Number.isFinite(a) || a >= 1) {
    return `hsl(${h} 76% 56%)`;
  }
  const clamped = Math.max(0.06, Math.min(1, a));
  return `hsl(${h} 76% 56% / ${clamped})`;
}

function normalizeGraphSeriesPoints(points) {
  const source = Array.isArray(points) ? points : [];
  return source
    .map((row) => {
      const x = finiteNumber(row?.x);
      const y = finiteNumber(row?.y);
      if (x === null || y === null) return null;
      return { x, y };
    })
    .filter(Boolean);
}

function normalizeGraphSample(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  const varName = String(row.var ?? row.variable ?? "").trim();
  const xMin = finiteNumber(row.x_min ?? row.xMin);
  const xMax = finiteNumber(row.x_max ?? row.xMax);
  const step = finiteNumber(row.step);
  if (!varName || xMin === null || xMax === null || step === null || xMax < xMin || step <= 0) {
    return null;
  }
  const tick = finiteNumber(row.tick);
  return {
    var: varName,
    x_min: xMin,
    x_max: xMax,
    step,
    ...(tick === null ? {} : { tick }),
  };
}

function normalizeGraphView(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  const xMin = finiteNumber(row.x_min ?? row.xMin);
  const xMax = finiteNumber(row.x_max ?? row.xMax);
  const yMin = finiteNumber(row.y_min ?? row.yMin);
  const yMax = finiteNumber(row.y_max ?? row.yMax);
  if ([xMin, xMax, yMin, yMax].some((value) => value === null) || xMax <= xMin || yMax <= yMin) {
    return null;
  }
  return {
    auto: Boolean(row.auto),
    x_min: xMin,
    x_max: xMax,
    y_min: yMin,
    y_max: yMax,
    pan_x: finiteNumber(row.pan_x ?? row.panX) ?? 0,
    pan_y: finiteNumber(row.pan_y ?? row.panY) ?? 0,
    zoom: finiteNumber(row.zoom) ?? 1,
  };
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

function buildRunInputHash({ ddnText = "", controls = {}, sample = null } = {}) {
  const seed = [
    String(ddnText ?? "").trim(),
    stableStringify(controls ?? {}),
    stableStringify(sample ?? {}),
  ].join("||");
  return stableHashHex(seed);
}

function cloneGraphForRunManager(rawGraph) {
  const graph = rawGraph && typeof rawGraph === "object" ? rawGraph : null;
  if (!graph) return null;
  const axisRaw = graph.axis && typeof graph.axis === "object" ? graph.axis : null;
  const axis = axisRaw
    ? {
      x_min: finiteNumber(axisRaw.x_min),
      x_max: finiteNumber(axisRaw.x_max),
      y_min: finiteNumber(axisRaw.y_min),
      y_max: finiteNumber(axisRaw.y_max),
    }
    : null;
  const seriesRaw = Array.isArray(graph.series) ? graph.series : [];
  const series = seriesRaw
    .map((row, index) => {
      const points = normalizeGraphSeriesPoints(row?.points);
      if (!points.length) return null;
      return {
        id: String(row?.id ?? row?.label ?? `series_${index + 1}`).trim() || `series_${index + 1}`,
        points,
      };
    })
    .filter(Boolean);
  if (!series.length) return null;
  const meta = toPlainObject(graph.meta, {});
  const schema = String(graph.schema ?? "").trim();
  const sample = normalizeGraphSample(graph.sample ?? null);
  const view = normalizeGraphView(graph.view ?? null);
  return {
    ...(schema ? { schema } : {}),
    axis: axis && Object.values(axis).every((value) => Number.isFinite(value)) ? axis : null,
    ...(sample ? { sample } : {}),
    ...(view ? { view } : {}),
    series,
    meta: { ...meta },
  };
}

function pickPrimarySeriesFromGraph(graph) {
  const series = Array.isArray(graph?.series) ? graph.series : [];
  if (!series.length) return null;
  return series.find((row) => Array.isArray(row?.points) && row.points.length > 0) ?? null;
}

function normalizeRunManagerLayer(raw, fallback = 0) {
  const num = Number(raw);
  if (!Number.isFinite(num)) return fallback;
  return Math.max(0, Math.trunc(num));
}

function formatAxisRange(range) {
  if (!range || typeof range !== "object") return "-";
  const xMin = Number(range.x_min ?? range.xMin);
  const xMax = Number(range.x_max ?? range.xMax);
  const yMin = Number(range.y_min ?? range.yMin);
  const yMax = Number(range.y_max ?? range.yMax);
  if (![xMin, xMax, yMin, yMax].every(Number.isFinite) || xMax <= xMin || yMax <= yMin) {
    return "-";
  }
  return `x[${formatStatusNumber(xMin, 2)}, ${formatStatusNumber(xMax, 2)}] y[${formatStatusNumber(yMin, 2)}, ${formatStatusNumber(yMax, 2)}]`;
}

function normalizeRunRequiredViews(requiredViews) {
  return normalizeViewFamilyList(requiredViews);
}

export function resolveRunLayoutProfile(requiredViews) {
  const families = normalizeRunRequiredViews(requiredViews);
  const hasSpatial = hasSpatialViewFamily(families);
  const hasGraph = families.includes("graph");
  const hasTable = families.includes("table");
  const hasTextual = families.includes("text") || families.includes("structure");

  let mode = "split";
  if (!hasSpatial && (hasGraph || hasTable || hasTextual)) {
    mode = "dock_only";
  } else if (hasSpatial && !hasGraph && !hasTable && !hasTextual) {
    mode = "space_primary";
  }

  return {
    mode,
    families,
    hasSpatial,
    hasGraph,
    hasTable,
    hasTextual,
  };
}

export function resolveRunDockPanelOrder(requiredViews) {
  return resolveRunDockPanelOrderFromFamilies(requiredViews);
}

function hasRuntimeGraphContent(runtimeDerived) {
  const observation = runtimeDerived?.observation ?? null;
  const channels = Array.isArray(observation?.channels) ? observation.channels.length : 0;
  if (channels > 0) return true;
  const graph = runtimeDerived?.views?.graph;
  const series = Array.isArray(graph?.series) ? graph.series : [];
  return series.some((row) => Array.isArray(row?.points) && row.points.length > 0);
}

function hasRuntimeTableContent(runtimeDerived) {
  return Boolean(normalizeRuntimeTableView(runtimeDerived?.views?.table ?? null));
}

function hasRuntimeTextContent(markdown) {
  return Boolean(String(markdown ?? "").trim());
}

export function resolveRunDockPanelVisibility(requiredViews, { runtimeDerived = null, textMarkdown = "" } = {}) {
  const families = new Set(normalizeRunRequiredViews(requiredViews));
  return {
    graph: families.has("graph") || hasRuntimeGraphContent(runtimeDerived),
    table: families.has("table") || hasRuntimeTableContent(runtimeDerived),
    text: families.has("text") || families.has("structure") || hasRuntimeTextContent(textMarkdown),
  };
}

function deriveRunKindAndChannels({ observation = null, hasSpace2d = false } = {}) {
  const channels = Array.isArray(observation?.channels) ? observation.channels.length : 0;
  if (!hasSpace2d && channels <= 0) {
    return { kind: "empty", channels };
  }
  if (!hasSpace2d) {
    return { kind: "obs_only", channels };
  }
  return { kind: "space2d", channels };
}

function hasSpace2dDrawable(space2d) {
  if (!space2d || typeof space2d !== "object") return false;
  const shapes = Array.isArray(space2d.shapes) ? space2d.shapes.length : 0;
  const points = Array.isArray(space2d.points) ? space2d.points.length : 0;
  const drawlist = Array.isArray(space2d.drawlist) ? space2d.drawlist.length : 0;
  return shapes > 0 || points > 0 || drawlist > 0;
}

function parseFiniteNumericValue(raw, depth = 0) {
  if (depth > 2) return null;
  if (raw === null || raw === undefined) return null;
  if (typeof raw === "number") {
    return Number.isFinite(raw) ? raw : null;
  }
  if (typeof raw === "string") {
    const text = raw.trim();
    if (!text) return null;
    const num = Number(text);
    return Number.isFinite(num) ? num : null;
  }
  if (Array.isArray(raw)) {
    for (const item of raw) {
      const hit = parseFiniteNumericValue(item, depth + 1);
      if (hit !== null) return hit;
    }
    return null;
  }
  if (typeof raw === "object") {
    const fields = ["value", "num", "number", "raw", "f64", "i64", "fixed64", "scalar"];
    for (const field of fields) {
      if (!Object.prototype.hasOwnProperty.call(raw, field)) continue;
      const hit = parseFiniteNumericValue(raw[field], depth + 1);
      if (hit !== null) return hit;
    }
  }
  return null;
}

function readObservationChannelKey(channel) {
  if (typeof channel === "string") return channel.trim();
  if (!channel || typeof channel !== "object") return "";
  const direct = String(channel.key ?? "").trim();
  if (direct) return direct;
  return String(channel.name ?? channel.id ?? channel.label ?? channel.token ?? "").trim();
}

function readObservationValueEntries(observation) {
  const valuesSource = observation && typeof observation.all_values === "object"
    ? observation.all_values
    : observation && typeof observation.values === "object"
      ? observation.values
      : {};
  const values = valuesSource && typeof valuesSource === "object" ? valuesSource : {};
  const directEntries = Object.entries(values);
  if (directEntries.length > 0) return directEntries;

  const channels = Array.isArray(observation?.channels) ? observation.channels : [];
  const row = Array.isArray(observation?.row) ? observation.row : [];
  if (!channels.length || !row.length) return [];

  const out = [];
  channels.forEach((channel, index) => {
    const key = readObservationChannelKey(channel);
    if (!key) return;
    out.push([key, row[index]]);
  });
  return out;
}

function isTimeLikeObservationKey(rawKey) {
  const key = String(rawKey ?? "").trim().toLowerCase();
  if (!key) return false;
  return key === "t" || key === "time" || key === "tick" || key === "frame" || key === "프레임수" || key === "시간";
}

function readFallbackAngleFromObservation(observation) {
  const entries = readObservationValueEntries(observation);
  if (!entries.length) return null;

  const aliases = ["theta", "각도", "angle", "rad"];
  for (const [key, raw] of entries) {
    const normalized = String(key ?? "").trim().toLowerCase();
    if (!normalized) continue;
    if (!aliases.some((alias) => normalized.includes(alias))) continue;
    const num = parseFiniteNumericValue(raw);
    if (num !== null) return num;
  }
  return null;
}

function readNumericObservationValue(observation, keys = []) {
  const entries = readObservationValueEntries(observation);
  for (const key of keys) {
    const target = String(key ?? "").trim();
    if (!target) continue;
    const direct = entries.find(([entryKey]) => String(entryKey ?? "").trim() === target);
    const lower = direct
      ? null
      : entries.find(([entryKey]) => String(entryKey ?? "").trim().toLowerCase() === target.toLowerCase());
    const hit = direct ?? lower ?? null;
    if (!hit) continue;
    const num = parseFiniteNumericValue(hit[1]);
    if (num !== null) return num;
  }
  return null;
}

function finiteNumber(raw) {
  const n = Number(raw);
  return Number.isFinite(n) ? n : null;
}

function formatStatusNumber(raw, digits = 3) {
  const n = Number(raw);
  if (!Number.isFinite(n)) return "";
  return Number(n.toFixed(digits)).toString();
}

function normalizeParseWarnings(rawWarnings) {
  if (!Array.isArray(rawWarnings)) return [];
  return rawWarnings
    .filter((warning) => warning && typeof warning === "object")
    .map((warning) => ({
      code: String(warning.code ?? "").trim(),
      message: String(warning.message ?? "").trim(),
      span: warning.span ?? null,
    }));
}

function formatParseWarningSummary(warnings) {
  const normalized = normalizeParseWarnings(warnings);
  if (!normalized.length) return "";
  const codes = [];
  normalized.forEach((warning) => {
    const code = String(warning.code ?? "").trim();
    if (!code) return;
    if (codes.includes(code)) return;
    codes.push(code);
  });
  if (!codes.length) {
    return `문법경고: ${normalized.length}건`;
  }
  const headCodes = codes.slice(0, 2).join(", ");
  const moreCount = normalized.length - Math.min(normalized.length, 2);
  return moreCount > 0 ? `문법경고: ${headCodes} +${moreCount}` : `문법경고: ${headCodes}`;
}

function readGraphSeriesId(series) {
  if (!series || typeof series !== "object") return "";
  return String(series.id ?? series.name ?? series.label ?? "").trim();
}

function readLatestPointFromSeries(series) {
  if (!series || typeof series !== "object") return null;
  const points = Array.isArray(series.points) ? series.points : [];
  for (let i = points.length - 1; i >= 0; i -= 1) {
    const row = points[i];
    const x = finiteNumber(row?.x);
    const y = finiteNumber(row?.y);
    if (x === null || y === null) continue;
    return { x, y };
  }
  return null;
}

function isPendulumSeriesId(seriesId) {
  const normalized = String(seriesId ?? "").trim().toLowerCase();
  if (!normalized) return false;
  return ["theta", "각도", "angle", "rad"].some((token) => normalized.includes(token));
}

function readPreferredGraphSeries(graph) {
  const seriesList = Array.isArray(graph?.series) ? graph.series : [];
  if (!seriesList.length) return null;

  const pendulumHit = seriesList.find((series) => isPendulumSeriesId(readGraphSeriesId(series)));
  if (pendulumHit) return pendulumHit;

  return seriesList.find((series) => readLatestPointFromSeries(series)) ?? null;
}

export function synthesizePendulumSpace2dFromObservation(observation) {
  const thetaRaw = readNumericObservationValue(observation, ["theta", "각도", "theta_rad"]);
  const theta = thetaRaw === null ? readFallbackAngleFromObservation(observation) : thetaRaw;
  if (theta === null) return null;

  const lengthRaw = readNumericObservationValue(observation, ["L", "length", "len", "길이"]);
  const length = Math.max(0.2, Math.min(5, Number.isFinite(lengthRaw) ? lengthRaw : 1));
  const bx = length * Math.sin(theta);
  const by = -length * Math.cos(theta);
  const span = length + 0.35;

  return {
    schema: "seamgrim.space2d.v0",
    meta: {
      title: "pendulum-observation-fallback",
      source: "observation",
    },
    camera: {
      x_min: -span,
      x_max: span,
      y_min: -(length + 0.6),
      y_max: 0.6,
    },
    points: [{ x: bx, y: by }],
    shapes: [
      { kind: "line", x1: 0, y1: 0, x2: bx, y2: by, stroke: "#9ca3af", width: 0.02, group_id: "pendulum.rod" },
      { kind: "circle", x: bx, y: by, r: 0.08, fill: "#38bdf8", stroke: "#0ea5e9", width: 0.02, group_id: "pendulum.bob" },
      { kind: "point", x: 0, y: 0, size: 0.045, color: "#f59e0b", group_id: "pendulum.pivot" },
    ],
  };
}

export function synthesizePointSpace2dFromObservation(observation) {
  const x = readNumericObservationValue(observation, ["x", "x_pos", "pos_x", "px", "위치x"]);
  const y = readNumericObservationValue(observation, ["y", "y_pos", "pos_y", "py", "위치y"]);
  if (x === null && y === null) return null;

  const px = x === null ? 0 : x;
  const py = y === null ? 0 : y;
  const xMin = Math.min(0, px) - 1;
  const xMax = Math.max(0, px) + 1;
  const yMin = Math.min(0, py) - 1;
  const yMax = Math.max(0, py) + 1;

  return {
    schema: "seamgrim.space2d.v0",
    meta: {
      title: y === null ? "x-observation-fallback" : "xy-observation-fallback",
      source: "observation",
    },
    camera: {
      x_min: xMin,
      x_max: xMax,
      y_min: yMin,
      y_max: yMax,
    },
    points: [{ x: px, y: py }],
    shapes: [
      { kind: "line", x1: xMin, y1: 0, x2: xMax, y2: 0, stroke: "#4b5563", width: 0.01, group_id: "graph.axis.x" },
      { kind: "line", x1: 0, y1: yMin, x2: 0, y2: yMax, stroke: "#374151", width: 0.01, group_id: "graph.axis.y" },
      { kind: "circle", x: px, y: py, r: 0.07, fill: "#22c55e", stroke: "#16a34a", width: 0.02, group_id: "graph.point.focus" },
    ],
  };
}

export function synthesizeSpace2dFromGraph(graph, observation = null) {
  const hitSeries = readPreferredGraphSeries(graph);
  if (!hitSeries) return null;
  const point = readLatestPointFromSeries(hitSeries);
  if (!point) return null;

  const seriesId = readGraphSeriesId(hitSeries);
  if (isPendulumSeriesId(seriesId)) {
    const theta = point.y;
    const lengthRaw = readNumericObservationValue(observation, ["L", "length", "len", "길이"]);
    const length = Math.max(0.2, Math.min(5, Number.isFinite(lengthRaw) ? lengthRaw : 1));
    const bx = length * Math.sin(theta);
    const by = -length * Math.cos(theta);
    const span = length + 0.35;
    return {
      schema: "seamgrim.space2d.v0",
      meta: {
        title: "pendulum-graph-fallback",
        source: "graph",
      },
      camera: {
        x_min: -span,
        x_max: span,
        y_min: -(length + 0.6),
        y_max: 0.6,
      },
      points: [{ x: bx, y: by }],
      shapes: [
        { kind: "line", x1: 0, y1: 0, x2: bx, y2: by, stroke: "#9ca3af", width: 0.02, group_id: "pendulum.rod" },
        { kind: "circle", x: bx, y: by, r: 0.08, fill: "#38bdf8", stroke: "#0ea5e9", width: 0.02, group_id: "pendulum.bob" },
        { kind: "point", x: 0, y: 0, size: 0.045, color: "#f59e0b", group_id: "pendulum.pivot" },
      ],
    };
  }

  const axis = graph && typeof graph === "object" ? graph.axis : null;
  const xMin = finiteNumber(axis?.x_min) ?? Math.min(0, point.x) - 1;
  const xMax = finiteNumber(axis?.x_max) ?? Math.max(0, point.x) + 1;
  const yMin = finiteNumber(axis?.y_min) ?? Math.min(0, point.y) - 1;
  const yMax = finiteNumber(axis?.y_max) ?? Math.max(0, point.y) + 1;
  return {
    schema: "seamgrim.space2d.v0",
    meta: {
      title: "graph-point-fallback",
      source: "graph",
    },
    camera: {
      x_min: xMin,
      x_max: xMax,
      y_min: yMin,
      y_max: yMax,
    },
    points: [{ x: point.x, y: point.y }],
    shapes: [
      { kind: "line", x1: xMin, y1: 0, x2: xMax, y2: 0, stroke: "#4b5563", width: 0.01, group_id: "graph.axis.x" },
      { kind: "line", x1: 0, y1: yMin, x2: 0, y2: yMax, stroke: "#374151", width: 0.01, group_id: "graph.axis.y" },
      { kind: "circle", x: point.x, y: point.y, r: 0.07, fill: "#22c55e", stroke: "#16a34a", width: 0.02, group_id: "graph.point.focus" },
    ],
  };
}

function synthesizePendulumSpace2dFromGraph(graph, observation = null) {
  const candidate = synthesizeSpace2dFromGraph(graph, observation);
  const title = String(candidate?.meta?.title ?? "").trim();
  if (!candidate || !title.startsWith("pendulum-")) return null;
  return candidate;
}

export function synthesizeSpace2dFromObservation(observation) {
  return (
    synthesizePendulumSpace2dFromObservation(observation) ??
    synthesizePointSpace2dFromObservation(observation)
  );
}

function formatRecentTimeLabel(isoText) {
  const ms = Date.parse(String(isoText ?? ""));
  if (!Number.isFinite(ms)) return "";
  const d = new Date(ms);
  const month = String(d.getMonth() + 1);
  const day = String(d.getDate());
  const hour = String(d.getHours()).padStart(2, "0");
  const minute = String(d.getMinutes()).padStart(2, "0");
  return `${month}/${day} ${hour}:${minute}`;
}

function normalizeRunLaunchKind(raw) {
  const kind = String(raw ?? "").trim().toLowerCase();
  if (!kind) return "manual";
  if (kind === "manual") return "manual";
  if (kind === "browse_select") return "browse_select";
  if (kind === "editor_run") return "editor_run";
  if (kind === "featured_seed_quick") return "featured_seed_quick";
  return "manual";
}

function buildRunSummaryText(pref) {
  if (!pref || typeof pref !== "object") return "최근 실행: 기록 없음";
  const kind = String(pref.lastRunKind ?? "").trim();
  const channels = Math.max(0, Number.isFinite(Number(pref.lastRunChannels)) ? Math.trunc(Number(pref.lastRunChannels)) : 0);
  const timeLabel = formatRecentTimeLabel(pref.lastRunAt);
  const hash = String(pref.lastRunHash ?? "").trim();
  const shortHash = hash && hash !== "-" ? hash.slice(0, 12) : "";
  let label = "";
  if (kind === "space2d") {
    label = `최근 실행: 보개 출력 · 채널=${channels}`;
  } else if (kind === "obs_only") {
    label = `최근 실행: 보개 없음 · 채널=${channels}`;
  } else if (kind === "empty") {
    label = "최근 실행: 출력 없음";
  } else if (kind === "error") {
    label = "최근 실행: 실패";
  } else {
    label = "최근 실행: 기록 없음";
  }
  if (shortHash) {
    label = `${label} · hash:${shortHash}`;
  }
  return timeLabel ? `${label} · ${timeLabel}` : label;
}

function extractRuntimeDerived(stateJson) {
  if (!stateJson) return null;
  return {
    observation: extractObservationChannelsFromState(stateJson),
    views: extractStructuredViewsFromState(stateJson, { preferPatch: false }),
  };
}

function buildObservationFromGraph(graph) {
  const seriesList = Array.isArray(graph?.series) ? graph.series : [];
  if (!seriesList.length) return null;
  const first = seriesList.find((series) => Array.isArray(series?.points) && series.points.length > 0) ?? null;
  if (!first) return null;
  const points = Array.isArray(first.points) ? first.points : [];
  const last = points[points.length - 1];
  const x = finiteNumber(last?.x);
  const y = finiteNumber(last?.y);
  if (x === null || y === null) return null;
  const yKey = String(first?.id ?? first?.label ?? "y").trim() || "y";
  return {
    channels: [
      { key: "x", dtype: "number", role: "state" },
      { key: yKey, dtype: "number", role: "state" },
    ],
    row: [x, y],
    values: {
      x,
      [yKey]: y,
    },
    all_values: {
      x,
      [yKey]: y,
    },
  };
}

function clampRuntimeTableCellChars(value) {
  const numeric = Math.max(1, Math.trunc(Number(value) || 0));
  return Math.max(
    RUNTIME_TABLE_CELL_MIN_CHARS,
    Math.min(RUNTIME_TABLE_CELL_MAX_CHARS_LIMIT, numeric),
  );
}

function escapeRuntimeTableHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function truncateRuntimeTableCellText(value, { maxChars = 24 } = {}) {
  const text = String(value ?? "");
  const limit = clampRuntimeTableCellChars(Number(maxChars) || RUNTIME_TABLE_CELL_MAX_CHARS);
  if (text.length <= limit) {
    return { text, truncated: false };
  }
  return {
    text: `${text.slice(0, Math.max(1, limit - 1))}…`,
    truncated: true,
  };
}

export function resolveRuntimeTableCellMaxChars(container, { maxChars = null } = {}) {
  const direct = Number(maxChars);
  if (Number.isFinite(direct) && direct > 0) {
    return clampRuntimeTableCellChars(direct);
  }

  const datasetValue = Number(container?.dataset?.cellMaxChars ?? "");
  if (Number.isFinite(datasetValue) && datasetValue > 0) {
    return clampRuntimeTableCellChars(datasetValue);
  }

  const width = Number(container?.clientWidth ?? container?.offsetWidth ?? 0);
  if (Number.isFinite(width) && width > 0) {
    const estimated = Math.floor(
      Math.max(0, width - RUNTIME_TABLE_CELL_WIDTH_PADDING_PX) / RUNTIME_TABLE_CELL_ESTIMATED_PX_PER_CHAR,
    );
    if (estimated > 0) {
      return clampRuntimeTableCellChars(estimated);
    }
  }

  return RUNTIME_TABLE_CELL_MAX_CHARS;
}

function buildRuntimeTableCellHtml(value, options = {}) {
  const fullText = String(value ?? "");
  const truncated = truncateRuntimeTableCellText(fullText, {
    maxChars: options?.maxChars ?? RUNTIME_TABLE_CELL_MAX_CHARS,
  });
  const escapedText = escapeRuntimeTableHtml(truncated.text);
  const titleAttr = truncated.truncated ? ` title="${escapeRuntimeTableHtml(fullText)}"` : "";
  return `<span class="runtime-table-celltext"${titleAttr}>${escapedText}</span>`;
}

function readRuntimeTableColumnKey(column, index) {
  const direct = String(column?.key ?? "").trim();
  if (direct) return direct;
  return `col_${index}`;
}

export function normalizeRuntimeTableView(table, { maxRows = 24 } = {}) {
  if (!table || typeof table !== "object") return null;
  const columnsRaw = Array.isArray(table.columns) ? table.columns : [];
  const rowsRaw = Array.isArray(table.rows) ? table.rows : [];
  if (!columnsRaw.length) return null;
  const columns = columnsRaw
    .map((column, index) => {
      const key = readRuntimeTableColumnKey(column, index);
      return {
        key,
        label: String(column?.label ?? key).trim() || key,
        type: String(column?.type ?? "").trim(),
      };
    })
    .filter((column) => Boolean(column.key));
  if (!columns.length) return null;

  const limitedRows = rowsRaw.slice(0, Math.max(0, Number(maxRows) || 0));
  const rows = limitedRows.map((row, rowIndex) => {
    const rowObj = row && typeof row === "object" ? row : {};
    const cells = columns.map((column, colIndex) => {
      const direct = Object.prototype.hasOwnProperty.call(rowObj, column.key)
        ? rowObj[column.key]
        : Array.isArray(rowObj)
          ? rowObj[colIndex]
          : undefined;
      return formatObservationCellValue(direct, { numericMode: "compact" });
    });
    return {
      key: `row_${rowIndex}`,
      cells,
    };
  });

  return {
    columns,
    rows,
    rowCount: rowsRaw.length,
    shownRowCount: rows.length,
    truncated: rowsRaw.length > rows.length,
    schema: String(table.schema ?? "").trim(),
    source: String(table?.meta?.source ?? "").trim(),
  };
}

function formatRuntimeTableSchemaBadge(schema) {
  const text = String(schema ?? "").trim();
  if (!text) return "";
  if (text.startsWith("seamgrim.")) {
    return text.slice("seamgrim.".length);
  }
  return text;
}

export function summarizeRuntimeTableView(normalized, { emptyText = "" } = {}) {
  if (!normalized || typeof normalized !== "object" || !Array.isArray(normalized.columns) || !normalized.columns.length) {
    return String(emptyText ?? "");
  }
  const columnCount = Math.max(0, normalized.columns.length);
  const rowCount = Math.max(0, Number(normalized.rowCount) || 0);
  const shownRowCount = Math.max(0, Number(normalized.shownRowCount) || 0);
  const schemaBadge = formatRuntimeTableSchemaBadge(normalized.schema);
  const sourceBadge = String(normalized.source ?? "").trim();
  const suffix = [schemaBadge, sourceBadge].filter(Boolean).join(" · ");
  if (normalized.truncated && rowCount > shownRowCount) {
    return suffix
      ? `${columnCount}열 · ${rowCount}행 중 ${shownRowCount}행 표시 · ${suffix}`
      : `${columnCount}열 · ${rowCount}행 중 ${shownRowCount}행 표시`;
  }
  return suffix ? `${columnCount}열 · ${rowCount}행 · ${suffix}` : `${columnCount}열 · ${rowCount}행`;
}

export function renderRuntimeTable(container, table, options = {}) {
  const {
    maxRows = 24,
    emptyText = "표 출력 없음",
    maxChars = null,
  } = options && typeof options === "object" ? options : {};
  if (!container || typeof container !== "object") return false;
  const normalized = normalizeRuntimeTableView(table, { maxRows });
  const cellMaxChars = resolveRuntimeTableCellMaxChars(container, { maxChars });
  if (!normalized || !normalized.columns.length) {
    if ("innerHTML" in container) {
      container.innerHTML = `<div class="runtime-table-empty">${escapeRuntimeTableHtml(emptyText)}</div>`;
    } else if ("textContent" in container) {
      container.textContent = emptyText;
    }
    return false;
  }

  const head = normalized.columns
    .map((column) => `<th data-col-key="${escapeRuntimeTableHtml(column.key)}">${escapeRuntimeTableHtml(column.label)}</th>`)
    .join("");
  const body = normalized.rows
    .map(
      (row) =>
        `<tr>${row.cells
          .map((cell, index) => `<td data-col-key="${escapeRuntimeTableHtml(normalized.columns[index]?.key ?? "")}">${buildRuntimeTableCellHtml(cell, { maxChars: cellMaxChars })}</td>`)
          .join("")}</tr>`,
    )
    .join("");
  const moreText = normalized.truncated
    ? `<div class="runtime-table-empty">+${escapeRuntimeTableHtml(normalized.rowCount - normalized.rows.length)}행 더 있음</div>`
    : "";
  const html = `<table class="runtime-table"><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table>${moreText}`;
  if ("innerHTML" in container) {
    container.innerHTML = html;
  } else if ("textContent" in container) {
    container.textContent = normalized.rows
      .map((row) => row.cells.join(" | "))
      .join("\n");
  }
  return true;
}

function buildServerPlaybackPlan(graph) {
  const series = Array.isArray(graph?.series) ? graph.series : [];
  const first = series.find((row) => Array.isArray(row?.points) && row.points.length > 1) ?? null;
  if (!first) return null;
  const seriesId = String(first?.id ?? first?.label ?? "y").trim() || "y";
  const frames = [];
  const points = Array.isArray(first.points) ? first.points : [];
  points.forEach((row) => {
    const x = finiteNumber(row?.x);
    const y = finiteNumber(row?.y);
    if (x === null || y === null) return;
    frames.push({ x, y });
  });
  if (frames.length < 2) return null;
  return {
    seriesId,
    frames,
    axis: graph?.axis ?? null,
  };
}

function readOverlayMarkdownFromViewText(textView) {
  if (!textView || typeof textView !== "object") return "";
  const markdown = String(textView.markdown ?? textView.text ?? "").trim();
  return markdown;
}

export function summarizeRuntimeStructureMarkdown(structureView, { sampleLimit = 3 } = {}) {
  return buildStructureSummaryMarkdown(structureView, { sampleLimit });
}

export function summarizeRuntimeGraphMarkdown(graphView, { seriesLimit = 3 } = {}) {
  return buildGraphSummaryMarkdown(graphView, { seriesLimit });
}

function readRuntimeTextMarkdownFromViews(views) {
  const directText = readOverlayMarkdownFromViewText(views?.text);
  if (directText) return directText;
  return summarizeRuntimeStructureMarkdown(views?.structure);
}

export function buildRuntimeStructurePreviewHtml(structureView, { width = 280, height = 164, maxNodes = 8 } = {}) {
  return buildStructurePreviewHtml(structureView, { width, height, maxNodes });
}

export function buildRuntimeGraphPreviewHtml(graphView, { width = 280, height = 164, maxSeries = 3 } = {}) {
  return buildGraphPreviewHtml(graphView, { width, height, maxSeries });
}

export class RunScreen {
  constructor({
    root,
    wasmState,
    onBack,
    onEditDdn,
    onOpenAdvanced,
    onSelectLesson,
    onGetInspectorContext,
    getOverlaySession,
    getRuntimeSessionV0,
    onOverlaySessionChange,
    onSaveSnapshot,
    onSaveSession,
    onFormulaApplied,
    allowShapeFallback = false,
    allowServerFallback = false,
  } = {}) {
    this.root = root;
    this.wasmState = wasmState;
    this.onBack = typeof onBack === "function" ? onBack : () => {};
    this.onEditDdn = typeof onEditDdn === "function" ? onEditDdn : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};
    this.onSelectLesson = typeof onSelectLesson === "function" ? onSelectLesson : null;
    this.onGetInspectorContext = typeof onGetInspectorContext === "function" ? onGetInspectorContext : null;
    this.getOverlaySession = typeof getOverlaySession === "function" ? getOverlaySession : null;
    this.getRuntimeSessionV0 = typeof getRuntimeSessionV0 === "function" ? getRuntimeSessionV0 : null;
    this.onOverlaySessionChange = typeof onOverlaySessionChange === "function" ? onOverlaySessionChange : null;
    this.onSaveSnapshot = typeof onSaveSnapshot === "function" ? onSaveSnapshot : null;
    this.onSaveSession = typeof onSaveSession === "function" ? onSaveSession : null;
    this.onFormulaApplied = typeof onFormulaApplied === "function" ? onFormulaApplied : null;
    this.allowShapeFallback = Boolean(allowShapeFallback);
    this.allowServerFallback = Boolean(allowServerFallback);

    this.lesson = null;
    this.baseDdn = "";
    this.currentDdn = "";
    this.lastState = null;
    this.lastRuntimeDerived = null;
    this.lessonLayoutProfile = resolveRunLayoutProfile([]);

    this.loopActive = false;
    this.screenVisible = false;
    this.loop = null;
    this.viewPanStep = 0.08;
    this.lastRuntimeHash = "-";
    this.lastRuntimeSnapshotKey = "";
    this.lastExecPathHint = "";
    this.lastSpace2dMode = "none";
    this.lastRuntimeHintText = "";
    this.lastPreviewViewModel = null;
    this.lastOverlayMarkdown = "";
    this.lastRuntimeTextMarkdown = "";
    this.lastParseWarnings = [];
    this.runtimeTickCounter = 0;
    this.runtimeTimeValue = null;
    this.serverPlayback = null;
    this.activeOverlayRunId = "";
    this.hoverOverlayRunId = "";
    this.soloOverlayRunId = "";
    this.lastRuntimeTableCellMaxChars = 0;
    this.runtimeTableResizeObserver = null;
    this.runtimeTableResizeFallbackInstalled = false;
    this.activeRunTab = "lesson";
    this.dockTarget = "space2d";
    this.playbackPaused = false;
    this.playbackSpeed = 1;
    this.playbackLoop = true;
    this.dockCursorTick = 0;
    this.dockCursorFollowLive = true;
    this.viewPlaybackTimer = null;
    this.lessonOptions = [];
    this.lastInspectorReport = null;
    this.lastInspectorStatusText = "";
    this.lastFormulaDerivedDdn = "";
    this.heldInputMask = 0;
    this.pulsePressedMask = 0;
    this.lastInputToken = "";
    this.lastLaunchKind = "manual";
    this.overlayRuns = [];
    this.activeOverlayRunId = "";
    this.hoverOverlayRunId = "";
    this.soloOverlayRunId = "";
    this.runManagerSequence = 0;
    this.boundKeyDownHandler = (event) => {
      this.handleViewHotkeys(event);
      this.handleRuntimeInputKeyDown(event);
    };
    this.boundKeyUpHandler = (event) => {
      this.handleRuntimeInputKeyUp(event);
    };
    this.boundWindowBlurHandler = () => {
      this.clearRuntimeInputState();
    };
    this.boundRuntimeTableResizeHandler = () => {
      this.refreshRuntimeTableForCurrentWidth();
    };
    this.uiPrefs = {
      lessons: {},
    };
  }

  isSimCorePolicyEnabled() {
    try {
      return Boolean(document?.body?.classList?.contains("policy-sim-core"));
    } catch (_) {
      return false;
    }
  }

  init() {
    this.titleEl = this.root.querySelector("#run-lesson-title");
    this.lastSummaryEl = this.root.querySelector("#run-last-summary");
    this.layoutEl = this.root.querySelector(".run-layout");
    this.runLessonSummaryEl = this.root.querySelector("#run-lesson-summary");
    this.runLessonSelectEl = this.root.querySelector("#run-lesson-select");
    this.runLessonLoadBtn = this.root.querySelector("#btn-run-lesson-load");
    this.runDdnPreviewEl = this.root.querySelector("#run-ddn-preview");
    this.formulaTextEl = this.root.querySelector("#run-formula-text");
    this.formulaAxisEl = this.root.querySelector("#run-formula-axis");
    this.formulaXMinEl = this.root.querySelector("#run-formula-x-min");
    this.formulaXMaxEl = this.root.querySelector("#run-formula-x-max");
    this.formulaStepEl = this.root.querySelector("#run-formula-step");
    this.formulaDdnPreviewEl = this.root.querySelector("#run-formula-ddn-preview");
    this.formulaStatusEl = this.root.querySelector("#run-formula-status");
    this.formulaApplyReplaceBtn = this.root.querySelector("#btn-formula-apply-replace");
    this.formulaApplyInsertBtn = this.root.querySelector("#btn-formula-apply-insert");
    this.runResetToLessonBtn = this.root.querySelector("#btn-reset-to-lesson");
    this.runInspectorMetaEl = this.root.querySelector("#run-inspector-meta");
    this.runInspectorStatusEl = this.root.querySelector("#run-inspector-status");
    this.saveSnapshotBtn = this.root.querySelector("#btn-save-snapshot-v0");
    this.saveSessionBtn = this.root.querySelector("#btn-save-session-v0");
    this.runTabButtons = RUN_TAB_IDS.map((tab) => this.root.querySelector(`#run-tab-btn-${tab}`)).filter(Boolean);
    this.runTabPanels = new Map(
      RUN_TAB_IDS.map((tab) => [tab, this.root.querySelector(`#run-tab-panel-${tab}`)]),
    );
    this.bogaeAreaEl = this.root.querySelector(".bogae-area");
    this.dotbogiPanelEl = this.root.querySelector(".dotbogi-panel");
    this.graphPanelEl = this.root.querySelector("#dotbogi-graph");
    this.ensureRunManagerUi();
    this.runManagerPanelEl = this.root.querySelector("#run-manager-panel");
    this.runManagerListEl = this.root.querySelector("#run-manager-list");
    this.runManagerClearBtn = this.root.querySelector("#btn-run-manager-clear");
    this.runManagerPruneBtn = this.root.querySelector("#btn-run-manager-prune");
    this.runtimeStatusEl = this.root.querySelector("#slider-status");
    this.runtimeTablePanelEl = this.root.querySelector("#runtime-table-panel");
    this.runtimeTableMetaEl = this.root.querySelector("#runtime-table-meta");
    this.runtimeTableEl = this.root.querySelector("#runtime-table");
    this.runtimeTextPanelEl = this.root.querySelector("#runtime-text-panel");
    this.runtimeTextBodyEl = this.root.querySelector("#runtime-text-body");
    this.overlayToggleBtn = this.root.querySelector("#btn-overlay-toggle");
    this.dockSpaceRangeEl = this.root.querySelector("#dock-space-range");
    this.dockGraphRangeEl = this.root.querySelector("#dock-graph-range");
    this.dockTargetSelectEl = this.root.querySelector("#select-dock-target");
    this.dockGridCheckEl = this.root.querySelector("#chk-dock-grid");
    this.dockAxisCheckEl = this.root.querySelector("#chk-dock-axis");
    this.dockOverlayCheckEl = this.root.querySelector("#chk-dock-overlay");
    this.dockHighlightCheckEl = this.root.querySelector("#chk-dock-highlight");
    this.dockLoopCheckEl = this.root.querySelector("#chk-dock-loop");
    this.dockSpeedSelectEl = this.root.querySelector("#select-dock-speed");
    this.dockTimeCursorEl = this.root.querySelector("#range-dock-time-cursor");
    this.dockTimeTextEl = this.root.querySelector("#text-dock-time");
    this.installRuntimeTableResizeObserver();

    this.uiPrefs = {
      lessons: {},
      ...readStorageJson(RUN_UI_PREFS_STORAGE_KEY, {}),
    };
    if (!this.uiPrefs.lessons || typeof this.uiPrefs.lessons !== "object") {
      this.uiPrefs.lessons = {};
    }

    this.bogae = new Bogae({
      canvas: this.root.querySelector("#canvas-bogae"),
      onRangeChange: (range) => {
        this.syncDockRangeLabels({ spaceRange: range });
      },
    });
    this.dotbogi = new DotbogiPanel({
      graphCanvas: this.root.querySelector("#canvas-graph"),
      xAxisSelect: this.root.querySelector("#select-x-axis"),
      yAxisSelect: this.root.querySelector("#select-y-axis"),
      onAxisChange: (axis) => {
        this.syncDockRangeLabels({ graphAxis: axis });
      },
    });
    this.overlay = new OverlayDescription(this.root.querySelector("#overlay-description"));

    this.sliderPanel = new SliderPanel({
      container: this.root.querySelector("#slider-list"),
      statusEl: this.root.querySelector("#slider-status"),
      onCommit: () => {
        void this.restart();
      },
    });

    this.root.querySelector("#btn-back-run")?.addEventListener("click", () => {
      this.onBack();
    });

    this.root.querySelector("#btn-edit-ddn")?.addEventListener("click", () => {
      this.onEditDdn({
        ddnText: this.baseDdn,
        title: this.lesson?.title || this.lesson?.id || "DDN 보기",
      });
    });

    this.root.querySelector("#btn-overlay-toggle")?.addEventListener("click", () => {
      this.overlay.toggle();
      this.syncDockGuideToggles();
    });

    this.root.querySelector("#btn-advanced-run")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });

    this.root.querySelector("#btn-restart")?.addEventListener("click", () => {
      void this.restart();
    });
    this.saveSnapshotBtn?.addEventListener("click", () => {
      this.handleSaveSnapshot();
    });
    this.saveSessionBtn?.addEventListener("click", () => {
      this.handleSaveSession();
    });
    this.runManagerClearBtn?.addEventListener("click", () => {
      this.clearRunManagerRuns();
    });
    this.runManagerPruneBtn?.addEventListener("click", () => {
      this.pruneHiddenRunManagerRuns();
    });
    this.root.querySelector("#select-x-axis")?.addEventListener("change", () => {
      this.saveCurrentLessonUiPrefs();
    });
    this.root.querySelector("#select-y-axis")?.addEventListener("change", () => {
      this.saveCurrentLessonUiPrefs();
    });
    this.runLessonLoadBtn?.addEventListener("click", () => {
      void this.loadSelectedLessonFromTab();
    });
    this.runLessonSelectEl?.addEventListener("change", () => {
      this.syncRunLessonLoadButtonState();
    });
    this.runResetToLessonBtn?.addEventListener("click", () => {
      void this.resetToLessonSource();
    });
    this.bindFormulaSugarUi();
    this.bindRunTabUi();
    this.bindViewDockUi();
    this.switchRunTab("lesson");
    this.syncDockGuideToggles();
    this.applyDockGuideToggles();
    this.syncDockRangeLabels();
    this.syncDockTimeUi();
    this.syncRunLessonLoadButtonState();
    this.hydrateRunManagerFromSession({ publish: false });
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.setInspectorStatus(this.lastInspectorStatusText || "저장 작업 대기");

    this.loop = createManagedRafStepLoop({
      getFps: () => {
        const base = Math.max(1, Number(this.wasmState?.fpsLimit ?? 30) || 30);
        return Math.max(1, Math.round(base * this.playbackSpeed));
      },
      isActive: () => this.loopActive,
      setActive: (active) => {
        this.loopActive = Boolean(active);
      },
      onStep: () => {
        this.stepFrame();
      },
      onError: (err) => {
        // runtime loop errors are surfaced through diagnostics/gates.
      },
    });
    if (typeof window !== "undefined" && window?.addEventListener) {
      window.addEventListener("keydown", this.boundKeyDownHandler);
      window.addEventListener("keyup", this.boundKeyUpHandler);
      window.addEventListener("blur", this.boundWindowBlurHandler);
    }
  }

  ensureRunManagerUi() {
    if (!this.graphPanelEl || typeof document === "undefined") return;
    const existing = this.root.querySelector("#run-manager-panel");
    if (existing) return;
    const panel = document.createElement("div");
    panel.id = "run-manager-panel";
    panel.className = "run-manager-panel";
    panel.innerHTML = `
      <div class="run-manager-head">
        <span class="run-manager-title">오버레이 실행</span>
        <button id="btn-run-manager-prune" type="button">숨김 정리</button>
        <button id="btn-run-manager-clear" type="button">전체 삭제</button>
      </div>
      <div id="run-manager-list" class="run-manager-list"></div>
    `;
    const toolbar = this.graphPanelEl.querySelector(".graph-toolbar");
    if (toolbar?.insertAdjacentElement) {
      toolbar.insertAdjacentElement("afterend", panel);
      return;
    }
    this.graphPanelEl.prepend(panel);
  }

  buildRunManagerBasePayload() {
    const base = this.getOverlaySession && typeof this.getOverlaySession === "function"
      ? this.getOverlaySession() ?? {}
      : {};
    return toPlainObject(base, {});
  }

  normalizeRunManagerRun(raw, index = 0) {
    const row = toPlainObject(raw, {});
    const id = String(row.id ?? `run-${index + 1}`).trim() || `run-${index + 1}`;
    const label = String(row.label ?? id).trim() || id;
    const source = toPlainObject(row.source, {});
    const inputs = toPlainObject(row.inputs, {});
    const hashRaw = toPlainObject(row.hash, {});
    const graph = cloneGraphForRunManager(row.graph ?? row.result ?? null);
    const hashInput = String(hashRaw.input ?? graph?.meta?.source_input_hash ?? "").trim();
    const hashResult = String(hashRaw.result ?? graph?.meta?.result_hash ?? "").trim();
    const hue = stableColorHue(label, hashResult || hashInput || id);
    return {
      id,
      label,
      visible: row.visible !== false,
      layerIndex: normalizeRunManagerLayer(row.layerIndex ?? row.layer_index ?? row.order, index),
      source,
      inputs,
      graph,
      result: graph,
      hash: {
        input: hashInput,
        result: hashResult,
      },
      hue,
    };
  }

  serializeRunManagerRun(run) {
    const row = toPlainObject(run, {});
    return {
      id: String(row.id ?? "").trim(),
      label: String(row.label ?? "").trim(),
      visible: row.visible !== false,
      layer_index: normalizeRunManagerLayer(row.layerIndex ?? row.layer_index, 0),
      source: toPlainObject(row.source, {}),
      inputs: toPlainObject(row.inputs, {}),
      graph: cloneGraphForRunManager(row.graph ?? null),
      result: cloneGraphForRunManager(row.graph ?? null),
      hash: {
        input: String(row?.hash?.input ?? "").trim(),
        result: String(row?.hash?.result ?? "").trim(),
      },
    };
  }

  getCurrentLessonRunLabel() {
    const lessonLabel = String(this.lesson?.title ?? this.lesson?.id ?? "run").trim() || "run";
    this.runManagerSequence += 1;
    return `${lessonLabel} #${this.runManagerSequence}`;
  }

  getNextRunLayerIndex() {
    if (!Array.isArray(this.overlayRuns) || !this.overlayRuns.length) return 0;
    return Math.max(...this.overlayRuns.map((row) => normalizeRunManagerLayer(row?.layerIndex, 0))) + 1;
  }

  buildCurrentGraphViewPayload() {
    const axis = this.dotbogi?.getCurrentAxis?.() ?? null;
    return normalizeGraphView({
      auto: false,
      x_min: axis?.x_min,
      x_max: axis?.x_max,
      y_min: axis?.y_min,
      y_max: axis?.y_max,
      pan_x: 0,
      pan_y: 0,
      zoom: 1,
    });
  }

  beginLiveRunCapture(ddnText = "") {
    const lessonId = String(this.lesson?.id ?? "").trim();
    const inputValues = this.sliderPanel?.getValues?.() ?? {};
    const inputHash = buildRunInputHash({
      ddnText,
      controls: inputValues,
      sample: null,
    });
    const id = `run:${lessonId || "custom"}:${Date.now().toString(36)}:${this.runManagerSequence + 1}`;
    const label = this.getCurrentLessonRunLabel();
    const run = this.normalizeRunManagerRun(
      {
        id,
        label,
        visible: true,
        layer_index: this.getNextRunLayerIndex(),
        source: {
          kind: "ddn",
          lessonId,
          launchKind: this.lastLaunchKind,
          text: String(ddnText ?? ""),
        },
        inputs: {
          controls: toPlainObject(inputValues, {}),
        },
        hash: {
          input: inputHash,
          result: "",
        },
      },
      this.overlayRuns.length,
    );
    this.overlayRuns.push(run);
    if (this.overlayRuns.length > RUN_MANAGER_MAX_RUNS) {
      this.overlayRuns = this.overlayRuns.slice(this.overlayRuns.length - RUN_MANAGER_MAX_RUNS);
    }
    this.activeOverlayRunId = run.id;
    this.soloOverlayRunId = "";
    this.hoverOverlayRunId = "";
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
    return run.id;
  }

  findRunManagerIndexById(runId) {
    const target = String(runId ?? "").trim();
    if (!target) return -1;
    return this.overlayRuns.findIndex((row) => String(row?.id ?? "").trim() === target);
  }

  updateLiveRunCaptureFromDerived(derived) {
    const activeId = String(this.activeOverlayRunId ?? "").trim();
    if (!activeId) return;
    const index = this.findRunManagerIndexById(activeId);
    if (index < 0) return;
    const current = this.overlayRuns[index];
    const graph = cloneGraphForRunManager(derived?.views?.graph ?? null);
    if (!graph) return;
    const currentGraphView = this.buildCurrentGraphViewPayload();
    if (currentGraphView) {
      graph.view = currentGraphView;
    }
    const sample = normalizeGraphSample(graph.sample ?? null);
    const controls = toPlainObject(this.sliderPanel?.getValues?.(), {});
    const hashInput = buildRunInputHash({
      ddnText: String(current?.source?.text ?? this.currentDdn ?? ""),
      controls,
      sample,
    });
    const hashResult = String(this.lastRuntimeHash ?? "").trim();
    const next = this.normalizeRunManagerRun(
      {
        ...current,
        graph,
        result: graph,
        inputs: {
          ...(toPlainObject(current?.inputs, {})),
          controls,
          ...(sample ? { sample } : {}),
        },
        hash: {
          ...(toPlainObject(current?.hash, {})),
          input: hashInput,
          result: hashResult,
        },
      },
      index,
    );
    this.overlayRuns[index] = next;
    this.syncRunManagerOverlaySeries();
  }

  discardActiveRunCaptureIfEmpty() {
    const activeId = String(this.activeOverlayRunId ?? "").trim();
    if (!activeId) return;
    const index = this.findRunManagerIndexById(activeId);
    if (index < 0) return;
    const run = this.overlayRuns[index];
    const hasGraph = Boolean(run?.graph && Array.isArray(run.graph.series) && run.graph.series.length > 0);
    if (hasGraph) return;
    this.overlayRuns.splice(index, 1);
    this.activeOverlayRunId = "";
    if (this.soloOverlayRunId === activeId) this.soloOverlayRunId = "";
    if (this.hoverOverlayRunId === activeId) this.hoverOverlayRunId = "";
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
  }

  clearRunManagerRuns() {
    this.overlayRuns = [];
    this.activeOverlayRunId = "";
    this.hoverOverlayRunId = "";
    this.soloOverlayRunId = "";
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
  }

  pruneHiddenRunManagerRuns() {
    const next = this.overlayRuns.filter((row) => row.visible !== false);
    if (next.length === this.overlayRuns.length) return;
    this.overlayRuns = next;
    if (this.activeOverlayRunId && this.findRunManagerIndexById(this.activeOverlayRunId) < 0) {
      this.activeOverlayRunId = "";
    }
    if (this.hoverOverlayRunId && this.findRunManagerIndexById(this.hoverOverlayRunId) < 0) {
      this.hoverOverlayRunId = "";
    }
    if (this.soloOverlayRunId && this.findRunManagerIndexById(this.soloOverlayRunId) < 0) {
      this.soloOverlayRunId = "";
    }
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
  }

  removeRunManagerRun(runId) {
    const target = String(runId ?? "").trim();
    if (!target) return;
    const index = this.findRunManagerIndexById(target);
    if (index < 0) return;
    this.overlayRuns.splice(index, 1);
    if (this.activeOverlayRunId === target) this.activeOverlayRunId = "";
    if (this.hoverOverlayRunId === target) this.hoverOverlayRunId = "";
    if (this.soloOverlayRunId === target) this.soloOverlayRunId = "";
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
  }

  setRunManagerHover(runId = "") {
    const nextId = String(runId ?? "").trim();
    if (this.hoverOverlayRunId === nextId) return;
    this.hoverOverlayRunId = nextId;
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
  }

  toggleRunManagerSolo(runId) {
    const target = String(runId ?? "").trim();
    if (!target) return;
    this.soloOverlayRunId = this.soloOverlayRunId === target ? "" : target;
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
  }

  renderRunManagerUi() {
    if (!this.runManagerListEl) return;
    const runs = [...(Array.isArray(this.overlayRuns) ? this.overlayRuns : [])]
      .sort((a, b) => normalizeRunManagerLayer(a?.layerIndex, 0) - normalizeRunManagerLayer(b?.layerIndex, 0));
    if (!runs.length) {
      this.runManagerListEl.innerHTML = '<div class="run-manager-empty">저장된 run 없음</div>';
      return;
    }
    const html = runs
      .map((row) => {
        const id = String(row.id ?? "").trim();
        const label = String(row.label ?? id).trim() || id;
        const active = id && id === this.activeOverlayRunId;
        const hovered = id && id === this.hoverOverlayRunId;
        const solo = id && id === this.soloOverlayRunId;
        const color = buildRunColor(row.hue, 1);
        const hashText = shortHash(row?.hash?.result || row?.hash?.input || "");
        const classes = [
          "run-manager-row",
          active ? "is-active" : "",
          hovered ? "is-hovered" : "",
          solo ? "is-solo" : "",
        ]
          .filter(Boolean)
          .join(" ");
        return `
          <div class="${classes}" data-run-row="${escapeHtml(id)}">
            <label class="run-manager-visible">
              <input type="checkbox" data-run-visible="${escapeHtml(id)}" ${row.visible !== false ? "checked" : ""} />
              <span class="run-manager-dot" style="--run-color:${escapeHtml(color)};"></span>
            </label>
            <button type="button" class="run-manager-label" data-run-solo="${escapeHtml(id)}" title="solo 토글">${escapeHtml(label)}</button>
            <span class="run-manager-hash" title="${escapeHtml(String(row?.hash?.result || row?.hash?.input || "-"))}">${escapeHtml(hashText)}</span>
            <button type="button" class="run-manager-remove" data-run-remove="${escapeHtml(id)}" title="삭제">×</button>
          </div>
        `;
      })
      .join("");
    this.runManagerListEl.innerHTML = html;

    this.runManagerListEl.querySelectorAll("[data-run-visible]").forEach((input) => {
      input.addEventListener("change", (event) => {
        const target = event.currentTarget;
        const runId = String(target?.getAttribute?.("data-run-visible") ?? "").trim();
        const index = this.findRunManagerIndexById(runId);
        if (index < 0) return;
        this.overlayRuns[index].visible = Boolean(target?.checked);
        this.syncRunManagerOverlaySeries();
        this.publishRunManagerSession();
      });
    });
    this.runManagerListEl.querySelectorAll("[data-run-solo]").forEach((button) => {
      button.addEventListener("click", (event) => {
        const target = event.currentTarget;
        const runId = String(target?.getAttribute?.("data-run-solo") ?? "").trim();
        this.toggleRunManagerSolo(runId);
      });
    });
    this.runManagerListEl.querySelectorAll("[data-run-remove]").forEach((button) => {
      button.addEventListener("click", (event) => {
        const target = event.currentTarget;
        const runId = String(target?.getAttribute?.("data-run-remove") ?? "").trim();
        this.removeRunManagerRun(runId);
      });
    });
    this.runManagerListEl.querySelectorAll("[data-run-row]").forEach((rowEl) => {
      rowEl.addEventListener("mouseenter", () => {
        const runId = String(rowEl?.getAttribute?.("data-run-row") ?? "").trim();
        this.setRunManagerHover(runId);
      });
      rowEl.addEventListener("mouseleave", () => {
        this.setRunManagerHover("");
      });
    });
  }

  syncRunManagerOverlaySeries() {
    const source = Array.isArray(this.overlayRuns) ? this.overlayRuns : [];
    const soloId = String(this.soloOverlayRunId ?? "").trim();
    const hoverId = String(this.hoverOverlayRunId ?? "").trim();
    const overlaySeries = [];
    source.forEach((run) => {
      const runId = String(run?.id ?? "").trim();
      if (!runId) return;
      if (run.visible === false) return;
      if (soloId && runId !== soloId) return;
      const primary = pickPrimarySeriesFromGraph(run?.graph);
      const points = normalizeGraphSeriesPoints(primary?.points);
      if (!points.length) return;
      const isActive = runId === this.activeOverlayRunId;
      const isHovered = runId === hoverId;
      const dimmed = Boolean(hoverId) && !isHovered && !isActive;
      const alpha = dimmed ? 0.22 : 1;
      overlaySeries.push({
        id: `overlay:${runId}`,
        color: buildRunColor(run.hue, alpha),
        points,
      });
    });
    this.dotbogi?.setOverlaySeries?.(overlaySeries);
    this.syncDockRangeLabels();
  }

  hydrateRunManagerFromSession({ publish = false } = {}) {
    const payload = this.buildRunManagerBasePayload();
    const runs = Array.isArray(payload?.runs) ? payload.runs : [];
    const lessonId = String(this.lesson?.id ?? "").trim();
    const filtered = runs.filter((row) => {
      const sourceLessonId = String(row?.source?.lessonId ?? row?.source?.lesson_id ?? "").trim();
      if (!lessonId) return true;
      if (!sourceLessonId) return false;
      return sourceLessonId === lessonId;
    });
    this.overlayRuns = filtered.map((row, index) => this.normalizeRunManagerRun(row, index));
    this.overlayRuns.sort((a, b) => normalizeRunManagerLayer(a?.layerIndex, 0) - normalizeRunManagerLayer(b?.layerIndex, 0));
    if (this.activeOverlayRunId && this.findRunManagerIndexById(this.activeOverlayRunId) < 0) {
      this.activeOverlayRunId = "";
    }
    if (this.hoverOverlayRunId && this.findRunManagerIndexById(this.hoverOverlayRunId) < 0) {
      this.hoverOverlayRunId = "";
    }
    if (this.soloOverlayRunId && this.findRunManagerIndexById(this.soloOverlayRunId) < 0) {
      this.soloOverlayRunId = "";
    }
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    if (publish) {
      this.publishRunManagerSession();
    }
  }

  publishRunManagerSession() {
    if (!this.onOverlaySessionChange) return;
    const base = this.buildRunManagerBasePayload();
    const baseRuns = Array.isArray(base?.runs) ? base.runs : [];
    const lessonId = String(this.lesson?.id ?? "").trim();
    const keepRuns = baseRuns.filter((row) => {
      const sourceLessonId = String(row?.source?.lessonId ?? row?.source?.lesson_id ?? "").trim();
      if (!lessonId) return true;
      return sourceLessonId && sourceLessonId !== lessonId;
    });
    const nextRuns = [...keepRuns, ...this.overlayRuns.map((row) => this.serializeRunManagerRun(row))];
    this.onOverlaySessionChange({
      runs: nextRuns,
      compare: toPlainObject(base?.compare, {}),
      viewCombo: toPlainObject(base?.viewCombo, {}),
    });
  }

  flushOverlaySession() {
    this.publishRunManagerSession();
  }

  setInspectorStatus(message = "", { isError = false } = {}) {
    const text = String(message ?? "").trim();
    this.lastInspectorStatusText = text;
    if (!this.runInspectorStatusEl) return;
    this.runInspectorStatusEl.textContent = text || "저장 작업 대기";
    this.runInspectorStatusEl.dataset.status = text ? (isError ? "error" : "ok") : "idle";
  }

  async handleSaveSnapshot() {
    if (!this.onSaveSnapshot) {
      this.setInspectorStatus("결과 저장 기능이 연결되지 않았습니다.", { isError: true });
      return false;
    }
    try {
      const result = await this.onSaveSnapshot();
      if (result === false) {
        this.setInspectorStatus("결과 저장 실패: 저장할 결과가 없습니다.", { isError: true });
        return false;
      }
      this.setInspectorStatus("결과 저장 완료");
      return true;
    } catch (error) {
      this.setInspectorStatus(`결과 저장 실패: ${String(error?.message ?? error)}`, { isError: true });
      return false;
    }
  }

  async handleSaveSession() {
    if (!this.onSaveSession) {
      this.setInspectorStatus("세션 저장 기능이 연결되지 않았습니다.", { isError: true });
      return false;
    }
    try {
      const result = await this.onSaveSession();
      if (result === false) {
        this.setInspectorStatus("세션 저장 실패: 저장할 세션이 없습니다.", { isError: true });
        return false;
      }
      this.setInspectorStatus("세션 저장 완료");
      return true;
    } catch (error) {
      this.setInspectorStatus(`세션 저장 실패: ${String(error?.message ?? error)}`, { isError: true });
      return false;
    }
  }

  bindRunTabUi() {
    this.runTabButtons?.forEach((button) => {
      button?.addEventListener?.("click", () => {
        this.switchRunTab(button.dataset.runTab);
      });
    });
  }

  switchRunTab(tabId) {
    const next = normalizeRunTab(tabId);
    this.activeRunTab = next;
    RUN_TAB_IDS.forEach((tab) => {
      const button = this.root?.querySelector?.(`#run-tab-btn-${tab}`);
      const panel = this.runTabPanels?.get(tab);
      button?.classList?.toggle("active", tab === next);
      panel?.classList?.toggle("hidden", tab !== next);
    });
    return next;
  }

  setLessonOptions(lessons = []) {
    const rows = Array.isArray(lessons) ? lessons : [];
    this.lessonOptions = rows
      .map((row) => ({
        id: String(row?.id ?? "").trim(),
        title: String(row?.title ?? row?.id ?? "").trim(),
      }))
      .filter((row) => row.id)
      .sort((a, b) => String(a.title || a.id).localeCompare(String(b.title || b.id), "ko"));

    if (!this.runLessonSelectEl) return;
    this.runLessonSelectEl.innerHTML = "";
    this.lessonOptions.forEach((row) => {
      const option = document.createElement("option");
      option.value = row.id;
      option.textContent = row.title || row.id;
      this.runLessonSelectEl.appendChild(option);
    });
    const currentId = String(this.lesson?.id ?? "").trim();
    if (currentId && this.lessonOptions.some((row) => row.id === currentId)) {
      this.runLessonSelectEl.value = currentId;
    }
    this.syncRunLessonLoadButtonState();
  }

  syncRunLessonLoadButtonState() {
    if (!this.runLessonLoadBtn || !this.runLessonSelectEl) return;
    const selectedId = String(this.runLessonSelectEl.value ?? "").trim();
    const currentId = String(this.lesson?.id ?? "").trim();
    this.runLessonLoadBtn.disabled = !selectedId || selectedId === currentId;
  }

  async loadSelectedLessonFromTab() {
    if (!this.onSelectLesson || !this.runLessonSelectEl) return false;
    const targetId = String(this.runLessonSelectEl.value ?? "").trim();
    if (!targetId) return false;
    const currentId = String(this.lesson?.id ?? "").trim();
    if (targetId === currentId) return false;
    try {
      await this.onSelectLesson(targetId);
      return true;
    } catch (error) {
      this.setParseWarnings([]);
      this.setHash("-");
      this.lastExecPathHint = `교과 불러오기 실패: ${String(error?.message ?? error)}`;
      this.updateRuntimeHint();
      return false;
    }
  }

  async resetToLessonSource() {
    const lessonId = String(this.lesson?.id ?? "").trim();
    if (!lessonId) return false;
    if (this.onSelectLesson) {
      try {
        await this.onSelectLesson(lessonId);
        return true;
      } catch (error) {
        this.lastExecPathHint = `교과 초기화 실패: ${String(error?.message ?? error)}`;
        this.updateRuntimeHint();
        return false;
      }
    }
    this.baseDdn = String(this.lesson?.ddnText ?? this.baseDdn ?? "");
    if (this.runDdnPreviewEl) {
      this.runDdnPreviewEl.value = this.baseDdn;
    }
    this.lastFormulaDerivedDdn = "";
    if (this.formulaTextEl) this.formulaTextEl.value = "";
    if (this.formulaAxisEl) this.formulaAxisEl.value = "x";
    if (this.formulaXMinEl) this.formulaXMinEl.value = "0";
    if (this.formulaXMaxEl) this.formulaXMaxEl.value = "10";
    if (this.formulaStepEl) this.formulaStepEl.value = "1";
    if (this.formulaDdnPreviewEl) this.formulaDdnPreviewEl.value = "";
    this.setFormulaStatus("");
    await this.restart();
    return true;
  }

  bindFormulaSugarUi() {
    const onInput = () => {
      this.refreshFormulaPreview({ silent: true });
    };
    this.formulaTextEl?.addEventListener("input", onInput);
    this.formulaAxisEl?.addEventListener("input", onInput);
    this.formulaXMinEl?.addEventListener("input", onInput);
    this.formulaXMaxEl?.addEventListener("input", onInput);
    this.formulaStepEl?.addEventListener("input", onInput);
    this.formulaApplyReplaceBtn?.addEventListener("click", () => {
      void this.applyFormulaSugar({ mode: "replace" });
    });
    this.formulaApplyInsertBtn?.addEventListener("click", () => {
      void this.applyFormulaSugar({ mode: "insert" });
    });
  }

  setFormulaStatus(message = "", { isError = false } = {}) {
    if (!this.formulaStatusEl) return;
    const text = String(message ?? "").trim();
    this.formulaStatusEl.textContent = text || "수식 입력 대기";
    this.formulaStatusEl.dataset.status = text ? (isError ? "error" : "ok") : "idle";
  }

  readFormulaSampleFromInputs() {
    const axisVar = String(this.formulaAxisEl?.value ?? "").trim();
    const xMin = Number(this.formulaXMinEl?.value);
    const xMax = Number(this.formulaXMaxEl?.value);
    const step = Number(this.formulaStepEl?.value);
    if (!axisVar || !Number.isFinite(xMin) || !Number.isFinite(xMax) || !Number.isFinite(step)) {
      return null;
    }
    if (xMax < xMin || step <= 0) return null;
    return {
      var: axisVar,
      x_min: xMin,
      x_max: xMax,
      step,
    };
  }

  refreshFormulaPreview({ silent = false } = {}) {
    const parsed = parseFormulaSugarDraft({
      formulaText: this.formulaTextEl?.value ?? "",
      axisVar: this.formulaAxisEl?.value ?? "x",
      xMin: this.formulaXMinEl?.value ?? 0,
      xMax: this.formulaXMaxEl?.value ?? 10,
      step: this.formulaStepEl?.value ?? 1,
    });
    if (!parsed.ok) {
      this.lastFormulaDerivedDdn = "";
      if (this.formulaDdnPreviewEl) {
        this.formulaDdnPreviewEl.value = "";
      }
      if (!silent) {
        this.setFormulaStatus(parsed.error, { isError: true });
      } else if (String(this.formulaTextEl?.value ?? "").trim()) {
        this.setFormulaStatus(parsed.error, { isError: true });
      }
      return null;
    }
    const derivedDdn = buildFormulaSugarDdn(parsed.data);
    this.lastFormulaDerivedDdn = derivedDdn;
    if (this.formulaDdnPreviewEl) {
      this.formulaDdnPreviewEl.value = derivedDdn;
    }
    if (!silent) {
      this.setFormulaStatus("DDN 미리보기 갱신 완료");
    }
    return {
      ...parsed.data,
      derivedDdn,
    };
  }

  applyBaseDdnText(nextDdn = "", { preserveControlValues = true, restart = true } = {}) {
    this.baseDdn = String(nextDdn ?? "");
    this.currentDdn = this.baseDdn;
    if (this.lesson && typeof this.lesson === "object") {
      this.lesson.ddnText = this.baseDdn;
    }
    if (this.runDdnPreviewEl) {
      this.runDdnPreviewEl.value = this.baseDdn;
    }
    const parsed = this.sliderPanel.parseFromDdn(this.baseDdn, {
      preserveValues: Boolean(preserveControlValues),
      maegimControlJson: this.lesson?.maegimControlJson ?? "",
    });
    this.dotbogi.setSeedKeys(parsed.axisKeys);
    this.refreshFormulaPreview({ silent: true });
    if (restart) {
      void this.restart();
    }
  }

  applyFormulaDraftFromSession(sessionLike = null) {
    const row = toPlainObject(sessionLike, {});
    const text = String(row.formula_text ?? row.formulaText ?? "").trim();
    if (this.formulaTextEl && text) {
      this.formulaTextEl.value = text;
    }
    const sample = toPlainObject(row.sample, {});
    if (this.formulaAxisEl && String(sample.var ?? "").trim()) {
      this.formulaAxisEl.value = String(sample.var).trim();
    }
    if (this.formulaXMinEl && Number.isFinite(Number(sample.x_min ?? sample.xMin))) {
      this.formulaXMinEl.value = String(Number(sample.x_min ?? sample.xMin));
    }
    if (this.formulaXMaxEl && Number.isFinite(Number(sample.x_max ?? sample.xMax))) {
      this.formulaXMaxEl.value = String(Number(sample.x_max ?? sample.xMax));
    }
    if (this.formulaStepEl && Number.isFinite(Number(sample.step))) {
      this.formulaStepEl.value = String(Number(sample.step));
    }
    this.refreshFormulaPreview({ silent: true });
  }

  async applyFormulaSugar({ mode = "replace" } = {}) {
    const preview = this.refreshFormulaPreview({ silent: false });
    if (!preview) return false;
    const insertMode = String(mode ?? "").trim().toLowerCase() === "insert";
    const selectionStart = insertMode ? this.runDdnPreviewEl?.selectionStart ?? null : null;
    const selectionEnd = insertMode ? this.runDdnPreviewEl?.selectionEnd ?? null : null;
    const nextDdn = applyFormulaDdnToSource(this.baseDdn, preview.derivedDdn, {
      mode: insertMode ? "insert" : "replace",
      selectionStart,
      selectionEnd,
    });
    this.applyBaseDdnText(nextDdn, { preserveControlValues: true, restart: true });
    this.switchRunTab("ddn");
    this.setFormulaStatus(insertMode ? "수식 DDN을 선택 위치에 삽입했습니다." : "수식 DDN으로 전체 교체했습니다.");
    if (this.onFormulaApplied) {
      try {
        await this.onFormulaApplied({
          formulaText: preview.formulaText,
          axisVar: preview.axisVar,
          outputVar: preview.outputVar,
          expression: preview.expression,
          sample: preview.sample,
          derivedDdn: preview.derivedDdn,
          mode: insertMode ? "insert" : "replace",
        });
      } catch (_) {
        // ignore formula source callback errors
      }
    }
    return true;
  }

  bindViewDockUi() {
    this.root.querySelector("#btn-dock-space-autoscale")?.addEventListener("click", () => {
      this.bogae?.resetView?.();
      this.syncDockRangeLabels();
    });
    this.root.querySelector("#btn-dock-graph-autoscale")?.addEventListener("click", () => {
      this.dotbogi?.resetAxis?.();
      this.syncDockRangeLabels();
    });
    this.root.querySelector("#btn-dock-pan-left")?.addEventListener("click", () => this.panDockTarget(-this.viewPanStep, 0));
    this.root.querySelector("#btn-dock-pan-right")?.addEventListener("click", () => this.panDockTarget(this.viewPanStep, 0));
    this.root.querySelector("#btn-dock-pan-up")?.addEventListener("click", () => this.panDockTarget(0, this.viewPanStep));
    this.root.querySelector("#btn-dock-pan-down")?.addEventListener("click", () => this.panDockTarget(0, -this.viewPanStep));
    this.root.querySelector("#btn-dock-zoom-in")?.addEventListener("click", () => this.zoomDockTarget(0.9));
    this.root.querySelector("#btn-dock-zoom-out")?.addEventListener("click", () => this.zoomDockTarget(1.1));
    this.dockTargetSelectEl?.addEventListener("change", () => {
      this.dockTarget = normalizeDockTarget(this.dockTargetSelectEl?.value);
    });
    this.dockGridCheckEl?.addEventListener("change", () => this.applyDockGuideToggles());
    this.dockAxisCheckEl?.addEventListener("change", () => this.applyDockGuideToggles());
    this.dockOverlayCheckEl?.addEventListener("change", () => {
      const shouldShow = Boolean(this.dockOverlayCheckEl?.checked);
      if (shouldShow) {
        this.overlay?.show?.();
      } else {
        this.overlay?.hide?.();
      }
      this.syncDockGuideToggles();
    });
    this.dockHighlightCheckEl?.addEventListener("change", () => {
      const on = Boolean(this.dockHighlightCheckEl?.checked);
      try {
        document?.body?.setAttribute?.("data-dock-highlight", on ? "on" : "off");
      } catch (_) {
        // ignore highlight toggle errors
      }
    });
    this.root.querySelector("#btn-dock-play")?.addEventListener("click", () => {
      this.dockCursorFollowLive = false;
      this.setPlaybackPaused(false);
    });
    this.root.querySelector("#btn-dock-pause")?.addEventListener("click", () => {
      this.setPlaybackPaused(true);
    });
    this.root.querySelector("#btn-dock-next")?.addEventListener("click", () => {
      this.setPlaybackPaused(true);
      this.advanceDockCursor(1, { loop: this.playbackLoop });
    });
    this.dockLoopCheckEl?.addEventListener("change", () => {
      this.playbackLoop = Boolean(this.dockLoopCheckEl?.checked);
      this.syncDockTimeUi();
    });
    this.dockSpeedSelectEl?.addEventListener("change", () => {
      this.playbackSpeed = normalizeDockSpeed(this.dockSpeedSelectEl?.value);
      this.syncDockTimeUi();
      if (!this.playbackPaused) {
        this.startViewPlaybackTimer();
      }
    });
    this.dockTimeCursorEl?.addEventListener("input", () => {
      this.setDockCursorTick(this.dockTimeCursorEl?.value ?? 0, {
        followLive: false,
      });
    });
  }

  panDockTarget(dx, dy) {
    const target = normalizeDockTarget(this.dockTargetSelectEl?.value || this.dockTarget);
    this.dockTarget = target;
    if (target === "graph") {
      this.dotbogi?.panByRatio?.(dx, dy);
    } else {
      this.bogae?.panByRatio?.(dx, dy);
    }
    this.syncDockRangeLabels();
  }

  zoomDockTarget(factor) {
    const target = normalizeDockTarget(this.dockTargetSelectEl?.value || this.dockTarget);
    this.dockTarget = target;
    if (target === "graph") {
      this.dotbogi?.zoomByFactor?.(factor);
    } else {
      this.bogae?.zoomByFactor?.(factor);
    }
    this.syncDockRangeLabels();
  }

  applyDockGuideToggles() {
    const showGrid = Boolean(this.dockGridCheckEl?.checked);
    const showAxis = Boolean(this.dockAxisCheckEl?.checked);
    this.dotbogi?.setGuides?.({ showGrid, showAxis });
    this.bogae?.setGuides?.({ showGrid, showAxis });
    this.syncDockGuideToggles();
  }

  syncDockGuideToggles() {
    const graphGuides = this.dotbogi?.getGuides?.() ?? {};
    const spaceGuides = this.bogae?.getGuides?.() ?? {};
    const showGrid = Boolean(graphGuides.showGrid ?? spaceGuides.showGrid ?? false);
    const showAxis = Boolean(graphGuides.showAxis ?? spaceGuides.showAxis ?? false);
    if (this.dockGridCheckEl) {
      this.dockGridCheckEl.checked = showGrid;
    }
    if (this.dockAxisCheckEl) {
      this.dockAxisCheckEl.checked = showAxis;
    }
    if (this.dockOverlayCheckEl) {
      this.dockOverlayCheckEl.checked = Boolean(this.overlay?.visible);
    }
    if (this.dockTargetSelectEl) {
      this.dockTargetSelectEl.value = normalizeDockTarget(this.dockTarget);
    }
  }

  syncDockRangeLabels({ spaceRange = null, graphAxis = null } = {}) {
    const nextSpaceRange = spaceRange || this.bogae?.getCurrentRange?.() || null;
    const nextGraphRange = graphAxis || this.dotbogi?.getCurrentAxis?.() || null;
    if (this.dockSpaceRangeEl) {
      this.dockSpaceRangeEl.textContent = `보개: ${formatAxisRange(nextSpaceRange)}`;
    }
    if (this.dockGraphRangeEl) {
      this.dockGraphRangeEl.textContent = `그래프: ${formatAxisRange(nextGraphRange)}`;
    }
  }

  clearViewPlaybackTimer() {
    if (this.viewPlaybackTimer === null || this.viewPlaybackTimer === undefined) return;
    try {
      clearInterval(this.viewPlaybackTimer);
    } catch (_) {
      // ignore timer clear errors
    }
    this.viewPlaybackTimer = null;
  }

  resolveViewPlaybackIntervalMs() {
    const speed = normalizeDockSpeed(this.playbackSpeed);
    return Math.max(60, Math.round(240 / speed));
  }

  startViewPlaybackTimer() {
    this.clearViewPlaybackTimer();
    if (this.playbackPaused || !this.screenVisible) return;
    const intervalMs = this.resolveViewPlaybackIntervalMs();
    this.viewPlaybackTimer = setInterval(() => {
      this.advanceDockCursor(1, { loop: this.playbackLoop });
    }, intervalMs);
  }

  setPlaybackPaused(paused = true) {
    this.playbackPaused = Boolean(paused);
    if (this.playbackPaused) {
      this.clearViewPlaybackTimer();
    } else {
      this.startViewPlaybackTimer();
    }
    this.syncDockTimeUi();
  }

  resolveDockCursorMaxTick() {
    const runtimeTick = Math.max(0, Number(this.runtimeTickCounter) || 0);
    const timelineTick = Math.max(0, Number(this.dotbogi?.getTimelineLength?.() ?? 0) - 1);
    return Math.max(runtimeTick, timelineTick);
  }

  clampDockCursorTick(rawTick) {
    const maxTick = this.resolveDockCursorMaxTick();
    const raw = Number(rawTick);
    if (!Number.isFinite(raw)) return Math.min(this.dockCursorTick, maxTick);
    return Math.max(0, Math.min(maxTick, Math.trunc(raw)));
  }

  setDockCursorTick(rawTick, { followLive = null, syncUi = true } = {}) {
    const nextTick = this.clampDockCursorTick(rawTick);
    this.dockCursorTick = nextTick;
    if (typeof followLive === "boolean") {
      this.dockCursorFollowLive = followLive;
    }
    this.dotbogi?.setPlaybackCursor?.(nextTick, { render: true });
    if (syncUi) {
      this.syncDockTimeUi();
    }
    return nextTick;
  }

  advanceDockCursor(step = 1, { loop = true } = {}) {
    const maxTick = this.resolveDockCursorMaxTick();
    if (maxTick <= 0) {
      this.setDockCursorTick(0, { followLive: false });
      return false;
    }
    const delta = Math.max(1, Math.trunc(Number(step) || 1));
    let nextTick = this.dockCursorTick + delta;
    if (nextTick > maxTick) {
      if (loop) {
        nextTick = 0;
      } else {
        nextTick = maxTick;
        this.setPlaybackPaused(true);
      }
    }
    this.setDockCursorTick(nextTick, {
      followLive: false,
    });
    return true;
  }

  resolveDockCursorTimeValue(cursorTick = 0) {
    const sample = this.dotbogi?.getTimelineSampleAt?.(cursorTick);
    const values = sample?.values;
    if (!values || typeof values !== "object") return this.runtimeTimeValue;
    const observation = {
      values,
      all_values: values,
      channels: Object.keys(values).map((key) => ({ key })),
    };
    const t = readNumericObservationValue(observation, ["t", "time", "tick", "프레임수", "시간"]);
    return Number.isFinite(t) ? t : this.runtimeTimeValue;
  }

  syncDockTimeUi() {
    if (this.dockSpeedSelectEl) {
      this.dockSpeedSelectEl.value = String(normalizeDockSpeed(this.playbackSpeed));
    }
    if (this.dockLoopCheckEl) {
      this.dockLoopCheckEl.checked = Boolean(this.playbackLoop);
    }
    const cursorMaxTick = this.resolveDockCursorMaxTick();
    if (this.dockCursorFollowLive) {
      this.dockCursorTick = cursorMaxTick;
    } else {
      this.dockCursorTick = this.clampDockCursorTick(this.dockCursorTick);
    }
    this.dotbogi?.setPlaybackCursor?.(this.dockCursorTick, { render: false });
    const cursorMax = Math.max(100, cursorMaxTick);
    if (this.dockTimeCursorEl) {
      this.dockTimeCursorEl.max = String(cursorMax);
      this.dockTimeCursorEl.value = String(this.dockCursorTick);
    }
    const cursorTimeValue = this.resolveDockCursorTimeValue(this.dockCursorTick);
    const tText = cursorTimeValue === null ? "-" : formatStatusNumber(cursorTimeValue, 3);
    if (this.dockTimeTextEl) {
      const mode = this.playbackPaused ? "일시정지" : "재생(보기)";
      this.dockTimeTextEl.textContent = `${mode} · t=${tText} / 틱=${this.dockCursorTick} / 끝=${cursorMaxTick}`;
    }
  }

  installRuntimeTableResizeObserver() {
    const hostWindow = globalThis?.window;
    if (this.runtimeTableResizeObserver && typeof this.runtimeTableResizeObserver.disconnect === "function") {
      this.runtimeTableResizeObserver.disconnect();
    }
    this.runtimeTableResizeObserver = null;
    if (this.runtimeTableResizeFallbackInstalled && typeof hostWindow?.removeEventListener === "function") {
      hostWindow.removeEventListener("resize", this.boundRuntimeTableResizeHandler);
    }
    this.runtimeTableResizeFallbackInstalled = false;
    if (!this.runtimeTablePanelEl || typeof globalThis?.ResizeObserver !== "function") {
      if (this.runtimeTablePanelEl && typeof hostWindow?.addEventListener === "function") {
        hostWindow.addEventListener("resize", this.boundRuntimeTableResizeHandler);
        this.runtimeTableResizeFallbackInstalled = true;
      }
      return;
    }
    this.runtimeTableResizeObserver = new globalThis.ResizeObserver(() => {
      this.refreshRuntimeTableForCurrentWidth();
    });
    this.runtimeTableResizeObserver.observe(this.runtimeTablePanelEl);
  }

  renderCurrentRuntimeTable(table, { maxChars = null } = {}) {
    const normalizedTable = normalizeRuntimeTableView(table ?? null);
    const hasTable = renderRuntimeTable(this.runtimeTableEl, table ?? null, { maxChars });
    if (this.runtimeTableMetaEl) {
      this.runtimeTableMetaEl.textContent = hasTable ? summarizeRuntimeTableView(normalizedTable) : "";
    }
    this.lastRuntimeTableCellMaxChars = hasTable
      ? resolveRuntimeTableCellMaxChars(this.runtimeTableEl, { maxChars })
      : 0;
    return hasTable;
  }

  refreshRuntimeTableForCurrentWidth({ force = false } = {}) {
    if (!this.screenVisible) return false;
    const table = this.lastRuntimeDerived?.views?.table ?? null;
    if (!table) return false;
    const nextMaxChars = resolveRuntimeTableCellMaxChars(this.runtimeTableEl);
    if (!force && nextMaxChars === this.lastRuntimeTableCellMaxChars) {
      return false;
    }
    this.renderCurrentRuntimeTable(table, { maxChars: nextMaxChars });
    return true;
  }

  persistUiPrefs() {
    writeStorageJson(RUN_UI_PREFS_STORAGE_KEY, this.uiPrefs);
    try {
      window.dispatchEvent(new CustomEvent("seamgrim:run-prefs-changed"));
    } catch (_) {
      // ignore event dispatch errors
    }
  }

  getLessonUiPref(lessonId, { create = false } = {}) {
    const id = String(lessonId ?? "").trim();
    if (!id) return null;
    if (!this.uiPrefs.lessons || typeof this.uiPrefs.lessons !== "object") {
      this.uiPrefs.lessons = {};
    }
    if (!this.uiPrefs.lessons[id] && create) {
      this.uiPrefs.lessons[id] = {};
    }
    return this.uiPrefs.lessons[id] ?? null;
  }

  restoreLessonUiPrefs(lessonId) {
    const pref = this.getLessonUiPref(lessonId, { create: false });
    this.dotbogi.setSelectedAxes({
      xKey: String(pref?.selectedXKey ?? ""),
      yKey: String(pref?.selectedYKey ?? ""),
    });
  }

  saveCurrentLessonUiPrefs() {
    const lessonId = String(this.lesson?.id ?? "").trim();
    if (!lessonId) return;
    const pref = this.getLessonUiPref(lessonId, { create: true });
    const selected = this.dotbogi?.getSelectedAxes?.() ?? {};
    pref.selectedXKey = String(selected.xKey ?? "");
    pref.selectedYKey = String(selected.yKey ?? "");
    this.persistUiPrefs();
  }

  exportRuntimeSessionState() {
    const selectedAxes = this.dotbogi?.getSelectedAxes?.() ?? {};
    const graphGuides = this.dotbogi?.getGuides?.() ?? {};
    const spaceGuides = this.bogae?.getGuides?.() ?? {};
    const controls = toPlainObject(this.sliderPanel?.getValues?.() ?? {}, {});
    const tick = Math.max(0, Number(this.runtimeTickCounter) || 0);
    const cursor = this.clampDockCursorTick(this.dockCursorTick);
    const cursorTimeValue = this.resolveDockCursorTimeValue(cursor);
    const viewTarget = normalizeDockTarget(this.dockTargetSelectEl?.value || this.dockTarget);
    const graphSample = toPlainObject(this.lastRuntimeDerived?.views?.graph?.sample, {});
    const formulaSample = this.readFormulaSampleFromInputs();
    const effectiveSample = Object.keys(graphSample).length > 0
      ? graphSample
      : (formulaSample ?? {});
    return {
      formula_text: String(this.formulaTextEl?.value ?? ""),
      controls,
      sample: effectiveSample,
      time: {
        enabled: true,
        t_min: 0,
        t_max: Math.max(1, Number(this.resolveDockCursorMaxTick()) || Number(this.runtimeTimeValue) || 1),
        step: 1,
        now: cursorTimeValue === null ? 0 : Number(cursorTimeValue),
        interval: 300,
        loop: Boolean(this.playbackLoop),
        tick,
        cursor,
        speed: normalizeDockSpeed(this.playbackSpeed),
        paused: Boolean(this.playbackPaused),
        playing: !this.playbackPaused,
      },
      view: {
        auto: false,
        panX: 0,
        panY: 0,
        zoom: 1,
        range: null,
        showGrid: Boolean(graphGuides.showGrid ?? spaceGuides.showGrid),
        showAxis: Boolean(graphGuides.showAxis ?? spaceGuides.showAxis),
        graph: {
          auto_fit: false,
          axis: this.dotbogi?.getCurrentAxis?.() ?? null,
          guides: {
            showGrid: Boolean(graphGuides.showGrid),
            showAxis: Boolean(graphGuides.showAxis),
          },
          selected_axes: {
            x: String(selectedAxes.xKey ?? ""),
            y: String(selectedAxes.yKey ?? ""),
          },
        },
        space2d: {
          auto_fit: false,
          range: this.bogae?.getCurrentRange?.() ?? null,
          guides: {
            showGrid: Boolean(spaceGuides.showGrid),
            showAxis: Boolean(spaceGuides.showAxis),
          },
        },
        dock: {
          target: viewTarget,
          highlight: Boolean(this.dockHighlightCheckEl?.checked),
        },
      },
      ui_layout: {
        screen_mode: "run",
        workspace_mode: "basic",
        main_tab: this.activeRunTab === "lesson" ? "lesson-tab" : "tools-tab",
        active_view: viewTarget === "graph" ? "view-graph" : "view-2d",
        run_tab: normalizeRunTab(this.activeRunTab),
      },
      active_run_id: String(this.activeOverlayRunId ?? "").trim(),
      last_state_hash: String(this.lastRuntimeHash ?? "").trim(),
    };
  }

  sessionMatchesCurrentLesson(sessionLike) {
    const row = toPlainObject(sessionLike, {});
    const sessionLesson = String(row.lesson ?? "").trim();
    const currentLesson = String(this.lesson?.id ?? "").trim();
    if (!sessionLesson || !currentLesson) return true;
    return sessionLesson === currentLesson;
  }

  applyRuntimeSessionState(sessionLike = null) {
    const row = toPlainObject(sessionLike, {});
    if (!this.sessionMatchesCurrentLesson(row)) return false;

    const uiLayout = toPlainObject(row.ui_layout ?? row.uiLayout, {});
    const runTab = normalizeRunTab(uiLayout.run_tab ?? uiLayout.runTab ?? this.activeRunTab);
    this.switchRunTab(runTab);

    const view = toPlainObject(row.view, {});
    const graphView = toPlainObject(view.graph, {});
    const graphGuides = toPlainObject(graphView.guides, {});
    this.dotbogi?.setGuides?.({
      showGrid: typeof graphGuides.showGrid === "boolean" ? graphGuides.showGrid : null,
      showAxis: typeof graphGuides.showAxis === "boolean" ? graphGuides.showAxis : null,
    });
    const selectedAxes = toPlainObject(graphView.selected_axes ?? graphView.selectedAxes, {});
    this.dotbogi?.setSelectedAxes?.({
      xKey: String(selectedAxes.x ?? selectedAxes.xKey ?? ""),
      yKey: String(selectedAxes.y ?? selectedAxes.yKey ?? ""),
    });
    if (graphView.auto_fit === true || graphView.autoFit === true) {
      this.dotbogi?.resetAxis?.();
    } else {
      this.dotbogi?.setAxis?.(graphView.axis ?? graphView.range ?? null);
    }

    const space2dView = toPlainObject(row.space2d_view ?? row.space2dView ?? view.space2d, {});
    const space2dGuides = toPlainObject(space2dView.guides, {});
    this.bogae?.setGuides?.({
      showGrid: typeof space2dGuides.showGrid === "boolean" ? space2dGuides.showGrid : null,
      showAxis: typeof space2dGuides.showAxis === "boolean" ? space2dGuides.showAxis : null,
    });
    if (space2dView.auto_fit === true || space2dView.autoFit === true) {
      this.bogae?.resetView?.();
    } else {
      this.bogae?.setRange?.(space2dView.range ?? space2dView.axis ?? null);
    }

    const dockView = toPlainObject(view.dock, {});
    this.dockTarget = normalizeDockTarget(dockView.target ?? this.dockTarget);
    if (this.dockTargetSelectEl) {
      this.dockTargetSelectEl.value = this.dockTarget;
    }
    if (this.dockHighlightCheckEl && typeof dockView.highlight === "boolean") {
      this.dockHighlightCheckEl.checked = dockView.highlight;
    }

    const time = toPlainObject(row.time, {});
    if (typeof time.loop === "boolean") {
      this.playbackLoop = Boolean(time.loop);
    }
    if (typeof time.paused === "boolean") {
      this.playbackPaused = Boolean(time.paused);
    } else if (typeof time.playing === "boolean") {
      this.playbackPaused = !time.playing;
    }
    this.playbackSpeed = normalizeDockSpeed(time.speed ?? this.playbackSpeed);
    const hasSavedCursor = Object.prototype.hasOwnProperty.call(time, "cursor") || Object.prototype.hasOwnProperty.call(time, "tick");
    const savedTick = Math.max(0, Number(time.tick) || 0);
    const savedCursor = Math.max(0, Number(time.cursor ?? savedTick) || 0);
    this.dockCursorTick = savedCursor;
    this.dockCursorFollowLive = !hasSavedCursor;
    this.dotbogi?.setPlaybackCursor?.(savedCursor, { render: false });

    const activeRunId = String(row.active_run_id ?? "").trim();
    if (activeRunId && this.findRunManagerIndexById(activeRunId) >= 0) {
      this.activeOverlayRunId = activeRunId;
    }
    this.applyFormulaDraftFromSession(row);

    this.syncDockGuideToggles();
    this.applyDockGuideToggles();
    this.syncDockRangeLabels();
    this.syncDockTimeUi();
    if (!this.playbackPaused) {
      this.startViewPlaybackTimer();
    }
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    return true;
  }

  applyLessonLayoutProfile(lesson) {
    const profile = resolveRunLayoutProfile(lesson?.requiredViews ?? []);
    const dockOrder = resolveRunDockPanelOrder(lesson?.requiredViews ?? []);
    this.lessonLayoutProfile = profile;
    const mode = String(profile.mode ?? "split");
    const classList = this.layoutEl?.classList;
    if (classList?.toggle) {
      classList.toggle("run-layout--split", mode === "split");
      classList.toggle("run-layout--dock-only", mode === "dock_only");
      classList.toggle("run-layout--space-primary", mode === "space_primary");
    }
    if (this.root?.dataset) {
      this.root.dataset.requiredViews = profile.families.join(",");
      this.root.dataset.runLayoutMode = mode;
      this.root.dataset.runDockOrder = dockOrder.join(",");
    }
    this.applyDockPanelOrder(dockOrder);
    this.syncOverlayToggleState();
    this.syncDockPanelVisibility();
    return profile;
  }

  applyDockPanelOrder(order) {
    const dockOrder = Array.isArray(order) ? order : resolveRunDockPanelOrder(this.lesson?.requiredViews ?? []);
    const panelMap = {
      graph: this.graphPanelEl,
      table: this.runtimeTablePanelEl,
      text: this.runtimeTextPanelEl,
    };
    dockOrder.forEach((panel, index) => {
      const el = panelMap[panel];
      if (!el?.style) return;
      el.style.order = String(index);
    });
  }

  renderRuntimeText(markdown) {
    return this.renderRuntimeTextContent({ markdown, structure: null, graph: null });
  }

  setRuntimePreviewViewModel(viewModel) {
    this.lastPreviewViewModel = viewModel ?? null;
    applyPreviewViewModelMetadata(this.runtimeTextPanelEl, this.lastPreviewViewModel);
  }

  renderRuntimeTextContent({ markdown = "", structure = null, graph = null } = {}) {
    const text = String(markdown ?? "").trim();
    this.lastRuntimeTextMarkdown = text;
    if (this.runtimeTextBodyEl) {
      const graphResult = buildFamilyPreviewResult({
        family: "graph",
        payload: graph,
        html: buildRuntimeGraphPreviewHtml(graph),
      });
      const structureResult = buildFamilyPreviewResult({
        family: "structure",
        payload: structure,
        html: buildRuntimeStructurePreviewHtml(structure),
      });
      const previewCollection = buildPreviewResultCollection([graphResult, structureResult], {
        preferredFamilies: ["graph", "structure", "text"],
        summaryClassName: "runtime-preview-summary",
        cardClassName: "runtime-preview-card",
      });
      const previewViewModel = buildPreviewViewModel(previewCollection, {
        sourceId: String(this.lesson?.id ?? "run.runtime"),
      });
      this.setRuntimePreviewViewModel(previewViewModel);
      const previewHtml = String(previewCollection?.html ?? "");
      const textHtml = text ? markdownToHtml(text) : "";
      if (previewHtml && textHtml) {
        this.runtimeTextBodyEl.innerHTML = `${previewHtml}<div class="runtime-text-markdown">${textHtml}</div>`;
      } else if (previewHtml) {
        this.runtimeTextBodyEl.innerHTML = previewHtml;
      } else if (textHtml) {
        this.setRuntimePreviewViewModel(null);
        this.runtimeTextBodyEl.innerHTML = textHtml;
      } else {
        this.setRuntimePreviewViewModel(null);
        this.runtimeTextBodyEl.innerHTML = '<div class="runtime-text-empty">설명 출력 없음</div>';
      }
    }
    this.updateRuntimeHint();
    this.syncDockPanelVisibility();
    this.syncOverlayToggleState();
    return Boolean(text || structure || graph);
  }

  syncDockPanelVisibility(runtimeDerived = this.lastRuntimeDerived) {
    const visible = resolveRunDockPanelVisibility(this.lesson?.requiredViews ?? [], {
      runtimeDerived,
      textMarkdown: this.lastRuntimeTextMarkdown,
    });
    this.graphPanelEl?.classList?.toggle("hidden", !visible.graph);
    this.runtimeTablePanelEl?.classList?.toggle("hidden", !visible.table);
    this.runtimeTextPanelEl?.classList?.toggle("hidden", !visible.text);
    return visible;
  }

  syncOverlayToggleState() {
    if (!this.overlayToggleBtn) return;
    const hasSpatial = Boolean(this.lessonLayoutProfile?.hasSpatial);
    const hasMarkdown = Boolean(
      String(this.lastOverlayMarkdown ?? "").trim() || String(this.lastRuntimeTextMarkdown ?? "").trim(),
    );
    const enabled = hasSpatial && hasMarkdown;
    this.overlayToggleBtn.disabled = !enabled;
    this.overlayToggleBtn.title = enabled
      ? "설명 overlay 보기"
      : hasMarkdown
        ? "이 교과의 설명은 오른쪽 패널에 표시됩니다."
        : "표시할 설명이 없습니다.";
    if (this.dockOverlayCheckEl) {
      this.dockOverlayCheckEl.disabled = !enabled;
      this.dockOverlayCheckEl.checked = enabled && Boolean(this.overlay?.visible);
    }
  }

  isEditableTarget(target) {
    if (!(target instanceof Element)) return false;
    if (target instanceof HTMLInputElement) return true;
    if (target instanceof HTMLTextAreaElement) return true;
    if (target instanceof HTMLSelectElement) return true;
    if (target.closest("input, textarea, select, [contenteditable='true']")) return true;
    return target.isContentEditable;
  }

  handleViewHotkeys(event) {
    if (!this.root || this.root.classList.contains("hidden")) return;
    if (this.isEditableTarget(event.target)) return;

    const key = String(event.key ?? "");
    const pan = this.viewPanStep;
    const graphHotkey = event.shiftKey && !event.ctrlKey && !event.metaKey && !event.altKey;
    const bogaeHotkey = event.shiftKey && (event.ctrlKey || event.metaKey) && !event.altKey;

    if (graphHotkey) {
      if (key === "ArrowLeft") {
        event.preventDefault();
        this.dotbogi.panByRatio(-pan, 0);
        return;
      }
      if (key === "ArrowRight") {
        event.preventDefault();
        this.dotbogi.panByRatio(pan, 0);
        return;
      }
      if (key === "ArrowUp") {
        event.preventDefault();
        this.dotbogi.panByRatio(0, pan);
        return;
      }
      if (key === "ArrowDown") {
        event.preventDefault();
        this.dotbogi.panByRatio(0, -pan);
        return;
      }
      if (key === "+" || key === "=") {
        event.preventDefault();
        this.dotbogi.zoomByFactor(0.9);
        return;
      }
      if (key === "-" || key === "_") {
        event.preventDefault();
        this.dotbogi.zoomByFactor(1.1);
        return;
      }
    }

    if (bogaeHotkey) {
      if (key === "ArrowLeft") {
        event.preventDefault();
        this.bogae.panByRatio(-pan, 0);
        return;
      }
      if (key === "ArrowRight") {
        event.preventDefault();
        this.bogae.panByRatio(pan, 0);
        return;
      }
      if (key === "ArrowUp") {
        event.preventDefault();
        this.bogae.panByRatio(0, pan);
        return;
      }
      if (key === "ArrowDown") {
        event.preventDefault();
        this.bogae.panByRatio(0, -pan);
        return;
      }
      if (key === "+" || key === "=") {
        event.preventDefault();
        this.bogae.zoomByFactor(0.9);
        return;
      }
      if (key === "-" || key === "_") {
        event.preventDefault();
        this.bogae.zoomByFactor(1.1);
      }
    }
  }

  isRuntimeInputEnabled() {
    if (!this.screenVisible) return false;
    if (!this.root || this.root.classList.contains("hidden")) return false;
    return Boolean(this.wasmState?.inputEnabled ?? true);
  }

  clearRuntimeInputState() {
    this.heldInputMask = 0;
    this.pulsePressedMask = 0;
    this.lastInputToken = "";
  }

  handleRuntimeInputKeyDown(event) {
    if (!this.isRuntimeInputEnabled()) return;
    if (!event || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if (this.isEditableTarget(event.target)) return;
    const token = runtimeInputTokenFromKeyboardEvent(event);
    if (!token) return;
    const bit = runtimeInputBitFromToken(token);
    if (!bit) return;
    const wasHeld = (this.heldInputMask & bit) !== 0;
    this.heldInputMask = (this.heldInputMask | bit) & RUNTIME_INPUT_MASK_LIMIT;
    if (!wasHeld) {
      this.pulsePressedMask = (this.pulsePressedMask | bit) & RUNTIME_INPUT_MASK_LIMIT;
    }
    this.lastInputToken = token;
    event.preventDefault();
  }

  handleRuntimeInputKeyUp(event) {
    const token = runtimeInputTokenFromKeyboardEvent(event);
    if (!token) return;
    const bit = runtimeInputBitFromToken(token);
    if (!bit) return;
    this.heldInputMask = (this.heldInputMask & ~bit) & RUNTIME_INPUT_MASK_LIMIT;
    if (!this.isRuntimeInputEnabled()) return;
    if (!event || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if (this.isEditableTarget(event.target)) return;
    event.preventDefault();
  }

  setHash(hashText) {
    this.lastRuntimeHash = String(hashText ?? "-");
    this.renderInspectorMeta();
  }

  setParseWarnings(warnings) {
    const normalized = normalizeParseWarnings(warnings);
    this.lastParseWarnings = normalized;
    if (this.wasmState && typeof this.wasmState === "object") {
      this.wasmState.parseWarnings = normalized;
    }
    this.renderInspectorMeta();
  }

  renderInspectorMeta() {
    if (!this.runInspectorMetaEl) return;
    const external =
      this.onGetInspectorContext && typeof this.onGetInspectorContext === "function"
        ? this.onGetInspectorContext() ?? {}
        : {};
    const report = buildInspectorReport({
      lesson: this.lesson,
      lastRuntimeHash: this.lastRuntimeHash,
      parseWarnings: this.lastParseWarnings,
      runtimeTickCounter: this.runtimeTickCounter,
      runtimeTimeValue: this.runtimeTimeValue,
      playbackPaused: this.playbackPaused,
      playbackSpeed: this.playbackSpeed,
      lastExecPathHint: this.lastExecPathHint,
      lastRuntimeDerived: this.lastRuntimeDerived,
      sceneSummary: external.sceneSummary ?? null,
      snapshotV0: external.snapshotV0 ?? null,
      sessionV0: external.sessionV0 ?? null,
      uiPrefsLessons: this.uiPrefs?.lessons ?? null,
    });
    this.lastInspectorReport = report;
    const bridgeOk = Boolean(report?.bridge_check?.ok);
    this.runInspectorMetaEl.dataset.bridgeCheck = bridgeOk ? "ok" : "fail";
    this.runInspectorMetaEl.textContent = formatInspectorReportText(report);
  }

  updateRunSummaryFromPrefs() {
    if (!this.lastSummaryEl) return;
    const lessonId = String(this.lesson?.id ?? "").trim();
    const pref = lessonId ? this.getLessonUiPref(lessonId, { create: false }) : null;
    this.lastSummaryEl.textContent = buildRunSummaryText(pref);
  }

  loadLesson(lesson, { launchKind = "manual" } = {}) {
    this.lesson = lesson;
    this.lastLaunchKind = normalizeRunLaunchKind(launchKind);
    this.baseDdn = String(lesson?.ddnText ?? "");
    this.currentDdn = this.baseDdn;
    this.lastRuntimeHash = "-";
    this.lastRuntimeSnapshotKey = "";
    this.lastRuntimeDerived = null;
    this.lastExecPathHint = "";
    this.lastSpace2dMode = "none";
    this.lastRuntimeHintText = "";
    this.setRuntimePreviewViewModel(null);
    this.lastOverlayMarkdown = "";
    this.lastRuntimeTextMarkdown = "";
    this.lastParseWarnings = [];
    this.runtimeTickCounter = 0;
    this.runtimeTimeValue = null;
    this.playbackPaused = false;
    this.playbackSpeed = normalizeDockSpeed(this.playbackSpeed);
    this.playbackLoop = true;
    this.dockCursorTick = 0;
    this.dockCursorFollowLive = true;
    this.clearViewPlaybackTimer();
    this.serverPlayback = null;

    if (this.titleEl) {
      this.titleEl.textContent = lesson?.title || lesson?.id || "-";
    }
    if (this.runDdnPreviewEl) {
      this.runDdnPreviewEl.value = this.baseDdn;
    }
    if (this.runLessonSummaryEl) {
      const lessonId = String(lesson?.id ?? "-").trim() || "-";
      const title = String(lesson?.title ?? "").trim() || "-";
      const description = String(lesson?.description ?? "").trim();
      const views = normalizeRunRequiredViews(lesson?.requiredViews ?? []);
      const ddnMetaName = String(lesson?.ddnMetaHeader?.name ?? "").trim();
      const ddnMetaDesc = String(lesson?.ddnMetaHeader?.desc ?? "").trim();
      const lines = [
        `id: ${lessonId}`,
        `제목: ${title}`,
        `설명: ${description || "-"}`,
        `required_views: ${views.length ? views.join(", ") : "-"}`,
      ];
      if (ddnMetaName || ddnMetaDesc) {
        lines.push(`DDN 헤더 이름: ${ddnMetaName || "-"}`);
        lines.push(`DDN 헤더 설명: ${ddnMetaDesc || "-"}`);
      }
      this.runLessonSummaryEl.textContent = lines.join("\n");
    }
    if (this.runLessonSelectEl) {
      this.runLessonSelectEl.value = String(lesson?.id ?? "").trim();
    }
    if (this.runResetToLessonBtn) {
      this.runResetToLessonBtn.disabled = !String(lesson?.id ?? "").trim();
    }
    this.applyLessonLayoutProfile(lesson);
    this.switchRunTab(this.activeRunTab || "lesson");
    this.syncRunLessonLoadButtonState();

    this.lastOverlayMarkdown = String(lesson?.textMd ?? "");
    this.overlay.setContent(this.lastOverlayMarkdown);
    this.renderRuntimeText(this.lastOverlayMarkdown);
    if (this.isSimCorePolicyEnabled()) {
      this.overlay.show();
    } else {
      this.overlay.hide();
    }
    const parsed = this.sliderPanel.parseFromDdn(this.baseDdn, {
      preserveValues: false,
      maegimControlJson: lesson?.maegimControlJson ?? "",
    });
    this.dotbogi.setSeedKeys(parsed.axisKeys);
    this.dotbogi.setPreferredXKey(parsed.defaultXAxisKey);
    this.dotbogi.setPreferredYKey(parsed.defaultAxisKey);
    this.dotbogi.clearTimeline();
    this.restoreLessonUiPrefs(lesson?.id);
    this.bogae.resetView();
    this.dotbogi.resetAxis();
    this.hydrateRunManagerFromSession({ publish: false });
    const restoredFromSession = this.applyRuntimeSessionState(this.getRuntimeSessionV0?.() ?? null);
    if (!restoredFromSession) {
      this.refreshFormulaPreview({ silent: true });
    }
    this.syncDockGuideToggles();
    this.applyDockGuideToggles();
    this.syncDockRangeLabels();
    this.syncDockTimeUi();
    if (!restoredFromSession) {
      this.setInspectorStatus(this.lastInspectorStatusText || "저장 작업 대기");
    }
    this.saveCurrentLessonUiPrefs();
    this.updateRunSummaryFromPrefs();
    this.updateRuntimeHint();
    this.setHash("-");
    void this.restart();
  }

  updateRuntimeHint(execPathText = "", runtimeDerived = null) {
    if (!this.runtimeStatusEl) return;
    const nextPath = String(execPathText ?? "").trim();
    if (nextPath) {
      this.lastExecPathHint = nextPath;
    }

    const controlCount = Array.isArray(this.sliderPanel?.specs) ? this.sliderPanel.specs.length : 0;
    const segments = [controlCount > 0 ? `control 채비: ${controlCount}개` : "control: -"];
    if (this.lastExecPathHint) {
      segments.push(this.lastExecPathHint);
    }

    const mode = String(this.lastSpace2dMode ?? "none").trim().toLowerCase();
    const shapeModeLabel = mode === "native" ? "native" : mode === "fallback" ? "fallback" : "none";
    segments.push(`보개: ${shapeModeLabel}`);
    const parseWarningSummary = formatParseWarningSummary(this.lastParseWarnings);
    if (parseWarningSummary) {
      segments.push(parseWarningSummary);
    }

    const viewFamilies = Array.isArray(runtimeDerived?.views?.families)
      ? runtimeDerived.views.families
      : Array.isArray(this.lastRuntimeDerived?.views?.families)
        ? this.lastRuntimeDerived.views.families
        : [];
    if (viewFamilies.length) {
      segments.push(`보기: ${viewFamilies.join("+")}`);
    }
    const previewSummary = String(this.lastPreviewViewModel?.summaryText ?? "").trim();
    if (previewSummary) {
      segments.push(`대표보기: ${previewSummary}`);
    }

    const obs = runtimeDerived?.observation ?? this.lastRuntimeDerived?.observation ?? null;
    const t = readNumericObservationValue(obs, ["t", "time", "tick", "프레임수", "시간"]);
    this.runtimeTimeValue = Number.isFinite(t) ? t : this.runtimeTimeValue;
    const theta = readNumericObservationValue(obs, ["theta", "각도", "theta_rad", "angle", "rad"]);
    const omega = readNumericObservationValue(obs, ["omega", "각속도", "angular_velocity"]);
    const obsParts = [];
    const tText = formatStatusNumber(t);
    if (tText) obsParts.push(`t=${tText}`);
    const thetaText = formatStatusNumber(theta);
    if (thetaText) obsParts.push(`theta=${thetaText}`);
    const omegaText = formatStatusNumber(omega);
    if (omegaText) obsParts.push(`omega=${omegaText}`);
    if (obsParts.length) {
      segments.push(obsParts.join(" "));
    }

    const nextText = segments.join(" · ");
    if (nextText === this.lastRuntimeHintText) {
      this.syncDockTimeUi();
      this.renderInspectorMeta();
      return;
    }
    this.lastRuntimeHintText = nextText;
    this.runtimeStatusEl.textContent = nextText;
    this.syncDockTimeUi();
    this.renderInspectorMeta();
  }

  async runViaExecServer(ddnText) {
    if (!this.allowServerFallback) {
      return null;
    }
    const payload = {
      ddn_text: String(ddnText ?? ""),
      madi: 420,
    };
    try {
      const response = await fetch("/api/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        cache: "no-cache",
        body: JSON.stringify(payload),
      });
      if (!response.ok) return null;
      const data = await response.json();
      if (!data || data.ok !== true) return null;
      const views = {
        graph: data.graph ?? null,
        space2d: data.space2d ?? null,
        text: data.text ?? null,
        table: data.table ?? null,
        structure: data.structure ?? null,
      };
      const observation = buildObservationFromGraph(data.graph);
      const serverPlayback = buildServerPlaybackPlan(data.graph);
      return { observation, views, serverPlayback };
    } catch (_) {
      return null;
    }
  }

  collectServerPlaybackExtraValues() {
    const raw = this.sliderPanel?.getValues?.() ?? {};
    if (!raw || typeof raw !== "object") return {};
    const out = {};
    const length = parseFiniteNumericValue(raw.L);
    if (length !== null) {
      out.L = length;
    }
    return out;
  }

  startServerPlayback(derived) {
    const plan = derived?.serverPlayback;
    if (!plan || !Array.isArray(plan.frames) || plan.frames.length < 2) {
      this.serverPlayback = null;
      return false;
    }
    this.serverPlayback = {
      seriesId: String(plan.seriesId ?? "y"),
      frames: plan.frames.map((row) => ({ x: Number(row.x), y: Number(row.y) })),
      axis: plan.axis ?? null,
      index: 0,
    };
    return true;
  }

  stepServerPlaybackFrame({ forceView = false } = {}) {
    const plan = this.serverPlayback;
    if (!plan || !Array.isArray(plan.frames) || plan.frames.length <= 1) return false;
    const size = plan.frames.length;
    const index = Math.max(0, Math.min(size - 1, Number(plan.index) || 0));
    const point = plan.frames[index];
    const extra = this.collectServerPlaybackExtraValues();
    const values = {
      x: point.x,
      [plan.seriesId]: point.y,
      ...extra,
    };
    const channels = [
      { key: "x", dtype: "number", role: "state" },
      { key: plan.seriesId, dtype: "number", role: "state" },
    ];
    if (Object.prototype.hasOwnProperty.call(extra, "L")) {
      channels.push({ key: "L", dtype: "number", role: "param" });
    }
    const row = channels.map((channel) => values[channel.key]);
    const seriesPoints = plan.frames.slice(0, index + 1).map((row0) => ({ x: row0.x, y: row0.y }));
    const graph = {
      axis: plan.axis ?? null,
      series: [{ id: plan.seriesId, points: seriesPoints }],
    };
    const observation = {
      channels,
      row,
      values,
      all_values: values,
    };
    this.applyRuntimeDerived(
      {
        observation,
        views: {
          graph,
          space2d: null,
          text: null,
          table: null,
          structure: null,
        },
      },
      { forceView },
    );
    plan.index = (index + 1) % size;
    this.runtimeTickCounter += 1;
    this.runtimeTimeValue = Number.isFinite(point?.x) ? point.x : this.runtimeTimeValue;
    this.setHash(`server-fallback:${index}`);
    this.syncDockTimeUi();
    this.renderInspectorMeta();
    return true;
  }

  getEffectiveDdn() {
    return applyControlValuesToDdnText(this.baseDdn, this.sliderPanel.getValues());
  }

  getEffectiveWasmSource(rawText = null) {
    const raw = rawText === null || rawText === undefined ? this.getEffectiveDdn() : String(rawText);
    try {
      const pre = preprocessDdnText(raw);
      const body = String(pre?.bodyText ?? "");
      return body.trim() ? body : raw;
    } catch (_) {
      return raw;
    }
  }

  getStepInput() {
    const fps = Math.max(1, Number(this.wasmState?.fpsLimit ?? 30) || 30);
    const dtMaxRaw = Number(this.wasmState?.dtMax ?? 0.1);
    const dtMax = Number.isFinite(dtMaxRaw) && dtMaxRaw > 0 ? dtMaxRaw : 0.1;
    const baseDt = 1 / fps;
    let dt = Math.min(baseDt, dtMax);
    if (!Number.isFinite(dt) || dt <= 0) {
      dt = baseDt;
    }
    if (!Number.isFinite(dt) || dt <= 0) {
      dt = 1 / 30;
    }
    const inputEnabled = Boolean(this.wasmState?.inputEnabled ?? true);
    const keys = inputEnabled
      ? ((Number(this.heldInputMask) | Number(this.pulsePressedMask)) & RUNTIME_INPUT_MASK_LIMIT)
      : 0;
    const lastKey = inputEnabled ? String(this.lastInputToken ?? "") : "";
    this.pulsePressedMask = 0;
    this.lastInputToken = "";
    return { dt, keys, lastKey, px: 0, py: 0 };
  }

  resolveSteppedState(client, steppedState) {
    let nextState = steppedState;
    try {
      const steppedViews = extractStructuredViewsFromState(nextState);
      if (!hasSpace2dDrawable(steppedViews?.space2d) && typeof client.getStateParsed === "function") {
        const fullState = client.getStateParsed();
        const fullViews = extractStructuredViewsFromState(fullState);
        if (hasSpace2dDrawable(fullViews?.space2d)) {
          nextState = fullState;
        }
      }
    } catch (_) {
      // keep stepped state when enrichment fails
    }
    return nextState;
  }

  stepClientOne(client) {
    if (!client || typeof client !== "object") {
      return { state: null };
    }
    return stepWasmClientParsed({
      client,
      input: this.getStepInput(),
      errorPrefix: "RunScreen.stepClientOne",
    });
  }

  async restart() {
    this.clearRuntimeInputState();
    this.runtimeTickCounter = 0;
    this.runtimeTimeValue = null;
    this.dockCursorTick = 0;
    this.dockCursorFollowLive = true;
    const rawDdnText = this.getEffectiveDdn();
    const wasmDdnText = this.getEffectiveWasmSource(rawDdnText);
    this.currentDdn = rawDdnText;
    if (this.runDdnPreviewEl) {
      this.runDdnPreviewEl.value = rawDdnText;
    }
    this.dotbogi.clearTimeline({ preserveAxes: true, preserveView: true });
    this.beginLiveRunCapture(rawDdnText);

    try {
      const ensureWasm = (source) => this.wasmState.loader.ensure(source);
      const tryRunWithMode = async (mode) =>
        applyWasmLogicAndDispatchState({
          sourceText: wasmDdnText,
          ensureWasm,
          mode,
        });

      let result;
      const preferredMode = String(this.wasmState?.langMode ?? "strict");
      try {
        result = await tryRunWithMode(preferredMode);
      } catch (err) {
        if (preferredMode === "compat") throw err;
        result = await tryRunWithMode("compat");
        this.wasmState.langMode = "compat";
      }

      this.wasmState.client = result.client;
      this.setParseWarnings(result.parseWarnings ?? readWasmClientParseWarnings(result.client));
      this.serverPlayback = null;
      let initialState = result.state;
      // 재실행은 로직 갱신뿐 아니라 시뮬 상태를 초기 프레임으로 되돌려야 한다.
      if (typeof result.client?.resetParsed === "function") {
        try {
          result.client.resetParsed(true);
          if (typeof result.client?.getStateParsed === "function") {
            initialState = result.client.getStateParsed();
          }
        } catch (_) {
          // reset 실패 시 로직 갱신 상태를 그대로 사용
        }
      }
      this.lastState = initialState;
      const hash = typeof result.client?.getStateHash === "function" ? result.client.getStateHash() : "-";
      this.setHash(hash);
      this.lastExecPathHint = `실행 경로: wasm(${String(this.wasmState?.langMode ?? "strict")})`;
      this.applyRuntimeState(initialState, { forceView: true });
      this.syncDockRangeLabels();
      this.syncDockTimeUi();
      this.renderInspectorMeta();
      if (this.lastSpace2dMode === "none" && result.client) {
        for (let i = 0; i < 3; i += 1) {
          const stepped = this.stepClientOne(result.client);
          const nextState = this.resolveSteppedState(result.client, stepped.state);
          this.lastState = nextState;
          const stepHash = typeof result.client?.getStateHash === "function" ? result.client.getStateHash() : "-";
          this.setHash(stepHash);
          this.applyRuntimeState(nextState, { forceView: true });
          this.runtimeTickCounter += 1;
          this.syncDockTimeUi();
          if (this.lastSpace2dMode !== "none") break;
        }
      }
      this.updateRuntimeHint();
      this.publishRunManagerSession();
      this.syncLoopState();
      return true;
    } catch (err) {
      console.error("[RunScreen.restart] wasm execution failed", err);
      if (this.allowServerFallback) {
        // server fallback은 wasm 내부 변환본이 아닌 원본 DDN(슬라이더 반영본)으로 실행한다.
        const serverDerived = await this.runViaExecServer(rawDdnText);
        if (serverDerived) {
          this.wasmState.client = null;
          this.setParseWarnings([]);
          this.lastState = null;
          this.lastRuntimeDerived = serverDerived;
          this.setHash("server-fallback");
          this.lastExecPathHint = "실행 경로: server-fallback";
          this.startServerPlayback(serverDerived);
          this.applyRuntimeDerived(serverDerived, { forceView: true });
          this.syncDockRangeLabels();
          this.syncDockTimeUi();
          this.renderInspectorMeta();
          this.updateRuntimeStatus(serverDerived);
          this.updateRuntimeHint();
          this.publishRunManagerSession();
          this.syncLoopState();
          return true;
        }
      }
      this.haltLoop();
      this.setParseWarnings([]);
      this.setHash("-");
      this.lastRuntimeDerived = null;
      this.lastExecPathHint = this.allowServerFallback
        ? "실행 실패: wasm/server 모두 실패"
        : "실행 실패: wasm-direct-only (E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED)";
      this.lastSpace2dMode = "none";
      this.setRuntimePreviewViewModel(null);
      this.updateRuntimeHint();
      this.syncDockRangeLabels();
      this.syncDockTimeUi();
      this.renderInspectorMeta();
      this.saveRuntimeSnapshot({ kind: "error", channels: 0 });
      this.discardActiveRunCaptureIfEmpty();
      return false;
    }
  }

  startLoop() {
    this.syncLoopState();
  }

  stopLoop() {
    this.haltLoop();
  }

  syncLoopState() {
    if (!this.loop) return;
    if (this.screenVisible) {
      this.loop.start();
      if (!this.playbackPaused) {
        this.startViewPlaybackTimer();
      } else {
        this.clearViewPlaybackTimer();
      }
      return;
    }
    this.clearViewPlaybackTimer();
    this.loop.stop();
  }

  haltLoop() {
    this.clearViewPlaybackTimer();
    if (!this.loop) return;
    this.loop.stop();
  }

  stepFrame() {
    if (!this.screenVisible) return;
    const client = this.wasmState?.client;
    if (!client) {
      this.stepServerPlaybackFrame();
      return;
    }

    try {
      const stepped = this.stepClientOne(client);
      const nextState = this.resolveSteppedState(client, stepped.state);
      this.lastState = nextState;
      const hash = typeof client.getStateHash === "function" ? client.getStateHash() : "-";
      this.setHash(hash);
      this.applyRuntimeState(nextState);
      this.runtimeTickCounter += 1;
      this.syncDockTimeUi();
      this.renderInspectorMeta();
    } catch (err) {
      console.error("[RunScreen.stepFrame] wasm step failed", err);
      this.haltLoop();
    }
  }

  applyRuntimeState(stateJson, { forceView = false } = {}) {
    const derived = extractRuntimeDerived(stateJson);
    if (!derived) return;
    const prevDerived = this.lastRuntimeDerived;
    const applied = this.applyRuntimeDerived(derived, { forceView, prevDerived });
    this.lastRuntimeDerived = applied ?? derived;
  }

  applyRuntimeDerived(derived, { forceView = false, prevDerived = null } = {}) {
    if (!derived || typeof derived !== "object") return;
    const observation = derived.observation ?? null;
    let views = derived.views ?? null;
    const prevSpace2d = hasSpace2dDrawable(prevDerived?.views?.space2d)
      ? prevDerived.views.space2d
      : null;
    const nativeSpace2d = views?.space2d ?? null;
    const hasNativeSpace2d = hasSpace2dDrawable(nativeSpace2d);
    const allowShapeFallback = Boolean(this.allowShapeFallback);
    // strict 모드에서도 진자(theta) 케이스는 최소 보개를 보장한다.
    const pendulumFallback = synthesizePendulumSpace2dFromObservation(observation) ??
      synthesizePendulumSpace2dFromGraph(views?.graph, observation);
    const fallbackSpace2d = hasNativeSpace2d
      ? null
      : allowShapeFallback
        ? synthesizeSpace2dFromObservation(observation) ??
          synthesizeSpace2dFromGraph(views?.graph, observation)
        : pendulumFallback;
    const space2d = hasNativeSpace2d ? nativeSpace2d : (fallbackSpace2d ?? prevSpace2d);

    this.lastSpace2dMode = hasNativeSpace2d ? "native" : space2d ? "fallback" : "none";
    this.updateRuntimeHint("", { observation, views });

    if (space2d && (!views || views.space2d !== space2d)) {
      views = {
        ...(views && typeof views === "object" ? views : {}),
        space2d,
      };
    }
    const runtimeOverlayMarkdown = readRuntimeTextMarkdownFromViews(views);
    if (runtimeOverlayMarkdown && runtimeOverlayMarkdown !== this.lastOverlayMarkdown) {
      this.lastOverlayMarkdown = runtimeOverlayMarkdown;
      this.overlay.setContent(runtimeOverlayMarkdown);
      this.renderRuntimeTextContent({
        markdown: runtimeOverlayMarkdown,
        structure: views?.structure ?? null,
        graph: views?.graph ?? null,
      });
    } else if (views?.structure) {
      this.renderRuntimeTextContent({
        markdown: this.lastRuntimeTextMarkdown,
        structure: views.structure,
        graph: views?.graph ?? null,
      });
    } else if (views?.graph) {
      this.renderRuntimeTextContent({
        markdown: this.lastRuntimeTextMarkdown || summarizeRuntimeGraphMarkdown(views.graph),
        structure: null,
        graph: views.graph,
      });
    }
    this.updateLiveRunCaptureFromDerived({ observation, views });
    const shouldUpdateView = forceView || this.screenVisible;

    if (shouldUpdateView) {
      this.dotbogi.appendObservation(observation);
      this.bogae.render(space2d ?? null);
      this.renderCurrentRuntimeTable(views?.table ?? null);
    }
    this.syncDockRangeLabels();
    this.syncDockPanelVisibility({ observation, views });
    this.updateRuntimeStatus({ observation, views });
    return { observation, views };
  }

  setScreenVisible(visible) {
    const next = Boolean(visible);
    if (this.screenVisible === next) return;
    this.screenVisible = next;
    if (!next) {
      this.clearRuntimeInputState();
      this.clearViewPlaybackTimer();
      this.flushOverlaySession();
    }
    this.syncLoopState();
    this.syncDockTimeUi();
    if (!next) return;
    if (this.lastRuntimeDerived) {
      this.applyRuntimeDerived(this.lastRuntimeDerived, { forceView: true });
    } else if (this.lastState) {
      this.applyRuntimeState(this.lastState, { forceView: true });
    }
  }

  updateRuntimeStatus({ observation = null, views = null } = {}) {
    const hasSpace2d = hasSpace2dDrawable(views?.space2d);
    const { kind, channels } = deriveRunKindAndChannels({ observation, hasSpace2d });
    this.saveRuntimeSnapshot({ kind, channels });
    this.renderInspectorMeta();
  }

  saveRuntimeSnapshot({ kind = "empty", channels = 0 } = {}) {
    const lessonId = String(this.lesson?.id ?? "").trim();
    if (!lessonId) return;
    const normalizedKind = String(kind ?? "empty").trim() || "empty";
    const normalizedChannels = Math.max(0, Number.isFinite(Number(channels)) ? Math.trunc(Number(channels)) : 0);
    const normalizedLaunchKind = normalizeRunLaunchKind(this.lastLaunchKind);
    const snapshotKey = `${lessonId}:${normalizedKind}:${normalizedChannels}:${normalizedLaunchKind}`;
    if (snapshotKey === this.lastRuntimeSnapshotKey) return;
    this.lastRuntimeSnapshotKey = snapshotKey;

    const pref = this.getLessonUiPref(lessonId, { create: true });
    pref.lastRunKind = normalizedKind;
    pref.lastRunChannels = normalizedChannels;
    pref.lastRunAt = new Date().toISOString();
    pref.lastRunHash = String(this.lastRuntimeHash ?? "-");
    pref.lastLaunchKind = normalizedLaunchKind;
    this.persistUiPrefs();
    this.updateRunSummaryFromPrefs();
  }
}
