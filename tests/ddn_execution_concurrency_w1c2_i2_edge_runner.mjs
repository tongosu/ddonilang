#!/usr/bin/env node

import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { pathToFileURL } from "node:url";

const root = path.resolve(".");
const playwrightModule = path.resolve(String(process.env.DDN_PLAYWRIGHT_MODULE ?? "").trim());
const cli = path.resolve(String(process.env.DDN_W1C2_I2_EDGE_CLI ?? "").trim());
const expectedWasm = String(process.env.DDN_W1C2_I2_EXPECTED_WASM_SHA256 ?? "").trim().toLowerCase();
assert(String(process.env.DDN_PLAYWRIGHT_MODULE ?? "").trim(), "DDN_PLAYWRIGHT_MODULE is required");
assert(String(process.env.DDN_W1C2_I2_EDGE_CLI ?? "").trim(), "DDN_W1C2_I2_EDGE_CLI is required");
assert.equal(expectedWasm.length, 64, "DDN_W1C2_I2_EXPECTED_WASM_SHA256 is required");
const playwright = await import(pathToFileURL(playwrightModule).href);
const chromium = playwright.chromium ?? playwright.default?.chromium;
assert(chromium, "Playwright chromium export is required");

const serverScript = path.join(root, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");
const workbenchPath = "/solutions/seamgrim_ui_mvp/ui/workbench/index.html";
const successSource = await fs.readFile(path.join(
  root,
  "pack/ddn_execution_concurrency_w1c1_sender_route_provider/00_any_sender_builtin_success/input.ddn",
), "utf8");
const unhandledSource = await fs.readFile(path.join(
  root,
  "pack/ddn_execution_concurrency_w1c1_sender_route_provider/02_sender_mismatch_unhandled/input.ddn",
), "utf8");
const ambiguitySource = await fs.readFile(path.join(
  root,
  "pack/ddn_execution_concurrency_w1c1_sender_route_provider/07_duplicate_route_ambiguity/input.ddn",
), "utf8");
const sha256 = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");

async function freePort() {
  const socket = net.createServer();
  await new Promise((resolve, reject) => {
    socket.once("error", reject);
    socket.listen(0, "127.0.0.1", resolve);
  });
  const port = socket.address().port;
  await new Promise((resolve) => socket.close(resolve));
  assert.notEqual(port, 8787, "runner must not use protected port 8787");
  return port;
}

async function startServer(dataDir, port) {
  const child = spawn(process.env.PYTHON || "python", [
    serverScript,
    "--host", "127.0.0.1",
    "--port", String(port),
  ], {
    cwd: root,
    env: {
      ...process.env,
      LOCAL_OFFICIAL_STORE: dataDir,
      SEAMGRIM_PROJECT_DATA_DIR: dataDir,
      DDN_MODEL_TEUL_CLI: cli,
      TEUL_CLI_BIN: cli,
      SEAMGRIM_PRODUCT_HEAD: "242857bb70cfd2f5bb18797210481d97e86f3577",
      PYTHONIOENCODING: "utf-8",
    },
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  let output = "";
  child.stdout.on("data", (chunk) => { output += String(chunk); });
  child.stderr.on("data", (chunk) => { output += String(chunk); });
  const baseUrl = `http://127.0.0.1:${port}`;
  for (let attempt = 0; attempt < 300; attempt += 1) {
    try {
      if ((await fetch(`${baseUrl}/api/health`)).ok) {
        assert.notEqual(child.pid, 53196, "runner must not use protected PID 53196");
        return { child, baseUrl, output: () => output };
      }
    } catch {
      // Retry only the runner-owned server.
    }
    if (child.exitCode !== null) throw new Error(`server exited: ${output}`);
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(`server health timeout: ${output}`);
}

async function stopServer(server) {
  if (!server?.child || server.child.exitCode !== null) return;
  server.child.kill();
  await new Promise((resolve) => server.child.once("exit", resolve));
}

function observePage(page, label, errors, wasmResponses) {
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`${label}:console:${message.text()}`);
  });
  page.on("pageerror", (error) => errors.push(`${label}:pageerror:${error.message}`));
  page.on("requestfailed", (request) => {
    errors.push(`${label}:requestfailed:${request.method()}:${request.url()}:${request.failure()?.errorText ?? ""}`);
  });
  page.on("response", async (response) => {
    if (response.status() >= 400) errors.push(`${label}:http:${response.status()}:${response.url()}`);
    if (response.url().endsWith("/ddonirang_tool_bg.wasm") && response.ok()) {
      wasmResponses.push(sha256(await response.body()));
    }
  });
}

async function snapshot(page) {
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
}

async function replaceSourceAndRun(page, source, expected) {
  const before = await snapshot(page);
  await page.getByTestId("source-editor").fill(source);
  await page.getByTestId("lifecycle-run").click();
  if (expected.kind === "success") {
    try {
      await page.waitForFunction(({ priorRun, value }) => {
        const state = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
        const text = document.querySelector("[data-testid='public-observation-table-surface']")?.textContent ?? "";
        const resources = JSON.stringify(state?.activeFrame?.wasm_state?.resources?.value_json ?? {});
        return Boolean(state?.activeFrame?.state_hash)
          && state.activeRunId !== priorRun
          && (text.includes(value) || resources.includes(value));
      }, { priorRun: before.activeRunId ?? "", value: expected.value }, { timeout: 60_000 });
    } catch (error) {
      const debug = await page.evaluate(() => ({
        snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.(),
        table: document.querySelector("[data-testid='public-observation-table-surface']")?.textContent ?? "",
      }));
      throw new Error(`success settle timeout: ${JSON.stringify(debug)}`, { cause: error });
    }
  } else {
    await page.waitForFunction((code) => {
      const state = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return JSON.stringify(state?.diagnostics ?? []).includes(code);
    }, expected.code, { timeout: 60_000 });
  }
  return snapshot(page);
}

async function saveProject(page) {
  const before = Number((await snapshot(page)).serverProject?.revision_count ?? 0);
  const menu = page.getByTestId("project-more-menu");
  if (!(await menu.evaluate((node) => node.open))) await menu.locator("summary").click();
  await page.getByTestId("project-save").click();
  await page.getByTestId("project-save-dialog").waitFor({ state: "visible" });
  await page.getByTestId("project-save-confirm").click();
  await page.waitForFunction((count) => {
    const state = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return Number(state?.serverProject?.revision_count ?? 0) > count
      && state?.persistenceStatus?.server_saved === true;
  }, before, { timeout: 60_000 });
  return snapshot(page);
}

async function capabilityIdentity(page) {
  return page.evaluate(() => {
    const session = JSON.parse(sessionStorage.getItem("seamgrim.project.capability.v0") || "{}");
    const rows = JSON.parse(localStorage.getItem("seamgrim.workbench.my_work_index.v1") || "[]");
    const row = rows.find((item) => String(item?.project_id ?? "") === String(session?.project_id ?? "")) ?? {};
    return {
      projectId: String(session.project_id ?? ""),
      edit: String(session.role === "edit" ? session.capability ?? "" : row.edit_capability ?? ""),
    };
  });
}

const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "ddn-w1c2-i2-edge-data-"));
const port = await freePort();
let server = await startServer(dataDir, port);
const firstServerPid = server.child.pid;
let browser = null;
const errors = [];
const wasmResponses = [];
let secondServerPid = 0;

