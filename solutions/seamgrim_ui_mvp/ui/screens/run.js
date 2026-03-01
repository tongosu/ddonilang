import {
  applyWasmLogicAndDispatchState,
  createManagedRafStepLoop,
  stepWasmClientParsed,
} from "../wasm_page_common.js";
import { extractObservationChannelsFromState, extractStructuredViewsFromState } from "../seamgrim_runtime_state.js";
import { applyControlValuesToDdnText } from "../components/control_parser.js";
import { Bogae } from "../components/bogae.js";
import { DotbogiPanel } from "../components/dotbogi.js";
import { SliderPanel } from "../components/slider_panel.js";
import { OverlayDescription } from "../components/overlay.js";
import { preprocessDdnText } from "../runtime/ddn_preprocess.js";

const RUN_UI_PREFS_STORAGE_KEY = "seamgrim.ui.run_prefs.v1";

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
      { kind: "line", x1: 0, y1: 0, x2: bx, y2: by, stroke: "#9ca3af", width: 0.02 },
      { kind: "circle", x: bx, y: by, r: 0.08, fill: "#38bdf8", stroke: "#0ea5e9", width: 0.02 },
      { kind: "point", x: 0, y: 0, size: 0.045, color: "#f59e0b" },
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
      { kind: "line", x1: xMin, y1: 0, x2: xMax, y2: 0, stroke: "#4b5563", width: 0.01 },
      { kind: "line", x1: 0, y1: yMin, x2: 0, y2: yMax, stroke: "#374151", width: 0.01 },
      { kind: "circle", x: px, y: py, r: 0.07, fill: "#22c55e", stroke: "#16a34a", width: 0.02 },
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
        { kind: "line", x1: 0, y1: 0, x2: bx, y2: by, stroke: "#9ca3af", width: 0.02 },
        { kind: "circle", x: bx, y: by, r: 0.08, fill: "#38bdf8", stroke: "#0ea5e9", width: 0.02 },
        { kind: "point", x: 0, y: 0, size: 0.045, color: "#f59e0b" },
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
      { kind: "line", x1: xMin, y1: 0, x2: xMax, y2: 0, stroke: "#4b5563", width: 0.01 },
      { kind: "line", x1: 0, y1: yMin, x2: 0, y2: yMax, stroke: "#374151", width: 0.01 },
      { kind: "circle", x: point.x, y: point.y, r: 0.07, fill: "#22c55e", stroke: "#16a34a", width: 0.02 },
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

export class RunScreen {
  constructor({ root, wasmState, onBack, onEditDdn, onOpenAdvanced, allowShapeFallback = false } = {}) {
    this.root = root;
    this.wasmState = wasmState;
    this.onBack = typeof onBack === "function" ? onBack : () => {};
    this.onEditDdn = typeof onEditDdn === "function" ? onEditDdn : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};
    this.allowShapeFallback = Boolean(allowShapeFallback);

    this.lesson = null;
    this.baseDdn = "";
    this.currentDdn = "";
    this.lastState = null;
    this.lastRuntimeDerived = null;

