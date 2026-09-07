//! `ddn.exact_acyclic_relation.v1` owner-approved bounded model runtime.
//!
//! This module intentionally implements only one-shot, acyclic, directly
//! targetable relations. It does not implement events, derivatives, connector
//! execution, general equations, ODE/DAE, or backend fallback.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

use crate::work_cost::{
    model_expr_registry_key, verify_stdlib_registry_totality, WorkCostFailure, WorkCostMeter,
};

use ddonirang_core::{unit_spec_from_symbol, Fixed64, UnitDim};
use ddonirang_lang::{
    fixed_default_precise_recipe, ExecutionContractProfile, Initialization, LoweredRelationTarget,
    ModelDeclaration, ModelDefinition, ModelExpr, ModelExprNode, ModelInstance, ModelProgram,
    ModelSpan, PrimaryRole, RunRequest, SolveRecipe, EXACT_ACYCLIC_RELATION_BACKEND_ID,
    EXECUTION_CONTRACT_FIELD_EVIDENCE, EXECUTION_CONTRACT_FIELD_EXTERNAL_INGRESS,
    EXECUTION_CONTRACT_FIELD_PERFORMANCE, EXECUTION_CONTRACT_FIELD_REPLAY,
    EXECUTION_CONTRACT_FIELD_REPRO_SCOPE, EXECUTION_CONTRACT_FIELD_RESOURCE,
    EXECUTION_CONTRACT_FIELD_RESULT,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use serde_json::{json, Value as JsonValue};

pub const MODEL_BACKEND_ID: &str = EXACT_ACYCLIC_RELATION_BACKEND_ID;
pub const MODEL_BACKEND_DESCRIPTOR_SCHEMA: &str = "ddn.model_backend_descriptor.candidate.v1";
pub const MODEL_EXECUTION_SCHEMA: &str = "ddn.model_execution.candidate.v1";
pub const MODEL_RECEIPT_SCHEMA: &str = "ddn.model_execution_receipt.candidate.v1";
pub const MODEL_ARITHMETIC_POLICY_ID: &str = "ddn.strict_exact.v1";
pub const EXECUTION_CONTRACT_CAPABILITY_MATRIX_SCHEMA: &str =
    "ddn.model_execution_contract_capability_matrix.candidate.v1";

#[derive(Debug, Clone)]
pub struct ModelExecutionError {
    pub candidate_code: &'static str,
    pub phase: &'static str,
    pub message: String,
    pub source_path: String,
    pub provenance: ModelSpan,
    pub cause: Option<JsonValue>,
    pub offending_field: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub pre_state_hash: String,
    pub post_state_hash: String,
    pub receipt: JsonValue,
}

impl ModelExecutionError {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "schema": "ddn.model_execution_failure.candidate.v1",
            "wire_status": "candidate_non_public",
            "status": "failed",
            "candidate_code": self.candidate_code,
            "phase": self.phase,
            "message": self.message,
            "source_path": self.source_path,
            "source_span": {"start": self.provenance.start, "end": self.provenance.end},
            "provenance": [self.provenance.start, self.provenance.end],
            "cause": self.cause,
            "offending_field": self.offending_field,
            "expected": self.expected,
            "actual": self.actual,
            "pre_state_hash": self.pre_state_hash,
            "post_state_hash": self.post_state_hash,
            "receipt": self.receipt,
        })
    }
}

impl fmt::Display for ModelExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}..{}]: {}",
            self.candidate_code, self.provenance.start, self.provenance.end, self.message
        )
    }
}

impl std::error::Error for ModelExecutionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericKind {
    Exact,
    FixedQ32,
}

