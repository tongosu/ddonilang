#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_workbench_animation_oneshot_gate: ok";

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

async function waitReady(page) {
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snap?.readiness?.initialization === "ready";
  }, null, { timeout: 30000 });
}

async function selectProject(page, projectId) {
  await page.click(`[data-testid='project-card-${projectId}']`);
  await page.waitForFunction(
    (id) => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.projectId === id,
    projectId,
    { timeout: 30000 },
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
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.projectId && snap.projectId !== previousId && snap.project?.native_creator === true;
    },
    previous?.projectId ?? "",
    { timeout: 30000 },
  );
}

function liveEquivalenceSource(title) {
  return `설정 {
  제목: "${title}".
  마디수: 20.
}.

채비 {
  값: 수 <- 0.
}.

(매마디)마다 {
  값 <- 값 + 1.
  값 보여주기.
}.`;
}

function oneShotSource() {
  return `설정 {
  제목: "G12 one-shot 정적 결과".
}.

채비 {
  a: 수 <- 2.
  b: 수 <- 1.
  x: 수 <- 2.
  y: 수 <- 5.
}.

x 보여주기.
y 보여주기.`;
}

async function canvasEvidence(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    if (!canvas) return null;
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
      non_white: nonWhite,
      pixel_hash: `fnv1a32:${(hash >>> 0).toString(16).padStart(8, "0")}`,
      canvas_signature: canvas.dataset.canvasSignature ?? "",
      semantic_payload_hash: canvas.dataset.semanticPayloadHash ?? "",
      committed_state_hash: canvas.dataset.committedStateHash ?? "",
      view_artifact_hash: canvas.dataset.viewArtifactHash ?? "",
    };
  });
}

function activeRun(snap) {
  return snap?.runs?.find((run) => run.run_id === snap.activeRunId) ?? snap?.runs?.at(-1) ?? null;
}

function committedStepCount(snap) {
  return Number(activeRun(snap)?.committed_step_count ?? 0);
}

function stepStateHashes(snap, intent = null) {
  const frames = activeRun(snap)?.frames ?? [];
  return frames
    .filter((frame) => frame.input_snapshot?.intent !== "run_start")
    .filter((frame) => !intent || frame.input_snapshot?.intent === intent)
    .map((frame) => frame.state_hash);
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
  return snapshot(page);
}

async function pauseAndAssertFrozen(page, label) {
  await page.click("[data-testid='lifecycle-run'][data-action='pause']");
  await page.waitForFunction(
    () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "paused",
    null,
    { timeout: 60000 },
  );
  const paused = await snapshot(page);
  const count = committedStepCount(paused);
  const hash = paused.activeFrame?.state_hash ?? "";
  await page.waitForTimeout(220);
  const later = await snapshot(page);
  assert(committedStepCount(later) === count, `${label}: pause did not freeze committed step count`);
  assert(later.activeFrame?.state_hash === hash, `${label}: pause changed active state hash`);
  return paused;
}

async function assertSameFrameRedrawAndViewBoundary(page, label) {
  const before = await snapshot(page);
  const stateHash = before.activeFrame?.state_hash ?? "";
  const semanticHash = before.semanticViewPayload?.semantic_payload_hash ?? "";
  await page.click("[data-mode='ddn-main']");
  await page.waitForTimeout(80);
  const redraw = await snapshot(page);
  assert(redraw.activeFrame?.state_hash === stateHash, `${label}: redraw changed state hash`);
  assert(redraw.semanticViewPayload?.semantic_payload_hash === semanticHash, `${label}: same-frame redraw changed semantic payload hash`);
  await page.click("[data-testid='view-role-side']");
  await page.waitForFunction(() => document.querySelector("[data-testid='bogae-stage']")?.dataset.sideOpen === "true");
  const sideOpen = await snapshot(page);
  assert(sideOpen.activeFrame?.state_hash === stateHash, `${label}: opening side view changed state hash`);
  await page.click("[data-testid='side-view-close']");
  await page.waitForFunction(() => document.querySelector("[data-testid='bogae-stage']")?.dataset.sideOpen === "false");
  const sideClosed = await snapshot(page);
  assert(sideClosed.activeFrame?.state_hash === stateHash, `${label}: closing side view changed state hash`);
}

