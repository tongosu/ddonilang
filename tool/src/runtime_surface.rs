use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use blake3::hash as blake3_hash;
pub use ddonirang_core::platform::InputSource;
pub use ddonirang_lang::AgeTarget;

use ddonirang_core::platform::{DetNuri, InputSnapshot, Patch, PatchOp, ResourceValue};
use ddonirang_core::{
    ExternalBoundaryFailure, ExternalBoundaryRequest, ExternalBoundaryResult, Fixed64, Nuri,
    ProcessLocalCapabilityStore, ResourceHandle, SealedBoundaryProvider, SealedExecutionLedger,
    SeulgiIntent,
};
use ddonirang_lang::runtime::Value;
use ddonirang_lang::{ExecutionModeDecision, ParseMode};
use serde_json::{json, Map, Value as JsonValue};

use crate::canon::CanonOutput;
use crate::ddn_runtime::{
    formula_from_resource_value, formula_summary_parts, resolve_supported_source_profile,
    DdnProgram, DdnRunner, SupportedSourceProfile, E_V1_CORE_RECURSION_UNSUPPORTED,
    V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
};
use crate::deterministic_debug::{
    build_public_observation_table_projection, build_public_observation_trace,
    public_observation_events_json,
};
use crate::gate0_registry::ProjectAssetRegistry;
use crate::host_resource::PreparedHostResources;
use crate::input_tape::{encode_input_tape, mask_from_bytes, tape_mask_to_runtime_keys, InputRecord, InputTape};
use crate::preprocess::{
    execution_mode_decision_from_source_with_path, preprocess_source_for_parse, split_file_meta,
    validate_no_legacy_boim_surface, validate_no_legacy_header, ExecutionDiagnostic,
};
use crate::unit_wire::{
    resource_value_to_detjson, RESOURCE_SNAPSHOT_SCHEMA_V2, UNIT_VALUE_WIRE_SCHEMA_V2,
};
use crate::work_cost::{StandardResourceLimitsV1, WorkCostFailure};

/// One typed runtime diagnostic projection for CLI and persistent WASM steps.
pub(crate) fn execution_error_diagnostic(error: crate::ddn_runtime::EvalError, file_path: &str) -> ExecutionDiagnostic {
    let (code, message, span) = error.diagnostic_parts();
    let phase = error.diagnostic_phase();
    let (offending_field, expected, actual) = match &error {
        crate::ddn_runtime::EvalError::TypeMismatch { pin, expected, actual } =>
            (Some(pin.clone()), Some(expected.clone()), Some(actual.clone())),
        _ => (None, None, None),
    };
    ExecutionDiagnostic { code, message, source_path: file_path.to_string(), span,
        phase, cause: None, offending_field, expected, actual }
}

#[cfg(test)]
thread_local! {
    static RUN_SUMMARY_DIAGNOSTIC_INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FORCE_HOST_PREFLIGHT_REJECTION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn reset_run_summary_diagnostic_invocations() {
    RUN_SUMMARY_DIAGNOSTIC_INVOCATIONS.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn run_summary_diagnostic_invocations() -> usize {
    RUN_SUMMARY_DIAGNOSTIC_INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
fn set_force_host_preflight_rejection(value: bool) {
    FORCE_HOST_PREFLIGHT_REJECTION.with(|flag| flag.set(value));
}

fn preflight_observation_frames(
    runner: &DdnRunner,
    capacity: usize,
) -> Result<Vec<JsonValue>, WorkCostFailure> {
    #[cfg(test)]
    if FORCE_HOST_PREFLIGHT_REJECTION.with(std::cell::Cell::get) {
        return Err(runner.host_preflight_rejected("observation_frames.try_reserve_exact:fault"));
    }
    let mut frames = Vec::new();
    frames.try_reserve_exact(capacity).map_err(|error| {
        runner.host_preflight_rejected(format!(
            "observation_frames.try_reserve_exact:{capacity}:{error}"
        ))
    })?;
    Ok(frames)
}

pub const DEFAULT_UPDATE_NAME: &str = "매마디";
pub const ENGINE_RESPONSE_SCHEMA: &str = "seamgrim.engine_response.v0";
pub const DEFAULT_VIEW_PREFIXES: &[&str] = &["보개_", "__view_"];

const BOGAE_DRAWLIST_TAG: &str = "보개_그림판_목록";
const BOGAE_WIDTH_TAG: &str = "보개_그림판_가로";
const BOGAE_HEIGHT_TAG: &str = "보개_그림판_세로";

/// Internal structured-consumer owner for the sealed external boundary.
/// Native and WASM adapters both call this function; it neither parses a new
/// wire format nor selects a provider from ambient state.
pub(crate) fn execute_sealed_external_boundary(
    ledger: &mut SealedExecutionLedger,
    capability_store: &ProcessLocalCapabilityStore,
    provider: &dyn SealedBoundaryProvider,
    request: &ExternalBoundaryRequest,
) -> Result<ExternalBoundaryResult, ExternalBoundaryFailure> {
    ledger.execute(capability_store, provider, request)
}
const BOGAE_SHOW_LINES_TAG: &str = "보개_출력_줄들";
const GRAPH_SCHEMA: &str = "seamgrim.graph.v0";
const GRAPH_POINTS_TAGS: &[&str] = &[
    "보개_그래프_점목록_f",
    "그래프_점목록_f",
    "그래프_점목록_df",
    "그래프_점목록_fi",
    "graph_points_f",
    "graph_points_df",
    "graph_points_fi",
];

#[derive(Debug, Clone)]
pub struct PreparedSource {
    pub stripped_source: String,
    pub preprocessed_source: String,
    pub execution_source: String,
    pub canonical_ddn: String,
    pub meta_lines: usize,
    pub execution_mode_decision: ExecutionModeDecision,
}

// Historical adapter only. Current callers must use the explicit profile API.
// Never select a grammar epoch from tokens in the document (RD-492).
fn program_from_legacy_source(
    source: &str,
    source_path: &str,
    mode: ParseMode,
) -> Result<DdnProgram, String> {
    DdnProgram::from_source_with_mode(source, source_path, mode)
}

fn program_from_legacy_source_with_age_target(
    source: &str,
    source_path: &str,
    mode: ParseMode,
    age_target: AgeTarget,
) -> Result<DdnProgram, String> {
    DdnProgram::from_source_with_mode_and_age_target(source, source_path, mode, age_target)
}

pub fn prepare_source_for_execution(source: &str) -> Result<PreparedSource, String> {
    prepare_source_for_execution_with_path(source, "<execution-source>")
        .map_err(|err| err.to_string())
}

pub fn prepare_source_for_execution_with_path(
    source: &str,
    source_path: &str,
) -> Result<PreparedSource, ExecutionDiagnostic> {
    if ddonirang_lang::has_model_surface(source) {
        ddonirang_lang::parse_model_program(source).map_err(|err| ExecutionDiagnostic {
            code: "E_MODEL_FRONTDOOR",
            message: err.to_string(),
            source_path: source_path.to_string(),
            span: None,
            phase: "model_frontdoor",
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        })?;
        return Err(ExecutionDiagnostic {
            code: "E_MODEL_PREPROCESS_ROUTE_CANDIDATE",
            message:
                "모형 원문은 legacy DDN preprocess가 아니라 typed model 실행 경로를 사용해야 합니다"
                    .to_string(),
            source_path: source_path.to_string(),
            span: None,
            phase: "route_selection",
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        });
    }
    let meta = split_file_meta(source);
    validate_no_legacy_header(&meta.stripped).map_err(|message| ExecutionDiagnostic {
        code: "E_EXECUTION_PREPROCESS",
        message,
        source_path: source_path.to_string(),
        span: None,
        phase: "preprocess",
        cause: None,
        offending_field: None,
        expected: None,
        actual: None,
    })?;
    validate_no_legacy_boim_surface(&meta.stripped).map_err(|message| ExecutionDiagnostic {
        code: "E_EXECUTION_PREPROCESS",
        message,
        source_path: source_path.to_string(),
        span: None,
        phase: "preprocess",
        cause: None,
        offending_field: None,
        expected: None,
        actual: None,
    })?;
    let execution_mode_decision =
        execution_mode_decision_from_source_with_path(&meta.stripped, source_path)?;
    let preprocessed = preprocess_source_for_parse(&meta.stripped).map_err(|message| ExecutionDiagnostic {
            code: "E_EXECUTION_PREPROCESS",
            message,
            source_path: source_path.to_string(),
            span: None,
            phase: "preprocess",
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        })?;
    let execution_source = ddonirang_lang::preprocess_frontdoor_source(&preprocessed);
    if let Err(err) = ddonirang_lang::parse_with_mode(&execution_source, source_path, ParseMode::Strict) {
        if matches!(
            err.code(),
            "E_FIELD_MISSING" | "E_FIELD_TARGET_TYPE" | "E_FIELD_POSSESSIVE_SURFACE_FORBIDDEN"
        ) {
            let shift = execution_source.find(&preprocessed).unwrap_or(0);
            let span = (
                err.span.start.saturating_sub(shift),
                err.span.end.saturating_sub(shift),
            );
            return Err(ExecutionDiagnostic {
                code: err.code(),
                message: err.message,
                source_path: source_path.to_string(),
                span: Some(span),
                phase: "lang_parse",
                cause: None,
                offending_field: None,
                expected: None,
                actual: None,
            });
        }
    }
    let canonical_ddn = program_from_legacy_source(source, source_path, ParseMode::Strict)
        .map_err(|message| ExecutionDiagnostic {
            code: "E_SHARED_CANONICAL",
            message,
            source_path: source_path.to_string(),
            span: None,
            phase: "canonicalization",
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        })?
        .canonical_ddn();
    Ok(PreparedSource {
        stripped_source: meta.stripped,
        preprocessed_source: preprocessed,
        execution_source,
        canonical_ddn,
        meta_lines: meta.meta_lines,
        execution_mode_decision,
    })
}

pub(crate) fn prepare_supported_source_for_execution_with_program(
    source: &str,
    source_path: &str,
    program: &DdnProgram,
) -> Result<PreparedSource, ExecutionDiagnostic> {
    if ddonirang_lang::has_model_surface(source) {
        return Err(ExecutionDiagnostic {
            code: "E_MODEL_PREPROCESS_ROUTE_CANDIDATE",
            message: "모형 원문은 V1-Core supported source profile 대상이 아닙니다".to_string(),
            source_path: source_path.to_string(),
            span: None,
            phase: "route_selection",
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        });
    }
    let meta = split_file_meta(source);
    let preprocessed = if program.has_root_program() {
        meta.stripped.clone()
    } else {
        preprocess_source_for_parse(&meta.stripped).map_err(|message| ExecutionDiagnostic {
            code: "E_EXECUTION_PREPROCESS",
            message,
            source_path: source_path.to_string(),
            span: None,
            phase: "preprocess",
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        })?
    };
    let execution_source = if program.has_root_program() {
        preprocessed.clone()
    } else {
        ddonirang_lang::preprocess_frontdoor_source(&preprocessed)
    };
    Ok(PreparedSource {
        stripped_source: meta.stripped,
        preprocessed_source: preprocessed,
        execution_source,
        canonical_ddn: program.canonical_ddn(),
        meta_lines: meta.meta_lines,
        execution_mode_decision: program.execution_mode_decision(),
    })
}

// Preserve structured parser failures at the existing string-returning
// DdnProgram boundary. Reparse only a rejected input, with the same explicit
// epoch; never retry execution or route it through legacy preprocessing.
fn supported_source_parse_diagnostic(
    source: &str, path: &str, message: &str,
) -> Option<ExecutionDiagnostic> {
    if source.starts_with('\u{feff}') {
        return Some(run_summary_diagnostic(
            "E_SOURCE_UTF8_BOM_FORBIDDEN", message.to_string(), path, "source_ingress",
        ));
    }
    if message.starts_with("E_BOGAE_TIME_POLICY_UNSUPPORTED:") {
        return Some(run_summary_diagnostic(
            "E_BOGAE_TIME_POLICY_UNSUPPORTED", message.to_string(), path, "source_ingress",
        ));
    }
    let stripped = split_file_meta(source).stripped;
    let error = ddonirang_lang::parse_v25_with_mode(&stripped, path, ParseMode::Strict).err()?;
    (error.code() != "E_PARSE" && message.contains(error.code()))
        .then(|| {
            let mut diagnostic = ExecutionDiagnostic::execution_mode_parse(error, path);
            // DdnProgram already formats the original line/column and caret.
            // Keep that same message at CLI and WASM frontdoors.
            diagnostic.message = message.to_string();
            diagnostic
        })
}

pub fn canonicalize_shared_source(source: &str) -> Result<CanonOutput, String> {
    if ddonirang_lang::has_model_surface(source) {
        let program = ddonirang_lang::parse_model_program(source)
            .map_err(|err| format!("E_MODEL_FRONTDOOR: {err}"))?;
        return Ok(CanonOutput {
            ddn: program.canonical_ddn,
            guseong_flat_json: "{}\n".to_string(),
            alrim_plan_json: "{}\n".to_string(),
            block_editor_plan_json: "{}\n".to_string(),
            exec_policy_map_json: "{}\n".to_string(),
            maegim_control_json: "{}\n".to_string(),
            meta: crate::file_meta::split_file_meta(source).meta,
            warnings: Vec::new(),
        });
    }
    let meta = crate::file_meta::split_file_meta(source);
    let program =
        program_from_legacy_source(source, "<shared-canonical>", ParseMode::Strict)?;
    let warnings = program
        .parse_warnings()
        .iter()
        .map(|warning| format!("{} {}", warning.code, warning.message))
        .collect();
    Ok(CanonOutput {
        ddn: program.canonical_ddn(),
        guseong_flat_json: "{}\n".to_string(),
        alrim_plan_json: "{}\n".to_string(),
        block_editor_plan_json: "{}\n".to_string(),
        exec_policy_map_json: "{}\n".to_string(),
        maegim_control_json: "{}\n".to_string(),
        meta: meta.meta,
        warnings,
    })
}

pub fn canonicalize_shared_source_with_supported_profile(
    source: &str,
    source_profile_identity: Option<&str>,
) -> Result<CanonOutput, String> {
    let profile = resolve_supported_source_profile(source_profile_identity)
        .map_err(|error| format!("{}: {}", error.code, error.message))?;
    let meta = crate::file_meta::split_file_meta(source);
    let program = DdnProgram::from_supported_source_profile_with_mode(
        source,
        "<shared-canonical-supported-profile>",
        ParseMode::Strict,
        profile,
    )?;
    let warnings = program
        .parse_warnings()
        .iter()
        .map(|warning| format!("{} {}", warning.code, warning.message))
        .collect();
    Ok(CanonOutput {
        ddn: program.canonical_ddn(),
        guseong_flat_json: "{}\n".to_string(),
        alrim_plan_json: "{}\n".to_string(),
        block_editor_plan_json: "{}\n".to_string(),
        exec_policy_map_json: "{}\n".to_string(),
        maegim_control_json: "{}\n".to_string(),
        meta: meta.meta,
        warnings,
    })
}

pub fn model_ir_candidate_json_from_source(source: &str) -> Result<String, String> {
    let program = ddonirang_lang::parse_model_program(source)
        .map_err(|err| format!("E_MODEL_FRONTDOOR: {err}"))?;
    serde_json::to_string_pretty(&ddonirang_lang::model_ir_candidate_json(&program))
        .map(|json| format!("{json}\n"))
        .map_err(|err| format!("E_MODEL_IR_CANDIDATE_SERIALIZE: {err}"))
}

pub fn model_execution_verify_json(report_json: &str) -> Result<String, String> {
    let report: JsonValue = serde_json::from_str(report_json)
        .map_err(|err| format!("E_MODEL_EXECUTION_VERIFY_JSON_CANDIDATE: {err}"))?;
    Ok(crate::model_runtime::verify_execution_report(&report).to_string())
}

pub fn canonicalize_shared_ddn(source: &str) -> Result<String, String> {
    canonicalize_shared_source(source).map(|output| output.ddn)
}

pub fn historical_async_migration_evidence_json(
    source: &str,
    source_path: &str,
) -> Result<String, String> {
    let evidence = ddonirang_lang::historical_async_migration_evidence(source, source_path)
        .map_err(|err| format!("{}: {err}", err.code()))?;
    let artifacts = evidence
        .iter()
        .map(|item| {
            json!({
                "diagnostic_code": item.diagnostic_code,
                "historical_identity": item.historical_identity,
                "original_surface": item.original_surface,
                "source_span": {
                    "end": item.source_span.end,
                    "start": item.source_span.start,
                },
                "successor_replacement": item.successor_replacement,
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&json!({
        "artifacts": artifacts,
        "diagnostic_code": ddonirang_lang::HISTORICAL_ASYNC_MIGRATION_DIAGNOSTIC_CODE,
        "parser_mode": "HistoricalMigrationEvidence",
        "product_execution_authorized": false,
        "schema": "ddn.historical_async_migration_evidence.v1",
        "source_path": source_path,
    }))
    .map_err(|err| format!("E_HISTORICAL_ASYNC_MIGRATION_ARTIFACT_SERIALIZE: {err}"))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn sha256_identity(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

pub fn current_state_hash(world: &ddonirang_core::platform::NuriWorld) -> String {
    format!(
        "blake3:{}",
        world
            .state_hash_excluding_resource_prefixes(DEFAULT_VIEW_PREFIXES)
            .to_hex()
    )
}

pub fn execution_envelope_from_source(
    source: &str,
    requested_ticks: Option<u32>,
) -> Result<JsonValue, String> {
    execution_envelope_from_source_with_path(source, "<execution-envelope>", requested_ticks)
        .map_err(|err| err.to_string())
}

pub fn execution_envelope_from_source_with_path(
    source: &str,
    source_path: &str,
    requested_ticks: Option<u32>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    execution_envelope_internal(source, source_path, requested_ticks, None)
}

pub fn execution_envelope_from_source_with_profile(
    source: &str,
    source_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    let profile = resolve_supported_source_profile(source_profile_identity)
        .map_err(|error| run_summary_diagnostic(error.code, error.message, source_path, "source_profile"))?;
    execution_envelope_internal(source, source_path, requested_ticks, Some(profile))
}

fn execution_envelope_internal(
    source: &str,
    source_path: &str,
    requested_ticks: Option<u32>,
    profile: Option<SupportedSourceProfile>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    if ddonirang_lang::has_model_surface(source) {
        validate_model_oneshot_ticks(requested_ticks).map_err(|message| ExecutionDiagnostic {
            code: "E_MODEL_TICK_POLICY",
            message,
            source_path: source_path.to_string(),
            span: None,
            phase: "model_execution",
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        })?;
        return crate::model_runtime::execution_envelope_json(source).map_err(|err| {
            let mut diagnostic = model_execution_diagnostic(err, source);
            diagnostic.source_path = source_path.to_string();
            if let Some(cause) = diagnostic.cause.as_mut() {
                cause.source_path = source_path.to_string();
            }
            diagnostic
        });
    }
    let program = match profile {
        Some(profile) => DdnProgram::from_supported_source_profile_with_mode(source, source_path, ParseMode::Strict, profile),
        None => program_from_legacy_source(source, source_path, ParseMode::Strict),
    }
        .map_err(|message| {
            if profile.is_some() {
                if let Some(diagnostic) = supported_source_parse_diagnostic(source, source_path, &message) {
                    return diagnostic;
                }
            }
            ExecutionDiagnostic {
            code: "E_PARSE",
            message,
            source_path: source_path.to_string(),
            span: None,
            phase: "execution_parse",
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        }})?;
    let prepared = match profile {
        Some(_) => prepare_supported_source_for_execution_with_program(source, source_path, &program)?,
        None => prepare_source_for_execution_with_path(source, source_path)?,
    };
    let configured_madi = program.configured_madi().unwrap_or(0);
    let execution_mode_decision = prepared.execution_mode_decision;
    let effective_ticks = requested_ticks
        .map(u64::from)
        .unwrap_or_else(|| configured_madi.max(1))
        .max(1);
    let mut envelope = build_execution_envelope(
        source,
        &prepared,
        configured_madi,
        execution_mode_decision,
        requested_ticks,
        effective_ticks,
        Fixed64::from_i64(1),
        0,
    );
    let sealed_dataset_assets = JsonValue::Array(Vec::new());
    let sealed_dataset_identity_sha256 = sha256_hex(
        serde_json::to_string(&sealed_dataset_assets)
            .map_err(|err| ExecutionDiagnostic {
                code: "E_DATASET_IDENTITY_SERIALIZE",
                message: err.to_string(),
                source_path: source_path.to_string(),
                span: None,
                phase: "execution_envelope",
                cause: None,
                offending_field: None,
                expected: None,
                actual: None,
            })?
            .as_bytes(),
    );
    if let Some(object) = envelope.as_object_mut() {
        object.insert("sealed_dataset_assets".to_string(), sealed_dataset_assets);
        object.insert(
            "sealed_dataset_identity_sha256".to_string(),
            JsonValue::String(sealed_dataset_identity_sha256),
        );
    }
    Ok(envelope)
}

fn build_execution_envelope(
    source: &str,
    prepared: &PreparedSource,
    configured_madi: u64,
    execution_mode_decision: ExecutionModeDecision,
    requested_ticks: Option<u32>,
    effective_ticks: u64,
    input_dt: Fixed64,
    rng_base_seed: u64,
) -> JsonValue {
    json!({
        "source_bytes_sha256": sha256_hex(source.as_bytes()),
        "stripped_source_sha256": sha256_hex(prepared.stripped_source.as_bytes()),
        "preprocessed_source_sha256": sha256_hex(prepared.execution_source.as_bytes()),
        "canonical_ddn_sha256": sha256_hex(prepared.canonical_ddn.as_bytes()),
        "input_snapshot": {
            "keys_pressed": 0,
            "last_key_name": "",
            "pointer_x_i32": 0,
            "pointer_y_i32": 0,
            "dt": input_dt.to_string(),
            "frame_source": InputSource::Person.label(),
            "frame_source_code": InputSource::Person.code_u8(),
            "rng_base_seed": rng_base_seed
        },
        "frame_source": InputSource::Person.label(),
        "frame_source_code": InputSource::Person.code_u8(),
        "rng_seed": rng_base_seed,
        "dt": input_dt.to_string(),
        "configured_madi": configured_madi,
        "execution_mode_decision": execution_mode_decision_json(execution_mode_decision),
        "requested_ticks": requested_ticks.map(u64::from),
        "effective_ticks": effective_ticks,
        "dialect_or_mode": "strict",
        "runtime_schema": ENGINE_RESPONSE_SCHEMA,
        "resource_snapshot_schema": RESOURCE_SNAPSHOT_SCHEMA_V2,
        "unit_wire_schema": UNIT_VALUE_WIRE_SCHEMA_V2
    })
}

pub fn execution_mode_decision_json(decision: ExecutionModeDecision) -> JsonValue {
    json!({
        "schema": "ddn.execution_mode_decision.v1",
        "mode": decision.mode.as_str(),
        "authority": "parsed_program",
        "reason": decision.reason(),
        "live_hook_count": decision.live_hook_count,
        "every_madi_hook_count": decision.every_madi_hook_count,
        "every_n_madi_hook_count": decision.every_n_madi_hook_count,
    })
}

pub fn run_summary_from_source(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
) -> Result<JsonValue, String> {
    run_summary_from_source_diagnostic(source, file_path, requested_ticks)
        .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_source_with_seed(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    rng_base_seed: u64,
) -> Result<JsonValue, String> {
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        None,
        None,
        rng_base_seed,
    )
    .map_err(|err| err.to_json_value().to_string())
}

/// Runs the shared runtime with one already-admitted held-key mask per tick.
/// The CLI owns host collection; this owner remains responsible for parsing,
/// typed execution, and the all-or-nothing public summary boundary.
pub fn run_summary_from_source_with_live_input<F>(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    rng_base_seed: u64,
    source_profile_identity: Option<&str>,
    madi_hz: u32,
    mut next_mask: F,
) -> Result<JsonValue, String>
where
    F: FnMut(u64) -> Result<InputRecord, String>,
{
    let profile = source_profile_identity.map(|identity| resolve_supported_source_profile(Some(identity)))
        .transpose().map_err(|error| error.message)?;
    if madi_hz == 0 {
        return Err("E_SAM_BAD_MADI_HZ --초당-마디(--madi-hz)는 0보다 커야 합니다".into());
    }
    run_summary_from_source_with_assets_diagnostic_with_update_name(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        profile,
        None,
        rng_base_seed,
        DEFAULT_UPDATE_NAME,
        Some((madi_hz, &mut next_mask)),
        false,
        None,
        None,
        None,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_source_with_age_target(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    age_target: AgeTarget,
) -> Result<JsonValue, String> {
    run_summary_from_source_with_age_target_diagnostic(
        source,
        file_path,
        requested_ticks,
        age_target,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_source_with_project_assets(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    project_asset_registry: Arc<ProjectAssetRegistry>,
) -> Result<JsonValue, String> {
    run_summary_from_source_with_project_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        project_asset_registry,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_source_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        None,
        None,
        0,
    )
}

pub fn run_summary_from_source_with_age_target_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    age_target: AgeTarget,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        Some(age_target),
        None,
        None,
        0,
    )
}

pub fn run_summary_from_supported_source_profile_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
    })?;
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        Some(profile),
        None,
        0,
    )
}

/// Runs a current source through the supported profile using the caller's
/// already-selected update callable.  Local Project targets are typed source
/// identities, not a request to rewrite that callable to the legacy default.
pub fn run_summary_from_supported_source_profile_with_update_name_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    update_name: &str,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
    })?;
    run_summary_from_source_with_assets_diagnostic_with_update_name(
        source,
        file_path,
        requested_ticks,
        project_asset_registry,
        None,
        None,
        Some(profile),
        None,
        0,
        update_name,
        None,
        true,
        None,
        None,
        None,
    )
}

pub fn run_summary_from_supported_source_profile_with_receipts(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    receipts: &JsonValue,
    param_overrides: Option<&BTreeMap<String, Value>>,
) -> Result<JsonValue, String> {
    run_summary_with_receipts_and_bogae_entry(source,file_path,requested_ticks,
        source_profile_identity,receipts,param_overrides,None)
}

pub fn run_summary_with_receipts_and_bogae_entry(
    source: &str, file_path: &str, requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>, receipts: &JsonValue,
    param_overrides: Option<&BTreeMap<String, Value>>, bogae_entry: Option<&str>,
) -> Result<JsonValue, String> {
    let profile = resolve_supported_source_profile(source_profile_identity)
        .map_err(|error| error.message)?;
    // In-process host admission only. A saved run summary is not an approved
    // Alpha input format and must never be accepted as an executable record.
    let records = receipts.as_array()
        .ok_or("E_INPUT_RECEIPT_INVALID: 실행 중 전달된 입력 영수증 목록만 받습니다")?;
    let inputs = records.iter().map(crate::input_receipt::parse_receipt_snapshot)
        .collect::<Result<Vec<_>, _>>()?;
    let summary = run_summary_from_source_with_assets_diagnostic_with_update_name(
        source, file_path, requested_ticks, None, None, None, Some(profile),
        None, 0, DEFAULT_UPDATE_NAME, None, false, Some(&inputs), param_overrides, bogae_entry,
    ).map_err(|error| error.to_json_value().to_string())?;
    Ok(summary)
}

/// Explicit product activation selection. It never rewrites source or guesses
/// a root by spelling/order, and it creates no persistent authored identity.
pub fn run_summary_with_bogae_entry(
    source: &str, file_path: &str, requested_ticks: Option<u32>,
    profile_identity: &str, entry: &str,
) -> Result<JsonValue, String> {
    let profile=resolve_supported_source_profile(Some(profile_identity)).map_err(|error|error.message)?;
    run_summary_from_source_with_assets_diagnostic_with_update_name(
        source,file_path,requested_ticks,None,None,None,Some(profile),None,0,
        DEFAULT_UPDATE_NAME,None,false,None,None,Some(entry),
    ).map_err(|error|error.to_json_value().to_string())
}

/// Permanent identity is admitted from the sealed authoring artifact. Receipts
/// remain in-process input evidence, not a new executable recording format.
pub fn run_summary_with_authored_bogae(
    source: &str, file_path: &str, requested_ticks: Option<u32>,
    profile_identity: &str, artifact_json: &str,
    receipts: Option<&JsonValue>, param_overrides: Option<&BTreeMap<String, Value>>,
) -> Result<JsonValue, String> {
    let profile = resolve_supported_source_profile(Some(profile_identity))
        .map_err(|error| error.message)?;
    let project = crate::authored_project::open(artifact_json.as_bytes())?;
    if project.source_profile != profile_identity {
        return Err("E_AUTHORED_BOGAE_PROFILE: 작성 자료와 실행 프로필이 다릅니다".into());
    }
    let (authored_source, _, project_start) = project.execution_source()?;
    if authored_source != source {
        return Err("E_AUTHORED_BOGAE_SOURCE: 실행 원문과 작성 자료의 공통 연결 원문이 다릅니다".into());
    }
    let inputs = receipts.map(|receipts| {
        receipts.as_array().ok_or_else(|| "E_INPUT_RECEIPT_INVALID: 입력 영수증 목록만 받습니다".to_string())?
            .iter().map(|row| {
                let input = crate::input_receipt::parse_receipt_snapshot(row)?;
                crate::input_receipt::validate_alpha_host_dt(input.dt)?;
                Ok(input)
            }).collect::<Result<Vec<_>, String>>()
    }).transpose()?;
    run_summary_from_source_with_authored_bogae(
        source, file_path, requested_ticks, None, None, None, Some(profile),
        None, 0, project_start.as_deref().unwrap_or(DEFAULT_UPDATE_NAME), None,
        project_start.is_some(), inputs.as_deref(),
        param_overrides, None, Some(artifact_json),
    ).map_err(|error| error.to_json_value().to_string())
}

pub fn run_summary_from_supported_source_profile(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
) -> Result<JsonValue, String> {
    run_summary_from_supported_source_profile_diagnostic(
        source,
        file_path,
        requested_ticks,
        source_profile_identity,
    )
    .map_err(|err| err.to_json_value().to_string())
}

/// Runs the ordinary shared runtime only after the declared host resources
/// have already been sealed.  The resource receipt is a product result field,
/// not a CLI-local annotation, so native and WASM consumers receive the same
/// identity-bearing payload.
pub fn run_summary_from_supported_source_profile_with_host_resources(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    host_resources: &PreparedHostResources,
) -> Result<JsonValue, String> {
    let mut summary = run_summary_from_supported_source_profile(
        source,
        file_path,
        requested_ticks,
        source_profile_identity,
    )?;
    attach_host_resource_receipt(&mut summary, host_resources);
    Ok(summary)
}

pub fn attach_host_resource_receipt(
    summary: &mut JsonValue,
    host_resources: &PreparedHostResources,
) {
    if let Some(object) = summary.as_object_mut() {
        object.insert(
            "host_resource_transcript".to_string(),
            serde_json::to_value(&host_resources.transcript)
                .expect("host resource transcript serializes"),
        );
        object.insert(
            "host_resource_receipt".to_string(),
            serde_json::to_value(&host_resources.receipt)
                .expect("host resource receipt serializes"),
        );
    }
}

pub fn run_summary_from_supported_source_profile_with_seed(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    rng_base_seed: u64,
) -> Result<JsonValue, String> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
            .to_json_value()
            .to_string()
    })?;
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        Some(profile),
        None,
        rng_base_seed,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_supported_source_profile_with_project_assets_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    project_asset_registry: Arc<ProjectAssetRegistry>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
    })?;
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        Some(project_asset_registry),
        None,
        None,
        Some(profile),
        None,
        0,
    )
}

