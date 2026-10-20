use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
pub(crate) mod general_result;
use general_result::{GeneralExecutionCollector,GeneralResultCollectorV1,GeneralEvidenceSourceV1};
#[cfg(windows)]
pub(crate) mod native_source_world;
#[cfg(windows)]
pub use native_source_world::{NativeFirstSourceWorldExecution,NativeFirstSourceWorldPrepared,NativeFirstSourceWorldDurablyPrepared,NativeFirstSourceWorldRetained,NativeGeneralRecordingPrepared};
#[cfg(windows)]
pub use native_source_world::{resume_first_saved_source_world_native,save_continued_source_world_native};
#[cfg(windows)]
pub use native_source_world::{NativeGeneralReplayContentV1,read_current_general_recording_content};
#[cfg(windows)]
pub(crate) use native_source_world::{native_source_browser_snapshot,native_source_current_digest};
#[cfg(windows)]
pub(crate) use native_source_world::{resume_logical_source_world,save_continued_logical_source_world,verify_native_logical_current_receipt};
#[cfg(windows)]
pub(crate) use native_source_world::NativeLogicalFinishSessionV1;

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
    // Saved v25 projects were already parsed through the sealed product
    // frontdoor. Re-running the legacy text preprocessor misreads a normal
    // local name followed by a sentence terminator (e.g. `바탕.`) as a prefix.
    // Only the exact source/path bound to this program may reuse that admission.
    let sealed_project = program.sealed_project_sources();
    if let Some(saved) = sealed_project {
        let prepared = saved.preparation();
        if prepared.source != source || prepared.target_path != source_path {
            return Err(ExecutionDiagnostic {
                code: "E_SOURCE_ADMISSION_CUSTODY",
                message: "실행기에 결속된 프로젝트 원문과 입력 원문이 다릅니다".into(),
                source_path: source_path.into(), span: None, phase: "source_admission",
                cause: None, offending_field: None, expected: None, actual: None,
            });
        }
    }
    let admitted_source = program.has_root_program() || sealed_project.is_some();
    let preprocessed = if admitted_source {
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
    let execution_source = if admitted_source {
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
    let code=["E_TYPE_MISMATCH","E_CHECK_TYPE_MISMATCH","E_CHECK_TUPLE_SELECTION","E_CHECK_UNKNOWN_NAME",
        "E_NUMERIC_COMPARISON_TYPE_MISMATCH","E_NUMERIC_COMPARISON_TYPE_UNRESOLVED"]
        .into_iter().find(|code|message.starts_with(&format!("{code}:")));
    if let Some(code)=code {
        let mut diagnostic=run_summary_diagnostic(code,message.to_owned(),path,"source_type_check");
        if message.contains(")\n") {
            let span=crate::source_analysis::formatted_error_span(source,message);
            diagnostic.span=Some((span.start,span.end));
        } else if let Some((_,rest))=message.split_once("Some(Span { start: ") {
            if let Some((start,rest))=rest.split_once(", end: ") {
                if let Some(end)=rest.split(" }").next() {
                    if let (Ok(start),Ok(end))=(start.parse::<usize>(),end.parse::<usize>()) {
                        if start<=end && end<=source.len() && source.is_char_boundary(start) && source.is_char_boundary(end) {
                            diagnostic.span=Some((start,end));
                        }
                    }
                }
            }
        }
        return Some(diagnostic);
    }
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

/// D1 content uses the existing world owner's exact bytes and exclusions.
pub(crate) fn result_world_canonical_bytes(world:&ddonirang_core::platform::NuriWorld)->Vec<u8> {
    world.encode_canonical_with_filter(DEFAULT_VIEW_PREFIXES)
}

/// Resume the existing RD-686 source-bound artifact, not an input recording.
pub(crate) fn fresh_authored_logical_run(
    authored:&str,binding:crate::logical_input_tape::LogicalInputBinding,
)->Result<crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1,String> {
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
    crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1::fresh(
        &owned,binding,crate::ddn_runtime::default_age_target())
        .map_err(|failure|format!("E_PROJECT_RUNTIME_LOGICAL: 프로젝트 실행 결속 실패 ({failure:?})"))
}

#[cfg(not(target_arch="wasm32"))]
pub struct NativeLogicalCheckpoint {
    pub saved_artifact:String,
    pub committed_receipt:Vec<u8>,
    pub transaction_digest:[u8;32],
    pub summary:JsonValue,
}

/// Explicit fresh authored execution and Native checkpoint. This never labels
/// a restored execution as fresh, provisions an existing store implicitly,
/// or treats project CAS history as the run's checkpoint history.
#[cfg(not(target_arch="wasm32"))]
pub fn record_authored_logical_project_native(
    authored:&str,input:&crate::logical_input_tape::LogicalRecordRequest,
    parent:&[u8],revision:&[u8],storage_root:&std::path::Path,
    initialize_store:bool,expected_project_receipt:Option<&[u8]>,
)->Result<NativeLogicalCheckpoint,String> {
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
    // Source admission issues an owner once. Readmission would create a
    // different source owner even when the authored text is byte-identical.
    let mut run=crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1::fresh(
        &owned,input.binding.clone(),crate::ddn_runtime::default_age_target())
        .map_err(|failure|format!("E_PROJECT_RUNTIME_NATIVE: 저장·복원 확인 실패 ({failure:?})"))?;
    for snapshot in &input.snapshots {run.step(snapshot)?;}
    checkpoint_logical_project_native(&mut run,&owned,parent,revision,storage_root,initialize_store,expected_project_receipt)
}

/// Ordinary authored execution uses its own actual InputSnapshot supplier and
/// XOR-tick boundary. Never substitute a logical Tape or fixed-seed session.
#[cfg(not(target_arch="wasm32"))]
pub fn record_authored_general_project_native(
    authored:&str,madi:u32,base_seed:u64,parent:&[u8],revision:&[u8],
    storage_root:&std::path::Path,initialize_store:bool,expected_project_receipt:Option<&[u8]>,
)->Result<NativeLogicalCheckpoint,String> {
    use crate::unit_wire::project_runtime_attachment as attachment;
    let run=run_authored_general_input_prefix(authored,madi,base_seed,None,None)?;
    let saved_artifact=run.export_saved_artifact(parent,revision)?;
    let checked=attachment::check_bytes(saved_artifact.as_bytes())?;
    let context=attachment::check_saved_revision_context(&checked,run.source_owner(),parent,revision)?;
    let raw=run.encode_checkpoint_request(&context,parent,revision)?;
    let (receipt,digest)=persist_general_checkpoint_request(&raw,context.pesi().as_raw(),parent,
        storage_root,initialize_store,expected_project_receipt)?;
    let summary=crate::unit_wire::SnapshotJsonOwner::clone_from_value(run.summary()).into_parent_value();
    Ok(NativeLogicalCheckpoint {saved_artifact,committed_receipt:receipt,transaction_digest:digest,summary})
}

#[cfg(not(target_arch="wasm32"))]
fn persist_general_checkpoint_request(
    raw:&[u8],pesi:&[u8;32],parent:&[u8],storage_root:&std::path::Path,
    initialize_store:bool,expected_project_receipt:Option<&[u8]>,
)->Result<(Vec<u8>,[u8;32]),String> {
    use crate::snapshot_v2_durable_transaction::{native::{SqliteAdapterV1,StoredPayloadV1},commit_native_evidence};
    use crate::snapshot_v2_host_bridge as bridge;
    let reject=|failure|format!("E_PROJECT_RUNTIME_NATIVE: 저장·복원 확인 실패 ({failure:?})");
    let request=bridge::check_resumable_request_candidate_v2(raw,pesi).map_err(reject)?;
    if request.request_kind!=1 {return Err("E_PROJECT_RUNTIME_NATIVE: 저장 요청 종류가 다릅니다".into());}
    let expected_head=expected_project_receipt.map(|receipt|
        bridge::check_committed_receipt_payload_v1(receipt).and_then(|checked|checked.native_head()))
        .transpose().map_err(reject)?;
    if expected_head.as_ref().map(|head|head.revision.as_slice()).unwrap_or(&[])!=parent
        || expected_head.as_ref().is_some_and(|head|head.project!=request.binding.project) {
        return Err(reject(crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1::CustodyConflict));
    }
    // Everything above is staging/admission. Only now discover/open storage.
    let adapter=if initialize_store {SqliteAdapterV1::provision(storage_root)}
        else {SqliteAdapterV1::open_admitted(storage_root)}.map_err(reject)?;
    let profile=adapter.capability().map_err(reject)?;
    let receipt=bridge::encode_committed_receipt_candidate(&request.binding,profile.as_bytes()).map_err(reject)?;
    let digest=request.binding.transaction_digest().map_err(reject)?;
    let payload=StoredPayloadV1 {digest,binding:request.binding.clone(),
        snapshot:request.snapshot_bytes.to_vec(),envelope:request.revision_bytes.to_vec()};
    let evidence=commit_native_evidence(&adapter,&payload,expected_head.as_ref(),profile.as_bytes()).map_err(|failure|
        format!("E_PROJECT_RUNTIME_NATIVE: 저장·복원 확인 실패 (state={:?}, cause={:?})",failure.state,failure.cause))?;
    evidence.check_committed_receipt_evidence(&receipt,profile.as_bytes()).map_err(reject)?;
    Ok((receipt,digest))
}

/// Native ordinary restore validates the actual stored committed chain and
/// stages the original saved-source construction before any next-madi output.
/// This entry has no logical-input fallback and cannot issue a fresh receipt.
#[cfg(not(target_arch="wasm32"))]
pub fn resume_saved_general_project_artifact_native(
    artifact:&str,expected_parent:&[u8],revision:&[u8],storage_root:&std::path::Path,
    transaction_digest:&[u8;32],madi:u32,
)->Result<JsonValue,String> {
    let verified=restore_general_native_checkpoint(artifact,expected_parent,revision,storage_root,transaction_digest)?;
    let (result,_,_)=resume_saved_project_artifact_from_runner(artifact,expected_parent,revision,madi,
        Some((verified.execution.runner,verified.age)))?;
    Ok(result)
}

#[cfg(not(target_arch="wasm32"))]
pub(crate) struct VerifiedGeneralNativeRun {
    owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    execution:GeneralCompletedExecution,
    rows:Vec<crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow>,
    base:u64,age:AgeTarget,receipts:Vec<Vec<u8>>,
}

#[cfg(not(target_arch="wasm32"))]
impl VerifiedGeneralNativeRun {
    pub(crate) fn checkpoint_request(&self,context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        parent:&[u8],revision:&[u8])->Result<Vec<u8>,String> {
        use crate::unit_wire::resumable_complete_state as complete;
        use sha2::Digest;
        let reject=|failure|format!("E_PROJECT_RUNTIME_NATIVE: 이어 저장 요청 결속 실패 ({failure:?})");
        let actual=if context.world_frame().is_some(){self.execution.export_continued_artifact_native_complete(&self.owned,parent,revision)?}
            else{self.execution.export_continued_artifact(&self.owned,parent,revision)?};
        let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(actual.as_bytes())?;
        let checked=if let Some(frame)=context.world_frame(){crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(&bytes,&self.owned,parent,revision,frame)?}
            else{crate::unit_wire::project_runtime_attachment::check_saved_revision_context(&bytes,&self.owned,parent,revision)?};
        if checked.runtime_state()!=context.runtime_state() || checked.resource_snapshot()!=context.resource_snapshot()
            || checked.revision_envelope()!=context.revision_envelope() {
            return Err("E_PROJECT_RUNTIME_NATIVE: 이어 저장의 현재 실행 상태가 다릅니다".into());
        }
        let boundary=complete::encode_general_boundary_supplies(&self.owned,&self.execution.runner,&self.rows,self.base,self.execution.next_tick).map_err(reject)?;
        let supplies=[context.revision_envelope().to_vec(),complete::encode_source_capture_supply(self.owned.sources()).map_err(reject)?,
            complete::encode_constituent_supply(&self.owned).map_err(reject)?,complete::encode_execution_contract_supply(&self.owned,&self.execution.runner).map_err(reject)?,
            complete::encode_general_processed_inputs(&self.rows,0,self.base).map_err(reject)?,boundary[0].clone(),boundary[1].clone(),boundary[2].clone(),
            complete::encode_general_predecessor(&self.receipts).map_err(reject)?,
            complete::encode_general_source_descriptor_with_receipts(&self.owned,&self.execution.runner,self.age,self.base,self.execution.next_tick,&self.receipts).map_err(reject)?];
        let pesi=*context.pesi().as_raw();
        let world_bytes=if context.world_frame().is_some(){complete::encode_owned_world_candidate_with_source(self.execution.world.world(),&pesi,self.execution.next_tick,&self.owned)}
            else{complete::encode_resource_only_world_candidate(self.execution.world.world(),&pesi,self.execution.next_tick)}.map_err(reject)?;
        let world=crate::unit_wire::check_complete_state_frame_v1(&world_bytes,&pesi).map_err(reject)?;
        let layout=crate::unit_wire::ResumableCompleteStateLayoutV2 {world,supplies:std::array::from_fn(|i|supplies[i].as_slice())};
        complete::rebuild_general_complete_state_source_candidate(&layout,context).map_err(reject)?;
        let definition=self.owned.definition_identity().as_bytes();
        complete::check_boundary_supply_v1(&layout,complete::GENERAL_XOR_RNG_PROFILE,definition,definition,&self.execution.runner.work_cost_receipt()).map_err(reject)?;
        let catalog=if context.world_frame().is_some(){self.owned.registration().map(|r|r.saved_nominal_catalog(&self.owned)).transpose()?}else{None};
        let snapshot=if context.world_frame().is_some(){layout.encode_canonical_with_catalog(catalog.as_ref())}else{layout.encode_canonical()}.map_err(reject)?;
        let envelope:JsonValue=serde_json::from_slice(context.revision_envelope()).map_err(|_|"E_PROJECT_RUNTIME_REVISION")?;
        let integrity=envelope["revision_integrity_sha256"].as_str().ok_or("E_PROJECT_RUNTIME_REVISION")?;
        let binding=crate::snapshot_v2_durable_transaction::TransactionBindingV1 {
            project:hex::decode(self.owned.project().authored_identity.logical_dependency_id.as_str()).map_err(|_|"E_PROJECT_RUNTIME_IDENTITY")?,
            expected_parent:parent.to_vec(),new_revision:revision.to_vec(),execution_profile:self.owned.project().source_profile.as_bytes().to_vec(),
            pesi,snapshot_sha256:sha2::Sha256::digest(&snapshot).into(),snapshot_length:snapshot.len() as u64,
            revision_integrity:hex::decode(integrity).map_err(|_|"E_PROJECT_RUNTIME_REVISION")?.try_into().map_err(|_|"E_PROJECT_RUNTIME_REVISION")?,
        };
        let request=crate::snapshot_v2_host_bridge::ResumableRequestCandidateV2 {binding,layout,revision_bytes:context.revision_envelope(),
            request_kind:1,snapshot_bytes:&snapshot};
        if context.world_frame().is_some(){request.encode_canonical_with_catalog(catalog.as_ref()).map_err(reject)}else{request.encode_canonical().map_err(reject)}
    }
}

#[cfg(not(target_arch="wasm32"))]
pub fn resume_and_save_general_project_native(
    artifact:&str,expected_parent:&[u8],revision:&[u8],storage_root:&std::path::Path,
    transaction_digest:&[u8;32],madi:u32,next_revision:&[u8],expected_project_receipt:&[u8],
)->Result<NativeLogicalCheckpoint,String> {
    use crate::unit_wire::project_runtime_attachment as attachment;
    let mut run=restore_general_native_checkpoint(artifact,expected_parent,revision,storage_root,transaction_digest)?;
    if run.receipts.last().map(Vec::as_slice)!=Some(expected_project_receipt) {
        return Err("E_PROJECT_RUNTIME_NATIVE: 이어 저장의 이전 영수증이 다릅니다".into());
    }
    let (summary,execution,rows)=resume_saved_project_artifact_from_runner(artifact,expected_parent,revision,madi,
        Some((run.execution.runner,run.age)))?;
    run.execution=execution;run.rows.extend(rows);
    let saved_artifact=run.execution.export_continued_artifact(&run.owned,revision,next_revision)?;
    let checked=attachment::check_bytes(saved_artifact.as_bytes())?;
    let context=attachment::check_saved_revision_context(&checked,&run.owned,revision,next_revision)?;
    let raw=run.checkpoint_request(&context,revision,next_revision)?;
    let (committed_receipt,transaction_digest)=persist_general_checkpoint_request(&raw,context.pesi().as_raw(),revision,
        storage_root,false,Some(expected_project_receipt))?;
    Ok(NativeLogicalCheckpoint {saved_artifact,committed_receipt,transaction_digest,summary})
}

#[cfg(not(target_arch="wasm32"))]
#[derive(Clone, Copy)]
pub enum NativeRootGrantOperation<'a> { Register, Check(&'a str), Revoke(&'a str) }

/// Product source-save/readback surface, not an execution summary or receipt.
/// It coordinates strict source ownership, leased registration, the frozen
/// common candidate and actual SQLite readback; it never starts a world.
#[cfg(not(target_arch="wasm32"))]
pub struct NativeSourceRevisionReadback {
    pub project_identity:Vec<u8>,pub revision_identity:Vec<u8>,pub revision_integrity:[u8;32],
    pub pesi:[u8;32],pub project_payload_utf8:String,pub revision_envelope:Vec<u8>,
    /// Locator only; a caller must reopen/confirm/consume the actual protected
    /// owner. Source material and this string never issue a first-run permit.
    pub grant_locator:Option<String>,
}
#[cfg(not(target_arch="wasm32"))]
fn source_readback_projection(source:&crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision)
    ->Result<NativeSourceRevisionReadback,String> {
    Ok(NativeSourceRevisionReadback{project_identity:source.project().to_vec(),revision_identity:source.revision().to_vec(),
        revision_integrity:*source.integrity(),pesi:*source.pesi(),
        project_payload_utf8:std::str::from_utf8(source.payload()).map_err(|_|"E_SOURCE_REVISION_UTF8")?.into(),
        revision_envelope:source.envelope().to_vec(),grant_locator:None})
}
#[cfg(not(target_arch="wasm32"))]
pub fn save_initial_authored_source_native(artifact:&str,revision:&[u8],storage_root:&std::path::Path)
    ->Result<NativeSourceRevisionReadback,String> {
    use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
    let source=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(artifact.as_bytes())?;
    let mut adapter=SqliteAdapterV1::provision_source_revision(storage_root)
        .map_err(|e|format!("E_SOURCE_SAVE_BEFORE_COMMIT: {e:?}"))?;
    let draft=crate::authored_project::registration::prepare_native_precanon_authoring(&mut adapter,&source,None,None)?.finish()?;
    let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(source,draft)?;
    let candidate=crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision::prepare(&owned,&[],revision)?;
    #[cfg(windows)]
    {
        let grant=adapter.register_source_root_grant(&candidate)
            .map_err(|error|format!("E_SOURCE_GRANT_REGISTRATION: phase={} cause={:?}",error.phase,error.cause))?;
        let mut result=source_readback_projection(&candidate)?;
        result.grant_locator=Some(grant.locator());
        return Ok(result);
    }
    #[cfg(not(windows))]
    {
    use crate::snapshot_v2_durable_transaction::native::source_revision::SourceSaveFailure;
    let saved=adapter.save_initial_source_revision(&candidate).map_err(|e|match e {
        SourceSaveFailure::BeforeCommit(cause)=>format!("E_SOURCE_SAVE_BEFORE_COMMIT: {cause:?}"),
        SourceSaveFailure::AfterCommitUnconfirmed(cause)=>format!("E_SOURCE_SAVE_UNCONFIRMED: 저장 뒤 확인되지 않았습니다. 같은 판을 재읽기해야 합니다 ({cause:?})"),
    })?;
    source_readback_projection(saved.source())
    }
}

/// A live Native owner, not bytes/JSON/receipt. Beginning this session durably
/// consumes first use before any VM/world creation. Dropping it does not undo
/// Started, even when no world has run. The VM/custody stage must take this
/// non-Clone session rather than reconstruct permission from saved metadata.
#[cfg(windows)]
pub struct NativeFirstSourceWorldSession {
    adapter:crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    permit:crate::snapshot_v2_durable_transaction::native::FirstSourceWorldPermitV2,
}
#[cfg(windows)]
impl NativeFirstSourceWorldSession {
    pub fn source_readback(&self)->Result<NativeSourceRevisionReadback,String>{
        self.permit.verify_before_world(&self.adapter).map_err(|cause|format!("E_SOURCE_FIRST_USE_CURRENT: {cause:?}"))?;
        let mut result=source_readback_projection(self.permit.source())?;
        result.grant_locator=Some(self.permit.locator());Ok(result)
    }
}
#[cfg(windows)]
pub fn begin_first_authored_source_native(storage_root:&std::path::Path,project:&[u8],locator:&str,
    expected_revision:&[u8],expected_integrity:&[u8;32])
    ->Result<NativeFirstSourceWorldSession,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(storage_root)
        .map_err(|cause|format!("E_SOURCE_REVISION_OPEN: {cause:?}"))?;
    let grant=adapter.confirm_unused_source_root_grant(locator,project)
        .map_err(|error|format!("E_SOURCE_FIRST_USE_CONFIRM: phase={} cause={:?}",error.phase,error.cause))?;
    let permit=grant.consume_first_source_world(&adapter,project,expected_revision,expected_integrity)
        .map_err(|error|format!("E_SOURCE_FIRST_USE_CONSUME: phase={} cause={:?}",error.phase,error.cause))?;
    Ok(NativeFirstSourceWorldSession{adapter,permit})
}
#[cfg(windows)]
pub fn update_authored_source_native(artifact:&str,revision:&[u8],storage_root:&std::path::Path,
    project:&[u8],locator:&str,expected_parent:&[u8],expected_integrity:&[u8;32])->Result<NativeSourceRevisionReadback,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(storage_root)
        .map_err(|cause|format!("E_SOURCE_REVISION_OPEN: {cause:?}"))?;
    let grant=adapter.confirm_unused_source_root_grant(locator,project)
        .map_err(|error|format!("E_SOURCE_SUCCESSOR_CONFIRM: phase={} cause={:?}",error.phase,error.cause))?;
    use crate::snapshot_v2_durable_transaction::native::source_revision::SourceEditV2;
    let candidate=match grant.prepare_source_successor(&adapter,artifact,revision,expected_parent,expected_integrity)? {
        SourceEditV2::Unchanged(source)=>{
            let mut result=source_readback_projection(&source)?;result.grant_locator=Some(grant.locator());return Ok(result);
        },
        SourceEditV2::Successor(source)=>source,
    };
    let saved=adapter.save_source_successor(&candidate,&grant)
        .map_err(|error|format!("E_SOURCE_SUCCESSOR_SAVE: {error:?}"))?;
    let mut result=source_readback_projection(saved.source())?;
    result.grant_locator=Some(grant.locator());Ok(result)
}

