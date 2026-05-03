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

function pushFinitePoint(out, x, y) {
  const nx = Number(x);
  const ny = Number(y);
  if (!Number.isFinite(nx) || !Number.isFinite(ny)) return;
  out.push({ x: nx, y: ny });
}

function collectRangePointsFromPrimitive(out, item) {
  if (!item || typeof item !== "object") return;
  const kind = String(item.kind ?? "").trim().toLowerCase();
  if (!kind) return;

  if (kind === "line" || kind === "arrow") {
    pushFinitePoint(out, item.x1, item.y1);
    pushFinitePoint(out, item.x2, item.y2);
    return;
  }
  if (kind === "circle") {
    const x = Number(item.x ?? item.cx);
    const y = Number(item.y ?? item.cy);
    const r = Math.abs(Number(item.r));
    if (!Number.isFinite(x) || !Number.isFinite(y)) return;
    pushFinitePoint(out, x, y);
    if (Number.isFinite(r) && r > 0) {
      pushFinitePoint(out, x - r, y - r);
      pushFinitePoint(out, x + r, y + r);
    }
    return;
  }
  if (kind === "point" || kind === "text") {
    pushFinitePoint(out, item.x, item.y);
    return;
  }
  if (kind === "rect") {
    pushFinitePoint(out, item.x1, item.y1);
    pushFinitePoint(out, item.x2, item.y2);
    return;
  }
  if (kind === "curve" || kind === "polyline" || kind === "polygon" || kind === "fill") {
    const points = Array.isArray(item.points) ? item.points : [];
    points.forEach((pt) => {
      pushFinitePoint(out, pt?.x, pt?.y);
    });
  }
}

