#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_numeric_result_stage: ok";

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
  return "text/plain; charset=utf-8";
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
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
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
      if (!address || typeof address === "string") reject(new Error("failed to bind static server"));
      else resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    return (
      url.pathname === "/api/lessons/inventory" ||
      url.pathname === "/api/lesson-inventory" ||
      url.pathname.startsWith("/lessons/") ||
      url.pathname.startsWith("/seed_lessons_v1/")
    );
  } catch (_) {
    return false;
  }
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of ["index.html", "app.js", "studio_numeric_result_report_stage.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1180, height: 760 }, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error" && !String(msg.text() ?? "").includes("Failed to load resource")) {
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400 && !res.url().endsWith("/favicon.ico") && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-numeric-result-report-stage][data-numeric-result-report-stage-status='numeric_result_report_stage_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_NUMERIC_RESULT_REPORT_STAGE_ROWS,
        buildNumericResultReportStage,
        formatNumericResultReportStageText,
      } = await import("./studio_numeric_result_report_stage.js");
      const stage = buildNumericResultReportStage({
        resultRows: DEFAULT_NUMERIC_RESULT_REPORT_STAGE_ROWS,
      });
      return {
        stage,
        text: formatNumericResultReportStageText(stage),
      };
    });

    const stage = moduleResult.stage;
    assert(stage.__종류 === "studio_numeric_result_report_stage", "kind mismatch");
    assert(stage.schema === "ddn.studio.numeric_result_report_stage.v1", "schema mismatch");
    assert(stage.workflow_schema === "seamgrim.numeric_result_report_consolidation.v1", "workflow schema mismatch");
    assert(stage.product_ui_change === true, "must claim product ui change");
    assert(stage.runtime_claim === false, "must not claim runtime");
    assert(stage.replay_claim === false, "must not claim replay");
    assert(stage.new_export_wrapper_claim === false, "must not claim new export wrapper");
    assert(stage.solver_implementation_change === false, "must not claim solver change");
    assert(stage.status === "numeric_result_report_stage_ready", `status mismatch: ${stage.status}`);
    assert(stage.result_row_count === 5, `row count mismatch: ${stage.result_row_count}`);
    assert(stage.result_count === 3, "result count mismatch");
    assert(stage.pair_count === 2, "pair count mismatch");
    assert(stage.evidence_pack_count === 3, "evidence count mismatch");
    assert(stage.report_workflow_stage_count === 17, "workflow stage count mismatch");
    assert(stage.result_report_stage_count === 10, "result report stage count mismatch");
    assert(stage.progress.super_long_behavior_closed === 18, "super-long closed mismatch");
    assert(stage.progress.super_long_percent === 100, "super-long percent mismatch");
    assert(stage.progress.current_stage_closed === 4, "current stage closed mismatch");
    assert(stage.progress.current_stage_percent === 80, "current stage percent mismatch");
    assert(stage.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(stage.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("result_report_stage_count\t10"), "formatted text missing result count");
    assert(String(moduleResult.text).includes("new_export_wrapper_claim\tfalse"), "formatted text missing export boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-numeric-result-report-stage]");
      const buttons = Array.from(document.querySelectorAll("[data-numeric-result-stage]"));
      const progress = document.querySelector("[data-numeric-result-stage-progress]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-numeric-result-stage") === "evidence_pack_rollup")?.click();
      const title = document.querySelector("[data-numeric-result-stage-active-title]")?.textContent || "";
      const lane = document.querySelector("[data-numeric-result-stage-active-lane]")?.textContent || "";
      document.querySelector("[data-numeric-result-stage-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-numeric-result-report-stage-status") || "",
        copied: root?.getAttribute("data-numeric-result-report-stage-copied") || "",
        buttonCount: buttons.length,
        progress,
        title,
        lane,
        globalSchema: window.__SEAMGRIM_NUMERIC_RESULT_REPORT_STAGE__?.schema || "",
        globalText: window.__SEAMGRIM_NUMERIC_RESULT_REPORT_STAGE_TEXT__ || "",
      };
    });
    assert(domResult.status === "numeric_result_report_stage_ready", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 5, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("4/5") && domResult.progress.includes("80%"), `progress text mismatch: ${domResult.progress}`);
    assert(domResult.progress.includes("10/10 result"), `result text mismatch: ${domResult.progress}`);
    assert(domResult.progress.includes("17/17 workflow"), `workflow text mismatch: ${domResult.progress}`);
    assert(domResult.title.includes("evidence pack"), `title mismatch: ${domResult.title}`);
    assert(domResult.lane === "evidence_rollup", `lane mismatch: ${domResult.lane}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.numeric_result_report_stage.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("result_id\tlane"), "global text missing header");

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
