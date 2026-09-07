use crate::{Body, CanonProgram, Expr, ExprKind, HookKind, Stmt, TopLevelItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    OneShot,
    Live,
}

impl ExecutionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneShot => "oneshot",
            Self::Live => "live",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionModeDecision {
    pub mode: ExecutionMode,
    pub live_hook_count: u64,
    pub every_madi_hook_count: u64,
    pub every_n_madi_hook_count: u64,
}

impl ExecutionModeDecision {
    pub fn reason(self) -> &'static str {
        if self.mode == ExecutionMode::Live {
            "parsed_tick_hook"
        } else {
            "no_parsed_tick_hook"
        }
    }
}

pub fn decide_execution_mode(program: &CanonProgram) -> ExecutionModeDecision {
    let mut counts = LiveHookCounts::default();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            scan_body(body, &mut counts);
        }
    }
    ExecutionModeDecision {
        mode: if counts.live > 0 {
            ExecutionMode::Live
        } else {
            ExecutionMode::OneShot
        },
        live_hook_count: counts.live,
        every_madi_hook_count: counts.every_madi,
        every_n_madi_hook_count: counts.every_n_madi,
    }
}

#[derive(Debug, Default)]
struct LiveHookCounts {
    live: u64,
    every_madi: u64,
    every_n_madi: u64,
}

fn scan_body(body: &Body, counts: &mut LiveHookCounts) {
    for stmt in &body.stmts {
        scan_stmt(stmt, counts);
    }
}

fn scan_optional_body(body: Option<&Body>, counts: &mut LiveHookCounts) {
    if let Some(body) = body {
        scan_body(body, counts);
    }
}

fn scan_stmt(stmt: &Stmt, counts: &mut LiveHookCounts) {
    match stmt {
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    scan_expr(value, counts);
                }
            }
        }
        Stmt::Definition { value, .. } => scan_expr(value, counts),
        Stmt::Mutate { target, value, .. } => {
            scan_expr(target, counts);
            scan_expr(value, counts);
        }
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            scan_expr(expr, counts);
        }
        Stmt::Receive {
            condition, body, ..
        } => {
            if let Some(condition) = condition {
                scan_expr(condition, counts);
            }
            scan_body(body, counts);
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            if let Some(sender) = sender {
                scan_expr(sender, counts);
            }
            scan_expr(payload, counts);
            scan_expr(receiver, counts);
        }
        Stmt::Return { value, .. } => scan_expr(value, counts),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            scan_expr(condition, counts);
            scan_body(then_body, counts);
            scan_optional_body(else_body.as_ref(), counts);
        }
        Stmt::Try { action, body, .. } => {
            scan_expr(action, counts);
            scan_body(body, counts);
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                scan_expr(&branch.condition, counts);
                scan_body(&branch.body, counts);
            }
            scan_body(else_body, counts);
        }
        Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. } => scan_body(body, counts),
        Stmt::While {
            condition, body, ..
        }
        | Stmt::Guard {
            condition, body, ..
        } => {
            scan_expr(condition, counts);
            scan_body(body, counts);
        }
        Stmt::ForEach { iterable, body, .. } => {
            scan_expr(iterable, counts);
            scan_body(body, counts);
        }
        Stmt::Quantifier { body, .. } => scan_body(body, counts),
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            scan_expr(condition, counts);
            scan_optional_body(then_body.as_ref(), counts);
            scan_body(else_body, counts);
        }
        Stmt::Hook { kind, body, .. } => {
            match kind {
                HookKind::EveryMadi => {
                    counts.live = counts.live.saturating_add(1);
                    counts.every_madi = counts.every_madi.saturating_add(1);
                }
                HookKind::EveryNMadi(_) => {
                    counts.live = counts.live.saturating_add(1);
                    counts.every_n_madi = counts.every_n_madi.saturating_add(1);
                }
                HookKind::Start | HookKind::End => {}
            }
            scan_body(body, counts);
        }
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => {
            scan_expr(condition, counts);
            scan_body(body, counts);
        }
        Stmt::PublicObservation { fields, .. } => {
            for field in fields {
                scan_expr(&field.value, counts);
            }
        }
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => {}
    }
}

fn scan_expr(expr: &Expr, counts: &mut LiveHookCounts) {
    match &expr.kind {
        ExprKind::FieldAccess { target, .. } => scan_expr(target, counts),
        ExprKind::SeedLiteral { body, .. } => scan_expr(body, counts),
        ExprKind::Call { args, .. } => {
            for arg in args {
                scan_expr(&arg.expr, counts);
            }
        }
        ExprKind::Infix { left, right, .. } => {
            scan_expr(left, counts);
            scan_expr(right, counts);
        }
        ExprKind::Suffix { value, .. } => scan_expr(value, counts),
        ExprKind::Thunk(body) => scan_body(body, counts),
        ExprKind::Eval { thunk, .. } => scan_expr(thunk, counts),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                scan_expr(stage, counts);
            }
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                scan_expr(value, counts);
            }
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                scan_expr(value, counts);
            }
        }
        ExprKind::Nuance { expr, .. } => scan_expr(expr, counts),
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
    use crate::{canonicalize, parse};

    fn decision(source: &str) -> ExecutionModeDecision {
        let mut program =
            parse(source, "execution_mode.ddn").expect("parse execution-mode fixture");
        canonicalize(&mut program).expect("canonicalize execution-mode fixture");
        decide_execution_mode(&program)
    }

    #[test]
    fn no_hook_start_hook_and_condition_hook_are_one_shot() {
        for source in [
            "검사:움직씨 = { 1 보여주기. }",
            "검사:움직씨 = { (시작)할때 { 1 보여주기. }. }",
            "검사:움직씨 = { (1 < 2)이 될때 { 1 보여주기. }. }",
        ] {
            assert_eq!(decision(source).mode, ExecutionMode::OneShot);
        }
    }

    #[test]
    fn every_madi_and_every_n_madi_hooks_are_live() {
        let every = decision("검사:움직씨 = { (매마디)마다 { 1 보여주기. }. }");
        assert_eq!(every.mode, ExecutionMode::Live);
        assert_eq!(every.every_madi_hook_count, 1);

        let every_n = decision("검사:움직씨 = { (3마디)마다 { 1 보여주기. }. }");
        assert_eq!(every_n.mode, ExecutionMode::Live);
        assert_eq!(every_n.every_n_madi_hook_count, 1);
    }

    #[test]
    fn alias_and_whitespace_share_the_live_decision() {
        let alias = decision("검사:움직씨 = { ( 매틱 ) 마다 { 1 보여주기. }. }");
        let canonical = decision("검사:움직씨 = { (매마디)마다 { 1 보여주기. }. }");
        assert_eq!(alias, canonical);
        assert_eq!(alias.mode, ExecutionMode::Live);
    }

    #[test]
    fn tick_hook_text_in_string_or_comment_is_not_live() {
        let string = decision("검사:움직씨 = { \"(매마디)마다 { }\" 보여주기. }");
        let comment = decision("검사:움직씨 = { // (매마디)마다 { }\n1 보여주기. }");
        assert_eq!(string.mode, ExecutionMode::OneShot);
        assert_eq!(comment.mode, ExecutionMode::OneShot);
    }
}
