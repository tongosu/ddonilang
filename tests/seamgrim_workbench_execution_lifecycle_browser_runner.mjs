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
  return page.evaluate(() => {
    const result = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    const button = document.querySelector("[data-testid='lifecycle-run']");
    return {...result, primaryControl: {action: button?.dataset.action, enabled: button?.disabled === false}};
  });
}

async function waitForState(page, state, timeout = 60000) {
  try {
    await page.waitForFunction(
      (expected) => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === expected,
      state,
      { timeout },
    );
  } catch (err) {
    const current = await snapshot(page).catch(() => null);
    throw new Error(`state ${state} timeout: ${JSON.stringify(current)}\n${err?.message ?? err}`);
  }
  return snapshot(page);
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const { server, baseUrl } = await serve(root);
  let browser = null;
  const pageErrors = [];
  try {
    browser = await chromium.launch({ headless: true, executablePath: process.env.CHROME_EXECUTABLE || 'C:/Program Files/Google/Chrome/Application/chrome.exe' });
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    page.on("pageerror", (err) => pageErrors.push(err.message));
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await page.click("[data-testid='project-rail-toggle']");
    await page.click("[data-testid='project-card-rep_phys_pendulum_wave1_v1']");
    await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_pendulum_wave1_v1']");

    await page.evaluate(() => {
      document.querySelector("[data-testid='lifecycle-run']")?.click();
      document.querySelector("[data-testid='project-card-rep_phys_projectile_xy_v1']")?.click();
    });
    await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_projectile_xy_v1']");
    let snap = await waitForState(page, "idle");
    assert(snap.runs.length === 0 && !snap.activeRunId, "preparing project switch retained stale run");

    if (await page.getByTestId('project-rail-toggle').getAttribute('aria-expanded') !== 'true') {
      await page.getByTestId('project-rail-toggle').click();
    }
    await page.click("[data-testid='project-card-rep_phys_pendulum_wave1_v1']");
    await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_pendulum_wave1_v1']");
    const duplicateResults = await page.evaluate(async () => {
      const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
      const first = api.startRun();
      const second = api.startRun();
      return Promise.all([first, second]);
    });
    assert(duplicateResults[0] === true && duplicateResults[1] === false, `duplicate start results: ${JSON.stringify(duplicateResults)}`);
    snap = await waitForState(page, "running");
    assert(snap.runs.length === 1, `duplicate start created ${snap.runs.length} runs`);
    assert(snap.executionLifecycle.mode === "live", `expected live mode: ${JSON.stringify(snap.executionLifecycle)}`);
    assert(snap.primaryControl.action === "pause", "primary must pause, not start another run, while live/running");
    assert(snap.primaryControl.enabled, "pause must be enabled while live/running");
    assert(snap.controls.lifecycle_step.enabled === false, "manual step must be disabled while live/running");

    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
    snap = await waitForState(page, "paused");
    assert(snap.primaryControl.action === "resume" && snap.primaryControl.enabled && snap.controls.lifecycle_step.enabled, "paused controls mismatch");
    const beforeStep = snap.runs[0].frame_count;
    await page.click("[data-testid='lifecycle-step']");
    await page.waitForFunction(
      (count) => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runs?.[0]?.frame_count === count + 1,
      beforeStep,
      { timeout: 60000 },
    );
    snap = await waitForState(page, "paused");
    assert(snap.executionLifecycle.run_id === snap.activeRunId, "manual step changed active run");

    await page.click("[data-testid='lifecycle-run'][data-action='resume']");
    snap = await waitForState(page, "running");
    const parentRunId = snap.activeRunId;
    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
    await waitForState(page, "paused");
    await page.click("[data-testid='history-back']");
    snap = await waitForState(page, "history_readonly");
    assert(snap.controls.history_branch.enabled, "branch must be enabled at a past cursor");
    await page.click("[data-testid='history-branch']");
    snap = await waitForState(page, "running");
    assert(snap.runs.length === 2, "branch did not create exactly one child run");
    const child = snap.runs.find((run) => run.run_id === snap.activeRunId);
    assert(child?.parent_run_id === parentRunId, `branch parent mismatch: ${JSON.stringify(child)}`);

    await page.click("[data-testid='lifecycle-stop']");
    snap = await waitForState(page, "done");
    assert(snap.runtimeStatus === "stopped", `legacy stopped alias mismatch: ${snap.runtimeStatus}`);
    assert(snap.primaryControl.action === "start" && snap.primaryControl.enabled && !snap.controls.lifecycle_stop.enabled, "done controls mismatch");

    if (await page.getByTestId('project-rail-toggle').getAttribute('aria-expanded') !== 'true') {
      await page.getByTestId('project-rail-toggle').click();
    }
    await page.click("[data-testid='new-ddn-project']");
    await page.waitForSelector("[data-testid='project-create-dialog'][open]");
    await page.fill("[data-testid='project-create-name']", "상태기계 한 번 실행");
    await page.check("input[name='project-template'][value='single']");
    await page.click("[data-testid='project-create-submit']");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.project?.native_creator === true);
    await page.fill("[data-testid='source-editor']", "1 보여주기.");
    await page.click("[data-testid='lifecycle-run']");
    snap = await waitForState(page, "done");
    assert(snap.executionLifecycle.mode === "oneshot", `one-shot mode mismatch: ${JSON.stringify(snap.executionLifecycle)}`);
    assert(snap.runs.length === 1 && snap.runs[0].frame_count === 1, "one-shot must commit exactly one immediate frame");
    assert(snap.primaryControl.action === "start", "completed one-shot must offer a new run, never pause or resume");
    assert(!snap.controls.lifecycle_step.enabled && !snap.controls.lifecycle_stop.enabled, "one-shot step/stop must stay disabled");
    assert(pageErrors.length === 0, `page errors: ${pageErrors.join(" | ")}`);
    console.log("seamgrim_workbench_execution_lifecycle_browser: ok");
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
