#!/usr/bin/env node

import fs from "node:fs/promises";
import crypto from "node:crypto";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const SERVER = path.resolve(ROOT, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");
const OUT_DIR = path.resolve(ROOT, "out/seamgrim_workbench_g14");
const RUN_OWNER_ID = `sgg14_${process.pid}_${Date.now()}_${crypto.randomBytes(4).toString("hex")}`;

const evidence = {
  schema: "seamgrim.workbench_race_matrix.runner_result.v1",
  run_owner_id: RUN_OWNER_ID,
  server: {},
  api_faults: {
    delayed_failed_checkpoint_posts: 0,
  },
  assertions: {},
  browser_issues: [],
  cases: [],
};

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function recordCase(id, details = {}) {
  evidence.cases.push({ id, ok: true, ...details });
}

function tailBuffer() {
  const chunks = [];
  return {
    push(data) {
      chunks.push(String(data || ""));
      while (chunks.join("").length > 8000) chunks.shift();
    },
    text() {
      return chunks.join("").slice(-8000);
    },
  };
}

async function waitForReadyRecord(child, stdoutBuffer, timeoutMs = 15_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`server exited before ready: ${child.exitCode}`);
    const line = stdoutBuffer.text().split(/\r?\n/u).find((row) => row.startsWith("SEAMGRIM_SERVER_READY "));
    if (line) return JSON.parse(line.slice("SEAMGRIM_SERVER_READY ".length));
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error("server ready timeout");
}

async function waitForHealth(baseUrl, child) {
  for (let i = 0; i < 90; i += 1) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    try {
      const response = await fetch(`${baseUrl}/api/v1/projects/health`);
      if (response.ok) return;
    } catch (_) {
      // Retry below.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("server health timeout");
}

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-workbench-g14-"));
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", "0"], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir, SEAMGRIM_RUN_OWNER_ID: RUN_OWNER_ID },
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  const stdout = tailBuffer();
  const stderr = tailBuffer();
  child.stdout.on("data", (data) => stdout.push(data));
  child.stderr.on("data", (data) => stderr.push(data));
  const ready = await waitForReadyRecord(child, stdout);
  const baseUrl = `http://${ready.host}:${ready.port}`;
  await waitForHealth(baseUrl, child);
  evidence.server = {
    schema: ready.schema,
    host: ready.host,
    port: ready.port,
    data_dir_redacted: `${path.basename(dataDir)}:${crypto.createHash("sha256").update(dataDir).digest("hex").slice(0, 12)}`,
  };
  return { child, baseUrl, dataDir, stdout, stderr };
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => {
    const timer = setTimeout(resolve, 3000);
    child.once("exit", () => {
      clearTimeout(timer);
      resolve();
    });
  });
  if (child.exitCode === null) {
    try {
      process.kill(child.pid, "SIGKILL");
    } catch (_) {
      // Already stopped.
    }
  }
}

function installPageCapture(page, label) {
  page.on("console", (msg) => {
    if (msg.type() === "error") {
      const text = String(msg.text() || "").slice(0, 500);
      if (label === "checkpoint_failure" && text.includes("503")) return;
      evidence.browser_issues.push({ label, type: "console_error", text });
    }
  });
  page.on("pageerror", (err) => {
    evidence.browser_issues.push({ label, type: "pageerror", text: String(err.message || "").slice(0, 500) });
  });
}

async function waitForWorkbenchApi(page, label = "workbench") {
  await page.waitForFunction(() => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    if (typeof api?.getSnapshot !== "function") return false;
    const snap = api.getSnapshot();
    return Array.isArray(snap?.runs) && snap?.readiness?.workbench_api_ready === true;
  }, null, { timeout: 30_000 }).catch(async (err) => {
    const state = await page.evaluate(() => ({
      href: location.href.replace(/sg_cap=[^&]+/g, "sg_cap=<redacted>"),
      root: Boolean(document.querySelector("[data-testid='approved-workbench-root']")),
      api: typeof window.__SEAMGRIM_APPROVED_WORKBENCH__,
    }));
    throw new Error(`${label}: workbench API not ready: ${err.message}: ${JSON.stringify(state)}`);
  });
}

