#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_lesson_authoring_flow_browser: ok";
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

function base64UrlUtf8(text) {
  return Buffer.from(String(text ?? ""), "utf8")
    .toString("base64")
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/g, "");
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
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const sourceDdn = [
    '설정 { 제목: "저자_스모크". 설명: "lesson authoring smoke". }.',
    "채비 { 프레임수:수 <- 0. t:수 <- 0. y:수 <- 0. }.",
    "(매마디)마다 { t <- 프레임수. y <- t. y 보여주기. 프레임수 <- (프레임수 + 1). }.",
  ].join("\n");
  const editedDdn = `${sourceDdn}\n"수정_스모크" 보여주기.`;

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
    await context.addInitScript(() => {
      window.__lessonAuthoringSmoke = {
        downloads: [],
        objectUrls: [],
        revoked: [],
      };
      const originalCreateObjectUrl = URL.createObjectURL.bind(URL);
      URL.createObjectURL = (blob) => {
        const store = window.__lessonAuthoringSmoke;
        const href = `blob:lesson-authoring-smoke/${store.objectUrls.length}`;
        store.objectUrls.push({
          href,
          type: String(blob?.type ?? ""),
          size: Number(blob?.size ?? 0),
        });
        try {
          originalCreateObjectUrl(blob);
        } catch (_) {
          // The smoke only needs to verify that the existing save path requested a blob URL.
        }
        return href;
      };
      URL.revokeObjectURL = (href) => {
        window.__lessonAuthoringSmoke.revoked.push(String(href ?? ""));
      };
      HTMLAnchorElement.prototype.click = function click() {
        window.__lessonAuthoringSmoke.downloads.push({
          href: String(this.href ?? ""),
          download: String(this.download ?? ""),
        });
        return undefined;
      };
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

    const route = `${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&ddn=${base64UrlUtf8(sourceDdn)}`;
    await page.goto(route, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => {
      const preview = document.querySelector("#run-ddn-preview");
      return preview && String(preview.value ?? "").includes("저자_스모크");
    });

    const directRouteState = await page.evaluate(() => ({
      browseHidden: document.querySelector("#screen-browse")?.classList.contains("hidden"),
      runHidden: document.querySelector("#screen-run")?.classList.contains("hidden"),
      editorHidden: document.querySelector("#screen-editor")?.classList.contains("hidden"),
      sourceLabel: document.querySelector("#studio-source-label")?.textContent?.trim(),
      previewValue: document.querySelector("#run-ddn-preview")?.value ?? "",
      hasSaveButton: Boolean(document.querySelector("#btn-ddn-save")),
      hasRunButton: Boolean(document.querySelector("#btn-run")),
      hasEditorSave: Boolean(document.querySelector("#btn-save-ddn")),
      hasEditorRun: Boolean(document.querySelector("#btn-run-from-editor")),
      hasLessonLoaderContract: Boolean(document.querySelector("script[type='module']")),
    }));
    assert(directRouteState.browseHidden === true, "browse screen should hide for direct DDN route");
    assert(directRouteState.runHidden === false, "run screen should be visible for direct DDN route");
    assert(directRouteState.editorHidden === true, "editor screen should stay hidden for direct DDN route");
    assert(Boolean(directRouteState.sourceLabel), "direct DDN source label missing");
    assert(directRouteState.previewValue.includes("저자_스모크"), "direct DDN text missing title marker");
    assert(directRouteState.previewValue.includes("(매마디)마다"), "direct DDN text missing tick block");
    assert(directRouteState.hasSaveButton, "run local save button missing");
    assert(directRouteState.hasRunButton, "run execution button missing");
    assert(directRouteState.hasEditorSave, "editor save anchor missing");
    assert(directRouteState.hasEditorRun, "editor run anchor missing");
    assert(directRouteState.hasLessonLoaderContract, "module script anchor missing");

    await page.fill("#run-ddn-preview", editedDdn);
    await page.waitForFunction(() => {
      const preview = document.querySelector("#run-ddn-preview");
      return String(preview?.value ?? "").includes("수정_스모크")
        && document.querySelector("#run-local-save-status")?.dataset?.status === "warn";
    });
    const editState = await page.evaluate(() => ({
      previewValue: document.querySelector("#run-ddn-preview")?.value ?? "",
      saveStatus: document.querySelector("#run-local-save-status")?.textContent?.trim(),
      saveStatusKind: document.querySelector("#run-local-save-status")?.dataset?.status,
    }));
    assert(editState.previewValue.includes("수정_스모크"), "run editor did not retain edited DDN text");
    assert(editState.saveStatusKind === "warn", "edit should mark local save status as warn");
    assert(editState.saveStatus === "저장 필요", "edit save status text mismatch");

    await page.click("#btn-ddn-save");
    await page.waitForFunction(() => document.querySelector("#run-local-save-status")?.dataset?.status === "ok");
    const saveState = await page.evaluate(() => ({
      saveStatus: document.querySelector("#run-local-save-status")?.textContent?.trim(),
      saveStatusKind: document.querySelector("#run-local-save-status")?.dataset?.status,
      downloads: window.__lessonAuthoringSmoke?.downloads ?? [],
      objectUrls: window.__lessonAuthoringSmoke?.objectUrls ?? [],
    }));
    assert(saveState.saveStatusKind === "ok", "local save should report ok");
    assert(saveState.saveStatus === "저장 완료", "local save status text mismatch");
    assert(saveState.downloads.length >= 1, "local save did not request anchor download");
    assert(saveState.downloads.some((row) => row.download === "lesson.ddn"), "local save download filename mismatch");
    assert(saveState.objectUrls.some((row) => row.type.includes("text/plain") && row.size > 0), "local save blob evidence missing");

    await page.click('#screen-run .main-shell-tab[data-main-tab-target="browse"]');
    await waitVisible(page, "#screen-browse");
    await page.click("#btn-create");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => {
      const preview = document.querySelector("#run-ddn-preview");
      return preview && String(preview.value ?? "").includes("(매마디)마다");
    });
    const createState = await page.evaluate(() => ({
      sourceLabel: document.querySelector("#studio-source-label")?.textContent?.trim(),
      runTitle: document.querySelector("#run-lesson-title")?.textContent?.trim(),
      previewValue: document.querySelector("#run-ddn-preview")?.value ?? "",
      saveStatusKind: document.querySelector("#run-local-save-status")?.dataset?.status,
    }));
    assert(createState.sourceLabel === "새 작업", "browse create source label mismatch");
    assert(createState.runTitle === "새 작업", "browse create run title mismatch");
    assert(createState.previewValue.includes("(매마디)마다"), "browse create draft missing DDN template");
    assert(createState.saveStatusKind === "idle", "browse create save status should be idle");

    assert(responses.some((row) => row.url.endsWith("/solutions/seamgrim_ui_mvp/ui/app.js")), "app.js response missing");
    assert(responses.some((row) => row.url.includes("/solutions/seamgrim_ui_mvp/ui/lesson_loader_contract.js")), "lesson loader contract response missing");
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
