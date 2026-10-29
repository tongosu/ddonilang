#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function loadRuntime(rootDir) {
  const uiDir = path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModule = await import(pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href);
  const wrapper = await import(pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href);
  const wasmBytes = await fs.readFile(path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm"));
  await wasmModule.default({ module_or_path: wasmBytes });
  return { wasmModule, DdnWasmVmClient: wrapper.DdnWasmVmClient };
}

async function main() {
  const rootDir = process.env.DDN_REPO_ROOT
    ? path.resolve(process.env.DDN_REPO_ROOT)
    : path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const { wasmModule, DdnWasmVmClient } = await loadRuntime(rootDir);
  const cases = [
    ["no_hook", "1 보여주기.", "oneshot"],
    ["start_only", "(시작)할때 { 1 보여주기. }.", "oneshot"],
    ["condition_only", "(1 < 2)이 될때 { 1 보여주기. }.", "oneshot"],
    ["every_madi", "(매마디)마다 { 1 보여주기. }.", "live"],
    ["every_n_madi", "(4마디)마다 { 1 보여주기. }.", "live"],
    ["alias_whitespace", "( 매틱 ) 마다 { 1 보여주기. }.", "live"],
    ["hook_text_string", "\"(매마디)마다 { }\" 보여주기.", "oneshot"],
    ["hook_text_comment", "// (매마디)마다 { }\n1 보여주기.", "oneshot"],
  ];

  const rows = [];
  for (const [id, source, expectedMode] of cases) {
    const envelope = JSON.parse(wasmModule.wasm_execution_envelope_json(source, 1));
    const envelopeDecision = envelope.execution_mode_decision;
    assert(envelopeDecision?.authority === "parsed_program", `${id}: envelope authority`);
    assert(envelopeDecision?.mode === expectedMode, `${id}: envelope mode`);

    const client = new DdnWasmVmClient(new wasmModule.DdnWasmVm(source));
    const vmDecision = client.executionModeDecisionParsed();
    assert(vmDecision?.authority === "parsed_program", `${id}: vm authority`);
    assert(vmDecision?.mode === expectedMode, `${id}: vm mode`);
    assert(JSON.stringify(vmDecision) === JSON.stringify(envelopeDecision), `${id}: VM/envelope divergence`);
    rows.push({ id, expected_mode: expectedMode, decision: vmDecision });
  }

  console.log(JSON.stringify({
    schema: "seamgrim.execution_mode_decision_runner.v1",
    status: "PASS",
    case_count: rows.length,
    rows,
  }));
}

main().catch((err) => {
  console.error(err?.stack ?? String(err));
  process.exitCode = 1;
});