#[derive(Debug, Clone)]
struct ExactQuantity {
    base_value: BigRational,
    dimension: UnitDim,
    numeric_kind: NumericKind,
    authored_scale: BigRational,
    authored_unit: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct ReceiptContext {
    work_cost: Rc<RefCell<WorkCostMeter>>,
    source_path: String,
    program_source_hash: Option<String>,
    canonical_program_hash: Option<String>,
    definition_semantic_hash: Option<String>,
    definition_source_hash: Option<String>,
    instance_identity: Option<String>,
    instance_binding_hash: Option<String>,
    recipe_identity: Option<String>,
    recipe_hash: Option<String>,
    request_identity: Option<String>,
    request_hash: Option<String>,
    input_snapshot_hash: Option<String>,
    unit_registry_hash: Option<String>,
    dependency_order_hash: Option<String>,
    requested_output_order: Vec<String>,
    solve_method_binding_kind: Option<&'static str>,
    solve_method_authored: Option<bool>,
    solve_method_source_file: Option<String>,
    solve_method_source_range: Option<[usize; 2]>,
    public_solve_method: Option<String>,
    execution_contract_profile_identity: Option<String>,
    execution_contract_typed_input_component_hash: Option<String>,
    execution_contract_authored: Option<bool>,
    execution_contract_source_file: Option<String>,
    execution_contract_source_range: Option<[usize; 2]>,
    execution_contract_capability_matrix: Option<JsonValue>,
}

pub fn backend_descriptor_json() -> JsonValue {
    let mut descriptor = json!({
        "schema": MODEL_BACKEND_DESCRIPTOR_SCHEMA,
        "wire_status": "candidate_non_public",
        "backend_id": MODEL_BACKEND_ID,
        "version": 1,
        "arithmetic_policy_id": MODEL_ARITHMETIC_POLICY_ID,
        "accepted_model_class": "pure_deterministic_acyclic_direct_target_relation",
        "capabilities": [
            "relation",
            "acyclic_dependency",
            "exact_arithmetic",
            "unit_dimension",
            "explicit_unknown",
            "explicit_output"
        ],
        "target_eligibility": {
            "relation_is_directionless": true,
            "exactly_one_direct_explicit_unknown_side": true,
            "target_may_be_left_or_right": true,
            "target_reappearance_forbidden": true
        },
        "unsupported": [
            "event",
            "derivative",
            "connection_runtime",
            "relation_cycle",
            "general_equation",
            "ode",
            "dae",
            "external_solver",
            "fallback"
        ]
    });
    let descriptor_hash = hash_json(&descriptor);
    descriptor
        .as_object_mut()
        .expect("descriptor object")
        .insert("descriptor_hash".to_string(), json!(descriptor_hash));
    descriptor
}

pub fn runtime_build_identity_json() -> JsonValue {
    json!({
        "schema": "ddn.model_runtime_build_identity.candidate.v1",
        "package": env!("CARGO_PKG_NAME"),
        "package_version": env!("CARGO_PKG_VERSION"),
        "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "model_ir_schema": ddonirang_lang::MODEL_IR_SCHEMA_CANDIDATE,
        "backend_descriptor_schema": MODEL_BACKEND_DESCRIPTOR_SCHEMA,
        "receipt_schema": MODEL_RECEIPT_SCHEMA,
    })
}

fn backend_capability_set_hash() -> String {
    hash_json(&backend_descriptor_json()["capabilities"])
}

pub fn execution_contract_capability_matrix_json(contract: &ExecutionContractProfile) -> JsonValue {
    json!({
        "schema": EXECUTION_CONTRACT_CAPABILITY_MATRIX_SCHEMA,
        "wire_status": "candidate_non_public",
        "profile_identity": contract.profile_identity,
        "profile_version": contract.profile_version,
        "runtime_activation_authorized": true,
        "all_required_capabilities_satisfied": true,
        "silent_fallback": false,
        "unsupported_contract_downgrade": false,
        "rows": [
            {
                "field": EXECUTION_CONTRACT_FIELD_RESULT,
                "requested": format!("#{}", contract.result.value.surface()),
                "status": "available_bounded_backend",
                "runtime_success_ready": true,
                "evidence": "exact arithmetic backend and canonical result-pack hash",
            },
            {
                "field": EXECUTION_CONTRACT_FIELD_PERFORMANCE,
                "requested": format!("#{}", contract.performance.value.surface()),
                "status": "available_no_unsafe_optimization",
                "runtime_success_ready": true,
                "evidence": "fixed exact backend; acceleration, parallel discovery, retry and fallback absent",
            },
            {
                "field": EXECUTION_CONTRACT_FIELD_EXTERNAL_INGRESS,
                "requested": format!("#{}", contract.external_ingress.value.surface()),
                "status": "available_bounded_input_snapshot",
                "runtime_success_ready": true,
                "evidence": "bounded model input snapshot hash; external adapter execution absent",
            },
            {
                "field": EXECUTION_CONTRACT_FIELD_RESOURCE,
                "requested": format!("#{}", contract.resource.value.surface()),
                "status": "available_common_rust_workcost_v1",
                "runtime_success_ready": true,
                "evidence": "independently reviewed WorkCostV1 totality and standard-limit enforcement are bound to the shared model runtime meter",
            },
            {
                "field": EXECUTION_CONTRACT_FIELD_REPLAY,
                "requested": format!("#{}", contract.replay.value.surface()),
                "status": "available_verified_identical_input_rerun",
                "runtime_success_ready": true,
                "evidence": "the parsed canonical program is executed again with the same sealed input and result plus typed identity are compared before commit",
                "nonclaim": "checkpoint, continuation and N-slice equivalence remain OPEN",
            },
            {
                "field": EXECUTION_CONTRACT_FIELD_REPRO_SCOPE,
                "requested": format!("#{}", contract.repro_scope.value.surface()),
                "status": "available_shared_rust_native_wasm",
                "runtime_success_ready": true,
                "evidence": "native CLI, WASM and Workbench consume the same Rust execution and verified-replay owner; clean-detached identity is a release gate",
            },
            {
                "field": EXECUTION_CONTRACT_FIELD_EVIDENCE,
                "requested": format!("#{}", contract.evidence.value.surface()),
                "status": "available_candidate_receipt",
                "runtime_success_ready": true,
                "evidence": "candidate receipt preserves source, canonical, input, result and state hashes",
            },
        ],
        "blocking_fields": [],
        "claim_boundaries": {
            "workcost_v1_totality": "independent_pass_bound",
            "checkpoint_and_continuation": "open",
            "rd207_stable_public_wire": "open",
        },
    })
}

pub fn execution_envelope_json(source: &str) -> Result<JsonValue, ModelExecutionError> {
    let program = ddonirang_lang::parse_model_program(source).map_err(|err| {
        failure(
            err.candidate_code,
            "model_frontdoor_candidate",
            err.message,
            err.span,
            &ReceiptContext::default(),
        )
    })?;
    let model_ir = ddonirang_lang::model_ir_candidate_json(&program);
    let execution_contract = model_ir["requests"]
        .as_array()
        .and_then(|requests| requests.first())
        .and_then(|request| request.get("execution_contract"))
        .cloned()
        .unwrap_or(JsonValue::Null);
    let capability_matrix = program
        .ir
        .requests
        .first()
        .map(|request| execution_contract_capability_matrix_json(&request.execution_contract))
        .unwrap_or(JsonValue::Null);
    Ok(json!({
        "schema": "ddn.model_execution_envelope.candidate.v1",
        "wire_status": "candidate_non_public",
        "source_hash": program.source_hash,
        "semantic_hash": program.ir.semantic_hash,
        "canonical_ddn_hash": hash_bytes(program.canonical_ddn.as_bytes()),
        "backend_descriptor": backend_descriptor_json(),
        "execution_contract": execution_contract,
        "execution_contract_capability_matrix": capability_matrix,
        "execution_mode": "oneshot",
        "configured_madi": 1,
        "execution_mode_decision": {
            "schema": "ddn.execution_mode_decision.v1",
            "mode": "oneshot",
            "authority": "parsed_program",
            "reason": "bounded_model_backend",
            "live_hook_count": 0,
            "every_madi_hook_count": 0,
            "every_n_madi_hook_count": 0,
        },
        "atomic_commit": true,
        "silent_fallback": false,
    }))
}

pub fn execute_source(source: &str) -> Result<JsonValue, ModelExecutionError> {
    execute_source_with_source_file(source, "<memory>")
}

pub fn execute_source_with_source_file(
    source: &str,
    source_file: &str,
) -> Result<JsonValue, ModelExecutionError> {
    let context = ReceiptContext {
        source_path: source_file.to_string(),
        ..ReceiptContext::default()
    };
    context
        .work_cost
        .borrow_mut()
        .set_sealed_input_bytes(source.len())
        .map_err(|error| work_cost_failure(error, ModelSpan { start: 0, end: 0 }, &context))?;
    context
        .work_cost
        .borrow_mut()
        .set_replay_capsule_bytes(0)
        .map_err(|error| work_cost_failure(error, ModelSpan { start: 0, end: 0 }, &context))?;
    verify_stdlib_registry_totality()
        .map_err(|error| work_cost_failure(error, ModelSpan { start: 0, end: 0 }, &context))?;
    let program = ddonirang_lang::parse_model_program_with_source_file(source, source_file)
        .map_err(|err| {
            failure(
                err.candidate_code,
                "model_frontdoor_candidate",
                err.message,
                err.span,
                &context,
            )
        })?;
    execute_program(&program, context.work_cost)
}

pub fn execute_source_pretty_json(source: &str) -> Result<String, ModelExecutionError> {
    execute_source_pretty_json_with_source_file(source, "<memory>")
}

pub fn execute_source_pretty_json_with_source_file(
    source: &str,
    source_file: &str,
) -> Result<String, ModelExecutionError> {
    execute_source_with_source_file(source, source_file).and_then(|value| {
        serde_json::to_string_pretty(&value)
            .map(|text| format!("{text}\n"))
            .map_err(|err| {
                failure(
                    "E_MODEL_EXECUTION_SERIALIZE_CANDIDATE",
                    "model_receipt_candidate",
                    err.to_string(),
                    ModelSpan { start: 0, end: 0 },
                    &ReceiptContext::default(),
                )
            })
    })
}

fn verification_failure(candidate_code: &'static str, message: impl Into<String>) -> JsonValue {
    json!({
        "schema": "ddn.model_execution_verification.candidate.v1",
        "wire_status": "candidate_non_public",
        "ok": false,
        "candidate_code": candidate_code,
        "message": message.into(),
    })
}

pub fn verify_execution_report(report: &JsonValue) -> JsonValue {
    if report.get("schema").and_then(JsonValue::as_str) != Some(MODEL_EXECUTION_SCHEMA) {
        return verification_failure(
            "E_MODEL_EXECUTION_SCHEMA_MISMATCH_CANDIDATE",
            "model execution schema가 일치하지 않습니다",
        );
    }
    if report.get("status").and_then(JsonValue::as_str) != Some("completed")
        || report.get("atomic_commit").and_then(JsonValue::as_str) != Some("committed")
    {
        return verification_failure(
            "E_MODEL_EXECUTION_NOT_COMMITTED_CANDIDATE",
            "검증 대상은 committed 성공 실행이어야 합니다",
        );
    }
    let Some(descriptor) = report.get("backend_descriptor") else {
        return verification_failure(
            "E_MODEL_BACKEND_DESCRIPTOR_MISSING_CANDIDATE",
            "backend descriptor가 없습니다",
        );
    };
    if descriptor.get("backend_id").and_then(JsonValue::as_str) != Some(MODEL_BACKEND_ID) {
        return verification_failure(
            "E_MODEL_BACKEND_DESCRIPTOR_MISMATCH_CANDIDATE",
            "승인된 bounded backend가 아닙니다",
        );
    }
    let mut descriptor_without_hash = descriptor.clone();
    let declared_descriptor_hash = descriptor_without_hash
        .as_object_mut()
        .and_then(|object| object.remove("descriptor_hash"))
        .and_then(|value| value.as_str().map(str::to_string));
    let computed_descriptor_hash = hash_json(&descriptor_without_hash);
    if declared_descriptor_hash.as_deref() != Some(computed_descriptor_hash.as_str()) {
        return verification_failure(
            "E_MODEL_BACKEND_DESCRIPTOR_HASH_MISMATCH_CANDIDATE",
            "backend descriptor hash가 일치하지 않습니다",
        );
    }
    let Some(receipt) = report.get("receipt") else {
        return verification_failure(
            "E_MODEL_RECEIPT_MISSING_CANDIDATE",
            "model execution receipt가 없습니다",
        );
    };
    let mut receipt_without_hash = receipt.clone();
    let declared_receipt_hash = receipt_without_hash
        .as_object_mut()
        .and_then(|object| object.remove("receipt_hash"))
        .and_then(|value| value.as_str().map(str::to_string));
    let computed_receipt_hash = hash_json(&receipt_without_hash);
    if declared_receipt_hash.as_deref() != Some(computed_receipt_hash.as_str()) {
        return verification_failure(
            "E_MODEL_RECEIPT_HASH_MISMATCH_CANDIDATE",
            "receipt content hash가 일치하지 않습니다",
        );
    }
    if receipt.get("backend_descriptor_hash") != descriptor.get("descriptor_hash")
        || receipt.get("backend_id").and_then(JsonValue::as_str) != Some(MODEL_BACKEND_ID)
    {
        return verification_failure(
            "E_MODEL_RECEIPT_BACKEND_BINDING_MISMATCH_CANDIDATE",
            "receipt와 backend descriptor 결속이 일치하지 않습니다",
        );
    }
    if receipt
        .get("backend_capability_set_hash")
        .and_then(JsonValue::as_str)
        != Some(backend_capability_set_hash().as_str())
        || receipt.get("runtime_build_identity") != Some(&runtime_build_identity_json())
    {
        return verification_failure(
            "E_MODEL_RECEIPT_RUNTIME_BINDING_MISMATCH_CANDIDATE",
            "receipt의 풀이 능력 또는 runtime build 결속이 일치하지 않습니다",
        );
    }
    let bindings = &report["execution_bindings"];
    if bindings["program_source_hash"] != receipt["program_source_hash"]
        || bindings["canonical_program_hash"] != receipt["canonical_program_hash"]
        || hash_bytes(
            report["canonical_ddn"]
                .as_str()
                .unwrap_or_default()
                .as_bytes(),
        ) != receipt["canonical_program_hash"]
            .as_str()
            .unwrap_or_default()
    {
        return verification_failure(
            "E_MODEL_PROGRAM_IDENTITY_BINDING_MISMATCH_CANDIDATE",
            "source/canon identity와 receipt 결속이 일치하지 않습니다",
        );
    }
    let method = &receipt["solve_method"];
    let request_method = &bindings["run_request"]["method_provenance"];
    if method["binding_kind"] != request_method["kind"]
        || method["authored"] != request_method["authored"]
        || method["source_file"] != request_method["source_file"]
        || method["source_range"] != request_method["source_range"]
        || method["public_surface"] != request_method["public_method"]
        || method["resolution_policy"] != "fixed_precise_no_discovery_no_fallback"
        || method["backend_options"] != json!({})
        || method["backend_limits"]["accepted_model_class"]
            != "pure_deterministic_acyclic_direct_target_relation"
        || method["backend_limits"]["numeric_policy"] != MODEL_ARITHMETIC_POLICY_ID
        || method["backend_registry"]["lookup"] != "fixed_public_method_mapping"
        || method["backend_registry"]["registration_order_observed"] != false
        || method["backend_registry"]["installed_backend_count_observed"] != false
        || method["compatibility"]["fixed_default_contract"] != "ssot.v24.28.2.rd153-155"
        || method["compatibility"]["historical_exact_descriptor_explicit_only"] != true
        || method["retry"] != false
        || method["fallback"] != false
    {
        return verification_failure(
            "E_MODEL_SOLVE_METHOD_BINDING_MISMATCH_CANDIDATE",
            "풀이방법 작성 여부·공개 의미·원문 위치 결속이 일치하지 않습니다",
        );
    }
    if method["binding_kind"] == "omitted_fixed_precise"
        && (method["authored"] != false || method["public_surface"] != "#정밀")
    {
        return verification_failure(
            "E_MODEL_FIXED_DEFAULT_METHOD_MISMATCH_CANDIDATE",
            "생략된 풀이방법은 고정 공개 의미 #정밀이어야 합니다",
        );
    }
    let binding_checks = [
        (
            &bindings["model_definition_artifact"]["source_hash"],
            &receipt["model_definition_source_hash"],
            "E_MODEL_DEFINITION_BINDING_MISMATCH_CANDIDATE",
        ),
        (
            &bindings["solve_recipe"]["source_hash"],
            &receipt["solve_recipe_hash"],
            "E_MODEL_RECIPE_BINDING_MISMATCH_CANDIDATE",
        ),
        (
            &bindings["run_request"]["source_hash"],
            &receipt["run_request_hash"],
            "E_MODEL_REQUEST_BINDING_MISMATCH_CANDIDATE",
        ),
    ];
    for (actual, expected, code) in binding_checks {
        if actual.is_null() || actual != expected {
            return verification_failure(code, "execution artifact와 receipt hash 결속이 다릅니다");
        }
    }
    if hash_json(&report["result_pack"]) != receipt["result_pack_hash"].as_str().unwrap_or_default()
    {
        return verification_failure(
            "E_MODEL_RESULT_PACK_HASH_MISMATCH_CANDIDATE",
            "result pack hash가 receipt와 일치하지 않습니다",
        );
    }
    let committed_state_hash = hash_json(&report["committed_state"]);
    if report["post_state_hash"].as_str() != Some(committed_state_hash.as_str())
        || receipt["post_state_hash"].as_str() != Some(committed_state_hash.as_str())
    {
        return verification_failure(
            "E_MODEL_COMMITTED_STATE_HASH_MISMATCH_CANDIDATE",
            "committed state hash 결속이 일치하지 않습니다",
        );
    }
    let output_order = report["result_pack"]["ordered_fields"]
        .as_array()
        .map(|fields| {
            fields
                .iter()
                .filter_map(|field| field["name"].as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let receipt_order = receipt["requested_output_order"]
        .as_array()
        .map(|fields| {
            fields
                .iter()
                .filter_map(JsonValue::as_str)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if output_order != receipt_order {
        return verification_failure(
            "E_MODEL_OUTPUT_ORDER_BINDING_MISMATCH_CANDIDATE",
            "요청 출력 순서와 result pack 순서가 일치하지 않습니다",
        );
    }
    let verified_replay = &report["verified_replay"];
    let capsule = &verified_replay["capsule"];
    let capsule_hash = hash_json(capsule);
    if verified_replay["capsule_hash"].as_str() != Some(capsule_hash.as_str())
        || receipt["verified_replay_capsule_hash"].as_str() != Some(capsule_hash.as_str())
    {
        return verification_failure(
            "E_MODEL_EXECUTION_CONTRACT_CAPABILITY_UNSUPPORTED_CANDIDATE",
            "#검증재생 capsule hash와 receipt 결속이 일치하지 않습니다",
        );
    }
    if capsule["sealed_identical_input"] != true
        || capsule["result_and_identity_verified"] != true
        || capsule["checkpoint_restored"] != false
        || capsule["continuation_present"] != false
        || capsule["n_slice_equivalence_claimed"] != false
        || capsule["identity_projection"] != replay_identity_projection(report)
    {
        return verification_failure(
            "E_MODEL_EXECUTION_CONTRACT_CAPABILITY_UNSUPPORTED_CANDIDATE",
            "#검증재생 capsule의 동일 입력 결과·identity 검증이 일치하지 않습니다",
        );
    }
    json!({
        "schema": "ddn.model_execution_verification.candidate.v1",
        "wire_status": "candidate_non_public",
        "ok": true,
        "backend_id": MODEL_BACKEND_ID,
        "receipt_hash": declared_receipt_hash,
        "post_state_hash": committed_state_hash,
        "verified_replay_capsule_hash": capsule_hash,
    })
}

fn replay_identity_projection(report: &JsonValue) -> JsonValue {
    json!({
        "program_source_hash": report["execution_bindings"]["program_source_hash"],
        "canonical_program_hash": report["execution_bindings"]["canonical_program_hash"],
        "model_ir_semantic_hash": report["execution_bindings"]["model_ir_semantic_hash"],
        "model_definition_source_hash": report["receipt"]["model_definition_source_hash"],
        "solve_recipe_identity": report["receipt"]["solve_recipe_identity"],
        "solve_recipe_hash": report["receipt"]["solve_recipe_hash"],
        "run_request_identity": report["receipt"]["run_request_identity"],
        "run_request_hash": report["receipt"]["run_request_hash"],
        "execution_contract_profile_identity": report["receipt"]["execution_contract"]["profile_identity"],
        "execution_contract_typed_input_component_hash": report["receipt"]["execution_contract"]["typed_input_component_hash"],
        "input_snapshot_hash": report["receipt"]["input_snapshot_hash"],
        "result_pack_hash": report["receipt"]["result_pack_hash"],
        "post_state_hash": report["post_state_hash"],
        "result_pack": report["result_pack"],
        "public_observations": report["public_observations"],
    })
}

fn execute_program(
    program: &ModelProgram,
    work_cost: Rc<RefCell<WorkCostMeter>>,
) -> Result<JsonValue, ModelExecutionError> {
    let mut first = execute_program_once(program, work_cost.clone())?;
    let second = execute_program_once(program, work_cost.clone())?;
    let first_projection = replay_identity_projection(&first);
    let second_projection = replay_identity_projection(&second);
    if first_projection != second_projection {
        let request = program.ir.requests.first();
        let provenance = request
            .and_then(|request| {
                request
                    .execution_contract
                    .field_value_span(EXECUTION_CONTRACT_FIELD_REPLAY)
            })
            .unwrap_or(ModelSpan { start: 0, end: 0 });
        return Err(failure(
            "E_MODEL_EXECUTION_CONTRACT_CAPABILITY_UNSUPPORTED_CANDIDATE",
            "model_execution_contract_verified_replay_candidate",
            "#검증재생 동일 입력 재실행의 결과 또는 typed identity가 일치하지 않습니다",
            provenance,
            &ReceiptContext {
                work_cost,
                program_source_hash: Some(program.source_hash.clone()),
                canonical_program_hash: Some(hash_bytes(program.canonical_ddn.as_bytes())),
                ..ReceiptContext::default()
            },
        ));
    }

    let capsule_body = json!({
        "schema": "ddn.model_execution_verified_replay_capsule.candidate.v1",
        "wire_status": "candidate_non_public",
        "sealed_identical_input": true,
        "result_and_identity_verified": true,
        "checkpoint_restored": false,
        "continuation_present": false,
        "n_slice_equivalence_claimed": false,
        "identity_projection": first_projection,
    });
    let capsule_bytes = serde_json::to_vec(&capsule_body).map_err(|error| {
        failure(
            "E_MODEL_EXECUTION_CONTRACT_CAPABILITY_UNSUPPORTED_CANDIDATE",
            "model_execution_contract_verified_replay_candidate",
            error.to_string(),
            ModelSpan { start: 0, end: 0 },
            &ReceiptContext {
                work_cost: work_cost.clone(),
                program_source_hash: Some(program.source_hash.clone()),
                canonical_program_hash: Some(hash_bytes(program.canonical_ddn.as_bytes())),
                ..ReceiptContext::default()
            },
        )
    })?;
    work_cost
        .borrow_mut()
        .set_replay_capsule_bytes(capsule_bytes.len())
        .map_err(|error| {
            work_cost_failure(
                error,
                ModelSpan { start: 0, end: 0 },
                &ReceiptContext {
                    work_cost: work_cost.clone(),
                    program_source_hash: Some(program.source_hash.clone()),
                    canonical_program_hash: Some(hash_bytes(program.canonical_ddn.as_bytes())),
                    ..ReceiptContext::default()
                },
            )
        })?;
    let capsule_hash = hash_json(&capsule_body);
    first["verified_replay"] = json!({
        "capsule": capsule_body,
        "capsule_hash": capsule_hash.clone(),
        "capsule_bytes": capsule_bytes.len(),
    });
    let updated_work_cost =
        serde_json::to_value(work_cost.borrow().receipt()).expect("WorkCostV1 receipt serializes");
    first["work_cost_receipt"] = updated_work_cost.clone();
    let mut updated_receipt = first["receipt"].clone();
    if let Some(object) = updated_receipt.as_object_mut() {
        object.remove("receipt_hash");
        object.insert("work_cost".to_string(), updated_work_cost);
        object.insert(
            "verified_replay_capsule_hash".to_string(),
            json!(capsule_hash),
        );
    }
    first["receipt"] = seal_receipt(updated_receipt);
    Ok(first)
}

fn execute_program_once(
    program: &ModelProgram,
    work_cost: Rc<RefCell<WorkCostMeter>>,
) -> Result<JsonValue, ModelExecutionError> {
    let empty_state = empty_state_hash();
    let mut context = ReceiptContext {
        work_cost,
        program_source_hash: Some(program.source_hash.clone()),
        canonical_program_hash: Some(hash_bytes(program.canonical_ddn.as_bytes())),
        ..ReceiptContext::default()
    };
    if program.ir.requests.len() != 1 {
        return Err(failure(
            "E_MODEL_RUN_COUNT_UNSUPPORTED_CANDIDATE",
            "model_request_candidate",
            "첫 제한 backend는 정확히 한 개의 풀기 요청만 실행합니다",
            ModelSpan {
                start: 0,
                end: program.canonical_ddn.len(),
            },
            &context,
        ));
    }
    let request = &program.ir.requests[0];
    charge_model_dispatch(&context, request.span)?;
    context.request_identity = Some(request.identity.clone());
    context.request_hash = Some(request.source_hash.clone());
    context.requested_output_order = request.output_selection.clone();
    context.solve_method_binding_kind = Some(request.method_provenance.surface_kind());
    context.solve_method_authored = Some(request.method_provenance.is_authored());
    context.solve_method_source_file = Some(request.method_provenance.source_file().to_string());
    let method_range = request.method_provenance.source_range();
    context.solve_method_source_range = Some([method_range.start, method_range.end]);
    if !request.method_provenance.is_authored() {
        context.public_solve_method = Some("#정밀".to_string());
    }
    let contract = &request.execution_contract;
    let contract_range = contract.provenance.source_range();
    context.execution_contract_profile_identity = Some(contract.profile_identity.to_string());
    context.execution_contract_typed_input_component_hash =
        Some(contract.typed_input_component_hash());
    context.execution_contract_authored = Some(contract.provenance.is_authored());
    context.execution_contract_source_file = Some(contract.provenance.source_file().to_string());
    context.execution_contract_source_range = Some([contract_range.start, contract_range.end]);
    context.execution_contract_capability_matrix =
        Some(execution_contract_capability_matrix_json(contract));

    validate_execution_contract_runtime_capability(contract, &context)?;

    let target = request.instance_binding.as_deref().ok_or_else(|| {
        failure(
            "E_MODEL_RUN_TARGET_CANDIDATE",
            "model_binding_candidate",
            "대상은 direct model value path여야 합니다",
            request.span,
            &context,
        )
    })?;
    let definition = program
        .ir
        .definitions
        .iter()
        .find(|definition| definition.name == target)
        .ok_or_else(|| {
            failure(
                "E_MODEL_INSTANCE_PATH_CANDIDATE",
                "model_binding_candidate",
                format!("대상 '{target}'에 결속된 ModelDefinition을 찾을 수 없습니다"),
                request.span,
                &context,
            )
        })?;
    context.definition_source_hash = Some(definition.source_hash.clone());
    context.definition_semantic_hash = Some(definition_semantic_hash(definition));

    // Capability rejection precedes structural solve diagnostics. This keeps an
    // event/derivative/connection model visibly OPEN instead of accidentally
    // presenting a secondary equation-count diagnostic as backend behavior.
    validate_backend_eligibility(definition, request, &context)?;
    // A whole-system equation-count failure is more specific than the
    // duplicate-producer detail that necessarily accompanies it.  A plain
    // duplicate producer remains visible when equation and unknown counts are
    // otherwise balanced.
    let structural_diagnostic = program
        .ir
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.candidate_code == "E_MODEL_OVERDETERMINED_CANDIDATE"
                && diagnostic.message.contains("대수 관계")
        })
        .or_else(|| program.ir.diagnostics.first());
    if let Some(diagnostic) = structural_diagnostic {
        return Err(failure(
            diagnostic.candidate_code,
            diagnostic.phase,
            diagnostic.message.clone(),
            diagnostic.provenance,
            &context,
        ));
    }
    let recipe = resolve_recipe(program, request, &context)?;
    context.recipe_identity = Some(recipe.identity.clone());
    context.recipe_hash = Some(recipe.source_hash.clone());
    context.public_solve_method = recipe
        .method_value
        .as_ref()
        .map(|method| method.diagnostic_surface().to_string());
    validate_recipe(&recipe, &context)?;

    let initial_map = request
        .initial_overrides
        .iter()
        .map(|field| (field.name.clone(), field.value.clone()))
        .collect::<BTreeMap<_, _>>();
    let instance = ModelInstance::candidate(
        definition.identity.clone(),
        format!("{}#{}", definition.name, request.binding_name),
        BTreeMap::new(),
        initial_map,
        request.span,
    );
    context
        .work_cost
        .borrow_mut()
        .note_created_identity()
        .map_err(|error| work_cost_failure(error, request.span, &context))?;
    context.instance_identity = Some(instance.identity.clone());
    context.instance_binding_hash = Some(instance.source_hash.clone());
    context.input_snapshot_hash = Some(input_snapshot_hash(request));
    context.unit_registry_hash = Some(unit_registry_hash(definition)?);

    let mut environment = prepare_environment(program, definition, request, &context)?;
    let dependency_order = solve_relations(definition, &mut environment, &context)?;
    context.dependency_order_hash = Some(hash_json(&json!(dependency_order)));

    let result_pack = build_result_pack(definition, request, &environment, &context)?;
    let result_pack_hash = hash_json(&result_pack);
    let committed_state = json!({
        "schema": "ddn.model_committed_state.candidate.v1",
        "bindings": [{
            "name": request.binding_name,
            "value": result_pack,
        }]
    });
    let post_state_hash = hash_json(&committed_state);
    let public_observations = build_public_observations(program, request, &result_pack, &context)?;
    context
        .work_cost
        .borrow_mut()
        .note_public_observation_samples(public_observations.len())
        .map_err(|error| work_cost_failure(error, request.span, &context))?;
    let receipt = success_receipt(&context, &empty_state, &post_state_hash, &result_pack_hash);
    let model_ir_artifact = ddonirang_lang::model_ir_candidate_json(program);
    let model_definition_artifact = model_ir_artifact["definitions"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|row| row["identity"].as_str() == Some(definition.identity.as_str()))
        })
        .cloned()
        .unwrap_or(JsonValue::Null);
    let solve_recipe_artifact = model_ir_artifact["recipes"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|row| row["identity"].as_str() == Some(recipe.identity.as_str()))
        })
        .cloned()
        .unwrap_or_else(|| solve_recipe_artifact_json(&recipe, request));
    let run_request_artifact = model_ir_artifact["requests"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|row| row["identity"].as_str() == Some(request.identity.as_str()))
        })
        .cloned()
        .unwrap_or(JsonValue::Null);

    Ok(json!({
        "schema": MODEL_EXECUTION_SCHEMA,
        "wire_status": "candidate_non_public",
        "status": "completed",
        "backend_descriptor": backend_descriptor_json(),
        "canonical_ddn": program.canonical_ddn,
        "model_ir_semantic_hash": program.ir.semantic_hash,
        "work_cost_receipt": context.work_cost.borrow().receipt(),
        "pre_state_hash": empty_state,
        "post_state_hash": post_state_hash,
        "atomic_commit": "committed",
        "dependency_order": dependency_order,
        "result_binding": request.binding_name,
        "result_pack": result_pack,
        "public_observations": public_observations,
        "execution_bindings": {
            "schema": "ddn.model_execution_bindings.candidate.v1",
            "wire_status": "candidate_non_public",
            "model_ir_schema": model_ir_artifact["schema"],
            "program_source_hash": program.source_hash,
            "canonical_program_hash": hash_bytes(program.canonical_ddn.as_bytes()),
            "model_ir_semantic_hash": model_ir_artifact["semantic_hash"],
            "model_definition_artifact": model_definition_artifact,
            "solve_recipe": solve_recipe_artifact,
            "run_request": run_request_artifact,
        },
        "receipt": receipt,
        "committed_state": committed_state,
    }))
}

