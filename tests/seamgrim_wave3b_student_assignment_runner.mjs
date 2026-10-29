#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3b_student_assignment: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const FOUNDATION12 = Object.freeze([
  "elem_math_number_line",
  "elem_math_fraction_line",
  "elem_math_rectangle_area",
  "elem_science_daily_temperature",
  "elem_econ_allowance_budget",
  "elem_econ_store_stock",
  "rep_math_function_line_v1",
  "rep_phys_pendulum_wave1_v1",
  "rep_phys_projectile_xy_v1",
  "rep_econ_supply_demand_tax_v1",
  "rep_phys_damped_oscillation_v1",
  "high_math_derivative",
]);

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

async function openRunLesson(page, baseUrl, lessonId) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&catalog_mode=full&lesson=${encodeURIComponent(lessonId)}`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:"), null, {
    timeout: 30_000,
  });
}

async function exportLessonBundle(page, baseUrl, lessonId) {
  await openRunLesson(page, baseUrl, lessonId);
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
  await page.waitForFunction(() => window.__STUDIO_CLASSROOM_EXCHANGE_EXPORT_ACTION__?.payload?.classroom_exchange?.payload_hash, null, {
    timeout: 30_000,
  });
  const model = await page.evaluate(() => window.__STUDIO_CLASSROOM_EXCHANGE_EXPORT_ACTION__);
  assert(model?.copied === true, `bundle export failed for ${lessonId}`);
  assert(model.payload.classroom_exchange.kind === "lesson_bundle", `wrong exchange kind for ${lessonId}`);
  return model.payload;
}

async function importAssignment(page, baseUrl, payload) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=publish&catalog_mode=full`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-publish");
  await page.locator("#btn-paste-local-package").click();
  await page.waitForSelector("#local-package-paste-text", { state: "visible", timeout: 20_000 });
  await pastePayload(page, payload);
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true, null, { timeout: 30_000 });
  try {
    await page.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { timeout: 30_000 });
  } catch (error) {
    const state = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      runDataset: { ...document.querySelector("#screen-run")?.dataset },
      inquiryHidden: document.querySelector("[data-curriculum-inquiry-panel]")?.classList?.contains("hidden") ?? null,
      inquiryCardText: document.querySelector("[data-curriculum-inquiry-card]")?.textContent ?? "",
      title: document.querySelector("[data-run-title]")?.textContent ?? "",
    }));
    throw new Error(`student inquiry panel did not open: ${JSON.stringify(state)}`, { cause: error });
  }
}

