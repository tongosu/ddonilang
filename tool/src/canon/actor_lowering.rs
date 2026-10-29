//! Existing single-actor event lowering; private carriers and emitted source bodies unchanged.
use super::*;

#[derive(Debug, Clone)]
struct SimpleActorField {
    name: String,
    init: Expr,
}

#[derive(Debug, Clone)]
struct SimpleReceiveHandler {
    kind: Option<String>,
    binding: Option<String>,
    condition: Option<Expr>,
    body: Vec<Stmt>,
    source_order: usize,
}

#[derive(Debug, Clone)]
struct SimpleActorModel {
    name: String,
    fields: Vec<SimpleActorField>,
    handlers: Vec<SimpleReceiveHandler>,
}

#[derive(Debug, Clone)]
struct SimpleRootField {
    name: String,
    init: Expr,
}

#[derive(Debug, Clone)]
struct EventLowerContext<'a> {
    actor_name: &'a str,
    actor_names: &'a HashSet<String>,
    namespace_actor_fields: bool,
    payload_kind: &'a str,
    payload_fields: &'a HashMap<String, Expr>,
    payload_binding: Option<&'a str>,
    alert_binding: Option<&'a str>,
}

#[derive(Debug, Clone)]
struct PendingDispatch {
    receiver_name: String,
    payload_kind: String,
    payload_fields: HashMap<String, Expr>,
    guard: Option<Expr>,
    depth: usize,
}

const MAX_GENERIC_EVENT_CHAIN_DEPTH: usize = 16;

pub(super) fn lower_single_imja_event_subset_program(program: &[Stmt]) -> Option<String> {
    let mut actor_defs = Vec::new();
    let mut start_hooks = Vec::new();
    let mut every_hooks = Vec::new();
    let mut top_level_draw = Vec::new();
    for stmt in program {
        match stmt {
            Stmt::SeedDef {
                name, kind, body, ..
            } if kind == "임자" => actor_defs.push(build_simple_actor_model(name, body)?),
            Stmt::SeedDef { kind, .. } if kind == "알림씨" => {}
            Stmt::Hook { name, suffix, body }
                if name == "시작" && matches!(suffix, HookSuffix::Halttae) =>
            {
                start_hooks.push(body.clone());
            }
            Stmt::Hook { suffix, body, .. } if matches!(suffix, HookSuffix::Mada) => {
                every_hooks.push(body.clone());
            }
            Stmt::BogaeDraw => top_level_draw.push(stmt.clone()),
            _ => return None,
        }
    }

    if actor_defs.is_empty() || (start_hooks.is_empty() && every_hooks.is_empty()) {
        return None;
    }
    let actor_names: HashSet<String> = actor_defs.iter().map(|actor| actor.name.clone()).collect();
    let namespace_actor_fields = actor_defs.len() > 1;
    let root_extras = collect_simple_root_fields_from_hooks(
        &start_hooks,
        &every_hooks,
        &actor_defs,
        namespace_actor_fields,
    )?;
    let mut out = Vec::new();
    out.push(Stmt::SeedDef {
        name: "매틱".to_string(),
        kind: "움직씨".to_string(),
        params: Vec::new(),
        body: vec![Stmt::Expr {
            value: Expr::Literal(Literal::None),
        }],
    });

    let empty_fields = HashMap::new();
    let top_level_ctx = EventLowerContext {
        actor_name: "",
        actor_names: &actor_names,
        namespace_actor_fields,
        payload_kind: "",
        payload_fields: &empty_fields,
        payload_binding: None,
        alert_binding: None,
    };
    if !actor_defs.is_empty() || !root_extras.is_empty() || !start_hooks.is_empty() {
        let mut lowered_start = Vec::new();
        for actor in &actor_defs {
            for field in &actor.fields {
                lowered_start.push(Stmt::Assign {
                    target: Path {
                        segments: vec![lowered_actor_field_name(
                            &actor.name,
                            &field.name,
                            namespace_actor_fields,
                        )],
                    },
                    value: field.init.clone(),
                });
            }
        }
        for field in &root_extras {
            lowered_start.push(Stmt::Assign {
                target: Path {
                    segments: vec![field.name.clone()],
                },
                value: field.init.clone(),
            });
        }
        for body in &start_hooks {
            for stmt in body {
                lower_top_level_stmt(stmt, &actor_defs, &top_level_ctx, &mut lowered_start, 0)?;
            }
        }
        out.push(Stmt::Hook {
            name: "시작".to_string(),
            suffix: HookSuffix::Halttae,
            body: lowered_start,
        });
    }
    for body in &every_hooks {
        let mut lowered_every = Vec::new();
        for stmt in body {
            lower_top_level_stmt(stmt, &actor_defs, &top_level_ctx, &mut lowered_every, 0)?;
        }
        out.push(Stmt::Hook {
            name: "매마디".to_string(),
            suffix: HookSuffix::Mada,
            body: lowered_every,
        });
    }
    out.extend(top_level_draw);
    Some(format_program(&out))
}

