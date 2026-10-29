import assert from "node:assert/strict";
import { WasmVmHandle } from "../solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js";
import { DdnWasmVmClient } from "../solutions/seamgrim_ui_mvp/ui/wasm_ddn_wrapper.js";
import { WorkerRuntimeAdapter } from "../solutions/seamgrim_ui_mvp/ui/runtime/worker_runtime_adapter.js";

const calls = [];
const client = {
  updateLogicWithSourceProfile(source, profile) { calls.push(["update", source, profile]); },
  configureProjectEntry(name) { calls.push(["entry", name]); },
  parseWarningsParsed: () => [], configuredMadi: () => 0,
  executionModeDecisionParsed: () => ({ mode: "oneshot", authority: "parsed_program" }),
  getStateParsed: () => ({ state_hash: "test" }),
  resetParsed: () => ({ state_hash: "reset" }),
};
const handle = new WasmVmHandle({ loader: { ensure: async () => client }, sourceProfileIdentity: "v1-core-v25", projectEntryName: "실행" });
await handle.updateLogic("실행:움직씨 := { 1 보여주기. }.");
await handle.reset();
await handle.updateLogic("실행:움직씨 := { 2 보여주기. }.");
assert.deepEqual(calls.map((row) => row[0]), ["update", "entry", "update", "entry"]);
assert.deepEqual(calls.filter((row) => row[0] === "entry"), [["entry", "실행"], ["entry", "실행"]]);
const noEntryApi = { ...client, configureProjectEntry: undefined };
await assert.rejects(new WasmVmHandle({ loader: { ensure: async () => noEntryApi }, sourceProfileIdentity: "v1-core-v25", projectEntryName: "실행" }).updateLogic("실행:움직씨 := {}."), /E_WASM_PROJECT_ENTRY_API_MISSING/);
await new WasmVmHandle({ loader: { ensure: async () => noEntryApi }, sourceProfileIdentity: "v1-core-v25" }).updateLogic("1 보여주기.");
let received = "";
new DdnWasmVmClient({ configure_project_entry(name) { received = name; } }).configureProjectEntry("정확한이름");
assert.equal(received, "정확한이름");
assert.throws(() => new DdnWasmVmClient({}).configureProjectEntry("실행"), /E_WASM_PROJECT_ENTRY_API_MISSING/);
let request;
const worker = { postMessage(value) { request = value; }, terminate() {} };
const adapter = new WorkerRuntimeAdapter({ workerFactory: () => worker });
const creation = adapter.createSession({ ddnSourceText: "source", projectEntryName: "실행", sourceProfileIdentity: "v1-core-v25" });
assert.equal(request.args[0].projectEntryName, "실행");
worker.onmessage({ data: { id: request.id, ok: true, value: { sessionId: 1, metadata: {} } } });
await creation;
adapter.cancelExecution();
console.log("WO03_PROJECT_ENTRY_PASS rebind_each_update=1 standalone_unchanged=1 missing_api_closed=1 worker_transport=1");
