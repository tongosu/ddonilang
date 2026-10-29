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
const LESSON_ID = "rep_phys_pendulum_wave1_v1";
const SERVER_HEALTH_TIMEOUT_MS = 30_000;
const VIEWPORTS = [
  { label: "1440x900", width: 1440, height: 900 },
  { label: "1280x720", width: 1280, height: 720 },
  { label: "1024x768", width: 1024, height: 768 },
  { label: "390x844", width: 390, height: 844 },
];

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

function rewriteLinkHost(link, baseUrl) {
  const source = new URL(link);
  const target = new URL(baseUrl);
  source.protocol = target.protocol;
  source.host = target.host;
  return source.href;
}

async function fetchJson(baseUrl, pathName, capability) {
  const response = await fetch(`${baseUrl}${pathName}`, {
    headers: { Authorization: `Bearer ${capability}` },
  });
  const payload = await response.json();
  if (!response.ok) throw new Error(`fetch failed ${response.status}: ${JSON.stringify(payload)}`);
  return payload;
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

function textTail(chunks, limit = 1600) {
  return chunks.join("").slice(-limit);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForHealth(baseUrl, child, diagnostics) {
  const deadline = Date.now() + SERVER_HEALTH_TIMEOUT_MS;
  let lastError = "";
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    try {
      const response = await fetch(`${baseUrl}/api/v1/projects/health`);
      if (response.ok) return;
      lastError = `HTTP ${response.status}`;
    } catch (err) {
      lastError = err?.message || String(err);
      await sleep(100);
    }
  }
  throw new Error(`server health timeout ${JSON.stringify({
    base_url: baseUrl,
    pid: child.pid || null,
    exit_code: child.exitCode,
    last_error: lastError,
    stdout_tail: textTail(diagnostics.stdout),
    stderr_tail: textTail(diagnostics.stderr),
  })}`);
}

async function startServer(dataDir) {
  const port = await freePort();
  const env = { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir };
  const diagnostics = { stdout: [], stderr: [] };
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", (chunk) => diagnostics.stdout.push(String(chunk)));
  child.stderr.on("data", (chunk) => diagnostics.stderr.push(String(chunk)));
  const baseUrl = `http://127.0.0.1:${port}`;
  try {
    await waitForHealth(baseUrl, child, diagnostics);
  } catch (err) {
    await stopServer(child);
    throw err;
  }
  return { child, baseUrl, diagnostics };
}

async function waitForChildExit(child, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (child.exitCode === null && isProcessAlive(child.pid) && Date.now() < deadline) {
    await sleep(50);
  }
}

function isProcessAlive(pid) {
  if (!pid) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch (_) {
    return false;
  }
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await waitForChildExit(child, 5_000);
  if (child.exitCode === null && isProcessAlive(child.pid)) {
    child.kill("SIGKILL");
    await waitForChildExit(child, 5_000);
  }
  if (child.exitCode === null && isProcessAlive(child.pid)) {
    throw new Error(`server process did not exit after kill: ${child.pid || "unknown"}`);
  }
}

async function openPendulumEditor(page, baseUrl) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&lesson=${LESSON_ID}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 40_000 });
  await page.locator("#screen-run [data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.waitForFunction(() => String(document.querySelector("#ddn-textarea")?.value ?? "").includes("진자"), null, { timeout: 30_000 });
}

