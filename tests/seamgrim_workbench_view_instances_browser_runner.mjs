#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

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
  const browserUnsafePorts = new Set([6000, 6665, 6666, 6667, 6668, 6669, 6697]);
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
  let address = null;
  do {
    await new Promise((resolve, reject) => {
      server.once("error", reject);
      server.listen(0, "127.0.0.1", resolve);
    });
    address = server.address();
    if (address && typeof address !== "string" && browserUnsafePorts.has(address.port)) {
      await new Promise((resolve) => server.close(resolve));
      address = null;
    }
  } while (!address);
  assert(address && typeof address !== "string", "server address unavailable");
  return { server, baseUrl: `http://127.0.0.1:${address.port}` };
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function snapshot(page) {
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.());
}

async function visibleViewActions(page) {
  return page.evaluate(() => Array.from(document.querySelectorAll("[data-view-action]"))
    .filter((button) => !button.hidden)
    .map((button) => ({ action: button.dataset.viewAction, disabled: button.disabled, reason: button.dataset.disabledReason })));
}

async function waitForPausedPendulum(page) {
  const railToggle = page.getByTestId("project-rail-toggle");
  if (await railToggle.getAttribute("aria-expanded") !== "true") await railToggle.click();
  await page.click("[data-testid='project-card-rep_phys_pendulum_wave1_v1']");
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snap?.projectId === "rep_phys_pendulum_wave1_v1" && !snap.activeRunId && snap.semanticRenderer?.status === "awaiting_committed_frame";
  });
  await page.click("[data-testid='lifecycle-run']");
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snap?.executionLifecycle?.status === "running" && Number(snap?.runs?.[0]?.committed_step_count ?? 0) >= 3;
  }, null, { timeout: 60000 });
  await page.click("[data-testid='lifecycle-run'][data-action='pause']");
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "paused");
  return snapshot(page);
}

async function stageLayout(page) {
  return page.evaluate(() => {
    const stage = document.querySelector("[data-testid='bogae-stage']");
    const primary = document.querySelector("[data-testid='bogae-canvas-wrap']");
    const side = document.querySelector("[data-testid='side-view-panel']");
    const source = document.querySelector("[data-testid='source-panel']");
    const stageStyle = getComputedStyle(stage);
    const primaryRect = primary.getBoundingClientRect();
    const sideRect = side.getBoundingClientRect();
    const sourceRect = source.getBoundingClientRect();
    return {
      columns: stageStyle.gridTemplateColumns,
      primary: { left: primaryRect.left, top: primaryRect.top, right: primaryRect.right, bottom: primaryRect.bottom, width: primaryRect.width },
      side: { left: sideRect.left, top: sideRect.top, right: sideRect.right, bottom: sideRect.bottom, width: sideRect.width },
      source: { left: sourceRect.left, top: sourceRect.top, width: sourceRect.width, height: sourceRect.height },
    };
  });
}