pub fn run_summary_from_source_with_project_assets_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    project_asset_registry: Arc<ProjectAssetRegistry>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        Some(project_asset_registry),
        None,
        None,
        None,
        None,
        0,
    )
}

pub fn run_summary_from_source_with_input_tape(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    input_tape: &InputTape,
) -> Result<JsonValue, String> {
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        None,
        Some(input_tape),
        0,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_source_with_seed_and_input_tape(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    rng_base_seed: u64,
    input_tape: &InputTape,
) -> Result<JsonValue, String> {
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        None,
        Some(input_tape),
        rng_base_seed,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_supported_source_profile_with_input_tape(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    input_tape: &InputTape,
) -> Result<JsonValue, String> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
            .to_json_value()
            .to_string()
    })?;
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        Some(profile),
        Some(input_tape),
        0,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_supported_source_profile_with_seed_and_input_tape(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    rng_base_seed: u64,
    input_tape: &InputTape,
) -> Result<JsonValue, String> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
            .to_json_value()
            .to_string()
    })?;
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        Some(profile),
        Some(input_tape),
        rng_base_seed,
    )
    .map_err(|err| err.to_json_value().to_string())
}

fn run_summary_diagnostic(
    code: &'static str,
    message: impl Into<String>,
    file_path: &str,
    phase: &'static str,
) -> ExecutionDiagnostic {
    ExecutionDiagnostic {
        code,
        message: message.into(),
        source_path: file_path.to_string(),
        span: None,
        phase,
        cause: None,
        offending_field: None,
        expected: None,
        actual: None,
    }
}

fn work_cost_failure_diagnostic(
    failure: WorkCostFailure,
    file_path: &str,
    phase: &'static str,
) -> ExecutionDiagnostic {
    let receipt = serde_json::to_string(&failure.receipt)
        .unwrap_or_else(|_| "{\"receipt_error\":true}".to_string());
    ExecutionDiagnostic {
        code: failure.code,
        message: format!("{}; work_cost_receipt={receipt}", failure),
        source_path: file_path.to_string(),
        span: None,
        phase,
        cause: None,
        offending_field: None,
        expected: None,
        actual: None,
    }
}

fn run_summary_from_source_with_assets_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
    standard_limits: Option<StandardResourceLimitsV1>,
    age_target: Option<AgeTarget>,
    source_profile: Option<SupportedSourceProfile>,
    input_tape: Option<&InputTape>,
    rng_base_seed: u64,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_from_source_with_assets_diagnostic_with_update_name(
        source,
        file_path,
        requested_ticks,
        project_asset_registry,
        standard_limits,
        age_target,
        source_profile,
        input_tape,
        rng_base_seed,
        DEFAULT_UPDATE_NAME,
        None,
        false,
        None,
        None,
        None,
    )
}

/// The old source frontdoor remains a temporary-identity execution route.
fn run_summary_from_source_with_assets_diagnostic_with_update_name(
    source: &str, file_path: &str, requested_ticks: Option<u32>,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
    standard_limits: Option<StandardResourceLimitsV1>, age_target: Option<AgeTarget>,
    source_profile: Option<SupportedSourceProfile>, input_tape: Option<&InputTape>,
    rng_base_seed: u64, update_name: &str,
    live_next_mask: Option<(u32, &mut dyn FnMut(u64) -> Result<InputRecord, String>)>,
    named_entry_after_definitions: bool, input_snapshots: Option<&[InputSnapshot]>,
    param_overrides: Option<&BTreeMap<String, Value>>, bogae_entry: Option<&str>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_from_source_with_authored_bogae(
        source, file_path, requested_ticks, project_asset_registry, standard_limits,
        age_target, source_profile, input_tape, rng_base_seed, update_name,
        live_next_mask, named_entry_after_definitions, input_snapshots,
        param_overrides, bogae_entry, None)
}

