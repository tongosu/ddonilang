#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave2b_quality_gate: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const VIEWPORTS = [
  { width: 1440, height: 900 },
  { width: 1280, height: 720 },
  { width: 1024, height: 768 },
  { width: 390, height: 844 },
];

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
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|detjson|md)$/i.test(pathname)
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
    for (let i = 0; i < data.length; i += stride) {
      colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
      if (colors.size > 12) break;
    }
    return { width: rect.width, height: rect.height, left: rect.left, right: rect.right, colors: colors.size };
  });
}

async function verifyViewport(page, baseUrl, viewport) {
  await page.setViewportSize(viewport);
  await page.goto(`${baseUrl}${PROJECT_URL}?first_delight=pendulum`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => {
    const root = document.querySelector("#screen-run");
    return root?.dataset?.viewPlanRuntime === "1"
      && String(root?.dataset?.mainViewRoles ?? "").includes("angle_measure")
      && String(document.querySelector("#run-ddn-preview")?.value ?? "").includes("theta <- theta + omega * dt")
      && String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:");
  }, null, { timeout: 35_000 });
  const canvas = await readCanvasEvidence(page);
  assert(canvas && canvas.width >= Math.min(300, viewport.width - 40), `canvas width invalid at ${viewport.width}x${viewport.height}: ${JSON.stringify(canvas)}`);
  assert(canvas.height >= (viewport.width <= 420 ? 150 : 220), `canvas height invalid at ${viewport.width}x${viewport.height}: ${JSON.stringify(canvas)}`);
  assert(canvas.colors > 4, `canvas blank at ${viewport.width}x${viewport.height}: ${JSON.stringify(canvas)}`);
  assert(canvas.left >= -2 && canvas.right <= viewport.width + 2, `canvas clipped at ${viewport.width}x${viewport.height}: ${JSON.stringify(canvas)}`);
  if (viewport.width <= 420) {
    await page.locator("#btn-run-mobile-ddn-toggle").click();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.mobileDdnOpen === "1", null, { timeout: 10_000 });
  }
  const evidence = await page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const graph = document.querySelector("#run-main-graph-host")?.getBoundingClientRect();
    const preview = document.querySelector("#run-ddn-preview")?.getBoundingClientRect();
    const run = document.querySelector("#btn-run");
    const reset = document.querySelector("#btn-reset");
    const step = document.querySelector("#btn-step");
    return {
      root: { ...(root?.dataset ?? {}) },
      scrollWidth: document.documentElement.scrollWidth,
      clientWidth: document.documentElement.clientWidth,
      graphHeight: graph?.height ?? 0,
      ddnHeight: preview?.height ?? 0,
      sourceLength: document.querySelector("#run-ddn-preview")?.value?.length ?? 0,
      runButtonText: run?.textContent?.trim() ?? "",
      resetTitle: reset?.getAttribute("title") ?? "",
      resetAria: reset?.getAttribute("aria-label") ?? "",
      stepTitle: step?.getAttribute("title") ?? "",
      stepAria: step?.getAttribute("aria-label") ?? "",
    };
  });
  assert(evidence.scrollWidth - evidence.clientWidth <= 2, `horizontal overflow at ${viewport.width}x${viewport.height}: ${evidence.scrollWidth - evidence.clientWidth}`);
  assert(evidence.sourceLength > 80 && evidence.ddnHeight > 40, `DDN source inaccessible at ${viewport.width}x${viewport.height}`);
  assert(evidence.graphHeight >= 0, "graph host measurement missing");
  assert(evidence.runButtonText.includes("수업 실행") || evidence.runButtonText.includes("실행"), "run button text missing");
  assert(evidence.resetTitle && evidence.resetAria, "reset tooltip/aria missing");
  assert(evidence.stepTitle && evidence.stepAria, "step tooltip/aria missing");
}

async function verifyKeyboardAndReplay(page, baseUrl) {
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto(`${baseUrl}${PROJECT_URL}?first_delight=projectile`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:"), null, { timeout: 35_000 });
  await page.waitForTimeout(250);
  await page.locator("#btn-reset").press("Enter");
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "idle", null, { timeout: 10_000 });
  await page.locator("#btn-run").press("Enter");
  await page.waitForFunction(() => ["running", "done"].includes(document.querySelector("#screen-run")?.dataset?.engineStatus ?? ""), null, { timeout: 10_000 });
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:"), null, { timeout: 35_000 });
}

async function verifyRepresentativeGraphLayout(page, baseUrl) {
  await page.setViewportSize({ width: 1024, height: 768 });
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=rep_math_function_line_v1`, {
    waitUntil: "domcontentloaded",
  });
  await page.waitForFunction(() => {
    const root = document.querySelector("#screen-run");
    return !root?.classList?.contains("hidden")
      && root?.dataset?.mainVisualMode === "graph"
      && String(document.querySelector("#run-ddn-preview")?.value ?? "").includes("일차 함수 그래프");
  }, null, { timeout: 35_000 });
  const evidence = await page.evaluate(() => {
    const metric = (selector) => {
      const element = document.querySelector(selector);
      if (!(element instanceof HTMLElement)) return null;
      const rect = element.getBoundingClientRect();
      return {
        width: rect.width,
        height: rect.height,
        scrollWidth: element.scrollWidth,
        clientWidth: element.clientWidth,
      };
    };
    const brokenLabels = [...document.querySelectorAll(".slider-label")]
      .filter((element) => {
        const rect = element.getBoundingClientRect();
        const lineHeight = Number.parseFloat(getComputedStyle(element).lineHeight) || 16;
        return rect.height > lineHeight * 2.2;
      })
      .map((element) => element.textContent?.trim() ?? "");
    return {
      topExplanationButtonCount: document.querySelectorAll("#btn-overlay-toggle").length,
      explanationTabCount: document.querySelectorAll("#run-tab-btn-overlay").length,
      controlBar: metric(".run-control-bar"),
      presetRail: metric("#run-preset-rail"),
      graph: metric("#run-main-graph-host"),
      brokenLabels,
    };
  });
  assert(evidence.topExplanationButtonCount === 0, `duplicate explanation entry remains: ${JSON.stringify(evidence)}`);
  assert(evidence.explanationTabCount === 1, `explanation tab missing: ${JSON.stringify(evidence)}`);
  assert(
    evidence.controlBar && evidence.controlBar.scrollWidth <= evidence.controlBar.clientWidth + 2,
    `control bar clipped at 1024x768: ${JSON.stringify(evidence)}`,
  );
  assert(
    evidence.presetRail && evidence.presetRail.scrollWidth <= evidence.presetRail.clientWidth + 2,
    `preset rail clipped at 1024x768: ${JSON.stringify(evidence)}`,
  );
  assert(evidence.graph && evidence.graph.width > 400 && evidence.graph.height > 220, `primary graph too small: ${JSON.stringify(evidence)}`);
  assert(evidence.brokenLabels.length === 0, `slider labels break vertically: ${JSON.stringify(evidence)}`);
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: VIEWPORTS[0], locale: "ko-KR" });
    installFailureCapture(page, failures);
    for (const viewport of VIEWPORTS) {
      await verifyViewport(page, baseUrl, viewport);
    }
    await verifyRepresentativeGraphLayout(page, baseUrl);
    await verifyKeyboardAndReplay(page, baseUrl);
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
