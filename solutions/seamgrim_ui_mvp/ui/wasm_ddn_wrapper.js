import { normalizeWasmStatePayload } from "./seamgrim_runtime_state.js";

export function parseStateJson(payload) {
  return normalizeWasmStatePayload(payload);
}

function nowMs() {
  if (typeof performance !== "undefined" && typeof performance.now === "function") {
    return performance.now();
  }
  return Date.now();
}

export class DdnWasmVmClient {
  constructor(vm) {
    this.vm = vm;
    this.frameId = 0;
    this.lastStepMs = null;
  }

  updateLogic(source) {
    this.vm.update_logic(source);
  }

  exportSavedProjectArtifact(parentHex, revisionHex) {
    if (typeof this.vm.export_saved_project_artifact_json !== "function") {
      throw new Error("E_PROJECT_RUNTIME_SAVE_API_MISSING: 실행 저장 기능이 없습니다.");
    }
    return this.vm.export_saved_project_artifact_json(parentHex, revisionHex);
  }

  updateLogicWithMode(source, mode) {
    if (typeof this.vm.update_logic_with_mode !== "function") {
      throw new Error("update_logic_with_mode is not available in this wasm build");
    }
    this.vm.update_logic_with_mode(source, mode);
  }

  updateLogicWithSourceProfile(source, sourceProfileIdentity) {
    if (typeof this.vm.update_logic_with_source_profile !== "function") {
      throw new Error("E_WASM_SOURCE_PROFILE_API_MISSING");
    }
    this.vm.update_logic_with_source_profile(source, sourceProfileIdentity);
  }

  configureProjectEntry(name) {
    if (typeof this.vm.configure_project_entry !== "function") {
      throw new Error("E_WASM_PROJECT_ENTRY_API_MISSING: 이 WASM은 프로젝트 시작 정의를 실행할 수 없습니다.");
    }
    return this.vm.configure_project_entry(name);
  }

  configureBogaeEntry(name) {
    if (typeof this.vm.configure_bogae_entry !== "function") {
      throw new Error("E_WASM_BOGAE_ENTRY_API_MISSING: 명시 보개 시작 해소 기능이 없습니다.");
    }
    return this.vm.configure_bogae_entry(name);
  }

  configureAuthoredBogae(artifactText) {
    if (typeof this.vm.configure_authored_bogae !== "function") {
      throw new Error("E_WASM_AUTHORED_BOGAE_API_MISSING: 저장된 보개 정체 해소 기능이 없습니다.");
    }
    return this.vm.configure_authored_bogae(artifactText);
  }

  setRngSeed(seed) {
    this.vm.set_rng_seed(seed);
  }

  setInputFixed64V1(keys_pressed, last_key_name, pointer_x_i32, pointer_y_i32, dt_high_i32, dt_low_u32) {
    if (typeof this.vm.set_input_fixed64_v1 !== "function") {
      throw new Error("set_input_fixed64_v1 is not available in this wasm build");
    }
    return JSON.parse(this.vm.set_input_fixed64_v1(
      keys_pressed,
      last_key_name,
      pointer_x_i32,
      pointer_y_i32,
      dt_high_i32,
      dt_low_u32,
    ));
  }

  fixed64FromDecimalV1Parsed(decimalText) {
    if (typeof this.vm.fixed64_from_decimal_v1 !== "function") {
      throw new Error("fixed64_from_decimal_v1 is not available in this wasm build");
    }
    return JSON.parse(this.vm.fixed64_from_decimal_v1(String(decimalText ?? "")));
  }

  columnsParsed() {
    if (typeof this.vm.columns !== "function") {
      throw new Error("columns is not available in this wasm build");
    }
    return JSON.parse(this.vm.columns());
  }

  parseWarningsParsed() {
    if (typeof this.vm.get_parse_warnings !== "function") {
      throw new Error("get_parse_warnings is not available in this wasm build");
    }
    const parsed = JSON.parse(this.vm.get_parse_warnings());
    return Array.isArray(parsed?.warnings) ? parsed.warnings : [];
  }

  configuredMadi() {
    if (typeof this.vm.get_configured_madi !== "function") {
      return 0;
    }
    const value = Number(this.vm.get_configured_madi());
    return Number.isFinite(value) && value > 0 ? Math.floor(value) : 0;
  }

  setParamParsed(key, value) {
    if (typeof this.vm.set_param !== "function") {
      throw new Error("set_param is not available in this wasm build");
    }
    if (typeof value === "number") {
      throw new TypeError("Fixed64 number input requires the raw Fixed64 setter");
    }
    return JSON.parse(this.vm.set_param(key, value));
  }

  setParamFixed64Parsed(key, rawI64) {
    if (typeof this.vm.set_param_fixed64 !== "function") {
      throw new Error("set_param_fixed64 is not available in this wasm build");
    }
    return JSON.parse(this.vm.set_param_fixed64(key, rawI64));
  }

