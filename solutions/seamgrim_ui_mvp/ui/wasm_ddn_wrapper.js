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

  updateLogicWithMode(source, mode) {
    if (typeof this.vm.update_logic_with_mode === "function") {
      this.vm.update_logic_with_mode(source, mode);
      return;
    }
    this.updateLogic(source);
  }

  setRngSeed(seed) {
    this.vm.set_rng_seed(seed);
  }

  setInput(keys_pressed, last_key_name, pointer_x_i32, pointer_y_i32, dt) {
    this.vm.set_input(keys_pressed, last_key_name, pointer_x_i32, pointer_y_i32, dt);
  }

  columnsParsed() {
    if (typeof this.vm.columns !== "function") {
      return { columns: [], row: [] };
    }
    return JSON.parse(this.vm.columns());
  }

  setParamParsed(key, value) {
    if (typeof this.vm.set_param !== "function") {
      throw new Error("set_param is not available in this wasm build");
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

  stepOneWithInputParsed(keys_pressed, last_key_name, pointer_x_i32, pointer_y_i32, dt) {
    const state = parseStateJson(
      this.vm.step_one_with_input(keys_pressed, last_key_name, pointer_x_i32, pointer_y_i32, dt),
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
