#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "question_card_workflow_hardening: ok";

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
    "question_card_workflow_hardening.js",
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
    await page.waitForSelector("[data-question-card-workflow-hardening][data-question-card-workflow-hardening-status='question_card_workflow_hardening_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_QUESTION_CARD_WORKFLOW_HARDENING_ROWS,
        buildQuestionCardWorkflowHardening,
        formatQuestionCardWorkflowHardeningText,
      } = await import("./question_card_workflow_hardening.js");
      const hardening = buildQuestionCardWorkflowHardening({ rows: DEFAULT_QUESTION_CARD_WORKFLOW_HARDENING_ROWS });
      return { hardening, text: formatQuestionCardWorkflowHardeningText(hardening) };
    });
    const hardening = moduleResult.hardening;
    assert(hardening.schema === "ddn.geo.question_card_workflow_hardening.v1", "schema mismatch");
    assert(hardening.work_item === "GEO5_AI_WORKFLOW_HARDENING_V1", "work item mismatch");
    assert(hardening.primary_coordinate === "거-5", "coordinate mismatch");
    assert(hardening.depends_on_coordinate.join(",") === "거-4,거-3,거-2,거-1,타-2", "dependency mismatch");
    assert(hardening.pack === "question_card_workflow_hardening_v1", "pack mismatch");
    assert(hardening.status === "question_card_workflow_hardening_ready", "status mismatch");
    assert(hardening.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(hardening.workflow_hardening_claim === true, "workflow hardening claim mismatch");
    assert(hardening.approval_gate_claim === true, "approval gate mismatch");
    assert(hardening.replay_packet_claim === true, "replay packet mismatch");
    assert(hardening.audit_trail_claim === true, "audit trail mismatch");
    assert(hardening.rollback_plan_claim === true, "rollback plan mismatch");
    assert(hardening.lts_gate_claim === true, "LTS gate mismatch");
    assert(hardening.ai_call_claim === false, "AI call must stay false");
    assert(hardening.parser_preprocessor_claim === false, "parser/preprocessor must stay false");
    assert(hardening.auto_apply_claim === false, "auto apply must stay false");
    assert(hardening.patch_execution_claim === false, "patch execution must stay false");
    assert(hardening.file_write_claim === false, "file write must stay false");
    assert(hardening.network_call_claim === false, "network call must stay false");
    assert(hardening.runtime_ast_persisted === false, "runtime AST persistence must stay false");
    assert(hardening.state_hash_owner === false, "state hash owner must stay false");
    assert(hardening.registry_publish_claim === false, "registry publish must stay false");
    assert(hardening.account_permission_change_claim === false, "account permission must stay false");
    assert(hardening.cloud_sync_claim === false, "cloud sync must stay false");
    assert(hardening.release_execution_claim === false, "release execution must stay false");
    assert(hardening.lts_certification_claim === false, "LTS certification must stay false");
    assert(hardening.runtime_claim === false, "runtime claim must stay false");
    assert(hardening.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(hardening.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(hardening.progress.roadmap_v2_matrix_behavior_closed === 37, "roadmap closed mismatch");
    assert(hardening.progress.roadmap_v2_matrix_behavior_percent === 41, "roadmap percent mismatch");
    assert(hardening.progress.roadmap_v2_pack_evidence_reference_closed === 58, "pack ref mismatch");
    assert(hardening.progress.roadmap_v2_pack_evidence_reference_percent === 64, "pack ref percent mismatch");
    assert(hardening.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(hardening.rows.map((row) => row.id).join(",") === "approval_gate,replay_packet,audit_trail,rollback_plan,lts_gate", "row order mismatch");
    assert(String(hardening.hardening_text).includes("release_execution:false"), "hardening text missing release boundary");
    assert(String(hardening.hardening_text).includes("lts_certification:false"), "hardening text missing LTS boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t58/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-question-card-workflow-hardening]");
      const buttons = Array.from(document.querySelectorAll(".question-hardening-btn[data-question-hardening-row]"));
      buttons.find((button) => button.getAttribute("data-question-hardening-row") === "audit_trail")?.click();
      document.querySelector("[data-question-hardening-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-question-card-workflow-hardening-status") || "",
        copied: root?.getAttribute("data-question-card-workflow-hardening-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-question-hardening-artifact]").length,
        progress: document.querySelector("[data-question-hardening-progress]")?.textContent || "",
        summary: document.querySelector("[data-question-hardening-summary]")?.textContent || "",
        title: document.querySelector("[data-question-hardening-active-title]")?.textContent || "",
        link: document.querySelector("[data-question-hardening-active-link]")?.textContent || "",
        preview: document.querySelector("[data-question-hardening-preview]")?.textContent || "",
        globalSchema: window.__QUESTION_CARD_WORKFLOW_HARDENING__?.schema || "",
        globalText: window.__QUESTION_CARD_WORKFLOW_HARDENING_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "question_card_workflow_hardening_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("37/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("Release execution") && domResult.summary.includes("LTS certification") && domResult.summary.includes("auto apply"), "summary missing boundary");
    assert(domResult.title === "Audit trail", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("question-card://workflow-hardening/audit-trail"), "audit trail URI mismatch");
    assert(domResult.preview.includes("release_execution:false") && domResult.preview.includes("lts_certification:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.geo.question_card_workflow_hardening.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t37/90"), "global text missing roadmap matrix");
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
