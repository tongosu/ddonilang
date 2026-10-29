#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const VIEWPORTS = [
  { name: "desktop", width: 1440, height: 900 },
  { name: "desktop_short", width: 1280, height: 720 },
  { name: "tablet", width: 1024, height: 768 },
  { name: "mobile", width: 390, height: 844 },
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function argValue(name, fallback = "") {
  const prefix = `${name}=`;
  const found = process.argv.find((arg) => arg === name || arg.startsWith(prefix));
  if (!found) return fallback;
  if (found === name) return "1";
  return found.slice(prefix.length);
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
        res.writeHead(403, { "cache-control": "no-store" });
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
      res.writeHead(404, { "content-type": "text/plain; charset=utf-8", "cache-control": "no-store" });
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
  page.on("requestfailed", (req) => {
    if (isAllowedFallback404(req.url())) return;
    failures.push(`${label}: request failed: ${req.url()} ${req.failure()?.errorText || ""}`);
  });
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`${label}: response ${res.status()}: ${res.url()}`);
  });
}

async function waitRunReady(page, lessonId = "rep_phys_pendulum_wave1_v1") {
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction((id) => {
    const root = document.querySelector("#screen-run");
    const source = String(document.querySelector("#run-ddn-preview")?.value ?? "");
    const hash = String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "");
    return source.trim().length > 80
      && hash.includes("blake3:")
      && (!id || new URL(location.href).searchParams.get("lesson") === id || root?.dataset?.firstDelight === "pendulum");
  }, lessonId, { timeout: 35_000 });
}

async function clickRunWhenAvailable(page) {
  const button = page.locator("#btn-run");
  await button.waitFor({ state: "visible", timeout: 10_000 });
  if (await button.isEnabled()) {
    await button.click();
    return true;
  }
  return false;
}

async function waitCardReady(page, lessonId) {
  const selector = `#screen-browse .lesson-card[data-lesson-id='${lessonId}']`;
  await page.waitForSelector(selector, { state: "visible", timeout: 25_000 });
  return selector;
}

function rectToJson(rect) {
  if (!rect) return null;
  return {
    x: Number(rect.x.toFixed(3)),
    y: Number(rect.y.toFixed(3)),
    width: Number(rect.width.toFixed(3)),
    height: Number(rect.height.toFixed(3)),
    top: Number(rect.top.toFixed(3)),
    right: Number(rect.right.toFixed(3)),
    bottom: Number(rect.bottom.toFixed(3)),
    left: Number(rect.left.toFixed(3)),
  };
}

async function canvasEvidence(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("#canvas-bogae");
    if (!(canvas instanceof HTMLCanvasElement)) return null;
    const rect = canvas.getBoundingClientRect();
    const ctx = canvas.getContext("2d");
    if (!ctx || canvas.width <= 0 || canvas.height <= 0) {
      return { rect: null, pixel_width: canvas.width, pixel_height: canvas.height, colors: 0 };
    }
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    const colors = new Set();
    const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 5000) * 4);
    for (let i = 0; i < data.length; i += stride) {
      colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
      if (colors.size > 18) break;
    }
    return {
      rect: {
        x: Number(rect.x.toFixed(3)),
        y: Number(rect.y.toFixed(3)),
        width: Number(rect.width.toFixed(3)),
        height: Number(rect.height.toFixed(3)),
        top: Number(rect.top.toFixed(3)),
        right: Number(rect.right.toFixed(3)),
        bottom: Number(rect.bottom.toFixed(3)),
        left: Number(rect.left.toFixed(3)),
      },
      pixel_width: canvas.width,
      pixel_height: canvas.height,
      colors: colors.size,
    };
  });
}

