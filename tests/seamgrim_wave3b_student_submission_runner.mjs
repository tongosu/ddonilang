#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3b_student_submission: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
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
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/") || pathname.startsWith("/lessons_rewrite_v1/"))
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

async function waitVisible(page, selector) {
  await page.waitForFunction((sel) => {
    const node = document.querySelector(sel);
    return node && !String(node.className ?? "").split(/\s+/).includes("hidden");
  }, selector, { timeout: 30_000 });
}

async function pastePayload(page, payload) {
  const text = JSON.stringify(payload, null, 2);
  await page.evaluate((value) => {
    const node = document.querySelector("#local-package-paste-text");
    if (!node) throw new Error("paste textarea missing");
    node.value = value;
    node.dispatchEvent(new Event("input", { bubbles: true }));
  }, text);
  await page.locator("#btn-local-package-paste-open").click();
}

async function openPublishForPaste(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=publish&catalog_mode=full`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-publish");
  await page.locator("#btn-paste-local-package").click();
  await page.waitForSelector("#local-package-paste-text", { state: "visible", timeout: 20_000 });
}

async function exportLessonBundle(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&catalog_mode=full&lesson=${encodeURIComponent(LESSON_ID)}`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:"), null, { timeout: 30_000 });
  const control = await page.evaluate(() => {
    const number = document.querySelector('#slider-list input[type="number"][data-control-name]');
    return {
      name: String(number?.dataset?.controlName ?? "").trim(),
      min: String(number?.getAttribute("min") ?? "-1"),
      max: String(number?.getAttribute("max") ?? "1"),
      step: String(number?.getAttribute("step") ?? "0.1"),
    };
  });
  await page.locator('#screen-run:not(.hidden) .main-shell-tab[data-main-tab-target="publish"][data-shell-space="publish"]').click();
  await waitVisible(page, "#screen-publish");
  if (control.name) {
    await page.locator("#deployment-profile-control-name").fill(control.name);
    await page.locator("#deployment-profile-control-name").dispatchEvent("change");
    await page.locator("#deployment-profile-control-min").fill(control.min);
    await page.locator("#deployment-profile-control-min").dispatchEvent("change");
    await page.locator("#deployment-profile-control-max").fill(control.max);
    await page.locator("#deployment-profile-control-max").dispatchEvent("change");
    await page.locator("#deployment-profile-control-step").fill(control.step);
    await page.locator("#deployment-profile-control-step").dispatchEvent("change");
  }
  await page.locator("#btn-copy-current-lesson-bundle").click();
  await page.waitForFunction(() => window.__STUDIO_CLASSROOM_EXCHANGE_EXPORT_ACTION__?.payload?.classroom_exchange?.payload_hash, null, { timeout: 30_000 });
  return page.evaluate(() => window.__STUDIO_CLASSROOM_EXCHANGE_EXPORT_ACTION__.payload);
}

async function importBundle(page, baseUrl, payload) {
  await openPublishForPaste(page, baseUrl);
  await pastePayload(page, payload);
  await page.waitForFunction(() => {
    const run = document.querySelector("#screen-run");
    const runVisible = run && !String(run.className ?? "").split(/\s+/).includes("hidden");
    const status = document.querySelector("[data-local-package-import-status]");
    return runVisible || status?.dataset?.state === "error";
  }, null, { timeout: 30_000 });
  const importState = await page.evaluate(() => ({
    runVisible: Boolean(document.querySelector("#screen-run") && !String(document.querySelector("#screen-run").className ?? "").split(/\s+/).includes("hidden")),
    status: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
  }));
  assert(importState.runVisible, `bundle import failed: ${importState.status}`);
  await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true, null, { timeout: 30_000 });
  await page.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { timeout: 30_000 });
}

