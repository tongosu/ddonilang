import { renderSpace2dCanvas2d } from "../wasm_page_common.js";

function finiteRangeFromCamera(space2d) {
  const camera = space2d?.camera;
  const xMin = Number(camera?.x_min);
  const xMax = Number(camera?.x_max);
  const yMin = Number(camera?.y_min);
  const yMax = Number(camera?.y_max);
  if ([xMin, xMax, yMin, yMax].every(Number.isFinite) && xMax > xMin && yMax > yMin) {
    return { x_min: xMin, x_max: xMax, y_min: yMin, y_max: yMax };
  }
  return null;
}

function normalizeRange(range) {
  if (!range) return null;
  const xMin = Number(range.x_min ?? range.xMin);
  const xMax = Number(range.x_max ?? range.xMax);
  const yMin = Number(range.y_min ?? range.yMin);
  const yMax = Number(range.y_max ?? range.yMax);
  if (![xMin, xMax, yMin, yMax].every(Number.isFinite) || xMax <= xMin || yMax <= yMin) {
    return null;
  }
  return { x_min: xMin, x_max: xMax, y_min: yMin, y_max: yMax };
}

function cloneRange(range) {
  const normalized = normalizeRange(range);
  return normalized ? { ...normalized } : null;
}

function scaleRange(range, factor) {
  const cx = (range.x_min + range.x_max) / 2;
  const cy = (range.y_min + range.y_max) / 2;
  const halfW = ((range.x_max - range.x_min) / 2) * factor;
  const halfH = ((range.y_max - range.y_min) / 2) * factor;
  return {
    x_min: cx - halfW,
    x_max: cx + halfW,
    y_min: cy - halfH,
    y_max: cy + halfH,
  };
}

export class Bogae {
  constructor({ canvas, onRangeChange } = {}) {
    this.canvas = canvas;
    this.onRangeChange = typeof onRangeChange === "function" ? onRangeChange : null;
    this.lockInitialRange = true;
    this.initialRangeLocked = false;
    this.view = {
      autoFit: true,
      range: null,
    };
    this.lastAutoRange = null;
    this.drag = {
      active: false,
      x: 0,
      y: 0,
    };
    this.lastSpace2d = null;
    this.bindInteractions();
  }

  emitRangeChange() {
    if (!this.onRangeChange) return;
    this.onRangeChange(this.getCurrentRange(), { autoFit: this.view.autoFit });
  }

  bindInteractions() {
    if (!this.canvas) return;

    this.canvas.addEventListener("wheel", (event) => {
      event.preventDefault();
      this.ensureManualRange();
      const current = normalizeRange(this.view.range);
      if (!current) return;
      const factor = event.deltaY < 0 ? 0.9 : 1.1;
      this.view.range = scaleRange(current, factor);
      this.view.autoFit = false;
      this.render(this.lastSpace2d);
    });

    this.canvas.addEventListener("mousedown", (event) => {
      this.ensureManualRange();
      if (!normalizeRange(this.view.range)) return;
      this.drag.active = true;
      this.drag.x = event.clientX;
      this.drag.y = event.clientY;
    });

    window.addEventListener("mouseup", () => {
      this.drag.active = false;
    });

    window.addEventListener("mousemove", (event) => {
      if (!this.drag.active) return;
      const range = normalizeRange(this.view.range);
      if (!range) return;
      const widthPx = this.canvas.clientWidth || 1;
      const heightPx = this.canvas.clientHeight || 1;
      const dxPx = event.clientX - this.drag.x;
      const dyPx = event.clientY - this.drag.y;
      this.drag.x = event.clientX;
      this.drag.y = event.clientY;

      const dxWorld = (dxPx / widthPx) * (range.x_max - range.x_min);
      const dyWorld = (dyPx / heightPx) * (range.y_max - range.y_min);
      this.view.range = {
        x_min: range.x_min - dxWorld,
        x_max: range.x_max - dxWorld,
        y_min: range.y_min + dyWorld,
        y_max: range.y_max + dyWorld,
      };
      this.view.autoFit = false;
      this.render(this.lastSpace2d);
    });
  }

  ensureManualRange() {
    if (!this.view.autoFit) return;
    const preferred = normalizeRange(this.lastAutoRange) || normalizeRange(finiteRangeFromCamera(this.lastSpace2d));
    if (!preferred) return;
    this.view.range = preferred;
    this.view.autoFit = false;
    this.initialRangeLocked = true;
  }

  getCurrentRange() {
    if (this.view.autoFit) {
      return cloneRange(this.lastAutoRange) || cloneRange(finiteRangeFromCamera(this.lastSpace2d));
    }
    return cloneRange(this.view.range);
  }

  setRange(range) {
    const normalized = normalizeRange(range);
    if (!normalized) return false;
    this.view.range = normalized;
    this.view.autoFit = false;
    this.initialRangeLocked = true;
    this.render(this.lastSpace2d);
    return true;
  }

  setInitialRange(range) {
    const normalized = normalizeRange(range);
    if (!normalized) return;
    this.view.range = normalized;
    this.view.autoFit = false;
    this.initialRangeLocked = true;
  }

  zoomByFactor(factor = 1) {
    const numeric = Number(factor);
    if (!Number.isFinite(numeric) || numeric <= 0) return;
    this.ensureManualRange();
    const range = normalizeRange(this.view.range);
    if (!range) return;
    this.view.range = scaleRange(range, numeric);
    this.render(this.lastSpace2d);
  }

  zoomIn() {
    this.zoomByFactor(0.9);
  }

  zoomOut() {
    this.zoomByFactor(1.1);
  }

  panByRatio(dxRatio = 0, dyRatio = 0) {
    this.ensureManualRange();
    const range = normalizeRange(this.view.range);
    if (!range) return;
    const dx = Number(dxRatio);
    const dy = Number(dyRatio);
    const shiftX = (range.x_max - range.x_min) * (Number.isFinite(dx) ? dx : 0);
    const shiftY = (range.y_max - range.y_min) * (Number.isFinite(dy) ? dy : 0);
    this.view.range = {
      x_min: range.x_min + shiftX,
      x_max: range.x_max + shiftX,
      y_min: range.y_min + shiftY,
      y_max: range.y_max + shiftY,
    };
    this.render(this.lastSpace2d);
  }

  resetView() {
    this.view.autoFit = true;
    this.view.range = null;
    this.initialRangeLocked = false;
    this.render(this.lastSpace2d);
  }

  render(space2dData) {
    this.lastSpace2d = space2dData ?? this.lastSpace2d;
    const cameraRange = finiteRangeFromCamera(this.lastSpace2d);
    if (cameraRange) this.lastAutoRange = cameraRange;

    if (
      this.lockInitialRange &&
      this.view.autoFit &&
      !this.initialRangeLocked &&
      normalizeRange(this.lastAutoRange)
    ) {
      this.view.range = normalizeRange(this.lastAutoRange);
      this.view.autoFit = false;
      this.initialRangeLocked = true;
    }

    const viewState = this.view.autoFit
      ? { autoFit: true }
      : { autoFit: false, range: this.view.range };

    renderSpace2dCanvas2d({
      canvas: this.canvas,
      space2d: this.lastSpace2d,
      viewState,
      primitiveSource: "shapes",
      showGrid: false,
      showAxis: false,
      emptyText: "보개 출력 없음",
      noPointsText: "보개 도형 없음",
    });
    this.emitRangeChange();
  }
}
