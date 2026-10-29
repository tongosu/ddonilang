#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";
import { buildStudentSubmissionExchangeV0 } from "../solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js";

const OK = "seamgrim_wave3b_submission_inbox: ok";
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
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true, null, { timeout: 30_000 });
  await page.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { timeout: 30_000 });
}

async function buildSubmission(browser, baseUrl, bundle, label, explanation) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await context.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
  const page = await context.newPage();
  await importBundle(page, baseUrl, bundle);
  await page.locator("textarea[data-curriculum-inquiry-prediction]").fill(`${label} prediction`);
  await page.locator("textarea[data-curriculum-inquiry-prediction]").dispatchEvent("input");
  await page.waitForFunction(() => !document.querySelector("#btn-run")?.disabled, null, { timeout: 30_000 });
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
  await page.locator("textarea[data-curriculum-inquiry-explanation]").fill(explanation);
  await page.locator("textarea[data-curriculum-inquiry-explanation]").dispatchEvent("input");
  await page.locator("#run-delivery-student-name").fill(label);
  await page.locator("#run-delivery-student-name").dispatchEvent("input");
  await page.locator("#btn-run-student-version-save").click();
  await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ASSIGNMENT_VERSION_SAVE_ACTION__?.saved === true, null, { timeout: 20_000 });
  await page.locator("#btn-run-student-submission-copy").click();
  await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_SUBMISSION_EXPORT_ACTION__?.submission?.payload_hash, null, { timeout: 20_000 });
  const submission = await page.evaluate(() => window.__SEAMGRIM_STUDENT_SUBMISSION_EXPORT_ACTION__.submission);
  await context.close();
  return submission;
}

function rebuildWithOrigin(submission, originPayloadHash) {
  return buildStudentSubmissionExchangeV0({
    lessonId: submission.lesson_id,
    originHash: originPayloadHash,
    originPayloadHash,
    deploymentProfileHash: submission.deployment_profile_hash,
    predictionText: submission.response?.prediction_text,
    explanationText: submission.response?.explanation_text,
    runEvidence: submission.run_evidence,
    controlSnapshot: submission.control_snapshot,
    observationRefs: submission.observation_refs,
    studentProjectSnapshot: submission.student_project_snapshot,
    lineage: {
      ...(submission.lineage ?? {}),
      origin_exchange_hash: originPayloadHash,
    },
    localLabel: submission.local_label,
    createdAtOrSequence: submission.created_at_or_sequence,
  });
}

