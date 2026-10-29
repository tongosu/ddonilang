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
const VIEWPORTS = [
  { width: 1440, height: 900 },
  { width: 1024, height: 768 },
  { width: 778, height: 912 },
  { width: 390, height: 844 },
];

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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-workbench-g9-"));
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", "0"], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
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
  return { child, baseUrl, dataDir, ready };
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

async function canvasNonblank(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const ctx = canvas?.getContext?.("2d");
    if (!canvas || !ctx) return false;
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let colored = 0;
    for (let i = 0; i < data.length; i += 16) {
      if (data[i] < 245 || data[i + 1] < 245 || data[i + 2] < 245) colored += 1;
      if (colored > 80) return true;
    }
    return false;
  });
}

async function qualitySmoke(page, viewport) {
  await page.waitForSelector("[data-testid='approved-workbench-root']", { state: "visible", timeout: 30_000 });
  await page.waitForFunction(() => {
    const root = document.documentElement;
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    return root.scrollWidth <= window.innerWidth + 2 && canvas?.clientHeight >= 220;
  }, null, { timeout: 30_000 });
  assert(await canvasNonblank(page), `canvas blank at ${viewport.width}x${viewport.height}`);
}

async function waitForCapabilityLinks(page) {
  await page.waitForFunction(() => {
    const edit = String(document.querySelector("[data-testid='project-edit-link']")?.href ?? "");
    const view = String(document.querySelector("[data-testid='project-view-link']")?.href ?? "");
    return edit.includes("sg_role=edit") && edit.includes("sg_cap=") && view.includes("sg_role=view") && view.includes("sg_cap=");
  }, null, { timeout: 30_000 });
}

async function waitServerSaved(page) {
  await page.waitForFunction(() => {
    const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snapshot?.persistenceStatus?.server_saved === true &&
      Boolean(snapshot?.serverProject?.headRevisionId || snapshot?.serverProject?.head_revision_id);
  }, null, { timeout: 30_000 });
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

async function waitForWorkbenchApi(page, label) {
  await page.waitForFunction(() => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    if (typeof api?.getSnapshot !== "function") return false;
    try {
      const snap = api.getSnapshot();
      return Array.isArray(snap?.runs) && Array.isArray(snap?.runArchives) && Boolean(snap?.serverProject);
    } catch (_) {
      return false;
    }
  }, null, { timeout: 30_000 }).catch(async (err) => {
    const snap = await page.evaluate((labelText) => ({
      label: labelText,
      api_type: typeof window.__SEAMGRIM_APPROVED_WORKBENCH__,
      get_snapshot_type: typeof window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot,
      root_visible: Boolean(document.querySelector("[data-testid='approved-workbench-root']")),
      source_visible: Boolean(document.querySelector("[data-testid='source-editor']")),
      current_url: location.href.replace(/sg_cap=[^&]+/g, "sg_cap=<redacted>"),
      archive_count: (() => {
        try {
          const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
          const raw = typeof api?.getSnapshot === "function" ? api.getSnapshot() : {};
          return Array.isArray(raw.runArchives) ? raw.runArchives.length : 0;
        } catch (_) {
          return 0;
        }
      })(),
    }), label);
    throw new Error(`${label}: workbench API did not settle: ${err.message}: ${JSON.stringify(snap)}`);
  });
}

async function waitForArchiveCount(page, expected, label) {
  await page.waitForFunction((count) => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return (snap?.runArchives?.length || 0) >= count;
  }, expected, { timeout: 45_000 }).catch(async (err) => {
    const snap = await page.evaluate((labelText) => {
      const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
      const raw = typeof api?.getSnapshot === "function" ? api.getSnapshot() : {};
      return {
        label: labelText,
        api_ready: typeof api?.getSnapshot === "function",
        project_id: raw.serverProject?.project_id || raw.serverProject?.projectId || null,
        run_count: Array.isArray(raw.runs) ? raw.runs.length : 0,
        archive_count: Array.isArray(raw.runArchives) ? raw.runArchives.length : 0,
        run_statuses: Array.isArray(raw.runs)
          ? raw.runs.map((run) => ({
              server_run_id: run.server_run_id || null,
              final_status: run.server_final_status || null,
              checkpoint_count: run.server_checkpoint_count || 0,
            }))
          : [],
        current_url: location.href.replace(/sg_cap=[^&]+/g, "sg_cap=<redacted>"),
      };
    }, label);
    throw new Error(`${label}: run archives did not reach ${expected}: ${err.message}: ${JSON.stringify(snap)}`);
  });
}

