#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_projectile_representation_pair: ok";

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
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store", "access-control-allow-origin": "*" });
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

function allowed404(urlText) {
  try {
    const pathname = new URL(urlText).pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if ((pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/")) && /\/(?:graph|table|space2d)\.json$/i.test(pathname)) return true;
    if ((pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/")) && /\/(?:text\.md|maegim_control\.json)$/i.test(pathname)) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function captureFailures(page, failures) {
  page.on("console", (msg) => {
    if (msg.type() !== "error" || String(msg.text() ?? "").includes("Failed to load resource")) return;
    failures.push(`console error: ${msg.text()}`);
  });
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() >= 400 && !(res.status() === 404 && allowed404(res.url()))) failures.push(`response ${res.status()}: ${res.url()}`);
  });
}

async function waitReady(page) {
  await page.waitForSelector("#screen-run[data-first-delight='projectile']", { state: "visible" });
  await page.waitForFunction(() => {
    const root = document.querySelector("#screen-run");
    const panel = document.querySelector("[data-testid='lesson-representation-pair']");
    return Number(root?.dataset?.mainTrajectorySegmentCount ?? 0) >= 3
      && root?.dataset?.representationPair === "teaching_pair"
      && panel
      && !panel.classList.contains("hidden");
  }, null, { timeout: 20_000 });
}

async function verifyPair(page, { mobile = false } = {}) {
  await waitReady(page);
  const model = await page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const panel = document.querySelector("[data-testid='lesson-representation-pair']");
    const panelRect = panel?.getBoundingClientRect();
    const grid = document.querySelector(".run-representation-pair-grid");
    return {
      pair: root?.dataset?.representationPair ?? "",
      equivalence: root?.dataset?.representationEquivalence ?? "",
      declarativeRuntime: root?.dataset?.declarativeRuntime ?? "",
      classText: document.querySelector("#run-representation-pair-class")?.textContent?.trim() ?? "",
      leftStatus: document.querySelector("#run-representation-left-status")?.textContent?.trim() ?? "",
      rightStatus: document.querySelector("#run-representation-right-status")?.textContent?.trim() ?? "",
      leftSource: document.querySelector("#run-representation-left-source")?.textContent?.trim() ?? "",
      rightSource: document.querySelector("#run-representation-right-source")?.textContent?.trim() ?? "",
      plan: document.querySelector("#run-representation-plan-summary")?.textContent?.trim() ?? "",
      equivalenceText: document.querySelector("#run-representation-equivalence")?.textContent?.trim() ?? "",
      panel: panelRect ? { left: panelRect.left, right: panelRect.right, width: panelRect.width, height: panelRect.height } : null,
      gridColumns: grid ? getComputedStyle(grid).gridTemplateColumns.split(/\s+/).filter(Boolean).length : 0,
      tabPanelScrollHeight: panel?.closest(".run-tab-panel")?.scrollHeight ?? 0,
      tabPanelClientHeight: panel?.closest(".run-tab-panel")?.clientHeight ?? 0,
      viewportWidth: innerWidth,
      scrollWidth: document.documentElement.scrollWidth,
    };
  });
  assert(model.pair === "teaching_pair", `pair class mismatch: ${model.pair}`);
  assert(model.equivalence === "not_asserted", `equivalence mismatch: ${model.equivalence}`);
  assert(model.declarativeRuntime === "not_landed", `declarative runtime mismatch: ${model.declarativeRuntime}`);
  assert(model.classText === "가르침 짝", `pair label mismatch: ${model.classText}`);
  assert(model.leftStatus === "실행됨" && model.rightStatus === "풀이계획만", `status mismatch: ${model.leftStatus}/${model.rightStatus}`);
  assert(model.leftSource.includes("x <- x + vx * dt"), "explicit teaching snippet missing");
  assert(model.rightSource.includes("위치_x' =:= 속도_x") && model.rightSource.includes("속도_y' =:= 중력_y"), "declarative teaching snippet missing");
  assert(model.plan === "S2 · ode_explicit_euler_v1 · 관계 4개", `solveplan summary mismatch: ${model.plan}`);
  assert(model.equivalenceText === "관측 동등성: 미검증", `equivalence text mismatch: ${model.equivalenceText}`);
  assert(model.scrollWidth <= model.viewportWidth + 1, `horizontal overflow: ${model.scrollWidth}/${model.viewportWidth}`);
  assert(model.panel && model.panel.width > 280 && model.panel.height > 180, `panel size invalid: ${JSON.stringify(model.panel)}`);
  assert(mobile ? model.gridColumns === 1 : model.gridColumns === 2, `grid columns mismatch: ${model.gridColumns}`);
  if (mobile) {
    assert(model.tabPanelScrollHeight <= model.tabPanelClientHeight + 1, `nested comparison scroll remains: ${model.tabPanelScrollHeight}/${model.tabPanelClientHeight}`);
  }
  const status = await page.locator("#screen-run").getAttribute("data-engine-status");
  if (status === "running") {
    await page.getByTestId("lesson-pause-button").click();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "paused");
  }
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const desktop = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    const desktopPage = await desktop.newPage();
    captureFailures(desktopPage, failures);
    await desktopPage.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=projectile`, { waitUntil: "domcontentloaded" });
    await verifyPair(desktopPage);
    await desktop.close();

    const mobile = await browser.newContext({ viewport: { width: 390, height: 844 }, locale: "ko-KR" });
    const mobilePage = await mobile.newPage();
    captureFailures(mobilePage, failures);
    await mobilePage.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=projectile`, { waitUntil: "domcontentloaded" });
    await verifyPair(mobilePage, { mobile: true });
    await mobile.close();
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
