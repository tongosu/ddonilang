#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_classroom_operations_panel_preview: ok";

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
  for (const rel of ["index.html", "app.js", "studio_classroom_operations_panel_preview.js"]) {
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
    await page.waitForSelector("[data-classroom-operations-panel-preview][data-classroom-operations-status='classroom_operations_panel_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_ROWS,
        buildClassroomOperationsPanelPreview,
        formatClassroomOperationsPanelPreviewText,
      } = await import("./studio_classroom_operations_panel_preview.js");
      const panel = buildClassroomOperationsPanelPreview({ triageRows: DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_ROWS });
      return {
        panel,
        text: formatClassroomOperationsPanelPreviewText(panel),
      };
    });

    const panel = moduleResult.panel;
    assert(panel.__종류 === "studio_classroom_operations_panel_preview", "panel kind mismatch");
    assert(panel.schema === "ddn.studio.classroom_operations_panel_preview.v1", "panel schema mismatch");
    assert(panel.workflow_claim === "classroom_operations_panel_preview", "workflow claim mismatch");
    assert(panel.product_ui_change === true, "panel must claim product ui change");
    assert(panel.runtime_claim === false, "panel must not claim runtime");
    assert(panel.student_data_collection_claim === false, "panel must not collect student data");
    assert(panel.panel_write_claim === false, "panel must not write");
    assert(panel.status === "classroom_operations_panel_ready", `status mismatch: ${panel.status}`);
    assert(panel.panel_row_count === 6, `panel count mismatch: ${panel.panel_row_count}`);
    assert(panel.ready_stage_count === 6, `ready stage mismatch: ${panel.ready_stage_count}`);
    assert(panel.progress.super_long_behavior_closed === 18, "super-long closed mismatch");
    assert(panel.progress.super_long_percent === 100, "super-long percent mismatch");
    assert(panel.progress.current_stage_closed === 3, "current stage closed mismatch");
    assert(panel.progress.current_stage_percent === 38, "current stage percent mismatch");
    assert(panel.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(panel.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("product_ui_change\ttrue"), "formatted text missing product ui change");
    assert(String(moduleResult.text).includes("panel_write_claim\tfalse"), "formatted text missing write boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-classroom-operations-panel-preview]");
      const buttons = Array.from(document.querySelectorAll("[data-classroom-operations-panel]"));
      const firstTitle = document.querySelector("[data-classroom-operations-active-title]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-classroom-operations-panel") === "misconception_review_queue_panel")?.click();
      const reviewTitle = document.querySelector("[data-classroom-operations-active-title]")?.textContent || "";
      const reviewLane = document.querySelector("[data-classroom-operations-active-lane]")?.textContent || "";
      document.querySelector("[data-classroom-operations-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-classroom-operations-status") || "",
        copied: root?.getAttribute("data-classroom-operations-copied") || "",
        buttonCount: buttons.length,
        firstTitle,
        reviewTitle,
        reviewLane,
        globalSchema: window.__SEAMGRIM_CLASSROOM_OPERATIONS_PANEL_PREVIEW__?.schema || "",
        globalText: window.__SEAMGRIM_CLASSROOM_OPERATIONS_PANEL_PREVIEW_TEXT__ || "",
      };
    });
    assert(domResult.status === "classroom_operations_panel_ready", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 6, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.firstTitle.includes("수업 보고"), `first title mismatch: ${domResult.firstTitle}`);
    assert(domResult.reviewTitle.includes("오개념"), `review title mismatch: ${domResult.reviewTitle}`);
    assert(domResult.reviewLane === "misconception_review", `review lane mismatch: ${domResult.reviewLane}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.classroom_operations_panel_preview.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("panel_id\toperations_lane"), "global text missing panel header");

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
