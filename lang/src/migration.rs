use crate::{
    Body, CanonProgram, Expr, ExprKind, MessageTransfer, ParseError, ParseMode, Span, Stmt,
    TopLevelItem,
};

pub const HISTORICAL_ASYNC_MIGRATION_DIAGNOSTIC_CODE: &str =
    "E_HISTORICAL_ASYNC_MIGRATION_REQUIRED";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalAsyncMigrationEvidence {
    pub original_surface: String,
    pub historical_identity: &'static str,
    pub successor_replacement: String,
    pub diagnostic_code: &'static str,
    pub source_span: Span,
}

pub fn historical_async_migration_evidence(
    source: &str,
    file_path: &str,
) -> Result<Vec<HistoricalAsyncMigrationEvidence>, ParseError> {
    let program =
        crate::parse_with_mode(source, file_path, ParseMode::HistoricalMigrationEvidence)?;
    Ok(collect_program(&program, source))
}

fn collect_program(program: &CanonProgram, source: &str) -> Vec<HistoricalAsyncMigrationEvidence> {
    let mut out = Vec::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            collect_body(body, source, &mut out);
        }
    }
    out
}

fn collect_body(body: &Body, source: &str, out: &mut Vec<HistoricalAsyncMigrationEvidence>) {
    for stmt in &body.stmts {
        collect_stmt(stmt, source, out);
    }
}

fn collect_optional_body(
    body: Option<&Body>,
    source: &str,
    out: &mut Vec<HistoricalAsyncMigrationEvidence>,
) {
    if let Some(body) = body {
        collect_body(body, source, out);
    }
}

fn collect_stmt(stmt: &Stmt, source: &str, out: &mut Vec<HistoricalAsyncMigrationEvidence>) {
    match stmt {
        Stmt::Send {
            transfer,
            span,
            sender,
            payload,
            receiver,
            ..
        } => {
            if matches!(transfer, MessageTransfer::HistoricalAsync(_)) {
                let original_surface = source
                    .get(span.start..span.end)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                out.push(HistoricalAsyncMigrationEvidence {
                    successor_replacement: original_surface.replacen("~~>", "~>", 1),
                    original_surface,
                    historical_identity: "MessageTransfer::HistoricalAsync",
                    diagnostic_code: HISTORICAL_ASYNC_MIGRATION_DIAGNOSTIC_CODE,
                    source_span: *span,
                });
            }
            if let Some(sender) = sender {
                collect_expr(sender, source, out);
            }
            collect_expr(payload, source, out);
            collect_expr(receiver, source, out);
        }
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    collect_expr(value, source, out);
                }
            }
        }
        Stmt::Definition { value, .. } => collect_expr(value, source, out),
        Stmt::Mutate { target, value, .. } => {
            collect_expr(target, source, out);
            collect_expr(value, source, out);
        }
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            collect_expr(expr, source, out);
        }
        Stmt::Receive {
            condition, body, ..
        } => {
            if let Some(condition) = condition {
                collect_expr(condition, source, out);
            }
            collect_body(body, source, out);
        }
        Stmt::Return { value, .. } => collect_expr(value, source, out),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_expr(condition, source, out);
            collect_body(then_body, source, out);
            collect_optional_body(else_body.as_ref(), source, out);
        }
        Stmt::Try { action, body, .. } => {
            collect_expr(action, source, out);
            collect_body(body, source, out);
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                collect_expr(&branch.condition, source, out);
                collect_body(&branch.body, source, out);
            }
            collect_body(else_body, source, out);
        }
        Stmt::Repeat { body, .. }
        | Stmt::BeatBlock { body, .. }
        | Stmt::Quantifier { body, .. } => {
            collect_body(body, source, out);
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
            collect_expr(condition, source, out);
            collect_body(body, source, out);
        }
        Stmt::ForEach { iterable, body, .. } => {
            collect_expr(iterable, source, out);
            collect_body(body, source, out);
        }
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_expr(condition, source, out);
            collect_optional_body(then_body.as_ref(), source, out);
            collect_body(else_body, source, out);
        }
        Stmt::Hook { body, .. } => collect_body(body, source, out),
        Stmt::PublicObservation { fields, .. } => {
            for field in fields {
                collect_expr(&field.value, source, out);
            }
        }
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => {}
    }
}

