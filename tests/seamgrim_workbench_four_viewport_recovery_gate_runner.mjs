#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_workbench_four_viewport_recovery_gate: ok";
const VIEWPORTS = [
  { width: 1440, height: 900 },
  { width: 1024, height: 768 },
  { width: 778, height: 912 },
  { width: 390, height: 844 },
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  return "application/octet-stream";
}

async function serve(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      const relative = decodeURIComponent(url.pathname === "/"
        ? "/solutions/seamgrim_ui_mvp/ui/workbench/index.html"
        : url.pathname).replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, relative);
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
        res.writeHead(403).end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
      res.end(bytes);
    } catch (_) {
      res.writeHead(404).end("not found");
    }
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert(address && typeof address !== "string", "server address unavailable");
  return { server, baseUrl: `http://127.0.0.1:${address.port}` };
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function snapshot(page) {
  return page.evaluate(() => {
    const result = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    const primary = document.querySelector("[data-testid='lifecycle-run']");
    return {...result, primaryControl: {action: primary?.dataset.action, enabled: primary?.disabled === false}};
  });
}

async function waitReady(page) {
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snap?.readiness?.initialization === "ready";
  }, null, { timeout: 30000 });
}

async function canvasEvidence(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const rect = canvas?.getBoundingClientRect();
    if (!canvas || !rect) return null;
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let hash = 0x811c9dc5;
    let nonWhite = 0;
    for (let i = 0; i < data.length; i += 4) {
      const r = data[i] ?? 0;
      const g = data[i + 1] ?? 0;
      const b = data[i + 2] ?? 0;
      const a = data[i + 3] ?? 0;
      if (a && (r < 240 || g < 240 || b < 240)) nonWhite += 1;
      hash ^= r; hash = Math.imul(hash, 0x01000193);
      hash ^= g; hash = Math.imul(hash, 0x01000193);
      hash ^= b; hash = Math.imul(hash, 0x01000193);
      hash ^= a; hash = Math.imul(hash, 0x01000193);
    }
    return {
      width: canvas.width,
      height: canvas.height,
      rect: { left: rect.left, top: rect.top, width: rect.width, height: rect.height },
      non_white: nonWhite,
      pixel_hash: `fnv1a32:${(hash >>> 0).toString(16).padStart(8, "0")}`,
      state_hash: canvas.dataset.committedStateHash ?? "",
      semantic_payload_hash: canvas.dataset.semanticPayloadHash ?? "",
    };
  });
}

async function waitForCommittedCount(page, minimum) {
  await page.waitForFunction(
    (target) => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const run = snap?.runs?.find((row) => row.run_id === snap.activeRunId) ?? snap?.runs?.at(-1);
      return Number(run?.committed_step_count ?? 0) >= target && Boolean(snap?.activeFrame);
    },
    minimum,
    { timeout: 60000 },
  );
}

async function pause(page) {
  await page.click("[data-testid='lifecycle-run'][data-action='pause']");
  await page.waitForFunction(
    () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "paused",
    null,
    { timeout: 60000 },
  );
}

async function stepOnce(page, previousFrameId) {
  await page.click("[data-testid='lifecycle-step']");
  await page.waitForFunction(
    (frameId) => Number(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.activeFrame?.frame_id ?? -1) > frameId,
    previousFrameId,
    { timeout: 60000 },
  );
}

function rectOf(map, key) {
  return map[key] ?? { left: 0, top: 0, right: 0, bottom: 0, width: 0, height: 0 };
}

