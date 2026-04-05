#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function sortJson(value) {
  if (Array.isArray(value)) return value.map((item) => sortJson(item));
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.keys(value)
      .sort((a, b) => a.localeCompare(b))
      .map((key) => [key, sortJson(value[key])]),
  );
}

function formatJson(value) {
  return `${JSON.stringify(sortJson(value), null, 2)}\n`;
}

function summarizeExprNode(node) {
  if (!node || typeof node !== "object") return null;
  const inputs = Object.fromEntries(
    Object.entries(node.inputs && typeof node.inputs === "object" ? node.inputs : {}).map(([key, value]) => [
      key,
      (Array.isArray(value) ? value : []).map((child) => summarizeExprNode(child)),
    ]),
  );
  return {
    kind: String(node.kind ?? ""),
    text: String(node.text ?? ""),
    fields: node.fields && typeof node.fields === "object" ? node.fields : {},
    inputs,
  };
}

function summarizeBlockExprs(blocks) {
  return (Array.isArray(blocks) ? blocks : []).map((block) => ({
    id: String(block?.id ?? ""),
    kind: String(block?.kind ?? ""),
    exprs: Object.fromEntries(
      Object.entries(block?.exprs && typeof block.exprs === "object" ? block.exprs : {}).map(([key, value]) => [
        key,
        summarizeExprNode(value),
      ]),
    ),
  }));
}

function createStubNode(tagName = "div") {
  const listeners = new Map();
  return {
    tagName: String(tagName ?? "div").toUpperCase(),
    className: "",
    textContent: "",
    value: "",
    children: [],
    dataset: {},
    options: [],
    appendChild(child) {
      this.children.push(child);
      return child;
    },
    replaceChildren(...items) {
      this.children = items;
    },
    addEventListener(type, handler) {
      const list = listeners.get(type) ?? [];
      list.push(handler);
      listeners.set(type, list);
    },
    emit(type, payload = {}) {
      const list = listeners.get(type) ?? [];
      list.forEach((handler) => handler({ target: this, ...payload }));
    },
    click() {
      this.emit("click");
    },
  };
}

function createRoot(elements) {
  return {
    querySelector(selector) {
      return elements.get(selector) ?? null;
    },
  };
}

function visitBlocks(blocks, visitor) {
  for (const block of Array.isArray(blocks) ? blocks : []) {
    visitor(block);
    Object.values(block?.inputs && typeof block.inputs === "object" ? block.inputs : {}).forEach((children) => {
      visitBlocks(children, visitor);
    });
  }
}

function findNthBlockByKind(blocks, kind, index = 0) {
  const targetKind = String(kind ?? "");
  const targetIndex = Number(index ?? 0);
  let cursor = 0;
  let hit = null;
  visitBlocks(blocks, (block) => {
    if (hit) return;
    if (String(block?.kind ?? "") !== targetKind) return;
    if (cursor === targetIndex) {
      hit = block;
      return;
    }
    cursor += 1;
  });
  return hit;
}

async function readJson(filePath) {
  return JSON.parse(await fs.readFile(filePath, "utf8"));
}

