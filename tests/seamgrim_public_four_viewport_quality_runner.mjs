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
const OUT_DIR = path.resolve(ROOT, "out/seamgrim_public_wedge_g8");
const OK = "seamgrim_public_four_viewport_quality: ok";

const VIEWPORTS = [
  { name: "desktop_1440x900", width: 1440, height: 900 },
  { name: "tablet_landscape_1024x768", width: 1024, height: 768 },
  { name: "tablet_portrait_778x912", width: 778, height: 912 },
  { name: "mobile_390x844", width: 390, height: 844 },
];

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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-four-viewport-"));
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

async function pageMetrics(page) {
  return page.evaluate(() => {
    const source = document.querySelector("[data-testid='source-editor']");
    const sourcePanel = document.querySelector("[data-testid='source-panel']");
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const canvasWrap = document.querySelector("[data-testid='bogae-canvas-wrap']");
    const buttons = Array.from(document.querySelectorAll("button"));
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let nonBackground = 0;
    for (let i = 0; i < data.length; i += 16) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const a = data[i + 3];
      if (a && (r < 235 || g < 240 || b < 232)) nonBackground += 1;
    }
    return {
      scrollWidth: document.documentElement.scrollWidth,
      clientWidth: document.documentElement.clientWidth,
      bodyScrollWidth: document.body.scrollWidth,
      sourceHeight: source?.getBoundingClientRect().height ?? 0,
      sourceWidth: source?.getBoundingClientRect().width ?? 0,
      sourcePanelHeight: sourcePanel?.getBoundingClientRect().height ?? 0,
      canvasCssHeight: canvasWrap?.getBoundingClientRect().height ?? 0,
      canvasCssWidth: canvasWrap?.getBoundingClientRect().width ?? 0,
      canvasWidth: canvas?.width ?? 0,
      canvasHeight: canvas?.height ?? 0,
      nonBackground,
      unnamedButtons: buttons
        .filter((button) => !(button.getAttribute("aria-label") || button.getAttribute("title") || button.textContent?.trim()))
        .map((button) => button.outerHTML.slice(0, 80)),
      saveVisible: Boolean(document.querySelector("[data-testid='project-save']")?.offsetParent),
      viewLinkVisible: Boolean(document.querySelector("[data-testid='project-view-link']")?.offsetParent),
      cloneVisible: Boolean(document.querySelector("[data-testid='project-clone']")?.offsetParent),
      activeProject: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId,
    };
  });
}

async function storageControlMetrics(page) {
  return page.evaluate(() => ({
    drawerOpen: document.querySelector("[data-testid='project-storage-drawer']")?.open === true,
    viewLinkVisible: Boolean(document.querySelector("[data-testid='project-view-link']")?.offsetParent),
    cloneVisible: Boolean(document.querySelector("[data-testid='project-clone']")?.offsetParent),
  }));
}

async function main() {
  await fs.mkdir(OUT_DIR, { recursive: true });
  const { child, baseUrl, dataDir } = await startServer();
  const browser = await chromium.launch({ headless: true });
  const results = [];
  try {
    for (const vp of VIEWPORTS) {
      const context = await browser.newContext({
        viewport: { width: vp.width, height: vp.height },
        locale: "ko-KR",
        reducedMotion: "reduce",
      });
      const page = await context.newPage();
      const failures = [];
      page.on("console", (msg) => {
        if (msg.type() === "error") failures.push(`console: ${msg.text()}`);
      });
      page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
      const started = performance.now();
      await page.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
      await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_projectile_xy_v1']", { timeout: 30_000 });
      await page.click("[data-testid='lifecycle-run']");
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().runs?.[0]?.latest_state_hash, null, { timeout: 30_000 });
      const renderMs = Math.round(performance.now() - started);
      await page.keyboard.press("Tab");
      const focusTag = await page.evaluate(() => ({
        tag: document.activeElement?.tagName ?? "",
        testid: document.activeElement?.getAttribute("data-testid") ?? "",
        aria: document.activeElement?.getAttribute("aria-label") ?? "",
        title: document.activeElement?.getAttribute("title") ?? "",
      }));
      const metrics = await pageMetrics(page);
      await page.locator("[data-testid='project-more-menu'] > summary").click();
      await page.click("[data-testid='project-storage-open']");
      await page.waitForFunction(() => document.querySelector("[data-testid='project-storage-drawer']")?.open === true, null, { timeout: 10_000 });
      const storageMetrics = await storageControlMetrics(page);
      const screenshot = path.join(OUT_DIR, `${vp.name}.png`);
      await page.screenshot({ path: screenshot, fullPage: true });
      assert(metrics.activeProject === "rep_phys_projectile_xy_v1", `${vp.name}: wrong project ${metrics.activeProject}`);
      assert(metrics.scrollWidth <= metrics.clientWidth + 2, `${vp.name}: document horizontal overflow ${JSON.stringify(metrics)}`);
      assert(metrics.bodyScrollWidth <= metrics.clientWidth + 2, `${vp.name}: body horizontal overflow ${JSON.stringify(metrics)}`);
      assert(metrics.sourceHeight >= 180 && metrics.sourceWidth >= 260, `${vp.name}: source editor too small ${JSON.stringify(metrics)}`);
      assert(metrics.canvasCssHeight >= 220 && metrics.canvasCssWidth >= 240, `${vp.name}: canvas display too small ${JSON.stringify(metrics)}`);
      assert(metrics.nonBackground > 200, `${vp.name}: canvas appears blank ${JSON.stringify(metrics)}`);
      assert(metrics.unnamedButtons.length === 0, `${vp.name}: unnamed buttons ${JSON.stringify(metrics.unnamedButtons)}`);
      assert(metrics.saveVisible && storageMetrics.drawerOpen && storageMetrics.viewLinkVisible && storageMetrics.cloneVisible, `${vp.name}: save/share/clone controls not visible ${JSON.stringify({ metrics, storageMetrics })}`);
      assert(focusTag.tag, `${vp.name}: keyboard focus missing`);
      assert(renderMs < 15_000, `${vp.name}: first render timeout ${renderMs}ms`);
      assert(failures.length === 0, `${vp.name}: browser failures ${failures.join("\n")}`);
      results.push({ viewport: vp, render_ms: renderMs, metrics, storageMetrics, focus: focusTag, screenshot });
      await context.close();
    }
    const reportPath = path.join(OUT_DIR, "summary.detjson");
    await fs.writeFile(reportPath, `${JSON.stringify({
      schema: "seamgrim.public_four_viewport_quality.runner_result.v1",
      base_url: baseUrl,
      data_dir: dataDir,
      results,
    }, null, 2)}\n`, "utf8");
    console.log(JSON.stringify({
      schema: "seamgrim.public_four_viewport_quality.runner_result.v1",
      result_count: results.length,
      screenshots: results.map((row) => row.screenshot),
      report: reportPath,
    }));
    console.log(OK);
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

await main();
