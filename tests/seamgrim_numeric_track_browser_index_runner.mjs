#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_numeric_track_browser_index: ok";
const TRACK_IDS = [
  "rep_math_function_line_v1",
  "rep_phys_projectile_xy_v1",
  "rep_econ_supply_demand_tax_v1",
];
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
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_INDEX__?.schema === "seamgrim.numeric_track_index.v1");

    const snapshot = await page.evaluate(() => window.__SEAMGRIM_NUMERIC_TRACK_INDEX__);
    const text = await page.evaluate(() => window.__SEAMGRIM_NUMERIC_TRACK_INDEX_TEXT__);
    assert(snapshot.track_id === "studio_numeric_curriculum_track_v1", `track id mismatch: ${snapshot.track_id}`);
    assert(snapshot.module_count === 5, `module count mismatch: ${snapshot.module_count}`);
    assert(snapshot.active_lesson_count === 3, `active lesson count mismatch: ${snapshot.active_lesson_count}`);
    assert(Array.isArray(snapshot.missing_lesson_ids) && snapshot.missing_lesson_ids.length === 0, "missing numeric track anchors");
    assert(TRACK_IDS.every((lessonId) => snapshot.lesson_ids.includes(lessonId)), "snapshot lesson ids incomplete");
    assert(typeof text === "string" && text.includes("active_lesson_count\t3"), "numeric track text count missing");

    await page.click("#btn-filter-numeric-track");
    await page.waitForFunction((ids) => {
      const button = document.querySelector("#btn-filter-numeric-track");
      const cards = Array.from(document.querySelectorAll(".lesson-card"));
      const cardIds = cards.map((card) => card.dataset.lessonId).sort();
      return button?.dataset?.active === "1"
        && cards.length === ids.length
        && ids.every((id) => cardIds.includes(id))
        && cards.every((card) => card.textContent.includes("수치트랙"));
    }, [...TRACK_IDS].sort());

    const filtered = await page.evaluate(() => ({
      active: document.querySelector("#btn-filter-numeric-track")?.dataset?.active ?? "",
      count: document.querySelector("#btn-filter-numeric-track")?.dataset?.count ?? "",
      ids: Array.from(document.querySelectorAll(".lesson-card")).map((card) => card.dataset.lessonId).sort(),
      badges: Array.from(document.querySelectorAll(".badge-numeric-track")).map((node) => node.textContent.trim()),
    }));
    assert(filtered.active === "1", "numeric track button did not activate");
    assert(filtered.count === "3", `numeric track button count mismatch: ${filtered.count}`);
    assert(JSON.stringify(filtered.ids) === JSON.stringify([...TRACK_IDS].sort()), `filtered ids mismatch: ${JSON.stringify(filtered.ids)}`);
    assert(filtered.badges.length === 3, `numeric track badge count mismatch: ${filtered.badges.length}`);

    await page.click("#btn-filter-numeric-track");
    await page.waitForFunction(() => {
      const button = document.querySelector("#btn-filter-numeric-track");
      return button?.dataset?.active === "0" && document.querySelectorAll(".lesson-card").length > 3;
    });

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
