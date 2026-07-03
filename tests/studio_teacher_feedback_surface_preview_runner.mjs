#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_teacher_feedback_surface_preview: ok";

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
  for (const rel of ["index.html", "app.js", "studio_teacher_feedback_surface_preview.js"]) {
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-teacher-feedback-preview-panel][data-teacher-feedback-status='teacher_feedback_preview_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_TEACHER_FEEDBACK_SEED_ROWS,
        buildTeacherFeedbackSurfacePreview,
        formatTeacherFeedbackSurfacePreviewText,
      } = await import("./studio_teacher_feedback_surface_preview.js");
      const preview = buildTeacherFeedbackSurfacePreview({ seedRows: DEFAULT_TEACHER_FEEDBACK_SEED_ROWS });
      return {
        preview,
        text: formatTeacherFeedbackSurfacePreviewText(preview),
      };
    });

    const preview = moduleResult.preview;
    assert(preview.__종류 === "studio_teacher_feedback_surface_preview", "preview kind mismatch");
    assert(preview.schema === "ddn.studio.teacher_feedback_surface_preview.v1", "preview schema mismatch");
    assert(preview.workflow_claim === "teacher_feedback_surface_preview", "workflow claim mismatch");
    assert(preview.product_ui_change === true, "preview must claim product ui change");
    assert(preview.runtime_claim === false, "preview must not claim runtime");
    assert(preview.student_data_collection_claim === false, "preview must not collect student data");
    assert(preview.feedback_write_claim === false, "preview must not write feedback");
    assert(preview.status === "teacher_feedback_preview_ready", `status mismatch: ${preview.status}`);
    assert(preview.preview_section_count === 6, `section count mismatch: ${preview.preview_section_count}`);
    assert(preview.ready_stage_count === 6, `ready stage mismatch: ${preview.ready_stage_count}`);
    assert(preview.progress.super_long_behavior_closed === 9, "super-long closed mismatch");
    assert(preview.progress.super_long_percent === 50, "super-long percent mismatch");
    assert(preview.progress.current_stage_closed === 2, "current stage closed mismatch");
    assert(preview.progress.current_stage_percent === 25, "current stage percent mismatch");
    assert(preview.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(preview.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("product_ui_change\ttrue"), "formatted text missing product ui change");
    assert(String(moduleResult.text).includes("feedback_write_claim\tfalse"), "formatted text missing write boundary");

    const domResult = await page.evaluate(async () => {
      const panel = document.querySelector("[data-teacher-feedback-preview-panel]");
      const buttons = Array.from(document.querySelectorAll("[data-teacher-feedback-section]"));
      const firstTitle = document.querySelector("[data-teacher-feedback-active-title]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-teacher-feedback-section") === "retry_prompt_panel")?.click();
      const retryTitle = document.querySelector("[data-teacher-feedback-active-title]")?.textContent || "";
      const retryLane = document.querySelector("[data-teacher-feedback-active-lane]")?.textContent || "";
      document.querySelector("[data-teacher-feedback-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: panel?.getAttribute("data-teacher-feedback-status") || "",
        copied: panel?.getAttribute("data-teacher-feedback-copied") || "",
        buttonCount: buttons.length,
        firstTitle,
        retryTitle,
        retryLane,
        globalSchema: window.__SEAMGRIM_TEACHER_FEEDBACK_SURFACE_PREVIEW__?.schema || "",
        globalText: window.__SEAMGRIM_TEACHER_FEEDBACK_SURFACE_PREVIEW_TEXT__ || "",
      };
    });
    assert(domResult.status === "teacher_feedback_preview_ready", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 6, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.firstTitle.includes("교사용 요약"), `first title mismatch: ${domResult.firstTitle}`);
    assert(domResult.retryTitle.includes("재시도"), `retry title mismatch: ${domResult.retryTitle}`);
    assert(domResult.retryLane === "retry_prompt", `retry lane mismatch: ${domResult.retryLane}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.teacher_feedback_surface_preview.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("section_id\tsurface_lane"), "global text missing section header");

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