async function runAndSubmit(page) {
  await page.locator("textarea[data-curriculum-inquiry-prediction]").fill("포물선의 속도와 궤적을 예상한다.");
  await page.locator("textarea[data-curriculum-inquiry-prediction]").dispatchEvent("input");
  await page.locator("#btn-run").click();
  await page.waitForFunction(() => document.querySelector("[data-run-delivery-result]")?.dataset?.ready === "1", null, { timeout: 45_000 });
  await page.evaluate(() => {
    const number = document.querySelector('#slider-list input[type="number"][data-control-name]');
    if (!number) return;
    const step = Number(number.getAttribute("step"));
    const max = Number(number.getAttribute("max"));
    const current = Number(number.value);
    const next = Number.isFinite(current) && Number.isFinite(step) && current + step <= max ? current + step : current;
    number.value = String(next);
    number.dispatchEvent(new Event("input", { bubbles: true }));
    number.dispatchEvent(new Event("change", { bubbles: true }));
  });
  if (await page.locator("#btn-run").isDisabled()) {
    await page.locator("#btn-reset").click();
    await page.waitForFunction(() => !document.querySelector("#btn-run")?.disabled, null, { timeout: 20_000 });
  }
  await page.locator("#btn-run").click();
  await page.waitForFunction(() => document.querySelector("[data-run-delivery-result]")?.dataset?.ready === "1", null, { timeout: 45_000 });
  await page.locator("textarea[data-curriculum-inquiry-explanation]").fill("<img src=x onerror=\"window.__SEAMGRIM_XSS=1\"> 관찰 결과를 글로 설명한다.");
  await page.locator("textarea[data-curriculum-inquiry-explanation]").dispatchEvent("input");
  await page.locator("#run-delivery-student-name").fill("제출학생");
  await page.locator("#run-delivery-student-name").dispatchEvent("input");
  await page.locator("#btn-run-student-version-save").click();
  await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ASSIGNMENT_VERSION_SAVE_ACTION__?.saved === true, null, { timeout: 20_000 });
  await page.locator("#btn-run-student-submission-copy").click();
  await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_SUBMISSION_EXPORT_ACTION__?.submission?.payload_hash, null, { timeout: 20_000 });
  const first = await page.evaluate(() => window.__SEAMGRIM_STUDENT_SUBMISSION_EXPORT_ACTION__.submission);
  assert(first.kind === "student_submission", "submission kind mismatch");
  assert(first.response.explanation_text.includes("<img"), "submission response missing student text");
  assert(first.run_evidence.runtime_hash, "submission missing run evidence");
  assert(Object.keys(first.control_snapshot ?? {}).length === 1, "submission control snapshot not profile-limited");
  assert(first.origin_payload_hash, "submission missing origin payload hash");

  await page.locator("textarea[data-curriculum-inquiry-explanation]").fill("설명만 바꾼 새 제출");
  await page.locator("textarea[data-curriculum-inquiry-explanation]").dispatchEvent("input");
  await page.locator("#btn-run-student-submission-copy").click();
  await page.waitForFunction((prior) => {
    const next = window.__SEAMGRIM_STUDENT_SUBMISSION_EXPORT_ACTION__?.submission;
    return next?.payload_hash && next.payload_hash !== prior;
  }, first.payload_hash, { timeout: 20_000 });
  const changed = await page.evaluate(() => window.__SEAMGRIM_STUDENT_SUBMISSION_EXPORT_ACTION__.submission);
  assert(changed.payload_hash !== first.payload_hash, "changed explanation reused submission payload hash");
  assert(changed.evidence_binding_hash !== first.evidence_binding_hash, "changed explanation reused evidence binding hash");
  return first;
}

async function importSubmission(page, baseUrl, submission) {
  await openPublishForPaste(page, baseUrl);
  await pastePayload(page, submission);
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__?.imported === true, null, { timeout: 30_000 });
  const state = await page.evaluate(() => ({
    action: window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__,
    prediction: document.querySelector("textarea[data-curriculum-inquiry-prediction]")?.value ?? "",
    explanation: document.querySelector("textarea[data-curriculum-inquiry-explanation]")?.value ?? "",
    xss: window.__SEAMGRIM_XSS === true,
    imgCount: document.querySelectorAll("[data-curriculum-inquiry-panel] img").length,
    openedLessonId: window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__?.opened_lesson_id ?? "",
  }));
  assert(state.action.payload_hash === submission.payload_hash, "submission import hash mismatch");
  assert(state.explanation === submission.response.explanation_text, "submission explanation did not reopen");
  assert(state.openedLessonId.startsWith("student_submission_"), "student submission overwrote original lesson id");
  assert(!state.xss && state.imgCount === 0, "student text XSS executed or rendered as HTML");
}

function cloneWith(payload, mutator) {
  const next = JSON.parse(JSON.stringify(payload));
  mutator(next);
  return next;
}

async function expectRejected(page, baseUrl, payload, expectedText) {
  await openPublishForPaste(page, baseUrl);
  await pastePayload(page, payload);
  await page.waitForFunction(() => {
    const node = document.querySelector("[data-local-package-import-status]");
    return node?.dataset?.state === "error" && String(node.textContent ?? "").trim();
  }, null, { timeout: 30_000 });
  const text = String(await page.locator("[data-local-package-import-status]").textContent());
  assert(text.includes(expectedText), `expected diagnostic ${expectedText}, got: ${text}`);
  assert(!/student_submission_exchange_|studio_local_package_|Error:|http:\/\/|https:\/\/|[A-Z]:\\/u.test(text), `internal diagnostic leaked: ${text}`);
}

async function main() {
  const { server, baseUrl } = await createServer(process.cwd());
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  try {
    const teacherContext = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await teacherContext.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
    const teacherPage = await teacherContext.newPage();
    installFailureCapture(teacherPage, failures);
    const bundle = await exportLessonBundle(teacherPage, baseUrl);

    const studentContext = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await studentContext.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
    const studentPage = await studentContext.newPage();
    installFailureCapture(studentPage, failures);
    await importBundle(studentPage, baseUrl, bundle);
    const submission = await runAndSubmit(studentPage);

    const importContext = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await importContext.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
    const importPage = await importContext.newPage();
    installFailureCapture(importPage, failures);
    await importSubmission(importPage, baseUrl, submission);

    const tamperPage = await importContext.newPage();
    installFailureCapture(tamperPage, failures);
    await expectRejected(tamperPage, baseUrl, cloneWith(submission, (next) => { next.schema = "bad.schema"; }), "학생 제출 형식");
    await expectRejected(tamperPage, baseUrl, cloneWith(submission, (next) => { next.payload_hash = "candidate:fnv1a32:student_submission_payload:00000000"; }), "학생 제출 해시");
    await expectRejected(tamperPage, baseUrl, cloneWith(submission, (next) => { next.student_project_snapshot.padding = "x".repeat(1_000_050); }), "허용 크기");

    await teacherContext.close();
    await studentContext.close();
    await importContext.close();
    assert(failures.length === 0, failures.join("\n"));
    console.log(OK);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((error) => {
  console.error(error?.stack || String(error));
  process.exit(1);
});
