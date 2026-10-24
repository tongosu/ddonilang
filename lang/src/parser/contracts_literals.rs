//! Existing effect checks and structured literal parsing; original method bodies and Parser ownership retained.
use super::*;

impl Parser {
    pub(super) fn validate_eval_body(
        &self,
        body: &Body,
        mode: ThunkEvalMode,
        span: Span,
    ) -> Result<(), ParseError> {
        let label = match mode {
            ThunkEvalMode::Value => "}한것",
            ThunkEvalMode::Bool => "}인것",
            ThunkEvalMode::Not => "}아닌것",
            ThunkEvalMode::Do => "}하고",
            ThunkEvalMode::Pipe => "}해서",
        };
        if matches!(mode, ThunkEvalMode::Bool | ThunkEvalMode::Not) {
            if self.body_has_mutation(body) {
                return Err(ParseError {
                    span,
                    message: format!("Gate0: {} 안에서는 '<-'를 사용할 수 없습니다", label),
                });
            }
            if self.body_has_eval_do(body) {
                return Err(ParseError {
                    span,
                    message: format!("Gate0: {} 안에서는 '}}하고'를 사용할 수 없습니다", label),
                });
            }
            if self.body_has_random(body) {
                return Err(ParseError {
                    span,
                    message: format!("Gate0: {} 안에서는 무작위 함수를 사용할 수 없습니다", label),
                });
            }
        }
        if matches!(mode, ThunkEvalMode::Value | ThunkEvalMode::Pipe)
            && self.body_has_mutation(body)
        {
            return Err(ParseError {
                span,
                message: format!("Gate0: {} 안에서는 '<-'를 사용할 수 없습니다", label),
            });
        }
        Ok(())
    }