async function driveRunToDurableBoundary(page, label) {
  await page.click("[data-testid='lifecycle-run']");
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    const run = snap?.runs?.[0];
    const pause = document.querySelector("[data-testid='lifecycle-run'][data-action='pause']");
    const step = document.querySelector("[data-testid='lifecycle-step']");
    return Number(run?.server_checkpoint_count || 0) >= 1 ||
      Boolean(run?.server_final_status && run.server_final_status !== "running") ||
      pause?.disabled === false ||
      step?.disabled === false;
  }, null, { timeout: 45_000 });
  const controls = await page.evaluate(() => ({
    canPause: document.querySelector("[data-testid='lifecycle-run'][data-action='pause']")?.disabled === false,
    canStep: document.querySelector("[data-testid='lifecycle-step']")?.disabled === false,
  }));
  if (controls.canPause) {
    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
    await page.waitForFunction(() => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const run = snap?.runs?.[0];
      return document.querySelector("[data-testid='lifecycle-step']")?.disabled === false ||
        Boolean(run?.server_final_status && run.server_final_status !== "running");
    }, null, { timeout: 15_000 });
  }
  const canStep = await page.evaluate(() => document.querySelector("[data-testid='lifecycle-step']")?.disabled === false);
  if (canStep) {
    await page.click("[data-testid='lifecycle-step']");
    await page.waitForFunction(() => {
      const run = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runs?.[0];
      return Number(run?.server_checkpoint_count || 0) >= 2 ||
        Boolean(run?.server_final_status && run.server_final_status !== "running");
    }, null, { timeout: 45_000 });
  }
  await page.waitForFunction(() => {
    const run = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runs?.[0];
    return Number(run?.server_checkpoint_count || 0) >= 1 ||
      Boolean(run?.server_final_status && run.server_final_status !== "running");
  }, null, { timeout: 45_000 }).catch(async (err) => {
    const snap = await page.evaluate((labelText) => ({
      label: labelText,
      controls: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().controls ?? {},
      runs: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().runs ?? [],
    }), label);
    throw new Error(`${label}: run did not reach durable boundary: ${err.message}: ${JSON.stringify(snap)}`);
  });
}

async function finishActiveRunIfPossible(page, label) {
  const canStop = await page.evaluate(() => document.querySelector("[data-testid='lifecycle-stop']")?.disabled === false);
  if (canStop) {
    await page.locator("[data-testid='lifecycle-stop']").click({ timeout: 1_000 }).catch(() => {});
  }
  await page.waitForFunction(() => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    const runs = snap?.runs || [];
    const run = runs.find((row) => row.run_id === snap?.activeRunId) || runs[0];
    return Boolean(run?.server_final_status && run.server_final_status !== "running");
  }, null, { timeout: 45_000 }).catch(async (err) => {
    const snap = await page.evaluate((labelText) => ({
      label: labelText,
      controls: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().controls ?? {},
      activeRunId: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().activeRunId ?? "",
      runs: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().runs ?? [],
    }), label);
    throw new Error(`${label}: active run did not finish: ${err.message}: ${JSON.stringify(snap)}`);
  });
}

async function openCatalogProject(page, projectId) {
  await page.click(`[data-testid='project-card-${projectId}']`);
  await page.waitForFunction((expected) => {
    const root = document.querySelector("[data-testid='approved-workbench-root']");
    const save = document.querySelector("[data-testid='project-save']");
    const serverMeta = String(document.querySelector("[data-testid='project-server-meta']")?.textContent ?? "");
    return root?.dataset?.projectId === expected && save?.disabled === false && serverMeta.includes("아직 서버 저장 전");
  }, projectId, { timeout: 30_000 });
}

