#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;
mod phase_diagnostics;
pub(crate) mod persistent_dependencies;

use crate::fixed64_boundary::Fixed64FloatBoundary;
fn manipulation_phase_text(phase: ddonirang_core::input::ValueEventPhase) -> &'static str {
    use ddonirang_core::input::ValueEventPhase;
    match phase { ValueEventPhase::Set => "설정", ValueEventPhase::Begin => "시작",
        ValueEventPhase::Move => "이동", ValueEventPhase::End => "끝" }
}
use crate::gate0_registry;
use crate::geoul_snapshot::{input_action_key_aliases, project_keyboard_state, SnapshotStateValue};
use crate::input_tape::runtime_keys_to_tape_mask;
use crate::preprocess::{
    preprocess_source_for_parse, split_file_meta, validate_no_legacy_boim_surface,
    validate_no_legacy_header, FileMeta,
};
use crate::result_place::ResultPlace;
use crate::sealed_domain_dataset::{
    DatasetCell, DatasetColumn, DatasetDtype, DatasetRow, DatasetScalar, DatasetSource,
    MissingReason,
};
use crate::work_cost::{
    builtin_work_cost_rule, expr_registry_key, literal_registry_key, stmt_registry_key,
    verify_stdlib_registry_totality, BuiltinWorkCostRule, StandardResourceLimitsV1,
    WorkCostFailure, WorkCostMeter, WorkCostReceipt,
};
use ddonirang_core::platform::{
    EntityId, NuriWorld, Origin, Patch, PatchOp, ResourceMapEntry, ResourceValue,
};
use ddonirang_core::signals::DiagEvent;
use ddonirang_core::{
    unit_spec_from_symbol, ArithmeticFaultKind, ExprTrace, FaultContext, Fixed64, InputSnapshot,
    ResourceHandle, Signal, SourceSpan, UnitDim, UnitError, UnitValue,
};
use ddonirang_lang::runtime::{
    check_value_type, input_just_pressed, input_pressed, list_add, list_len, list_new, list_nth,
    list_remove, map_get, map_key_canon, string_concat, string_contains, string_ends, string_join,
    string_len, string_split, string_starts, string_to_number, value_type_name, InputState,
    LambdaValue, MapEntry, RuntimeError, Value,
};
use ddonirang_lang::{
    age_not_available_error, canonicalize, normalize_expression_identity, parse_v25_with_mode,
    parse_with_mode, plan_internal_delivery_occurrence, AgeTarget, Assertion, AtSuffix, Body,
    CanonProgram, DeliveryOccurrenceIdentity, EndpointRoute, EndpointRouteFailure,
    EndpointRouteSemantic, ExecutionInstanceIdentity, Expr, ExprKind, Formula, FormulaDialect,
    InternalDeliveryEnvelopeSemantic, InternalDeliveryOccurrenceFailure, Lexer, Literal,
    LogicalAsyncQueue, LogicalWaitReceipt, MessageTransfer, NodeId, ParamPin, ParseError,
    ParseMode, ProviderRegistrationIdentity, QueueAdmission, ReactionProviderRegistry,
    RegexLiteral, SeedDef, SeedKind, SenderIdentityPredicate, StagedDeliveryOutbox, StateMachine,
    StateTransition, Stmt, SumBranchDecl, TemplateFormat, TemplatePart, Token, TokenKind, TopLevelItem, TypeRef,
};
use libm;
use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};
use regex::RegexBuilder;

static LAMBDA_SEQ: AtomicU64 = AtomicU64::new(1);
static DEFAULT_PARSE_MODE: AtomicU64 = AtomicU64::new(1);
static DEFAULT_AGE_TARGET: AtomicU64 = AtomicU64::new(3);
pub const V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY: &str = "v1-core-v25";
pub const E_SOURCE_PROFILE_REQUIRED: &str = "E_SOURCE_PROFILE_REQUIRED";
pub const E_SOURCE_PROFILE_UNSUPPORTED: &str = "E_SOURCE_PROFILE_UNSUPPORTED";
pub const E_V1_CORE_RECURSION_UNSUPPORTED: &str = "E_V1_CORE_RECURSION_UNSUPPORTED";

// Current Bogae defaults are Q31.32 values, not host floating-point inputs.
// These raw values preserve the former truncating values exactly while keeping
// the active runtime inside the canonical Fixed64 representation.
const BOGAE_DEFAULT_STROKE_WIDTH: Fixed64 = Fixed64::from_raw_i64(85_899_345);
const BOGAE_DEFAULT_CIRCLE_RADIUS: Fixed64 = Fixed64::from_raw_i64(343_597_383);
const BOGAE_DEFAULT_POINT_SIZE: Fixed64 = Fixed64::from_raw_i64(193_273_528);
const BOGAE_DEFAULT_RECT_WIDTH: Fixed64 = Fixed64::from_raw_i64(858_993_459);
const BOGAE_DEFAULT_RECT_HEIGHT: Fixed64 = Fixed64::from_raw_i64(515_396_075);
const BOGAE_DEFAULT_TEXT_SIZE: Fixed64 = Fixed64::from_raw_i64(343_597_383);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedSourceProfile {
    V1CoreV25,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedSourceProfileError {
    pub code: &'static str,
    pub message: String,
}

pub fn resolve_supported_source_profile(
    identity: Option<&str>,
) -> Result<SupportedSourceProfile, SupportedSourceProfileError> {
    let identity = identity.map(str::trim).filter(|value| !value.is_empty());
    match identity {
        None => Err(SupportedSourceProfileError {
            code: E_SOURCE_PROFILE_REQUIRED,
            message: "supported source profile을 명시해야 합니다.".to_string(),
        }),
        Some(V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY) => Ok(SupportedSourceProfile::V1CoreV25),
        Some(other) => Err(SupportedSourceProfileError {
            code: E_SOURCE_PROFILE_UNSUPPORTED,
            message: format!("지원하지 않는 source profile입니다: {other}"),
        }),
    }
}
const STATE_MACHINE_RESOURCE_KIND: &str = "ddn.state_machine.v1";
const ASSERTION_RESOURCE_KIND: &str = "ddn.assertion.v1";
const FORMULA_RESOURCE_KIND: &str = "ddn.formula.v1";
const NUMERIC_PACK_KIND_KEY: &str = "__수타입";
const NUMERIC_PACK_APPROX_KEY: &str = "__근사";
const NUMERIC_KIND_BIG_INT: &str = "큰바른수";
const NUMERIC_KIND_RATIONAL: &str = "나눔수";
const NUMERIC_KIND_FACTOR: &str = "곱수";
const RELATION_KIND_FIELD: &str = "__관계종류";
const RELATION_KIND_EQUATION: &str = "방정식";
const RELATION_LEFT_FIELD: &str = "왼쪽";
const RELATION_RIGHT_FIELD: &str = "오른쪽";
const RELATION_SOLVE_RESULT_KIND_FIELD: &str = "__풀이결과종류";
const RELATION_SOLVE_RESULT_SUCCESS: &str = "성공";
const RELATION_SOLVE_RESULT_FAILURE: &str = "실패";
const RELATION_SOLVE_VAR_FIELD: &str = "미지수";
const RELATION_SOLVE_VALUE_FIELD: &str = "값";
const RELATION_SOLVE_BINDINGS_FIELD: &str = "해";
const RELATION_SOLVE_REASON_FIELD: &str = "사유";
const NUMERIC_DIAG_RULE_ID_FACTOR_DECOMP_DEFERRED: &str = "L1-NUMERIC-01";
const NUMERIC_DIAG_REASON_FACTOR_DECOMP_DEFERRED: &str = "NUMERIC_FACTOR_DECOMP_DEFERRED";
const NUMERIC_DIAG_TAG_FACTOR_DECOMP_DEFERRED: &str = "numeric:factor:deferred";
const NUMERIC_DIAG_RULE_ID_FACTOR_ROUTE_SUMMARY: &str = "L1-NUMERIC-02";
const NUMERIC_DIAG_REASON_FACTOR_ROUTE_SUMMARY: &str = "NUMERIC_FACTOR_ROUTE_SUMMARY";
const NUMERIC_DIAG_TAG_FACTOR_ROUTE_SUMMARY: &str = "numeric:factor:routes";
const NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY: &str = "수진단.곱수경로집계";
const NUMERIC_FACTOR_ROUTE_TOTAL_RESOURCE_KEY: &str = "수진단.곱수호출수";
const NUMERIC_FACTOR_BITS_MIN_RESOURCE_KEY: &str = "수진단.곱수비트최소";
const NUMERIC_FACTOR_BITS_MAX_RESOURCE_KEY: &str = "수진단.곱수비트최대";
const NUMERIC_FACTOR_BITS_SUM_RESOURCE_KEY: &str = "수진단.곱수비트합";
const NUMERIC_FACTOR_POLICY_RESOURCE_KEY: &str = "수진단.곱수정책";
const FACTOR_DECOMP_STATUS_KEY: &str = "분해상태";
const FACTOR_DECOMP_ROUTE_KEY: &str = "분해경로";
const FACTOR_DECOMP_BITS_KEY: &str = "분해비트수";
const FACTOR_DECOMP_STATUS_DONE: &str = "완료";
const FACTOR_DECOMP_STATUS_DEFERRED: &str = "지연";
const FACTOR_DECOMP_DEFERRED_REASON_KEY: &str = "지연사유";
const FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT: &str = "비트한계초과";
const FACTOR_DECOMP_DEFERRED_REASON_FACTOR_FAILED: &str = "분해실패";
const FACTOR_DECOMP_ROUTE_ZERO: &str = "zero";
const FACTOR_DECOMP_ROUTE_I64: &str = "i64";
const FACTOR_DECOMP_ROUTE_BIGINT: &str = "bigint";
const FACTOR_DECOMP_ROUTE_BIGINT_SMALL_PRIME: &str = "bigint:smallprime";
const FACTOR_DECOMP_ROUTE_BIGINT_POLLARD: &str = "bigint:pollard";
const FACTOR_DECOMP_ROUTE_BIGINT_FALLBACK: &str = "bigint:fallback";
const FACTOR_DECOMP_ROUTE_BIGINT_MIXED: &str = "bigint:mixed";
const FACTOR_DECOMP_ROUTE_DEFERRED_BIT_LIMIT: &str = "deferred:bitlimit";
const FACTOR_DECOMP_ROUTE_DEFERRED_FACTOR_FAILED: &str = "deferred:factorfailed";
const FACTOR_BIGINT_FACTOR_BITS_LIMIT: usize = 512;
const FACTOR_POLLARD_MAX_ITERS: usize = 200_000;
const FACTOR_POLLARD_C_SEED_LIMIT: u64 = 64;
const FACTOR_POLLARD_X0_SEEDS: [u64; 6] = [2, 3, 5, 7, 11, 13];
const FACTOR_TRIAL_FALLBACK_LIMIT: u64 = 1_000_000;
const FACTOR_SMALL_PRIMES: [u64; 25] = [
    3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101,
];
#[cfg(test)]
thread_local! {
    static FACTOR_POLLARD_MAX_ITERS_OVERRIDE: Cell<Option<usize>> = const { Cell::new(None) };
    static FACTOR_TRIAL_FALLBACK_LIMIT_OVERRIDE: Cell<Option<u64>> = const { Cell::new(None) };
}

#[cfg(test)]
mod w1c2_i2_ddnrunner_dispatch_tests {
    include!("../tests/ddn_execution_concurrency_w1c2_ddnrunner_dispatch.rs");
    w1c2_i2_ddnrunner_dispatch_tests!();
}
mod grid_game_values;
use grid_game_values::{
    STD_GRID_KIND,
    STD_BLOCK_PIECE_KIND,
    STD_RANDOM_BAG_KIND,
    STD_RANDOM_BAG_DRAW_KIND,
    STD_RANDOM_BAG_KIND_FIELD,
    STD_RANDOM_BAG_REMAINING_FIELD,
    STD_RANDOM_BAG_VALUE_FIELD,
    STD_RANDOM_BAG_BAG_FIELD,
    STD_GRID_GAME_STATE_KIND,
    STD_GRID_LINE_CLEAR_KIND,
    STD_FALLING_PIECE_KIND,
    STD_GRID_GAME_SCORE_KIND,
    STD_GRID_GAME_SESSION_KIND,
    STD_GRID_GAME_TICK_KIND,
    STD_GRID_GAME_NEXT_PIECE_KIND,
    STD_GRID_GAME_VIEW_SUMMARY_KIND,
    STD_GRID_GAME_HOLD_KIND,
    STD_GRID_GAME_HOLD_SWAP_KIND,
    STD_GRID_GAME_ROTATION_TRY_KIND,
    make_std_grid,
    std_grid_fields,
    std_grid_dims,
    std_grid_cells,
    std_grid_index,
    std_grid_inside,
    std_block_piece_cells_value,
    std_block_piece_cells_from_value,
    make_std_block_piece,
    std_block_piece_cells,
    std_block_piece_collides,
    std_block_piece_lock,
    make_std_random_bag,
    std_random_bag_fields,
    std_random_bag_list_field,
    std_random_bag_draw_once,
    is_allowed_grid_game_state,
    value_to_grid_game_state_name,
    make_std_grid_game_state,
    std_grid_game_state_fields,
    std_grid_game_state_state,
    std_grid_game_state_previous,
    std_grid_game_state_tick,
    tetromino_names,
    tetromino_cells,
    value_to_tetromino_name,
    std_grid_line_full_rows,
    std_grid_line_clear,
    make_std_falling_piece,
    std_falling_piece_parts,
    std_falling_piece_cells,
    std_falling_piece_block,
    std_falling_piece_rotate,
    make_std_grid_game_score,
    std_grid_game_score_fields,
    std_score_int_field,
    std_grid_game_score_add,
    make_std_grid_game_session,
    std_grid_game_session_fields,
    std_session_field,
    std_grid_game_next_piece,
    make_std_grid_game_hold,
    std_grid_game_hold_fields,
    std_grid_game_hold_piece,
    std_grid_game_hold_used,
    std_grid_game_hold_swap,
    std_grid_game_placeable,
    std_grid_game_rotation_try,
    std_grid_game_apply_input,
    std_grid_game_tick,
    std_grid_game_view_cells,
    std_grid_game_ghost_piece,
    std_grid_game_view_text,
    std_grid_game_view_summary,
    std_grid_game_bogae_drawlist,
    std_grid_game_ghost_bogae_drawlist,
    std_grid_game_bogae_size,
    value_matches_any,
    std_grid_pathfind,
};

const STD_INPUT_MAP_KIND: &str = "표준.입력사상";
fn fixed_value(value: i64) -> Value {
    Value::Fixed64(Fixed64::from_i64(value))
}

fn splitmix64_next(state: u64) -> (u64, u64) {
    let next_state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = next_state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    (next_state, z ^ (z >> 31))
}

fn expect_unit_value(value: &Value) -> Result<UnitValue, EvalError> {
    match value {
        Value::Fixed64(raw) => Ok(UnitValue {
            value: *raw,
            dim: UnitDim::NONE,
        }),
        Value::Unit(unit) => Ok(*unit),
        _ => Err("수 값이 필요합니다".to_string().into()),
    }
}

fn unit_value_to_runtime_value(value: UnitValue) -> Value {
    if value.dim == UnitDim::NONE {
        Value::Fixed64(value.value)
    } else {
        Value::Unit(value)
    }
}

fn ensure_same_unit_dim(left: &UnitValue, right: &UnitValue) -> Result<(), EvalError> {
    if left.dim != right.dim {
        return Err("단위 차원이 맞지 않습니다".to_string().into());
    }
    Ok(())
}

fn ensure_dimensionless_unit(value: &UnitValue) -> Result<(), EvalError> {
    if value.dim != UnitDim::NONE {
        return Err("무차원 수가 필요합니다".to_string().into());
    }
    Ok(())
}

fn checked_unit_mul(left: UnitValue, right: UnitValue) -> Result<UnitValue, EvalError> {
    Ok(UnitValue {
        value: left.value.checked_mul(right.value)?,
        dim: left.dim.add(right.dim),
    })
}

fn checked_unit_add(left: UnitValue, right: UnitValue) -> Result<UnitValue, EvalError> {
    if left.dim != right.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: left.dim,
            right: right.dim,
        }));
    }
    Ok(UnitValue {
        value: left.value.checked_add(right.value)?,
        dim: left.dim,
    })
}

