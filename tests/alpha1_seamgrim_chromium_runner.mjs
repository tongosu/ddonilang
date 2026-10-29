#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const UI_PATH = "/solutions/seamgrim_ui_mvp/ui/index.html";

function assert(condition, message) { if (!condition) throw new Error(message); }
function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".ddn") || file.endsWith(".md") || file.endsWith(".toml")) return "text/plain; charset=utf-8";
  return "application/octet-stream";
}

async function createServer() {
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      if (url.pathname === "/favicon.ico") { res.writeHead(204); res.end(); return; }
      const rel = decodeURIComponent(url.pathname === "/" ? UI_PATH : url.pathname).replace(/^\/+/, "");
      const file = path.resolve(ROOT, rel);
      if (file !== ROOT && !file.startsWith(`${ROOT}${path.sep}`)) { res.writeHead(403); res.end("forbidden"); return; }
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store", "access-control-allow-origin": "*" });
      res.end(bytes);
    } catch (_) { res.writeHead(404); res.end("not found"); }
  });
  await new Promise((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", resolve); });
  const address = server.address();
  assert(address && typeof address !== "string", "static server address unavailable");
  return { server, baseUrl: `http://127.0.0.1:${address.port}` };
}

async function openEditor(page, baseUrl) {
  await page.goto(`${baseUrl}${UI_PATH}?tab=studio`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 40_000 });
  await page.locator("#screen-run .main-shell-tab[data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
}

async function editAndRun(page, source) {
  await page.fill("#ddn-textarea", source);
  await page.waitForTimeout(150);
  await page.click("#btn-run-from-editor");
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 40_000 });
  await page.waitForFunction(() => ["done", "blocked", "fatal"].includes(String(document.querySelector("#screen-run")?.dataset?.engineStatus ?? "")), null, { timeout: 60_000 });
  return page.evaluate(() => ({
    status: String(document.querySelector("#screen-run")?.dataset?.engineStatus ?? ""),
    output: [
      document.querySelector("#run-observe-output-body")?.textContent,
      document.querySelector("#run-main-console-host")?.textContent,
    ].map((value) => String(value ?? "").replace(/\s+/g, " ").trim()).join(" | "),
    outputMeta: String(document.querySelector("#run-observe-output-meta")?.textContent ?? "").trim(),
    consoleLines: [...document.querySelectorAll("#run-main-console-host .run-main-console-code")]
      .map((node) => String(node.textContent ?? "").trim()),
    error: String(document.querySelector("#run-error-text")?.textContent ?? "").replace(/\s+/g, " ").trim(),
    execPath: String(document.querySelector("#run-runtime-hint")?.textContent ?? "").replace(/\s+/g, " ").trim(),
    wasmFingerprint: String(document.querySelector("#screen-run")?.dataset?.wasmFingerprint ?? "").trim(),
    wasmDiag: String(document.querySelector("#screen-run")?.dataset?.wasmArtifactDiagCode ?? "").trim(),
  }));
}

async function returnToEditor(page) {
  await page.locator("#screen-run .main-shell-tab[data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
}

async function main() {
  const programsDir = path.join(ROOT, "solutions", "seamgrim_ui_mvp", "alpha1");
  const cases = [
    { file: "01_요금_계산기.ddn", expected: ["2800", "2100"] },
    { file: "02_조건과_반복.ddn", expected: ["작은 제곱", "작은 제곱", "큰 제곱", "큰 제곱", "30"] },
    { file: "03_자동_함수표.ddn", expected: ["0", "0", "2", "1", "1", "4", "2", "4", "10", "3", "9", "20"] },
    { file: "04_시간단계_시뮬레이션.ddn", expected: ["1", "2", "3", "2", "5", "4", "3", "9", "5", "4", "14", "6"] },
  ];
  for (const testCase of cases) testCase.source = await fs.readFile(path.join(programsDir, testCase.file), "utf8");
  const invalidSource = await fs.readFile(path.join(programsDir, "잘못된입력", "03_함수표_인자누락.ddn"), "utf8");
  const { server, baseUrl } = await createServer();
  let browser;
  const browserFailures = [];
  try {
    browser = await chromium.launch({
      headless: true,
      executablePath: "C:/Program Files/Google/Chrome/Application/chrome.exe",
    });
    const page = await (await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" })).newPage();
    page.on("pageerror", (error) => browserFailures.push(`pageerror: ${error.message}`));
    page.on("requestfailed", (request) => browserFailures.push(`requestfailed: ${request.url()} ${request.failure()?.errorText ?? ""}`));
    await openEditor(page, baseUrl);
    const results = [];
    for (const testCase of cases) {
      const result = await editAndRun(page, testCase.source);
      assert(result.status === "done", `${testCase.file}: expected done, got ${JSON.stringify(result)}`);
      assert(
        JSON.stringify(result.consoleLines) === JSON.stringify(testCase.expected),
        `${testCase.file}: output mismatch ${JSON.stringify(result.consoleLines)}`,
      );
      assert(result.wasmFingerprint && !result.wasmDiag, `${testCase.file}: WASM artifact not bound: ${JSON.stringify(result)}`);
      results.push({ file: testCase.file, status: result.status, outputMeta: result.outputMeta });
      await returnToEditor(page);
    }
    const shortLiteral = await editAndRun(page, "값 := 100.\n값 보여주기.\n");
    assert(shortLiteral.status === "done", `short literal definition: expected done, got ${JSON.stringify(shortLiteral)}`);
    assert(
      JSON.stringify(shortLiteral.consoleLines) === JSON.stringify(["100"]),
      `short literal definition output mismatch: ${JSON.stringify(shortLiteral.consoleLines)}`,
    );
    assert(shortLiteral.wasmFingerprint && !shortLiteral.wasmDiag, `short literal definition WASM artifact not bound: ${JSON.stringify(shortLiteral)}`);
    results.push({ file: "inline:short-literal-definition", status: shortLiteral.status, outputMeta: shortLiteral.outputMeta });
    await returnToEditor(page);
    const invalid = await editAndRun(page, invalidSource);
    assert(["blocked", "fatal"].includes(invalid.status), `invalid source was not rejected: ${JSON.stringify(invalid)}`);
    assert(invalid.error.includes("문법") || invalid.error.includes("인자") || invalid.error.includes("실행"), `invalid source error not visible: ${invalid.error}`);
    await returnToEditor(page);
    const repaired = await editAndRun(page, cases[2].source);
    assert(repaired.status === "done", `repaired source did not run: ${JSON.stringify(repaired)}`);
    assert(
      JSON.stringify(repaired.consoleLines) === JSON.stringify(cases[2].expected),
      `repaired source output mismatch: ${JSON.stringify(repaired.consoleLines)}`,
    );
    assert(repaired.wasmFingerprint && !repaired.wasmDiag, `repaired source WASM artifact not bound: ${JSON.stringify(repaired)}`);
    await fs.mkdir(path.join(ROOT, "output", "playwright"), { recursive: true });
    await page.screenshot({ path: path.join(ROOT, "output", "playwright", "alpha1-function-table-repaired.png"), fullPage: true });
    assert(browserFailures.length === 0, `browser failures: ${browserFailures.join(" | ")}`);
    console.log(`ALPHA1_SEAMGRIM_CHROMIUM_PASS programs=${cases.length} short_literal_definition=1 error_fix_rerun=1 actual_wasm=1`);
    console.log(JSON.stringify(results));
  } finally {
    await browser?.close();
    await new Promise((resolve) => server.close(resolve));
  }
}

main().catch((error) => { console.error(String(error?.stack ?? error)); process.exit(1); });