async function editWorldAndLessonPlan(page, label, suffix) {
  const ddn = await page.locator("#ddn-textarea").inputValue();
  const nextDdn = ddn.includes("진자 보기 코어")
    ? ddn.replace("진자 보기 코어", `나의 진자 실험 ${label} ${suffix}`)
    : ddn.replace(/제목:\s*\".*?\"/, `제목: "나의 진자 실험 ${label} ${suffix}"`);
  await page.fill("#ddn-textarea", nextDdn);
  await page.click("#btn-editor-lesson-plan");
  await page.waitForSelector("[data-lesson-plan-code]", { state: "visible", timeout: 20_000 });
  const plan = await page.locator("[data-lesson-plan-code]").inputValue();
  const marker = `wave2e_g8_${label.replace(/x/g, "_")}_${suffix}`;
  const nextPlan = plan.includes('title = "진자 각도와 에너지"')
    ? plan.replace('title = "진자 각도와 에너지"', `title = "진자 각도와 에너지 ${marker}"`)
    : plan.replace(/^title\s*=\s*\".*?\"/m, `title = "진자 각도와 에너지 ${marker}"`);
  await page.fill("[data-lesson-plan-code]", nextPlan);
  await page.click("#btn-editor-world-code");
  await page.waitForSelector("#ddn-textarea:not(.hidden)", { state: "visible", timeout: 20_000 });
  return { nextDdn, marker };
}

async function saveProject(page, name) {
  await page.fill("#project-name-input", name);
  await page.click("#btn-project-save");
  await page.waitForFunction(() => {
    const status = document.querySelector("#project-save-status");
    const edit = document.querySelector("#project-edit-link");
    const view = document.querySelector("#project-view-link");
    return status?.dataset?.state === "saved"
      && String(edit?.href ?? "").includes("sg_role=edit")
      && String(view?.href ?? "").includes("sg_role=view");
  }, null, { timeout: 40_000 });
  return page.evaluate(() => ({
    status: String(document.querySelector("#project-save-status")?.textContent ?? ""),
    edit_link: String(document.querySelector("#project-edit-link")?.href ?? ""),
    view_link: String(document.querySelector("#project-view-link")?.href ?? ""),
    session: JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}"),
  }));
}

async function saveNextRevision(page, previousRevision, label) {
  const ddn = await page.locator("#ddn-textarea").inputValue();
  await page.fill("#ddn-textarea", ddn.replace(`나의 진자 실험 ${label} initial`, `나의 진자 실험 ${label} revision`));
  await page.click("#btn-project-save");
  await page.waitForFunction((oldRevision) => {
    const status = document.querySelector("#project-save-status");
    const text = String(status?.textContent ?? "");
    return status?.dataset?.state === "saved" && text.includes("저장됨") && !text.includes(oldRevision);
  }, previousRevision, { timeout: 40_000 });
}

async function checkEditorQuality(page, viewport) {
  const result = await page.evaluate(() => {
    const viewportWidth = document.documentElement.clientWidth;
    const widthOverflow = document.documentElement.scrollWidth - viewportWidth;
    const textarea = document.querySelector("#ddn-textarea")?.getBoundingClientRect();
    const panel = document.querySelector("[data-project-save-panel]")?.getBoundingClientRect();
    const icon = document.querySelector("#btn-editor-expand");
    return {
      widthOverflow,
      ddnHeight: textarea?.height ?? 0,
      panelWidth: panel?.width ?? 0,
      expandTitle: icon?.getAttribute("title") ?? "",
      expandAria: icon?.getAttribute("aria-label") ?? "",
      saveButtonText: document.querySelector("#btn-project-save")?.textContent ?? "",
    };
  });
  assert(result.widthOverflow <= Math.max(28, Math.round(viewport.width * 0.08)), `${viewport.label} horizontal overflow ${result.widthOverflow}`);
  assert(result.ddnHeight >= 140 || viewport.width <= 430, `${viewport.label} DDN source too short`);
  assert(result.panelWidth > 200, `${viewport.label} project panel too narrow`);
  assert(result.expandTitle && result.expandAria, `${viewport.label} icon tooltip/accessibility missing`);
  assert(String(result.saveButtonText).includes("저장"), `${viewport.label} save command inaccessible`);
  await page.focus("#project-name-input");
  await page.keyboard.press("Tab");
  const active = await page.evaluate(() => document.activeElement?.id ?? "");
  assert(["btn-project-save", "btn-editor-expand"].includes(active), `${viewport.label} keyboard tab did not reach project actions: ${active}`);
}

async function runAndCheckCanvas(page, viewport) {
  await page.click("#btn-run-from-editor");
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 40_000 });
  try {
    await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.textContent ?? "").includes("blake3:"), null, { timeout: 10_000 });
  } catch (_) {
    await page.waitForSelector("#btn-run:not([disabled])", { state: "visible", timeout: 30_000 });
    await page.click("#btn-run");
    await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.textContent ?? "").includes("blake3:"), null, { timeout: 45_000 });
  }
  const before = await page.evaluate(() => {
    const canvas = document.querySelector("#canvas-bogae");
    const box = canvas?.getBoundingClientRect();
    const ctx = canvas?.getContext?.("2d");
    let nonblank = 0;
    if (canvas && ctx && canvas.width && canvas.height) {
      const sample = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
      for (let index = 0; index < sample.length; index += 4) {
        const alpha = sample[index + 3];
        const rgb = sample[index] + sample[index + 1] + sample[index + 2];
        if (alpha > 0 && rgb < 740) nonblank += 1;
      }
    }
    const root = document.querySelector("#screen-run");
    return {
      canvasWidth: box?.width ?? 0,
      canvasHeight: box?.height ?? 0,
      visualHeight: document.querySelector(".run-visual-column")?.getBoundingClientRect()?.height ?? 0,
      nonblank,
      traceFrame: root?.dataset?.traceReactionFrameHash ?? "",
      resolvedSceneTrace: root?.dataset?.traceReactionResolvedSceneTrace ?? "",
      hashText: document.querySelector("#run-mirror-hash")?.textContent ?? "",
      resetTitle: document.querySelector("#btn-reset")?.getAttribute("title") ?? "",
      resetAria: document.querySelector("#btn-reset")?.getAttribute("aria-label") ?? "",
    };
  });
  assert(before.visualHeight >= (viewport.width <= 430 ? 180 : 260), `${viewport.label} main visual too short`);
  assert(before.canvasWidth >= (viewport.width <= 430 ? 240 : 360), `${viewport.label} canvas width too small`);
  assert(before.canvasHeight >= (viewport.width <= 430 ? 160 : 220), `${viewport.label} canvas height too small`);
  assert(before.nonblank > 20, `${viewport.label} canvas blank`);
  assert(before.hashText.includes("blake3:"), `${viewport.label} run hash missing`);
  assert(before.resetTitle && before.resetAria, `${viewport.label} reset tooltip/accessibility missing`);
  await page.click("#btn-reset");
  await page.click("#btn-run");
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.textContent ?? "").includes("blake3:"), null, { timeout: 45_000 });
  const after = await page.evaluate(() => ({
    traceFrame: document.querySelector("#screen-run")?.dataset?.traceReactionFrameHash ?? "",
    resolvedSceneTrace: document.querySelector("#screen-run")?.dataset?.traceReactionResolvedSceneTrace ?? "",
    hashText: document.querySelector("#run-mirror-hash")?.textContent ?? "",
  }));
  if (before.traceFrame || after.traceFrame) {
    assert(before.traceFrame === after.traceFrame, `${viewport.label} resolved scene trace frame changed after replay`);
  }
  assert(after.hashText.includes("blake3:"), `${viewport.label} replay hash missing`);
}

