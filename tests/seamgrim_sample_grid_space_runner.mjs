#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function loadStateForSample({ uiDir, wasmModule, wrapper, sampleFile, transformSource = null }) {
  const source = await fs.readFile(path.join(uiDir, "samples", sampleFile), "utf8");
  const executableSource = typeof transformSource === "function" ? transformSource(source) : source;
  const vm = new wasmModule.DdnWasmVm(executableSource);
  const client = new wrapper.DdnWasmVmClient(vm);
  try {
    return client.stepOneParsed();
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

function deriveMainVisual({ runtimeState, runMod, state }) {
  const outputRows = runtimeState.extractObservationOutputRowsFromState(state);
  const outputLog = runtimeState.extractObservationOutputLogFromState(state);
  const outputLines = runtimeState.extractObservationOutputLinesFromState(state);
  const observation = runtimeState.extractObservationChannelsFromState(state);
  const strictViews = runtimeState.extractStructuredViewsFromState(state, {
    preferPatch: false,
    allowObservationOutputFallback: false,
  });
  const views = typeof runMod.mergeRuntimeViewsWithObservationOutputFallback === "function"
    ? runMod.mergeRuntimeViewsWithObservationOutputFallback(state, strictViews)
    : strictViews;
  return runMod.resolveRunMainVisualMode({
    views,
    observation,
    outputRows,
    outputLog,
    outputLines,
  });
}

function createFakeCanvas() {
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
      calls.fillRect.push({ fill: String(this.fillStyle ?? ""), x: Number(x), y: Number(y), w: Number(w), h: Number(h) });
    },
    strokeRect(x, y, w, h) {
      calls.strokeRect.push({ stroke: String(this.strokeStyle ?? ""), x: Number(x), y: Number(y), w: Number(w), h: Number(h) });
    },
    fillText(text, x, y) {
      calls.fillText.push({ text: String(text ?? ""), fill: String(this.fillStyle ?? ""), x: Number(x), y: Number(y), font: String(this.font ?? "") });
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
    addEventListener() {},
    removeEventListener() {},
  };
}

function collectSpaceItems(space2d) {
  return [
    ...(Array.isArray(space2d?.shapes) ? space2d.shapes : []),
    ...(Array.isArray(space2d?.drawlist) ? space2d.drawlist : []),
  ];
}

function normalizeNumber(value) {
  const num = Number(value);
  return Number.isFinite(num) ? Number(num.toFixed(6)) : null;
}

function normalizeSpaceItemForCompare(item) {
  const kind = String(item?.kind ?? "").toLowerCase();
  if (kind === "line") {
    return {
      kind,
      x1: normalizeNumber(item?.x1),
      y1: normalizeNumber(item?.y1),
      x2: normalizeNumber(item?.x2),
      y2: normalizeNumber(item?.y2),
    };
  }
  if (kind === "circle") {
    return {
      kind,
      x: normalizeNumber(item?.x),
      y: normalizeNumber(item?.y),
      r: normalizeNumber(item?.r),
    };
  }
  if (kind === "point") {
    return {
      kind,
      x: normalizeNumber(item?.x),
      y: normalizeNumber(item?.y),
      size: normalizeNumber(item?.size),
    };
  }
  return { kind };
}

function normalizeSpaceItemsForCompare(items) {
  return items.map(normalizeSpaceItemForCompare);
}

