#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const SERVER = path.resolve(ROOT, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function freePort() {
  return new Promise((resolve, reject) => {
    import("node:net").then(({ default: net }) => {
      const server = net.createServer();
      server.once("error", reject);
      server.listen(0, "127.0.0.1", () => {
        const address = server.address();
        const port = address && typeof address !== "string" ? address.port : 0;
        server.close(() => resolve(port));
      });
    }).catch(reject);
  });
}

async function waitForHealth(baseUrl, child) {
  for (let i = 0; i < 90; i += 1) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    try {
      const response = await fetch(`${baseUrl}/api/v1/projects/health`);
      if (response.ok) return;
    } catch (_) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
  throw new Error("server health timeout");
}

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-workbench-g3-"));
  const port = await freePort();
  const env = { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir };
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", () => {});
  child.stderr.on("data", () => {});
  const baseUrl = `http://127.0.0.1:${port}`;
  await waitForHealth(baseUrl, child);
  return { child, baseUrl, dataDir };
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

async function waitSaved(page, minRevisionRows = 1) {
  try {
    await page.waitForFunction((minRows) => {
      const revisions = document.querySelectorAll("[data-testid='project-revision-item']");
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snapshot?.persistenceStatus?.server_saved === true &&
        Boolean(snapshot?.serverProject?.headRevisionId || snapshot?.serverProject?.head_revision_id) &&
        revisions.length >= minRows;
    }, minRevisionRows, { timeout: 30_000 });
  } catch (err) {
    const detail = await page.evaluate(() => ({
      status: String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? ""),
      revision_rows: document.querySelectorAll("[data-testid='project-revision-item']").length,
      diagnostics: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().diagnostics?.slice(-5) ?? [],
      persistenceStatus: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().persistenceStatus ?? null,
      serverProject: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().serverProject ?? null,
    }));
    throw new Error(`waitSaved timeout: ${JSON.stringify(detail)}`);
  }
}

async function openStorageDrawer(page) {
  const alreadyOpen = await page.evaluate(() => document.querySelector("[data-testid='project-storage-drawer']")?.open === true);
  if (alreadyOpen) return;
  await page.evaluate(() => {
    const menu = document.querySelector("[data-testid='project-more-menu']");
    if (menu) menu.open = true;
  });
  await page.click("[data-testid='project-storage-open']");
  await page.waitForFunction(() => document.querySelector("[data-testid='project-storage-drawer']")?.open === true, null, { timeout: 10_000 });
}

async function waitWorkbenchReady(page) {
  await page.waitForFunction(() => typeof window.__SEAMGRIM_APPROVED_WORKBENCH__?.saveProjectSnapshot === "function", null, { timeout: 30_000 });
}

