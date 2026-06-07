#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_lesson_publication_review_surface: ok";

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
  for (const rel of ["index.html", "app.js", "studio_lesson_publication_review_surface.js"]) {
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
    await page.waitForSelector("[data-lesson-publication-review-surface][data-lesson-publication-status='lesson_publication_review_surface_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS,
        DEFAULT_LESSON_PUBLICATION_DASHBOARD_ROWS,
        DEFAULT_LESSON_PUBLICATION_REVIEW_GATES,
        buildLessonPublicationReviewSurface,
        formatLessonPublicationReviewSurfaceText,
      } = await import("./studio_lesson_publication_review_surface.js");
      const surface = buildLessonPublicationReviewSurface({
        reviewGates: DEFAULT_LESSON_PUBLICATION_REVIEW_GATES,
        dashboardRows: DEFAULT_LESSON_PUBLICATION_DASHBOARD_ROWS,
        candidateIds: DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS,
      });
      return {
        surface,
        text: formatLessonPublicationReviewSurfaceText(surface),
      };
    });

    const surface = moduleResult.surface;
    assert(surface.__종류 === "studio_lesson_publication_review_surface", "surface kind mismatch");
    assert(surface.schema === "ddn.studio.lesson_publication_review_surface.v1", "surface schema mismatch");
    assert(surface.workflow_claim === "lesson_publication_review_surface", "workflow claim mismatch");
    assert(surface.product_ui_change === true, "surface must claim product ui change");
    assert(surface.runtime_claim === false, "surface must not claim runtime");
    assert(surface.public_upload_claim === false, "surface must not upload publicly");
    assert(surface.registry_publish_claim === false, "surface must not publish registry");
    assert(surface.publication_snapshot_emit_claim === false, "surface must not emit publication snapshot");
    assert(surface.lesson_schema_change === false, "surface must not change lesson schema");
    assert(surface.active_allowlist_mutation === false, "surface must not mutate allowlist");
    assert(surface.status === "lesson_publication_review_surface_ready", `status mismatch: ${surface.status}`);
    assert(surface.candidate_count === 12, `candidate count mismatch: ${surface.candidate_count}`);
    assert(surface.surface_row_count === 6, `surface count mismatch: ${surface.surface_row_count}`);
    assert(surface.ready_stage_count === 6, `ready stage mismatch: ${surface.ready_stage_count}`);
    assert(surface.progress.super_long_behavior_closed === 18, "super-long closed mismatch");
    assert(surface.progress.super_long_percent === 100, "super-long percent mismatch");
    assert(surface.progress.current_stage_closed === 6, "current stage closed mismatch");
    assert(surface.progress.current_stage_percent === 75, "current stage percent mismatch");
    assert(surface.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(surface.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("public_upload_claim\tfalse"), "formatted text missing upload boundary");
    assert(String(moduleResult.text).includes("active_allowlist_mutation\tfalse"), "formatted text missing allowlist boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-lesson-publication-review-surface]");
      const buttons = Array.from(document.querySelectorAll("[data-lesson-publication-surface]"));
      const firstTitle = document.querySelector("[data-lesson-publication-active-title]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-lesson-publication-surface") === "registry_share_handoff_surface")?.click();
      const registryTitle = document.querySelector("[data-lesson-publication-active-title]")?.textContent || "";
      const registryLane = document.querySelector("[data-lesson-publication-active-lane]")?.textContent || "";
      const candidateCount = document.querySelector("[data-lesson-publication-candidate-count]")?.textContent || "";
      document.querySelector("[data-lesson-publication-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-lesson-publication-status") || "",
        copied: root?.getAttribute("data-lesson-publication-copied") || "",
        buttonCount: buttons.length,
        firstTitle,
        registryTitle,
        registryLane,
        candidateCount,
        globalSchema: window.__SEAMGRIM_LESSON_PUBLICATION_REVIEW_SURFACE__?.schema || "",
        globalText: window.__SEAMGRIM_LESSON_PUBLICATION_REVIEW_SURFACE_TEXT__ || "",
      };
    });
    assert(domResult.status === "lesson_publication_review_surface_ready", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 6, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.firstTitle.includes("후보 수업"), `first title mismatch: ${domResult.firstTitle}`);
    assert(domResult.registryTitle.includes("Registry/share"), `registry title mismatch: ${domResult.registryTitle}`);
    assert(domResult.registryLane === "registry_share_handoff", `registry lane mismatch: ${domResult.registryLane}`);
    assert(domResult.candidateCount === "12", `dom candidate count mismatch: ${domResult.candidateCount}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.lesson_publication_review_surface.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("surface_id\tsurface_lane"), "global text missing surface header");

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
