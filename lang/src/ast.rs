// lang/src/ast.rs
use ddonirang_core::Fixed64;
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct CanonProgram {
    pub id: NodeId,
    pub items: Vec<TopLevelItem>,
    pub origin: OriginMap,
}

#[derive(Debug, Clone)]
pub struct OriginMap {
    pub file_path: String,
    pub source: String,
    pub node_spans: HashMap<NodeId, Span>,
}

#[derive(Debug, Clone)]
pub enum TopLevelItem {
    SeedDef(SeedDef),
}

#[derive(Debug, Clone)]
pub struct SeedDef {
    pub id: NodeId,
    pub span: Span,
    pub canonical_name: String,
    pub seed_kind: SeedKind,
    pub params: Vec<ParamPin>,
    pub body: Option<Body>,
    pub modifiers: Vec<Modifier>,
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
    Named(String),
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

#[derive(Debug, Clone)]
pub enum TypeRef {
    Named(String),
    Applied { name: String, args: Vec<TypeRef> },
    Infer,
}

#[derive(Debug, Clone)]
pub struct Body {
    pub id: NodeId,
    pub span: Span,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaBlockKind {
    Setting,
    Bogae,
    Seulgi,
    /// view-only 연출 블록 (Manim식 보개마당). state_hash 제외.
    /// entries[0] = { } 안의 원본 텍스트 그대로(opaque).
    BogeaMadang,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    DeclBlock {
        id: NodeId,
        span: Span,
        mood: Mood,
        kind: DeclKind,
        items: Vec<DeclItem>,
    },
    Mutate {
        id: NodeId,
        span: Span,
        mood: Mood,
        target: Expr,
        value: Expr,
    },
    Expr {
        id: NodeId,
        span: Span,
        mood: Mood,
        expr: Expr,
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
    Break {
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
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Literal(Literal),
    Var(String),
    FieldAccess {
        target: Box<Expr>,
        field: String,
    },
    SeedLiteral {
        param: String,
        body: Box<Expr>,
    },
    Call {
        args: Vec<ArgBinding>,
        func: String,
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
        Self { id, span, kind }
    }
}
