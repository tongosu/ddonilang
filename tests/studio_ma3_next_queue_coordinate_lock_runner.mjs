#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_ma3_next_queue_coordinate_lock: ok";

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
  for (const rel of ["index.html", "app.js", "studio_ma3_next_queue_coordinate_lock.js"]) {
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
    await page.waitForSelector("[data-ma3-next-queue-coordinate-lock][data-ma3-coordinate-lock-status='ma3_next_queue_coordinate_lock_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_MA3_NEXT_QUEUE_LOCK_ROWS,
        buildMa3NextQueueCoordinateLock,
        formatMa3NextQueueCoordinateLockText,
      } = await import("./studio_ma3_next_queue_coordinate_lock.js");
      const lock = buildMa3NextQueueCoordinateLock({ lockRows: DEFAULT_MA3_NEXT_QUEUE_LOCK_ROWS });
      return {
        lock,
        text: formatMa3NextQueueCoordinateLockText(lock),
      };
    });

    const lock = moduleResult.lock;
    assert(lock.__종류 === "studio_ma3_next_queue_coordinate_lock", "lock kind mismatch");
    assert(lock.schema === "ddn.studio.ma3_next_queue_coordinate_lock.v1", "lock schema mismatch");
    assert(lock.product_ui_change === true, "lock must claim product ui change");
    assert(lock.runtime_claim === false, "lock must not claim runtime");
    assert(lock.new_automatic_queue_claim === false, "lock must not claim new automatic queue");
    assert(lock.release_execution_claim === false, "lock must not claim release execution");
    assert(lock.public_upload_claim === false, "lock must not claim public upload");
    assert(lock.selected_default_coordinate === "마-3", "selected coordinate mismatch");
    assert(lock.status === "ma3_next_queue_coordinate_lock_ready", `status mismatch: ${lock.status}`);
    assert(lock.lock_row_count === 6, `lock count mismatch: ${lock.lock_row_count}`);
    assert(lock.ready_stage_count === 6, `ready stage mismatch: ${lock.ready_stage_count}`);
    assert(lock.progress.super_long_behavior_closed === 9, "super-long closed mismatch");
    assert(lock.progress.super_long_percent === 50, "super-long percent mismatch");
    assert(lock.progress.current_stage_closed === 8, "current stage closed mismatch");
    assert(lock.progress.current_stage_percent === 100, "current stage percent mismatch");
    assert(lock.progress.roadmap_v2_behavior_closed === 51, "roadmap closed mismatch");
    assert(lock.progress.roadmap_v2_percent === 57, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("new_automatic_queue_claim\tfalse"), "formatted text missing queue boundary");
    assert(String(moduleResult.text).includes("selected_default_coordinate\t마-3"), "formatted text missing selected coordinate");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-ma3-next-queue-coordinate-lock]");
      const buttons = Array.from(document.querySelectorAll("[data-ma3-coordinate-lock]"));
      const firstTitle = document.querySelector("[data-ma3-coordinate-lock-active-title]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-ma3-coordinate-lock") === "next_stage_handoff_lock")?.click();
      const handoffTitle = document.querySelector("[data-ma3-coordinate-lock-active-title]")?.textContent || "";
      const handoffLane = document.querySelector("[data-ma3-coordinate-lock-active-lane]")?.textContent || "";
      const handoffCoordinate = document.querySelector("[data-ma3-coordinate-lock-active-coordinate]")?.textContent || "";
      document.querySelector("[data-ma3-coordinate-lock-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-ma3-coordinate-lock-status") || "",
        copied: root?.getAttribute("data-ma3-coordinate-lock-copied") || "",
        buttonCount: buttons.length,
        firstTitle,
        handoffTitle,
        handoffLane,
        handoffCoordinate,
        globalSchema: window.__SEAMGRIM_MA3_NEXT_QUEUE_COORDINATE_LOCK__?.schema || "",
        globalText: window.__SEAMGRIM_MA3_NEXT_QUEUE_COORDINATE_LOCK_TEXT__ || "",
      };
    });
    assert(domResult.status === "ma3_next_queue_coordinate_lock_ready", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 6, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.firstTitle.includes("마-3 좌표"), `first title mismatch: ${domResult.firstTitle}`);
    assert(domResult.handoffTitle.includes("다음 stage"), `handoff title mismatch: ${domResult.handoffTitle}`);
    assert(domResult.handoffLane === "stage_handoff", `handoff lane mismatch: ${domResult.handoffLane}`);
    assert(domResult.handoffCoordinate === "마-3", `handoff coordinate mismatch: ${domResult.handoffCoordinate}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.ma3_next_queue_coordinate_lock.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("lock_id\tcoordinate"), "global text missing lock header");

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
