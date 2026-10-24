//! Authored-source validation, feature gates and observation discovery; unchanged bodies.
use super::{
    AgeTarget,
    Assertion,
    BTreeMap,
    BTreeSet,
    Body,
    CanonProgram,
    E_V1_CORE_RECURSION_UNSUPPORTED,
    Expr,
    ExprKind,
    Fixed64,
    Formula,
    HashMap,
    HashSet,
    Lexer,
    Literal,
    OriginalObservationFieldRef,
    SeedDef,
    StateMachine,
    Stmt,
    Token,
    TokenKind,
    TopLevelItem,
    age_not_available_error,
    func,
};

pub(super) fn validate_v1_core_no_named_callable_recursion(program: &CanonProgram) -> Result<(), String> {
    let mut alias_owner = BTreeMap::<String, String>::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        for surface in seed.callable_surface_names() {
            alias_owner.insert(surface, seed.canonical_name.clone());
        }
    }

    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        let mut calls = BTreeSet::new();
        for param in &seed.params {
            if let Some(default) = &param.default_value {
                collect_v1_core_named_calls_expr(default, &alias_owner, &mut calls);
            }
        }
        if let Some(body) = &seed.body {
            collect_v1_core_named_calls_body(body, &alias_owner, &mut calls);
        }
        graph
            .entry(seed.canonical_name.clone())
            .or_default()
            .extend(calls);
    }

    fn visit(
        node: &str,
        graph: &BTreeMap<String, BTreeSet<String>>,
        states: &mut BTreeMap<String, u8>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        states.insert(node.to_string(), 1);
        stack.push(node.to_string());
        if let Some(next_nodes) = graph.get(node) {
            for next in next_nodes {
                match states.get(next).copied().unwrap_or(0) {
                    0 => {
                        if let Some(cycle) = visit(next, graph, states, stack) {
                            return Some(cycle);
                        }
                    }
                    1 => {
                        let start = stack.iter().position(|item| item == next).unwrap_or(0);
                        let mut cycle = stack[start..].to_vec();
                        cycle.push(next.clone());
                        return Some(cycle);
                    }
                    _ => {}
                }
            }
        }
        stack.pop();
        states.insert(node.to_string(), 2);
        None
    }

    let mut states = BTreeMap::<String, u8>::new();
    for node in graph.keys() {
        if states.get(node).copied().unwrap_or(0) == 0 {
            if let Some(cycle) = visit(node, &graph, &mut states, &mut Vec::new()) {
                return Err(format!(
                    "{E_V1_CORE_RECURSION_UNSUPPORTED}: V1-Core는 이름 있는 callable 재귀를 지원하지 않습니다; cycle={}",
                    cycle.join(" -> ")
                ));
            }
        }
    }
    Ok(())
}

fn collect_v1_core_named_calls_body(
    body: &Body,
    alias_owner: &BTreeMap<String, String>,
    calls: &mut BTreeSet<String>,
) {
    for stmt in &body.stmts {
        match stmt {
            Stmt::SumDefinition { .. } => {}
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                collect_v1_core_named_calls_expr(subject, alias_owner, calls);
                for arm in arms {
                    collect_v1_core_named_calls_body(&arm.body, alias_owner, calls);
                }
                if let Some(body) = default_body {
                    collect_v1_core_named_calls_body(body, alias_owner, calls);
                }
            }
            Stmt::PublicObservation { fields, .. } => {
                for field in fields {
                    collect_v1_core_named_calls_expr(&field.value, alias_owner, calls);
                }
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    if let Some(value) = &item.value {
                        collect_v1_core_named_calls_expr(value, alias_owner, calls);
                    }
                }
            }
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => {
                collect_v1_core_named_calls_expr(value, alias_owner, calls)
            }
            Stmt::Mutate { target, value, .. } => {
                collect_v1_core_named_calls_expr(target, alias_owner, calls);
                collect_v1_core_named_calls_expr(value, alias_owner, calls);
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                collect_v1_core_named_calls_expr(producer, alias_owner, calls);
                collect_v1_core_named_calls_expr(destination, alias_owner, calls);
            }
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                collect_v1_core_named_calls_expr(expr, alias_owner, calls)
            }
            Stmt::Receive {
                condition, body, ..
            } => {
                if let Some(condition) = condition {
                    collect_v1_core_named_calls_expr(condition, alias_owner, calls);
                }
                collect_v1_core_named_calls_body(body, alias_owner, calls);
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                if let Some(sender) = sender {
                    collect_v1_core_named_calls_expr(sender, alias_owner, calls);
                }
                collect_v1_core_named_calls_expr(payload, alias_owner, calls);
                collect_v1_core_named_calls_expr(receiver, alias_owner, calls);
            }
            Stmt::Return { value, .. } => {
                collect_v1_core_named_calls_expr(value, alias_owner, calls)
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                collect_v1_core_named_calls_expr(condition, alias_owner, calls);
                collect_v1_core_named_calls_body(then_body, alias_owner, calls);
                if let Some(else_body) = else_body {
                    collect_v1_core_named_calls_body(else_body, alias_owner, calls);
                }
            }
            Stmt::Try { action, body, .. } => {
                collect_v1_core_named_calls_expr(action, alias_owner, calls);
                collect_v1_core_named_calls_body(body, alias_owner, calls);
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_v1_core_named_calls_expr(&branch.condition, alias_owner, calls);
                    collect_v1_core_named_calls_body(&branch.body, alias_owner, calls);
                }
                collect_v1_core_named_calls_body(else_body, alias_owner, calls);
            }
            Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => {
                collect_v1_core_named_calls_body(body, alias_owner, calls)
            }
            Stmt::While {
                condition, body, ..
            }
            | Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            }
            | Stmt::Guard {
                condition, body, ..
            } => {
                collect_v1_core_named_calls_expr(condition, alias_owner, calls);
                collect_v1_core_named_calls_body(body, alias_owner, calls);
            }
            Stmt::ForEach { iterable, body, .. } => {
                collect_v1_core_named_calls_expr(iterable, alias_owner, calls);
                collect_v1_core_named_calls_body(body, alias_owner, calls);
            }
            Stmt::Quantifier { body, .. } => {
                collect_v1_core_named_calls_body(body, alias_owner, calls)
            }
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                collect_v1_core_named_calls_expr(condition, alias_owner, calls);
                if let Some(then_body) = then_body {
                    collect_v1_core_named_calls_body(then_body, alias_owner, calls);
                }
                collect_v1_core_named_calls_body(else_body, alias_owner, calls);
            }
            Stmt::MetaBlock { .. }
            | Stmt::Pragma { .. }
            | Stmt::Break { .. }
            | Stmt::ContinueLoop { .. } => {}
        }
    }
}

