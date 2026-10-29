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

function parseCapabilityLink(link) {
  const url = new URL(link);
  const params = new URLSearchParams(url.hash.replace(/^#/, ""));
  return {
    project_id: String(params.get("sg_project") ?? ""),
    role: String(params.get("sg_role") ?? ""),
    capability: String(params.get("sg_cap") ?? ""),
  };
}

async function fetchJson(baseUrl, pathName, capability) {
  const response = await fetch(`${baseUrl}${pathName}`, {
    headers: { Authorization: `Bearer ${capability}` },
  });
  const payload = await response.json();
  if (!response.ok) throw new Error(`fetch failed ${response.status}: ${JSON.stringify(payload)}`);
  return payload;
}

async function waitForRevisionCount(baseUrl, projectId, capability, minCount) {
  for (let index = 0; index < 60; index += 1) {
    const payload = await fetchJson(baseUrl, `/api/v1/projects/${projectId}/revisions`, capability);
    const revisions = Array.isArray(payload.revisions) ? payload.revisions : [];
    if (revisions.length >= minCount) return payload;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`revision count timeout: ${minCount}`);
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

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-revision-conflict-"));
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

async function openEditorFromDdn(page, baseUrl, ddnText) {
  const encoded = Buffer.from(ddnText, "utf8").toString("base64");
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&ddn=${encodeURIComponent(encoded)}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.locator("#screen-run [data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
}

async function waitSaved(page) {
  await page.waitForFunction(() => {
    const status = document.querySelector("#project-save-status");
    return status?.dataset?.state === "saved" && String(status.textContent ?? "").includes("저장");
  }, null, { timeout: 30_000 });
}

async function waitSavedAfter(page, previousRevisionId) {
  const previous = String(previousRevisionId ?? "");
  await page.waitForFunction((oldRevision) => {
    const status = document.querySelector("#project-save-status");
    const text = String(status?.textContent ?? "");
    return status?.dataset?.state === "saved" && text.includes("저장") && (!oldRevision || !text.includes(oldRevision));
  }, previous, { timeout: 30_000 });
}

async function main() {
  const { child, baseUrl, dataDir } = await startServer();
  const browser = await chromium.launch({ headless: true });
  try {
    const initialDdn = `설정 {
  제목: 리비전_충돌_복원.
}.

1 보여주기.
2 보여주기.
`;
    const contextA = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const pageA = await contextA.newPage();
    await openEditorFromDdn(pageA, baseUrl, initialDdn);
    await pageA.fill("#project-name-input", "리비전 충돌 복원");
    await pageA.click("#btn-project-save");
    await waitSaved(pageA);
    await pageA.waitForFunction(() => document.querySelectorAll("[data-project-revision-id]").length === 1, null, { timeout: 20_000 });
    const firstSave = await pageA.evaluate(() => ({
      edit_link: String(document.querySelector("#project-edit-link")?.href ?? ""),
      first_revision: JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}"),
    }));
    const editCap = parseCapabilityLink(firstSave.edit_link);
    const revisionsInitial = await fetchJson(baseUrl, `/api/v1/projects/${editCap.project_id}/revisions`, editCap.capability);
    const firstRevision = revisionsInitial.revisions[0].revision_id;

    const contextB = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const pageB = await contextB.newPage();
    await pageB.goto(firstSave.edit_link, { waitUntil: "domcontentloaded" });
    await pageB.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
    await pageB.waitForFunction(() => String(document.querySelector("#project-save-status")?.textContent ?? "").includes("다시 열림"), null, { timeout: 30_000 });

    const ddnA = `${initialDdn}\n10 보여주기.\n`;
    await pageA.fill("#ddn-textarea", ddnA);
    await pageA.click("#btn-project-save");
    await waitSavedAfter(pageA, firstRevision);
    await pageA.waitForFunction(() => document.querySelectorAll("[data-project-revision-id]").length >= 2, null, { timeout: 20_000 });
    const revisionsAfterA = await waitForRevisionCount(baseUrl, editCap.project_id, editCap.capability, 2);
    const secondRevision = revisionsAfterA.revisions[revisionsAfterA.revisions.length - 1].revision_id;

    const ddnB = `${initialDdn}\n20 보여주기.\n`;
    await pageB.fill("#ddn-textarea", ddnB);
    await pageB.click("#btn-project-save");
    await pageB.waitForFunction(() => {
      const status = document.querySelector("#project-save-status");
      return status?.dataset?.state === "conflict" && !document.querySelector("#project-conflict-panel")?.classList.contains("hidden");
    }, null, { timeout: 30_000 });
    await pageB.waitForFunction((expectedHead) => {
      return String(document.querySelector("#project-conflict-server")?.textContent ?? "").includes(expectedHead);
    }, secondRevision, { timeout: 20_000 });
    await pageB.waitForFunction(() => document.querySelectorAll("[data-project-revision-id]").length >= 2, null, { timeout: 20_000 });
    const conflictState = await pageB.evaluate(() => ({
      status: String(document.querySelector("#project-save-status")?.textContent ?? ""),
      server: String(document.querySelector("#project-conflict-server")?.textContent ?? ""),
      local: String(document.querySelector("#project-conflict-local")?.textContent ?? ""),
      local_package: String(document.querySelector("#project-conflict-local-package")?.value ?? ""),
      revision_rows: document.querySelectorAll("[data-project-revision-id]").length,
    }));
    assert(conflictState.server.includes(secondRevision), "conflict panel shows server head");
    assert(conflictState.local_package.includes("20 보여주기."), "conflict panel preserves local package");
    assert(conflictState.revision_rows >= 2, "conflict page lists server revisions");
    await pageB.click("#btn-project-conflict-copy-local");
    await pageB.waitForFunction(() => String(document.querySelector("#project-save-status")?.textContent ?? "").includes("보존됨"), null, { timeout: 10_000 });

    const restoreButton = pageA.locator("#project-revision-list [data-project-restore-revision]:not([disabled])").first();
    const restoreSource = await restoreButton.getAttribute("data-project-restore-revision");
    assert(restoreSource === firstRevision, "first non-head revision is the initial revision");
    await restoreButton.click();
    await pageA.waitForFunction(() => {
      const status = document.querySelector("#project-save-status");
      return status?.dataset?.state === "saved" && String(status.textContent ?? "").includes("복원됨");
    }, null, { timeout: 30_000 });
    await pageA.waitForFunction(() => document.querySelectorAll("[data-project-revision-id]").length >= 3, null, { timeout: 20_000 });
    const restoredState = await pageA.evaluate(() => ({
      ddn: String(document.querySelector("#ddn-textarea")?.value ?? ""),
      revision_rows: document.querySelectorAll("[data-project-revision-id]").length,
      list_text: String(document.querySelector("#project-revision-list")?.textContent ?? ""),
    }));
    assert(restoredState.ddn.includes("1 보여주기."), "restored DDN keeps initial body");
    assert(!restoredState.ddn.includes("10 보여주기."), "restored DDN drops later edit");
    assert(restoredState.list_text.includes("parent:"), "revision list audits parent");
    assert(restoredState.list_text.includes("content:"), "revision list audits content sha");
    assert(restoredState.list_text.includes("package:"), "revision list audits package hash");

    const revisionsAfterRestore = await waitForRevisionCount(baseUrl, editCap.project_id, editCap.capability, 3);
    const restoredRevision = revisionsAfterRestore.revisions.find((row) => row.restore_of_revision_id === firstRevision);
    assert(revisionsAfterRestore.revisions.length === 3, "restore creates a third revision");
    assert(
      restoredRevision?.parent_revision_id === secondRevision,
      `restore parent is previous head: parent=${restoredRevision?.parent_revision_id} expected=${secondRevision} all=${JSON.stringify(revisionsAfterRestore.revisions)}`,
    );
    assert(
      restoredRevision?.restore_of_revision_id === firstRevision,
      `restore records source revision: restore_of=${restoredRevision?.restore_of_revision_id} expected=${firstRevision}`,
    );

    console.log(JSON.stringify({
      schema: "seamgrim.project_revision_conflict_restore.runner_result.v1",
      conflict_detected: conflictState.status.includes("다른 창") || conflictState.status.includes("버전"),
      local_package_preserved: conflictState.local_package.includes("20 보여주기."),
      revision_rows_after_conflict: conflictState.revision_rows,
      restore_created_new_revision: revisionsAfterRestore.revisions.length === 3,
      restore_parent_is_previous_head: restoredRevision?.parent_revision_id === secondRevision,
      restore_source_recorded: restoredRevision?.restore_of_revision_id === firstRevision,
      audit_fields_visible: restoredState.list_text.includes("content:") && restoredState.list_text.includes("package:"),
      data_dir: dataDir,
    }));
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

await main();
