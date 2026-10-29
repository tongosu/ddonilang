#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_approved_workbench_runtime_lineage: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".toml") || file.endsWith(".ddn") || file.endsWith(".md")) return "text/plain; charset=utf-8";
  return "application/octet-stream";
}

function createServer(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      const rel = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/workbench/index.html" : url.pathname).replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, rel);
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
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
        reject(new Error("failed to bind server"));
        return;
      }
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function snapshot(page) {
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.());
}

async function waitForRuntime(page, predicate, label) {
  await page.waitForFunction(predicate, label, { timeout: 60000 });
  return snapshot(page);
}

async function selectProjectByCard(page, testId, projectId) {
  await page.evaluate((id) => document.querySelector(`[data-testid='${id}']`)?.click(), testId);
  await waitForRuntime(
    page,
    (expected) => document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId === expected,
    projectId,
  );
}

async function pauseAtBoundary(page, label = "paused") {
  let snap = await snapshot(page);
  if (snap?.executionLifecycle?.status === "running") {
    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
  }
  snap = await waitForRuntime(
    page,
    () => {
      const s = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return s?.executionLifecycle?.status === "paused" && !s?.livePump?.in_flight;
    },
    label,
  );
  return snap;
}

async function manualStep(page, label = "manual step") {
  const before = await pauseAtBoundary(page, `${label} pause`);
  const runId = before.activeRunId;
  const frameCount = before.runs.find((run) => run.run_id === runId)?.frame_count ?? 0;
  await page.click("[data-testid='lifecycle-step']");
  return waitForRuntime(
    page,
    ({ activeRunId, beforeFrameCount }) => {
      const s = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return s?.runs?.find((row) => row.run_id === activeRunId)?.frame_count >= beforeFrameCount + 1;
    },
    { activeRunId: runId, beforeFrameCount: frameCount },
  );
}

