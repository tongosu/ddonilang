#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import zlib from "node:zlib";
import { chromium } from "playwright";

const ROOT = path.resolve(process.cwd());
const CSP = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js") || file.endsWith(".mjs")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".toml") || file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  if (file.endsWith(".md")) return "text/markdown; charset=utf-8";
  return "application/octet-stream";
}

function securityHeaders(extra = {}) {
  return {
    "content-security-policy": CSP,
    "x-content-type-options": "nosniff",
    "referrer-policy": "strict-origin-when-cross-origin",
    "x-frame-options": "DENY",
    "permissions-policy": "camera=(), microphone=(), geolocation=()",
    ...extra,
  };
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|text|maegim_control)\.(?:json|md)$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function createEdgeSmokeServer() {
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      if (url.pathname === "/favicon.ico") {
        res.writeHead(204, securityHeaders({ "cache-control": "no-store" }));
        res.end();
        return;
      }
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/index.html" : url.pathname);
      const file = path.resolve(ROOT, rawPath.replace(/^\/+/, ""));
      if (file !== ROOT && !file.startsWith(`${ROOT}${path.sep}`)) {
        res.writeHead(403, securityHeaders({ "content-type": "text/plain; charset=utf-8" }));
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      const headers = securityHeaders({
        "content-type": mimeType(file),
        "cache-control": file.endsWith(".wasm") ? "public, max-age=31536000, immutable" : "no-store",
      });
      res.writeHead(200, headers);
      res.end(bytes);
    } catch (_) {
      res.writeHead(404, securityHeaders({ "content-type": "text/plain; charset=utf-8" }));
      res.end("not found");
    }
  });
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("failed to bind edge smoke server"));
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
    const text = String(msg.text() ?? "");
    if (msg.type() !== "error") return;
    if (text.includes("Failed to load resource")) return;
    failures.push(`console error: ${text}`);
  });
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`response ${res.status()}: ${res.url()}`);
  });
}

function unfilterPngScanlines(width, height, bytesPerPixel, raw) {
  const stride = width * bytesPerPixel;
  const out = Buffer.alloc(height * stride);
  let src = 0;
  for (let y = 0; y < height; y += 1) {
    const filter = raw[src];
    src += 1;
    const rowStart = y * stride;
    const prevStart = (y - 1) * stride;
    for (let x = 0; x < stride; x += 1) {
      const value = raw[src + x];
      const left = x >= bytesPerPixel ? out[rowStart + x - bytesPerPixel] : 0;
      const up = y > 0 ? out[prevStart + x] : 0;
      const upLeft = y > 0 && x >= bytesPerPixel ? out[prevStart + x - bytesPerPixel] : 0;
      let recon = value;
      if (filter === 1) recon = value + left;
      if (filter === 2) recon = value + up;
      if (filter === 3) recon = value + Math.floor((left + up) / 2);
      if (filter === 4) {
        const p = left + up - upLeft;
        const pa = Math.abs(p - left);
        const pb = Math.abs(p - up);
        const pc = Math.abs(p - upLeft);
        const predictor = pa <= pb && pa <= pc ? left : pb <= pc ? up : upLeft;
        recon = value + predictor;
      }
      out[rowStart + x] = recon & 0xff;
    }
    src += stride;
  }
  return out;
}

function decodePngEvidence(buffer) {
  const signature = buffer.subarray(0, 8).toString("hex");
  assert(signature === "89504e470d0a1a0a", "canvas screenshot is not PNG");
  let offset = 8;
  let width = 0;
  let height = 0;
  let colorType = 0;
  const idat = [];
  while (offset < buffer.length) {
    const length = buffer.readUInt32BE(offset);
    const type = buffer.subarray(offset + 4, offset + 8).toString("ascii");
    const data = buffer.subarray(offset + 8, offset + 8 + length);
    if (type === "IHDR") {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      colorType = data[9];
    }
    if (type === "IDAT") idat.push(data);
    if (type === "IEND") break;
    offset += 12 + length;
  }
  const bytesPerPixel = colorType === 6 ? 4 : colorType === 2 ? 3 : 0;
  assert(width > 0 && height > 0 && bytesPerPixel > 0, `unsupported PNG format: ${width}x${height} type=${colorType}`);
  const raw = zlib.inflateSync(Buffer.concat(idat));
  const pixels = unfilterPngScanlines(width, height, bytesPerPixel, raw);
  const colors = new Set();
  const pixelCount = width * height;
  const step = Math.max(1, Math.floor(pixelCount / 4000));
  for (let pixel = 0; pixel < pixelCount; pixel += step) {
    const idx = pixel * bytesPerPixel;
    colors.add(`${pixels[idx]}:${pixels[idx + 1]}:${pixels[idx + 2]}:${bytesPerPixel === 4 ? pixels[idx + 3] : 255}`);
    if (colors.size > 12) break;
  }
  return { width, height, colors: colors.size };
}

async function canvasEvidence(page) {
  const canvas = page.locator("[data-testid='lesson-space2d-canvas']");
  await canvas.waitFor({ state: "visible", timeout: 20_000 });
  const box = await canvas.boundingBox();
  const screenshot = await canvas.screenshot({ type: "png" });
  const evidence = decodePngEvidence(screenshot);
  return { ...evidence, cssWidth: box?.width ?? 0, cssHeight: box?.height ?? 0 };
}

async function main() {
  const { server, baseUrl } = await createEdgeSmokeServer();
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await context.newPage();
    installFailureCapture(page, failures);

    const response = await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=projectile`, {
      waitUntil: "domcontentloaded",
    });
    assert(response?.ok(), `index response failed: ${response?.status()}`);
    const headers = response.headers();
    assert(headers["content-security-policy"] === CSP, "CSP header mismatch");
    assert(headers["x-content-type-options"] === "nosniff", "nosniff header missing");
    assert(headers["x-frame-options"] === "DENY", "frame header missing");

    const root = page.locator("#screen-run[data-first-delight='projectile']");
    await root.waitFor({ state: "visible", timeout: 20_000 });
    const deadline = Date.now() + 20_000;
    let ready = false;
    let lastState = {};
    while (Date.now() < deadline) {
      const mode = String((await root.getAttribute("data-main-visual-mode")) || "");
      const segmentCount = Number((await root.getAttribute("data-main-trajectory-segment-count")) || 0);
      lastState = { mode, segmentCount };
      if (["space2d", "debug-fallback"].includes(mode) && segmentCount >= 3) {
        ready = true;
        break;
      }
      await page.waitForTimeout(200);
    }
    assert(ready, `main visual did not become ready: ${JSON.stringify(lastState)}`);

    const wasmResponse = await page.request.get(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm`);
    assert(wasmResponse.ok(), `wasm response failed: ${wasmResponse.status()}`);
    assert(String(wasmResponse.headers()["content-type"] || "").includes("application/wasm"), "wasm MIME mismatch");
    assert(String(wasmResponse.headers()["cache-control"] || "").includes("immutable"), "wasm cache policy missing");

    const evidence = await canvasEvidence(page);
    assert(evidence && evidence.cssWidth >= 320 && evidence.cssHeight >= 240, `canvas too small or missing: ${JSON.stringify(evidence)}`);
    assert(evidence.colors >= 2, `canvas appears blank: ${JSON.stringify(evidence)}`);
    assert(failures.length === 0, failures.join("\n"));
    console.log("seamgrim_private_pilot_edge_policy: ok");
    await context.close();
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