fn run_summary_from_source_with_authored_bogae(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
    standard_limits: Option<StandardResourceLimitsV1>,
    age_target: Option<AgeTarget>,
    source_profile: Option<SupportedSourceProfile>,
    input_tape: Option<&InputTape>,
    rng_base_seed: u64,
    update_name: &str,
    mut live_next_mask: Option<(u32, &mut dyn FnMut(u64) -> Result<InputRecord, String>)>,
    named_entry_after_definitions: bool,
    input_snapshots: Option<&[InputSnapshot]>,
    param_overrides: Option<&BTreeMap<String, Value>>,
    bogae_entry: Option<&str>,
    authored_artifact: Option<&str>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    #[cfg(test)]
    RUN_SUMMARY_DIAGNOSTIC_INVOCATIONS.with(|count| count.set(count.get() + 1));

    if ddonirang_lang::has_model_surface(source) {
        if authored_artifact.is_some() {
            return Err(run_summary_diagnostic("E_AUTHORED_BOGAE_MODEL_UNSUPPORTED",
                "모델 원문의 영구 보개 정체 연결은 아직 지원하지 않습니다", file_path, "authored_bogae"));
        }
        if live_next_mask.is_some() || input_snapshots.is_some() {
            return Err(run_summary_diagnostic("E_SHARED_RUNTIME_OPTION_UNSUPPORTED", "모델 실행 경로는 실시간 제어입력을 지원하지 않습니다", file_path, "live_input"));
        }
        if rng_base_seed != 0 {
            return Err(run_summary_diagnostic(
                "E_MODEL_SEED_UNSUPPORTED",
                "model runtime에는 explicit seed 계약이 없습니다",
                file_path,
                "model_execution",
            ));
        }
        validate_model_oneshot_ticks(requested_ticks).map_err(|message| {
            run_summary_diagnostic("E_MODEL_TICK_POLICY", message, file_path, "model_execution")
        })?;
        return model_run_summary_from_source(source, file_path, project_asset_registry);
    }
    let source_profile_identity = source_profile
        .map(|_| V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY)
        .unwrap_or("strict");
    let effective_age_target = age_target.unwrap_or_else(crate::ddn_runtime::default_age_target);
    let (prepared, program) = match source_profile {
        Some(profile) => {
            let program = DdnProgram::from_supported_source_profile_with_mode_and_age_target(
                source,
                file_path,
                ParseMode::Strict,
                profile,
                effective_age_target,
            )
            .map_err(|message| {
                if let Some(diagnostic) = supported_source_parse_diagnostic(source, file_path, &message) {
                    return diagnostic;
                }
                let code = if message.contains(E_V1_CORE_RECURSION_UNSUPPORTED) {
                    E_V1_CORE_RECURSION_UNSUPPORTED
                } else if message.contains("E_V25_DEFINITION_MIGRATION_REQUIRED") {
                    "E_V25_DEFINITION_MIGRATION_REQUIRED"
                } else if message.contains("E_AGE_NOT_AVAILABLE") {
                    "E_AGE_NOT_AVAILABLE"
                } else {
                    "E_PARSE"
                };
                run_summary_diagnostic(code, message, file_path, "execution_parse")
            })?;
            let prepared =
                prepare_supported_source_for_execution_with_program(source, file_path, &program)?;
            (prepared, program)
        }
        None => {
            let prepared = prepare_source_for_execution_with_path(source, file_path)?;
            let program = program_from_legacy_source_with_age_target(
                source,
                file_path,
                ParseMode::Strict,
                effective_age_target,
            )
            .map_err(|message| {
                let code = if message.contains("E_AGE_NOT_AVAILABLE") {
                    "E_AGE_NOT_AVAILABLE"
                } else {
                    "E_PARSE"
                };
                run_summary_diagnostic(code, message, file_path, "execution_parse")
            })?;
            (prepared, program)
        }
    };
    crate::source_analysis::validate_program_for_execution(program.canonical_program())
        .map_err(|message| {
            run_summary_diagnostic("E_STATIC_VALIDATION", message, file_path, "static_validation")
        })?;
    let configured_madi = program.configured_madi().unwrap_or(0);
    let execution_mode_decision = prepared.execution_mode_decision;
    let parse_warnings = program.parse_warnings().to_vec();
    let (input_masks, sealed_input_tape_sha256) = match input_tape {
        Some(tape) => {
            if tape.madi_hz == 0 {
                return Err(run_summary_diagnostic(
                    "E_SAM_BAD_MADI_HZ",
                    "madi-hz는 0이 될 수 없습니다",
                    file_path,
                    "input_tape",
                ));
            }
            if tape.records.is_empty() {
                return Err(run_summary_diagnostic(
                    "E_SAM_RECORD_COUNT_MISMATCH",
                    "record_count=0",
                    file_path,
                    "input_tape",
                ));
            }
            if let Some(ticks) = requested_ticks {
                if usize::try_from(ticks).ok() != Some(tape.records.len()) {
                    return Err(run_summary_diagnostic(
                        "E_SAM_RECORD_COUNT_MISMATCH",
                        format!("record_count={} ticks={ticks}", tape.records.len()),
                        file_path,
                        "input_tape",
                    ));
                }
            }
            let mut masks = Vec::with_capacity(tape.records.len());
            for (index, record) in tape.records.iter().enumerate() {
                if record.madi != index as u32 {
                    return Err(run_summary_diagnostic(
                        "E_SAM_RECORD_ORDER_MISMATCH",
                        format!("record.madi={} idx={index}", record.madi),
                        file_path,
                        "input_tape",
                    ));
                }
                let mask = mask_from_bytes(&record.held_mask).map_err(|message| {
                    run_summary_diagnostic("E_SAM_TAPE", message, file_path, "input_tape")
                })?;
                masks.push(tape_mask_to_runtime_keys(mask));
            }
            let bytes = encode_input_tape(tape).map_err(|message| {
                run_summary_diagnostic("E_SAM_TAPE", message, file_path, "input_tape")
            })?;
            (masks, Some(sha256_hex(&bytes)))
        }
        None => (Vec::new(), None),
    };
    let effective_ticks = if input_tape.is_some() {
        input_masks.len() as u64
    } else {
        requested_ticks
            .map(u64::from)
            .unwrap_or_else(|| configured_madi.max(1))
            .max(1)
    };

    let sealed_dataset_assets = match project_asset_registry.as_deref() {
        Some(registry) => {
            serde_json::to_value(registry.sealed_dataset_identities().map_err(|message| {
                run_summary_diagnostic("E_DATASET_IDENTITY", message, file_path, "dataset_identity")
            })?)
            .map_err(|err| {
                run_summary_diagnostic(
                    "E_DATASET_IDENTITY_SERIALIZE",
                    err.to_string(),
                    file_path,
                    "dataset_identity",
                )
            })?
        }
        None => JsonValue::Array(Vec::new()),
    };
    let sealed_dataset_identity_sha256 = sha256_hex(
        serde_json::to_string(&sealed_dataset_assets)
            .map_err(|err| {
                run_summary_diagnostic(
                    "E_DATASET_IDENTITY_SERIALIZE",
                    err.to_string(),
                    file_path,
                    "dataset_identity",
                )
            })?
            .as_bytes(),
    );
    let mut runner = DdnRunner::new(program, update_name).with_bogae_activation();
    if let Some(artifact) = authored_artifact {
        runner.configure_authored_bogae(artifact).map_err(|message|
            run_summary_diagnostic("E_AUTHORED_BOGAE_ADMISSION", message, file_path, "authored_bogae"))?;
    }
    if let Some(entry) = bogae_entry {
        runner.configure_bogae_entry(entry).map_err(|message|
            run_summary_diagnostic("E_BOGAE_ENTRY_UNRESOLVED", message, file_path, "bogae_entry"))?;
    }
    if named_entry_after_definitions {
        runner = runner.with_named_entry_after_definitions();
    }
    if let Some(limits) = standard_limits {
        runner = runner.with_standard_resource_limits(limits);
    }
    let mut sealed_input_bytes = input_tape
        .and_then(|tape| encode_input_tape(tape).ok())
        .map(|bytes| source.len().saturating_add(bytes.len()))
        .unwrap_or(source.len());
    if let Some(inputs) = input_snapshots {
        let bytes = json!(inputs.iter().map(crate::input_receipt::snapshot_value)
            .collect::<Vec<_>>()).to_string().len();
        sealed_input_bytes = sealed_input_bytes.checked_add(bytes).ok_or_else(||
            run_summary_diagnostic("E_INPUT_RECEIPT_SIZE", "입력 기록 크기가 표현 범위를 넘었습니다",
                file_path, "resource_preflight"))?;
    }
    runner
        .set_sealed_input_bytes(sealed_input_bytes)
        .map_err(|failure| {
            work_cost_failure_diagnostic(failure, file_path, "resource_preflight")
        })?;
    if let Some(registry) = project_asset_registry {
        runner = runner.with_project_asset_registry(registry);
    }
    let mut world = DetNuri::new();
    let mut defaults = HashMap::new();
    seed_bogae_defaults(&mut defaults);
    let mut tick_id = 0u64;
    let input_dt = crate::input_receipt::alpha_host_dt();
    let mut last_patch = Patch::default();
    let mut last_input = serialize_input_state(0, "", 0, 0, input_dt, rng_base_seed, rng_base_seed);
    let source_identity = format!("sha256:{}", sha256_hex(source.as_bytes()));
    let revision_identity = format!(
        "sha256:{}",
        sha256_hex(
            format!(
                "{}\0{}",
                prepared.canonical_ddn, sealed_dataset_identity_sha256
            )
            .as_bytes()
        )
    );
    if let Some(inputs) = input_snapshots {
        if input_tape.is_some() || live_next_mask.is_some() || inputs.len() != effective_ticks as usize
            || inputs.iter().enumerate().any(|(index, input)| input.tick_id != index as u64) {
            return Err(run_summary_diagnostic("E_INPUT_RECEIPT_SEQUENCE_INVALID",
                "입력 기록의 마디 수·순서 또는 입력 경로가 다릅니다", file_path, "input_receipt"));
        }
    }
    let input_record_sha256 = input_snapshots.map(crate::input_receipt::record_sha256);
    let mut input_identity_suffix = sealed_input_tape_sha256
        .as_deref()
        .map(|identity| format!("\0input_tape_sha256={identity}"))
        .unwrap_or_default();
    if let Some(identity) = &input_record_sha256 {
        input_identity_suffix.push_str(&format!("\0input_record_sha256={identity}"));
    }
    let seed_identity_suffix = if rng_base_seed == 0 {
        String::new()
    } else {
        format!("\0rng_base_seed={rng_base_seed}")
    };
    let run_identity_material = |input_identity_suffix: &str| match age_target {
        Some(age_target) => format!(
            "{}\0{}\0{}\0{}\0age_target={}{}{}",
            file_path,
            source_identity,
            revision_identity,
            effective_ticks,
            age_target.label(),
            input_identity_suffix,
            seed_identity_suffix
        ),
        None => format!(
            "{}\0{}\0{}\0{}{}{}",
            file_path,
            source_identity,
            revision_identity,
            effective_ticks,
            input_identity_suffix,
            seed_identity_suffix
        ),
    };
    let mut run_id = format!("run:{}", sha256_hex(run_identity_material(&input_identity_suffix).as_bytes()));
    let mut live_records = Vec::new();
    let mut input_receipts = Vec::new();
    let mut last_bogae_projection = None;
    let mut display_outputs = Vec::new();
    let mut observation_frames = preflight_observation_frames(&runner, effective_ticks as usize)
        .map_err(|failure| {
            work_cost_failure_diagnostic(failure, file_path, "resource_preflight")
        })?;

    for tick_index in 0..effective_ticks {
        let tick_seed = rng_base_seed ^ tick_id;
        let keys_pressed = match live_next_mask.as_mut() {
            Some((_, next_mask)) => {
                let record = next_mask(tick_id).map_err(|message| {
                    let code = match message.split_whitespace().next() {
                        Some("E_SAM_LIVE_QUEUE_OVERFLOW") => "E_SAM_LIVE_QUEUE_OVERFLOW",
                        Some("E_SAM_LIVE_SEQUENCE_OVERFLOW") => "E_SAM_LIVE_SEQUENCE_OVERFLOW",
                        Some("E_SAM_LIVE_INPUT_POLL") => "E_SAM_LIVE_INPUT_POLL",
                        Some("E_SAM_LIVE_INPUT_READ") => "E_SAM_LIVE_INPUT_READ",
                        Some("E_SAM_LIVE_RAW_RESTORE") => "E_SAM_LIVE_RAW_RESTORE",
                        _ => "E_SAM_LIVE_INPUT",
                    };
                    run_summary_diagnostic(code, message, file_path, "live_input")
                })?;
                if u64::from(record.madi) != tick_id {
                    return Err(run_summary_diagnostic("E_SAM_RECORD_ORDER_MISMATCH", "실시간 입력의 마디 순서가 다릅니다", file_path, "live_input"));
                }
                let mask = mask_from_bytes(&record.held_mask).map_err(|message| run_summary_diagnostic("E_SAM_TAPE", message, file_path, "live_input"))?;
                live_records.push(record);
                tape_mask_to_runtime_keys(mask)
            },
            None => input_masks.get(tick_index as usize).copied().unwrap_or(0),
        };
        let input = input_snapshots.map(|inputs| inputs[tick_index as usize].clone()).unwrap_or_else(|| InputSnapshot {
            tick_id,
            dt: input_dt,
            keys_pressed,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            frame_source: InputSource::Person,
            rng_seed: tick_seed,
        });
        let control_edges = if live_next_mask.is_some() {
            live_records.last().map(InputRecord::runtime_edges)
        } else {
            input_tape.filter(|tape| tape.version == 2)
                .and_then(|tape| tape.records.get(tick_index as usize))
                .map(InputRecord::runtime_edges)
        };
        let output = runner
            .run_update_diagnostic_with_control_edges(world.world(), &input, &defaults, control_edges, param_overrides)
            .map_err(|error| execution_error_diagnostic(error, file_path))?;
        if input_tape.is_some_and(|tape| tape.version == 1) {
            crate::input_receipt::validate_tape_v1_reads(&output.input_reads)
                .map_err(|message| run_summary_diagnostic(
                    "E_INPUT_TAPE_V1_UNRECORDED_READ", message, file_path, "input_tape"))?;
        }
        let public_observations = public_observation_events_json(&output.public_observations)
            .map_err(|message| {
                run_summary_diagnostic(
                    "E_OBSERVATION_SERIALIZE",
                    message,
                    file_path,
                    "observation_projection",
                )
            })?;
        let mut sink = ddonirang_core::signals::VecSignalSink::default();
        world.apply_patch(&output.patch, input.tick_id, &mut sink);
        display_outputs.extend(output.display_outputs);
        if !display_outputs.is_empty() {
            crate::output_display::retain_current(&mut display_outputs, &serialize_world_resources(world.world()));
        }
        let input_json = serialize_input_snapshot(&input, Some(rng_base_seed));
        input_receipts.push(crate::input_receipt::receipt(&input, &output.input_reads));
        observation_frames.push(json!({
            "run_id": run_id,
            "source_identity": source_identity,
            "revision_identity": revision_identity,
            "committed_madi": input.tick_id,
            "state_hash": current_state_hash(world.world()),
            "input_snapshot": input_json,
            "public_observations": public_observations,
        }));
        last_patch = output.patch;
        last_bogae_projection = output.bogae_projection;
        last_input = serialize_input_snapshot(&input, Some(rng_base_seed));
        tick_id = tick_id.saturating_add(1);
    }

    if let Some((madi_hz, _)) = live_next_mask {
        let bytes = encode_input_tape(&InputTape { version: 2, madi_hz, records: live_records })
            .map_err(|message| run_summary_diagnostic("E_SAM_TAPE", message, file_path, "live_input"))?;
        let sealed_bytes = source.len().checked_add(bytes.len()).ok_or_else(||
            run_summary_diagnostic("E_SAM_TAPE", "입력 크기가 표현 범위를 넘었습니다", file_path, "live_input"))?;
        runner.set_sealed_input_bytes(sealed_bytes).map_err(|failure|
            work_cost_failure_diagnostic(failure, file_path, "resource_preflight"))?;
        // All frames are private candidates until input sealing and host restoration succeed.
        run_id = format!("run:{}", sha256_hex(run_identity_material(&format!("\0input_tape_sha256={}", sha256_hex(&bytes))).as_bytes()));
        for frame in &mut observation_frames {
            frame["run_id"] = json!(run_id);
        }
    }
    if let Some(projection)=last_bogae_projection.as_mut() {
        crate::bogae_time::resolve_result_tables(projection,&observation_frames,&source_identity,&revision_identity)
            .map_err(|message|run_summary_diagnostic("E_BOGAE_TIME_PROJECTION",message,file_path,"observation_projection"))?;
    }
    let mut state = build_engine_state_json(&world, tick_id, last_input, &last_patch);
    crate::bogae_projection::attach(&mut state, last_bogae_projection.as_ref());
    crate::output_display::attach(&mut state, &display_outputs);
    let trace_bundle = build_public_observation_trace(&json!({
        "project_id": "",
        "run_id": run_id,
        "source_identity": source_identity,
        "revision_identity": revision_identity,
        "run_status": "completed",
        "frames": observation_frames,
    }))
    .map_err(|message| {
        run_summary_diagnostic(
            "E_OBSERVATION_TRACE",
            message,
            file_path,
            "observation_projection",
        )
    })?;
    let observation_trace = trace_bundle
        .get("trace")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let observation_trace_hash = trace_bundle
        .get("trace_hash")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let observation_table_projection =
        build_public_observation_table_projection(&observation_trace, &run_id).map_err(
            |message| {
                run_summary_diagnostic(
                    "E_OBSERVATION_TABLE_PROJECTION",
                    message,
                    file_path,
                    "observation_projection",
                )
            },
        )?;
    let observation_columns = observation_trace
        .get("columns")
        .cloned()
        .unwrap_or_else(|| JsonValue::Array(Vec::new()));
    let output_rows = observation_trace
        .get("samples")
        .cloned()
        .unwrap_or_else(|| JsonValue::Array(Vec::new()));
    if let Some(state_map) = state.as_object_mut() {
        state_map.insert(
            "public_observation_trace".to_string(),
            observation_trace.clone(),
        );
        state_map.insert(
            "public_observation_trace_hash".to_string(),
            observation_trace_hash.clone(),
        );
        state_map.insert(
            "public_observation_table_projection".to_string(),
            observation_table_projection.clone(),
        );
        state_map.insert(
            "observation_columns".to_string(),
            observation_columns.clone(),
        );
        state_map.insert("output_rows".to_string(), output_rows.clone());
        if let Some(nested_state) = state_map
            .get_mut("state")
            .and_then(JsonValue::as_object_mut)
        {
            nested_state.insert(
                "snapshot_v2".to_string(),
                runtime_state_snapshot(&world, observation_frames.clone(), tick_id),
            );
        }
    }
    let state_hash = state
        .get("state_hash")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
    let view_hash = state
        .get("view_hash")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
    let primary_view_decision = crate::primary_view::decide(
        state.get("view_meta").unwrap_or(&JsonValue::Null),
        &sha256_hex(source.as_bytes()),
        &sha256_hex(prepared.canonical_ddn.as_bytes()),
        &state_hash,
        &view_hash,
    );
    if let Some(state_map) = state.as_object_mut() {
        state_map.insert(
            "primary_view_decision".to_string(),
            primary_view_decision.clone(),
        );
    }
    let final_row = index_row_from_state(&state);
    let output_log_texts = output_log_texts_from_state_json(&state);
    let parse_warnings_json = parse_warnings
        .iter()
        .map(|warning| {
            json!({
                "code": warning.code,
                "message": warning.message,
                "span": {
                    "start": warning.span_start,
                    "end": warning.span_end,
                }
            })
        })
        .collect::<Vec<_>>();

    let mut execution_envelope = build_execution_envelope(
        source,
        &prepared,
        configured_madi,
        execution_mode_decision,
        requested_ticks,
        effective_ticks,
        input_dt,
        rng_base_seed,
    );
    if let Some(envelope) = execution_envelope.as_object_mut() {
        envelope.insert("input_record_sha256".into(), json!(input_record_sha256));
        if let Some(inputs) = input_snapshots {
            envelope.insert("input_snapshots".into(), json!(inputs.iter()
                .map(crate::input_receipt::snapshot_value).collect::<Vec<_>>()));
            envelope.insert("input_snapshot".into(), if inputs.len() == 1 {
                serialize_input_snapshot(&inputs[0], None)
            } else { JsonValue::Null });
            envelope.insert("dt".into(), if inputs.first().is_some_and(|first|
                inputs.iter().all(|input| input.dt == first.dt)) {
                json!(crate::input_receipt::exact_dt_text(inputs[0].dt))
            } else { JsonValue::Null });
        }
        envelope.insert(
            "sealed_dataset_identity_sha256".to_string(),
            JsonValue::String(sealed_dataset_identity_sha256.clone()),
        );
        envelope.insert(
            "sealed_dataset_assets".to_string(),
            sealed_dataset_assets.clone(),
        );
        if let Some(age_target) = age_target {
            envelope.insert(
                "age_target_identity".to_string(),
                JsonValue::String(age_target.label().to_string()),
            );
        }
    }
    runner
        .set_replay_capsule_bytes(0)
        .map_err(|failure| work_cost_failure_diagnostic(failure, file_path, "resource_commit"))?;
    let mut summary = json!({
        "schema": "ddn.ddonirang_tool.shared_run_summary.v1",
        "runtime_schema": ENGINE_RESPONSE_SCHEMA,
        "update_name": update_name,
        "source_path": file_path,
        "source_profile_identity": source_profile_identity,
        "canonical_ddn": prepared.canonical_ddn,
        "preprocessed_source": prepared.execution_source,
        "configured_ticks": configured_madi,
        "effective_ticks": effective_ticks,
        "ticks_run": effective_ticks,
        "parse_warnings": parse_warnings_json,
        "state_hash": state.get("state_hash").cloned().unwrap_or(JsonValue::String(String::new())),
        "view_hash": state.get("view_hash").cloned().unwrap_or(JsonValue::String(String::new())),
        "primary_view_decision": primary_view_decision,
        "final_row": final_row,
        "resources": JsonValue::Null,
        "channels": JsonValue::Null,
        "row": JsonValue::Null,
        "observation_columns": JsonValue::Null,
        "public_observation_trace": JsonValue::Null,
        "public_observation_trace_hash": JsonValue::Null,
        "public_observation_table_projection": JsonValue::Null,
        "output_log_texts": JsonValue::Null,
        "output_rows": JsonValue::Null,
        "engine_state": JsonValue::Null,
        "execution_envelope": execution_envelope,
        "input_receipts": input_receipts,
        "input_record_sha256": input_record_sha256,
        "sealed_dataset_identity_sha256": sealed_dataset_identity_sha256,
        "sealed_dataset_assets": sealed_dataset_assets,
        "parity_surface": {
            "domain": "ddonirang_tool_shared_runtime",
            "state_hash_formula_change": false,
            "view_prefix_exclusion": DEFAULT_VIEW_PREFIXES
        },
        "work_cost_receipt": runner.work_cost_receipt(),
        "meta_lines": prepared.meta_lines
    });
    // engine_state는 이미 완성된 공개 JSON이다. 동일 슬롯으로 옮겨 큰 상태의
    // json! to_value 재변환을 피한다. 공개 사본·필드·bytes는 바꾸지 않는다.
    for key in ["resources", "channels", "row"] {
        summary[key] = state.get(key).cloned().unwrap_or_else(|| {
            if key == "resources" { JsonValue::Object(Map::new()) }
            else { JsonValue::Array(Vec::new()) }
        });
    }
    for (key, value) in [
        ("observation_columns", observation_columns),
        ("public_observation_trace", observation_trace),
        ("public_observation_trace_hash", observation_trace_hash),
        ("public_observation_table_projection", observation_table_projection),
        ("output_rows", output_rows),
    ] {
        summary[key] = value;
    }
    summary["output_log_texts"] = JsonValue::Array(
        output_log_texts.into_iter().map(JsonValue::String).collect());
    summary["engine_state"] = state;
    crate::bogae_projection::attach(&mut summary, last_bogae_projection.as_ref());
    crate::output_display::attach(&mut summary, &display_outputs);
    if let Some(age_target) = age_target {
        summary.as_object_mut().expect("summary object").insert(
            "age_target_identity".to_string(),
            JsonValue::String(age_target.label().to_string()),
        );
    }
    let serialize_error = |error: serde_json::Error| {
        run_summary_diagnostic(
            "E_RESULT_ARTIFACT_SERIALIZE",
            error.to_string(),
            file_path,
            "resource_commit",
        )
    };
    let initial_bytes = serialized_json_byte_count(&summary).map_err(&serialize_error)?;
    let initial_receipt_bytes = serialized_json_byte_count(&summary["work_cost_receipt"])
        .map_err(&serialize_error)?;
    // 조립 완료 뒤에는 영수증 값만 바뀐다. 키·구분자·다른 공개 필드의
    // UTF-8 바이트는 고정이므로 전체를 다시 순회하지 않고 분해한 크기를 쓴다.
    let outside_receipt_bytes = initial_bytes.checked_sub(initial_receipt_bytes)
        .ok_or_else(|| serialize_error(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::Other, "JSON byte count overflow"))))?;
    for _ in 0..16 {
        let receipt_bytes = serialized_json_byte_count(&summary["work_cost_receipt"])
            .map_err(&serialize_error)?;
        let bytes = outside_receipt_bytes.checked_add(receipt_bytes)
            .ok_or_else(|| serialize_error(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::Other, "JSON byte count overflow"))))?;
        if runner.work_cost_receipt().counters.result_artifact_bytes == bytes as u64 {
            return Ok(summary);
        }
        runner.set_result_artifact_bytes(bytes).map_err(|failure| {
            work_cost_failure_diagnostic(failure, file_path, "resource_commit")
        })?;
        summary["work_cost_receipt"] =
            serde_json::to_value(runner.work_cost_receipt()).map_err(|error| {
                run_summary_diagnostic(
                    "E_RESULT_ARTIFACT_SERIALIZE",
                    error.to_string(),
                    file_path,
                    "resource_commit",
                )
            })?;
    }
    Err(run_summary_diagnostic(
        "E_RESULT_ARTIFACT_SIZE_FIXED_POINT",
        "결과 artifact byte 크기와 WorkCostV1 영수증이 수렴하지 않았습니다",
        file_path,
        "resource_commit",
    ))
}

