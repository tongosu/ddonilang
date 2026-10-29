#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_productization_stage_rebase: ok";

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
  for (const rel of ["index.html", "app.js", "studio_productization_stage_rebase.js"]) {
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
    await page.waitForSelector("[data-productization-stage-rebase][data-productization-stage-rebase-status='productization_stage_rebased']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_PRODUCTIZATION_STAGE_REBASE_ROWS,
        buildProductizationStageRebase,
        formatProductizationStageRebaseText,
      } = await import("./studio_productization_stage_rebase.js");
      const rebase = buildProductizationStageRebase({ rebaseRows: DEFAULT_PRODUCTIZATION_STAGE_REBASE_ROWS });
      return {
        rebase,
        text: formatProductizationStageRebaseText(rebase),
      };
    });

    const rebase = moduleResult.rebase;
    assert(rebase.__종류 === "studio_productization_stage_rebase", "rebase kind mismatch");
    assert(rebase.schema === "ddn.studio.productization_stage_rebase.v1", "rebase schema mismatch");
    assert(rebase.product_ui_change === true, "rebase must claim product ui change");
    assert(rebase.runtime_claim === false, "rebase must not claim runtime");
    assert(rebase.new_automatic_queue_claim === false, "rebase must not claim new automatic queue");
    assert(rebase.release_execution_claim === false, "rebase must not claim release execution");
    assert(rebase.public_upload_claim === false, "rebase must not claim public upload");
    assert(rebase.selected_next_item === "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1", "selected next item mismatch");
    assert(rebase.status === "productization_stage_rebased", `status mismatch: ${rebase.status}`);
    assert(rebase.rebase_row_count === 5, `rebase count mismatch: ${rebase.rebase_row_count}`);
    assert(rebase.ready_stage_count === 5, `ready stage mismatch: ${rebase.ready_stage_count}`);
    assert(rebase.progress.super_long_behavior_closed === 9, "super-long closed mismatch");
    assert(rebase.progress.super_long_percent === 50, "super-long percent mismatch");
    assert(rebase.progress.current_stage_closed === 1, "current stage closed mismatch");
    assert(rebase.progress.current_stage_percent === 20, "current stage percent mismatch");
    assert(rebase.progress.roadmap_v2_behavior_closed === 51, "roadmap closed mismatch");
    assert(rebase.progress.roadmap_v2_percent === 57, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("selected_next_item\tSEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1"), "formatted text missing next item");
    assert(String(moduleResult.text).includes("release_execution_claim\tfalse"), "formatted text missing release boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-productization-stage-rebase]");
      const buttons = Array.from(document.querySelectorAll("[data-productization-rebase]"));
      const progress = document.querySelector("[data-productization-rebase-progress]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-productization-rebase") === "micro_slice_consolidation_priority")?.click();
      const title = document.querySelector("[data-productization-rebase-active-title]")?.textContent || "";
      const lane = document.querySelector("[data-productization-rebase-active-lane]")?.textContent || "";
      const coordinate = document.querySelector("[data-productization-rebase-active-coordinate]")?.textContent || "";
      document.querySelector("[data-productization-rebase-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-productization-stage-rebase-status") || "",
        copied: root?.getAttribute("data-productization-stage-rebase-copied") || "",
        buttonCount: buttons.length,
        progress,
        title,
        lane,
        coordinate,
        globalSchema: window.__SEAMGRIM_PRODUCTIZATION_STAGE_REBASE__?.schema || "",
        globalText: window.__SEAMGRIM_PRODUCTIZATION_STAGE_REBASE_TEXT__ || "",
      };
    });
    assert(domResult.status === "productization_stage_rebased", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 5, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("1/5") && domResult.progress.includes("20%"), `progress text mismatch: ${domResult.progress}`);
    assert(domResult.title.includes("micro-slice"), `title mismatch: ${domResult.title}`);
    assert(domResult.lane === "micro_slice_consolidation", `lane mismatch: ${domResult.lane}`);
    assert(domResult.coordinate === "마-3", `coordinate mismatch: ${domResult.coordinate}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.productization_stage_rebase.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("rebase_id\tcoordinate"), "global text missing rebase header");

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