    pub(super) fn validate_guard_body(&self, body: &Body, span: Span) -> Result<(), ParseError> {
        if self.body_has_mutation(body) {
            return Err(ParseError {
                span,
                message: "Gate0: 늘지켜보고 안에서는 '<-'를 사용할 수 없습니다".to_string(),
            });
        }
        if self.body_has_eval_do(body) {
            return Err(ParseError {
                span,
                message: "Gate0: 늘지켜보고 안에서는 '}하고'를 사용할 수 없습니다".to_string(),
            });
        }
        if self.body_has_random(body) {
            return Err(ParseError {
                span,
                message: "Gate0: 늘지켜보고 안에서는 무작위 함수를 사용할 수 없습니다".to_string(),
            });
        }
        for stmt in &body.stmts {
            match stmt {
                Stmt::Expr { expr, .. } => {
                    if !matches!(expr.kind, ExprKind::Call { .. }) {
                        return Err(ParseError {
                            span: expr.span,
                            message: "Gate0: 늘지켜보고 안에서는 호출식만 허용합니다".to_string(),
                        });
                    }
                }
                _ => {
                    return Err(ParseError {
                        span,
                        message: "Gate0: 늘지켜보고 안에서는 호출식만 허용합니다".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_quantifier_body(&self, body: &Body, span: Span) -> Result<(), ParseError> {
        if self.body_has_mutation(body) {
            return Err(ParseError {
                span,
                message: "AGE4: 양화 블록 안에서는 '<-'를 사용할 수 없습니다".to_string(),
            });
        }
        if self.body_has_show(body) {
            return Err(ParseError {
                span,
                message: "AGE4: 양화 블록 안에서는 '보여주기'를 사용할 수 없습니다".to_string(),
            });
        }
        Ok(())
    }

    pub(super) fn body_has_mutation(&self, body: &Body) -> bool {
        body.stmts.iter().any(|stmt| self.stmt_has_mutation(stmt))
    }

    pub(super) fn body_has_eval_do(&self, body: &Body) -> bool {
        body.stmts.iter().any(|stmt| self.stmt_has_eval_do(stmt))
    }

    pub(super) fn body_has_random(&self, body: &Body) -> bool {
        body.stmts.iter().any(|stmt| self.stmt_has_random(stmt))
    }

    pub(super) fn body_has_show(&self, body: &Body) -> bool {
        body.stmts.iter().any(|stmt| self.stmt_has_show(stmt))
    }

    pub(super) fn stmt_has_mutation(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::SumDefinition { .. } => false,
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.expr_has_mutation(subject)
                    || arms.iter().any(|arm| self.body_has_mutation(&arm.body))
                    || default_body.as_ref().map_or(false, |body| self.body_has_mutation(body))
            }
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_mutation(expr))
            }),
            Stmt::Definition { value, .. } => self.expr_has_mutation(value),
            Stmt::Mutate { .. } | Stmt::ResultFlow { .. } | Stmt::DerivedBinding {..} => true,
            Stmt::Expr { expr, .. } => self.expr_has_mutation(expr),
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_mutation(expr))
                    || self.body_has_mutation(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_mutation(expr))
                    || self.expr_has_mutation(payload)
                    || self.expr_has_mutation(receiver)
            }
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => self.expr_has_mutation(expr),
            Stmt::Return { value, .. } => self.expr_has_mutation(value),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_mutation(condition)
                    || self.body_has_mutation(then_body)
                    || else_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_mutation(body))
            }
            Stmt::Try { action, body, .. } => {
                self.expr_has_mutation(action) || self.body_has_mutation(body)
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_mutation(&branch.condition)
                        || self.body_has_mutation(&branch.body)
                }) || self.body_has_mutation(else_body)
            }
            Stmt::Repeat { body, .. } => self.body_has_mutation(body),
            Stmt::BeatBlock { body, .. } => self.body_has_mutation(body),
            Stmt::Hook { body, .. } => self.body_has_mutation(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_mutation(condition) || self.body_has_mutation(body),
            Stmt::While {
                condition, body, ..
            } => self.expr_has_mutation(condition) || self.body_has_mutation(body),
            Stmt::ForEach { iterable, body, .. } => {
                self.expr_has_mutation(iterable) || self.body_has_mutation(body)
            }
            Stmt::Quantifier { body, .. } => self.body_has_mutation(body),
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } => false,
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_mutation(condition)
                    || self.body_has_mutation(else_body)
                    || then_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_mutation(body))
            }
            Stmt::Guard {
                condition, body, ..
            } => self.expr_has_mutation(condition) || self.body_has_mutation(body),
            Stmt::MetaBlock { .. } => false,
            Stmt::PublicObservation { fields, .. } => fields
                .iter()
                .any(|field| self.expr_has_mutation(&field.value)),
            Stmt::Pragma { .. } => false,
        }
    }

    pub(super) fn stmt_has_eval_do(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::SumDefinition { .. } => false,
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.expr_has_eval_do(subject)
                    || arms.iter().any(|arm| self.body_has_eval_do(&arm.body))
                    || default_body.as_ref().map_or(false, |body| self.body_has_eval_do(body))
            }
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_eval_do(expr))
            }),
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => self.expr_has_eval_do(value),
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => self.expr_has_eval_do(producer) || self.expr_has_eval_do(destination),
            Stmt::Expr { expr, .. } => self.expr_has_eval_do(expr),
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_eval_do(expr))
                    || self.body_has_eval_do(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_eval_do(expr))
                    || self.expr_has_eval_do(payload)
                    || self.expr_has_eval_do(receiver)
            }
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => self.expr_has_eval_do(expr),
            Stmt::Return { value, .. } => self.expr_has_eval_do(value),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_eval_do(condition)
                    || self.body_has_eval_do(then_body)
                    || else_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_eval_do(body))
            }
            Stmt::Try { action, body, .. } => {
                self.expr_has_eval_do(action) || self.body_has_eval_do(body)
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_eval_do(&branch.condition) || self.body_has_eval_do(&branch.body)
                }) || self.body_has_eval_do(else_body)
            }
            Stmt::Repeat { body, .. } => self.body_has_eval_do(body),
            Stmt::BeatBlock { body, .. } => self.body_has_eval_do(body),
            Stmt::Hook { body, .. } => self.body_has_eval_do(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_eval_do(condition) || self.body_has_eval_do(body),
            Stmt::While {
                condition, body, ..
            } => self.expr_has_eval_do(condition) || self.body_has_eval_do(body),
            Stmt::ForEach { iterable, body, .. } => {
                self.expr_has_eval_do(iterable) || self.body_has_eval_do(body)
            }
            Stmt::Quantifier { body, .. } => self.body_has_eval_do(body),
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } => false,
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_eval_do(condition)
                    || self.body_has_eval_do(else_body)
                    || then_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_eval_do(body))
            }
            Stmt::Guard {
                condition, body, ..
            } => self.expr_has_eval_do(condition) || self.body_has_eval_do(body),
            Stmt::Mutate { .. } => false,
            Stmt::MetaBlock { .. } => false,
            Stmt::PublicObservation { fields, .. } => fields
                .iter()
                .any(|field| self.expr_has_eval_do(&field.value)),
            Stmt::Pragma { .. } => false,
        }
    }

    pub(super) fn stmt_has_random(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::SumDefinition { .. } => false,
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.expr_has_random(subject)
                    || arms.iter().any(|arm| self.body_has_random(&arm.body))
                    || default_body.as_ref().map_or(false, |body| self.body_has_random(body))
            }
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_random(expr))
            }),
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => self.expr_has_random(value),
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => self.expr_has_random(producer) || self.expr_has_random(destination),
            Stmt::Expr { expr, .. } => self.expr_has_random(expr),
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_random(expr))
                    || self.body_has_random(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_random(expr))
                    || self.expr_has_random(payload)
                    || self.expr_has_random(receiver)
            }
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => self.expr_has_random(expr),
            Stmt::Return { value, .. } => self.expr_has_random(value),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_random(condition)
                    || self.body_has_random(then_body)
                    || else_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_random(body))
            }
            Stmt::Try { action, body, .. } => {
                self.expr_has_random(action) || self.body_has_random(body)
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_random(&branch.condition) || self.body_has_random(&branch.body)
                }) || self.body_has_random(else_body)
            }
            Stmt::Repeat { body, .. } => self.body_has_random(body),
            Stmt::BeatBlock { body, .. } => self.body_has_random(body),
            Stmt::Hook { body, .. } => self.body_has_random(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_random(condition) || self.body_has_random(body),
            Stmt::While {
                condition, body, ..
            } => self.expr_has_random(condition) || self.body_has_random(body),
            Stmt::ForEach { iterable, body, .. } => {
                self.expr_has_random(iterable) || self.body_has_random(body)
            }
            Stmt::Quantifier { body, .. } => self.body_has_random(body),
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } => false,
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_random(condition)
                    || self.body_has_random(else_body)
                    || then_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_random(body))
            }
            Stmt::Guard {
                condition, body, ..
            } => self.expr_has_random(condition) || self.body_has_random(body),
            Stmt::Mutate { .. } => false,
            Stmt::MetaBlock { .. } => false,
            Stmt::PublicObservation { fields, .. } => fields
                .iter()
                .any(|field| self.expr_has_random(&field.value)),
            Stmt::Pragma { .. } => false,
        }
    }

    pub(super) fn stmt_has_show(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::SumDefinition { .. } => false,
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.expr_has_show(subject)
                    || arms.iter().any(|arm| self.body_has_show(&arm.body))
                    || default_body.as_ref().map_or(false, |body| self.body_has_show(body))
            }
            Stmt::Show { .. } | Stmt::Inspect { .. } => true,
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_show(expr))
            }),
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => self.expr_has_show(value),
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => self.expr_has_show(producer) || self.expr_has_show(destination),
            Stmt::Expr { expr, .. } => self.expr_has_show(expr),
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_show(expr))
                    || self.body_has_show(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_show(expr))
                    || self.expr_has_show(payload)
                    || self.expr_has_show(receiver)
            }
            Stmt::Return { value, .. } => self.expr_has_show(value),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_show(condition)
                    || self.body_has_show(then_body)
                    || else_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_show(body))
            }
            Stmt::Try { action, body, .. } => {
                self.expr_has_show(action) || self.body_has_show(body)
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_show(&branch.condition) || self.body_has_show(&branch.body)
                }) || self.body_has_show(else_body)
            }
            Stmt::Repeat { body, .. } => self.body_has_show(body),
            Stmt::BeatBlock { body, .. } => self.body_has_show(body),
            Stmt::Hook { body, .. } => self.body_has_show(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_show(condition) || self.body_has_show(body),
            Stmt::While {
                condition, body, ..
            } => self.expr_has_show(condition) || self.body_has_show(body),
            Stmt::ForEach { iterable, body, .. } => {
                self.expr_has_show(iterable) || self.body_has_show(body)
            }
            Stmt::Quantifier { body, .. } => self.body_has_show(body),
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_show(condition)
                    || self.body_has_show(else_body)
                    || then_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_show(body))
            }
            Stmt::Guard {
                condition, body, ..
            } => self.expr_has_show(condition) || self.body_has_show(body),
            Stmt::Mutate { target, value, .. } => {
                self.expr_has_show(target) || self.expr_has_show(value)
            }
            Stmt::Break { .. }
            | Stmt::ContinueLoop { .. }
            | Stmt::MetaBlock { .. }
            | Stmt::Pragma { .. } => false,
            Stmt::PublicObservation { fields, .. } => {
                fields.iter().any(|field| self.expr_has_show(&field.value))
            }
        }
    }

    pub(super) fn expr_has_mutation(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::SumBranchValue {payload,..} => payload.iter().any(|value|self.expr_has_mutation(&value.value)),
            ExprKind::TupleConstructor { elements } => elements.iter().any(|e| self.expr_has_mutation(e)),
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_mutation(thunk),
            ExprKind::Thunk(_) => false,
            ExprKind::FieldAccess { target, .. } => self.expr_has_mutation(target),
            ExprKind::SeedLiteral { body, .. } => self.expr_has_mutation(body),
            ExprKind::Call { args, .. } => args.iter().any(|arg| self.expr_has_mutation(&arg.expr)),
            ExprKind::Infix { left, right, .. } => {
                self.expr_has_mutation(left) || self.expr_has_mutation(right)
            }
            ExprKind::Suffix { value, .. } => self.expr_has_mutation(value),
            ExprKind::Pipe { stages } => stages.iter().any(|stage| self.expr_has_mutation(stage)),
            ExprKind::Pack { fields } => {
                fields.iter().any(|(_, expr)| self.expr_has_mutation(expr))
            }
            ExprKind::Assertion(_) => false,
            ExprKind::Formula(_) => false,
            ExprKind::Template(_) => false,
            ExprKind::TemplateRender { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_mutation(expr))
            }
            ExprKind::FormulaEval { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_mutation(expr))
            }
            ExprKind::Nuance { expr, .. } => self.expr_has_mutation(expr),
            _ => false,
        }
    }

    pub(super) fn expr_exec_has_mutation(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Thunk(body) => self.body_has_mutation(body),
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_mutation(thunk),
            _ => self.expr_has_mutation(expr),
        }
    }

    pub(super) fn expr_has_eval_do(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::SumBranchValue {payload,..} => payload.iter().any(|value|self.expr_has_eval_do(&value.value)),
            ExprKind::Eval { thunk, mode } => {
                if matches!(mode, ThunkEvalMode::Do) {
                    return true;
                }
                self.expr_has_eval_do(thunk)
            }
            ExprKind::Thunk(_) => false,
            ExprKind::FieldAccess { target, .. } => self.expr_has_eval_do(target),
            ExprKind::SeedLiteral { body, .. } => self.expr_has_eval_do(body),
            ExprKind::Call { args, .. } => args.iter().any(|arg| self.expr_has_eval_do(&arg.expr)),
            ExprKind::Infix { left, right, .. } => {
                self.expr_has_eval_do(left) || self.expr_has_eval_do(right)
            }
            ExprKind::Suffix { value, .. } => self.expr_has_eval_do(value),
            ExprKind::Pipe { stages } => stages.iter().any(|stage| self.expr_has_eval_do(stage)),
            ExprKind::TupleConstructor { elements } => elements.iter().any(|e| self.expr_has_eval_do(e)),
            ExprKind::Pack { fields } => fields.iter().any(|(_, expr)| self.expr_has_eval_do(expr)),
            ExprKind::Assertion(_) => false,
            ExprKind::Formula(_) => false,
            ExprKind::Template(_) => false,
            ExprKind::TemplateRender { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_eval_do(expr))
            }
            ExprKind::FormulaEval { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_eval_do(expr))
            }
            ExprKind::Nuance { expr, .. } => self.expr_has_eval_do(expr),
            _ => false,
        }
    }

    pub(super) fn expr_has_random(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::SumBranchValue {payload,..} => payload.iter().any(|value|self.expr_has_random(&value.value)),
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_random(thunk),
            ExprKind::Thunk(_) => false,
            ExprKind::FieldAccess { target, .. } => self.expr_has_random(target),
            ExprKind::SeedLiteral { body, .. } => self.expr_has_random(body),
            ExprKind::Call { func, args } => {
                if is_random_func(func) {
                    return true;
                }
                args.iter().any(|arg| self.expr_has_random(&arg.expr))
            }
            ExprKind::Infix { left, right, .. } => {
                self.expr_has_random(left) || self.expr_has_random(right)
            }
            ExprKind::Suffix { value, .. } => self.expr_has_random(value),
            ExprKind::Pipe { stages } => stages.iter().any(|stage| self.expr_has_random(stage)),
            ExprKind::TupleConstructor { elements } => elements.iter().any(|e| self.expr_has_random(e)),
            ExprKind::Pack { fields } => fields.iter().any(|(_, expr)| self.expr_has_random(expr)),
            ExprKind::Assertion(_) => false,
            ExprKind::Formula(_) => false,
            ExprKind::Template(_) => false,
            ExprKind::TemplateRender { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_random(expr))
            }
            ExprKind::FormulaEval { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_random(expr))
            }
            ExprKind::Nuance { expr, .. } => self.expr_has_random(expr),
            _ => false,
        }
    }

    pub(super) fn expr_has_show(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::SumBranchValue {payload,..} => payload.iter().any(|value|self.expr_has_show(&value.value)),
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_show(thunk),
            ExprKind::Thunk(_) => false,
            ExprKind::FieldAccess { target, .. } => self.expr_has_show(target),
            ExprKind::SeedLiteral { body, .. } => self.expr_has_show(body),
            ExprKind::Call { args, .. } => args.iter().any(|arg| self.expr_has_show(&arg.expr)),
            ExprKind::Infix { left, right, .. } => {
                self.expr_has_show(left) || self.expr_has_show(right)
            }
            ExprKind::Suffix { value, .. } => self.expr_has_show(value),
            ExprKind::Pipe { stages } => stages.iter().any(|stage| self.expr_has_show(stage)),
            ExprKind::TupleConstructor { elements } => elements.iter().any(|e| self.expr_has_show(e)),
            ExprKind::Pack { fields } => fields.iter().any(|(_, expr)| self.expr_has_show(expr)),
            ExprKind::Assertion(_) => false,
            ExprKind::Formula(_) => false,
            ExprKind::Template(_) => false,
            ExprKind::StateMachine(_) => false,
            ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_show(expr))
            }
            ExprKind::Nuance { expr, .. } => self.expr_has_show(expr),
            _ => false,
        }
    }

    pub(super) fn expr_exec_has_show(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Thunk(body) => self.body_has_show(body),
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_show(thunk),
            _ => self.expr_has_show(expr),
        }
    }

    pub(super) fn expr_exec_has_random(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Thunk(body) => self.body_has_random(body),
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_random(thunk),
            _ => self.expr_has_random(expr),
        }
    }

    pub(super) fn parse_at_payload(&mut self, at_end: usize) -> Result<AtSuffix, ParseError> {
        if let TokenKind::StringLit(path) = &self.current().kind {
            let path = path.clone();
            let token = self.advance();
            if token.span.start != at_end {
                return Err(self.error("@ 뒤 자원은 붙여서 써야 합니다"));
            }
            return Ok(AtSuffix::Asset(path));
        }
        let unit = self.parse_unit_chain(at_end)?;
        if !is_known_unit(&unit) {
            return Err(ParseError {
                span: self.current_span(),
                message: format!(
                    "단위 '{}'를 알 수 없습니다. 핀 고정은 값:핀을 사용하세요",
                    unit
                ),
            });
        }
        Ok(AtSuffix::Unit(unit))
    }

    pub(super) fn parse_unit_chain(&mut self, at_end: usize) -> Result<String, ParseError> {
        let mut out = String::new();
        let mut prev_end = at_end;
        while !self.is_at_end() {
            let token = self.current().clone();
            if token.span.start != prev_end {
                break;
            }
            match &token.kind {
                TokenKind::Ident(name) => out.push_str(name),
                TokenKind::Integer(n) => out.push_str(&n.to_string()),
                TokenKind::Slash => out.push('/'),
                TokenKind::Caret => out.push('^'),
                _ => break,
            }
            prev_end = token.span.end;
            self.advance();
        }
        if out.is_empty() {
            return Err(self.error("@ 뒤 단위 또는 자원"));
        }
        Ok(out)
    }

    pub(super) fn infer_mood_from_suffix(&self) -> Mood {
        let interrogative = ["까", "니", "냐", "나요", "습니까", "인가", "인가요"];
        let suggestive = ["자", "합시다"];
        let imperative = ["해줘", "주세요", "하세요", "해라", "하라", "줘"];
        let exclamative = ["야", "네", "구나"];

        let mut idx = self.pos;
        while idx > 0 {
            idx -= 1;
            match &self.tokens[idx].kind {
                TokenKind::Ident(raw) => {
                    if interrogative.iter().any(|s| raw.ends_with(s)) {
                        return Mood::Interrogative;
                    }
                    if suggestive.iter().any(|s| raw.ends_with(s)) {
                        return Mood::Suggestive;
                    }
                    if imperative.iter().any(|s| raw.ends_with(s)) {
                        return Mood::Imperative;
                    }
                    if exclamative.iter().any(|s| raw.ends_with(s)) {
                        return Mood::Exclamative;
                    }
                    return Mood::Declarative;
                }
                TokenKind::Dot
                | TokenKind::Question
                | TokenKind::Bang
                | TokenKind::Comma
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace => {
                    continue;
                }
                _ => return Mood::Declarative,
            }
        }

        Mood::Declarative
    }

    pub(super) fn parse_arg_suffix(&mut self) -> Result<ArgSuffix, ParseError> {
        let mut fixed_pin = None;
        let mut josa = None;
        if self.check(&TokenKind::Colon) {
            self.advance();
            let pin_token = self.expect_ident("핀")?;
            let mut pin = pin_token.raw.clone();
            let mut pin_end = pin_token.span.end;
            while let TokenKind::Josa(value) = &self.current().kind {
                if self.current().span.start != pin_end {
                    break;
                }
                pin.push_str(value);
                pin_end = self.current().span.end;
                self.advance();
            }
            fixed_pin = Some(pin);
        }
        if self.check(&TokenKind::Tilde) {
            self.advance();
            if let TokenKind::Josa(value) = &self.current().kind {
                josa = Some(value.clone());
                self.advance();
            } else {
                return Err(self.error("~ 뒤 조사"));
            }
        } else if let TokenKind::Josa(value) = &self.current().kind {
            josa = Some(value.clone());
            self.advance();
        }
        if self.check(&TokenKind::At) {
            return Err(self.error("접미 순서는 값@단위/자원:핀~조사 여야 합니다 (@는 :핀 앞)"));
        }
        if self.check(&TokenKind::Colon) {
            return Err(self.error("접미 순서는 값@단위/자원:핀~조사 여야 합니다 (:핀은 ~조사 앞)"));
        }
        if self.check(&TokenKind::Tilde) {
            return Err(self.error("접미 순서는 값@단위/자원:핀~조사 여야 합니다 (~조사는 맨 끝)"));
        }
        let reason = if fixed_pin.is_some() {
            BindingReason::UserFixed
        } else if josa.is_some() {
            BindingReason::Dictionary
        } else {
            BindingReason::Positional
        };
        Ok(ArgSuffix {
            josa,
            binding_reason: reason,
            fixed_pin,
        })
    }

    pub(super) fn try_parse_arg_suffix(&mut self) -> Result<Option<ArgSuffix>, ParseError> {
        let colon_starts_pin = self.check(&TokenKind::Colon)
            && self
                .tokens
                .get(self.pos + 1)
                .map(|token| matches!(token.kind, TokenKind::Ident(_) | TokenKind::Josa(_)))
                .unwrap_or(false);
        if !(colon_starts_pin
            || self.check(&TokenKind::Tilde)
            || matches!(self.current().kind, TokenKind::Josa(_)))
        {
            return Ok(None);
        }
        self.parse_arg_suffix().map(Some)
    }

    pub(super) fn bind_arg_with_suffix(&mut self, expr: Expr, suffix: ArgSuffix) -> ArgBinding {
        ArgBinding {
            id: self.next_id(),
            span: expr.span,
            expr,
            josa: suffix.josa,
            resolved_pin: suffix.fixed_pin,
            binding_reason: suffix.binding_reason,
        }
    }

    pub(super) fn try_parse_bare_bound_call(&mut self, first_expr: Expr) -> Result<Expr, ParseError> {
        // R1.1 retains the approved postfix value receiver without inventing
        // a josa or a public named pin. Other bare calls keep their grammar.
        if self.source_epoch==SourceEpoch::V25&&matches!(self.current().raw.as_str(),"가리키기"|"읽기") {
            let token=self.advance();
            let span=first_expr.span.merge(&self.to_ast_span(token.span));
            let arg=ArgBinding{id:self.next_id(),span:first_expr.span,expr:first_expr,josa:None,
                resolved_pin:None,binding_reason:BindingReason::Positional};
            return Ok(Expr::new(self.next_id(),span,ExprKind::Call{args:vec![arg],func:token.raw}));
        }
        let checkpoint = self.pos;
        let Some(first_suffix) = self.try_parse_arg_suffix()? else {
            return Ok(first_expr);
        };

        let mut args = vec![self.bind_arg_with_suffix(first_expr.clone(), first_suffix)];
        loop {
            let arg_checkpoint = self.pos;
            if !self.token_starts_expr(&self.current().kind) {
                break;
            }
            let expr = self.parse_primary_inner(false)?;
            let Some(suffix) = self.try_parse_arg_suffix()? else {
                self.pos = arg_checkpoint;
                break;
            };
            args.push(self.bind_arg_with_suffix(expr, suffix));
        }

        if let Some((func_name, func_span)) = self.parse_call_name()? {
            let span = args
                .first()
                .map(|arg| arg.span)
                .unwrap_or(first_expr.span)
                .merge(&func_span);
            return Ok(Expr::new(
                self.next_id(),
                span,
                ExprKind::Call {
                    args,
                    func: func_name,
                },
            ));
        }

        self.pos = checkpoint;
        Ok(first_expr)
    }

    pub(super) fn parse_call_arg(&mut self) -> Result<ArgBinding, ParseError> {
        if self.check_ident() && self.peek_is_equals() {
            let pin = self.expect_ident("핀")?.raw.clone();
            self.expect(&TokenKind::Equals, "=")?;
            let arg_expr = self.parse_expr()?;
            let suffix = self.parse_arg_suffix()?;
            return Ok(ArgBinding {
                id: self.next_id(),
                span: arg_expr.span,
                expr: arg_expr,
                josa: suffix.josa,
                resolved_pin: Some(pin),
                binding_reason: BindingReason::UserFixed,
            });
        }
        let arg_expr = self.parse_expr()?;
        let suffix = self.parse_arg_suffix()?;
        Ok(ArgBinding {
            id: self.next_id(),
            span: arg_expr.span,
            expr: arg_expr,
            josa: suffix.josa,
            resolved_pin: suffix.fixed_pin,
            binding_reason: suffix.binding_reason,
        })
    }

    pub(super) fn parse_tensor_type(&mut self) -> Result<TypeRef, ParseError> {
        self.expect(&TokenKind::LParen, "텐서씨 (")?;
        let mut pins = HashSet::new();
        let mut element = None;
        let mut shape = None;
        while !self.check(&TokenKind::RParen) {
            let token = self.expect_ident("텐서씨 이름 핀")?;
            let pin = self.source.get(token.span.start..token.span.end)
                .ok_or_else(|| self.error("텐서씨 핀 원문 범위"))?.to_owned();
            if !pins.insert(pin.clone()) { return Err(self.error("중복 텐서씨 핀")); }
            self.expect(&TokenKind::Equals, "텐서씨 핀 =")?;
            match pin.as_str() {
                "원소씨" => {
                    let kind = self.parse_type_ref()?;
                    if !matches!(&kind, TypeRef::Named(name) if name == "셈수") {
                        return Err(self.error("V1 Tensor 원소씨는 정본 셈수만 허용됩니다"));
                    }
                    element = Some(Box::new(kind));
                }
                "모양" => shape = Some(self.parse_tensor_static_shape()?),
                _ => return Err(self.error("등록되지 않은 텐서씨 핀")),
            }
            if !self.check(&TokenKind::Comma) { break; }
            self.advance();
        }
        self.expect(&TokenKind::RParen, "텐서씨 )")?;
        if self.advance().raw != "텐서씨" { return Err(self.error("정본 텐서씨가 필요합니다")); }
        Ok(TypeRef::Tensor {
            element: element.ok_or_else(|| self.error("텐서씨 원소씨 누락"))?,
            shape: shape.ok_or_else(|| self.error("텐서씨 모양 누락"))?,
        })
    }

    pub(super) fn parse_tensor_static_shape(&mut self) -> Result<Vec<u64>, ParseError> {
        self.expect(&TokenKind::LParen, "정적 모양 (")?;
        let mut axes = Vec::new();
        if self.check(&TokenKind::RParen) { self.advance(); return Ok(axes); }
        loop {
            let token = self.advance();
            if !matches!(token.kind, TokenKind::Integer(_)) || !token.raw.bytes().all(|b| b.is_ascii_digit()) {
                return Err(self.error("Tensor 축은 정적인 음이 아닌 정수여야 합니다"));
            }
            let axis = token.raw.parse::<u64>().map_err(|_| self.error("Tensor 축 범위 초과"))?;
            axes.push(axis);
            if self.check(&TokenKind::Comma) {
                self.advance();
                if self.check(&TokenKind::RParen) { self.advance(); break; }
            } else {
                if axes.len() == 1 { return Err(self.error("rank-1 모양은 (d,)로 적어야 합니다")); }
                self.expect(&TokenKind::RParen, "정적 모양 )")?;
                break;
            }
        }
        if !axes.contains(&0) {
            axes.iter().try_fold(1u64, |n, axis| n.checked_mul(*axis))
                .ok_or_else(|| self.error("Tensor 원소 수 곱셈 범위 초과"))?;
        }
        Ok(axes)
    }

    pub(super) fn parse_tensor_provider(&mut self, provider: &str) -> Result<Expr, ParseError> {
        let start_token = self.expect(&TokenKind::LParen, "(")?;
        let start = self.to_ast_span(start_token.span);
        let mut pins = HashSet::new();
        let mut input = None;
        let mut element_type = None;
        let mut shape = None;
        while !self.check(&TokenKind::RParen) {
            let pin_token = self.expect_ident("Tensor 이름 핀")?;
            // RD-510 pins are exact authored spellings, not dialect aliases.
            let pin = self.source.get(pin_token.span.start..pin_token.span.end)
                .ok_or_else(|| self.error("Tensor 핀 원문 범위를 확인할 수 없습니다"))?
                .to_owned();
            if !pins.insert(pin.clone()) { return Err(self.error("중복 Tensor 핀")); }
            self.expect(&TokenKind::Equals, "Tensor 핀 =")?;
            match (provider, pin.as_str()) {
                ("텐서로", "차림") | ("차림으로", "텐서") => input = Some(Box::new(self.parse_expr()?)),
                ("텐서로", "원소씨") => {
                    let kind = self.parse_type_ref()?;
                    if !matches!(&kind, TypeRef::Named(name) if name == "셈수") {
                        return Err(self.error("V1 Tensor 원소씨는 정본 셈수만 허용됩니다"));
                    }
                    element_type = Some(kind);
                }
                ("텐서로", "모양") => shape = Some(self.parse_tensor_static_shape()?),
                _ => return Err(self.error(&format!("등록되지 않은 Tensor 핀: {provider} / {pin}"))),
            }
            if !self.check(&TokenKind::Comma) { break; }
            self.advance();
        }
        self.expect(&TokenKind::RParen, ")")?;
        let end = self.advance();
        if end.raw != provider { return Err(self.error("정확한 Tensor provider가 필요합니다")); }
        let input = input.ok_or_else(|| self.error("Tensor 입력 핀 누락"))?;
        let value = if provider == "텐서로" {
            TensorProviderExpr::FromCollection {
                input,
                element_type: element_type.ok_or_else(|| self.error("원소씨 핀 누락"))?,
                shape: shape.ok_or_else(|| self.error("모양 핀 누락"))?,
            }
        } else { TensorProviderExpr::ToCollection { input } };
        Ok(Expr::new(self.next_id(), start.merge(&self.to_ast_span(end.span)), ExprKind::Tensor(value)))
    }

    pub(super) fn parse_injection_fields(
        &mut self,
        args: Vec<ArgBinding>,
        span: Span,
    ) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut fields = Vec::new();
        let mut seen = HashSet::new();
        for arg in args {
            if arg.josa.is_some() {
                return Err(ParseError {
                    span,
                    message: "주입은 이름=값 형태만 허용됩니다".to_string(),
                });
            }
            let Some(pin) = arg.resolved_pin else {
                return Err(ParseError {
                    span,
                    message: "주입은 이름=값 형태만 허용됩니다".to_string(),
                });
            };
            if !matches!(arg.binding_reason, BindingReason::UserFixed) {
                return Err(ParseError {
                    span,
                    message: "주입은 이름=값 형태만 허용됩니다".to_string(),
                });
            }
            if seen.contains(&pin) {
                return Err(ParseError {
                    span,
                    message: format!("주입 키 '{}'가 중복되었습니다", pin),
                });
            }
            seen.insert(pin.clone());
            fields.push((pin, arg.expr));
        }
        Ok(fields)
    }

    pub(super) fn peek_tagged_template(&self) -> bool {
        let Some(t0) = self.tokens.get(self.pos) else {
            return false;
        };
        if !matches!(t0.kind, TokenKind::LParen) {
            return false;
        }
        let Some(t1) = self.tokens.get(self.pos + 1) else {
            return false;
        };
        if !matches!(t1.kind, TokenKind::Atom(_)) {
            return false;
        }
        let Some(t2) = self.tokens.get(self.pos + 2) else {
            return false;
        };
        if !matches!(t2.kind, TokenKind::RParen) {
            return false;
        }
        let Some(t3) = self.tokens.get(self.pos + 3) else {
            return false;
        };
        matches!(t3.kind, TokenKind::TemplateBlock(_))
    }

    pub(super) fn parse_tagged_template(&mut self) -> Result<Expr, ParseError> {
        let lparen = self.expect(&TokenKind::LParen, "(")?;
        let tag_token = self.advance();
        let tag = match tag_token.kind {
            TokenKind::Atom(tag) => tag,
            _ => {
                return Err(ParseError {
                    span: self.to_ast_span(tag_token.span),
                    message: "글무늬 태그는 (#tag) 형태여야 합니다".to_string(),
                })
            }
        };
        self.expect(&TokenKind::RParen, ")")?;
        let template_token = self.advance();
        let TokenKind::TemplateBlock(raw) = &template_token.kind else {
            return Err(ParseError {
                span: self.to_ast_span(template_token.span),
                message: "글무늬 블록이 필요합니다".to_string(),
            });
        };
        let span = self
            .to_ast_span(lparen.span)
            .merge(&self.to_ast_span(template_token.span));
        let template = self.parse_template_block(raw, span, Some(tag))?;
        Ok(Expr::new(
            self.next_id(),
            span,
            ExprKind::Template(template),
        ))
    }

    pub(super) fn peek_tagged_formula(&self) -> bool {
        let Some(t0) = self.tokens.get(self.pos) else {
            return false;
        };
        if !matches!(t0.kind, TokenKind::LParen) {
            return false;
        }
        let Some(t1) = self.tokens.get(self.pos + 1) else {
            return false;
        };
        if !matches!(t1.kind, TokenKind::Atom(_)) {
            return false;
        }
        let Some(t2) = self.tokens.get(self.pos + 2) else {
            return false;
        };
        if !matches!(t2.kind, TokenKind::RParen) {
            return false;
        }
        let Some(t3) = self.tokens.get(self.pos + 3) else {
            return false;
        };
        matches!(t3.kind, TokenKind::FormulaBlock(_))
    }

    pub(super) fn parse_tagged_formula(&mut self) -> Result<Expr, ParseError> {
        let lparen = self.expect(&TokenKind::LParen, "(")?;
        let tag_token = self.advance();
        let tag = match tag_token.kind {
            TokenKind::Atom(tag) => tag,
            _ => {
                return Err(ParseError {
                    span: self.to_ast_span(tag_token.span),
                    message: "수식 태그는 (#tag) 형태여야 합니다".to_string(),
                })
            }
        };
        self.expect(&TokenKind::RParen, ")")?;
        let formula_token = self.advance();
        let TokenKind::FormulaBlock(raw) = &formula_token.kind else {
            return Err(ParseError {
                span: self.to_ast_span(formula_token.span),
                message: "수식 블록이 필요합니다".to_string(),
            });
        };
        let span = self
            .to_ast_span(lparen.span)
            .merge(&self.to_ast_span(formula_token.span));
        let formula = self.parse_formula_block(raw, span, Some(tag), true)?;
        Ok(Expr::new(self.next_id(), span, ExprKind::Formula(formula)))
    }

    pub(super) fn parse_regex_literal_block(
        &mut self,
        ident_span: Span,
    ) -> Result<(RegexLiteral, Span), ParseError> {
        let open = self.expect(&TokenKind::LBrace, "{")?;
        let pattern_token = self.current().clone();
        let pattern = match &pattern_token.kind {
            TokenKind::StringLit(value) => value.clone(),
            _ => {
                return Err(ParseError {
                    span: self.to_ast_span(pattern_token.span),
                    message: "정규식 본문은 첫째 인자로 문자열 패턴이 필요합니다".to_string(),
                })
            }
        };
        self.advance();

        let mut flags = String::new();
        if self.check(&TokenKind::Comma) {
            self.advance();
            let flags_token = self.current().clone();
            flags = match &flags_token.kind {
                TokenKind::StringLit(value) => normalize_regex_flags(value),
                _ => {
                    return Err(ParseError {
                        span: self.to_ast_span(flags_token.span),
                        message: "정규식 둘째 인자는 문자열 깃발이어야 합니다".to_string(),
                    })
                }
            };
            self.advance();
        }

        if self.check(&TokenKind::Comma) {
            return Err(ParseError {
                span: self.current_span(),
                message: "정규식은 패턴과 깃발만 허용됩니다".to_string(),
            });
        }

        let close = self.expect(&TokenKind::RBrace, "}")?;
        let span = ident_span
            .merge(&self.to_ast_span(open.span))
            .merge(&self.to_ast_span(close.span));
        Ok((RegexLiteral { pattern, flags }, span))
    }

    pub(super) fn parse_template_block(
        &self,
        raw: &str,
        span: Span,
        tag: Option<String>,
    ) -> Result<Template, ParseError> {
        let body = self.decode_template_body(raw, span)?;
        let parts = self.parse_template_parts(&body, span)?;
        Ok(Template {
            raw: body,
            parts,
            tag,
        })
    }

    pub(super) fn parse_formula_block(
        &self,
        raw: &str,
        span: Span,
        tag: Option<String>,
        explicit_tag: bool,
    ) -> Result<Formula, ParseError> {
        let body = self.normalize_formula_body(raw);
        let dialect = match tag.as_deref() {
            Some("ascii") => FormulaDialect::Ascii,
            Some("ascii1") => FormulaDialect::Ascii1,
            Some("latex") => FormulaDialect::Latex,
            Some(other) => FormulaDialect::Other(other.to_string()),
            None => FormulaDialect::Ascii,
        };
        if matches!(dialect, FormulaDialect::Ascii | FormulaDialect::Ascii1)
            && !formula_body_is_ascii(&body)
        {
            return Err(ParseError {
                span,
                message: "FATAL:FORMULA_BODY_NONASCII".to_string(),
            });
        }
        Ok(Formula {
            raw: body,
            dialect,
            explicit_tag,
        })
    }

    pub(super) fn parse_state_machine_block(&self, raw: &str, span: Span) -> Result<StateMachine, ParseError> {
        let clauses = split_state_machine_clauses(raw);
        let mut states = Vec::new();
        let mut initial: Option<String> = None;
        let mut transitions = Vec::new();
        let mut checks = Vec::new();

        for clause in clauses {
            if let Some(prefix) = clause.strip_suffix("으로 이뤄짐") {
                if !states.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 상태 선언은 한 번만 허용됩니다".to_string(),
                    });
                }
                states = prefix
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(ToString::to_string)
                    .collect();
                if states.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 하나 이상의 상태가 필요합니다".to_string(),
                    });
                }
                continue;
            }
            if let Some(prefix) = clause.strip_suffix("으로 시작") {
                let name = prefix.trim();
                if name.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 시작 상태 이름이 필요합니다".to_string(),
                    });
                }
                initial = Some(name.to_string());
                continue;
            }
            if let Some(rest) = clause.strip_prefix("바뀔때마다 ") {
                let Some(check) = rest.strip_suffix(" 살피기") else {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 바뀔때마다 절은 '<세움값> 살피기'만 허용됩니다"
                            .to_string(),
                    });
                };
                let check = check.trim();
                if check.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 바뀔때마다 절에는 세움값 이름이 필요합니다".to_string(),
                    });
                }
                checks.push(check.to_string());
                continue;
            }
            if let Some((from, rhs)) = clause.split_once(" 에서 ") {
                if let Some((to, guard_name, action_name)) =
                    parse_state_machine_transition_clause(rhs)
                {
                    let from = from.trim();
                    let to = to.trim();
                    if from.is_empty() || to.is_empty() {
                        return Err(ParseError {
                            span,
                            message: "상태머신: 전이에는 출발/도착 상태가 모두 필요합니다"
                                .to_string(),
                        });
                    }
                    transitions.push(StateTransition {
                        from: from.to_string(),
                        to: to.to_string(),
                        guard_name,
                        action_name,
                    });
                    continue;
                }
            }
            return Err(ParseError {
                span,
                message: format!("상태머신: 해석할 수 없는 절입니다: {}", clause),
            });
        }

        if states.is_empty() {
            return Err(ParseError {
                span,
                message: "상태머신: '으로 이뤄짐' 절이 필요합니다".to_string(),
            });
        }
        let initial = initial.ok_or_else(|| ParseError {
            span,
            message: "상태머신: '으로 시작' 절이 필요합니다".to_string(),
        })?;
        if !states.iter().any(|state| state == &initial) {
            return Err(ParseError {
                span,
                message: format!("E_STATE_UNDECLARED: {}", initial),
            });
        }
        for transition in &transitions {
            if !states.iter().any(|state| state == &transition.from) {
                return Err(ParseError {
                    span,
                    message: format!("E_STATE_UNDECLARED: {}", transition.from),
                });
            }
            if !states.iter().any(|state| state == &transition.to) {
                return Err(ParseError {
                    span,
                    message: format!("E_STATE_UNDECLARED: {}", transition.to),
                });
            }
        }

        Ok(StateMachine {
            states,
            initial,
            transitions,
            on_transition_checks: checks,
        })
    }

    pub(super) fn parse_assertion_block(&self, raw: &str, span: Span) -> Result<Assertion, ParseError> {
        let body_source = raw.replace("\r\n", "\n").replace('\r', "\n");
        let wrapped = format!("검사:셈씨 = {{{}}}", body_source);
        let tokens = Lexer::new(&wrapped).tokenize().map_err(|err| ParseError {
            span,
            message: err.message,
        })?;
        let mut parser = Parser::new_with_mode(tokens, ParseMode::Strict);
        let program = match parser.parse_program(wrapped, "#assertion".to_string()) {
            Ok(program) => program,
            Err(_) => {
                if let Some((lhs, rhs)) = parse_symbolic_equivalence_assertion(&body_source) {
                    return Ok(Assertion {
                        body_source,
                        canon: format!("세움{{수식동치: {} = {}}}", lhs, rhs),
                    });
                }
                if let Some((lhs, rhs)) = parse_symbolic_relation_assertion(&body_source) {
                    return Ok(Assertion {
                        body_source,
                        canon: format!("세움{{수식관계: {} =:= {}}}", lhs, rhs),
                    });
                }
                return Err(ParseError {
                    span,
                    message: "세움: 유효한 본문을 해석할 수 없습니다".to_string(),
                });
            }
        };
        let mut normalizer = Normalizer::new(NormalizationLevel::N1);
        let normalized = normalizer.normalize_program(&program);
        let open = normalized.find('{').ok_or_else(|| ParseError {
            span,
            message: "세움: 본문 시작을 찾을 수 없습니다".to_string(),
        })?;
        let close = normalized.rfind('}').ok_or_else(|| ParseError {
            span,
            message: "세움: 본문 끝을 찾을 수 없습니다".to_string(),
        })?;
        let canon = format!("세움{}", &normalized[open..=close]);
        Ok(Assertion { body_source, canon })
    }

    pub(super) fn normalize_formula_body(&self, raw: &str) -> String {
        raw.replace("\r\n", "\n").replace('\r', "\n")
    }

    pub(super) fn decode_template_body(&self, raw: &str, span: Span) -> Result<String, ParseError> {
        let trimmed = raw.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            return self.unescape_string_literal(trimmed, span);
        }
        Ok(self.normalize_raw_template(raw))
    }

    pub(super) fn unescape_string_literal(&self, raw: &str, span: Span) -> Result<String, ParseError> {
        if !raw.starts_with('"') || !raw.ends_with('"') || raw.len() < 2 {
            return Err(ParseError {
                span,
                message: "글무늬 문자열은 \"...\" 형태여야 합니다".to_string(),
            });
        }
        let body = &raw[1..raw.len() - 1];
        let chars: Vec<char> = body.chars().collect();
        let mut out = String::new();
        let mut idx = 0usize;
        while idx < chars.len() {
            let ch = chars[idx];
            if ch == '\\' {
                idx += 1;
                out.push_str(&self.decode_text_escape_chars(&chars, &mut idx, span)?);
                continue;
            }
            out.push(ch);
            idx += 1;
        }
        Ok(out)
    }

    pub(super) fn decode_text_escape_chars(
        &self,
        chars: &[char],
        idx: &mut usize,
        span: Span,
    ) -> Result<String, ParseError> {
        if *idx >= chars.len() {
            return Err(ParseError {
                span,
                message: "글무늬 문자열이 닫히지 않았습니다".to_string(),
            });
        }
        for (surface, value) in [
            ("줄", "\n"),
            ("칸", "\t"),
            ("앞", "\r"),
            ("따옴", "\""),
            ("역빗금", "\\"),
        ] {
            if chars_match_at(chars, *idx, surface) {
                *idx += surface.chars().count();
                return Ok(value.to_string());
            }
        }
        for surface in ["반전끝", "굵게끝", "반전", "굵게", "되돌림"] {
            if chars_match_at(chars, *idx, surface) {
                *idx += surface.chars().count();
                return Ok(format!("\\{}", surface));
            }
        }
        for surface in ["색", "배경"] {
            if chars_match_at(chars, *idx, surface) {
                *idx += surface.chars().count();
                return self.decode_rich_markup_escape_chars(surface, chars, idx, span);
            }
        }
        let ch = chars[*idx];
        match ch {
            'n' => {
                *idx += 1;
                Ok("\n".to_string())
            }
            'r' => {
                *idx += 1;
                Ok("\r".to_string())
            }
            't' => {
                *idx += 1;
                Ok("\t".to_string())
            }
            '"' => {
                *idx += 1;
                Ok("\"".to_string())
            }
            '\\' => {
                *idx += 1;
                Ok("\\".to_string())
            }
            other => Err(ParseError {
                span,
                message: format!("알 수 없는 글 이스케이프: \\{}", other),
            }),
        }
    }

    pub(super) fn decode_rich_markup_escape_chars(
        &self,
        name: &str,
        chars: &[char],
        idx: &mut usize,
        span: Span,
    ) -> Result<String, ParseError> {
        if chars.get(*idx).copied() != Some('{') {
            return Err(ParseError {
                span,
                message: format!("콘솔 {} 표지는 '{{...}}'가 필요합니다", name),
            });
        }
        *idx += 1;
        let mut body = String::new();
        while *idx < chars.len() {
            let ch = chars[*idx];
            *idx += 1;
            if ch == '}' {
                return Ok(format!("\\{}{{{}}}", name, body));
            }
            if ch == '\n' || ch == '\r' {
                return Err(ParseError {
                    span,
                    message: "콘솔 표지 본문이 닫히지 않았습니다".to_string(),
                });
            }
            body.push(ch);
        }
        Err(ParseError {
            span,
            message: "콘솔 표지 본문이 닫히지 않았습니다".to_string(),
        })
    }

    pub(super) fn normalize_raw_template(&self, raw: &str) -> String {
        let mut text = raw.replace("\r\n", "\n").replace('\r', "\n");
        if text.starts_with('\n') {
            text.remove(0);
        }
        if text.ends_with('\n') {
            text.pop();
        }
        let mut min_indent: Option<usize> = None;
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let indent = line
                .chars()
                .take_while(|ch| matches!(ch, ' ' | '\t'))
                .count();
            min_indent = Some(min_indent.map_or(indent, |min| min.min(indent)));
        }
        let indent = min_indent.unwrap_or(0);
        let mut dedented = String::new();
        let mut lines = text.lines().peekable();
        while let Some(line) = lines.next() {
            let mut idx = 0usize;
            let mut removed = 0usize;
            for (i, ch) in line.char_indices() {
                if removed < indent && matches!(ch, ' ' | '\t') {
                    removed += 1;
                    idx = i + ch.len_utf8();
                } else {
                    break;
                }
            }
            dedented.push_str(&line[idx..]);
            if lines.peek().is_some() {
                dedented.push('\n');
            }
        }
        let mut out = String::new();
        let mut lines = dedented.split('\n').peekable();
        while let Some(line) = lines.next() {
            if line.ends_with('\\') {
                out.push_str(&line[..line.len() - 1]);
                continue;
            }
            out.push_str(line);
            if lines.peek().is_some() {
                out.push('\n');
            }
        }
        out
    }

    pub(super) fn parse_template_parts(
        &self,
        body: &str,
        span: Span,
    ) -> Result<Vec<TemplatePart>, ParseError> {
        let mut parts = Vec::new();
        let mut buf = String::new();
        let mut chars = body.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                if matches!(chars.peek(), Some('{')) {
                    chars.next();
                    buf.push('{');
                    continue;
                }
                if !buf.is_empty() {
                    parts.push(TemplatePart::Text(std::mem::take(&mut buf)));
                }
                let mut placeholder = String::new();
                let mut closed = false;
                while let Some(pc) = chars.next() {
                    if pc == '}' {
                        closed = true;
                        break;
                    }
                    if pc == '{' {
                        return Err(ParseError {
                            span,
                            message: "글무늬 자리표시자 안에는 '{'를 사용할 수 없습니다"
                                .to_string(),
                        });
                    }
                    placeholder.push(pc);
                }
                if !closed {
                    return Err(ParseError {
                        span,
                        message: "글무늬 자리표시자가 닫히지 않았습니다".to_string(),
                    });
                }
                let placeholder = placeholder.trim();
                if placeholder.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "글무늬 자리표시자 키가 비었습니다".to_string(),
                    });
                }
                let (path, format) = if let Some((left, right)) = placeholder.split_once('|') {
                    let format_raw = right.trim();
                    if !format_raw.starts_with('@') {
                        return Err(ParseError {
                            span,
                            message: "글무늬 자리표시자 포맷은 @로 시작해야 합니다".to_string(),
                        });
                    }
                    let format = self.parse_template_format(&format_raw[1..], span)?;
                    (left.trim(), Some(format))
                } else {
                    (placeholder, None)
                };
                let path = self.parse_template_path(path, span)?;
                parts.push(TemplatePart::Placeholder(TemplatePlaceholder {
                    path,
                    format,
                }));
                continue;
            }
            if ch == '}' {
                if matches!(chars.peek(), Some('}')) {
                    chars.next();
                    buf.push('}');
                    continue;
                }
                return Err(ParseError {
                    span,
                    message: "글무늬 본문에 '}'가 잘못 쓰였습니다".to_string(),
                });
            }
            buf.push(ch);
        }
        if !buf.is_empty() {
            parts.push(TemplatePart::Text(buf));
        }
        Ok(parts)
    }

    pub(super) fn parse_template_path(&self, raw: &str, span: Span) -> Result<Vec<String>, ParseError> {
        let mut out = Vec::new();
        for seg in raw.split('.') {
            if seg.is_empty() {
                return Err(ParseError {
                    span,
                    message: "글무늬 자리표시자 경로가 비어 있습니다".to_string(),
                });
            }
            if !seg.chars().all(|c| matches!(c, '가'..='힣' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')) {
                return Err(ParseError { span, message: "글무늬 자리표시자 키에는 식별자 문자를 사용해야 합니다".to_string() });
            }
            out.push(seg.to_string());
        }
        Ok(out)
    }

    pub(super) fn parse_template_format(&self, raw: &str, span: Span) -> Result<TemplateFormat, ParseError> {
        if raw.is_empty() {
            return Err(ParseError {
                span,
                message: "글무늬 포맷이 비었습니다".to_string(),
            });
        }
        if let Some(rest) = raw.strip_prefix('.') {
            let mut digits = String::new();
            let mut chars = rest.chars();
            while let Some(ch) = chars.next() {
                if ch.is_ascii_digit() {
                    digits.push(ch);
                } else {
                    let unit: String = std::iter::once(ch).chain(chars).collect();
                    let precision = digits.parse::<u8>().map_err(|_| ParseError {
                        span,
                        message: "글무늬 포맷 소수 자릿수가 올바르지 않습니다".to_string(),
                    })?;
                    if precision > 9 {
                        return Err(ParseError {
                            span,
                            message: "글무늬 포맷 소수 자릿수는 0..9 범위입니다".to_string(),
                        });
                    }
                    return Ok(TemplateFormat {
                        raw: format!("@{}", raw),
                        width: None,
                        zero_pad: false,
                        precision: Some(precision),
                        unit: Some(unit),
                    });
                }
            }
            if digits.is_empty() {
                return Err(ParseError {
                    span,
                    message: "글무늬 포맷 소수 자릿수가 비었습니다".to_string(),
                });
            }
            let precision = digits.parse::<u8>().map_err(|_| ParseError {
                span,
                message: "글무늬 포맷 소수 자릿수가 올바르지 않습니다".to_string(),
            })?;
            if precision > 9 {
                return Err(ParseError {
                    span,
                    message: "글무늬 포맷 소수 자릿수는 0..9 범위입니다".to_string(),
                });
            }
            return Ok(TemplateFormat {
                raw: format!("@{}", raw),
                width: None,
                zero_pad: false,
                precision: Some(precision),
                unit: None,
            });
        }
        let mut zero_pad = false;
        let mut digits = raw;
        if let Some(rest) = raw.strip_prefix('0') {
            zero_pad = true;
            digits = rest;
        }
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(ParseError {
                span,
                message: "글무늬 포맷 폭은 숫자여야 합니다".to_string(),
            });
        }
        let width = digits.parse::<usize>().map_err(|_| ParseError {
            span,
            message: "글무늬 포맷 폭이 올바르지 않습니다".to_string(),
        })?;
        if width < 1 || width > 99 {
            return Err(ParseError {
                span,
                message: "글무늬 포맷 폭은 1..99 범위입니다".to_string(),
            });
        }
        Ok(TemplateFormat {
            raw: format!("@{}", raw),
            width: Some(width),
            zero_pad,
            precision: None,
            unit: None,
        })
    }

}
