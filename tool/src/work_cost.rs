//! WorkCostV1 결정적 의미 비용과 `#표준한도_v1` 후보 한도.
//!
//! 벽시계, 호스트 부하, FPS는 이 모듈의 입력이 아니다. RD186 slice 한도는
//! 영수증에 계측만 하고, sealed continuation이 생기기 전까지 집행하지 않는다.

use std::collections::BTreeSet;

use ddonirang_lang::model::EventDetectionPolicy;
use ddonirang_lang::{
    ConnectionMode, ContractKind, ContractMode, ExprKind, HookKind, Literal, LoweredRelationTarget,
    ModelBodyItem, ModelEventKind, ModelExprNode, QuantifierKind, Stmt, ThunkEvalMode,
};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const WORK_COST_SCHEMA: &str = "ddonirang.work_cost.v1-candidate";
pub const STANDARD_LIMIT_SCHEMA: &str = "ddonirang.standard_limit.v1-candidate";
pub const WORK_COST_BASE_DISPATCH: u64 = 1;

pub const E_WORK_COST_REGISTRY_MISSING_CANDIDATE: &str = "E_WORK_COST_REGISTRY_MISSING_CANDIDATE";
pub const E_WORK_COST_OVERFLOW_CANDIDATE: &str = "E_WORK_COST_OVERFLOW_CANDIDATE";
pub const E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE: &str =
    "E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE";
pub const E_STANDARD_RESOURCE_PREFLIGHT_REJECTED_CANDIDATE: &str =
    "E_STANDARD_RESOURCE_PREFLIGHT_REJECTED_CANDIDATE";

/// RD186 권고값을 Owner 승인에 따라 후보 상수로 결속한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandardResourceLimitsV1 {
    /// 계측 전용. 이 W1에서는 집행하지 않는다.
    pub slice_committed_madi: u64,
    /// 계측 전용. 이 W1에서는 집행하지 않는다.
    pub slice_work_cost: u64,
    /// 계측 전용. 이 W1에서는 집행하지 않는다.
    pub slice_solver_evaluations: u64,
    /// 계측 전용. 이 W1에서는 집행하지 않는다.
    pub slice_external_events: u64,
    pub call_depth: u64,
    pub created_identities: u64,
    pub public_observation_samples: u64,
    pub sealed_input_bytes: u64,
    pub result_artifact_bytes: u64,
    pub replay_capsule_bytes: u64,
}

impl StandardResourceLimitsV1 {
    pub const CANDIDATE: Self = Self {
        slice_committed_madi: 100_000,
        slice_work_cost: 100_000_000,
        slice_solver_evaluations: 1_000_000,
        slice_external_events: 100_000,
        call_depth: 512,
        created_identities: 100_000,
        public_observation_samples: 2_000_000,
        sealed_input_bytes: 67_108_864,
        result_artifact_bytes: 268_435_456,
        replay_capsule_bytes: 268_435_456,
    };
}

impl Default for StandardResourceLimitsV1 {
    fn default() -> Self {
        Self::CANDIDATE
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkCostCounters {
    pub work_cost: u64,
    pub base_dispatches: u64,
    pub collection_tensor_element_visits: u64,
    pub canonical_utf8_byte_visits: u64,
    pub bigint_rational_signed_magnitude_byte_visits: u64,
    pub solver_evaluations: u64,
    pub external_events: u64,
    pub committed_madi: u64,
    pub call_depth_current: u64,
    pub call_depth_peak: u64,
    pub created_identities: u64,
    pub public_observation_samples: u64,
    pub sealed_input_bytes: u64,
    pub result_artifact_bytes: u64,
    pub replay_capsule_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkCostReceipt {
    pub schema: &'static str,
    pub standard_limit_schema: &'static str,
    pub counters: WorkCostCounters,
    pub limits: StandardResourceLimitsV1,
    pub slice_limits_enforced: bool,
    pub continuation_present: bool,
    pub semantic_memory_limit_present: bool,
    pub independent_result_row_limit_present: bool,
    pub rd206_resource_capability: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkCostFailure {
    pub code: &'static str,
    pub resource: String,
    pub actual: u64,
    pub limit: Option<u64>,
    pub receipt: WorkCostReceipt,
}

impl std::fmt::Display for WorkCostFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: resource={}, actual={}, limit={}",
            self.code,
            self.resource,
            self.actual,
            self.limit
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        )
    }
}

impl std::error::Error for WorkCostFailure {}

#[derive(Debug, Clone)]
pub struct WorkCostMeter {
    counters: WorkCostCounters,
    limits: StandardResourceLimitsV1,
    rd206_resource_capability: bool,
}

impl Default for WorkCostMeter {
    fn default() -> Self {
        Self::new(StandardResourceLimitsV1::CANDIDATE)
    }
}

impl WorkCostMeter {
    pub fn new(limits: StandardResourceLimitsV1) -> Self {
        Self {
            counters: WorkCostCounters::default(),
            limits,
            rd206_resource_capability: false,
        }
    }

    pub(crate) fn from_receipt_parts(
        counters: WorkCostCounters,
        limits: StandardResourceLimitsV1,
        rd206_resource_capability: bool,
    ) -> Self {
        Self {
            counters,
            limits,
            rd206_resource_capability,
        }
    }

    /// RD-206의 표준 profile 제품 경로가 이 meter를 공통 WorkCostV1 owner로
    /// 실제 사용한다고 확인한 뒤에만 호출한다. 일반 runtime의 기본값은 false다.
    pub fn authorize_rd206_resource_capability(&mut self) {
        self.rd206_resource_capability = true;
    }