async function snapshot(page) {
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.());
}

async function waitForStatus(page, status, label = status, timeout = 60_000) {
  await page.waitForFunction(
    (expected) => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === expected,
    status,
    { timeout },
  ).catch(async (err) => {
    throw new Error(`${label}: status timeout: ${err.message}: ${JSON.stringify(await snapshot(page).catch(() => null))}`);
  });
  return snapshot(page);
}

async function waitForRunCommitAtLeast(page, count, label = "commit") {
  await page.waitForFunction((expected) => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    const run = snap?.runs?.find((row) => row.run_id === snap.activeRunId);
    return snap?.executionLifecycle?.status === "running"
      && Number(run?.committed_step_count ?? -1) >= expected;
  }, count, { timeout: 60_000 }).catch(async (err) => {
    throw new Error(`${label}: committed count timeout: ${err.message}: ${JSON.stringify(await snapshot(page).catch(() => null))}`);
  });
  return snapshot(page);
}

async function selectPendulum(page) {
  await page.click("[data-testid='project-card-rep_phys_pendulum_wave1_v1']");
  await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_pendulum_wave1_v1']");
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.projectControls?.status === "ready", null, { timeout: 30_000 });
}

async function selectProjectile(page) {
  await page.click("[data-testid='project-card-rep_phys_projectile_xy_v1']");
  await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_projectile_xy_v1']");
}

async function createSingleProject(page, name) {
  const before = await snapshot(page);
  await page.click("[data-testid='new-ddn-project']");
  await page.waitForSelector("[data-testid='project-create-dialog'][open]");
  await page.fill("[data-testid='project-create-name']", name);
  await page.check("input[name='project-template'][value='single']");
  await page.click("[data-testid='project-create-submit']");
  await page.waitForFunction(
    (previousId) => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.projectId !== previousId && snap?.project?.native_creator === true;
    },
    before?.projectId ?? "",
    { timeout: 30_000 },
  );
}

async function saveProject(page, label = "save") {
  const before = await snapshot(page);
  await page.click("[data-testid='project-save']");
  await page.waitForFunction((previousRevision) => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snap?.persistence?.phase === "saved"
      && Boolean(snap?.serverProject?.project_id)
      && Boolean(snap?.serverProject?.head_revision_id)
      && snap.serverProject.head_revision_id !== previousRevision;
  }, before?.serverProject?.head_revision_id ?? "", { timeout: 60_000 }).catch(async (err) => {
    throw new Error(`${label}: project save timeout: ${err.message}: ${JSON.stringify(await snapshot(page).catch(() => null))}`);
  });
  return snapshot(page);
}

function assertRunFrameInvariants(run, label) {
  assert(run, `${label}: run missing`);
  assert(Number(run.frame_count) >= 1, `${label}: run has no committed frame`);
  assert(Number(run.committed_step_count) >= 0, `${label}: negative committed step count`);
  assert(Number(run.server_checkpoint_count ?? 0) >= 0, `${label}: negative server checkpoint count`);
  const frames = Array.isArray(run.frames) ? run.frames : [];
  assert(frames.length === Number(run.frame_count), `${label}: frame_count mismatch`);
  const ids = frames.map((frame) => Number(frame.frame_id));
  assert(new Set(ids).size === ids.length, `${label}: duplicate frame ids: ${ids.join(",")}`);
  assert(ids.every((id, index) => id === index), `${label}: non-contiguous frame ids: ${ids.join(",")}`);
  assert(frames.every((frame, index) => index === 0 || Number(frame.madi) >= Number(frames[index - 1].madi)), `${label}: madi regressed`);
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
  assert(point, "committed target has no canvas projection");
  await page.mouse.click(point.x, point.y);
  await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.viewWorkspace?.selected_target), null, { timeout: 30_000 });
}

