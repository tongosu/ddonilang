//! RD-309 ordered conjunction checking. This is a checker API, not a new
//! source syntax, serialized certificate profile, provider or closed theorem.
use ddonirang_lang::{proof_authoring::{Assumption, Proposition, ResolvedTransfer}, ParseError};

/// Candidate goal. Atomic identities come only from the shared source resolver.
/// Nested conjunctions are not flattened and operand order is significant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropositionalGoal {
    Atom(Proposition),
    Conjunction(Vec<PropositionalGoal>),
}
impl PropositionalGoal {
    pub fn from_proposition(p: &Proposition) -> Self {
        match p.conjunction_parts() {
            Some(parts) => Self::Conjunction(parts.iter().map(Self::from_proposition).collect()),
            None => Self::Atom(p.clone()),
        }
    }
}

/// Untrusted candidate derivation; constructing it does not issue evidence.
#[derive(Clone, Debug)]
pub enum PropositionalEvidence {
    Assumption(Assumption),
    Conjunction(Vec<PropositionalEvidence>),
    Projection { evidence: Box<PropositionalEvidence>, index: usize },
}

/// Only the checker constructs this conditional evidence. The entire source
/// context (including unused assumptions), derivation and goal are retained.
#[derive(Clone, Debug)]
pub struct CheckedPropositionalEvidence {
    context: ResolvedTransfer,
    goal: PropositionalGoal,
    evidence: PropositionalEvidence,
}

impl CheckedPropositionalEvidence {
    pub fn context(&self) -> &ResolvedTransfer { &self.context }
    pub fn goal(&self) -> &PropositionalGoal { &self.goal }
    pub fn evidence(&self) -> &PropositionalEvidence { &self.evidence }
    pub fn validate_source(&self, source: &str, path: &str) -> Result<(), ParseError> {
        if !self.context.source_matches(source, path) {
            return Err(error(&self.context, "E_PROOF_STALE_SOURCE", "검사한 원문 문맥이 변경되었습니다"));
        }
        check_propositional_evidence(&self.context, &self.goal, &self.evidence).map(|_| ())
    }
}

fn error(context: &ResolvedTransfer, code: &str, message: &str) -> ParseError {
    // This API has no authored term locations. Source integration must supply
    // term spans; this context span is not claimed as precise projection syntax.
    ParseError { span: context.result_span(), message: format!("{code}: {message}") }
}

struct Budget { remaining: usize }
impl Budget {
    fn visit(&mut self, context: &ResolvedTransfer, depth: usize) -> Result<(), ParseError> {
        if depth > 64 || self.remaining == 0 {
            return Err(error(context, "E_PROOF_BUDGET", "논리 증거 검사 한도를 넘었습니다"));
        }
        self.remaining -= 1;
        Ok(())
    }
}

fn validate_goal(context: &ResolvedTransfer, goal: &PropositionalGoal,
    depth: usize, budget: &mut Budget) -> Result<(), ParseError> {
    budget.visit(context, depth)?;
    match goal {
        PropositionalGoal::Atom(p) if p.scope() == context.scope() => {
            if let Some(goal) = p.coordinate_goal() {
                validate_coordinate_goal(context, goal)?;
            }
            if let Some(law) = p.structure_law() {
                use ddonirang_lang::TypeRef;
                let op=law.operation();
                let expected=TypeRef::Named("바른수".into());
                if !law.belongs_to(context.scope()) || law.domain()!=&expected
                    || law.binder_indices()!=(0,0,0) || op.params.len()!=1
                    || op.params[0].type_ref!=expected || op.result_type.as_ref()!=Some(&expected) {
                    return Err(error(context,"E_PROOF_STRUCTURE_TYPE","전칭 법칙의 문맥·묶인 변수·연산 형식이 일치하지 않습니다"));
                }
                let dependency=law.dependency();
                if dependency.type_params.len()!=1 || dependency.params.len()!=1 {
                    return Err(error(context,"E_PROOF_STRUCTURE_TYPE","법칙의 제네릭 정의 의존이 불완전합니다"));
                }
                let formal=TypeRef::Named(dependency.type_params[0].name.clone());
                if dependency.params[0].type_ref!=formal || dependency.result_type.as_ref()!=Some(&formal) {
                    return Err(error(context,"E_PROOF_STRUCTURE_TYPE","법칙의 입력·결과 씨 치환이 다릅니다"));
                }
            }
            Ok(())
        },
        PropositionalGoal::Atom(_) => Err(error(context, "E_PROOF_CONTEXT_MISMATCH", "다른 선언 문맥의 명제입니다")),
        PropositionalGoal::Conjunction(parts) => {
            if parts.len() < 2 {
                return Err(error(context, "E_PROOF_RULE_UNSUPPORTED", "이 검사는 둘 이상의 항을 가진 논리곱을 지원합니다"));
            }
            for part in parts { validate_goal(context, part, depth + 1, budget)?; }
            Ok(())
        }
    }
}