async function collectRunLayoutMetrics(page, baseUrl, viewport) {
  await page.setViewportSize({ width: viewport.width, height: viewport.height });
  await page.goto(`${baseUrl}${PROJECT_URL}?first_delight=pendulum`, { waitUntil: "domcontentloaded" });
  await waitRunReady(page);
  await page.waitForTimeout(250);
  const canvas = await canvasEvidence(page);
  const metrics = await page.evaluate(() => {
    const box = (selector) => {
      const node = document.querySelector(selector);
      if (!(node instanceof HTMLElement)) return null;
      const rect = node.getBoundingClientRect();
      return {
        x: Number(rect.x.toFixed(3)),
        y: Number(rect.y.toFixed(3)),
        width: Number(rect.width.toFixed(3)),
        height: Number(rect.height.toFixed(3)),
        top: Number(rect.top.toFixed(3)),
        right: Number(rect.right.toFixed(3)),
        bottom: Number(rect.bottom.toFixed(3)),
        left: Number(rect.left.toFixed(3)),
      };
    };
    const visible = (selector) => {
      const node = document.querySelector(selector);
      if (!(node instanceof HTMLElement)) return false;
      const rect = node.getBoundingClientRect();
      const style = getComputedStyle(node);
      return style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
    };
    const toolbar = document.querySelector(".run-control-bar");
    const primary = document.querySelector(".bogae-area");
    const editor = document.querySelector("#run-ddn-preview");
    const topbar = document.querySelector(".run-topbar");
    const lessonBrief = document.querySelector("#run-lesson-brief");
    const viewportHeight = window.innerHeight;
    const beforeVisualHeight =
      (topbar?.getBoundingClientRect?.().height ?? 0)
      + (toolbar?.getBoundingClientRect?.().height ?? 0)
      + (lessonBrief && getComputedStyle(lessonBrief).display !== "none" ? lessonBrief.getBoundingClientRect().height : 0);
    const visualRect = primary?.getBoundingClientRect?.();
    const editorRect = editor?.getBoundingClientRect?.();
    const commandButtons = ["#btn-run", "#btn-pause", "#btn-reset", "#btn-step"].filter(visible);
    const advancedVisible = [
      ".view-plan-angle-control",
      ".view-plan-measure-control",
      ".run-view-mode-group",
      "#btn-run-teacher-report-copy",
    ].filter(visible);
    return {
      document: {
        client_width: document.documentElement.clientWidth,
        scroll_width: document.documentElement.scrollWidth,
        viewport_height: viewportHeight,
      },
      toolbar: {
        rect: box(".run-control-bar"),
        client_width: toolbar?.clientWidth ?? 0,
        scroll_width: toolbar?.scrollWidth ?? 0,
        overflow_x: Math.max(0, (toolbar?.scrollWidth ?? 0) - (toolbar?.clientWidth ?? 0)),
        command_buttons_visible: commandButtons,
        advanced_controls_visible: advancedVisible,
        has_command_group: Boolean(document.querySelector("[data-run-command-group]")),
        has_view_group: Boolean(document.querySelector("[data-run-view-group]")),
        camera_visible: visible(".view-plan-camera-control"),
      },
      primary_visual: {
        rect: box(".bogae-area"),
        canvas_rect: box("#canvas-bogae"),
        starts_in_first_view: Boolean(visualRect && visualRect.top < viewportHeight),
        available_height_after_chrome: Number(Math.max(0, viewportHeight - beforeVisualHeight).toFixed(3)),
        height_ratio_after_chrome: Number(((visualRect?.height ?? 0) / Math.max(1, viewportHeight - beforeVisualHeight)).toFixed(4)),
      },
      ddn_editor: {
        rect: box("#run-ddn-preview"),
        panel_rect: box(".run-editor-panel"),
        visible: visible("#run-ddn-preview"),
        before_primary_visual: Boolean(editorRect && visualRect && editorRect.top < visualRect.top),
      },
      mobile_ddn_toggle: {
        visible: visible("#btn-run-mobile-ddn-toggle"),
        text: document.querySelector("#btn-run-mobile-ddn-toggle")?.textContent?.trim() ?? "",
      },
      lesson_brief_visible: visible("#run-lesson-brief"),
    };
  });
  return { viewport, metrics, canvas };
}

