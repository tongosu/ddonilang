#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "rpg_engine_adapter_lts: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) throw new Error(`missing file: ${file}`);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  return "text/plain; charset=utf-8";
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
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/index.html" : url.pathname);
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
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

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    return (
      url.pathname === "/api/lessons/inventory" ||
      url.pathname === "/api/lesson-inventory" ||
      url.pathname.startsWith("/lessons/") ||
      url.pathname.startsWith("/seed_lessons_v1/")
    );
  } catch (_) {
    return false;
  }
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "rpg_story_package.js",
    "rpg_engine_adapter_lts.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 840 }, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error" && !String(msg.text() ?? "").includes("Failed to load resource")) {
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400 && !res.url().endsWith("/favicon.ico") && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-rpg-engine-adapter-lts][data-rpg-engine-adapter-lts-status='rpg_engine_adapter_lts_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_RPG_ENGINE_ADAPTER_LTS_ROWS,
        buildRpgEngineAdapterLts,
        formatRpgEngineAdapterLtsText,
      } = await import("./rpg_engine_adapter_lts.js");
      const adapter = buildRpgEngineAdapterLts({ rows: DEFAULT_RPG_ENGINE_ADAPTER_LTS_ROWS });
      return {
        adapter,
        text: formatRpgEngineAdapterLtsText(adapter),
      };
    });
    const adapter = moduleResult.adapter;
    assert(adapter.schema === "ddn.seamgrim.rpg_engine.adapter_lts.v1", "schema mismatch");
    assert(adapter.work_item === "CHA5_RPG_ENGINE_ADAPTER_LTS_V1", "work item mismatch");
    assert(adapter.primary_coordinate === "차-5", "coordinate mismatch");
    assert(adapter.status === "rpg_engine_adapter_lts_ready", "status mismatch");
    assert(adapter.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(adapter.adapter_lts_claim === true, "adapter LTS claim mismatch");
    assert(adapter.godot_manifest_claim === true, "godot manifest claim mismatch");
    assert(adapter.native_bridge_contract_claim === true, "native bridge claim mismatch");
    assert(adapter.asset_map_claim === true, "asset map claim mismatch");
    assert(adapter.lts_gate_claim === true, "lts gate claim mismatch");
    assert(adapter.product_ui_change === true, "product UI change mismatch");
    assert(adapter.runtime_claim === false, "runtime claim must stay false");
    assert(adapter.native_runtime_execution_claim === false, "native execution must stay false");
    assert(adapter.godot_project_build_claim === false, "godot build must stay false");
    assert(adapter.native_binary_claim === false, "native binary must stay false");
    assert(adapter.cloud_sync_claim === false, "cloud sync must stay false");
    assert(adapter.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(adapter.progress.current_stage_total === 5, "stage total mismatch");
    assert(adapter.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(adapter.progress.roadmap_v2_matrix_behavior_closed === 17, "roadmap closed mismatch");
    assert(adapter.progress.roadmap_v2_matrix_behavior_percent === 19, "roadmap percent mismatch");
    assert(adapter.progress.roadmap_v2_pack_evidence_reference_closed === 37, "pack ref mismatch");
    assert(adapter.progress.roadmap_v2_pack_evidence_reference_percent === 41, "pack ref percent mismatch");
    assert(adapter.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(adapter.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(adapter.adapters.map((row) => row.id).join(",") === "godot_manifest,native_bridge,asset_map,lts_gate", "adapter order mismatch");
    assert(adapter.adapter_files.map((file) => file.kind).join(",") === "godot_manifest,native_bridge,asset_map,lts_gate", "adapter file order mismatch");
    assert(String(adapter.adapter_text).includes("runtime_execution:false"), "adapter text missing runtime boundary");
    assert(String(moduleResult.text).includes("adapter_lts_claim\ttrue"), "text missing adapter claim");
    assert(String(moduleResult.text).includes("native_runtime_execution_claim\tfalse"), "text missing native execution boundary");
    assert(String(moduleResult.text).includes("roadmap_matrix\t17/90"), "text missing roadmap progress");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-rpg-engine-adapter-lts]");
      const buttons = Array.from(document.querySelectorAll(".rpg-engine-adapter-btn[data-rpg-engine-adapter]"));
      buttons.find((button) => button.getAttribute("data-rpg-engine-adapter") === "native_bridge")?.click();
      document.querySelector("[data-rpg-engine-adapter-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-rpg-engine-adapter-lts-status") || "",
        copied: root?.getAttribute("data-rpg-engine-adapter-lts-copied") || "",
        buttonCount: buttons.length,
        fileCount: document.querySelectorAll("[data-rpg-engine-adapter-file]").length,
        progress: document.querySelector("[data-rpg-engine-adapter-progress]")?.textContent || "",
        summary: document.querySelector("[data-rpg-engine-adapter-summary]")?.textContent || "",
        title: document.querySelector("[data-rpg-engine-adapter-active-title]")?.textContent || "",
        link: document.querySelector("[data-rpg-engine-adapter-active-link]")?.textContent || "",
        preview: document.querySelector("[data-rpg-engine-adapter-preview]")?.textContent || "",
        globalSchema: window.__SEAMGRIM_RPG_ENGINE_ADAPTER_LTS__?.schema || "",
        globalText: window.__SEAMGRIM_RPG_ENGINE_ADAPTER_LTS_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "rpg_engine_adapter_lts_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `adapter count mismatch: ${domResult.buttonCount}`);
    assert(domResult.fileCount === 4, `adapter file count mismatch: ${domResult.fileCount}`);
    assert(domResult.progress.includes("17/90 ROADMAP") && domResult.progress.includes("5/5 stage"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("Godot manifest") && domResult.summary.includes("native bridge") && domResult.summary.includes("native execution"), "summary missing adapter workflow");
    assert(domResult.title === "Native bridge", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("seamgrim://rpg-engine-adapter/local/native/"), "native bridge local URI mismatch");
    assert(domResult.preview.includes("godot.project.manifest.detjson") && domResult.preview.includes("runtime_execution:false"), "adapter preview mismatch");
    assert(domResult.globalSchema === "ddn.seamgrim.rpg_engine.adapter_lts.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t37/90"), "global text missing pack reference");
    assert(domResult.globalText.includes("native_binary_claim\tfalse"), "global text missing binary boundary");

    if (failures.length > 0) throw new Error(failures.join("\n"));
    await context.close();
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
