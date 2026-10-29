#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function readPath(rootDir, relPath) {
  return fs.readFile(path.join(rootDir, relPath), "utf8");
}

function cleanSource(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").trim();
}

async function initRuntime(rootDir) {
  const uiDir = path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModule = await import(pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href);
  const wrapper = await import(pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href);
  const wasmBytes = await fs.readFile(path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm"));
  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }
  assert(typeof wasmModule.DdnWasmVm === "function", "DdnWasmVm export missing");
  assert(typeof wrapper.DdnWasmVmClient === "function", "DdnWasmVmClient export missing");
  return { wasmModule, wrapper };
}

function requireSource(input, expectedLabel, expectedCode, label) {
  assert(input && typeof input === "object", `${label}: input missing`);
  assert(input.frame_source === expectedLabel, `${label}: frame_source expected ${expectedLabel}, got ${input.frame_source}`);
  assert(input.frame_source_code === expectedCode, `${label}: frame_source_code expected ${expectedCode}, got ${input.frame_source_code}`);
}

function checkPersonInput(input, label) {
  requireSource(input, "사람", 0, label);
  assert(Array.isArray(input.ai_injections ?? []), `${label}: ai_injections must be array`);
  assert(Array.isArray(input.net_events ?? []), `${label}: net_events must be array`);
}