try {
  browser = await chromium.launch({ channel: "msedge", headless: true });
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, locale: "ko-KR" });
  const page = await context.newPage();
  observePage(page, "initial", errors, wasmResponses);
  await page.goto(`${server.baseUrl}${workbenchPath}?command=new`, { waitUntil: "networkidle" });
  await page.getByRole("radio", { name: "현재 DDN 시작본" }).click();
  await page.getByTestId("model-template-select").selectOption("canonical-public-observation-order-v1");
  await page.getByTestId("project-create-name").fill("W1-C2 I2 Edge 검증");
  await page.getByTestId("project-create-submit").click();
  await page.waitForFunction(() => Boolean(
    window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.projectId,
  ), null, { timeout: 60_000 });
  const success = await replaceSourceAndRun(page, successSource, { kind: "success", value: "7" });
  assert(success.activeFrame?.state_hash, "success state hash missing");
  assert.equal(await page.locator("canvas").count(), 0, "success must remain table-first");
  const saved = await saveProject(page);
  const identity = await capabilityIdentity(page);
  assert(identity.projectId && identity.edit, "saved edit capability missing");
  const revisionId = saved.serverProject?.head_revision_id;
  assert(revisionId, "saved revision missing");
  await context.close();

  await stopServer(server);
  server = await startServer(dataDir, port);
  secondServerPid = server.child.pid;
  assert.notEqual(secondServerPid, firstServerPid, "runner-owned server PID did not change");

  const reopenedContext = await browser.newContext({ viewport: { width: 1280, height: 800 }, locale: "ko-KR" });
  await reopenedContext.addInitScript(({ projectId, capability }) => {
    sessionStorage.setItem("seamgrim.project.capability.v0", JSON.stringify({
      project_id: projectId,
      role: "edit",
      capability,
    }));
  }, { projectId: identity.projectId, capability: identity.edit });
  const reopened = await reopenedContext.newPage();
  observePage(reopened, "reopen", errors, wasmResponses);
  const url = new URL(workbenchPath, server.baseUrl);
  url.search = new URLSearchParams({ project_id: identity.projectId, shared_revision: revisionId });
  await reopened.goto(url.href, { waitUntil: "networkidle" });
  await reopened.waitForFunction((expected) => (
    document.querySelector("[data-testid='source-editor']")?.value === expected
  ), successSource, { timeout: 60_000 });
  await replaceSourceAndRun(reopened, successSource, { kind: "success", value: "7" });
  const unhandled = await replaceSourceAndRun(reopened, unhandledSource, {
    kind: "failure",
    code: "E_ASYNC_ENDPOINT_UNHANDLED_CANDIDATE",
  });
  assert(
    unhandled.diagnostics.some((diagnostic) => (
      diagnostic.code === "E_ASYNC_ENDPOINT_UNHANDLED_CANDIDATE"
      && diagnostic.severity === "error"
    )),
    `unhandled error was masked: ${JSON.stringify(unhandled.diagnostics)}`,
  );
  const ambiguous = await replaceSourceAndRun(reopened, ambiguitySource, {
    kind: "failure",
    code: "E_ASYNC_ENDPOINT_AMBIGUOUS_CANDIDATE",
  });
  assert(
    ambiguous.diagnostics.some((diagnostic) => (
      diagnostic.code === "E_ASYNC_ENDPOINT_AMBIGUOUS_CANDIDATE"
      && diagnostic.severity === "error"
    )),
    `ambiguity error was masked: ${JSON.stringify(ambiguous.diagnostics)}`,
  );
  assert(!JSON.stringify(ambiguous.diagnostics).includes("E_ASYNC_PROVIDER"), "public provider diagnostic leaked");
  await reopenedContext.close();

  assert(wasmResponses.length >= 2, "tracked WASM browser responses missing");
  assert(wasmResponses.every((hash) => hash === expectedWasm), "browser-served WASM hash mismatch");
  assert.deepEqual(errors, [], `browser console/page/network errors: ${JSON.stringify(errors)}`);

  console.log(JSON.stringify({
    schema: "ddn.execution_concurrency.w1c2.i2.actual_edge.v1",
    status: "PASS",
    browser: "Microsoft Edge",
    success: true,
    unhandled: "E_ASYNC_ENDPOINT_UNHANDLED_CANDIDATE",
    ambiguity: "E_ASYNC_ENDPOINT_AMBIGUOUS_CANDIDATE",
    exact_reopen: true,
    pid_swap: { first: firstServerPid, second: secondServerPid },
    port,
    protected_port_8787_untouched: true,
    wasm_sha256: expectedWasm,
    wasm_response_count: wasmResponses.length,
    console_page_network_errors: errors.length,
    public_provider_ui_or_input: 0,
  }));
} finally {
  if (browser) await browser.close();
  await stopServer(server);
  await fs.rm(dataDir, { recursive: true, force: true });
}
