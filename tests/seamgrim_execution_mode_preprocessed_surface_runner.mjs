#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { WasmVmHandle } from "../solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const ui = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModule = await import(pathToFileURL(path.join(ui, "wasm", "ddonirang_tool.js")).href);
  const { DdnWasmVmClient } = await import(pathToFileURL(path.join(ui, "wasm_ddn_wrapper.js")).href);
  const wasmBytes = await fs.readFile(path.join(ui, "wasm", "ddonirang_tool_bg.wasm"));
  await wasmModule.default({ module_or_path: wasmBytes });

  const source = await fs.readFile(
    path.join(root, "solutions", "seamgrim_ui_mvp", "lessons", "rep_phys_pendulum_wave1_v1", "lesson.ddn"),
    "utf8",
  );
  assert(source.includes("(매마디)마다"), "pendulum live hook missing");

  const loader = {
    async ensure(preprocessedSource) {
      const client = new DdnWasmVmClient(new wasmModule.DdnWasmVm(preprocessedSource));
      client.wasmModule = wasmModule;
      return client;
    },
  };
  const vm = new WasmVmHandle({ loader, defaultSourceText: source });
  await vm.updateLogic(source);
  const decision = vm.getExecutionModeDecision();
  assert(decision?.authority === "parsed_program", `decision authority mismatch: ${JSON.stringify(decision)}`);
  assert(decision?.mode === "live", `preprocessed product path misclassified live source: ${JSON.stringify(decision)}`);
  assert(vm.getConfiguredMadi() === 240, `preprocessed product path lost configured madi: ${vm.getConfiguredMadi()}`);
  assert(vm.lastBodyText.includes("매마디:움직씨"), "negative control: legacy lowering did not run");
  assert(!vm.lastBodyText.includes("(매마디)마다"), "negative control: hook unexpectedly survived lowering");

  console.log("seamgrim_execution_mode_preprocessed_surface: ok");
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