fn build_simple_actor_model(name: &str, body: &[Stmt]) -> Option<SimpleActorModel> {
    let mut fields = Vec::new();
    let mut handlers = Vec::new();
    let mut seen_handler = false;
    for stmt in body {
        match stmt {
            Stmt::Assign { target, value } if !seen_handler => {
                let (field_name, init_value) = extract_actor_field_assign(target, value, name)?;
                fields.push(SimpleActorField {
                    name: field_name,
                    init: init_value,
                });
            }
            Stmt::Receive {
                kind,
                binding,
                condition,
                body,
            } => {
                seen_handler = true;
                handlers.push(SimpleReceiveHandler {
                    kind: kind.clone(),
                    binding: binding.clone(),
                    condition: condition.clone(),
                    body: body.clone(),
                    source_order: handlers.len(),
                });
            }
            _ => return None,
        }
    }
    if handlers.is_empty() {
        return None;
    }
    Some(SimpleActorModel {
        name: name.to_string(),
        fields,
        handlers,
    })
}

fn actor_field_name_from_path(path: &Path, actor_name: &str) -> Option<String> {
    // F5-GA[ACTOR_FIELD]: segments 길이2를 확인한 뒤에만 0/1번을 읽으며 공유 Path는 이 함수에서 바뀌지 않는다.
    if path.segments.len() != 2 {
        return None;
    }
    match path.segments[0].as_str() {
        "제" => Some(path.segments[1].clone()),
        prefix if prefix == actor_name => Some(path.segments[1].clone()),
        _ => None,
    }
}

fn lowered_actor_field_name(
    actor_name: &str,
    field_name: &str,
    namespace_actor_fields: bool,
) -> String {
    if namespace_actor_fields {
        format!("{actor_name}__{field_name}")
    } else {
        field_name.to_string()
    }
}

fn root_field_name_from_path(path: &Path) -> Option<String> {
    // F5-GA[ACTOR_ROOT]: 길이2가 아니면 반환하므로 이어지는 0/1번 접근이 범위 안이다.
    if path.segments.len() != 2 {
        return None;
    }
    match path.segments[0].as_str() {
        "바탕" | "살림" | "샘" => Some(path.segments[1].clone()),
        _ => None,
    }
}

fn extract_actor_field_assign(
    target: &Path,
    value: &Expr,
    actor_name: &str,
) -> Option<(String, Expr)> {
    // F5-GA[ACTOR_ASSIGN]: target 길이1 검사 뒤에만 0번, args 길이3 검사 뒤에만 0/1/2번, base 길이1 검사 뒤에만 0번을 읽는다. 조건의 단락 평가도 이 순서를 유지한다.
    if let Some(field_name) = actor_field_name_from_path(target, actor_name) {
        return Some((field_name, value.clone()));
    }
    if target.segments.len() != 1
        || !(target.segments[0] == "제" || target.segments[0] == actor_name)
    {
        return None;
    }
    let Expr::Call { name, args } = value else {
        return None;
    };
    if name != "짝맞춤.바꾼값" || args.len() != 3 {
        return None;
    }
    let Expr::Path(base) = &args[0] else {
        return None;
    };
    if base.segments.len() != 1 || base.segments[0] != target.segments[0] {
        return None;
    }
    let Expr::Literal(Literal::Str(field_name)) = &args[1] else {
        return None;
    };
    Some((field_name.clone(), args[2].clone()))
}

