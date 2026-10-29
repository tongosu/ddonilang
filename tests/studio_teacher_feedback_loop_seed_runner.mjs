#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_teacher_feedback_loop_seed: ok";

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
  for (const rel of ["index.html", "app.js", "studio_teacher_feedback_loop_seed.js"]) {
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
    await page.waitForSelector("[data-teacher-feedback-loop-seed][data-teacher-feedback-loop-seed-status='teacher_feedback_loop_seed_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_TEACHER_FEEDBACK_LOOP_SEED_ROWS,
        buildTeacherFeedbackLoopSeed,
        formatTeacherFeedbackLoopSeedText,
      } = await import("./studio_teacher_feedback_loop_seed.js");
      const seed = buildTeacherFeedbackLoopSeed({
        seedRows: DEFAULT_TEACHER_FEEDBACK_LOOP_SEED_ROWS,
      });
      return {
        seed,
        text: formatTeacherFeedbackLoopSeedText(seed),
      };
    });

    const seed = moduleResult.seed;
    assert(seed.__종류 === "studio_teacher_feedback_loop_seed", "kind mismatch");
    assert(seed.schema === "ddn.studio.teacher_feedback_loop_seed.v1", "schema mismatch");
    assert(seed.status === "teacher_feedback_loop_seed_ready", `status mismatch: ${seed.status}`);
    assert(seed.seed_row_count === 6, `seed count mismatch: ${seed.seed_row_count}`);
    assert(seed.all_seed_rows_seed_only === true, "seed-only aggregate mismatch");
    assert(seed.all_seed_rows_generated_now === false, "generated aggregate mismatch");
    assert(seed.all_seed_rows_write_claim === false, "write aggregate mismatch");
    assert(seed.teacher_feedback_runtime_claim === false, "must not claim teacher feedback runtime");
    assert(seed.student_data_collection_claim === false, "must not claim student data collection");
    assert(seed.feedback_write_claim === false, "must not claim feedback write");
    assert(seed.release_execution_claim === false, "must not claim release execution");
    assert(seed.product_ui_change === true, "must claim product ui change");
    assert(seed.ready_stage_count === 6, `ready stage mismatch: ${seed.ready_stage_count}`);
    assert(seed.progress.super_long_percent === 50, "super-long percent mismatch");
    assert(seed.progress.current_stage_closed === 5, "followup closed mismatch");
    assert(seed.progress.current_stage_percent === 63, "followup percent mismatch");
    assert(seed.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(seed.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(seed.next_item === "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1", "next item mismatch");
    assert(String(moduleResult.text).includes("seed_row_count\t6"), "formatted text missing seed count");
    assert(String(moduleResult.text).includes("student_data_collection_claim\tfalse"), "formatted text missing student data boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-teacher-feedback-loop-seed]");
      const buttons = Array.from(document.querySelectorAll("[data-teacher-loop-seed]"));
      const progress = document.querySelector("[data-teacher-loop-seed-progress]")?.textContent || "";
      const summary = document.querySelector("[data-teacher-loop-seed-summary]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-teacher-loop-seed") === "student_next_step_note")?.click();
      const title = document.querySelector("[data-teacher-loop-seed-active-title]")?.textContent || "";
      const surface = document.querySelector("[data-teacher-loop-seed-active-surface]")?.textContent || "";
      const source = document.querySelector("[data-teacher-loop-seed-active-source]")?.textContent || "";
      document.querySelector("[data-teacher-loop-seed-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-teacher-feedback-loop-seed-status") || "",
        copied: root?.getAttribute("data-teacher-feedback-loop-seed-copied") || "",
        buttonCount: buttons.length,
        progress,
        summary,
        title,
        surface,
        source,
        globalSchema: window.__SEAMGRIM_TEACHER_FEEDBACK_LOOP_SEED__?.schema || "",
        globalText: window.__SEAMGRIM_TEACHER_FEEDBACK_LOOP_SEED_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "teacher_feedback_loop_seed_ready", `dom status mismatch: ${domResult.rootStatus}`);
    assert(domResult.buttonCount === 6, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("5/8 follow-up") && domResult.progress.includes("63%"), `progress text mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("6 seed rows") && domResult.summary.includes("write_claim=false"), `summary text mismatch: ${domResult.summary}`);
    assert(domResult.title === "student next step", `title mismatch: ${domResult.title}`);
    assert(domResult.surface === "student_sheet.md", `surface mismatch: ${domResult.surface}`);
    assert(domResult.source === "classroom_report_workflow", `source mismatch: ${domResult.source}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.teacher_feedback_loop_seed.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("seed_id\tsurface\tsource_anchor\twrite_claim"), "global text missing header");

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
