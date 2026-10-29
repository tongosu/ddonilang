#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const root = process.cwd();
const contentTypes = new Map([
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".json", "application/json; charset=utf-8"],
]);

const server = http.createServer(async (request, response) => {
  try {
    const pathname = decodeURIComponent(new URL(request.url, "http://127.0.0.1").pathname);
    if (pathname === "/t4f-probe.html") {
      response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
      response.end("<!doctype html><html lang=\"ko\"><body><main id=\"probe\">T4f</main></body></html>");
      return;
    }
    if (pathname === "/favicon.ico") {
      response.writeHead(204);
      response.end();
      return;
    }
    const relative = pathname.replace(/^\/+/, "");
    const filePath = path.resolve(root, relative);
    assert(filePath.startsWith(`${root}${path.sep}`), "request escaped root");
    const bytes = await fs.readFile(filePath);
    response.writeHead(200, { "content-type": contentTypes.get(path.extname(filePath)) ?? "application/octet-stream" });
    response.end(bytes);
  } catch (error) {
    response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    response.end(String(error?.message ?? error));
  }
});

await new Promise((resolve, reject) => {
  server.once("error", reject);
  server.listen(0, "127.0.0.1", resolve);
});
const address = server.address();
assert(address && typeof address !== "string", "server address unavailable");
assert.notEqual(address.port, 8787, "protected port 8787 must not be used");
const baseUrl = `http://127.0.0.1:${address.port}`;
let browser = null;
const errors = [];

try {
  browser = await chromium.launch({ channel: "msedge", headless: true });
  const page = await browser.newPage({ locale: "ko-KR" });
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`console:${message.text()}`);
  });
  page.on("pageerror", (error) => errors.push(`pageerror:${error.message}`));
  page.on("requestfailed", (request) => errors.push(`requestfailed:${request.url()}`));
  await page.goto(`${baseUrl}/t4f-probe.html`, { waitUntil: "networkidle" });
  const result = await page.evaluate(async ({ origin }) => {
    const stateMod = await import(`${origin}/solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js`);
    const runMod = await import(`${origin}/solutions/seamgrim_ui_mvp/ui/screens/run.js`);
    const missingState = {
      state_hash: "blake3:browser-preserved",
      channels: [{ key: "보개_점", dtype: "f64" }],
      row: [42],
    };
    const missing = stateMod.extractObservationChannelsFromState(missingState);
    const derived = runMod.extractRuntimeDerived(missingState);
    const runKind = runMod.deriveRunKindAndChannels({ observation: derived.observation, hasSpace2d: false });
    const manifest = {
      schema: "ddn.observation_manifest.v0",
      version: "20.6.33",
      nodes: [{ name: "보개_점", dtype: "f64", role: "뷰" }],
      params: [],
      traces: [],
      diagnostics: [],
      default_pivot: { x_channel: "보개_점", y_channels: [] },
    };
    const available = stateMod.extractObservationChannelsFromState({
      ...missingState,
      observation_manifest: manifest,
    });
    return {
      missingStatus: missing.status,
      missingCode: missing.diagnostic?.code,
      missingChannels: missing.channels.length,
      stateHash: missingState.state_hash,
      graph: derived.views.graph,
      graphSource: derived.views.graphSource,
      runKind,
      availableStatus: available.status,
      availableChannel: available.channels[0],
      availableValue: available.values["보개_점"],
    };
  }, { origin: baseUrl });

  assert.equal(result.missingStatus, "unavailable", "browser missing manifest fails closed");
  assert.equal(result.missingCode, "E_WASM_OBSERVATION_MANIFEST_MISSING", "browser exact missing code");
  assert.equal(result.missingChannels, 0, "browser synthetic channel count zero");
  assert.equal(result.stateHash, "blake3:browser-preserved", "browser state hash preserved");
  assert.equal(result.graph, null, "browser synthetic graph absent");
  assert.equal(result.graphSource, null, "browser graph source absent");
  assert.deepEqual(result.runKind, { kind: "observation_unavailable", channels: 0 }, "browser Run outcome separated");
  assert.equal(result.availableStatus, "available", "browser native manifest available");
  assert.deepEqual(
    result.availableChannel,
    { key: "보개_점", dtype: "f64", role: "뷰", unit: undefined },
    "browser native semantics preserved",
  );
  assert.equal(result.availableValue, 42, "browser native value bound");
  assert.deepEqual(errors, [], `browser console/page/network errors: ${JSON.stringify(errors)}`);

  console.log(JSON.stringify({
    schema: "seamgrim.workbench.observation_channel_t4f.browser_result.v1",
    browser: "Microsoft Edge",
    manifest_present: "PASS",
    manifest_missing: "FAIL_CLOSED",
    execution_outcome: "PRESERVED",
    observation_projection: "UNAVAILABLE",
    synthetic_graph_count: 0,
    synthetic_channel_count: 0,
    console_page_network_errors: 0,
    protected_port_8787_untouched: true,
    result: "PASS",
  }));
} finally {
  if (browser) await browser.close();
  await new Promise((resolve) => server.close(resolve));
}
