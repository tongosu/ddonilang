#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function sortJson(value) {
  if (Array.isArray(value)) {
    return value.map((item) => sortJson(item));
  }
  if (!value || typeof value !== "object") {
    return value;
  }
  return Object.fromEntries(
    Object.keys(value)
      .sort((a, b) => a.localeCompare(b))
      .map((key) => [key, sortJson(value[key])]),
  );
}

function formatJson(value) {
  return `${JSON.stringify(sortJson(value), null, 2)}\n`;
}

function createElementStub(tagName = "div") {
  return {
    tagName: String(tagName ?? "div").toUpperCase(),
    className: "",
    textContent: "",
    dataset: {},
    value: "",
    children: [],
    appendChild(child) {
      this.children.push(child);
      return child;
    },
    replaceChildren(...items) {
      this.children = items;
    },
    addEventListener() {},
  };
}

function createCanvasStub() {
  return {
    width: 0,
    height: 0,
    calls: {
      clearRect: 0,
      fillRect: 0,
      strokeRect: 0,
      fillText: [],
    },
    getContext(type) {
      if (type !== "2d") return null;
      return {
        fillStyle: "",
        strokeStyle: "",
        font: "",
        clearRect: () => {
          this.calls.clearRect += 1;
        },
        fillRect: () => {
          this.calls.fillRect += 1;
        },
        strokeRect: () => {
          this.calls.strokeRect += 1;
        },
        fillText: (text) => {
          this.calls.fillText.push(String(text ?? ""));
        },
      };
    },
    addEventListener() {},
    removeEventListener() {},
    getBoundingClientRect() {
      return {
        left: 0,
        top: 0,
        width: this.width || 1,
        height: this.height || 1,
      };
    },
  };
}

async function readJson(filePath) {
  return JSON.parse(await fs.readFile(filePath, "utf8"));
}

