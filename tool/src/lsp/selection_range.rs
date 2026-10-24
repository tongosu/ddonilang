use super::document_store::DocumentSnapshot;
use super::position::{
    byte_range_to_lsp_range, position_to_byte_offset, ByteRange, LspPosition, LspRange,
    PositionEncoding, PositionError,
};
use ddonirang_lang::{
    parse_v25, Body, Expr, ExprKind, Lexer, Stmt, Token, TokenKind, TopLevelItem,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionRange {
    pub range: LspRange,
    pub parent: Option<Box<SelectionRange>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionRangeError {
    InvalidUtf8,
    InvalidRange(String),
}

impl fmt::Display for SelectionRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 => f.write_str("selection range source가 UTF-8이 아닙니다"),
            Self::InvalidRange(error) => write!(f, "selection range가 잘못되었습니다: {error}"),
        }
    }
}

impl std::error::Error for SelectionRangeError {}

pub fn selection_ranges_snapshot(
    snapshot: &DocumentSnapshot,
    positions: &[LspPosition],
    encoding: PositionEncoding,
) -> Result<Vec<Option<SelectionRange>>, SelectionRangeError> {
    let source = snapshot
        .text()
        .map_err(|_| SelectionRangeError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(vec![None; positions.len()]);
    }
    let Ok(tokens) = Lexer::new(source).tokenize() else {
        return Ok(vec![None; positions.len()]);
    };
    let Ok(program) = parse_v25(source, &snapshot.uri) else {
        return Ok(vec![None; positions.len()]);
    };
    let mut candidates = Vec::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        push_range(
            &mut candidates,
            ByteRange::new(seed.span.start, seed.span.end),
        );
        for parameter in &seed.params {
            push_range(
                &mut candidates,
                ByteRange::new(parameter.span.start, parameter.span.end),
            );
        }
        if let Some(body) = &seed.body {
            collect_body(body, &mut candidates);
        }
    }
    for token in &tokens {
        if token.kind != TokenKind::Eof {
            push_range(
                &mut candidates,
                ByteRange::new(token.span.start, token.span.end),
            );
        }
    }
    validate_candidates(&candidates, source)?;
    Ok(positions
        .iter()
        .map(|position| selection_for_position(source, &tokens, &candidates, *position, encoding))
        .collect())
}

fn selection_for_position(
    source: &str,
    tokens: &[Token],
    candidates: &[ByteRange],
    position: LspPosition,
    encoding: PositionEncoding,
) -> Option<SelectionRange> {
    let cursor = position_to_byte_offset(source, position, encoding).ok()?;
    let token = tokens
        .iter()
        .find(|token| token.span.start <= cursor && cursor < token.span.end)?;
    if !selectable_token(&token.kind) {
        return None;
    }
    let token_range = ByteRange::new(token.span.start, token.span.end).ok()?;
    let mut selected = candidates
        .iter()
        .copied()
        .filter(|range| range.start <= cursor && cursor < range.end)
        .filter(|range| {
            token_range.start <= range.start
                || range.end <= token_range.end
                || !is_string_token(&token.kind)
        })
        .collect::<Vec<_>>();
    selected.push(token_range);
    selected.sort_by_key(|range| (range.end - range.start, range.start, range.end));
    selected.dedup();
    if !validate_selection_chain(&selected) {
        return None;
    }
    let mut chain = None;
    for range in selected.iter().rev() {
        let lsp_range = byte_range_to_lsp_range(source, *range, encoding).ok()?;
        chain = Some(Box::new(SelectionRange {
            range: lsp_range,
            parent: chain,
        }));
    }
    chain.map(|value| *value)
}