async function collectCardPreviewMetrics(page, baseUrl) {
  const lessonId = "rep_phys_pendulum_wave1_v1";
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  const cardSelector = await waitCardReady(page, lessonId);
  const before = await page.evaluate((selector) => {
    const card = document.querySelector(selector);
    const preview = card?.querySelector(".card-structure-preview");
    const visible = (node) => Boolean(node) && getComputedStyle(node).display !== "none" && node.getBoundingClientRect().height > 0;
    const detailSelectors = [
      "[data-course-result]",
      "[data-course-delivery]",
      "[data-course-readiness]",
      "[data-course-activity]",
      "[data-course-goals]",
      "[data-course-missions]",
    ];
    return {
      card: card?.getBoundingClientRect?.().toJSON?.() ?? null,
      preview: preview?.getBoundingClientRect?.().toJSON?.() ?? null,
      hidden: preview?.classList?.contains("hidden") ?? null,
      text: preview?.textContent?.trim() ?? "",
      previewState: preview?.getAttribute("data-preview-state") ?? "",
      launchActions: Array.from(card?.querySelectorAll(".card-launch-btn") ?? []).map((node) => ({
        action: node.getAttribute("data-launch-action") ?? "",
        text: node.textContent?.trim() ?? "",
      })),
      visibleDetailFields: detailSelectors.filter((item) => visible(card?.querySelector(item))),
    };
  }, cardSelector);
  let hydrationTimedOut = false;
  try {
    await page.waitForFunction((selector) => {
      const preview = document.querySelector(selector)?.querySelector(".card-structure-preview");
      return preview instanceof HTMLElement && !preview.classList.contains("hidden") && preview.getBoundingClientRect().height > 0;
    }, cardSelector, { timeout: 20_000 });
  } catch (_) {
    hydrationTimedOut = true;
  }
  const after = await page.evaluate((selector) => {
    const card = document.querySelector(selector);
    const preview = card?.querySelector(".card-structure-preview");
    return {
      card: card?.getBoundingClientRect?.().toJSON?.() ?? null,
      preview: preview?.getBoundingClientRect?.().toJSON?.() ?? null,
      hidden: preview?.classList?.contains("hidden") ?? null,
      text: preview?.textContent?.trim() ?? "",
      previewState: preview?.getAttribute("data-preview-state") ?? "",
    };
  }, cardSelector);
  return {
    lesson_id: lessonId,
    before: {
      card: rectToJson(before.card),
      preview: rectToJson(before.preview),
      hidden: before.hidden,
      text: before.text,
      preview_state: before.previewState,
      launch_actions: before.launchActions,
      visible_detail_fields: before.visibleDetailFields,
    },
    after: {
      card: rectToJson(after.card),
      preview: rectToJson(after.preview),
      hidden: after.hidden,
      text: after.text,
      preview_state: after.previewState,
    },
    card_height_shift: Number(Math.abs((after.card?.height ?? 0) - (before.card?.height ?? 0)).toFixed(3)),
    preview_height_shift: Number(Math.abs((after.preview?.height ?? 0) - (before.preview?.height ?? 0)).toFixed(3)),
    hydration_timed_out: hydrationTimedOut,
  };
}

function classifyFailures(measurement) {
  const failures = [];
  for (const row of measurement.viewports) {
    const { viewport, metrics, canvas } = row;
    const label = `${viewport.width}x${viewport.height}`;
    if (metrics.toolbar.overflow_x > 2) failures.push({ code: "toolbar_overflow", viewport: label, value: metrics.toolbar.overflow_x });
    if (viewport.width >= 1280 && !metrics.toolbar.has_command_group) failures.push({ code: "toolbar_command_group_missing", viewport: label });
    if (viewport.width >= 1280 && !metrics.toolbar.has_view_group) failures.push({ code: "toolbar_view_group_missing", viewport: label });
    if (viewport.width >= 1280 && !metrics.toolbar.camera_visible) failures.push({ code: "camera_not_visible_desktop", viewport: label });
    if (viewport.width === 1440 && metrics.primary_visual.height_ratio_after_chrome < 0.55) {
      failures.push({ code: "primary_visual_ratio_below_55", viewport: label, value: metrics.primary_visual.height_ratio_after_chrome });
    }
    if (viewport.width === 1024 && (metrics.primary_visual.rect?.height ?? 0) < 360) {
      failures.push({ code: "tablet_primary_visual_height_below_360", viewport: label, value: metrics.primary_visual.rect?.height ?? 0 });
    }
    if (viewport.width === 1024 && metrics.ddn_editor.before_primary_visual) failures.push({ code: "tablet_ddn_before_visual", viewport: label });
    if (viewport.width === 390) {
      const primaryHeight = metrics.primary_visual.rect?.height ?? 0;
      if (!metrics.primary_visual.starts_in_first_view) failures.push({ code: "mobile_visual_starts_below_fold", viewport: label });
      if (primaryHeight < 360 && primaryHeight / viewport.height < 0.55) {
        failures.push({ code: "mobile_primary_visual_too_short", viewport: label, value: primaryHeight });
      }
      if (metrics.toolbar.advanced_controls_visible.length > 0) {
        failures.push({ code: "mobile_advanced_controls_not_collapsed", viewport: label, value: metrics.toolbar.advanced_controls_visible });
      }
    }
    if (!canvas || canvas.colors <= 4) failures.push({ code: "canvas_blank", viewport: label, value: canvas });
  }
  const card = measurement.card_preview;
  if ((card.before.preview?.height ?? 0) <= 0 || card.before.hidden) failures.push({ code: "card_preview_space_not_reserved" });
  if (card.hydration_timed_out) failures.push({ code: "card_preview_hydration_timeout" });
  if (!card.before.launch_actions?.some((item) => item.action === "learn") || !card.before.launch_actions?.some((item) => item.action === "lab")) {
    failures.push({ code: "card_launch_commands_not_split", value: card.before.launch_actions });
  }
  if ((card.before.visible_detail_fields ?? []).length > 0) {
    failures.push({ code: "card_detail_fields_visible_by_default", value: card.before.visible_detail_fields });
  }
  if (card.card_height_shift > 2 || card.preview_height_shift > 2) {
    failures.push({
      code: "card_preview_layout_shift",
      card_height_shift: card.card_height_shift,
      preview_height_shift: card.preview_height_shift,
    });
  }
  return failures;
}

