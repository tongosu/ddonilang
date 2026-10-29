#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const ROOT = process.cwd();
const UI_INDEX = "/solutions/seamgrim_ui_mvp/ui/index.html";
const MANIFEST_REL = "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.manifest.detjson";
const JS_BINDING_REL = "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js";
const WORLDS = [
  "elem_math_number_line",
  "rep_phys_projectile_xy_v1",
  "rep_phys_pendulum_wave1_v1",
  "rep_econ_supply_demand_tax_v1",
];
const VIEWPORTS = [
  { name: "desktop_1440", width: 1440, height: 900 },
  { name: "desktop_1280", width: 1280, height: 720 },
  { name: "tablet_1024", width: 1024, height: 768 },
  { name: "mobile_390", width: 390, height: 844 },
];

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
  return "application/octet-stream";
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    const optionalLessonDataPrefix =
      pathname.startsWith("/lessons/")
      || pathname.startsWith("/lessons_rewrite_v1/")
      || pathname.startsWith("/seed_lessons_v1/");
    if (optionalLessonDataPrefix && /\/(?:graph|table|space2d|maegim_control|text\.md)\b/i.test(pathname)) {
      return true;
    }
  } catch (_) {
    return false;
  }
  return false;
}

function createServer(root, { tamperManifest = false, tamperJs = false } = {}) {
  const resolvedRoot = path.resolve(root);
  const manifestPath = path.resolve(resolvedRoot, MANIFEST_REL);
  const jsBindingPath = path.resolve(resolvedRoot, JS_BINDING_REL);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      if (url.pathname === "/favicon.ico") {
        res.writeHead(204, { "cache-control": "no-store" });
        res.end();
        return;
      }
      const rawPath = decodeURIComponent(url.pathname === "/" ? UI_INDEX : url.pathname);
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      if (tamperManifest && file === manifestPath) {
        const manifest = JSON.parse(await fs.readFile(file, "utf8"));
        manifest.wasm_sha256 = `${"0".repeat(63)}1`;
        res.writeHead(200, {
          "content-type": "application/json; charset=utf-8",
          "cache-control": "no-store",
          "access-control-allow-origin": "*",
        });
        res.end(`${JSON.stringify(manifest, null, 2)}\n`);
        return;
      }
      if (tamperJs && file === jsBindingPath) {
        const bytes = Buffer.from(await fs.readFile(file));
        const marker = Buffer.from("export class DdnWasmVm");
        const markerOffset = bytes.indexOf(marker);
        assert(markerOffset >= 0, "JS tamper marker missing");
        bytes[markerOffset] = "m".charCodeAt(0);
        res.writeHead(200, {
          "content-type": mimeType(file),
          "cache-control": "no-store",
          "access-control-allow-origin": "*",
        });
        res.end(bytes);
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

async function readRunDataset(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const hash = document.querySelector("#run-mirror-hash");
    const errorPanel = document.querySelector("[data-testid='lesson-error-panel']");
    return {
      visible: root ? !root.classList.contains("hidden") : false,
      engineStatus: root?.dataset?.engineStatus ?? "",
      artifactId: root?.dataset?.wasmArtifactId ?? "",
      manifestSchema: root?.dataset?.wasmManifestSchema ?? "",
      fingerprint: root?.dataset?.wasmFingerprint ?? "",
      jsSha: root?.dataset?.wasmJsSha256 ?? "",
      wasmSha: root?.dataset?.wasmBinarySha256 ?? "",
      diagCode: root?.dataset?.wasmArtifactDiagCode ?? "",
      diagMessage: root?.dataset?.wasmArtifactDiagMessage ?? "",
      hashTitle: hash?.getAttribute("title") ?? "",
      hashText: hash?.textContent ?? "",
      errorHidden: errorPanel?.classList.contains("hidden") ?? false,
      errorText: errorPanel?.textContent ?? "",
      overflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    };
  });
}