pub(super) fn validate_v25_known_infix_operand_types(program: &CanonProgram) -> Result<(), String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum KnownType {
        Number,
        Text,
        Other,
        Unknown,
    }

    fn type_label(kind: KnownType) -> &'static str {
        match kind {
            KnownType::Number => "수",
            KnownType::Text => "글",
            KnownType::Other => "다른 값",
            KnownType::Unknown => "미확정",
        }
    }

    fn infer_expr(expr: &Expr, bindings: &HashMap<String, KnownType>, checking_product: bool) -> Result<KnownType, String> {
        match &expr.kind {
            ExprKind::Literal(Literal::Int(_) | Literal::Fixed64(_)) => Ok(KnownType::Number),
            ExprKind::Literal(Literal::String(_) | Literal::Atom(_)) => Ok(KnownType::Text),
            ExprKind::Literal(_) => Ok(KnownType::Other),
            ExprKind::Var(name) => Ok(*bindings.get(name).unwrap_or(&KnownType::Unknown)),
            ExprKind::Call { args, func } => {
                for arg in args {
                    infer_expr(&arg.expr, bindings, checking_product)?;
                }
                if func == "글로" {
                    Ok(KnownType::Text)
                } else {
                    Ok(KnownType::Unknown)
                }
            }
            ExprKind::Infix { left, op, right } if op == "+" => {
                let left_kind = infer_expr(left, bindings, checking_product)?;
                let right_kind = infer_expr(right, bindings, checking_product)?;
                match (left_kind, right_kind) {
                    (KnownType::Text, KnownType::Text) => Ok(KnownType::Text),
                    (KnownType::Number, KnownType::Number) => Ok(KnownType::Number),
                    (KnownType::Unknown, _) | (_, KnownType::Unknown) => Ok(KnownType::Unknown),
                    (KnownType::Text, _) if checking_product => Err(format!(
                        "[E_RUNTIME_TYPE_MISMATCH] 핀=더하기 오른쪽 피연산자 기대=글 실제={}",
                        type_label(right_kind)
                    )),
                    (_, KnownType::Text) if checking_product => Err(format!(
                        "[E_RUNTIME_TYPE_MISMATCH] 핀=더하기 왼쪽 피연산자 기대=글 실제={}",
                        type_label(left_kind)
                    )),
                    _ => Ok(KnownType::Other),
                }
            }
            ExprKind::Infix { left, right, .. } => {
                infer_expr(left, bindings, checking_product)?;
                infer_expr(right, bindings, checking_product)?;
                Ok(KnownType::Unknown)
            }
            ExprKind::FieldAccess { target, .. }
            | ExprKind::SeedLiteral { body: target, .. }
            | ExprKind::Suffix { value: target, .. }
            | ExprKind::Eval { thunk: target, .. }
            | ExprKind::Nuance { expr: target, .. } => {
                infer_expr(target, bindings, checking_product)?;
                Ok(KnownType::Unknown)
            }
            ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } | ExprKind::Pipe { stages: elements } => {
                for element in elements {
                    infer_expr(element, bindings, checking_product)?;
                }
                Ok(KnownType::Other)
            }
            ExprKind::SelectionPlace { target, index } => {
                infer_expr(target, bindings, checking_product)?;
                infer_expr(index, bindings, checking_product)?;
                Ok(KnownType::Unknown)
            }
            ExprKind::Pack { fields } => {
                for (_, value) in fields {
                    infer_expr(value, bindings, checking_product)?;
                }
                Ok(KnownType::Other)
            }
            ExprKind::ProductConstruction { fields, .. } => {
                for field in fields {
                    infer_expr(&field.expr, bindings, true)?;
                }
                Ok(KnownType::Other)
            }
            ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
                for (_, value) in inject {
                    infer_expr(value, bindings, checking_product)?;
                }
                Ok(KnownType::Unknown)
            }
            _ => Ok(KnownType::Unknown),
        }
    }

    fn validate_stmts(
        stmts: &[Stmt],
        bindings: &mut HashMap<String, KnownType>,
    ) -> Result<(), String> {
        for stmt in stmts {
        match stmt {
            Stmt::SumDefinition { name, .. } => {
                bindings.insert(name.clone(), KnownType::Other);
            }
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                infer_expr(subject, bindings, false)?;
                for arm in arms {
                    validate_stmts(&arm.body.stmts, &mut bindings.clone())?;
                }
                if let Some(body) = default_body {
                    validate_stmts(&body.stmts, &mut bindings.clone())?;
                }
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    if let Some(value) = &item.value {
                        let value_kind = infer_expr(value, bindings, false)?;
                        bindings.insert(item.name.clone(), value_kind);
                    }
                }
            }
            Stmt::Mutate { target, value, .. } => {
                let value_kind = infer_expr(value, &bindings, false)?;
                if let ExprKind::Var(name) = &target.kind {
                    bindings.insert(name.clone(), value_kind);
                }
            }
            Stmt::Definition { name, value, .. } | Stmt::DerivedBinding {name,value,..} => {
                let value_kind = infer_expr(value, &bindings, false)?;
                bindings.insert(name.clone(), value_kind);
            }
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                infer_expr(expr, &bindings, false)?;
            }
            Stmt::Return { value, .. } => {
                infer_expr(value, bindings, false)?;
            }
            Stmt::PublicObservation { fields, .. } => {
                for field in fields {
                    infer_expr(&field.value, bindings, false)?;
                }
            }
                Stmt::ResultFlow {
                    producer,
                    destination,
                    ..
                } => {
                infer_expr(producer, bindings, false)?;
                infer_expr(destination, bindings, false)?;
            }
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                infer_expr(condition, bindings, false)?;
                validate_stmts(&then_body.stmts, &mut bindings.clone())?;
                if let Some(else_body) = else_body {
                    validate_stmts(&else_body.stmts, &mut bindings.clone())?;
                }
            }
                Stmt::Receive {
                    condition, body, ..
                } => {
                if let Some(condition) = condition {
                    infer_expr(condition, bindings, false)?;
                }
                validate_stmts(&body.stmts, &mut bindings.clone())?;
            }
            Stmt::Try { action, body, .. } => {
                infer_expr(action, bindings, false)?;
                validate_stmts(&body.stmts, &mut bindings.clone())?;
            }
                Stmt::Choose {
                    branches,
                    else_body,
                    ..
                } => {
                for branch in branches {
                    infer_expr(&branch.condition, bindings, false)?;
                    validate_stmts(&branch.body.stmts, &mut bindings.clone())?;
                }
                validate_stmts(&else_body.stmts, &mut bindings.clone())?;
            }
                Stmt::Repeat { body, .. }
                | Stmt::BeatBlock { body, .. }
                | Stmt::Hook { body, .. }
                | Stmt::Quantifier { body, .. } => {
                    validate_stmts(&body.stmts, &mut bindings.clone())?
                }
                Stmt::While {
                    condition, body, ..
                }
                | Stmt::HookWhenBecomes {
                    condition, body, ..
                }
                | Stmt::HookWhile {
                    condition, body, ..
                }
                | Stmt::Guard {
                    condition, body, ..
                } => {
                infer_expr(condition, bindings, false)?;
                validate_stmts(&body.stmts, &mut bindings.clone())?;
            }
            Stmt::ForEach { iterable, body, .. } => {
                infer_expr(iterable, bindings, false)?;
                validate_stmts(&body.stmts, &mut bindings.clone())?;
            }
                Stmt::Contract {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                infer_expr(condition, bindings, false)?;
                if let Some(then_body) = then_body {
                    validate_stmts(&then_body.stmts, &mut bindings.clone())?;
                }
                validate_stmts(&else_body.stmts, &mut bindings.clone())?;
            }
                Stmt::Send {
                    sender,
                    payload,
                    receiver,
                    ..
                } => {
                if let Some(sender) = sender {
                    infer_expr(sender, bindings, false)?;
                }
                infer_expr(payload, bindings, false)?;
                infer_expr(receiver, bindings, false)?;
            }
                Stmt::MetaBlock { .. }
                | Stmt::Pragma { .. }
                | Stmt::Break { .. }
                | Stmt::ContinueLoop { .. } => {}
        }
    }
        Ok(())
    }

    validate_stmts(&program.root_stmts, &mut HashMap::new())?;
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            validate_stmts(&body.stmts, &mut HashMap::new())?;
        }
    }
    Ok(())
}