async function reopenProject(page, link, expectedDdn, expectedPlanMarker) {
  await page.goto(link, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 40_000 });
  await page.waitForFunction((marker) => {
    const text = String(document.querySelector("#ddn-textarea")?.value ?? "");
    const status = String(document.querySelector("#project-save-status")?.textContent ?? "");
    return text.includes("나의 진자 실험") && status.includes("다시 열림");
  }, expectedPlanMarker, { timeout: 40_000 });
  await page.click("#btn-editor-lesson-plan");
  await page.waitForSelector("[data-lesson-plan-code]", { state: "visible", timeout: 20_000 });
  const state = await page.evaluate(() => ({
    href: location.href,
    ddn: String(document.querySelector("#ddn-textarea")?.value ?? ""),
    plan: String(document.querySelector("[data-lesson-plan-code]")?.value ?? ""),
    readonly: Boolean(document.querySelector("#ddn-textarea")?.readOnly),
  }));
  assert(!state.href.includes("sg_cap="), "capability fragment not scrubbed");
  assert(state.ddn.includes(expectedDdn), "reopened DDN edit missing");
  assert(state.plan.includes(expectedPlanMarker), "reopened lesson_plan edit missing");
  await page.click("#btn-editor-world-code");
  await page.waitForSelector("#ddn-textarea:not(.hidden)", { state: "visible", timeout: 20_000 });
  return state;
}

