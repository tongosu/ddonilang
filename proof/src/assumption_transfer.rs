use ddonirang_lang::{proof_authoring::{Assumption, Proposition, ResolvedTransfer, ResolvedEvidence}, ParseError};
use crate::{check_propositional_evidence, PropositionalGoal, PropositionalEvidence};

/// First integer identity lemma, derived from the owned definitions, not from
/// a caller assumption or a successful execution sample. No wire constructor.
#[derive(Debug, Clone)]
pub struct CheckedInitialIdentity { definition: ResolvedTransfer }

/// Static admission of one direct calculation definition. This is not a
/// runtime Proof and does not claim that a numeric invocation has executed.
#[derive(Debug, Clone)]
pub struct CheckedIdentityConsumption {
    law: CheckedInitialIdentity,
    definition: ResolvedTransfer,
}

fn direct_result(seed: &ddonirang_lang::ast::SeedDef) -> Option<&ddonirang_lang::ast::Expr> {
    use ddonirang_lang::ast::{Stmt, ExprKind};
    match seed.body.as_ref()?.stmts.as_slice() {
        [Stmt::Mutate { target, value, deferred: false, .. }]
            if matches!(&target.kind, ExprKind::Var(name) if name == &seed.canonical_name) => Some(value),
        _ => None,
    }
}

impl CheckedAssumptionTransfer {
    /// Independent definition replay. The existing conditional verdict is not
    /// used to issue this lemma; every formal/body/type binding is checked here.
    pub fn check_initial_identity(definition: &ResolvedTransfer) -> Result<CheckedInitialIdentity, ParseError> {
        use ddonirang_lang::ast::{ExprKind, SeedKind, TypeRef};
        let reject=||application_error(definition,"E_PROOF_IDENTITY_DEFINITION","최초 항등 근거에는 같은 바른수 입력의 실제 직접 반환 정의가 필요합니다");
        let law=definition.goal().structure_law().ok_or_else(reject)?;
        let op=law.operation();
        let generic=law.dependency();
        let integer=TypeRef::Named("바른수".into());
        if !law.belongs_to(definition.scope()) || law.domain()!=&integer || law.binder_indices()!=(0,0,0)
            || op.seed_kind!=SeedKind::Semssi || !op.modifiers.is_empty() || !op.type_params.is_empty()
            || op.params.len()!=1 || op.result_type.as_ref()!=Some(&integer) || op.proof_header.is_some()
            || generic.seed_kind!=SeedKind::Semssi || !generic.modifiers.is_empty()
            || generic.type_params.len()!=1 || generic.params.len()!=1 || generic.proof_header.is_some() {
            return Err(reject());
        }
        let input=&op.params[0];
        let formal=&generic.params[0];
        let t=TypeRef::Named(generic.type_params[0].name.clone());
        if input.type_ref!=integer || input.optional || input.default_value.is_some()
            || formal.optional || formal.default_value.is_some() || formal.type_ref!=t
            || generic.result_type.as_ref()!=Some(&t)
            || input.pin_name==op.canonical_name || input.pin_name==generic.canonical_name
            || formal.pin_name==generic.canonical_name {
            return Err(reject());
        }
        let body=direct_result(generic).ok_or_else(reject)?;
        let call=direct_result(op).ok_or_else(reject)?;
        let ExprKind::Call { args, .. }=&call.kind else {return Err(reject())};
        if !law.dependency_call_matches(call) || args.len()!=1
            || args[0].resolved_pin.as_deref()!=Some(formal.pin_name.as_str())
            || !matches!(&args[0].expr.kind,ExprKind::Var(name) if name==&input.pin_name)
            || !matches!(&body.kind,ExprKind::Var(name) if name==&formal.pin_name) {
            return Err(reject());
        }
        Ok(CheckedInitialIdentity { definition: definition.clone() })
    }
}

impl CheckedInitialIdentity {
    pub fn validate_source(&self, source: &str, path: &str) -> Result<(), ParseError> {
        if !self.definition.source_matches(source,path) {
            return Err(application_error(&self.definition,"E_PROOF_STALE_SOURCE","최초 항등 근거의 원문과 현재 원문이 다릅니다"));
        }
        CheckedAssumptionTransfer::check_initial_identity(&self.definition).map(|_| ())
    }

