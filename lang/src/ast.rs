// lang/src/ast.rs
use ddonirang_core::Fixed64;
use std::collections::HashMap;

use crate::delivery::MessageTransfer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

pub type NodeId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceEpoch {
    PreV25,
    V25,
}

#[derive(Debug, Clone)]
pub struct CanonProgram {
    pub id: NodeId,
    pub source_epoch: SourceEpoch,
    pub items: Vec<TopLevelItem>,
    /// Imperative source-program statements. They deliberately remain outside
    /// callable definitions, so root `<-` never manufactures a callable or a
    /// DefinitionIdentity.
    pub root_stmts: Vec<Stmt>,
    pub origin: OriginMap,
}

/// DDL Wave 1 is a non-executable public-surface document.  It deliberately
/// stays separate from `CanonProgram`: a DDL projection has no runtime body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlDocument {
    pub extension: DdlExtension,
    pub source_span: Span,
    pub sections: Vec<LanguageSection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdlExtension {
    Ddl,
    Olgim,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageSection {
    pub language_profile_identity: String,
    pub header_span: Span,
    pub content_span: Span,
    pub projection: ImmutablePublicValueProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmutablePublicValueProjection {
    /// Stable registry-owned identity; localized spelling is never identity.
    pub definition_identity: String,
    pub type_identity: String,
    pub localized_name: String,
    pub localized_type: String,
    pub declaration_span: Span,
    pub omitted_payload: OmittedPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmittedPayload {
    ValueInitializer,
}

#[derive(Debug, Clone)]
pub struct OriginMap {
    pub file_path: String,
    pub source: String,
    pub node_spans: HashMap<NodeId, Span>,
    /// Compiler-local cache rebuilt from verified registration rows. NodeId
    /// locates a binder; it never issues the registered ParamPinIdentity.
    pub registered_sum_pin_order:Option<std::collections::BTreeMap<NodeId,[u8;32]>>,
    /// Exact definition candidates admitted by the authored registered
    /// surface. This is private compilation provenance, not a public identity.
    pub registered_call_candidates: HashMap<NodeId, Vec<NodeId>>,
    /// Compilation-local exact RD-656 conversion binding; not a saved wire.
    pub bytes_calls: HashMap<NodeId, crate::bytes::BytesConversion>,
    /// Source occurrences owned by actual atom-value AST nodes. A lexer hash
    /// token used as a tag/type/profile/selector is not automatically a value.
    /// Compilation provenance only; this is not registration admission.
    pub atom_value_occurrences: HashMap<NodeId, AtomValueOccurrence>,
    /// M02 accepted definition/pin binding. Command-only, with no result type.
    pub retirement_commands: HashMap<NodeId, RetirementCommandBinding>,
    /// R1.1 accepted postfix receiver, compilation-local only.
    pub subject_reference_calls: HashMap<NodeId, SubjectReferenceCallBinding>,
    /// Approved B1 definition/pin admission. Compilation provenance only;
    /// this is not an input event or a persisted authority.
    pub manipulation_phase_calls: HashMap<NodeId, ManipulationPhaseCallBinding>,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ManipulationPhaseCallBinding {pub definition:&'static str,pub ordinal_pin:&'static str}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SubjectReferenceCallBinding {pub definition:&'static str,pub receiver_pin:&'static str}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct RetirementCommandBinding {pub definition:&'static str,pub target_pin:&'static str}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomValueOccurrence {
    pub span: Span,
    pub label: String,
    pub role:AtomValueRole,
}
/// Compilation provenance only. Standard policy roles are supplied after
/// resolving the actual standard callable and its owned parameter position.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum AtomValueRole {LocalValue,StandardBogaeSide{selector:&'static str}}

#[derive(Debug, Clone)]
pub enum TopLevelItem {
    SeedDef(SeedDef),
}

#[derive(Debug, Clone)]
pub struct SeedDef {
    pub id: NodeId,
    pub span: Span,
    pub canonical_name: String,
    /// Explicit lexical alternates registered by `name{alternate~...}`.
    ///
    /// These spellings preserve this definition's identity. They are not
    /// separate definitions and are never inferred from natural-language
    /// knowledge.
    pub alternate_forms: Vec<String>,
    pub seed_kind: SeedKind,
    pub params: Vec<ParamPin>,
    /// The explicitly pinned result type for the RD-406 named, single-
    /// expression callable surface. `None` keeps the established seed forms
    /// unchanged.
    pub result_type: Option<TypeRef>,
    pub body: Option<Body>,
    pub modifiers: Vec<Modifier>,
}

impl SeedDef {
    /// Authored top-level setup relocated by the parser into the tick seed.
    /// Ordinary callable-local declarations retain their own scope.
    pub fn is_relocated_top_level_setup(&self, statement: &Stmt) -> bool {
        if !matches!(self.seed_kind, SeedKind::Umjikssi)
            || !matches!(self.canonical_name.as_str(), "매마디" | "매틱") {
            return false;
        }
        let Some(body) = &self.body else { return false; };
        let Stmt::DeclBlock { span, .. } = statement else { return false; };
        let synthetic = self.span == body.span
            && body.stmts.iter().all(|stmt| matches!(stmt, Stmt::DeclBlock { .. }));
        synthetic || span.start < self.span.start || span.end > self.span.end
    }

    /// Exact source spellings admitted for calls.
    ///
    /// A definition without an explicit set preserves the predecessor Korean
    /// call tails. Once an explicit set is authored, that set is closed:
    /// neither the preferred spelling nor an alternate gains a hidden tail.
    pub fn callable_surface_names(&self) -> Vec<String> {
        if matches!(self.seed_kind, SeedKind::Schema(_)) {
            return Vec::new();
        }
        const CALL_TAILS: [&str; 4] = ["기", "고", "면", "면서"];
        let mut surfaces = vec![self.canonical_name.clone()];
        if self.alternate_forms.is_empty() {
            surfaces.extend(CALL_TAILS.map(|tail| format!("{}{}", self.canonical_name, tail)));
        } else {
            surfaces.extend(self.alternate_forms.iter().cloned());
        }
        surfaces
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedKind {
    Imeumssi,
    Umjikssi,
    ValueFunc,
    Gallaessi,
    Relationssi,
    Sam,
    Heureumssi,
    Ieumssi,
    Semssi,
    /// RD-346/350: schema-only structural Product definition, not a callable.
    Product,
    /// VC04 exact R4: resolved declaration category, never inferred from a
    /// user-visible TypeRef spelling or from a callable body.
    Schema(SchemaCategory),
    Named(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaCategory {
    Subject,
    NominalValue,
}

impl SeedKind {
    pub fn is_schema(&self) -> bool {
        matches!(self, Self::Product | Self::Schema(_))
    }
}

#[derive(Debug, Clone)]
pub struct ParamPin {
    pub id: NodeId,
    pub span: Span,
    pub pin_name: String,
    pub type_ref: TypeRef,
    pub default_value: Option<Expr>,
    pub optional: bool,
    pub josa_list: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    /// Approved positional Tuple source type; existing type03 identity.
    Tuple(Vec<TypeRef>),
    /// RD-386·510: 씨 정체에는 원소씨와 정적인 순서 있는 모양이 들어간다.
    Tensor { element: Box<TypeRef>, shape: Vec<u64> },
    Named(String),
    Applied {
        name: String,
        args: Vec<TypeRef>,
    },
    Function {
        params: Vec<TypeRef>,
        result: Box<TypeRef>,
    },
    Infer,
}

#[derive(Debug, Clone)]
pub struct Body {
    pub id: NodeId,
    pub span: Span,
    pub stmts: Vec<Stmt>,
}

/// RD-347/348: a schema branch is not a field or a runtime statement.
#[derive(Debug, Clone)]
pub struct SumBranchDecl {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub payload: Vec<SumBranchPayload>,
}

#[derive(Debug, Clone)]
pub struct SumBranchPayload {
    pub binder: String,
    pub binder_span: Span,
    pub type_ref: TypeRef,
}

/// A pattern selects an owned pin, but creates a separate arm-local binder.
/// The node identity is not the branch's ParamPinIdentity or source ordinal.
#[derive(Debug, Clone)]
pub struct SumPatternBinder {
    pub id: NodeId,
    pub span: Span,
    pub pin: String,
    /// Checked compiler projection only; not a serialized source/value tag.
    pub registered_pin_identity:Option<[u8;32]>,
}

#[derive(Debug, Clone)]
pub struct SumPayloadArgument {
    pub pin: String,
    pub span: Span,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct SumMatchArm {
    pub id: NodeId,
    pub span: Span,
    pub owner: String,
    pub applied_owner: Option<TypeRef>,
    pub branch: String,
    pub binder: Vec<SumPatternBinder>,
    pub body: Body,
}

/// `(<이름>: <식>, ...) 보임.`이 선언한 공개 관찰 필드.
///
/// 순서는 곧 사용자 계약이므로 map으로 바꾸지 않는다. `name_span`은
/// 작성 도구가 순번 기반 ID 없이 DDN 이름 자체를 가리키는 데 사용한다.
#[derive(Debug, Clone)]
pub struct PublicObservationField {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub name_span: Span,
    pub value: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaBlockKind {
    Setting,
    Bogae,
    Seulgi,
    /// view-only 연출 블록 (Manim식 보개마당). state_hash 제외.
    /// entries[0] = { } 안의 원본 텍스트 그대로(opaque).
    BogeaMadang,
    /// AGE5 짜임 블록 최소 수용용 opaque 블록.
    /// entries[0] = { } 안의 원본 텍스트 그대로(opaque).
    Jjaim,
}

#[derive(Debug, Clone)]
pub struct ReactionRegistration {
    pub alert_type: String,
    pub payload: ParamPin,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// RD-294 one-way relation, not a direct assignment spelling. Runtime
    /// settles a graph of these declarations atomically with the world.
    DerivedBinding {
        id: NodeId,
        span: Span,
        mood: Mood,
        name: String,
        value: Expr,
    },
    SumDefinition {
        id: NodeId,
        span: Span,
        mood: Mood,
        name: String,
        branches: Vec<SumBranchDecl>,
    },
    SumMatch {
        id: NodeId,
        span: Span,
        mood: Mood,
        subject: Expr,
        arms: Vec<SumMatchArm>,
        default_body: Option<Body>,
    },
    DeclBlock {
        id: NodeId,
        span: Span,
        mood: Mood,
        kind: DeclKind,
        items: Vec<DeclItem>,
    },
    Definition {
        id: NodeId,
        span: Span,
        mood: Mood,
        name: String,
        type_ref: TypeRef,
        value: Expr,
        /// RD-445 owned named provider; never inferred from a thunk's body.
        reaction: Option<ReactionRegistration>,
    },
    Mutate {
        id: NodeId,
        span: Span,
        mood: Mood,
        target: Expr,
        value: Expr,
        deferred: bool,
        deferred_span: Option<Span>,
    },
    /// A produced value written to an already-addressed writable place.
    /// This is deliberately not a reverse spelling of `Mutate`: `producer ->
    /// destination` preserves the producer/result role through canon and runtime.
    ResultFlow {
        id: NodeId,
        span: Span,
        mood: Mood,
        producer: Expr,
        destination: Expr,
    },
    Expr {
        id: NodeId,
        span: Span,
        mood: Mood,
        expr: Expr,
    },
    Receive {
        id: NodeId,
        span: Span,
        mood: Mood,
        /// Exact W1-C1 sender identity predicate. None means ANY_SENDER.
        sender: Option<Expr>,
        kind: Option<String>,
        binding: Option<String>,
        /// Present only for the exact `(이름:형)를 받으면` surface. This is
        /// a payload type predicate, not a historical alert-kind alias.
        payload_type: Option<String>,
        condition: Option<Expr>,
        body: Body,
    },
    Send {
        id: NodeId,
        span: Span,
        mood: Mood,
        transfer: MessageTransfer,
        sender: Option<Expr>,
        payload: Expr,
        receiver: Expr,
    },
    Show {
        id: NodeId,
        span: Span,
        mood: Mood,
        expr: Expr,
    },
    Inspect {
        id: NodeId,
        span: Span,
        mood: Mood,
        expr: Expr,
    },
    PublicObservation {
        id: NodeId,
        span: Span,
        mood: Mood,
        fields: Vec<PublicObservationField>,
    },
    MetaBlock {
        id: NodeId,
        span: Span,
        mood: Mood,
        kind: MetaBlockKind,
        entries: Vec<String>,
    },
    Pragma {
        id: NodeId,
        span: Span,
        name: String,
        args: String,
    },
    Return {
        id: NodeId,
        span: Span,
        mood: Mood,
        value: Expr,
    },
    If {
        id: NodeId,
        span: Span,
        mood: Mood,
        condition: Expr,
        then_body: Body,
        else_body: Option<Body>,
    },
    Try {
        id: NodeId,
        span: Span,
        mood: Mood,
        action: Expr,
        body: Body,
    },
    Choose {
        id: NodeId,
        span: Span,
        mood: Mood,
        branches: Vec<ChooseBranch>,
        else_body: Body,
    },
    Repeat {
        id: NodeId,
        span: Span,
        mood: Mood,
        body: Body,
    },
    While {
        id: NodeId,
        span: Span,
        mood: Mood,
        condition: Expr,
        body: Body,
    },
    ForEach {
        id: NodeId,
        span: Span,
        mood: Mood,
        item: String,
        item_type: Option<TypeRef>,
        iterable: Expr,
        body: Body,
    },
    Quantifier {
        id: NodeId,
        span: Span,
        mood: Mood,
        kind: QuantifierKind,
        variable: String,
        domain: TypeRef,
        body: Body,
    },
    Break {
        id: NodeId,
        span: Span,
        mood: Mood,
    },
    ContinueLoop {
        id: NodeId,
        span: Span,
        mood: Mood,
    },
    Contract {
        id: NodeId,
        span: Span,
        mood: Mood,
        kind: ContractKind,
        mode: ContractMode,
        condition: Expr,
        then_body: Option<Body>,
        else_body: Body,
    },
    Guard {
        id: NodeId,
        span: Span,
        mood: Mood,
        condition: Expr,
        body: Body,
    },
    BeatBlock {
        id: NodeId,
        span: Span,
        mood: Mood,
        body: Body,
    },
    Hook {
        id: NodeId,
        span: Span,
        mood: Mood,
        kind: HookKind,
        body: Body,
    },
    HookWhenBecomes {
        id: NodeId,
        span: Span,
        mood: Mood,
        condition: Expr,
        body: Body,
    },
    HookWhile {
        id: NodeId,
        span: Span,
        mood: Mood,
        condition: Expr,
        body: Body,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclKind {
    Gureut,
    Butbak,
}

#[derive(Debug, Clone)]
pub struct DeclItem {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub kind: DeclKind,
    pub type_ref: TypeRef,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub span: Span,
    pub kind: ExprKind,
    /// Compilation-local source evidence, not a runtime value or saved type.
    /// In particular `1` and `1.0` must not be distinguished by Fixed64 bits.
    pub numeric_literal_source: Option<NumericLiteralSource>,
    pub(crate) source_unary_minus: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericLiteralSource {
    raw: String,
    decimal: bool,
}

impl NumericLiteralSource {
    /// Only the numeric token producer calls this constructor. An evaluated
    /// value must never be converted back into literal source evidence.
    pub(crate) fn from_token(raw: &str, decimal: bool) -> Self {
        Self { raw: raw.to_owned(), decimal }
    }

    pub fn raw(&self) -> &str { &self.raw }
    pub fn is_decimal(&self) -> bool { self.decimal }

    /// Owner-approved expectation-free literal policy. Expected rational
    /// admission and operator resolution are separate compilation steps.
    pub fn default_type(&self) -> TypeRef {
        TypeRef::Named(if self.decimal { "셈수" } else { "바른수" }.into())
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    /// RD-510: 원소씨와 모양은 실행 값이 아닌 정적 인자다.
    Tensor(TensorProviderExpr),
    /// RD-388: one authored bracket group; never flattened into A[i][j].
    MultiAxisSelection(MultiAxisSelection),
    Literal(Literal),
    Var(String),
    FieldAccess {
        target: Box<Expr>,
        field: String,
        /// `A.B`에서 마지막 `B`의 정확한 UTF-8 byte 범위.
        field_span: Span,
    },
    SeedLiteral {
        param: String,
        body: Box<Expr>,
    },
    Call {
        args: Vec<ArgBinding>,
        func: String,
    },
    /// RD-348/357: exact owned branch, not field access or indexed selection.
    SumBranchValue {
        owner: String,
        applied_owner: Option<TypeRef>,
        branch: String,
        payload: Vec<SumPayloadArgument>,
    },
    /// RD-601 reuses the named-pin shell but preserves construction as a
    /// distinct typed category. The schema owner is resolved before execution.
    ProductConstruction {
        owner: String,
        definition: NodeId,
        /// None is the existing structural Product. VC04's actual parsed
        /// schema category must survive canon; spelling is not authority.
        category: Option<SchemaCategory>,
        fields: Vec<ArgBinding>,
    },
    /// `차림[...]` is a source-level collection constructor, not a call or
    /// postfix selection.  Its elements retain their authored order.
    CollectionConstructor {
        elements: Vec<Expr>,
    },
    /// RD-090: positional pack, distinct from homogeneous collections and records.
    TupleConstructor {
        elements: Vec<Expr>,
    },
    /// An indexed writable place. This is only valid as `Stmt::Mutate`'s
    /// target and is deliberately distinct from postfix selection values.
    SelectionPlace {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Infix {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Suffix {
        value: Box<Expr>,
        at: AtSuffix,
    },
    Thunk(Body),
    Eval {
        thunk: Box<Expr>,
        mode: ThunkEvalMode,
    },
    Pipe {
        stages: Vec<Expr>,
    },
    FlowValue,
    Pack {
        fields: Vec<(String, Expr)>,
    },
    Assertion(Assertion),
    StateMachine(StateMachine),
    Formula(Formula),
    Template(Template),
    TemplateRender {
        template: Template,
        inject: Vec<(String, Expr)>,
    },
    FormulaEval {
        formula: Formula,
        inject: Vec<(String, Expr)>,
    },
    Nuance {
        level: String,
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct MultiAxisSelection {
    pub target: Box<Expr>,
    pub selectors: Vec<TensorAxisSelector>,
    /// SelectionPlace role, set only by assignment/result-flow parsing.
    pub writable: bool,
}

#[derive(Debug, Clone)]
pub enum TensorAxisSelector {
    Index(Box<Expr>),
    Range { start: u64, end: u64 },
    Full,
}

impl MultiAxisSelection {
    pub fn children(&self) -> impl Iterator<Item = &Expr> {
        std::iter::once(self.target.as_ref()).chain(self.selectors.iter().filter_map(|s| match s {
            TensorAxisSelector::Index(e) => Some(e.as_ref()), _ => None,
        }))
    }
    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut Expr> {
        std::iter::once(self.target.as_mut()).chain(self.selectors.iter_mut().filter_map(|s| match s {
            TensorAxisSelector::Index(e) => Some(e.as_mut()), _ => None,
        }))
    }
}

#[derive(Debug, Clone)]
pub enum TensorProviderExpr {
    FromCollection {
        input: Box<Expr>,
        element_type: TypeRef,
        shape: Vec<u64>,
    },
    ToCollection { input: Box<Expr> },
}

impl TensorProviderExpr {
    pub fn input(&self) -> &Expr {
        match self {
            Self::FromCollection { input, .. } | Self::ToCollection { input } => input,
        }
    }
    pub fn input_mut(&mut self) -> &mut Expr {
        match self {
            Self::FromCollection { input, .. } | Self::ToCollection { input } => input,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArgBinding {
    pub id: NodeId,
    pub span: Span,
    pub expr: Expr,
    pub josa: Option<String>,
    pub resolved_pin: Option<String>,
    pub binding_reason: BindingReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtSuffix {
    Unit(String),
    Asset(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingReason {
    Dictionary,
    Positional,
    UserFixed,
    FlowInjected,
    ContextItemInjected,
    Ambiguous { candidates: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThunkEvalMode {
    Value,
    Bool,
    Not,
    Do,
    Pipe,
}

#[derive(Debug, Clone)]
pub struct ChooseBranch {
    pub condition: Expr,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractKind {
    Pre,
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractMode {
    Abort,
    Alert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantifierKind {
    ForAll,
    Exists,
    ExistsUnique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookKind {
    Start,
    End,
    EveryMadi,
    EveryNMadi(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mood {
    Declarative,
    Imperative,
    Suggestive,
    Interrogative,
    Exclamative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Int(i64),
    Fixed64(Fixed64),
    String(String),
    Bytes(ddonirang_core::bytes::ByteSequence),
    Bool(bool),
    Atom(String),
    Regex(RegexLiteral),
    Resource(String),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexLiteral {
    pub pattern: String,
    pub flags: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub raw: String,
    pub parts: Vec<TemplatePart>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaDialect {
    Ascii,
    Ascii1,
    Latex,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formula {
    pub raw: String,
    pub dialect: FormulaDialect,
    pub explicit_tag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachine {
    pub states: Vec<String>,
    pub initial: String,
    pub transitions: Vec<StateTransition>,
    pub on_transition_checks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assertion {
    pub body_source: String,
    pub canon: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransition {
    pub from: String,
    pub to: String,
    pub guard_name: Option<String>,
    pub action_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplatePart {
    Text(String),
    Placeholder(TemplatePlaceholder),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePlaceholder {
    pub path: Vec<String>,
    pub format: Option<TemplateFormat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateFormat {
    pub raw: String,
    pub width: Option<usize>,
    pub zero_pad: bool,
    pub precision: Option<u8>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Modifier {
    pub id: NodeId,
    pub span: Span,
}

impl Expr {
    pub fn new(id: NodeId, span: Span, kind: ExprKind) -> Self {
        Self { id, span, kind, numeric_literal_source: None, source_unary_minus: false }
    }

    /// Parser-owned negation evidence; synthetic zero is not a source literal.
    pub fn source_negated_operand(&self) -> Option<&Expr> {
        if !self.source_unary_minus { return None; }
        match &self.kind {
            ExprKind::Infix { op, right, .. } if op == "-" => Some(right),
            _ => None,
        }
    }
}
