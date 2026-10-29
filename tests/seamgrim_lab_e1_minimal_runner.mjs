#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_lab_e1_minimal: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const LESSON_ID = "rep_math_function_line_v1";

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
      const rawPath = decodeURIComponent(url.pathname === "/" ? PROJECT_URL : url.pathname);
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
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|maegim_control)\.(?:json|detjson)$/i.test(pathname)
    ) return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/text\.md$/i.test(pathname)
    ) return true;
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

async function readLineage(page) {
  return page.evaluate(() => {
    const el = document.querySelector("[data-lab-e1-lineage]");
    return {
      state: el?.dataset?.state ?? "",
      schema: el?.dataset?.schema ?? "",
      runCount: Number(el?.dataset?.runCount ?? 0),
      originHash: el?.dataset?.originHash ?? "",
      deltaRef: el?.dataset?.deltaRef ?? "",
      comparisonStatus: el?.dataset?.comparisonStatus ?? "",
      autoApplyClaim: el?.dataset?.autoApplyClaim ?? "",
      fileWriteClaim: el?.dataset?.fileWriteClaim ?? "",
      text: el?.innerText ?? "",
    };
  });
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    installFailureCapture(page, failures);
    await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${LESSON_ID}`, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(() => {
      const root = document.querySelector("#screen-run");
      const lineage = document.querySelector("[data-lab-e1-lineage]");
      return root?.dataset?.mainVisualMode === "graph"
        && root?.dataset?.engineStatus === "done"
        && Number(root?.dataset?.mainGraphPointCount ?? 0) >= 3
        && lineage?.dataset?.schema === "seamgrim.lab_e1_lineage.v1"
        && Number(lineage?.dataset?.runCount ?? 0) >= 1
        && String(lineage?.dataset?.originHash ?? "").length >= 8;
    }, null, { timeout: 20_000 });

    const origin = await readLineage(page);
    assert(origin.schema === "seamgrim.lab_e1_lineage.v1", "lineage schema missing");
    assert(origin.runCount >= 1, `origin run not recorded: ${JSON.stringify(origin)}`);
    assert(origin.originHash.length >= 8, `origin_hash missing: ${JSON.stringify(origin)}`);
    assert(origin.deltaRef === "delta:none", `initial delta_ref should be none: ${JSON.stringify(origin)}`);
    assert(origin.autoApplyClaim === "false" && origin.fileWriteClaim === "false", "E1 must not claim auto apply or file write");

    const modes = await page.evaluate(() => Array.from(document.querySelectorAll(".run-view-mode-btn")).map((node) => node.textContent?.trim() ?? ""));
    assert(["기본", "확대", "전체"].every((label) => modes.includes(label)), `layout mode controls missing: ${modes}`);

    const slopeInput = page.locator("input[type='number'][data-control-name='기울기']");
    await slopeInput.fill("3.5");
    await slopeInput.dispatchEvent("change");
    await page.waitForFunction((originHash) => {
      const lineage = document.querySelector("[data-lab-e1-lineage]");
      return lineage?.dataset?.comparisonStatus === "changed"
        && Number(lineage?.dataset?.runCount ?? 0) >= 2
        && lineage?.dataset?.originHash === originHash
        && String(lineage?.dataset?.deltaRef ?? "").startsWith("delta:")
        && lineage?.dataset?.deltaRef !== "delta:none";
    }, origin.originHash, { timeout: 20_000 });

    const changed = await readLineage(page);
    const source = await page.locator("#run-ddn-preview").inputValue();
    assert(source.includes("기울기: 수 <- (3.5)"), "control change did not update editable DDN source");
    assert(changed.text.includes("내 버전 변화 있음"), `changed status not visible: ${JSON.stringify(changed)}`);
    assert(changed.originHash === origin.originHash, "origin_hash should stay stable after my-version run");
    assert(changed.deltaRef.startsWith("delta:"), `delta_ref not generated: ${JSON.stringify(changed)}`);

    const manager = await page.evaluate(() => ({
      runCount: document.querySelector("[data-run-history-count]")?.dataset?.value ?? "",
      active: document.querySelector("[data-run-history-active]")?.dataset?.state ?? "",
      exportText: document.querySelector("[data-run-history-export-summary-text]")?.textContent ?? "",
    }));
    assert(Number(manager.runCount) >= 2, `run history did not record original and my version: ${JSON.stringify(manager)}`);
    assert(manager.active === "active", `active run missing: ${JSON.stringify(manager)}`);
    assert(manager.exportText.includes("Seamgrim run history export summary"), "run history export summary missing");
    assert(failures.length === 0, failures.join("\n"));
    process.stdout.write(`${OK}\n`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