fn validate_execution_contract_runtime_capability(
    contract: &ExecutionContractProfile,
    context: &ReceiptContext,
) -> Result<(), ModelExecutionError> {
    let matrix = execution_contract_capability_matrix_json(contract);
    if matrix["runtime_activation_authorized"] != true
        || matrix["all_required_capabilities_satisfied"] != true
        || matrix["blocking_fields"] != json!([])
    {
        let resource_span = contract
            .field_value_span(EXECUTION_CONTRACT_FIELD_RESOURCE)
            .unwrap_or_else(|| contract.provenance.source_range());
        return Err(failure(
            "E_MODEL_EXECUTION_CONTRACT_CAPABILITY_UNSUPPORTED_CANDIDATE",
            "model_execution_contract_capability_candidate",
            "표준재현_v1의 모든 runtime capability가 충족되지 않았습니다",
            resource_span,
            context,
        ));
    }
    context
        .work_cost
        .borrow_mut()
        .authorize_rd206_resource_capability();
    Ok(())
}

fn validate_backend_eligibility(
    definition: &ModelDefinition,
    request: &RunRequest,
    context: &ReceiptContext,
) -> Result<(), ModelExecutionError> {
    if !definition.events.is_empty() {
        return Err(failure(
            "E_MODEL_EVENT_RUNTIME_OPEN_CANDIDATE",
            "model_capability_candidate",
            "사건 실행은 #정밀 풀이방법에서 아직 지원하지 않습니다",
            definition.events[0].identity.span,
            context,
        ));
    }
    if !definition.connections.is_empty() {
        return Err(failure(
            "E_MODEL_CONNECTION_RUNTIME_OPEN_CANDIDATE",
            "model_capability_candidate",
            "연결 실행은 #정밀 풀이방법에서 아직 지원하지 않습니다",
            definition.connections[0].identity.span,
            context,
        ));
    }
    if let Some(relation) = definition.relations.iter().find(|relation| {
        matches!(
            relation.lowered.as_ref().map(|node| &node.target),
            Some(LoweredRelationTarget::Derivative(_))
        )
    }) {
        return Err(failure(
            "E_MODEL_DERIVATIVE_RUNTIME_OPEN_CANDIDATE",
            "model_capability_candidate",
            "미분·ODE 실행은 #정밀 풀이방법에서 아직 지원하지 않습니다",
            relation.identity.span,
            context,
        ));
    }
    if request.end.is_some() {
        return Err(failure(
            "E_MODEL_BACKEND_PIN_UNSUPPORTED_CANDIDATE",
            "model_request_candidate",
            "#정밀 풀이방법은 '끝=' 받는자리를 받지 않습니다",
            request
                .pins
                .iter()
                .find(|pin| pin.name == "끝")
                .map_or(request.span, |pin| pin.span),
            context,
        ));
    }
    Ok(())
}

fn resolve_recipe<'a>(
    program: &'a ModelProgram,
    request: &RunRequest,
    context: &ReceiptContext,
) -> Result<Cow<'a, SolveRecipe>, ModelExecutionError> {
    if !request.method_provenance.is_authored() {
        if let Some(recipe) = program
            .ir
            .recipes
            .iter()
            .find(|recipe| recipe.name == ddonirang_lang::FIXED_DEFAULT_PRECISE_RECIPE_NAME)
        {
            return Ok(Cow::Borrowed(recipe));
        }
        return Ok(Cow::Owned(fixed_default_precise_recipe(request)));
    }
    let binding = request.recipe_binding.as_deref().ok_or_else(|| {
        failure(
            "E_MODEL_RECIPE_BINDING_REQUIRED_CANDIDATE",
            "model_request_candidate",
            "방법=에는 direct SolveRecipe 이름이 필요합니다",
            request.span,
            context,
        )
    })?;
    program
        .ir
        .recipes
        .iter()
        .find(|recipe| recipe.name == binding)
        .map(Cow::Borrowed)
        .ok_or_else(|| {
            failure(
                "E_MODEL_RECIPE_BINDING_MISMATCH_CANDIDATE",
                "model_request_candidate",
                format!("방법 '{binding}'을 찾을 수 없습니다"),
                request.span,
                context,
            )
        })
}

