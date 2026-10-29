#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3a_representative_viewport_gate: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const VIEWPORTS = [
  { name: "desktop", width: 1440, height: 900 },
  { name: "desktop_short", width: 1280, height: 800 },
  { name: "tablet", width: 1024, height: 768 },
  { name: "mobile", width: 390, height: 844 },
];
const LESSONS = [
  "rep_math_function_line_v1",
  "rep_phys_pendulum_wave1_v1",
  "rep_phys_projectile_xy_v1",
  "rep_phys_damped_oscillation_v1",
  "rep_econ_supply_demand_tax_v1",
];
const PRIMARY_MODES = new Map([
  ["rep_math_function_line_v1", ["graph"]],
  ["rep_phys_pendulum_wave1_v1", ["space2d", "debug-fallback"]],
  ["rep_phys_projectile_xy_v1", ["space2d", "debug-fallback"]],
  ["rep_phys_damped_oscillation_v1", ["graph"]],
  ["rep_econ_supply_demand_tax_v1", ["graph"]],
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

function installFailureCapture(page, failures, label) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    const text = String(msg.text() ?? "");
    if (text.includes("Failed to load resource")) return;
    failures.push(`${label}: console error: ${text}`);
  });
  page.on("pageerror", (err) => failures.push(`${label}: pageerror: ${err.message}`));
}