async function resumeAndStop(page, label) {
  const beforeResume = await snapshot(page);
  const count = committedStepCount(beforeResume);
  await page.click("[data-testid='lifecycle-run'][data-action='resume']");
  await waitForCommittedCount(page, count + 1);
  const resumed = await snapshot(page);
  assert(committedStepCount(resumed) > count, `${label}: resume did not advance`);
  await page.click("[data-testid='lifecycle-stop']");
  await page.waitForFunction(
    () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "done",
    null,
    { timeout: 60000 },
  );
  const stopped = await snapshot(page);
  const stoppedCount = committedStepCount(stopped);
  await page.waitForTimeout(220);
  const later = await snapshot(page);
  assert(committedStepCount(later) === stoppedCount, `${label}: stop/done advanced after finalization`);
  return { resumed_count: committedStepCount(resumed), stopped_count: stoppedCount };
}

async function proveLiveAnimation(page, projectId, label) {
  await selectProject(page, projectId);
  await page.click("[data-testid='lifecycle-run']");
  const samples = [];
  for (const target of [1, 2, 3]) {
    const snap = await waitForCommittedCount(page, target);
    const canvas = await canvasEvidence(page);
    assert(canvas?.non_white > 500, `${label}: canvas blank at step ${target}`);
    samples.push({
      target,
      committed_step_count: committedStepCount(snap),
      madi: snap.activeFrame?.madi,
      state_hash: snap.activeFrame?.state_hash,
      semantic_payload_hash: snap.semanticViewPayload?.semantic_payload_hash,
      pixel_hash: canvas.pixel_hash,
    });
  }
  const last = samples.at(-1);
  assert(Number(last?.madi ?? 0) >= 3, `${label}: madi did not reach 3 without extra step click`);
  assert(new Set(samples.map((row) => row.state_hash)).size >= 3, `${label}: committed state hashes did not change`);
  assert(new Set(samples.map((row) => row.semantic_payload_hash)).size >= 3, `${label}: semantic payload hashes did not change`);
  assert(new Set(samples.map((row) => row.pixel_hash)).size >= 3, `${label}: canvas pixel hashes did not change`);
  await pauseAndAssertFrozen(page, label);
  await assertSameFrameRedrawAndViewBoundary(page, label);
  const final = await resumeAndStop(page, label);
  return { project_id: projectId, samples, ...final };
}

async function proveOneShot(page) {
  await createSingleProject(page, "G12 one-shot 정적 결과");
  await page.fill("[data-testid='source-editor']", oneShotSource());
  await page.click("[data-testid='lifecycle-run']");
  await page.waitForFunction(
    () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "done",
    null,
    { timeout: 60000 },
  );
  const done = await snapshot(page);
  const canvas = await canvasEvidence(page);
  assert(done.executionLifecycle.mode === "oneshot", "one-shot source did not use oneshot mode");
  assert(committedStepCount(done) === 1, `one-shot committed unexpected steps: ${committedStepCount(done)}`);
  assert(canvas?.non_white > 500, "one-shot static result view is blank");
  assert(done.controls.lifecycle_pause.enabled === false, "one-shot pause control enabled");
  assert(done.controls.lifecycle_resume.enabled === false, "one-shot resume control enabled");
  assert(done.controls.lifecycle_step.enabled === false, "one-shot step control enabled");
  const hash = done.activeFrame?.state_hash ?? "";
  await page.waitForTimeout(240);
  const later = await snapshot(page);
  assert(committedStepCount(later) === 1, "one-shot auto-progressed after done");
  assert(later.activeFrame?.state_hash === hash, "one-shot state hash changed after done");
  return {
    state_hash: hash,
    semantic_payload_hash: done.semanticViewPayload?.semantic_payload_hash ?? "",
    pixel_hash: canvas.pixel_hash,
  };
}