    pub fn admit_direct_calculation(&self, definition: &ResolvedTransfer) -> Result<CheckedIdentityConsumption, ParseError> {
        use ddonirang_lang::ast::ExprKind;
        let reject=||application_error(definition,"E_PROOF_STRUCTURE_CONSUMPTION","최초 항등 근거와 직접 계산의 구조·정의·값 핀이 다릅니다");
        // Preserve all existing conditional checks; a missing required premise
        // in that separate API never becomes an implicit success.
        check_assumption_transfer(definition)?;
        let law=self.definition.goal().structure_law().ok_or_else(reject)?;
        let consumption=definition.structure_consumption().ok_or_else(reject)?;
        if !law.same_structure(consumption.law()) || !consumption.member_matches()
            || definition.calculation_application().is_some() || definition.application().is_some()
            || definition.assumptions().len()!=1 || consumption.input().type_ref!=*law.domain()
            || !law.belongs_to(definition.scope()) {return Err(reject())}
        let ExprKind::Call{args,..}=&consumption.call().kind else {return Err(reject())};
        if args.len()!=1 || args[0].resolved_pin.as_deref()!=Some(law.operation().params[0].pin_name.as_str())
            || !matches!(&args[0].expr.kind,ExprKind::Var(name) if name==&consumption.input().pin_name) {return Err(reject())}
        Ok(CheckedIdentityConsumption {law:self.clone(),definition:definition.clone()})
    }
}

impl CheckedIdentityConsumption {
    pub fn definition(&self) -> &ResolvedTransfer { &self.definition }
    pub fn validate_source(&self, source: &str, path: &str) -> Result<(), ParseError> {
        self.law.validate_source(source,path)?;
        if !self.definition.source_matches(source,path) {
            return Err(application_error(&self.definition,"E_PROOF_STALE_SOURCE","정적 항등 소비의 원문과 현재 원문이 다릅니다"));
        }
        self.law.admit_direct_calculation(&self.definition).map(|_| ())
    }
}

fn instantiated_matches(formal: &Proposition, actual: &Proposition, substitution: &[(Proposition, Proposition)]) -> bool {
    match formal.conjunction_parts() {
        Some(parts) => actual.conjunction_parts().is_some_and(|actual_parts|
            parts.len() == actual_parts.len() && parts.iter().zip(actual_parts).all(|(f,a)| instantiated_matches(f,a,substitution))),
        None => substitution.iter().find(|(from, _)| from == formal).is_some_and(|(_, to)| to == actual),
    }
}

/// A checked conditional definition retaining its complete resolved context.
/// It does not issue a closed proof of P, and cannot be deserialized or built
/// by assigning a caller-provided success flag.
///
/// ```compile_fail
/// use ddonirang_proof::CheckedAssumptionTransfer;
/// use ddonirang_lang::proof_authoring::ResolvedTransfer;
/// fn forge(resolved: ResolvedTransfer) -> CheckedAssumptionTransfer {
///     CheckedAssumptionTransfer { resolved }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CheckedAssumptionTransfer { resolved: ResolvedTransfer }
impl CheckedAssumptionTransfer {
    pub fn resolved(&self) -> &ResolvedTransfer { &self.resolved }
    pub fn validate_source(&self, source: &str, path: &str) -> Result<(), ParseError> {
        if !self.resolved.source_matches(source, path) {
            return Err(ParseError { span: self.resolved.definition_span(),
                message: "E_PROOF_STALE_SOURCE: 검사한 원문과 현재 원문이 다릅니다".into() });
        }
        check_assumption_transfer(&self.resolved).map(|_| ())
    }
}

/// A checked application under the caller's explicit assumptions. This is not
/// a closed theorem or a serializable success receipt. Both source contexts and
/// the explicit substitution survive application; spelling is never identity.
#[derive(Debug, Clone)]
pub struct CheckedAssumptionApplication {
    definition: CheckedAssumptionTransfer,
    caller: ResolvedTransfer,
    substitution: Vec<(Proposition, Proposition)>,
    arguments: Vec<Assumption>,
}

impl CheckedAssumptionApplication {
    pub fn definition(&self) -> &CheckedAssumptionTransfer { &self.definition }
    pub fn caller(&self) -> &ResolvedTransfer { &self.caller }
    pub fn substitution(&self) -> &[(Proposition, Proposition)] { &self.substitution }
    pub fn arguments(&self) -> &[Assumption] { &self.arguments }
    pub fn goal(&self) -> &Proposition { self.caller.goal() }

    pub fn validate_sources(&self, definition_source: &str, definition_path: &str,
        caller_source: &str, caller_path: &str) -> Result<(), ParseError> {
        self.definition.validate_source(definition_source, definition_path)?;
        if !self.caller.source_matches(caller_source, caller_path) {
            return Err(application_error(&self.caller, "E_PROOF_STALE_SOURCE",
                "증거를 소비하는 정의의 원문이 변경되었습니다"));
        }
        self.definition.apply_in(&self.caller, &self.substitution, &self.arguments).map(|_| ())
    }
}

fn application_error(caller: &ResolvedTransfer, code: &str, message: &str) -> ParseError {
    ParseError { span: caller.result_span(), message: format!("{code}: {message}") }
}