/// Physical selected-head data, never a root/source/Run capability. Native
/// admission rechecks this against the actual protected root and complete DB.
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeAuthoringSelection {
    pub project:Vec<u8>,pub revision:Vec<u8>,pub integrity:[u8;32],pub key:[u8;32],pub kind:u8,
}
#[cfg(windows)]
pub use native_source_world::{begin_new_authored_source_native,NativeRestartSourceWorldSession,NativeRestartSourceWorldExecution,NativeRestartSourceWorldPrepared};
#[cfg(windows)]
pub(crate) fn native_authoring_selection_from_adapter(adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    project:&[u8],locator:&str,artifact:&str)->Result<NativeAuthoringSelection,String>{
    let head=adapter.selected_authoring_head(project).map_err(|e|format!("E_SOURCE_AUTHORING_HEAD: {e:?}"))?;
    let base=adapter.admit_authoring_base(locator,&head,artifact).map_err(|e|format!("E_SOURCE_AUTHORING_ADMISSION: {e:?}"))?;
    let h=base.head();Ok(NativeAuthoringSelection{project:h.project.clone(),revision:h.revision.clone(),integrity:h.integrity,key:h.key,kind:h.kind})
}
#[cfg(windows)]
pub fn native_authoring_selection(storage_root:&std::path::Path,project:&[u8],locator:&str,artifact:&str)->Result<NativeAuthoringSelection,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(storage_root).map_err(|e|format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    native_authoring_selection_from_adapter(&mut adapter,project,locator,artifact)
}
#[cfg(windows)]
pub fn save_changed_authored_source_native(artifact:&str,original_artifact:&str,revision:&[u8],storage_root:&std::path::Path,
    locator:&str,selection:&NativeAuthoringSelection)->Result<NativeSourceRevisionReadback,String>{
    use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,AuthoringHeadV1,source_revision::SourceSaveFailure};
    if selection.project.len()!=32||selection.revision.is_empty()||selection.revision.len()>4096||selection.kind>1{return Err("E_SOURCE_AUTHORING_SELECTION".into());}
    let mut adapter=SqliteAdapterV1::open_source_revision(storage_root).map_err(|e|format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    save_changed_authored_source_with_adapter(&mut adapter,artifact,original_artifact,revision,locator,selection)
}
#[cfg(windows)]
pub(crate) fn save_changed_authored_source_with_adapter(adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    artifact:&str,original_artifact:&str,revision:&[u8],locator:&str,selection:&NativeAuthoringSelection)->Result<NativeSourceRevisionReadback,String>{
    use crate::snapshot_v2_durable_transaction::native::{AuthoringHeadV1,source_revision::SourceSaveFailure};
    if selection.project.len()!=32||selection.revision.is_empty()||selection.revision.len()>4096||selection.kind>1{return Err("E_SOURCE_AUTHORING_SELECTION".into());}
    let head=AuthoringHeadV1{project:selection.project.clone(),revision:selection.revision.clone(),integrity:selection.integrity,key:selection.key,kind:selection.kind};
    let base=adapter.admit_authoring_base(locator,&head,original_artifact).map_err(|e|format!("E_SOURCE_AUTHORING_ADMISSION: {e:?}"))?;
    let prepared=base.prepare(&adapter,artifact,revision)?;
    let saved=adapter.save_authoring_source(prepared).map_err(|e|match e{SourceSaveFailure::BeforeCommit(cause)=>format!("E_SOURCE_AUTHORING_BEFORE_COMMIT: {cause:?}"),
        SourceSaveFailure::AfterCommitUnconfirmed(cause)=>format!("E_SOURCE_AUTHORING_RECOVERY_REQUIRED: {cause:?}")})?;
    let mut result=source_readback_projection(saved.source())?;result.grant_locator=Some(saved.locator());Ok(result)
}
#[cfg(windows)]
pub(crate) fn current_authoring_source_with_adapter(adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    project:&[u8],locator:&str)->Result<Option<(NativeSourceRevisionReadback,NativeAuthoringSelection)>,String>{
    let Some(base)=adapter.current_authoring_source_if_ready(locator,project).map_err(|e|format!("E_SOURCE_AUTHORING_CURRENT: {e:?}"))? else{return Ok(None);};
    let mut source=source_readback_projection(base.source())?;source.grant_locator=Some(locator.into());let h=base.head();
    let selection=NativeAuthoringSelection{project:h.project.clone(),revision:h.revision.clone(),integrity:h.integrity,key:h.key,kind:h.kind};Ok(Some((source,selection)))
}
#[cfg(windows)]
pub fn current_authoring_source_native(root:&std::path::Path,project:&[u8],locator:&str)->Result<Option<(NativeSourceRevisionReadback,NativeAuthoringSelection)>,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root).map_err(|e|format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    current_authoring_source_with_adapter(&mut adapter,project,locator)
}
#[cfg(windows)]
pub fn read_changed_authored_source_native(storage_root:&std::path::Path,project:&[u8],locator:&str,artifact:&str)->Result<NativeSourceRevisionReadback,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(storage_root).map_err(|e|format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    let head=adapter.selected_authoring_head(project).map_err(|e|format!("E_SOURCE_AUTHORING_HEAD: {e:?}"))?;
    if head.kind!=0{return Err("E_SOURCE_AUTHORING_NOT_READY: 현재 머리는 원문 저장 판이 아닙니다".into());}
    let base=adapter.admit_authoring_base(locator,&head,artifact).map_err(|e|format!("E_SOURCE_AUTHORING_ADMISSION: {e:?}"))?;
    let mut result=source_readback_projection(base.source())?;result.grant_locator=Some(locator.into());Ok(result)
}
#[cfg(windows)]
pub fn read_authored_source_native(storage_root:&std::path::Path,project:&[u8],locator:&str)
    ->Result<NativeSourceRevisionReadback,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(storage_root)
        .map_err(|cause|format!("E_SOURCE_REVISION_OPEN: {cause:?}"))?;
    let grant=adapter.confirm_unused_source_root_grant(locator,project)
        .map_err(|error|format!("E_SOURCE_READ_CONFIRM: phase={} cause={:?}",error.phase,error.cause))?;
    let source=grant.current_source(&adapter,project).map_err(|cause|format!("E_SOURCE_READ_CURRENT: {cause:?}"))?;
    let mut result=source_readback_projection(&source)?;result.grant_locator=Some(grant.locator());Ok(result)
}
#[cfg(not(target_arch="wasm32"))]
pub fn read_initial_authored_source_native(storage_root:&std::path::Path,project:&[u8],revision:&[u8],integrity:&[u8;32])
    ->Result<NativeSourceRevisionReadback,String> {
    let adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(storage_root)
        .map_err(|e|format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    let saved=adapter.load_current_source_revision(project,revision,integrity)
        .map_err(|e|format!("E_SOURCE_REVISION_READBACK: {e:?}"))?;
    source_readback_projection(saved.source())
}

/// Private host transport for an explicitly enrolled INITIAL custody store.
/// The request/token are recovery inputs, not capabilities or new wire formats.
#[cfg(not(target_arch="wasm32"))]
pub struct NativeCustodyCheckpoint {
    pub checkpoint:NativeLogicalCheckpoint,
    pub grant_locator:String,
    pub recovery_request:Vec<u8>,
    pub recovery_token:[u8;33],
}

#[cfg(windows)]
struct NativeCustodyCommitParts {
    receipt:Vec<u8>,digest:[u8;32],locator:String,request:Vec<u8>,token:[u8;33],
}

#[cfg(windows)]
impl NativeCustodyCommitParts {
    fn finish(self,saved_artifact:String,summary:JsonValue)->NativeCustodyCheckpoint {
        NativeCustodyCheckpoint{checkpoint:NativeLogicalCheckpoint{saved_artifact,summary,committed_receipt:self.receipt,transaction_digest:self.digest},
            grant_locator:self.locator,recovery_request:self.request,recovery_token:self.token}
    }
}

#[cfg(windows)]
fn persist_initial_custody_actual(
    actual:crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1,
    source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    saved_artifact:&str,storage_root:&std::path::Path,recovery_directory:Option<&std::path::Path>,
)->Result<NativeCustodyCommitParts,String> {
    use crate::snapshot_v2_durable_transaction::{self as durable,native::SqliteAdapterV1};
    let reject=|failure|format!("E_NATIVE_CUSTODY: 저장 결속 실패 ({failure:?})");
    let recovery_request=actual.raw().to_vec();
    let digest=actual.payload().digest;
    let mut recovery_token=[0u8;33];recovery_token[0]=1;recovery_token[1..].copy_from_slice(&digest);
    let mut adapter=SqliteAdapterV1::provision_recovery_custody(storage_root).map_err(reject)?;
    let grant=adapter.register_saved_root_grant(source).map_err(reject)?;
    let grant_locator=grant.locator();
    if let Some(directory)=recovery_directory {
        export_native_custody_preparation(directory,storage_root,saved_artifact,&recovery_request,&recovery_token,&grant_locator)?;
    }
    let start=grant.start_initial_attempt(&adapter,source,actual).map_err(reject)?;
    let actual=durable::run_initial_native_owner_with_custody(&adapter,source,start)
        .map_err(|failure|format!("E_NATIVE_CUSTODY: 저장 확인 실패 (state={:?}, cause={:?})",failure.state,failure.cause))?;
    let retained=actual.retain(&adapter,source)
        .map_err(|failure|format!("E_NATIVE_CUSTODY: 복구 기록 확인 실패 ({})",failure.diagnostic()))?;
    if let Some(failure)=retained.owner().failure() {
        return Err(format!("E_NATIVE_CUSTODY: 저장이 완료되지 않았습니다 (state={:?}, cause={:?})",failure.state,failure.cause));
    }
    let profile=adapter.capability().map_err(reject)?;
    let binding=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2(&recovery_request,source.pesi().as_raw()).map_err(reject)?.binding;
    let receipt=crate::snapshot_v2_host_bridge::encode_committed_receipt_candidate(&binding,profile.as_bytes()).map_err(reject)?;
    retained.owner().evidence().check_committed_receipt_evidence(&receipt,profile.as_bytes()).map_err(reject)?;
    #[cfg(feature="beta2-crash-probe")]
    durable::native::crash_probe::checkpoint("after_receipt");
    Ok(NativeCustodyCommitParts{receipt,digest,locator:grant_locator,request:recovery_request,token:recovery_token})
}

#[cfg(not(target_arch="wasm32"))]
pub fn record_authored_general_project_custody_native(
    authored:&str,madi:u32,base_seed:u64,revision:&[u8],storage_root:&std::path::Path,
    recovery_directory:Option<&std::path::Path>,
)->Result<NativeCustodyCheckpoint,String> {
    #[cfg(not(windows))]
    {let _=(authored,madi,base_seed,revision,storage_root,recovery_directory);Err("E_NATIVE_CUSTODY_UNSUPPORTED: 현재 복구 기록 저장은 Windows local NTFS에서만 지원합니다".into())}
    #[cfg(windows)]
    {
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1;
        let run=run_authored_general_input_prefix(authored,madi,base_seed,None,None)?;
        let saved_artifact=run.export_saved_artifact(&[],revision)?;
        let bytes=attachment::check_bytes(saved_artifact.as_bytes())?;
        let source=attachment::check_saved_revision_context(&bytes,run.source_owner(),&[],revision)?;
        let actual=ActualPreparedRequestV1::from_general(&run,&source,&[],revision)?;
        let committed=persist_initial_custody_actual(actual,&source,&saved_artifact,storage_root,recovery_directory)?;
        let summary=crate::unit_wire::SnapshotJsonOwner::clone_from_value(run.summary()).into_parent_value();
        Ok(committed.finish(saved_artifact,summary))
    }
}

#[cfg(not(target_arch="wasm32"))]
pub fn record_authored_logical_project_custody_native(
    authored:&str,input:&crate::logical_input_tape::LogicalRecordRequest,revision:&[u8],storage_root:&std::path::Path,
    recovery_directory:Option<&std::path::Path>,
)->Result<NativeCustodyCheckpoint,String> {
    #[cfg(not(windows))]
    {let _=(authored,input,revision,storage_root,recovery_directory);Err("E_NATIVE_CUSTODY_UNSUPPORTED: 현재 복구 기록 저장은 Windows local NTFS에서만 지원합니다".into())}
    #[cfg(windows)]
    {
        use crate::unit_wire::{project_runtime_attachment as attachment,resumable_complete_state::CheckpointedLogicalRunV1};
        let reject=|failure|format!("E_NATIVE_CUSTODY: 논리 실행 결속 실패 ({failure:?})");
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
        let mut run=CheckpointedLogicalRunV1::fresh(&owned,input.binding.clone(),crate::ddn_runtime::default_age_target()).map_err(reject)?;
        for snapshot in &input.snapshots {run.step(snapshot)?;}
        let saved_artifact=run.export_saved_artifact(&[],revision).map_err(reject)?;
        let bytes=attachment::check_bytes(saved_artifact.as_bytes())?;
        let source=attachment::check_saved_revision_context(&bytes,&owned,&[],revision)?;
        let actual=crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1::from_logical(&run,&source,&[],revision).map_err(reject)?;
        let committed=persist_initial_custody_actual(actual,&source,&saved_artifact,storage_root,recovery_directory)?;
        Ok(committed.finish(saved_artifact,run.session().summary()))
    }
}

#[cfg(windows)]
fn export_native_custody_preparation(directory:&std::path::Path,storage_root:&std::path::Path,
    artifact:&str,request:&[u8],token:&[u8;33],locator:&str,
)->Result<(),String> {
    use std::io::Write;
    let reject=|error|format!("E_NATIVE_CUSTODY_PREPARATION: 복구 준비 자료를 보관할 수 없습니다 ({error})");
    if directory.exists() {return Err(reject("새 폴더를 지정해야 합니다".to_string()));}
    let parent=directory.parent().filter(|path|!path.as_os_str().is_empty()).unwrap_or_else(||std::path::Path::new("."));
    let parent=std::fs::canonicalize(parent).map_err(|error|reject(error.to_string()))?;
    let root=std::fs::canonicalize(storage_root).map_err(|error|reject(error.to_string()))?;
    let target=parent.join(directory.file_name().ok_or_else(||reject("폴더 이름이 없습니다".into()))?);
    if target.starts_with(&root) || root.starts_with(&target) {return Err(reject("저장소와 분리된 새 폴더가 필요합니다".into()));}
    std::fs::create_dir(&target).map_err(|error|reject(error.to_string()))?;
    let token_text=hex::encode(token);
    let note="이 폴더는 저장 전 준비 자료입니다. 존재한다고 저장 성공이 아닙니다.\nsaved.json·request.bin·token.hex·grant.txt를 명시적으로 지정해 같은 저장소에서 확인하세요.\n일부 파일 쓰기가 실패하면 이 폴더를 완료된 준비 자료로 사용하지 마세요.\n";
    for (name,bytes) in [("saved.json",artifact.as_bytes()),("request.bin",request),("token.hex",token_text.as_bytes()),
        ("grant.txt",locator.as_bytes()),("안내.txt",note.as_bytes())] {
        let path=target.join(name);
        let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|error|reject(error.to_string()))?;
        file.write_all(bytes).and_then(|_|file.sync_all()).map_err(|error|reject(error.to_string()))?;
        drop(file);
        if std::fs::read(&path).map_err(|error|reject(error.to_string()))?!=bytes {return Err(reject("보관 뒤 다시 읽은 값이 다릅니다".into()));}
    }
    Ok(())
}

#[cfg(windows)]
fn admit_initial_custody_for_restore(
    adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    locator:&str,raw_request:&[u8],token:&[u8;33],
)->Result<crate::snapshot_v2_durable_transaction::ReadmittedCommittedV1,String> {
    use crate::snapshot_v2_durable_transaction::{self as durable,SnapshotV2TransactionStateV1 as State};
    let reject=|failure|format!("E_NATIVE_CUSTODY: 복구 확인 실패 ({failure:?})");
    let grant=adapter.confirm_saved_root_grant(locator,source).map_err(reject)?;
    let prior=grant.load_initial_attempt(adapter,source,raw_request,token).map_err(reject)?;
    let prior=if prior.historical_state()==State::RecoveryRequired {
        let start=prior.start_recovery(adapter,source).map_err(reject)?;
        let actual=durable::recover_initial_native_owner_with_custody(adapter,source,start)
            .map_err(|failure|format!("E_NATIVE_CUSTODY: 복구 실패 (state={:?}, cause={:?})",failure.state,failure.cause))?;
        let retained=actual.retain(adapter,source)
            .map_err(|failure|format!("E_NATIVE_CUSTODY: 복구 기록 확인 실패 ({})",failure.diagnostic()))?;
        if let Some(failure)=retained.owner().failure() {
            return Err(format!("E_NATIVE_CUSTODY: 이어 실행할 수 없습니다 (state={:?}, original={:?}, attempt={:?})",
                failure.state,failure.cause,retained.attempt_failure()));
        }
        let grant=adapter.confirm_saved_root_grant(locator,source).map_err(reject)?;
        grant.load_initial_attempt(adapter,source,raw_request,token).map_err(reject)?
    }else{prior};
    durable::readmit_initial_committed_native(adapter,source,prior)
        .map_err(|blocked|format!("E_NATIVE_CUSTODY: 현재 저장 확인 실패 (historical={:?}, original={:?}, cause={:?})",
            blocked.prior.historical_state(),blocked.prior.original_cause(),blocked.failure))
}

