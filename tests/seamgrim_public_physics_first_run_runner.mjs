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
const OK = "seamgrim_public_physics_first_run: ok";

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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-public-physics-"));
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

async function canvasSample(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const { width, height } = canvas;
    const data = ctx.getImageData(0, 0, width, height).data;
    let nonBackground = 0;
    for (let i = 0; i < data.length; i += 4) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const a = data[i + 3];
      if (a && (r < 235 || g < 240 || b < 232)) nonBackground += 1;
    }
    return { width, height, nonBackground };
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

    await page.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
    await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_projectile_xy_v1']", { timeout: 30_000 });
    const initial = await page.evaluate(() => ({
      url: location.href,
      projectId: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId,
      title: document.querySelector("#project-title")?.textContent?.trim(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      observations: document.querySelector("[data-testid='semantic-observations']")?.textContent ?? "",
      controls: Array.from(document.querySelectorAll("[data-testid='setup-control-list'] input[data-control-name]"))
        .map((el) => el.getAttribute("data-control-name")),
      viewSummary: document.querySelector("[data-testid='view-role-summary']")?.textContent ?? "",
      projectCards: Array.from(document.querySelectorAll("[data-testid^='project-card-']")).map((el) => el.dataset.projectId),
      activeCard: document.querySelector(".project-card.active")?.dataset.projectId ?? "",
    }));
    assert(initial.url.includes("/solutions/seamgrim_ui_mvp/ui/workbench/index.html"), `default URL not workbench: ${initial.url}`);
    assert(initial.projectId === "rep_phys_projectile_xy_v1", `first project mismatch: ${JSON.stringify(initial)}`);
    assert(initial.title === "포물선 운동", `title mismatch: ${JSON.stringify(initial)}`);
    assert(initial.source.includes("초기속도x") && initial.source.includes("중력가속도") && initial.source.includes("보개로 그려"), "projectile DDN source missing");
    assert(initial.observations.includes("실행 후 committed frame"), `initial observations should wait for committed frame: ${initial.observations}`);
    for (const control of ["초기속도x", "초기속도y", "중력가속도"]) {
      assert(initial.controls.includes(control), `projectile setup control missing: ${control}; ${JSON.stringify(initial.controls)}`);
    }
    assert(initial.viewSummary.includes("포물선 공간 보개"), `view policy not projectile: ${initial.viewSummary}`);
    assert(initial.projectCards[0] === "rep_phys_projectile_xy_v1", `projectile is not first card: ${JSON.stringify(initial.projectCards)}`);
    assert(initial.activeCard === "rep_phys_projectile_xy_v1", `projectile active card mismatch: ${initial.activeCard}`);

    const sample = await canvasSample(page);
    assert(sample.width >= 320 && sample.height >= 320, `projectile canvas too small: ${JSON.stringify(sample)}`);
    assert(sample.nonBackground > 1200, `projectile canvas blank: ${JSON.stringify(sample)}`);

    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const rowCount = Number(document.querySelector("[data-testid='semantic-observations']")?.dataset.rowCount ?? 0);
      return Number(snapshot?.runs?.[0]?.committed_step_count ?? 0) >= 1
        && snapshot?.runs?.[0]?.latest_state_hash
        && snapshot.runs[0].latest_state_hash !== "-"
        && rowCount > 0;
    }, null, { timeout: 30_000 });
    const afterRun = await page.evaluate(() => ({
      runtimeStatus: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runtimeStatus,
      run: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs[0],
      observations: document.querySelector("[data-testid='semantic-observations']")?.textContent ?? "",
      hashText: document.querySelector("[data-testid='run-hash']")?.textContent ?? "",
      madiText: document.querySelector("[data-testid='current-madi']")?.textContent ?? "",
    }));
    assert(afterRun.runtimeStatus === "running", `runtime did not enter running state: ${JSON.stringify(afterRun)}`);
    assert(afterRun.run.frame_count >= 1, `run frame missing: ${JSON.stringify(afterRun)}`);
    assert(/x|초기속도x|가로속도/.test(afterRun.observations), `x velocity observation missing after run: ${afterRun.observations}`);
    assert(/y|초기속도y|세로속도/.test(afterRun.observations), `y velocity observation missing after run: ${afterRun.observations}`);
    assert(/중력|중력가속도/.test(afterRun.observations), `gravity observation missing after run: ${afterRun.observations}`);
    assert(afterRun.hashText.includes("state_hash:"), `state hash not rendered: ${JSON.stringify(afterRun)}`);
    assert(afterRun.madiText.includes("madi:"), `madi not rendered: ${JSON.stringify(afterRun)}`);
    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);

    console.log(JSON.stringify({
      schema: "seamgrim.public_physics_first_run.runner_result.v1",
      default_url: `${baseUrl}/`,
      final_url: initial.url,
      project_id: initial.projectId,
      latest_state_hash: afterRun.run.latest_state_hash,
      canvas_non_background: sample.nonBackground,
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