fn checked_unit_sub(left: UnitValue, right: UnitValue) -> Result<UnitValue, EvalError> {
    if left.dim != right.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: left.dim,
            right: right.dim,
        }));
    }
    Ok(UnitValue {
        value: left.value.checked_sub(right.value)?,
        dim: left.dim,
    })
}

fn make_std_input_map(bindings: BTreeMap<String, Value>) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__kind".to_string(),
        Value::String(STD_INPUT_MAP_KIND.to_string()),
    );
    for (key, value) in bindings {
        fields.insert(key, value);
    }
    Value::Pack(fields)
}

fn std_input_map_fields(value: &Value) -> Result<BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(EvalError::from("입력사상 인자가 필요합니다".to_string()));
    };
    if matches!(fields.get("__kind"), Some(Value::String(kind)) if kind == STD_INPUT_MAP_KIND) {
        return Ok(fields.clone());
    }
    Ok(fields.clone())
}

fn input_map_value_keys(map: &BTreeMap<String, Value>, action: &str) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(value) = map
        .get(action)
        .or_else(|| map.get(&format!("동작.{action}")))
        .or_else(|| map.get(&format!("키.{action}")))
    {
        match value {
            Value::String(text) => keys.push(text.clone()),
            Value::List(items) => {
                keys.extend(
                    items
                        .iter()
                        .map(value_to_string)
                        .filter(|key| !key.is_empty()),
                );
            }
            other => {
                let rendered = value_to_string(other);
                if !rendered.is_empty() {
                    keys.push(rendered);
                }
            }
        }
    }
    if keys.is_empty() {
        keys.extend(
            input_action_key_aliases(action)
                .iter()
                .map(|key| key.to_string()),
        );
    }
    keys
}

fn input_map_action_pressed(
    input: &InputState,
    map: &BTreeMap<String, Value>,
    action: &str,
) -> bool {
    input_map_value_keys(map, action)
        .iter()
        .any(|key| matches!(input_pressed(input, key), Value::Bool(true)))
}

fn seed_keyboard_state_resources(
    resources: &mut ResourceBindings,
    prev_mask: u64,
    next_mask: u64,
) {
    let held = runtime_keys_to_tape_mask(next_mask);
    let previous = runtime_keys_to_tape_mask(prev_mask);
    for entry in project_keyboard_state(held, (!previous) & held, previous & !held) {
        if let SnapshotStateValue::Number(value) = entry.value {
            resources.insert(entry.key, Value::Fixed64(Fixed64::from_i64(value)));
        }
    }
}

#[derive(Clone, Debug)]
pub struct DdnParseWarning {
    pub code: String,
    pub message: String,
    pub span_start: usize,
    pub span_end: usize,
}

fn next_lambda_id() -> u64 {
    LAMBDA_SEQ.fetch_add(1, Ordering::Relaxed)
}

fn encode_parse_mode(mode: ParseMode) -> u64 {
    match mode {
        ParseMode::SuccessorDefault => 1,
        ParseMode::HistoricalMigrationEvidence => 2,
    }
}

fn decode_parse_mode(value: u64) -> ParseMode {
    match value {
        1 => ParseMode::SuccessorDefault,
        2 => ParseMode::HistoricalMigrationEvidence,
        _ => ParseMode::SuccessorDefault,
    }
}

fn encode_age_target(age: AgeTarget) -> u64 {
    match age {
        AgeTarget::Age0 => 0,
        AgeTarget::Age1 => 1,
        AgeTarget::Age2 => 2,
        AgeTarget::Age3 => 3,
        AgeTarget::Age4 => 4,
        AgeTarget::Age5 => 5,
        AgeTarget::Age6 => 6,
        AgeTarget::Age7 => 7,
    }
}

fn decode_age_target(value: u64) -> AgeTarget {
    match value {
        0 => AgeTarget::Age0,
        1 => AgeTarget::Age1,
        2 => AgeTarget::Age2,
        3 => AgeTarget::Age3,
        4 => AgeTarget::Age4,
        5 => AgeTarget::Age5,
        6 => AgeTarget::Age6,
        7 => AgeTarget::Age7,
        _ => AgeTarget::Age3,
    }
}

pub fn default_parse_mode() -> ParseMode {
    decode_parse_mode(DEFAULT_PARSE_MODE.load(Ordering::Relaxed))
}

pub fn set_default_parse_mode(mode: ParseMode) {
    DEFAULT_PARSE_MODE.store(encode_parse_mode(mode), Ordering::Relaxed);
}

pub fn default_age_target() -> AgeTarget {
    decode_age_target(DEFAULT_AGE_TARGET.load(Ordering::Relaxed))
}

pub fn set_default_age_target(age: AgeTarget) {
    DEFAULT_AGE_TARGET.store(encode_age_target(age), Ordering::Relaxed);
}

