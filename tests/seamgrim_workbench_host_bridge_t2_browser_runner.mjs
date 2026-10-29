#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  return "text/plain; charset=utf-8";
}

function createServer() {
  const server = http.createServer(async (request, response) => {
    try {
      const url = new URL(request.url || "/", "http://127.0.0.1");
      const relative = decodeURIComponent(url.pathname === "/"
        ? "/solutions/seamgrim_ui_mvp/ui/workbench/index.html"
        : url.pathname).replace(/^\/+/, "");
      const file = path.resolve(ROOT, relative);
      if (file !== ROOT && !file.startsWith(`${ROOT}${path.sep}`)) {
        response.writeHead(403);
        response.end("forbidden");
        return;
      }
      response.writeHead(200, {
        "content-type": mimeType(file),
        "cache-control": "no-store",
      });
      response.end(await fs.readFile(file));
    } catch (_) {
      response.writeHead(404);
      response.end("not found");
    }
  });
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      assert.ok(address && typeof address !== "string", "server address unavailable");
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function streamBytes(stream) {
  const chunks = [];
  for await (const chunk of stream) chunks.push(Buffer.from(chunk));
  return Buffer.concat(chunks);
}

async function main() {
  const { server, baseUrl } = await createServer();
  let browser;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ acceptDownloads: true, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("pageerror", (error) => failures.push(`pageerror: ${error.message}`));
    page.on("console", (message) => {
      if (message.type() === "error") failures.push(`console: ${message.text()}`);
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await page.waitForFunction(() => typeof window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot === "function");

    const exported = await page.evaluate(async () => {
      const result = await window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact();
      return {
        filename: result.file.filename,
        jsonText: result.file.json_text,
      };
    });
    assert.ok(exported.filename.endsWith(".seamgrim-project.json"), `unexpected filename: ${exported.filename}`);
    assert.ok(exported.jsonText.includes("seamgrim.project_snapshot_artifact.v0"), "portable artifact schema missing");

    const downloadPromise = page.waitForEvent("download");
    await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.exportPortableProject());
    const download = await downloadPromise;
    assert.equal(download.suggestedFilename(), exported.filename, "download filename drifted");
    const downloadedBytes = await streamBytes(await download.createReadStream());
    assert.equal(downloadedBytes.toString("utf8"), exported.jsonText, "download payload drifted");

    await page.locator("[data-testid='portable-project-import-input']").setInputFiles({
      name: exported.filename,
      mimeType: "application/json",
      buffer: Buffer.from(exported.jsonText, "utf8"),
    });
    await page.waitForFunction(() => {
      const pending = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.pendingPortableImport;
      return pending?.has_artifact === true && pending?.error_code === "" && pending?.byte_length > 0;
    });
    const pending = await page.evaluate(() => (
      window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().pendingPortableImport
    ));

    const clipboard = await page.evaluate(async () => {
      const { BrowserHostAdapter } = await import("/solutions/seamgrim_ui_mvp/ui/host/browser_host_adapter.js");
      const writes = [];
      const adapter = new BrowserHostAdapter({
        documentRef: document,
        navigatorRef: { clipboard: { writeText(text) { writes.push(text); return "written"; } } },
      });
      return { result: await adapter.writeClipboardText("host-bridge-browser"), writes };
    });
    assert.deepEqual(clipboard, { result: "written", writes: ["host-bridge-browser"] });

    const resources = await page.evaluate(() => performance.getEntriesByType("resource").map((entry) => entry.name));
    assert.ok(resources.some((url) => url.endsWith("/ui/host/host_bridge.js")), "HostBridge module not loaded");
    assert.ok(resources.some((url) => url.endsWith("/ui/host/browser_host_adapter.js")), "BrowserHostAdapter module not loaded");
    assert.deepEqual(failures, [], `browser failures: ${JSON.stringify(failures)}`);

    console.log(JSON.stringify({
      schema: "seamgrim.workbench.host_bridge_t2.browser_result.v1",
      download_filename_preserved: true,
      download_payload_preserved: true,
      portable_file_read_preserved: pending.has_artifact === true,
      portable_file_byte_length: pending.byte_length,
      clipboard_adapter_preserved: true,
      host_modules_loaded: true,
      result: "PASS",
    }));
  } finally {
    await browser?.close();
    await new Promise((resolve) => server.close(resolve));
  }
}

await main();
