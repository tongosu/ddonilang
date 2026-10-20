export type SeamgrimSchemaId = "seamgrim.state.v0" | "seamgrim.engine_response.v0";

export interface SeamgrimInput {
  keys_pressed: number;
  last_key_name: string;
  pointer_x_i32: number;
  pointer_y_i32: number;
  dt: string;
  frame_source?: string;
  frame_source_code?: number;
  rng_seed: number;
  rng_base_seed?: number;
  pending_ai_injection_count?: number;
  ai_injections?: Array<Record<string, unknown>>;
  net_events?: Array<Record<string, unknown>>;
}

export interface SeamgrimResources {
  json: Record<string, string>;
  fixed64: Record<string, string>;
  handle: Record<string, string>;
  value: Record<string, string>;
}

export interface SeamgrimChannel {
  key: string;
  dtype: string;
  role: string;
  unit?: string;
}

export type SeamgrimPatchOp =
  | { op: "set_resource_json"; tag: string; value: string }
  | { op: "set_resource_fixed64"; tag: string; value: string }
  | { op: "set_resource_handle"; tag: string; value: string }
  | { op: "set_resource_value"; tag: string; value: string }
  | { op: "div_assign_resource_fixed64"; tag: string; value: string }
  | { op: "set_component_json"; entity: number; tag: number; value: string }
  | { op: "remove_component"; entity: number; tag: number }
  | { op: "emit_signal"; signal: string; targets: number[] }
  | { op: "guard_violation"; entity: number; rule_id: number };

export interface SeamgrimState {
  schema: "seamgrim.state.v0";
  tick_id: number;
  state_hash: string;
  input: SeamgrimInput;
  resources: SeamgrimResources;
  channels?: SeamgrimChannel[];
  row?: unknown[];
  patch?: SeamgrimPatchOp[];
  view_meta?: Record<string, unknown>;
  streams?: Record<string, unknown>;
  view_hash?: string | null;
  engine_schema?: "seamgrim.engine_response.v0";
}

export interface SeamgrimDerivedState extends SeamgrimState {
  frame_id: number;
  tick_time_ms: number;
}

export interface DdnParseWarning {
  code: string;
  message: string;
  span?: {
    start?: number;
    end?: number;
  };
}

export interface DdnWasmVm {
  columns(): string;
  update_logic(source: string): void;
  update_logic_with_mode?(source: string, mode: string): void;
  get_parse_warnings?(): string;
  set_param(key: string, value: boolean | string): string;
  set_param_fixed64?(key: string, raw_i64: number): string;
  set_param_fixed64_str?(key: string, raw_i64: string): string;
  reset(keep_params?: boolean | null): string;
  restore_state?(state_json: string): string;
  set_rng_seed(seed: number): void;
  add_view_prefix?(prefix: string): void;
  clear_view_prefixes?(): void;
  inject_ai_action?(key: string, value_json: string): void;
  clear_ai_injections?(): void;
  get_input_state_json?(): string;
  fixed64_from_decimal_v1(decimal_text: string): string;
  set_input_fixed64_v1(
    keys_pressed: number,
    last_key_name: string,
    pointer_x_i32: number,
    pointer_y_i32: number,
    dt_high_i32: number,
    dt_low_u32: number
  ): string;
  set_keys_pressed(keys_pressed: number): void;
  set_last_key_name(last_key_name: string): void;
  set_pointer(pointer_x_i32: number, pointer_y_i32: number): void;
  set_dt_fixed64_v1(dt_high_i32: number, dt_low_u32: number): string;
  step_one(): string;
  run_ticks?(count: number): string;
  apply_currentline_cell?(source: string, context_json?: string): string;
  step_one_with_input_fixed64_v1(
    keys_pressed: number,
    last_key_name: string,
    pointer_x_i32: number,
    pointer_y_i32: number,
    dt_high_i32: number,
    dt_low_u32: number
  ): string;
  get_state_hash(): string;
  get_state_json(): string;
}
