import { WasmRuntimeAdapter } from "./runtime_bridge.js";
import { requireSafeWorkbenchResult } from "./execution_safety.js";

const adapter = new WasmRuntimeAdapter();
const sessions = new Map();
const logicalSessions = new Map();
const logicalOperations = new Map([
  ["logicalStep", "step"], ["logicalSummary", "summary"],
  ["logicalSeal", "seal"], ["logicalAbort", "abort"], ["logicalClose", "close"],
  ["logicalStopRecordingV3", "stopRecording"],
  ["logicalRegisterInputProducer", "registerInputProducer"],
  ["logicalFinishPending", "finishPending"], ["logicalFinishNotice", "finishNotice"],
  ["logicalPreviewFinishPending", "previewFinishPending"],
  ["logicalResultIdentity", "resultIdentity"],
  ["logicalExportSavedProjectArtifact", "exportSavedProjectArtifact"],
  ["logicalExportProjectCheckpoint", "exportProjectCheckpoint"],
]);
const sessionOperations = new Set(["updateLogic", "reset", "getStateJson", "step", "setParam", "exportSavedProjectArtifact", "saveNativeCheckpoint", "close"]);
const adapterOperations = new Set([
  "authoredProjectOperation",
  "compareGeneralOriginalRecording",
  "canonMaegimPlan", "sourceAnalysis", "moduleDirectiveAnalysis", "sharedCompletion",
  "runSummaryWithSourceProfile", "planGajiExtraction", "projectSourceSha256",
  "exportInputTapeV1", "runSummaryWithTapeV1",
  "replayLogicalInput",
  "recordFiniteInput",
  "bindLogicalSource",
  "bindManipulationSource", "replayManipulationInput",
  "manipulationValueRaw",
  "buildLocalGajiArtifact", "planGajiLifecycle", "projectGraph", "projectSymbols",
  "prepareProjectLink", "planProjectEdit", "projectGajiLock", "publishLocalGotgan", "searchLocalGotgan",
]);
let nextSessionId = 1;
let queue = Promise.resolve();

function metadata(session) {
  return { executionModeDecision: session.getExecutionModeDecision(), configuredMadi: session.getConfiguredMadi(),nativeSourceBound:session.hasNativeSource?.()===true,nativeSourceSelection:session.nativeSourceSelection?.()??null };
}

async function dispatch({ operation, args, sessionId }) {
  if (operation === "createLogicalSession" || operation === "createManipulationSession") {
    const session = await adapter[operation](args[0]);
    const id = nextSessionId++;
    logicalSessions.set(id, session);
    return { sessionId: id };
  }
  if (logicalOperations.has(operation)) {
    const session = logicalSessions.get(sessionId);
    if (!session) throw new Error("E_RUNTIME_SESSION_INVALID");
    const result = await session[logicalOperations.get(operation)](...args);
    if (operation === "logicalClose") logicalSessions.delete(sessionId);
    return result;
  }
  if (operation === "createSession") {
    const session = await adapter.createSession(args[0]);
    const id = nextSessionId++;
    sessions.set(id, session);
    return { sessionId: id, metadata: metadata(session) };
  }
  if (sessionId !== null) {
    const session = sessions.get(sessionId);
    if (!session || !sessionOperations.has(operation)) throw new Error("E_RUNTIME_SESSION_INVALID");
    const result = await session[operation](...args);
    if (operation === "close") sessions.delete(sessionId);
    if (operation === "step") requireSafeWorkbenchResult(result);
    return { result, metadata: metadata(session) };
  }
  if (!adapterOperations.has(operation)) throw new Error("E_RUNTIME_OPERATION_INVALID");
  const result = await adapter[operation](...args);
  return operation === "runSummaryWithSourceProfile" ? requireSafeWorkbenchResult(result) : result;
}

self.onmessage = ({ data }) => {
  // One VM transaction at a time, including asynchronous module loading.
  queue = queue.then(async () => {
    try { self.postMessage({ id: data.id, ok: true, value: await dispatch(data) }); }
    catch (error) {
      self.postMessage({ id: data.id, ok: false, error: { message: String(error?.message ?? error), code: error?.code ?? null } });
    }
  });
};
