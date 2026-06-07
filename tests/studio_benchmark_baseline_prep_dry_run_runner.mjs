#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_benchmark_baseline_prep_dry_run: ok";

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
  for (const rel of ["index.html", "app.js", "studio_benchmark_baseline_prep_dry_run.js"]) {
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
    await page.waitForSelector("[data-benchmark-baseline-prep-dry-run][data-benchmark-baseline-prep-dry-run-status='benchmark_baseline_prep_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_BENCHMARK_BASELINE_PREP_INPUT_ROWS,
        buildBenchmarkBaselinePrepDryRun,
        formatBenchmarkBaselinePrepDryRunText,
      } = await import("./studio_benchmark_baseline_prep_dry_run.js");
      const prep = buildBenchmarkBaselinePrepDryRun({
        inputRows: DEFAULT_BENCHMARK_BASELINE_PREP_INPUT_ROWS,
      });
      return {
        prep,
        text: formatBenchmarkBaselinePrepDryRunText(prep),
      };
    });

    const prep = moduleResult.prep;
    assert(prep.__종류 === "studio_benchmark_baseline_prep_dry_run", "kind mismatch");
    assert(prep.schema === "ddn.studio.benchmark_baseline_prep_dry_run.v1", "schema mismatch");
    assert(prep.status === "benchmark_baseline_prep_ready", `status mismatch: ${prep.status}`);
    assert(prep.planned_baseline_input_count === 5, `input count mismatch: ${prep.planned_baseline_input_count}`);
    assert(prep.all_inputs_prep_only === true, "prep-only aggregate mismatch");
    assert(prep.all_inputs_generated_now === false, "generated aggregate mismatch");
    assert(prep.all_inputs_benchmark_execution_claim === false, "benchmark aggregate mismatch");
    assert(prep.benchmark_execution_claim === false, "must not claim benchmark execution");
    assert(prep.performance_baseline_generation_claim === false, "must not claim baseline generation");
    assert(prep.lts_certification_claim === false, "must not claim LTS certification");
    assert(prep.release_execution_claim === false, "must not claim release execution");
    assert(prep.product_ui_change === true, "must claim product ui change");
    assert(prep.ready_stage_count === 6, `ready stage mismatch: ${prep.ready_stage_count}`);
    assert(prep.progress.super_long_percent === 100, "super-long percent mismatch");
    assert(prep.progress.current_stage_closed === 7, "followup closed mismatch");
    assert(prep.progress.current_stage_percent === 88, "followup percent mismatch");
    assert(prep.progress.roadmap_v2_behavior_closed === 87, "roadmap closed mismatch");
    assert(prep.progress.roadmap_v2_percent === 97, "roadmap percent mismatch");
    assert(prep.next_item === "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1", "next item mismatch");
    assert(String(moduleResult.text).includes("planned_baseline_input_count\t5"), "formatted text missing input count");
    assert(String(moduleResult.text).includes("benchmark_execution_claim\tfalse"), "formatted text missing benchmark boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-benchmark-baseline-prep-dry-run]");
      const buttons = Array.from(document.querySelectorAll("[data-benchmark-prep]"));
      const progress = document.querySelector("[data-benchmark-prep-progress]")?.textContent || "";
      const summary = document.querySelector("[data-benchmark-prep-summary]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-benchmark-prep") === "classroom_operations_triage_input")?.click();
      const title = document.querySelector("[data-benchmark-prep-active-title]")?.textContent || "";
      const lane = document.querySelector("[data-benchmark-prep-active-lane]")?.textContent || "";
      const pathText = document.querySelector("[data-benchmark-prep-active-path]")?.textContent || "";
      document.querySelector("[data-benchmark-prep-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-benchmark-baseline-prep-dry-run-status") || "",
        copied: root?.getAttribute("data-benchmark-baseline-prep-dry-run-copied") || "",
        buttonCount: buttons.length,
        progress,
        summary,
        title,
        lane,
        pathText,
        globalSchema: window.__SEAMGRIM_BENCHMARK_BASELINE_PREP_DRY_RUN__?.schema || "",
        globalText: window.__SEAMGRIM_BENCHMARK_BASELINE_PREP_DRY_RUN_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "benchmark_baseline_prep_ready", `dom status mismatch: ${domResult.rootStatus}`);
    assert(domResult.buttonCount === 5, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("7/8 follow-up") && domResult.progress.includes("88%"), `progress text mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("5 inputs") && domResult.summary.includes("benchmark_execution=false"), `summary text mismatch: ${domResult.summary}`);
    assert(domResult.title === "classroom triage", `title mismatch: ${domResult.title}`);
    assert(domResult.lane === "classroom_operations", `lane mismatch: ${domResult.lane}`);
    assert(domResult.pathText === "build/studio_benchmark/baseline/classroom_operations_triage.detjson", `path mismatch: ${domResult.pathText}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.benchmark_baseline_prep_dry_run.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("input_id\tlane\tsource_anchor\tgenerated_now"), "global text missing header");

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
