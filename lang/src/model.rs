//! RD-114~122 선언형 모형의 제품 frontdoor와 typed in-memory IR.
//!
//! 이 모듈은 solver가 아니다. 공개 DDN 표면을 typed 구조로 읽고, 정본화하고,
//! 실행 backend 선택 전에 닫을 수 있는 구조 오류만 진단한다. JSON 출력은 검사
//! 편의를 위한 candidate projection이며 persistent public wire가 아니다.

use ddonirang_core::{unit_spec_from_symbol, UnitDim};
use serde_json::{json, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use unicode_normalization::UnicodeNormalization;

pub const MODEL_IR_SCHEMA_CANDIDATE: &str = "ddn.model_ir.candidate.w1";
pub const EXACT_ACYCLIC_RELATION_BACKEND_ID: &str = "ddn.exact_acyclic_relation.v1";
pub const FIXED_DEFAULT_PRECISE_RECIPE_NAME: &str = "정밀한방법";
pub const STANDARD_REPRO_PROFILE_IDENTITY: &str = "표준재현_v1";
pub const EXECUTION_CONTRACT_SCHEMA_CANDIDATE: &str = "ddn.model_execution_contract.candidate.v1";

pub const EXECUTION_CONTRACT_FIELD_RESULT: &str = "결과";
pub const EXECUTION_CONTRACT_FIELD_REPRO_SCOPE: &str = "재현범위";
pub const EXECUTION_CONTRACT_FIELD_PERFORMANCE: &str = "성능";
pub const EXECUTION_CONTRACT_FIELD_EXTERNAL_INGRESS: &str = "외부유입";
pub const EXECUTION_CONTRACT_FIELD_EVIDENCE: &str = "증거";
pub const EXECUTION_CONTRACT_FIELD_REPLAY: &str = "재생";
pub const EXECUTION_CONTRACT_FIELD_RESOURCE: &str = "자원";

pub const EXECUTION_CONTRACT_VALUE_CANONICAL_EQUAL: &str = "정본값동일";
pub const EXECUTION_CONTRACT_VALUE_PORTABLE: &str = "이식가능";
pub const EXECUTION_CONTRACT_VALUE_SAFE_OPTIMIZATION: &str = "안전최적화";
pub const EXECUTION_CONTRACT_VALUE_SEALED_INPUT: &str = "봉인입력";
pub const EXECUTION_CONTRACT_VALUE_MINIMUM_EVIDENCE: &str = "최소증거";
pub const EXECUTION_CONTRACT_VALUE_VERIFIED_REPLAY: &str = "검증재생";
pub const EXECUTION_CONTRACT_VALUE_STANDARD_LIMIT_V1: &str = "표준한도_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSpan {
    pub start: usize,
    pub end: usize,
}

impl ModelSpan {
    fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredIdentity {
    pub stable_id: String,
    pub occurrence: u32,
    pub span: ModelSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryRole {
    Axis,
    Continuous,
    Discrete,
    Algebraic,
    Input,
}

impl PrimaryRole {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "축" => Some(Self::Axis),
            "이어감" => Some(Self::Continuous),
            "띄엄" => Some(Self::Discrete),
            "그자리" => Some(Self::Algebraic),
            "들임" => Some(Self::Input),
            _ => None,
        }
    }

