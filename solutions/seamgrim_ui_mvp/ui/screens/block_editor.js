import { createWasmCanon } from "../runtime/index.js";
import { DdnBlockEngine } from "../block_editor/ddn_block_engine.js";
import { decodeDdnToBlocks, encodeBlocksToDdn } from "../block_editor/ddn_block_codec.js";
import { SEAMGRIM_PALETTE } from "../block_editor/seamgrim_palette.js";
import { RPGBOX_PALETTE } from "../block_editor/rpgbox_palette.js";

function guessBlockMode(sourceText, fallback = "seamgrim") {
  const text = String(sourceText ?? "");
  if (!text.trim()) return String(fallback ?? "seamgrim");
  if (text.includes("알림씨") || text.includes("~~>") || text.includes("받으면") || text.includes(":임자")) {
    return "rpg";
  }
  return "seamgrim";
}

export class BlockEditorScreen {
  constructor({
    root,
    onBack,
    onRun,
    onTextMode,
    onOpenAdvanced,
    canon = null,
    createCanon = createWasmCanon,
  } = {}) {
    this.root = root;
    this.onBack = typeof onBack === "function" ? onBack : () => {};
    this.onRun = typeof onRun === "function" ? onRun : () => {};
    this.onTextMode = typeof onTextMode === "function" ? onTextMode : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};
    this.canon = canon;
    this.createCanon = typeof createCanon === "function" ? createCanon : createWasmCanon;
    this.engine = null;
    this.sourceText = "";
    this.blockMode = "seamgrim";
    this.currentBlocks = [];
    this.lastDecode = null;
  }

  init() {
    this.titleEl = this.root.querySelector("#block-editor-title");
    this.modeSelectEl = this.root.querySelector("#block-editor-mode");
    this.resultEl = this.root.querySelector("#block-editor-result");
    this.summaryEl = this.root.querySelector("#block-editor-summary");
    this.generatedEl = this.root.querySelector("#block-ddn-preview");
    this.paletteEl = this.root.querySelector("#block-palette");
    this.canvasEl = this.root.querySelector("#block-canvas");

    this.engine = new DdnBlockEngine({
      paletteEl: this.paletteEl,
      canvasEl: this.canvasEl,
    });
    this.engine.onChange = (blocks) => {
      this.currentBlocks = Array.isArray(blocks) ? blocks : [];
      this.syncGeneratedDdn();
      this.renderSummary();
    };

    this.root.querySelector("#btn-back-from-block-editor")?.addEventListener("click", () => {
      this.onBack();
    });
    this.root.querySelector("#btn-block-to-text")?.addEventListener("click", () => {
      this.onTextMode(this.getDdn(), {
        title: String(this.titleEl?.textContent ?? "블록 편집"),
        mode: this.blockMode,
      });
    });
    this.root.querySelector("#btn-block-run")?.addEventListener("click", () => {
      this.onRun(this.getDdn(), {
        title: String(this.titleEl?.textContent ?? "블록 편집"),
        mode: this.blockMode,
      });
    });
    this.root.querySelector("#btn-advanced-block-editor")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });
    this.modeSelectEl?.addEventListener("change", () => {
      const nextMode = String(this.modeSelectEl?.value ?? "seamgrim");
      void this.reload({ mode: nextMode });
    });
  }

  async ensureCanon() {
    if (this.canon) return this.canon;
    this.canon = await this.createCanon({ cacheBust: 0 });
    return this.canon;
  }

  getPaletteForMode(mode = this.blockMode) {
    return String(mode ?? "seamgrim") === "rpg" ? RPGBOX_PALETTE : SEAMGRIM_PALETTE;
  }

  async loadSource(sourceText, { title = "블록 편집", mode = "" } = {}) {
    this.sourceText = String(sourceText ?? "");
    this.blockMode = String(mode ?? "").trim() || guessBlockMode(this.sourceText, "seamgrim");
    if (this.modeSelectEl) {
      this.modeSelectEl.value = this.blockMode;
    }
    if (this.titleEl) {
      this.titleEl.textContent = String(title ?? "블록 편집");
    }
    return this.reload({ mode: this.blockMode });
  }

  async reload({ mode = this.blockMode } = {}) {
    this.blockMode = String(mode ?? "seamgrim");
    const canon = await this.ensureCanon();
    const decoded = await decodeDdnToBlocks(this.sourceText, {
      canon,
      mode: this.blockMode,
    });
    this.lastDecode = decoded;
    this.currentBlocks = Array.isArray(decoded?.blocks) ? decoded.blocks : [];
    this.engine.init(decoded?.palette ?? this.getPaletteForMode(this.blockMode), this.currentBlocks);
    this.syncGeneratedDdn();
    this.renderSummary();
    return decoded;
  }

  syncGeneratedDdn() {
    const generated = encodeBlocksToDdn(this.currentBlocks);
    if (this.generatedEl) {
      this.generatedEl.value = generated;
    }
    return generated;
  }

  renderSummary() {
    const errorCount = Array.isArray(this.lastDecode?.errors) ? this.lastDecode.errors.length : 0;
    const message = `mode=${this.blockMode} blocks=${this.currentBlocks.length} errors=${errorCount}`;
    if (this.summaryEl) {
      this.summaryEl.textContent = message;
    }
    if (this.resultEl) {
      if (errorCount > 0) {
        const first = String(this.lastDecode.errors[0]?.message ?? "");
        this.resultEl.textContent = `${message} · ${first}`;
      } else {
        this.resultEl.textContent = `${message} · ok`;
      }
    }
  }

  getDdn() {
    return String(this.generatedEl?.value ?? encodeBlocksToDdn(this.currentBlocks));
  }

  getSummary() {
    return {
      mode: this.blockMode,
      block_count: this.currentBlocks.length,
      error_count: Array.isArray(this.lastDecode?.errors) ? this.lastDecode.errors.length : 0,
      generated_length: this.getDdn().length,
      palette_summary: this.engine?.lastPaletteSummary ?? null,
      canvas_summary: this.engine?.lastCanvasSummary ?? null,
    };
  }
}
