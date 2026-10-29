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
const OK = "seamgrim_legacy_deeplink_compat: ok";

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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-legacy-compat-"));
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

async function expectWorkbenchProject(page, url, projectId) {
  await page.goto(url, { waitUntil: "networkidle" });
  await page.waitForSelector(`[data-testid='approved-workbench-root'][data-project-id='${projectId}']`, { timeout: 30_000 });
  const state = await page.evaluate(() => ({
    title: document.title,
    projectId: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId,
    hasLegacy: Boolean(document.querySelector("#screen-browse")),
    source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
  }));
  assert(state.title === "셈그림 결정적 작업대", `workbench title mismatch: ${JSON.stringify(state)}`);
  assert(state.projectId === projectId, `project mismatch for ${url}: ${JSON.stringify(state)}`);
  assert(!state.hasLegacy, `legacy DOM initialized in workbench route: ${url}`);
  return state;
}

async function expectLegacy(page, url) {
  await page.goto(url, { waitUntil: "networkidle" });
  await page.waitForSelector("#app", { state: "attached", timeout: 30_000 });
  const state = await page.evaluate(() => ({
    title: document.title,
    hasWorkbench: Boolean(document.querySelector("[data-testid='approved-workbench-root']")),
    hasLegacy: Boolean(document.querySelector("#screen-browse")),
    hasCatalog: Boolean(document.querySelector("[data-testid='lesson-catalog']")),
  }));
  assert(state.title === "셈그림", `legacy title mismatch: ${JSON.stringify(state)}`);
  assert(!state.hasWorkbench, `workbench DOM initialized in legacy route: ${url}`);
  assert(state.hasLegacy && state.hasCatalog, `legacy shell incomplete: ${JSON.stringify(state)}`);
  return state;
}

async function main() {
  const { child, baseUrl, dataDir } = await startServer();
  const browser = await chromium.launch({ headless: true });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    const page = await context.newPage();
    await expectWorkbenchProject(page, `${baseUrl}/index.html?lesson=rep_phys_pendulum_wave1_v1`, "rep_phys_pendulum_wave1_v1");
    await expectWorkbenchProject(page, `${baseUrl}/index.html?lesson=rep_phys_projectile_xy_v1`, "rep_phys_projectile_xy_v1");
    await expectWorkbenchProject(page, `${baseUrl}/index.html?project=rep_grid_game_state_drop_v1`, "rep_grid_game_state_drop_v1");
    await expectLegacy(page, `${baseUrl}/index.html?lesson=legacy_only_unknown_v1`);
    await expectLegacy(page, `${baseUrl}/index.html?tab=studio&lesson=rep_phys_projectile_xy_v1`);
    await context.close();
    console.log(JSON.stringify({
      schema: "seamgrim.legacy_deeplink_compat.runner_result.v1",
      mapped_projects: ["rep_phys_pendulum_wave1_v1", "rep_phys_projectile_xy_v1", "rep_grid_game_state_drop_v1"],
      legacy_fallbacks: ["legacy_only_unknown_v1", "tab=studio"],
      data_dir: dataDir,
    }));
    console.log(OK);
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

await main();
