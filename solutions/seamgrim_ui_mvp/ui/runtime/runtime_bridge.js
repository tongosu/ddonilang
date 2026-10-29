import { createWasmVm } from "./wasm_vm_runtime.js";
import { WorkerRuntimeAdapter } from "./worker_runtime_adapter.js";

export const RUNTIME_BRIDGE_V1_SCHEMA = "seamgrim.runtime_bridge.v1";

export const RUNTIME_BRIDGE_V1_OPERATIONS = Object.freeze([
  "authoredProjectOperation",
  "observeMathNotebook",
  "compareGeneralOriginalRecording",
  "createSession",
  "createLogicalSession",
  "createManipulationSession",
  "bindManipulationSource",
  "replayManipulationInput",
  "manipulationValueRaw",
  "bindLogicalSource",
  "replayLogicalInput",
  "recordFiniteInput",
  "sourceAnalysis",
  "verifyAssumptionSource",
  "sharedCompletion",
  "runSummaryWithSourceProfile",
  "exportInputTapeV1",
  "runSummaryWithTapeV1",
  "canonMaegimPlan",
  "planGajiExtraction",
  "projectSourceSha256",
  "buildLocalGajiArtifact",
  "planGajiLifecycle",
  "projectGraph",
  "projectSymbols",
  "prepareProjectLink",
  "planProjectEdit",
  "projectGajiLock",
  "publishLocalGotgan",
  "searchLocalGotgan",
]);

function requireExport(wasmModule, name, message = `${name} export 누락`) {
  const operation = wasmModule?.[name];
  if (typeof operation !== "function") throw new Error(message);
  return operation.bind(wasmModule);
}

class RuntimeSessionV1 {
  #adapterSession;

  constructor(adapterSession) {
    this.#adapterSession = adapterSession;
  }

  updateLogic(sourceText) {
    return this.#adapterSession.updateLogic(sourceText);
  }

  getExecutionModeDecision() {
    return this.#adapterSession.getExecutionModeDecision();
  }

  getConfiguredMadi() {
    return this.#adapterSession.getConfiguredMadi();
  }

  reset(options) {
    return this.#adapterSession.reset(options);
  }

  getStateJson() {
    return this.#adapterSession.getStateJson();
  }

