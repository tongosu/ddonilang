use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use crate::file_meta::{format_file_meta, split_file_meta, FileMeta};
use serde::Serialize;

#[derive(Debug)]
pub struct CanonError {
    code: &'static str,
    message: String,
}

impl CanonError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl std::fmt::Display for CanonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code, self.message)
    }
}

impl std::error::Error for CanonError {}

fn has_root_hide_directive(input: &str) -> bool {
    for line in input.lines() {
        let trimmed = line.trim_start_matches(|ch| matches!(ch, ' ' | '\t' | '\r' | '\u{feff}'));
        if !trimmed.starts_with('#') {
            continue;
        }
        let rest = trimmed[1..].trim_start_matches(|ch| matches!(ch, ' ' | '\t'));
        if rest.starts_with("바탕숨김") || rest.starts_with("암묵살림") {
            return true;
        }
    }
    false
}

fn is_bridge_alias_type_name(type_name: &str) -> bool {
    matches!(
        type_name,
        "글" | "수" | "셈수" | "fixed64" | "sim_num" | "참거짓" | "논" | "bool" | "boolean"
    )
}

fn bridge_alias_matches_literal(type_name: &str, literal: &Literal) -> bool {
    match type_name {
        "글" => matches!(literal, Literal::Str(_)),
        "수" | "셈수" | "fixed64" | "sim_num" => matches!(literal, Literal::Num(_)),
        "참거짓" | "논" | "bool" | "boolean" => matches!(literal, Literal::Bool(_)),
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Ident(String),
    Atom(String),
    Template(String),
    Formula(String),
    BogaeMadangBlock(String),
    BogaeJangmyeonBlock(String),
    ExecPolicyBlock(String),
    JjaimBlock(String),
    GuseongBlock(String),
    String(String),
    Number(String),
    True,
    False,
    None,
    Show,
    Inspect,
    Manyak,
    Iramyeon,
    Ilttae,
    Aniramyeon,
    Anigo,
    Majeumyeon,
    Jeonjehae,
    Bojanghago,
    Goreugi,
    Repeat,
    During,
    Daehae,
    Break,
    ContinueLoop,
    Plus,
    PlusEqual,
    PlusArrow,
    Minus,
    MinusEqual,
    MinusArrow,
    Star,
    Slash,
    Percent,
    And,
    Or,
    RelationEq,
    Comma,
    EqEq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    DotDot,
    DotDotEq,
    Dot,
    Arrow,
    TypeArrow,
    SignalArrow,
    Colon,
    Define,
    Equals,
    Pipe,
    At,
    Question,
    Prompt,
    PromptBlock(String),
    Bang,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
}

struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    _input: &'a str,
}

mod lexer;

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch == '?' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch == '?' || ch == '~' || ch.is_alphanumeric()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclKind {
    Gureut,
    Butbak,
}

#[derive(Debug, Clone)]
struct DeclItem {
    name: String,
    kind: DeclKind,
    type_name: String,
    value: Option<Expr>,
    maegim: Option<MaegimSpec>,
}

#[derive(Debug, Clone)]
struct MaegimSpec {
    fields: Vec<Binding>,
}

#[derive(Debug, Clone)]
enum SurfaceStmt {
    RootDecl {
        items: Vec<DeclItem>,
    },
    Decl {
        name: String,
        type_name: String,
        value: Expr,
    },
    SeedDef {
        name: String,
        kind: String,
        params: Vec<Param>,
        body: Vec<SurfaceStmt>,
    },
    Assign {
        target: Path,
        value: Expr,
    },
    Show {
        value: Expr,
    },
    Inspect {
        value: Expr,
    },
    Boim {
        entries: Vec<Binding>,
    },
    BogaeMadangBlock {
        body: String,
    },
    JjaimBlock {
        body: String,
    },
    BogaeDraw,
    If {
        condition: Condition,
        then_body: Vec<SurfaceStmt>,
        else_body: Option<Vec<SurfaceStmt>>,
    },
    Contract {
        kind: ContractKind,
        mode: ContractMode,
        condition: Condition,
        then_body: Option<Vec<SurfaceStmt>>,
        else_body: Vec<SurfaceStmt>,
    },
    Choose {
        branches: Vec<SurfaceChooseBranch>,
        else_body: Option<Vec<SurfaceStmt>>,
        exhaustive: bool,
    },
    PromptChoose {
        branches: Vec<SurfaceChooseBranch>,
        else_body: Option<Vec<SurfaceStmt>>,
    },
    PromptAfter {
        value: Expr,
        body: Vec<SurfaceStmt>,
    },
    PromptCondition {
        condition: Condition,
        body: Vec<SurfaceStmt>,
    },
    PromptBlock {
        body: Vec<SurfaceStmt>,
    },
    ExecPolicyBlock {
        body: String,
    },
    Repeat {
        body: Vec<SurfaceStmt>,
    },
    While {
        condition: Condition,
        body: Vec<SurfaceStmt>,
    },
    ForEach {
        item: String,
        iterable: Expr,
        body: Vec<SurfaceStmt>,
    },
    Hook {
        name: String,
        suffix: HookSuffix,
        body: Vec<SurfaceStmt>,
    },
    Receive {
        kind: Option<String>,
        binding: Option<String>,
        condition: Option<Expr>,
        body: Vec<SurfaceStmt>,
    },
    EventReact {
        kind: String,
        body: Vec<SurfaceStmt>,
    },
    OpenBlock {
        body: Vec<SurfaceStmt>,
    },
    Return {
        value: Expr,
    },
    Send {
        sender: Option<Expr>,
        payload: Expr,
        receiver: Expr,
    },
    Expr {
        value: Expr,
    },
    Break,
    ContinueLoop,
}

