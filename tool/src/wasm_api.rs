use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use blake3::hash as blake3_hash;
use ddonirang_core::platform::{
    DetNuri, InputSnapshot, InputSource, Patch, PatchOp, ResourceValue,
};
use ddonirang_core::{
    ExternalBoundaryFailure, ExternalBoundaryRequest, ExternalBoundaryResult, Fixed64, Nuri,
    ProcessLocalCapabilityStore, ResourceHandle, SealedBoundaryProvider, SealedExecutionLedger,
    SeulgiIntent, SeulgiPacket,
};
use ddonirang_lang::runtime::Value;
use ddonirang_lang::{ExecutionModeDecision, ParseMode};
use serde_json::{json, Map, Value as JsonValue};
use wasm_bindgen::prelude::*;

use crate::canon;
use crate::corpus_csv;
use crate::ddn_runtime::{
    formula_from_resource_value, formula_summary_parts, resolve_supported_source_profile,
    DdnParseWarning, DdnProgram, DdnRunner,
};
use crate::deterministic_debug::{
    append_public_observation_surface, build_public_observation_table_projection,
    build_public_observation_trace, deterministic_debug_operation, public_observation_events_json,
    DebugEvaluator,
};
use crate::fixed64_boundary::Fixed64FloatBoundary;
use crate::host_resource;
use crate::local_project;
use crate::project_authoring;
use crate::runtime_surface;
use crate::source_analysis;
use crate::unit_wire::{RESOURCE_SNAPSHOT_SCHEMA_V2, UNIT_VALUE_WIRE_SCHEMA_V2};

const DEFAULT_UPDATE_NAME: &str = "매마디";
const ENGINE_RESPONSE_SCHEMA: &str = "seamgrim.engine_response.v0";
const WASM_STATE_SNAPSHOT_SCHEMA_V2: &str = "ddn.wasm_state_snapshot.v2";
const SPACE2D_SCHEMA: &str = "seamgrim.space2d.v0";
const BOGAE_DRAWLIST_TAG: &str = "보개_그림판_목록";
const BOGAE_WIDTH_TAG: &str = "보개_그림판_가로";
const BOGAE_HEIGHT_TAG: &str = "보개_그림판_세로";

/// WASM adapter only: no bindgen export and no separate external-boundary
/// semantics. The public product surface remains unchanged.
pub(crate) fn wasm_execute_sealed_external_boundary(
    ledger: &mut SealedExecutionLedger,
    capability_store: &ProcessLocalCapabilityStore,
    provider: &dyn SealedBoundaryProvider,
    request: &ExternalBoundaryRequest,
) -> Result<ExternalBoundaryResult, ExternalBoundaryFailure> {
    runtime_surface::execute_sealed_external_boundary(ledger, capability_store, provider, request)
}
const BOGAE_SHOW_LINES_TAG: &str = "보개_출력_줄들";
const GRAPH_SCHEMA: &str = "seamgrim.graph.v0";
const WORKBENCH_RESULT_SET_SCHEMA: &str =
    "seamgrim.workbench.authoritative_result_set.v1_candidate";
const WORKBENCH_LINE_RESULT_SCHEMA: &str = "seamgrim.workbench.line_result.v1_candidate";
const WORKBENCH_PENDULUM_RESULT_SCHEMA: &str = "seamgrim.workbench.pendulum_result.v1_candidate";
const WORKBENCH_PROJECTILE_RESULT_SCHEMA: &str =
    "seamgrim.workbench.projectile_result.v1_candidate";
const WORKBENCH_GRID_RESULT_SCHEMA: &str = "seamgrim.workbench.grid_result.v1_candidate";
const DEFAULT_VIEW_PREFIXES: &[&str] = &["보개_", "__view_"];
const GRAPH_POINTS_TAGS: &[(&str, &str, &str)] = &[
    ("보개_그래프_점목록_f", "f", "f(x)"),
    ("그래프_점목록_f", "f", "f(x)"),
    ("그래프_점목록_df", "df", "f'(x)"),
    ("그래프_점목록_fi", "fi", "int(f)"),
    ("graph_points_f", "f", "f(x)"),
    ("graph_points_df", "df", "f'(x)"),
    ("graph_points_fi", "fi", "int(f)"),
];
fn parse_mode_from_str(mode: &str) -> Result<ParseMode, JsValue> {
    let normalized = mode.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" => Ok(ParseMode::SuccessorDefault),
        "strict" | "successor-default" => Ok(ParseMode::SuccessorDefault),
        _ => Err(JsValue::from_str(&format!(
            "지원하지 않는 lang-mode: {mode} (strict|successor-default)"
        ))),
    }
}

fn require_wasm_source_path(source_path: &str) -> Result<&str, JsValue> {
    let trimmed = source_path.trim();
    if trimmed.is_empty() {
        return Err(JsValue::from_str(
            "E_WASM_SOURCE_PATH_EMPTY: 공개 관찰 source_ref에 사용할 DDN 경로가 필요합니다",
        ));
    }
    Ok(trimmed)
}

fn canonicalize_for_wasm(source: &str) -> Result<canon::CanonOutput, JsValue> {
    runtime_surface::canonicalize_shared_source(source)
        .map_err(|err| JsValue::from_str(&format!("WASM canon 실패: {err}")))
}

fn canonicalize_for_wasm_with_supported_profile(
    source: &str,
    source_profile_identity: &str,
) -> Result<canon::CanonOutput, JsValue> {
    runtime_surface::canonicalize_shared_source_with_supported_profile(
        source,
        Some(source_profile_identity),
    )
    .map_err(|err| JsValue::from_str(&format!("WASM canon 실패: {err}")))
}

fn canonicalize_flat_for_wasm(source: &str) -> Result<canon::CanonOutput, JsValue> {
    canon::canonicalize(source, false)
        .map_err(|err| JsValue::from_str(&format!("WASM canon 실패: {err}")))
}

fn preprocess_program_source_for_wasm(source: &str) -> Result<String, JsValue> {
    runtime_surface::prepare_source_for_execution(source)
        .map(|prepared| prepared.execution_source)
        .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))
}

fn parse_program_for_wasm(
    source: &str,
    source_path: &str,
    mode: ParseMode,
) -> Result<(DdnProgram, ExecutionModeDecision), JsValue> {
    let prepared = runtime_surface::prepare_source_for_execution(source)
        .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?;
    let program = DdnProgram::from_source_with_mode(source, source_path, mode)
        .map_err(|err| JsValue::from_str(&err))?;
    Ok((program, prepared.execution_mode_decision))
}

#[wasm_bindgen]
pub fn wasm_build_info() -> String {
    format!(
        "pkg={} version={} state_schema={} state_snapshot={} resource_schema={} unit_wire={} update={}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        ENGINE_RESPONSE_SCHEMA,
        WASM_STATE_SNAPSHOT_SCHEMA_V2,
        RESOURCE_SNAPSHOT_SCHEMA_V2,
        UNIT_VALUE_WIRE_SCHEMA_V2,
        DEFAULT_UPDATE_NAME
    )
}

#[wasm_bindgen]
pub fn wasm_deterministic_debug_json(operation: &str, input_json: &str) -> Result<String, JsValue> {
    let input: JsonValue = serde_json::from_str(input_json)
        .map_err(|err| JsValue::from_str(&format!("E_DEBUG_JSON_INVALID: {err}")))?;
    deterministic_debug_operation(operation, &input)
        .map(|value| value.to_string())
        .map_err(|err| JsValue::from_str(&err))
}

#[wasm_bindgen]
pub fn wasm_preprocess_source(source: &str) -> Result<String, JsValue> {
    preprocess_program_source_for_wasm(source)
}

#[wasm_bindgen]
pub fn wasm_source_analysis_snapshot_json(request_json: &str) -> Result<String, JsValue> {
    source_analysis::source_analysis_snapshot_json(request_json)
        .map_err(|err| JsValue::from_str(&err))
}

#[wasm_bindgen]
pub fn wasm_historical_async_migration_evidence_json(
    source: &str,
    source_path: &str,
) -> Result<String, JsValue> {
    let source_path = require_wasm_source_path(source_path)?;
    runtime_surface::historical_async_migration_evidence_json(source, source_path)
        .map_err(|err| JsValue::from_str(&err))
}

