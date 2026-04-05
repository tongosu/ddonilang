import { NurimakerGridRenderer, normalizeGridState } from "../runtime/nurimaker_grid_renderer.js";
import { DdnBlockEngine } from "../block_editor/ddn_block_engine.js";
import { buildPaletteBlock, decodeDdnToBlocks, encodeBlocksToDdn } from "../block_editor/ddn_block_codec.js";
import { RPGBOX_PALETTE } from "../block_editor/rpgbox_palette.js";

function ensureButton(button, label) {
  if (button) return button;
  if (typeof document === "undefined" || typeof document.createElement !== "function") {
    return {
      textContent: label,
      addEventListener() {},
    };
  }
  const next = document.createElement("button");
  next.type = "button";
  next.textContent = label;
  return next;
}

function ensureTextarea(textarea) {
  if (textarea) return textarea;
  if (typeof document === "undefined" || typeof document.createElement !== "function") {
    return { value: "" };
  }
  return document.createElement("textarea");
}

function ensureStatusElement(element) {
  if (element) return element;
  if (typeof document === "undefined" || typeof document.createElement !== "function") {
    return { textContent: "" };
  }
  return document.createElement("div");
}

function ensureSelect(select) {
  if (select) return select;
  if (typeof document === "undefined" || typeof document.createElement !== "function") {
    return {
      value: "",
      options: [],
      addEventListener() {},
    };
  }
  return document.createElement("select");
}

function ensureCanvas(canvas) {
  if (canvas) return canvas;
  if (typeof document === "undefined" || typeof document.createElement !== "function") {
    return null;
  }
  return document.createElement("canvas");
}

function ensureDiv(element) {
  if (element) return element;
  if (typeof document === "undefined" || typeof document.createElement !== "function") {
    return null;
  }
  return document.createElement("div");
}

function isDomContainer(node) {
  return Boolean(node) && typeof node === "object" && typeof node.appendChild === "function";
}

function cloneGridState(state) {
  return normalizeGridState(JSON.parse(JSON.stringify(state ?? {})));
}

export class RpgBoxScreen {
  constructor({
    root = null,
    canvas = null,
    editorEl = null,
    statusEl = null,
    runButton = null,
    stopButton = null,
    resetButton = null,
    sceneSelect = null,
    blockPaletteEl = null,
    blockCanvasEl = null,
    blockStatusEl = null,
    onRun = null,
    onStop = null,
    onReset = null,
    onSceneChange = null,
    onCellPaint = null,
  } = {}) {
    this.root = root;
    this.canvas = ensureCanvas(canvas);
    this.editorEl = ensureTextarea(editorEl);
    this.statusEl = ensureStatusElement(statusEl);
    this.runButton = ensureButton(runButton, "실행");
    this.stopButton = ensureButton(stopButton, "중지");
    this.resetButton = ensureButton(resetButton, "리셋");
    this.sceneSelect = ensureSelect(sceneSelect);
    this.blockPaletteEl = ensureDiv(blockPaletteEl);
    this.blockCanvasEl = ensureDiv(blockCanvasEl);
    this.blockStatusEl = ensureStatusElement(blockStatusEl);
    this.onRun = typeof onRun === "function" ? onRun : () => {};
    this.onStop = typeof onStop === "function" ? onStop : () => {};
    this.onReset = typeof onReset === "function" ? onReset : () => {};
    this.onSceneChange = typeof onSceneChange === "function" ? onSceneChange : () => {};
    this.onCellPaint = typeof onCellPaint === "function" ? onCellPaint : () => {};
    this.selectedTileKind = 1;
    this.clickLog = [];
    this.sceneOptions = [];
    this.initialGridState = null;
    this.gridState = normalizeGridState({ gridW: 1, gridH: 1, tiles: [[0]] });
    this.actorList = [];
    this.blockEngine = null;
    this.blockPalette = RPGBOX_PALETTE;
    this.blockBlocks = [];

    this.maybeMountTemplate();
    this.renderer = new NurimakerGridRenderer(this.canvas, { gridState: this.gridState });
    this.unsubscribeCellClick = this.renderer.onCellClick((hit) => {
      this.handleCellPaint(hit);
    });
    if (this.blockPaletteEl || this.blockCanvasEl) {
      this.blockEngine = new DdnBlockEngine({
        paletteEl: this.blockPaletteEl,
        canvasEl: this.blockCanvasEl,
      });
      this.blockEngine.onChange = (blocks) => {
        this.blockBlocks = JSON.parse(JSON.stringify(blocks));
        this.editorEl.value = encodeBlocksToDdn(blocks);
        this.blockStatusEl.textContent = `블록 ${blocks.length}개`;
      };
      this.blockEngine.init(this.blockPalette, []);
    }
    this.bindControls();
  }

