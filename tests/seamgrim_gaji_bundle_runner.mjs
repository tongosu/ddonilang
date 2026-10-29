#!/usr/bin/env node
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import net from "node:net";
import { spawn } from "node:child_process";
import { chromium } from "playwright";

function assert(value, message) { if (!value) throw new Error(message); }
async function installedPackageHash(page) {
  await page.locator("[data-testid='file-list'] .file-row", { hasText: "vendor/나만의힘/artifact.detjson" }).click();
  const artifact = JSON.parse(await page.getByTestId("source-editor").inputValue());
  await page.locator("[data-testid='file-list'] .file-row", { hasText: "app/main.ddn" }).click();
  return artifact.package_sha256;
}
async function freePort() {
  const socket = net.createServer();
  await new Promise((resolve) => socket.listen(0, "127.0.0.1", resolve));
  const port = socket.address().port;
  await new Promise((resolve) => socket.close(resolve));
  return port;
}
async function serve(existingDataDir = null) {
  const port = await freePort();
  const dataDir = existingDataDir || await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-gaji-shelf-"));
  const child = spawn(process.env.PYTHON || "python", ["solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py", "--host", "127.0.0.1", "--port", String(port)], {
    cwd: path.resolve("."), env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir }, stdio: "ignore",
  });
  const url = `http://127.0.0.1:${port}`;
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try { if ((await fetch(`${url}/api/v1/projects/health`)).ok) return { child, url, dataDir }; } catch {}
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("server health timeout");
}
function rebase(link, baseUrl) {
  const parsed = new URL(link);
  return `${baseUrl}${parsed.pathname}${parsed.search}${parsed.hash}`;
}

