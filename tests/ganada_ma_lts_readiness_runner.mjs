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

const OK = "ganada_ma_lts_readiness: ok";

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "education_operations_lts.js",
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
    await page.waitForSelector("[data-testid='lts-readiness-panel'][data-education-operations-lts-status='education_operations_lts_ready']");

    await page.click("[data-education-operations-row='lts_gate']");
    const title = await page.textContent("[data-education-operations-active-title]");
    assert(String(title).trim() === "LTS gate", `LTS gate row mismatch: ${title}`);
    await page.click("[data-testid='lts-regression-gate']");

    const lts = await page.evaluate(() => {
      const root = document.querySelector("[data-testid='lts-readiness-panel']");
      return {
        gate: root?.getAttribute("data-lts-regression-gate") ?? "",
        matrixPolicy: root?.getAttribute("data-lts-runner-matrix-policy") ?? "",
        report: document.querySelector("[data-testid='lts-operations-report']")?.textContent ?? "",
        warning: document.querySelector("[data-testid='lts-boundary-warning']")?.textContent ?? "",
        globalSchema: window.__EDUCATION_OPERATIONS_LTS__?.schema ?? "",
      };
    });
    assert(lts.gate === "true", "LTS regression gate state missing");
    assert(lts.matrixPolicy === "local_regression_gate", `LTS matrix policy mismatch: ${lts.matrixPolicy}`);
    assert(lts.report.includes("lts_runner_matrix:local"), "LTS runner matrix line missing");
    assert(lts.report.includes("lts_certification_claim:false"), "LTS certification boundary missing");
    assert(lts.report.includes("remote_lts_certification:false"), "remote LTS boundary missing");
    assert(lts.report.includes("release_execution:false"), "release execution boundary missing");
    assert(lts.warning.includes("remote LTS certification") && lts.warning.includes("gradebook write"), "LTS boundary warning missing");
    assert(lts.globalSchema === "ddn.education.operations_lts.v1", "global LTS schema mismatch");
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
