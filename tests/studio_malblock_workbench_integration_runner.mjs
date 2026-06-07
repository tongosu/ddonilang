#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_malblock_workbench_integration: ok";

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
    return url.pathname === "/api/lessons/inventory" || url.pathname === "/api/lesson-inventory";
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
    "screens/block_editor.js",
    "block_editor/ddn_block_engine.js",
    "block_editor/ddn_block_codec.js",
    "block_editor/seamgrim_palette.js",
    "studio_malblock_workbench_integration.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 860 }, locale: "ko-KR" });
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    const result = await page.evaluate(async () => {
      const { BlockEditorScreen } = await import("./screens/block_editor.js");
      const {
        buildMalblockWorkbenchIntegration,
        formatMalblockWorkbenchIntegrationText,
      } = await import("./studio_malblock_workbench_integration.js");
      const host = document.createElement("section");
      host.id = "malblock-workbench-integration-host";
      host.innerHTML = `
        <span id="block-editor-title"></span>
        <select id="block-editor-mode">
          <option value="seamgrim">seamgrim</option>
        </select>
        <div id="block-editor-result"></div>
        <div id="block-editor-summary"></div>
        <textarea id="block-ddn-preview"></textarea>
        <div id="block-palette"></div>
        <div id="block-canvas"></div>
        <button id="btn-back-from-block-editor" type="button"></button>
        <button id="btn-block-to-text" type="button"></button>
        <button id="btn-block-run" type="button"></button>
        <button id="btn-advanced-block-editor" type="button"></button>
      `;
      document.body.appendChild(host);
      const textModeCalls = [];
      const runCalls = [];
      const fakeCanon = {
        async canonBlockEditorPlan(source) {
          if (String(source ?? "").includes("BROKEN")) {
            throw new Error("malblock_workbench_decode_error");
          }
          return {
            schema: "ddn.block_editor_plan.v1",
            blocks: [
              {
                kind: "charim_item_var",
                fields: { name: "t", type_name: "수", value: "0" },
                exprs: {},
                inputs: {},
              },
            ],
          };
        },
      };
      const screen = new BlockEditorScreen({
        root: host,
        canon: fakeCanon,
        onTextMode(ddnText, meta) {
          textModeCalls.push({ ddnText: String(ddnText ?? ""), mode: String(meta?.mode ?? "") });
        },
        onRun(ddnText, meta) {
          runCalls.push({ ddnText: String(ddnText ?? ""), mode: String(meta?.mode ?? "") });
        },
      });
      screen.init();
      await screen.loadSource("채비 { t:수 <- 0. }.", { title: "Malblock Workbench", mode: "seamgrim" });
      const showButton = Array.from(host.querySelectorAll("#block-palette .block-category__item"))
        .find((button) => button.dataset.kind === "show");
      if (!showButton) throw new Error("show palette button missing");
      showButton.click();
      const ddnText = screen.getDdn();
      host.querySelector("#btn-block-to-text").click();
      host.querySelector("#btn-block-run").click();
      await screen.loadSource("BROKEN", { title: "Broken Workbench", mode: "seamgrim" });
      const decodeSummary = screen.engine.lastCanvasSummary;
      const workflow = buildMalblockWorkbenchIntegration({
        paletteSummary: screen.engine.lastPaletteSummary,
        canvasSummary: {
          block_count: 2,
          block_kinds: ["charim_item_var", "show"],
        },
        ddnText,
        callbacks: {
          text_mode_count: textModeCalls.length,
          run_count: runCalls.length,
          last_text_mode_ddn: textModeCalls.at(-1)?.ddnText ?? "",
          last_run_ddn: runCalls.at(-1)?.ddnText ?? "",
        },
        saveState: {
          local_save_available: true,
          filename: "lesson.ddn",
          remote_save_claim: false,
        },
        runRequest: {
          launch_kind: "block_editor_run",
          source_type: "ddn",
        },
        decodeState: {
          error_count: 1,
          raw_fallback_count: Array.isArray(decodeSummary?.block_kinds)
            ? decodeSummary.block_kinds.filter((kind) => kind === "raw").length
            : 0,
          error_text: host.querySelector("#block-editor-result")?.textContent ?? "",
        },
        workbenchContext: {
          source_mode: "malblock",
          target_mode: "studio_run",
          workbench_shell_reused: true,
          lesson_schema_change: false,
          active_allowlist_mutation: false,
          parser_frontdoor_change: false,
          runtime_claim: false,
        },
      });
      return {
        workflow,
        text: formatMalblockWorkbenchIntegrationText(workflow),
      };
    });

    const workflow = result.workflow;
    assert(workflow.schema === "seamgrim.malblock_workbench_integration.v1", "workflow schema mismatch");
    assert(workflow.primary_coordinate === "마-3", `primary coordinate mismatch: ${workflow.primary_coordinate}`);
    assert(workflow.support_coordinate === "라-3", `support coordinate mismatch: ${workflow.support_coordinate}`);
    assert(workflow.workflow_claim === "malblock_workbench_integration", `workflow claim mismatch: ${workflow.workflow_claim}`);
    assert(workflow.generated_locally === true, "workflow must be local");
    assert(workflow.lesson_schema_change === false, "workflow must not claim lesson schema change");
    assert(workflow.active_allowlist_mutation === false, "workflow must not claim allowlist mutation");
    assert(workflow.parser_frontdoor_change === false, "workflow must not claim parser/frontdoor change");
    assert(workflow.runtime_claim === false, "workflow must not claim runtime change");
    assert(workflow.remote_save_claim === false, "workflow must not claim remote save");
    assert(workflow.replay_claim === false, "workflow must not claim replay");
    assert(workflow.status === "malblock_workbench_ready", `status mismatch: ${workflow.status}`);
    assert(workflow.stage_count === 9, `stage count mismatch: ${workflow.stage_count}`);
    assert(workflow.ready_stage_count === 9, `ready stage count mismatch: ${workflow.ready_stage_count}`);
    assert(workflow.missing_stage_count === 0, `missing stage count mismatch: ${workflow.missing_stage_count}`);
    assert(workflow.palette_category_count >= 4, "palette category count too small");
    assert(workflow.palette_category_ids.includes("charim"), "palette charim category missing");
    assert(workflow.canvas_block_count === 2, `canvas block count mismatch: ${workflow.canvas_block_count}`);
    assert(workflow.canvas_block_kinds.includes("show"), "show block missing");
    assert(workflow.ddn_line_count >= 2, `ddn line count mismatch: ${workflow.ddn_line_count}`);
    assert(workflow.text_mode_count === 1, "text callback count mismatch");
    assert(workflow.run_count === 1, "run callback count mismatch");
    assert(workflow.local_save_filename === "lesson.ddn", "local save filename mismatch");
    assert(workflow.launch_kind === "block_editor_run", "launch kind mismatch");
    assert(workflow.source_type === "ddn", "source type mismatch");
    assert(workflow.decode_error_count === 1, "decode error count mismatch");
    assert(workflow.raw_fallback_count === 1, "raw fallback count mismatch");
    assert(String(result.text).includes("schema\tseamgrim.malblock_workbench_integration.v1"), "text schema missing");
    assert(String(result.text).includes("support_coordinate\t라-3"), "text support coordinate missing");
    assert(String(result.text).includes("status\tmalblock_workbench_ready"), "text status missing");
    assert(!String(result.text).endsWith("\n"), "workflow text must not have trailing newline");

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