/// Explicit cold recovery: source and the complete previous Witness are
/// admitted first. RecoveryRequired must take a new durable Pending epoch;
/// terminal Committed is readmitted without manufacturing a recovery attempt.
#[cfg(not(target_arch="wasm32"))]
pub fn resume_saved_general_project_custody_native(
    artifact:&str,revision:&[u8],storage_root:&std::path::Path,locator:&str,
    raw_request:&[u8],token:&[u8;33],madi:u32,
)->Result<JsonValue,String> {
    #[cfg(not(windows))]
    {let _=(artifact,revision,storage_root,locator,raw_request,token,madi);Err("E_NATIVE_CUSTODY_UNSUPPORTED: 현재 복구 기록 저장은 Windows local NTFS에서만 지원합니다".into())}
    #[cfg(windows)]
    {
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::{self as durable,native::SqliteAdapterV1};
        let reject=|failure|format!("E_NATIVE_CUSTODY: 복구 확인 실패 ({failure:?})");
        let bytes=attachment::check_bytes(artifact.as_bytes())?;
        let owned=attachment::restore_saved_runtime_payload(&bytes)?;
        let source=attachment::check_saved_revision_context(&bytes,&owned,&[],revision)?;
        let mut adapter=SqliteAdapterV1::open_recovery_custody(storage_root).map_err(reject)?;
        let admitted=admit_initial_custody_for_restore(&mut adapter,&source,locator,raw_request,token)?;
        let binding=&admitted.payload().binding;
        let digest=binding.transaction_digest().map_err(reject)?;
        let chain=adapter.reread_committed(&digest,&binding.project,&binding.new_revision).map_err(reject)?;
        let payload=durable::checked_native_committed_payload(admitted.payload(),&chain).map_err(reject)?;
        let (mut replay,expected)=rebuild_general_native_history(&adapter,payload,&source)?;
        let expected_bytes=attachment::check_bytes(expected.as_bytes())?;
        let expected_source=attachment::check_saved_revision_context(&expected_bytes,&owned,&[],revision)?;
        if expected_source.runtime_state()!=source.runtime_state() || expected_source.resource_snapshot()!=source.resource_snapshot()
            || expected_source.revision_envelope()!=source.revision_envelope() {
            return Err("E_NATIVE_CUSTODY: 실제 입력·난수·원문·세계 복원 결과가 다릅니다".into());
        }
        let profile=adapter.capability().map_err(reject)?;
        let receipt=crate::snapshot_v2_host_bridge::encode_committed_receipt_candidate(binding,profile.as_bytes()).map_err(reject)?;
        admitted.evidence().check_committed_receipt_evidence(&receipt,profile.as_bytes()).map_err(reject)?;
        replay.receipts.push(receipt);
        let (summary,_,_)=resume_saved_project_artifact_from_runner(artifact,&[],revision,madi,Some((replay.execution.runner,replay.age)))?;
        Ok(summary)
    }
}

#[cfg(not(target_arch="wasm32"))]
pub fn resume_saved_logical_project_custody_native(
    artifact:&str,revision:&[u8],storage_root:&std::path::Path,locator:&str,
    raw_request:&[u8],token:&[u8;33],input:&crate::logical_input_tape::LogicalRecordRequest,
)->Result<JsonValue,String> {
    #[cfg(not(windows))]
    {let _=(artifact,revision,storage_root,locator,raw_request,token,input);Err("E_NATIVE_CUSTODY_UNSUPPORTED: 현재 복구 기록 저장은 Windows local NTFS에서만 지원합니다".into())}
    #[cfg(windows)]
    {
        use crate::unit_wire::{project_runtime_attachment as attachment,resumable_complete_state::CheckpointedLogicalRunV1};
        let reject=|failure|format!("E_NATIVE_CUSTODY: 논리 실행 복원 확인 실패 ({failure:?})");
        let bytes=attachment::check_bytes(artifact.as_bytes())?;
        let owned=attachment::restore_saved_runtime_payload(&bytes)?;
        let source=attachment::check_saved_revision_context(&bytes,&owned,&[],revision)?;
        let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_recovery_custody(storage_root).map_err(reject)?;
        let admitted=admit_initial_custody_for_restore(&mut adapter,&source,locator,raw_request,token)?;
        let digest=admitted.payload().digest;
        let mut run=CheckpointedLogicalRunV1::restore_native_checkpoint(&adapter,&digest,revision,&source,&input.binding).map_err(reject)?;
        // Reuse the logical fixed-seed source/input/work reconstruction owner;
        // it is not a General InputSnapshot or an XOR-tick substitution.
        for snapshot in &input.snapshots {run.step(snapshot)?;}
        Ok(run.session().summary())
    }
}

/// Explicit host operation for the R4 private grant. The report is not a
/// transaction receipt, recovered world, source issuer or successful resume.
#[cfg(not(target_arch="wasm32"))]
pub fn manage_saved_root_grant_native(
    artifact: &str, expected_parent: &[u8], revision: &[u8], storage_root: &std::path::Path,
    operation: NativeRootGrantOperation<'_>,
) -> Result<serde_json::Value, String> {
    #[cfg(not(windows))]
    { let _ = (artifact, expected_parent, revision, storage_root, operation);
      Err("E_NATIVE_ROOT_GRANT_UNSUPPORTED: 현재 저장소 허가는 Windows local NTFS에서만 지원합니다".into()) }
    #[cfg(windows)]
    {
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
        let bytes = attachment::check_bytes(artifact.as_bytes())?;
        let owned = attachment::restore_saved_runtime_payload(&bytes)?;
        let source = attachment::check_saved_revision_context(&bytes, &owned, expected_parent, revision)?;
        let reject = |failure| format!("E_NATIVE_ROOT_GRANT: 저장소 허가를 확인할 수 없습니다 ({failure:?})");
        // Only an explicit registration creates a new DB. create_new refuses
        // existing/copied/revoked containers; check/revoke never provision.
        let mut adapter = match operation {
            NativeRootGrantOperation::Register => SqliteAdapterV1::provision(storage_root),
            _ => SqliteAdapterV1::open_admitted(storage_root),
        }.map_err(reject)?;
        let (locator, action) = match operation {
            NativeRootGrantOperation::Register => {
                let grant=adapter.register_saved_root_grant(&source).map_err(reject)?;
                grant.verify_current(&adapter,&source).map_err(reject)?;
                (grant.locator(),"등록")
            },
            NativeRootGrantOperation::Check(locator) => {
                let grant=adapter.confirm_saved_root_grant(locator,&source).map_err(reject)?;
                grant.verify_current(&adapter,&source).map_err(reject)?;
                (grant.locator(),"확인")
            },
            NativeRootGrantOperation::Revoke(locator) => (adapter.revoke_saved_root_grant(locator,&source).map_err(reject)?,"폐기"),
        };
        Ok(serde_json::json!({"operation":action,"grant_locator":locator,
            "status":if action=="폐기" {"Revoked"} else {"Active"},
            "historical_registration_ack":"NOT_ASSERTED",
            "scope":"native-root-grant-only","recovery_receipt":null,"world_published":false}))
    }
}

#[cfg(not(target_arch="wasm32"))]
fn restore_general_native_checkpoint(
    artifact:&str,expected_parent:&[u8],revision:&[u8],storage_root:&std::path::Path,
    transaction_digest:&[u8;32],
)->Result<VerifiedGeneralNativeRun,String> {
    use crate::unit_wire::{project_runtime_attachment as attachment,resumable_complete_state as complete};
    use crate::snapshot_v2_durable_transaction::{native::SqliteAdapterV1,TransactionEvidenceV1};
    use crate::snapshot_v2_host_bridge as bridge;
    let reject=|failure|format!("E_PROJECT_RUNTIME_NATIVE: 일반 이어 실행 확인 실패 ({failure:?})");
    let bytes=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&bytes)?;
    let context=attachment::check_saved_revision_context(&bytes,&owned,expected_parent,revision)?;
    let project=hex::decode(owned.project().authored_identity.logical_dependency_id.as_str())
        .map_err(|_|"E_PROJECT_RUNTIME_IDENTITY")?;
    let adapter=SqliteAdapterV1::open_admitted(storage_root).map_err(reject)?;
    let profile=adapter.capability().map_err(reject)?;
    let chain=adapter.reread_committed(transaction_digest,&project,revision).map_err(reject)?;
    let payload=chain.payload.as_ref().ok_or("E_PROJECT_RUNTIME_NATIVE: 저장 payload가 없습니다")?;
    if payload.digest!=*transaction_digest || payload.binding.project!=project
        || payload.binding.new_revision!=revision || payload.binding.expected_parent!=expected_parent
        || payload.envelope!=context.revision_envelope() {
        return Err("E_PROJECT_RUNTIME_NATIVE: 원 저장 계보가 다릅니다".into());
    }
    let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&payload.snapshot,context.pesi().as_raw()).map_err(reject)?;
    let request=bridge::ResumableRequestCandidateV2 {binding:payload.binding.clone(),layout,
        revision_bytes:&payload.envelope,request_kind:3,snapshot_bytes:&payload.snapshot};
    let catalog=context.owned().registration().map(|registration|registration.saved_nominal_catalog(context.owned())).transpose()?;
    request.encode_canonical_with_catalog(catalog.as_ref()).map_err(reject)?;
    let source=complete::rebuild_general_complete_state_source_candidate(&request.layout,&context).map_err(reject)?;
    // Preserve the original default construction, including checkpoint history.
    // Never substitute a logical Tape or erase nonempty initial-world state.
    if source.initial_tick!=0 {
        return Err("E_PROJECT_RUNTIME_GENERAL_CONSTRUCTION_UNSUPPORTED: 원 실행 문맥과 저장 이력의 추가 공급이 필요합니다".into());
    }
    let initial=source.initial_world.decode_value_tables().map_err(reject)?;
    if !initial.resources.is_empty() || !initial.parameters.is_empty() || !initial.subjects.is_empty()
        || initial.subject_custody.is_some() || !source.initial_world.atom_evidence.is_empty() {
        return Err("E_PROJECT_RUNTIME_GENERAL_CONSTRUCTION_UNSUPPORTED: 초기 세계를 지울 수 없습니다".into());
    }
    let (mut replay,expected)=rebuild_general_native_history(&adapter,payload,&context)?;
    if expected!=artifact {
        // Compare admitted transport components, not caller pretty whitespace.
        let expected_bytes=attachment::check_bytes(expected.as_bytes())?;
        let expected_context=attachment::check_saved_revision_context(&expected_bytes,&owned,expected_parent,revision)?;
        if expected_context.runtime_state()!=context.runtime_state()
            || expected_context.resource_snapshot()!=context.resource_snapshot()
            || expected_context.revision_envelope()!=context.revision_envelope() {
            return Err("E_PROJECT_RUNTIME_GENERAL_REPLAY: 저장 실행 이력 또는 세계가 다릅니다".into());
        }
    }
    let receipt=bridge::encode_committed_receipt_candidate(&payload.binding,profile.as_bytes()).map_err(reject)?;
    let mut evidence=TransactionEvidenceV1::prepared(payload.binding.clone()).map_err(reject)?;
    evidence.synced().map_err(reject)?;
    evidence.verified(&payload.snapshot,&payload.snapshot).map_err(reject)?;
    evidence.head_changed().map_err(reject)?;
    evidence.container_confirmation(adapter.confirm_container_durable()).map_err(reject)?;
    let fresh_chain=adapter.reread_committed(transaction_digest,&project,revision).map_err(reject)?;
    evidence.committed_native_chain(payload,&fresh_chain).map_err(reject)?;
    evidence.check_committed_receipt_evidence(&receipt,profile.as_bytes()).map_err(reject)?;
    replay.receipts.push(receipt);
    Ok(replay)
}

/// Reconstruct each checkpoint interval from its original owner. Intermediate
/// receipts are matched to actual stored payload/revision rows, never treated
/// as current-head receipts or trusted merely because their bytes decode.
#[cfg(not(target_arch="wasm32"))]
fn rebuild_general_native_history(
    adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    current:&crate::snapshot_v2_durable_transaction::native::StoredPayloadV1,
    context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
)->Result<(VerifiedGeneralNativeRun,String),String> {
    rebuild_general_native_history_with_source(adapter,current,context,None)
}
#[cfg(not(target_arch="wasm32"))]
fn rebuild_general_native_history_with_source(
    adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    current:&crate::snapshot_v2_durable_transaction::native::StoredPayloadV1,
    context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    source_history:Option<&dyn Fn(usize)->Result<crate::snapshot_v2_durable_transaction::native::StoredPayloadV1,String>>,
)->Result<(VerifiedGeneralNativeRun,String),String> {
    rebuild_general_native_history_with_evidence(adapter,current,context,source_history,None)
}
#[cfg(not(target_arch="wasm32"))]
fn rebuild_general_native_history_with_evidence(
    adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    current:&crate::snapshot_v2_durable_transaction::native::StoredPayloadV1,
    context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    source_history:Option<&dyn Fn(usize)->Result<crate::snapshot_v2_durable_transaction::native::StoredPayloadV1,String>>,
    evidence:Option<GeneralEvidenceSourceV1<'_>>,
)->Result<(VerifiedGeneralNativeRun,String),String> {
    use crate::unit_wire::{project_runtime_attachment as attachment,resumable_complete_state as complete};
    use crate::snapshot_v2_host_bridge as bridge;
    let reject=|failure|format!("E_PROJECT_RUNTIME_NATIVE: 일반 저장 이력 확인 실패 ({failure:?})");
    let owned=context.owned();let issued_pesi=context.pesi();let pesi=issued_pesi.as_raw();
    let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose()?;
    let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&current.snapshot,pesi).map_err(reject)?;
    let candidate=complete::rebuild_general_complete_state_source_candidate(&layout,context).map_err(reject)?;
    let subject_bootstrap=crate::unit_wire::subject_execution::SubjectExecutionStage::from_recorded_initial(owned,&candidate.initial_world)?;
    let profile=adapter.capability().map_err(reject)?;
    let mut staged:Option<VerifiedGeneralNativeRun>=None;
    let mut previous_artifact=String::new();
    let mut previous_parent=Vec::new();let mut previous_revision=Vec::new();
    // Keep only one historical payload at a time; do not multiply the 2MB
    // frame bound by an attacker-supplied number of checkpoint receipts.
    for index in 0..=candidate.prior_durable_receipts.len() {
        let payload=if index<candidate.prior_durable_receipts.len() {
            let parsed=bridge::check_committed_receipt_payload_v1(candidate.prior_durable_receipts[index]).map_err(reject)?;
            let head=parsed.native_head().map_err(reject)?;
            if head.project!=current.binding.project{return Err("E_PROJECT_RUNTIME_NATIVE: 다른 작품의 저장 이력입니다".into());}
            if let Some(source_history)=source_history {
                let old=source_history(index)?;
                parsed.check_binding(&old.binding,profile.as_bytes()).map_err(reject)?;
                if old.digest!=head.digest||old.binding.new_revision!=head.revision{return Err("E_SOURCE_WORLD_HISTORY: 저장 계보가 다릅니다".into());}
                old
            }else{
            let chain=adapter.reread_committed(&head.digest,&head.project,&head.revision).map_err(reject)?;
            let old=chain.payload.ok_or("E_PROJECT_RUNTIME_NATIVE: 이전 저장 payload가 없습니다")?;
            let revision=chain.revision.ok_or("E_PROJECT_RUNTIME_NATIVE: 이전 저장 revision이 없습니다")?;
            if revision.project!=old.binding.project || revision.revision!=old.binding.new_revision
                || revision.digest!=old.digest || revision.integrity!=old.binding.revision_integrity {return Err("E_PROJECT_RUNTIME_NATIVE: 이전 저장 계보가 다릅니다".into());}
            parsed.check_binding(&old.binding,profile.as_bytes()).map_err(reject)?;
            old}
        } else {current.clone()};
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&payload.snapshot,pesi).map_err(reject)?;
        let descriptor=complete::rebuild_source_descriptor_candidate(layout.supplies[9],owned).map_err(reject)?;
        if descriptor.source_descriptor_prefix!=candidate.source_descriptor_prefix
            || descriptor.execution_instance!=candidate.execution_instance
            || descriptor.prior_durable_receipts!=candidate.prior_durable_receipts[..index] {
            return Err("E_PROJECT_RUNTIME_NATIVE: 이전 실행 정체·저장 이력이 다릅니다".into());
        }
        let count=u32::try_from(descriptor.successful_madi_count).map_err(|_|"E_PROJECT_RUNTIME_GENERAL_COUNT")?;
        let mut run=if let Some(mut run)=staged.take() {
            if payload.binding.expected_parent!=previous_revision {return Err("E_PROJECT_RUNTIME_NATIVE: 이어 저장의 부모가 다릅니다".into());}
            let suffix=u64::from(count).checked_sub(run.execution.next_tick).ok_or("E_PROJECT_RUNTIME_NATIVE: 저장 마디가 거꾸로입니다")?;
            let previous_frame=if source_history.is_some(){
                let previous_payload=source_history.unwrap()(index-1)?;
                Some(previous_payload)
            }else{None};
            let previous_layout=previous_frame.as_ref().map(|p|crate::unit_wire::check_resumable_complete_state_layout_v2(&p.snapshot,pesi)).transpose().map_err(reject)?;
            // Source history carries the actual complete lifetime tables. The
            // ordinary JSON-only restore path remains unchanged and denied.
            let result=run.execution.result.take();
            let (_,execution,rows)=resume_saved_project_artifact_with_general_result(&previous_artifact,&previous_parent,
                &previous_revision,u32::try_from(suffix).map_err(|_|"E_PROJECT_RUNTIME_GENERAL_COUNT")?,Some((run.execution.runner,run.age)),previous_layout.as_ref().map(|l|l.world),result)?;
            run.rows.extend(rows);run.execution=execution;run
        } else {
            let authored=owned.project().save()?;
            let prefix=run_general_input_prefix_with_evidence(owned.clone(),&authored,count,candidate.base_rng_seed,None,None,candidate.age,subject_bootstrap.as_ref(),evidence)?;
            previous_artifact=prefix.export_saved_artifact_native_complete(&payload.binding.expected_parent,&payload.binding.new_revision)?;
            VerifiedGeneralNativeRun {owned:owned.clone(),execution:prefix.execution,rows:prefix.rows,
                base:candidate.base_rng_seed,age:candidate.age,receipts:Vec::new()}
        };
        if index>0 {previous_artifact=if source_history.is_some(){run.execution.export_continued_artifact_native_complete(owned,
            &payload.binding.expected_parent,&payload.binding.new_revision)?}else{run.execution.export_continued_artifact(owned,
            &payload.binding.expected_parent,&payload.binding.new_revision)?};}
        let checked=attachment::check_bytes(previous_artifact.as_bytes())?;
        let old_context=attachment::check_saved_revision_context_with_frame(&checked,owned,
            &payload.binding.expected_parent,&payload.binding.new_revision,layout.world)?;
        if payload.envelope!=old_context.revision_envelope(){return Err("E_PROJECT_RUNTIME_NATIVE: 이전 저장 원문 결속이 다릅니다".into());}
        let request=bridge::ResumableRequestCandidateV2 {binding:payload.binding.clone(),layout,
            revision_bytes:&payload.envelope,request_kind:3,snapshot_bytes:&payload.snapshot};
        request.encode_canonical_with_catalog(catalog.as_ref()).map_err(reject)?;
        complete::rebuild_general_complete_state_source_candidate(&request.layout,&old_context).map_err(reject)?;
        let expected_descriptor=complete::encode_general_source_descriptor_with_receipts(owned,&run.execution.runner,
            run.age,run.base,run.execution.next_tick,&run.receipts).map_err(reject)?;
        let world=complete::encode_owned_world_candidate_with_source(run.execution.world.world(),pesi,run.execution.next_tick,owned).map_err(reject)?;
        if request.layout.supplies[9]!=expected_descriptor
            || request.layout.supplies[8]!=complete::encode_general_predecessor(&run.receipts).map_err(reject)?
            || request.layout.supplies[4]!=complete::encode_general_processed_inputs(&run.rows,0,run.base).map_err(reject)?
            || request.layout.world.encode_canonical_with_catalog(catalog.as_ref()).map_err(reject)?!=world {
            return Err("E_PROJECT_RUNTIME_GENERAL_REPLAY: 저장 이력의 실제 입력·세계·정체가 다릅니다".into());
        }
        let definition=owned.definition_identity().as_bytes();
        complete::check_boundary_supply_v1(&request.layout,complete::GENERAL_XOR_RNG_PROFILE,
            definition,definition,&run.execution.runner.work_cost_receipt()).map_err(reject)?;
        if index<candidate.prior_durable_receipts.len() {run.receipts.push(candidate.prior_durable_receipts[index].to_vec());}
        previous_parent=payload.binding.expected_parent.clone();previous_revision=payload.binding.new_revision.clone();
        staged=Some(run);
    }
    Ok((staged.ok_or("E_PROJECT_RUNTIME_NATIVE: 저장 이력이 없습니다")?,previous_artifact))
}

