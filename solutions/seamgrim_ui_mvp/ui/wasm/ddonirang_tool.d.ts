/* tslint:disable */
/* eslint-disable */

/**
 * Live logical input uses the same transactional session as file replay.
 * Device mapping remains outside this VM; no raw-key/default-input fallback.
 */
export class DdnLogicalInputVm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Host input loss is explicit failure, not a manufactured Release event.
     */
    abort_input(message: string): void;
    static bind_authored_project_json(authored: string, binding_json: string): string;
    static bind_source_json(source: string, binding_json: string): string;
    /**
     * Existing complete-state request bytes, paired with the RD-686 carrier.
     * IndexedDB confirmation and Native durable receipts remain separate.
     */
    export_project_checkpoint(parent: Uint8Array, revision: Uint8Array): Uint8Array;
    /**
     * Existing source-bound carrier only; not a committed durable receipt.
     */
    export_saved_project(parent: Uint8Array, revision: Uint8Array): string;
    failed_prefix_v3_json(): string;
    /**
     * Explicit project admission preserves modules and the named Start.
     * No linked-text fallback or durable browser receipt is manufactured.
     */
    static from_authored_project(authored: string, binding_json: string): DdnLogicalInputVm;
    /**
     * Existing source-bound carrier only; not a committed durable receipt.
     */
    static from_saved_project_checkpoint(artifact: string, checkpoint: Uint8Array, parent: Uint8Array, revision: Uint8Array, binding_json: string): DdnLogicalInputVm;
    constructor(source: string, binding_json: string);
    seal_json(): string;
    step_json(snapshot_json: string): string;
    stop_recording_v3_json(): string;
    summary_json(): string;
    /**
     * Display projection only. The shared accepted world is never mutated.
     */
    view_json(): string;
}

/**
 * WO15 provisional typed value intake; the V1 logical VM is unchanged.
 */
export class DdnManipulationVm {
    free(): void;
    [Symbol.dispose](): void;
    abort_input(message: string): void;
    static bind_source_json(source: string, binding_json: string): string;
    failed_prefix_v3_json(): string;
    constructor(source: string, binding_json: string);
    seal_json(): string;
    step_json(snapshot_json: string): string;
    stop_recording_v3_json(): string;
    summary_json(): string;
    view_json(): string;
}

export class DdnWasmVm {
    free(): void;
    [Symbol.dispose](): void;
    add_view_prefix(prefix: string): void;
    apply_currentline_cell(source: string, context_json?: string | null): any;
    /**
     * 명시적 supported source profile을 소비하는 currentline 내부 candidate entry다.
     * 기존 apply_currentline_cell의 legacy 기본 동작은 바꾸지 않는다.
     */
    apply_currentline_cell_with_source_profile(source: string, source_profile_identity: string, context_json?: string | null): any;
    clear_ai_injections(): void;
    clear_debug_plan(): any;
    clear_view_prefixes(): void;
    columns(): any;
    configure_authored_bogae(artifact_json: string): void;
    configure_bogae_entry(name: string): void;
    configure_debug_plan(plan_json: string): any;
    configure_project_assets(project_source_json: string): string;
    /**
     * Bind the entry selected by the explicit work-root linker. Call again
     * after installing a replacement program; no filename or name inference.
     */
    configure_project_entry(name: string): void;
    export_saved_project_artifact_json(expected_parent_hex: string, revision_hex: string): string;
    /**
     * Canonical textual admission for a Workbench frame duration.  JavaScript
     * carries only this decimal text and the returned Q31.32 pair; the
     * numeric conversion itself remains owned by Fixed64.
     */
    fixed64_from_decimal_v1(decimal: string): any;
    /**
     * Explicit project execution retains exactly the captures that this VM
     * executes. Saving does not reread source or allocate replacement ordinals.
     */
    static from_authored_project_json(artifact_json: string): DdnWasmVm;
    static from_native_project_json(authored: string): DdnWasmVm;
    /**
     * Restores into a separate candidate. The host supplies the revision pair
     * selected from its storage lineage; neither value is taken from the file.
     * A failure leaves every existing VM untouched.
     */
    static from_saved_project_artifact_json(artifact_json: string, expected_parent_hex: string, revision_hex: string): DdnWasmVm;
    get_build_info(): string;
    get_configured_madi(): bigint;
    get_debug_source_identity_json(): string;
    get_execution_mode_decision_json(): string;
    get_input_state_json(): any;
    get_parse_warnings(): any;
    get_state_hash(): string;
    get_state_json(): any;
    inject_ai_action(key: string, value_json: string): void;
    is_native_source_bound(): boolean;
    constructor(source: string);
    static new_with_mode(source: string, mode: string): DdnWasmVm;
    static new_with_source_path(source: string, source_path: string): DdnWasmVm;
    reset(keep_params?: boolean | null): any;
    restore_state(state_json: string): any;
    run_ticks(count: number): any;
    save_native_source_checkpoint(authored: string): string;
    set_dt_fixed64_v1(dt_high_i32: number, dt_low_u32: number): any;
    set_input_fixed64_v1(keys_pressed: number, last_key_name: string, pointer_x_i32: number, pointer_y_i32: number, dt_high_i32: number, dt_low_u32: number): any;
    set_keys_pressed(keys_pressed: number): void;
    set_last_key_name(last_key_name: string): void;
    set_param(key: string, value: any): any;
    set_param_fixed64(key: string, raw_i64: bigint): any;
    set_param_fixed64_str(key: string, raw_i64: string): any;
    set_pointer(pointer_x_i32: number, pointer_y_i32: number): void;
    set_rng_seed(seed: bigint): void;
    step_one(): any;
    step_one_with_input_fixed64_v1(keys_pressed: number, last_key_name: string, pointer_x_i32: number, pointer_y_i32: number, dt_high_i32: number, dt_low_u32: number): any;
    update_logic(source: string): void;
    update_logic_with_mode(source: string, mode: string): void;
    update_logic_with_source_path(source: string, source_path: string): void;
    update_logic_with_source_profile(source: string, source_profile_identity: string): void;
}

