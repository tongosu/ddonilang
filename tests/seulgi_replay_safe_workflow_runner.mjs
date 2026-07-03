#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seulgi_replay_safe_workflow: ok";

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
    "seulgi_replay_safe_workflow.js",
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
    await page.waitForSelector("[data-seulgi-replay-safe-workflow][data-seulgi-replay-safe-status='seulgi_replay_safe_workflow_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_SEULGI_REPLAY_SAFE_ROWS,
        buildSeulgiReplaySafeWorkflow,
        formatSeulgiReplaySafeWorkflowText,
      } = await import("./seulgi_replay_safe_workflow.js");
      const workflow = buildSeulgiReplaySafeWorkflow({ rows: DEFAULT_SEULGI_REPLAY_SAFE_ROWS });
      return { workflow, text: formatSeulgiReplaySafeWorkflowText(workflow) };
    });
    const workflow = moduleResult.workflow;
    assert(workflow.schema === "ddn.ja.seulgi_replay_safe_workflow.v1", "schema mismatch");
    assert(workflow.work_item === "JA5_REPLAY_SAFE_AI_WORKFLOW_V1", "work item mismatch");
    assert(workflow.primary_coordinate === "자-5", "coordinate mismatch");
    assert(workflow.depends_on_coordinate.join(",") === "자-2,자-3,자-4", "dependency mismatch");
    assert(workflow.pack === "seulgi_replay_safe_workflow_v1", "pack mismatch");
    assert(workflow.status === "seulgi_replay_safe_workflow_ready", "status mismatch");
    assert(workflow.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(workflow.replay_safe_workflow_claim === true, "workflow claim mismatch");
    assert(workflow.input_snapshot_claim === true, "input snapshot mismatch");
    assert(workflow.approval_receipt_claim === true, "approval receipt mismatch");
    assert(workflow.artifact_hash_claim === true, "artifact hash mismatch");
    assert(workflow.no_ai_recall_claim === true, "no AI recall mismatch");
    assert(workflow.lts_gate_claim === true, "LTS gate mismatch");
    assert(workflow.ai_recall_claim === false, "AI recall must stay false");
    assert(workflow.auto_apply_claim === false, "auto apply must stay false");
    assert(workflow.file_write_claim === false, "file write must stay false");
    assert(workflow.runtime_ast_persisted === false, "runtime AST persistence must stay false");
    assert(workflow.state_hash_owner === false, "state hash owner must stay false");
    assert(workflow.model_training_claim === false, "model training must stay false");
    assert(workflow.runtime_claim === false, "runtime claim must stay false");
    assert(workflow.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(workflow.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(workflow.progress.roadmap_v2_matrix_behavior_closed === 75, "roadmap closed mismatch");
    assert(workflow.progress.roadmap_v2_matrix_behavior_percent === 83, "roadmap percent mismatch");
    assert(workflow.progress.roadmap_v2_pack_evidence_reference_closed === 77, "pack ref mismatch");
    assert(workflow.progress.roadmap_v2_pack_evidence_reference_percent === 86, "pack ref percent mismatch");
    assert(String(workflow.replay_text).includes("ai_recall:false"), "replay text missing AI recall boundary");
    assert(String(workflow.replay_text).includes("no_recall"), "replay text missing no recall row");
    assert(String(moduleResult.text).includes("roadmap_matrix\t75/90"), "text missing roadmap matrix");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t77/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-seulgi-replay-safe-workflow]");
      const buttons = Array.from(document.querySelectorAll("[data-seulgi-replay-row]"));
      buttons.find((button) => button.getAttribute("data-seulgi-replay-row") === "no_recall")?.click();
      document.querySelector("[data-seulgi-replay-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-seulgi-replay-safe-status") || "",
        copied: root?.getAttribute("data-seulgi-replay-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-seulgi-replay-artifact]").length,
        progress: document.querySelector("[data-seulgi-replay-progress]")?.textContent || "",
        summary: document.querySelector("[data-seulgi-replay-summary]")?.textContent || "",
        title: document.querySelector("[data-seulgi-replay-active-title]")?.textContent || "",
        link: document.querySelector("[data-seulgi-replay-active-link]")?.textContent || "",
        preview: document.querySelector("[data-seulgi-replay-preview]")?.textContent || "",
        globalSchema: window.__SEULGI_REPLAY_SAFE_WORKFLOW__?.schema || "",
        globalText: window.__SEULGI_REPLAY_SAFE_WORKFLOW_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "seulgi_replay_safe_workflow_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("75/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("AI recall") && domResult.summary.includes("auto-apply") && domResult.summary.includes("file write"), "summary missing boundary");
    assert(domResult.title === "AI 재호출 없음", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("seulgi://replay/no-ai-recall"), "no-recall URI mismatch");
    assert(domResult.preview.includes("ai_recall:false") && domResult.preview.includes("state_hash_owner:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.ja.seulgi_replay_safe_workflow.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t75/90"), "global text missing roadmap matrix");
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
