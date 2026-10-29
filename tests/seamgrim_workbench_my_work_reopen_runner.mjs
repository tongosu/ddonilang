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

async function startServer(dataDir, port) {
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", () => {});
  child.stderr.on("data", () => {});
  const baseUrl = `http://127.0.0.1:${port}`;
  await waitForHealth(baseUrl, child);
  return { child, baseUrl };
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

async function waitForWorkbenchReady(page, { requireSave = false } = {}) {
  await page.waitForFunction(
    (needsSave) => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      if (snapshot?.readiness?.initialization !== "ready") return false;
      return !needsSave || document.querySelector("[data-testid='project-save']")?.disabled === false;
    },
    requireSave,
    { timeout: 30_000 },
  );
}

async function main() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-workbench-g7-"));
  const port = await freePort();
  let server = await startServer(dataDir, port);
  let browser = null;
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await context.newPage();
    await page.goto(`${server.baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-testid='source-editor']", { state: "visible", timeout: 30_000 });
    await waitForWorkbenchReady(page, { requireSave: true });
    await page.fill("[data-testid='source-editor']", `설정 { 제목: "내 작업 재열기". 마디수: 4. }.\n\n1 보여주기.\n`);
    await page.click("[data-testid='project-save']");
    await page.waitForFunction(() => String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? "").includes("서버 저장 완료"), null, { timeout: 30_000 });
    const saved = await page.evaluate(() => ({
      editLink: String(document.querySelector("[data-testid='project-edit-link']")?.href ?? ""),
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      storage: localStorage.getItem("seamgrim.workbench.my_work_index.v1") || "",
    }));
    assert(saved.snapshot.myWork.length >= 1, "my work index recorded after save");
    assert(saved.storage.includes(saved.snapshot.serverProject.project_id), "local index includes project id");
    assert(!JSON.stringify(saved.snapshot).includes("sg_cap="), "debug snapshot does not expose capability fragment");
    await page.click("[data-testid='global-my-work']");
    await page.waitForFunction(() => document.querySelectorAll("[data-testid='my-work-item']").length >= 1, null, { timeout: 30_000 });
    const beforeRestart = await page.evaluate(() => ({
      count: document.querySelectorAll("[data-testid='my-work-item']").length,
      text: String(document.querySelector("[data-testid='my-work-list']")?.textContent ?? ""),
    }));
    assert(beforeRestart.count >= 1, "my work item visible before restart");

    await stopServer(server.child);
    server = await startServer(dataDir, port);

    const reopened = await context.newPage();
    await reopened.goto(`${server.baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "domcontentloaded" });
    await reopened.waitForSelector("[data-testid='approved-workbench-root']", { state: "visible", timeout: 30_000 });
    await waitForWorkbenchReady(reopened);
    await reopened.click("[data-testid='global-my-work']");
    await reopened.waitForFunction(() => document.querySelectorAll("[data-testid='my-work-item']").length >= 1, null, { timeout: 30_000 });
    const itemHref = await reopened.locator("[data-testid='my-work-item']").first().getAttribute("href");
    assert(String(itemHref ?? "").includes("#sg_project="), "my work item carries fragment link");
    await reopened.locator("[data-testid='my-work-item']").first().click();
    await reopened.waitForFunction(() => {
      const source = String(document.querySelector("[data-testid='source-editor']")?.value ?? "");
      const status = String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? "");
      return source.includes("내 작업 재열기") && status.includes("서버 저장 완료");
    }, null, { timeout: 30_000 });
    const reopenedState = await reopened.evaluate(() => ({
      href: location.href,
      source: String(document.querySelector("[data-testid='source-editor']")?.value ?? ""),
      myWorkText: String(document.querySelector("[data-testid='my-work-list']")?.textContent ?? ""),
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(!reopenedState.href.includes("sg_cap="), "my work reopen scrubbed capability fragment");
    assert(reopenedState.source.includes("내 작업 재열기"), "server truth reopened after restart");
    assert(reopenedState.snapshot.serverProject.project_id === saved.snapshot.serverProject.project_id, "same project reopened from index");

    console.log(JSON.stringify({
      schema: "seamgrim.workbench_my_work_reopen.runner_result.v1",
      project_id: saved.snapshot.serverProject.project_id,
      before_restart_items: beforeRestart.count,
      reopened_after_restart: reopenedState.source.includes("내 작업 재열기"),
      fragment_scrubbed: !reopenedState.href.includes("sg_cap="),
      data_dir: dataDir,
    }));
  } finally {
    await browser?.close();
    await stopServer(server.child);
    await fs.rm(dataDir, { recursive: true, force: true });
  }
}

await main();
