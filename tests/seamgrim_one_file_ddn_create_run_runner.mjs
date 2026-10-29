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
const NATIVE_CATALOG_KEY = "seamgrim.workbench.native_project_catalog.v1";
const OK = "seamgrim_one_file_ddn_create_run: ok";

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

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-one-file-"));
  const port = await freePort();
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
    stdio: ["ignore", "pipe", "pipe"],
  });
  const stderr = [];
  child.stdout.on("data", () => {});
  child.stderr.on("data", (chunk) => stderr.push(String(chunk)));
  const baseUrl = `http://127.0.0.1:${port}`;
  try {
    for (let i = 0; i < 90; i += 1) {
      if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}\n${stderr.join("")}`);
      try {
        const response = await fetch(`${baseUrl}/api/health`);
        if (response.ok) return { child, baseUrl, dataDir };
      } catch (_) {
        await new Promise((resolve) => setTimeout(resolve, 100));
      }
    }
    throw new Error(`server health timeout\n${stderr.join("")}`);
  } catch (err) {
    await stopServer(child);
    await fs.rm(dataDir, { recursive: true, force: true });
    throw err;
  }
}

async function workbenchSnapshot(page) {
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.());
}

async function createNativeProject(page, { name, template }) {
  const before = await workbenchSnapshot(page);
  await page.click("[data-testid='new-ddn-project']");
  await page.waitForSelector("[data-testid='project-create-dialog'][open]");
  await page.fill("[data-testid='project-create-name']", name);
  await page.check(`input[name='project-template'][value='${template}']`);
  await page.click("[data-testid='project-create-submit']");
  await page.waitForFunction(
    ({ previousId, expectedTemplate }) => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snapshot?.projectId
        && snapshot.projectId !== previousId
        && snapshot.project?.native_creator === true
        && snapshot.project?.creation_template === expectedTemplate;
    },
    { previousId: before?.projectId ?? "", expectedTemplate: template },
    { timeout: 30_000 },
  );
  return workbenchSnapshot(page);
}

async function main() {
  const { child, baseUrl, dataDir } = await startServer();
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") failures.push(`${msg.type()}: ${msg.text()}`);
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));

    await page.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
    await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()));

    const created = await createNativeProject(page, { name: "직접 만든 한 파일", template: "single" });
    const createdUi = await page.evaluate(() => ({
      singleFile: document.querySelector("[data-testid='approved-workbench-root']")?.dataset.singleFile,
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      fileDrawerHidden: document.querySelector("[data-testid='file-drawer']")?.classList.contains("hidden"),
      fileTabsHidden: document.querySelector("[data-testid='file-tabs']")?.classList.contains("hidden"),
      status: document.querySelector("[data-testid='project-save-status']")?.textContent ?? "",
      diagnostics: document.querySelector("[data-testid='diagnostic-list']")?.textContent ?? "",
    }));
    assert(/^native_[a-z0-9]+_[a-z0-9]+$/u.test(created.projectId), `dynamic native project id missing: ${created.projectId}`);
    assert(created.project.file_count === 1, `native one-file structure mismatch: ${JSON.stringify(created.project)}`);
    assert(created.project.files[0]?.path === "lesson.ddn", `native one-file path mismatch: ${JSON.stringify(created.project)}`);
    assert(created.project.creation_template === "single", `native template mismatch: ${JSON.stringify(created.project)}`);
    assert(createdUi.singleFile === "true", "new native DDN is not single-file");
    assert(createdUi.fileDrawerHidden === true && createdUi.fileTabsHidden === true, "single-file creator should hide module drawer/tabs");
    assert(createdUi.source.includes("직접 만든 한 파일") && createdUi.source.includes("값 보여주기"), "native one-file default source missing");
    assert(createdUi.status.includes("local recovery"), `native one-file recovery status missing: ${createdUi.status}`);
    assert(createdUi.diagnostics.includes("W_NATIVE_PROJECT_CREATED"), `native creation diagnostic missing: ${createdUi.diagnostics}`);

    const catalogAtCreation = await page.evaluate((key) => JSON.parse(localStorage.getItem(key) ?? "[]"), NATIVE_CATALOG_KEY);
    const createdCatalogRow = catalogAtCreation.find((row) => row.id === created.projectId);
    assert(createdCatalogRow?.files?.length === 1, "native catalog did not persist a structural one-file project");
    assert(!createdCatalogRow?.portableSnapshot?.local_package_payload, "single-file project was mislabeled as a multi-file project tree");

    const editedSource = `설정 { 제목: "직접 쓴 한 파일". 마디수: 5. }.\n\n채비 {\n  값: 수 <- (3) 매김 { 범위: -9..9. 간격: 1. }.\n}.\n\n(매마디)마다 {\n  값 보여주기.\n}.`;
    await page.fill("[data-testid='source-editor']", editedSource);
    await page.waitForFunction(() => String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? "").includes("local recovery 저장됨"));
    const edited = await page.evaluate(() => ({
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      status: document.querySelector("[data-testid='project-save-status']")?.textContent ?? "",
    }));
    assert(edited.source.includes("직접 쓴 한 파일"), "edited native one-file source not reflected");
    assert(!edited.status.includes("서버 저장 완료"), `local-only native draft mislabeled as server saved: ${edited.status}`);

    await page.reload({ waitUntil: "networkidle" });
    await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()));
    await page.click("[data-global-entry='my-work']");
    const recoveredItem = page.locator(`[data-testid='my-work-item'][data-project-id='${created.projectId}']`);
    await recoveredItem.waitFor({ state: "visible" });
    await recoveredItem.click();
    await page.waitForFunction(
      (projectId) => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().projectId === projectId,
      created.projectId,
    );
    const recovered = await page.evaluate(() => ({
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      diagnostics: document.querySelector("[data-testid='diagnostic-list']")?.textContent ?? "",
    }));
    assert(recovered.source.includes("직접 쓴 한 파일"), `native local recovery did not restore source: ${JSON.stringify(recovered)}`);
    assert(recovered.snapshot.project.file_count === 1, "reopened native project lost one-file structure");
    assert(recovered.diagnostics.includes("W_NATIVE_PROJECT_REOPENED"), `native reopen diagnostic missing: ${recovered.diagnostics}`);

    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return Boolean(snap?.runs?.[0]?.latest_state_hash)
        || snap?.executionLifecycle?.status === "failed"
        || snap?.diagnostics?.some((diag) => String(diag?.code ?? "").startsWith("E_"));
    }, null, { timeout: 30_000 });
    const runState = await workbenchSnapshot(page);
    assert(runState.projectId === created.projectId, "WASM run left the native one-file project");
    assert(
      runState.runs[0]?.latest_state_hash && runState.runs[0].latest_state_hash !== "-",
      `native one-file WASM state hash missing: ${JSON.stringify({ lifecycle: runState.executionLifecycle, diagnostics: runState.diagnostics, runs: runState.runs })}`,
    );

    await page.fill("[data-testid='source-editor']", "");
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().diagnostics?.some((diag) => diag.code === "E_SOURCE_EMPTY"), null, { timeout: 30_000 });
    const sourceError = await workbenchSnapshot(page);
    assert(sourceError.runtimeStatus === "source-error", `source error status mismatch: ${sourceError.runtimeStatus}`);
    assert(sourceError.diagnostics.some((diag) => diag.code === "E_SOURCE_EMPTY"), "empty native source diagnostic not recorded");

    const second = await createNativeProject(page, { name: "두 번째 한 파일", template: "single" });
    assert(second.projectId !== created.projectId, `new native project id collided: ${second.projectId}`);
    assert(second.project.file_count === 1, "second native one-file project is not structural one-file");
    const multifile = await createNativeProject(page, { name: "함께 있는 물리 프로젝트", template: "physics" });
    assert(multifile.projectId !== created.projectId && multifile.projectId !== second.projectId, "native multi-file project id collided");
    assert(multifile.project.file_count === 3, `native multi-file template regressed: ${JSON.stringify(multifile.project)}`);
    const finalCatalog = await page.evaluate((key) => JSON.parse(localStorage.getItem(key) ?? "[]"), NATIVE_CATALOG_KEY);
    for (const projectId of [created.projectId, second.projectId, multifile.projectId]) {
      assert(finalCatalog.some((row) => row.id === projectId), `native creator catalog lost ${projectId}`);
    }
    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);

    console.log(JSON.stringify({
      schema: "seamgrim.one_file_ddn_create_run.runner_result.v2",
      default_url: `${baseUrl}/`,
      project_id: created.projectId,
      second_project_id: second.projectId,
      multifile_project_id: multifile.projectId,
      latest_state_hash: runState.runs[0].latest_state_hash,
      native_catalog_size: finalCatalog.length,
      data_dir: dataDir,
    }));
    console.log(OK);
    await context.close();
  } finally {
    await browser?.close();
    await stopServer(child);
    await fs.rm(dataDir, { recursive: true, force: true });
  }
}

await main();