async function layoutMetrics(page) {
  return page.evaluate(() => {
    const rect = (testId) => {
      const node = document.querySelector(`[data-testid='${testId}']`);
      const box = node?.getBoundingClientRect();
      return box ? { left: box.left, top: box.top, right: box.right, bottom: box.bottom, width: box.width, height: box.height } : null;
    };
    const visible = (testId) => {
      const node = document.querySelector(`[data-testid='${testId}']`);
      if (!node) return false;
      const closedDetails = node.closest("details:not([open])");
      if (closedDetails && node.tagName.toLowerCase() !== "summary") return false;
      const style = getComputedStyle(node);
      const box = node.getBoundingClientRect();
      return style.display !== "none" && style.visibility !== "hidden" && box.width > 0 && box.height > 0;
    };
    const managementInputs = [
      "portable-meta-title",
      "portable-meta-summary",
      "portable-meta-content-kind",
      "portable-meta-tags",
      "portable-meta-subject",
      "portable-meta-level",
      "portable-meta-required-views",
      "portable-meta-required-gaji",
      "project-conflict-local-package",
      "gaji-shelf-search",
      "gaji-install-alias",
      "gaji-bundle-id",
      "gaji-remix-source",
    ].filter(visible);
    const boxes = Object.fromEntries([
      "source-panel",
      "source-editor",
      "bogae-panel",
      "bogae-canvas-wrap",
      "side-view-panel",
      "run-status",
      "control-grid",
    ].map((id) => [id, rect(id)]));
    return {
      viewport: { width: innerWidth, height: innerHeight },
      scrollWidth: document.documentElement.scrollWidth,
      clientWidth: document.documentElement.clientWidth,
      boxes,
      managementInputs,
      visibleActionControls: visible("action-controls"),
      gajiShelfVisible: visible("gaji-shelf-dialog"),
      magnifierInstances: Array.from(document.querySelectorAll("[data-view-instance-role='magnifier']")).filter((node) => {
        const style = getComputedStyle(node);
        const box = node.getBoundingClientRect();
        return style.display !== "none" && style.visibility !== "hidden" && box.width > 0 && box.height > 0;
      }).length,
    };
  });
}

async function overlapReport(page) {
  return page.evaluate(() => {
    const nodes = Array.from(document.querySelectorAll("button, input, textarea, select, label, a"));
    const visible = nodes
      .map((node, index) => {
        const style = getComputedStyle(node);
        const rect = node.getBoundingClientRect();
        const text = String(node.getAttribute("data-testid") || node.getAttribute("aria-label") || node.textContent || node.tagName).replace(/\s+/g, " ").trim();
        const closedDetails = node.closest("details:not([open])");
        const centerX = rect.left + rect.width / 2;
        const centerY = rect.top + rect.height / 2;
        const hit = centerX >= 0 && centerY >= 0 && centerX <= innerWidth && centerY <= innerHeight
          ? document.elementFromPoint(centerX, centerY)
          : null;
        const hitVisible = Boolean(hit && (node === hit || node.contains(hit) || hit.contains(node)));
        return { node, index, text, rect, display: style.display, visibility: style.visibility, hiddenByDetails: Boolean(closedDetails && node.tagName.toLowerCase() !== "summary"), hitVisible };
      })
      .filter((row) => row.hitVisible && !row.hiddenByDetails && row.display !== "none" && row.visibility !== "hidden" && row.rect.width > 1 && row.rect.height > 1);
    const overlaps = [];
    for (let i = 0; i < visible.length; i += 1) {
      for (let j = i + 1; j < visible.length; j += 1) {
        const a = visible[i];
        const b = visible[j];
        if (a.node.contains(b.node) || b.node.contains(a.node)) continue;
        const x = Math.max(0, Math.min(a.rect.right, b.rect.right) - Math.max(a.rect.left, b.rect.left));
        const y = Math.max(0, Math.min(a.rect.bottom, b.rect.bottom) - Math.max(a.rect.top, b.rect.top));
        if (x * y > 48) overlaps.push({ a: a.text, b: b.text, area: Math.round(x * y) });
      }
    }
    return overlaps;
  });
}

