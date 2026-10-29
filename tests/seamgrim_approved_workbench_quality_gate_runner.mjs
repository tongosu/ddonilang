#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_approved_workbench_quality_gate: ok";
const VIEWPORTS = [
  { width: 1440, height: 900 },
  { width: 1024, height: 768 },
  { width: 778, height: 912 },
  { width: 390, height: 844 },
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
  if (file.endsWith(".toml") || file.endsWith(".ddn") || file.endsWith(".md")) return "text/plain; charset=utf-8";
  return "application/octet-stream";
}

function createServer(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      const rel = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/workbench/index.html" : url.pathname).replace(/^\/+/, "");
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
        reject(new Error("failed to bind server"));
        return;
      }
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function assertCanvasNonBlank(page) {
  const sample = await page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let nonWhite = 0;
    for (let i = 0; i < data.length; i += 4) {
      if (data[i + 3] && (data[i] < 240 || data[i + 1] < 240 || data[i + 2] < 240)) nonWhite += 1;
    }
    return { width: canvas.width, height: canvas.height, nonWhite };
  });
  assert(sample.width >= 320 && sample.height >= 240, `canvas too small: ${JSON.stringify(sample)}`);
  assert(sample.nonWhite > 500, `canvas blank: ${JSON.stringify(sample)}`);
}

async function assertNoOverflow(page) {
  const metrics = await page.evaluate(() => {
    function insideClosedDetails(node) {
      let current = node.parentElement;
      while (current) {
        if (current.tagName === "DETAILS" && !current.open) {
          const summary = current.querySelector(":scope > summary");
          if (!summary || !summary.contains(node)) return true;
        }
        current = current.parentElement;
      }
      return false;
    }
    const viewportHeight = window.innerHeight;
    const boxes = Array.from(document.querySelectorAll("button, input, textarea, canvas, [data-testid]"))
      .filter((node) => {
        const style = getComputedStyle(node);
        const rect = node.getBoundingClientRect();
        return style.display !== "none"
          && style.visibility !== "hidden"
          && rect.width > 0
          && rect.height > 0
          && !insideClosedDetails(node)
          && rect.top >= 0
          && rect.top <= viewportHeight;
      })
      .map((node) => {
        const rect = node.getBoundingClientRect();
        return {
          testid: node.getAttribute("data-testid") || node.tagName.toLowerCase(),
          left: rect.left,
          right: rect.right,
          top: rect.top,
          bottom: rect.bottom,
          width: rect.width,
          height: rect.height,
        };
      });
    return {
      clientWidth: document.documentElement.clientWidth,
      scrollWidth: document.documentElement.scrollWidth,
      boxes,
    };
  });
  assert(metrics.scrollWidth <= metrics.clientWidth + 2, `horizontal overflow: ${JSON.stringify(metrics)}`);
  for (const box of metrics.boxes) {
    assert(box.right <= metrics.clientWidth + 2, `box overflows right edge: ${JSON.stringify(box)}`);
    assert(box.left >= -2, `box overflows left edge: ${JSON.stringify(box)}`);
    assert(box.width >= 1 && box.height >= 1, `box collapsed: ${JSON.stringify(box)}`);
  }
}

async function assertHeights(page, viewport) {
  const sizes = await page.evaluate(() => ({
    source: document.querySelector("[data-testid='source-panel']").getBoundingClientRect(),
    canvas: document.querySelector("[data-testid='bogae-canvas-wrap']").getBoundingClientRect(),
    sourceText: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    sourceAccess: Boolean(document.querySelector("[data-testid='source-access-button']")),
  }));
  assert(sizes.sourceAccess, "DDN source access button missing");
  assert(sizes.sourceText.includes("설정"), "DDN source not accessible");
  assert(sizes.canvas.height >= (viewport.width <= 720 ? 300 : 230), `canvas height too small: ${JSON.stringify(sizes.canvas)}`);
  assert(sizes.source.height >= (viewport.width <= 720 ? 250 : 230), `source panel height too small: ${JSON.stringify(sizes.source)}`);
}