#[cfg(not(target_arch="wasm32"))]
fn checkpoint_logical_project_native(
    run:&mut crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1,
    owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    parent:&[u8],revision:&[u8],storage_root:&std::path::Path,
    initialize_store:bool,expected_project_receipt:Option<&[u8]>,
)->Result<NativeLogicalCheckpoint,String> {
    use crate::unit_wire::project_runtime_attachment as attachment;
    use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
    let reject=|failure|format!("E_PROJECT_RUNTIME_NATIVE: 저장·복원 확인 실패 ({failure:?})");
    run.check_checkpoint_channels()?;
    let saved_artifact=run.export_saved_artifact(parent,revision).map_err(reject)?;
    let checked=attachment::check_bytes(saved_artifact.as_bytes())?;
    let context=attachment::check_saved_revision_context(&checked,&owned,parent,revision)?;
    let world=run.encode_world_frame().map_err(reject)?;
    let request=run.encode_checkpoint_request(&world,&context,parent,revision).map_err(reject)?;
    let candidate=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2(&request,context.pesi().as_raw()).map_err(reject)?;
    let transaction_digest=candidate.binding.transaction_digest().map_err(reject)?;
    let expected_head=expected_project_receipt.map(|bytes|
        crate::snapshot_v2_host_bridge::check_committed_receipt_payload_v1(bytes)
            .and_then(|receipt|receipt.native_head())).transpose().map_err(reject)?;
    if expected_head.as_ref().map(|head|head.revision.as_slice()).unwrap_or(&[])!=parent
        || expected_head.as_ref().is_some_and(|head|head.project!=candidate.binding.project) {
        return Err(reject(crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1::CustodyConflict));
    }
    let summary=run.session().summary();
    let adapter=if initialize_store {SqliteAdapterV1::provision(storage_root)}else{SqliteAdapterV1::open_admitted(storage_root)}.map_err(reject)?;
    let committed_receipt=run.save_native_checkpoint(&request,&context,&adapter,expected_head.as_ref()).map_err(|failure|
        format!("E_PROJECT_RUNTIME_NATIVE: 저장·복원 확인 실패 (state={:?}, cause={:?})",failure.state,failure.cause))?;
    Ok(NativeLogicalCheckpoint {saved_artifact,committed_receipt,transaction_digest,summary})
}

/// Continue the restored execution and checkpoint it without reissuing a
/// fresh execution instance or discarding its confirmed receipt lineage.
#[cfg(not(target_arch="wasm32"))]
pub fn resume_and_save_logical_project_native(
    artifact:&str,expected_parent:&[u8],revision:&[u8],
    storage_root:&std::path::Path,transaction_digest:&[u8;32],
    input:&crate::logical_input_tape::LogicalRecordRequest,
    next_revision:&[u8],expected_project_receipt:&[u8],
)->Result<NativeLogicalCheckpoint,String> {
    use crate::unit_wire::{project_runtime_attachment as attachment,resumable_complete_state::CheckpointedLogicalRunV1};
    let bytes=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&bytes)?;
    let context=attachment::check_saved_revision_context(&bytes,&owned,expected_parent,revision)?;
    let reject=|failure|format!("E_PROJECT_RUNTIME_NATIVE: 저장·복원 확인 실패 ({failure:?})");
    let adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_admitted(storage_root).map_err(reject)?;
    let mut run=CheckpointedLogicalRunV1::restore_native_checkpoint(
        &adapter,transaction_digest,revision,&context,&input.binding).map_err(reject)?;
    for snapshot in &input.snapshots {run.step(snapshot)?;}
    // No two adapters overlap on the store. The common save path freshly
    // rereads and CAS-checks the independently supplied project receipt.
    drop(adapter);
    checkpoint_logical_project_native(&mut run,&owned,revision,next_revision,storage_root,false,Some(expected_project_receipt))
}

/// Resume the existing RD-686 source-bound artifact, not an input recording.
#[cfg(not(target_arch="wasm32"))]
pub fn resume_saved_project_artifact_native(
    artifact:&str,expected_parent:&[u8],revision:&[u8],
    storage_root:&std::path::Path,transaction_digest:&[u8;32],
    input:&crate::logical_input_tape::LogicalRecordRequest,
) -> Result<JsonValue,String> {
    use crate::unit_wire::{project_runtime_attachment as attachment,resumable_complete_state::CheckpointedLogicalRunV1};
    let bytes=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&bytes)?;
    let context=attachment::check_saved_revision_context(&bytes,&owned,expected_parent,revision)?;
    let reject=|failure|format!("E_PROJECT_RUNTIME_NATIVE: 저장·복원 확인 실패 ({failure:?})");
    let adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_admitted(storage_root)
        .map_err(reject)?;
    let mut run=CheckpointedLogicalRunV1::restore_native_checkpoint(
        &adapter,transaction_digest,revision,&context,&input.binding).map_err(reject)?;
    // All transitions remain private until the complete requested suffix has
    // succeeded. A failed suffix publishes no partial stdout/state or seal.
    for snapshot in &input.snapshots {run.step(snapshot)?;}
    Ok(run.session().summary())
}

/// Resume the existing RD-686 source-bound artifact, not an input recording.
/// The selected storage lineage is explicit. No fresh-source reread, source
/// repair, initialization step, durable receipt or invented replay history.
pub fn resume_saved_project_artifact(
    artifact:&str,expected_parent:&[u8],revision:&[u8],ticks:u32,
)->Result<JsonValue,String>{
    resume_saved_project_artifact_from_runner(artifact,expected_parent,revision,ticks,None)
        .map(|(result,_,_)|result)
}

/// Native admission passes the runner reconstructed from retained captures,
/// successful inputs and checked work boundary. Do not replace that runner
/// with an unmetered fresh instance before the next real update.
fn resume_saved_project_artifact_from_runner(
    artifact:&str,expected_parent:&[u8],revision:&[u8],ticks:u32,
    admitted_runner:Option<(DdnRunner,AgeTarget)>,
)->Result<(JsonValue,GeneralCompletedExecution,Vec<crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow>),String>{
    resume_saved_project_artifact_from_runner_with_frame(artifact,expected_parent,revision,ticks,admitted_runner,None)
}
fn resume_saved_project_artifact_from_runner_with_frame(
    artifact:&str,expected_parent:&[u8],revision:&[u8],ticks:u32,admitted_runner:Option<(DdnRunner,AgeTarget)>,
    frame:Option<crate::unit_wire::CompleteStateFrameV1<'_>>,
)->Result<(JsonValue,GeneralCompletedExecution,Vec<crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow>),String>{
    resume_saved_project_artifact_with_general_result(artifact,expected_parent,revision,ticks,admitted_runner,frame,None)
}
fn resume_saved_project_artifact_with_general_result(
    artifact:&str,expected_parent:&[u8],revision:&[u8],ticks:u32,admitted_runner:Option<(DdnRunner,AgeTarget)>,
    frame:Option<crate::unit_wire::CompleteStateFrameV1<'_>>,mut result_collector:Option<GeneralResultCollectorV1>,
)->Result<(JsonValue,GeneralCompletedExecution,Vec<crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow>),String>{
    use crate::unit_wire::project_runtime_attachment as attachment;
    let bytes=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&bytes)?;
    let context=if let Some(frame)=frame{attachment::check_saved_revision_context_with_frame(&bytes,&owned,expected_parent,revision,frame)?}
        else{attachment::check_saved_revision_context(&bytes,&owned,expected_parent,revision)?};
    let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(&owned)).transpose()?;
    let runtime=context.parse_runtime()?;
    if runtime["schema"]!=ENGINE_RESPONSE_SCHEMA{return Err("E_PROJECT_RUNTIME_STATE_SNAPSHOT: 실행 응답 판이 다릅니다".into());}
    let history=runtime.get("runner_input_history").and_then(JsonValue::as_object).ok_or("E_PROJECT_RUNTIME_INPUT_HISTORY")?;
    if history.len()!=2||history.get("schema").and_then(JsonValue::as_str)!=Some("ddn.runner_input_history.v1"){
        return Err("E_PROJECT_RUNTIME_INPUT_HISTORY".into());
    }
    let consumed=history.get("consumed_keys_decimal").and_then(JsonValue::as_str).ok_or("E_PROJECT_RUNTIME_INPUT_HISTORY")?;
    let consumed_keys=consumed.parse::<u64>().map_err(|_|"E_PROJECT_RUNTIME_INPUT_HISTORY")?;
    if consumed_keys.to_string()!=consumed{return Err("E_PROJECT_RUNTIME_INPUT_HISTORY".into());}
    let sources=owned.sources();
    let age=admitted_runner.as_ref().map(|(_,age)|*age).unwrap_or_else(crate::ddn_runtime::default_age_target);
    let mut program=DdnProgram::from_saved_v25_project_with_mode(sources.clone(),ParseMode::Strict,
        age)?;
    program.bind_registered_project_context(&owned)?;
    crate::source_analysis::validate_program_for_execution(program.canonical_program())?;
    let source_path=&sources.preparation().target_path;
    let source=&sources.preparation().source;
    let prepared=prepare_supported_source_for_execution_with_program(source,source_path,&program)
        .map_err(|error|format!("{}: {}",error.code,error.message))?;
    let source_hash=sha256_hex(source.as_bytes());
    let canonical_hash=sha256_hex(prepared.canonical_ddn.as_bytes());
    let mut source_runner = DdnRunner::new(program,DEFAULT_UPDATE_NAME).with_bogae_activation();
    source_runner.configure_project_entry(sources.entry_declaration().name())?;
    let tuple_plan = source_runner.saved_tuple_restore_plan();
    let staged=crate::unit_wire::runtime_checkpoint::stage_runtime_state_with_complete_frame(&runtime,source_path,&source_hash,&canonical_hash,tuple_plan.as_ref(),catalog.as_ref(),context.world_frame())?;
    if staged.tick_id>0 {source_runner.validate_restored_derived_world(staged.world.world())?;}
    let mut next_tick=staged.tick_id;
    next_tick.checked_add(u64::from(ticks)).ok_or("E_WASM_STATE_TICK_INVALID: 마디 순번 넘침")?;
    let params=staged.param_overrides;
    let input=staged.input;
    let dt=input.dt;crate::input_receipt::validate_alpha_host_dt(dt)?;
    let mut world=staged.world;
    let mut display_outputs=crate::output_display::restore(&runtime)?;
    if runtime.get("output_display").is_some() && runtime["output_display"]["world_state_hash"]!=current_state_hash(world.world()){
        return Err("E_OUTPUT_DISPLAY_SNAPSHOT: 표시 정보가 복원한 세계와 다릅니다.".into());
    }
    let mut public_observation_frames=staged.public_observation_frames;
    let mut bogae_projection=staged.bogae_projection;
    let mut observation_surface=staged.observation_surface;
    let mut runner=if let Some((runner,_))=admitted_runner {
        let reject=||"E_PROJECT_RUNTIME_GENERAL_REPLAY: 복원 실행기의 원문·시작·입력·장면 이력이 다릅니다".to_string();
        let running_sources=runner.saved_program_sources().ok_or_else(reject)?;
        let supply=crate::unit_wire::resumable_complete_state::encode_source_capture_supply(sources)
            .map_err(|_|reject())?;
        crate::unit_wire::resumable_complete_state::check_source_capture_supply(&supply,running_sources)
            .map_err(|_|reject())?;
        if runner.configured_saved_start()!=Some(sources.entry_declaration().name())
            || runner.consumed_keys_pressed()!=consumed_keys
            || crate::unit_wire::SnapshotJsonOwner::from_value(runner.scene_history_snapshot()).compact_text()
                !=crate::unit_wire::SnapshotJsonOwner::from_value(staged.scene_history.snapshot()).compact_text() {
            return Err(reject());
        }
        runner
    } else {
        let mut runner=source_runner;
        if owned.project().schema==crate::authored_project::SCHEMA_V2
            || owned.project().authored_identity.modules.iter().any(|module|!module.compositions.is_empty()){
            runner.configure_authored_bogae(&owned.project().save()?)?;
        }
        runner.restore_consumed_keys_pressed(consumed_keys);
        runner.restore_scene_history(staged.scene_history);
        runner
    };
    let mut defaults=HashMap::new();seed_bogae_defaults(&mut defaults);
    let captures=sources.captures().iter().map(|(_,capture)|capture).collect::<Vec<_>>();
    crate::source_snapshot_admission::retain_verified_replay(&captures)
        .map_err(|error|format!("E_SOURCE_ADMISSION_CUSTODY: {error:?}"))?;
    let mut frames=Vec::new();
    let mut actual_inputs=Vec::new();
    actual_inputs.try_reserve_exact(usize::try_from(ticks).map_err(|_|"E_PROJECT_RUNTIME_INPUT_SUPPLY")?)
        .map_err(|_|"E_PROJECT_RUNTIME_INPUT_SUPPLY: 이어 실행 입력을 보존할 공간이 없습니다")?;
    for _ in 0..ticks{
        let frame=InputSnapshot{tick_id:next_tick,dt,keys_pressed:input.keys_pressed,last_key_name:input.last_key_name.clone(),
            pointer_x_i32:input.pointer_x_i32,pointer_y_i32:input.pointer_y_i32,ai_injections:Vec::new(),net_events:Vec::new(),
            frame_source:InputSource::Person,rng_seed:input.rng_seed^next_tick};
        let output=runner.run_update_diagnostic_with_control_edges(world.world(),&frame,&defaults,None,Some(&params))
            .map_err(|error|execution_error_diagnostic(error,&sources.preparation().target_path).to_json_value().to_string())?;
        let receipt=crate::input_receipt::receipt(&frame,&output.input_reads);
        let prepared_result=if let Some(collector)=result_collector.as_mut(){
            Some(collector.prepare(output.result_emissions,next_tick,&receipt)?)
        }else{None};
        let observations=public_observation_events_json(&output.public_observations)?;
        let mut sink=ddonirang_core::signals::VecSignalSink::default();
        world.apply_patch(&output.patch,next_tick,&mut sink);
        if let (Some(collector),Some(prepared))=(result_collector.as_mut(),prepared_result){collector.commit(prepared)?;}
        display_outputs.extend(output.display_outputs);
        crate::output_display::retain_current(&mut display_outputs,&serialize_world_resources(world.world()));
        public_observation_frames.value_mut().as_array_mut().unwrap().push(json!({"committed_madi":next_tick,"state_hash":current_state_hash(world.world()),
            "input_snapshot":serialize_input_snapshot(&frame,Some(input.rng_seed)),"public_observations":observations,
            "input_snapshot_sha256":crate::input_receipt::snapshot_sha256(&frame),"input_reads":output.input_reads,
            "tape_v1_recording_error":crate::input_receipt::validate_tape_v1_reads(&output.input_reads).err()}));
        observation_surface=crate::unit_wire::runtime_checkpoint::build_public_observation_surface(
            source_path,&source_hash,&canonical_hash,public_observation_frames.as_array().unwrap())?;
        bogae_projection=output.bogae_projection.map(crate::unit_wire::SnapshotJsonOwner::from_value);
        if let Some(projection)=bogae_projection.as_mut(){
            crate::bogae_time::resolve_result_tables(projection.value_mut(),public_observation_frames.as_array().unwrap(),
                &format!("sha256:{source_hash}"),&format!("sha256:{canonical_hash}"))?;
        }
        frames.push(json!({"madi":next_tick,"state_hash":current_state_hash(world.world()),"input_receipt":receipt}));
        actual_inputs.push(crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow {input:frame,edges:None});
        next_tick+=1;
    }
    let mut result=crate::unit_wire::SnapshotJsonOwner::from_value(json!({"ok":true,"next_tick_id":next_tick,"state_hash":current_state_hash(world.world()),
        "resources":serialize_world_resources(world.world()),"frames":frames,
        "public_observation_frames":null,"public_observation_trace":null,
        "public_observation_trace_hash":observation_surface.trace_hash,
        "public_observation_table_projection":null,
        "durable_receipt":null,"replay_history":"NOT_PRODUCED"}));
    result.value_mut()["public_observation_frames"]=public_observation_frames.into_parent_value();
    result.value_mut()["public_observation_trace"]=observation_surface.trace.into_parent_value();
    result.value_mut()["public_observation_table_projection"]=observation_surface.table_projection.into_parent_value();
    crate::bogae_projection::attach(result.value_mut(),bogae_projection.as_deref());
    crate::output_display::attach(result.value_mut(),&display_outputs);
    let last_input=actual_inputs.last().map(|row|serialize_input_snapshot(&row.input,Some(input.rng_seed)))
        .unwrap_or_else(||runtime["input"].clone());
    let mut saved_runtime=crate::unit_wire::SnapshotJsonOwner::from_value(
        build_engine_state_json(&world,next_tick,last_input,&Patch::default()));
    let observations=result["public_observation_frames"].as_array().ok_or("E_PROJECT_RUNTIME_OBSERVATION_FRAMES")?
        .iter().map(|row|crate::unit_wire::SnapshotJsonOwner::clone_from_value(row).into_parent_value()).collect();
    let mut snapshot=runtime_state_snapshot(&world,observations,next_tick);
    snapshot["scene_history"]=runner.scene_history_snapshot();
    snapshot["param_overrides"]=runtime.pointer("/state/snapshot_v2/param_overrides")
        .ok_or("E_PROJECT_RUNTIME_PARAMETER_SNAPSHOT")?.clone();
    saved_runtime.value_mut()["state"]["snapshot_v2"]=snapshot;
    saved_runtime.value_mut()["runner_input_history"]=json!({"schema":"ddn.runner_input_history.v1",
        "consumed_keys_decimal":runner.consumed_keys_pressed().to_string()});
    crate::bogae_projection::attach(saved_runtime.value_mut(),bogae_projection.as_deref());
    crate::output_display::attach(saved_runtime.value_mut(),&display_outputs);
    Ok((result.into_parent_value(),GeneralCompletedExecution {runner,world,next_tick,result:result_collector,saved_runtime:Some(saved_runtime)},actual_inputs))
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
    // A one-file authored Bogae has explicit presentation identity but no
    // runtime-storage root. Keep its existing execution path; only rooted
    // projects acquire the complete-state source owner. Saving stays strict.
    let owned = project.source_snapshot.root_path.as_ref().map(|_|
        crate::project_revision_envelope_supply::admit_fresh_authored_project(artifact_json.as_bytes())
    ).transpose()?;
    run_summary_with_general_input_supplier(
        source, file_path, requested_ticks, None, None, None, Some(profile),
        None, 0, project_start.as_deref().unwrap_or(DEFAULT_UPDATE_NAME), None,
        project_start.is_some(), inputs.as_deref(),
        param_overrides, None, Some(artifact_json), None, owned.as_ref(), None, None,
    ).map_err(|error| error.to_json_value().to_string())
}