    fn surface(self) -> &'static str {
        match self {
            Self::Axis => "축",
            Self::Continuous => "이어감",
            Self::Discrete => "띄엄",
            Self::Algebraic => "그자리",
            Self::Input => "들임",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Initialization {
    None,
    Fixed(ModelExpr),
    Guess(ModelExpr),
    Computed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelType {
    pub name: String,
    pub unit: Option<String>,
    pub dimension: Option<UnitDim>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDeclaration {
    pub identity: AuthoredIdentity,
    pub name: String,
    pub model_type: ModelType,
    pub role: PrimaryRole,
    pub initialization: Initialization,
    pub accepts_initial_override: bool,
    pub exported: bool,
    pub unknown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivativeIdentity {
    pub base: String,
    pub axis: Option<String>,
    pub order: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelExprKind {
    Path(String),
    Derivative(DerivativeIdentity),
    Literal,
    Compound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelExprNode {
    Number {
        surface: String,
        unit: Option<String>,
        dimension: UnitDim,
    },
    String(String),
    Atom(String),
    Path(String),
    Derivative(DerivativeIdentity),
    Unary {
        operator: String,
        operand: Box<ModelExprNode>,
    },
    Binary {
        operator: String,
        left: Box<ModelExprNode>,
        right: Box<ModelExprNode>,
    },
    Group(Box<ModelExprNode>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelExpr {
    pub span: ModelSpan,
    pub canonical: String,
    pub kind: ModelExprKind,
    pub node: Option<ModelExprNode>,
    pub references: Vec<String>,
    pub dimension: Option<UnitDim>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRelation {
    pub identity: AuthoredIdentity,
    pub semantic_identity: String,
    pub left: ModelExpr,
    pub right: ModelExpr,
    pub lowered: Option<LoweredRelationNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredRelationTarget {
    Algebraic(String),
    Derivative(DerivativeIdentity),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredRelationNode {
    pub identity: String,
    pub authored_relation_identity: String,
    pub target: LoweredRelationTarget,
    pub expression: ModelExpr,
    pub dependencies: Vec<String>,
    pub provenance: ModelSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelEventKind {
    Becomes,
    While,
    EveryMadi(u64),
    CrossingBoth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventDetectionPolicy {
    FalseToTrueTransition,
    WhileGuard,
    MadiInterval(u64),
    BidirectionalContinuousCrossing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredEventNode {
    pub identity: String,
    pub authored_event_identity: String,
    pub semantic_identity: String,
    pub detection: EventDetectionPolicy,
    pub remember_last_nonzero_sign: bool,
    pub simultaneous_grouping: bool,
    pub reset_conflict_policy: &'static str,
    pub reset_targets: Vec<String>,
    pub required_capability: &'static str,
    pub provenance: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEvent {
    pub identity: AuthoredIdentity,
    pub semantic_identity: String,
    pub kind: ModelEventKind,
    pub trigger: ModelExpr,
    pub resets: Vec<ModelReset>,
    pub lowered: LoweredEventNode,
    pub body_span: ModelSpan,
    pub canonical: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelReset {
    pub identity: String,
    pub event_identity: String,
    pub target: String,
    pub value: ModelExpr,
    pub provenance: ModelSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    Equal,
    Flow,
    ReverseFlow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionClause {
    pub quantity: String,
    pub mode: ConnectionMode,
    pub provenance: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredConnectionRelation {
    pub identity: String,
    pub authored_connection_identity: String,
    pub semantic_identity: String,
    pub quantity: String,
    pub mode: ConnectionMode,
    pub orientation_sign: Option<i8>,
    pub endpoints: Vec<String>,
    pub required_capability: &'static str,
    pub provenance: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelConnection {
    pub identity: AuthoredIdentity,
    pub semantic_identity: String,
    pub endpoints: Vec<String>,
    pub clauses: Vec<ConnectionClause>,
    pub lowered_relations: Vec<LoweredConnectionRelation>,
    pub canonical: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelBodyItem {
    Declaration(String),
    Unknown(String),
    Relation(usize),
    Event(usize),
    Connection(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDefinition {
    pub identity: String,
    pub source_hash: String,
    pub schema_version: &'static str,
    pub name: String,
    pub span: ModelSpan,
    pub header_span: ModelSpan,
    pub body_span: ModelSpan,
    pub declarations: Vec<ModelDeclaration>,
    pub relations: Vec<ModelRelation>,
    pub events: Vec<ModelEvent>,
    pub connections: Vec<ModelConnection>,
    pub authored_order: Vec<ModelBodyItem>,
    pub required_capabilities: BTreeSet<String>,
}

/// 공개 `모형 {}` 식이 만드는 `모형값`. definition artifact와 값 identity를
/// 의도적으로 분리한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelValue {
    pub identity: String,
    pub definition_identity: String,
    pub public_type: &'static str,
    pub provenance: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInstance {
    pub identity: String,
    pub source_hash: String,
    pub schema_version: &'static str,
    pub definition_identity: String,
    pub binding_name: String,
    pub parameters: BTreeMap<String, ModelExpr>,
    pub initial_overrides: BTreeMap<String, ModelExpr>,
    pub provenance: ModelSpan,
}

impl ModelInstance {
    /// 공개 instance 작성 문법을 만들지 않고 typed in-memory candidate를 구성한다.
    pub fn candidate(
        definition_identity: impl Into<String>,
        binding_name: impl Into<String>,
        parameters: BTreeMap<String, ModelExpr>,
        initial_overrides: BTreeMap<String, ModelExpr>,
        provenance: ModelSpan,
    ) -> Self {
        let definition_identity = definition_identity.into();
        let binding_name = binding_name.into();
        let parameters_canon = parameters
            .iter()
            .map(|(name, value)| format!("{name}:{}", value.canonical))
            .collect::<Vec<_>>()
            .join(";");
        let initials_canon = initial_overrides
            .iter()
            .map(|(name, value)| format!("{name}:{}", value.canonical))
            .collect::<Vec<_>>()
            .join(";");
        let candidate_canon = format!(
            "definition={definition_identity}|binding={binding_name}|parameters={parameters_canon}|initials={initials_canon}"
        );
        let source_hash = hash_prefixed(candidate_canon.as_bytes());
        Self {
            identity: format!("instance:{binding_name}:{source_hash}"),
            source_hash,
            schema_version: MODEL_IR_SCHEMA_CANDIDATE,
            definition_identity,
            binding_name,
            parameters,
            initial_overrides,
            provenance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordField {
    pub name: String,
    pub value: ModelExpr,
    pub span: ModelSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionResultContract {
    CanonicalEqual,
}

impl ExecutionResultContract {
    pub fn surface(self) -> &'static str {
        match self {
            Self::CanonicalEqual => EXECUTION_CONTRACT_VALUE_CANONICAL_EQUAL,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionReproScope {
    Portable,
}

impl ExecutionReproScope {
    pub fn surface(self) -> &'static str {
        match self {
            Self::Portable => EXECUTION_CONTRACT_VALUE_PORTABLE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPerformancePolicy {
    SafeOptimization,
}

impl ExecutionPerformancePolicy {
    pub fn surface(self) -> &'static str {
        match self {
            Self::SafeOptimization => EXECUTION_CONTRACT_VALUE_SAFE_OPTIMIZATION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionExternalIngressPolicy {
    SealedInput,
}

impl ExecutionExternalIngressPolicy {
    pub fn surface(self) -> &'static str {
        match self {
            Self::SealedInput => EXECUTION_CONTRACT_VALUE_SEALED_INPUT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionEvidencePolicy {
    MinimumEvidence,
}

impl ExecutionEvidencePolicy {
    pub fn surface(self) -> &'static str {
        match self {
            Self::MinimumEvidence => EXECUTION_CONTRACT_VALUE_MINIMUM_EVIDENCE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionReplayPolicy {
    VerifiedReplay,
}

impl ExecutionReplayPolicy {
    pub fn surface(self) -> &'static str {
        match self {
            Self::VerifiedReplay => EXECUTION_CONTRACT_VALUE_VERIFIED_REPLAY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionResourcePolicy {
    StandardLimitV1,
}

impl ExecutionResourcePolicy {
    pub fn surface(self) -> &'static str {
        match self {
            Self::StandardLimitV1 => EXECUTION_CONTRACT_VALUE_STANDARD_LIMIT_V1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExecutionContractField<T> {
    pub value: T,
    pub field_span: ModelSpan,
    pub value_span: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionContractProvenance {
    Explicit {
        source_file: String,
        source_range: ModelSpan,
    },
    OmittedStandard {
        source_file: String,
        source_range: ModelSpan,
    },
}

impl ExecutionContractProvenance {
    pub fn is_authored(&self) -> bool {
        matches!(self, Self::Explicit { .. })
    }

    pub fn source_file(&self) -> &str {
        match self {
            Self::Explicit { source_file, .. } | Self::OmittedStandard { source_file, .. } => {
                source_file
            }
        }
    }

    pub fn source_range(&self) -> ModelSpan {
        match self {
            Self::Explicit { source_range, .. } | Self::OmittedStandard { source_range, .. } => {
                *source_range
            }
        }
    }

    pub fn surface_kind(&self) -> &'static str {
        match self {
            Self::Explicit { .. } => "explicit",
            Self::OmittedStandard { .. } => "omitted_standard_profile",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContractProfile {
    pub schema_version: &'static str,
    pub profile_identity: &'static str,
    pub profile_version: u32,
    pub result: TypedExecutionContractField<ExecutionResultContract>,
    pub repro_scope: TypedExecutionContractField<ExecutionReproScope>,
    pub performance: TypedExecutionContractField<ExecutionPerformancePolicy>,
    pub external_ingress: TypedExecutionContractField<ExecutionExternalIngressPolicy>,
    pub evidence: TypedExecutionContractField<ExecutionEvidencePolicy>,
    pub replay: TypedExecutionContractField<ExecutionReplayPolicy>,
    pub resource: TypedExecutionContractField<ExecutionResourcePolicy>,
    pub provenance: ExecutionContractProvenance,
}

impl ExecutionContractProfile {
    fn standard_omitted(source_file: &str, insertion: ModelSpan) -> Self {
        Self {
            schema_version: EXECUTION_CONTRACT_SCHEMA_CANDIDATE,
            profile_identity: STANDARD_REPRO_PROFILE_IDENTITY,
            profile_version: 1,
            result: omitted_execution_contract_field(
                ExecutionResultContract::CanonicalEqual,
                insertion,
            ),
            repro_scope: omitted_execution_contract_field(ExecutionReproScope::Portable, insertion),
            performance: omitted_execution_contract_field(
                ExecutionPerformancePolicy::SafeOptimization,
                insertion,
            ),
            external_ingress: omitted_execution_contract_field(
                ExecutionExternalIngressPolicy::SealedInput,
                insertion,
            ),
            evidence: omitted_execution_contract_field(
                ExecutionEvidencePolicy::MinimumEvidence,
                insertion,
            ),
            replay: omitted_execution_contract_field(
                ExecutionReplayPolicy::VerifiedReplay,
                insertion,
            ),
            resource: omitted_execution_contract_field(
                ExecutionResourcePolicy::StandardLimitV1,
                insertion,
            ),
            provenance: ExecutionContractProvenance::OmittedStandard {
                source_file: source_file.to_string(),
                source_range: insertion,
            },
        }
    }

    /// RD-090 canonical open-record order: NFC field-name UTF-8 bytes ascending.
    pub fn canonical_fields(&self) -> [(&'static str, &'static str, ModelSpan); 7] {
        [
            (
                EXECUTION_CONTRACT_FIELD_RESULT,
                self.result.value.surface(),
                self.result.value_span,
            ),
            (
                EXECUTION_CONTRACT_FIELD_PERFORMANCE,
                self.performance.value.surface(),
                self.performance.value_span,
            ),
            (
                EXECUTION_CONTRACT_FIELD_EXTERNAL_INGRESS,
                self.external_ingress.value.surface(),
                self.external_ingress.value_span,
            ),
            (
                EXECUTION_CONTRACT_FIELD_RESOURCE,
                self.resource.value.surface(),
                self.resource.value_span,
            ),
            (
                EXECUTION_CONTRACT_FIELD_REPLAY,
                self.replay.value.surface(),
                self.replay.value_span,
            ),
            (
                EXECUTION_CONTRACT_FIELD_REPRO_SCOPE,
                self.repro_scope.value.surface(),
                self.repro_scope.value_span,
            ),
            (
                EXECUTION_CONTRACT_FIELD_EVIDENCE,
                self.evidence.value.surface(),
                self.evidence.value_span,
            ),
        ]
    }

    pub fn canonical_compact(&self) -> String {
        let fields = self
            .canonical_fields()
            .into_iter()
            .map(|(name, value, _)| format!("{name}:#{value}"))
            .collect::<Vec<_>>()
            .join(",");
        format!("({fields})")
    }

    pub fn typed_input_component_hash(&self) -> String {
        hash_prefixed(
            format!(
                "profile={};version={};fields={}",
                self.profile_identity,
                self.profile_version,
                self.canonical_compact()
            )
            .as_bytes(),
        )
    }

    pub fn field_value_span(&self, name: &str) -> Option<ModelSpan> {
        self.canonical_fields()
            .into_iter()
            .find_map(|(field, _, span)| (field == name).then_some(span))
    }

    pub fn field_span(&self, name: &str) -> Option<ModelSpan> {
        match name {
            EXECUTION_CONTRACT_FIELD_RESULT => Some(self.result.field_span),
            EXECUTION_CONTRACT_FIELD_REPRO_SCOPE => Some(self.repro_scope.field_span),
            EXECUTION_CONTRACT_FIELD_PERFORMANCE => Some(self.performance.field_span),
            EXECUTION_CONTRACT_FIELD_EXTERNAL_INGRESS => Some(self.external_ingress.field_span),
            EXECUTION_CONTRACT_FIELD_EVIDENCE => Some(self.evidence.field_span),
            EXECUTION_CONTRACT_FIELD_REPLAY => Some(self.replay.field_span),
            EXECUTION_CONTRACT_FIELD_RESOURCE => Some(self.resource.field_span),
            _ => None,
        }
    }
}

fn omitted_execution_contract_field<T>(
    value: T,
    insertion: ModelSpan,
) -> TypedExecutionContractField<T> {
    TypedExecutionContractField {
        value,
        field_span: insertion,
        value_span: insertion,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolveMethodValue {
    Precise,
    HistoricalExactAcyclicRelationV1,
    Unsupported(String),
}

impl SolveMethodValue {
    fn from_expr(value: &ModelExpr) -> Self {
        match value.node.as_ref() {
            Some(ModelExprNode::Atom(name)) if name == "정밀" => Self::Precise,
            Some(ModelExprNode::String(name)) if name == EXACT_ACYCLIC_RELATION_BACKEND_ID => {
                Self::HistoricalExactAcyclicRelationV1
            }
            _ => Self::Unsupported(value.canonical.clone()),
        }
    }

    pub fn backend_id(&self) -> Option<&'static str> {
        match self {
            Self::Precise | Self::HistoricalExactAcyclicRelationV1 => {
                Some(EXACT_ACYCLIC_RELATION_BACKEND_ID)
            }
            Self::Unsupported(_) => None,
        }
    }

    pub fn diagnostic_surface(&self) -> &str {
        match self {
            Self::Precise => "#정밀",
            Self::HistoricalExactAcyclicRelationV1 => EXACT_ACYCLIC_RELATION_BACKEND_ID,
            Self::Unsupported(surface) => surface,
        }
    }

    pub fn is_historical_surface(&self) -> bool {
        matches!(self, Self::HistoricalExactAcyclicRelationV1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveRecipe {
    pub identity: String,
    pub source_hash: String,
    pub schema_version: &'static str,
    pub name: String,
    pub fields: Vec<RecordField>,
    pub method_value: Option<SolveMethodValue>,
    pub span: ModelSpan,
}

/// A bounded exact value prepared before the single model run.
///
/// This is intentionally narrower than a general top-level evaluator: it only
/// carries typed arithmetic expressions that the exact model backend can
/// evaluate deterministically.  Local-project linking uses it for exported
/// input/data module values without misclassifying those values as solve
/// recipes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactPreludeBinding {
    pub identity: String,
    pub source_hash: String,
    pub name: String,
    pub value: ModelExpr,
    pub span: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolveMethodProvenance {
    Explicit {
        source_file: String,
        source_range: ModelSpan,
    },
    OmittedFixedPrecise {
        source_file: String,
        source_range: ModelSpan,
    },
}

impl SolveMethodProvenance {
    pub fn is_authored(&self) -> bool {
        matches!(self, Self::Explicit { .. })
    }

    pub fn source_file(&self) -> &str {
        match self {
            Self::Explicit { source_file, .. }
            | Self::OmittedFixedPrecise { source_file, .. } => source_file,
        }
    }

    pub fn source_range(&self) -> ModelSpan {
        match self {
            Self::Explicit { source_range, .. }
            | Self::OmittedFixedPrecise { source_range, .. } => *source_range,
        }
    }

    pub fn surface_kind(&self) -> &'static str {
        match self {
            Self::Explicit { .. } => "explicit",
            Self::OmittedFixedPrecise { .. } => "omitted_fixed_precise",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPin {
    pub name: String,
    pub value: ModelExpr,
    pub span: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRequest {
    pub identity: String,
    pub source_hash: String,
    pub schema_version: &'static str,
    pub binding_name: String,
    pub pins: Vec<RunPin>,
    pub instance_binding: Option<String>,
    pub input_bindings: Vec<RecordField>,
    pub initial_overrides: Vec<RecordField>,
    pub execution_contract: ExecutionContractProfile,
    pub recipe_binding: Option<String>,
    pub method_provenance: SolveMethodProvenance,
    pub output_selection: Vec<String>,
    pub end: Option<ModelExpr>,
    pub span: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelView {
    pub identity: AuthoredIdentity,
    pub fields: Vec<RecordField>,
    pub span: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDiagnostic {
    /// G8 동안만 쓰는 내부 candidate code다. public stable code가 아니다.
    pub candidate_code: &'static str,
    pub phase: &'static str,
    pub severity: &'static str,
    pub effect: &'static str,
    pub message: String,
    pub provenance: ModelSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelIr {
    pub schema_version: &'static str,
    pub semantic_hash: String,
    pub definitions: Vec<ModelDefinition>,
    pub model_values: Vec<ModelValue>,
    pub instances: Vec<ModelInstance>,
    pub exact_prelude_bindings: Vec<ExactPreludeBinding>,
    pub recipes: Vec<SolveRecipe>,
    pub requests: Vec<RunRequest>,
    pub views: Vec<ModelView>,
    pub dependency_graph: BTreeMap<String, Vec<String>>,
    pub diagnostics: Vec<ModelDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelProgram {
    pub source_hash: String,
    pub canonical_ddn: String,
    pub ir: ModelIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelParseError {
    pub candidate_code: &'static str,
    pub message: String,
    pub span: ModelSpan,
}

impl fmt::Display for ModelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}..{}]: {}",
            self.candidate_code, self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for ModelParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Ident(String),
    Number(String),
    String(String),
    Atom(String),
    Colon,
    DoubleColon,
    Equal,
    RelationEqual,
    Assign,
    CrossingBoth,
    At,
    Dot,
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Prime,
    Operator(String),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    raw: String,
    span: ModelSpan,
}

struct Lexer<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, ModelParseError> {
        let mut out = Vec::new();
        while self.pos < self.source.len() {
            self.skip_ws_and_comments();
            if self.pos >= self.source.len() {
                break;
            }
            out.push(self.next_token()?);
        }
        out.push(Token {
            kind: TokenKind::Eof,
            raw: String::new(),
            span: ModelSpan {
                start: self.pos,
                end: self.pos,
            },
        });
        Ok(out)
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.bump();
            }
            if self.source[self.pos..].starts_with("//") {
                while self.peek().is_some_and(|ch| ch != '\n') {
                    self.bump();
                }
                continue;
            }
            break;
        }
    }

    fn next_token(&mut self) -> Result<Token, ModelParseError> {
        let start = self.pos;
        for (surface, kind) in [
            ("=:=", TokenKind::RelationEqual),
            ("::", TokenKind::DoubleColon),
            ("<-", TokenKind::Assign),
            ("><", TokenKind::CrossingBoth),
            ("<=", TokenKind::Operator("<=".to_string())),
            (">=", TokenKind::Operator(">=".to_string())),
            ("==", TokenKind::Operator("==".to_string())),
            ("!=", TokenKind::Operator("!=".to_string())),
        ] {
            if self.source[self.pos..].starts_with(surface) {
                self.pos += surface.len();
                return Ok(Token {
                    kind,
                    raw: surface.to_string(),
                    span: ModelSpan {
                        start,
                        end: self.pos,
                    },
                });
            }
        }
        let ch = self.peek().expect("not eof");
        if ch == '"' {
            self.bump();
            let content_start = self.pos;
            let mut escaped = false;
            while let Some(current) = self.peek() {
                if current == '"' && !escaped {
                    let content: String = self.source[content_start..self.pos].nfc().collect();
                    self.bump();
                    let raw = format!("\"{content}\"");
                    return Ok(Token {
                        kind: TokenKind::String(content),
                        raw,
                        span: ModelSpan {
                            start,
                            end: self.pos,
                        },
                    });
                }
                escaped = current == '\\' && !escaped;
                if current != '\\' {
                    escaped = false;
                }
                self.bump();
            }
            return Err(error(
                "E_MODEL_STRING_UNCLOSED_CANDIDATE",
                "문자열이 닫히지 않았습니다",
                start,
                self.pos,
            ));
        }
        if ch == '#' {
            self.bump();
            let content_start = self.pos;
            while self.peek().is_some_and(is_ident_continue) {
                self.bump();
            }
            if content_start == self.pos {
                return Err(error(
                    "E_MODEL_ATOM_EMPTY_CANDIDATE",
                    "# 뒤에 이름이 필요합니다",
                    start,
                    self.pos,
                ));
            }
            let content: String = self.source[content_start..self.pos].nfc().collect();
            return Ok(Token {
                kind: TokenKind::Atom(content.clone()),
                raw: format!("#{content}"),
                span: ModelSpan {
                    start,
                    end: self.pos,
                },
            });
        }
        if ch.is_ascii_digit() {
            self.bump();
            while self.peek().is_some_and(|next| next.is_ascii_digit()) {
                self.bump();
            }
            if self.peek() == Some('.')
                && self.source[self.pos + 1..]
                    .chars()
                    .next()
                    .is_some_and(|next| next.is_ascii_digit())
            {
                self.bump();
                while self.peek().is_some_and(|next| next.is_ascii_digit()) {
                    self.bump();
                }
            }
            let raw: String = self.source[start..self.pos].nfc().collect();
            return Ok(Token {
                kind: TokenKind::Number(raw.clone()),
                raw,
                span: ModelSpan {
                    start,
                    end: self.pos,
                },
            });
        }
        if is_ident_start(ch) {
            self.bump();
            while self.peek().is_some_and(is_ident_continue) {
                self.bump();
            }
            let raw: String = self.source[start..self.pos].nfc().collect();
            return Ok(Token {
                kind: TokenKind::Ident(raw.clone()),
                raw,
                span: ModelSpan {
                    start,
                    end: self.pos,
                },
            });
        }
        let kind = match ch {
            ':' => TokenKind::Colon,
            '=' => TokenKind::Equal,
            '@' => TokenKind::At,
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '\'' => TokenKind::Prime,
            '+' | '-' | '*' | '/' | '^' | '<' | '>' | '|' => TokenKind::Operator(ch.to_string()),
            _ => {
                self.bump();
                return Err(error(
                    "E_MODEL_UNKNOWN_CHAR_CANDIDATE",
                    format!("모형 표면에서 알 수 없는 문자: {ch}"),
                    start,
                    self.pos,
                ));
            }
        };
        self.bump();
        Ok(Token {
            kind,
            raw: self.source[start..self.pos].to_string(),
            span: ModelSpan {
                start,
                end: self.pos,
            },
        })
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn bump(&mut self) {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic() || ('가'..='힣').contains(&ch)
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}

fn error(
    code: &'static str,
    message: impl Into<String>,
    start: usize,
    end: usize,
) -> ModelParseError {
    ModelParseError {
        candidate_code: code,
        message: message.into(),
        span: ModelSpan { start, end },
    }
}

struct Parser<'a> {
    source: &'a str,
    source_file: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    occurrence: u32,
    definitions: Vec<ModelDefinition>,
    exact_prelude_bindings: Vec<ExactPreludeBinding>,
    recipes: Vec<SolveRecipe>,
    requests: Vec<RunRequest>,
    views: Vec<ModelView>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, source_file: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            source,
            source_file,
            tokens,
            pos: 0,
            occurrence: 0,
            definitions: Vec::new(),
            exact_prelude_bindings: Vec::new(),
            recipes: Vec::new(),
            requests: Vec::new(),
            views: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<ModelProgram, ModelParseError> {
        while !self.at_eof() {
            if self.is_model_definition_start() {
                let definition = self.parse_definition()?;
                self.definitions.push(definition);
                continue;
            }
            if self.is_assignment_start() {
                self.parse_top_level_assignment()?;
                continue;
            }
            if self.is_view_start() {
                let view = self.parse_view()?;
                self.views.push(view);
                continue;
            }
            let token = self.current().clone();
            return Err(error(
                "E_MODEL_TOP_LEVEL_CANDIDATE",
                "모형 frontdoor에는 모형 정의, 방법 record, 풀기 요청, 결과표 보임만 둘 수 있습니다",
                token.span.start,
                token.span.end,
            ));
        }
        if self.definitions.is_empty() && self.requests.is_empty() {
            return Err(error(
                "E_MODEL_SURFACE_MISSING_CANDIDATE",
                "모형 정의 또는 풀기 요청이 없습니다",
                0,
                self.source.len(),
            ));
        }

        let source_hash = hash_prefixed(self.source.as_bytes());
        let mut ir = build_ir(
            self.definitions,
            self.exact_prelude_bindings,
            self.recipes,
            self.requests,
            self.views,
        );
        validate_fixed_default_recipe_name(&ir)?;
        let canonical_ddn = canonical_program(&ir);
        ir.semantic_hash = hash_prefixed(semantic_canon(&ir).as_bytes());
        Ok(ModelProgram {
            source_hash,
            canonical_ddn,
            ir,
        })
    }

    fn is_model_definition_start(&self) -> bool {
        matches!(self.kind(0), Some(TokenKind::Ident(_)))
            && matches!(self.kind(1), Some(TokenKind::Equal))
            && matches!(self.kind(2), Some(TokenKind::Ident(name)) if name == "모형")
            && matches!(self.kind(3), Some(TokenKind::LBrace))
    }

    fn is_assignment_start(&self) -> bool {
        matches!(self.kind(0), Some(TokenKind::Ident(_)))
            && matches!(self.kind(1), Some(TokenKind::Assign))
    }

    fn is_view_start(&self) -> bool {
        if !matches!(self.kind(0), Some(TokenKind::LParen)) {
            return false;
        }
        let Some(close) =
            matching_close(&self.tokens, self.pos, TokenKind::LParen, TokenKind::RParen)
        else {
            return false;
        };
        self.tokens
            .get(close + 1)
            .and_then(token_ident)
            .is_some_and(|word| word == "보임")
    }

    fn parse_definition(&mut self) -> Result<ModelDefinition, ModelParseError> {
        let start = self.current().span.start;
        let name = self.expect_ident("모형 이름")?;
        self.expect_kind(TokenKind::Equal, "'='")?;
        self.expect_named("모형")?;
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let header_span = ModelSpan {
            start,
            end: self.previous().span.end,
        };
        let body_start_pos = self.pos;
        let mut declarations = Vec::new();
        let mut relations = Vec::new();
        let mut events = Vec::new();
        let mut connections = Vec::new();
        let mut authored_order = Vec::new();
        let mut unknowns: Vec<(String, ModelSpan)> = Vec::new();

        while !matches!(self.current().kind, TokenKind::RBrace | TokenKind::Eof) {
            let statement = self.take_model_statement()?;
            if statement.is_empty() {
                continue;
            }
            if is_unknown_statement(&statement) {
                let name = token_ident(&statement[0])
                    .expect("classified ident")
                    .to_string();
                unknowns.push((name.clone(), span_of(&statement)));
                authored_order.push(ModelBodyItem::Unknown(name));
            } else if is_declaration_statement(&statement) {
                let declaration = self.parse_declaration(&name, &statement)?;
                authored_order.push(ModelBodyItem::Declaration(declaration.name.clone()));
                declarations.push(declaration);
            } else if statement
                .iter()
                .any(|t| matches!(t.kind, TokenKind::RelationEqual))
            {
                let relation = self.parse_relation(&name, &statement)?;
                authored_order.push(ModelBodyItem::Relation(relations.len()));
                relations.push(relation);
            } else if is_event_statement(&statement) {
                let event = self.parse_event(&name, &statement)?;
                authored_order.push(ModelBodyItem::Event(events.len()));
                events.push(event);
            } else if statement
                .iter()
                .rev()
                .find_map(token_ident)
                .is_some_and(|word| word == "잇기")
            {
                let connection = self.parse_connection(&name, &statement)?;
                authored_order.push(ModelBodyItem::Connection(connections.len()));
                connections.push(connection);
            } else {
                let span = span_of(&statement);
                return Err(error(
                    "E_MODEL_BODY_ITEM_CANDIDATE",
                    "모형 몸통에는 선언, 관계, 사건, 잇기만 둘 수 있습니다",
                    span.start,
                    span.end,
                ));
            }
        }
        if matches!(self.current().kind, TokenKind::Eof) {
            return Err(error(
                "E_MODEL_BLOCK_UNCLOSED_CANDIDATE",
                "모형 블록이 닫히지 않았습니다",
                start,
                self.current().span.end,
            ));
        }
        let body_end_pos = self.pos;
        self.advance();
        if matches!(self.current().kind, TokenKind::Dot) {
            self.advance();
        }
        let end = self.previous().span.end;

        let mut seen = HashMap::new();
        for declaration in &declarations {
            if let Some(previous) = seen.insert(declaration.name.clone(), declaration.identity.span)
            {
                return Err(error(
                    "E_MODEL_DECL_DUP_CANDIDATE",
                    format!(
                        "선언 '{}'이 중복되었습니다(첫 위치 {}..{})",
                        declaration.name, previous.start, previous.end
                    ),
                    declaration.identity.span.start,
                    declaration.identity.span.end,
                ));
            }
        }
        for (unknown, span) in unknowns {
            let Some(declaration) = declarations.iter_mut().find(|decl| decl.name == unknown)
            else {
                return Err(error(
                    "E_MODEL_UNKNOWN_SLOT_UNDECLARED_CANDIDATE",
                    format!("구함 대상 '{unknown}' 선언이 없습니다"),
                    span.start,
                    span.end,
                ));
            };
            if declaration.unknown {
                return Err(error(
                    "E_MODEL_UNKNOWN_SLOT_DUP_CANDIDATE",
                    format!("'{unknown}::구함'이 중복되었습니다"),
                    span.start,
                    span.end,
                ));
            }
            declaration.unknown = true;
        }

        let body_span = if body_start_pos < body_end_pos {
            self.tokens[body_start_pos]
                .span
                .merge(self.tokens[body_end_pos - 1].span)
        } else {
            ModelSpan { start, end }
        };
        let source_hash = hash_prefixed(self.source[start..end].as_bytes());
        let identity = format!("model:{}:{}", name, source_hash);
        let mut required_capabilities = BTreeSet::new();
        if !relations.is_empty() {
            required_capabilities.insert("relation".to_string());
        }
        if relations.iter().any(relation_has_derivative) {
            required_capabilities.insert("derivative".to_string());
        }
        if !events.is_empty() {
            required_capabilities.insert("event".to_string());
        }
        if !connections.is_empty() {
            required_capabilities.insert("connection_runtime".to_string());
        }
        Ok(ModelDefinition {
            identity,
            source_hash,
            schema_version: MODEL_IR_SCHEMA_CANDIDATE,
            name,
            span: ModelSpan { start, end },
            header_span,
            body_span,
            declarations,
            relations,
            events,
            connections,
            authored_order,
            required_capabilities,
        })
    }

    fn take_model_statement(&mut self) -> Result<Vec<Token>, ModelParseError> {
        let start = self.pos;
        let mut paren = 0i32;
        let mut brace = 0i32;
        while !matches!(self.current().kind, TokenKind::Eof) {
            match self.current().kind {
                TokenKind::LParen => paren += 1,
                TokenKind::RParen => paren -= 1,
                TokenKind::LBrace => brace += 1,
                TokenKind::RBrace if brace > 0 => brace -= 1,
                TokenKind::RBrace if brace == 0 && paren == 0 => {
                    if start == self.pos {
                        return Ok(Vec::new());
                    }
                    let span = self.tokens[start].span.merge(self.previous().span);
                    return Err(error(
                        "E_MODEL_TERMINATOR_CANDIDATE",
                        "모형 몸통 항목 끝에 '.'이 필요합니다",
                        span.start,
                        span.end,
                    ));
                }
                TokenKind::Dot if paren == 0 && brace == 0 && !self.current_dot_is_path() => {
                    let end = self.pos;
                    self.advance();
                    return Ok(self.tokens[start..end].to_vec());
                }
                _ => {}
            }
            self.advance();
        }
        let span = self.tokens[start].span.merge(self.current().span);
        Err(error(
            "E_MODEL_TERMINATOR_CANDIDATE",
            "모형 몸통 항목 끝에 '.'이 필요합니다",
            span.start,
            span.end,
        ))
    }

    fn parse_declaration(
        &mut self,
        model_name: &str,
        tokens: &[Token],
    ) -> Result<ModelDeclaration, ModelParseError> {
        let span = span_of(tokens);
        let name = token_ident(&tokens[0])
            .expect("classified declaration")
            .to_string();
        let property_at = tokens
            .iter()
            .position(|token| matches!(token.kind, TokenKind::DoubleColon))
            .expect("classified declaration");
        let type_tokens = &tokens[2..property_at];
        if type_tokens.is_empty() {
            return Err(error(
                "E_MODEL_TYPE_MISSING_CANDIDATE",
                format!("선언 '{name}'의 씨가 없습니다"),
                span.start,
                span.end,
            ));
        }
        let at = type_tokens
            .iter()
            .position(|token| matches!(token.kind, TokenKind::At));
        let (type_part, unit_part) = match at {
            Some(index) => (&type_tokens[..index], Some(&type_tokens[index + 1..])),
            None => (type_tokens, None),
        };
        let type_name = canonical_tokens(type_part);
        let unit = unit_part
            .map(compact_tokens)
            .filter(|text| !text.is_empty());
        let dimension = match unit.as_deref() {
            Some(unit) => Some(parse_unit_dimension(unit).map_err(|message| {
                error(
                    "E_MODEL_UNIT_INVALID_CANDIDATE",
                    message,
                    span.start,
                    span.end,
                )
            })?),
            None => Some(UnitDim::NONE),
        };
        let property_tokens = &tokens[property_at + 1..];
        if property_tokens.len() < 2
            || !matches!(
                property_tokens.first().map(|t| &t.kind),
                Some(TokenKind::LParen)
            )
            || !matches!(
                property_tokens.last().map(|t| &t.kind),
                Some(TokenKind::RParen)
            )
        {
            return Err(error(
                "E_MODEL_PROPERTY_LIST_CANDIDATE",
                "'::' 뒤에는 닫힌 성질 목록 '(...)'이 필요합니다",
                span.start,
                span.end,
            ));
        }
        let groups = split_top_level(&property_tokens[1..property_tokens.len() - 1], |kind| {
            matches!(kind, TokenKind::Comma)
        });
        let mut roles = Vec::new();
        let mut initialization = Initialization::None;
        let mut accepts_initial_override = false;
        let mut exported = false;
        let mut seen_properties = HashSet::new();
        for group in groups {
            if group.is_empty() {
                continue;
            }
            let property_surface = canonical_tokens(group);
            if !seen_properties.insert(property_surface.clone()) {
                return Err(error(
                    "E_MODEL_PROPERTY_DUP_CANDIDATE",
                    format!("성질 '{property_surface}'이 중복되었습니다"),
                    span_of(group).start,
                    span_of(group).end,
                ));
            }
            if group.len() == 1 {
                let Some(word) = token_ident(&group[0]) else {
                    return Err(unknown_property(group));
                };
                if let Some(role) = PrimaryRole::parse(word) {
                    roles.push(role);
                } else {
                    match word {
                        "처음받음" => accepts_initial_override = true,
                        "내줌" => exported = true,
                        _ => return Err(unknown_property(group)),
                    }
                }
                continue;
            }
            if group.len() >= 3
                && token_ident(&group[0]) == Some("처음")
                && matches!(group[1].kind, TokenKind::DoubleColon)
            {
                let Some(mode) = token_ident(&group[2]) else {
                    return Err(unknown_property(group));
                };
                let next = match mode {
                    "계산" if group.len() == 3 => Initialization::Computed,
                    "고정" | "어림"
                        if group.len() >= 5 && matches!(group[3].kind, TokenKind::Equal) =>
                    {
                        let expr = parse_expr_tokens(&group[4..], None)?;
                        if mode == "고정" {
                            Initialization::Fixed(expr)
                        } else {
                            Initialization::Guess(expr)
                        }
                    }
                    _ => return Err(unknown_property(group)),
                };
                if !matches!(initialization, Initialization::None) {
                    return Err(error(
                        "E_MODEL_INIT_CONFLICT_CANDIDATE",
                        "초기화 성질은 하나만 허용됩니다",
                        span_of(group).start,
                        span_of(group).end,
                    ));
                }
                initialization = next;
                continue;
            }
            return Err(unknown_property(group));
        }
        if roles.len() != 1 {
            return Err(error(
                "E_MODEL_PRIMARY_ROLE_COUNT_CANDIDATE",
                format!("선언 '{name}'의 주된 역할은 정확히 하나여야 합니다"),
                span.start,
                span.end,
            ));
        }
        let identity = self.authored_identity(model_name, "decl", &name, span);
        Ok(ModelDeclaration {
            identity,
            name,
            model_type: ModelType {
                name: type_name,
                unit,
                dimension,
            },
            role: roles[0],
            initialization,
            accepts_initial_override,
            exported,
            unknown: false,
        })
    }

    fn parse_relation(
        &mut self,
        model_name: &str,
        tokens: &[Token],
    ) -> Result<ModelRelation, ModelParseError> {
        let index = tokens
            .iter()
            .position(|token| matches!(token.kind, TokenKind::RelationEqual))
            .expect("classified relation");
        if index == 0 || index + 1 == tokens.len() {
            let span = span_of(tokens);
            return Err(error(
                "E_MODEL_RELATION_SHAPE_CANDIDATE",
                "관계 양쪽에 식이 필요합니다",
                span.start,
                span.end,
            ));
        }
        let left = parse_expr_tokens(&tokens[..index], None)?;
        let right = parse_expr_tokens(&tokens[index + 1..], None)?;
        let span = span_of(tokens);
        let semantic_surface = directionless_relation_canon(&left, &right);
        Ok(ModelRelation {
            identity: self.authored_identity(model_name, "relation", "relation", span),
            semantic_identity: format!(
                "semantic-relation:{}",
                hash_prefixed(semantic_surface.as_bytes())
            ),
            left,
            right,
            lowered: None,
        })
    }

    fn parse_event(
        &mut self,
        model_name: &str,
        tokens: &[Token],
    ) -> Result<ModelEvent, ModelParseError> {
        let span = span_of(tokens);
        let open = tokens
            .iter()
            .position(|token| matches!(token.kind, TokenKind::LBrace))
            .ok_or_else(|| {
                error(
                    "E_MODEL_EVENT_BODY_CANDIDATE",
                    "사건 몸통이 필요합니다",
                    span.start,
                    span.end,
                )
            })?;
        let close = matching_close(tokens, open, TokenKind::LBrace, TokenKind::RBrace).ok_or_else(
            || {
                error(
                    "E_MODEL_EVENT_BODY_CANDIDATE",
                    "사건 몸통이 닫히지 않았습니다",
                    span.start,
                    span.end,
                )
            },
        )?;
        let header = &tokens[..open];
        let has_becomes = header
            .iter()
            .any(|token| token_ident(token) == Some("될때"));
        let has_while = contains_adjacent_words(header, "인", "동안");
        let every_madi_index = header.windows(2).position(|window| {
            matches!(window[0].kind, TokenKind::Number(_))
                && token_ident(&window[1]) == Some("마디마다")
        });
        let has_crossing = header
            .iter()
            .any(|token| matches!(token.kind, TokenKind::CrossingBoth));
        let marker_count = usize::from(has_becomes)
            + usize::from(has_while)
            + usize::from(every_madi_index.is_some())
            + usize::from(has_crossing);
        if marker_count != 1 {
            return Err(error(
                "E_MODEL_EVENT_KIND_CANDIDATE",
                "사건 표면은 될때, 인 동안, N마디마다, >< 중 정확히 하나여야 합니다",
                span.start,
                span.end,
            ));
        }
        let kind = if has_becomes {
            ModelEventKind::Becomes
        } else if has_while {
            ModelEventKind::While
        } else if has_crossing {
            ModelEventKind::CrossingBoth
        } else if let Some(index) = every_madi_index {
            let TokenKind::Number(value) = &header[index].kind else {
                unreachable!("N마디마다 marker starts with a number")
            };
            let n = value.parse::<u64>().map_err(|_| {
                error(
                    "E_MODEL_EVENT_INTERVAL_CANDIDATE",
                    "N마디마다의 N은 바른수여야 합니다",
                    header[index].span.start,
                    header[index].span.end,
                )
            })?;
            if n == 0 {
                return Err(error(
                    "E_MODEL_EVENT_INTERVAL_CANDIDATE",
                    "N마디마다의 N은 양수여야 합니다",
                    span.start,
                    span.end,
                ));
            }
            ModelEventKind::EveryMadi(n)
        } else {
            return Err(error(
                "E_MODEL_EVENT_KIND_CANDIDATE",
                "지원되는 사건은 될때, 인 동안, N마디마다, >< 입니다",
                span.start,
                span.end,
            ));
        };
        let trigger_end = if has_becomes {
            let marker = header
                .iter()
                .position(|token| token_ident(token) == Some("될때"))
                .expect("checked becomes marker");
            marker.saturating_sub(usize::from(
                marker > 0 && token_ident(&header[marker - 1]) == Some("이"),
            ))
        } else if has_while {
            header
                .windows(2)
                .position(|window| {
                    token_ident(&window[0]) == Some("인") && token_ident(&window[1]) == Some("동안")
                })
                .expect("checked while marker")
        } else if has_crossing {
            header
                .iter()
                .position(|token| matches!(token.kind, TokenKind::CrossingBoth))
                .expect("checked crossing marker")
        } else {
            every_madi_index.expect("checked interval marker") + 1
        };
        let trigger_tokens = trim_outer_parens(&header[..trigger_end]);
        let trigger = parse_expr_tokens(trigger_tokens, None)?;
        let identity = self.authored_identity(model_name, "event", "event", span);
        let resets = parse_event_resets(&identity.stable_id, &tokens[open + 1..close])?;
        let semantic_surface = event_semantic_surface(&kind, &trigger, &resets);
        let semantic_identity = format!(
            "semantic-event:{}",
            hash_prefixed(semantic_surface.as_bytes())
        );
        let mut reset_targets = resets
            .iter()
            .map(|reset| reset.target.clone())
            .collect::<Vec<_>>();
        reset_targets.sort();
        let detection = match kind {
            ModelEventKind::Becomes => EventDetectionPolicy::FalseToTrueTransition,
            ModelEventKind::While => EventDetectionPolicy::WhileGuard,
            ModelEventKind::EveryMadi(n) => EventDetectionPolicy::MadiInterval(n),
            ModelEventKind::CrossingBoth => EventDetectionPolicy::BidirectionalContinuousCrossing,
        };
        let lowered = LoweredEventNode {
            identity: format!(
                "lowered-event:{}",
                hash_prefixed(semantic_surface.as_bytes())
            ),
            authored_event_identity: identity.stable_id.clone(),
            semantic_identity: semantic_identity.clone(),
            remember_last_nonzero_sign: matches!(
                detection,
                EventDetectionPolicy::BidirectionalContinuousCrossing
            ),
            detection,
            simultaneous_grouping: true,
            reset_conflict_policy: "fail_closed",
            reset_targets,
            required_capability: "event",
            provenance: span,
        };
        let body_span = if open + 1 == close {
            ModelSpan {
                start: tokens[open].span.end,
                end: tokens[close].span.start,
            }
        } else {
            span_of(&tokens[open + 1..close])
        };
        Ok(ModelEvent {
            identity,
            semantic_identity,
            kind,
            trigger,
            resets,
            lowered,
            body_span,
            canonical: canonical_tokens(tokens),
        })
    }

    fn parse_connection(
        &mut self,
        model_name: &str,
        tokens: &[Token],
    ) -> Result<ModelConnection, ModelParseError> {
        let span = span_of(tokens);
        let open = tokens
            .iter()
            .position(|token| matches!(token.kind, TokenKind::LParen))
            .ok_or_else(|| {
                error(
                    "E_MODEL_CONNECTION_SHAPE_CANDIDATE",
                    "잇기에는 관계 목록 '(...)'이 필요합니다",
                    span.start,
                    span.end,
                )
            })?;
        let close = matching_close(tokens, open, TokenKind::LParen, TokenKind::RParen).ok_or_else(
            || {
                error(
                    "E_MODEL_CONNECTION_SHAPE_CANDIDATE",
                    "잇기 관계 목록이 닫히지 않았습니다",
                    span.start,
                    span.end,
                )
            },
        )?;
        let endpoints = collect_paths(&tokens[..open]);
        if endpoints.len() < 2 {
            return Err(error(
                "E_MODEL_CONNECTION_ARITY_CANDIDATE",
                "잇기는 둘 이상의 ordinary instance path가 필요합니다",
                span.start,
                span.end,
            ));
        }
        if endpoints.iter().any(|endpoint| !endpoint.contains('.')) {
            return Err(error(
                "E_MODEL_INSTANCE_PATH_CANDIDATE",
                "잇기 endpoint는 'instance.member' ordinary path여야 합니다",
                span.start,
                span.end,
            ));
        }
        let mut clauses = Vec::new();
        let mut clause_names = HashSet::new();
        for group in split_top_level(&tokens[open + 1..close], |kind| {
            matches!(kind, TokenKind::Comma)
        }) {
            let has_reverse = group
                .iter()
                .any(|token| token_ident(token) == Some("거슬러"));
            let has_flow = group
                .iter()
                .any(|token| token_ident(token) == Some("흐르게"));
            let has_equal = group.iter().any(|token| token_ident(token) == Some("같게"));
            if has_reverse && !has_flow {
                return Err(error(
                    "E_MODEL_CONNECTION_DIRECTION_CANDIDATE",
                    "거슬러 방향은 흐르게 관계에만 사용할 수 있습니다",
                    span_of(group).start,
                    span_of(group).end,
                ));
            }
            if usize::from(has_flow) + usize::from(has_equal) != 1 {
                return Err(error(
                    "E_MODEL_CONNECTION_MODE_CANDIDATE",
                    "잇기 관계는 같게 또는 흐르게 중 정확히 하나여야 합니다",
                    span_of(group).start,
                    span_of(group).end,
                ));
            }
            let mode = if has_reverse {
                ConnectionMode::ReverseFlow
            } else if has_flow {
                ConnectionMode::Flow
            } else if has_equal {
                ConnectionMode::Equal
            } else {
                unreachable!("exactly one connection mode was checked")
            };
            let quantity = group
                .iter()
                .find_map(token_ident)
                .unwrap_or("")
                .trim_end_matches(['은', '는'])
                .to_string();
            if quantity.is_empty() {
                return Err(error(
                    "E_MODEL_CONNECTION_QUANTITY_CANDIDATE",
                    "잇기 관계의 양 이름이 필요합니다",
                    span_of(group).start,
                    span_of(group).end,
                ));
            }
            if !clause_names.insert(quantity.clone()) {
                return Err(error(
                    "E_MODEL_CONNECTION_QUANTITY_DUP_CANDIDATE",
                    format!("잇기 양 '{quantity}'가 중복되었습니다"),
                    span_of(group).start,
                    span_of(group).end,
                ));
            }
            clauses.push(ConnectionClause {
                quantity,
                mode,
                provenance: span_of(group),
            });
        }
        let identity = self.authored_identity(model_name, "connection", "connection", span);
        let semantic_surface = connection_semantic_surface(&endpoints, &clauses);
        let semantic_identity = format!(
            "semantic-connection:{}",
            hash_prefixed(semantic_surface.as_bytes())
        );
        let lowered_relations = clauses
            .iter()
            .map(|clause| LoweredConnectionRelation {
                identity: format!(
                    "lowered-connection:{}",
                    hash_prefixed(format!("{}|{}", semantic_surface, clause.quantity).as_bytes())
                ),
                authored_connection_identity: identity.stable_id.clone(),
                semantic_identity: semantic_identity.clone(),
                quantity: clause.quantity.clone(),
                mode: clause.mode,
                orientation_sign: match clause.mode {
                    ConnectionMode::Equal => None,
                    ConnectionMode::Flow => Some(1),
                    ConnectionMode::ReverseFlow => Some(-1),
                },
                endpoints: endpoints.clone(),
                required_capability: "connection_runtime",
                provenance: clause.provenance,
            })
            .collect();
        Ok(ModelConnection {
            identity,
            semantic_identity,
            endpoints,
            clauses,
            lowered_relations,
            canonical: canonical_tokens(tokens),
        })
    }

    fn parse_view(&mut self) -> Result<ModelView, ModelParseError> {
        let open = self.pos;
        let close = matching_close(&self.tokens, open, TokenKind::LParen, TokenKind::RParen)
            .ok_or_else(|| {
                let span = self.current().span;
                error(
                    "E_MODEL_VIEW_SHAPE_CANDIDATE",
                    "결과표 보임 값꾸러미가 닫히지 않았습니다",
                    span.start,
                    span.end,
                )
            })?;
        let view_word = self.tokens.get(close + 1).cloned().ok_or_else(|| {
            let span = self.tokens[open].span.merge(self.tokens[close].span);
            error(
                "E_MODEL_VIEW_KEYWORD_CANDIDATE",
                "결과표 값꾸러미 뒤에 '보임'이 필요합니다",
                span.start,
                span.end,
            )
        })?;
        if token_ident(&view_word) != Some("보임") {
            return Err(error(
                "E_MODEL_VIEW_KEYWORD_CANDIDATE",
                "결과표 값꾸러미 뒤에 '보임'이 필요합니다",
                view_word.span.start,
                view_word.span.end,
            ));
        }
        let terminator = self.tokens.get(close + 2).cloned().ok_or_else(|| {
            error(
                "E_MODEL_VIEW_TERMINATOR_CANDIDATE",
                "결과표 보임 문장 끝에 '.'이 필요합니다",
                view_word.span.start,
                view_word.span.end,
            )
        })?;
        if !matches!(terminator.kind, TokenKind::Dot) {
            return Err(error(
                "E_MODEL_VIEW_TERMINATOR_CANDIDATE",
                "결과표 보임 문장 끝에 '.'이 필요합니다",
                terminator.span.start,
                terminator.span.end,
            ));
        }
        let span = self.tokens[open].span.merge(terminator.span);
        let fields = parse_named_record_fields(
            &self.tokens[open..=close],
            "E_MODEL_VIEW_FIELD_CANDIDATE",
            "결과표 보임 필드는 '이름: 값'이어야 합니다",
        )?;
        if fields.is_empty() {
            return Err(error(
                "E_MODEL_VIEW_EMPTY_CANDIDATE",
                "결과표 보임에는 한 개 이상의 필드가 필요합니다",
                span.start,
                span.end,
            ));
        }
        self.pos = close + 3;
        Ok(ModelView {
            identity: self.authored_identity("program", "view", "result-table", span),
            fields,
            span,
        })
    }

    fn parse_top_level_assignment(&mut self) -> Result<(), ModelParseError> {
        let start_pos = self.pos;
        let name = self.expect_ident("받을 이름")?;
        self.expect_kind(TokenKind::Assign, "'<-'")?;
        let expression_start = self.pos;
        let mut paren = 0i32;
        let mut brace = 0i32;
        while !matches!(self.current().kind, TokenKind::Eof) {
            match self.current().kind {
                TokenKind::LParen => paren += 1,
                TokenKind::RParen => paren -= 1,
                TokenKind::LBrace => brace += 1,
                TokenKind::RBrace => brace -= 1,
                TokenKind::Dot if paren == 0 && brace == 0 && !self.current_dot_is_path() => break,
                _ => {}
            }
            self.advance();
        }
        if matches!(self.current().kind, TokenKind::Eof) {
            let span = self.tokens[start_pos].span.merge(self.current().span);
            return Err(error(
                "E_MODEL_TOP_LEVEL_TERMINATOR_CANDIDATE",
                "최상위 문장 끝에 '.'이 필요합니다",
                span.start,
                span.end,
            ));
        }
        let expression_end = self.pos;
        self.advance();
        let statement_span = self.tokens[start_pos]
            .span
            .merge(self.tokens[expression_end].span);
        let expression = &self.tokens[expression_start..expression_end];
        let is_request = expression
            .iter()
            .rev()
            .find_map(token_ident)
            .is_some_and(|word| word == "풀기");
        let source_hash =
            hash_prefixed(self.source[statement_span.start..statement_span.end].as_bytes());
        if is_request {
            let mut request =
                parse_run_request(&name, expression, statement_span, self.source_file)?;
            request.source_hash = source_hash;
            self.requests.push(request);
        } else if outer_paren_contents(expression).is_some() {
            let mut recipe = parse_recipe(&name, expression, statement_span)?;
            recipe.source_hash = source_hash;
            self.recipes.push(recipe);
        } else {
            let value = parse_expr_tokens(expression, None)?;
            if value.node.is_none()
                || matches!(
                    value.node.as_ref(),
                    Some(
                        ModelExprNode::String(_)
                            | ModelExprNode::Atom(_)
                            | ModelExprNode::Derivative(_)
                    )
                )
            {
                return Err(error(
                    "E_MODEL_EXACT_PRELUDE_VALUE_CANDIDATE",
                    "자료 바인딩은 exact 산술 값 또는 앞선 자료 이름이어야 합니다",
                    statement_span.start,
                    statement_span.end,
                ));
            }
            let canonical = format!("{}<-{}", name, value.canonical);
            self.exact_prelude_bindings.push(ExactPreludeBinding {
                identity: format!(
                    "exact-prelude:{}:{}",
                    name,
                    hash_prefixed(canonical.as_bytes())
                ),
                source_hash,
                name,
                value,
                span: statement_span,
            });
        }
        Ok(())
    }

    fn authored_identity(
        &mut self,
        model_name: &str,
        kind: &str,
        local_name: &str,
        span: ModelSpan,
    ) -> AuthoredIdentity {
        self.occurrence += 1;
        AuthoredIdentity {
            stable_id: format!("{}:{}:{}:{}", model_name, kind, local_name, self.occurrence),
            occurrence: self.occurrence,
            span,
        }
    }

    fn expect_ident(&mut self, expected: &str) -> Result<String, ModelParseError> {
        let token = self.current().clone();
        let TokenKind::Ident(value) = token.kind else {
            return Err(error(
                "E_MODEL_EXPECTED_NAME_CANDIDATE",
                format!("{expected}이 필요합니다"),
                token.span.start,
                token.span.end,
            ));
        };
        self.advance();
        Ok(value)
    }

    fn expect_named(&mut self, expected: &str) -> Result<(), ModelParseError> {
        let token = self.current().clone();
        if token_ident(&token) != Some(expected) {
            return Err(error(
                "E_MODEL_EXPECTED_KEYWORD_CANDIDATE",
                format!("'{expected}'이 필요합니다"),
                token.span.start,
                token.span.end,
            ));
        }
        self.advance();
        Ok(())
    }

    fn expect_kind(&mut self, expected: TokenKind, surface: &str) -> Result<(), ModelParseError> {
        let token = self.current().clone();
        if std::mem::discriminant(&token.kind) != std::mem::discriminant(&expected) {
            return Err(error(
                "E_MODEL_EXPECTED_TOKEN_CANDIDATE",
                format!("{surface}가 필요합니다"),
                token.span.start,
                token.span.end,
            ));
        }
        self.advance();
        Ok(())
    }

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos.saturating_sub(1)]
    }

    fn kind(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens.get(self.pos + offset).map(|token| &token.kind)
    }

    fn advance(&mut self) {
        if !self.at_eof() {
            self.pos += 1;
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    fn current_dot_is_path(&self) -> bool {
        matches!(self.current().kind, TokenKind::Dot)
            && self.pos > 0
            && matches!(self.tokens[self.pos - 1].kind, TokenKind::Ident(_))
            && matches!(
                self.tokens.get(self.pos + 1).map(|token| &token.kind),
                Some(TokenKind::Ident(_))
            )
            && self.tokens[self.pos - 1].span.end == self.current().span.start
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|next| self.current().span.end == next.span.start)
    }
}

fn is_declaration_statement(tokens: &[Token]) -> bool {
    tokens.len() >= 4
        && matches!(tokens[0].kind, TokenKind::Ident(_))
        && matches!(tokens[1].kind, TokenKind::Colon)
        && tokens
            .iter()
            .skip(2)
            .any(|token| matches!(token.kind, TokenKind::DoubleColon))
}

fn is_unknown_statement(tokens: &[Token]) -> bool {
    tokens.len() == 3
        && matches!(tokens[0].kind, TokenKind::Ident(_))
        && matches!(tokens[1].kind, TokenKind::DoubleColon)
        && token_ident(&tokens[2]) == Some("구함")
}

fn is_event_statement(tokens: &[Token]) -> bool {
    let has_body = tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::LBrace));
    let has_exact_marker = tokens
        .iter()
        .any(|token| token_ident(token) == Some("될때"))
        || contains_adjacent_words(tokens, "인", "동안")
        || tokens.windows(2).any(|window| {
            matches!(window[0].kind, TokenKind::Number(_))
                && token_ident(&window[1]) == Some("마디마다")
        })
        || tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::CrossingBoth));
    has_body && has_exact_marker
}

fn contains_adjacent_words(tokens: &[Token], first: &str, second: &str) -> bool {
    tokens.windows(2).any(|window| {
        token_ident(&window[0]) == Some(first) && token_ident(&window[1]) == Some(second)
    })
}

fn parse_event_resets(
    event_identity: &str,
    tokens: &[Token],
) -> Result<Vec<ModelReset>, ModelParseError> {
    if tokens.is_empty() {
        return Ok(Vec::new());
    }
    let mut groups = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket => depth += 1,
            TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket => depth -= 1,
            TokenKind::Dot if depth == 0 && !dot_is_path(tokens, index) => {
                groups.push(&tokens[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if start != tokens.len() {
        let span = span_of(&tokens[start..]);
        return Err(error(
            "E_MODEL_EVENT_RESET_TERMINATOR_CANDIDATE",
            "사건 reset 문장 끝에 '.'이 필요합니다",
            span.start,
            span.end,
        ));
    }

    let mut resets = Vec::new();
    let mut seen_targets = HashSet::new();
    for group in groups.into_iter().filter(|group| !group.is_empty()) {
        let Some(assign) = find_top_level(group, |kind| matches!(kind, TokenKind::Assign)) else {
            let span = span_of(group);
            return Err(error(
                "E_MODEL_EVENT_RESET_SHAPE_CANDIDATE",
                "사건 몸통은 '<-' reset 문장만 허용합니다",
                span.start,
                span.end,
            ));
        };
        let target_expr = parse_expr_tokens(&group[..assign], None)?;
        let ModelExprKind::Path(target) = target_expr.kind else {
            return Err(error(
                "E_MODEL_EVENT_RESET_TARGET_CANDIDATE",
                "사건 reset target은 ordinary path여야 합니다",
                target_expr.span.start,
                target_expr.span.end,
            ));
        };
        if !seen_targets.insert(target.clone()) {
            let span = span_of(group);
            return Err(error(
                "E_MODEL_EVENT_RESET_DUP_CANDIDATE",
                format!("한 사건에서 reset target '{target}'이 중복되었습니다"),
                span.start,
                span.end,
            ));
        }
        let value = parse_expr_tokens(&group[assign + 1..], None)?;
        let provenance = span_of(group);
        resets.push(ModelReset {
            identity: format!("{event_identity}:reset:{target}:{}", resets.len() + 1),
            event_identity: event_identity.to_string(),
            target,
            value,
            provenance,
        });
    }
    Ok(resets)
}

fn dot_is_path(tokens: &[Token], index: usize) -> bool {
    index > 0
        && index + 1 < tokens.len()
        && matches!(tokens[index - 1].kind, TokenKind::Ident(_))
        && matches!(tokens[index + 1].kind, TokenKind::Ident(_))
}

fn unknown_property(tokens: &[Token]) -> ModelParseError {
    let span = span_of(tokens);
    error(
        "E_MODEL_PROPERTY_UNKNOWN_CANDIDATE",
        format!("알 수 없는 닫힌 모형 성질: {}", canonical_tokens(tokens)),
        span.start,
        span.end,
    )
}

fn parse_recipe(
    name: &str,
    expression: &[Token],
    span: ModelSpan,
) -> Result<SolveRecipe, ModelParseError> {
    let inner = outer_paren_contents(expression).ok_or_else(|| {
        error(
            "E_MODEL_RECIPE_SHAPE_CANDIDATE",
            "방법은 열린 이름형 값꾸러미 '(이름: 값, ...)'여야 합니다",
            span.start,
            span.end,
        )
    })?;
    let mut fields = Vec::new();
    let mut seen = HashSet::new();
    for group in split_top_level(inner, |kind| matches!(kind, TokenKind::Comma)) {
        if group.is_empty() {
            continue;
        }
        let delimiter = find_top_level(group, |kind| {
            matches!(kind, TokenKind::Colon | TokenKind::Equal)
        })
        .ok_or_else(|| {
            let item_span = span_of(group);
            error(
                "E_MODEL_RECIPE_FIELD_DELIMITER_CANDIDATE",
                "방법 필드에는 ':'가 필요합니다",
                item_span.start,
                item_span.end,
            )
        })?;
        if matches!(group[delimiter].kind, TokenKind::Equal) {
            let item_span = span_of(group);
            return Err(error(
                "E_MODEL_RECIPE_FIELD_EQUALS_CANDIDATE",
                "열린 방법 필드는 '='가 아니라 ':'를 사용합니다",
                item_span.start,
                item_span.end,
            ));
        }
        if delimiter != 1 {
            let item_span = span_of(group);
            return Err(error(
                "E_MODEL_RECIPE_FIELD_NAME_CANDIDATE",
                "방법 필드 이름은 단일 이름이어야 합니다",
                item_span.start,
                item_span.end,
            ));
        }
        let field_name = token_ident(&group[0]).ok_or_else(|| {
            let item_span = span_of(group);
            error(
                "E_MODEL_RECIPE_FIELD_NAME_CANDIDATE",
                "방법 필드 이름이 필요합니다",
                item_span.start,
                item_span.end,
            )
        })?;
        if !seen.insert(field_name.to_string()) {
            let item_span = span_of(group);
            return Err(error(
                "E_MODEL_RECIPE_FIELD_DUP_CANDIDATE",
                format!("방법 필드 '{field_name}'이 중복되었습니다"),
                item_span.start,
                item_span.end,
            ));
        }
        fields.push(RecordField {
            name: field_name.to_string(),
            value: parse_expr_tokens(&group[delimiter + 1..], None)?,
            span: span_of(group),
        });
    }
    let method_value = fields
        .iter()
        .find(|field| field.name == "방식")
        .map(|field| SolveMethodValue::from_expr(&field.value));
    let canonical = canonical_tokens(expression);
    Ok(SolveRecipe {
        identity: format!("recipe:{}:{}", name, hash_prefixed(canonical.as_bytes())),
        source_hash: hash_prefixed(canonical.as_bytes()),
        schema_version: MODEL_IR_SCHEMA_CANDIDATE,
        name: name.to_string(),
        fields,
        method_value,
        span,
    })
}

#[derive(Debug, Clone, Copy)]
struct ParsedExecutionContractField {
    field_span: ModelSpan,
    value_span: ModelSpan,
}

fn expected_execution_contract_value(field: &str) -> Option<&'static str> {
    match field {
        EXECUTION_CONTRACT_FIELD_RESULT => Some(EXECUTION_CONTRACT_VALUE_CANONICAL_EQUAL),
        EXECUTION_CONTRACT_FIELD_REPRO_SCOPE => Some(EXECUTION_CONTRACT_VALUE_PORTABLE),
        EXECUTION_CONTRACT_FIELD_PERFORMANCE => Some(EXECUTION_CONTRACT_VALUE_SAFE_OPTIMIZATION),
        EXECUTION_CONTRACT_FIELD_EXTERNAL_INGRESS => Some(EXECUTION_CONTRACT_VALUE_SEALED_INPUT),
        EXECUTION_CONTRACT_FIELD_EVIDENCE => Some(EXECUTION_CONTRACT_VALUE_MINIMUM_EVIDENCE),
        EXECUTION_CONTRACT_FIELD_REPLAY => Some(EXECUTION_CONTRACT_VALUE_VERIFIED_REPLAY),
        EXECUTION_CONTRACT_FIELD_RESOURCE => Some(EXECUTION_CONTRACT_VALUE_STANDARD_LIMIT_V1),
        _ => None,
    }
}

fn is_standard_execution_contract_atom(value: &str) -> bool {
    [
        EXECUTION_CONTRACT_VALUE_CANONICAL_EQUAL,
        EXECUTION_CONTRACT_VALUE_PORTABLE,
        EXECUTION_CONTRACT_VALUE_SAFE_OPTIMIZATION,
        EXECUTION_CONTRACT_VALUE_SEALED_INPUT,
        EXECUTION_CONTRACT_VALUE_MINIMUM_EVIDENCE,
        EXECUTION_CONTRACT_VALUE_VERIFIED_REPLAY,
        EXECUTION_CONTRACT_VALUE_STANDARD_LIMIT_V1,
    ]
    .contains(&value)
}

fn parse_execution_contract(
    tokens: &[Token],
    pin_span: ModelSpan,
    source_file: &str,
) -> Result<ExecutionContractProfile, ModelParseError> {
    let inner = outer_paren_contents(tokens).ok_or_else(|| {
        let value_span = span_of(tokens);
        error(
            "E_MODEL_EXECUTION_CONTRACT_VALUE_UNKNOWN_CANDIDATE",
            "실행계약은 profile atom shorthand가 아니라 열린 이름형 값꾸러미여야 합니다",
            value_span.start,
            value_span.end,
        )
    })?;
    let mut fields = HashMap::<String, ParsedExecutionContractField>::new();
    for group in split_top_level(inner, |kind| matches!(kind, TokenKind::Comma)) {
        if group.is_empty() {
            continue;
        }
        let field_span = span_of(group);
        let delimiter = find_top_level(group, |kind| {
            matches!(kind, TokenKind::Colon | TokenKind::Equal)
        })
        .ok_or_else(|| {
            error(
                "E_MODEL_EXECUTION_CONTRACT_FIELD_DELIMITER_CANDIDATE",
                "실행계약 내부 필드는 '이름: 값' 형식이어야 합니다",
                field_span.start,
                field_span.end,
            )
        })?;
        if delimiter != 1 || matches!(group[delimiter].kind, TokenKind::Equal) {
            return Err(error(
                "E_MODEL_EXECUTION_CONTRACT_FIELD_DELIMITER_CANDIDATE",
                "실행계약 내부 필드는 '='가 아니라 ':'를 사용합니다",
                field_span.start,
                field_span.end,
            ));
        }
        let field_name = token_ident(&group[0]).ok_or_else(|| {
            error(
                "E_MODEL_EXECUTION_CONTRACT_FIELD_UNKNOWN_CANDIDATE",
                "실행계약 필드 이름은 승인된 한국어 이름이어야 합니다",
                field_span.start,
                field_span.end,
            )
        })?;
        let Some(expected_value) = expected_execution_contract_value(field_name) else {
            return Err(error(
                "E_MODEL_EXECUTION_CONTRACT_FIELD_UNKNOWN_CANDIDATE",
                format!("알 수 없는 실행계약 필드: {field_name}"),
                field_span.start,
                field_span.end,
            ));
        };
        if fields.contains_key(field_name) {
            return Err(error(
                "E_MODEL_EXECUTION_CONTRACT_FIELD_DUPLICATE_CANDIDATE",
                format!("실행계약 필드 '{field_name}'이 중복되었습니다"),
                field_span.start,
                field_span.end,
            ));
        }
        let value_tokens = &group[delimiter + 1..];
        let value_span = span_of(value_tokens);
        let value = parse_expr_tokens(value_tokens, None).map_err(|_| {
            error(
                "E_MODEL_EXECUTION_CONTRACT_VALUE_UNKNOWN_CANDIDATE",
                format!("실행계약 필드 '{field_name}'의 값은 승인된 atom이어야 합니다"),
                value_span.start,
                value_span.end,
            )
        })?;
        let actual_value = match value.node.as_ref() {
            Some(ModelExprNode::Atom(value)) => value.as_str(),
            _ => {
                return Err(error(
                    "E_MODEL_EXECUTION_CONTRACT_VALUE_UNKNOWN_CANDIDATE",
                    format!("실행계약 필드 '{field_name}'의 값은 승인된 atom이어야 합니다"),
                    value_span.start,
                    value_span.end,
                ));
            }
        };
        if actual_value != expected_value {
            let (code, message) = if is_standard_execution_contract_atom(actual_value) {
                (
                    "E_MODEL_EXECUTION_CONTRACT_COMBINATION_UNSUPPORTED_CANDIDATE",
                    format!(
                        "실행계약 필드 '{field_name}'에는 #{expected_value}가 필요하며 #{} 조합은 지원하지 않습니다",
                        actual_value
                    ),
                )
            } else {
                (
                    "E_MODEL_EXECUTION_CONTRACT_VALUE_UNKNOWN_CANDIDATE",
                    format!(
                        "알 수 없는 실행계약 값: 필드 '{field_name}', 실제 #{}",
                        actual_value
                    ),
                )
            };
            return Err(error(code, message, value_span.start, value_span.end));
        }
        fields.insert(
            field_name.to_string(),
            ParsedExecutionContractField {
                field_span,
                value_span,
            },
        );
    }

    let missing = [
        EXECUTION_CONTRACT_FIELD_RESULT,
        EXECUTION_CONTRACT_FIELD_REPRO_SCOPE,
        EXECUTION_CONTRACT_FIELD_PERFORMANCE,
        EXECUTION_CONTRACT_FIELD_EXTERNAL_INGRESS,
        EXECUTION_CONTRACT_FIELD_EVIDENCE,
        EXECUTION_CONTRACT_FIELD_REPLAY,
        EXECUTION_CONTRACT_FIELD_RESOURCE,
    ]
    .into_iter()
    .filter(|name| !fields.contains_key(*name))
    .collect::<Vec<_>>();
    if !missing.is_empty() {
        let insertion = tokens.last().map_or(pin_span.end, |token| token.span.start);
        return Err(error(
            "E_MODEL_EXECUTION_CONTRACT_FIELD_MISSING_CANDIDATE",
            format!("실행계약 필드가 빠졌습니다: {}", missing.join(",")),
            insertion,
            insertion,
        ));
    }

    let mut take = |name: &str| {
        fields
            .remove(name)
            .expect("all execution-contract fields checked")
    };
    let result = take(EXECUTION_CONTRACT_FIELD_RESULT);
    let repro_scope = take(EXECUTION_CONTRACT_FIELD_REPRO_SCOPE);
    let performance = take(EXECUTION_CONTRACT_FIELD_PERFORMANCE);
    let external_ingress = take(EXECUTION_CONTRACT_FIELD_EXTERNAL_INGRESS);
    let evidence = take(EXECUTION_CONTRACT_FIELD_EVIDENCE);
    let replay = take(EXECUTION_CONTRACT_FIELD_REPLAY);
    let resource = take(EXECUTION_CONTRACT_FIELD_RESOURCE);
    Ok(ExecutionContractProfile {
        schema_version: EXECUTION_CONTRACT_SCHEMA_CANDIDATE,
        profile_identity: STANDARD_REPRO_PROFILE_IDENTITY,
        profile_version: 1,
        result: TypedExecutionContractField {
            value: ExecutionResultContract::CanonicalEqual,
            field_span: result.field_span,
            value_span: result.value_span,
        },
        repro_scope: TypedExecutionContractField {
            value: ExecutionReproScope::Portable,
            field_span: repro_scope.field_span,
            value_span: repro_scope.value_span,
        },
        performance: TypedExecutionContractField {
            value: ExecutionPerformancePolicy::SafeOptimization,
            field_span: performance.field_span,
            value_span: performance.value_span,
        },
        external_ingress: TypedExecutionContractField {
            value: ExecutionExternalIngressPolicy::SealedInput,
            field_span: external_ingress.field_span,
            value_span: external_ingress.value_span,
        },
        evidence: TypedExecutionContractField {
            value: ExecutionEvidencePolicy::MinimumEvidence,
            field_span: evidence.field_span,
            value_span: evidence.value_span,
        },
        replay: TypedExecutionContractField {
            value: ExecutionReplayPolicy::VerifiedReplay,
            field_span: replay.field_span,
            value_span: replay.value_span,
        },
        resource: TypedExecutionContractField {
            value: ExecutionResourcePolicy::StandardLimitV1,
            field_span: resource.field_span,
            value_span: resource.value_span,
        },
        provenance: ExecutionContractProvenance::Explicit {
            source_file: source_file.to_string(),
            source_range: pin_span,
        },
    })
}

fn parse_run_request(
    name: &str,
    expression: &[Token],
    span: ModelSpan,
    source_file: &str,
) -> Result<RunRequest, ModelParseError> {
    let solve_index = expression
        .iter()
        .rposition(|token| token_ident(token) == Some("풀기"))
        .expect("classified request");
    if solve_index + 1 != expression.len() {
        return Err(error(
            "E_MODEL_RUN_REQUEST_TAIL_CANDIDATE",
            "풀기는 요청 값꾸러미 바로 뒤에 와야 합니다",
            span.start,
            span.end,
        ));
    }
    let inner = outer_paren_contents(&expression[..solve_index]).ok_or_else(|| {
        error(
            "E_MODEL_RUN_REQUEST_SHAPE_CANDIDATE",
            "풀기 앞에는 고정 핀 값꾸러미가 필요합니다",
            span.start,
            span.end,
        )
    })?;
    let allowed: HashSet<&str> = ["대상", "들임", "처음", "실행계약", "방법", "내줌", "끝"]
        .into_iter()
        .collect();
    let mut pins = Vec::new();
    let mut seen = HashSet::new();
    let mut instance_binding = None;
    let mut input_bindings = Vec::new();
    let mut initial_overrides = Vec::new();
    let mut execution_contract = None;
    let mut recipe_binding = None;
    let mut method_source_range = None;
    let mut output_selection = Vec::new();
    let mut end = None;
    for group in split_top_level(inner, |kind| matches!(kind, TokenKind::Comma)) {
        if group.is_empty() {
            continue;
        }
        let delimiter = find_top_level(group, |kind| {
            matches!(kind, TokenKind::Equal | TokenKind::Colon)
        })
        .ok_or_else(|| {
            let item_span = span_of(group);
            error(
                "E_MODEL_RUN_PIN_DELIMITER_CANDIDATE",
                "풀기 고정 핀에는 '='가 필요합니다",
                item_span.start,
                item_span.end,
            )
        })?;
        if matches!(group[delimiter].kind, TokenKind::Colon) {
            let item_span = span_of(group);
            return Err(error(
                "E_MODEL_RUN_PIN_COLON_CANDIDATE",
                "풀기 바깥 고정 핀은 ':'가 아니라 '='를 사용합니다",
                item_span.start,
                item_span.end,
            ));
        }
        if delimiter != 1 {
            let item_span = span_of(group);
            return Err(error(
                "E_MODEL_RUN_PIN_NAME_CANDIDATE",
                "풀기 핀 이름은 단일 이름이어야 합니다",
                item_span.start,
                item_span.end,
            ));
        }
        let pin_name = token_ident(&group[0]).ok_or_else(|| {
            let item_span = span_of(group);
            error(
                "E_MODEL_RUN_PIN_NAME_CANDIDATE",
                "풀기 핀 이름이 필요합니다",
                item_span.start,
                item_span.end,
            )
        })?;
        if !allowed.contains(pin_name) {
            let item_span = span_of(group);
            return Err(error(
                "E_MODEL_RUN_PIN_UNKNOWN_CANDIDATE",
                format!("알 수 없는 풀기 고정 핀: {pin_name}"),
                item_span.start,
                item_span.end,
            ));
        }
        if !seen.insert(pin_name.to_string()) {
            let item_span = span_of(group);
            return Err(error(
                "E_MODEL_RUN_PIN_DUP_CANDIDATE",
                format!("풀기 핀 '{pin_name}'이 중복되었습니다"),
                item_span.start,
                item_span.end,
            ));
        }
        let value_tokens = &group[delimiter + 1..];
        let value = if pin_name == "실행계약" {
            let contract = parse_execution_contract(value_tokens, span_of(group), source_file)?;
            let value_span = span_of(value_tokens);
            let canonical = contract.canonical_compact();
            execution_contract = Some(contract);
            ModelExpr {
                span: value_span,
                canonical,
                kind: ModelExprKind::Compound,
                node: None,
                references: Vec::new(),
                dimension: None,
            }
        } else {
            parse_expr_tokens(value_tokens, None)?
        };
        match pin_name {
            "대상" => {
                instance_binding = direct_path(&value).map(str::to_string);
            }
            "들임" => {
                input_bindings = parse_named_record_fields(
                    value_tokens,
                    "E_MODEL_RUN_INPUT_FIELD_CANDIDATE",
                    "들임은 열린 이름형 값꾸러미 '(이름: 값, ...)'여야 합니다",
                )?;
            }
            "처음" => {
                initial_overrides = parse_named_record_fields(
                    value_tokens,
                    "E_MODEL_RUN_INITIAL_FIELD_CANDIDATE",
                    "처음은 열린 이름형 값꾸러미 '(이름: 값, ...)'여야 합니다",
                )?;
            }
            "실행계약" => {}
            "방법" => {
                recipe_binding = direct_path(&value).map(str::to_string);
                method_source_range = Some(span_of(group));
            }
            "내줌" => {
                output_selection = parse_ordered_name_list(value_tokens)?;
            }
            "끝" => end = Some(value.clone()),
            _ => unreachable!("closed run pin registry"),
        }
        pins.push(RunPin {
            name: pin_name.to_string(),
            value,
            span: span_of(group),
        });
    }
    let canonical = canonical_tokens(expression);
    let method_provenance = method_source_range.map_or_else(
        || {
            let insertion = expression
                .get(solve_index.saturating_sub(1))
                .map_or(span.end, |token| token.span.start);
            SolveMethodProvenance::OmittedFixedPrecise {
                source_file: source_file.to_string(),
                source_range: ModelSpan {
                    start: insertion,
                    end: insertion,
                },
            }
        },
        |source_range| SolveMethodProvenance::Explicit {
            source_file: source_file.to_string(),
            source_range,
        },
    );
    let execution_contract = execution_contract.unwrap_or_else(|| {
        let insertion = pins
            .iter()
            .rev()
            .find(|pin| pin.name == "처음")
            .or_else(|| pins.iter().rev().find(|pin| pin.name == "들임"))
            .or_else(|| pins.iter().rev().find(|pin| pin.name == "대상"))
            .map_or(span.end, |pin| pin.span.end);
        ExecutionContractProfile::standard_omitted(
            source_file,
            ModelSpan {
                start: insertion,
                end: insertion,
            },
        )
    });
    Ok(RunRequest {
        identity: format!("run:{}:{}", name, hash_prefixed(canonical.as_bytes())),
        source_hash: hash_prefixed(canonical.as_bytes()),
        schema_version: MODEL_IR_SCHEMA_CANDIDATE,
        binding_name: name.to_string(),
        pins,
        instance_binding,
        input_bindings,
        initial_overrides,
        execution_contract,
        recipe_binding,
        method_provenance,
        output_selection,
        end,
        span,
    })
}

fn direct_path(expr: &ModelExpr) -> Option<&str> {
    match &expr.kind {
        ModelExprKind::Path(path) if !path.contains('.') => Some(path),
        _ => None,
    }
}

fn parse_named_record_fields(
    tokens: &[Token],
    code: &'static str,
    message: &'static str,
) -> Result<Vec<RecordField>, ModelParseError> {
    let inner = outer_paren_contents(tokens).ok_or_else(|| {
        let span = span_of(tokens);
        error(code, message, span.start, span.end)
    })?;
    let mut fields = Vec::new();
    let mut seen = HashSet::new();
    for group in split_top_level(inner, |kind| matches!(kind, TokenKind::Comma)) {
        if group.is_empty() {
            continue;
        }
        let delimiter = find_top_level(group, |kind| {
            matches!(kind, TokenKind::Colon | TokenKind::Equal)
        })
        .ok_or_else(|| {
            let span = span_of(group);
            error(code, message, span.start, span.end)
        })?;
        if delimiter != 1 || matches!(group[delimiter].kind, TokenKind::Equal) {
            let span = span_of(group);
            return Err(error(code, message, span.start, span.end));
        }
        let name = token_ident(&group[0]).ok_or_else(|| {
            let span = span_of(group);
            error(code, message, span.start, span.end)
        })?;
        if !seen.insert(name.to_string()) {
            let span = span_of(group);
            return Err(error(
                "E_MODEL_OPEN_FIELD_DUP_CANDIDATE",
                format!("열린 값꾸러미 필드 '{name}'이 중복되었습니다"),
                span.start,
                span.end,
            ));
        }
        fields.push(RecordField {
            name: name.to_string(),
            value: parse_expr_tokens(&group[delimiter + 1..], None)?,
            span: span_of(group),
        });
    }
    Ok(fields)
}

fn parse_ordered_name_list(tokens: &[Token]) -> Result<Vec<String>, ModelParseError> {
    let inner = outer_paren_contents(tokens).ok_or_else(|| {
        let span = span_of(tokens);
        error(
            "E_MODEL_OUTPUT_SELECTION_CANDIDATE",
            "내줌은 ordered 이름 목록 '(이름, ...)'이어야 합니다",
            span.start,
            span.end,
        )
    })?;
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    for group in split_top_level(inner, |kind| matches!(kind, TokenKind::Comma)) {
        if group.len() != 1 {
            let span = span_of(group);
            return Err(error(
                "E_MODEL_OUTPUT_SELECTION_CANDIDATE",
                "내줌 항목은 단일 선언 이름이어야 합니다",
                span.start,
                span.end,
            ));
        }
        let name = token_ident(&group[0]).ok_or_else(|| {
            let span = span_of(group);
            error(
                "E_MODEL_OUTPUT_SELECTION_CANDIDATE",
                "내줌 항목은 단일 선언 이름이어야 합니다",
                span.start,
                span.end,
            )
        })?;
        if !seen.insert(name.to_string()) {
            let span = span_of(group);
            return Err(error(
                "E_MODEL_OUTPUT_SELECTION_DUP_CANDIDATE",
                format!("내줌 이름 '{name}'이 중복되었습니다"),
                span.start,
                span.end,
            ));
        }
        names.push(name.to_string());
    }
    Ok(names)
}

fn parse_expr_tokens(tokens: &[Token], axis: Option<&str>) -> Result<ModelExpr, ModelParseError> {
    if tokens.is_empty() {
        return Err(error(
            "E_MODEL_EXPR_MISSING_CANDIDATE",
            "식이 필요합니다",
            0,
            0,
        ));
    }
    let span = span_of(tokens);
    let node = ModelExprNodeParser::new(tokens, axis).parse()?;
    let canonical = node
        .as_ref()
        .map(canonical_expr_node)
        .unwrap_or_else(|| canonical_tokens(tokens));
    // Token-level path collection confuses an attached unit (`1@m`) with a
    // model value reference. The parsed expression node already distinguishes
    // those roles, so dependency and declaration checks must use it.
    let mut references = Vec::new();
    if let Some(node) = &node {
        collect_expr_references(node, &mut references);
    }
    let kind = match &node {
        Some(ModelExprNode::Path(path)) => ModelExprKind::Path(path.clone()),
        Some(ModelExprNode::Derivative(identity)) => ModelExprKind::Derivative(identity.clone()),
        Some(ModelExprNode::Number { .. } | ModelExprNode::String(_) | ModelExprNode::Atom(_)) => {
            ModelExprKind::Literal
        }
        _ => ModelExprKind::Compound,
    };
    let dimension = match &node {
        Some(ModelExprNode::Number { dimension, .. }) => Some(*dimension),
        _ => None,
    };
    Ok(ModelExpr {
        span,
        canonical,
        kind,
        node,
        references,
        dimension,
    })
}

fn collect_expr_references(node: &ModelExprNode, references: &mut Vec<String>) {
    match node {
        ModelExprNode::Path(path) => {
            if !references.contains(path) {
                references.push(path.clone());
            }
        }
        ModelExprNode::Derivative(identity) => {
            if !references.contains(&identity.base) {
                references.push(identity.base.clone());
            }
        }
        ModelExprNode::Unary { operand, .. } | ModelExprNode::Group(operand) => {
            collect_expr_references(operand, references);
        }
        ModelExprNode::Binary { left, right, .. } => {
            collect_expr_references(left, references);
            collect_expr_references(right, references);
        }
        ModelExprNode::Number { .. } | ModelExprNode::String(_) | ModelExprNode::Atom(_) => {}
    }
}

fn canonical_expr_node(node: &ModelExprNode) -> String {
    match node {
        ModelExprNode::Number { surface, unit, .. } => unit
            .as_ref()
            .map(|unit| format!("{surface}@{unit}"))
            .unwrap_or_else(|| surface.clone()),
        ModelExprNode::String(value) => format!("\"{value}\""),
        ModelExprNode::Atom(value) => format!("#{value}"),
        ModelExprNode::Path(path) => path.clone(),
        ModelExprNode::Derivative(identity) => {
            format!("{}{}", identity.base, "'".repeat(identity.order as usize))
        }
        ModelExprNode::Unary { operator, operand } => {
            format!("{operator}{}", canonical_expr_node(operand))
        }
        ModelExprNode::Binary {
            operator,
            left,
            right,
        } => format!(
            "{} {operator} {}",
            canonical_expr_node(left),
            canonical_expr_node(right)
        ),
        ModelExprNode::Group(inner) => format!("({})", canonical_expr_node(inner)),
    }
}

struct ModelExprNodeParser<'a> {
    tokens: &'a [Token],
    pos: usize,
    axis: Option<&'a str>,
}

impl<'a> ModelExprNodeParser<'a> {
    fn new(tokens: &'a [Token], axis: Option<&'a str>) -> Self {
        Self {
            tokens,
            pos: 0,
            axis,
        }
    }

    fn parse(mut self) -> Result<Option<ModelExprNode>, ModelParseError> {
        let Some(node) = self.parse_binary(0)? else {
            return Ok(None);
        };
        if self.pos == self.tokens.len() {
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    fn parse_binary(
        &mut self,
        minimum_precedence: u8,
    ) -> Result<Option<ModelExprNode>, ModelParseError> {
        let Some(mut left) = self.parse_primary()? else {
            return Ok(None);
        };
        loop {
            let Some((operator, precedence, right_associative)) = self.current_binary_operator()
            else {
                break;
            };
            if precedence < minimum_precedence {
                break;
            }
            self.pos += 1;
            let next_minimum = if right_associative {
                precedence
            } else {
                precedence + 1
            };
            let Some(right) = self.parse_binary(next_minimum)? else {
                return Ok(None);
            };
            left = ModelExprNode::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(Some(left))
    }

    fn parse_primary(&mut self) -> Result<Option<ModelExprNode>, ModelParseError> {
        let Some(token) = self.tokens.get(self.pos) else {
            return Ok(None);
        };
        if matches!(&token.kind, TokenKind::Operator(operator) if operator == "+" || operator == "-")
        {
            let operator = token.raw.clone();
            self.pos += 1;
            let Some(operand) = self.parse_primary()? else {
                return Ok(None);
            };
            return Ok(Some(ModelExprNode::Unary {
                operator,
                operand: Box::new(operand),
            }));
        }
        match &token.kind {
            TokenKind::Number(surface) => {
                let surface = surface.clone();
                self.pos += 1;
                let (unit, dimension) = self.parse_attached_unit()?;
                Ok(Some(ModelExprNode::Number {
                    surface,
                    unit,
                    dimension,
                }))
            }
            TokenKind::String(value) => {
                let value = value.clone();
                self.pos += 1;
                Ok(Some(ModelExprNode::String(value)))
            }
            TokenKind::Atom(value) => {
                let value = value.clone();
                self.pos += 1;
                Ok(Some(ModelExprNode::Atom(value)))
            }
            TokenKind::Ident(_) => self.parse_path_or_derivative(),
            TokenKind::LParen => {
                self.pos += 1;
                let Some(inner) = self.parse_binary(0)? else {
                    return Ok(None);
                };
                if !matches!(
                    self.tokens.get(self.pos).map(|token| &token.kind),
                    Some(TokenKind::RParen)
                ) {
                    return Ok(None);
                }
                self.pos += 1;
                Ok(Some(ModelExprNode::Group(Box::new(inner))))
            }
            _ => Ok(None),
        }
    }

    fn parse_path_or_derivative(&mut self) -> Result<Option<ModelExprNode>, ModelParseError> {
        let start = self.pos;
        self.pos += 1;
        while self.pos + 1 < self.tokens.len()
            && matches!(self.tokens[self.pos].kind, TokenKind::Dot)
            && matches!(self.tokens[self.pos + 1].kind, TokenKind::Ident(_))
        {
            self.pos += 2;
        }
        let path_end = self.pos;
        while self.pos < self.tokens.len() && matches!(self.tokens[self.pos].kind, TokenKind::Prime)
        {
            self.pos += 1;
        }
        let order = self.pos - path_end;
        if order > 2 {
            let span = span_of(&self.tokens[start..self.pos]);
            return Err(error(
                "E_MODEL_DERIVATIVE_ORDER_CANDIDATE",
                "prime 미분 차수는 현재 1 또는 2만 허용됩니다",
                span.start,
                span.end,
            ));
        }
        let path = canonical_tokens(&self.tokens[start..path_end]);
        if order == 0 {
            Ok(Some(ModelExprNode::Path(path)))
        } else {
            Ok(Some(ModelExprNode::Derivative(DerivativeIdentity {
                base: path,
                axis: self.axis.map(str::to_string),
                order: order as u8,
            })))
        }
    }

    fn parse_attached_unit(&mut self) -> Result<(Option<String>, UnitDim), ModelParseError> {
        if !matches!(
            self.tokens.get(self.pos).map(|token| &token.kind),
            Some(TokenKind::At)
        ) {
            return Ok((None, UnitDim::NONE));
        }
        let at = self.pos;
        self.pos += 1;
        let unit_start = self.pos;
        if !self.consume_unit_factor() {
            let span = self.tokens[at].span;
            return Err(error(
                "E_MODEL_UNIT_INVALID_CANDIDATE",
                "'@' 뒤에 단위가 필요합니다",
                span.start,
                span.end,
            ));
        }
        loop {
            let Some(Token {
                kind: TokenKind::Operator(operator),
                ..
            }) = self.tokens.get(self.pos)
            else {
                break;
            };
            if operator != "*" && operator != "/" {
                break;
            }
            let operator_pos = self.pos;
            self.pos += 1;
            if !self.consume_unit_factor() {
                self.pos = operator_pos;
                break;
            }
        }
        let unit = compact_tokens(&self.tokens[unit_start..self.pos]);
        let dimension = parse_unit_dimension(&unit).map_err(|message| {
            let span = span_of(&self.tokens[at..self.pos]);
            error(
                "E_MODEL_UNIT_INVALID_CANDIDATE",
                message,
                span.start,
                span.end,
            )
        })?;
        Ok((Some(unit), dimension))
    }

    fn consume_unit_factor(&mut self) -> bool {
        let Some(Token {
            kind: TokenKind::Ident(symbol),
            ..
        }) = self.tokens.get(self.pos)
        else {
            return false;
        };
        if unit_spec_from_symbol(symbol).is_none() {
            return false;
        }
        self.pos += 1;
        if matches!(
            self.tokens.get(self.pos).map(|token| &token.kind),
            Some(TokenKind::Operator(operator)) if operator == "^"
        ) && matches!(
            self.tokens.get(self.pos + 1).map(|token| &token.kind),
            Some(TokenKind::Number(_))
        ) {
            self.pos += 2;
        }
        true
    }

    fn current_binary_operator(&self) -> Option<(String, u8, bool)> {
        let TokenKind::Operator(operator) = &self.tokens.get(self.pos)?.kind else {
            return None;
        };
        let (precedence, right_associative) = match operator.as_str() {
            "<" | ">" | "<=" | ">=" | "==" | "!=" => (0, false),
            "+" | "-" => (1, false),
            "*" | "/" => (2, false),
            "^" => (3, true),
            _ => return None,
        };
        Some((operator.clone(), precedence, right_associative))
    }
}

fn build_ir(
    mut definitions: Vec<ModelDefinition>,
    exact_prelude_bindings: Vec<ExactPreludeBinding>,
    recipes: Vec<SolveRecipe>,
    requests: Vec<RunRequest>,
    views: Vec<ModelView>,
) -> ModelIr {
    let mut diagnostics = Vec::new();
    let mut dependency_graph = BTreeMap::new();
    for definition in &mut definitions {
        let axes: Vec<String> = definition
            .declarations
            .iter()
            .filter(|declaration| declaration.role == PrimaryRole::Axis)
            .map(|declaration| declaration.name.clone())
            .collect();
        let axis = if axes.len() == 1 {
            Some(axes[0].clone())
        } else {
            None
        };
        let declarations: HashMap<String, ModelDeclaration> = definition
            .declarations
            .iter()
            .cloned()
            .map(|declaration| (declaration.name.clone(), declaration))
            .collect();
        let explicit_unknowns: HashSet<String> = definition
            .declarations
            .iter()
            .filter(|declaration| declaration.unknown)
            .map(|declaration| declaration.name.clone())
            .collect();
        let mut algebraic_relation_count = 0usize;
        let mut producers: HashMap<String, Vec<ModelSpan>> = HashMap::new();
        for relation in &mut definition.relations {
            bind_derivative_axis(&mut relation.left, axis.as_deref());
            bind_derivative_axis(&mut relation.right, axis.as_deref());
            let has_derivative = relation_has_derivative(relation);
            if !has_derivative {
                algebraic_relation_count += 1;
            }
            relation.lowered = lower_relation_node(&definition.name, relation, &explicit_unknowns);
            if contains_unbound_derivative(&relation.left)
                || contains_unbound_derivative(&relation.right)
            {
                diagnostics.push(diag(
                    "E_MODEL_DERIVATIVE_AXIS_MISSING_CANDIDATE",
                    "미분 축을 추정하지 않습니다. 모형 안에 정확히 하나의 '축' 선언이 필요합니다",
                    relation.identity.span,
                ));
            }
            if !has_derivative && relation.lowered.is_none() {
                let (code, message) = algebraic_target_diagnostic(relation, &explicit_unknowns);
                diagnostics.push(diag(code, message, relation.identity.span));
            }
            for reference in relation
                .left
                .references
                .iter()
                .chain(&relation.right.references)
            {
                if !declarations.contains_key(reference) {
                    diagnostics.push(diag(
                        "E_MODEL_RELATION_REFERENCE_UNDECLARED_CANDIDATE",
                        format!("관계가 선언되지 않은 값 '{reference}'을 참조합니다"),
                        relation.identity.span,
                    ));
                }
            }
            if let Some(LoweredRelationNode {
                target: LoweredRelationTarget::Algebraic(target),
                dependencies,
                ..
            }) = &relation.lowered
            {
                let target = target.clone();
                let target_producers = producers.entry(target.clone()).or_default();
                if let Some(previous) = target_producers.first() {
                    diagnostics.push(diag(
                        "E_MODEL_MULTI_PRODUCER_CANDIDATE",
                        format!(
                            "관계 target '{target}'에 둘 이상의 producer가 있습니다(첫 위치 {}..{})",
                            previous.start, previous.end
                        ),
                        relation.identity.span,
                    ));
                }
                target_producers.push(relation.identity.span);
                let entry = dependency_graph
                    .entry(format!("{}.{}", definition.name, target))
                    .or_insert_with(Vec::new);
                for dependency in dependencies {
                    if dependency != &target && !entry.contains(dependency) {
                        entry.push(dependency.clone());
                    }
                }
            }
            validate_relation_dimensions(
                relation,
                &declarations,
                axis.as_deref(),
                &mut diagnostics,
            );
        }
        if algebraic_relation_count < explicit_unknowns.len() {
            diagnostics.push(diag(
                "E_MODEL_UNDERDETERMINED_CANDIDATE",
                format!(
                    "대수 관계 {}개보다 구함 slot {}개가 많아 미결정 후보입니다",
                    algebraic_relation_count,
                    explicit_unknowns.len()
                ),
                definition.span,
            ));
        } else if algebraic_relation_count > explicit_unknowns.len() {
            diagnostics.push(diag(
                "E_MODEL_OVERDETERMINED_CANDIDATE",
                format!(
                    "구함 slot {}개보다 대수 관계 {}개가 많아 과결정 후보입니다",
                    explicit_unknowns.len(),
                    algebraic_relation_count
                ),
                definition.span,
            ));
        }
        for declaration in &definition.declarations {
            let producer_count = producers
                .get(&declaration.name)
                .map_or(0, |items| items.len());
            if declaration.unknown && producer_count == 0 {
                diagnostics.push(diag(
                    "E_MODEL_UNDERDETERMINED_CANDIDATE",
                    format!(
                        "구함 slot '{}'을 계산할 relation producer가 없습니다",
                        declaration.name
                    ),
                    declaration.identity.span,
                ));
            }
            if declaration.unknown && producer_count > 1 {
                diagnostics.push(diag(
                    "E_MODEL_OVERDETERMINED_CANDIDATE",
                    format!(
                        "구함 slot '{}'에 relation producer가 {}개여서 과결정 후보입니다",
                        declaration.name, producer_count
                    ),
                    declaration.identity.span,
                ));
            }
            if declaration.role == PrimaryRole::Algebraic
                && !declaration.unknown
                && producer_count == 0
                && matches!(declaration.initialization, Initialization::None)
            {
                diagnostics.push(diag(
                    "E_MODEL_UNDERDETERMINED_CANDIDATE",
                    format!(
                        "그자리 선언 '{}'의 producer 또는 구함 표시가 없습니다",
                        declaration.name
                    ),
                    declaration.identity.span,
                ));
            }
        }
        for event in &definition.events {
            for reset in &event.resets {
                if !declarations.contains_key(&reset.target) {
                    diagnostics.push(diag(
                        "E_MODEL_EVENT_RESET_TARGET_UNDECLARED_CANDIDATE",
                        format!("사건 reset target '{}' 선언이 없습니다", reset.target),
                        reset.provenance,
                    ));
                }
            }
        }
        detect_cycles(definition, &dependency_graph, &mut diagnostics);
    }
    let model_values = definitions
        .iter()
        .map(|definition| ModelValue {
            identity: format!("model-value:{}", definition.identity),
            definition_identity: definition.identity.clone(),
            public_type: "모형값",
            provenance: definition.span,
        })
        .collect();
    validate_instance_and_request_bindings(&definitions, &[], &requests, &mut diagnostics);
    ModelIr {
        schema_version: MODEL_IR_SCHEMA_CANDIDATE,
        semantic_hash: String::new(),
        definitions,
        model_values,
        instances: Vec::new(),
        exact_prelude_bindings,
        recipes,
        requests,
        views,
        dependency_graph,
        diagnostics,
    }
}

fn validate_instance_and_request_bindings(
    definitions: &[ModelDefinition],
    instances: &[ModelInstance],
    requests: &[RunRequest],
    diagnostics: &mut Vec<ModelDiagnostic>,
) {
    let definitions_by_id: HashMap<&str, &ModelDefinition> = definitions
        .iter()
        .map(|definition| (definition.identity.as_str(), definition))
        .collect();
    let definitions_by_name: HashMap<&str, &ModelDefinition> = definitions
        .iter()
        .map(|definition| (definition.name.as_str(), definition))
        .collect();
    let instances_by_name: HashMap<&str, &ModelInstance> = instances
        .iter()
        .map(|instance| (instance.binding_name.as_str(), instance))
        .collect();
    for instance in instances {
        let Some(definition) = definitions_by_id.get(instance.definition_identity.as_str()) else {
            diagnostics.push(binding_diag(
                "E_MODEL_INSTANCE_DEFINITION_CANDIDATE",
                format!(
                    "instance '{}'의 ModelDefinition identity가 없습니다",
                    instance.binding_name
                ),
                instance.provenance,
            ));
            continue;
        };
        for name in instance.initial_overrides.keys() {
            match definition
                .declarations
                .iter()
                .find(|declaration| &declaration.name == name)
            {
                Some(declaration) if declaration.accepts_initial_override => {}
                Some(declaration) => diagnostics.push(binding_diag(
                    "E_MODEL_INITIAL_OVERRIDE_UNAUTHORIZED_CANDIDATE",
                    format!("선언 '{}'에는 처음받음 권한이 없습니다", declaration.name),
                    instance.provenance,
                )),
                None => diagnostics.push(binding_diag(
                    "E_MODEL_INITIAL_OVERRIDE_TARGET_CANDIDATE",
                    format!("초기 대체 대상 '{name}' 선언이 없습니다"),
                    instance.provenance,
                )),
            }
        }
    }
    for request in requests {
        let target_span = request
            .pins
            .iter()
            .find(|pin| pin.name == "대상")
            .map_or(request.span, |pin| pin.span);
        let Some(target) = request.instance_binding.as_deref() else {
            diagnostics.push(binding_diag(
                "E_MODEL_RUN_TARGET_CANDIDATE",
                "풀기 요청의 대상은 explicit ModelInstance path여야 합니다",
                request.span,
            ));
            continue;
        };
        // 공개 instance 작성 문법은 아직 OPEN이다. raw source의 top-level
        // 모형값 이름은 request-scoped typed ModelInstance를 만들 수 있는
        // explicit identity evidence로 취급한다. 주입된 instance가 있으면 그
        // 결속을 우선 사용하고, 둘 다 없을 때만 fail-closed한다.
        let definition = if let Some(instance) = instances_by_name.get(target) {
            let Some(definition) = definitions_by_id.get(instance.definition_identity.as_str())
            else {
                continue;
            };
            *definition
        } else if let Some(definition) = definitions_by_name.get(target) {
            *definition
        } else {
            diagnostics.push(binding_diag(
                "E_MODEL_INSTANCE_PATH_CANDIDATE",
                format!("풀기 대상 model value 또는 instance path '{target}'를 찾을 수 없습니다"),
                target_span,
            ));
            continue;
        };
        if let Some(output_pin) = request.pins.iter().find(|pin| pin.name == "내줌") {
            for output in &request.output_selection {
                if let Some(declaration) = definition
                    .declarations
                    .iter()
                    .find(|declaration| &declaration.name == output)
                {
                    if !declaration.exported {
                        diagnostics.push(binding_diag(
                            "E_MODEL_OUTPUT_UNAVAILABLE_CANDIDATE",
                            format!("선언 '{}'에는 ::내줌 권한이 없습니다", declaration.name),
                            output_pin.span,
                        ));
                    }
                } else {
                    diagnostics.push(binding_diag(
                        "E_MODEL_OUTPUT_UNAVAILABLE_CANDIDATE",
                        format!("선택한 output '{output}' 선언이 없습니다"),
                        output_pin.span,
                    ));
                }
            }
        }
    }
}

/// backend descriptor가 승인된 뒤 그 capability allowlist에 대해 재사용할 수 있는
/// 순수 검사다. Wave A는 어떤 allowlist도 기본 선택하지 않는다.
pub fn validate_model_capabilities(
    ir: &ModelIr,
    supported: &BTreeSet<String>,
) -> Vec<ModelDiagnostic> {
    let mut diagnostics = Vec::new();
    for definition in &ir.definitions {
        for capability in definition.required_capabilities.difference(supported) {
            diagnostics.push(capability_diag(
                "E_MODEL_SOLVER_CAPABILITY_UNSUPPORTED_CANDIDATE",
                format!("선택한 backend가 capability '{capability}'를 지원하지 않습니다"),
                definition.span,
            ));
        }
    }
    diagnostics
}

pub fn validate_model_ir_bindings(ir: &ModelIr) -> Vec<ModelDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_instance_and_request_bindings(
        &ir.definitions,
        &ir.instances,
        &ir.requests,
        &mut diagnostics,
    );
    diagnostics
}

fn bind_derivative_axis(expr: &mut ModelExpr, axis: Option<&str>) {
    if let ModelExprKind::Derivative(identity) = &mut expr.kind {
        identity.axis = axis.map(str::to_string);
    }
    if let Some(node) = &mut expr.node {
        bind_node_derivative_axis(node, axis);
    }
}

fn bind_node_derivative_axis(node: &mut ModelExprNode, axis: Option<&str>) {
    match node {
        ModelExprNode::Derivative(identity) => identity.axis = axis.map(str::to_string),
        ModelExprNode::Unary { operand, .. } | ModelExprNode::Group(operand) => {
            bind_node_derivative_axis(operand, axis)
        }
        ModelExprNode::Binary { left, right, .. } => {
            bind_node_derivative_axis(left, axis);
            bind_node_derivative_axis(right, axis);
        }
        ModelExprNode::Number { .. }
        | ModelExprNode::String(_)
        | ModelExprNode::Atom(_)
        | ModelExprNode::Path(_) => {}
    }
}

fn relation_has_derivative(relation: &ModelRelation) -> bool {
    expr_contains_derivative(&relation.left) || expr_contains_derivative(&relation.right)
}

fn expr_contains_derivative(expr: &ModelExpr) -> bool {
    matches!(expr.kind, ModelExprKind::Derivative(_))
        || expr.node.as_ref().is_some_and(node_contains_derivative)
}

fn node_contains_derivative(node: &ModelExprNode) -> bool {
    match node {
        ModelExprNode::Derivative(_) => true,
        ModelExprNode::Unary { operand, .. } | ModelExprNode::Group(operand) => {
            node_contains_derivative(operand)
        }
        ModelExprNode::Binary { left, right, .. } => {
            node_contains_derivative(left) || node_contains_derivative(right)
        }
        _ => false,
    }
}

fn lower_relation_node(
    model_name: &str,
    relation: &ModelRelation,
    explicit_unknowns: &HashSet<String>,
) -> Option<LoweredRelationNode> {
    let (target, expression) = match (&relation.left.kind, &relation.right.kind) {
        (ModelExprKind::Derivative(identity), _) if !expr_contains_derivative(&relation.right) => (
            LoweredRelationTarget::Derivative(identity.clone()),
            relation.right.clone(),
        ),
        (_, ModelExprKind::Derivative(identity)) if !expr_contains_derivative(&relation.left) => (
            LoweredRelationTarget::Derivative(identity.clone()),
            relation.left.clone(),
        ),
        _ => {
            let left_target = direct_unknown_path(&relation.left, explicit_unknowns);
            let right_target = direct_unknown_path(&relation.right, explicit_unknowns);
            match (left_target, right_target) {
                (Some(target), None)
                    if !relation.right.references.contains(&target.to_string()) =>
                {
                    (
                        LoweredRelationTarget::Algebraic(target.to_string()),
                        relation.right.clone(),
                    )
                }
                (None, Some(target)) if !relation.left.references.contains(&target.to_string()) => {
                    (
                        LoweredRelationTarget::Algebraic(target.to_string()),
                        relation.left.clone(),
                    )
                }
                _ => return None,
            }
        }
    };
    let semantic = format!(
        "{}|{}|{}",
        model_name, relation.semantic_identity, relation.identity.stable_id
    );
    let dependencies = expression
        .references
        .iter()
        .filter(|reference| match &target {
            LoweredRelationTarget::Algebraic(target) => *reference != target,
            LoweredRelationTarget::Derivative(identity) => *reference != &identity.base,
        })
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Some(LoweredRelationNode {
        identity: format!("lowered-relation:{}", hash_prefixed(semantic.as_bytes())),
        authored_relation_identity: relation.identity.stable_id.clone(),
        target,
        expression,
        dependencies,
        provenance: relation.identity.span,
    })
}

fn direct_unknown_path<'a>(
    expr: &'a ModelExpr,
    explicit_unknowns: &HashSet<String>,
) -> Option<&'a str> {
    match &expr.kind {
        ModelExprKind::Path(path) if !path.contains('.') && explicit_unknowns.contains(path) => {
            Some(path)
        }
        _ => None,
    }
}

fn algebraic_target_diagnostic(
    relation: &ModelRelation,
    explicit_unknowns: &HashSet<String>,
) -> (&'static str, String) {
    let left = direct_unknown_path(&relation.left, explicit_unknowns);
    let right = direct_unknown_path(&relation.right, explicit_unknowns);
    match (left, right) {
        (Some(left), Some(right)) => (
            "E_MODEL_RELATION_TARGET_AMBIGUOUS_CANDIDATE",
            format!("관계 양쪽 '{left}', '{right}'이 모두 ::구함 target 조건을 만족합니다"),
        ),
        (Some(target), None) if relation.right.references.iter().any(|name| name == target) => (
            "E_MODEL_RELATION_TARGET_SELF_REFERENCE_CANDIDATE",
            format!("target '{target}'이 반대쪽 식에 다시 나타납니다"),
        ),
        (None, Some(target)) if relation.left.references.iter().any(|name| name == target) => (
            "E_MODEL_RELATION_TARGET_SELF_REFERENCE_CANDIDATE",
            format!("target '{target}'이 반대쪽 식에 다시 나타납니다"),
        ),
        _ if matches!(relation.left.kind, ModelExprKind::Path(_))
            || matches!(relation.right.kind, ModelExprKind::Path(_)) =>
        {
            (
                "E_MODEL_UNKNOWN_NOT_EXPLICIT_CANDIDATE",
                "관계 양쪽 중 정확히 한 direct path를 '이름::구함'으로 명시해야 합니다".to_string(),
            )
        }
        _ => (
            "E_MODEL_RELATION_TARGET_SHAPE_CANDIDATE",
            "관계 양쪽 중 정확히 한쪽에 단일 ::구함 path가 필요합니다".to_string(),
        ),
    }
}

fn contains_unbound_derivative(expr: &ModelExpr) -> bool {
    matches!(
        &expr.kind,
        ModelExprKind::Derivative(DerivativeIdentity { axis: None, .. })
    ) || expr
        .node
        .as_ref()
        .is_some_and(node_contains_unbound_derivative)
}

fn node_contains_unbound_derivative(node: &ModelExprNode) -> bool {
    match node {
        ModelExprNode::Derivative(DerivativeIdentity { axis: None, .. }) => true,
        ModelExprNode::Unary { operand, .. } | ModelExprNode::Group(operand) => {
            node_contains_unbound_derivative(operand)
        }
        ModelExprNode::Binary { left, right, .. } => {
            node_contains_unbound_derivative(left) || node_contains_unbound_derivative(right)
        }
        _ => false,
    }
}

fn validate_relation_dimensions(
    relation: &ModelRelation,
    declarations: &HashMap<String, ModelDeclaration>,
    axis: Option<&str>,
    diagnostics: &mut Vec<ModelDiagnostic>,
) {
    let left = match expression_dimension(&relation.left, declarations, axis) {
        Ok(value) => value,
        Err(message) => {
            diagnostics.push(diag(
                "E_MODEL_UNIT_MISMATCH_CANDIDATE",
                message,
                relation.identity.span,
            ));
            return;
        }
    };
    let right = match expression_dimension(&relation.right, declarations, axis) {
        Ok(value) => value,
        Err(message) => {
            diagnostics.push(diag(
                "E_MODEL_UNIT_MISMATCH_CANDIDATE",
                message,
                relation.identity.span,
            ));
            return;
        }
    };
    if let (Some(left), Some(right)) = (left, right) {
        if left != right {
            diagnostics.push(diag(
                "E_MODEL_UNIT_MISMATCH_CANDIDATE",
                format!(
                    "관계 양쪽 단위 차원이 다릅니다: {} != {}",
                    left.format(),
                    right.format()
                ),
                relation.identity.span,
            ));
        }
    }
}

fn expression_dimension(
    expr: &ModelExpr,
    declarations: &HashMap<String, ModelDeclaration>,
    axis: Option<&str>,
) -> Result<Option<UnitDim>, String> {
    if let Some(node) = &expr.node {
        return node_dimension(node, declarations, axis);
    }
    match &expr.kind {
        ModelExprKind::Path(path) => declarations
            .get(path)
            .and_then(|declaration| declaration.model_type.dimension)
            .map(Some)
            .ok_or_else(|| format!("선언되지 않은 값 '{path}'의 단위를 추정하지 않습니다")),
        ModelExprKind::Derivative(identity) => {
            let base = declarations
                .get(&identity.base)
                .and_then(|declaration| declaration.model_type.dimension)
                .ok_or_else(|| {
                    format!("미분 밑값 '{}'의 단위를 찾을 수 없습니다", identity.base)
                })?;
            let Some(axis_name) = identity.axis.as_deref().or(axis) else {
                return Ok(None);
            };
            let axis_dim = declarations
                .get(axis_name)
                .and_then(|declaration| declaration.model_type.dimension)
                .ok_or_else(|| format!("미분 축 '{axis_name}'의 단위를 찾을 수 없습니다"))?;
            let mut result = base;
            for _ in 0..identity.order {
                result = result.sub(axis_dim);
            }
            Ok(Some(result))
        }
        ModelExprKind::Literal => Ok(expr.dimension),
        ModelExprKind::Compound => Ok(None),
    }
}

fn node_dimension(
    node: &ModelExprNode,
    declarations: &HashMap<String, ModelDeclaration>,
    axis: Option<&str>,
) -> Result<Option<UnitDim>, String> {
    match node {
        ModelExprNode::Number { dimension, .. } => Ok(Some(*dimension)),
        ModelExprNode::String(_) | ModelExprNode::Atom(_) => Ok(None),
        ModelExprNode::Path(path) => Ok(declarations
            .get(path)
            .and_then(|declaration| declaration.model_type.dimension)),
        ModelExprNode::Derivative(identity) => {
            let Some(base) = declarations
                .get(&identity.base)
                .and_then(|declaration| declaration.model_type.dimension)
            else {
                return Ok(None);
            };
            let Some(axis_name) = identity.axis.as_deref().or(axis) else {
                return Ok(None);
            };
            let Some(axis_dim) = declarations
                .get(axis_name)
                .and_then(|declaration| declaration.model_type.dimension)
            else {
                return Ok(None);
            };
            let mut result = base;
            for _ in 0..identity.order {
                result = result.sub(axis_dim);
            }
            Ok(Some(result))
        }
        ModelExprNode::Unary { operand, .. } | ModelExprNode::Group(operand) => {
            node_dimension(operand, declarations, axis)
        }
        ModelExprNode::Binary {
            operator,
            left,
            right,
        } => {
            let left_dimension = node_dimension(left, declarations, axis)?;
            let right_dimension = node_dimension(right, declarations, axis)?;
            match operator.as_str() {
                "+" | "-" => match (left_dimension, right_dimension) {
                    (Some(left), Some(right)) if left == right => Ok(Some(left)),
                    (Some(left), Some(right)) => Err(format!(
                        "산술식 '{operator}' 양쪽 단위 차원이 다릅니다: {} != {}",
                        left.format(),
                        right.format()
                    )),
                    _ => Ok(None),
                },
                "*" => Ok(match (left_dimension, right_dimension) {
                    (Some(left), Some(right)) => Some(left.add(right)),
                    _ => None,
                }),
                "/" => Ok(match (left_dimension, right_dimension) {
                    (Some(left), Some(right)) => Some(left.sub(right)),
                    _ => None,
                }),
                "^" => {
                    let Some(base) = left_dimension else {
                        return Ok(None);
                    };
                    let Some(exponent) = dimensionless_integer(right) else {
                        return Ok(None);
                    };
                    scale_dimension(base, exponent).map(Some)
                }
                "<" | ">" | "<=" | ">=" | "==" | "!=" => {
                    if let (Some(left), Some(right)) = (left_dimension, right_dimension) {
                        if left != right {
                            return Err(format!(
                                "비교식 양쪽 단위 차원이 다릅니다: {} != {}",
                                left.format(),
                                right.format()
                            ));
                        }
                    }
                    Ok(Some(UnitDim::NONE))
                }
                _ => Ok(None),
            }
        }
    }
}

fn dimensionless_integer(node: &ModelExprNode) -> Option<i8> {
    let ModelExprNode::Number {
        surface,
        unit: None,
        ..
    } = node
    else {
        return None;
    };
    surface.parse::<i8>().ok()
}

fn scale_dimension(dimension: UnitDim, exponent: i8) -> Result<UnitDim, String> {
    let mut scaled = [0i8; UnitDim::AXIS_COUNT];
    for (index, value) in dimension.exponents().into_iter().enumerate() {
        scaled[index] = value
            .checked_mul(exponent)
            .ok_or_else(|| "단위 지수 범위를 벗어났습니다".to_string())?;
    }
    Ok(UnitDim::from_exponents(scaled))
}

fn detect_cycles(
    definition: &ModelDefinition,
    graph: &BTreeMap<String, Vec<String>>,
    diagnostics: &mut Vec<ModelDiagnostic>,
) {
    fn visit(
        node: &str,
        prefix: &str,
        graph: &BTreeMap<String, Vec<String>>,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visiting.contains(node) {
            return true;
        }
        if !visited.insert(node.to_string()) {
            return false;
        }
        visiting.insert(node.to_string());
        let key = format!("{prefix}.{node}");
        let cycle = graph.get(&key).is_some_and(|deps| {
            deps.iter()
                .any(|dependency| visit(dependency, prefix, graph, visiting, visited))
        });
        visiting.remove(node);
        cycle
    }
    let mut visited = HashSet::new();
    for declaration in &definition.declarations {
        if visit(
            &declaration.name,
            &definition.name,
            graph,
            &mut HashSet::new(),
            &mut visited,
        ) {
            diagnostics.push(diag(
                "E_MODEL_RELATION_CYCLE_CANDIDATE",
                format!(
                    "'{}'에서 정적 relation cycle이 발견되었습니다",
                    declaration.name
                ),
                declaration.identity.span,
            ));
        }
    }
}

fn diag(
    candidate_code: &'static str,
    message: impl Into<String>,
    span: ModelSpan,
) -> ModelDiagnostic {
    diagnostic(
        candidate_code,
        message,
        span,
        "model_structure_candidate",
        "model_ir_rejected_candidate",
    )
}

fn binding_diag(
    candidate_code: &'static str,
    message: impl Into<String>,
    span: ModelSpan,
) -> ModelDiagnostic {
    diagnostic(
        candidate_code,
        message,
        span,
        "model_binding_candidate",
        "run_request_rejected_candidate",
    )
}

fn capability_diag(
    candidate_code: &'static str,
    message: impl Into<String>,
    span: ModelSpan,
) -> ModelDiagnostic {
    diagnostic(
        candidate_code,
        message,
        span,
        "model_capability_candidate",
        "backend_selection_rejected_candidate",
    )
}

fn diagnostic(
    candidate_code: &'static str,
    message: impl Into<String>,
    provenance: ModelSpan,
    phase: &'static str,
    effect: &'static str,
) -> ModelDiagnostic {
    ModelDiagnostic {
        candidate_code,
        phase,
        severity: "error_candidate",
        effect,
        message: message.into(),
        provenance,
    }
}

fn parse_unit_dimension(surface: &str) -> Result<UnitDim, String> {
    if surface.is_empty() {
        return Ok(UnitDim::NONE);
    }
    let chars: Vec<char> = surface.chars().collect();
    let mut index = 0usize;
    let mut result = UnitDim::NONE;
    let mut divide = false;
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
        let symbol: String = chars[start..index].iter().collect();
        let Some(spec) = unit_spec_from_symbol(&symbol) else {
            return Err(format!("알 수 없는 단위: {symbol}"));
        };
        let mut exponent = 1u8;
        if index < chars.len() && chars[index] == '^' {
            index += 1;
            let exponent_start = index;
            while index < chars.len() && chars[index].is_ascii_digit() {
                index += 1;
            }
            exponent = chars[exponent_start..index]
                .iter()
                .collect::<String>()
                .parse::<u8>()
                .map_err(|_| format!("잘못된 단위 지수: {surface}"))?;
        }
        for _ in 0..exponent {
            result = if divide {
                result.sub(spec.dim)
            } else {
                result.add(spec.dim)
            };
        }
    }
    Ok(result)
}

fn compact_tokens(tokens: &[Token]) -> String {
    tokens.iter().map(|token| token.raw.as_str()).collect()
}

fn canonical_program(ir: &ModelIr) -> String {
    let mut fragments: Vec<(usize, String)> = Vec::new();
    for definition in &ir.definitions {
        let mut out = format!("{} = 모형 {{\n", definition.name);
        for item in &definition.authored_order {
            let line = match item {
                ModelBodyItem::Declaration(name) => definition
                    .declarations
                    .iter()
                    .find(|declaration| &declaration.name == name)
                    .map(canonical_declaration),
                ModelBodyItem::Unknown(name) => Some(format!("{name}::구함.")),
                ModelBodyItem::Relation(index) => {
                    definition.relations.get(*index).map(|relation| {
                        format!(
                            "{} =:= {}.",
                            relation.left.canonical, relation.right.canonical
                        )
                    })
                }
                ModelBodyItem::Event(index) => definition
                    .events
                    .get(*index)
                    .map(|event| format!("{}.", event.canonical)),
                ModelBodyItem::Connection(index) => definition
                    .connections
                    .get(*index)
                    .map(|connection| format!("{}.", connection.canonical)),
            };
            if let Some(line) = line {
                out.push_str("  ");
                out.push_str(&line);
                out.push('\n');
            }
        }
        out.push_str("}.\n");
        fragments.push((definition.span.start, out));
    }
    for binding in &ir.exact_prelude_bindings {
        fragments.push((
            binding.span.start,
            format!("{} <- {}.\n", binding.name, binding.value.canonical),
        ));
    }
    for recipe in &ir.recipes {
        let mut out = format!("{} <- (\n", recipe.name);
        for (index, field) in recipe.fields.iter().enumerate() {
            out.push_str(&format!("  {}: {}", field.name, field.value.canonical));
            if index + 1 != recipe.fields.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str(").\n");
        fragments.push((recipe.span.start, out));
    }
    if let Some(request) = ir
        .requests
        .iter()
        .find(|request| !request.method_provenance.is_authored())
    {
        if !ir
            .recipes
            .iter()
            .any(|recipe| recipe.name == FIXED_DEFAULT_PRECISE_RECIPE_NAME)
        {
            fragments.push((
                request.span.start,
                format!(
                    "{FIXED_DEFAULT_PRECISE_RECIPE_NAME} <- (\n  방식: #정밀\n).\n"
                ),
            ));
        }
    }
    for request in &ir.requests {
        let mut out = format!("{} <- (\n", request.binding_name);
        let mut pins = request
            .pins
            .iter()
            .filter(|pin| !matches!(pin.name.as_str(), "실행계약" | "방법"))
            .map(|pin| (pin.name.as_str(), pin.value.canonical.clone()))
            .collect::<Vec<_>>();
        let contract_insert_at = pins
            .iter()
            .rposition(|(name, _)| matches!(*name, "들임" | "처음"))
            .or_else(|| pins.iter().rposition(|(name, _)| *name == "대상"))
            .map_or(0, |index| index + 1);
        pins.insert(
            contract_insert_at,
            ("실행계약", request.execution_contract.canonical_compact()),
        );
        let method_value = request
            .pins
            .iter()
            .find(|pin| pin.name == "방법")
            .map_or_else(
                || FIXED_DEFAULT_PRECISE_RECIPE_NAME.to_string(),
                |pin| pin.value.canonical.clone(),
            );
        pins.insert(contract_insert_at + 1, ("방법", method_value));
        for (index, (name, value)) in pins.iter().enumerate() {
            if *name == "실행계약" {
                out.push_str("  실행계약=(\n");
                for (field_index, (field_name, field_value, _)) in request
                    .execution_contract
                    .canonical_fields()
                    .into_iter()
                    .enumerate()
                {
                    out.push_str(&format!("    {field_name}: #{field_value}"));
                    if field_index + 1 != 7 {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str("  )");
            } else {
                out.push_str(&format!("  {name}={value}"));
            }
            if index + 1 != pins.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str(") 풀기.\n");
        fragments.push((request.span.start, out));
    }
    for view in &ir.views {
        let mut out = "(\n".to_string();
        for (index, field) in view.fields.iter().enumerate() {
            out.push_str(&format!("  {}: {}", field.name, field.value.canonical));
            if index + 1 != view.fields.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str(") 보임.\n");
        fragments.push((view.span.start, out));
    }
    fragments.sort_by_key(|(start, _)| *start);
    fragments
        .into_iter()
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn semantic_canon(ir: &ModelIr) -> String {
    let mut definitions: Vec<String> = ir
        .definitions
        .iter()
        .map(|definition| {
            let mut declarations: Vec<String> = definition
                .declarations
                .iter()
                .map(canonical_declaration)
                .collect();
            declarations.sort();
            let mut relations: Vec<String> = definition
                .relations
                .iter()
                .map(|relation| directionless_relation_canon(&relation.left, &relation.right))
                .collect();
            relations.sort();
            let mut events: Vec<String> = definition
                .events
                .iter()
                .map(|event| event.semantic_identity.clone())
                .collect();
            events.sort();
            let mut connections: Vec<String> = definition
                .connections
                .iter()
                .map(|connection| connection.semantic_identity.clone())
                .collect();
            connections.sort();
            format!(
                "model:{}|decl:{}|rel:{}|event:{}|connect:{}",
                definition.name,
                declarations.join(";"),
                relations.join(";"),
                events.join(";"),
                connections.join(";")
            )
        })
        .collect();
    definitions.sort();
    let mut exact_prelude_bindings = ir
        .exact_prelude_bindings
        .iter()
        .map(|binding| format!("exact-prelude:{}={}", binding.name, binding.value.canonical))
        .collect::<Vec<_>>();
    exact_prelude_bindings.sort();
    let mut recipes: Vec<String> = ir
        .recipes
        .iter()
        .map(|recipe| {
            let mut fields: Vec<String> = recipe
                .fields
                .iter()
                .map(|field| format!("{}:{}", field.name, field.value.canonical))
                .collect();
            fields.sort();
            format!("recipe:{}|{}", recipe.name, fields.join(";"))
        })
        .collect();
    if ir
        .requests
        .iter()
        .any(|request| !request.method_provenance.is_authored())
        && !ir
            .recipes
            .iter()
            .any(|recipe| recipe.name == FIXED_DEFAULT_PRECISE_RECIPE_NAME)
    {
        recipes.push(format!(
            "recipe:{FIXED_DEFAULT_PRECISE_RECIPE_NAME}|방식:#정밀"
        ));
    }
    recipes.sort();
    let mut requests: Vec<String> = ir
        .requests
        .iter()
        .map(|request| {
            let mut pins: Vec<String> = request
                .pins
                .iter()
                .filter(|pin| pin.name != "실행계약")
                .map(|pin| format!("{}={}", pin.name, pin.value.canonical))
                .collect();
            pins.push(format!(
                "실행계약={}",
                request.execution_contract.canonical_compact()
            ));
            if !request.method_provenance.is_authored() {
                pins.push(format!("방법={FIXED_DEFAULT_PRECISE_RECIPE_NAME}"));
            }
            pins.sort();
            format!("run:{}|{}", request.binding_name, pins.join(";"))
        })
        .collect();
    requests.sort();
    let mut views: Vec<String> = ir
        .views
        .iter()
        .map(|view| {
            view.fields
                .iter()
                .map(|field| format!("{}:{}", field.name, field.value.canonical))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect();
    views.sort();
    format!(
        "{}\n{}\n{}\n{}\n{}",
        definitions.join("\n"),
        exact_prelude_bindings.join("\n"),
        recipes.join("\n"),
        requests.join("\n"),
        views.join("\n")
    )
}

fn validate_fixed_default_recipe_name(ir: &ModelIr) -> Result<(), ModelParseError> {
    let Some(request) = ir
        .requests
        .iter()
        .find(|request| !request.method_provenance.is_authored())
    else {
        return Ok(());
    };
    if let Some(definition) = ir
        .definitions
        .iter()
        .find(|definition| definition.name == FIXED_DEFAULT_PRECISE_RECIPE_NAME)
    {
        return Err(error(
            "E_MODEL_FIXED_DEFAULT_NAME_CONFLICT_CANDIDATE",
            "방법 생략 canon의 고정 이름 '정밀한방법'이 모형 이름과 충돌합니다",
            definition.span.start,
            definition.span.end,
        ));
    }
    if let Some(recipe) = ir
        .recipes
        .iter()
        .find(|recipe| recipe.name == FIXED_DEFAULT_PRECISE_RECIPE_NAME)
    {
        let is_precise = recipe.fields.len() == 1
            && recipe.fields[0].name == "방식"
            && recipe.method_value == Some(SolveMethodValue::Precise);
        if !is_precise {
            return Err(error(
                "E_MODEL_FIXED_DEFAULT_NAME_CONFLICT_CANDIDATE",
                "방법 생략 canon의 고정 이름 '정밀한방법'에는 #정밀 방식만 결속할 수 있습니다",
                recipe.span.start,
                recipe.span.end,
            ));
        }
    }
    // The request provenance is intentionally retained even when an authored,
    // semantically identical recipe already occupies the fixed canon name.
    let _ = request;
    Ok(())
}

fn directionless_relation_canon(left: &ModelExpr, right: &ModelExpr) -> String {
    let mut sides = [left.canonical.as_str(), right.canonical.as_str()];
    sides.sort_unstable_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    format!("{}=:={}", sides[0], sides[1])
}

fn event_semantic_surface(
    kind: &ModelEventKind,
    trigger: &ModelExpr,
    resets: &[ModelReset],
) -> String {
    let kind = match kind {
        ModelEventKind::Becomes => "becomes".to_string(),
        ModelEventKind::While => "while".to_string(),
        ModelEventKind::EveryMadi(n) => format!("every_madi:{n}"),
        ModelEventKind::CrossingBoth => "crossing_both".to_string(),
    };
    let mut reset_surfaces = resets
        .iter()
        .map(|reset| format!("{}<-{}", reset.target, reset.value.canonical))
        .collect::<Vec<_>>();
    reset_surfaces.sort();
    format!(
        "kind:{kind}|trigger:{}|resets:{}",
        trigger.canonical,
        reset_surfaces.join(";")
    )
}

fn connection_semantic_surface(endpoints: &[String], clauses: &[ConnectionClause]) -> String {
    let mut endpoints = endpoints.to_vec();
    endpoints.sort();
    let mut clauses = clauses
        .iter()
        .map(|clause| {
            let mode = match clause.mode {
                ConnectionMode::Equal => "equal",
                ConnectionMode::Flow => "flow",
                ConnectionMode::ReverseFlow => "reverse_flow",
            };
            format!("{}:{mode}", clause.quantity)
        })
        .collect::<Vec<_>>();
    clauses.sort();
    format!(
        "endpoints:{}|clauses:{}",
        endpoints.join(";"),
        clauses.join(";")
    )
}

fn canonical_declaration(declaration: &ModelDeclaration) -> String {
    let mut properties = vec![declaration.role.surface().to_string()];
    match &declaration.initialization {
        Initialization::None => {}
        Initialization::Fixed(value) => {
            properties.push(format!("처음::고정={}", value.canonical));
        }
        Initialization::Guess(value) => {
            properties.push(format!("처음::어림={}", value.canonical));
        }
        Initialization::Computed => properties.push("처음::계산".to_string()),
    }
    if declaration.accepts_initial_override {
        properties.push("처음받음".to_string());
    }
    if declaration.exported {
        properties.push("내줌".to_string());
    }
    let unit = declaration
        .model_type
        .unit
        .as_ref()
        .map(|unit| format!("@{unit}"))
        .unwrap_or_default();
    format!(
        "{}:{}{} :: ({}).",
        declaration.name,
        declaration.model_type.name,
        unit,
        properties.join(", ")
    )
}

fn canonical_tokens(tokens: &[Token]) -> String {
    let mut out = String::new();
    for (index, token) in tokens.iter().enumerate() {
        let tight_left = matches!(
            token.kind,
            TokenKind::Colon
                | TokenKind::DoubleColon
                | TokenKind::Dot
                | TokenKind::Comma
                | TokenKind::RParen
                | TokenKind::RBrace
                | TokenKind::RBracket
                | TokenKind::Prime
                | TokenKind::At
        );
        let previous_tight_right = index > 0
            && matches!(
                tokens[index - 1].kind,
                TokenKind::Colon
                    | TokenKind::DoubleColon
                    | TokenKind::Dot
                    | TokenKind::LParen
                    | TokenKind::LBrace
                    | TokenKind::LBracket
                    | TokenKind::At
                    | TokenKind::Prime
            );
        let operator = matches!(
            token.kind,
            TokenKind::Equal
                | TokenKind::RelationEqual
                | TokenKind::Assign
                | TokenKind::CrossingBoth
                | TokenKind::Operator(_)
        );
        let previous_operator = index > 0
            && matches!(
                tokens[index - 1].kind,
                TokenKind::Equal
                    | TokenKind::RelationEqual
                    | TokenKind::Assign
                    | TokenKind::CrossingBoth
                    | TokenKind::Operator(_)
            );
        if !out.is_empty()
            && !tight_left
            && !previous_tight_right
            && !out.ends_with(' ')
            && (operator || previous_operator || needs_word_space(&tokens[index - 1], token))
        {
            out.push(' ');
        }
        out.push_str(&token.raw);
        if matches!(token.kind, TokenKind::Comma) {
            out.push(' ');
        }
    }
    out.trim().to_string()
}

fn needs_word_space(left: &Token, right: &Token) -> bool {
    matches!(
        left.kind,
        TokenKind::Ident(_) | TokenKind::Number(_) | TokenKind::String(_) | TokenKind::Atom(_)
    ) && matches!(
        right.kind,
        TokenKind::Ident(_) | TokenKind::Number(_) | TokenKind::String(_) | TokenKind::Atom(_)
    )
}

fn collect_paths(tokens: &[Token]) -> Vec<String> {
    let mut paths = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let TokenKind::Ident(first) = &tokens[index].kind else {
            index += 1;
            continue;
        };
        let mut parts = vec![strip_josa(first)];
        let mut cursor = index + 1;
        while cursor + 1 < tokens.len()
            && matches!(tokens[cursor].kind, TokenKind::Dot)
            && matches!(tokens[cursor + 1].kind, TokenKind::Ident(_))
        {
            if let TokenKind::Ident(part) = &tokens[cursor + 1].kind {
                parts.push(strip_josa(part));
            }
            cursor += 2;
        }
        let path = parts.join(".");
        if !is_non_reference_word(&path) && !paths.contains(&path) {
            paths.push(path);
        }
        index = cursor;
    }
    paths
}

fn strip_josa(text: &str) -> String {
    for suffix in [
        "에게", "에서", "으로", "까지", "부터", "마다", "은", "는", "이", "가", "을", "를", "와",
        "과",
    ] {
        if let Some(stem) = text.strip_suffix(suffix) {
            if !stem.is_empty() {
                return stem.to_string();
            }
        }
    }
    text.to_string()
}

fn is_non_reference_word(word: &str) -> bool {
    matches!(
        word,
        "모형"
            | "수"
            | "축"
            | "이어감"
            | "띄엄"
            | "그자리"
            | "들임"
            | "처음"
            | "고정"
            | "어림"
            | "계산"
            | "처음받음"
            | "내줌"
            | "구함"
            | "될때"
            | "동안"
            | "같게"
            | "흐르게"
            | "거슬러"
            | "잇기"
            | "풀기"
    )
}

fn span_of(tokens: &[Token]) -> ModelSpan {
    match (tokens.first(), tokens.last()) {
        (Some(first), Some(last)) => first.span.merge(last.span),
        _ => ModelSpan { start: 0, end: 0 },
    }
}

fn token_ident(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(value) => Some(value),
        _ => None,
    }
}

fn split_top_level<F>(tokens: &[Token], mut is_delimiter: F) -> Vec<&[Token]>
where
    F: FnMut(&TokenKind) -> bool,
{
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket => depth += 1,
            TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket => depth -= 1,
            _ if depth == 0 && is_delimiter(&token.kind) => {
                out.push(&tokens[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    out.push(&tokens[start..]);
    out
}

fn find_top_level<F>(tokens: &[Token], mut predicate: F) -> Option<usize>
where
    F: FnMut(&TokenKind) -> bool,
{
    let mut depth = 0i32;
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket => depth += 1,
            TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket => depth -= 1,
            _ if depth == 0 && predicate(&token.kind) => return Some(index),
            _ => {}
        }
    }
    None
}

fn matching_close(
    tokens: &[Token],
    open: usize,
    open_kind: TokenKind,
    close_kind: TokenKind,
) -> Option<usize> {
    let open_discriminant = std::mem::discriminant(&open_kind);
    let close_discriminant = std::mem::discriminant(&close_kind);
    let mut depth = 0i32;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        if std::mem::discriminant(&token.kind) == open_discriminant {
            depth += 1;
        } else if std::mem::discriminant(&token.kind) == close_discriminant {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn outer_paren_contents(tokens: &[Token]) -> Option<&[Token]> {
    if tokens.len() < 2 || !matches!(tokens[0].kind, TokenKind::LParen) {
        return None;
    }
    let close = matching_close(tokens, 0, TokenKind::LParen, TokenKind::RParen)?;
    if close + 1 != tokens.len() {
        return None;
    }
    Some(&tokens[1..close])
}

fn trim_outer_parens(tokens: &[Token]) -> &[Token] {
    if tokens.len() >= 2 && matches!(tokens[0].kind, TokenKind::LParen) {
        if let Some(close) = matching_close(tokens, 0, TokenKind::LParen, TokenKind::RParen) {
            return &tokens[1..close];
        }
    }
    tokens
}

fn hash_prefixed(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

/// 빠른 surface 감지다. 문자열·주석·원자 안의 단어는 활성화하지 않는다.
/// 의미 판단은 반드시 `parse_model_program`이 한다.
pub fn has_model_surface(source: &str) -> bool {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    while lexer.pos < source.len() {
        lexer.skip_ws_and_comments();
        if lexer.pos >= source.len() {
            break;
        }
        match lexer.next_token() {
            Ok(token) => tokens.push(token),
            Err(_) => break,
        }
    }

    let has_definition = tokens.windows(4).any(|window| {
        matches!(window[0].kind, TokenKind::Ident(_))
            && matches!(window[1].kind, TokenKind::Equal)
            && matches!(&window[2].kind, TokenKind::Ident(name) if name == "모형")
            && matches!(window[3].kind, TokenKind::LBrace)
    });
    let has_solve = tokens
        .iter()
        .any(|token| matches!(&token.kind, TokenKind::Ident(name) if name == "풀기"));
    let has_target_pin = tokens.windows(2).any(|window| {
        matches!(&window[0].kind, TokenKind::Ident(name) if name == "대상")
            && matches!(window[1].kind, TokenKind::Equal)
    });
    let has_assignment = tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Assign));

    has_definition || has_assignment && has_solve && has_target_pin
}

pub fn parse_model_program(source: &str) -> Result<ModelProgram, ModelParseError> {
    parse_model_program_with_source_file(source, "<memory>")
}

pub fn parse_model_program_with_source_file(
    source: &str,
    source_file: &str,
) -> Result<ModelProgram, ModelParseError> {
    let tokens = Lexer::new(source).tokenize()?;
    Parser::new(source, source_file, tokens).parse()
}

pub fn fixed_default_precise_recipe(request: &RunRequest) -> SolveRecipe {
    let span = request.method_provenance.source_range();
    let expression_canon = "(방식:#정밀)";
    let recipe_source = format!(
        "{FIXED_DEFAULT_PRECISE_RECIPE_NAME}<-(방식:#정밀)."
    );
    SolveRecipe {
        identity: format!(
            "recipe:{}:{}",
            FIXED_DEFAULT_PRECISE_RECIPE_NAME,
            hash_prefixed(expression_canon.as_bytes())
        ),
        source_hash: hash_prefixed(recipe_source.as_bytes()),
        schema_version: MODEL_IR_SCHEMA_CANDIDATE,
        name: FIXED_DEFAULT_PRECISE_RECIPE_NAME.to_string(),
        fields: vec![RecordField {
            name: "방식".to_string(),
            value: ModelExpr {
                span,
                canonical: "#정밀".to_string(),
                kind: ModelExprKind::Literal,
                node: Some(ModelExprNode::Atom("정밀".to_string())),
                references: Vec::new(),
                dimension: None,
            },
            span,
        }],
        method_value: Some(SolveMethodValue::Precise),
        span,
    }
}

fn request_public_method_surface(ir: &ModelIr, request: &RunRequest) -> Option<String> {
    if !request.method_provenance.is_authored() {
        return Some("#정밀".to_string());
    }
    let binding = request.recipe_binding.as_deref()?;
    ir.recipes
        .iter()
        .find(|recipe| recipe.name == binding)
        .and_then(|recipe| recipe.method_value.as_ref())
        .map(|method| method.diagnostic_surface().to_string())
}

pub fn model_ir_candidate_json(program: &ModelProgram) -> JsonValue {
    json!({
        "schema": MODEL_IR_SCHEMA_CANDIDATE,
        "wire_status": "candidate_non_public",
        "source_hash": program.source_hash,
        "semantic_hash": program.ir.semantic_hash,
        "definitions": program.ir.definitions.iter().map(|definition| json!({
            "identity": definition.identity,
            "name": definition.name,
            "source_hash": definition.source_hash,
            "schema_version": definition.schema_version,
            "span": [definition.span.start, definition.span.end],
            "header_span": [definition.header_span.start, definition.header_span.end],
            "body_span": [definition.body_span.start, definition.body_span.end],
            "capabilities": definition.required_capabilities,
            "declarations": definition.declarations.iter().map(|declaration| json!({
                "identity": declaration.identity.stable_id,
                "occurrence": declaration.identity.occurrence,
                "name": declaration.name,
                "type": declaration.model_type.name,
                "unit": declaration.model_type.unit,
                "role": declaration.role.surface(),
                "initialization": match &declaration.initialization {
                    Initialization::None => "none",
                    Initialization::Fixed(_) => "fixed",
                    Initialization::Guess(_) => "guess",
                    Initialization::Computed => "computed",
                },
                "accepts_initial_override": declaration.accepts_initial_override,
                "exported": declaration.exported,
                "unknown": declaration.unknown,
                "span": [declaration.identity.span.start, declaration.identity.span.end],
            })).collect::<Vec<_>>(),
            "relations": definition.relations.iter().map(|relation| json!({
                "identity": relation.identity.stable_id,
                "semantic_identity": relation.semantic_identity,
                "left": relation.left.canonical,
                "right": relation.right.canonical,
                "lowered": relation.lowered.as_ref().map(|node| {
                    let (target_kind, target, axis, order) = match &node.target {
                        LoweredRelationTarget::Algebraic(target) => {
                            ("algebraic", target.as_str(), None, None)
                        }
                        LoweredRelationTarget::Derivative(identity) => (
                            "derivative",
                            identity.base.as_str(),
                            identity.axis.as_deref(),
                            Some(identity.order),
                        ),
                    };
                    json!({
                        "identity": node.identity,
                        "authored_relation_identity": node.authored_relation_identity,
                        "target_kind": target_kind,
                        "target": target,
                        "expression": node.expression.canonical,
                        "axis": axis,
                        "order": order,
                        "dependencies": node.dependencies,
                        "span": [node.provenance.start, node.provenance.end],
                    })
                }),
            })).collect::<Vec<_>>(),
            "events": definition.events.iter().map(|event| json!({
                "identity": event.identity.stable_id,
                "semantic_identity": event.semantic_identity,
                "kind": format!("{:?}", event.kind),
                "trigger": event.trigger.canonical,
                "resets": event.resets.iter().map(|reset| json!({
                    "identity": reset.identity,
                    "event_identity": reset.event_identity,
                    "target": reset.target,
                    "value": reset.value.canonical,
                    "provenance": [reset.provenance.start, reset.provenance.end],
                })).collect::<Vec<_>>(),
                "lowered": {
                    "identity": event.lowered.identity,
                    "authored_event_identity": event.lowered.authored_event_identity,
                    "semantic_identity": event.lowered.semantic_identity,
                    "detection": match event.lowered.detection {
                        EventDetectionPolicy::FalseToTrueTransition => "false_to_true_transition".to_string(),
                        EventDetectionPolicy::WhileGuard => "while_guard".to_string(),
                        EventDetectionPolicy::MadiInterval(n) => format!("madi_interval:{n}"),
                        EventDetectionPolicy::BidirectionalContinuousCrossing => "bidirectional_continuous_crossing".to_string(),
                    },
                    "remember_last_nonzero_sign": event.lowered.remember_last_nonzero_sign,
                    "simultaneous_grouping": event.lowered.simultaneous_grouping,
                    "reset_conflict_policy": event.lowered.reset_conflict_policy,
                    "reset_targets": event.lowered.reset_targets,
                    "required_capability": event.lowered.required_capability,
                    "provenance": [event.lowered.provenance.start, event.lowered.provenance.end],
                },
                "body_span": [event.body_span.start, event.body_span.end],
                "span": [event.identity.span.start, event.identity.span.end],
            })).collect::<Vec<_>>(),
            "connections": definition.connections.iter().map(|connection| json!({
                "identity": connection.identity.stable_id,
                "semantic_identity": connection.semantic_identity,
                "endpoints": connection.endpoints,
                "clauses": connection.clauses.iter().map(|clause| json!({
                    "quantity": clause.quantity,
                    "mode": format!("{:?}", clause.mode),
                    "provenance": [clause.provenance.start, clause.provenance.end],
                })).collect::<Vec<_>>(),
                "lowered_relations": connection.lowered_relations.iter().map(|relation| json!({
                    "identity": relation.identity,
                    "authored_connection_identity": relation.authored_connection_identity,
                    "semantic_identity": relation.semantic_identity,
                    "quantity": relation.quantity,
                    "mode": format!("{:?}", relation.mode),
                    "orientation_sign": relation.orientation_sign,
                    "endpoints": relation.endpoints,
                    "required_capability": relation.required_capability,
                    "provenance": [relation.provenance.start, relation.provenance.end],
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "model_values": program.ir.model_values.iter().map(|value| json!({
            "identity": value.identity,
            "definition_identity": value.definition_identity,
            "public_type": value.public_type,
            "span": [value.provenance.start, value.provenance.end],
        })).collect::<Vec<_>>(),
        "instances": program.ir.instances.iter().map(|instance| json!({
            "identity": instance.identity,
            "source_hash": instance.source_hash,
            "schema_version": instance.schema_version,
            "definition_identity": instance.definition_identity,
            "binding_name": instance.binding_name,
            "parameters": instance.parameters.iter().map(|(name, value)| {
                (name.clone(), value.canonical.clone())
            }).collect::<BTreeMap<_, _>>(),
            "initial_overrides": instance.initial_overrides.iter().map(|(name, value)| {
                (name.clone(), value.canonical.clone())
            }).collect::<BTreeMap<_, _>>(),
            "span": [instance.provenance.start, instance.provenance.end],
        })).collect::<Vec<_>>(),
        "exact_prelude_bindings": program.ir.exact_prelude_bindings.iter().map(|binding| json!({
            "identity": binding.identity,
            "name": binding.name,
            "value": binding.value.canonical,
            "source_hash": binding.source_hash,
            "span": [binding.span.start, binding.span.end],
        })).collect::<Vec<_>>(),
        "recipes": program.ir.recipes.iter().map(|recipe| json!({
            "identity": recipe.identity,
            "name": recipe.name,
            "source_hash": recipe.source_hash,
            "schema_version": recipe.schema_version,
            "fields": recipe.fields.iter().map(|field| json!({
                "name": field.name,
                "value": field.value.canonical,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "requests": program.ir.requests.iter().map(|request| json!({
            "identity": request.identity,
            "binding_name": request.binding_name,
            "source_hash": request.source_hash,
            "schema_version": request.schema_version,
            "instance_binding": request.instance_binding,
            "input_bindings": request.input_bindings.iter().map(|field| json!({
                "name": field.name,
                "value": field.value.canonical,
            })).collect::<Vec<_>>(),
            "initial_overrides": request.initial_overrides.iter().map(|field| json!({
                "name": field.name,
                "value": field.value.canonical,
            })).collect::<Vec<_>>(),
            "execution_contract": {
                "schema": request.execution_contract.schema_version,
                "wire_status": "candidate_non_public",
                "profile_identity": request.execution_contract.profile_identity,
                "profile_version": request.execution_contract.profile_version,
                "typed_input_component_hash": request.execution_contract.typed_input_component_hash(),
                "authored": request.execution_contract.provenance.is_authored(),
                "provenance_kind": request.execution_contract.provenance.surface_kind(),
                "source_file": request.execution_contract.provenance.source_file(),
                "source_range": [
                    request.execution_contract.provenance.source_range().start,
                    request.execution_contract.provenance.source_range().end,
                ],
                "canonical_field_order": request.execution_contract.canonical_fields()
                    .into_iter()
                    .map(|(name, _, _)| name)
                    .collect::<Vec<_>>(),
                "fields": request.execution_contract.canonical_fields()
                    .into_iter()
                    .map(|(name, value, value_span)| {
                        let field_span = request.execution_contract.field_span(name)
                            .expect("canonical execution-contract field span");
                        json!({
                            "name": name,
                            "value": format!("#{value}"),
                            "field_span": [field_span.start, field_span.end],
                            "value_span": [value_span.start, value_span.end],
                        })
                    })
                    .collect::<Vec<_>>(),
            },
            "recipe_binding": request.recipe_binding,
            "method_provenance": {
                "kind": request.method_provenance.surface_kind(),
                "authored": request.method_provenance.is_authored(),
                "source_file": request.method_provenance.source_file(),
                "source_range": [
                    request.method_provenance.source_range().start,
                    request.method_provenance.source_range().end,
                ],
                "public_method": request_public_method_surface(&program.ir, request),
            },
            "output_selection": request.output_selection,
            "end": request.end.as_ref().map(|value| value.canonical.clone()),
            "pins": request.pins.iter().map(|pin| json!({
                "name": pin.name,
                "value": pin.value.canonical,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "views": program.ir.views.iter().map(|view| json!({
            "identity": view.identity.stable_id,
            "span": [view.span.start, view.span.end],
            "fields": view.fields.iter().map(|field| json!({
                "name": field.name,
                "value": field.value.canonical,
                "span": [field.span.start, field.span.end],
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "dependency_graph": program.ir.dependency_graph,
        "diagnostics": program.ir.diagnostics.iter().map(|diagnostic| json!({
            "candidate_code": diagnostic.candidate_code,
            "phase": diagnostic.phase,
            "severity": diagnostic.severity,
            "effect": diagnostic.effect,
            "message": diagnostic.message,
            "provenance": [diagnostic.provenance.start, diagnostic.provenance.end],
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FALLING: &str = r#"
낙하운동 = 모형 {
  시간:수@s :: (축, 처음::고정=0@s).
  위치:수@m :: (이어감, 처음::고정=100@m, 처음받음, 내줌).
  속도:수@m/s :: (이어감, 처음::어림=0@m/s, 내줌).
  가속도:수@m/s^2 :: (그자리, 내줌).
  위치' =:= 속도.
  속도' =:= 가속도.
}.
"#;

    #[test]
    fn parses_typed_model_and_derivative_axis() {
        let program = parse_model_program(FALLING).expect("model parse");
        let definition = &program.ir.definitions[0];
        assert_eq!(definition.declarations.len(), 4);
        assert!(definition.header_span.end <= definition.body_span.start);
        assert!(
            FALLING[definition.header_span.start..definition.header_span.end].contains("모형 {")
        );
        assert!(
            FALLING[definition.body_span.start..definition.body_span.end].contains("위치:수@m")
        );
        let ModelExprKind::Derivative(identity) = &definition.relations[0].left.kind else {
            panic!("typed derivative expected");
        };
        assert_eq!(identity.base, "위치");
        assert_eq!(identity.axis.as_deref(), Some("시간"));
        assert_eq!(identity.order, 1);
        let lowered = definition.relations[0]
            .lowered
            .as_ref()
            .expect("lowered relation node");
        assert_ne!(
            definition.relations[0].identity.stable_id,
            definition.relations[0].semantic_identity
        );
        assert_ne!(lowered.identity, definition.relations[0].identity.stable_id);
        assert_ne!(lowered.identity, definition.relations[0].semantic_identity);
        assert_eq!(
            lowered.authored_relation_identity,
            definition.relations[0].identity.stable_id
        );
        assert!(matches!(
            &lowered.target,
            LoweredRelationTarget::Derivative(DerivativeIdentity {
                base,
                axis: Some(axis),
                order: 1,
            }) if base == "위치" && axis == "시간"
        ));
        assert!(!program.ir.diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.candidate_code,
                "E_MODEL_DERIVATIVE_AXIS_MISSING_CANDIDATE"
                    | "E_MODEL_UNIT_MISMATCH_CANDIDATE"
                    | "E_MODEL_UNKNOWN_NOT_EXPLICIT_CANDIDATE"
            )
        }));
    }

    #[test]
    fn rejects_unknown_property() {
        let err = parse_model_program("오류 = 모형 { 값:수 :: (알수없는성질). }.")
            .expect_err("unknown property");
        assert_eq!(err.candidate_code, "E_MODEL_PROPERTY_UNKNOWN_CANDIDATE");
    }

    #[test]
    fn rejects_duplicate_primary_role() {
        let err = parse_model_program("오류 = 모형 { 값:수 :: (이어감, 그자리). }.")
            .expect_err("role conflict");
        assert_eq!(err.candidate_code, "E_MODEL_PRIMARY_ROLE_COUNT_CANDIDATE");
    }

    #[test]
    fn rejects_missing_role_duplicate_property_and_initialization_conflict() {
        let missing =
            parse_model_program("오류 = 모형 { 값:수 :: (내줌). }.").expect_err("missing role");
        assert_eq!(
            missing.candidate_code,
            "E_MODEL_PRIMARY_ROLE_COUNT_CANDIDATE"
        );

        let duplicate = parse_model_program("오류 = 모형 { 값:수 :: (그자리, 내줌, 내줌). }.")
            .expect_err("duplicate property");
        assert_eq!(duplicate.candidate_code, "E_MODEL_PROPERTY_DUP_CANDIDATE");

        let conflict =
            parse_model_program("오류 = 모형 { 값:수 :: (그자리, 처음::고정=1, 처음::어림=2). }.")
                .expect_err("initialization conflict");
        assert_eq!(conflict.candidate_code, "E_MODEL_INIT_CONFLICT_CANDIDATE");
    }

    #[test]
    fn fixed_pins_and_open_fields_are_distinct() {
        let source = r#"
방법1 <- (방식: #정밀, 허용오차: 0.000001).
결과 <- (대상=공1, 들임=(힘: 0@N), 방법=방법1, 내줌=(위치, 속도), 끝=10@s) 풀기.
"#;
        let program = parse_model_program(source).expect("request parse");
        assert_eq!(program.ir.recipes.len(), 1);
        assert_eq!(program.ir.requests.len(), 1);
        assert_eq!(
            program.ir.requests[0].instance_binding.as_deref(),
            Some("공1")
        );
        assert_eq!(
            program.ir.requests[0].output_selection,
            vec!["위치".to_string(), "속도".to_string()]
        );
        assert_eq!(
            program.ir.requests[0].schema_version,
            MODEL_IR_SCHEMA_CANDIDATE
        );
        assert_eq!(
            program.ir.recipes[0].method_value,
            Some(SolveMethodValue::Precise)
        );
        assert!(program.canonical_ddn.contains("방식: #정밀"));
        assert!(program.canonical_ddn.contains("대상=공1"));
    }

    #[test]
    fn exact_prelude_bindings_are_not_misclassified_as_recipes() {
        let source = r#"
입력자료 <- 5.
배수자료 <- 입력자료 + 1.
계산=모형{입력:나눔수::(들임). 결과:나눔수::(그자리,내줌). 결과::구함. 결과 =:= 입력 * 2.}.
실행결과 <- (대상=계산, 들임=(입력: 배수자료), 내줌=(결과)) 풀기.
"#;
        let program = parse_model_program(source).expect("exact prelude parse");
        assert_eq!(program.ir.exact_prelude_bindings.len(), 2);
        assert!(program.ir.recipes.is_empty());
        assert_eq!(program.ir.exact_prelude_bindings[0].name, "입력자료");
        assert_eq!(program.ir.exact_prelude_bindings[1].value.canonical, "입력자료 + 1");
        assert!(program.canonical_ddn.contains("입력자료 <- 5."));
        assert_eq!(
            model_ir_candidate_json(&program)["exact_prelude_bindings"][1]["name"],
            "배수자료"
        );
    }

    #[test]
    fn omitted_method_keeps_authorship_provenance_and_canonicalizes_to_fixed_precise() {
        let omitted = r#"
계산=모형{입력:나눔수::(들임). 결과:나눔수::(그자리,내줌). 결과::구함. 결과 =:= 입력 + 1.}.
실행결과 <- (대상=계산, 들임=(입력: 2), 내줌=(결과)) 풀기.
"#;
        let explicit = r#"
계산=모형{입력:나눔수::(들임). 결과:나눔수::(그자리,내줌). 결과::구함. 결과 =:= 입력 + 1.}.
정밀한방법 <- (방식: #정밀).
실행결과 <- (대상=계산, 들임=(입력: 2), 방법=정밀한방법, 내줌=(결과)) 풀기.
"#;
        let omitted = parse_model_program_with_source_file(omitted, "entry.ddn")
            .expect("omitted method parse");
        let explicit = parse_model_program_with_source_file(explicit, "entry.ddn")
            .expect("explicit method parse");
        let provenance = &omitted.ir.requests[0].method_provenance;
        assert!(!provenance.is_authored());
        assert_eq!(provenance.source_file(), "entry.ddn");
        assert_eq!(provenance.source_range().start, provenance.source_range().end);
        assert_eq!(omitted.canonical_ddn, explicit.canonical_ddn);
        assert_eq!(omitted.ir.semantic_hash, explicit.ir.semantic_hash);
        assert_ne!(omitted.source_hash, explicit.source_hash);
        let candidate = model_ir_candidate_json(&omitted);
        assert_eq!(
            candidate["requests"][0]["method_provenance"]["kind"],
            "omitted_fixed_precise"
        );
        assert_eq!(
            candidate["requests"][0]["method_provenance"]["public_method"],
            "#정밀"
        );
    }

    #[test]
    fn omitted_method_rejects_conflicting_fixed_canon_name() {
        let source = r#"
계산=모형{입력:나눔수::(들임). 결과:나눔수::(그자리,내줌). 결과::구함. 결과 =:= 입력 + 1.}.
정밀한방법 <- (방식: #빠름).
실행결과 <- (대상=계산, 들임=(입력: 2), 내줌=(결과)) 풀기.
"#;
        let error = parse_model_program(source).expect_err("reserved canon conflict");
        assert_eq!(
            error.candidate_code,
            "E_MODEL_FIXED_DEFAULT_NAME_CONFLICT_CANDIDATE"
        );
    }

    #[test]
    fn solve_method_surface_is_typed_without_rewriting_historical_source() {
        let historical = format!(
            "빈모형 = 모형 {{}}. 방법 <- (방식: \"{EXACT_ACYCLIC_RELATION_BACKEND_ID}\")."
        );
        let legacy = parse_model_program(&historical).expect("historical recipe");
        assert_eq!(
            legacy.ir.recipes[0].method_value,
            Some(SolveMethodValue::HistoricalExactAcyclicRelationV1)
        );
        assert!(legacy
            .canonical_ddn
            .contains(EXACT_ACYCLIC_RELATION_BACKEND_ID));
        assert!(!legacy.canonical_ddn.contains("#정밀"));

        let unsupported = parse_model_program("빈모형 = 모형 {}. 방법 <- (방식: #빠름).")
            .expect("unsupported surface remains typed for runtime diagnosis");
        assert_eq!(
            unsupported.ir.recipes[0].method_value,
            Some(SolveMethodValue::Unsupported("#빠름".to_string()))
        );
    }

    #[test]
    fn rejects_colon_for_outer_run_pin() {
        let source = "결과 <- (대상: 공1, 끝=1@s) 풀기.";
        let err = parse_model_program(source).expect_err("colon must fail");
        assert_eq!(err.candidate_code, "E_MODEL_RUN_PIN_COLON_CANDIDATE");
    }

    #[test]
    fn whitespace_variants_have_same_semantic_hash() {
        let compact = "값모형=모형{값:수::(그자리,내줌).값::구함.}.";
        let spaced = "값모형 = 모형 {\n  값 : 수 :: (그자리, 내줌).\n  값 :: 구함.\n}.";
        let left = parse_model_program(compact).expect("compact");
        let right = parse_model_program(spaced).expect("spaced");
        assert_eq!(left.canonical_ddn, right.canonical_ddn);
        assert_ne!(left.source_hash, right.source_hash);
        assert_ne!(
            left.ir.definitions[0].source_hash,
            right.ir.definitions[0].source_hash
        );
        assert_eq!(left.ir.semantic_hash, right.ir.semantic_hash);
    }

    #[test]
    fn recipe_identity_and_source_hash_are_distinct() {
        let compact = "빈모형 = 모형 {}. 방법 <- (방식:#정밀,허용오차:0.1).";
        let spaced = "빈모형 = 모형 {}. 방법 <- (\n  방식: #정밀,\n  허용오차: 0.1\n).";
        let left = parse_model_program(compact).expect("compact recipe");
        let right = parse_model_program(spaced).expect("spaced recipe");
        assert_eq!(left.ir.recipes[0].identity, right.ir.recipes[0].identity);
        assert_ne!(
            left.ir.recipes[0].source_hash,
            right.ir.recipes[0].source_hash
        );
        assert_eq!(left.ir.semantic_hash, right.ir.semantic_hash);
    }

    #[test]
    fn declaration_source_order_does_not_change_semantic_hash() {
        let left = "값모형=모형{가:수::(그자리). 가::구함. 나:수::(들임). }.";
        let right = "값모형=모형{나:수::(들임). 가:수::(그자리). 가::구함. }.";
        let left = parse_model_program(left).expect("left");
        let right = parse_model_program(right).expect("right");
        assert_ne!(left.canonical_ddn, right.canonical_ddn);
        assert_eq!(left.ir.semantic_hash, right.ir.semantic_hash);
    }

    #[test]
    fn relation_source_order_does_not_change_semantic_hash_or_define_priority() {
        let left = r#"
계산 = 모형 {
  가:수 :: (그자리). 가::구함.
  나:수 :: (그자리). 나::구함.
  입력:수 :: (들임).
  가 =:= 입력 + 1.
  나 =:= 입력 + 2.
}.
"#;
        let right = r#"
계산 = 모형 {
  가:수 :: (그자리). 가::구함.
  나:수 :: (그자리). 나::구함.
  입력:수 :: (들임).
  나 =:= 입력 + 2.
  가 =:= 입력 + 1.
}.
"#;
        let left = parse_model_program(left).expect("left");
        let right = parse_model_program(right).expect("right");
        assert_ne!(left.canonical_ddn, right.canonical_ddn);
        assert_eq!(left.ir.semantic_hash, right.ir.semantic_hash);
    }

    #[test]
    fn declarations_are_precollected_across_free_body_placement() {
        let source = r#"
계산 = 모형 {
  결과 =:= 입력.
  결과::구함.
  (결과 > 0)이 될때 { 결과 <- 0. }.
  입력:수 :: (들임).
  결과:수 :: (그자리, 내줌).
}.
"#;
        let program = parse_model_program(source).expect("free placement");
        assert_eq!(program.ir.definitions[0].declarations.len(), 2);
        assert_eq!(program.ir.definitions[0].relations.len(), 1);
        assert_eq!(program.ir.definitions[0].events.len(), 1);
        assert!(!program.ir.diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.candidate_code,
                "E_MODEL_UNKNOWN_NOT_EXPLICIT_CANDIDATE"
                    | "E_MODEL_UNDERDETERMINED_CANDIDATE"
                    | "E_MODEL_OVERDETERMINED_CANDIDATE"
            )
        }));
    }

    #[test]
    fn one_explicit_unknown_and_one_producer_is_structurally_determined() {
        let source = r#"
계산 = 모형 {
  결과:수 :: (그자리, 내줌).
  입력:수 :: (들임).
  결과 =:= 입력.
  결과::구함.
}.
"#;
        let program = parse_model_program(source).expect("structural parse");
        assert!(!program.ir.diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.candidate_code,
                "E_MODEL_UNKNOWN_NOT_EXPLICIT_CANDIDATE"
                    | "E_MODEL_UNDERDETERMINED_CANDIDATE"
                    | "E_MODEL_OVERDETERMINED_CANDIDATE"
            )
        }));
    }

    #[test]
    fn relation_target_without_explicit_unknown_is_diagnosed() {
        let source = "계산 = 모형 { 결과:수 :: (그자리). 결과 =:= 1. }.";
        let program = parse_model_program(source).expect("structural parse");
        assert!(program.ir.diagnostics.iter().any(|diagnostic| {
            diagnostic.candidate_code == "E_MODEL_UNKNOWN_NOT_EXPLICIT_CANDIDATE"
        }));
    }

    #[test]
    fn explicit_unknown_without_producer_is_underdetermined() {
        let source = "계산 = 모형 { 결과:수 :: (그자리). 결과::구함. }.";
        let program = parse_model_program(source).expect("structural parse");
        assert!(program.ir.diagnostics.iter().any(|diagnostic| {
            diagnostic.candidate_code == "E_MODEL_UNDERDETERMINED_CANDIDATE"
        }));
    }

    #[test]
    fn multiple_producers_for_explicit_unknown_are_overdetermined() {
        let source = r#"
계산 = 모형 {
  결과:수 :: (그자리).
  결과::구함.
  결과 =:= 1.
  결과 =:= 2.
}.
"#;
        let program = parse_model_program(source).expect("structural parse");
        assert!(program
            .ir
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.candidate_code == "E_MODEL_MULTI_PRODUCER_CANDIDATE" }));
        assert!(program
            .ir
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.candidate_code == "E_MODEL_OVERDETERMINED_CANDIDATE" }));
    }

    #[test]
    fn undeclared_relation_reference_is_diagnosed() {
        let source = "계산 = 모형 { 없는값 =:= 1. }.";
        let program = parse_model_program(source).expect("structural parse");
        assert!(program.ir.diagnostics.iter().any(|diagnostic| {
            diagnostic.candidate_code == "E_MODEL_RELATION_REFERENCE_UNDECLARED_CANDIDATE"
        }));
    }

    #[test]
    fn relation_multi_producer_and_cycle_are_diagnosed() {
        let source = r#"
오류 = 모형 {
  가:수 :: (그자리).
  나:수 :: (그자리).
  가::구함.
  나::구함.
  가 =:= 나 + 1.
  가 =:= 1.
  나 =:= 가 + 1.
}.
"#;
        let program = parse_model_program(source).expect("structural parse");
        assert!(program
            .ir
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.candidate_code == "E_MODEL_MULTI_PRODUCER_CANDIDATE" }));
        assert!(
            program.ir.diagnostics.iter().any(|diagnostic| {
                diagnostic.candidate_code == "E_MODEL_RELATION_CYCLE_CANDIDATE"
            }),
            "graph={:?} diagnostics={:?}",
            program.ir.dependency_graph,
            program.ir.diagnostics
        );
    }

    #[test]
    fn directionless_relation_selects_exactly_one_unknown_on_either_side() {
        let left = parse_model_program(
            "계산=모형{입력:나눔수::(들임). 결과:나눔수::(그자리,내줌). 결과::구함. 결과 =:= 입력 + 1.}.",
        )
        .expect("left target");
        let right = parse_model_program(
            "계산=모형{입력:나눔수::(들임). 결과:나눔수::(그자리,내줌). 결과::구함. 입력 + 1 =:= 결과.}.",
        )
        .expect("right target");
        assert_eq!(left.ir.semantic_hash, right.ir.semantic_hash);
        for program in [&left, &right] {
            let lowered = program.ir.definitions[0].relations[0]
                .lowered
                .as_ref()
                .expect("lowered relation");
            assert!(matches!(
                &lowered.target,
                LoweredRelationTarget::Algebraic(target) if target == "결과"
            ));
            assert_eq!(lowered.expression.canonical, "입력 + 1");
        }
    }

    #[test]
    fn directionless_relation_rejects_ambiguous_and_self_referential_targets() {
        let ambiguous = parse_model_program(
            "계산=모형{가:나눔수::(그자리). 나:나눔수::(그자리). 가::구함. 나::구함. 가 =:= 나.}.",
        )
        .expect("ambiguous remains typed with diagnostic");
        assert!(ambiguous.ir.diagnostics.iter().any(|diagnostic| {
            diagnostic.candidate_code == "E_MODEL_RELATION_TARGET_AMBIGUOUS_CANDIDATE"
        }));

        let self_ref =
            parse_model_program("계산=모형{가:나눔수::(그자리). 가::구함. 가 =:= 가 + 1.}.")
                .expect("self reference remains typed with diagnostic");
        assert!(self_ref.ir.diagnostics.iter().any(|diagnostic| {
            diagnostic.candidate_code == "E_MODEL_RELATION_TARGET_SELF_REFERENCE_CANDIDATE"
        }));
    }

    #[test]
    fn run_request_and_result_table_keep_typed_ordered_fields() {
        let source = r#"
계산=모형{입력:나눔수::(들임,내줌). 결과:나눔수::(그자리,내줌). 결과::구함. 입력 + 1 =:= 결과.}.
정확한방법 <- (방식: "ddn.exact_acyclic_relation.v1").
실행결과 <- (대상=계산, 들임=(입력: 2), 처음=(), 방법=정확한방법, 내줌=(결과, 입력)) 풀기.
(계산값: 실행결과.결과, 입력값: 실행결과.입력) 보임.
"#;
        let program = parse_model_program(source).expect("typed run/view");
        let request = &program.ir.requests[0];
        assert_eq!(request.input_bindings[0].name, "입력");
        assert!(request.initial_overrides.is_empty());
        assert_eq!(request.recipe_binding.as_deref(), Some("정확한방법"));
        assert_eq!(request.output_selection, ["결과", "입력"]);
        assert_eq!(program.ir.views[0].fields[0].name, "계산값");
        assert!(program.canonical_ddn.contains(") 보임."));
    }

    #[test]
    fn derivative_unit_mismatch_is_diagnosed() {
        let source = r#"
오류 = 모형 {
  시간:수@s :: (축).
  위치:수@m :: (이어감).
  속도:수@m :: (들임).
  위치' =:= 속도.
}.
"#;
        let program = parse_model_program(source).expect("structural parse");
        assert!(program
            .ir
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.candidate_code == "E_MODEL_UNIT_MISMATCH_CANDIDATE" }));
    }

    #[test]
    fn compound_arithmetic_has_typed_tree_and_checked_units() {
        let valid = r#"
계산 = 모형 {
  결과:수@m :: (그자리). 결과::구함.
  거리:수@m :: (들임).
  결과 =:= 2 * 거리 + 1@m.
}.
"#;
        let program = parse_model_program(valid).expect("typed arithmetic");
        assert!(matches!(
            program.ir.definitions[0].relations[0].right.node,
            Some(ModelExprNode::Binary { .. })
        ));
        assert!(program.canonical_ddn.contains("2 * 거리 + 1@m"));
        assert!(!program
            .ir
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.candidate_code == "E_MODEL_UNIT_MISMATCH_CANDIDATE" }));

        let invalid = r#"
계산 = 모형 {
  결과:수@m :: (그자리). 결과::구함.
  거리:수@m :: (들임).
  결과 =:= 거리 + 1@s.
}.
"#;
        let program = parse_model_program(invalid).expect("typed arithmetic mismatch");
        assert!(program
            .ir
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.candidate_code == "E_MODEL_UNIT_MISMATCH_CANDIDATE" }));
    }

    #[test]
    fn all_approved_event_surfaces_have_typed_kinds() {
        let source = r#"
사건모형 = 모형 {
  값:수 :: (띄엄, 처음::고정=0).
  (값 > 0)이 될때 { 값 <- 1. }.
  (값 > 0)인 동안 { 값 <- 2. }.
  3마디마다 { 값 <- 3. }.
  (값) >< { 값 <- 4. }.
}.
"#;
        let program = parse_model_program(source).expect("event surfaces");
        let events = &program.ir.definitions[0].events;
        assert_eq!(events.len(), 4);
        assert!(matches!(events[0].kind, ModelEventKind::Becomes));
        assert!(matches!(events[1].kind, ModelEventKind::While));
        assert!(matches!(events[2].kind, ModelEventKind::EveryMadi(3)));
        assert!(matches!(events[3].kind, ModelEventKind::CrossingBoth));
        assert!(events.iter().all(|event| event.resets.len() == 1));
        assert_eq!(
            events[0].resets[0].event_identity,
            events[0].identity.stable_id
        );
        assert_eq!(events[0].trigger.canonical, "값 > 0");
        assert_eq!(events[1].trigger.canonical, "값 > 0");
        assert_eq!(events[2].trigger.canonical, "3");
        assert_eq!(events[3].trigger.canonical, "값");
        assert!(events.iter().all(|event| {
            event.lowered.simultaneous_grouping
                && event.lowered.reset_conflict_policy == "fail_closed"
                && event.lowered.required_capability == "event"
        }));
        assert!(!events[0].lowered.remember_last_nonzero_sign);
        assert!(events[3].lowered.remember_last_nonzero_sign);
        assert!(matches!(
            events[3].lowered.detection,
            EventDetectionPolicy::BidirectionalContinuousCrossing
        ));
        let candidate = model_ir_candidate_json(&program);
        assert_eq!(
            candidate["definitions"][0]["events"][3]["lowered"]["remember_last_nonzero_sign"],
            true
        );
        assert_eq!(
            candidate["definitions"][0]["events"][0]["lowered"]["reset_conflict_policy"],
            "fail_closed"
        );
    }

    #[test]
    fn event_source_order_does_not_define_semantic_priority() {
        let left = r#"
사건모형 = 모형 {
  값:수 :: (띄엄, 처음::고정=0).
  (값 > 0)이 될때 { 값 <- 1. }.
  3마디마다 { 값 <- 2. }.
}.
"#;
        let right = r#"
사건모형 = 모형 {
  값:수 :: (띄엄, 처음::고정=0).
  3마디마다 { 값 <- 2. }.
  (값 > 0)이 될때 { 값 <- 1. }.
}.
"#;
        let left = parse_model_program(left).expect("left event order");
        let right = parse_model_program(right).expect("right event order");
        assert_eq!(left.ir.semantic_hash, right.ir.semantic_hash);
        assert_ne!(
            left.ir.definitions[0].events[0].semantic_identity,
            right.ir.definitions[0].events[0].semantic_identity
        );
    }

    #[test]
    fn event_aliases_and_ambiguous_markers_are_rejected() {
        let missing_in = "사건 = 모형 { 값:수 :: (띄엄). (값 > 0) 동안 { 값 <- 1. }. }.";
        let err = parse_model_program(missing_in).expect_err("'인 동안' is exact");
        assert_eq!(err.candidate_code, "E_MODEL_BODY_ITEM_CANDIDATE");

        let arbitrary_mada = "사건 = 모형 { 값:수 :: (띄엄). 매일마다 { 값 <- 1. }. }.";
        let err = parse_model_program(arbitrary_mada).expect_err("N마디마다 is exact");
        assert_eq!(err.candidate_code, "E_MODEL_BODY_ITEM_CANDIDATE");

        let ambiguous = "사건 = 모형 { 값:수 :: (띄엄). (값 > 0)이 될때 >< { 값 <- 1. }. }.";
        let err = parse_model_program(ambiguous).expect_err("one event marker");
        assert_eq!(err.candidate_code, "E_MODEL_EVENT_KIND_CANDIDATE");

        let duplicate = "사건 = 모형 { 값:수 :: (띄엄). (값 > 0)이 될때 { 값 <- 1. 값 <- 2. }. }.";
        let err = parse_model_program(duplicate).expect_err("duplicate reset target");
        assert_eq!(err.candidate_code, "E_MODEL_EVENT_RESET_DUP_CANDIDATE");
    }

    #[test]
    fn undeclared_event_reset_target_is_diagnosed() {
        let source = "사건 = 모형 { 값:수 :: (띄엄). (값 > 0)이 될때 { 없는값 <- 1. }. }.";
        let program = parse_model_program(source).expect("typed reset");
        assert!(program.ir.diagnostics.iter().any(|diagnostic| {
            diagnostic.candidate_code == "E_MODEL_EVENT_RESET_TARGET_UNDECLARED_CANDIDATE"
        }));
    }

    #[test]
    fn canonical_output_is_lf_and_identifiers_are_nfc() {
        let decomposed = "값모형 = 모형 {\r\n  값:수 :: (그자리).\r\n  값::구함.\r\n}.";
        let program = parse_model_program(decomposed).expect("nfc model");
        assert!(!program.canonical_ddn.contains('\r'));
        assert!(program.canonical_ddn.contains("값모형"));
        assert!(program.canonical_ddn.contains("값:수"));
    }

    #[test]
    fn surface_probe_ignores_strings_comments_and_atoms() {
        assert!(!has_model_surface(
            "문구 <- \"모형 풀기 대상=공1\". // 이름 = 모형 { }\n종류 <- #모형."
        ));
        assert!(has_model_surface("낙하 = 모형 { 값:수 :: (그자리). }."));
        assert!(has_model_surface("결과 <- (대상=공1, 끝=1) 풀기."));
    }

    #[test]
    fn surface_probe_keeps_model_header_before_later_lex_error() {
        assert!(has_model_surface("낙하 = 모형 { 값:수 :: (그자리). ? }."));
    }

    #[test]
    fn third_prime_is_rejected_in_model_relation() {
        let source = "오류 = 모형 { 시간:수@s :: (축). 값:수 :: (이어감). 값''' =:= 값. }.";
        let err = parse_model_program(source).expect_err("third prime");
        assert_eq!(err.candidate_code, "E_MODEL_DERIVATIVE_ORDER_CANDIDATE");
    }

    #[test]
    fn unsupported_capability_is_reported_without_selecting_backend() {
        let program = parse_model_program(FALLING).expect("model parse");
        let diagnostics = validate_model_capabilities(&program.ir, &BTreeSet::new());
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.candidate_code == "E_MODEL_SOLVER_CAPABILITY_UNSUPPORTED_CANDIDATE"
            })
            .expect("capability diagnostic");
        assert_eq!(diagnostic.phase, "model_capability_candidate");
        assert_eq!(diagnostic.effect, "backend_selection_rejected_candidate");
    }

    #[test]
    fn connector_path_and_direction_fail_closed() {
        let bad_path = "회로 = 모형 { 왼쪽과 오른쪽을 (전압은 같게) 잇기. }.";
        let err = parse_model_program(bad_path).expect_err("ordinary path required");
        assert_eq!(err.candidate_code, "E_MODEL_INSTANCE_PATH_CANDIDATE");
        let bad_direction = "회로 = 모형 { 전지.양극과 전구.왼핀을 (전압은 거슬러 같게) 잇기. }.";
        let err = parse_model_program(bad_direction).expect_err("direction rejected");
        assert_eq!(err.candidate_code, "E_MODEL_CONNECTION_DIRECTION_CANDIDATE");
        let conflicting_mode =
            "회로 = 모형 { 전지.양극과 전구.왼핀을 (전압은 같게 흐르게) 잇기. }.";
        let err = parse_model_program(conflicting_mode).expect_err("one mode required");
        assert_eq!(err.candidate_code, "E_MODEL_CONNECTION_MODE_CANDIDATE");
    }

    #[test]
    fn n_ary_connection_preserves_paths_clauses_and_runtime_capability() {
        let source = "회로 = 모형 { 전지.양극과 전구.왼핀과 계기.입력을 (전압은 같게, 전류는 거슬러 흐르게) 잇기. }.";
        let program = parse_model_program(source).expect("n-ary connection");
        let definition = &program.ir.definitions[0];
        assert_eq!(definition.connections[0].endpoints.len(), 3);
        assert_eq!(definition.connections[0].clauses.len(), 2);
        assert!(matches!(
            definition.connections[0].clauses[1].mode,
            ConnectionMode::ReverseFlow
        ));
        assert!(definition
            .required_capabilities
            .contains("connection_runtime"));
        let lowered = &definition.connections[0].lowered_relations;
        assert_eq!(lowered.len(), 2);
        assert_eq!(lowered[0].orientation_sign, None);
        assert_eq!(lowered[1].orientation_sign, Some(-1));
        assert!(lowered
            .iter()
            .all(|relation| relation.required_capability == "connection_runtime"));
        assert_eq!(
            lowered[0].authored_connection_identity,
            definition.connections[0].identity.stable_id
        );
        let candidate = model_ir_candidate_json(&program);
        assert_eq!(
            candidate["definitions"][0]["connections"][0]["lowered_relations"][1]
                ["orientation_sign"],
            -1
        );
        assert!(
            candidate["definitions"][0]["connections"][0]["lowered_relations"][1]["provenance"]
                .is_array()
        );
    }

    #[test]
    fn connection_authored_order_is_preserved_but_not_semantic_priority() {
        let left = "회로 = 모형 { 전지.양극과 전구.왼핀과 계기.입력을 (전압은 같게, 전류는 흐르게) 잇기. }.";
        let right = "회로 = 모형 { 계기.입력과 전지.양극과 전구.왼핀을 (전류는 흐르게, 전압은 같게) 잇기. }.";
        let left = parse_model_program(left).expect("left connection order");
        let right = parse_model_program(right).expect("right connection order");
        let left_connection = &left.ir.definitions[0].connections[0];
        let right_connection = &right.ir.definitions[0].connections[0];
        assert_ne!(left_connection.endpoints, right_connection.endpoints);
        assert_eq!(
            left_connection.semantic_identity,
            right_connection.semantic_identity
        );
        assert_eq!(left.ir.semantic_hash, right.ir.semantic_hash);
        assert_eq!(
            left_connection.lowered_relations[0].endpoints,
            left_connection.endpoints
        );
    }

    #[test]
    fn instance_override_and_output_permissions_are_validated() {
        let source = r#"
값모형 = 모형 {
  값:수 :: (그자리).
  값::구함.
}.
결과 <- (대상=공1, 처음=(값: 1), 내줌=(값), 끝=1) 풀기.
"#;
        let mut program = parse_model_program(source).expect("model and request");
        let definition = &program.ir.definitions[0];
        let value = ModelExpr {
            span: definition.span,
            canonical: "1".to_string(),
            kind: ModelExprKind::Literal,
            node: Some(ModelExprNode::Number {
                surface: "1".to_string(),
                unit: None,
                dimension: UnitDim::NONE,
            }),
            references: Vec::new(),
            dimension: Some(UnitDim::NONE),
        };
        program.ir.instances.push(ModelInstance::candidate(
            definition.identity.clone(),
            "공1",
            BTreeMap::new(),
            BTreeMap::from([("값".to_string(), value)]),
            definition.span,
        ));
        let diagnostics = validate_model_ir_bindings(&program.ir);
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.candidate_code == "E_MODEL_INITIAL_OVERRIDE_UNAUTHORIZED_CANDIDATE"
            })
            .expect("binding diagnostic");
        assert_eq!(diagnostic.phase, "model_binding_candidate");
        assert_eq!(diagnostic.effect, "run_request_rejected_candidate");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.candidate_code == "E_MODEL_OUTPUT_UNAVAILABLE_CANDIDATE"
        }));
    }

    fn execution_contract_source(pin: Option<&str>) -> String {
        let pin = pin.map_or(String::new(), |pin| format!("{pin}, "));
        format!(
            "계산=모형{{입력:나눔수::(들임). 결과:나눔수::(그자리,내줌). 결과::구함. 결과 =:= 입력 + 1.}}.\n실행결과 <- (대상=계산, 들임=(입력: 2), {pin}내줌=(결과)) 풀기.\n"
        )
    }

    const STANDARD_EXECUTION_CONTRACT: &str = "실행계약=(결과: #정본값동일, 재현범위: #이식가능, 성능: #안전최적화, 외부유입: #봉인입력, 증거: #최소증거, 재생: #검증재생, 자원: #표준한도_v1)";

    #[test]
    fn omitted_and_explicit_standard_execution_contract_converge() {
        let omitted = parse_model_program(&execution_contract_source(None))
            .expect("omitted standard execution contract");
        let reversed = "실행계약=(자원: #표준한도_v1, 재생: #검증재생, 증거: #최소증거, 외부유입: #봉인입력, 성능: #안전최적화, 재현범위: #이식가능, 결과: #정본값동일)";
        let explicit = parse_model_program(&execution_contract_source(Some(reversed)))
            .expect("explicit standard execution contract");

        assert_eq!(omitted.canonical_ddn, explicit.canonical_ddn);
        assert_eq!(omitted.ir.semantic_hash, explicit.ir.semantic_hash);
        let omitted_contract = &omitted.ir.requests[0].execution_contract;
        let explicit_contract = &explicit.ir.requests[0].execution_contract;
        assert!(!omitted_contract.provenance.is_authored());
        assert!(explicit_contract.provenance.is_authored());
        assert_eq!(
            omitted_contract.profile_identity,
            STANDARD_REPRO_PROFILE_IDENTITY
        );
        assert_eq!(omitted_contract.profile_version, 1);
        assert_eq!(
            omitted_contract.typed_input_component_hash(),
            explicit_contract.typed_input_component_hash()
        );

        let canon = &omitted.canonical_ddn;
        let input = canon.find("  들임=").expect("input pin");
        let contract = canon.find("  실행계약=(").expect("contract pin");
        let method = canon.find("  방법=").expect("method pin");
        assert!(input < contract && contract < method);
        let ordered_fields = [
            "    결과: #정본값동일",
            "    성능: #안전최적화",
            "    외부유입: #봉인입력",
            "    자원: #표준한도_v1",
            "    재생: #검증재생",
            "    재현범위: #이식가능",
            "    증거: #최소증거",
        ];
        let mut previous = 0;
        for field in ordered_fields {
            let index = canon
                .find(field)
                .expect("canonical execution-contract field");
            assert!(index >= previous, "field order: {field}");
            previous = index;
        }
    }

    #[test]
    fn execution_contract_stays_after_initial_and_before_method() {
        let source = execution_contract_source(Some(STANDARD_EXECUTION_CONTRACT)).replace(
            "들임=(입력: 2), 실행계약=",
            "방법=정밀한방법, 처음=(결과: 0), 들임=(입력: 2), 실행계약=",
        );
        let program = parse_model_program(&source).expect("contract pin order canon");
        let canon = &program.canonical_ddn;
        let initial = canon.find("  처음=").expect("initial pin");
        let input = canon.find("  들임=").expect("input pin");
        let contract = canon.find("  실행계약=(").expect("contract pin");
        let method = canon.find("  방법=").expect("method pin");
        assert!(input < contract && initial < contract && contract < method);
    }

    #[test]
    fn execution_contract_exact_surface_is_fail_closed() {
        let cases = [
            (
                "실행계약: (결과: #정본값동일)",
                "E_MODEL_RUN_PIN_COLON_CANDIDATE",
            ),
            (
                "실행계약=(결과=#정본값동일, 재현범위: #이식가능, 성능: #안전최적화, 외부유입: #봉인입력, 증거: #최소증거, 재생: #검증재생, 자원: #표준한도_v1)",
                "E_MODEL_EXECUTION_CONTRACT_FIELD_DELIMITER_CANDIDATE",
            ),
            (
                "실행계약=(결과: #정본값동일, 결과: #정본값동일, 재현범위: #이식가능, 성능: #안전최적화, 외부유입: #봉인입력, 증거: #최소증거, 재생: #검증재생, 자원: #표준한도_v1)",
                "E_MODEL_EXECUTION_CONTRACT_FIELD_DUPLICATE_CANDIDATE",
            ),
            (
                "실행계약=(결과: #정본값동일, 재현범위: #이식가능, 성능: #안전최적화, 외부유입: #봉인입력, 증거: #최소증거, 재생: #검증재생, 자원: #표준한도_v1, 미지: #미지)",
                "E_MODEL_EXECUTION_CONTRACT_FIELD_UNKNOWN_CANDIDATE",
            ),
            (
                "실행계약=(결과: #정본값동일)",
                "E_MODEL_EXECUTION_CONTRACT_FIELD_MISSING_CANDIDATE",
            ),
            (
                "실행계약=(결과: #미지, 재현범위: #이식가능, 성능: #안전최적화, 외부유입: #봉인입력, 증거: #최소증거, 재생: #검증재생, 자원: #표준한도_v1)",
                "E_MODEL_EXECUTION_CONTRACT_VALUE_UNKNOWN_CANDIDATE",
            ),
            (
                "실행계약=(결과: #이식가능, 재현범위: #이식가능, 성능: #안전최적화, 외부유입: #봉인입력, 증거: #최소증거, 재생: #검증재생, 자원: #표준한도_v1)",
                "E_MODEL_EXECUTION_CONTRACT_COMBINATION_UNSUPPORTED_CANDIDATE",
            ),
            (
                "실행계약=#표준재현_v1",
                "E_MODEL_EXECUTION_CONTRACT_VALUE_UNKNOWN_CANDIDATE",
            ),
        ];
        for (pin, expected_code) in cases {
            let error = parse_model_program(&execution_contract_source(Some(pin)))
                .expect_err("invalid execution contract must fail");
            assert_eq!(error.candidate_code, expected_code, "pin: {pin}");
            assert!(error.span.start <= error.span.end, "pin: {pin}");
        }
    }

    #[test]
    fn execution_contract_typed_ir_preserves_identity_order_and_spans() {
        let source = execution_contract_source(Some(STANDARD_EXECUTION_CONTRACT));
        let program = parse_model_program_with_source_file(&source, "계약.ddn")
            .expect("typed execution contract");
        let candidate = model_ir_candidate_json(&program);
        let contract = &candidate["requests"][0]["execution_contract"];
        assert_eq!(contract["schema"], EXECUTION_CONTRACT_SCHEMA_CANDIDATE);
        assert_eq!(
            contract["profile_identity"],
            STANDARD_REPRO_PROFILE_IDENTITY
        );
        assert_eq!(contract["profile_version"], 1);
        assert_eq!(contract["authored"], true);
        assert_eq!(contract["source_file"], "계약.ddn");
        assert!(contract["typed_input_component_hash"]
            .as_str()
            .is_some_and(|value| value.starts_with("blake3:")));
        assert_eq!(
            contract["canonical_field_order"],
            json!([
                "결과",
                "성능",
                "외부유입",
                "자원",
                "재생",
                "재현범위",
                "증거"
            ])
        );
        let fields = contract["fields"].as_array().expect("typed fields");
        assert_eq!(fields.len(), 7);
        assert!(fields.iter().all(|field| {
            field["field_span"][0].as_u64().is_some() && field["value_span"][1].as_u64().is_some()
        }));
    }
}
