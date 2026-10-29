#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_ui_v2_product_shell: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const PROJECTILE_ID = "rep_phys_projectile_xy_v1";

function assert(condition, message) {
  if (!condition) throw new Error(message);
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
      const rawPath = decodeURIComponent(url.pathname === "/" ? PROJECT_URL : url.pathname);
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
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
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|maegim_control)\.(?:json|detjson)$/i.test(pathname)
    ) return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/text\.md$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function installFailureCapture(page, failures) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    if (String(msg.text() ?? "").includes("Failed to load resource")) return;
    failures.push(`console error: ${msg.text()}`);
  });
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`response ${res.status()}: ${res.url()}`);
  });
}

async function readCanvasEvidence(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']");
    if (!(canvas instanceof HTMLCanvasElement)) return null;
    const rect = canvas.getBoundingClientRect();
    const ctx = canvas.getContext("2d");
    if (!ctx || canvas.width <= 0 || canvas.height <= 0) return { width: rect.width, height: rect.height, colors: 0 };
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    const colors = new Set();
    const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 4000) * 4);
    for (let index = 0; index < data.length; index += stride) {
      colors.add(`${data[index]}:${data[index + 1]}:${data[index + 2]}:${data[index + 3]}`);
      if (colors.size > 12) break;
    }
    return { width: rect.width, height: rect.height, colors: colors.size };
  });
}

async function waitForProjectileRun(page) {
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction(() => {
    const root = document.querySelector("#screen-run");
    return root?.dataset?.primaryViewContract === "1"
      && root?.dataset?.primaryViewSchemaId === "seamgrim.primary_view_rules"
      && root?.dataset?.primaryViewHeuristicSelection === "0"
      && ["debug-fallback", "space2d"].includes(String(root?.dataset?.mainVisualMode ?? ""));
  }, null, { timeout: 20_000 });
  await page.waitForSelector("[data-testid='lesson-control-panel']", { state: "visible", timeout: 20_000 });
}

async function verifyDesktopShell(page, baseUrl) {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse:not(.hidden)", { state: "visible" });
  await page.waitForSelector("#screen-browse .lesson-card", { state: "visible", timeout: 20_000 });

  const shell = await page.evaluate(() => ({
    labels: Array.from(document.querySelectorAll("#screen-browse .main-shell-tab"))
      .map((node) => node.textContent?.trim() ?? ""),
    cardCount: document.querySelectorAll("#screen-browse .lesson-card").length,
    visibleText: document.body.innerText,
    firstCardHasPreview: Boolean(document.querySelector("#screen-browse .lesson-card .card-structure-preview")),
    roleSwitchCount: document.querySelectorAll("[data-classroom-mode-switch]").length,
    roleButtonCount: document.querySelectorAll("[data-classroom-mode]").length,
  }));
  assert(JSON.stringify(shell.labels) === JSON.stringify(["배우기", "실험실", "만들기", "배포"]), "main shell labels mismatch");
  assert(shell.cardCount >= 3, `lesson catalog too small: ${shell.cardCount}`);
  assert(shell.firstCardHasPreview, "lesson card preview slot missing");
  assert(shell.roleSwitchCount === 0 && shell.roleButtonCount === 0, `runtime role toggle DOM exists in shell: ${JSON.stringify(shell)}`);
  for (const blocked of ["학생으로 실행", "교사용 배포 준비", "학생 시작", "교사용 배포"]) {
    assert(!shell.visibleText.includes(blocked), `legacy role entry is visible: ${blocked}`);
  }
  assert(shell.visibleText.includes("수업 열기"), "lesson open action missing");
  assert(shell.visibleText.includes("실험실에서 바꾸기"), "lab action missing");

  await page.locator(`#screen-browse [data-lesson-id='${PROJECTILE_ID}'] .card-launch-primary`).click();
  await waitForProjectileRun(page);
  const canvas = await readCanvasEvidence(page);
  assert(canvas && canvas.width >= 440 && canvas.height >= 260 && canvas.colors > 4, `projectile canvas blank or too small: ${JSON.stringify(canvas)}`);

  const layout = await page.evaluate(() => {
    const editor = document.querySelector(".run-editor-panel")?.getBoundingClientRect();
    const main = document.querySelector(".bogae-area")?.getBoundingClientRect();
    const observe = document.querySelector(".dotbogi-panel.subpanel")?.getBoundingClientRect();
    const total = (editor?.width ?? 0) + (main?.width ?? 0) + (observe?.width ?? 0);
    return {
      editorRatio: total ? (editor?.width ?? 0) / total : 0,
      mainRatio: total ? (main?.width ?? 0) / total : 0,
      observeRatio: total ? (observe?.width ?? 0) / total : 0,
      ddnText: document.querySelector("#run-ddn-preview")?.value ?? "",
      navActive: Array.from(document.querySelectorAll("#screen-run .main-shell-tab.active"))
        .map((node) => node.textContent?.trim() ?? ""),
      roleSwitchCount: document.querySelectorAll("[data-classroom-mode-switch]").length,
      roleButtonCount: document.querySelectorAll("[data-classroom-mode]").length,
      classroomModeAccess: document.querySelector("#screen-run")?.dataset?.classroomModeAccess ?? "",
    };
  });
  assert(layout.ddnText.includes("포물선") || layout.ddnText.includes("초기속도"), "DDN source not loaded into L1");
  assert(layout.roleSwitchCount === 0 && layout.roleButtonCount === 0 && layout.classroomModeAccess === "", `runtime role toggle survived L1: ${JSON.stringify(layout)}`);
  assert(layout.navActive.length === 1 && layout.navActive[0] === "배우기", `lesson-open shell active tab mismatch: ${layout.navActive}`);
  assert(
    await page.locator("#screen-run").getAttribute("data-shell-space") === "learn",
    "lesson-open did not preserve learn space",
  );
  assert(layout.editorRatio >= 0.16 && layout.editorRatio <= 0.30, `DDN column ratio out of range: ${layout.editorRatio}`);
  assert(layout.mainRatio >= 0.44 && layout.mainRatio <= 0.66, `main view ratio out of range: ${layout.mainRatio}`);
  assert(layout.observeRatio >= 0.16 && layout.observeRatio <= 0.32, `observe column ratio out of range: ${layout.observeRatio}`);

  await page.locator("#screen-run .main-shell-tab[data-shell-space='learn']").click();
  await page.waitForSelector("#screen-browse:not(.hidden)", { state: "visible" });
  await page.locator(`#screen-browse [data-lesson-id='${PROJECTILE_ID}'] [data-launch-action='lab']`).click();
  await waitForProjectileRun(page);
  const labSpace = await page.evaluate(() => ({
    shellSpace: document.querySelector("#screen-run")?.dataset?.shellSpace ?? "",
    active: Array.from(document.querySelectorAll("#screen-run .main-shell-tab.active"))
      .map((node) => node.textContent?.trim() ?? ""),
  }));
  assert(labSpace.shellSpace === "lab", `lab action space mismatch: ${JSON.stringify(labSpace)}`);
  assert(labSpace.active.length === 1 && labSpace.active[0] === "실험실", `lab action active tab mismatch: ${JSON.stringify(labSpace)}`);
}

