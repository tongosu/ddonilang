#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_lesson_authoring_run_integration: ok";

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
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|md)$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "studio_lesson_authoring_run_integration.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1180, height: 760 }, locale: "ko-KR" });
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
      const {
        buildLessonAuthoringRunIntegration,
        formatLessonAuthoringRunIntegrationText,
      } = await import("./studio_lesson_authoring_run_integration.js");
      const draft = {
        draft_id: "draft-authoring-001",
        lesson_id: "local_authoring_math_v1",
        title: "작성 실행 통합",
        source_kind: "direct_ddn",
        dirty: true,
        ddn_text: [
          '설정 { 제목: "작성 실행 통합". 설명: "authoring run integration". }.',
          "채비 { 프레임수:수 <- 0. }.",
          '"실행 통합" 보여주기.',
        ].join("\n"),
      };
      const runRequest = {
        launch_kind: "editor_run",
        source_type: "ddn",
        created_at_ms: 1770000000000,
        auto_execute: false,
      };
      const saveState = {
        local_save_available: true,
        local_save_status: "로컬 저장됨",
        filename: "lesson.ddn",
        remote_save_claim: false,
      };
      const loaderContract = {
        lesson_loader_reused: true,
        lesson_schema_change: false,
        active_allowlist_mutation: false,
      };
      const runPresetContext = {
        preset_schema: "seamgrim.run_preset_rail.v1",
        onboarding_profile: "teacher",
        layout_mode: "split",
        required_view_count: 2,
      };
      const workflow = buildLessonAuthoringRunIntegration({
        draft,
        runRequest,
        saveState,
        loaderContract,
        runPresetContext,
      });
      return {
        workflow,
        text: formatLessonAuthoringRunIntegrationText(workflow),
      };
    });

    const workflow = result.workflow;
    assert(workflow.schema === "seamgrim.lesson_authoring_run_integration.v1", "workflow schema mismatch");
    assert(workflow.primary_coordinate === "마-3", `primary coordinate mismatch: ${workflow.primary_coordinate}`);
    assert(workflow.support_coordinate === "라-3", `support coordinate mismatch: ${workflow.support_coordinate}`);
    assert(workflow.workflow_claim === "lesson_authoring_run_integration", `workflow claim mismatch: ${workflow.workflow_claim}`);
    assert(workflow.generated_locally === true, "workflow must be local");
    assert(workflow.lesson_schema_change === false, "workflow must not claim lesson schema change");
    assert(workflow.active_allowlist_mutation === false, "workflow must not claim allowlist mutation");
    assert(workflow.runtime_claim === false, "workflow must not claim runtime change");
    assert(workflow.remote_save_claim === false, "workflow must not claim remote save");
    assert(workflow.replay_claim === false, "workflow must not claim replay");
    assert(workflow.status === "authoring_run_ready", `status mismatch: ${workflow.status}`);
    assert(workflow.stage_count === 7, `stage count mismatch: ${workflow.stage_count}`);
    assert(workflow.ready_stage_count === 7, `ready stage count mismatch: ${workflow.ready_stage_count}`);
    assert(workflow.missing_stage_count === 0, `missing stage count mismatch: ${workflow.missing_stage_count}`);
    assert(workflow.draft_id === "draft-authoring-001", "draft id mismatch");
    assert(workflow.lesson_id === "local_authoring_math_v1", "lesson id mismatch");
    assert(workflow.ddn_line_count === 3, `ddn line count mismatch: ${workflow.ddn_line_count}`);
    assert(workflow.launch_kind === "editor_run", "launch kind mismatch");
    assert(workflow.source_type === "ddn", "source type mismatch");
    assert(workflow.local_save_filename === "lesson.ddn", "save filename mismatch");
    assert(workflow.onboarding_profile === "teacher", "onboarding profile mismatch");
    assert(workflow.required_view_count === 2, "required view count mismatch");
    assert(String(result.text).includes("schema\tseamgrim.lesson_authoring_run_integration.v1"), "text schema missing");
    assert(String(result.text).includes("support_coordinate\t라-3"), "text support coordinate missing");
    assert(String(result.text).includes("status\tauthoring_run_ready"), "text status missing");
    assert(String(result.text).includes("stage_id\tready"), "text stage header missing");
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
