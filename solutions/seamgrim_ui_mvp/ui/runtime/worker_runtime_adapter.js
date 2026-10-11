const SESSION_METHODS = ["updateLogic", "reset", "getStateJson", "step", "setParam", "exportSavedProjectArtifact", "close"];
const ADAPTER_METHODS = [
  "authoredProjectOperation",
  "bindLogicalSource",
  "canonMaegimPlan", "sourceAnalysis", "moduleDirectiveAnalysis", "sharedCompletion",
  "runSummaryWithSourceProfile", "planGajiExtraction", "projectSourceSha256",
  "exportInputTapeV1", "runSummaryWithTapeV1",
  "replayLogicalInput",
  "buildLocalGajiArtifact", "planGajiLifecycle", "projectGraph", "projectSymbols",
  "prepareProjectLink", "planProjectEdit", "projectGajiLock", "publishLocalGotgan",
  "searchLocalGotgan",
];

function runtimeError(message, code = "E_EXECUTION_CANCELLED") {
  const error = new Error(message);
  error.code = code;
  return error;
}

// Every WASM entry (including constructor, reset and receipt generation) runs
// outside the UI thread. Cancellation terminates that execution context; it
// never substitutes a partial JS result for an authoritative WASM frame.
export class WorkerRuntimeAdapter {
  constructor({ workerFactory = () => new Worker(new URL("./runtime_worker.js", import.meta.url), { type: "module" }) } = {}) {
    this.workerFactory = workerFactory;
    this.worker = null;
    this.generation = 0;
    this.nextId = 1;
    this.pending = new Map();
  }

  ensureWorker() {
    if (this.worker) return this.worker;
    const worker = this.workerFactory();
    this.worker = worker;
    worker.onmessage = ({ data }) => {
      if (this.worker !== worker) return;
      const pending = this.pending.get(data?.id);
      if (!pending) return;
      this.pending.delete(data.id);
      if (data.ok) pending.resolve(data.value);
      else {
        const message = String(data.error?.message ?? "실행 작업에 실패했습니다.");
        // Preserve product diagnostics. A delivered product error is not a Worker failure.
        const code = data.error?.code ?? message.match(/\b(E_[A-Z0-9_]+)\b/u)?.[1] ?? null;
        pending.reject(runtimeError(message, code));
      }
    };
    worker.onerror = (event) => {
      if (this.worker !== worker) return;
      this.cancelExecution(runtimeError(String(event.message ?? "실행 작업을 불러오지 못했습니다."), "E_RUNTIME_WORKER"));
    };
    return worker;
  }

  request(operation, args = [], sessionId = null, expectedGeneration = this.generation) {
    if (expectedGeneration !== this.generation) return Promise.reject(runtimeError("중지된 실행입니다. 다시 실행해 주세요."));
    return new Promise((resolve, reject) => {
      let worker;
      try { worker = this.ensureWorker(); } catch (error) { reject(error); return; }
      const id = this.nextId++;
      this.pending.set(id, { resolve, reject });
      try { worker.postMessage({ id, operation, args, sessionId }); }
      catch (error) { this.pending.delete(id); reject(error); }
    });
  }

  cancelExecution(reason = "실행을 중지했습니다.") {
    const error = reason instanceof Error ? reason : runtimeError(String(reason));
    const worker = this.worker;
    this.worker = null;
    this.generation += 1;
    worker?.terminate();
    for (const pending of this.pending.values()) pending.reject(error);
    this.pending.clear();
  }

  async createSession(options = {}) {
    const generation = this.generation;
    // Presentation callbacks cannot cross structured-clone boundaries.
    const transferable = Object.fromEntries(Object.entries(options).filter(([, value]) => typeof value !== "function"));
    const created = await this.request("createSession", [transferable], null, generation);
    let metadata = created.metadata;
    const session = {
      getExecutionModeDecision: () => metadata.executionModeDecision,
      getConfiguredMadi: () => metadata.configuredMadi,
    };
    for (const operation of SESSION_METHODS) {
      session[operation] = async (...args) => {
        const result = await this.request(operation, args, created.sessionId, generation);
        metadata = result.metadata;
        return result.result;
      };
    }
    return session;
  }

  async createLogicalSession(options) {
    const generation = this.generation;
    const created = await this.request("createLogicalSession", [options], null, generation);
    const operations = { step:"logicalStep", summary:"logicalSummary", seal:"logicalSeal", abort:"logicalAbort", close:"logicalClose" };
    return Object.fromEntries(Object.entries(operations).map(([method, operation]) => [method,
      (...args) => this.request(operation, args, created.sessionId, generation),
    ]));
  }
}

for (const operation of ADAPTER_METHODS) {
  WorkerRuntimeAdapter.prototype[operation] = function (...args) { return this.request(operation, args); };
}