export function authored_logical_input_record_v3(authored: string, request_json: string): string;

export function authored_logical_input_replay_v3(authored: string, tape: Uint8Array, binding_json: string): string;

export function authored_project_operation(operation: string, artifact_json: string): string;

export function logical_input_record(source: string, request_json: string): string;

/**
 * Host snapshot recording and replay use the same Rust executor as Native.
 * These entrypoints do not accept old tape versions or raw device inputs.
 */
export function logical_input_record_v3(source: string, request_json: string): string;

export function logical_input_replay(source: string, tape: Uint8Array, binding_json: string): string;

export function logical_input_replay_v3(source: string, tape: Uint8Array, binding_json: string): string;

export function manipulation_input_record(source: string, request_json: string): string;

export function manipulation_input_record_v3(source: string, request_json: string): string;

export function manipulation_input_replay(source: string, tape: Uint8Array, binding_json: string): string;

export function manipulation_input_replay_v3(source: string, tape: Uint8Array, binding_json: string): string;

export function manipulation_value_raw(text: string): string;

export function wasm_apply_maegim_source_edit(source: string, source_path: string, control_name: string, value_text: string): string;

export function wasm_build_info(): string;

export function wasm_canon_alrim_plan(source: string): string;

export function wasm_canon_block_editor_plan(source: string): string;

export function wasm_canon_ddn(source: string): string;

/**
 * 명시적 supported source profile용 내부 candidate canon entry다.
 * stable public WASM API 또는 V1 membership을 선언하지 않는다.
 */
export function wasm_canon_ddn_with_source_profile(source: string, source_profile_identity: string): string;

export function wasm_canon_exec_policy_map(source: string): string;

export function wasm_canon_flat_json(source: string): string;

export function wasm_canon_maegim_plan(source: string): string;

export function wasm_canon_maegim_source_plan(source: string, source_path: string): string;

export function wasm_corpus_columns_json(request_json: string): string;

export function wasm_corpus_csv_intake_json(request_json: string): string;

export function wasm_corpus_evidence_json(request_json: string): string;

export function wasm_corpus_review_json(request_json: string): string;

export function wasm_corpus_seal_json(request_json: string): string;

export function wasm_corpus_sources_json(request_json: string): string;

export function wasm_deterministic_debug_json(operation: string, input_json: string): string;

export function wasm_discover_project_root_json(request_json: string): string;

export function wasm_execution_envelope_json(source: string, requested_ticks: number): string;

/**
 * Existing Tape V1 bytes only; no full-input receipt file replay format.
 */
export function wasm_export_input_tape_v1(receipts_json: string, madi_hz: number): Uint8Array;

export function wasm_gaji_extraction_plan_json(request_json: string): string;

export function wasm_gaji_lifecycle_plan_json(request_json: string): string;

export function wasm_historical_async_migration_evidence_json(source: string, source_path: string): string;

export function wasm_local_gaji_artifact_json(request_json: string): string;

/**
 * Stored candidate execution evidence verifier. It never executes a fallback
 * backend and returns a structured fail-closed verdict for content tampering.
 */
export function wasm_model_execution_verify_json(report_json: string): string;

