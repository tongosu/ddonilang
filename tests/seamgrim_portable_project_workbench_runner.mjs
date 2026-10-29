#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import { buildPortableProjectArtifact } from "../solutions/seamgrim_ui_mvp/ui/workbench/persistence/portable_project_artifact.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const SERVER = path.resolve(ROOT, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");
const OK = "seamgrim_portable_project_workbench: ok";

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

async function startServer(label = "portable-workbench") {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), `seamgrim-${label}-`));
  const port = await freePort();
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", () => {});
  child.stderr.on("data", () => {});
  const baseUrl = `http://127.0.0.1:${port}`;
  for (let index = 0; index < 100; index += 1) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    try {
      const response = await fetch(`${baseUrl}/api/health`);
      if (response.ok) return { child, baseUrl, dataDir };
    } catch (_) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
  throw new Error("server health timeout");
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

function assertNoSecrets(jsonText) {
  for (const token of ["Authorization", "Bearer ", "sg_cap_", "edit_capability", "view_capability", "edit_link", "view_link", "localStorage"]) {
    assert(!jsonText.includes(token), `portable export leaked forbidden token: ${token}`);
  }
}

async function readDownload(download) {
  const suggested = download.suggestedFilename();
  const file = await download.path();
  assert(file, "download path missing");
  return { suggested, text: await fs.readFile(file, "utf8") };
}

async function writeTempJson(prefix, text) {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), `seamgrim-${prefix}-`));
  const file = path.join(dir, `${prefix}.seamgrim-project.json`);
  await fs.writeFile(file, text, "utf8");
  return file;
}

async function openProjectMoreMenu(page) {
  const isOpen = await page.locator("[data-testid='project-more-menu']").evaluate((node) => node.open);
  if (!isOpen) await page.click("[data-testid='project-more-menu'] > summary");
}

async function openPortableProjectSettings(page) {
  await openProjectMoreMenu(page);
  await page.click("[data-testid='portable-project-settings']");
  await page.waitForFunction(() => document.querySelector("[data-testid='portable-project-dialog']")?.open === true);
}

async function openProjectStorageDrawer(page) {
  await openProjectMoreMenu(page);
  await page.click("[data-testid='project-storage-open']");
  await page.waitForFunction(() => document.querySelector("[data-testid='project-storage-drawer']")?.open === true);
}

async function createNativeSingleProject(page, name) {
  const previous = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.projectId ?? "");
  await page.click("[data-testid='new-ddn-project']");
  await page.waitForSelector("[data-testid='project-create-dialog'][open]");
  await page.fill("[data-testid='project-create-name']", name);
  await page.check("input[name='project-template'][value='single']");
  await page.click("[data-testid='project-create-submit']");
  await page.waitForFunction(
    (previousId) => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snapshot?.projectId
        && snapshot.projectId !== previousId
        && snapshot.project?.native_creator === true
        && snapshot.project?.creation_template === "single";
    },
    previous,
    { timeout: 30_000 },
  );
}