#[wasm_bindgen]
pub fn wasm_execution_envelope_json(source: &str, requested_ticks: i32) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    runtime_surface::execution_envelope_from_source(source, requested)
        .map(|envelope| envelope.to_string())
        .map_err(|err| JsValue::from_str(&format!("WASM execution envelope 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_run_summary_json(
    source: &str,
    source_path: &str,
    requested_ticks: i32,
) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    let source_path = require_wasm_source_path(source_path)?;
    runtime_surface::run_summary_from_source(source, source_path, requested)
        .map(|summary| summary.to_string())
        .map_err(|err| JsValue::from_str(&format!("WASM run summary 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_run_summary_with_source_profile_json(
    source: &str,
    source_path: &str,
    requested_ticks: i32,
    source_profile_identity: &str,
) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    let source_path = require_wasm_source_path(source_path)?;
    runtime_surface::run_summary_from_supported_source_profile(
        source,
        source_path,
        requested,
        Some(source_profile_identity),
    )
    .map(|summary| summary.to_string())
    .map_err(|err| JsValue::from_str(&err))
}

/// Typed host-resource request entrypoint for RuntimeBridge/Workbench.
/// The bridge supplies sealed text payloads for record or a sealed transcript
/// for replay; it never interprets their semantic identity itself.
#[wasm_bindgen]
pub fn wasm_run_summary_with_source_profile_host_resources_json(
    source: &str,
    source_path: &str,
    requested_ticks: i32,
    source_profile_identity: &str,
    host_resource_request_json: &str,
) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    let source_path = require_wasm_source_path(source_path)?;
    let request = host_resource::parse_request_json(host_resource_request_json)
        .map_err(|error| JsValue::from_str(&error))?;
    let prepared = host_resource::prepare_request(&request)
        .map_err(|error| JsValue::from_str(&error))?;
    runtime_surface::run_summary_from_supported_source_profile_with_host_resources(
        source,
        source_path,
        requested,
        Some(source_profile_identity),
        &prepared,
    )
    .map(|summary| summary.to_string())
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
pub fn wasm_model_prepared_state_json(source: &str, source_path: &str) -> Result<String, JsValue> {
    let source_path = require_wasm_source_path(source_path)?;
    runtime_surface::model_prepared_state_from_source(source, source_path)
        .map(|state| state.to_string())
        .map_err(|err| JsValue::from_str(&format!("WASM model prepared state 실패: {err}")))
}

/// Returns the bounded runtime-artifact primary-view decision for one run.
/// This is a candidate product ABI for the primary-only wave; callers must
/// render only a selected artifact and surface diagnostics for absent/ambiguous.
#[wasm_bindgen]
pub fn wasm_primary_view_decision_json(
    source: &str,
    requested_ticks: i32,
) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    let summary =
        runtime_surface::run_summary_from_source(source, "<wasm-primary-view>", requested)
            .map_err(|err| JsValue::from_str(&format!("WASM primary view 실패: {err}")))?;
    Ok(summary
        .get("primary_view_decision")
        .cloned()
        .unwrap_or(JsonValue::Null)
        .to_string())
}

#[wasm_bindgen]
pub fn wasm_normalize_project_source_json(source_json: &str) -> Result<String, JsValue> {
    local_project::normalize_virtual_project_source_json(source_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_discover_project_root_json(request_json: &str) -> Result<String, JsValue> {
    local_project::discover_virtual_project_root_json(request_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_project_graph_json(source_json: &str) -> Result<String, JsValue> {
    local_project::build_project_graph_json(source_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_project_edit_plan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_project_edit_plan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_execution_recipe_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_project_execution_recipe_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_gaji_extraction_plan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_gaji_extraction_plan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_local_gaji_artifact_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_local_gaji_artifact_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_verify_local_gaji_artifact_json(artifact_json: &str) -> Result<String, JsValue> {
    local_project::verify_local_gaji_artifact_json(artifact_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_publish_local_gotgan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::publish_local_gotgan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_search_local_gotgan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::search_local_gotgan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_gaji_lifecycle_plan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_gaji_lifecycle_plan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_source_sha256_json(source_json: &str) -> Result<String, JsValue> {
    local_project::virtual_project_source_sha256_json(source_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_symbols_json(source_json: &str) -> Result<String, JsValue> {
    local_project::build_project_symbol_boundary_json(source_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_project_authoring_snapshot_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_snapshot_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_definition_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_definition_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_references_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_references_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_hover_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_hover_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_completion_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_completion_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_rename_preview_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_rename_preview_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_corpus_csv_intake_json(request_json: &str) -> Result<String, JsValue> {
    corpus_csv::build_corpus_csv_intake_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_columns_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_columns_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_sources_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_sources_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_evidence_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_evidence_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_review_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_review_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_seal_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_seal_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_project_gaji_lock_json(source_json: &str) -> Result<String, JsValue> {
    local_project::build_project_gaji_lock_json(source_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_project_prepare_link_json(
    source_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<String, JsValue> {
    local_project::build_project_prepare_link_json(source_json, target, inspect_graph, inspect_lock)
        .map_err(|err| {
            JsValue::from_str(
                &serde_json::to_string(&err.to_json_value())
                    .unwrap_or_else(|_| err.message.clone()),
            )
        })
}

#[wasm_bindgen]
pub fn wasm_project_run_summary_json(
    source_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<String, JsValue> {
    local_project::build_project_run_summary_with_supported_profile_json(
        source_json,
        target,
        inspect_graph,
        inspect_lock,
    )
    .map_err(|err| {
            JsValue::from_str(
                &serde_json::to_string(&err.to_json_value())
                    .unwrap_or_else(|_| err.message.clone()),
            )
        })
}

/// RuntimeBridge/Workbench local-project variant of the same shared
/// host-resource invocation algorithm used by the native product frontdoor.
#[wasm_bindgen]
pub fn wasm_project_run_summary_with_host_resources_json(
    source_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
    host_resource_request_json: &str,
) -> Result<String, JsValue> {
    local_project::build_project_run_summary_with_supported_profile_host_resources_json(
        source_json,
        target,
        inspect_graph,
        inspect_lock,
        host_resource_request_json,
    )
    .map_err(|err| {
        JsValue::from_str(
            &serde_json::to_string(&err.to_json_value())
                .unwrap_or_else(|_| err.message.clone()),
        )
    })
}

#[wasm_bindgen]
pub fn wasm_canon_flat_json(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.guseong_flat_json)
}

#[wasm_bindgen]
pub fn wasm_canon_ddn(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_for_wasm(source)?;
    Ok(output.ddn)
}

/// 명시적 supported source profile용 내부 candidate canon entry다.
/// stable public WASM API 또는 V1 membership을 선언하지 않는다.
#[wasm_bindgen]
pub fn wasm_canon_ddn_with_source_profile(
    source: &str,
    source_profile_identity: &str,
) -> Result<String, JsValue> {
    let output =
        canonicalize_for_wasm_with_supported_profile(source, source_profile_identity)?;
    Ok(output.ddn)
}

/// 검사 전용 candidate projection이다. persistent public wire/ABI가 아니다.
#[wasm_bindgen]
pub fn wasm_model_ir_candidate_json(source: &str) -> Result<String, JsValue> {
    runtime_surface::model_ir_candidate_json_from_source(source)
        .map_err(|err| JsValue::from_str(&format!("WASM model IR 실패: {err}")))
}

/// Owner-approved bounded backend execution. The JSON wire remains a
/// versioned candidate and is not a public stable ABI.
#[wasm_bindgen]
pub fn wasm_model_run_candidate_json(source: &str) -> Result<String, JsValue> {
    crate::model_runtime::execute_source_pretty_json(source).map_err(|err| {
        JsValue::from_str(&format!("WASM model 실행 실패: {} {}", err, err.to_json()))
    })
}

/// Source labels are provenance only; they never participate in backend
/// selection. This candidate API lets project/CLI parity bind the same label.
#[wasm_bindgen]
pub fn wasm_model_run_candidate_json_with_source_file(
    source: &str,
    source_file: &str,
) -> Result<String, JsValue> {
    crate::model_runtime::execute_source_pretty_json_with_source_file(source, source_file).map_err(
        |err| JsValue::from_str(&format!("WASM model 실행 실패: {} {}", err, err.to_json())),
    )
}

/// Stored candidate execution evidence verifier. It never executes a fallback
/// backend and returns a structured fail-closed verdict for content tampering.
#[wasm_bindgen]
pub fn wasm_model_execution_verify_json(report_json: &str) -> Result<String, JsValue> {
    runtime_surface::model_execution_verify_json(report_json)
        .map_err(|err| JsValue::from_str(&format!("WASM model receipt 검증 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_canon_maegim_plan(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.maegim_control_json)
}

#[wasm_bindgen]
pub fn wasm_canon_maegim_source_plan(source: &str, source_path: &str) -> Result<String, JsValue> {
    canon::maegim_control_plan_with_source(source, source_path)
        .map_err(|err| JsValue::from_str(&format!("WASM 채비 source plan 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_apply_maegim_source_edit(
    source: &str,
    source_path: &str,
    control_name: &str,
    value_text: &str,
) -> Result<String, JsValue> {
    canon::apply_maegim_source_edit(source, source_path, control_name, value_text)
        .map_err(|err| JsValue::from_str(&format!("WASM 채비 원문 수정 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_canon_alrim_plan(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.alrim_plan_json)
}

#[wasm_bindgen]
pub fn wasm_canon_exec_policy_map(source: &str) -> Result<String, JsValue> {
    if !canon::has_exec_policy_surface(source) {
        return Ok("{}\n".to_string());
    }
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.exec_policy_map_json)
}

#[wasm_bindgen]
pub fn wasm_canon_block_editor_plan(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.block_editor_plan_json)
}

struct PublicObservationSurface {
    trace: JsonValue,
    trace_hash: String,
    table_projection: JsonValue,
}

fn build_wasm_state_snapshot_v2(
    resources: JsonValue,
    param_overrides: JsonValue,
    public_observation_frames: Vec<JsonValue>,
    next_tick_id: u64,
) -> JsonValue {
    json!({
        "schema": WASM_STATE_SNAPSHOT_SCHEMA_V2,
        "resource_snapshot_schema": RESOURCE_SNAPSHOT_SCHEMA_V2,
        "unit_wire_schema": UNIT_VALUE_WIRE_SCHEMA_V2,
        "resources": resources,
        "param_overrides": param_overrides,
        "public_observation_frames": public_observation_frames,
        "next_tick_id": next_tick_id,
    })
}

fn last_committed_madi(frames: &[JsonValue]) -> JsonValue {
    frames
        .last()
        .and_then(|frame| frame.get("committed_madi"))
        .cloned()
        .unwrap_or(JsonValue::Null)
}

fn build_wasm_public_observation_surface(
    source_path: &str,
    source_bytes_sha256: &str,
    canonical_ddn_sha256: &str,
    frames: &[JsonValue],
) -> Result<PublicObservationSurface, String> {
    let source_identity = format!("sha256:{source_bytes_sha256}");
    let revision_identity = format!("sha256:{canonical_ddn_sha256}");
    let run_id = format!(
        "run:{}",
        runtime_surface::sha256_hex(
            format!(
                "{}\0{}\0{}\0{}",
                source_path,
                source_identity,
                revision_identity,
                frames.len()
            )
            .as_bytes()
        )
    );
    let bundle = build_public_observation_trace(&json!({
        "project_id": "",
        "run_id": run_id,
        "source_identity": source_identity,
        "revision_identity": revision_identity,
        "run_status": "completed",
        "frames": frames,
    }))?;
    let trace = bundle
        .get("trace")
        .cloned()
        .ok_or_else(|| "E_WASM_OBSERVATION_TRACE_MISSING: trace가 없습니다".to_string())?;
    let trace_hash = bundle
        .get("trace_hash")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| "E_WASM_OBSERVATION_TRACE_HASH_MISSING: trace hash가 없습니다".to_string())?
        .to_string();
    let table_projection = build_public_observation_table_projection(&trace, &run_id)?;
    Ok(PublicObservationSurface {
        trace,
        trace_hash,
        table_projection,
    })
}

fn append_wasm_public_observation_surface(
    vm: &DdnWasmVm,
) -> Result<PublicObservationSurface, String> {
    let frame = vm.public_observation_frames.last().ok_or_else(|| {
        "E_WASM_OBSERVATION_APPEND_FRAME_MISSING: 최신 frame이 없습니다".to_string()
    })?;
    let source_identity = format!("sha256:{}", vm.source_bytes_sha256);
    let revision_identity = format!("sha256:{}", vm.canonical_ddn_sha256);
    let run_id = format!(
        "run:{}",
        runtime_surface::sha256_hex(
            format!(
                "{}\0{}\0{}\0{}",
                vm.source_path,
                source_identity,
                revision_identity,
                vm.public_observation_frames.len()
            )
            .as_bytes()
        )
    );
    let delta_input = json!({
        "project_id": "",
        "run_id": run_id,
        "source_identity": source_identity,
        "revision_identity": revision_identity,
        "run_status": "completed",
        "frames": [frame],
    });
    let (trace, trace_hash, table_projection) = append_public_observation_surface(
        &vm.public_observation_trace,
        &vm.public_observation_table_projection,
        &delta_input,
    )?;
    Ok(PublicObservationSurface {
        trace,
        trace_hash,
        table_projection,
    })
}

#[wasm_bindgen]
pub struct DdnWasmVm {
    runner: DdnRunner,
    world: DetNuri,
    defaults: HashMap<String, Value>,
    param_overrides: BTreeMap<String, Value>,
    tick_id: u64,
    rng_seed: u64,
    view_prefixes: Vec<String>,
    input_keys_pressed: u64,
    input_last_key_name: String,
    input_pointer_x_i32: i32,
    input_pointer_y_i32: i32,
    input_dt: Fixed64,
    last_patch: Option<Patch>,
    pending_ai_injections: Vec<(String, String)>,
    lang_mode: ParseMode,
    parse_warnings: Vec<DdnParseWarning>,
    configured_madi: u64,
    execution_mode_decision: ExecutionModeDecision,
    source_path: String,
    source_bytes_sha256: String,
    canonical_ddn_sha256: String,
    debug_evaluator: Option<DebugEvaluator>,
    last_debug_evidence: JsonValue,
    last_public_observations: Vec<JsonValue>,
    public_observation_frames: Vec<JsonValue>,
    public_observation_trace: JsonValue,
    public_observation_trace_hash: String,
    public_observation_table_projection: JsonValue,
    project_asset_registry: Option<Arc<crate::gate0_registry::ProjectAssetRegistry>>,
}

fn update_wasm_logic_with_supported_source_profile(
    vm: &mut DdnWasmVm,
    source: &str,
    source_profile_identity: &str,
) -> Result<(), JsValue> {
    let profile = resolve_supported_source_profile(Some(source_profile_identity))
        .map_err(|error| JsValue::from_str(&format!("{}: {}", error.code, error.message)))?;
    let program = DdnProgram::from_supported_source_profile_with_mode(
        source,
        &vm.source_path,
        vm.lang_mode,
        profile,
    )
    .map_err(|error| JsValue::from_str(&error))?;
    let canonical_ddn = program.canonical_ddn();
    vm.configured_madi = program.configured_madi().unwrap_or(0);
    vm.execution_mode_decision = program.execution_mode_decision();
    vm.parse_warnings = program.parse_warnings().to_vec();
    vm.runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME);
    vm.runner
        .set_project_asset_registry(vm.project_asset_registry.clone());
    vm.source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
    vm.canonical_ddn_sha256 = runtime_surface::sha256_hex(canonical_ddn.as_bytes());
    vm.debug_evaluator = None;
    vm.last_debug_evidence = json!({"triggered": false, "reasons": []});
    reset_wasm_public_observation_surface(vm).map_err(|error| JsValue::from_str(&error))?;
    Ok(())
}

fn apply_currentline_result_to_vm(
    vm: &mut DdnWasmVm,
    result: ddonirang_lang::CurrentLineResult,
    source_profile_identity: Option<&str>,
) -> Result<JsValue, JsValue> {
    let _ = vm.reset(Some(false))?;
    if let Some(source_profile_identity) = source_profile_identity {
        update_wasm_logic_with_supported_source_profile(
            vm,
            &result.project_source,
            source_profile_identity,
        )?;
    } else {
        vm.update_logic(&result.project_source)?;
    }
    let state = vm.run_ticks(1)?;
    let state_text = state
        .as_string()
        .ok_or_else(|| JsValue::from_str("apply_currentline_cell: state JSON 누락"))?;
    let mut payload: JsonValue = serde_json::from_str(&state_text).map_err(|error| {
        JsValue::from_str(&format!(
            "apply_currentline_cell: state JSON 파싱 실패: {error}"
        ))
    })?;
    let context: JsonValue = serde_json::from_str(&result.context_json).map_err(|error| {
        JsValue::from_str(&format!(
            "apply_currentline_cell: context JSON 생성 실패: {error}"
        ))
    })?;
    if let Some(map) = payload.as_object_mut() {
        map.insert("currentline_context".to_string(), context.clone());
        if let Some(state) = map.get_mut("state").and_then(JsonValue::as_object_mut) {
            state.insert("currentline_context".to_string(), context);
        }
    }
    Ok(JsValue::from_str(&payload.to_string()))
}

fn refresh_wasm_public_observation_surface(vm: &mut DdnWasmVm) -> Result<(), String> {
    let surface = if vm.public_observation_frames.is_empty() {
        build_wasm_public_observation_surface(
            &vm.source_path,
            &vm.source_bytes_sha256,
            &vm.canonical_ddn_sha256,
            &vm.public_observation_frames,
        )?
    } else {
        append_wasm_public_observation_surface(vm)?
    };
    vm.public_observation_trace = surface.trace;
    vm.public_observation_trace_hash = surface.trace_hash;
    vm.public_observation_table_projection = surface.table_projection;
    Ok(())
}

fn reset_wasm_public_observation_surface(vm: &mut DdnWasmVm) -> Result<(), String> {
    vm.last_public_observations.clear();
    vm.public_observation_frames.clear();
    refresh_wasm_public_observation_surface(vm)
}

struct RestoredInputState {
    keys_pressed: u64,
    last_key_name: String,
    pointer_x_i32: i32,
    pointer_y_i32: i32,
    rng_seed: u64,
    dt: Fixed64,
}

struct StagedWasmStateRestore {
    world: DetNuri,
    param_overrides: BTreeMap<String, Value>,
    tick_id: u64,
    input: RestoredInputState,
    public_observation_frames: Vec<JsonValue>,
    last_public_observations: Vec<JsonValue>,
    observation_surface: PublicObservationSurface,
}

fn require_schema(
    object: &Map<String, JsonValue>,
    key: &str,
    expected: &str,
    code: &str,
) -> Result<(), String> {
    let actual = object.get(key).and_then(JsonValue::as_str);
    if actual != Some(expected) {
        return Err(format!(
            "{code}: expected {expected}, got {}",
            actual.unwrap_or("<missing>")
        ));
    }
    Ok(())
}

fn parse_restored_input(payload: Option<&JsonValue>) -> Result<RestoredInputState, String> {
    let Some(payload) = payload else {
        return Ok(RestoredInputState {
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            rng_seed: 0,
            dt: Fixed64::from_i64(1),
        });
    };
    let JsonValue::Object(obj) = payload else {
        return Err("E_WASM_STATE_INPUT_INVALID: state.input은 object여야 합니다".to_string());
    };
    let parse_u64 = |key: &str| -> Result<Option<u64>, String> {
        obj.get(key)
            .map(|value| {
                json_to_u64(value)
                    .ok_or_else(|| format!("E_WASM_STATE_INPUT_INVALID: state.input.{key}"))
            })
            .transpose()
    };
    let parse_i32 = |key: &str| -> Result<Option<i32>, String> {
        obj.get(key)
            .map(|value| {
                json_to_i32(value)
                    .ok_or_else(|| format!("E_WASM_STATE_INPUT_INVALID: state.input.{key}"))
            })
            .transpose()
    };
    let last_key_name = match obj.get("last_key_name") {
        Some(JsonValue::String(value)) => value.clone(),
        Some(_) => {
            return Err(
                "E_WASM_STATE_INPUT_INVALID: state.input.last_key_name은 string이어야 합니다"
                    .to_string(),
            )
        }
        None => String::new(),
    };
    let dt = match obj.get("dt") {
        Some(value) => json_to_fixed64(value)
            .ok_or_else(|| "E_WASM_STATE_INPUT_INVALID: state.input.dt".to_string())?,
        None => Fixed64::from_i64(1),
    };
    Ok(RestoredInputState {
        keys_pressed: parse_u64("keys_pressed")?.unwrap_or(0),
        last_key_name,
        pointer_x_i32: parse_i32("pointer_x_i32")?.unwrap_or(0),
        pointer_y_i32: parse_i32("pointer_y_i32")?.unwrap_or(0),
        rng_seed: parse_u64("rng_base_seed")?
            .or(parse_u64("rng_seed")?)
            .unwrap_or(0),
        dt,
    })
}

fn stage_wasm_state_restore(
    payload: &JsonValue,
    source_path: &str,
    source_bytes_sha256: &str,
    canonical_ddn_sha256: &str,
) -> Result<StagedWasmStateRestore, String> {
    let state = payload
        .get("state")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| "E_WASM_STATE_CHANNEL_MISSING: state object가 필요합니다".to_string())?;
    if state.contains_key("snapshot_v1") {
        return Err(
            "E_WASM_STATE_SNAPSHOT_MIXED_VERSION: snapshot_v1은 snapshot_v2와 함께 읽지 않습니다"
                .to_string(),
        );
    }
    let snapshot = state
        .get("snapshot_v2")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| {
            "E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH: snapshot_v2가 필요합니다".to_string()
        })?;
    require_schema(
        snapshot,
        "schema",
        WASM_STATE_SNAPSHOT_SCHEMA_V2,
        "E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH",
    )?;
    require_schema(
        snapshot,
        "resource_snapshot_schema",
        RESOURCE_SNAPSHOT_SCHEMA_V2,
        "E_RESOURCE_SNAPSHOT_VERSION_MISMATCH",
    )?;
    require_schema(
        snapshot,
        "unit_wire_schema",
        UNIT_VALUE_WIRE_SCHEMA_V2,
        "E_UNIT_WIRE_CHECKPOINT_MISMATCH",
    )?;

    let resources = snapshot
        .get("resources")
        .ok_or_else(|| "E_RESOURCE_SNAPSHOT_MISSING: snapshot_v2.resources".to_string())?;
    let mut world = DetNuri::new();
    restore_world_resources(world.world_mut(), resources)?;

    let param_overrides = snapshot
        .get("param_overrides")
        .ok_or_else(|| "E_WASM_STATE_PARAMS_MISSING: snapshot_v2.param_overrides".to_string())
        .and_then(parse_param_overrides)?;
    let public_observation_frames = snapshot
        .get("public_observation_frames")
        .and_then(JsonValue::as_array)
        .cloned()
        .ok_or_else(|| {
            "E_WASM_STATE_OBSERVATION_FRAMES_INVALID: public_observation_frames array가 필요합니다"
                .to_string()
        })?;
    let tick_id = snapshot
        .get("next_tick_id")
        .and_then(json_to_u64)
        .ok_or_else(|| "E_WASM_STATE_TICK_INVALID: next_tick_id가 필요합니다".to_string())?;
    let input = parse_restored_input(state.get("input"))?;
    let observation_surface = build_wasm_public_observation_surface(
        source_path,
        source_bytes_sha256,
        canonical_ddn_sha256,
        &public_observation_frames,
    )?;
    let last_public_observations = public_observation_frames
        .last()
        .and_then(|frame| frame.get("public_observations"))
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();

    Ok(StagedWasmStateRestore {
        world,
        param_overrides,
        tick_id,
        input,
        public_observation_frames,
        last_public_observations,
        observation_surface,
    })
}

#[wasm_bindgen]
impl DdnWasmVm {
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str) -> Result<DdnWasmVm, JsValue> {
        let source_path = "<wasm>";
        let (program, execution_mode_decision) =
            parse_program_for_wasm(source, source_path, ParseMode::Strict)?;
        let configured_madi = program.configured_madi().unwrap_or(0);
        let parse_warnings = program.parse_warnings().to_vec();
        let prepared = runtime_surface::prepare_source_for_execution(source)
            .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?;
        let source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
        let canonical_ddn_sha256 = runtime_surface::sha256_hex(prepared.canonical_ddn.as_bytes());
        let observation_surface = build_wasm_public_observation_surface(
            source_path,
            &source_bytes_sha256,
            &canonical_ddn_sha256,
            &[],
        )
        .map_err(|err| JsValue::from_str(&err))?;
        let mut defaults = HashMap::new();
        seed_bogae_defaults(&mut defaults);
        Ok(DdnWasmVm {
            runner: DdnRunner::new(program, DEFAULT_UPDATE_NAME),
            world: DetNuri::new(),
            defaults,
            param_overrides: BTreeMap::new(),
            tick_id: 0,
            rng_seed: 0,
            view_prefixes: DEFAULT_VIEW_PREFIXES
                .iter()
                .map(|prefix| (*prefix).to_string())
                .collect(),
            input_keys_pressed: 0,
            input_last_key_name: String::new(),
            input_pointer_x_i32: 0,
            input_pointer_y_i32: 0,
            input_dt: Fixed64::from_i64(1),
            last_patch: None,
            pending_ai_injections: Vec::new(),
            lang_mode: ParseMode::Strict,
            parse_warnings,
            configured_madi,
            execution_mode_decision,
            source_path: source_path.to_string(),
            source_bytes_sha256,
            canonical_ddn_sha256,
            debug_evaluator: None,
            last_debug_evidence: json!({"triggered": false, "reasons": []}),
            last_public_observations: Vec::new(),
            public_observation_frames: Vec::new(),
            public_observation_trace: observation_surface.trace,
            public_observation_trace_hash: observation_surface.trace_hash,
            public_observation_table_projection: observation_surface.table_projection,
            project_asset_registry: None,
        })
    }

    pub fn new_with_mode(source: &str, mode: &str) -> Result<DdnWasmVm, JsValue> {
        let source_path = "<wasm>";
        let mode = parse_mode_from_str(mode)?;
        let (program, execution_mode_decision) = parse_program_for_wasm(source, source_path, mode)?;
        let configured_madi = program.configured_madi().unwrap_or(0);
        let parse_warnings = program.parse_warnings().to_vec();
        let prepared = runtime_surface::prepare_source_for_execution(source)
            .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?;
        let source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
        let canonical_ddn_sha256 = runtime_surface::sha256_hex(prepared.canonical_ddn.as_bytes());
        let observation_surface = build_wasm_public_observation_surface(
            source_path,
            &source_bytes_sha256,
            &canonical_ddn_sha256,
            &[],
        )
        .map_err(|err| JsValue::from_str(&err))?;
        let mut defaults = HashMap::new();
        seed_bogae_defaults(&mut defaults);
        Ok(DdnWasmVm {
            runner: DdnRunner::new(program, DEFAULT_UPDATE_NAME),
            world: DetNuri::new(),
            defaults,
            param_overrides: BTreeMap::new(),
            tick_id: 0,
            rng_seed: 0,
            view_prefixes: DEFAULT_VIEW_PREFIXES
                .iter()
                .map(|prefix| (*prefix).to_string())
                .collect(),
            input_keys_pressed: 0,
            input_last_key_name: String::new(),
            input_pointer_x_i32: 0,
            input_pointer_y_i32: 0,
            input_dt: Fixed64::from_i64(1),
            last_patch: None,
            pending_ai_injections: Vec::new(),
            lang_mode: mode,
            parse_warnings,
            configured_madi,
            execution_mode_decision,
            source_path: source_path.to_string(),
            source_bytes_sha256,
            canonical_ddn_sha256,
            debug_evaluator: None,
            last_debug_evidence: json!({"triggered": false, "reasons": []}),
            last_public_observations: Vec::new(),
            public_observation_frames: Vec::new(),
            public_observation_trace: observation_surface.trace,
            public_observation_trace_hash: observation_surface.trace_hash,
            public_observation_table_projection: observation_surface.table_projection,
            project_asset_registry: None,
        })
    }

    pub fn new_with_source_path(source: &str, source_path: &str) -> Result<DdnWasmVm, JsValue> {
        let source_path = require_wasm_source_path(source_path)?;
        let mut vm = DdnWasmVm::new(source)?;
        vm.update_logic_with_source_path(source, source_path)?;
        Ok(vm)
    }

    pub fn update_logic(&mut self, source: &str) -> Result<(), JsValue> {
        let source_path = self.source_path.clone();
        self.update_logic_with_source_path(source, &source_path)
    }

    pub fn update_logic_with_source_profile(
        &mut self,
        source: &str,
        source_profile_identity: &str,
    ) -> Result<(), JsValue> {
        update_wasm_logic_with_supported_source_profile(self, source, source_profile_identity)
    }

    pub fn update_logic_with_source_path(
        &mut self,
        source: &str,
        source_path: &str,
    ) -> Result<(), JsValue> {
        let source_path = require_wasm_source_path(source_path)?;
        let (program, execution_mode_decision) =
            parse_program_for_wasm(source, source_path, self.lang_mode)?;
        let prepared = runtime_surface::prepare_source_for_execution(source)
            .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?;
        self.configured_madi = program.configured_madi().unwrap_or(0);
        self.execution_mode_decision = execution_mode_decision;
        self.parse_warnings = program.parse_warnings().to_vec();
        self.runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME);
        self.runner
            .set_project_asset_registry(self.project_asset_registry.clone());
        self.source_path = source_path.to_string();
        self.source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
        self.canonical_ddn_sha256 = runtime_surface::sha256_hex(prepared.canonical_ddn.as_bytes());
        self.debug_evaluator = None;
        self.last_debug_evidence = json!({"triggered": false, "reasons": []});
        reset_wasm_public_observation_surface(self).map_err(|err| JsValue::from_str(&err))?;
        Ok(())
    }

    pub fn get_configured_madi(&self) -> u64 {
        self.configured_madi
    }

    pub fn get_execution_mode_decision_json(&self) -> String {
        runtime_surface::execution_mode_decision_json(self.execution_mode_decision).to_string()
    }

    pub fn update_logic_with_mode(&mut self, source: &str, mode: &str) -> Result<(), JsValue> {
        let mode = parse_mode_from_str(mode)?;
        let (program, execution_mode_decision) =
            parse_program_for_wasm(source, &self.source_path, mode)?;
        let prepared = runtime_surface::prepare_source_for_execution(source)
            .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?;
        self.configured_madi = program.configured_madi().unwrap_or(0);
        self.execution_mode_decision = execution_mode_decision;
        self.parse_warnings = program.parse_warnings().to_vec();
        self.runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME);
        self.runner
            .set_project_asset_registry(self.project_asset_registry.clone());
        self.lang_mode = mode;
        self.source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
        self.canonical_ddn_sha256 = runtime_surface::sha256_hex(prepared.canonical_ddn.as_bytes());
        self.debug_evaluator = None;
        self.last_debug_evidence = json!({"triggered": false, "reasons": []});
        reset_wasm_public_observation_surface(self).map_err(|err| JsValue::from_str(&err))?;
        Ok(())
    }

    pub fn configure_project_assets(
        &mut self,
        project_source_json: &str,
    ) -> Result<String, JsValue> {
        let registry = local_project::project_asset_registry_from_json(project_source_json)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        let identities = registry
            .as_deref()
            .map(crate::gate0_registry::ProjectAssetRegistry::sealed_dataset_identities)
            .transpose()
            .map_err(|err| JsValue::from_str(&err))?
            .unwrap_or_default();
        self.project_asset_registry = registry;
        self.runner
            .set_project_asset_registry(self.project_asset_registry.clone());
        Ok(json!({
            "schema": "ddn.wasm.project_asset_registry.v1",
            "sealed_dataset_assets": identities,
        })
        .to_string())
    }

    pub fn get_debug_source_identity_json(&self) -> String {
        json!({
            "source_bytes_sha256": self.source_bytes_sha256,
            "canonical_ddn_sha256": self.canonical_ddn_sha256,
        })
        .to_string()
    }

    pub fn configure_debug_plan(&mut self, plan_json: &str) -> Result<JsValue, JsValue> {
        let plan: JsonValue = serde_json::from_str(plan_json)
            .map_err(|err| JsValue::from_str(&format!("E_DEBUG_PLAN_JSON_INVALID: {err}")))?;
        let evaluator = DebugEvaluator::from_plan(&plan, Some(&self.source_bytes_sha256))
            .map_err(|err| JsValue::from_str(&err))?;
        let plan_hash = evaluator.plan_hash().to_string();
        self.debug_evaluator = Some(evaluator);
        self.last_debug_evidence =
            json!({"triggered": false, "reasons": [], "debug_plan_hash": plan_hash});
        Ok(JsValue::from_str(
            &json!({"ok": true, "debug_plan_hash": plan_hash}).to_string(),
        ))
    }

    pub fn clear_debug_plan(&mut self) -> JsValue {
        self.debug_evaluator = None;
        self.last_debug_evidence = json!({"triggered": false, "reasons": []});
        JsValue::from_str(&json!({"ok": true}).to_string())
    }

    pub fn get_build_info(&self) -> String {
        format!(
            "pkg={} version={} state_schema={} resource_schema={} unit_wire={} update={}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            ENGINE_RESPONSE_SCHEMA,
            RESOURCE_SNAPSHOT_SCHEMA_V2,
            UNIT_VALUE_WIRE_SCHEMA_V2,
            DEFAULT_UPDATE_NAME
        )
    }

    pub fn get_parse_warnings(&self) -> JsValue {
        let warnings: Vec<JsonValue> = self
            .parse_warnings
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
            .collect();
        JsValue::from_str(&json!({ "warnings": warnings }).to_string())
    }

    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng_seed = seed;
    }

    pub fn add_view_prefix(&mut self, prefix: &str) {
        let trimmed = prefix.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.view_prefixes.iter().any(|p| p == trimmed) {
            return;
        }
        self.view_prefixes.push(trimmed.to_string());
    }

    pub fn clear_view_prefixes(&mut self) {
        self.view_prefixes.clear();
    }

    pub fn set_input(
        &mut self,
        keys_pressed: u32,
        last_key_name: &str,
        pointer_x_i32: i32,
        pointer_y_i32: i32,
        dt: f64,
    ) {
        self.input_keys_pressed = u64::from(keys_pressed);
        self.input_last_key_name = last_key_name.to_string();
        self.input_pointer_x_i32 = pointer_x_i32;
        self.input_pointer_y_i32 = pointer_y_i32;
        if dt.is_finite() {
            self.input_dt = fixed64_from_f64_checked(dt);
        }
    }

    pub fn set_keys_pressed(&mut self, keys_pressed: u32) {
        self.input_keys_pressed = u64::from(keys_pressed);
    }

    pub fn set_last_key_name(&mut self, last_key_name: &str) {
        self.input_last_key_name = last_key_name.to_string();
    }

    pub fn set_pointer(&mut self, pointer_x_i32: i32, pointer_y_i32: i32) {
        self.input_pointer_x_i32 = pointer_x_i32;
        self.input_pointer_y_i32 = pointer_y_i32;
    }

    pub fn set_dt_f64(&mut self, dt: f64) {
        if dt.is_finite() {
            self.input_dt = fixed64_from_f64_checked(dt);
        }
    }

    pub fn columns(&self) -> JsValue {
        let columns = self
            .public_observation_trace
            .get("columns")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        let rows = self
            .public_observation_trace
            .get("samples")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        let payload = json!({
            "observation_columns": columns,
            "output_rows": rows,
            "public_observation_trace": self.public_observation_trace,
            "public_observation_trace_hash": self.public_observation_trace_hash,
            "public_observation_table_projection": self.public_observation_table_projection,
            "deprecated": true,
            "use_instead": "step_one() 또는 get_state_json()의 public_observation_trace",
        });
        JsValue::from_str(&payload.to_string())
    }

    pub fn set_param(&mut self, key: &str, value: JsValue) -> Result<JsValue, JsValue> {
        let key = key.trim();
        if key.is_empty() {
            return Err(wasm_error(
                "PARAM_KEY_EMPTY",
                "set_param: key가 비어 있습니다",
                "변수 이름을 입력해 주세요. 예: set_param(\"속도\", 10)",
            ));
        }
        let parsed = js_scalar_to_runtime_value(&value).ok_or_else(|| {
            wasm_error(
                "PARAM_VALUE_INVALID",
                "set_param: value는 수/참거짓/글 스칼라만 허용됩니다",
                "수/참거짓/글 스칼라만 전달해 주세요.",
            )
        })?;
        apply_param_value(self.world.world_mut(), key, &parsed).map_err(|err| {
            wasm_error(
                "PARAM_APPLY_FAILED",
                &err,
                "대상 변수 이름과 타입을 확인해 주세요.",
            )
        })?;
        self.param_overrides.insert(key.to_string(), parsed);

        let payload = json!({
            "ok": true,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn set_param_fixed64(&mut self, key: &str, raw_i64: i64) -> Result<JsValue, JsValue> {
        let key = key.trim();
        if key.is_empty() {
            return Err(wasm_error(
                "PARAM_KEY_EMPTY",
                "set_param_fixed64: key가 비어 있습니다",
                "변수 이름을 입력해 주세요. 예: set_param_fixed64(\"속도\", 4294967296)",
            ));
        }
        let parsed = Value::Fixed64(Fixed64::from_raw_i64(raw_i64));
        apply_param_value(self.world.world_mut(), key, &parsed).map_err(|err| {
            wasm_error(
                "PARAM_APPLY_FAILED",
                &err,
                "대상 변수 이름과 타입을 확인해 주세요.",
            )
        })?;
        self.param_overrides.insert(key.to_string(), parsed);
        let payload = json!({
            "ok": true,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
            "raw_i64": raw_i64,
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn set_param_fixed64_str(&mut self, key: &str, raw_i64: &str) -> Result<JsValue, JsValue> {
        let key = key.trim();
        if key.is_empty() {
            return Err(wasm_error(
                "PARAM_KEY_EMPTY",
                "set_param_fixed64_str: key가 비어 있습니다",
                "변수 이름을 입력해 주세요. 예: set_param_fixed64_str(\"속도\", \"4294967296\")",
            ));
        }
        let trimmed = raw_i64.trim();
        if trimmed.is_empty() {
            return Err(wasm_error(
                "PARAM_RAW_I64_EMPTY",
                "set_param_fixed64_str: raw_i64가 비어 있습니다",
                "raw_i64 정수 문자열을 입력해 주세요. 예: \"4294967296\"",
            ));
        }
        let parsed = trimmed.parse::<i64>().map_err(|_| {
            wasm_error(
                "PARAM_RAW_I64_INVALID",
                "set_param_fixed64_str: raw_i64는 i64 정수 문자열이어야 합니다",
                "정수 문자열만 허용됩니다. 예: -123, 4294967296",
            )
        })?;
        let value = Value::Fixed64(Fixed64::from_raw_i64(parsed));
        apply_param_value(self.world.world_mut(), key, &value).map_err(|err| {
            wasm_error(
                "PARAM_APPLY_FAILED",
                &err,
                "대상 변수 이름과 타입을 확인해 주세요.",
            )
        })?;
        self.param_overrides.insert(key.to_string(), value);
        let payload = json!({
            "ok": true,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
            "raw_i64": trimmed,
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn reset(&mut self, keep_params: Option<bool>) -> Result<JsValue, JsValue> {
        let keep_params = keep_params.unwrap_or(false);
        self.world = DetNuri::new();
        self.tick_id = 0;
        self.rng_seed = 0;
        self.input_keys_pressed = 0;
        self.input_last_key_name.clear();
        self.input_pointer_x_i32 = 0;
        self.input_pointer_y_i32 = 0;
        self.input_dt = Fixed64::from_i64(1);
        self.last_patch = None;
        self.pending_ai_injections.clear();
        self.runner.reset_transient_state();
        if let Some(evaluator) = self.debug_evaluator.as_mut() {
            evaluator.reset_history();
        }
        self.last_debug_evidence = json!({"triggered": false, "reasons": []});
        reset_wasm_public_observation_surface(self).map_err(|err| JsValue::from_str(&err))?;

        if !keep_params {
            self.param_overrides.clear();
        } else {
            for (key, value) in &self.param_overrides {
                apply_param_value(self.world.world_mut(), key, value)
                    .map_err(|err| JsValue::from_str(&err))?;
            }
        }

        let (columns, row) = collect_columns_and_row(self.world.world());
        let observation_columns = self
            .public_observation_trace
            .get("columns")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        let output_rows = self
            .public_observation_trace
            .get("samples")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        let payload = json!({
            "ok": true,
            "tick": self.tick_id,
            "columns": columns,
            "row": row,
            "input": serialize_input_state(self),
            "observation_columns": observation_columns,
            "output_rows": output_rows,
            "public_observation_trace": self.public_observation_trace,
            "public_observation_trace_hash": self.public_observation_trace_hash,
            "public_observation_table_projection": self.public_observation_table_projection,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn step_one_with_input(
        &mut self,
        keys_pressed: u32,
        last_key_name: &str,
        pointer_x_i32: i32,
        pointer_y_i32: i32,
        dt: f64,
    ) -> Result<JsValue, JsValue> {
        self.set_input(
            keys_pressed,
            last_key_name,
            pointer_x_i32,
            pointer_y_i32,
            dt,
        );
        self.step_one()
    }

    pub fn step_one(&mut self) -> Result<JsValue, JsValue> {
        let tick_seed = self.rng_seed ^ self.tick_id;
        let ai_injections = self
            .pending_ai_injections
            .drain(..)
            .enumerate()
            .map(|(idx, (key, value_json))| SeulgiPacket {
                agent_id: 0,
                recv_seq: idx as u64,
                accepted_madi: self.tick_id,
                target_madi: self.tick_id,
                source: InputSource::Seulgi,
                intent: SeulgiIntent::Say {
                    text: format!("{key}={value_json}"),
                },
            })
            .collect();
        let input = InputSnapshot {
            tick_id: self.tick_id,
            dt: self.input_dt,
            keys_pressed: self.input_keys_pressed,
            last_key_name: self.input_last_key_name.clone(),
            pointer_x_i32: self.input_pointer_x_i32,
            pointer_y_i32: self.input_pointer_y_i32,
            ai_injections,
            net_events: Vec::new(),
            frame_source: InputSource::Person,
            rng_seed: tick_seed,
        };

        let output = self
            .runner
            .run_update_with_param_overrides(
                self.world.world(),
                &input,
                &self.defaults,
                &self.param_overrides,
            )
            .map_err(|err| JsValue::from_str(&err))?;
        let public_observations = public_observation_events_json(&output.public_observations)
            .map_err(|err| JsValue::from_str(&err))?;

        let mut sink = ddonirang_core::signals::VecSignalSink::default();
        self.world
            .apply_patch(&output.patch, input.tick_id, &mut sink);

        self.last_patch = Some(output.patch);
        let state_hash = current_state_hash(&self.world, &self.view_prefixes);
        let resources = serialize_world_resources(self.world.world());
        let (columns, row) = collect_columns_and_row(self.world.world());
        let streams = collect_streams(self.world.world());
        let view_meta = build_view_meta(self.world.world());
        let view_hash = hash_json_string(&view_meta);
        let primary_view_decision = crate::primary_view::decide(
            &view_meta,
            &self.source_bytes_sha256,
            &self.canonical_ddn_sha256,
            &state_hash,
            &view_hash,
        );
        let snapshot_resources = serialize_world_resources_snapshot(self.world.world());
        let snapshot_params = serialize_param_overrides(&self.param_overrides);
        let patch_json = self
            .last_patch
            .as_ref()
            .map(serialize_patch)
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));

        let input_json = serialize_input_snapshot(&input, Some(self.rng_seed));
        self.public_observation_frames.push(json!({
            "committed_madi": input.tick_id,
            "state_hash": state_hash,
            "input_snapshot": input_json,
            "public_observations": public_observations,
        }));
        if let Err(err) = refresh_wasm_public_observation_surface(self) {
            self.public_observation_frames.pop();
            return Err(JsValue::from_str(&err));
        }
        let observation_columns = self
            .public_observation_trace
            .get("columns")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        let output_rows = self
            .public_observation_trace
            .get("samples")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        self.tick_id = self.tick_id.saturating_add(1);
        self.last_public_observations = public_observations.clone();

        let debug_evidence = if let Some(evaluator) = self.debug_evaluator.as_mut() {
            evaluator
                .evaluate_frame(&json!({
                    "committed_madi": input.tick_id,
                    "state_hash": state_hash,
                    "input_snapshot": input_json,
                    "public_observations": public_observations,
                }))
                .map_err(|err| JsValue::from_str(&err))?
        } else {
            json!({
                "schema": crate::deterministic_debug::DEBUG_EVIDENCE_SCHEMA,
                "triggered": false,
                "committed_madi": input.tick_id,
                "state_hash": state_hash,
                "debug_plan_hash": "",
                "reasons": [],
            })
        };
        self.last_debug_evidence = debug_evidence.clone();
        let payload = json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "tick_id": input.tick_id,
            "committed_madi": input.tick_id,
            "state_hash": state_hash,
            "input": input_json.clone(),
            "resources": resources,
            "channels": columns.clone(),
            "row": row.clone(),
            "observation_columns": observation_columns.clone(),
            "output_rows": output_rows.clone(),
            "public_observation_trace": self.public_observation_trace.clone(),
            "public_observation_trace_hash": self.public_observation_trace_hash.clone(),
            "public_observation_table_projection": self.public_observation_table_projection.clone(),
            "public_observations": public_observations.clone(),
            "debug_pause": debug_evidence.clone(),
            "streams": streams.clone(),
            "patch": patch_json,
            "state": {
                "tick_id": input.tick_id,
                "committed_madi": input.tick_id,
                "input": input_json,
                "resources": serialize_world_resources(self.world.world()),
                "channels": columns,
                "row": row,
                "public_observations": public_observations,
                "debug_pause": debug_evidence,
                "patch": self
                    .last_patch
                    .as_ref()
                    .map(serialize_patch)
                    .unwrap_or_else(|| JsonValue::Array(Vec::new())),
                "streams": streams,
                "snapshot_v2": build_wasm_state_snapshot_v2(
                    snapshot_resources,
                    snapshot_params,
                    self.public_observation_frames.clone(),
                    self.tick_id,
                )
            },
            "view_meta": view_meta,
            "view_hash": view_hash,
            "primary_view_decision": primary_view_decision,
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn run_ticks(&mut self, count: u32) -> Result<JsValue, JsValue> {
        let count = count.max(1);
        let mut state = JsValue::NULL;
        for _ in 0..count {
            state = self.step_one()?;
            if self
                .last_debug_evidence
                .get("triggered")
                .and_then(JsonValue::as_bool)
                == Some(true)
            {
                break;
            }
        }

        if self
            .last_debug_evidence
            .get("triggered")
            .and_then(JsonValue::as_bool)
            == Some(true)
        {
            return Ok(state);
        }

        let input = InputSnapshot {
            tick_id: self.tick_id,
            dt: self.input_dt,
            keys_pressed: self.input_keys_pressed,
            last_key_name: self.input_last_key_name.clone(),
            pointer_x_i32: self.input_pointer_x_i32,
            pointer_y_i32: self.input_pointer_y_i32,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            frame_source: InputSource::Person,
            rng_seed: self.rng_seed ^ self.tick_id,
        };
        let output = self
            .runner
            .run_finish_with_param_overrides(
                self.world.world(),
                &input,
                &self.defaults,
                &self.param_overrides,
            )
            .map_err(|err| JsValue::from_str(&err))?;
        if !output.patch.ops.is_empty() {
            let mut sink = ddonirang_core::signals::VecSignalSink::default();
            self.world
                .apply_patch(&output.patch, input.tick_id, &mut sink);
            self.last_patch = Some(output.patch);
            state = self.get_state_json();
        }
        Ok(state)
    }

    pub fn apply_currentline_cell(
        &mut self,
        source: &str,
        context_json: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let result = ddonirang_lang::apply_currentline_cell(source, context_json.as_deref())
            .map_err(|err| JsValue::from_str(&err))?;
        apply_currentline_result_to_vm(self, result, None)
    }

    /// 명시적 supported source profile을 소비하는 currentline 내부 candidate entry다.
    /// 기존 apply_currentline_cell의 legacy 기본 동작은 바꾸지 않는다.
    pub fn apply_currentline_cell_with_source_profile(
        &mut self,
        source: &str,
        source_profile_identity: &str,
        context_json: Option<String>,
    ) -> Result<JsValue, JsValue> {
        resolve_supported_source_profile(Some(source_profile_identity))
            .map_err(|error| JsValue::from_str(&format!("{}: {}", error.code, error.message)))?;
        let result = ddonirang_lang::apply_currentline_cell_v25(source, context_json.as_deref())
            .map_err(|err| JsValue::from_str(&err))?;
        apply_currentline_result_to_vm(self, result, Some(source_profile_identity))
    }

    pub fn get_state_hash(&self) -> String {
        current_state_hash(&self.world, &self.view_prefixes)
    }

    pub fn get_state_json(&self) -> JsValue {
        let resources = serialize_world_resources(self.world.world());
        let public_observations = self.last_public_observations.clone();
        let (columns, row) = collect_columns_and_row(self.world.world());
        let streams = collect_streams(self.world.world());
        let view_meta = build_view_meta(self.world.world());
        let view_hash = hash_json_string(&view_meta);
        let state_hash = current_state_hash(&self.world, &self.view_prefixes);
        let primary_view_decision = crate::primary_view::decide(
            &view_meta,
            &self.source_bytes_sha256,
            &self.canonical_ddn_sha256,
            &state_hash,
            &view_hash,
        );
        let snapshot_resources = serialize_world_resources_snapshot(self.world.world());
        let snapshot_params = serialize_param_overrides(&self.param_overrides);
        let observation_columns = self
            .public_observation_trace
            .get("columns")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        let output_rows = self
            .public_observation_trace
            .get("samples")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        let committed_madi = last_committed_madi(&self.public_observation_frames);
        let payload = json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "tick_id": self.tick_id,
            "committed_madi": committed_madi.clone(),
            "state_hash": state_hash,
            "input": serialize_input_state(self),
            "resources": resources,
            "channels": columns.clone(),
            "row": row.clone(),
            "observation_columns": observation_columns.clone(),
            "output_rows": output_rows.clone(),
            "public_observation_trace": self.public_observation_trace.clone(),
            "public_observation_trace_hash": self.public_observation_trace_hash.clone(),
            "public_observation_table_projection": self.public_observation_table_projection.clone(),
            "public_observations": public_observations.clone(),
            "debug_pause": self.last_debug_evidence.clone(),
            "streams": streams.clone(),
            "state": {
                "tick_id": self.tick_id,
                "committed_madi": committed_madi,
                "input": serialize_input_state(self),
                "resources": serialize_world_resources(self.world.world()),
                "channels": columns,
                "row": row,
                "public_observations": public_observations,
                "debug_pause": self.last_debug_evidence.clone(),
                "patch": JsonValue::Array(Vec::new()),
                "streams": streams,
                "snapshot_v2": build_wasm_state_snapshot_v2(
                    snapshot_resources,
                    snapshot_params,
                    self.public_observation_frames.clone(),
                    self.tick_id,
                )
            },
            "view_meta": view_meta,
            "view_hash": view_hash,
            "primary_view_decision": primary_view_decision,
        });
        JsValue::from_str(&payload.to_string())
    }

    pub fn get_input_state_json(&self) -> JsValue {
        JsValue::from_str(&serialize_input_state(self).to_string())
    }

    pub fn restore_state(&mut self, state_json: &str) -> Result<JsValue, JsValue> {
        let payload: JsonValue = serde_json::from_str(state_json)
            .map_err(|err| JsValue::from_str(&format!("restore_state: JSON 파싱 실패: {err}")))?;
        let schema = payload
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if schema != ENGINE_RESPONSE_SCHEMA {
            return Err(JsValue::from_str(&format!(
                "restore_state: 지원하지 않는 schema: {schema}"
            )));
        }

        let staged = stage_wasm_state_restore(
            &payload,
            &self.source_path,
            &self.source_bytes_sha256,
            &self.canonical_ddn_sha256,
        )
        .map_err(|err| JsValue::from_str(&format!("restore_state: {err}")))?;
        let state_hash = current_state_hash(&staged.world, &self.view_prefixes);
        let view_hash = hash_json_string(&build_view_meta(staged.world.world()));
        let diag = restore_hash_diags(&payload, &state_hash, &view_hash);

        self.world = staged.world;
        self.param_overrides = staged.param_overrides;
        self.tick_id = staged.tick_id;
        self.rng_seed = staged.input.rng_seed;
        self.input_keys_pressed = staged.input.keys_pressed;
        self.input_last_key_name = staged.input.last_key_name;
        self.input_pointer_x_i32 = staged.input.pointer_x_i32;
        self.input_pointer_y_i32 = staged.input.pointer_y_i32;
        self.input_dt = staged.input.dt;
        self.last_patch = None;
        self.pending_ai_injections.clear();
        self.runner.reset_transient_state();
        self.debug_evaluator = None;
        self.last_debug_evidence = json!({"triggered": false, "reasons": []});
        self.last_public_observations = staged.last_public_observations;
        self.public_observation_frames = staged.public_observation_frames;
        self.public_observation_trace = staged.observation_surface.trace;
        self.public_observation_trace_hash = staged.observation_surface.trace_hash;
        self.public_observation_table_projection = staged.observation_surface.table_projection;

        let (columns, row) = collect_columns_and_row(self.world.world());
        let observation_columns = self
            .public_observation_trace
            .get("columns")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        let output_rows = self
            .public_observation_trace
            .get("samples")
            .cloned()
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
        let out = json!({
            "ok": true,
            "tick": self.tick_id,
            "committed_madi": last_committed_madi(&self.public_observation_frames),
            "columns": columns,
            "row": row,
            "observation_columns": observation_columns,
            "output_rows": output_rows,
            "public_observation_trace": self.public_observation_trace,
            "public_observation_trace_hash": self.public_observation_trace_hash,
            "public_observation_table_projection": self.public_observation_table_projection,
            "state_hash": state_hash,
            "view_hash": view_hash,
            "diag": diag,
        });
        Ok(JsValue::from_str(&out.to_string()))
    }

    pub fn inject_ai_action(&mut self, key: &str, value_json: &str) {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return;
        }
        self.pending_ai_injections
            .push((trimmed.to_string(), value_json.to_string()));
    }

    pub fn clear_ai_injections(&mut self) {
        self.pending_ai_injections.clear();
    }
}

fn current_state_hash(world: &DetNuri, view_prefixes: &[String]) -> String {
    let refs: Vec<&str> = view_prefixes.iter().map(String::as_str).collect();
    format!(
        "blake3:{}",
        world
            .world()
            .state_hash_excluding_resource_prefixes(&refs)
            .to_hex()
    )
}

fn hash_json_string(value: &JsonValue) -> String {
    let canonical = value.to_string();
    let digest = blake3_hash(canonical.as_bytes());
    format!("blake3:{}", hex::encode(digest.as_bytes()))
}

fn restore_hash_diags(
    payload: &JsonValue,
    actual_state_hash: &str,
    actual_view_hash: &str,
) -> Vec<JsonValue> {
    let mut diags = Vec::new();

    if let Some(expected_state_hash) = payload.get("state_hash").and_then(JsonValue::as_str) {
        if expected_state_hash != actual_state_hash {
            diags.push(json!({
                "code": "STATE_HASH_MISMATCH",
                "expected": expected_state_hash,
                "actual": actual_state_hash,
                "detail": "복원 후 state_hash 불일치",
            }));
        }
    }

    if let Some(expected_view_hash) = payload.get("view_hash").and_then(JsonValue::as_str) {
        if expected_view_hash != actual_view_hash {
            diags.push(json!({
                "code": "VIEW_HASH_MISMATCH",
                "expected": expected_view_hash,
                "actual": actual_view_hash,
                "detail": "복원 후 view_hash 불일치 (view_meta 변경 가능)",
            }));
        }
    }

    diags
}

fn wasm_error(code: &str, detail: &str, hint: &str) -> JsValue {
    let payload = json!({
        "error": true,
        "code": code,
        "detail": detail,
        "hint": hint,
    });
    JsValue::from_str(&payload.to_string())
}

fn fixed64_from_f64_checked(value: f64) -> Fixed64 {
    let result = Fixed64::from_f64_lossy(value);
    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    {
        let roundtrip = result.raw_i64() as f64 / Fixed64::ONE_RAW as f64;
        if (value - roundtrip).abs() > 1e-10 {
            web_sys::console::warn_1(
                &format!("Fixed64 정밀도 손실: {value} -> {roundtrip}").into(),
            );
        }
    }
    result
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
    if let Some(graph_json) = runtime_surface::derive_graph_from_points(world) {
        if let Ok(graph) = serde_json::from_str::<JsonValue>(&graph_json) {
            out.insert("graph".to_string(), graph);
            available_families.push("graph");
        }
    }
    if let Some(workbench) = build_workbench_authoritative_results(world, out.get("space2d")) {
        out.insert("workbench".to_string(), workbench);
    }

    let mut graph_hints = Vec::new();
    for (tag, id, label) in GRAPH_POINTS_TAGS.iter() {
        let Some(value) = world.get_resource_value(tag) else {
            continue;
        };
        let Some(points) = points_from_resource_value(&value) else {
            continue;
        };
        if points.is_empty() {
            continue;
        }
        graph_hints.push(json!({
            "series_id": *id,
            "source": *tag,
            "y_label": *label,
            "overlay": true,
        }));
    }
    out.insert("graph_hints".to_string(), JsonValue::Array(graph_hints));
    append_view_stack_meta(&mut out, &available_families);

    JsonValue::Object(out)
}

fn world_number(world: &ddonirang_core::platform::NuriWorld, tag: &str) -> Option<f64> {
    world
        .get_resource_fixed64(tag)
        .map(fixed64_to_f64)
        .or_else(|| {
            world
                .get_resource_value(tag)
                .and_then(|value| resource_value_to_number(&value))
        })
        .filter(|value| value.is_finite())
}

fn optional_world_number_json(world: &ddonirang_core::platform::NuriWorld, tag: &str) -> JsonValue {
    world_number(world, tag).map_or(JsonValue::Null, |value| json!(value))
}

fn workbench_result(schema: &str, family: &str, content: JsonValue) -> JsonValue {
    json!({
        "schema": schema,
        "family": family,
        "status": "available",
        "producer": "ddonirang_tool.wasm_runtime",
        "provenance": "native_authoritative_structured_result",
        "content": content,
    })
}

fn space2d_dimensions(space2d: &JsonValue) -> Option<JsonValue> {
    let camera = space2d.get("camera")?.as_object()?;
    let x_min = camera.get("x_min")?.as_f64()?;
    let x_max = camera.get("x_max")?.as_f64()?;
    let y_min = camera.get("y_min")?.as_f64()?;
    let y_max = camera.get("y_max")?.as_f64()?;
    let columns = (x_max - x_min).abs();
    let rows = (y_max - y_min).abs();
    if !columns.is_finite() || !rows.is_finite() || columns <= 0.0 || rows <= 0.0 {
        return None;
    }
    Some(json!({ "columns": columns, "rows": rows }))
}

fn build_workbench_authoritative_results(
    world: &ddonirang_core::platform::NuriWorld,
    space2d: Option<&JsonValue>,
) -> Option<JsonValue> {
    let mut results = Map::new();

    if let (Some(x), Some(y), Some(a), Some(b)) = (
        world_number(world, "x"),
        world_number(world, "y"),
        world_number(world, "a"),
        world_number(world, "b"),
    ) {
        results.insert(
            "line".to_string(),
            workbench_result(
                WORKBENCH_LINE_RESULT_SCHEMA,
                "line",
                json!({
                    "current": { "x": x, "y": y },
                    "equation": { "a": a, "b": b },
                }),
            ),
        );
    }

    if let (Some(theta), Some(omega), Some(length), Some(bob_x), Some(bob_y)) = (
        world_number(world, "theta"),
        world_number(world, "omega"),
        world_number(world, "L"),
        world_number(world, "bob_x"),
        world_number(world, "bob_y"),
    ) {
        if length > 0.0 {
            results.insert(
                "pendulum".to_string(),
                workbench_result(
                    WORKBENCH_PENDULUM_RESULT_SCHEMA,
                    "pendulum",
                    json!({
                        "current": {
                            "theta": theta,
                            "omega": omega,
                            "energy": optional_world_number_json(world, "energy"),
                            "kinetic_energy": optional_world_number_json(world, "kinetic_energy"),
                            "potential_energy": optional_world_number_json(world, "potential_energy"),
                            "length": length,
                            "physical_bob": { "x": bob_x, "y": bob_y },
                            "normalized_bob": { "x": bob_x / length, "y": -bob_y / length },
                        }
                    }),
                ),
            );
        }
    }

    if let (Some(t), Some(x), Some(y), Some(vx), Some(vy)) = (
        world_number(world, "t"),
        world_number(world, "x"),
        world_number(world, "y"),
        world_number(world, "vx"),
        world_number(world, "vy"),
    ) {
        results.insert(
            "projectile".to_string(),
            workbench_result(
                WORKBENCH_PROJECTILE_RESULT_SCHEMA,
                "projectile",
                json!({
                    "current": { "t": t, "x": x, "y": y, "vx": vx, "vy": vy },
                }),
            ),
        );
    }

    let grid_position = world
        .get_resource_value("위치")
        .and_then(|value| resource_value_to_point(&value));
    let grid_space2d = space2d.filter(|value| {
        value
            .get("drawlist")
            .and_then(JsonValue::as_array)
            .is_some_and(|items| !items.is_empty())
    });
    if let (Some((x, y)), Some(space2d)) = (grid_position, grid_space2d) {
        results.insert(
            "grid".to_string(),
            workbench_result(
                WORKBENCH_GRID_RESULT_SCHEMA,
                "grid",
                json!({
                    "position": { "x": x, "y": y },
                    "dimensions": space2d_dimensions(space2d),
                    "space2d": space2d.clone(),
                }),
            ),
        );
    }

    if results.is_empty() {
        return None;
    }
    Some(json!({
        "schema": WORKBENCH_RESULT_SET_SCHEMA,
        "producer": "ddonirang_tool.wasm_runtime",
        "provenance": "native_authoritative_structured_result",
        "results": results,
    }))
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

fn collect_streams(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    let sidecars = collect_stream_sidecars(world);
    for (tag, value) in world.resource_value_entries() {
        let meta = sidecars.get(&tag).copied();
        if !is_stream_resource(&tag, &value, meta.as_ref()) {
            continue;
        }
        let Some(stream) = extract_ring_buffer(&value, meta.as_ref()) else {
            continue;
        };
        out.insert(tag, stream);
    }
    JsonValue::Object(out)
}

#[derive(Clone, Copy, Default)]
struct StreamMeta {
    capacity: Option<u64>,
    head: Option<u64>,
    len: Option<u64>,
}

impl StreamMeta {
    fn has_any(self) -> bool {
        self.capacity.is_some() || self.head.is_some() || self.len.is_some()
    }

    fn set_from_kind(&mut self, kind: &str, value: Option<u64>) {
        if value.is_none() {
            return;
        }
        match kind {
            "capacity" => self.capacity = value,
            "head" => self.head = value,
            "len" => self.len = value,
            _ => {}
        }
    }
}

fn collect_stream_sidecars(
    world: &ddonirang_core::platform::NuriWorld,
) -> HashMap<String, StreamMeta> {
    let mut out: HashMap<String, StreamMeta> = HashMap::new();
    for (tag, value) in world.resource_fixed64_entries() {
        let Some((base, kind)) = parse_stream_sidecar_tag(&tag) else {
            continue;
        };
        let numeric = u64::try_from(value.raw_i64()).ok();
        out.entry(base).or_default().set_from_kind(kind, numeric);
    }
    for (tag, value) in world.resource_value_entries() {
        let Some((base, kind)) = parse_stream_sidecar_tag(&tag) else {
            continue;
        };
        let numeric = resource_value_to_u64(&value);
        out.entry(base).or_default().set_from_kind(kind, numeric);
    }
    out
}

fn parse_stream_sidecar_tag(tag: &str) -> Option<(String, &'static str)> {
    const PATTERNS: [(&str, &str); 16] = [
        ("_stream_capacity", "capacity"),
        (".stream_capacity", "capacity"),
        ("_capacity", "capacity"),
        (".capacity", "capacity"),
        ("_head", "head"),
        (".head", "head"),
        ("_length", "len"),
        (".length", "len"),
        ("_len", "len"),
        (".len", "len"),
        ("_용량", "capacity"),
        (".용량", "capacity"),
        ("_머리", "head"),
        (".머리", "head"),
        ("_길이", "len"),
        (".길이", "len"),
    ];
    let lower = tag.to_ascii_lowercase();
    for (suffix, kind) in PATTERNS {
        let is_match = if suffix.is_ascii() {
            lower.ends_with(suffix)
        } else {
            tag.ends_with(suffix)
        };
        if !is_match {
            continue;
        }
        // "foo_stream_capacity"는 base를 "foo_stream"으로 해석한다.
        let raw_base = if suffix == "_stream_capacity" {
            tag.strip_suffix("_capacity").unwrap_or_default()
        } else if suffix == ".stream_capacity" {
            tag.strip_suffix(".capacity").unwrap_or_default()
        } else {
            tag.strip_suffix(suffix).unwrap_or_default()
        };
        let base = raw_base
            .trim_matches(|c| c == '.' || c == '_' || c == '/')
            .to_string();
        if base.is_empty() {
            continue;
        }
        return Some((base, kind));
    }
    None
}

fn is_stream_resource(tag: &str, value: &ResourceValue, sidecar: Option<&StreamMeta>) -> bool {
    if sidecar.copied().unwrap_or_default().has_any() {
        return matches!(value, ResourceValue::List(_) | ResourceValue::Map(_));
    }
    let lower = tag.to_ascii_lowercase();
    if lower.contains("stream") || tag.contains("흐름") {
        return matches!(value, ResourceValue::List(_) | ResourceValue::Map(_));
    }
    let ResourceValue::Map(entries) = value else {
        return false;
    };
    let mut has_buffer = false;
    let mut has_meta = false;
    for entry in entries.values() {
        let key = resource_key_to_string(&entry.key);
        match key.as_str() {
            "buffer" | "버퍼" => has_buffer = true,
            "capacity" | "stream_capacity" | "용량" | "head" | "머리" | "len" | "length"
            | "길이" => has_meta = true,
            _ => {}
        }
    }
    has_buffer || has_meta
}

fn extract_ring_buffer(value: &ResourceValue, sidecar: Option<&StreamMeta>) -> Option<JsonValue> {
    match value {
        ResourceValue::List(items) => {
            let side = sidecar.copied().unwrap_or_default();
            let mut buffer = items.iter().map(resource_value_to_json).collect::<Vec<_>>();
            let capacity = side
                .capacity
                .unwrap_or(items.len() as u64)
                .max(items.len() as u64);
            if buffer.len() < capacity as usize {
                buffer.resize(capacity as usize, JsonValue::Null);
            }
            let len = side
                .len
                .unwrap_or(items.len() as u64)
                .min(buffer.len() as u64);
            let head = if buffer.is_empty() {
                0
            } else {
                side.head
                    .unwrap_or_else(|| len.saturating_sub(1))
                    .min(buffer.len().saturating_sub(1) as u64)
            };
            Some(json!({
                "capacity": capacity,
                "head": head,
                "len": len,
                "buffer": JsonValue::Array(buffer),
            }))
        }
        ResourceValue::Map(entries) => {
            let side = sidecar.copied().unwrap_or_default();
            let mut buffer_json: Option<Vec<JsonValue>> = None;
            let mut capacity: Option<u64> = None;
            let mut head: Option<u64> = None;
            let mut len: Option<u64> = None;

            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                match key.as_str() {
                    "buffer" | "버퍼" => {
                        buffer_json = match &entry.value {
                            ResourceValue::List(items) => {
                                Some(items.iter().map(resource_value_to_json).collect::<Vec<_>>())
                            }
                            other => Some(vec![resource_value_to_json(other)]),
                        };
                    }
                    "capacity" | "stream_capacity" | "용량" => {
                        capacity = resource_value_to_u64(&entry.value);
                    }
                    "head" | "머리" => {
                        head = resource_value_to_u64(&entry.value);
                    }
                    "len" | "length" | "길이" => {
                        len = resource_value_to_u64(&entry.value);
                    }
                    _ => {}
                }
            }
            if capacity.is_none() {
                capacity = side.capacity;
            }
            if head.is_none() {
                head = side.head;
            }
            if len.is_none() {
                len = side.len;
            }

            let mut buffer = buffer_json.unwrap_or_default();
            let cap = capacity.unwrap_or(buffer.len() as u64);
            let cap_clamped = cap.max(buffer.len() as u64);
            if buffer.len() < cap_clamped as usize {
                buffer.resize(cap_clamped as usize, JsonValue::Null);
            }
            let raw_len = len.unwrap_or(buffer.len() as u64);
            let safe_len = raw_len.min(buffer.len() as u64);
            let safe_head = head
                .unwrap_or_else(|| safe_len.saturating_sub(1))
                .min(buffer.len().saturating_sub(1) as u64);

            Some(json!({
                "capacity": cap_clamped,
                "head": if buffer.is_empty() { 0 } else { safe_head },
                "len": safe_len,
                "buffer": buffer,
            }))
        }
        _ => None,
    }
}

fn resource_value_to_u64(value: &ResourceValue) -> Option<u64> {
    match value {
        ResourceValue::Fixed64(v) => u64::try_from(v.raw_i64()).ok(),
        ResourceValue::Unit(v) => u64::try_from(v.value.raw_i64()).ok(),
        ResourceValue::Bool(v) => Some(if *v { 1 } else { 0 }),
        ResourceValue::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::{
        replay_sealed_external_execution, CanonicalExternalInput, CapabilityBinding, CapabilityRef,
        CapabilityScope, CapabilityValue, DependencyNodeKind, DeterministicSealedProvider,
        ExternalBoundaryRequest, ExternalDependencyNode,
    };
    use ddonirang_core::platform::ResourceMapEntry;
    use sha2::{Digest, Sha256};

    fn sealed_external_test_request() -> (ProcessLocalCapabilityStore, ExternalBoundaryRequest) {
        let binding = CapabilityBinding::new("owner-a", "world-a", 7, 3);
        let mut store = ProcessLocalCapabilityStore::new(7);
        assert!(store.insert(CapabilityValue::new(
            CapabilityRef::from_process_local(41),
            binding.clone(),
            [CapabilityScope::ExternalBoundaryExecute { boundary_id: "boundary:one".to_string() }],
        )));
        let request = ExternalBoundaryRequest {
            dependency: ExternalDependencyNode {
                kind: DependencyNodeKind::ExternalArtifact,
                artifact_identity: "artifact:sealed-01".to_string(),
                provenance_identity: "provenance:sealed-01".to_string(),
                provider_owner: "provider:sealed".to_string(),
                callable_owner: "callable:transform".to_string(),
                invocation_identity: "invocation:one".to_string(),
                execution_contract: "contract:sealed-v1".to_string(),
                external_boundary_identity: "boundary:one".to_string(),
            },
            input: CanonicalExternalInput { canonical_text: "입력:42".to_string() },
            capability: Some(CapabilityRef::from_process_local(41)),
            capability_binding: binding,
            max_input_bytes: 64,
            max_output_bytes: 64,
            max_calls: 1,
            call_ordinal: 1,
        };
        (store, request)
    }

    #[test]
    fn external_boundary_native_and_wasm_adapters_share_one_typed_owner() {
        let provider = DeterministicSealedProvider::new("provider:sealed", "callable:transform");
        let (native_store, request) = sealed_external_test_request();
        let (wasm_store, wasm_request) = sealed_external_test_request();
        let mut native_ledger = SealedExecutionLedger::new();
        let mut wasm_ledger = SealedExecutionLedger::new();

        let native = runtime_surface::execute_sealed_external_boundary(
            &mut native_ledger, &native_store, &provider, &request,
        ).expect("native adapter");
        let wasm = wasm_execute_sealed_external_boundary(
            &mut wasm_ledger, &wasm_store, &provider, &wasm_request,
        ).expect("wasm adapter");

        assert_eq!(native, wasm);
        assert_eq!(native.dispatch_count, 1);
        assert_eq!(native.publication_count, 1);
        assert_eq!(
            replay_sealed_external_execution(
                &request.dependency, &request.input, &native.output, &native.receipt,
            ),
            Ok(())
        );
    }

    #[test]
    fn declared_host_resource_native_and_wasm_share_one_receipt_bound_payload() {
        let source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 보여주기.\n}\n";
        let text = "고정 입력\n";
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let request = serde_json::json!({
            "schema": crate::host_resource::HOST_RESOURCE_REQUEST_SCHEMA,
            "mode": "record",
            "resources": [{
                "logical_resource_id": "caption",
                "sha256": format!("{:x}", hasher.finalize()),
                "byte_length": text.len(),
                "media_type": crate::host_resource::V1_TEXT_MEDIA_TYPE,
                "utf8_text": text,
            }]
        })
        .to_string();
        let parsed = crate::host_resource::parse_request_json(&request).expect("typed request");
        let prepared = crate::host_resource::prepare_request(&parsed).expect("record preflight");
        let native = runtime_surface::run_summary_from_supported_source_profile_with_host_resources(
            source,
            "host-resource.ddn",
            Some(1),
            Some("v1-core-v25"),
            &prepared,
        )
        .expect("native shared runtime");
        let wasm = wasm_run_summary_with_source_profile_host_resources_json(
            source,
            "host-resource.ddn",
            1,
            "v1-core-v25",
            &request,
        )
        .expect("WASM shared runtime");
        assert_eq!(wasm, native.to_string());
        assert_eq!(
            native["host_resource_receipt"]["receipt_identity"],
            prepared.receipt.receipt_identity
        );

        let replay = serde_json::json!({
            "schema": crate::host_resource::HOST_RESOURCE_REQUEST_SCHEMA,
            "mode": "replay",
            "transcript": prepared.transcript,
        })
        .to_string();
        assert_eq!(
            wasm_run_summary_with_source_profile_host_resources_json(
                source,
                "host-resource.ddn",
                1,
                "v1-core-v25",
                &replay,
            )
            .expect("WASM replay"),
            native.to_string()
        );
    }

    #[test]
    fn p1a_supported_profile_native_wasm_payloads_are_exactly_equal() {
        let cases = [
            (
                "p1a-parity.ddn",
                "매마디:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n",
            ),
            (
                "vc12-collection-parity.ddn",
                "매마디:움직씨 := {\n    값들:차림 := (1, 2, 3) 차림.\n    값들 보여주기.\n}\n",
            ),
        ];
        for (path, source) in cases {
            let native = runtime_surface::run_summary_from_supported_source_profile(
                source,
                path,
                Some(1),
                Some("v1-core-v25"),
            )
            .expect("native supported profile summary")
            .to_string();
            let wasm = wasm_run_summary_with_source_profile_json(
                source,
                path,
                1,
                "v1-core-v25",
            )
            .expect("WASM supported profile summary");
            assert_eq!(wasm, native, "path={path}");
        }

        let source = cases[0].1;

        let native_canon = runtime_surface::canonicalize_shared_source_with_supported_profile(
            source,
            Some("v1-core-v25"),
        )
        .expect("native supported profile canon")
        .ddn;
        let wasm_canon = wasm_canon_ddn_with_source_profile(source, "v1-core-v25")
            .expect("WASM supported profile canon");
        assert_eq!(wasm_canon, native_canon);

        let request = serde_json::json!({
            "source_text": source,
            "path": "p1a-parity.ddn",
            "analysis_generation": 201,
            "source_profile_identity": "v1-core-v25"
        })
        .to_string();
        assert_eq!(
            wasm_source_analysis_snapshot_json(&request).expect("WASM source analysis"),
            source_analysis::source_analysis_snapshot_json(&request)
                .expect("native source analysis")
        );
    }

    #[test]
    fn committed_madi_is_last_successfully_committed_frame_not_next_tick() {
        assert_eq!(last_committed_madi(&[]), JsonValue::Null);
        let frames = vec![
            json!({"committed_madi": 0, "state_hash": "a"}),
            json!({"committed_madi": 7, "state_hash": "b"}),
        ];
        assert_eq!(last_committed_madi(&frames), json!(7));
    }

    const SAMPLE_MAEGIM_SOURCE: &str = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
  }.
  theta0:수 <- (0.5) 매김 {
    범위: -1.2..1.2.
    분할수: 24.
  }.
}.
"#;

    const SAMPLE_GUSEONG_SOURCE: &str = r#"
짜임 {
  형식: 진자_틀.
  입력 {
    기준점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    끝점: (수,수) <- (0.0, 0.0).
  }.
}.

짜임 {
  형식: 물체_틀.
  입력 {
    붙는점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    꼭짓점: (수,수) <- (0.0, 0.0).
  }.
}.

a <- (L=1.0)인 진자_틀.
b <- (L=0.8)인 물체_틀.
a.기준점 <- b.꼭짓점.
"#;

    const SAMPLE_ALRIM_SOURCE: &str = r#"
"jump"라는 알림이 오면 {
  y <- 1.
}.

매틱:움직씨 = {
  "tick"라는 알림이 오면 {
    t <- t + 1.
  }.
}.

"reset"라는 알림이 오면 {
  y <- 0.
}.
"#;

    const SAMPLE_BLOCK_EDITOR_SOURCE: &str = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
  }.
  t:변수 <- 0.
  안내:글 = "초기".
}.

(시작)할때 {
  안내 보여주기.
}.

(매마디)마다 {
  t <- t + 1.
  t < 3 일때 {
    t 보여주기.
  } 아니면 {
    g 보여주기.
  }.
}.

(x) x목록에 대해 {
  고르기:
    { x < 1 }인것: {
      "low" 보여주기.
    }
    아니면: {
      "high" 보여주기.
    }.
}.

고르기:
  t < 10 인 경우 {
    "small" 보여주기.
  }
  모든 경우 다룸.

되풀이 {
  "tick" 보여주기.
}.

너머 {
  "side" 보여주기.
}.

{ t < 2 }인것 동안 {
  t <- t + 1.
}.

{ t == 2 }인것 바탕으로 아니면 {
  "bad" 보여주기.
} 맞으면 {
  "ok" 보여주기.
}.

{ t >= 0 }인것 다짐하고(알림) 아니면 {
  "warn" 보여주기.
}.

?? {
  "prompt" 보여주기.
}

"answer" ??: {
  "after" 보여주기.
}

{ t >= 0 }인것 ??
?? {
  "cond" 보여주기.
}

??:
  { t < 1 }인것: {
    "choose-low" 보여주기.
  }
??: {
    "choose-else" 보여주기.
  }

(기준:수 = 1) 판정:움직씨 = {
  기준 되돌림.
}.

기상청:임자 = {
  기상특보를 받으면 {
    알림.이름 보여주기.
  }.

  (정보 정보.온도 > 40)인 기상특보를 받으면 {
    (기상청)의 (온도:정보.온도) 경보 ~~> 제.
  }.

  (알림 알림.이름 == "경보")인 알림을 받으면 {
    알림.정보.온도 보여주기.
  }.
}.

"tick"라는 알림이 오면 {
  "evt" 보여주기.
}

보개로 그려.

(매마디)마다 {
  t 톺아보기.
  되풀이 {
    t <- t + 1.
    { t > 0 }인것 일때 {
      멈추기.
    }.
  }.
}

실행정책 {
  실행모드: 일반.
  효과정책: 허용.
}.

짜임 {
  형식: 점_틀.
  입력 {
    시작점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    끝점: (수,수) <- (0.0, 0.0).
  }.
}.

테스트:움직씨 = {
  보개마당 {
    #자막("정본 테스트").
  }.
}.

() 증명.
"#;

    const WAVE1_LESSONS: &[(&str, &str)] = &[
        (
            "synthetic/projectile-canon-surface.ddn",
            crate::self_contained_test_sources::PROJECTILE_CANON_SURFACE,
        ),
        (
            "synthetic/self-contained-multifile-entry.ddn",
            crate::self_contained_test_sources::MULTIFILE_ENTRY,
        ),
    ];

    fn wave1_lessons() -> Vec<(&'static str, String)> {
        WAVE1_LESSONS
            .iter()
            .map(|(path, source)| (*path, (*source).to_string()))
            .collect()
    }

    fn cli_canon_output(source: &str) -> crate::canon::CanonOutput {
        crate::canon::canonicalize(source, false).expect("cli canon")
    }

    fn cli_like_exec_policy_emit(source: &str) -> String {
        if !crate::canon::has_exec_policy_surface(source) {
            "{}\n".to_string()
        } else {
            cli_canon_output(source).exec_policy_map_json
        }
    }

    #[test]
    fn wasm_canon_flat_json_matches_cli_canon_surface() {
        let flat_json = wasm_canon_flat_json(SAMPLE_GUSEONG_SOURCE).expect("wasm flat json");
        assert!(flat_json.contains("\"schema\": \"ddn.guseong_flatten_plan.v1\""));
        assert!(flat_json.contains("\"name\": \"a\""));
        assert!(flat_json.contains("\"name\": \"b\""));
        assert!(flat_json.contains("\"dst_port\": \"기준점\""));
    }

    #[test]
    fn wasm_canon_flat_json_exact_parity_with_cli_output() {
        let wasm = wasm_canon_flat_json(SAMPLE_GUSEONG_SOURCE).expect("wasm flat json");
        let cli = cli_canon_output(SAMPLE_GUSEONG_SOURCE).guseong_flat_json;
        assert_eq!(wasm, cli);
    }

    #[test]
    fn wasm_model_frontdoor_matches_shared_rust_canon_and_semantic_hash() {
        let source = r#"
낙하운동 = 모형 {
  시간:수@s :: (축, 처음::고정=0@s).
  위치:수@m :: (이어감, 처음::고정=100@m, 처음받음, 내줌).
  속도:수@m/s :: (들임, 내줌).
  위치' =:= 속도.
}.
"#;
        let wasm_canon = wasm_canon_ddn(source).expect("wasm model canon");
        let shared_canon =
            runtime_surface::canonicalize_shared_ddn(source).expect("shared model canon");
        assert_eq!(wasm_canon, shared_canon);

        let wasm_ir: JsonValue =
            serde_json::from_str(&wasm_model_ir_candidate_json(source).expect("wasm model IR"))
                .expect("wasm model IR json");
        let shared_ir: JsonValue = serde_json::from_str(
            &runtime_surface::model_ir_candidate_json_from_source(source).expect("shared model IR"),
        )
        .expect("shared model IR json");
        assert_eq!(wasm_ir["semantic_hash"], shared_ir["semantic_hash"]);
        assert_eq!(wasm_ir, shared_ir);
    }

    #[test]
    fn wasm_canon_boim_surface_is_not_legacy() {
        let source = r#"
(매마디)마다 {
  n <- 1.
  (n: n) 보임.
}.
"#;
        assert!(!crate::canon::has_legacy_boim_surface(source));
        assert!(!crate::canon::has_legacy_boim_surface(
            SAMPLE_GUSEONG_SOURCE
        ));
    }

    #[test]
    fn wasm_run_summary_emits_ordered_successor_public_observation_trace() {
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
        let text = wasm_run_summary_json(source, "pack/ddn_boim_open_value_pack_w1/main.ddn", 2)
            .expect("wasm run summary");
        let summary: JsonValue = serde_json::from_str(&text).expect("summary json");
        let trace = &summary["public_observation_trace"];
        assert_eq!(trace["schema"], "ddn.public_observation_trace.v1");
        let names = trace["columns"]
            .as_array()
            .expect("columns")
            .iter()
            .map(|column| column["identity"]["name"].as_str().unwrap_or_default())
            .collect::<Vec<_>>();
        assert_eq!(names, ["시간", "각도", "각속도"]);
        let samples = trace["samples"].as_array().expect("samples");
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0]["cells"][0]["value_text"], "1@s");
        assert_eq!(samples[0]["cells"][1]["value_text"], "17@rad");
        assert_eq!(samples[1]["cells"][2]["value_text"], "2");
        assert!(trace["columns"][0]["identity"]["source_ref"]
            .as_str()
            .unwrap_or_default()
            .starts_with("pack/ddn_boim_open_value_pack_w1/main.ddn#bytes="));
        let projection = &summary["public_observation_table_projection"];
        assert_eq!(
            projection["schema"],
            "ddn.public_observation_table_projection.v2"
        );
        assert_eq!(projection["presentation_state"], "ready");
        assert_eq!(
            projection["tables"][0]["columns"]
                .as_array()
                .expect("projection columns")
                .iter()
                .map(|column| column["identity"]["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            names
        );
        assert_eq!(projection["tables"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            projection["tables"][0]["rows"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            projection["tables"][0]["rows"][0]["cells"][0]["value_text"],
            "1@s"
        );
        assert_eq!(
            projection["tables"][0]["rows"][0]["cells"][1]["value_text"],
            "17@rad"
        );
        assert_eq!(
            projection["tables"][0]["rows"][1]["cells"][2]["value_text"],
            "2"
        );
        assert_eq!(
            projection["source_trace_hash"],
            summary["public_observation_trace_hash"]
        );
        assert_eq!(
            &summary["engine_state"]["public_observation_table_projection"],
            projection
        );
        assert_eq!(
            projection["tables"][0]["rows"][1]["state_hash"],
            summary["state_hash"]
        );
        assert!(summary.get("observation_manifest").is_none());
        assert!(summary["engine_state"]
            .get("observation_manifest")
            .is_none());
    }

    #[test]
    fn wasm_run_summary_keeps_multiple_and_repeated_authored_observations_as_separate_tables() {
        let source = r#"
채비 {
  시간 <- 0.
  각도 <- 10.
  횟수 <- 0.
}.

(매마디)마다 {
  되풀이 {
    (값: 시간, 증가: 1) 보임.
    횟수 <- 횟수 + 1.
    { 횟수 >= 2 }인것 일때 {
      멈추기.
    }.
  }.
  (값: 각도, 다른: 시간) 보임.
}.
"#;
        let text =
            wasm_run_summary_json(source, "app/main.ddn", 1).expect("multiple observation summary");
        let summary: JsonValue = serde_json::from_str(&text).expect("summary json");
        let trace = &summary["public_observation_trace"];
        assert_eq!(trace["samples"].as_array().map(Vec::len), Some(3));
        assert!(trace["samples"]
            .as_array()
            .expect("samples")
            .iter()
            .all(|sample| sample["committed_madi"] == 0));

        let projection = &summary["public_observation_table_projection"];
        assert_eq!(
            projection["schema"],
            "ddn.public_observation_table_projection.v2"
        );
        assert_eq!(projection["tables"].as_array().map(Vec::len), Some(2));
        assert_eq!(
            projection["tables"][0]["columns"][0]["identity"]["name"],
            "값"
        );
        assert_eq!(
            projection["tables"][0]["columns"][1]["identity"]["name"],
            "증가"
        );
        assert_eq!(
            projection["tables"][0]["rows"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            projection["tables"][1]["columns"][0]["identity"]["name"],
            "값"
        );
        assert_eq!(
            projection["tables"][1]["columns"][1]["identity"]["name"],
            "다른"
        );
        assert_eq!(
            projection["tables"][1]["rows"].as_array().map(Vec::len),
            Some(1)
        );
        assert_ne!(
            projection["tables"][0]["columns"][0]["identity"]["source_ref"],
            projection["tables"][1]["columns"][0]["identity"]["source_ref"]
        );
        assert_eq!(projection["tables"][0]["rows"][0]["display_event_index"], 0);
        assert_eq!(projection["tables"][0]["rows"][1]["display_event_index"], 1);
        assert_eq!(projection["tables"][1]["rows"][0]["display_event_index"], 2);
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn wasm_vm_constructor_build_info_contains_pkg() {
        let vm = DdnWasmVm::new(SAMPLE_GUSEONG_SOURCE).expect("vm constructor");
        assert!(vm.get_build_info().contains("pkg="));
    }

    #[test]
    fn wasm_canon_maegim_plan_matches_cli_canon_surface() {
        let maegim_json = wasm_canon_maegim_plan(SAMPLE_MAEGIM_SOURCE).expect("wasm maegim plan");
        assert!(maegim_json.contains("\"schema\": \"ddn.maegim_control_plan.v1\""));
        assert!(maegim_json.contains("\"name\": \"g\""));
        assert!(maegim_json.contains("\"name\": \"theta0\""));
        assert!(maegim_json.contains("\"split_count_expr_canon\": \"24\""));
    }

    #[test]
    fn wasm_canon_maegim_plan_exact_parity_with_cli_output() {
        let wasm = wasm_canon_maegim_plan(SAMPLE_MAEGIM_SOURCE).expect("wasm maegim plan");
        let cli = cli_canon_output(SAMPLE_MAEGIM_SOURCE).maegim_control_json;
        assert_eq!(wasm, cli);
    }

    #[test]
    fn wasm_canon_alrim_plan_matches_cli_canon_surface() {
        let alrim_json = wasm_canon_alrim_plan(SAMPLE_ALRIM_SOURCE).expect("wasm alrim plan");
        assert!(alrim_json.contains("\"schema\": \"ddn.alrim_event_plan.v1\""));
        assert!(alrim_json.contains("\"kind\": \"jump\""));
        assert!(alrim_json.contains("\"kind\": \"tick\""));
        assert!(alrim_json.contains("\"scope\": \"root/seed:매틱\""));
    }

    #[test]
    fn wasm_canon_alrim_plan_exact_parity_with_cli_output() {
        let wasm = wasm_canon_alrim_plan(SAMPLE_ALRIM_SOURCE).expect("wasm alrim plan");
        let cli = cli_canon_output(SAMPLE_ALRIM_SOURCE).alrim_plan_json;
        assert_eq!(wasm, cli);
    }

    #[test]
    fn wasm_canon_exec_policy_map_uses_empty_fast_path_without_surface() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let exec_json = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        assert_eq!(exec_json.trim(), "{}");
    }

    #[test]
    fn wasm_canon_exec_policy_map_exact_parity_with_cli_emit_semantics_without_surface() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let wasm = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        let cli_like = cli_like_exec_policy_emit(source);
        assert_eq!(wasm, cli_like);
    }

    #[test]
    fn wasm_canon_exec_policy_map_emits_schema_with_surface() {
        let source = r#"
너머 {
  실행모드: 일반.
  효과정책: 허용.
}.
"#;
        let exec_json = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        assert!(exec_json.contains("\"schema\": \"ddn.exec_policy_effect_map.v1\""));
        assert_ne!(exec_json.trim(), "{}");
    }

    #[test]
    fn wasm_canon_exec_policy_map_exact_parity_with_cli_emit_semantics_with_surface() {
        let source = r#"
너머 {
  실행모드: 일반.
  효과정책: 허용.
}.
"#;
        let wasm = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        let cli_like = cli_like_exec_policy_emit(source);
        assert_eq!(wasm, cli_like);
    }

    #[test]
    fn wasm_canon_wave1_flat_json_exact_parity_with_cli_output() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_flat_json(&source).expect("wasm flat json");
            let cli = cli_canon_output(&source).guseong_flat_json;
            assert_eq!(wasm, cli, "wave1 flat parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_wave1_maegim_plan_exact_parity_with_cli_output() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_maegim_plan(&source).expect("wasm maegim plan");
            let cli = cli_canon_output(&source).maegim_control_json;
            assert_eq!(wasm, cli, "wave1 maegim parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_wave1_alrim_plan_exact_parity_with_cli_output() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_alrim_plan(&source).expect("wasm alrim plan");
            let cli = cli_canon_output(&source).alrim_plan_json;
            assert_eq!(wasm, cli, "wave1 alrim parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_wave1_exec_policy_map_exact_parity_with_cli_emit_semantics() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_exec_policy_map(&source).expect("wasm exec policy map");
            let cli_like = cli_like_exec_policy_emit(&source);
            assert_eq!(wasm, cli_like, "wave1 exec parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_block_editor_plan_matches_cli_canon_surface() {
        let block_json =
            wasm_canon_block_editor_plan(SAMPLE_BLOCK_EDITOR_SOURCE).expect("wasm block plan");
        assert!(block_json.contains("\"schema\": \"ddn.block_editor_plan.v1\""));
        assert!(block_json.contains("\"kind\": \"charim_block\""));
        assert!(block_json.contains("\"kind\": \"hook_start\""));
        assert!(block_json.contains("\"kind\": \"hook_tick\""));
        assert!(block_json.contains("\"kind\": \"if_else\""));
        assert!(block_json.contains("\"kind\": \"for_each\""));
        assert!(block_json.contains("\"kind\": \"choose_else\""));
        assert!(block_json.contains("\"kind\": \"choose_exhaustive\""));
        assert!(block_json.contains("\"exhaustive\": \"true\""));
        assert!(block_json.contains("\"kind\": \"repeat\""));
        assert!(block_json.contains("\"kind\": \"open_block\""));
        assert!(block_json.contains("\"kind\": \"while_block\""));
        assert!(block_json.contains("\"kind\": \"contract_guard\""));
        assert!(block_json.contains("\"kind\": \"prompt_block\""));
        assert!(block_json.contains("\"kind\": \"prompt_after\""));
        assert!(block_json.contains("\"kind\": \"prompt_condition\""));
        assert!(block_json.contains("\"kind\": \"prompt_choose\""));
        assert!(block_json.contains("\"kind\": \"seed_def\""));
        assert!(block_json.contains("\"kind\": \"receive_block\""));
        assert!(block_json.contains("\"kind\": \"event_react\""));
        assert!(block_json.contains("\"kind\": \"send_signal\""));
        assert!(block_json.contains("\"kind\": \"return_value\""));
        assert!(block_json.contains("\"kind\": \"inspect_value\""));
        assert!(block_json.contains("\"kind\": \"bogae_draw\""));
        assert!(block_json.contains("\"kind\": \"break_loop\""));
        assert!(block_json.contains("\"kind\": \"bogae_madang_block\""));
        assert!(block_json.contains("\"kind\": \"exec_policy_block\""));
        assert!(block_json.contains("\"kind\": \"jjaim_block\""));
        assert!(block_json.contains("\"kind\": \"expr_stmt\""));
        assert!(block_json.contains("\"exprs\": {"));
        assert!(block_json.contains("\"kind\": \"call\""));
        assert!(block_json.contains("\"kind\": \"binding\""));
    }

    #[test]
    fn wasm_canon_block_editor_plan_preserves_authoring_hooks() {
        let source = r#"
채비 {
  t:변수 <- 0.
}.

(시작)할때 {
  t 보여주기.
}.

(매마디)마다 {
  t <- t + 1.
}.
"#;
        let block_json = wasm_canon_block_editor_plan(source).expect("wasm block plan");
        assert!(block_json.contains("\"kind\": \"charim_block\""));
        assert!(block_json.contains("\"kind\": \"hook_start\""));
        assert!(block_json.contains("\"kind\": \"hook_tick\""));
        assert!(!block_json.contains("\"kind\": \"seed_def\""));
    }

    #[test]
    fn parse_stream_sidecar_tag_supports_ascii_and_ko_suffix() {
        assert_eq!(
            parse_stream_sidecar_tag("energy_stream_head"),
            Some(("energy_stream".to_string(), "head"))
        );
        assert_eq!(
            parse_stream_sidecar_tag("energy_stream_capacity"),
            Some(("energy_stream".to_string(), "capacity"))
        );
        assert_eq!(
            parse_stream_sidecar_tag("가격흐름_길이"),
            Some(("가격흐름".to_string(), "len"))
        );
        assert_eq!(parse_stream_sidecar_tag("plain_value"), None);
    }

    #[test]
    fn extract_ring_buffer_list_merges_sidecar_meta() {
        let value = ResourceValue::List(vec![
            ResourceValue::Fixed64(Fixed64::from_i64(10)),
            ResourceValue::Fixed64(Fixed64::from_i64(20)),
        ]);
        let meta = StreamMeta {
            capacity: Some(4),
            head: Some(1),
            len: Some(2),
        };
        let stream = extract_ring_buffer(&value, Some(&meta)).expect("stream json");
        let obj = stream.as_object().expect("object");
        assert_eq!(obj.get("capacity").and_then(JsonValue::as_u64), Some(4));
        assert_eq!(obj.get("head").and_then(JsonValue::as_u64), Some(1));
        assert_eq!(obj.get("len").and_then(JsonValue::as_u64), Some(2));
        let buffer = obj
            .get("buffer")
            .and_then(JsonValue::as_array)
            .expect("buffer array");
        assert_eq!(buffer.len(), 4);
        assert!(buffer[2].is_null());
        assert!(buffer[3].is_null());
    }

    #[test]
    fn state_hash_is_stable_when_only_view_resource_changes() {
        let mut world = DetNuri::new();
        world
            .world_mut()
            .set_resource_fixed64("경제.자본".to_string(), Fixed64::from_i64(1));
        world
            .world_mut()
            .set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(640));

        let view_prefixes = DEFAULT_VIEW_PREFIXES
            .iter()
            .map(|prefix| (*prefix).to_string())
            .collect::<Vec<_>>();
        let state_hash_a = current_state_hash(&world, &view_prefixes);
        let view_hash_a = hash_json_string(&build_view_meta(world.world()));

        world
            .world_mut()
            .set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(800));
        let state_hash_b = current_state_hash(&world, &view_prefixes);
        let view_hash_b = hash_json_string(&build_view_meta(world.world()));

        assert_eq!(state_hash_a, state_hash_b);
        assert_ne!(view_hash_a, view_hash_b);
    }

    fn point_resource(x: i64, y: i64) -> ResourceValue {
        ResourceValue::List(vec![
            ResourceValue::Fixed64(Fixed64::from_i64(x)),
            ResourceValue::Fixed64(Fixed64::from_i64(y)),
        ])
    }

    fn draw_item_resource(kind: &str) -> ResourceValue {
        ResourceValue::Map(BTreeMap::from([
            (
                "kind".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("kind".to_string()),
                    value: ResourceValue::String(kind.to_string()),
                },
            ),
            (
                "x".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("x".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
                },
            ),
            (
                "y".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("y".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
                },
            ),
            (
                "r".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("r".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(1)),
                },
            ),
        ]))
    }

    #[test]
    fn build_view_meta_emits_primary_and_secondary_stack() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            BOGAE_DRAWLIST_TAG.to_string(),
            ResourceValue::List(vec![draw_item_resource("circle")]),
        );
        world.world_mut().set_resource_value(
            "그래프_점목록_f".to_string(),
            ResourceValue::List(vec![point_resource(0, 1), point_resource(1, 2)]),
        );

        let view_meta = build_view_meta(world.world());
        let obj = view_meta.as_object().expect("view_meta object");

        assert_eq!(
            obj.get("primary")
                .and_then(JsonValue::as_object)
                .and_then(|row| row.get("family"))
                .and_then(JsonValue::as_str),
            Some("space2d")
        );
        let secondary = obj
            .get("secondary")
            .and_then(JsonValue::as_array)
            .expect("secondary array");
        assert_eq!(secondary.len(), 1);
        assert_eq!(
            secondary[0]
                .as_object()
                .and_then(|row| row.get("family"))
                .and_then(JsonValue::as_str),
            Some("graph")
        );
        assert_eq!(
            obj.get("overlays")
                .and_then(JsonValue::as_array)
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn build_view_meta_graph_only_sets_graph_primary() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "그래프_점목록_f".to_string(),
            ResourceValue::List(vec![point_resource(0, 1), point_resource(1, 2)]),
        );

        let view_meta = build_view_meta(world.world());
        let obj = view_meta.as_object().expect("view_meta object");

        assert_eq!(
            obj.get("primary")
                .and_then(JsonValue::as_object)
                .and_then(|row| row.get("family"))
                .and_then(JsonValue::as_str),
            Some("graph")
        );
        assert_eq!(
            obj.get("secondary")
                .and_then(JsonValue::as_array)
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn build_view_meta_emits_exact_workbench_authoritative_results() {
        let mut world = DetNuri::new();
        for (tag, value) in [
            ("t", 3),
            ("x", 4),
            ("y", 5),
            ("vx", 6),
            ("vy", 7),
            ("a", 2),
            ("b", 1),
            ("theta", 1),
            ("omega", -1),
            ("energy", 9),
            ("kinetic_energy", 4),
            ("potential_energy", 5),
            ("L", 2),
            ("bob_x", 1),
            ("bob_y", -1),
        ] {
            world
                .world_mut()
                .set_resource_fixed64(tag.to_string(), Fixed64::from_i64(value));
        }
        world.world_mut().set_resource_value(
            BOGAE_DRAWLIST_TAG.to_string(),
            ResourceValue::List(vec![draw_item_resource("circle")]),
        );
        world.world_mut().set_resource_value(
            "위치".to_string(),
            ResourceValue::Map(BTreeMap::from([
                (
                    "x".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("x".to_string()),
                        value: ResourceValue::Fixed64(Fixed64::from_i64(3)),
                    },
                ),
                (
                    "y".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("y".to_string()),
                        value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
                    },
                ),
            ])),
        );

        let meta = build_view_meta(world.world());
        let workbench = meta
            .get("workbench")
            .and_then(JsonValue::as_object)
            .expect("workbench result set");
        assert_eq!(
            workbench.get("schema").and_then(JsonValue::as_str),
            Some(WORKBENCH_RESULT_SET_SCHEMA)
        );
        let results = workbench
            .get("results")
            .and_then(JsonValue::as_object)
            .expect("workbench results");
        assert_eq!(
            results.keys().cloned().collect::<Vec<_>>(),
            vec!["grid", "line", "pendulum", "projectile"]
        );
        assert_eq!(
            results["line"].get("schema").and_then(JsonValue::as_str),
            Some(WORKBENCH_LINE_RESULT_SCHEMA)
        );
        assert_eq!(
            results["projectile"]
                .pointer("/content/current/vy")
                .and_then(JsonValue::as_f64),
            Some(7.0)
        );
        assert_eq!(
            results["pendulum"]
                .pointer("/content/current/physical_bob/y")
                .and_then(JsonValue::as_f64),
            Some(-1.0)
        );
        assert_eq!(
            results["grid"]
                .pointer("/content/position/x")
                .and_then(JsonValue::as_f64),
            Some(3.0)
        );
    }

    #[test]
    fn workbench_authoritative_results_are_exact_and_partial_inputs_fail_closed() {
        let mut world = DetNuri::new();
        world
            .world_mut()
            .set_resource_fixed64("x".to_string(), Fixed64::from_i64(1));
        world
            .world_mut()
            .set_resource_fixed64("y".to_string(), Fixed64::from_i64(2));
        world
            .world_mut()
            .set_resource_fixed64("가로속도".to_string(), Fixed64::from_i64(3));
        world
            .world_mut()
            .set_resource_fixed64("세로속도".to_string(), Fixed64::from_i64(4));

        let meta = build_view_meta(world.world());
        assert!(meta.get("workbench").is_none());
    }

    #[test]
    fn serialize_world_resources_emits_value_json_alongside_value_keys() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "묶음값".to_string(),
            ResourceValue::Map(BTreeMap::from([
                (
                    "x".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("x".to_string()),
                        value: ResourceValue::Fixed64(Fixed64::from_i64(1)),
                    },
                ),
                (
                    "ok".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("ok".to_string()),
                        value: ResourceValue::Bool(true),
                    },
                ),
            ])),
        );

        let resources = serialize_world_resources(world.world());
        let root = resources.as_object().expect("resources object");
        let value = root
            .get("value")
            .and_then(JsonValue::as_object)
            .expect("value map");
        let value_json = root
            .get("value_json")
            .and_then(JsonValue::as_object)
            .expect("value_json map");

        assert!(value.get("묶음값").and_then(JsonValue::as_str).is_some());
        let row = value_json
            .get("묶음값")
            .and_then(JsonValue::as_object)
            .expect("value_json row");
        assert_eq!(row.get("x").and_then(JsonValue::as_f64), Some(1.0));
        assert_eq!(row.get("ok").and_then(JsonValue::as_bool), Some(true));
    }

    #[test]
    fn serialize_patch_set_resource_value_emits_value_json_payload() {
        let mut patch = Patch::default();
        patch.ops.push(PatchOp::SetResourceValue {
            tag: "리스트".to_string(),
            value: ResourceValue::List(vec![
                ResourceValue::Fixed64(Fixed64::from_i64(1)),
                ResourceValue::String("둘".to_string()),
            ]),
        });

        let json = serialize_patch(&patch);
        let rows = json.as_array().expect("patch array");
        assert_eq!(rows.len(), 1);
        let row = rows[0].as_object().expect("patch row");
        assert_eq!(
            row.get("op").and_then(JsonValue::as_str),
            Some("set_resource_value")
        );
        assert_eq!(row.get("tag").and_then(JsonValue::as_str), Some("리스트"));
        let value_json = row
            .get("value_json")
            .and_then(JsonValue::as_array)
            .expect("value_json array");
        assert_eq!(value_json.len(), 2);
        assert_eq!(value_json[0].as_f64(), Some(1.0));
        assert_eq!(value_json[1].as_str(), Some("둘"));
    }

    #[test]
    fn restore_hash_diags_detect_mismatch_codes() {
        let payload = json!({
            "state_hash": "blake3:expected-state",
            "view_hash": "blake3:expected-view",
        });
        let diags = restore_hash_diags(&payload, "blake3:actual-state", "blake3:actual-view");
        assert_eq!(diags.len(), 2);
        let codes = diags
            .iter()
            .filter_map(|diag| diag.get("code").and_then(JsonValue::as_str))
            .collect::<Vec<_>>();
        assert!(codes.contains(&"STATE_HASH_MISMATCH"));
        assert!(codes.contains(&"VIEW_HASH_MISMATCH"));
    }

    #[test]
    fn resource_value_to_draw_item_maps_layer_index_aliases() {
        let draw_item = ResourceValue::Map(BTreeMap::from([
            (
                "kind".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("kind".to_string()),
                    value: ResourceValue::String("polygon".to_string()),
                },
            ),
            (
                "레이어".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("레이어".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(3)),
                },
            ),
            (
                "fill".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("fill".to_string()),
                    value: ResourceValue::String("#f59e0b".to_string()),
                },
            ),
        ]));
        let json = resource_value_to_draw_item(&draw_item).expect("draw item json");
        let obj = json.as_object().expect("draw item object");
        assert_eq!(obj.get("kind").and_then(JsonValue::as_str), Some("polygon"));
        assert_eq!(
            obj.get("layer_index").and_then(JsonValue::as_f64),
            Some(3.0)
        );
        assert_eq!(obj.get("fill").and_then(JsonValue::as_str), Some("#f59e0b"));
    }

    #[test]
    fn resource_value_to_draw_item_maps_group_id_aliases() {
        let draw_item = ResourceValue::Map(BTreeMap::from([
            (
                "kind".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("kind".to_string()),
                    value: ResourceValue::String("polyline".to_string()),
                },
            ),
            (
                "그룹".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("그룹".to_string()),
                    value: ResourceValue::String("pendulum.path".to_string()),
                },
            ),
            (
                "stroke".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("stroke".to_string()),
                    value: ResourceValue::String("#22c55e".to_string()),
                },
            ),
        ]));
        let json = resource_value_to_draw_item(&draw_item).expect("draw item json");
        let obj = json.as_object().expect("draw item object");
        assert_eq!(
            obj.get("kind").and_then(JsonValue::as_str),
            Some("polyline")
        );
        assert_eq!(
            obj.get("group_id").and_then(JsonValue::as_str),
            Some("pendulum.path")
        );
        assert_eq!(
            obj.get("stroke").and_then(JsonValue::as_str),
            Some("#22c55e")
        );
    }

    #[test]
    fn unit_value_wire_v2_is_explicit_and_roundtrips_ninth_axis() {
        let value = ResourceValue::Unit(ddonirang_core::UnitValue::new(
            Fixed64::from_i64(2),
            ddonirang_core::Unit::Mole,
        ));
        let encoded = resource_value_to_detjson(&value);
        assert_eq!(encoded["schema"], UNIT_VALUE_WIRE_SCHEMA_V2);
        assert_eq!(
            encoded["dimension"]["schema"],
            ddonirang_core::UnitDim::WIRE_SCHEMA_V2
        );
        assert_eq!(
            encoded["dimension"]["axis_order"],
            json!(ddonirang_core::UnitDim::AXIS_ORDER_V2)
        );
        assert_eq!(
            encoded["dimension"]["exponents"],
            json!([0, 0, 0, 0, 0, 0, 0, 0, 1])
        );
        assert_eq!(
            resource_value_from_detjson(&encoded).expect("v2 roundtrip"),
            value
        );
    }

    #[test]
    fn unit_value_wire_v2_rejects_v1_shape_unknown_version_and_axis_mismatch() {
        let legacy = json!({
            "type": "unit",
            "raw_i64": "2000000",
            "dimension": [0, 0, 0, 0, 0, 0, 0, 0],
        });
        assert!(resource_value_from_detjson(&legacy)
            .expect_err("unversioned v1 must fail")
            .contains("E_UNIT_WIRE_VERSION_MISMATCH"));

        let mut unknown_version = resource_value_to_detjson(&ResourceValue::Unit(
            ddonirang_core::UnitValue::new(Fixed64::from_i64(2), ddonirang_core::Unit::Mole),
        ));
        unknown_version["schema"] = json!("ddn.unit_value_wire.v3");
        assert!(resource_value_from_detjson(&unknown_version)
            .expect_err("unknown version must fail")
            .contains("E_UNIT_WIRE_VERSION_MISMATCH"));

        let mut reordered = resource_value_to_detjson(&ResourceValue::Unit(
            ddonirang_core::UnitValue::new(Fixed64::from_i64(2), ddonirang_core::Unit::Mole),
        ));
        reordered["dimension"]["axis_order"][0] = json!("time");
        reordered["dimension"]["axis_order"][1] = json!("length");
        assert!(resource_value_from_detjson(&reordered)
            .expect_err("axis reorder must fail")
            .contains("E_UNIT_DIMENSION_AXIS_ORDER_MISMATCH"));

        let mut missing_axis = resource_value_to_detjson(&ResourceValue::Unit(
            ddonirang_core::UnitValue::new(Fixed64::from_i64(2), ddonirang_core::Unit::Mole),
        ));
        missing_axis["dimension"]["exponents"]
            .as_array_mut()
            .expect("exponents")
            .pop();
        assert!(resource_value_from_detjson(&missing_axis)
            .expect_err("missing ninth axis must fail")
            .contains("E_UNIT_DIMENSION_AXIS_COUNT_MISMATCH"));
    }

    #[test]
    fn resource_snapshot_v2_rejects_checkpoint_unit_wire_mismatch() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "물질량".to_string(),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2),
                ddonirang_core::Unit::Mole,
            )),
        );
        let encoded = serialize_world_resources_snapshot(world.world());
        assert_eq!(encoded["schema"], RESOURCE_SNAPSHOT_SCHEMA_V2);
        assert_eq!(encoded["unit_wire_schema"], UNIT_VALUE_WIRE_SCHEMA_V2);

        let mut mismatch = encoded.clone();
        mismatch["unit_wire_schema"] = json!("ddn.unit_value_wire.v1");
        let mut restored = DetNuri::new();
        assert!(restore_world_resources(restored.world_mut(), &mismatch)
            .expect_err("checkpoint mismatch must fail")
            .contains("E_UNIT_WIRE_CHECKPOINT_MISMATCH"));
    }

    fn wasm_restore_payload(resources: JsonValue) -> JsonValue {
        json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "state": {
                "input": {
                    "keys_pressed": 1,
                    "last_key_name": "m",
                    "pointer_x_i32": 2,
                    "pointer_y_i32": 3,
                    "rng_base_seed": 4,
                    "dt": {"raw_i64": Fixed64::ONE_RAW.to_string()},
                },
                "snapshot_v2": build_wasm_state_snapshot_v2(
                    resources,
                    json!({"속도": 2}),
                    Vec::new(),
                    7,
                ),
            },
        })
    }

    #[test]
    fn wasm_state_snapshot_v2_roundtrips_ninth_axis_unit_exactly() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "물질량".to_string(),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2),
                ddonirang_core::Unit::Mole,
            )),
        );
        let resources = serialize_world_resources_snapshot(world.world());
        let payload = wasm_restore_payload(resources.clone());
        let staged = stage_wasm_state_restore(&payload, "app/main.ddn", "source", "canon")
            .expect("v2 state snapshot");

        assert_eq!(
            serialize_world_resources_snapshot(staged.world.world()),
            resources
        );
        assert_eq!(staged.tick_id, 7);
        assert_eq!(
            serialize_world_resources_snapshot(staged.world.world())["value_det"]["물질량"]
                ["dimension"]["exponents"],
            json!([0, 0, 0, 0, 0, 0, 0, 0, 1])
        );
        assert_eq!(
            serialize_world_resources_snapshot(staged.world.world())["value_det"]["물질량"]
                ["raw_i64"],
            json!(Fixed64::from_i64(2).raw_i64().to_string())
        );
    }

    #[test]
    fn wasm_state_snapshot_v2_rejects_old_missing_and_mixed_versions() {
        let resources = serialize_world_resources_snapshot(DetNuri::new().world());
        let mut old = wasm_restore_payload(resources.clone());
        old["state"]["snapshot_v1"] = old["state"]["snapshot_v2"].take();
        assert!(
            stage_wasm_state_restore(&old, "app/main.ddn", "source", "canon")
                .err()
                .expect("v1 must fail")
                .contains("E_WASM_STATE_SNAPSHOT_MIXED_VERSION")
        );

        let mut missing = wasm_restore_payload(resources.clone());
        missing["state"]
            .as_object_mut()
            .expect("state")
            .remove("snapshot_v2");
        assert!(
            stage_wasm_state_restore(&missing, "app/main.ddn", "source", "canon")
                .err()
                .expect("missing snapshot must fail")
                .contains("E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH")
        );

        let mut unknown = wasm_restore_payload(resources);
        unknown["state"]["snapshot_v2"]["schema"] = json!("ddn.wasm_state_snapshot.v3");
        assert!(
            stage_wasm_state_restore(&unknown, "app/main.ddn", "source", "canon")
                .err()
                .expect("unknown snapshot must fail")
                .contains("E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH")
        );
    }

    #[test]
    fn failed_staged_restore_leaves_existing_world_unchanged() {
        let mut current = DetNuri::new();
        current.world_mut().set_resource_value(
            "기존".to_string(),
            ResourceValue::Fixed64(Fixed64::from_i64(9)),
        );
        let before = serialize_world_resources_snapshot(current.world());
        let before_hash = current
            .world()
            .state_hash_excluding_resource_prefixes(&[])
            .to_hex()
            .to_string();

        let mut corrupt = serialize_world_resources_snapshot(DetNuri::new().world());
        corrupt["fixed64"]["먼저_유효"] = json!({"raw_i64": "1000000"});
        corrupt["fixed64"]["뒤에_손상"] = json!({"raw_i64": "not-an-i64"});
        let payload = wasm_restore_payload(corrupt);
        assert!(
            stage_wasm_state_restore(&payload, "app/main.ddn", "source", "canon")
                .err()
                .expect("corrupt later entry must fail")
                .contains("fixed64[뒤에_손상]")
        );

        assert_eq!(serialize_world_resources_snapshot(current.world()), before);
        assert_eq!(
            current
                .world()
                .state_hash_excluding_resource_prefixes(&[])
                .to_hex()
                .to_string(),
            before_hash
        );
    }
}