async function canvasEvidence(page) {
  return page.evaluate(() => {
    return [...document.querySelectorAll("canvas")]
      .filter((canvas) => canvas instanceof HTMLCanvasElement)
      .map((canvas) => {
        const rect = canvas.getBoundingClientRect();
        if (rect.width <= 20 || rect.height <= 20) {
          return { selector: canvas.id || canvas.dataset?.testid || "canvas", width: rect.width, height: rect.height, colors: 0 };
        }
        const ctx = canvas.getContext("2d");
        if (!ctx || canvas.width <= 0 || canvas.height <= 0) {
          return { selector: canvas.id || canvas.dataset?.testid || "canvas", width: rect.width, height: rect.height, colors: 0 };
        }
        const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
        const colors = new Set();
        const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 4000) * 4);
        for (let i = 0; i < data.length; i += stride) {
          colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
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

async function layoutEvidence(page) {
  return page.evaluate(() => {
    const rectOf = (selector) => {
      const node = document.querySelector(selector);
      const rect = node?.getBoundingClientRect?.();
      return rect ? { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom, width: rect.width, height: rect.height } : null;
    };
    const overflowing = [...document.querySelectorAll([
      ".main-shell-tab",
      ".run-control-bar button",
      "[data-curriculum-inquiry-card] span",
      "[data-curriculum-inquiry-artifact]",
    ].join(","))]
      .filter((node) => node instanceof HTMLElement)
      .map((node) => {
        const style = getComputedStyle(node);
        return {
          text: String(node.textContent ?? "").trim().slice(0, 80),
          scrollWidth: node.scrollWidth,
          clientWidth: node.clientWidth,
          overflowX: style.overflowX,
          textOverflow: style.textOverflow,
        };
      })
      .filter((row) => row.scrollWidth > row.clientWidth + 3)
      .filter((row) => !(row.overflowX === "hidden" && row.textOverflow === "ellipsis"));
    return {
      viewport: { width: innerWidth, height: innerHeight },
      scrollWidth: document.documentElement.scrollWidth,
      clientWidth: document.documentElement.clientWidth,
      runScreen: rectOf("#screen-run"),
      mainGraphHost: rectOf("#run-main-graph-host"),
      runPreview: rectOf("#run-ddn-preview"),
      inquiryPanel: rectOf("[data-curriculum-inquiry-panel]:not(.hidden)"),
      artifactPanel: rectOf("[data-curriculum-inquiry-artifact]"),
      overflowing,
    };
  });
}

async function verifyLessonViewport(page, baseUrl, lessonId, viewport) {
  const label = `${lessonId}:${viewport.name}`;
  const expectedModes = PRIMARY_MODES.get(lessonId) ?? [];
  await page.setViewportSize({ width: viewport.width, height: viewport.height });
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${lessonId}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction((id) => {
    const root = document.querySelector("#screen-run");
    return root?.dataset?.curriculumInquiryLessonId === id
      && root?.dataset?.curriculumArtifactBundle === "1"
      && root?.dataset?.curriculumModelComparison === "1"
      && root?.dataset?.curriculumSemanticScene === "1";
  }, lessonId, { timeout: 35_000 });
  await page.waitForFunction(({ modes }) => {
    const root = document.querySelector("#screen-run");
    const mode = String(root?.dataset?.mainVisualMode ?? "");
    const hasGraphPoints = Number(root?.dataset?.mainGraphPointCount || "0") > 0;
    const hasVisibleCanvas = [...document.querySelectorAll("canvas")].some((canvas) => {
      const rect = canvas.getBoundingClientRect();
      return rect.width >= 120 && rect.height >= 90;
    });
    return modes.includes(mode) && (hasGraphPoints || hasVisibleCanvas);
  }, { modes: expectedModes }, { timeout: 35_000 });
  await page.waitForTimeout(250);
  const evidence = await page.evaluate(async () => {
    const root = document.querySelector("#screen-run");
    const snapshot = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: "wave3a viewport gate" });
    return {
      dataset: { ...(root?.dataset ?? {}) },
      ddnTextLength: String(document.querySelector("#run-ddn-preview")?.value ?? "").length,
      runName: document.querySelector("#btn-run")?.textContent?.trim() ?? "",
      resetLabel: document.querySelector("#btn-reset")?.getAttribute("aria-label") ?? "",
      resetTitle: document.querySelector("#btn-reset")?.getAttribute("title") ?? "",
      stepLabel: document.querySelector("#btn-step")?.getAttribute("aria-label") ?? "",
      stepTitle: document.querySelector("#btn-step")?.getAttribute("title") ?? "",
      snapshot: snapshot.project_snapshot,
    };
  });
  assert(evidence.ddnTextLength > 40, `${label}: DDN source inaccessible`);
  assert(evidence.runName.includes("실행") || evidence.runName.includes("수업"), `${label}: run button label missing`);
  assert(evidence.resetLabel && evidence.resetTitle, `${label}: reset tooltip/aria missing`);
  assert(evidence.stepLabel && evidence.stepTitle, `${label}: step tooltip/aria missing`);
  assert(evidence.dataset.curriculumInquirySchema === "seamgrim.curriculum_inquiry_artifact.v0", `${label}: inquiry schema`);
  assert(evidence.dataset.curriculumArtifactBundleSchema === "seamgrim.curriculum_artifact_bundle.v0", `${label}: bundle schema`);
  assert(evidence.dataset.curriculumModelComparisonSchema === "seamgrim.curriculum_model_comparison.v0", `${label}: model schema`);
  assert(evidence.dataset.curriculumSemanticSceneSchema === "seamgrim.curriculum_semantic_scene.v0", `${label}: semantic scene schema`);
  assert(Number(evidence.dataset.curriculumSemanticSceneAnchorCount || "0") >= 2, `${label}: semantic anchor count`);
  assert(evidence.dataset.curriculumDeclarativeRuntimeLanded === "0", `${label}: declarative runtime overclaim`);
  assert(expectedModes.includes(String(evidence.dataset.mainVisualMode || "")), `${label}: unstable primary mode ${JSON.stringify(evidence.dataset)}`);
  assert(evidence.snapshot.curriculum_inquiry_artifact, `${label}: snapshot inquiry missing`);
  assert(evidence.snapshot.curriculum_artifact_bundle, `${label}: snapshot bundle missing`);
  assert(evidence.snapshot.curriculum_model_comparison, `${label}: snapshot model missing`);
  assert(evidence.snapshot.curriculum_semantic_scene, `${label}: snapshot semantic scene missing`);

  const layout = await layoutEvidence(page);
  const canvases = await canvasEvidence(page);
  const hasNonblankCanvas = canvases.some((row) => row.width >= 120 && row.height >= 90 && row.colors >= 2);
  const hasGraphPrimary = evidence.dataset.mainVisualMode === "graph"
    && String(evidence.dataset.requiredViews || "").includes("graph")
    && layout.mainGraphHost
    && layout.mainGraphHost.width >= 180
    && layout.mainGraphHost.height >= 120
    && String(evidence.dataset.curriculumInquiryRuntimeHash || "").includes("blake3:");
  assert(
    hasNonblankCanvas || hasGraphPrimary,
    `${label}: no visible primary view evidence ${JSON.stringify({ canvases, mainGraphHost: layout.mainGraphHost, dataset: evidence.dataset })}`,
  );
  assert(layout.scrollWidth <= layout.clientWidth + 2, `${label}: horizontal overflow ${JSON.stringify(layout)}`);
  assert(layout.runScreen && layout.runScreen.height > 260, `${label}: run screen too small ${JSON.stringify(layout)}`);
  assert(layout.inquiryPanel && layout.inquiryPanel.height > 40, `${label}: inquiry panel too small ${JSON.stringify(layout)}`);
  assert(layout.overflowing.length === 0, `${label}: overflowing text ${JSON.stringify(layout.overflowing.slice(0, 3))}`);
  return {
    lesson_id: lessonId,
    viewport: `${viewport.width}x${viewport.height}`,
    main_visual_mode: evidence.dataset.mainVisualMode,
    canvases: canvases.filter((row) => row.colors >= 2).slice(0, 3),
    ddn_text_length: evidence.ddnTextLength,
  };
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  const results = [];
  try {
    for (const viewport of VIEWPORTS) {
      const context = await browser.newContext({
        viewport: { width: viewport.width, height: viewport.height },
        locale: "ko-KR",
      });
      const page = await context.newPage();
      installFailureCapture(page, failures, viewport.name);
      for (const lessonId of LESSONS) {
        results.push(await verifyLessonViewport(page, baseUrl, lessonId, viewport));
      }
      await context.close();
    }
    if (failures.length) throw new Error(failures.join("\n"));
    console.log(JSON.stringify({
      schema: "seamgrim.wave3a_representative_viewport_gate.runner_result.v1",
      status: "ok",
      results,
    }));
    console.log(OK);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
