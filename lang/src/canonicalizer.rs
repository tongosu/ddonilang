use crate::ast::*;
use crate::parser::ParseError;
use crate::stdlib::minimal_stdlib_sigs;
use crate::term_map;
use std::collections::HashSet;

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
    for item in &mut program.items {
        canonicalize_top_level_item(item, &mut warnings)?;
    }
    let known_seeds = collect_known_seeds(program);
    let stdlib_names = collect_stdlib_names();
    lint_tailless_calls(program, &known_seeds, &stdlib_names, &mut warnings);
    Ok(CanonicalizeReport { warnings })
}

fn canonicalize_top_level_item(
    item: &mut TopLevelItem,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    match item {
        TopLevelItem::SeedDef(seed) => canonicalize_seed_def(seed, warnings),
    }
}

fn canonicalize_seed_def(
    seed: &mut SeedDef,
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
            canonicalize_expr(default_value, warnings)?;
        }
    }
    if let Some(body) = &mut seed.body {
        canonicalize_body(body, warnings)?;
    }
    Ok(())
}

fn canonicalize_body(body: &mut Body, warnings: &mut Vec<LintWarning>) -> Result<(), ParseError> {
    for stmt in &mut body.stmts {
        canonicalize_stmt(stmt, warnings)?;
    }
    Ok(())
}

fn canonicalize_stmt(stmt: &mut Stmt, warnings: &mut Vec<LintWarning>) -> Result<(), ParseError> {
    match stmt {
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                canonicalize_ident(&mut item.name, item.span, warnings)?;
                canonicalize_type_ref(&mut item.type_ref, item.span, warnings)?;
                if let Some(value) = &mut item.value {
                    canonicalize_expr(value, warnings)?;
                }
            }
        }
        Stmt::Mutate { target, value, .. } => {
            canonicalize_expr(target, warnings)?;
            canonicalize_expr(value, warnings)?;
        }
        Stmt::Expr { expr, .. } => canonicalize_expr(expr, warnings)?,
        Stmt::Pragma { .. } => {}
        Stmt::Return { value, .. } => canonicalize_expr(value, warnings)?,
        Stmt::If { condition, then_body, else_body, .. } => {
            canonicalize_expr(condition, warnings)?;
            canonicalize_body(then_body, warnings)?;
            if let Some(body) = else_body {
                canonicalize_body(body, warnings)?;
            }
        }
        Stmt::Try { action, body, .. } => {
            canonicalize_expr(action, warnings)?;
            canonicalize_body(body, warnings)?;
        }
        Stmt::Choose { branches, else_body, .. } => {
            for branch in branches {
                canonicalize_expr(&mut branch.condition, warnings)?;
                canonicalize_body(&mut branch.body, warnings)?;
            }
            canonicalize_body(else_body, warnings)?;
        }
        Stmt::Repeat { body, .. } => {
            canonicalize_body(body, warnings)?;
        }
        Stmt::While { condition, body, .. } => {
            canonicalize_expr(condition, warnings)?;
            canonicalize_body(body, warnings)?;
        }
        Stmt::ForEach { item, item_type, iterable, body, span, .. } => {
            canonicalize_ident(item, *span, warnings)?;
            if let Some(type_ref) = item_type.as_mut() {
                canonicalize_type_ref(type_ref, *span, warnings)?;
            }
            canonicalize_expr(iterable, warnings)?;
            canonicalize_body(body, warnings)?;
        }
        Stmt::Break { .. } => {}
        Stmt::Contract { condition, then_body, else_body, .. } => {
            canonicalize_expr(condition, warnings)?;
            if let Some(body) = then_body {
                canonicalize_body(body, warnings)?;
            }
            canonicalize_body(else_body, warnings)?;
        }
        Stmt::Guard { condition, body, .. } => {
            canonicalize_expr(condition, warnings)?;
            canonicalize_body(body, warnings)?;
        }
    }
    Ok(())
}