async function assertViewportWorkflow(browser, baseUrl, viewport) {
  const context = await browser.newContext({
    viewport,
    locale: "ko-KR",
    acceptDownloads: true,
  });
  const page = await context.newPage();
  await page.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
  await createNativeSingleProject(page, `viewport ${viewport.width}x${viewport.height}`);
  await openPortableProjectSettings(page);
  await page.fill("[data-testid='portable-meta-title']", `viewport ${viewport.width}x${viewport.height}`);
  await page.fill("[data-testid='portable-meta-tags']", `viewport, ${viewport.width}x${viewport.height}`);
  await page.click("[data-testid='portable-project-dialog-close']");
  await page.click("[data-testid='lifecycle-run']");
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().runs?.[0]?.latest_state_hash, null, { timeout: 30_000 });
  const beforeImport = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  const artifact = await page.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
  const file = await writeTempJson(`viewport-${viewport.width}x${viewport.height}`, artifact.file.json_text);
  await page.setInputFiles("[data-testid='portable-project-import-input']", file);
  await page.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "ready");
  const preview = await page.evaluate(() => ({
    runs: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.length,
    title: document.querySelector("[data-testid='portable-project-import-title']")?.textContent ?? "",
  }));
  assert(preview.runs === beforeImport.runs.length, `${viewport.width}x${viewport.height}: import preview auto-ran`);
  assert(preview.title.includes(`viewport ${viewport.width}x${viewport.height}`), `${viewport.width}x${viewport.height}: preview title missing`);
  await page.click("[data-testid='portable-project-open']");
  await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id^='portable_']");
  await page.click("[data-testid='lifecycle-run']");
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().runs?.[0]?.latest_state_hash, null, { timeout: 30_000 });
  const ui = await page.evaluate(() => {
    const rect = (selector) => {
      const node = document.querySelector(selector);
      if (!node) return null;
      const box = node.getBoundingClientRect();
      return { top: box.top, left: box.left, right: box.right, bottom: box.bottom, width: box.width, height: box.height };
    };
    const source = rect("[data-testid='source-panel']");
    const bogae = rect("[data-testid='bogae-panel']");
    const exportButton = document.querySelector("[data-testid='portable-project-export']");
    const importButton = document.querySelector("[data-testid='portable-project-import']");
    return {
      source,
      bogae,
      exportVisible: Boolean(exportButton && exportButton.getBoundingClientRect().width > 20),
      importVisible: Boolean(importButton && importButton.getBoundingClientRect().width > 20),
      exportLabel: exportButton?.getAttribute("aria-label") || exportButton?.getAttribute("title") || "",
      importLabel: importButton?.getAttribute("aria-label") || importButton?.getAttribute("title") || "",
      bodyOverflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    };
  });
  assert(ui.source?.height >= 180, `${viewport.width}x${viewport.height}: source panel too short`);
  assert(ui.bogae?.height >= 180, `${viewport.width}x${viewport.height}: bogae panel too short`);
  assert(ui.exportVisible && ui.importVisible, `${viewport.width}x${viewport.height}: import/export controls hidden`);
  assert(ui.exportLabel && ui.importLabel, `${viewport.width}x${viewport.height}: import/export accessible names missing`);
  assert(ui.bodyOverflowX <= 4, `${viewport.width}x${viewport.height}: horizontal overflow ${ui.bodyOverflowX}`);
  await context.close();
}

