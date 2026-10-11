import { createWasmVm } from "./wasm_vm_runtime.js";
import { WorkerRuntimeAdapter } from "./worker_runtime_adapter.js";

export const RUNTIME_BRIDGE_V1_SCHEMA = "seamgrim.runtime_bridge.v1";

export const RUNTIME_BRIDGE_V1_OPERATIONS = Object.freeze([
  "authoredProjectOperation",
  "createSession",
  "createLogicalSession",
  "bindLogicalSource",
  "replayLogicalInput",
  "sourceAnalysis",
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

  async createLogicalSession({ sourceText, bindingJson }) {
    if (typeof sourceText !== "string" || typeof bindingJson !== "string") {
      throw new Error("E_LOGICAL_INPUT_BINDING: 원문과 명시 입력 결속이 필요합니다.");
    }
    const module = await this.ensureProjectModule();
    const Constructor = requireExport(module, "DdnLogicalInputVm", "E_LOGICAL_INPUT_RUNTIME_MISSING: 새 논리 입력 실행기가 필요합니다.");
    const vm = new Constructor(sourceText, bindingJson);
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
      summary: () => JSON.parse(vm.summary_json()),
      seal: () => JSON.parse(vm.seal_json()),
      abort: message => vm.abort_input(String(message)),
      close: () => vm.free(),
    };
  }

  async bindLogicalSource({ sourceText, bindingJson }) {
    const module = await this.ensureProjectModule();
    const Constructor = module.DdnLogicalInputVm;
    const bind = requireExport(Constructor, "bind_source_json", "E_LOGICAL_INPUT_BIND_SOURCE_UNSUPPORTED: 새 원문 결속 기능이 필요합니다.");
    return bind(sourceText, bindingJson);
  }

  async replayLogicalInput({ sourceText, tapeBytes, bindingJson }) {
    const module = await this.ensureProjectModule();
    const replay = requireExport(module, "logical_input_replay");
    return JSON.parse(replay(sourceText, new Uint8Array(tapeBytes), bindingJson));
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
  bindLogicalSource(options) { return this.#adapter.bindLogicalSource(options); }
  replayLogicalInput(request) { return this.#adapter.replayLogicalInput(request); }

  cancelExecution(reason) { this.#adapter.cancelExecution?.(reason); }

  canonMaegimPlan(sourceText) { return this.#adapter.canonMaegimPlan(sourceText); }
  sourceAnalysis(request) { return this.#adapter.sourceAnalysis(request); }
  sharedCompletion(request) { return this.#adapter.sharedCompletion(request); }
  runSummaryWithSourceProfile(request) { return this.#adapter.runSummaryWithSourceProfile(request); }
  exportInputTapeV1(request) { return this.#adapter.exportInputTapeV1(request); }
  runSummaryWithTapeV1(request) { return this.#adapter.runSummaryWithTapeV1(request); }
  planGajiExtraction(request) { return this.#adapter.planGajiExtraction(request); }
  projectSourceSha256(source) { return this.#adapter.projectSourceSha256(source); }
  authoredProjectOperation(request) { return this.#adapter.authoredProjectOperation(request); }
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
