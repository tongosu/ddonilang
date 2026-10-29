#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_responsive_lesson_workflow: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const LESSON_ID = "rep_phys_pendulum_wave1_v1";
const VIEWPORTS = Object.freeze([
  { name: "desktop", width: 1440, height: 900 },
  { name: "desktop_short", width: 1280, height: 720 },
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

function installFailureCapture(page, failures, viewportName) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    if (String(msg.text() ?? "").includes("Failed to load resource")) return;
    failures.push(`${viewportName}: console error: ${msg.text()}`);
  });
  page.on("pageerror", (err) => failures.push(`${viewportName}: pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`${viewportName}: request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`${viewportName}: response ${res.status()}: ${res.url()}`);
  });
}

async function canvasEvidence(page, selector) {
  return page.evaluate((sel) => {
    const canvas = document.querySelector(sel);
    if (!(canvas instanceof HTMLCanvasElement)) return null;
    const rect = canvas.getBoundingClientRect();
    const ctx = canvas.getContext("2d");
    if (!ctx || canvas.width <= 0 || canvas.height <= 0) {
      return { width: rect.width, height: rect.height, pixelWidth: canvas.width, pixelHeight: canvas.height, colors: 0 };
    }
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    const colors = new Set();
    const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 5000) * 4);
    for (let i = 0; i < data.length; i += stride) {
      colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
      if (colors.size > 16) break;
    }
    return {
      width: rect.width,
      height: rect.height,
      pixelWidth: canvas.width,
      pixelHeight: canvas.height,
      colors: colors.size,
    };
  }, selector);
}

async function readLayoutEvidence(page) {
  return page.evaluate(() => {
    function rectOf(selector) {
      const node = document.querySelector(selector);
      const rect = node?.getBoundingClientRect?.();
      return rect ? { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom, width: rect.width, height: rect.height } : null;
    }
    function overlaps(a, b) {
      if (!a || !b) return false;
      return !(a.right <= b.left || a.left >= b.right || a.bottom <= b.top || a.top >= b.bottom);
    }
      const viewport = { width: innerWidth, height: innerHeight };
      const canvas = rectOf("[data-testid='lesson-space2d-canvas']");
      const graph = rectOf("#canvas-graph");
      const graphHost = rectOf("#dotbogi-graph");
    const lessonPlan = rectOf("[data-lesson-plan-panel]:not(.hidden)");
    const semantic = rectOf("[data-semantic-cursor-panel]:not(.hidden)");
    const controlBar = rectOf(".run-control-bar");
    const dotbogi = rectOf(".dotbogi-panel");
    const overflowingLabels = [...document.querySelectorAll([
      ".run-lesson-plan-body p",
      ".run-semantic-anchor-label",
      ".run-control-bar button",
      ".run-lesson-plan-meta",
    ].join(","))]
      .filter((node) => node instanceof HTMLElement)
      .map((node) => {
        const style = getComputedStyle(node);
        return {
          text: node.textContent?.trim() ?? "",
          scrollWidth: node.scrollWidth,
          clientWidth: node.clientWidth,
          scrollHeight: node.scrollHeight,
          clientHeight: node.clientHeight,
          overflowX: style.overflowX,
          textOverflow: style.textOverflow,
        };
      })
      .filter((row) => row.scrollWidth > row.clientWidth + 3)
      .filter((row) => !(row.overflowX === "hidden" && row.textOverflow === "ellipsis"));
      return {
        viewport,
        scrollWidth: document.documentElement.scrollWidth,
        scrollHeight: document.documentElement.scrollHeight,
        classes: {
          root: document.querySelector("#screen-run")?.className ?? "",
          rootDataset: { ...(document.querySelector("#screen-run")?.dataset ?? {}) },
          layout: document.querySelector(".run-layout")?.className ?? "",
          visual: document.querySelector(".run-visual-column")?.className ?? "",
          layoutDisplay: getComputedStyle(document.querySelector(".run-layout")).display,
          visualDisplay: getComputedStyle(document.querySelector(".run-visual-column")).display,
          visualGridTemplateColumns: getComputedStyle(document.querySelector(".run-visual-column")).gridTemplateColumns,
          visualFlexDirection: getComputedStyle(document.querySelector(".run-visual-column")).flexDirection,
        },
        canvas,
        bogaeArea: rectOf(".bogae-area"),
        bogaeFrame: rectOf(".bogae-frame"),
        graph,
        graphHost,
        lessonPlan,
        semantic,
        controlBar,
        dotbogi,
      overlap: {
        canvasLessonPlan: overlaps(canvas, lessonPlan),
        canvasSemantic: overlaps(canvas, semantic),
        canvasControlBar: overlaps(canvas, controlBar),
        graphSemantic: overlaps(graph, semantic) && viewport.width > 720,
      },
      overflowingLabels,
    };
  });
}