    pub fn receipt(&self) -> WorkCostReceipt {
        WorkCostReceipt {
            schema: WORK_COST_SCHEMA,
            standard_limit_schema: STANDARD_LIMIT_SCHEMA,
            counters: self.counters.clone(),
            limits: self.limits,
            slice_limits_enforced: false,
            continuation_present: false,
            semantic_memory_limit_present: false,
            independent_result_row_limit_present: false,
            rd206_resource_capability: self.rd206_resource_capability,
        }
    }

    fn failure(
        &self,
        code: &'static str,
        resource: impl Into<String>,
        actual: u64,
        limit: Option<u64>,
    ) -> WorkCostFailure {
        WorkCostFailure {
            code,
            resource: resource.into(),
            actual,
            limit,
            receipt: self.receipt(),
        }
    }

    fn checked_add(target: &mut u64, amount: u64) -> Option<u64> {
        let next = target.checked_add(amount)?;
        *target = next;
        Some(next)
    }

    pub fn charge_dispatch(&mut self) -> Result<(), WorkCostFailure> {
        let next_dispatches = self
            .counters
            .base_dispatches
            .checked_add(1)
            .ok_or_else(|| {
                self.failure(
                    E_WORK_COST_OVERFLOW_CANDIDATE,
                    "base_dispatches",
                    u64::MAX,
                    None,
                )
            })?;
        let next_work = self
            .counters
            .work_cost
            .checked_add(WORK_COST_BASE_DISPATCH)
            .ok_or_else(|| {
                self.failure(
                    E_WORK_COST_OVERFLOW_CANDIDATE,
                    "base_dispatch",
                    u64::MAX,
                    None,
                )
            })?;
        self.counters.base_dispatches = next_dispatches;
        self.counters.work_cost = next_work;
        Ok(())
    }

