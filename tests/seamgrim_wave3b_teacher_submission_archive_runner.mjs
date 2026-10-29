#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";
import { buildStudentSubmissionExchangeV0 } from "../solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js";

const OK = "seamgrim_wave3b_teacher_submission_archive: ok";
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

async function pastePackagePayload(page, payload) {
  const text = JSON.stringify(payload, null, 2);
  await page.evaluate((value) => {
    const node = document.querySelector("#local-package-paste-text");
    if (!node) throw new Error("package paste textarea missing");
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
  await pastePackagePayload(page, payload);
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true, null, { timeout: 30_000 });
  await page.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { timeout: 30_000 });
}

async function buildSubmission(page, baseUrl, bundle, { label, prediction, explanation, stepCount }) {
  await importBundle(page, baseUrl, bundle);
  await page.locator("textarea[data-curriculum-inquiry-prediction]").fill(prediction);
  await page.locator("textarea[data-curriculum-inquiry-prediction]").dispatchEvent("input");
  await page.locator("#btn-run").click();
  await page.waitForFunction(() => document.querySelector("[data-run-delivery-result]")?.dataset?.ready === "1", null, { timeout: 45_000 });
  await page.evaluate((count) => {
    const number = document.querySelector('#slider-list input[type="number"][data-control-name]');
    if (!number) return;
    const step = Number(number.getAttribute("step"));
    const max = Number(number.getAttribute("max"));
    const current = Number(number.value);
    const next = Number.isFinite(current) && Number.isFinite(step)
      ? Math.min(max, current + step * count)
      : current;
    number.value = String(next);
    number.dispatchEvent(new Event("input", { bubbles: true }));
    number.dispatchEvent(new Event("change", { bubbles: true }));
  }, stepCount);
  if (await page.locator("#btn-run").isDisabled()) {
    await page.locator("#btn-reset").click();
    await page.waitForFunction(() => !document.querySelector("#btn-run")?.disabled, null, { timeout: 20_000 });
  }
  await page.locator("#btn-run").click();
  await page.waitForFunction(() => document.querySelector("[data-run-delivery-result]")?.dataset?.ready === "1", null, { timeout: 45_000 });
  await page.locator("textarea[data-curriculum-inquiry-explanation]").fill(explanation);
  await page.locator("textarea[data-curriculum-inquiry-explanation]").dispatchEvent("input");
  await page.locator("#run-delivery-student-name").fill(label);
  await page.locator("#run-delivery-student-name").dispatchEvent("input");
  await page.locator("#btn-run-student-submission-copy").click();
  await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_SUBMISSION_EXPORT_ACTION__?.submission?.payload_hash, null, { timeout: 20_000 });
  const submission = await page.evaluate(() => window.__SEAMGRIM_STUDENT_SUBMISSION_EXPORT_ACTION__.submission);
  assert(submission.local_label === label, `submission label mismatch: ${submission.local_label}`);
  assert(submission.student_project_snapshot?.origin_control_baseline, "submission missing origin control baseline");
  return submission;
}

function rebuildDifferentLessonSubmission(base) {
  const snapshot = {
    ...(base.student_project_snapshot ?? {}),
    lesson_id: "other_foundation_lesson",
    exchange_id: "student_submission.other_foundation_lesson.valid",
  };
  return buildStudentSubmissionExchangeV0({
    lessonId: "other_foundation_lesson",
    originHash: base.origin_hash,
    originPayloadHash: base.origin_payload_hash,
    deploymentProfileHash: base.deployment_profile_hash,
    predictionText: base.response?.prediction_text,
    explanationText: base.response?.explanation_text,
    runEvidence: base.run_evidence,
    controlSnapshot: base.control_snapshot,
    observationRefs: base.observation_refs,
    studentProjectSnapshot: snapshot,
    lineage: base.lineage,
    localLabel: "다른수업",
    createdAtOrSequence: "local-student-sequence-0001",
  });
}

