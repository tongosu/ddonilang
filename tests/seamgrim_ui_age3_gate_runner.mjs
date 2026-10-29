#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_ui_age3_gate: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const LINE_LESSON_ID = "rep_math_function_line_v1";
const PROJECTILE_LESSON_ID = "rep_phys_projectile_xy_v1";

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
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|detjson|md)$/i.test(pathname)
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

async function waitRunReady(page, lessonId) {
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction((id) => {
    const root = document.querySelector("#screen-run");
    const href = new URL(location.href);
    return href.searchParams.get("lesson") === id
      && String(document.querySelector("#run-ddn-preview")?.value ?? "").trim().length > 50
      && (
        String(root?.dataset?.mainViewRoles ?? "").trim().length > 0
        || Number(root?.dataset?.mainGraphSeriesCount ?? 0) > 0
        || Number(root?.dataset?.mainGraphPointCount ?? 0) > 0
      )
      && String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:");
  }, lessonId, { timeout: 35_000 });
}

async function verifyFourSpacesAndRoleBoundary(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForSelector("#screen-browse .lesson-card", { state: "visible", timeout: 20_000 });
  const evidence = await page.evaluate(() => {
    const tabs = [...document.querySelectorAll("#screen-browse .main-shell-tab[data-shell-space]")];
    return {
      labels: tabs.map((node) => node.textContent?.trim() ?? ""),
      spaces: tabs.map((node) => node.getAttribute("data-shell-space") ?? ""),
      roleSwitchCount: document.querySelectorAll("[data-classroom-mode-switch]").length,
      roleButtonCount: document.querySelectorAll("[data-classroom-mode]").length,
      runtimeSwitchGlobal: typeof window.__STUDIO_CLASSROOM_MODE_SWITCH__,
      openActions: [...document.querySelectorAll("#screen-browse .lesson-card .card-launch-primary")]
        .map((node) => node.textContent?.trim() ?? "")
        .filter(Boolean)
        .slice(0, 3),
      labActions: [...document.querySelectorAll("#screen-browse .lesson-card [data-launch-action='lab']")]
        .map((node) => node.textContent?.trim() ?? "")
        .filter(Boolean)
        .slice(0, 3),
    };
  });
  assert(JSON.stringify(evidence.labels) === JSON.stringify(["배우기", "실험실", "만들기", "배포"]), `4-space labels mismatch: ${JSON.stringify(evidence)}`);
  assert(JSON.stringify(evidence.spaces) === JSON.stringify(["learn", "lab", "create", "publish"]), `4-space ids mismatch: ${JSON.stringify(evidence)}`);
  assert(evidence.roleSwitchCount === 0 && evidence.roleButtonCount === 0, `runtime role toggle DOM exists: ${JSON.stringify(evidence)}`);
  assert(evidence.runtimeSwitchGlobal === "undefined", `runtime role switch global exists: ${evidence.runtimeSwitchGlobal}`);
  assert(evidence.openActions.some((text) => text.includes("수업 열기")), `lesson open action missing: ${JSON.stringify(evidence)}`);
  assert(evidence.labActions.some((text) => text.includes("실험실에서 바꾸기")), `lab action missing: ${JSON.stringify(evidence)}`);
}