async function main() {
  const args = process.argv.slice(2);
  const update = args.includes("--update");
  const packArg = args.find((item) => !item.startsWith("--")) ?? "pack/block_codec_rpg_alrimsi_v1";
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const rootDir = path.resolve(__dirname, "..");
  const packDir = path.resolve(rootDir, packArg);

  const codecUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "ddn_block_codec.js"),
  ).href;
  const paletteUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "rpgbox_palette.js"),
  ).href;
  const engineUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "ddn_block_engine.js"),
  ).href;
  const screenUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "screens", "rpg_box.js"),
  ).href;
  const canonUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "wasm_canon_runtime.js"),
  ).href;

  const codecMod = await import(codecUrl);
  const paletteMod = await import(paletteUrl);
  const engineMod = await import(engineUrl);
  const screenMod = await import(screenUrl);
  const canonMod = await import(canonUrl);

  const fixture = await readJson(path.join(packDir, "fixtures", "block.detjson"));
  const sourceText = await fs.readFile(path.join(packDir, String(fixture.source_ddn ?? "fixtures/source.ddn")), "utf8");
  const wasmBytes = await fs.readFile(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm", "ddonirang_tool_bg.wasm"),
  );
  const canon = await canonMod.createWasmCanon({ cacheBust: 0, initInput: wasmBytes });

  const decoded = await codecMod.decodeDdnToBlocks(sourceText, { canon, mode: "rpg" });
  const palette = paletteMod.RPGBOX_PALETTE;
  const initialBlocks = (Array.isArray(fixture.initial_blocks) ? fixture.initial_blocks : []).map((row) =>
    codecMod.buildPaletteBlock(String(row.kind ?? ""), {
      fields: row.fields && typeof row.fields === "object" ? row.fields : {},
      inputs: row.inputs && typeof row.inputs === "object" ? row.inputs : {},
    }, palette),
  );
  const encoded = codecMod.encodeBlocksToDdn(initialBlocks);

  const paletteEl = createElementStub("div");
  const canvasEl = createElementStub("div");
  const engine = new engineMod.DdnBlockEngine({ paletteEl, canvasEl, createElement: createElementStub });
  engine.init(palette, initialBlocks);
  const appended = engine.appendPaletteBlock(palette.categories.at(-1).blocks[0]);

  const screen = new screenMod.RpgBoxScreen({
    canvas: createCanvasStub(),
    editorEl: { value: "" },
    statusEl: { textContent: "" },
    sceneSelect: { value: "", options: [], addEventListener() {} },
    runButton: { addEventListener() {} },
    stopButton: { addEventListener() {} },
    resetButton: { addEventListener() {} },
    blockPaletteEl: createElementStub("div"),
    blockCanvasEl: createElementStub("div"),
    blockStatusEl: { textContent: "" },
  });
  screen.loadScene({
    gridState: fixture.grid,
    actors: fixture.actors,
    sourceText: sourceText,
    statusText: "RPG block smoke",
  });
  const screenDecoded = await screen.loadRpgBlocksFromSource(sourceText, canon);
  screen.appendRpgPaletteBlock("alrimsi_send", {
    fields: fixture.append_send_fields && typeof fixture.append_send_fields === "object" ? fixture.append_send_fields : {},
  });

  const output = {
    schema: "seamgrim.web.rpgbox_block_codec_smoke.v1",
    palette: {
      category_count: Array.isArray(palette.categories) ? palette.categories.length : 0,
      category_ids: Array.isArray(palette.categories) ? palette.categories.map((item) => String(item?.id ?? "")) : [],
      imja_block_kinds: Array.isArray(palette.categories)
        ? (palette.categories.find((item) => String(item?.id ?? "") === "imja")?.blocks ?? []).map((item) =>
            String(item?.kind ?? ""),
          )
        : [],
      state_block_kinds: Array.isArray(palette.categories)
        ? (palette.categories.find((item) => String(item?.id ?? "") === "state")?.blocks ?? []).map((item) =>
            String(item?.kind ?? ""),
          )
        : [],
    },
    codec: {
      decoded_block_kinds: Array.isArray(decoded.blocks) ? decoded.blocks.map((item) => String(item?.kind ?? "")) : [],
      decoded_handler_orders: Array.isArray(decoded.blocks)
        ? decoded.blocks.map((item) => Number(item?.fields?.order ?? -1))
        : [],
      decoded_scopes: Array.isArray(decoded.blocks)
        ? decoded.blocks.map((item) => String(item?.fields?.scope ?? ""))
        : [],
      decoded_errors: Array.isArray(decoded.errors) ? decoded.errors.map((item) => String(item?.message ?? "")) : [],
      encoded_source: encoded.trimEnd(),
    },
    engine: {
      initial_palette: engine.lastPaletteSummary,
      initial_canvas: engine.lastCanvasSummary,
      appended_kind: String(appended?.kind ?? ""),
      block_count_after_append: engine.getBlocks().length,
      palette_dom_children: Array.isArray(paletteEl.children) ? paletteEl.children.length : 0,
      canvas_dom_children: Array.isArray(canvasEl.children) ? canvasEl.children.length : 0,
    },
    screen: {
      decoded_block_count: Array.isArray(screenDecoded?.blocks) ? screenDecoded.blocks.length : 0,
      decoded_status: screen.getBlockSummary().status,
      block_count_after_append: screen.getBlockSummary().block_count,
      editor_text: String(screen.getSourceText() ?? "").trimEnd(),
      block_summary: screen.getBlockSummary(),
    },
  };

  const expectedPath = path.join(packDir, "expected", "rpgbox_block.detjson");
  const actualText = formatJson(output);
  if (update) {
    await fs.mkdir(path.dirname(expectedPath), { recursive: true });
    await fs.writeFile(expectedPath, actualText, "utf8");
    console.log(`rpgbox block smoke updated: ${path.relative(rootDir, expectedPath)}`);
    return;
  }
  const expectedText = await fs.readFile(expectedPath, "utf8");
  if (expectedText !== actualText) {
    console.error(`rpgbox block smoke mismatch: ${path.relative(rootDir, expectedPath)}`);
    process.exit(1);
  }
  console.log(`rpgbox block smoke ok: ${path.relative(rootDir, packDir)}`);
}

await main();