fn solve_recipe_artifact_json(recipe: &SolveRecipe, request: &RunRequest) -> JsonValue {
    json!({
        "identity": recipe.identity,
        "name": recipe.name,
        "source_hash": recipe.source_hash,
        "schema_version": recipe.schema_version,
        "authored": request.method_provenance.is_authored(),
        "source_file": request.method_provenance.source_file(),
        "source_range": [
            request.method_provenance.source_range().start,
            request.method_provenance.source_range().end,
        ],
        "fields": recipe.fields.iter().map(|field| json!({
            "name": field.name,
            "value": field.value.canonical,
        })).collect::<Vec<_>>(),
    })
}

fn validate_recipe(
    recipe: &SolveRecipe,
    context: &ReceiptContext,
) -> Result<(), ModelExecutionError> {
    if recipe.fields.len() != 1 || recipe.fields[0].name != "방식" {
        return Err(failure(
            "E_MODEL_RECIPE_SCHEMA_CANDIDATE",
            "model_recipe_candidate",
            "첫 제한 recipe는 '방식' 필드 하나만 허용합니다",
            recipe.span,
            context,
        ));
    }
    let method = recipe.method_value.as_ref().ok_or_else(|| {
        failure(
            "E_MODEL_RECIPE_METHOD_REQUIRED_CANDIDATE",
            "model_recipe_candidate",
            "방식에는 풀이방법 값 #정밀이 필요합니다",
            recipe.fields[0].span,
            context,
        )
    })?;
    if method.backend_id() != Some(MODEL_BACKEND_ID) {
        return Err(failure(
            "E_MODEL_RECIPE_BACKEND_MISMATCH_CANDIDATE",
            "model_recipe_candidate",
            format!(
                "지원하지 않는 풀이방법 '{}'입니다. 현재 지원값은 #정밀입니다",
                method.diagnostic_surface()
            ),
            recipe.fields[0].span,
            context,
        ));
    }
    Ok(())
}

fn prepare_environment(
    program: &ModelProgram,
    definition: &ModelDefinition,
    request: &RunRequest,
    context: &ReceiptContext,
) -> Result<BTreeMap<String, ExactQuantity>, ModelExecutionError> {
    let declarations = definition
        .declarations
        .iter()
        .map(|declaration| (declaration.name.as_str(), declaration))
        .collect::<BTreeMap<_, _>>();
    let mut environment = BTreeMap::new();

    for binding in &program.ir.exact_prelude_bindings {
        if environment.contains_key(&binding.name) {
            return Err(failure(
                "E_MODEL_EXACT_PRELUDE_DUP_CANDIDATE",
                "model_binding_candidate",
                format!("자료 바인딩 '{}'이 중복되었습니다", binding.name),
                binding.span,
                context,
            ));
        }
        let node = binding.value.node.as_ref().ok_or_else(|| {
            failure(
                "E_MODEL_EXACT_PRELUDE_VALUE_CANDIDATE",
                "model_binding_candidate",
                "자료 바인딩은 exact 산술 값이어야 합니다",
                binding.span,
                context,
            )
        })?;
        let quantity = eval_node(
            node,
            &environment,
            Some(NumericKind::Exact),
            binding.span,
            context,
        )?;
        environment.insert(binding.name.clone(), quantity);
    }

    for declaration in &definition.declarations {
        if declaration.unknown {
            continue;
        }
        if let Initialization::Fixed(value) = &declaration.initialization {
            let quantity = evaluate_for_declaration(value, declaration, &environment, context)?;
            environment.insert(declaration.name.clone(), quantity);
        } else if matches!(
            declaration.initialization,
            Initialization::Guess(_) | Initialization::Computed
        ) {
            return Err(failure(
                "E_MODEL_INITIALIZATION_UNSUPPORTED_CANDIDATE",
                "model_capability_candidate",
                format!(
                    "선언 '{}'의 어림/계산 초기화는 #정밀 풀이방법 범위 밖입니다",
                    declaration.name
                ),
                declaration.identity.span,
                context,
            ));
        }
    }

    for field in &request.input_bindings {
        let declaration = declarations.get(field.name.as_str()).ok_or_else(|| {
            failure(
                "E_MODEL_INPUT_EXTRA_CANDIDATE",
                "model_binding_candidate",
                format!("들임 '{}' 선언이 없습니다", field.name),
                field.span,
                context,
            )
        })?;
        if declaration.role != PrimaryRole::Input || declaration.unknown {
            return Err(failure(
                "E_MODEL_INPUT_AUTHORITY_CANDIDATE",
                "model_binding_candidate",
                format!("'{}'은 들임으로 공급할 수 없습니다", field.name),
                field.span,
                context,
            ));
        }
        let quantity = evaluate_for_declaration(&field.value, declaration, &environment, context)?;
        environment.insert(field.name.clone(), quantity);
    }

    for declaration in definition
        .declarations
        .iter()
        .filter(|declaration| declaration.role == PrimaryRole::Input)
    {
        if !environment.contains_key(&declaration.name) {
            return Err(failure(
                "E_MODEL_INPUT_MISSING_CANDIDATE",
                "model_binding_candidate",
                format!("들임 '{}' 값이 없습니다", declaration.name),
                declaration.identity.span,
                context,
            ));
        }
    }

    for field in &request.initial_overrides {
        let declaration = declarations.get(field.name.as_str()).ok_or_else(|| {
            failure(
                "E_MODEL_INITIAL_OVERRIDE_TARGET_CANDIDATE",
                "model_binding_candidate",
                format!("처음 대상 '{}' 선언이 없습니다", field.name),
                field.span,
                context,
            )
        })?;
        if !declaration.accepts_initial_override || declaration.unknown {
            return Err(failure(
                "E_MODEL_INITIAL_OVERRIDE_UNAUTHORIZED_CANDIDATE",
                "model_binding_candidate",
                format!("선언 '{}'에는 처음받음 권한이 없습니다", field.name),
                field.span,
                context,
            ));
        }
        let quantity = evaluate_for_declaration(&field.value, declaration, &environment, context)?;
        environment.insert(field.name.clone(), quantity);
    }
    Ok(environment)
}

fn solve_relations(
    definition: &ModelDefinition,
    environment: &mut BTreeMap<String, ExactQuantity>,
    context: &ReceiptContext,
) -> Result<Vec<String>, ModelExecutionError> {
    let declarations = definition
        .declarations
        .iter()
        .map(|declaration| (declaration.name.as_str(), declaration))
        .collect::<BTreeMap<_, _>>();
    let mut remaining = BTreeMap::new();
    for relation in &definition.relations {
        context
            .work_cost
            .borrow_mut()
            .charge_element_visits(1)
            .map_err(|error| work_cost_failure(error, relation.identity.span, context))?;
        let Some(lowered) = relation.lowered.as_ref() else {
            return Err(failure(
                "E_MODEL_RELATION_TARGET_INELIGIBLE_CANDIDATE",
                "model_capability_candidate",
                "관계를 direct target 실행 계획으로 낮출 수 없습니다",
                relation.identity.span,
                context,
            ));
        };
        let LoweredRelationTarget::Algebraic(target) = &lowered.target else {
            return Err(failure(
                "E_MODEL_DERIVATIVE_RUNTIME_OPEN_CANDIDATE",
                "model_capability_candidate",
                "미분 관계는 #정밀 풀이방법 범위 밖입니다",
                relation.identity.span,
                context,
            ));
        };
        if remaining.insert(target.clone(), lowered).is_some() {
            return Err(failure(
                "E_MODEL_MULTI_PRODUCER_CANDIDATE",
                "model_structure_candidate",
                format!("target '{target}' producer가 중복되었습니다"),
                relation.identity.span,
                context,
            ));
        }
    }

    let mut order = Vec::new();
    while !remaining.is_empty() {
        let ready = remaining
            .iter()
            .find(|(_, relation)| {
                relation
                    .dependencies
                    .iter()
                    .all(|dependency| environment.contains_key(dependency))
            })
            .map(|(target, _)| target.clone());
        let Some(target) = ready else {
            return Err(failure(
                "E_MODEL_DEPENDENCY_CYCLE_CANDIDATE",
                "model_solve_candidate",
                "남은 관계의 dependency를 결정적으로 만족시킬 수 없습니다",
                definition.span,
                context,
            ));
        };
        let relation = remaining.remove(&target).expect("ready relation");
        context
            .work_cost
            .borrow_mut()
            .charge_solver_evaluations(1)
            .map_err(|error| work_cost_failure(error, relation.provenance, context))?;
        let declaration = declarations.get(target.as_str()).ok_or_else(|| {
            failure(
                "E_MODEL_RELATION_TARGET_UNDECLARED_CANDIDATE",
                "model_solve_candidate",
                format!("target '{target}' 선언이 없습니다"),
                relation.provenance,
                context,
            )
        })?;
        let quantity =
            evaluate_for_declaration(&relation.expression, declaration, environment, context)?;
        environment.insert(target.clone(), quantity);
        order.push(target);
    }
    Ok(order)
}

fn build_result_pack(
    definition: &ModelDefinition,
    request: &RunRequest,
    environment: &BTreeMap<String, ExactQuantity>,
    context: &ReceiptContext,
) -> Result<JsonValue, ModelExecutionError> {
    if request.output_selection.is_empty() {
        return Err(failure(
            "E_MODEL_OUTPUT_SELECTION_REQUIRED_CANDIDATE",
            "model_binding_candidate",
            "내줌=(...)에는 한 개 이상의 이름이 필요합니다",
            request.span,
            context,
        ));
    }
    let mut fields = Vec::new();
    for name in &request.output_selection {
        context
            .work_cost
            .borrow_mut()
            .charge_element_visits(1)
            .map_err(|error| work_cost_failure(error, request.span, context))?;
        let declaration = definition
            .declarations
            .iter()
            .find(|declaration| declaration.name == *name)
            .ok_or_else(|| {
                failure(
                    "E_MODEL_OUTPUT_UNAVAILABLE_CANDIDATE",
                    "model_binding_candidate",
                    format!("output '{name}' 선언이 없습니다"),
                    request.span,
                    context,
                )
            })?;
        if !declaration.exported {
            return Err(failure(
                "E_MODEL_OUTPUT_UNAVAILABLE_CANDIDATE",
                "model_binding_candidate",
                format!("선언 '{name}'에는 ::내줌 권한이 없습니다"),
                declaration.identity.span,
                context,
            ));
        }
        let quantity = environment.get(name).ok_or_else(|| {
            failure(
                "E_MODEL_OUTPUT_VALUE_MISSING_CANDIDATE",
                "model_solve_candidate",
                format!("output '{name}' 값이 계산되지 않았습니다"),
                declaration.identity.span,
                context,
            )
        })?;
        fields.push(quantity_json(name, declaration, quantity, context)?);
    }
    Ok(json!({
        "schema": "ddn.model_result_pack.candidate.v1",
        "ordered_fields": fields,
    }))
}

fn build_public_observations(
    program: &ModelProgram,
    request: &RunRequest,
    result_pack: &JsonValue,
    context: &ReceiptContext,
) -> Result<Vec<JsonValue>, ModelExecutionError> {
    let fields = result_pack["ordered_fields"]
        .as_array()
        .expect("result pack fields");
    let by_name = fields
        .iter()
        .filter_map(|field| Some((field.get("name")?.as_str()?, field)))
        .collect::<BTreeMap<_, _>>();
    let mut observations = Vec::new();
    for view in &program.ir.views {
        for (index, field) in view.fields.iter().enumerate() {
            let path = match &field.value.kind {
                ddonirang_lang::ModelExprKind::Path(path) => path,
                _ => {
                    return Err(failure(
                        "E_MODEL_VIEW_VALUE_PATH_CANDIDATE",
                        "model_view_candidate",
                        "결과표 필드 값은 '실행결과.이름' direct path여야 합니다",
                        field.span,
                        context,
                    ))
                }
            };
            let Some((binding, output)) = path.split_once('.') else {
                return Err(failure(
                    "E_MODEL_VIEW_VALUE_PATH_CANDIDATE",
                    "model_view_candidate",
                    "결과표 필드 값은 '실행결과.이름' direct path여야 합니다",
                    field.span,
                    context,
                ));
            };
            if binding != request.binding_name || output.contains('.') {
                return Err(failure(
                    "E_MODEL_VIEW_BINDING_MISMATCH_CANDIDATE",
                    "model_view_candidate",
                    format!("결과표 path '{path}'가 현재 결과 binding과 다릅니다"),
                    field.span,
                    context,
                ));
            }
            let value = by_name.get(output).ok_or_else(|| {
                failure(
                    "E_MODEL_VIEW_OUTPUT_UNAVAILABLE_CANDIDATE",
                    "model_view_candidate",
                    format!("결과표가 선택되지 않은 output '{output}'을 참조합니다"),
                    field.span,
                    context,
                )
            })?;
            observations.push(json!({
                "starts_observation_event": index == 0,
                "name": field.name,
                "source_ref": format!("source:{}-{}", field.span.start, field.span.end),
                "source_start": field.span.start,
                "source_end": field.span.end,
                "dtype": value["type"],
                "unit": value["unit"],
                "format": "ddn-canonical",
                "value_canon": value["value_canon"],
                "value_text": value["value_text"],
            }));
        }
    }
    Ok(observations)
}

