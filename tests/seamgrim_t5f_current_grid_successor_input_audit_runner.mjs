#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs";
import fsp from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wasmDir = path.resolve(
  String(process.env.DDN_T5F_SOURCE_WASM_DIR ?? "").trim()
    || path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm"),
);

async function ddnFiles(directory) {
  const result = [];
  for (const entry of await fsp.readdir(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) result.push(...await ddnFiles(full));
    else if (entry.isFile() && entry.name.endsWith(".ddn")) result.push(full);
  }
  return result;
}

const wasm = await import(`${pathToFileURL(path.join(wasmDir, "ddonirang_tool.js")).href}?t5f_current=${Date.now()}`);
const bytes = await fsp.readFile(path.join(wasmDir, "ddonirang_tool_bg.wasm"));
if (typeof wasm.default === "function") await wasm.default({ module_or_path: bytes });
assert.equal(typeof wasm.DdnWasmVm, "function", "current DdnWasmVm constructor missing");
assert.equal(
  typeof wasm.DdnWasmVm.fromPreV25Source,
  "undefined",
  "PC02-excluded current product must not restore the historical compatibility constructor",
);

const wrapper = await import(pathToFileURL(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm_ddn_wrapper.js")).href);
const searchRoots = [path.join(root, "pack"), path.join(root, "solutions")];
const all = (await Promise.all(searchRoots.map(ddnFiles))).flat();
const semanticCandidates = [];
for (const file of all) {
  const source = await fsp.readFile(file, "utf8");
  if (source.includes("위치") && source.includes("보개_그림판_목록")) {
    semanticCandidates.push({ file, source });
  }
}
semanticCandidates.sort((a, b) => a.file.localeCompare(b.file));
assert.deepEqual(
  semanticCandidates.map((entry) => path.relative(root, entry.file).replaceAll("\\", "/")),
  ["solutions/seamgrim_ui_mvp/lessons/rep_grid_game_state_drop_v1/lesson.ddn"],
  "current grid authority input inventory changed; audit requires reclassification",
);

const observations = [];
for (const entry of semanticCandidates) {
  const vm = new wasm.DdnWasmVm(entry.source);
  const client = new wrapper.DdnWasmVmClient(vm);
  try {
    const initial = client.getStateParsed();
    const stepped = client.stepOneParsed();
    const initialResult = initial?.view_meta?.workbench?.results?.grid ?? null;
    const steppedResult = stepped?.view_meta?.workbench?.results?.grid ?? null;
    observations.push({
      input: path.relative(root, entry.file).replaceAll("\\", "/"),
      current_constructor: "new DdnWasmVm(source)",
      accepted: true,
      initial_grid_result: initialResult,
      stepped_grid_result: steppedResult,
    });
  } finally {
    vm.free();
  }
}

assert.equal(
  observations.some((row) => row.initial_grid_result || row.stepped_grid_result),
  false,
  "a current grid successor result now exists; replace this owner-decision audit with successor success evidence",
);

console.log(JSON.stringify({
  schema: "seamgrim.t5f.current_grid_successor_input_audit.v1_candidate",
  status: "T5F_CURRENT_GRID_SUCCESSOR_INPUT_OWNER_DECISION_REQUIRED",
  historical_fixed_target: "T5F_HISTORICAL_FIXED_TARGET_PASS_PRESERVED",
  pc02_exclusion: "PRESERVED",
  candidate_count: observations.length,
  observations,
  product_change: 0,
}));