#[derive(Debug, Clone)]
enum Stmt {
    RootDecl {
        items: Vec<DeclItem>,
    },
    Assign {
        target: Path,
        value: Expr,
    },
    SeedDef {
        name: String,
        kind: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    Show {
        value: Expr,
    },
    Inspect {
        value: Expr,
    },
    Boim {
        entries: Vec<Binding>,
    },
    BogaeMadangBlock {
        body: String,
    },
    JjaimBlock {
        body: String,
    },
    BogaeDraw,
    If {
        condition: Condition,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    Contract {
        kind: ContractKind,
        mode: ContractMode,
        condition: Condition,
        then_body: Option<Vec<Stmt>>,
        else_body: Vec<Stmt>,
    },
    Choose {
        branches: Vec<ChooseBranch>,
        else_body: Option<Vec<Stmt>>,
        exhaustive: bool,
    },
    PromptChoose {
        branches: Vec<ChooseBranch>,
        else_body: Option<Vec<Stmt>>,
    },
    PromptAfter {
        value: Expr,
        body: Vec<Stmt>,
    },
    PromptCondition {
        condition: Condition,
        body: Vec<Stmt>,
    },
    PromptBlock {
        body: Vec<Stmt>,
    },
    ExecPolicyBlock {
        body: String,
    },
    Repeat {
        body: Vec<Stmt>,
    },
    While {
        condition: Condition,
        body: Vec<Stmt>,
    },
    ForEach {
        item: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Hook {
        name: String,
        suffix: HookSuffix,
        body: Vec<Stmt>,
    },
    Receive {
        kind: Option<String>,
        binding: Option<String>,
        condition: Option<Expr>,
        body: Vec<Stmt>,
    },
    EventReact {
        kind: String,
        body: Vec<Stmt>,
    },
    OpenBlock {
        body: Vec<Stmt>,
    },
    Return {
        value: Expr,
    },
    Send {
        sender: Option<Expr>,
        payload: Expr,
        receiver: Expr,
    },
    Expr {
        value: Expr,
    },
    Break,
    ContinueLoop,
}

#[derive(Debug, Clone, Copy)]
enum HookSuffix {
    Halttae,
    Mada,
}

#[derive(Debug, Clone)]
enum Expr {
    Literal(Literal),
    Resource(String),
    Template {
        body: String,
    },
    TemplateApply {
        bindings: Vec<Binding>,
        body: String,
    },
    Formula {
        tag: Option<String>,
        body: String,
    },
    PromptExpr {
        expr: Box<Expr>,
    },
    PromptBlock {
        body: String,
    },
    Path(Path),
    FieldAccess {
        target: Box<Expr>,
        field: String,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    CallIn {
        name: String,
        bindings: Vec<Binding>,
    },
    Pack {
        bindings: Vec<Binding>,
    },
    Unit {
        value: Box<Expr>,
        unit: String,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Pipe {
        left: Box<Expr>,
        kind: PipeKind,
        right: Box<Expr>,
    },
    SeedLiteral {
        param: String,
        body: Box<Expr>,
    },
    Grouped {
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    value: Expr,
}

#[derive(Debug, Clone)]
struct Param {
    name: String,
    type_name: Option<String>,
    default: Option<Expr>,
}

#[derive(Debug, Clone)]
struct Condition {
    expr: Expr,
    style: ConditionStyle,
    negated: bool,
}

#[derive(Debug, Clone, Copy)]
enum ConditionStyle {
    Current,
    Plain,
    Thunk,
}

#[derive(Debug, Clone, Copy)]
enum ContractKind {
    Pre,
    Post,
}

#[derive(Debug, Clone, Copy)]
enum ContractMode {
    Abort,
    Alert,
}

#[derive(Debug, Clone)]
struct ChooseBranch {
    condition: Condition,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct SurfaceChooseBranch {
    condition: Condition,
    body: Vec<SurfaceStmt>,
}

#[derive(Debug, Clone, Copy)]
enum UnaryOp {
    Neg,
}

#[derive(Debug, Clone, Copy)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    RelationEq,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone, Copy)]
enum PipeKind {
    Haseo,
    Hago,
}

#[derive(Debug, Clone)]
enum Literal {
    Str(String),
    Num(String),
    Bool(bool),
    None,
    Atom(String),
}

#[derive(Debug, Clone)]
struct Path {
    segments: Vec<String>,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    bridge: bool,
    deprecated_block_header_colon_count: usize,
    seed_kind_stack: Vec<String>,
    chaebi_forbidden_depth: usize,
}

mod parser;

fn is_allowed_jjaim_header(raw: &str) -> bool {
    let mut header = raw.trim();
    if let Some(stripped) = header.strip_suffix(':') {
        header = stripped.trim_end();
    }
    if header.is_empty() {
        return true;
    }
    let fixed = [
        "입력",
        "상태",
        "출력",
        "수식",
        "모양",
        "보개",
        "보개마당",
        "채비",
        "씨앗",
        "고르기",
        "반복",
        "되풀이",
    ];
    if fixed
        .iter()
        .any(|k| header == *k || header.starts_with(&format!("{} ", k)))
    {
        return true;
    }
    header.contains("마다")
        || header.contains("할때")
        || header.contains("일때")
        || header.contains("오면")
        || header.contains("대해")
        || header.contains("동안")
}

fn validate_jjaim_block_body(body: &str) -> Result<(), CanonError> {
    let mut depth: i32 = 0;
    for raw_line in body.lines() {
        let line = raw_line.trim();
        if depth == 0 && !line.is_empty() && !line.starts_with("//") {
            if let Some(open_idx) = line.find('{') {
                let head = line[..open_idx].trim();
                if !head.is_empty() && !is_allowed_jjaim_header(head) {
                    return Err(CanonError::new(
                        "E_JJAIM_SUBBLOCK_INVALID",
                        format!(
                            "짜임 상위 항목은 입력/상태/출력/수식/모양/매마디·조건·이벤트 계열만 허용됩니다: {}",
                            head
                        ),
                    ));
                }
            }
        }
        for ch in raw_line.chars() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
            }
        }
    }
    Ok(())
}

pub struct CanonOutput {
    pub ddn: String,
    pub guseong_flat_json: String,
    pub alrim_plan_json: String,
    #[allow(dead_code)]
    pub block_editor_plan_json: String,
    pub exec_policy_map_json: String,
    pub maegim_control_json: String,
    pub meta: FileMeta,
    pub warnings: Vec<String>,
}

struct FrontdoorFallbackPlans {
    guseong_flat_json: String,
    alrim_plan_json: String,
    exec_policy_map_json: String,
    maegim_control_json: String,
}

pub use ddonirang_lang::preprocess_frontdoor_source;

pub(crate) fn extract_prompt_string_literals(input: &str) -> Result<Vec<String>, CanonError> {
    // F5-GA[CANON_TOKENS] This scan and has_exec_policy_surface index only under their own i/index<len loop guard; each policy lookahead separately short-circuits on j<len.
    let prepared = preprocess_frontdoor_source(input);
    let tokens = Lexer::tokenize(&prepared)?;
    let mut prompts = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if !matches!(tokens[index].kind, TokenKind::Prompt) {
            index += 1;
            continue;
        }
        if !matches!(
            tokens.get(index + 1).map(|token| &token.kind),
            Some(TokenKind::LParen)
        ) {
            index += 1;
            continue;
        }

        let prompt = match tokens.get(index + 2).map(|token| &token.kind) {
            Some(TokenKind::String(prompt)) => prompt.clone(),
            _ => {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_STRING",
                    "AI 요청 추출에는 문자열 리터럴이 필요합니다.",
                ))
            }
        };
        if !matches!(
            tokens.get(index + 3).map(|token| &token.kind),
            Some(TokenKind::RParen)
        ) {
            return Err(CanonError::new(
                "E_CANON_UNEXPECTED_TOKEN",
                "AI 요청 문자열 뒤에는 닫는 괄호가 필요합니다.",
            ));
        }
        prompts.push(prompt);
        index += 4;
    }

    Ok(prompts)
}

pub fn has_exec_policy_surface(input: &str) -> bool {
    let prepared = preprocess_frontdoor_source(input);
    // 빠른 음성 판정: 실행정책/너머 키워드 자체가 없으면 exec-policy 표면이 없다.
    if !prepared.contains("너머") && !prepared.contains("실행정책") {
        return false;
    }
    let Ok(tokens) = Lexer::tokenize(&prepared) else {
        // 토큰화가 실패해도 키워드 기반으로만 보수 판정한다.
        return prepared.contains("너머") || prepared.contains("실행정책");
    };

    let mut i = 0usize;
    while i < tokens.len() {
        match &tokens[i].kind {
            TokenKind::ExecPolicyBlock(_) => return true,
            TokenKind::Ident(name) if name == "너머" => {
                let mut j = i + 1;
                while j < tokens.len() && matches!(tokens[j].kind, TokenKind::Newline) {
                    j += 1;
                }
                if j < tokens.len() && matches!(tokens[j].kind, TokenKind::LBrace) {
                    return true;
                }
                if j < tokens.len() && matches!(tokens[j].kind, TokenKind::Colon) {
                    j += 1;
                    while j < tokens.len() && matches!(tokens[j].kind, TokenKind::Newline) {
                        j += 1;
                    }
                    if j < tokens.len() && matches!(tokens[j].kind, TokenKind::LBrace) {
                        return true;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

pub use ddonirang_lang::has_legacy_boim_surface;

pub fn canonicalize(input: &str, bridge: bool) -> Result<CanonOutput, CanonError> {
    // This legacy projection canonicalizer has no typed message-transfer IR.
    // Keep it fail-closed: the shared language frontdoor is the sole owner of
    // successor async/rendezvous canon, and historical async is available only
    // through the explicit migration-evidence API.
    let tokens = Lexer::tokenize(input)?;
    if tokens.windows(3).any(|window| {
        // F5-GA[CANON_WINDOW] slice::windows(3) yields only complete three-element windows.
        matches!(window[0].kind, TokenKind::RBrace)
            && matches!(&window[1].kind, TokenKind::Ident(name) if name == "인것")
            && matches!(window[2].kind, TokenKind::During)
    }) {
        return Err(CanonError::new(
            "E_WHILE_SURFACE",
            "일반 반복은 '{ 조건 }인 동안 { ... }.'로 작성해야 합니다.",
        ));
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::SignalArrow))
    {
        return Err(CanonError::new(
            "E_CANON_TYPED_TRANSFER_REQUIRES_SHARED_FRONTDOOR",
            "`~~>` rendezvous 정본화는 공통 typed language frontdoor를 사용해야 합니다.",
        ));
    }
    canonicalize_with_fallback_mode(input, bridge, true)
}

fn normalized_maegim_source_path(source_path: &str) -> Result<String, CanonError> {
    let normalized = source_path.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.contains(['\r', '\n', '#']) {
        return Err(CanonError::new(
            "E_MAEGIM_SOURCE_PATH_INVALID",
            "채비 source_ref에 사용할 안전한 DDN 경로가 필요합니다",
        ));
    }
    Ok(normalized)
}

fn maegim_control_literal_locator(
    source: &str,
    control_name: &str,
) -> Result<Option<(usize, usize, usize, usize, String)>, CanonError> {
    let pattern = format!(
        r"(?m)^[\t ]*(?P<name>{})[\t ]*:[^\r\n]*?(?:<-|←|:=)[\t ]*\([\t ]*(?P<value>[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+))[\t ]*\)[\t ]*매김[\t ]*\{{",
        regex::escape(control_name),
    );
    let matcher = regex::Regex::new(&pattern).map_err(|err| {
        CanonError::new(
            "E_MAEGIM_SOURCE_LOCATOR_INVALID",
            format!("채비 원문 위치 판정기를 만들 수 없습니다: {err}"),
        )
    })?;
    let mut matches = matcher.captures_iter(source);
    let Some(captures) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Ok(None);
    }
    let Some(name_match) = captures.name("name") else {
        return Ok(None);
    };
    let Some(value_match) = captures.name("value") else {
        return Ok(None);
    };
    Ok(Some((
        name_match.start(),
        name_match.end(),
        value_match.start(),
        value_match.end(),
        value_match.as_str().to_string(),
    )))
}

pub fn maegim_control_plan_with_source(
    source: &str,
    source_path: &str,
) -> Result<String, CanonError> {
    let source_path = normalized_maegim_source_path(source_path)?;
    let output = canonicalize(source, false)?;
    let mut plan: serde_json::Value = serde_json::from_str(&output.maegim_control_json)
        .map_err(|err| CanonError::new("E_MAEGIM_PLAN_JSON_INVALID", err.to_string()))?;
    let controls = plan
        .get_mut("controls")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| {
            CanonError::new(
                "E_MAEGIM_PLAN_CONTROLS_INVALID",
                "채비 의미 계획에 controls 배열이 없습니다",
            )
        })?;
    for control in controls {
        let Some(object) = control.as_object_mut() else {
            continue;
        };
        let name = object
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let Some((name_start, name_end, value_start, value_end, value_text)) =
            maegim_control_literal_locator(source, &name)?
        else {
            object.insert(
                "source_edit".to_string(),
                serde_json::json!({"status": "unsupported"}),
            );
            continue;
        };
        object.insert(
            "source_ref".to_string(),
            serde_json::Value::String(format!("{source_path}#bytes={name_start}:{name_end}")),
        );
        object.insert(
            "value_source_ref".to_string(),
            serde_json::Value::String(format!("{source_path}#bytes={value_start}:{value_end}")),
        );
        object.insert(
            "source_edit".to_string(),
            serde_json::json!({
                "status": "ready",
                "byte_start": value_start,
                "byte_end": value_end,
                "expected_text": value_text,
            }),
        );
    }
    serde_json::to_string_pretty(&plan)
        .map(|text| format!("{text}\n"))
        .map_err(|err| CanonError::new("E_MAEGIM_PLAN_JSON_WRITE", err.to_string()))
}

pub fn apply_maegim_source_edit(
    source: &str,
    source_path: &str,
    control_name: &str,
    value_text: &str,
) -> Result<String, CanonError> {
    let source_path = normalized_maegim_source_path(source_path)?;
    let control_name = control_name.trim();
    let value_text = value_text.trim();
    if control_name.is_empty() {
        return Err(CanonError::new(
            "E_MAEGIM_CONTROL_NAME_EMPTY",
            "바꿀 채비 이름이 필요합니다",
        ));
    }
    let numeric = regex::Regex::new(r"^[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)$")
        .map_err(|err| CanonError::new("E_MAEGIM_VALUE_MATCHER_INVALID", err.to_string()))?;
    if !numeric.is_match(value_text)
        || value_text
            .parse::<f64>()
            .map(|value| !value.is_finite())
            .unwrap_or(true)
    {
        return Err(CanonError::new(
            "E_MAEGIM_VALUE_INVALID",
            "채비 값은 유한한 숫자 리터럴이어야 합니다",
        ));
    }

    let plan_text = maegim_control_plan_with_source(source, &source_path)?;
    let plan: serde_json::Value = serde_json::from_str(&plan_text)
        .map_err(|err| CanonError::new("E_MAEGIM_PLAN_JSON_INVALID", err.to_string()))?;
    let matches: Vec<&serde_json::Value> = plan
        .get("controls")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|control| {
            control.get("name").and_then(serde_json::Value::as_str) == Some(control_name)
        })
        .collect();
    if matches.len() != 1 {
        return Err(CanonError::new(
            "E_MAEGIM_CONTROL_NOT_UNIQUE",
            format!("원문에서 채비 '{control_name}'을 하나로 확정할 수 없습니다"),
        ));
    }
    // F5-GA[CANON_ONE_CONTROL] The unchanged local matches vector was required to have exactly one entry immediately above.
    let control = matches[0];
    let source_edit = control
        .get("source_edit")
        .unwrap_or(&serde_json::Value::Null);
    if source_edit
        .get("status")
        .and_then(serde_json::Value::as_str)
        != Some("ready")
    {
        return Err(CanonError::new(
            "E_MAEGIM_SOURCE_EDIT_UNSUPPORTED",
            format!("채비 '{control_name}'의 초기값을 안전하게 고칠 수 없습니다"),
        ));
    }
    let byte_start = source_edit
        .get("byte_start")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            CanonError::new(
                "E_MAEGIM_SOURCE_SPAN_INVALID",
                "채비 값 시작 위치가 없습니다",
            )
        })?;
    let byte_end = source_edit
        .get("byte_end")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            CanonError::new("E_MAEGIM_SOURCE_SPAN_INVALID", "채비 값 끝 위치가 없습니다")
        })?;
    let expected_text = source_edit
        .get("expected_text")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if source.get(byte_start..byte_end) != Some(expected_text)
        || !source.is_char_boundary(byte_start)
        || !source.is_char_boundary(byte_end)
    {
        return Err(CanonError::new(
            "E_MAEGIM_SOURCE_SPAN_STALE",
            "채비 의미 계획과 현재 원문 byte 위치가 다릅니다",
        ));
    }

