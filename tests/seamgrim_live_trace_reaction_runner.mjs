#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_live_trace_reaction: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const LESSON_ID = "rep_phys_pendulum_wave1_v1";

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
      if (!address || typeof address === "string") reject(new Error("failed to bind static server"));
      else resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
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
    if (res.status() >= 500) failures.push(`response ${res.status()}: ${res.url()}`);
  });
}

async function readLive(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    return {
      root: { ...(root?.dataset ?? {}) },
      live: window.__SEAMGRIM_LIVE_TRACE_REACTION_RUNTIME__ ?? null,
      offline: window.__SEAMGRIM_TRACE_REACTION_RUNTIME__ ?? null,
      hash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
    };
  });
}

async function stepUntil(page, predicate, label) {
  for (let i = 0; i < 80; i += 1) {
    const state = await readLive(page);
    if (predicate(state)) return state;
    await page.locator("#btn-step").click();
    await page.waitForTimeout(25);
  }
  throw new Error(`timed out waiting for ${label}: ${JSON.stringify(await readLive(page))}`);
}

async function waitForLineageReset(page, oldLineage, oldResetCount, timeout = 10_000) {
  try {
    await page.waitForFunction((previous) => {
      const root = document.querySelector("#screen-run");
      return root?.dataset?.liveTraceReactionRuntime === "1"
        && root?.dataset?.liveTraceReactionLineageId
        && root.dataset.liveTraceReactionLineageId !== previous.lineage
        && Number(root.dataset.liveTraceReactionLineageResetCount || "0") >= 1
        && Number(root.dataset.liveTraceReactionSampleCount || "0") >= 1;
    }, { lineage: oldLineage, resetCount: oldResetCount }, { timeout });
  } catch (err) {
    throw new Error(`timed out waiting for lineage reset from ${oldLineage}: ${String(err?.message ?? err)} ${JSON.stringify(await readLive(page))}`);
  }
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
    installFailureCapture(page, failures);
    await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${LESSON_ID}`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.liveTraceReactionRuntime === "1", null, { timeout: 30_000 });
    await page.locator("#btn-pause").click();

    const initial = await readLive(page);
    assert(initial.root.liveTraceReactionClaim === "1", "live reaction claim dataset missing");
    assert(initial.root.liveTraceReactionMode === "live_committed_observation", "live mode mismatch");
    assert(initial.root.liveTraceReactionOncePerTick === "1", "once-per-tick dataset missing");
    assert(Number(initial.root.liveTraceReactionLineageResetCount || "0") >= 1, "initial lineage reset missing");
    assert(initial.live?.state_hash_participation === false, "live reaction entered state hash boundary");
    assert(initial.offline?.live_online_claim === false, "offline path overclaimed live online");

    const zero = await stepUntil(page, (state) => String(state.root.liveTraceReactionFiredActions ?? "").includes("highlight.theta_measure"), "zero crossing live action");
    assert(zero.root.liveTraceReactionFiredTicks.includes("26"), `zero live fired tick mismatch: ${zero.root.liveTraceReactionFiredTicks}`);
    assert(zero.live?.emissions?.some((row) => row.tick_id === 26 && row.selected_actions?.includes("highlight.theta_measure")), "zero live emission missing");

    const repeatedBefore = zero.live?.committed_observation_hash_candidate;
    await page.evaluate(() => {
      const runScreen = window.__SEAMGRIM_LIVE_TRACE_REACTION_RUNTIME__;
      window.__SEAMGRIM_LIVE_TRACE_REACTION_DUPLICATE_PROBE__ = runScreen;
    });
    await page.waitForTimeout(30);
    const still = await readLive(page);
    assert(still.live?.committed_observation_hash_candidate === repeatedBefore, "live report changed without a new commit");

    const extremum = await stepUntil(page, (state) => String(state.root.liveTraceReactionFiredActions ?? "").includes("camera_focus.bob"), "extremum live action");
    assert(extremum.root.liveTraceReactionFiredTicks.includes("50"), `extremum live fired tick mismatch: ${extremum.root.liveTraceReactionFiredTicks}`);
    assert(extremum.live?.emissions?.some((row) => row.tick_id === 50 && row.selected_actions?.includes("camera_focus.bob")), "extremum live emission missing");
    assert(extremum.live?.camera_focuses?.some((row) => row.target_anchor === "obs.bob"), "live camera focus target missing");

    const oldLineage = extremum.root.liveTraceReactionLineageId;
    const oldResetCount = Number(extremum.root.liveTraceReactionLineageResetCount || "0");
    await page.locator("[data-testid='lesson-control-panel'] input[type='range']").first().evaluate((input) => {
      input.value = String(Number(input.value || "0") + Number(input.step || "0.1"));
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    });
    try {
      await waitForLineageReset(page, oldLineage, oldResetCount, 5_000);
    } catch (_) {
      const runButton = page.locator("#btn-run");
      if (!(await runButton.isEnabled())) {
        await page.locator("#btn-reset").click();
        await page.waitForFunction(() => document.querySelector("#btn-run") && !document.querySelector("#btn-run").disabled, null, { timeout: 10_000 });
      }
      await runButton.click();
      await waitForLineageReset(page, oldLineage, oldResetCount, 30_000);
    }
    const changed = await readLive(page);
    assert(Number(changed.root.liveTraceReactionSampleCount || "0") >= 1, `lineage reset sample count mismatch: ${changed.root.liveTraceReactionSampleCount}`);
    assert(changed.root.liveTraceReactionLineageId !== oldLineage, "lineage did not change after control update");

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