async function openTeacherInbox(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&catalog_mode=full&lesson=${encodeURIComponent(LESSON_ID)}`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-run");
  await page.locator("#run-tab-btn-mirror").click();
  await page.evaluate(() => {
    const details = document.querySelector("#run-inspector-tools");
    if (details) details.open = true;
    window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__ = null;
    window.__SEAMGRIM_STUDENT_SUBMISSION_INBOX_REOPEN_RESULT__ = null;
  });
  await page.waitForFunction(() => {
    const panel = document.querySelector("#run-tab-panel-mirror");
    const details = document.querySelector("#run-inspector-tools");
    return panel && !panel.classList.contains("hidden") && details?.open === true;
  }, null, { timeout: 20_000 });
  await page.waitForFunction(() => document.querySelector("#run-student-result-input"), null, { timeout: 20_000 });
}

async function reviewInbox(page, submissions) {
  const text = JSON.stringify(submissions, null, 2);
  await page.evaluate((value) => {
    const input = document.querySelector("#run-student-result-input");
    if (!input) throw new Error("student result input missing");
    input.value = value;
    input.dispatchEvent(new Event("input", { bubbles: true }));
    document.querySelector("#btn-run-student-result-review")?.click();
  }, text);
  await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_SUBMISSION_INBOX_REVIEW__?.row_count >= 4, null, { timeout: 30_000 });
  const model = await page.evaluate(() => {
    const batch = window.__SEAMGRIM_STUDENT_SUBMISSION_INBOX_REVIEW__;
    return {
      row_count: batch.row_count,
      ready_count: batch.ready_count,
      review_count: batch.review_count,
      duplicate_count: batch.duplicate_count,
      ai_grading_claim: batch.ai_grading_claim,
      rows: batch.rows.map((row) => ({
        local_label: row.local_label,
        origin_match: row.origin_match,
        prediction_present: row.prediction_present,
        explanation_present: row.explanation_present,
        run_evidence_present: row.run_evidence_present,
        observation_ref_count: row.observation_ref_count,
        control_diff: row.control_diff,
        integrity_state: row.integrity_state,
        version_state: row.version_state,
        duplicate_state: row.duplicate_state,
        reopenable: row.reopenable,
        status: row.status,
      })),
      tableRows: document.querySelectorAll("[data-student-submission-inbox-row]").length,
      reopenButtons: document.querySelectorAll("[data-student-submission-reopen]:not([disabled])").length,
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent ?? "",
    };
  });
  assert(model.row_count === 4, `expected four inbox rows, got ${model.row_count}`);
  assert(model.ready_count >= 1, "expected at least one ready submission");
  assert(model.review_count >= 2, "expected review rows for duplicate/origin mismatch");
  assert(model.duplicate_count >= 2, "expected duplicate payload state");
  assert(model.ai_grading_claim === false, "inbox must not claim autograding");
  assert(model.tableRows === 4, "inbox table row count mismatch");
  assert(model.reopenButtons >= 2, "ready submissions must expose reopen buttons");
  assert(model.rows.every((row) => row.prediction_present && row.explanation_present && row.run_evidence_present), "submission rows missing response/evidence");
  assert(model.rows.every((row) => row.observation_ref_count >= 1), "submission rows missing observation refs");
  assert(model.rows.some((row) => row.origin_match === false), "origin mismatch row missing");
  assert(model.rows.some((row) => row.duplicate_state === "duplicate"), "duplicate row missing");
  assert(model.summaryText.includes("자동 채점 없음"), "no-autograding boundary missing from summary");
}

async function reopenFirstSubmission(page) {
  await page.locator("#run-tab-btn-mirror").click();
  await page.evaluate(() => {
    const details = document.querySelector("#run-inspector-tools");
    if (details) details.open = true;
  });
  await page.evaluate(() => {
    const button = document.querySelector("[data-student-submission-reopen]:not([disabled])");
    if (!button) throw new Error("student submission reopen button missing");
    button.click();
  });
  await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_SUBMISSION_INBOX_REOPEN_ACTION__?.requested === true, null, { timeout: 10_000 });
  try {
    await page.waitForFunction(() => (
      window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__?.imported === true
      && String(window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__?.opened_lesson_id ?? "").startsWith("student_submission_")
    ), null, { timeout: 30_000 });
  } catch (error) {
    const state = await page.evaluate(() => ({
      reopenAction: window.__SEAMGRIM_STUDENT_SUBMISSION_INBOX_REOPEN_ACTION__ ?? null,
      reopenResult: window.__SEAMGRIM_STUDENT_SUBMISSION_INBOX_REOPEN_RESULT__ ?? null,
      importAction: window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__ ?? null,
      actionKeys: Object.keys(window).filter((key) => key.includes("STUDENT_SUBMISSION")).sort(),
    }));
    throw new Error(`student submission reopen import did not finish: ${JSON.stringify(state)}\n${error.message}`);
  }
  const state = await page.evaluate(() => ({
    imported: window.__STUDIO_STUDENT_SUBMISSION_IMPORT_ACTION__,
    visibleRun: !document.querySelector("#screen-run")?.classList?.contains("hidden"),
    sourceTextLength: String(document.querySelector("#run-ddn-preview")?.value ?? "").trim().length,
  }));
  assert(state.visibleRun, "reopen did not leave run screen visible");
  assert(state.imported.opened_lesson_id.startsWith("student_submission_"), "reopen did not use student submission import path");
  assert(state.sourceTextLength > 20, "reopened submission source missing");
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

    const alice = await buildSubmission(browser, baseUrl, bundle, "Alice", "Alice explanation");
    const bob = await buildSubmission(browser, baseUrl, bundle, "Bob", "Bob explanation");
    const duplicateAlice = JSON.parse(JSON.stringify(alice));
    const otherOrigin = rebuildWithOrigin(bob, "candidate:fnv1a32:classroom_exchange_payload:ffffffff");

    await openTeacherInbox(teacherPage, baseUrl);
    await reviewInbox(teacherPage, [alice, bob, duplicateAlice, otherOrigin]);
    await reopenFirstSubmission(teacherPage);

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