fn collect_expr(expr: &Expr, source: &str, out: &mut Vec<HistoricalAsyncMigrationEvidence>) {
    match &expr.kind {
        ExprKind::FieldAccess { target, .. } => collect_expr(target, source, out),
        ExprKind::SeedLiteral { body, .. } => collect_expr(body, source, out),
        ExprKind::Call { args, .. } => {
            for arg in args {
                collect_expr(&arg.expr, source, out);
            }
        }
        ExprKind::Infix { left, right, .. } => {
            collect_expr(left, source, out);
            collect_expr(right, source, out);
        }
        ExprKind::Suffix { value, .. } => collect_expr(value, source, out),
        ExprKind::Thunk(body) => collect_body(body, source, out),
        ExprKind::Eval { thunk, .. } => collect_expr(thunk, source, out),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                collect_expr(stage, source, out);
            }
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                collect_expr(value, source, out);
            }
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                collect_expr(value, source, out);
            }
        }
        ExprKind::Nuance { expr, .. } => collect_expr(expr, source, out),
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
    use super::*;
    use crate::{normalize, NormalizationLevel, TopLevelItem};

    const SOURCE: &str = "검사:움직씨 = { 값 ~~> 대상. }";

    #[test]
    fn successor_default_owns_double_arrow_as_rendezvous() {
        let program = crate::parse(SOURCE, "successor.ddn").expect("successor parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let Stmt::Send { transfer, .. } = &seed.body.as_ref().unwrap().stmts[0] else {
            panic!("send expected")
        };
        assert!(matches!(transfer, MessageTransfer::Rendezvous(_)));
        assert!(normalize(&program, NormalizationLevel::N1).contains("값 ~~> 대상."));
    }

    #[test]
    fn explicit_migration_mode_emits_distinct_evidence() {
        let report = historical_async_migration_evidence(SOURCE, "historical.ddn")
            .expect("migration evidence");
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].original_surface, "값 ~~> 대상.");
        assert_eq!(report[0].successor_replacement, "값 ~> 대상.");
        assert_eq!(
            report[0].diagnostic_code,
            HISTORICAL_ASYNC_MIGRATION_DIAGNOSTIC_CODE
        );
        assert!(report[0].source_span.end > report[0].source_span.start);

        let historical = crate::parse_with_mode(
            SOURCE,
            "historical.ddn",
            ParseMode::HistoricalMigrationEvidence,
        )
        .expect("historical parse");
        let tagged = normalize(&historical, NormalizationLevel::N1);
        assert!(tagged.contains("~[historical-async-migration-required]>"));
        assert_ne!(
            tagged,
            normalize(
                &crate::parse(SOURCE, "successor.ddn").unwrap(),
                NormalizationLevel::N1
            )
        );
    }

    #[test]
    fn successor_identity_never_depends_on_path_or_handler_shape() {
        let source = r#"
대상:임자 = { 알림을 받으면 { 없음. }. }
검사:움직씨 = { 값 ~~> 대상. }
"#;
        for path in [
            "historical/legacy_async/input.ddn",
            "pack/lang_consistency_v1/old_signal.ddn",
        ] {
            let program = crate::parse(source, path).expect("successor parse");
            let TopLevelItem::SeedDef(seed) = &program.items[1];
            let Stmt::Send { transfer, .. } = &seed.body.as_ref().unwrap().stmts[0] else {
                panic!("send expected")
            };
            assert!(matches!(transfer, MessageTransfer::Rendezvous(_)));
        }
    }

    #[test]
    fn successor_direction_pairs_keep_distinct_typed_identities() {
        let source = r#"
검사:움직씨 = {
    받는이 <~ 값.
    값 ~> 받는이.
    받는이 <~~ 값.
    값 ~~> 받는이.
}
"#;
        let program = crate::parse(source, "directions.ddn").expect("direction parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let transfers = seed
            .body
            .as_ref()
            .unwrap()
            .stmts
            .iter()
            .map(|stmt| match stmt {
                Stmt::Send { transfer, .. } => *transfer,
                _ => panic!("send expected"),
            })
            .collect::<Vec<_>>();
        assert!(matches!(transfers[0], MessageTransfer::Async(_)));
        assert!(matches!(transfers[1], MessageTransfer::Async(_)));
        assert!(matches!(transfers[2], MessageTransfer::Rendezvous(_)));
        assert!(matches!(transfers[3], MessageTransfer::Rendezvous(_)));
    }
}