async function assertA11y(page) {
  const result = await page.evaluate(() => {
    const iconButtons = Array.from(document.querySelectorAll(".icon-command"));
    const badIcon = iconButtons.filter((button) => !button.getAttribute("aria-label") || !button.getAttribute("title"));
    const namedControls = Array.from(document.querySelectorAll("button, input, textarea")).filter((node) => {
      const label = node.getAttribute("aria-label") || node.getAttribute("title") || node.textContent || node.getAttribute("placeholder");
      return String(label ?? "").trim();
    });
    return {
      badIconCount: badIcon.length,
      namedCount: namedControls.length,
      total: document.querySelectorAll("button, input, textarea").length,
    };
  });
  assert(result.badIconCount === 0, `icon buttons missing tooltip/a11y name: ${JSON.stringify(result)}`);
  assert(result.namedCount === result.total, `controls missing accessible names: ${JSON.stringify(result)}`);
  await page.keyboard.press("Tab");
  await page.keyboard.press("Tab");
  const focusVisible = await page.evaluate(() => {
    const active = document.activeElement;
    const rect = active?.getBoundingClientRect?.();
    return Boolean(active && rect && rect.width > 0 && rect.height > 0);
  });
  assert(focusVisible, "keyboard focus did not land on a visible control");
}

async function prepareRuntimeViewActions(page) {
  await page.click("[data-testid='lifecycle-run']");
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snap?.executionLifecycle?.status === "running"
      && Boolean(snap?.activeFrame)
      && Boolean(snap?.semanticRenderer)
      && Number(snap?.runs?.[0]?.frame_count ?? 0) >= 1;
  }, null, { timeout: 60000 });
  await page.click("[data-testid='lifecycle-run'][data-action='pause']");
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    const side = document.querySelector("[data-testid='view-role-side']");
    return snap?.executionLifecycle?.status === "paused"
      && !snap?.livePump?.in_flight
      && side
      && side.hidden === false
      && side.disabled === false;
  }, null, { timeout: 60000 });
}

async function assertSideRefit(page) {
  await page.click("[data-testid='view-role-side']");
  await page.waitForFunction(() => document.querySelector("[data-testid='bogae-stage']")?.dataset.sideOpen === "true");
  const open = await page.evaluate(() => ({
    sideVisible: getComputedStyle(document.querySelector("[data-testid='side-view-panel']")).display !== "none",
    canvasWidth: document.querySelector("[data-testid='bogae-canvas-wrap']").getBoundingClientRect().width,
  }));
  assert(open.sideVisible, "side panel did not open");
  await page.click("[data-testid='side-view-close']");
  await page.waitForFunction(() => document.querySelector("[data-testid='bogae-stage']")?.dataset.sideOpen === "false");
  const closed = await page.evaluate(() => ({
    sideVisible: getComputedStyle(document.querySelector("[data-testid='side-view-panel']")).display !== "none",
    canvasWidth: document.querySelector("[data-testid='bogae-canvas-wrap']").getBoundingClientRect().width,
  }));
  assert(!closed.sideVisible, "side panel did not close");
  assert(closed.canvasWidth >= open.canvasWidth, `canvas did not refit after side close: ${JSON.stringify({ open, closed })}`);
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    for (const viewport of VIEWPORTS) {
      const context = await browser.newContext({
        viewport,
        locale: "ko-KR",
        reducedMotion: "reduce",
      });
      const pageFailures = [];
      const page = await context.newPage();
      page.on("console", (msg) => {
        if (["error", "warning"].includes(msg.type())) pageFailures.push(`${msg.type()}: ${msg.text()}`);
      });
      page.on("pageerror", (err) => pageFailures.push(`pageerror: ${err.message}`));
      page.on("response", (res) => {
        if (res.status() >= 400) pageFailures.push(`response ${res.status()}: ${res.url()}`);
      });
      await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
      await page.click("[data-testid='project-card-rep_phys_pendulum_wave1_v1']");
      await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_pendulum_wave1_v1']");
      await assertHeights(page, viewport);
      await prepareRuntimeViewActions(page);
      await assertCanvasNonBlank(page);
      await assertSideRefit(page);
      await assertNoOverflow(page);
      await assertA11y(page);
      const reduced = await page.evaluate(() => matchMedia("(prefers-reduced-motion: reduce)").matches);
      assert(reduced === true, "reduced motion context not active");
      if (pageFailures.length) failures.push(`${viewport.width}x${viewport.height}: ${pageFailures.join("\n")}`);
      await context.close();
    }
    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);
    console.log(OK);
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
