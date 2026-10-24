use super::document_store::DocumentSnapshot;
use super::position::{
    byte_offset_to_position, position_to_byte_offset, LspPosition, PositionEncoding, PositionError,
};
use super::symbol::{local_signature, type_surface, SOURCE_PROFILE_V25};
use ddonirang_lang::stdlib::{canonicalize_stdlib_alias, minimal_stdlib_sigs};
use ddonirang_lang::{parse_v25, ArgBinding, Body, Expr, ExprKind, ParamPin, Stmt, TopLevelItem};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureParameter {
    pub label: String,
    pub param_pin_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureHelpResolution {
    pub document_version: i64,
    pub snapshot_identity: String,
    pub identity: String,
    pub label: String,
    pub parameters: Vec<SignatureParameter>,
    pub active_parameter: u32,
    pub source_profile: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlayHintResolution {
    pub position: LspPosition,
    pub label: String,
    pub definition_identity: String,
    pub param_pin_id: u64,
    pub document_version: i64,
    pub snapshot_identity: String,
    pub source_profile: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureHelpError {
    InvalidUtf8,
    InvalidPosition(PositionError),
}

impl fmt::Display for SignatureHelpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 => f.write_str("signature help source가 UTF-8이 아닙니다"),
            Self::InvalidPosition(error) => {
                write!(f, "signature help position이 잘못되었습니다: {error}")
            }
        }
    }
}

impl std::error::Error for SignatureHelpError {}

struct CallCandidate<'a> {
    expr: &'a Expr,
    func: &'a str,
    args: &'a [ArgBinding],
}

pub fn inlay_hints_snapshot(
    snapshot: &DocumentSnapshot,
    encoding: PositionEncoding,
) -> Result<Vec<InlayHintResolution>, SignatureHelpError> {
    let source = snapshot
        .text()
        .map_err(|_| SignatureHelpError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(Vec::new());
    }
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(Vec::new()),
    };
    let mut calls = Vec::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            collect_body_calls(body, &mut calls);
        }
    }

    let mut hints = Vec::new();
    for candidate in calls {
        let Some(seed) = program.items.iter().find_map(|item| {
            let TopLevelItem::SeedDef(seed) = item;
            (seed.canonical_name == candidate.func).then_some(seed)
        }) else {
            continue;
        };
        let identity = format!("definition:local:{}", seed.canonical_name);
        for arg in candidate.args {
            if arg.span.start < candidate.expr.span.start
                || arg.span.end > candidate.expr.span.end
                || !matches!(
                    arg.binding_reason,
                    ddonirang_lang::BindingReason::Positional
                )
            {
                continue;
            }
            let Some(pin_name) = arg.resolved_pin.as_deref() else {
                continue;
            };
            let Some(param) = seed.params.iter().find(|param| param.pin_name == pin_name) else {
                continue;
            };
            let position = byte_offset_to_position(source, arg.span.start, encoding)
                .map_err(SignatureHelpError::InvalidPosition)?;
            hints.push(InlayHintResolution {
                position,
                label: format!("{}:", param.pin_name),
                definition_identity: identity.clone(),
                param_pin_id: param.id,
                document_version: snapshot.version,
                snapshot_identity: snapshot.identity.as_str().to_string(),
                source_profile: SOURCE_PROFILE_V25,
            });
        }
    }
    hints.sort_by_key(|hint| {
        (
            hint.position.line,
            hint.position.character,
            hint.param_pin_id,
            hint.label.clone(),
        )
    });
    Ok(hints)
}