async function main() {
  const { child, baseUrl } = await startServer();
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  try {
    const context = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      locale: "ko-KR",
      acceptDownloads: true,
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") failures.push(`${msg.type()}: ${msg.text()}`);
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));

    await page.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
    await page.waitForSelector("[data-testid='portable-project-export']");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.buildCurrentPortableProjectArtifact);

    const first = await page.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
    const second = await page.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
    assert(first.file.json_text === second.file.json_text, "same official state export must be byte-identical");
    assert(first.artifact.schema === "seamgrim.project_snapshot_artifact.v0", "portable export must reuse snapshot artifact schema");
    assert(first.artifact.project_snapshot.ddn_text.includes("포물선"), "official projectile source missing from export");
    assert(
      first.artifact.project_snapshot.controls_snapshot.view_workspace?.schema === "seamgrim.workbench.view_workspace.v1",
      "portable export did not use the new view workspace structure",
    );
    assert(first.artifact.project_snapshot.metadata.content_kinds.includes("simulation"), "content kind metadata missing");
    assert(first.artifact.project_snapshot.metadata.trust_badge === "unverified_local_file", "portable export must not claim verified trust");
    assert(first.file.filename.endsWith(".seamgrim-project.json"), `filename extension mismatch: ${first.file.filename}`);
    assertNoSecrets(first.file.json_text);

    await openPortableProjectSettings(page);
    await page.fill("[data-testid='portable-meta-title']", "포물선 공유본 A");
    await page.fill("[data-testid='portable-meta-summary']", "portable metadata roundtrip");
    await page.selectOption("[data-testid='portable-meta-content-kind']", "lesson");
    await page.fill("[data-testid='portable-meta-tags']", "공유, 포물선, metadata");
    await page.fill("[data-testid='portable-meta-subject']", "physics");
    await page.fill("[data-testid='portable-meta-level']", "middle");
    await page.fill("[data-testid='portable-meta-required-views']", "primary, overlay");
    await page.fill("[data-testid='portable-meta-required-gaji']", "std_physics_1d");
    const metadataExport = await page.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
    assert(metadataExport.file.content_sha256 !== first.file.content_sha256, "metadata edit should change portable content hash");
    assert(metadataExport.artifact.project_snapshot.metadata.title === "포물선 공유본 A", "metadata title missing from export");
    assert(metadataExport.artifact.project_snapshot.metadata.content_kinds[0] === "lesson", "metadata content kind missing from export");
    assert(metadataExport.artifact.project_snapshot.metadata.tags.includes("metadata"), "metadata tags missing from export");
    assert(metadataExport.artifact.project_snapshot.metadata.required_gaji.includes("std_physics_1d"), "required gaji missing from export");
    const legacyOverlayArtifact = await buildPortableProjectArtifact({
      projectName: "기존 overlay 보기 프로젝트",
      ddnText: metadataExport.artifact.project_snapshot.ddn_text,
      sourceLessonId: metadataExport.artifact.project_snapshot.source_lesson_id,
      controlsSnapshot: {
        selected_view: "overlay",
        run_mode: "ddn-main",
      },
      metadata: {
        title: "기존 overlay 보기 프로젝트",
        summary: "legacy selected_view migration fixture",
        tags: ["legacy", "view-migration"],
        content_kinds: ["simulation"],
      },
    });
    assert(!legacyOverlayArtifact.artifact.project_snapshot.controls_snapshot.view_workspace, "legacy fixture unexpectedly contained a modern workspace");

    const before = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.click("[data-testid='portable-project-export-confirm']"),
    ]);
    const saved = await readDownload(download);
    assert(saved.suggested === metadataExport.file.filename, `download filename mismatch: ${saved.suggested} !== ${metadataExport.file.filename}`);
    assert(saved.text === metadataExport.file.json_text, "downloaded bytes differ from product artifact bytes");
    const status = await page.locator("[data-testid='portable-project-export-status']").textContent();
    assert(status.includes(metadataExport.file.content_sha256.slice(0, 12)), `portable status hash missing: ${status}`);
    await page.click("[data-testid='portable-project-dialog-close']");

    const after = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert(before.serverProject.project_id === after.serverProject.project_id, "export must not create server project");
    assert(before.serverProject.revision_count === after.serverProject.revision_count, "export must not append revision");

    const originalSource = await page.locator("[data-testid='source-editor']").inputValue();
    await page.fill("[data-testid='source-editor']", `${originalSource}\n\"portable changed\" 보여주기.`);
    const changed = await page.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
    assert(changed.file.content_sha256 !== metadataExport.file.content_sha256, "source edit should change portable content hash");

    await createNativeSingleProject(page, "portable native 한 파일");
    const localExport = await page.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
    assert(localExport.artifact.project_snapshot.source_lesson_id.startsWith("native_"), "native DDN portable export should use its project id");
    assert(localExport.artifact.project_snapshot.metadata.tags.includes("한 파일"), "native single-file metadata tag missing from export");
    assert(localExport.artifact.project_snapshot.ddn_text.includes("portable native 한 파일"), "native single-file source missing from export");
    assertNoSecrets(localExport.file.json_text);

    const importContext = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      locale: "ko-KR",
      acceptDownloads: true,
    });
    const importPage = await importContext.newPage();
    importPage.on("console", (msg) => {
      if (msg.type() === "error") failures.push(`import ${msg.type()}: ${msg.text()}`);
    });
    importPage.on("pageerror", (err) => failures.push(`import pageerror: ${err.message}`));
    await importPage.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
    await importPage.waitForSelector("[data-testid='portable-project-import-input']");
    const importFile = await writeTempJson("portable-import", metadataExport.file.json_text);
    await importPage.setInputFiles("[data-testid='portable-project-import-input']", importFile);
    await importPage.waitForFunction(() => {
      const preview = document.querySelector("[data-testid='portable-project-import-preview']");
      return preview && !preview.classList.contains("hidden") && preview.dataset.state === "ready";
    });
    const previewState = await importPage.evaluate(() => ({
      title: document.querySelector("[data-testid='portable-project-import-title']")?.textContent ?? "",
      details: document.querySelector("[data-testid='portable-project-import-details']")?.textContent ?? "",
      trust: document.querySelector("[data-testid='portable-project-import-trust']")?.textContent ?? "",
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(previewState.title.includes("포물선 공유본 A"), `preview title mismatch: ${previewState.title}`);
    assert(previewState.details.includes("source"), `preview source length missing: ${previewState.details}`);
    assert(previewState.trust.includes("공식 여부 확인 안 됨"), `trust boundary mismatch: ${previewState.trust}`);
    assert(previewState.snapshot.runs.length === 0, "import preview must not auto-run");
    await importPage.click("[data-testid='portable-project-open']");
    await importPage.waitForSelector("[data-testid='approved-workbench-root'][data-project-id^='portable_']");
    const opened = await importPage.evaluate(() => ({
      projectId: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId ?? "",
      title: document.querySelector("#project-title")?.textContent ?? "",
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(opened.projectId.startsWith("portable_"), `portable project id mismatch: ${opened.projectId}`);
    assert(opened.title.includes("포물선"), `opened title mismatch: ${opened.title}`);
    assert(opened.source.includes("포물선"), "opened source missing imported DDN");
    assert(opened.snapshot.runs.length === 0, "opened import must not auto-run");
    assert(opened.snapshot.persistenceStatus.mode === "portable_import", `portable import status mismatch: ${opened.snapshot.persistenceStatus.mode}`);
    const lineageText = await importPage.locator("[data-testid='portable-lineage-panel']").textContent();
    assert(lineageText.includes(metadataExport.file.content_sha256.slice(0, 12)), `lineage hash missing: ${lineageText}`);
    await openProjectStorageDrawer(importPage);
    await importPage.click("[data-testid='project-clone']");
    await importPage.waitForSelector("[data-testid='approved-workbench-root'][data-project-id*='_clone_']");
    const cloned = await importPage.evaluate(() => ({
      projectId: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId ?? "",
      title: document.querySelector("#project-title")?.textContent ?? "",
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    assert(cloned.projectId.includes("_clone_"), `clone id mismatch: ${cloned.projectId}`);
    assert(cloned.title.includes("내 버전"), `clone title mismatch: ${cloned.title}`);
    assert(cloned.source === opened.source, "clone source changed before edit");
    assert(cloned.snapshot.persistenceStatus.mode === "portable_local_clone", `clone status mismatch: ${cloned.snapshot.persistenceStatus.mode}`);
    assert(cloned.snapshot.portableLineage.origin_kind === "portable_clone", `clone lineage kind mismatch: ${cloned.snapshot.portableLineage.origin_kind}`);
    assert(cloned.snapshot.portableLineage.origin_content_sha256 === metadataExport.file.content_sha256, "clone lost original content hash");
    await importPage.click("[data-testid='project-storage-close']");
    await importPage.waitForFunction(() => document.querySelector("[data-testid='project-storage-drawer']")?.open === false);
    await importPage.fill("[data-testid='source-editor']", `${cloned.source}\n\"clone edit\" 보여주기.`);
    const cloneExport = await importPage.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
    assert(cloneExport.file.content_sha256 !== metadataExport.file.content_sha256, "clone edit should change current hash");
    assert(cloneExport.artifact.project_snapshot.lineage.origin_content_sha256 === metadataExport.file.content_sha256, "clone export lost original hash");
    await importPage.click("[data-testid='global-ddn-search']");
    await importPage.fill("[data-testid='project-search']", "std_physics_1d");
    await importPage.waitForFunction(() => String(document.querySelector("[data-testid='project-list']")?.textContent ?? "").includes("포물선 공유본 A"));
    await importPage.click("[data-testid='global-official-collection']");
    await importPage.waitForFunction(() => !String(document.querySelector("[data-testid='project-list']")?.textContent ?? "").includes("포물선 공유본 A"));
    await importPage.fill("[data-testid='project-search']", "");
    await importPage.click("[data-testid='global-my-work']");
    await importPage.waitForFunction(() => String(document.querySelector("[data-testid='my-work-list']")?.textContent ?? "").includes("포물선 공유본 A"));
    await importPage.click("[data-testid='my-work-item']");
    await importPage.waitForSelector("[data-testid='approved-workbench-root'][data-project-id^='portable_']");
    const reopenedFromMyWork = await importPage.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert(reopenedFromMyWork.persistenceStatus.mode === "portable_catalog_reopen", `my work reopen status mismatch: ${reopenedFromMyWork.persistenceStatus.mode}`);
    await importPage.setInputFiles("[data-testid='portable-project-import-input']", importFile);
    await importPage.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "ready");
    await importPage.click("[data-testid='portable-project-open']");
    await importPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().portableCatalog.filter((row) => row.content_sha256).length >= 2);
    const duplicateCatalog = await importPage.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().portableCatalog);
    assert(duplicateCatalog.filter((row) => row.content_sha256 === metadataExport.file.content_sha256).length >= 2, "duplicate import should be explicit, not silent overwrite");
    const serverArtifact = await importPage.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildProjectArtifact());
    assert(serverArtifact.project_snapshot.lineage.origin_content_sha256 === metadataExport.file.content_sha256, "server artifact lost received origin hash");
    assert(serverArtifact.project_snapshot.metadata.title === "포물선 공유본 A", "server artifact lost portable metadata title");
    await importPage.click("[data-testid='project-save']");
    await importPage.waitForFunction(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return snapshot.serverProject.project_id && snapshot.serverProject.revision_count >= 1;
    });
    const savedReceived = await importPage.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert(savedReceived.serverProject.project_id.startsWith("sgp_"), `received save did not create server project: ${savedReceived.serverProject.project_id}`);
    assert(savedReceived.persistenceStatus.mode === "server_project", `received save status mismatch: ${savedReceived.persistenceStatus.mode}`);
    assert(savedReceived.serverProject.project_id !== "rep_phys_projectile_xy_v1", "received save attempted to append to source lesson id");

    const tamperedText = metadataExport.file.json_text.replace(metadataExport.artifact.content_sha256, "0".repeat(64));
    const tamperedFile = await writeTempJson("portable-tampered", tamperedText);
    const beforeTamper = await importPage.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    await importPage.setInputFiles("[data-testid='portable-project-import-input']", tamperedFile);
    await importPage.waitForFunction(() => {
      const preview = document.querySelector("[data-testid='portable-project-import-preview']");
      return preview && !preview.classList.contains("hidden") && preview.dataset.state === "error";
    });
    const afterTamper = await importPage.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert(afterTamper.projectId === beforeTamper.projectId, "tampered import changed current project");
    assert(afterTamper.runs.length === beforeTamper.runs.length, "tampered import changed run state");
    assert(JSON.stringify(afterTamper.viewWorkspace) === JSON.stringify(beforeTamper.viewWorkspace), "tampered import changed current view workspace");
    assert(afterTamper.activeFrame?.state_hash === beforeTamper.activeFrame?.state_hash, "tampered import changed current world state hash");
    assert(afterTamper.pendingPortableImport.error_code === "E_PORTABLE_SHA_MISMATCH", `tamper diagnostic mismatch: ${afterTamper.pendingPortableImport.error_code}`);

    const oldSchemaArtifact = JSON.parse(metadataExport.file.json_text);
    oldSchemaArtifact.schema = "seamgrim.project_snapshot_artifact.v-1";
    const oldSchemaFile = await writeTempJson("portable-old-schema", JSON.stringify(oldSchemaArtifact));
    const beforeOldSchema = await importPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    await importPage.setInputFiles("[data-testid='portable-project-import-input']", oldSchemaFile);
    await importPage.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "error");
    const afterOldSchema = await importPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    assert(afterOldSchema.snapshot.projectId === beforeOldSchema.snapshot.projectId, "old schema import changed current project");
    assert(afterOldSchema.snapshot.serverProject.project_id === beforeOldSchema.snapshot.serverProject.project_id, "old schema import changed server project");
    assert(afterOldSchema.source === beforeOldSchema.source, "old schema import changed source editor");
    assert(JSON.stringify(afterOldSchema.snapshot.viewWorkspace) === JSON.stringify(beforeOldSchema.snapshot.viewWorkspace), "old schema import changed current view workspace");
    assert(afterOldSchema.snapshot.activeFrame?.state_hash === beforeOldSchema.snapshot.activeFrame?.state_hash, "old schema import changed current world state hash");
    assert(afterOldSchema.snapshot.pendingPortableImport.error_code === "E_PORTABLE_WRAPPER_SCHEMA", `old schema diagnostic mismatch: ${afterOldSchema.snapshot.pendingPortableImport.error_code}`);

    await importPage.setInputFiles("[data-testid='portable-project-import-input']", importFile);
    await importPage.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "ready");
    const beforeCancel = await importPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    await importPage.click("[data-testid='portable-project-cancel']");
    await importPage.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.classList.contains("hidden"));
    const afterCancel = await importPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    assert(afterCancel.snapshot.projectId === beforeCancel.snapshot.projectId, "cancelled import changed current project");
    assert(afterCancel.snapshot.serverProject.project_id === beforeCancel.snapshot.serverProject.project_id, "cancelled import changed server project");
    assert(afterCancel.source === beforeCancel.source, "cancelled import changed source editor");
    assert(JSON.stringify(afterCancel.snapshot.viewWorkspace) === JSON.stringify(beforeCancel.snapshot.viewWorkspace), "cancelled import changed current view workspace");
    assert(afterCancel.snapshot.activeFrame?.state_hash === beforeCancel.snapshot.activeFrame?.state_hash, "cancelled import changed current world state hash");
    assert(afterCancel.snapshot.pendingPortableImport === null, "cancelled import left pending preview");

    const htmlLike = await buildPortableProjectArtifact({
      projectName: "<img src=x onerror=alert(1)>",
      ddnText: "설정 { 제목: \"HTML-like\". }.\n\"html\" 보여주기.\n",
      sourceLessonId: "custom",
      metadata: {
        title: "<img src=x onerror=alert(1)>",
        summary: "<b>not html</b>",
        tags: ["<script>bad</script>"],
      },
    });
    const htmlFile = await writeTempJson("portable-html-title", htmlLike.file.json_text);
    await importPage.setInputFiles("[data-testid='portable-project-import-input']", htmlFile);
    await importPage.waitForFunction(() => {
      const preview = document.querySelector("[data-testid='portable-project-import-preview']");
      return preview && preview.dataset.state === "ready";
    });
    const htmlPreview = await importPage.evaluate(() => ({
      text: document.querySelector("[data-testid='portable-project-import-title']")?.textContent ?? "",
      injected: Boolean(document.querySelector("[data-testid='portable-project-import-title'] img"))
        || Boolean(document.querySelector("[data-testid='portable-project-import-details'] script")),
    }));
    assert(htmlPreview.text.includes("<img"), "HTML-like title should render as text");
    assert(htmlPreview.injected === false, "HTML-like metadata was injected as DOM");
    await importContext.close();

    const legacyContext = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      locale: "ko-KR",
      acceptDownloads: true,
    });
    const legacyPage = await legacyContext.newPage();
    legacyPage.on("console", (msg) => {
      if (msg.type() === "error") failures.push(`legacy ${msg.type()}: ${msg.text()}`);
    });
    legacyPage.on("pageerror", (err) => failures.push(`legacy pageerror: ${err.message}`));
    await legacyPage.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
    await legacyPage.waitForSelector("[data-testid='portable-project-import-input']");
    const beforeLegacyPreview = await legacyPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    const legacyFile = await writeTempJson("portable-legacy-overlay", legacyOverlayArtifact.file.json_text);
    await legacyPage.setInputFiles("[data-testid='portable-project-import-input']", legacyFile);
    await legacyPage.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "ready");
    const afterLegacyPreview = await legacyPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    assert(afterLegacyPreview.snapshot.projectId === beforeLegacyPreview.snapshot.projectId, "legacy preview changed current project before confirmation");
    assert(afterLegacyPreview.source === beforeLegacyPreview.source, "legacy preview changed source before confirmation");
    assert(JSON.stringify(afterLegacyPreview.snapshot.viewWorkspace) === JSON.stringify(beforeLegacyPreview.snapshot.viewWorkspace), "legacy preview changed current view workspace");
    await legacyPage.click("[data-testid='portable-project-open']");
    await legacyPage.waitForSelector("[data-testid='approved-workbench-root'][data-project-id^='portable_']");
    const migratedLegacy = await legacyPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    const migratedRoles = migratedLegacy.snapshot.viewWorkspace.instances.map((instance) => instance.role);
    assert(migratedLegacy.source === legacyOverlayArtifact.artifact.project_snapshot.ddn_text, "legacy migration changed imported DDN source");
    assert(migratedLegacy.snapshot.runs.length === 0 && !migratedLegacy.snapshot.activeFrame, "legacy migration created or changed world state");
    assert(migratedRoles.filter((role) => role === "primary").length === 1, `legacy migration lost primary view: ${JSON.stringify(migratedRoles)}`);
    assert(migratedRoles.filter((role) => role === "overlay").length === 1, `legacy overlay did not become an independent instance: ${JSON.stringify(migratedRoles)}`);
    assert(
      migratedLegacy.snapshot.diagnostics.some((diag) => diag.code === "W_LEGACY_VIEW_SELECTION_MIGRATED"),
      "legacy migration compatibility diagnostic missing",
    );
    const migratedExport = await legacyPage.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
    assert(migratedExport.artifact.project_snapshot.controls_snapshot.view_workspace?.schema === "seamgrim.workbench.view_workspace.v1", "migrated export did not write the new view workspace structure");
    assert(migratedExport.artifact.project_snapshot.controls_snapshot.view_workspace.instances.some((instance) => instance.role === "overlay"), "migrated export lost the independent overlay instance");
    assert(migratedExport.artifact.project_snapshot.controls_snapshot.selected_view === "overlay", "migrated export lost legacy compatibility projection");
    await openProjectStorageDrawer(legacyPage);
    await legacyPage.click("[data-testid='project-clone']");
    await legacyPage.waitForSelector("[data-testid='approved-workbench-root'][data-project-id*='_clone_']");
    const legacyClone = await legacyPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    assert(legacyClone.source === migratedLegacy.source, "legacy-compatible clone changed source");
    assert(legacyClone.snapshot.viewWorkspace.instances.some((instance) => instance.role === "overlay"), "legacy-compatible clone lost migrated overlay instance");
    await legacyContext.close();

    const localFallbackContext = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      locale: "ko-KR",
      acceptDownloads: true,
    });
    const localFallbackPage = await localFallbackContext.newPage();
    localFallbackPage.on("console", (msg) => {
      if (msg.type() === "error") failures.push(`local fallback ${msg.type()}: ${msg.text()}`);
    });
    localFallbackPage.on("pageerror", (err) => failures.push(`local fallback pageerror: ${err.message}`));
    await localFallbackPage.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
    await localFallbackPage.waitForSelector("[data-testid='source-editor']");
    const beforeLocalFallback = await localFallbackPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    await localFallbackPage.evaluate(() => {
      const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
      const snap = api.getSnapshot();
      const source = document.querySelector("[data-testid='source-editor']")?.value ?? "";
      const files = snap.project.files.map((file, index) => ({
        path: file.path,
        role: file.role,
        text: index === 0 ? source : "",
      }));
      const legacySnapshot = {
        schema: "seamgrim.workbench.local_fallback_snapshot.v1",
        saved_scope: "local_device_only",
        server_saved: false,
        project_id: snap.projectId,
        active_file_path: snap.project.files[0]?.path ?? "lesson.ddn",
        mode: "ddn-main",
        view_role: "side",
        recipe_version: 1,
        files,
        runs: [],
      };
      localStorage.setItem(api.projectStorageKey(snap.projectId), JSON.stringify(legacySnapshot));
    });
    await localFallbackPage.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.reopenProjectSnapshot());
    await localFallbackPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics.some((diag) => diag.code === "W_LEGACY_VIEW_SELECTION_MIGRATED"));
    const afterLocalFallback = await localFallbackPage.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
    }));
    assert(afterLocalFallback.snapshot.projectId === beforeLocalFallback.snapshot.projectId, "legacy view_role fallback changed current project");
    assert(afterLocalFallback.source === beforeLocalFallback.source, "legacy view_role fallback changed source");
    assert(afterLocalFallback.snapshot.runs.length === 0 && !afterLocalFallback.snapshot.activeFrame, "legacy view_role fallback changed world state");
    assert(afterLocalFallback.snapshot.viewWorkspace.instances.some((instance) => instance.role === "side"), "legacy view_role fallback did not create an independent side instance");
    await localFallbackContext.close();

    for (const viewport of [
      { width: 1440, height: 900 },
      { width: 1024, height: 768 },
      { width: 778, height: 912 },
      { width: 390, height: 844 },
    ]) {
      await assertViewportWorkflow(browser, baseUrl, viewport);
    }
    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);

    console.log(JSON.stringify({
      schema: "seamgrim.portable_project_workbench.runner_result.v1",
      official_sha: first.file.content_sha256,
      metadata_sha: metadataExport.file.content_sha256,
      changed_sha: changed.file.content_sha256,
      local_sha: localExport.file.content_sha256,
      import_project_id: opened.projectId,
    }));
    console.log(OK);
    await context.close();
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

await main();