  maybeMountTemplate() {
    if (!isDomContainer(this.root) || !this.canvas || this.root.children?.length) return;
    const toolbar = document.createElement("div");
    toolbar.className = "rpg-box-toolbar";
    toolbar.appendChild(this.sceneSelect);
    toolbar.appendChild(this.runButton);
    toolbar.appendChild(this.stopButton);
    toolbar.appendChild(this.resetButton);

    const body = document.createElement("div");
    body.className = "rpg-box-body";
    body.appendChild(this.canvas);
    body.appendChild(this.editorEl);

    if (this.blockPaletteEl && this.blockCanvasEl) {
      const blockWrap = document.createElement("div");
      blockWrap.className = "rpg-box-block-wrap";
      blockWrap.appendChild(this.blockPaletteEl);
      blockWrap.appendChild(this.blockCanvasEl);
      body.appendChild(blockWrap);
    }

    this.root.appendChild(toolbar);
    this.root.appendChild(body);
    this.root.appendChild(this.statusEl);
    if (this.blockStatusEl) {
      this.root.appendChild(this.blockStatusEl);
    }
  }

  bindControls() {
    this.runButton?.addEventListener?.("click", () => {
      this.onRun({
        sceneId: this.getSceneValue(),
        sourceText: this.getSourceText(),
        gridState: this.getGridState(),
      });
    });
    this.stopButton?.addEventListener?.("click", () => {
      this.onStop();
    });
    this.resetButton?.addEventListener?.("click", () => {
      this.resetGrid();
      this.onReset({
        gridState: this.getGridState(),
      });
    });
    this.sceneSelect?.addEventListener?.("change", () => {
      this.onSceneChange(this.getSceneValue());
    });
  }

  destroy() {
    if (typeof this.unsubscribeCellClick === "function") {
      this.unsubscribeCellClick();
    }
    this.renderer?.destroy?.();
  }

  appendRpgPaletteBlock(kind, overrides = {}) {
    if (!this.blockEngine) return null;
    const block = buildPaletteBlock(kind, overrides, this.blockPalette);
    this.blockBlocks.push(block);
    this.blockEngine.setBlocks(this.blockBlocks);
    this.editorEl.value = encodeBlocksToDdn(this.blockBlocks);
    this.blockStatusEl.textContent = `블록 ${this.blockBlocks.length}개`;
    return block;
  }

  async loadRpgBlocksFromSource(sourceText, canon = null) {
    if (!this.blockEngine) return null;
    const decoded = await decodeDdnToBlocks(sourceText, { canon, mode: "rpg" });
    this.blockBlocks = Array.isArray(decoded?.blocks) ? decoded.blocks : [];
    this.blockEngine.init(this.blockPalette, this.blockBlocks);
    this.editorEl.value = encodeBlocksToDdn(this.blockBlocks);
    this.blockStatusEl.textContent = decoded?.errors?.length
      ? `블록 ${this.blockBlocks.length}개 / 오류 ${decoded.errors.length}건`
      : `블록 ${this.blockBlocks.length}개`;
    return decoded;
  }

