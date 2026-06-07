#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_numeric_track_report_export: ok";
const TARGET_ID = "rep_math_function_line_v1";
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
      && /\/(?:graph|table|space2d|maegim_control)\.json$/i.test(pathname)
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

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");

  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "screens/browse.js",
    "numeric_curriculum_track.js",
    "../lessons/index.json",
    "../lessons/active_allowlist.detjson",
  ]) {
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
    await context.addInitScript(() => {
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          writeText: async (text) => {
            window.__SEAMGRIM_TEST_CLIPBOARD_TEXT__ = String(text ?? "");
          },
        },
      });
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
    await page.waitForSelector("#lesson-card-grid");
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT__?.schema === "seamgrim.numeric_track_report_export.v1");

    const report = await page.evaluate(() => window.__SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT__);
    const reportText = await page.evaluate(() => window.__SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT_TEXT__);
    assert(report.track_id === "studio_numeric_curriculum_track_v1", `track id mismatch: ${report.track_id}`);
    assert(report.lesson_count === 3, `lesson count mismatch: ${report.lesson_count}`);
    assert(report.module_count === 5, `module count mismatch: ${report.module_count}`);
    assert(report.evidence_pack_count >= 6, `evidence pack count too small: ${report.evidence_pack_count}`);
    assert(report.lessons.some((lesson) => lesson.lesson_id === TARGET_ID), "target lesson missing from report");
    assert(report.evidence_packs.includes("numeric_root_finding_bisection_v1"), "root finding evidence missing");
    assert(report.evidence_packs.includes("connect_flow_v1v_closure_v1"), "connect flow evidence missing");
    assert(String(reportText).includes("schema\tseamgrim.numeric_track_report_export.v1"), "report text schema missing");
    assert(String(reportText).includes("lesson_id\ttitle\tmodule_labels\tevidence_packs"), "lesson table header missing");
    assert(String(reportText).includes("module_id\tlabel\tlesson_count\tlesson_ids"), "module table header missing");
    assert(!String(reportText).endsWith("\n"), "report text must not have trailing newline");

    await page.click("#btn-filter-numeric-track");
    await page.waitForSelector(`.lesson-card[data-lesson-id="${TARGET_ID}"]`);
    await page.click(`.lesson-card[data-lesson-id="${TARGET_ID}"]`);
    await page.waitForFunction(() => {
      const button = document.querySelector("#btn-copy-numeric-track-report");
      return button && !button.classList.contains("hidden") && button.dataset.active === "1";
    });
    await page.click("#btn-copy-numeric-track-report");
    await page.waitForFunction(() => String(window.__SEAMGRIM_TEST_CLIPBOARD_TEXT__ ?? "").includes("seamgrim.numeric_track_report_export.v1"));
    const copied = await page.evaluate(() => window.__SEAMGRIM_TEST_CLIPBOARD_TEXT__);
    assert(String(copied).includes("rep_math_function_line_v1"), "copied report missing math lesson");
    assert(String(copied).includes("rep_phys_projectile_xy_v1"), "copied report missing physics lesson");
    assert(String(copied).includes("rep_econ_supply_demand_tax_v1"), "copied report missing econ lesson");

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