fn evaluate_for_declaration(
    expr: &ModelExpr,
    declaration: &ModelDeclaration,
    environment: &BTreeMap<String, ExactQuantity>,
    context: &ReceiptContext,
) -> Result<ExactQuantity, ModelExecutionError> {
    let kind = numeric_kind(declaration).map_err(|message| {
        failure(
            "E_MODEL_NUMERIC_TYPE_UNSUPPORTED_CANDIDATE",
            "model_arithmetic_candidate",
            message,
            declaration.identity.span,
            context,
        )
    })?;
    let node = expr.node.as_ref().ok_or_else(|| {
        failure(
            "E_MODEL_EXPRESSION_UNSUPPORTED_CANDIDATE",
            "model_arithmetic_candidate",
            format!("typed arithmetic expression이 아닙니다: {}", expr.canonical),
            expr.span,
            context,
        )
    })?;
    let quantity = eval_node(node, environment, Some(kind), expr.span, context)?;
    assign_to_declaration(quantity, declaration, expr.span, context)
}

fn charge_model_exact_parts(
    operands: &[&ExactQuantity],
    results: &[&ExactQuantity],
    span: ModelSpan,
    context: &ReceiptContext,
) -> Result<(), ModelExecutionError> {
    let operand_parts = operands
        .iter()
        .flat_map(|quantity| {
            [
                quantity.base_value.numer().clone(),
                quantity.base_value.denom().clone(),
            ]
        })
        .collect::<Vec<_>>();
    let result_parts = results
        .iter()
        .flat_map(|quantity| {
            [
                quantity.base_value.numer().clone(),
                quantity.base_value.denom().clone(),
            ]
        })
        .collect::<Vec<_>>();
    let operand_refs = operand_parts.iter().collect::<Vec<_>>();
    let result_refs = result_parts.iter().collect::<Vec<_>>();
    context
        .work_cost
        .borrow_mut()
        .charge_bigint_rational_primitive(&operand_refs, &result_refs)
        .map_err(|error| work_cost_failure(error, span, context))
}

fn eval_node(
    node: &ModelExprNode,
    environment: &BTreeMap<String, ExactQuantity>,
    literal_kind: Option<NumericKind>,
    span: ModelSpan,
    context: &ReceiptContext,
) -> Result<ExactQuantity, ModelExecutionError> {
    let _registry_key = model_expr_registry_key(node);
    charge_model_dispatch(context, span)?;
    match node {
        ModelExprNode::Number {
            surface,
            unit,
            dimension,
        } => {
            let numeric_kind = literal_kind.unwrap_or(NumericKind::Exact);
            let authored_value = parse_decimal_rational(surface).map_err(|message| {
                arithmetic_failure(
                    "E_MODEL_EXACT_LITERAL_INVALID_CANDIDATE",
                    message,
                    span,
                    context,
                )
            })?;
            let scale = unit_scale(unit.as_deref()).map_err(|message| {
                arithmetic_failure(
                    "E_MODEL_UNIT_CONVERSION_UNSUPPORTED_CANDIDATE",
                    message,
                    span,
                    context,
                )
            })?;
            let quantity = ExactQuantity {
                base_value: authored_value * scale.clone(),
                dimension: *dimension,
                numeric_kind,
                authored_scale: scale,
                authored_unit: unit.clone(),
            };
            ensure_fixed_exact(&quantity, span, context)?;
            Ok(quantity)
        }
        ModelExprNode::Path(path) => {
            let quantity = environment.get(path).cloned().ok_or_else(|| {
                failure(
                    "E_MODEL_INPUT_MISSING_CANDIDATE",
                    "model_solve_candidate",
                    format!("식이 참조한 값 '{path}'이 아직 준비되지 않았습니다"),
                    span,
                    context,
                )
            })?;
            if let Some(expected) = literal_kind {
                if expected != quantity.numeric_kind {
                    return Err(arithmetic_failure(
                        "E_MODEL_IMPLICIT_NUMERIC_CONVERSION_FORBIDDEN_CANDIDATE",
                        format!("'{path}'에서 exact와 셈수 사이의 암묵 변환을 허용하지 않습니다"),
                        span,
                        context,
                    ));
                }
            }
            Ok(quantity)
        }
        ModelExprNode::Unary { operator, operand } => {
            let mut value = eval_node(operand, environment, literal_kind, span, context)?;
            match operator.as_str() {
                "+" => {}
                "-" => value.base_value = -value.base_value,
                _ => {
                    return Err(arithmetic_failure(
                        "E_MODEL_OPERATOR_UNSUPPORTED_CANDIDATE",
                        format!("단항 연산자 '{operator}'는 지원하지 않습니다"),
                        span,
                        context,
                    ))
                }
            }
            ensure_fixed_exact(&value, span, context)?;
            Ok(value)
        }
        ModelExprNode::Group(inner) => eval_node(inner, environment, literal_kind, span, context),
        ModelExprNode::Binary {
            operator,
            left,
            right,
        } => {
            let left = eval_node(left, environment, literal_kind, span, context)?;
            let right = eval_node(right, environment, literal_kind, span, context)?;
            if left.numeric_kind != right.numeric_kind {
                return Err(arithmetic_failure(
                    "E_MODEL_IMPLICIT_NUMERIC_CONVERSION_FORBIDDEN_CANDIDATE",
                    "exact와 셈수 사이의 암묵 변환을 허용하지 않습니다",
                    span,
                    context,
                ));
            }
            if left.numeric_kind == NumericKind::Exact {
                charge_model_exact_parts(&[&left, &right], &[], span, context)?;
            }
            let mut result = match operator.as_str() {
                "+" | "-" => {
                    if left.dimension != right.dimension {
                        return Err(arithmetic_failure(
                            "E_MODEL_UNIT_MISMATCH_CANDIDATE",
                            "덧셈·뺄셈 양쪽 단위 차원이 다릅니다",
                            span,
                            context,
                        ));
                    }
                    ExactQuantity {
                        base_value: if operator == "+" {
                            left.base_value + right.base_value
                        } else {
                            left.base_value - right.base_value
                        },
                        dimension: left.dimension,
                        numeric_kind: left.numeric_kind,
                        authored_scale: left.authored_scale,
                        authored_unit: left.authored_unit,
                    }
                }
                "*" => ExactQuantity {
                    base_value: left.base_value * right.base_value,
                    dimension: left.dimension.add(right.dimension),
                    numeric_kind: left.numeric_kind,
                    authored_scale: left.authored_scale * right.authored_scale,
                    authored_unit: combine_units(
                        left.authored_unit.as_deref(),
                        right.authored_unit.as_deref(),
                        "*",
                    ),
                },
                "/" => {
                    if right.base_value.is_zero() {
                        return Err(arithmetic_failure(
                            "E_MODEL_DIVISION_BY_ZERO_CANDIDATE",
                            "0으로 나눌 수 없습니다",
                            span,
                            context,
                        ));
                    }
                    ExactQuantity {
                        base_value: left.base_value / right.base_value,
                        dimension: left.dimension.sub(right.dimension),
                        numeric_kind: left.numeric_kind,
                        authored_scale: left.authored_scale / right.authored_scale,
                        authored_unit: combine_units(
                            left.authored_unit.as_deref(),
                            right.authored_unit.as_deref(),
                            "/",
                        ),
                    }
                }
                "^" => {
                    let exponent = exact_integer(&right).ok_or_else(|| {
                        arithmetic_failure(
                            "E_MODEL_EXPONENT_UNSUPPORTED_CANDIDATE",
                            "지수는 단위 없는 exact 정수여야 합니다",
                            span,
                            context,
                        )
                    })?;
                    if !right.dimension.eq(&UnitDim::NONE) || !(-16..=16).contains(&exponent) {
                        return Err(arithmetic_failure(
                            "E_MODEL_EXPONENT_UNSUPPORTED_CANDIDATE",
                            "지수는 -16..16 범위의 단위 없는 정수여야 합니다",
                            span,
                            context,
                        ));
                    }
                    pow_quantity(left, exponent, span, context)?
                }
                _ => {
                    return Err(arithmetic_failure(
                        "E_MODEL_OPERATOR_UNSUPPORTED_CANDIDATE",
                        format!("연산자 '{operator}'는 #정밀 풀이방법에서 지원하지 않습니다"),
                        span,
                        context,
                    ))
                }
            };
            if result.authored_scale.is_zero() {
                result.authored_scale = BigRational::one();
            }
            ensure_fixed_exact(&result, span, context)?;
            if result.numeric_kind == NumericKind::Exact {
                charge_model_exact_parts(&[], &[&result], span, context)?;
            }
            Ok(result)
        }
        ModelExprNode::String(_) | ModelExprNode::Atom(_) | ModelExprNode::Derivative(_) => {
            Err(arithmetic_failure(
                "E_MODEL_EXPRESSION_UNSUPPORTED_CANDIDATE",
                "문자열·원자·미분은 exact arithmetic 식으로 계산하지 않습니다",
                span,
                context,
            ))
        }
    }
}

fn assign_to_declaration(
    mut quantity: ExactQuantity,
    declaration: &ModelDeclaration,
    span: ModelSpan,
    context: &ReceiptContext,
) -> Result<ExactQuantity, ModelExecutionError> {
    let dimension = declaration.model_type.dimension.unwrap_or(UnitDim::NONE);
    if quantity.dimension != dimension {
        return Err(arithmetic_failure(
            "E_MODEL_UNIT_MISMATCH_CANDIDATE",
            format!("'{}' 선언 단위와 계산값 차원이 다릅니다", declaration.name),
            span,
            context,
        ));
    }
    let expected_kind = numeric_kind(declaration).map_err(|message| {
        arithmetic_failure(
            "E_MODEL_NUMERIC_TYPE_UNSUPPORTED_CANDIDATE",
            message,
            span,
            context,
        )
    })?;
    if quantity.numeric_kind != expected_kind {
        return Err(arithmetic_failure(
            "E_MODEL_IMPLICIT_NUMERIC_CONVERSION_FORBIDDEN_CANDIDATE",
            "exact와 셈수 사이의 암묵 변환을 허용하지 않습니다",
            span,
            context,
        ));
    }
    quantity.authored_scale =
        unit_scale(declaration.model_type.unit.as_deref()).map_err(|message| {
            arithmetic_failure(
                "E_MODEL_UNIT_CONVERSION_UNSUPPORTED_CANDIDATE",
                message,
                span,
                context,
            )
        })?;
    quantity.authored_unit = declaration.model_type.unit.clone();
    ensure_fixed_exact(&quantity, span, context)?;
    validate_exact_type_value(declaration, &quantity, span, context)?;
    Ok(quantity)
}

fn numeric_kind(declaration: &ModelDeclaration) -> Result<NumericKind, String> {
    match declaration.model_type.name.as_str() {
        "수" | "셈수" => Ok(NumericKind::FixedQ32),
        "바른수" | "큰바른수" | "나눔수" => Ok(NumericKind::Exact),
        other => Err(format!("수 갈래 '{other}'는 #정밀 풀이방법 범위 밖입니다")),
    }
}

fn validate_exact_type_value(
    declaration: &ModelDeclaration,
    quantity: &ExactQuantity,
    span: ModelSpan,
    context: &ReceiptContext,
) -> Result<(), ModelExecutionError> {
    let authored = &quantity.base_value / &quantity.authored_scale;
    if matches!(declaration.model_type.name.as_str(), "바른수" | "큰바른수")
        && (!authored.denom().is_one() || authored.is_negative())
    {
        return Err(arithmetic_failure(
            "E_MODEL_EXACT_TYPE_DOMAIN_CANDIDATE",
            format!(
                "'{}'에는 0 이상의 exact 정수만 넣을 수 있습니다",
                declaration.name
            ),
            span,
            context,
        ));
    }
    Ok(())
}

fn ensure_fixed_exact(
    quantity: &ExactQuantity,
    span: ModelSpan,
    context: &ReceiptContext,
) -> Result<(), ModelExecutionError> {
    if quantity.numeric_kind == NumericKind::Exact {
        return Ok(());
    }
    // Runtime arithmetic is performed in canonical base units. Checking only
    // the authored scalar could hide an inexact unit conversion (for example
    // 1@cm), so every intermediate canonical value must fit Q32.32 exactly.
    let scaled = quantity.base_value.clone() * BigInt::from(Fixed64::ONE_RAW);
    if !scaled.denom().is_one() || scaled.numer().to_i64().is_none() {
        return Err(arithmetic_failure(
            "E_MODEL_EXACT_VALUE_UNREPRESENTABLE_CANDIDATE",
            "셈수 연산 결과를 Q32.32에 수학적으로 정확히 표현할 수 없습니다",
            span,
            context,
        ));
    }
    Ok(())
}

fn quantity_json(
    name: &str,
    declaration: &ModelDeclaration,
    quantity: &ExactQuantity,
    context: &ReceiptContext,
) -> Result<JsonValue, ModelExecutionError> {
    let authored = &quantity.base_value / &quantity.authored_scale;
    let number = rational_canon(&authored);
    let suffix = declaration
        .model_type
        .unit
        .as_ref()
        .map(|unit| format!("@{unit}"))
        .unwrap_or_default();
    let value_canon = format!("{number}{suffix}");
    let q32_raw = if quantity.numeric_kind == NumericKind::FixedQ32 {
        let scaled = authored.clone() * BigInt::from(Fixed64::ONE_RAW);
        Some(scaled.numer().to_i64().ok_or_else(|| {
            arithmetic_failure(
                "E_MODEL_ARITHMETIC_OVERFLOW_CANDIDATE",
                "셈수 raw 값 범위를 벗어났습니다",
                declaration.identity.span,
                context,
            )
        })?)
    } else {
        None
    };
    Ok(json!({
        "name": name,
        "type": declaration.model_type.name,
        "unit": declaration.model_type.unit.clone().unwrap_or_default(),
        "dimension": declaration.model_type.dimension.unwrap_or(UnitDim::NONE).exponents(),
        "numeric_kind": match quantity.numeric_kind {
            NumericKind::Exact => "exact",
            NumericKind::FixedQ32 => "fixed_q32_32_exact",
        },
        "numerator": authored.numer().to_string(),
        "denominator": authored.denom().to_string(),
        "q32_raw": q32_raw,
        "value_canon": value_canon,
        "value_text": value_canon,
    }))
}

