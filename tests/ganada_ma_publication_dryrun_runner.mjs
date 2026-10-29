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

const OK = "ganada_ma_publication_dryrun: ok";

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "education_publication_pack.js",
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
    await page.waitForSelector("[data-testid='publication-review-panel'][data-education-publication-pack-status='education_publication_pack_ready']");

    await page.click("[data-education-publication-row='publication_bundle']");
    const title = await page.textContent("[data-education-publication-active-title]");
    assert(String(title).trim() === "Publication bundle", `publication row mismatch: ${title}`);
    await page.click("[data-testid='publication-dryrun']");

    const dryRun = await page.evaluate(() => {
      const root = document.querySelector("[data-testid='publication-review-panel']");
      return {
        ran: root?.getAttribute("data-publication-dry-run") ?? "",
        policy: root?.getAttribute("data-publication-dry-run-policy") ?? "",
        preview: document.querySelector("[data-testid='publication-artifact-preview']")?.textContent ?? "",
        warning: document.querySelector("[data-testid='publication-boundary-warning']")?.textContent ?? "",
        artifactCount: document.querySelectorAll("[data-education-publication-artifact]").length,
        globalSchema: window.__EDUCATION_PUBLICATION_PACK__?.schema ?? "",
      };
    });
    assert(dryRun.ran === "true", "publication dry-run state missing");
    assert(dryRun.policy === "local_only", `publication dry-run policy mismatch: ${dryRun.policy}`);
    assert(dryRun.preview.includes("publication_dry_run:local_only"), "dry-run line missing");
    assert(dryRun.preview.includes("public_upload:false"), "public upload boundary missing");
    assert(dryRun.preview.includes("registry_publish:false"), "registry publish boundary missing");
    assert(dryRun.warning.includes("public upload") && dryRun.warning.includes("release execution"), "publication boundary warning missing");
    assert(dryRun.artifactCount === 5, `publication artifact count mismatch: ${dryRun.artifactCount}`);
    assert(dryRun.globalSchema === "ddn.education.publication_pack.v1", "global publication schema mismatch");
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