fn collect_simple_root_fields_from_hooks(
    start_hooks: &[Vec<Stmt>],
    every_hooks: &[Vec<Stmt>],
    actors: &[SimpleActorModel],
    namespace_actor_fields: bool,
) -> Option<Vec<SimpleRootField>> {
    let actor_field_names: HashSet<String> = if namespace_actor_fields {
        HashSet::new()
    } else {
        actors
            .iter()
            .flat_map(|actor| actor.fields.iter().map(|field| field.name.clone()))
            .collect()
    };
    let mut extras = Vec::new();
    let mut seen = HashSet::new();
    for body in start_hooks {
        collect_simple_root_fields_from_stmts(body, &actor_field_names, &mut seen, &mut extras)?;
    }
    for body in every_hooks {
        collect_simple_root_fields_from_stmts(body, &actor_field_names, &mut seen, &mut extras)?;
    }
    Some(extras)
}

fn collect_simple_root_fields_from_stmts(
    stmts: &[Stmt],
    actor_field_names: &HashSet<String>,
    seen: &mut HashSet<String>,
    extras: &mut Vec<SimpleRootField>,
) -> Option<()> {
    for stmt in stmts {
        match stmt {
            Stmt::Assign { target, value } => {
                let name = if target.segments.len() == 1 {
                    // F5-GA[ACTOR_COLLECT]: 이 분기는 바로 앞 길이1 조건이 참일 때만 실행된다.
                    Some(target.segments[0].clone())
                } else {
                    root_field_name_from_path(target)
                };
                let Some(name) = name else {
                    continue;
                };
                if actor_field_names.contains(&name) || !seen.insert(name.clone()) {
                    continue;
                }
                extras.push(SimpleRootField {
                    name,
                    init: value.clone(),
                });
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_simple_root_fields_from_stmts(then_body, actor_field_names, seen, extras)?;
                if let Some(body) = else_body {
                    collect_simple_root_fields_from_stmts(body, actor_field_names, seen, extras)?;
                }
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_simple_root_fields_from_stmts(
                        &branch.body,
                        actor_field_names,
                        seen,
                        extras,
                    )?;
                }
                if let Some(body) = else_body {
                    collect_simple_root_fields_from_stmts(body, actor_field_names, seen, extras)?;
                }
            }
            _ => {}
        }
    }
    Some(())
}

fn lower_top_level_stmt(
    stmt: &Stmt,
    actors: &[SimpleActorModel],
    ctx: &EventLowerContext<'_>,
    out: &mut Vec<Stmt>,
    depth: usize,
) -> Option<()> {
    match stmt {
        Stmt::Send {
            sender: _,
            payload,
            receiver,
        } => lower_send_dispatch(payload, receiver, actors, ctx, out, depth),
        Stmt::Assign { target, value } => {
            let lowered_target = lower_path(target, ctx)?;
            if lowered_target.segments.len() != 1 {
                return None;
            }
            out.push(Stmt::Assign {
                target: lowered_target,
                value: lower_expr(value, ctx)?,
            });
            Some(())
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let mut lowered_then = Vec::new();
            for child in then_body {
                lower_top_level_stmt(child, actors, ctx, &mut lowered_then, depth)?;
            }
            let lowered_else = match else_body {
                Some(body) => {
                    let mut lowered = Vec::new();
                    for child in body {
                        lower_top_level_stmt(child, actors, ctx, &mut lowered, depth)?;
                    }
                    Some(lowered)
                }
                None => None,
            };
            out.push(Stmt::If {
                condition: Condition {
                    expr: lower_expr(&condition.expr, ctx)?,
                    style: condition.style,
                    negated: condition.negated,
                },
                then_body: lowered_then,
                else_body: lowered_else,
            });
            Some(())
        }
        Stmt::Choose {
            branches,
            else_body,
            exhaustive,
        } => {
            let mut lowered_branches = Vec::new();
            for branch in branches {
                let mut lowered_body = Vec::new();
                for child in &branch.body {
                    lower_top_level_stmt(child, actors, ctx, &mut lowered_body, depth)?;
                }
                lowered_branches.push(ChooseBranch {
                    condition: Condition {
                        expr: lower_expr(&branch.condition.expr, ctx)?,
                        style: branch.condition.style,
                        negated: branch.condition.negated,
                    },
                    body: lowered_body,
                });
            }
            let lowered_else = match else_body {
                Some(body) => {
                    let mut lowered = Vec::new();
                    for child in body {
                        lower_top_level_stmt(child, actors, ctx, &mut lowered, depth)?;
                    }
                    Some(lowered)
                }
                None => None,
            };
            out.extend(lower_choose_to_if_stmts(
                lowered_branches,
                lowered_else,
                *exhaustive,
            ));
            Some(())
        }
        Stmt::Show { value } => {
            out.push(Stmt::Show {
                value: lower_expr(value, ctx)?,
            });
            Some(())
        }
        Stmt::Inspect { value } => {
            out.push(Stmt::Inspect {
                value: lower_expr(value, ctx)?,
            });
            Some(())
        }
        _ => None,
    }
}

