import { renderGraphCanvas2d } from "../wasm_page_common.js";

function finite(value) {
  const n = Number(value);
  return Number.isFinite(n) ? n : null;
}

function normalizeAxis(axis) {
  if (!axis || typeof axis !== "object") return null;
  const xMin = finite(axis.x_min ?? axis.xMin);
  const xMax = finite(axis.x_max ?? axis.xMax);
  const yMin = finite(axis.y_min ?? axis.yMin);
  const yMax = finite(axis.y_max ?? axis.yMax);
  if ([xMin, xMax, yMin, yMax].some((value) => value === null)) return null;
  if (xMax <= xMin || yMax <= yMin) return null;
  return {
    x_min: xMin,
    x_max: xMax,
    y_min: yMin,
    y_max: yMax,
  };
}

function cloneAxis(axis) {
  const normalized = normalizeAxis(axis);
  return normalized ? { ...normalized } : null;
}

function colorWithAlpha(color, alpha = 1) {
  const text = String(color ?? "").trim();
  const numeric = Number(alpha);
  const clamped = Number.isFinite(numeric) ? Math.max(0, Math.min(1, numeric)) : 1;
  if (!text || clamped >= 0.999) return text || "#22d3ee";
  const hex = text.replace(/^#/, "");
  if (hex.length === 6) {
    const r = Number.parseInt(hex.slice(0, 2), 16);
    const g = Number.parseInt(hex.slice(2, 4), 16);
    const b = Number.parseInt(hex.slice(4, 6), 16);
    if ([r, g, b].every(Number.isFinite)) {
      return `rgba(${r}, ${g}, ${b}, ${clamped})`;
    }
  }
  return text;
}

const INDEX_X_KEY = "__index__";
const GRAPH_KIND_LINE = "line";
const GRAPH_KIND_POINT = "point";
const GRAPH_KIND_STEP = "step";
const GRAPH_KIND_OPTIONS = Object.freeze([GRAPH_KIND_LINE, GRAPH_KIND_POINT, GRAPH_KIND_STEP]);
const STATIC_TIME_KEYS = Object.freeze(["t", "time", "시간", "tick", "프레임수"]);
const GRAPH_RANGE_RECENT_500 = "500";
const GRAPH_RANGE_RECENT_2000 = "2000";
const GRAPH_RANGE_ALL = "all";
const GRAPH_RANGE_OPTIONS = Object.freeze([GRAPH_RANGE_RECENT_500, GRAPH_RANGE_RECENT_2000, GRAPH_RANGE_ALL]);
const INTERNAL_AXIS_HIDDEN_KEYS = new Set([
  "__wasm_start_once",
  "__wasm_legacy_on_start_done__",
]);
const INTERNAL_AXIS_HIDDEN_PREFIXES = ["__wasm_"];

function normalizeKey(raw) {
  return String(raw ?? "").trim().toLowerCase();
}

function isHiddenInternalAxisKey(rawKey) {
  const key = String(rawKey ?? "").trim();
  if (!key) return true;
  if (INTERNAL_AXIS_HIDDEN_KEYS.has(key)) return true;
  return INTERNAL_AXIS_HIDDEN_PREFIXES.some((prefix) => key.startsWith(prefix));
}

function isSelectableAxisKey(rawKey) {
  const key = String(rawKey ?? "").trim();
  if (!key) return false;
  if (key === INDEX_X_KEY) return false;
  return !isHiddenInternalAxisKey(key);
}

function normalizeObservationChannelKey(channel, index) {
  if (typeof channel === "string") return channel.trim();
  if (!channel || typeof channel !== "object") return "";
  const direct = String(channel.key ?? "").trim();
  if (direct) return direct;
  return String(channel.name ?? channel.id ?? channel.label ?? channel.token ?? "").trim();
}

function pickXAxisByTimeHeuristic(sample, index) {
  if (!sample || typeof sample !== "object") return index;
  const candidates = ["t", "time", "시간", "tick", "프레임수"];
  for (const key of candidates) {
    const found = Object.keys(sample).find((name) => normalizeKey(name) === normalizeKey(key));
    if (!found) continue;
    const value = finite(sample[found]);
    if (value !== null) return value;
  }
  return index;
}

function pickDefaultTimeKey(keys = []) {
  const candidates = STATIC_TIME_KEYS;
  const normalizedMap = new Map(
    keys.map((key) => [normalizeKey(key), String(key ?? "").trim()]).filter(([, key]) => Boolean(key)),
  );
  for (const candidate of candidates) {
    const found = normalizedMap.get(normalizeKey(candidate));
    if (found) return found;
  }
  return "";
}

function normalizeGraphKind(rawKind, fallback = GRAPH_KIND_LINE) {
  const kind = String(rawKind ?? "").trim().toLowerCase();
  return GRAPH_KIND_OPTIONS.includes(kind) ? kind : fallback;
}

function buildDraftObservableKeys(seedKeys = []) {
  const rows = Array.isArray(seedKeys) ? seedKeys : [];
  return Array.from(new Set([...STATIC_TIME_KEYS, ...rows].filter(isSelectableAxisKey)));
}

function normalizeGraphRange(rawRange, fallback = GRAPH_RANGE_RECENT_500) {
  const range = String(rawRange ?? "").trim().toLowerCase();
  return GRAPH_RANGE_OPTIONS.includes(range) ? range : fallback;
}

function resolveGraphMaxPoints(range = GRAPH_RANGE_RECENT_500) {
  const normalized = normalizeGraphRange(range);
  if (normalized === GRAPH_RANGE_ALL) return Number.POSITIVE_INFINITY;
  return normalized === GRAPH_RANGE_RECENT_2000 ? 2000 : 500;
}

export class DotbogiPanel {
  constructor({
    graphCanvas,
    xAxisSelect,
    yAxisSelect,
    onAxisChange = null,
  } = {}) {
    this.graphCanvas = graphCanvas;
    this.xAxisSelect = xAxisSelect;
    this.yAxisSelect = yAxisSelect;
    this.onAxisChange = typeof onAxisChange === "function" ? onAxisChange : null;
    this.seedKeys = [];
    this.observableKeys = [];
    this.timeline = [];
    this.maxPointsMode = GRAPH_RANGE_RECENT_500;
    this.maxPoints = resolveGraphMaxPoints(this.maxPointsMode);
    this.selectedXKey = INDEX_X_KEY;
    this.selectedYKey = "";
    this.preferredXKey = "";
    this.preferredYKey = "";
    this.lockInitialAxis = true;
    this.initialAxisLocked = false;
    this.overlaySeries = [];
    this.persistedGraph = null;
    this.preferPersistedGraph = false;
    this.baseSeriesVisible = true;
    this.baseSeriesAlpha = 1;
    this.baseSeriesColor = "#22d3ee";
    this.playbackCursorIndex = null;
    this.graphView = {
      autoFit: true,
      axis: null,
      showGrid: true,
      showAxis: true,
      kind: GRAPH_KIND_LINE,
    };
    this.drag = {
      active: false,
      x: 0,
      y: 0,
    };

    this.bind();
  }

  emitAxisChange(axis, meta = {}) {
    if (!this.onAxisChange) return;
    this.onAxisChange(cloneAxis(axis), meta);
  }

  bind() {
    this.yAxisSelect?.addEventListener("change", () => {
      this.selectedYKey = String(this.yAxisSelect.value ?? "");
      this.graphView.autoFit = true;
      this.graphView.axis = null;
      this.initialAxisLocked = false;
      this.renderGraph();
    });
    this.xAxisSelect?.addEventListener("change", () => {
      this.selectedXKey = String(this.xAxisSelect.value ?? INDEX_X_KEY);
      this.graphView.autoFit = true;
      this.graphView.axis = null;
      this.initialAxisLocked = false;
      this.renderGraph();
    });

    this.graphCanvas?.addEventListener("wheel", (event) => {
      event.preventDefault();
      this.ensureManualAxis();
      if (!this.graphView.axis) return;
      const factor = event.deltaY < 0 ? 0.9 : 1.1;
      const axis = this.graphView.axis;
      const cx = (axis.x_min + axis.x_max) / 2;
      const cy = (axis.y_min + axis.y_max) / 2;
      const halfW = ((axis.x_max - axis.x_min) / 2) * factor;
      const halfH = ((axis.y_max - axis.y_min) / 2) * factor;
      this.graphView.axis = {
        x_min: cx - halfW,
        x_max: cx + halfW,
        y_min: cy - halfH,
        y_max: cy + halfH,
      };
      this.graphView.autoFit = false;
      this.initialAxisLocked = true;
      this.renderGraph();
    });

    this.graphCanvas?.addEventListener("mousedown", (event) => {
      this.ensureManualAxis();
      if (!this.graphView.axis) return;
      this.drag.active = true;
      this.drag.x = event.clientX;
      this.drag.y = event.clientY;
    });

    window.addEventListener("mouseup", () => {
      this.drag.active = false;
    });

    window.addEventListener("mousemove", (event) => {
      if (!this.drag.active) return;
      const axis = this.graphView.axis;
      if (!axis) return;
      const widthPx = this.graphCanvas?.clientWidth || 1;
      const heightPx = this.graphCanvas?.clientHeight || 1;
      const dxPx = event.clientX - this.drag.x;
      const dyPx = event.clientY - this.drag.y;
      this.drag.x = event.clientX;
      this.drag.y = event.clientY;
      const dxWorld = (dxPx / widthPx) * (axis.x_max - axis.x_min);
      const dyWorld = (dyPx / heightPx) * (axis.y_max - axis.y_min);
      this.graphView.axis = {
        x_min: axis.x_min - dxWorld,
        x_max: axis.x_max - dxWorld,
        y_min: axis.y_min + dyWorld,
        y_max: axis.y_max + dyWorld,
      };
      this.graphView.autoFit = false;
      this.initialAxisLocked = true;
      this.renderGraph();
    });
  }

  ensureManualAxis() {
    if (!this.graphView.autoFit && this.graphView.axis) return;
    const auto = this.computeAutoAxis();
    if (!auto) return;
    this.graphView.axis = auto;
    this.graphView.autoFit = false;
    this.initialAxisLocked = true;
  }

  getCurrentAxis() {
    if (this.graphView.autoFit) {
      return cloneAxis(this.computeAutoAxis());
    }
    return cloneAxis(this.graphView.axis ?? this.computeAutoAxis());
  }

  setAxis(axis) {
    const normalized = normalizeAxis(axis);
    if (!normalized) return false;
    this.graphView.axis = normalized;
    this.graphView.autoFit = false;
    this.initialAxisLocked = true;
    this.renderGraph();
    return true;
  }

  getGuides() {
    return {
      showGrid: Boolean(this.graphView.showGrid),
      showAxis: Boolean(this.graphView.showAxis),
    };
  }

  setGuides({ showGrid = null, showAxis = null } = {}) {
    let changed = false;
    if (typeof showGrid === "boolean" && this.graphView.showGrid !== showGrid) {
      this.graphView.showGrid = showGrid;
      changed = true;
    }
    if (typeof showAxis === "boolean" && this.graphView.showAxis !== showAxis) {
      this.graphView.showAxis = showAxis;
      changed = true;
    }
    if (changed) {
      this.renderGraph();
    }
    return this.getGuides();
  }

  resetAxis() {
    this.graphView.autoFit = true;
    this.graphView.axis = null;
    this.initialAxisLocked = false;
    this.renderGraph();
  }

  setSeedKeys(keys = []) {
    this.seedKeys = Array.isArray(keys) ? keys.map((v) => String(v ?? "").trim()).filter(Boolean) : [];
    if (!this.timeline.length) {
      this.observableKeys = buildDraftObservableKeys(this.seedKeys);
      if (!this.selectedXKey) {
        this.selectedXKey = pickDefaultTimeKey(this.observableKeys) || INDEX_X_KEY;
      }
      if (!this.selectedYKey) {
        this.selectedYKey = this.observableKeys.find((key) => !STATIC_TIME_KEYS.includes(key)) ?? this.observableKeys[0] ?? "";
      }
      this.syncXAxisSelect();
      this.syncYAxisSelect();
      this.renderGraph();
    }
  }

  getSelectedAxes() {
    return {
      xKey: this.selectedXKey,
      yKey: this.selectedYKey,
    };
  }

  getGraphKind() {
    return normalizeGraphKind(this.graphView.kind, GRAPH_KIND_LINE);
  }

  setGraphKind(kind = GRAPH_KIND_LINE) {
    const next = normalizeGraphKind(kind, this.getGraphKind());
    if (next === this.graphView.kind) return next;
    this.graphView.kind = next;
    this.renderGraph();
    return next;
  }

  getMaxPointsMode() {
    return normalizeGraphRange(this.maxPointsMode, GRAPH_RANGE_RECENT_500);
  }

  getMaxPoints() {
    return this.maxPoints;
  }

  trimTimelineToMaxPoints() {
    if (!Number.isFinite(this.maxPoints)) return;
    if (this.timeline.length <= this.maxPoints) return;
    this.timeline = this.timeline.slice(this.timeline.length - this.maxPoints);
    this.timeline.forEach((item, idx) => {
      item.index = idx;
    });
  }

  setMaxPointsMode(range = GRAPH_RANGE_RECENT_500) {
    const nextMode = normalizeGraphRange(range, this.getMaxPointsMode());
    const nextMaxPoints = resolveGraphMaxPoints(nextMode);
    const changed = nextMode !== this.maxPointsMode || nextMaxPoints !== this.maxPoints;
    this.maxPointsMode = nextMode;
    this.maxPoints = nextMaxPoints;
    this.trimTimelineToMaxPoints();
    if (changed) {
      this.resetAxis();
    }
    return this.getMaxPointsMode();
  }

  setSelectedAxes({ xKey = "", yKey = "" } = {}) {
    const nextX = String(xKey ?? "").trim();
    const nextY = String(yKey ?? "").trim();
    if (nextX) {
      this.selectedXKey = nextX;
    }
    if (nextY) {
      this.selectedYKey = nextY;
    }
    this.syncXAxisSelect();
    this.syncYAxisSelect();
    this.resetAxis();
  }

  setPreferredXKey(key) {
    this.preferredXKey = String(key ?? "").trim();
    if (!this.preferredXKey) return;
    if (!this.observableKeys.includes(this.preferredXKey)) return;
    this.selectedXKey = this.preferredXKey;
    this.syncXAxisSelect();
    this.resetAxis();
  }

  setPreferredYKey(key) {
    this.preferredYKey = String(key ?? "").trim();
    if (!this.preferredYKey) return;
    if (!this.observableKeys.includes(this.preferredYKey)) return;
    this.selectedYKey = this.preferredYKey;
    this.syncYAxisSelect();
    this.resetAxis();
  }

  clearTimeline({ preserveAxes = false, preserveView = false } = {}) {
    this.timeline = [];
    this.playbackCursorIndex = null;
    if (!preserveAxes) {
      this.selectedXKey = INDEX_X_KEY;
      this.selectedYKey = "";
      this.observableKeys = buildDraftObservableKeys(this.seedKeys);
      this.syncXAxisSelect();
      this.syncYAxisSelect();
    }
    if (!preserveView) {
      this.graphView.autoFit = true;
      this.graphView.axis = null;
      this.initialAxisLocked = false;
    }
    this.renderGraph();
  }

  getTimelineLength() {
    return Array.isArray(this.timeline) ? this.timeline.length : 0;
  }

  getTimelineSampleAt(index = -1) {
    const size = this.getTimelineLength();
    if (size <= 0) return null;
    const raw = Number(index);
    const normalized = Number.isFinite(raw) ? Math.max(0, Math.min(size - 1, Math.trunc(raw))) : size - 1;
    return this.timeline[normalized] ?? null;
  }

  getPlaybackCursor() {
    const size = this.getTimelineLength();
    if (size <= 0) return 0;
    if (this.playbackCursorIndex === null || this.playbackCursorIndex === undefined) {
      return size - 1;
    }
    const raw = Number(this.playbackCursorIndex);
    if (!Number.isFinite(raw)) return size - 1;
    return Math.max(0, Math.min(size - 1, Math.trunc(raw)));
  }

  setPlaybackCursor(index = null, { render = true } = {}) {
    if (index === null || index === undefined || index === "") {
      this.playbackCursorIndex = null;
      if (render) this.renderGraph();
      return this.getPlaybackCursor();
    }
    const size = this.getTimelineLength();
    if (size <= 0) {
      this.playbackCursorIndex = 0;
      if (render) this.renderGraph();
      return 0;
    }
    const raw = Number(index);
    const normalized = Number.isFinite(raw) ? Math.max(0, Math.min(size - 1, Math.trunc(raw))) : size - 1;
    this.playbackCursorIndex = normalized;
    if (render) this.renderGraph();
    return normalized;
  }

  setOverlaySeries(seriesList = []) {
    const source = Array.isArray(seriesList) ? seriesList : [];
    this.overlaySeries = source
      .map((row, index) => {
        const item = row && typeof row === "object" ? row : {};
        const id = String(item.id ?? item.label ?? `overlay_${index + 1}`).trim() || `overlay_${index + 1}`;
        const color = String(item.color ?? "").trim();
        const pointsRaw = Array.isArray(item.points) ? item.points : [];
        const points = pointsRaw
          .map((point) => {
            const x = finite(point?.x);
            const y = finite(point?.y);
            if (x === null || y === null) return null;
            return { x, y };
          })
          .filter(Boolean);
        return {
          id,
          color,
          points,
        };
      })
      .filter((row) => row.points.length > 0);
    this.renderGraph();
  }

  setPersistedGraph(graph = null, { render = true } = {}) {
    const row = graph && typeof graph === "object" ? graph : null;
    if (!row) {
      this.persistedGraph = null;
      if (render) this.renderGraph();
      return;
    }
    const axis = normalizeAxis(row.axis ?? null);
    const seriesSource = Array.isArray(row.series) ? row.series : [];
    const series = seriesSource
      .map((item, index) => {
        const points = Array.isArray(item?.points)
          ? item.points
            .map((point) => {
              const x = finite(point?.x);
              const y = finite(point?.y);
              if (x === null || y === null) return null;
              return { x, y };
            })
            .filter(Boolean)
          : [];
        if (!points.length) return null;
        return {
          id: String(item?.id ?? item?.label ?? `persisted_${index + 1}`).trim() || `persisted_${index + 1}`,
          color: String(item?.color ?? "").trim(),
          points,
        };
      })
      .filter(Boolean);
    this.persistedGraph = series.length ? { axis, series } : null;
    if (render) this.renderGraph();
  }

  setBaseSeriesDisplay({ visible = true, alpha = 1, color = "#22d3ee", preferPersisted = null } = {}, { render = true } = {}) {
    this.baseSeriesVisible = visible !== false;
    const nextAlpha = Number(alpha);
    this.baseSeriesAlpha = Number.isFinite(nextAlpha) && nextAlpha >= 0 ? nextAlpha : 1;
    const nextColor = String(color ?? "").trim();
    if (nextColor) {
      this.baseSeriesColor = nextColor;
    }
    if (typeof preferPersisted === "boolean") {
      this.preferPersistedGraph = preferPersisted;
    }
    if (render) this.renderGraph();
  }

  exportCurrentGraphSnapshot() {
    const basePoints = this.buildSeriesPoints();
    if (basePoints.length && this.selectedYKey) {
      const axis = cloneAxis(this.getCurrentAxis());
      return {
        axis,
        series: [{
          id: String(this.selectedYKey ?? "y"),
          color: this.baseSeriesColor,
          points: basePoints.map((point) => ({ x: point.x, y: point.y })),
        }],
      };
    }
    const filtered = Array.isArray(this.persistedGraph?.series)
      ? this.persistedGraph.series
        .map((row, index) => {
          const points = Array.isArray(row?.points)
            ? row.points
              .map((point) => {
                const x = finite(point?.x);
                const y = finite(point?.y);
                if (x === null || y === null) return null;
                return { x, y };
              })
              .filter(Boolean)
            : [];
          if (!points.length) return null;
          return {
            id: String(row?.id ?? `series_${index + 1}`),
            color: String(row?.color ?? "").trim(),
            points,
          };
        })
        .filter(Boolean)
      : [];
    if (!filtered.length) return null;
    return {
      axis: this.getCurrentAxis(),
      series: filtered,
    };
  }

  appendObservation(observation) {
    const valuesSource =
      observation && typeof observation.all_values === "object"
        ? observation.all_values
        : observation && typeof observation.values === "object"
          ? observation.values
          : {};
    const values = valuesSource && typeof valuesSource === "object" ? valuesSource : {};
    const nextIndex = this.timeline.length;
    this.timeline.push({ index: nextIndex, values: { ...values } });
    this.trimTimelineToMaxPoints();

    const observationKeys = Array.isArray(observation?.channels)
      ? observation.channels.map((item, index) => normalizeObservationChannelKey(item, index)).filter(isSelectableAxisKey)
      : Object.keys(values).map((key) => String(key ?? "").trim()).filter(isSelectableAxisKey);
    const valueKeys = Object.keys(values).map((key) => String(key ?? "").trim()).filter(isSelectableAxisKey);
    const seedKeys = this.seedKeys.filter(isSelectableAxisKey);
    this.observableKeys = Array.from(new Set([...observationKeys, ...valueKeys, ...seedKeys].filter(isSelectableAxisKey)));

    if (!this.selectedXKey) {
      if (this.preferredXKey && this.observableKeys.includes(this.preferredXKey)) {
        this.selectedXKey = this.preferredXKey;
      } else {
        const timeKey = pickDefaultTimeKey(this.observableKeys);
        this.selectedXKey = timeKey || INDEX_X_KEY;
      }
    }
    const preferredYCandidate = this.observableKeys.find((key) => !STATIC_TIME_KEYS.includes(key)) ?? this.observableKeys[0] ?? "";
    if (!this.selectedYKey) {
      if (this.preferredYKey && this.observableKeys.includes(this.preferredYKey)) {
        this.selectedYKey = this.preferredYKey;
      } else {
        this.selectedYKey = preferredYCandidate;
      }
    } else if (
      STATIC_TIME_KEYS.includes(this.selectedYKey)
      && preferredYCandidate
      && !STATIC_TIME_KEYS.includes(preferredYCandidate)
      && !this.preferredYKey
    ) {
      this.selectedYKey = preferredYCandidate;
    }
    this.syncXAxisSelect();
    this.syncYAxisSelect();
    this.renderGraph();
  }

  syncXAxisSelect() {
    if (!this.xAxisSelect) return;
    const previous = String(this.xAxisSelect.value ?? "").trim();
    this.xAxisSelect.innerHTML = "";

    const indexOption = document.createElement("option");
    indexOption.value = INDEX_X_KEY;
    indexOption.textContent = "index (샘플순번)";
    this.xAxisSelect.appendChild(indexOption);

    this.observableKeys.forEach((key) => {
      const option = document.createElement("option");
      option.value = key;
      option.textContent = this.preferredXKey && key === this.preferredXKey ? `${key} (기본)` : key;
      this.xAxisSelect.appendChild(option);
    });

    const current = String(this.selectedXKey ?? "").trim();
    const hasCurrentOption = current === INDEX_X_KEY || this.observableKeys.includes(current);
    if (current && current !== INDEX_X_KEY && !hasCurrentOption) {
      const stickyOption = document.createElement("option");
      stickyOption.value = current;
      stickyOption.textContent = `${current} (유지)`;
      this.xAxisSelect.appendChild(stickyOption);
    }

    let next = INDEX_X_KEY;
    if (previous && (previous === INDEX_X_KEY || this.observableKeys.includes(previous))) {
      next = previous;
    } else if (current) {
      next = current;
    }
    this.selectedXKey = next;
    this.xAxisSelect.value = next;
  }

  syncYAxisSelect() {
    if (!this.yAxisSelect) return;
    const previous = String(this.yAxisSelect.value ?? "").trim();
    this.yAxisSelect.innerHTML = "";
    this.observableKeys.forEach((key) => {
      const option = document.createElement("option");
      option.value = key;
      option.textContent = this.preferredYKey && key === this.preferredYKey ? `${key} (기본)` : key;
      this.yAxisSelect.appendChild(option);
    });

    const current = String(this.selectedYKey ?? "").trim();
    const hasCurrentOption = this.observableKeys.includes(current);
    if (current && !hasCurrentOption) {
      const stickyOption = document.createElement("option");
      stickyOption.value = current;
      stickyOption.textContent = `${current} (유지)`;
      this.yAxisSelect.appendChild(stickyOption);
    }

    let next = "";
    if (previous && this.observableKeys.includes(previous)) {
      next = previous;
    } else if (current) {
      next = current;
    }
    this.selectedYKey = next;
    this.yAxisSelect.value = next;
  }

  buildSeriesPoints() {
    const xKey = String(this.selectedXKey ?? INDEX_X_KEY);
    const yKey = this.selectedYKey;
    if (!yKey) return [];
    const out = [];
    this.timeline.forEach((sample, index) => {
      const rawY = finite(sample.values?.[yKey]);
      if (rawY === null) return;
      let rawX = null;
      if (xKey === INDEX_X_KEY) {
        rawX = index;
      } else {
        rawX = finite(sample.values?.[xKey]);
      }
      if (rawX === null) {
        rawX = pickXAxisByTimeHeuristic(sample.values, index);
      }
      out.push({ x: rawX, y: rawY, sourceIndex: index });
    });
    return out;
  }

  buildRenderSeries() {
    const out = [];
    const focusPoints = [];
    const basePoints = this.buildSeriesPoints();
    const usePersistedBase = Boolean(this.preferPersistedGraph && this.persistedGraph?.series?.length);
    if (this.baseSeriesVisible && basePoints.length && !usePersistedBase) {
      const focus = basePoints[basePoints.length - 1];
      if (focus && Number.isFinite(focus.x) && Number.isFinite(focus.y)) {
        focusPoints.push({ x: focus.x, y: focus.y, color: this.baseSeriesColor });
      }
      out.push({
        id: this.selectedYKey || "y",
        kind: this.getGraphKind(),
        color: colorWithAlpha(this.baseSeriesColor, this.baseSeriesAlpha),
        points: basePoints.map((row) => ({ x: row.x, y: row.y })),
      });
    } else if (this.baseSeriesVisible && this.persistedGraph?.series?.length) {
      const primary = this.persistedGraph.series[0] ?? null;
      const primaryPoints = Array.isArray(primary?.points) ? primary.points : [];
      const focus = primaryPoints[primaryPoints.length - 1] ?? null;
      if (focus && Number.isFinite(focus.x) && Number.isFinite(focus.y)) {
        focusPoints.push({ x: focus.x, y: focus.y, color: this.baseSeriesColor });
      }
      this.persistedGraph.series.forEach((row, index) => {
        const points = Array.isArray(row?.points) ? row.points : [];
        if (!points.length) return;
        const color = index === 0
          ? colorWithAlpha(this.baseSeriesColor, this.baseSeriesAlpha)
          : (String(row?.color ?? "").trim() || undefined);
        out.push({
          id: String(row?.id ?? `persisted_${index + 1}`),
          kind: this.getGraphKind(),
          color,
          points,
        });
      });
    }
    const overlayRows = Array.isArray(this.overlaySeries) ? this.overlaySeries : [];
    overlayRows.forEach((row) => {
      if (!row || typeof row !== "object") return;
      const points = Array.isArray(row.points) ? row.points : [];
      if (!points.length) return;
      out.push({
        id: String(row.id ?? "").trim() || `overlay_${out.length + 1}`,
        kind: this.getGraphKind(),
        color: String(row.color ?? "").trim() || undefined,
        points,
      });
    });
    return { series: out, focusPoints };
  }

  computeAutoAxis() {
    const { series } = this.buildRenderSeries();
    if (!series.length) return null;
    const points = series.flatMap((item) => Array.isArray(item.points) ? item.points : []);
    if (!points.length) return null;
    let xMin = Math.min(...points.map((p) => p.x));
    let xMax = Math.max(...points.map((p) => p.x));
    let yMin = Math.min(...points.map((p) => p.y));
    let yMax = Math.max(...points.map((p) => p.y));
    if (xMin === xMax) xMax = xMin + 1;
    if (yMin === yMax) yMax = yMin + 1;
    const px = (xMax - xMin) * 0.05;
    const py = (yMax - yMin) * 0.08;
    return {
      x_min: xMin - px,
      x_max: xMax + px,
      y_min: yMin - py,
      y_max: yMax + py,
    };
  }

  renderGraph() {
    const { series, focusPoints } = this.buildRenderSeries();
    const autoAxis = this.computeAutoAxis();
    const axis = this.graphView.autoFit ? autoAxis : this.graphView.axis ?? autoAxis;
    this.emitAxisChange(axis, { autoFit: this.graphView.autoFit });

    renderGraphCanvas2d({
      canvas: this.graphCanvas,
      graph: series.length
        ? {
            axis,
            series,
          }
        : null,
      showGrid: Boolean(this.graphView.showGrid),
      showAxis: Boolean(this.graphView.showAxis),
      emptyText: "그래프 출력 없음",
      noPointsText: "그래프 데이터 없음",
      focusPoints,
    });
  }

  zoomByFactor(factor = 1) {
    const numeric = Number(factor);
    if (!Number.isFinite(numeric) || numeric <= 0) return;
    this.ensureManualAxis();
    const axis = this.graphView.axis;
    if (!axis) return;
    const cx = (axis.x_min + axis.x_max) / 2;
    const cy = (axis.y_min + axis.y_max) / 2;
    const halfW = ((axis.x_max - axis.x_min) / 2) * numeric;
    const halfH = ((axis.y_max - axis.y_min) / 2) * numeric;
    this.graphView.axis = {
      x_min: cx - halfW,
      x_max: cx + halfW,
      y_min: cy - halfH,
      y_max: cy + halfH,
    };
    this.graphView.autoFit = false;
    this.initialAxisLocked = true;
    this.renderGraph();
  }

  panByRatio(dxRatio = 0, dyRatio = 0) {
    this.ensureManualAxis();
    const axis = this.graphView.axis;
    if (!axis) return;
    const dx = Number(dxRatio);
    const dy = Number(dyRatio);
    const shiftX = (axis.x_max - axis.x_min) * (Number.isFinite(dx) ? dx : 0);
    const shiftY = (axis.y_max - axis.y_min) * (Number.isFinite(dy) ? dy : 0);
    this.graphView.axis = {
      x_min: axis.x_min + shiftX,
      x_max: axis.x_max + shiftX,
      y_min: axis.y_min + shiftY,
      y_max: axis.y_max + shiftY,
    };
    this.graphView.autoFit = false;
    this.initialAxisLocked = true;
    this.renderGraph();
  }

}
