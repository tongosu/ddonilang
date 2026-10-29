#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3b_foundation12_model_alignment: ok";
const ROOT = process.cwd();
const WORLDS = [
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
];

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
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/index.html" : url.pathname);
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
    const optionalLessonDataPrefix =
      pathname.startsWith("/lessons/")
      || pathname.startsWith("/lessons_rewrite_v1/")
      || pathname.startsWith("/seed_lessons_v1/");
    if (optionalLessonDataPrefix && /\/(?:graph|table|space2d|maegim_control)\.json$/i.test(pathname)) return true;
    if (optionalLessonDataPrefix && /\/(?:text\.md)$/i.test(pathname)) return true;
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

function parsePayloadJson(text) {
  const raw = String(text ?? "").trim();
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch (_) {
    return [];
  }
}

function assertNoBadNumbers(lessonId, rows) {
  rows.forEach((row) => {
    const text = String(row?.value ?? "").trim();
    assert(!/(?:^|[^A-Za-z])(?:NaN|Infinity|-Infinity)(?:$|[^A-Za-z])/u.test(text), `${lessonId}: non-finite observation ${text}`);
  });
}

async function waitForRunReady(page, lessonId) {
  await page.waitForSelector("#screen-run:not(.hidden)", { timeout: 20_000 });
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.title ?? "").includes("blake3:"), null, { timeout: 30_000 });
  await page.waitForFunction(() => {
    const body = document.querySelector("[data-testid='lesson-observe-output-body']") ?? document.querySelector("#run-observe-output-body");
    return Number(body?.dataset?.observationPayloadCount ?? 0) > 0;
  }, null, { timeout: 30_000 });
  const route = new URL(page.url());
  assert(route.searchParams.get("lesson") === lessonId, `${lessonId}: route lesson mismatch ${page.url()}`);
  assert(route.searchParams.get("tab") === "studio", `${lessonId}: studio tab missing`);
}

async function readEvidence(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const body = document.querySelector("[data-testid='lesson-observe-output-body']") ?? document.querySelector("#run-observe-output-body");
    const errorPanel = document.querySelector("[data-testid='lesson-error-panel']");
    const payload = String(body?.dataset?.observationPayloadJson ?? "");
    const controls = [...document.querySelectorAll("input[type='number'][data-control-name]")].map((input) => ({
      name: input.getAttribute("data-control-name") ?? "",
      value: input.value,
      min: input.getAttribute("min") ?? "",
      max: input.getAttribute("max") ?? "",
      step: input.getAttribute("step") ?? "",
    }));
    return {
      status: root?.dataset?.engineStatus ?? "",
      mainVisualMode: root?.dataset?.mainVisualMode ?? "",
      roles: root?.dataset?.mainViewRoles ?? "",
      observationPayloadCount: Number(body?.dataset?.observationPayloadCount ?? 0),
      observationPayloadJson: payload,
      errorHidden: errorPanel?.classList.contains("hidden") ?? false,
      errorText: (errorPanel?.textContent ?? "").trim(),
      hashTitle: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      controls,
      horizontalOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    };
  });
}

async function sampleControlBoundary(page, lessonId, control) {
  const min = Number(control.min);
  const max = Number(control.max);
  if (!Number.isFinite(min) || !Number.isFinite(max) || min === max) return [];
  const samples = [min, max];
  const results = [];
  for (const value of samples) {
    await page.click("#run-tab-btn-console");
    await page.waitForSelector("#run-tab-panel-console:not(.hidden) #run-slider-area:not(.hidden)", { timeout: 10_000 });
    const escapedName = String(control.name).replace(/\\/g, "\\\\").replace(/"/g, '\\"');
    const selector = `input[type='number'][data-control-name="${escapedName}"]`;
    const input = page.locator(selector).first();
    await input.fill(String(value));
    await input.dispatchEvent("change");
    await page.waitForTimeout(700);
    const evidence = await readEvidence(page);
    assert(evidence.errorHidden, `${lessonId}: boundary ${control.name}=${value} showed error ${evidence.errorText}`);
    const payload = parsePayloadJson(evidence.observationPayloadJson);
    assert(payload.length > 0, `${lessonId}: boundary ${control.name}=${value} observation missing`);
    assertNoBadNumbers(lessonId, payload);
    results.push({ name: control.name, value, payloadCount: payload.length });
  }
  return results;
}

async function verifyLesson(browser, baseUrl, lessonId) {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  const failures = [];
  installFailureCapture(page, failures);
  try {
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&catalog_mode=full&lesson=${lessonId}`, {
      waitUntil: "domcontentloaded",
    });
    await waitForRunReady(page, lessonId);
    const evidence = await readEvidence(page);
    assert(evidence.errorHidden, `${lessonId}: error panel visible ${evidence.errorText}`);
    assert(evidence.hashTitle.includes("blake3:"), `${lessonId}: hash missing`);
    assert(evidence.observationPayloadCount > 0, `${lessonId}: observation payload missing`);
    assert(evidence.horizontalOverflow <= 1, `${lessonId}: horizontal overflow ${evidence.horizontalOverflow}`);
    const payload = parsePayloadJson(evidence.observationPayloadJson);
    assert(payload.length === evidence.observationPayloadCount, `${lessonId}: payload count mismatch`);
    assertNoBadNumbers(lessonId, payload);
    const controls = evidence.controls.filter((control) => control.name);
    const boundarySamples = [];
    for (const control of controls.slice(0, 2)) {
      boundarySamples.push(...await sampleControlBoundary(page, lessonId, control));
    }
    assert(!failures.length, `${lessonId}: browser failures ${failures.join(" | ")}`);
    return {
      lessonId,
      status: evidence.status,
      mainVisualMode: evidence.mainVisualMode,
      roles: evidence.roles,
      observationPayloadCount: evidence.observationPayloadCount,
      controls: controls.map((control) => control.name),
      boundarySamples,
    };
  } finally {
    await page.close();
  }
}

async function main() {
  const { server, baseUrl } = await createServer(ROOT);
  const browser = await chromium.launch({ headless: true });
  try {
    const rows = [];
    for (const lessonId of WORLDS) {
      rows.push(await verifyLesson(browser, baseUrl, lessonId));
    }
    assert(rows.length === 12, `expected 12 rows, got ${rows.length}`);
    console.log(`${OK} ${JSON.stringify({ worlds: rows.length, observation_payloads: rows.map((row) => row.observationPayloadCount) })}`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
