#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3b_foundation12_classroom_authoring: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const START_LESSON = "elem_math_number_line";
const TARGET_LESSON = "high_math_derivative";
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
  }, selector, { timeout: 20_000 });
}

async function openLessonPlanAuthoring(page, baseUrl, lessonId = START_LESSON) {
  await page.goto(`${baseUrl}${PROJECT_URL}?catalog_mode=full`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-browse");
  const cardSelector = `.lesson-card[data-lesson-id='${lessonId}']`;
  await page.waitForSelector(cardSelector, { state: "attached", timeout: 20_000 });
  await page.locator(cardSelector).scrollIntoViewIfNeeded();
  await page.locator(cardSelector).click();
  await page.waitForSelector(`[data-curriculum-card][data-curriculum-card-lesson-id='${lessonId}']`, {
    state: "visible",
    timeout: 10_000,
  });
  await page.locator("#btn-open-in-studio").click();
  await waitVisible(page, "#screen-run");
  const createTab = page.locator('#screen-run:not(.hidden) .main-shell-tab[data-main-tab-target="studio"][data-shell-space="create"]');
  await createTab.waitFor({ state: "visible", timeout: 20_000 });
  await createTab.click();
  await waitVisible(page, "#screen-editor");
  await page.locator("#btn-editor-lesson-plan").click();
  await page.waitForSelector("[data-testid='lesson-plan-classroom-fields']", { state: "visible", timeout: 20_000 });
}

async function fillAndChange(page, selector, value) {
  await page.locator(selector).fill(value);
  await page.locator(selector).dispatchEvent("change");
}

async function selectAndChange(page, selector, value) {
  await page.locator(selector).selectOption(value);
  await page.locator(selector).dispatchEvent("change");
}

async function readAuthoringState(page) {
  return page.evaluate(async () => {
    const { parseLessonPlanToml, serializeLessonPlanToml, roundtripLessonPlanToml } = await import("./lesson_plan_v0.js");
    const code = document.querySelector("[data-lesson-plan-code]")?.value ?? "";
    const parsed = parseLessonPlanToml(code);
    return {
      code,
      canonical: serializeLessonPlanToml(parsed),
      parsed,
      roundtrip: roundtripLessonPlanToml(code),
      options: [...document.querySelectorAll("[data-testid='foundation-lesson-select'] option")].map((option) => option.value),
      selectedWorld: document.querySelector("[data-testid='foundation-lesson-select']")?.value ?? "",
      ddnSource: document.querySelector("#ddn-textarea")?.value ?? "",
      editorMode: document.querySelector("#screen-editor")?.dataset?.editorMode ?? "",
      roleToggleCount: document.querySelectorAll("[data-runtime-role-toggle], [data-role-toggle], [data-student-teacher-toggle]").length,
      fieldValues: {
        title: document.querySelector("[data-testid='lesson-plan-title-field']")?.value ?? "",
        readingCheck: document.querySelector("[data-testid='lesson-plan-reading-check-field']")?.value ?? "",
      },
    };
  });
}

async function editClassroomPlan(page) {
  let state = await readAuthoringState(page);
  assert(state.options.length === 12, `foundation option count mismatch: ${state.options.length}`);
  assert(FOUNDATION12.every((id) => state.options.includes(id)), `foundation options missing: ${state.options.join(",")}`);

  await selectAndChange(page, "[data-testid='foundation-lesson-select']", TARGET_LESSON);
  await page.waitForFunction((lessonId) => {
    const code = document.querySelector("[data-lesson-plan-code]")?.value ?? "";
    return code.includes(`lesson_id = "${lessonId}"`);
  }, TARGET_LESSON, { timeout: 20_000 });

  await fillAndChange(page, "[data-testid='lesson-plan-title-field']", "Foundation 12 derivative classroom");
  await fillAndChange(page, "[data-testid='lesson-plan-question-field']", "How does the slope change as x moves?");
  await fillAndChange(page, "[data-testid='lesson-plan-prediction-field']", "Predict where the derivative is largest.");
  await fillAndChange(page, "[data-testid='lesson-plan-explanation-field']", "Explain using the graph and the observed slope.");
  await fillAndChange(page, "[data-testid='lesson-plan-misconception-field']", "slope-as-height, derivative-is-original");
  await fillAndChange(page, "[data-testid='lesson-plan-reading-check-field']", "Name the x value and slope you used.");
  await fillAndChange(page, "[data-testid='lesson-plan-rubric-field']", "1 prediction, 1 controlled change, 1 graph evidence, 1 explanation.");
  await fillAndChange(page, "[data-testid='lesson-plan-root-allowed-controls']", "start_x, end_x, step");
  await fillAndChange(page, "[data-testid='lesson-plan-control-ranges']", "start_x:-3..0@0.5, end_x:0..3@0.5, step:0.1..1@0.1");
  await selectAndChange(page, "[data-testid='lesson-plan-root-primary-view']", "graph");
  await fillAndChange(page, "[data-testid='lesson-plan-root-secondary-views']", "text, table");
  await selectAndChange(page, "[data-testid='lesson-plan-deployment-lock']", "locked");

  await fillAndChange(page, "[data-lesson-plan-field='title']", "Observe derivative trend");
  await fillAndChange(page, "[data-lesson-plan-field='question']", "Which interval changes fastest?");
  await fillAndChange(page, "[data-lesson-plan-field='allowed_controls']", "start_x, end_x");
  await page.locator("[data-lesson-plan-action='add']").click();
  await fillAndChange(page, "[data-lesson-plan-field='title']", "Explain slope evidence");
  await fillAndChange(page, "[data-lesson-plan-field='prompt']", "Use one graph point and one numeric observation.");
  await page.locator("[data-lesson-plan-action='up']").click();

  state = await readAuthoringState(page);
  const model = state.parsed;
  assert(model.lesson_id === TARGET_LESSON, "selected world did not update lesson_id");
  assert(model.title === "Foundation 12 derivative classroom", "title mismatch");
  assert(model.lesson_question.includes("slope change"), "lesson question missing");
  assert(model.prediction_question.includes("derivative"), "prediction question missing");
  assert(model.explanation_question.includes("observed slope"), "explanation question missing");
  assert(model.misconception_refs.join(",") === "slope-as-height,derivative-is-original", "misconceptions mismatch");
  assert(model.reading_check.includes("x value"), "reading check mismatch");
  assert(model.rubric.includes("controlled change"), "rubric mismatch");
  assert(model.allowed_controls.join(",") === "start_x,end_x,step", "root allowed controls mismatch");
  assert(model.control_ranges.length === 3, "control ranges mismatch");
  assert(model.primary_view === "graph", "root primary view mismatch");
  assert(model.secondary_views.join(",") === "text,table", "secondary views mismatch");
  assert(model.deployment_profile_lock === "locked", "deployment lock mismatch");
  assert(model.activities[0].title === "Explain slope evidence", "activity order did not move");
  assert(state.roundtrip.byte_stable === true, "plan code must be byte-stable");
  assert(state.roundtrip.semantic_equal === true, "plan code must be semantically stable");

  const beforeSource = state.ddnSource;
  await page.locator("#btn-editor-world-code").click();
  await page.waitForFunction(() => document.querySelector("#screen-editor")?.dataset?.editorMode === "world-code");
  await page.locator("#ddn-textarea").fill(`${beforeSource}\n// DDN_ONLY_MARKER`);
  await page.locator("#btn-editor-lesson-plan").click();
  state = await readAuthoringState(page);
  assert(!state.code.includes("DDN_ONLY_MARKER"), "world source leaked into lesson plan code");

  await page.locator("#btn-save-ddn").click();
  await page.waitForFunction(() => {
    const draft = window.localStorage?.getItem?.("seamgrim.lesson_plan.v0.draft:high_math_derivative") ?? "";
    return draft.includes("Foundation 12 derivative classroom") && draft.includes("deployment_profile_lock = \"locked\"");
  }, null, { timeout: 10_000 });
}

async function verifyReopen(context, baseUrl) {
  const page = await context.newPage({ viewport: { width: 1280, height: 800 }, locale: "ko-KR" });
  const failures = [];
  installFailureCapture(page, failures);
  await openLessonPlanAuthoring(page, baseUrl, TARGET_LESSON);
  const state = await readAuthoringState(page);
  assert(state.fieldValues.title === "Foundation 12 derivative classroom", "reopened title mismatch");
  assert(state.parsed.deployment_profile_lock === "locked", "reopened deployment lock mismatch");
  assert(state.parsed.reading_check.includes("x value"), "reopened reading check mismatch");
  assert(state.roleToggleCount === 0, "runtime role toggle appeared in editor");
  assert(failures.length === 0, failures.join("\n"));
  await page.close();
}

async function verifyMobile(context, baseUrl) {
  const page = await context.newPage({ viewport: { width: 390, height: 844 }, locale: "ko-KR" });
  const failures = [];
  installFailureCapture(page, failures);
  await openLessonPlanAuthoring(page, baseUrl, TARGET_LESSON);
  await page.waitForFunction(() => {
    const box = document.querySelector("[data-testid='lesson-plan-classroom-fields']")?.getBoundingClientRect();
    return box && box.width > 280 && box.height > 120;
  }, null, { timeout: 10_000 });
  await fillAndChange(page, "[data-testid='lesson-plan-reading-check-field']", "Mobile reading check edit");
  const state = await readAuthoringState(page);
  assert(state.fieldValues.readingCheck === "Mobile reading check edit", "mobile small text edit mismatch");
  assert(state.editorMode === "lesson-plan", "mobile editor mode mismatch");
  assert(failures.length === 0, failures.join("\n"));
  await page.close();
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, locale: "ko-KR" });
    const page = await context.newPage();
    installFailureCapture(page, failures);
    await openLessonPlanAuthoring(page, baseUrl, START_LESSON);
    await editClassroomPlan(page);
    assert(failures.length === 0, failures.join("\n"));
    await verifyReopen(context, baseUrl);
    await verifyMobile(context, baseUrl);
    await context.close();
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
