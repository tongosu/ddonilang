use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use ddonirang_lang::ast::{
    ArgBinding, BindingReason, Body, Expr, ExprKind, Literal, SeedDef, SeedKind, Stmt,
};
use ddonirang_lang::{
    canonicalize, normalize, parse, LintWarning, NormalizationLevel, TopLevelItem,
};

struct CallDet {
    func: String,
    args: Vec<ArgDet>,
}

struct ArgDet {
    resolved_pin: Option<String>,
    josa: Option<String>,
    binding_reason: String,
    expr: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args();
    let _exe = args.next();
    let input_arg = args.next().ok_or_else(|| {
        "사용법: cargo run -q -p ddonirang-lang --example canon_ast_dump -- <input.ddn>".to_string()
    })?;
    let mut out_artifacts_dir: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out-artifacts" => {
                let out_dir = args
                    .next()
                    .ok_or_else(|| "--out-artifacts 뒤에 출력 디렉터리가 필요합니다.".to_string())?;
                out_artifacts_dir = Some(PathBuf::from(out_dir));
            }
            _ => {
                return Err(
                    "사용법: cargo run -q -p ddonirang-lang --example canon_ast_dump -- <input.ddn> [--out-artifacts <dir>]".to_string(),
                )
            }
        }
    }

    let input_path = Path::new(&input_arg);
    let source = fs::read_to_string(input_path)
        .map_err(|e| format!("E_READ_INPUT {}: {}", input_path.display(), e))?;
    let file_label = input_path.to_string_lossy().replace('\\', "/");

    let mut program = parse(&source, &file_label)
        .map_err(|e| format!("{} {}: {}", e.code(), file_label, e.message))?;
    let report = canonicalize(&mut program).map_err(|e| format!("E_CANON {}", e.message))?;
    let normalized_n1 = normalize(&program, NormalizationLevel::N1);

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"schema\": ");
    push_json_string(&mut out, "ddn.canon_ast.detjson.v1");
    out.push_str(",\n  \"normalized_n1\": ");
    push_json_string(&mut out, &normalized_n1);
    out.push_str(",\n  \"warnings\": [\n");
    write_warnings_detjson(&mut out, &report.warnings, 4)?;
    out.push_str("\n  ],\n  \"seeds\": [\n");

    let mut first_seed = true;
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if !first_seed {
            out.push_str(",\n");
        }
        first_seed = false;
        write_seed_detjson(&mut out, seed, 4)?;
    }
    out.push_str("\n  ]\n}\n");
    if let Some(out_dir) = out_artifacts_dir {
        write_artifacts(&out_dir, &normalized_n1, &out)?;
    }
    print!("{}", out);
    Ok(())
}

fn write_warnings_detjson(
    out: &mut String,
    warnings: &[LintWarning],
    indent: usize,
) -> Result<(), String> {
    let pad = " ".repeat(indent);
    for (idx, warning) in warnings.iter().enumerate() {
        if idx > 0 {
            out.push_str(",\n");
        }
        out.push_str(&pad);
        out.push('{');
        out.push_str("\"code\": ");
        push_json_string(out, warning.code);
        out.push_str(", \"message\": ");
        push_json_string(out, &warning.message);
        out.push('}');
    }
    Ok(())
}

fn write_artifacts(out_dir: &Path, canon_text: &str, ast_detjson: &str) -> Result<(), String> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("E_WRITE_ARTIFACT_DIR {}: {}", out_dir.display(), e))?;
    let canon_path = out_dir.join("canon.ddn");
    let ast_path = out_dir.join("ast.detjson");
    let hash_path = out_dir.join("ast_hash.txt");
    let canon_body = if canon_text.ends_with('\n') {
        canon_text.to_string()
    } else {
        format!("{canon_text}\n")
    };
    fs::write(&canon_path, canon_body)
        .map_err(|e| format!("E_WRITE_CANON {}: {}", canon_path.display(), e))?;
    fs::write(&ast_path, ast_detjson)
        .map_err(|e| format!("E_WRITE_AST {}: {}", ast_path.display(), e))?;
    let ast_hash = format!("blake3:{}\n", blake3::hash(ast_detjson.as_bytes()).to_hex());
    fs::write(&hash_path, ast_hash)
        .map_err(|e| format!("E_WRITE_AST_HASH {}: {}", hash_path.display(), e))?;
    Ok(())
}