/**
 * 검사 전용 candidate projection이다. persistent public wire/ABI가 아니다.
 */
export function wasm_model_ir_candidate_json(source: string): string;

export function wasm_model_prepared_state_json(source: string, source_path: string): string;

/**
 * Owner-approved bounded backend execution. The JSON wire remains a
 * versioned candidate and is not a public stable ABI.
 */
export function wasm_model_run_candidate_json(source: string): string;

/**
 * Source labels are provenance only; they never participate in backend
 * selection. This candidate API lets project/CLI parity bind the same label.
 */
export function wasm_model_run_candidate_json_with_source_file(source: string, source_file: string): string;

/**
 * Additive internal candidate carrier for lexer-backed RD-457 metadata.
 * It deliberately does not alter the established source-analysis v1 export.
 */
export function wasm_module_directive_analysis_snapshot_json(request_json: string): string;

export function wasm_normalize_project_source_json(source_json: string): string;

export function wasm_preprocess_source(source: string): string;

/**
 * Returns the bounded runtime-artifact primary-view decision for one run.
 * This is a candidate product ABI for the primary-only wave; callers must
 * render only a selected artifact and surface diagnostics for absent/ambiguous.
 */
export function wasm_primary_view_decision_json(source: string, requested_ticks: number): string;

export function wasm_project_authoring_completion_json(request_json: string): string;

export function wasm_project_authoring_definition_json(request_json: string): string;

export function wasm_project_authoring_hover_json(request_json: string): string;

export function wasm_project_authoring_references_json(request_json: string): string;

export function wasm_project_authoring_rename_preview_json(request_json: string): string;

export function wasm_project_authoring_snapshot_json(request_json: string): string;

export function wasm_project_edit_plan_json(request_json: string): string;

export function wasm_project_execution_recipe_json(request_json: string): string;

export function wasm_project_gaji_lock_json(source_json: string): string;

export function wasm_project_graph_json(source_json: string): string;

export function wasm_project_prepare_link_json(source_json: string, target: string, inspect_graph: boolean, inspect_lock: boolean): string;

export function wasm_project_run_summary_json(source_json: string, target: string, inspect_graph: boolean, inspect_lock: boolean): string;

/**
 * RuntimeBridge/Workbench local-project variant of the same shared
 * host-resource invocation algorithm used by the native product frontdoor.
 */
export function wasm_project_run_summary_with_host_resources_json(source_json: string, target: string, inspect_graph: boolean, inspect_lock: boolean, host_resource_request_json: string): string;

export function wasm_project_source_sha256_json(source_json: string): string;

export function wasm_project_symbols_json(source_json: string): string;

export function wasm_publish_local_gotgan_json(request_json: string): string;

/**
 * Recorded source readback only. No fresh registration/capture, VM, world,
 * runtime restoration, host first-use authority or publication capability.
 */
export function wasm_restore_recorded_source_revision(payload_utf8: string, envelope: Uint8Array, project: Uint8Array, parent: Uint8Array, revision: Uint8Array, integrity: Uint8Array, profile: Uint8Array, pesi: Uint8Array): string;

export function wasm_run_summary_json(source: string, source_path: string, requested_ticks: number): string;

export function wasm_run_summary_with_authored_bogae_json(source: string, source_path: string, requested_ticks: number, source_profile: string, artifact_json: string, receipts_json?: string | null, param_overrides_json?: string | null): string;

/**
 * Typed host-resource request entrypoint for RuntimeBridge/Workbench.
 */
export function wasm_run_summary_with_bogae_entry_json(source: string, source_path: string, requested_ticks: number, source_profile: string, entry: string): string;

/**
 * Typed host-resource request entrypoint for RuntimeBridge/Workbench.
 * The bridge supplies sealed text payloads for record or a sealed transcript
 * for replay; it never interprets their semantic identity itself.
 */
export function wasm_run_summary_with_source_profile_host_resources_json(source: string, source_path: string, requested_ticks: number, source_profile_identity: string, host_resource_request_json: string): string;

/**
 * Typed host-resource request entrypoint for RuntimeBridge/Workbench.
 */
export function wasm_run_summary_with_source_profile_input_receipts_json(source: string, source_path: string, requested_ticks: number, source_profile_identity: string, receipts_json: string, param_overrides_json?: string | null, bogae_entry?: string | null): string;

export function wasm_run_summary_with_source_profile_json(source: string, source_path: string, requested_ticks: number, source_profile_identity: string): string;

export function wasm_run_summary_with_source_profile_tape_v1_json(source: string, source_path: string, source_profile_identity: string, tape_bytes: Uint8Array): string;

export function wasm_search_local_gotgan_json(request_json: string): string;