async function runPrimaryRaceMatrix(page) {
  await selectPendulum(page);
  await saveProject(page, "primary initial save");

  const doubleStart = await page.evaluate(async () => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    return Promise.all([api.startRun(), api.startRun()]);
  });
  assert(doubleStart[0] === true && doubleStart[1] === false, `double start did not reject the second request: ${JSON.stringify(doubleStart)}`);
  let snap = await waitForRunCommitAtLeast(page, 2, "double-start live advance");
  assert(snap.runs.length === 1, `double start created ${snap.runs.length} runs`);
  assert(snap.controls.lifecycle_run.enabled === false, "run button remained enabled while live");
  assert(snap.livePump.max_in_flight === 1, `double start exceeded one step in flight: ${JSON.stringify(snap.livePump)}`);
  assertRunFrameInvariants(snap.runs[0], "double_start");
  recordCase("run_double_click_one_loop_one_run", {
    results: doubleStart,
    run_id: snap.activeRunId,
    committed_step_count: snap.runs[0].committed_step_count,
  });

  const sealedSourceHash = snap.runs[0].source_hash;
  const sealedRunId = snap.activeRunId;
  await page.fill("[data-testid='source-editor']", '"live 중 편집 후보" 보여주기.');
  await page.waitForTimeout(160);
  snap = await snapshot(page);
  assert(snap.activeRunId === sealedRunId, "source edit changed active run id");
  assert(snap.runs[0].source_hash === sealedSourceHash, "source edit mutated sealed active run source hash");
  assert(["single_file_source", "virtual_project_source"].includes(snap.runs[0].source_kind), `unexpected source kind: ${snap.runs[0].source_kind}`);
  assertRunFrameInvariants(snap.runs[0], "source_edit_live");
  recordCase("source_edit_during_live_preserves_sealed_run", {
    run_id: sealedRunId,
    source_hash: sealedSourceHash,
    source_kind: snap.runs[0].source_kind,
  });

  const pauseSpam = await page.evaluate(() => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    return [api.pauseRun(), api.pauseRun(), api.pauseRun(), api.pauseRun()];
  });
  assert(pauseSpam.filter(Boolean).length === 1, `pause spam accepted more than once: ${JSON.stringify(pauseSpam)}`);
  snap = await waitForStatus(page, "paused", "pause spam boundary");
  const pausedCount = snap.runs[0].committed_step_count;
  const pausedFrameCount = snap.runs[0].frame_count;
  await page.waitForTimeout(220);
  snap = await snapshot(page);
  assert(snap.runs[0].committed_step_count === pausedCount, "paused run advanced after boundary");
  assert(snap.runs[0].frame_count === pausedFrameCount, "paused run appended frames after boundary");
  assert(!snap.livePump.in_flight && !snap.livePump.scheduled, `paused pump retained work: ${JSON.stringify(snap.livePump)}`);

  const viewHash = snap.activeFrame.state_hash;
  await page.click("[data-testid='view-role-side']");
  await page.click("[data-testid='view-role-overlay']");
  let afterView = await snapshot(page);
  assert(afterView.activeFrame.state_hash === viewHash, "side/overlay visibility changed runtime state hash");
  await clickCommittedTarget(page);
  await page.click("[data-testid='view-role-magnifier']");
  afterView = await snapshot(page);
  assert(afterView.activeFrame.state_hash === viewHash, "magnifier visibility changed runtime state hash");
  assert(afterView.viewWorkspace.instances.some((instance) => instance.role === "overlay"), "overlay instance missing");
  assert(afterView.viewWorkspace.instances.some((instance) => instance.role === "magnifier"), "magnifier instance missing");
  recordCase("overlay_magnifier_visibility_state_hash_invariant", {
    state_hash: viewHash,
    roles: afterView.viewWorkspace.instances.map((instance) => instance.role),
  });

  const exported = await page.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
  const beforeTamper = await snapshot(page);
  const tamperedText = JSON.stringify((() => {
    const doc = JSON.parse(exported.file.json_text);
    doc.project_snapshot.controls_snapshot = {
      ...(doc.project_snapshot.controls_snapshot ?? {}),
      selected_view: "magnifier",
      view_role: "magnifier",
    };
    return doc;
  })());
  const tamperPreview = await page.evaluate(async (text) => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    const file = new File([text], "legacy-selected-view-tamper.seamgrim-project.json", { type: "application/json" });
    await api.previewPortableProjectFile(file);
    return api.getSnapshot();
  }, tamperedText);
  assert(
    String(tamperPreview.pendingPortableImport?.error_code ?? "").startsWith("E_PORTABLE_"),
    `legacy selected_view tamper was not rejected: ${JSON.stringify(tamperPreview.pendingPortableImport)}`,
  );
  assert(tamperPreview.activeRunId === beforeTamper.activeRunId, "tamper preview changed active run id");
  assert(tamperPreview.activeFrame.state_hash === beforeTamper.activeFrame.state_hash, "tamper preview changed active world state");
  await page.evaluate(() => document.querySelector("[data-testid='portable-project-cancel']")?.click());
  snap = await snapshot(page);
  assert(snap.pendingPortableImport === null, "portable import cancel left pending import state");
  assert(snap.activeRunId === beforeTamper.activeRunId, "portable import cancel changed active run id");
  assert(snap.activeFrame.state_hash === beforeTamper.activeFrame.state_hash, "portable import cancel changed active state");
  recordCase("legacy_selected_view_tamper_and_import_cancel_world_invariant", {
    diagnostic: tamperPreview.pendingPortableImport.error_code,
    state_hash: beforeTamper.activeFrame.state_hash,
  });

  const resumeSpam = await page.evaluate(() => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    return [api.resumeRun(), api.resumeRun(), api.resumeRun(), api.resumeRun()];
  });
  assert(resumeSpam.filter(Boolean).length === 1, `resume spam accepted more than once: ${JSON.stringify(resumeSpam)}`);
  snap = await page.waitForFunction((count) => {
    const current = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    const run = current?.runs?.find((row) => row.run_id === current.activeRunId);
    return current?.executionLifecycle?.status === "running" && Number(run?.committed_step_count ?? 0) > count
      ? current
      : false;
  }, pausedCount, { timeout: 60_000 }).then((handle) => handle.jsonValue());
  assert(snap.activeRunId === sealedRunId, "resume spam created a new active run");
  assert(snap.livePump.max_in_flight === 1, `resume spam exceeded one step in flight: ${JSON.stringify(snap.livePump)}`);
  assertRunFrameInvariants(snap.runs[0], "pause_resume_spam");
  recordCase("pause_resume_spam_no_duplicate_steps", {
    pause_results: pauseSpam,
    resume_results: resumeSpam,
    committed_before_resume: pausedCount,
    committed_after_resume: snap.runs[0].committed_step_count,
  });

  const stopSpam = await page.evaluate(async () => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    return Promise.all([api.stopRun(), api.stopRun(), api.stopRun()]);
  });
  assert(stopSpam.filter(Boolean).length === 1, `stop spam accepted more than once: ${JSON.stringify(stopSpam)}`);
  snap = await waitForStatus(page, "done", "stop spam");
  const stopFrameCount = snap.runs[0].frame_count;
  await page.waitForTimeout(260);
  const afterStop = await snapshot(page);
  assert(afterStop.runs[0].frame_count === stopFrameCount, "stale callback appended a frame after stop");
  assert(afterStop.runs[0].server_finalize_attempt_count === 1, `stop/finalize attempt count mismatch: ${afterStop.runs[0].server_finalize_attempt_count}`);
  assert(afterStop.runs[0].server_finalized === true, "stopped run did not finalize");
  assert(!afterStop.livePump.in_flight && !afterStop.livePump.scheduled, "stopped live pump retained work");
  recordCase("stop_stale_callback_and_finalize_retry_guard", {
    stop_results: stopSpam,
    final_status: afterStop.runs[0].server_final_status,
    finalize_attempts: afterStop.runs[0].server_finalize_attempt_count,
  });

  await page.fill("[data-testid='source-editor']", `설정 { 제목: "프로젝트 전환 race". 마디수: 80. }.\n\n(매마디)마다 {\n  1 보여주기.\n}.\n`);
  await saveProject(page, "before project switch race");
  await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.startRun());
  snap = await waitForRunCommitAtLeast(page, 1, "project-switch live source");
  const oldRunId = snap.activeRunId;
  await selectProjectile(page);
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snap?.projectId === "rep_phys_projectile_xy_v1"
      && snap.executionLifecycle.status === "idle"
      && snap.runs.length === 0
      && !snap.activeRunId
      && !snap.livePump.in_flight
      && !snap.livePump.scheduled;
  }, null, { timeout: 60_000 });
  snap = await snapshot(page);
  assert(snap.projectId === "rep_phys_projectile_xy_v1", "project switch did not select projectile");
  assert(!JSON.stringify(snap).includes(oldRunId), "old live run leaked into switched project snapshot");
  recordCase("project_switch_during_live_drops_old_callbacks", {
    old_run_id: oldRunId,
    new_project_id: snap.projectId,
  });

  await createSingleProject(page, "G14 문자열 tick 오판 방지");
  await page.fill("[data-testid='source-editor']", `문장 <- "(매마디)마다 { 값 <- 값 + 1. }".\n// (매마디)마다 { 주석 속 tick은 live가 아님 }.\n문장 보여주기.\n`);
  await page.click("[data-testid='lifecycle-run']");
  snap = await waitForStatus(page, "done", "string/comment tick one-shot");
  assert(snap.runs.length === 1, "string/comment tick fixture created unexpected runs");
  assert(snap.runs[0].execution_mode === "oneshot", `string/comment tick was misclassified as ${snap.runs[0].execution_mode}`);
  assert(snap.runs[0].committed_step_count === 1, "string/comment tick one-shot did not commit exactly once");
  assert(!snap.controls.lifecycle_pause.enabled && !snap.controls.lifecycle_resume.enabled, "string/comment tick exposed live controls");
  recordCase("string_comment_tick_surface_not_live", {
    execution_mode: snap.runs[0].execution_mode,
    committed_step_count: snap.runs[0].committed_step_count,
  });

  await page.fill("[data-testid='source-editor']", `설정 { 제목: "reopen live explicit transition". 마디수: 30. }.\n\n(매마디)마다 {\n  3 보여주기.\n}.\n`);
  await saveProject(page, "before reopen race");
  await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.startRun());
  snap = await waitForRunCommitAtLeast(page, 1, "reopen live source");
  const reopenOldRunId = snap.activeRunId;
  await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.reopenProjectSnapshot());
  await page.waitForFunction((oldRun) => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snap?.executionLifecycle?.status === "idle"
      && snap.runs.length === 0
      && !snap.activeRunId
      && !snap.livePump.in_flight
      && !snap.livePump.scheduled
      && !JSON.stringify(snap).includes(oldRun);
  }, reopenOldRunId, { timeout: 60_000 });
  snap = await snapshot(page);
  recordCase("reopen_during_live_explicitly_clears_run_and_pump", {
    old_run_id: reopenOldRunId,
    project_id: snap.projectId,
  });
}