fn exact_integer(quantity: &ExactQuantity) -> Option<i32> {
    if quantity.dimension != UnitDim::NONE {
        return None;
    }
    let value = &quantity.base_value / &quantity.authored_scale;
    value
        .denom()
        .is_one()
        .then(|| value.numer().to_i32())
        .flatten()
}

fn pow_quantity(
    quantity: ExactQuantity,
    exponent: i32,
    span: ModelSpan,
    context: &ReceiptContext,
) -> Result<ExactQuantity, ModelExecutionError> {
    if exponent == 0 {
        return Ok(ExactQuantity {
            base_value: BigRational::one(),
            dimension: UnitDim::NONE,
            numeric_kind: quantity.numeric_kind,
            authored_scale: BigRational::one(),
            authored_unit: None,
        });
    }
    if exponent < 0 && quantity.base_value.is_zero() {
        return Err(arithmetic_failure(
            "E_MODEL_DIVISION_BY_ZERO_CANDIDATE",
            "0의 음수 지수는 계산할 수 없습니다",
            span,
            context,
        ));
    }
    let magnitude = exponent.unsigned_abs();
    let (base_value, authored_scale) = if exponent > 0 {
        (
            quantity.base_value.pow(magnitude as i32),
            quantity.authored_scale.pow(magnitude as i32),
        )
    } else {
        (
            BigRational::one() / quantity.base_value.pow(magnitude as i32),
            BigRational::one() / quantity.authored_scale.pow(magnitude as i32),
        )
    };
    let mut exponents = [0i8; UnitDim::AXIS_COUNT];
    for (index, value) in quantity.dimension.exponents().into_iter().enumerate() {
        exponents[index] = value.checked_mul(exponent as i8).ok_or_else(|| {
            arithmetic_failure(
                "E_MODEL_UNIT_EXPONENT_OVERFLOW_CANDIDATE",
                "단위 지수 범위를 벗어났습니다",
                span,
                context,
            )
        })?;
    }
    Ok(ExactQuantity {
        base_value,
        dimension: UnitDim::from_exponents(exponents),
        numeric_kind: quantity.numeric_kind,
        authored_scale,
        authored_unit: quantity
            .authored_unit
            .map(|unit| format!("{unit}^{exponent}")),
    })
}

fn parse_decimal_rational(surface: &str) -> Result<BigRational, String> {
    let text = surface.trim();
    if text.is_empty() {
        return Err("빈 수 리터럴입니다".to_string());
    }
    let (mantissa, exponent) = match text.find(['e', 'E']) {
        Some(index) => {
            let exponent = text[index + 1..]
                .parse::<i32>()
                .map_err(|_| format!("잘못된 지수 표기: {surface}"))?;
            (&text[..index], exponent)
        }
        None => (text, 0),
    };
    let negative = mantissa.starts_with('-');
    let unsigned = mantissa.strip_prefix(['-', '+']).unwrap_or(mantissa);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty() && fraction.is_empty() {
        return Err(format!("잘못된 수 리터럴: {surface}"));
    }
    let digits = format!("{}{}", if whole.is_empty() { "0" } else { whole }, fraction);
    if !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(format!("잘못된 수 리터럴: {surface}"));
    }
    let mut numerator = BigInt::parse_bytes(digits.as_bytes(), 10)
        .ok_or_else(|| format!("수를 읽을 수 없습니다: {surface}"))?;
    if negative {
        numerator = -numerator;
    }
    let decimal_power = fraction.len() as i32 - exponent;
    if decimal_power >= 0 {
        Ok(BigRational::new(
            numerator,
            BigInt::from(10u8).pow(decimal_power as u32),
        ))
    } else {
        Ok(BigRational::from_integer(
            numerator * BigInt::from(10u8).pow((-decimal_power) as u32),
        ))
    }
}

fn unit_scale(unit: Option<&str>) -> Result<BigRational, String> {
    let Some(surface) = unit else {
        return Ok(BigRational::one());
    };
    if unit_spec_from_symbol(surface).is_some() {
        return exact_unit_scale_atom(surface);
    }
    let chars = surface.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut divide = false;
    let mut scale = BigRational::one();
    while index < chars.len() {
        if chars[index] == '*' {
            divide = false;
            index += 1;
            continue;
        }
        if chars[index] == '/' {
            divide = true;
            index += 1;
            continue;
        }
        let start = index;
        while index < chars.len() && (chars[index].is_alphanumeric() || chars[index] == '_') {
            index += 1;
        }
        if start == index {
            return Err(format!("단위 표면을 읽을 수 없습니다: {surface}"));
        }
        let symbol = chars[start..index].iter().collect::<String>();
        unit_spec_from_symbol(&symbol).ok_or_else(|| format!("알 수 없는 단위: {symbol}"))?;
        let mut exponent = 1u32;
        if index < chars.len() && chars[index] == '^' {
            index += 1;
            let exponent_start = index;
            while index < chars.len() && chars[index].is_ascii_digit() {
                index += 1;
            }
            exponent = chars[exponent_start..index]
                .iter()
                .collect::<String>()
                .parse::<u32>()
                .map_err(|_| format!("잘못된 단위 지수: {surface}"))?;
        }
        let factor = exact_unit_scale_atom(&symbol)?.pow(exponent as i32);
        scale = if divide {
            scale / factor
        } else {
            scale * factor
        };
    }
    Ok(scale)
}

fn exact_unit_scale_atom(symbol: &str) -> Result<BigRational, String> {
    let ratio = |numerator: i64, denominator: i64| {
        BigRational::new(BigInt::from(numerator), BigInt::from(denominator))
    };
    match symbol {
        "m" | "s" | "초" | "kg" | "rad" | "px" | "K" | "m/s" | "mps" | "m/s^2" | "N" | "KRW"
        | "USD" | "mol" => Ok(BigRational::one()),
        "mm" => Ok(ratio(1, 1_000)),
        "cm" => Ok(ratio(1, 100)),
        "km" => Ok(BigRational::from_integer(BigInt::from(1_000))),
        "inch" => Ok(ratio(127, 5_000)),
        "ft" => Ok(ratio(381, 1_250)),
        "평" => Ok(ratio(3_305_785, 1_000_000)),
        "us" => Ok(ratio(1, 1_000_000)),
        "ms" | "g" => Ok(ratio(1, 1_000)),
        "min" => Ok(BigRational::from_integer(BigInt::from(60))),
        "h" => Ok(BigRational::from_integer(BigInt::from(3_600))),
        "kmh" => Ok(ratio(5, 18)),
        "C" | "F" => Err(format!("affine 단위 '{symbol}' 변환은 OPEN입니다")),
        _ => Err(format!(
            "단위 '{symbol}'의 정밀 환산계수는 #정밀 풀이방법에 봉인되지 않았습니다"
        )),
    }
}

fn combine_units(left: Option<&str>, right: Option<&str>, operator: &str) -> Option<String> {
    match (left, right, operator) {
        (None, None, _) => None,
        (Some(left), None, _) => Some(left.to_string()),
        (None, Some(right), "*") => Some(right.to_string()),
        (None, Some(right), "/") => Some(format!("1/{right}")),
        (Some(left), Some(right), operator) => Some(format!("{left}{operator}{right}")),
        _ => None,
    }
}

