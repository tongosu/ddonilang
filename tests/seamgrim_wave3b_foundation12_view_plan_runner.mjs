#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3b_foundation12_view_plan: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const TARGETS = Object.freeze([
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

async function pauseRunningOrAcceptComplete(page) {
  const status = await page.evaluate(() => document.querySelector("#screen-run")?.dataset?.engineStatus ?? "");
  if (status === "running") {
    const pause = page.locator("#btn-pause");
    assert(await pause.isEnabled(), "pause must be enabled while execution is running");
    await pause.click();
  } else {
    assert(["paused", "done"].includes(status), `unexpected execution lifecycle before evidence: ${status}`);
  }
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    const optionalLessonDataPrefix =
      pathname.startsWith("/lessons/")
      || pathname.startsWith("/lessons_rewrite_v1/")
      || pathname.startsWith("/seed_lessons_v1/");
    if (optionalLessonDataPrefix && /\/(?:graph|table|space2d|maegim_control)\.json$/i.test(pathname)) return true;
    if (optionalLessonDataPrefix && /\/(?:text\.md)$/i.test(pathname)) return true;
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
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`${label}: response ${res.status()}: ${res.url()}`);
  });
}

async function readCanvasEvidence(page) {
  return page.evaluate(() => {
    return Array.from(document.querySelectorAll("canvas"))
      .filter((canvas) => canvas instanceof HTMLCanvasElement)
      .map((canvas) => {
        const rect = canvas.getBoundingClientRect();
        const ctx = canvas.getContext("2d");
        if (!ctx || canvas.width <= 0 || canvas.height <= 0 || rect.width <= 20 || rect.height <= 20) {
          return { selector: canvas.id || canvas.dataset?.testid || "canvas", width: rect.width, height: rect.height, colors: 0 };
        }
        const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
        const colors = new Set();
        const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 4000) * 4);
        for (let index = 0; index < data.length; index += stride) {
          colors.add(`${data[index]}:${data[index + 1]}:${data[index + 2]}:${data[index + 3]}`);
          if (colors.size > 12) break;
        }
        return {
          selector: canvas.id || canvas.dataset?.testid || "canvas",
          width: Math.round(rect.width),
          height: Math.round(rect.height),
          colors: colors.size,
        };
      });
  });
}

async function readViewEvidence(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const graphHost = document.querySelector("#run-main-graph-host")?.getBoundingClientRect();
    const report = window.__SEAMGRIM_VIEW_PLAN_RUNTIME__ ?? null;
    const cursor = window.__SEAMGRIM_SEMANTIC_CURSOR__ ?? null;
    const views = window.__SEAMGRIM_LAST_VIEWS__ ?? null;
    return {
      url: location.href,
      dataset: { ...(root?.dataset ?? {}) },
      graphHost: graphHost ? {
        width: Math.round(graphHost.width),
        height: Math.round(graphHost.height),
      } : null,
      report,
      cursor,
      views,
      ddnTextLength: String(document.querySelector("#run-ddn-preview")?.value ?? "").length,
      horizontalOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    };
  });
}

function assertSemanticEvidence(label, evidence, expectedMode) {
  const lessonId = label.split(":")[0] ?? "";
  const dataset = evidence.dataset ?? {};
  const report = evidence.report ?? {};
  const cursor = evidence.cursor ?? {};
  const anchors = Array.isArray(report.anchors) ? report.anchors : [];
  const cursorAnchors = Array.isArray(cursor.anchors) ? cursor.anchors : [];
  assert(dataset.viewPlanRuntime === "1", `${label}: view plan runtime missing`);
  assert(dataset.viewPlanBindingMode === "explicit", `${label}: binding mode mismatch`);
  assert(dataset.viewPlanStateHashParticipation === "0", `${label}: view plan entered state_hash`);
  assert(String(dataset.viewPlanHash ?? "").startsWith("candidate:fnv1a32:view_plan:"), `${label}: view hash missing`);
  assert(report.schema === "seamgrim.view_plan.runtime.v1", `${label}: runtime report schema`);
  assert(report.lesson_id === lessonId, `${label}: stale runtime report lesson ${report.lesson_id}`);
  assert(report.state_hash_participation === false, `${label}: report entered state_hash`);
  assert(Array.isArray(report.diagnostics) && report.diagnostics.length === 0, `${label}: diagnostics ${JSON.stringify(report.diagnostics)}`);
  assert(anchors.length >= 2, `${label}: too few anchors`);
  assert(anchors.every((row) => row.ok === true), `${label}: unresolved report anchor`);
  assert(anchors.every((row) => String(row.unit ?? "").trim()), `${label}: anchor unit missing`);
  assert(anchors.every((row) => row.provenance?.path && row.provenance?.source_span), `${label}: anchor provenance missing`);
  assert(dataset.semanticCursorRuntime === "1", `${label}: semantic cursor runtime missing`);
  assert(cursor.schema === "seamgrim.semantic_cursor.v1", `${label}: semantic cursor schema`);
  assert(cursorAnchors.length >= anchors.length, `${label}: semantic cursor anchor coverage`);
  assert(cursorAnchors.every((row) => row.ok === true), `${label}: unresolved semantic cursor anchor`);
  assert(cursorAnchors.every((row) => row.source_available === true), `${label}: semantic cursor source unavailable`);
  assert(evidence.ddnTextLength > 40, `${label}: DDN source inaccessible`);
  assert(evidence.horizontalOverflow <= 2, `${label}: horizontal overflow ${evidence.horizontalOverflow}`);
  assert(dataset.mainVisualMode === expectedMode, `${label}: primary mode ${dataset.mainVisualMode} != ${expectedMode}`);
  assert(String(dataset.mainViewSource ?? "") !== "console_grid", `${label}: fell back to console grid`);
}

