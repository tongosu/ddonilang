use crate::core::unit::UnitExpr;
use crate::lang::span::Span;

#[derive(Clone, Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclKind {
    Gureut,
    Butbak,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeedKind {
    Semssi,
    Umjikssi,
    Balhigi,
    Named(String),
}

impl SeedKind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "셈씨" => Some(SeedKind::Semssi),
            "움직씨" => Some(SeedKind::Umjikssi),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuantifierKind {
    ForAll,
    Exists,
    ExistsUnique,
}

#[derive(Clone, Debug)]
pub struct DeclItem {
    pub name: String,
    pub kind: DeclKind,
    #[allow(dead_code)]
    pub type_name: String,
    pub value: Option<Expr>,
    pub maegim: Option<MaegimSpec>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct MaegimSpec {
    pub fields: Vec<Binding>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ParamPin {
    pub name: String,
    pub josa_list: Vec<String>,
    #[allow(dead_code)]
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ModuleImportItem {
    #[allow(dead_code)]
    pub alias: String,
    #[allow(dead_code)]
    pub path: String,
    #[allow(dead_code)]
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ModuleExportItem {
    #[allow(dead_code)]
    pub external_name: String,
    #[allow(dead_code)]
    pub internal_name: String,
    #[allow(dead_code)]
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    ImportBlock {
        #[allow(dead_code)]
        items: Vec<ModuleImportItem>,
        #[allow(dead_code)]
        span: Span,
    },
    ExportBlock {
        #[allow(dead_code)]
        items: Vec<ModuleExportItem>,
        #[allow(dead_code)]
        span: Span,
    },
    DeclBlock {
        items: Vec<DeclItem>,
        #[allow(dead_code)]
        span: Span,
    },
    SeedDef {
        name: String,
        params: Vec<ParamPin>,
        kind: SeedKind,
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    Assign {
        target: Path,
        value: Expr,
        deferred: bool,
        #[allow(dead_code)]
        deferred_span: Option<Span>,
        #[allow(dead_code)]
        span: Span,
    },
    Expr {
        value: Expr,
        #[allow(dead_code)]
        span: Span,
    },
    Receive {
        kind: Option<String>,
        binding: Option<String>,
        condition: Option<Expr>,
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    Send {
        sender: Option<Expr>,
        payload: Expr,
        receiver: Expr,
        #[allow(dead_code)]
        span: Span,
    },
    Return {
        value: Expr,
        #[allow(dead_code)]
        span: Span,
    },
    Show {
        value: Expr,
        #[allow(dead_code)]
        span: Span,
    },
    Inspect {
        value: Expr,
        #[allow(dead_code)]
        span: Span,
    },
    BogaeDraw {
        #[allow(dead_code)]
        span: Span,
    },
    Hook {
        kind: HookKind,
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    HookWhenBecomes {
        condition: Expr,
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    HookWhile {
        condition: Expr,
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    OpenBlock {
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    BeatBlock {
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    LifecycleBlock {
        name: Option<String>,
        kind: LifecycleUnitKind,
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
        #[allow(dead_code)]
        span: Span,
    },
    Choose {
        branches: Vec<ChooseBranch>,
        else_body: Option<Vec<Stmt>>,
        exhaustive: bool,
        #[allow(dead_code)]
        span: Span,
    },
    Repeat {
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    ForEach {
        item: String,
        iterable: Expr,
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    Quantifier {
        #[allow(dead_code)]
        kind: QuantifierKind,
        #[allow(dead_code)]
        variable: String,
        #[allow(dead_code)]
        domain: String,
        body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    Break {
        #[allow(dead_code)]
        span: Span,
    },
    Contract {
        kind: ContractKind,
        mode: ContractMode,
        condition: Expr,
        then_body: Option<Vec<Stmt>>,
        else_body: Vec<Stmt>,
        #[allow(dead_code)]
        span: Span,
    },
    Pragma {
        #[allow(dead_code)]
        name: String,
        #[allow(dead_code)]
        args: String,
        #[allow(dead_code)]
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub struct ChooseBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assertion {
    pub body_source: String,
    pub canon: String,
}

#[derive(Clone, Debug)]
pub struct Path {
    pub segments: Vec<String>,
    pub span: Span,
    pub implicit_root: bool,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Literal, Span),
    Path(Path),
    FieldAccess {
        target: Box<Expr>,
        field: String,
        span: Span,
    },
    Atom {
        text: String,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    SeedLiteral {
        param: String,
        body: Box<Expr>,
        span: Span,
    },
    Call {
        name: String,
        args: Vec<ArgBinding>,
        span: Span,
    },
    Formula {
        dialect: FormulaDialect,
        body: String,
        span: Span,
    },
    Assertion {
        assertion: Assertion,
        span: Span,
    },
    FormulaEval {
        dialect: FormulaDialect,
        body: String,
        bindings: Vec<Binding>,
        span: Span,
    },
    Template {
        body: String,
        span: Span,
    },
    TemplateFill {
        template: Box<Expr>,
        bindings: Vec<Binding>,
        span: Span,
    },
    Pack {
        bindings: Vec<Binding>,
        span: Span,
    },
    FormulaFill {
        formula: Box<Expr>,
        bindings: Vec<Binding>,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub struct ArgBinding {
    pub expr: Expr,
    pub josa: Option<String>,
    pub resolved_pin: Option<String>,
    #[allow(dead_code)]
    pub binding_reason: BindingReason,
    #[allow(dead_code)]
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingReason {
    Dictionary,
    Positional,
    UserFixed,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Path(path) => path.span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::Atom { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::SeedLiteral { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::Formula { span, .. } => *span,
            Expr::Assertion { span, .. } => *span,
            Expr::FormulaEval { span, .. } => *span,
            Expr::Template { span, .. } => *span,
            Expr::TemplateFill { span, .. } => *span,
            Expr::Pack { span, .. } => *span,
            Expr::FormulaFill { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormulaDialect {
    Ascii,
    Ascii1,
}

impl FormulaDialect {
    pub fn tag(self) -> &'static str {
        match self {
            FormulaDialect::Ascii => "#ascii",
            FormulaDialect::Ascii1 => "#ascii1",
        }
    }

    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "#ascii" => Some(FormulaDialect::Ascii),
            "#ascii1" => Some(FormulaDialect::Ascii1),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookKind {
    Start,
    End,
    EveryMadi,
    EveryNMadi(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleUnitKind {
    Pan,
    Madang,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractMode {
    Abort,
    Alert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractKind {
    Pre,
    Post,
}

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Clone, Debug)]
pub enum Literal {
    None,
    Bool(bool),
    Num(NumberLiteral),
    Str(String),
}

#[derive(Clone, Debug)]
pub struct NumberLiteral {
    pub raw: i64,
    pub unit: Option<UnitExpr>,
}
