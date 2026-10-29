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
pub(crate) use native_source_world::{resume_logical_source_world,resume_logical_source_world_with_result,save_continued_logical_source_world,verify_native_logical_current_receipt};
#[cfg(windows)]
pub(crate) use native_source_world::{NativeLogicalFinishSessionV1,recover_native_logical_finish};

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
    if let Some(facts) = crate::numeric_constraint_plan::generic_source_diagnostic(source,message) {
        let mut diagnostic=run_summary_diagnostic(facts.code,message.to_owned(),path,"source_type_check");
        if let Some(primary)=facts.spans.last() { diagnostic.span=Some((primary.start,primary.end)); }
        if facts.spans.len()==2 {
            let related=&facts.spans[0];
            let mut cause=run_summary_diagnostic(facts.code,message.to_owned(),path,"source_type_check");
            cause.span=Some((related.start,related.end));
            diagnostic.cause=Some(Box::new(cause));
        }
        return Some(diagnostic);
    }
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
    if error.code() == "E_PARSE" && message.starts_with(&format!("파싱 실패: {}", error.message)) {
        // Metadata stripping preserves lines but removes header bytes. The parser
        // span must be translated back before exposing it to source consumers.
        let removed = source.len() - stripped.len();
        let start = error.span.start.checked_add(removed)?;
        let end = error.span.end.checked_add(removed)?;
        if start > end || end > source.len()
            || !source.is_char_boundary(start) || !source.is_char_boundary(end) {
            return None;
        }
        let mut diagnostic = run_summary_diagnostic("E_PARSE", message.to_owned(), path, "execution_parse");
        diagnostic.span = Some((start, end));
        return Some(diagnostic);
    }
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
    // F5-GA[SURFACE_SUPPLIES] Both checkpoint layouts construct ten local supply vectors; the layout field is a ten-element array, so from_fn can issue only indices0..9.
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
        // F5-GA[SURFACE_CHAIN] The immutable candidate length bounds the prefix slice and both guarded receipt indices. source_history is the same immutable Option after is_some; a staged predecessor exists only after an earlier iteration.
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
    // F5-GA[SURFACE_READ_JSON] Shared Value/owner indexing in schema/display/input/frame and envelope integrity reads uses serde_json Index, which yields Null for missing or wrong-shaped fields; it is not mutable IndexMut.
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
    // F5-GA[RESTORED_FRAMES] The exact stage_runtime_state supplier checked is_array before cloning this owner; this function only pushes entries and lends immutable slices until moving the whole array into result.
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
    // F5-GA[RESTORED_LOCAL] runtime_state_snapshot returns an Object; build_engine_state_json likewise returns Object/state Object, and these local field insertions do not replace either container.
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
        let issued=crate::project_snapshot_identity::current::issue_saved_project(owned)?;
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
        let issued = crate::project_snapshot_identity::current::issue_saved_project(&self.owned)
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
        let issued=crate::project_snapshot_identity::current::issue_saved_project(&self.owned)?;
        let frame=crate::unit_wire::check_complete_state_frame_v1(&bytes,issued.pesi().as_raw())
            .map_err(|cause|format!("E_PROJECT_RUNTIME_GENERAL_CHANNEL: {cause:?}"))?;
        self.export_saved_artifact_with_frame(parent,revision,Some(frame))
    }
    fn export_saved_artifact_with_frame(&self,parent:&[u8],revision:&[u8],frame:Option<crate::unit_wire::CompleteStateFrameV1<'_>>)->Result<String,String>{
        self.check_reconstructed_prefix()?;
        self.encode_world_frame().map_err(|error|format!("E_PROJECT_RUNTIME_GENERAL_CHANNEL: 누락 없는 세계 저장 공급이 필요합니다 ({error:?})"))?;
        let mut runtime = crate::unit_wire::SnapshotJsonOwner::clone_from_value(&self.summary["engine_state"]);
        // F5-GA[RESTORED_PREFIX] This private prefix summary is created only by the shared run builder with Object/state/snapshot_v2 Object; it has no mutable summary accessor. check_reconstructed_prefix above also compares the complete freshly rebuilt summary before this clone.
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
mod general_execution;
use general_execution::*;
pub use general_execution::{run_summary_from_supported_source_profile, run_summary_from_supported_source_profile_with_host_resources, attach_host_resource_receipt, run_summary_from_supported_source_profile_with_seed, run_summary_from_supported_source_profile_with_project_assets_diagnostic, run_summary_from_source_with_project_assets_diagnostic, run_summary_from_source_with_input_tape, run_summary_from_source_with_seed_and_input_tape, run_summary_from_supported_source_profile_with_input_tape, run_summary_from_supported_source_profile_with_seed_and_input_tape};

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
        // F5-GA[SURFACE_UTF8] The successful JSON writer emits UTF-8 JSON into this private Vec; no arbitrary bytes are inserted between serialization and String::from_utf8.
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
        // F5-GA[SURFACE_LOCAL_STATE] result was just constructed as an Object containing an Object state; this loop only replaces fields and cannot change either container shape before state/patch insertion.
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

mod view_projection;
use view_projection::*;
pub use view_projection::{seed_bogae_defaults, serialize_world_resources, collect_columns_and_row};
pub(crate) use view_projection::{runtime_state_snapshot, resource_value_patch_json, serialize_input_snapshot, resource_value_to_json, derive_graph_from_points};

#[cfg(test)]
mod tests;
