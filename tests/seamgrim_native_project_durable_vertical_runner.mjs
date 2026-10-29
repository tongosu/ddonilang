#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import net from "node:net";
import { spawn } from "node:child_process";
import { chromium } from "playwright";

const ROOT = path.resolve(".");
const SERVER = path.join(ROOT, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function freePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}

async function startServer(dataDir) {
  const port = await freePort();
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
    stdio: ["ignore", "pipe", "pipe"],
  });
  const baseUrl = `http://127.0.0.1:${port}`;
  for (let index = 0; index < 100; index += 1) {
    try {
      const response = await fetch(`${baseUrl}/api/v1/projects/health`);
      if (response.ok) return { child, baseUrl };
    } catch (_) { /* retry */ }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("server health timeout");
}

async function stopServer(child) {
  if (child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

function rebase(link, baseUrl) {
  const url = new URL(link);
  return `${baseUrl}${url.pathname}${url.search}${url.hash}`;
}

async function stepToMadi(page, target) {
  for (let attempt = 0; attempt < 16; attempt += 1) {
    const snapshot = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    const current = Number(snapshot.activeFrame?.madi ?? -1);
    if (current >= target) return snapshot;
    await page.getByTestId("lifecycle-step").click();
    await page.waitForFunction(
      (previous) => Number(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi ?? -1) > previous,
      current,
      { timeout: 5_000 },
    ).catch(() => {});
  }
  const snapshot = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  throw new Error(`madi ${target} timeout: ${JSON.stringify({ status: snapshot.runtimeStatus, frame: snapshot.activeFrame, diagnostics: snapshot.diagnostics })}`);
}

async function runFourMadi(page) {
  await page.getByTestId("lifecycle-run").click();
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runtimeStatus === "running");
  return stepToMadi(page, 4);
}

async function createPhysics(page, baseUrl) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
  await page.getByTestId("new-ddn-project").click();
  await page.getByTestId("project-create-name").fill("서버 낙하 운동");
  await page.getByTestId("project-create-submit").click();
  await page.locator("#project-title").filter({ hasText: "서버 낙하 운동" }).waitFor();
}

const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-native-durable-"));
const browser = await chromium.launch({ headless: true });
let server = await startServer(dataDir);
try {
  const owner = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await owner.newPage();
  await createPhysics(page, server.baseUrl);
  const initial = await runFourMadi(page);
  const initialHash = initial.activeFrame.state_hash;
  await page.getByTestId("project-save").click();
  await page.waitForFunction(() => /서버 저장|server saved/u.test(document.querySelector("[data-testid='project-save-status']")?.textContent || ""));
  const editLink = await page.getByTestId("project-edit-link").getAttribute("href");
  const viewLink = await page.getByTestId("project-view-link").getAttribute("href");
  const saved = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(saved.serverProject.revision_count >= 1, "first server revision missing");
  const originalProjectId = saved.serverProject.project_id;
  await page.getByTestId("global-my-work").click();
  await page.getByTestId("project-search").fill("서버 낙하");
  const myWorkText = await page.getByTestId("my-work-list").textContent();
  assert(myWorkText.includes("서버 저장") && myWorkText.includes("편집 가능"), "my work source/access labels missing");
  assert(!myWorkText.includes("포물선 운동"), "my work title search did not filter unrelated projects");
  await owner.close();

  await stopServer(server.child);
  server = await startServer(dataDir);

  const reopenedContext = await browser.newContext({ viewport: { width: 1024, height: 768 } });
  const reopened = await reopenedContext.newPage();
  await reopened.goto(rebase(editLink, server.baseUrl), { waitUntil: "networkidle" });
  await reopened.locator("[data-testid='file-list'] .file-row").first().waitFor();
  assert(await reopened.locator("[data-testid='file-list'] .file-row").count() === 3, "reopen lost file tree");
  const replay = await runFourMadi(reopened);
  assert(replay.activeFrame.state_hash === initialHash, "server restart replay hash mismatch");
  await reopened.getByTestId("lifecycle-stop").click();
  await reopened.locator("[data-testid='file-list'] .file-row", { hasText: "app/main.ddn" }).click();
  await reopened.getByTestId("source-editor").fill(`${await reopened.getByTestId("source-editor").inputValue()}\n// revision 2`);
  await reopened.getByTestId("project-save").click();
  await reopened.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject.revision_count >= 2);

  await stopServer(server.child);
  const downloadPromise = reopened.waitForEvent("download");
  await reopened.getByTestId("portable-project-export").click();
  const download = await downloadPromise;
  const portableFile = path.join(dataDir, "native-project.seamgrim-project.json");
  await download.saveAs(portableFile);
  await reopenedContext.close();
  server = await startServer(dataDir);

  const viewContext = await browser.newContext({ viewport: { width: 778, height: 912 } });
  const view = await viewContext.newPage();
  await view.goto(rebase(viewLink, server.baseUrl), { waitUntil: "networkidle" });
  await view.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().capabilitySession?.role === "view");
  assert(await view.getByTestId("source-editor").isEditable() === false, "view capability allowed source edit");
  assert(!(await view.getByTestId("lifecycle-run").isDisabled()), "view capability did not allow local run");
  await view.getByTestId("lifecycle-run").click();
  await view.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.length >= 1);
  const viewRun = await view.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs[0]);
  assert(!viewRun.server_run_id, "view local run appended server archive");
  await view.getByTestId("lifecycle-stop").click();
  await view.getByTestId("project-clone").click();
  await view.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().capabilitySession?.role === "edit");
  assert(await view.getByTestId("source-editor").isEditable(), "clone did not become independent edit project");
  const cloneSnapshot = await view.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(cloneSnapshot.serverProject.project_id !== originalProjectId, "clone reused original project id");
  await view.getByTestId("source-editor").fill(`${await view.getByTestId("source-editor").inputValue()}\n// clone only`);
  await view.getByTestId("project-save").click();
  await view.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject.revision_count >= 2);
  await viewContext.close();

  const importedContext = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const imported = await importedContext.newPage();
  await imported.goto(`${server.baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
  await imported.setInputFiles("[data-testid='portable-project-import-input']", portableFile);
  await imported.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "ready");
  await imported.getByTestId("portable-project-open").click();
  assert(await imported.locator("[data-testid='file-list'] .file-row").count() === 3, "portable import lost graph files");
  const portableReplay = await runFourMadi(imported);
  assert(portableReplay.activeFrame.state_hash === initialHash, "portable replay hash mismatch");
  await importedContext.close();

  console.log("seamgrim_native_project_durable_vertical: ok");
} finally {
  await browser.close();
  await stopServer(server.child);
}
