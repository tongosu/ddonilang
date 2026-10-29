#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import {
  assert,
  closeServer,
  collectPageFailures,
  createServer,
  requireFile,
} from "./ganada_browser_runner_lib.mjs";

const OK = "ganada_ja_seulgi_context: ok";

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "seulgi_proposal_ui.js",
    "seulgi_replay_safe_workflow.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 840 }, locale: "ko-KR" });
    const page = await context.newPage();
    const failures = collectPageFailures(page);
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-testid='seulgi-proposal-panel'][data-seulgi-proposal-ui-status='seulgi_proposal_ui_ready']");
    await page.waitForSelector("[data-testid='seulgi-replay-panel'][data-seulgi-replay-safe-status='seulgi_replay_safe_workflow_ready']");

    await page.click("[data-testid='seulgi-fixture-import']");
    await page.click("[data-seulgi-proposal-row='preview']");
    const proposal = await page.evaluate(() => {
      const root = document.querySelector("[data-testid='seulgi-proposal-panel']");
      return {
        imported: root?.getAttribute("data-seulgi-fixture-imported") ?? "",
        lessonContext: root?.getAttribute("data-seulgi-lesson-context") ?? "",
        editorContext: root?.getAttribute("data-seulgi-editor-context") ?? "",
        preview: document.querySelector("[data-testid='seulgi-preview']")?.textContent ?? "",
        activeTitle: document.querySelector("[data-seulgi-proposal-active-title]")?.textContent ?? "",
      };
    });
    assert(proposal.imported === "true", "fixture proposal import state missing");
    assert(proposal.lessonContext === "seamgrim_lesson_run_rail_v1", `lesson context mismatch: ${proposal.lessonContext}`);
    assert(proposal.editorContext === "seamgrim_lesson_authoring_flow_v1", `editor context mismatch: ${proposal.editorContext}`);
    assert(proposal.activeTitle === "미리보기", `preview row mismatch: ${proposal.activeTitle}`);
    assert(proposal.preview.includes("fixture_proposal_import:true"), "fixture import line missing");
    assert(proposal.preview.includes("lesson_context:seamgrim_lesson_run_rail_v1"), "lesson context line missing");
    assert(proposal.preview.includes("editor_context:seamgrim_lesson_authoring_flow_v1"), "editor context line missing");
    assert(proposal.preview.includes("ai_call:false") && proposal.preview.includes("file_write:false"), "AI/file boundary missing");

    const replay = await page.evaluate(() => {
      const root = document.querySelector("[data-testid='seulgi-replay-panel']");
      return {
        replayContext: root?.getAttribute("data-seulgi-replay-context") ?? "",
        policy: root?.getAttribute("data-seulgi-receipt-replay-policy") ?? "",
        log: document.querySelector("[data-testid='seulgi-replay-log'][data-seulgi-replay-log]")?.textContent ?? "",
      };
    });
    assert(replay.replayContext === "lesson_editor_fixture", `replay context mismatch: ${replay.replayContext}`);
    assert(replay.policy === "local_append_only", `replay policy mismatch: ${replay.policy}`);
    assert(replay.log.includes("proposal_receipt_source:seulgi_proposal_ui_v1"), "proposal receipt source missing");
    assert(replay.log.includes("receipt_replay_policy:local_append_only"), "receipt replay policy missing");
    assert(replay.log.includes("ai_recall:false"), "AI recall boundary missing");
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