fn collect_v1_core_named_calls_expr(
    expr: &Expr,
    alias_owner: &BTreeMap<String, String>,
    calls: &mut BTreeSet<String>,
) {
    match &expr.kind {
        ExprKind::SumBranchValue { payload, .. } => {
            for payload in payload {
                collect_v1_core_named_calls_expr(&payload.value, alias_owner, calls);
            }
        }
        ExprKind::Tensor(value) => collect_v1_core_named_calls_expr(value.input(), alias_owner, calls),
        ExprKind::MultiAxisSelection(value) => { for child in value.children() { collect_v1_core_named_calls_expr(child, alias_owner, calls); } }
        ExprKind::FieldAccess { target, .. }
        | ExprKind::SeedLiteral { body: target, .. }
        | ExprKind::Suffix { value: target, .. }
        | ExprKind::Eval { thunk: target, .. }
        | ExprKind::Nuance { expr: target, .. } => {
            collect_v1_core_named_calls_expr(target, alias_owner, calls)
        }
        ExprKind::Call { args, func } => {
            if let Some(owner) = alias_owner.get(func) {
                calls.insert(owner.clone());
            }
            for arg in args {
                collect_v1_core_named_calls_expr(&arg.expr, alias_owner, calls);
            }
        }
        ExprKind::ProductConstruction { fields, .. } => {
            for field in fields {
                collect_v1_core_named_calls_expr(&field.expr, alias_owner, calls);
            }
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                collect_v1_core_named_calls_expr(element, alias_owner, calls);
            }
        }
        ExprKind::SelectionPlace { target, index } => {
            collect_v1_core_named_calls_expr(target, alias_owner, calls);
            collect_v1_core_named_calls_expr(index, alias_owner, calls);
        }
        ExprKind::Infix { left, right, .. } => {
            collect_v1_core_named_calls_expr(left, alias_owner, calls);
            collect_v1_core_named_calls_expr(right, alias_owner, calls);
        }
        ExprKind::Thunk(body) => collect_v1_core_named_calls_body(body, alias_owner, calls),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                collect_v1_core_named_calls_expr(stage, alias_owner, calls);
            }
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                collect_v1_core_named_calls_expr(value, alias_owner, calls);
            }
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                collect_v1_core_named_calls_expr(value, alias_owner, calls);
            }
        }
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Assertion(_)
        | ExprKind::StateMachine(_)
        | ExprKind::Formula(_)
        | ExprKind::Template(_) => {}
    }
}

pub(super) fn scan_original_public_observation_fields(
    source: &str,
) -> Result<Vec<OriginalObservationFieldRef>, String> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|err| format!("원문 공개 관찰 위치 토큰화 실패: {}", err.message))?;
    let mut fields = Vec::new();
    for boim in 1..tokens.len() {
        if tokens[boim].raw != "내보임" || !matches!(tokens[boim - 1].kind, TokenKind::RParen) {
            continue;
        }
        let mut depth = 0usize;
        let mut open = None;
        for index in (0..boim).rev() {
            match tokens[index].kind {
                TokenKind::RParen => depth += 1,
                TokenKind::LParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        open = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(open) = open else {
            continue;
        };
        let end = boim - 1;
        let mut cursor = open + 1;
        while cursor < end {
            let name_token = &tokens[cursor];
            let name = token_observation_name(name_token, source).ok_or_else(|| {
                "E_BOIM_SOURCE_REF_MISMATCH: 원문 공개 관찰 필드 이름을 찾을 수 없습니다"
                    .to_string()
            })?;
            fields.push(OriginalObservationFieldRef {
                name,
                start: name_token.span.start,
                end: name_token.span.end,
            });
            cursor += 1;
            if cursor >= end {
                break;
            }
            if !matches!(tokens[cursor].kind, TokenKind::Colon) {
                return Err(
                    "E_BOIM_SOURCE_REF_MISMATCH: 원문 공개 관찰 필드 구분자를 찾을 수 없습니다"
                        .to_string(),
                );
            }
            cursor += 1;
            let mut paren = 0usize;
            let mut bracket = 0usize;
            let mut brace = 0usize;
            while cursor < end {
                match tokens[cursor].kind {
                    TokenKind::LParen => paren += 1,
                    TokenKind::RParen => paren = paren.saturating_sub(1),
                    TokenKind::LBracket => bracket += 1,
                    TokenKind::RBracket => bracket = bracket.saturating_sub(1),
                    TokenKind::LBrace => brace += 1,
                    TokenKind::RBrace => brace = brace.saturating_sub(1),
                    TokenKind::Comma if paren == 0 && bracket == 0 && brace == 0 => {
                        cursor += 1;
                        break;
                    }
                    _ => {}
                }
                cursor += 1;
            }
        }
    }
    Ok(fields)
}

pub(super) fn token_observation_name(token: &Token, source: &str) -> Option<String> {
    match &token.kind {
        TokenKind::Ident(name) | TokenKind::Josa(name) => source
            .get(token.span.start..token.span.end)
            .map(str::to_string)
            .or_else(|| Some(name.clone())),
        _ => None,
    }
}

pub(super) fn collect_public_observation_fields(
    program: &CanonProgram,
) -> Vec<&ddonirang_lang::PublicObservationField> {
    let mut fields = Vec::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        for param in &seed.params {
            if let Some(default) = &param.default_value {
                collect_public_observation_fields_expr(default, &mut fields);
            }
        }
        if let Some(body) = &seed.body {
            collect_public_observation_fields_body(body, &mut fields);
        }
    }
    collect_public_observation_fields_stmts(&program.root_stmts, &mut fields);
    fields.sort_by_key(|field| field.span.start);
    fields
}

