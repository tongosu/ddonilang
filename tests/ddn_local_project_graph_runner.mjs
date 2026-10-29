#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function initWasm(rootDir) {
  const uiDir = path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModule = await import(pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href);
  const wasmBytes = await fs.readFile(path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm"));
  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }
  assert(typeof wasmModule.wasm_project_graph_json === "function", "wasm_project_graph_json export missing");
  assert(typeof wasmModule.wasm_project_symbols_json === "function", "wasm_project_symbols_json export missing");
  assert(typeof wasmModule.wasm_project_gaji_lock_json === "function", "wasm_project_gaji_lock_json export missing");
  assert(typeof wasmModule.wasm_project_run_summary_json === "function", "wasm_project_run_summary_json export missing");
  return wasmModule;
}

async function main() {
  const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const mode = process.argv[2] ?? "graph";
  const fixturePath =
    process.argv[3] ?? path.join(rootDir, "pack", "ddn_local_project_graph_v1", "fixtures", "project_graph_v1.json");
  const sourceJson = await fs.readFile(path.resolve(fixturePath), "utf8");
  const wasmModule = await initWasm(rootDir);
  if (mode === "graph") {
    process.stdout.write(String(wasmModule.wasm_project_graph_json(sourceJson)));
    return;
  }
  if (mode === "symbols") {
    process.stdout.write(String(wasmModule.wasm_project_symbols_json(sourceJson)));
    return;
  }
  if (mode === "lock") {
    process.stdout.write(String(wasmModule.wasm_project_gaji_lock_json(sourceJson)));
    return;
  }
  if (mode === "run") {
    const target = process.argv[4] ?? "앱";
    process.stdout.write(String(wasmModule.wasm_project_run_summary_json(sourceJson, target, true, true)));
    return;
  }
  throw new Error(`unsupported mode: ${mode}`);
}

main().catch((err) => {
  process.stderr.write(`${err?.stack ?? err}\n`);
  process.exit(1);
});