    pub fn charge_element_visits(&mut self, visits: usize) -> Result<(), WorkCostFailure> {
        let visits = u64::try_from(visits).map_err(|_| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "collection_tensor_element_visits",
                u64::MAX,
                None,
            )
        })?;
        let next_visits = self
            .counters
            .collection_tensor_element_visits
            .checked_add(visits)
            .ok_or_else(|| {
                self.failure(
                    E_WORK_COST_OVERFLOW_CANDIDATE,
                    "collection_tensor_element_visits",
                    u64::MAX,
                    None,
                )
            })?;
        let next_work = self.counters.work_cost.checked_add(visits).ok_or_else(|| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "collection_tensor_element_visits",
                u64::MAX,
                None,
            )
        })?;
        self.counters.collection_tensor_element_visits = next_visits;
        self.counters.work_cost = next_work;
        Ok(())
    }

    pub fn charge_canonical_utf8_bytes(&mut self, text: &str) -> Result<(), WorkCostFailure> {
        let visits = u64::try_from(text.len()).map_err(|_| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "canonical_utf8_byte_visits",
                u64::MAX,
                None,
            )
        })?;
        let next_visits = self
            .counters
            .canonical_utf8_byte_visits
            .checked_add(visits)
            .ok_or_else(|| {
                self.failure(
                    E_WORK_COST_OVERFLOW_CANDIDATE,
                    "canonical_utf8_byte_visits",
                    u64::MAX,
                    None,
                )
            })?;
        let next_work = self.counters.work_cost.checked_add(visits).ok_or_else(|| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "canonical_utf8_byte_visits",
                u64::MAX,
                None,
            )
        })?;
        self.counters.canonical_utf8_byte_visits = next_visits;
        self.counters.work_cost = next_work;
        Ok(())
    }

    /// 부호 한 바이트와 최소 big-endian magnitude 바이트를 센다. 0도 부호
    /// 한 바이트를 가진다.
    pub fn canonical_signed_magnitude_bytes(value: &BigInt) -> u64 {
        let (_, magnitude) = value.to_bytes_be();
        1u64.saturating_add(magnitude.len() as u64)
    }

    pub fn charge_bigint_rational_primitive(
        &mut self,
        operands: &[&BigInt],
        results: &[&BigInt],
    ) -> Result<(), WorkCostFailure> {
        let amount = operands
            .iter()
            .chain(results.iter())
            .try_fold(0u64, |sum, value| {
                sum.checked_add(Self::canonical_signed_magnitude_bytes(value))
            })
            .ok_or_else(|| {
                self.failure(
                    E_WORK_COST_OVERFLOW_CANDIDATE,
                    "bigint_rational_signed_magnitude_byte_visits",
                    u64::MAX,
                    None,
                )
            })?;
        let next_visits = self
            .counters
            .bigint_rational_signed_magnitude_byte_visits
            .checked_add(amount)
            .ok_or_else(|| {
                self.failure(
                    E_WORK_COST_OVERFLOW_CANDIDATE,
                    "bigint_rational_signed_magnitude_byte_visits",
                    u64::MAX,
                    None,
                )
            })?;
        let next_work = self.counters.work_cost.checked_add(amount).ok_or_else(|| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "bigint_rational_signed_magnitude_byte_visits",
                u64::MAX,
                None,
            )
        })?;
        self.counters.bigint_rational_signed_magnitude_byte_visits = next_visits;
        self.counters.work_cost = next_work;
        Ok(())
    }

    /// 별도 결정적 counter다. 같은 solver evaluation에 primitive surcharge를
    /// 중복 부과하지 않는다.
    pub fn charge_solver_evaluations(&mut self, evaluations: u64) -> Result<(), WorkCostFailure> {
        if Self::checked_add(&mut self.counters.solver_evaluations, evaluations).is_none() {
            return Err(self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "solver_evaluations",
                u64::MAX,
                None,
            ));
        }
        Ok(())
    }

    pub fn note_committed_madi(&mut self) -> Result<(), WorkCostFailure> {
        if Self::checked_add(&mut self.counters.committed_madi, 1).is_none() {
            return Err(self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "committed_madi",
                u64::MAX,
                None,
            ));
        }
        Ok(())
    }

    pub fn note_external_events(&mut self, count: usize) -> Result<(), WorkCostFailure> {
        let count = u64::try_from(count).map_err(|_| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "external_events",
                u64::MAX,
                None,
            )
        })?;
        if Self::checked_add(&mut self.counters.external_events, count).is_none() {
            return Err(self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "external_events",
                u64::MAX,
                None,
            ));
        }
        Ok(())
    }

    pub fn enter_call(&mut self) -> Result<(), WorkCostFailure> {
        let next = self
            .counters
            .call_depth_current
            .checked_add(1)
            .ok_or_else(|| {
                self.failure(
                    E_WORK_COST_OVERFLOW_CANDIDATE,
                    "call_depth",
                    u64::MAX,
                    Some(self.limits.call_depth),
                )
            })?;
        if next > self.limits.call_depth {
            return Err(self.failure(
                E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE,
                "call_depth",
                next,
                Some(self.limits.call_depth),
            ));
        }
        self.counters.call_depth_current = next;
        self.counters.call_depth_peak = self.counters.call_depth_peak.max(next);
        Ok(())
    }

    pub fn exit_call(&mut self) {
        self.counters.call_depth_current = self.counters.call_depth_current.saturating_sub(1);
    }

    fn enforce_cumulative(
        &self,
        resource: &'static str,
        amount: u64,
        limit: u64,
    ) -> Result<(), WorkCostFailure> {
        if amount > limit {
            return Err(self.failure(
                E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE,
                resource,
                amount,
                Some(limit),
            ));
        }
        Ok(())
    }

    pub fn note_created_identity(&mut self) -> Result<(), WorkCostFailure> {
        let next = self
            .counters
            .created_identities
            .checked_add(1)
            .ok_or_else(|| {
                self.failure(
                    E_WORK_COST_OVERFLOW_CANDIDATE,
                    "created_identities",
                    u64::MAX,
                    Some(self.limits.created_identities),
                )
            })?;
        self.enforce_cumulative("created_identities", next, self.limits.created_identities)?;
        self.counters.created_identities = next;
        Ok(())
    }

    pub fn note_public_observation_samples(&mut self, count: usize) -> Result<(), WorkCostFailure> {
        let count = u64::try_from(count).map_err(|_| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "public_observation_samples",
                u64::MAX,
                Some(self.limits.public_observation_samples),
            )
        })?;
        let next = self
            .counters
            .public_observation_samples
            .checked_add(count)
            .ok_or_else(|| {
                self.failure(
                    E_WORK_COST_OVERFLOW_CANDIDATE,
                    "public_observation_samples",
                    u64::MAX,
                    Some(self.limits.public_observation_samples),
                )
            })?;
        self.enforce_cumulative(
            "public_observation_samples",
            next,
            self.limits.public_observation_samples,
        )?;
        self.counters.public_observation_samples = next;
        Ok(())
    }

    pub fn set_sealed_input_bytes(&mut self, bytes: usize) -> Result<(), WorkCostFailure> {
        let next = u64::try_from(bytes).map_err(|_| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "sealed_input_bytes",
                u64::MAX,
                Some(self.limits.sealed_input_bytes),
            )
        })?;
        self.enforce_cumulative("sealed_input_bytes", next, self.limits.sealed_input_bytes)?;
        self.counters.sealed_input_bytes = next;
        Ok(())
    }

    pub fn set_result_artifact_bytes(&mut self, bytes: usize) -> Result<(), WorkCostFailure> {
        let next = u64::try_from(bytes).map_err(|_| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "result_artifact_bytes",
                u64::MAX,
                Some(self.limits.result_artifact_bytes),
            )
        })?;
        self.enforce_cumulative(
            "result_artifact_bytes",
            next,
            self.limits.result_artifact_bytes,
        )?;
        self.counters.result_artifact_bytes = next;
        Ok(())
    }

    pub fn set_replay_capsule_bytes(&mut self, bytes: usize) -> Result<(), WorkCostFailure> {
        let next = u64::try_from(bytes).map_err(|_| {
            self.failure(
                E_WORK_COST_OVERFLOW_CANDIDATE,
                "replay_capsule_bytes",
                u64::MAX,
                Some(self.limits.replay_capsule_bytes),
            )
        })?;
        self.enforce_cumulative(
            "replay_capsule_bytes",
            next,
            self.limits.replay_capsule_bytes,
        )?;
        self.counters.replay_capsule_bytes = next;
        Ok(())
    }

    pub fn host_preflight_rejected(&self, fact: impl Into<String>) -> WorkCostFailure {
        self.failure(
            E_STANDARD_RESOURCE_PREFLIGHT_REJECTED_CANDIDATE,
            fact,
            0,
            None,
        )
    }

    pub fn registry_missing(&self, row: impl Into<String>) -> WorkCostFailure {
        self.failure(E_WORK_COST_REGISTRY_MISSING_CANDIDATE, row, 0, None)
    }
}