fn write_seed_detjson(out: &mut String, seed: &SeedDef, indent: usize) -> Result<(), String> {
    let mut calls = Vec::new();
    if let Some(body) = &seed.body {
        collect_calls_from_body(body, &mut calls);
    }

    let pad = " ".repeat(indent);
    let pad2 = " ".repeat(indent + 2);
    let pad3 = " ".repeat(indent + 4);
    let pad4 = " ".repeat(indent + 6);

    out.push_str(&pad);
    out.push_str("{\n");
    out.push_str(&pad2);
    out.push_str("\"name\": ");
    push_json_string(out, &seed.canonical_name);
    out.push_str(",\n");
    out.push_str(&pad2);
    out.push_str("\"kind\": ");
    push_json_string(out, seed_kind_label(&seed.seed_kind));
    out.push_str(",\n");
    out.push_str(&pad2);
    out.push_str("\"params\": [\n");
    for (idx, param) in seed.params.iter().enumerate() {
        if idx > 0 {
            out.push_str(",\n");
        }
        out.push_str(&pad3);
        out.push('{');
        out.push_str("\"pin\": ");
        push_json_string(out, &param.pin_name);
        out.push_str(", \"josa_list\": [");
        for (jdx, josa) in param.josa_list.iter().enumerate() {
            if jdx > 0 {
                out.push_str(", ");
            }
            push_json_string(out, josa);
        }
        out.push_str("]}");
    }
    out.push_str("\n");
    out.push_str(&pad2);
    out.push_str("],\n");
    out.push_str(&pad2);
    out.push_str("\"calls\": [\n");
    for (idx, call) in calls.iter().enumerate() {
        if idx > 0 {
            out.push_str(",\n");
        }
        out.push_str(&pad3);
        out.push_str("{\n");
        out.push_str(&pad4);
        out.push_str("\"func\": ");
        push_json_string(out, &call.func);
        out.push_str(",\n");
        out.push_str(&pad4);
        out.push_str("\"args\": [");
        for (adx, arg) in call.args.iter().enumerate() {
            if adx > 0 {
                out.push_str(", ");
            }
            out.push('{');
            out.push_str("\"resolved_pin\": ");
            push_json_opt_string(out, arg.resolved_pin.as_deref());
            out.push_str(", \"josa\": ");
            push_json_opt_string(out, arg.josa.as_deref());
            out.push_str(", \"binding_reason\": ");
            push_json_string(out, &arg.binding_reason);
            out.push_str(", \"expr\": ");
            push_json_string(out, &arg.expr);
            out.push('}');
        }
        out.push_str("]\n");
        out.push_str(&pad3);
        out.push('}');
    }
    out.push_str("\n");
    out.push_str(&pad2);
    out.push_str("],\n");
    out.push_str(&pad2);
    out.push_str("\"contract_count\": ");
    write!(out, "{}", count_contracts(seed)).map_err(|e| e.to_string())?;
    out.push_str("\n");
    out.push_str(&pad);
    out.push('}');
    Ok(())
}

fn count_contracts(seed: &SeedDef) -> usize {
    let Some(body) = seed.body.as_ref() else {
        return 0;
    };
    count_contracts_in_body(body)
}

fn count_contracts_in_body(body: &Body) -> usize {
    let mut count = 0usize;
    for stmt in &body.stmts {
        match stmt {
            Stmt::Contract { .. } => count += 1,
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                count += count_contracts_in_body(then_body);
                if let Some(else_body) = else_body {
                    count += count_contracts_in_body(else_body);
                }
            }
            Stmt::Try { body, .. }
            | Stmt::Repeat { body, .. }
            | Stmt::While { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::Guard { body, .. } => {
                count += count_contracts_in_body(body);
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    count += count_contracts_in_body(&branch.body);
                }
                count += count_contracts_in_body(else_body);
            }
            _ => {}
        }
    }
    count
}

fn collect_calls_from_body(body: &Body, out: &mut Vec<CallDet>) {
    for stmt in &body.stmts {
        collect_calls_from_stmt(stmt, out);
    }
}

fn collect_calls_from_stmt(stmt: &Stmt, out: &mut Vec<CallDet>) {
    match stmt {
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    collect_calls_from_expr(value, out);
                }
            }
        }
        Stmt::Mutate { target, value, .. } => {
            collect_calls_from_expr(target, out);
            collect_calls_from_expr(value, out);
        }
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            collect_calls_from_expr(expr, out)
        }
        Stmt::Return { value, .. } => collect_calls_from_expr(value, out),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_calls_from_expr(condition, out);
            collect_calls_from_body(then_body, out);
            if let Some(else_body) = else_body {
                collect_calls_from_body(else_body, out);
            }
        }
        Stmt::Try { action, body, .. } => {
            collect_calls_from_expr(action, out);
            collect_calls_from_body(body, out);
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                collect_calls_from_expr(&branch.condition, out);
                collect_calls_from_body(&branch.body, out);
            }
            collect_calls_from_body(else_body, out);
        }
        Stmt::Repeat { body, .. }
        | Stmt::While { body, .. }
        | Stmt::ForEach { body, .. }
        | Stmt::Quantifier { body, .. }
        | Stmt::Guard { body, .. } => collect_calls_from_body(body, out),
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_calls_from_expr(condition, out);
            if let Some(then_body) = then_body {
                collect_calls_from_body(then_body, out);
            }
            collect_calls_from_body(else_body, out);
        }
        Stmt::Pragma { .. } | Stmt::MetaBlock { .. } | Stmt::Break { .. } => {}
    }
}