async function main() {
  const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const runtime = await initRuntime(rootDir);
  const source = cleanSource(
    await readPath(rootDir, "pack/seamgrim_wasm_cli_runtime_parity_v1/cases/c10_setting_madi_count/input.ddn"),
  );
  const vm = new runtime.wasmModule.DdnWasmVm(source);
  const client = new runtime.wrapper.DdnWasmVmClient(vm);
  const evidence = {
    schema: "ddn.cli_wasm.post_wasm_shared_changes.report.v1",
    ok: true,
    source: "pack/seamgrim_wasm_cli_runtime_parity_v1/cases/c10_setting_madi_count/input.ddn",
    checks: {},
    limits: {
      state_hash_formula_change: false,
      net_event_source_product_injection: "not_exposed_by_wasm_api",
      runtime_semantics_change: false,
    },
  };

  try {
    evidence.checks.public_wasm_api = {
      has_get_input_state_json: typeof vm.get_input_state_json === "function",
      has_wrapper_inputStateParsed: typeof client.inputStateParsed === "function",
    };
    assert(evidence.checks.public_wasm_api.has_get_input_state_json, "get_input_state_json export missing");
    assert(evidence.checks.public_wasm_api.has_wrapper_inputStateParsed, "inputStateParsed wrapper missing");

    const initialInput = client.inputStateParsed();
    checkPersonInput(initialInput, "initial input");
    evidence.checks.initial_input = initialInput;

    const configuredMadi = client.configuredMadi();
    assert(configuredMadi === 3, `configured madi expected 3 got ${configuredMadi}`);
    evidence.checks.configured_madi = configuredMadi;

    const stepWithInput = client.stepOneWithInputFixed64V1Parsed(8, "d", 12, -34, 0, 0x80000000);
    checkPersonInput(stepWithInput.input, "step_one_with_input top-level");
    assert(stepWithInput.input.keys_pressed === 8, "step input keys_pressed not preserved");
    assert(stepWithInput.input.last_key_name === "d", "step input last_key_name not preserved");
    assert(stepWithInput.input.pointer_x_i32 === 12, "step input pointer_x_i32 not preserved");
    assert(stepWithInput.input.pointer_y_i32 === -34, "step input pointer_y_i32 not preserved");
    assert(stepWithInput.input.dt === "0.5", `step input dt expected 0.5 got ${stepWithInput.input.dt}`);
    evidence.checks.step_input = {
      keys_pressed: stepWithInput.input.keys_pressed,
      last_key_name: stepWithInput.input.last_key_name,
      pointer_x_i32: stepWithInput.input.pointer_x_i32,
      pointer_y_i32: stepWithInput.input.pointer_y_i32,
      dt: stepWithInput.input.dt,
      frame_source: stepWithInput.input.frame_source,
      frame_source_code: stepWithInput.input.frame_source_code,
    };

    assert(client.injectAiAction("assist", "{\"dir\":\"left\"}") === true, "injectAiAction must be available");
    const aiStep = client.stepOneParsed();
    checkPersonInput(aiStep.input, "ai step input");
    const aiRows = aiStep.input.ai_injections ?? [];
    assert(aiRows.length === 1, `ai injection count expected 1 got ${aiRows.length}`);
    const ai = aiRows[0];
    assert(ai.source === "슬기", `ai source expected 슬기 got ${ai.source}`);
    assert(ai.source_code === 1, `ai source_code expected 1 got ${ai.source_code}`);
    assert(ai.intent?.kind === "say", `ai intent kind expected say got ${ai.intent?.kind}`);
    assert(String(ai.intent?.text ?? "").includes("assist="), "ai intent text must preserve fixture key");
    evidence.checks.seulgi_injection = {
      source: ai.source,
      source_code: ai.source_code,
      intent_kind: ai.intent?.kind,
      accepted_madi: ai.accepted_madi,
      target_madi: ai.target_madi,
    };

    const afterAiInput = client.inputStateParsed();
    checkPersonInput(afterAiInput, "after ai input state");
    assert(afterAiInput.pending_ai_injection_count === 0, "pending ai injections must drain after step");
    evidence.checks.after_ai_input_state = {
      frame_source: afterAiInput.frame_source,
      pending_ai_injection_count: afterAiInput.pending_ai_injection_count,
    };

    const reset = client.resetParsed(false);
    checkPersonInput(reset.input, "reset input");
    assert(reset.input.keys_pressed === 0, "reset keys_pressed must be zero");
    assert(reset.input.last_key_name === "", "reset last_key_name must be empty");
    evidence.checks.reset_input = {
      frame_source: reset.input.frame_source,
      frame_source_code: reset.input.frame_source_code,
      keys_pressed: reset.input.keys_pressed,
      last_key_name: reset.input.last_key_name,
    };

    const replayStateJson = vm.step_one_with_input_fixed64_v1(2, "a", 7, 9, 0, 0x40000000);
    const restoreResult = client.restoreStateParsed(replayStateJson);
    assert(restoreResult.ok === true, "restore_state result ok must be true");
    const restoredInput = client.inputStateParsed();
    checkPersonInput(restoredInput, "restored input state");
    assert(restoredInput.keys_pressed === 2, "restored keys_pressed not preserved");
    assert(restoredInput.last_key_name === "a", "restored last_key_name not preserved");
    assert(restoredInput.pointer_x_i32 === 7, "restored pointer_x_i32 not preserved");
    assert(restoredInput.pointer_y_i32 === 9, "restored pointer_y_i32 not preserved");
    assert(restoredInput.dt === "0.25", `restored dt expected 0.25 got ${restoredInput.dt}`);
    evidence.checks.restore_input = {
      frame_source: restoredInput.frame_source,
      frame_source_code: restoredInput.frame_source_code,
      keys_pressed: restoredInput.keys_pressed,
      last_key_name: restoredInput.last_key_name,
      pointer_x_i32: restoredInput.pointer_x_i32,
      pointer_y_i32: restoredInput.pointer_y_i32,
      dt: restoredInput.dt,
    };

    evidence.checks.state_hash_exposed = {
      state_hash: String(client.getStateHash() ?? ""),
      formula_change: false,
    };
    assert(evidence.checks.state_hash_exposed.state_hash.startsWith("blake3:"), "state hash must remain exposed");
  } finally {
    if (typeof vm.free === "function") {
      vm.free();
    }
  }

  process.stdout.write(`${JSON.stringify(evidence, null, 2)}\n`);
}

main().catch((err) => {
  process.stderr.write(`${err?.stack ?? err}\n`);
  process.exit(1);
});
