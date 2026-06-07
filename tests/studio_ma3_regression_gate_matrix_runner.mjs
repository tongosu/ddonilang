#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_ma3_regression_gate_matrix: ok";

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
  for (const rel of ["index.html", "app.js", "studio_ma3_regression_gate_matrix.js"]) {
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
    await page.waitForSelector("[data-ma3-regression-gate-matrix][data-ma3-regression-status='ma3_regression_gate_matrix_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_MA3_REGRESSION_GATE_EVIDENCE,
        buildMa3RegressionGateMatrix,
        formatMa3RegressionGateMatrixText,
      } = await import("./studio_ma3_regression_gate_matrix.js");
      const matrix = buildMa3RegressionGateMatrix({ evidenceRows: DEFAULT_MA3_REGRESSION_GATE_EVIDENCE });
      return {
        matrix,
        text: formatMa3RegressionGateMatrixText(matrix),
      };
    });

    const matrix = moduleResult.matrix;
    assert(matrix.__종류 === "studio_ma3_regression_gate_matrix", "matrix kind mismatch");
    assert(matrix.schema === "ddn.studio.ma3_regression_gate_matrix.v1", "matrix schema mismatch");
    assert(matrix.product_ui_change === true, "matrix must claim product ui change");
    assert(matrix.runtime_claim === false, "matrix must not claim runtime");
    assert(matrix.test_execution_claim === false, "matrix must not claim test execution");
    assert(matrix.release_execution_claim === false, "matrix must not claim release execution");
    assert(matrix.public_upload_claim === false, "matrix must not claim public upload");
    assert(matrix.status === "ma3_regression_gate_matrix_ready", `status mismatch: ${matrix.status}`);
    assert(matrix.gate_row_count === 6, `gate count mismatch: ${matrix.gate_row_count}`);
    assert(matrix.ready_stage_count === 6, `ready stage mismatch: ${matrix.ready_stage_count}`);
    assert(matrix.progress.super_long_behavior_closed === 18, "super-long closed mismatch");
    assert(matrix.progress.super_long_percent === 100, "super-long percent mismatch");
    assert(matrix.progress.current_stage_closed === 7, "current stage closed mismatch");
    assert(matrix.progress.current_stage_percent === 88, "current stage percent mismatch");
    assert(matrix.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(matrix.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("test_execution_claim\tfalse"), "formatted text missing test boundary");
    assert(String(moduleResult.text).includes("product_stabilization_smoke_gate"), "formatted text missing product smoke gate");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-ma3-regression-gate-matrix]");
      const buttons = Array.from(document.querySelectorAll("[data-ma3-regression-gate]"));
      const firstTitle = document.querySelector("[data-ma3-regression-active-title]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-ma3-regression-gate") === "product_stabilization_smoke_gate")?.click();
      const smokeTitle = document.querySelector("[data-ma3-regression-active-title]")?.textContent || "";
      const smokeLane = document.querySelector("[data-ma3-regression-active-lane]")?.textContent || "";
      const smokeRunner = document.querySelector("[data-ma3-regression-active-runner]")?.textContent || "";
      document.querySelector("[data-ma3-regression-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-ma3-regression-status") || "",
        copied: root?.getAttribute("data-ma3-regression-copied") || "",
        buttonCount: buttons.length,
        firstTitle,
        smokeTitle,
        smokeLane,
        smokeRunner,
        globalSchema: window.__SEAMGRIM_MA3_REGRESSION_GATE_MATRIX__?.schema || "",
        globalText: window.__SEAMGRIM_MA3_REGRESSION_GATE_MATRIX_TEXT__ || "",
      };
    });
    assert(domResult.status === "ma3_regression_gate_matrix_ready", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 6, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.firstTitle.includes("교사 피드백"), `first title mismatch: ${domResult.firstTitle}`);
    assert(domResult.smokeTitle.includes("제품 smoke"), `smoke title mismatch: ${domResult.smokeTitle}`);
    assert(domResult.smokeLane === "product_stabilization", `smoke lane mismatch: ${domResult.smokeLane}`);
    assert(domResult.smokeRunner === "tests/run_seamgrim_product_stabilization_smoke_check.py", `smoke runner mismatch: ${domResult.smokeRunner}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.ma3_regression_gate_matrix.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("gate_id\tgate_lane"), "global text missing gate header");

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