fn selectable_token(kind: &TokenKind) -> bool {
    !matches!(
        kind,
        TokenKind::Eof
            | TokenKind::Colon
            | TokenKind::Define
            | TokenKind::Equals
            | TokenKind::Arrow
            | TokenKind::RightArrow
            | TokenKind::TypeArrow
            | TokenKind::AsyncLeftArrow
            | TokenKind::AsyncRightArrow
            | TokenKind::RendezvousLeftArrow
            | TokenKind::PersistentDependencyArrow
            | TokenKind::DoubleArrow
            | TokenKind::Dot
            | TokenKind::DotDot
            | TokenKind::DotDotEq
            | TokenKind::Comma
            | TokenKind::Semicolon
            | TokenKind::Pipe
            | TokenKind::LParen
            | TokenKind::RParen
            | TokenKind::LBrace
            | TokenKind::RBrace
            | TokenKind::LBracket
            | TokenKind::RBracket
            | TokenKind::Plus
            | TokenKind::PlusArrow
            | TokenKind::PlusEqual
            | TokenKind::Minus
            | TokenKind::MinusArrow
            | TokenKind::MinusEqual
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::Caret
            | TokenKind::RelationEq
            | TokenKind::EqEq
            | TokenKind::NotEq
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::LtEq
            | TokenKind::GtEq
            | TokenKind::And
            | TokenKind::Or
            | TokenKind::Not
            | TokenKind::Question
            | TokenKind::Bang
            | TokenKind::Tilde
            | TokenKind::At
    )
}

fn is_string_token(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::StringLit(_)
            | TokenKind::TemplateBlock(_)
            | TokenKind::FormulaBlock(_)
            | TokenKind::AssertionBlock(_)
            | TokenKind::StateMachineBlock(_)
    )
}

fn collect_body(body: &Body, candidates: &mut Vec<ByteRange>) {
    push_range(candidates, ByteRange::new(body.span.start, body.span.end));
    for statement in &body.stmts {
        collect_stmt(statement, candidates);
    }
}

fn collect_stmt(statement: &Stmt, candidates: &mut Vec<ByteRange>) {
    let span = statement_span(statement);
    push_range(candidates, ByteRange::new(span.start, span.end));
    match statement {
        Stmt::SumDefinition { .. } => {}
        Stmt::SumMatch { subject, arms, default_body, .. } => {
            collect_expr(subject, candidates);
            for arm in arms {
                collect_body(&arm.body, candidates);
            }
            if let Some(body) = default_body {
                collect_body(body, candidates);
            }
        }
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                push_range(candidates, ByteRange::new(item.span.start, item.span.end));
                if let Some(value) = &item.value {
                    collect_expr(value, candidates);
                }
            }
        }
        Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => collect_expr(value, candidates),
        Stmt::Mutate { target, value, .. } => {
            collect_expr(target, candidates);
            collect_expr(value, candidates);
        }
        Stmt::ResultFlow {
            producer,
            destination,
            ..
        } => {
            collect_expr(producer, candidates);
            collect_expr(destination, candidates);
        }
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            collect_expr(expr, candidates)
        }
        Stmt::Receive {
            sender,
            condition,
            body,
            ..
        } => {
            if let Some(sender) = sender {
                collect_expr(sender, candidates);
            }
            if let Some(condition) = condition {
                collect_expr(condition, candidates);
            }
            collect_body(body, candidates);
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            if let Some(sender) = sender {
                collect_expr(sender, candidates);
            }
            collect_expr(payload, candidates);
            collect_expr(receiver, candidates);
        }
        Stmt::PublicObservation { fields, .. } => {
            for field in fields {
                push_range(
                    candidates,
                    ByteRange::new(field.name_span.start, field.name_span.end),
                );
                collect_expr(&field.value, candidates);
            }
        }
        Stmt::Return { value, .. } => collect_expr(value, candidates),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_expr(condition, candidates);
            collect_body(then_body, candidates);
            if let Some(else_body) = else_body {
                collect_body(else_body, candidates);
            }
        }
        Stmt::Try { action, body, .. } => {
            collect_expr(action, candidates);
            collect_body(body, candidates);
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                collect_expr(&branch.condition, candidates);
                collect_body(&branch.body, candidates);
            }
            collect_body(else_body, candidates);
        }
        Stmt::Repeat { body, .. }
        | Stmt::Quantifier { body, .. }
        | Stmt::BeatBlock { body, .. }
        | Stmt::Hook { body, .. } => collect_body(body, candidates),
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
            collect_expr(condition, candidates);
            collect_body(body, candidates);
        }
        Stmt::ForEach { iterable, body, .. } => {
            collect_expr(iterable, candidates);
            collect_body(body, candidates);
        }
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_expr(condition, candidates);
            if let Some(then_body) = then_body {
                collect_body(then_body, candidates);
            }
            collect_body(else_body, candidates);
        }
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => {}
    }
}

