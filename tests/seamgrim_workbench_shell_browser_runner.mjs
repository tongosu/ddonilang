#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_workbench_shell_browser: ok";
const UNSAFE_BROWSER_PORTS = new Set([
  1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
  101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161,
  179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563,
  587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
  6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
]);

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) {
    throw new Error(`missing file: ${file}`);
  }
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".toml")) return "text/plain; charset=utf-8";
  if (file.endsWith(".md")) return "text/markdown; charset=utf-8";
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  return "application/octet-stream";
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
      const rel = rawPath.replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, rel);
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, {
        "content-type": mimeType(file),
        "cache-control": "no-store",
        "access-control-allow-origin": "*",
      });
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
      if (!address || typeof address === "string") {
        reject(new Error("failed to bind static server"));
        return;
      }
      if (UNSAFE_BROWSER_PORTS.has(address.port)) {
        server.close(() => {
          createServer(root).then(resolve, reject);
        });
        return;
      }
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

function isHiddenByClass(className) {
  return String(className ?? "").split(/\s+/).includes("hidden");
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    const pathname = url.pathname;
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") {
      return true;
    }
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d)\.json$/i.test(pathname)
    ) {
      return true;
    }
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/text\.md$/i.test(pathname)
    ) {
      return true;
    }
  } catch (_) {
    return false;
  }
  return false;
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
    "screens/browse.js",
    "screens/editor.js",
    "screens/run.js",
    "lesson_loader_contract.js",
    "studio_edit_run_contract.js",
    "runtime/wasm_vm_runtime.js",
    "block_editor/ddn_block_codec.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  const responses = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
      viewport: { width: 1360, height: 860 },
      locale: "ko-KR",
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        if (String(msg.text() ?? "").includes("Failed to load resource")) {
          return;
        }
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => {
      failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`);
    });
    page.on("response", (res) => {
      responses.push({ status: res.status(), url: res.url() });
      if (res.status() >= 400) {
        if (res.status() === 404 && isAllowedFallback404(res.url())) {
          return;
        }
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    const url = `${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`;
    await page.goto(url, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-browse");
    await page.waitForSelector("#lesson-card-grid");

    const initial = await page.evaluate(() => ({
      title: document.title,
      browseHidden: document.querySelector("#screen-browse")?.classList.contains("hidden"),
      editorHidden: document.querySelector("#screen-editor")?.classList.contains("hidden"),
      runHidden: document.querySelector("#screen-run")?.classList.contains("hidden"),
      brandCount: document.querySelectorAll(".brand").length,
      mainTabCount: document.querySelectorAll(".main-shell-tab[data-main-tab-target]").length,
      hasCreate: Boolean(document.querySelector("#btn-create")),
      hasRunControl: Boolean(document.querySelector(".run-control-bar")),
      hasErrorBanner: Boolean(document.querySelector("#run-error-banner")),
      hasRunWarningLogin: Boolean(document.querySelector("#btn-run-warning-platform-login")),
      hasRunWarningAccess: Boolean(document.querySelector("#btn-run-warning-platform-request-access")),
      hasLocalSave: Boolean(document.querySelector("#btn-run-warning-platform-open-local-save")),
    }));
    assert(initial.title === "셈그림", "document title mismatch");
    assert(initial.browseHidden === false, "browse screen should be visible at startup");
    assert(initial.editorHidden === true, "editor screen should be hidden at startup");
    assert(initial.runHidden === true, "run screen should be hidden at startup");
    assert(initial.brandCount >= 3, "shell brand markers missing");
    assert(initial.mainTabCount >= 6, "main shell tab buttons missing");
    assert(initial.hasCreate, "create button missing");
    assert(initial.hasRunControl, "run control bar missing");
    assert(initial.hasErrorBanner, "run error banner missing");
    assert(initial.hasRunWarningLogin, "platform login warning anchor missing");
    assert(initial.hasRunWarningAccess, "platform access warning anchor missing");
    assert(initial.hasLocalSave, "platform local save warning anchor missing");

    await page.click('#screen-browse .main-shell-tab[data-main-tab-target="studio"]');
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => {
      const source = document.querySelector("#run-ddn-preview");
      return source && String(source.value ?? "").includes("(매마디)마다");
    });

    const runShell = await page.evaluate(() => ({
      browseHidden: document.querySelector("#screen-browse")?.classList.contains("hidden"),
      runHidden: document.querySelector("#screen-run")?.classList.contains("hidden"),
      editorHidden: document.querySelector("#screen-editor")?.classList.contains("hidden"),
      sourceLabel: document.querySelector("#studio-source-label")?.textContent?.trim(),
      runTitle: document.querySelector("#run-lesson-title")?.textContent?.trim(),
      sourceLength: document.querySelector("#run-ddn-preview")?.value?.length ?? 0,
      canvasExists: Boolean(document.querySelector("#canvas-bogae")),
      bogaeFrameExists: Boolean(document.querySelector(".bogae-frame")),
      subpanelTabs: Array.from(document.querySelectorAll(".run-tab-btn[data-run-tab]")).map((el) => el.dataset.runTab),
      activeSubpanelTabs: Array.from(document.querySelectorAll(".run-tab-btn.active[data-run-tab]")).map((el) => el.dataset.runTab),
      graphPanelExists: Boolean(document.querySelector("#run-tab-panel-graph")),
    }));
    assert(runShell.browseHidden === true, "browse screen should hide after workbench open");
    assert(runShell.runHidden === false, "run screen should be visible after workbench open");
    assert(runShell.editorHidden === true, "editor screen should remain hidden after workbench open");
    assert(runShell.sourceLabel === "새 작업", "studio source label mismatch");
    assert(runShell.runTitle === "새 작업", "run lesson title mismatch");
    assert(runShell.sourceLength > 20, "run DDN draft missing");
    assert(runShell.canvasExists, "Bogae canvas missing");
    assert(runShell.bogaeFrameExists, "Bogae frame missing");
    assert(JSON.stringify(runShell.subpanelTabs) === JSON.stringify(["console", "output", "mirror", "graph", "overlay"]), "run subpanel tab order mismatch");
    assert(runShell.graphPanelExists, "graph tab panel missing");
    assert(runShell.activeSubpanelTabs.length === 1, "exactly one run subpanel tab should be active");

    await page.click("#btn-run-view-analyze");
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.studioViewMode === "analyze");
    await page.click("#btn-run-view-full");
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.studioViewMode === "full");
    await page.click("#btn-run-view-basic");
    await page.waitForFunction(() => {
      const mode = document.querySelector("#screen-run")?.dataset?.studioViewMode;
      return !mode || mode === "basic";
    });

    await page.click("#btn-studio-new");
    await waitVisible(page, "#screen-run");
    const editorShell = await page.evaluate(() => ({
      editorHidden: String(document.querySelector("#screen-editor")?.className ?? "").split(/\s+/).includes("hidden"),
      runHidden: String(document.querySelector("#screen-run")?.className ?? "").split(/\s+/).includes("hidden"),
      readinessStage: document.querySelector("#editor-readiness-stage")?.textContent?.trim(),
      readinessAction: document.querySelector("#btn-editor-readiness-action")?.textContent?.trim(),
      hasTextarea: Boolean(document.querySelector("#ddn-textarea")),
      hasCanonPanel: Boolean(document.querySelector("#editor-canon-panel")),
    }));
    assert(editorShell.editorHidden === true, "editor screen should stay hidden during run-shell new work reset");
    assert(editorShell.runHidden === false, "run screen should stay visible after new work reset");
    assert(Boolean(editorShell.readinessStage), "editor readiness stage missing");
    assert(Boolean(editorShell.readinessAction), "editor readiness action missing");
    assert(editorShell.hasTextarea, "editor textarea missing");
    assert(editorShell.hasCanonPanel, "editor canon panel missing");

    await page.click('#screen-run .main-shell-tab[data-main-tab-target="browse"]');
    await waitVisible(page, "#screen-browse");
    const finalState = await page.evaluate(() => ({
      browseHidden: String(document.querySelector("#screen-browse")?.className ?? "").split(/\s+/).includes("hidden"),
      editorHidden: String(document.querySelector("#screen-editor")?.className ?? "").split(/\s+/).includes("hidden"),
      runHidden: String(document.querySelector("#screen-run")?.className ?? "").split(/\s+/).includes("hidden"),
      activeBrowseTabs: document.querySelectorAll('#screen-browse .main-shell-tab.active[data-main-tab-target="browse"]').length,
    }));
    assert(finalState.browseHidden === false, "browse screen should be visible after returning");
    assert(finalState.editorHidden === true, "editor screen should hide after returning");
    assert(finalState.runHidden === true, "run screen should hide after returning");
    assert(finalState.activeBrowseTabs === 1, "browse main tab active state mismatch");

    assert(responses.some((row) => row.url.endsWith("/solutions/seamgrim_ui_mvp/ui/app.js")), "app.js response missing");
    assert(responses.some((row) => row.url.endsWith("/solutions/seamgrim_ui_mvp/ui/styles.css?v=20260503-view-mode-split")), "styles.css response missing");
    if (failures.length > 0) {
      throw new Error(failures.join("\n"));
    }
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