fn rational_canon(value: &BigRational) -> String {
    if value.denom().is_one() {
        value.numer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

fn definition_semantic_hash(definition: &ModelDefinition) -> String {
    let mut declarations = definition
        .declarations
        .iter()
        .map(|declaration| {
            json!({
                "name": declaration.name,
                "type": declaration.model_type.name,
                "unit": declaration.model_type.unit,
                "role": format!("{:?}", declaration.role),
                "unknown": declaration.unknown,
                "exported": declaration.exported,
            })
        })
        .collect::<Vec<_>>();
    declarations.sort_by_key(|value| value["name"].as_str().unwrap_or_default().to_string());
    let mut relations = definition
        .relations
        .iter()
        .map(|relation| relation.semantic_identity.clone())
        .collect::<Vec<_>>();
    relations.sort();
    hash_json(&json!({
        "name": definition.name,
        "declarations": declarations,
        "relations": relations,
    }))
}

fn input_snapshot_hash(request: &RunRequest) -> String {
    let mut fields = request
        .input_bindings
        .iter()
        .map(|field| (field.name.clone(), field.value.canonical.clone()))
        .collect::<Vec<_>>();
    fields.sort();
    hash_json(&json!({"inputs": fields}))
}

fn unit_registry_hash(definition: &ModelDefinition) -> Result<String, ModelExecutionError> {
    let context = ReceiptContext::default();
    let mut units = BTreeSet::new();
    for declaration in &definition.declarations {
        if let Some(unit) = declaration.model_type.unit.as_deref() {
            let scale = unit_scale(Some(unit)).map_err(|message| {
                arithmetic_failure(
                    "E_MODEL_UNIT_CONVERSION_UNSUPPORTED_CANDIDATE",
                    message,
                    declaration.identity.span,
                    &context,
                )
            })?;
            units.insert((
                unit.to_string(),
                declaration
                    .model_type
                    .dimension
                    .unwrap_or(UnitDim::NONE)
                    .exponents(),
                scale.numer().to_string(),
                scale.denom().to_string(),
            ));
        }
    }
    Ok(hash_json(&json!(units)))
}

pub(crate) fn empty_state_hash() -> String {
    hash_json(&json!({
        "schema": "ddn.model_committed_state.candidate.v1",
        "bindings": []
    }))
}

fn work_cost_failure(
    error: WorkCostFailure,
    provenance: ModelSpan,
    context: &ReceiptContext,
) -> ModelExecutionError {
    failure(
        error.code,
        "model_work_cost_candidate",
        error.to_string(),
        provenance,
        context,
    )
}

fn charge_model_dispatch(
    context: &ReceiptContext,
    provenance: ModelSpan,
) -> Result<(), ModelExecutionError> {
    context
        .work_cost
        .borrow_mut()
        .charge_dispatch()
        .map_err(|error| work_cost_failure(error, provenance, context))
}

fn success_receipt(
    context: &ReceiptContext,
    pre_state_hash: &str,
    post_state_hash: &str,
    result_pack_hash: &str,
) -> JsonValue {
    seal_receipt(json!({
        "schema": MODEL_RECEIPT_SCHEMA,
        "wire_status": "candidate_non_public",
        "status": "success",
        "backend_id": MODEL_BACKEND_ID,
        "backend_version": 1,
        "backend_descriptor_hash": backend_descriptor_json()["descriptor_hash"],
        "backend_capability_set_hash": backend_capability_set_hash(),
        "runtime_build_identity": runtime_build_identity_json(),
        "program_source_hash": context.program_source_hash,
        "canonical_program_hash": context.canonical_program_hash,
        "solve_method": solve_method_receipt_json(context),
        "execution_contract": execution_contract_receipt_json(context),
        "model_definition_semantic_hash": context.definition_semantic_hash,
        "model_definition_source_hash": context.definition_source_hash,
        "model_instance_identity": context.instance_identity,
        "model_instance_binding_hash": context.instance_binding_hash,
        "solve_recipe_identity": context.recipe_identity,
        "solve_recipe_hash": context.recipe_hash,
        "run_request_identity": context.request_identity,
        "run_request_hash": context.request_hash,
        "parent_committed_state_hash": pre_state_hash,
        "input_snapshot_hash": context.input_snapshot_hash,
        "unit_registry_hash": context.unit_registry_hash,
        "arithmetic_policy_id": MODEL_ARITHMETIC_POLICY_ID,
        "dependency_order_hash": context.dependency_order_hash,
        "requested_output_order": context.requested_output_order,
        "work_cost": context.work_cost.borrow().receipt(),
        "result_pack_hash": result_pack_hash,
        "diagnostics": [],
        "atomic_commit_outcome": "committed",
        "post_state_hash": post_state_hash,
    }))
}

fn failure(
    candidate_code: &'static str,
    phase: &'static str,
    message: impl Into<String>,
    provenance: ModelSpan,
    context: &ReceiptContext,
) -> ModelExecutionError {
    let pre_state_hash = empty_state_hash();
    let message = message.into();
    let (cause, offending_field, expected, actual) =
        execution_contract_diagnostic_details(candidate_code, &message);
    let receipt = seal_receipt(json!({
        "schema": MODEL_RECEIPT_SCHEMA,
        "wire_status": "candidate_non_public",
        "status": "failure",
        "backend_id": MODEL_BACKEND_ID,
        "backend_version": 1,
        "backend_descriptor_hash": backend_descriptor_json()["descriptor_hash"],
        "backend_capability_set_hash": backend_capability_set_hash(),
        "runtime_build_identity": runtime_build_identity_json(),
        "program_source_hash": context.program_source_hash,
        "canonical_program_hash": context.canonical_program_hash,
        "solve_method": solve_method_receipt_json(context),
        "execution_contract": execution_contract_receipt_json(context),
        "model_definition_semantic_hash": context.definition_semantic_hash,
        "model_definition_source_hash": context.definition_source_hash,
        "model_instance_identity": context.instance_identity,
        "model_instance_binding_hash": context.instance_binding_hash,
        "solve_recipe_identity": context.recipe_identity,
        "solve_recipe_hash": context.recipe_hash,
        "run_request_identity": context.request_identity,
        "run_request_hash": context.request_hash,
        "parent_committed_state_hash": pre_state_hash,
        "input_snapshot_hash": context.input_snapshot_hash,
        "unit_registry_hash": context.unit_registry_hash,
        "arithmetic_policy_id": MODEL_ARITHMETIC_POLICY_ID,
        "dependency_order_hash": context.dependency_order_hash,
        "requested_output_order": context.requested_output_order,
        "work_cost": context.work_cost.borrow().receipt(),
        "result_pack_hash": JsonValue::Null,
        "diagnostics": [{
            "candidate_code": candidate_code,
            "phase": phase,
            "message": message,
            "provenance": [provenance.start, provenance.end],
        }],
        "atomic_commit_outcome": "not_committed",
        "post_state_hash": pre_state_hash,
    }));
    ModelExecutionError {
        candidate_code,
        phase,
        message,
        source_path: context.source_path.clone(),
        provenance,
        cause,
        offending_field,
        expected,
        actual,
        pre_state_hash: pre_state_hash.clone(),
        post_state_hash: pre_state_hash,
        receipt,
    }
}

fn execution_contract_diagnostic_details(
    candidate_code: &'static str,
    message: &str,
) -> (
    Option<JsonValue>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    if !candidate_code.starts_with("E_MODEL_EXECUTION_CONTRACT_") {
        return (None, None, None, None);
    }
    let quoted = message.split('\'').nth(1).map(str::to_string);
    let actual_atom = message
        .split("실제 #")
        .nth(1)
        .map(str::trim)
        .map(str::to_string);
    let approved_fields = "결과,성능,외부유입,자원,재생,재현범위,증거";
    let approved_values =
        "#정본값동일,#안전최적화,#봉인입력,#표준한도_v1,#검증재생,#이식가능,#최소증거";
    let (offending_field, expected, actual) = match candidate_code {
        "E_MODEL_EXECUTION_CONTRACT_FIELD_DELIMITER_CANDIDATE" => {
            (None, Some(":".to_string()), Some("= 또는 누락".to_string()))
        }
        "E_MODEL_EXECUTION_CONTRACT_FIELD_DUPLICATE_CANDIDATE" => (
            quoted,
            Some("필드별 정확히 1회".to_string()),
            Some("중복".to_string()),
        ),
        "E_MODEL_EXECUTION_CONTRACT_FIELD_UNKNOWN_CANDIDATE" => (
            message
                .split(": ")
                .nth(1)
                .map(str::trim)
                .map(str::to_string),
            Some(approved_fields.to_string()),
            message
                .split(": ")
                .nth(1)
                .map(str::trim)
                .map(str::to_string),
        ),
        "E_MODEL_EXECUTION_CONTRACT_FIELD_MISSING_CANDIDATE" => (
            message
                .split(": ")
                .nth(1)
                .map(str::trim)
                .map(str::to_string),
            Some(approved_fields.to_string()),
            Some("부분 필드".to_string()),
        ),
        "E_MODEL_EXECUTION_CONTRACT_VALUE_UNKNOWN_CANDIDATE" => (
            quoted,
            Some(approved_values.to_string()),
            actual_atom.or_else(|| Some("profile atom shorthand 또는 비승인 값".to_string())),
        ),
        "E_MODEL_EXECUTION_CONTRACT_COMBINATION_UNSUPPORTED_CANDIDATE" => (
            quoted,
            message
                .split("에는 #")
                .nth(1)
                .and_then(|tail| tail.split_whitespace().next())
                .map(|value| format!("#{value}")),
            message
                .split("#")
                .last()
                .and_then(|tail| tail.split_whitespace().next())
                .map(|value| format!("#{value}")),
        ),
        "E_MODEL_EXECUTION_CONTRACT_CAPABILITY_UNSUPPORTED_CANDIDATE" => (
            quoted,
            Some("runtime_success_ready=true".to_string()),
            Some("runtime_success_ready=false".to_string()),
        ),
        _ => (None, None, None),
    };
    (
        Some(json!({
            "code": "execution_contract_validation_candidate",
            "message": message,
        })),
        offending_field,
        expected,
        actual,
    )
}

fn solve_method_receipt_json(context: &ReceiptContext) -> JsonValue {
    json!({
        "binding_kind": context.solve_method_binding_kind,
        "authored": context.solve_method_authored,
        "source_file": context.solve_method_source_file,
        "source_range": context.solve_method_source_range,
        "public_surface": context.public_solve_method,
        "resolution_policy": "fixed_precise_no_discovery_no_fallback",
        "backend_options": {},
        "backend_limits": {
            "accepted_model_class": "pure_deterministic_acyclic_direct_target_relation",
            "numeric_policy": MODEL_ARITHMETIC_POLICY_ID,
        },
        "backend_registry": {
            "lookup": "fixed_public_method_mapping",
            "registration_order_observed": false,
            "installed_backend_count_observed": false,
        },
        "compatibility": {
            "fixed_default_contract": "ssot.v24.28.2.rd153-155",
            "historical_exact_descriptor_explicit_only": true,
        },
        "retry": false,
        "fallback": false,
    })
}

fn execution_contract_receipt_json(context: &ReceiptContext) -> JsonValue {
    let runtime_activation_authorized = context
        .execution_contract_capability_matrix
        .as_ref()
        .and_then(|matrix| matrix["runtime_activation_authorized"].as_bool())
        .unwrap_or(false);
    json!({
        "schema": ddonirang_lang::EXECUTION_CONTRACT_SCHEMA_CANDIDATE,
        "wire_status": "candidate_non_public",
        "profile_identity": context.execution_contract_profile_identity,
        "typed_input_component_hash": context.execution_contract_typed_input_component_hash,
        "authored": context.execution_contract_authored,
        "source_file": context.execution_contract_source_file,
        "source_range": context.execution_contract_source_range,
        "capability_matrix": context.execution_contract_capability_matrix,
        "runtime_activation_authorized": runtime_activation_authorized,
        "state_hash_participation": false,
        "silent_fallback": false,
    })
}

fn arithmetic_failure(
    candidate_code: &'static str,
    message: impl Into<String>,
    provenance: ModelSpan,
    context: &ReceiptContext,
) -> ModelExecutionError {
    failure(
        candidate_code,
        "model_arithmetic_candidate",
        message,
        provenance,
        context,
    )
}

fn seal_receipt(mut receipt: JsonValue) -> JsonValue {
    let hash = hash_json(&receipt);
    receipt
        .as_object_mut()
        .expect("receipt object")
        .insert("receipt_hash".to_string(), json!(hash));
    receipt
}

fn hash_json(value: &JsonValue) -> String {
    hash_bytes(
        serde_json::to_string(value)
            .expect("candidate JSON serialization")
            .as_bytes(),
    )
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUCCESS: &str = r#"
정밀한방법 <- (방식: #정밀).
계산 = 모형 {
  입력:나눔수@m :: (들임, 내줌).
  중간:나눔수@m :: (그자리, 내줌).
  결과:나눔수@m :: (그자리, 내줌).
  중간::구함.
  결과::구함.
  입력 + 1@m =:= 중간.
  결과 =:= 중간 * 2.
}.
실행결과 <- (대상=계산, 들임=(입력: 2@m), 방법=정밀한방법, 내줌=(결과, 입력, 중간)) 풀기.
(계산값: 실행결과.결과, 입력값: 실행결과.입력) 보임.
"#;

    fn explicit_standard_contract(source: &str) -> String {
        source.replace(
            "방법=정밀한방법, 내줌=",
            "실행계약=(자원: #표준한도_v1, 재생: #검증재생, 증거: #최소증거, 외부유입: #봉인입력, 성능: #안전최적화, 재현범위: #이식가능, 결과: #정본값동일), 방법=정밀한방법, 내줌=",
        )
    }

    #[test]
    fn execution_contract_envelope_preserves_surface_and_all_capabilities() {
        let omitted = execution_envelope_json(SUCCESS).expect("omitted contract envelope");
        let explicit_source = explicit_standard_contract(SUCCESS);
        let explicit =
            execution_envelope_json(&explicit_source).expect("explicit contract envelope");
        assert_eq!(
            omitted["canonical_ddn_hash"],
            explicit["canonical_ddn_hash"]
        );
        assert_eq!(
            omitted["execution_contract"]["typed_input_component_hash"],
            explicit["execution_contract"]["typed_input_component_hash"]
        );
        assert_eq!(
            omitted["execution_contract"]["profile_identity"],
            ddonirang_lang::STANDARD_REPRO_PROFILE_IDENTITY
        );
        let matrix = &omitted["execution_contract_capability_matrix"];
        assert_eq!(matrix["runtime_activation_authorized"], true);
        assert_eq!(matrix["all_required_capabilities_satisfied"], true);
        assert_eq!(matrix["silent_fallback"], false);
        assert_eq!(matrix["blocking_fields"], json!([]));
    }

    #[test]
    fn omitted_and_explicit_standard_contract_execute_with_verified_replay() {
        let mut reports = Vec::new();
        for (source, authored) in [
            (SUCCESS.to_string(), false),
            (explicit_standard_contract(SUCCESS), true),
        ] {
            let report = execute_source_with_source_file(&source, "계약.ddn")
                .expect("standard profile runtime activation");
            assert_eq!(report["status"], "completed");
            assert_eq!(
                report["receipt"]["execution_contract"]["profile_identity"],
                ddonirang_lang::STANDARD_REPRO_PROFILE_IDENTITY
            );
            assert_eq!(
                report["receipt"]["execution_contract"]["authored"],
                authored
            );
            assert_eq!(
                report["receipt"]["execution_contract"]["runtime_activation_authorized"],
                true
            );
            assert_eq!(
                report["receipt"]["execution_contract"]["state_hash_participation"],
                false
            );
            assert_eq!(
                report["receipt"]["work_cost"]["counters"]["sealed_input_bytes"],
                source.len()
            );
            assert!(
                report["receipt"]["work_cost"]["counters"]["base_dispatches"]
                    .as_u64()
                    .is_some_and(|count| count > 1)
            );
            assert!(
                report["receipt"]["work_cost"]["counters"]["replay_capsule_bytes"]
                    .as_u64()
                    .is_some_and(|count| count > 0)
            );
            assert_eq!(
                report["receipt"]["work_cost"]["rd206_resource_capability"],
                true
            );
            assert_eq!(
                report["verified_replay"]["capsule"]["sealed_identical_input"],
                true
            );
            assert_eq!(
                report["verified_replay"]["capsule"]["result_and_identity_verified"],
                true
            );
            assert_eq!(
                report["verified_replay"]["capsule"]["continuation_present"],
                false
            );
            assert_eq!(verify_execution_report(&report)["ok"], true);
            reports.push(report);
        }
        assert_eq!(reports[0]["canonical_ddn"], reports[1]["canonical_ddn"]);
        assert_eq!(reports[0]["result_pack"], reports[1]["result_pack"]);
        assert_eq!(reports[0]["post_state_hash"], reports[1]["post_state_hash"]);
        assert_eq!(
            reports[0]["receipt"]["execution_contract"]["typed_input_component_hash"],
            reports[1]["receipt"]["execution_contract"]["typed_input_component_hash"]
        );
    }

    #[test]
    fn bounded_model_internal_plan_uses_model_node_ledger_and_solver_counter() {
        let program = ddonirang_lang::parse_model_program(SUCCESS).expect("model parse");
        let request = &program.ir.requests[0];
        let definition = program
            .ir
            .definitions
            .iter()
            .find(|definition| definition.name == "계산")
            .expect("definition");
        let context = ReceiptContext::default();
        let mut environment =
            prepare_environment(&program, definition, request, &context).expect("environment");
        let order = solve_relations(definition, &mut environment, &context).expect("solve");
        assert_eq!(order, vec!["중간".to_string(), "결과".to_string()]);

        let receipt = context.work_cost.borrow().receipt();
        assert_eq!(receipt.counters.solver_evaluations, 2);
        assert_eq!(receipt.counters.collection_tensor_element_visits, 2);
        assert!(receipt.counters.base_dispatches > 0);
        assert!(
            receipt
                .counters
                .bigint_rational_signed_magnitude_byte_visits
                > 0
        );
        assert!(!receipt.rd206_resource_capability);
    }

    #[test]
    fn exact_backend_executes_right_or_left_targets_and_preserves_output_order() {
        let report = execute_source(SUCCESS).expect("exact run");
        assert_eq!(report["status"], "completed");
        assert_eq!(report["dependency_order"], json!(["중간", "결과"]));
        let fields = report["result_pack"]["ordered_fields"]
            .as_array()
            .expect("fields");
        assert_eq!(fields[0]["name"], "결과");
        assert_eq!(fields[0]["value_canon"], "6@m");
        assert_eq!(fields[1]["name"], "입력");
        assert_eq!(fields[2]["name"], "중간");
        assert_eq!(report["public_observations"][0]["name"], "계산값");
        assert_ne!(report["pre_state_hash"], report["post_state_hash"]);
    }

    #[test]
    fn exact_prelude_bindings_feed_the_single_run_in_source_order() {
        let source = r#"
입력자료 <- 5.
배수자료 <- 입력자료 - 3.
관계계산 = 모형 {
  입력값:나눔수 :: (들임, 내줌).
  배수:나눔수 :: (들임, 내줌).
  결과값:나눔수 :: (그자리, 내줌).
  결과값::구함.
  결과값 =:= 입력값 * 배수.
}.
계산 <- (대상=관계계산, 들임=(입력값: 입력자료, 배수: 배수자료), 내줌=(결과값, 입력값, 배수)) 풀기.
"#;
        let report = execute_source(source).expect("prelude-backed exact run");
        assert_eq!(report["status"], "completed");
        assert_eq!(
            report["result_pack"]["ordered_fields"][0]["value_canon"],
            "10"
        );
        assert_eq!(
            report["execution_bindings"]["model_ir_semantic_hash"],
            report["model_ir_semantic_hash"]
        );
    }

    #[test]
    fn omitted_method_uses_same_exact_backend_and_seals_distinct_provenance() {
        let omitted = SUCCESS
            .replace("정밀한방법 <- (방식: #정밀).\n", "")
            .replace(", 방법=정밀한방법", "");
        let explicit = execute_source(SUCCESS).expect("explicit run");
        let omitted = execute_source_with_source_file(&omitted, "entry.ddn")
            .expect("omitted fixed precise run");
        for field in [
            "backend_descriptor",
            "dependency_order",
            "result_pack",
            "post_state_hash",
            "committed_state",
        ] {
            assert_eq!(omitted[field], explicit[field], "field={field}");
        }
        let observation_values = |report: &JsonValue| {
            report["public_observations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| (row["name"].clone(), row["value_canon"].clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(observation_values(&omitted), observation_values(&explicit));
        assert_eq!(
            omitted["receipt"]["solve_method"]["binding_kind"],
            "omitted_fixed_precise"
        );
        assert_eq!(omitted["receipt"]["solve_method"]["authored"], false);
        assert_eq!(
            omitted["receipt"]["solve_method"]["public_surface"],
            "#정밀"
        );
        assert_eq!(
            omitted["receipt"]["solve_method"]["source_file"],
            "entry.ddn"
        );
        assert_ne!(
            omitted["receipt"]["run_request_hash"],
            explicit["receipt"]["run_request_hash"]
        );
        assert_eq!(verify_execution_report(&omitted)["ok"], true);

        let mut tampered = omitted.clone();
        tampered["receipt"]["solve_method"]["public_surface"] = json!("#빠름");
        let receipt_hash = hash_json(&{
            let mut receipt = tampered["receipt"].clone();
            receipt.as_object_mut().unwrap().remove("receipt_hash");
            receipt
        });
        tampered["receipt"]["receipt_hash"] = json!(receipt_hash);
        assert_eq!(
            verify_execution_report(&tampered)["candidate_code"],
            "E_MODEL_SOLVE_METHOD_BINDING_MISMATCH_CANDIDATE"
        );
    }

    #[test]
    fn historical_r1_method_executes_without_canonical_surface_disguise() {
        let historical = SUCCESS
            .replace(
                "정밀한방법 <- (방식: #정밀).",
                "정확한방법 <- (방식: \"ddn.exact_acyclic_relation.v1\").",
            )
            .replace("방법=정밀한방법", "방법=정확한방법");
        let program = ddonirang_lang::parse_model_program(&historical).expect("historical parse");
        assert!(program
            .canonical_ddn
            .contains("ddn.exact_acyclic_relation.v1"));
        assert!(!program.canonical_ddn.contains("#정밀"));
        let report = execute_source(&historical).expect("historical R1 compatibility run");
        assert_eq!(report["status"], "completed");
    }

    #[test]
    fn unsupported_korean_method_fails_closed_without_internal_id_in_message() {
        let source = SUCCESS.replace("방식: #정밀", "방식: #빠름");
        let error = execute_source(&source).expect_err("unsupported method");
        assert_eq!(
            error.candidate_code,
            "E_MODEL_RECIPE_BACKEND_MISMATCH_CANDIDATE"
        );
        assert_eq!(error.pre_state_hash, error.post_state_hash);
        assert_eq!(error.receipt["atomic_commit_outcome"], "not_committed");
        assert!(error.message.contains("#정밀"));
        assert!(!error.message.contains(MODEL_BACKEND_ID));
    }

    #[test]
    fn source_relation_order_does_not_change_result_or_dependency_order() {
        let reversed = SUCCESS.replace(
            "  입력 + 1@m =:= 중간.\n  결과 =:= 중간 * 2.",
            "  결과 =:= 중간 * 2.\n  입력 + 1@m =:= 중간.",
        );
        let left = execute_source(SUCCESS).expect("left");
        let right = execute_source(&reversed).expect("right");
        assert_eq!(left["dependency_order"], right["dependency_order"]);
        assert_eq!(left["result_pack"], right["result_pack"]);
    }

    #[test]
    fn fixed_q32_rejects_unrepresentable_each_operation_without_commit() {
        let source = SUCCESS.replace("나눔수@m", "수@m").replace("2@m", "0.1@m");
        let error = execute_source(&source).expect_err("0.1 is not exact Q32.32");
        assert_eq!(
            error.candidate_code,
            "E_MODEL_EXACT_VALUE_UNREPRESENTABLE_CANDIDATE"
        );
        assert_eq!(error.pre_state_hash, error.post_state_hash);
        assert_eq!(error.receipt["atomic_commit_outcome"], "not_committed");
        assert!(error.receipt["result_pack_hash"].is_null());
    }

    #[test]
    fn unsupported_event_derivative_and_connection_do_not_fallback() {
        let cases = [
            (
                SUCCESS.replace(
                    "  입력 + 1@m =:= 중간.",
                    "  입력이 될때 { 중간 <- 입력. }.\n  입력 + 1@m =:= 중간.",
                ),
                "E_MODEL_EVENT_RUNTIME_OPEN_CANDIDATE",
            ),
            (
                SUCCESS
                    .replace(
                        "중간:나눔수@m :: (그자리, 내줌).",
                        "시간:나눔수 :: (축).\n  중간:나눔수@m :: (이어감, 내줌).",
                    )
                    .replace("입력 + 1@m =:= 중간.", "중간' =:= 입력."),
                "E_MODEL_DERIVATIVE_RUNTIME_OPEN_CANDIDATE",
            ),
            (
                SUCCESS.replace(
                    "  입력 + 1@m =:= 중간.",
                    "  계산.입력과 계산.중간을 (값은 같게) 잇기.\n  입력 + 1@m =:= 중간.",
                ),
                "E_MODEL_CONNECTION_RUNTIME_OPEN_CANDIDATE",
            ),
        ];
        for (source, code) in cases {
            let omitted = source
                .replace("정밀한방법 <- (방식: #정밀).\n", "")
                .replace(", 방법=정밀한방법", "");
            for candidate in [&source, &omitted] {
                let error = execute_source(candidate).expect_err(code);
                assert_eq!(error.candidate_code, code);
                assert_eq!(error.pre_state_hash, error.post_state_hash);
                assert_eq!(error.receipt["atomic_commit_outcome"], "not_committed");
                assert!(error.receipt["result_pack_hash"].is_null());
                assert_eq!(error.receipt["solve_method"]["fallback"], false);
                assert_eq!(error.receipt["solve_method"]["retry"], false);
            }
        }
    }

    #[test]
    fn descriptor_and_receipt_are_candidate_and_hash_bound() {
        let report = execute_source(SUCCESS).expect("run");
        assert_eq!(
            report["backend_descriptor"]["wire_status"],
            "candidate_non_public"
        );
        assert_eq!(report["receipt"]["schema"], MODEL_RECEIPT_SCHEMA);
        assert_eq!(report["receipt"]["status"], "success");
        assert_eq!(
            report["execution_bindings"]["model_definition_artifact"]["source_hash"],
            report["receipt"]["model_definition_source_hash"]
        );
        assert_eq!(
            report["execution_bindings"]["solve_recipe"]["source_hash"],
            report["receipt"]["solve_recipe_hash"]
        );
        assert_eq!(
            report["execution_bindings"]["run_request"]["source_hash"],
            report["receipt"]["run_request_hash"]
        );
        assert!(report["receipt"]["receipt_hash"]
            .as_str()
            .is_some_and(|hash| hash.starts_with("blake3:")));
        let envelope = execution_envelope_json(SUCCESS).expect("model envelope");
        assert_eq!(envelope["execution_mode_decision"]["mode"], "oneshot");
        assert_eq!(
            envelope["execution_mode_decision"]["authority"],
            "parsed_program"
        );
        assert_eq!(envelope["configured_madi"], 1);
    }

    #[test]
    fn stored_execution_verifier_rejects_tampered_recipe_and_receipt() {
        let report = execute_source(SUCCESS).expect("run");
        let verified = verify_execution_report(&report);
        assert_eq!(verified["ok"], true);

        let mut recipe_tamper = report.clone();
        recipe_tamper["execution_bindings"]["solve_recipe"]["source_hash"] =
            json!("blake3:tampered-recipe");
        let recipe_verdict = verify_execution_report(&recipe_tamper);
        assert_eq!(recipe_verdict["ok"], false);
        assert_eq!(
            recipe_verdict["candidate_code"],
            "E_MODEL_RECIPE_BINDING_MISMATCH_CANDIDATE"
        );

        let mut receipt_tamper = report;
        receipt_tamper["receipt"]["post_state_hash"] = json!("blake3:tampered-state");
        let receipt_verdict = verify_execution_report(&receipt_tamper);
        assert_eq!(receipt_verdict["ok"], false);
        assert_eq!(
            receipt_verdict["candidate_code"],
            "E_MODEL_RECEIPT_HASH_MISMATCH_CANDIDATE"
        );

        let mut runtime_tamper = execute_source(SUCCESS).expect("run");
        runtime_tamper["receipt"]["runtime_build_identity"]["profile"] = json!("tampered");
        runtime_tamper["receipt"]
            .as_object_mut()
            .expect("receipt object")
            .remove("receipt_hash");
        runtime_tamper["receipt"] = seal_receipt(runtime_tamper["receipt"].clone());
        let runtime_verdict = verify_execution_report(&runtime_tamper);
        assert_eq!(runtime_verdict["ok"], false);
        assert_eq!(
            runtime_verdict["candidate_code"],
            "E_MODEL_RECEIPT_RUNTIME_BINDING_MISMATCH_CANDIDATE"
        );
    }

    #[test]
    fn exact_unit_conversion_uses_rational_scale_and_fixed_rejects_it_when_unrepresentable() {
        let exact = SUCCESS.replace("2@m", "1@cm");
        let report = execute_source(&exact).expect("exact 1cm input");
        assert_eq!(
            report["result_pack"]["ordered_fields"][0]["value_canon"],
            "101/50@m"
        );

        let fixed = exact.replace("나눔수@m", "수@m");
        let error = execute_source(&fixed).expect_err("canonical cm value is not exact Q32.32");
        assert_eq!(
            error.candidate_code,
            "E_MODEL_EXACT_VALUE_UNREPRESENTABLE_CANDIDATE"
        );
        assert_eq!(error.pre_state_hash, error.post_state_hash);
    }

    #[test]
    fn wave_b_negative_corpus_fails_closed_without_result_pack() {
        let cases = [
            (
                crate::self_contained_test_sources::MODEL_RELATION_CYCLE,
                "E_MODEL_RELATION_CYCLE_CANDIDATE",
            ),
            (
                crate::self_contained_test_sources::MODEL_UNIT_MISMATCH,
                "E_MODEL_UNIT_MISMATCH_CANDIDATE",
            ),
            (
                crate::self_contained_test_sources::MODEL_UNAUTHORIZED_INITIAL,
                "E_MODEL_INITIAL_OVERRIDE_UNAUTHORIZED_CANDIDATE",
            ),
            (
                crate::self_contained_test_sources::MODEL_UNSUPPORTED_EVENT,
                "E_MODEL_EVENT_RUNTIME_OPEN_CANDIDATE",
            ),
            (
                crate::self_contained_test_sources::MODEL_WRONG_BACKEND,
                "E_MODEL_RECIPE_BACKEND_MISMATCH_CANDIDATE",
            ),
        ];
        for (source, code) in cases {
            let error = execute_source(source).expect_err(code);
            assert_eq!(error.candidate_code, code);
            assert_eq!(error.pre_state_hash, error.post_state_hash);
            assert_eq!(error.receipt["atomic_commit_outcome"], "not_committed");
            assert!(error.receipt["result_pack_hash"].is_null());
        }
    }

    #[test]
    fn omitted_method_rejects_relation_cycle_before_any_commit() {
        let source = crate::self_contained_test_sources::MODEL_RELATION_CYCLE
            .replace(
                "검증방법 <- (방식: \"ddn.exact_acyclic_relation.v1\").\n\n",
                "",
            )
            .replace("  방법=검증방법,\n", "");
        let error = execute_source(&source).expect_err("omitted cycle must fail closed");
        assert_eq!(error.candidate_code, "E_MODEL_RELATION_CYCLE_CANDIDATE");
        assert_eq!(error.pre_state_hash, error.post_state_hash);
        assert_eq!(error.receipt["atomic_commit_outcome"], "not_committed");
        assert!(error.receipt["result_pack_hash"].is_null());
        assert_eq!(
            error.receipt["solve_method"]["binding_kind"],
            "omitted_fixed_precise"
        );
    }
}
