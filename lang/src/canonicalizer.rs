use crate::ast::*;
use crate::lexer::{Lexer, TokenKind};
use crate::parser::ParseError;
use crate::stdlib::minimal_stdlib_sigs;
use crate::term_map;
use std::collections::{HashMap, HashSet};

const CALL_TAIL_SHORT_FORMS: [&str; 4] = ["기", "고", "면", "면서"];

pub struct LintWarning {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
}

pub struct CanonicalizeReport {
    pub warnings: Vec<LintWarning>,
}

pub fn canonicalize(program: &mut CanonProgram) -> Result<CanonicalizeReport, ParseError> {
    let mut warnings = Vec::new();
    let signatures = collect_seed_signatures(program);
    for item in &mut program.items {
        canonicalize_top_level_item(item, &signatures, &mut warnings)?;
    }
    let known_seeds = collect_known_seeds(program);
    let stdlib_names = collect_stdlib_names();
    lint_tailless_calls(program, &known_seeds, &stdlib_names, &mut warnings);
    lint_deprecated_block_header_colon(program, &mut warnings);
    Ok(CanonicalizeReport { warnings })
}

fn lint_deprecated_block_header_colon(program: &CanonProgram, warnings: &mut Vec<LintWarning>) {
    let Ok(tokens) = Lexer::new(&program.origin.source).tokenize() else {
        return;
    };
    if tokens.len() < 3 {
        return;
    }
    for idx in 0..(tokens.len() - 2) {
        if !matches!(tokens[idx + 1].kind, TokenKind::Colon) {
            continue;
        }
        if !matches!(tokens[idx + 2].kind, TokenKind::LBrace) {
            continue;
        }
        let is_deprecated_header = match &tokens[idx].kind {
            TokenKind::KwBanbok | TokenKind::KwDongan | TokenKind::KwDaehae | TokenKind::KwBeat => true,
            TokenKind::Ident(name) | TokenKind::Josa(name) => {
                matches!(name.as_str(), "채비" | "설정" | "보개" | "모양" | "슬기")
            }
            _ => false,
        };
        if !is_deprecated_header {
            continue;
        }
        let keyword = tokens[idx].raw.trim();
        warnings.push(LintWarning {
            code: "W_BLOCK_HEADER_COLON_DEPRECATED",
            span: Span {
                start: tokens[idx].span.start,
                end: tokens[idx + 2].span.end,
            },
            message: format!(
                "블록 헤더의 `{keyword}:` 표기는 예정된 비권장입니다. `{keyword} {{` 표기로 전환하세요"
            ),
        });
    }
}

fn canonicalize_top_level_item(
    item: &mut TopLevelItem,
    signatures: &HashMap<String, Vec<ParamPin>>,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    match item {
        TopLevelItem::SeedDef(seed) => canonicalize_seed_def(seed, signatures, warnings),
    }
}

fn canonicalize_seed_def(
    seed: &mut SeedDef,
    signatures: &HashMap<String, Vec<ParamPin>>,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    canonicalize_ident(&mut seed.canonical_name, seed.span, warnings)?;
    if let SeedKind::Named(name) = &mut seed.seed_kind {
        canonicalize_ident(name, seed.span, warnings)?;
    }
    for param in &mut seed.params {
        canonicalize_ident(&mut param.pin_name, param.span, warnings)?;
        canonicalize_type_ref(&mut param.type_ref, param.span, warnings)?;
        if let Some(default_value) = &mut param.default_value {
            canonicalize_expr(default_value, signatures, warnings)?;
        }
    }
    if let Some(body) = &mut seed.body {
        canonicalize_body(body, signatures, warnings)?;
    }
    Ok(())
}

fn canonicalize_body(
    body: &mut Body,
    signatures: &HashMap<String, Vec<ParamPin>>,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    for stmt in &mut body.stmts {
        canonicalize_stmt(stmt, signatures, warnings)?;
    }
    Ok(())
}

