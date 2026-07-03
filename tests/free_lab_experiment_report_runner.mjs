#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "free_lab_experiment_report: ok";

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
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
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
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "free_lab_first_run.js",
    "free_lab_experiment_report.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 840 }, locale: "ko-KR" });
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-free-lab-experiment-report][data-free-lab-experiment-report-status='experiment_report_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_FREE_LAB_EXPERIMENT_REPORT_ROWS,
        buildFreeLabExperimentReport,
        formatFreeLabExperimentReportText,
      } = await import("./free_lab_experiment_report.js");
      const report = buildFreeLabExperimentReport({ rows: DEFAULT_FREE_LAB_EXPERIMENT_REPORT_ROWS });
      return {
        report,
        text: formatFreeLabExperimentReportText(report),
      };
    });
    const report = moduleResult.report;
    assert(report.schema === "ddn.seamgrim.free_lab.experiment_report.v1", "schema mismatch");
    assert(report.work_item === "BA2_FREE_LAB_EXPERIMENT_REPORT_PACK_CLOSURE_V1", "work item mismatch");
    assert(report.primary_coordinate === "바-2", "coordinate mismatch");
    assert(report.status === "experiment_report_ready", "status mismatch");
    assert(report.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(report.experiment_report_claim === true, "report claim mismatch");
    assert(report.product_ui_change === true, "product UI change mismatch");
    assert(report.runtime_claim === false, "runtime claim must stay false");
    assert(report.share_claim === false, "share claim must stay false");
    assert(report.registry_publish_claim === false, "registry publish claim must stay false");
    assert(report.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(report.progress.current_stage_total === 5, "stage total mismatch");
    assert(report.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(report.progress.roadmap_v2_matrix_behavior_closed === 9, "roadmap closed mismatch");
    assert(report.progress.roadmap_v2_matrix_behavior_percent === 10, "roadmap percent mismatch");
    assert(report.progress.roadmap_v2_pack_evidence_reference_closed === 28, "pack ref mismatch");
    assert(report.progress.roadmap_v2_pack_evidence_reference_percent === 31, "pack ref percent mismatch");
    assert(report.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(report.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(report.sections.map((section) => section.id).join(",") === "hypothesis,lever,metric,conclusion", "section order mismatch");
    assert(String(moduleResult.text).includes("section_id\tartifact_key\tready\tprompt"), "text header missing");
    assert(String(moduleResult.text).includes("roadmap_matrix\t9/90"), "text missing roadmap progress");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-free-lab-experiment-report]");
      const buttons = Array.from(document.querySelectorAll("[data-free-lab-report-section]"));
      buttons.find((button) => button.getAttribute("data-free-lab-report-section") === "metric")?.click();
      document.querySelector("[data-free-lab-report-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-free-lab-experiment-report-status") || "",
        copied: root?.getAttribute("data-free-lab-experiment-report-copied") || "",
        buttonCount: buttons.length,
        progress: document.querySelector("[data-free-lab-report-progress]")?.textContent || "",
        summary: document.querySelector("[data-free-lab-report-summary]")?.textContent || "",
        title: document.querySelector("[data-free-lab-report-active-title]")?.textContent || "",
        prompt: document.querySelector("[data-free-lab-report-active-prompt]")?.textContent || "",
        globalSchema: window.__SEAMGRIM_FREE_LAB_EXPERIMENT_REPORT__?.schema || "",
        globalText: window.__SEAMGRIM_FREE_LAB_EXPERIMENT_REPORT_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "experiment_report_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `section count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("9/90 ROADMAP") && domResult.progress.includes("5/5 stage"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("가설") && domResult.summary.includes("결론"), "summary missing artifact sections");
    assert(domResult.title === "지표", `active title mismatch: ${domResult.title}`);
    assert(domResult.prompt.includes("프레임수") && domResult.prompt.includes("결과"), "metric prompt mismatch");
    assert(domResult.globalSchema === "ddn.seamgrim.free_lab.experiment_report.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t28/90"), "global text missing pack reference");
    assert(domResult.globalText.includes("registry_publish_claim\tfalse"), "global text missing false claim boundary");

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
