#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_learning_spine_index: ok";
const EXPECTED_TITLES = [
  "값·식·단위",
  "상태와 마디",
  "조건·반복·사건",
  "순수 함수·함수값·자료형·차림",
  "모듬·가지·테스트·진단",
  "관계·모델·논리·증명",
  "AI 협업과 뜻 단위 diff",
  "Python·Rust·JS·Modelica 대응 교과",
];
const LANDED_WORLDS = [
  { stageId: "LS01", lessonId: "rep_math_function_line_v1", title: "일차함수 그래프" },
  { stageId: "LS02", lessonId: "rep_phys_projectile_xy_v1", title: "포물선 운동" },
  { stageId: "LS02", lessonId: "rep_physics_velocity_history_v1", title: "속도 기록 그래프" },
  { stageId: "LS03", lessonId: "rep_ddonirang_vol4_event_dispatch_v1", title: "사건이 상태를 바꾸는 과정" },
  { stageId: "LS03", lessonId: "rep_ddonirang_vol4_state_transition_v1", title: "상태 전이와 체력 변화" },
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
  if (file.endsWith(".toml") || file.endsWith(".ddn") || file.endsWith(".md")) return "text/plain; charset=utf-8";
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
      if (!address || typeof address === "string") reject(new Error("failed to bind server"));
      else resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
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

async function readLayout(page) {
  return page.evaluate(() => {
    const panel = document.querySelector("#learning-spine-panel");
    const body = panel?.querySelector(".learning-spine-body");
    const detail = panel?.querySelector(".learning-spine-stage-detail");
    const rect = panel?.getBoundingClientRect();
    return {
      schema: panel?.dataset?.schema ?? "",
      hidden: panel?.classList?.contains("hidden") ?? true,
      activeStage: detail?.dataset?.learningStageActive ?? "",
      panel: rect ? { width: rect.width, height: rect.height, left: rect.left, right: rect.right } : null,
      bodyScrollWidth: body?.scrollWidth ?? 0,
      bodyClientWidth: body?.clientWidth ?? 0,
      documentScrollWidth: document.documentElement.scrollWidth,
      viewportWidth: innerWidth,
      activeElementId: document.activeElement?.id ?? "",
      activeElementClass: document.activeElement?.className ?? "",
    };
  });
}

async function verifyIndex(page, baseUrl, { mobile = false } = {}) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?learning_spine=1`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#learning-spine-panel:not(.hidden) [data-learning-stage-id='LS01']", { state: "visible" });
  const stageButtons = page.locator("[data-learning-stage-id]");
  assert(await stageButtons.count() === 8, `stage count mismatch: ${await stageButtons.count()}`);
  const titles = await stageButtons.locator(".learning-spine-stage-button-title").allTextContents();
  assert(JSON.stringify(titles) === JSON.stringify(EXPECTED_TITLES), `stage order mismatch: ${JSON.stringify(titles)}`);

  let layout = await readLayout(page);
  assert(layout.schema === "ddn.learning_spine_index.v1", `schema mismatch: ${layout.schema}`);
  assert(!layout.hidden && layout.activeStage === "LS01", `initial state mismatch: ${JSON.stringify(layout)}`);
  assert(String(layout.activeElementClass).includes("learning-spine-close"), `open focus mismatch: ${JSON.stringify(layout)}`);
  assert(layout.panel && layout.panel.width > (mobile ? 350 : 900), `panel width mismatch: ${JSON.stringify(layout.panel)}`);
  assert(layout.panel && layout.panel.height > (mobile ? 560 : 600), `panel height mismatch: ${JSON.stringify(layout.panel)}`);
  assert(layout.bodyScrollWidth <= layout.bodyClientWidth + 1, `body overflow: ${layout.bodyScrollWidth}/${layout.bodyClientWidth}`);
  assert(layout.documentScrollWidth <= layout.viewportWidth + 1, `document overflow: ${layout.documentScrollWidth}/${layout.viewportWidth}`);
  assert((await page.locator(".learning-spine-stage-status").textContent())?.includes("지금 배울 수 있음"), "LS01 status missing");

  await page.locator("[data-learning-stage-id='LS04']").click();
  await page.waitForFunction(() => document.querySelector(".learning-spine-stage-detail")?.dataset?.learningStageActive === "LS04");
  const stage4Text = await page.locator(".learning-spine-stage-detail").textContent();
  assert(stage4Text?.includes("저장·반환되는 일급 함수 전체 계약"), "LS04 open boundary missing");
  assert(stage4Text?.includes("기반 있음 · 일부 열림"), "LS04 mixed status missing");

  await page.locator("[data-learning-stage-id='LS08']").click();
  await page.waitForFunction(() => document.querySelector(".learning-spine-stage-detail")?.dataset?.learningStageActive === "LS08");
  const stage8Text = await page.locator(".learning-spine-stage-detail").textContent();
  assert(stage8Text?.includes("설계 열림"), "LS08 gap status missing");
  for (const label of ["Python", "Rust", "JavaScript", "Modelica"]) {
    assert(stage8Text?.includes(label), `LS08 correspondence missing: ${label}`);
  }
  assert(stage8Text?.includes("full DAE·일반 해석기 완성은 주장하지 않음"), "Modelica non-goal missing");
  assert(stage8Text?.includes("설계 제안 · 승인 전"), "proposal boundary group missing");
  assert(stage8Text?.includes("연결 색인에만 반영") && stage8Text?.includes("승격 결정 대기"), "state-axis boundary missing");

  layout = await readLayout(page);
  assert(layout.documentScrollWidth <= layout.viewportWidth + 1, `LS08 document overflow: ${layout.documentScrollWidth}/${layout.viewportWidth}`);
  assert(layout.bodyScrollWidth <= layout.bodyClientWidth + 1, `LS08 body overflow: ${layout.bodyScrollWidth}/${layout.bodyClientWidth}`);

  await page.locator(".learning-spine-close").click();
  await page.waitForSelector("#learning-spine-panel", { state: "hidden" });
  layout = await readLayout(page);
  assert(layout.activeElementId === "btn-learning-spine", `close focus mismatch: ${JSON.stringify(layout)}`);
  await page.locator("#btn-learning-spine").click();
  await page.waitForSelector("#learning-spine-panel:not(.hidden)", { state: "visible" });
  layout = await readLayout(page);
  assert(String(layout.activeElementClass).includes("learning-spine-close"), `reopen focus mismatch: ${JSON.stringify(layout)}`);
}

async function verifyLandedWorlds(page, baseUrl) {
  for (const world of LANDED_WORLDS) {
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?learning_spine=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector(`#learning-spine-panel:not(.hidden) [data-learning-stage-id='${world.stageId}']`, { state: "visible" });
    if (world.stageId !== "LS01") {
      await page.locator(`[data-learning-stage-id='${world.stageId}']`).click();
    }
    await page.waitForSelector(`[data-learning-world-open='${world.lessonId}']`, { state: "visible" });
    await page.locator(`[data-learning-world-open='${world.lessonId}']`).click();
    await page.waitForSelector("#screen-run:not(.hidden) #run-lesson-title", { state: "visible", timeout: 35_000 });
    try {
      await page.waitForFunction(
        () => {
          const status = document.querySelector("#screen-run")?.dataset?.engineStatus ?? "";
          const madi = document.querySelector("#run-madi-status")?.textContent?.trim() ?? "";
          return status === "done" || (status === "running" && !/^0(?:\/|\D)/u.test(madi));
        },
        null,
        { timeout: 35_000 },
      );
    } catch (error) {
      const runtime = await page.evaluate(() => ({
        engineStatus: document.querySelector("#screen-run")?.dataset?.engineStatus ?? "",
        title: document.querySelector("#run-lesson-title")?.textContent?.trim() ?? "",
        madi: document.querySelector("#run-madi-status")?.textContent?.trim() ?? "",
        error: document.querySelector("#run-error-message")?.textContent?.trim() ?? "",
      }));
      throw new Error(`landed world did not execute: ${world.lessonId} ${JSON.stringify(runtime)} ${error?.message ?? error}`);
    }
    const title = (await page.locator("#run-lesson-title").textContent())?.trim();
    assert(title === world.title, `landed world mismatch: ${world.lessonId} -> ${title}`);
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
    await verifyIndex(desktopPage, baseUrl);
    await verifyLandedWorlds(desktopPage, baseUrl);
    await desktop.close();

    const mobile = await browser.newContext({ viewport: { width: 390, height: 844 }, locale: "ko-KR" });
    const mobilePage = await mobile.newPage();
    captureFailures(mobilePage, failures);
    await verifyIndex(mobilePage, baseUrl, { mobile: true });
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
