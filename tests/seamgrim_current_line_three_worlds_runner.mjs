#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_current_line_three_worlds: ok";
const WORLDS = [
  {
    firstId: "linear-function",
    lessonId: "rep_math_function_line_v1",
    role: "graph_series",
    graphAxes: ["x", "y"],
    segmentDataset: "mainGraphSeriesSegmentCount",
    controls: [["기울기", "기울기"], ["절편", "y 절편"], ["간격", "x 간격"]],
    changedControl: { name: "기울기", value: "3.5", source: "기울기: 수 <- (3.5)" },
    alias: "math_function_line",
  },
  {
    firstId: "projectile",
    lessonId: "rep_phys_projectile_xy_v1",
    role: "trajectory_trail",
    graphAxes: ["x", "y"],
    segmentDataset: "mainTrajectorySegmentCount",
    controls: [["초기속도x", "가로 속도"], ["초기속도y", "세로 속도"], ["중력가속도", "중력"]],
    changedControl: null,
    alias: "physics_projectile",
  },
  {
    firstId: "market-tax",
    lessonId: "rep_econ_supply_demand_tax_v1",
    role: "graph_series",
    graphAxes: ["t", "가격"],
    segmentDataset: "mainGraphSeriesSegmentCount",
    controls: [["시작가격", "시작 가격"], ["세금", "세금"], ["수요기울기", "수요 민감도"]],
    changedControl: { name: "세금", value: "12", source: "세금: 수 <- (12)" },
    alias: "econ_supply_demand_tax",
  },
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".toml") || file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  if (file.endsWith(".md")) return "text/markdown; charset=utf-8";
  return "application/octet-stream";
}

function createServer(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      if (url.pathname === "/favicon.ico") {
        res.writeHead(204, { "cache-control": "no-store" });
        res.end();
        return;
      }
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/index.html" : url.pathname);
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
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
        reject(new Error("failed to bind static server"));
        return;
      }
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d)\.json$/i.test(pathname)
    ) return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:text\.md|maegim_control\.json)$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function installFailureCapture(page, failures) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    if (String(msg.text() ?? "").includes("Failed to load resource")) return;
    failures.push(`console error: ${msg.text()}`);
  });
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`response ${res.status()}: ${res.url()}`);
  });
}

async function waitWorldReady(page, world) {
  await page.waitForSelector(`#screen-run[data-first-delight='${world.firstId}']`, { state: "visible" });
  await page.waitForFunction(({ role, segmentDataset }) => {
    const root = document.querySelector("#screen-run");
    return ["space2d", "debug-fallback"].includes(String(root?.dataset?.mainVisualMode ?? ""))
      && String(root?.dataset?.mainViewRoles ?? "").split(",").includes(role)
      && Number(root?.dataset?.[segmentDataset] ?? 0) >= 3;
  }, { role: world.role, segmentDataset: world.segmentDataset }, { timeout: 20_000 });
  await page.waitForSelector("[data-testid='lesson-control-panel']", { state: "visible" });
}

async function readCanvasEvidence(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']");
    if (!(canvas instanceof HTMLCanvasElement)) return null;
    const rect = canvas.getBoundingClientRect();
    const ctx = canvas.getContext("2d");
    if (!ctx || canvas.width <= 0 || canvas.height <= 0) return null;
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    const colors = new Set();
    const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 4000) * 4);
    for (let i = 0; i < data.length; i += stride) {
      colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
      if (colors.size > 12) break;
    }
    return { width: rect.width, height: rect.height, colors: colors.size };
  });
}

async function readFullHash(page) {
  return page.locator("#run-mirror-hash").getAttribute("title");
}

