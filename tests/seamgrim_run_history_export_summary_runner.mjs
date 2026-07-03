#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_run_history_export_summary: ok";

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
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function readSummary(page) {
  return page.evaluate(() => {
    const root = document.querySelector("[data-run-history-export-summary]");
    return {
      count: document.querySelectorAll("[data-run-history-export-summary]").length,
      schema: root?.dataset?.schema ?? "",
      state: root?.dataset?.state ?? "",
      meta: document.querySelector("[data-run-history-export-summary-meta]")?.textContent?.trim() ?? "",
      metaValue: document.querySelector("[data-run-history-export-summary-meta]")?.dataset?.value ?? "",
      text: document.querySelector("[data-run-history-export-summary-text]")?.textContent ?? "",
      buttonDisabled: document.querySelector("#btn-run-history-export-summary-copy")?.disabled ?? true,
      model: window.__SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY__ ?? null,
      copied: window.__SEAMGRIM_COPIED_RUN_HISTORY_EXPORT_SUMMARY__ ?? "",
    };
  });
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");

  for (const rel of ["index.html", "app.js", "styles.css", "screens/run.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    const page = await context.newPage();
    await page.goto("about:blank");
    await page.evaluate(async ({ moduleUrl }) => {
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          async writeText(value) {
            window.__SEAMGRIM_COPIED_RUN_HISTORY_EXPORT_SUMMARY__ = String(value ?? "");
          },
        },
      });
      const { RunScreen } = await import(moduleUrl);
      document.body.innerHTML = `
        <section id="screen-run">
          <div id="dotbogi-graph">
            <div id="run-manager-panel-anchor"></div>
          </div>
        </section>
      `;
      const screen = new RunScreen({ root: document.querySelector("#screen-run"), wasmState: { fpsLimit: 30, dtMax: 0.1 } });
      screen.graphPanelEl = document.querySelector("#dotbogi-graph");
      screen.dotbogi = { setPersistedGraph() {}, setBaseSeriesDisplay() {}, setOverlaySeries() {} };
      screen.syncDockRangeLabels = () => {};
      screen.ensureRunManagerUi();
      screen.runManagerPanelEl = document.querySelector("#run-manager-panel");
      screen.runManagerListEl = document.querySelector("#run-manager-list");
      screen.runHistoryComparisonRailEl = document.querySelector("[data-run-history-comparison-rail]");
      screen.runHistoryCountEl = document.querySelector("[data-run-history-count]");
      screen.runHistoryVisibleEl = document.querySelector("[data-run-history-visible]");
      screen.runHistoryActiveEl = document.querySelector("[data-run-history-active]");
      screen.runHistorySoloEl = document.querySelector("[data-run-history-solo]");
      screen.runHistoryExportSummaryEl = document.querySelector("[data-run-history-export-summary]");
      screen.runHistoryExportSummaryMetaEl = document.querySelector("[data-run-history-export-summary-meta]");
      screen.runHistoryExportSummaryTextEl = document.querySelector("[data-run-history-export-summary-text]");
      screen.runHistoryExportSummaryCopyBtn = document.querySelector("#btn-run-history-export-summary-copy");
      screen.runHistoryExportSummaryCopyBtn?.addEventListener("click", () => {
        void screen.handleCopyRunHistoryExportSummary();
      });
      screen.overlayRuns = [
        screen.normalizeRunManagerRun({
          id: "history-alpha",
          label: "기준 run",
          visible: true,
          layer_index: 0,
          source: { kind: "ddn", lessonId: "sample", text: "" },
          inputs: { controls: {} },
          graph: {
            schema: "seamgrim.graph.v1",
            meta: { source_input_hash: "input-alpha", result_hash: "result-alpha" },
            series: [{ id: "value", points: [{ x: 0, y: 1 }, { x: 1, y: 2 }] }],
          },
          hash: { input: "input-alpha", result: "result-alpha" },
        }, 0),
        screen.normalizeRunManagerRun({
          id: "history-beta",
          label: "비교 run",
          visible: false,
          layer_index: 1,
          source: { kind: "ddn", lessonId: "sample", text: "" },
          inputs: { controls: {} },
          graph: {
            schema: "seamgrim.graph.v1",
            meta: { source_input_hash: "input-beta", result_hash: "result-beta" },
            series: [{ id: "value", points: [{ x: 0, y: 3 }, { x: 1, y: 4 }, { x: 2, y: 5 }] }],
          },
          hash: { input: "input-beta", result: "result-beta" },
        }, 1),
      ];
      screen.activeOverlayRunId = "history-alpha";
      screen.soloOverlayRunId = "";
      screen.renderRunManagerUi();
      window.__SEAMGRIM_TEST_RUN_SCREEN__ = screen;
    }, { moduleUrl: `${baseUrl}/solutions/seamgrim_ui_mvp/ui/screens/run.js` });

    await page.waitForFunction(() => window.__SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY__?.run_count === 2);
    const summary = await readSummary(page);
    assert(summary.count === 1, `summary count mismatch: ${summary.count}`);
    assert(summary.schema === "seamgrim.run_history_export_summary.v1", `schema mismatch: ${summary.schema}`);
    assert(summary.state === "ready", `state mismatch: ${summary.state}`);
    assert(summary.meta === "2개 run · 표시 1", `meta mismatch: ${summary.meta}`);
    assert(summary.metaValue === "2", `meta value mismatch: ${summary.metaValue}`);
    assert(summary.buttonDisabled === false, "copy button should be enabled");
    assert(summary.text.includes("Seamgrim run history export summary"), "missing export heading");
    assert(summary.text.includes("runs: 2"), "missing run count");
    assert(summary.text.includes("visible: 1"), "missing visible count");
    assert(summary.text.includes("1. 기준 run [visible] hash:result-a points:2"), `missing alpha row:\n${summary.text}`);
    assert(summary.text.includes("2. 비교 run [hidden] hash:result-b points:3"), `missing beta row:\n${summary.text}`);
    assert(summary.model?.generated_by === "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1", "generated_by mismatch");
    assert(summary.model?.latest_run_id === "history-beta", `latest id mismatch: ${summary.model?.latest_run_id}`);

    await page.click("#btn-run-history-export-summary-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_COPIED_RUN_HISTORY_EXPORT_SUMMARY__?.includes?.("runs: 2"));
    const copied = await readSummary(page);
    assert(copied.copied.includes("2. 비교 run [hidden] hash:result-b points:3"), "copied summary mismatch");

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