async function runStudentFlow(page, lessonId) {
  await page.locator("textarea[data-curriculum-inquiry-prediction]").fill(`예상: ${lessonId} 값을 바꾸면 관찰값도 바뀐다.`);
  await page.locator("textarea[data-curriculum-inquiry-prediction]").dispatchEvent("input");

  await page.waitForFunction(() => {
    const button = document.querySelector("#btn-run");
    return button && !button.disabled;
  }, null, { timeout: 30_000 }).catch(async (error) => {
    const state = await page.evaluate(() => ({
      runButtonText: document.querySelector("#btn-run")?.textContent ?? "",
      runButtonDisabled: document.querySelector("#btn-run")?.disabled ?? null,
      engineStatus: document.querySelector("#screen-run")?.dataset?.engineStatus ?? "",
      errorBannerHidden: document.querySelector("#run-error-banner")?.classList?.contains("hidden") ?? null,
      errorText: document.querySelector("#run-error-banner")?.textContent ?? "",
      sliderCount: document.querySelectorAll('#slider-list input[data-control-name]').length,
      sliderNames: Array.from(document.querySelectorAll('#slider-list input[data-control-name]')).map((node) => node.dataset.controlName),
      inquiryRuntime: document.querySelector("#screen-run")?.dataset?.curriculumInquiryRuntime ?? "",
      localPackageHash: document.querySelector("#screen-run")?.dataset?.localPackageHash ?? "",
    }));
    throw new Error(`run button did not enable for ${lessonId}: ${JSON.stringify(state)}`, { cause: error });
  });
  await page.locator("#btn-run").click();
  await page.waitForFunction(() => {
    const hash = String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "");
    return hash.includes("blake3:") && document.querySelector("[data-run-delivery-result]")?.dataset?.ready === "1";
  }, null, { timeout: 45_000 });
  const firstHash = await page.locator("#run-mirror-hash").getAttribute("title");

  const changedControl = await page.evaluate(() => {
    const number = document.querySelector('#slider-list input[type="number"][data-control-name]');
    if (!number) return { changed: false, count: 0 };
    const min = Number(number.getAttribute("min"));
    const max = Number(number.getAttribute("max"));
    const step = Number(number.getAttribute("step"));
    const current = Number(number.value);
    const next = Number.isFinite(current) && Number.isFinite(step) && current + step <= max
      ? current + step
      : (Number.isFinite(min) ? min : current);
    number.value = String(next);
    number.dispatchEvent(new Event("input", { bubbles: true }));
    number.dispatchEvent(new Event("change", { bubbles: true }));
    return {
      changed: true,
      count: document.querySelectorAll('#slider-list input[type="number"][data-control-name]').length,
      name: String(number.dataset.controlName ?? ""),
      min,
      max,
      step,
      value: next,
    };
  });
  assert(changedControl.changed, `no allowed control available for ${lessonId}`);
  assert(changedControl.count === 1, `deployment profile did not restrict controls for ${lessonId}`);

  if (await page.locator("#btn-run").isDisabled()) {
    await page.locator("#btn-reset").click();
    await page.waitForFunction(() => !document.querySelector("#btn-run")?.disabled, null, { timeout: 20_000 });
  }
  await page.locator("#btn-run").click();
  await page.waitForFunction((prior) => {
    const hash = String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "");
    return hash.includes("blake3:") && hash !== prior && document.querySelector("[data-run-delivery-result]")?.dataset?.ready === "1";
  }, firstHash, { timeout: 45_000 });

  await page.locator("textarea[data-curriculum-inquiry-explanation]").fill(`설명: ${lessonId} 실행 근거와 관찰값을 비교했다.`);
  await page.locator("textarea[data-curriculum-inquiry-explanation]").dispatchEvent("input");

  const codeVisible = await page.evaluate(() => String(document.querySelector("#run-ddn-preview")?.value ?? "").trim().length > 20);
  assert(codeVisible, `DDN source unavailable for ${lessonId}`);

  await page.locator("#run-delivery-student-name").fill(`학생-${lessonId}`);
  await page.locator("#run-delivery-student-name").dispatchEvent("input");
  await page.locator("#btn-run-student-version-save").click();
  await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ASSIGNMENT_VERSION_SAVE_ACTION__?.saved === true, null, { timeout: 20_000 });
  const saveAction = await page.evaluate(() => window.__SEAMGRIM_STUDENT_ASSIGNMENT_VERSION_SAVE_ACTION__);
  assert(saveAction.snapshot.response.prediction_text.includes(lessonId), `prediction not saved for ${lessonId}`);
  assert(saveAction.snapshot.response.explanation_text.includes(lessonId), `explanation not saved for ${lessonId}`);
  assert(saveAction.snapshot.run_evidence.runtime_hash, `runtime hash not saved for ${lessonId}`);
  assert(Object.keys(saveAction.snapshot.control_snapshot ?? {}).length === 1, `control snapshot not profile-restricted for ${lessonId}`);
  return saveAction.snapshot;
}

async function assertReopenRestores(page, baseUrl, payload, snapshot, lessonId) {
  await importAssignment(page, baseUrl, payload);
  await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ASSIGNMENT_VERSION_RESTORE_ACTION__?.restored === true, null, {
    timeout: 20_000,
  });
  const state = await page.evaluate(() => ({
    prediction: document.querySelector("textarea[data-curriculum-inquiry-prediction]")?.value ?? "",
    explanation: document.querySelector("textarea[data-curriculum-inquiry-explanation]")?.value ?? "",
    restoredHash: document.querySelector("#screen-run")?.dataset?.studentAssignmentVersionHash ?? "",
    runtimeHash: document.querySelector("#screen-run")?.dataset?.curriculumInquiryRuntimeHash ?? "",
  }));
  assert(state.prediction === snapshot.response.prediction_text, `prediction did not restore for ${lessonId}`);
  assert(state.explanation === snapshot.response.explanation_text, `explanation did not restore for ${lessonId}`);
  assert(state.restoredHash === snapshot.snapshot_hash, `snapshot hash did not restore for ${lessonId}`);
  assert(state.runtimeHash === snapshot.run_evidence.runtime_hash, `runtime hash did not restore for ${lessonId}`);
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

    const studentContext = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await studentContext.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
    const studentPage = await studentContext.newPage();
    installFailureCapture(studentPage, failures);

    const results = [];
    for (const lessonId of FOUNDATION12) {
      const payload = await exportLessonBundle(teacherPage, baseUrl, lessonId);
      await importAssignment(studentPage, baseUrl, payload);
      const snapshot = await runStudentFlow(studentPage, lessonId);
      const reopenPage = await studentContext.newPage();
      installFailureCapture(reopenPage, failures);
      await assertReopenRestores(reopenPage, baseUrl, payload, snapshot, lessonId);
      await reopenPage.close();
      results.push({
        lesson_id: lessonId,
        exchange_hash: payload.classroom_exchange.payload_hash,
        snapshot_hash: snapshot.snapshot_hash,
      });
    }

    await teacherContext.close();
    await studentContext.close();
    assert(results.length === FOUNDATION12.length, "not all foundation lessons were checked");
    assert(failures.length === 0, failures.join("\n"));
    console.log(`${OK} ${JSON.stringify({ checked: results.length })}`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((error) => {
  console.error(error?.stack || String(error));
  process.exit(1);
});