fn canonicalize_expr(expr: &mut Expr, warnings: &mut Vec<LintWarning>) -> Result<(), ParseError> {
    match &mut expr.kind {
        ExprKind::Var(name) => canonicalize_ident(name, expr.span, warnings)?,
        ExprKind::FieldAccess { target, field } => {
            canonicalize_expr(target, warnings)?;
            canonicalize_ident(field, expr.span, warnings)?;
        }
        ExprKind::Call { args, func } => {
            canonicalize_ident(func, expr.span, warnings)?;
            for arg in args {
                canonicalize_expr(&mut arg.expr, warnings)?;
                if let Some(pin) = &mut arg.resolved_pin {
                    canonicalize_ident(pin, arg.span, warnings)?;
                }
            }
        }
        ExprKind::Infix { left, right, .. } => {
            canonicalize_expr(left, warnings)?;
            canonicalize_expr(right, warnings)?;
        }
        ExprKind::Suffix { value, .. } => canonicalize_expr(value, warnings)?,
        ExprKind::Thunk(body) => canonicalize_body(body, warnings)?,
        ExprKind::Eval { thunk, .. } => canonicalize_expr(thunk, warnings)?,
        ExprKind::SeedLiteral { param, body } => {
            canonicalize_ident(param, expr.span, warnings)?;
            canonicalize_expr(body, warnings)?;
        }
        ExprKind::Pipe { stages } => {
            for stage in stages {
                canonicalize_expr(stage, warnings)?;
            }
        }
        ExprKind::Pack { fields } => {
            for (name, value) in fields {
                canonicalize_ident(name, expr.span, warnings)?;
                canonicalize_expr(value, warnings)?;
            }
        }
        ExprKind::Formula(_) => {}
        ExprKind::Template(_) => {}
        ExprKind::TemplateRender { inject, .. } => {
            for (_, value) in inject {
                canonicalize_expr(value, warnings)?;
            }
        }
        ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                canonicalize_expr(value, warnings)?;
            }
        }
        ExprKind::Nuance { expr, .. } => canonicalize_expr(expr, warnings)?,
        ExprKind::Literal(_) | ExprKind::FlowValue => {}
    }
    Ok(())
}

fn canonicalize_type_ref(
    type_ref: &mut TypeRef,
    span: Span,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    match type_ref {
        TypeRef::Named(name) => canonicalize_ident(name, span, warnings)?,
        TypeRef::Applied { name, args } => {
            canonicalize_ident(name, span, warnings)?;
            for arg in args {
                canonicalize_type_ref(arg, span, warnings)?;
            }
        }
        TypeRef::Infer => {}
    }
    Ok(())
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
            Stmt::Pragma { .. } => {}
            Stmt::Mutate { target, value, .. } => {
                lint_tailless_expr(target, known_seeds, stdlib_names, warnings);
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
            Stmt::Return { value, .. } => {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
            Stmt::If { condition, then_body, else_body, .. } => {
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
            Stmt::Choose { branches, else_body, .. } => {
                for branch in branches {
                    lint_tailless_expr(&branch.condition, known_seeds, stdlib_names, warnings);
                    lint_tailless_body(&branch.body, known_seeds, stdlib_names, warnings);
                }
                lint_tailless_body(else_body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Repeat { body, .. } => lint_tailless_body(body, known_seeds, stdlib_names, warnings),
            Stmt::While { condition, body, .. } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::ForEach { iterable, body, .. } => {
                lint_tailless_expr(iterable, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Contract { condition, then_body, else_body, .. } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                if let Some(body) = then_body {
                    lint_tailless_body(body, known_seeds, stdlib_names, warnings);
                }
                lint_tailless_body(else_body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Guard { condition, body, .. } => {
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
            if known_seeds.contains(func)
                && !stdlib_names.contains(func)
                && !has_call_tail(func)
            {
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
        ExprKind::Suffix { value, .. } => lint_tailless_expr(value, known_seeds, stdlib_names, warnings),
        ExprKind::Thunk(body) => lint_tailless_body(body, known_seeds, stdlib_names, warnings),
        ExprKind::Eval { thunk, .. } => lint_tailless_expr(thunk, known_seeds, stdlib_names, warnings),
        ExprKind::SeedLiteral { body, .. } => lint_tailless_expr(body, known_seeds, stdlib_names, warnings),
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
        ExprKind::Nuance { expr, .. } => lint_tailless_expr(expr, known_seeds, stdlib_names, warnings),
        ExprKind::Literal(_) | ExprKind::Var(_) | ExprKind::FlowValue | ExprKind::Formula(_) | ExprKind::Template(_) => {}
    }
}

fn has_call_tail(name: &str) -> bool {
    let tails = ["기", "하기", "고", "하고", "면", "하면"];
    tails.iter().any(|tail| name.ends_with(tail))
}
