#!/usr/bin/env node

import fs from "node:fs/promises";
import net from "node:net";
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

function measureSimpleCommand() {
  const started = performance.now();
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, ["-e", "process.exit(0)"], { stdio: "ignore" });
    child.once("error", reject);
    child.once("exit", (code) => {
      if (code !== 0) reject(new Error(`simple process command exited ${code}`));
      else resolve({ latencyMs: performance.now() - started, pid: child.pid, exited: true });
    });
  });
}

function isPortListening(port) {
  return new Promise((resolve) => {
    const socket = net.createConnection({ host: "127.0.0.1", port });
    let settled = false;
    const finish = (value) => {
      if (settled) return;
      settled = true;
      socket.destroy();
      resolve(value);
    };
    socket.once("connect", () => finish(true));
    socket.once("error", () => finish(false));
    socket.setTimeout(1_000, () => finish(false));
  });
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

async function fetchJson(baseUrl, pathName, capability, body = null) {
  const response = await fetch(`${baseUrl}${pathName}`, {
    method: body ? "POST" : "GET",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${capability}`,
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  let payload = {};
  try {
    payload = await response.json();
  } catch (_) {
    payload = {};
  }
  return { status: response.status, payload };
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
    if (child.exitCode !== null || child.signalCode !== null) {
      throw new Error(`server exited: code=${child.exitCode} signal=${child.signalCode}`);
    }
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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-workbench-g6-"));
  const port = await freePort();
  const started = performance.now();
  const env = { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir };
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const stderr = [];
  child.stdout.on("data", () => {});
  child.stderr.on("data", (chunk) => stderr.push(String(chunk)));
  const baseUrl = `http://127.0.0.1:${port}`;
  try {
    await waitForHealth(baseUrl, child);
    return { child, baseUrl, dataDir, port, serverSpawnToHealthMs: performance.now() - started };
  } catch (err) {
    await stopServer(child);
    await fs.rm(dataDir, { recursive: true, force: true });
    throw new Error(`${String(err)}\n${stderr.join("")}`);
  }
}

async function stopServer(child) {
  if (!child || child.exitCode !== null || child.signalCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

async function waitForWorkbenchReady(page, baseUrl, { role = "", requireClone = false } = {}) {
  try {
    await page.waitForFunction(({ expectedRole, cloneRequired }) => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      if (snapshot?.readiness?.workbench_api_ready !== true || snapshot?.readiness?.initialization !== "ready") return false;
      if (expectedRole && snapshot?.capabilitySession?.role !== expectedRole) return false;
      if (expectedRole === "view" && !snapshot?.serverProject?.project_id) return false;
      if (snapshot?.controls?.lifecycle_run?.enabled !== true) return false;
      if (cloneRequired && snapshot?.controls?.project_clone?.enabled !== true) return false;
      return expectedRole === "view" || snapshot?.controls?.project_save?.enabled === true;
    }, { expectedRole: role, cloneRequired: requireClone }, { timeout: 30_000 });
  } catch (err) {
    const product = await page.evaluate(() => ({
      api: Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot),
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.() ?? null,
      controls: ["project-save", "lifecycle-run", "project-clone"].map((testId) => {
        const node = document.querySelector(`[data-testid='${testId}']`);
        return { test_id: testId, disabled: node?.disabled ?? null, reason: node?.dataset?.disabledReason ?? "" };
      }),
      status: document.querySelector("[data-testid='project-save-status']")?.textContent ?? "",
    })).catch(() => null);
    const health = await fetch(`${baseUrl}/api/v1/projects/health`)
      .then((response) => ({ status: response.status, ok: response.ok }))
      .catch((healthError) => ({ status: 0, ok: false, error: String(healthError) }));
    throw new Error(`workbench capability readiness timeout: ${JSON.stringify({ role, requireClone, product, health, cause: String(err) })}`);
  }
}

async function openProjectMoreMenu(page) {
  const isOpen = await page.locator("[data-testid='project-more-menu']").evaluate((node) => node.open);
  if (!isOpen) await page.click("[data-testid='project-more-menu'] > summary");
}

async function openProjectStorageDrawer(page) {
  await openProjectMoreMenu(page);
  await page.click("[data-testid='project-storage-open']");
  await page.waitForFunction(() => document.querySelector("[data-testid='project-storage-drawer']")?.open === true);
}

async function main() {
  const simpleCommand = await measureSimpleCommand();
  const { child, baseUrl, dataDir, port, serverSpawnToHealthMs } = await startServer();
  let browser = null;
  let result = null;
  try {
    const browserStarted = performance.now();
    browser = await chromium.launch({ headless: true });
    const browserLaunchLatencyMs = performance.now() - browserStarted;
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await context.newPage();
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "domcontentloaded" });
    await waitForWorkbenchReady(page, baseUrl);
    await page.fill("[data-testid='source-editor']", `설정 { 제목: "권한 공유 복제". 마디수: 4. }.\n\n1 보여주기.\n`);
    const beforeSaveGeneration = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().persistence.generation);
    await page.click("[data-testid='project-save']");
    await page.waitForFunction((generation) => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snapshot?.persistence?.completed_generation > generation
        && snapshot?.persistence?.phase === "saved"
        && snapshot?.persistence?.mode === "server_project"
        && snapshot?.serverProject?.head_revision_id;
    }, beforeSaveGeneration, { timeout: 30_000 });
    const links = await page.evaluate(() => ({
      edit: String(document.querySelector("[data-testid='project-edit-link']")?.href ?? ""),
      view: String(document.querySelector("[data-testid='project-view-link']")?.href ?? ""),
      snap: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    const editCap = parseCapabilityLink(links.edit);
    const viewCap = parseCapabilityLink(links.view);
    assert(editCap.role === "edit" && viewCap.role === "view", `edit/view links separated: ${JSON.stringify({ links, editCap, viewCap })}`);
    assert(!JSON.stringify(links.snap).includes(editCap.capability), "edit capability redacted from snapshot");
    assert(!JSON.stringify(links.snap).includes(viewCap.capability), "view capability redacted from snapshot");

    const viewContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const viewPage = await viewContext.newPage();
    await viewPage.goto(links.view, { waitUntil: "domcontentloaded" });
    await waitForWorkbenchReady(viewPage, baseUrl, { role: "view", requireClone: true });
    const viewInitial = await viewPage.evaluate(() => ({
      href: location.href,
      saveDisabled: document.querySelector("[data-testid='project-save']")?.disabled,
      restoreButtons: document.querySelectorAll("[data-testid='project-restore-revision']").length,
      cloneDisabled: document.querySelector("[data-testid='project-clone']")?.disabled,
      runDisabled: document.querySelector("[data-testid='lifecycle-run']")?.disabled,
      saveResult: null,
      snap: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(!viewInitial.href.includes("sg_cap="), "view capability fragment scrubbed");
    assert(viewInitial.saveDisabled === true, "view save button disabled");
    assert(viewInitial.restoreButtons === 0, "view restore buttons hidden");
    assert(viewInitial.cloneDisabled === false, "view clone available");
    assert(viewInitial.runDisabled === false, `view local run enabled: ${JSON.stringify(viewInitial.snap.controls)}`);
    const saveResult = await viewPage.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.saveProjectSnapshot());
    assert(saveResult?.code === "E_PROJECT_VIEW_WRITE_REJECTED", "debug save rejected for view role");
    await viewPage.click("[data-testid='lifecycle-run']");
    await viewPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.length >= 1, null, { timeout: 30_000 });
    const viewRun = await viewPage.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs[0]);
    assert(!viewRun.server_run_id, "view run did not append server archive");
    const denied = await fetchJson(
      baseUrl,
      `/api/v1/projects/${viewCap.project_id}/runs`,
      viewCap.capability,
      { source_revision_id: links.snap.serverProject.head_revision_id, recipe_hash: "candidate:recipe:view-denied" },
    );
    assert(denied.status === 403, "view capability denied by run API");

    await openProjectStorageDrawer(viewPage);
    await viewPage.click("[data-testid='project-clone']");
    await viewPage.waitForFunction(() => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      const editLink = String(document.querySelector("[data-testid='project-edit-link']")?.href ?? "");
      return snap.capabilitySession?.role === "edit" &&
        snap.serverProject.project_id &&
        editLink.includes("#sg_project=") &&
        document.querySelector("[data-testid='project-save']")?.disabled === false;
    }, null, { timeout: 30_000 });
    const cloneState = await viewPage.evaluate(() => ({
      saveDisabled: document.querySelector("[data-testid='project-save']")?.disabled,
      editLink: String(document.querySelector("[data-testid='project-edit-link']")?.href ?? ""),
      snap: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    const cloneCap = parseCapabilityLink(cloneState.editLink);
    assert(cloneState.saveDisabled === false, `clone edit save enabled: ${JSON.stringify(cloneState)}`);
    assert(cloneCap.project_id && cloneCap.project_id !== viewCap.project_id, "clone has new project id");
    assert(cloneCap.role === "edit", "clone issues edit capability");
    const cloned = await fetchJson(baseUrl, `/api/v1/projects/${cloneCap.project_id}`, cloneCap.capability);
    assert(cloned.status === 200, "clone project readable");
    assert(cloned.payload.origin_project_id === viewCap.project_id, "clone origin project recorded");
    assert(cloned.payload.origin_revision_id === links.snap.serverProject.head_revision_id, "clone origin revision recorded");

    result = {
      schema: "seamgrim.workbench_capability_share_clone.runner_result.v2",
      view_save_disabled: viewInitial.saveDisabled,
      view_debug_save_rejected: saveResult?.code === "E_PROJECT_VIEW_WRITE_REJECTED",
      view_run_server_append_absent: !viewRun.server_run_id,
      clone_project_id: cloneCap.project_id,
      clone_origin_project: cloned.payload.origin_project_id,
      readiness: viewInitial.snap.readiness,
      view_controls: viewInitial.snap.controls,
      environment: {
        simple_process_command_latency_ms: Number(simpleCommand.latencyMs.toFixed(3)),
        server_spawn_to_health_latency_ms: Number(serverSpawnToHealthMs.toFixed(3)),
        browser_launch_latency_ms: Number(browserLaunchLatencyMs.toFixed(3)),
      },
      owned_child_pid: child.pid,
    };
    await viewContext.close();
    await context.close();
  } finally {
    await browser?.close();
    await stopServer(child);
    await fs.rm(dataDir, { recursive: true, force: true });
  }
  const [socketOpen, dataDirStat] = await Promise.all([
    isPortListening(port),
    fs.stat(dataDir).catch(() => null),
  ]);
  const childRunning = child.exitCode === null && child.signalCode === null;
  assert(childRunning === false, `owned child process still running: ${child.pid}`);
  assert(socketOpen === false, `owned server socket still listening: ${port}`);
  assert(dataDirStat === null, `owned temp dir still exists: ${dataDir}`);
  assert(result, "runner result missing after cleanup");
  result.owned_residual = {
    processes: childRunning ? 1 : 0,
    sockets: socketOpen ? 1 : 0,
    temp_dirs: dataDirStat ? 1 : 0,
  };
  console.log(JSON.stringify(result));
}

await main();