fn lower_stmt_list(
    stmts: &[Stmt],
    actors: &[SimpleActorModel],
    ctx: &EventLowerContext<'_>,
    out: &mut Vec<Stmt>,
    pending: &mut VecDeque<PendingDispatch>,
    active_send_guard: Option<Expr>,
    depth: usize,
) -> Option<()> {
    for stmt in stmts {
        match stmt {
            Stmt::Assign { target, value } => {
                let (field_name, lowered_value) = if let Some((field_name, lowered_source)) =
                    extract_actor_field_assign(target, value, ctx.actor_name)
                {
                    (field_name, lower_expr(&lowered_source, ctx)?)
                } else {
                    let lowered_target = lower_path(target, ctx)?;
                    (
                        lowered_target.segments.last()?.clone(),
                        lower_expr(value, ctx)?,
                    )
                };
                out.push(Stmt::Assign {
                    target: Path {
                        segments: vec![lowered_actor_field_name(
                            ctx.actor_name,
                            &field_name,
                            ctx.namespace_actor_fields,
                        )],
                    },
                    value: lowered_value,
                });
            }
            Stmt::Send {
                sender: _,
                payload,
                receiver,
            } => {
                pending.push_back(build_pending_dispatch(
                    payload,
                    receiver,
                    ctx,
                    active_send_guard.clone(),
                    depth + 1,
                )?);
            }
            Stmt::Show { value } => {
                out.push(Stmt::Show {
                    value: lower_expr(value, ctx)?,
                });
            }
            Stmt::Inspect { value } => {
                out.push(Stmt::Inspect {
                    value: lower_expr(value, ctx)?,
                });
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let lowered_condition = Condition {
                    expr: lower_expr(&condition.expr, ctx)?,
                    style: condition.style,
                    negated: condition.negated,
                };
                let mut lowered_then = Vec::new();
                lower_stmt_list(
                    then_body,
                    actors,
                    ctx,
                    &mut lowered_then,
                    pending,
                    combine_guard_expr(
                        active_send_guard.clone(),
                        Some(condition_to_guard_expr(&lowered_condition, true)),
                    ),
                    depth,
                )?;
                let lowered_else = match else_body {
                    Some(body) => {
                        let mut lowered = Vec::new();
                        lower_stmt_list(
                            body,
                            actors,
                            ctx,
                            &mut lowered,
                            pending,
                            combine_guard_expr(
                                active_send_guard.clone(),
                                Some(condition_to_guard_expr(&lowered_condition, false)),
                            ),
                            depth,
                        )?;
                        Some(lowered)
                    }
                    None => None,
                };
                out.push(Stmt::If {
                    condition: lowered_condition,
                    then_body: lowered_then,
                    else_body: lowered_else,
                });
            }
            Stmt::Choose {
                branches,
                else_body,
                exhaustive,
            } => {
                let mut lowered_branches = Vec::new();
                let mut previous_branch_miss_guard: Option<Expr> = None;
                for branch in branches {
                    let lowered_condition = Condition {
                        expr: lower_expr(&branch.condition.expr, ctx)?,
                        style: branch.condition.style,
                        negated: branch.condition.negated,
                    };
                    let selected_guard = combine_guard_expr(
                        combine_guard_expr(
                            active_send_guard.clone(),
                            previous_branch_miss_guard.clone(),
                        ),
                        Some(condition_to_guard_expr(&lowered_condition, true)),
                    );
                    let mut lowered_body = Vec::new();
                    lower_stmt_list(
                        &branch.body,
                        actors,
                        ctx,
                        &mut lowered_body,
                        pending,
                        selected_guard,
                        depth,
                    )?;
                    previous_branch_miss_guard = combine_guard_expr(
                        previous_branch_miss_guard,
                        Some(condition_to_guard_expr(&lowered_condition, false)),
                    );
                    lowered_branches.push(ChooseBranch {
                        condition: lowered_condition,
                        body: lowered_body,
                    });
                }
                let lowered_else = match else_body {
                    Some(body) => {
                        let mut lowered = Vec::new();
                        lower_stmt_list(
                            body,
                            actors,
                            ctx,
                            &mut lowered,
                            pending,
                            combine_guard_expr(
                                active_send_guard.clone(),
                                previous_branch_miss_guard,
                            ),
                            depth,
                        )?;
                        Some(lowered)
                    }
                    None => None,
                };
                out.extend(lower_choose_to_if_stmts(
                    lowered_branches,
                    lowered_else,
                    *exhaustive,
                ));
            }
            _ => return None,
        }
    }
    Some(())
}