async function verifyDesktopWorld(page, baseUrl, world) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=${world.firstId}`, {
    waitUntil: "domcontentloaded",
  });
  await waitWorldReady(page, world);
  const route = new URL(page.url());
  assert(route.searchParams.get("lesson") === world.lessonId, `${world.firstId}: canonical lesson mismatch`);
  assert(route.searchParams.get("tab") === "studio", `${world.firstId}: studio route missing`);

  const model = await page.evaluate((segmentDataset) => {
    const root = document.querySelector("#screen-run");
    return {
      launchKind: root?.dataset?.launchKind ?? "",
      role: root?.dataset?.mainViewRoles ?? "",
      axes: String(root?.dataset?.mainViewAxes ?? "").split(",").filter(Boolean),
      segmentCount: Number(root?.dataset?.[segmentDataset] ?? 0),
      controls: [...document.querySelectorAll(".slider-row")].map((row) => [
        row.dataset.controlName ?? "",
        row.querySelector("label")?.textContent?.trim() ?? "",
      ]),
      errorHidden: document.querySelector("[data-testid='lesson-error-panel']")?.classList.contains("hidden") ?? false,
      overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    };
  }, world.segmentDataset);
  assert(model.launchKind === "first_delight", `${world.firstId}: launch kind mismatch`);
  assert(model.role.split(",").includes(world.role), `${world.firstId}: role mismatch ${model.role}`);
  assert(JSON.stringify(model.axes) === JSON.stringify(world.graphAxes), `${world.firstId}: axes ${JSON.stringify(model.axes)}`);
  assert(model.segmentCount >= 3, `${world.firstId}: segment count ${model.segmentCount}`);
  assert(JSON.stringify(model.controls) === JSON.stringify(world.controls), `${world.firstId}: controls ${JSON.stringify(model.controls)}`);
  assert(model.errorHidden, `${world.firstId}: error panel visible`);
  assert(model.overflow <= 1, `${world.firstId}: horizontal overflow ${model.overflow}`);
  const canvas = await readCanvasEvidence(page);
  assert(canvas && canvas.width > 200 && canvas.height > 160 && canvas.colors >= 4, `${world.firstId}: blank canvas ${JSON.stringify(canvas)}`);

  if (!world.changedControl) {
    await page.getByTestId("lesson-pause-button").click();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "paused");
    return;
  }

  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "done", null, { timeout: 20_000 });
  const defaultHash = await readFullHash(page);
  assert(defaultHash?.includes("blake3:"), `${world.firstId}: default state hash missing`);
  const input = page.locator(`input[type='number'][data-control-name='${world.changedControl.name}']`);
  await input.fill(world.changedControl.value);
  await input.dispatchEvent("change");
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "running");
  const source = await page.locator("#run-ddn-preview").inputValue();
  assert(source.includes(world.changedControl.source), `${world.firstId}: changed source missing`);
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "done", null, { timeout: 20_000 });
  const changedHash = await readFullHash(page);
  assert(changedHash?.includes("blake3:") && changedHash !== defaultHash, `${world.firstId}: changed hash did not change`);
  await page.getByTestId("lesson-reset-button").click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "idle");
  await page.getByTestId("lesson-run-button").click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "done", null, { timeout: 20_000 });
  const repeatedHash = await readFullHash(page);
  assert(repeatedHash === changedHash, `${world.firstId}: same changed input is not deterministic`);
}

async function verifyAlias(page, baseUrl, world) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&lesson=${world.alias}`, {
    waitUntil: "domcontentloaded",
  });
  await page.waitForFunction((lessonId) => new URL(location.href).searchParams.get("lesson") === lessonId, world.lessonId);
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.title ?? "").includes("blake3:"), null, { timeout: 20_000 });
  const status = await page.locator("#screen-run").getAttribute("data-engine-status");
  if (status === "running") {
    await page.getByTestId("lesson-pause-button").click();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "paused");
  }
}

async function verifyMobileWorld(page, baseUrl, world) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=${world.firstId}`, {
    waitUntil: "domcontentloaded",
  });
  await waitWorldReady(page, world);
  const layout = await page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']")?.getBoundingClientRect();
    const controls = document.querySelector("[data-testid='lesson-control-panel']")?.getBoundingClientRect();
    const rows = [...document.querySelectorAll(".slider-row")].map((row) => {
      const rect = row.getBoundingClientRect();
      return { left: rect.left, right: rect.right };
    });
    return {
      width: innerWidth,
      scrollWidth: document.documentElement.scrollWidth,
      canvas: canvas ? { top: canvas.top, bottom: canvas.bottom, width: canvas.width, height: canvas.height } : null,
      controls: controls ? { top: controls.top, bottom: controls.bottom, width: controls.width, height: controls.height } : null,
      rows,
    };
  });
  assert(layout.scrollWidth <= layout.width + 1, `${world.firstId}: mobile horizontal overflow`);
  assert(layout.canvas?.width > 260 && layout.canvas?.height > 160, `${world.firstId}: mobile canvas invalid`);
  assert(
    layout.controls?.width > 260 && layout.controls?.height > 100,
    `${world.firstId}: mobile controls invalid ${JSON.stringify(layout.controls)}`,
  );
  assert(layout.canvas.bottom <= layout.controls.top + 1, `${world.firstId}: mobile canvas/control overlap`);
  assert(layout.rows.every((row) => row.left >= -1 && row.right <= layout.width + 1), `${world.firstId}: mobile row overflow`);
  const canvas = await readCanvasEvidence(page);
  assert(canvas?.colors >= 4, `${world.firstId}: mobile canvas blank`);
  const status = await page.locator("#screen-run").getAttribute("data-engine-status");
  if (status === "running") {
    await page.getByTestId("lesson-pause-button").click();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "paused");
  }
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const desktopContext = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    const desktopPage = await desktopContext.newPage();
    installFailureCapture(desktopPage, failures);
    for (const world of WORLDS) {
      await verifyDesktopWorld(desktopPage, baseUrl, world);
      await verifyAlias(desktopPage, baseUrl, world);
    }
    await desktopContext.close();

    const mobileContext = await browser.newContext({ viewport: { width: 390, height: 844 }, locale: "ko-KR" });
    const mobilePage = await mobileContext.newPage();
    installFailureCapture(mobilePage, failures);
    for (const world of WORLDS) {
      await verifyMobileWorld(mobilePage, baseUrl, world);
    }
    await mobileContext.close();
    if (failures.length) throw new Error(failures.join("\n"));
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