function assertPrimaryViewVisible(label, evidence, canvases, expectedMode) {
  if (expectedMode === "graph") {
    const pointCount = Number(evidence.dataset?.mainGraphPointCount ?? 0);
    assert(pointCount > 0, `${label}: graph point count missing`);
    assert(evidence.graphHost?.width >= 160 && evidence.graphHost?.height >= 100, `${label}: graph host too small ${JSON.stringify(evidence.graphHost)}`);
    return;
  }
  const visible = canvases.some((row) => row.width >= 160 && row.height >= 100 && row.colors >= 3);
  assert(visible, `${label}: nonblank space2d canvas missing ${JSON.stringify(canvases)}`);
}

async function verifyStability(page, label, before) {
  const beforeAnchorIds = (before.report?.anchors ?? []).map((row) => String(row.anchor_id ?? "")).join(",");
  const beforeHash = String(before.dataset?.viewPlanHash ?? "");
  const stepButton = page.locator("#btn-step");
  if (await stepButton.isEnabled()) {
    await stepButton.click();
    await page.waitForTimeout(500);
    const stepped = await readViewEvidence(page);
    const stepAnchorIds = (stepped.report?.anchors ?? []).map((row) => String(row.anchor_id ?? "")).join(",");
    assert(stepAnchorIds === beforeAnchorIds, `${label}: anchor set changed after step`);
    assert(String(stepped.dataset?.viewPlanHash ?? "") === beforeHash, `${label}: view hash changed after step`);
  } else {
    assert(before.dataset?.engineStatus === "done", `${label}: disabled step requires completed execution`);
  }
  await page.locator("#btn-reset").click();
  await page.waitForTimeout(500);
  const reset = await readViewEvidence(page);
  const resetAnchorIds = (reset.report?.anchors ?? []).map((row) => String(row.anchor_id ?? "")).join(",");
  assert(resetAnchorIds === beforeAnchorIds, `${label}: anchor set changed after reset`);
  assert(String(reset.dataset?.viewPlanHash ?? "") === beforeHash, `${label}: view hash changed after reset`);
}

async function verifyLessonViewport(page, baseUrl, lessonId, expectedMode, viewport) {
  const label = `${lessonId}:${viewport.name}`;
  await page.setViewportSize({ width: viewport.width, height: viewport.height });
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${lessonId}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
  try {
    await page.waitForFunction(
      ({ mode, lessonId }) => {
        const root = document.querySelector("#screen-run");
        const report = window.__SEAMGRIM_VIEW_PLAN_RUNTIME__ ?? null;
        const cursor = window.__SEAMGRIM_SEMANTIC_CURSOR__ ?? null;
        const anchors = Array.isArray(report?.anchors) ? report.anchors : [];
        const cursorAnchors = Array.isArray(cursor?.anchors) ? cursor.anchors : [];
        return root?.dataset?.viewPlanRuntime === "1"
          && root?.dataset?.semanticCursorRuntime === "1"
          && root?.dataset?.mainVisualMode === mode
          && report?.schema === "seamgrim.view_plan.runtime.v1"
          && report?.lesson_id === lessonId
          && anchors.length >= 2
          && anchors.every((row) => row?.ok === true)
          && cursor?.schema === "seamgrim.semantic_cursor.v1"
          && cursorAnchors.length >= anchors.length
          && cursorAnchors.every((row) => row?.ok === true && row?.source_available === true);
      },
      { mode: expectedMode, lessonId },
      { timeout: 35_000 },
    );
  } catch (error) {
    const diagnostic = await page.evaluate(() => ({
      root: { ...(document.querySelector("#screen-run")?.dataset ?? {}) },
      report: window.__SEAMGRIM_VIEW_PLAN_RUNTIME__ ?? null,
      cursor: window.__SEAMGRIM_SEMANTIC_CURSOR__ ?? null,
    }));
    throw new Error(`${label}: view readiness timeout ${JSON.stringify(diagnostic)}`, { cause: error });
  }
  await pauseRunningOrAcceptComplete(page);
  await page.waitForTimeout(100);
  const evidence = await readViewEvidence(page);
  const canvases = await readCanvasEvidence(page);
  assertSemanticEvidence(label, evidence, expectedMode);
  assertPrimaryViewVisible(label, evidence, canvases, expectedMode);
  if (viewport.name === "desktop") {
    await verifyStability(page, label, evidence);
  }
  return {
    lesson_id: lessonId,
    viewport: `${viewport.width}x${viewport.height}`,
    mode: expectedMode,
    anchors: evidence.report.anchors.length,
    view_hash: evidence.dataset.viewPlanHash,
  };
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch({ headless: true });
  const results = [];
  try {
    for (const viewport of VIEWPORTS) {
      const context = await browser.newContext({
        viewport: { width: viewport.width, height: viewport.height },
        locale: "ko-KR",
      });
      const page = await context.newPage();
      const failures = [];
      installFailureCapture(page, failures, viewport.name);
      try {
        for (const [lessonId, expectedMode] of TARGETS) {
          results.push(await verifyLessonViewport(page, baseUrl, lessonId, expectedMode, viewport));
        }
        assert(failures.length === 0, `${viewport.name}: browser failures ${failures.join(" | ")}`);
      } finally {
        await context.close();
      }
    }
  } finally {
    await browser.close();
    await closeServer(server);
  }
  console.log(`${OK} ${JSON.stringify({ cases: results.length, lessons: TARGETS.length, viewports: VIEWPORTS.length })}`);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