async function verifyOpenAndLabAreSeparate(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector(`#screen-browse [data-lesson-id='${PROJECTILE_LESSON_ID}'] .card-launch-primary`, { state: "visible", timeout: 20_000 });
  await page.locator(`#screen-browse [data-lesson-id='${PROJECTILE_LESSON_ID}'] .card-launch-primary`).click();
  await waitRunReady(page, PROJECTILE_LESSON_ID);
  const openEvidence = await page.evaluate(() => ({
    screen: document.querySelector("#screen-run")?.classList?.contains("hidden") ? "not-run" : "run",
    shellSpace: document.querySelector("#screen-run")?.dataset?.shellSpace ?? "",
    activeTab: [...document.querySelectorAll("#screen-run .main-shell-tab.active")]
      .map((node) => String(node?.textContent ?? "").replace(/\s+/g, " ").trim()),
    launchKind: document.querySelector("#screen-run")?.dataset?.launchKind ?? "",
  }));
  assert(openEvidence.screen === "run", `open action did not enter run screen: ${JSON.stringify(openEvidence)}`);
  assert(openEvidence.shellSpace === "learn", `open action shell space leaked: ${JSON.stringify(openEvidence)}`);
  assert(openEvidence.activeTab.join(",") === "배우기", `open action active tab mismatch: ${JSON.stringify(openEvidence)}`);

  await page.locator("#screen-run .main-shell-tab[data-shell-space='learn']").click();
  await page.waitForSelector("#screen-browse:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.locator(`#screen-browse [data-lesson-id='${PROJECTILE_LESSON_ID}'] [data-launch-action='lab']`).click();
  await waitRunReady(page, PROJECTILE_LESSON_ID);
  const labEvidence = await page.evaluate(() => ({
    shellSpace: document.querySelector("#screen-run")?.dataset?.shellSpace ?? "",
    activeTab: [...document.querySelectorAll("#screen-run .main-shell-tab.active")]
      .map((node) => String(node?.textContent ?? "").replace(/\s+/g, " ").trim()),
    launchKind: document.querySelector("#screen-run")?.dataset?.launchKind ?? "",
  }));
  assert(labEvidence.shellSpace === "lab", `lab action shell space mismatch: ${JSON.stringify(labEvidence)}`);
  assert(labEvidence.activeTab.join(",") === "실험실", `lab action active tab mismatch: ${JSON.stringify(labEvidence)}`);
}

async function verifyDescriptionAndPrimaryView(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${LINE_LESSON_ID}`, { waitUntil: "domcontentloaded" });
  await waitRunReady(page, LINE_LESSON_ID);
  await page.waitForFunction(() => {
    const root = document.querySelector("#screen-run");
    return root?.dataset?.primaryViewContract === "1"
      && root?.dataset?.primaryViewHeuristicSelection === "0"
      && root?.dataset?.mainVisualMode === "graph"
      && Number(root?.dataset?.mainGraphSeriesCount ?? 0) >= 1
      && Number(root?.dataset?.mainGraphPointCount ?? 0) >= 3;
  }, null, { timeout: 20_000 });
  const evidence = await page.evaluate(() => {
    const runTabs = [...document.querySelectorAll("#screen-run .run-tab-btn")]
      .filter((node) => node.getBoundingClientRect().width > 0 && node.getBoundingClientRect().height > 0)
      .map((node) => ({
        id: node.id,
        tab: node.getAttribute("data-run-tab") ?? "",
        text: String(node?.textContent ?? "").replace(/\s+/g, " ").trim(),
      }));
    const toolbarDescriptionButtons = [...document.querySelectorAll("#screen-run .run-control-bar button")]
      .filter((node) => String(node?.textContent ?? "").replace(/\s+/g, " ").trim() === "설명");
    const root = document.querySelector("#screen-run");
    return {
      runTabs,
      overlayTabCount: runTabs.filter((row) => row.tab === "overlay" && row.text === "설명").length,
      toolbarDescriptionCount: toolbarDescriptionButtons.length,
      primaryViewContract: root?.dataset?.primaryViewContract ?? "",
      primaryViewSchemaId: root?.dataset?.primaryViewSchemaId ?? "",
      primaryViewHeuristicSelection: root?.dataset?.primaryViewHeuristicSelection ?? "",
      mainVisualMode: root?.dataset?.mainVisualMode ?? "",
      mainViewRoles: root?.dataset?.mainViewRoles ?? "",
      consoleFallbackText: document.querySelector("#run-tab-panel-console .run-observe-console-fallback")?.textContent ?? "",
      graphSegmentCount: Number(root?.dataset?.mainGraphSeriesSegmentCount ?? 0),
      graphSeriesCount: Number(root?.dataset?.mainGraphSeriesCount ?? 0),
      graphPointCount: Number(root?.dataset?.mainGraphPointCount ?? 0),
    };
  });
  assert(evidence.overlayTabCount === 1, `description tab must be single overlay entry: ${JSON.stringify(evidence)}`);
  assert(evidence.toolbarDescriptionCount === 0, `top description duplicate exists: ${JSON.stringify(evidence)}`);
  assert(evidence.primaryViewContract === "1" && evidence.primaryViewSchemaId === "seamgrim.primary_view_rules", `primary view contract missing: ${JSON.stringify(evidence)}`);
  assert(evidence.primaryViewHeuristicSelection === "0", `primary view used heuristic fallback: ${JSON.stringify(evidence)}`);
  assert(evidence.mainVisualMode === "graph", `graph primary view not selected over console fallback: ${JSON.stringify(evidence)}`);
  assert(evidence.graphSeriesCount >= 1 && evidence.graphPointCount >= 3, `graph primary view not rendered: ${JSON.stringify(evidence)}`);
}

async function verifyDdnEditAndRun(page) {
  const beforeHash = await page.locator("#run-mirror-hash").getAttribute("title");
  await page.locator("#screen-run .main-shell-tab[data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 15_000 });
  if (await page.locator("#btn-editor-world-code").isVisible()) {
    await page.locator("#btn-editor-world-code").click();
  }
  await page.waitForSelector("#ddn-textarea:not(.hidden)", { state: "visible", timeout: 10_000 });
  const beforeSource = await page.locator("#ddn-textarea").inputValue();
  const slopeMatch = beforeSource.match(/기울기\s*:\s*수\s*<-\s*\(([-+]?\d+(?:\.\d+)?)\)/u);
  assert(slopeMatch, `line lesson slope assignment missing before edit: ${beforeSource.slice(0, 160)}`);
  const currentSlope = Number(slopeMatch[1]);
  const nextSlope = Math.abs(currentSlope - 3.0) < 0.0001 ? 2.5 : 3.0;
  const edited = beforeSource.replace(/기울기\s*:\s*수\s*<-\s*\(([-+]?\d+(?:\.\d+)?)\)/u, `기울기: 수 <- (${nextSlope.toFixed(1)})`);
  await page.locator("#ddn-textarea").fill(edited);
  await page.locator("#ddn-textarea").dispatchEvent("input");
  await page.locator("#ddn-textarea").dispatchEvent("change");
  await page.waitForFunction((expected) => String(document.querySelector("#ddn-textarea")?.value ?? "").includes(expected), `기울기: 수 <- (${nextSlope.toFixed(1)})`, { timeout: 10_000 });
  const runDisabled = await page.locator("#btn-run-from-editor").isDisabled();
  assert(!runDisabled, "editor run button disabled after valid DDN edit");
  await page.locator("#btn-run-from-editor").click();
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 15_000 });
  await page.waitForFunction((oldHash) => {
    const hash = String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "");
    const status = String(document.querySelector("#screen-run")?.dataset?.engineStatus ?? "");
    return hash.includes("blake3:") && hash !== oldHash && ["done", "running", "paused"].includes(status);
  }, beforeHash, { timeout: 30_000 });
  const after = await page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    return {
      hash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      status: root?.dataset?.engineStatus ?? "",
      source: document.querySelector("#run-ddn-preview")?.value ?? "",
      sliderNames: [...document.querySelectorAll(".slider-row")].map((row) => row.getAttribute("data-control-name") ?? ""),
      warningVisible: !(document.querySelector("#run-error-banner")?.classList?.contains("hidden") ?? true),
    };
  });
  assert(after.hash.includes("blake3:") && after.hash !== beforeHash, `edited DDN did not produce new hash: ${JSON.stringify(after)}`);
  const slopeAfter = after.source.match(/기울기\s*:\s*수\s*<-\s*\(([-+]?\d+(?:\.\d+)?)\)/u);
  assert(slopeAfter && Math.abs(Number(slopeAfter[1]) - nextSlope) < 0.0001, `edited DDN not preserved in run path: expected=${nextSlope.toFixed(1)} ${JSON.stringify(after)}`);
  assert(after.sliderNames.includes("기울기"), `edited DDN did not rebuild controls: ${JSON.stringify(after)}`);
  assert(after.warningVisible === false, `valid edited DDN produced error banner: ${JSON.stringify(after)}`);
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, locale: "ko-KR" });
    installFailureCapture(page, failures);
    await verifyFourSpacesAndRoleBoundary(page, baseUrl);
    await verifyOpenAndLabAreSeparate(page, baseUrl);
    await verifyDescriptionAndPrimaryView(page, baseUrl);
    await verifyDdnEditAndRun(page);
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