#[derive(Default)]
struct JsonByteCounter {
    bytes: usize,
}

impl std::io::Write for JsonByteCounter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let next = self.bytes.checked_add(buffer.len()).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "JSON byte count overflow")
        })?;
        self.bytes = next;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn serialized_json_byte_count(value: &JsonValue) -> Result<usize, serde_json::Error> {
    // 영수증은 같은 UTF-8 직렬화 바이트 수를 요구하지만 결과 버퍼는 필요 없다.
    let mut counter = JsonByteCounter::default();
    serde_json::to_writer(&mut counter, value)?;
    Ok(counter.bytes)
}

fn validate_model_oneshot_ticks(requested_ticks: Option<u32>) -> Result<(), String> {
    if requested_ticks.is_some_and(|ticks| ticks != 1) {
        return Err(
            "E_MODEL_ONESHOT_TICKS_CANDIDATE: #정밀 풀이방법은 정확히 1회 실행만 허용합니다"
                .to_string(),
        );
    }
    Ok(())
}

fn model_execution_error_text(error: crate::model_runtime::ModelExecutionError) -> String {
    format!("{} {}", error, error.to_json())
}

fn model_runtime_state_snapshot(
    public_observation_frames: Vec<JsonValue>,
    next_tick_id: u64,
) -> JsonValue {
    let resource_schema = crate::unit_wire::RESOURCE_SNAPSHOT_SCHEMA_V2;
    let unit_wire_schema = crate::unit_wire::UNIT_VALUE_WIRE_SCHEMA_V2;
    json!({
        "schema": "ddn.wasm_state_snapshot.v2",
        "resource_snapshot_schema": resource_schema,
        "unit_wire_schema": unit_wire_schema,
        "resources": {
            "schema": resource_schema,
            "unit_wire_schema": unit_wire_schema,
            "next_entity": 0,
            "components": [],
            "json": {},
            "fixed64": {},
            "handle": {},
            "value": {},
            "value_json": {},
            "value_det": {},
            "component": {},
        },
        "param_overrides": {},
        "public_observation_frames": public_observation_frames,
        "next_tick_id": next_tick_id,
    })
}

pub fn model_prepared_state_from_source(
    source: &str,
    file_path: &str,
) -> Result<JsonValue, String> {
    let program = ddonirang_lang::parse_model_program(source)
        .map_err(|err| format!("E_MODEL_FRONTDOOR: {err}"))?;
    let execution_envelope = crate::model_runtime::execution_envelope_json(source)
        .map_err(model_execution_error_text)?;
    let state_hash = crate::model_runtime::empty_state_hash();
    let source_identity = format!("sha256:{}", sha256_hex(source.as_bytes()));
    let revision_identity = format!("sha256:{}", sha256_hex(program.canonical_ddn.as_bytes()));
    let run_id = format!(
        "run:{}",
        sha256_hex(
            format!("{file_path}\0{source_identity}\0{revision_identity}\0model-prepared")
                .as_bytes()
        )
    );
    let trace_bundle = build_public_observation_trace(&json!({
        "project_id": "",
        "run_id": run_id,
        "source_identity": source_identity,
        "revision_identity": revision_identity,
        "run_status": "running",
        "frames": [],
    }))?;
    let observation_trace = trace_bundle
        .get("trace")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let observation_trace_hash = trace_bundle
        .get("trace_hash")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let observation_table_projection =
        build_public_observation_table_projection(&observation_trace, &run_id)?;
    let view_meta = json!({});
    let view_hash = hash_json_string(&view_meta);
    let primary_view_decision = crate::primary_view::decide(
        &view_meta,
        &sha256_hex(source.as_bytes()),
        &sha256_hex(program.canonical_ddn.as_bytes()),
        &state_hash,
        &view_hash,
    );
    let runtime_state_snapshot = model_runtime_state_snapshot(Vec::new(), 0);
    Ok(json!({
        "schema": ENGINE_RESPONSE_SCHEMA,
        "tick_id": 0,
        "state_hash": state_hash,
        "view_hash": view_hash,
        "view_meta": view_meta,
        "resources": {},
        "channels": [],
        "row": [],
        "state": {
            "tick_id": 0,
            "committed_madi": JsonValue::Null,
            "input": {},
            "resources": {},
            "channels": [],
            "row": [],
            "public_observations": [],
            "snapshot_v2": runtime_state_snapshot,
        },
        "public_observation_trace": observation_trace,
        "public_observation_trace_hash": observation_trace_hash,
        "public_observation_table_projection": observation_table_projection,
        "observation_columns": [],
        "output_rows": [],
        "primary_view_decision": primary_view_decision,
        "model_execution": JsonValue::Null,
        "model_execution_envelope": execution_envelope,
    }))
}

fn model_run_summary_from_source(
    source: &str,
    file_path: &str,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    if project_asset_registry.is_some() {
        return Err(run_summary_diagnostic(
            "E_MODEL_PROJECT_ASSET_RUNTIME_OPEN_CANDIDATE",
            "제한 model backend의 project asset 결속은 OPEN입니다",
            file_path,
            "model_project_link",
        ));
    }
    let execution = crate::model_runtime::execute_source_with_source_file(source, file_path)
        .map_err(|err| model_execution_diagnostic(err, source))?;
    let canonical_ddn = execution
        .get("canonical_ddn")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let state_hash = execution
        .get("post_state_hash")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let source_identity = format!("sha256:{}", sha256_hex(source.as_bytes()));
    let revision_identity = format!("sha256:{}", sha256_hex(canonical_ddn.as_bytes()));
    let run_id = format!(
        "run:{}",
        sha256_hex(
            format!("{file_path}\0{source_identity}\0{revision_identity}\0model-oneshot")
                .as_bytes()
        )
    );
    let mut public_observations = execution
        .get("public_observations")
        .cloned()
        .unwrap_or_else(|| JsonValue::Array(Vec::new()));
    if let Some(rows) = public_observations.as_array_mut() {
        for row in rows {
            let start = row.get("source_start").and_then(JsonValue::as_u64);
            let end = row.get("source_end").and_then(JsonValue::as_u64);
            if let (Some(start), Some(end), Some(object)) = (start, end, row.as_object_mut()) {
                object.insert(
                    "source_ref".to_string(),
                    json!(format!("{file_path}#bytes={start}:{end}")),
                );
            }
        }
    }
    let committed_frame = json!({
        "run_id": run_id,
        "source_identity": source_identity,
        "revision_identity": revision_identity,
        "committed_madi": 0,
        "state_hash": state_hash,
        "input_snapshot": JsonValue::Null,
        "public_observations": public_observations,
    });
    let trace_bundle = build_public_observation_trace(&json!({
        "project_id": "",
        "run_id": run_id,
        "source_identity": source_identity,
        "revision_identity": revision_identity,
        "run_status": "completed",
        "frames": [committed_frame.clone()],
    }))
    .map_err(|message| {
        run_summary_diagnostic(
            "E_MODEL_OBSERVATION_TRACE_CANDIDATE",
            message,
            file_path,
            "model_observation_trace",
        )
    })?;
    let observation_trace = trace_bundle
        .get("trace")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let observation_trace_hash = trace_bundle
        .get("trace_hash")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let observation_table_projection =
        build_public_observation_table_projection(&observation_trace, &run_id).map_err(
            |message| {
                run_summary_diagnostic(
                    "E_MODEL_OBSERVATION_TABLE_CANDIDATE",
                    message,
                    file_path,
                    "model_observation_table",
                )
            },
        )?;
    let observation_columns = observation_trace
        .get("columns")
        .cloned()
        .unwrap_or_else(|| JsonValue::Array(Vec::new()));
    let output_rows = observation_trace
        .get("samples")
        .cloned()
        .unwrap_or_else(|| JsonValue::Array(Vec::new()));

    let mut final_row = Map::new();
    if let Some(fields) = execution
        .pointer("/result_pack/ordered_fields")
        .and_then(JsonValue::as_array)
    {
        for field in fields {
            if let Some(name) = field.get("name").and_then(JsonValue::as_str) {
                final_row.insert(
                    name.to_string(),
                    field.get("value_canon").cloned().unwrap_or(JsonValue::Null),
                );
            }
        }
    }
    let view_meta = json!({});
    let view_hash = hash_json_string(&view_meta);
    let primary_view_decision = crate::primary_view::decide(
        &view_meta,
        &sha256_hex(source.as_bytes()),
        &sha256_hex(canonical_ddn.as_bytes()),
        state_hash,
        &view_hash,
    );
    let execution_envelope = crate::model_runtime::execution_envelope_json(source)
        .map_err(|err| model_execution_diagnostic(err, source))?;
    let work_cost_receipt = execution
        .pointer("/receipt/work_cost")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let runtime_state_snapshot = model_runtime_state_snapshot(vec![committed_frame], 1);
    let engine_state = json!({
        "schema": ENGINE_RESPONSE_SCHEMA,
        "tick_id": 1,
        "state_hash": state_hash,
        "view_hash": view_hash,
        "view_meta": view_meta,
        "resources": {},
        "channels": [],
        "row": [],
        "state": {
            "tick_id": 1,
            "committed_madi": 0,
            "input": {},
            "resources": {},
            "channels": [],
            "row": [],
            "public_observations": public_observations,
            "snapshot_v2": runtime_state_snapshot,
        },
        "public_observation_trace": observation_trace,
        "public_observation_trace_hash": observation_trace_hash,
        "public_observation_table_projection": observation_table_projection,
        "observation_columns": observation_columns,
        "output_rows": output_rows,
        "primary_view_decision": primary_view_decision,
        "model_execution": execution,
        "model_execution_envelope": execution_envelope,
    });
    Ok(json!({
        "schema": "ddn.ddonirang_tool.shared_run_summary.v1",
        "runtime_schema": crate::model_runtime::MODEL_EXECUTION_SCHEMA,
        "update_name": "model-oneshot",
        "source_path": file_path,
        "canonical_ddn": canonical_ddn,
        "preprocessed_source": source,
        "configured_ticks": 1,
        "effective_ticks": 1,
        "ticks_run": 1,
        "parse_warnings": [],
        "state_hash": state_hash,
        "view_hash": view_hash,
        "primary_view_decision": primary_view_decision,
        "final_row": JsonValue::Object(final_row),
        "resources": {},
        "channels": [],
        "row": [],
        "observation_columns": observation_columns,
        "public_observation_trace": observation_trace,
        "public_observation_trace_hash": observation_trace_hash,
        "public_observation_table_projection": observation_table_projection,
        "output_log_texts": [],
        "output_rows": output_rows,
        "engine_state": engine_state,
        "execution_envelope": execution_envelope,
        "sealed_dataset_identity_sha256": sha256_hex(b"[]"),
        "sealed_dataset_assets": [],
        "parity_surface": {
            "domain": "ddonirang_tool_shared_model_runtime_candidate",
            "state_hash_formula_change": false,
            "view_prefix_exclusion": []
        },
        "meta_lines": 0,
        "model_execution": execution,
        "work_cost_receipt": work_cost_receipt,
    }))
}

fn model_execution_diagnostic(
    err: crate::model_runtime::ModelExecutionError,
    source: &str,
) -> ExecutionDiagnostic {
    let failure_json = err.to_json();
    let offending_source = source
        .get(err.provenance.start..err.provenance.end)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let source_field =
        if err.candidate_code == "E_MODEL_EXECUTION_CONTRACT_FIELD_DELIMITER_CANDIDATE" {
            source
                .get(..err.provenance.end)
                .and_then(|prefix| prefix.rsplit([',', '(']).next())
        } else {
            offending_source
        };
    let offending_field = err.offending_field.or_else(|| {
        source_field.and_then(|value| {
            value
                .split([':', '='])
                .next()
                .map(str::trim)
                .filter(|field| !field.is_empty())
                .map(str::to_string)
        })
    });
    let actual = err.actual.or_else(|| offending_source.map(str::to_string));
    let cause = Some(Box::new(ExecutionDiagnostic {
        code: err.candidate_code,
        message: failure_json.to_string(),
        source_path: err.source_path.clone(),
        span: Some((err.provenance.start, err.provenance.end)),
        phase: err.phase,
        cause: None,
        offending_field: None,
        expected: None,
        actual: None,
    }));
    ExecutionDiagnostic {
        code: err.candidate_code,
        message: err.message,
        source_path: err.source_path,
        span: Some((err.provenance.start, err.provenance.end)),
        phase: err.phase,
        cause,
        offending_field,
        expected: err.expected,
        actual,
    }
}

