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
  assert(
    typeof wasmModule.wasm_normalize_project_source_json === "function",
    "wasm_normalize_project_source_json export missing",
  );
  return wasmModule;
}

async function main() {
  const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const fixturePath =
    process.argv[2] ??
    path.join(rootDir, "pack", "ddn_local_project_cli_wasm_parity_v1", "fixtures", "virtual_project.json");
  const sourceJson = await fs.readFile(path.resolve(fixturePath), "utf8");
  const wasmModule = await initWasm(rootDir);
  const normalized = wasmModule.wasm_normalize_project_source_json(sourceJson);
  process.stdout.write(String(normalized));
}

main().catch((err) => {
  process.stderr.write(`${err?.stack ?? err}\n`);
  process.exit(1);
});