  setParamFixed64StringParsed(key, rawI64Text) {
    if (typeof this.vm.set_param_fixed64_str !== "function") {
      throw new Error("set_param_fixed64_str is not available in this wasm build");
    }
    return JSON.parse(this.vm.set_param_fixed64_str(key, String(rawI64Text ?? "")));
  }

  addViewPrefix(prefix) {
    if (typeof this.vm.add_view_prefix !== "function") {
      return false;
    }
    this.vm.add_view_prefix(prefix);
    return true;
  }

  clearViewPrefixes() {
    if (typeof this.vm.clear_view_prefixes !== "function") {
      return false;
    }
    this.vm.clear_view_prefixes();
    return true;
  }

  injectAiAction(key, valueJson) {
    if (typeof this.vm.inject_ai_action !== "function") {
      return false;
    }
    this.vm.inject_ai_action(key, valueJson);
    return true;
  }

  clearAiInjections() {
    if (typeof this.vm.clear_ai_injections !== "function") {
      return false;
    }
    this.vm.clear_ai_injections();
    return true;
  }

  resetParsed(keepParams = false) {
    if (typeof this.vm.reset !== "function") {
      throw new Error("reset is not available in this wasm build");
    }
    this.frameId = 0;
    this.lastStepMs = null;
    return JSON.parse(this.vm.reset(keepParams));
  }

  restoreStateParsed(stateJson) {
    if (typeof this.vm.restore_state !== "function") {
      throw new Error("restore_state is not available in this wasm build");
    }
    const result = JSON.parse(this.vm.restore_state(stateJson));
    this.frameId = 0;
    this.lastStepMs = null;
    return result;
  }

  stepOneParsed() {
    const state = parseStateJson(this.vm.step_one());
    return this.attachDerived(state, true);
  }

  runTicksParsed(count) {
    if (typeof this.vm.run_ticks !== "function") {
      throw new Error("run_ticks is not available in this wasm build");
    }
    const state = parseStateJson(this.vm.run_ticks(count));
    return this.attachDerived(state, true);
  }

  applyCurrentlineCellParsed(source, context = null) {
    if (typeof this.vm.apply_currentline_cell !== "function") {
      throw new Error("apply_currentline_cell is not available in this wasm build");
    }
    const contextJson = context === null || context === undefined
      ? undefined
      : (typeof context === "string" ? context : JSON.stringify(context));
    const state = parseStateJson(this.vm.apply_currentline_cell(source, contextJson));
    return this.attachDerived(state, true);
  }

  applyCurrentlineCellWithSourceProfileParsed(source, sourceProfileIdentity, context = null) {
    if (typeof this.vm.apply_currentline_cell_with_source_profile !== "function") {
      throw new Error("apply_currentline_cell_with_source_profile is not available in this wasm build");
    }
    const contextJson = context === null || context === undefined
      ? undefined
      : (typeof context === "string" ? context : JSON.stringify(context));
    const state = parseStateJson(
      this.vm.apply_currentline_cell_with_source_profile(source, sourceProfileIdentity, contextJson),
    );
    return this.attachDerived(state, true);
  }

  stepOneWithInputFixed64V1Parsed(
    keys_pressed,
    last_key_name,
    pointer_x_i32,
    pointer_y_i32,
    dt_high_i32,
    dt_low_u32,
  ) {
    if (typeof this.vm.step_one_with_input_fixed64_v1 !== "function") {
      throw new Error("step_one_with_input_fixed64_v1 is not available in this wasm build");
    }
    const state = parseStateJson(
      this.vm.step_one_with_input_fixed64_v1(
        keys_pressed,
        last_key_name,
        pointer_x_i32,
        pointer_y_i32,
        dt_high_i32,
        dt_low_u32,
      ),
    );
    return this.attachDerived(state, true);
  }

  getStateHash() {
    return this.vm.get_state_hash();
  }

  getStateParsed() {
    const state = parseStateJson(this.vm.get_state_json());
    return this.attachDerived(state, false);
  }

  executionModeDecisionParsed() {
    if (typeof this.vm.get_execution_mode_decision_json !== "function") {
      throw new Error("get_execution_mode_decision_json is not available in this wasm build");
    }
    return JSON.parse(this.vm.get_execution_mode_decision_json());
  }

  inputStateParsed() {
    if (typeof this.vm.get_input_state_json === "function") {
      return JSON.parse(this.vm.get_input_state_json());
    }
    const state = this.getStateParsed();
    return state.input ?? {};
  }

  attachDerived(state, advance) {
    const now = nowMs();
    const tickTime = this.lastStepMs === null ? 0 : Math.max(0, now - this.lastStepMs);
    if (advance) {
      this.lastStepMs = now;
      const next = { ...state, frame_id: this.frameId, tick_time_ms: tickTime };
      this.frameId += 1;
      return next;
    }
    return { ...state, frame_id: this.frameId, tick_time_ms: 0 };
  }
}