/// Actual general-run input supplier. This is an in-process source-bound
/// candidate, not a complete-state admission or a durability receipt.
pub(crate) struct GeneralSuccessfulInputPrefix {
    owned: crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    rows: Vec<crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow>,
    base_seed: u64,
    summary: RuntimeJsonOutput,
    execution: GeneralCompletedExecution,
    construction: GeneralInputConstruction,
}
struct GeneralCompletedExecution {
    runner: DdnRunner, world: DetNuri, next_tick: u64,
    result:Option<GeneralResultCollectorV1>,
    /// Actual continuation projection, not a reconstructed fresh-run summary.
    saved_runtime:Option<crate::unit_wire::SnapshotJsonOwner>,
}
impl GeneralCompletedExecution {
    fn export_continued_artifact(
        &self,owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
        parent:&[u8],revision:&[u8],
    )->Result<String,String> {
        self.export_continued_artifact_with_context(owned,parent,revision,false)
    }
    fn export_continued_artifact_native_complete(&self,owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
        parent:&[u8],revision:&[u8])->Result<String,String>{self.export_continued_artifact_with_context(owned,parent,revision,true)}
    fn export_continued_artifact_with_context(&self,owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
        parent:&[u8],revision:&[u8],native_complete:bool)->Result<String,String>{
        if self.runner.has_temporary_registration(){return Err("E_VC04_TEMPORARY_SAVED_ESCAPE: 임시 실행을 저장 작품 상태로 승격할 수 없습니다".into())}
        if !native_complete&&self.world.world().subject_world().is_some() {
            return Err("E_VC04_SUBJECT_RUNTIME_CONTEXT_REQUIRED: 대상 수명 표를 보존하는 실행 상태 공급이 필요합니다".into());
        }
        let runtime=self.saved_runtime.as_ref().ok_or("E_PROJECT_RUNTIME_CONTINUATION_CARRIER")?;
        let resources=runtime.pointer("/state/snapshot_v2/resources").ok_or("E_PROJECT_RUNTIME_RESOURCE_SCHEMA")?;
        if runtime["state_hash"]!=current_state_hash(self.world.world())
            || runtime.pointer("/state/snapshot_v2/next_tick_id").and_then(JsonValue::as_u64)!=Some(self.next_tick)
            || !crate::unit_wire::SnapshotJsonOwner::clone_from_value(resources)
                .equals_json(&serialize_world_resources_snapshot(self.world.world())) {
            return Err("E_PROJECT_RUNTIME_CONTINUATION_CARRIER: 저장 투영과 현재 실행 세계가 다릅니다".into());
        }
        let sources=self.runner.saved_program_sources().ok_or("E_PROJECT_RUNTIME_SOURCE_OWNER")?;
        let supply=crate::unit_wire::resumable_complete_state::encode_source_capture_supply(owned.sources())
            .map_err(|_|"E_PROJECT_RUNTIME_SOURCE_OWNER")?;
        crate::unit_wire::resumable_complete_state::check_source_capture_supply(&supply,sources)
            .map_err(|_|"E_PROJECT_RUNTIME_SOURCE_OWNER")?;
        if self.runner.configured_saved_start()!=Some(owned.sources().entry_declaration().name()) {
            return Err("E_PROJECT_RUNTIME_SOURCE_OWNER".into());
        }
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(owned)?;
        // Same complete world gate as first checkpoint: never drop legacy lanes.
        let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose()?;
        let world_bytes=crate::unit_wire::resumable_complete_state::encode_owned_world_candidate_with_source(
            self.world.world(),issued.pesi().as_raw(),self.next_tick,owned)
            .map_err(|error|format!("E_PROJECT_RUNTIME_GENERAL_CHANNEL: {error:?}"))?;
        let snapshot=crate::unit_wire::serialize_resource_snapshot_json(resources);
        if native_complete {
            let frame=crate::unit_wire::check_complete_state_frame_v1(&world_bytes,issued.pesi().as_raw())
                .map_err(|e|format!("E_PROJECT_RUNTIME_GENERAL_CHANNEL: {e:?}"))?;
            crate::unit_wire::project_runtime_attachment::encode_saved_runtime_artifact_with_frame(
                owned,&runtime.compact_text(),&snapshot,parent,revision,Some(frame))
        }else{crate::unit_wire::project_runtime_attachment::encode_saved_runtime_artifact(
            owned,&runtime.compact_text(),&snapshot,parent,revision)}
    }
}
/// Retained at the real fresh-run frontdoor, never inferred from a capsule or
/// observation JSON. Input origin affects receipts/byte work as well as values.
struct GeneralInputConstruction {
    subject_bootstrap:Option<crate::unit_wire::subject_execution::SubjectExecutionStage>,
    requested_ticks:u32,
    age:AgeTarget,
    snapshots:Option<Vec<InputSnapshot>>,
    tape:Option<InputTape>,
}
impl GeneralSuccessfulInputPrefix {
    /// A fresh request is built only from this non-public actual execution
    /// owner. It never accepts a decoded capsule as its expected context.
    pub(crate) fn encode_checkpoint_request(
        &self,context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        parent:&[u8],revision:&[u8],
    )->Result<Vec<u8>,String> {
        use crate::unit_wire::resumable_complete_state as complete;
        use sha2::Digest;
        let reject=|failure|format!("E_PROJECT_RUNTIME_NATIVE: 일반 저장 요청 결속 실패 ({failure:?})");
        self.check_reconstructed_prefix()?;
        let expected_artifact=if context.world_frame().is_some(){self.export_saved_artifact_native_complete(parent,revision)?}
            else{self.export_saved_artifact(parent,revision)?};
        let expected=crate::unit_wire::project_runtime_attachment::check_bytes(expected_artifact.as_bytes())?;
        let expected_context=if let Some(frame)=context.world_frame(){
            crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(&expected,&self.owned,parent,revision,frame)?
        }else{crate::unit_wire::project_runtime_attachment::check_saved_revision_context(&expected,&self.owned,parent,revision)?};
        if expected_context.revision_envelope()!=context.revision_envelope()
            || expected_context.runtime_state()!=context.runtime_state()
            || expected_context.resource_snapshot()!=context.resource_snapshot() {
            return Err("E_PROJECT_RUNTIME_NATIVE: 저장 원문·현재 실행 소유권이 다릅니다".into());
        }
        let boundary=complete::encode_general_boundary_supplies(&self.owned,&self.execution.runner,
            &self.rows,self.base_seed,self.execution.next_tick).map_err(reject)?;
        let supplies=[context.revision_envelope().to_vec(),complete::encode_source_capture_supply(self.owned.sources()).map_err(reject)?,
            complete::encode_constituent_supply(&self.owned).map_err(reject)?,
            complete::encode_execution_contract_supply(&self.owned,&self.execution.runner).map_err(reject)?,
            self.input_supply().map_err(reject)?,boundary[0].clone(),boundary[1].clone(),boundary[2].clone(),
            vec![0],self.source_rebuild_descriptor_supply().map_err(reject)?];
        let world_bytes=self.encode_world_frame().map_err(reject)?;
        let pesi=*context.pesi().as_raw();
        let world=crate::unit_wire::check_complete_state_frame_v1(&world_bytes,&pesi).map_err(reject)?;
        let layout=crate::unit_wire::ResumableCompleteStateLayoutV2 {
            world,supplies:std::array::from_fn(|i|supplies[i].as_slice())};
        complete::rebuild_general_complete_state_source_candidate(&layout,context).map_err(reject)?;
        let definition=self.owned.definition_identity().as_bytes();
        let checked=complete::check_boundary_supply_v1(&layout,complete::GENERAL_XOR_RNG_PROFILE,
            definition,definition,&self.execution.runner.work_cost_receipt()).map_err(reject)?;
        if checked.base_seed!=self.base_seed || checked.next_tick!=self.execution.next_tick {
            return Err("E_PROJECT_RUNTIME_NATIVE: 난수·다음 마디 공급이 다릅니다".into());
        }
        let catalog=self.owned.registration().map(|registration|registration.saved_nominal_catalog(&self.owned)).transpose()?;
        let snapshot=layout.encode_canonical_with_catalog(catalog.as_ref()).map_err(reject)?;
        let envelope:JsonValue=serde_json::from_slice(context.revision_envelope()).map_err(|_|"E_PROJECT_RUNTIME_REVISION")?;
        let integrity=envelope["revision_integrity_sha256"].as_str().ok_or("E_PROJECT_RUNTIME_REVISION")?;
        let binding=crate::snapshot_v2_durable_transaction::TransactionBindingV1 {
            project:hex::decode(self.owned.project().authored_identity.logical_dependency_id.as_str()).map_err(|_|"E_PROJECT_RUNTIME_IDENTITY")?,
            expected_parent:parent.to_vec(),new_revision:revision.to_vec(),execution_profile:self.owned.project().source_profile.as_bytes().to_vec(),
            pesi,snapshot_sha256:sha2::Sha256::digest(&snapshot).into(),snapshot_length:snapshot.len() as u64,
            revision_integrity:hex::decode(integrity).map_err(|_|"E_PROJECT_RUNTIME_REVISION")?.try_into().map_err(|_|"E_PROJECT_RUNTIME_REVISION")?,
        };
        crate::snapshot_v2_host_bridge::ResumableRequestCandidateV2 {
            binding,layout,revision_bytes:context.revision_envelope(),request_kind:1,snapshot_bytes:&snapshot,
        }.encode_canonical_with_catalog(catalog.as_ref()).map_err(reject)
    }
    pub(crate) fn input_supply(&self) -> Result<Vec<u8>, crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1> {
        crate::unit_wire::resumable_complete_state::encode_general_processed_inputs(&self.rows,0,self.base_seed)
    }
    pub(crate) fn source_owner(&self) -> &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1 { &self.owned }
    pub(crate) fn summary(&self) -> &RuntimeJsonOutput { &self.summary }
    pub(crate) fn source_rebuild_descriptor_supply(&self)->Result<Vec<u8>,crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1> {
        self.input_supply()?;
        crate::unit_wire::resumable_complete_state::encode_general_fresh_source_descriptor(
            &self.owned,&self.execution.runner,self.construction.age,self.base_seed,self.execution.next_tick)
    }
    /// Reconstruct using retained original construction, then compare actual
    /// consumed rows, complete world, lifecycle/history and all work counters.
    /// Neither replay results nor user-visible success are published here.
    pub(crate) fn check_reconstructed_prefix(&self)->Result<(),String> {
        let artifact=self.owned.project().save()?;
        let replayed=run_general_input_prefix_from_owned_with_subject(self.owned.clone(),&artifact,self.construction.requested_ticks,
            self.base_seed,self.construction.snapshots.as_deref(),self.construction.tape.as_ref(),self.construction.age,self.construction.subject_bootstrap.as_ref(),None)?;
        let reject=|section|format!("E_PROJECT_RUNTIME_GENERAL_REPLAY: 실제 성공 이력과 원 실행 문맥이 다릅니다 ({section})");
        let checks=[
            ("input",self.input_supply().map_err(|error|reject(format!("input:{error:?}")))?
                ==replayed.input_supply().map_err(|error|reject(format!("replay input:{error:?}")))?),
            ("world",self.encode_world_frame().map_err(|error|reject(format!("world:{error:?}")))?
                ==replayed.encode_world_frame().map_err(|error|reject(format!("replay world:{error:?}")))?),
            ("source",self.source_rebuild_descriptor_supply().map_err(|error|reject(format!("source:{error:?}")))?
                ==replayed.source_rebuild_descriptor_supply().map_err(|error|reject(format!("replay source:{error:?}")))?),
            ("keys",self.execution.runner.consumed_keys_pressed()==replayed.execution.runner.consumed_keys_pressed()),
            ("summary",self.summary.pretty_text().map_err(|_|reject("summary".into()))?
                ==replayed.summary.pretty_text().map_err(|_|reject("replay summary".into()))?),
            ("history",crate::unit_wire::SnapshotJsonOwner::from_value(self.execution.runner.scene_history_snapshot()).compact_text()
                ==crate::unit_wire::SnapshotJsonOwner::from_value(replayed.execution.runner.scene_history_snapshot()).compact_text()),
            ("work",self.execution.runner.work_cost_receipt()==replayed.execution.runner.work_cost_receipt()),
        ];
        if let Some((section,_))=checks.iter().find(|(_,matches)|!matches) {
            return Err(reject((*section).into()));
        }
        Ok(())
    }
    pub(crate) fn encode_world_frame(&self) -> Result<Vec<u8>, crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1> {
        use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
        let actual_sources = self.execution.runner.saved_program_sources().ok_or(Failure::InvalidInput)?;
        let source_supply = crate::unit_wire::resumable_complete_state::encode_source_capture_supply(self.owned.sources())?;
        crate::unit_wire::resumable_complete_state::check_source_capture_supply(&source_supply,actual_sources)?;
        if self.execution.runner.configured_saved_start()!=Some(self.owned.sources().entry_declaration().name())
            || self.execution.next_tick!=u64::try_from(self.rows.len()).map_err(|_|Failure::LimitExceeded)? {
            return Err(Failure::CustodyConflict);
        }
        let issued = crate::project_snapshot_identity::current::issue_saved_local_project(&self.owned)
            .map_err(|_|Failure::InvalidInput)?;
        crate::unit_wire::resumable_complete_state::encode_owned_world_candidate_with_source(
            self.execution.world.world(),issued.pesi().as_raw(),self.execution.next_tick,&self.owned)
    }
    pub(crate) fn export_saved_artifact(&self,parent:&[u8],revision:&[u8])->Result<String,String> {
        if self.execution.world.world().subject_world().is_some() {
            return Err("E_VC04_SUBJECT_RUNTIME_CONTEXT_REQUIRED: 대상 수명 표를 보존하는 실행 상태 공급이 필요합니다".into());
        }
        self.export_saved_artifact_with_frame(parent,revision,None)
    }
    /// Value transport only, consumed by actual Native session/request owners.
    /// The frame comes from this run, never JSON or an adapter status flag.
    fn export_saved_artifact_native_complete(&self,parent:&[u8],revision:&[u8])->Result<String,String>{
        let bytes=self.encode_world_frame().map_err(|cause|format!("E_PROJECT_RUNTIME_GENERAL_CHANNEL: {cause:?}"))?;
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(&self.owned)?;
        let frame=crate::unit_wire::check_complete_state_frame_v1(&bytes,issued.pesi().as_raw())
            .map_err(|cause|format!("E_PROJECT_RUNTIME_GENERAL_CHANNEL: {cause:?}"))?;
        self.export_saved_artifact_with_frame(parent,revision,Some(frame))
    }
    fn export_saved_artifact_with_frame(&self,parent:&[u8],revision:&[u8],frame:Option<crate::unit_wire::CompleteStateFrameV1<'_>>)->Result<String,String>{
        self.check_reconstructed_prefix()?;
        self.encode_world_frame().map_err(|error|format!("E_PROJECT_RUNTIME_GENERAL_CHANNEL: 누락 없는 세계 저장 공급이 필요합니다 ({error:?})"))?;
        let mut runtime = crate::unit_wire::SnapshotJsonOwner::clone_from_value(&self.summary["engine_state"]);
        runtime.value_mut()["schema"] = "seamgrim.engine_response.v0".into();
        runtime.value_mut()["runner_input_history"] = json!({"schema":"ddn.runner_input_history.v1",
            "consumed_keys_decimal":self.execution.runner.consumed_keys_pressed().to_string()});
        runtime.value_mut()["state"]["snapshot_v2"]["scene_history"] = self.execution.runner.scene_history_snapshot();
        // The existing VM checkpoint stores committed frame data; its trace
        // identities are reissued from the separately verified saved sources.
        // Do not mix bulk-run trace labels into that VM transport projection.
        // The original summary and actual processed rows remain unchanged.
        let expected_source = sha256_identity(self.owned.sources().preparation().source.as_bytes());
        let frames = runtime.value_mut()["state"]["snapshot_v2"]["public_observation_frames"]
            .as_array_mut().ok_or("E_PROJECT_RUNTIME_OBSERVATION_FRAMES")?;
        for frame in frames {
            let object = frame.as_object_mut().ok_or("E_PROJECT_RUNTIME_OBSERVATION_FRAME")?;
            if object.get("source_identity").and_then(JsonValue::as_str)!=Some(expected_source.as_str()) {
                return Err("E_PROJECT_RUNTIME_OBSERVATION_SOURCE: 실제 관찰 원문이 다릅니다".into());
            }
            for key in ["run_id","source_identity","revision_identity"] { object.remove(key); }
        }
        let resources = crate::unit_wire::serialize_resource_snapshot_json(
            runtime.pointer("/state/snapshot_v2/resources").ok_or("E_PROJECT_RUNTIME_RESOURCE_SCHEMA")?);
        crate::unit_wire::project_runtime_attachment::encode_saved_runtime_artifact_with_frame(
            &self.owned,&runtime.compact_text(),&resources,parent,revision,frame)
    }
}

/// Preserve actual snapshots/edges directly from the shared runner. Never
/// reconstruct rows by parsing observation JSON or reinterpret a logical Tape.
pub(crate) fn run_authored_general_input_prefix(
    artifact: &str, requested_ticks: u32, base_seed: u64,
    input_snapshots: Option<&[InputSnapshot]>, input_tape: Option<&InputTape>,
) -> Result<GeneralSuccessfulInputPrefix, String> {
    run_authored_general_input_prefix_with_age(artifact,requested_ticks,base_seed,input_snapshots,input_tape,
        crate::ddn_runtime::default_age_target())
}
fn run_authored_general_input_prefix_with_age(
    artifact:&str,requested_ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,
    input_tape:Option<&InputTape>,age:AgeTarget,
)->Result<GeneralSuccessfulInputPrefix,String> {
    let owned = crate::project_revision_envelope_supply::admit_fresh_authored_project(artifact.as_bytes())?;
    run_general_input_prefix_from_owned(owned,artifact,requested_ticks,base_seed,input_snapshots,input_tape,age)
}
/// Reuse the retained source captures for staging. Re-admitting fresh source
/// would issue new capture ordinals/PESI and lose original execution custody.
fn run_general_input_prefix_from_owned(
    owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    artifact:&str,requested_ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,
    input_tape:Option<&InputTape>,age:AgeTarget,
)->Result<GeneralSuccessfulInputPrefix,String> {
    run_general_input_prefix_from_owned_with_subject(owned,artifact,requested_ticks,base_seed,input_snapshots,input_tape,age,None,None)
}
fn run_general_input_prefix_from_owned_with_subject(
    owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    artifact:&str,requested_ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,
    input_tape:Option<&InputTape>,age:AgeTarget,
    subject_bootstrap:Option<&crate::unit_wire::subject_execution::SubjectExecutionStage>,
    replay_source:Option<&crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision>,
)->Result<GeneralSuccessfulInputPrefix,String> {
    run_general_input_prefix_with_evidence(owned,artifact,requested_ticks,base_seed,input_snapshots,input_tape,age,
        subject_bootstrap,replay_source.map(GeneralEvidenceSourceV1::Original))
}
fn run_general_input_prefix_with_evidence(
    owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    artifact:&str,requested_ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,
    input_tape:Option<&InputTape>,age:AgeTarget,
    subject_bootstrap:Option<&crate::unit_wire::subject_execution::SubjectExecutionStage>,
    replay_source:Option<GeneralEvidenceSourceV1<'_>>,
)->Result<GeneralSuccessfulInputPrefix,String> {
    let preparation = owned.sources().preparation();
    let profile = resolve_supported_source_profile(Some("v1-core-v25")).map_err(|error|error.message)?;
    let mut rows = Vec::new();
    let mut execution = None;
    let summary = run_summary_with_general_input_supplier(
        &preparation.source,&preparation.target_path,Some(requested_ticks),None,None,Some(age),
        Some(profile),input_tape,base_seed,owned.sources().entry_declaration().name(),
        None,true,input_snapshots,None,None,Some(artifact),Some(&mut rows),Some(&owned),Some(GeneralExecutionCollector{destination:&mut execution,source:replay_source}),subject_bootstrap,
    ).map_err(|error|error.to_json_value().to_string())?;
    let result = GeneralSuccessfulInputPrefix { owned, rows, base_seed, summary:RuntimeJsonOutput::new(summary),
        execution:execution.ok_or("E_PROJECT_RUNTIME_GENERAL_BOUNDARY: 성공 실행 상태가 없습니다")?,
        construction:GeneralInputConstruction {requested_ticks,age,subject_bootstrap:subject_bootstrap.cloned(),
            snapshots:input_snapshots.map(<[InputSnapshot]>::to_vec),tape:input_tape.cloned()} };
    // Wrong actual seeds and unsupported sources only reject this new save
    // supplier; the existing executable frontdoors retain their semantics.
    result.input_supply().map_err(|error|format!("E_PROJECT_RUNTIME_INPUT_SUPPLY: 실제 입력 저장 결속 실패 ({error:?})"))?;
    Ok(result)
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
    let mut summary = crate::unit_wire::SnapshotJsonOwner::from_value(run_summary_from_supported_source_profile(
        source,
        file_path,
        requested_ticks,
        source_profile_identity,
    )?);
    attach_host_resource_receipt(summary.value_mut(), host_resources);
    Ok(summary.into_parent_value())
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
    live_next_mask: Option<(u32, &mut dyn FnMut(u64) -> Result<InputRecord, String>)>,
    named_entry_after_definitions: bool,
    input_snapshots: Option<&[InputSnapshot]>,
    param_overrides: Option<&BTreeMap<String, Value>>,
    bogae_entry: Option<&str>,
    authored_artifact: Option<&str>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_with_general_input_supplier(source,file_path,requested_ticks,
        project_asset_registry,standard_limits,age_target,source_profile,input_tape,
        rng_base_seed,update_name,live_next_mask,named_entry_after_definitions,
        input_snapshots,param_overrides,bogae_entry,authored_artifact,None,None,None,None)
}