#[derive(Clone)]
pub struct DdnProgram {
    source_identity: String,
    source_capture: Option<crate::source_snapshot_admission::SourceCapture>,
    saved_project_sources: Option<Arc<crate::project_revision_envelope_supply::VerifiedSavedV25Sources>>,
    #[allow(dead_code)]
    program: CanonProgram,
    functions: HashMap<String, SeedDef>,
    products: HashMap<String, SeedDef>,
    record_schemas: ddonirang_lang::runtime::RecordSchemas,
    nominal_constructors:BTreeMap<NodeId,crate::authored_project::registration::ResolvedConstructorV1>,
    registered_atom_literals:BTreeMap<NodeId,ddonirang_core::platform::RegisteredAtomValueV1>,
    result_output_type_plan:Option<Arc<crate::registered_atom_contract::output_evidence::OutputTypePlanV1>>,
    temporary_registration:Option<Arc<crate::authored_project::registration::temporary::TemporaryRegisteredSource>>,
    temporary_subject_bootstrap:Option<crate::unit_wire::subject_execution::SubjectExecutionStage>,
    registered_nominal_catalog:Option<Arc<crate::authored_project::registration::SavedNominalCatalogV1>>,
    registered_atom_contracts:Arc<Vec<crate::registered_atom_contract::RecordedAtomContract>>,
    ordinary_overloads: HashMap<String, Vec<SeedDef>>,
    ordinary_callable_admission_evidence:
        Vec<ddonirang_lang::OrdinaryCallableAdmissionEvidence>,
    root_stmts: Vec<ddonirang_lang::Stmt>,
    derived_plan: Arc<persistent_dependencies::DerivedPlan>,
    top_level_decl_names: HashSet<String>,
    #[allow(dead_code)]
    file_meta: FileMeta,
    parse_warnings: Vec<DdnParseWarning>,
    configured_madi: Option<u64>,
    public_observation_source_refs: HashMap<NodeId, OriginalObservationFieldRef>,
    numeric_constraint_plan: Option<Arc<crate::numeric_constraint_plan::NumericConstraintPlan>>,
    host_numeric_supply:Option<crate::runtime_surface::BogaeHostNumericSupply>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OriginalObservationFieldRef {
    name: String,
    start: usize,
    end: usize,
}

mod program_lifecycle;
mod saved_program_rebuild;
pub(crate) use saved_program_rebuild::RequestScope as SavedProgramRebuildScope;

mod source_validation;
pub use source_validation::{extract_setting_madi};
use source_validation::{validate_v1_core_no_named_callable_recursion, validate_v25_known_infix_operand_types, scan_original_public_observation_fields, token_observation_name, collect_public_observation_fields, collect_top_level_decl_names, authored_top_level_decl_names, enforce_regex_age_gate, enforce_assertion_age_gate, enforce_state_machine_age_gate, enforce_quantifier_age_gate};

mod internal_reactions;
pub use internal_reactions::{InternalReactionProgram, InternalReactionOperation, InternalEffectIntent, ReactionProviderPatchCandidate, ReactionGroupCommitCandidate, InternalReactionFailure, InternalReactionSuccess, InternalReactionPrepared, InternalTypedReactionRuntime};
use internal_reactions::{map_internal_occurrence_failure, internal_reaction_failure_to_eval, evaluate_internal_program, merge_internal_candidates, internal_hash_text, internal_world_digest, internal_reaction_group_digest};

#[derive(Debug, Clone)]
struct InternalDdnRunnerDispatchState {
    provider_registry: ReactionProviderRegistry<InternalReactionProgram>,
    reached_registration_override: Option<Vec<ProviderRegistrationIdentity>>,
    runtime: InternalTypedReactionRuntime,
    execution_identity: ExecutionInstanceIdentity,
    occurrence_counter: u64,
    last_success: Option<InternalReactionSuccess>,
    commit_events: Vec<String>,
    effect_sink_observations: Rc<RefCell<Vec<InternalEffectSinkObservation>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InternalEffectSinkObservation {
    identity: String,
    committed_world: BTreeMap<String, String>,
    committed_revision: u64,
    route_frame_committed: bool,
}

impl Default for InternalDdnRunnerDispatchState {
    fn default() -> Self {
        Self {
            provider_registry: ReactionProviderRegistry::default(),
            reached_registration_override: None,
            runtime: InternalTypedReactionRuntime::default(),
            execution_identity: ExecutionInstanceIdentity::derive(
                "ddn-runner-default-empty-provider-registry",
            ),
            occurrence_counter: 0,
            last_success: None,
            commit_events: Vec::new(),
            effect_sink_observations: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

pub struct DdnRunner {
    subject_bootstrap:Option<crate::unit_wire::subject_execution::SubjectExecutionStage>,
    tuple_type_plan: Rc<crate::tuple_saved_type_plan::TupleSavedTypePlan>,
    program: DdnProgram,
    update_name: String,
    named_entry_after_definitions: bool,
    bogae_activation_requested: bool,
    bogae_entry: Option<String>,
    authored_bogae: Option<crate::authored_bogae::Activation>,
    bogae_history: crate::bogae_time::SceneHistory,
    prev_keys_pressed: u64,
    project_asset_registry: Option<Arc<gate0_registry::ProjectAssetRegistry>>,
    work_cost: Rc<RefCell<WorkCostMeter>>,
    async_queue_capacity: usize,
    internal_dispatch: InternalDdnRunnerDispatchState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransitionRecord {
    pub madi: u64,
    pub origin: String,
    pub from: String,
    pub to: String,
    pub message: String,
    pub guard_name: Option<String>,
    pub action_name: Option<String>,
}

pub struct DdnRunOutput {
    pub(crate) result_emissions: crate::replay_evidence::StagedResultEmissionsV1,
    /// Explicit UTF-8 decoding provenance, separate from display and world state.
    pub explicit_nfc_provenance: Vec<(NodeId, ddonirang_core::bytes::NfcDecodeProvenance)>,
    /// A3-P1: typed display sidecar; never a world resource or input authority.
    pub display_outputs: Vec<crate::output_display::OutputDisplay>,
    pub bogae_projection: Option<serde_json::Value>,
    pub patch: Patch,
    /// 실제 실행 중 읽은 입력 필드. 세계 상태에는 넣지 않는다.
    pub input_reads: BTreeSet<String>,
    pub resources: HashMap<String, Value>,
    /// 성공적으로 commit된 현재 마디의 공개 관찰 event.
    ///
    /// 선언 순서는 곧 wire 순서다. state resource에 넣지 않으므로 state hash에는
    /// 참여하지 않으며, 이후 trace/view hash 계층이 이 값을 소비한다.
    pub public_observations: Vec<PublicObservationEvent>,
    #[allow(dead_code)]
    pub state_transitions: Vec<StateTransitionRecord>,
    pub work_cost: WorkCostReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicObservationEvent {
    pub committed_madi: u64,
    /// 같은 committed 마디 안에서 새 공개 관찰문이 시작되는 경계.
    ///
    /// 배열 순번이나 사용자 의미 ID가 아니라 AST 문장 실행 경계다. trace는
    /// 이 경계를 사용해 서로 다른 필드 집합을 한 행으로 임의 병합하지 않는다.
    pub starts_observation_event: bool,
    pub name: String,
    /// DDN 이름과 함께 의미 정체성을 이루는 정확한 원문 위치.
    pub source_ref: String,
    pub source_start: usize,
    pub source_end: usize,
    pub dtype: String,
    pub unit: String,
    pub format: String,
    pub value_canon: String,
    pub value_text: String,
    /// 봉인 dataset scalar에서 직접 이어진 경우에만 존재하는 Rust provenance.
    ///
    /// UI는 이 target과 검증된 evidence inspection을 결합할 수 있지만 값이나
    /// 과목 이름으로 target을 추정해서는 안 된다.
    pub dataset_target: Option<PublicObservationDatasetTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicObservationDatasetTarget {
    pub dataset_content_sha256: String,
    pub row_source_ordinal: u64,
    pub column_name: String,
}

fn parse_state_transition_tag(tag: &str) -> Option<(String, String)> {
    let body = tag.strip_prefix("state_transition:")?;
    let (from, to) = body.split_once("->")?;
    Some((from.to_string(), to.to_string()))
}

fn parse_state_transition_text(text: Option<&str>) -> (Option<String>, Option<String>) {
    let mut guard_name = None;
    let mut action_name = None;
    let Some(text) = text else {
        return (None, None);
    };
    for part in text.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("guard=") {
            let value = value.trim();
            if !value.is_empty() && value != "-" {
                guard_name = Some(value.to_string());
            }
        } else if let Some(value) = part.strip_prefix("action=") {
            let value = value.trim();
            if !value.is_empty() && value != "-" {
                action_name = Some(value.to_string());
            }
        }
    }
    (guard_name, action_name)
}

fn collect_state_transition_records(ops: &[PatchOp]) -> Vec<StateTransitionRecord> {
    ops.iter()
        .filter_map(|op| match op {
            PatchOp::EmitSignal {
                signal: Signal::Diag { event },
                ..
            } if event.rule_id == "L1-STATE-01" && event.reason == "STATE_TRANSITION" => {
                let expr = event.expr.as_ref()?;
                let (from, to) = parse_state_transition_tag(&expr.tag)?;
                let (guard_name, action_name) = parse_state_transition_text(expr.text.as_deref());
                Some(StateTransitionRecord {
                    madi: event.madi,
                    origin: event.origin.clone(),
                    from,
                    to,
                    message: event.message.clone().unwrap_or_default(),
                    guard_name,
                    action_name,
                })
            }
            _ => None,
        })
        .collect()
}

pub(crate) struct SavedTupleRestorePlan {
    plan: crate::tuple_saved_type_plan::TupleSavedTypePlan,
}

impl SavedTupleRestorePlan {
    pub(crate) fn admit_resource(&self, tag: &str, value: &ResourceValue) -> Result<(), String> {
        let kind = self.plan.expected(tag).ok_or("E_TUPLE_PERSISTENCE_UNBOUND: 저장 원문의 유일한 자원 결속·위치씨를 확인할 수 없습니다")?;
        crate::unit_wire::validate_saved_value_type(value, kind)
            .map_err(|_| "E_TUPLE_PERSISTENCE_UNBOUND: 복원 값이 원문에 결속된 위치씨와 다릅니다".to_owned())
    }
}

mod runner_execution;

// Entries are immutable once appended. Snapshots share their payloads while
// retaining independent prefix membership and the existing rollback flow.
#[derive(Clone, Debug, Default, PartialEq)]
struct PatchLog(Vec<Arc<PatchOp>>);

impl PatchLog {
    fn push(&mut self, op: PatchOp) { self.0.push(Arc::new(op)); }
    fn clear(&mut self) { self.0.clear(); }
    fn len(&self) -> usize { self.0.len() }
    fn is_empty(&self) -> bool { self.0.is_empty() }
    fn iter(&self) -> impl Iterator<Item = &PatchOp> { self.0.iter().map(Arc::as_ref) }
    fn extend(&mut self, ops: impl IntoIterator<Item = PatchOp>) {
        self.0.extend(ops.into_iter().map(Arc::new));
    }
    fn into_owned(self) -> Vec<PatchOp> {
        self.0.into_iter().map(|op| Arc::try_unwrap(op).unwrap_or_else(|op| (*op).clone())).collect()
    }
}

// Binding payloads are immutable; replacement/removal changes only this map.
// Snapshots retain the original values without copying growing output lists.
#[derive(Clone, Debug, Default)]
struct ResourceBindings(HashMap<String, Arc<Value>>);

impl ResourceBindings {
    fn get(&self, name: &str) -> Option<&Value> { self.0.get(name).map(Arc::as_ref) }
    fn contains_key(&self, name: &str) -> bool { self.0.contains_key(name) }
    fn insert(&mut self, name: String, value: Value) {
        self.0.insert(name, Arc::new(value));
    }
    fn remove(&mut self, name: &str) -> Option<Value> {
        self.0.remove(name).map(|value| Arc::try_unwrap(value).unwrap_or_else(|value| (*value).clone()))
    }
    fn into_owned(self) -> HashMap<String, Value> {
        self.0.into_iter().map(|(name, value)|
            (name, Arc::try_unwrap(value).unwrap_or_else(|value| (*value).clone()))).collect()
    }
}

impl From<HashMap<String, Value>> for ResourceBindings {
    fn from(values: HashMap<String, Value>) -> Self {
        Self(values.into_iter().map(|(name, value)| (name, Arc::new(value))).collect())
    }
}

struct EvalContext<'a> {
    result_emissions: crate::replay_evidence::StagedResultEmissionsV1,
    subject_stage:Option<Rc<RefCell<crate::unit_wire::subject_execution::SubjectExecutionStage>>>,
    tuple_type_plan: Rc<crate::tuple_saved_type_plan::TupleSavedTypePlan>,
    explicit_nfc_provenance: Vec<(NodeId, ddonirang_core::bytes::NfcDecodeProvenance)>,
    program: &'a DdnProgram,
    // Bound to this immutable program; shared only by its child contexts.
    source_span_cache: Rc<RefCell<HashMap<(usize, usize), SourceSpan>>>,
    world: &'a NuriWorld,
    defaults: &'a HashMap<String, Value>,
    param_overrides: Option<&'a BTreeMap<String, Value>>,
    resources: ResourceBindings,
    input: InputState,
    logical_input: Option<ddonirang_core::input::LogicalInputQuery<String>>,
    input_reads: Rc<RefCell<BTreeSet<String>>>,
    control_pressed: Option<u64>,
    patch_ops: PatchLog,
    guard_rejected: bool,
    aborted: bool,
    current_seed_name: Option<String>,
    /// Real callable invocation, distinct from synthetic periodic hook context.
    active_callable_name: Option<String>,
    invocation_ordinal: u64,
    result_place_frames: Vec<CallableResultPlaceFrame>,
    /// Lexical/delivery owner context. Ordinary functions inherit it; top-level
    /// movement seeds do not become an implicit sender identity.
    current_owner_identity: Option<String>,
    pending_async_signals: LogicalAsyncQueue<PendingAsyncSignal>,
    async_outbox: StagedDeliveryOutbox<PendingAsyncSignal>,
    async_wait_receipts: Vec<LogicalWaitReceipt>,
    rng_state: u64,
    flow_stack: Vec<Option<Value>>,
    tick_id: u64,
    const_scopes: Vec<HashSet<String>>,
    binding_type_scopes: Vec<HashMap<String, (TypeRef, bool)>>,
    pending_top_level_decl_names: HashSet<String>,
    factor_route_counts: BTreeMap<String, u64>,
    factor_bits_total: u128,
    factor_bits_min: Option<u64>,
    factor_bits_max: u64,
    public_observations: Vec<PublicObservationEvent>,
    display_outputs: Vec<crate::output_display::OutputDisplay>,
    /// Root-program `<-` bindings are ordinary mutable bindings, but they must
    /// not silently change their runtime value category on a later rebind.
    root_program_active: bool,
    derived_initialized: bool,
    root_binding_types: HashMap<String, String>,
    sum_definitions: HashMap<String, Vec<SumBranchDecl>>,
    // Pattern payloads are lexical bindings, never world slots by spelling.
    // A named call starts its own binding owner and restores this stack.
    sum_pattern_bindings: Vec<String>,
    project_asset_registry: Option<&'a gate0_registry::ProjectAssetRegistry>,
    work_cost: Rc<RefCell<WorkCostMeter>>,
    internal_dispatch: InternalDdnRunnerDispatchState,
}

#[derive(Clone)]
struct CallableResultPlaceFrame {
    canonical_name: String,
    _invocation_identity: String,
    result_type: Option<TypeRef>,
    place: ResultPlace<Value>,
}

#[derive(Clone)]
struct PendingAsyncSignal {
    receiver_name: String,
    event_kind: String,
    sender_name: String,
    event_payload: Value,
    source_span: ddonirang_lang::Span,
}

enum ThunkResult {
    Value(Value),
    Return(Value),
    ContinueLoop(ddonirang_lang::Span),
    Break(ddonirang_lang::Span),
}

enum FlowControl {
    Continue,
    Return(Value),
    ContinueLoop(ddonirang_lang::Span),
    Break(ddonirang_lang::Span),
}

fn restore_local_binding(locals: &mut HashMap<String, Value>, name: &str, previous: Option<Value>) {
    if let Some(value) = previous {
        locals.insert(name.to_string(), value);
    } else {
        locals.remove(name);
    }
}

#[derive(Clone)]
struct EvalFrameSnapshot {
    result_emissions: crate::replay_evidence::StagedResultEmissionsV1,
    derived_initialized: bool,
    subject_stage:Option<crate::unit_wire::subject_execution::SubjectExecutionStage>,
    explicit_nfc_provenance: Vec<(NodeId, ddonirang_core::bytes::NfcDecodeProvenance)>,
    locals: Option<HashMap<String, Value>>,
    resources: ResourceBindings,
    patch_ops: PatchLog,
    guard_rejected: bool,
    const_scopes: Vec<HashSet<String>>,
    binding_type_scopes: Vec<HashMap<String, (TypeRef, bool)>>,
    factor_route_counts: BTreeMap<String, u64>,
    factor_bits_total: u128,
    factor_bits_min: Option<u64>,
    factor_bits_max: u64,
    public_observations: Vec<PublicObservationEvent>,
    display_outputs: Vec<crate::output_display::OutputDisplay>,
    root_binding_types: HashMap<String, String>,
    sum_definitions: HashMap<String, Vec<SumBranchDecl>>,
    sum_pattern_bindings: Vec<String>,
}

// This is an implementation proof of no local writes, not a language purity
// inference. Calls, thunks, flows and all future expression kinds fall back.
// A collection/pack allocates values, and a field lookup reads a value; neither
// can write locals when every evaluated child has the same bounded proof.
fn expression_preserves_local_bindings(expr: &Expr) -> bool {
    fn check(expr: &Expr, depth: usize, visits: &mut usize) -> bool {
        // This optimization must not walk a skipped deep subtree without a
        // bound. Beyond this CPU-only budget, retain the original snapshot.
        if depth == 0 || *visits == 0 { return false; }
        *visits -= 1;
        match &expr.kind {
            ExprKind::Literal(Literal::Int(_) | Literal::Fixed64(_) | Literal::Bool(_)
                | Literal::String(_) | Literal::Atom(_) | Literal::None)
            | ExprKind::Var(_) => true,
            ExprKind::Infix { left, right, .. } => {
                check(left, depth - 1, visits) && check(right, depth - 1, visits)
            }
            ExprKind::FieldAccess { target, .. } => check(target, depth - 1, visits),
            ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => elements.iter()
                .all(|element| check(element, depth - 1, visits)),
            ExprKind::Pack { fields } => fields.iter()
                .all(|(_, value)| check(value, depth - 1, visits)),
            _ => false,
        }
    }
    check(expr, 32, &mut 128)
}

// Body snapshots restore a successful statement only when an abort contract
// fired. These bounded, call-free statements cannot evaluate a contract. An
// ordinary error still follows the existing outer candidate rejection path.
// Calls, indexed writes, deferred writes and all compound/future statements
// retain their full rollback snapshot. This does not infer language purity.
fn statement_cannot_abort(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Definition {value,..} | Stmt::Expr {expr:value,..}
            | Stmt::Return {value,..} => expression_preserves_local_bindings(value),
        Stmt::Mutate {target,value,deferred:false,..} =>
            matches!(&target.kind,ExprKind::Var(_)) && expression_preserves_local_bindings(value),
        _ => false,
    }
}

fn root_binding_type_key(value: &Value) -> String {
    match value {
        Value::Record { field_types, .. } => {
            let mut bytes = Vec::new();
            ddonirang_core::platform::RecordFieldType::Record { fields: field_types.clone() }.encode_canon(&mut bytes);
            format!("record-v1:{}", bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>())
        }
        Value::Tensor(tensor) => {
            let mut axes = tensor.shape().iter().map(u64::to_string).collect::<Vec<_>>().join(", ");
            if tensor.shape().len() == 1 { axes.push(','); }
            format!("(원소씨=셈수, 모양=({axes})) 텐서씨")
        }
        // Integer-valued and fractional Fixed64 values are one current numeric
        // category; `value_type_name` deliberately distinguishes their display
        // labels, which is not a rebinding-type distinction.
        Value::Fixed64(_) => "수".to_string(),
        Value::Unit(unit) if unit.is_dimensionless() => "수".to_string(),
        Value::Unit(unit) => format!("수@{}", unit.dim.format()),
        _ => value_type_name(value),
    }
}

fn inferred_binding_type(value: &Value) -> TypeRef {
    if let Value::Tensor(tensor) = value {
        return TypeRef::Tensor { element: Box::new(TypeRef::Named("셈수".into())), shape: tensor.shape().to_vec() };
    }
    // Reuse only types already understood by the runtime checker. None has
    // no inferred payload type; its policy remains a separate decision.
    if matches!(value, Value::None) {
        return TypeRef::Infer;
    }
    let candidate = TypeRef::Named(root_binding_type_key(value));
    if check_value_type(value, &candidate).is_ok() {
        candidate
    } else {
        TypeRef::Infer
    }
}

#[cfg(test)]
mod alpha3_unit_binding_tests {
    use super::*;

    // A3-P2 provisional: retain the existing dimension, not a display unit.
    #[test]
    fn a3_p2_derived_unit_binding_retains_dimension() {
        let dimension = UnitDim::LENGTH.add(UnitDim::LENGTH);
        let area = Value::Unit(UnitValue { value: Fixed64::from_i64(2), dim: dimension });
        let inferred = inferred_binding_type(&area);
        assert!(!matches!(inferred, TypeRef::Infer));
        assert!(check_value_type(&area, &inferred).is_ok());
        let distance = Value::Unit(UnitValue { value: Fixed64::from_i64(2), dim: UnitDim::LENGTH });
        assert!(check_value_type(&distance, &inferred).is_err());
        assert!(check_value_type(&Value::Fixed64(Fixed64::from_i64(2)), &inferred).is_err());
    }

    #[test]
    fn a3_p2_known_scaled_unit_binding_uses_same_dimension() {
        let length = Value::Unit(UnitValue { value: Fixed64::from_i64(1), dim: UnitDim::LENGTH });
        let inferred = inferred_binding_type(&length);
        assert!(check_value_type(&length, &inferred).is_ok());
        assert!(check_value_type(&length, &TypeRef::Named("수@cm".into())).is_ok());
        let time = Value::Unit(UnitValue { value: Fixed64::from_i64(1), dim: UnitDim::TIME });
        assert!(check_value_type(&time, &inferred).is_err());
    }
}

#[cfg(test)]
mod alpha3_chaebi_lifetime_tests {
    // A3-P3 PROVISIONAL: initialization is not a per-madi recalculation.
    #[test]
    fn a3_p3_root_chaebi_initial_expression_runs_once() {
        let source = include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha3_chaebi_once.ddn");
        let result = crate::runtime_surface::run_summary_from_supported_source_profile(
            source, "chaebi-once.ddn", Some(3), Some("v1-core-v25"),
        ).expect("a retained initial value must not re-evaluate division on madi two");
        assert_eq!(result["output_display"]["texts"], serde_json::json!(["1", "1", "1"]));
        assert_eq!(result["resources"]["fixed64"]["분모"], "0");
    }

    #[test]
    fn a3_p3_root_chaebi_without_initial_value_reads_retained_slot() {
        let source = include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha3_chaebi_empty.ddn");
        let result = crate::runtime_surface::run_summary_from_supported_source_profile(
            source, "chaebi-empty.ddn", Some(3), Some("v1-core-v25"),
        ).expect("a declaration without an initializer must not mask a retained slot with None");
        assert_eq!(result["output_display"]["texts"], serde_json::json!(["4", "5", "6"]));
    }
}

fn checked_optional_map_get(map: &BTreeMap<String, MapEntry>, key: &Value) -> Result<Value, EvalError> {
    match map.get(&map_key_canon(key)) {
        Some(entry) if matches!(entry.value, Value::None) => Err(
            "E_RUNTIME_NESTED_OPTIONAL_UNSUPPORTED: 저장된 없음과 없는 열쇠를 구별하는 선택값은 아직 지원하지 않습니다".to_string().into(),
        ),
        Some(entry) => Ok(entry.value.clone()),
        None => Ok(Value::None),
    }
}

enum IndexedSelectionExpr<'a> {
    Single(&'a Expr),
    Axes(&'a [ddonirang_lang::ast::TensorAxisSelector]),
}

enum EvaluatedIndexedSelection {
    Single {value:Value, static_index:Option<usize>},
    Range {start:u64,end:u64,dynamic:bool},
    Axes(Vec<crate::unit_wire::TensorSelector>),
}

fn tuple_position_type_equal(left:&Value,right:&Value)->bool {
    let mut pending=vec![(left,right)];
    while let Some((left,right))=pending.pop() {
        match (left,right) {
            (Value::Tuple(a),Value::Tuple(b))=>{
                if a.len()!=b.len(){return false;}
                pending.extend(a.iter().zip(b));
            }
            (Value::Record{field_types:a,..},Value::Record{field_types:b,..})=>if a!=b{return false;},
            (Value::List(a),Value::List(b))=>{
                if let Some(first)=a.first(){pending.extend(b.iter().map(|item|(first,item)));}
            }
            _=>if root_binding_type_key(left)!=root_binding_type_key(right){return false;},
        }
    }
    true
}

fn checked_selection_range(start:u64,end:u64,len:usize) -> Result<(usize,usize),EvalError> {
    let convert=|| {Some((usize::try_from(start).ok()?,usize::try_from(end).ok()?))};
    let Some((start,end))=convert() else {return Err("E_SELECTION_RANGE_BOUNDS: 범위가 주소 크기를 넘습니다".to_string().into());};
    if start>end || end>len {return Err("E_SELECTION_RANGE_BOUNDS: 차림 범위는 시작 ≤ 끝 ≤ 길이여야 합니다".to_string().into());}
    Ok((start,end))
}

#[derive(Debug)]
pub(crate) enum EvalError {
    Message(String),
    SourceLocated {
        error: Box<EvalError>,
        span: (usize, usize),
        line: u32,
        column: u32,
    },
    TypeMismatch {
        pin: String,
        expected: String,
        actual: String,
    },
    AsyncCandidate {
        code: &'static str,
        phase: &'static str,
        message: String,
        span: (usize, usize),
    },
    UnitMismatch {
        left: UnitDim,
        right: UnitDim,
    },
    ArithmeticFault(ArithmeticFaultKind),
    DivisionByZero,
    FieldAccess {
        code: &'static str,
        message: String,
        span: (usize, usize),
    },
    WorkCost(WorkCostFailure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BogaeShapeKind {
    Line,
    Circle,
    Point,
    Rect,
    Text,
    Polyline,
    Polygon,
}

#[derive(Debug, Clone)]
struct BogaeShapeCall {
    kind: BogaeShapeKind,
    positional: Vec<String>,
    named: BTreeMap<String, String>,
}

impl EvalError {
    fn async_candidate(
        code: &'static str,
        phase: &'static str,
        message: impl Into<String>,
        span: ddonirang_lang::Span,
    ) -> Self {
        Self::AsyncCandidate {
            code,
            phase,
            message: message.into(),
            span: (span.start, span.end),
        }
    }

    fn message(&self) -> String {
        match self {
            EvalError::Message(message) => message.clone(),
            EvalError::SourceLocated { error, line, column, .. } => {
                format!("{} ({line}:{column})", error.message())
            }
            EvalError::TypeMismatch {
                pin,
                expected,
                actual,
            } => format!(
                "[E_RUNTIME_TYPE_MISMATCH] 핀={} 기대={} 실제={}",
                pin, expected, actual
            ),
            EvalError::AsyncCandidate { code, message, .. } => format!("{code}: {message}"),
            EvalError::UnitMismatch { left, right } => {
                format!(
                    "단위 차원이 다릅니다: {} vs {}",
                    left.format(),
                    right.format()
                )
            }
            EvalError::DivisionByZero => "0으로 나눌 수 없습니다".to_string(),
            EvalError::ArithmeticFault(ArithmeticFaultKind::Overflow) => {
                "Fixed64 계산 범위를 벗어났습니다".to_string()
            }
            EvalError::ArithmeticFault(ArithmeticFaultKind::DivByZero) => {
                "0으로 나눌 수 없습니다".to_string()
            }
            EvalError::ArithmeticFault(ArithmeticFaultKind::DimensionMismatch { left, right }) => {
                format!("단위 차원이 다릅니다: {} vs {}", left.format(), right.format())
            }
            EvalError::FieldAccess { code, message, .. } => format!("{code}: {message}"),
            EvalError::WorkCost(failure) => {
                let receipt = serde_json::to_string(&failure.receipt)
                    .unwrap_or_else(|_| "{\"receipt_error\":true}".to_string());
                format!("{}; work_cost_receipt={receipt}", failure)
            }
        }
    }

    pub(crate) fn diagnostic_parts(&self) -> (&'static str, String, Option<(usize, usize)>) {
        match self {
            EvalError::SourceLocated { error, span, .. } => {
                (error.diagnostic_parts().0, self.message(), Some(*span))
            }
            EvalError::TypeMismatch { .. } => ("E_RUNTIME_TYPE_MISMATCH", self.message(), None),
            EvalError::AsyncCandidate {
                code,
                message,
                span,
                ..
            } => (*code, message.clone(), Some(*span)),
            EvalError::FieldAccess {
                code,
                message,
                span,
            } => (*code, message.clone(), Some(*span)),
            EvalError::WorkCost(failure) => (failure.code, self.message(), None),
            EvalError::ArithmeticFault(ArithmeticFaultKind::Overflow) => {
                ("E_RUNTIME_ARITH_OVERFLOW", self.message(), None)
            }
            _ => ("E_RUNTIME_EXECUTION", self.message(), None),
        }
    }

    pub(crate) fn diagnostic_phase(&self) -> &'static str {
        match self {
            EvalError::SourceLocated { error, .. } => error.diagnostic_phase(),
            EvalError::AsyncCandidate { phase, .. } => phase,
            _ => "runtime_execution",
        }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl From<String> for EvalError {
    fn from(message: String) -> Self {
        EvalError::Message(message)
    }
}

impl From<WorkCostFailure> for EvalError {
    fn from(failure: WorkCostFailure) -> Self {
        EvalError::WorkCost(failure)
    }
}

impl From<UnitError> for EvalError {
    fn from(err: UnitError) -> Self {
        match err {
            UnitError::DimensionMismatch { left, right } => EvalError::UnitMismatch { left, right },
            UnitError::DivisionByZero => EvalError::DivisionByZero,
            UnitError::Overflow => EvalError::ArithmeticFault(ArithmeticFaultKind::Overflow),
        }
    }
}

impl From<ArithmeticFaultKind> for EvalError {
    fn from(kind: ArithmeticFaultKind) -> Self {
        Self::ArithmeticFault(kind)
    }
}

impl From<RuntimeError> for EvalError {
    fn from(err: RuntimeError) -> Self {
        EvalError::Message(runtime_error_message(err))
    }
}

mod evaluation_context;

fn receive_stmt_matches_rank(
    rank: u8,
    kind: Option<&str>,
    binding: Option<&String>,
    condition: Option<&Expr>,
    event_kind: &str,
) -> bool {
    let kind_matches = match kind {
        Some(kind_name) => kind_name == event_kind,
        None => true,
    };
    if !kind_matches {
        return false;
    }
    match rank {
        0 => kind.is_some() && binding.is_some() && condition.is_some(),
        1 => kind.is_some() && binding.is_none() && condition.is_none(),
        2 => kind.is_none() && binding.is_some() && condition.is_some(),
        3 => kind.is_none() && binding.is_none() && condition.is_none(),
        _ => false,
    }
}

fn literal_to_value(lit: &Literal) -> Value {
    match lit {
        Literal::Int(n) => Value::Fixed64(Fixed64::from_i64(*n)),
        Literal::Fixed64(n) => Value::Fixed64(*n),
        Literal::String(s) => Value::String(s.clone()),
        Literal::Bytes(bytes) => Value::Bytes(bytes.clone()),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Atom(a) => Value::String(a.clone()),
        Literal::Regex(regex) => Value::Regex(regex.clone()),
        Literal::Resource(path) => Value::ResourceHandle(ResourceHandle::from_path(path)),
        Literal::None => Value::None,
    }
}

fn sealed_dataset_runtime_value(opened: &gate0_registry::OpenedSealedDataset) -> Value {
    let dataset = &opened.dataset;
    dataset_pack(vec![
        ("schema", Value::String(dataset.schema.clone())),
        ("artifact_id", Value::String(dataset.artifact_id.clone())),
        (
            "artifact_version",
            Value::String(dataset.artifact_version.clone()),
        ),
        (
            "content_sha256",
            Value::String(dataset.content_sha256.clone()),
        ),
        (
            "schema_sha256",
            Value::String(dataset.schema_sha256.clone()),
        ),
        (
            "payload_sha256",
            Value::String(opened.payload_sha256.clone()),
        ),
        (
            "asset_manifest_sha256",
            Value::String(opened.asset_manifest_sha256.clone()),
        ),
        (
            "envelope_asset_blake3",
            Value::String(opened.envelope_asset_blake3.clone()),
        ),
        (
            "envelope_sha256",
            Value::String(opened.envelope_sha256.clone()),
        ),
        (
            "recipe_sha256",
            opened
                .recipe_sha256
                .as_ref()
                .map(|value| Value::String(value.clone()))
                .unwrap_or(Value::None),
        ),
        (
            "derived_artifact_sha256",
            opened
                .derived_artifact_sha256
                .as_ref()
                .map(|value| Value::String(value.clone()))
                .unwrap_or(Value::None),
        ),
        ("source", dataset_source_runtime_value(&dataset.source)),
        (
            "열",
            Value::List(
                dataset
                    .ordered_columns
                    .iter()
                    .map(dataset_column_runtime_value)
                    .collect(),
            ),
        ),
        (
            "행",
            Value::List(
                dataset
                    .ordered_rows
                    .iter()
                    .map(|row| {
                        dataset_row_runtime_value(
                            row,
                            &dataset.ordered_columns,
                            &dataset.content_sha256,
                        )
                    })
                    .collect(),
            ),
        ),
    ])
}

fn dataset_source_runtime_value(source: &DatasetSource) -> Value {
    match source {
        DatasetSource::ProjectAuthored {
            project_revision_ref,
            source_ref,
            author_label,
        } => dataset_pack(vec![
            ("kind", Value::String("project_authored".to_string())),
            (
                "project_revision_ref",
                Value::String(project_revision_ref.clone()),
            ),
            ("source_ref", Value::String(source_ref.clone())),
            ("author_label", Value::String(author_label.clone())),
        ]),
        DatasetSource::ExternalSnapshot {
            provider,
            title,
            canonical_uri,
            published_or_version,
            retrieved_at,
            license_id,
            attribution,
        } => dataset_pack(vec![
            ("kind", Value::String("external_snapshot".to_string())),
            ("provider", Value::String(provider.clone())),
            ("title", Value::String(title.clone())),
            ("canonical_uri", Value::String(canonical_uri.clone())),
            (
                "published_or_version",
                Value::String(published_or_version.clone()),
            ),
            ("retrieved_at", Value::String(retrieved_at.clone())),
            ("license_id", Value::String(license_id.clone())),
            ("attribution", Value::String(attribution.clone())),
        ]),
    }
}

fn dataset_column_runtime_value(column: &DatasetColumn) -> Value {
    dataset_pack(vec![
        ("name", Value::String(column.name.clone())),
        (
            "dtype",
            Value::String(
                match column.dtype {
                    DatasetDtype::Integer => "integer",
                    DatasetDtype::Fixed64 => "fixed64",
                    DatasetDtype::String => "string",
                    DatasetDtype::Boolean => "boolean",
                }
                .to_string(),
            ),
        ),
        (
            "unit",
            column
                .unit
                .as_ref()
                .map(|unit| Value::String(unit.clone()))
                .unwrap_or(Value::None),
        ),
        (
            "source_locator",
            Value::String(column.source_locator.clone()),
        ),
    ])
}

fn dataset_row_runtime_value(
    row: &DatasetRow,
    columns: &[DatasetColumn],
    content_sha256: &str,
) -> Value {
    dataset_pack(vec![
        (
            "source_ordinal",
            Value::String(row.source_ordinal.to_string()),
        ),
        (
            "cells",
            Value::List(
                row.cells
                    .iter()
                    .zip(columns)
                    .map(|(cell, column)| {
                        dataset_cell_runtime_value(cell, column, content_sha256, row.source_ordinal)
                    })
                    .collect(),
            ),
        ),
    ])
}

fn dataset_cell_runtime_value(
    cell: &DatasetCell,
    column: &DatasetColumn,
    content_sha256: &str,
    row_source_ordinal: u64,
) -> Value {
    match cell {
        DatasetCell::Present { value } => dataset_pack(vec![
            ("status", Value::String("present".to_string())),
            (
                "value",
                dataset_scalar_runtime_value(value, column, content_sha256, row_source_ordinal),
            ),
        ]),
        DatasetCell::Gap => dataset_pack(vec![("status", Value::String("gap".to_string()))]),
        DatasetCell::Missing { reason } => dataset_pack(vec![
            ("status", Value::String("missing".to_string())),
            (
                "reason",
                Value::String(
                    match reason {
                        MissingReason::CalculationFailed => "#계산실패",
                        MissingReason::SourceUnavailable => "#원천없음",
                        MissingReason::RecordDamaged => "#기록손상",
                    }
                    .to_string(),
                ),
            ),
        ]),
        DatasetCell::Pruned => dataset_pack(vec![("status", Value::String("pruned".to_string()))]),
    }
}

fn dataset_scalar_runtime_value(
    value: &DatasetScalar,
    column: &DatasetColumn,
    content_sha256: &str,
    row_source_ordinal: u64,
) -> Value {
    let (kind, value) = match value {
        DatasetScalar::Integer { value } => ("integer", Value::String(value.to_string())),
        DatasetScalar::Fixed64 { value } => ("fixed64", Value::String(value.clone())),
        DatasetScalar::String { value } => ("string", Value::String(value.clone())),
        DatasetScalar::Boolean { value } => ("boolean", Value::Bool(*value)),
    };
    dataset_pack(vec![
        ("name", Value::String(column.name.clone())),
        ("kind", Value::String(kind.to_string())),
        ("value", value),
        (
            "unit",
            column
                .unit
                .as_ref()
                .map(|unit| Value::String(unit.clone()))
                .unwrap_or(Value::None),
        ),
        (
            "source_locator",
            Value::String(column.source_locator.clone()),
        ),
        ("content_sha256", Value::String(content_sha256.to_string())),
        (
            "row_source_ordinal",
            Value::String(row_source_ordinal.to_string()),
        ),
    ])
}

fn dataset_pack(fields: Vec<(&str, Value)>) -> Value {
    Value::Pack(
        fields
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn build_regex(regex: &RegexLiteral) -> Result<regex::Regex, EvalError> {
    validate_regex_flags(&regex.flags)?;
    let mut builder = RegexBuilder::new(&regex.pattern);
    for ch in regex.flags.chars() {
        match ch {
            'i' => builder.case_insensitive(true),
            'm' => builder.multi_line(true),
            's' => builder.dot_matches_new_line(true),
            _ => {
                return Err(format!(
                    "E_REGEX_FLAGS_INVALID: 지원하지 않는 깃발 '{}'",
                    regex.flags
                )
                .into())
            }
        };
    }
    builder
        .build()
        .map_err(|e| format!("E_REGEX_PATTERN_INVALID: {}", e).into())
}

mod state_assertions;
use state_assertions::*;

fn build_regex_full_match(regex: &RegexLiteral) -> Result<regex::Regex, EvalError> {
    validate_regex_flags(&regex.flags)?;
    let wrapped = format!(r"\A(?:{})\z", regex.pattern);
    let mut builder = RegexBuilder::new(&wrapped);
    for ch in regex.flags.chars() {
        match ch {
            'i' => builder.case_insensitive(true),
            'm' => builder.multi_line(true),
            's' => builder.dot_matches_new_line(true),
            _ => {
                return Err(format!(
                    "E_REGEX_FLAGS_INVALID: 지원하지 않는 깃발 '{}'",
                    regex.flags
                )
                .into())
            }
        };
    }
    builder
        .build()
        .map_err(|e| format!("E_REGEX_PATTERN_INVALID: {}", e).into())
}

fn validate_regex_flags(flags: &str) -> Result<(), EvalError> {
    let mut seen_i = false;
    let mut seen_m = false;
    let mut seen_s = false;
    for ch in flags.chars() {
        match ch {
            'i' if !seen_i => seen_i = true,
            'm' if !seen_m => seen_m = true,
            's' if !seen_s => seen_s = true,
            _ => {
                return Err(format!("E_REGEX_FLAGS_INVALID: 지원하지 않는 깃발 '{}'", flags).into())
            }
        }
    }
    Ok(())
}

fn regex_is_full_match(text: &str, regex: &RegexLiteral) -> Result<bool, EvalError> {
    let re = build_regex_full_match(regex)?;
    Ok(re.is_match(text))
}

fn regex_find_first(text: &str, regex: &RegexLiteral) -> Result<Option<String>, EvalError> {
    let re = build_regex(regex)?;
    Ok(re.find(text).map(|m| m.as_str().to_string()))
}

fn regex_capture_first(text: &str, regex: &RegexLiteral) -> Result<Vec<String>, EvalError> {
    let re = build_regex(regex)?;
    let Some(captures) = re.captures(text) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::with_capacity(captures.len());
    for idx in 0..captures.len() {
        if let Some(hit) = captures.get(idx) {
            out.push(hit.as_str().to_string());
        } else {
            out.push(String::new());
        }
    }
    Ok(out)
}

fn regex_named_capture_first(
    text: &str,
    regex: &RegexLiteral,
) -> Result<BTreeMap<String, MapEntry>, EvalError> {
    let re = build_regex(regex)?;
    let Some(captures) = re.captures(text) else {
        return Ok(BTreeMap::new());
    };
    let mut out = BTreeMap::new();
    for name in re.capture_names().flatten() {
        let key = Value::String(name.to_string());
        let value = Value::String(
            captures
                .name(name)
                .map(|item| item.as_str().to_string())
                .unwrap_or_default(),
        );
        out.insert(map_key_canon(&key), MapEntry { key, value });
    }
    Ok(out)
}

fn regex_replacement_ref_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn validate_regex_replacement(re: &regex::Regex, replacement: &str) -> Result<(), EvalError> {
    let names: BTreeSet<&str> = re.capture_names().flatten().collect();
    let chars: Vec<char> = replacement.chars().collect();
    let mut idx = 0usize;
    while idx < chars.len() {
        if chars[idx] != '$' {
            idx += 1;
            continue;
        }
        if idx + 1 >= chars.len() {
            return Err(
                "E_REGEX_REPLACEMENT_INVALID: 치환 참조가 '$'에서 끝났습니다"
                    .to_string()
                    .into(),
            );
        }
        let next = chars[idx + 1];
        if next == '$' {
            idx += 2;
            continue;
        }
        if next == '{' {
            let mut end = idx + 2;
            while end < chars.len() && chars[end] != '}' {
                end += 1;
            }
            if end >= chars.len() {
                return Err(
                    "E_REGEX_REPLACEMENT_INVALID: 치환 참조의 닫는 '}'가 없습니다"
                        .to_string()
                        .into(),
                );
            }
            let token: String = chars[idx + 2..end].iter().collect();
            if token.is_empty() {
                return Err(
                    "E_REGEX_REPLACEMENT_INVALID: 빈 치환 참조는 사용할 수 없습니다"
                        .to_string()
                        .into(),
                );
            }
            if token.chars().all(|ch| ch.is_ascii_digit()) {
                let capture_index = token.parse::<usize>().unwrap_or(usize::MAX);
                if capture_index >= re.captures_len() {
                    return Err(format!(
                        "E_REGEX_REPLACEMENT_INVALID: 알 수 없는 캡처 번호입니다: {}",
                        token
                    )
                    .into());
                }
            } else if !names.contains(token.as_str()) {
                return Err(format!(
                    "E_REGEX_REPLACEMENT_INVALID: 알 수 없는 이름 캡처입니다: {}",
                    token
                )
                .into());
            }
            idx = end + 1;
            continue;
        }
        if next.is_ascii_digit() {
            let mut end = idx + 1;
            while end < chars.len() && chars[end].is_ascii_digit() {
                end += 1;
            }
            let token: String = chars[idx + 1..end].iter().collect();
            let capture_index = token.parse::<usize>().unwrap_or(usize::MAX);
            if capture_index >= re.captures_len() {
                return Err(format!(
                    "E_REGEX_REPLACEMENT_INVALID: 알 수 없는 캡처 번호입니다: {}",
                    token
                )
                .into());
            }
            idx = end;
            continue;
        }
        if regex_replacement_ref_name_char(next) {
            let mut end = idx + 1;
            while end < chars.len() && regex_replacement_ref_name_char(chars[end]) {
                end += 1;
            }
            let token: String = chars[idx + 1..end].iter().collect();
            if !names.contains(token.as_str()) {
                return Err(format!(
                    "E_REGEX_REPLACEMENT_INVALID: 알 수 없는 이름 캡처입니다: {}",
                    token
                )
                .into());
            }
            idx = end;
            continue;
        }
        return Err(format!(
            "E_REGEX_REPLACEMENT_INVALID: 지원하지 않는 치환 참조 시작입니다: ${}",
            next
        )
        .into());
    }
    Ok(())
}

fn regex_replace_all(
    text: &str,
    regex: &RegexLiteral,
    replacement: &str,
) -> Result<String, EvalError> {
    let re = build_regex(regex)?;
    validate_regex_replacement(&re, replacement)?;
    Ok(re.replace_all(text, replacement).to_string())
}

fn lambda_param_names(param: &str) -> Vec<String> {
    let names: Vec<String> = param
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect();
    if names.is_empty() {
        vec![param.trim().to_string()]
    } else {
        names
    }
}

fn regex_split(text: &str, regex: &RegexLiteral) -> Result<Vec<String>, EvalError> {
    let re = build_regex(regex)?;
    Ok(re.split(text).map(ToString::to_string).collect())
}

fn contains_unresolved_bogae_relation<'a>(
    value: &'a Value,
    meter: &mut WorkCostMeter,
) -> Result<bool, EvalError> {
    contains_unresolved_bogae_relation_for_builtin(value, meter, false)
}

fn contains_unresolved_bogae_relation_for_builtin<'a>(
    value: &'a Value,
    meter: &mut WorkCostMeter,
    ordered_scene_carrier: bool,
) -> Result<bool, EvalError> {
    // 원소 방문은 실패·조기 종료에서도 누적한다. 반복자 스택은 폭만큼의
    // 임시 목록이나 Rust 재귀 호출 없이 작성 순서대로 중첩 값을 방문한다.
    let mut stack: Vec<Box<dyn Iterator<Item = &'a Value> + 'a>> =
        vec![Box::new(std::iter::once(value))];
    while let Some(frame) = stack.last_mut() {
        let Some(value) = frame.next() else {
            stack.pop();
            continue;
        };
        meter.charge_element_visits(1)?;
        match value {
            Value::BogaeRelation(_) | Value::BogaeComposition(_) => return Ok(true),
            Value::Scene(_) | Value::SceneElement(_) | Value::View(ddonirang_lang::runtime::ViewValue::Scene(_)) =>
                if !ordered_scene_carrier { return Err("E_SCENE_OPERATION_UNSUPPORTED: 장면 값의 비교·열쇠 사용은 아직 지원하지 않습니다".to_string().into()); },
            Value::View(view) if matches!(view.without_time(), ddonirang_lang::runtime::ViewValue::Scene(_)) =>
                if !ordered_scene_carrier { return Err("E_SCENE_OPERATION_UNSUPPORTED: 장면 값의 비교·열쇠 사용은 아직 지원하지 않습니다".to_string().into()); },
            Value::List(items) => stack.push(Box::new(items.iter())),
            Value::Set(items) | Value::Pack(items) => stack.push(Box::new(items.values())),
            Value::Map(items) => stack.push(Box::new(items.values()
                .flat_map(|entry| [&entry.key, &entry.value]))),
            _ => {}
        }
    }
    Ok(false)
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Rational(a), Value::Rational(b)) => a == b,
        (Value::SumBranch {..},Value::SumBranch {..}) => left==right,
        (Value::Tuple(_),Value::Tuple(_)) => left == right,
        (Value::Tensor(a), Value::Tensor(b)) => a == b,
        (Value::Fixed64(a), Value::Fixed64(b)) => a.raw_i64() == b.raw_i64(),
        (Value::Fixed64(a), Value::Unit(b)) if b.is_dimensionless() => {
            a.raw_i64() == b.value.raw_i64()
        }
        (Value::Unit(a), Value::Fixed64(b)) if a.is_dimensionless() => {
            a.value.raw_i64() == b.raw_i64()
        }
        (Value::Unit(a), Value::Unit(b)) => {
            a.dim == b.dim && a.value.raw_i64() == b.value.raw_i64()
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::RegisteredAtom(a), Value::RegisteredAtom(b)) => a.owner_registry_identity == b.owner_registry_identity && a.registered_atom_identity == b.registered_atom_identity,
        (Value::IdentityReference(a),Value::IdentityReference(b)) => a==b,
        (Value::NominalValueV1(a),Value::NominalValueV1(b)) => a==b,
        (Value::FrozenSubjectViewV1(a),Value::FrozenSubjectViewV1(b)) => a==b,
        (Value::SubjectHandleV1(a),Value::SubjectHandleV1(b)) => a==b,
        (Value::Bytes(a), Value::Bytes(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::ResourceHandle(a), Value::ResourceHandle(b)) => a == b,
        (Value::None, Value::None) => true,
        (Value::Pack(a), Value::Pack(b)) => a == b,
        (Value::Record { field_types: a, fields: av }, Value::Record { field_types: b, fields: bv }) => a == b && av == bv,
        (Value::Assertion(a), Value::Assertion(b)) => a.canon == b.canon,
        (Value::StateMachine(a), Value::StateMachine(b)) => a == b,
        (Value::Formula(a), Value::Formula(b)) => a == b,
        (Value::Template(a), Value::Template(b)) => a.raw == b.raw && a.tag == b.tag,
        (Value::Regex(a), Value::Regex(b)) => a == b,
        (Value::Lambda(a), Value::Lambda(b)) => a.id == b.id,
        (Value::View(a), Value::View(b)) => a == b,
        (Value::TimePolicy(a),Value::TimePolicy(b))=>a==b,
        _ => false,
    }
}

fn value_rank(value: &Value) -> u8 {
    match value {
        Value::FrozenSubjectViewV1(_) => 30,
        Value::SumBranch {..} => 27,
        Value::Tuple(_) => 26,
        Value::RegisteredAtom(_) => 24,
        Value::IdentityReference(_) => 25,
        Value::NominalValueV1(_) => 28,
        Value::SubjectHandleV1(_) => 29,
        Value::Bytes(_) => 23,
        Value::Record { .. } => 22,
        Value::Tensor(_) => 21,
        Value::None => 0,
        Value::Bool(_) => 1,
        Value::Fixed64(_) | Value::Unit(_) | Value::Rational(_) => 2,
        Value::String(_) => 3,
        Value::ResourceHandle(_) => 4,
        Value::Formula(_) => 5,
        Value::Template(_) => 6,
        Value::Regex(_) => 7,
        Value::Assertion(_) => 8,
        Value::StateMachine(_) => 9,
        Value::Lambda(_) => 10,
        Value::View(_) => 15,
        Value::TimePolicy(_) => 20,
        Value::SceneElement(_) => 18,
        Value::Scene(_) => 19,
        Value::BogaeRelation(_) => 16,
        Value::BogaeComposition(_) => 17,
        Value::Pack(_) => {
            if numeric_pack_kind(value).is_some() {
                2
            } else {
                11
            }
        }
        Value::List(_) => 12,
        Value::Set(_) => 13,
        Value::Map(_) => 14,
    }
}

fn value_cmp(left: &Value, right: &Value) -> std::cmp::Ordering {
    let rank_left = value_rank(left);
    let rank_right = value_rank(right);
    if rank_left != rank_right {
        return rank_left.cmp(&rank_right);
    }
    match (left, right) {
        (Value::None, Value::None) => std::cmp::Ordering::Equal,
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        (Value::Fixed64(a), Value::Fixed64(b)) => a.raw_i64().cmp(&b.raw_i64()),
        (Value::Unit(a), Value::Unit(b)) => {
            if a.dim != b.dim {
                return a.dim.format().cmp(&b.dim.format());
            }
            a.value.raw_i64().cmp(&b.value.raw_i64())
        }
        (Value::Fixed64(a), Value::Unit(b)) if b.is_dimensionless() => {
            a.raw_i64().cmp(&b.value.raw_i64())
        }
        (Value::Unit(a), Value::Fixed64(b)) if a.is_dimensionless() => {
            a.value.raw_i64().cmp(&b.raw_i64())
        }
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::RegisteredAtom(a), Value::RegisteredAtom(b)) => (&a.owner_registry_identity, &a.registered_atom_identity).cmp(&(&b.owner_registry_identity, &b.registered_atom_identity)),
        (Value::IdentityReference(a),Value::IdentityReference(b)) => (&a.target_type_identity,&a.owner_space_identity,&a.target_identity,a.generation).cmp(&(&b.target_type_identity,&b.owner_space_identity,&b.target_identity,b.generation)),
        (Value::NominalValueV1(a),Value::NominalValueV1(b)) => a.canonical_body().cmp(b.canonical_body()),
        (Value::FrozenSubjectViewV1(a),Value::FrozenSubjectViewV1(b)) => a.canonical_body().cmp(b.canonical_body()),
        (Value::SubjectHandleV1(a),Value::SubjectHandleV1(b)) => (&a.name_type_identity,&a.world_identity,&a.subject_identity).cmp(&(&b.name_type_identity,&b.world_identity,&b.subject_identity)),
        (Value::Bytes(a), Value::Bytes(b)) => a.as_slice().cmp(b.as_slice()),
        (Value::ResourceHandle(a), Value::ResourceHandle(b)) => a.cmp(b),
        (Value::Formula(a), Value::Formula(b)) => a.raw.cmp(&b.raw),
        (Value::Template(a), Value::Template(b)) => a.raw.cmp(&b.raw),
        (Value::Regex(a), Value::Regex(b)) => a
            .pattern
            .cmp(&b.pattern)
            .then_with(|| a.flags.cmp(&b.flags)),
        (Value::Assertion(a), Value::Assertion(b)) => a.canon.cmp(&b.canon),
        (Value::StateMachine(a), Value::StateMachine(b)) => {
            state_machine_canon(a).cmp(&state_machine_canon(b))
        }
        (Value::Lambda(a), Value::Lambda(b)) => a.id.cmp(&b.id),
        (Value::Record { .. }, Value::Record { .. }) => map_key_canon(left).cmp(&map_key_canon(right)),
        _ => value_canon(left).cmp(&value_canon(right)),
    }
}

mod numeric_analysis;
use numeric_analysis::*;

mod numeric_values;
use numeric_values::{make_numeric_pack, numeric_pack_kind, numeric_pack_approx, value_to_fixed64_coerce, value_to_i64_strict, parse_bigint_text, factor_policy_summary_text, make_big_int_value, make_rational_value, make_factor_value, eval_exact_numeric_infix, eval_exact_numeric_compare_infix, is_exact_numeric_family_value, exact_numeric_from_value, exact_cmp, make_big_int_pack_from_bigint, make_rational_pack_from_big_rational};
#[cfg(test)]
use numeric_values::{swap_factor_pollard_max_iters_for_test, swap_factor_trial_fallback_limit_for_test};

mod relation_endpoints;
use relation_endpoints::{make_relation_pack, eval_endpoint_relation_list, eval_endpoint_relation_normalize, eval_endpoint_formula_relation_list, eval_endpoint_formula_relation_set, eval_endpoint_boundary_value_relation_list, eval_endpoint_boundary_value_injection, eval_endpoint_explicit_solve_with_meter, eval_endpoint_explicit_solve_range_violation_list_with_meter, eval_endpoint_explicit_solve_range_check_with_meter, eval_endpoint_solve_range_report_rows_with_meter, eval_endpoint_solve_range_report_with_meter, eval_endpoint_solve_range_text_report, eval_endpoint_explicit_solve_range_text_report_with_meter, eval_endpoint_solve_range_case_with_meter, eval_endpoint_solve_range_case_suite_with_meter, eval_endpoint_solve_range_case_suite_text, eval_endpoint_solve_range_case_suite_detail_text, eval_endpoint_solve_range_case_suite_run_detail_text_with_meter, eval_endpoint_solve_range_case_suite_summary, eval_endpoint_solve_range_case_suite_run_summary_with_meter, eval_endpoint_solve_range_case_suite_check, eval_endpoint_solve_range_case_suite_run_check_with_meter, endpoint_solve_range_case_suite_pack, endpoint_solve_range_case_suite_summary, endpoint_solve_range_case_suite_check, eval_endpoint_solve_result_value_list, eval_endpoint_solve_result_remap, endpoint_boundary_value_number, eval_endpoint_boundary_range_violation_list, eval_endpoint_boundary_range_check, expect_single_equation_relation, expect_equation_relations, relation_formula_text, eval_relation_solve_result_with_meter};
#[cfg(test)]
use relation_endpoints::{eval_endpoint_explicit_solve, eval_endpoint_explicit_solve_range_check, eval_endpoint_solve_range_report, eval_endpoint_solve_range_report_rows, eval_endpoint_explicit_solve_range_text_report, eval_endpoint_solve_range_case, eval_endpoint_solve_range_case_suite, eval_endpoint_solve_range_case_suite_run_detail_text, eval_relation_solve_result};

fn value_to_i64(value: &Value) -> Result<i64, EvalError> {
    match value {
        Value::Fixed64(n) => {
            if n.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(n.int_part())
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            if unit.value.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(unit.value.int_part())
        }
        Value::Pack(_) => {
            let Some(exact) = exact_numeric_from_value(value)? else {
                return Err("정수 값이 필요합니다".to_string().into());
            };
            if !exact.is_integer() {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            let integer = exact.value.to_integer();
            integer
                .to_i64()
                .ok_or_else(|| EvalError::from("정수 값(i64 범위)이 필요합니다".to_string()))
        }
        _ => Err("정수 값이 필요합니다".to_string().into()),
    }
}

fn value_to_index(value: &Value) -> Result<usize, EvalError> {
    match value {
        Value::Fixed64(n) => {
            let idx = n.int_part();
            if idx < 0 {
                Err("인덱스는 0 이상이어야 합니다".to_string().into())
            } else {
                Ok(idx as usize)
            }
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            let idx = unit.value.int_part();
            if idx < 0 {
                Err("인덱스는 0 이상이어야 합니다".to_string().into())
            } else {
                Ok(idx as usize)
            }
        }
        _ => Err("인덱스는 정수여야 합니다".to_string().into()),
    }
}

fn value_to_tensor_index(value: &Value) -> Result<usize, EvalError> {
    // RD-388: never silently truncate a fractional or negative coordinate.
    let number = match value {
        Value::Fixed64(n) => *n,
        Value::Unit(n) if n.is_dimensionless() => n.value,
        _ => return Err("E_TENSOR_SELECTION: 축 번호는 0 이상의 정수여야 합니다".to_string().into()),
    };
    if number.raw_i64() < 0 || number.frac_part() != 0 {
        return Err("E_TENSOR_SELECTION: 축 번호는 0 이상의 정수여야 합니다".to_string().into());
    }
    usize::try_from(number.int_part()).map_err(|_| "E_TENSOR_SELECTION: 축 번호 초과".to_string().into())
}

const STREAM_V1_SCHEMA: &str = "ddn.stream.v1";
const STREAM_CAPACITY_MAX: usize = 1_000_000;
const STREAM_VALUE_LABEL: &str = "흐름(흐름.만들기 결과)";

#[derive(Clone)]
struct RuntimeStream {
    capacity: usize,
    head: usize,
    len: usize,
    buffer: Vec<Value>,
}

fn stream_map_lookup<'a>(entries: &'a BTreeMap<String, MapEntry>, key: &str) -> Option<&'a Value> {
    let key_canon = map_key_canon(&Value::String(key.to_string()));
    entries.get(&key_canon).map(|entry| &entry.value)
}

fn stream_parse_u64(value: &Value, field: &str) -> Result<u64, EvalError> {
    match value {
        Value::Fixed64(n) => {
            if n.frac_part() != 0 {
                return Err(format!("흐름.{}는 정수여야 합니다", field).into());
            }
            let v = n.int_part();
            if v < 0 {
                return Err(format!("흐름.{}는 0 이상이어야 합니다", field).into());
            }
            Ok(v as u64)
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            if unit.value.frac_part() != 0 {
                return Err(format!("흐름.{}는 정수여야 합니다", field).into());
            }
            let v = unit.value.int_part();
            if v < 0 {
                return Err(format!("흐름.{}는 0 이상이어야 합니다", field).into());
            }
            Ok(v as u64)
        }
        Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
        Value::String(v) => v
            .parse::<u64>()
            .map_err(|_| format!("흐름.{}는 정수 문자열이어야 합니다", field).into()),
        _ => Err(format!("흐름.{}는 정수 값이어야 합니다", field).into()),
    }
}

fn stream_from_value(value: &Value) -> Result<RuntimeStream, EvalError> {
    let Value::Map(entries) = value else {
        return Err(format!("{} 인자가 필요합니다", STREAM_VALUE_LABEL).into());
    };
    let Some(Value::String(schema)) = stream_map_lookup(entries, "__schema") else {
        return Err(format!("{}에 __schema가 없습니다", STREAM_VALUE_LABEL).into());
    };
    if schema != STREAM_V1_SCHEMA {
        return Err(format!(
            "지원하지 않는 흐름 schema: {} (기대값: {})",
            schema, STREAM_V1_SCHEMA
        )
        .into());
    }
    let capacity_raw = match stream_map_lookup(entries, "capacity") {
        Some(value) => stream_parse_u64(value, "용량")?,
        None => 0,
    };
    let head_raw = match stream_map_lookup(entries, "head") {
        Some(value) => Some(stream_parse_u64(value, "머리")?),
        None => None,
    };
    let Some(Value::List(buffer_raw)) = stream_map_lookup(entries, "buffer") else {
        return Err(format!("{}에 buffer 차림이 없습니다", STREAM_VALUE_LABEL).into());
    };
    let len_raw = match stream_map_lookup(entries, "len") {
        Some(value) => stream_parse_u64(value, "길이")?,
        None => buffer_raw.len() as u64,
    };
    let mut capacity_u64 = capacity_raw.max(buffer_raw.len() as u64);
    if capacity_u64 > STREAM_CAPACITY_MAX as u64 {
        return Err(format!(
            "흐름 용량이 너무 큽니다 (최대 {}): {}",
            STREAM_CAPACITY_MAX, capacity_u64
        )
        .into());
    }
    let capacity = capacity_u64 as usize;
    let mut buffer = buffer_raw.clone();
    if buffer.len() < capacity {
        buffer.resize(capacity, Value::None);
    } else if buffer.len() > capacity {
        buffer.truncate(capacity);
        capacity_u64 = capacity as u64;
    }
    let len = len_raw.min(capacity_u64) as usize;
    let head = if capacity == 0 {
        0
    } else {
        let default_head = len.saturating_sub(1) as u64;
        head_raw
            .unwrap_or(default_head)
            .min(capacity_u64.saturating_sub(1)) as usize
    };
    Ok(RuntimeStream {
        capacity,
        head,
        len,
        buffer,
    })
}

fn stream_to_value(stream: &RuntimeStream) -> Value {
    let mut entries = BTreeMap::new();
    let insert = |entries: &mut BTreeMap<String, MapEntry>, key: &str, value: Value| {
        let key_value = Value::String(key.to_string());
        entries.insert(
            map_key_canon(&key_value),
            MapEntry {
                key: key_value,
                value,
            },
        );
    };
    insert(
        &mut entries,
        "__schema",
        Value::String(STREAM_V1_SCHEMA.to_string()),
    );
    insert(
        &mut entries,
        "capacity",
        Value::Fixed64(Fixed64::from_i64(
            stream.capacity.min(i64::MAX as usize) as i64
        )),
    );
    insert(
        &mut entries,
        "head",
        Value::Fixed64(Fixed64::from_i64(stream.head.min(i64::MAX as usize) as i64)),
    );
    insert(
        &mut entries,
        "len",
        Value::Fixed64(Fixed64::from_i64(stream.len.min(i64::MAX as usize) as i64)),
    );
    insert(&mut entries, "buffer", Value::List(stream.buffer.clone()));
    Value::Map(entries)
}

fn stream_new(capacity: usize) -> Result<RuntimeStream, EvalError> {
    if capacity == 0 {
        return Err("흐름 용량은 1 이상이어야 합니다".to_string().into());
    }
    if capacity > STREAM_CAPACITY_MAX {
        return Err(format!(
            "흐름 용량이 너무 큽니다 (최대 {}): {}",
            STREAM_CAPACITY_MAX, capacity
        )
        .into());
    }
    Ok(RuntimeStream {
        capacity,
        head: 0,
        len: 0,
        buffer: vec![Value::None; capacity],
    })
}

fn stream_push(mut stream: RuntimeStream, value: Value) -> RuntimeStream {
    if stream.capacity == 0 {
        return stream;
    }
    let next_head = if stream.len == 0 {
        0
    } else {
        (stream.head + 1) % stream.capacity
    };
    if stream.buffer.len() < stream.capacity {
        stream.buffer.resize(stream.capacity, Value::None);
    }
    stream.buffer[next_head] = value;
    stream.head = next_head;
    stream.len = stream.len.saturating_add(1).min(stream.capacity);
    stream
}

fn stream_clear(mut stream: RuntimeStream) -> RuntimeStream {
    if stream.buffer.len() < stream.capacity {
        stream.buffer.resize(stream.capacity, Value::None);
    }
    for slot in &mut stream.buffer {
        *slot = Value::None;
    }
    stream.head = 0;
    stream.len = 0;
    stream
}

fn stream_oldest_to_newest(stream: &RuntimeStream) -> Vec<Value> {
    if stream.capacity == 0 || stream.len == 0 {
        return Vec::new();
    }
    let oldest = (stream.head + stream.capacity + 1 - stream.len) % stream.capacity;
    let mut out = Vec::with_capacity(stream.len);
    for offset in 0..stream.len {
        let idx = (oldest + offset) % stream.capacity;
        out.push(stream.buffer.get(idx).cloned().unwrap_or(Value::None));
    }
    out
}

mod value_bridge;
pub(crate) use value_bridge::{value_to_string, value_to_display_text, formula_from_resource_value, formula_summary_parts, formula_display_string, resource_value_from_typed_value, resource_value_from_typed_value_with_output, value_from_resource_value};
use value_bridge::{format_value_text, format_value_leaf_text, state_machine_canon, resource_map_to_state_machine, runtime_value_contains_tuple, resource_value_from_value, public_observation_type, dataset_public_observation_scalar, canonical_dataset_observation_name, value_canon, render_template};

#[cfg(test)]
#[test]
fn rd694_saved_display_heap_jobs_preserve_existing_spellings() {
    let fields = BTreeMap::from([("a".into(),Value::Fixed64(Fixed64::from_ratio(1,3))),
        ("b".into(),Value::List(vec![Value::Bool(true),Value::String("한글".into())]))]);
    let cases = vec![Value::List(fields.values().cloned().collect()),Value::Pack(fields.clone()),
        Value::Record{field_types:BTreeMap::new(),fields},
        Value::Pack(BTreeMap::from([(RELATION_KIND_FIELD.into(),Value::String(RELATION_KIND_EQUATION.into())),
            (RELATION_LEFT_FIELD.into(),Value::List(vec![Value::Fixed64(Fixed64::ONE)])),
            (RELATION_RIGHT_FIELD.into(),Value::Bool(false))])),
        Value::Pack(BTreeMap::from([(RELATION_SOLVE_RESULT_KIND_FIELD.into(),Value::String(RELATION_SOLVE_RESULT_SUCCESS.into())),
            (RELATION_SOLVE_VAR_FIELD.into(),Value::String("x".into())),
            (RELATION_SOLVE_VALUE_FIELD.into(),Value::List(vec![Value::Fixed64(Fixed64::ONE)]))])),
        Value::Pack(BTreeMap::from([(RELATION_SOLVE_RESULT_KIND_FIELD.into(),Value::String(RELATION_SOLVE_RESULT_SUCCESS.into())),
            (RELATION_SOLVE_BINDINGS_FIELD.into(),Value::Pack(BTreeMap::from([("x".into(),Value::Bool(true))])))])),
        Value::Pack(BTreeMap::from([(RELATION_SOLVE_RESULT_KIND_FIELD.into(),Value::String(RELATION_SOLVE_RESULT_FAILURE.into()))]))];
    for value in cases {for display in [false,true] {
        assert_eq!(format_value_text(&value,display),format_value_leaf_text(&value,display));
    }}
}

#[cfg(test)]
#[test]
fn rd694_saved_conversion_and_automatic_cleanup_use_heap_traversal() {
    std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
        use ddonirang_core::platform::RecordFieldType as R;
        let mut key = ResourceValue::None;
        let mut kind = R::Named { name: "셈수".into() };
        let mut field = ResourceValue::Fixed64(Fixed64::ZERO);
        for _ in 0..20_000 {
            key = ResourceValue::List(vec![key]);
            field = ResourceValue::List(vec![field]);
            kind = R::Applied { name: "차림".into(), args: vec![kind] };
        }
        let saved = ResourceValue::Map(BTreeMap::from([("stored-index".into(), ResourceMapEntry {
            key, value: ResourceValue::Record {
                field_types: BTreeMap::from([("a".into(), kind)]),
                fields: BTreeMap::from([("a".into(), field)]),
            },
        })]));
        let restored = value_from_resource_value(&saved);
        let Value::Map(entries) = &restored else { panic!("map") };
        let entry = &entries["stored-index"];
        let admitted = ddonirang_lang::runtime::validate_record_value(&entry.value);
        let shown_key = value_to_display_text(&entry.key);
        let shown_field = value_to_display_text(&entry.value);
        assert_eq!(shown_key.matches("차림[").count(),20_000);
        assert_eq!(shown_field.matches("차림[").count(),20_000);
        assert!(shown_field.starts_with("(a: 차림["));
        let mut leaf = &entry.key;
        let mut depth = 0;
        while let Value::List(items) = leaf { depth += 1; leaf = &items[0]; }
        let none = matches!(leaf, Value::None);
        // Real restored runtime values must remain safe when project staging
        // copies them before saving again, including record type identities.
        let copied = restored.clone();
        let same_runtime = restored == copied;
        let serialized = resource_value_from_value(&copied);
        let identical = serialized.as_ref().is_ok_and(|value| value == &saved);
        drop(serialized);
        drop(copied);
        drop(restored);
        drop(saved);
        assert!(admitted.is_ok());
        assert_eq!(depth, 20_000);
        assert!(none);
        assert!(same_runtime);
        assert!(identical);
    }).unwrap().join().unwrap();
}

#[cfg(test)]
#[test]
fn rd694_failed_save_cleans_completed_siblings_and_pending_map_keys() {
    std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
        for map in [false, true] {
            let mut deep = Value::None;
            for _ in 0..20_000 { deep = Value::List(vec![deep]); }
            let value = if map {
                Value::Map(BTreeMap::from([("index".into(), MapEntry {
                    key: deep, value: Value::View(ddonirang_lang::runtime::ViewValue::ResultTable),
                })]))
            } else {
                Value::List(vec![deep, Value::View(ddonirang_lang::runtime::ViewValue::ResultTable)])
            };
            let rejected = resource_value_from_value(&value).err().map(|error| error.to_string());
            drop(value);
            assert_eq!(rejected.as_deref(), Some("보기값은 세계 자원에 대입할 수 없습니다"));
        }
        let value = Value::Map(BTreeMap::from([("index".into(), MapEntry {
            key: Value::View(ddonirang_lang::runtime::ViewValue::ResultTable),
            value: Value::TimePolicy(ddonirang_lang::runtime::BogaeTimePolicy::Common),
        })]));
        assert_eq!(resource_value_from_value(&value).err().map(|error| error.to_string()).as_deref(),
            Some("보기값은 세계 자원에 대입할 수 없습니다"));
    }).unwrap().join().unwrap();
}

#[cfg(test)]
#[test]
fn rd694_malformed_state_machine_does_not_clone_deep_rejected_members() {
    std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
        for field in ["states", "checks", "transitions"] {
            let mut entries = BTreeMap::new();
            for (name, value) in [
                ("__ddn_kind", ResourceValue::String(STATE_MACHINE_RESOURCE_KIND.into())),
                ("states", ResourceValue::List(vec![ResourceValue::String("ready".into())])),
                ("initial", ResourceValue::String("ready".into())),
                ("checks", ResourceValue::List(vec![])),
                ("transitions", ResourceValue::List(vec![])),
            ] {
                entries.insert(name.into(), ResourceMapEntry { key: ResourceValue::String(name.into()), value });
            }
            let valid = resource_map_to_state_machine(&entries).is_some();
            let mut deep = ResourceValue::None;
            for _ in 0..20_000 { deep = ResourceValue::List(vec![deep]); }
            entries.get_mut(field).unwrap().value = ResourceValue::List(vec![deep]);
            let rejected = resource_map_to_state_machine(&entries).is_none();
            let saved = ResourceValue::Map(entries);
            let restored = value_from_resource_value(&saved);
            let map = matches!(&restored, Value::Map(_));
            drop(restored);
            drop(saved);
            assert!(valid && rejected && map);
        }
    }).unwrap().join().unwrap();
}