fn collect_public_observation_fields_body<'a>(
    body: &'a Body,
    fields: &mut Vec<&'a ddonirang_lang::PublicObservationField>,
) {
    collect_public_observation_fields_stmts(&body.stmts, fields);
}

fn collect_public_observation_fields_stmts<'a>(
    stmts: &'a [Stmt],
    fields: &mut Vec<&'a ddonirang_lang::PublicObservationField>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::SumDefinition { .. } => {}
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                collect_public_observation_fields_expr(subject, fields);
                for arm in arms {
                    collect_public_observation_fields_body(&arm.body, fields);
                }
                if let Some(body) = default_body {
                    collect_public_observation_fields_body(body, fields);
                }
            }
            Stmt::PublicObservation {
                fields: observed, ..
            } => {
                fields.extend(observed);
                for field in observed {
                    collect_public_observation_fields_expr(&field.value, fields);
                }
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    if let Some(value) = &item.value {
                        collect_public_observation_fields_expr(value, fields);
                    }
                }
            }
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => {
                collect_public_observation_fields_expr(value, fields);
            }
            Stmt::Mutate { target, value, .. } => {
                collect_public_observation_fields_expr(target, fields);
                collect_public_observation_fields_expr(value, fields);
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                collect_public_observation_fields_expr(producer, fields);
                collect_public_observation_fields_expr(destination, fields);
            }
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                collect_public_observation_fields_expr(expr, fields);
            }
            Stmt::Receive {
                condition, body, ..
            } => {
                if let Some(condition) = condition {
                    collect_public_observation_fields_expr(condition, fields);
                }
                collect_public_observation_fields_body(body, fields);
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                if let Some(sender) = sender {
                    collect_public_observation_fields_expr(sender, fields);
                }
                collect_public_observation_fields_expr(payload, fields);
                collect_public_observation_fields_expr(receiver, fields);
            }
            Stmt::Return { value, .. } => {
                collect_public_observation_fields_expr(value, fields);
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                collect_public_observation_fields_expr(condition, fields);
                collect_public_observation_fields_body(then_body, fields);
                if let Some(else_body) = else_body {
                    collect_public_observation_fields_body(else_body, fields);
                }
            }
            Stmt::Try { action, body, .. } => {
                collect_public_observation_fields_expr(action, fields);
                collect_public_observation_fields_body(body, fields);
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_public_observation_fields_expr(&branch.condition, fields);
                    collect_public_observation_fields_body(&branch.body, fields);
                }
                collect_public_observation_fields_body(else_body, fields);
            }
            Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => {
                collect_public_observation_fields_body(body, fields);
            }
            Stmt::While {
                condition, body, ..
            }
            | Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            }
            | Stmt::Guard {
                condition, body, ..
            } => {
                collect_public_observation_fields_expr(condition, fields);
                collect_public_observation_fields_body(body, fields);
            }
            Stmt::ForEach { iterable, body, .. } => {
                collect_public_observation_fields_expr(iterable, fields);
                collect_public_observation_fields_body(body, fields);
            }
            Stmt::Quantifier { body, .. } => {
                collect_public_observation_fields_body(body, fields);
            }
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                collect_public_observation_fields_expr(condition, fields);
                if let Some(then_body) = then_body {
                    collect_public_observation_fields_body(then_body, fields);
                }
                collect_public_observation_fields_body(else_body, fields);
            }
            Stmt::MetaBlock { .. }
            | Stmt::Pragma { .. }
            | Stmt::Break { .. }
            | Stmt::ContinueLoop { .. } => {}
        }
    }
}

fn collect_public_observation_fields_expr<'a>(
    expr: &'a Expr,
    fields: &mut Vec<&'a ddonirang_lang::PublicObservationField>,
) {
    match &expr.kind {
        ExprKind::SumBranchValue { payload, .. } => {
            for payload in payload {
                collect_public_observation_fields_expr(&payload.value, fields);
            }
        }
        ExprKind::Tensor(value) => collect_public_observation_fields_expr(value.input(), fields),
        ExprKind::MultiAxisSelection(value) => { for child in value.children() { collect_public_observation_fields_expr(child, fields); } }
        ExprKind::FieldAccess { target, .. }
        | ExprKind::SeedLiteral { body: target, .. }
        | ExprKind::Suffix { value: target, .. }
        | ExprKind::Eval { thunk: target, .. }
        | ExprKind::Nuance { expr: target, .. } => {
            collect_public_observation_fields_expr(target, fields);
        }
        ExprKind::Call { args, .. } | ExprKind::ProductConstruction { fields: args, .. } => {
            for arg in args {
                collect_public_observation_fields_expr(&arg.expr, fields);
            }
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                collect_public_observation_fields_expr(element, fields);
            }
        }
        ExprKind::SelectionPlace { target, index } => {
            collect_public_observation_fields_expr(target, fields);
            collect_public_observation_fields_expr(index, fields);
        }
        ExprKind::Infix { left, right, .. } => {
            collect_public_observation_fields_expr(left, fields);
            collect_public_observation_fields_expr(right, fields);
        }
        ExprKind::Thunk(body) => collect_public_observation_fields_body(body, fields),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                collect_public_observation_fields_expr(stage, fields);
            }
        }
        ExprKind::Pack {
            fields: pack_fields,
        } => {
            for (_, value) in pack_fields {
                collect_public_observation_fields_expr(value, fields);
            }
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                collect_public_observation_fields_expr(value, fields);
            }
        }
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Assertion(_)
        | ExprKind::StateMachine(_)
        | ExprKind::Formula(_)
        | ExprKind::Template(_) => {}
    }
}

