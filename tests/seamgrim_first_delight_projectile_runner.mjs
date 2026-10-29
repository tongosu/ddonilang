#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_first_delight_projectile: ok";
const LESSON_ID = "rep_phys_projectile_xy_v1";

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

async function waitForFirstDelight(page) {
  await page.waitForSelector("#screen-run[data-first-delight='projectile']", { state: "visible" });
  try {
    await page.waitForFunction(() => {
      const root = document.querySelector("#screen-run");
      const space2d = window.__SEAMGRIM_MAIN_VISUAL__?.space2d;
      return String(root?.dataset?.mainVisualMode ?? "") === "space2d"
        && String(root?.dataset?.canonicalSpace2dPayloadStatus ?? "") === "available"
        && Array.isArray(space2d?.drawlist)
        && space2d.drawlist.length >= 5;
    }, null, { timeout: 20_000 });
  } catch (err) {
    const evidence = await page.evaluate(() => {
      const root = document.querySelector("#screen-run");
      return {
        href: location.href,
        rootDataset: { ...(root?.dataset ?? {}) },
        sourcePrefix: String(document.querySelector("#run-ddn-preview")?.value ?? "").slice(0, 240),
        mirrorHash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
        errorText: document.querySelector("[data-testid='lesson-error-panel']")?.textContent?.trim() ?? "",
        mainVisual: window.__SEAMGRIM_MAIN_VISUAL__ ?? null,
      };
    });
    throw new Error(`first delight readiness timeout: ${JSON.stringify(evidence)}\n${String(err?.message ?? err)}`);
  }
  await page.waitForSelector("[data-testid='lesson-control-panel']", { state: "visible" });
}

async function readMadi(page) {
  return page.evaluate(() => {
    const text = document.querySelector("[data-testid='lesson-madi-status']")?.textContent ?? "";
    const match = text.match(/(\d+)\s*\/\s*(\d+)/u);
    return match ? { current: Number(match[1]), max: Number(match[2]) } : null;
  });
}

async function readCanvasEvidence(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']");
    if (!(canvas instanceof HTMLCanvasElement)) return null;
    const rect = canvas.getBoundingClientRect();
    const ctx = canvas.getContext("2d");
    if (!ctx || canvas.width <= 0 || canvas.height <= 0) {
      return { width: rect.width, height: rect.height, pixelWidth: canvas.width, pixelHeight: canvas.height, colors: 0 };
    }
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    const colors = new Set();
    const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 4000) * 4);
    for (let i = 0; i < data.length; i += stride) {
      colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
      if (colors.size > 12) break;
    }
    return {
      width: rect.width,
      height: rect.height,
      pixelWidth: canvas.width,
      pixelHeight: canvas.height,
      colors: colors.size,
    };
  });
}