fn serialize_param_overrides(params: &BTreeMap<String, Value>) -> JsonValue {
    let mut out = Map::new();
    for (key, value) in params {
        let raw = match value {
            Value::Fixed64(v) => json!(fixed64_to_f64(*v)),
            Value::Bool(v) => JsonValue::Bool(*v),
            Value::String(v) => JsonValue::String(v.clone()),
            _ => continue,
        };
        out.insert(key.clone(), raw);
    }
    JsonValue::Object(out)
}

fn parse_param_overrides(payload: &JsonValue) -> Result<BTreeMap<String, Value>, String> {
    let mut out = BTreeMap::new();
    let JsonValue::Object(obj) = payload else {
        return Err("E_WASM_STATE_PARAMS_INVALID: param_overrides는 object여야 합니다".to_string());
    };
    for (key, value) in obj {
        let parsed = match value {
            JsonValue::Bool(v) => Value::Bool(*v),
            JsonValue::String(v) => Value::String(v.clone()),
            JsonValue::Number(_) => {
                let n = value
                    .as_f64()
                    .ok_or_else(|| format!("param_overrides[{key}] 숫자 파싱 실패"))?;
                if !n.is_finite() {
                    return Err(format!("param_overrides[{key}] 유한 숫자가 아닙니다"));
                }
                Value::Fixed64(fixed64_from_f64_checked(n))
            }
            _ => {
                return Err(format!(
                    "E_WASM_STATE_PARAMS_INVALID: param_overrides[{key}] 지원하지 않는 값"
                ))
            }
        };
        out.insert(key.clone(), parsed);
    }
    Ok(out)
}