pub(super) fn collect_top_level_decl_names(source: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    for range in top_level_decl_ranges(source) { collect_decl_names_from_block(&source[range],&mut names); }
    names
}

pub(super) fn authored_top_level_decl_names(program:&CanonProgram,source:&str) -> HashSet<String> {
    let ranges=top_level_decl_ranges(source);
    let mut names=HashSet::new();
    let in_range=|span:ddonirang_lang::Span| ranges.iter().any(|range|range.start<=span.start && span.end<=range.end);
    let mut statements=program.root_stmts.iter().collect::<Vec<_>>();
    for ddonirang_lang::TopLevelItem::SeedDef(seed) in &program.items {
        if let Some(body)=&seed.body {statements.extend(body.stmts.iter());}
    }
    for statement in statements {
        match statement {
            Stmt::DeclBlock{items,..} => for item in items {if in_range(item.span) {names.insert(item.name.clone());}},
            Stmt::Definition{name,span,..} if in_range(*span) => {names.insert(name.clone());},
            _=>{},
        }
    }
    names
}

fn top_level_decl_ranges(source:&str) -> Vec<std::ops::Range<usize>> {
    let mut ranges=Vec::new();
    let mut depth = 0usize;
    let mut i = 0usize;
    let mut in_string = false;
    while i < source.len() {
        let rest = &source[i..];
        let Some(ch) = rest.chars().next() else {
            break;
        };
        if in_string {
            if ch == '"' && !source[..i].ends_with('\\') {
                in_string = false;
            }
            i += ch.len_utf8();
            continue;
        }
        if ch == '"' {
            in_string = true;
            i += ch.len_utf8();
            continue;
        }
        if depth == 0 && rest.starts_with("채비") && is_ident_boundary(source, i, "채비") {
            let mut cursor = i + "채비".len();
            cursor = skip_ws(source, cursor);
            if source[cursor..].starts_with(':') {
                cursor += ':'.len_utf8();
                cursor = skip_ws(source, cursor);
            }
            if source[cursor..].starts_with('{') {
                if let Some(close) = find_matching_brace(source, cursor) {
                    ranges.push(cursor+1..close);
                    i = close + '}'.len_utf8();
                    continue;
                }
            }
        }
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        i += ch.len_utf8();
    }
    ranges
}

fn is_ident_boundary(source: &str, start: usize, keyword: &str) -> bool {
    let before = source[..start].chars().next_back();
    let after = source[start + keyword.len()..].chars().next();
    !before.is_some_and(is_ident_char) && !after.is_some_and(is_ident_char)
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

fn skip_ws(source: &str, mut idx: usize) -> usize {
    while idx < source.len() {
        let Some(ch) = source[idx..].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        idx += ch.len_utf8();
    }
    idx
}

fn find_matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut i = open;
    while i < source.len() {
        let ch = source[i..].chars().next()?;
        if in_string {
            if ch == '"' && !source[..i].ends_with('\\') {
                in_string = false;
            }
            i += ch.len_utf8();
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += ch.len_utf8();
    }
    None
}

fn collect_decl_names_from_block(block: &str, names: &mut HashSet<String>) {
    for line in block.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some(colon) = trimmed.find(':') else {
            continue;
        };
        let name = trimmed[..colon].trim();
        if !name.is_empty() && name.chars().all(is_ident_char) {
            names.insert(name.to_string());
        }
    }
}
pub fn extract_setting_madi(source: &str) -> Result<Option<u64>, String> {
    let mut search_start = 0;
    while let Some(rel_idx) = source[search_start..].find("설정") {
        let setting_start = search_start + rel_idx;
        let after_name = setting_start + "설정".len();
        let rest = &source[after_name..];
        let leading_ws = rest
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
            .map(|(idx, _)| idx)
            .unwrap_or(rest.len());
        let brace_idx = after_name + leading_ws;
        if source[brace_idx..].starts_with('{') {
            if let Some((body, end_idx)) = extract_braced_body(source, brace_idx) {
                if let Some(value) = parse_madi_from_setting_body(body)? {
                    return Ok(Some(value));
                }
                search_start = end_idx;
                continue;
            }
        }
        search_start = after_name;
    }
    Ok(None)
}

fn extract_braced_body(source: &str, open_brace_idx: usize) -> Option<(&str, usize)> {
    let mut depth = 0usize;
    let mut body_start = None;
    for (rel_idx, ch) in source[open_brace_idx..].char_indices() {
        let idx = open_brace_idx + rel_idx;
        match ch {
            '{' => {
                depth += 1;
                if body_start.is_none() {
                    body_start = Some(idx + ch.len_utf8());
                }
            }
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return body_start.map(|start| (&source[start..idx], idx + ch.len_utf8()));
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_madi_from_setting_body(body: &str) -> Result<Option<u64>, String> {
    let Some((_, after_key)) = find_setting_key(body, "마디수") else {
        return Ok(None);
    };
    let after_ws = after_key.trim_start();
    let after_colon = after_ws[1..].trim_start();
    let raw_value = after_colon
        .split(|ch| ch == '.' || ch == '\n' || ch == '\r' || ch == '}')
        .next()
        .unwrap_or("")
        .trim();
    if raw_value.is_empty()
        || raw_value.starts_with('-')
        || !raw_value.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(setting_madi_error());
    }
    let value = raw_value.parse::<u64>().map_err(|_| setting_madi_error())?;
    if value == 0 {
        return Err(setting_madi_error());
    }
    Ok(Some(value))
}

fn find_setting_key<'a>(body: &'a str, key: &str) -> Option<(usize, &'a str)> {
    let mut search_start = 0;
    while let Some(rel_idx) = body[search_start..].find(key) {
        let key_idx = search_start + rel_idx;
        let before = body[..key_idx].chars().rev().find(|ch| !ch.is_whitespace());
        let after_key = &body[key_idx + key.len()..];
        let after_ws = after_key.trim_start();
        let key_boundary = match before {
            Some(ch) => ch == '.' || ch == '{' || ch == '\n' || ch == '\r',
            None => true,
        };
        if key_boundary && after_ws.starts_with(':') {
            return Some((key_idx, after_ws));
        }
        search_start = key_idx + key.len();
    }
    None
}

fn setting_madi_error() -> String {
    "E_SETTING_MADI_BAD_VALUE 설정 마디수는 1 이상의 정수여야 합니다.".to_string()
}

pub(super) fn enforce_regex_age_gate(program: &CanonProgram, age_target: AgeTarget) -> Result<(), String> {
    if age_target >= AgeTarget::Age3 {
        return Ok(());
    }
    if let Some(feature) = detect_regex_feature(program) {
        return Err(age_not_available_error(
            feature,
            AgeTarget::Age3,
            age_target,
        ));
    }
    Ok(())
}

pub(super) fn enforce_assertion_age_gate(program: &CanonProgram, age_target: AgeTarget) -> Result<(), String> {
    if age_target >= AgeTarget::Age1 {
        return Ok(());
    }
    if let Some(feature) = detect_assertion_feature(program) {
        return Err(age_not_available_error(
            feature,
            AgeTarget::Age1,
            age_target,
        ));
    }
    Ok(())
}

pub(super) fn enforce_state_machine_age_gate(
    program: &CanonProgram,
    age_target: AgeTarget,
) -> Result<(), String> {
    if age_target >= AgeTarget::Age1 {
        return Ok(());
    }
    if let Some(feature) = detect_state_machine_feature(program) {
        return Err(age_not_available_error(
            feature,
            AgeTarget::Age1,
            age_target,
        ));
    }
    Ok(())
}

pub(super) fn enforce_quantifier_age_gate(
    program: &CanonProgram,
    age_target: AgeTarget,
) -> Result<(), String> {
    if age_target >= AgeTarget::Age4 {
        return Ok(());
    }
    if let Some(feature) = detect_quantifier_feature(program) {
        return Err(age_not_available_error(
            feature,
            AgeTarget::Age4,
            age_target,
        ));
    }
    Ok(())
}

fn detect_regex_feature(program: &CanonProgram) -> Option<&'static str> {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            if let Some(feature) = body_regex_feature(body) {
                return Some(feature);
            }
        }
    }
    None
}

