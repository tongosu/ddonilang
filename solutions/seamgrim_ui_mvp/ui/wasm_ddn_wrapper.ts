import type { DdnWasmVm, SeamgrimDerivedState, SeamgrimState, SeamgrimSchemaId } from "./wasm_ddn_types";
import { normalizeWasmStatePayload } from "./seamgrim_runtime_state.js";

const EXPECTED_SCHEMA: SeamgrimSchemaId = "seamgrim.state.v0";

export function parseStateJson(payload: string): SeamgrimState {
  const parsed = normalizeWasmStatePayload(payload) as SeamgrimState;
  if (parsed.schema !== EXPECTED_SCHEMA) {
    throw new Error(`Unexpected schema: ${String(parsed.schema)}`);
  }
  return parsed;
}

function nowMs(): number {
  if (typeof performance !== "undefined" && typeof performance.now === "function") {
    return performance.now();
  }
  return Date.now();
}

export class DdnWasmVmClient {
  private readonly vm: DdnWasmVm;
  private frameId: number;
  private lastStepMs: number | null;

  constructor(vm: DdnWasmVm) {
    this.vm = vm;
    this.frameId = 0;
    this.lastStepMs = null;
  }

  updateLogic(source: string): void {
    this.vm.update_logic(source);
  }

  updateLogicWithMode(source: string, mode: string): void {
    if (typeof (this.vm as any).update_logic_with_mode === "function") {
      (this.vm as any).update_logic_with_mode(source, mode);
      return;
    }
    this.updateLogic(source);
  }

  setRngSeed(seed: number): void {
    this.vm.set_rng_seed(seed);
  }

  setInput(
    keys_pressed: number,
    last_key_name: string,
    pointer_x_i32: number,
    pointer_y_i32: number,
    dt: number
  ): void {
    this.vm.set_input(keys_pressed, last_key_name, pointer_x_i32, pointer_y_i32, dt);
  }

  columnsParsed(): any {
    if (typeof (this.vm as any).columns !== "function") {
      return { columns: [], row: [] };
    }
    return JSON.parse((this.vm as any).columns());
  }

  setParamParsed(key: string, value: number | boolean | string): any {
    if (typeof (this.vm as any).set_param !== "function") {
      throw new Error("set_param is not available in this wasm build");
    }
    return JSON.parse((this.vm as any).set_param(key, value));
  }

  setParamFixed64Parsed(key: string, rawI64: number): any {
    if (typeof (this.vm as any).set_param_fixed64 !== "function") {
      throw new Error("set_param_fixed64 is not available in this wasm build");
    }
    return JSON.parse((this.vm as any).set_param_fixed64(key, rawI64));
  }

  setParamFixed64StringParsed(key: string, rawI64Text: string): any {
    if (typeof (this.vm as any).set_param_fixed64_str !== "function") {
      throw new Error("set_param_fixed64_str is not available in this wasm build");
    }
    return JSON.parse((this.vm as any).set_param_fixed64_str(key, String(rawI64Text ?? "")));
  }

  addViewPrefix(prefix: string): boolean {
    if (typeof (this.vm as any).add_view_prefix !== "function") {
      return false;
    }
    (this.vm as any).add_view_prefix(prefix);
    return true;
  }

  clearViewPrefixes(): boolean {
    if (typeof (this.vm as any).clear_view_prefixes !== "function") {
      return false;
    }
    (this.vm as any).clear_view_prefixes();
    return true;
  }

  injectAiAction(key: string, valueJson: string): boolean {
    if (typeof (this.vm as any).inject_ai_action !== "function") {
      return false;
    }
    (this.vm as any).inject_ai_action(key, valueJson);
    return true;
  }

  clearAiInjections(): boolean {
    if (typeof (this.vm as any).clear_ai_injections !== "function") {
      return false;
    }
    (this.vm as any).clear_ai_injections();
    return true;
  }

  resetParsed(keepParams = false): any {
    if (typeof (this.vm as any).reset !== "function") {
      throw new Error("reset is not available in this wasm build");
    }
    this.frameId = 0;
    this.lastStepMs = null;
    return JSON.parse((this.vm as any).reset(keepParams));
  }

  restoreStateParsed(stateJson: string): any {
    if (typeof (this.vm as any).restore_state !== "function") {
      throw new Error("restore_state is not available in this wasm build");
    }
    const result = JSON.parse((this.vm as any).restore_state(stateJson));
    this.frameId = 0;
    this.lastStepMs = null;
    return result;
  }

  stepOneParsed(): SeamgrimDerivedState {
    const state = parseStateJson(this.vm.step_one());
    return this.attachDerived(state, true);
  }

  stepOneWithInputParsed(
    keys_pressed: number,
    last_key_name: string,
    pointer_x_i32: number,
    pointer_y_i32: number,
    dt: number
  ): SeamgrimDerivedState {
    const state = parseStateJson(
      this.vm.step_one_with_input(keys_pressed, last_key_name, pointer_x_i32, pointer_y_i32, dt)
    );
    return this.attachDerived(state, true);
  }

  getStateHash(): string {
    return this.vm.get_state_hash();
  }

  getStateParsed(): SeamgrimDerivedState {
    const state = parseStateJson(this.vm.get_state_json());
    return this.attachDerived(state, false);
  }

  private attachDerived(state: SeamgrimState, advance: boolean): SeamgrimDerivedState {
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
