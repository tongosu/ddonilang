use super::{failure, PreparedProofSource};
use crate::ast::{BindingReason, Body, Expr, ExprKind, ExplicitGoalCall, ParamPin, SeedDef, SeedKind, Span, Stmt, SumBranchDecl, SumMatchArm, TopLevelItem, TypeRef};
use crate::ParseError;

pub struct ResolvedCardCall<'a> {
    pub syntax: &'a ExplicitGoalCall,
    pub definition: &'a SeedDef,
    pub parameter: &'a ParamPin,
}

pub struct ResolvedCardGoal<'a> {
    pub definition: &'a SeedDef,
    pub card_id: u64,
    pub card_span: Span,
    pub branches: &'a [SumBranchDecl],
    pub argument: &'a ParamPin,
    pub left: ResolvedCardCall<'a>,
    pub right: ResolvedCardCall<'a>,
    card_type: &'a str,
    definitions: Vec<&'a SeedDef>,
}

impl PreparedProofSource {
    pub fn resolve_card_goal(&self) -> Result<Option<ResolvedCardGoal<'_>>, ParseError> {
        let definitions: Vec<_> = self.program.items.iter().map(|item| {
            let TopLevelItem::SeedDef(seed) = item;
            seed
        }).collect();
        let mut goals = definitions.iter().filter_map(|&seed| {
            let TypeRef::Applied { name, args } = seed.result_type.as_ref()? else { return None };
            match (name.as_str(), args.as_slice()) {
                ("보임", [TypeRef::CardCallEquality { left, right }]) => Some((seed, left, right)),
                _ => None,
            }
        });
        let Some((definition, left, right)) = goals.next() else { return Ok(None) };
        let invalid = || failure(definition.span, "E_PROOF_CARD_GOAL_HEADER",
            "닫힌 두 갈래 카드 보임의 이름 있는 셈씨와 필수 받는자리 하나가 필요합니다");
        let [argument] = definition.params.as_slice() else { return Err(invalid()) };
        let TypeRef::Named(card_type) = &argument.type_ref else { return Err(invalid()) };
        if goals.next().is_some() || !card_signature(definition) {
            return Err(invalid());
        }

        let [Stmt::SumDefinition { id, span, name, branches, .. }] = self.program.root_stmts.as_slice()
            else { return Err(failure(argument.span, "E_PROOF_ROOT_UNSUPPORTED",
                "두 갈래 카드 목표의 원문에는 닫힌 고름씨 선언 하나가 필요합니다")); };
        if !self.program.origin.authored_jjaim_blocks.is_empty() || name != card_type
            || branches.len() != 2 || branches.iter().any(|branch| !branch.payload.is_empty()) {
            return Err(failure(*span, "E_PROOF_CARD_DECLARATION", "받는자리가 소유한 두 개의 빈 갈래가 필요합니다"));
        }

        let left = resolve_card_call(left, argument, &definitions, card_type)?;
        let right = resolve_card_call(right, argument, &definitions, card_type)?;
        Ok(Some(ResolvedCardGoal { definition, card_id: *id, card_span: *span, branches,
            argument, left, right, card_type, definitions }))
    }
}

fn card_signature(seed: &SeedDef) -> bool {
    seed.seed_kind == SeedKind::Semssi && seed.modifiers.is_empty()
        && seed.type_params.is_empty() && seed.proof_header.is_none()
        && seed.body.is_some()
        && matches!(seed.params.as_slice(), [pin] if !pin.optional && pin.default_value.is_none())
}

fn resolve_card_call<'a>(
    syntax: &'a ExplicitGoalCall, argument: &'a ParamPin, definitions: &[&'a SeedDef],
    card_type: &str,
) -> Result<ResolvedCardCall<'a>, ParseError> {
    if syntax.argument != argument.pin_name {
        return Err(failure(syntax.argument_span, "E_PROOF_CARD_GOAL_ARGUMENT",
            "양변은 같은 머리의 카드 받는자리를 사용해야 합니다"));
    }
    let definition = resolve_target(&syntax.name, syntax.span, definitions, card_type)?;
    let parameter = &definition.params[0];
    if parameter.pin_name != syntax.pin || parameter.type_ref != argument.type_ref {
        return Err(failure(syntax.pin_span, "E_PROOF_CARD_GOAL_PIN",
            "호출한 실제 셈씨의 카드 받는자리가 필요합니다"));
    }
    Ok(ResolvedCardCall { syntax, definition, parameter })
}

fn resolve_target<'a>(name: &str, span: Span, definitions: &[&'a SeedDef],
    card_type: &str,
) -> Result<&'a SeedDef, ParseError> {
    let mut matches = definitions.iter().copied().filter(|definition| {
        definition.canonical_name == name || definition.alternate_forms.iter().any(|alias| alias == name)
    });
    let unresolved = || failure(span, "E_PROOF_CARD_GOAL_CALL",
        "유일한 카드에서 카드로 가는 셈씨가 필요합니다");
    let definition = matches.next().ok_or_else(unresolved)?;
    if matches.next().is_some() || !card_signature(definition)
        || !matches!(&definition.result_type, Some(TypeRef::Named(name)) if name==card_type) {
        return Err(unresolved());
    }
    Ok(definition)
}

pub struct ReducedCardBranch<'a> {
    pub input: &'a SumBranchDecl,
    pub left: &'a SumBranchDecl,
    pub right: &'a SumBranchDecl,
    pub evidence_span: Span,
}