async function collectMeasurement(baseUrl, browser) {
  const context = await browser.newContext({ locale: "ko-KR" });
  const page = await context.newPage();
  const failures = [];
  installFailureCapture(page, failures, "wave2d");
  const viewports = [];
  for (const viewport of VIEWPORTS) {
    viewports.push(await collectRunLayoutMetrics(page, baseUrl, viewport));
  }
  const cardPreview = await collectCardPreviewMetrics(page, baseUrl);
  await context.close();
  const measurement = {
    schema: "seamgrim.wave2d.first_view.measurement.v1",
    runner: "tests/seamgrim_wave2d_first_view_authoring_runner.mjs",
    product_url: "solutions/seamgrim_ui_mvp/ui/index.html",
    viewports,
    card_preview: cardPreview,
    console_failures: failures,
  };
  return {
    ...measurement,
    failure_vectors: classifyFailures(measurement),
  };
}

function findViewport(measurement, width, height) {
  const row = measurement.viewports.find((item) => item.viewport.width === width && item.viewport.height === height);
  assert(row, `missing viewport ${width}x${height}`);
  return row;
}

async function verifyWorkflow(baseUrl, browser) {
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, locale: "ko-KR" });
  const page = await context.newPage();
  const failures = [];
  installFailureCapture(page, failures, "workflow");
  const lessonId = "rep_phys_pendulum_wave1_v1";

  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  const cardSelector = await waitCardReady(page, lessonId);
  await page.locator(`${cardSelector} .card-launch-primary`).click();
  await waitRunReady(page, lessonId);
  assert((await canvasEvidence(page))?.colors > 4, "workflow: pendulum canvas is blank");

  await clickRunWhenAvailable(page);
  await page.waitForFunction(() => ["running", "paused", "done"].includes(document.querySelector("#screen-run")?.dataset?.engineStatus ?? ""), null, { timeout: 15_000 });
  const thetaInput = page.locator("input[data-control-name='theta0']").first();
  await thetaInput.waitFor({ state: "visible", timeout: 20_000 });
  await thetaInput.fill("0.35");
  await thetaInput.dispatchEvent("input");
  await thetaInput.dispatchEvent("change");
  await page.waitForFunction(() => String(document.querySelector("#run-observe-output-body")?.dataset?.observationPayloadJson ?? "").includes("theta"), null, { timeout: 20_000 });
  await page.waitForFunction(() => String(document.querySelector("#screen-run")?.dataset?.mainViewRoles ?? "").includes("angle_measure"), null, { timeout: 20_000 });

  const toggle = page.locator("#btn-run-mobile-ddn-toggle");
  await page.setViewportSize({ width: 390, height: 844 });
  await toggle.click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.mobileDdnOpen === "1", null, { timeout: 10_000 });
  const mobileEditor = await page.locator("#run-ddn-preview").boundingBox();
  assert(mobileEditor && mobileEditor.width >= 355 && mobileEditor.height >= 240, `workflow: mobile DDN size invalid ${JSON.stringify(mobileEditor)}`);
  await toggle.click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.mobileDdnOpen === "0", null, { timeout: 10_000 });

  await page.setViewportSize({ width: 1440, height: 900 });
  await page.locator("#screen-run .main-shell-tab[data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 15_000 });
  await page.locator("#btn-editor-world-code").click();
  const authoringMetrics = await page.evaluate(() => {
    const rectOf = (selector) => {
      const node = document.querySelector(selector);
      const rect = node?.getBoundingClientRect?.();
      return rect ? { width: rect.width, height: rect.height, top: rect.top, left: rect.left } : null;
    };
    return {
      screen: rectOf("#screen-editor"),
      workspace: rectOf("#ddn-editor-area"),
      code: rectOf("#ddn-textarea"),
      worldTabSelected: document.querySelector("#btn-editor-world-code")?.getAttribute("aria-selected") ?? "",
      planTabSelected: document.querySelector("#btn-editor-lesson-plan")?.getAttribute("aria-selected") ?? "",
      expandedButtonVisible: (document.querySelector("#btn-editor-expand")?.getBoundingClientRect?.().width ?? 0) > 0,
    };
  });
  assert(authoringMetrics.code?.width >= 1440 * 0.45, `workflow: authoring code width below 45% ${JSON.stringify(authoringMetrics)}`);
  assert(authoringMetrics.code?.height >= 480, `workflow: authoring code height below 480 ${JSON.stringify(authoringMetrics)}`);
  assert(authoringMetrics.worldTabSelected === "true" && authoringMetrics.planTabSelected === "false", `workflow: world/plan tabs not coherent ${JSON.stringify(authoringMetrics)}`);
  assert(authoringMetrics.expandedButtonVisible, "workflow: editor expand command missing");
  const editorBefore = await page.locator("#ddn-textarea").inputValue();
  assert(editorBefore.includes("theta") || editorBefore.includes("진자"), "workflow: editor did not preserve lesson source");
  await page.locator("#ddn-textarea").fill(`${editorBefore}\n// Wave2D browser workflow edit`);
  await page.locator("#ddn-textarea").dispatchEvent("input");
  await page.locator("#btn-editor-lesson-plan").click();
  await page.waitForSelector("[data-lesson-plan-editor]", { state: "visible", timeout: 15_000 });
  const titleField = page.locator("[data-lesson-plan-field='title']").first();
  await titleField.fill("Wave2D 활동 수정");
  await titleField.dispatchEvent("change");
  await page.waitForFunction(() => String(document.querySelector("[data-lesson-plan-code]")?.value ?? "").includes("Wave2D 활동 수정"), null, { timeout: 10_000 });
  await page.locator("#btn-editor-world-code").click();
  await page.locator("#btn-run-from-editor").click();
  await waitRunReady(page, "");
  await page.waitForFunction(() => String(document.querySelector("#run-ddn-preview")?.value ?? "").includes("Wave2D browser workflow edit"), null, { timeout: 20_000 });
  assert((await canvasEvidence(page))?.colors > 4, "workflow: run after editor edit is blank");

  assert(failures.length === 0, failures.join("\n"));
  await context.close();
}