pub fn signature_help_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<Option<SignatureHelpResolution>, SignatureHelpError> {
    let source = snapshot
        .text()
        .map_err(|_| SignatureHelpError::InvalidUtf8)?;
    let cursor = position_to_byte_offset(source, position, encoding)
        .map_err(SignatureHelpError::InvalidPosition)?;
    if snapshot.language_id != "ddn" {
        return Ok(None);
    }
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(None),
    };

    let mut candidates = Vec::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            collect_body_calls(body, &mut candidates);
        }
    }
    let Some(candidate) = candidates
        .into_iter()
        .filter(|candidate| {
            candidate
                .args
                .iter()
                .any(|arg| arg.span.start <= cursor && cursor < arg.span.end)
        })
        .min_by_key(|candidate| {
            candidate
                .expr
                .span
                .end
                .saturating_sub(candidate.expr.span.start)
        })
    else {
        return Ok(None);
    };

    let local_seed = program.items.iter().find_map(|item| {
        let TopLevelItem::SeedDef(seed) = item;
        (seed.canonical_name == candidate.func).then_some(seed)
    });
    let (identity, label, parameters, params): (
        String,
        String,
        Vec<SignatureParameter>,
        Vec<ParamPin>,
    ) = if let Some(seed) = local_seed {
        let params = seed.params.clone();
        let parameters = params
            .iter()
            .map(|param| SignatureParameter {
                label: format!("{}:{}", param.pin_name, type_surface(&param.type_ref)),
                param_pin_id: Some(param.id),
            })
            .collect();
        (
            format!("definition:local:{}", seed.canonical_name),
            local_signature(seed),
            parameters,
            params,
        )
    } else {
        let Some(signature) = minimal_stdlib_sigs().into_iter().find(|signature| {
            signature.name == candidate.func
                && canonicalize_stdlib_alias(signature.name) == signature.name
        }) else {
            return Ok(None);
        };
        let parameters = signature
            .params
            .iter()
            .map(|label| SignatureParameter {
                label: (*label).to_string(),
                param_pin_id: None,
            })
            .collect();
        (
            format!("definition:stdlib:{}", signature.name),
            format!(
                "{}({}) -> {}",
                signature.name,
                signature.params.join(", "),
                signature.ret
            ),
            parameters,
            Vec::new(),
        )
    };

    let arg = candidate
        .args
        .iter()
        .find(|arg| arg.span.start <= cursor && cursor < arg.span.end)
        .expect("candidate was selected by an argument span");
    let active_parameter = if params.is_empty() {
        candidate
            .args
            .iter()
            .position(|candidate_arg| candidate_arg.id == arg.id)
    } else if let Some(pin) = &arg.resolved_pin {
        params.iter().position(|param| param.pin_name == *pin)
    } else {
        candidate
            .args
            .iter()
            .position(|candidate_arg| candidate_arg.id == arg.id)
    };
    let Some(active_parameter) = active_parameter else {
        return Ok(None);
    };
    if active_parameter >= parameters.len() {
        return Ok(None);
    }

    Ok(Some(SignatureHelpResolution {
        document_version: snapshot.version,
        snapshot_identity: snapshot.identity.as_str().to_string(),
        identity,
        label,
        parameters,
        active_parameter: active_parameter as u32,
        source_profile: SOURCE_PROFILE_V25,
    }))
}

fn collect_body_calls<'a>(body: &'a Body, calls: &mut Vec<CallCandidate<'a>>) {
    for stmt in &body.stmts {
        collect_stmt_calls(stmt, calls);
    }
}