impl ResolvedCardGoal<'_> {
    /// Reduce only the adopted closed fragment. No VM, host, or effectful
    /// statement participates in definitional reduction.
    pub fn reduce_reflexivity(&self) -> Result<Vec<ReducedCardBranch<'_>>, ParseError> {
        let body = self.definition.body.as_ref().expect("resolved goal has a body");
        let arms = self.closed_arms(body, self.argument)?;
        self.branches.iter().map(|input| {
            let arm = self.select_arm(arms, input)?;
            let evidence = named_result(&arm.body, self.definition)?;
            if !matches!(&evidence.kind, ExprKind::Var(name) if name == "제같음") {
                return Err(failure(evidence.span, "E_PROOF_CARD_REFLEXIVITY_REQUIRED",
                    "이 두 갈래 카드 검사에는 해당 갈래의 제같음이 필요합니다"));
            }
            let mut stack = Vec::new();
            let left = self.reduce_call(self.left.definition, input, &mut stack, self.left.syntax.span)?;
            let right = self.reduce_call(self.right.definition, input, &mut stack, self.right.syntax.span)?;
            Ok(ReducedCardBranch { input, left, right, evidence_span: evidence.span })
        }).collect()
    }

    fn closed_arms<'a>(
        &self, body: &'a Body, parameter: &ParamPin,
    ) -> Result<&'a [SumMatchArm], ParseError> {
        let [Stmt::SumMatch { subject, arms, default_body: None, .. }] = body.stmts.as_slice()
            else { return Err(unsupported(body.span)); };
        if !matches!(&subject.kind, ExprKind::Var(name) if name == &parameter.pin_name) {
            return Err(unsupported(subject.span));
        }
        if arms.len() != self.branches.len() { return Err(unsupported(body.span)); }
        for arm in arms {
            if arm.owner != self.card_type || arm.applied_owner.is_some() || !arm.binder.is_empty() {
                return Err(unsupported(arm.span));
            }
        }
        for branch in self.branches {
            self.select_arm(arms, branch)?;
        }
        Ok(arms)
    }

    fn select_arm<'a>(
        &self, arms: &'a [SumMatchArm], branch: &SumBranchDecl,
    ) -> Result<&'a SumMatchArm, ParseError> {
        let mut matches = arms.iter().filter(|arm| arm.branch == branch.name);
        let arm = matches.next().ok_or_else(|| unsupported(branch.span))?;
        if let Some(duplicate) = matches.next() { return Err(unsupported(duplicate.span)); }
        Ok(arm)
    }

    fn reduce_call<'a>(
        &'a self, definition: &SeedDef, input: &'a SumBranchDecl, stack: &mut Vec<u64>, call_span: Span,
    ) -> Result<&'a SumBranchDecl, ParseError> {
        if stack.contains(&definition.id) || stack.len() >= 32 {
            return Err(failure(call_span, "E_PROOF_CARD_UNFOLD_CYCLE_OR_LIMIT",
                "닫힌 카드 전개에서 순환하거나 지원 깊이를 넘은 셈씨입니다"));
        }
        stack.push(definition.id);
        let body = definition.body.as_ref().expect("resolved card callable has a body");
        let parameter = &definition.params[0];
        let result = (|| {
            let result_body = if matches!(body.stmts.as_slice(), [Stmt::SumMatch { .. }]) {
                &self.select_arm(self.closed_arms(body, parameter)?, input)?.body
            } else {
                body
            };
            self.reduce_expr(named_result(result_body, definition)?, parameter, input, stack)
        })();
        stack.pop();
        result
    }

    fn reduce_expr<'a>(
        &'a self, expression: &Expr, parameter: &ParamPin, input: &'a SumBranchDecl, stack: &mut Vec<u64>,
    ) -> Result<&'a SumBranchDecl, ParseError> {
        match &expression.kind {
            ExprKind::Var(name) if name == &parameter.pin_name => Ok(input),
            ExprKind::SumBranchValue { owner, applied_owner: None, branch, payload }
                if owner == self.card_type && payload.is_empty() => self.branches.iter()
                    .find(|decl| decl.name == *branch).ok_or_else(|| unsupported(expression.span)),
            ExprKind::Call { func, args } => {
                let [arg] = args.as_slice() else { return Err(unsupported(expression.span)); };
                let Some(pin) = &arg.resolved_pin else { return Err(unsupported(arg.span)); };
                if arg.binding_reason != BindingReason::UserFixed || arg.josa.is_some() {
                    return Err(unsupported(arg.span));
                }
                if !matches!(&arg.expr.kind, ExprKind::Var(name) if name == &parameter.pin_name) {
                    return Err(unsupported(arg.expr.span));
                }
                let target = resolve_target(func, expression.span, &self.definitions, self.card_type)?;
                if target.params[0].pin_name != *pin || target.params[0].type_ref != parameter.type_ref {
                    return Err(unsupported(arg.span));
                }
                self.reduce_call(target, input, stack, expression.span)
            }
            _ => Err(unsupported(expression.span)),
        }
    }
}

fn named_result<'a>(body: &'a Body, definition: &SeedDef) -> Result<&'a Expr, ParseError> {
    let [Stmt::Mutate { target, value, deferred: false, .. }] = body.stmts.as_slice()
        else { return Err(unsupported(body.span)); };
    if !matches!(&target.kind, ExprKind::Var(name) if name == &definition.canonical_name) {
        return Err(unsupported(target.span));
    }
    Ok(value)
}

fn unsupported(span: Span) -> ParseError {
    failure(span, "E_PROOF_CARD_UNFOLD_UNSUPPORTED",
        "닫힌 카드 전개에는 받는자리·빈 갈래·명시 핀 전달 또는 닫힌 갈래 선택만 사용할 수 있습니다")
}