pub fn build_engine_state_json(
    world: &DetNuri,
    tick_id: u64,
    input_json: JsonValue,
    patch: &Patch,
) -> JsonValue {
    let committed_madi = tick_id.saturating_sub(1);
    let resources = serialize_world_resources(world.world());
    let (columns, row) = collect_columns_and_row(world.world());
    let view_meta = build_view_meta(world.world());
    let view_hash = hash_json_string(&view_meta);
    let patch_json = serialize_patch(patch);
    let mut result = json!({
        "schema": ENGINE_RESPONSE_SCHEMA,
        "tick_id": tick_id,
        "committed_madi": committed_madi,
        "state_hash": current_state_hash(world.world()),
        "input": JsonValue::Null,
        "resources": JsonValue::Null,
        "channels": JsonValue::Null,
        "row": JsonValue::Null,
        "streams": JsonValue::Object(Map::new()),
        "patch": JsonValue::Null,
        "state": {
            "tick_id": tick_id,
            "committed_madi": committed_madi,
            "input": JsonValue::Null,
            "resources": JsonValue::Null,
            "channels": JsonValue::Null,
            "row": JsonValue::Null,
            "patch": JsonValue::Null,
            "streams": JsonValue::Object(Map::new())
        },
        "view_meta": JsonValue::Null,
        "view_hash": view_hash
    });
    // 이미 정본 JSON인 값은 다시 직렬화하지 않는다. 공개 두 경로를 그대로
    // 유지하고 각 값의 필수 사본 하나만 만든 뒤 원 값을 중첩 상태로 옮긴다.
    for (key, value) in [("input", input_json), ("resources", resources),
        ("channels", JsonValue::Array(columns)), ("row", JsonValue::Array(row))] {
        result[key] = value.clone();
        result["state"][key] = value;
    }
    result["view_meta"] = view_meta;
    result["patch"] = patch_json.clone();
    result["state"]["patch"] = patch_json;
    result
}

fn index_row_from_state(state: &JsonValue) -> JsonValue {
    let mut out = Map::new();
    let channels = state
        .get("channels")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let row = state
        .get("row")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    for (idx, channel) in channels.iter().enumerate() {
        let key = channel
            .get("key")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .trim();
        if key.is_empty() {
            continue;
        }
        out.insert(
            key.to_string(),
            row.get(idx).cloned().unwrap_or(JsonValue::Null),
        );
    }
    JsonValue::Object(out)
}

fn output_log_texts_from_state_json(state: &JsonValue) -> Vec<String> {
    let Some(value) = state.pointer("/resources/value_json/output_log") else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
    items.iter().filter_map(output_log_entry_text).collect()
}

fn output_log_entry_text(value: &JsonValue) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }
    if let Some(obj) = value.as_object() {
        if let Some(text) = obj.get("text").and_then(JsonValue::as_str) {
            return Some(text.to_string());
        }
        if let Some(value) = obj.get("value") {
            return Some(json_scalar_to_display_text(value));
        }
    }
    Some(json_scalar_to_display_text(value))
}