    let requested_value = value_text.parse::<f64>().unwrap_or_default();
    if let Some(range) = control.get("range") {
        let min = range
            .get("min_expr_canon")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<f64>().ok());
        let max = range
            .get("max_expr_canon")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<f64>().ok());
        if min.zip(max).is_some_and(|(min, max)| {
            requested_value < min.min(max) || requested_value > min.max(max)
        }) {
            return Err(CanonError::new(
                "E_MAEGIM_VALUE_OUT_OF_RANGE",
                format!("채비 '{control_name}' 값이 DDN 매김 범위를 벗어났습니다"),
            ));
        }
    }

    let mut edited = source.to_string();
    edited.replace_range(byte_start..byte_end, value_text);
    let edited_plan_text = maegim_control_plan_with_source(&edited, &source_path)?;
    let edited_plan: serde_json::Value = serde_json::from_str(&edited_plan_text)
        .map_err(|err| CanonError::new("E_MAEGIM_PLAN_JSON_INVALID", err.to_string()))?;
    let edited_control = edited_plan
        .get("controls")
        .and_then(serde_json::Value::as_array)
        .and_then(|controls| {
            controls.iter().find(|control| {
                control.get("name").and_then(serde_json::Value::as_str) == Some(control_name)
            })
        })
        .ok_or_else(|| {
            CanonError::new(
                "E_MAEGIM_SOURCE_EDIT_REPARSE_FAILED",
                "바꾼 원문에서 같은 채비를 다시 확인하지 못했습니다",
            )
        })?;
    let value_canon = edited_control
        .get("init_expr_canon")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if value_canon
        .parse::<f64>()
        .map(|value| value != requested_value)
        .unwrap_or(true)
    {
        return Err(CanonError::new(
            "E_MAEGIM_SOURCE_EDIT_VALUE_MISMATCH",
            "바꾼 원문의 Rust 의미 값이 요청값과 다릅니다",
        ));
    }
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "ddn.maegim_source_edit_result.v1",
        "status": "ready",
        "source_path": source_path,
        "control_name": control_name,
        "source_ref": edited_control.get("source_ref").cloned().unwrap_or(serde_json::Value::Null),
        "value_source_ref": edited_control.get("value_source_ref").cloned().unwrap_or(serde_json::Value::Null),
        "value_canon": value_canon,
        "source_text": edited,
    }))
    .map(|text| format!("{text}\n"))
    .map_err(|err| CanonError::new("E_MAEGIM_SOURCE_EDIT_JSON_WRITE", err.to_string()))
}

pub fn lower_single_imja_event_subset(input: &str) -> Result<Option<String>, CanonError> {
    let prepared = preprocess_frontdoor_source(input);
    if ddonirang_lang::find_legacy_header(&prepared).is_some()
        || ddonirang_lang::has_legacy_boim_surface(&prepared)
    {
        return Ok(None);
    }
    let meta_parse = split_file_meta(&prepared);
    let root_hide = has_root_hide_directive(&prepared);
    let tokens = Lexer::tokenize(&meta_parse.stripped)?;
    let mut parser = Parser::new(tokens, false);
    let surface = match parser.parse_program() {
        Ok(surface) => surface,
        Err(_) => return Ok(None),
    };
    let declared = collect_declared_names(&surface);
    let canonical = surface_to_canonical(surface, &declared, false, "바탕", root_hide)?;
    Ok(lower_single_imja_event_subset_program(&canonical))
}