fn canonicalize_stmt(
    stmt: &mut Stmt,
    signatures: &HashMap<String, Vec<ParamPin>>,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    match stmt {
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                canonicalize_ident(&mut item.name, item.span, warnings)?;
                canonicalize_type_ref(&mut item.type_ref, item.span, warnings)?;
                if let Some(value) = &mut item.value {
                    canonicalize_expr(value, signatures, warnings)?;
                }
            }
        }
        Stmt::Mutate { target, value, .. } => {
            canonicalize_expr(target, signatures, warnings)?;
            canonicalize_expr(value, signatures, warnings)?;
        }
        Stmt::Expr { expr, .. } => canonicalize_expr(expr, signatures, warnings)?,
        Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            canonicalize_expr(expr, signatures, warnings)?
        }
        Stmt::MetaBlock { .. } => {}
        Stmt::Pragma { .. } => {}
        Stmt::Return { value, .. } => canonicalize_expr(value, signatures, warnings)?,
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            canonicalize_body(then_body, signatures, warnings)?;
            if let Some(body) = else_body {
                canonicalize_body(body, signatures, warnings)?;
            }
        }
        Stmt::Try { action, body, .. } => {
            canonicalize_expr(action, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                canonicalize_expr(&mut branch.condition, signatures, warnings)?;
                canonicalize_body(&mut branch.body, signatures, warnings)?;
            }
            canonicalize_body(else_body, signatures, warnings)?;
        }
        Stmt::Repeat { body, .. } => {
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::BeatBlock { body, .. } => {
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::Hook { body, .. } => {
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::While {
            condition, body, ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::ForEach {
            item,
            item_type,
            iterable,
            body,
            span,
            ..
        } => {
            canonicalize_ident(item, *span, warnings)?;
            if let Some(type_ref) = item_type.as_mut() {
                canonicalize_type_ref(type_ref, *span, warnings)?;
            }
            canonicalize_expr(iterable, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::Quantifier {
            variable,
            domain,
            body,
            span,
            ..
        } => {
            canonicalize_ident(variable, *span, warnings)?;
            canonicalize_type_ref(domain, *span, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::Break { .. } => {}
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            if let Some(body) = then_body {
                canonicalize_body(body, signatures, warnings)?;
            }
            canonicalize_body(else_body, signatures, warnings)?;
        }
        Stmt::Guard {
            condition, body, ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
    }
    Ok(())
}

fn canonicalize_expr(
    expr: &mut Expr,
    signatures: &HashMap<String, Vec<ParamPin>>,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    match &mut expr.kind {
        ExprKind::Var(name) => canonicalize_ident(name, expr.span, warnings)?,
        ExprKind::FieldAccess { target, field } => {
            canonicalize_expr(target, signatures, warnings)?;
            canonicalize_ident(field, expr.span, warnings)?;
        }
        ExprKind::Call { args, func } => {
            canonicalize_ident(func, expr.span, warnings)?;
            for arg in args {
                canonicalize_expr(&mut arg.expr, signatures, warnings)?;
                if let Some(pin) = &mut arg.resolved_pin {
                    canonicalize_ident(pin, arg.span, warnings)?;
                }
                canonicalize_call_arg_josa(arg, func, signatures);
                lint_call_arg_josa_conflict(arg, func, signatures, warnings);
            }
        }
        ExprKind::Infix { left, right, .. } => {
            canonicalize_expr(left, signatures, warnings)?;
            canonicalize_expr(right, signatures, warnings)?;
        }
        ExprKind::Suffix { value, .. } => canonicalize_expr(value, signatures, warnings)?,
        ExprKind::Thunk(body) => canonicalize_body(body, signatures, warnings)?,
        ExprKind::Eval { thunk, .. } => canonicalize_expr(thunk, signatures, warnings)?,
        ExprKind::SeedLiteral { param, body } => {
            canonicalize_ident(param, expr.span, warnings)?;
            canonicalize_expr(body, signatures, warnings)?;
        }
        ExprKind::Pipe { stages } => {
            for stage in stages {
                canonicalize_expr(stage, signatures, warnings)?;
            }
        }
        ExprKind::Pack { fields } => {
            for (name, value) in fields {
                canonicalize_ident(name, expr.span, warnings)?;
                canonicalize_expr(value, signatures, warnings)?;
            }
        }
        ExprKind::Assertion(_) => {}
        ExprKind::Formula(_) => {}
        ExprKind::Template(_) => {}
        ExprKind::StateMachine(_) => {}
        ExprKind::TemplateRender { inject, .. } => {
            for (_, value) in inject {
                canonicalize_expr(value, signatures, warnings)?;
            }
        }
        ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                canonicalize_expr(value, signatures, warnings)?;
            }
        }
        ExprKind::Nuance { expr, .. } => canonicalize_expr(expr, signatures, warnings)?,
        ExprKind::Literal(_) | ExprKind::FlowValue => {}
    }
    Ok(())
}

fn canonicalize_call_arg_josa(
    arg: &mut ArgBinding,
    func: &str,
    signatures: &HashMap<String, Vec<ParamPin>>,
) {
    let Some(josa) = arg.josa.as_ref() else {
        return;
    };
    let Some(pin) = arg.resolved_pin.as_deref() else {
        return;
    };
    let Some(params) = signatures.get(func) else {
        return;
    };
    let Some(param) = params.iter().find(|param| param.pin_name == pin) else {
        return;
    };
    if !param.josa_list.iter().any(|candidate| candidate == josa) {
        return;
    }
    if let Some(primary_josa) = param.josa_list.first() {
        arg.josa = Some(primary_josa.clone());
    }
}

fn lint_call_arg_josa_conflict(
    arg: &ArgBinding,
    func: &str,
    signatures: &HashMap<String, Vec<ParamPin>>,
    warnings: &mut Vec<LintWarning>,
) {
    if !matches!(arg.binding_reason, BindingReason::UserFixed) {
        return;
    }
    let Some(josa) = arg.josa.as_deref() else {
        return;
    };
    let Some(pin) = arg.resolved_pin.as_deref() else {
        return;
    };
    let Some(params) = signatures.get(func) else {
        return;
    };
    let matches: Vec<&ParamPin> = params
        .iter()
        .filter(|param| param.josa_list.iter().any(|candidate| candidate == josa))
        .collect();
    if matches.len() < 2 {
        return;
    }
    if !matches.iter().any(|param| param.pin_name == pin) {
        return;
    }
    warnings.push(LintWarning {
        code: "W_CALL_JOSA_CONFLICT_FIXED",
        span: arg.span,
        message: format!(
            "호출 `{func}`에서 조사 `{josa}`는 여러 핀에 걸칩니다. 현재 인자는 `값:핀` 고정으로 해석했습니다"
        ),
    });
}

fn canonicalize_type_ref(
    type_ref: &mut TypeRef,
    span: Span,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    match type_ref {
        TypeRef::Named(name) => {
            canonicalize_ident(name, span, warnings)?;
            if let Some(mapped) = canonical_type_alias(name.as_str()) {
                *name = mapped.to_string();
            }
        }
        TypeRef::Applied { name, args } => {
            canonicalize_ident(name, span, warnings)?;
            if let Some(mapped) = canonical_type_alias(name.as_str()) {
                *name = mapped.to_string();
            }
            for arg in args {
                canonicalize_type_ref(arg, span, warnings)?;
            }
        }
        TypeRef::Infer => {}
    }
    Ok(())
}

fn canonical_type_alias(name: &str) -> Option<&'static str> {
    match name.trim() {
        "셈수" | "셈수2" | "셈수4" | "셈수8" | "fixed64" | "sim_num" => Some("셈수"),
        "바른수" | "바른수1" | "바른수2" | "바른수4" | "바른수8" | "정수" | "int" | "int64" => {
            Some("바른수")
        }
        "큰바른수" | "bigint" | "big_int" => Some("큰바른수"),
        "나눔수" | "유리수" | "rational" | "ratio" | "frac" => Some("나눔수"),
        "곱수" | "factor" | "factorized" | "primepow" => Some("곱수"),
        "논" | "bool" | "boolean" => Some("참거짓"),
        "목록" | "list" => Some("차림"),
        "모둠" | "set" => Some("모음"),
        "그림표" | "map" => Some("짝맞춤"),
        "값꾸러미" | "pack" => Some("묶음"),
        _ => None,
    }
}

fn canonicalize_ident(
    name: &mut String,
    span: Span,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    if let Some(entry) = term_map::find_legacy_term(name.as_str()) {
        warnings.push(LintWarning {
            code: entry.code,
            span,
            message: format!(
                "TERM-LINT-01: 레거시 용어 '{}'는 '{}'를 권장합니다",
                entry.input, entry.canonical
            ),
        });
    }
    Ok(())
}

fn collect_known_seeds(program: &CanonProgram) -> HashSet<String> {
    let mut known = HashSet::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        known.insert(seed.canonical_name.clone());
    }
    for sig in minimal_stdlib_sigs() {
        known.insert(sig.name.to_string());
    }
    known
}

fn collect_seed_signatures(program: &CanonProgram) -> HashMap<String, Vec<ParamPin>> {
    let mut signatures = HashMap::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        signatures.insert(seed.canonical_name.clone(), seed.params.clone());
        for alias in seed_call_surface_aliases(&seed.canonical_name) {
            signatures.insert(alias, seed.params.clone());
        }
    }
    signatures
}

fn seed_call_surface_aliases(seed_name: &str) -> Vec<String> {
    let mut aliases = Vec::new();
    for tail in CALL_TAIL_SHORT_FORMS {
        aliases.push(format!("{seed_name}{tail}"));
    }
    aliases
}

fn collect_stdlib_names() -> HashSet<String> {
    let mut names = HashSet::new();
    for sig in minimal_stdlib_sigs() {
        names.insert(sig.name.to_string());
    }
    names
}

fn lint_tailless_calls(
    program: &CanonProgram,
    known_seeds: &HashSet<String>,
    stdlib_names: &HashSet<String>,
    warnings: &mut Vec<LintWarning>,
) {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            lint_tailless_body(body, known_seeds, stdlib_names, warnings);
        }
    }
}

