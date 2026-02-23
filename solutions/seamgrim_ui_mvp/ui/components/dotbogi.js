import { renderGraphCanvas2d } from "../wasm_page_common.js";
import { markdownToHtml } from "./markdown.js";

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

const INDEX_X_KEY = "__index__";

function normalizeKey(raw) {
  return String(raw ?? "").trim().toLowerCase();
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
  const candidates = ["t", "time", "시간", "tick", "프레임수"];
  const normalizedMap = new Map(
    keys.map((key) => [normalizeKey(key), String(key ?? "").trim()]).filter(([, key]) => Boolean(key)),
  );
  for (const candidate of candidates) {
    const found = normalizedMap.get(normalizeKey(candidate));
    if (found) return found;
  }
  return "";
}

function buildObservationTableRows(timeline, limit = 20) {
  const rows = Array.isArray(timeline) ? timeline.slice(-limit) : [];
  if (!rows.length) return { columns: [], rows: [] };
  const columnSet = new Set(["index"]);
  rows.forEach((sample) => {
    Object.keys(sample.values ?? {}).forEach((key) => columnSet.add(key));
  });
  const columns = Array.from(columnSet);
  const mappedRows = rows.map((sample, index) => {
    const row = { index: Number(sample.index ?? index) };
    columns.forEach((column) => {
      if (column === "index") return;
      row[column] = sample.values?.[column];
    });
    return row;
  });
  return { columns, rows: mappedRows };
}

