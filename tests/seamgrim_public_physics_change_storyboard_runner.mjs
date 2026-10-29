#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const SERVER = path.resolve(ROOT, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");
const OK = "seamgrim_public_physics_change_storyboard: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function freePort() {
  return new Promise((resolve, reject) => {
    import("node:net").then(({ default: net }) => {
      const server = net.createServer();
      server.once("error", reject);
      server.listen(0, "127.0.0.1", () => {
        const address = server.address();
        const port = address && typeof address !== "string" ? address.port : 0;
        server.close(() => resolve(port));
      });
    }).catch(reject);
  });
}

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-change-storyboard-"));
  const port = await freePort();
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", () => {});
  child.stderr.on("data", () => {});
  const baseUrl = `http://127.0.0.1:${port}`;
  for (let i = 0; i < 90; i += 1) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    try {
      const response = await fetch(`${baseUrl}/api/health`);
      if (response.ok) return { child, baseUrl, dataDir };
    } catch (_) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
  throw new Error("server health timeout");
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

async function canvasHash(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const { width, height } = canvas;
    const data = ctx.getImageData(0, 0, width, height).data;
    let hash = 0x811c9dc5;
    let nonBackground = 0;
    const stride = Math.max(4, Math.floor(data.length / 8000));
    for (let i = 0; i < data.length; i += stride) {
      hash ^= data[i] ?? 0;
      hash = Math.imul(hash, 0x01000193);
      if ((data[i + 3] ?? 0) && ((data[i] ?? 255) < 235 || (data[i + 1] ?? 255) < 240 || (data[i + 2] ?? 255) < 232)) {
        nonBackground += 1;
      }
    }
    return { hash: (hash >>> 0).toString(16).padStart(8, "0"), nonBackground, width, height };
  });
}

async function main() {
  const { child, baseUrl, dataDir } = await startServer();
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") failures.push(`${msg.type()}: ${msg.text()}`);
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));

    const steps = [];
    await page.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
    await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_projectile_xy_v1']", { timeout: 30_000 });
    steps.push("official projectile opened");

    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return Number(snapshot?.runs?.[0]?.committed_step_count ?? 0) >= 1
        && snapshot?.runs?.[0]?.latest_state_hash
        && Number(document.querySelector("[data-testid='semantic-observations']")?.dataset.rowCount ?? 0) > 0;
    }, null, { timeout: 30_000 });
    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
    await page.waitForFunction(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snapshot?.executionLifecycle?.status === "paused" && !snapshot?.livePump?.in_flight;
    }, null, { timeout: 30_000 });
    const first = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    const beforeCanvas = await canvasHash(page);
    assert(first.runs[0].latest_state_hash, "initial WASM state hash missing");
    assert(beforeCanvas.nonBackground > 100, `initial canvas blank: ${JSON.stringify(beforeCanvas)}`);
    steps.push("initial sealed WASM run");

    const source = await page.locator("[data-testid='source-editor']").inputValue();
    const changed = source.replace("중력가속도: 수 <- (9.8)", "중력가속도: 수 <- (4.9)");
    assert(changed !== source, "gravity source replacement did not apply");
    await page.fill("[data-testid='source-editor']", changed);
    await page.waitForFunction(() => document.querySelector("[data-testid='approved-workbench-root']")?.dataset.sourceDirty === "true");
    const candidate = await page.evaluate(() => ({
      dirty: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.sourceDirty,
      status: document.querySelector("[data-testid='run-status']")?.textContent ?? "",
      observations: document.querySelector("[data-testid='semantic-observations']")?.textContent ?? "",
      run: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs[0],
    }));
    const candidateCanvas = await canvasHash(page);
    assert(candidate.dirty === "true", `candidate state not marked dirty: ${JSON.stringify(candidate)}`);
    assert(candidate.status.includes("source candidate differs"), `candidate status missing: ${candidate.status}`);
    assert(!candidate.observations.includes("4.9"), `candidate edit mutated sealed observations: ${candidate.observations}`);
    assert(candidate.run.latest_state_hash === first.runs[0].latest_state_hash, "candidate edit changed sealed run state hash");
    assert(candidateCanvas.hash === beforeCanvas.hash, `candidate edit changed sealed canvas: ${JSON.stringify({ beforeCanvas, candidateCanvas })}`);
    steps.push("gravity edit staged as source candidate only");

    await page.click("[data-testid='lifecycle-stop']");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "done", null, { timeout: 30_000 });
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const latest = snapshot?.runs?.at?.(-1);
      return snapshot?.runs?.length >= 2 &&
        latest?.latest_state_hash &&
        Number(latest?.committed_step_count ?? 0) >= 1 &&
        String(document.querySelector("[data-testid='semantic-observations']")?.textContent ?? "").includes("4.9") &&
        document.querySelector("[data-testid='approved-workbench-root']")?.dataset.sourceDirty === "false";
    }, null, { timeout: 30_000 });
    const second = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert(second.runs.length >= 2, "second run not recorded");
    assert(second.runs.at(-1).latest_state_hash, `second WASM state hash missing: ${JSON.stringify(second)}`);
    assert(second.runs.at(-1).latest_state_hash !== first.runs[0].latest_state_hash || second.runs.at(-1).run_id !== first.runs[0].run_id, "second committed run not distinct");
    steps.push("changed source committed by new run");

    await page.click("[data-testid='project-save']");
    await page.waitForFunction(() => String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? "").includes("서버 저장 완료"), null, { timeout: 30_000 });
    const saved = await page.evaluate(() => ({
      status: document.querySelector("[data-testid='project-save-status']")?.textContent ?? "",
      viewHref: document.querySelector("[data-testid='project-view-link']")?.href ?? "",
      editHref: document.querySelector("[data-testid='project-edit-link']")?.href ?? "",
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(saved.status.includes("서버 저장 완료"), `server save status missing: ${saved.status}`);
    assert(saved.viewHref.includes("#sg_project=") && saved.viewHref.includes("sg_role=view"), `view link missing: ${saved.viewHref}`);
    assert(saved.editHref.includes("#sg_project=") && saved.editHref.includes("sg_role=edit"), `edit link missing: ${saved.editHref}`);
    assert(saved.snapshot.serverProject.head_revision_id, "server revision missing after save");
    steps.push("server save and view link created");

    assert(steps.length === 5, `storyboard step count mismatch: ${JSON.stringify(steps)}`);
    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);
    console.log(JSON.stringify({
      schema: "seamgrim.public_physics_change_storyboard.runner_result.v1",
      default_url: `${baseUrl}/`,
      project_id: second.projectId,
      initial_state_hash: first.runs[0].latest_state_hash,
      changed_state_hash: second.runs.at(-1).latest_state_hash,
      canvas_before: beforeCanvas.hash,
      canvas_after_candidate: candidateCanvas.hash,
      story_steps: steps,
      view_href_present: Boolean(saved.viewHref),
      data_dir: dataDir,
    }));
    console.log(OK);
    await context.close();
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

await main();