async function prepareLiveSource(page, name, source) {
  await createSingleProject(page, name);
  await page.fill("[data-testid='source-editor']", source);
}

async function autoSequence(page, source, n) {
  await prepareLiveSource(page, `G12 auto ${n}`, source);
  await page.click("[data-testid='lifecycle-run']");
  await waitForCommittedCount(page, n);
  await pauseAndAssertFrozen(page, `auto-${n}`);
  const snap = await snapshot(page);
  const hashes = stepStateHashes(snap, "auto_step").slice(0, n);
  assert(hashes.length === n, `auto-${n}: expected ${n} auto hashes, got ${hashes.length}`);
  return hashes;
}

async function manualSequence(page, source, n) {
  await prepareLiveSource(page, `G12 manual ${n}`, source);
  await page.evaluate(async () => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    await api.startRun();
    api.pauseRun();
  });
  await page.waitForFunction(
    () => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const run = snap?.runs?.find((row) => row.run_id === snap.activeRunId);
      return snap?.executionLifecycle?.status === "paused" && Number(run?.committed_step_count ?? 0) === 0;
    },
    null,
    { timeout: 60000 },
  );
  for (let i = 1; i <= n; i += 1) {
    await page.click("[data-testid='lifecycle-step']");
    await page.waitForFunction(
      (target) => {
        const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
        const run = snap?.runs?.find((row) => row.run_id === snap.activeRunId);
        return snap?.executionLifecycle?.status === "paused" && Number(run?.committed_step_count ?? 0) === target;
      },
      i,
      { timeout: 60000 },
    );
  }
  const snap = await snapshot(page);
  const hashes = stepStateHashes(snap, "step").slice(0, n);
  assert(hashes.length === n, `manual-${n}: expected ${n} manual hashes, got ${hashes.length}`);
  assert(stepStateHashes(snap, "auto_step").length === 0, "manual sequence leaked an auto step");
  return hashes;
}

async function pauseResumeSequence(page, source) {
  await prepareLiveSource(page, "G12 pause resume 10", source);
  await page.click("[data-testid='lifecycle-run']");
  await waitForCommittedCount(page, 5);
  await pauseAndAssertFrozen(page, "pause-resume-10");
  await page.click("[data-testid='lifecycle-run'][data-action='resume']");
  await waitForCommittedCount(page, 10);
  await pauseAndAssertFrozen(page, "pause-resume-10-final");
  const hashes = stepStateHashes(await snapshot(page), "auto_step").slice(0, 10);
  assert(hashes.length === 10, `pause/resume: expected 10 hashes, got ${hashes.length}`);
  return hashes;
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
    page.on("console", (msg) => {
      if (msg.type() === "error") pageErrors.push(msg.text());
    });
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await waitReady(page);

    const projectile = await proveLiveAnimation(page, "rep_phys_projectile_xy_v1", "projectile");
    const pendulum = await proveLiveAnimation(page, "rep_phys_pendulum_wave1_v1", "pendulum");
    const oneshot = await proveOneShot(page);

    const source = liveEquivalenceSource("G12 동등성");
    const auto5 = await autoSequence(page, source, 5);
    const manual5 = await manualSequence(page, source, 5);
    assert(JSON.stringify(auto5) === JSON.stringify(manual5), `auto/manual 5-step hashes differ: ${JSON.stringify({ auto5, manual5 })}`);
    const continuous10 = await autoSequence(page, source, 10);
    const paused10 = await pauseResumeSequence(page, source);
    assert(JSON.stringify(continuous10) === JSON.stringify(paused10), `continuous 10 differs from 5+pause+resume+5: ${JSON.stringify({ continuous10, paused10 })}`);
    assert(pageErrors.length === 0, `browser errors: ${pageErrors.join(" | ")}`);

    console.log(JSON.stringify({
      schema: "seamgrim.workbench_animation_oneshot_gate.runner_result.v1",
      projectile,
      pendulum,
      oneshot,
      auto_manual_equal_steps: auto5.length,
      continuous_pause_resume_equal_steps: continuous10.length,
    }));
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
