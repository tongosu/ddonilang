use std::collections::{BTreeMap, BTreeSet};

use ddonirang_lang::{
    CanonProgram, Expr, ExprKind, Lexer, Literal, ParseMode, Stmt, TokenKind, TopLevelItem, TypeRef,
};
use serde::{Deserialize, Serialize};

use crate::ddn_runtime::{
    resolve_supported_source_profile, DdnParseWarning, DdnProgram,
    V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
};

pub const SOURCE_ANALYSIS_SNAPSHOT_SCHEMA: &str = "ddn.source_analysis_snapshot.candidate.v1";
pub const STRICT_SOURCE_PROFILE_IDENTITY: &str = "strict";
pub const REPAIR_PROPOSAL_SCHEMA: &str = "ddn.source_analysis_repair_proposal.candidate.v1";
pub const REMOVE_SETUP_COLON_FIX_ID: &str = "FIX_SOURCE_REMOVE_SETUP_COLON";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StaticType {
    Number,
    Text,
    Bool,
    Collection,
    None,
    Unknown,
}

impl StaticType {
    fn label(self) -> &'static str {
        match self {
            Self::Number => "수",
            Self::Text => "글",
            Self::Bool => "참거짓",
            Self::Collection => "차림",
            Self::None => "없음",
            Self::Unknown => "알수없음",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StaticDiagnostic {
    code: &'static str,
    message: String,
    start: usize,
    end: usize,
}

fn type_ref_kind(type_ref: &TypeRef) -> StaticType {
    match type_ref {
        TypeRef::Named(name) => match name.as_str() {
            "수" | "셈수" | "바른수" | "큰바른수" | "나눔수" => StaticType::Number,
            "글" | "이름씨" => StaticType::Text,
            "참거짓" | "논" => StaticType::Bool,
            "없음" => StaticType::None,
            _ => StaticType::Unknown,
        },
        TypeRef::Applied { name, .. } if name == "차림" => StaticType::Collection,
        _ => StaticType::Unknown,
    }
}

fn static_error(code: &'static str, message: impl Into<String>, expr: &Expr) -> StaticDiagnostic {
    StaticDiagnostic {
        code,
        message: message.into(),
        start: expr.span.start,
        end: expr.span.end,
    }
}

fn check_logical_operand(
    operator: &str,
    kind: StaticType,
    expr: &Expr,
) -> Result<(), StaticDiagnostic> {
    if matches!(kind, StaticType::Bool | StaticType::Unknown) {
        return Ok(());
    }
    Err(static_error(
        "E_CHECK_TYPE_MISMATCH",
        format!(
            "{operator}의 피연산자는 참거짓이어야 합니다: {}",
            kind.label()
        ),
        expr,
    ))
}

fn infer_expr(
    expr: &Expr,
    symbols: &BTreeMap<String, StaticType>,
    call_results: &BTreeMap<String, StaticType>,
    unknown_names_allowed: bool,
) -> Result<StaticType, StaticDiagnostic> {
    match &expr.kind {
        ExprKind::Literal(value) => Ok(match value {
            Literal::Int(_) | Literal::Fixed64(_) => StaticType::Number,
            Literal::String(_) | Literal::Atom(_) => StaticType::Text,
            Literal::Bool(_) => StaticType::Bool,
            Literal::None => StaticType::None,
            Literal::Regex(_) | Literal::Resource(_) => StaticType::Unknown,
        }),
        ExprKind::Var(name) => match name.as_str() {
            "참" | "거짓" => Ok(StaticType::Bool),
            "없음" => Ok(StaticType::None),
            _ => symbols
                .get(name)
                .copied()
                .map(|kind| if unknown_names_allowed && kind == StaticType::None { StaticType::Unknown } else { kind })
                .or_else(|| unknown_names_allowed.then_some(StaticType::Unknown))
                .ok_or_else(|| {
                    static_error(
                        "E_CHECK_UNKNOWN_NAME",
                        format!("정의되지 않은 이름입니다: {name}"),
                        expr,
                    )
                }),
        },
        ExprKind::Infix { left, op, right } => {
            let left_type = infer_expr(left, symbols, call_results, unknown_names_allowed)?;
            let right_type = infer_expr(right, symbols, call_results, unknown_names_allowed)?;
            if matches!(op.as_str(), "그리고" | "또는" | "&&" | "||") {
                // 실행의 단락 평가와 달리 정적 검사는 양쪽 식을 모두 검사한다.
                check_logical_operand(op, left_type, left)?;
                check_logical_operand(op, right_type, right)?;
                return Ok(StaticType::Bool);
            }
            if matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
                return Ok(StaticType::Bool);
            }
            if op == "+" && left_type == StaticType::Text && right_type == StaticType::Text {
                return Ok(StaticType::Text);
            }
            if matches!(op.as_str(), "+" | "-" | "*" | "/") {
                if left_type == StaticType::Number && right_type == StaticType::Number {
                    return Ok(StaticType::Number);
                }
                if left_type != StaticType::Unknown && right_type != StaticType::Unknown {
                    return Err(static_error(
                        "E_CHECK_TYPE_MISMATCH",
                        format!(
                            "연산자 {op}의 피연산자 형식이 맞지 않습니다: {} {op} {}",
                            left_type.label(),
                            right_type.label()
                        ),
                        expr,
                    ));
                }
            }
            Ok(StaticType::Unknown)
        }
        ExprKind::Call { args, func } => {
            for (index,arg) in args.iter().enumerate() {
                // RD-520: the recipe pin refers to an authored named callable,
                // not a world variable. Only this exact pin admits a definition
                // reference; its type and capture-free seal are checked by the
                // shared runtime. Ordinary unknown variable errors stay intact.
                if func=="표준보개.돋" && (arg.resolved_pin.as_deref()==Some("보기법")
                    || (arg.resolved_pin.is_none() && index==1)) {
                    if let ExprKind::Var(name)=&arg.expr.kind {
                        if !symbols.contains_key(name) && call_results.contains_key(name) {continue;}
                    }
                }
                let kind = infer_expr(&arg.expr, symbols, call_results, unknown_names_allowed)?;
                if matches!(func.as_str(), "그리고" | "또는") {
                    check_logical_operand(func, kind, &arg.expr)?;
                }
            }
            if matches!(func.as_str(), "그리고" | "또는") {
                return Ok(StaticType::Bool);
            }
            Ok(call_results
                .get(func)
                .copied()
                .unwrap_or(StaticType::Unknown))
        }
        ExprKind::CollectionConstructor { elements } => {
            for element in elements {
                infer_expr(element, symbols, call_results, unknown_names_allowed)?;
            }
            Ok(StaticType::Collection)
        }
        ExprKind::SelectionPlace { target, index } => {
            infer_expr(target, symbols, call_results, unknown_names_allowed)?;
            let index_type = infer_expr(index, symbols, call_results, unknown_names_allowed)?;
            if index_type != StaticType::Number && index_type != StaticType::Unknown {
                return Err(static_error(
                    "E_CHECK_TYPE_MISMATCH",
                    format!("차림 위치는 수여야 합니다: {}", index_type.label()),
                    index,
                ));
            }
            Ok(StaticType::Unknown)
        }
        ExprKind::FieldAccess { target, .. } => {
            infer_expr(target, symbols, call_results, unknown_names_allowed)?;
            Ok(StaticType::Unknown)
        }
        ExprKind::Suffix { value, .. } | ExprKind::Nuance { expr: value, .. } => {
            infer_expr(value, symbols, call_results, unknown_names_allowed)
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                infer_expr(value, symbols, call_results, unknown_names_allowed)?;
            }
            Ok(StaticType::Unknown)
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                infer_expr(value, symbols, call_results, unknown_names_allowed)?;
            }
            Ok(StaticType::Unknown)
        }
        ExprKind::SeedLiteral { param, body } => {
            let mut local = symbols.clone();
            local.insert(param.clone(), StaticType::Unknown);
            infer_expr(body, &local, call_results, true)?;
            Ok(StaticType::Unknown)
        }
        ExprKind::Thunk(body) => {
            validate_root_statements(&body.stmts, &mut symbols.clone(), call_results, true)?;
            Ok(StaticType::Unknown)
        }
        ExprKind::Eval { thunk, mode } => {
            infer_expr(thunk, symbols, call_results, unknown_names_allowed)?;
            // D6's braced condition is represented as a Bool-evaluated thunk.
            // Preserve the existing static rejection of known non-Bool values.
            if matches!(mode, ddonirang_lang::ThunkEvalMode::Bool | ddonirang_lang::ThunkEvalMode::Not) {
                if let ExprKind::Thunk(body) = &thunk.kind {
                    let mut local = symbols.clone();
                    validate_root_statements(&body.stmts, &mut local, call_results, true)?;
                    if let Some(Stmt::Expr { expr: result, .. }) = body.stmts.last() {
                        let actual = infer_expr(result, &local, call_results, true)?;
                        if actual != StaticType::Bool && actual != StaticType::Unknown {
                            return Err(static_error("E_CHECK_TYPE_MISMATCH",
                                format!("조건은 참거짓이어야 합니다: {}", actual.label()), result));
                        }
                    }
                }
                return Ok(StaticType::Bool);
            }
            Ok(StaticType::Unknown)
        }
        ExprKind::Pipe { stages } => {
            for stage in stages {
                infer_expr(stage, symbols, call_results, unknown_names_allowed)?;
            }
            Ok(StaticType::Unknown)
        }
        _ => Ok(StaticType::Unknown),
    }
}

