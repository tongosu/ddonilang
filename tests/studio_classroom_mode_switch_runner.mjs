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

function isAllowedRequestFailure(req) {
  const errorText = String(req.failure()?.errorText || "");
  if (!errorText.includes("net::ERR_ABORTED")) return false;
  const urlText = String(req.url() || "");
  return urlText.includes("/wasm_ddn_wrapper.js") || urlText.endsWith(".wasm") || urlText.includes(".wasm?");
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
    const visible = (sel) => {
      const node = document.querySelector(sel);
      if (!node) return false;
      const style = getComputedStyle(node);
      return style.display !== "none" && style.visibility !== "hidden" && !node.classList.contains("hidden");
    };
    return {
      mode: document.querySelector("[data-classroom-mode-switch]")?.dataset?.mode ?? "",
      modeAccess: document.querySelector("[data-classroom-mode-switch]")?.dataset?.access ?? "",
      switchVisible: visible("[data-classroom-mode-switch]"),
      onboarding: text("[data-run-preset-onboarding]"),
      onboardingValue: value("[data-run-preset-onboarding]"),
      studentPressed: pressed("[data-classroom-mode='student']"),
      teacherPressed: pressed("[data-classroom-mode='teacher']"),
      classroomReportVisible: visible("[data-run-classroom-report-export]"),
      localPackageVisible: visible("[data-run-local-package-export]"),
      teacherReportCopyVisible: visible("#btn-run-teacher-report-copy"),
      teacherPackageCopyVisible: visible("#btn-run-teacher-package-copy"),
      teacherPackageDownloadVisible: visible("#btn-run-teacher-package-download"),
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
  const contractPath = path.join(root, "pack", "studio_classroom_mode_switch_v1", "contract.detjson");
  for (const rel of ["index.html", "app.js", "styles.css", "screens/browse.js", "screens/run.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }
  await requireFile(contractPath);

  const contract = JSON.parse(await fs.readFile(contractPath, "utf8"));
  assert(contract.status === "superseded", `switch pack status mismatch: ${contract.status}`);
  assert(contract.superseded_by === "SEAMGRIM_WAVE2A_PRODUCT_IA_V1", `switch superseded_by mismatch: ${contract.superseded_by}`);
  assert(contract.runtime_claim === false && contract.product_code_change === false, "switch superseded pack must not claim runtime/product code");
  assert(contract.browser_runner === null && contract.workflow_schema === null, "switch superseded pack must not claim active runner/schema");
  assert(contract.workflow_claim === "runtime_student_teacher_toggle_absent", `switch workflow claim mismatch: ${contract.workflow_claim}`);
  assert(contract.student_mode_click === false && contract.teacher_mode_click === false, "switch pack must not claim runtime click behavior");
  for (const key of ["roadmap_v2_closed", "roadmap_v2_total", "roadmap_v2_percent"]) {
    assert(!(key in contract), `superseded switch pack must not own global progress: ${key}`);
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
    page.on("requestfailed", (req) => {
      if (isAllowedRequestFailure(req)) return;
      failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`);
    });
    page.on("response", (res) => {
      if (res.status() >= 400 && !res.url().endsWith("/favicon.ico") && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-browse");
    await page.waitForSelector("#screen-browse .lesson-card[data-lesson-id]");
    await page.click("#screen-browse .lesson-card[data-lesson-id]");
    await page.waitForSelector("#btn-open-in-studio");
    await page.click("#btn-open-in-studio");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.schema === "seamgrim.run_preset_rail.v1");
    const initialStudent = await readModeState(page);
    assert(initialStudent.mode === "", `runtime role mode must be absent after student launch: ${initialStudent.mode}`);
    assert(initialStudent.modeAccess === "", `runtime role access must be absent after student launch: ${initialStudent.modeAccess}`);
    assert(initialStudent.switchVisible === false, "student launch must not render classroom mode switch");
    assert(initialStudent.studentPressed === "", "student runtime role button must be absent after student launch");
    assert(initialStudent.teacherPressed === "", "teacher runtime role button must be absent after student launch");
    assert(initialStudent.payload === null, "legacy classroom mode switch payload must be absent after student launch");
    assert(initialStudent.rail?.launch_kind === "browse_select", "learn launch kind mismatch");
    assert(initialStudent.rail?.onboarding_profile === "", "runtime student profile must not survive in rail");
    assert(initialStudent.onboarding === "기본 시작", `default onboarding text mismatch: ${initialStudent.onboarding}`);
    assert(initialStudent.onboardingValue === "default", `default onboarding value mismatch: ${initialStudent.onboardingValue}`);
    assert(initialStudent.classroomReportVisible === false, "student mode must hide classroom report export");
    assert(initialStudent.localPackageVisible === false, "student mode must hide local package export");
    assert(initialStudent.teacherReportCopyVisible === false, "student mode must hide teacher report copy action");
    assert(initialStudent.teacherPackageCopyVisible === false, "student mode must hide teacher package copy action");
    assert(initialStudent.teacherPackageDownloadVisible === false, "student mode must hide teacher package download action");

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-browse");
    await page.waitForSelector("#screen-browse .lesson-card[data-lesson-id]");
    await page.click("#screen-browse .lesson-card[data-lesson-id]");
    await page.waitForSelector("#btn-open-in-studio-teacher");
    await page.click("#btn-open-in-studio-teacher");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.schema === "seamgrim.run_preset_rail.v1");
    const lab = await readModeState(page);
    assert(lab.mode === "", `runtime role mode must be absent after lab launch: ${lab.mode}`);
    assert(lab.modeAccess === "", `runtime role access must be absent after lab launch: ${lab.modeAccess}`);
    assert(lab.switchVisible === false, "lab launch must not render classroom mode switch");
    assert(lab.onboarding === "기본 시작", `lab onboarding text mismatch: ${lab.onboarding}`);
    assert(lab.onboardingValue === "default", `lab onboarding value mismatch: ${lab.onboardingValue}`);
    assert(lab.studentPressed === "", "student runtime role button must be absent after lab launch");
    assert(lab.teacherPressed === "", "teacher runtime role button must be absent after lab launch");
    assert(lab.payload === null, "legacy classroom mode switch payload must be absent after lab launch");
    assert(lab.rail?.onboarding_profile === "", "runtime teacher profile must not survive in rail");
    assert(["browse_select", "lab"].includes(String(lab.rail?.launch_kind ?? "")), `lab launch rail kind mismatch: ${lab.rail?.launch_kind}`);

    const legacyToggle = await page.evaluate(() => ({
      switchCount: document.querySelectorAll("[data-classroom-mode-switch]").length,
      buttonCount: document.querySelectorAll("[data-classroom-mode]").length,
      payload: window.__STUDIO_CLASSROOM_MODE_SWITCH__ ?? null,
      runDataset: document.querySelector("#screen-run")?.dataset?.classroomModeAccess ?? "",
    }));
    assert(legacyToggle.switchCount === 0, `legacy switch DOM survived: ${JSON.stringify(legacyToggle)}`);
    assert(legacyToggle.buttonCount === 0, `legacy switch buttons survived: ${JSON.stringify(legacyToggle)}`);
    assert(legacyToggle.payload === null, `legacy switch payload survived: ${JSON.stringify(legacyToggle)}`);
    assert(legacyToggle.runDataset === "", `legacy runtime role dataset survived: ${JSON.stringify(legacyToggle)}`);

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
