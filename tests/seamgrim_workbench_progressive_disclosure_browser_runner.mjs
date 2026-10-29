#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  return "application/octet-stream";
}

async function serve(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      const relative = decodeURIComponent(url.pathname === "/"
        ? "/solutions/seamgrim_ui_mvp/ui/workbench/index.html"
        : url.pathname).replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, relative);
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
        res.writeHead(403).end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
      res.end(bytes);
    } catch (_) {
      res.writeHead(404).end("not found");
    }
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert(address && typeof address !== "string", "server address unavailable");
  return { server, baseUrl: `http://127.0.0.1:${address.port}` };
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const { server, baseUrl } = await serve(root);
  let browser = null;
  const pageErrors = [];
  try {
    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage({ viewport: { width: 1360, height: 840 }, locale: "ko-KR" });
    page.on("pageerror", (err) => pageErrors.push(err.message));
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()));

    const firstViewport = await page.evaluate(() => {
      const rect = (testId) => {
        const box = document.querySelector(`[data-testid='${testId}']`)?.getBoundingClientRect();
        return box ? { top: box.top, bottom: box.bottom, width: box.width, height: box.height } : null;
      };
      const dialogs = ["project-storage-drawer", "project-history-drawer", "project-verification-drawer", "portable-project-dialog"];
      return {
        viewportHeight: innerHeight,
        title: rect("project-workbench"),
        run: rect("run-status"),
        controls: rect("control-grid"),
        source: rect("source-panel"),
        primary: rect("bogae-panel"),
        fileTabs: rect("file-tabs"),
        dialogDisplay: Object.fromEntries(dialogs.map((id) => [id, getComputedStyle(document.querySelector(`[data-testid='${id}']`)).display])),
      };
    });
    assert(firstViewport.source.top < firstViewport.viewportHeight * 0.7, `DDN source was still pushed out of the first viewport: ${JSON.stringify(firstViewport)}`);
    assert(firstViewport.primary.top === firstViewport.source.top && firstViewport.primary.width > 0, "primary view is not paired with DDN in the first viewport");
    assert(firstViewport.run.top < firstViewport.source.top && firstViewport.controls.top < firstViewport.source.top, "run line or project controls are not first-viewport content");
    assert(firstViewport.fileTabs.top < firstViewport.source.top, "project files are not visible before the editor");
    assert(Object.values(firstViewport.dialogDisplay).every((display) => display === "none"), `closed management content occupied layout: ${JSON.stringify(firstViewport.dialogDisplay)}`);
    assert(!(await page.locator("[data-testid='gaji-shelf-open']").isVisible()), "gaji publish/install entry leaked outside its workflow menu");

    const requiredContent = [
      "project-server-panel",
      "project-revision-list",
      "portable-metadata-panel",
      "portable-lineage-panel",
      "diagnostic-list",
      "run-hash",
      "gaji-shelf-open",
    ];
    for (const testId of requiredContent) {
      assert(await page.locator(`[data-testid='${testId}']`).count() === 1, `progressive disclosure deleted ${testId}`);
    }

    const menu = page.locator("[data-testid='project-more-menu']");
    await menu.locator("summary").focus();
    await page.keyboard.press("Enter");
    assert(await menu.getAttribute("open") !== null, "keyboard could not open project details menu");
    assert(await page.locator("[data-testid='gaji-shelf-open']").isVisible(), "gaji workflow entry unavailable on request");
    await page.click("[data-testid='gaji-shelf-open']");
    await page.waitForSelector("[data-testid='gaji-shelf-dialog'][open]");
    await page.click("[data-testid='gaji-shelf-close']");
    await menu.locator("summary").click();

    await page.locator("[data-testid='project-storage-open']").focus();
    await page.keyboard.press("Enter");
    assert(await page.locator("[data-testid='project-storage-drawer']").getAttribute("open") !== null, "storage drawer did not open from keyboard");
    assert(await page.locator("[data-testid='project-server-panel']").isVisible(), "server storage missing from storage drawer");
    assert(await page.locator("[data-testid='project-revision-list']").isVisible(), "revision list missing from storage drawer");
    assert(await page.locator("[data-testid='project-storage-close']").isVisible(), "storage drawer lacks an accessible close action");
    await page.keyboard.press("Escape");
    assert(await page.locator("[data-testid='project-storage-drawer']").getAttribute("open") === null, "Escape did not close storage drawer");

    await menu.locator("summary").click();
    await page.click("[data-testid='project-history-open']");
    assert(await page.locator("[data-testid='project-history-drawer']").getAttribute("open") !== null, "history drawer did not open");
    assert(await page.locator("[data-testid='portable-lineage-panel']").isVisible(), "project lineage missing from history drawer");
    assert(await page.locator("[data-testid='run-lineage']").isVisible() && await page.locator("[data-testid='run-archive-status']").isVisible(), "run lineage/archive missing from history drawer");
    await page.click("[data-testid='project-history-close']");

    await menu.locator("summary").click();
    await page.click("[data-testid='project-verification-open']");
    assert(await page.locator("[data-testid='project-verification-drawer']").getAttribute("open") !== null, "verification drawer did not open");
    assert(await page.locator("[data-testid='run-hash']").isVisible(), "state hash missing from verification drawer");
    assert(await page.locator("[data-testid='diagnostic-list']").isVisible(), "diagnostics missing from verification drawer");
    await page.click("[data-testid='project-verification-close']");

    await menu.locator("summary").click();
    await page.click("[data-testid='portable-project-settings']");
    assert(await page.locator("[data-testid='portable-project-dialog']").getAttribute("open") !== null, "portable dialog did not open");
    assert(await page.locator("[data-testid='portable-metadata-panel']").isVisible(), "portable metadata missing from import/export dialog");
    assert(await page.locator("[data-testid='portable-project-export-confirm']").isVisible(), "portable dialog lacks export action");
    await page.click("[data-testid='portable-project-dialog-close']");
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.click("[data-testid='portable-project-export']"),
    ]);
    assert(download.suggestedFilename().endsWith(".seamgrim-project.json"), "direct portable export compatibility changed");
    assert(await page.locator("[data-testid='portable-project-dialog']").getAttribute("open") === null, "direct export reopened the metadata dialog");

    await page.setViewportSize({ width: 600, height: 900 });
    const narrow = await page.evaluate(() => ({
      titleVisible: Boolean(document.querySelector("#project-title")?.offsetParent),
      runTop: document.querySelector("[data-testid='run-status']")?.getBoundingClientRect().top,
      sourceTop: document.querySelector("[data-testid='source-panel']")?.getBoundingClientRect().top,
      primaryTop: document.querySelector("[data-testid='bogae-panel']")?.getBoundingClientRect().top,
    }));
    assert(narrow.titleVisible && narrow.runTop < narrow.sourceTop, `narrow first-screen order regressed: ${JSON.stringify(narrow)}`);

    assert(pageErrors.length === 0, `page errors: ${pageErrors.join(" | ")}`);
    console.log("seamgrim_workbench_progressive_disclosure_browser: ok");
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