#[cfg(test)]
#[test]
fn grid_standard_records_survive_committed_resource_roundtrip() {
    let nested = Value::Pack(BTreeMap::from([
        ("__종류".into(), Value::String(STD_FALLING_PIECE_KIND.into())),
        ("위치".into(), Value::List(vec![fixed_value(3), fixed_value(2)])),
    ]));
    let session = Value::Pack(BTreeMap::from([
        ("__종류".into(), Value::String(STD_GRID_GAME_SESSION_KIND.into())),
        ("낙하조각".into(), nested),
    ]));
    let restored = value_from_resource_value(&resource_value_from_value(&session).unwrap());
    assert_eq!(value_canon(&restored), value_canon(&session));
    assert!(matches!(restored, Value::Pack(_)));
    let ordinary = Value::Pack(BTreeMap::from([("__종류".into(), Value::String("unknown".into()))]));
    assert!(matches!(value_from_resource_value(&resource_value_from_value(&ordinary).unwrap()), Value::Map(_)));
    for (key, kind) in [("__종류", STD_GRID_KIND), ("__kind", STD_GRID_GAME_SESSION_KIND)] {
        let wrong = Value::Pack(BTreeMap::from([(key.into(), Value::String(kind.into()))]));
        assert!(matches!(value_from_resource_value(&resource_value_from_value(&wrong).unwrap()), Value::Map(_)));
    }
}