async function stepUntil(page, predicate, label) {
  for (let i = 0; i < 90; i += 1) {
    const state = await page.evaluate(() => ({ ...(document.querySelector("#screen-run")?.dataset ?? {}) }));
    if (predicate(state)) return state;
    if (await page.locator("#btn-step").isEnabled()) {
      await page.locator("#btn-step").click();
    }
    await page.waitForTimeout(25);
  }
  const state = await page.evaluate(() => ({ ...(document.querySelector("#screen-run")?.dataset ?? {}) }));
  if (predicate(state)) return state;
  throw new Error(`timed out waiting for ${label}: ${JSON.stringify(state)}`);
}

async function selectRunTab(page, tab) {
  const button = page.locator(`#run-tab-btn-${tab}`);
  if (await button.isVisible()) await button.click();
}

async function clickRunIfEnabled(page) {
  const button = page.locator("#btn-run");
  if (await button.isEnabled()) {
    await button.click();
    return true;
  }
  return false;
}

async function pauseIfRunning(page) {
  const button = page.locator("#btn-pause");
  if (await button.isEnabled()) {
    await button.click();
    await page.waitForFunction(() => {
      const status = document.querySelector("#screen-run")?.dataset?.engineStatus ?? "";
      return status === "paused" || status === "done";
    }, null, { timeout: 10_000 });
    return true;
  }
  return false;
}