  setSceneOptions(options, selectedValue = "") {
    this.sceneOptions = Array.isArray(options)
      ? options.map((item) => ({
          value: String(item?.value ?? item?.id ?? ""),
          label: String(item?.label ?? item?.name ?? item?.value ?? item?.id ?? ""),
        }))
      : [];
    const resolvedValue =
      String(selectedValue ?? "").trim() || String(this.sceneOptions[0]?.value ?? "").trim();
    if (Array.isArray(this.sceneSelect?.options)) {
      this.sceneSelect.options = this.sceneOptions.map((item) => ({ ...item }));
    } else if (this.sceneSelect && typeof this.sceneSelect.appendChild === "function" && typeof document !== "undefined") {
      this.sceneSelect.textContent = "";
      this.sceneOptions.forEach((item) => {
        const option = document.createElement("option");
        option.value = item.value;
        option.textContent = item.label;
        this.sceneSelect.appendChild(option);
      });
    }
    this.sceneSelect.value = resolvedValue;
    return this.sceneOptions;
  }

  getSceneValue() {
    return String(this.sceneSelect?.value ?? "");
  }

  setSelectedTileKind(kind) {
    this.selectedTileKind = Number.isFinite(Number(kind)) ? Math.max(0, Math.trunc(Number(kind))) : 0;
    return this.selectedTileKind;
  }

  setSourceText(text) {
    this.editorEl.value = String(text ?? "");
  }

  getSourceText() {
    return String(this.editorEl?.value ?? "");
  }

  setStatus(text) {
    this.statusEl.textContent = String(text ?? "");
  }

  loadScene({ gridState, actors = [], sourceText = "", statusText = "" } = {}) {
    this.initialGridState = cloneGridState(gridState);
    this.gridState = cloneGridState(gridState);
    this.actorList = Array.isArray(actors) ? JSON.parse(JSON.stringify(actors)) : [];
    this.setSourceText(sourceText);
    this.setStatus(statusText);
    this.renderer.renderGrid(this.gridState);
    this.renderer.renderActors(this.actorList);
    if (this.blockEngine) {
      this.blockEngine.init(this.blockPalette, this.blockBlocks);
    }
    return this.getRenderSummary();
  }

  resetGrid() {
    if (!this.initialGridState) return false;
    this.gridState = cloneGridState(this.initialGridState);
    this.renderer.renderGrid(this.gridState);
    this.renderer.renderActors(this.actorList);
    this.setStatus("격자를 초기 상태로 되돌렸습니다.");
    return true;
  }

  handleCellPaint(hit) {
    const row = Number(hit?.row);
    const col = Number(hit?.col);
    if (!Number.isFinite(row) || !Number.isFinite(col)) return false;
    if (row < 0 || col < 0 || row >= this.gridState.gridH || col >= this.gridState.gridW) return false;
    this.clickLog.push({ row, col });
    this.gridState.tiles[row][col] = this.selectedTileKind;
    this.renderer.renderGrid(this.gridState);
    this.renderer.renderActors(this.actorList);
    const statusText = `칸 (${row},${col}) = ${this.selectedTileKind}`;
    this.setStatus(statusText);
    this.onCellPaint({
      row,
      col,
      kind: this.selectedTileKind,
      gridState: this.getGridState(),
    });
    return true;
  }

  getGridState() {
    return cloneGridState(this.gridState);
  }

  getRenderSummary() {
    return this.renderer?.lastRenderSummary ?? null;
  }

  getBlockSummary() {
    return {
      palette: this.blockEngine?.lastPaletteSummary ?? null,
      canvas: this.blockEngine?.lastCanvasSummary ?? null,
      status: String(this.blockStatusEl?.textContent ?? ""),
      block_count: this.blockBlocks.length,
    };
  }
}