async function openPublishArchive(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=publish&catalog_mode=full`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-publish");
  await page.waitForSelector("[data-student-submission-archive]", { state: "visible", timeout: 20_000 });
}

async function addToArchive(page, payload, expectedCount) {
  await page.evaluate((text) => {
    const node = document.querySelector("#student-submission-paste-text");
    if (!node) throw new Error("submission paste textarea missing");
    node.value = text;
    node.dispatchEvent(new Event("input", { bubbles: true }));
  }, JSON.stringify(payload, null, 2));
  await page.locator("#btn-student-submission-add").click();
  await page.waitForFunction((count) => (
    window.__STUDIO_STUDENT_SUBMISSION_ARCHIVE_ACTION__?.accepted === true
    && window.__STUDIO_STUDENT_SUBMISSION_ARCHIVE_ACTION__?.count === count
  ), expectedCount, { timeout: 20_000 });
}

async function expectArchiveRejected(page, payload, expectedText) {
  await page.evaluate((text) => {
    const node = document.querySelector("#student-submission-paste-text");
    if (!node) throw new Error("submission paste textarea missing");
    node.value = text;
    node.dispatchEvent(new Event("input", { bubbles: true }));
  }, JSON.stringify(payload, null, 2));
  await page.locator("#btn-student-submission-add").click();
  await page.waitForFunction((text) => {
    const action = window.__STUDIO_STUDENT_SUBMISSION_ARCHIVE_ACTION__;
    const status = document.querySelector("[data-student-submission-archive-status]")?.textContent?.trim() ?? "";
    return action?.accepted === false && status.includes(text);
  }, expectedText, { timeout: 20_000 });
  const state = await page.evaluate(() => ({
    action: window.__STUDIO_STUDENT_SUBMISSION_ARCHIVE_ACTION__,
    status: document.querySelector("[data-student-submission-archive-status]")?.textContent?.trim() ?? "",
    count: document.querySelectorAll("[data-student-submission-row]").length,
  }));
  assert(state.status.includes(expectedText), `expected rejection ${expectedText}, got: ${state.status}`);
  assert(state.count === 3, `archive count changed after rejection: ${state.count}`);
}

async function validateArchive(page, submissions) {
  await openPublishArchive(page, page.url().replace(/\/solutions\/seamgrim_ui_mvp\/ui\/index\.html.*$/u, ""));
  for (const [index, submission] of submissions.entries()) {
    await addToArchive(page, submission, index + 1);
  }
  const summary = await page.locator("[data-student-submission-archive-summary]").textContent();
  assert(String(summary).includes("제출 3건"), `archive summary mismatch: ${summary}`);
  const rows = await page.evaluate(() => Array.from(document.querySelectorAll("[data-student-submission-row]")).map((node) => ({
    hash: node.getAttribute("data-student-submission-row"),
    text: node.textContent,
  })));
  assert(rows.length === 3, `archive row count mismatch: ${rows.length}`);
  rows.forEach((row, index) => {
    assert(row.text.includes(submissions[index].local_label), `row missing label: ${row.text}`);
    assert(row.text.includes("무결성:valid"), `row missing integrity: ${row.text}`);
    assert(row.text.includes("중복:unique"), `row missing duplicate status: ${row.text}`);
    assert(row.text.includes("실행 근거:"), `row missing run evidence: ${row.text}`);
    assert(row.text.includes("->"), `row missing control diff: ${row.text}`);
  });
  await expectArchiveRejected(page, submissions[0], "중복 제출");
  await expectArchiveRejected(page, rebuildDifferentLessonSubmission(submissions[0]), "다른 수업");

  await page.locator("#btn-student-submission-archive-export").click();
  await page.waitForFunction(() => window.__STUDIO_STUDENT_SUBMISSION_ARCHIVE_EXPORT_ACTION__?.archive?.review_count === 3, null, { timeout: 20_000 });
  const archive = await page.evaluate(() => window.__STUDIO_STUDENT_SUBMISSION_ARCHIVE_EXPORT_ACTION__.archive);
  assert(archive.export_hash, "archive export hash missing");
  assert(archive.auto_score_claim === false, "archive must not claim auto score");
  assert(archive.rows.every((row) => row.integrity_status === "valid"), "archive export integrity mismatch");

  await page.locator(`[data-student-submission-reopen="${submissions[0].payload_hash}"]`).click();
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__?.imported === true, null, { timeout: 30_000 });
  const reopen = await page.evaluate(() => ({
    action: window.__STUDIO_STUDENT_SUBMISSION_ARCHIVE_REOPEN_ACTION__,
    prediction: document.querySelector("textarea[data-curriculum-inquiry-prediction]")?.value ?? "",
    explanation: document.querySelector("textarea[data-curriculum-inquiry-explanation]")?.value ?? "",
    codeVisible: String(document.querySelector("#run-ddn-preview")?.value ?? "").trim().length > 20,
    runVisible: Boolean(document.querySelector("#screen-run") && !String(document.querySelector("#screen-run").className ?? "").split(/\s+/).includes("hidden")),
  }));
  assert(reopen.action?.reopened === true, "archive reopen action missing");
  assert(reopen.prediction === submissions[0].response.prediction_text, "reopened prediction mismatch");
  assert(reopen.explanation === submissions[0].response.explanation_text, "reopened explanation mismatch");
  assert(reopen.runVisible && reopen.codeVisible, "reopened project/code unavailable");
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

    const specs = [
      { label: "가영", prediction: "속도 벡터가 궤적을 따라 바뀐다.", explanation: "첫 번째 학생 설명", stepCount: 1 },
      { label: "나래", prediction: "높이가 커지면 궤적도 달라진다.", explanation: "두 번째 학생 설명", stepCount: 2 },
      { label: "다온", prediction: "초기 속도가 클수록 더 멀리 간다.", explanation: "세 번째 학생 설명", stepCount: 3 },
    ];
    const submissions = [];
    for (const spec of specs) {
      const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
      await context.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
      const page = await context.newPage();
      installFailureCapture(page, failures);
      submissions.push(await buildSubmission(page, baseUrl, bundle, spec));
      await context.close();
    }
    assert(new Set(submissions.map((item) => item.payload_hash)).size === 3, "submissions must be distinct");
    assert(new Set(submissions.map((item) => item.origin_payload_hash)).size === 1, "submissions must share one origin");

    const archiveContext = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await archiveContext.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
    const archivePage = await archiveContext.newPage();
    installFailureCapture(archivePage, failures);
    await openPublishArchive(archivePage, baseUrl);
    await validateArchive(archivePage, submissions);
    await archiveContext.close();

    await teacherContext.close();
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
