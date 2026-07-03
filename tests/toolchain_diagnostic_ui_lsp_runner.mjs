#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "toolchain_diagnostic_ui_lsp: ok";

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
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
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
    return (
      url.pathname === "/api/lessons/inventory" ||
      url.pathname === "/api/lesson-inventory" ||
      url.pathname.startsWith("/lessons/") ||
      url.pathname.startsWith("/seed_lessons_v1/")
    );
  } catch (_) {
    return false;
  }
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "studio_diagnostic_fixit_integration.js",
    "toolchain_diagnostic_ui_lsp.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 840 }, locale: "ko-KR" });
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-toolchain-diagnostic-ui-lsp][data-toolchain-diagnostic-ui-lsp-status='toolchain_diagnostic_ui_lsp_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_TOOLCHAIN_DIAGNOSTIC_UI_LSP_ROWS,
        buildToolchainDiagnosticUiLsp,
        formatToolchainDiagnosticUiLspText,
      } = await import("./toolchain_diagnostic_ui_lsp.js");
      const diagnosticUi = buildToolchainDiagnosticUiLsp({ rows: DEFAULT_TOOLCHAIN_DIAGNOSTIC_UI_LSP_ROWS });
      return { diagnosticUi, text: formatToolchainDiagnosticUiLspText(diagnosticUi) };
    });
    const diagnosticUi = moduleResult.diagnosticUi;
    assert(diagnosticUi.schema === "ddn.toolchain.diagnostic_ui_lsp.v1", "schema mismatch");
    assert(diagnosticUi.work_item === "TA3_DIAGNOSTIC_UI_LSP_V1", "work item mismatch");
    assert(diagnosticUi.primary_coordinate === "타-3", "coordinate mismatch");
    assert(diagnosticUi.depends_on_coordinate.join(",") === "타-2", "dependency mismatch");
    assert(diagnosticUi.pack === "toolchain_pack_3_v1", "pack mismatch");
    assert(diagnosticUi.status === "toolchain_diagnostic_ui_lsp_ready", "status mismatch");
    assert(diagnosticUi.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(diagnosticUi.diagnostic_viewer_claim === true, "diagnostic claim mismatch");
    assert(diagnosticUi.fixit_preview_claim === true, "fixit claim mismatch");
    assert(diagnosticUi.lsp_lite_contract_claim === true, "lsp-lite claim mismatch");
    assert(diagnosticUi.full_lsp_server_claim === false, "full lsp server must stay false");
    assert(diagnosticUi.lsp_protocol_change === false, "protocol change must stay false");
    assert(diagnosticUi.auto_apply_claim === false, "auto apply must stay false");
    assert(diagnosticUi.file_write_claim === false, "file write must stay false");
    assert(diagnosticUi.runtime_claim === false, "runtime claim must stay false");
    assert(diagnosticUi.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(diagnosticUi.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(diagnosticUi.progress.roadmap_v2_matrix_behavior_closed === 22, "roadmap closed mismatch");
    assert(diagnosticUi.progress.roadmap_v2_matrix_behavior_percent === 24, "roadmap percent mismatch");
    assert(diagnosticUi.progress.roadmap_v2_pack_evidence_reference_closed === 42, "pack ref mismatch");
    assert(diagnosticUi.progress.roadmap_v2_pack_evidence_reference_percent === 47, "pack ref percent mismatch");
    assert(diagnosticUi.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(diagnosticUi.workflow.diagnostic_count === 3, "diagnostic count mismatch");
    assert(diagnosticUi.workflow.fixit_count === 2, "fixit count mismatch");
    assert(diagnosticUi.workflow.unsupported_count === 1, "unsupported count mismatch");
    assert(String(diagnosticUi.workflow.preview.diff_text).includes("--- original"), "diff missing");
    assert(String(moduleResult.text).includes("full_lsp_server_claim\tfalse"), "text missing full LSP boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t42/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-toolchain-diagnostic-ui-lsp]");
      const buttons = Array.from(document.querySelectorAll(".toolchain-diagnostic-btn[data-toolchain-diagnostic-row]"));
      buttons.find((button) => button.getAttribute("data-toolchain-diagnostic-row") === "lsp_contract")?.click();
      document.querySelector("[data-toolchain-diagnostic-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-toolchain-diagnostic-ui-lsp-status") || "",
        copied: root?.getAttribute("data-toolchain-diagnostic-ui-lsp-copied") || "",
        buttonCount: buttons.length,
        progress: document.querySelector("[data-toolchain-diagnostic-progress]")?.textContent || "",
        summary: document.querySelector("[data-toolchain-diagnostic-summary]")?.textContent || "",
        title: document.querySelector("[data-toolchain-diagnostic-active-title]")?.textContent || "",
        source: document.querySelector("[data-toolchain-diagnostic-source]")?.textContent || "",
        diff: document.querySelector("[data-toolchain-diagnostic-diff]")?.textContent || "",
        preview: document.querySelector("[data-toolchain-diagnostic-preview]")?.textContent || "",
        globalSchema: window.__TOOLCHAIN_DIAGNOSTIC_UI_LSP__?.schema || "",
        globalText: window.__TOOLCHAIN_DIAGNOSTIC_UI_LSP_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "toolchain_diagnostic_ui_lsp_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("22/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("full LSP server") && domResult.summary.includes("file write") && domResult.summary.includes("auto-apply"), "summary missing boundary");
    assert(domResult.title === "LSP-lite contract", `active title mismatch: ${domResult.title}`);
    assert(domResult.source.includes("채비: {"), "source mismatch");
    assert(domResult.diff.includes("블록 헤더 ':' 제거") || domResult.diff.includes("--- original"), "diff mismatch");
    assert(domResult.preview.includes("채비 {") && domResult.preview.includes("1 + 2)"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.toolchain.diagnostic_ui_lsp.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t22/90"), "global text missing roadmap matrix");
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
