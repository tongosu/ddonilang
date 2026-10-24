//! Existing block-editor plan construction and rendering; original source bodies preserved.
use super::*;

pub(super) fn build_block_editor_plan(stmts: &[Stmt]) -> BlockEditorPlan {
    let mut blocks = Vec::new();
    for stmt in stmts {
        blocks.push(build_block_editor_plan_stmt(stmt));
    }
    BlockEditorPlan { blocks }
}

pub(super) fn build_block_editor_plan_stmt(stmt: &Stmt) -> BlockEditorPlanNode {
    match stmt {
        Stmt::RootDecl { items } => {
            let mut node = block_plan_node("charim_block");
            node.inputs.insert(
                "items".to_string(),
                items
                    .iter()
                    .map(build_block_editor_plan_decl_item)
                    .collect(),
            );
            node
        }
        Stmt::SeedDef {
            name,
            kind,
            params,
            body,
        } => {
            let mut node = block_plan_node("seed_def");
            node.fields.insert("name".to_string(), name.clone());
            node.fields.insert("kind".to_string(), kind.clone());
            node.fields
                .insert("params".to_string(), format_params(params));
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::BogaeMadangBlock { body } => {
            let mut node = block_plan_node("bogae_madang_block");
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Stmt::ExecPolicyBlock { body } => {
            let mut node = block_plan_node("exec_policy_block");
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Stmt::JjaimBlock { body } => {
            let mut node = block_plan_node("jjaim_block");
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Stmt::Expr { value } => {
            let mut node = block_plan_node("expr_stmt");
            node.fields.insert("expr".to_string(), format_expr(value));
            node.exprs
                .insert("expr".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::Assign { target, value } => {
            let mut node = block_plan_node("assign");
            node.fields
                .insert("target".to_string(), format_path(target));
            node.fields.insert("value".to_string(), format_expr(value));
            node.exprs
                .insert("value".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::Show { value } => {
            let mut node = block_plan_node("show");
            node.fields.insert("expr".to_string(), format_expr(value));
            node.exprs
                .insert("expr".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::Inspect { value } => {
            let mut node = block_plan_node("inspect_value");
            node.fields.insert("expr".to_string(), format_expr(value));
            node.exprs
                .insert("expr".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let kind = if else_body.is_some() {
                "if_else"
            } else {
                "if_then"
            };
            let mut node = block_plan_node(kind);
            node.fields
                .insert("cond".to_string(), format_condition(condition));
            node.exprs.insert(
                "cond".to_string(),
                build_block_editor_expr_node(&condition.expr),
            );
            node.inputs.insert(
                "then".to_string(),
                then_body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            if let Some(body) = else_body {
                node.inputs.insert(
                    "else".to_string(),
                    body.iter().map(build_block_editor_plan_stmt).collect(),
                );
            }
            node
        }
        Stmt::Repeat { body } => {
            let mut node = block_plan_node("repeat");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::While { condition, body } => {
            let mut node = block_plan_node("while_block");
            node.fields
                .insert("cond".to_string(), format_condition(condition));
            node.exprs.insert(
                "cond".to_string(),
                build_block_editor_expr_node(&condition.expr),
            );
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::ForEach {
            item,
            iterable,
            body,
        } => {
            let mut node = block_plan_node("for_each");
            node.fields.insert("item".to_string(), item.clone());
            node.fields
                .insert("iterable".to_string(), format_expr(iterable));
            node.exprs.insert(
                "iterable".to_string(),
                build_block_editor_expr_node(iterable),
            );
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Choose {
            branches,
            else_body,
            exhaustive,
        } => {
            let mut node = block_plan_node(if else_body.is_some() {
                "choose_else"
            } else {
                "choose_exhaustive"
            });
            node.inputs.insert(
                "branches".to_string(),
                branches
                    .iter()
                    .map(build_block_editor_plan_choose_branch)
                    .collect(),
            );
            node.fields
                .insert("exhaustive".to_string(), exhaustive.to_string());
            if let Some(body) = else_body {
                node.inputs.insert(
                    "else".to_string(),
                    body.iter().map(build_block_editor_plan_stmt).collect(),
                );
            }
            node
        }
        Stmt::PromptChoose {
            branches,
            else_body,
        } => {
            let mut node = block_plan_node("prompt_choose");
            node.inputs.insert(
                "branches".to_string(),
                branches
                    .iter()
                    .map(build_block_editor_plan_choose_branch)
                    .collect(),
            );
            if let Some(body) = else_body {
                node.inputs.insert(
                    "else".to_string(),
                    body.iter().map(build_block_editor_plan_stmt).collect(),
                );
            }
            node
        }
        Stmt::PromptAfter { value, body } => {
            let mut node = block_plan_node("prompt_after");
            node.fields.insert("value".to_string(), format_expr(value));
            node.exprs
                .insert("value".to_string(), build_block_editor_expr_node(value));
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::PromptCondition { condition, body } => {
            let mut node = block_plan_node("prompt_condition");
            node.fields
                .insert("cond".to_string(), format_condition(condition));
            node.exprs.insert(
                "cond".to_string(),
                build_block_editor_expr_node(&condition.expr),
            );
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::PromptBlock { body } => {
            let mut node = block_plan_node("prompt_block");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Contract {
            kind,
            mode,
            condition,
            then_body,
            else_body,
        } => {
            let mut node = block_plan_node("contract_guard");
            node.fields
                .insert("cond".to_string(), format_condition(condition));
            node.exprs.insert(
                "cond".to_string(),
                build_block_editor_expr_node(&condition.expr),
            );
            node.fields.insert(
                "contract_kind".to_string(),
                match kind {
                    ContractKind::Pre => "pre".to_string(),
                    ContractKind::Post => "post".to_string(),
                },
            );
            node.fields.insert(
                "mode".to_string(),
                match mode {
                    ContractMode::Abort => "abort".to_string(),
                    ContractMode::Alert => "alert".to_string(),
                },
            );
            node.inputs.insert(
                "else".to_string(),
                else_body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            if let Some(body) = then_body {
                node.inputs.insert(
                    "then".to_string(),
                    body.iter().map(build_block_editor_plan_stmt).collect(),
                );
            }
            node
        }
        Stmt::OpenBlock { body } => {
            let mut node = block_plan_node("open_block");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Hook { name, suffix, body }
            if name == "시작" && matches!(suffix, HookSuffix::Halttae) =>
        {
            let mut node = block_plan_node("hook_start");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Hook { name, suffix, body }
            if name == "매마디" && matches!(suffix, HookSuffix::Mada) =>
        {
            let mut node = block_plan_node("hook_tick");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Receive {
            kind,
            binding,
            condition,
            body,
        } => {
            let mut node = block_plan_node("receive_block");
            node.fields
                .insert("kind".to_string(), kind.clone().unwrap_or_default());
            node.fields
                .insert("binding".to_string(), binding.clone().unwrap_or_default());
            if let Some(condition) = condition {
                node.fields
                    .insert("condition".to_string(), format_expr(condition));
                node.exprs.insert(
                    "condition".to_string(),
                    build_block_editor_expr_node(condition),
                );
            } else {
                node.fields.insert("condition".to_string(), String::new());
            }
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::EventReact { kind, body } => {
            let mut node = block_plan_node("event_react");
            node.fields.insert("kind".to_string(), kind.clone());
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
        } => {
            let mut node = block_plan_node("send_signal");
            node.fields.insert(
                "sender".to_string(),
                sender.as_ref().map(format_expr).unwrap_or_default(),
            );
            node.fields
                .insert("payload".to_string(), format_expr(payload));
            node.fields
                .insert("receiver".to_string(), format_expr(receiver));
            if let Some(sender) = sender {
                node.exprs
                    .insert("sender".to_string(), build_block_editor_expr_node(sender));
            }
            node.exprs
                .insert("payload".to_string(), build_block_editor_expr_node(payload));
            node.exprs.insert(
                "receiver".to_string(),
                build_block_editor_expr_node(receiver),
            );
            node
        }
        Stmt::Return { value } => {
            let mut node = block_plan_node("return_value");
            node.fields.insert("value".to_string(), format_expr(value));
            node.exprs
                .insert("value".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::BogaeDraw => block_plan_node("bogae_draw"),
        Stmt::Break => block_plan_node("break_loop"),
        _ => raw_block_plan_node(format_program(&[stmt.clone()])),
    }
}

pub(super) fn build_block_editor_plan_decl_item(item: &DeclItem) -> BlockEditorPlanNode {
    if let Some(maegim) = &item.maegim {
        let mut range = None;
        let mut step = None;
        let mut split_count = None;
        let mut unsupported = false;
        for field in &maegim.fields {
            let leaf = maegim_field_leaf_name(&field.name);
            match leaf {
                "범위" => {
                    range = Some(format_maegim_field_value(field));
                }
                "간격" => {
                    step = Some(format_expr(&field.value));
                }
                "분할수" => {
                    split_count = Some(format_expr(&field.value));
                }
                _ => {
                    unsupported = true;
                }
            }
        }
        if !unsupported && range.is_some() && (step.is_some() ^ split_count.is_some()) {
            let kind = if step.is_some() {
                "charim_item_const_step"
            } else {
                "charim_item_const_split"
            };
            let mut node = block_plan_node(kind);
            node.fields.insert("name".to_string(), item.name.clone());
            node.fields
                .insert("type_name".to_string(), item.type_name.clone());
            node.fields.insert(
                "value".to_string(),
                item.value
                    .as_ref()
                    .map(format_expr)
                    .unwrap_or_else(String::new),
            );
            if let Some(value) = item.value.as_ref() {
                node.exprs
                    .insert("value".to_string(), build_block_editor_expr_node(value));
            }
            node.fields
                .insert("range".to_string(), range.unwrap_or_default());
            if let Some(step) = step {
                node.fields.insert("step".to_string(), step);
            }
            if let Some(split_count) = split_count {
                node.fields.insert("split_count".to_string(), split_count);
            }
            return node;
        }
        return raw_block_plan_node(format_decl_item(item, 0));
    }

    if item.kind == DeclKind::Gureut {
        let mut node = block_plan_node("charim_item_var");
        node.fields.insert("name".to_string(), item.name.clone());
        node.fields
            .insert("type_name".to_string(), item.type_name.clone());
        node.fields.insert(
            "value".to_string(),
            item.value
                .as_ref()
                .map(format_expr)
                .unwrap_or_else(String::new),
        );
        if let Some(value) = item.value.as_ref() {
            node.exprs
                .insert("value".to_string(), build_block_editor_expr_node(value));
        }
        return node;
    }

    let mut node = block_plan_node("charim_item_plain");
    node.fields.insert("name".to_string(), item.name.clone());
    node.fields
        .insert("type_name".to_string(), item.type_name.clone());
    node.fields.insert(
        "value".to_string(),
        item.value
            .as_ref()
            .map(format_expr)
            .unwrap_or_else(String::new),
    );
    if let Some(value) = item.value.as_ref() {
        node.exprs
            .insert("value".to_string(), build_block_editor_expr_node(value));
    }
    node
}

pub(super) fn build_block_editor_plan_choose_branch(branch: &ChooseBranch) -> BlockEditorPlanNode {
    let mut node = block_plan_node("choose_branch");
    node.fields
        .insert("cond".to_string(), format_condition(&branch.condition));
    node.exprs.insert(
        "cond".to_string(),
        build_block_editor_expr_node(&branch.condition.expr),
    );
    node.inputs.insert(
        "body".to_string(),
        branch
            .body
            .iter()
            .map(build_block_editor_plan_stmt)
            .collect(),
    );
    node
}

pub(super) fn build_block_editor_binding_expr_node(binding: &Binding) -> BlockEditorExprNode {
    let mut node = block_editor_expr_node("binding", format_expr(&binding.value));
    node.fields.insert("name".to_string(), binding.name.clone());
    node.inputs.insert(
        "value".to_string(),
        vec![build_block_editor_expr_node(&binding.value)],
    );
    node
}

pub(super) fn build_block_editor_expr_node(expr: &Expr) -> BlockEditorExprNode {
    match expr {
        Expr::Literal(value) => {
            let mut node = block_editor_expr_node("literal", format_expr(expr));
            node.fields.insert(
                "literal_kind".to_string(),
                match value {
                    Literal::Str(_) => "str".to_string(),
                    Literal::Num(_) => "num".to_string(),
                    Literal::Bool(_) => "bool".to_string(),
                    Literal::None => "none".to_string(),
                    Literal::Atom(_) => "atom".to_string(),
                },
            );
            node.fields
                .insert("value".to_string(), format_literal(value));
            node
        }
        Expr::Resource(name) => {
            let mut node = block_editor_expr_node("resource", format_expr(expr));
            node.fields.insert("name".to_string(), name.clone());
            node
        }
        Expr::Template { body } => {
            let mut node = block_editor_expr_node("template", format_expr(expr));
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Expr::TemplateApply { bindings, body } => {
            let mut node = block_editor_expr_node("template_apply", format_expr(expr));
            node.fields.insert("body".to_string(), body.clone());
            node.inputs.insert(
                "bindings".to_string(),
                bindings
                    .iter()
                    .map(build_block_editor_binding_expr_node)
                    .collect(),
            );
            node
        }
        Expr::Formula { tag, body } => {
            let mut node = block_editor_expr_node("formula", format_expr(expr));
            node.fields
                .insert("tag".to_string(), tag.clone().unwrap_or_default());
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Expr::PromptExpr { expr: inner } => {
            let mut node = block_editor_expr_node("prompt_expr", format_expr(expr));
            node.inputs.insert(
                "expr".to_string(),
                vec![build_block_editor_expr_node(inner)],
            );
            node
        }
        Expr::PromptBlock { body } => {
            let mut node = block_editor_expr_node("prompt_block_expr", format_expr(expr));
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Expr::Path(path) => {
            let mut node = block_editor_expr_node("path", format_expr(expr));
            node.fields.insert("path".to_string(), format_path(path));
            node.fields
                .insert("segments".to_string(), path.segments.join("/"));
            node
        }
        Expr::FieldAccess { target, field } => {
            let mut node = block_editor_expr_node("field_access", format_expr(expr));
            node.fields.insert("field".to_string(), field.clone());
            node.inputs.insert(
                "target".to_string(),
                vec![build_block_editor_expr_node(target)],
            );
            node
        }
        Expr::Call { name, args } => {
            let mut node = block_editor_expr_node("call", format_expr(expr));
            node.fields.insert("name".to_string(), name.clone());
            node.inputs.insert(
                "args".to_string(),
                args.iter().map(build_block_editor_expr_node).collect(),
            );
            node
        }
        Expr::CallIn { name, bindings } => {
            let mut node = block_editor_expr_node("call_in", format_expr(expr));
            node.fields.insert("name".to_string(), name.clone());
            node.inputs.insert(
                "bindings".to_string(),
                bindings
                    .iter()
                    .map(build_block_editor_binding_expr_node)
                    .collect(),
            );
            node
        }
        Expr::Pack { bindings } => {
            let mut node = block_editor_expr_node("pack", format_expr(expr));
            node.inputs.insert(
                "bindings".to_string(),
                bindings
                    .iter()
                    .map(build_block_editor_binding_expr_node)
                    .collect(),
            );
            node
        }
        Expr::Unit { value, unit } => {
            let mut node = block_editor_expr_node("unit", format_expr(expr));
            node.fields.insert("unit".to_string(), unit.clone());
            node.inputs.insert(
                "value".to_string(),
                vec![build_block_editor_expr_node(value)],
            );
            node
        }
        Expr::Unary { op, expr: inner } => {
            let mut node = block_editor_expr_node("unary", format_expr(expr));
            node.fields.insert(
                "op".to_string(),
                match op {
                    UnaryOp::Neg => "neg".to_string(),
                },
            );
            node.inputs.insert(
                "expr".to_string(),
                vec![build_block_editor_expr_node(inner)],
            );
            node
        }
        Expr::Binary { left, op, right } => {
            let mut node = block_editor_expr_node("binary", format_expr(expr));
            node.fields.insert(
                "op".to_string(),
                match op {
                    BinaryOp::Add => "add".to_string(),
                    BinaryOp::Sub => "sub".to_string(),
                    BinaryOp::Mul => "mul".to_string(),
                    BinaryOp::Div => "div".to_string(),
                    BinaryOp::Mod => "mod".to_string(),
                    BinaryOp::And => "and".to_string(),
                    BinaryOp::Or => "or".to_string(),
                    BinaryOp::RelationEq => "relation_eq".to_string(),
                    BinaryOp::Eq => "eq".to_string(),
                    BinaryOp::NotEq => "neq".to_string(),
                    BinaryOp::Lt => "lt".to_string(),
                    BinaryOp::Lte => "lte".to_string(),
                    BinaryOp::Gt => "gt".to_string(),
                    BinaryOp::Gte => "gte".to_string(),
                },
            );
            node.inputs
                .insert("left".to_string(), vec![build_block_editor_expr_node(left)]);
            node.inputs.insert(
                "right".to_string(),
                vec![build_block_editor_expr_node(right)],
            );
            node
        }
        Expr::Pipe { left, kind, right } => {
            let mut node = block_editor_expr_node("pipe", format_expr(expr));
            node.fields.insert(
                "pipe_kind".to_string(),
                match kind {
                    PipeKind::Haseo => "haseo".to_string(),
                    PipeKind::Hago => "hago".to_string(),
                },
            );
            node.inputs
                .insert("left".to_string(), vec![build_block_editor_expr_node(left)]);
            node.inputs.insert(
                "right".to_string(),
                vec![build_block_editor_expr_node(right)],
            );
            node
        }
        Expr::SeedLiteral { param, body } => {
            let mut node = block_editor_expr_node("seed_literal", format_expr(expr));
            node.fields.insert("param".to_string(), param.clone());
            node.inputs
                .insert("body".to_string(), vec![build_block_editor_expr_node(body)]);
            node
        }
        Expr::Grouped { expr: inner } => {
            let mut node = block_editor_expr_node("grouped", format_expr(expr));
            node.inputs.insert(
                "expr".to_string(),
                vec![build_block_editor_expr_node(inner)],
            );
            node
        }
    }
}

pub(super) fn block_editor_expr_node(kind: &str, text: String) -> BlockEditorExprNode {
    BlockEditorExprNode {
        kind: kind.to_string(),
        text,
        fields: BTreeMap::new(),
        inputs: BTreeMap::new(),
    }
}

pub(super) fn block_plan_node(kind: &str) -> BlockEditorPlanNode {
    BlockEditorPlanNode {
        kind: kind.to_string(),
        fields: BTreeMap::new(),
        exprs: BTreeMap::new(),
        inputs: BTreeMap::new(),
        raw_text: None,
    }
}

pub(super) fn raw_block_plan_node(text: String) -> BlockEditorPlanNode {
    let mut node = block_plan_node("raw");
    node.raw_text = Some(text.trim().to_string());
    node
}