fn serialize_world_resources_snapshot(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();

    out.insert(
        "schema".to_string(),
        JsonValue::String(RESOURCE_SNAPSHOT_SCHEMA_V2.to_string()),
    );
    out.insert(
        "unit_wire_schema".to_string(),
        JsonValue::String(UNIT_VALUE_WIRE_SCHEMA_V2.to_string()),
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

fn restore_world_resources(
    world: &mut ddonirang_core::platform::NuriWorld,
    resources: &JsonValue,
) -> Result<(), String> {
    let JsonValue::Object(root) = resources else {
        return Err("E_RESOURCE_SNAPSHOT_INVALID: resources는 object여야 합니다".to_string());
    };
    let resource_schema = root.get("schema").and_then(JsonValue::as_str);
    if resource_schema != Some(RESOURCE_SNAPSHOT_SCHEMA_V2) {
        return Err(format!(
            "E_RESOURCE_SNAPSHOT_VERSION_MISMATCH: expected {RESOURCE_SNAPSHOT_SCHEMA_V2}, got {}",
            resource_schema.unwrap_or("<missing>")
        ));
    }
    let unit_wire_schema = root.get("unit_wire_schema").and_then(JsonValue::as_str);
    if unit_wire_schema != Some(UNIT_VALUE_WIRE_SCHEMA_V2) {
        return Err(format!(
            "E_UNIT_WIRE_CHECKPOINT_MISMATCH: expected {UNIT_VALUE_WIRE_SCHEMA_V2}, got {}",
            unit_wire_schema.unwrap_or("<missing>")
        ));
    }

    if let Some(JsonValue::Array(components)) = root.get("components") {
        for component in components {
            let entity = component
                .get("entity")
                .and_then(JsonValue::as_u64)
                .ok_or_else(|| "component entity 파싱 실패".to_string())?;
            let tag = component
                .get("tag")
                .and_then(JsonValue::as_str)
                .ok_or_else(|| "component tag 파싱 실패".to_string())?;
            let value = component
                .get("json")
                .and_then(JsonValue::as_str)
                .ok_or_else(|| "component json 파싱 실패".to_string())?;
            world.set_component_json(
                ddonirang_core::platform::EntityId(entity),
                ddonirang_core::platform::ComponentTag(tag.to_string()),
                value.to_string(),
            );
        }
    }
    if let Some(next_entity) = root.get("next_entity").and_then(JsonValue::as_u64) {
        world.restore_next_entity_cursor(next_entity);
    }

    if let Some(JsonValue::Object(json_map)) = root.get("json") {
        for (tag, value) in json_map {
            if let Some(raw) = value.as_str() {
                world.set_resource_json(tag.clone(), raw.to_string());
            }
        }
    }

    if let Some(JsonValue::Object(fixed_map)) = root.get("fixed64") {
        for (tag, value) in fixed_map {
            let parsed =
                json_to_fixed64(value).ok_or_else(|| format!("fixed64[{tag}] 파싱 실패"))?;
            world.set_resource_fixed64(tag.clone(), parsed);
        }
    }

    if let Some(JsonValue::Object(handle_map)) = root.get("handle") {
        for (tag, value) in handle_map {
            let raw = match value {
                JsonValue::String(s) => parse_handle_string(s)?,
                JsonValue::Number(n) => n
                    .as_u64()
                    .ok_or_else(|| format!("handle[{tag}] 파싱 실패"))?,
                _ => return Err(format!("handle[{tag}] 형식 오류")),
            };
            world.set_resource_handle(tag.clone(), ResourceHandle::from_raw(raw));
        }
    }

    if let Some(JsonValue::Object(value_map)) = root.get("value_det") {
        for (tag, value) in value_map {
            let parsed = resource_value_from_detjson(value)?;
            world.set_resource_value(tag.clone(), parsed);
        }
    }

    Ok(())
}

fn json_to_fixed64(value: &JsonValue) -> Option<Fixed64> {
    match value {
        JsonValue::Number(n) => n.as_f64().map(fixed64_from_f64_checked),
        JsonValue::String(s) => Fixed64::parse_decimal(s),
        JsonValue::Object(obj) => obj
            .get("raw_i64")
            .and_then(JsonValue::as_str)
            .and_then(|raw| raw.parse::<i64>().ok())
            .map(Fixed64::from_raw_i64),
        _ => None,
    }
}

fn parse_handle_string(text: &str) -> Result<u64, String> {
    let trimmed = text.trim();
    let without_handle = trimmed.strip_prefix("handle:").unwrap_or(trimmed);
    let hex = without_handle
        .strip_prefix("자원#")
        .unwrap_or(without_handle);
    u64::from_str_radix(hex, 16).map_err(|_| format!("handle 파싱 실패: {text}"))
}

fn resource_value_to_detjson(value: &ResourceValue) -> JsonValue {
    crate::unit_wire::resource_value_to_detjson(value)
}

fn resource_value_from_detjson(payload: &JsonValue) -> Result<ResourceValue, String> {
    crate::unit_wire::resource_value_from_detjson(payload)
}

fn json_to_u64(value: &JsonValue) -> Option<u64> {
    match value {
        JsonValue::Number(n) => n.as_u64(),
        JsonValue::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

fn json_to_i32(value: &JsonValue) -> Option<i32> {
    match value {
        JsonValue::Number(n) => n.as_i64().and_then(|v| i32::try_from(v).ok()),
        JsonValue::String(s) => s.parse::<i32>().ok(),
        _ => None,
    }
}

fn serialize_world_resources(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    let mut value_json_map = Map::new();

    let mut json_map = Map::new();
    for (tag, json) in world.resource_json_entries() {
        value_json_map.insert(tag.clone(), resource_json_text_to_json(&json));
        json_map.insert(tag, JsonValue::String(json));
    }
    out.insert("json".to_string(), JsonValue::Object(json_map));

    let mut fixed_map = Map::new();
    for (tag, value) in world.resource_fixed64_entries() {
        value_json_map.insert(tag.clone(), json!(fixed64_to_f64(value)));
        fixed_map.insert(tag, JsonValue::String(value.to_string()));
    }
    out.insert("fixed64".to_string(), JsonValue::Object(fixed_map));

    let mut handle_map = Map::new();
    for (tag, handle) in world.resource_handle_entries() {
        handle_map.insert(tag, JsonValue::String(handle_to_string(handle)));
    }
    out.insert("handle".to_string(), JsonValue::Object(handle_map));

    let mut value_map = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_map.insert(tag.clone(), JsonValue::String(value.canon_key()));
        value_json_map.insert(tag, resource_value_to_json(&value));
    }
    out.insert("value".to_string(), JsonValue::Object(value_map));
    out.insert("value_json".to_string(), JsonValue::Object(value_json_map));

    JsonValue::Object(out)
}

fn resource_json_text_to_json(text: &str) -> JsonValue {
    serde_json::from_str::<JsonValue>(text).unwrap_or_else(|_| JsonValue::String(text.to_string()))
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

fn serialize_input_snapshot(input: &InputSnapshot, rng_base_seed: Option<u64>) -> JsonValue {
    let mut out = Map::new();
    out.insert("keys_pressed".to_string(), json!(input.keys_pressed));
    out.insert("last_key_name".to_string(), json!(input.last_key_name));
    out.insert("pointer_x_i32".to_string(), json!(input.pointer_x_i32));
    out.insert("pointer_y_i32".to_string(), json!(input.pointer_y_i32));
    out.insert("dt".to_string(), json!(input.dt.to_string()));
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

fn serialize_input_state(vm: &DdnWasmVm) -> JsonValue {
    json!({
        "keys_pressed": vm.input_keys_pressed,
        "last_key_name": vm.input_last_key_name,
        "pointer_x_i32": vm.input_pointer_x_i32,
        "pointer_y_i32": vm.input_pointer_y_i32,
        "dt": vm.input_dt.to_string(),
        "frame_source": InputSource::Person.label(),
        "frame_source_code": InputSource::Person.code_u8(),
        "rng_seed": vm.rng_seed,
        "rng_base_seed": vm.rng_seed,
        "pending_ai_injection_count": vm.pending_ai_injections.len(),
    })
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

fn collect_columns_and_row(
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

fn js_scalar_to_runtime_value(value: &JsValue) -> Option<Value> {
    if let Some(v) = value.as_f64() {
        if v.is_finite() {
            return Some(Value::Fixed64(fixed64_from_f64_checked(v)));
        }
        return None;
    }
    if let Some(v) = value.as_bool() {
        return Some(Value::Bool(v));
    }
    value.as_string().map(Value::String)
}

fn apply_param_value(
    world: &mut ddonirang_core::platform::NuriWorld,
    key: &str,
    value: &Value,
) -> Result<(), String> {
    match value {
        Value::Fixed64(v) => {
            world.set_resource_fixed64(key.to_string(), *v);
            Ok(())
        }
        Value::Bool(v) => {
            world.set_resource_json(key.to_string(), if *v { "참" } else { "거짓" }.to_string());
            Ok(())
        }
        Value::String(v) => {
            world.set_resource_json(key.to_string(), v.clone());
            Ok(())
        }
        _ => Err("set_param: 수/참거짓/글 스칼라만 허용됩니다".to_string()),
    }
}

fn handle_to_string(handle: ResourceHandle) -> String {
    format!("handle:{}", handle.to_hex())
}

fn seed_bogae_defaults(defaults: &mut HashMap<String, Value>) {
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
        JsonValue::String(SPACE2D_SCHEMA.to_string()),
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

fn points_from_resource_value(value: &ResourceValue) -> Option<Vec<(f64, f64)>> {
    let ResourceValue::List(items) = value else {
        return None;
    };
    let mut out = Vec::new();
    for item in items {
        if let Some(point) = resource_value_to_point(item) {
            out.push(point);
        }
    }
    Some(out)
}

fn resource_value_to_point(value: &ResourceValue) -> Option<(f64, f64)> {
    match value {
        ResourceValue::Map(entries) => {
            let mut x = None;
            let mut y = None;
            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                if key == "x" {
                    x = resource_value_to_number(&entry.value);
                } else if key == "y" {
                    y = resource_value_to_number(&entry.value);
                }
            }
            match (x, y) {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            }
        }
        ResourceValue::List(items) => {
            if items.len() < 2 {
                return None;
            }
            Some((
                resource_value_to_number(&items[0])?,
                resource_value_to_number(&items[1])?,
            ))
        }
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

fn resource_key_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

fn resource_value_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
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

fn fixed64_to_f64(value: Fixed64) -> f64 {
    value.raw_i64() as f64 / Fixed64::ONE_RAW as f64
}