async function verifyDesktop(page, baseUrl) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=projectile`, {
    waitUntil: "domcontentloaded",
  });
  await waitForFirstDelight(page);

  const route = new URL(page.url());
  assert(route.searchParams.get("first_delight") === "projectile", "first_delight route marker missing");
  assert(route.searchParams.get("tab") === "studio", "first delight did not canonicalize to studio tab");
  assert(route.searchParams.get("lesson") === LESSON_ID, `first delight lesson mismatch: ${route.searchParams.get("lesson")}`);

  const model = await page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const rail = document.querySelector("[data-run-preset-rail]");
    const rows = [...document.querySelectorAll(".slider-row")];
    return {
      launchKind: root?.dataset?.launchKind ?? "",
      launchLabel: rail?.querySelector("[data-run-preset-launch-kind]")?.textContent?.trim() ?? "",
      mode: root?.dataset?.mainVisualMode ?? "",
      roles: String(root?.dataset?.mainViewRoles ?? "").split(",").filter(Boolean),
      structuredSpace2dStatus: root?.dataset?.canonicalSpace2dPayloadStatus ?? "",
      drawlist: window.__SEAMGRIM_MAIN_VISUAL__?.space2d?.drawlist ?? [],
      viewSource: window.__SEAMGRIM_MAIN_VISUAL__?.space2d?.source ?? "",
      controls: rows.map((row) => ({
        name: row.dataset.controlName ?? "",
        label: row.querySelector("label")?.textContent?.trim() ?? "",
      })),
      errorHidden: document.querySelector("[data-testid='lesson-error-panel']")?.classList.contains("hidden") ?? false,
      horizontalOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    };
  });
  assert(model.launchKind === "first_delight", `launch kind mismatch: ${model.launchKind}`);
  assert(model.launchLabel === "첫 세계", `launch label mismatch: ${model.launchLabel}`);
  assert(model.mode === "space2d", `main visual mismatch: ${model.mode}`);
  assert(model.structuredSpace2dStatus === "available", `structured space2d unavailable: ${model.structuredSpace2dStatus}`);
  assert(model.drawlist.length >= 5, `WASM draw list missing: ${JSON.stringify(model.drawlist)}`);
  assert(model.viewSource.includes("authoritative_structured"), `WASM view source missing: ${model.viewSource}`);
  assert(model.errorHidden, "first delight error panel should stay hidden");
  assert(model.horizontalOverflow <= 1, `desktop horizontal overflow: ${model.horizontalOverflow}`);
  assert(JSON.stringify(model.controls) === JSON.stringify([
    { name: "초기속도x", label: "가로 속도" },
    { name: "초기속도y", label: "세로 속도" },
    { name: "중력가속도", label: "중력" },
  ]), `control surface mismatch: ${JSON.stringify(model.controls)}`);

  const canvas = await readCanvasEvidence(page);
  assert(canvas && canvas.width > 200 && canvas.height > 160, `canvas size invalid: ${JSON.stringify(canvas)}`);
  assert(canvas.colors >= 4, `canvas appears blank: ${JSON.stringify(canvas)}`);

  try {
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "running");
  } catch (err) {
    const evidence = await page.evaluate(() => {
      const root = document.querySelector("#screen-run");
      return {
        rootDataset: { ...(root?.dataset ?? {}) },
        madiText: document.querySelector("[data-testid='lesson-madi-status']")?.textContent?.trim() ?? "",
        sourcePrefix: String(document.querySelector("#run-ddn-preview")?.value ?? "").slice(0, 320),
        errorText: document.querySelector("[data-testid='lesson-error-panel']")?.textContent?.trim() ?? "",
        mirrorHash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      };
    });
    throw new Error(`first delight never reached a live running state: ${JSON.stringify(evidence)}\n${String(err?.message ?? err)}`);
  }
  await page.getByTestId("lesson-pause-button").click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "paused");
  const beforeStep = await readMadi(page);
  assert(beforeStep, "madi status missing before step");
  await page.getByTestId("lesson-step-button").click();
  await page.waitForFunction((before) => {
    const root = document.querySelector("#screen-run");
    const text = document.querySelector("[data-testid='lesson-madi-status']")?.textContent ?? "";
    const match = text.match(/(\d+)\s*\//u);
    return root?.dataset?.engineStatus === "paused" && match && Number(match[1]) === before + 1;
  }, beforeStep.current);
  const beforeControlDrawlist = await page.evaluate(() => JSON.stringify(
    window.__SEAMGRIM_MAIN_VISUAL__?.space2d?.drawlist ?? [],
  ));

  const gravityInput = page.locator("input[type='number'][data-control-name='중력가속도']");
  await gravityInput.fill("12.5");
  await gravityInput.dispatchEvent("change");
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "running");
  await page.evaluate(() => {
    const select = document.querySelector("#select-dock-speed");
    if (!(select instanceof HTMLSelectElement)) return;
    select.value = "2";
    select.dispatchEvent(new Event("change", { bubbles: true }));
  });
  const source = await page.locator("#run-ddn-preview").inputValue();
  assert(source.includes("중력가속도: 수 := (12.5)"), "gravity control did not update V25 product DDN input");
  assert(!source.includes("중력가속도: 수 <-"), "gravity control regressed the V25 definition surface");

  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "done", null, { timeout: 35_000 });
  const changedControlDrawlist = await page.evaluate(() => JSON.stringify(
    window.__SEAMGRIM_MAIN_VISUAL__?.space2d?.drawlist ?? [],
  ));
  assert(changedControlDrawlist !== beforeControlDrawlist, "gravity control did not change the actual WASM bogae drawlist");
  const firstHash = await page.locator("#run-mirror-hash").getAttribute("title");
  assert(firstHash?.includes("blake3:"), `final state hash missing: ${firstHash}`);

  await page.getByTestId("lesson-reset-button").click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "idle");
  const resetMadi = await readMadi(page);
  assert(resetMadi?.current === 0, `reset did not return to madi 0: ${JSON.stringify(resetMadi)}`);
  await page.getByTestId("lesson-run-button").click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "done", null, { timeout: 35_000 });
  const secondHash = await page.locator("#run-mirror-hash").getAttribute("title");
  assert(secondHash === firstHash, `determinism mismatch: ${firstHash} != ${secondHash}`);
}

async function verifyAlias(page, baseUrl) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&lesson=physics_projectile`, {
    waitUntil: "domcontentloaded",
  });
  await page.waitForSelector("#screen-run", { state: "visible" });
  await page.waitForFunction(() => new URL(location.href).searchParams.get("lesson") === "rep_phys_projectile_xy_v1");
  assert((await page.locator("#run-lesson-title").textContent())?.trim() === "포물선 운동", "legacy alias did not open active representative lesson");
}