fn lower_choose_to_if_stmts(
    branches: Vec<ChooseBranch>,
    else_body: Option<Vec<Stmt>>,
    exhaustive: bool,
) -> Vec<Stmt> {
    let mut current = else_body.unwrap_or_default();
    for branch in branches.into_iter().rev() {
        current = vec![Stmt::If {
            condition: branch.condition,
            then_body: branch.body,
            else_body: if current.is_empty() && exhaustive {
                None
            } else {
                Some(current)
            },
        }];
    }
    current
}

fn lower_receiver_name(receiver: &Expr, current_actor_name: &str) -> Option<String> {
    // F5-GA[ACTOR_RECEIVER]: match guard가 길이1을 보장한 가지에서만 segments[0]을 읽는다.
    match receiver {
        Expr::Path(path) if path.segments.len() == 1 => match path.segments[0].as_str() {
            "제" if !current_actor_name.is_empty() => Some(current_actor_name.to_string()),
            _ => Some(path.segments[0].clone()),
        },
        _ => None,
    }
}

fn lower_send_dispatch(
    payload: &Expr,
    receiver: &Expr,
    actors: &[SimpleActorModel],
    ctx: &EventLowerContext<'_>,
    out: &mut Vec<Stmt>,
    depth: usize,
) -> Option<()> {
    let initial = build_pending_dispatch(payload, receiver, ctx, None, depth)?;
    lower_pending_dispatch_queue(initial, actors, ctx, out)
}

fn build_pending_dispatch(
    payload: &Expr,
    receiver: &Expr,
    ctx: &EventLowerContext<'_>,
    guard: Option<Expr>,
    depth: usize,
) -> Option<PendingDispatch> {
    if depth > MAX_GENERIC_EVENT_CHAIN_DEPTH {
        return None;
    }
    let receiver_name = lower_receiver_name(receiver, ctx.actor_name)?;
    let (payload_kind, payload_fields) =
        if let Some(forwarded) = extract_forwarded_current_payload(payload, ctx) {
            forwarded
        } else {
            let lowered_payload = lower_expr(payload, ctx)?;
            extract_event_payload(&lowered_payload)?
        };
    Some(PendingDispatch {
        receiver_name,
        payload_kind,
        payload_fields,
        guard,
        depth,
    })
}