impl CheckedAssumptionTransfer {
    /// Check an explicitly instantiated application. Source-level inference and
    /// argument ordering belong to the shared resolver, not this checker.
    /// The caller supplies a lexical context, not a claimed checker verdict.
    pub fn apply_in(&self, caller: &ResolvedTransfer,
        substitution: &[(Proposition, Proposition)], arguments: &[Assumption])
        -> Result<CheckedAssumptionApplication, ParseError> {
        self.apply_with_goal(caller, substitution, arguments, caller.goal())
    }

    fn apply_with_goal(&self, caller: &ResolvedTransfer,
        substitution: &[(Proposition, Proposition)], arguments: &[Assumption], goal: &Proposition)
        -> Result<CheckedAssumptionApplication, ParseError> {
        // This immutable handle is issued only by the checker; do not recurse
        // through its definition graph again for every application.
        let formal = &self.resolved;
        let reject = |code, message| application_error(caller, code, message);
        if formal.goal().coordinate_goal().is_some() {
            return Err(reject("E_PROOF_CALL_UNSUPPORTED", "좌표 첫 흐름은 직접 가정 전달과 직접 계산 소비만 지원합니다"));
        }
        if formal.structure_consumption().is_some() || formal.calculation_application().is_some() || formal.coordinate_calculation().is_some() {
            return Err(reject("E_PROOF_CALL_UNSUPPORTED", "계산 의무 검사를 보임 반환 함수로 사용할 수 없습니다"));
        }
        if arguments.len() != formal.assumptions().len() {
            return Err(reject("E_PROOF_ARGUMENT_COUNT", "모든 증거 인수를 정확히 전달해야 합니다"));
        }
        if caller.goal().scope() != caller.scope() || caller.assumptions().iter().any(|a|
            a.scope() != caller.scope() || a.goal().scope() != caller.scope()) {
            return Err(reject("E_PROOF_CONTEXT_MISMATCH", "호출자 가정 문맥이 일치하지 않습니다"));
        }
        for (index, (from, to)) in substitution.iter().enumerate() {
            if let Some(law)=from.structure_law() {
                if !to.structure_law().is_some_and(|actual|law.same_structure(actual)) {
                    return Err(reject("E_PROOF_SUBSTITUTION", "고정된 구조 법칙을 다른 목표로 치환할 수 없습니다"));
                }
            }
            if from.scope() != formal.scope() || to.scope() != caller.scope()
                || substitution[..index].iter().any(|(previous, _)| previous == from) {
                return Err(reject("E_PROOF_SUBSTITUTION", "명제 치환의 선언 문맥이 다르거나 중복됩니다"));
            }
        }
        if goal.scope() != caller.scope() || !instantiated_matches(formal.goal(), goal, substitution) {
            return Err(reject("E_PROOF_CONTEXT_MISMATCH", "적용한 증거의 목표 명제가 일치하지 않습니다"));
        }
        for (parameter, argument) in formal.assumptions().iter().zip(arguments) {
            if !caller.assumptions().iter().any(|a| a == argument)
                || argument.scope() != caller.scope()
                || !instantiated_matches(parameter.goal(), argument.goal(), substitution) {
                return Err(reject("E_PROOF_ASSUMPTION", "인수는 해당 명제의 실제 호출자 가정이어야 합니다"));
            }
        }
        Ok(CheckedAssumptionApplication { definition: self.clone(), caller: caller.clone(),
            substitution: substitution.to_vec(), arguments: arguments.to_vec() })
    }
}

