#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_diagnostic_fixit_editor_apply: ok";

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
    return url.pathname === "/api/lessons/inventory" || url.pathname === "/api/lesson-inventory";
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
    "screens/editor.js",
    "play_diagnostic_contract.js",
    "studio_diagnostic_fixit_preview.js",
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
      const { EditorScreen } = await import("./screens/editor.js");
      const { buildDiagnosticFixitPreview } = await import("./studio_diagnostic_fixit_preview.js");
      const root = document.querySelector("#screen-editor");
      root?.classList?.remove("hidden");
      const sourceText = [
        "설정 { 제목: \"Editor Apply\". }.",
        "채비: {",
        "  t:수 <- 0.",
        "}.",
        "(매마디)마다 {",
        "  t 보여주기.",
        "}."
      ].join("\n");
      let dirty = false;
      let changed = false;
      const screen = new EditorScreen({
        root,
        onSourceChange: () => {
          changed = true;
        },
        onApplyFixit: async (preview, { sourceText: before }) => {
          const after = String(preview?.preview_text ?? before);
          screen.replaceDdn(after, { emitSourceChange: true });
          dirty = true;
          const payload = {
            schema: "seamgrim.diagnostic_fixit_editor_apply.v1",
            applied: true,
            source_line_count_before: String(before ?? "").split("\n").length,
            source_line_count_after: after.split("\n").length,
            applied_fixit_count: Number(preview?.fixit_count ?? 0),
            dirty,
            source_changed: after !== before,
            file_write_claim: false,
          };
          window.__STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY__ = payload;
          screen.setFixitModel(null);
          screen.setSmokeResult(`Diagnostic fix-it 적용: ${payload.applied_fixit_count}건`);
          return true;
        },
      });
      screen.init();
      screen.loadLesson(sourceText, { title: "Editor Apply", readOnly: false });
      changed = false;
      const diagnostics = [{ code: "E_BLOCK_HEADER_COLON_FORBIDDEN", message: "block header colon forbidden", span: null }];
      const preview = buildDiagnosticFixitPreview({ sourceText, diagnostics });
      screen.setFixitModel(preview);
      const beforeVisible = !document.querySelector("#editor-fixit-card")?.classList.contains("hidden");
      document.querySelector("#btn-editor-fixit-apply")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        beforeVisible,
        summary: document.querySelector("#editor-fixit-summary")?.textContent?.trim(),
        textareaValue: document.querySelector("#ddn-textarea")?.value ?? "",
        smoke: document.querySelector("#editor-smoke-result")?.textContent?.trim(),
        payload: window.__STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY__,
        dirty,
        changed,
        previewText: preview.preview_text,
      };
    });

    assert(result.beforeVisible === true, "inline fix-it panel should be visible before apply");
    assert(result.textareaValue === result.previewText, "textarea value should match preview text after apply");
    assert(result.textareaValue.includes("채비 {"), "fixed source should remove block header colon");
    assert(!result.textareaValue.includes("채비: {"), "legacy block header colon should be gone");
    assert(result.smoke === "Diagnostic fix-it 적용: 1건", `smoke mismatch: ${result.smoke}`);
    assert(result.dirty === true, "dirty flag should be true");
    assert(result.changed === true, "source change callback should fire");
    const payload = result.payload ?? {};
    assert(payload.schema === "seamgrim.diagnostic_fixit_editor_apply.v1", "payload schema mismatch");
    assert(payload.applied === true, "payload applied mismatch");
    assert(payload.source_line_count_before === 7, "payload before line count mismatch");
    assert(payload.source_line_count_after === 7, "payload after line count mismatch");
    assert(payload.applied_fixit_count === 1, "payload fixit count mismatch");
    assert(payload.dirty === true, "payload dirty mismatch");
    assert(payload.source_changed === true, "payload source_changed mismatch");
    assert(payload.file_write_claim === false, "payload file_write_claim mismatch");

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