fn detect_assertion_feature(program: &CanonProgram) -> Option<&'static str> {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            if let Some(feature) = body_assertion_feature(body) {
                return Some(feature);
            }
        }
    }
    None
}

fn detect_state_machine_feature(program: &CanonProgram) -> Option<&'static str> {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            if let Some(feature) = body_state_machine_feature(body) {
                return Some(feature);
            }
        }
    }
    None
}

fn detect_quantifier_feature(program: &CanonProgram) -> Option<&'static str> {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            if let Some(feature) = body_quantifier_feature(body) {
                return Some(feature);
            }
        }
    }
    None
}

fn body_regex_feature(body: &Body) -> Option<&'static str> {
    for stmt in &body.stmts {
        if let Some(feature) = stmt_regex_feature(stmt) {
            return Some(feature);
        }
    }
    None
}

fn body_assertion_feature(body: &Body) -> Option<&'static str> {
    for stmt in &body.stmts {
        if let Some(feature) = stmt_assertion_feature(stmt) {
            return Some(feature);
        }
    }
    None
}

fn body_state_machine_feature(body: &Body) -> Option<&'static str> {
    for stmt in &body.stmts {
        if let Some(feature) = stmt_state_machine_feature(stmt) {
            return Some(feature);
        }
    }
    None
}

fn body_quantifier_feature(body: &Body) -> Option<&'static str> {
    for stmt in &body.stmts {
        if let Some(feature) = stmt_quantifier_feature(stmt) {
            return Some(feature);
        }
    }
    None
}

fn stmt_regex_feature(stmt: &Stmt) -> Option<&'static str> {
    match stmt {
        Stmt::SumDefinition { .. } => None,
        Stmt::SumMatch { subject, arms, default_body, .. } => {
            expr_regex_feature(subject)
                .or_else(|| arms.iter().find_map(|arm| body_regex_feature(&arm.body)))
                .or_else(|| default_body.as_ref().and_then(body_regex_feature))
        }
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    if let Some(feature) = expr_regex_feature(value) {
                        return Some(feature);
                    }
                }
            }
            None
        }
        Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => expr_regex_feature(value),
        Stmt::Mutate { target, value, .. } => {
            expr_regex_feature(target).or_else(|| expr_regex_feature(value))
        }
        Stmt::ResultFlow {
            producer,
            destination,
            ..
        } => expr_regex_feature(producer).or_else(|| expr_regex_feature(destination)),
        Stmt::Receive {
            condition, body, ..
        } => condition
            .as_ref()
            .and_then(expr_regex_feature)
            .or_else(|| body_regex_feature(body)),
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => sender
            .as_ref()
            .and_then(expr_regex_feature)
            .or_else(|| expr_regex_feature(payload))
            .or_else(|| expr_regex_feature(receiver)),
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            expr_regex_feature(expr)
        }
        Stmt::Return { value, .. } => expr_regex_feature(value),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => expr_regex_feature(condition)
            .or_else(|| body_regex_feature(then_body))
            .or_else(|| else_body.as_ref().and_then(body_regex_feature)),
        Stmt::Try { action, body, .. } => {
            expr_regex_feature(action).or_else(|| body_regex_feature(body))
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                if let Some(feature) = expr_regex_feature(&branch.condition) {
                    return Some(feature);
                }
                if let Some(feature) = body_regex_feature(&branch.body) {
                    return Some(feature);
                }
            }
            body_regex_feature(else_body)
        }
        Stmt::Repeat { body, .. } => body_regex_feature(body),
        Stmt::While {
            condition, body, ..
        } => expr_regex_feature(condition).or_else(|| body_regex_feature(body)),
        Stmt::ForEach { iterable, body, .. } => {
            expr_regex_feature(iterable).or_else(|| body_regex_feature(body))
        }
        Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => body_regex_feature(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_regex_feature(condition).or_else(|| body_regex_feature(body)),
        Stmt::Quantifier { body, .. } => body_regex_feature(body),
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => expr_regex_feature(condition)
            .or_else(|| then_body.as_ref().and_then(body_regex_feature))
            .or_else(|| body_regex_feature(else_body)),
        Stmt::Guard {
            condition, body, ..
        } => expr_regex_feature(condition).or_else(|| body_regex_feature(body)),
        Stmt::PublicObservation { fields, .. } => fields
            .iter()
            .find_map(|field| expr_regex_feature(&field.value)),
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => None,
    }
}

