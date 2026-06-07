#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_productization_stage_closure: ok";

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
  for (const rel of ["index.html", "app.js", "studio_productization_stage_closure.js"]) {
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
    await page.waitForSelector("[data-productization-stage-closure][data-productization-stage-closure-status='productization_stage_closed']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_PRODUCTIZATION_STAGE_CLOSURE_ROWS,
        buildProductizationStageClosure,
        formatProductizationStageClosureText,
      } = await import("./studio_productization_stage_closure.js");
      const closure = buildProductizationStageClosure({
        closureRows: DEFAULT_PRODUCTIZATION_STAGE_CLOSURE_ROWS,
      });
      return {
        closure,
        text: formatProductizationStageClosureText(closure),
      };
    });

    const closure = moduleResult.closure;
    assert(closure.__종류 === "studio_productization_stage_closure", "kind mismatch");
    assert(closure.schema === "ddn.studio.productization_stage_closure.v1", "schema mismatch");
    assert(closure.workflow_claim === "productization_stage_closure", "claim mismatch");
    assert(closure.status === "productization_stage_closed", `status mismatch: ${closure.status}`);
    assert(closure.product_ui_change === true, "must claim product ui change");
    assert(closure.runtime_claim === false, "must not claim runtime");
    assert(closure.release_execution_claim === false, "must not claim release execution");
    assert(closure.public_release_claim === false, "must not claim public release");
    assert(closure.solver_implementation_change === false, "must not claim solver change");
    assert(closure.stage_chain_closed === 5 && closure.stage_chain_total === 5, "stage chain mismatch");
    assert(closure.closure_row_count === 5, `row count mismatch: ${closure.closure_row_count}`);
    assert(closure.ready_stage_count === 6, `ready stage mismatch: ${closure.ready_stage_count}`);
    assert(closure.progress.super_long_behavior_closed === 18, "super-long closed mismatch");
    assert(closure.progress.super_long_percent === 100, "super-long percent mismatch");
    assert(closure.progress.current_stage_closed === 5, "current stage closed mismatch");
    assert(closure.progress.current_stage_percent === 100, "current stage percent mismatch");
    assert(closure.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(closure.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(closure.next_item === "STUDIO_POST_SUPER_LONG_REBASE_V1", "next item mismatch");
    assert(String(moduleResult.text).includes("stage_chain\t5/5"), "formatted text missing stage chain");
    assert(String(moduleResult.text).includes("release_execution_claim\tfalse"), "formatted text missing release boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-productization-stage-closure]");
      const buttons = Array.from(document.querySelectorAll("[data-productization-closure]"));
      const progress = document.querySelector("[data-productization-closure-progress]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-productization-closure") === "result_report_anchor")?.click();
      const title = document.querySelector("[data-productization-closure-active-title]")?.textContent || "";
      const lane = document.querySelector("[data-productization-closure-active-lane]")?.textContent || "";
      document.querySelector("[data-productization-closure-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-productization-stage-closure-status") || "",
        copied: root?.getAttribute("data-productization-stage-closure-copied") || "",
        buttonCount: buttons.length,
        progress,
        title,
        lane,
        globalSchema: window.__SEAMGRIM_PRODUCTIZATION_STAGE_CLOSURE__?.schema || "",
        globalText: window.__SEAMGRIM_PRODUCTIZATION_STAGE_CLOSURE_TEXT__ || "",
      };
    });
    assert(domResult.status === "productization_stage_closed", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 5, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("5/5") && domResult.progress.includes("100%"), `progress text mismatch: ${domResult.progress}`);
    assert(domResult.progress.includes("18/18 overall"), `overall text mismatch: ${domResult.progress}`);
    assert(domResult.title.includes("result report"), `title mismatch: ${domResult.title}`);
    assert(domResult.lane === "result_report", `lane mismatch: ${domResult.lane}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.productization_stage_closure.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("closure_id\tlane"), "global text missing header");

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
