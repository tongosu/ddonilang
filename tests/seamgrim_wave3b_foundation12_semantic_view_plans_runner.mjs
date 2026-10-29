#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3b_foundation12_semantic_view_plans: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const WORLDS = Object.freeze([
  ["elem_math_number_line", "graph"],
  ["elem_math_fraction_line", "graph"],
  ["elem_math_rectangle_area", "graph"],
  ["elem_science_daily_temperature", "graph"],
  ["elem_econ_allowance_budget", "graph"],
  ["elem_econ_store_stock", "graph"],
  ["rep_math_function_line_v1", "graph"],
  ["rep_phys_pendulum_wave1_v1", "space2d"],
  ["rep_phys_projectile_xy_v1", "space2d"],
  ["rep_econ_supply_demand_tax_v1", "graph"],
  ["rep_phys_damped_oscillation_v1", "graph"],
  ["high_math_derivative", "graph"],
]);
const VIEWPORTS = Object.freeze([
  { name: "desktop", width: 1440, height: 900 },
  { name: "wide", width: 1280, height: 800 },
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

function installFailureCapture(page, failures) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    if (String(msg.text() ?? "").includes("Failed to load resource")) return;
    failures.push(`console error: ${msg.text()}`);
  });
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
}

async function waitForResolvedViewPlan(page, expectedMode) {
  const predicate = (mode) => {
    const root = document.querySelector("#screen-run");
    const report = window.__SEAMGRIM_VIEW_PLAN_RUNTIME__ ?? null;
    const anchors = Array.isArray(report?.anchors) ? report.anchors : [];
    return root?.dataset?.viewPlanRuntime === "1"
      && root?.dataset?.mainVisualMode === mode
      && anchors.length >= 2
      && anchors.every((anchor) => anchor?.ok === true);
  };
  await page.waitForFunction(predicate, expectedMode, { timeout: 30_000 });
  await page.waitForTimeout(180);
  await page.waitForFunction(predicate, expectedMode, { timeout: 30_000 });
}

async function readState(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const report = window.__SEAMGRIM_VIEW_PLAN_RUNTIME__ ?? null;
    const graphHost = document.querySelector("#run-main-graph-host");
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']");
    const visual = root?.dataset?.mainVisualMode === "graph" ? graphHost : canvas;
    const rect = visual?.getBoundingClientRect?.() ?? { width: 0, height: 0 };
    let canvasColors = 0;
    if (canvas instanceof HTMLCanvasElement && canvas.width > 0 && canvas.height > 0) {
      const ctx = canvas.getContext("2d");
      if (ctx) {
        const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
        const colors = new Set();
        const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 3500) * 4);
        for (let i = 0; i < data.length; i += stride) {
          colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
          if (colors.size > 10) break;
        }
        canvasColors = colors.size;
      }
    }
    return {
      root: { ...(root?.dataset ?? {}) },
      report,
      stateHash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      visualRect: { width: rect.width, height: rect.height },
      graphText: graphHost?.textContent?.trim() ?? "",
      canvasColors,
    };
  });
}