async function runRepresentativeWorkflow(page, browser, projectId, marker) {
  await openCatalogProject(page, projectId);
  await waitForWorkbenchApi(page, `${projectId}:catalog-open`);
  await page.fill("[data-testid='source-editor']", `설정 { 제목: "${marker}". 마디수: 8. }.\n\n1 보여주기.\n`);
  await page.click("[data-testid='project-save']");
  await waitServerSaved(page);
  await waitForCapabilityLinks(page);
  await waitForWorkbenchApi(page, `${projectId}:after-save`);
  const links = await page.evaluate(() => ({
    edit: String(document.querySelector("[data-testid='project-edit-link']")?.href ?? ""),
    view: String(document.querySelector("[data-testid='project-view-link']")?.href ?? ""),
    server: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject,
  }));
  assert(links.server.project_id, `${projectId}: server project id present`);

  await driveRunToDurableBoundary(page, `${projectId}:primary-run`);
  await finishActiveRunIfPossible(page, `${projectId}:primary-finish`);
  const canBranchFromHistory = await page.evaluate(() =>
    document.querySelector("[data-testid='history-back']")?.disabled === false &&
    document.querySelector("[data-testid='history-branch']")?.disabled === false
  );
  if (canBranchFromHistory) {
    await page.click("[data-testid='history-back']");
    await page.click("[data-testid='history-branch']");
    await page.waitForFunction(() => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const runs = snap?.runs || [];
      const branch = runs.find((run) => run.run_id === snap?.activeRunId);
      return Boolean(branch?.server_run_id) && Number(branch?.server_checkpoint_count || 0) >= 1;
    }, null, { timeout: 45_000 });
    await finishActiveRunIfPossible(page, `${projectId}:branch-finish`);
  } else {
    await driveRunToDurableBoundary(page, `${projectId}:second-run`);
    await finishActiveRunIfPossible(page, `${projectId}:second-finish`);
  }
  await waitForArchiveCount(page, 2, `${projectId}:archives-before-reopen`);

  const viewContext = await browser.newContext({ viewport: { width: 1024, height: 768 } });
  const viewPage = await viewContext.newPage();
  await viewPage.goto(links.view, { waitUntil: "domcontentloaded" });
  await viewPage.waitForSelector("[data-testid='approved-workbench-root']", { state: "visible", timeout: 30_000 });
  await waitForWorkbenchApi(viewPage, `${projectId}:view-open`);
  await viewPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.capabilitySession?.role === "view", null, { timeout: 30_000 });
  assert(await viewPage.evaluate(() => document.querySelector("[data-testid='project-save']")?.disabled === true), `${projectId}: view link save disabled`);
  await openStorageDrawer(viewPage);
  await viewPage.click("[data-testid='project-clone']");
  await viewPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.capabilitySession?.role === "edit", null, { timeout: 30_000 });

  const reopenContext = await browser.newContext({ viewport: { width: 778, height: 912 } });
  const reopened = await reopenContext.newPage();
  await reopened.goto(links.edit, { waitUntil: "domcontentloaded" });
  await reopened.waitForSelector("[data-testid='approved-workbench-root']", { state: "visible", timeout: 30_000 });
  await waitForWorkbenchApi(reopened, `${projectId}:reopen-edit`);
  await waitForArchiveCount(reopened, 2, `${projectId}:reopened-archives`);
  const integrated = await reopened.evaluate(() => ({
    project_id: document.querySelector("[data-testid='approved-workbench-root']")?.dataset?.projectId,
    source: String(document.querySelector("[data-testid='source-editor']")?.value ?? ""),
    archives: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runArchives.length,
    href: location.href,
  }));
  assert(integrated.project_id === projectId, `${projectId}: reopened catalog identity preserved`);
  assert(integrated.source.includes(marker), `${projectId}: reopened source preserved`);
  assert(integrated.archives >= 2, `${projectId}: reopened run lineage preserved`);
  assert(!integrated.href.includes("sg_cap="), `${projectId}: edit link fragment scrubbed`);
  await viewContext.close();
  await reopenContext.close();
  return { project_id: projectId, server_project_id: links.server.project_id, run_archives: integrated.archives };
}

async function main() {
  const { child, baseUrl, dataDir, ready } = await startServer();
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  const representativeResults = [];
  try {
    for (const viewport of VIEWPORTS) {
      const context = await browser.newContext({ viewport, locale: "ko-KR", reducedMotion: "reduce" });
      const page = await context.newPage();
      page.on("console", (msg) => {
        const text = msg.text();
        if (["error", "warning"].includes(msg.type()) && !text.includes("Failed to load resource")) failures.push(`${viewport.width}x${viewport.height}: ${msg.type()}: ${text}`);
      });
      await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "domcontentloaded" });
      await qualitySmoke(page, viewport);

      if (viewport.width === 1440) {
        representativeResults.push(await runRepresentativeWorkflow(page, browser, "rep_phys_pendulum_wave1_v1", "진자 durable workflow"));
        representativeResults.push(await runRepresentativeWorkflow(page, browser, "rep_grid_game_state_drop_v1", "격자 durable workflow"));
      }
      await context.close();
    }
    assert(failures.length === 0, `browser console failures: ${failures.join("\n")}`);
    assert(representativeResults.length === 2, "pendulum and grid representative workflows both ran");
    console.log(JSON.stringify({
      schema: "seamgrim.workbench_durable_full_workflow_viewport.runner_result.v1",
      server_ready_schema: ready?.schema ?? "",
      requested_port: 0,
      actual_bound_port: Number(new URL(baseUrl).port),
      viewports: VIEWPORTS.map((v) => `${v.width}x${v.height}`),
      representative_projects: representativeResults,
      integrated_workflow: true,
      data_dir_redacted: path.basename(dataDir),
    }));
  } finally {
    await browser.close();
    await stopServer(child);
    await fs.rm(dataDir, { recursive: true, force: true });
  }
}

await main();
