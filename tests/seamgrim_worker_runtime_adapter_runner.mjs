import assert from "node:assert/strict";
import { WorkerRuntimeAdapter } from "../solutions/seamgrim_ui_mvp/ui/runtime/worker_runtime_adapter.js";

const workers = [];
class ControlledWorker {
  constructor() { this.requests = []; this.terminated = false; workers.push(this); }
  postMessage(request) { this.requests.push(request); }
  terminate() { this.terminated = true; }
  reply(request, value) { this.onmessage({ data: { id: request.id, ok: true, value } }); }
}
const adapter = new WorkerRuntimeAdapter({ workerFactory: () => new ControlledWorker() });
const creation = adapter.createSession({ ddnSourceText: "값 := 100.", setStatus() {} });
const first = workers[0];
assert.equal(first.requests[0].args[0].setStatus, undefined);
first.reply(first.requests[0], { sessionId: 1, metadata: { executionModeDecision: { mode: "oneshot" }, configuredMadi: 0 } });
const session = await creation;
assert.equal(session.getExecutionModeDecision().mode, "oneshot");
const update = session.updateLogic("값 := 101.");
const state = { state_hash: "authoritative", output: ["101"] };
first.reply(first.requests.at(-1), { result: state, metadata: { executionModeDecision: { mode: "live" }, configuredMadi: 10 } });
assert.deepEqual(await update, state);
assert.equal(session.getExecutionModeDecision().mode, "live");
assert.equal(session.getConfiguredMadi(), 10);

const blockedStep = session.step({ n: 1 });
const blockedAnalysis = adapter.sourceAnalysis({ source: "bad" });
const stepRejected = assert.rejects(blockedStep, { code: "E_EXECUTION_CANCELLED" });
const analysisRejected = assert.rejects(blockedAnalysis, { code: "E_EXECUTION_CANCELLED" });
adapter.cancelExecution();
await Promise.all([stepRejected, analysisRejected]);
assert.equal(first.terminated, true);
assert.equal(adapter.pending.size, 0);
await assert.rejects(session.step({ n: 1 }), { code: "E_EXECUTION_CANCELLED" });
assert.equal(workers.length, 1, "stale VM must not resurrect a worker");

const recovery = adapter.sourceAnalysis({ source: "valid" });
const second = workers[1];
first.reply(first.requests.at(-1), { stale: true });
assert.equal(adapter.pending.size, 1, "late terminated-worker replies must be ignored");
second.reply(second.requests[0], { recovered: true });
assert.deepEqual(await recovery, { recovered: true });
const failure = adapter.sourceAnalysis({ source: "fails" });
second.onmessage({ data: { id: second.requests.at(-1).id, ok: false, error: { code: "E_CHECK_TYPE_MISMATCH", message: "씨가 다릅니다." } } });
await assert.rejects(failure, { code: "E_CHECK_TYPE_MISMATCH", message: "씨가 다릅니다." });
console.log("SEAMGRIM_WORKER_ADAPTER_PASS cancellation/recovery/stale-reply/metadata/authoritative-payload");
const embedded = adapter.sourceAnalysis({ source: "typed error" });
second.onmessage({ data: { id: second.requests.at(-1).id, ok: false,
  error: { code: null, message: "[E_RUNTIME_TYPE_MISMATCH] 핀=사용량 기대=수 실제=글" } } });
await assert.rejects(embedded, { code: "E_RUNTIME_TYPE_MISMATCH" });
const plain = adapter.sourceAnalysis({ source: "plain error" });
second.onmessage({ data: { id: second.requests.at(-1).id, ok: false,
  error: { code: null, message: "0으로 나눌 수 없습니다" } } });
await assert.rejects(plain, error => error.code === null && error.message === "0으로 나눌 수 없습니다");