fn canonicalize_with_fallback_mode(
    input: &str,
    bridge: bool,
    allow_frontdoor_fallback: bool,
) -> Result<CanonOutput, CanonError> {
    let prepared = preprocess_frontdoor_source(input);
    if let Some((line, key)) = ddonirang_lang::find_legacy_header(&prepared) {
        return Err(CanonError::new(
            "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN",
            format!("line={line} key={key} use=설정{{}}/매김{{}}/설정.보개"),
        ));
    }
    if ddonirang_lang::has_legacy_boim_surface(&prepared) {
        return Err(CanonError::new(
            "E_BOIM_LEGACY_REMOVED",
            "`보임 {}` 블록은 제거되었습니다. `(<이름>: <식>, ...) 내보임.`을 사용하세요",
        ));
    }
    let shared_assignment = ddonirang_lang::has_right_assignment_arrow(&prepared)
        .map_err(|err| CanonError::new("E_CANON_LEX", err.to_string()))?;
    let shared_natural_foreach = ddonirang_lang::has_natural_foreach_surface(&prepared)
        .map_err(|err| CanonError::new("E_CANON_LEX", err.to_string()))?;
    if shared_assignment || shared_natural_foreach {
        // Shared lang owns raw assignment orientation and natural iteration
        // lowering/current-item selection. This adapter only consumes the
        // explicit canonical result in the existing execution plan.
        let original_meta = split_file_meta(&prepared).meta;
        let canonical = ddonirang_lang::parse_frontdoor_and_normalize(
            &prepared,
            if shared_natural_foreach {
                "<canon-shared-natural-foreach>"
            } else {
                "<canon-shared-assignment>"
            },
            ddonirang_lang::NormalizationLevel::N1,
        )
        .map_err(|err| CanonError::new(err.code(), err.to_string()))?;
        let mut output = canonicalize_with_fallback_mode(
            &canonical,
            bridge,
            if shared_natural_foreach {
                false
            } else {
                allow_frontdoor_fallback
            },
        )?;
        if shared_natural_foreach {
            let mut ddn = String::new();
            ddn.push_str(&format_file_meta(&original_meta));
            ddn.push_str(canonical.trim_end());
            ddn.push('\n');
            output.ddn = ddn;
        }
        output.meta = original_meta;
        return Ok(output);
    }
    let meta_parse = split_file_meta(&prepared);
    validate_decl_item_maegim_contract(&meta_parse.stripped)?;
    let root_hide = has_root_hide_directive(&prepared);
    let normalized_frontdoor = match ddonirang_lang::parse_frontdoor_and_normalize(
        &meta_parse.stripped,
        "<canon-frontdoor-prepared>",
        ddonirang_lang::NormalizationLevel::N1,
    ) {
        Ok(normalized) => Some(normalized),
        Err(err) if err.message.contains("E_WHILE_SURFACE:") => {
            return Err(CanonError::new("E_WHILE_SURFACE", err.message));
        }
        Err(_) => None,
    };
    // Parse the current-line source first. Frontdoor normalization intentionally
    // strips some compatibility-only suffixes (notably decl-item `매김 {}`),
    // so using it as the primary parse input loses control metadata.
    let parse_source = &meta_parse.stripped;
    let default_root = "바탕";
    let tokens = Lexer::tokenize(parse_source)?;
    let legacy_guseong_alias_seen = tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::GuseongBlock(_)));
    let legacy_bogae_madang_alias_seen = tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::BogaeJangmyeonBlock(_)));
    let mut parser = Parser::new(tokens, bridge);
    let surface = match parser.parse_program() {
        Ok(surface) => surface,
        Err(primary_err) => {
            if matches!(
                primary_err.code(),
                "E_WHILE_SURFACE"
                    | "E_CHAEBI_IN_LOOP"
                    | "E_CANON_RECEIVE_OUTSIDE_IMJA"
                    | "E_EVENT_SURFACE_ALIAS_FORBIDDEN"
                    | "E_CANON_HOOK_EVERY_N_MADI_INTERVAL_INVALID"
                    | "E_CANON_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED"
                    | "E_CANON_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED"
                    | "E_CANON_MAEGIM_NESTED_SECTION_UNSUPPORTED"
                    | "E_CANON_MAEGIM_NESTED_FIELD_UNSUPPORTED"
            ) {
                return Err(primary_err);
            }
            if !allow_frontdoor_fallback {
                return Err(primary_err);
            }
            let normalized = normalized_frontdoor.clone().or_else(|| {
                ddonirang_lang::parse_frontdoor_and_normalize(
                    &meta_parse.stripped,
                    "<canon-fallback-frontdoor>",
                    ddonirang_lang::NormalizationLevel::N1,
                )
                .ok()
            });

            if let Some(normalized_text) = normalized.as_ref() {
                if let Ok(mut strict_output) =
                    canonicalize_with_fallback_mode(normalized_text, bridge, false)
                {
                    let mut ddn = String::new();
                    ddn.push_str(&format_file_meta(&meta_parse.meta));
                    ddn.push_str(strict_output.ddn.trim_end());
                    ddn.push('\n');
                    strict_output.ddn = ddn;
                    strict_output.meta = meta_parse.meta.clone();
                    return Ok(strict_output);
                }
            }

            let fallback_source = normalized.as_deref().unwrap_or(&meta_parse.stripped);
            let fallback_plans = build_frontdoor_fallback_plans(fallback_source, &meta_parse.stripped)?;
            let ddn_body =
                normalized.unwrap_or_else(|| preprocess_frontdoor_source(&meta_parse.stripped));
            let mut ddn = String::new();
            ddn.push_str(&format_file_meta(&meta_parse.meta));
            ddn.push_str(ddn_body.trim_end());
            ddn.push('\n');
            let mut warnings = Vec::new();
            if normalized_frontdoor.is_some() {
                warnings.push("W_CANON_FALLBACK_LANG_NORMALIZER".to_string());
            }
            warnings.push("W_CANON_PASSTHROUGH".to_string());
            return Ok(CanonOutput {
                ddn,
                guseong_flat_json: fallback_plans.guseong_flat_json,
                alrim_plan_json: fallback_plans.alrim_plan_json,
                block_editor_plan_json: "{}\n".to_string(),
                exec_policy_map_json: fallback_plans.exec_policy_map_json,
                maegim_control_json: fallback_plans.maegim_control_json,
                meta: meta_parse.meta,
                warnings,
            });
        }
    };
    let deprecated_block_header_colon_count = parser.deprecated_block_header_colon_count();
    let flatten_plan = build_guseong_flatten_plan_surface(&surface)?;
    let guseong_flat_json = flatten_plan.to_json_string()?;

    let declared = collect_declared_names(&surface);

    let mut warnings = Vec::new();
    collect_top_level_chaebi_reassign_warnings(&surface, &mut warnings);
    let canonical = surface_to_canonical(surface, &declared, bridge, default_root, root_hide)?;

    let ddn = format_program(&canonical);
    let alrim_plan_json = build_alrim_event_plan(&canonical).to_json_string()?;
    let block_editor_plan_json = build_block_editor_plan(&canonical).to_json_string()?;
    let exec_policy_map_json = build_exec_policy_effect_map(&canonical).to_json_string()?;
    let maegim_control_json = build_maegim_control_plan(&canonical).to_json_string()?;
    if !meta_parse.dup_keys.is_empty() {
        warnings.push(format!("META_DUP_KEY {}", meta_parse.dup_keys.join(", ")));
    }
    if legacy_guseong_alias_seen {
        warnings.push(
            "W_JJAIM_ALIAS_DEPRECATED `구성`은 별칭입니다. 정본 키워드 `짜임`을 사용하세요"
                .to_string(),
        );
    }
    if legacy_bogae_madang_alias_seen {
        warnings.push(
            "W_BOGAE_MADANG_ALIAS_DEPRECATED `보개장면`은 별칭입니다. 정본 키워드 `보개마당`을 사용하세요"
                .to_string(),
        );
    }
    if deprecated_block_header_colon_count > 0 {
        warnings.push(format!(
            "W_BLOCK_HEADER_COLON_DEPRECATED 블록 헤더의 `키워드:` 표기는 예정된 비권장입니다. `키워드 {{` 표기로 전환하세요 (count={})",
            deprecated_block_header_colon_count
        ));
    }
    let mut output = String::new();
    output.push_str(&format_file_meta(&meta_parse.meta));
    output.push_str(&ddn);
    Ok(CanonOutput {
        ddn: output,
        guseong_flat_json,
        alrim_plan_json,
        block_editor_plan_json,
        exec_policy_map_json,
        maegim_control_json,
        meta: meta_parse.meta,
        warnings,
    })
}

fn validate_decl_item_maegim_contract(source: &str) -> Result<(), CanonError> {
    // F5-GA[CANON_LINES] Both outer and inner lines[idx] reads follow their own idx<lines.len condition on an immutable vector.
    let lines: Vec<&str> = source.lines().collect();
    let mut idx = 0usize;
    while idx < lines.len() {
        let line = lines[idx].trim();
        if !line.contains("매김 {") && !line.contains("조건 {") {
            idx += 1;
            continue;
        }
        let Some(keyword_idx) = line.find("매김 {").or_else(|| line.find("조건 {")) else {
            idx += 1;
            continue;
        };
        let before_keyword = line[..keyword_idx].trim_end();
        if !(before_keyword.ends_with(')')) {
            return Err(CanonError::new(
                "E_CANON_MAEGIM_GROUPED_VALUE_REQUIRED",
                "매김은 `(<식>) 매김 { ... }` 형태만 허용됩니다.",
            ));
        }

        let mut has_step = false;
        let mut has_split_count = false;
        idx += 1;
        while idx < lines.len() {
            let inner = lines[idx].trim();
            if inner.starts_with("}.") || inner == "}" {
                break;
            }
            if inner.starts_with("간격:") || inner.contains(".간격:") {
                has_step = true;
            }
            if inner.starts_with("분할수:") || inner.contains(".분할수:") {
                has_split_count = true;
            }
            if has_step && has_split_count {
                return Err(CanonError::new(
                    "E_CANON_MAEGIM_STEP_SPLIT_CONFLICT",
                    "`매김`에서 `간격`과 `분할수`는 동시에 사용할 수 없습니다.",
                ));
            }
            idx += 1;
        }
        idx += 1;
    }
    Ok(())
}

fn collect_declared_names(surface: &[SurfaceStmt]) -> HashSet<String> {
    let mut declared = HashSet::new();
    for stmt in surface {
        match stmt {
            SurfaceStmt::Decl { name, .. } => {
                declared.insert(name.clone());
            }
            SurfaceStmt::RootDecl { items, .. } => {
                for item in items {
                    if item.kind == DeclKind::Gureut {
                        declared.insert(item.name.clone());
                    }
                }
            }
            _ => {}
        }
    }
    declared
}

