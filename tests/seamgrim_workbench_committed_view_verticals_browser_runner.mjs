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
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.());
}

async function canvasSnapshot(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    return {
      signature: canvas?.dataset?.canvasSignature ?? "",
      family: canvas?.dataset?.rendererFamily ?? "",
      status: canvas?.dataset?.rendererStatus ?? "",
      source: canvas?.dataset?.rendererDataSource ?? "",
      trajectoryLength: Number(canvas?.dataset?.rendererTrajectoryLength ?? 0),
      theta: canvas?.dataset?.rendererCurrentTheta ?? "",
      x: canvas?.dataset?.rendererCurrentX ?? "",
      y: canvas?.dataset?.rendererCurrentY ?? "",
      tick: canvas?.dataset?.rendererTick ?? "",
      stateHash: canvas?.dataset?.committedStateHash ?? "",
    };
  });
}

async function canonicalWorkbenchSnapshot(page, family) {
  return page.evaluate((familyName) => {
    const payload = window.__SEAMGRIM_WORKBENCH_CANONICAL_VIEW_PAYLOADS__?.[`workbench_${familyName}`] ?? null;
    return payload ? JSON.parse(JSON.stringify(payload)) : null;
  }, family);
}

function channelValue(frame, name) {
  const channels = Array.isArray(frame?.wasm_state?.channels) ? frame.wasm_state.channels : [];
  const row = Array.isArray(frame?.wasm_state?.row) ? frame.wasm_state.row : [];
  const index = channels.findIndex((channel) => String(channel?.key ?? "") === name);
  return index >= 0 ? row[index] : undefined;
}

async function selectProject(page, projectId) {
  await page.click(`[data-testid='project-card-${projectId}']`);
  await page.waitForFunction(
    (id) => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.projectId === id
        && !snap?.activeRunId
        && snap?.semanticRenderer?.status === "awaiting_committed_frame";
    },
    projectId,
  );
}

async function runToPaused(page, projectId, minimumSteps = 3) {
  await selectProject(page, projectId);
  const before = await snapshot(page);
  assert(before.semanticRenderer?.status === "awaiting_committed_frame", `${projectId}: pre-run renderer fabricated a committed view`);
  await page.click("[data-testid='lifecycle-run']");
  await page.waitForFunction(
    (count) => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const active = snap?.runs?.find((run) => run.run_id === snap.activeRunId);
      return snap?.executionLifecycle?.status === "running" && Number(active?.committed_step_count ?? 0) >= count;
    },
    minimumSteps,
    { timeout: 60000 },
  );
  await page.click("[data-testid='lifecycle-run'][data-action='pause']");
  await page.waitForFunction(
    () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "paused",
    null,
    { timeout: 60000 },
  );
  return snapshot(page);
}

async function stepOnce(page, previousFrameId) {
  await page.click("[data-testid='lifecycle-step']");
  await page.waitForFunction(
    (frameId) => Number(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.activeFrame?.frame_id ?? -1) > frameId,
    previousFrameId,
    { timeout: 60000 },
  );
  return snapshot(page);
}

async function stopPausedRun(page) {
  await page.click("[data-testid='lifecycle-stop']");
  await page.waitForFunction(
    () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "done",
    null,
    { timeout: 60000 },
  );
}

