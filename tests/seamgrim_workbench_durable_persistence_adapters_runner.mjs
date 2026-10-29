#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import net from "node:net";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_workbench_durable_persistence_adapters: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function measureSimpleCommand() {
  const started = performance.now();
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, ["-e", "process.exit(0)"], { stdio: "ignore" });
    child.once("error", reject);
    child.once("exit", (code) => {
      if (code !== 0) reject(new Error(`simple process command exited ${code}`));
      else resolve({ latencyMs: performance.now() - started, pid: child.pid, exited: true });
    });
  });
}

function isPortListening(port) {
  return new Promise((resolve) => {
    const socket = net.createConnection({ host: "127.0.0.1", port });
    let settled = false;
    const finish = (value) => {
      if (settled) return;
      settled = true;
      socket.destroy();
      resolve(value);
    };
    socket.once("connect", () => finish(true));
    socket.once("error", () => finish(false));
    socket.setTimeout(1_000, () => finish(false));
  });
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".toml") || file.endsWith(".ddn") || file.endsWith(".md")) return "text/plain; charset=utf-8";
  return "application/octet-stream";
}

function createServer(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      const rel = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/workbench/index.html" : url.pathname).replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, rel);
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, {
        "content-type": mimeType(file),
        "cache-control": "no-store",
        "access-control-allow-origin": "*",
      });
      res.end(bytes);
    } catch (_) {
      res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      res.end("not found");
    }
  });
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("failed to bind server"));
        return;
      }
      resolve({ server, port: address.port, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function waitForWorkbenchReady(page, baseUrl) {
  try {
    await page.waitForFunction(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snapshot?.readiness?.workbench_api_ready === true
        && snapshot?.readiness?.initialization === "ready"
        && snapshot?.controls?.project_save?.enabled === true;
    }, null, { timeout: 30_000 });
  } catch (err) {
    const product = await page.evaluate(() => ({
      api: Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot),
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.() ?? null,
      save_disabled: document.querySelector("[data-testid='project-save']")?.disabled ?? null,
      status: document.querySelector("[data-testid='project-save-status']")?.textContent ?? "",
    })).catch(() => null);
    const health = await fetch(baseUrl).then((response) => response.status).catch(() => "unreachable");
    throw new Error(`workbench persistence readiness timeout: ${JSON.stringify({ product, health, cause: String(err) })}`);
  }
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  for (const rel of [
    "solutions/seamgrim_ui_mvp/ui/workbench/persistence/local_fallback.js",
    "solutions/seamgrim_ui_mvp/ui/workbench/persistence/project_adapter.js",
    "solutions/seamgrim_ui_mvp/ui/workbench/persistence/capability_session.js",
    "solutions/seamgrim_ui_mvp/ui/workbench/persistence/run_archive_adapter.js",
  ]) {
    const stat = await fs.stat(path.join(root, rel)).catch(() => null);
    assert(stat?.isFile(), `missing persistence module: ${rel}`);
  }

  const simpleCommand = await measureSimpleCommand();
  const serverStarted = performance.now();
  const { server, port, baseUrl } = await createServer(root);
  const serverBindLatencyMs = performance.now() - serverStarted;
  let browser = null;
  const failures = [];
  let result = null;
  try {
    const browserStarted = performance.now();
    browser = await chromium.launch({ headless: true });
    const browserLaunchLatencyMs = performance.now() - browserStarted;
    const context = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      locale: "ko-KR",
      reducedMotion: "reduce",
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      const text = msg.text();
      if (["error", "warning"].includes(msg.type()) && !text.includes("Failed to load resource")) {
        failures.push(`${msg.type()}: ${text}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("response", (res) => {
      const url = res.url();
      if (res.status() >= 400 && !url.includes("/api/v1/projects")) {
        failures.push(`response ${res.status()}: ${url}`);
      }
    });

    const cap = "test-capability-do-not-expose-raw-value-1234567890";
    await page.goto(
      `${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html#sg_project=sgp_test&sg_role=edit&sg_cap=${encodeURIComponent(cap)}`,
      { waitUntil: "networkidle" },
    );
    await waitForWorkbenchReady(page, baseUrl);

    const scrubbedHash = await page.evaluate(() => window.location.hash);
    assert(scrubbedHash === "", `capability fragment was not scrubbed: ${scrubbedHash}`);

    const beforeSave = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().persistence.generation);
    await page.click("[data-testid='project-save']");
    await page.waitForFunction((generation) => {
      const persistence = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.().persistence;
      return persistence?.completed_generation > generation
        && persistence?.phase === "saved"
        && persistence?.mode === "local_fallback";
    }, beforeSave, { timeout: 30_000 });
    const statusText = await page.textContent("[data-testid='project-save-status']");
    assert(statusText.includes("이 기기에만 임시 저장"), `local fallback status missing: ${statusText}`);
    assert(!statusText.includes("서버 저장 완료"), `local fallback overclaimed server save: ${statusText}`);

    let snap = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.());
    assert(snap.persistenceStatus?.mode === "local_fallback", "debug snapshot did not expose local fallback mode");
    assert(snap.persistenceStatus?.server_saved === false, "local fallback marked server_saved");
    assert(snap.capabilitySession?.capability_redacted === true, "capability should be redacted in debug snapshot");
    assert(JSON.stringify(snap).includes(cap) === false, "raw capability leaked through debug snapshot");

    const artifact = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.buildProjectArtifact?.());
    assert(artifact?.schema === "seamgrim.project_snapshot_artifact.v0", `artifact schema mismatch: ${artifact?.schema}`);
    assert(artifact?.project_snapshot?.metadata?.schema === "seamgrim.workbench.project_adapter.v1", "project adapter metadata missing");
    assert(artifact?.project_snapshot?.boundaries?.server_executes_ddn_for_save === false, "save boundary changed unexpectedly");

    const runBoundary = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.summarizeRunArchiveBoundary?.());
    assert(["G4", "G5"].includes(runBoundary?.server_binding_gate), `run archive adapter gate mismatch: ${runBoundary?.server_binding_gate}`);
    assert(runBoundary?.renderer_pixels_stored === false, "run archive boundary must exclude renderer pixels");

    await page.evaluate(() => {
      const legacy = {
        schema: "seamgrim.approved_workbench.project_snapshot.v1",
        project_id: "rep_phys_projectile_xy_v1",
        active_file_path: "lesson.ddn",
        mode: "bogae-main",
        view_role: "side",
        recipe_version: 3,
        files: [{ path: "lesson.ddn", role: "main", text: "legacy migrated source" }],
        runs: [],
      };
      localStorage.setItem("seamgrim.approved_workbench.project.rep_phys_projectile_xy_v1.v1", JSON.stringify(legacy));
    });
    await page.click("[data-testid='project-reopen']");
    await page.waitForFunction(() => document.querySelector("[data-testid='source-editor']")?.value === "legacy migrated source");
    const mode = await page.evaluate(() => document.querySelector("[data-testid='workbench-layout']")?.dataset.mode);
    assert(mode === "bogae-main", `legacy mode did not migrate: ${mode}`);

    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);
    await context.close();
    result = {
      schema: "seamgrim.workbench_durable_persistence_adapters.runner_result.v2",
      readiness: snap.readiness,
      persistence: snap.persistence,
      environment: {
        simple_process_command_latency_ms: Number(simpleCommand.latencyMs.toFixed(3)),
        static_server_bind_latency_ms: Number(serverBindLatencyMs.toFixed(3)),
        browser_launch_latency_ms: Number(browserLaunchLatencyMs.toFixed(3)),
      },
    };
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
  const socketOpen = await isPortListening(port);
  assert(socketOpen === false, `owned static server socket still listening: ${port}`);
  assert(result, "runner result missing after cleanup");
  result.owned_residual = { processes: 0, sockets: socketOpen ? 1 : 0, temp_dirs: 0 };
  console.log(JSON.stringify(result));
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