async function verifyNormal(browser, baseUrl, manifest, lessonId, viewport) {
  const page = await browser.newPage({ viewport: { width: viewport.width, height: viewport.height } });
  const failures = [];
  installFailureCapture(page, failures);
  try {
    await page.goto(`${baseUrl}${UI_INDEX}?tab=studio&catalog_mode=full&lesson=${lessonId}`, {
      waitUntil: "domcontentloaded",
    });
    await page.waitForSelector("#screen-run:not(.hidden)", { timeout: 20_000 });
    await page.waitForFunction(() => {
      const root = document.querySelector("#screen-run");
      const hash = document.querySelector("#run-mirror-hash");
      return Boolean(root?.dataset?.wasmArtifactId)
        && String(root?.dataset?.wasmFingerprint ?? "").startsWith("sha256:")
        && String(hash?.getAttribute("title") ?? "").includes("blake3:");
    }, null, { timeout: 40_000 });
    const data = await readRunDataset(page);
    assert(data.visible, `${lessonId}/${viewport.name}: run screen hidden`);
    assert(data.artifactId === manifest.artifact_id, `${lessonId}/${viewport.name}: artifact id mismatch`);
    assert(data.manifestSchema === "ddn.wasm_artifact_manifest.v1", `${lessonId}/${viewport.name}: manifest schema mismatch`);
    assert(/^sha256:[0-9a-f]{64}$/i.test(data.fingerprint), `${lessonId}/${viewport.name}: fingerprint missing`);
    assert(data.jsSha === manifest.js_binding_sha256, `${lessonId}/${viewport.name}: js sha mismatch`);
    assert(data.wasmSha === manifest.wasm_sha256, `${lessonId}/${viewport.name}: wasm sha mismatch`);
    assert(!data.diagCode, `${lessonId}/${viewport.name}: artifact diag unexpected ${data.diagCode}`);
    assert(data.hashTitle.includes("WASM manifest:"), `${lessonId}/${viewport.name}: hash title missing manifest`);
    assert(data.hashTitle.includes("WASM fingerprint:"), `${lessonId}/${viewport.name}: hash title missing fingerprint`);
    assert(data.overflowX <= 1, `${lessonId}/${viewport.name}: horizontal overflow ${data.overflowX}`);
    assert(!failures.length, `${lessonId}/${viewport.name}: browser failures ${failures.join(" | ")}`);
    return {
      lesson_id: lessonId,
      viewport: viewport.name,
      artifact_id: data.artifactId,
      fingerprint: data.fingerprint,
      engine_status: data.engineStatus,
      hash_title_has_manifest: data.hashTitle.includes("WASM manifest:"),
    };
  } finally {
    await page.close();
  }
}

async function verifyTamper(browser, baseUrl) {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  try {
    await page.goto(`${baseUrl}${UI_INDEX}?tab=studio&catalog_mode=full&lesson=elem_math_number_line`, {
      waitUntil: "domcontentloaded",
    });
    await page.waitForSelector("#screen-run:not(.hidden)", { timeout: 20_000 });
    await page.waitForFunction(() => {
      const root = document.querySelector("#screen-run");
      const err = document.querySelector("[data-testid='lesson-error-panel']");
      return String(root?.dataset?.wasmArtifactDiagCode ?? "").startsWith("E_WASM_ARTIFACT")
        || String(err?.textContent ?? "").includes("WASM 실행 파일 검증");
    }, null, { timeout: 40_000 });
    const data = await readRunDataset(page);
    assert(data.diagCode === "E_WASM_ARTIFACT_HASH_MISMATCH", `tamper diag code mismatch: ${data.diagCode}`);
    assert(data.diagMessage.includes("WASM 산출물"), `tamper diag message not Korean artifact diag: ${data.diagMessage}`);
    assert(data.errorText.includes("WASM 실행 파일 검증"), "tamper user error missing Korean verification text");
    assert(!data.errorText.includes("서버 보조 실행"), "tamper failure silently fell back to generic server fallback text");
    return {
      diag_code: data.diagCode,
      diag_message: data.diagMessage,
      error_text_includes_korean_verification: data.errorText.includes("WASM 실행 파일 검증"),
    };
  } finally {
    await page.close();
  }
}

async function main() {
  const manifest = JSON.parse(await fs.readFile(path.join(ROOT, MANIFEST_REL), "utf8"));
  const normal = await createServer(ROOT);
  const tamperedManifest = await createServer(ROOT, { tamperManifest: true });
  const tamperedJs = await createServer(ROOT, { tamperJs: true });
  const browser = await chromium.launch({ headless: true });
  try {
    const normalRows = [];
    for (const lessonId of WORLDS) {
      for (const viewport of VIEWPORTS) {
        normalRows.push(await verifyNormal(browser, normal.baseUrl, manifest, lessonId, viewport));
      }
    }
    const manifestTamper = await verifyTamper(browser, tamperedManifest.baseUrl);
    const jsContentTamper = await verifyTamper(browser, tamperedJs.baseUrl);
    process.stdout.write(`${JSON.stringify({
      schema: "seamgrim.wasm.sealed_runtime_binding.report.v1",
      ok: true,
      artifact_id: manifest.artifact_id,
      normal_case_count: normalRows.length,
      worlds: WORLDS,
      viewports: VIEWPORTS.map((row) => row.name),
      normal: normalRows,
      tamper: manifestTamper,
      js_content_tamper: jsContentTamper,
      non_claims: {
        full_cli_wasm_single_runtime_closure: false,
        state_hash_formula_change: false,
      },
    }, null, 2)}\n`);
  } finally {
    await browser.close();
    await closeServer(normal.server);
    await closeServer(tamperedManifest.server);
    await closeServer(tamperedJs.server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
