#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "free_lab_first_run: ok";

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

async function waitVisible(page, selector) {
  await page.waitForFunction((sel) => {
    const node = document.querySelector(sel);
    return node && !String(node.className ?? "").split(/\s+/).includes("hidden");
  }, selector);
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "free_lab_first_run.js",
    "screens/browse.js",
    "screens/run.js",
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
    await page.waitForSelector("[data-free-lab-first-run][data-free-lab-first-run-status='first_run_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_FREE_LAB_FIRST_RUN_ROWS,
        buildFreeLabFirstRun,
        formatFreeLabFirstRunText,
      } = await import("./free_lab_first_run.js");
      const firstRun = buildFreeLabFirstRun({ rows: DEFAULT_FREE_LAB_FIRST_RUN_ROWS });
      return {
        firstRun,
        text: formatFreeLabFirstRunText(firstRun),
      };
    });
    const firstRun = moduleResult.firstRun;
    assert(firstRun.schema === "ddn.seamgrim.free_lab.first_run.v1", "schema mismatch");
    assert(firstRun.work_item === "BA1_FREE_LAB_FIRST_RUN_V1", "work item mismatch");
    assert(firstRun.primary_coordinate === "바-1", "coordinate mismatch");
    assert(firstRun.status === "first_run_ready", "status mismatch");
    assert(firstRun.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(firstRun.first_run_claim === true, "first-run claim mismatch");
    assert(firstRun.product_ui_change === true, "product UI change mismatch");
    assert(firstRun.runtime_claim === false, "runtime claim must stay false");
    assert(firstRun.share_claim === false, "share claim must stay false");
    assert(firstRun.registry_publish_claim === false, "registry publish claim must stay false");
    assert(firstRun.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(firstRun.progress.current_stage_total === 5, "stage total mismatch");
    assert(firstRun.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(firstRun.progress.roadmap_v2_matrix_behavior_closed === 8, "roadmap closed mismatch");
    assert(firstRun.progress.roadmap_v2_matrix_behavior_percent === 9, "roadmap percent mismatch");
    assert(firstRun.progress.roadmap_v2_pack_evidence_reference_closed === 27, "pack ref mismatch");
    assert(firstRun.progress.roadmap_v2_pack_evidence_reference_percent === 30, "pack ref percent mismatch");
    assert(firstRun.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(firstRun.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(String(moduleResult.text).includes("share_claim\tfalse"), "text missing share boundary");
    assert(String(moduleResult.text).includes("roadmap_matrix\t8/90"), "text missing matrix progress");

    const beforeOpen = await page.evaluate(() => ({
      status: document.querySelector("[data-free-lab-first-run]")?.getAttribute("data-free-lab-first-run-status") || "",
      progress: document.querySelector("[data-free-lab-progress]")?.textContent || "",
      summary: document.querySelector("[data-free-lab-summary]")?.textContent || "",
      title: document.querySelector("[data-free-lab-active-title]")?.textContent || "",
      globalSchema: window.__SEAMGRIM_FREE_LAB_FIRST_RUN__?.schema || "",
      globalText: window.__SEAMGRIM_FREE_LAB_FIRST_RUN_TEXT__ || "",
    }));
    assert(beforeOpen.status === "first_run_ready", `DOM status mismatch: ${beforeOpen.status}`);
    assert(beforeOpen.progress.includes("8/90 ROADMAP") && beforeOpen.progress.includes("5/5 stage"), `DOM progress mismatch: ${beforeOpen.progress}`);
    assert(beforeOpen.summary.includes("빈 캔버스"), "DOM summary missing first-run copy");
    assert(beforeOpen.title === "새 실험", `active title mismatch: ${beforeOpen.title}`);
    assert(beforeOpen.globalSchema === "ddn.seamgrim.free_lab.first_run.v1", "global schema mismatch");
    assert(beforeOpen.globalText.includes("pack_evidence_reference\t27/90"), "global text missing pack reference");

    await page.click("[data-free-lab-open-first-run]");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => {
      const preview = document.querySelector("#run-ddn-preview");
      return String(preview?.value ?? "").includes("자유_실험_첫실행")
        && String(preview?.value ?? "").includes("계수:수 <- 2")
        && String(preview?.value ?? "").includes("(매마디)마다");
    });
    const runState = await page.evaluate(() => ({
      browseHidden: document.querySelector("#screen-browse")?.classList.contains("hidden"),
      runHidden: document.querySelector("#screen-run")?.classList.contains("hidden"),
      sourceLabel: document.querySelector("#studio-source-label")?.textContent?.trim(),
      runTitle: document.querySelector("#run-lesson-title")?.textContent?.trim(),
      saveStatusKind: document.querySelector("#run-local-save-status")?.dataset?.status,
      previewValue: document.querySelector("#run-ddn-preview")?.value || "",
      lastOpen: window.__SEAMGRIM_FREE_LAB_LAST_OPEN__ || null,
    }));
    assert(runState.browseHidden === true, "browse should hide after free lab open");
    assert(runState.runHidden === false, "run screen should show after free lab open");
    assert(runState.sourceLabel === "자유 실험 첫실행", `source label mismatch: ${runState.sourceLabel}`);
    assert(runState.runTitle === "자유 실험 첫실행", `run title mismatch: ${runState.runTitle}`);
    assert(runState.saveStatusKind === "idle", `save status mismatch: ${runState.saveStatusKind}`);
    assert(runState.previewValue.includes("결과 보여주기"), "free lab DDN missing observation output");
    assert(runState.lastOpen?.work_item === "BA1_FREE_LAB_FIRST_RUN_V1", "last open instrumentation mismatch");

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