async function changeTheta(page) {
  await selectRunTab(page, "console");
  const oldLineage = await page.evaluate(() => document.querySelector("#screen-run")?.dataset?.liveTraceReactionLineageId ?? "");
  await page.locator(".slider-row[data-control-name='theta0'] input[type='range']").evaluate((input) => {
    input.value = String(Number(input.value || "0") + Number(input.step || "0.05"));
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await page.waitForFunction((oldValue) => {
    const root = document.querySelector("#screen-run");
    return root?.dataset?.liveTraceReactionLineageId
      && root.dataset.liveTraceReactionLineageId !== oldValue
      && Number(root.dataset.liveTraceReactionSampleCount || "0") >= 1;
  }, oldLineage, { timeout: 30_000 });
}

async function runLessonPlanActivities(page) {
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.lessonPlanRuntime === "1", null, { timeout: 15_000 });
  const seen = [];
  for (let i = 0; i < 4; i += 1) {
    const current = await page.evaluate(() => {
      const root = document.querySelector("#screen-run");
      return {
        id: root?.dataset?.lessonPlanCurrentActivityId ?? "",
        kind: root?.dataset?.lessonPlanActivityKind ?? "",
        anchor: root?.dataset?.lessonPlanSelectedAnchor ?? "",
      };
    });
    seen.push(current.kind);
    await page.locator("[data-lesson-plan-complete]").click();
    if (i < 3) await page.locator("[data-lesson-plan-next]").click();
  }
  assert(seen.join(",") === "observe,change,compare,explain", `lesson activities mismatch: ${seen.join(",")}`);
}

async function verifyEditorRoundtrip(page) {
  await page.locator("#screen-run .main-shell-tab[data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 10_000 });
  await page.locator("#btn-editor-lesson-plan").click();
  await page.waitForSelector("[data-lesson-plan-editor]", { state: "visible", timeout: 10_000 });
  const before = await page.locator("[data-lesson-plan-code]").inputValue();
  assert(before.includes("observe-angle-energy"), "lesson plan editor did not load current lesson plan");
  await page.locator("[data-lesson-plan-action='add']").click();
  await page.locator("[data-lesson-plan-field='title']").fill("반응형 검수 활동");
  await page.locator("[data-lesson-plan-field='title']").dispatchEvent("change");
  await page.locator("[data-lesson-plan-action='up']").click();
  await page.locator("#btn-save-ddn").click();
  await page.waitForFunction(() => (window.localStorage?.getItem?.("seamgrim.lesson_plan.v0.draft") ?? "").includes("반응형 검수 활동"));
}

async function verifyPublishSpace(page) {
  await page.locator("#screen-editor .main-shell-tab[data-shell-space='lab']").click();
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 10_000 });
  await page.locator("#screen-run .main-shell-tab[data-shell-space='publish']").click();
  await page.waitForSelector("#screen-publish:not(.hidden)", { state: "visible", timeout: 10_000 });
  await page.waitForSelector("#screen-publish [data-publish-panel]", { state: "visible", timeout: 10_000 });
  const publish = await page.evaluate(() => ({
    openVisible: document.querySelector("#screen-publish #btn-open-local-package")?.getBoundingClientRect?.().width > 0,
    pasteVisible: document.querySelector("#screen-publish #btn-paste-local-package")?.getBoundingClientRect?.().width > 0,
    copyVisible: document.querySelector("#screen-publish #btn-copy-course-package")?.getBoundingClientRect?.().width > 0,
    panelText: document.querySelector("#screen-publish [data-publish-panel]")?.textContent ?? "",
    browseHidden: document.querySelector("#screen-browse")?.classList?.contains("hidden") ?? false,
    browsePublishClass: document.querySelector("#screen-browse")?.classList?.contains("browse-space-publish") ?? false,
  }));
  assert(publish.browseHidden && !publish.browsePublishClass, `publish screen leaked browse state: ${JSON.stringify(publish)}`);
  assert(publish.openVisible, "publish open-package button not visible");
  assert(publish.pasteVisible, "publish paste toggle not visible");
  assert(publish.copyVisible, "publish course package copy not visible");
  assert(publish.panelText.includes("배포"), "publish panel text missing");
}