fn surface_to_canonical(
    surface: Vec<SurfaceStmt>,
    declared: &HashSet<String>,
    bridge: bool,
    default_root: &str,
    root_hide: bool,
) -> Result<Vec<Stmt>, CanonError> {
    let mut canonical = Vec::new();
    for stmt in surface {
        match stmt {
            SurfaceStmt::RootDecl { items } => {
                let items = items
                    .into_iter()
                    .map(|item| DeclItem {
                        name: item.name,
                        kind: item.kind,
                        type_name: item.type_name,
                        value: item.value.map(|expr| {
                            canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                        }),
                        maegim: item.maegim.map(|spec| MaegimSpec {
                            fields: spec
                                .fields
                                .into_iter()
                                .map(|binding| Binding {
                                    name: binding.name,
                                    value: canonicalize_expr(
                                        binding.value,
                                        declared,
                                        bridge,
                                        default_root,
                                        root_hide,
                                    ),
                                })
                                .collect(),
                        }),
                    })
                    .collect();
                canonical.push(Stmt::RootDecl { items });
            }
            SurfaceStmt::Decl {
                name,
                type_name,
                value,
            } => {
                if !is_bridge_alias_type_name(&type_name) {
                    return Err(CanonError::new(
                        "E_CANON_BAD_ALIAS",
                        format!("알 수 없는 별칭입니다: {}", type_name),
                    ));
                }
                let target = Path {
                    segments: vec![default_root.to_string(), name],
                };
                let value = canonicalize_expr(value, declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Assign { target, value });
            }
            SurfaceStmt::Assign { target, value } => {
                if root_hide
                    && !matches!(
                        target.segments.first().map(|seg| seg.as_str()),
                        Some("살림" | "바탕" | "샘")
                    )
                {
                    if let Some(name) = target.segments.first() {
                        if !declared.contains(name) {
                            return Err(CanonError::new(
                                "E_CANON_ROOT_HIDE_UNDECLARED",
                                format!("채비에 선언되지 않은 전역 쓰기: {}", name),
                            ));
                        }
                    }
                }
                let target = canonicalize_path(target, declared, bridge, default_root, root_hide);
                let value = canonicalize_expr(value, declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Assign { target, value });
            }
            SurfaceStmt::SeedDef {
                name,
                kind,
                params,
                body,
            } => {
                let params = params
                    .into_iter()
                    .map(|param| Param {
                        name: param.name,
                        type_name: param.type_name,
                        default: param.default.map(|expr| {
                            canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                        }),
                    })
                    .collect();
                let body = canonicalize_body(body, declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::SeedDef {
                    name,
                    kind,
                    params,
                    body,
                });
            }
            SurfaceStmt::Send {
                sender,
                payload,
                receiver,
            } => {
                canonical.push(Stmt::Send {
                    sender: sender.map(|expr| {
                        canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                    }),
                    payload: canonicalize_expr(payload, declared, bridge, default_root, root_hide),
                    receiver: canonicalize_expr(
                        receiver,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                });
            }
            SurfaceStmt::Show { value } => {
                let value = canonicalize_expr(value, declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Show { value });
            }
            SurfaceStmt::Inspect { value } => {
                let value = canonicalize_expr(value, declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Inspect { value });
            }
            SurfaceStmt::Boim { entries } => {
                let entries = entries
                    .into_iter()
                    .map(|entry| Binding {
                        name: entry.name,
                        value: canonicalize_expr(
                            entry.value,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                    })
                    .collect();
                canonical.push(Stmt::Boim { entries });
            }
            SurfaceStmt::BogaeMadangBlock { body, .. } => {
                canonical.push(Stmt::BogaeMadangBlock { body });
            }
            SurfaceStmt::JjaimBlock { body, .. } => {
                canonical.push(Stmt::JjaimBlock { body });
            }
            SurfaceStmt::BogaeDraw => {
                canonical.push(Stmt::BogaeDraw);
            }
            SurfaceStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition =
                    canonicalize_condition(condition, declared, bridge, default_root, root_hide);
                let then_body =
                    canonicalize_body(then_body, declared, bridge, default_root, root_hide)?;
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                canonical.push(Stmt::If {
                    condition,
                    then_body,
                    else_body,
                });
            }
            SurfaceStmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
            } => {
                let condition =
                    canonicalize_condition(condition, declared, bridge, default_root, root_hide);
                let else_body =
                    canonicalize_body(else_body, declared, bridge, default_root, root_hide)?;
                let then_body = match then_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                canonical.push(Stmt::Contract {
                    kind,
                    mode,
                    condition,
                    then_body,
                    else_body,
                });
            }
            SurfaceStmt::Repeat { body } => {
                let body = canonicalize_body(body, declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::Repeat { body });
            }
            SurfaceStmt::While { condition, body } => {
                let condition =
                    canonicalize_condition(condition, declared, bridge, default_root, root_hide);
                let body = canonicalize_body(body, declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::While { condition, body });
            }
            SurfaceStmt::ForEach {
                item,
                iterable,
                body,
            } => {
                let iterable =
                    canonicalize_expr(iterable, declared, bridge, default_root, root_hide);
                let body = canonicalize_body(body, declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::ForEach {
                    item,
                    iterable,
                    body,
                });
            }
            SurfaceStmt::Hook { name, suffix, body } => {
                let body = canonicalize_body(body, declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::Hook { name, suffix, body });
            }
            SurfaceStmt::Receive {
                kind,
                binding,
                condition,
                body,
            } => {
                canonical.push(Stmt::Receive {
                    kind,
                    binding,
                    condition: condition.map(|expr| {
                        canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                    }),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::EventReact { kind, body } => {
                canonical.push(Stmt::EventReact {
                    kind,
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::OpenBlock { body } => {
                let body = canonicalize_body(body, declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::OpenBlock { body });
            }
            SurfaceStmt::Return { value } => {
                let value = canonicalize_expr(value, declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Return { value });
            }
            SurfaceStmt::Expr { value } => {
                let value = canonicalize_expr(value, declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Expr { value });
            }
            SurfaceStmt::Break => {
                canonical.push(Stmt::Break);
            }
            SurfaceStmt::ContinueLoop => {
                canonical.push(Stmt::ContinueLoop);
            }
            SurfaceStmt::PromptChoose {
                branches,
                else_body,
            } => {
                let mut mapped = Vec::new();
                for branch in branches {
                    mapped.push(ChooseBranch {
                        condition: canonicalize_condition(
                            branch.condition,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                        body: canonicalize_body(
                            branch.body,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        )?,
                    });
                }
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                canonical.push(Stmt::PromptChoose {
                    branches: mapped,
                    else_body,
                });
            }
            SurfaceStmt::PromptAfter { value, body } => {
                canonical.push(Stmt::PromptAfter {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptCondition { condition, body } => {
                canonical.push(Stmt::PromptCondition {
                    condition: canonicalize_condition(
                        condition,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptBlock { body } => {
                canonical.push(Stmt::PromptBlock {
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::ExecPolicyBlock { body } => {
                canonical.push(Stmt::ExecPolicyBlock { body });
            }
            SurfaceStmt::Choose {
                branches,
                else_body,
                exhaustive,
            } => {
                let mut mapped = Vec::new();
                for branch in branches {
                    mapped.push(ChooseBranch {
                        condition: canonicalize_condition(
                            branch.condition,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                        body: canonicalize_body(
                            branch.body,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        )?,
                    });
                }
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                canonical.push(Stmt::Choose {
                    branches: mapped,
                    else_body,
                    exhaustive,
                });
            }
        }
    }
    Ok(canonical)
}

mod actor_lowering;
use actor_lowering::{lower_single_imja_event_subset_program};

fn collect_top_level_chaebi_reassign_warnings(surface: &[SurfaceStmt], warnings: &mut Vec<String>) {
    for stmt in surface {
        let SurfaceStmt::SeedDef { body, .. } = stmt else {
            continue;
        };
        let mut declared_names = HashSet::new();
        for body_stmt in body {
            match body_stmt {
                SurfaceStmt::RootDecl { items } => {
                    for item in items {
                        declared_names.insert(item.name.clone());
                    }
                }
                SurfaceStmt::Assign { target, value } => {
                    if target.segments.len() != 1 {
                        continue;
                    }
                    let Some(name) = target.segments.first() else {
                        continue;
                    };
                    if !declared_names.contains(name) {
                        continue;
                    }
                    if !is_redundant_top_level_chaebi_reassign_value(value) {
                        continue;
                    }
                    warnings.push(format!(
                        "W_CHAEBI_REDUNDANT_TOP_REASSIGN `채비 {{}}`에서 이미 준비한 `{}`를 같은 최상위에서 다시 대입했습니다. 중복 준비가 아니면 일반 대입만 남기세요.",
                        name
                    ));
                }
                _ => {}
            }
        }
    }
}

fn is_redundant_top_level_chaebi_reassign_value(value: &Expr) -> bool {
    match value {
        Expr::Literal(_) => true,
        Expr::Grouped { expr } => is_redundant_top_level_chaebi_reassign_value(expr),
        _ => false,
    }
}

#[derive(Serialize)]
struct FallbackGuseongEnvelope {
    schema: &'static str,
    groups: Vec<FallbackGuseongGroup>,
}

#[derive(Serialize)]
struct FallbackGuseongGroup {
    id: String,
}

#[derive(Serialize)]
struct FallbackAlrimEnvelope {
    schema: &'static str,
    plans: Vec<String>,
}

#[derive(Serialize)]
struct FallbackExecEnvelope {
    schema: &'static str,
    map: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct FallbackMaegimEnvelope {
    schema: &'static str,
    controls: Vec<FallbackMaegimControl>,
}

#[derive(Serialize)]
struct FallbackMaegimControl {
    name: String,
    decl_kind: &'static str,
    type_name: String,
    init_expr_canon: Option<String>,
    fields: Vec<MaegimControlField>,
    range: Option<MaegimControlRange>,
    min_expr_canon: Option<String>,
    max_expr_canon: Option<String>,
    inclusive_end: bool,
    step_expr_canon: Option<String>,
    split_count_expr_canon: Option<String>,
}

fn build_frontdoor_fallback_plans(source: &str, maegim_source: &str) -> Result<FrontdoorFallbackPlans, CanonError> {
    let guseong_flat_json = build_frontdoor_fallback_guseong_json(source)?;
    let alrim_plan_json = build_frontdoor_fallback_alrim_json()?;
    let exec_policy_map_json = build_frontdoor_fallback_exec_json()?;
    // A normalized fallback can shorten decimal literals. Source editing and
    // control defaults must remain bound to the original, verified literal.
    let maegim_control_json = build_frontdoor_fallback_maegim_json(maegim_source)?;
    Ok(FrontdoorFallbackPlans {
        guseong_flat_json,
        alrim_plan_json,
        exec_policy_map_json,
        maegim_control_json,
    })
}

fn build_frontdoor_fallback_guseong_json(source: &str) -> Result<String, CanonError> {
    let mut uniq = BTreeSet::new();
    let mut groups = Vec::new();
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if !line.starts_with("무리 <- ") {
            continue;
        }
        let Some(start) = line.find('"') else {
            continue;
        };
        let remain = &line[start + 1..];
        let Some(end) = remain.find('"') else {
            continue;
        };
        let id = remain[..end].trim();
        if id.is_empty() || !uniq.insert(id.to_string()) {
            continue;
        }
        groups.push(FallbackGuseongGroup { id: id.to_string() });
    }
    let envelope = FallbackGuseongEnvelope {
        schema: "ddn.guseong_flatten_plan.v1",
        groups,
    };
    serde_json::to_string_pretty(&envelope)
        .map(|text| format!("{}\n", text))
        .map_err(|err| {
            CanonError::new(
                "E_CANON_GUSEONG_FLAT_JSON",
                format!("frontdoor fallback guseong JSON 직렬화 실패: {}", err),
            )
        })
}

fn build_frontdoor_fallback_alrim_json() -> Result<String, CanonError> {
    let envelope = FallbackAlrimEnvelope {
        schema: "ddn.alrim_plan.v1",
        plans: Vec::new(),
    };
    serde_json::to_string_pretty(&envelope)
        .map(|text| format!("{}\n", text))
        .map_err(|err| {
            CanonError::new(
                "E_CANON_ALRIM_PLAN_JSON",
                format!("frontdoor fallback alrim JSON 직렬화 실패: {}", err),
            )
        })
}

fn build_frontdoor_fallback_exec_json() -> Result<String, CanonError> {
    let envelope = FallbackExecEnvelope {
        schema: "ddn.exec_policy_effect_map.v1",
        map: BTreeMap::new(),
    };
    serde_json::to_string_pretty(&envelope)
        .map(|text| format!("{}\n", text))
        .map_err(|err| {
            CanonError::new(
                "E_CANON_EXEC_POLICY_MAP_JSON",
                format!("frontdoor fallback exec-policy JSON 직렬화 실패: {}", err),
            )
        })
}

fn build_frontdoor_fallback_maegim_json(source: &str) -> Result<String, CanonError> {
    let mut controls = Vec::new();
    let mut offset = 0usize;
    for raw_line in source.split_inclusive('\n') {
        let line_start = offset;
        offset += raw_line.len();
        let line = raw_line.trim();
        if !line.contains("매김 {") {
            continue;
        }

        let Some(colon_pos) = line.find(':') else {
            continue;
        };
        let name = line[..colon_pos].trim();
        if name.is_empty() {
            continue;
        }
        // Keep existing generic fallback controls, but leave an ambiguous or
        // nonliteral initial value unavailable rather than fabricating one.
        let init_expr_canon = maegim_control_literal_locator(source, name)?
            .map(|(_, _, _, _, literal)| literal);
        // F5-GA[CANON_LOCATOR] trim retained the verified literal substring, so raw_line.find must succeed; split_inclusive offsets and literal endpoints remain source UTF-8 boundaries.
        let body_start = line_start + raw_line.find("매김 {").unwrap() + "매김 {".len();
        let body_tokens = ddonirang_lang::Lexer::new(&source[body_start..])
            .tokenize()
            .map_err(|err| CanonError::new("E_CANON_MAEGIM_CONTROL_PLAN_JSON", err.to_string()))?;
        let mut depth = 1usize;
        let mut body_end = None;
        for token in &body_tokens {
            match token.kind {
                ddonirang_lang::TokenKind::LBrace => depth += 1,
                ddonirang_lang::TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        body_end = Some(token.span.start);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(body_end) = body_end else { continue; };
        let body = &source[body_start..body_start + body_end];
        let fields = fallback_maegim_control_fields(body)?;
        let type_header = &line[colon_pos + 1..];
        let type_end = ["<-", "←", ":="]
            .iter()
            .filter_map(|operator| type_header.find(operator))
            .min();
        let Some(type_end) = type_end else { continue; };
        let type_name = type_header[..type_end].trim().to_string();

        let mut min_expr_canon = None;
        let mut max_expr_canon = None;
        let mut inclusive_end = false;
        let mut step_expr_canon = None;
        let mut split_count_expr_canon = None;

        for field in &fields {
            if field.name == "범위" {
                if let Some((left, right, incl)) = parse_fallback_range(&field.value_canon) {
                    min_expr_canon = Some(left);
                    max_expr_canon = Some(right);
                    inclusive_end = incl;
                }
            } else if field.name == "간격" {
                step_expr_canon = Some(field.value_canon.clone());
            } else if field.name == "분할수" {
                split_count_expr_canon = Some(field.value_canon.clone());
            }
        }
        let range = min_expr_canon.clone().zip(max_expr_canon.clone())
            .map(|(min_expr_canon, max_expr_canon)| MaegimControlRange {
                min_expr_canon, max_expr_canon, inclusive_end,
            });

        controls.push(FallbackMaegimControl {
            name: name.to_string(),
            decl_kind: "gureut",
            type_name,
            init_expr_canon,
            fields,
            range,
            min_expr_canon,
            max_expr_canon,
            inclusive_end,
            step_expr_canon,
            split_count_expr_canon,
        });
    }

    let envelope = FallbackMaegimEnvelope {
        schema: "ddn.maegim_control_plan.v1",
        controls,
    };
    serde_json::to_string_pretty(&envelope)
        .map(|text| format!("{}\n", text))
        .map_err(|err| {
            CanonError::new(
                "E_CANON_MAEGIM_CONTROL_PLAN_JSON",
                format!("frontdoor fallback maegim JSON 직렬화 실패: {}", err),
            )
        })
}

fn fallback_maegim_control_fields(body: &str) -> Result<Vec<MaegimControlField>, CanonError> {
    // F5-GA[CANON_BODY_TOKENS] index+1<len guards the pair; end<len guards the stop token. The private Lexer reads this same body and emits ordered UTF-8 byte spans, so colon.end..dot.start is a bounded source slice.
    use ddonirang_lang::TokenKind as LangTokenKind;
    let tokens = ddonirang_lang::Lexer::new(body).tokenize()
        .map_err(|err| CanonError::new("E_CANON_MAEGIM_CONTROL_PLAN_JSON", err.to_string()))?;
    let mut fields = Vec::new();
    let mut index = 0usize;
    while index + 1 < tokens.len() {
        if let (LangTokenKind::Ident(name), LangTokenKind::Colon) =
            (&tokens[index].kind, &tokens[index + 1].kind) {
            let start = tokens[index + 1].span.end;
            let mut end = index + 2;
            while end < tokens.len() && !matches!(tokens[end].kind, LangTokenKind::Dot) {
                end += 1;
            }
            if end < tokens.len() {
                fields.push(MaegimControlField {
                    name: name.clone(),
                    value_canon: body[start..tokens[end].span.start].trim().to_string(),
                });
                index = end + 1;
                continue;
            }
        }
        index += 1;
    }
    Ok(fields)
}

fn parse_fallback_range(input: &str) -> Option<(String, String, bool)> {
    if let Some((left, right)) = input.split_once("..=") {
        return Some((left.trim().to_string(), right.trim().to_string(), true));
    }
    if let Some((left, right)) = input.split_once("..") {
        return Some((left.trim().to_string(), right.trim().to_string(), false));
    }
    None
}

#[derive(Debug, Default, Serialize)]
struct GuseongFlattenPlan {
    topo_order: Vec<String>,
    instances: Vec<GuseongFlattenInstance>,
    links: Vec<GuseongFlattenLink>,
}

#[derive(Debug, Clone, Serialize)]
struct GuseongFlattenInstance {
    name: String,
    type_name: String,
}

#[derive(Debug, Clone, Serialize)]
struct GuseongFlattenLink {
    dst_instance: String,
    dst_port: String,
    src_instance: String,
    src_port: String,
}

#[derive(Debug, Serialize)]
struct GuseongFlattenPlanEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    plan: &'a GuseongFlattenPlan,
}

impl GuseongFlattenPlan {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = GuseongFlattenPlanEnvelope {
            schema: "ddn.guseong_flatten_plan.v1",
            plan: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_GUSEONG_FLATTEN_JSON",
                    format!("짜임 flatten JSON 직렬화 실패: {}", err),
                )
            })
    }
}

#[derive(Debug, Default, Serialize)]
struct AlrimEventPlan {
    handlers: Vec<AlrimEventHandler>,
}

#[derive(Debug, Clone, Serialize)]
struct AlrimEventHandler {
    order: u64,
    kind: String,
    scope: String,
    body_canon: String,
}

#[derive(Debug, Serialize)]
struct AlrimEventPlanEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    plan: &'a AlrimEventPlan,
}

impl AlrimEventPlan {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = AlrimEventPlanEnvelope {
            schema: "ddn.alrim_event_plan.v1",
            plan: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_ALRIM_EVENT_JSON",
                    format!("알림 이벤트 계획 JSON 직렬화 실패: {}", err),
                )
            })
    }
}

#[derive(Debug, Default, Serialize)]
struct MaegimControlPlan {
    controls: Vec<MaegimControlItem>,
}

#[derive(Debug, Serialize)]
struct MaegimControlPlanEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    plan: &'a MaegimControlPlan,
}

#[derive(Debug, Serialize)]
struct MaegimControlItem {
    name: String,
    decl_kind: String,
    type_name: String,
    init_expr_canon: String,
    fields: Vec<MaegimControlField>,
    range: Option<MaegimControlRange>,
    step_expr_canon: Option<String>,
    split_count_expr_canon: Option<String>,
}

#[derive(Debug, Default, Serialize)]
struct BlockEditorPlan {
    blocks: Vec<BlockEditorPlanNode>,
}

#[derive(Debug, Serialize)]
struct BlockEditorPlanEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    plan: &'a BlockEditorPlan,
}

impl BlockEditorPlan {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = BlockEditorPlanEnvelope {
            schema: "ddn.block_editor_plan.v1",
            plan: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_BLOCK_EDITOR_PLAN_JSON",
                    format!("block editor plan JSON 직렬화 실패: {}", err),
                )
            })
    }
}

#[derive(Debug, Default, Serialize)]
struct BlockEditorPlanNode {
    kind: String,
    fields: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    exprs: BTreeMap<String, BlockEditorExprNode>,
    inputs: BTreeMap<String, Vec<BlockEditorPlanNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_text: Option<String>,
}

#[derive(Debug, Serialize)]
struct BlockEditorExprNode {
    kind: String,
    text: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    fields: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    inputs: BTreeMap<String, Vec<BlockEditorExprNode>>,
}

mod block_editor;
use block_editor::*;

#[derive(Debug, Serialize)]
struct MaegimControlField {
    name: String,
    value_canon: String,
}

#[derive(Debug, Serialize)]
struct MaegimControlRange {
    min_expr_canon: String,
    max_expr_canon: String,
    inclusive_end: bool,
}

impl MaegimControlPlan {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = MaegimControlPlanEnvelope {
            schema: "ddn.maegim_control_plan.v1",
            plan: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_MAEGIM_CONTROL_JSON",
                    format!("매김 제어 계획 JSON 직렬화 실패: {}", err),
                )
            })
    }
}

#[derive(Debug, Serialize)]
struct ExecPolicyEffectMap {
    language_axis: ExecPolicyLanguageAxis,
    tool_axis: ExecPolicyToolAxis,
    selected_behavior_by_open_mode: Vec<ExecPolicyOpenModeBehavior>,
}

#[derive(Debug, Serialize)]
struct ExecPolicyLanguageAxis {
    block_count: u64,
    exec_mode_raw: Option<String>,
    effect_policy_raw: Option<String>,
    exec_mode_effective: String,
    effect_policy_effective: String,
    effect_policy_explicit: bool,
    strict_effect_ignored: bool,
    would_fail_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExecPolicyToolAxis {
    open_mode_values: Vec<String>,
    effective_open_mode_resolution_order: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ExecPolicyOpenModeBehavior {
    open_mode: String,
    effect_boundary_result: String,
}

#[derive(Debug, Serialize)]
struct ExecPolicyEffectMapEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    map: &'a ExecPolicyEffectMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecMode {
    Strict,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectPolicy {
    Isolated,
    Allowed,
}

#[derive(Debug, Default)]
struct ExecPolicyFields {
    exec_mode_raw: Option<String>,
    effect_policy_raw: Option<String>,
}

impl ExecPolicyEffectMap {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = ExecPolicyEffectMapEnvelope {
            schema: "ddn.exec_policy_effect_map.v1",
            map: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_EXEC_POLICY_MAP_JSON",
                    format!("실행정책/효과 경계 매핑 JSON 직렬화 실패: {}", err),
                )
            })
    }
}

fn parse_exec_policy_fields(body: &str) -> ExecPolicyFields {
    let mut fields = ExecPolicyFields::default();
    for raw_line in body.lines() {
        let mut line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if let Some(stripped) = line.strip_suffix('.') {
            line = stripped.trim_end();
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().to_string();
        match key {
            "실행모드" => fields.exec_mode_raw = Some(value),
            "효과정책" => fields.effect_policy_raw = Some(value),
            _ => {}
        }
    }
    fields
}

fn parse_exec_mode_value(text: &str) -> Option<ExecMode> {
    match text.trim() {
        "엄밀" | "strict" | "STRICT" => Some(ExecMode::Strict),
        "일반" | "general" | "GENERAL" => Some(ExecMode::General),
        _ => None,
    }
}

fn parse_effect_policy_value(text: &str) -> Option<EffectPolicy> {
    match text.trim() {
        "격리" | "isolate" | "isolated" | "ISOLATE" | "ISOLATED" => Some(EffectPolicy::Isolated),
        "허용" | "allow" | "ALLOW" | "allowed" | "ALLOWED" => Some(EffectPolicy::Allowed),
        _ => None,
    }
}

fn exec_mode_label(mode: ExecMode) -> String {
    match mode {
        ExecMode::Strict => "엄밀".to_string(),
        ExecMode::General => "일반".to_string(),
    }
}

#[cfg(test)]
mod tests;

fn effect_policy_label(policy: EffectPolicy) -> String {
    match policy {
        EffectPolicy::Isolated => "격리".to_string(),
        EffectPolicy::Allowed => "허용".to_string(),
    }
}

fn build_maegim_control_plan(stmts: &[Stmt]) -> MaegimControlPlan {
    let mut controls = Vec::new();
    collect_maegim_controls_from_stmts(stmts, &mut controls);
    MaegimControlPlan { controls }
}

fn collect_maegim_controls_from_stmts(stmts: &[Stmt], controls: &mut Vec<MaegimControlItem>) {
    for stmt in stmts {
        match stmt {
            Stmt::RootDecl { items } => collect_maegim_controls_from_decl_items(items, controls),
            Stmt::SeedDef { body, .. }
            | Stmt::Repeat { body }
            | Stmt::While { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::Hook { body, .. }
            | Stmt::Receive { body, .. }
            | Stmt::EventReact { body, .. }
            | Stmt::OpenBlock { body }
            | Stmt::PromptAfter { body, .. }
            | Stmt::PromptCondition { body, .. }
            | Stmt::PromptBlock { body } => collect_maegim_controls_from_stmts(body, controls),
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_maegim_controls_from_stmts(then_body, controls);
                if let Some(else_body) = else_body {
                    collect_maegim_controls_from_stmts(else_body, controls);
                }
            }
            Stmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                if let Some(then_body) = then_body {
                    collect_maegim_controls_from_stmts(then_body, controls);
                }
                collect_maegim_controls_from_stmts(else_body, controls);
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_maegim_controls_from_stmts(&branch.body, controls);
                }
                if let Some(else_body) = else_body {
                    collect_maegim_controls_from_stmts(else_body, controls);
                }
            }
            Stmt::PromptChoose {
                branches,
                else_body,
            } => {
                for branch in branches {
                    collect_maegim_controls_from_stmts(&branch.body, controls);
                }
                if let Some(else_body) = else_body {
                    collect_maegim_controls_from_stmts(else_body, controls);
                }
            }
            _ => {}
        }
    }
}

fn collect_maegim_controls_from_decl_items(
    items: &[DeclItem],
    controls: &mut Vec<MaegimControlItem>,
) {
    for item in items {
        let Some(maegim) = &item.maegim else {
            continue;
        };
        let Some(value) = &item.value else {
            continue;
        };
        let fields = maegim
            .fields
            .iter()
            .map(|binding| MaegimControlField {
                name: binding.name.clone(),
                value_canon: format_maegim_field_value(binding),
            })
            .collect();
        let range = maegim
            .fields
            .iter()
            .find(|binding| maegim_field_leaf_name(&binding.name) == "범위")
            .and_then(|binding| parse_maegim_range_expr(&binding.value))
            .map(
                |(min_expr_canon, max_expr_canon, inclusive_end)| MaegimControlRange {
                    min_expr_canon,
                    max_expr_canon,
                    inclusive_end,
                },
            );
        let step_expr_canon = maegim
            .fields
            .iter()
            .find(|binding| maegim_field_leaf_name(&binding.name) == "간격")
            .map(|binding| format_expr(&binding.value));
        let split_count_expr_canon = maegim
            .fields
            .iter()
            .find(|binding| maegim_field_leaf_name(&binding.name) == "분할수")
            .map(|binding| format_expr(&binding.value));
        controls.push(MaegimControlItem {
            name: item.name.clone(),
            decl_kind: match item.kind {
                DeclKind::Gureut => "gureut".to_string(),
                DeclKind::Butbak => "butbak".to_string(),
            },
            type_name: item.type_name.clone(),
            init_expr_canon: format_expr(value),
            fields,
            range,
            step_expr_canon,
            split_count_expr_canon,
        });
    }
}

fn parse_maegim_range_expr(expr: &Expr) -> Option<(String, String, bool)> {
    // F5-GA[CANON_RANGE_ARGS] The exact three-argument guard precedes all accesses0..2 on this immutably borrowed call.
    let Expr::Call { name, args, .. } = expr else {
        return None;
    };
    if name != "표준.범위" || args.len() != 3 {
        return None;
    }
    let inclusive_end = matches!(&args[2], Expr::Literal(Literal::Num(value)) if value == "1");
    Some((format_expr(&args[0]), format_expr(&args[1]), inclusive_end))
}

fn build_exec_policy_effect_map(stmts: &[Stmt]) -> ExecPolicyEffectMap {
    let blocks: Vec<&str> = stmts
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::ExecPolicyBlock { body } => Some(body.as_str()),
            _ => None,
        })
        .collect();
    let block_count = blocks.len() as u64;
    let first_fields = blocks
        .first()
        .map(|body| parse_exec_policy_fields(body))
        .unwrap_or_default();

    let mode_parse = first_fields
        .exec_mode_raw
        .as_ref()
        .map(|raw| parse_exec_mode_value(raw));
    let effect_parse = first_fields
        .effect_policy_raw
        .as_ref()
        .map(|raw| parse_effect_policy_value(raw));

    let mut would_fail_code: Option<String> = None;
    if matches!(mode_parse, Some(None)) || matches!(effect_parse, Some(None)) {
        would_fail_code = Some("E_EXEC_ENUM_INVALID".to_string());
    } else if block_count > 1 {
        would_fail_code = Some("E_EXEC_POLICY_DUPLICATE".to_string());
    }

    let mode = mode_parse.flatten().unwrap_or(ExecMode::Strict);
    let effect_explicit = effect_parse.flatten();
    let mut effect = effect_explicit.unwrap_or(EffectPolicy::Isolated);
    let strict_effect_ignored =
        mode == ExecMode::Strict && matches!(effect_explicit, Some(EffectPolicy::Allowed));
    if mode == ExecMode::Strict {
        effect = EffectPolicy::Isolated;
    }

    let language_axis = ExecPolicyLanguageAxis {
        block_count,
        exec_mode_raw: first_fields.exec_mode_raw,
        effect_policy_raw: first_fields.effect_policy_raw,
        exec_mode_effective: exec_mode_label(mode),
        effect_policy_effective: effect_policy_label(effect),
        effect_policy_explicit: effect_explicit.is_some(),
        strict_effect_ignored,
        would_fail_code: would_fail_code.clone(),
    };

    let open_modes = ["deny", "record", "replay"];
    let selected_behavior_by_open_mode = open_modes
        .iter()
        .map(|open_mode| {
            let effect_boundary_result = if let Some(code) = &would_fail_code {
                format!("gate_error:{}", code)
            } else if mode == ExecMode::Strict {
                "compile_error:E_EFFECT_IN_STRICT_MODE".to_string()
            } else if effect == EffectPolicy::Isolated {
                "runtime_error:E_EFFECT_IN_ISOLATED_MODE".to_string()
            } else {
                match *open_mode {
                    "deny" => "runtime_error:E_OPEN_DENIED".to_string(),
                    "record" => "open_log:record".to_string(),
                    "replay" => "open_log:replay".to_string(),
                    _ => "unknown".to_string(),
                }
            };
            ExecPolicyOpenModeBehavior {
                open_mode: (*open_mode).to_string(),
                effect_boundary_result,
            }
        })
        .collect();

    ExecPolicyEffectMap {
        language_axis,
        tool_axis: ExecPolicyToolAxis {
            open_mode_values: vec![
                "deny".to_string(),
                "record".to_string(),
                "replay".to_string(),
            ],
            effective_open_mode_resolution_order: vec![
                "cli".to_string(),
                "open_policy".to_string(),
                "default_deny".to_string(),
            ],
        },
        selected_behavior_by_open_mode,
    }
}