/// match에 wildcard를 두지 않는다. 새 실행 가능 문장 변형은 이 원장을 먼저
/// 갱신하지 않으면 컴파일되지 않는다.
pub fn stmt_registry_key(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::DeclBlock { .. } => "stmt.decl_block",
        Stmt::Definition { .. } => "stmt.definition",
        Stmt::Mutate { .. } => "stmt.mutate",
        Stmt::Expr { .. } => "stmt.expr",
        Stmt::Receive { .. } => "stmt.receive",
        Stmt::Send { .. } => "stmt.send",
        Stmt::Show { .. } => "stmt.show",
        Stmt::Inspect { .. } => "stmt.inspect",
        Stmt::PublicObservation { .. } => "stmt.public_observation",
        Stmt::MetaBlock { .. } => "stmt.meta_block",
        Stmt::Pragma { .. } => "stmt.pragma",
        Stmt::Return { .. } => "stmt.return",
        Stmt::If { .. } => "stmt.if",
        Stmt::Try { .. } => "stmt.try",
        Stmt::Choose { .. } => "stmt.choose",
        Stmt::Repeat { .. } => "stmt.repeat",
        Stmt::While { .. } => "stmt.while",
        Stmt::ForEach { .. } => "stmt.for_each",
        Stmt::Quantifier { .. } => "stmt.quantifier",
        Stmt::Break { .. } => "stmt.break",
        Stmt::ContinueLoop { .. } => "stmt.continue_loop",
        Stmt::Contract { .. } => "stmt.contract",
        Stmt::Guard { .. } => "stmt.guard",
        Stmt::BeatBlock { .. } => "stmt.beat_block",
        Stmt::Hook { .. } => "stmt.hook",
        Stmt::HookWhenBecomes { .. } => "stmt.hook_when_becomes",
        Stmt::HookWhile { .. } => "stmt.hook_while",
    }
}

pub fn expr_registry_key(expr: &ExprKind) -> &'static str {
    match expr {
        ExprKind::Literal(_) => "expr.literal",
        ExprKind::Var(_) => "expr.var",
        ExprKind::FieldAccess { .. } => "expr.field_access",
        ExprKind::SeedLiteral { .. } => "expr.seed_literal",
        ExprKind::Call { .. } => "expr.call",
        ExprKind::Infix { .. } => "expr.infix",
        ExprKind::Suffix { .. } => "expr.suffix",
        ExprKind::Thunk(_) => "expr.thunk",
        ExprKind::Eval { .. } => "expr.eval",
        ExprKind::Pipe { .. } => "expr.pipe",
        ExprKind::FlowValue => "expr.flow_value",
        ExprKind::Pack { .. } => "expr.pack",
        ExprKind::Assertion(_) => "expr.assertion",
        ExprKind::StateMachine(_) => "expr.state_machine",
        ExprKind::Formula(_) => "expr.formula",
        ExprKind::Template(_) => "expr.template",
        ExprKind::TemplateRender { .. } => "expr.template_render",
        ExprKind::FormulaEval { .. } => "expr.formula_eval",
        ExprKind::Nuance { .. } => "expr.nuance",
    }
}

pub fn literal_registry_key(literal: &Literal) -> &'static str {
    match literal {
        Literal::Int(_) => "literal.int",
        Literal::Fixed64(_) => "literal.fixed64",
        Literal::String(_) => "literal.string",
        Literal::Bool(_) => "literal.bool",
        Literal::Atom(_) => "literal.atom",
        Literal::Regex(_) => "literal.regex",
        Literal::Resource(_) => "literal.resource",
        Literal::None => "literal.none",
    }
}

pub fn model_expr_registry_key(node: &ModelExprNode) -> &'static str {
    match node {
        ModelExprNode::Number { .. } => "model_expr.number",
        ModelExprNode::String(_) => "model_expr.string",
        ModelExprNode::Atom(_) => "model_expr.atom",
        ModelExprNode::Path(_) => "model_expr.path",
        ModelExprNode::Derivative(_) => "model_expr.derivative",
        ModelExprNode::Unary { .. } => "model_expr.unary",
        ModelExprNode::Binary { .. } => "model_expr.binary",
        ModelExprNode::Group(_) => "model_expr.group",
    }
}

pub fn thunk_eval_mode_registry_key(mode: &ThunkEvalMode) -> &'static str {
    match mode {
        ThunkEvalMode::Value => "thunk_eval.value",
        ThunkEvalMode::Bool => "thunk_eval.bool",
        ThunkEvalMode::Not => "thunk_eval.not",
        ThunkEvalMode::Do => "thunk_eval.do",
        ThunkEvalMode::Pipe => "thunk_eval.pipe",
    }
}

pub fn contract_kind_registry_key(kind: &ContractKind) -> &'static str {
    match kind {
        ContractKind::Pre => "contract.pre",
        ContractKind::Post => "contract.post",
    }
}

pub fn contract_mode_registry_key(mode: &ContractMode) -> &'static str {
    match mode {
        ContractMode::Abort => "contract_mode.abort",
        ContractMode::Alert => "contract_mode.alert",
    }
}

pub fn quantifier_registry_key(kind: &QuantifierKind) -> &'static str {
    match kind {
        QuantifierKind::ForAll => "quantifier.for_all",
        QuantifierKind::Exists => "quantifier.exists",
        QuantifierKind::ExistsUnique => "quantifier.exists_unique",
    }
}

pub fn hook_registry_key(kind: &HookKind) -> &'static str {
    match kind {
        HookKind::Start => "hook.start",
        HookKind::End => "hook.end",
        HookKind::EveryMadi => "hook.every_madi",
        HookKind::EveryNMadi(_) => "hook.every_n_madi",
    }
}

pub fn model_body_item_registry_key(item: &ModelBodyItem) -> &'static str {
    match item {
        ModelBodyItem::Declaration(_) => "model_body.declaration",
        ModelBodyItem::Unknown(_) => "model_body.unknown",
        ModelBodyItem::Relation(_) => "model_body.relation",
        ModelBodyItem::Event(_) => "model_body.event",
        ModelBodyItem::Connection(_) => "model_body.connection",
    }
}

pub fn lowered_relation_target_registry_key(target: &LoweredRelationTarget) -> &'static str {
    match target {
        LoweredRelationTarget::Algebraic(_) => "model_relation.algebraic",
        LoweredRelationTarget::Derivative(_) => "model_relation.derivative",
    }
}