// F4: one selected builtin branch owns its scratch, not every authored call.
// Successful early-return arms stay outside this helper to preserve post-charge bypass.
#[inline(never)]
fn in_call_arm_frame<F>(arm: F) -> Result<Value, EvalError>
where
    F: FnOnce() -> Result<Value, EvalError>,
{
    arm()
}

mod builtin_calls;
mod expression_execution;
mod signal_execution;
mod bogae_projection;
mod statement_execution;
mod formula_values;
use formula_values::{
    ParsedFormula,
    FormulaTransformOptions,
    FormulaExpr,
    FormulaOp,
    parse_formula_value,
    analyze_formula_for_transform,
    format_formula_body,
    format_formula_expr,
    expect_formula_transform,
    expect_single_formula,
    expect_two_formulas,
    expect_int_strict,
    transform_formula_value,
    symbolic_formula_error,
    symbolic_formulas_equivalent,
    verify_symbolic_assertion,
    parse_symbolic_equivalence_assertion,
    parse_symbolic_relation_assertion,
    solve_bindings_from_hashmap,
    eval_symbolic_proof_tactic_with_meter,
    num_ln10,
    num_ln2,
    func,
    add,
    sub,
    mul,
    div,
    pow,
    is_zero,
    ensure_formula_ident,
    eval_formula_expr,
    eval_formula_func,
    eval_formula_pow,
};