async function runCheckpointFailureRace(context, baseUrl) {
  const page = await context.newPage();
  installPageCapture(page, "checkpoint_failure");
  await page.route("**/api/v1/projects/*/runs/*/checkpoints", async (route) => {
    evidence.api_faults.delayed_failed_checkpoint_posts += 1;
    await new Promise((resolve) => setTimeout(resolve, 220));
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({
        code: "E_G14_INJECTED_CHECKPOINT_FAILURE",
        user_message: "G14 injected checkpoint delay/failure",
      }),
    });
  });
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "domcontentloaded" });
  await waitForWorkbenchApi(page, "checkpoint page");
  await createSingleProject(page, "G14 checkpoint failure");
  await page.fill("[data-testid='source-editor']", `설정 { 제목: "checkpoint failure". 마디수: 20. }.\n\n(매마디)마다 {\n  7 보여주기.\n}.\n`);
  await saveProject(page, "checkpoint failure save");
  await page.click("[data-testid='lifecycle-run']");
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    const run = snap?.runs?.find((row) => row.run_id === snap.activeRunId);
    return snap?.executionLifecycle?.status === "running"
      && Number(run?.committed_step_count ?? 0) >= 2
      && run?.server_archive_status === "checkpoint-error"
      && snap?.diagnostics?.some((diag) => diag.code === "E_RUN_ARCHIVE_CHECKPOINT_FAILED");
  }, null, { timeout: 60_000 });
  const snap = await snapshot(page);
  const run = snap.runs[0];
  assertRunFrameInvariants(run, "checkpoint_failure");
  assert(run.committed_step_count <= snap.livePump.step_commit_count, "checkpoint failure duplicated committed world steps");
  assert(snap.livePump.max_in_flight === 1, `checkpoint failure exceeded one in-flight step: ${JSON.stringify(snap.livePump)}`);
  assert(evidence.api_faults.delayed_failed_checkpoint_posts >= 2, "checkpoint failure route did not observe multiple checkpoint attempts");
  await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.stopRun());
  await waitForStatus(page, "done", "checkpoint failure stop");
  recordCase("server_checkpoint_delay_failure_no_world_step_duplication", {
    delayed_failed_checkpoint_posts: evidence.api_faults.delayed_failed_checkpoint_posts,
    committed_step_count: run.committed_step_count,
    frame_count: run.frame_count,
    server_archive_status: run.server_archive_status,
  });
  await page.close();
}

