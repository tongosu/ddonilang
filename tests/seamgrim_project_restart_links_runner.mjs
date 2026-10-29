#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawn, execFile } from "node:child_process";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const SERVER = path.resolve(ROOT, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function tailBuffer() {
  const chunks = [];
  return {
    push(data) {
      chunks.push(String(data || ""));
      while (chunks.join("").length > 4000) chunks.shift();
    },
    text() {
      return chunks.join("").slice(-4000);
    },
  };
}

function execFileText(command, args) {
  return new Promise((resolve) => {
    execFile(command, args, { windowsHide: true }, (error, stdout, stderr) => {
      resolve({ ok: !error, stdout: String(stdout || ""), stderr: String(stderr || "") });
    });
  });
}

async function waitForReadyRecord(child, stdoutBuffer, timeoutMs = 10_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`server exited before ready: ${child.exitCode}`);
    const line = stdoutBuffer.text().split(/\r?\n/u).find((row) => row.startsWith("SEAMGRIM_SERVER_READY "));
    if (line) return JSON.parse(line.slice("SEAMGRIM_SERVER_READY ".length));
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error("server ready record timeout");
}

async function waitForHealth(baseUrl, child) {
  for (let i = 0; i < 80; i += 1) {
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

async function startServer(dataDir) {
  const env = { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir };
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", "0"], {
    cwd: ROOT,
    env,
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  const stdout = tailBuffer();
  const stderr = tailBuffer();
  child.stdout.on("data", (data) => stdout.push(data));
  child.stderr.on("data", (data) => stderr.push(data));
  const ready = await waitForReadyRecord(child, stdout);
  const baseUrl = `http://${ready.host}:${ready.port}`;
  await waitForHealth(baseUrl, child);
  return { child, baseUrl, ready };
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  const exited = await new Promise((resolve) => {
    const timer = setTimeout(() => resolve(false), 3000);
    child.once("exit", () => {
      clearTimeout(timer);
      resolve(true);
    });
  });
  if (!exited && child.pid) {
    if (process.platform === "win32") {
      await execFileText("taskkill", ["/PID", String(child.pid), "/T", "/F"]);
    } else {
      try {
        process.kill(child.pid, "SIGKILL");
      } catch (_) {
        // Already gone.
      }
    }
  }
}

async function openEditorFromDdn(page, baseUrl, ddnText) {
  const encoded = Buffer.from(ddnText, "utf8").toString("base64");
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&ddn=${encodeURIComponent(encoded)}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.locator("#screen-run [data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
}

function rewriteLinkHost(link, baseUrl) {
  const source = new URL(link);
  const target = new URL(baseUrl);
  source.protocol = target.protocol;
  source.host = target.host;
  return source.href;
}

async function main() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-restart-links-"));
  let server = await startServer(dataDir);
  const browser = await chromium.launch({ headless: true });
  try {
    const ddnText = `설정 {
  제목: 재시작_링크.
}.

1 보여주기.
2 보여주기.
`;
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await context.newPage();
    await openEditorFromDdn(page, server.baseUrl, ddnText);
    await page.fill("#project-name-input", "재시작 링크");
    await page.click("#btn-project-save");
    await page.waitForFunction(() => {
      const status = document.querySelector("#project-save-status");
      const edit = document.querySelector("#project-edit-link");
      const view = document.querySelector("#project-view-link");
      return status?.dataset?.state === "saved"
        && String(edit?.href ?? "").includes("sg_role=edit")
        && String(view?.href ?? "").includes("sg_role=view");
    }, null, { timeout: 30_000 });
    const links = await page.evaluate(() => ({
      edit: String(document.querySelector("#project-edit-link")?.href ?? ""),
      view: String(document.querySelector("#project-view-link")?.href ?? ""),
    }));

    await stopServer(server.child);
    server = await startServer(dataDir);
    const editAfterRestart = rewriteLinkHost(links.edit, server.baseUrl);
    const viewAfterRestart = rewriteLinkHost(links.view, server.baseUrl);

    const editContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const editPage = await editContext.newPage();
    await editPage.goto(editAfterRestart, { waitUntil: "domcontentloaded" });
    await editPage.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
    await editPage.waitForFunction(() => {
      const text = String(document.querySelector("#ddn-textarea")?.value ?? "");
      const status = String(document.querySelector("#project-save-status")?.textContent ?? "");
      return text.includes("2 보여주기.") && status.includes("다시 열림");
    }, null, { timeout: 30_000 });
    const editState = await editPage.evaluate(() => ({
      readonly: Boolean(document.querySelector("#ddn-textarea")?.readOnly),
      save_disabled: Boolean(document.querySelector("#btn-project-save")?.disabled),
      url_scrubbed: !location.href.includes("sg_cap="),
    }));

    const viewContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const viewPage = await viewContext.newPage();
    await viewPage.goto(viewAfterRestart, { waitUntil: "domcontentloaded" });
    await viewPage.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
    await viewPage.waitForFunction(() => {
      const text = String(document.querySelector("#ddn-textarea")?.value ?? "");
      const status = String(document.querySelector("#project-save-status")?.textContent ?? "");
      return text.includes("2 보여주기.") && status.includes("다시 열림");
    }, null, { timeout: 30_000 });
    const viewState = await viewPage.evaluate(() => ({
      readonly: Boolean(document.querySelector("#ddn-textarea")?.readOnly),
      save_disabled: Boolean(document.querySelector("#btn-project-save")?.disabled),
      clone_disabled: Boolean(document.querySelector("#btn-project-clone")?.disabled),
      url_scrubbed: !location.href.includes("sg_cap="),
    }));

    assert(!editState.readonly && !editState.save_disabled, "edit link remains editable after restart");
    assert(editState.url_scrubbed, "edit restart link scrubs capability fragment");
    assert(viewState.readonly && viewState.save_disabled, "view link remains read-only after restart");
    assert(!viewState.clone_disabled, "view link can still clone after restart");
    assert(viewState.url_scrubbed, "view restart link scrubs capability fragment");

    console.log(JSON.stringify({
      schema: "seamgrim.project_restart_links.runner_result.v1",
      server_ready_schema: server.ready?.schema ?? "",
      requested_port: 0,
      actual_bound_port: Number(new URL(server.baseUrl).port),
      edit_link_reopened_after_restart: !editState.readonly && !editState.save_disabled,
      view_link_reopened_after_restart: viewState.readonly && viewState.save_disabled,
      view_clone_available_after_restart: !viewState.clone_disabled,
      fragments_scrubbed_after_restart: editState.url_scrubbed && viewState.url_scrubbed,
      data_dir_redacted: path.basename(dataDir),
    }));
  } finally {
    await browser.close();
    await stopServer(server.child);
    await fs.rm(dataDir, { recursive: true, force: true });
  }
}

await main();
