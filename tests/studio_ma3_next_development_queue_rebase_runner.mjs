#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_ma3_next_development_queue_rebase: ok";

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
  for (const rel of ["index.html", "app.js", "studio_ma3_next_development_queue_rebase.js"]) {
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
    await page.waitForSelector("[data-ma3-next-development-queue-rebase][data-ma3-next-development-queue-status='ma3_next_development_queue_rebased']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_MA3_NEXT_DEVELOPMENT_QUEUE_ROWS,
        buildMa3NextDevelopmentQueueRebase,
        formatMa3NextDevelopmentQueueRebaseText,
      } = await import("./studio_ma3_next_development_queue_rebase.js");
      const rebase = buildMa3NextDevelopmentQueueRebase({
        queueRows: DEFAULT_MA3_NEXT_DEVELOPMENT_QUEUE_ROWS,
      });
      return {
        rebase,
        text: formatMa3NextDevelopmentQueueRebaseText(rebase),
      };
    });

    const rebase = moduleResult.rebase;
    assert(rebase.__종류 === "studio_ma3_next_development_queue_rebase", "kind mismatch");
    assert(rebase.schema === "ddn.studio.ma3_next_development_queue_rebase.v1", "schema mismatch");
    assert(rebase.status === "ma3_next_development_queue_rebased", `status mismatch: ${rebase.status}`);
    assert(rebase.product_ui_change === true, "must claim product ui change");
    assert(rebase.product_code_change === true, "must claim product code change");
    assert(rebase.runtime_claim === false, "must not claim runtime");
    assert(rebase.new_automatic_queue_claim === false, "must not claim automatic queue");
    assert(rebase.release_execution_claim === false, "must not claim release execution");
    assert(rebase.public_upload_claim === false, "must not claim public upload");
    assert(rebase.benchmark_execution_claim === false, "must not claim benchmark execution");
    assert(rebase.queue_item_count === 8, `queue count mismatch: ${rebase.queue_item_count}`);
    assert(rebase.ready_stage_count === 6, `ready stage mismatch: ${rebase.ready_stage_count}`);
    assert(rebase.progress.super_long_behavior_closed === 18, "super-long closed mismatch");
    assert(rebase.progress.super_long_percent === 100, "super-long percent mismatch");
    assert(rebase.progress.previous_stage_percent === 100, "previous stage percent mismatch");
    assert(rebase.progress.current_stage_closed === 1, "new queue closed mismatch");
    assert(rebase.progress.current_stage_percent === 13, "new queue percent mismatch");
    assert(rebase.progress.roadmap_v2_behavior_closed === 89, "roadmap closed mismatch");
    assert(rebase.progress.roadmap_v2_percent === 99, "roadmap percent mismatch");
    assert(rebase.next_item === "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1", "next item mismatch");
    assert(String(moduleResult.text).includes("new_queue\t1/8"), "formatted text missing queue progress");
    assert(String(moduleResult.text).includes("release_execution_claim\tfalse"), "formatted text missing release boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-ma3-next-development-queue-rebase]");
      const buttons = Array.from(document.querySelectorAll("[data-ma3-dev-queue]"));
      const progress = document.querySelector("[data-ma3-dev-queue-progress]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-ma3-dev-queue") === "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1")?.click();
      const title = document.querySelector("[data-ma3-dev-queue-active-title]")?.textContent || "";
      const status = document.querySelector("[data-ma3-dev-queue-active-status]")?.textContent || "";
      const coordinate = document.querySelector("[data-ma3-dev-queue-active-coordinate]")?.textContent || "";
      document.querySelector("[data-ma3-dev-queue-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-ma3-next-development-queue-status") || "",
        copied: root?.getAttribute("data-ma3-next-development-queue-copied") || "",
        buttonCount: buttons.length,
        progress,
        title,
        status,
        coordinate,
        globalSchema: window.__SEAMGRIM_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE__?.schema || "",
        globalText: window.__SEAMGRIM_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "ma3_next_development_queue_rebased", `dom status mismatch: ${domResult.rootStatus}`);
    assert(domResult.buttonCount === 8, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("18/18 overall") && domResult.progress.includes("100%"), `overall progress mismatch: ${domResult.progress}`);
    assert(domResult.progress.includes("1/8 new queue") && domResult.progress.includes("13%"), `queue progress mismatch: ${domResult.progress}`);
    assert(domResult.title === "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1", `title mismatch: ${domResult.title}`);
    assert(domResult.status === "next", `status mismatch: ${domResult.status}`);
    assert(domResult.coordinate === "하-3", `coordinate mismatch: ${domResult.coordinate}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.ma3_next_development_queue_rebase.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("queue_id\tcoordinate"), "global text missing queue header");

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
