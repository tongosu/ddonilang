#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_world_diff_review: ok";

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

async function waitReady(page) {
  await page.waitForSelector("#screen-run[data-world-diff='candidate_preview'][data-world-diff-authority='none']", { state: "visible" });
  await page.waitForSelector("[data-testid='world-diff-review']", { state: "visible" });
  await page.waitForSelector("input[type='number'][data-control-name='중력가속도']", { state: "visible" });
}

async function readModel(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const panel = document.querySelector("[data-testid='world-diff-review']");
    const tab = panel?.closest(".run-tab-panel");
    const rect = panel?.getBoundingClientRect();
    return {
      status: root?.dataset?.worldDiff ?? "",
      authority: root?.dataset?.worldDiffAuthority ?? "",
      decision: root?.dataset?.worldDiffDecision ?? "",
      panelDecision: panel?.dataset?.worldDiffDecision ?? "",
      text: panel?.textContent?.trim() ?? "",
      before: panel?.querySelector("[data-world-diff-before]")?.textContent?.trim() ?? "",
      after: panel?.querySelector("[data-world-diff-after]")?.textContent?.trim() ?? "",
      tabScrollHeight: tab?.scrollHeight ?? 0,
      tabClientHeight: tab?.clientHeight ?? 0,
      panel: rect ? { width: rect.width, height: rect.height, left: rect.left, right: rect.right } : null,
      scrollWidth: document.documentElement.scrollWidth,
      viewportWidth: innerWidth,
    };
  });
}

async function verifyInitial(page, { mobile = false } = {}) {
  await waitReady(page);
  const model = await readModel(page);
  assert(model.status === "candidate_preview", `status mismatch: ${model.status}`);
  assert(model.authority === "none", `authority mismatch: ${model.authority}`);
  assert(model.decision === "pending" && model.panelDecision === "pending", `initial decision mismatch: ${model.decision}/${model.panelDecision}`);
  assert(model.before === "9.8" && model.after === "4.9", `value diff mismatch: ${model.before}/${model.after}`);
  assert(model.text.includes("후보 · 권한 없음"), "no-authority label missing");
  assert(model.text.includes("적용과 버리기는 사람이 결정합니다"), "human boundary missing");
  assert(model.text.includes("같은 240마디에서 실행 결과가 달라집니다"), "execution preview missing");
  assert(!model.text.includes("blake3:") && !model.text.includes("sha256:"), "hashes must stay hidden in learning UI");
  assert(model.tabScrollHeight <= model.tabClientHeight + 1, `nested review scroll: ${model.tabScrollHeight}/${model.tabClientHeight}`);
  assert(model.scrollWidth <= model.viewportWidth + 1, `horizontal overflow: ${model.scrollWidth}/${model.viewportWidth}`);
  assert(model.panel && model.panel.width > (mobile ? 300 : 420) && model.panel.height > 220, `panel size mismatch: ${JSON.stringify(model.panel)}`);
  assert(await page.locator("input[type='number'][data-control-name='중력가속도']").inputValue() === "9.8", "candidate applied before confirmation");
}

async function verifyReject(page, baseUrl) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=projectile`, { waitUntil: "domcontentloaded" });
  await verifyInitial(page);
  await page.getByTestId("world-diff-reject").click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.worldDiffDecision === "rejected");
  assert(await page.locator("input[type='number'][data-control-name='중력가속도']").inputValue() === "9.8", "reject changed control value");
  const source = await page.locator("#run-ddn-preview").inputValue();
  assert(source.includes("중력가속도: 수 <- (9.8)"), "reject changed product source");
  const event = await page.evaluate(() => window.__SEAMGRIM_WORLD_DIFF_REVIEW_EVENT__);
  assert(event?.decision === "rejected" && event?.applied_to_session === false && event?.file_write === false, `reject event mismatch: ${JSON.stringify(event)}`);
}

async function verifyApprove(page, baseUrl, candidateHash) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=projectile`, { waitUntil: "domcontentloaded" });
  await verifyInitial(page);
  await page.evaluate(() => {
    const select = document.querySelector("#select-dock-speed");
    if (!(select instanceof HTMLSelectElement)) return;
    select.value = "2";
    select.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "done", null, { timeout: 35_000 });
  const baselineBrowserHash = await page.locator("#run-mirror-hash").getAttribute("title");
  assert(baselineBrowserHash?.includes("blake3:"), `baseline browser hash missing: ${baselineBrowserHash}`);
  await page.getByTestId("world-diff-approve").click();
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.worldDiffDecision === "approved");
  assert(await page.locator("input[type='number'][data-control-name='중력가속도']").inputValue() === "4.9", "approved value not applied");
  const source = await page.locator("#run-ddn-preview").inputValue();
  assert(source.includes("중력가속도: 수 <- (4.9)"), "approved candidate missing from runtime source");
  const event = await page.evaluate(() => window.__SEAMGRIM_WORLD_DIFF_REVIEW_EVENT__);
  assert(event?.decision === "approved" && event?.applied_to_session === true && event?.authority_after_decision === "local_session_only", `approve event mismatch: ${JSON.stringify(event)}`);
  await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.engineStatus === "done", null, { timeout: 35_000 });
  const candidateBrowserHash = await page.locator("#run-mirror-hash").getAttribute("title");
  assert(candidateBrowserHash?.includes("blake3:"), `candidate browser hash missing: ${candidateBrowserHash}`);
  assert(candidateBrowserHash !== baselineBrowserHash, "approved candidate did not change browser result hash");
  assert(candidateHash.startsWith("blake3:"), "CLI candidate evidence hash missing");
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const artifact = JSON.parse(await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "lessons", "rep_phys_projectile_xy_v1", "world_diff_gravity_half.detjson"), "utf8"));
  const candidateHash = artifact?.execution_preview?.candidate?.state_hash ?? "";
  assert(candidateHash.startsWith("blake3:"), "candidate execution hash missing");
  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const desktop = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    const rejectPage = await desktop.newPage();
    captureFailures(rejectPage, failures);
    await verifyReject(rejectPage, baseUrl);
    await rejectPage.close();
    const approvePage = await desktop.newPage();
    captureFailures(approvePage, failures);
    await verifyApprove(approvePage, baseUrl, candidateHash);
    await desktop.close();

    const mobile = await browser.newContext({ viewport: { width: 390, height: 844 }, locale: "ko-KR" });
    const mobilePage = await mobile.newPage();
    captureFailures(mobilePage, failures);
    await mobilePage.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=projectile`, { waitUntil: "domcontentloaded" });
    await verifyInitial(mobilePage, { mobile: true });
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