fn stmt_assertion_feature(stmt: &Stmt) -> Option<&'static str> {
    match stmt {
        Stmt::SumDefinition { .. } => None,
        Stmt::SumMatch { subject, arms, default_body, .. } => {
            expr_assertion_feature(subject)
                .or_else(|| arms.iter().find_map(|arm| body_assertion_feature(&arm.body)))
                .or_else(|| default_body.as_ref().and_then(body_assertion_feature))
        }
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    if let Some(feature) = expr_assertion_feature(value) {
                        return Some(feature);
                    }
                }
            }
            None
        }
        Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => expr_assertion_feature(value),
        Stmt::Mutate { target, value, .. } => {
            expr_assertion_feature(target).or_else(|| expr_assertion_feature(value))
        }
        Stmt::ResultFlow {
            producer,
            destination,
            ..
        } => expr_assertion_feature(producer).or_else(|| expr_assertion_feature(destination)),
        Stmt::Receive {
            condition, body, ..
        } => condition
            .as_ref()
            .and_then(expr_assertion_feature)
            .or_else(|| body_assertion_feature(body)),
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => sender
            .as_ref()
            .and_then(expr_assertion_feature)
            .or_else(|| expr_assertion_feature(payload))
            .or_else(|| expr_assertion_feature(receiver)),
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            expr_assertion_feature(expr)
        }
        Stmt::Return { value, .. } => expr_assertion_feature(value),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => expr_assertion_feature(condition)
            .or_else(|| body_assertion_feature(then_body))
            .or_else(|| else_body.as_ref().and_then(body_assertion_feature)),
        Stmt::Try { action, body, .. } => {
            expr_assertion_feature(action).or_else(|| body_assertion_feature(body))
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                if let Some(feature) = expr_assertion_feature(&branch.condition) {
                    return Some(feature);
                }
                if let Some(feature) = body_assertion_feature(&branch.body) {
                    return Some(feature);
                }
            }
            body_assertion_feature(else_body)
        }
        Stmt::Repeat { body, .. } => body_assertion_feature(body),
        Stmt::While {
            condition, body, ..
        } => expr_assertion_feature(condition).or_else(|| body_assertion_feature(body)),
        Stmt::ForEach { iterable, body, .. } => {
            expr_assertion_feature(iterable).or_else(|| body_assertion_feature(body))
        }
        Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => body_assertion_feature(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_assertion_feature(condition).or_else(|| body_assertion_feature(body)),
        Stmt::Quantifier { body, .. } => body_assertion_feature(body),
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => expr_assertion_feature(condition)
            .or_else(|| then_body.as_ref().and_then(body_assertion_feature))
            .or_else(|| body_assertion_feature(else_body)),
        Stmt::Guard {
            condition, body, ..
        } => expr_assertion_feature(condition).or_else(|| body_assertion_feature(body)),
        Stmt::PublicObservation { fields, .. } => fields
            .iter()
            .find_map(|field| expr_assertion_feature(&field.value)),
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => None,
    }
}

fn stmt_state_machine_feature(stmt: &Stmt) -> Option<&'static str> {
    match stmt {
        Stmt::SumDefinition { .. } => None,
        Stmt::SumMatch { subject, arms, default_body, .. } => {
            expr_state_machine_feature(subject)
                .or_else(|| arms.iter().find_map(|arm| body_state_machine_feature(&arm.body)))
                .or_else(|| default_body.as_ref().and_then(body_state_machine_feature))
        }
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    if let Some(feature) = expr_state_machine_feature(value) {
                        return Some(feature);
                    }
                }
            }
            None
        }
        Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => expr_state_machine_feature(value),
        Stmt::Mutate { target, value, .. } => {
            expr_state_machine_feature(target).or_else(|| expr_state_machine_feature(value))
        }
        Stmt::ResultFlow {
            producer,
            destination,
            ..
        } => expr_state_machine_feature(producer)
            .or_else(|| expr_state_machine_feature(destination)),
        Stmt::Receive {
            condition, body, ..
        } => condition
            .as_ref()
            .and_then(expr_state_machine_feature)
            .or_else(|| body_state_machine_feature(body)),
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => sender
            .as_ref()
            .and_then(expr_state_machine_feature)
            .or_else(|| expr_state_machine_feature(payload))
            .or_else(|| expr_state_machine_feature(receiver)),
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            expr_state_machine_feature(expr)
        }
        Stmt::Return { value, .. } => expr_state_machine_feature(value),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => expr_state_machine_feature(condition)
            .or_else(|| body_state_machine_feature(then_body))
            .or_else(|| else_body.as_ref().and_then(body_state_machine_feature)),
        Stmt::Try { action, body, .. } => {
            expr_state_machine_feature(action).or_else(|| body_state_machine_feature(body))
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                if let Some(feature) = expr_state_machine_feature(&branch.condition) {
                    return Some(feature);
                }
                if let Some(feature) = body_state_machine_feature(&branch.body) {
                    return Some(feature);
                }
            }
            body_state_machine_feature(else_body)
        }
        Stmt::Repeat { body, .. } => body_state_machine_feature(body),
        Stmt::While {
            condition, body, ..
        } => expr_state_machine_feature(condition).or_else(|| body_state_machine_feature(body)),
        Stmt::ForEach { iterable, body, .. } => {
            expr_state_machine_feature(iterable).or_else(|| body_state_machine_feature(body))
        }
        Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => body_state_machine_feature(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_state_machine_feature(condition).or_else(|| body_state_machine_feature(body)),
        Stmt::Quantifier { body, .. } => body_state_machine_feature(body),
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => expr_state_machine_feature(condition)
            .or_else(|| then_body.as_ref().and_then(body_state_machine_feature))
            .or_else(|| body_state_machine_feature(else_body)),
        Stmt::Guard {
            condition, body, ..
        } => expr_state_machine_feature(condition).or_else(|| body_state_machine_feature(body)),
        Stmt::PublicObservation { fields, .. } => fields
            .iter()
            .find_map(|field| expr_state_machine_feature(&field.value)),
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => None,
    }
}

fn stmt_quantifier_feature(stmt: &Stmt) -> Option<&'static str> {
    match stmt {
        Stmt::SumDefinition { .. } => None,
        Stmt::SumMatch { arms, default_body, .. } => arms
            .iter()
            .find_map(|arm| body_quantifier_feature(&arm.body))
            .or_else(|| default_body.as_ref().and_then(body_quantifier_feature)),
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    let _ = value;
                }
            }
            None
        }
        Stmt::Definition { .. }
        | Stmt::DerivedBinding { .. }
        | Stmt::Mutate { .. }
        | Stmt::ResultFlow { .. }
        | Stmt::Send { .. }
        | Stmt::Expr { .. }
        | Stmt::Show { .. }
        | Stmt::Inspect { .. }
        | Stmt::PublicObservation { .. }
        | Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Return { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => None,
        Stmt::Receive { body, .. } => body_quantifier_feature(body),
        Stmt::If {
            then_body,
            else_body,
            ..
        } => body_quantifier_feature(then_body)
            .or_else(|| else_body.as_ref().and_then(body_quantifier_feature)),
        Stmt::Try { body, .. } => body_quantifier_feature(body),
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                if let Some(feature) = body_quantifier_feature(&branch.body) {
                    return Some(feature);
                }
            }
            body_quantifier_feature(else_body)
        }
        Stmt::Repeat { body, .. } => body_quantifier_feature(body),
        Stmt::While { body, .. } => body_quantifier_feature(body),
        Stmt::ForEach { body, .. } => body_quantifier_feature(body),
        Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => body_quantifier_feature(body),
        Stmt::HookWhenBecomes { body, .. } | Stmt::HookWhile { body, .. } => {
            body_quantifier_feature(body)
        }
        Stmt::Quantifier { .. } => Some("logic_quantifier"),
        Stmt::Contract {
            then_body,
            else_body,
            ..
        } => then_body
            .as_ref()
            .and_then(body_quantifier_feature)
            .or_else(|| body_quantifier_feature(else_body)),
        Stmt::Guard { body, .. } => body_quantifier_feature(body),
    }
}

