#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_publication_prep_export_action: ok";

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
      else resolve({
        server,
        baseUrl: `http://127.0.0.1:${address.port}`,
        publicBaseUrl: `http://studio.example.test:${address.port}`,
      });
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

async function assertNonLocalAdvancedExportsBlocked(page, publicBaseUrl) {
  await page.goto(`${publicBaseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?advancedExports=1`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-browse");
  await page.waitForSelector(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='teacher']");
  await page.click(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='teacher']");
  await waitVisible(page, "#screen-run");
  await page.click("#run-tab-btn-mirror");
  await page.evaluate(() => {
    const tools = document.querySelector("#run-inspector-tools");
    if (tools) tools.open = true;
  });
  const queryState = await page.evaluate(() => {
    const visible = (selector) => {
      const node = document.querySelector(selector);
      return Boolean(node && !node.classList.contains("hidden"));
    };
    return {
      hostname: window.location.hostname,
      publicationPrepVisible: visible("[data-run-publication-prep-export]"),
      registrySeedVisible: visible("[data-run-registry-seed-export]"),
      approvalContinuityVisible: visible("[data-run-approval-continuity-export]"),
      benchmarkLtsVisible: visible("[data-run-benchmark-lts-export]"),
      educationOperationsVisible: visible("[data-run-education-operations-lts-export]"),
    };
  });
  assert(queryState.hostname === "studio.example.test", `non-local advanced export host mismatch: ${queryState.hostname}`);
  assert(queryState.publicationPrepVisible === false, "non-local query leaked publication prep export");
  assert(queryState.registrySeedVisible === false, "non-local query leaked registry seed export");
  assert(queryState.approvalContinuityVisible === false, "non-local query leaked approval continuity export");
  assert(queryState.benchmarkLtsVisible === false, "non-local query leaked benchmark LTS export");
  assert(queryState.educationOperationsVisible === false, "non-local query leaked education operations export");

  await page.addInitScript(() => {
    window.SEAMGRIM_ADVANCED_EXPORTS = true;
  });
  await page.goto(`${publicBaseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-browse");
  await page.waitForSelector(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='teacher']");
  await page.click(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='teacher']");
  await waitVisible(page, "#screen-run");
  await page.click("#run-tab-btn-mirror");
  await page.evaluate(() => {
    const tools = document.querySelector("#run-inspector-tools");
    if (tools) tools.open = true;
  });
  const globalState = await page.evaluate(() => {
    const visible = (selector) => {
      const node = document.querySelector(selector);
      return Boolean(node && !node.classList.contains("hidden"));
    };
    return {
      hostname: window.location.hostname,
      globalFlag: window.SEAMGRIM_ADVANCED_EXPORTS === true,
      publicationPrepVisible: visible("[data-run-publication-prep-export]"),
      registrySeedVisible: visible("[data-run-registry-seed-export]"),
      approvalContinuityVisible: visible("[data-run-approval-continuity-export]"),
      benchmarkLtsVisible: visible("[data-run-benchmark-lts-export]"),
      educationOperationsVisible: visible("[data-run-education-operations-lts-export]"),
    };
  });
  assert(globalState.hostname === "studio.example.test", `non-local advanced export global host mismatch: ${globalState.hostname}`);
  assert(globalState.globalFlag === true, "non-local advanced export global override setup failed");
  assert(globalState.publicationPrepVisible === false, "non-local global override leaked publication prep export");
  assert(globalState.registrySeedVisible === false, "non-local global override leaked registry seed export");
  assert(globalState.approvalContinuityVisible === false, "non-local global override leaked approval continuity export");
  assert(globalState.benchmarkLtsVisible === false, "non-local global override leaked benchmark LTS export");
  assert(globalState.educationOperationsVisible === false, "non-local global override leaked education operations export");
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
    "screens/run.js",
    "studio_local_share_package.js",
    "studio_lesson_publication_review_surface.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl, publicBaseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({
      headless: true,
      args: ["--host-resolver-rules=MAP studio.example.test 127.0.0.1"],
    });
    const context = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    await context.addInitScript(() => {
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          async writeText(value) {
            window.__STUDIO_PUBLICATION_PREP_COPIED_TEXT__ = String(value ?? "");
          },
        },
      });
    });
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

    await assertNonLocalAdvancedExportsBlocked(page, publicBaseUrl);

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?advancedExports=1`, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-browse");
    await page.waitForSelector(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='teacher']");
    await page.click(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='teacher']");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.onboarding_profile === "teacher");
    await page.click("#run-tab-btn-mirror");
    await page.evaluate(() => {
      const tools = document.querySelector("#run-inspector-tools");
      if (tools) tools.open = true;
    });
    await page.click("#btn-run-publication-prep-copy");
    await page.waitForFunction(() => window.__STUDIO_PUBLICATION_PREP_EXPORT_ACTION__?.copied === true);

    const state = await page.evaluate(() => ({
      schema: document.querySelector("[data-run-publication-prep-export]")?.dataset?.schema ?? "",
      state: document.querySelector("[data-run-publication-prep-export]")?.dataset?.state ?? "",
      candidateCount: document.querySelector("[data-run-publication-prep-export]")?.dataset?.candidateCount ?? "",
      meta: document.querySelector("[data-run-publication-prep-meta]")?.textContent?.trim() ?? "",
      metaValue: document.querySelector("[data-run-publication-prep-meta]")?.dataset?.value ?? "",
      text: document.querySelector("[data-run-publication-prep-text]")?.textContent ?? "",
      copied: window.__STUDIO_PUBLICATION_PREP_COPIED_TEXT__ ?? "",
      payload: window.__STUDIO_PUBLICATION_PREP_EXPORT_ACTION__ ?? null,
    }));
    const copiedPayload = JSON.parse(state.copied);

    assert(state.schema === "seamgrim.publication_prep_export_action.v1", `schema mismatch: ${state.schema}`);
    assert(state.state === "ready", `state mismatch: ${state.state}`);
    assert(state.candidateCount === "15", `candidate count mismatch: ${state.candidateCount}`);
    assert(state.meta.includes("교사 시작"), `meta mismatch: ${state.meta}`);
    assert(state.metaValue === "15", `meta value mismatch: ${state.metaValue}`);
    assert(state.text.startsWith("항목\t값"), "preview header mismatch");
    assert(state.text.includes("public_upload_claim\tfalse"), "preview upload boundary missing");
    assert(state.text.includes("registry_publish_claim\tfalse"), "preview registry boundary missing");
    assert(state.text.includes("active_allowlist_mutation\tfalse"), "preview allowlist boundary missing");
    assert(copiedPayload.__종류 === "studio_publication_prep_export_payload", "clipboard payload kind mismatch");
    assert(copiedPayload.schema === "seamgrim.publication_prep_export_action.v1", "clipboard schema mismatch");
    assert(copiedPayload.candidate_count === 15, "clipboard candidate count mismatch");
    assert(copiedPayload.local_package_manifest?.__종류 === "studio_local_package_manifest", "local package manifest mismatch");
    assert(copiedPayload.publication_review_surface?.schema === "ddn.studio.lesson_publication_review_surface.v1", "review surface schema mismatch");
    assert(copiedPayload.public_upload_claim === false, "upload boundary mismatch");
    assert(copiedPayload.registry_publish_claim === false, "registry boundary mismatch");
    assert(copiedPayload.cloud_sync === false, "cloud boundary mismatch");
    assert(copiedPayload.account_required === false, "account boundary mismatch");
    assert(copiedPayload.permission_system === false, "permission boundary mismatch");
    assert(copiedPayload.remote_save === false, "remote save boundary mismatch");
    assert(copiedPayload.active_allowlist_mutation === false, "allowlist boundary mismatch");
    assert(state.payload?.schema === "seamgrim.publication_prep_export_action.v1", "instrumentation schema mismatch");
    assert(state.payload?.copied === true, "instrumentation copied mismatch");
    assert(state.payload?.candidate_count === 15, "instrumentation candidate count mismatch");
    assert(state.payload?.payload_text.trim() === state.copied, "payload text clipboard mismatch");

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
