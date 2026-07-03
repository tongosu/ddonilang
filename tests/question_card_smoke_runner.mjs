#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "question_card_smoke: ok";

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
    "question_card_smoke.js",
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
    await page.waitForSelector("[data-question-card-smoke][data-question-card-smoke-status='question_card_smoke_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_QUESTION_CARD_SMOKE_ROWS,
        buildQuestionCardSmoke,
        formatQuestionCardSmokeText,
      } = await import("./question_card_smoke.js");
      const smoke = buildQuestionCardSmoke({ rows: DEFAULT_QUESTION_CARD_SMOKE_ROWS });
      return { smoke, text: formatQuestionCardSmokeText(smoke) };
    });
    const smoke = moduleResult.smoke;
    assert(smoke.schema === "ddn.geo.question_card_smoke.v1", "schema mismatch");
    assert(smoke.work_item === "GEO1_QUESTION_CARD_SMOKE_V1", "work item mismatch");
    assert(smoke.primary_coordinate === "거-1", "coordinate mismatch");
    assert(smoke.depends_on_coordinate.join(",") === "거-0,타-1,자-0", "dependency mismatch");
    assert(smoke.pack === "question_card_smoke_v1", "pack mismatch");
    assert(smoke.status === "question_card_smoke_ready", "status mismatch");
    assert(smoke.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(smoke.question_card_smoke_claim === true, "question card smoke claim mismatch");
    assert(smoke.proposal_draft_claim === true, "proposal draft mismatch");
    assert(smoke.patch_preview_claim === true, "patch preview mismatch");
    assert(smoke.fix_it_hint_claim === true, "fix-it hint mismatch");
    assert(smoke.review_packet_claim === true, "review packet mismatch");
    assert(smoke.boundary_receipt_claim === true, "boundary receipt mismatch");
    assert(smoke.ai_call_claim === false, "AI call must stay false");
    assert(smoke.parser_preprocessor_claim === false, "parser/preprocessor must stay false");
    assert(smoke.auto_apply_claim === false, "auto apply must stay false");
    assert(smoke.file_write_claim === false, "file write must stay false");
    assert(smoke.runtime_ast_persisted === false, "runtime AST persistence must stay false");
    assert(smoke.state_hash_owner === false, "state hash owner must stay false");
    assert(smoke.replay_ai_recall_claim === false, "replay AI recall must stay false");
    assert(smoke.model_training_claim === false, "model training must stay false");
    assert(smoke.account_permission_change_claim === false, "account permission must stay false");
    assert(smoke.runtime_claim === false, "runtime claim must stay false");
    assert(smoke.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(smoke.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(smoke.progress.roadmap_v2_matrix_behavior_closed === 33, "roadmap closed mismatch");
    assert(smoke.progress.roadmap_v2_matrix_behavior_percent === 37, "roadmap percent mismatch");
    assert(smoke.progress.roadmap_v2_pack_evidence_reference_closed === 54, "pack ref mismatch");
    assert(smoke.progress.roadmap_v2_pack_evidence_reference_percent === 60, "pack ref percent mismatch");
    assert(smoke.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(smoke.rows.map((row) => row.id).join(",") === "proposal_draft,patch_preview,fix_it_hint,review_packet,boundary_receipt", "row order mismatch");
    assert(String(smoke.smoke_text).includes("ai_call:false"), "smoke text missing AI boundary");
    assert(String(smoke.smoke_text).includes("auto_apply:false"), "smoke text missing auto-apply boundary");
    assert(String(smoke.smoke_text).includes("state_hash_owner:false"), "smoke text missing state hash boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t54/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-question-card-smoke]");
      const buttons = Array.from(document.querySelectorAll(".question-card-btn[data-question-card-row]"));
      buttons.find((button) => button.getAttribute("data-question-card-row") === "fix_it_hint")?.click();
      document.querySelector("[data-question-card-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-question-card-smoke-status") || "",
        copied: root?.getAttribute("data-question-card-smoke-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-question-card-artifact]").length,
        progress: document.querySelector("[data-question-card-progress]")?.textContent || "",
        summary: document.querySelector("[data-question-card-summary]")?.textContent || "",
        title: document.querySelector("[data-question-card-active-title]")?.textContent || "",
        link: document.querySelector("[data-question-card-active-link]")?.textContent || "",
        preview: document.querySelector("[data-question-card-preview]")?.textContent || "",
        globalSchema: window.__QUESTION_CARD_SMOKE__?.schema || "",
        globalText: window.__QUESTION_CARD_SMOKE_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "question_card_smoke_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("33/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("AI call") && domResult.summary.includes("auto-apply") && domResult.summary.includes("file write"), "summary missing boundary");
    assert(domResult.title === "Fix-it hint", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("question-card://smoke/fix-it-hint"), "fix-it URI mismatch");
    assert(domResult.preview.includes("ai_call:false") && domResult.preview.includes("state_hash_owner:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.geo.question_card_smoke.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t33/90"), "global text missing roadmap matrix");
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