fn is_truthy(value: &Value) -> Result<bool, EvalError> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err("E_RUNTIME_CONDITION_TYPE: 조건은 참거짓이어야 합니다".to_string().into()),
    }
}

fn parse_bogae_shape_call(entry: &str) -> Option<BogaeShapeCall> {
    let raw = entry.trim().trim_end_matches('.').trim();
    if raw.is_empty() {
        return None;
    }
    let open = raw.find('(')?;
    let close = raw.rfind(')')?;
    if close <= open || !raw[close + 1..].trim().is_empty() {
        return None;
    }
    let name = raw[..open].trim();
    let kind = match name {
        "선" | "line" => BogaeShapeKind::Line,
        "원" | "circle" => BogaeShapeKind::Circle,
        "점" | "point" => BogaeShapeKind::Point,
        "사각형" | "rect" => BogaeShapeKind::Rect,
        "글" | "text" => BogaeShapeKind::Text,
        "꺾은선" | "polyline" => BogaeShapeKind::Polyline,
        "다각형" | "polygon" => BogaeShapeKind::Polygon,
        _ => return None,
    };

    let mut positional = Vec::new();
    let mut named = BTreeMap::new();
    for token in split_top_level_args(&raw[open + 1..close]) {
        if let Some((key, value)) = split_named_arg(&token) {
            let lower = key.to_lowercase();
            named.insert(key, value.clone());
            if !lower.is_empty() {
                named.insert(lower, value);
            }
        } else {
            positional.push(token);
        }
    }

    Some(BogaeShapeCall {
        kind,
        positional,
        named,
    })
}

