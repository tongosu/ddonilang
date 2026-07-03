#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_run_history_comparison_rail: ok";
const UNSAFE_BROWSER_PORTS = new Set([
  1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
  101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161,
  179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563,
  587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
  6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
]);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) throw new Error(`missing file: ${file}`);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".toml") || file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  if (file.endsWith(".md")) return "text/markdown; charset=utf-8";
  return "application/octet-stream";
}

function createServer(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      if (url.pathname === "/favicon.ico") {
        res.writeHead(204, { "cache-control": "no-store" });
        res.end();
        return;
      }
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/index.html" : url.pathname);
      const rel = rawPath.replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, rel);
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, {
        "content-type": mimeType(file),
        "cache-control": "no-store",
        "access-control-allow-origin": "*",
      });
      res.end(bytes);
    } catch (_) {
      res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      res.end("not found");
    }
  });
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("failed to bind static server"));
        return;
      }
      if (UNSAFE_BROWSER_PORTS.has(address.port)) {
        server.close(() => {
          createServer(root).then(resolve, reject);
        });
        return;
      }
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d)\.json$/i.test(pathname)
    ) return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:text|maegim_control)\.(?:md|json)$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

async function readRail(page) {
  return page.evaluate(() => {
    const rail = document.querySelector("[data-run-history-comparison-rail]");
    const text = (sel) => rail?.querySelector?.(sel)?.textContent?.trim() ?? "";
    const value = (sel) => rail?.querySelector?.(sel)?.dataset?.value ?? "";
    const state = (sel) => rail?.querySelector?.(sel)?.dataset?.state ?? "";
    return {
      railCount: document.querySelectorAll("[data-run-history-comparison-rail]").length,
      schema: rail?.dataset?.schema ?? "",
      state: rail?.dataset?.state ?? "",
      count: text("[data-run-history-count]"),
      countValue: value("[data-run-history-count]"),
      visible: text("[data-run-history-visible]"),
      visibleValue: value("[data-run-history-visible]"),
      active: text("[data-run-history-active]"),
      activeValue: value("[data-run-history-active]"),
      activeState: state("[data-run-history-active]"),
      solo: text("[data-run-history-solo]"),
      soloValue: value("[data-run-history-solo]"),
      soloState: state("[data-run-history-solo]"),
      model: window.__SEAMGRIM_RUN_HISTORY_COMPARISON_RAIL__ ?? null,
    };
  });
}

