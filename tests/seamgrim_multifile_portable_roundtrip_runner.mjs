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
const OK = "seamgrim_multifile_portable_roundtrip: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function base64Utf8(text) {
  return Buffer.from(String(text ?? ""), "utf8").toString("base64");
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
  for (let index = 0; index < 100; index += 1) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    for (const suffix of ["/api/v1/projects/health", "/api/health"]) {
      try {
        const response = await fetch(`${baseUrl}${suffix}`);
        if (response.ok) return;
      } catch (_) {
        // Try again while the Python server starts.
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("server health timeout");
}

async function startServer(existingDataDir = "") {
  const dataDir = existingDataDir || await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-g11-roundtrip-"));
  const port = await freePort();
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
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

async function writeTempJson(prefix, text) {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), `seamgrim-${prefix}-`));
  const file = path.join(dir, `${prefix}.seamgrim-project.json`);
  await fs.writeFile(file, text, "utf8");
  return file;
}

async function openWorkbenchStorageDrawer(page) {
  const alreadyOpen = await page.evaluate(() => document.querySelector("[data-testid='project-storage-drawer']")?.open === true);
  if (alreadyOpen) return;
  await page.evaluate(() => {
    const menu = document.querySelector("[data-testid='project-more-menu']");
    if (menu) menu.open = true;
  });
  await page.click("[data-testid='project-storage-open']");
  await page.waitForFunction(() => document.querySelector("[data-testid='project-storage-drawer']")?.open === true, null, { timeout: 10_000 });
}

async function workbenchProjectRunState(page) {
  return page.evaluate(() => {
    const root = document.querySelector("[data-testid='approved-workbench-root']");
    return {
      executionState: root?.dataset?.projectExecutionState ?? "",
      projectRunBound: root?.dataset?.projectRunBound ?? "",
      runtimeStateHash: root?.dataset?.projectRunRuntimeStateHash ?? "",
      lineageHash: root?.dataset?.projectRunLineageHash ?? "",
      outputRows: String(root?.dataset?.projectRunOutputRows ?? "").split("|").filter(Boolean),
      importedRecordSourceHash: root?.dataset?.importedProjectRecordSourceHash ?? "",
      runStatus: document.querySelector("[data-testid='run-status']")?.textContent ?? "",
    };
  });
}

async function runWorkbenchProjectAndWait(page, expectedRows) {
  await page.click("[data-testid='lifecycle-run']");
  const expected = expectedRows.map(String).join("|");
  await page.waitForFunction((rows) => {
    const root = document.querySelector("[data-testid='approved-workbench-root']");
    return root?.dataset?.projectExecutionState === "current_success"
      && root?.dataset?.projectRunBound === "true"
      && root?.dataset?.projectRunOutputRows === rows;
  }, expected, { timeout: 30_000 });
  return workbenchProjectRunState(page);
}

async function clickProjectFile(page, filePath) {
  await page.locator(`[data-project-file-path="${filePath}"]`).click();
}

async function waitForProjectSummary(page, output = "11") {
  await page.waitForFunction((expected) => {
    const text = document.querySelector("[data-testid='project-run-result']")?.textContent ?? "";
    return text.includes("ddn.local_project_run_summary.v1") && text.includes(`"${expected}"`);
  }, output, { timeout: 30_000 });
  return page.evaluate(() => JSON.parse(document.querySelector("[data-testid='project-run-result']")?.textContent ?? "{}"));
}