fn bogae_pick_arg<'a>(shape: &'a BogaeShapeCall, keys: &[&str], idx: usize) -> Option<&'a str> {
    for key in keys {
        if let Some(value) = shape.named.get(*key) {
            return Some(value.as_str());
        }
        let lower = key.to_lowercase();
        if let Some(value) = shape.named.get(&lower) {
            return Some(value.as_str());
        }
    }
    if idx == usize::MAX {
        return None;
    }
    shape.positional.get(idx).map(|value| value.as_str())
}

fn split_top_level_args(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if let Some(active) = quote {
            if ch == active && (i == 0 || chars[i - 1] != '\\') {
                quote = None;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
            }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                let token: String = chars[start..i].iter().collect();
                let trimmed = token.trim();
                if !trimmed.is_empty() {
                    out.push(trimmed.to_string());
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let token: String = chars[start..].iter().collect();
    let trimmed = token.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_string());
    }
    out
}

fn split_named_arg(token: &str) -> Option<(String, String)> {
    let chars: Vec<char> = token.chars().collect();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    for (i, ch) in chars.iter().enumerate() {
        if let Some(active) = quote {
            if *ch == active && (i == 0 || chars[i - 1] != '\\') {
                quote = None;
            }
            continue;
        }
        match *ch {
            '"' | '\'' => quote = Some(*ch),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '=' if depth == 0 => {
                let key: String = chars[..i].iter().collect();
                let value: String = chars[i + 1..].iter().collect();
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                if key.is_empty() || value.is_empty() {
                    return None;
                }
                return Some((key, value));
            }
            _ => {}
        }
    }
    None
}

