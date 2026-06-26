#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_lesson_run_preset_rail: ok";
const UNSAFE_BROWSER_PORTS = new Set([
  1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
  101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161,
  179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563,
  587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
  6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
]);

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
      const rel = rawPath.replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, rel);
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
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
      if (UNSAFE_BROWSER_PORTS.has(address.port)) {
        server.close(() => {
          createServer(root).then(resolve, reject);
        });
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
      && /\/(?:graph|table|space2d)\.json$/i.test(pathname)
    ) return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/text\.md$/i.test(pathname)
    ) return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/maegim_control\.json$/i.test(pathname)
    ) return true;
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

async function readRail(page) {
  return page.evaluate(() => {
    const rail = document.querySelector("[data-run-preset-rail]");
    const text = (sel) => rail?.querySelector?.(sel)?.textContent?.trim() ?? "";
    const value = (sel) => rail?.querySelector?.(sel)?.dataset?.value ?? "";
    return {
      railCount: document.querySelectorAll("[data-run-preset-rail]").length,
      launch: text("[data-run-preset-launch-kind]"),
      launchValue: value("[data-run-preset-launch-kind]"),
      onboarding: text("[data-run-preset-onboarding]"),
      onboardingValue: value("[data-run-preset-onboarding]"),
      layout: text("[data-run-preset-layout]"),
      layoutValue: value("[data-run-preset-layout]"),
      views: text("[data-run-preset-views]"),
      viewsValue: value("[data-run-preset-views]"),
      model: window.__SEAMGRIM_RUN_PRESET_RAIL__ ?? null,
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
    const context = await browser.newContext({
      viewport: { width: 1360, height: 860 },
      locale: "ko-KR",
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        if (String(msg.text() ?? "").includes("Failed to load resource")) return;
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400) {
        if (res.status() === 404 && isAllowedFallback404(res.url())) return;
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-browse");
    const presetRailLessonSelector = ".lesson-card[data-lesson-id='rep_math_function_line_v1']";
    await page.waitForSelector(`${presetRailLessonSelector} .card-launch-btn[data-launch-profile='student']`);

    await page.click(`${presetRailLessonSelector} .card-launch-btn[data-launch-profile='student']`);
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.onboarding_profile === "student");
    const studentRail = await readRail(page);
    assert(studentRail.railCount === 1, `run preset rail count mismatch: ${studentRail.railCount}`);
    assert(studentRail.launch === "탐색 선택", `student launch label mismatch: ${studentRail.launch}`);
    assert(studentRail.launchValue === "browse_select_student", `student launch value mismatch: ${studentRail.launchValue}`);
    assert(studentRail.onboarding === "학생 시작", `student onboarding label mismatch: ${studentRail.onboarding}`);
    assert(studentRail.onboardingValue === "student", `student onboarding value mismatch: ${studentRail.onboardingValue}`);
    assert(studentRail.layout.startsWith("화면:"), `student layout label mismatch: ${studentRail.layout}`);
    assert(studentRail.views.startsWith("결과 확인:"), `student views label mismatch: ${studentRail.views}`);
    assert(studentRail.model?.schema === "seamgrim.run_preset_rail.v1", "student rail model schema mismatch");

    await page.click('#screen-run .main-shell-tab[data-main-tab-target="browse"]');
    await waitVisible(page, "#screen-browse");
    await page.waitForSelector(`${presetRailLessonSelector} .card-launch-btn[data-launch-profile='teacher']`);
    await page.click(`${presetRailLessonSelector} .card-launch-btn[data-launch-profile='teacher']`);
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.onboarding_profile === "teacher");
    const teacherRail = await readRail(page);
    assert(teacherRail.launch === "탐색 선택", `teacher launch label mismatch: ${teacherRail.launch}`);
    assert(teacherRail.launchValue === "browse_select_teacher", `teacher launch value mismatch: ${teacherRail.launchValue}`);
    assert(teacherRail.onboarding === "교사 시작", `teacher onboarding label mismatch: ${teacherRail.onboarding}`);
    assert(teacherRail.onboardingValue === "teacher", `teacher onboarding value mismatch: ${teacherRail.onboardingValue}`);
    assert(teacherRail.model?.onboarding_label === "교사 시작", "teacher rail model onboarding mismatch");

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
