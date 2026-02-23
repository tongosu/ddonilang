/* tslint:disable */
/* eslint-disable */

export class DdnWasmVm {
    free(): void;
    [Symbol.dispose](): void;
    add_view_prefix(prefix: string): void;
    clear_ai_injections(): void;
    clear_view_prefixes(): void;
    columns(): any;
    get_build_info(): string;
    get_state_hash(): string;
    get_state_json(): any;
    inject_ai_action(key: string, value_json: string): void;
    constructor(source: string);
    static new_with_mode(source: string, mode: string): DdnWasmVm;
    reset(keep_params?: boolean | null): any;
    restore_state(state_json: string): any;
    set_dt_f64(dt: number): void;
    set_input(keys_pressed: number, last_key_name: string, pointer_x_i32: number, pointer_y_i32: number, dt: number): void;
    set_keys_pressed(keys_pressed: number): void;
    set_last_key_name(last_key_name: string): void;
    set_param(key: string, value: any): any;
    set_param_fixed64(key: string, raw_i64: bigint): any;
    set_param_fixed64_str(key: string, raw_i64: string): any;
    set_pointer(pointer_x_i32: number, pointer_y_i32: number): void;
    set_rng_seed(seed: bigint): void;
    step_one(): any;
    step_one_with_input(keys_pressed: number, last_key_name: string, pointer_x_i32: number, pointer_y_i32: number, dt: number): any;
    update_logic(source: string): void;
    update_logic_with_mode(source: string, mode: string): void;
}

export function wasm_build_info(): string;

export function wasm_preprocess_source(source: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_ddnwasmvm_free: (a: number, b: number) => void;
    readonly ddnwasmvm_add_view_prefix: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_clear_ai_injections: (a: number) => void;
    readonly ddnwasmvm_clear_view_prefixes: (a: number) => void;
    readonly ddnwasmvm_columns: (a: number) => number;
    readonly ddnwasmvm_get_build_info: (a: number, b: number) => void;
    readonly ddnwasmvm_get_state_hash: (a: number, b: number) => void;
    readonly ddnwasmvm_get_state_json: (a: number) => number;
    readonly ddnwasmvm_inject_ai_action: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnwasmvm_new: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_new_with_mode: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnwasmvm_reset: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_restore_state: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_set_dt_f64: (a: number, b: number) => void;
    readonly ddnwasmvm_set_input: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly ddnwasmvm_set_keys_pressed: (a: number, b: number) => void;
    readonly ddnwasmvm_set_last_key_name: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_set_param: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnwasmvm_set_param_fixed64: (a: number, b: number, c: number, d: number, e: bigint) => void;
    readonly ddnwasmvm_set_param_fixed64_str: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly ddnwasmvm_set_pointer: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_set_rng_seed: (a: number, b: bigint) => void;
    readonly ddnwasmvm_step_one: (a: number, b: number) => void;
    readonly ddnwasmvm_step_one_with_input: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly ddnwasmvm_update_logic: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_update_logic_with_mode: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wasm_build_info: (a: number) => void;
    readonly wasm_preprocess_source: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
