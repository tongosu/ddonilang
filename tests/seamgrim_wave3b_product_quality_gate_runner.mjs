#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3b_product_quality_gate: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const LESSON_ID = "rep_math_function_line_v1";
const VIEWPORTS = Object.freeze([
  { name: "desktop", width: 1440, height: 900 },
  { name: "laptop", width: 1280, height: 800 },
  { name: "tablet", width: 1024, height: 768 },
  { name: "mobile", width: 390, height: 844 },
]);

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
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/") || pathname.startsWith("/lessons_rewrite_v1/"))
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|detjson|md)$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function installFailureCapture(page, failures, label) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    const text = String(msg.text() ?? "");
    if (text.includes("Failed to load resource")) return;
    failures.push(`${label}: console error: ${text}`);
  });
  page.on("pageerror", (err) => failures.push(`${label}: pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`${label}: request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`${label}: response ${res.status()}: ${res.url()}`);
  });
}

async function waitVisible(page, selector) {
  await page.waitForFunction((sel) => {
    const node = document.querySelector(sel);
    return node && !String(node.className ?? "").split(/\s+/).includes("hidden");
  }, selector, { timeout: 30_000 });
}

async function verifyRunQuality(page, baseUrl, viewport) {
  await page.setViewportSize({ width: viewport.width, height: viewport.height });
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&catalog_mode=full&lesson=${encodeURIComponent(LESSON_ID)}`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:"), null, { timeout: 30_000 });
  const before = await page.evaluate(() => ({
    overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    ddnLength: String(document.querySelector("#run-ddn-preview")?.value ?? "").length,
    runButtonText: document.querySelector("#btn-run")?.textContent?.trim() ?? "",
    resetButtonText: document.querySelector("#btn-reset")?.textContent?.trim() ?? "",
    ddnToggleName: document.querySelector("#btn-run-mobile-ddn-toggle")?.textContent?.trim() ?? "",
    ddnToggleExpanded: document.querySelector("#btn-run-mobile-ddn-toggle")?.getAttribute("aria-expanded") ?? "",
    graphRect: (() => {
      const candidates = [
        "#run-main-graph-host",
        "#run-main-graph-host canvas",
        "#run-main-graph-host .runtime-graph-canvas",
        "#canvas-graph",
        "#canvas-bogae",
        ".bogae-frame",
      ];
      const rects = candidates
        .map((selector) => document.querySelector(selector)?.getBoundingClientRect())
        .filter((rect) => rect && rect.width > 0 && rect.height > 0)
        .map((rect) => ({ width: Math.round(rect.width), height: Math.round(rect.height), area: Math.round(rect.width * rect.height) }))
        .sort((left, right) => right.area - left.area);
      return rects[0] ?? null;
    })(),
    controls: Array.from(document.querySelectorAll('#slider-list input[data-control-name]')).map((node) => ({
      name: node.getAttribute("data-control-name"),
      aria: node.getAttribute("aria-label") || "",
      width: Math.round(node.getBoundingClientRect().width),
      height: Math.round(node.getBoundingClientRect().height),
    })),
    tabs: Array.from(document.querySelectorAll('#screen-run .main-shell-tab')).map((node) => node.textContent?.trim() ?? ""),
  }));
  assert(before.overflow <= 2, `${viewport.name}: horizontal overflow ${before.overflow}`);
  assert(before.ddnLength > 40, `${viewport.name}: DDN source unavailable`);
  assert(before.runButtonText, `${viewport.name}: run button has no accessible text`);
  assert(before.resetButtonText, `${viewport.name}: reset button has no accessible text`);
  assert(before.tabs.includes("배우기") && before.tabs.includes("배포"), `${viewport.name}: shell tabs missing labels`);
  assert(before.controls.length >= 1, `${viewport.name}: control inputs missing`);
  before.controls.forEach((control) => {
    assert(control.aria.length >= 3, `${viewport.name}: control aria missing ${control.name}`);
    assert(control.width > 0 && control.height > 0, `${viewport.name}: control has zero size ${control.name}`);
  });
  if (viewport.width <= 480) {
    assert(before.ddnToggleName === "DDN" || before.ddnToggleName === "보기", `${viewport.name}: mobile DDN toggle missing`);
    assert(before.ddnToggleExpanded === "false" || before.ddnToggleExpanded === "true", `${viewport.name}: mobile DDN toggle aria missing`);
  }
  assert(before.graphRect?.width >= 160 && before.graphRect?.height >= 100, `${viewport.name}: primary view too small ${JSON.stringify(before.graphRect)}`);

  if (await page.locator("#btn-run").isDisabled()) {
    await page.locator("#btn-reset").click();
    await page.waitForFunction(() => !document.querySelector("#btn-run")?.disabled, null, { timeout: 20_000 });
  }
  await page.locator("#btn-run").focus();
  const activeBefore = await page.evaluate(() => document.activeElement?.id ?? "");
  assert(activeBefore === "btn-run", `${viewport.name}: run button did not receive keyboard focus`);
  await page.keyboard.press("Enter");
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:"), null, { timeout: 45_000 });
  await page.waitForTimeout(500);
  const after = await page.evaluate(() => ({
    runtimeHash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
    runButtonText: document.querySelector("#btn-run")?.textContent?.trim() ?? "",
  }));
  assert(after.runtimeHash.includes("blake3:"), `${viewport.name}: runtime hash missing after keyboard run`);
  assert(after.runButtonText, `${viewport.name}: run button label lost after keyboard run`);
}

async function verifyPublishKeyboard(page, baseUrl, viewport) {
  await page.setViewportSize({ width: viewport.width, height: viewport.height });
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=publish&catalog_mode=full`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-publish");
  await page.locator("#btn-paste-local-package").focus();
  await page.keyboard.press("Enter");
  await page.waitForSelector("#local-package-paste-text", { state: "visible", timeout: 20_000 });
  const state = await page.evaluate(() => ({
    pasteExpanded: document.querySelector("#btn-paste-local-package")?.getAttribute("aria-expanded") ?? "",
    archiveVisible: Boolean(document.querySelector("[data-student-submission-archive]")),
    archiveLabel: document.querySelector("[data-student-submission-archive]")?.getAttribute("aria-label") ?? "",
    archiveAddText: document.querySelector("#btn-student-submission-add")?.textContent?.trim() ?? "",
    archiveExportDisabled: document.querySelector("#btn-student-submission-archive-export")?.disabled === true,
    overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
  }));
  assert(state.pasteExpanded === "true", `${viewport.name}: keyboard paste toggle did not open`);
  assert(state.archiveVisible && state.archiveLabel.includes("학생 제출"), `${viewport.name}: submission archive a11y label missing`);
  assert(state.archiveAddText.includes("제출"), `${viewport.name}: archive add button label missing`);
  assert(state.archiveExportDisabled, `${viewport.name}: empty archive export should be disabled`);
  assert(state.overflow <= 2, `${viewport.name}: publish horizontal overflow ${state.overflow}`);
}

async function main() {
  const { server, baseUrl } = await createServer(process.cwd());
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  try {
    for (const viewport of VIEWPORTS) {
      const context = await browser.newContext({ viewport: { width: viewport.width, height: viewport.height }, locale: "ko-KR" });
      await context.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
      const page = await context.newPage();
      installFailureCapture(page, failures, viewport.name);
      await verifyRunQuality(page, baseUrl, viewport);
      await verifyPublishKeyboard(page, baseUrl, viewport);
      await context.close();
    }
    assert(failures.length === 0, failures.join("\n"));
    console.log(`${OK} ${JSON.stringify({ viewports: VIEWPORTS.length, lesson_id: LESSON_ID })}`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((error) => {
  console.error(error?.stack || String(error));
  process.exit(1);
});