async function verifyViewport(page, baseUrl, viewport) {
  const started = performance.now();
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${LESSON_ID}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.semanticCursorRuntime === "1", null, { timeout: 35_000 });
  const firstOpenMs = performance.now() - started;

  await clickRunIfEnabled(page);
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.liveTraceReactionRuntime === "1", null, { timeout: 30_000 });
  const firstRunMs = performance.now() - started;
  if (await page.locator("#btn-pause").isEnabled()) await page.locator("#btn-pause").click();
  await page.waitForTimeout(100);
  await page.locator("#btn-step").click();
  await clickRunIfEnabled(page);
  await pauseIfRunning(page);
  await stepUntil(page, (state) => String(state.liveTraceReactionFiredActions ?? "").includes("highlight.theta_measure"), "live zero crossing");
  await changeTheta(page);
  const firstControlMs = performance.now() - started;

  await page.locator("button[data-semantic-anchor-action='graph'][data-anchor-id='obs.kinetic_energy']").click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.semanticCursorSelectedAnchor === "obs.kinetic_energy", null, { timeout: 10_000 });
  await runLessonPlanActivities(page);
  const firstCompareSemanticMs = performance.now() - started;

  const bogae = await canvasEvidence(page, "#canvas-bogae");
  await selectRunTab(page, "graph");
  await page.evaluate(() => window.dispatchEvent(new Event("resize")));
  try {
    await page.waitForFunction(() => {
      const canvas = document.querySelector("#canvas-graph");
      const rect = canvas?.getBoundingClientRect?.();
      return rect && rect.width > 160 && rect.height > 100;
    }, null, { timeout: 10_000 });
  } catch (error) {
    const debug = await page.evaluate(() => {
      function rectOf(selector) {
        const node = document.querySelector(selector);
        const rect = node?.getBoundingClientRect?.();
        return rect ? { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom, width: rect.width, height: rect.height } : null;
      }
      return {
        activeTab: document.querySelector("#screen-run")?.dataset?.activeSubpanelTab ?? "",
        buttonGraphHidden: document.querySelector("#run-tab-btn-graph")?.classList?.contains("hidden") ?? null,
        panelGraphHidden: document.querySelector("#run-tab-panel-graph")?.classList?.contains("hidden") ?? null,
        dotbogi: rectOf(".dotbogi-panel"),
        panel: rectOf("#run-tab-panel-graph"),
        canvas: rectOf("#canvas-graph"),
        subpanel: rectOf("#subpanel-tab-panel"),
      };
    });
    throw new Error(`${viewport.name}: graph canvas resize timeout ${JSON.stringify(debug)} ${String(error?.message ?? error)}`);
  }
  const graph = await canvasEvidence(page, "#canvas-graph");
  const minCanvasHeight = viewport.name === "desktop" ? 150 : 120;
  const layout = await readLayoutEvidence(page);
  assert(
    bogae && bogae.width > 220 && bogae.height > minCanvasHeight && bogae.colors >= 4,
    `${viewport.name}: bogae canvas invalid ${JSON.stringify({ bogae, layout })}`,
  );
  assert(
    graph && graph.width > 180 && graph.height > 110 && graph.colors >= 2,
    `${viewport.name}: graph canvas invalid ${JSON.stringify({ graph, layout })}`,
  );
  assert(layout.scrollWidth <= layout.viewport.width + 2, `${viewport.name}: horizontal overflow ${JSON.stringify(layout)}`);
  assert(!layout.overlap.canvasLessonPlan, `${viewport.name}: lesson plan overlaps canvas ${JSON.stringify(layout)}`);
  assert(!layout.overlap.canvasSemantic, `${viewport.name}: semantic panel overlaps canvas ${JSON.stringify(layout)}`);
  assert(!layout.overlap.canvasControlBar, `${viewport.name}: control bar overlaps canvas ${JSON.stringify(layout)}`);
  assert(!layout.overlap.graphSemantic, `${viewport.name}: semantic panel overlaps graph ${JSON.stringify(layout)}`);
  assert(layout.overflowingLabels.length === 0, `${viewport.name}: overflowing labels ${JSON.stringify(layout.overflowingLabels.slice(0, 3))}`);
  if (viewport.name.startsWith("desktop")) {
    assert(
      layout.graph && layout.dotbogi
        && layout.graph.top >= layout.dotbogi.top - 2
        && layout.graph.bottom <= layout.dotbogi.bottom + 2
        && layout.graphHost
        && layout.graph.top >= layout.graphHost.top - 2
        && layout.graph.bottom <= layout.graphHost.bottom + 2,
      `${viewport.name}: selected graph is clipped outside result panel ${JSON.stringify(layout)}`,
    );
  }
  const rootData = await page.evaluate(() => ({ ...(document.querySelector("#screen-run")?.dataset ?? {}) }));
  assert(rootData.viewPlanAngleMeasureVisible === "1", `${viewport.name}: angle label not visible`);
  assert(rootData.viewPlanAngleLabel, `${viewport.name}: angle label missing`);

  if (viewport.name === "desktop") {
    await verifyEditorRoundtrip(page);
    await verifyPublishSpace(page);
  }
  return {
    viewport: viewport.name,
    first_open_ms: Math.round(firstOpenMs),
    first_run_ms: Math.round(firstRunMs),
    first_control_change_ms: Math.round(firstControlMs),
    first_compare_semantic_ms: Math.round(firstCompareSemanticMs),
    bogae,
    graph,
  };
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  const reports = [];
  try {
    for (const viewport of VIEWPORTS) {
      const context = await browser.newContext({
        viewport: { width: viewport.width, height: viewport.height },
        locale: "ko-KR",
      });
      const page = await context.newPage();
      installFailureCapture(page, failures, viewport.name);
      reports.push(await verifyViewport(page, baseUrl, viewport));
      await context.close();
    }
    if (failures.length) throw new Error(failures.join("\n"));
    globalThis.process.stdout.write(`${OK} ${JSON.stringify({ reports })}\n`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