async function createSingleProject(page, name) {
  const previous = await snapshot(page);
  await page.click("[data-testid='new-ddn-project']");
  await page.waitForSelector("[data-testid='project-create-dialog'][open]");
  await page.fill("[data-testid='project-create-name']", name);
  await page.check("input[name='project-template'][value='single']");
  await page.click("[data-testid='project-create-submit']");
  await page.waitForFunction(
    (previousId) => {
      const current = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return current?.projectId !== previousId
        && current?.project?.native_creator === true
        && !current?.activeRunId
        && current?.semanticRenderer?.status === "awaiting_committed_frame";
    },
    previous?.projectId ?? "",
  );
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const { server, baseUrl } = await serve(root);
  let browser = null;
  const pageErrors = [];
  try {
    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    page.on("pageerror", (err) => pageErrors.push(err.message));
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()));

    let snap = await runToPaused(page, "rep_phys_projectile_xy_v1");
    let renderer = snap.semanticRenderer;
    let canvas = await canvasSnapshot(page);
    assert(renderer.family === "projectile" && renderer.status === "ready", "projectile committed renderer not ready");
    assert(renderer.data_source === "canonical_workbench_result", "projectile renderer did not use canonical runtime result");
    assert(renderer.current.x === channelValue(snap.activeFrame, "x"), "projectile current x differs from committed channel");
    assert(renderer.current.y === channelValue(snap.activeFrame, "y"), "projectile current y differs from committed channel");
    assert(renderer.trajectory_length >= 3, "projectile did not connect committed history");
    assert(renderer.trajectory_length <= snap.activeFrame.frame_id + 1, "projectile trajectory crossed the active cursor");
    assert(canvas.family === "projectile" && canvas.source === "canonical_workbench_result", "projectile canvas source metadata mismatch");
    assert((await canonicalWorkbenchSnapshot(page, "projectile"))?.status === "available", "projectile canonical result unavailable");
    assert(Number(canvas.x) === renderer.current.x && Number(canvas.y) === renderer.current.y, "projectile canvas did not expose committed position");
    const projectileSignature = canvas.signature;
    const projectileX = renderer.current.x;
    snap = await stepOnce(page, snap.activeFrame.frame_id);
    canvas = await canvasSnapshot(page);
    assert(snap.semanticRenderer.current.x !== projectileX, "projectile point did not move after a committed step");
    assert(canvas.signature !== projectileSignature, "projectile canvas artifact did not change after a committed step");
    const projectileCanvasCurrent = snap.semanticRenderer.canvas_projection.canvas_current;
    await page.click("[data-testid='view-role-overlay']");
    snap = await snapshot(page);
    assert(snap.semanticRenderer.overlay_enabled === true, "projectile overlay instance was not passed through semantic config");
    assert(JSON.stringify(snap.semanticRenderer.canvas_projection.canvas_current) === JSON.stringify(projectileCanvasCurrent), "projectile overlay left the primary coordinate system");
    await page.click("[data-testid='view-role-overlay']");
    await stopPausedRun(page);

    snap = await runToPaused(page, "rep_phys_pendulum_wave1_v1");
    renderer = snap.semanticRenderer;
    canvas = await canvasSnapshot(page);
    assert(renderer.family === "pendulum" && renderer.status === "ready", "pendulum committed renderer not ready");
    assert(renderer.data_source === "canonical_workbench_result", "pendulum renderer did not use canonical runtime result");
    assert((await canonicalWorkbenchSnapshot(page, "pendulum"))?.status === "available", "pendulum canonical result unavailable");
    assert(renderer.current.theta === channelValue(snap.activeFrame, "theta"), "pendulum theta differs from committed channel");
    assert(renderer.current.omega === channelValue(snap.activeFrame, "omega"), "pendulum omega differs from committed channel");
    assert(renderer.current.energy === channelValue(snap.activeFrame, "energy"), "pendulum energy differs from committed channel");
    assert(Math.abs(renderer.current.physical_bob.x - channelValue(snap.activeFrame, "bob_x")) < 1e-8, "pendulum bob x was not derived from committed theta/L");
    assert(Math.abs(renderer.current.physical_bob.y - channelValue(snap.activeFrame, "bob_y")) < 1e-8, "pendulum bob y was not derived from committed theta/L");
    assert(Number(canvas.theta) === renderer.current.theta, "pendulum canvas theta metadata mismatch");
    const primaryBob = renderer.canvas_projection.canvas_bob;
    await page.click("[data-testid='view-role-overlay']");
    snap = await snapshot(page);
    assert(JSON.stringify(snap.semanticRenderer.canvas_projection.canvas_bob) === JSON.stringify(primaryBob), "pendulum overlay left the primary coordinate system");
    await page.click("[data-testid='view-role-overlay']");
    const pendulumSignature = (await canvasSnapshot(page)).signature;
    const pendulumTheta = snap.semanticRenderer.current.theta;
    snap = await stepOnce(page, snap.activeFrame.frame_id);
    assert(snap.semanticRenderer.current.theta !== pendulumTheta, "pendulum did not move after a committed step");
    assert((await canvasSnapshot(page)).signature !== pendulumSignature, "pendulum canvas artifact did not change after a committed step");
    await stopPausedRun(page);

    snap = await runToPaused(page, "rep_grid_game_state_drop_v1");
    renderer = snap.semanticRenderer;
    canvas = await canvasSnapshot(page);
    assert(renderer.family === "grid" && renderer.status === "ready", "grid committed renderer not ready");
    assert(renderer.geometry_source === "canonical_workbench_grid_result", "grid used non-canonical geometry");
    assert(renderer.data_source === "canonical_workbench_result", "grid renderer did not use canonical runtime result");
    assert((await canonicalWorkbenchSnapshot(page, "grid"))?.status === "available", "grid canonical result unavailable");
    assert(renderer.draw_item_count >= 64, `grid committed drawlist missing: ${renderer.draw_item_count}`);
    assert(renderer.position?.x === 3 && renderer.position?.y === 0, `grid committed piece position mismatch: ${JSON.stringify(renderer.position)}`);
    assert(renderer.tick === snap.activeFrame.madi, "grid tick did not use committed frame fallback");
    assert(Number(canvas.tick) === renderer.tick, "grid canvas committed tick metadata mismatch");
    await stopPausedRun(page);

    await createSingleProject(page, "정적 결과 그래프");
    const oneShotSource = `설정 {
  제목: "정적 결과 그래프".
}.

채비 {
  a: 수 <- 2.
  b: 수 <- 1.
  x: 수 <- 2.
  y: 수 <- 5.
}.

x 보여주기.
y 보여주기.`;
    await page.fill("[data-testid='source-editor']", oneShotSource);
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(
      () => ["done", "failed"].includes(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status),
      null,
      { timeout: 60000 },
    );
    snap = await snapshot(page);
    assert(snap.executionLifecycle.status === "done", `one-shot static graph failed: ${JSON.stringify({ lifecycle: snap.executionLifecycle, diagnostics: snap.diagnostics })}`);
    renderer = snap.semanticRenderer;
    canvas = await canvasSnapshot(page);
    assert(snap.executionLifecycle.mode === "oneshot", "static graph source was not one-shot");
    assert(renderer.family === "line" && renderer.status === "ready", "one-shot static graph not ready");
    assert(renderer.data_source === "canonical_workbench_result", "line renderer did not use canonical runtime result");
    assert((await canonicalWorkbenchSnapshot(page, "line"))?.status === "available", "line canonical result unavailable");
    assert(renderer.current.x === 2 && renderer.current.y === 5, `one-shot static graph point mismatch: ${JSON.stringify(renderer.current)}`);
    assert(renderer.labels.map((row) => row.key).join(",") === "x,y,a,b", "one-shot static graph lacks semantic labels");
    assert(canvas.family === "line" && canvas.status === "ready", "one-shot static graph was not drawn");
    assert(pageErrors.length === 0, `page errors: ${pageErrors.join(" | ")}`);

    console.log("seamgrim_workbench_committed_view_verticals_browser: ok");
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
