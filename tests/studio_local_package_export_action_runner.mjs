#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_local_package_export_action: ok";
const LEGACY_LOCAL_PACKAGE_COPY_BUTTON_ID = "btn-run-local-package-copy";

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
  if (file.endsWith(".toml") || file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  if (file.endsWith(".md")) return "text/markdown; charset=utf-8";
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
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if ((pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/")) && /\/(?:graph|table|space2d|text|maegim_control)\.(?:json|md)$/i.test(pathname)) return true;
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
  for (const rel of ["index.html", "app.js", "styles.css", "screens/browse.js", "screens/run.js", "studio_local_share_package.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR", acceptDownloads: true });
    await context.addInitScript(() => {
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          async writeText(value) {
            window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ = String(value ?? "");
          },
        },
      });
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      const text = String(msg.text() ?? "");
      if (msg.type() === "error" && !text.includes("Failed to load resource") && !text.includes("[RunScreen.restart] wasm execution failed")) {
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
    await waitVisible(page, "#screen-browse");
    const defaultDevSurfaceState = await page.evaluate(() => ({
      bodyEnabled: document.body.classList.contains("dev-surfaces-enabled"),
      devRootExists: Boolean(document.querySelector("#dev-surface-root")),
      templateExists: Boolean(document.querySelector("#dev-surface-template")),
      advancedBrowseDisplay: getComputedStyle(document.querySelector("#btn-advanced-browse")).display,
      advancedEditorDisplay: getComputedStyle(document.querySelector("#btn-advanced-editor")).display,
      advancedRunDisplay: getComputedStyle(document.querySelector("#btn-advanced-run")).display,
      numericTrackDisplay: getComputedStyle(document.querySelector("#btn-filter-numeric-track")).display,
      numericResultsDisplay: getComputedStyle(document.querySelector("#btn-filter-numeric-track-results")).display,
      featuredSeedDisplay: getComputedStyle(document.querySelector("#btn-preset-featured-seed-quick-recent")).display,
      numericSummaryDisplay: getComputedStyle(document.querySelector("#btn-copy-numeric-track-result-summary")).display,
      numericTimelineDisplay: getComputedStyle(document.querySelector("#btn-toggle-numeric-track-result-timeline")).display,
      presetLinkDisplay: getComputedStyle(document.querySelector("#btn-copy-browse-preset-link")).display,
      seedScopeDisplay: getComputedStyle(document.querySelector("#filter-seed-scope")).display,
      runStatusDisplay: getComputedStyle(document.querySelector("#filter-run-status")).display,
      runLaunchDisplay: getComputedStyle(document.querySelector("#filter-run-launch")).display,
      warningStatusDisplay: getComputedStyle(document.querySelector("#filter-warning-status")).display,
      sortDisplay: getComputedStyle(document.querySelector("#filter-sort")).display,
    }));
    assert(defaultDevSurfaceState.templateExists === false, "dev surface template should not ship in the default teacher UI");
    assert(defaultDevSurfaceState.bodyEnabled === false, "dev surfaces should not be enabled by default");
    assert(defaultDevSurfaceState.devRootExists === false, "dev surface root should not mount on the default teacher UI");
    assert(defaultDevSurfaceState.advancedBrowseDisplay === "none", `advanced browse button should be hidden by default: ${defaultDevSurfaceState.advancedBrowseDisplay}`);
    assert(defaultDevSurfaceState.advancedEditorDisplay === "none", `advanced editor button should be hidden by default: ${defaultDevSurfaceState.advancedEditorDisplay}`);
    assert(defaultDevSurfaceState.advancedRunDisplay === "none", `advanced run button should be hidden by default: ${defaultDevSurfaceState.advancedRunDisplay}`);
    assert(defaultDevSurfaceState.numericTrackDisplay === "none", `numeric track filter should be hidden by default: ${defaultDevSurfaceState.numericTrackDisplay}`);
    assert(defaultDevSurfaceState.numericResultsDisplay === "none", `numeric result filter should be hidden by default: ${defaultDevSurfaceState.numericResultsDisplay}`);
    assert(defaultDevSurfaceState.featuredSeedDisplay === "none", `featured seed shortcut should be hidden by default: ${defaultDevSurfaceState.featuredSeedDisplay}`);
    assert(defaultDevSurfaceState.numericSummaryDisplay === "none", `numeric summary export should be hidden by default: ${defaultDevSurfaceState.numericSummaryDisplay}`);
    assert(defaultDevSurfaceState.numericTimelineDisplay === "none", `numeric timeline should be hidden by default: ${defaultDevSurfaceState.numericTimelineDisplay}`);
    assert(defaultDevSurfaceState.presetLinkDisplay === "none", `preset link copy should be hidden by default: ${defaultDevSurfaceState.presetLinkDisplay}`);
    assert(defaultDevSurfaceState.seedScopeDisplay === "none", `seed scope filter should be hidden by default: ${defaultDevSurfaceState.seedScopeDisplay}`);
    assert(defaultDevSurfaceState.runStatusDisplay === "none", `run status filter should be hidden by default: ${defaultDevSurfaceState.runStatusDisplay}`);
    assert(defaultDevSurfaceState.runLaunchDisplay === "none", `run launch filter should be hidden by default: ${defaultDevSurfaceState.runLaunchDisplay}`);
    assert(defaultDevSurfaceState.warningStatusDisplay === "none", `legacy warning filter should be hidden by default: ${defaultDevSurfaceState.warningStatusDisplay}`);
    assert(defaultDevSurfaceState.sortDisplay === "none", `sort filter should be hidden by default: ${defaultDevSurfaceState.sortDisplay}`);
    await page.waitForSelector(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='student']");
    await page.click(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='student']");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.onboarding_profile === "student");
    await page.click("[data-classroom-mode='teacher']");
    await page.waitForFunction(() => window.__STUDIO_CLASSROOM_MODE_SWITCH__?.mode === "teacher");
    await page.click("#run-tab-btn-mirror");
    await page.evaluate(() => {
      const tools = document.querySelector("#run-inspector-tools");
      if (tools) tools.open = true;
    });
    await page.click("#btn-run-teacher-package-copy");
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_EXPORT_ACTION__?.copied === true);
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-teacher-package-download"),
    ]);
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_DOWNLOAD_ACTION__?.downloaded === true);

    const state = await page.evaluate(() => ({
      schema: document.querySelector("[data-run-local-package-export]")?.dataset?.schema ?? "",
      state: document.querySelector("[data-run-local-package-export]")?.dataset?.state ?? "",
      packageId: document.querySelector("[data-run-local-package-export]")?.dataset?.packageId ?? "",
      meta: document.querySelector("[data-run-local-package-meta]")?.textContent?.trim() ?? "",
      metaValue: document.querySelector("[data-run-local-package-meta]")?.dataset?.value ?? "",
      indexText: document.querySelector("[data-run-local-package-text]")?.textContent ?? "",
      saveButtonText: document.querySelector("#btn-run-teacher-package-download")?.textContent?.trim() ?? "",
      copied: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      payload: window.__STUDIO_LOCAL_PACKAGE_EXPORT_ACTION__ ?? null,
      downloadPayload: window.__STUDIO_LOCAL_PACKAGE_DOWNLOAD_ACTION__ ?? null,
    }));
    const copiedPayload = JSON.parse(state.copied);
    const downloadedPath = await download.path();
    const downloadedText = downloadedPath ? await fs.readFile(downloadedPath, "utf-8") : "";
    const downloadedPayload = JSON.parse(downloadedText);

    assert(state.schema === "seamgrim.local_package_export_action.v1", `schema mismatch: ${state.schema}`);
    assert(state.state === "ready", `state mismatch: ${state.state}`);
    assert(state.packageId.startsWith("studio.local."), `package id mismatch: ${state.packageId}`);
    assert(state.meta.includes("교사 시작"), `meta mismatch: ${state.meta}`);
    assert(Number(state.metaValue) >= 5, `meta value mismatch: ${state.metaValue}`);
    assert(state.indexText.startsWith("구분\t경로\t제목\t크기"), "index header mismatch");
    assert(state.indexText.includes("\nlesson\tlessons/"), `lesson index row missing:\n${state.indexText}`);
    assert(state.indexText.includes("\nreport\treports/"), `report index row missing:\n${state.indexText}`);
    assert(state.saveButtonText.startsWith("배포 저장 "), `save button text mismatch: ${state.saveButtonText}`);
    assert(copiedPayload.__종류 === "studio_local_package_payload", "clipboard payload kind mismatch");
    assert(copiedPayload.manifest?.__종류 === "studio_local_package_manifest", "clipboard manifest kind mismatch");
    assert(copiedPayload.manifest?.account_required === false, "account boundary mismatch");
    assert(copiedPayload.manifest?.cloud_sync === false, "cloud boundary mismatch");
    assert(copiedPayload.manifest?.public_registry === false, "registry boundary mismatch");
    assert(state.payload?.schema === "seamgrim.local_package_export_action.v1", "instrumentation schema mismatch");
    assert(state.payload?.copied === true, "instrumentation copied mismatch");
    assert(state.payload?.lesson_count === 1 && state.payload?.report_count === 1, "instrumentation counts mismatch");
    assert(state.payload?.account_required === false && state.payload?.cloud_sync === false && state.payload?.public_registry === false && state.payload?.remote_save === false, "instrumentation boundary mismatch");
    assert(state.payload?.payload_text.trim() === state.copied, "payload text clipboard mismatch");
    assert(state.downloadPayload?.downloaded === true, "download instrumentation mismatch");
    assert(String(state.downloadPayload?.file_name ?? "").endsWith(".json"), "download filename mismatch");
    assert(download.suggestedFilename() === state.downloadPayload.file_name, "suggested filename mismatch");
    assert(downloadedPayload.__종류 === "studio_local_package_payload", "downloaded payload kind mismatch");
    assert(downloadedText.trim() === state.copied, "downloaded payload text mismatch");

    await page.click("#screen-run:not(.hidden) .main-shell-tab[data-main-tab-target='browse']");
    await waitVisible(page, "#screen-browse");
    await page.setInputFiles("#input-local-package-file", downloadedPath);
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true);
    await waitVisible(page, "#screen-run");
    const importState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      rail: window.__SEAMGRIM_RUN_PRESET_RAIL__ ?? null,
      runText: document.querySelector("#run-ddn-preview")?.value ?? "",
      sourceLabel: document.querySelector("[data-shell-source-label]")?.textContent?.trim() ?? "",
      classroomMode: document.querySelector("[data-classroom-mode-switch]")?.dataset?.mode ?? "",
      studentPressed: document.querySelector("[data-classroom-mode='student']")?.getAttribute("aria-pressed") ?? "",
      teacherPressed: document.querySelector("[data-classroom-mode='teacher']")?.getAttribute("aria-pressed") ?? "",
    }));
    assert(importState.importAction?.schema === "seamgrim.local_package_import_action.v1", "import action schema mismatch");
    assert(importState.importAction?.package_id === downloadedPayload.manifest.package_id, "import package id mismatch");
    assert(importState.importAction?.lesson_id === downloadedPayload.lessons[0].lesson_id, "import lesson id mismatch");
    assert(importState.importAction?.account_required === false && importState.importAction?.cloud_sync === false && importState.importAction?.public_registry === false, "import boundary mismatch");
    assert(importState.rail?.launch_kind === "local_package_import", `import launch kind mismatch: ${importState.rail?.launch_kind}`);
    assert(importState.rail?.launch_label === "배포 열기", `import launch label mismatch: ${importState.rail?.launch_label}`);
    assert(importState.rail?.onboarding_profile === "student", `import onboarding mismatch: ${importState.rail?.onboarding_profile}`);
    assert(importState.classroomMode === "student" && importState.studentPressed === "true" && importState.teacherPressed === "false", "import should open in student mode");
    assert(importState.runText.trim() === String(downloadedPayload.lessons[0].source_text ?? "").trim(), "imported run source mismatch");
    assert(importState.sourceLabel.includes(downloadedPayload.lessons[0].title), `imported source label mismatch: ${importState.sourceLabel}`);

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