fn lower_pending_dispatch_queue(
    initial: PendingDispatch,
    actors: &[SimpleActorModel],
    ctx: &EventLowerContext<'_>,
    out: &mut Vec<Stmt>,
) -> Option<()> {
    let mut pending = VecDeque::new();
    pending.push_back(initial);
    while let Some(dispatch) = pending.pop_front() {
        let actor = actors
            .iter()
            .find(|actor| actor.name == dispatch.receiver_name)?;
        let dispatch_ctx = EventLowerContext {
            actor_name: &actor.name,
            actor_names: ctx.actor_names,
            namespace_actor_fields: ctx.namespace_actor_fields,
            payload_kind: &dispatch.payload_kind,
            payload_fields: &dispatch.payload_fields,
            payload_binding: None,
            alert_binding: None,
        };
        for handler in ordered_handlers_for_dispatch(&actor.handlers, dispatch_ctx.payload_kind) {
            let binding_ctx = EventLowerContext {
                actor_name: dispatch_ctx.actor_name,
                actor_names: dispatch_ctx.actor_names,
                namespace_actor_fields: dispatch_ctx.namespace_actor_fields,
                payload_kind: dispatch_ctx.payload_kind,
                payload_fields: dispatch_ctx.payload_fields,
                payload_binding: handler.kind.as_ref().and(handler.binding.as_deref()),
                alert_binding: handler
                    .kind
                    .is_none()
                    .then_some(handler.binding.as_deref())
                    .flatten(),
            };
            let lowered_condition = match handler.condition.as_ref() {
                Some(condition) => Some(lower_expr(condition, &binding_ctx)?),
                None => None,
            };
            if matches!(
                lowered_condition.as_ref().and_then(eval_static_bool_expr),
                Some(false)
            ) {
                continue;
            }
            let mut lowered_body = Vec::new();
            let mut handler_pending = VecDeque::new();
            lower_stmt_list(
                &handler.body,
                actors,
                &binding_ctx,
                &mut lowered_body,
                &mut handler_pending,
                None,
                dispatch.depth,
            )?;
            let effective_guard = combine_guard_expr(dispatch.guard.clone(), lowered_condition);
            apply_guard_to_pending_queue(&mut handler_pending, effective_guard.clone());
            if let Some(guard) = effective_guard {
                if matches!(eval_static_bool_expr(&guard), Some(true)) {
                    out.extend(lowered_body);
                    pending.extend(handler_pending);
                    continue;
                }
                out.push(Stmt::If {
                    condition: Condition {
                        expr: guard,
                        style: ConditionStyle::Plain,
                        negated: false,
                    },
                    then_body: lowered_body,
                    else_body: None,
                });
            } else {
                out.extend(lowered_body);
                pending.extend(handler_pending);
            }
        }
    }
    Some(())
}

fn ordered_handlers_for_dispatch<'a>(
    handlers: &'a [SimpleReceiveHandler],
    payload_kind: &str,
) -> Vec<&'a SimpleReceiveHandler> {
    let mut ordered = Vec::new();
    for rank in 0..4u8 {
        for handler in handlers {
            if !handler
                .kind
                .as_ref()
                .is_none_or(|kind| kind == payload_kind)
            {
                continue;
            }
            if receive_handler_rank(handler) != rank {
                continue;
            }
            ordered.push(handler);
        }
    }
    ordered.sort_by_key(|handler| (receive_handler_rank(handler), handler.source_order));
    ordered
}

fn receive_handler_rank(handler: &SimpleReceiveHandler) -> u8 {
    match (handler.kind.is_some(), handler.condition.is_some()) {
        (true, true) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (false, false) => 3,
    }
}

fn combine_guard_expr(base: Option<Expr>, extra: Option<Expr>) -> Option<Expr> {
    match (base, extra) {
        (None, None) => None,
        (Some(expr), None) | (None, Some(expr)) => Some(expr),
        (Some(left), Some(right)) => Some(Expr::Binary {
            left: Box::new(left),
            op: BinaryOp::And,
            right: Box::new(right),
        }),
    }
}

fn condition_to_guard_expr(condition: &Condition, truth: bool) -> Expr {
    let expects_expr_true = if truth {
        !condition.negated
    } else {
        condition.negated
    };
    if expects_expr_true {
        return condition.expr.clone();
    }
    Expr::Binary {
        left: Box::new(condition.expr.clone()),
        op: BinaryOp::Eq,
        right: Box::new(Expr::Literal(Literal::Bool(false))),
    }
}

fn apply_guard_to_pending_queue(pending: &mut VecDeque<PendingDispatch>, guard: Option<Expr>) {
    if guard.is_none() {
        return;
    }
    for dispatch in pending.iter_mut() {
        dispatch.guard = combine_guard_expr(dispatch.guard.clone(), guard.clone());
    }
}

fn extract_forwarded_current_payload(
    payload: &Expr,
    ctx: &EventLowerContext<'_>,
) -> Option<(String, HashMap<String, Expr>)> {
    // F5-GA[ACTOR_FORWARD]: 아래 길이1 조기 거부를 통과한 공유 Path만 head로 읽는다.
    let Expr::Path(path) = payload else {
        return None;
    };
    if path.segments.len() != 1 || ctx.payload_kind.is_empty() {
        return None;
    }
    let head = path.segments[0].as_str();
    let matches_current = head == ctx.payload_kind
        || ctx.payload_binding.is_some_and(|binding| head == binding)
        || ctx.alert_binding.is_some_and(|binding| head == binding)
        || (ctx.alert_binding.is_none() && head == "알림");
    if !matches_current {
        return None;
    }
    Some((ctx.payload_kind.to_string(), ctx.payload_fields.clone()))
}

