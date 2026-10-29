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
const OK = "seamgrim_public_single_node_serving: ok";

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

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-single-node-"));
  const port = await freePort();
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", () => {});
  child.stderr.on("data", () => {});
  const baseUrl = `http://127.0.0.1:${port}`;
  for (let i = 0; i < 90; i += 1) {
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

async function fetchJson(url) {
  const response = await fetch(url);
  const payload = await response.json();
  return { status: response.status, payload, text: JSON.stringify(payload) };
}

async function main() {
  const { child, baseUrl, dataDir } = await startServer();
  const browser = await chromium.launch({ headless: true });
  try {
    const health = await fetchJson(`${baseUrl}/api/health`);
    assert(health.status === 200 && health.payload.ok === true, `server health failed: ${health.text}`);
    const projectHealth = await fetchJson(`${baseUrl}/api/v1/projects/health`);
    assert(projectHealth.status === 200 && projectHealth.payload.storage === "ready", `project health failed: ${projectHealth.text}`);
    const readiness = await fetchJson(`${baseUrl}/api/readiness`);
    assert(readiness.status === 200 && readiness.payload.ok === true, `readiness failed: ${readiness.text}`);
    assert(readiness.payload.schema === "seamgrim.public_wedge.readiness.v1", `readiness schema mismatch: ${readiness.text}`);
    assert(readiness.payload.wasm_artifact?.ready === true, `wasm artifact not ready: ${readiness.text}`);
    assert(readiness.payload.project_store?.ready === true, `project store not ready: ${readiness.text}`);
    assert(readiness.payload.external_idc_deployment_claim === false, "readiness overclaimed IDC deployment");
    assert(!/[A-Z]:\\\\|I:\\\\|sg_cap|capability.*[A-Za-z0-9]{20,}/.test(readiness.text), `readiness leaked path/capability: ${readiness.text}`);

    const context = await browser.newContext({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    const page = await context.newPage();
    const failures = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") failures.push(msg.text());
    });
    page.on("pageerror", (err) => failures.push(err.message));
    await page.goto(`${baseUrl}/`, { waitUntil: "networkidle" });
    await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_projectile_xy_v1']", { timeout: 30_000 });
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().runs?.[0]?.latest_state_hash, null, { timeout: 30_000 });
    const run = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs[0]);
    assert(run.latest_state_hash, "clean browser did not execute WASM");
    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);
    await context.close();

    console.log(JSON.stringify({
      schema: "seamgrim.public_single_node_serving.runner_result.v1",
      base_url: baseUrl,
      data_dir: dataDir,
      wasm_files: readiness.payload.wasm_artifact.files.map((row) => row.name),
      project_store_status: readiness.payload.project_store.status,
      latest_state_hash: run.latest_state_hash,
    }));
    console.log(OK);
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

await main();
