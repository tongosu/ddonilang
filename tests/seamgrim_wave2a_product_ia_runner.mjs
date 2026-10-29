#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave2a_product_ia: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const PROJECTILE_ID = "rep_phys_projectile_xy_v1";

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
      const rawPath = decodeURIComponent(url.pathname === "/" ? PROJECT_URL : url.pathname);
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
      if (!address || typeof address === "string") reject(new Error("failed to bind static server"));
      else resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
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
    if ((pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/")) && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|detjson|md)$/i.test(pathname)) return true;
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

async function waitForCatalog(page) {
  await page.waitForSelector("#screen-browse:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForSelector("#screen-browse .lesson-card", { state: "visible", timeout: 20_000 });
}

async function waitForRun(page) {
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction(() => {
    const root = document.querySelector("#screen-run");
    return root?.dataset?.primaryViewContract === "1" && ["debug-fallback", "space2d"].includes(String(root?.dataset?.mainVisualMode ?? ""));
  }, null, { timeout: 20_000 });
}

async function verifyFourSpaces(page, baseUrl) {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  await waitForCatalog(page);

  const browseTabs = await page.evaluate(() => Array.from(document.querySelectorAll("#screen-browse .browse-tabs button")).map((node) => node.textContent?.trim() ?? ""));
  for (const blocked of ["배포 열기", "붙여넣기 열기", "대표 묶음 복사", "대표 묶음 저장", "교과 만들기"]) {
    assert(!browseTabs.includes(blocked), `browse tab row still contains non-learn command: ${blocked}`);
  }
  assert(browseTabs.includes("대표 교과") && browseTabs.includes("샘플 교과") && browseTabs.includes("전체 검색"), "learn browse categories missing");

  await page.locator("#screen-browse .main-shell-tab[data-shell-space='publish']").click();
  await page.waitForSelector("#screen-publish:not(.hidden) [data-publish-panel]", { state: "visible" });
  const publish = await page.evaluate(() => {
    const tabs = document.querySelector("#screen-browse .browse-tabs");
    const panel = document.querySelector("#screen-publish [data-publish-panel]");
    return {
      shellSpace: document.querySelector("#screen-publish")?.dataset?.shellSpace ?? "",
      publishVisible: !document.querySelector("#screen-publish")?.classList?.contains("hidden"),
      browseVisible: !document.querySelector("#screen-browse")?.classList?.contains("hidden"),
      browsePublishClass: document.querySelector("#screen-browse")?.classList?.contains("browse-space-publish") ?? false,
      active: Array.from(document.querySelectorAll("#screen-publish .main-shell-tab.active")).map((node) => node.textContent?.trim() ?? ""),
      tabsDisplay: tabs ? getComputedStyle(tabs).display : "",
      panelHidden: panel?.classList?.contains("hidden") ?? true,
      buttons: Array.from(document.querySelectorAll("#screen-publish [data-publish-panel] button")).map((node) => node.textContent?.trim() ?? ""),
      boundary: document.querySelector("#screen-publish [data-publish-boundary]")?.textContent?.trim() ?? "",
    };
  });
  assert(publish.shellSpace === "publish", `publish shell space mismatch: ${JSON.stringify(publish)}`);
  assert(publish.publishVisible && !publish.browseVisible, `publish screen isolation mismatch: ${JSON.stringify(publish)}`);
  assert(publish.browsePublishClass === false, `browse publish state leaked: ${JSON.stringify(publish)}`);
  assert(publish.active.length === 1 && publish.active[0] === "배포", `publish active tab mismatch: ${JSON.stringify(publish)}`);
  assert(publish.tabsDisplay !== "none", `learn browse tabs were mutated by publish space: ${publish.tabsDisplay}`);
  assert(!publish.panelHidden, "publish panel hidden");
  for (const required of ["배포 열기", "붙여넣기 열기", "대표 묶음 복사", "대표 묶음 저장"]) {
    assert(publish.buttons.includes(required), `publish action missing: ${required}`);
  }
  assert(publish.boundary.includes("deployment profile"), "publish boundary missing");

  await page.locator("#screen-publish #btn-paste-local-package").click();
  await page.waitForSelector("#screen-publish [data-local-package-paste-panel]:not(.hidden)", { state: "visible", timeout: 5_000 });

  await page.locator("#screen-publish .main-shell-tab[data-shell-space='learn']").click();
  await waitForCatalog(page);
  const learnBack = await page.evaluate(() => ({
    shellSpace: document.querySelector("#screen-browse")?.dataset?.shellSpace ?? "",
    publishHidden: document.querySelector("#screen-publish")?.classList?.contains("hidden") ?? false,
    catalogVisible: getComputedStyle(document.querySelector(".catalog-body")).display,
  }));
  assert(learnBack.shellSpace === "learn", `learn shell space mismatch: ${JSON.stringify(learnBack)}`);
  assert(learnBack.publishHidden === true, "publish screen still visible in learn space");
  assert(learnBack.catalogVisible !== "none", "catalog hidden in learn space");

  await page.locator("#screen-browse .main-shell-tab[data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 10_000 });
  const editorInitial = await page.evaluate(() => {
    const textarea = document.querySelector("#ddn-textarea");
    return {
      mode: document.querySelector("#screen-editor")?.dataset?.editorMode ?? "",
      tabs: Array.from(document.querySelectorAll(".editor-mode-tab")).map((node) => node.textContent?.trim() ?? ""),
      textareaHeight: textarea?.getBoundingClientRect().height ?? 0,
      lessonPlanDisplay: getComputedStyle(document.querySelector("[data-lesson-plan-authoring]")).display,
    };
  });
  assert(editorInitial.mode === "world-code", `editor initial mode mismatch: ${JSON.stringify(editorInitial)}`);
  assert(JSON.stringify(editorInitial.tabs) === JSON.stringify(["세계 코드", "수업 구성"]), `editor tabs mismatch: ${JSON.stringify(editorInitial.tabs)}`);
  assert(editorInitial.textareaHeight >= 420, `DDN editor height too small: ${editorInitial.textareaHeight}`);
  assert(editorInitial.lessonPlanDisplay === "none", "lesson plan panel visible in world-code mode");

  await page.locator("#btn-editor-expand").click();
  const expanded = await page.locator("#screen-editor").getAttribute("data-editor-expanded");
  assert(expanded === "1", `editor expand did not latch: ${expanded}`);

  await page.locator("#btn-editor-lesson-plan").click();
  const lessonPlanMode = await page.evaluate(() => ({
    mode: document.querySelector("#screen-editor")?.dataset?.editorMode ?? "",
    textareaDisplay: getComputedStyle(document.querySelector("#ddn-textarea")).display,
    planDisplay: getComputedStyle(document.querySelector("[data-lesson-plan-authoring]")).display,
  }));
  assert(lessonPlanMode.mode === "lesson-plan", `lesson-plan mode mismatch: ${JSON.stringify(lessonPlanMode)}`);
  assert(lessonPlanMode.textareaDisplay === "none", "DDN textarea visible in lesson-plan mode");
  assert(lessonPlanMode.planDisplay !== "none", "lesson-plan panel hidden");
}

async function verifyRunEntrypoints(page, baseUrl) {
  await page.setViewportSize({ width: 1024, height: 768 });
  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  await waitForCatalog(page);
  await page.locator(`#screen-browse [data-lesson-id='${PROJECTILE_ID}'] .card-launch-primary`).click();
  await waitForRun(page);
  const learn = await page.evaluate(() => ({
    shellSpace: document.querySelector("#screen-run")?.dataset?.shellSpace ?? "",
    active: Array.from(document.querySelectorAll("#screen-run .main-shell-tab.active")).map((node) => node.textContent?.trim() ?? ""),
    body: document.body.innerText,
    roleSwitchCount: document.querySelectorAll("[data-classroom-mode-switch]").length,
    roleButtonCount: document.querySelectorAll("[data-classroom-mode]").length,
    classroomModeAccess: document.querySelector("#screen-run")?.dataset?.classroomModeAccess ?? "",
    legacySwitchPayload: window.__STUDIO_CLASSROOM_MODE_SWITCH__ ?? null,
  }));
  assert(learn.shellSpace === "learn", `lesson open shell mismatch: ${JSON.stringify(learn)}`);
  assert(learn.active.length === 1 && learn.active[0] === "배우기", `lesson open active tab mismatch: ${JSON.stringify(learn)}`);
  assert(learn.roleSwitchCount === 0 && learn.roleButtonCount === 0, `runtime role toggle DOM exists: ${JSON.stringify(learn)}`);
  assert(learn.classroomModeAccess === "" && learn.legacySwitchPayload === null, `runtime role switch state exists: ${JSON.stringify(learn)}`);

  await page.locator("#screen-run .main-shell-tab[data-shell-space='learn']").click();
  await waitForCatalog(page);
  await page.locator(`#screen-browse [data-lesson-id='${PROJECTILE_ID}'] [data-launch-action='lab']`).click();
  await waitForRun(page);
  const lab = await page.evaluate(() => ({
    shellSpace: document.querySelector("#screen-run")?.dataset?.shellSpace ?? "",
    active: Array.from(document.querySelectorAll("#screen-run .main-shell-tab.active")).map((node) => node.textContent?.trim() ?? ""),
    controlsVisible: Boolean(document.querySelector("[data-testid='lesson-control-panel']")),
    roleSwitchCount: document.querySelectorAll("[data-classroom-mode-switch]").length,
    roleButtonCount: document.querySelectorAll("[data-classroom-mode]").length,
    classroomModeAccess: document.querySelector("#screen-run")?.dataset?.classroomModeAccess ?? "",
  }));
  assert(lab.shellSpace === "lab", `lab shell mismatch: ${JSON.stringify(lab)}`);
  assert(lab.active.length === 1 && lab.active[0] === "실험실", `lab active tab mismatch: ${JSON.stringify(lab)}`);
  assert(lab.controlsVisible, "lab control panel missing");
  assert(lab.roleSwitchCount === 0 && lab.roleButtonCount === 0 && lab.classroomModeAccess === "", `lab runtime role toggle exists: ${JSON.stringify(lab)}`);
}

async function main() {
  const root = path.resolve(".");
  const { server, baseUrl } = await createServer(root);
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage();
    installFailureCapture(page, failures);
    await verifyFourSpaces(page, baseUrl);
    await verifyRunEntrypoints(page, baseUrl);
    await page.close();
    assert(failures.length === 0, failures.join("\n"));
    process.stdout.write(`${OK}\n`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
