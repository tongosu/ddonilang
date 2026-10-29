#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_publish_screen_isolation: ok";

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

function installFailureCapture(page, failures) {
  page.on("console", (msg) => {
    const text = String(msg.text() ?? "");
    if (msg.type() === "error" && !text.includes("Failed to load resource")) {
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
}

async function verifyViewport(page, baseUrl, viewport) {
  await page.setViewportSize({ width: viewport.width, height: viewport.height });
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse:not(.hidden) .lesson-card[data-lesson-id]", { state: "visible", timeout: 20_000 });

  const before = await page.evaluate(() => ({
    browseHidden: document.querySelector("#screen-browse")?.classList?.contains("hidden") ?? true,
    publishHidden: document.querySelector("#screen-publish")?.classList?.contains("hidden") ?? true,
    cardCount: document.querySelectorAll("#screen-browse .lesson-card[data-lesson-id]").length,
    browsePublishClass: document.querySelector("#screen-browse")?.classList?.contains("browse-space-publish") ?? false,
    queryValue: document.querySelector("#filter-query")?.value ?? "",
  }));
  assert(before.browseHidden === false && before.publishHidden === true, `${viewport.name}: initial screen mismatch ${JSON.stringify(before)}`);
  assert(before.cardCount >= 3, `${viewport.name}: catalog too small ${JSON.stringify(before)}`);
  assert(before.browsePublishClass === false, `${viewport.name}: initial browse publish class leaked`);

  await page.locator("#screen-browse .main-shell-tab[data-shell-space='publish']").click();
  await page.waitForSelector("#screen-publish:not(.hidden) [data-publish-panel]", { state: "visible", timeout: 10_000 });
  const publish = await page.evaluate(() => {
    const panel = document.querySelector("#screen-publish [data-publish-panel]");
    const rect = panel?.getBoundingClientRect?.();
    return {
      publishHidden: document.querySelector("#screen-publish")?.classList?.contains("hidden") ?? true,
      browseHidden: document.querySelector("#screen-browse")?.classList?.contains("hidden") ?? false,
      browsePublishClass: document.querySelector("#screen-browse")?.classList?.contains("browse-space-publish") ?? false,
      publishShell: document.querySelector("#screen-publish")?.dataset?.shellSpace ?? "",
      browseShell: document.querySelector("#screen-browse")?.dataset?.shellSpace ?? "",
      active: Array.from(document.querySelectorAll("#screen-publish .main-shell-tab.active")).map((node) => node.textContent?.trim() ?? ""),
      panelWidth: rect?.width ?? 0,
      panelHeight: rect?.height ?? 0,
      actions: Array.from(document.querySelectorAll("#screen-publish [data-publish-panel] button")).map((node) => node.textContent?.trim() ?? ""),
      catalogBodyDisplay: getComputedStyle(document.querySelector("#screen-browse .catalog-body")).display,
      pasteOpen: document.querySelector("#screen-publish [data-local-package-paste-panel]")?.dataset?.open ?? "",
    };
  });
  assert(publish.publishHidden === false && publish.browseHidden === true, `${viewport.name}: publish screen isolation mismatch ${JSON.stringify(publish)}`);
  assert(publish.browsePublishClass === false, `${viewport.name}: browse publish class leaked ${JSON.stringify(publish)}`);
  assert(publish.publishShell === "publish", `${viewport.name}: publish shell mismatch ${JSON.stringify(publish)}`);
  assert(publish.browseShell === "learn", `${viewport.name}: browse shell mutated ${JSON.stringify(publish)}`);
  assert(publish.active.length === 1 && publish.active[0] === "배포", `${viewport.name}: active publish tab mismatch ${JSON.stringify(publish)}`);
  assert(publish.panelWidth > 240 && publish.panelHeight > 160, `${viewport.name}: publish panel size invalid ${JSON.stringify(publish)}`);
  for (const action of ["배포 열기", "붙여넣기 열기", "대표 묶음 복사", "대표 묶음 저장"]) {
    assert(publish.actions.includes(action), `${viewport.name}: missing publish action ${action}`);
  }

  await page.locator("#screen-publish #btn-paste-local-package").click();
  await page.waitForSelector("#screen-publish [data-local-package-paste-panel]:not(.hidden)", { state: "visible", timeout: 5_000 });
  const paste = await page.evaluate(() => ({
    open: document.querySelector("#screen-publish [data-local-package-paste-panel]")?.dataset?.open ?? "",
    browseHidden: document.querySelector("#screen-browse")?.classList?.contains("hidden") ?? false,
    activeElement: document.activeElement?.id ?? "",
  }));
  assert(paste.open === "1" && paste.browseHidden === true, `${viewport.name}: paste panel isolation mismatch ${JSON.stringify(paste)}`);

  await page.locator("#screen-publish .main-shell-tab[data-shell-space='learn']").click();
  await page.waitForSelector("#screen-browse:not(.hidden) .lesson-card[data-lesson-id]", { state: "visible", timeout: 10_000 });
  const back = await page.evaluate(() => ({
    browseHidden: document.querySelector("#screen-browse")?.classList?.contains("hidden") ?? true,
    publishHidden: document.querySelector("#screen-publish")?.classList?.contains("hidden") ?? false,
    browsePublishClass: document.querySelector("#screen-browse")?.classList?.contains("browse-space-publish") ?? false,
    catalogDisplay: getComputedStyle(document.querySelector("#screen-browse .catalog-body")).display,
    cardCount: document.querySelectorAll("#screen-browse .lesson-card[data-lesson-id]").length,
  }));
  assert(back.browseHidden === false && back.publishHidden === true, `${viewport.name}: learn return screen mismatch ${JSON.stringify(back)}`);
  assert(back.browsePublishClass === false, `${viewport.name}: browse publish class survived return ${JSON.stringify(back)}`);
  assert(back.catalogDisplay !== "none" && back.cardCount === before.cardCount, `${viewport.name}: catalog state changed ${JSON.stringify(back)}`);
}

async function main() {
  const root = path.resolve(".");
  const { server, baseUrl } = await createServer(root);
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  try {
    const page = await browser.newPage({ locale: "ko-KR" });
    installFailureCapture(page, failures);
    for (const viewport of [
      { name: "desktop", width: 1440, height: 900 },
      { name: "tablet", width: 1024, height: 768 },
      { name: "mobile", width: 390, height: 844 },
    ]) {
      await verifyViewport(page, baseUrl, viewport);
    }
    assert(failures.length === 0, failures.join("\n"));
    await page.close();
  } finally {
    await browser.close();
    await closeServer(server);
  }
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