fn build_alrim_event_plan(stmts: &[Stmt]) -> AlrimEventPlan {
    let mut handlers = Vec::new();
    collect_alrim_event_handlers(stmts, "root", &mut handlers);
    for (idx, handler) in handlers.iter_mut().enumerate() {
        handler.order = idx as u64;
    }
    AlrimEventPlan { handlers }
}

fn collect_alrim_event_handlers(stmts: &[Stmt], scope: &str, out: &mut Vec<AlrimEventHandler>) {
    for stmt in stmts {
        match stmt {
            Stmt::Receive { kind, body, .. } => {
                let handler_kind = kind.clone().unwrap_or_else(|| "알림".to_string());
                out.push(AlrimEventHandler {
                    order: 0,
                    kind: handler_kind.clone(),
                    scope: scope.to_string(),
                    body_canon: format_program(body),
                });
                let nested_scope = format!("{}/receive:{}", scope, handler_kind);
                collect_alrim_event_handlers(body, &nested_scope, out);
            }
            Stmt::EventReact { kind, body } => {
                out.push(AlrimEventHandler {
                    order: 0,
                    kind: kind.clone(),
                    scope: scope.to_string(),
                    body_canon: format_program(body),
                });
                let nested_scope = format!("{}/event:{}", scope, kind);
                collect_alrim_event_handlers(body, &nested_scope, out);
            }
            Stmt::SeedDef { name, body, .. } => {
                let next = format!("{}/seed:{}", scope, name);
                collect_alrim_event_handlers(body, &next, out);
            }
            Stmt::Hook { name, suffix, body } => {
                let suffix_text = match suffix {
                    HookSuffix::Halttae => "할때",
                    HookSuffix::Mada => "마다",
                };
                let next = format!("{}/hook:{}:{}", scope, name, suffix_text);
                collect_alrim_event_handlers(body, &next, out);
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_alrim_event_handlers(then_body, scope, out);
                if let Some(else_body) = else_body {
                    collect_alrim_event_handlers(else_body, scope, out);
                }
            }
            Stmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                if let Some(then_body) = then_body {
                    collect_alrim_event_handlers(then_body, scope, out);
                }
                collect_alrim_event_handlers(else_body, scope, out);
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_alrim_event_handlers(&branch.body, scope, out);
                }
                if let Some(else_body) = else_body {
                    collect_alrim_event_handlers(else_body, scope, out);
                }
            }
            Stmt::PromptChoose {
                branches,
                else_body: Some(else_body),
            } => {
                for branch in branches {
                    collect_alrim_event_handlers(&branch.body, scope, out);
                }
                collect_alrim_event_handlers(else_body, scope, out);
            }
            Stmt::PromptChoose {
                branches,
                else_body: None,
            } => {
                for branch in branches {
                    collect_alrim_event_handlers(&branch.body, scope, out);
                }
            }
            Stmt::PromptAfter { body, .. }
            | Stmt::PromptCondition { body, .. }
            | Stmt::PromptBlock { body }
            | Stmt::Repeat { body }
            | Stmt::While { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::OpenBlock { body } => {
                collect_alrim_event_handlers(body, scope, out);
            }
            Stmt::RootDecl { .. }
            | Stmt::Assign { .. }
            | Stmt::Show { .. }
            | Stmt::Inspect { .. }
            | Stmt::Boim { .. }
            | Stmt::Send { .. }
            | Stmt::BogaeMadangBlock { .. }
            | Stmt::ExecPolicyBlock { .. }
            | Stmt::JjaimBlock { .. }
            | Stmt::Return { .. }
            | Stmt::Expr { .. }
            | Stmt::BogaeDraw
            | Stmt::Break
            | Stmt::ContinueLoop => {}
        }
    }
}