const viewportMatch = String(process.env.GAJI_VIEWPORT || "1440x900").match(/^(\d+)x(\d+)$/);
assert(viewportMatch, `invalid GAJI_VIEWPORT: ${process.env.GAJI_VIEWPORT}`);
const viewport = { width: Number(viewportMatch[1]), height: Number(viewportMatch[2]) };
let { child, url, dataDir } = await serve();
const browser = await chromium.launch({ headless: true });
try {
  const context = await browser.newContext({ viewport });
  const page = await context.newPage();
  const browserErrors = [];
  page.on("console", (message) => {
    if (["error", "warning"].includes(message.type())) browserErrors.push(`console.${message.type()}: ${message.text()}`);
  });
  page.on("pageerror", (error) => browserErrors.push(`pageerror: ${error.message}`));
  await page.goto(`${url}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
  for (const testid of ["source-editor", "bogae-canvas"]) {
    const box = await page.getByTestId(testid).boundingBox();
    assert(box?.width > 0 && box?.height > 0, `${viewport.width}x${viewport.height}: ${testid} is 0x0`);
  }
  await page.getByTestId("new-ddn-project").click();
  await page.getByTestId("project-create-name").fill("임의 이름 창작자 물리");
  await page.getByTestId("project-create-submit").click();
  const fileListBox = await page.getByTestId("file-list").boundingBox();
  assert(fileListBox?.width > 0 && fileListBox?.height > 0, `${viewport.width}x${viewport.height}: file-list is 0x0 after native project open`);
  await page.locator("[data-testid='file-list'] .file-row", { hasText: "lib/힘.ddn" }).click();
  const before = await page.evaluate(() => localStorage.getItem("seamgrim.local_gotgan_store.v1"));
  await page.getByTestId("gaji-bundle-open").click();
  await page.getByTestId("gaji-bundle-id").fill("나만의힘");
  await page.getByTestId("gaji-bundle-version").fill("1.0.0");
  await page.getByTestId("gaji-bundle-tags").fill("물리, 중력");
  await page.getByTestId("gaji-bundle-plan").click();
  try {
    await page.getByTestId("gaji-bundle-preview").filter({ hasText: "공개 심볼:" }).waitFor();
  } catch (error) {
    const status = await page.getByTestId("gaji-bundle-status").textContent();
    const preview = await page.getByTestId("gaji-bundle-preview").textContent();
    const snapshot = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.());
    throw new Error(`gaji bundle plan did not become ready: status=${status} preview=${preview} snapshot=${JSON.stringify(snapshot)} browser_errors=${JSON.stringify(browserErrors)}`, { cause: error });
  }
  const preview = await page.getByTestId("gaji-bundle-preview").textContent();
  assert(preview.includes("lib/힘.ddn"), "selected module missing from preview");
  assert(preview.includes("content hash:"), "content hash missing from preview");
  assert(preview.includes("project:"), "origin lineage missing from preview");
  assert(await page.getByTestId("gaji-bundle-publish").isEnabled(), "publish not enabled after valid plan");
  await page.getByTestId("gaji-bundle-publish").click();
  await page.getByTestId("gaji-bundle-status").filter({ hasText: "발행 완료" }).waitFor();
  const store = await page.evaluate(() => JSON.parse(localStorage.getItem("seamgrim.local_gotgan_store.v1")));
  assert(store.artifacts.length === 1, "publication was not persisted");
  assert(store.artifacts[0].gaji_id === "나만의힘", "wrong arbitrary gaji id");
  assert(store.artifacts[0].profile === "내", "wrong local gotgan profile");
  assert(store.artifacts[0].files.some((file) => file.path === "ddn/lib/힘.ddn"), "real DDN module bytes missing");
  await page.getByTestId("gaji-bundle-close").click();
  await page.getByTestId("gaji-bundle-open").click();
  await page.getByTestId("gaji-bundle-cancel").click();
  const afterCancel = await page.evaluate(() => localStorage.getItem("seamgrim.local_gotgan_store.v1"));
  assert(afterCancel !== before && JSON.parse(afterCancel).artifacts.length === 1, "cancel changed registry");

  const originalModule = await page.getByTestId("source-editor").inputValue();
  const changedModule = originalModule.replace("9.8", "10.8");
  assert(changedModule !== originalModule, "physics template did not contain replaceable gravity value");
  await page.getByTestId("source-editor").fill(changedModule);
  await page.getByTestId("gaji-bundle-open").click();
  await page.getByTestId("gaji-bundle-id").fill("나만의힘");
  await page.getByTestId("gaji-bundle-version").fill("2.0.0");
  await page.getByTestId("gaji-bundle-tags").fill("물리, 중력");
  await page.getByTestId("gaji-bundle-plan").click();
  await page.getByTestId("gaji-bundle-publish").click();
  await page.getByTestId("gaji-bundle-status").filter({ hasText: "발행 완료" }).waitFor();
  await page.getByTestId("gaji-bundle-close").click();
  const versionStore = await page.evaluate(() => JSON.parse(localStorage.getItem("seamgrim.local_gotgan_store.v1")));
  assert(versionStore.artifacts.length === 2, "retained V1/V2 artifacts missing");

  await page.getByTestId("new-ddn-project").click();
  await page.getByTestId("project-create-name").fill("깨끗한 설치 작업");
  await page.locator("input[name='project-template'][value='empty']").check();
  await page.getByTestId("project-create-submit").click();
  const emptyProjectActivePath = await page.locator("[data-testid='file-list'] .file-row.active").getAttribute("data-file-path");
  assert(emptyProjectActivePath === "app/main.ddn", `new project did not open entry file: ${emptyProjectActivePath}`);
  await page.getByTestId("source-editor").fill(`쓰임 { 힘: "./vendor/나만의힘/ddn/lib/힘". }\n\n설정 { 제목: "설치 실행". 마디수: 4. }.\n\n(매마디)마다 {\n  힘.중력 보여주기.\n}.`);
  await page.getByTestId("gaji-shelf-open").click();
  await page.getByTestId("gaji-shelf-search").fill("중력");
  const result = page.getByRole("option", { name: /나만의힘 1\.0\.0 설치 미리보기/u });
  await result.waitFor();
  await result.click();
  await page.waitForTimeout(1000);
  const shelfStatus = await page.getByTestId("gaji-install-status").textContent();
  assert(!shelfStatus.includes("E_"), `install preview failed: ${shelfStatus}`);
  const installPreview = await page.getByTestId("gaji-install-preview").textContent();
  assert(installPreview.includes("vendor/나만의힘/gaji.toml"), `vendor change missing from install preview: ${installPreview}`);
  assert(installPreview.includes("lock:"), "lock diff missing from install preview");
  await page.getByTestId("gaji-install-apply").click();
  await page.getByTestId("gaji-install-status").filter({ hasText: "설치 완료" }).waitFor();
  await page.getByTestId("gaji-shelf-close").click();
  const installedFiles = await page.locator("[data-testid='file-list'] .file-row").allTextContents();
  assert(installedFiles.some((text) => text.includes("vendor/나만의힘/gaji.toml")), "installed manifest missing from project tree");
  assert(installedFiles.some((text) => text.includes("ddn.lock")), "ddn.lock missing after install");
  await page.locator("[data-testid='file-list'] .file-row", { hasText: "app/main.ddn" }).click();
  await page.getByTestId("lifecycle-run").click();
  await page.waitForTimeout(1500);
  const run = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(run.runtimeStatus === "running", `installed gaji runtime failed: ${JSON.stringify(run.diagnostics)}`);
  assert(run.activeFrame?.state_hash, `installed gaji did not execute: ${JSON.stringify(run.diagnostics)}`);
  const v1StateHash = run.activeFrame.state_hash;
  const installedV1PackageHash = await installedPackageHash(page);
  await page.getByTestId("lifecycle-stop").click();

  await page.getByTestId("gaji-shelf-open").click();
  await page.getByTestId("gaji-shelf-search").fill("중력");
  await page.getByRole("option", { name: /나만의힘 2\.0\.0 설치 미리보기/u }).click();
  await page.getByTestId("gaji-install-status").filter({ hasText: "갱신 미리보기 완료" }).waitFor();
  const updatePreview = await page.getByTestId("gaji-install-preview").textContent();
  assert(updatePreview.includes("버전: 1.0.0 → 2.0.0"), `version diff missing: ${updatePreview}`);
  assert(updatePreview.includes("파일:"), "file diff missing");
  assert(updatePreview.includes("API:"), "API diff missing");
  await page.getByTestId("gaji-install-apply").click();
  await page.getByTestId("gaji-install-status").filter({ hasText: "갱신 완료" }).waitFor();
  await page.getByTestId("gaji-shelf-close").click();
  await page.getByTestId("lifecycle-run").click();
  await page.waitForTimeout(1000);
  const v2Run = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(v2Run.activeFrame?.state_hash, `V2 execution failed: ${JSON.stringify(v2Run)}`);
  const installedV2PackageHash = await installedPackageHash(page);
  assert(installedV2PackageHash !== installedV1PackageHash, "V2 package hash did not change");
  await page.getByTestId("lifecycle-stop").click();

  await page.getByTestId("gaji-shelf-open").click();
  await page.getByTestId("gaji-shelf-search").fill("중력");
  await page.getByRole("option", { name: /나만의힘 1\.0\.0 설치 미리보기/u }).click();
  await page.getByTestId("gaji-install-status").filter({ hasText: "되돌리기 미리보기 완료" }).waitFor();
  await page.getByTestId("gaji-install-apply").click();
  await page.getByTestId("gaji-install-status").filter({ hasText: "갱신 완료" }).waitFor();
  await page.getByTestId("gaji-shelf-close").click();
  await page.getByTestId("lifecycle-run").click();
  await page.waitForTimeout(1000);
  const rollbackRun = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(rollbackRun.activeFrame?.state_hash === v1StateHash, "rollback did not reproduce V1 state hash");
  assert(await installedPackageHash(page) === installedV1PackageHash, "rollback did not reproduce V1 package hash");

  await page.getByTestId("gaji-shelf-open").click();
  await page.getByTestId("gaji-shelf-search").fill("중력");
  await page.getByRole("option", { name: /나만의힘 1\.0\.0 설치 미리보기/u }).click();
  await page.getByTestId("gaji-remix-start").click();
  await page.getByTestId("gaji-remix-id").fill("나만의힘-달");
  const remixOriginal = await page.getByTestId("gaji-remix-source").inputValue();
  await page.getByTestId("gaji-remix-source").fill(remixOriginal.replace("9.8", "1.62"));
  await page.getByTestId("gaji-remix-publish").click();
  await page.waitForTimeout(1000);
  const remixStatus = await page.getByTestId("gaji-install-status").textContent();
  assert(remixStatus.includes("리믹스 발행 완료"), `remix publish failed: ${remixStatus}`);
  const remixPreview = await page.getByTestId("gaji-install-preview").textContent();
  assert(remixPreview.includes("계보: 나만의힘@1.0.0 · remix"), `remix lineage missing: ${remixPreview}`);
  assert(remixPreview.includes("원본 나만의힘@1.0.0:"), "original hash missing from remix diff");
  assert(remixPreview.includes("리믹스 나만의힘-달@1.0.0:"), "remix hash missing from diff");
  const remixStore = await page.evaluate(() => JSON.parse(localStorage.getItem("seamgrim.local_gotgan_store.v1")));
  const originalAfterRemix = remixStore.artifacts.find((item) => item.gaji_id === "나만의힘" && item.version === "1.0.0");
  const remixArtifact = remixStore.artifacts.find((item) => item.gaji_id === "나만의힘-달");
  assert(originalAfterRemix.package_sha256 === installedV1PackageHash, "remix mutated original publication");
  assert(remixArtifact?.provenance.origin_gaji === "나만의힘@1.0.0" && remixArtifact.provenance.relation === "remix", "remix artifact lineage invalid");
  await page.getByRole("option", { name: /나만의힘 1\.0\.0 설치 미리보기/u }).click();
  await page.getByTestId("gaji-install-remove").click();
  await page.getByTestId("gaji-install-status").filter({ hasText: "E_GAJI_REMOVE_REFERENCED" }).waitFor();
  const removeError = await page.getByTestId("gaji-install-status").textContent();
  assert(removeError.includes("app/main.ddn"), `referenced remove did not identify file: ${removeError}`);
  await page.getByTestId("gaji-shelf-close").click();
  await page.getByTestId("source-editor").fill('설정 { 제목: "제거 뒤". 마디수: 1. }.\n"가지 제거 완료" 보여주기.');
  await page.getByTestId("gaji-shelf-open").click();
  await page.getByTestId("gaji-shelf-search").fill("중력");
  await page.getByRole("option", { name: /나만의힘 1\.0\.0 설치 미리보기/u }).click();
  await page.getByTestId("gaji-install-remove").click();
  await page.getByTestId("gaji-install-status").filter({ hasText: "제거 완료" }).waitFor();
  await page.getByTestId("gaji-shelf-close").click();
  const removedFiles = await page.locator("[data-testid='file-list'] .file-row").allTextContents();
  assert(!removedFiles.some((text) => text.includes("vendor/나만의힘/")), "unused vendor remained after remove");
  assert(!removedFiles.some((text) => text.includes("ddn.lock")), "lock remained after last gaji remove");

  await page.getByTestId("new-ddn-project").click();
  await page.getByTestId("project-create-name").fill("달 중력 리믹스 소비자");
  await page.locator("input[name='project-template'][value='empty']").check();
  await page.getByTestId("project-create-submit").click();
  await page.getByTestId("source-editor").fill(`쓰임 { 달힘: "./vendor/나만의힘-달/ddn/lib/힘". }\n\n설정 { 제목: "달 리믹스". 마디수: 2. }.\n\n(매마디)마다 {\n  달힘.중력 보여주기.\n}.`);
  await page.getByTestId("gaji-shelf-open").click();
  await page.getByTestId("gaji-shelf-search").fill("나만의힘-달");
  await page.getByRole("option", { name: /나만의힘-달 1\.0\.0 설치 미리보기/u }).click();
  await page.getByTestId("gaji-install-apply").click();
  await page.getByTestId("gaji-install-status").filter({ hasText: "설치 완료" }).waitFor();
  await page.getByTestId("gaji-shelf-close").click();
  await page.getByTestId("lifecycle-run").click();
  await page.waitForTimeout(1000);
  const remixRun = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(remixRun.activeFrame?.state_hash, `remix consumer execution failed: ${JSON.stringify(remixRun.diagnostics)}`);
  await page.getByTestId("project-save").click();
  await page.waitForTimeout(1000);
  const saved = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  const rawViewHref = await page.getByTestId("project-view-link").getAttribute("href");
  const saveText = await page.getByTestId("project-save-status").textContent();
  assert(rawViewHref && rawViewHref !== "#", `server save did not create view link: ${saveText} ${JSON.stringify(saved.serverProject)} ${JSON.stringify(saved.diagnostics)}`);
  const viewUrl = await page.getByTestId("project-view-link").evaluate((node) => node.href);
  const portableDownloadPromise = page.waitForEvent("download");
  await page.getByTestId("portable-project-export").click();
  const portableDownload = await portableDownloadPromise;
  const portableFile = path.join(dataDir, "gaji-remix-project.seamgrim-project.json");
  await portableDownload.saveAs(portableFile);
  await page.getByTestId("gaji-shelf-open").click();
  await page.getByTestId("gaji-shelf-search").fill("나만의힘-달");
  await page.getByRole("option", { name: /나만의힘-달 1\.0\.0 설치 미리보기/u }).click();
  const artifactDownloadPromise = page.waitForEvent("download");
  await page.getByTestId("gaji-store-export").click();
  const artifactDownload = await artifactDownloadPromise;
  const artifactFile = path.join(dataDir, "나만의힘-달-1.0.0.gaji.detjson");
  await artifactDownload.saveAs(artifactFile);
  await page.getByTestId("gaji-shelf-close").click();
  const viewPage = await context.newPage();
  await viewPage.goto(viewUrl, { waitUntil: "networkidle" });
  await viewPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().sourceReadOnly === true);
  await viewPage.getByTestId("gaji-shelf-open").click();
  await viewPage.getByTestId("gaji-shelf-search").fill("중력");
  await viewPage.getByRole("option", { name: /나만의힘 1\.0\.0 설치 미리보기/u }).click();
  await viewPage.getByTestId("gaji-install-status").filter({ hasText: "E_GAJI_VIEW_CAPABILITY_WRITE" }).waitFor();
  assert(await viewPage.getByTestId("gaji-install-apply").isDisabled(), "view capability enabled install");
  await viewPage.getByTestId("gaji-shelf-close").click();
  await viewPage.getByTestId("project-clone").click();
  await viewPage.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().sourceReadOnly === false);
  assert(await viewPage.getByTestId("source-editor").isEditable(), "clone did not grant independent edit capability");
  await viewPage.close();

  await context.close();
  if (child.exitCode === null) child.kill();
  await new Promise((resolve) => child.exitCode === null ? child.once("exit", resolve) : resolve());
  ({ child, url } = await serve(dataDir));
  const restartedView = await browser.newPage({ viewport: { width: 778, height: 912 } });
  await restartedView.goto(rebase(viewUrl, url), { waitUntil: "networkidle" });
  await restartedView.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().sourceReadOnly === true);
  const restartedFiles = await restartedView.locator("[data-testid='file-list'] .file-row").allTextContents();
  assert(restartedFiles.some((text) => text.includes("vendor/나만의힘-달/artifact.detjson")), "server restart lost embedded remix bytes/lineage");
  await restartedView.close();

  const cleanContext = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const imported = await cleanContext.newPage();
  await imported.goto(`${url}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
  assert(await imported.evaluate(() => localStorage.getItem("seamgrim.local_gotgan_store.v1")) === null, "clean context inherited local gotgan");
  await imported.getByTestId("gaji-shelf-open").click();
  await imported.setInputFiles("[data-testid='gaji-store-import-input']", artifactFile);
  await imported.getByTestId("gaji-install-status").filter({ hasText: "artifact 가져오기 완료" }).waitFor();
  await imported.getByTestId("gaji-shelf-close").click();
  await imported.setInputFiles("[data-testid='portable-project-import-input']", portableFile);
  await imported.waitForFunction(() => document.querySelector("[data-testid='portable-project-import-preview']")?.dataset.state === "ready");
  await imported.getByTestId("portable-project-open").click();
  const importedFiles = await imported.locator("[data-testid='file-list'] .file-row").allTextContents();
  assert(importedFiles.some((text) => text.includes("vendor/나만의힘-달/artifact.detjson")), "portable import lost embedded artifact lineage");
  await imported.getByTestId("lifecycle-run").click();
  await imported.waitForTimeout(1000);
  const importedRun = await imported.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(importedRun.activeFrame?.state_hash === remixRun.activeFrame.state_hash, "clean portable replay did not preserve runtime hash");
  await cleanContext.close();
  console.log(`seamgrim_gaji_bundle ${viewport.width}x${viewport.height}: ok`);
} finally {
  await browser.close();
  if (child.exitCode === null) child.kill();
  await new Promise((resolve) => child.exitCode === null ? child.once("exit", resolve) : resolve());
}