pub fn model_event_kind_registry_key(kind: &ModelEventKind) -> &'static str {
    match kind {
        ModelEventKind::Becomes => "model_event.becomes",
        ModelEventKind::While => "model_event.while",
        ModelEventKind::EveryMadi(_) => "model_event.every_madi",
        ModelEventKind::CrossingBoth => "model_event.crossing_both",
    }
}

pub fn event_detection_registry_key(policy: &EventDetectionPolicy) -> &'static str {
    match policy {
        EventDetectionPolicy::FalseToTrueTransition => "model_event.false_to_true",
        EventDetectionPolicy::WhileGuard => "model_event.while_guard",
        EventDetectionPolicy::MadiInterval(_) => "model_event.madi_interval",
        EventDetectionPolicy::BidirectionalContinuousCrossing => {
            "model_event.bidirectional_crossing"
        }
    }
}

pub fn connection_mode_registry_key(mode: &ConnectionMode) -> &'static str {
    match mode {
        ConnectionMode::Equal => "model_connection.equal",
        ConnectionMode::Flow => "model_connection.flow",
        ConnectionMode::ReverseFlow => "model_connection.reverse_flow",
    }
}

pub const STDLIB_SIGNATURE_ROW_COUNT_V1: usize = 256;
/// G2 source inventory hash. The source-text extractor retained one trailing
/// comma in a multiline parameter row, so product totality uses the semantic
/// signature hash below rather than treating Rust formatting as runtime identity.
pub const STDLIB_SOURCE_INVENTORY_LEDGER_SHA256_V1: &str =
    "4220c2f4348d95e9a91905823e6dca44aae74e05a0e58d26570cb935ce644772";
pub const STDLIB_SIGNATURE_LEDGER_SHA256_V1: &str =
    "7e1bb12e2063b6ed3855d7ebccfc83d29ee70c168f46e981d95e18aec07d2794";

fn stdlib_signature_rows() -> Vec<(&'static str, ddonirang_lang::stdlib::FunctionSig)> {
    let mut rows = Vec::new();
    macro_rules! extend_group {
        ($group:literal, $function:path) => {
            rows.extend($function().into_iter().map(|signature| ($group, signature)));
        };
    }
    extend_group!(
        "string_function_sigs",
        ddonirang_lang::stdlib::string_function_sigs
    );
    extend_group!(
        "regex_function_sigs",
        ddonirang_lang::stdlib::regex_function_sigs
    );
    extend_group!(
        "language_core_collection_constructor_sigs",
        ddonirang_lang::stdlib::language_core_collection_constructor_sigs
    );
    extend_group!(
        "list_function_sigs",
        ddonirang_lang::stdlib::list_function_sigs
    );
    extend_group!(
        "container_function_sigs",
        ddonirang_lang::stdlib::container_function_sigs
    );
    extend_group!(
        "stream_function_sigs",
        ddonirang_lang::stdlib::stream_function_sigs
    );
    extend_group!(
        "input_function_sigs",
        ddonirang_lang::stdlib::input_function_sigs
    );
    extend_group!(
        "grid_function_sigs",
        ddonirang_lang::stdlib::grid_function_sigs
    );
    extend_group!(
        "block_piece_function_sigs",
        ddonirang_lang::stdlib::block_piece_function_sigs
    );
    extend_group!(
        "physics_1d_function_sigs",
        ddonirang_lang::stdlib::physics_1d_function_sigs
    );
    extend_group!(
        "resource_function_sigs",
        ddonirang_lang::stdlib::resource_function_sigs
    );
    extend_group!(
        "random_function_sigs",
        ddonirang_lang::stdlib::random_function_sigs
    );
    extend_group!(
        "grid_game_state_function_sigs",
        ddonirang_lang::stdlib::grid_game_state_function_sigs
    );
    extend_group!(
        "grid_game_playable_function_sigs",
        ddonirang_lang::stdlib::grid_game_playable_function_sigs
    );
    extend_group!(
        "math_function_sigs",
        ddonirang_lang::stdlib::math_function_sigs
    );
    extend_group!(
        "pack_function_sigs",
        ddonirang_lang::stdlib::pack_function_sigs
    );
    extend_group!(
        "transform_function_sigs",
        ddonirang_lang::stdlib::transform_function_sigs
    );
    extend_group!(
        "connect_relation_function_sigs",
        ddonirang_lang::stdlib::connect_relation_function_sigs
    );
    extend_group!(
        "logic_function_sigs",
        ddonirang_lang::stdlib::logic_function_sigs
    );
    extend_group!(
        "age1_runtime_function_sigs",
        ddonirang_lang::stdlib::age1_runtime_function_sigs
    );
    extend_group!(
        "numeric_type_constructor_sigs",
        ddonirang_lang::stdlib::numeric_type_constructor_sigs
    );
    rows
}

fn stdlib_signature_ledger() -> String {
    let mut ledger_rows = Vec::new();
    for (group, sig) in stdlib_signature_rows() {
        let params = sig
            .params
            .iter()
            .map(|param| format!("\"{param}\""))
            .collect::<Vec<_>>()
            .join(",");
        ledger_rows.push(format!("{}|{}|{}|{}", group, sig.name, params, sig.ret));
    }
    ledger_rows.join("\n")
}

pub fn stdlib_signature_ledger_sha256() -> String {
    hex::encode(Sha256::digest(stdlib_signature_ledger().as_bytes()))
}

pub fn verify_stdlib_registry_totality() -> Result<(), WorkCostFailure> {
    let signatures = ddonirang_lang::stdlib::minimal_stdlib_sigs();
    let actual_hash = stdlib_signature_ledger_sha256();
    if signatures.len() != STDLIB_SIGNATURE_ROW_COUNT_V1
        || actual_hash != STDLIB_SIGNATURE_LEDGER_SHA256_V1
    {
        return Err(WorkCostMeter::default().failure(
            E_WORK_COST_REGISTRY_MISSING_CANDIDATE,
            format!(
                "stdlib_signature_ledger:rows={}:sha256={}",
                signatures.len(),
                actual_hash
            ),
            signatures.len() as u64,
            Some(STDLIB_SIGNATURE_ROW_COUNT_V1 as u64),
        ));
    }
    Ok(())
}

