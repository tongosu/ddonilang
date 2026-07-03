#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "question_card_validation: ok";

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
    "question_card_validation.js",
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
    await page.waitForSelector("[data-question-card-validation][data-question-card-validation-status='question_card_validation_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_QUESTION_CARD_VALIDATION_ROWS,
        buildQuestionCardValidation,
        formatQuestionCardValidationText,
      } = await import("./question_card_validation.js");
      const validation = buildQuestionCardValidation({ rows: DEFAULT_QUESTION_CARD_VALIDATION_ROWS });
      return { validation, text: formatQuestionCardValidationText(validation) };
    });
    const validation = moduleResult.validation;
    assert(validation.schema === "ddn.geo.question_card_validation.v1", "schema mismatch");
    assert(validation.work_item === "GEO2_AI_OUTPUT_VALIDATION_PACK_V1", "work item mismatch");
    assert(validation.primary_coordinate === "거-2", "coordinate mismatch");
    assert(validation.depends_on_coordinate.join(",") === "거-1,타-2,자-1/자-2", "dependency mismatch");
    assert(validation.pack === "question_card_validation_v1", "pack mismatch");
    assert(validation.status === "question_card_validation_ready", "status mismatch");
    assert(validation.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(validation.ai_output_validation_claim === true, "AI output validation claim mismatch");
    assert(validation.ddn_lint_claim === true, "DDN lint mismatch");
    assert(validation.lesson_lint_claim === true, "lesson lint mismatch");
    assert(validation.intent_alignment_claim === true, "intent alignment mismatch");
    assert(validation.patch_safety_claim === true, "patch safety mismatch");
    assert(validation.validation_receipt_claim === true, "validation receipt mismatch");
    assert(validation.ai_call_claim === false, "AI call must stay false");
    assert(validation.parser_preprocessor_claim === false, "parser/preprocessor must stay false");
    assert(validation.auto_apply_claim === false, "auto apply must stay false");
    assert(validation.patch_execution_claim === false, "patch execution must stay false");
    assert(validation.file_write_claim === false, "file write must stay false");
    assert(validation.network_call_claim === false, "network call must stay false");
    assert(validation.runtime_ast_persisted === false, "runtime AST persistence must stay false");
    assert(validation.state_hash_owner === false, "state hash owner must stay false");
    assert(validation.model_training_claim === false, "model training must stay false");
    assert(validation.account_permission_change_claim === false, "account permission must stay false");
    assert(validation.runtime_claim === false, "runtime claim must stay false");
    assert(validation.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(validation.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(validation.progress.roadmap_v2_matrix_behavior_closed === 34, "roadmap closed mismatch");
    assert(validation.progress.roadmap_v2_matrix_behavior_percent === 38, "roadmap percent mismatch");
    assert(validation.progress.roadmap_v2_pack_evidence_reference_closed === 55, "pack ref mismatch");
    assert(validation.progress.roadmap_v2_pack_evidence_reference_percent === 61, "pack ref percent mismatch");
    assert(validation.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(validation.rows.map((row) => row.id).join(",") === "ddn_lint,lesson_lint,intent_alignment,patch_safety,validation_receipt", "row order mismatch");
    assert(String(validation.validation_text).includes("patch_execution:false"), "validation text missing patch execution boundary");
    assert(String(validation.validation_text).includes("network_call:false"), "validation text missing network boundary");
    assert(String(validation.validation_text).includes("state_hash_owner:false"), "validation text missing state hash boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t55/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-question-card-validation]");
      const buttons = Array.from(document.querySelectorAll(".question-validation-btn[data-question-validation-row]"));
      buttons.find((button) => button.getAttribute("data-question-validation-row") === "patch_safety")?.click();
      document.querySelector("[data-question-validation-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-question-card-validation-status") || "",
        copied: root?.getAttribute("data-question-card-validation-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-question-validation-artifact]").length,
        progress: document.querySelector("[data-question-validation-progress]")?.textContent || "",
        summary: document.querySelector("[data-question-validation-summary]")?.textContent || "",
        title: document.querySelector("[data-question-validation-active-title]")?.textContent || "",
        link: document.querySelector("[data-question-validation-active-link]")?.textContent || "",
        preview: document.querySelector("[data-question-validation-preview]")?.textContent || "",
        globalSchema: window.__QUESTION_CARD_VALIDATION__?.schema || "",
        globalText: window.__QUESTION_CARD_VALIDATION_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "question_card_validation_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("34/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("AI call") && domResult.summary.includes("patch execution") && domResult.summary.includes("network call"), "summary missing boundary");
    assert(domResult.title === "Patch safety", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("question-card://validation/patch-safety"), "patch safety URI mismatch");
    assert(domResult.preview.includes("patch_execution:false") && domResult.preview.includes("network_call:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.geo.question_card_validation.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t34/90"), "global text missing roadmap matrix");
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