export function wasm_source_analysis_snapshot_json(request_json: string): string;

export function wasm_verify_local_gaji_artifact_json(artifact_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_ddnlogicalinputvm_free: (a: number, b: number) => void;
    readonly __wbg_ddnmanipulationvm_free: (a: number, b: number) => void;
    readonly __wbg_ddnwasmvm_free: (a: number, b: number) => void;
    readonly authored_logical_input_record_v3: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly authored_logical_input_replay_v3: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly authored_project_operation: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnlogicalinputvm_abort_input: (a: number, b: number, c: number) => void;
    readonly ddnlogicalinputvm_bind_authored_project_json: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnlogicalinputvm_bind_source_json: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnlogicalinputvm_export_project_checkpoint: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly ddnlogicalinputvm_export_saved_project: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly ddnlogicalinputvm_failed_prefix_v3_json: (a: number, b: number) => void;
    readonly ddnlogicalinputvm_from_authored_project: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnlogicalinputvm_from_saved_project_checkpoint: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly ddnlogicalinputvm_new: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnlogicalinputvm_seal_json: (a: number, b: number) => void;
    readonly ddnlogicalinputvm_step_json: (a: number, b: number, c: number, d: number) => void;
    readonly ddnlogicalinputvm_stop_recording_v3_json: (a: number, b: number) => void;
    readonly ddnlogicalinputvm_summary_json: (a: number, b: number) => void;
    readonly ddnlogicalinputvm_view_json: (a: number, b: number) => void;
    readonly ddnmanipulationvm_abort_input: (a: number, b: number, c: number) => void;
    readonly ddnmanipulationvm_bind_source_json: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnmanipulationvm_failed_prefix_v3_json: (a: number, b: number) => void;
    readonly ddnmanipulationvm_new: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnmanipulationvm_seal_json: (a: number, b: number) => void;
    readonly ddnmanipulationvm_step_json: (a: number, b: number, c: number, d: number) => void;
    readonly ddnmanipulationvm_stop_recording_v3_json: (a: number, b: number) => void;
    readonly ddnmanipulationvm_summary_json: (a: number, b: number) => void;
    readonly ddnmanipulationvm_view_json: (a: number, b: number) => void;
    readonly ddnwasmvm_add_view_prefix: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_apply_currentline_cell: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly ddnwasmvm_apply_currentline_cell_with_source_profile: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly ddnwasmvm_clear_ai_injections: (a: number) => void;
    readonly ddnwasmvm_clear_debug_plan: (a: number) => number;
    readonly ddnwasmvm_clear_view_prefixes: (a: number) => void;
    readonly ddnwasmvm_columns: (a: number) => number;
    readonly ddnwasmvm_configure_authored_bogae: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_configure_bogae_entry: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_configure_debug_plan: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_configure_project_assets: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_configure_project_entry: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_export_saved_project_artifact_json: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly ddnwasmvm_fixed64_from_decimal_v1: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_from_authored_project_json: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_from_native_project_json: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_from_saved_project_artifact_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly ddnwasmvm_get_build_info: (a: number, b: number) => void;
    readonly ddnwasmvm_get_configured_madi: (a: number) => bigint;
    readonly ddnwasmvm_get_debug_source_identity_json: (a: number, b: number) => void;
    readonly ddnwasmvm_get_execution_mode_decision_json: (a: number, b: number) => void;
    readonly ddnwasmvm_get_input_state_json: (a: number) => number;
    readonly ddnwasmvm_get_parse_warnings: (a: number) => number;
    readonly ddnwasmvm_get_state_hash: (a: number, b: number) => void;
    readonly ddnwasmvm_get_state_json: (a: number) => number;
    readonly ddnwasmvm_inject_ai_action: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnwasmvm_is_native_source_bound: (a: number) => number;
    readonly ddnwasmvm_new: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_new_with_mode: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnwasmvm_new_with_source_path: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnwasmvm_reset: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_restore_state: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_run_ticks: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_save_native_source_checkpoint: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_set_dt_fixed64_v1: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_set_input_fixed64_v1: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly ddnwasmvm_set_keys_pressed: (a: number, b: number) => void;
    readonly ddnwasmvm_set_last_key_name: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_set_param: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly ddnwasmvm_set_param_fixed64: (a: number, b: number, c: number, d: number, e: bigint) => void;
    readonly ddnwasmvm_set_param_fixed64_str: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly ddnwasmvm_set_pointer: (a: number, b: number, c: number) => void;
    readonly ddnwasmvm_set_rng_seed: (a: number, b: bigint) => void;
    readonly ddnwasmvm_step_one: (a: number, b: number) => void;
    readonly ddnwasmvm_step_one_with_input_fixed64_v1: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly ddnwasmvm_update_logic: (a: number, b: number, c: number, d: number) => void;
    readonly ddnwasmvm_update_logic_with_mode: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly ddnwasmvm_update_logic_with_source_path: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly ddnwasmvm_update_logic_with_source_profile: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly logical_input_record: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly logical_input_record_v3: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly logical_input_replay: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly logical_input_replay_v3: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly manipulation_input_record: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly manipulation_input_record_v3: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly manipulation_input_replay: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly manipulation_input_replay_v3: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly manipulation_value_raw: (a: number, b: number, c: number) => void;
    readonly wasm_apply_maegim_source_edit: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly wasm_build_info: (a: number) => void;
    readonly wasm_canon_alrim_plan: (a: number, b: number, c: number) => void;
    readonly wasm_canon_block_editor_plan: (a: number, b: number, c: number) => void;
    readonly wasm_canon_ddn: (a: number, b: number, c: number) => void;
    readonly wasm_canon_ddn_with_source_profile: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasm_canon_exec_policy_map: (a: number, b: number, c: number) => void;
    readonly wasm_canon_flat_json: (a: number, b: number, c: number) => void;
    readonly wasm_canon_maegim_plan: (a: number, b: number, c: number) => void;
    readonly wasm_canon_maegim_source_plan: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasm_corpus_columns_json: (a: number, b: number, c: number) => void;
    readonly wasm_corpus_csv_intake_json: (a: number, b: number, c: number) => void;
    readonly wasm_corpus_evidence_json: (a: number, b: number, c: number) => void;
    readonly wasm_corpus_review_json: (a: number, b: number, c: number) => void;
    readonly wasm_corpus_seal_json: (a: number, b: number, c: number) => void;
    readonly wasm_corpus_sources_json: (a: number, b: number, c: number) => void;
    readonly wasm_deterministic_debug_json: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasm_discover_project_root_json: (a: number, b: number, c: number) => void;
    readonly wasm_execution_envelope_json: (a: number, b: number, c: number, d: number) => void;
    readonly wasm_export_input_tape_v1: (a: number, b: number, c: number, d: number) => void;
    readonly wasm_gaji_extraction_plan_json: (a: number, b: number, c: number) => void;
    readonly wasm_gaji_lifecycle_plan_json: (a: number, b: number, c: number) => void;
    readonly wasm_historical_async_migration_evidence_json: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasm_local_gaji_artifact_json: (a: number, b: number, c: number) => void;
    readonly wasm_model_execution_verify_json: (a: number, b: number, c: number) => void;
    readonly wasm_model_ir_candidate_json: (a: number, b: number, c: number) => void;
    readonly wasm_model_prepared_state_json: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasm_model_run_candidate_json: (a: number, b: number, c: number) => void;
    readonly wasm_model_run_candidate_json_with_source_file: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasm_module_directive_analysis_snapshot_json: (a: number, b: number, c: number) => void;
    readonly wasm_normalize_project_source_json: (a: number, b: number, c: number) => void;
    readonly wasm_preprocess_source: (a: number, b: number, c: number) => void;
    readonly wasm_primary_view_decision_json: (a: number, b: number, c: number, d: number) => void;
    readonly wasm_project_authoring_completion_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_authoring_definition_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_authoring_hover_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_authoring_references_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_authoring_rename_preview_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_authoring_snapshot_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_edit_plan_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_execution_recipe_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_gaji_lock_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_graph_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_prepare_link_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wasm_project_run_summary_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wasm_project_run_summary_with_host_resources_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly wasm_project_source_sha256_json: (a: number, b: number, c: number) => void;
    readonly wasm_project_symbols_json: (a: number, b: number, c: number) => void;
    readonly wasm_publish_local_gotgan_json: (a: number, b: number, c: number) => void;
    readonly wasm_restore_recorded_source_revision: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number, q: number) => void;
    readonly wasm_run_summary_json: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wasm_run_summary_with_authored_bogae_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => void;
    readonly wasm_run_summary_with_bogae_entry_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly wasm_run_summary_with_source_profile_host_resources_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly wasm_run_summary_with_source_profile_input_receipts_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => void;
    readonly wasm_run_summary_with_source_profile_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly wasm_run_summary_with_source_profile_tape_v1_json: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly wasm_search_local_gotgan_json: (a: number, b: number, c: number) => void;
    readonly wasm_source_analysis_snapshot_json: (a: number, b: number, c: number) => void;
    readonly wasm_verify_local_gaji_artifact_json: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
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