pub fn check_assumption_transfer(resolved: &ResolvedTransfer) -> Result<CheckedAssumptionTransfer, ParseError> {
    if let Some(goal)=resolved.goal().coordinate_goal() {
        let reject=||application_error(resolved,"E_PROOF_COORDINATE_ASSUMPTIONS","좌표의 여덟 실제 가정과 최종 소비가 필요합니다");
        let mut members=std::collections::HashSet::new();
        if resolved.assumptions().len()!=8 || goal.index()!=7 {return Err(reject())}
        for assumption in resolved.assumptions() {
            let actual=assumption.goal().coordinate_goal().ok_or_else(reject)?;
            if !goal.same_structure(actual) || !members.insert(actual.index()) {return Err(reject())}
            check_propositional_evidence(resolved,&PropositionalGoal::from_proposition(assumption.goal()),
                &PropositionalEvidence::Assumption(assumption.clone()))?;
        }
        if let Some(call)=resolved.coordinate_calculation() {
            if !goal.calculation_matches(call) {return Err(reject())}
        }
    }
    if let Some(calculation)=resolved.calculation_application() {
        let checked=check_assumption_transfer(calculation.definition())?;
        checked.check_calculation_in(resolved)?;
    }
    if let Some(application) = resolved.application() {
        let checked = check_assumption_transfer(application.definition())?;
        checked.apply_with_goal(resolved, application.substitution(), application.arguments(), application.result_goal())?;
        let mut expected = application.instantiated_evidence()
            .ok_or_else(|| application_error(resolved, "E_PROOF_ASSUMPTION", "호출 증거 결속이 없습니다"))?;
        // The inherited projection is already part of the instantiated tree.
        for index in &resolved.projection()[application.definition().projection().len()..] {
            expected = ResolvedEvidence::Projection { evidence: Box::new(expected), index: *index };
        }
        if &expected != resolved.evidence() {
            return Err(application_error(resolved, "E_PROOF_CONTEXT_MISMATCH", "적용 결과의 증거 정체가 일치하지 않습니다"));
        }
    }
    let scope = resolved.scope();
    let accepted = resolved.goal().scope() == scope
        && resolved.projected_goal() == Some(resolved.goal())
        && resolved.assumptions().iter().all(|assumption|
            assumption.scope() == scope && assumption.goal().scope() == scope);
    if !accepted {
        return Err(ParseError { span: resolved.result_span(),
            message: "E_PROOF_CONTEXT_MISMATCH: 목표·가정·정의 문맥이 일치하지 않습니다".into() });
    }
    let evidence = checked_candidate(resolved, resolved.evidence(), 0, &mut 4096)?;
    check_propositional_evidence(resolved, &PropositionalGoal::from_proposition(resolved.goal()), &evidence)?;
    if let Some(consumption)=resolved.structure_consumption() {
        let law=resolved.goal().structure_law().ok_or_else(||application_error(resolved,
            "E_PROOF_STRUCTURE_CONSUMPTION","구조 법칙이 없는 소비 요청입니다"))?;
        if !law.same_structure(consumption.law()) || !consumption.member_matches()
            || consumption.input().type_ref!=*law.domain() {
            return Err(application_error(resolved,"E_PROOF_STRUCTURE_CONSUMPTION",
                "검사한 법칙의 구조·연산 멤버·입력 형식과 실제 소비가 다릅니다"));
        }
    }
    Ok(CheckedAssumptionTransfer { resolved: resolved.clone() })
}

impl CheckedAssumptionTransfer {
    /// Validate a separate numeric calculation application. This does not
    /// return a proof of the numeric result or a closed structure law.
    pub fn check_calculation_in(&self, caller: &ResolvedTransfer) -> Result<(), ParseError> {
        let reject=|message|application_error(caller,"E_PROOF_STRUCTURE_CONSUMPTION",message);
        let calculation=caller.calculation_application().ok_or_else(||reject("실제 계산 적용이 없습니다"))?;
        let formal=&self.resolved;
        if formal.scope()!=calculation.definition().scope() || formal.structure_consumption().is_none()
            || formal.calculation_application().is_some() || caller.structure_consumption().is_some()
            || caller.application().is_some() || !calculation.bindings_match() {
            return Err(reject("검사된 직접 계산 정의와 실제 적용 결속이 다릅니다"));
        }
        let argument=calculation.argument();
        let Some(law)=formal.goal().structure_law() else { return Err(reject("구조 법칙이 없는 계산입니다")) };
        if caller.assumptions().len()!=1 || !caller.assumptions().contains(argument)
            || argument.scope()!=caller.scope() || argument.goal()!=caller.goal()
            || !law.belongs_to(caller.scope())
            || !caller.goal().structure_law().is_some_and(|actual|law.same_structure(actual)) {
            return Err(reject("호출자의 실제 가정·구조·원문 문맥이 일치하지 않습니다"));
        }
        Ok(())
    }
}

fn checked_candidate(context: &ResolvedTransfer, term: &ResolvedEvidence, depth: usize,
    remaining: &mut usize) -> Result<PropositionalEvidence, ParseError> {
    if depth > 64 || *remaining == 0 { return Err(application_error(context, "E_PROOF_BUDGET", "증거 트리 검사 한도를 넘었습니다")); }
    *remaining -= 1;
    Ok(match term {
        ResolvedEvidence::Assumption(a) => PropositionalEvidence::Assumption(a.clone()),
        ResolvedEvidence::Projection { evidence, index } => PropositionalEvidence::Projection {
            evidence: Box::new(checked_candidate(context,evidence,depth+1,remaining)?), index: *index,
        },
        ResolvedEvidence::Conjunction { goal, parts } => {
            let candidate = PropositionalEvidence::Conjunction(parts.iter()
                .map(|p| checked_candidate(context,p,depth+1,remaining)).collect::<Result<_,_>>()?);
            check_propositional_evidence(context, &PropositionalGoal::from_proposition(goal), &candidate)?;
            candidate
        },
    })
}