fn collect_calls_from_expr(expr: &Expr, out: &mut Vec<CallDet>) {
    match &expr.kind {
        ExprKind::Call { args, func } => {
            let call = CallDet {
                func: func.clone(),
                args: args.iter().map(arg_to_det).collect(),
            };
            out.push(call);
            for arg in args {
                collect_calls_from_expr(&arg.expr, out);
            }
        }
        ExprKind::FieldAccess { target, .. } => collect_calls_from_expr(target, out),
        ExprKind::SeedLiteral { body, .. } => collect_calls_from_expr(body, out),
        ExprKind::Infix { left, right, .. } => {
            collect_calls_from_expr(left, out);
            collect_calls_from_expr(right, out);
        }
        ExprKind::Suffix { value, .. } => collect_calls_from_expr(value, out),
        ExprKind::Thunk(body) => collect_calls_from_body(body, out),
        ExprKind::Eval { thunk, .. } => collect_calls_from_expr(thunk, out),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                collect_calls_from_expr(stage, out);
            }
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                collect_calls_from_expr(value, out);
            }
        }
        ExprKind::Assertion(_) | ExprKind::StateMachine(_) => {}
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                collect_calls_from_expr(value, out);
            }
        }
        ExprKind::Nuance { expr, .. } => collect_calls_from_expr(expr, out),
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Formula(_)
        | ExprKind::Template(_) => {}
    }
}

fn arg_to_det(arg: &ArgBinding) -> ArgDet {
    ArgDet {
        resolved_pin: arg.resolved_pin.clone(),
        josa: arg.josa.clone(),
        binding_reason: binding_reason_label(&arg.binding_reason).to_string(),
        expr: expr_fingerprint(&arg.expr),
    }
}

fn seed_kind_label(kind: &SeedKind) -> &str {
    match kind {
        SeedKind::Imeumssi => "이음씨",
        SeedKind::Umjikssi => "움직씨",
        SeedKind::ValueFunc => "값함수",
        SeedKind::Gallaessi => "갈래씨",
        SeedKind::Relationssi => "관계씨",
        SeedKind::Sam => "샘",
        SeedKind::Heureumssi => "흐름씨",
        SeedKind::Ieumssi => "이음씨",
        SeedKind::Semssi => "셈씨",
        SeedKind::Named(_) => "이름씨",
    }
}

fn binding_reason_label(reason: &BindingReason) -> &str {
    match reason {
        BindingReason::Dictionary => "dictionary",
        BindingReason::Positional => "positional",
        BindingReason::UserFixed => "user_fixed",
        BindingReason::FlowInjected => "flow_injected",
        BindingReason::Ambiguous { .. } => "ambiguous",
    }
}

fn expr_fingerprint(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Literal(lit) => literal_fingerprint(lit),
        ExprKind::Var(name) => format!("var:{}", name),
        ExprKind::FieldAccess { field, .. } => format!("field:{}", field),
        ExprKind::SeedLiteral { param, .. } => format!("seed_literal:{}", param),
        ExprKind::Call { func, .. } => format!("call:{}", func),
        ExprKind::Infix { op, .. } => format!("infix:{}", op),
        ExprKind::Suffix { at, .. } => format!("suffix:{:?}", at),
        ExprKind::Thunk(_) => "thunk".to_string(),
        ExprKind::Eval { mode, .. } => format!("eval:{:?}", mode),
        ExprKind::Pipe { stages } => format!("pipe:{}", stages.len()),
        ExprKind::FlowValue => "flow".to_string(),
        ExprKind::Pack { fields } => format!("pack:{}", fields.len()),
        ExprKind::Formula(formula) => format!("formula:{}:{:?}", formula.raw, formula.dialect),
        ExprKind::Template(template) => {
            format!("template:{}:{}", template.raw, template.parts.len())
        }
        ExprKind::Assertion(assertion) => format!("assertion:{}", assertion.canon),
        ExprKind::StateMachine(machine) => {
            format!(
                "state_machine:{}:{}",
                machine.initial,
                machine.transitions.len()
            )
        }
        ExprKind::TemplateRender { inject, .. } => format!("template_render:{}", inject.len()),
        ExprKind::FormulaEval { inject, .. } => format!("formula_eval:{}", inject.len()),
        ExprKind::Nuance { level, .. } => format!("nuance:{}", level),
    }
}

fn literal_fingerprint(lit: &Literal) -> String {
    match lit {
        Literal::Int(value) => format!("int:{}", value),
        Literal::Fixed64(value) => format!("fixed64:{}", value),
        Literal::String(value) => format!("string:{}", value),
        Literal::Bool(value) => format!("bool:{}", value),
        Literal::Atom(value) => format!("atom:{}", value),
        Literal::Regex(value) => format!("regex:{}/{}", value.pattern, value.flags),
        Literal::Resource(value) => format!("resource:{}", value),
        Literal::None => "none".to_string(),
    }
}

fn push_json_opt_string(out: &mut String, value: Option<&str>) {
    match value {
        Some(value) => push_json_string(out, value),
        None => out.push_str("null"),
    }
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            ch if ch.is_control() => {
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            _ => out.push(ch),
        }
    }
    out.push('"');
}
