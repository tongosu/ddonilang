#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_diagnostic_fixit_integration: ok";

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
  return "text/plain; charset=utf-8";
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
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|md)$/i.test(pathname)
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
    "play_diagnostic_contract.js",
    "studio_diagnostic_fixit_preview.js",
    "studio_diagnostic_fixit_integration.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1180, height: 760 }, locale: "ko-KR" });
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    const result = await page.evaluate(async () => {
      const {
        buildDiagnosticFixitIntegration,
        formatDiagnosticFixitIntegrationText,
      } = await import("./studio_diagnostic_fixit_integration.js");
      const sourceText = [
        "#이름: 통합진단",
        "채비: {",
        "  t:수 <- 0.",
        "}.",
        "값 <- (1 + 2.",
        "받으면 { 값 보여주기. }.",
      ].join("\n");
      const workflow = buildDiagnosticFixitIntegration({
        sourceText,
        diagnostics: [
          { code: "E_BLOCK_HEADER_HASH_FORBIDDEN", message: "hash header forbidden", span: { line: 1, column: 1 } },
          { code: "E_BLOCK_HEADER_COLON_FORBIDDEN", message: "block header colon forbidden", span: { line: 2, column: 5 } },
          { code: "E_PARSE_EXPECTED_RPAREN", message: "expected rparen", span: { line: 5, column: 12 } },
          { code: "E_RECEIVE_OUTSIDE_IMJA", message: "unsupported receive", span: { line: 6, column: 1 } },
        ],
        boundary: {
          auto_apply: false,
          file_write: false,
          lsp_protocol_change: false,
          parser_frontdoor_change: false,
          runtime_claim: false,
          lesson_schema_change: false,
          active_allowlist_mutation: false,
        },
      });
      return {
        workflow,
        text: formatDiagnosticFixitIntegrationText(workflow),
        original: sourceText,
      };
    });

    const workflow = result.workflow;
    assert(workflow.schema === "seamgrim.diagnostic_fixit_integration.v1", "workflow schema mismatch");
    assert(workflow.primary_coordinate === "마-3", `primary coordinate mismatch: ${workflow.primary_coordinate}`);
    assert(workflow.support_coordinate === "타-3", `support coordinate mismatch: ${workflow.support_coordinate}`);
    assert(workflow.workflow_claim === "diagnostic_fixit_integration", `workflow claim mismatch: ${workflow.workflow_claim}`);
    assert(workflow.preview_only === true, "workflow must be preview-only");
    assert(workflow.auto_apply === false, "workflow must not auto apply");
    assert(workflow.file_write === false, "workflow must not write files");
    assert(workflow.lsp_protocol_change === false, "workflow must not claim LSP change");
    assert(workflow.parser_frontdoor_change === false, "workflow must not claim parser/frontdoor change");
    assert(workflow.runtime_claim === false, "workflow must not claim runtime change");
    assert(workflow.lesson_schema_change === false, "workflow must not claim lesson schema change");
    assert(workflow.active_allowlist_mutation === false, "workflow must not claim allowlist mutation");
    assert(workflow.replay_claim === false, "workflow must not claim replay");
    assert(workflow.status === "diagnostic_fixit_ready", `status mismatch: ${workflow.status}`);
    assert(workflow.stage_count === 9, `stage count mismatch: ${workflow.stage_count}`);
    assert(workflow.ready_stage_count === 9, `ready stage count mismatch: ${workflow.ready_stage_count}`);
    assert(workflow.missing_stage_count === 0, `missing stage count mismatch: ${workflow.missing_stage_count}`);
    assert(workflow.diagnostic_count === 4, `diagnostic count mismatch: ${workflow.diagnostic_count}`);
    assert(workflow.fixit_count === 3, `fixit count mismatch: ${workflow.fixit_count}`);
    assert(workflow.unsupported_count === 1, `unsupported count mismatch: ${workflow.unsupported_count}`);
    assert(workflow.preview_text_line_count === 6, `preview line count mismatch: ${workflow.preview_text_line_count}`);
    assert(workflow.diff_text_line_count >= 11, `diff line count too small: ${workflow.diff_text_line_count}`);
    assert(workflow.formatter_text_line_count >= 8, `formatter line count too small: ${workflow.formatter_text_line_count}`);
    assert(result.original.includes("채비: {"), "original must remain unchanged");
    assert(workflow.preview.preview_text.includes("채비 {"), "preview patch missing");
    assert(workflow.preview.diff_text.includes("-채비: {"), "diff before missing");
    assert(String(result.text).includes("schema\tseamgrim.diagnostic_fixit_integration.v1"), "text schema missing");
    assert(String(result.text).includes("support_coordinate\t타-3"), "text support coordinate missing");
    assert(String(result.text).includes("status\tdiagnostic_fixit_ready"), "text status missing");
    assert(!String(result.text).endsWith("\n"), "workflow text must not have trailing newline");

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