fn extract_event_payload(payload: &Expr) -> Option<(String, HashMap<String, Expr>)> {
    // F5-GA[ACTOR_PAYLOAD]: Path는 길이1 match guard 안에서, Call 인자는 비어 있음/길이1 검사 뒤에만 첫 원소를 읽는다.
    match payload {
        Expr::Path(path) if path.segments.len() == 1 => {
            Some((path.segments[0].clone(), HashMap::new()))
        }
        Expr::CallIn { name, bindings } => {
            let mut fields = HashMap::new();
            for binding in bindings {
                fields.insert(binding.name.clone(), binding.value.clone());
            }
            Some((name.clone(), fields))
        }
        Expr::Call { name, args } => {
            if args.is_empty() {
                return Some((name.clone(), HashMap::new()));
            }
            if args.len() != 1 {
                return None;
            }
            let Expr::Pack { bindings } = &args[0] else {
                return None;
            };
            let mut fields = HashMap::new();
            for binding in bindings {
                fields.insert(binding.name.clone(), binding.value.clone());
            }
            Some((name.clone(), fields))
        }
        _ => None,
    }
}

fn lower_path(path: &Path, ctx: &EventLowerContext<'_>) -> Option<Path> {
    // F5-GA[ACTOR_PATH]: 빈 경로는 먼저 반환하고, 각 가지의 1번 원소는 별도 길이2 검사 뒤에만 읽는다.
    if path.segments.is_empty() {
        return None;
    }
    if let Some(name) = root_field_name_from_path(path) {
        return Some(Path {
            segments: vec![name],
        });
    }
    match path.segments[0].as_str() {
        "제" => {
            if path.segments.len() != 2 || ctx.actor_name.is_empty() {
                return None;
            }
            Some(Path {
                segments: vec![lowered_actor_field_name(
                    ctx.actor_name,
                    &path.segments[1],
                    ctx.namespace_actor_fields,
                )],
            })
        }
        prefix if prefix == ctx.actor_name => {
            if path.segments.len() != 2 {
                return None;
            }
            Some(Path {
                segments: vec![lowered_actor_field_name(
                    ctx.actor_name,
                    &path.segments[1],
                    ctx.namespace_actor_fields,
                )],
            })
        }
        prefix if ctx.actor_names.contains(prefix) => {
            if path.segments.len() != 2 {
                return None;
            }
            Some(Path {
                segments: vec![lowered_actor_field_name(
                    prefix,
                    &path.segments[1],
                    ctx.namespace_actor_fields,
                )],
            })
        }
        _ => Some(path.clone()),
    }
}

