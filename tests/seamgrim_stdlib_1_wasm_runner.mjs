#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

function fail(message) {
  console.error(`[seamgrim-stdlib-1-wasm] fail: ${message}`);
  process.exit(1);
}

function normalizeStdout(stdout) {
  return String(stdout ?? "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("state_hash=") && !line.startsWith("trace_hash="));
}

async function initWasm(root) {
  const uiDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModule = await import(pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href);
  const wrapper = await import(pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href);
  const runtimeState = await import(pathToFileURL(path.join(uiDir, "seamgrim_runtime_state.js")).href);
  const wasmBytes = await fs.readFile(path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm"));
  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }
  return { wasmModule, wrapper, runtimeState };
}

function runCli(root, sourcePath) {
  const proc = spawnSync(
    "cargo",
    ["run", "-q", "--manifest-path", "tools/teul-cli/Cargo.toml", "--", "run", sourcePath],
    { cwd: root, encoding: "utf8", windowsHide: true },
  );
  if (proc.status !== 0) {
    fail(`cli failed: ${proc.stderr || proc.stdout}`);
  }
  return normalizeStdout(proc.stdout);
}

async function runWasm(runtime, source) {
  const vm = new runtime.wasmModule.DdnWasmVm(source);
  const client = new runtime.wrapper.DdnWasmVmClient(vm);
  try {
    const state = client.runTicksParsed(1);
    return runtime.runtimeState
      .extractObservationOutputLogFromState(state)
      .map((entry) => String(entry?.text ?? ""))
      .filter(Boolean);
  } finally {
    if (typeof vm.free === "function") {
      vm.free();
    }
  }
}

async function main() {
  const root = process.cwd();
  const sourcePath = path.join(root, "pack", "stdlib_1_v1", "wasm_cli_stdlib_1.ddn");
  const source = await fs.readFile(sourcePath, "utf8");
  const cli = runCli(root, sourcePath);
  const wasm = await runWasm(await initWasm(root), source);
  if (JSON.stringify(cli) !== JSON.stringify(wasm)) {
    fail(`cli/wasm output mismatch cli=${JSON.stringify(cli)} wasm=${JSON.stringify(wasm)}`);
  }
  console.log("seamgrim stdlib 1 wasm runner ok");
}

main().catch((err) => fail(err?.stack ?? String(err)));