pub fn registered_stdlib_names() -> Result<BTreeSet<&'static str>, WorkCostFailure> {
    verify_stdlib_registry_totality()?;
    Ok(ddonirang_lang::stdlib::minimal_stdlib_sigs()
        .into_iter()
        .map(|sig| sig.name)
        .collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltinWorkCostRule {
    pub visit_collection_arguments: bool,
    pub visit_canonical_utf8_arguments: bool,
    pub bigint_rational_operand_and_result_bytes: bool,
    pub solver_evaluation_counter: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltinWorkCostLedgerRow {
    pub group: &'static str,
    pub name: &'static str,
    pub canonical_name: &'static str,
    pub params: Vec<&'static str>,
    pub ret: &'static str,
    pub rule: BuiltinWorkCostRule,
}

const UTF8_VISIT_BUILTINS: &[&str] = &[
    "길이",
    "대문자로바꾸기",
    "소문자로바꾸기",
    "다듬기",
    "되풀이하기",
    "포함하나",
    "시작하나",
    "끝나나",
    "자르기",
    "글자뽑기",
    "숫자로",
    "글로",
    "찾기",
    "바꾸기",
    "글바꾸기",
    "글바꾸기!",
    "정규맞추기",
    "정규찾기",
    "정규캡처하기",
    "정규이름캡처하기",
    "정규바꾸기",
    "정규나누기",
    "입력키",
    "입력키?",
    "입력키!",
    "자료.열기",
    "묶음값",
    "채우기",
];

const BIGINT_RATIONAL_BUILTINS: &[&str] = &["큰바른수", "나눔수", "곱수", "인수분해하기"];

const MANUAL_COLLECTION_VISIT_BUILTINS: &[&str] = &[
    "차림",
    "목록",
    "모음",
    "짝맞춤",
    "번째",
    "차림.값",
    "차림.바꾼값",
    "길이",
    "첫번째",
    "마지막",
    "추가",
    "제거",
    "정렬",
    "거르기",
    "변환",
    "바꾸기",
    "토막내기",
    "들어있나",
    "찾아보기",
    "뒤집기",
    "펼치기",
    "각각돌며",
    "합치기",
    "붙이기",
    "합계",
    "평균",
    "지니",
    "분위수",
];

/// `eval_call`이 직접 소유하지만 public stdlib signature 표에는 아직 없는
/// 고정폭 scalar primitive. 이 목록 밖의 runtime 전용 primitive는 totality
/// source checker가 실패해야 한다.
pub const RUNTIME_ONLY_BUILTINS: &[&str] = &[
    "abs", "acos", "asin", "atan", "atan2", "clamp", "cos", "max", "min", "powi", "sin", "sqrt",
    "tan", "셈수",
];

const SOLVER_BUILTINS: &[&str] = &[
    "수치해.이분법",
    "다항식.풀기",
    "선형부등식.풀기",
    "방정식풀기",
    "해찾기",
    "풀기",
    "증명하기",
    "반례찾기",
    "이음관계.풀기",
    "이음관계.풀고범위위반목록",
    "이음관계.풀고범위검사",
    "이음관계.풀고범위행목록",
    "이음관계.풀고범위보고서",
    "이음관계.풀고범위문자표",
    "이음관계.풀고범위스위트",
    "이음관계.풀고범위케이스",
    "이음관계.풀고범위실행상세문자표",
    "이음관계.풀고범위실행요약",
    "이음관계.풀고범위실행판정",
];

fn signature_has_collection_surface(sig: &ddonirang_lang::stdlib::FunctionSig) -> bool {
    const TOKENS: &[&str] = &[
        "차림",
        "목록",
        "모음",
        "짝맞춤",
        "묶음",
        "흐름",
        "이력",
        "격자",
        "조각",
        "행",
        "열쇠",
        "요소들",
        "값들",
        "관계",
        "수식",
    ];
    sig.params
        .iter()
        .chain(std::iter::once(&sig.ret))
        .any(|surface| TOKENS.iter().any(|token| surface.contains(token)))
}

/// 256개 봉인 signature row 각각에 대한 규칙을 반환한다. signature ledger
/// hash가 달라지면 이 함수보다 먼저 totality gate가 실패하므로 새 primitive가
/// dispatch-only 규칙으로 자동 유입되지 않는다.
fn builtin_work_cost_rule_unchecked(canonical_name: &str) -> Option<BuiltinWorkCostRule> {
    let signatures = ddonirang_lang::stdlib::minimal_stdlib_sigs();
    let matching = signatures
        .iter()
        .filter(|sig| ddonirang_lang::stdlib::canonicalize_stdlib_alias(sig.name) == canonical_name)
        .collect::<Vec<_>>();
    if matching.is_empty() && !RUNTIME_ONLY_BUILTINS.contains(&canonical_name) {
        return None;
    }
    let visit_collection_arguments = matching
        .iter()
        .any(|sig| signature_has_collection_surface(sig))
        && !MANUAL_COLLECTION_VISIT_BUILTINS.contains(&canonical_name);
    let visit_canonical_utf8_arguments = UTF8_VISIT_BUILTINS.contains(&canonical_name);
    let bigint_rational_operand_and_result_bytes =
        BIGINT_RATIONAL_BUILTINS.contains(&canonical_name);
    let solver_evaluation_counter = SOLVER_BUILTINS.contains(&canonical_name);
    Some(BuiltinWorkCostRule {
        visit_collection_arguments,
        visit_canonical_utf8_arguments,
        bigint_rational_operand_and_result_bytes,
        solver_evaluation_counter,
    })
}

pub fn builtin_work_cost_ledger() -> Result<Vec<BuiltinWorkCostLedgerRow>, WorkCostFailure> {
    verify_stdlib_registry_totality()?;
    stdlib_signature_rows()
        .into_iter()
        .map(|(group, signature)| {
            let canonical_name = ddonirang_lang::stdlib::canonicalize_stdlib_alias(signature.name);
            let rule = builtin_work_cost_rule_unchecked(canonical_name).ok_or_else(|| {
                WorkCostMeter::default().registry_missing(format!(
                    "stdlib_signature:{group}|{}|{}|{}",
                    signature.name,
                    signature.params.join(","),
                    signature.ret
                ))
            })?;
            Ok(BuiltinWorkCostLedgerRow {
                group,
                name: signature.name,
                canonical_name,
                params: signature.params.to_vec(),
                ret: signature.ret,
                rule,
            })
        })
        .collect()
}

pub fn builtin_work_cost_rule(
    canonical_name: &str,
) -> Result<Option<BuiltinWorkCostRule>, WorkCostFailure> {
    verify_stdlib_registry_totality()?;
    Ok(builtin_work_cost_rule_unchecked(canonical_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_call_static_dispatch_names() -> BTreeSet<String> {
        let source = include_str!("ddn_runtime.rs");
        let start = source
            .find("    fn eval_call(")
            .expect("eval_call source boundary");
        let end = source[start..]
            .find("    fn resource_backed_caller_names")
            .map(|offset| start + offset)
            .expect("eval_call source end boundary");
        let mut names = BTreeSet::new();
        let mut arm_head = String::new();
        for line in source[start..end].lines() {
            // 기존 source와 rustfmt 결과 모두에서 `match func` arm은 이 두
            // level 중 하나다. 더 깊은 helper match의 문자열은 제외한다.
            let indentation = line.len() - line.trim_start().len();
            if indentation != 12 && indentation != 16 {
                continue;
            }
            let trimmed = line.trim_start();
            if trimmed.starts_with('"') || trimmed.starts_with('|') {
                arm_head.push(' ');
                arm_head.push_str(trimmed);
            } else {
                arm_head.clear();
                continue;
            }
            if !arm_head.contains("=>") {
                continue;
            }
            let mut rest = arm_head.as_str();
            while let Some(open) = rest.find('"') {
                rest = &rest[open + 1..];
                let Some(close) = rest.find('"') else {
                    break;
                };
                names.insert(rest[..close].to_string());
                rest = &rest[close + 1..];
            }
            arm_head.clear();
        }
        names
    }

    #[test]
    fn standard_candidate_constants_match_rd186_owner_binding() {
        let limits = StandardResourceLimitsV1::CANDIDATE;
        assert_eq!(limits.slice_committed_madi, 100_000);
        assert_eq!(limits.slice_work_cost, 100_000_000);
        assert_eq!(limits.slice_solver_evaluations, 1_000_000);
        assert_eq!(limits.slice_external_events, 100_000);
        assert_eq!(limits.call_depth, 512);
        assert_eq!(limits.created_identities, 100_000);
        assert_eq!(limits.public_observation_samples, 2_000_000);
        assert_eq!(limits.sealed_input_bytes, 67_108_864);
        assert_eq!(limits.result_artifact_bytes, 268_435_456);
        assert_eq!(limits.replay_capsule_bytes, 268_435_456);
    }

    #[test]
    fn slice_counters_are_measured_but_not_enforced() {
        let mut meter = WorkCostMeter::new(StandardResourceLimitsV1 {
            slice_committed_madi: 0,
            slice_work_cost: 0,
            slice_solver_evaluations: 0,
            slice_external_events: 0,
            ..StandardResourceLimitsV1::CANDIDATE
        });
        meter.charge_dispatch().unwrap();
        meter.charge_solver_evaluations(1).unwrap();
        meter.note_external_events(1).unwrap();
        meter.note_committed_madi().unwrap();
        let receipt = meter.receipt();
        assert_eq!(receipt.counters.work_cost, 1);
        assert!(!receipt.slice_limits_enforced);
        assert!(!receipt.continuation_present);
        assert!(!receipt.rd206_resource_capability);
    }

    #[test]
    fn repeated_visits_charge_each_visit() {
        let mut meter = WorkCostMeter::default();
        meter.charge_element_visits(3).unwrap();
        meter.charge_element_visits(3).unwrap();
        meter.charge_canonical_utf8_bytes("가").unwrap();
        meter.charge_canonical_utf8_bytes("가").unwrap();
        assert_eq!(meter.receipt().counters.work_cost, 12);
        assert_eq!(meter.receipt().counters.collection_tensor_element_visits, 6);
        assert_eq!(meter.receipt().counters.canonical_utf8_byte_visits, 6);
    }

    #[test]
    fn cumulative_limit_is_fail_closed_at_boundary_plus_one() {
        let mut meter = WorkCostMeter::new(StandardResourceLimitsV1 {
            created_identities: 1,
            ..StandardResourceLimitsV1::CANDIDATE
        });
        meter.note_created_identity().unwrap();
        let before = meter.receipt();
        let failure = meter.note_created_identity().unwrap_err();
        assert_eq!(failure.code, E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE);
        assert_eq!(failure.resource, "created_identities");
        assert_eq!(failure.actual, 2);
        assert_eq!(failure.receipt.counters.created_identities, 1);
        assert_eq!(meter.receipt(), before);
    }

    #[test]
    fn call_depth_nested_boundary_and_unwind_are_exact() {
        let mut meter = WorkCostMeter::new(StandardResourceLimitsV1 {
            call_depth: 2,
            ..StandardResourceLimitsV1::CANDIDATE
        });
        meter.enter_call().unwrap();
        meter.enter_call().unwrap();
        let failure = meter.enter_call().unwrap_err();
        assert_eq!(failure.actual, 3);
        assert_eq!(meter.receipt().counters.call_depth_current, 2);
        meter.exit_call();
        meter.exit_call();
        assert_eq!(meter.receipt().counters.call_depth_current, 0);
        assert_eq!(meter.receipt().counters.call_depth_peak, 2);
    }

    #[test]
    fn bigint_signed_magnitude_charges_operands_and_result() {
        let minus_one = BigInt::from(-1);
        let zero = BigInt::from(0);
        let result = BigInt::from(256);
        let mut meter = WorkCostMeter::default();
        meter
            .charge_bigint_rational_primitive(&[&minus_one, &zero], &[&result])
            .unwrap();
        // num-bigint canonical magnitude는 0도 한 바이트다.
        // -1: sign+1, 0: sign+1, 256: sign+2
        assert_eq!(
            meter
                .receipt()
                .counters
                .bigint_rational_signed_magnitude_byte_visits,
            7
        );
        assert_eq!(meter.receipt().counters.work_cost, 7);
    }

    #[test]
    fn stdlib_signature_ledger_is_exact_and_total() {
        verify_stdlib_registry_totality().unwrap();
        assert_eq!(
            ddonirang_lang::stdlib::minimal_stdlib_sigs().len(),
            STDLIB_SIGNATURE_ROW_COUNT_V1
        );
        assert_eq!(
            stdlib_signature_ledger_sha256(),
            STDLIB_SIGNATURE_LEDGER_SHA256_V1
        );
        assert_eq!(registered_stdlib_names().unwrap().len(), 251);
        for sig in ddonirang_lang::stdlib::minimal_stdlib_sigs() {
            let canonical = ddonirang_lang::stdlib::canonicalize_stdlib_alias(sig.name);
            assert!(
                builtin_work_cost_rule(canonical).unwrap().is_some(),
                "missing builtin WorkCostV1 row: {}",
                sig.name
            );
        }
        for name in RUNTIME_ONLY_BUILTINS {
            assert!(
                builtin_work_cost_rule(name).unwrap().is_some(),
                "missing runtime-only WorkCostV1 row: {name}"
            );
        }
    }

    #[test]
    fn every_static_runtime_dispatch_arm_has_a_work_cost_rule() {
        let dispatch_names = eval_call_static_dispatch_names();
        assert_eq!(dispatch_names.len(), 226, "eval_call dispatch ledger drift");
        let missing = dispatch_names
            .into_iter()
            .filter_map(|name| {
                let canonical = ddonirang_lang::stdlib::canonicalize_stdlib_alias(&name);
                builtin_work_cost_rule(canonical)
                    .unwrap()
                    .is_none()
                    .then_some(format!("{name} (canonical: {canonical})"))
            })
            .collect::<Vec<_>>();
        assert!(
            missing.is_empty(),
            "missing runtime dispatch WorkCostV1 rows: {missing:?}"
        );
    }

    #[test]
    fn every_stdlib_overload_row_has_an_exact_cost_ledger_entry() {
        let ledger = builtin_work_cost_ledger().expect("sealed stdlib ledger");
        assert_eq!(ledger.len(), STDLIB_SIGNATURE_ROW_COUNT_V1);
        assert!(ledger.iter().all(|row| !row.group.is_empty()));
        assert!(ledger.iter().all(|row| !row.name.is_empty()));
        assert!(ledger.iter().all(|row| !row.canonical_name.is_empty()));
        let identities = ledger
            .iter()
            .map(|row| {
                format!(
                    "{}|{}|{}|{}",
                    row.group,
                    row.name,
                    row.params.join(","),
                    row.ret
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(identities.len(), STDLIB_SIGNATURE_ROW_COUNT_V1);
    }

    #[test]
    fn host_preflight_is_distinct_from_semantic_limit_failure() {
        let meter = WorkCostMeter::default();
        let failure = meter.host_preflight_rejected("host_alloc_probe");
        assert_eq!(
            failure.code,
            E_STANDARD_RESOURCE_PREFLIGHT_REJECTED_CANDIDATE
        );
        assert_eq!(failure.limit, None);
    }

    #[test]
    fn artifact_byte_limits_reject_boundary_plus_one_without_counter_mutation() {
        let mut meter = WorkCostMeter::new(StandardResourceLimitsV1 {
            sealed_input_bytes: 1,
            result_artifact_bytes: 1,
            replay_capsule_bytes: 1,
            ..StandardResourceLimitsV1::CANDIDATE
        });
        meter.set_sealed_input_bytes(1).unwrap();
        meter.set_result_artifact_bytes(1).unwrap();
        meter.set_replay_capsule_bytes(1).unwrap();
        let before = meter.receipt();
        for failure in [
            meter.set_sealed_input_bytes(2).unwrap_err(),
            meter.set_result_artifact_bytes(2).unwrap_err(),
            meter.set_replay_capsule_bytes(2).unwrap_err(),
        ] {
            assert_eq!(failure.code, E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE);
            assert_eq!(failure.actual, 2);
            assert_eq!(failure.limit, Some(1));
        }
        assert_eq!(meter.receipt(), before);
    }

    #[test]
    fn component_charge_is_atomic_when_total_work_overflows() {
        let mut meter = WorkCostMeter::default();
        meter.counters.work_cost = u64::MAX;
        let before = meter.receipt();
        let failure = meter.charge_dispatch().unwrap_err();
        assert_eq!(failure.code, E_WORK_COST_OVERFLOW_CANDIDATE);
        assert_eq!(failure.resource, "base_dispatch");
        assert_eq!(meter.receipt(), before);
    }
}