mod port_graph;
use port_graph::{build_guseong_flatten_plan_surface};

fn is_event_noun_canonical(text: &str) -> bool {
    matches!(text, "알림" | "알림이" | "알림가")
}

fn is_event_noun_alias(text: &str) -> bool {
    matches!(
        text,
        "소식" | "소식이" | "소식가" | "알람" | "알람이" | "알람가"
    )
}

fn is_event_noun_any(text: &str) -> bool {
    is_event_noun_canonical(text) || is_event_noun_alias(text)
}

fn strip_receive_object_particle(text: &str) -> Option<String> {
    text.strip_suffix('를')
        .or_else(|| text.strip_suffix('을'))
        .map(|raw| raw.to_string())
}

fn object_particle(text: &str) -> &'static str {
    let Some(last) = text.chars().last() else {
        return "를";
    };
    if !('가'..='힣').contains(&last) {
        return "를";
    }
    let code = last as u32 - '가' as u32;
    if code % 28 == 0 {
        "를"
    } else {
        "을"
    }
}

fn is_open_block_keyword(name: &str) -> bool {
    matches!(name, "열림" | "너머" | "효과" | "바깥")
}

fn flatten_field_access(target: &Expr, field: &str) -> Option<Path> {
    let mut segments = match target {
        Expr::Path(path) => path.segments.clone(),
        Expr::FieldAccess {
            target: inner,
            field: inner_field,
        } => {
            let inner_path = flatten_field_access(inner, inner_field)?;
            inner_path.segments
        }
        _ => return None,
    };
    segments.push(field.to_string());
    Some(Path { segments })
}

