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

const OK = "ganada_ja_seulgi_local_workflow: ok";

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
    await page.waitForSelector("[data-testid='seulgi-replay-log'][data-seulgi-replay-log]");

    await page.click("[data-testid='seulgi-fixture-import']");
    await page.click("[data-seulgi-proposal-row='preview']");
    const preview = await page.textContent("[data-testid='seulgi-preview']");
    assert(String(preview).includes("receipt_policy:approve_reject_local"), "receipt policy missing in preview");
    assert(String(preview).includes("auto_apply:false"), "auto-apply boundary missing in preview");

    await page.click("[data-testid='seulgi-approve']");
    const approve = await page.evaluate(() => ({
      receipt: document.querySelector("[data-testid='seulgi-proposal-panel']")?.getAttribute("data-seulgi-approval-receipt") ?? "",
      title: document.querySelector("[data-seulgi-proposal-active-title]")?.textContent ?? "",
      link: document.querySelector("[data-seulgi-proposal-active-link]")?.textContent ?? "",
    }));
    assert(approve.receipt === "local", "approval receipt state missing");
    assert(approve.title === "승인", `approval title mismatch: ${approve.title}`);
    assert(approve.link.includes("seulgi://proposal/approval-receipt"), "approval receipt link missing");

    await page.click("[data-testid='seulgi-reject']");
    const reject = await page.evaluate(() => ({
      receipt: document.querySelector("[data-testid='seulgi-proposal-panel']")?.getAttribute("data-seulgi-rejection-receipt") ?? "",
      title: document.querySelector("[data-seulgi-proposal-active-title]")?.textContent ?? "",
      link: document.querySelector("[data-seulgi-proposal-active-link]")?.textContent ?? "",
      approvalReceipt: document.querySelector("[data-testid='seulgi-proposal-panel']")?.getAttribute("data-seulgi-approval-receipt") ?? "",
    }));
    assert(reject.receipt === "local", "rejection receipt state missing");
    assert(reject.approvalReceipt === "local", "approval receipt should remain recorded");
    assert(reject.title === "거절", `rejection title mismatch: ${reject.title}`);
    assert(reject.link.includes("seulgi://proposal/rejection-receipt"), "rejection receipt link missing");

    await page.click("[data-seulgi-replay-row='approval']");
    const replay = await page.evaluate(() => ({
      title: document.querySelector("[data-seulgi-replay-active-title]")?.textContent ?? "",
      log: document.querySelector("[data-testid='seulgi-replay-log'][data-seulgi-replay-log]")?.textContent ?? "",
      replayContext: document.querySelector("[data-testid='seulgi-replay-panel']")?.getAttribute("data-seulgi-replay-context") ?? "",
    }));
    assert(replay.title === "승인 영수증", `replay receipt title mismatch: ${replay.title}`);
    assert(replay.replayContext === "lesson_editor_fixture", `replay context mismatch: ${replay.replayContext}`);
    assert(replay.log.includes("proposal_receipt_source:seulgi_proposal_ui_v1"), "proposal receipt provenance missing");
    assert(replay.log.includes("approval\tapproval_receipt\tdecision_fixed\tready"), "approval receipt replay row missing");
    assert(replay.log.includes("ai_recall:false") && replay.log.includes("file_write:false"), "replay boundary missing");
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
