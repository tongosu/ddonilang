#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_lesson_library_curation: ok";
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
    const pathname = url.pathname;
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d)\.json$/i.test(pathname)
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
    "lesson_loader_contract.js",
    "lesson_library_curation.js",
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
      viewport: { width: 1280, height: 820 },
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
    await page.waitForFunction(() => window.__SEAMGRIM_LESSON_LIBRARY_CURATION__?.schema === "seamgrim.lesson_library_curation.v1");

    const snapshot = await page.evaluate(() => window.__SEAMGRIM_LESSON_LIBRARY_CURATION__);
    const text = await page.evaluate(() => window.__SEAMGRIM_LESSON_LIBRARY_CURATION_TEXT__);

    assert(snapshot.schema === "seamgrim.lesson_library_curation.v1", `schema mismatch: ${snapshot.schema}`);
    assert(snapshot.mode === "reps_only", `allowlist mode mismatch: ${snapshot.mode}`);
    assert(snapshot.catalog_mode === "reps_only", `catalog mode mismatch: ${snapshot.catalog_mode}`);
    assert(snapshot.allowlist_count === 15, `allowlist count mismatch: ${snapshot.allowlist_count}`);
    assert(snapshot.active_count === 15, `active count mismatch: ${snapshot.active_count}`);
    assert(Array.isArray(snapshot.missing_allowlist_ids) && snapshot.missing_allowlist_ids.length === 0, "missing allowlist ids not empty");
    assert(Array.isArray(snapshot.duplicate_allowlist_ids) && snapshot.duplicate_allowlist_ids.length === 0, "duplicate allowlist ids not empty");
    assert(snapshot.subject_counts?.cs === 8, `cs subject count mismatch: ${JSON.stringify(snapshot.subject_counts)}`);
    assert(snapshot.subject_counts?.econ === 2, `econ subject count mismatch: ${JSON.stringify(snapshot.subject_counts)}`);
    assert(snapshot.subject_counts?.math === 2, `math subject count mismatch: ${JSON.stringify(snapshot.subject_counts)}`);
    assert(snapshot.subject_counts?.physics === 2, `physics subject count mismatch: ${JSON.stringify(snapshot.subject_counts)}`);
    assert(snapshot.subject_counts?.science === 1, `science subject count mismatch: ${JSON.stringify(snapshot.subject_counts)}`);
    assert(snapshot.grade_counts?.middle === 14, `middle grade count mismatch: ${JSON.stringify(snapshot.grade_counts)}`);
    assert(snapshot.grade_counts?.high === 1, `high grade count mismatch: ${JSON.stringify(snapshot.grade_counts)}`);
    assert(snapshot.source_counts?.official === 15, `official source count mismatch: ${JSON.stringify(snapshot.source_counts)}`);
    assert(snapshot.required_view_counts?.text === 12, `text view count mismatch: ${JSON.stringify(snapshot.required_view_counts)}`);
    assert(snapshot.required_view_counts?.table === 12, `table view count mismatch: ${JSON.stringify(snapshot.required_view_counts)}`);
    assert(snapshot.required_view_counts?.graph === 7, `graph view count mismatch: ${JSON.stringify(snapshot.required_view_counts)}`);
    assert(snapshot.required_view_counts?.space2d === 2, `space2d view count mismatch: ${JSON.stringify(snapshot.required_view_counts)}`);
    assert(typeof text === "string" && text.includes("active_count\t15"), "curation text active_count missing");
    assert(text.includes("subject_counts\tcs=8|econ=2|math=2|physics=2|science=1"), `curation text subject counts mismatch: ${text}`);

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