fn expr_regex_feature(expr: &Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::SumBranchValue { payload, .. } => payload.iter().find_map(|arg|expr_regex_feature(&arg.value)),
        ExprKind::ProductConstruction { fields, .. } => fields.iter().find_map(|field| expr_regex_feature(&field.expr)),
        ExprKind::Tensor(value) => expr_regex_feature(value.input()),
        ExprKind::MultiAxisSelection(value) => value.children().find_map(expr_regex_feature),
        ExprKind::Literal(Literal::Regex(_)) => Some("regex_literal"),
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Assertion(_)
        | ExprKind::Formula(_)
        | ExprKind::Template(_)
        | ExprKind::StateMachine(_) => None,
        ExprKind::FieldAccess { target, .. } => expr_regex_feature(target),
        ExprKind::SeedLiteral { body, .. } => expr_regex_feature(body),
        ExprKind::Call { args, func } => {
            if matches!(
                func.as_str(),
                "정규맞추기"
                    | "정규찾기"
                    | "정규캡처하기"
                    | "정규이름캡처하기"
                    | "정규바꾸기"
                    | "정규나누기"
            ) {
                return Some("regex_api");
            }
            for arg in args {
                if let Some(feature) = expr_regex_feature(&arg.expr) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                if let Some(feature) = expr_regex_feature(element) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::SelectionPlace { target, index } => expr_regex_feature(target).or_else(|| expr_regex_feature(index)),
        ExprKind::Infix { left, right, .. } => {
            expr_regex_feature(left).or_else(|| expr_regex_feature(right))
        }
        ExprKind::Suffix { value, .. } => expr_regex_feature(value),
        ExprKind::Thunk(body) => body_regex_feature(body),
        ExprKind::Eval { thunk, .. } => expr_regex_feature(thunk),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                if let Some(feature) = expr_regex_feature(stage) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                if let Some(feature) = expr_regex_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                if let Some(feature) = expr_regex_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Nuance { expr, .. } => expr_regex_feature(expr),
    }
}

fn expr_assertion_feature(expr: &Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::SumBranchValue { payload, .. } => payload.iter().find_map(|arg|expr_assertion_feature(&arg.value)),
        ExprKind::ProductConstruction { fields, .. } => fields.iter().find_map(|field| expr_assertion_feature(&field.expr)),
        ExprKind::Tensor(value) => expr_assertion_feature(value.input()),
        ExprKind::MultiAxisSelection(value) => value.children().find_map(expr_assertion_feature),
        ExprKind::Assertion(_) => Some("assertion_literal"),
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Formula(_)
        | ExprKind::Template(_)
        | ExprKind::StateMachine(_) => None,
        ExprKind::FieldAccess { target, .. } => expr_assertion_feature(target),
        ExprKind::SeedLiteral { body, .. } => expr_assertion_feature(body),
        ExprKind::Call { args, func } => {
            if func == "살피기" {
                return Some("assertion_api");
            }
            for arg in args {
                if let Some(feature) = expr_assertion_feature(&arg.expr) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                if let Some(feature) = expr_assertion_feature(element) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::SelectionPlace { target, index } => expr_assertion_feature(target).or_else(|| expr_assertion_feature(index)),
        ExprKind::Infix { left, right, .. } => {
            expr_assertion_feature(left).or_else(|| expr_assertion_feature(right))
        }
        ExprKind::Suffix { value, .. } => expr_assertion_feature(value),
        ExprKind::Thunk(body) => body_assertion_feature(body),
        ExprKind::Eval { thunk, .. } => expr_assertion_feature(thunk),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                if let Some(feature) = expr_assertion_feature(stage) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                if let Some(feature) = expr_assertion_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                if let Some(feature) = expr_assertion_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Nuance { expr, .. } => expr_assertion_feature(expr),
    }
}

fn expr_state_machine_feature(expr: &Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::SumBranchValue { payload, .. } => payload.iter().find_map(|arg|expr_state_machine_feature(&arg.value)),
        ExprKind::ProductConstruction { fields, .. } => fields.iter().find_map(|field| expr_state_machine_feature(&field.expr)),
        ExprKind::Tensor(value) => expr_state_machine_feature(value.input()),
        ExprKind::MultiAxisSelection(value) => value.children().find_map(expr_state_machine_feature),
        ExprKind::StateMachine(_) => Some("state_machine_literal"),
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Assertion(_)
        | ExprKind::Formula(_)
        | ExprKind::Template(_) => None,
        ExprKind::FieldAccess { target, .. } => expr_state_machine_feature(target),
        ExprKind::SeedLiteral { body, .. } => expr_state_machine_feature(body),
        ExprKind::Call { args, func } => {
            if matches!(func.as_str(), "처음으로" | "지금상태" | "다음으로") {
                return Some("state_machine_api");
            }
            for arg in args {
                if let Some(feature) = expr_state_machine_feature(&arg.expr) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                if let Some(feature) = expr_state_machine_feature(element) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::SelectionPlace { target, index } => expr_state_machine_feature(target).or_else(|| expr_state_machine_feature(index)),
        ExprKind::Infix { left, right, .. } => {
            expr_state_machine_feature(left).or_else(|| expr_state_machine_feature(right))
        }
        ExprKind::Suffix { value, .. } => expr_state_machine_feature(value),
        ExprKind::Thunk(body) => body_state_machine_feature(body),
        ExprKind::Eval { thunk, .. } => expr_state_machine_feature(thunk),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                if let Some(feature) = expr_state_machine_feature(stage) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                if let Some(feature) = expr_state_machine_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                if let Some(feature) = expr_state_machine_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Nuance { expr, .. } => expr_state_machine_feature(expr),
    }
}