/// Optional internal supplier shares the actual product loop. Public summaries
/// and old callers are unchanged. A failed run never publishes a partial prefix.
fn run_summary_with_general_input_supplier(
    source: &str, file_path: &str, requested_ticks: Option<u32>,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
    standard_limits: Option<StandardResourceLimitsV1>, age_target: Option<AgeTarget>,
    source_profile: Option<SupportedSourceProfile>, input_tape: Option<&InputTape>,
    rng_base_seed: u64, update_name: &str,
    mut live_next_mask: Option<(u32, &mut dyn FnMut(u64) -> Result<InputRecord, String>)>,
    named_entry_after_definitions: bool, input_snapshots: Option<&[InputSnapshot]>,
    param_overrides: Option<&BTreeMap<String, Value>>, bogae_entry: Option<&str>,
    authored_artifact: Option<&str>,
    successful_inputs: Option<&mut Vec<crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow>>,
    general_saved_owner: Option<&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>,
    completed_execution: Option<GeneralExecutionCollector<'_>>,
    subject_bootstrap:Option<&crate::unit_wire::subject_execution::SubjectExecutionStage>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    let mut staged_inputs = Vec::new();
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
    let (prepared, mut program) = if let Some(owned) = general_saved_owner {
        let mut program = DdnProgram::from_saved_v25_project_with_mode(
            owned.sources().clone(),ParseMode::Strict,effective_age_target,
        ).map_err(|message|run_summary_diagnostic("E_PROJECT_RUNTIME_SOURCE",message,file_path,"saved_source"))?;
        program.bind_registered_project_context(owned)
            .map_err(|message|run_summary_diagnostic("E_SCHEMA_REGISTERED_CONTEXT_REQUIRED",message,file_path,"saved_source"))?;
        let prepared = prepare_supported_source_for_execution_with_program(source,file_path,&program)?;
        (prepared,program)
    } else { match source_profile {
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
                } else if message.contains("E_RUNTIME_TYPE_MISMATCH") {
                    "E_RUNTIME_TYPE_MISMATCH"
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
    }};
    crate::source_analysis::validate_program_for_execution(program.canonical_program())
        .map_err(|message| {
            run_summary_diagnostic("E_STATIC_VALIDATION", message, file_path, "static_validation")
        })?;
    if general_saved_owner.is_none(){
        program.prepare_temporary_source_execution().map_err(|message|
            run_summary_diagnostic("E_VC04_TEMPORARY_SOURCE",message,file_path,"source_registration"))?;
    }
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
    if let Some(stage)=subject_bootstrap {runner.bind_subject_bootstrap(stage.clone());}
    if let Some(owned) = general_saved_owner {
        runner.configure_project_entry(owned.sources().entry_declaration().name()).map_err(|message|
            run_summary_diagnostic("E_PROJECT_RUNTIME_ENTRY",message,file_path,"saved_source"))?;
        let project = owned.project();
        if project.schema == crate::authored_project::SCHEMA_V2
            || project.authored_identity.modules.iter().any(|module|!module.compositions.is_empty()) {
            runner.configure_authored_bogae(authored_artifact.ok_or_else(||run_summary_diagnostic(
                "E_PROJECT_RUNTIME_SOURCE", "작품 작성 자료가 없습니다",file_path,"saved_source"))?).map_err(|message|
                run_summary_diagnostic("E_AUTHORED_BOGAE_ADMISSION",message,file_path,"authored_bogae"))?;
        }
    } else if let Some(artifact) = authored_artifact {
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

    // Capture only here, before the first actual input, from the original
    // source-only owner. A legacy/cold prefix never manufactures this proof.
    let mut result_collector=match completed_execution.as_ref().and_then(|collector|collector.source){
        Some(original)=>Some(GeneralResultCollectorV1::from_source(general_saved_owner.ok_or_else(||
            run_summary_diagnostic("E_GENERAL_REPLAY_SEED_SOURCE","원 실행의 원문 소유자가 없습니다",file_path,"result_evidence"))?,
            &runner,original,rng_base_seed,effective_age_target)
            .map_err(|message|run_summary_diagnostic("E_GENERAL_REPLAY_SEED",message,file_path,"result_evidence"))?),
        None=>None,
    };
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
        let actual_receipt=crate::input_receipt::receipt(&input,&output.input_reads);
        let prepared_result=match result_collector.as_mut(){
            Some(collector)=>Some(collector.prepare(output.result_emissions,tick_id,&actual_receipt)
                .map_err(|message|run_summary_diagnostic("E_GENERAL_RESULT_OUTPUT",message,file_path,"result_evidence"))?),
            None=>None,
        };
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
        input_receipts.push(actual_receipt);
        if successful_inputs.is_some() {
            staged_inputs.try_reserve(1).map_err(|_|run_summary_diagnostic(
                "E_PROJECT_RUNTIME_INPUT_SUPPLY", "실제 처리 입력을 보존할 공간이 없습니다",
                file_path, "input_supply"))?;
            staged_inputs.push(crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow {
                input: input.clone(), edges: control_edges,
            });
        }
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
        if let (Some(collector),Some(prepared))=(result_collector.as_mut(),prepared_result){
            collector.commit(prepared).map_err(|message|run_summary_diagnostic("E_GENERAL_RESULT_OUTPUT",message,file_path,"result_evidence"))?;
        }
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
    let mut state = crate::unit_wire::SnapshotJsonOwner::from_value(
        build_engine_state_json(&world, tick_id, last_input, &last_patch));
    crate::bogae_projection::attach(state.value_mut(), last_bogae_projection.as_ref());
    crate::output_display::attach(state.value_mut(), &display_outputs);
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
    if let Some(state_map) = state.value_mut().as_object_mut() {
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
    if let Some(state_map) = state.value_mut().as_object_mut() {
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
    let mut summary = crate::unit_wire::SnapshotJsonOwner::from_value(json!({
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
    }));
    // engine_state는 이미 완성된 공개 JSON이다. 동일 슬롯으로 옮겨 큰 상태의
    // json! to_value 재변환을 피한다. 공개 사본·필드·bytes는 바꾸지 않는다.
    for key in ["resources", "channels", "row"] {
        summary.value_mut()[key] = state.get(key).map(|value|
            crate::unit_wire::SnapshotJsonOwner::clone_from_value(value).into_parent_value()).unwrap_or_else(|| {
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
        summary.value_mut()[key] = value;
    }
    summary.value_mut()["output_log_texts"] = JsonValue::Array(
        output_log_texts.into_iter().map(JsonValue::String).collect());
    summary.value_mut()["engine_state"] = state.into_parent_value();
    crate::bogae_projection::attach(summary.value_mut(), last_bogae_projection.as_ref());
    crate::output_display::attach(summary.value_mut(), &display_outputs);
    if let Some(age_target) = age_target {
        summary.value_mut().as_object_mut().expect("summary object").insert(
            "age_target_identity".to_string(),
            JsonValue::String(age_target.label().to_string()),
        );
    }
    let summary = finalize_run_summary_result_artifact(summary, &mut runner, file_path)?;
    if let Some(destination) = successful_inputs { *destination = staged_inputs; }
    if let Some(collector) = completed_execution { *collector.destination = Some(GeneralCompletedExecution { runner, world, next_tick:tick_id,result:result_collector,saved_runtime:None }); }
    Ok(summary)
}

fn finalize_run_summary_result_artifact(
    mut summary: crate::unit_wire::SnapshotJsonOwner,
    runner: &mut DdnRunner,
    file_path: &str,
) -> Result<JsonValue, ExecutionDiagnostic> {
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
            return Ok(summary.into_parent_value());
        }
        runner.set_result_artifact_bytes(bytes).map_err(|failure| {
            work_cost_failure_diagnostic(failure, file_path, "resource_commit")
        })?;
        summary.value_mut()["work_cost_receipt"] =
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

/// Owns an already-produced runtime response through CLI output and error cleanup.
/// This is not a source/state admission token and grants no saved-state authority.
pub struct RuntimeJsonOutput(crate::unit_wire::SnapshotJsonOwner);

impl RuntimeJsonOutput {
    pub fn new(value: JsonValue) -> Self {
        Self(crate::unit_wire::SnapshotJsonOwner::from_value(value))
    }
    pub fn pretty_text(&self) -> Result<String, serde_json::Error> {
        let mut bytes = Vec::new();
        crate::unit_wire::write_pretty_json(&mut bytes, &self.0)?;
        Ok(String::from_utf8(bytes).expect("JSON is UTF-8"))
    }
}

impl std::ops::Deref for RuntimeJsonOutput {
    type Target = JsonValue;
    fn deref(&self) -> &Self::Target { &self.0 }
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
    crate::unit_wire::write_compact_json(&mut counter, value)?;
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
    let input_json = crate::unit_wire::SnapshotJsonOwner::from_value(input_json);
    let committed_madi = tick_id.saturating_sub(1);
    let resources = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_world_resources(world.world()));
    let (columns, row) = collect_columns_and_row(world.world());
    let view_meta = crate::unit_wire::SnapshotJsonOwner::from_value(build_view_meta(world.world()));
    let view_hash = hash_json_string(&view_meta);
    let patch_json = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_patch(patch));
    let mut result = crate::unit_wire::SnapshotJsonOwner::from_value(json!({
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
    }));
    // 이미 정본 JSON인 값은 다시 직렬화하지 않는다. 공개 두 경로를 그대로
    // 유지하고 각 값의 필수 사본 하나만 만든 뒤 원 값을 중첩 상태로 옮긴다.
    for (key, value) in [("input", input_json), ("resources", resources),
        ("channels", crate::unit_wire::SnapshotJsonOwner::from_value(JsonValue::Array(columns))),
        ("row", crate::unit_wire::SnapshotJsonOwner::from_value(JsonValue::Array(row)))] {
        result.value_mut()[key] = value.clone().into_parent_value();
        result.value_mut()["state"][key] = value.into_parent_value();
    }
    result.value_mut()["view_meta"] = view_meta.into_parent_value();
    result.value_mut()["patch"] = patch_json.clone().into_parent_value();
    result.value_mut()["state"]["patch"] = patch_json.into_parent_value();
    result.into_parent_value()
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

/// R1.2: the actual host-default producer also owns its static declaration.
/// Not deserializable, not an ambient registry or caller-supplied JSON flag.
#[derive(Clone,Copy)]
pub(crate) struct BogaeHostNumericSupply(());
pub(crate) fn bogae_host_numeric_supply()->BogaeHostNumericSupply {BogaeHostNumericSupply(())}
impl BogaeHostNumericSupply {
    pub(crate) fn frame_initial(self)->Value {Value::Fixed64(Fixed64::ZERO)}
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
    defaults.insert("프레임수".to_string(), bogae_host_numeric_supply().frame_initial());
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

pub(crate) fn runtime_state_snapshot(
    world: &DetNuri,
    public_observation_frames: Vec<JsonValue>,
    next_tick_id: u64,
) -> JsonValue {
    JsonValue::Object([
        ("schema".into(), "ddn.wasm_state_snapshot.v2".into()),
        ("resource_snapshot_schema".into(), RESOURCE_SNAPSHOT_SCHEMA_V2.into()),
        ("unit_wire_schema".into(), UNIT_VALUE_WIRE_SCHEMA_V2.into()),
        ("resources".into(), serialize_world_resources_snapshot(world.world())),
        ("param_overrides".into(), JsonValue::Object(Map::new())),
        ("public_observation_frames".into(), JsonValue::Array(public_observation_frames)),
        ("next_tick_id".into(), next_tick_id.into()),
    ].into_iter().collect())
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
                items.push(resource_value_patch_json(tag, value));
            }
            PatchOp::SetSubjectWorld { value } => {
                items.push(json!({"op":"set_subject_world","subject_values_sha256":sha256_hex(&value.subject_values),
                    "subject_custody_sha256":sha256_hex(&value.subject_custody)}));
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

pub(crate) fn resource_value_patch_json(tag: &str, value: &ResourceValue) -> JsonValue {
    JsonValue::Object([
        ("op".into(), JsonValue::String("set_resource_value".into())),
        ("tag".into(), JsonValue::String(tag.into())),
        ("value".into(), JsonValue::String(value.canon_key())),
        ("value_json".into(), resource_value_to_json(value)),
    ].into_iter().collect())
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

pub(crate) fn resource_value_to_json(value: &ResourceValue) -> JsonValue {
    // Display projection has the same arbitrary saved-tree nesting as detjson.
    // Keep completed children owned until they are transferred into a parent.
    enum Job<'a> { Value(&'a ResourceValue), Array(usize), Object(Vec<String>) }
    let mut jobs = vec![Job::Value(value)];
    let mut built = Vec::<crate::unit_wire::SnapshotJsonOwner>::new();
    while let Some(job) = jobs.pop() {
        match job {
            Job::Array(count) => {
                let children = built.split_off(built.len() - count);
                built.push(crate::unit_wire::SnapshotJsonOwner::from_value(JsonValue::Array(
                    children.into_iter().map(|child| child.into_parent_value()).collect())));
            }
            Job::Object(keys) => {
                let children = built.split_off(built.len() - keys.len());
                let mut object = Map::new();
                for (key, child) in keys.into_iter().zip(children) {
                    // Display keys can collide. Do not recursively drop the displaced value.
                    if let Some(previous) = object.insert(key, child.into_parent_value()) {
                        drop(crate::unit_wire::SnapshotJsonOwner::from_value(previous));
                    }
                }
                built.push(crate::unit_wire::SnapshotJsonOwner::from_value(JsonValue::Object(object)));
            }
            Job::Value(value) => {
                if let Some(formula) = formula_from_resource_value(value) {
                    let (dialect, raw) = formula_summary_parts(&formula);
                    built.push(crate::unit_wire::SnapshotJsonOwner::from_value(json!({"dialect":dialect,"raw":raw})));
                    continue;
                }
                match value {
                    ResourceValue::List(items) => {
                        jobs.push(Job::Array(items.len()));
                        jobs.extend(items.iter().rev().map(Job::Value));
                    }
                    ResourceValue::Set(items) => {
                        jobs.push(Job::Array(items.len()));
                        jobs.extend(items.values().rev().map(Job::Value));
                    }
                    ResourceValue::Map(entries) => {
                        jobs.push(Job::Object(entries.values().map(|entry| resource_key_to_string(&entry.key)).collect()));
                        jobs.extend(entries.values().rev().map(|entry| Job::Value(&entry.value)));
                    }
                    _ => built.push(crate::unit_wire::SnapshotJsonOwner::from_value(resource_value_leaf_to_json(value))),
                }
            }
        }
    }
    built.pop().expect("display root").into_parent_value()
}

fn resource_value_leaf_to_json(value: &ResourceValue) -> JsonValue {
    if let Some(formula) = formula_from_resource_value(value) {
        let (dialect, raw) = formula_summary_parts(&formula);
        return json!({
            "dialect": dialect,
            "raw": raw,
        });
    }
    match value {
        ResourceValue::Record { .. } | ResourceValue::Tuple { .. } | ResourceValue::RationalV1(_) => crate::unit_wire::resource_value_to_detjson(value),
        ResourceValue::IdentityReference(_) | ResourceValue::RegisteredAtom(_) | ResourceValue::Bytes(_) => crate::unit_wire::resource_value_to_detjson(value),
        ResourceValue::BasicTensorV1(_) => crate::unit_wire::resource_value_to_detjson(value),
        ResourceValue::NominalValueV1(_) | ResourceValue::SubjectHandleV1(_) | ResourceValue::SubjectPayloadV1(_) => crate::unit_wire::resource_value_to_detjson(value),
        ResourceValue::None => JsonValue::Null,
        ResourceValue::Bool(v) => JsonValue::Bool(*v),
        ResourceValue::Fixed64(v) => json!(fixed64_to_f64(*v)),
        ResourceValue::Unit(v) => json!(fixed64_to_f64(v.value)),
        ResourceValue::String(s) => JsonValue::String(s.clone()),
        ResourceValue::ResourceHandle(handle) => JsonValue::String(handle_to_string(*handle)),
        ResourceValue::List(_) | ResourceValue::Set(_) | ResourceValue::Map(_) => unreachable!("container is scheduled"),
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
        if let ResourceValue::List(items) = &draw_list_value {
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
    let ResourceValue::List(items) = &list else {
        return None;
    };
    let mut draw_items = Vec::new();
    for item in items {
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
    let ResourceValue::List(items) = &list else {
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
    let mut canonical = Vec::new();
    crate::unit_wire::write_compact_json(&mut canonical, value).expect("JSON Vec writer");
    let digest = blake3_hash(&canonical);
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
    #[test]
    fn authored_single_file_bogae_runs_without_a_runtime_storage_root() {
        use super::*;
        let source=include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha2_linear_composition.ddn");
        let request=json!({"source_profile":"v1-core-v25","module_name":"앱",
            "source_snapshot":{"entry_path":"lesson.ddn","files":[{"path":"lesson.ddn","text":source}]}});
        let original=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let registered=crate::authored_project::operate("register_bogae",&json!({
            "original_json":original,"module_name":"앱","source_path":"lesson.ddn",
            "composition_definition":"일차함수짜임","make_entry":true}).to_string()).unwrap();
        let stored=crate::authored_project::open(registered.as_bytes()).unwrap();
        assert!(stored.source_snapshot.root_path.is_none());
        let temporary=run_summary_with_bogae_entry(source,"lesson.ddn",Some(1),"v1-core-v25","일차함수짜임").unwrap();
        let actual=run_summary_with_authored_bogae(source,"lesson.ddn",Some(1),"v1-core-v25",&registered,None,None).unwrap();
        assert_eq!(actual["state_hash"],temporary["state_hash"]);
        assert_eq!(actual["bogae_projection"]["persistent_identity"],true);
        assert!(crate::project_revision_envelope_supply::admit_fresh_authored_project(registered.as_bytes())
            .unwrap_err().starts_with("E_PROJECT_RUNTIME_ROOT_REQUIRED:"),"execution must not invent storage custody");
    }
    #[cfg(not(target_arch="wasm32"))]
    #[test]
    fn general_native_actual_sqlite_save_cold_restore_and_stale_failure_preserves_old_copy() {
        use super::*;
        use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":"채비 { 값:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }."}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root=std::env::temp_dir().join(format!("ddn-general-native-{}-{nonce}",std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let saved=record_authored_general_project_native(&authored,2,7,&[],&[9],&root,true,None).unwrap();
        let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(saved.saved_artifact.as_bytes()).unwrap();
        let owned=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes).unwrap();
        let project=hex::decode(owned.project().authored_identity.logical_dependency_id.as_str()).unwrap();
        let store=SqliteAdapterV1::open_admitted(&root).unwrap();
        let before=store.reread_committed(&saved.transaction_digest,&project,&[9]).unwrap();
        drop(store); // release the single-writer root lease before cold reopen
        let resumed=resume_saved_general_project_artifact_native(
            &saved.saved_artifact,&[],&[9],&root,&saved.transaction_digest,2).unwrap();
        let uninterrupted=run_authored_general_input_prefix(&authored,4,7,None,None).unwrap();
        let retained=run_general_input_prefix_from_owned(owned.clone(),&authored,2,7,None,None,
            crate::ddn_runtime::default_age_target()).unwrap();
        let previous_work=retained.execution.runner.work_cost_receipt();
        let (_,mut continued,actual_inputs)=resume_saved_project_artifact_from_runner(
            &saved.saved_artifact,&[],&[9],2,Some((retained.execution.runner,crate::ddn_runtime::default_age_target()))).unwrap();
        let work=continued.runner.work_cost_receipt();
        assert!(work.counters.base_dispatches>previous_work.counters.base_dispatches);
        assert_eq!(work.counters.base_dispatches,uninterrupted.execution.runner.work_cost_receipt().counters.base_dispatches);
        assert_eq!(work.counters.committed_madi,4);
        assert_eq!(work.counters.sealed_input_bytes,previous_work.counters.sealed_input_bytes);
        assert_eq!(work.counters.call_depth_current,0);
        assert_eq!(continued.next_tick,4);
        assert_eq!(actual_inputs,uninterrupted.rows[2..]);
        assert_eq!(current_state_hash(continued.world.world()),uninterrupted.summary()["state_hash"]);
        // A continued carrier uses the actual retained world/runner, not a
        // newly admitted fresh source or a copied pre-continuation carrier.
        let second=continued.export_continued_artifact(&owned,&[9],&[10]).unwrap();
        let second_bytes=crate::unit_wire::project_runtime_attachment::check_bytes(second.as_bytes()).unwrap();
        let second_owned=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&second_bytes).unwrap();
        let second_context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context(
            &second_bytes,&second_owned,&[9],&[10]).unwrap();
        assert_eq!(second_context.pesi().as_raw(),crate::project_snapshot_identity::current::issue_saved_local_project(&owned).unwrap().pesi().as_raw());
        let after_second=resume_saved_project_artifact(&second,&[9],&[10],2).unwrap();
        let six=run_authored_general_input_prefix(&authored,6,7,None,None).unwrap();
        assert_eq!(after_second["next_tick_id"],6);
        assert_eq!(after_second["state_hash"],six.summary()["state_hash"]);
        assert_eq!(after_second["resources"],six.summary()["engine_state"]["resources"]);
        let original_runtime=continued.saved_runtime.as_ref().unwrap().clone();
        let unchanged_world=current_state_hash(continued.world.world());
        continued.next_tick+=1;
        assert!(continued.export_continued_artifact(&owned,&[9],&[10]).is_err());
        continued.next_tick-=1;
        assert_eq!(continued.export_continued_artifact(&owned,&[9],&[10]).unwrap(),second);
        continued.saved_runtime.as_mut().unwrap().value_mut()["state"]["snapshot_v2"]["resources"]["fixed64"]["값"]="999".into();
        assert!(continued.export_continued_artifact(&owned,&[9],&[10]).is_err());
        assert_eq!(current_state_hash(continued.world.world()),unchanged_world);
        continued.saved_runtime=Some(original_runtime);
        assert_eq!(continued.export_continued_artifact(&owned,&[9],&[10]).unwrap(),second);
        assert_eq!(resumed["next_tick_id"],4);
        assert_eq!(resumed["state_hash"],uninterrupted.summary()["state_hash"]);
        let input=crate::input_receipt::parse_receipt_snapshot(&resumed["frames"][0]["input_receipt"]).unwrap();
        assert_eq!(input.tick_id,2);assert_eq!(input.rng_seed,7^2);
        // A fresh stale project save may prepare a candidate, but cannot
        // replace the old head or return a partially published success.
        assert!(record_authored_general_project_native(&authored,1,7,&[],&[10],&root,false,None).is_err());
        let store=SqliteAdapterV1::open_admitted(&root).unwrap();
        let after=store.reread_committed(&saved.transaction_digest,&project,&[9]).unwrap();
        assert_eq!(after.payload,before.payload);
        assert_eq!(after.revision,before.revision);
        assert_eq!(after.head,before.head);
        drop(store);
        assert!(resume_saved_general_project_artifact_native(&saved.saved_artifact,&[],&[9],
            &root,&[0;32],2).is_err());
        let store=SqliteAdapterV1::open_admitted(&root).unwrap();
        let after_rejection=store.reread_committed(&saved.transaction_digest,&project,&[9]).unwrap();
        assert_eq!(after_rejection.payload,before.payload);
        assert_eq!(after_rejection.revision,before.revision);
        assert_eq!(after_rejection.head,before.head);
        drop(store);
        let second_native=resume_and_save_general_project_native(&saved.saved_artifact,&[],&[9],
            &root,&saved.transaction_digest,2,&[10],&saved.committed_receipt).unwrap();
        assert_eq!(second_native.summary["state_hash"],uninterrupted.summary()["state_hash"]);
        let native_six=resume_saved_general_project_artifact_native(&second_native.saved_artifact,&[9],&[10],
            &root,&second_native.transaction_digest,2).unwrap();
        assert_eq!(native_six["state_hash"],six.summary()["state_hash"]);
        assert_eq!(native_six["next_tick_id"],6);
        assert!(resume_and_save_general_project_native(&second_native.saved_artifact,&[9],&[10],
            &root,&second_native.transaction_digest,1,&[11],&saved.committed_receipt).is_err());
        let third_native=resume_and_save_general_project_native(&second_native.saved_artifact,&[9],&[10],
            &root,&second_native.transaction_digest,1,&[11],&second_native.committed_receipt).unwrap();
        let third_six=resume_saved_general_project_artifact_native(&third_native.saved_artifact,&[10],&[11],
            &root,&third_native.transaction_digest,1).unwrap();
        assert_eq!(third_six["state_hash"],six.summary()["state_hash"]);
        assert_eq!(third_six["resources"],six.summary()["engine_state"]["resources"]);
        // Keep this small owned test store as evidence; no shared path removal.
        eprintln!("general Native evidence store: {}",root.display());
    }

    /// Real source/capture/input admission and SQLite persistence, with a real
    /// forced child-process kill. This is a test-build Native execution proof,
    /// not a shipped CLI3, power-loss or recovery-record/receipt oracle.
    #[cfg(windows)]
    #[test]
    fn native_general_save_forced_process_kill_preserves_old_and_raw_new() {
        use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
        use crate::unit_wire::project_runtime_attachment as attachment;
        const ROOT: &str = "DDN_NATIVE_OWNER_TEST_ROOT";
        if let Some(root) = std::env::var_os(ROOT) {
            let root = std::path::PathBuf::from(root);
            let authored = std::fs::read_to_string(root.join("authored.fixture")).unwrap();
            let receipt = std::fs::read(root.join("old-receipt.fixture")).unwrap();
            record_authored_general_project_native(&authored,4,7,&[9],&[10],&root,false,Some(&receipt)).unwrap();
            std::fs::write(root.join("unexpected-success.fixture"), b"published").unwrap();
            panic!("interrupted transaction unexpectedly returned success");
        }
        // Other ddn_runtime unit tests temporarily change the process-global
        // default AGE. Their mutex is not shared with this module, and a new
        // child starts with its own default. Run the coordinator in its own
        // exact-test process so both producers have identical execution policy.
        // Never relax a digest assertion to mask different execution profiles.
        if std::env::var_os("DDN_NATIVE_OWNER_TEST_COORDINATOR").is_none() {
            let result=std::process::Command::new(std::env::current_exe().unwrap())
                .arg("--exact").arg("runtime_surface::tests::native_general_save_forced_process_kill_preserves_old_and_raw_new")
                .arg("--nocapture").env("DDN_NATIVE_OWNER_TEST_COORDINATOR","1").output().unwrap();
            eprintln!("{}",String::from_utf8_lossy(&result.stdout));
            assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
            return;
        }
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":"채비 { 값:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }."}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let four=run_authored_general_input_prefix(&authored,4,7,None,None).unwrap();
        let candidate=four.export_saved_artifact(&[9],&[10]).unwrap();
        let checked=attachment::check_bytes(candidate.as_bytes()).unwrap();
        let context=attachment::check_saved_revision_context(&checked,four.source_owner(),&[9],&[10]).unwrap();
        let raw=four.encode_checkpoint_request(&context,&[9],&[10]).unwrap();
        let admitted=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2(&raw,context.pesi().as_raw()).unwrap();
        let next_digest=admitted.binding.transaction_digest().unwrap();
        let project=admitted.binding.project.clone();
        for phase in ["before_prepared_commit","after_prepared_commit","after_head_commit","after_container_confirmation"] {
            let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let root=std::env::temp_dir().join(format!("ddn-native-forced-kill-{}-{nonce}",std::process::id()));
            std::fs::create_dir(&root).unwrap();
            let old=record_authored_general_project_native(&authored,2,7,&[],&[9],&root,true,None).unwrap();
            let adapter=SqliteAdapterV1::open_admitted(&root).unwrap();
            let before=adapter.reread_committed(&old.transaction_digest,&project,&[9]).unwrap();
            drop(adapter);
            // Test evidence only, never adapter recovery metadata or authority.
            std::fs::write(root.join("authored.fixture"),&authored).unwrap();
            std::fs::write(root.join("old-receipt.fixture"),&old.committed_receipt).unwrap();
            std::fs::write(root.join("expected-request.fixture"),&raw).unwrap();
            let marker=root.join("paused.fixture");
            let mut child=std::process::Command::new(std::env::current_exe().unwrap())
                .arg("--exact").arg("runtime_surface::tests::native_general_save_forced_process_kill_preserves_old_and_raw_new")
                .env(ROOT,&root).env("DDN_NATIVE_OWNER_TEST_PHASE",phase).env("DDN_NATIVE_OWNER_TEST_MARKER",&marker)
                .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().unwrap();
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(15);
            while !marker.exists() && std::time::Instant::now()<deadline && child.try_wait().unwrap().is_none() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let reached=marker.exists();
            // Always reap the test-owned child, including the timeout path.
            if child.try_wait().unwrap().is_none() { child.kill().unwrap(); }
            let status=child.wait().unwrap();
            assert!(reached,"child did not reach {phase}; {status}");
            assert!(!status.success());
            assert!(!root.join("unexpected-success.fixture").exists());
            let adapter=SqliteAdapterV1::open_admitted(&root).unwrap();
            let after=adapter.reread_committed(&old.transaction_digest,&project,&[9]).unwrap();
            assert_eq!(after.payload,before.payload,"old payload preserved: {phase}");
            assert_eq!(after.revision,before.revision,"old revision preserved: {phase}");
            if phase=="before_prepared_commit" || phase=="after_prepared_commit" {
                assert_eq!(after.head,before.head,"CAS has not occurred: {phase}");
                drop(adapter);
                let resumed=resume_saved_general_project_artifact_native(&old.saved_artifact,&[],&[9],&root,&old.transaction_digest,2).unwrap();
                assert_eq!(resumed["state_hash"],four.summary()["state_hash"]);
            } else {
                let next=adapter.reread_committed(&next_digest,&project,&[10]).unwrap();
                assert_eq!(next.head,next.revision);
                let head=next.head.as_ref().unwrap();
                if head.digest!=next_digest {
                    let actual=adapter.reread_prepared(&head.digest).unwrap().unwrap();
                    eprintln!("expected binding {:?}; actual binding {:?}; root {}",admitted.binding,actual.binding,root.display());
                    std::fs::write(root.join("actual-snapshot.fixture"),actual.snapshot).unwrap();
                }
                assert_eq!(head.digest,next_digest);
                drop(adapter);
                // NEW_UNCONFIRMED only. Do not call ordinary restore to bypass
                // RD-669's retained recovery record/token admission. Their
                // Native persisted producer is still an explicit open gate.
            }
            eprintln!("forced termination {phase}: {}",root.display());
        }
    }
    #[cfg(not(target_arch="wasm32"))]
    #[test]
    fn n1_rational_tuple_native_sqlite_save_reopen_continue_resave() {
        let body="채비 { 진행 := 0. }. 정확:나눔수 := 0.1. 실행:움직씨 := { 진행 <- 진행+1. 묶음값 <- (진행,정확,참). 묶음값[1] 보여주기. }.";
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/root.ddn")},
                {"path":"main.ddn","text":body}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root=std::env::temp_dir().join(format!("ddn-rational-native-{}-{nonce}",std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let saved=record_authored_general_project_native(&authored,3,7,&[],&[9],&root,true,None).unwrap();
        let resumed=resume_saved_general_project_artifact_native(&saved.saved_artifact,&[],&[9],&root,&saved.transaction_digest,4).unwrap();
        let continuous=run_authored_general_input_prefix(&authored,7,7,None,None).unwrap();
        assert_eq!(resumed["state_hash"],continuous.summary()["state_hash"]);
        assert_eq!(resumed["resources"],continuous.summary()["engine_state"]["resources"]);
        assert_eq!(resumed["next_tick_id"],7);
        let second=resume_and_save_general_project_native(&saved.saved_artifact,&[],&[9],&root,
            &saved.transaction_digest,4,&[10],&saved.committed_receipt).unwrap();
        let reopened=resume_saved_general_project_artifact_native(&second.saved_artifact,&[9],&[10],&root,&second.transaction_digest,1).unwrap();
        let eight=run_authored_general_input_prefix(&authored,8,7,None,None).unwrap();
        assert_eq!(reopened["state_hash"],eight.summary()["state_hash"]);
        assert_eq!(reopened["resources"],eight.summary()["engine_state"]["resources"]);
        eprintln!("Rational Native evidence store: {}",root.display());
    }

    #[cfg(windows)]
    #[test]
    fn m2_actual_runtime_frontdoor_save_cold_resume_and_refuse_bad_inputs() {
        use super::*;
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/root.ddn")},
                {"path":"main.ddn","text":"채비 { 진행:바른수 := 0. }.\n실행:움직씨 := { (매마디)마다 { 진행 <- 진행 + 1. 진행 보여주기. }. }.\n"}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
        let root=std::env::temp_dir().join(format!("ddn-m2-runtime-{}",hex::encode(nonce)));
        std::fs::create_dir(&root).unwrap();
        let prepared=root.with_extension("prepared");
        let saved=record_authored_general_project_custody_native(&authored,3,17,&[9],&root,Some(&prepared)).unwrap();
        assert_eq!(std::fs::read(prepared.join("request.bin")).unwrap(),saved.recovery_request);
        assert_eq!(std::fs::read_to_string(prepared.join("saved.json")).unwrap(),saved.checkpoint.saved_artifact);
        let resume=|raw:&[u8],token:&[u8;33]|resume_saved_general_project_custody_native(
            &saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,raw,token,4);
        let continued=resume(&saved.recovery_request,&saved.recovery_token).unwrap();
        let continuous=run_authored_general_input_prefix(&authored,7,17,None,None).unwrap();
        assert_eq!(continued["state_hash"],continuous.summary()["state_hash"]);
        assert_eq!(continued["resources"],continuous.summary()["engine_state"]["resources"]);
        assert_eq!(continued["next_tick_id"],7);
        let mut wrong_token=saved.recovery_token;wrong_token[32]^=1;
        assert!(resume(&saved.recovery_request,&wrong_token).is_err());
        let mut wrong_request=saved.recovery_request.clone();*wrong_request.last_mut().unwrap()^=1;
        assert!(resume(&wrong_request,&saved.recovery_token).is_err());
        assert!(record_authored_general_project_custody_native(&authored,3,17,&[9],&root,Some(&prepared)).is_err(),"existing root must not be re-enrolled");
        let unchanged=resume(&saved.recovery_request,&saved.recovery_token).unwrap();
        assert_eq!(unchanged["state_hash"],continued["state_hash"]);
        println!("M2_RUNTIME_FRONTDOOR_ROOT {}",root.display());
    }

    #[cfg(windows)]
    #[test]
    #[ignore="only an explicitly spawned interruption child enters this test"]
    fn m2_actual_runtime_interruption_child() {
        let directory=std::path::PathBuf::from(std::env::var_os("DDN_M2_RUNTIME_INTERRUPT_DIR").expect("exact own interruption directory"));
        if std::env::var("DDN_M2_RUNTIME_INTERRUPT_MODE").unwrap()=="initial" {
            let authored=std::fs::read_to_string(directory.join("authored.json")).unwrap();
            super::record_authored_general_project_custody_native(&authored,3,0,&[9],&directory.join("root"),Some(&directory.join("prepared"))).unwrap();
        }else{
            let artifact=std::fs::read_to_string(directory.join("saved.json")).unwrap();
            let raw=std::fs::read(directory.join("request.bin")).unwrap();
            let token:[u8;33]=std::fs::read(directory.join("token.bin")).unwrap().try_into().unwrap();
            let locator=std::fs::read_to_string(directory.join("locator.txt")).unwrap();
            super::resume_saved_general_project_custody_native(&artifact,&[9],&directory.join("root"),&locator,&raw,&token,4).unwrap();
        }
        std::fs::write(directory.join("unexpected-success.fixture"),b"returned").unwrap();
        panic!("owned interruption child unexpectedly returned success");
    }

    #[cfg(windows)]
    #[test]
    fn m2_actual_runtime_frontdoor_logical_tetris_fixed_seed() {
        use super::*;
        use crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1;
        let authored=include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/authored-project.json");
        let initial:crate::logical_input_tape::LogicalRecordRequest=serde_json::from_str(
            include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/initial-input.json")).unwrap();
        let next:crate::logical_input_tape::LogicalRecordRequest=serde_json::from_str(
            include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/next-input.json")).unwrap();
        let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
        let root=std::env::temp_dir().join(format!("ddn-m2-logical-{}",hex::encode(nonce)));std::fs::create_dir(&root).unwrap();
        let prepared=root.with_extension("prepared");
        let saved=record_authored_logical_project_custody_native(authored,&initial,&[9],&root,Some(&prepared)).unwrap();
        let restored=resume_saved_logical_project_custody_native(&saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,
            &saved.recovery_request,&saved.recovery_token,&next).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes()).unwrap();
        let mut continuous=CheckpointedLogicalRunV1::fresh(&owned,initial.binding.clone(),crate::ddn_runtime::default_age_target()).unwrap();
        for snapshot in initial.snapshots.iter().chain(next.snapshots.iter()) {continuous.step(snapshot).unwrap();}
        assert_eq!(restored,continuous.session().summary());
        let mut foreign=next.clone();foreign.binding.initial_rng^=1;
        assert!(resume_saved_logical_project_custody_native(&saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,
            &saved.recovery_request,&saved.recovery_token,&foreign).is_err());
        assert!(resume_saved_general_project_custody_native(&saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,
            &saved.recovery_request,&saved.recovery_token,4).is_err(),"logical history must not enter the general input supplier");
        let again=resume_saved_logical_project_custody_native(&saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,
            &saved.recovery_request,&saved.recovery_token,&next).unwrap();
        assert_eq!(again,restored);
        println!("M2_LOGICAL_RUNTIME_ROOT {}",root.display());
    }

    #[cfg(not(target_arch="wasm32"))]
    #[test]
    fn vc13_tuple_native_sqlite_save_cold_resume_and_resave() {
        use super::*;
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/root.ddn")},
                {"path":"main.ddn","text":include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/main.ddn")} ]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root=std::env::temp_dir().join(format!("ddn-tuple-native-{}-{nonce}",std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let saved=record_authored_general_project_native(&authored,3,7,&[],&[9],&root,true,None).unwrap();
        let resumed=resume_saved_general_project_artifact_native(&saved.saved_artifact,&[],&[9],&root,&saved.transaction_digest,4).unwrap();
        let uninterrupted=run_authored_general_input_prefix(&authored,7,7,None,None).unwrap();
        assert_eq!(resumed["state_hash"],uninterrupted.summary()["state_hash"]);
        assert_eq!(resumed["resources"],uninterrupted.summary()["engine_state"]["resources"]);
        assert_eq!(resumed["next_tick_id"],7);
        assert_eq!(resumed["resources"]["fixed64"]["진행"],"7");
        let second=resume_and_save_general_project_native(&saved.saved_artifact,&[],&[9],&root,
            &saved.transaction_digest,4,&[10],&saved.committed_receipt).unwrap();
        let reopened=resume_saved_general_project_artifact_native(&second.saved_artifact,&[9],&[10],&root,&second.transaction_digest,1).unwrap();
        let eight=run_authored_general_input_prefix(&authored,8,7,None,None).unwrap();
        assert_eq!(reopened["state_hash"],eight.summary()["state_hash"]);
        assert_eq!(reopened["resources"],eight.summary()["engine_state"]["resources"]);
        assert_eq!(reopened["resources"]["fixed64"]["진행"],"8");
        eprintln!("Tuple Native evidence store: {}",root.display());
    }

    #[cfg(not(target_arch="wasm32"))]
    #[test]
    fn vc13_declared_call_tuple_native_cold_resume_and_resave() {
        let body = include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/declared_call.ddn");
        let request = json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/root.ddn")},
                {"path":"main.ddn","text":body}]}});
        let authored = crate::authored_project::operate("create", &request.to_string()).unwrap();
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let store=std::env::temp_dir().join(format!("ddn-declared-call-tuple-{}-{nonce}",std::process::id()));
        std::fs::create_dir(&store).unwrap();
        let saved=record_authored_general_project_native(&authored,3,7,&[],&[9],&store,true,None).unwrap();
        let resumed=resume_saved_general_project_artifact_native(&saved.saved_artifact,&[],&[9],&store,&saved.transaction_digest,4).unwrap();
        let continuous=run_authored_general_input_prefix(&authored,7,7,None,None).unwrap();
        assert_eq!(resumed["state_hash"],continuous.summary()["state_hash"]);
        assert_eq!(resumed["resources"],continuous.summary()["engine_state"]["resources"]);
        let second=resume_and_save_general_project_native(&saved.saved_artifact,&[],&[9],&store,
            &saved.transaction_digest,4,&[10],&saved.committed_receipt).unwrap();
        let reopened=resume_saved_general_project_artifact_native(&second.saved_artifact,&[9],&[10],&store,&second.transaction_digest,1).unwrap();
        let eight=run_authored_general_input_prefix(&authored,8,7,None,None).unwrap();
        assert_eq!(reopened["state_hash"],eight.summary()["state_hash"]);
        assert_eq!(reopened["resources"],eight.summary()["engine_state"]["resources"]);
        eprintln!("declared-call Tuple evidence store: {}",store.display());
    }

    #[test]
    fn general_input_supplier_uses_actual_runner_rows_and_never_publishes_failed_prefix() {
        use super::*;
        use crate::input_tape::{mask_to_bytes, InputEvent, InputEventKind};
        use crate::unit_wire::resumable_complete_state::{decode_general_processed_inputs,GeneralProcessedInputRow};
        let source = include_str!("../../tools/teul-cli/tests/fixtures/cli_control_input_r1.ddn");
        let tape = InputTape { version:2,madi_hz:60,records:vec![
            InputRecord { madi:0,held_mask:mask_to_bytes(2),events:vec![InputEvent {sequence:0,key_bit:2,kind:InputEventKind::Press}] },
            InputRecord { madi:1,held_mask:mask_to_bytes(0),events:vec![InputEvent {sequence:1,key_bit:2,kind:InputEventKind::Release}] },
            InputRecord { madi:2,held_mask:mask_to_bytes(0),events:vec![] },
        ] };
        let profile = resolve_supported_source_profile(Some("v1-core-v25")).unwrap();
        let mut actual = Vec::new();
        let result = run_summary_with_general_input_supplier(source,"supplier.ddn",Some(3),None,None,None,
            Some(profile),Some(&tape),7,DEFAULT_UPDATE_NAME,None,false,None,None,None,None,Some(&mut actual),None,None,None).unwrap();
        let old = run_summary_from_source_with_authored_bogae(source,"supplier.ddn",Some(3),None,None,None,
            Some(profile),Some(&tape),7,DEFAULT_UPDATE_NAME,None,false,None,None,None,None).unwrap();
        assert_eq!(result,old,"old summaries, hash and work evidence are unchanged");
        assert_eq!(actual.len(),3);
        assert_eq!(actual.iter().map(|row|row.input.rng_seed).collect::<Vec<_>>(),vec![7,6,5]);
        for (row,record) in actual.iter().zip(&tape.records) {
            assert_eq!(row.edges,Some(record.runtime_edges()));
        }
        let bytes = crate::unit_wire::resumable_complete_state::encode_general_processed_inputs(&actual,0,7).unwrap();
        assert_eq!(decode_general_processed_inputs(&bytes,0,7).unwrap(),actual);
        let sentinel = GeneralProcessedInputRow { input:actual[0].input.clone(),edges:None };
        let mut published = vec![sentinel];
        let mut polling = |madi| if madi==1 { Err("E_SAM_LIVE_QUEUE_OVERFLOW 입력 실패".into()) }
            else { Ok(tape.records[madi as usize].clone()) };
        assert!(run_summary_with_general_input_supplier(source,"supplier.ddn",Some(3),None,None,None,
            Some(profile),None,7,DEFAULT_UPDATE_NAME,Some((60,&mut polling)),false,None,None,None,None,
            Some(&mut published),None,None,None).is_err());
        assert_eq!(published.len(),1);
        assert_eq!(published[0].edges,None,"first successful madi must not escape a later failed run");
        assert_eq!(published[0].input,actual[0].input);
    }

    #[test]
    fn general_input_supplier_binds_actual_named_project_source_and_xor_seed() {
        use super::*;
        let request = json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":"채비 { 값:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }."}]}});
        let artifact = crate::authored_project::operate("create",&request.to_string()).unwrap();
        let mut prefix = run_authored_general_input_prefix(&artifact,2,7,None,None).unwrap();
        prefix.check_reconstructed_prefix().unwrap();
        let descriptor=prefix.source_rebuild_descriptor_supply().unwrap();
        let reconstructed=crate::unit_wire::resumable_complete_state::rebuild_source_descriptor_candidate(
            &descriptor,prefix.source_owner()).unwrap();
        assert_eq!(reconstructed.successful_madi_count,2);
        assert_eq!(reconstructed.base_rng_seed,7);
        assert_eq!(reconstructed.execution_instance,prefix.execution.runner.execution_instance_owner_bytes());
        let receipt=prefix.execution.runner.work_cost_receipt();
        let work=crate::unit_wire::resumable_complete_state::encode_work_receipt_supply(&receipt).unwrap();
        crate::unit_wire::resumable_complete_state::check_replayed_work_supply(&work,&receipt).unwrap();
        assert_eq!(prefix.source_owner().project().save().unwrap(),artifact);
        let rows = crate::unit_wire::resumable_complete_state::decode_general_processed_inputs(&prefix.input_supply().unwrap(),0,7).unwrap();
        assert_eq!(rows.len(),2);
        assert!(rows.iter().all(|row|row.edges.is_none()));
        assert_eq!(rows.iter().map(|row|row.input.rng_seed).collect::<Vec<_>>(),vec![7,6]);
        let original_frame=prefix.encode_world_frame().unwrap();
        // An unused input can leave world/hash unchanged. Full actual input
        // custody must nevertheless reject it, then accept exact restoration.
        prefix.rows[0].input.pointer_x_i32=19;
        assert!(prefix.check_reconstructed_prefix().unwrap_err().contains("input"));
        assert_eq!(prefix.encode_world_frame().unwrap(),original_frame);
        prefix.rows[0].input.pointer_x_i32=0;
        prefix.check_reconstructed_prefix().unwrap();
        prefix.rows[0].edges=Some((0,0));
        assert!(prefix.check_reconstructed_prefix().unwrap_err().contains("input"));
        prefix.rows[0].edges=None;
        prefix.check_reconstructed_prefix().unwrap();
        // Direct snapshots are not rejected just because their origin differs.
        // Replay retains that origin and its sealed-input byte/work evidence.
        let snapshots=rows.iter().map(|row|row.input.clone()).collect::<Vec<_>>();
        let direct=run_authored_general_input_prefix(&artifact,2,7,Some(&snapshots),None).unwrap();
        direct.check_reconstructed_prefix().unwrap();
        use crate::input_tape::mask_to_bytes;
        let tape=InputTape {version:2,madi_hz:60,records:vec![
            InputRecord {madi:0,held_mask:mask_to_bytes(0),events:vec![]},
            InputRecord {madi:1,held_mask:mask_to_bytes(0),events:vec![]},
        ]};
        let recorded=run_authored_general_input_prefix(&artifact,2,7,None,Some(&tape)).unwrap();
        recorded.check_reconstructed_prefix().unwrap();
        assert!(recorded.rows.iter().all(|row|row.edges==Some((0,0))));
        assert_eq!(prefix.summary()["source_profile_identity"],"v1-core-v25");
        assert!(!prefix.summary()["state_hash"].as_str().unwrap().is_empty());
        let preparation = prefix.source_owner().sources().preparation();
        let public = run_summary_with_authored_bogae(&preparation.source,&preparation.target_path,
            Some(2),"v1-core-v25",&artifact,None,None).unwrap();
        assert_eq!(public["state_hash"],prefix.summary()["state_hash"]);
        let changed = rows.iter().map(|row| { let mut input=row.input.clone(); input.rng_seed=7; input }).collect::<Vec<_>>();
        assert!(run_authored_general_input_prefix(&artifact,2,7,Some(&changed),None).is_err());
        // Existing execution of those snapshots is not changed to XOR seeds.
        let profile = resolve_supported_source_profile(Some("v1-core-v25")).unwrap();
        let mut ordinary_inputs = Vec::new();
        let _ordinary = run_summary_with_general_input_supplier(&preparation.source,&preparation.target_path,
            Some(2),None,None,None,Some(profile),None,7,"실행",None,true,Some(&changed),None,None,
            Some(&artifact),Some(&mut ordinary_inputs),Some(prefix.source_owner()),None,None).unwrap();
        assert_eq!(ordinary_inputs.len(),changed.len());
        assert!(ordinary_inputs.iter().zip(&changed).all(|(row,input)|row.input==*input));
        let frame = prefix.encode_world_frame().unwrap();
        let issued = crate::project_snapshot_identity::current::issue_saved_local_project(prefix.source_owner()).unwrap();
        let checked_frame = crate::unit_wire::check_complete_state_frame_v1(&frame,issued.pesi().as_raw()).unwrap();
        assert_eq!(checked_frame.next_tick,2);
        let saved = prefix.export_saved_artifact(&[],&[9]).unwrap();
        let attachment = crate::unit_wire::project_runtime_attachment::check_bytes(saved.as_bytes()).unwrap();
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context(
            &attachment,prefix.source_owner(),&[],&[9]).unwrap();
        let mut supplies:[Vec<u8>;10]=std::array::from_fn(|_|Vec::new());
        use crate::unit_wire::resumable_complete_state as complete;
        supplies[0]=context.revision_envelope().to_vec();
        supplies[1]=complete::encode_source_capture_supply(prefix.source_owner().sources()).unwrap();
        supplies[2]=complete::encode_constituent_supply(prefix.source_owner()).unwrap();
        supplies[3]=complete::encode_execution_contract_supply(prefix.source_owner(),&prefix.execution.runner).unwrap();
        supplies[4]=prefix.input_supply().unwrap();
        supplies[9]=prefix.source_rebuild_descriptor_supply().unwrap();
        // This is only the static source/input gate; empty boundary supplies
        // cannot pass full request admission or authorize Native I/O.
        fn static_layout<'a>(supplies:&'a [Vec<u8>;10],frame:&'a [u8],pesi:&'a [u8;32])
            ->crate::unit_wire::ResumableCompleteStateLayoutV2<'a> {
            crate::unit_wire::ResumableCompleteStateLayoutV2 {
                world:crate::unit_wire::check_complete_state_frame_v1(frame,pesi).unwrap(),
                supplies:std::array::from_fn(|i|supplies[i].as_slice()),
            }
        }
        let static_pesi=*issued.pesi().as_raw();
        let layout=static_layout(&supplies,&frame,&static_pesi);
        assert_eq!(complete::rebuild_general_complete_state_source_candidate(&layout,&context).unwrap().successful_madi_count,2);
        assert!(complete::rebuild_complete_state_source_candidate(&layout,&context).is_err(),"general input must not enter logical admission");
        supplies[3]=complete::encode_logical_execution_contract_supply(prefix.source_owner(),&prefix.execution.runner).unwrap();
        assert!(complete::rebuild_general_complete_state_source_candidate(&static_layout(&supplies,&frame,&static_pesi),&context).is_err());
        supplies[3]=complete::encode_execution_contract_supply(prefix.source_owner(),&prefix.execution.runner).unwrap();
        assert!(complete::rebuild_general_complete_state_source_candidate(&static_layout(&supplies,&frame,&static_pesi),&context).is_ok());
        let continued = resume_saved_project_artifact(&saved,&[],&[9],2).unwrap();
        let uninterrupted = run_authored_general_input_prefix(&artifact,4,7,None,None).unwrap();
        assert_eq!(continued["next_tick_id"],4);
        assert_eq!(continued["state_hash"],uninterrupted.summary()["state_hash"]);
        let resumed_input = crate::input_receipt::parse_receipt_snapshot(&continued["frames"][0]["input_receipt"]).unwrap();
        assert_eq!(resumed_input.tick_id,2);
        assert_eq!(resumed_input.rng_seed,7^2);
    }
    #[test]
    fn deep_result_artifact_finalizer_accepts_exact_limit_and_cleans_rejection() {
        use super::*;
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let make_runner = |limit| {
                let program = DdnProgram::from_source("(매마디)마다 { }.", "deep-result.ddn").unwrap();
                DdnRunner::new(program, "매틱").with_standard_resource_limits(
                    StandardResourceLimitsV1 { result_artifact_bytes:limit, ..StandardResourceLimitsV1::CANDIDATE })
            };
            let make_summary = |runner: &DdnRunner| {
                let mut nested = JsonValue::Bool(true);
                for _ in 0..20_000 { nested = JsonValue::Array(vec![nested]); }
                crate::unit_wire::SnapshotJsonOwner::from_value(JsonValue::Object([
                    ("resources".into(), nested),
                    ("work_cost_receipt".into(), serde_json::to_value(runner.work_cost_receipt()).unwrap()),
                ].into_iter().collect()))
            };
            let mut zero = make_runner(0);
            let before = zero.work_cost_receipt().counters.result_artifact_bytes;
            let failure = finalize_run_summary_result_artifact(make_summary(&zero), &mut zero, "deep-result.ddn").unwrap_err();
            assert_eq!(failure.code, "E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE");
            assert_eq!(zero.work_cost_receipt().counters.result_artifact_bytes, before);
            // Iterate only the limit's decimal-width effect to find its exact fixed point.
            let mut exact = 100_000;
            for _ in 0..16 {
                let mut runner = make_runner(exact);
                let summary = make_summary(&runner);
                let finalized = crate::unit_wire::SnapshotJsonOwner::from_value(
                    finalize_run_summary_result_artifact(summary, &mut runner, "deep-result.ddn").unwrap());
                let actual = serialized_json_byte_count(&finalized).unwrap() as u64;
                assert_eq!(runner.work_cost_receipt().counters.result_artifact_bytes, actual);
                if actual == exact { break; }
                exact = actual;
            }
            let mut runner = make_runner(exact);
            let finalized = crate::unit_wire::SnapshotJsonOwner::from_value(
                finalize_run_summary_result_artifact(make_summary(&runner), &mut runner, "deep-result.ddn").unwrap());
            assert_eq!(serialized_json_byte_count(&finalized).unwrap() as u64, exact);
            let mut below = make_runner(exact - 1);
            assert!(finalize_run_summary_result_artifact(make_summary(&below), &mut below, "deep-result.ddn").is_err());
            drop(finalized);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn result_byte_count_and_resource_patch_handle_deep_values_without_buffering() {
        use super::*;
        for value in [json!(null), json!({"a":["한글\n😀", 1.25, false]}), json!({"type":"bytes_v1"})] {
            assert_eq!(serialized_json_byte_count(&value).unwrap(), serde_json::to_vec(&value).unwrap().len());
        }
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = ResourceValue::Bool(true);
            for _ in 0..20_000 { value = ResourceValue::List(vec![value]); }
            let mut patch = Patch::default();
            patch.ops.push(PatchOp::SetResourceValue { tag:"tree".into(), value });
            let payload = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_patch(&patch));
            let text = payload.compact_text();
            assert_eq!(serialized_json_byte_count(&payload).unwrap(), text.len());
            assert_eq!(payload[0]["op"], json!("set_resource_value"));
            let cloned = crate::unit_wire::SnapshotJsonOwner::clone_from_value(&payload);
            assert!(cloned.equals_json(&payload));
            let mut world = DetNuri::new();
            let PatchOp::SetResourceValue { value, .. } = &patch.ops[0] else { panic!() };
            world.world_mut().set_resource_value("tree".into(), value.clone());
            let before = world.world().state_hash();
            let response = crate::unit_wire::SnapshotJsonOwner::from_value(build_engine_state_json(&world, 1, json!({}), &patch));
            assert_eq!(serialized_json_byte_count(&response).unwrap(), response.compact_text().len());
            assert_eq!(world.world().state_hash(), before);
            drop(response); drop(world); drop(cloned); drop(payload); drop(patch);
        }).unwrap().join().unwrap();
        let value = ResourceValue::List(vec![ResourceValue::Bool(true), ResourceValue::String("한글".into())]);
        assert_eq!(resource_value_patch_json("tree", &value).to_string(), json!({
            "op":"set_resource_value", "tag":"tree", "value":value.canon_key(),
            "value_json":resource_value_to_json(&value)}).to_string());
    }

    #[test]
    fn display_projection_preserves_legacy_bytes_and_deep_tree_lifetime() {
        use super::*;
        use ddonirang_core::platform::ResourceMapEntry;
        fn old(value: &ResourceValue) -> JsonValue {
            if let Some(formula) = formula_from_resource_value(value) {
                let (dialect, raw) = formula_summary_parts(&formula);
                return json!({"dialect":dialect,"raw":raw});
            }
            match value {
                ResourceValue::List(items) => JsonValue::Array(items.iter().map(old).collect()),
                ResourceValue::Set(items) => JsonValue::Array(items.values().map(old).collect()),
                ResourceValue::Map(entries) => JsonValue::Object(entries.values().map(|entry|
                    (resource_key_to_string(&entry.key), old(&entry.value))).collect()),
                _ => resource_value_leaf_to_json(value),
            }
        }
        let formula = ResourceValue::Map([
            ("a".into(), ResourceMapEntry { key: ResourceValue::String("__ddn_kind".into()), value: ResourceValue::String("ddn.formula.v1".into()) }),
            ("b".into(), ResourceMapEntry { key: ResourceValue::String("raw".into()), value: ResourceValue::String("x + 1".into()) }),
            ("c".into(), ResourceMapEntry { key: ResourceValue::String("dialect".into()), value: ResourceValue::String("ascii".into()) }),
        ].into_iter().collect());
        assert!(formula_from_resource_value(&formula).is_some());
        let sample = ResourceValue::List(vec![ResourceValue::None, ResourceValue::Bool(true),
            ResourceValue::Fixed64(Fixed64::from_i64(-2)), ResourceValue::String("한글\n🙂".into()),
            ResourceValue::Set([("a".into(), ResourceValue::Bool(false))].into_iter().collect()), formula,
            ResourceValue::Map([
                ("a".into(), ResourceMapEntry { key: ResourceValue::String("same".into()), value: ResourceValue::List(vec![ResourceValue::Bool(false)]) }),
                ("b".into(), ResourceMapEntry { key: ResourceValue::String("same".into()), value: ResourceValue::Bool(true) }),
            ].into_iter().collect())]);
        let actual = crate::unit_wire::SnapshotJsonOwner::from_value(resource_value_to_json(&sample));
        assert_eq!(actual.compact_text(), old(&sample).to_string());
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = ResourceValue::Bool(true);
            for index in 0..20_000 {
                value = match index % 3 {
                    0 => ResourceValue::List(vec![value]),
                    1 => ResourceValue::Set([("x".into(), value)].into_iter().collect()),
                    _ => ResourceValue::Map([("x".into(), ResourceMapEntry { key: ResourceValue::String("x".into()), value })].into_iter().collect()),
                };
            }
            let projected = crate::unit_wire::SnapshotJsonOwner::from_value(resource_value_to_json(&value));
            let text = projected.compact_text();
            assert_eq!(text.matches("[" ).count(), 13_334);
            assert_eq!(text.matches("{\"x\":" ).count(), 6_666);
            // The second display key displaces a deep first value in the actual product map path.
            let collision = ResourceValue::Map([
                ("a".into(), ResourceMapEntry { key: ResourceValue::String("same".into()), value }),
                ("b".into(), ResourceMapEntry { key: ResourceValue::String("same".into()), value: ResourceValue::Bool(false) }),
            ].into_iter().collect());
            let displaced = crate::unit_wire::SnapshotJsonOwner::from_value(resource_value_to_json(&collision));
            assert_eq!(displaced.compact_text(), "{\"same\":false}");
            let mut world = DetNuri::new();
            world.world_mut().set_resource_value("tree".into(), collision);
            let before = world.world().state_hash();
            let resources = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_world_resources(world.world()));
            assert_eq!(resources["value_json"]["tree"]["same"], json!(false));
            assert!(!resources.compact_text().is_empty());
            assert_eq!(world.world().state_hash(), before);
            drop(resources); drop(world); drop(displaced); drop(projected);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn runtime_snapshot_builder_transfers_deep_resources_without_reencoding() {
        use super::*;
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = ResourceValue::Bool(true);
            for _ in 0..20_000 { value = ResourceValue::List(vec![value]); }
            let mut world = DetNuri::new();
            world.world_mut().set_resource_value("tree".into(), value);
            let before = world.world().state_hash();
            let snapshot = crate::unit_wire::SnapshotJsonOwner::from_value(runtime_state_snapshot(&world, vec![], 7));
            let text = snapshot.compact_text();
            assert_eq!(text.matches("\"type\":\"list\"").count(), 20_000);
            assert_eq!(snapshot["next_tick_id"], json!(7));
            assert_eq!(world.world().state_hash(), before);
            // RD-694: exercise the actual shared raw parse/restore path, not
            // just an internal projection that bypasses saved admission.
            let mut parsed = crate::unit_wire::parse_snapshot_json(&text).unwrap();
            let mut restored = DetNuri::new();
            crate::unit_wire::restore_world_resources_snapshot(
                restored.world_mut(), &parsed["resources"]).unwrap();
            assert_eq!(restored.world().state_hash(), before);
            let again = crate::unit_wire::SnapshotJsonOwner::from_value(
                runtime_state_snapshot(&restored, vec![], 7));
            assert_eq!(again.compact_text(), text);
            // A later invalid typed row must not publish the admitted deep
            // first row or replace a pre-existing normal world.
            parsed.value_mut()["resources"]["value_det"]["z"] =
                json!({"type":"bool","value":1});
            let preserved = restored.world().state_hash();
            assert!(crate::unit_wire::restore_world_resources_snapshot(
                restored.world_mut(), &parsed["resources"]).is_err());
            assert_eq!(restored.world().state_hash(), preserved);
            drop(again); drop(restored); drop(parsed);
            drop(snapshot); drop(world);
        }).unwrap().join().unwrap();
        let world = DetNuri::new();
        let expected = json!({"schema":"ddn.wasm_state_snapshot.v2",
            "resource_snapshot_schema":RESOURCE_SNAPSHOT_SCHEMA_V2,"unit_wire_schema":UNIT_VALUE_WIRE_SCHEMA_V2,
            "resources":serialize_world_resources_snapshot(world.world()),"param_overrides":{},
            "public_observation_frames":[],"next_tick_id":7});
        assert_eq!(runtime_state_snapshot(&world, vec![], 7).to_string(), expected.to_string());
    }
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
        assert_eq!(error.expected.as_deref(), Some("셈수"));
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
