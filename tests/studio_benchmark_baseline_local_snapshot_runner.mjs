#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_benchmark_baseline_local_snapshot: ok";

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

function listenOnSafePort(server, startPort) {
  return new Promise((resolve, reject) => {
    let port = startPort;
    const tryListen = () => {
      const onError = (error) => {
        server.off("listening", onListening);
        if (error?.code === "EADDRINUSE" && port < startPort + 50) {
          port += 1;
          tryListen();
          return;
        }
        reject(error);
      };
      const onListening = () => {
        server.off("error", onError);
        resolve({ server, baseUrl: `http://127.0.0.1:${port}` });
      };
      server.once("error", onError);
      server.once("listening", onListening);
      server.listen(port, "127.0.0.1");
    };
    tryListen();
  });
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
  return listenOnSafePort(server, 17660);
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
  for (const rel of ["index.html", "app.js", "studio_benchmark_baseline_local_snapshot.js"]) {
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
    await page.waitForSelector("[data-benchmark-baseline-local-snapshot][data-benchmark-baseline-status='benchmark_baseline_snapshot_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_BENCHMARK_BASELINE_INPUTS,
        DEFAULT_BENCHMARK_CLASSROOM_PANEL_ROWS,
        buildBenchmarkBaselineLocalSnapshot,
        formatBenchmarkBaselineLocalSnapshotText,
      } = await import("./studio_benchmark_baseline_local_snapshot.js");
      const snapshot = buildBenchmarkBaselineLocalSnapshot({
        plannedInputs: DEFAULT_BENCHMARK_BASELINE_INPUTS,
        panelRows: DEFAULT_BENCHMARK_CLASSROOM_PANEL_ROWS,
      });
      return {
        snapshot,
        text: formatBenchmarkBaselineLocalSnapshotText(snapshot),
      };
    });

    const snapshot = moduleResult.snapshot;
    assert(snapshot.__종류 === "studio_benchmark_baseline_local_snapshot", "snapshot kind mismatch");
    assert(snapshot.schema === "ddn.studio.benchmark_baseline_local_snapshot.v1", "snapshot schema mismatch");
    assert(snapshot.workflow_claim === "benchmark_baseline_local_snapshot", "workflow claim mismatch");
    assert(snapshot.product_ui_change === true, "snapshot must claim product ui change");
    assert(snapshot.runtime_claim === false, "snapshot must not claim runtime");
    assert(snapshot.benchmark_execution_claim === false, "snapshot must not execute benchmark");
    assert(snapshot.performance_baseline_generation_claim === false, "snapshot must not generate performance baseline");
    assert(snapshot.performance_baseline_publication_claim === false, "snapshot must not publish performance baseline");
    assert(snapshot.status === "benchmark_baseline_snapshot_ready", `status mismatch: ${snapshot.status}`);
    assert(snapshot.snapshot_row_count === 6, `snapshot count mismatch: ${snapshot.snapshot_row_count}`);
    assert(snapshot.ready_stage_count === 6, `ready stage mismatch: ${snapshot.ready_stage_count}`);
    assert(snapshot.progress.super_long_behavior_closed === 9, "super-long closed mismatch");
    assert(snapshot.progress.super_long_percent === 50, "super-long percent mismatch");
    assert(snapshot.progress.current_stage_closed === 4, "current stage closed mismatch");
    assert(snapshot.progress.current_stage_percent === 50, "current stage percent mismatch");
    assert(snapshot.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(snapshot.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("product_ui_change\ttrue"), "formatted text missing product ui change");
    assert(String(moduleResult.text).includes("benchmark_execution_claim\tfalse"), "formatted text missing benchmark boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-benchmark-baseline-local-snapshot]");
      const buttons = Array.from(document.querySelectorAll("[data-benchmark-baseline-snapshot]"));
      const firstTitle = document.querySelector("[data-benchmark-baseline-active-title]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-benchmark-baseline-snapshot") === "classroom_operations_panel_snapshot")?.click();
      const panelTitle = document.querySelector("[data-benchmark-baseline-active-title]")?.textContent || "";
      const panelLane = document.querySelector("[data-benchmark-baseline-active-lane]")?.textContent || "";
      document.querySelector("[data-benchmark-baseline-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-benchmark-baseline-status") || "",
        copied: root?.getAttribute("data-benchmark-baseline-copied") || "",
        buttonCount: buttons.length,
        firstTitle,
        panelTitle,
        panelLane,
        globalSchema: window.__SEAMGRIM_BENCHMARK_BASELINE_LOCAL_SNAPSHOT__?.schema || "",
        globalText: window.__SEAMGRIM_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_TEXT__ || "",
      };
    });
    assert(domResult.status === "benchmark_baseline_snapshot_ready", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 6, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.firstTitle.includes("Benchmark/LTS"), `first title mismatch: ${domResult.firstTitle}`);
    assert(domResult.panelTitle.includes("수업 운영 패널"), `panel title mismatch: ${domResult.panelTitle}`);
    assert(domResult.panelLane === "classroom_operations_panel", `panel lane mismatch: ${domResult.panelLane}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.benchmark_baseline_local_snapshot.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("snapshot_id\tsnapshot_lane"), "global text missing snapshot header");

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
