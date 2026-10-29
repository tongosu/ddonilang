#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_approved_workbench_shell: ok";

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

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  assert(stat?.isFile(), `missing file: ${file}`);
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

async function ratio(page) {
  return page.evaluate(() => {
    const source = document.querySelector("[data-testid='source-panel']").getBoundingClientRect();
    const bogae = document.querySelector("[data-testid='bogae-panel']").getBoundingClientRect();
    const total = source.width + bogae.width;
    return {
      source: source.width / total,
      bogae: bogae.width / total,
      sourceWidth: source.width,
      bogaeWidth: bogae.width,
    };
  });
}

async function assertNoHorizontalOverflow(page) {
  const metrics = await page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    clientWidth: document.documentElement.clientWidth,
    bodyScrollWidth: document.body.scrollWidth,
  }));
  assert(metrics.scrollWidth <= metrics.clientWidth + 2, `document horizontal overflow: ${JSON.stringify(metrics)}`);
  assert(metrics.bodyScrollWidth <= metrics.clientWidth + 2, `body horizontal overflow: ${JSON.stringify(metrics)}`);
}

async function assertCanvasNonBlank(page) {
  const sample = await page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const { width, height } = canvas;
    const data = ctx.getImageData(0, 0, width, height).data;
    let nonWhite = 0;
    for (let i = 0; i < data.length; i += 4) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const a = data[i + 3];
      if (a && (r < 240 || g < 240 || b < 240)) nonWhite += 1;
    }
    return { width, height, nonWhite };
  });
  assert(sample.width >= 320 && sample.height >= 240, `canvas too small: ${JSON.stringify(sample)}`);
  assert(sample.nonWhite > 500, `canvas appears blank: ${JSON.stringify(sample)}`);
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const productFiles = [
    "solutions/seamgrim_ui_mvp/ui/workbench/index.html",
    "solutions/seamgrim_ui_mvp/ui/workbench/workbench.css",
    "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js",
    "solutions/seamgrim_ui_mvp/lessons/rep_phys_projectile_xy_v1/lesson.ddn",
    "solutions/seamgrim_ui_mvp/lessons/rep_phys_pendulum_wave1_v1/lesson.ddn",
    "solutions/seamgrim_ui_mvp/lessons/rep_grid_game_state_drop_v1/lesson.ddn",
  ];
  for (const rel of productFiles) {
    await requireFile(path.join(root, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true,
      ...(process.env.SEAMGRIM_BROWSER_CHANNEL ? { channel: process.env.SEAMGRIM_BROWSER_CHANNEL } : {}) });
    const context = await browser.newContext({
      viewport: { width: 1440, height: 900 },
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
    await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_projectile_xy_v1']");

    const global = await page.evaluate(() => ({
      title: document.title,
      ddnSearch: document.querySelector("[data-testid='global-ddn-search']")?.textContent?.trim(),
      official: document.querySelector("[data-testid='global-official-collection']")?.textContent?.trim(),
      myWork: document.querySelector("[data-testid='global-my-work']")?.textContent?.trim(),
      profileShelfCount: Array.from(document.querySelectorAll("button, a, nav *")).filter((el) => /학생|교사|프로필/.test(el.textContent ?? "")).length,
      projectCards: document.querySelectorAll("[data-testid^='project-card-']").length,
      singleFile: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.singleFile,
      fileDrawerHidden: document.querySelector("[data-testid='file-drawer']")?.classList.contains("hidden"),
      fileTabsHidden: document.querySelector("[data-testid='file-tabs']")?.classList.contains("hidden"),
      sourceText: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    assert(global.title === "셈그림 결정적 작업대 · 1.0.0-alpha.1 · 알파 후보", "document title mismatch");
    assert(global.ddnSearch === "DDN 찾기", "DDN search entry missing");
    assert(global.official === "공식 모음", "official collection entry missing");
    assert(global.myWork === "내 작업", "my work entry missing");
    assert(global.profileShelfCount === 0, "fixed profile shelf text should not exist");
    assert(global.projectCards >= 3, "project discovery cards missing");
    assert(global.singleFile === "true", "pendulum should be a single-file project in this shell");
    assert(global.fileDrawerHidden === true, "single-file project should hide file drawer");
    assert(global.fileTabsHidden === true, "single-file project should hide module tabs");
    assert(global.sourceText.includes("포물선 운동"), "projectile DDN source not loaded");
    await assertCanvasNonBlank(page);

    let layoutRatio = await ratio(page);
    assert(layoutRatio.source > 0.55 && layoutRatio.source < 0.65, `DDN main ratio mismatch: ${JSON.stringify(layoutRatio)}`);
    assert(layoutRatio.bogae > 0.35 && layoutRatio.bogae < 0.45, `Bogae side ratio mismatch: ${JSON.stringify(layoutRatio)}`);

    await page.click("[data-testid='mode-bogae-main']");
    await page.waitForFunction(() => document.querySelector("[data-testid='workbench-layout']")?.dataset.mode === "bogae-main");
    layoutRatio = await ratio(page);
    assert(layoutRatio.source > 0.35 && layoutRatio.source < 0.45, `Bogae main source ratio mismatch: ${JSON.stringify(layoutRatio)}`);
    assert(layoutRatio.bogae > 0.55 && layoutRatio.bogae < 0.65, `Bogae main ratio mismatch: ${JSON.stringify(layoutRatio)}`);

    await page.click("[data-testid='mode-bogae-full']");
    await page.waitForFunction(() => document.querySelector("[data-testid='workbench-layout']")?.dataset.mode === "bogae-full");
    const fullMode = await page.evaluate(() => ({
      sourceAccess: Boolean(document.querySelector("[data-testid='source-access-button']")),
      modeText: document.querySelector("#mode-ratio-label")?.textContent?.trim(),
      bogaeWidth: document.querySelector("[data-testid='bogae-panel']").getBoundingClientRect().width,
      layoutWidth: document.querySelector("[data-testid='workbench-layout']").getBoundingClientRect().width,
    }));
    assert(fullMode.sourceAccess, "source access button missing in full mode");
    assert(fullMode.modeText.includes("원문 접근 유지"), "full mode source access label missing");
    assert(fullMode.bogaeWidth > fullMode.layoutWidth * 0.92, `bogae full width mismatch: ${JSON.stringify(fullMode)}`);

    await page.click("[data-testid='project-card-rep_phys_pendulum_wave1_v1']");
    await page.waitForFunction(() => document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId === "rep_phys_pendulum_wave1_v1");
    const pendulum = await page.evaluate(() => ({
      singleFile: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.singleFile,
      sourceText: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    assert(pendulum.singleFile === "true", "pendulum remains a single-file project");
    assert(pendulum.sourceText.includes("진자 보기 코어"), "pendulum DDN source not loaded after selection");

    await page.click("[data-testid='project-card-rep_grid_game_state_drop_v1']");
    await page.waitForFunction(() => document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId === "rep_grid_game_state_drop_v1");
    const multi = await page.evaluate(() => ({
      singleFile: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.singleFile,
      fileDrawerHidden: document.querySelector("[data-testid='file-drawer']")?.classList.contains("hidden"),
      tabs: Array.from(document.querySelectorAll("[data-testid^='file-tab-']")).map((el) => el.textContent.trim()),
      fileRows: Array.from(document.querySelectorAll("[data-testid^='file-row-']")).map((el) => el.textContent.trim()),
      sourceText: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    assert(multi.singleFile === "false", "grid project should expose multi-file mode");
    assert(multi.fileDrawerHidden === false, "multi-file project should show file drawer");
    assert(multi.tabs.length >= 2 && multi.tabs.length <= 3, `multi-file tab count mismatch: ${JSON.stringify(multi.tabs)}`);
    assert(multi.fileRows.length >= 4, "multi-file drawer rows missing");
    assert(multi.sourceText.includes("격자 상태와 낙하 조각"), "grid DDN source not loaded");

    await page.fill("[data-testid='file-search']", "space");
    const filteredRows = await page.evaluate(() => Array.from(document.querySelectorAll("[data-testid^='file-row-']")).map((el) => el.textContent.trim()));
    assert(filteredRows.length === 1 && filteredRows[0].includes("space2d.json"), `file drawer search mismatch: ${JSON.stringify(filteredRows)}`);

    await page.setViewportSize({ width: 390, height: 844 });
    await page.waitForTimeout(100);
    await assertNoHorizontalOverflow(page);
    await assertCanvasNonBlank(page);

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