fn strip_wrapping_quotes(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.len() < 2 {
        return None;
    }
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return Some(trimmed[1..trimmed.len() - 1].to_string());
    }
    None
}

fn bogae_point_value(x: Value, y: Value) -> Value {
    let mut map = BTreeMap::new();
    let x_key = Value::String("x".to_string());
    map.insert(
        map_key_canon(&x_key),
        MapEntry {
            key: x_key,
            value: x,
        },
    );
    let y_key = Value::String("y".to_string());
    map.insert(
        map_key_canon(&y_key),
        MapEntry {
            key: y_key,
            value: y,
        },
    );
    Value::Map(map)
}

fn bogae_points_from_positional(
    shape: &BogaeShapeCall,
    locals: &mut HashMap<String, Value>,
    ctx: &mut EvalContext<'_>,
) -> Option<Value> {
    if shape.positional.len() < 4 || shape.positional.len() % 2 != 0 {
        return None;
    }
    let mut points = Vec::new();
    let mut idx = 0usize;
    while idx + 1 < shape.positional.len() {
        let x = ctx.parse_bogae_value_token(locals, &shape.positional[idx])?;
        let y = ctx.parse_bogae_value_token(locals, &shape.positional[idx + 1])?;
        points.push(bogae_point_value(x, y));
        idx += 2;
    }
    Some(Value::List(points))
}

fn bogae_read_arg(
    ctx: &mut EvalContext<'_>,
    locals: &mut HashMap<String, Value>,
    shape: &BogaeShapeCall,
    keys: &[&str],
    idx: usize,
    default: Value,
) -> Value {
    bogae_pick_arg(shape, keys, idx)
        .and_then(|text| ctx.parse_bogae_value_token(locals, text))
        .unwrap_or(default)
}

fn runtime_error(err: RuntimeError) -> EvalError {
    EvalError::Message(runtime_error_message(err))
}

fn runtime_error_message(err: RuntimeError) -> String {
    match err {
        RuntimeError::TypeMismatch { expected } => format!("타입 오류: {}", expected),
        RuntimeError::IndexOutOfRange => "FATAL:CHARIM_INDEX_OUT_OF_RANGE".to_string(),
    }
}

fn type_mismatch_error(pin: &str, expected: &str, actual: &str) -> EvalError {
    EvalError::TypeMismatch {
        pin: pin.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    }
}

pub(crate) fn format_parse_error(source: &str, err: &ParseError) -> String {
    let mut line = 1usize;
    let mut col = 1usize;
    let mut line_start = 0usize;
    for (idx, ch) in source.char_indices() {
        if idx >= err.span.start {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
            line_start = idx + ch.len_utf8();
        } else {
            col += 1;
        }
    }
    let line_end = source[line_start..]
        .find('\n')
        .map(|offset| line_start + offset)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];
    let caret = " ".repeat(col.saturating_sub(1)) + "^";
    format!(
        "파싱 실패: {} ({}:{})\n{}\n{}",
        err.message, line, col, line_text, caret
    )
}

fn position_to_line_col(source: &str, byte_pos: usize) -> (u32, u32) {
    let target = byte_pos.min(source.len());
    let mut line = 1u32;
    let mut col = 0u32;
    let mut idx = 0usize;
    for ch in source.chars() {
        if idx >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
        idx += ch.len_utf8();
    }
    (line, col)
}

#[cfg(test)]
mod tests;
