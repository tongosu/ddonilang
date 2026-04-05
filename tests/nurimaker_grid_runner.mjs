#!/usr/bin/env node

import crypto from "node:crypto";
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

async function readJson(filePath) {
  return JSON.parse(await fs.readFile(filePath, "utf8"));
}

function makeEventTarget() {
  const handlers = new Map();
  return {
    addEventListener(type, callback) {
      if (!handlers.has(type)) handlers.set(type, new Set());
      handlers.get(type).add(callback);
    },
    removeEventListener(type, callback) {
      handlers.get(type)?.delete(callback);
    },
    dispatch(type, event) {
      handlers.get(type)?.forEach((callback) => {
        callback(event);
      });
    },
  };
}

function createFakeCanvas() {
  const target = makeEventTarget();
  const calls = {
    clearRect: 0,
    fillRect: [],
    strokeRect: [],
    fillText: [],
  };
  const ctx = {
    fillStyle: "",
    strokeStyle: "",
    font: "",
    clearRect() {
      calls.clearRect += 1;
    },
    fillRect(x, y, w, h) {
      calls.fillRect.push({
        fill: String(this.fillStyle ?? ""),
        x: Number(x),
        y: Number(y),
        w: Number(w),
        h: Number(h),
      });
    },
    strokeRect(x, y, w, h) {
      calls.strokeRect.push({
        stroke: String(this.strokeStyle ?? ""),
        x: Number(x),
        y: Number(y),
        w: Number(w),
        h: Number(h),
      });
    },
    fillText(text, x, y) {
      calls.fillText.push({
        text: String(text ?? ""),
        fill: String(this.fillStyle ?? ""),
        x: Number(x),
        y: Number(y),
        font: String(this.font ?? ""),
      });
    },
  };
  return {
    width: 0,
    height: 0,
    calls,
    getContext(type) {
      return type === "2d" ? ctx : null;
    },
    getBoundingClientRect() {
      return {
        left: 0,
        top: 0,
        width: this.width || 1,
        height: this.height || 1,
      };
    },
    addEventListener(type, callback) {
      target.addEventListener(type, callback);
    },
    removeEventListener(type, callback) {
      target.removeEventListener(type, callback);
    },
    dispatchClick(event) {
      target.dispatch("click", event);
    },
  };
}

function canvasDigest(calls) {
  const payload = formatJson(calls);
  return `sha256:${crypto.createHash("sha256").update(payload).digest("hex")}`;
}

