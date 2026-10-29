#!/usr/bin/env node

import fs from "node:fs/promises";
import crypto from "node:crypto";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { spawn, execFile } from "node:child_process";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const SERVER = path.resolve(ROOT, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");
const MODE = process.argv.includes("--mode")
  ? String(process.argv[process.argv.indexOf("--mode") + 1] || "normal")
  : String(process.env.SEAMGRIM_LINEAGE_MODE || "normal");
const RUN_OWNER_ID = `sglife_${process.pid}_${Date.now()}_${crypto.randomBytes(4).toString("hex")}`;
const evidence = {
  schema: "seamgrim.workbench_run_archive_lineage.runner_result.v3",
  mode: MODE,
  run_owner_id: RUN_OWNER_ID,
  parent_pid: process.pid,
  owned_child_pids: [],
  ownership_basis: {
    strategy: "pre_stop_process_identity_snapshot",
    run_owner_id: RUN_OWNER_ID,
    global_process_diff_used: false,
    unrelated_process_termination_allowed: false,
    identity_match_fields: ["pid", "creation_time", "executable_name", "command_fingerprint_sha256"],
  },
  process_identity_fields: ["pid", "parent_pid", "creation_time", "executable_name", "command_fingerprint_sha256", "identity_sha256"],
  owned_process_identities: [],
  ignored_reused_pid_identities: [],
  requested_port: null,
  actual_bound_port: null,
  ready_record: null,
  temp_data_dir: null,
  timing_ms: {},
  assertions: {},
  api_requests: [],
  browser_events: [],
  teardown: {
    started: false,
    finished: false,
    escalated: false,
    pre_stop_owned_identities: [],
    residual_owned_pids: [],
    residual_owned_processes: [],
    residual_owned_sockets: 0,
  },
};
const startedAt = Date.now();

function nowMs() {
  return Date.now() - startedAt;
}

function redactTempDir(dir) {
  const base = path.basename(String(dir || ""));
  const digest = crypto.createHash("sha256").update(String(dir || "")).digest("hex").slice(0, 12);
  return `${base}:${digest}`;
}

function hashText(value) {
  return crypto.createHash("sha256").update(String(value || "")).digest("hex");
}

function publicProcessIdentity(raw, role = "process") {
  if (!raw) return null;
  const pid = Number(raw.pid ?? raw.ProcessId ?? 0);
  if (!pid) return null;
  const parentPid = Number(raw.parent_pid ?? raw.ParentProcessId ?? 0);
  const creationTime = String(raw.creation_time ?? raw.CreationDate ?? "");
  const executablePath = String(raw.executable_path ?? raw.ExecutablePath ?? raw.command_name ?? "");
  const commandLine = String(raw.command_line ?? raw.CommandLine ?? "");
  const executableName = path.basename(executablePath || commandLine.split(/\s+/u)[0] || "unknown");
  const identityMaterial = JSON.stringify({
    pid,
    parent_pid: parentPid,
    creation_time: creationTime,
    executable_path: executablePath,
    command_line: commandLine,
  });
  return {
    role,
    run_owner_id: RUN_OWNER_ID,
    pid,
    parent_pid: parentPid,
    creation_time: creationTime,
    executable_name: executableName,
    command_fingerprint_sha256: hashText(commandLine || executableName),
    identity_sha256: hashText(identityMaterial),
  };
}

function sameProcessIdentity(left, right) {
  return Boolean(left && right && Number(left.pid) === Number(right.pid) && left.identity_sha256 === right.identity_sha256);
}

function fail(code, message) {
  const err = new Error(message);
  err.code = code;
  throw err;
}

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

function requireProjectIdentity(cap, stage) {
  evidence.assertions[`${stage}_project_id_present`] = Boolean(cap.project_id);
  evidence.assertions[`${stage}_project_id_length`] = String(cap.project_id || "").length;
  evidence.assertions[`${stage}_capability_present`] = Boolean(cap.capability);
  if (!cap.project_id) fail("E_TEST_PROJECT_ID_EMPTY", `${stage}: project id is empty before API request`);
  if (!cap.capability) fail("E_TEST_CAPABILITY_EMPTY", `${stage}: capability is empty before API request`);
}

function workbenchUrlFromCapability(baseUrl, cap) {
  requireProjectIdentity(cap, "canonical_reopen_link");
  const url = new URL("/solutions/seamgrim_ui_mvp/ui/workbench/index.html", baseUrl);
  const params = new URLSearchParams();
  params.set("sg_project", cap.project_id);
  params.set("sg_role", cap.role === "edit" ? "edit" : "view");
  params.set("sg_cap", cap.capability);
  url.hash = params.toString();
  return url.href;
}

async function fetchJson(baseUrl, pathName, capability, body = null) {
  evidence.api_requests.push({
    method: body ? "POST" : "GET",
    path: pathName.replace(/[A-Za-z0-9_-]{18,}/g, "<id>"),
  });
  const response = await fetch(`${baseUrl}${pathName}`, {
    method: body ? "POST" : "GET",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${capability}`,
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  const payload = await response.json();
  if (!response.ok) throw new Error(`fetch failed ${response.status}: ${JSON.stringify(payload)}`);
  return payload;
}

async function fetchJsonExpectFailure(baseUrl, pathName, capability, body = null, expectedStatus = 403) {
  evidence.api_requests.push({
    method: body ? "POST" : "GET",
    path: pathName.replace(/[A-Za-z0-9_-]{18,}/g, "<id>"),
    expected_failure: true,
  });
  const response = await fetch(`${baseUrl}${pathName}`, {
    method: body ? "POST" : "GET",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${capability}`,
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  let payload = {};
  try {
    payload = await response.json();
  } catch (_) {
    payload = {};
  }
  if (response.ok) fail("E_TEST_VIEW_CAPABILITY_WRITE_ALLOWED", `${pathName} unexpectedly accepted view capability`);
  if (Number(response.status) !== Number(expectedStatus)) {
    fail("E_TEST_VIEW_CAPABILITY_REJECTION_STATUS", `${pathName} returned ${response.status}, expected ${expectedStatus}: ${JSON.stringify(payload)}`);
  }
  return {
    status: response.status,
    code: String(payload?.code ?? ""),
    user_message_present: Boolean(payload?.user_message),
  };
}

function tailBuffer() {
  const chunks = [];
  return {
    push(data) {
      chunks.push(String(data || ""));
      while (chunks.join("").length > 4000) chunks.shift();
    },
    text() {
      return chunks.join("").slice(-4000);
    },
  };
}

function execFileText(command, args, options = {}) {
  return new Promise((resolve) => {
    execFile(command, args, { ...options, windowsHide: true }, (error, stdout, stderr) => {
      resolve({
        ok: !error,
        code: error?.code ?? 0,
        stdout: String(stdout || ""),
        stderr: String(stderr || ""),
      });
    });
  });
}

async function processIdentity(pid, role = "process") {
  const target = Number(pid || 0);
  if (!target) return null;
  if (process.platform === "win32") {
    const ps = [
      "$p = Get-CimInstance Win32_Process -Filter \"ProcessId = " + target + "\";",
      "if ($null -eq $p) { exit 3 }",
      "[pscustomobject]@{",
      "  pid = [int]$p.ProcessId;",
      "  parent_pid = [int]$p.ParentProcessId;",
      "  creation_time = [string]$p.CreationDate;",
      "  executable_path = [string]$p.ExecutablePath;",
      "  command_line = [string]$p.CommandLine",
      "} | ConvertTo-Json -Compress",
    ].join(" ");
    const result = await execFileText("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", ps]);
    if (!result.ok || !result.stdout.trim()) return null;
    try {
      return publicProcessIdentity(JSON.parse(result.stdout), role);
    } catch (_) {
      return null;
    }
  }
  try {
    const procRoot = `/proc/${target}`;
    const stat = await fs.readFile(`${procRoot}/stat`, "utf-8");
    const statTail = stat.slice(stat.lastIndexOf(")") + 2).split(/\s+/u);
    const parentPid = Number(statTail[1] || 0);
    const startTicks = String(statTail[19] || "");
    const cmdline = (await fs.readFile(`${procRoot}/cmdline`, "utf-8")).replace(/\0/g, " ").trim();
    let executablePath = "";
    try {
      executablePath = await fs.readlink(`${procRoot}/exe`);
    } catch (_) {
      executablePath = cmdline.split(/\s+/u)[0] || "";
    }
    return publicProcessIdentity(
      {
        pid: target,
        parent_pid: parentPid,
        creation_time: startTicks,
        executable_path: executablePath,
        command_line: cmdline,
      },
      role,
    );
  } catch (_) {
    return null;
  }
}

async function descendantPids(rootPid) {
  const root = Number(rootPid || 0);
  if (!root) return [];
  if (process.platform === "win32") {
    const ps = [
      "Get-CimInstance Win32_Process |",
      "Select-Object ProcessId,ParentProcessId |",
      "ConvertTo-Csv -NoTypeInformation",
    ].join(" ");
    const result = await execFileText("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", ps]);
    if (!result.ok) return [];
    const rows = result.stdout
      .split(/\r?\n/u)
      .map((line) => line.trim().replace(/^"|"$/g, ""))
      .filter((line) => line && !line.startsWith("ProcessId"));
    const children = new Map();
    for (const row of rows) {
      const parts = row.split(/","|,/u).map((item) => item.replace(/^"|"$/g, ""));
      const pid = Number(parts[0]);
      const ppid = Number(parts[1]);
      if (!Number.isFinite(pid) || !Number.isFinite(ppid) || pid <= 0) continue;
      if (!children.has(ppid)) children.set(ppid, []);
      children.get(ppid).push(pid);
    }
    const out = [];
    const stack = [...(children.get(root) || [])];
    while (stack.length) {
      const pid = stack.pop();
      out.push(pid);
      stack.push(...(children.get(pid) || []));
    }
    return out;
  }
  const result = await execFileText("ps", ["-eo", "pid=,ppid="]);
  if (!result.ok) return [];
  const children = new Map();
  for (const line of result.stdout.split(/\r?\n/u)) {
    const [pidText, ppidText] = line.trim().split(/\s+/u);
    const pid = Number(pidText);
    const ppid = Number(ppidText);
    if (!Number.isFinite(pid) || !Number.isFinite(ppid)) continue;
    if (!children.has(ppid)) children.set(ppid, []);
    children.get(ppid).push(pid);
  }
  const out = [];
  const stack = [...(children.get(root) || [])];
  while (stack.length) {
    const pid = stack.pop();
    out.push(pid);
    stack.push(...(children.get(pid) || []));
  }
  return out;
}

async function captureOwnedProcessIdentities(rootPid) {
  const ids = [];
  const root = await processIdentity(rootPid, "ddn_exec_server");
  if (root) ids.push(root);
  const descendants = await descendantPids(rootPid);
  for (const pid of descendants) {
    const identity = await processIdentity(pid, "ddn_exec_server_descendant");
    if (identity) ids.push(identity);
  }
  const seen = new Set();
  return ids.filter((identity) => {
    const key = identity.identity_sha256;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function socketAccepts(port) {
  const target = Number(port || 0);
  if (!target) return Promise.resolve(false);
  return new Promise((resolve) => {
    const socket = net.createConnection({ host: "127.0.0.1", port: target });
    const done = (value) => {
      socket.removeAllListeners();
      socket.destroy();
      resolve(value);
    };
    socket.setTimeout(250, () => done(false));
    socket.once("connect", () => done(true));
    socket.once("error", () => done(false));
  });
}

function pidAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (_) {
    return false;
  }
}

async function forceKillOwned(pid) {
  if (!pid || !pidAlive(pid)) return;
  evidence.teardown.escalated = true;
  if (process.platform === "win32") {
    await execFileText("taskkill", ["/PID", String(pid), "/T", "/F"]);
    return;
  }
  try {
    process.kill(pid, "SIGKILL");
  } catch (_) {
    // Already gone.
  }
}

async function waitExit(child, timeoutMs = 1500) {
  if (!child || child.exitCode !== null) return true;
  return await new Promise((resolve) => {
    const timer = setTimeout(() => resolve(false), timeoutMs);
    child.once("exit", () => {
      clearTimeout(timer);
      resolve(true);
    });
  });
}

async function waitForHealth(baseUrl, child) {
  evidence.timing_ms.server_ready_wait_start = nowMs();
  for (let i = 0; i < 90; i += 1) {
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

async function waitForReadyRecord(child, stdoutBuffer, timeoutMs = 10_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) fail("E_TEST_SERVER_EXITED_BEFORE_READY", `server exited before ready: ${child.exitCode}`);
    const text = stdoutBuffer.text();
    const line = text.split(/\r?\n/u).find((row) => row.startsWith("SEAMGRIM_SERVER_READY "));
    if (line) {
      try {
        return JSON.parse(line.slice("SEAMGRIM_SERVER_READY ".length));
      } catch (_) {
        fail("E_TEST_SERVER_READY_PARSE", "server ready record is not valid JSON");
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  fail("E_TEST_SERVER_READY_TIMEOUT", "server did not emit structured ready record before timeout");
}

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-workbench-g5-"));
  const port = 0;
  evidence.requested_port = 0;
  evidence.temp_data_dir = redactTempDir(dataDir);
  const env = { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir, SEAMGRIM_RUN_OWNER_ID: RUN_OWNER_ID };
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env,
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  const stdout = tailBuffer();
  const stderr = tailBuffer();
  child.stdout.on("data", (data) => stdout.push(data));
  child.stderr.on("data", (data) => stderr.push(data));
  child.__seamgrim_stdout = stdout;
  child.__seamgrim_stderr = stderr;
  evidence.owned_child_pids.push(child.pid);
  const childIdentity = await processIdentity(child.pid, "ddn_exec_server");
  if (childIdentity) evidence.owned_process_identities.push(childIdentity);
  evidence.timing_ms.server_spawned = nowMs();
  const ready = await waitForReadyRecord(child, stdout);
  evidence.ready_record = {
    schema: ready.schema,
    host: ready.host,
    port: ready.port,
    deploy_profile: ready.deploy_profile,
    sqlite_sync: ready.sqlite_sync,
  };
  evidence.actual_bound_port = Number(ready.port);
  const baseUrl = `http://${ready.host}:${ready.port}`;
  await waitForHealth(baseUrl, child);
  evidence.timing_ms.server_ready = nowMs();
  return { child, baseUrl, dataDir };
}

async function stopServer(child, options = {}) {
  if (!child) return;
  evidence.teardown.started = true;
  evidence.timing_ms.teardown_start = nowMs();
  const preStopIdentities = child.exitCode === null ? await captureOwnedProcessIdentities(child.pid) : [];
  evidence.teardown.pre_stop_owned_identities = preStopIdentities;
  for (const identity of preStopIdentities) {
    if (!evidence.owned_process_identities.some((row) => row.identity_sha256 === identity.identity_sha256)) {
      evidence.owned_process_identities.push(identity);
    }
  }
  let exited = false;
  if (child.exitCode !== null) {
    exited = true;
  } else if (!options.forceOnly) {
    child.kill();
    exited = await waitExit(child);
  }
  if (options.forceOnly || !exited) await forceKillOwned(child.pid);
  await waitExit(child, 3000);
  const residualProcesses = [];
  for (const before of preStopIdentities) {
    const after = await processIdentity(before.pid, before.role);
    if (!after) continue;
    if (sameProcessIdentity(before, after)) {
      residualProcesses.push(after);
    } else {
      evidence.ignored_reused_pid_identities.push({
        pid: before.pid,
        before_identity_sha256: before.identity_sha256,
        after_identity_sha256: after.identity_sha256,
        classification: "pid_reused_or_identity_changed_ignored",
      });
    }
  }
  evidence.teardown.residual_owned_processes = residualProcesses;
  evidence.teardown.residual_owned_pids = residualProcesses.map((row) => row.pid);
  if (residualProcesses.length) {
    for (const row of residualProcesses) await forceKillOwned(row.pid);
  }
  evidence.teardown.residual_owned_sockets = (await socketAccepts(evidence.actual_bound_port)) ? 1 : 0;
  evidence.teardown.finished = true;
  evidence.timing_ms.teardown_end = nowMs();
}

async function waitArchive(page, predicate, label) {
  await page.waitForFunction(predicate, null, { timeout: 45_000 }).catch(async (err) => {
    const snap = await page.evaluate(() => {
      const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
      const apiReady = typeof api?.getSnapshot === "function";
      const raw = apiReady ? api.getSnapshot() : {};
      return {
        api_ready: apiReady,
        api_type: typeof api,
        root_visible: Boolean(document.querySelector("[data-testid='approved-workbench-root']")),
        current_url: location.href.replace(/sg_cap=[^&]+/g, "sg_cap=<redacted>"),
        run_count: Array.isArray(raw.runs) ? raw.runs.length : 0,
        archive_count: Array.isArray(raw.runArchives) ? raw.runArchives.length : 0,
        has_project_id: Boolean(raw.serverProject?.project_id || raw.serverProject?.projectId),
        has_head_revision: Boolean(raw.serverProject?.head_revision_id || raw.serverProject?.headRevisionId),
      };
    });
    fail("E_TEST_ARCHIVE_WAIT_TIMEOUT", `${label}: ${err.message}: ${JSON.stringify(snap)}`);
  });
}

async function waitForWorkbenchApi(page, label) {
  evidence.timing_ms[`${label}_api_wait_start`] = nowMs();
  await page.waitForFunction(() => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    if (typeof api?.getSnapshot !== "function") return false;
    try {
      const snap = api.getSnapshot();
      return Array.isArray(snap?.runs) && Array.isArray(snap?.runArchives) && Boolean(snap?.serverProject);
    } catch (_) {
      return false;
    }
  }, null, { timeout: 30_000 }).catch(async (err) => {
    const snap = await page.evaluate(() => ({
      api_type: typeof window.__SEAMGRIM_APPROVED_WORKBENCH__,
      get_snapshot_type: typeof window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot,
      root_visible: Boolean(document.querySelector("[data-testid='approved-workbench-root']")),
      source_visible: Boolean(document.querySelector("[data-testid='source-editor']")),
      current_url: location.href.replace(/sg_cap=[^&]+/g, "sg_cap=<redacted>"),
    }));
    fail("E_TEST_WORKBENCH_API_NOT_READY", `${label}: workbench API did not settle: ${err.message}: ${JSON.stringify(snap)}`);
  });
  evidence.timing_ms[`${label}_api_ready`] = nowMs();
}

async function runNoReadyNegativeControl() {
  const child = spawn(process.execPath, ["-e", "setTimeout(() => {}, 60000)"], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_RUN_OWNER_ID: RUN_OWNER_ID },
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  evidence.owned_child_pids.push(child.pid);
  const childIdentity = await processIdentity(child.pid, "no_ready_control");
  if (childIdentity) evidence.owned_process_identities.push(childIdentity);
  try {
    for (let i = 0; i < 5; i += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    fail("E_TEST_SERVER_READY_TIMEOUT", "server did not emit readiness before timeout");
  } finally {
    await stopServer(child);
  }
}

async function runTeardownDelayControl() {
  const child = spawn(process.execPath, ["-e", "setTimeout(() => {}, 60000)"], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_RUN_OWNER_ID: RUN_OWNER_ID },
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  evidence.owned_child_pids.push(child.pid);
  const childIdentity = await processIdentity(child.pid, "teardown_delay_control");
  if (childIdentity) evidence.owned_process_identities.push(childIdentity);
  await stopServer(child, { forceOnly: true });
  if (evidence.teardown.residual_owned_pids.length) {
    fail("E_TEST_OWNED_PROCESS_RESIDUAL", "owned teardown-delay process survived cleanup");
  }
}

async function runPidReuseSimulationControl() {
  const before = {
    role: "simulated_owned_process",
    run_owner_id: RUN_OWNER_ID,
    pid: 424242,
    parent_pid: process.pid,
    creation_time: "generation-a",
    executable_name: "owned-helper",
    command_fingerprint_sha256: hashText("owned-helper generation-a"),
    identity_sha256: hashText("pid-reuse-before"),
  };
  const after = {
    ...before,
    creation_time: "generation-b",
    command_fingerprint_sha256: hashText("unrelated-helper generation-b"),
    identity_sha256: hashText("pid-reuse-after"),
  };
  evidence.owned_process_identities.push(before);
  evidence.teardown.pre_stop_owned_identities = [before];
  if (!sameProcessIdentity(before, after)) {
    evidence.ignored_reused_pid_identities.push({
      pid: before.pid,
      before_identity_sha256: before.identity_sha256,
      after_identity_sha256: after.identity_sha256,
      classification: "pid_reused_or_identity_changed_ignored",
    });
  }
  evidence.assertions.pid_reuse_identity_mismatch_ignored = evidence.ignored_reused_pid_identities.length === 1;
  evidence.teardown.finished = true;
}

async function runOwnedLeakInjectionControl() {
  const child = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_RUN_OWNER_ID: RUN_OWNER_ID },
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  evidence.owned_child_pids.push(child.pid);
  const identity = await processIdentity(child.pid, "owned_leak_injection");
  if (identity) {
    evidence.owned_process_identities.push(identity);
    evidence.teardown.pre_stop_owned_identities = [identity];
    const after = await processIdentity(identity.pid, identity.role);
    if (sameProcessIdentity(identity, after)) {
      evidence.teardown.residual_owned_processes = [after];
      evidence.teardown.residual_owned_pids = [after.pid];
    }
  }
  evidence.teardown.finished = true;
  try {
    if (evidence.teardown.residual_owned_pids.length) {
      fail("E_TEST_OWNED_PROCESS_RESIDUAL", "owned leak injection was detected before cleanup");
    }
  } finally {
    await forceKillOwned(child.pid);
    await waitExit(child, 3000);
  }
}

async function requireSavedProjectFromPage(page, stage) {
  await waitForWorkbenchApi(page, `${stage}_require_saved`);
  const state = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject || {});
  const projectId = String(state.project_id ?? state.projectId ?? "");
  const headRevisionId = String(state.head_revision_id ?? state.headRevisionId ?? "");
  evidence.assertions[`${stage}_project_id_present`] = Boolean(projectId);
  evidence.assertions[`${stage}_project_id_length`] = projectId.length;
  evidence.assertions[`${stage}_head_revision_present`] = Boolean(headRevisionId);
  if (!projectId || !headRevisionId) {
    fail("E_TEST_PROJECT_SAVE_PREREQUISITE", `${stage}: project save prerequisite not satisfied`);
  }
  return { projectId, headRevisionId };
}

async function waitForStructuredProjectSave(page, stage) {
  await page.waitForFunction(() => {
    const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
    const raw = typeof api?.getSnapshot === "function" ? api.getSnapshot() : {};
    const server = raw.serverProject || {};
    const projectId = String(server.project_id ?? server.projectId ?? "");
    const headRevisionId = String(server.head_revision_id ?? server.headRevisionId ?? "");
    const editLink = String(document.querySelector("[data-testid='project-edit-link']")?.href ?? "");
    const viewLink = String(document.querySelector("[data-testid='project-view-link']")?.href ?? "");
    return Boolean(projectId && headRevisionId && editLink.includes("sg_cap=") && viewLink.includes("sg_cap="));
  }, null, { timeout: 30_000 }).catch(async (err) => {
    const snap = await page.evaluate(() => {
      const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
      const raw = typeof api?.getSnapshot === "function" ? api.getSnapshot() : {};
      return {
        api_ready: typeof api?.getSnapshot === "function",
        serverProject: raw.serverProject || {},
        capabilitySession: raw.capabilitySession || null,
        editLinkReady: String(document.querySelector("[data-testid='project-edit-link']")?.href ?? "").includes("sg_cap="),
        viewLinkReady: String(document.querySelector("[data-testid='project-view-link']")?.href ?? "").includes("sg_cap="),
        saveStatus: String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? ""),
      };
    });
    fail("E_TEST_PROJECT_SAVE_PREREQUISITE", `${stage}: structured save state did not settle: ${err.message}: ${JSON.stringify(snap)}`);
  });
  const saved = await page.evaluate(() => ({
    edit_link: String(document.querySelector("[data-testid='project-edit-link']")?.href ?? ""),
    view_link: String(document.querySelector("[data-testid='project-view-link']")?.href ?? ""),
    server: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject,
    capability_session: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().capabilitySession,
    status_text: String(document.querySelector("[data-testid='project-save-status']")?.textContent ?? ""),
  }));
  const serverProjectId = String(saved.server?.project_id ?? saved.server?.projectId ?? "");
  const serverHeadRevisionId = String(saved.server?.head_revision_id ?? saved.server?.headRevisionId ?? "");
  evidence.assertions[`${stage}_server_project_id_present`] = Boolean(serverProjectId);
  evidence.assertions[`${stage}_server_head_revision_present`] = Boolean(serverHeadRevisionId);
  evidence.assertions[`${stage}_edit_link_has_capability_fragment`] = saved.edit_link.includes("sg_cap=");
  evidence.assertions[`${stage}_view_link_has_capability_fragment`] = saved.view_link.includes("sg_cap=");
  evidence.assertions[`${stage}_capability_session_redacted`] = saved.capability_session?.capability_redacted === true;
  if (!serverProjectId || !serverHeadRevisionId) {
    fail("E_TEST_PROJECT_SAVE_PREREQUISITE", `${stage}: saved server project lacks identity`);
  }
  return saved;
}

function installPageCapture(page) {
  page.on("console", (msg) => {
    if (msg.type() === "error") evidence.browser_events.push({ type: "console_error", text: String(msg.text() || "").slice(0, 500) });
  });
  page.on("pageerror", (err) => evidence.browser_events.push({ type: "pageerror", text: String(err.message || "").slice(0, 500) }));
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (url.pathname.includes("/api/")) {
      evidence.api_requests.push({ method: request.method(), path: url.pathname.replace(/[A-Za-z0-9_-]{18,}/g, "<id>") });
    }
  });
}