async function main() {
  const args = process.argv.slice(2);
  const update = args.includes("--update");
  const packArg = args.find((item) => !item.startsWith("--")) ?? "pack/block_editor_screen_rpg_smoke_v1";
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const rootDir = path.resolve(__dirname, "..");
  const packDir = path.resolve(rootDir, packArg);

  const screenUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "screens", "block_editor.js"),
  ).href;
  const canonUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "wasm_canon_runtime.js"),
  ).href;
  const paletteUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "rpgbox_palette.js"),
  ).href;
  const seamgrimPaletteUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "seamgrim_palette.js"),
  ).href;
  const screenMod = await import(screenUrl);
  const canonMod = await import(canonUrl);
  const [paletteMod, seamgrimPaletteMod] = await Promise.all([
    import(paletteUrl),
    import(seamgrimPaletteUrl),
  ]);

  const fixture = await readJson(path.join(packDir, "fixtures", "screen.detjson"));
  const sourceText = await fs.readFile(path.join(packDir, String(fixture.source_ddn ?? "fixtures/source.ddn")), "utf8");
  const wasmBytes = await fs.readFile(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm", "ddonirang_tool_bg.wasm"),
  );
  const canon = await canonMod.createWasmCanon({ cacheBust: 0, initInput: wasmBytes });

  const elements = new Map([
    ["#block-editor-title", createStubNode("span")],
    ["#block-editor-mode", createStubNode("select")],
    ["#block-editor-result", createStubNode("div")],
    ["#block-editor-summary", createStubNode("div")],
    ["#block-ddn-preview", createStubNode("textarea")],
    ["#block-palette", createStubNode("div")],
    ["#block-canvas", createStubNode("div")],
    ["#btn-back-from-block-editor", createStubNode("button")],
    ["#btn-block-to-text", createStubNode("button")],
    ["#btn-block-run", createStubNode("button")],
    ["#btn-advanced-block-editor", createStubNode("button")],
  ]);
  const root = createRoot(elements);
  const textModeCalls = [];
  const runCalls = [];
  const screen = new screenMod.BlockEditorScreen({
    root,
    canon,
    onTextMode(ddnText, meta) {
      textModeCalls.push({
        ddnText: String(ddnText ?? "").trimEnd(),
        meta,
      });
    },
    onRun(ddnText, meta) {
      runCalls.push({
        ddnText: String(ddnText ?? "").trimEnd(),
        meta,
      });
    },
  });
  screen.init();
  await screen.loadSource(sourceText, {
    title: String(fixture.title ?? "RPG Block Screen"),
    mode: String(fixture.mode ?? "rpg"),
  });
  const exprEdits = Array.isArray(fixture.expr_edits) ? fixture.expr_edits : [];
  for (const edit of exprEdits) {
    const target = findNthBlockByKind(
      screen.currentBlocks,
      String(edit?.target_kind ?? ""),
      Number(edit?.target_index ?? 0),
    );
    if (!target) {
      throw new Error(`expr edit target missing kind=${String(edit?.target_kind ?? "")} index=${String(edit?.target_index ?? 0)}`);
    }
    const ok = screen.engine.updateExprText(
      String(target.id ?? ""),
      String(edit?.expr_key ?? "expr"),
      String(edit?.value ?? ""),
    );
    if (!ok) {
      throw new Error(`expr edit apply failed id=${String(target.id ?? "")} key=${String(edit?.expr_key ?? "expr")}`);
    }
  }
  const exprNodeEdits = Array.isArray(fixture.expr_node_edits) ? fixture.expr_node_edits : [];
  for (const edit of exprNodeEdits) {
    const target = findNthBlockByKind(
      screen.currentBlocks,
      String(edit?.target_kind ?? ""),
      Number(edit?.target_index ?? 0),
    );
    if (!target) {
      throw new Error(
        `expr node edit target missing kind=${String(edit?.target_kind ?? "")} index=${String(edit?.target_index ?? 0)}`,
      );
    }
    const ok = screen.engine.updateExprNodeField(
      String(target.id ?? ""),
      String(edit?.expr_key ?? "expr"),
      Array.isArray(edit?.expr_path) ? edit.expr_path : [],
      String(edit?.field_key ?? ""),
      String(edit?.value ?? ""),
    );
    if (!ok) {
      throw new Error(
        `expr node edit apply failed id=${String(target.id ?? "")} key=${String(edit?.expr_key ?? "expr")}`,
      );
    }
  }
  const mode = String(fixture.mode ?? "rpg");
  const appendKind = String(
    fixture.append_block_kind ?? (mode === "rpg" ? "alrimsi_send" : "show"),
  );
  const appendPalette = mode === "rpg" ? paletteMod.RPGBOX_PALETTE : seamgrimPaletteMod.SEAMGRIM_PALETTE;
  const appendDef = appendPalette.categories
    .flatMap((item) => (Array.isArray(item?.blocks) ? item.blocks : []))
    .find((item) => String(item?.kind ?? "") === appendKind);
  if (appendDef) {
    screen.engine.appendPaletteBlock(appendDef);
  }
  elements.get("#btn-block-to-text")?.click();
  elements.get("#btn-block-run")?.click();

  const output = {
    schema: "seamgrim.web.block_editor_screen_smoke.v1",
    screen_summary: screen.getSummary(),
    expr_summary: summarizeBlockExprs(screen.currentBlocks),
    result_text: String(elements.get("#block-editor-result")?.textContent ?? ""),
    summary_text: String(elements.get("#block-editor-summary")?.textContent ?? ""),
    generated_preview: String(elements.get("#block-ddn-preview")?.value ?? "").trimEnd(),
    palette_children: Array.isArray(elements.get("#block-palette")?.children)
      ? elements.get("#block-palette").children.length
      : 0,
    canvas_children: Array.isArray(elements.get("#block-canvas")?.children)
      ? elements.get("#block-canvas").children.length
      : 0,
    text_mode_calls: textModeCalls,
    run_calls: runCalls,
  };

  const expectedPath = path.join(packDir, "expected", "block_editor_screen.detjson");
  const actualText = formatJson(output);
  if (update) {
    await fs.mkdir(path.dirname(expectedPath), { recursive: true });
    await fs.writeFile(expectedPath, actualText, "utf8");
    console.log(`block editor screen smoke updated: ${path.relative(rootDir, expectedPath)}`);
    return;
  }
  const expectedText = await fs.readFile(expectedPath, "utf8");
  if (expectedText !== actualText) {
    console.error(`block editor screen smoke mismatch: ${path.relative(rootDir, expectedPath)}`);
    process.exit(1);
  }
  console.log(`block editor screen smoke ok: ${path.relative(rootDir, packDir)}`);
}

await main();
