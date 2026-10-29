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
const OK = "seamgrim_public_share_clone_reopen: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function parseCapabilityLink(link) {
  const url = new URL(link);
  const params = new URLSearchParams(url.hash.replace(/^#/, ""));
  return {
    project_id: String(params.get("sg_project") ?? ""),
    role: String(params.get("sg_role") ?? ""),
    capability: String(params.get("sg_cap") ?? ""),
  };
}

function isExpectedConsoleError(text) {
  return text.includes("net::ERR_CONNECTION_RESET") ||
    text.includes("the server responded with a status of 409 (Conflict)");
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

async function startServer(dataDir = "", preferredPort = 0) {
  const durableDir = dataDir || await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-public-share-"));
  const port = preferredPort || await freePort();
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: durableDir },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", () => {});
  child.stderr.on("data", () => {});
  const baseUrl = `http://127.0.0.1:${port}`;
  for (let i = 0; i < 90; i += 1) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    try {
      const response = await fetch(`${baseUrl}/api/v1/projects/health`);
      if (response.ok) return { child, baseUrl, dataDir: durableDir, port };
    } catch (_) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
  throw new Error("project service health timeout");
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

async function main() {
  let server = await startServer();
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  try {
    const creator = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    const creatorPage = await creator.newPage();
    creatorPage.on("console", (msg) => {
      if (msg.type() === "error" && !isExpectedConsoleError(msg.text())) failures.push(`creator console: ${msg.text()}`);
    });
    await creatorPage.goto(`${server.baseUrl}/`, { waitUntil: "networkidle" });
    await creatorPage.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_projectile_xy_v1']", { timeout: 30_000 });
    const source = await creatorPage.locator("[data-testid='source-editor']").inputValue();
    await creatorPage.fill("[data-testid='source-editor']", source.replace("초기속도x: 수 <- (8.0)", "초기속도x: 수 <- (12.0)"));
    await creatorPage.click("[data-testid='project-save']");
    await creatorPage.waitForFunction(() => String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? "").includes("서버 저장 완료"), null, { timeout: 30_000 });
    await creatorPage.click("[data-testid='lifecycle-run']");
    await creatorPage.waitForFunction(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snapshot?.runs?.[0]?.latest_state_hash &&
        (snapshot.runs[0].server_run_id || snapshot.runArchives?.length >= 1);
    }, null, { timeout: 30_000 });
    const creatorState = await creatorPage.evaluate(() => ({
      editHref: document.querySelector("[data-testid='project-edit-link']")?.href ?? "",
      viewHref: document.querySelector("[data-testid='project-view-link']")?.href ?? "",
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    const editCap = parseCapabilityLink(creatorState.editHref);
    const viewCap = parseCapabilityLink(creatorState.viewHref);
    assert(editCap.project_id && editCap.role === "edit", `creator edit link invalid: ${creatorState.editHref}`);
    assert(viewCap.project_id === editCap.project_id && viewCap.role === "view", `creator view link invalid: ${creatorState.viewHref}`);
    assert(creatorState.snapshot.serverProject.head_revision_id, "creator head revision missing");
    assert(creatorState.snapshot.runArchives.length >= 1 || creatorState.snapshot.runs[0].server_run_id, "creator run archive missing");

    const viewer = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    const viewPage = await viewer.newPage();
    viewPage.on("console", (msg) => {
      if (msg.type() === "error" && !isExpectedConsoleError(msg.text())) failures.push(`viewer console: ${msg.text()}`);
    });
    await viewPage.goto(creatorState.viewHref, { waitUntil: "domcontentloaded" });
    await viewPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().capabilitySession?.role === "view", null, { timeout: 30_000 });
    const viewState = await viewPage.evaluate(() => ({
      href: location.href,
      saveDisabled: document.querySelector("[data-testid='project-save']")?.disabled,
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(!viewState.href.includes("sg_cap="), "view capability fragment was not scrubbed from URL");
    assert(viewState.saveDisabled === true, "viewer save button should be disabled");
    assert(viewState.source.includes("초기속도x: 수 <- (12.0)"), "viewer did not receive creator source");
    const rejected = await viewPage.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.saveProjectSnapshot());
    assert(rejected?.code === "E_PROJECT_VIEW_WRITE_REJECTED", `viewer write not rejected: ${JSON.stringify(rejected)}`);

    await viewPage.locator("[data-testid='project-more-menu'] > summary").click();
    await viewPage.click("[data-testid='project-storage-open']");
    await viewPage.waitForFunction(() => document.querySelector("[data-testid='project-storage-drawer']")?.open === true, null, { timeout: 10_000 });
    await viewPage.click("[data-testid='project-clone']");
    await viewPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().capabilitySession?.role === "edit", null, { timeout: 30_000 });
    const cloneState = await viewPage.evaluate(() => ({
      editHref: document.querySelector("[data-testid='project-edit-link']")?.href ?? "",
      saveDisabled: document.querySelector("[data-testid='project-save']")?.disabled,
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    const cloneCap = parseCapabilityLink(cloneState.editHref);
    assert(cloneCap.project_id && cloneCap.project_id !== viewCap.project_id, "clone project id must be distinct");
    assert(cloneState.saveDisabled === false, "clone must be editable");
    await viewer.close();

    await stopServer(server.child);
    server = await startServer(server.dataDir, server.port);
    await creatorPage.goto(`${server.baseUrl}/`, { waitUntil: "networkidle" });
    await creatorPage.click("[data-testid='global-my-work']");
    await creatorPage.waitForSelector("[data-testid='my-work-item']", { timeout: 30_000 });
    await creatorPage.click("[data-testid='my-work-item']");
    await creatorPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().serverProject?.head_revision_id, null, { timeout: 30_000 });
    const reopened = await creatorPage.evaluate(() => ({
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(reopened.snapshot.serverProject.project_id === editCap.project_id, `reopened wrong project: ${JSON.stringify(reopened.snapshot.serverProject)}`);
    assert(reopened.source.includes("초기속도x: 수 <- (12.0)"), "server restart reopen lost creator source");
    assert(reopened.snapshot.serverProject.head_revision_id === creatorState.snapshot.serverProject.head_revision_id, "reopen head revision mismatch");

    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);
    console.log(JSON.stringify({
      schema: "seamgrim.public_share_clone_reopen.runner_result.v1",
      creator_project_id: editCap.project_id,
      clone_project_id: cloneCap.project_id,
      reopened_revision_id: reopened.snapshot.serverProject.head_revision_id,
      data_dir: server.dataDir,
    }));
    console.log(OK);
    await creator.close();
  } finally {
    await browser.close();
    await stopServer(server.child);
  }
}

await main();
