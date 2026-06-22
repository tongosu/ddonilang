#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_release_approval_continuity_export_action: ok";
const REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다";

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

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of ["index.html", "app.js", "styles.css", "screens/browse.js", "screens/run.js", "studio_lesson_publication_review_surface.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    await context.addInitScript(() => {
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          async writeText(value) {
            window.__STUDIO_RELEASE_APPROVAL_CONTINUITY_COPIED_TEXT__ = String(value ?? "");
          },
        },
      });
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error" && !String(msg.text() ?? "").includes("Failed to load resource")) {
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
    await waitVisible(page, "[data-run-approval-continuity-export]");
    await page.click("#btn-run-approval-continuity-copy");
    await page.waitForFunction(() => window.__STUDIO_RELEASE_APPROVAL_CONTINUITY_EXPORT_ACTION__?.copied === true);

    const state = await page.evaluate(() => ({
      schema: document.querySelector("[data-run-approval-continuity-export]")?.dataset?.schema ?? "",
      state: document.querySelector("[data-run-approval-continuity-export]")?.dataset?.state ?? "",
      blockedCount: document.querySelector("[data-run-approval-continuity-export]")?.dataset?.blockedCount ?? "",
      meta: document.querySelector("[data-run-approval-continuity-meta]")?.textContent?.trim() ?? "",
      metaValue: document.querySelector("[data-run-approval-continuity-meta]")?.dataset?.value ?? "",
      text: document.querySelector("[data-run-approval-continuity-text]")?.textContent ?? "",
      copied: window.__STUDIO_RELEASE_APPROVAL_CONTINUITY_COPIED_TEXT__ ?? "",
      payload: window.__STUDIO_RELEASE_APPROVAL_CONTINUITY_EXPORT_ACTION__ ?? null,
    }));
    const copiedPayload = JSON.parse(state.copied);

    assert(state.schema === "seamgrim.release_approval_continuity_export_action.v1", `schema mismatch: ${state.schema}`);
    assert(state.state === "AWAIT_EXPLICIT_RELEASE_APPROVAL", `state mismatch: ${state.state}`);
    assert(Number(state.blockedCount) >= 12, `blocked count mismatch: ${state.blockedCount}`);
    assert(state.meta.includes("교사 시작"), `meta mismatch: ${state.meta}`);
    assert(state.text.includes(REQUIRED_APPROVAL), "preview approval phrase missing");
    assert(state.text.includes("release_approval_claim\tfalse"), "preview approval boundary missing");
    assert(state.text.includes("release_execution_claim\tfalse"), "preview execution boundary missing");
    assert(copiedPayload.__종류 === "studio_release_approval_continuity_export_payload", "clipboard kind mismatch");
    assert(copiedPayload.schema === "seamgrim.release_approval_continuity_export_action.v1", "clipboard schema mismatch");
    assert(copiedPayload.required_approval_phrase === REQUIRED_APPROVAL, "approval phrase mismatch");
    assert(copiedPayload.generic_next_dev_request_is_approval === false, "generic approval mismatch");
    assert(copiedPayload.next_state === "AWAIT_EXPLICIT_RELEASE_APPROVAL", "next state mismatch");
    assert(Array.isArray(copiedPayload.blocked_until_approval) && copiedPayload.blocked_until_approval.includes("github_release_create"), "blocked actions mismatch");
    assert(copiedPayload.release_approval_claim === false, "release approval boundary mismatch");
    assert(copiedPayload.release_execution_claim === false, "release execution boundary mismatch");
    assert(copiedPayload.public_release_claim === false, "public release boundary mismatch");
    assert(copiedPayload.github_release_claim === false, "github release boundary mismatch");
    assert(copiedPayload.public_upload_claim === false, "public upload boundary mismatch");
    assert(copiedPayload.registry_publish_claim === false, "registry boundary mismatch");
    assert(copiedPayload.cloud_sync === false, "cloud boundary mismatch");
    assert(copiedPayload.account_required === false, "account boundary mismatch");
    assert(copiedPayload.registry_seed_payload?.schema === "seamgrim.registry_share_seed_export_action.v1", "registry seed payload mismatch");
    assert(state.payload?.schema === "seamgrim.release_approval_continuity_export_action.v1", "instrumentation schema mismatch");
    assert(state.payload?.copied === true, "instrumentation copied mismatch");
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
