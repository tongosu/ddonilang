#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "question_card_dev_assist: ok";

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
    "question_card_dev_assist.js",
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
    await page.waitForSelector("[data-question-card-dev-assist][data-question-card-dev-assist-status='question_card_dev_assist_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_QUESTION_CARD_DEV_ASSIST_ROWS,
        buildQuestionCardDevAssist,
        formatQuestionCardDevAssistText,
      } = await import("./question_card_dev_assist.js");
      const assist = buildQuestionCardDevAssist({ rows: DEFAULT_QUESTION_CARD_DEV_ASSIST_ROWS });
      return { assist, text: formatQuestionCardDevAssistText(assist) };
    });
    const assist = moduleResult.assist;
    assert(assist.schema === "ddn.geo.question_card_dev_assist.v1", "schema mismatch");
    assert(assist.work_item === "GEO3_DEV_ASSIST_UI_V1", "work item mismatch");
    assert(assist.primary_coordinate === "거-3", "coordinate mismatch");
    assert(assist.depends_on_coordinate.join(",") === "거-2,거-1,타-2", "dependency mismatch");
    assert(assist.pack === "question_card_dev_assist_v1", "pack mismatch");
    assert(assist.status === "question_card_dev_assist_ready", "status mismatch");
    assert(assist.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(assist.dev_assist_ui_claim === true, "dev assist UI claim mismatch");
    assert(assist.codex_work_item_claim === true, "Codex work item mismatch");
    assert(assist.lesson_draft_claim === true, "lesson draft mismatch");
    assert(assist.report_draft_claim === true, "report draft mismatch");
    assert(assist.review_queue_claim === true, "review queue mismatch");
    assert(assist.handoff_receipt_claim === true, "handoff receipt mismatch");
    assert(assist.ai_call_claim === false, "AI call must stay false");
    assert(assist.parser_preprocessor_claim === false, "parser/preprocessor must stay false");
    assert(assist.auto_apply_claim === false, "auto apply must stay false");
    assert(assist.patch_execution_claim === false, "patch execution must stay false");
    assert(assist.file_write_claim === false, "file write must stay false");
    assert(assist.network_call_claim === false, "network call must stay false");
    assert(assist.runtime_ast_persisted === false, "runtime AST persistence must stay false");
    assert(assist.state_hash_owner === false, "state hash owner must stay false");
    assert(assist.registry_publish_claim === false, "registry publish must stay false");
    assert(assist.account_permission_change_claim === false, "account permission must stay false");
    assert(assist.runtime_claim === false, "runtime claim must stay false");
    assert(assist.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(assist.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(assist.progress.roadmap_v2_matrix_behavior_closed === 35, "roadmap closed mismatch");
    assert(assist.progress.roadmap_v2_matrix_behavior_percent === 39, "roadmap percent mismatch");
    assert(assist.progress.roadmap_v2_pack_evidence_reference_closed === 56, "pack ref mismatch");
    assert(assist.progress.roadmap_v2_pack_evidence_reference_percent === 62, "pack ref percent mismatch");
    assert(assist.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(assist.rows.map((row) => row.id).join(",") === "codex_work_item,lesson_draft,report_draft,review_queue,handoff_receipt", "row order mismatch");
    assert(String(assist.assist_text).includes("file_write:false"), "assist text missing file-write boundary");
    assert(String(assist.assist_text).includes("registry_publish:false"), "assist text missing registry boundary");
    assert(String(assist.assist_text).includes("state_hash_owner:false"), "assist text missing state hash boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t56/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-question-card-dev-assist]");
      const buttons = Array.from(document.querySelectorAll(".question-assist-btn[data-question-assist-row]"));
      buttons.find((button) => button.getAttribute("data-question-assist-row") === "review_queue")?.click();
      document.querySelector("[data-question-assist-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-question-card-dev-assist-status") || "",
        copied: root?.getAttribute("data-question-card-dev-assist-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-question-assist-artifact]").length,
        progress: document.querySelector("[data-question-assist-progress]")?.textContent || "",
        summary: document.querySelector("[data-question-assist-summary]")?.textContent || "",
        title: document.querySelector("[data-question-assist-active-title]")?.textContent || "",
        link: document.querySelector("[data-question-assist-active-link]")?.textContent || "",
        preview: document.querySelector("[data-question-assist-preview]")?.textContent || "",
        globalSchema: window.__QUESTION_CARD_DEV_ASSIST__?.schema || "",
        globalText: window.__QUESTION_CARD_DEV_ASSIST_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "question_card_dev_assist_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("35/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("AI call") && domResult.summary.includes("file write") && domResult.summary.includes("registry publish"), "summary missing boundary");
    assert(domResult.title === "Review queue", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("question-card://dev-assist/review-queue"), "review queue URI mismatch");
    assert(domResult.preview.includes("file_write:false") && domResult.preview.includes("registry_publish:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.geo.question_card_dev_assist.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t35/90"), "global text missing roadmap matrix");
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
