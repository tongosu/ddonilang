#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_diagnostic_fixit_preview_browser: ok";
const UNSAFE_BROWSER_PORTS = new Set([
  1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
  101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161,
  179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563,
  587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
  6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
]);

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) {
    throw new Error(`missing file: ${file}`);
  }
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
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
    const context = await browser.newContext({
      viewport: { width: 1180, height: 760 },
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
      if (res.status() >= 400 && !res.url().endsWith("/favicon.ico") && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    const result = await page.evaluate(async () => {
      const {
        buildDiagnosticFixitPreview,
        formatDiagnosticFixitPreviewText,
      } = await import("./studio_diagnostic_fixit_preview.js");
      const sourceText = [
        "#이름: 진단스모크",
        "채비: {",
        "  t:수 <- 0.",
        "}.",
        "값 <- (1 + 2.",
        "받으면 { 값 보여주기. }.",
      ].join("\n");
      const preview = buildDiagnosticFixitPreview({
        sourceText,
        diagnostics: [
          {
            code: "E_BLOCK_HEADER_HASH_FORBIDDEN",
            message: "hash header forbidden",
            span: { line: 1, column: 1 },
          },
          {
            code: "E_BLOCK_HEADER_COLON_FORBIDDEN",
            technical_message: "block header has colon",
            span: { line: 2, column: 5 },
          },
          {
            code: "E_PARSE_EXPECTED_RPAREN",
            technical_message: "닫는 괄호 필요",
            span: { line: 5, column: 12 },
          },
          {
            code: "E_RECEIVE_OUTSIDE_IMJA",
            technical_message: "receive outside imja",
            span: { line: 6, column: 1 },
          },
        ],
      });
      return {
        preview,
        text: formatDiagnosticFixitPreviewText(preview),
        original: sourceText,
      };
    });

    const preview = result.preview;
    assert(preview.__종류 === "studio_diagnostic_fixit_preview", "preview kind mismatch");
    assert(preview.preview_only === true, "preview_only must be true");
    assert(preview.auto_apply === false, "auto_apply must be false");
    assert(preview.diagnostic_count === 4, "diagnostic count mismatch");
    assert(preview.fixit_count === 3, "fixit count mismatch");
    assert(preview.unsupported_count === 1, "unsupported count mismatch");
    assert(result.original.includes("채비: {"), "original source should remain legacy");
    assert(preview.preview_text.includes('설정 { 제목: "진단스모크". }.'), "hash header preview missing");
    assert(preview.preview_text.includes("채비 {"), "block header colon preview missing");
    assert(preview.preview_text.includes("값 <- (1 + 2)."), "expected rparen preview missing");
    assert(preview.preview_text.includes("받으면 { 값 보여주기. }."), "unsupported line should remain");
    assert(preview.diff_text.includes("--- original\n+++ preview"), "diff header missing");
    assert(preview.diff_text.includes("-채비: {"), "colon diff before missing");
    assert(preview.diff_text.includes("+채비 {"), "colon diff after missing");
    assert(preview.items.some((item) => item.code === "E_RECEIVE_OUTSIDE_IMJA" && item.fixit_available === false), "unsupported item missing");
    assert(result.text.includes("진단\t4"), "formatted diagnostic count missing");
    assert(result.text.includes("수정후보\t3"), "formatted fixit count missing");
    assert(result.text.includes("E_RECEIVE_OUTSIDE_IMJA\t없음"), "formatted unsupported row missing");

    if (failures.length > 0) {
      throw new Error(failures.join("\n"));
    }
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