async function main() {
  const args = process.argv.slice(2);
  const update = args.includes("--update");
  const packArg = args.find((item) => !item.startsWith("--")) ?? "pack/nurimaker_grid_render_smoke_v1";
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const rootDir = path.resolve(__dirname, "..");
  const packDir = path.resolve(rootDir, packArg);

  const rendererUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "nurimaker_grid_renderer.js"),
  ).href;
  const screenUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "screens", "rpg_box.js"),
  ).href;
  const rendererMod = await import(rendererUrl);
  const screenMod = await import(screenUrl);

  if (typeof rendererMod.NurimakerGridRenderer !== "function") {
    throw new Error("NurimakerGridRenderer export 누락");
  }
  if (typeof screenMod.RpgBoxScreen !== "function") {
    throw new Error("RpgBoxScreen export 누락");
  }

  const fixture = await readJson(path.join(packDir, "fixtures", "grid.detjson"));
  const expectedPath = path.join(packDir, "expected", "grid_render.detjson");

  const rendererCanvas = createFakeCanvas();
  const renderer = new rendererMod.NurimakerGridRenderer(rendererCanvas, {
    gridState: fixture.grid,
    actorList: fixture.actors,
  });
  const rendererClickHits = [];
  renderer.onCellClick((hit) => {
    rendererClickHits.push({ row: hit.row, col: hit.col });
  });
  const initialSummary = renderer.render();
  const clickRow = Number(fixture?.click?.row ?? 0);
  const clickCol = Number(fixture?.click?.col ?? 0);
  const tileSize = Number(initialSummary?.tile_size ?? 24);
  rendererCanvas.dispatchClick({
    clientX: clickCol * tileSize + tileSize / 2,
    clientY: clickRow * tileSize + tileSize / 2,
  });

  const screenCanvas = createFakeCanvas();
  const editorEl = { value: "" };
  const statusEl = { textContent: "" };
  const sceneSelect = {
    value: "",
    options: [],
    addEventListener() {},
  };
  const screenPaintEvents = [];
  const screen = new screenMod.RpgBoxScreen({
    canvas: screenCanvas,
    editorEl,
    statusEl,
    sceneSelect,
    runButton: { addEventListener() {} },
    stopButton: { addEventListener() {} },
    resetButton: { addEventListener() {} },
    onCellPaint(payload) {
      screenPaintEvents.push({
        row: Number(payload?.row ?? 0),
        col: Number(payload?.col ?? 0),
        kind: Number(payload?.kind ?? 0),
      });
    },
  });
  screen.setSceneOptions(fixture.scene_options ?? [], fixture.scene_id ?? "");
  screen.setSelectedTileKind(fixture.selected_tile_kind ?? 0);
  screen.loadScene({
    gridState: fixture.grid,
    actors: fixture.actors,
    sourceText: fixture.source_text ?? "",
    statusText: "초기 격자 로드",
  });
  screenCanvas.dispatchClick({
    clientX: clickCol * tileSize + tileSize / 2,
    clientY: clickRow * tileSize + tileSize / 2,
  });

  const finalGridState = screen.getGridState();
  const summary = {
    schema: "seamgrim.web.nurimaker_grid_render_smoke.v1",
    fixture_schema: String(fixture?.schema ?? ""),
    renderer: {
      summary: initialSummary,
      click_hits: rendererClickHits,
      canvas_calls: {
        clear_rect: rendererCanvas.calls.clearRect,
        fill_rect_count: rendererCanvas.calls.fillRect.length,
        stroke_rect_count: rendererCanvas.calls.strokeRect.length,
        fill_text: rendererCanvas.calls.fillText.map((item) => item.text),
        digest: canvasDigest(rendererCanvas.calls),
      },
    },
    screen: {
      scene_id: screen.getSceneValue(),
      source_line_count: String(screen.getSourceText()).split(/\r?\n/).length,
      selected_tile_kind: Number(fixture?.selected_tile_kind ?? 0),
      status_text: String(statusEl.textContent ?? ""),
      click_log: screen.clickLog,
      paint_events: screenPaintEvents,
      painted_value: Number(finalGridState?.tiles?.[clickRow]?.[clickCol] ?? -1),
      non_zero_cells: finalGridState?.tiles
        ? finalGridState.tiles.flatMap((row, rowIndex) =>
            row.flatMap((value, colIndex) =>
              Number(value) === 0 ? [] : [{ row: rowIndex, col: colIndex, kind: Number(value) }],
            ),
          )
        : [],
      render_summary: screen.getRenderSummary(),
      canvas_calls: {
        clear_rect: screenCanvas.calls.clearRect,
        fill_rect_count: screenCanvas.calls.fillRect.length,
        stroke_rect_count: screenCanvas.calls.strokeRect.length,
        fill_text: screenCanvas.calls.fillText.map((item) => item.text),
        digest: canvasDigest(screenCanvas.calls),
      },
    },
  };

  const actualText = formatJson(summary);
  if (update) {
    await fs.mkdir(path.dirname(expectedPath), { recursive: true });
    await fs.writeFile(expectedPath, actualText, "utf8");
    console.log(`nurimaker grid smoke updated: ${path.relative(rootDir, expectedPath)}`);
    return;
  }

  const expectedText = await fs.readFile(expectedPath, "utf8");
  if (expectedText !== actualText) {
    console.error(`nurimaker grid smoke mismatch: ${path.relative(rootDir, expectedPath)}`);
    process.exit(1);
  }
  console.log(`nurimaker grid smoke ok: ${path.relative(rootDir, packDir)}`);
}

await main();
