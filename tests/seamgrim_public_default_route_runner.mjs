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
const OK = "seamgrim_public_default_route: ok";

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

async function waitForHealth(baseUrl, child) {
  for (let i = 0; i < 90; i += 1) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    try {
      const response = await fetch(`${baseUrl}/api/health`);
      if (response.ok) return;
    } catch (_) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
  throw new Error("server health timeout");
}

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-public-default-"));
  const port = await freePort();
  const env = { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir };
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const stderr = [];
  child.stderr.on("data", (chunk) => stderr.push(String(chunk)));
  child.stdout.on("data", () => {});
  const baseUrl = `http://127.0.0.1:${port}`;
  await waitForHealth(baseUrl, child);
  return { child, baseUrl, dataDir, stderr };
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

async function assertWorkbench(page, label) {
  await page.waitForSelector("[data-testid='approved-workbench-root']", { state: "visible", timeout: 30_000 });
  const state = await page.evaluate(() => ({
    title: document.title,
    hasWorkbench: Boolean(document.querySelector("[data-testid='approved-workbench-root']")),
    hasLegacyBrowse: Boolean(document.querySelector("#screen-browse")),
    rootProject: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId ?? "",
  }));
  assert(state.title === "셈그림 결정적 작업대", `${label}: workbench title mismatch ${JSON.stringify(state)}`);
  assert(state.hasWorkbench, `${label}: workbench root missing`);
  assert(!state.hasLegacyBrowse, `${label}: legacy browse shell initialized with workbench`);
  return state;
}

async function assertLegacy(page, label) {
  await page.waitForSelector("#app", { state: "attached", timeout: 30_000 });
  const state = await page.evaluate(() => ({
    title: document.title,
    hasWorkbench: Boolean(document.querySelector("[data-testid='approved-workbench-root']")),
    hasLegacyBrowse: Boolean(document.querySelector("#screen-browse")),
    hasLegacyCatalog: Boolean(document.querySelector("[data-testid='lesson-catalog']")),
    visibleScreens: Array.from(document.querySelectorAll(".screen:not(.hidden)")).map((el) => el.id),
  }));
  assert(state.title === "셈그림", `${label}: legacy title mismatch ${JSON.stringify(state)}`);
  assert(!state.hasWorkbench, `${label}: workbench initialized in legacy shell`);
  assert(state.hasLegacyBrowse && state.hasLegacyCatalog, `${label}: legacy shell incomplete ${JSON.stringify(state)}`);
  return state;
}

async function main() {
  const { child, baseUrl, dataDir, stderr } = await startServer();
  const browser = await chromium.launch({ headless: true });
  const consoleFailures = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") consoleFailures.push(`${msg.type()}: ${msg.text()}`);
    });
    page.on("pageerror", (err) => consoleFailures.push(`pageerror: ${err.message}`));

    await page.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
    await assertWorkbench(page, "default root");
    assert(page.url().includes("/solutions/seamgrim_ui_mvp/ui/workbench/index.html"), `default did not redirect to workbench: ${page.url()}`);

    await page.goto(`${baseUrl}/index.html?first_delight=projectile`, { waitUntil: "networkidle" });
    await assertWorkbench(page, "default query");
    assert(page.url().includes("first_delight=projectile"), `safe query lost on workbench redirect: ${page.url()}`);

    await page.goto(`${baseUrl}/index.html?sg_cap=SECRET&first_delight=projectile`, { waitUntil: "networkidle" });
    await assertWorkbench(page, "capability query scrub");
    assert(!page.url().includes("sg_cap=SECRET"), `capability query leaked through default redirect: ${page.url()}`);
    assert(consoleFailures.length === 0, `default/workbench console failures: ${consoleFailures.join("\n")}`);
    consoleFailures.length = 0;

    await page.goto(`${baseUrl}/legacy.html`, { waitUntil: "networkidle" });
    await assertLegacy(page, "explicit legacy");

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/legacy.html`, { waitUntil: "networkidle" });
    await assertLegacy(page, "prefixed legacy");

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio`, { waitUntil: "networkidle" });
    await assertLegacy(page, "old tab deep link");
    consoleFailures.length = 0;

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "networkidle" });
    await assertLegacy(page, "old plain index route");
    await context.close();
    console.log(JSON.stringify({
      schema: "seamgrim.public_default_route.runner_result.v1",
      default_url: `${baseUrl}/`,
      workbench_url: `${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`,
      legacy_url: `${baseUrl}/legacy.html`,
      data_dir: dataDir,
      stderr_lines: stderr.length,
    }));
    console.log(OK);
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

await main();
