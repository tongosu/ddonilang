#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "std_grid_game_bogae_browser_dom_smoke: ok";

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
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".detbin")) return "application/octet-stream";
  return "application/octet-stream";
}

function createServer(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/viewer/index.html" : url.pathname);
      if (rawPath === "/favicon.ico") {
        res.writeHead(204, {
          "cache-control": "no-store",
          "access-control-allow-origin": "*",
        });
        res.end();
        return;
      }
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
      resolve({
        server,
        baseUrl: `http://127.0.0.1:${address.port}`,
      });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function newSmokePage(context, failures, requestUrls) {
  const page = await context.newPage();
  await page.addInitScript(() => {
    window.__bogaeSmoke = {
      clearRect: 0,
      fillRect: 0,
      stroke: 0,
      strokeRect: 0,
      drawImage: 0,
    };
    const proto = window.CanvasRenderingContext2D && window.CanvasRenderingContext2D.prototype;
    if (proto && !proto.__bogaeSmokePatched) {
      for (const name of ["clearRect", "fillRect", "stroke", "strokeRect", "drawImage"]) {
        const original = proto[name];
        proto[name] = function (...args) {
          window.__bogaeSmoke[name] += 1;
          return original.apply(this, args);
        };
      }
      proto.__bogaeSmokePatched = true;
    }
  });
  page.on("console", (msg) => {
    if (msg.type() === "error") {
      failures.push(`console error: ${msg.text()}`);
    }
  });
  page.on("pageerror", (err) => {
    failures.push(`pageerror: ${err.message}`);
  });
  page.on("request", (req) => {
    requestUrls.push(req.url());
  });
  page.on("response", (res) => {
    if (res.status() >= 400) {
      failures.push(`response ${res.status()}: ${res.url()}`);
    }
  });
  page.on("requestfailed", (req) => {
    failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`);
  });
  return page;
}

async function waitForRender(page) {
  await page.waitForFunction(() => {
    const canvas = document.getElementById("canvas");
    return (
      canvas &&
      canvas.width > 0 &&
      canvas.height > 0 &&
      window.__bogaeSmoke &&
      window.__bogaeSmoke.clearRect > 0 &&
      window.__bogaeSmoke.fillRect > 0
    );
  });
}

async function smokePlayback(context, baseUrl, failures) {
  const requests = [];
  const page = await newSmokePage(context, failures, requests);
  await page.goto(`${baseUrl}/viewer/index.html`, { waitUntil: "domcontentloaded" });
  await waitForRender(page);
  const initial = await page.evaluate(() => ({
    title: document.title,
    canvasWidth: document.getElementById("canvas").width,
    canvasHeight: document.getElementById("canvas").height,
    playDisabled: document.getElementById("btn-play").disabled,
    stepForwardDisabled: document.getElementById("btn-step-forward").disabled,
    seekDisabled: document.getElementById("seek").disabled,
    status: document.getElementById("status").textContent,
    clearRect: window.__bogaeSmoke.clearRect,
    fillRect: window.__bogaeSmoke.fillRect,
  }));
  assert(initial.title === "Bogae Playback Viewer", "playback title mismatch");
  assert(initial.canvasWidth === 32 && initial.canvasHeight === 32, "playback canvas size mismatch");
  assert(initial.playDisabled === false, "playback play button disabled");
  assert(initial.stepForwardDisabled === false, "playback step button disabled");
  assert(initial.seekDisabled === false, "playback seek disabled");
  assert(String(initial.status).includes("madi=0"), "playback initial status mismatch");

  await page.click("#btn-step-forward");
  await page.waitForFunction(() => document.getElementById("status").textContent.includes("madi=1"));

  await page.$eval("#seek", (el) => {
    el.value = "2";
    el.dispatchEvent(new Event("input", { bubbles: true }));
  });
  await page.waitForFunction(() => document.getElementById("status").textContent.includes("madi=2"));

  await page.click("#ov-grid");
  await page.waitForFunction(() => window.__bogaeSmoke.stroke > 0);
  const overlay = await page.evaluate(() => ({
    checked: document.getElementById("ov-grid").checked,
    stroke: window.__bogaeSmoke.stroke,
  }));
  assert(overlay.checked === true, "grid overlay toggle did not stay checked");
  assert(overlay.stroke > 0, "grid overlay did not draw stroke calls");
  assert(requests.some((url) => url.endsWith("/manifest.detjson")), "playback manifest request missing");
  assert(requests.some((url) => url.includes("/frames/000002.bdl1.detbin")), "playback frame 2 request missing");
  await page.close();
}

async function smokeLive(context, baseUrl, failures) {
  const requests = [];
  const page = await newSmokePage(context, failures, requests);
  await page.goto(`${baseUrl}/viewer/live.html`, { waitUntil: "domcontentloaded" });
  await waitForRender(page);
  await page.waitForFunction(() => document.getElementById("status").textContent.includes(" live"));
  const live = await page.evaluate(() => ({
    title: document.title,
    dataLive: document.body.dataset.live,
    canvasWidth: document.getElementById("canvas").width,
    canvasHeight: document.getElementById("canvas").height,
    playDisabled: document.getElementById("btn-play").disabled,
    stepBackDisabled: document.getElementById("btn-step-back").disabled,
    stepForwardDisabled: document.getElementById("btn-step-forward").disabled,
    seekDisabled: document.getElementById("seek").disabled,
    status: document.getElementById("status").textContent,
  }));
  assert(live.title === "Bogae Live Viewer", "live title mismatch");
  assert(live.dataLive === "1", "live data-live marker missing");
  assert(live.canvasWidth === 32 && live.canvasHeight === 32, "live canvas size mismatch");
  assert(live.playDisabled === true, "live play button should be disabled");
  assert(live.stepBackDisabled === true, "live step back should be disabled");
  assert(live.stepForwardDisabled === true, "live step forward should be disabled");
  assert(live.seekDisabled === true, "live seek should be disabled");
  assert(String(live.status).includes("madi=3 live"), "live status should load latest frame");
  assert(requests.some((url) => url.includes("/manifest.detjson?t=")), "live no-cache manifest request missing");
  assert(requests.some((url) => url.includes("/frames/000003.bdl1.detbin")), "live latest frame request missing");
  await page.close();
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const assetDir = process.argv[2]
    ? path.resolve(process.argv[2])
    : path.join(root, "build", "std_grid_game_bogae_live_web_assets_v1");

  for (const rel of [
    "manifest.detjson",
    "frames/000000.bdl1.detbin",
    "frames/000001.bdl1.detbin",
    "frames/000002.bdl1.detbin",
    "frames/000003.bdl1.detbin",
    "viewer/index.html",
    "viewer/live.html",
    "viewer/viewer.js",
    "viewer/overlay.detjson",
  ]) {
    await requireFile(path.join(assetDir, rel));
  }

  const { server, baseUrl } = await createServer(assetDir);
  let browser = null;
  let context = null;
  try {
    try {
      browser = await chromium.launch({ headless: true });
    } catch (err) {
      throw new Error(`${err.message}\nRun: npx playwright install chromium`);
    }
    context = await browser.newContext({ viewport: { width: 640, height: 480 } });
    const failures = [];
    await smokePlayback(context, baseUrl, failures);
    await smokeLive(context, baseUrl, failures);
    if (failures.length > 0) {
      throw new Error(failures.join("\n"));
    }
  } finally {
    if (context) {
      await context.close().catch(() => {});
    }
    if (browser) {
      await browser.close().catch(() => {});
    }
    await closeServer(server).catch(() => {});
  }
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exit(1);
});
