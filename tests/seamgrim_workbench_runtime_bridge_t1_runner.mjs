import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import {
  RUNTIME_BRIDGE_V1_OPERATIONS,
  RUNTIME_BRIDGE_V1_SCHEMA,
  RuntimeBridgeV1,
  WasmRuntimeAdapter,
} from "../solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js";
import {
  importLocalGotganArtifact,
  loadLocalGotganStore,
  publishToLocalGotgan,
  searchLocalGotgan,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/persistence/local_gotgan_store.js";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const baselineCommit = "15f983fe32e0f04c4783e67118610f7441b401c6";
const workbenchRel = "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js";
const gotganRel = "solutions/seamgrim_ui_mvp/ui/workbench/persistence/local_gotgan_store.js";

function gitShow(commit, rel) {
  return execFileSync("git", ["show", `${commit}:${rel}`], {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
  }).replaceAll("\r\n", "\n");
}

function rawWasmCalls(text) {
  return [...text.matchAll(/\b(?:wasmModule|wasm)\.(wasm_[A-Za-z0-9_]+)\s*\(/gu)]
    .map((match) => match[1]);
}

function rawCallInventory(workbenchText, gotganText) {
  const calls = [...rawWasmCalls(workbenchText), ...rawWasmCalls(gotganText)];
  return {
    families: [...new Set(calls)].sort(),
    call_sites: calls.length,
  };
}

const baselineWorkbench = gitShow(baselineCommit, workbenchRel);
const baselineGotgan = gitShow(baselineCommit, gotganRel);
const currentWorkbench = fs.readFileSync(path.join(root, workbenchRel), "utf8");
// Exercise the product snapshot function: queued input must not masquerade
// as delivered input, and exact dt text must never pass through Number().
const snapshotFunction = currentWorkbench.slice(
  currentWorkbench.indexOf("function makeSnapshot("),
  currentWorkbench.indexOf("function buildFrame("),
);
const snapshotContext = vm.createContext({
  state: { nextInput: null }, activeRun: () => null,
  currentProject: () => ({}), inputSnapshotRequestBinding: () => ({}),
  activeRuntimeSourceText: () => "", readStateHash: () => "world",
});
vm.runInContext(`${snapshotFunction}\nglobalThis.makeSnapshot = makeSnapshot;`, snapshotContext);
const receiptHash = "a".repeat(64);
const exactDt = "0.016666666604578495025634765625";
const actualState = { input_receipt: {
  input_snapshot_sha256: receiptHash,
  input_snapshot: { madi: 7, input: { dt: exactDt, keys_pressed: 2,
    last_key_name: "ArrowLeft", pointer_x_i32: 3, pointer_y_i32: 4 } },
} };
const actualSnapshot = snapshotContext.makeSnapshot("test", {
  dt: 99, dtText: "99", keys: 999, lastKey: "wrong", px: 99, py: 99,
}, {}, actualState);
assert.equal(actualSnapshot.dtText, exactDt);
assert.equal(actualSnapshot.dt, undefined);
assert.equal(actualSnapshot.madi, 7);
assert.equal(actualSnapshot.keys, 2);
assert.equal(actualSnapshot.lastKey, "ArrowLeft");
assert.equal(actualSnapshot.pointer.x, 3);
assert.equal(actualSnapshot.snapshot_hash, receiptHash);
assert.equal(actualSnapshot.input_snapshot_sha256, receiptHash);
assert.throws(() => snapshotContext.makeSnapshot("test", {}, {}, {}), /E_INPUT_RECEIPT_MISSING/u);
const liveStepSource = currentWorkbench.slice(currentWorkbench.indexOf("async function commitNextLiveStep("));
assert(liveStepSource.indexOf("await stepWithHostStop(") < liveStepSource.indexOf("const snapshot = makeSnapshot("),
  "live/manual step must build its snapshot only after the committed WASM receipt exists");
assert.match(liveStepSource, /const snapshot = makeSnapshot\([\s\S]*?run,\s*wasmState,\s*\);/u);
const currentGotgan = fs.readFileSync(path.join(root, gotganRel), "utf8");
const before = rawCallInventory(baselineWorkbench, baselineGotgan);
const after = rawCallInventory(currentWorkbench, currentGotgan);

assert.equal(before.families.length, 12, "baseline raw WASM API family drift");
assert.equal(before.call_sites, 21, "baseline raw WASM call-site drift");
assert.deepEqual(after, { families: [], call_sites: 0 });
assert.match(currentWorkbench, /createRuntimeBridgeV1\(\)/u);
assert.doesNotMatch(currentWorkbench, /\bcreateWasmVm\s*\(/u);
assert.doesNotMatch(currentWorkbench, /\bwasmModule\b/u);
assert.doesNotMatch(currentWorkbench, /runtime\/wasm_vm_runtime\.js/u);

const diagnostic = Object.freeze({
  code: "E_RUNTIME_TEST",
  phase: "runtime",
  span: Object.freeze({ start: 3, end: 8 }),
  verdict: "rejected",
});
const state = Object.freeze({ state_hash: "state-hash-unchanged", value: 7 });
const adapterCalls = [];
const adapterSession = {
  updateLogic(sourceText) { adapterCalls.push(["updateLogic", sourceText]); return state; },
  getExecutionModeDecision() { adapterCalls.push(["getExecutionModeDecision"]); return diagnostic; },
  getConfiguredMadi() { adapterCalls.push(["getConfiguredMadi"]); return 17; },
  reset(options) { adapterCalls.push(["reset", options]); return state; },
  getStateJson() { adapterCalls.push(["getStateJson"]); return state; },
  step(options) { adapterCalls.push(["step", options]); return state; },
  setParam(options) { adapterCalls.push(["setParam", options]); return diagnostic; },
};
const bridgeAdapter = {
  createSession(options) { adapterCalls.push(["createSession", options]); return adapterSession; },
};
for (const operation of RUNTIME_BRIDGE_V1_OPERATIONS.filter((name) => name !== "createSession")) {
  bridgeAdapter[operation] = (payload) => {
    adapterCalls.push([operation, payload]);
    return payload;
  };
}

const bridge = new RuntimeBridgeV1({ adapter: bridgeAdapter });
assert.equal(bridge.schema, RUNTIME_BRIDGE_V1_SCHEMA);
const sessionOptions = { ddnSourceText: "1 보여주기.", cacheBust: "test" };
const session = await bridge.createSession(sessionOptions);
assert.equal(adapterCalls[0][1], sessionOptions, "session options identity changed");
assert.equal(await session.updateLogic("source"), state, "runtime result identity changed");
assert.equal(session.getExecutionModeDecision(), diagnostic, "diagnostic identity changed");
assert.equal(session.getConfiguredMadi(), 17);
assert.equal(await session.reset({ keepParams: false }), state);
assert.equal(await session.getStateJson(), state);
const stepOptions = { n: 1, input: null, sourceText: "source" };
assert.equal(await session.step(stepOptions), state, "state payload identity changed");
assert.equal(await session.setParam({ key: "g", value: 9.8 }), diagnostic);

const routeInput = Object.freeze({ schema: "route.input.v1", nested: Object.freeze({ value: 1 }) });
for (const operation of RUNTIME_BRIDGE_V1_OPERATIONS.filter((name) => name !== "createSession")) {
  assert.equal(await bridge[operation](routeInput), routeInput, `${operation}: payload identity changed`);
}

const routeError = Object.assign(new Error("runtime failed"), { diagnostic });
const failingBridge = new RuntimeBridgeV1({
  adapter: {
    createSession() { throw routeError; },
  },
});
await assert.rejects(
  () => failingBridge.createSession({}),
  (error) => error === routeError && error.diagnostic === diagnostic,
  "runtime error or diagnostic identity changed",
);

const wasmCalls = [];
const wasmModule = {
  wasm_export_input_tape_v1(receipts, rate) {
    wasmCalls.push(["tapeExport", receipts, rate]);
    return Uint8Array.from([1, 2, 3]);
  },
  wasm_run_summary_with_source_profile_tape_v1_json(source, sourcePath, profile, bytes) {
    wasmCalls.push(["tapeReplay", source, sourcePath, profile, Array.from(bytes)]);
    return JSON.stringify({ state_hash: "tape-world-hash" });
  },
  wasm_source_analysis_snapshot_json(json) {
    wasmCalls.push(["sourceAnalysis", json]);
    return JSON.stringify({ schema: "ddn.source_analysis_snapshot.candidate.v1", request: JSON.parse(json) });
  },
  wasm_project_prepare_link_json(json, target, strict, includeLock) {
    wasmCalls.push(["prepare", json, target, strict, includeLock]);
    return JSON.stringify({ kind: "prepare", source: JSON.parse(json) });
  },
  wasm_project_edit_plan_json(json) { wasmCalls.push(["edit", json]); return JSON.stringify({ kind: "edit" }); },
  wasm_project_source_sha256_json(json) { wasmCalls.push(["hash", json]); return "sha256-fixed"; },
  wasm_run_summary_with_source_profile_json(sourceText, sourcePath, requestedTicks, sourceProfileIdentity) {
    wasmCalls.push(["runSummary", sourceText, sourcePath, requestedTicks, sourceProfileIdentity]);
    return JSON.stringify({ sourceText, sourcePath, requestedTicks, sourceProfileIdentity });
  },
  wasm_run_summary_with_source_profile_input_receipts_json(sourceText, sourcePath, requestedTicks, sourceProfileIdentity, receiptsJson, paramsJson) {
    wasmCalls.push(["inputSummary", sourceText, sourcePath, requestedTicks, sourceProfileIdentity, receiptsJson, paramsJson]);
    return JSON.stringify({ input_receipts: JSON.parse(receiptsJson) });
  },
  wasm_canon_maegim_plan(sourceText) { wasmCalls.push(["canon", sourceText]); return JSON.stringify({ sourceText }); },
  wasm_gaji_extraction_plan_json(json) { wasmCalls.push(["extract", json]); return json; },
  wasm_local_gaji_artifact_json(json) { wasmCalls.push(["artifact", json]); return json; },
  wasm_gaji_lifecycle_plan_json(json) { wasmCalls.push(["lifecycle", json]); return json; },
  wasm_project_graph_json(json) { wasmCalls.push(["graph", json]); return JSON.stringify({ graph: JSON.parse(json) }); },
  wasm_project_symbols_json(json) { wasmCalls.push(["symbols", json]); return JSON.stringify({ symbols: JSON.parse(json) }); },
  wasm_project_gaji_lock_json(json) { wasmCalls.push(["lock", json]); return `lock:${json}`; },
  wasm_publish_local_gotgan_json(json) { wasmCalls.push(["publish", json]); return json; },
  wasm_search_local_gotgan_json(json) { wasmCalls.push(["search", json]); return json; },
};
const fakeHandle = { client: { wasmModule } };
const wasmAdapter = new WasmRuntimeAdapter({ createWasmVmImpl: async () => fakeHandle });
const adapterInput = { schema: "adapter.input.v1", value: 9 };
assert.deepEqual(await wasmAdapter.sourceAnalysis(adapterInput), {
  schema: "ddn.source_analysis_snapshot.candidate.v1",
  request: adapterInput,
});
const rawV25Source = "값:수 := 1.\n값 보여주기.\n";
assert.deepEqual(await wasmAdapter.runSummaryWithSourceProfile({
  sourceText: rawV25Source,
  sourcePath: "lesson.ddn",
  requestedTicks: 1,
  sourceProfileIdentity: "v1-core-v25",
}), {
  sourceText: rawV25Source,
  sourcePath: "lesson.ddn",
  requestedTicks: 1,
  sourceProfileIdentity: "v1-core-v25",
});
assert.deepEqual(wasmCalls.at(-1), [
  "runSummary",
  rawV25Source,
  "lesson.ddn",
  1,
  "v1-core-v25",
], "raw V25 source/profile must reach shared WASM unchanged");
const { wasm_run_summary_with_source_profile_json: omittedRunSummary, ...missingRunSummaryModule } = wasmModule;
const deliveredReceipts = [{ input_snapshot_sha256: "exact-input-hash", input_snapshot: { madi: 0 } }];
assert.deepEqual(await wasmAdapter.exportInputTapeV1({ inputReceipts: deliveredReceipts, madiHz: 60 }), Uint8Array.from([1, 2, 3]));
assert.deepEqual(wasmCalls.at(-1), ["tapeExport", JSON.stringify(deliveredReceipts), 60]);
assert.deepEqual(await wasmAdapter.runSummaryWithTapeV1({ sourceText: rawV25Source,
  sourcePath: "lesson.ddn", sourceProfileIdentity: "v1-core-v25", tapeBytes: Uint8Array.from([1, 2, 3]) }), { state_hash: "tape-world-hash" });
assert.deepEqual(wasmCalls.at(-1), ["tapeReplay", rawV25Source, "lesson.ddn", "v1-core-v25", [1, 2, 3]]);
assert.deepEqual(await wasmAdapter.runSummaryWithSourceProfile({
  sourceText: rawV25Source, sourcePath: "lesson.ddn", requestedTicks: 1,
  sourceProfileIdentity: "v1-core-v25", inputReceipts: deliveredReceipts,
}), { input_receipts: deliveredReceipts });
assert.deepEqual(wasmCalls.at(-1), ["inputSummary", rawV25Source, "lesson.ddn", 1,
  "v1-core-v25", JSON.stringify(deliveredReceipts), undefined], "actual receipt is forwarded unchanged");
const rawParams = { 중력: { raw_i64: "52828097741" } };
await wasmAdapter.runSummaryWithSourceProfile({ sourceText: rawV25Source, sourcePath: "lesson.ddn",
  requestedTicks: 1, sourceProfileIdentity: "v1-core-v25", inputReceipts: deliveredReceipts, paramOverrides: rawParams });
assert.equal(wasmCalls.at(-1)[6], JSON.stringify(rawParams), "shared verification must use the VM's actual raw parameters");
const { wasm_run_summary_with_source_profile_input_receipts_json: omittedInputSummary, ...missingInputModule } = wasmModule;
await assert.rejects(() => new WasmRuntimeAdapter({
  createWasmVmImpl: async () => ({ client: { wasmModule: missingInputModule } }),
}).runSummaryWithSourceProfile({
  sourceText: rawV25Source, sourcePath: "lesson.ddn", requestedTicks: 1,
  sourceProfileIdentity: "v1-core-v25", inputReceipts: deliveredReceipts,
}), /E_WASM_RUN_SUMMARY_REQUIRED/u, "missing input export never falls back to no-input summary");
const missingRunSummaryAdapter = new WasmRuntimeAdapter({
  createWasmVmImpl: async () => ({ client: { wasmModule: missingRunSummaryModule } }),
});
await assert.rejects(
  () => missingRunSummaryAdapter.runSummaryWithSourceProfile({
    sourceText: rawV25Source,
    sourcePath: "lesson.ddn",
    requestedTicks: 1,
    sourceProfileIdentity: "v1-core-v25",
  }),
  /E_WASM_RUN_SUMMARY_REQUIRED/u,
  "missing shared run-summary export must not fall back locally",
);
assert.deepEqual(await wasmAdapter.canonMaegimPlan("원문"), { sourceText: "원문" });
assert.deepEqual(await wasmAdapter.planGajiExtraction(adapterInput), adapterInput);
assert.equal(await wasmAdapter.projectSourceSha256(adapterInput), "sha256-fixed");
assert.deepEqual(await wasmAdapter.buildLocalGajiArtifact(adapterInput), adapterInput);
assert.deepEqual(await wasmAdapter.planGajiLifecycle(adapterInput), adapterInput);
assert.deepEqual(await wasmAdapter.projectGraph(adapterInput), { graph: adapterInput });
assert.deepEqual(await wasmAdapter.projectSymbols(adapterInput), { symbols: adapterInput });
assert.deepEqual(await wasmAdapter.prepareProjectLink(adapterInput, "앱", true, true), {
  kind: "prepare",
  source: adapterInput,
});
assert.deepEqual(await wasmAdapter.planProjectEdit(adapterInput), { kind: "edit" });
assert.equal(await wasmAdapter.projectGajiLock(adapterInput), `lock:${JSON.stringify(adapterInput)}`);
assert.deepEqual(await wasmAdapter.publishLocalGotgan(adapterInput), adapterInput);
assert.deepEqual(await wasmAdapter.searchLocalGotgan(adapterInput), adapterInput);
// Includes both receipt-only and receipt+raw-parameter summaries; no fallback call.
assert.equal(wasmCalls.length, 18, "adapter must perform one raw call per operation");

const storageRows = new Map();
const storage = {
  getItem(key) { return storageRows.get(key) ?? null; },
  setItem(key, value) { storageRows.set(key, value); },
};
const gotganRuntime = {
  async publishLocalGotgan(request) {
    return {
      schema: "ddn.local_gotgan_store.v1",
      artifacts: [...request.store.artifacts, request.artifact],
    };
  },
  async searchLocalGotgan(request) {
    return { rows: request.store.artifacts.filter((row) => row.gaji_id.includes(request.query)) };
  },
};
const artifact = { gaji_id: "물리", version: "1.0.0" };
assert.deepEqual(await publishToLocalGotgan(gotganRuntime, artifact, storage), {
  schema: "ddn.local_gotgan_store.v1",
  artifacts: [artifact],
});
assert.deepEqual(loadLocalGotganStore(storage).artifacts, [artifact]);
assert.deepEqual(await searchLocalGotgan(gotganRuntime, { query: "물" }, storage), { rows: [artifact] });
const imported = { gaji_id: "수학", version: "1.0.0" };
await importLocalGotganArtifact(gotganRuntime, JSON.stringify(imported), storage);
assert.deepEqual(loadLocalGotganStore(storage).artifacts, [artifact, imported]);

console.log(JSON.stringify({
  schema: "seamgrim.workbench.runtime_bridge_t1.runner_result.v1",
  baseline_commit: baselineCommit,
  bridge_schema: RUNTIME_BRIDGE_V1_SCHEMA,
  raw_wasm_api_families_before: before.families.length,
  raw_wasm_call_sites_before: before.call_sites,
  raw_wasm_api_families_after: after.families.length,
  raw_wasm_call_sites_after: after.call_sites,
  bridge_operations: RUNTIME_BRIDGE_V1_OPERATIONS,
  adapter_raw_operations_executed: wasmCalls.length,
  payload_identity_preserved: true,
  diagnostic_identity_preserved: true,
  state_hash_identity_preserved: true,
  error_identity_preserved: true,
  storage_behavior_preserved: true,
  result: "PASS",
}));