async function assertFirstScreen(page, viewport) {
  const metrics = await layoutMetrics(page);
  const source = rectOf(metrics.boxes, "source-panel");
  const primary = rectOf(metrics.boxes, "bogae-panel");
  const canvas = rectOf(metrics.boxes, "bogae-canvas-wrap");
  assert(source.width > 0 && source.height > 0, `${viewport.width}x${viewport.height}: DDN source inaccessible`);
  assert(primary.width > 0 && primary.height > 0, `${viewport.width}x${viewport.height}: primary view inaccessible`);
  assert(canvas.width > 0 && canvas.height > 0, `${viewport.width}x${viewport.height}: canvas collapsed`);
  if (viewport.width > 720) {
    assert(source.top < metrics.viewport.height && primary.top < metrics.viewport.height, `${viewport.width}x${viewport.height}: source/primary not in first viewport`);
  } else {
    assert(source.top < metrics.viewport.height * 1.25, `${viewport.width}x${viewport.height}: DDN source not reachable near the first screen`);
    assert(primary.top < metrics.viewport.height * 1.75, `${viewport.width}x${viewport.height}: primary view not reachable near the first screen`);
  }
  assert(metrics.managementInputs.length === 0, `${viewport.width}x${viewport.height}: management inputs visible: ${metrics.managementInputs.join(", ")}`);
  assert(metrics.visibleActionControls === false, `${viewport.width}x${viewport.height}: action controls visible for pendulum without action map`);
  assert(metrics.scrollWidth <= metrics.clientWidth + 2, `${viewport.width}x${viewport.height}: horizontal overflow ${metrics.scrollWidth} > ${metrics.clientWidth}`);
  assert(metrics.magnifierInstances === 0, `${viewport.width}x${viewport.height}: magnifier instance visible before target selection`);
  const sourceText = await page.locator("[data-testid='source-editor']").inputValue();
  assert(sourceText.includes("진자") || sourceText.includes("마디"), `${viewport.width}x${viewport.height}: DDN source text unavailable`);
  const overlaps = await overlapReport(page);
  assert(overlaps.length === 0, `${viewport.width}x${viewport.height}: overlapping controls ${JSON.stringify(overlaps.slice(0, 5))}`);
  return metrics;
}

async function assertProjectControlsRelevant(page, viewport) {
  const controls = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectControls);
  const names = controls.controls.map((control) => control.name);
  assert(names.includes("g") && names.includes("L"), `${viewport.width}x${viewport.height}: pendulum controls missing: ${names.join(",")}`);
  for (const unrelated of ["초기속도x", "초기속도y", "left", "right", "drop"]) {
    assert(!names.includes(unrelated), `${viewport.width}x${viewport.height}: unrelated control leaked: ${unrelated}`);
  }
  assert((controls.actions ?? []).length === 0, `${viewport.width}x${viewport.height}: action map leaked into pendulum`);
  const arrangement = await page.evaluate(() => {
    const box = selector => {
      const rect = document.querySelector(selector)?.getBoundingClientRect();
      return rect ? {top: rect.top, bottom: rect.bottom, left: rect.left, right: rect.right} : null;
    };
    const title = document.querySelector("[data-testid='bogae-panel'] > .panel-header > strong");
    const range = document.createRange();
    range.selectNodeContents(title);
    return {
      title: box("[data-testid='setup-controls'] .control-title"),
      input: box("[data-testid='setup-control-list'] input"),
      save: box("[data-testid='setup-apply']"),
      bogaeTitleLines: range.getClientRects().length,
    };
  });
  assert(arrangement.title && arrangement.input && arrangement.save,
    `${viewport.width}x${viewport.height}: initial-value header, input or save missing`);
  assert(arrangement.title.bottom <= arrangement.input.top + 1,
    `${viewport.width}x${viewport.height}: initial-value title is not above inputs ${JSON.stringify(arrangement)}`);
  assert(arrangement.save.top < arrangement.title.bottom && arrangement.save.bottom > arrangement.title.top
    && arrangement.save.bottom <= arrangement.input.top + 1,
    `${viewport.width}x${viewport.height}: save button is not in initial-value header row ${JSON.stringify(arrangement)}`);
  assert(arrangement.bogaeTitleLines === 1,
    `${viewport.width}x${viewport.height}: Bogae title wraps ${JSON.stringify(arrangement)}`);
  const scroll = await page.getByTestId('control-grid').evaluate(node => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const canvasBefore = canvas.getBoundingClientRect().height;
    const maximum = Number.parseFloat(getComputedStyle(node).maxHeight);
    const oldTop = node.scrollTop;
    node.scrollTop = node.scrollHeight;
    const endTop = node.scrollTop;
    const result = {height: node.clientHeight, maximum, overflow: node.scrollHeight - node.clientHeight,
      endTop, canvasBefore, canvasAfter: canvas.getBoundingClientRect().height};
    node.scrollTop = oldTop;
    return result;
  });
  assert(Number.isFinite(scroll.maximum) && scroll.height <= scroll.maximum + 1,
    `${viewport.width}x${viewport.height}: controls have no bounded height ${JSON.stringify(scroll)}`);
  assert(scroll.overflow <= 1 || scroll.endTop > 0,
    `${viewport.width}x${viewport.height}: overflowing controls cannot scroll ${JSON.stringify(scroll)}`);
  assert(scroll.canvasAfter === scroll.canvasBefore, `${viewport.width}x${viewport.height}: scrolling controls changed Bogae height`);
}

