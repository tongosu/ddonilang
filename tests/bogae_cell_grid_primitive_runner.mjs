#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function createFakeCanvas() {
  const calls = {
    fillRect: [],
    strokeRect: [],
    fillText: [],
    clearRect: 0,
  };
  const ctx = {
    fillStyle: "",
    strokeStyle: "",
    font: "",
    textBaseline: "",
    lineWidth: 0,
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
      calls.fillText.push({ text: String(text ?? ""), x: Number(x), y: Number(y), font: String(this.font ?? "") });
    },
    beginPath() {},
    moveTo() {},
    lineTo() {},
    stroke() {},
    measureText(text) {
      return { width: String(text ?? "").length * 8 };
    },
  };
  return {
    width: 0,
    height: 0,
    clientWidth: 240,
    clientHeight: 120,
    calls,
    getContext(type) {
      return type === "2d" ? ctx : null;
    },
    getBoundingClientRect() {
      return { left: 0, top: 0, width: this.width || this.clientWidth || 1, height: this.height || this.clientHeight || 1 };
    },
    addEventListener() {},
    removeEventListener() {},
  };
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const runtimeDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "runtime");
  const cellGridMod = await import(pathToFileURL(path.join(runtimeDir, "cell_grid_renderer.js")).href);
  const gridRendererMod = await import(pathToFileURL(path.join(runtimeDir, "nurimaker_grid_renderer.js")).href);
  const viewFamily = await import(pathToFileURL(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "view_family_contract.js")).href);

  const consoleGrid = cellGridMod.consoleLinesToCellGrid(["P#G"]);
  assert(consoleGrid.schema === "seamgrim.cell_grid_render.v1", "console cell-grid schema mismatch");
  assert(consoleGrid.family === "console", "console cell-grid family mismatch");
  assert(consoleGrid.role === "output", "console cell-grid role mismatch");
  assert(consoleGrid.renderer === "cell_grid", "console renderer mismatch");
  assert(consoleGrid.cells.some((cell) => cell.text === "P"), "console cell P missing");

  const gridFixture = JSON.parse(
    await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "samples", "grid2d_v0", "maze_probe.grid.detjson"), "utf8"),
  );
  const tileTextMap = { "0": " ", "1": "#", "2": ".", "3": ".", "4": "*" };
  const gridCell = cellGridMod.grid2dFixtureToCellGrid(gridFixture, { tileTextMap });
  assert(gridCell.schema === "seamgrim.cell_grid_render.v1", "grid2d cell-grid schema mismatch");
  assert(gridCell.family === "grid2d", "grid2d cell-grid family mismatch");
  assert(gridCell.role === "world_grid", "grid2d cell-grid role mismatch");
  assert(gridCell.renderer === "cell_grid", "grid2d renderer mismatch");
  assert(gridCell.cells.some((cell) => cell.source === "tile" && cell.text === "#"), "grid2d wall tile text missing");
  assert(gridCell.cells.some((cell) => cell.source === "actor" && cell.actor_id === "player" && cell.text === "P"), "grid2d player actor text missing");
  assert(gridCell.cells.some((cell) => cell.source === "actor" && cell.actor_id === "goal" && cell.text === "G"), "grid2d goal actor text missing");

  const consoleCanvas = createFakeCanvas();
  cellGridMod.renderCellGridCanvas2d(consoleCanvas, consoleGrid);
  assert(consoleCanvas.calls.fillText.some((call) => call.text === "P"), "console render should draw P");

  const gridCanvas = createFakeCanvas();
  cellGridMod.renderCellGridCanvas2d(gridCanvas, gridCell, {
    gridState: gridRendererMod.normalizeGridState(gridFixture.grid),
    actorList: gridFixture.actors,
  });
  assert(gridCanvas.calls.fillRect.length > 0, "grid2d render should draw fill rects");
  assert(gridCanvas.calls.fillText.some((call) => call.text === "P"), "grid2d render should draw player label");

  for (const listName of ["VIEW_FAMILY_PRIORITY", "SPATIAL_VIEW_FAMILIES", "LESSON_PREVIEW_FAMILIES"]) {
    const values = viewFamily[listName] ?? [];
    assert(!values.includes("cell_grid"), `${listName} must not contain cell_grid`);
  }

  const renderer = new gridRendererMod.NurimakerGridRenderer(createFakeCanvas(), {
    gridState: gridFixture.grid,
    actorList: gridFixture.actors,
  });
  const summary = renderer.render();
  assert(summary.schema === "seamgrim.nurimaker.grid_render.v1", "Nurimaker summary schema changed");
  assert(summary.draw_calls?.tiles === 30, "Nurimaker tile draw count changed");
  assert(summary.draw_calls?.actors === 2, "Nurimaker actor draw count changed");
  assert(summary.cell_grid_schema === "seamgrim.cell_grid_render.v1", "Nurimaker cell-grid schema metadata missing");
  assert(summary.cell_grid_family === "grid2d", "Nurimaker cell-grid family metadata missing");
  assert(summary.cell_grid_renderer === "cell_grid", "Nurimaker cell-grid renderer metadata missing");
  assert(summary.cell_grid_sample_cells.length <= 12, "Nurimaker cell-grid sample should be capped");
  assert(renderer.getLastCellGrid().cells.length === 32, "Nurimaker full cell-grid debug cells missing");

  console.log("[bogae-cell-grid-primitive] ok");
}

await main();