async function readDebugState(page) {
  return page.evaluate(() => ({
    railModel: window.__SEAMGRIM_RUN_HISTORY_COMPARISON_RAIL__ ?? null,
    railText: document.querySelector("[data-run-history-comparison-rail]")?.textContent?.trim() ?? "",
    rowCount: document.querySelectorAll("[data-run-row]").length,
  }));
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");

  for (const rel of ["index.html", "app.js", "styles.css", "screens/browse.js", "screens/run.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
      viewport: { width: 1360, height: 860 },
      locale: "ko-KR",
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        if (String(msg.text() ?? "").includes("Failed to load resource")) return;
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400) {
        if (res.status() === 404 && isAllowedFallback404(res.url())) return;
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto("about:blank");
    await page.evaluate(async ({ moduleUrl }) => {
      const { RunScreen } = await import(moduleUrl);
      document.body.innerHTML = `
        <section id="screen-run">
          <div id="dotbogi-graph">
            <div id="run-manager-panel-anchor"></div>
          </div>
        </section>
      `;
      const root = document.querySelector("#screen-run");
      const screen = new RunScreen({ root, wasmState: { fpsLimit: 30, dtMax: 0.1 } });
      screen.graphPanelEl = root.querySelector("#dotbogi-graph");
      screen.dotbogi = {
        setPersistedGraph() {},
        setBaseSeriesDisplay() {},
        setOverlaySeries() {},
      };
      screen.syncDockRangeLabels = () => {};
      screen.ensureRunManagerUi();
      screen.runManagerPanelEl = root.querySelector("#run-manager-panel");
      screen.runManagerListEl = root.querySelector("#run-manager-list");
      screen.runHistoryComparisonRailEl = root.querySelector("[data-run-history-comparison-rail]");
      screen.runHistoryCountEl = root.querySelector("[data-run-history-count]");
      screen.runHistoryVisibleEl = root.querySelector("[data-run-history-visible]");
      screen.runHistoryActiveEl = root.querySelector("[data-run-history-active]");
      screen.runHistorySoloEl = root.querySelector("[data-run-history-solo]");
      screen.overlayRuns = [
        screen.normalizeRunManagerRun({
          id: "history-visible",
          label: "기준 run",
          visible: true,
          layer_index: 0,
          source: { kind: "ddn", lessonId: "rep_cs_linear_search_timeline_v1", text: "" },
          inputs: { controls: {} },
          graph: {
            schema: "seamgrim.graph.v1",
            meta: {
              graph_kind: "line",
              axis_x_kind: "time",
              axis_x_unit: "tick",
              axis_y_kind: "value",
              axis_y_unit: "count",
              source_input_hash: "input-visible",
              result_hash: "result-visible",
            },
            axis: { x_min: 0, x_max: 2, y_min: 0, y_max: 4 },
            series: [{
              id: "value",
              points: [{ x: 0, y: 1 }, { x: 1, y: 2 }, { x: 2, y: 3 }],
            }],
          },
          hash: { input: "input-visible", result: "result-visible" },
        }, 0),
        screen.normalizeRunManagerRun({
          id: "history-hidden",
          label: "숨김 run",
          visible: false,
          layer_index: 1,
          source: { kind: "ddn", lessonId: "rep_cs_linear_search_timeline_v1", text: "" },
          inputs: { controls: {} },
          graph: {
            schema: "seamgrim.graph.v1",
            meta: {
              graph_kind: "line",
              axis_x_kind: "time",
              axis_x_unit: "tick",
              axis_y_kind: "value",
              axis_y_unit: "count",
              source_input_hash: "input-hidden",
              result_hash: "result-hidden",
            },
            axis: { x_min: 0, x_max: 2, y_min: 0, y_max: 4 },
            series: [{
              id: "value",
              points: [{ x: 0, y: 4 }, { x: 1, y: 5 }, { x: 2, y: 6 }],
            }],
          },
          hash: { input: "input-hidden", result: "result-hidden" },
        }, 1),
      ];
      screen.activeOverlayRunId = "";
      screen.soloOverlayRunId = "";
      screen.hoverOverlayRunId = "";
      screen.renderRunManagerUi();
      window.__SEAMGRIM_TEST_RUN_SCREEN__ = screen;
    }, { moduleUrl: `${baseUrl}/solutions/seamgrim_ui_mvp/ui/screens/run.js` });
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_HISTORY_COMPARISON_RAIL__?.run_count === 2).catch(async (error) => {
      const debug = await readDebugState(page);
      throw new Error(`${error.message}\n${JSON.stringify(debug, null, 2)}`);
    });

    const initialRail = await readRail(page);
    assert(initialRail.railCount === 1, `history rail count mismatch: ${initialRail.railCount}`);
    assert(initialRail.schema === "seamgrim.run_history_comparison_rail.v1", `history rail schema mismatch: ${initialRail.schema}`);
    assert(initialRail.state === "ready", `history rail state mismatch: ${initialRail.state}`);
    assert(initialRail.count === "저장 run 2", `history count label mismatch: ${initialRail.count}`);
    assert(initialRail.countValue === "2", `history count value mismatch: ${initialRail.countValue}`);
    assert(initialRail.visible === "표시 1", `history visible label mismatch: ${initialRail.visible}`);
    assert(initialRail.visibleValue === "1", `history visible value mismatch: ${initialRail.visibleValue}`);
    assert(initialRail.active === "활성 없음", `history active label mismatch: ${initialRail.active}`);
    assert(initialRail.activeState === "none", `history active state mismatch: ${initialRail.activeState}`);
    assert(initialRail.solo === "solo 없음", `history solo label mismatch: ${initialRail.solo}`);
    assert(initialRail.model?.schema === "seamgrim.run_history_comparison_rail.v1", "history rail model schema mismatch");
    assert(initialRail.model?.hidden_count === 1, `history hidden count mismatch: ${initialRail.model?.hidden_count}`);

    await page.check('[data-run-visible="history-hidden"]');
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_HISTORY_COMPARISON_RAIL__?.visible_count === 2);
    const visibleRail = await readRail(page);
    assert(visibleRail.visible === "표시 2", `history visible after check mismatch: ${visibleRail.visible}`);
    assert(visibleRail.model?.hidden_count === 0, `history hidden after check mismatch: ${visibleRail.model?.hidden_count}`);

    await page.click('[data-run-solo="history-visible"]');
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_HISTORY_COMPARISON_RAIL__?.solo_run_id === "history-visible");
    const soloRail = await readRail(page);
    assert(soloRail.solo === "solo 기준 run", `history solo label mismatch: ${soloRail.solo}`);
    assert(soloRail.soloValue === "history-visible", `history solo value mismatch: ${soloRail.soloValue}`);
    assert(soloRail.soloState === "solo", `history solo state mismatch: ${soloRail.soloState}`);
    assert(soloRail.model?.has_solo === true, "history solo model flag mismatch");

    if (failures.length > 0) throw new Error(failures.join("\n"));
    await context.close();
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }

  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