function assertLocalTreePayload(payload, { expectedOutput = "11" } = {}) {
  assert(payload?.kind === "seamgrim.local_project_tree.v0", `local payload kind mismatch: ${payload?.kind}`);
  const acceptedFrontdoors = new Set(["wasm_project_run_summary_json", "wasm_project_prepare_link_json"]);
  assert(acceptedFrontdoors.has(payload.frontdoor), `frontdoor mismatch: ${payload.frontdoor}`);
  assert(payload.state_hash_formula_change === false, "state hash formula boundary changed");
  assert(Array.isArray(payload.files) && payload.files.length >= 5, `file payload too small: ${payload.files?.length}`);
  const paths = new Set(payload.files.map((file) => file.path));
  for (const required of ["뿌리.ddn", "app/main.ddn", "lib/helper_renamed.ddn", "ddn.lock", "vendor/edu.physics/gaji.toml"]) {
    assert(paths.has(required), `payload file missing: ${required}`);
  }
  for (const file of payload.files) {
    assert(Number(file.byte_length) >= 0, `byte length missing: ${file.path}`);
    assert(/^[0-9a-f]{64}$/u.test(String(file.sha256 ?? "")), `sha256 missing: ${file.path}`);
  }
  assert(JSON.stringify(payload.output_rows) === JSON.stringify([expectedOutput]), `output rows mismatch: ${JSON.stringify(payload.output_rows)}`);
  for (const key of ["project_graph_sha256", "lock_sha256", "symbol_boundary_sha256", "bounded_state_sha256"]) {
    assert(String(payload[key] ?? "").length >= 32, `${key} missing`);
  }
  const lineage = payload.run_lineage ?? {};
  assert(lineage.schema === "seamgrim.local_project_run_lineage.v0", `run lineage schema mismatch: ${lineage.schema}`);
  for (const key of ["project_graph_hash", "linked_source_hash", "run_request_hash", "final_state_hash", "created_revision", "lineage_hash"]) {
    assert(String(lineage[key] ?? "").length >= 12, `run lineage ${key} missing`);
  }
  assert(lineage.project_graph_hash === payload.project_graph_sha256, "run lineage project graph mismatch");
  assert(lineage.final_state_hash === payload.runtime_state_hash, "run lineage final state mismatch");
}

function rebaseProjectLink(link, baseUrl) {
  const source = new URL(String(link));
  const base = new URL(String(baseUrl));
  return `${base.origin}${source.pathname}${source.search}${source.hash}`;
}

