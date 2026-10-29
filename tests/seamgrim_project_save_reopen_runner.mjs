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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-save-reopen-"));
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
  제목: 나의_저장_실험.
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
    await page.fill("#project-name-input", "나의 저장 실험");
    await page.click("#btn-project-save");
    await page.waitForFunction(() => {
      const status = document.querySelector("#project-save-status");
      const link = document.querySelector("#project-edit-link");
      return status?.dataset?.state === "saved" && String(link?.href ?? "").includes("sg_cap=");
    }, null, { timeout: 20_000 });
    const firstSave = await page.evaluate(() => {
      const link = document.querySelector("#project-edit-link");
      const status = document.querySelector("#project-save-status");
      const recent = [...document.querySelectorAll("[data-project-recent-item]")].map((node) => ({
        text: String(node.textContent ?? "").trim(),
        href: String(node.href ?? ""),
      }));
      return {
        edit_link: String(link?.href ?? ""),
        status_text: String(status?.textContent ?? ""),
        recent,
        session_project: JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}"),
      };
    });
    assert(firstSave.edit_link.includes("#sg_project="), "edit link uses fragment");
    assert(firstSave.edit_link.includes("sg_cap="), "edit link carries capability fragment");
    assert(!firstSave.edit_link.includes("?sg_cap="), "edit capability must not be query parameter");
    assert(firstSave.recent.length >= 1, "recent work recorded");
    assert(firstSave.session_project.role === "edit", "edit capability stored in session");

    const reopenContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const reopened = await reopenContext.newPage();
    await reopened.goto(firstSave.edit_link, { waitUntil: "domcontentloaded" });
    await reopened.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
    await reopened.waitForFunction(() => {
      const text = String(document.querySelector("#ddn-textarea")?.value ?? "");
      const status = String(document.querySelector("#project-save-status")?.textContent ?? "");
      return text.includes("2 보여주기.") && status.includes("다시 열림");
    }, null, { timeout: 30_000 });
    const reopenedState = await reopened.evaluate(() => ({
      href: location.href,
      ddn: String(document.querySelector("#ddn-textarea")?.value ?? ""),
      lesson_plan: String(document.querySelector("[data-lesson-plan-code]")?.value ?? ""),
      name: String(document.querySelector("#project-name-input")?.value ?? ""),
      status: String(document.querySelector("#project-save-status")?.textContent ?? ""),
      session_project: JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}"),
    }));
    assert(!reopenedState.href.includes("sg_cap="), "reopen should scrub capability fragment from URL");
    assert(reopenedState.ddn.includes("2 보여주기."), "reopened DDN preserved");
    assert(reopenedState.lesson_plan.includes("lesson"), "reopened lesson plan preserved");
    assert(reopenedState.name === "나의 저장 실험", "project name restored");
    assert(reopenedState.session_project.role === "edit", "reopened session role");

    await reopened.locator("#screen-editor [data-shell-space='lab']").click();
    await reopened.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
    await reopened.waitForSelector("#btn-run:not([disabled])", { state: "visible", timeout: 30_000 });
    await reopened.click("#btn-run");
    await reopened.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.textContent ?? "").includes("blake3:"), null, { timeout: 35_000 });
    const runState = await reopened.evaluate(() => ({
      runVisible: !document.querySelector("#screen-run")?.classList?.contains("hidden"),
      hash: String(document.querySelector("#run-mirror-hash")?.textContent ?? ""),
    }));
    assert(runState.runVisible, "run screen visible after reopen");
    assert(runState.hash.includes("blake3:"), "WASM/frontdoor run hash available");

    console.log(JSON.stringify({
      schema: "seamgrim.project_save_reopen.runner_result.v1",
      edit_link_fragment: firstSave.edit_link.split("#")[1],
      reopened_fragment_scrubbed: !reopenedState.href.includes("sg_cap="),
      reopened_ddn_preserved: reopenedState.ddn.includes("2 보여주기."),
      reopened_lesson_plan_preserved: reopenedState.lesson_plan.includes("lesson"),
      run_after_reopen: runState.hash.includes("blake3:"),
      data_dir: dataDir,
    }));
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

await main();