fn collect_expr(expression: &Expr, candidates: &mut Vec<ByteRange>) {
    push_range(
        candidates,
        ByteRange::new(expression.span.start, expression.span.end),
    );
    match &expression.kind {
        ExprKind::SumBranchValue { payload, .. } => {
            for payload in payload {
                collect_expr(&payload.value, candidates);
            }
        }
        ExprKind::Tensor(value) => collect_expr(value.input(), candidates),
        ExprKind::MultiAxisSelection(value) => { for child in value.children() { collect_expr(child, candidates); } }
        ExprKind::Call { args, .. } | ExprKind::ProductConstruction { fields: args, .. } => {
            for arg in args {
                push_range(candidates, ByteRange::new(arg.span.start, arg.span.end));
                collect_expr(&arg.expr, candidates);
            }
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                collect_expr(element, candidates);
            }
        }
        ExprKind::SelectionPlace { target, index } => {
            collect_expr(target, candidates);
            collect_expr(index, candidates);
        }
        ExprKind::FieldAccess {
            target, field_span, ..
        } => {
            push_range(candidates, ByteRange::new(field_span.start, field_span.end));
            collect_expr(target, candidates);
        }
        ExprKind::SeedLiteral { body, .. }
        | ExprKind::Eval { thunk: body, .. }
        | ExprKind::Nuance { expr: body, .. }
        | ExprKind::Suffix { value: body, .. } => collect_expr(body, candidates),
        ExprKind::Infix { left, right, .. } => {
            collect_expr(left, candidates);
            collect_expr(right, candidates);
        }
        ExprKind::Thunk(body) => collect_body(body, candidates),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                collect_expr(stage, candidates);
            }
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                collect_expr(value, candidates);
            }
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                collect_expr(value, candidates);
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

fn statement_span(statement: &Stmt) -> ddonirang_lang::Span {
    match statement {
        Stmt::DeclBlock { span, .. }
        | Stmt::SumDefinition {span,..}
        | Stmt::SumMatch {span,..}
        | Stmt::Definition { span, .. }
        | Stmt::DerivedBinding {span,..}
        | Stmt::Mutate { span, .. }
        | Stmt::ResultFlow { span, .. }
        | Stmt::Expr { span, .. }
        | Stmt::Receive { span, .. }
        | Stmt::Send { span, .. }
        | Stmt::Show { span, .. }
        | Stmt::Inspect { span, .. }
        | Stmt::PublicObservation { span, .. }
        | Stmt::MetaBlock { span, .. }
        | Stmt::Pragma { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::If { span, .. }
        | Stmt::Try { span, .. }
        | Stmt::Choose { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::While { span, .. }
        | Stmt::ForEach { span, .. }
        | Stmt::Quantifier { span, .. }
        | Stmt::Break { span, .. }
        | Stmt::ContinueLoop { span, .. }
        | Stmt::Contract { span, .. }
        | Stmt::Guard { span, .. }
        | Stmt::BeatBlock { span, .. }
        | Stmt::Hook { span, .. }
        | Stmt::HookWhenBecomes { span, .. }
        | Stmt::HookWhile { span, .. } => *span,
    }
}

fn push_range(candidates: &mut Vec<ByteRange>, range: Result<ByteRange, PositionError>) {
    if let Ok(range) = range {
        if range.start < range.end {
            candidates.push(range);
        }
    }
}

fn validate_selection_chain(selected: &[ByteRange]) -> bool {
    selected
        .windows(2)
        .all(|pair| pair[1].start <= pair[0].start && pair[0].end <= pair[1].end)
}

fn validate_candidates(candidates: &[ByteRange], source: &str) -> Result<(), SelectionRangeError> {
    for range in candidates {
        if range.start >= range.end
            || range.end > source.len()
            || !source.is_char_boundary(range.start)
            || !source.is_char_boundary(range.end)
        {
            return Err(SelectionRangeError::InvalidRange(format!(
                "{}..{}",
                range.start, range.end
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_selection_chain;
    use crate::lsp::position::ByteRange;

    #[test]
    fn rejects_crossing_selection_ranges() {
        let crossing = vec![
            ByteRange { start: 0, end: 8 },
            ByteRange { start: 4, end: 12 },
        ];
        assert!(!validate_selection_chain(&crossing));
    }
}
