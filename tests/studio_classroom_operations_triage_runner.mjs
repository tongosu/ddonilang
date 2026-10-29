#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_classroom_operations_triage: ok";

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
  for (const rel of ["index.html", "app.js", "studio_classroom_operations_triage.js"]) {
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
    await page.waitForSelector("[data-classroom-operations-triage][data-classroom-operations-triage-status='classroom_operations_triage_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_UI_ROWS,
        buildClassroomOperationsTriage,
        formatClassroomOperationsTriageText,
      } = await import("./studio_classroom_operations_triage.js");
      const triage = buildClassroomOperationsTriage({
        triageRows: DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_UI_ROWS,
      });
      return {
        triage,
        text: formatClassroomOperationsTriageText(triage),
      };
    });

    const triage = moduleResult.triage;
    assert(triage.__종류 === "studio_classroom_operations_triage", "kind mismatch");
    assert(triage.schema === "ddn.studio.classroom_operations_triage.v1", "schema mismatch");
    assert(triage.status === "classroom_operations_triage_ready", `status mismatch: ${triage.status}`);
    assert(triage.triage_row_count === 6, `triage count mismatch: ${triage.triage_row_count}`);
    assert(triage.all_triage_rows_triage_only === true, "triage-only aggregate mismatch");
    assert(triage.all_triage_rows_generated_now === false, "generated aggregate mismatch");
    assert(triage.all_triage_rows_write_claim === false, "write aggregate mismatch");
    assert(triage.classroom_operations_runtime_claim === false, "must not claim classroom runtime");
    assert(triage.student_data_collection_claim === false, "must not claim student data collection");
    assert(triage.triage_write_claim === false, "must not claim triage write");
    assert(triage.release_execution_claim === false, "must not claim release execution");
    assert(triage.product_ui_change === true, "must claim product ui change");
    assert(triage.ready_stage_count === 6, `ready stage mismatch: ${triage.ready_stage_count}`);
    assert(triage.progress.super_long_percent === 50, "super-long percent mismatch");
    assert(triage.progress.current_stage_closed === 6, "followup closed mismatch");
    assert(triage.progress.current_stage_percent === 75, "followup percent mismatch");
    assert(triage.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(triage.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(triage.next_item === "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1", "next item mismatch");
    assert(String(moduleResult.text).includes("triage_row_count\t6"), "formatted text missing triage count");
    assert(String(moduleResult.text).includes("student_data_collection_claim\tfalse"), "formatted text missing student data boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-classroom-operations-triage]");
      const buttons = Array.from(document.querySelectorAll("[data-classroom-triage]"));
      const progress = document.querySelector("[data-classroom-triage-progress]")?.textContent || "";
      const summary = document.querySelector("[data-classroom-triage-summary]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-classroom-triage") === "student_next_step_queue")?.click();
      const title = document.querySelector("[data-classroom-triage-active-title]")?.textContent || "";
      const lane = document.querySelector("[data-classroom-triage-active-lane]")?.textContent || "";
      const source = document.querySelector("[data-classroom-triage-active-source]")?.textContent || "";
      document.querySelector("[data-classroom-triage-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-classroom-operations-triage-status") || "",
        copied: root?.getAttribute("data-classroom-operations-triage-copied") || "",
        buttonCount: buttons.length,
        progress,
        summary,
        title,
        lane,
        source,
        globalSchema: window.__SEAMGRIM_CLASSROOM_OPERATIONS_TRIAGE__?.schema || "",
        globalText: window.__SEAMGRIM_CLASSROOM_OPERATIONS_TRIAGE_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "classroom_operations_triage_ready", `dom status mismatch: ${domResult.rootStatus}`);
    assert(domResult.buttonCount === 6, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("6/8 follow-up") && domResult.progress.includes("75%"), `progress text mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("6 triage rows") && domResult.summary.includes("write_claim=false"), `summary text mismatch: ${domResult.summary}`);
    assert(domResult.title === "student queue", `title mismatch: ${domResult.title}`);
    assert(domResult.lane === "student_next_step", `lane mismatch: ${domResult.lane}`);
    assert(domResult.source === "teacher_feedback_loop_seed", `source mismatch: ${domResult.source}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.classroom_operations_triage.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("triage_id\tlane\tsource_anchor\twrite_claim"), "global text missing header");

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