async function runViewport(browser, serverRef, dataDir, viewport) {
  const context = await browser.newContext({ viewport: { width: viewport.width, height: viewport.height }, locale: "ko-KR" });
  const page = await context.newPage();
  await openPendulumEditor(page, serverRef.baseUrl);
  const { nextDdn, marker } = await editWorldAndLessonPlan(page, viewport.label, "initial");
  await checkEditorQuality(page, viewport);
  const firstSave = await saveProject(page, "나의 진자 실험");
  const originalEdit = parseCapabilityLink(firstSave.edit_link);
  const originalBefore = await fetchJson(serverRef.baseUrl, `/api/v1/projects/${originalEdit.project_id}`, originalEdit.capability);
  await runAndCheckCanvas(page, viewport);
  await context.close();

  const editContext = await browser.newContext({ viewport: { width: viewport.width, height: viewport.height }, locale: "ko-KR" });
  const editPage = await editContext.newPage();
  await reopenProject(editPage, firstSave.edit_link, `나의 진자 실험 ${viewport.label} initial`, marker);
  await saveNextRevision(editPage, originalBefore.head_revision_id, viewport.label);
  const originalAfterRevision = await fetchJson(serverRef.baseUrl, `/api/v1/projects/${originalEdit.project_id}`, originalEdit.capability);
  assert(originalAfterRevision.head_revision_id !== originalBefore.head_revision_id, `${viewport.label} revision did not advance`);
  const secondEditLink = await editPage.evaluate(() => String(document.querySelector("#project-edit-link")?.href ?? ""));
  const viewLink = firstSave.view_link;

  const viewContext = await browser.newContext({ viewport: { width: viewport.width, height: viewport.height }, locale: "ko-KR" });
  const viewPage = await viewContext.newPage();
  await viewPage.goto(viewLink, { waitUntil: "domcontentloaded" });
  await viewPage.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 40_000 });
  await viewPage.waitForFunction(() => String(document.querySelector("#project-save-status")?.textContent ?? "").includes("다시 열림"), null, { timeout: 40_000 });
  const viewState = await viewPage.evaluate(() => ({
    readonly: Boolean(document.querySelector("#ddn-textarea")?.readOnly),
    saveDisabled: Boolean(document.querySelector("#btn-project-save")?.disabled),
    cloneDisabled: Boolean(document.querySelector("#btn-project-clone")?.disabled),
  }));
  assert(viewState.readonly && viewState.saveDisabled && !viewState.cloneDisabled, `${viewport.label} view permissions invalid`);
  await viewPage.click("#btn-project-clone");
  await viewPage.waitForFunction(() => {
    const status = String(document.querySelector("#project-save-status")?.textContent ?? "");
    const session = JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}");
    return status.includes("내 버전 생성") && session.role === "edit";
  }, null, { timeout: 40_000 });
  const cloneState = await viewPage.evaluate(() => ({
    session: JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}"),
    editLink: String(document.querySelector("#project-edit-link")?.href ?? ""),
  }));
  assert(cloneState.session.project_id !== originalEdit.project_id, `${viewport.label} clone did not create a new project`);
  await viewContext.close();

  await stopServer(serverRef.child);
  Object.assign(serverRef, await startServer(dataDir));
  const restartedOriginalLink = rewriteLinkHost(secondEditLink, serverRef.baseUrl);
  const restartedCloneLink = rewriteLinkHost(cloneState.editLink, serverRef.baseUrl);
  const restartContext = await browser.newContext({ viewport: { width: viewport.width, height: viewport.height }, locale: "ko-KR" });
  const originalRestart = await restartContext.newPage();
  await reopenProject(originalRestart, restartedOriginalLink, `나의 진자 실험 ${viewport.label} revision`, marker);
  const cloneRestart = await restartContext.newPage();
  await cloneRestart.goto(restartedCloneLink, { waitUntil: "domcontentloaded" });
  await cloneRestart.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 40_000 });
  await cloneRestart.waitForFunction(() => String(document.querySelector("#project-save-status")?.textContent ?? "").includes("다시 열림"), null, { timeout: 40_000 });
  const cloneReopen = await cloneRestart.evaluate(() => ({
    projectId: JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}").project_id,
    readonly: Boolean(document.querySelector("#ddn-textarea")?.readOnly),
  }));
  assert(cloneReopen.projectId === cloneState.session.project_id && !cloneReopen.readonly, `${viewport.label} clone did not reopen after restart`);
  await restartContext.close();
  await editContext.close();

  return {
    viewport: viewport.label,
    original_project: originalEdit.project_id,
    clone_project: cloneState.session.project_id,
    revision_advanced: originalAfterRevision.head_revision_id !== originalBefore.head_revision_id,
  };
}

async function main() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-full-workflow-"));
  let serverRef = null;
  let browser = null;
  try {
    serverRef = await startServer(dataDir);
    browser = await chromium.launch({ headless: true });
    const results = [];
    for (const viewport of VIEWPORTS) {
      results.push(await runViewport(browser, serverRef, dataDir, viewport));
    }
    console.log(JSON.stringify({
      schema: "seamgrim.project_full_workflow_viewport.runner_result.v1",
      viewport_count: results.length,
      viewports: results.map((row) => row.viewport),
      revision_advanced_all: results.every((row) => row.revision_advanced),
      clone_distinct_all: results.every((row) => row.original_project !== row.clone_project),
      data_dir: dataDir,
    }));
  } finally {
    if (browser) await browser.close();
    if (serverRef?.child) await stopServer(serverRef.child);
    await fs.rm(dataDir, { recursive: true, force: true });
  }
}

await main();