function finiteRangeFromSpace2d(space2d) {
  if (!space2d || typeof space2d !== "object") return null;
  const points = [];
  const basePoints = Array.isArray(space2d.points) ? space2d.points : [];
  basePoints.forEach((pt) => {
    pushFinitePoint(points, pt?.x, pt?.y);
  });
  const shapes = Array.isArray(space2d.shapes) ? space2d.shapes : [];
  shapes.forEach((item) => collectRangePointsFromPrimitive(points, item));
  const drawlist = Array.isArray(space2d.drawlist) ? space2d.drawlist : [];
  drawlist.forEach((item) => collectRangePointsFromPrimitive(points, item));

  if (!points.length) return null;
  let xMin = Infinity;
  let xMax = -Infinity;
  let yMin = Infinity;
  let yMax = -Infinity;
  points.forEach((pt) => {
    xMin = Math.min(xMin, pt.x);
    xMax = Math.max(xMax, pt.x);
    yMin = Math.min(yMin, pt.y);
    yMax = Math.max(yMax, pt.y);
  });
  if (![xMin, xMax, yMin, yMax].every(Number.isFinite)) return null;
  if (xMax <= xMin) {
    xMin -= 1;
    xMax += 1;
  }
  if (yMax <= yMin) {
    yMin -= 1;
    yMax += 1;
  }
  return { x_min: xMin, x_max: xMax, y_min: yMin, y_max: yMax };
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
    this.renderMode = "none";
    this.lockInitialRange = false;
    this.initialRangeLocked = false;
    this.view = {
      autoFit: true,
      range: null,
      showGrid: false,
      showAxis: false,
      showXAxisTicks: false,
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
      if (this.renderMode !== "space2d") return;
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
      if (this.renderMode !== "space2d") return;
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
      if (this.renderMode !== "space2d") return;
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

  getGuides() {
    return {
      showGrid: Boolean(this.view.showGrid),
      showAxis: Boolean(this.view.showAxis),
      showXAxisTicks: Boolean(this.view.showXAxisTicks),
    };
  }

  setGuides({ showGrid = null, showAxis = null, showXAxisTicks = null } = {}) {
    let changed = false;
    if (typeof showGrid === "boolean" && this.view.showGrid !== showGrid) {
      this.view.showGrid = showGrid;
      changed = true;
    }
    if (typeof showAxis === "boolean" && this.view.showAxis !== showAxis) {
      this.view.showAxis = showAxis;
      changed = true;
    }
    if (typeof showXAxisTicks === "boolean" && this.view.showXAxisTicks !== showXAxisTicks) {
      this.view.showXAxisTicks = showXAxisTicks;
      changed = true;
    }
    if (changed) {
      this.render(this.lastSpace2d);
    }
    return this.getGuides();
  }

  render(space2dData) {
    // Explicit null/undefined input should clear stale frame data.
    // Without this, "보개 없음" 상태에서도 이전 보개가 남아 보일 수 있다.
    if (arguments.length > 0) {
      this.lastSpace2d = space2dData ?? null;
      this.renderMode = this.lastSpace2d ? "space2d" : "none";
    }
    const cameraRange = finiteRangeFromCamera(this.lastSpace2d);
    const inferredRange = cameraRange ?? finiteRangeFromSpace2d(this.lastSpace2d);
    if (inferredRange) this.lastAutoRange = inferredRange;

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
      ? { autoFit: true, baseRange: this.lastAutoRange }
      : { autoFit: false, range: this.view.range, baseRange: this.lastAutoRange };

    renderSpace2dCanvas2d({
      canvas: this.canvas,
      space2d: this.lastSpace2d,
      viewState,
      primitiveSource: "shapes",
      showGrid: Boolean(this.view.showGrid),
      showAxis: Boolean(this.view.showAxis),
      showXAxisTicks: Boolean(this.view.showXAxisTicks),
      emptyText: "보개 출력 없음",
      noPointsText: "보개 도형 없음",
    });
    this.emitRangeChange();
  }

  // ─── 콘솔 격자 보개 ────────────────────────────────────────────────────────
  // grid2d와 같은 캔버스에 문자 격자(터미널 스타일)를 렌더한다.
  // lines: string[] — 표시할 텍스트 줄 목록 (빈 배열이면 빈 격자만 표시)
  renderConsoleGrid(lines = []) {
    const canvas = this.canvas;
    if (!canvas) return;
    this.renderMode = "console-grid";
    this.lastSpace2d = null;
    this.lastAutoRange = null;
    this.view.autoFit = true;
    this.view.range = null;
    this.initialRangeLocked = false;
    this.drag.active = false;

    const rect = typeof canvas.getBoundingClientRect === "function" ? canvas.getBoundingClientRect() : null;
    const parentRect =
      typeof canvas.parentElement?.getBoundingClientRect === "function"
        ? canvas.parentElement.getBoundingClientRect()
        : null;
    const displayW = Math.max(
      1,
      Math.round(Number(canvas.clientWidth || rect?.width || canvas.parentElement?.clientWidth || parentRect?.width || 640)),
    );
    const displayH = Math.max(
      1,
      Math.round(Number(canvas.clientHeight || rect?.height || canvas.parentElement?.clientHeight || parentRect?.height || 360)),
    );
    if (!displayW || !displayH) return;

    // 캔버스 해상도를 표시 크기에 맞춤 (devicePixelRatio 불필요, CSS가 맞춤)
    if (canvas.width !== displayW || canvas.height !== displayH) {
      canvas.width = displayW;
      canvas.height = displayH;
    }

    const ctx = canvas.getContext("2d");
    const W = canvas.width;
    const H = canvas.height;

    // ── 셀 크기 계산 ──────────────────────────────────────────────────────────
    // 너비를 기준으로 폰트 크기를 자동 조정 (60열 기준)
    const FONT_PX = Math.max(11, Math.min(16, Math.floor(W / 58)));
    const CELL_W = Math.round(FONT_PX * 0.62); // 모노스페이스 ASCII 비율
    const CELL_H = Math.round(FONT_PX * 1.75);
    const COLS = Math.floor(W / CELL_W);
    const ROWS = Math.floor(H / CELL_H);

    // ── 배경 ──────────────────────────────────────────────────────────────────
    ctx.fillStyle = "#080c18";
    ctx.fillRect(0, 0, W, H);

    // ── 격자선 ────────────────────────────────────────────────────────────────
    ctx.strokeStyle = "rgba(30, 70, 140, 0.28)";
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    for (let c = 0; c <= COLS; c++) {
      ctx.moveTo(c * CELL_W, 0);
      ctx.lineTo(c * CELL_W, H);
    }
    for (let r = 0; r <= ROWS; r++) {
      ctx.moveTo(0, r * CELL_H);
      ctx.lineTo(W, r * CELL_H);
    }
    ctx.stroke();

    // ── 텍스트 렌더 ───────────────────────────────────────────────────────────
    ctx.font = `${FONT_PX}px "Courier New", Courier, monospace`;
    ctx.textBaseline = "middle";

    const allLines = Array.isArray(lines) ? lines.map((l) => String(l ?? "")) : [];
    // 화면에 넘치면 최근 줄부터 (터미널 스크롤 동작)
    const visibleLines = allLines.length > ROWS ? allLines.slice(-ROWS) : allLines;

    for (let rowIdx = 0; rowIdx < visibleLines.length; rowIdx++) {
      const line = visibleLines[rowIdx];
      const cellY = rowIdx * CELL_H + CELL_H / 2;

      let colIdx = 0;
      for (let i = 0; i < line.length && colIdx < COLS; i++) {
        const ch = line[i];
        const cellX = colIdx * CELL_W + 1;

        // 문자별 색상
        if (ch >= "0" && ch <= "9" || ch === ".") {
          ctx.fillStyle = "#7dd3fc"; // 숫자: 파랑
        } else if (ch === "=" || ch === ":" || ch === "[" || ch === "]") {
          ctx.fillStyle = "#475569"; // 구분자: 어두운 회색
        } else if (ch === " ") {
          colIdx += 1;
          continue; // 공백은 스킵 (격자가 배경 역할)
        } else {
          ctx.fillStyle = "#cbd5e1"; // 일반 텍스트: 밝은 회색
        }

        ctx.fillText(ch, cellX, cellY);

        // CJK(한국어 등) 문자는 셀 2칸 차지
        const charW = ctx.measureText(ch).width;
        colIdx += charW > CELL_W * 1.3 ? 2 : 1;
      }
    }

    // ── 빈 격자 안내문 (출력 없을 때) ────────────────────────────────────────
    if (allLines.length === 0) {
      ctx.fillStyle = "rgba(71, 85, 105, 0.6)";
      ctx.font = `${FONT_PX + 1}px "Courier New", Courier, monospace`;
      ctx.textBaseline = "middle";
      const msg = "콘솔 출력 없음";
      const msgW = ctx.measureText(msg).width;
      ctx.fillText(msg, (W - msgW) / 2, H / 2);
    }
  }
}
