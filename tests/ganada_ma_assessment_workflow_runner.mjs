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

const OK = "ganada_ma_assessment_workflow: ok";

async function waitVisible(page, selector) {
  await page.waitForSelector(selector, { state: "visible", timeout: 15000 });
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "education_assessment_pack.js",
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-browse");
    const lessonSelector = ".lesson-card[data-lesson-id='rep_math_function_line_v1']";
    await page.waitForSelector(`${lessonSelector} .card-launch-btn[data-launch-profile='student']`, { state: "attached" });
    await page.click(`${lessonSelector} .card-launch-btn[data-launch-profile='student']`);
    await waitVisible(page, "#screen-run");
    await page.waitForSelector("[data-testid='lesson-run-result'][data-lesson-run-result]", { state: "attached" });
    await page.waitForFunction(() => {
      const hash = document.querySelector("#run-mirror-hash")?.textContent ?? "";
      const madi = document.querySelector("#run-madi-status")?.textContent ?? "";
      return hash.includes("blake3:") || /\d+\s*\/\s*\d+/.test(madi);
    });
    await page.click("#run-tab-btn-output");
    await waitVisible(page, "[data-testid='lesson-run-result']");
    const lessonRun = await page.evaluate(() => ({
      resultText: document.querySelector("[data-testid='lesson-run-result']")?.textContent?.trim() ?? "",
      errorPanelHidden: document.querySelector("[data-testid='lesson-error-panel']")?.classList.contains("hidden"),
      hashText: document.querySelector("#run-mirror-hash")?.textContent?.trim() ?? "",
    }));
    assert(lessonRun.errorPanelHidden === true, "lesson run should not show error panel");
    assert(lessonRun.resultText.length > 0, "lesson run result panel should not be empty");
    assert(lessonRun.hashText.includes("blake3:"), `lesson state hash missing: ${lessonRun.hashText}`);

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-testid='assessment-panel'][data-education-assessment-pack-status='education_assessment_pack_ready']");
    await page.click("[data-education-assessment-row='sample_submission']");
    await page.click("[data-testid='assessment-run']");
    const assessment = await page.evaluate(() => ({
      rootStatus: document.querySelector("[data-testid='assessment-panel']")?.getAttribute("data-education-assessment-pack-status") ?? "",
      ran: document.querySelector("[data-testid='assessment-panel']")?.getAttribute("data-assessment-workflow-ran") ?? "",
      source: document.querySelector("[data-testid='assessment-panel']")?.getAttribute("data-assessment-lesson-run-source") ?? "",
      result: document.querySelector("[data-testid='assessment-result']")?.textContent ?? "",
      feedback: document.querySelector("[data-testid='assessment-feedback']")?.textContent ?? "",
      activeTitle: document.querySelector("[data-education-assessment-active-title]")?.textContent ?? "",
    }));
    assert(assessment.rootStatus === "education_assessment_pack_ready", `assessment status mismatch: ${assessment.rootStatus}`);
    assert(assessment.ran === "true", "assessment workflow run state missing");
    assert(assessment.source === "seamgrim_lesson_run_preset_rail_v1", `lesson run source mismatch: ${assessment.source}`);
    assert(assessment.activeTitle === "Sample submission", `assessment active row mismatch: ${assessment.activeTitle}`);
    assert(assessment.result.includes("lesson_run_result_source:seamgrim_lesson_run_preset_rail_v1"), "assessment result missing lesson run source");
    assert(assessment.result.includes("gradebook_write:false"), "assessment result missing gradebook boundary");
    assert(assessment.feedback.includes("gradebook write") && assessment.feedback.includes("remote classroom sync"), "assessment feedback missing boundary");
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
