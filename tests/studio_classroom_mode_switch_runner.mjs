#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_classroom_mode_switch: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) throw new Error(`missing file: ${file}`);
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
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/index.html" : url.pathname);
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
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
    if ((pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/")) && /\/(?:graph|table|space2d|text|maegim_control)\.(?:json|md)$/i.test(pathname)) return true;
  } catch (_) {
    return false;
  }
  return false;
}

async function waitVisible(page, selector) {
  await page.waitForFunction((sel) => {
    const node = document.querySelector(sel);
    return node && !String(node.className ?? "").split(/\s+/).includes("hidden");
  }, selector);
}

async function readModeState(page) {
  return page.evaluate(() => {
    const text = (sel) => document.querySelector(sel)?.textContent?.trim() ?? "";
    const value = (sel) => document.querySelector(sel)?.dataset?.value ?? "";
    const pressed = (sel) => document.querySelector(sel)?.getAttribute("aria-pressed") ?? "";
    return {
      mode: document.querySelector("[data-classroom-mode-switch]")?.dataset?.mode ?? "",
      onboarding: text("[data-run-preset-onboarding]"),
      onboardingValue: value("[data-run-preset-onboarding]"),
      studentPressed: pressed("[data-classroom-mode='student']"),
      teacherPressed: pressed("[data-classroom-mode='teacher']"),
      status: text("#run-onboarding-status"),
      rail: window.__SEAMGRIM_RUN_PRESET_RAIL__ ?? null,
      payload: window.__STUDIO_CLASSROOM_MODE_SWITCH__ ?? null,
    };
  });
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of ["index.html", "app.js", "styles.css", "screens/browse.js", "screens/run.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("console", (msg) => {
      const text = String(msg.text() ?? "");
      if (msg.type() === "error" && !text.includes("Failed to load resource") && !text.includes("[RunScreen.restart] wasm execution failed")) {
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400 && !res.url().endsWith("/favicon.ico") && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-browse");
    await page.waitForSelector(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='student']");
    await page.click(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='student']");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.onboarding_profile === "student");
    const initialStudent = await readModeState(page);
    assert(initialStudent.mode === "student", `initial mode mismatch: ${initialStudent.mode}`);
    assert(initialStudent.studentPressed === "true", "student button should be active after student launch");
    assert(initialStudent.teacherPressed === "false", "teacher button should be inactive after student launch");

    await page.click("[data-classroom-mode='teacher']");
    await page.waitForFunction(() => window.__STUDIO_CLASSROOM_MODE_SWITCH__?.mode === "teacher");
    const teacher = await readModeState(page);
    assert(teacher.mode === "teacher", `teacher mode mismatch: ${teacher.mode}`);
    assert(teacher.onboarding === "교사 시작", `teacher onboarding text mismatch: ${teacher.onboarding}`);
    assert(teacher.onboardingValue === "teacher", `teacher onboarding value mismatch: ${teacher.onboardingValue}`);
    assert(teacher.studentPressed === "false", "student button should deactivate after teacher click");
    assert(teacher.teacherPressed === "true", "teacher button should activate after teacher click");
    assert(teacher.status.includes("교사 시작 적용"), `teacher status mismatch: ${teacher.status}`);
    assert(teacher.payload?.schema === "seamgrim.classroom_mode_switch.v1", "teacher payload schema mismatch");
    assert(teacher.payload?.applied === true, "teacher payload applied mismatch");
    assert(teacher.payload?.rail_onboarding_profile === "teacher", "teacher payload rail profile mismatch");
    assert(teacher.payload?.teacher_active === true && teacher.payload?.student_active === false, "teacher payload active flags mismatch");
    assert(teacher.payload?.account_required === false && teacher.payload?.cloud_sync === false && teacher.payload?.permission_system === false, "teacher payload boundary mismatch");

    await page.click("[data-classroom-mode='student']");
    await page.waitForFunction(() => window.__STUDIO_CLASSROOM_MODE_SWITCH__?.mode === "student");
    const student = await readModeState(page);
    assert(student.mode === "student", `student mode mismatch: ${student.mode}`);
    assert(student.onboarding === "학생 시작", `student onboarding text mismatch: ${student.onboarding}`);
    assert(student.onboardingValue === "student", `student onboarding value mismatch: ${student.onboardingValue}`);
    assert(student.studentPressed === "true", "student button should activate after student click");
    assert(student.teacherPressed === "false", "teacher button should deactivate after student click");
    assert(student.status.includes("학생 시작 적용"), `student status mismatch: ${student.status}`);
    assert(student.payload?.mode === "student", "student payload mode mismatch");
    assert(student.payload?.rail_onboarding_profile === "student", "student payload rail profile mismatch");
    assert(student.payload?.student_active === true && student.payload?.teacher_active === false, "student payload active flags mismatch");

    if (failures.length > 0) throw new Error(failures.join("\n"));
    await context.close();
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