fn lint_tailless_body(
    body: &Body,
    known_seeds: &HashSet<String>,
    stdlib_names: &HashSet<String>,
    warnings: &mut Vec<LintWarning>,
) {
    for stmt in &body.stmts {
        match stmt {
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    if let Some(value) = &item.value {
                        lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
                    }
                }
            }
            Stmt::Expr { expr, .. } => {
                if let ExprKind::Var(name) = &expr.kind {
                    if known_seeds.contains(name) && !stdlib_names.contains(name) {
                        warnings.push(LintWarning {
                            code: "E_CALL_TAIL_MISSING_STMT",
                            span: expr.span,
                            message: format!(
                                "호출 꼬리가 필요합니다. 예: '{}하기.' 또는 '{}기.'",
                                name, name
                            ),
                        });
                    }
                }
                lint_tailless_expr(expr, known_seeds, stdlib_names, warnings);
            }
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                lint_tailless_expr(expr, known_seeds, stdlib_names, warnings);
            }
            Stmt::MetaBlock { .. } => {}
            Stmt::Pragma { .. } => {}
            Stmt::Mutate { target, value, .. } => {
                lint_tailless_expr(target, known_seeds, stdlib_names, warnings);
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
            Stmt::Return { value, .. } => {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                lint_tailless_body(then_body, known_seeds, stdlib_names, warnings);
                if let Some(body) = else_body {
                    lint_tailless_body(body, known_seeds, stdlib_names, warnings);
                }
            }
            Stmt::Try { action, body, .. } => {
                lint_tailless_expr(action, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    lint_tailless_expr(&branch.condition, known_seeds, stdlib_names, warnings);
                    lint_tailless_body(&branch.body, known_seeds, stdlib_names, warnings);
                }
                lint_tailless_body(else_body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Repeat { body, .. } => {
                lint_tailless_body(body, known_seeds, stdlib_names, warnings)
            }
            Stmt::BeatBlock { body, .. } => {
                lint_tailless_body(body, known_seeds, stdlib_names, warnings)
            }
            Stmt::Hook { body, .. } => {
                lint_tailless_body(body, known_seeds, stdlib_names, warnings)
            }
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::While {
                condition, body, ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::ForEach { iterable, body, .. } => {
                lint_tailless_expr(iterable, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Quantifier { body, .. } => {
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                if let Some(body) = then_body {
                    lint_tailless_body(body, known_seeds, stdlib_names, warnings);
                }
                lint_tailless_body(else_body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Guard {
                condition, body, ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Break { .. } => {}
        }
    }
}

fn lint_tailless_expr(
    expr: &Expr,
    known_seeds: &HashSet<String>,
    stdlib_names: &HashSet<String>,
    warnings: &mut Vec<LintWarning>,
) {
    match &expr.kind {
        ExprKind::Call { args, func } => {
            if known_seeds.contains(func) && !stdlib_names.contains(func) && !has_call_tail(func) {
                warnings.push(LintWarning {
                    code: "E_CALL_TAIL_MISSING_AFTER_ARGS",
                    span: expr.span,
                    message: format!(
                        "호출 꼬리가 필요합니다. 예: '{}하기.' 또는 '{}기.'",
                        func, func
                    ),
                });
            }
            for arg in args {
                lint_tailless_expr(&arg.expr, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::FieldAccess { target, .. } => {
            lint_tailless_expr(target, known_seeds, stdlib_names, warnings)
        }
        ExprKind::Infix { left, right, .. } => {
            lint_tailless_expr(left, known_seeds, stdlib_names, warnings);
            lint_tailless_expr(right, known_seeds, stdlib_names, warnings);
        }
        ExprKind::Suffix { value, .. } => {
            lint_tailless_expr(value, known_seeds, stdlib_names, warnings)
        }
        ExprKind::Thunk(body) => lint_tailless_body(body, known_seeds, stdlib_names, warnings),
        ExprKind::Eval { thunk, .. } => {
            lint_tailless_expr(thunk, known_seeds, stdlib_names, warnings)
        }
        ExprKind::SeedLiteral { body, .. } => {
            lint_tailless_expr(body, known_seeds, stdlib_names, warnings)
        }
        ExprKind::Pipe { stages } => {
            for stage in stages {
                lint_tailless_expr(stage, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::TemplateRender { inject, .. } => {
            for (_, value) in inject {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::Nuance { expr, .. } => {
            lint_tailless_expr(expr, known_seeds, stdlib_names, warnings)
        }
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Assertion(_)
        | ExprKind::Formula(_)
        | ExprKind::Template(_)
        | ExprKind::StateMachine(_) => {}
    }
}

fn has_call_tail(name: &str) -> bool {
    let tails = ["기", "하기", "고", "하고", "면", "하면"];
    tails.iter().any(|tail| name.ends_with(tail))
}