async function assertLiveCanvasAndViews(page, viewport) {
  await page.click("[data-testid='lifecycle-run']");
  await waitForCommittedCount(page, 3);
  const first = await canvasEvidence(page);
  assert(first?.width > 0 && first?.height > 0, `${viewport.width}x${viewport.height}: canvas has zero backing size`);
  assert(first.non_white > 500, `${viewport.width}x${viewport.height}: canvas blank`);
  await waitForCommittedCount(page, 5);
  const second = await canvasEvidence(page);
  assert(second.pixel_hash !== first.pixel_hash, `${viewport.width}x${viewport.height}: canvas pixel hash did not change over time`);
  await pause(page);
  const paused = await snapshot(page);
  assert(paused.primaryControl.action === "resume" && paused.primaryControl.enabled, `${viewport.width}x${viewport.height}: resume unavailable after pause`);
  assert(paused.controls.lifecycle_step.enabled === true, `${viewport.width}x${viewport.height}: step unavailable after pause`);
  assert(paused.controls.lifecycle_stop.enabled === true, `${viewport.width}x${viewport.height}: stop unavailable after pause`);
  assert(paused.primaryControl.action !== "start", `${viewport.width}x${viewport.height}: unchanged paused source must resume instead of starting a new run`);
  assert(paused.viewWorkspace.instances.every((instance) => instance.role !== "magnifier"), `${viewport.width}x${viewport.height}: magnifier exists before target selection`);
  assert(await page.locator("[data-testid='view-role-magnifier']").isDisabled(), `${viewport.width}x${viewport.height}: magnifier button enabled before target selection`);

  const beforeSide = await layoutMetrics(page);
  await page.click("[data-testid='view-role-side']");
  await page.waitForFunction(() => document.querySelector("[data-testid='bogae-stage']")?.dataset.sideOpen === "true");
  const sideOpen = await layoutMetrics(page);
  assert(sideOpen.boxes["bogae-canvas-wrap"].width > 0, `${viewport.width}x${viewport.height}: side open collapsed primary canvas`);
  await page.click("[data-testid='side-view-close']");
  await page.waitForFunction(() => document.querySelector("[data-testid='bogae-stage']")?.dataset.sideOpen === "false");
  const sideClosed = await layoutMetrics(page);
  if (viewport.width >= 778) {
    assert(
      sideClosed.boxes["bogae-canvas-wrap"].width >= sideOpen.boxes["bogae-canvas-wrap"].width,
      `${viewport.width}x${viewport.height}: primary did not refit after side close`,
    );
  } else {
    assert(
      sideOpen.boxes["side-view-panel"].top >= sideOpen.boxes["bogae-canvas-wrap"].bottom - 2,
      `${viewport.width}x${viewport.height}: narrow side view did not stack below primary`,
    );
  }

  const overlayOffPixels = await canvasEvidence(page);
  const overlayOffStateHash = paused.activeFrame.state_hash;
  await page.click("[data-testid='view-role-overlay']");
  const overlayBefore = await snapshot(page);
  assert(overlayBefore.semanticRenderer.overlay_enabled === true, `${viewport.width}x${viewport.height}: overlay not enabled`);
  assert(overlayBefore.semanticRenderer.overlay_drawlist?.length > 0, `${viewport.width}x${viewport.height}: overlay drawlist absent ${JSON.stringify(overlayBefore.semanticRenderer)}`);
  assert(overlayBefore.semanticRenderer.overlay_world_target != null, `${viewport.width}x${viewport.height}: overlay target absent`);
  const overlayOnPixels = await canvasEvidence(page);
  assert(overlayOnPixels.pixel_hash !== overlayOffPixels.pixel_hash, `${viewport.width}x${viewport.height}: overlay did not change actual pixels`);
  assert(overlayBefore.activeFrame.state_hash === overlayOffStateHash, `${viewport.width}x${viewport.height}: enabling overlay changed runtime state`);
  await page.click("[data-testid='view-role-overlay']");
  const restoredOff = await canvasEvidence(page);
  assert(restoredOff.pixel_hash === overlayOffPixels.pixel_hash, `${viewport.width}x${viewport.height}: hiding overlay did not restore the same paused pixels`);
  assert((await snapshot(page)).activeFrame.state_hash === overlayOffStateHash, `${viewport.width}x${viewport.height}: hiding overlay changed runtime state`);
  await page.click("[data-testid='view-role-overlay']");
  const projectionBefore = JSON.stringify((await snapshot(page)).semanticRenderer.overlay_world_target);
  await stepOnce(page, overlayBefore.activeFrame.frame_id);
  const overlayAfter = await snapshot(page);
  const projectionAfter = JSON.stringify(overlayAfter.semanticRenderer.overlay_world_target);
  assert(overlayAfter.semanticRenderer.overlay_enabled === true, `${viewport.width}x${viewport.height}: overlay disabled during step`);
  assert(projectionBefore !== projectionAfter, `${viewport.width}x${viewport.height}: overlay target did not move with primary coordinates`);
  await page.click("[data-testid='view-role-overlay']");
  await page.click("[data-testid='lifecycle-stop']");
  await page.waitForFunction(
    () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "done",
    null,
    { timeout: 60000 },
  );
  return { first, second };
}

