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

async function fetchProject(baseUrl, projectId, capability) {
  const response = await fetch(`${baseUrl}/api/v1/projects/${projectId}`, {
    headers: { Authorization: `Bearer ${capability}` },
  });
  const payload = await response.json();
  if (!response.ok) {
    throw new Error(`project fetch failed ${response.status}: ${JSON.stringify(payload)}`);
  }
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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-share-clone-"));
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

async function main() {
  const { child, baseUrl, dataDir } = await startServer();
  const browser = await chromium.launch({ headless: true });
  try {
    const ddnText = `설정 {
  제목: 공유_복제_실험.
}.

1 보여주기.
2 보여주기.
`;
    const encoded = Buffer.from(ddnText, "utf8").toString("base64");
    const url = `${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&ddn=${encodeURIComponent(encoded)}`;
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await context.newPage();
    await page.goto(url, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
    await page.locator("#screen-run [data-shell-space='create']").click();
    await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
    await page.waitForFunction(() => String(document.querySelector("#ddn-textarea")?.value ?? "").includes("2 보여주기."), null, { timeout: 20_000 });
    await page.fill("#project-name-input", "공유 복제 실험");
    await page.click("#btn-project-save");
    await page.waitForFunction(() => {
      const status = document.querySelector("#project-save-status");
      const view = document.querySelector("#project-view-link");
      return status?.dataset?.state === "saved" && String(view?.href ?? "").includes("sg_role=view");
    }, null, { timeout: 20_000 });
    const firstSave = await page.evaluate(() => ({
      edit_link: String(document.querySelector("#project-edit-link")?.href ?? ""),
      view_link: String(document.querySelector("#project-view-link")?.href ?? ""),
      session_project: JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}"),
    }));
    assert(firstSave.view_link.includes("#sg_project="), "view link uses fragment");
    assert(firstSave.view_link.includes("sg_role=view"), "view link carries view role");
    assert(!firstSave.view_link.includes("?sg_cap="), "view capability must not be query parameter");

    const originalCap = parseCapabilityLink(firstSave.view_link);
    assert(originalCap.project_id && originalCap.role === "view" && originalCap.capability, "view capability parsed");
    const originalBefore = await fetchProject(baseUrl, originalCap.project_id, originalCap.capability);

    const viewContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const viewer = await viewContext.newPage();
    await viewer.goto(firstSave.view_link, { waitUntil: "domcontentloaded" });
    await viewer.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
    await viewer.waitForFunction(() => {
      const text = String(document.querySelector("#ddn-textarea")?.value ?? "");
      const status = String(document.querySelector("#project-save-status")?.textContent ?? "");
      return text.includes("2 보여주기.") && status.includes("다시 열림");
    }, null, { timeout: 30_000 });
    const viewState = await viewer.evaluate(() => ({
      href: location.href,
      ddn_readonly: Boolean(document.querySelector("#ddn-textarea")?.readOnly),
      save_disabled: Boolean(document.querySelector("#btn-project-save")?.disabled),
      edit_disabled: document.querySelector("#project-edit-link")?.getAttribute("aria-disabled") === "true",
      clone_disabled: Boolean(document.querySelector("#btn-project-clone")?.disabled),
      clone_text: String(document.querySelector("#btn-project-clone")?.textContent ?? ""),
      session_project: JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}"),
    }));
    assert(!viewState.href.includes("sg_cap="), "view URL scrubbed after capture");
    assert(viewState.ddn_readonly, "view project DDN is read-only");
    assert(viewState.save_disabled, "view project cannot save original");
    assert(viewState.edit_disabled, "view project does not expose edit link");
    assert(!viewState.clone_disabled, "view project can be cloned");
    assert(viewState.clone_text.includes("내 버전"), "view project clone label");
    assert(viewState.session_project.role === "view", "view session role");

    await viewer.click("#btn-project-clone");
    await viewer.waitForFunction(() => {
      const status = String(document.querySelector("#project-save-status")?.textContent ?? "");
      const session = JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}");
      return status.includes("내 버전 생성") && session.role === "edit";
    }, null, { timeout: 30_000 });
    const cloneState = await viewer.evaluate(() => ({
      clone_response: window.__SEAMGRIM_LAST_PROJECT_CLONE__,
      session_project: JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}"),
      ddn_readonly: Boolean(document.querySelector("#ddn-textarea")?.readOnly),
      save_disabled: Boolean(document.querySelector("#btn-project-save")?.disabled),
      name: String(document.querySelector("#project-name-input")?.value ?? ""),
    }));
    assert(cloneState.session_project.role === "edit", "clone switches to edit session");
    assert(cloneState.session_project.project_id !== originalCap.project_id, "clone uses a new project id");
    assert(!cloneState.ddn_readonly, "clone opens editable DDN");
    assert(!cloneState.save_disabled, "clone can be saved");
    assert(String(cloneState.name).includes("복제본"), "clone name marks copy");

    const updatedDdn = `${ddnText}\n3 보여주기.\n`;
    await viewer.fill("#ddn-textarea", updatedDdn);
    await viewer.click("#btn-project-save");
    await viewer.waitForFunction(() => {
      const status = document.querySelector("#project-save-status");
      return status?.dataset?.state === "saved" && String(status.textContent ?? "").includes("저장됨");
    }, null, { timeout: 30_000 });

    const originalAfter = await fetchProject(baseUrl, originalCap.project_id, originalCap.capability);
    const cloneAfter = await fetchProject(baseUrl, cloneState.session_project.project_id, cloneState.session_project.capability);
    const cloneLineage = cloneAfter.project_snapshot?.lineage ?? {};
    assert(originalAfter.head_revision_id === originalBefore.head_revision_id, "clone save must not move original head");
    assert(originalAfter.content_sha256 === originalBefore.content_sha256, "clone save must not alter original content");
    assert(String(cloneAfter.project_snapshot?.ddn_text ?? "").includes("3 보여주기."), "clone save records clone edit");
    assert(cloneLineage.source_project_id === originalCap.project_id, "clone lineage source project");
    assert(cloneLineage.source_revision_id === originalBefore.head_revision_id, "clone lineage source revision");
    assert(cloneLineage.origin_content_sha256 === originalBefore.content_sha256, "clone lineage source content hash");

    console.log(JSON.stringify({
      schema: "seamgrim.project_share_clone.runner_result.v1",
      view_fragment_role: originalCap.role,
      view_readonly: viewState.ddn_readonly,
      clone_project_distinct: cloneState.session_project.project_id !== originalCap.project_id,
      origin_head_unchanged: originalAfter.head_revision_id === originalBefore.head_revision_id,
      origin_content_unchanged: originalAfter.content_sha256 === originalBefore.content_sha256,
      clone_lineage_recorded: cloneLineage.origin_content_sha256 === originalBefore.content_sha256,
      data_dir: dataDir,
    }));
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

await main();