async function main() {
  if (MODE === "pid-reuse-simulation") {
    await runPidReuseSimulationControl();
    return;
  }
  if (MODE === "owned-leak-injection") {
    await runOwnedLeakInjectionControl();
    return;
  }
  if (MODE === "no-ready") {
    await runNoReadyNegativeControl();
    return;
  }
  if (MODE === "teardown-delay") {
    await runTeardownDelayControl();
    return;
  }
  const { child, baseUrl, dataDir } = await startServer();
  const browser = await chromium.launch({ headless: true });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await context.newPage();
    installPageCapture(page);
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-testid='source-editor']", { state: "visible", timeout: 30_000 });
    await waitForWorkbenchApi(page, "initial_page");
    if (MODE === "premature-run") {
      await requireSavedProjectFromPage(page, "premature_run");
      return;
    }
    await page.fill("[data-testid='source-editor']", `설정 { 제목: "워크벤치 실행계보". 마디수: 120. }.\n\n(매마디)마다 {\n  1 보여주기.\n}.\n`);
    await page.click("[data-testid='project-save']");
    const saveState = await waitForStructuredProjectSave(page, "after_save");
    evidence.timing_ms.project_save_done = nowMs();
    await requireSavedProjectFromPage(page, "after_save");
    const editCap = parseCapabilityLink(saveState.edit_link);
    const viewCap = parseCapabilityLink(saveState.view_link);
    evidence.timing_ms.capability_parse = nowMs();
    if (MODE === "empty-project-id") editCap.project_id = "";
    requireProjectIdentity(editCap, "edit_link");
    requireProjectIdentity(viewCap, "view_link");
    assert(editCap.role === "edit", "edit link carries edit role");
    assert(viewCap.role === "view", "view link carries view role");
    assert(String(saveState.server.project_id) === editCap.project_id, "save project id matches edit capability link");
    assert(String(saveState.server.project_id) === viewCap.project_id, "save project id matches view capability link");

    await page.click("[data-testid='lifecycle-run']");
    await waitArchive(page, () => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const run = snap?.runs?.[0];
      return snap?.executionLifecycle?.status === "running" && Boolean(run?.server_run_id) && run.server_checkpoint_count >= 1;
    }, "first checkpoint");
    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
    await waitArchive(page, () => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.executionLifecycle?.status === "paused" && !snap?.livePump?.in_flight;
    }, "first pause boundary");
    const firstPause = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    const parentLocalRunId = firstPause.activeRunId;
    const checkpointCountAtPause = firstPause.runs[0].server_checkpoint_count;
    await page.click("[data-testid='lifecycle-run'][data-action='resume']");
    await page.waitForFunction(({ runId, checkpointCount }) => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.executionLifecycle?.status === "running"
        && snap?.activeRunId === runId
        && snap?.runs?.[0]?.server_checkpoint_count > checkpointCount;
    }, { runId: parentLocalRunId, checkpointCount: checkpointCountAtPause }, { timeout: 45_000 });
    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
    await waitArchive(page, () => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.executionLifecycle?.status === "paused" && !snap?.livePump?.in_flight;
    }, "second pause boundary");
    const secondPause = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert(secondPause.activeRunId === parentLocalRunId, "pause/resume changed local run id");
    const checkpointCountBeforeStep = secondPause.runs[0].server_checkpoint_count;
    await page.click("[data-testid='lifecycle-step']");
    await page.waitForFunction((checkpointCount) => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const run = snap?.runs?.[0];
      return Boolean(run?.server_run_id) && run.server_checkpoint_count === checkpointCount + 1;
    }, checkpointCountBeforeStep, { timeout: 45_000 });
    const beforeViewChange = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    await page.click("[data-mode='bogae-main']");
    const afterViewChange = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert(afterViewChange.activeFrame.state_hash === beforeViewChange.activeFrame.state_hash, "view state changed world state hash");
    assert(afterViewChange.runs[0].server_checkpoint_count === beforeViewChange.runs[0].server_checkpoint_count, "view state appended a world checkpoint");
    assert(afterViewChange.runs[0].server_latest_checkpoint_sha256 === beforeViewChange.runs[0].server_latest_checkpoint_sha256, "view state changed checkpoint lineage");
    await page.click("[data-mode='ddn-main']");
    const parentBeforeStop = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs[0]);
    await page.click("[data-testid='lifecycle-stop']");
    await waitArchive(page, () => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.runs?.[0]?.server_final_status === "stopped";
    }, "finalize parent");
    const stoppedParentLocal = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs[0]);
    assert(stoppedParentLocal.server_finalize_attempt_count === 1 && stoppedParentLocal.server_finalized, "stopped run did not finalize exactly once");
    const parentDetailBeforeBranch = await fetchJson(baseUrl, `/api/v1/projects/${editCap.project_id}/runs/${stoppedParentLocal.server_run_id}`, editCap.capability);

    await page.click("[data-testid='history-back']");
    await page.click("[data-testid='history-branch']");
    await waitArchive(page, () => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.runs?.length >= 2 && Boolean(snap.runs[1].server_run_id) && snap.runs[1].server_checkpoint_count >= 1;
    }, "branch checkpoint");
    await page.click("[data-testid='lifecycle-stop']");
    await waitArchive(page, () => {
      const runs = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runs ?? [];
      return runs[1]?.server_final_status === "stopped";
    }, "finalize branch");
    const localRuns = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs);
    const parent = localRuns[0];
    const childRun = localRuns[1];
    assert(childRun.parent_server_run_id === parent.server_run_id, "child run records parent server_run_id");
    assert(childRun.parent_state_hash, "child run records parent branch hash");
    assert(parent.server_finalize_attempt_count === 1 && childRun.server_finalize_attempt_count === 1, "stopped parent/branch did not finalize exactly once");

    const listPayload = await fetchJson(baseUrl, `/api/v1/projects/${editCap.project_id}/runs`, editCap.capability);
    assert(listPayload.runs.length >= 2, "server lists parent and child runs");
    const parentDetail = await fetchJson(baseUrl, `/api/v1/projects/${editCap.project_id}/runs/${parent.server_run_id}`, editCap.capability);
    const childDetail = await fetchJson(baseUrl, `/api/v1/projects/${editCap.project_id}/runs/${childRun.server_run_id}`, editCap.capability);
    assert(parentDetail.run.checkpoints.length >= 3, "parent checkpoints persisted");
    assert(parentDetail.run.final_status === "stopped", "parent finalized");
    assert(JSON.stringify(parentDetail.run) === JSON.stringify(parentDetailBeforeBranch.run), "branch mutated the finalized parent archive");
    assert(childDetail.run.parent_server_run_id === parent.server_run_id, "server child parent lineage persisted");
    assert(childDetail.run.branch_state_hash === childRun.parent_state_hash, "server child branch hash persisted");
    assert(childDetail.run.checkpoints.every((cp) => cp.artifact.frame_summary.renderer_pixels_stored === false), "renderer pixels excluded");
    assert(parent.source_revision_id === saveState.server.head_revision_id, "run source revision sealed");
    assert(parentDetail.project_id === editCap.project_id, "parent detail project id matches saved project");
    assert(childDetail.project_id === editCap.project_id, "child detail project id matches saved project");
    assert(parentBeforeStop.latest_state_hash, "state hash observed before stop");
    const parentFrames = parent.frames ?? [];
    assert(parentFrames.every((frame, index) => frame.frame_id === index), "local frame ids are not contiguous");
    assert(parentFrames.every((frame, index) => index === 0 || frame.madi >= parentFrames[index - 1].madi), "local committed madi regressed");
    const parentCheckpoints = parentDetail.run.checkpoints;
    assert(parentCheckpoints.every((checkpoint, index) => checkpoint.sequence === index), "server checkpoint sequence is not contiguous");
    assert(parentCheckpoints.every((checkpoint, index) => index === 0 || checkpoint.committed_madi >= parentCheckpoints[index - 1].committed_madi), "server committed madi regressed");
    assert(parentCheckpoints[0].previous_checkpoint_sha256 === "", "first checkpoint has a previous hash");
    assert(parentCheckpoints.every((checkpoint, index) => index === 0 || checkpoint.previous_checkpoint_sha256 === parentCheckpoints[index - 1].checkpoint_sha256), "checkpoint hash chain broke");

    const revisionBeforeDone = String((await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject.head_revision_id)) ?? "");
    await page.fill("[data-testid='source-editor']", `설정 { 제목: "완료 실행". 마디수: 2. }.\n\n(매마디)마다 {\n  2 보여주기.\n}.\n`);
    await page.click("[data-testid='project-save']");
    await page.waitForFunction((before) => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.persistence?.phase === "saved"
        && Boolean(snap?.serverProject?.head_revision_id)
        && snap.serverProject.head_revision_id !== before;
    }, revisionBeforeDone, { timeout: 30_000 });
    await page.click("[data-testid='lifecycle-run']");
    await waitArchive(page, () => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const run = snap?.runs?.[2];
      return run?.server_final_status === "completed" && run?.server_finalize_attempt_count === 1;
    }, "completed finalize exactly once");
    const completedRun = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs[2]);
    assert(completedRun.server_finalized && completedRun.server_finalize_attempt_count === 1, "completed run did not finalize exactly once");

    const revisionBeforeFailure = String((await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject.head_revision_id)) ?? "");
    await page.fill("[data-testid='source-editor']", `설정 { 제목: "실패 실행". 마디수: 2. }.\n\n@@@\n`);
    await page.click("[data-testid='project-save']");
    await page.waitForFunction((before) => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.persistence?.phase === "saved"
        && Boolean(snap?.serverProject?.head_revision_id)
        && snap.serverProject.head_revision_id !== before;
    }, revisionBeforeFailure, { timeout: 30_000 });
    await page.click("[data-testid='lifecycle-run']");
    await waitArchive(page, () => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      const run = snap?.runs?.[3];
      return Boolean(run?.server_final_status) && run.server_final_status !== "running";
    }, "failed finalize exactly once");
    const failedOutcome = await page.evaluate(() => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return {
        diagnostics: snap.diagnostics,
        run: snap.runs[3],
      };
    });
    const failedRun = failedOutcome.run;
    assert(failedRun.server_final_status === "failed", `runtime failure fixture finalized as ${failedRun.server_final_status}: ${JSON.stringify(failedOutcome)}`);
    assert(failedRun.server_finalized && failedRun.server_finalize_attempt_count === 1, "failed run did not finalize exactly once");
    const runsAfterTerminalCases = await fetchJson(baseUrl, `/api/v1/projects/${editCap.project_id}/runs`, editCap.capability);
    assert(!runsAfterTerminalCases.runs.some((run) => run.final_status === "running"), "terminal runs left a running archive orphan");

    const viewRejectCreate = await fetchJsonExpectFailure(baseUrl, `/api/v1/projects/${viewCap.project_id}/runs`, viewCap.capability, {
      source_revision_id: saveState.server.head_revision_id,
      recipe_hash: "view-capability-negative",
      seed: "view-capability-negative",
    });
    const viewRejectCheckpoint = await fetchJsonExpectFailure(baseUrl, `/api/v1/projects/${viewCap.project_id}/runs/${parent.server_run_id}/checkpoints`, viewCap.capability, {
      checkpoint: {
        sequence: 999,
        committed_madi: 999,
        previous_checkpoint_sha256: "",
        input_snapshots: [],
        state_hashes: [],
        frame_summary: { local_run_id: "view-negative", frame_count: 0, latest_frame_id: 999, renderer_pixels_stored: false },
      },
    });
    const viewRejectFinalize = await fetchJsonExpectFailure(baseUrl, `/api/v1/projects/${viewCap.project_id}/runs/${parent.server_run_id}/finalize`, viewCap.capability, {
      final_status: "view_negative",
    });
    const viewRejectRestore = await fetchJsonExpectFailure(baseUrl, `/api/v1/projects/${viewCap.project_id}/restore`, viewCap.capability, {
      source_revision_id: saveState.server.head_revision_id,
      expected_parent_revision_id: saveState.server.head_revision_id,
    });
    evidence.view_capability_rejections = {
      create_run: viewRejectCreate,
      checkpoint: viewRejectCheckpoint,
      finalize: viewRejectFinalize,
      restore: viewRejectRestore,
    };

    const reopenedContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const reopened = await reopenedContext.newPage();
    installPageCapture(reopened);
    const canonicalReopenUrl = workbenchUrlFromCapability(baseUrl, editCap);
    await reopened.goto(canonicalReopenUrl, { waitUntil: "domcontentloaded" });
    await reopened.waitForSelector("[data-testid='source-editor']", { state: "visible", timeout: 30_000 });
    await waitForWorkbenchApi(reopened, "reopened_page");
    await waitArchive(reopened, () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runArchives?.length >= 4, "reopen archive list");
    const reopenedState = await reopened.evaluate(() => ({
      href: location.href,
      archiveText: String(document.querySelector("[data-testid='run-archive-status']")?.textContent ?? ""),
      archives: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runArchives,
    }));
    assert(!reopenedState.href.includes("sg_cap="), "capability fragment scrubbed on reopen");
    assert(reopenedState.archives.length >= 4, "new browser sees every terminal run archive");
    assert(reopenedState.archiveText.includes("server runs"), "archive status visible without active local run");
    await reopened.locator("[data-testid='project-more-menu'] > summary").click();
    await reopened.click("[data-testid='project-history-open']");
    const reopenedParentButton = reopened.locator(`[data-testid='run-archive-list'] button[data-server-run-id='${parent.server_run_id}']`);
    await reopenedParentButton.waitFor({ state: "visible", timeout: 30_000 });
    await reopenedParentButton.click();
    await reopened.waitForFunction((serverRunId) => {
      const selected = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.selectedRunArchive;
      return selected?.server_run_id === serverRunId && selected?.checkpoints?.length >= 3;
    }, parent.server_run_id, { timeout: 30_000 });
    const reopenedParent = await reopened.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().selectedRunArchive);
    assert(reopenedParent.checkpoints.length === parentDetail.run.checkpoints.length, "reopened archive detail lost checkpoint frames");
    assert(await reopened.locator("[data-testid='run-archive-checkpoint-list'] li").count() === parentDetail.run.checkpoints.length, "history drawer did not render archived checkpoint frames");
    assert(reopenedParent.checkpoints.every((checkpoint, index) => index === 0 || checkpoint.previous_checkpoint_sha256 === reopenedParent.checkpoints[index - 1].checkpoint_sha256), "reopened checkpoint hash chain mismatch");

    Object.assign(evidence.assertions, {
      parent_checkpoints: parentDetail.run.checkpoints.length,
      child_parent_recorded: childDetail.run.parent_server_run_id === parent.server_run_id,
      run_archive_project_binding_preserved: parentDetail.project_id === editCap.project_id && childDetail.project_id === editCap.project_id,
      view_capability_write_rejected: Object.values(evidence.view_capability_rejections).every((row) => row.status === 403),
      reopened_run_count: reopenedState.archives.length,
      reopened_parent_checkpoint_count: reopenedParent.checkpoints.length,
      no_projects_double_slash_request: !evidence.api_requests.some((row) => row.path.includes("/projects//")),
      temp_data_dir_removed: false,
    });
    await reopenedContext.close();
    await context.close();
  } finally {
    await browser.close();
    await stopServer(child);
    await fs.rm(dataDir, { recursive: true, force: true });
    evidence.assertions.temp_data_dir_removed = true;
  }
}

await main()
  .then(() => {
    console.log(JSON.stringify(evidence, null, 2));
  })
  .catch(async (err) => {
    evidence.failure = {
      code: String(err?.code || "E_TEST_UNEXPECTED"),
      message: String(err?.message || err || "").slice(0, 1200),
    };
    console.error(JSON.stringify(evidence, null, 2));
    process.exit(1);
  });