fn normalize_root_segments(segments: &[String]) -> Vec<String> {
    // F5-GA[CANON_SEGMENTS] The len>=2 short-circuit bounds index0 and the subsequent suffix slice[1..].
    if segments.len() >= 2 && is_root_segment(&segments[0]) {
        return segments[1..].to_vec();
    }
    segments.to_vec()
}

fn is_root_segment(name: &str) -> bool {
    matches!(name, "바탕" | "살림" | "샘")
}

fn topo_sort_or_cycle(
    edges: &HashMap<String, HashSet<String>>,
    nodes: &BTreeSet<String>,
) -> Result<Vec<String>, CanonError> {
    let mut indegree: HashMap<String, usize> = nodes.iter().map(|n| (n.clone(), 0usize)).collect();
    for tos in edges.values() {
        for to in tos {
            *indegree.entry(to.clone()).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<String> = indegree
        .iter()
        .filter_map(|(name, deg)| (*deg == 0).then(|| name.clone()))
        .collect();
    let mut queue_vec: Vec<String> = queue.drain(..).collect();
    queue_vec.sort();
    queue = queue_vec.into_iter().collect();

    let mut out = Vec::new();
    let mut indegree_mut = indegree;
    while let Some(node) = queue.pop_front() {
        out.push(node.clone());
        if let Some(tos) = edges.get(&node) {
            let mut sorted_tos: Vec<String> = tos.iter().cloned().collect();
            sorted_tos.sort();
            for to in sorted_tos {
                if let Some(deg) = indegree_mut.get_mut(&to) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push_back(to);
                    }
                }
            }
        }
    }

    if out.len() == indegree_mut.len() {
        return Ok(out);
    }

    let cycle_nodes = detect_cycle_nodes(edges, nodes);
    let cycle_text = if cycle_nodes.is_empty() {
        "알 수 없는 순환".to_string()
    } else {
        cycle_nodes.join(" -> ")
    };
    Err(CanonError::new(
        "E_GUSEONG_LINK_CYCLE",
        format!("짜임 연결 순환: {}", cycle_text),
    ))
}

fn detect_cycle_nodes(
    edges: &HashMap<String, HashSet<String>>,
    nodes: &BTreeSet<String>,
) -> Vec<String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Temp,
        Perm,
    }
    fn dfs(
        node: &str,
        edges: &HashMap<String, HashSet<String>>,
        marks: &mut HashMap<String, Mark>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        marks.insert(node.to_string(), Mark::Temp);
        stack.push(node.to_string());
        if let Some(tos) = edges.get(node) {
            let mut nexts: Vec<String> = tos.iter().cloned().collect();
            nexts.sort();
            for to in nexts {
                match marks.get(&to).copied() {
                    Some(Mark::Temp) => {
                        if let Some(pos) = stack.iter().position(|s| s == &to) {
                            let mut cycle = stack[pos..].to_vec();
                            cycle.push(to);
                            return Some(cycle);
                        }
                    }
                    Some(Mark::Perm) => {}
                    None => {
                        if let Some(cycle) = dfs(&to, edges, marks, stack) {
                            return Some(cycle);
                        }
                    }
                }
            }
        }
        stack.pop();
        marks.insert(node.to_string(), Mark::Perm);
        None
    }

    let mut marks: HashMap<String, Mark> = HashMap::new();
    let mut stack = Vec::new();
    for node in nodes {
        if marks.contains_key(node) {
            continue;
        }
        if let Some(cycle) = dfs(node, edges, &mut marks, &mut stack) {
            return cycle;
        }
    }
    Vec::new()
}

mod expressions;
use expressions::*;

mod formatting;
use formatting::*;