async function inspectViewport(browser, baseUrl, root, outDir, viewport) {
  const context = await browser.newContext({ viewport, locale: "ko-KR" });
  const page = await context.newPage();
  const browserIssues = [];
  page.on("console", (msg) => {
    if (["error", "warning"].includes(msg.type())) {
      const location = msg.location();
      browserIssues.push(`${msg.type()}: ${msg.text()} (${location.url}:${location.lineNumber}:${location.columnNumber})`);
    }
  });
  page.on("pageerror", (err) => browserIssues.push(`pageerror: ${err.message}`));
  page.on("response", (res) => {
    if (res.status() >= 400) browserIssues.push(`response ${res.status()}: ${res.url()}`);
  });
  try {
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await waitReady(page);
    if (!await page.getByTestId('project-card-rep_phys_pendulum_wave1_v1').isVisible()) {
      await page.getByTestId('project-rail-toggle').click();
    }
    await page.click("[data-testid='project-card-rep_phys_pendulum_wave1_v1']");
    await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_pendulum_wave1_v1']");
    await assertFirstScreen(page, viewport);
    await assertProjectControlsRelevant(page, viewport);
    const canvas = await assertLiveCanvasAndViews(page, viewport);
    assert(browserIssues.length === 0, `${viewport.width}x${viewport.height}: browser issues ${browserIssues.join(" | ")}`);
    const screenshot = path.join(outDir, `workbench_g13_${viewport.width}x${viewport.height}.png`);
    await page.screenshot({ path: screenshot, fullPage: true });
    return {
      viewport: `${viewport.width}x${viewport.height}`,
      screenshot: path.relative(root, screenshot).replace(/\\/g, "/"),
      canvas,
      browser_issues: browserIssues,
    };
  } finally {
    await context.close();
  }
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const outDir = path.resolve(root, "out", "seamgrim_workbench_g13_layout_r2");
  await fs.mkdir(outDir, { recursive: true });
  const { server, baseUrl } = await serve(root);
  let browser = null;
  try {
    browser = await chromium.launch({ headless: true,
      ...(process.env.CHROME_EXECUTABLE ? { executablePath: process.env.CHROME_EXECUTABLE } : {}) });
    const results = [];
    for (const viewport of VIEWPORTS) {
      results.push(await inspectViewport(browser, baseUrl, root, outDir, viewport));
    }
    const summary = {
      schema: "seamgrim.workbench_four_viewport_recovery_gate.runner_result.v1",
      viewports: results.map((row) => row.viewport),
      results,
    };
    await fs.writeFile(path.join(outDir, "summary.detjson"), `${JSON.stringify(summary, null, 2)}\n`, "utf8");
    console.log(JSON.stringify(summary));
    console.log(OK);
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