fn collect_stmt_calls<'a>(stmt: &'a Stmt, calls: &mut Vec<CallCandidate<'a>>) {
    match stmt {
        Stmt::SumDefinition { .. } => {}
        Stmt::SumMatch { subject, arms, default_body, .. } => {
            collect_expr_calls(subject, calls);
            for arm in arms {
                collect_body_calls(&arm.body, calls);
            }
            if let Some(body) = default_body {
                collect_body_calls(body, calls);
            }
        }
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    collect_expr_calls(value, calls);
                }
            }
        }
        Stmt::Definition { value, .. }
        | Stmt::DerivedBinding {value,..}
        | Stmt::Show { expr: value, .. }
        | Stmt::Inspect { expr: value, .. }
        | Stmt::Return { value, .. } => collect_expr_calls(value, calls),
        Stmt::Mutate { target, value, .. } => {
            collect_expr_calls(target, calls);
            collect_expr_calls(value, calls);
        }
        Stmt::ResultFlow {
            producer,
            destination,
            ..
        } => {
            collect_expr_calls(producer, calls);
            collect_expr_calls(destination, calls);
        }
        Stmt::Expr { expr, .. } => collect_expr_calls(expr, calls),
        Stmt::Receive {
            sender,
            condition,
            body,
            ..
        } => {
            if let Some(sender) = sender {
                collect_expr_calls(sender, calls);
            }
            if let Some(condition) = condition {
                collect_expr_calls(condition, calls);
            }
            collect_body_calls(body, calls);
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            if let Some(sender) = sender {
                collect_expr_calls(sender, calls);
            }
            collect_expr_calls(payload, calls);
            collect_expr_calls(receiver, calls);
        }
        Stmt::PublicObservation { fields, .. } => {
            for field in fields {
                collect_expr_calls(&field.value, calls);
            }
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_expr_calls(condition, calls);
            collect_body_calls(then_body, calls);
            if let Some(else_body) = else_body {
                collect_body_calls(else_body, calls);
            }
        }
        Stmt::Try { action, body, .. } => {
            collect_expr_calls(action, calls);
            collect_body_calls(body, calls);
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                collect_expr_calls(&branch.condition, calls);
                collect_body_calls(&branch.body, calls);
            }
            collect_body_calls(else_body, calls);
        }
        Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => {
            collect_body_calls(body, calls)
        }
        Stmt::While {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => {
            collect_expr_calls(condition, calls);
            collect_body_calls(body, calls);
        }
        Stmt::ForEach { iterable, body, .. } => {
            collect_expr_calls(iterable, calls);
            collect_body_calls(body, calls);
        }
        Stmt::Quantifier { body, .. } => {
            collect_body_calls(body, calls);
        }
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_expr_calls(condition, calls);
            if let Some(then_body) = then_body {
                collect_body_calls(then_body, calls);
            }
            collect_body_calls(else_body, calls);
        }
        Stmt::Guard {
            condition, body, ..
        }
        | Stmt::HookWhenBecomes {
            condition, body, ..
        } => {
            collect_expr_calls(condition, calls);
            collect_body_calls(body, calls);
        }
        Stmt::Break { .. }
        | Stmt::ContinueLoop { .. }
        | Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. } => {}
    }
}

fn collect_expr_calls<'a>(expr: &'a Expr, calls: &mut Vec<CallCandidate<'a>>) {
    match &expr.kind {
        ExprKind::SumBranchValue { payload, .. } => {
            for payload in payload {
                collect_expr_calls(&payload.value, calls);
            }
        }
        ExprKind::ProductConstruction { fields, .. } => {
            for field in fields {
                collect_expr_calls(&field.expr, calls);
            }
        }
        ExprKind::Tensor(value) => collect_expr_calls(value.input(), calls),
        ExprKind::MultiAxisSelection(value) => { for child in value.children() { collect_expr_calls(child, calls); } }
        ExprKind::Call { args, func } => {
            calls.push(CallCandidate { expr, func, args });
            for arg in args {
                collect_expr_calls(&arg.expr, calls);
            }
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                collect_expr_calls(element, calls);
            }
        }
        ExprKind::SelectionPlace { target, index } => {
            collect_expr_calls(target, calls);
            collect_expr_calls(index, calls);
        }
        ExprKind::FieldAccess { target, .. }
        | ExprKind::SeedLiteral { body: target, .. }
        | ExprKind::Eval { thunk: target, .. }
        | ExprKind::Nuance { expr: target, .. } => collect_expr_calls(target, calls),
        ExprKind::Infix { left, right, .. } => {
            collect_expr_calls(left, calls);
            collect_expr_calls(right, calls);
        }
        ExprKind::Suffix { value, .. } => collect_expr_calls(value, calls),
        ExprKind::Thunk(body) => collect_body_calls(body, calls),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                collect_expr_calls(stage, calls);
            }
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                collect_expr_calls(value, calls);
            }
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                collect_expr_calls(value, calls);
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

#[cfg(test)]
mod tests {
    use super::signature_help_snapshot;
    use crate::lsp::{DocumentStore, LspPosition, PositionEncoding};

    #[test]
    fn collection_constructor_is_not_published_as_a_callable_signature() {
        let source = "검산:움직씨 := {\n    목록:(수)차림 := 차림[1, 2].\n}\n";
        let uri = "file:///collection_constructor.ddn";
        let mut store = DocumentStore::new();
        store
            .open(uri, "ddn", 1, source.as_bytes().to_vec())
            .expect("open snapshot");
        let snapshot = store.get(uri).expect("snapshot");

        let resolution = signature_help_snapshot(
            snapshot,
            LspPosition {
                line: 1,
                character: 19,
            },
            PositionEncoding::Utf8,
        )
        .expect("valid position");

        assert!(resolution.is_none());
    }
}
