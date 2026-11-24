const SESSION_METHODS = ["updateLogic", "configureProjectAssetsBinary", "reset", "getStateJson", "step", "setParam", "exportSavedProjectArtifact", "saveNativeCheckpoint", "close"];
const ADAPTER_METHODS = [
  "authoredProjectOperation",
  "observeMathNotebook",
  "observeAlgebraProject",
  "compareGeneralOriginalRecording",
  "bindLogicalSource",
  "verifyAssumptionSource",
  "canonMaegimPlan", "sourceAnalysis", "moduleDirectiveAnalysis", "sharedCompletion",
  "runSummaryWithSourceProfile", "planGajiExtraction", "projectSourceSha256",
  "exportInputTapeV1", "runSummaryWithTapeV1",
  "replayLogicalInput",
  "recordFiniteInput",
  "bindManipulationSource", "replayManipulationInput",
  "manipulationValueRaw",
  "buildLocalGajiArtifact", "planGajiLifecycle", "projectGraph", "projectSymbols",
  "prepareProjectLink", "planProjectEdit", "projectGajiLock", "publishLocalGotgan",
  "searchLocalGotgan",
];

function runtimeError(message, code = "E_EXECUTION_CANCELLED") {
  const error = new Error(message);
  error.code = code;
  return error;
}

// This existing API throws Rust ExecutionDiagnostic JSON as its machine payload.
// Decode that payload only for its captured request, never positions from prose.
function machineDiagnostic(message,wireCode) {
  let value;try{value=JSON.parse(message);}catch{return null;}
  if(!value||typeof value!=='object'||Array.isArray(value)
      ||typeof value.code!=='string'||!/^E_[A-Z0-9_]+$/u.test(value.code)
      ||typeof value.message!=='string'||typeof value.phase!=='string'
      ||typeof value.source_path!=='string'||!Object.hasOwn(value,'source_span')
      ||(wireCode!==null&&wireCode!==undefined&&wireCode!==value.code))return null;
  return value;
}
function actualSourceDiagnostic(value,context) {
  if(!value||!context?.has(value.source_path))return null;
  const span=value.source_span;
  if(span!==null&&(!span||!Number.isSafeInteger(span.start)||!Number.isSafeInteger(span.end)
      ||span.start<0||span.end<span.start||span.end>context.get(value.source_path)))return null;
  return value;
}
function capturedDiagnosticContext(operation,request) {
  if(operation==='runSummaryWithSourceProfile'&&typeof request?.sourceText==='string')
    return new Map([[String(request.sourcePath??'lesson.ddn'),new TextEncoder().encode(request.sourceText).byteLength]]);
  let snapshot;
  try {
    if(operation==='createSession'&&typeof request?.projectRuntimeArtifactText==='string'){
      const artifact=JSON.parse(request.projectRuntimeArtifactText);
      if(artifact.source_profile!==request.sourceProfileIdentity)return null;
      snapshot=artifact.source_snapshot;
    } else if(operation==='authoredProjectOperation'){
      const op=request?.operation;
      if(!['create','update','source_probe','source_prepare','public_callable_source_prepare','public_callable_source_run','linked_callable_source_run'].includes(op))return null;
      const input=JSON.parse(request.text);
      if(op==='public_callable_source_prepare'||op==='linked_callable_source_run'){
        snapshot=JSON.parse(input.consumer_project_json_utf8).source_snapshot;
      }else if(op==='public_callable_source_run'){
        const artifact=JSON.parse(input.source_artifact_utf8);
        if(artifact.schema!=='ddn.source_only_project_artifact.v1')return null;
        const payload=JSON.parse(artifact.project_payload_utf8);
        if(!['ddn.saved_runtime_project.v2','ddn.saved_runtime_project.v4'].includes(payload.schema))return null;
        snapshot=JSON.parse(payload.authored_project_json_utf8).source_snapshot;
      }else{
        snapshot=['create','update'].includes(op)?input.source_snapshot:JSON.parse(input.artifact_json_utf8).source_snapshot;
      }
    }
  }catch{return null;}
  if(!Array.isArray(snapshot?.files)||snapshot.files.length===0)return null;
  const rows=new Map();
  for(const file of snapshot.files){
    if(typeof file?.path!=='string'||!file.path||typeof file.text!=='string'||rows.has(file.path))return null;
    rows.set(file.path,new TextEncoder().encode(file.text).byteLength);
  }
  // This is request custody only. The Rust producer still admits the actual
  // profile, source, Definition/Pin, error and span; JSON is not a permit.
  return rows;
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
        const machine=machineDiagnostic(message,data.error?.code);
        const diagnostic=actualSourceDiagnostic(machine,pending.diagnosticContext);
        const code = machine?.code ?? data.error?.code ?? message.match(/\b(E_[A-Z0-9_]+)\b/u)?.[1] ?? null;
        const error=runtimeError(message,code);
        if(machine)error.diagnosticMessage=machine.message;
        if(diagnostic){
          error.diagnostic=diagnostic;error.source_path=diagnostic.source_path;
          error.source_span=diagnostic.source_span;error.phase=diagnostic.phase;
          error.cause=diagnostic.cause??null;
        }
        pending.reject(error);
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
      const request=args[0];
      const diagnosticContext=capturedDiagnosticContext(operation,request);
      this.pending.set(id, { resolve, reject, diagnosticContext });
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
      hasNativeSource: () => metadata.nativeSourceBound === true,
      nativeSourceSelection: () => metadata.nativeSourceSelection ?? null,
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

  async createLogicalSession(options, operationName = "createLogicalSession") {
    const generation = this.generation;
    const created = await this.request(operationName, [options], null, generation);
    const operations = { step:"logicalStep", summary:"logicalSummary", seal:"logicalSeal", stopRecording:"logicalStopRecordingV3", abort:"logicalAbort", close:"logicalClose", exportSavedProjectArtifact:"logicalExportSavedProjectArtifact", exportProjectCheckpoint:"logicalExportProjectCheckpoint" };
    Object.assign(operations,{registerInputProducer:"logicalRegisterInputProducer",finishPending:"logicalFinishPending",finishNotice:"logicalFinishNotice",previewFinishPending:"logicalPreviewFinishPending",resultIdentity:"logicalResultIdentity"});
    return Object.fromEntries(Object.entries(operations).map(([method, operation]) => [method,
      (...args) => this.request(operation, args, created.sessionId, generation),
    ]));
  }
  createManipulationSession(options) {return this.createLogicalSession(options,"createManipulationSession");}
}

for (const operation of ADAPTER_METHODS) {
  WorkerRuntimeAdapter.prototype[operation] = function (...args) { return this.request(operation, args); };
}