async function main() {
  await fs.mkdir(OUT_DIR, { recursive: true });
  const { child, baseUrl, dataDir } = await startServer();
  let browser = null;
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1360, height: 840 }, locale: "ko-KR" });
    const page = await context.newPage();
    installPageCapture(page, "primary");
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "domcontentloaded" });
    await waitForWorkbenchApi(page, "primary page");
    await runPrimaryRaceMatrix(page);
    await runCheckpointFailureRace(context, baseUrl);
    await context.close();
  } finally {
    if (browser) await browser.close();
    await stopServer(child);
    await fs.rm(dataDir, { recursive: true, force: true });
  }

  assert(evidence.browser_issues.length === 0, `browser issues: ${JSON.stringify(evidence.browser_issues)}`);
  const requiredCases = [
    "run_double_click_one_loop_one_run",
    "source_edit_during_live_preserves_sealed_run",
    "pause_resume_spam_no_duplicate_steps",
    "stop_stale_callback_and_finalize_retry_guard",
    "project_switch_during_live_drops_old_callbacks",
    "reopen_during_live_explicitly_clears_run_and_pump",
    "server_checkpoint_delay_failure_no_world_step_duplication",
    "string_comment_tick_surface_not_live",
    "legacy_selected_view_tamper_and_import_cancel_world_invariant",
    "overlay_magnifier_visibility_state_hash_invariant",
  ];
  const observed = new Set(evidence.cases.map((row) => row.id));
  const missing = requiredCases.filter((id) => !observed.has(id));
  assert(missing.length === 0, `missing race cases: ${missing.join(", ")}`);
  const outPath = path.join(OUT_DIR, "summary.detjson");
  await fs.writeFile(outPath, `${JSON.stringify(evidence, null, 2)}\n`, "utf8");
  console.log(JSON.stringify({ schema: evidence.schema, cases: evidence.cases.map((row) => row.id), out: path.relative(ROOT, outPath).replace(/\\/g, "/") }, null, 2));
  console.log("seamgrim_workbench_race_matrix: ok");
}

main().catch(async (err) => {
  evidence.failure = {
    code: String(err?.code || "E_G14_RACE_MATRIX_UNEXPECTED"),
    message: String(err?.message || err || "").slice(0, 2000),
  };
  try {
    await fs.mkdir(OUT_DIR, { recursive: true });
    await fs.writeFile(path.join(OUT_DIR, "summary.detjson"), `${JSON.stringify(evidence, null, 2)}\n`, "utf8");
  } catch (_) {
    // Best effort evidence write.
  }
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
