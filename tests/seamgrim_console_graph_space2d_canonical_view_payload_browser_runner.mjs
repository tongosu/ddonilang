#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function sha256(text) {
  return crypto.createHash("sha256").update(text, "utf8").digest("hex");
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "text/javascript; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  return "application/octet-stream";
}

async function serve(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (request, response) => {
    try {
      const url = new URL(request.url || "/", "http://127.0.0.1");
      if (url.pathname === "/") {
        response.writeHead(200, { "content-type": "text/html; charset=utf-8", "cache-control": "no-store" });
        response.end("<!doctype html><meta charset='utf-8'><pre id='console'></pre><div id='graph'></div><div id='space2d'></div>");
        return;
      }
      const relative = decodeURIComponent(url.pathname).replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, relative);
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
        response.writeHead(403).end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      response.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
      response.end(bytes);
    } catch (_) {
      response.writeHead(404).end("not found");
    }
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert(address && typeof address !== "string", "browser server address unavailable");
  return { server, baseUrl: `http://127.0.0.1:${address.port}` };
}

const root = path.resolve(import.meta.dirname, "..");
const { server, baseUrl } = await serve(root);
let browser = null;

try {
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 960, height: 720 }, locale: "ko-KR" });
  const pageErrors = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));
  await page.goto(baseUrl, { waitUntil: "networkidle" });

  const result = await page.evaluate(async () => {
    const [t5bState, t5cState, expectedConsole, expectedGraph, expectedSpace2d] = await Promise.all([
      fetch("/pack/seamgrim_console_graph_canonical_view_payload_v1/fixtures/authoritative_state.detjson").then((row) => row.json()),
      fetch("/pack/seamgrim_space2d_canonical_view_payload_v1/fixtures/authoritative_state.detjson").then((row) => row.json()),
      fetch("/pack/seamgrim_console_graph_canonical_view_payload_v1/expected/console.payload.detjson").then((row) => row.json()),
      fetch("/pack/seamgrim_console_graph_canonical_view_payload_v1/expected/graph.payload.detjson").then((row) => row.json()),
      fetch("/pack/seamgrim_space2d_canonical_view_payload_v1/expected/space2d.payload.detjson").then((row) => row.json()),
    ]);
    const stateModule = await import("/solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js");
    const payloadModule = await import("/solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js");
    const graphModule = await import("/solutions/seamgrim_ui_mvp/ui/components/graph_preview.js");
    const previewModule = await import("/solutions/seamgrim_ui_mvp/ui/components/preview_registry.js");

    const t5bPayloads = payloadModule.buildCanonicalViewPayloadSet(
      stateModule.extractCanonicalViewAuthorityInputsFromState(t5bState),
    );
    const t5cPayloads = payloadModule.buildCanonicalViewPayloadSet(
      stateModule.extractCanonicalViewAuthorityInputsFromState(t5cState),
    );
    const consolePayload = t5bPayloads.console;
    const graphPayload = t5bPayloads.graph;
    const space2dPayload = t5cPayloads.space2d;

    document.querySelector("#console").textContent = consolePayload.content.ordered_entries
      .map((entry) => entry.text)
      .join("\n");
    document.querySelector("#graph").innerHTML = graphModule.buildGraphPreviewHtml(graphPayload.content.graph, {
      containerClass: "probe-graph",
      canvasClass: "probe-graph-canvas",
      axisClass: "probe-graph-axis",
      lineClass: "probe-graph-line",
      lineVariantClasses: ["probe-graph-line-a", "probe-graph-line-b"],
    });
    document.querySelector("#space2d").innerHTML = previewModule.buildSpace2dPreviewHtml(space2dPayload.content.space2d, {
      containerClass: "probe-space2d",
      canvasClass: "probe-space2d-canvas",
      pointClass: "probe-space2d-point",
      lineClass: "probe-space2d-line",
      circleClass: "probe-space2d-circle",
    });

    return {
      statuses: [consolePayload.status, graphPayload.status, space2dPayload.status],
      families: [consolePayload.family, graphPayload.family, space2dPayload.family],
      exact_golden_parity: [
        payloadModule.stableSemanticJson(consolePayload) === payloadModule.stableSemanticJson(expectedConsole),
        payloadModule.stableSemanticJson(graphPayload) === payloadModule.stableSemanticJson(expectedGraph),
        payloadModule.stableSemanticJson(space2dPayload) === payloadModule.stableSemanticJson(expectedSpace2d),
      ],
      console_text: document.querySelector("#console").textContent,
      graph_path_count: document.querySelectorAll("#graph path.probe-graph-line").length,
      graph_point_count: document.querySelectorAll("#graph circle").length,
      space2d_line_count: document.querySelectorAll("#space2d line.probe-space2d-line").length,
      space2d_point_count: document.querySelectorAll("#space2d circle.probe-space2d-point").length,
      space2d_circle_count: document.querySelectorAll("#space2d circle.probe-space2d-circle").length,
      canonical_bytes: [consolePayload, graphPayload, space2dPayload].map(payloadModule.stableSemanticJson),
    };
  });

  assert(result.statuses.every((status) => status === "available"), "T5B/T5C browser canonical family unavailable");
  assert(JSON.stringify(result.families) === JSON.stringify(["console", "graph", "space2d"]), "T5B/T5C browser family drift");
  assert(result.exact_golden_parity.every(Boolean), "T5B/T5C browser exact golden parity drift");
  assert(result.console_text === "준비\n위치 12.5", "browser console order/value drift");
  assert(result.graph_path_count === 2 && result.graph_point_count === 6, "browser graph renderer drift");
  assert(
    result.space2d_line_count === 1 && result.space2d_point_count === 2 && result.space2d_circle_count === 1,
    "browser space2d renderer drift",
  );
  assert(
    result.canonical_bytes.every((bytes) => bytes.includes("seamgrim.semantic_view_payload.v1_candidate")),
    "browser canonical bytes missing",
  );
  assert(pageErrors.length === 0, `browser page errors: ${pageErrors.join(" | ")}`);

  const { canonical_bytes: canonicalBytes, ...summary } = result;
  console.log(JSON.stringify({
    status: "PASS",
    schema: "seamgrim.console_graph_space2d.browser_result.v1",
    browser_family_probe_count: 3,
    network_install: 0,
    page_errors: 0,
    canonical_byte_sha256: canonicalBytes.map(sha256),
    ...summary,
  }));
} finally {
  if (browser) await browser.close();
  await new Promise((resolve) => server.close(resolve));
}