function assertGate(measurement) {
  assert(measurement.console_failures.length === 0, `console/browser failures: ${measurement.console_failures.join("\n")}`);
  assert(measurement.failure_vectors.length === 0, `layout failure vectors remain: ${JSON.stringify(measurement.failure_vectors, null, 2)}`);
  const desktop = findViewport(measurement, 1440, 900);
  const desktopShort = findViewport(measurement, 1280, 720);
  const tablet = findViewport(measurement, 1024, 768);
  const mobile = findViewport(measurement, 390, 844);
  assert(desktop.metrics.toolbar.has_command_group && desktop.metrics.toolbar.has_view_group, "desktop toolbar groups missing");
  assert(desktopShort.metrics.toolbar.overflow_x <= 2, "1280 toolbar overflow");
  assert((desktop.metrics.ddn_editor.rect?.width ?? 0) >= 420, "desktop run DDN width below 420");
  assert((tablet.metrics.primary_visual.rect?.height ?? 0) >= 360, "tablet primary visual below 360px");
  assert(!tablet.metrics.ddn_editor.before_primary_visual, "tablet DDN appears before primary visual");
  assert((mobile.metrics.primary_visual.rect?.height ?? 0) >= 360 || mobile.metrics.primary_visual.height_ratio_after_chrome >= 0.55, "mobile primary visual too short");
  assert((measurement.card_preview.before.preview?.height ?? 0) > 0, "card preview space not reserved before async load");
}

async function main() {
  const mode = argValue("--mode", "gate");
  const outPath = argValue("--out", "");
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  try {
    const measurement = await collectMeasurement(baseUrl, browser);
    if (mode === "workflow") {
      await verifyWorkflow(baseUrl, browser);
      process.stdout.write("seamgrim_wave2d_first_view_workflow: ok\n");
      return;
    }
    if (mode === "measure") {
      const text = `${JSON.stringify(measurement, null, 2)}\n`;
      if (outPath) await fs.writeFile(outPath, text, "utf8");
      else process.stdout.write(text);
      return;
    }
    assertGate(measurement);
    await verifyWorkflow(baseUrl, browser);
    process.stdout.write("seamgrim_wave2d_first_view_authoring: ok\n");
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
