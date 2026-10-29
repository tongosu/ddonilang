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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-workbench-g2-"));
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
  await page.waitForFunction((minRows) => {
    const status = document.querySelector("[data-testid='project-save-status']");
    const revisions = document.querySelectorAll("[data-testid='project-revision-item']");
    return String(status?.textContent ?? "").includes("서버 저장 완료") && revisions.length >= minRows;
  }, minRevisionRows, { timeout: 30_000 });
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
  const { child, baseUrl, dataDir } = await startServer();
  let browser = null;
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await context.newPage();
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-testid='approved-workbench-root']", { state: "visible", timeout: 30_000 });
    await page.waitForSelector("[data-testid='source-editor']", { state: "visible", timeout: 30_000 });
    await waitForWorkbenchReady(page, { requireSave: true });
    await page.fill("[data-testid='source-editor']", `설정 { 제목: "워크벤치 서버 저장". 마디수: 4. }.\n\n1 보여주기.\n`);
    await page.click("[data-testid='project-save']");
    await waitSaved(page, 1);
    const firstSave = await page.evaluate(() => ({
      edit_link: String(document.querySelector("[data-testid='project-edit-link']")?.href ?? ""),
      view_link: String(document.querySelector("[data-testid='project-view-link']")?.href ?? ""),
      meta: String(document.querySelector("[data-testid='project-server-meta']")?.textContent ?? ""),
      status: String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? ""),
      revisions: [...document.querySelectorAll("[data-testid='project-revision-item']")].map((node) => String(node.textContent ?? "")),
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(firstSave.edit_link.includes("#sg_project="), "edit link uses capability fragment");
    assert(firstSave.edit_link.includes("sg_cap="), "edit link has capability");
    assert(!firstSave.edit_link.includes("?sg_cap="), "capability is not query data");
    assert(firstSave.view_link.includes("#sg_project="), "view link uses capability fragment");
    assert(firstSave.snapshot.capabilitySession.capability_redacted === true, "debug snapshot redacts capability");
    assert(firstSave.revisions.length === 1, "first save lists one revision");

    await page.fill("[data-testid='source-editor']", `설정 { 제목: "워크벤치 서버 저장". 마디수: 4. }.\n\n1 보여주기.\n2 보여주기.\n`);
    await page.click("[data-testid='project-save']");
    await waitSaved(page, 2);
    const secondSave = await page.evaluate(() => ({
      status: String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? ""),
      revisions: [...document.querySelectorAll("[data-testid='project-revision-item']")].map((node) => ({
        text: String(node.textContent ?? ""),
        head: String(node.getAttribute("data-head") ?? ""),
      })),
    }));
    assert(secondSave.revisions.length === 2, "append save lists two revisions");
    assert(secondSave.revisions.some((row) => row.head === "true"), "head revision marked");
    assert(secondSave.revisions.every((row) => row.text.includes("content")), "revision rows include content audit");

    const reopenedContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const reopened = await reopenedContext.newPage();
    await reopened.goto(firstSave.edit_link, { waitUntil: "domcontentloaded" });
    await reopened.waitForSelector("[data-testid='source-editor']", { state: "visible", timeout: 30_000 });
    await waitForWorkbenchReady(reopened);
    await reopened.waitForFunction(() => {
      const text = String(document.querySelector("[data-testid='source-editor']")?.value ?? "");
      const status = String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? "");
      return text.includes("2 보여주기.") && status.includes("서버 저장 완료");
    }, null, { timeout: 30_000 });
    const reopenedState = await reopened.evaluate(() => ({
      href: location.href,
      source: String(document.querySelector("[data-testid='source-editor']")?.value ?? ""),
      revisions: document.querySelectorAll("[data-testid='project-revision-item']").length,
      session: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().capabilitySession,
    }));
    assert(!reopenedState.href.includes("sg_cap="), "capability fragment scrubbed after capture");
    assert(reopenedState.source.includes("2 보여주기."), "reopened latest source preserved");
    assert(reopenedState.revisions === 2, "reopened revision history visible");
    assert(reopenedState.session.role === "edit", "edit capability session restored");

    console.log(JSON.stringify({
      schema: "seamgrim.workbench_server_project_save_reopen.runner_result.v1",
      server_project_saved: firstSave.status.includes("서버 저장 완료"),
      append_revision_count: secondSave.revisions.length,
      capability_fragment_scrubbed: !reopenedState.href.includes("sg_cap="),
      revision_audit_visible: secondSave.revisions.every((row) => row.text.includes("content")),
      data_dir: dataDir,
    }));
  } finally {
    await browser?.close();
    await stopServer(child);
    await fs.rm(dataDir, { recursive: true, force: true });
  }
}

await main();