  exportSavedProjectArtifact(parentHex, revisionHex) {
    return this.#adapterSession.exportSavedProjectArtifact(parentHex, revisionHex);
  }
  hasNativeSource() { return this.#adapterSession.hasNativeSource?.() === true; }
  nativeSourceSelection() { return this.#adapterSession.nativeSourceSelection?.() ?? null; }
  saveNativeCheckpoint(authoredText) { return this.#adapterSession.saveNativeCheckpoint(authoredText); }

  close() { return this.#adapterSession.close(); }

  step(options) {
    return this.#adapterSession.step(options);
  }

  setParam(options) {
    return this.#adapterSession.setParam(options);
  }
}

export class WasmRuntimeAdapter {
  constructor({ createWasmVmImpl = createWasmVm } = {}) {
    if (typeof createWasmVmImpl !== "function") {
      throw new TypeError("createWasmVmImpl must be a function");
    }
    this.createWasmVmImpl = createWasmVmImpl;
    this.projectModulePromise = null;
  }

  async createSession(options) {
    return this.createWasmVmImpl(options);
  }

  async createLogicalSession({ sourceText, bindingJson, authoredProjectJson = null, savedCheckpoint = null }) {
    if (typeof bindingJson !== "string" ||
        (authoredProjectJson === null ? typeof sourceText !== "string" : typeof authoredProjectJson !== "string")) {
      throw new Error("E_LOGICAL_INPUT_BINDING: 원문과 명시 입력 결속이 필요합니다.");
    }
    const module = await this.ensureProjectModule();
    requireExport(module, "DdnLogicalInputVm", "E_LOGICAL_INPUT_RUNTIME_MISSING: 새 논리 입력 실행기가 필요합니다.");
    const Constructor = module.DdnLogicalInputVm;
    const vm = savedCheckpoint !== null ? requireExport(Constructor, "from_saved_project_checkpoint",
      "E_PROJECT_RUNTIME_LOGICAL: 논리 입력 저장 복원 실행기가 필요합니다.")(
        savedCheckpoint.artifact_text, new Uint8Array(savedCheckpoint.checkpoint_bytes),
        new Uint8Array(savedCheckpoint.parent_bytes), new Uint8Array(savedCheckpoint.revision_bytes), bindingJson) :
      authoredProjectJson === null ? new Constructor(sourceText, bindingJson) :
      requireExport(Constructor, "from_authored_project", "E_PROJECT_RUNTIME_LOGICAL: 프로젝트 논리 실행기가 필요합니다.")(authoredProjectJson, bindingJson);
    if (typeof vm.view_json !== 'function') {
      vm.free();
      throw new Error('E_LOGICAL_INPUT_VIEW_UNSUPPORTED: 실행기의 그림 투영 계약이 없습니다.');
    }
    return {
      step: snapshot => {
        const result = JSON.parse(vm.step_json(JSON.stringify(snapshot)));
        result.view_meta = JSON.parse(vm.view_json());
        return result;
      },
      summary: ({includeView=false}={}) => {
        const result=JSON.parse(vm.summary_json());
        if(includeView) result.view_meta=JSON.parse(vm.view_json());
        return result;
      },
      seal: () => JSON.parse(vm.seal_json()),
      registerInputProducer: () => JSON.parse(requireExport(vm,'register_input_producer_json','E_MISSING_PENDING_SUPPLY: 입력 생산자 실행기가 필요합니다.').call(vm)),
      finishPending: packet => JSON.parse(requireExport(vm,'finish_pending_json','E_MISSING_PENDING_SUPPLY: queue 종료 실행기가 필요합니다.').call(vm,JSON.stringify(packet))),
      previewFinishPending: packet => JSON.parse(requireExport(vm,'preview_finish_pending_json','E_INPUT_FINISH_NOT_READY: 종료 사전 확인 실행기가 필요합니다.').call(vm,JSON.stringify(packet))),
      resultIdentity: () => JSON.parse(requireExport(vm,'result_identity_json','E_INPUT_FINISH_NOT_READY: 원 형식 결과 확인이 필요합니다.').call(vm)),
      finishNotice: () => JSON.parse(requireExport(vm,'finish_notice_json','E_MISSING_PENDING_SUPPLY: 종료 확인 실행기가 필요합니다.').call(vm)),
      stopRecording: () => JSON.parse(requireExport(vm,'stop_recording_v3_json','E_INPUT_V3_UNSUPPORTED: 통합 기록 실행기가 필요합니다.').call(vm)),
      exportSavedProjectArtifact: (parent, revision) =>
        requireExport(vm, "export_saved_project", "E_PROJECT_RUNTIME_LOGICAL: 프로젝트 저장 기능이 필요합니다.").call(vm, parent, revision),
      exportProjectCheckpoint: (parent, revision) =>
        requireExport(vm, "export_project_checkpoint", "E_PROJECT_RUNTIME_LOGICAL: 입력 이력 저장 기능이 필요합니다.").call(vm, parent, revision),
      abort: message => vm.abort_input(String(message)),
      close: () => vm.free(),
    };
  }

  async bindLogicalSource({ sourceText, bindingJson, authoredProjectJson = null }) {
    const module = await this.ensureProjectModule();
    const Constructor = module.DdnLogicalInputVm;
    const bind = requireExport(Constructor, authoredProjectJson === null ? "bind_source_json" : "bind_authored_project_json", "E_LOGICAL_INPUT_BIND_SOURCE_UNSUPPORTED: 새 원문 결속 기능이 필요합니다.");
    return bind(authoredProjectJson === null ? sourceText : authoredProjectJson, bindingJson);
  }

  async createManipulationSession({sourceText,bindingJson}) {
    const module=await this.ensureProjectModule();
    const Constructor=requireExport(module,'DdnManipulationVm','E_MANIPULATION_RUNTIME_MISSING: 조작 입력 실행기가 필요합니다.');
    const vm=new Constructor(sourceText,bindingJson);
    return {step:snapshot=>{const result=JSON.parse(vm.step_json(JSON.stringify(snapshot)));result.view_meta=JSON.parse(vm.view_json());return result;},
      summary:()=>JSON.parse(vm.summary_json()),seal:()=>JSON.parse(vm.seal_json()),
      registerInputProducer:()=>JSON.parse(requireExport(vm,'register_input_producer_json','E_MISSING_PENDING_SUPPLY: 조작 입력 생산자 실행기가 필요합니다.').call(vm)),
      finishPending:packet=>JSON.parse(requireExport(vm,'finish_pending_json','E_MISSING_PENDING_SUPPLY: 조작 queue 종료 실행기가 필요합니다.').call(vm,JSON.stringify(packet))),
      finishNotice:()=>JSON.parse(requireExport(vm,'finish_notice_json','E_MISSING_PENDING_SUPPLY: 조작 종료 확인 실행기가 필요합니다.').call(vm)),
      stopRecording:()=>JSON.parse(requireExport(vm,'stop_recording_v3_json','E_INPUT_V3_UNSUPPORTED: 통합 기록 실행기가 필요합니다.').call(vm)),
      abort:message=>vm.abort_input(String(message)),close:()=>vm.free()};
  }
  async bindManipulationSource({sourceText,bindingJson}) {
    const module=await this.ensureProjectModule();
    return requireExport(module.DdnManipulationVm,'bind_source_json')(sourceText,bindingJson);
  }
  async manipulationValueRaw(text) {
    const module = await this.ensureProjectModule();
    return requireExport(module, 'manipulation_value_raw')(String(text));
  }
  async replayManipulationInput({sourceText,tapeBytes,bindingJson,allowPartial=false}) {
    if(typeof allowPartial!=='boolean')throw new Error('E_INPUT_REPLAY_MODE: 부분 재생 선택은 참·거짓이어야 합니다.');
    const module=await this.ensureProjectModule();
    return JSON.parse(requireExport(module,'manipulation_input_replay_v3')(sourceText,new Uint8Array(tapeBytes),bindingJson,allowPartial));
  }

  async recordFiniteInput({sourceText,requestJson,bindingJson,authoredProjectJson=null,manipulation=false}) {
    if(typeof requestJson!=='string'||typeof bindingJson!=='string')throw new Error('E_INPUT_TAPE_V3: 입력 요청과 독립적인 현재 입력 계약이 필요합니다.');
    if(manipulation&&authoredProjectJson!==null)throw new Error('E_MANIPULATION_PROJECT_UNSUPPORTED: 여러 파일 조작 입력은 지원하지 않습니다.');
    const module=await this.ensureProjectModule();
    const name=manipulation?'manipulation_input_record_v3':authoredProjectJson===null?'logical_input_record_v3':'authored_logical_input_record_v3';
    // Pass the original JSON text to Rust. Do not reparse large integers or
    // promote a live accepted prefix to a newly invented finite request.
    const record=JSON.parse(requireExport(module,name)(authoredProjectJson??sourceText,requestJson));
    const options={sourceText,tapeBytes:record.tape_bytes,bindingJson,authoredProjectJson};
    // The file's own binding is not its admission authority. Re-admit against
    // the separately selected current project/source and input contract.
    const summary=await (manipulation?this.replayManipulationInput(options):this.replayLogicalInput(options));
    return {tape_bytes:record.tape_bytes,summary};
  }

  async replayLogicalInput({ sourceText, tapeBytes, bindingJson, authoredProjectJson = null, allowPartial = false }) {
    if(typeof allowPartial!=='boolean')throw new Error('E_INPUT_REPLAY_MODE: 부분 재생 선택은 참·거짓이어야 합니다.');
    const module = await this.ensureProjectModule();
    const replay = requireExport(module, authoredProjectJson === null ? "logical_input_replay_v3" : "authored_logical_input_replay_v3");
    return JSON.parse(replay(authoredProjectJson === null ? sourceText : authoredProjectJson, new Uint8Array(tapeBytes), bindingJson, allowPartial));
  }

  async ensureProjectModule() {
    if (!this.projectModulePromise) {
      this.projectModulePromise = this.createWasmVmImpl({
        ddnSourceText: "1 보여주기.",
        cacheBust: "approved-workbench-project-frontdoor-v1",
        setStatus: () => {},
        clearStatusError: () => {},
      }).then(async (handle) => {
        const client = handle.client ?? await handle.ensureClient("1 보여주기.");
        const wasmModule = client?.wasmModule;
        requireExport(wasmModule, "wasm_project_prepare_link_json");
        if (
          typeof wasmModule.wasm_project_edit_plan_json !== "function"
          || typeof wasmModule.wasm_project_source_sha256_json !== "function"
        ) {
          throw new Error("project edit-plan WASM export 누락");
        }
        return wasmModule;
      });
    }
    return this.projectModulePromise;
  }

  async canonMaegimPlan(sourceText) {
    const wasmModule = await this.ensureProjectModule();
    const operation = requireExport(wasmModule, "wasm_canon_maegim_plan");
    return JSON.parse(String(operation(sourceText) ?? "{}"));
  }

  async verifyAssumptionSource(sourceText, sourcePath, options = undefined) {
    if (typeof sourceText !== "string" || typeof sourcePath !== "string" || !sourcePath.length) {
      throw new TypeError("보임 검사에는 현재 원문과 파일 경로가 필요합니다.");
    }
    const module = await this.ensureProjectModule();
    if (options !== undefined && (options === null || typeof options !== "object" || Array.isArray(options)
        || Object.keys(options).some((key) => key !== "includeSteps")
        || (options.includeSteps !== undefined && typeof options.includeSteps !== "boolean"))) {
      throw new TypeError("단계 관찰 옵션은 includeSteps 참·거짓만 허용합니다.");
    }
    const operation = requireExport(module, options?.includeSteps === true
      ? "wasm_observe_algebra_source_json" : "wasm_verify_assumption_source_json");
    return JSON.parse(operation(sourceText, sourcePath));
  }

  async sourceAnalysis(request) {
    const wasmModule = await this.ensureProjectModule();
    const operation = requireExport(
      wasmModule,
      "wasm_source_analysis_snapshot_json",
      "E_SOURCE_ANALYSIS_WASM_REQUIRED 공통 Rust/WASM 원문 분석 기능이 필요합니다",
    );
    return JSON.parse(String(operation(JSON.stringify(request)) ?? "{}"));
  }

  async moduleDirectiveAnalysis(request) {
    const wasmModule = await this.ensureProjectModule();
    const operation = requireExport(
      wasmModule,
      "wasm_module_directive_analysis_snapshot_json",
      "E_MODULE_DIRECTIVE_ANALYSIS_WASM_REQUIRED 공통 Rust/WASM module directive 분석 기능이 필요합니다",
    );
    return JSON.parse(String(operation(JSON.stringify(request)) ?? "{}"));
  }

  async sharedCompletion(request) {
    const wasmModule = await this.ensureProjectModule();
    const operation = requireExport(
      wasmModule,
      "wasm_project_authoring_completion_json",
      "E_COMPLETION_WASM_REQUIRED 공통 Rust/WASM 자동완성 기능이 필요합니다",
    );
    return JSON.parse(String(operation(JSON.stringify(request)) ?? "{}"));
  }

  async runSummaryWithSourceProfile({
    sourceText,
    sourcePath,
    requestedTicks,
    sourceProfileIdentity,
    inputReceipts = null,
    paramOverrides = null,
    bogaeEntryName = "",
    authoredArtifactText = "",
  }) {
    const wasmModule = await this.ensureProjectModule();
    if (authoredArtifactText) {
      const operation = requireExport(wasmModule, "wasm_run_summary_with_authored_bogae_json",
        "E_WASM_AUTHORED_BOGAE_API_MISSING: 저장된 보개 정체 해소 기능이 없습니다.");
      return JSON.parse(String(operation(sourceText, String(sourcePath ?? "lesson.ddn"),
        Number(requestedTicks ?? 1), String(sourceProfileIdentity ?? ""), authoredArtifactText,
        inputReceipts === null ? undefined : JSON.stringify(inputReceipts),
        paramOverrides === null ? undefined : JSON.stringify(paramOverrides))));
    }
    const operation = requireExport(
      wasmModule,
      inputReceipts === null ? (bogaeEntryName ? "wasm_run_summary_with_bogae_entry_json" : "wasm_run_summary_with_source_profile_json")
        : "wasm_run_summary_with_source_profile_input_receipts_json",
      "E_WASM_RUN_SUMMARY_REQUIRED 공통 Rust/WASM 실행 결과와 영수증 기능이 필요합니다",
    );
    return JSON.parse(String(operation(
      sourceText,
      String(sourcePath ?? "lesson.ddn"),
      Number(requestedTicks ?? 1),
      String(sourceProfileIdentity ?? ""),
      ...(inputReceipts === null ? (bogaeEntryName ? [bogaeEntryName] : []) : [JSON.stringify(inputReceipts), paramOverrides === null ? undefined : JSON.stringify(paramOverrides), bogaeEntryName || undefined]),
    ) ?? "{}"));
  }

  async planGajiExtraction(request) {
    const wasmModule = await this.ensureProjectModule();
    return JSON.parse(requireExport(wasmModule, "wasm_gaji_extraction_plan_json")(JSON.stringify(request)));
  }

  async projectSourceSha256(source) {
    const wasmModule = await this.ensureProjectModule();
    return String(requireExport(wasmModule, "wasm_project_source_sha256_json")(JSON.stringify(source)));
  }

  async authoredProjectOperation({ operation, text }) {
    const wasmModule = await this.ensureProjectModule();
    return String(requireExport(wasmModule, "authored_project_operation",
      "E_AUTHORED_PROJECT_WASM_REQUIRED: 새 작품자료 WASM이 필요합니다")(String(operation), String(text)));
  }
  async observeMathNotebook({ sourceText, sourcePath, sourceProjectText }) {
    if (typeof sourceProjectText === "string") {
      if (sourceText !== undefined || sourcePath !== undefined) throw new Error("E_NOTEBOOK_INPUT: 작품과 단일 원문을 함께 넘기지 마세요.");
      const wasmModule = await this.ensureProjectModule();
      return JSON.parse(requireExport(wasmModule, "wasm_math_notebook_observe_project")(sourceProjectText));
    }
    if (typeof sourceText !== "string" || typeof sourcePath !== "string") {
      throw new Error("E_NOTEBOOK_INPUT: 원문과 원문 경로가 필요합니다");
    }
    const wasmModule = await this.ensureProjectModule();
    return JSON.parse(requireExport(wasmModule, "wasm_math_notebook_observe")(sourceText, sourcePath));
  }
  async compareGeneralOriginalRecording({bundleHex}) {
    if(typeof bundleHex!=="string"||!bundleHex.length||bundleHex.length%2||bundleHex.length>16*1024*1024||!/^[0-9a-f]+$/u.test(bundleHex))
      throw new Error("E_GENERAL_RECORDING_BYTES: 원 기록의 전송 형식이나 크기가 맞지 않습니다");
    const bytes=new Uint8Array(bundleHex.length/2);
    for(let i=0;i<bytes.length;i++)bytes[i]=Number.parseInt(bundleHex.slice(i*2,i*2+2),16);
    const wasmModule=await this.ensureProjectModule();
    return String(requireExport(wasmModule,"wasm_compare_general_original_recording_json",
      "E_GENERAL_RECORDING_WASM_REQUIRED: 원 기록을 대조하는 새 WASM이 필요합니다")(bytes));
  }

  async exportInputTapeV1({ inputReceipts, madiHz }) {
    const wasmModule = await this.ensureProjectModule();
    return requireExport(wasmModule, "wasm_export_input_tape_v1")(
      JSON.stringify(inputReceipts), madiHz);
  }

  async runSummaryWithTapeV1({ sourceText, sourcePath, sourceProfileIdentity, tapeBytes }) {
    const wasmModule = await this.ensureProjectModule();
    return JSON.parse(requireExport(wasmModule, "wasm_run_summary_with_source_profile_tape_v1_json")(
      String(sourceText), String(sourcePath), String(sourceProfileIdentity), new Uint8Array(tapeBytes)));
  }

  async buildLocalGajiArtifact(request) {
    const wasmModule = await this.ensureProjectModule();
    return JSON.parse(requireExport(wasmModule, "wasm_local_gaji_artifact_json")(JSON.stringify(request)));
  }

  async planGajiLifecycle(request) {
    const wasmModule = await this.ensureProjectModule();
    return JSON.parse(requireExport(wasmModule, "wasm_gaji_lifecycle_plan_json")(JSON.stringify(request)));
  }

  async projectGraph(source) {
    const wasmModule = await this.ensureProjectModule();
    const raw = requireExport(wasmModule, "wasm_project_graph_json")(JSON.stringify(source));
    return JSON.parse(String(raw ?? "{}"));
  }

  async projectSymbols(source) {
    const wasmModule = await this.ensureProjectModule();
    const raw = requireExport(wasmModule, "wasm_project_symbols_json")(JSON.stringify(source));
    return JSON.parse(String(raw ?? "{}"));
  }

  async prepareProjectLink(projectSource, target = "앱", strict = true, includeLock = true) {
    const wasmModule = await this.ensureProjectModule();
    const raw = requireExport(wasmModule, "wasm_project_prepare_link_json")(
      JSON.stringify(projectSource),
      target,
      strict,
      includeLock,
    );
    return JSON.parse(String(raw ?? "{}"));
  }

  async planProjectEdit(request) {
    const wasmModule = await this.ensureProjectModule();
    const raw = requireExport(wasmModule, "wasm_project_edit_plan_json")(JSON.stringify(request));
    return JSON.parse(String(raw ?? "{}"));
  }

  async projectGajiLock(source, { allowMissing = false } = {}) {
    const wasmModule = await this.ensureProjectModule();
    if (allowMissing && typeof wasmModule.wasm_project_gaji_lock_json !== "function") return null;
    const raw = requireExport(wasmModule, "wasm_project_gaji_lock_json")(JSON.stringify(source));
    return String(raw ?? "");
  }

  async publishLocalGotgan(request) {
    const wasmModule = await this.ensureProjectModule();
    const operation = requireExport(
      wasmModule,
      "wasm_publish_local_gotgan_json",
      "E_GOTGAN_WASM_REQUIRED 공통 Rust/WASM 발행 기능이 필요합니다",
    );
    return JSON.parse(operation(JSON.stringify(request)));
  }

  async searchLocalGotgan(request) {
    const wasmModule = await this.ensureProjectModule();
    const operation = requireExport(
      wasmModule,
      "wasm_search_local_gotgan_json",
      "E_GOTGAN_WASM_REQUIRED 공통 Rust/WASM 검색 기능이 필요합니다",
    );
    return JSON.parse(operation(JSON.stringify(request)));
  }
}

export class RuntimeBridgeV1 {
  #adapter;

  constructor({ adapter = new WorkerRuntimeAdapter() } = {}) {
    if (!adapter || typeof adapter !== "object") {
      throw new TypeError("runtime adapter must be an object");
    }
    this.schema = RUNTIME_BRIDGE_V1_SCHEMA;
    this.#adapter = adapter;
  }

  async createSession(options) {
    return new RuntimeSessionV1(await this.#adapter.createSession(options));
  }

  createLogicalSession(options) { return this.#adapter.createLogicalSession(options); }
  createManipulationSession(options) { return this.#adapter.createManipulationSession(options); }
  bindManipulationSource(options) { return this.#adapter.bindManipulationSource(options); }
  replayManipulationInput(options) { return this.#adapter.replayManipulationInput(options); }
  bindLogicalSource(options) { return this.#adapter.bindLogicalSource(options); }
  manipulationValueRaw(text) { return this.#adapter.manipulationValueRaw(text); }
  replayLogicalInput(request) { return this.#adapter.replayLogicalInput(request); }
  recordFiniteInput(request) { return this.#adapter.recordFiniteInput(request); }

  cancelExecution(reason) { this.#adapter.cancelExecution?.(reason); }

  canonMaegimPlan(sourceText) { return this.#adapter.canonMaegimPlan(sourceText); }
  sourceAnalysis(request) { return this.#adapter.sourceAnalysis(request); }
  verifyAssumptionSource(sourceText, sourcePath, options = undefined) { return this.#adapter.verifyAssumptionSource(sourceText, sourcePath, options); }
  sharedCompletion(request) { return this.#adapter.sharedCompletion(request); }
  runSummaryWithSourceProfile(request) { return this.#adapter.runSummaryWithSourceProfile(request); }
  exportInputTapeV1(request) { return this.#adapter.exportInputTapeV1(request); }
  runSummaryWithTapeV1(request) { return this.#adapter.runSummaryWithTapeV1(request); }
  planGajiExtraction(request) { return this.#adapter.planGajiExtraction(request); }
  projectSourceSha256(source) { return this.#adapter.projectSourceSha256(source); }
  authoredProjectOperation(request) { return this.#adapter.authoredProjectOperation(request); }
  observeMathNotebook(request) {
    if (typeof this.#adapter.observeMathNotebook !== "function") {
      throw new Error("E_NOTEBOOK_ADAPTER_UNAVAILABLE: 현재 실행 어댑터에 수학 노트 연결이 없습니다");
    }
    return this.#adapter.observeMathNotebook(request);
  }
  compareGeneralOriginalRecording(request) { return this.#adapter.compareGeneralOriginalRecording(request); }
  buildLocalGajiArtifact(request) { return this.#adapter.buildLocalGajiArtifact(request); }
  planGajiLifecycle(request) { return this.#adapter.planGajiLifecycle(request); }
  projectGraph(source) { return this.#adapter.projectGraph(source); }
  projectSymbols(source) { return this.#adapter.projectSymbols(source); }
  prepareProjectLink(projectSource, target, strict, includeLock) {
    return this.#adapter.prepareProjectLink(projectSource, target, strict, includeLock);
  }
  planProjectEdit(request) { return this.#adapter.planProjectEdit(request); }
  projectGajiLock(source, options) { return this.#adapter.projectGajiLock(source, options); }
  publishLocalGotgan(request) { return this.#adapter.publishLocalGotgan(request); }
  searchLocalGotgan(request) { return this.#adapter.searchLocalGotgan(request); }
}

export function createRuntimeBridgeV1(options = {}) {
  return new RuntimeBridgeV1(options);
}
