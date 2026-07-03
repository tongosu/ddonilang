#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "std_grid_game_bogae_finite_live_loop: ok";

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
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/viewer/live.html" : url.pathname);
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
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function newPage(context, failures, requestUrls) {
  const page = await context.newPage();
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

async function waitForLiveReady(page) {
  await page.waitForFunction(() => {
    const canvas = document.getElementById("canvas");
    const status = document.getElementById("status");
    return (
      document.body.dataset.live === "1" &&
      canvas &&
      canvas.width > 0 &&
      canvas.height > 0 &&
      status &&
      status.textContent.includes("live")
    );
  });
}

function countInputRequests(urls) {
  const counts = new Map();
  for (const raw of urls) {
    let url;
    try {
      url = new URL(raw);
    } catch (_) {
      continue;
    }
    if (url.pathname !== "/input") {
      continue;
    }
    const code = url.searchParams.get("code") || "";
    const kind = url.searchParams.get("kind") || "";
    const key = code ? `${code}/${kind}` : kind;
    counts.set(key, (counts.get(key) || 0) + 1);
  }
  return counts;
}

async function waitForRequestCount(requests, key, minimum) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    const counts = countInputRequests(requests);
    if ((counts.get(key) || 0) >= minimum) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
  const counts = Object.fromEntries(countInputRequests(requests));
  throw new Error(`timed out waiting for input request ${key}>=${minimum}; counts=${JSON.stringify(counts)}`);
}

async function hold(ms) {
  await new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const assetDir = process.argv[2]
    ? path.resolve(process.argv[2])
    : path.join(root, "build", "std_grid_game_bogae_finite_live_loop_v1", "web");
  const inputUrl = process.argv[3] || "";
  assert(inputUrl.startsWith("http://127.0.0.1:"), "input_url argument must be a local http endpoint");

  for (const rel of [
    "manifest.detjson",
    "viewer/live.html",
    "viewer/viewer.js",
    "viewer/overlay.detjson",
    "viewer/skin.detjson",
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
    const requests = [];
    const page = await newPage(context, failures, requests);
    await page.goto(`${baseUrl}/viewer/live.html?input=${encodeURIComponent(inputUrl)}`, {
      waitUntil: "domcontentloaded",
    });
    await waitForLiveReady(page);

    await page.keyboard.down("ArrowLeft");
    await waitForRequestCount(requests, "ArrowLeft/down", 1);
    await hold(350);

    await page.keyboard.down("ArrowLeft");
    await hold(150);
    let counts = countInputRequests(requests);
    assert((counts.get("ArrowLeft/down") || 0) === 1, "repeat ArrowLeft emitted duplicate down");

    await page.keyboard.up("ArrowLeft");
    await waitForRequestCount(requests, "ArrowLeft/up", 1);
    await hold(250);

    await page.keyboard.down("ArrowRight");
    await waitForRequestCount(requests, "ArrowRight/down", 1);
    await hold(350);

    await page.evaluate(() => window.dispatchEvent(new Event("blur")));
    await waitForRequestCount(requests, "clear", 1);
    await hold(250);

    await page.keyboard.down("ArrowDown");
    await waitForRequestCount(requests, "ArrowDown/down", 1);
    await hold(350);

    await page.evaluate(() => {
      Object.defineProperty(document, "hidden", {
        configurable: true,
        get() {
          return true;
        },
      });
      document.dispatchEvent(new Event("visibilitychange"));
    });
    await waitForRequestCount(requests, "clear", 2);
    await hold(350);

    await page.close();
    counts = countInputRequests(requests);
    assert((counts.get("ArrowLeft/down") || 0) === 1, "ArrowLeft/down count mismatch");
    assert((counts.get("ArrowLeft/up") || 0) === 1, "ArrowLeft/up count mismatch");
    assert((counts.get("ArrowRight/down") || 0) === 1, "ArrowRight/down count mismatch");
    assert((counts.get("ArrowDown/down") || 0) === 1, "ArrowDown/down count mismatch");
    assert((counts.get("clear") || 0) >= 2, "clear count mismatch");
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