async function buildMainEditorProjectArtifact(page, baseUrl) {
  const ddnText = `설정 {
  제목: G10_휴대형_프로젝트.
}.

1 보여주기.
`;
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&ddn=${encodeURIComponent(base64Utf8(ddnText))}`, {
    waitUntil: "domcontentloaded",
  });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.locator("#screen-run [data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.waitForSelector("#ddn-textarea", { state: "visible", timeout: 10_000 });

  await page.locator("[data-testid='project-enable']").click();
  await page.waitForFunction(() => document.querySelector("[data-project-workbench-panel]")?.dataset?.projectMode === "1");
  await clickProjectFile(page, "lib/helper.ddn");
  await page.locator("[data-testid='project-file-rename']").click();
  await page.waitForSelector("[data-project-file-path='lib/helper_renamed.ddn']", { state: "visible", timeout: 10_000 });
  await clickProjectFile(page, "app/main.ddn");
  await page.fill("#ddn-textarea", `쓰임 {
  도움: "./lib/helper_renamed".
}
도움.값 보여주기.
`);
  await page.locator("[data-testid='project-run']").click();
  const summary = await waitForProjectSummary(page, "11");
  assert(JSON.stringify(summary.output_rows) === JSON.stringify(["11"]), "main UI project run output mismatch");

  const artifact = await page.evaluate(async () => window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: "G10 휴대형 프로젝트" }));
  assert(artifact.schema === "seamgrim.project_snapshot_artifact.v0", `wrapper schema mismatch: ${artifact.schema}`);
  assertLocalTreePayload(artifact.project_snapshot.local_package_payload);
  assert(artifact.project_snapshot.ddn_text.includes("도움.값"), "snapshot ddn_text should preserve entry source");

  await page.fill("#project-name-input", "G10 휴대형 프로젝트");
  await page.click("#btn-project-save");
  await page.waitForFunction(() => {
    const status = document.querySelector("#project-save-status");
    const edit = document.querySelector("#project-edit-link");
    const view = document.querySelector("#project-view-link");
    return status?.dataset?.state === "saved" && String(edit?.href ?? "").includes("sg_cap=") && String(view?.href ?? "").includes("sg_cap=");
  }, null, { timeout: 30_000 });
  const saveState = await page.evaluate(() => ({
    editLink: String(document.querySelector("#project-edit-link")?.href ?? ""),
    viewLink: String(document.querySelector("#project-view-link")?.href ?? ""),
    status: String(document.querySelector("#project-save-status")?.textContent ?? ""),
  }));
  assert(saveState.editLink.includes("#sg_project="), "edit link fragment missing");
  assert(saveState.viewLink.includes("#sg_project="), "view link fragment missing");
  return { artifact, editLink: saveState.editLink, viewLink: saveState.viewLink };
}

async function verifyMainReopenAndClone(browser, editLink, viewLink) {
  const editContext = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
  const editPage = await editContext.newPage();
  await editPage.goto(editLink, { waitUntil: "domcontentloaded" });
  await editPage.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
  await editPage.waitForFunction(() => document.querySelector("[data-project-workbench-panel]")?.dataset?.projectMode === "1", null, { timeout: 30_000 });
  const reopened = await editPage.evaluate(() => ({
    files: [...document.querySelectorAll("[data-project-file-path]")].map((node) => node.getAttribute("data-project-file-path")),
    current: document.querySelector("[data-project-workbench-panel]")?.dataset?.currentPath ?? "",
    text: document.querySelector("#ddn-textarea")?.value ?? "",
  }));
  assert(reopened.files.includes("lib/helper_renamed.ddn"), `reopened files missing helper: ${JSON.stringify(reopened.files)}`);
  assert(reopened.current === "app/main.ddn", `reopened current file mismatch: ${reopened.current}`);
  assert(reopened.text.includes("도움.값"), "reopened entry source missing import call");
  await editPage.locator("[data-testid='project-run']").click();
  await waitForProjectSummary(editPage, "11");
  await editContext.close();

  const viewContext = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
  const viewPage = await viewContext.newPage();
  await viewPage.goto(viewLink, { waitUntil: "domcontentloaded" });
  await viewPage.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
  await viewPage.waitForFunction(() => document.querySelector("[data-project-workbench-panel]")?.dataset?.projectMode === "1", null, { timeout: 30_000 });
  await viewPage.waitForFunction(() => !document.querySelector("#btn-project-clone")?.disabled, null, { timeout: 20_000 });
  await viewPage.click("#btn-project-clone");
  await viewPage.waitForFunction(() => {
    const status = document.querySelector("#project-save-status")?.textContent ?? "";
    return status.includes("내 버전 생성됨");
  }, null, { timeout: 30_000 });
  const cloned = await viewPage.evaluate(async () => {
    const artifact = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: "G10 복제본" });
    return {
      current: document.querySelector("[data-project-workbench-panel]")?.dataset?.currentPath ?? "",
      files: [...document.querySelectorAll("[data-project-file-path]")].map((node) => node.getAttribute("data-project-file-path")),
      payload: artifact.project_snapshot.local_package_payload,
    };
  });
  assert(cloned.current === "app/main.ddn", `clone current file mismatch: ${cloned.current}`);
  assert(cloned.files.includes("lib/helper_renamed.ddn"), "clone lost helper file");
  assertLocalTreePayload(cloned.payload);
  assert(cloned.payload.run_lineage.lineage_hash, "clone lost run lineage hash");
  await viewContext.close();
}

async function verifyWorkbenchPortableRoundtrip(browser, baseUrl, artifact) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR", acceptDownloads: true });
  const page = await context.newPage();
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
  await page.waitForSelector("[data-testid='approved-workbench-root']");
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot);
  const before = await page.evaluate(() => ({
    snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
  }));
  const importFile = await writeTempJson("g10-import", JSON.stringify(artifact, null, 2));
  await page.setInputFiles("[data-testid='portable-project-import-input']", importFile);
  await page.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "ready");
  const preview = await page.evaluate(() => ({
    title: document.querySelector("[data-testid='portable-project-import-title']")?.textContent ?? "",
    details: document.querySelector("[data-testid='portable-project-import-details']")?.textContent ?? "",
    snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
  }));
  assert(preview.title.includes("G10"), `preview title mismatch: ${preview.title}`);
  for (const token of ["entry app/main.ddn", "files", "lock checked", "가지 1"]) {
    assert(preview.details.includes(token), `preview details missing ${token}: ${preview.details}`);
  }
  assert(preview.snapshot.projectId === before.snapshot.projectId, "preview should preserve current project id");
  assert(preview.snapshot.runs.length === before.snapshot.runs.length, "preview should not auto-run");
  await page.click("[data-testid='portable-project-cancel']");
  await page.waitForFunction(() => !document.querySelector("[data-testid='portable-project-import-preview']") || document.querySelector("[data-testid='portable-project-import-preview']")?.classList.contains("hidden"));
  const afterCancel = await page.evaluate(() => ({
    projectId: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId,
    source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
  }));
  assert(afterCancel.projectId === before.snapshot.projectId, "cancel changed project");
  assert(afterCancel.source === before.source, "cancel changed source");

  await page.setInputFiles("[data-testid='portable-project-import-input']", importFile);
  await page.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "ready");
  await page.click("[data-testid='portable-project-open']");
  await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id^='portable_']");
  const opened = await page.evaluate(() => ({
    projectId: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId ?? "",
    files: [...document.querySelectorAll("[data-file-path], [data-testid^='file-row-'], [data-testid^='file-tab-']")].map((node) => node.getAttribute("data-file-path") || node.textContent || ""),
    sourceTitle: document.querySelector("#source-panel-title")?.textContent ?? "",
    source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
  }));
  assert(opened.projectId.startsWith("portable_"), `opened project id mismatch: ${opened.projectId}`);
  assert(opened.sourceTitle.includes("app/main.ddn"), `opened source title mismatch: ${opened.sourceTitle}`);
  assert(opened.source.includes("도움.값"), "opened entry source missing");
  for (const required of ["app/main.ddn", "lib/helper_renamed.ddn", "ddn.lock", "vendor/edu.physics/gaji.toml"]) {
    assert(opened.files.some((text) => String(text).includes(required)), `workbench file missing: ${required}`);
  }
  assert(opened.snapshot.persistenceStatus.mode === "portable_import", `import status mismatch: ${opened.snapshot.persistenceStatus.mode}`);
  const openedRunState = await workbenchProjectRunState(page);
  assert(openedRunState.executionState === "imported_record", `workbench open should start as imported record: ${JSON.stringify(openedRunState)}`);
  assert(openedRunState.projectRunBound === "false", "workbench open should not bind imported record as current run");

  const expectedRows = artifact.project_snapshot.local_package_payload.output_rows.map(String);
  const currentRun = await runWorkbenchProjectAndWait(page, expectedRows);
  assert(currentRun.runtimeStateHash.startsWith("blake3:"), `workbench current run missing state hash: ${JSON.stringify(currentRun)}`);
  assert(currentRun.lineageHash && currentRun.lineageHash !== artifact.project_snapshot.local_package_payload.run_lineage.lineage_hash, "workbench current run should have its own lineage");
  const workbenchArtifact = await page.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
  assertLocalTreePayload(workbenchArtifact.artifact.project_snapshot.local_package_payload);
  assert(
    workbenchArtifact.artifact.project_snapshot.local_package_payload.run_lineage.lineage_hash === currentRun.lineageHash,
    "workbench export did not preserve current run lineage",
  );
  await page.click("[data-testid='project-save']");
  try {
    await page.waitForFunction(() => {
      const status = document.querySelector("[data-testid='project-save-status']")?.textContent ?? "";
      const edit = document.querySelector("[data-testid='project-edit-link']")?.href ?? "";
      return /서버 저장|server saved/u.test(status) && edit.includes("sg_cap=");
    }, null, { timeout: 30_000 });
  } catch (error) {
    const saveDebug = await page.evaluate(() => ({
      status: document.querySelector("[data-testid='project-save-status']")?.textContent ?? "",
      edit: document.querySelector("[data-testid='project-edit-link']")?.href ?? "",
      diagnostics: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().diagnostics ?? [],
    }));
    throw new Error(`workbench save did not complete: ${JSON.stringify(saveDebug)}`);
  }
  const editLink = await page.locator("[data-testid='project-edit-link']").getAttribute("href");
  assert(String(editLink ?? "").includes("#sg_project="), "workbench edit link missing fragment");

  const reopenContext = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
  const reopened = await reopenContext.newPage();
  await reopened.goto(new URL(editLink, baseUrl).href, { waitUntil: "networkidle" });
  await reopened.waitForSelector("[data-testid='approved-workbench-root']");
  await reopened.waitForFunction(() => document.querySelector("#source-panel-title")?.textContent?.includes("app/main.ddn"), null, { timeout: 30_000 });
  const reopenedState = await reopened.evaluate(async () => {
    const artifactAgain = await window.__SEAMGRIM_APPROVED_WORKBENCH__.buildProjectArtifact();
    return {
      title: document.querySelector("#source-panel-title")?.textContent ?? "",
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      files: [...document.querySelectorAll("[data-file-path], [data-testid^='file-row-'], [data-testid^='file-tab-']")].map((node) => node.getAttribute("data-file-path") || node.textContent || ""),
      payload: artifactAgain.project_snapshot.local_package_payload,
    };
  });
  assert(reopenedState.title.includes("app/main.ddn"), `workbench reopen active file mismatch: ${reopenedState.title}`);
  assert(reopenedState.source.includes("도움.값"), "workbench reopen source missing entry text");
  assert(reopenedState.files.some((text) => String(text).includes("lib/helper_renamed.ddn")), "workbench reopen lost helper");
  assertLocalTreePayload(reopenedState.payload);
  assert(reopenedState.payload.run_lineage.lineage_hash === currentRun.lineageHash, "workbench reopen lost current run lineage");
  const reopenedImported = await workbenchProjectRunState(reopened);
  assert(reopenedImported.executionState === "imported_record", `reopened project should start as imported record: ${JSON.stringify(reopenedImported)}`);
  const reopenedRun = await runWorkbenchProjectAndWait(reopened, expectedRows);
  assert(reopenedRun.projectRunBound === "true", "reopened project did not rerun");
  await reopenContext.close();

  await openWorkbenchStorageDrawer(page);
  await page.click("[data-testid='project-clone']");
  await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id*='_clone_']");
  await page.evaluate(() => document.querySelector("[data-testid='project-storage-drawer']")?.close?.());
  const cloned = await page.evaluate(() => ({
    projectId: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId ?? "",
    sourceTitle: document.querySelector("#source-panel-title")?.textContent ?? "",
    source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    executionState: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectExecutionState ?? "",
    importedRecordSourceHash: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.importedProjectRecordSourceHash ?? "",
  }));
  assert(cloned.projectId.includes("_clone_"), `workbench clone id mismatch: ${cloned.projectId}`);
  assert(cloned.sourceTitle.includes("app/main.ddn"), "workbench clone active file mismatch");
  assert(cloned.source.includes("도움.값"), "workbench clone lost entry source");
  assert(cloned.executionState === "imported_record", `workbench clone should start as imported record: ${JSON.stringify(cloned)}`);
  assert(cloned.importedRecordSourceHash, "workbench clone lost imported record source hash");
  const clonedRun = await runWorkbenchProjectAndWait(page, expectedRows);
  assert(clonedRun.projectRunBound === "true", "workbench clone could not run independently");
  assert(clonedRun.lineageHash && clonedRun.lineageHash !== currentRun.lineageHash, "workbench clone did not create independent run lineage");

  const corruptFile = await writeTempJson("g10-corrupt", JSON.stringify({ ...artifact, content_sha256: "0".repeat(64) }));
  const beforeCorrupt = await page.evaluate(() => ({
    projectId: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId,
    source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
  }));
  await page.setInputFiles("[data-testid='portable-project-import-input']", corruptFile);
  await page.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "error");
  const afterCorrupt = await page.evaluate(() => ({
    title: document.querySelector("[data-testid='portable-project-import-title']")?.textContent ?? "",
    projectId: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId,
    source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
  }));
  assert(afterCorrupt.title.includes("가져오기 실패"), `corrupt preview title mismatch: ${afterCorrupt.title}`);
  assert(afterCorrupt.projectId === beforeCorrupt.projectId, "corrupt import changed project");
  assert(afterCorrupt.source === beforeCorrupt.source, "corrupt import changed source");
  const oldWrapperFile = await writeTempJson("g11-old-wrapper", JSON.stringify({
    ...artifact,
    schema: "seamgrim.project_snapshot_artifact.v-1",
  }));
  await page.setInputFiles("[data-testid='portable-project-import-input']", oldWrapperFile);
  await page.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "error");
  const afterOldWrapper = await page.evaluate(() => ({
    title: document.querySelector("[data-testid='portable-project-import-title']")?.textContent ?? "",
    integrity: document.querySelector("[data-testid='portable-project-import-integrity']")?.textContent ?? "",
    projectId: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId,
    source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
  }));
  assert(afterOldWrapper.title.includes("가져오기 실패"), `old wrapper preview title mismatch: ${afterOldWrapper.title}`);
  assert(afterOldWrapper.integrity.includes("E_PORTABLE_WRAPPER_SCHEMA"), `old wrapper diag mismatch: ${afterOldWrapper.integrity}`);
  assert(afterOldWrapper.projectId === beforeCorrupt.projectId, "old wrapper import changed project");
  assert(afterOldWrapper.source === beforeCorrupt.source, "old wrapper import changed source");
  await context.close();
  return { editLink: new URL(editLink, baseUrl).href, currentLineageHash: currentRun.lineageHash };
}

async function verifyServerRestartReopen(browser, baseUrl, mainEditLink, workbenchEditLink, artifact, workbenchCurrentLineageHash) {
  const mainContext = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
  const mainPage = await mainContext.newPage();
  await mainPage.goto(rebaseProjectLink(mainEditLink, baseUrl), { waitUntil: "domcontentloaded" });
  await mainPage.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
  await mainPage.waitForFunction(() => document.querySelector("[data-project-workbench-panel]")?.dataset?.projectMode === "1", null, { timeout: 30_000 });
  const mainState = await mainPage.evaluate(async () => {
    const artifactAgain = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: "G11 restart reopen" });
    return {
      current: document.querySelector("[data-project-workbench-panel]")?.dataset?.currentPath ?? "",
      source: document.querySelector("#ddn-textarea")?.value ?? "",
      payload: artifactAgain.project_snapshot.local_package_payload,
    };
  });
  assert(mainState.current === "app/main.ddn", `restart main current file mismatch: ${mainState.current}`);
  assert(mainState.source.includes("도움.값"), "restart main source missing entry text");
  assertLocalTreePayload(mainState.payload);
  assert(mainState.payload.run_lineage.lineage_hash === artifact.project_snapshot.local_package_payload.run_lineage.lineage_hash, "restart main lost run lineage");
  await mainContext.close();

  const workbenchContext = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
  const workbenchPage = await workbenchContext.newPage();
  await workbenchPage.goto(rebaseProjectLink(workbenchEditLink, baseUrl), { waitUntil: "networkidle" });
  await workbenchPage.waitForSelector("[data-testid='approved-workbench-root']", { state: "visible", timeout: 30_000 });
  await workbenchPage.waitForFunction(() => document.querySelector("#source-panel-title")?.textContent?.includes("app/main.ddn"), null, { timeout: 30_000 });
  const workbenchState = await workbenchPage.evaluate(async () => {
    const artifactAgain = await window.__SEAMGRIM_APPROVED_WORKBENCH__.buildProjectArtifact();
    const root = document.querySelector("[data-testid='approved-workbench-root']");
    return {
      title: document.querySelector("#source-panel-title")?.textContent ?? "",
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      executionState: root?.dataset?.projectExecutionState ?? "",
      projectRunBound: root?.dataset?.projectRunBound ?? "",
      importedRecordSourceHash: root?.dataset?.importedProjectRecordSourceHash ?? "",
      payload: artifactAgain.project_snapshot.local_package_payload,
    };
  });
  assert(workbenchState.title.includes("app/main.ddn"), `restart workbench active file mismatch: ${workbenchState.title}`);
  assert(workbenchState.source.includes("도움.값"), "restart workbench source missing entry text");
  assert(workbenchState.executionState === "imported_record", `restart workbench should start as imported record: ${JSON.stringify(workbenchState)}`);
  assert(workbenchState.projectRunBound === "false", "restart workbench should not bind imported record as current run");
  assert(workbenchState.importedRecordSourceHash, "restart workbench lost imported record source hash");
  assertLocalTreePayload(workbenchState.payload);
  assert(workbenchState.payload.run_lineage.lineage_hash === workbenchCurrentLineageHash, "restart workbench payload lost current run lineage");
  const workbenchRun = await runWorkbenchProjectAndWait(workbenchPage, artifact.project_snapshot.local_package_payload.output_rows.map(String));
  assert(workbenchRun.projectRunBound === "true", "restart workbench did not rerun current project");
  await workbenchContext.close();
}

async function main() {
  let server = await startServer();
  const browser = await chromium.launch({ headless: true });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    const page = await context.newPage();
    const { artifact, editLink, viewLink } = await buildMainEditorProjectArtifact(page, server.baseUrl);
    await verifyMainReopenAndClone(browser, editLink, viewLink);
    const workbenchRoundtrip = await verifyWorkbenchPortableRoundtrip(browser, server.baseUrl, artifact);
    await context.close();
    await stopServer(server.child);
    server = await startServer(server.dataDir);
    await verifyServerRestartReopen(browser, server.baseUrl, editLink, workbenchRoundtrip.editLink, artifact, workbenchRoundtrip.currentLineageHash);
    process.stdout.write(`${OK} ${server.dataDir}\n`);
  } finally {
    await browser.close();
    await stopServer(server?.child);
  }
}

await main();
