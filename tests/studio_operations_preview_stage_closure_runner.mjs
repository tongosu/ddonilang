#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_operations_preview_stage_closure: ok";

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
  return listenOnSafePort(server, 17690);
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
  for (const rel of ["index.html", "app.js", "studio_operations_preview_stage_closure.js"]) {
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
    await page.waitForSelector("[data-operations-preview-stage-closure][data-operations-preview-stage-status='operations_preview_stage_closed']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_OPERATIONS_PREVIEW_STAGE_CLOSURE_ROWS,
        buildOperationsPreviewStageClosure,
        formatOperationsPreviewStageClosureText,
      } = await import("./studio_operations_preview_stage_closure.js");
      const closure = buildOperationsPreviewStageClosure({ closureRows: DEFAULT_OPERATIONS_PREVIEW_STAGE_CLOSURE_ROWS });
      return {
        closure,
        text: formatOperationsPreviewStageClosureText(closure),
      };
    });

    const closure = moduleResult.closure;
    assert(closure.__종류 === "studio_operations_preview_stage_closure", "closure kind mismatch");
    assert(closure.schema === "ddn.studio.operations_preview_stage_closure.v1", "closure schema mismatch");
    assert(closure.product_ui_change === true, "closure must claim product ui change");
    assert(closure.runtime_claim === false, "closure must not claim runtime");
    assert(closure.new_automatic_queue_claim === false, "closure must not claim new automatic queue");
    assert(closure.release_execution_claim === false, "closure must not claim release execution");
    assert(closure.public_upload_claim === false, "closure must not claim public upload");
    assert(closure.status === "operations_preview_stage_closed", `status mismatch: ${closure.status}`);
    assert(closure.closure_row_count === 8, `closure count mismatch: ${closure.closure_row_count}`);
    assert(closure.ready_stage_count === 6, `ready stage mismatch: ${closure.ready_stage_count}`);
    assert(closure.progress.super_long_behavior_closed === 18, "super-long closed mismatch");
    assert(closure.progress.super_long_percent === 100, "super-long percent mismatch");
    assert(closure.progress.current_stage_closed === 8, "current stage closed mismatch");
    assert(closure.progress.current_stage_percent === 100, "current stage percent mismatch");
    assert(closure.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(closure.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("current_stage_percent\t100"), "formatted text missing stage percent");
    assert(String(moduleResult.text).includes("release_execution_claim\tfalse"), "formatted text missing release boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-operations-preview-stage-closure]");
      const buttons = Array.from(document.querySelectorAll("[data-operations-stage-closure]"));
      const firstProgress = document.querySelector("[data-operations-stage-closure-progress]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-operations-stage-closure") === "ma3_next_queue_coordinate_lock")?.click();
      const lockTitle = document.querySelector("[data-operations-stage-closure-active-title]")?.textContent || "";
      const lockLane = document.querySelector("[data-operations-stage-closure-active-lane]")?.textContent || "";
      const lockCoordinate = document.querySelector("[data-operations-stage-closure-active-coordinate]")?.textContent || "";
      document.querySelector("[data-operations-stage-closure-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-operations-preview-stage-status") || "",
        copied: root?.getAttribute("data-operations-preview-stage-copied") || "",
        buttonCount: buttons.length,
        firstProgress,
        lockTitle,
        lockLane,
        lockCoordinate,
        globalSchema: window.__SEAMGRIM_OPERATIONS_PREVIEW_STAGE_CLOSURE__?.schema || "",
        globalText: window.__SEAMGRIM_OPERATIONS_PREVIEW_STAGE_CLOSURE_TEXT__ || "",
      };
    });
    assert(domResult.status === "operations_preview_stage_closed", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 8, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.firstProgress.includes("8/8") && domResult.firstProgress.includes("100%"), `progress text mismatch: ${domResult.firstProgress}`);
    assert(domResult.lockTitle.includes("좌표 잠금"), `lock title mismatch: ${domResult.lockTitle}`);
    assert(domResult.lockLane === "coordinate_lock", `lock lane mismatch: ${domResult.lockLane}`);
    assert(domResult.lockCoordinate === "마-3", `lock coordinate mismatch: ${domResult.lockCoordinate}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.operations_preview_stage_closure.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("closure_id\twork_item"), "global text missing closure header");

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