    this.loopActive = false;
    this.screenVisible = false;
    this.loop = null;
    this.viewPanStep = 0.08;
    this.lastRuntimeHash = "-";
    this.lastRuntimeSnapshotKey = "";
    this.lastExecPathHint = "";
    this.lastSpace2dMode = "none";
    this.lastRuntimeHintText = "";
    this.lastOverlayMarkdown = "";
    this.serverPlayback = null;
    this.boundKeyHandler = (event) => {
      this.handleViewHotkeys(event);
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
    this.runtimeStatusEl = this.root.querySelector("#slider-status");

    this.uiPrefs = {
      lessons: {},
      ...readStorageJson(RUN_UI_PREFS_STORAGE_KEY, {}),
    };
    if (!this.uiPrefs.lessons || typeof this.uiPrefs.lessons !== "object") {
      this.uiPrefs.lessons = {};
    }

    this.bogae = new Bogae({
      canvas: this.root.querySelector("#canvas-bogae"),
    });
    this.dotbogi = new DotbogiPanel({
      graphCanvas: this.root.querySelector("#canvas-graph"),
      xAxisSelect: this.root.querySelector("#select-x-axis"),
      yAxisSelect: this.root.querySelector("#select-y-axis"),
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
    });

    this.root.querySelector("#btn-advanced-run")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });

    this.root.querySelector("#btn-restart")?.addEventListener("click", () => {
      void this.restart();
    });
    this.root.querySelector("#select-x-axis")?.addEventListener("change", () => {
      this.saveCurrentLessonUiPrefs();
    });
    this.root.querySelector("#select-y-axis")?.addEventListener("change", () => {
      this.saveCurrentLessonUiPrefs();
    });

    this.loop = createManagedRafStepLoop({
      getFps: () => Math.max(1, Number(this.wasmState?.fpsLimit ?? 30) || 30),
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
    window.addEventListener("keydown", this.boundKeyHandler);
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

  setHash(hashText) {
    this.lastRuntimeHash = String(hashText ?? "-");
  }

  updateRunSummaryFromPrefs() {
    if (!this.lastSummaryEl) return;
    const lessonId = String(this.lesson?.id ?? "").trim();
    const pref = lessonId ? this.getLessonUiPref(lessonId, { create: false }) : null;
    this.lastSummaryEl.textContent = buildRunSummaryText(pref);
  }

  loadLesson(lesson) {
    this.lesson = lesson;
    this.baseDdn = String(lesson?.ddnText ?? "");
    this.currentDdn = this.baseDdn;
    this.lastRuntimeHash = "-";
    this.lastRuntimeSnapshotKey = "";
    this.lastRuntimeDerived = null;
    this.lastExecPathHint = "";
    this.lastSpace2dMode = "none";
    this.lastRuntimeHintText = "";
    this.lastOverlayMarkdown = "";
    this.serverPlayback = null;

    if (this.titleEl) {
      this.titleEl.textContent = lesson?.title || lesson?.id || "-";
    }

    this.lastOverlayMarkdown = String(lesson?.textMd ?? "");
    this.overlay.setContent(this.lastOverlayMarkdown);
    if (this.isSimCorePolicyEnabled()) {
      this.overlay.show();
    } else {
      this.overlay.hide();
    }
    const parsed = this.sliderPanel.parseFromDdn(this.baseDdn, { preserveValues: false });
    this.dotbogi.setSeedKeys(parsed.axisKeys);
    this.dotbogi.setPreferredXKey(parsed.defaultXAxisKey);
    this.dotbogi.setPreferredYKey(parsed.defaultAxisKey);
    this.dotbogi.clearTimeline();
    this.restoreLessonUiPrefs(lesson?.id);
    this.bogae.resetView();
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

    const obs = runtimeDerived?.observation ?? this.lastRuntimeDerived?.observation ?? null;
    const t = readNumericObservationValue(obs, ["t", "time", "tick", "프레임수", "시간"]);
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
    if (nextText === this.lastRuntimeHintText) return;
    this.lastRuntimeHintText = nextText;
    this.runtimeStatusEl.textContent = nextText;
  }

  async runViaExecServer(ddnText) {
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
    this.setHash(`server-fallback:${index}`);
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
    return { dt, keys: 0, lastKey: "", px: 0, py: 0 };
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
    // 실행 화면은 키/포인터 입력을 쓰지 않으므로, 가능하면 입력 없는 step API를 우선한다.
    if (typeof client.stepOneParsed === "function") {
      return { state: client.stepOneParsed(), input: null, mode: "step_one" };
    }
    return stepWasmClientParsed({
      client,
      input: this.getStepInput(),
    });
  }

  async restart() {
    const rawDdnText = this.getEffectiveDdn();
    const wasmDdnText = this.getEffectiveWasmSource(rawDdnText);
    this.currentDdn = rawDdnText;
    this.dotbogi.clearTimeline({ preserveAxes: true, preserveView: true });

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
      if (this.lastSpace2dMode === "none" && result.client) {
        for (let i = 0; i < 3; i += 1) {
          const stepped = this.stepClientOne(result.client);
          const nextState = this.resolveSteppedState(result.client, stepped.state);
          this.lastState = nextState;
          const stepHash = typeof result.client?.getStateHash === "function" ? result.client.getStateHash() : "-";
          this.setHash(stepHash);
          this.applyRuntimeState(nextState, { forceView: true });
          if (this.lastSpace2dMode !== "none") break;
        }
      }
      this.updateRuntimeHint();
      this.syncLoopState();
      return true;
    } catch (err) {
      console.error("[RunScreen.restart] wasm execution failed", err);
      // server fallback은 wasm 내부 변환본이 아닌 원본 DDN(슬라이더 반영본)으로 실행한다.
      const serverDerived = await this.runViaExecServer(rawDdnText);
      if (serverDerived) {
        this.wasmState.client = null;
        this.lastState = null;
        this.lastRuntimeDerived = serverDerived;
        this.setHash("server-fallback");
        this.lastExecPathHint = "실행 경로: server-fallback";
        this.startServerPlayback(serverDerived);
        this.applyRuntimeDerived(serverDerived, { forceView: true });
        this.updateRuntimeStatus(serverDerived);
        this.updateRuntimeHint();
        this.syncLoopState();
        return true;
      }
      this.haltLoop();
      this.setHash("-");
      this.lastRuntimeDerived = null;
      this.lastExecPathHint = "실행 실패: wasm/server 모두 실패";
      this.lastSpace2dMode = "none";
      this.updateRuntimeHint();
      this.saveRuntimeSnapshot({ kind: "error", channels: 0 });
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
      return;
    }
    this.loop.stop();
  }

  haltLoop() {
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
    const runtimeOverlayMarkdown = readOverlayMarkdownFromViewText(views?.text);
    if (runtimeOverlayMarkdown && runtimeOverlayMarkdown !== this.lastOverlayMarkdown) {
      this.lastOverlayMarkdown = runtimeOverlayMarkdown;
      this.overlay.setContent(runtimeOverlayMarkdown);
    }
    const shouldUpdateView = forceView || this.screenVisible;

    if (shouldUpdateView) {
      this.dotbogi.appendObservation(observation);
      this.bogae.render(space2d ?? null);
    }
    this.updateRuntimeStatus({ observation, views });
    return { observation, views };
  }

  setScreenVisible(visible) {
    const next = Boolean(visible);
    if (this.screenVisible === next) return;
    this.screenVisible = next;
    this.syncLoopState();
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
  }

  saveRuntimeSnapshot({ kind = "empty", channels = 0 } = {}) {
    const lessonId = String(this.lesson?.id ?? "").trim();
    if (!lessonId) return;
    const normalizedKind = String(kind ?? "empty").trim() || "empty";
    const normalizedChannels = Math.max(0, Number.isFinite(Number(channels)) ? Math.trunc(Number(channels)) : 0);
    const snapshotKey = `${lessonId}:${normalizedKind}:${normalizedChannels}`;
    if (snapshotKey === this.lastRuntimeSnapshotKey) return;
    this.lastRuntimeSnapshotKey = snapshotKey;

    const pref = this.getLessonUiPref(lessonId, { create: true });
    pref.lastRunKind = normalizedKind;
    pref.lastRunChannels = normalizedChannels;
    pref.lastRunAt = new Date().toISOString();
    pref.lastRunHash = String(this.lastRuntimeHash ?? "-");
    this.persistUiPrefs();
    this.updateRunSummaryFromPrefs();
  }
}
