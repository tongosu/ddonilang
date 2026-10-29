#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

function assert(condition, message) { if (!condition) throw new Error(message); }
function mime(file) { return file.endsWith(".js") || file.endsWith(".mjs") ? "application/javascript" : file.endsWith(".json") || file.endsWith(".detjson") ? "application/json" : "application/octet-stream"; }
async function serve(root) {
  const base = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      if (url.pathname === "/") { res.writeHead(200, { "content-type": "text/html" }); res.end("<!doctype html><canvas id='console'></canvas><canvas id='grid'></canvas>"); return; }
      const file = path.resolve(base, decodeURIComponent(url.pathname).replace(/^\/+/, ""));
      if (file !== base && !file.startsWith(`${base}${path.sep}`)) { res.writeHead(403).end(); return; }
      const bytes = await fs.readFile(file); res.writeHead(200, { "content-type": mime(file) }); res.end(bytes);
    } catch (_) { res.writeHead(404).end(); }
  });
  await new Promise((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", resolve); });
  return { server, url: `http://127.0.0.1:${server.address().port}` };
}

const root = process.cwd();
const { server, url } = await serve(root);
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage();
  await page.goto(url);
  const result = await page.evaluate(async () => {
    const state = await fetch("/pack/seamgrim_console_grid_grid2d_canonical_view_payload_v1/fixtures/authoritative_state.detjson").then((r) => r.json());
    const stateMod = await import("/solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js");
    const payloadMod = await import("/solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js");
    const cellMod = await import("/solutions/seamgrim_ui_mvp/ui/runtime/cell_grid_renderer.js");
    const payloads = payloadMod.buildCanonicalViewPayloadSet(stateMod.extractCanonicalViewAuthorityInputsFromState(state));
    const consoleGrid = cellMod.canonicalViewPayloadToCellGrid(payloads.console_grid);
    const grid2d = cellMod.canonicalViewPayloadToCellGrid(payloads.grid2d, { gridState: state.view_meta.grid2d, actorList: state.view_meta.grid2d.actors });
    const consoleSummary = cellMod.renderCellGridCanvas2d(document.querySelector("#console"), consoleGrid);
    const gridSummary = cellMod.renderCellGridCanvas2d(document.querySelector("#grid"), grid2d, { gridState: state.view_meta.grid2d, actorList: state.view_meta.grid2d.actors });
    return {
      statuses: [payloads.console_grid.status, payloads.grid2d.status],
      console_cells: consoleGrid.cells.length,
      grid_cells: grid2d.cells.length,
      actor_ids: grid2d.cells.filter((cell) => cell.actor_id).map((cell) => cell.actor_id),
      console_summary: consoleSummary,
      grid_summary: gridSummary,
      payload_bytes: [payloads.console_grid, payloads.grid2d].map(payloadMod.stableSemanticJson),
    };
  });
  assert(result.statuses.every((row) => row === "available"), "browser payload unavailable");
  assert(result.console_cells === 4, "browser console cell count drift");
  assert(result.grid_cells === 8, "browser grid cell count drift");
  assert(result.actor_ids.join(",") === "player,goal", "browser actor identity/order drift");
  assert(result.payload_bytes.every((row) => row.includes("seamgrim.semantic_view_payload.v1_candidate")), "browser payload schema missing");
  console.log(JSON.stringify({ status: "PASS", ...result }));
} finally {
  await browser.close();
  await new Promise((resolve) => server.close(resolve));
}
