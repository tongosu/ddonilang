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

const OK = "ganada_ma_classroom_workflow: ok";

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "education_classroom_ui_pack.js",
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
    await page.waitForSelector("[data-testid='classroom-workflow-panel'][data-education-classroom-ui-pack-status='education_classroom_ui_pack_ready']");

    await page.click("[data-testid='classroom-teacher-mode']");
    const teacher = await page.textContent("[data-education-classroom-active-title]");
    assert(String(teacher).trim() === "Teacher mode", `teacher mode mismatch: ${teacher}`);

    await page.click("[data-testid='classroom-student-mode']");
    const student = await page.textContent("[data-education-classroom-active-title]");
    assert(String(student).trim() === "Student mode", `student mode mismatch: ${student}`);

    await page.click("[data-testid='classroom-assignment-panel']");
    const rosterText = await page.textContent("[data-testid='classroom-roster']");
    assert(String(rosterText).includes("roster_result_review:local_fixture"), "roster/result fixture line missing");
    assert(String(rosterText).includes("student_personal_data_collection:false"), "student data boundary missing");

    await page.click("[data-testid='classroom-report-review']");
    const reviewTitle = await page.textContent("[data-testid='classroom-result-review']");
    assert(String(reviewTitle).trim() === "Report review", `report review mismatch: ${reviewTitle}`);

    await page.click("[data-testid='classroom-report-export']");
    const workflow = await page.evaluate(() => {
      const root = document.querySelector("[data-testid='classroom-workflow-panel']");
      return {
        exported: root?.getAttribute("data-classroom-report-exported") ?? "",
        policy: root?.getAttribute("data-classroom-report-export-policy") ?? "",
        preview: document.querySelector("[data-testid='classroom-roster']")?.textContent ?? "",
        globalSchema: window.__EDUCATION_CLASSROOM_UI_PACK__?.schema ?? "",
      };
    });
    assert(workflow.exported === "true", "classroom report export state missing");
    assert(workflow.policy === "local_dry_run", `report export policy mismatch: ${workflow.policy}`);
    assert(workflow.preview.includes("report_export:local_dry_run"), "report export dry-run line missing");
    assert(workflow.preview.includes("gradebook_write:false"), "gradebook boundary missing");
    assert(workflow.globalSchema === "ddn.education.classroom_ui_pack.v1", "global classroom schema mismatch");
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