async function clickCommittedTarget(page) {
  await page.locator("[data-testid='bogae-canvas']").scrollIntoViewIfNeeded();
  const point = await page.evaluate(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const canvasPoint = snap?.semanticRenderer?.canvas_projection?.canvas_bob
      ?? snap?.semanticRenderer?.canvas_projection?.canvas_current;
    if (!canvas || !canvasPoint) return null;
    const rect = canvas.getBoundingClientRect();
    return {
      x: rect.left + (canvasPoint.x / canvas.width) * rect.width,
      y: rect.top + (canvasPoint.y / canvas.height) * rect.height,
    };
  });
  assert(point, `committed target has no canvas projection: ${JSON.stringify((await snapshot(page))?.semanticRenderer)}`);
  await page.mouse.click(point.x, point.y);
  await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.viewWorkspace?.selected_target));
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const { server, baseUrl } = await serve(root);
  let browser = null;
  const pageErrors = [];
  try {
    browser = await chromium.launch({ headless: true,
      ...(process.env.DDN_TEST_CHROME ? { executablePath: process.env.DDN_TEST_CHROME } : {}) });
    const page = await browser.newPage({ viewport: { width: 1360, height: 840 }, locale: "ko-KR" });
    page.on("pageerror", (err) => pageErrors.push(err.message));
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()));

    let snap = await snapshot(page);
    let actions = await visibleViewActions(page);
    assert(snap.viewWorkspace.instances.length === 1 && snap.viewWorkspace.instances[0].role === "primary", "pre-run workspace must contain only the primary instance");
    assert(actions.map((row) => row.action).join(",") === "focus-primary", `unavailable pre-run role actions were advertised: ${JSON.stringify(actions)}`);

    snap = await waitForPausedPendulum(page);
    actions = await visibleViewActions(page);
    assert(actions.some((row) => row.action === "add-side" && !row.disabled), "runtime-backed side action missing");
    assert(actions.some((row) => row.action === "toggle-overlay" && !row.disabled), "runtime-backed overlay action missing");
    assert(actions.some((row) => row.action === "open-magnifier" && row.disabled && row.reason.includes("선택")), "magnifier must require a selected target");

    await page.click("[data-testid='view-role-side']");
    await page.click("[data-testid='view-role-side']");
    snap = await snapshot(page);
    const sideInstances = snap.viewWorkspace.instances.filter((instance) => instance.role === "side");
    assert(sideInstances.length === 1, "same side action created duplicate instances");
    assert(new Set(sideInstances.map((instance) => instance.view_instance_id)).size === 1, "same side action changed identity");
    const sideDomCount = await page.locator(".side-view-instance[data-view-instance-role='side']").count();
    assert(sideDomCount === 1, `side instance DOM surface missing: count=${sideDomCount} html=${await page.locator("[data-testid='side-view-list']").innerHTML()}`);
    const sideCanvasHashes = await page.evaluate(() => Array.from(document.querySelectorAll("[data-view-instance-role='side'] canvas"))
      .map((canvas) => canvas.dataset.semanticPayloadHash));
    assert(new Set(sideCanvasHashes).size === 1, "side surface payload missing");

    let layout = await stageLayout(page);
    assert(layout.side.top >= layout.primary.bottom - 1, `DDN-main side views were not placed below the primary: ${JSON.stringify(layout)}`);
    assert((await page.textContent("[data-testid='layout-policy']")).includes("원문 중심"), "source-main layout policy not announced");

    await page.click("[data-mode='bogae-main']");
    layout = await stageLayout(page);
    assert(layout.side.left >= layout.primary.right - 1, `bogae-main side views were not placed beside the primary: ${JSON.stringify(layout)}`);
    assert((await page.textContent("[data-testid='layout-policy']")).includes("보개 중심"), "bogae-main layout policy not announced");

    const primarySignature = await page.locator("[data-testid='bogae-canvas']").getAttribute("data-view-artifact-hash");
    await page.click("[data-testid='view-role-overlay']");
    snap = await snapshot(page);
    const overlays = snap.viewWorkspace.instances.filter((instance) => instance.role === "overlay");
    assert(overlays.length === 1, "overlay did not create an independent instance");
    assert(overlays[0].target_view_instance_id === snap.viewWorkspace.primary_view_instance_id, "overlay did not target the primary coordinate space");
    assert(overlays[0].render_surface === "target_overlay", "overlay became a global role mode");
    assert(snap.semanticRenderer.overlay_enabled === true, "primary renderer did not consume overlay instance state");
    assert((await page.locator("[data-testid='bogae-canvas']").getAttribute("data-view-artifact-hash")) !== primarySignature, "overlay instance did not change the target view artifact");
    await page.click("[data-testid='view-role-overlay']");
    snap = await snapshot(page);
    assert(!snap.viewWorkspace.instances.some((instance) => instance.role === "overlay"), "overlay toggle did not close its instance");

    await clickCommittedTarget(page);
    snap = await snapshot(page);
    assert(snap.viewWorkspace.selected_target.state_hash === snap.activeFrame.state_hash, "selected target is not bound to the active committed frame");
    assert(!(await page.locator("[data-testid='view-role-magnifier']").isDisabled()), "magnifier remained disabled after target selection");
    await page.click("[data-testid='view-role-magnifier']");
    snap = await snapshot(page);
    const magnifier = snap.viewWorkspace.instances.find((instance) => instance.role === "magnifier");
    assert(magnifier?.temporary === true, "magnifier is not a temporary view instance");
    assert(magnifier?.target?.state_hash === snap.activeFrame.state_hash, "magnifier lost its selected committed target");
    assert(await page.locator(".side-view-instance[data-view-instance-role='magnifier']").count() === 1, "magnifier side surface missing");

    const magnifierWorldHash = snap.activeFrame.state_hash;
    const magnifierFrameId = snap.activeFrame.frame_id;
    for (let repeat = 0; repeat < 3; repeat += 1) {
      await clickCommittedTarget(page);
      await page.click("[data-testid='view-role-magnifier']");
    }
    snap = await snapshot(page);
    assert(snap.viewWorkspace.instances.filter((instance) => instance.role === "magnifier").length === 1, "repeated selection/open duplicated the magnifier");
    assert(snap.viewWorkspace.instances.find((instance) => instance.role === "magnifier").view_instance_id === magnifier.view_instance_id, "same target changed magnifier identity");
    assert(snap.activeFrame.state_hash === magnifierWorldHash && snap.activeFrame.frame_id === magnifierFrameId, "view-only interaction advanced world state");
    assert(await page.locator(".side-view-instance[data-view-instance-role='magnifier']").count() === 1, "repeated open duplicated DOM surfaces");

    const previousFrameId = snap.activeFrame.frame_id;
    await page.click("[data-testid='lifecycle-step']");
    await page.waitForFunction(
      (frameId) => Number(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.activeFrame?.frame_id ?? -1) > frameId,
      previousFrameId,
      { timeout: 60000 },
    );
    snap = await snapshot(page);
    assert(snap.viewWorkspace.selected_target === null, "frame change retained a stale selected target");
    assert(!snap.viewWorkspace.instances.some((instance) => instance.role === "magnifier"), "frame change retained a stale temporary magnifier");
    assert(snap.viewWorkspace.instances.filter((instance) => instance.role === "side").length === 1, "frame change closed durable side instance");

    if (process.env.DDN_TEST_SCOPE === "magnifier-lifetime") {
      assert(pageErrors.length === 0, `page errors: ${pageErrors.join(" | ")}`);
      console.log("seamgrim_workbench_view_instances_browser: magnifier-lifetime ok (full persistence workflow NOT_RUN)");
      return;
    }

    const firstSideId = snap.viewWorkspace.instances.find((instance) => instance.role === "side").view_instance_id;
    await page.click(`[data-view-instance-close='${firstSideId}']`);
    snap = await snapshot(page);
    assert(snap.viewWorkspace.instances.filter((instance) => instance.role === "side" && !instance.hidden).length === 0, "individual side close failed");
    await page.click("[data-testid='side-view-close']");
    snap = await snapshot(page);
    assert(!snap.viewWorkspace.instances.some((instance) => ["side", "magnifier"].includes(instance.role) && !instance.hidden), "close-all retained a visible side surface");
    assert(await page.locator("[data-testid='bogae-stage']").getAttribute("data-side-open") === "false", "empty side area remained open");

    await page.click("[data-mode='bogae-full']");
    layout = await stageLayout(page);
    assert(layout.source.left < 0 || layout.source.width <= 1, "desktop bogae-full did not prioritize the view surface");
    assert(await page.locator("[data-testid='bogae-source-access']").isVisible(), "bogae-full lost DDN access");
    await page.click("[data-testid='bogae-source-access']");
    snap = await snapshot(page);
    assert(snap.mode === "ddn-main", "DDN access did not return to source mode");
    assert(await page.locator("[data-testid='source-editor']").isVisible(), "DDN source remained inaccessible");

    await page.setViewportSize({ width: 600, height: 900 });
    await page.click("[data-testid='view-role-side']");
    await page.click("[data-mode='bogae-main']");
    layout = await stageLayout(page);
    assert(layout.side.top >= layout.primary.bottom - 1, `narrow layout did not stack view surfaces: ${JSON.stringify(layout)}`);

    snap = await snapshot(page);
    const durableSideIds = snap.viewWorkspace.instances.filter((instance) => instance.role === "side").map((instance) => instance.view_instance_id);
    await page.click("[data-testid='project-save']");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.persistence?.phase === "saved", null, { timeout: 60000 });
    await page.click("[data-testid='side-view-close']");
    assert((await snapshot(page)).viewWorkspace.instances.filter((instance) => instance.role === "side" && !instance.hidden).length === 0, "pre-reopen side close failed");
    await page.click("[data-testid='project-reopen']");
    await page.waitForFunction(
      (ids) => {
        const instances = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.viewWorkspace?.instances ?? [];
        return ids.every((id) => instances.some((instance) => instance.view_instance_id === id && instance.role === "side"));
      },
      durableSideIds,
      { timeout: 60000 },
    );
    snap = await snapshot(page);
    assert(snap.viewWorkspace.selected_target === null && !snap.viewWorkspace.instances.some((instance) => instance.temporary), "reopen restored volatile target or magnifier state");

    assert(pageErrors.length === 0, `page errors: ${pageErrors.join(" | ")}`);
    console.log("seamgrim_workbench_view_instances_browser: ok");
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