async function verifyMobileShell(page, baseUrl) {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${PROJECTILE_ID}`, { waitUntil: "domcontentloaded" });
  await waitForProjectileRun(page);
  const before = await page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const editor = document.querySelector(".run-editor-panel");
    const visual = document.querySelector(".run-visual-column");
    const nav = document.querySelector("#screen-run .main-shell-tabs");
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']");
    const canvasRect = canvas?.getBoundingClientRect();
    const body = document.body;
    return {
      mobileDdnOpen: root?.dataset?.mobileDdnOpen ?? "",
      editorDisplay: getComputedStyle(editor).display,
      visualDisplay: getComputedStyle(visual).display,
      navTop: nav?.getBoundingClientRect().top ?? 0,
      navBottom: nav?.getBoundingClientRect().bottom ?? 0,
      viewportHeight: window.innerHeight,
      overflowX: body.scrollWidth - window.innerWidth,
      canvasLeft: canvasRect?.left ?? -1,
      canvasRight: canvasRect?.right ?? -1,
      viewportWidth: window.innerWidth,
    };
  });
  assert(before.mobileDdnOpen === "0", `mobile DDN should start closed: ${before.mobileDdnOpen}`);
  assert(before.editorDisplay === "none", `mobile editor should be hidden by default: ${before.editorDisplay}`);
  assert(before.visualDisplay !== "none", "mobile visual should be visible by default");
  assert(before.navTop > before.viewportHeight * 0.80 && before.navBottom <= before.viewportHeight + 2, `mobile nav not bottom anchored: ${JSON.stringify(before)}`);
  assert(before.overflowX <= 2, `mobile horizontal overflow: ${before.overflowX}`);
  assert(before.canvasLeft >= -2 && before.canvasRight <= before.viewportWidth + 2, `mobile canvas clipped: ${JSON.stringify(before)}`);

  await page.locator("[data-testid='lesson-ddn-toggle']").click();
  const after = await page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const editor = document.querySelector(".run-editor-panel");
    const visual = document.querySelector(".run-visual-column");
    return {
      mobileDdnOpen: root?.dataset?.mobileDdnOpen ?? "",
      editorDisplay: getComputedStyle(editor).display,
      visualDisplay: getComputedStyle(visual).display,
      toggleText: document.querySelector("[data-testid='lesson-ddn-toggle']")?.textContent?.trim() ?? "",
    };
  });
  assert(after.mobileDdnOpen === "1", "mobile DDN toggle did not open");
  assert(after.editorDisplay !== "none", "mobile editor should be visible after toggle");
  assert(after.visualDisplay === "none", "mobile visual should hide when DDN is open");
  assert(after.toggleText === "보기", `mobile toggle text mismatch: ${after.toggleText}`);
}

async function main() {
  const root = path.resolve(".");
  const { server, baseUrl } = await createServer(root);
  const browser = await chromium.launch();
  const failures = [];
  try {
    const desktop = await browser.newPage();
    installFailureCapture(desktop, failures);
    await verifyDesktopShell(desktop, baseUrl);
    await desktop.close();

    const mobile = await browser.newPage();
    installFailureCapture(mobile, failures);
    await verifyMobileShell(mobile, baseUrl);
    await mobile.close();

    assert(failures.length === 0, failures.join("\n"));
    process.stdout.write(`${OK}\n`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
