#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  return "application/octet-stream";
}

async function serve(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (request, response) => {
    try {
      const url = new URL(request.url || "/", "http://127.0.0.1");
      const relative = decodeURIComponent(url.pathname === "/"
        ? "/solutions/seamgrim_ui_mvp/ui/workbench/index.html"
        : url.pathname).replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, relative);
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
        response.writeHead(403).end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      response.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
      response.end(bytes);
    } catch (_) {
      response.writeHead(404).end("not found");
    }
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert(address && typeof address !== "string", "server address unavailable");
  return { server, baseUrl: `http://127.0.0.1:${address.port}` };
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const { server, baseUrl } = await serve(root);
let browser;
const pageErrors = [];
try {
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, locale: "ko-KR" });
  page.on("pageerror", (error) => pageErrors.push(error.message));
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.readiness?.initialization === "ready");

  const warningSource = [
    "매틱:움직씨 = {",
    "  채비 { 값:수 <- 0. }.",
    "  값 <- 1.",
    "  값 보여주기.",
    "}",
  ].join("\n");
  await page.fill("[data-testid='source-editor']", warningSource);
  const beforeWarning = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  const warningStarted = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.startRun());
  const afterWarning = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(warningStarted === true, "authoritative LANGUAGE_WARNING blocked a valid run");
  assert(afterWarning.runs.length === beforeWarning.runs.length + 1, "warning run record missing");
  assert(afterWarning.sourceReadiness.authority_status === "shared_frontend", "warning authority is not shared frontend");
  assert(afterWarning.sourceReadiness.analysis_outcome === "ACCEPTED", "warning source was not accepted");
  assert(afterWarning.sourceReadiness.code === "W_CHAEBI_REDUNDANT_TOP_REASSIGN", `warning code drift: ${afterWarning.sourceReadiness.code}`);
  assert(afterWarning.sourceReadiness.ready === true && !afterWarning.sourceReadiness.run_blocked, "warning changed success to error");
  if (afterWarning.executionLifecycle.status === "running" || afterWarning.executionLifecycle.status === "paused") {
    await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.stopRun());
  }

  const invalidSource = "매틱:움직씨 = { 값:수 <- . }";
  await page.fill("[data-testid='source-editor']", invalidSource);
  const beforeInvalid = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  const invalidStarted = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.startRun());
  const afterInvalid = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(invalidStarted === false, "authoritative REJECTED was overridden to success");
  assert(afterInvalid.runtimeStatus === "source-blocked", "rejected source did not fail closed");
  assert(afterInvalid.runs.length === beforeInvalid.runs.length, "rejected source created a run record");
  assert(afterInvalid.sourceReadiness.authority_status === "shared_frontend", "invalid authority is not shared frontend");
  assert(afterInvalid.sourceReadiness.analysis_outcome === "REJECTED", "invalid source outcome drift");
  assert(afterInvalid.sourceReadiness.code === "E_PARSE", `invalid code drift: ${afterInvalid.sourceReadiness.code}`);
  assert(afterInvalid.sourceReadiness.span?.encoding === "utf-8-byte", "authoritative UTF-8 span missing");

  assert(pageErrors.length === 0, `page errors: ${JSON.stringify(pageErrors)}`);
  console.log(JSON.stringify({
    schema: "seamgrim.workbench.source_analysis_t4.browser_result.v1",
    warning_run_started: warningStarted,
    warning_code: afterWarning.sourceReadiness.code,
    warning_authority: afterWarning.sourceReadiness.authority_status,
    rejected_run_started: invalidStarted,
    rejected_code: afterInvalid.sourceReadiness.code,
    rejected_outcome: afterInvalid.sourceReadiness.analysis_outcome,
    rejected_run_record_atomicity: afterInvalid.runs.length === beforeInvalid.runs.length,
    utf8_span_preserved: afterInvalid.sourceReadiness.span?.encoding === "utf-8-byte",
    page_error_count: pageErrors.length,
    result: "PASS",
  }));
} finally {
  if (browser) await browser.close();
  await closeServer(server);
}
