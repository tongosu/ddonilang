#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "toolchain_benchmark_lts: ok";

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
    "toolchain_diagnostic_ui_lsp.js",
    "toolchain_registry_verification.js",
    "toolchain_benchmark_lts.js",
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
    await page.waitForSelector("[data-toolchain-benchmark-lts][data-toolchain-benchmark-lts-status='toolchain_benchmark_lts_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_TOOLCHAIN_BENCHMARK_LTS_ROWS,
        buildToolchainBenchmarkLts,
        formatToolchainBenchmarkLtsText,
      } = await import("./toolchain_benchmark_lts.js");
      const benchmark = buildToolchainBenchmarkLts({ rows: DEFAULT_TOOLCHAIN_BENCHMARK_LTS_ROWS });
      return { benchmark, text: formatToolchainBenchmarkLtsText(benchmark) };
    });
    const benchmark = moduleResult.benchmark;
    assert(benchmark.schema === "ddn.toolchain.benchmark_lts.v1", "schema mismatch");
    assert(benchmark.work_item === "TA5_BENCHMARK_LTS_V1", "work item mismatch");
    assert(benchmark.primary_coordinate === "타-5", "coordinate mismatch");
    assert(benchmark.depends_on_coordinate.join(",") === "타-4,타-3,타-2", "dependency mismatch");
    assert(benchmark.pack === "toolchain_pack_5_v1", "pack mismatch");
    assert(benchmark.status === "toolchain_benchmark_lts_ready", "status mismatch");
    assert(benchmark.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(benchmark.benchmark_lts_claim === true, "benchmark LTS mismatch");
    assert(benchmark.perf_budget_claim === true, "perf budget mismatch");
    assert(benchmark.reference_band_claim === true, "reference band mismatch");
    assert(benchmark.migration_ledger_claim === true, "migration ledger mismatch");
    assert(benchmark.lts_gate_claim === true, "LTS gate mismatch");
    assert(benchmark.benchmark_execution_claim === false, "benchmark execution must stay false");
    assert(benchmark.lts_certification_claim === false, "LTS certification must stay false");
    assert(benchmark.perf_regression_blocker_claim === false, "perf blocker must stay false");
    assert(benchmark.release_gate_execution_claim === false, "release gate execution must stay false");
    assert(benchmark.public_release_claim === false, "public release must stay false");
    assert(benchmark.cloud_benchmark_claim === false, "cloud benchmark must stay false");
    assert(benchmark.runtime_claim === false, "runtime claim must stay false");
    assert(benchmark.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(benchmark.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(benchmark.progress.roadmap_v2_matrix_behavior_closed === 24, "roadmap closed mismatch");
    assert(benchmark.progress.roadmap_v2_matrix_behavior_percent === 27, "roadmap percent mismatch");
    assert(benchmark.progress.roadmap_v2_pack_evidence_reference_closed === 44, "pack ref mismatch");
    assert(benchmark.progress.roadmap_v2_pack_evidence_reference_percent === 49, "pack ref percent mismatch");
    assert(benchmark.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(benchmark.rows.map((row) => row.id).join(",") === "perf_budget,reference_band,migration_ledger,lts_gate", "row order mismatch");
    assert(String(benchmark.benchmark_text).includes("benchmark_execution:false"), "benchmark text missing execution boundary");
    assert(String(benchmark.benchmark_text).includes("release_gate_execution:false"), "benchmark text missing release boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t44/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-toolchain-benchmark-lts]");
      const buttons = Array.from(document.querySelectorAll(".toolchain-benchmark-btn[data-toolchain-benchmark-row]"));
      buttons.find((button) => button.getAttribute("data-toolchain-benchmark-row") === "migration_ledger")?.click();
      document.querySelector("[data-toolchain-benchmark-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-toolchain-benchmark-lts-status") || "",
        copied: root?.getAttribute("data-toolchain-benchmark-lts-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-toolchain-benchmark-artifact]").length,
        progress: document.querySelector("[data-toolchain-benchmark-progress]")?.textContent || "",
        summary: document.querySelector("[data-toolchain-benchmark-summary]")?.textContent || "",
        title: document.querySelector("[data-toolchain-benchmark-active-title]")?.textContent || "",
        link: document.querySelector("[data-toolchain-benchmark-active-link]")?.textContent || "",
        preview: document.querySelector("[data-toolchain-benchmark-preview]")?.textContent || "",
        globalSchema: window.__TOOLCHAIN_BENCHMARK_LTS__?.schema || "",
        globalText: window.__TOOLCHAIN_BENCHMARK_LTS_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "toolchain_benchmark_lts_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("24/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("benchmark execution") && domResult.summary.includes("LTS certification") && domResult.summary.includes("release gate execution"), "summary missing boundary");
    assert(domResult.title === "Migration ledger", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("toolchain://benchmark/lts/migration"), "migration URI mismatch");
    assert(domResult.preview.includes("benchmark_execution:false") && domResult.preview.includes("release_gate_execution:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.toolchain.benchmark_lts.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t24/90"), "global text missing roadmap matrix");
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