fn validate_root_statements(
    statements: &[Stmt],
    symbols: &mut BTreeMap<String, StaticType>,
    call_results: &BTreeMap<String, StaticType>,
    unknown_names_allowed: bool,
) -> Result<(), StaticDiagnostic> {
    for statement in statements {
        match statement {
            Stmt::Definition {
                name,
                type_ref,
                value,
                ..
            } => {
                let actual = infer_expr(value, symbols, call_results, unknown_names_allowed)?;
                let expected = type_ref_kind(type_ref);
                // Existing callable-body declared-type diagnostics are emitted
                // by the runtime with pin/expected/actual fields. Keep that
                // contract; this conservative body pass adds logical checking.
                if !unknown_names_allowed && expected != StaticType::Unknown
                    && actual != StaticType::Unknown
                    && expected != actual
                {
                    return Err(static_error(
                        "E_CHECK_TYPE_MISMATCH",
                        format!(
                            "{name}의 정의 형식이 맞지 않습니다: 기대 {}, 실제 {}",
                            expected.label(),
                            actual.label()
                        ),
                        value,
                    ));
                }
                symbols.insert(
                    name.clone(),
                    if expected == StaticType::Unknown {
                        actual
                    } else {
                        expected
                    },
                );
            }
            Stmt::Mutate { target, value, .. } => {
                let actual = infer_expr(value, symbols, call_results, unknown_names_allowed)?;
                if let ExprKind::Var(name) = &target.kind {
                    if let Some(previous) = symbols.get(name).copied() {
                        // 첫 없음의 후속 씨 고정 정책은 이 변경에서 결정하지 않는다.
                        if unknown_names_allowed && previous == StaticType::None {
                            continue;
                        }
                        if previous != StaticType::Unknown
                            && actual != StaticType::Unknown
                            && previous != actual
                        {
                            return Err(static_error(
                                "E_CHECK_TYPE_MISMATCH",
                                format!(
                                    "{name}의 재결속 형식이 맞지 않습니다: 기대 {}, 실제 {}",
                                    previous.label(),
                                    actual.label()
                                ),
                                value,
                            ));
                        }
                        // 동적으로만 알 수 있는 중간 값이 이미 추론한 씨를 지우지 않는다.
                        if previous != StaticType::Unknown {
                            continue;
                        }
                    }
                    symbols.insert(name.clone(), actual);
                } else {
                    infer_expr(target, symbols, call_results, unknown_names_allowed)?;
                }
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                infer_expr(producer, symbols, call_results, unknown_names_allowed)?;
                infer_expr(destination, symbols, call_results, unknown_names_allowed)?;
            }
            Stmt::Expr { expr, .. }
            | Stmt::Show { expr, .. }
            | Stmt::Inspect { expr, .. }
            | Stmt::Return { value: expr, .. } => {
                infer_expr(expr, symbols, call_results, unknown_names_allowed)?;
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let condition_type =
                    infer_expr(condition, symbols, call_results, unknown_names_allowed)?;
                if condition_type != StaticType::Bool && condition_type != StaticType::Unknown {
                    return Err(static_error(
                        "E_CHECK_TYPE_MISMATCH",
                        format!(
                            "만약 조건은 참거짓이어야 합니다: {}",
                            condition_type.label()
                        ),
                        condition,
                    ));
                }
                let mut then_symbols = symbols.clone();
                validate_root_statements(
                    &then_body.stmts,
                    &mut then_symbols,
                    call_results,
                    unknown_names_allowed,
                )?;
                if let Some(else_body) = else_body {
                    let mut else_symbols = symbols.clone();
                    validate_root_statements(
                        &else_body.stmts,
                        &mut else_symbols,
                        call_results,
                        unknown_names_allowed,
                    )?;
                }
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    let actual = item
                        .value
                        .as_ref()
                        .map(|value| {
                            infer_expr(value, symbols, call_results, unknown_names_allowed)
                        })
                        .transpose()?
                        .unwrap_or(StaticType::Unknown);
                    let declared = type_ref_kind(&item.type_ref);
                    symbols.insert(
                        item.name.clone(),
                        if declared == StaticType::Unknown {
                            if unknown_names_allowed && actual == StaticType::None {
                                StaticType::Unknown
                            } else {
                                actual
                            }
                        } else {
                            declared
                        },
                    );
                }
            }
            Stmt::While {
                condition, body, ..
            }
            | Stmt::Guard {
                condition, body, ..
            }
            | Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => {
                infer_expr(condition, symbols, call_results, true)?;
                validate_root_statements(&body.stmts, &mut symbols.clone(), call_results, true)?;
            }
            Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => {
                validate_root_statements(&body.stmts, &mut symbols.clone(), call_results, true)?;
            }
            Stmt::ForEach {
                item,
                item_type,
                iterable,
                body,
                ..
            } => {
                infer_expr(iterable, symbols, call_results, unknown_names_allowed)?;
                let mut nested = symbols.clone();
                nested.insert(
                    item.clone(),
                    item_type
                        .as_ref()
                        .map(type_ref_kind)
                        .unwrap_or(StaticType::Unknown),
                );
                validate_root_statements(&body.stmts, &mut nested, call_results, true)?;
            }
            Stmt::Quantifier {
                variable,
                domain,
                body,
                ..
            } => {
                let mut nested = symbols.clone();
                nested.insert(variable.clone(), type_ref_kind(domain));
                validate_root_statements(&body.stmts, &mut nested, call_results, true)?;
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    infer_expr(
                        &branch.condition,
                        symbols,
                        call_results,
                        unknown_names_allowed,
                    )?;
                    validate_root_statements(
                        &branch.body.stmts,
                        &mut symbols.clone(),
                        call_results,
                        true,
                    )?;
                }
                validate_root_statements(
                    &else_body.stmts,
                    &mut symbols.clone(),
                    call_results,
                    true,
                )?;
            }
            Stmt::Try { action, body, .. } => {
                infer_expr(action, symbols, call_results, unknown_names_allowed)?;
                validate_root_statements(&body.stmts, &mut symbols.clone(), call_results, true)?;
            }
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                infer_expr(condition, symbols, call_results, unknown_names_allowed)?;
                if let Some(body) = then_body {
                    validate_root_statements(
                        &body.stmts,
                        &mut symbols.clone(),
                        call_results,
                        true,
                    )?;
                }
                validate_root_statements(
                    &else_body.stmts,
                    &mut symbols.clone(),
                    call_results,
                    true,
                )?;
            }
            Stmt::Receive {
                sender,
                condition,
                binding,
                body,
                ..
            } => {
                for expr in sender.iter().chain(condition.iter()) {
                    infer_expr(expr, symbols, call_results, true)?;
                }
                let mut nested = symbols.clone();
                if let Some(name) = binding {
                    nested.insert(name.clone(), StaticType::Unknown);
                }
                validate_root_statements(&body.stmts, &mut nested, call_results, true)?;
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                for expr in sender.iter().chain([payload, receiver]) {
                    infer_expr(expr, symbols, call_results, unknown_names_allowed)?;
                }
            }
            Stmt::PublicObservation { fields, .. } => {
                for field in fields {
                    infer_expr(&field.value, symbols, call_results, unknown_names_allowed)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

// Lifecycle bodies can bind world resources before authored root views read
// them. Admit those names, not arbitrary unknown names or callable locals.
// Their value/type remains runtime-checked; this does not initialize a resource.
fn collect_world_bindings(
    statements: &[Stmt],
    locals: &BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    let mut locals = locals.clone();
    for statement in statements {
        match statement {
            Stmt::Definition { name, .. } => { locals.insert(name.clone()); }
            Stmt::DeclBlock { items, .. } => {
                for item in items { locals.insert(item.name.clone()); }
            }
            Stmt::Mutate { target, .. } => {
                if let ExprKind::Var(name) = &target.kind {
                    if !locals.contains(name) { names.insert(name.clone()); }
                }
            }
            Stmt::If { then_body, else_body, .. } => {
                collect_world_bindings(&then_body.stmts, &locals, names);
                if let Some(body) = else_body { collect_world_bindings(&body.stmts, &locals, names); }
            }
            Stmt::While { body, .. } | Stmt::Guard { body, .. }
            | Stmt::HookWhenBecomes { body, .. } | Stmt::HookWhile { body, .. }
            | Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. }
            | Stmt::Hook { body, .. } => collect_world_bindings(&body.stmts, &locals, names),
            Stmt::ForEach { item, body, .. } => {
                let mut nested = locals.clone(); nested.insert(item.clone());
                collect_world_bindings(&body.stmts, &nested, names);
            }
            Stmt::Quantifier { variable, body, .. } => {
                let mut nested = locals.clone(); nested.insert(variable.clone());
                collect_world_bindings(&body.stmts, &nested, names);
            }
            Stmt::Choose { branches, else_body, .. } => {
                for branch in branches { collect_world_bindings(&branch.body.stmts, &locals, names); }
                collect_world_bindings(&else_body.stmts, &locals, names);
            }
            Stmt::Try { body, .. } => {
                let mut nested = locals.clone(); nested.insert("그것".to_string());
                collect_world_bindings(&body.stmts, &nested, names);
            }
            Stmt::Contract { then_body, else_body, .. } => {
                if let Some(body) = then_body { collect_world_bindings(&body.stmts, &locals, names); }
                collect_world_bindings(&else_body.stmts, &locals, names);
            }
            // No expression/thunk/callable traversal: their locals do not
            // become names visible in a root authored view.
            _ => {}
        }
    }
}

fn validate_minimal_root_program(program: &CanonProgram) -> Result<(), StaticDiagnostic> {
    let call_results = program
        .items
        .iter()
        .map(|item| {
            let TopLevelItem::SeedDef(seed) = item;
            (
                seed.canonical_name.clone(),
                seed.result_type
                    .as_ref()
                    .map(type_ref_kind)
                    .unwrap_or(StaticType::Unknown),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut symbols = BTreeMap::new();
    // The parser relocates authored top-level setup into a tick seed. The
    // runtime executes that setup before root statements; analysis must keep
    // its names and declared types visible there without exporting locals
    // from an ordinary callable's own declaration block.
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if !matches!(seed.canonical_name.as_str(), "매마디" | "매틱") { continue; }
        let Some(body) = &seed.body else { continue; };
        for statement in &body.stmts {
            if seed.is_relocated_top_level_setup(statement) {
                validate_root_statements(std::slice::from_ref(statement), &mut symbols, &call_results, false)?;
            }
        }
    }
    let mut world_names = BTreeSet::new();
    for statement in &program.root_stmts {
        match statement {
            Stmt::Hook { body, .. } | Stmt::HookWhenBecomes { body, .. }
            | Stmt::HookWhile { body, .. } =>
                collect_world_bindings(&body.stmts, &BTreeSet::new(), &mut world_names),
            _ => {}
        }
    }
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if matches!(seed.canonical_name.as_str(), "매마디" | "매틱") {
            if let Some(body) = &seed.body {
                let params = seed.params.iter().map(|param| param.pin_name.clone()).collect();
                collect_world_bindings(&body.stmts, &params, &mut world_names);
            }
        }
        // An explicit project entry may own the same lifecycle hooks as a
        // root program. Only their world bindings are visible to root views;
        // ordinary callable locals and parameter names remain private.
        if let Some(body) = &seed.body {
            let params = seed.params.iter().map(|param| param.pin_name.clone()).collect();
            for statement in &body.stmts {
                if let Stmt::Hook { body, .. } = statement {
                    collect_world_bindings(&body.stmts, &params, &mut world_names);
                }
            }
        }
    }
    for name in world_names {
        symbols.entry(name).or_insert(StaticType::Unknown);
    }
    validate_root_statements(&program.root_stmts, &mut symbols, &call_results, false)?;
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        let mut local = symbols.clone();
        for param in &seed.params {
            local.insert(param.pin_name.clone(), type_ref_kind(&param.type_ref));
        }
        if let Some(body) = &seed.body {
            validate_root_statements(&body.stmts, &mut local, &call_results, true)?;
        }
    }
    Ok(())
}

pub(crate) fn validate_program_for_execution(program: &CanonProgram) -> Result<(), String> {
    validate_minimal_root_program(program)
        .map_err(|error| format!("{}: {}", error.code, error.message))
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceAnalysisRequest {
    pub source_text: String,
    #[serde(default = "default_path")]
    pub path: String,
    pub analysis_generation: u64,
    #[serde(default = "default_profile")]
    pub source_profile_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisSpan {
    pub encoding: &'static str,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisOriginalSourceSpan {
    pub coordinate_space: &'static str,
    pub encoding: &'static str,
    pub half_open: bool,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisRepairProposal {
    pub schema: &'static str,
    pub fix_id: &'static str,
    pub source_revision_identity: String,
    pub analysis_generation: u64,
    pub diagnostic_identity: String,
    pub original_source_span: SourceAnalysisOriginalSourceSpan,
    pub expected_text: String,
    pub replacement_text: String,
    pub proposal_identity: String,
    pub explanation: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisDiagnostic {
    pub code: String,
    pub severity: &'static str,
    pub message: String,
    pub path: String,
    pub span: SourceAnalysisSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repair_proposal: Option<SourceAnalysisRepairProposal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisSnapshot {
    pub schema: &'static str,
    pub source_revision_identity: String,
    pub analysis_generation: u64,
    pub source_profile_identity: String,
    pub outcome: &'static str,
    pub diagnostics: Vec<SourceAnalysisDiagnostic>,
}

fn default_path() -> String {
    "lesson.ddn".to_string()
}

fn default_profile() -> String {
    String::new()
}

fn source_revision_identity(source: &str) -> String {
    format!("blake3:{}", blake3::hash(source.as_bytes()).to_hex())
}

fn deterministic_identity(domain: &str, parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&(part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    format!("{domain}:blake3:{}", hasher.finalize().to_hex())
}

fn raw_chaebi_header_colon_spans(source: &str) -> Option<Vec<(usize, usize)>> {
    let tokens = Lexer::new(source).tokenize().ok()?;
    let mut spans = Vec::new();
    for window in tokens.windows(3) {
        let is_chaebi = match &window[0].kind {
            TokenKind::Ident(name) | TokenKind::Josa(name) => name == "채비",
            _ => false,
        };
        if is_chaebi
            && matches!(window[1].kind, TokenKind::Colon)
            && matches!(window[2].kind, TokenKind::LBrace)
        {
            spans.push((window[1].span.start, window[1].span.end));
        }
    }
    Some(spans)
}

fn is_chaebi_header_warning(warning: &DdnParseWarning) -> bool {
    warning.code == "W_BLOCK_HEADER_COLON_DEPRECATED" && warning.message.contains("`채비:`")
}

fn authoritative_repair_proposal(
    source: &str,
    revision: &str,
    generation: u64,
    occurrence_index: usize,
    original_span: (usize, usize),
) -> Option<(String, SourceAnalysisRepairProposal)> {
    let (start, end) = original_span;
    if source.get(start..end)? != ":" {
        return None;
    }
    let generation_bytes = generation.to_be_bytes();
    let occurrence_bytes = (occurrence_index as u64).to_be_bytes();
    let start_bytes = (start as u64).to_be_bytes();
    let end_bytes = (end as u64).to_be_bytes();
    let diagnostic_identity = deterministic_identity(
        "ddn.source_analysis_diagnostic.candidate.v1",
        &[
            revision.as_bytes(),
            &generation_bytes,
            b"W_BLOCK_HEADER_COLON_DEPRECATED",
            &occurrence_bytes,
            &start_bytes,
            &end_bytes,
        ],
    );
    let proposal_identity = deterministic_identity(
        "ddn.source_analysis_repair_proposal.candidate.v1",
        &[
            revision.as_bytes(),
            diagnostic_identity.as_bytes(),
            &start_bytes,
            &end_bytes,
            b":",
            b"",
        ],
    );
    let proposal = SourceAnalysisRepairProposal {
        schema: REPAIR_PROPOSAL_SCHEMA,
        fix_id: REMOVE_SETUP_COLON_FIX_ID,
        source_revision_identity: revision.to_string(),
        analysis_generation: generation,
        diagnostic_identity: diagnostic_identity.clone(),
        original_source_span: SourceAnalysisOriginalSourceSpan {
            coordinate_space: "original-source",
            encoding: "utf-8-byte",
            half_open: true,
            start,
            end,
        },
        expected_text: ":".to_string(),
        replacement_text: String::new(),
        proposal_identity,
        explanation: "비권장 블록 헤더의 콜론을 제거합니다.",
    };
    Some((diagnostic_identity, proposal))
}

fn accepted_diagnostics(
    source: &str,
    path: &str,
    revision: &str,
    generation: u64,
    warnings: &[DdnParseWarning],
) -> Vec<SourceAnalysisDiagnostic> {
    let repairable_count = warnings
        .iter()
        .filter(|warning| is_chaebi_header_warning(warning))
        .count();
    let raw_spans =
        raw_chaebi_header_colon_spans(source).filter(|spans| spans.len() == repairable_count);
    let mut repairable_ordinal = 0usize;

    warnings
        .iter()
        .map(|warning| {
            let repair = if is_chaebi_header_warning(warning) {
                let ordinal = repairable_ordinal;
                repairable_ordinal += 1;
                raw_spans.as_ref().and_then(|spans| {
                    spans.get(ordinal).and_then(|span| {
                        authoritative_repair_proposal(source, revision, generation, ordinal, *span)
                    })
                })
            } else {
                None
            };
            let (diagnostic_identity, repair_proposal) = match repair {
                Some((identity, proposal)) => (Some(identity), Some(proposal)),
                None => (None, None),
            };
            SourceAnalysisDiagnostic {
                code: warning.code.clone(),
                severity: "warning",
                message: warning.message.clone(),
                path: path.to_string(),
                span: SourceAnalysisSpan {
                    encoding: "utf-8-byte",
                    start: warning.span_start,
                    end: warning.span_end,
                },
                diagnostic_identity,
                repair_proposal,
            }
        })
        .collect()
}

fn diagnostic_code(message: &str) -> String {
    let Some(start) = message.find("E_") else {
        return "E_PARSE".to_string();
    };
    let tail = &message[start..];
    let end = tail
        .char_indices()
        .find_map(|(index, ch)| {
            if index > 0 && !(ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_') {
                Some(index)
            } else {
                None
            }
        })
        .unwrap_or(tail.len());
    tail[..end].to_string()
}

fn byte_offset_for_line_column(source: &str, target_line: usize, target_column: usize) -> usize {
    let mut line = 1usize;
    let mut column = 1usize;
    for (offset, ch) in source.char_indices() {
        if line == target_line && column >= target_column {
            return offset;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    source.len()
}

fn formatted_error_span(source: &str, message: &str) -> SourceAnalysisSpan {
    let mut start = 0usize;
    if let Some(location_end) = message.find(")\n") {
        if let Some(location_start) = message[..location_end].rfind('(') {
            let location = &message[location_start + 1..location_end];
            if let Some((line, column)) = location.split_once(':') {
                if let (Ok(line), Ok(column)) = (line.parse::<usize>(), column.parse::<usize>()) {
                    start = byte_offset_for_line_column(source, line, column);
                }
            }
        }
    }
    let end = source[start..]
        .chars()
        .next()
        .map(|ch| start + ch.len_utf8())
        .unwrap_or(start);
    SourceAnalysisSpan {
        encoding: "utf-8-byte",
        start,
        end,
    }
}

pub fn analyze_source(request: SourceAnalysisRequest) -> SourceAnalysisSnapshot {
    let revision = source_revision_identity(&request.source_text);
    let program = if request.source_profile_identity == STRICT_SOURCE_PROFILE_IDENTITY {
        DdnProgram::from_source_with_mode(&request.source_text, &request.path, ParseMode::Strict)
    } else {
        let profile = match resolve_supported_source_profile(Some(&request.source_profile_identity))
        {
            Ok(profile) => profile,
            Err(error) => {
                return SourceAnalysisSnapshot {
                    schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
                    source_revision_identity: revision,
                    analysis_generation: request.analysis_generation,
                    source_profile_identity: request.source_profile_identity,
                    outcome: "UNSUPPORTED",
                    diagnostics: vec![SourceAnalysisDiagnostic {
                        code: error.code.to_string(),
                        severity: "error",
                        message: error.message,
                        path: request.path,
                        span: SourceAnalysisSpan {
                            encoding: "utf-8-byte",
                            start: 0,
                            end: 0,
                        },
                        diagnostic_identity: None,
                        repair_proposal: None,
                    }],
                };
            }
        };
        DdnProgram::from_supported_source_profile_with_mode(
            &request.source_text,
            &request.path,
            ParseMode::Strict,
            profile,
        )
    };

    match program {
        Ok(program) => {
            let mut diagnostics = accepted_diagnostics(
                &request.source_text,
                &request.path,
                &revision,
                request.analysis_generation,
                program.parse_warnings(),
            );
            let mut outcome = "ACCEPTED";
            if request.source_profile_identity == V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY {
                if let Err(error) = validate_minimal_root_program(program.canonical_program()) {
                    outcome = "REJECTED";
                    diagnostics.push(SourceAnalysisDiagnostic {
                        code: error.code.to_string(),
                        severity: "error",
                        message: error.message,
                        path: request.path.clone(),
                        span: SourceAnalysisSpan {
                            encoding: "utf-8-byte",
                            start: error.start,
                            end: error.end,
                        },
                        diagnostic_identity: None,
                        repair_proposal: None,
                    });
                }
            }
            SourceAnalysisSnapshot {
                schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
                source_revision_identity: revision.clone(),
                analysis_generation: request.analysis_generation,
                source_profile_identity: request.source_profile_identity,
                outcome,
                diagnostics,
            }
        }
        Err(message) => SourceAnalysisSnapshot {
            schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
            source_revision_identity: revision,
            analysis_generation: request.analysis_generation,
            source_profile_identity: request.source_profile_identity,
            outcome: "REJECTED",
            diagnostics: vec![SourceAnalysisDiagnostic {
                code: diagnostic_code(&message),
                severity: "error",
                span: formatted_error_span(&request.source_text, &message),
                message,
                path: request.path,
                diagnostic_identity: None,
                repair_proposal: None,
            }],
        },
    }
}

pub fn source_analysis_snapshot_json(request_json: &str) -> Result<String, String> {
    let request: SourceAnalysisRequest = serde_json::from_str(request_json)
        .map_err(|error| format!("E_SOURCE_ANALYSIS_REQUEST_INVALID {error}"))?;
    serde_json::to_string(&analyze_source(request))
        .map_err(|error| format!("E_SOURCE_ANALYSIS_SERIALIZE_FAILED {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(source: &str, generation: u64, profile: &str) -> SourceAnalysisRequest {
        SourceAnalysisRequest {
            source_text: source.to_string(),
            path: "lesson.ddn".to_string(),
            analysis_generation: generation,
            source_profile_identity: profile.to_string(),
        }
    }

    #[test]
    fn alpha3_rd136_time_constructors_are_not_rejected_by_retired_alpha2_guard() {
        for source in [
            "시간 := (2) 표준시간.고정마디.",
            "시간 := (-1) 표준시간.상대마디.",
            "시간 := (번호=2) 표준시간.고정마디.",
            "시간 := (차이=-1) 표준시간.상대마디.",
            "계산:움직씨 := { 시간 := (2) 표준시간.고정마디. }.",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_ne!(snapshot.outcome, "REJECTED", "{source}: {:?}",snapshot.diagnostics);
        }
        let text = analyze_source(request("글 := \"표준시간.고정마디\".", 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
        assert_ne!(text.outcome, "REJECTED");
    }

    #[test]
    fn logical_static_checks_do_not_skip_known_bad_rhs() {
        for source in [
            "결과 := (거짓 그리고 1).",
            "결과 := (참 또는 1).",
            "결과 := (1 그리고 참).",
            "결과 := (거짓 && 1).",
            "결과 := (참 || 1).",
            "계산:움직씨 := { 결과 := (거짓 그리고 1). }",
            "계산:움직씨 := { { 참 }이면 { 결과 := (참 또는 1). }. }",
            "(매마디)마다 { 결과 := (거짓 그리고 1). }",
        ] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(snapshot.outcome, "REJECTED", "{source}");
            assert_eq!(
                snapshot.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH",
                "{source}: {:?}",
                snapshot.diagnostics
            );
        }
    }

    #[test]
    fn logical_static_checks_accept_boolean_and_runtime_unknown_operands() {
        for source in [
            "결과 := (거짓 그리고 참).",
            "결과 := (참 또는 거짓).",
            "계산:움직씨 := { 결과 := (외부값 그리고 참). }",
            "계산:움직씨 := { 결과 := (외부값 또는 거짓). }",
        ] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(
                snapshot.outcome, "ACCEPTED",
                "{source}: {:?}",
                snapshot.diagnostics
            );
        }
    }

    #[test]
    fn logical_call_source_is_not_an_accepted_surface() {
        for source in ["결과 := (거짓, 참) 그리고.", "결과 := (참, 거짓) 또는."] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(snapshot.outcome, "REJECTED");
            assert_eq!(snapshot.diagnostics[0].code, "E_PARSE");
        }
    }

    #[test]
    fn logical_internal_call_ast_checks_both_operands() {
        use ddonirang_lang::{ArgBinding, BindingReason, Span};
        for operator in ["그리고", "또는"] {
            for (rhs, accepted) in [(Literal::Bool(true), true), (Literal::Int(1), false)] {
                let args = [Literal::Bool(false), rhs]
                    .into_iter()
                    .enumerate()
                    .map(|(id, literal)| ArgBinding {
                        id: id as u64,
                        span: Span::new(0, 0),
                        expr: Expr::new(id as u64, Span::new(0, 0), ExprKind::Literal(literal)),
                        josa: None,
                        resolved_pin: None,
                        binding_reason: BindingReason::Positional,
                    })
                    .collect();
                let expr = Expr::new(
                    3,
                    Span::new(0, 0),
                    ExprKind::Call {
                        args,
                        func: operator.to_string(),
                    },
                );
                let result = infer_expr(&expr, &BTreeMap::new(), &BTreeMap::new(), false);
                if accepted {
                    assert_eq!(result.unwrap(), StaticType::Bool);
                } else {
                    assert_eq!(result.unwrap_err().code, "E_CHECK_TYPE_MISMATCH");
                }
            }
        }
    }

    #[test]
    fn unknown_intermediate_rebind_does_not_erase_first_inferred_type() {
        for first in ["값 := 1.", "값 <- 1."] {
            let source = format!("{first} 값 <- (\"a\", 1) 짝맞춤. 값 <- \"가\".");
            let snapshot = analyze_source(request(
                &source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(snapshot.outcome, "REJECTED", "{source}");
            assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH");
        }
    }

    #[test]
    fn body_checks_preserve_first_none_and_local_definition_shadowing() {
        for source in [
            "계산:움직씨 := { 값 := 없음. 값 <- 1. }",
            "계산:움직씨 := { 값 <- 없음. 값 <- 1. }",
            "계산:움직씨 := { 값 <- 없음. 값 <- 1. 값 <- \"가\". }",
            "값 := 1. 계산:움직씨 := { 값 := \"가\". 값 <- \"나\". }",
        ] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(
                snapshot.outcome, "ACCEPTED",
                "{source}: {:?}",
                snapshot.diagnostics
            );
        }
        let root = analyze_source(request(
            "값 := 없음. 값 <- 1.",
            0,
            V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(root.outcome, "REJECTED");
        assert_eq!(root.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH");
    }

    #[test]
    fn current_minimal_root_program_rejects_mixed_addition_before_execution() {
        let snapshot = analyze_source(request(
            "\"점수: \" + 10 보여주기.\n",
            0,
            V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH");
    }

    #[test]
    fn lifecycle_world_bindings_are_visible_without_callable_local_leaks() {
        for source in [
            "(시작)할때 { 초기화됨 <- 0. }. (매마디)마다 { { 초기화됨 == 0 }이면 { 시간 <- 0. 초기화됨 <- 1. }. 시간 <- 시간 + 1. x <- 시간 * 2. }. x 보여주기.",
            "(시작)할때 { x <- 1. }. x 보여주기.",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{source}: {:?}", snapshot.diagnostics);
        }
        for source in [
            "계산:움직씨 := { 지역값 := 1. }. 지역값 보여주기.",
            "(매마디)마다 { 지역값 := 1. }. 지역값 보여주기.",
            "(매마디)마다 { x <- 1. }. 없는값 보여주기.",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "REJECTED", "{source}");
            assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_UNKNOWN_NAME", "{source}: {:?}", snapshot.diagnostics);
        }
    }

    #[test]
    fn current_minimal_root_program_rejects_unknown_name_before_execution() {
        let snapshot = analyze_source(request(
            "없는값 보여주기.\n",
            0,
            V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_UNKNOWN_NAME");
    }

    #[test]
    fn current_minimal_root_program_accepts_checked_arithmetic() {
        let snapshot = analyze_source(request(
            "결과 <- 3 + 4.\n결과 보여주기.\n",
            0,
            V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(snapshot.outcome, "ACCEPTED", "{:?}", snapshot.diagnostics);
    }

    #[test]
    fn root_program_reads_parser_relocated_top_level_setup() {
        // A3-P3: top-level setup remains world-owned after the parser moves
        // its declaration block into the tick seed. It is not a local name.
        for source in [
            "채비 { r:나눔수 := (6,9) 나눔수. f:곱수 := (72) 곱수. }. r 보여주기. f 보여주기.",
            "채비 { 값:수 := 2. }. (매마디)마다 { 값 <- 값 + 1. }. 값 보여주기.",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{:?}", snapshot.diagnostics);
        }
    }

    #[test]
    fn root_setup_preserves_types_and_does_not_export_callable_locals() {
        for (source, code) in [
            ("채비 { 값:글 := \"가\". }. (값 + 1) 보여주기.", "E_CHECK_TYPE_MISMATCH"),
            ("실행:움직씨 := { 채비 { 숨김:수 := 2. }. }. 숨김 보여주기.", "E_CHECK_UNKNOWN_NAME"),
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "REJECTED", "{:?}", snapshot.diagnostics);
            assert_eq!(snapshot.diagnostics[0].code, code);
        }
    }

    #[test]
    fn current_minimal_root_program_recognizes_runtime_builtin_values() {
        for source in [
            "{ 참 }이면 { 100 보여주기. } 아니면 { 200 보여주기. }.",
            "{ 거짓 }이면 { 100 보여주기. } 아니면 { 200 보여주기. }.",
            "없음 보여주기.",
        ] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(
                snapshot.outcome, "ACCEPTED",
                "{source}: {:?}",
                snapshot.diagnostics
            );
        }
    }

    #[test]
    fn accepted_snapshot_preserves_identity_generation_and_utf8_warning_span() {
        let source = "매틱:움직씨 = {\n  채비: { 값:수 <- 0. }.\n}";
        let snapshot = analyze_source(request(source, 7, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_eq!(snapshot.outcome, "ACCEPTED");
        assert_eq!(snapshot.analysis_generation, 7);
        assert_eq!(snapshot.source_profile_identity, "strict");
        assert!(snapshot.source_revision_identity.starts_with("blake3:"));
        let warning = snapshot
            .diagnostics
            .iter()
            .find(|row| row.code == "W_BLOCK_HEADER_COLON_DEPRECATED")
            .expect("authoritative warning");
        assert_eq!(warning.severity, "warning");
        assert_eq!(warning.span.encoding, "utf-8-byte");
        assert!(warning.span.end > warning.span.start);
        let proposal = warning
            .repair_proposal
            .as_ref()
            .expect("authoritative repair proposal");
        let original_colon = source.find("채비:").unwrap() + "채비".len();
        assert_eq!(proposal.original_source_span.start, original_colon);
        assert_eq!(proposal.original_source_span.end, original_colon + 1);
        assert_eq!(
            proposal.original_source_span.coordinate_space,
            "original-source"
        );
        assert_eq!(proposal.expected_text, ":");
        assert_eq!(proposal.replacement_text, "");
        assert_eq!(
            warning.diagnostic_identity.as_deref(),
            Some(proposal.diagnostic_identity.as_str())
        );
    }

    fn proposal_spans(snapshot: &SourceAnalysisSnapshot) -> Vec<(usize, usize)> {
        snapshot
            .diagnostics
            .iter()
            .filter_map(|row| row.repair_proposal.as_ref())
            .map(|proposal| {
                (
                    proposal.original_source_span.start,
                    proposal.original_source_span.end,
                )
            })
            .collect()
    }

    #[test]
    fn authoritative_repair_proposals_match_original_a_to_h() {
        let cases: Vec<(&str, String, Vec<usize>)> = vec![
            (
                "A_ASCII",
                "매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "B_KOREAN_COMMENT",
                "// 앞한글\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "C_NON_BMP_COMMENT",
                "// 😀\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "D_COMBINING_COMMENT",
                "// e\u{301}\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "E_MULTIPLE",
                "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }"
                    .to_string(),
                Vec::new(),
            ),
            (
                "F_STRING_DECOY",
                "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"
                    .to_string(),
                Vec::new(),
            ),
            (
                "G_COMMENT_DECOY",
                "// 채비: { 주석 decoy\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"
                    .to_string(),
                Vec::new(),
            ),
            (
                "H_OPAQUE_DECOY",
                "매틱:움직씨 = { 채비 { 이름:글 <- (x=1) 글무늬{\"채비:\"}. }. 채비: { 값:수 <- 1. }. }"
                    .to_string(),
                Vec::new(),
            ),
        ];

        for (case_id, source, _) in cases {
            let expected = if case_id == "E_MULTIPLE" {
                source
                    .match_indices("채비:")
                    .map(|(start, _)| start + "채비".len())
                    .collect::<Vec<_>>()
            } else {
                vec![source.rfind("채비:").unwrap() + "채비".len()]
            };
            let snapshot = analyze_source(request(&source, 41, STRICT_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{case_id}");
            assert_eq!(
                proposal_spans(&snapshot),
                expected
                    .iter()
                    .map(|start| (*start, *start + 1))
                    .collect::<Vec<_>>(),
                "{case_id}"
            );
            for diagnostic in snapshot
                .diagnostics
                .iter()
                .filter(|row| row.code == "W_BLOCK_HEADER_COLON_DEPRECATED")
            {
                let proposal = diagnostic.repair_proposal.as_ref().expect(case_id);
                assert_eq!(
                    source.get(
                        proposal.original_source_span.start..proposal.original_source_span.end
                    ),
                    Some(":"),
                    "{case_id}"
                );
            }
        }
    }

    #[test]
    fn multiple_occurrence_identities_are_unique_and_deterministic() {
        let source = "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }";
        let first = analyze_source(request(source, 42, STRICT_SOURCE_PROFILE_IDENTITY));
        let repeated = analyze_source(request(source, 42, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_eq!(first, repeated);
        let diagnostics = first
            .diagnostics
            .iter()
            .filter(|row| row.repair_proposal.is_some())
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 2);
        assert_ne!(
            diagnostics[0].diagnostic_identity,
            diagnostics[1].diagnostic_identity
        );
        assert_ne!(
            diagnostics[0]
                .repair_proposal
                .as_ref()
                .unwrap()
                .proposal_identity,
            diagnostics[1]
                .repair_proposal
                .as_ref()
                .unwrap()
                .proposal_identity
        );
        let first_span = &diagnostics[0]
            .repair_proposal
            .as_ref()
            .unwrap()
            .original_source_span;
        let second_span = &diagnostics[1]
            .repair_proposal
            .as_ref()
            .unwrap()
            .original_source_span;
        assert_ne!(first_span, second_span);
        assert_eq!(source.get(first_span.start..first_span.end), Some(":"));
        assert_eq!(source.get(second_span.start..second_span.end), Some(":"));
        assert!(first_span.start < second_span.start);
        let next_generation = analyze_source(request(source, 43, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_ne!(
            diagnostics[0].diagnostic_identity,
            next_generation.diagnostics[0].diagnostic_identity
        );
    }

    #[test]
    fn string_decoy_is_not_mutated_and_reanalysis_advances_revision() {
        let source = "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }";
        let snapshot = analyze_source(request(source, 44, STRICT_SOURCE_PROFILE_IDENTITY));
        let proposal = snapshot.diagnostics[0].repair_proposal.as_ref().unwrap();
        let mut edited = source.to_string();
        edited.replace_range(
            proposal.original_source_span.start..proposal.original_source_span.end,
            &proposal.replacement_text,
        );
        assert!(edited.contains("제목: \"채비: {\""));
        assert!(edited.contains("매틱:움직씨 = { 채비 {"));
        assert!(!edited.contains("매틱:움직씨 = { 채비: {"));
        let reanalyzed = analyze_source(request(&edited, 45, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_ne!(
            snapshot.source_revision_identity,
            reanalyzed.source_revision_identity
        );
        assert!(!reanalyzed
            .diagnostics
            .iter()
            .any(|row| row.code == "W_BLOCK_HEADER_COLON_DEPRECATED"));
    }

    #[test]
    fn native_payload_matches_t4h_cross_projection_seal() {
        let source = "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }";
        let snapshot = analyze_source(request(source, 270, STRICT_SOURCE_PROFILE_IDENTITY));
        let diagnostic = &snapshot.diagnostics[0];
        let proposal = diagnostic.repair_proposal.as_ref().unwrap();
        assert_eq!(
            snapshot.source_revision_identity,
            "blake3:3159cfef3cdbeeeb74ed267fe9cae2a12944661234c0a942b38c1339057abaab"
        );
        assert_eq!(
            diagnostic.diagnostic_identity.as_deref(),
            Some("ddn.source_analysis_diagnostic.candidate.v1:blake3:9ac895408889cb0cd9b7b2aaf73a98eb72637242512f8204fbdc9649c6e79e1f")
        );
        assert_eq!(
            proposal.proposal_identity,
            "ddn.source_analysis_repair_proposal.candidate.v1:blake3:b36a7c471fdcb01548dd56902ec27e449faeaf9bac72da74a7c67c5a248f7158"
        );
        assert_eq!(proposal.original_source_span.start, 60);
        assert_eq!(proposal.original_source_span.end, 61);
        assert_eq!(proposal.expected_text, ":");
        assert_eq!(proposal.replacement_text, "");
    }

    #[test]
    fn association_mismatch_preserves_diagnostic_without_proposal() {
        let source = "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }";
        let warnings = vec![DdnParseWarning {
            code: "W_BLOCK_HEADER_COLON_DEPRECATED".to_string(),
            message: "블록 헤더의 `채비:` 표기는 예정된 비권장입니다.".to_string(),
            span_start: 0,
            span_end: 1,
        }];
        let diagnostics =
            accepted_diagnostics(source, "lesson.ddn", "blake3:mismatch", 46, &warnings);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].diagnostic_identity.is_none());
        assert!(diagnostics[0].repair_proposal.is_none());
    }

    #[test]
    fn rejected_snapshot_is_fail_closed() {
        let snapshot = analyze_source(request(
            "매틱:움직씨 = { 값:수 <- . }",
            8,
            STRICT_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(snapshot.diagnostics.len(), 1);
        assert_eq!(snapshot.diagnostics[0].severity, "error");
        assert_eq!(snapshot.diagnostics[0].span.encoding, "utf-8-byte");
    }

    #[test]
    fn unsupported_profile_is_not_downgraded() {
        let snapshot = analyze_source(request("값:수 <- 1.", 9, "unknown-profile"));
        assert_eq!(snapshot.outcome, "UNSUPPORTED");
        assert_eq!(snapshot.diagnostics[0].code, "E_SOURCE_PROFILE_UNSUPPORTED");
    }

    const P1A_V1_CORE_PROFILE: &str = "v1-core-v25";

    #[test]
    fn p1a_v1_core_profile_accepts_definition_and_rebinding() {
        let source = "계산:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n";
        let snapshot = analyze_source(request(source, 101, P1A_V1_CORE_PROFILE));
        assert_eq!(snapshot.outcome, "ACCEPTED");
        assert!(snapshot.diagnostics.is_empty());
    }

    #[test]
    fn p1a_v1_core_profile_reports_exact_legacy_definition_migration() {
        let source = "계산:움직씨 = {\n}\n";
        let snapshot = analyze_source(request(source, 102, P1A_V1_CORE_PROFILE));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(
            snapshot.diagnostics[0].code,
            "E_V25_DEFINITION_MIGRATION_REQUIRED"
        );
    }

    #[test]
    fn p1a_v1_core_profile_rejects_direct_and_mutual_recursion_deterministically() {
        let direct = "첫계산:움직씨 := {\n    () 첫계산하기.\n}\n";
        let mutual = "첫계산:움직씨 := {\n    () 둘계산하기.\n}\n둘계산:움직씨 := {\n    () 첫계산하기.\n}\n";
        for source in [direct, mutual] {
            let first = analyze_source(request(source, 103, P1A_V1_CORE_PROFILE));
            let repeated = analyze_source(request(source, 103, P1A_V1_CORE_PROFILE));
            assert_eq!(first, repeated);
            assert_eq!(first.outcome, "REJECTED");
            assert_eq!(
                first.diagnostics[0].code, "E_V1_CORE_RECURSION_UNSUPPORTED",
                "diagnostic={:?}",
                first.diagnostics
            );
        }
    }

    #[test]
    fn p1a_v1_core_profile_omission_fails_closed_without_default_or_retry() {
        let request_json = serde_json::json!({
            "source_text": "계산:움직씨 := {\n    값:수 := 1.\n}\n",
            "path": "omitted-profile.ddn",
            "analysis_generation": 104
        })
        .to_string();
        let response = source_analysis_snapshot_json(&request_json).expect("structured response");
        let snapshot: serde_json::Value =
            serde_json::from_str(&response).expect("snapshot response");
        assert_eq!(snapshot["outcome"], "UNSUPPORTED");
        assert_eq!(
            snapshot["diagnostics"][0]["code"],
            "E_SOURCE_PROFILE_REQUIRED"
        );
    }
}