fn lower_expr(expr: &Expr, ctx: &EventLowerContext<'_>) -> Option<Expr> {
    match expr {
        Expr::Literal(_) | Expr::Resource(_) | Expr::Template { .. } | Expr::Formula { .. } => {
            Some(expr.clone())
        }
        Expr::PromptExpr { expr } => Some(Expr::PromptExpr {
            expr: Box::new(lower_expr(expr, ctx)?),
        }),
        Expr::PromptBlock { .. } => Some(expr.clone()),
        Expr::Path(path) => lower_expr_path(path, ctx),
        Expr::FieldAccess { target, field } => Some(Expr::FieldAccess {
            target: Box::new(lower_expr(target, ctx)?),
            field: field.clone(),
        }),
        Expr::Call { name, args } => Some(Expr::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| lower_expr(arg, ctx))
                .collect::<Option<Vec<_>>>()?,
        }),
        Expr::CallIn { name, bindings } => Some(Expr::CallIn {
            name: name.clone(),
            bindings: bindings
                .iter()
                .map(|binding| {
                    Some(Binding {
                        name: binding.name.clone(),
                        value: lower_expr(&binding.value, ctx)?,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
        }),
        Expr::Pack { bindings } => Some(Expr::Pack {
            bindings: bindings
                .iter()
                .map(|binding| {
                    Some(Binding {
                        name: binding.name.clone(),
                        value: lower_expr(&binding.value, ctx)?,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
        }),
        Expr::Unit { value, unit } => Some(Expr::Unit {
            value: Box::new(lower_expr(value, ctx)?),
            unit: unit.clone(),
        }),
        Expr::Unary { op, expr } => Some(Expr::Unary {
            op: *op,
            expr: Box::new(lower_expr(expr, ctx)?),
        }),
        Expr::Binary { left, op, right } => Some(Expr::Binary {
            left: Box::new(lower_expr(left, ctx)?),
            op: *op,
            right: Box::new(lower_expr(right, ctx)?),
        }),
        Expr::Pipe { left, kind, right } => Some(Expr::Pipe {
            left: Box::new(lower_expr(left, ctx)?),
            kind: *kind,
            right: Box::new(lower_expr(right, ctx)?),
        }),
        Expr::SeedLiteral { param, body } => Some(Expr::SeedLiteral {
            param: param.clone(),
            body: Box::new(lower_expr(body, ctx)?),
        }),
        Expr::TemplateApply { bindings, body } => Some(Expr::TemplateApply {
            bindings: bindings
                .iter()
                .map(|binding| {
                    Some(Binding {
                        name: binding.name.clone(),
                        value: lower_expr(&binding.value, ctx)?,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
            body: body.clone(),
        }),
        Expr::Grouped { expr } => Some(Expr::Grouped {
            expr: Box::new(lower_expr(expr, ctx)?),
        }),
    }
}

fn group_payload_field_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Binary { .. } | Expr::Pipe { .. } => Expr::Grouped {
            expr: Box::new(expr),
        },
        other => other,
    }
}

fn lower_expr_path(path: &Path, ctx: &EventLowerContext<'_>) -> Option<Expr> {
    // F5-GA[ACTOR_EXPR_PATH]: 공유 segments는 빈 경로 조기 거부 후 0번을 읽는다. 1번은 길이2, 2번은 길이3 가지 안에서만 읽으며 경로/문맥을 변형하지 않는다.
    if path.segments.is_empty() {
        return None;
    }
    if root_field_name_from_path(path).is_some() {
        return Some(Expr::Path(lower_path(path, ctx)?));
    }
    let segs = &path.segments;
    if segs[0] == "제" || segs[0] == ctx.actor_name || ctx.actor_names.contains(&segs[0]) {
        return Some(Expr::Path(lower_path(path, ctx)?));
    }
    if let Some(binding) = ctx.payload_binding {
        if segs[0] == binding {
            if segs.len() != 2 {
                return None;
            }
            return ctx
                .payload_fields
                .get(&segs[1])
                .cloned()
                .map(group_payload_field_expr);
        }
    }
    if ctx.payload_binding.is_none() && ctx.payload_kind != "" && segs[0] == "정보" {
        if segs.len() != 2 {
            return None;
        }
        return ctx
            .payload_fields
            .get(&segs[1])
            .cloned()
            .map(group_payload_field_expr);
    }
    if let Some(binding) = ctx.alert_binding {
        if segs[0] == binding {
            if segs.len() == 2 && segs[1] == "이름" {
                return Some(Expr::Literal(Literal::Str(ctx.payload_kind.to_string())));
            }
            if segs.len() == 3 && segs[1] == "정보" {
                return ctx
                    .payload_fields
                    .get(&segs[2])
                    .cloned()
                    .map(group_payload_field_expr);
            }
            return None;
        }
    }
    if ctx.alert_binding.is_none() && ctx.payload_kind != "" && segs[0] == "알림" {
        if segs.len() == 2 && segs[1] == "이름" {
            return Some(Expr::Literal(Literal::Str(ctx.payload_kind.to_string())));
        }
        if segs.len() == 3 && segs[1] == "정보" {
            return ctx
                .payload_fields
                .get(&segs[2])
                .cloned()
                .map(group_payload_field_expr);
        }
        return None;
    }
    Some(Expr::Path(path.clone()))
}

fn eval_static_bool_expr(expr: &Expr) -> Option<bool> {
    match expr {
        Expr::Grouped { expr } => eval_static_bool_expr(expr),
        Expr::Literal(Literal::Bool(value)) => Some(*value),
        Expr::Binary { left, op, right } => {
            let left_lit = match &**left {
                Expr::Literal(lit) => lit,
                _ => return None,
            };
            let right_lit = match &**right {
                Expr::Literal(lit) => lit,
                _ => return None,
            };
            match op {
                BinaryOp::Eq => Some(format_literal(left_lit) == format_literal(right_lit)),
                BinaryOp::NotEq => Some(format_literal(left_lit) != format_literal(right_lit)),
                _ => None,
            }
        }
        _ => None,
    }
}