fn validate_coordinate_goal(context: &ResolvedTransfer,
    goal: &ddonirang_lang::proof_authoring::ResolvedCoordinateGoal) -> Result<(),ParseError> {
    use ddonirang_symbolic::typed_math::{Term as T, TermType, Goal, NumericRole};
    use ddonirang_lang::TypeRef;
    let fail=||error(context,"E_PROOF_COORDINATE_GOAL","좌표 목표의 형식·정의·발생·숫자 계약이 일치하지 않습니다");
    if !goal.belongs_to(context.scope()) || goal.index()>7 {return Err(fail())}
    let input=T::Pair(Box::new(T::Integer(1)),Box::new(T::Integer(0)));
    if goal.input()!=&input {return Err(fail())}
    let q=TypeRef::Tuple(vec![TypeRef::Named("바른수".into());2]);
    for definition in goal.definitions() {
        if definition.params.len()!=1 || definition.params[0].type_ref!=q || definition.result_type.as_ref()!=Some(&q) {return Err(fail())}
    }
    let select=|t,i|T::Select{target:Box::new(t),index:i};
    let param=|d|T::Parameter{definition:d,pin:0};
    let apply=|d,t|T::Apply{definition:d,argument:Box::new(t)};
    let expected=[T::Pair(Box::new(T::Subtract(Box::new(T::Integer(0)),Box::new(select(param(0),1)))),Box::new(select(param(0),0))),
        T::Pair(Box::new(T::Add(Box::new(select(param(1),0)),Box::new(T::Integer(10)))),Box::new(T::Add(Box::new(select(param(1),1)),Box::new(T::Integer(2))))),
        apply(1,apply(0,param(2)))];
    if goal.bodies()!=&expected || goal.bodies().iter().any(|body|
        body.term_type(&[(TermType::IntegerPair,TermType::IntegerPair);3])!=Some(TermType::IntegerPair)) {return Err(fail())}
    let rotated=apply(0,input.clone());
    let expected_goal=match goal.index() {
        0..=5 => {
            let term=match goal.index()%3 {
                0=>T::Subtract(Box::new(T::Integer(0)),Box::new(select(input.clone(),1))),
                1=>T::Add(Box::new(select(rotated.clone(),0)),Box::new(T::Integer(10))),
                _=>T::Add(Box::new(select(rotated.clone(),1)),Box::new(T::Integer(2))),
            };
            Goal::Defined{occurrence:goal.index(),term}
        },
        6=>Goal::Defined{occurrence:6,term:apply(2,input.clone())},
        _=>Goal::Equal{left:apply(2,input),right:apply(1,rotated)},
    };
    if goal.term()!=&expected_goal || goal.numeric_roles().len()!=3 {return Err(fail())}
    for (index,(expr,role)) in goal.numeric_roles().iter().enumerate() {
        let expected=if index==0 {NumericRole::CheckedDimensionlessIntegerSubtract}else{NumericRole::CheckedDimensionlessIntegerAdd};
        let ddonirang_lang::ExprKind::Infix{op,..}=&expr.kind else {return Err(fail())};
        if *role!=expected || op!=if index==0 {"-"}else{"+"} {return Err(fail())}
    }
    Ok(())
}

fn infer(context: &ResolvedTransfer, evidence: &PropositionalEvidence,
    depth: usize, budget: &mut Budget) -> Result<PropositionalGoal, ParseError> {
    budget.visit(context, depth)?;
    match evidence {
        PropositionalEvidence::Assumption(a) => {
            if a.scope() != context.scope() || a.goal().scope() != context.scope()
                || !context.assumptions().iter().any(|declared| declared == a) {
                return Err(error(context, "E_PROOF_ASSUMPTION", "현재 문맥의 실제 가정 증거가 아닙니다"));
            }
            Ok(PropositionalGoal::from_proposition(a.goal()))
        }
        PropositionalEvidence::Conjunction(parts) => {
            if parts.len() < 2 {
                return Err(error(context, "E_PROOF_RULE_UNSUPPORTED", "빈 증거나 단일 증거를 논리곱으로 발급하지 않습니다"));
            }
            let mut goals = Vec::new();
            for part in parts { goals.push(infer(context, part, depth + 1, budget)?); }
            Ok(PropositionalGoal::Conjunction(goals))
        }
        PropositionalEvidence::Projection { evidence, index } => {
            // Check the complete premise, including components not selected.
            // An invalid discarded component cannot be hidden by projection.
            let PropositionalGoal::Conjunction(parts) = infer(context, evidence, depth + 1, budget)? else {
                return Err(error(context, "E_PROOF_PROJECTION", "논리곱 증거에서만 항을 꺼낼 수 있습니다"));
            };
            parts.get(*index).cloned().ok_or_else(|| error(context,
                "E_PROOF_PROJECTION", "논리곱 증거의 항 범위를 벗어났습니다"))
        }
    }
}

/// Infer the derivation independently, then compare the requested goal. Goal
/// text, hashes and success flags cannot supply a missing premise. This checker
/// accepts resolved atomic/compound assumptions and complete derivation trees.
/// Source admission belongs to the shared resolver; errors here use its result span.
pub fn check_propositional_evidence(context: &ResolvedTransfer, goal: &PropositionalGoal,
    evidence: &PropositionalEvidence) -> Result<CheckedPropositionalEvidence, ParseError> {
    if context.assumptions().iter().any(|a| a.scope() != context.scope() || a.goal().scope() != context.scope()) {
        return Err(error(context, "E_PROOF_CONTEXT_MISMATCH", "선언된 가정 문맥이 일치하지 않습니다"));
    }
    let mut budget = Budget { remaining: 4096 };
    validate_goal(context, goal, 0, &mut budget)?;
    let actual = infer(context, evidence, 0, &mut budget)?;
    if &actual != goal {
        return Err(error(context, "E_PROOF_CONTEXT_MISMATCH", "증거의 순서·구조·목표 명제가 일치하지 않습니다"));
    }
    Ok(CheckedPropositionalEvidence { context: context.clone(), goal: actual, evidence: evidence.clone() })
}
