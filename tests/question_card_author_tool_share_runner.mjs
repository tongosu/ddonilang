#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "question_card_author_tool_share: ok";

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
    "question_card_author_tool_share.js",
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
    await page.waitForSelector("[data-question-card-author-tool-share][data-question-card-author-tool-share-status='question_card_author_tool_share_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_QUESTION_CARD_AUTHOR_TOOL_SHARE_ROWS,
        buildQuestionCardAuthorToolShare,
        formatQuestionCardAuthorToolShareText,
      } = await import("./question_card_author_tool_share.js");
      const share = buildQuestionCardAuthorToolShare({ rows: DEFAULT_QUESTION_CARD_AUTHOR_TOOL_SHARE_ROWS });
      return { share, text: formatQuestionCardAuthorToolShareText(share) };
    });
    const share = moduleResult.share;
    assert(share.schema === "ddn.geo.question_card_author_tool_share.v1", "schema mismatch");
    assert(share.work_item === "GEO4_AUTHOR_TOOL_SHARE_V1", "work item mismatch");
    assert(share.primary_coordinate === "거-4", "coordinate mismatch");
    assert(share.depends_on_coordinate.join(",") === "거-3,거-2,거-1,타-2", "dependency mismatch");
    assert(share.pack === "question_card_author_tool_share_v1", "pack mismatch");
    assert(share.status === "question_card_author_tool_share_ready", "status mismatch");
    assert(share.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(share.author_tool_share_claim === true, "author share claim mismatch");
    assert(share.template_registry_claim === true, "template registry mismatch");
    assert(share.tool_manifest_claim === true, "tool manifest mismatch");
    assert(share.lesson_template_claim === true, "lesson template mismatch");
    assert(share.review_template_claim === true, "review template mismatch");
    assert(share.handoff_bundle_claim === true, "handoff bundle mismatch");
    assert(share.ai_call_claim === false, "AI call must stay false");
    assert(share.parser_preprocessor_claim === false, "parser/preprocessor must stay false");
    assert(share.auto_apply_claim === false, "auto apply must stay false");
    assert(share.patch_execution_claim === false, "patch execution must stay false");
    assert(share.file_write_claim === false, "file write must stay false");
    assert(share.network_call_claim === false, "network call must stay false");
    assert(share.runtime_ast_persisted === false, "runtime AST persistence must stay false");
    assert(share.state_hash_owner === false, "state hash owner must stay false");
    assert(share.registry_publish_claim === false, "registry publish must stay false");
    assert(share.account_permission_change_claim === false, "account permission must stay false");
    assert(share.cloud_sync_claim === false, "cloud sync must stay false");
    assert(share.runtime_claim === false, "runtime claim must stay false");
    assert(share.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(share.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(share.progress.roadmap_v2_matrix_behavior_closed === 36, "roadmap closed mismatch");
    assert(share.progress.roadmap_v2_matrix_behavior_percent === 40, "roadmap percent mismatch");
    assert(share.progress.roadmap_v2_pack_evidence_reference_closed === 57, "pack ref mismatch");
    assert(share.progress.roadmap_v2_pack_evidence_reference_percent === 63, "pack ref percent mismatch");
    assert(share.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(share.rows.map((row) => row.id).join(",") === "template_registry,tool_manifest,lesson_template,review_template,handoff_bundle", "row order mismatch");
    assert(String(share.share_text).includes("registry_publish:false"), "share text missing registry boundary");
    assert(String(share.share_text).includes("account_permission_change:false"), "share text missing account boundary");
    assert(String(share.share_text).includes("cloud_sync:false"), "share text missing cloud boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t57/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-question-card-author-tool-share]");
      const buttons = Array.from(document.querySelectorAll(".question-share-btn[data-question-share-row]"));
      buttons.find((button) => button.getAttribute("data-question-share-row") === "review_template")?.click();
      document.querySelector("[data-question-share-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-question-card-author-tool-share-status") || "",
        copied: root?.getAttribute("data-question-card-author-tool-share-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-question-share-artifact]").length,
        progress: document.querySelector("[data-question-share-progress]")?.textContent || "",
        summary: document.querySelector("[data-question-share-summary]")?.textContent || "",
        title: document.querySelector("[data-question-share-active-title]")?.textContent || "",
        link: document.querySelector("[data-question-share-active-link]")?.textContent || "",
        preview: document.querySelector("[data-question-share-preview]")?.textContent || "",
        globalSchema: window.__QUESTION_CARD_AUTHOR_TOOL_SHARE__?.schema || "",
        globalText: window.__QUESTION_CARD_AUTHOR_TOOL_SHARE_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "question_card_author_tool_share_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("36/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("Registry publish") && domResult.summary.includes("account permission change") && domResult.summary.includes("cloud sync"), "summary missing boundary");
    assert(domResult.title === "Review template", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("question-card://author-share/review-template"), "review template URI mismatch");
    assert(domResult.preview.includes("registry_publish:false") && domResult.preview.includes("cloud_sync:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.geo.question_card_author_tool_share.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t36/90"), "global text missing roadmap matrix");
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
