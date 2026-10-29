// Real product Worker dispatch and compiled WASM. Node supplies only the Worker host.
import { Worker, isMainThread, parentPort, workerData } from 'node:worker_threads';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';
import fs from 'node:fs';
import assert from 'node:assert/strict';

if (!isMainThread) {
  const api = createRequire(import.meta.url)(workerData.wasm);
  let queue = [];
  parentPort.on('message', data => {
    if (globalThis.self?.onmessage) self.onmessage({ data }); else queue.push(data);
  });
  globalThis.self = { postMessage: value => parentPort.postMessage(value) };
  const { WasmRuntimeAdapter } = await import(pathToFileURL(workerData.runtime + '/runtime_bridge.js'));
  WasmRuntimeAdapter.prototype.ensureProjectModule = async () => api;
  await import(pathToFileURL(workerData.runtime + '/runtime_worker.js'));
  for (const data of queue) self.onmessage({ data });
  queue = [];
} else {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const wasm = process.env.V4_WASM;
  if (!wasm) throw new Error('V4_WASM must identify the fixed compiled Node WASM module');
  const runtime = root + '/solutions/seamgrim_ui_mvp/ui/runtime';
  const { WorkerRuntimeAdapter } = await import(pathToFileURL(runtime + '/worker_runtime_adapter.js'));
  const { RuntimeBridgeV1 } = await import(pathToFileURL(runtime + '/runtime_bridge.js'));
  let latest;
  const adapter = new WorkerRuntimeAdapter({ workerFactory: () => {
    const worker = new Worker(new URL(import.meta.url), { workerData: { runtime, wasm } });
    const wrapper = { postMessage: data => worker.postMessage(data), terminate: () => worker.terminate() };
    worker.on('message', data => wrapper.onmessage?.({ data }));
    worker.on('error', error => wrapper.onerror?.({ message: error.message }));
    latest = wrapper;
    return wrapper;
  } });
  const bridge = new RuntimeBridgeV1({ adapter });
  const raw = fs.readFileSync(root + '/tool/tests/fixtures/v4_linear_steps/정상.ddn', 'utf8');
  const rows = [], reports = {};
  try {
    const original = await bridge.verifyAssumptionSource(raw, '수업.ddn');
    assert.equal(original.unconditional_proof_issued, false);
    assert.equal(original.steps, undefined);
    rows.push('existing default response preserved');
    reports.normal = await bridge.verifyAssumptionSource(raw, '수업.ddn', { includeSteps: true });
    assert.equal(reports.normal.status, 'accepted');
    assert.deepEqual(reports.normal.definitions[0].steps.map(s => s.status), Array(4).fill('checked'));
    rows.push('actual Worker exposes four replayed steps');
    const bad = raw.replace('오른=6', '오른=5');
    reports.intermediate = await bridge.verifyAssumptionSource(bad, '수업.ddn', { includeSteps: true });
    assert.deepEqual(reports.intermediate.definitions[0].steps.map(s => s.status), ['checked', 'failed', 'not_checked', 'not_checked']);
    assert.equal(reports.intermediate.definitions[0].steps[2].equality, null);
    await assert.rejects(bridge.verifyAssumptionSource(bad, '수업.ddn'), /E_PROOF_ALGEBRA_REWRITE/);
    rows.push('failure observation never publishes unchecked equalities; default still rejects');
    await assert.rejects(bridge.verifyAssumptionSource(raw, '수업.ddn', { includeSteps: 'true' }), /includeSteps/);
    rows.push('invalid observation option rejected');
    const stale = latest;
    const pending = bridge.verifyAssumptionSource(raw, '수업.ddn', { includeSteps: true });
    const rejected = assert.rejects(pending, /중지/);
    adapter.cancelExecution();
    await rejected;
    stale.onmessage({ data: { id: adapter.nextId - 1, ok: true, value: { status: 'accepted', forged: true } } });
    assert.equal(adapter.pending.size, 0);
    rows.push('cancel and stale generation response remain rejected');
    reports.fresh = await bridge.verifyAssumptionSource(raw + '\n', '새수업.ddn', { includeSteps: true });
    assert.notEqual(reports.fresh.source_sha256, reports.normal.source_sha256);
    assert.equal(reports.fresh.source_path, '새수업.ddn');
    rows.push('fresh generation binds actual source and path');
    const result = { pass: true, checks: rows, reports, scope: 'Node worker_threads host; actual product adapter/dispatch/WASM. Chrome NOT_RUN.' };
    if (process.env.V4_WORKER_OUTPUT) fs.writeFileSync(process.env.V4_WORKER_OUTPUT, JSON.stringify(result, null, 2));
    console.log(JSON.stringify({ pass: true, checks: rows }));
  } finally { adapter.cancelExecution(); }
}