function assertViewState(lessonId, expectedMode, state, viewportName) {
  const root = state.root ?? {};
  const report = state.report ?? {};
  const anchors = Array.isArray(report.anchors) ? report.anchors : [];
  assert(root.viewPlanRuntime === "1", `${lessonId}/${viewportName}: view_plan runtime missing`);
  assert(root.viewPlanStateHashParticipation === "0", `${lessonId}/${viewportName}: view_plan entered state_hash`);
  assert(String(root.viewPlanHash ?? "").startsWith("candidate:fnv1a32:view_plan:"), `${lessonId}/${viewportName}: view hash missing`);
  assert(root.mainVisualMode === expectedMode, `${lessonId}/${viewportName}: mode ${root.mainVisualMode} != ${expectedMode}`);
  assert(root.mainVisualMode !== "console-grid", `${lessonId}/${viewportName}: graph-primary fell back to console-grid`);
  assert(anchors.length >= 2, `${lessonId}/${viewportName}: anchor count ${anchors.length}`);
  assert(anchors.every((anchor) => anchor.ok === true), `${lessonId}/${viewportName}: unresolved anchors ${JSON.stringify(anchors)}`);
  assert(anchors.every((anchor) => String(anchor.unit ?? "").trim()), `${lessonId}/${viewportName}: unit missing`);
  assert(anchors.every((anchor) => String(anchor.label ?? "").trim()), `${lessonId}/${viewportName}: label missing`);
  assert(anchors.every((anchor) => String(anchor.provenance?.path ?? "").includes(`/lessons/${lessonId}/lesson.ddn`)), `${lessonId}/${viewportName}: provenance path mismatch`);
  assert(Array.isArray(report.diagnostics) && report.diagnostics.length === 0, `${lessonId}/${viewportName}: diagnostics ${JSON.stringify(report.diagnostics)}`);
  assert(state.stateHash.includes("blake3:"), `${lessonId}/${viewportName}: state hash missing`);
  assert(state.visualRect.width > 180 && state.visualRect.height > 120, `${lessonId}/${viewportName}: visual too small ${JSON.stringify(state.visualRect)}`);
  if (expectedMode === "space2d") {
    assert(state.canvasColors > 4, `${lessonId}/${viewportName}: blank canvas`);
  } else {
    assert(String(state.graphText ?? "").length > 0 || Number(root.mainGraphPointCount ?? 0) > 0, `${lessonId}/${viewportName}: graph visual blank`);
  }
}

async function readAssertedViewState(page, lessonId, expectedMode, viewportName) {
  const deadline = Date.now() + 30_000;
  let lastState = null;
  let lastError = null;
  while (Date.now() < deadline) {
    await waitForResolvedViewPlan(page, expectedMode);
    lastState = await readState(page);
    try {
      assertViewState(lessonId, expectedMode, lastState, viewportName);
      return lastState;
    } catch (err) {
      lastError = err;
      await page.waitForTimeout(120);
    }
  }
  if (lastState) {
    assertViewState(lessonId, expectedMode, lastState, viewportName);
  }
  throw lastError ?? new Error(`${lessonId}/${viewportName}: view_plan state did not settle`);
}

async function verifyLesson(browser, baseUrl, lessonId, expectedMode, viewport) {
  const page = await browser.newPage({ viewport: { width: viewport.width, height: viewport.height } });
  const failures = [];
  installFailureCapture(page, failures);
  try {
    const url = `${baseUrl}${PROJECT_URL}?tab=studio&lesson=${lessonId}`;
    await page.goto(url, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
    await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:"), null, { timeout: 30_000 });
    const first = await readAssertedViewState(page, lessonId, expectedMode, viewport.name);
    const hashBefore = first.root.viewPlanHash;
    const anchorCountBefore = first.report.anchors.length;
    await page.reload({ waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
    const reloaded = await readAssertedViewState(page, lessonId, expectedMode, viewport.name);
    assert(reloaded.root.viewPlanHash === hashBefore, `${lessonId}/${viewport.name}: reload changed view hash`);
    assert(reloaded.report.anchors.length === anchorCountBefore, `${lessonId}/${viewport.name}: reload changed anchor count`);
    const range = page.locator("#range-dock-time-cursor");
    if (await range.count() && await range.first().isVisible()) {
      await range.fill("25");
      await readAssertedViewState(page, lessonId, expectedMode, viewport.name);
      await range.fill("75");
      const scrubbed = await readAssertedViewState(page, lessonId, expectedMode, viewport.name);
      assert(scrubbed.root.viewPlanHash === hashBefore, `${lessonId}/${viewport.name}: scrub changed view hash`);
      assert(scrubbed.report.anchors.length === anchorCountBefore, `${lessonId}/${viewport.name}: scrub changed anchor count`);
    }
    assert(failures.length === 0, `${lessonId}/${viewport.name}: ${failures.join(" | ")}`);
    await page.close();
    return { lessonId, viewport: viewport.name, mode: expectedMode, anchors: anchorCountBefore, viewPlanHash: hashBefore };
  } catch (err) {
    await page.close();
    throw err;
  }
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch({ headless: true });
  try {
    const rows = [];
    for (const [lessonId, expectedMode] of WORLDS) {
      for (const viewport of VIEWPORTS) {
        rows.push(await verifyLesson(browser, baseUrl, lessonId, expectedMode, viewport));
      }
    }
    console.log(`${OK} ${JSON.stringify({ checks: rows.length, worlds: WORLDS.length })}`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
