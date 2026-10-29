#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_sealed_observation_trace: ok";
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

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
    await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${LESSON_ID}`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
    await page.waitForFunction(() => {
      const reaction = window.__SEAMGRIM_TRACE_REACTION_RUNTIME__;
      return reaction?.observation_trace?.schema === "seamgrim.sealed_observation_trace.v1"
        && Number(reaction?.trace_sample_count ?? 0) >= 60;
    }, null, { timeout: 30_000 });
    const pause = page.locator("#btn-pause");
    if (await pause.isEnabled()) await pause.click();
    await page.waitForTimeout(150);
    const state = await page.evaluate(() => {
      const root = document.querySelector("#screen-run");
      const reaction = window.__SEAMGRIM_TRACE_REACTION_RUNTIME__;
      return {
        root: { ...(root?.dataset ?? {}) },
        reaction,
        stateHash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      };
    });
    const trace = state.reaction.observation_trace;
    assert(trace.schema === "seamgrim.sealed_observation_trace.v1", "trace schema mismatch");
    assert(trace.sealed === true, "trace must be sealed");
    assert(trace.state_hash_participation === false, "trace must stay outside state_hash");
    assert(trace.provenance?.frontdoor === "seamgrim_product_wasm", "frontdoor provenance mismatch");
    assert(trace.provenance?.console_string_parse === false, "trace must not come from console string parsing");
    assert(String(trace.trace_hash_candidate ?? "").startsWith("candidate:fnv1a32:sealed_observation_trace:"), "trace hash candidate missing");
    assert(state.root.traceReactionObservationTraceHash === trace.trace_hash_candidate, "trace hash dataset mismatch");
    assert(Number(state.root.traceReactionObservationSampleCount) === trace.sample_count, "trace sample count dataset mismatch");
    const samples = Array.isArray(trace.samples) ? trace.samples : [];
    assert(samples.length >= 60, `sample count too small: ${samples.length}`);
    for (let index = 0; index < samples.length; index += 1) {
      const row = samples[index];
      assert(row.sample_index === index, `sample index mismatch: ${index}`);
      for (const key of ["tick_id", "t", "theta", "omega"]) {
        assert(Number.isFinite(Number(row[key])), `non-finite sample ${key} at ${index}`);
      }
      if (index > 0) {
        assert(row.tick_id > samples[index - 1].tick_id, `tick order violation at ${index}`);
      }
    }
    const baseStateHash = state.stateHash;
    await page.locator("#select-view-plan-camera-policy").selectOption("fixed");
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.viewPlanCameraPolicy === "fixed");
    const afterCamera = await page.evaluate(() => ({
      stateHash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      traceHash: window.__SEAMGRIM_TRACE_REACTION_RUNTIME__?.observation_trace_hash_candidate ?? "",
      sampleCount: window.__SEAMGRIM_TRACE_REACTION_RUNTIME__?.trace_sample_count ?? 0,
      tail: window.__SEAMGRIM_TRACE_REACTION_RUNTIME__?.observation_trace?.samples?.slice(-3) ?? [],
    }));
    assert(afterCamera.stateHash === baseStateHash, "state_hash changed after view-only camera change");
    assert(afterCamera.traceHash === trace.trace_hash_candidate, `trace artifact changed after view-only camera change: ${JSON.stringify({
      before: trace.trace_hash_candidate,
      after: afterCamera.traceHash,
      beforeCount: trace.sample_count,
      afterCount: afterCamera.sampleCount,
      beforeTail: trace.samples.slice(-3),
      afterTail: afterCamera.tail,
    })}`);
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