async function main() {
  const { child, baseUrl, dataDir } = await startServer();
  let browser = null;
  try {
    browser = await chromium.launch({ headless: true });
    const contextA = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const pageA = await contextA.newPage();
    await pageA.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "domcontentloaded" });
    await pageA.waitForSelector("[data-testid='source-editor']", { state: "visible", timeout: 30_000 });
    await waitWorkbenchReady(pageA);
    const sourceV1 = `설정 { 제목: "워크벤치 충돌 복원". 마디수: 4. }.\n\n1 보여주기.\n`;
    await pageA.fill("[data-testid='source-editor']", sourceV1);
    await pageA.click("[data-testid='project-save']");
    await waitSaved(pageA, 1);
    const firstSave = await pageA.evaluate(() => ({
      edit_link: String(document.querySelector("[data-testid='project-edit-link']")?.href ?? ""),
      first_revision: String(document.querySelector("[data-testid='project-revision-item']")?.getAttribute("data-project-revision-id") ?? ""),
    }));
    assert(firstSave.first_revision.startsWith("sgr_"), "first revision id visible");

    const contextB = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const pageB = await contextB.newPage();
    await pageB.goto(firstSave.edit_link, { waitUntil: "domcontentloaded" });
    await pageB.waitForSelector("[data-testid='source-editor']", { state: "visible", timeout: 30_000 });
    await waitWorkbenchReady(pageB);
    await pageB.waitForFunction(() => String(document.querySelector("[data-testid='source-editor']")?.value ?? "").includes("1 보여주기."), null, { timeout: 30_000 });
    const openedB = await pageB.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject);
    assert(openedB.head_revision_id === firstSave.first_revision, `page B opened stale first revision: ${JSON.stringify(openedB)} expected=${firstSave.first_revision}`);

    await pageA.fill("[data-testid='source-editor']", `${sourceV1}\n10 보여주기.\n`);
    await pageA.click("[data-testid='project-save']");
    await waitSaved(pageA, 2);
    const secondState = await pageA.evaluate(() => ({
      serverMeta: String(document.querySelector("[data-testid='project-server-meta']")?.textContent ?? ""),
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      head_revision: [...document.querySelectorAll("[data-testid='project-revision-item']")]
        .find((node) => node.getAttribute("data-head") === "true")
        ?.getAttribute("data-project-revision-id") ?? "",
      rows: document.querySelectorAll("[data-testid='project-revision-item']").length,
    }));
    assert(secondState.rows === 2, `page A has two revisions after append: ${JSON.stringify(secondState)}`);

    await pageB.fill("[data-testid='source-editor']", `${sourceV1}\n20 보여주기.\n`);
    await pageB.click("[data-testid='project-save']");
    await pageB.waitForTimeout(2500);
    const afterConflictAttempt = await pageB.evaluate(() => ({
      status: String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? ""),
      panelHidden: document.querySelector("[data-testid='project-conflict-panel']")?.classList.contains("hidden"),
      serverMeta: String(document.querySelector("[data-testid='project-server-meta']")?.textContent ?? ""),
      revisions: [...document.querySelectorAll("[data-testid='project-revision-item']")].map((node) => ({
        id: String(node.getAttribute("data-project-revision-id") ?? ""),
        head: String(node.getAttribute("data-head") ?? ""),
      })),
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(afterConflictAttempt.panelHidden === false, `conflict panel did not open: ${JSON.stringify(afterConflictAttempt)}`);
    await pageB.waitForFunction((expectedHead) => {
      const server = String(document.querySelector("[data-testid='project-conflict-server']")?.textContent ?? "");
      const local = String(document.querySelector("[data-testid='project-conflict-local-package']")?.value ?? "");
      return server.includes(expectedHead) && local.includes("20 보여주기.");
    }, secondState.head_revision, { timeout: 20_000 });
    const conflictState = await pageB.evaluate(() => ({
      message: String(document.querySelector("[data-testid='project-conflict-message']")?.textContent ?? ""),
      server: String(document.querySelector("[data-testid='project-conflict-server']")?.textContent ?? ""),
      local_package: String(document.querySelector("[data-testid='project-conflict-local-package']")?.value ?? ""),
      revision_rows: document.querySelectorAll("[data-testid='project-revision-item']").length,
    }));
    assert(conflictState.message.includes("먼저 저장"), "conflict message is Korean");
    assert(conflictState.server.includes(secondState.head_revision), "conflict shows server head");
    assert(conflictState.local_package.includes("20 보여주기."), "conflict preserves local package");
    assert(conflictState.revision_rows === 2, "conflict page keeps revision history visible");
    await openStorageDrawer(pageB);
    await pageB.click("[data-testid='project-conflict-copy-local']");
    await pageB.waitForFunction(() => String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? "").includes("보존됨"), null, { timeout: 10_000 });

    await openStorageDrawer(pageA);
    const firstRestoreButton = pageA.locator(`[data-project-restore-revision="${firstSave.first_revision}"]`);
    await firstRestoreButton.click();
    await pageA.waitForTimeout(2500);
    const afterRestoreAttempt = await pageA.evaluate(() => ({
      status: String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? ""),
      rows: document.querySelectorAll("[data-testid='project-revision-item']").length,
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      list: String(document.querySelector("[data-testid='project-revision-list']")?.textContent ?? ""),
    }));
    assert(
      afterRestoreAttempt.status.includes("복원됨") && afterRestoreAttempt.rows >= 3,
      `restore did not complete: ${JSON.stringify(afterRestoreAttempt)}`,
    );
    const restoredState = await pageA.evaluate(() => ({
      source: String(document.querySelector("[data-testid='source-editor']")?.value ?? ""),
      revision_rows: document.querySelectorAll("[data-testid='project-revision-item']").length,
      head_revision: [...document.querySelectorAll("[data-testid='project-revision-item']")]
        .find((node) => node.getAttribute("data-head") === "true")
        ?.getAttribute("data-project-revision-id") ?? "",
      list_text: String(document.querySelector("[data-testid='project-revision-list']")?.textContent ?? ""),
    }));
    assert(restoredState.revision_rows === 3, "restore creates third revision");
    assert(restoredState.head_revision !== secondState.head_revision, "restore creates a new head");
    assert(restoredState.source.includes("1 보여주기."), "restored source contains original line");
    assert(!restoredState.source.includes("10 보여주기."), "restored source removed later page A edit");
    assert(restoredState.list_text.includes("content"), "revision list includes content audit");
    assert(restoredState.list_text.includes("package"), "revision list includes package audit");

    console.log(JSON.stringify({
      schema: "seamgrim.workbench_revision_conflict_restore.runner_result.v1",
      conflict_detected: conflictState.message.includes("먼저 저장"),
      local_package_preserved: conflictState.local_package.includes("20 보여주기."),
      restore_created_new_revision: restoredState.revision_rows === 3,
      restored_source_loaded: restoredState.source.includes("1 보여주기.") && !restoredState.source.includes("10 보여주기."),
      data_dir: dataDir,
    }));
  } finally {
    await browser?.close();
    await stopServer(child);
    await fs.rm(dataDir, { recursive: true, force: true });
  }
}

await main();