async function stopActiveRun(page) {
  const snap = await snapshot(page);
  if (!["running", "paused", "pausing"].includes(String(snap?.executionLifecycle?.status ?? ""))) return snap;
  await page.click("[data-testid='lifecycle-stop']");
  return waitForRuntime(
    page,
    () => ["done", "idle"].includes(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status),
    "stopped",
  );
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  for (const rel of [
    "solutions/seamgrim_ui_mvp/ui/workbench/index.html",
    "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js",
    "solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js",
    "solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js",
    "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js",
    "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm",
  ]) {
    const stat = await fs.stat(path.join(root, rel)).catch(() => null);
    assert(stat?.isFile(), `missing file: ${rel}`);
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      locale: "ko-KR",
      reducedMotion: "reduce",
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (["error", "warning"].includes(msg.type())) failures.push(`${msg.type()}: ${msg.text()}`);
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("response", (res) => {
      if (res.status() >= 400) failures.push(`response ${res.status()}: ${res.url()}`);
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await selectProjectByCard(page, "project-card-rep_phys_pendulum_wave1_v1", "rep_phys_pendulum_wave1_v1");
    assert(
      (await page.evaluate(() => typeof window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot)) === "function",
      "workbench debug snapshot unavailable",
    );

    await page.click("[data-testid='lifecycle-run']");
    let snap = await waitForRuntime(
      page,
      () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runtimeStatus === "running",
      "running",
    );
    assert(snap.runs.length === 1, "run should be created");
    assert(snap.runs[0].frame_count >= 1, "initial reset frame missing");
    const runId = snap.activeRunId;
    const initialHash = snap.activeFrame.state_hash;
    assert(initialHash && !initialHash.startsWith("candidate:missing"), `state hash missing: ${initialHash}`);

    snap = await manualStep(page, "pendulum step");
    assert(snap.activeFrame.madi >= 1 || snap.cursorIndex >= 1, `madi did not advance: ${JSON.stringify(snap.activeFrame)}`);

    await page.click("[data-testid='lifecycle-run'][data-action='resume']");
    snap = await waitForRuntime(
      page,
      () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runtimeStatus === "running",
      "resumed",
    );
    snap = await pauseAtBoundary(page, "resumed pause");

    const hashBeforeSetup = snap.activeFrame.state_hash;
    await page.fill("[data-testid='setup-g']", "11.2");
    await page.click("[data-testid='setup-apply']");
    snap = await snapshot(page);
    assert(snap.pendingRecipe?.values?.g === 11.2, `setup recipe not captured: ${JSON.stringify(snap.pendingRecipe)}`);
    assert(snap.activeFrame.state_hash === hashBeforeSetup, "setup control should not mutate current state hash");

    await stopActiveRun(page);
    await selectProjectByCard(page, "project-card-rep_grid_game_state_drop_v1", "rep_grid_game_state_drop_v1");
    await page.click("[data-testid='action-left']");
    snap = await snapshot(page);
    assert(snap.nextInput?.intent === "left", "action did not create next InputSnapshot");
    await page.click("[data-testid='lifecycle-run']");
    snap = await waitForRuntime(
      page,
      () => {
        const s = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
        const run = s?.runs?.find((row) => row.run_id === s.activeRunId);
        return s?.executionLifecycle?.status === "running" && run?.frame_count >= 1 && !s.nextInput;
      },
      "action consumed",
    );
    const leftInputCount = (snap.runs.find((run) => run.run_id === snap.activeRunId)?.input_snapshots ?? [])
      .filter((input) => input?.intent === "left").length;
    assert(leftInputCount === 1, `action InputSnapshot not recorded exactly once: ${JSON.stringify(snap.runs.find((run) => run.run_id === snap.activeRunId)?.input_snapshots ?? [])}`);
    snap = await manualStep(page, "grid step");

    const hashBeforeView = snap.activeFrame.state_hash;
    await page.click("[data-testid='view-role-side']");
    snap = await snapshot(page);
    assert(snap.activeFrame.state_hash === hashBeforeView, "view role switch changed state hash");

    const branchParentRunId = snap.activeRunId;
    await page.click("[data-testid='history-back']");
    snap = await waitForRuntime(
      page,
      () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runtimeStatus === "history-readonly",
      "history readonly",
    );
    assert(snap.sourceReadOnly === true, "history cursor should make source read-only");
    const readonlyIndex = snap.cursorIndex;
    const rejectedHistoryStep = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.stepForward());
    snap = await snapshot(page);
    assert(rejectedHistoryStep === false, "history step API should be rejected");
    assert(snap.cursorIndex === readonlyIndex, "step from history should be rejected");
    assert(snap.diagnostics.some((diag) => diag.code === "E_RUN_HISTORY_READ_ONLY"), "history step rejection diagnostic missing");

    await page.click("[data-testid='history-branch']");
    snap = await waitForRuntime(
      page,
      () => {
        const s = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
        const active = s?.runs?.find((row) => row.run_id === s.activeRunId);
        return s?.runs?.length >= 2
          && s?.activeRunId?.startsWith("branch_")
          && Number(active?.frame_count ?? 0) >= 1
          && s?.executionLifecycle?.status === "running";
      },
      "branch created",
    );
    assert(snap.runs.length === 2, "branch run not created");
    const child = snap.runs.find((row) => row.run_id === snap.activeRunId);
    assert(child.parent_run_id === branchParentRunId, `branch parent mismatch: ${JSON.stringify(child)}`);
    assert(child.parent_state_hash, "branch parent state hash missing");
    assert(snap.sourceReadOnly === false, "branch run should return to editable latest cursor");

    const overwrite = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.attemptOverwriteActiveRun());
    const reuse = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.attemptReuseRunId(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeRunId));
    const parentMutation = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.attemptMutateParentHash());
    snap = await snapshot(page);
    assert(overwrite === false, "overwrite attempt should be rejected");
    assert(reuse === false, "run id reuse should be rejected");
    assert(parentMutation === false, "parent hash mutation should be rejected");
    for (const code of ["E_RUN_IMMUTABLE_OVERWRITE_REJECTED", "E_RUN_ID_REUSE_REJECTED", "E_PARENT_HASH_MUTATION_REJECTED"]) {
      assert(snap.diagnostics.some((diag) => diag.code === code), `${code} diagnostic missing`);
    }

    await page.click("[data-testid='lifecycle-stop']");
    snap = await waitForRuntime(
      page,
      () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runtimeStatus === "stopped",
      "stopped",
    );

    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);
    await context.close();
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
