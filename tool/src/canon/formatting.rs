//! Canonical source formatting; exact output construction bodies unchanged.
use super::*;

pub(super) fn format_program(stmts: &[Stmt]) -> String {
    let mut out = String::new();
    for stmt in stmts {
        format_stmt(stmt, 0, &mut out);
    }
    out
}

pub(super) fn format_stmt(stmt: &Stmt, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    match stmt {
        Stmt::RootDecl { items, .. } => {
            out.push_str(&format!("{}채비 {{\n", pad));
            for item in items {
                out.push_str(&format_decl_item(item, indent + 1));
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Assign { target, value } => {
            out.push_str(&format!(
                "{}{} <- {}.\n",
                pad,
                format_path(target),
                format_expr(value)
            ));
        }
        Stmt::SeedDef {
            name,
            kind,
            params,
            body,
        } => {
            let params_text = format_params(params);
            out.push_str(&format!("{}{}{}:{} = {{\n", pad, params_text, name, kind));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Show { value } => {
            out.push_str(&format!("{}{} 보여주기.\n", pad, format_expr(value)));
        }
        Stmt::Inspect { value } => {
            out.push_str(&format!("{}{} 톺아보기.\n", pad, format_expr(value)));
        }
        Stmt::Boim { entries } => {
            let fields = entries
                .iter()
                .map(|entry| format!("{}: {}", entry.name, format_expr(&entry.value)))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{}({}) 내보임.\n", pad, fields));
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
        } => {
            if let Some(sender) = sender {
                out.push_str(&format!(
                    "{}({})의 {} ~~> {}.\n",
                    pad,
                    format_expr(sender),
                    format_expr(payload),
                    format_expr(receiver)
                ));
            } else {
                out.push_str(&format!(
                    "{}{} ~~> {}.\n",
                    pad,
                    format_expr(payload),
                    format_expr(receiver)
                ));
            }
        }
        Stmt::BogaeMadangBlock { body } => {
            out.push_str(&format!("{}보개마당 {{", pad));
            out.push_str(body);
            out.push_str("}.\n");
        }
        Stmt::ExecPolicyBlock { body } => {
            out.push_str(&format!("{}실행정책 {{", pad));
            out.push_str(body);
            out.push_str("}.\n");
        }
        Stmt::JjaimBlock { body } => {
            out.push_str(&format!("{}짜임 {{", pad));
            out.push_str(body);
            out.push_str("}.\n");
        }
        Stmt::Return { value } => {
            out.push_str(&format!("{}{} 되돌림.\n", pad, format_expr(value)));
        }
        Stmt::Expr { value } => {
            out.push_str(&format!("{}{}.\n", pad, format_expr(value)));
        }
        Stmt::BogaeDraw => {
            out.push_str(&format!("{}보개로 그려.\n", pad));
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            if matches!(condition.style, ConditionStyle::Current) {
                out.push_str(&format!("{}{{ {} }}이면 {{\n", pad, format_if_condition(condition)));
            } else {
                out.push_str(&format!("{}만약 {} 이라면 {{\n", pad, format_if_condition(condition)));
            }
            for stmt in then_body {
                format_stmt(stmt, indent + 1, out);
            }
            if let Some(body) = else_body {
                out.push_str(&format!("{}}} 아니면 {{\n", pad));
                for stmt in body {
                    format_stmt(stmt, indent + 1, out);
                }
                out.push_str(&format!("{}}}.\n", pad));
            } else {
                out.push_str(&format!("{}}}.\n", pad));
            }
        }
        Stmt::Contract {
            kind,
            mode,
            condition,
            then_body,
            else_body,
        } => {
            let keyword = match kind {
                ContractKind::Pre => "바탕으로",
                ContractKind::Post => "다짐하고",
            };
            let mode_suffix = match mode {
                ContractMode::Alert => "(알림)",
                ContractMode::Abort => "",
            };
            out.push_str(&format!(
                "{}{} {}{} 아니면 {{\n",
                pad,
                format_condition(condition),
                keyword,
                mode_suffix
            ));
            for stmt in else_body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}", pad));
            if let Some(body) = then_body {
                out.push_str(" 맞으면 {\n");
                for stmt in body {
                    format_stmt(stmt, indent + 1, out);
                }
                out.push_str(&format!("{}}}", pad));
            }
            out.push_str(".\n");
        }
        Stmt::Choose {
            branches,
            else_body,
            exhaustive,
        } => {
            out.push_str(&format!("{}고르기:\n", pad));
            for branch in branches {
                out.push_str(&format!(
                    "{}  {}: {{\n",
                    pad,
                    format_condition(&branch.condition)
                ));
                for stmt in &branch.body {
                    format_stmt(stmt, indent + 2, out);
                }
                out.push_str(&format!("{}  }}\n", pad));
            }
            if let Some(body) = else_body {
                out.push_str(&format!("{}  아니면: {{\n", pad));
                for stmt in body {
                    format_stmt(stmt, indent + 2, out);
                }
                out.push_str(&format!("{}  }}.\n", pad));
            } else if *exhaustive {
                out.push_str(&format!("{}  모든 경우 다룸.\n", pad));
            } else {
                out.push_str(&format!("{}  아니면: {{\n{}  }}.\n", pad, pad));
            }
        }
        Stmt::PromptChoose {
            branches,
            else_body,
        } => {
            out.push_str(&format!("{}??:\n", pad));
            for branch in branches {
                out.push_str(&format!(
                    "{}  {}: {{\n",
                    pad,
                    format_condition(&branch.condition)
                ));
                for stmt in &branch.body {
                    format_stmt(stmt, indent + 2, out);
                }
                out.push_str(&format!("{}  }}\n", pad));
            }
            if let Some(else_body) = else_body {
                out.push_str(&format!("{}  ??: {{\n", pad));
                for stmt in else_body {
                    format_stmt(stmt, indent + 2, out);
                }
                out.push_str(&format!("{}  }}\n", pad));
            }
        }
        Stmt::PromptAfter { value, body } => {
            out.push_str(&format!("{}{} ??: {{\n", pad, format_expr(value)));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::PromptCondition { condition, body } => {
            out.push_str(&format!("{}{} ??\n", pad, format_condition(condition)));
            out.push_str(&format!("{}?? {{\n", pad));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::PromptBlock { body } => {
            out.push_str(&format!("{}?? {{\n", pad));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::Repeat { body } => {
            out.push_str(&format!("{}되풀이 {{\n", pad));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::While { condition, body } => {
            out.push_str(&format!(
                "{}{{ {} }}인 동안 {{\n",
                pad,
                format_expr(&condition.expr)
            ));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::ForEach {
            item,
            iterable,
            body,
        } => {
            out.push_str(&format!(
                "{}({}) {}에 대해 {{\n",
                pad,
                item,
                format_expr(iterable)
            ));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Hook { name, suffix, body } => {
            let suffix_text = match suffix {
                HookSuffix::Halttae => "할때",
                HookSuffix::Mada => "마다",
            };
            out.push_str(&format!("{}({}){} {{\n", pad, name, suffix_text));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Receive {
            kind,
            binding,
            condition,
            body,
        } => {
            out.push_str(&pad);
            if let Some(binding) = binding {
                out.push('(');
                out.push_str(binding);
                if let Some(condition) = condition {
                    out.push(' ');
                    out.push_str(&format_expr(condition));
                }
                out.push_str(")인 ");
            }
            if let Some(kind) = kind {
                out.push_str(kind);
                out.push_str(object_particle(kind));
            } else {
                out.push_str("알림을");
            }
            out.push_str(" 받으면 {\n");
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::EventReact { kind, body } => {
            out.push_str(&format!(
                "{}\"{}\"라는 알림이 오면 {{\n",
                pad,
                escape_string(kind)
            ));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::OpenBlock { body } => {
            out.push_str(&format!("{}너머 {{\n", pad));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Break => {
            out.push_str(&format!("{}멈추기.\n", pad));
        }
        Stmt::ContinueLoop => {
            out.push_str(&format!("{}건너뛰기.\n", pad));
        }
    }
}

pub(super) fn format_decl_item(item: &DeclItem, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let mut out = String::new();
    out.push_str(&pad);
    out.push_str(&item.name);
    out.push(':');
    out.push_str(&item.type_name);
    if let Some(value) = &item.value {
        if item.kind == DeclKind::Butbak {
            out.push_str(" = ");
        } else {
            out.push_str(" <- ");
        }
        if let Some(maegim) = &item.maegim {
            out.push('(');
            out.push_str(&format_expr(value));
            out.push_str(") 매김 {\n");
            let mut emitted_ganeum = false;
            let mut emitted_gallae = false;
            for field in &maegim.fields {
                if field.name.starts_with("가늠.") {
                    if emitted_ganeum {
                        continue;
                    }
                    emitted_ganeum = true;
                    out.push_str(&pad);
                    out.push_str("  가늠 {\n");
                    for nested in &maegim.fields {
                        let Some(leaf) = nested.name.strip_prefix("가늠.") else {
                            continue;
                        };
                        out.push_str(&pad);
                        out.push_str("    ");
                        out.push_str(leaf);
                        out.push_str(": ");
                        out.push_str(&format_maegim_field_value(nested));
                        out.push_str(".\n");
                    }
                    out.push_str(&pad);
                    out.push_str("  }.\n");
                    continue;
                }
                if field.name.starts_with("갈래.") {
                    if emitted_gallae {
                        continue;
                    }
                    emitted_gallae = true;
                    out.push_str(&pad);
                    out.push_str("  갈래 {\n");
                    for nested in &maegim.fields {
                        let Some(leaf) = nested.name.strip_prefix("갈래.") else {
                            continue;
                        };
                        out.push_str(&pad);
                        out.push_str("    ");
                        out.push_str(leaf);
                        out.push_str(": ");
                        out.push_str(&format_maegim_field_value(nested));
                        out.push_str(".\n");
                    }
                    out.push_str(&pad);
                    out.push_str("  }.\n");
                    continue;
                }
                out.push_str(&pad);
                out.push_str("  ");
                out.push_str(&field.name);
                out.push_str(": ");
                out.push_str(&format_maegim_field_value(field));
                out.push_str(".\n");
            }
            out.push_str(&pad);
            out.push('}');
        } else {
            out.push_str(&format_expr(value));
        }
    }
    out.push_str(".\n");
    out
}

pub(super) fn format_expr(expr: &Expr) -> String {
    format_expr_prec(expr, 0)
}

pub(super) fn seed_literal_param_count(param: &str) -> usize {
    param
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .count()
        .max(1)
}

pub(super) fn maegim_field_leaf_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

pub(super) fn format_maegim_field_value(binding: &Binding) -> String {
    if maegim_field_leaf_name(&binding.name) == "범위" {
        if let Some(rendered) = try_format_range_expr(&binding.value) {
            return rendered;
        }
    }
    format_expr(&binding.value)
}

pub(super) fn try_format_range_expr(expr: &Expr) -> Option<String> {
    let Expr::Call { name, args, .. } = expr else {
        return None;
    };
    if name != "표준.범위" || args.len() != 3 {
        return None;
    }
    let left = format_expr(&args[0]);
    let right = format_expr(&args[1]);
    let inclusive = matches!(&args[2], Expr::Literal(Literal::Num(value)) if value == "1");
    Some(format!(
        "{}{}{}",
        left,
        if inclusive { "..=" } else { ".." },
        right
    ))
}

pub(super) fn format_params(params: &[Param]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push('(');
    for (idx, param) in params.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&param.name);
        if let Some(type_name) = &param.type_name {
            out.push(':');
            out.push_str(type_name);
        }
        if let Some(default) = &param.default {
            out.push_str(" = ");
            out.push_str(&format_expr(default));
        }
    }
    out.push(')');
    out.push(' ');
    out
}

pub(super) fn format_expr_prec(expr: &Expr, parent_prec: u8) -> String {
    match expr {
        Expr::Literal(literal) => format_literal(literal),
        Expr::Resource(path) => format!("@\"{}\"", escape_string(path)),
        Expr::Template { body } => format!("글무늬{{{}}}", body),
        Expr::PromptExpr { expr } => format!("??({})", format_expr(expr)),
        Expr::PromptBlock { body } => format!("??{{{}}}", body),
        Expr::TemplateApply { bindings, body } => {
            let mut rendered = String::new();
            rendered.push('(');
            for (idx, binding) in bindings.iter().enumerate() {
                if idx > 0 {
                    rendered.push_str(", ");
                }
                rendered.push_str(&binding.name);
                rendered.push('=');
                rendered.push_str(&format_expr(&binding.value));
            }
            rendered.push(')');
            rendered.push(' ');
            rendered.push_str("글무늬");
            rendered.push('{');
            rendered.push_str(body);
            rendered.push('}');
            rendered
        }
        Expr::Formula { tag, body } => {
            let mut rendered = String::new();
            if let Some(tag) = tag {
                rendered.push('(');
                rendered.push('#');
                rendered.push_str(tag);
                rendered.push_str(") ");
            }
            rendered.push_str("수식");
            rendered.push('{');
            rendered.push_str(body);
            rendered.push('}');
            rendered
        }
        Expr::Path(path) => format_path(path),
        Expr::Call { name, args } => {
            let prec = 6;
            let rendered = if name == "차림.값" && args.len() == 2 {
                format!(
                    "(대상={}, i={}) {}",
                    format_expr(&args[0]),
                    format_expr(&args[1]),
                    name
                )
            } else if name == "차림.바꾼값" && args.len() == 3 {
                format!(
                    "(대상={}, i={}, 값={}) {}",
                    format_expr(&args[0]),
                    format_expr(&args[1]),
                    format_expr(&args[2]),
                    name
                )
            } else if args.len() == 1 {
                if let Expr::Pack { bindings } = &args[0] {
                    let mut rendered = String::new();
                    rendered.push('(');
                    for (idx, binding) in bindings.iter().enumerate() {
                        if idx > 0 {
                            rendered.push_str(", ");
                        }
                        rendered.push_str(&binding.name);
                        rendered.push(':');
                        rendered.push_str(&format_expr(&binding.value));
                    }
                    rendered.push(')');
                    rendered.push(' ');
                    rendered.push_str(name);
                    rendered
                } else {
                    let mut rendered = String::new();
                    rendered.push('(');
                    for (idx, arg) in args.iter().enumerate() {
                        if idx > 0 {
                            rendered.push_str(", ");
                        }
                        rendered.push_str(&format_expr(arg));
                    }
                    rendered.push(')');
                    rendered.push(' ');
                    rendered.push_str(name);
                    rendered
                }
            } else {
                let mut rendered = String::new();
                rendered.push('(');
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        rendered.push_str(", ");
                    }
                    rendered.push_str(&format_expr(arg));
                }
                rendered.push(')');
                rendered.push(' ');
                rendered.push_str(name);
                rendered
            };
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::CallIn { name, bindings } => {
            let prec = 6;
            let mut rendered = String::new();
            rendered.push('(');
            for (idx, binding) in bindings.iter().enumerate() {
                if idx > 0 {
                    rendered.push_str(", ");
                }
                rendered.push_str(&binding.name);
                rendered.push('=');
                rendered.push_str(&format_expr(&binding.value));
            }
            rendered.push(')');
            rendered.push_str("인 ");
            rendered.push_str(name);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::FieldAccess { target, field } => {
            let prec = 6;
            let rendered = format!("{}.{}", format_expr_prec(target, prec), field);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Pack { bindings } => {
            let prec = 6;
            let mut rendered = String::new();
            rendered.push('(');
            for (idx, binding) in bindings.iter().enumerate() {
                if idx > 0 {
                    rendered.push_str(", ");
                }
                rendered.push_str(&binding.name);
                rendered.push('=');
                rendered.push_str(&format_expr(&binding.value));
            }
            rendered.push(')');
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Unit { value, unit } => {
            let prec = 6;
            let rendered = format!("{}@{}", format_expr_prec(value, prec), unit);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Unary { op, expr } => {
            let prec = 5;
            let inner = format_expr_prec(expr, prec);
            let rendered = match op {
                UnaryOp::Neg => format!("-{}", inner),
            };
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Binary { left, op, right } => {
            let (prec, op_text) = match op {
                BinaryOp::Or => (0, "또는"),
                BinaryOp::And => (1, "그리고"),
                BinaryOp::RelationEq => (2, "=:="),
                BinaryOp::Eq => (2, "=="),
                BinaryOp::NotEq => (2, "!="),
                BinaryOp::Lt => (2, "<"),
                BinaryOp::Lte => (2, "<="),
                BinaryOp::Gt => (2, ">"),
                BinaryOp::Gte => (2, ">="),
                BinaryOp::Add => (3, "+"),
                BinaryOp::Sub => (3, "-"),
                BinaryOp::Mul => (4, "*"),
                BinaryOp::Div => (4, "/"),
                BinaryOp::Mod => (4, "%"),
            };
            let left_text = format_expr_prec(left, prec);
            let right_text = format_expr_prec(right, prec);
            let rendered = format!("{} {} {}", left_text, op_text, right_text);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Pipe { left, kind, right } => {
            let prec = 0;
            let op_text = match kind {
                PipeKind::Haseo => "해서",
                PipeKind::Hago => "하고",
            };
            let left_text = format_expr_prec(left, prec);
            let right_text = format_expr_prec(right, prec);
            let rendered = format!("{} {} {}", left_text, op_text, right_text);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::SeedLiteral { param, body } => {
            let prec = 6;
            let rendered_param = if seed_literal_param_count(param) > 1 {
                format!("({param})")
            } else {
                param.clone()
            };
            let rendered = format!("{{{} | {}}}", rendered_param, format_expr(body));
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Grouped { expr } => format!("({})", format_expr(expr)),
    }
}

pub(super) fn format_condition(condition: &Condition) -> String {
    let expr = format_expr(&condition.expr);
    match condition.style {
        ConditionStyle::Current => format!("{{ {} }}", expr),
        ConditionStyle::Plain => expr,
        ConditionStyle::Thunk => {
            let suffix = if condition.negated {
                "아닌것"
            } else {
                "인것"
            };
            format!("{{ {} }}{}", expr, suffix)
        }
    }
}

pub(super) fn format_if_condition(condition: &Condition) -> String {
    let expr = format_expr(&condition.expr);
    if condition.negated {
        return format!("({}) 아님", expr);
    }
    expr
}

pub(super) fn format_literal(literal: &Literal) -> String {
    match literal {
        Literal::Str(text) => format!("\"{}\"", escape_string(text)),
        Literal::Num(value) => value.clone(),
        Literal::Bool(true) => "참".to_string(),
        Literal::Bool(false) => "거짓".to_string(),
        Literal::None => "없음".to_string(),
        Literal::Atom(text) => format!("#{}", text),
    }
}

pub(super) fn format_path(path: &Path) -> String {
    path.segments.join(".")
}

pub(super) fn escape_string(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}
