#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function mimeType(file) {
  if (file.endsWith(".js") || file.endsWith(".mjs")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  return "application/octet-stream";
}

async function serve(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      if (url.pathname === "/") {
        res.writeHead(200, { "content-type": "text/html; charset=utf-8", "cache-control": "no-store" });
        res.end("<!doctype html><meta charset='utf-8'><div id='table'></div><div id='text'></div><div id='structure'></div>");
        return;
      }
      const relative = decodeURIComponent(url.pathname).replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, relative);
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
        res.writeHead(403).end("forbidden");
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
      res.writeHead(404).end("not found");
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

const root = process.cwd();
const { server, baseUrl } = await serve(root);
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage();
  await page.goto(baseUrl, { waitUntil: "domcontentloaded" });
  const result = await page.evaluate(async () => {
    const fixture = await fetch("/pack/seamgrim_table_text_structure_canonical_view_payload_v1/fixtures/authoritative_state.detjson").then((row) => row.json());
    const stateMod = await import("/solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js");
    const payloadMod = await import("/solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js");
    const tableMod = await import("/solutions/seamgrim_ui_mvp/ui/components/table_preview.js");
    const structureMod = await import("/solutions/seamgrim_ui_mvp/ui/components/structure_preview.js");
    const markdownMod = await import("/solutions/seamgrim_ui_mvp/ui/components/markdown.js");
    const payloads = payloadMod.buildCanonicalViewPayloadSet(
      stateMod.extractCanonicalViewAuthorityInputsFromState(fixture),
    );
    document.querySelector("#table").innerHTML = tableMod.buildTablePreviewHtml(payloads.table.content.table);
    document.querySelector("#text").innerHTML = markdownMod.markdownToHtml(payloads.text.content.text.content);
    document.querySelector("#structure").innerHTML = structureMod.buildStructurePreviewHtml(payloads.structure.content.structure);
    return {
      statuses: [payloads.table.status, payloads.text.status, payloads.structure.status],
      canonical_bytes: ["table", "text", "structure"].map((family) => payloadMod.stableSemanticJson(payloads[family])),
      table_rows: document.querySelectorAll("#table tbody tr").length,
      table_columns: document.querySelectorAll("#table thead th").length,
      text_heading: document.querySelector("#text h1")?.textContent ?? "",
      structure_nodes: document.querySelectorAll("#structure .runtime-structure-node").length,
      structure_edges: document.querySelectorAll("#structure .runtime-structure-edge").length,
    };
  });
  assert(result.statuses.every((status) => status === "available"), "browser canonical family unavailable");
  assert(result.canonical_bytes.every((bytes) => bytes.includes("seamgrim.semantic_view_payload.v1_candidate")), "browser canonical bytes missing");
  assert(result.table_rows === 2 && result.table_columns === 2, "browser table renderer drift");
  assert(result.text_heading === "실행 설명", "browser text renderer drift");
  assert(result.structure_nodes === 2 && result.structure_edges === 1, "browser structure renderer drift");
  console.log(JSON.stringify({ status: "PASS", ...result }));
} finally {
  await browser.close();
  await new Promise((resolve) => server.close(resolve));
}