async function verifyMobile(page, baseUrl) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=projectile`, {
    waitUntil: "domcontentloaded",
  });
  await waitForFirstDelight(page);
  const layout = await page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']")?.getBoundingClientRect();
    const controls = document.querySelector("[data-testid='lesson-control-panel']")?.getBoundingClientRect();
    const visual = document.querySelector(".run-visual-column")?.getBoundingClientRect();
    const dotbogi = document.querySelector(".dotbogi-panel.subpanel")?.getBoundingClientRect();
    const tabPanel = document.querySelector("#subpanel-tab-panel")?.getBoundingClientRect();
    const sliderList = document.querySelector("#slider-list")?.getBoundingClientRect();
    const styleOf = (selector) => {
      const el = document.querySelector(selector);
      if (!(el instanceof HTMLElement)) return null;
      const style = getComputedStyle(el);
      return {
        display: style.display,
        position: style.position,
        width: style.width,
        minWidth: style.minWidth,
        flexDirection: style.flexDirection,
        flexWrap: style.flexWrap,
        gridColumn: style.gridColumn,
        gridRow: style.gridRow,
      };
    };
    const rect = (row) => row ? { left: row.left, right: row.right, top: row.top, bottom: row.bottom, width: row.width, height: row.height } : null;
    const rows = [...document.querySelectorAll(".slider-row")].map((row) => {
      const rect = row.getBoundingClientRect();
      return { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom };
    });
    return {
      viewport: { width: innerWidth, height: innerHeight },
      scrollWidth: document.documentElement.scrollWidth,
      canvas: rect(canvas),
      controls: rect(controls),
      visual: rect(visual),
      dotbogi: rect(dotbogi),
      tabPanel: rect(tabPanel),
      sliderList: rect(sliderList),
      styles: {
        visual: styleOf(".run-visual-column"),
        dotbogi: styleOf(".dotbogi-panel.subpanel"),
        tabPanel: styleOf("#subpanel-tab-panel"),
      },
      rows,
    };
  });
  assert(layout.scrollWidth <= layout.viewport.width + 1, `mobile horizontal overflow: ${JSON.stringify(layout)}`);
  assert(layout.canvas && layout.canvas.width > 260 && layout.canvas.height > 160, `mobile canvas invalid: ${JSON.stringify(layout.canvas)}`);
  assert(layout.controls && layout.controls.width > 260 && layout.controls.height > 100, `mobile controls invalid: ${JSON.stringify(layout)}`);
  assert(layout.canvas.bottom <= layout.controls.top + 1, `mobile canvas/control overlap: ${JSON.stringify(layout)}`);
  assert(layout.rows.every((row) => row.left >= -1 && row.right <= layout.viewport.width + 1), `mobile control row overflow: ${JSON.stringify(layout)}`);
  const canvas = await readCanvasEvidence(page);
  assert(canvas?.colors >= 4, `mobile canvas appears blank: ${JSON.stringify(canvas)}`);
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    const executablePath = String(process.env.CHROME_EXECUTABLE ?? "").trim() || undefined;
    browser = await chromium.launch({ headless: true, executablePath });
    const desktopContext = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    const desktopPage = await desktopContext.newPage();
    installFailureCapture(desktopPage, failures);
    await verifyDesktop(desktopPage, baseUrl);
    await verifyAlias(desktopPage, baseUrl);
    await desktopContext.close();

    const mobileContext = await browser.newContext({ viewport: { width: 390, height: 844 }, locale: "ko-KR" });
    const mobilePage = await mobileContext.newPage();
    installFailureCapture(mobilePage, failures);
    await verifyMobile(mobilePage, baseUrl);
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
