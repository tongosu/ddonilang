#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";
import {
  LESSON_PLAN_V0_SCHEMA,
  parseLessonPlanToml,
  roundtripLessonPlanToml,
  serializeLessonPlanToml,
} from "../solutions/seamgrim_ui_mvp/ui/lesson_plan_v0.js";

const OK = "seamgrim_lesson_plan_v0: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const LESSON_ID = "rep_phys_pendulum_wave1_v1";
const LESSON_PLAN_PATH = "solutions/seamgrim_ui_mvp/lessons/rep_phys_pendulum_wave1_v1/lesson_plan.v0.toml";

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
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|maegim_control)\.(?:json|detjson)$/i.test(pathname)
    ) return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/text\.md$/i.test(pathname)
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

async function readUiState(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const editor = document.querySelector("#screen-editor");
    return {
      run: { ...(root?.dataset ?? {}) },
      editor: { ...(editor?.dataset ?? {}) },
      activityPrompt: document.querySelector("[data-lesson-plan-prompt]")?.textContent ?? "",
      activityMeta: document.querySelector("[data-lesson-plan-meta]")?.textContent ?? "",
      sliderRows: [...document.querySelectorAll(".slider-row")].map((row) => row.getAttribute("data-control-name") || ""),
      planCode: document.querySelector("[data-lesson-plan-code]")?.value ?? "",
      planStatus: document.querySelector("[data-lesson-plan-status]")?.textContent ?? "",
      savedDraft: window.localStorage?.getItem?.("seamgrim.lesson_plan.v0.draft") ?? "",
      lessonPlan: window.__SEAMGRIM_LESSON_PLAN__ ?? null,
    };
  });
}

async function main() {
  const source = await fs.readFile(LESSON_PLAN_PATH, "utf8");
  const parsed = parseLessonPlanToml(source);
  const roundtrip = roundtripLessonPlanToml(source);
  assert(parsed.schema === LESSON_PLAN_V0_SCHEMA, "module parser schema mismatch");
  assert(parsed.activities.length === 4, "pendulum lesson plan must have four activities");
  assert(roundtrip.byte_stable === true && roundtrip.semantic_equal === true, "lesson plan roundtrip not stable");
  assert(serializeLessonPlanToml(parsed) === roundtrip.canonical_text, "canonical text mismatch");

  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    installFailureCapture(page, failures);
    await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${LESSON_ID}`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
    await page.waitForFunction(() => {
      const root = document.querySelector("#screen-run");
      return root?.dataset?.lessonPlanRuntime === "1"
        && root?.dataset?.lessonPlanActivityCount === "4"
        && window.__SEAMGRIM_LESSON_PLAN__?.model?.schema === "seamgrim.lesson_plan.v0";
    }, null, { timeout: 25_000 });

    let state = await readUiState(page);
    assert(state.run.lessonPlanCurrentActivityId === "observe-angle-energy", "first activity id mismatch");
    assert(state.run.lessonPlanActivityKind === "observe", "first activity kind mismatch");
    assert(state.run.lessonPlanSelectedAnchor === "obs.theta", "observe anchor mismatch");
    assert(state.activityMeta.includes("관찰 대상: 각도, 운동에너지, 위치에너지"), `observe labels missing: ${state.activityMeta}`);
    assert(!state.activityMeta.includes("obs."), `internal anchor id leaked into lesson UI: ${state.activityMeta}`);

    await page.locator("[data-lesson-plan-next]").click();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.lessonPlanActivityKind === "change");
    state = await readUiState(page);
    assert(state.run.lessonPlanAllowedControls === "theta0", `change allowed controls mismatch: ${state.run.lessonPlanAllowedControls}`);
    assert(state.sliderRows.join(",") === "theta0", `change activity did not filter sliders: ${state.sliderRows.join(",")}`);

    await page.locator("[data-lesson-plan-next]").click();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.lessonPlanActivityKind === "compare");
    state = await readUiState(page);
    assert(state.activityMeta.includes("lineage:local_compare"), "compare activity missing check_ref");

    await page.locator("[data-lesson-plan-next]").click();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.lessonPlanActivityKind === "explain");
    state = await readUiState(page);
    assert(state.activityPrompt.includes("각도가 0"), "explain prompt missing");
    assert(state.activityMeta.includes("영점 통과"), "explain question missing");
    await page.locator("[data-lesson-plan-complete]").click();
    state = await readUiState(page);
    assert(state.lessonPlan.completed_activity_ids.includes("explain-zero-crossing-energy"), "complete action missing");

    await page.locator("#screen-run .main-shell-tab[data-shell-space='create']").click();
    await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 10_000 });
    await page.locator("#btn-editor-lesson-plan").click();
    await page.waitForSelector("[data-lesson-plan-editor]", { state: "visible", timeout: 10_000 });
    state = await readUiState(page);
    assert(state.planCode.includes("observe-angle-energy"), "editor did not load lesson plan code");

    await page.locator("[data-lesson-plan-action='add']").click();
    await page.locator("[data-lesson-plan-field='title']").fill("새 비교 질문");
    await page.locator("[data-lesson-plan-field='title']").dispatchEvent("change");
    await page.locator("[data-lesson-plan-action='up']").click();
    state = await readUiState(page);
    assert(state.planCode.includes("새 비교 질문"), "list edit did not update code");
    const canonicalAfterList = roundtripLessonPlanToml(state.planCode).canonical_text;

    const editedCode = canonicalAfterList.replace("새 비교 질문", "새 비교 질문 코드반영");
    await page.locator("[data-lesson-plan-code]").fill(editedCode);
    await page.locator("[data-lesson-plan-action='apply-code']").click();
    state = await readUiState(page);
    assert(state.planCode.includes("새 비교 질문 코드반영"), "code edit did not update list/model");

    await page.locator("#btn-save-ddn").click();
    state = await readUiState(page);
    assert(state.savedDraft.includes("새 비교 질문 코드반영"), "save did not persist lesson plan draft");

    await page.locator("#screen-editor .main-shell-tab[data-shell-space='lab']").click();
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 10_000 });
    await page.locator("#screen-run .main-shell-tab[data-shell-space='create']").click();
    await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 10_000 });
    await page.locator("#btn-editor-lesson-plan").click();
    state = await readUiState(page);
    assert(state.planCode.includes("새 비교 질문 코드반영"), "reopen did not keep saved lesson plan");

    await page.reload({ waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
    await page.locator("#screen-run .main-shell-tab[data-shell-space='create']").click();
    await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 10_000 });
    await page.locator("#btn-editor-lesson-plan").click();
    state = await readUiState(page);
    assert(state.planCode.includes("새 비교 질문 코드반영"), "page reload did not restore saved lesson plan");

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