export class DotbogiPanel {
  constructor({
    graphCanvas,
    tableEl,
    textEl,
    xAxisSelect,
    yAxisSelect,
    tabButtons = [],
    graphResetBtn = null,
    onAxisChange = null,
  } = {}) {
    this.graphCanvas = graphCanvas;
    this.tableEl = tableEl;
    this.textEl = textEl;
    this.xAxisSelect = xAxisSelect;
    this.yAxisSelect = yAxisSelect;
    this.tabButtons = Array.isArray(tabButtons) ? tabButtons : [];
    this.graphResetBtn = graphResetBtn;
    this.onAxisChange = typeof onAxisChange === "function" ? onAxisChange : null;

    this.activeTab = "graph";
    this.seedKeys = [];
    this.observableKeys = [];
    this.timeline = [];
    this.maxPoints = 500;
    this.selectedXKey = INDEX_X_KEY;
    this.selectedYKey = "";
    this.preferredXKey = "";
    this.preferredYKey = "";
    this.preferredAxisLock = true;
    this.lockInitialAxis = true;
    this.initialAxisLocked = false;
    this.graphView = {
      autoFit: true,
      axis: null,
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
    this.tabButtons.forEach((button) => {
      button.addEventListener("click", () => {
        this.switchTab(String(button.dataset.view ?? "graph"));
      });
    });

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

    this.graphResetBtn?.addEventListener("click", () => {
      this.resetAxis();
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

  resetAxis() {
    this.graphView.autoFit = true;
    this.graphView.axis = null;
    this.initialAxisLocked = false;
    this.renderGraph();
  }

  switchTab(view) {
    this.activeTab = view;
    document.querySelectorAll(".panel-tab").forEach((button) => {
      button.classList.toggle("active", String(button.dataset.view) === view);
    });
    document.querySelectorAll(".panel-content").forEach((content) => {
      const id = String(content.id || "");
      const on = id === `dotbogi-${view}`;
      content.classList.toggle("active", on);
      content.classList.toggle("hidden", !on);
    });
  }

  setSeedKeys(keys = []) {
    this.seedKeys = Array.isArray(keys) ? keys.map((v) => String(v ?? "").trim()).filter(Boolean) : [];
  }

  getSelectedAxes() {
    return {
      xKey: this.selectedXKey,
      yKey: this.selectedYKey,
    };
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
    this.renderGraph();
  }

  isPreferredAxisLock() {
    return this.preferredAxisLock;
  }

  setPreferredAxisLock(locked) {
    this.preferredAxisLock = Boolean(locked);
    if (this.xAxisSelect) this.xAxisSelect.disabled = this.preferredAxisLock;
    if (this.yAxisSelect) this.yAxisSelect.disabled = this.preferredAxisLock;
    if (this.preferredAxisLock) {
      let changed = false;
      if (this.preferredXKey && this.observableKeys.includes(this.preferredXKey)) {
        this.selectedXKey = this.preferredXKey;
        changed = true;
      }
      if (this.preferredYKey && this.observableKeys.includes(this.preferredYKey)) {
        this.selectedYKey = this.preferredYKey;
        changed = true;
      }
      this.syncXAxisSelect();
      this.syncYAxisSelect();
      if (changed) {
        this.resetAxis();
        return;
      }
    }
    this.renderGraph();
  }

  setPreferredXKey(key) {
    this.preferredXKey = String(key ?? "").trim();
    if (!this.preferredXKey) return;
    if (!this.observableKeys.includes(this.preferredXKey)) return;
    if (!this.preferredAxisLock) {
      this.syncXAxisSelect();
      this.renderGraph();
      return;
    }
    this.selectedXKey = this.preferredXKey;
    this.syncXAxisSelect();
    this.resetAxis();
  }

  setPreferredYKey(key) {
    this.preferredYKey = String(key ?? "").trim();
    if (!this.preferredYKey) return;
    if (!this.observableKeys.includes(this.preferredYKey)) return;
    if (!this.preferredAxisLock) {
      this.syncYAxisSelect();
      this.renderGraph();
      return;
    }
    this.selectedYKey = this.preferredYKey;
    this.syncYAxisSelect();
    this.resetAxis();
  }

  setText(markdown) {
    if (!this.textEl) return;
    this.textEl.innerHTML = markdownToHtml(markdown);
  }

  clearTimeline() {
    this.timeline = [];
    this.selectedXKey = INDEX_X_KEY;
    this.selectedYKey = "";
    this.graphView.autoFit = true;
    this.graphView.axis = null;
    this.initialAxisLocked = false;
    this.renderGraph();
    this.renderTable();
  }

  appendObservation(observation) {
    const values = observation && typeof observation.values === "object" ? observation.values : {};
    const nextIndex = this.timeline.length;
    this.timeline.push({ index: nextIndex, values: { ...values } });
    if (this.timeline.length > this.maxPoints) {
      this.timeline = this.timeline.slice(this.timeline.length - this.maxPoints);
      this.timeline.forEach((item, idx) => {
        item.index = idx;
      });
    }

    const observationKeys = Array.isArray(observation?.channels)
      ? observation.channels.map((item) => String(item?.key ?? "").trim()).filter(Boolean)
      : Object.keys(values);
    this.observableKeys = Array.from(new Set([...observationKeys, ...this.seedKeys].filter(Boolean)));

    if (!this.selectedXKey || (this.selectedXKey !== INDEX_X_KEY && !this.observableKeys.includes(this.selectedXKey))) {
      if (this.preferredAxisLock && this.preferredXKey && this.observableKeys.includes(this.preferredXKey)) {
        this.selectedXKey = this.preferredXKey;
      } else {
        const timeKey = pickDefaultTimeKey(this.observableKeys);
        this.selectedXKey = timeKey || INDEX_X_KEY;
      }
    }
    if (!this.selectedYKey || !this.observableKeys.includes(this.selectedYKey)) {
      if (this.preferredAxisLock && this.preferredYKey && this.observableKeys.includes(this.preferredYKey)) {
        this.selectedYKey = this.preferredYKey;
      } else {
        this.selectedYKey = this.observableKeys[0] ?? "";
      }
    }
    this.syncXAxisSelect();
    this.syncYAxisSelect();
    this.renderGraph();
    this.renderTable();
  }

  syncXAxisSelect() {
    if (!this.xAxisSelect) return;
    const previous = String(this.xAxisSelect.value ?? this.selectedXKey ?? INDEX_X_KEY).trim();
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

    if (previous === INDEX_X_KEY || this.observableKeys.includes(previous)) {
      this.xAxisSelect.value = previous;
      this.selectedXKey = previous;
      return;
    }
    this.xAxisSelect.value = this.selectedXKey || INDEX_X_KEY;
  }

  syncYAxisSelect() {
    if (!this.yAxisSelect) return;
    const previous = String(this.yAxisSelect.value ?? this.selectedYKey ?? "").trim();
    this.yAxisSelect.innerHTML = "";
    this.observableKeys.forEach((key) => {
      const option = document.createElement("option");
      option.value = key;
      option.textContent = this.preferredYKey && key === this.preferredYKey ? `${key} (기본)` : key;
      this.yAxisSelect.appendChild(option);
    });
    if (this.observableKeys.includes(previous)) {
      this.yAxisSelect.value = previous;
      this.selectedYKey = previous;
      return;
    }
    this.yAxisSelect.value = this.selectedYKey;
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
      out.push({ x: rawX, y: rawY });
    });
    return out;
  }

  computeAutoAxis() {
    const points = this.buildSeriesPoints();
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
    const points = this.buildSeriesPoints();
    const autoAxis = this.computeAutoAxis();
    if (
      this.lockInitialAxis &&
      this.graphView.autoFit &&
      !this.initialAxisLocked &&
      autoAxis
    ) {
      this.graphView.axis = autoAxis;
      this.graphView.autoFit = false;
      this.initialAxisLocked = true;
    }
    const axis = this.graphView.autoFit ? autoAxis : this.graphView.axis ?? autoAxis;
    this.emitAxisChange(axis, { autoFit: this.graphView.autoFit });

    renderGraphCanvas2d({
      canvas: this.graphCanvas,
      graph: points.length
        ? {
            axis,
            series: [
              {
                id: this.selectedYKey || "y",
                color: "#22d3ee",
                points,
              },
            ],
          }
        : null,
      emptyText: "그래프 출력 없음",
      noPointsText: "그래프 데이터 없음",
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

  renderTable(tableView = null) {
    if (!this.tableEl) return;

    const normalizedTable = tableView && Array.isArray(tableView.columns) && Array.isArray(tableView.rows)
      ? tableView
      : buildObservationTableRows(this.timeline, 20);

    if (!normalizedTable.columns.length) {
      this.tableEl.innerHTML = "";
      return;
    }

    const thead = `
      <thead>
        <tr>${normalizedTable.columns.map((column) => `<th>${column}</th>`).join("")}</tr>
      </thead>
    `;

    const bodyRows = normalizedTable.rows.map((row) => {
      const cols = normalizedTable.columns.map((column) => {
        const value = row?.[column];
        const text = Number.isFinite(Number(value)) ? Number(value).toFixed(4) : String(value ?? "");
        return `<td>${text}</td>`;
      });
      return `<tr>${cols.join("")}</tr>`;
    });

    this.tableEl.innerHTML = `${thead}<tbody>${bodyRows.join("")}</tbody>`;
  }
}