fn json_scalar_to_display_text(value: &JsonValue) -> String {
    match value {
        JsonValue::String(text) => text.clone(),
        JsonValue::Number(number) => number.to_string(),
        JsonValue::Bool(true) => "참".to_string(),
        JsonValue::Bool(false) => "거짓".to_string(),
        JsonValue::Null => "없음".to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

pub fn seed_bogae_defaults(defaults: &mut HashMap<String, Value>) {
    defaults.insert(BOGAE_DRAWLIST_TAG.to_string(), Value::List(Vec::new()));
    defaults.insert(BOGAE_SHOW_LINES_TAG.to_string(), Value::List(Vec::new()));
    defaults.insert(
        BOGAE_WIDTH_TAG.to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert(
        BOGAE_HEIGHT_TAG.to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert("프레임수".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
    defaults.insert(
        "__wasm_start_once".to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert("회전_x".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
    defaults.insert("회전_y".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
}

pub fn serialize_world_resources(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    let mut value_json_map = Map::new();
    out.insert("schema".to_string(), json!(RESOURCE_SNAPSHOT_SCHEMA_V2));
    out.insert(
        "unit_wire_schema".to_string(),
        json!(UNIT_VALUE_WIRE_SCHEMA_V2),
    );

    let mut json_map = Map::new();
    for (tag, json_text) in world.resource_json_entries() {
        value_json_map.insert(tag.clone(), resource_json_text_to_json(&json_text));
        json_map.insert(tag, JsonValue::String(json_text));
    }
    out.insert("json".to_string(), JsonValue::Object(json_map));

    let mut fixed_map = Map::new();
    for (tag, value) in world.resource_fixed64_entries() {
        value_json_map.insert(tag.clone(), json!(fixed64_to_f64(value)));
        fixed_map.insert(tag, JsonValue::String(value.display_rounded_six()));
    }
    out.insert("fixed64".to_string(), JsonValue::Object(fixed_map));

    let mut handle_map = Map::new();
    for (tag, handle) in world.resource_handle_entries() {
        handle_map.insert(tag, JsonValue::String(handle_to_string(handle)));
    }
    out.insert("handle".to_string(), JsonValue::Object(handle_map));

    let mut value_map = Map::new();
    let mut value_det_map = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_map.insert(tag.clone(), JsonValue::String(value.canon_key()));
        value_json_map.insert(tag.clone(), resource_value_to_json(&value));
        value_det_map.insert(tag, resource_value_to_detjson(&value));
    }
    out.insert("value".to_string(), JsonValue::Object(value_map));
    out.insert("value_json".to_string(), JsonValue::Object(value_json_map));
    out.insert("value_det".to_string(), JsonValue::Object(value_det_map));

    JsonValue::Object(out)
}

fn serialize_world_resources_snapshot(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    out.insert("schema".to_string(), json!(RESOURCE_SNAPSHOT_SCHEMA_V2));
    out.insert(
        "unit_wire_schema".to_string(),
        json!(UNIT_VALUE_WIRE_SCHEMA_V2),
    );
    out.insert("next_entity".to_string(), json!(world.next_entity_cursor()));
    out.insert(
        "components".to_string(),
        JsonValue::Array(
            world
                .component_json_entries()
                .into_iter()
                .map(
                    |(entity, tag, value)| json!({"entity": entity.0, "tag": tag.0, "json": value}),
                )
                .collect(),
        ),
    );
    let mut json_map = Map::new();
    for (tag, value) in world.resource_json_entries() {
        json_map.insert(tag, JsonValue::String(value));
    }
    out.insert("json".to_string(), JsonValue::Object(json_map));
    let mut fixed_map = Map::new();
    for (tag, value) in world.resource_fixed64_entries() {
        fixed_map.insert(tag, json!({"raw_i64": value.raw_i64().to_string()}));
    }
    out.insert("fixed64".to_string(), JsonValue::Object(fixed_map));
    let mut handle_map = Map::new();
    for (tag, handle) in world.resource_handle_entries() {
        handle_map.insert(tag, JsonValue::String(handle_to_string(handle)));
    }
    out.insert("handle".to_string(), JsonValue::Object(handle_map));
    let mut value_det = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_det.insert(tag, resource_value_to_detjson(&value));
    }
    out.insert("value_det".to_string(), JsonValue::Object(value_det));
    JsonValue::Object(out)
}

fn runtime_state_snapshot(
    world: &DetNuri,
    public_observation_frames: Vec<JsonValue>,
    next_tick_id: u64,
) -> JsonValue {
    json!({
        "schema": "ddn.wasm_state_snapshot.v2",
        "resource_snapshot_schema": RESOURCE_SNAPSHOT_SCHEMA_V2,
        "unit_wire_schema": UNIT_VALUE_WIRE_SCHEMA_V2,
        "resources": serialize_world_resources_snapshot(world.world()),
        "param_overrides": {},
        "public_observation_frames": public_observation_frames,
        "next_tick_id": next_tick_id,
    })
}

pub fn collect_columns_and_row(
    world: &ddonirang_core::platform::NuriWorld,
) -> (Vec<JsonValue>, Vec<JsonValue>) {
    let mut table: BTreeMap<String, (String, JsonValue)> = BTreeMap::new();

    for (tag, value) in world.resource_fixed64_entries() {
        table.insert(tag, ("num".to_string(), json!(fixed64_to_f64(value))));
    }

    for (tag, value) in world.resource_json_entries() {
        if let Some((dtype, scalar)) = json_text_to_scalar(&value) {
            table.insert(tag, (dtype, scalar));
        }
    }

    for (tag, value) in world.resource_value_entries() {
        if let Some((dtype, scalar)) = resource_value_to_scalar(&value) {
            table.insert(tag, (dtype, scalar));
        }
    }

    let mut columns = Vec::with_capacity(table.len());
    let mut row = Vec::with_capacity(table.len());
    for (key, (dtype, value)) in table {
        columns.push(json!({
            "key": key,
            "dtype": dtype,
            "role": "state",
        }));
        row.push(value);
    }
    (columns, row)
}

fn serialize_patch(patch: &Patch) -> JsonValue {
    let mut items = Vec::new();
    for op in &patch.ops {
        match op {
            PatchOp::SetResourceJson { tag, json } => {
                items.push(json!({ "op": "set_resource_json", "tag": tag, "value": json }));
            }
            PatchOp::SetResourceFixed64 { tag, value } => {
                items.push(
                    json!({ "op": "set_resource_fixed64", "tag": tag, "value": value.to_string() }),
                );
            }
            PatchOp::SetResourceHandle { tag, handle } => {
                items.push(json!({ "op": "set_resource_handle", "tag": tag, "value": handle_to_string(*handle) }));
            }
            PatchOp::SetResourceValue { tag, value } => {
                items.push(json!({
                    "op": "set_resource_value",
                    "tag": tag,
                    "value": value.canon_key(),
                    "value_json": resource_value_to_json(value),
                }));
            }
            PatchOp::DivAssignResourceFixed64 { tag, rhs, .. } => {
                items.push(json!({ "op": "div_assign_resource_fixed64", "tag": tag, "value": rhs.to_string() }));
            }
            PatchOp::SetComponentJson { entity, tag, json } => {
                items.push(json!({ "op": "set_component_json", "entity": entity.0, "tag": tag.0, "value": json }));
            }
            PatchOp::RemoveComponent { entity, tag } => {
                items.push(json!({ "op": "remove_component", "entity": entity.0, "tag": tag.0 }));
            }
            PatchOp::EmitSignal { signal, targets } => {
                items.push(
                    json!({ "op": "emit_signal", "signal": signal.name(), "targets": targets }),
                );
            }
            PatchOp::GuardViolation { entity, rule_id } => {
                items.push(
                    json!({ "op": "guard_violation", "entity": entity.0, "rule_id": rule_id }),
                );
            }
        }
    }
    JsonValue::Array(items)
}

pub(crate) fn serialize_input_snapshot(input: &InputSnapshot, rng_base_seed: Option<u64>) -> JsonValue {
    let mut out = Map::new();
    out.insert("input_encoding".into(), json!(crate::input_receipt::SNAPSHOT_ENCODING));
    out.insert("keys_pressed".to_string(), json!(input.keys_pressed));
    out.insert("last_key_name".to_string(), json!(input.last_key_name));
    out.insert("pointer_x_i32".to_string(), json!(input.pointer_x_i32));
    out.insert("pointer_y_i32".to_string(), json!(input.pointer_y_i32));
    out.insert("dt".to_string(), json!(crate::input_receipt::exact_dt_text(input.dt)));
    out.insert(
        "frame_source".to_string(),
        json!(input.frame_source.label()),
    );
    out.insert(
        "frame_source_code".to_string(),
        json!(input.frame_source.code_u8()),
    );
    out.insert("rng_seed".to_string(), json!(input.rng_seed));
    if let Some(base_seed) = rng_base_seed {
        out.insert("rng_base_seed".to_string(), json!(base_seed));
    }
    out.insert(
        "ai_injections".to_string(),
        JsonValue::Array(
            input
                .ai_injections
                .iter()
                .map(|packet| {
                    json!({
                        "agent_id": packet.agent_id,
                        "recv_seq": packet.recv_seq,
                        "accepted_madi": packet.accepted_madi,
                        "target_madi": packet.target_madi,
                        "source": packet.source.label(),
                        "source_code": packet.source.code_u8(),
                        "source_meta": input_source_json(packet.source),
                        "intent": seulgi_intent_json(&packet.intent),
                    })
                })
                .collect(),
        ),
    );
    out.insert(
        "net_events".to_string(),
        JsonValue::Array(
            input
                .net_events
                .iter()
                .map(|event| {
                    json!({
                        "sender": event.sender.as_str(),
                        "seq": event.seq,
                        "order_key": event.order_key.as_str(),
                        "payload_detjson": event.payload_detjson.as_str(),
                        "source": event.source.label(),
                        "source_code": event.source.code_u8(),
                        "source_meta": input_source_json(event.source),
                    })
                })
                .collect(),
        ),
    );
    JsonValue::Object(out)
}

fn serialize_input_state(
    keys_pressed: u64,
    last_key_name: &str,
    pointer_x_i32: i32,
    pointer_y_i32: i32,
    dt: Fixed64,
    rng_seed: u64,
    rng_base_seed: u64,
) -> JsonValue {
    json!({
        "keys_pressed": keys_pressed,
        "last_key_name": last_key_name,
        "pointer_x_i32": pointer_x_i32,
        "pointer_y_i32": pointer_y_i32,
        "input_encoding": crate::input_receipt::SNAPSHOT_ENCODING,
        "dt": crate::input_receipt::exact_dt_text(dt),
        "frame_source": InputSource::Person.label(),
        "frame_source_code": InputSource::Person.code_u8(),
        "rng_seed": rng_seed,
        "rng_base_seed": rng_base_seed,
        "pending_ai_injection_count": 0,
    })
}

fn input_source_json(source: InputSource) -> JsonValue {
    json!({
        "label": source.label(),
        "code": source.code_u8(),
    })
}

fn seulgi_intent_json(intent: &SeulgiIntent) -> JsonValue {
    match intent {
        SeulgiIntent::None => json!({
            "kind": "none",
            "kind_code": intent.kind_u32(),
        }),
        SeulgiIntent::MoveTo { x, y } => json!({
            "kind": "move_to",
            "kind_code": intent.kind_u32(),
            "x": x.to_string(),
            "y": y.to_string(),
            "x_raw_i64": x.raw_i64(),
            "y_raw_i64": y.raw_i64(),
        }),
        SeulgiIntent::Attack { target_id } => json!({
            "kind": "attack",
            "kind_code": intent.kind_u32(),
            "target_id": target_id,
        }),
        SeulgiIntent::Say { text } => json!({
            "kind": "say",
            "kind_code": intent.kind_u32(),
            "text": text.as_str(),
        }),
    }
}

fn resource_json_text_to_json(text: &str) -> JsonValue {
    serde_json::from_str::<JsonValue>(text).unwrap_or_else(|_| JsonValue::String(text.to_string()))
}

fn json_text_to_scalar(text: &str) -> Option<(String, JsonValue)> {
    let trimmed = text.trim();
    if trimmed == "참" {
        return Some(("bool".to_string(), JsonValue::Bool(true)));
    }
    if trimmed == "거짓" {
        return Some(("bool".to_string(), JsonValue::Bool(false)));
    }
    if let Ok(parsed) = serde_json::from_str::<JsonValue>(trimmed) {
        return match parsed {
            JsonValue::Bool(v) => Some(("bool".to_string(), JsonValue::Bool(v))),
            JsonValue::Number(v) => Some(("num".to_string(), JsonValue::Number(v))),
            JsonValue::String(v) => Some(("str".to_string(), JsonValue::String(v))),
            _ => None,
        };
    }
    if let Ok(number) = trimmed.parse::<f64>() {
        if number.is_finite() {
            return Some(("num".to_string(), json!(number)));
        }
    }
    Some(("str".to_string(), JsonValue::String(text.to_string())))
}

fn resource_value_to_scalar(value: &ResourceValue) -> Option<(String, JsonValue)> {
    match value {
        ResourceValue::Bool(v) => Some(("bool".to_string(), JsonValue::Bool(*v))),
        ResourceValue::Fixed64(v) => Some(("num".to_string(), json!(fixed64_to_f64(*v)))),
        ResourceValue::Unit(v) => Some(("num".to_string(), json!(fixed64_to_f64(v.value)))),
        ResourceValue::String(v) => Some(("str".to_string(), JsonValue::String(v.clone()))),
        _ => None,
    }
}

fn resource_value_to_json(value: &ResourceValue) -> JsonValue {
    if let Some(formula) = formula_from_resource_value(value) {
        let (dialect, raw) = formula_summary_parts(&formula);
        return json!({
            "dialect": dialect,
            "raw": raw,
        });
    }
    match value {
        ResourceValue::BasicTensorV1(_) => crate::unit_wire::resource_value_to_detjson(value),
        ResourceValue::None => JsonValue::Null,
        ResourceValue::Bool(v) => JsonValue::Bool(*v),
        ResourceValue::Fixed64(v) => json!(fixed64_to_f64(*v)),
        ResourceValue::Unit(v) => json!(fixed64_to_f64(v.value)),
        ResourceValue::String(s) => JsonValue::String(s.clone()),
        ResourceValue::ResourceHandle(handle) => JsonValue::String(handle_to_string(*handle)),
        ResourceValue::List(items) => {
            JsonValue::Array(items.iter().map(resource_value_to_json).collect())
        }
        ResourceValue::Set(items) => {
            JsonValue::Array(items.values().map(resource_value_to_json).collect())
        }
        ResourceValue::Map(entries) => {
            let mut obj = Map::new();
            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                obj.insert(key, resource_value_to_json(&entry.value));
            }
            JsonValue::Object(obj)
        }
    }
}

fn build_view_meta(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();

    if let Some(width) = world.get_resource_fixed64(BOGAE_WIDTH_TAG) {
        out.insert("canvas_width".to_string(), json!(fixed64_to_f64(width)));
    }
    if let Some(height) = world.get_resource_fixed64(BOGAE_HEIGHT_TAG) {
        out.insert("canvas_height".to_string(), json!(fixed64_to_f64(height)));
    }

    if let Some(draw_list_value) = world.get_resource_value(BOGAE_DRAWLIST_TAG) {
        if let ResourceValue::List(items) = draw_list_value {
            let draw_items: Vec<JsonValue> = items
                .iter()
                .filter_map(resource_value_to_draw_item)
                .collect();
            out.insert("draw_list".to_string(), JsonValue::Array(draw_items));
            out.insert(
                "draw_list_meta".to_string(),
                json!({
                    "deprecated": true,
                    "use_instead": "view_meta.space2d",
                }),
            );
        }
    }

    let mut available_families = Vec::new();
    if let Some(grid2d_json) = derive_grid2d_from_bogae(world) {
        if let Ok(grid2d) = serde_json::from_str::<JsonValue>(&grid2d_json) {
            out.insert("grid2d".to_string(), grid2d);
            available_families.push("grid2d");
        }
    } else if let Some(space2d_json) = derive_space2d_from_bogae(world) {
        if let Ok(space2d) = serde_json::from_str::<JsonValue>(&space2d_json) {
            out.insert("space2d".to_string(), space2d);
            available_families.push("space2d");
        }
    }
    if let Some(graph_json) = derive_graph_from_points(world) {
        if let Ok(graph) = serde_json::from_str::<JsonValue>(&graph_json) {
            out.insert("graph".to_string(), graph);
            available_families.push("graph");
        }
    }
    append_view_stack_meta(&mut out, &available_families);
    JsonValue::Object(out)
}

fn append_view_stack_meta(out: &mut Map<String, JsonValue>, available_families: &[&str]) {
    let Some((primary_family, secondary_families)) = available_families.split_first() else {
        return;
    };
    out.insert(
        "primary".to_string(),
        json!({
            "family": *primary_family,
            "role": "main",
        }),
    );
    let secondary = secondary_families
        .iter()
        .map(|family| {
            json!({
                "family": *family,
                "role": "secondary",
            })
        })
        .collect::<Vec<_>>();
    out.insert("secondary".to_string(), JsonValue::Array(secondary));
    out.insert("overlays".to_string(), JsonValue::Array(Vec::new()));
}

pub(crate) fn derive_graph_from_points(
    world: &ddonirang_core::platform::NuriWorld,
) -> Option<String> {
    let mut series = Vec::new();
    let mut single_series_labels = None;
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for tag in GRAPH_POINTS_TAGS {
        let Some(value) = world.get_resource_value(tag) else {
            continue;
        };
        let Some(graph_series) = graph_series_from_resource_value(&value) else {
            continue;
        };
        if graph_series.points.is_empty() {
            continue;
        }
        for (x, y) in &graph_series.points {
            if x.is_finite() {
                x_min = x_min.min(*x);
                x_max = x_max.max(*x);
            }
            if y.is_finite() {
                y_min = y_min.min(*y);
                y_max = y_max.max(*y);
            }
        }
        let mut row = Map::new();
        row.insert(
            "id".to_string(),
            json!(graph_series.y_label.as_deref().unwrap_or(tag)),
        );
        row.insert("source_ref".to_string(), json!(*tag));
        row.insert(
            "points".to_string(),
            JsonValue::Array(
                graph_series
                    .points
                    .iter()
                    .map(|(x, y)| json!([x, y]))
                    .collect(),
            ),
        );
        if let (Some(label), Some(source_ref)) = (
            graph_series.y_label.as_ref(),
            graph_series.y_source_ref.as_ref(),
        ) {
            row.insert("label".to_string(), json!(label));
            row.insert("label_source_ref".to_string(), json!(source_ref));
        }
        single_series_labels = Some((
            graph_series.x_label,
            graph_series.x_source_ref,
            graph_series.y_label,
            graph_series.y_source_ref,
        ));
        series.push(JsonValue::Object(row));
    }
    if series.is_empty() {
        return None;
    }
    let mut graph = Map::new();
    graph.insert("schema".to_string(), json!(GRAPH_SCHEMA));
    graph.insert("series".to_string(), JsonValue::Array(series));
    if x_min.is_finite() && x_max.is_finite() && y_min.is_finite() && y_max.is_finite() {
        let mut axis = Map::from_iter([
            ("x_min".to_string(), json!(x_min)),
            ("x_max".to_string(), json!(x_max)),
            ("y_min".to_string(), json!(y_min)),
            ("y_max".to_string(), json!(y_max)),
        ]);
        if graph
            .get("series")
            .and_then(JsonValue::as_array)
            .map(Vec::len)
            == Some(1)
        {
            if let Some((x_label, x_source_ref, y_label, y_source_ref)) = single_series_labels {
                if let (Some(label), Some(source_ref)) = (x_label, x_source_ref) {
                    axis.insert("x_label".to_string(), json!(label));
                    axis.insert("x_label_source_ref".to_string(), json!(source_ref));
                }
                if let (Some(label), Some(source_ref)) = (y_label, y_source_ref) {
                    axis.insert("y_label".to_string(), json!(label));
                    axis.insert("y_label_source_ref".to_string(), json!(source_ref));
                }
            }
        }
        graph.insert("axis".to_string(), JsonValue::Object(axis));
    }
    graph.insert(
        "meta".to_string(),
        json!({"graph_kind":"curve", "update":"replace", "source_kind":"runtime_resource"}),
    );
    Some(JsonValue::Object(graph).to_string())
}

#[derive(Default)]
struct GraphSeriesResource {
    points: Vec<(f64, f64)>,
    x_label: Option<String>,
    x_source_ref: Option<String>,
    y_label: Option<String>,
    y_source_ref: Option<String>,
}

fn graph_series_from_resource_value(value: &ResourceValue) -> Option<GraphSeriesResource> {
    let ResourceValue::List(items) = value else {
        return None;
    };
    let mut series = GraphSeriesResource::default();
    for item in items {
        if let Some(point) = resource_value_to_point(item) {
            series.points.push(point);
        }
        let ResourceValue::Map(entries) = item else {
            continue;
        };
        if series.x_label.is_none() {
            series.x_label = resource_map_string(entries, "x_label");
        }
        if series.x_source_ref.is_none() {
            series.x_source_ref = resource_map_string(entries, "x_source_ref");
        }
        if series.y_label.is_none() {
            series.y_label = resource_map_string(entries, "y_label");
        }
        if series.y_source_ref.is_none() {
            series.y_source_ref = resource_map_string(entries, "y_source_ref");
        }
    }
    Some(series)
}

fn resource_map_string(
    entries: &std::collections::BTreeMap<String, ddonirang_core::platform::ResourceMapEntry>,
    wanted: &str,
) -> Option<String> {
    entries.values().find_map(|entry| {
        if resource_key_to_string(&entry.key) != wanted {
            return None;
        }
        match &entry.value {
            ResourceValue::String(value) if !value.trim().is_empty() => Some(value.clone()),
            _ => None,
        }
    })
}

fn resource_value_to_point(value: &ResourceValue) -> Option<(f64, f64)> {
    match value {
        ResourceValue::Map(entries) => {
            let mut x = None;
            let mut y = None;
            for entry in entries.values() {
                match resource_key_to_string(&entry.key).as_str() {
                    "x" => x = resource_value_to_number(&entry.value),
                    "y" => y = resource_value_to_number(&entry.value),
                    _ => {}
                }
            }
            Some((x?, y?))
        }
        ResourceValue::List(items) if items.len() >= 2 => Some((
            resource_value_to_number(&items[0])?,
            resource_value_to_number(&items[1])?,
        )),
        _ => None,
    }
}

fn resource_value_to_number(value: &ResourceValue) -> Option<f64> {
    match value {
        ResourceValue::Fixed64(v) => Some(fixed64_to_f64(*v)),
        ResourceValue::Unit(v) => Some(fixed64_to_f64(v.value)),
        ResourceValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
        ResourceValue::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn derive_space2d_from_bogae(world: &ddonirang_core::platform::NuriWorld) -> Option<String> {
    let list = world.get_resource_value(BOGAE_DRAWLIST_TAG)?;
    let ResourceValue::List(items) = list else {
        return None;
    };
    let mut draw_items = Vec::new();
    for item in &items {
        if let Some(obj) = resource_value_to_draw_item(item) {
            draw_items.push(obj);
        }
    }
    if draw_items.is_empty() {
        return None;
    }
    let mut space2d = Map::new();
    space2d.insert(
        "schema".to_string(),
        JsonValue::String("seamgrim.space2d.v0".to_string()),
    );
    space2d.insert("drawlist".to_string(), JsonValue::Array(draw_items));
    if let (Some(w), Some(h)) = (
        world.get_resource_fixed64(BOGAE_WIDTH_TAG),
        world.get_resource_fixed64(BOGAE_HEIGHT_TAG),
    ) {
        let w = fixed64_to_f64(w);
        let h = fixed64_to_f64(h);
        if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 {
            space2d.insert(
                "camera".to_string(),
                json!({
                    "x_min": 0.0,
                    "x_max": w,
                    "y_min": 0.0,
                    "y_max": h,
                }),
            );
        }
    }
    Some(JsonValue::Object(space2d).to_string())
}

fn derive_grid2d_from_bogae(world: &ddonirang_core::platform::NuriWorld) -> Option<String> {
    let list = world.get_resource_value(BOGAE_DRAWLIST_TAG)?;
    let ResourceValue::List(items) = list else {
        return None;
    };
    let draw_items: Vec<JsonValue> = items
        .iter()
        .filter_map(resource_value_to_draw_item)
        .collect();
    if draw_items.is_empty()
        || !draw_items.iter().all(|item| {
            item.get("id")
                .and_then(JsonValue::as_str)
                .is_some_and(|id| id.starts_with("격자게임셀_") || id.starts_with("cell."))
        })
    {
        return None;
    }
    Some(
        json!({
            "schema": "seamgrim.grid2d.v0",
            "drawlist": draw_items,
            "source": "std_grid_game_bogae_drawlist"
        })
        .to_string(),
    )
}

fn resource_value_to_draw_item(value: &ResourceValue) -> Option<JsonValue> {
    let ResourceValue::Map(entries) = value else {
        return None;
    };
    let mut obj = Map::new();
    let mut kind_raw: Option<String> = None;
    for entry in entries.values() {
        let key = resource_key_to_string(&entry.key);
        let json_value = resource_value_to_json(&entry.value);
        match key.as_str() {
            "결" | "kind" | "shape" | "도형" | "형태" => {
                kind_raw = Some(resource_value_to_string(&entry.value));
            }
            "채움색" | "채움" => {
                obj.insert("fill".to_string(), json_value);
            }
            "색" | "선색" => {
                obj.insert("color".to_string(), json_value);
            }
            "굵기" => {
                obj.insert("width".to_string(), json_value);
            }
            "크기" => {
                obj.insert("size".to_string(), json_value);
            }
            "글" | "내용" => {
                obj.insert("text".to_string(), json_value);
            }
            "층" | "레이어" | "layer" | "layerIndex" | "z" | "z_index" => {
                obj.insert("layer_index".to_string(), json_value);
            }
            "그룹" | "묶음" | "group" | "groupId" => {
                obj.insert("group_id".to_string(), json_value);
            }
            _ => {
                obj.insert(key, json_value);
            }
        }
    }
    if let Some(raw) = kind_raw {
        obj.insert("kind".to_string(), JsonValue::String(map_bogae_kind(&raw)));
    }
    Some(JsonValue::Object(obj))
}

fn resource_value_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

fn map_bogae_kind(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("rect") {
        return "rect".to_string();
    }
    if lower.contains("text") {
        return "text".to_string();
    }
    if lower.contains("circle") {
        return "circle".to_string();
    }
    if lower.contains("polygon") {
        return "polygon".to_string();
    }
    if lower.contains("poly") {
        return "polyline".to_string();
    }
    if lower.contains("line") {
        return "line".to_string();
    }
    if lower.contains("point") {
        return "point".to_string();
    }
    if lower.contains("arrow") {
        return "arrow".to_string();
    }
    if lower.contains("sprite") {
        return "rect".to_string();
    }
    if lower.contains("사각") || lower.contains("네모") {
        return "rect".to_string();
    }
    if lower.contains("동그라미") || lower == "원" || lower.contains("원형") {
        return "circle".to_string();
    }
    if lower.contains("선분") || lower == "선" {
        return "line".to_string();
    }
    if lower == "점" || lower.contains("점형") {
        return "point".to_string();
    }
    if lower.contains("화살") || lower.contains("살표") {
        return "arrow".to_string();
    }
    if lower == "글" || lower.contains("글자") || lower.contains("문자") {
        return "text".to_string();
    }
    if lower.contains("다각") {
        return "polygon".to_string();
    }
    raw.to_string()
}

fn resource_key_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

fn hash_json_string(value: &JsonValue) -> String {
    let canonical = value.to_string();
    let digest = blake3_hash(canonical.as_bytes());
    format!("blake3:{}", hex::encode(digest.as_bytes()))
}

fn handle_to_string(handle: ResourceHandle) -> String {
    format!("handle:{}", handle.to_hex())
}

fn fixed64_to_f64(value: Fixed64) -> f64 {
    value.raw_i64() as f64 / Fixed64::ONE_RAW as f64
}

#[cfg(test)]
mod tests {
    // 이전 제품의 전체 JSON 재직렬화 루프. 실행이 끝난 동일 공개 후보에서
    // 영수증 초기값만 복원하여 최적화 경로와 완성 바이트/실패 영수증을 대조한다.
    fn receipt_full_scan_reference(mut summary: super::JsonValue,
        limits: super::StandardResourceLimitsV1, file_path: &str)
        -> Result<super::JsonValue, super::ExecutionDiagnostic> {
        use super::*;
        let mut counters: crate::work_cost::WorkCostCounters =
            serde_json::from_value(summary["work_cost_receipt"]["counters"].clone()).unwrap();
        counters.result_artifact_bytes = 0;
        let mut meter = crate::work_cost::WorkCostMeter::from_receipt_parts(
            counters, limits, summary["work_cost_receipt"]["rd206_resource_capability"].as_bool().unwrap());
        summary["work_cost_receipt"] = serde_json::to_value(meter.receipt()).unwrap();
        for _ in 0..16 {
            let bytes = serialized_json_byte_count(&summary).map_err(|error|
                run_summary_diagnostic("E_RESULT_ARTIFACT_SERIALIZE", error.to_string(), file_path, "resource_commit"))?;
            if meter.receipt().counters.result_artifact_bytes == bytes as u64 { return Ok(summary); }
            meter.set_result_artifact_bytes(bytes).map_err(|failure|
                work_cost_failure_diagnostic(failure, file_path, "resource_commit"))?;
            summary["work_cost_receipt"] = serde_json::to_value(meter.receipt()).unwrap();
        }
        Err(run_summary_diagnostic("E_RESULT_ARTIFACT_SIZE_FIXED_POINT",
            "결과 artifact byte 크기와 WorkCostV1 영수증이 수렴하지 않았습니다", file_path, "resource_commit"))
    }

    #[test]
    fn receipt_cache_matches_full_scan_source_1_2_40_and_exact_limit_plus_one() {
        use super::*;
        for source in ["(매마디)마다 { }.",
            "(매마디)마다 { 글 <- \"한글🙂\\n\\t\\\"\". 글 보여주기. }."] {
            for ticks in [1, 2, 40] {
                let file = "receipt-cache.ddn";
                let execute = |limits| run_summary_from_source_with_assets_diagnostic(
                    source, file, Some(ticks), None, Some(limits), None, None, None, 0);
                let actual = execute(StandardResourceLimitsV1::CANDIDATE).unwrap();
                let old = receipt_full_scan_reference(actual.clone(), StandardResourceLimitsV1::CANDIDATE, file).unwrap();
                assert_eq!(serde_json::to_vec(&actual).unwrap(), serde_json::to_vec(&old).unwrap());
                assert_eq!(hash_json_string(&actual), hash_json_string(&old));
                // limit의 자릿수도 영수증 바이트에 포함된다. 원 루프로 정확한 경계를 구한다.
                let mut exact = serialized_json_byte_count(&actual).unwrap() as u64;
                for _ in 0..16 {
                    let limits = StandardResourceLimitsV1 { result_artifact_bytes: exact,
                        ..StandardResourceLimitsV1::CANDIDATE };
                    let bounded = receipt_full_scan_reference(actual.clone(), limits, file).unwrap();
                    let next = serialized_json_byte_count(&bounded).unwrap() as u64;
                    if next == exact { break; }
                    exact = next;
                }
                let limits = StandardResourceLimitsV1 { result_artifact_bytes: exact,
                    ..StandardResourceLimitsV1::CANDIDATE };
                let expected = receipt_full_scan_reference(actual.clone(), limits, file).unwrap();
                assert_eq!(serde_json::to_vec(&execute(limits).unwrap()).unwrap(), serde_json::to_vec(&expected).unwrap());
                let below = StandardResourceLimitsV1 { result_artifact_bytes: exact - 1, ..limits };
                let expected_failure = receipt_full_scan_reference(actual.clone(), below, file).unwrap_err();
                let actual_failure = execute(below).unwrap_err();
                assert_eq!(actual_failure.to_json_value(), expected_failure.to_json_value());
                assert_eq!(actual_failure.phase, "resource_commit");
            }
        }
    }

    #[test]
    fn receipt_cache_decomposition_handles_unicode_empty_values_and_checked_overflow() {
        use super::*;
        for receipt in [JsonValue::Null, json!({}), json!(""),
            json!({"counter": 0, "text": "한글🙂\"\\\n\u{0000}"})] {
            for samples in [1, 2, 40] {
                let mut summary = json!({"work_cost_receipt": receipt,
                    "outside": (0..samples).map(|i|json!({"i":i,"text":"공개 🙂\\n"})).collect::<Vec<_>>()});
                let outside = serialized_json_byte_count(&summary).unwrap()
                    .checked_sub(serialized_json_byte_count(&summary["work_cost_receipt"]).unwrap()).unwrap();
                for replacement in [json!({}), json!({"counter": 999999}), json!("새 영수증🙂\n")] {
                    summary["work_cost_receipt"] = replacement;
                    let receipt_bytes = serialized_json_byte_count(&summary["work_cost_receipt"]).unwrap();
                    assert_eq!(outside.checked_add(receipt_bytes).unwrap(), serde_json::to_vec(&summary).unwrap().len());
                }
            }
        }
        let outside = usize::MAX;
        let error = outside.checked_add(1).ok_or_else(|| serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::Other, "JSON byte count overflow"))).unwrap_err();
        let mut counter = JsonByteCounter { bytes: outside };
        let original_error = serde_json::to_writer(&mut counter, &json!("x")).unwrap_err();
        assert_eq!(error.to_string(), original_error.to_string());
        assert_eq!(error.is_io(), original_error.is_io());
        assert_eq!(counter.bytes, usize::MAX);
    }

    // 최적화 전 literal을 그대로 유지한 공개 바이트 oracle.
    fn engine_state_literal_reference(world: &super::DetNuri, tick_id: u64,
        input_json: super::JsonValue, patch: &super::Patch) -> super::JsonValue {
        use super::*;
        let committed_madi = tick_id.saturating_sub(1);
        let resources = serialize_world_resources(world.world());
        let (columns, row) = collect_columns_and_row(world.world());
        let view_meta = build_view_meta(world.world());
        let view_hash = hash_json_string(&view_meta);
        let patch_json = serialize_patch(patch);
        json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "tick_id": tick_id,
            "committed_madi": committed_madi,
            "state_hash": current_state_hash(world.world()),
            "input": input_json.clone(),
            "resources": resources.clone(),
            "channels": columns.clone(),
            "row": row.clone(),
            "streams": JsonValue::Object(Map::new()),
            "patch": patch_json.clone(),
            "state": {
                "tick_id": tick_id,
                "committed_madi": committed_madi,
                "input": input_json,
                "resources": resources,
                "channels": columns,
                "row": row,
                "patch": patch_json,
                "streams": JsonValue::Object(Map::new())
            },
            "view_meta": view_meta,
            "view_hash": view_hash
        })
    }

    fn assert_engine_state_literal_bytes(world: &super::DetNuri, tick: u64,
        input: super::JsonValue, patch: &super::Patch) {
        let before_hash = super::current_state_hash(world.world());
        let expected = engine_state_literal_reference(world, tick, input.clone(), patch);
        let actual = super::build_engine_state_json(world, tick, input, patch);
        assert_eq!(serde_json::to_vec(&actual).unwrap(), serde_json::to_vec(&expected).unwrap());
        assert_eq!(serde_json::to_string_pretty(&actual).unwrap(),
            serde_json::to_string_pretty(&expected).unwrap());
        assert_eq!(super::hash_json_string(&actual), super::hash_json_string(&expected));
        assert_eq!(super::current_state_hash(world.world()), before_hash);
        assert!(actual.as_object().unwrap().contains_key("patch"));
        assert!(actual["state"].as_object().unwrap().contains_key("patch"));
        assert!(actual["patch"].is_array());
        assert_eq!(actual["patch"], actual["state"]["patch"]);
        assert_eq!(actual["patch"].as_array().unwrap().len(), patch.ops.len());
    }

    #[test]
    fn engine_state_patch_move_matches_literal_nested_unicode_unit_shape_and_empty() {
        use super::*;
        let mut world = DetNuri::new();
        let nested = ResourceValue::List(vec![
            ResourceValue::String("한글🙂\"\\\n\t\u{0000}".into()),
            ResourceValue::List(vec![ResourceValue::None, ResourceValue::Bool(true)]),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2), ddonirang_core::Unit::Mole)),
        ]);
        let shape = ResourceValue::map_from_entries(vec![
            ddonirang_core::platform::ResourceMapEntry { key: ResourceValue::String("kind".into()), value: ResourceValue::String("polyline".into()) },
            ddonirang_core::platform::ResourceMapEntry { key: ResourceValue::String("points".into()), value: ResourceValue::List(vec![
                ResourceValue::List(vec![ResourceValue::Fixed64(Fixed64::ZERO),
                    ResourceValue::Fixed64(Fixed64::ONE)]),
            ]) },
        ]);
        let mut patch = Patch::default();
        patch.ops.push(PatchOp::SetResourceValue { tag: "중첩".into(), value: nested.clone() });
        patch.ops.push(PatchOp::SetResourceValue { tag: "그림".into(), value: shape });
        patch.ops.push(PatchOp::SetResourceJson { tag: "json".into(), json: "{\"문장\":\"🙂\\n\"}".into() });
        patch.ops.push(PatchOp::SetResourceValue { tag: "중첩".into(), value: nested });
        world.apply_patch(&patch, 0, &mut ddonirang_core::signals::VecSignalSink::default());
        assert_engine_state_literal_bytes(&world, 1, json!({"text":"입력🙂", "nested":[1,null]}), &patch);
        assert_engine_state_literal_bytes(&world, 0, JsonValue::Null, &Patch::default());
    }

    #[test]
    fn engine_state_patch_move_matches_literal_source_observations_and_receipt() {
        use super::*;
        let source = "채비 { 시간 <- 0. }. (매마디)마다 { 시간 <- 시간 + 1. (시간: 시간 * 1@s, 값: 시간) 보임. 시간 보여주기. }.";
        let program = DdnProgram::from_source(source, "patch-move.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let mut world = DetNuri::new();
        let mut defaults = HashMap::new();
        seed_bogae_defaults(&mut defaults);
        for tick in 0..3 {
            let input = InputSnapshot { tick_id: tick, dt: Fixed64::ONE, keys_pressed: 0,
                last_key_name: String::new(), pointer_x_i32: 0, pointer_y_i32: 0,
                ai_injections: Vec::new(), net_events: Vec::new(),
                frame_source: InputSource::Person, rng_seed: tick };
            let output = runner.run_update(world.world(), &input, &defaults).unwrap();
            assert!(!output.public_observations.is_empty());
            world.apply_patch(&output.patch, tick, &mut ddonirang_core::signals::VecSignalSink::default());
            let receipt_before = serde_json::to_vec(&runner.work_cost_receipt()).unwrap();
            assert_engine_state_literal_bytes(&world, tick + 1,
                serialize_input_snapshot(&input, Some(0)), &output.patch);
            assert_eq!(serde_json::to_vec(&runner.work_cost_receipt()).unwrap(), receipt_before);
        }
    }

    #[test]
    fn json_byte_count_matches_allocating_serialization() {
        let fixtures = [
            json!("한글🙂\"\\\n\t\u{0000}"),
            json!({"중첩": [null, true, -12.5, {"escaped": "\r\n\\\""}]}),
            json!({"rows": (0..4096).map(|index| json!({
                "index": index, "text": "원 표본과 공개 필드 🙂", "values": [1, 2, 3]
            })).collect::<Vec<_>>()}),
        ];
        for fixture in fixtures {
            assert_eq!(super::serialized_json_byte_count(&fixture).unwrap(),
                serde_json::to_vec(&fixture).unwrap().len());
        }
        let summary = super::run_summary_from_supported_source_profile(
            "(매마디)마다 { a <- -0.08. a 보여주기. }.",
            "byte-count.ddn", Some(2), Some("v1-core-v25"),
        ).expect("public summary executes");
        let bytes = serde_json::to_vec(&summary).unwrap().len();
        assert_eq!(super::serialized_json_byte_count(&summary).unwrap(), bytes);
        assert_eq!(summary["work_cost_receipt"]["counters"]["result_artifact_bytes"], bytes);
    }

    #[test]
    fn summary_json_moves_preserve_all_public_duplicate_slots() {
        let summary = super::run_summary_from_supported_source_profile(
            "(매마디)마다 { 값 <- -0.08. (시간: 1@s, 값: 값) 보여주기. \"한글🙂\\n\" 보여주기. }.",
            "summary-move.ddn", Some(2), Some("v1-core-v25"),
        ).expect("public observations and output execute");
        let state = &summary["engine_state"];
        let mut literal_reference = summary.clone();
        for key in ["resources", "channels", "row", "observation_columns",
            "public_observation_trace", "public_observation_trace_hash",
            "public_observation_table_projection", "output_rows"] {
            assert!(!summary[key].is_null(), "{key} must be published");
            literal_reference[key] = serde_json::to_value(&state[key]).unwrap();
        }
        literal_reference["output_log_texts"] = serde_json::to_value(
            super::output_log_texts_from_state_json(state)).unwrap();
        assert_eq!(serde_json::to_vec(&summary).unwrap(),
            serde_json::to_vec(&literal_reference).unwrap());
        assert_eq!(serde_json::to_vec_pretty(&summary).unwrap(),
            serde_json::to_vec_pretty(&literal_reference).unwrap());
    }

    #[test]
    fn a3_p1_summary_projects_numbers_without_rewriting_world_output() {
        let summary = super::run_summary_from_supported_source_profile(
            "(매마디)마다 { 값 <- 20 - 9.8 * 0.1. 값 보여주기. \"19.019999999\" 보여주기. }.",
            "display-summary.ddn", Some(2), Some("v1-core-v25"),
        ).unwrap();
        let rows = summary["output_display"]["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0]["text"], "19.02");
        assert_eq!(rows[1]["text"], "19.019999999");
        assert_eq!(rows[2]["madi"], 1);
        assert_eq!(rows[2]["line_no"], 3);
        assert_eq!(summary["engine_state"]["output_display"], summary["output_display"]);
        assert_eq!(summary["output_display"]["world_hash_participation"], false);
        assert_ne!(summary["output_log_texts"][0], rows[0]["text"]);
        assert_eq!(summary["output_log_texts"][1], "19.019999999");
    }

    #[test]
    fn json_byte_counter_is_non_allocating_and_overflow_is_atomic() {
        use std::io::Write;
        assert_eq!(std::mem::size_of::<super::JsonByteCounter>(), std::mem::size_of::<usize>());
        let mut counter = super::JsonByteCounter { bytes: usize::MAX - 1 };
        counter.write_all(b"x").unwrap();
        assert_eq!(counter.bytes, usize::MAX);
        let error = serde_json::to_writer(&mut counter, &json!("x")).unwrap_err();
        assert!(error.is_io());
        assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::Other));
        assert_eq!(counter.bytes, usize::MAX);
        assert_eq!(counter.write(b"").unwrap(), 0);
        counter.flush().unwrap();
    }

    #[test]
    fn wo09_signed_fixed64_resource_display_matches_numeric_value() {
        let source = "(매마디)마다 { a <- -0.08. b <- -1.5. c <- -2. d <- 0.25. }.";
        let summary = super::run_summary_from_supported_source_profile(
            source, "wo09-display.ddn", Some(1), Some("v1-core-v25"),
        ).expect("canonical source executes");
        for (name, expected) in [("a", "-0.08"), ("b", "-1.5"), ("c", "-2"), ("d", "0.25")] {
            assert_eq!(summary["resources"]["fixed64"][name], expected, "{name}");
        }
    }

    use super::*;
    use ddonirang_core::KEY_D;

    const P1A_V1_CORE_PROFILE: &str = "v1-core-v25";

    #[test]
    fn wo03_explicit_profile_controls_arithmetic_without_token_inference() {
        for source in [
            "(1 / 0) 보여주기.",
            "값 <- (1 / 0). 값 보여주기.",
            "값 := (1 / 0). 값 보여주기.",
            "(매마디)마다 { (1 / 0) 보여주기. }.",
        ] {
            let error = run_summary_from_supported_source_profile(source, "wo03.ddn", Some(1), Some(P1A_V1_CORE_PROFILE))
                .expect_err("numeric failure is not a successful signal");
            assert!(error.contains("0으로 나눌 수 없습니다"), "{error}");
            assert!(run_summary_from_supported_source_profile(source, "wo03.ddn", Some(1), None).is_err());
        }
        for source in ["(1 / 0) 보여주기.", "값 <- (1 / 0). 값 보여주기."] {
            let error = run_summary_from_source(source, "legacy.ddn", Some(1)).expect_err("legacy numeric fault also fails");
            assert!(error.contains("0으로 나눌 수 없습니다"), "{error}");
        }
    }

    #[test]
    fn wo02_summary_entrypoints_reject_inferred_type_change_before_execution() {
        for source in [
            "값 := 1. 값 <- \"가\". 값 보여주기.",
            "값 <- 1. 값 <- \"가\". 값 보여주기.",
        ] {
            // D7: current sources bind their profile explicitly; the legacy
            // internal path is not authority for V25 validation semantics.
            for profile in [Some(P1A_V1_CORE_PROFILE)] {
                let error = run_summary_from_source_with_assets_diagnostic(
                    source, "wo02-rebind.ddn", Some(1), None, None, None,
                    profile.map(|_| resolve_supported_source_profile(profile).unwrap()),
                    None, 0,
                ).expect_err("all execution frontdoors must reject the same type change");
                assert_eq!(error.phase, "static_validation");
                assert!(error.message.contains("E_"));
            }
        }
    }

    #[test]
    fn wo02_summary_checks_even_skipped_logical_operand_before_execution() {
        for source in ["거짓 그리고 1 보여주기.", "참 또는 1 보여주기."] {
            let error = run_summary_from_source_diagnostic(source, "wo02-logical.ddn", Some(1))
                .expect_err("short circuit does not skip static type checks");
            assert_eq!(error.phase, "static_validation");
        }
    }

    #[test]
    fn shared_sha256_identity_preserves_product_artifact_contract() {
        assert_eq!(
            sha256_identity(b"ddn-artifact-v1"),
            "sha256:dd0db943346466681ea7255089423388fcf354f758febcd3d555a129d31aef16"
        );
    }

    #[test]
    fn explicit_age_target_is_per_invocation_and_fail_closed() {
        let source = "매틱:움직씨 = {\n  기계 <- 상태머신{\n    빨강, 초록 으로 이뤄짐.\n    빨강 으로 시작.\n    빨강 에서 초록 으로.\n  }.\n}\n";
        let before = crate::ddn_runtime::default_age_target();
        let error = run_summary_from_source_with_age_target_diagnostic(
            source,
            "explicit-age-target.ddn",
            Some(1),
            AgeTarget::Age0,
        )
        .expect_err("AGE0 must reject an AGE1 state-machine surface");
        assert_eq!(error.code, "E_AGE_NOT_AVAILABLE");
        assert!(error.message.contains("state_machine_literal"));
        assert_eq!(crate::ddn_runtime::default_age_target(), before);

        let summary = run_summary_from_source_with_age_target_diagnostic(
            source,
            "explicit-age-target.ddn",
            Some(1),
            AgeTarget::Age1,
        )
        .expect("AGE1 must accept the same state-machine source");
        assert_eq!(summary["age_target_identity"], "AGE1");
        assert_eq!(summary["execution_envelope"]["age_target_identity"], "AGE1");
        assert_eq!(crate::ddn_runtime::default_age_target(), before);
    }

    #[test]
    fn p1a_supported_profile_run_executes_v25_definition_and_rebinding() {
        let source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n";
        let summary = run_summary_from_supported_source_profile_diagnostic(
            source,
            "p1a-positive.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect("V1-Core run summary");
        assert!(
            summary["output_log_texts"]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|row| row.as_str() == Some("2"))),
            "summary={summary}"
        );
        assert_eq!(summary["source_profile_identity"], P1A_V1_CORE_PROFILE);
    }

    #[test]
    fn supported_profile_seed_is_sealed_and_changes_run_identity() {
        let source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 보여주기.\n}\n";
        let first = run_summary_from_supported_source_profile_with_seed(
            source,
            "seeded-v25.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
            7,
        )
        .expect("seeded V1-Core run");
        let repeated = run_summary_from_supported_source_profile_with_seed(
            source,
            "seeded-v25.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
            7,
        )
        .expect("repeated seeded V1-Core run");
        let changed = run_summary_from_supported_source_profile_with_seed(
            source,
            "seeded-v25.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
            8,
        )
        .expect("changed-seed V1-Core run");

        assert_eq!(first, repeated);
        assert_eq!(
            first["execution_envelope"]["input_snapshot"]["rng_base_seed"],
            7
        );
        assert_eq!(first["engine_state"]["state"]["input"]["rng_base_seed"], 7);
        assert_ne!(
            first["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"][0]["run_id"],
            changed["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"][0]
                ["run_id"]
        );
    }

    #[test]
    fn live_control_input_and_v2_replay_share_ddn_result_state_and_trace() {
        use crate::input_tape::{mask_to_bytes, InputEvent, InputEventKind};
        let source = include_str!("../../tools/teul-cli/tests/fixtures/cli_control_input_r1.ddn");
        let tape = InputTape { version: 2, madi_hz: 60, records: vec![
            InputRecord { madi: 0, held_mask: mask_to_bytes(2), events: vec![InputEvent { sequence: 0, key_bit: 2, kind: InputEventKind::Press }] },
            InputRecord { madi: 1, held_mask: mask_to_bytes(0), events: vec![InputEvent { sequence: 1, key_bit: 2, kind: InputEventKind::Release }] },
            InputRecord { madi: 2, held_mask: mask_to_bytes(2), events: vec![InputEvent { sequence: 2, key_bit: 2, kind: InputEventKind::Press }] },
        ] };
        let live = run_summary_from_source_with_live_input(source, "control-input.ddn", Some(3), 0,
            Some(P1A_V1_CORE_PROFILE), 60, |madi| Ok(tape.records[madi as usize].clone())).expect("live DDN execution");
        let bytes = encode_input_tape(&tape).expect("V2 candidate");
        let replay_tape = crate::input_tape::decode_input_tape(&bytes).expect("V2 admission");
        let replay = run_summary_from_supported_source_profile_with_input_tape(source, "control-input.ddn", Some(3), Some(P1A_V1_CORE_PROFILE), &replay_tape).expect("replay");
        assert_eq!(live["output_log_texts"], json!(["참", "거짓", "참"]));
        assert_eq!(live, replay, "all result, input frames, state/trace hashes and receipts must match");
        let failure = run_summary_from_source_with_live_input(source, "control-input.ddn", Some(3), 0,
            Some(P1A_V1_CORE_PROFILE), 60, |madi| {
                if madi == 1 { Err("E_SAM_LIVE_QUEUE_OVERFLOW 입력 대기열 오류".into()) }
                else { Ok(tape.records[madi as usize].clone()) }
            }).expect_err("no partial summary after a later input failure");
        assert!(failure.contains("E_SAM_LIVE_QUEUE_OVERFLOW"));
        assert_eq!(serde_json::from_str::<JsonValue>(&failure).unwrap()["code"], "E_SAM_LIVE_QUEUE_OVERFLOW");
        assert!(!failure.contains("output_log_texts"));
        assert!(!failure.contains("state_hash"));
    }

    #[test]
    fn live_control_same_madi_edges_survive_shared_runtime_and_replay() {
        use crate::input_tape::{mask_to_bytes, InputEvent, InputEventKind};
        let source = "매마디:움직씨 := {\n    누름 <- (\"right\") 눌렸나.\n    순간 <- (\"right\") 막눌렸나.\n    누름 보여주기.\n    순간 보여주기.\n}\n";
        let tape = InputTape { version: 2, madi_hz: 60, records: vec![
            InputRecord { madi: 0, held_mask: mask_to_bytes(0), events: vec![
                InputEvent { sequence: 0, key_bit: 2, kind: InputEventKind::Press },
                InputEvent { sequence: 1, key_bit: 2, kind: InputEventKind::Release },
            ] },
            InputRecord { madi: 1, held_mask: mask_to_bytes(0), events: Vec::new() },
        ] };
        let live = run_summary_from_source_with_live_input(source, "transient.ddn", Some(2), 0, Some(P1A_V1_CORE_PROFILE), 60,
            |madi| Ok(tape.records[madi as usize].clone())).unwrap();
        let replay = run_summary_from_supported_source_profile_with_input_tape(source, "transient.ddn", Some(2), Some(P1A_V1_CORE_PROFILE), &tape).unwrap();
        assert_eq!(live["output_log_texts"], json!(["거짓", "참", "거짓", "거짓"]));
        assert_eq!(live, replay);
    }

    #[test]
    fn supported_profile_run_replays_shared_input_tape_without_private_cli_runtime() {
        let source = "매마디:움직씨 := {\n    1 보여주기.\n}\n";
        let tape = InputTape {
            version: 1,
            madi_hz: 60,
            records: vec![
                crate::input_tape::InputRecord {
                    madi: 0,
                    held_mask: crate::input_tape::mask_to_bytes(0),
                    events: Vec::new(),
                },
                crate::input_tape::InputRecord {
                    madi: 1,
                    held_mask: crate::input_tape::mask_to_bytes(1 << 1),
                    events: Vec::new(),
                },
                crate::input_tape::InputRecord {
                    madi: 2,
                    held_mask: crate::input_tape::mask_to_bytes(1 << 1),
                    events: Vec::new(),
                },
            ],
        };

        let first = run_summary_from_supported_source_profile_with_input_tape(
            source,
            "supported-input-tape.ddn",
            Some(3),
            Some(P1A_V1_CORE_PROFILE),
            &tape,
        )
        .expect("supported V25 input-tape run");
        let repeated = run_summary_from_supported_source_profile_with_input_tape(
            source,
            "supported-input-tape.ddn",
            Some(3),
            Some(P1A_V1_CORE_PROFILE),
            &tape,
        )
        .expect("deterministic repeated input-tape run");

        assert_eq!(first, repeated);
        assert_eq!(first["ticks_run"], 3);
        assert_eq!(first["output_log_texts"], json!(["1", "1", "1"]));
        assert_eq!(first["source_profile_identity"], P1A_V1_CORE_PROFILE);
        let frames = first["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"]
            .as_array()
            .expect("committed observation frames");
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0]["input_snapshot"]["keys_pressed"], 0);
        assert_eq!(frames[1]["input_snapshot"]["keys_pressed"], KEY_D);
        assert_eq!(frames[2]["input_snapshot"]["keys_pressed"], KEY_D);
        assert!(frames[0]["run_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("run:")));

        let mut changed_tape = tape.clone();
        changed_tape.records[1].held_mask = crate::input_tape::mask_to_bytes(0);
        let changed = run_summary_from_supported_source_profile_with_input_tape(
            source,
            "supported-input-tape.ddn",
            Some(3),
            Some(P1A_V1_CORE_PROFILE),
            &changed_tape,
        )
        .expect("changed input-tape run");
        assert_ne!(
            frames[0]["run_id"],
            changed["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"][0]
                ["run_id"]
        );
    }

    #[test]
    fn shared_input_tape_fails_closed_before_partial_execution() {
        let source = "매마디:움직씨 := {\n    1 보여주기.\n}\n";
        let tape = InputTape {
            version: 1,
            madi_hz: 60,
            records: vec![crate::input_tape::InputRecord {
                madi: 1,
                held_mask: crate::input_tape::mask_to_bytes(0),
                events: Vec::new(),
            }],
        };

        let error = run_summary_from_supported_source_profile_with_input_tape(
            source,
            "invalid-input-tape.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
            &tape,
        )
        .expect_err("out-of-order tape must fail closed");
        let diagnostic: JsonValue = serde_json::from_str(&error).expect("structured diagnostic");
        assert_eq!(diagnostic["code"], "E_SAM_RECORD_ORDER_MISMATCH");
        assert!(diagnostic.get("summary").is_none());
    }

    #[test]
    fn c1_review_source_diagnostics_preserve_code_and_location() {
        for (source, code) in [
            ("\u{feff}값 <- 1.", "E_SOURCE_UTF8_BOM_FORBIDDEN"),
            ("만약 참 이라면 { 1 보여주기. }.", "E_V25_IF_SURFACE"),
        ] {
            let error = run_summary_from_supported_source_profile_diagnostic(
                source, "review.ddn", Some(1), Some(P1A_V1_CORE_PROFILE),
            ).expect_err("invalid source must be rejected without masking its code");
            assert_eq!(error.code, code);
            assert!(error.message.contains(code));
        }
        let source = include_str!("../../solutions/seamgrim_ui_mvp/alpha1/잘못된입력/03_함수표_인자누락.ddn");
        let error = run_summary_from_supported_source_profile_diagnostic(
            source, "missing-pin.ddn", Some(1), Some(P1A_V1_CORE_PROFILE),
        ).expect_err("missing pin must preserve its original location");
        assert_eq!(error.code, "E_CALL_REQUIRED_PIN_MISSING");
        assert!(error.message.contains("(4:1)"), "{}", error.message);
        assert!(error.span.is_some());
    }

    #[test]
    fn p1a_supported_profile_type_mismatch_preserves_shared_identity() {
        let source = "매마디:움직씨 := {\n    값:수 := \"글\".\n}\n";
        let error = run_summary_from_supported_source_profile_diagnostic(
            source,
            "p1a-type-mismatch.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect_err("type mismatch must fail closed");
        assert_eq!(error.code, "E_RUNTIME_TYPE_MISMATCH");
        assert_eq!(error.offending_field.as_deref(), Some("값"));
        assert_eq!(error.expected.as_deref(), Some("수"));
        assert_eq!(error.actual.as_deref(), Some("글"));
    }

    #[test]
    fn supported_profile_canon_is_explicit_and_roundtrips_without_default_inference() {
        let source = "매마디:움직씨 := {\n    글:글 := \"첫\\줄둘\\따옴인용\\따옴\".\n}\n";
        let first =
            canonicalize_shared_source_with_supported_profile(source, Some(P1A_V1_CORE_PROFILE))
                .expect("supported V25 canon");
        let second = canonicalize_shared_source_with_supported_profile(
            &first.ddn,
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect("supported V25 canon reparse");
        assert_eq!(first.ddn, second.ddn);
        assert!(first.ddn.contains("\\n"));
        assert!(first.ddn.contains("\\\"인용\\\""));

        // D7 supersedes token-driven current-default selection. A legacy
        // internal canon path must not silently reinterpret V25 definitions.
        assert!(canonicalize_shared_source(source).is_err());
        let omitted = match canonicalize_shared_source_with_supported_profile(source, None) {
            Ok(_) => panic!("profile is required"),
            Err(error) => error,
        };
        assert!(omitted.starts_with("E_SOURCE_PROFILE_REQUIRED:"));
        let unknown = match canonicalize_shared_source_with_supported_profile(
            source,
            Some("unknown-profile"),
        ) {
            Ok(_) => panic!("unknown profile must fail closed"),
            Err(error) => error,
        };
        assert!(unknown.starts_with("E_SOURCE_PROFILE_UNSUPPORTED:"));
    }

    #[test]
    fn p1a_supported_profile_run_fails_closed_for_profile_and_v25_boundaries() {
        let positive = "매마디:움직씨 := {\n    값:수 := 1.\n}\n";
        let omitted = run_summary_from_supported_source_profile_diagnostic(
            positive,
            "p1a-omitted.ddn",
            Some(1),
            None,
        )
        .expect_err("omitted profile");
        assert_eq!(omitted.code, "E_SOURCE_PROFILE_REQUIRED");

        let unknown = run_summary_from_supported_source_profile_diagnostic(
            positive,
            "p1a-unknown.ddn",
            Some(1),
            Some("unknown-profile"),
        )
        .expect_err("unknown profile");
        assert_eq!(unknown.code, "E_SOURCE_PROFILE_UNSUPPORTED");

        let legacy = run_summary_from_supported_source_profile_diagnostic(
            "매마디:움직씨 = {\n}\n",
            "p1a-legacy.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect_err("legacy definition");
        assert_eq!(legacy.code, "E_V25_DEFINITION_MIGRATION_REQUIRED");

        let recursive = "매마디:움직씨 := {\n    () 매마디하기.\n}\n";
        let first = run_summary_from_supported_source_profile_diagnostic(
            recursive,
            "p1a-recursive.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect_err("direct recursion");
        let repeated = run_summary_from_supported_source_profile_diagnostic(
            recursive,
            "p1a-recursive.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect_err("deterministic direct recursion");
        assert_eq!(first, repeated);
        assert_eq!(first.code, "E_V1_CORE_RECURSION_UNSUPPORTED");
    }

    #[test]
    fn migration_evidence_is_explicit_tagged_and_non_executable() {
        let source = "검사:움직씨 = { 값 ~~> 대상. }";
        let artifact = historical_async_migration_evidence_json(source, "history/input.ddn")
            .expect("migration artifact");
        let value: JsonValue = serde_json::from_str(&artifact).expect("artifact json");
        assert_eq!(value["parser_mode"], "HistoricalMigrationEvidence");
        assert_eq!(value["product_execution_authorized"], false);
        assert_eq!(
            value["artifacts"][0]["historical_identity"],
            "MessageTransfer::HistoricalAsync"
        );
        assert_eq!(
            value["artifacts"][0]["successor_replacement"],
            "값 ~> 대상."
        );
        assert!(value["artifacts"][0]["source_span"]["start"].is_u64());
        assert!(value["artifacts"][0]["source_span"]["end"].is_u64());
        let err = match DdnProgram::from_source_with_mode(
            source,
            "history/input.ddn",
            ParseMode::HistoricalMigrationEvidence,
        ) {
            Ok(_) => panic!("migration mode must not execute"),
            Err(err) => err,
        };
        assert!(err.contains("E_HISTORICAL_ASYNC_MIGRATION_EXECUTION_FORBIDDEN"));
    }

    fn envelope_mode(source: &str) -> String {
        execution_envelope_from_source(source, Some(1))
            .unwrap_or_else(|err| panic!("execution envelope for {source:?}: {err}"))
            .get("execution_mode_decision")
            .and_then(|decision| decision.get("mode"))
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn execution_envelope_carries_shared_parsed_mode_decision() {
        assert_eq!(envelope_mode("1 보여주기."), "oneshot");
        assert_eq!(envelope_mode("(시작)할때 { 1 보여주기. }."), "oneshot");
        assert_eq!(envelope_mode("(매마디)마다 { 1 보여주기. }."), "live");
        assert_eq!(envelope_mode("(4마디)마다 { 1 보여주기. }."), "live");
        let envelope =
            execution_envelope_from_source("1 보여주기.", Some(1)).expect("versioned envelope");
        assert_eq!(
            envelope["resource_snapshot_schema"],
            RESOURCE_SNAPSHOT_SCHEMA_V2
        );
        assert_eq!(envelope["unit_wire_schema"], UNIT_VALUE_WIRE_SCHEMA_V2);
    }

    #[test]
    fn shared_cli_runtime_resources_emit_the_same_unit_wire_v2() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "물질량@mol".to_string(),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2),
                ddonirang_core::Unit::Mole,
            )),
        );
        let resources = serialize_world_resources(world.world());
        assert_eq!(resources["schema"], RESOURCE_SNAPSHOT_SCHEMA_V2);
        assert_eq!(resources["unit_wire_schema"], UNIT_VALUE_WIRE_SCHEMA_V2);
        assert_eq!(
            resources["value_det"]["물질량@mol"]["dimension"]["exponents"],
            json!([0, 0, 0, 0, 0, 0, 0, 0, 1])
        );
    }

    #[test]
    fn public_observation_trace_keeps_ddn_order_source_refs_and_committed_rows() {
        let source = r#"
채비 {
  시간 <- 0.
  각도 <- 15.
  각속도 <- 2.
}.

(매마디)마다 {
  시간 <- 시간 + 1.
  각도 <- 각도 + 각속도.
  (시간: 시간 * 1@s, 각도: 각도 * 1@rad, 각속도: 각속도) 보임.
}.
"#;
        let summary = run_summary_from_source(source, "진자.ddn", Some(2)).expect("run summary");
        let trace = &summary["public_observation_trace"];
        assert_eq!(
            trace["columns"]
                .as_array()
                .expect("columns")
                .iter()
                .map(|column| column["identity"]["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec!["시간", "각도", "각속도"]
        );
        assert_eq!(
            trace["samples"]
                .as_array()
                .expect("samples")
                .iter()
                .map(|sample| sample["committed_madi"].as_u64().unwrap_or(u64::MAX))
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(trace["samples"][1]["cells"][0]["value_text"], json!("2@s"));
        assert_eq!(
            trace["samples"][1]["cells"][1]["value_text"],
            json!("19@rad")
        );
        for column in trace["columns"].as_array().expect("columns") {
            let source_ref = column["identity"]["source_ref"]
                .as_str()
                .expect("source ref");
            assert!(source_ref.starts_with("진자.ddn#bytes="));
        }
        assert_eq!(summary["output_rows"], trace["samples"]);
        assert!(summary["engine_state"]
            .get("observation_manifest")
            .is_none());
        assert!(!trace.to_string().contains("project.obs."));
        assert!(!trace.to_string().contains("output_"));
        let receipt = &summary["work_cost_receipt"];
        assert_eq!(receipt["schema"], "ddonirang.work_cost.v1-candidate");
        assert_eq!(receipt["counters"]["committed_madi"], 2);
        assert_eq!(receipt["counters"]["public_observation_samples"], 6);
        assert_eq!(receipt["counters"]["sealed_input_bytes"], source.len());
        assert_eq!(receipt["counters"]["replay_capsule_bytes"], 0);
        assert_eq!(receipt["slice_limits_enforced"], false);
        assert_eq!(receipt["continuation_present"], false);
        assert_eq!(receipt["rd206_resource_capability"], false);
        assert_eq!(
            receipt["counters"]["result_artifact_bytes"],
            serde_json::to_vec(&summary).expect("summary bytes").len()
        );
        assert!(receipt["counters"]["work_cost"]
            .as_u64()
            .is_some_and(|value| value > 0));
    }

    #[test]
    fn host_preflight_fault_is_distinct_and_returns_no_partial_summary() {
        struct ResetHostPreflightFault;
        impl Drop for ResetHostPreflightFault {
            fn drop(&mut self) {
                set_force_host_preflight_rejection(false);
            }
        }

        let source = "매틱:움직씨 = { (값: 1) 보임. }\n";
        set_force_host_preflight_rejection(true);
        let _reset = ResetHostPreflightFault;
        let error = run_summary_from_source_diagnostic(source, "host_preflight.ddn", Some(1))
            .expect_err("fault injection must reject before execution");
        assert_eq!(
            error.code,
            crate::work_cost::E_STANDARD_RESOURCE_PREFLIGHT_REJECTED_CANDIDATE
        );
        assert_eq!(error.phase, "resource_preflight");
        assert!(error
            .message
            .contains("observation_frames.try_reserve_exact:fault"));
        assert!(error.message.contains("work_cost_receipt="));
    }

    #[test]
    fn result_artifact_boundary_plus_one_returns_no_partial_summary() {
        let source = "매틱:움직씨 = { (값: 1) 보임. }\n";
        let limits = StandardResourceLimitsV1 {
            result_artifact_bytes: 1,
            ..StandardResourceLimitsV1::CANDIDATE
        };
        let error = run_summary_from_source_with_assets_diagnostic(
            source,
            "result_artifact_limit.ddn",
            Some(1),
            None,
            Some(limits),
            None,
            None,
            None,
            0,
        )
        .expect_err("one-byte result limit must reject the whole summary");
        assert_eq!(
            error.code,
            crate::work_cost::E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE
        );
        assert_eq!(error.phase, "resource_commit");
        assert!(error.message.contains("result_artifact_bytes"));
        assert!(error.message.contains("work_cost_receipt="));
    }

    #[test]
    fn bounded_model_run_reaches_shared_result_table_surface() {
        let source = r#"
정밀한방법 <- (방식: #정밀).
계산 = 모형 {
  입력:나눔수@m :: (들임, 내줌).
  결과:나눔수@m :: (그자리, 내줌).
  결과::구함.
  입력 + 1@m =:= 결과.
}.
실행결과 <- (대상=계산, 들임=(입력: 2@m), 방법=정밀한방법, 내줌=(결과, 입력)) 풀기.
(답: 실행결과.결과, 원본: 실행결과.입력) 보임.
"#;
        let summary = run_summary_from_source(source, "app/model.ddn", Some(1))
            .expect("bounded model summary");
        assert_eq!(
            summary["runtime_schema"],
            crate::model_runtime::MODEL_EXECUTION_SCHEMA
        );
        assert_eq!(summary["model_execution"]["status"], "completed");
        assert_eq!(summary["final_row"]["결과"], "3@m");
        assert_eq!(summary["final_row"]["입력"], "2@m");
        assert_eq!(
            summary["public_observation_trace"]["columns"][0]["identity"]["name"],
            "답"
        );
        assert!(
            summary["public_observation_trace"]["columns"][0]["identity"]["source_ref"]
                .as_str()
                .is_some_and(|value| value.starts_with("app/model.ddn#bytes="))
        );
        assert_eq!(
            summary["public_observation_table_projection"]["presentation_state"],
            "ready"
        );
        assert_eq!(summary["ticks_run"], 1);
    }

    #[test]
    fn bounded_model_prepared_state_is_rust_owned_and_empty_until_commit() {
        let source = r#"
정밀한방법 <- (방식: #정밀).
계산 = 모형 {
  입력:나눔수@m :: (들임, 내줌).
  결과:나눔수@m :: (그자리, 내줌).
  결과::구함.
  입력 + 1@m =:= 결과.
}.
실행결과 <- (대상=계산, 들임=(입력: 2@m), 방법=정밀한방법, 내줌=(결과, 입력)) 풀기.
(답: 실행결과.결과, 원본: 실행결과.입력) 보임.
"#;
        let prepared = model_prepared_state_from_source(source, "app/model.ddn")
            .expect("bounded model prepared state");
        let completed = run_summary_from_source(source, "app/model.ddn", Some(1))
            .expect("bounded model completed state");
        assert_eq!(prepared["schema"], ENGINE_RESPONSE_SCHEMA);
        assert_eq!(prepared["tick_id"], 0);
        assert_eq!(
            prepared["state"]["snapshot_v2"]["schema"],
            "ddn.wasm_state_snapshot.v2"
        );
        assert_eq!(
            prepared["state"]["snapshot_v2"]["resources"]["schema"],
            crate::unit_wire::RESOURCE_SNAPSHOT_SCHEMA_V2
        );
        assert_eq!(completed["engine_state"]["schema"], ENGINE_RESPONSE_SCHEMA);
        assert_eq!(
            completed["engine_state"]["state"]["snapshot_v2"]["next_tick_id"],
            1
        );
        assert_eq!(
            completed["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            prepared["state_hash"],
            completed["model_execution"]["pre_state_hash"]
        );
        assert_eq!(
            prepared["public_observation_trace"]["samples"]
                .as_array()
                .map(Vec::len),
            Some(0)
        );
        assert_eq!(
            prepared["model_execution_envelope"]["execution_mode_decision"]["mode"],
            "oneshot"
        );
        assert_eq!(
            prepared["model_execution_envelope"]["backend_descriptor"]["backend_id"],
            "ddn.exact_acyclic_relation.v1"
        );
    }

    #[test]
    fn bounded_model_summary_rejects_non_oneshot_tick_request() {
        let error = run_summary_from_source("값 = 모형 { }.", "model.ddn", Some(2))
            .expect_err("model backend is one-shot");
        assert!(error.contains("E_MODEL_ONESHOT_TICKS_CANDIDATE"));
    }

    #[test]
    fn execution_contract_diagnostic_remains_structured_through_run_summary() {
        let source = crate::self_contained_test_sources::RD206_INNER_EQUALS;
        let error = run_summary_from_source_diagnostic(source, "project/entry.ddn", Some(1))
            .expect_err("inner equals must fail before execution");
        assert_eq!(
            error.code,
            "E_MODEL_EXECUTION_CONTRACT_FIELD_DELIMITER_CANDIDATE"
        );
        assert_eq!(error.source_path, "project/entry.ddn");
        assert!(error.span.is_some_and(|(start, end)| start < end));
        assert_eq!(error.phase, "model_frontdoor_candidate");
        assert!(error.cause.is_some());
        assert_eq!(error.offending_field.as_deref(), Some("결과"));
        assert_eq!(error.expected.as_deref(), Some(":"));
        assert!(error
            .actual
            .as_deref()
            .is_some_and(|value| value.contains('=')));
    }
}