async function main() {
  const root = process.cwd();
  const uiDir = path.join(root, "solutions", "seamgrim_ui_mvp");
  const uiRuntimeDir = path.join(uiDir, "ui");
  const wasmModule = await import(pathToFileURL(path.join(uiRuntimeDir, "wasm", "ddonirang_tool.js")).href);
  const wrapper = await import(pathToFileURL(path.join(uiRuntimeDir, "wasm_ddn_wrapper.js")).href);
  const runtimeState = await import(pathToFileURL(path.join(uiRuntimeDir, "seamgrim_runtime_state.js")).href);
  const ddnPreprocess = await import(pathToFileURL(path.join(uiRuntimeDir, "runtime", "ddn_preprocess.js")).href);
  const runMod = await import(pathToFileURL(path.join(uiRuntimeDir, "screens", "run.js")).href);
  const gridRendererMod = await import(
    pathToFileURL(path.join(uiRuntimeDir, "runtime", "nurimaker_grid_renderer.js")).href
  );
  const wasmBytes = await fs.readFile(path.join(uiRuntimeDir, "wasm", "ddonirang_tool_bg.wasm"));
  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }

  const consoleGridState = await loadStateForSample({
    uiDir,
    wasmModule,
    wrapper,
    sampleFile: "15_console_grid_maze_probe.ddn",
  });
  const consoleGridVisual = deriveMainVisual({ runtimeState, runMod, state: consoleGridState });
  assert(consoleGridVisual.mode === "console-grid", `console-grid sample visual mode mismatch: ${consoleGridVisual.mode}`);
  assert(
    Array.isArray(consoleGridVisual.consoleLinesForGrid) && consoleGridVisual.consoleLinesForGrid.some((line) => String(line).includes("P")),
    "console-grid sample should render player marker P",
  );
  assert(
    consoleGridVisual.consoleLinesForGrid.some((line) => String(line).includes("G")),
    "console-grid sample should render goal marker G",
  );

  const gridFixture = JSON.parse(
    await fs.readFile(path.join(uiDir, "samples", "grid2d_v0", "maze_probe.grid.detjson"), "utf8"),
  );
  const gridCanvas = createFakeCanvas();
  const gridRenderer = new gridRendererMod.NurimakerGridRenderer(gridCanvas, {
    gridState: gridFixture.grid,
    actorList: gridFixture.actors,
  });
  const gridSummary = gridRenderer.render();
  assert(gridSummary.schema === "seamgrim.nurimaker.grid_render.v1", "grid2d renderer summary schema mismatch");
  assert(gridSummary.grid_w === 6 && gridSummary.grid_h === 5, "grid2d renderer should preserve fixture dimensions");
  assert(gridSummary.draw_calls?.tiles === 30, "grid2d renderer should draw every tile");
  assert(gridSummary.draw_calls?.actors === 2, "grid2d renderer should draw player and goal actors");
  assert(
    gridSummary.actor_cells.some((actor) => actor.id === "player" && actor.row === 1 && actor.col === 1),
    "grid2d renderer player actor cell missing",
  );
  assert(
    gridSummary.actor_cells.some((actor) => actor.id === "goal" && actor.row === 3 && actor.col === 4),
    "grid2d renderer goal actor cell missing",
  );

  const spaceState = await loadStateForSample({
    uiDir,
    wasmModule,
    wrapper,
    sampleFile: "16_space2d_bounce_probe.ddn",
  });
  const spaceOutputLog = runtimeState.extractObservationOutputLogFromState(spaceState);
  assert(
    spaceOutputLog.every((entry) => !["space2d", "space2d.shape", "line", "circle", "point", "x1", "y1"].includes(String(entry?.text ?? "").toLowerCase())),
    "space2d structured drawing tokens must not leak into output log",
  );
  const spaceVisual = deriveMainVisual({ runtimeState, runMod, state: spaceState });
  assert(spaceVisual.mode === "space2d", `space2d sample visual mode mismatch: ${spaceVisual.mode}`);
  const spaceItems = collectSpaceItems(spaceVisual.space2d);
  assert(spaceItems.length > 0, "space2d sample draw items missing");
  assert(
    spaceItems.some((shape) => String(shape?.kind ?? "").toLowerCase() === "circle"),
    "space2d sample should include a circle",
  );
  assert(
    spaceItems.some((shape) => String(shape?.kind ?? "").toLowerCase() === "line"),
    "space2d sample should include boundary lines",
  );
  const verticalWalls = spaceItems.filter(
    (shape) =>
      String(shape?.kind ?? "").toLowerCase() === "line" &&
      Number.isFinite(Number(shape?.x1)) &&
      Number(shape.x1) === Number(shape.x2),
  );
  assert(
    verticalWalls.some((shape) => Math.abs(Number(shape.x1) + 1.5) < 0.0001),
    "space2d sample should draw left wall at x=-1.5",
  );
  assert(
    verticalWalls.some((shape) => Math.abs(Number(shape.x1) - 1.5) < 0.0001),
    "space2d sample should draw right wall at x=1.5",
  );
  const ball = spaceItems.find((shape) => String(shape?.kind ?? "").toLowerCase() === "circle");
  const ballX = Number(ball?.x);
  const ballY = Number(ball?.y);
  const ballR = Number(ball?.r);
  assert(
    Number.isFinite(ballX) && Number.isFinite(ballY) && Number.isFinite(ballR),
    "space2d sample ball coordinates missing",
  );
  assert(
    ballX - ballR >= -1.5001 && ballX + ballR <= 1.5001 && ballY - ballR >= -0.9501 && ballY + ballR <= 0.9501,
    "space2d sample ball should bounce inside the drawn rectangle",
  );

  const preprocessedSpaceState = await loadStateForSample({
    uiDir,
    wasmModule,
    wrapper,
    sampleFile: "16_space2d_bounce_probe.ddn",
    transformSource: (source) => ddnPreprocess.preprocessDdnText(source).bodyText,
  });
  const preprocessedSpaceVisual = deriveMainVisual({ runtimeState, runMod, state: preprocessedSpaceState });
  assert(preprocessedSpaceVisual.mode === "space2d", `preprocessed space2d visual mode mismatch: ${preprocessedSpaceVisual.mode}`);
  const preprocessedItems = collectSpaceItems(preprocessedSpaceVisual.space2d);
  assert(
    JSON.stringify(normalizeSpaceItemsForCompare(preprocessedItems)) === JSON.stringify(normalizeSpaceItemsForCompare(spaceItems)),
    "preprocessed space2d sample should preserve all raw draw item coordinates",
  );
  const preprocessedVerticalWalls = preprocessedItems.filter(
    (shape) =>
      String(shape?.kind ?? "").toLowerCase() === "line" &&
      Number.isFinite(Number(shape?.x1)) &&
      Number(shape.x1) === Number(shape.x2),
  );
  assert(
    preprocessedVerticalWalls.some((shape) => Math.abs(Number(shape.x1) + 1.5) < 0.0001),
    "preprocessed space2d sample should preserve left wall at x=-1.5",
  );

  console.log("seamgrim sample console-grid/grid2d/space2d runner ok");
}

main().catch((error) => {
  console.error(error?.stack || String(error));
  process.exit(1);
});
