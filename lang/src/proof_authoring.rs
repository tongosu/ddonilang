//! Lexical resolution of the approved, conditional assumption-transfer source.
//! Handles are owned declarations in an immutable source context. Their
//! identity is not a spelling, position, NodeId, serialized ID or source hash.
use std::{collections::HashMap, sync::Arc};
use crate::{ast::*, ParseError, ParseMode};

#[derive(Debug)]
struct SourceContext { source: String, path: String }
#[derive(Debug)]
struct DefinitionScope { source: Arc<SourceContext> }
#[derive(Debug, Clone)]
pub struct Scope(Arc<DefinitionScope>);
impl PartialEq for Scope { fn eq(&self, other: &Self) -> bool { Arc::ptr_eq(&self.0, &other.0) } }
impl Eq for Scope {}
#[derive(Debug)]
struct PropositionDeclaration { scope: Scope, parts: Option<Vec<Proposition>>, law: Option<Arc<ResolvedStructureLaw>>, coordinate: Option<Arc<ResolvedCoordinateGoal>> }
#[derive(Debug, Clone)]
pub struct Proposition(Arc<PropositionDeclaration>);
impl PartialEq for Proposition { fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0) || (self.scope() == other.scope() && matches!(
        (&self.0.parts, &other.0.parts), (Some(a), Some(b)) if a == b))
} }
impl Eq for Proposition {}
impl Proposition {
    pub fn scope(&self) -> &Scope { &self.0.scope }
    pub fn coordinate_goal(&self) -> Option<&ResolvedCoordinateGoal> { self.0.coordinate.as_deref() }
    pub fn conjunction_parts(&self) -> Option<&[Proposition]> { self.0.parts.as_deref() }
    pub fn structure_law(&self) -> Option<&ResolvedStructureLaw> { self.0.law.as_deref() }
}
#[derive(Debug)]
struct AssumptionDeclaration { scope: Scope, goal: Proposition }
#[derive(Debug, Clone)]
pub struct Assumption(Arc<AssumptionDeclaration>);
impl PartialEq for Assumption { fn eq(&self, other: &Self) -> bool { Arc::ptr_eq(&self.0, &other.0) } }
impl Eq for Assumption {}
impl Assumption {
    pub fn scope(&self) -> &Scope { &self.0.scope }
    pub fn goal(&self) -> &Proposition { &self.0.goal }
}

/// Unforgeable resolution facts, not a checker success or an unconditional proof.
#[derive(Debug, Clone)]
pub struct ResolvedTransfer {
    scope: Scope,
    goal: Proposition,
    assumptions: Vec<Assumption>,
    evidence: ResolvedEvidence,
    definition_span: Span,
    result_span: Span,
    propositions: Vec<Proposition>,
    application: Option<ResolvedApplication>,
    application_depth: usize,
    projection: Vec<usize>,
    consumption: Option<ResolvedStructureConsumption>,
    calculation: Option<ResolvedCalculationApplication>,
    coordinate_calculation: Option<Expr>,
}
#[derive(Debug, Clone)]
pub struct ResolvedApplication {
    definition: Arc<ResolvedTransfer>,
    substitution: Vec<(Proposition, Proposition)>,
    arguments: Vec<Assumption>,
    result_goal: Proposition,
}
/// Source-resolved candidate evidence, never a checker verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedEvidence {
    Assumption(Assumption),
    Conjunction { goal: Proposition, parts: Vec<ResolvedEvidence> },
    Projection { evidence: Box<ResolvedEvidence>, index: usize },
}
impl ResolvedEvidence {
    pub fn goal(&self) -> Option<&Proposition> {
        match self {
            Self::Assumption(a) => Some(a.goal()),
            Self::Conjunction { goal, .. } => Some(goal),
            Self::Projection { evidence, index } => evidence.goal()?.conjunction_parts()?.get(*index),
        }
    }
    fn returned_assumption(&self) -> Option<&Assumption> {
        match self {
            Self::Assumption(a) => Some(a),
            Self::Projection { evidence, .. } => evidence.returned_assumption(),
            Self::Conjunction { .. } => None,
        }
    }
}
impl ResolvedApplication {
    pub fn definition(&self) -> &ResolvedTransfer { &self.definition }
    pub fn substitution(&self) -> &[(Proposition, Proposition)] { &self.substitution }
    pub fn arguments(&self) -> &[Assumption] { &self.arguments }
    pub fn result_goal(&self) -> &Proposition { &self.result_goal }
    pub fn instantiated_evidence(&self) -> Option<ResolvedEvidence> {
        instantiate_evidence(self.definition.evidence(), &self.definition, &self.arguments, &self.substitution)
    }
}
impl ResolvedTransfer {
    pub fn coordinate_calculation(&self) -> Option<&Expr> { self.coordinate_calculation.as_ref() }
    pub fn calculation_application(&self) -> Option<&ResolvedCalculationApplication> { self.calculation.as_ref() }
    pub fn structure_consumption(&self) -> Option<&ResolvedStructureConsumption> { self.consumption.as_ref() }
    pub fn application(&self) -> Option<&ResolvedApplication> { self.application.as_ref() }
    pub fn scope(&self) -> &Scope { &self.scope }
    pub fn goal(&self) -> &Proposition { &self.goal }
    pub fn assumptions(&self) -> &[Assumption] { &self.assumptions }
    pub fn returned(&self) -> Option<&Assumption> { self.evidence.returned_assumption() }
    pub fn evidence(&self) -> &ResolvedEvidence { &self.evidence }
    pub fn projection(&self) -> &[usize] { &self.projection }
    pub fn projected_goal(&self) -> Option<&Proposition> {
        self.evidence.goal()
    }
    pub fn definition_span(&self) -> Span { self.definition_span }
    pub fn result_span(&self) -> Span { self.result_span }
    pub fn source_matches(&self, source: &str, path: &str) -> bool {
        self.scope.0.source.source == source && self.scope.0.source.path == path
    }
}

fn failure(span: Span, code: &str, message: &str) -> ParseError {
    ParseError { span, message: format!("{code}: {message}") }
}

fn resolve_goal(kind: &TypeRef, declarations: &HashMap<String, Proposition>, span: Span) -> Result<Proposition, ParseError> {
    let TypeRef::Applied { name, args } = kind else {
        return Err(failure(span, "E_PROOF_TYPE", "명시된 명제의 보임 씨가 필요합니다"));
    };
    if name != "보임" || args.len() != 1 {
        return Err(failure(span, "E_PROOF_TYPE", "이 범위는 하나의 명제에 대한 보임 씨만 지원합니다"));
    }
    resolve_proposition(&args[0], declarations, span, 0)
}

fn resolve_proposition(kind: &TypeRef, declarations: &HashMap<String, Proposition>, span: Span, depth: usize) -> Result<Proposition, ParseError> {
    if depth > 64 { return Err(failure(span, "E_PROOF_BUDGET", "명제 중첩 검사 한도를 넘었습니다")); }
    match kind {
        TypeRef::Named(name) => declarations.get(name).cloned().ok_or_else(|| failure(span, "E_PROOF_GOAL", "선언되지 않은 정적 명제입니다")),
        TypeRef::PropositionConjunction(parts) if parts.len() >= 2 => {
            let parts: Vec<_> = parts.iter().map(|p| resolve_proposition(p, declarations, span, depth + 1)).collect::<Result<_,_>>()?;
            Ok(Proposition(Arc::new(PropositionDeclaration { scope: parts[0].scope().clone(), parts: Some(parts), law: None, coordinate: None })))
        },
        _ => Err(failure(span, "E_PROOF_GOAL", "정적 명제 또는 논리곱이 필요합니다")),
    }
}

fn infer_required(formal: &Proposition, actual: &Proposition, substitution: &mut Vec<(Proposition, Proposition)>, span: Span) -> Result<(), ParseError> {
    if let Some(parts) = formal.conjunction_parts() {
        let actual_parts = actual.conjunction_parts().filter(|a| a.len() == parts.len())
            .ok_or_else(|| failure(span, "E_GENERIC_UNIFY_CONFLICT", "필수 증거 입력의 논리곱 구조가 다릅니다"))?;
        for (f, a) in parts.iter().zip(actual_parts) { infer_required(f, a, substitution, span)?; }
    } else if let Some((_, previous)) = substitution.iter().find(|(from, _)| from == formal) {
        if previous != actual { return Err(failure(span, "E_GENERIC_UNIFY_CONFLICT", "필수 증거 입력의 명제 추론이 충돌합니다")); }
    } else { substitution.push((formal.clone(), actual.clone())); }
    Ok(())
}

fn instantiate_goal(goal: &Proposition, substitution: &[(Proposition, Proposition)]) -> Option<Proposition> {
    if let Some(parts) = goal.conjunction_parts() {
        let parts: Vec<_> = parts.iter().map(|p| instantiate_goal(p, substitution)).collect::<Option<_>>()?;
        Some(Proposition(Arc::new(PropositionDeclaration { scope: parts.first()?.scope().clone(), parts: Some(parts), law: None, coordinate: None })))
    } else { substitution.iter().find(|(from, _)| from == goal).map(|(_, to)| to.clone()) }
}

fn instantiate_evidence(term: &ResolvedEvidence, definition: &ResolvedTransfer,
    arguments: &[Assumption], substitution: &[(Proposition, Proposition)]) -> Option<ResolvedEvidence> {
    Some(match term {
        ResolvedEvidence::Assumption(a) => ResolvedEvidence::Assumption(arguments.get(
            definition.assumptions.iter().position(|formal| formal == a)?)?.clone()),
        ResolvedEvidence::Conjunction { goal, parts } => ResolvedEvidence::Conjunction {
            goal: instantiate_goal(goal, substitution)?,
            parts: parts.iter().map(|p| instantiate_evidence(p, definition, arguments, substitution)).collect::<Option<_>>()?,
        },
        ResolvedEvidence::Projection { evidence, index } => ResolvedEvidence::Projection {
            evidence: Box::new(instantiate_evidence(evidence, definition, arguments, substitution)?), index: *index,
        },
    })
}

fn resolve_direct_evidence(value: &Expr, expected: &Proposition, locals: &HashMap<String, Assumption>,
    depth: usize, remaining: &mut usize) -> Result<ResolvedEvidence, ParseError> {
    if depth > 64 || *remaining == 0 { return Err(failure(value.span, "E_PROOF_BUDGET", "증거 구성 검사 한도를 넘었습니다")); }
    *remaining -= 1;
    match &value.kind {
        ExprKind::TupleConstructor { elements } => {
            let goals = expected.conjunction_parts().filter(|parts| parts.len() == elements.len())
                .ok_or_else(|| failure(value.span, "E_PROOF_CONTEXT_MISMATCH", "직접 생성은 정확한 논리곱 목표와 항 수가 필요합니다"))?;
            let parts = elements.iter().zip(goals).map(|(e,g)| resolve_direct_evidence(e,g,locals,depth+1,remaining))
                .collect::<Result<_,_>>()?;
            Ok(ResolvedEvidence::Conjunction { goal: expected.clone(), parts })
        },
        _ => {
            let (base, indices) = selection_base(value)?;
            let ExprKind::Var(name) = &base.kind else {
                return Err(failure(value.span, "E_PROOF_EVIDENCE", "구성 항에는 명시 가정과 그 투영만 지원합니다"));
            };
            let mut term = ResolvedEvidence::Assumption(locals.get(name).cloned()
                .ok_or_else(|| failure(value.span, "E_PROOF_ASSUMPTION", "현재 문맥의 증거 가정이 아닙니다"))?);
            for index in indices { term = ResolvedEvidence::Projection { evidence: Box::new(term), index }; }
            // Type compatibility is a resolver fact. The checker still checks every premise.
            if term.goal() != Some(expected) { return Err(failure(value.span, "E_PROOF_CONTEXT_MISMATCH", "구성 항의 목표가 일치하지 않습니다")); }
            Ok(term)
        },
    }
}

fn selection_base(mut value: &Expr) -> Result<(&Expr, Vec<usize>), ParseError> {
    let mut indices = Vec::new();
    while let ExprKind::MultiAxisSelection(selection) = &value.kind {
        if selection.writable || selection.selectors.len() != 1 {
            return Err(failure(value.span, "E_PROOF_PROJECTION", "논리곱은 한 번에 하나의 정적 항을 선택해야 합니다"));
        }
        let TensorAxisSelector::Index(index) = &selection.selectors[0] else {
            return Err(failure(value.span, "E_PROOF_PROJECTION", "논리곱 투영에는 정적 정수 항 번호가 필요합니다"));
        };
        let n = index.numeric_literal_source.as_ref().filter(|literal| !literal.is_decimal())
            .and_then(|literal| literal.raw().parse::<usize>().ok())
            .ok_or_else(|| failure(index.span, "E_PROOF_PROJECTION", "논리곱 투영에는 음수가 아닌 정적 정수 원문이 필요합니다"))?;
        indices.push(n);
        if indices.len() > 64 { return Err(failure(value.span, "E_PROOF_BUDGET", "투영 깊이 한도를 넘었습니다")); }
        value = &selection.target;
    }
    indices.reverse();
    Ok((value, indices))
}

/// Uses the shared V25 parser on the exact input. No proof-specific tokenizer,
/// source rewrite, user-constructed AST or fabricated binding is admitted.
pub struct PreparedProofSource {
    program: CanonProgram,
    source: String,
    path: String,
    signature_owner: Arc<SourceContext>,
}

/// Own the single strict parse used by both numeric analysis and proof admission.
/// There is deliberately no constructor from a caller-created AST.
pub fn prepare_source(source: &str, path: &str) -> Result<PreparedProofSource, ParseError> {
    Ok(PreparedProofSource {
        program: crate::parse_v25_with_mode(source, path, ParseMode::Strict)?,
        source: source.into(), path: path.into(),
        signature_owner: Arc::new(SourceContext{source:source.into(),path:path.into()}),
    })
}

/// A compilation-local static declaration role, not proof or execution authority.
pub struct IdentityDeclarationRole<'a> { name: &'a str }
impl<'a> IdentityDeclarationRole<'a> { pub fn name(&self) -> &'a str { self.name } }
pub fn identity_declaration_role<'a>(program: &'a CanonProgram, statement: &'a Stmt)
    -> Option<IdentityDeclarationRole<'a>> {
    if !program.root_stmts.iter().any(|s|std::ptr::eq(s,statement)) { return None; }
    let Stmt::Definition{name,type_ref,value,..}=statement else {return None};
    if *type_ref!=TypeRef::Infer {return None;}
    let binding=program.origin.static_structure_calls.get(&value.id)?;
    if binding.contract!=StaticStructureContract::ExplicitUnaryIdentityV1
        || binding.domain!=TypeRef::Named("바른수".into()) {return None;}
    let ExprKind::Call{func,args}=&value.kind else {return None};
    if func!=binding.contract.authored_surface() || args.len()!=2 {return None;}
    let matches_origin=|expr:&Expr,origin:&Expr| expr.id==origin.id && expr.span==origin.span
        && matches!((&expr.kind,&origin.kind),(ExprKind::Var(a),ExprKind::Var(b)) if a==b);
    let domain=args.iter().find(|a|a.resolved_pin.as_deref()==Some("바탕씨"))?;
    let operation=args.iter().find(|a|a.resolved_pin.as_deref()==Some("연산"))?;
    if !matches_origin(&domain.expr,&binding.domain_origin)
        || !matches_origin(&operation.expr,&binding.operation_origin)
        || !program.items.iter().any(|TopLevelItem::SeedDef(s)|s.id==binding.operation.id) {return None;}
    Some(IdentityDeclarationRole{name})
}

impl PreparedProofSource {
    pub fn into_program(self) -> CanonProgram { self.program }

    /// First S4 execution candidate, from this actual parse only. Root output
    /// remains ordinary execution; its proof argument is a static law request.
    pub fn resolve_identity_execution(&self) -> Result<Vec<ResolvedIdentityExecution>, ParseError> {
        if !self.program.origin.static_structure_calls.values().any(|b|
            b.contract==StaticStructureContract::ExplicitUnaryIdentityV1) { return Ok(Vec::new()); }
        // Entry bodies are authored source, not generated root wrappers. Check
        // every admitted entry candidate before excluding it from proof definitions.
        let mut statements:Vec<&Stmt>=self.program.root_stmts.iter().collect();
        for TopLevelItem::SeedDef(entry) in &self.program.items {
            if entry.seed_kind!=SeedKind::Umjikssi {continue;}
            if !entry.params.is_empty() || !entry.type_params.is_empty() || !entry.modifiers.is_empty()
                || entry.result_type.is_some() || entry.body.is_none() {
                return Err(failure(entry.span,"E_PROOF_STRUCTURE_EXECUTION","첫 항등 시작은 인자와 결과씨가 없는 움직씨여야 합니다"));
            }
            let body=entry.body.as_ref().unwrap();
            if body.stmts.iter().any(|s|!matches!(s,Stmt::Expr{..}|Stmt::Show{..})) {
                return Err(failure(body.span,"E_PROOF_STRUCTURE_EXECUTION","첫 항등 시작 본문은 직접 계산 호출 출력만 지원합니다"));
            }
            statements.extend(body.stmts.iter());
        }
        let facts=resolve_structure_source_mode(&self.program,&self.source,&self.path,true)?;
        let mut out=Vec::new();
        for stmt in statements {
            let call=match stmt {
                Stmt::Expr{expr,..}|Stmt::Show{expr,..}=>expr,
                Stmt::Definition{..}=>continue,
                _=>return Err(failure(Span::new(0,self.source.len()),"E_PROOF_STRUCTURE_EXECUTION","첫 항등 실행은 직접 호출 출력만 지원합니다")),
            };
            let ExprKind::Call{args,..}=&call.kind else {return Err(failure(call.span,"E_PROOF_STRUCTURE_EXECUTION","검사할 직접 계산 호출이 필요합니다"))};
            let ids=self.program.origin.authored_call_candidates.get(&call.id)
                .ok_or_else(||failure(call.span,"E_PROOF_CALL_UNRESOLVED","실제 계산 정의가 없습니다"))?;
            let [id]=ids.as_slice() else {return Err(failure(call.span,"E_PROOF_CALL_AMBIGUOUS","유일한 계산 정의가 필요합니다"))};
            let seed=self.program.items.iter().map(|TopLevelItem::SeedDef(s)|s).find(|s|s.id==*id)
                .ok_or_else(||failure(call.span,"E_PROOF_CALL_UNRESOLVED","같은 원문의 계산 정의가 필요합니다"))?;
            let definition=facts.iter().find(|f|f.definition_span()==seed.span && f.structure_consumption().is_some())
                .ok_or_else(||failure(call.span,"E_PROOF_STRUCTURE_EXECUTION","직접 구조 소비 정의만 실행할 수 있습니다"))?;
            let consumption=definition.structure_consumption().unwrap();
            let proof_pin=seed.params.iter().find(|p|matches!(&p.type_ref,TypeRef::Applied{name,..} if name=="보임")).unwrap();
            let Stmt::Definition{name:structure,..}=consumption.law().declaration() else {unreachable!()};
            let mut value=None; let mut proof=false;
            for arg in args {
                if arg.resolved_pin.as_deref()==Some(consumption.input().pin_name.as_str()) && value.is_none() {
                    value=Some(arg.expr.clone());
                } else if arg.resolved_pin.as_deref()==Some(proof_pin.pin_name.as_str()) && !proof {
                    if !matches!(&arg.expr.kind,ExprKind::FieldAccess{target,field,..} if field=="법칙" && matches!(&target.kind,ExprKind::Var(n) if n==structure)) {
                        return Err(failure(arg.expr.span,"E_PROOF_STRUCTURE_CONSUMPTION","같은 구조의 정적 법칙 요청이 필요합니다"));
                    }
                    proof=true;
                } else {return Err(failure(arg.expr.span,"E_PROOF_STRUCTURE_CONSUMPTION","중복되거나 다른 값/법칙 핀입니다"))}
            }
            if args.len()!=2 || !proof || value.is_none() {return Err(failure(call.span,"E_PROOF_ASSUMPTION","실제 값과 정적 법칙 요청이 모두 필요합니다"))}
            out.push(ResolvedIdentityExecution{definition:definition.clone(),seed:seed.clone(),call:call.clone(),value:value.unwrap()});
        }
        Ok(out)
    }
    pub fn has_algebra_contract(&self)->bool {
        self.program.items.iter().any(|TopLevelItem::SeedDef(seed)|
            seed.result_type.as_ref().is_some_and(crate::algebra_authoring::is_goal))
    }
    pub fn resolve_algebra(&self)->Result<Vec<crate::algebra_authoring::ResolvedAlgebraDefinition>,ParseError> {
        crate::algebra_authoring::resolve(&self.program,&self.source,&self.path)
    }
    pub fn program(&self) -> &CanonProgram { &self.program }
    pub fn has_coordinate_contract(&self) -> bool {
        self.program.origin.static_structure_calls.values().any(|binding|
            binding.contract == StaticStructureContract::FixedCoordinateCompositionV1)
    }
    pub fn resolve_with_numeric_candidate(&self, candidate: &NumericSourceCandidate<'_>) -> Result<Vec<ResolvedTransfer>, ParseError> {
        if !std::ptr::eq(candidate.program, &self.program) {
            return Err(failure(Span::new(0,self.source.len()), "E_PROOF_NUMERIC_CONTEXT", "숫자 분석이 다른 파싱 문맥에 속합니다"));
        }
        if self.has_coordinate_contract() {
            resolve_coordinate_source(self, candidate)
        } else { self.resolve() }
    }
    pub fn resolve(&self) -> Result<Vec<ResolvedTransfer>, ParseError> {
        if self.has_coordinate_contract() {
            return Err(failure(Span::new(0,self.source.len()), "E_PROOF_NUMERIC_SUPPLY_REQUIRED", "좌표 검사는 같은 원문의 실제 숫자 분석 공급이 필요합니다"));
        }
        resolve_prepared_source(&self.program, &self.source, &self.path)
    }
}

pub fn resolve_assumption_source(source: &str, path: &str) -> Result<Vec<ResolvedTransfer>, ParseError> {
    prepare_source(source, path)?.resolve()
}

fn resolve_prepared_source(program: &CanonProgram, source: &str, path: &str) -> Result<Vec<ResolvedTransfer>, ParseError> {
    if program.items.iter().any(|TopLevelItem::SeedDef(seed)|
        seed.params.iter().any(|pin| contains_structure_law(&pin.type_ref))
        || seed.result_type.as_ref().is_some_and(contains_structure_law)) {
        return resolve_structure_source(&program, source, path);
    }
    if let Some(span) = program.origin.authored_jjaim_blocks.first() {
        return Err(failure(*span, "E_PROOF_ROOT_UNSUPPORTED", "이 보임 검사에서 미해석 짜임 블록을 무시할 수 없습니다"));
    }
    if !program.root_stmts.is_empty() {
        return Err(failure(Span::new(0, source.len()), "E_PROOF_ROOT_UNSUPPORTED", "보임 정의 검사에서 실행 문장을 무시할 수 없습니다"));
    }
    let context = Arc::new(SourceContext { source: source.into(), path: path.into() });
    let mut transfers = Vec::new();
    let mut definitions: HashMap<NodeId, Arc<ResolvedTransfer>> = HashMap::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        let Some(header) = &seed.proof_header else {
            return Err(failure(seed.span, "E_PROOF_DEFINITION", "이 검사는 명시적 정적 명제 가정 전달 정의만 지원합니다"));
        };
        if seed.seed_kind != SeedKind::Semssi || !seed.modifiers.is_empty() {
            return Err(failure(seed.span, "E_PROOF_DEFINITION", "가정 전달은 명시된 셈씨 정의여야 합니다"));
        }
        let scope = Scope(Arc::new(DefinitionScope { source: context.clone() }));
        let mut propositions = HashMap::new();
        for (name, span) in &header.propositions {
            if propositions.insert(name.clone(), Proposition(Arc::new(PropositionDeclaration { scope: scope.clone(), parts: None, law: None, coordinate: None }))).is_some() {
                return Err(failure(*span, "E_PROOF_BINDER_DUPLICATE", "정적 명제 이름이 중복되었습니다"));
            }
        }
        let result_type = seed.result_type.as_ref().ok_or_else(|| failure(seed.span, "E_PROOF_RESULT", "보임 결과씨가 필요합니다"))?;
        let goal = resolve_goal(result_type, &propositions, seed.span)?;
        let mut locals = HashMap::new();
        let mut public_pins = std::collections::HashSet::new();
        let mut assumptions = Vec::new();
        for (pin, binding) in seed.params.iter().zip(&header.evidence_bindings) {
            if pin.optional || pin.default_value.is_some() || binding.public_pin != pin.pin_name {
                return Err(failure(pin.span, "E_PROOF_ASSUMPTION", "증거는 명시적인 필수 인수여야 합니다"));
            }
            if binding.public_pin == "_" || binding.local_name == "_"
                || !public_pins.insert(&binding.public_pin) || binding.local_name == seed.canonical_name
                || propositions.contains_key(&binding.local_name) || locals.contains_key(&binding.local_name) {
                return Err(failure(binding.span, "E_PROOF_BINDER_DUPLICATE", "증거 변수·명제·결과 자리는 서로 구별되어야 합니다"));
            }
            let assumption = Assumption(Arc::new(AssumptionDeclaration {
                scope: scope.clone(), goal: resolve_goal(&pin.type_ref, &propositions, pin.span)?,
            }));
            locals.insert(binding.local_name.clone(), assumption.clone());
            assumptions.push(assumption);
        }
        if seed.params.len() != header.evidence_bindings.len() {
            return Err(failure(seed.span, "E_PROOF_ASSUMPTION", "공개 핀과 증거 변수의 결속이 누락되었습니다"));
        }
        let body = seed.body.as_ref().ok_or_else(|| failure(seed.span, "E_PROOF_RESULT", "증거 반환 본문이 필요합니다"))?;
        let [Stmt::Mutate { target, value, deferred: false, .. }] = body.stmts.as_slice() else {
            return Err(failure(body.span, "E_PROOF_RULE_UNSUPPORTED", "첫 보임은 이름 있는 결과에 가정 증거를 한 번 기록해야 합니다"));
        };
        if !matches!(&target.kind, ExprKind::Var(name) if name == &seed.canonical_name) {
            return Err(failure(target.span, "E_PROOF_RESULT", "이 정의의 결과 자리에 증거를 기록해야 합니다"));
        }
        let (base, suffix_projection) = selection_base(value)?;
        let mut projection = Vec::new();
        let (mut evidence, application, application_depth) = match &base.kind {
            ExprKind::TupleConstructor { .. } if suffix_projection.is_empty() =>
                (resolve_direct_evidence(base, &goal, &locals, 0, &mut 4096)?, None, 0),
            ExprKind::Var(name) => (ResolvedEvidence::Assumption(locals.get(name).cloned().ok_or_else(|| failure(value.span,
                "E_PROOF_ASSUMPTION", "현재 정의에 명시된 증거 가정이 아닙니다"))?), None, 0),
            ExprKind::Call { func, args } => {
                if locals.contains_key(func) || propositions.contains_key(func) {
                    return Err(failure(value.span, "E_PROOF_CALL_SHADOWED", "지역 선언에 가려진 호출은 지원하지 않습니다"));
                }
                let candidates = program.origin.authored_call_candidates.get(&base.id)
                    .ok_or_else(|| failure(value.span, "E_PROOF_CALL_UNRESOLVED", "실제 호출 정의가 해소되지 않았습니다"))?;
                let [definition_id] = candidates.as_slice() else {
                    return Err(failure(value.span, "E_PROOF_CALL_AMBIGUOUS", "증거 호출의 정의 후보가 하나여야 합니다"));
                };
                let definition = definitions.get(definition_id).cloned().ok_or_else(|| failure(value.span,
                    "E_PROOF_CALL_UNSUPPORTED", "현재 보임 호출은 앞서 작성한 비재귀 정의만 지원합니다"))?;
                if definition.application_depth >= 64 {
                    return Err(failure(value.span, "E_PROOF_BUDGET", "보임 호출 깊이 검사 한도를 넘었습니다"));
                }
                let callee = program.items.iter().map(|TopLevelItem::SeedDef(seed)| seed)
                    .find(|seed| seed.id == *definition_id).ok_or_else(|| failure(value.span,
                        "E_PROOF_CALL_UNRESOLVED", "호출 후보의 실제 선언이 없습니다"))?;
                if args.len() != definition.assumptions.len() {
                    return Err(failure(value.span, "E_PROOF_ARGUMENT_COUNT", "필수 증거 인수가 누락되거나 초과했습니다"));
                }
                let mut arguments = Vec::new();
                let mut substitution: Vec<(Proposition, Proposition)> = Vec::new();
                for ((arg, pin), formal) in args.iter().zip(&callee.params).zip(&definition.assumptions) {
                    if arg.resolved_pin.as_deref() != Some(pin.pin_name.as_str()) {
                        return Err(failure(arg.span, "E_PROOF_ARGUMENT_BINDING", "공통 해소자의 인수·핀 결속이 일치하지 않습니다"));
                    }
                    let ExprKind::Var(name) = &arg.expr.kind else {
                        return Err(failure(arg.expr.span, "E_PROOF_EVIDENCE", "명시된 가정 증거 인수만 지원합니다"));
                    };
                    let actual = locals.get(name).cloned().ok_or_else(|| failure(arg.expr.span,
                        "E_PROOF_ASSUMPTION", "호출자에 선언되지 않은 증거입니다"))?;
                    infer_required(formal.goal(), actual.goal(), &mut substitution, arg.expr.span)?;
                    arguments.push(actual);
                }
                if definition.propositions.iter().any(|p| !substitution.iter().any(|(from, _)| from == p)) {
                    return Err(failure(value.span, "E_PROOF_INFERENCE_UNRESOLVED", "필수 증거 입력만으로 정적 명제를 결정할 수 없습니다"));
                }
                let evidence = instantiate_evidence(&definition.evidence, &definition, &arguments, &substitution)
                    .ok_or_else(|| failure(value.span, "E_PROOF_ASSUMPTION", "호출 정의의 증거 결속이 없습니다"))?;
                let result_goal = instantiate_goal(definition.goal(), &substitution)
                    .ok_or_else(|| failure(value.span, "E_PROOF_INFERENCE_UNRESOLVED", "호출 목표의 명제가 미결입니다"))?;
                projection = definition.projection.clone();
                let depth = definition.application_depth + 1;
                (evidence, Some(ResolvedApplication { definition, substitution, arguments, result_goal }), depth)
            },
            _ => return Err(failure(value.span, "E_PROOF_EVIDENCE", "문자열·참거짓·검사 기록은 보임 증거가 아닙니다")),
        };
        for index in &suffix_projection { evidence = ResolvedEvidence::Projection { evidence: Box::new(evidence), index: *index }; }
        projection.extend(suffix_projection);
        let transfer = ResolvedTransfer { scope, goal, assumptions, evidence, definition_span: seed.span,
            result_span: value.span, propositions: header.propositions.iter().map(|(name, _)| propositions[name].clone()).collect(),
            application, application_depth, projection, consumption: None, calculation: None, coordinate_calculation: None };
        definitions.insert(seed.id, Arc::new(transfer.clone()));
        transfers.push(transfer);
    }
    if transfers.is_empty() {
        return Err(failure(Span::new(0, source.len()), "E_PROOF_DEFINITION", "검사할 가정 전달 정의가 없습니다"));
    }
    Ok(transfers)
}

fn contains_structure_law(kind: &TypeRef) -> bool {
    match kind {
        TypeRef::AlgebraEquality { .. } => false,
        TypeRef::StructureLaw { .. } => true,
        TypeRef::Applied { args, .. } | TypeRef::Tuple(args)
            | TypeRef::PropositionConjunction(args) => args.iter().any(contains_structure_law),
        TypeRef::Function { params, result } => params.iter().any(contains_structure_law)
            || contains_structure_law(result),
        TypeRef::Tensor { element, .. } => contains_structure_law(element),
        TypeRef::Named(_) | TypeRef::Infer => false,
    }
}


/// R3's source-owned carrier. No serialized IDs or external constructors.
#[derive(Debug)]
struct StructureOwner {
    contract: StaticStructureContract,
    source: Arc<SourceContext>,
    declaration: Stmt,
    descriptor: Stmt,
    operation: Arc<SeedDef>,
    dependency: Arc<SeedDef>,
    dependency_call: NodeId,
}
/// The admitted typed template is forall x:B. Eq_B(op(x),x).
/// Binder indices are local to this law, not source names or global IDs.
#[derive(Debug)]
pub struct ResolvedStructureLaw {
    owner: Arc<StructureOwner>,
    domain: TypeRef,
    binder: u32,
    application_argument: u32,
    equality_right: u32,
}
impl ResolvedStructureLaw {
    pub fn domain(&self) -> &TypeRef { &self.domain }
    pub fn operation(&self) -> &SeedDef { &self.owner.operation }
    pub fn dependency(&self) -> &SeedDef { &self.owner.dependency }
    /// The dependency was selected by the common resolver in this source owner.
    /// Call spelling is not the authority for this binding.
    pub fn dependency_call_matches(&self, call: &Expr) -> bool {
        call.id == self.owner.dependency_call
            && single_result(&self.owner.operation).is_ok_and(|actual| std::ptr::eq(actual, call))
    }
    pub fn binder_indices(&self) -> (u32,u32,u32) { (self.binder,self.application_argument,self.equality_right) }
    pub fn belongs_to(&self, scope: &Scope) -> bool { self.owner.contract == StaticStructureContract::ExplicitUnaryIdentityV1 && Arc::ptr_eq(&self.owner.source, &scope.0.source) }
    pub fn same_structure(&self, other: &Self) -> bool { Arc::ptr_eq(&self.owner,&other.owner) }
    pub fn declaration(&self) -> &Stmt { &self.owner.declaration }
    pub fn descriptor(&self) -> &Stmt { &self.owner.descriptor }
}
#[derive(Debug, Clone)]
pub struct ResolvedStructureConsumption {
    law: Arc<ResolvedStructureLaw>,
    operation: Arc<SeedDef>,
    call: Expr,
    input: ParamPin,
}
/// Compiler-owned candidate. A field reference requests a law check; it is
/// never an ordinary runtime value or a supplied success flag.
#[derive(Debug, Clone)]
pub struct ResolvedIdentityExecution {
    definition: ResolvedTransfer,
    seed: SeedDef,
    call: Expr,
    value: Expr,
}
impl ResolvedIdentityExecution {
    pub fn definition(&self)->&ResolvedTransfer { &self.definition }
    pub fn seed(&self)->&SeedDef { &self.seed }
    pub fn call(&self)->&Expr { &self.call }
    pub fn value(&self)->&Expr { &self.value }
}
/// A source-owned call of a direct conditional calculation, never proof return.
/// All fields are private; this is a candidate for the checker, not a receipt.
#[derive(Debug, Clone)]
pub struct ResolvedCalculationApplication {
    definition: Arc<ResolvedTransfer>,
    callee: Arc<SeedDef>,
    caller: SeedDef,
    call: Expr,
    input: ParamPin,
    proof_input: ParamPin,
    argument: Assumption,
}
impl ResolvedCalculationApplication {
    pub fn definition(&self) -> &ResolvedTransfer { &self.definition }
    pub fn argument(&self) -> &Assumption { &self.argument }
    pub fn call(&self) -> &Expr { &self.call }
    pub fn bindings_match(&self) -> bool {
        let Some(consumption) = self.definition.structure_consumption() else { return false };
        let ExprKind::Call { args, .. } = &self.call.kind else { return false };
        let Some(formal) = self.definition.assumptions().first() else { return false };
        let Some(actual_law) = self.argument.goal().structure_law() else { return false };
        if self.definition.assumptions().len()!=1 || self.definition.calculation_application().is_some()
            || self.definition.application().is_some() || !consumption.law().same_structure(actual_law)
            || self.callee.params.len()!=2 || args.len()!=2 || self.caller.params.len()!=2
            || self.callee.result_type.as_ref()!=Some(consumption.law().domain())
            || self.caller.result_type!=self.callee.result_type
            || self.input.type_ref!=*consumption.law().domain()
            || formal.goal().structure_law().is_none_or(|law| !law.same_structure(actual_law)) {
            return false;
        }
        // The resolver has selected the actual callee. Retain its precise pins,
        // rather than relabelling this call as the final structure operation.
        self.callee.params.iter().zip(args).all(|(pin,arg)| {
            if arg.resolved_pin.as_deref()!=Some(pin.pin_name.as_str()) { return false }
            let actual = if pin.pin_name==consumption.input().pin_name {
                if pin.type_ref!=self.input.type_ref { return false };
                &self.input
            } else {
                if !matches!(&pin.type_ref, TypeRef::Applied{name,..} if name=="보임") { return false };
                &self.proof_input
            };
            matches!(&arg.expr.kind, ExprKind::Var(name) if name==&actual.pin_name)
        })
    }
}
impl ResolvedStructureConsumption {
    pub fn law(&self) -> &ResolvedStructureLaw { &self.law }
    pub fn operation(&self) -> &SeedDef { &self.operation }
    pub fn call(&self) -> &Expr { &self.call }
    pub fn input(&self) -> &ParamPin { &self.input }
    pub fn member_matches(&self) -> bool { Arc::ptr_eq(&self.operation,&self.law.owner.operation) }
}

fn single_result(seed: &SeedDef) -> Result<&Expr,ParseError> {
    let body=seed.body.as_ref().ok_or_else(||failure(seed.span,"E_PROOF_RESULT","실제 본문이 필요합니다"))?;
    match body.stmts.as_slice() {
        [Stmt::Mutate{target,value,deferred:false,..}] if matches!(&target.kind,ExprKind::Var(n) if n==&seed.canonical_name) => Ok(value),
        _=>Err(failure(body.span,"E_PROOF_RULE_UNSUPPORTED","이 구조 법칙 흐름은 명명된 결과의 단일 전달만 지원합니다")),
    }
}
fn exact_fields<'a>(expr:&'a Expr, names:&[&str])->Result<HashMap<&'a str,&'a Expr>,ParseError> {
    let ExprKind::Pack{fields}=&expr.kind else {return Err(failure(expr.span,"E_PROOF_STRUCTURE_DESCRIPTOR","명시된 정적 필드 묶음이 필요합니다"))};
    let mut out=HashMap::new();
    for (name,value) in fields {
        if !names.contains(&name.as_str()) || out.insert(name.as_str(),value).is_some() {
            return Err(failure(value.span,"E_PROOF_STRUCTURE_DESCRIPTOR","중복되거나 지원되지 않는 차림새 필드입니다"));
        }
    }
    if out.len()!=names.len() {return Err(failure(expr.span,"E_PROOF_STRUCTURE_DESCRIPTOR","필수 차림새 필드가 누락되었습니다"))}
    Ok(out)
}
fn expect_text(expr:&Expr,text:&str)->Result<(),ParseError> {
    if matches!(&expr.kind,ExprKind::Literal(Literal::String(s)) if s==text) {Ok(())}
    else {Err(failure(expr.span,"E_PROOF_STRUCTURE_ADMISSION","채택된 제공자·차림새·법칙과 다릅니다"))}
}

fn resolve_structure_declaration<'a>(program:&CanonProgram, context:&Arc<SourceContext>, stmt:&'a Stmt,
    value:&'a Expr, callable_env:&HashMap<&str,Arc<SeedDef>>,
    descriptors:&HashMap<&str,(&'a Stmt,&'a Expr)>, used_descriptors:&mut std::collections::HashSet<&'a str>)
    ->Result<Arc<ResolvedStructureLaw>,ParseError> {
    let ExprKind::Call{func,args}=&value.kind else {return Err(failure(value.span,"E_PROOF_STRUCTURE_DESCRIPTOR","정적 차림새로 생성한 구조 정의가 필요합니다"))};
        let (descriptor, _descriptor_value, op) = if let Some(binding) = program.origin.static_structure_calls.get(&value.id) {
            if binding.contract != StaticStructureContract::ExplicitUnaryIdentityV1
                || binding.domain != TypeRef::Named("바른수".into()) {
                return Err(failure(binding.domain_origin.span,"E_PROOF_STRUCTURE_ADMISSION","채택된 정적 도우미 판1 결속이 필요합니다"));
            }
            let op = callable_env.values().find(|op| op.id == binding.operation.id).cloned()
                .ok_or_else(||failure(binding.operation_origin.span,"E_PROOF_STRUCTURE_MEMBER","공통 분석이 해소한 실제 연산 정의가 없습니다"))?;
            // The source-owned definition is retained, not a synthetic descriptor
            // with fabricated source spans. The contract fixes all R3 V1 axes.
            (stmt, value, op)
        } else {
        if func!="짜임" || args.len()!=1 || args[0].resolved_pin.as_deref()!=Some("차림새") {
            return Err(failure(value.span,"E_PROOF_STRUCTURE_DESCRIPTOR","명시된 차림새 받는자리 하나가 필요합니다"));
        }
        let ExprKind::Var(descriptor_name)=&args[0].expr.kind else {return Err(failure(args[0].span,"E_PROOF_STRUCTURE_DESCRIPTOR","정적 차림새 정의 참조가 필요합니다"))};
        let (descriptor,descriptor_value)=descriptors.get(descriptor_name.as_str()).ok_or_else(||failure(args[0].span,"E_PROOF_STRUCTURE_DESCRIPTOR","앞서 해소된 차림새 정의가 아닙니다"))?;
        used_descriptors.insert(descriptor_name.as_str());
        let fields=exact_fields(descriptor_value,&["제공자","제공자판","차림새","차림새판","바탕","구조","조각","능력","검사","투영"])?;
        expect_text(fields["제공자"],"ddn.structure.explicit-unary-law-transfer")?;
        expect_text(fields["차림새"],"ddn.structure.unary-identity-law-assumption")?;
        for key in ["제공자판","차림새판"] {
            let integer_literal=fields[key].numeric_literal_source.as_ref().is_some_and(|n|!n.is_decimal());
            let one=match &fields[key].kind { ExprKind::Literal(Literal::Int(v))=>*v==1,
                ExprKind::Literal(Literal::Fixed64(v))=>*v==ddonirang_core::Fixed64::ONE,_=>false };
            if !integer_literal || !one {return Err(failure(fields[key].span,"E_PROOF_STRUCTURE_ADMISSION","채택된 판은 정수 1입니다"))}
        }
        for key in ["바탕","조각","능력","투영"] {if !matches!(&fields[key].kind,ExprKind::TupleConstructor{elements} if elements.is_empty()) {return Err(failure(fields[key].span,"E_PROOF_STRUCTURE_DESCRIPTOR","이 차림새 축은 명시 빈 값꾸러미만 지원합니다"))}}
        let validation=exact_fields(fields["검사"],&["요구법칙"])?;expect_text(validation["요구법칙"],"단항항등")?;
        let structure=exact_fields(fields["구조"],&["바탕씨","연산"])?;
        if !matches!(&structure["바탕씨"].kind,ExprKind::Var(n) if n=="바른수") {return Err(failure(structure["바탕씨"].span,"E_PROOF_STRUCTURE_TYPE","가려지지 않은 기본 바른수 형식이 필요합니다"))}
        let ExprKind::Var(op_name)=&structure["연산"].kind else {return Err(failure(structure["연산"].span,"E_PROOF_STRUCTURE_MEMBER","실제 연산 정의 참조가 필요합니다"))};
        let op=callable_env.get(op_name.as_str()).filter(|op|op.span.end<descriptor_value.span.start).cloned()
            .ok_or_else(||failure(structure["연산"].span,"E_PROOF_STRUCTURE_MEMBER","차림새 앞의 실제 연산 정의가 아닙니다"))?;
            (*descriptor, *descriptor_value, op)
        };
        let integer=TypeRef::Named("바른수".into());
        if op.seed_kind!=SeedKind::Semssi || !op.modifiers.is_empty() || !op.type_params.is_empty() || op.params.len()!=1
            || op.params[0].pin_name==op.canonical_name || op.params[0].optional || op.params[0].default_value.is_some() || op.params[0].type_ref!=integer || op.result_type.as_ref()!=Some(&integer) {
            return Err(failure(op.span,"E_PROOF_STRUCTURE_TYPE","연산은 명시 필수 바른수 입력과 바른수 결과를 가져야 합니다"));
        }
        let call=single_result(&op)?;
        let ExprKind::Call{args,..}=&call.kind else {return Err(failure(call.span,"E_PROOF_STRUCTURE_POLICY","허용된 제네릭 직접 전달 호출 본문이 필요합니다"))};
        let ids=program.origin.authored_call_candidates.get(&call.id).ok_or_else(||failure(call.span,"E_PROOF_CALL_UNRESOLVED","공통 호출 해소 결과가 없습니다"))?;
        let [id]=ids.as_slice() else {return Err(failure(call.span,"E_PROOF_CALL_AMBIGUOUS","호출 정의가 유일하지 않습니다"))};
        let generic=callable_env.values().find(|d|d.id==*id).cloned().ok_or_else(||failure(call.span,"E_PROOF_CALL_UNRESOLVED","실제 선언 참조가 없습니다"))?;
        if generic.type_params.len()!=1 || generic.params.len()!=1 || generic.seed_kind!=SeedKind::Semssi || !generic.modifiers.is_empty()
            || generic.params[0].pin_name==generic.canonical_name || op.params[0].pin_name==generic.canonical_name
            || generic.params[0].optional || generic.params[0].default_value.is_some() || generic.proof_header.is_some() {
            return Err(failure(generic.span,"E_PROOF_STRUCTURE_POLICY","필수 단항 제네릭 직접 전달 정의만 참여할 수 있습니다"));
        }
        let t=TypeRef::Named(generic.type_params[0].name.clone());
        let body=single_result(&generic)?;
        if generic.params[0].type_ref!=t || generic.result_type.as_ref()!=Some(&t)
            || !matches!(&body.kind,ExprKind::Var(n) if n==&generic.params[0].pin_name)
            || args.len()!=1 || args[0].resolved_pin.as_deref()!=Some(generic.params[0].pin_name.as_str())
            || !matches!(&args[0].expr.kind,ExprKind::Var(n) if n==&op.params[0].pin_name) {
            return Err(failure(call.span,"E_PROOF_STRUCTURE_POLICY","실제 입력·출력·제네릭 치환 결속이 직접 전달 정책과 다릅니다"));
        }

    let dependency_call=call.id;
    let owner=Arc::new(StructureOwner{contract:StaticStructureContract::ExplicitUnaryIdentityV1,source:context.clone(),declaration:stmt.clone(),descriptor:descriptor.clone(),operation:op,dependency:generic,dependency_call});
    Ok(Arc::new(ResolvedStructureLaw{owner,domain:integer,binder:0,application_argument:0,equality_right:0}))
}

fn resolve_structure_source(program:&CanonProgram,source:&str,path:&str)->Result<Vec<ResolvedTransfer>,ParseError> {
    resolve_structure_source_mode(program,source,path,false)
}
fn resolve_structure_source_mode(program:&CanonProgram,source:&str,path:&str,execution:bool)->Result<Vec<ResolvedTransfer>,ParseError> {
    if let Some(span)=program.origin.authored_jjaim_blocks.first() {
        return Err(failure(*span,"E_PROOF_ROOT_UNSUPPORTED","미해석 짜임 블록을 검사에서 생략할 수 없습니다"));
    }
    let context=Arc::new(SourceContext{source:source.into(),path:path.into()});
    // This environment owns actual declarations. Spelling only selects a
    // lexical binder; pointer-owned declarations are the resulting references.
    let mut callable_env:HashMap<&str,Arc<SeedDef>>=HashMap::new();
    for TopLevelItem::SeedDef(seed) in &program.items {
        if matches!(seed.canonical_name.as_str(),"바른수"|"짜임") || callable_env.insert(&seed.canonical_name,Arc::new(seed.clone())).is_some() {
            return Err(failure(seed.span,"E_PROOF_BINDER_DUPLICATE","형식·정의의 소유자가 중복되거나 기본 형식을 가립니다"));
        }
    }
    let mut descriptors:HashMap<&str,(&Stmt,&Expr)>=HashMap::new();
    let mut laws:HashMap<&str,Arc<ResolvedStructureLaw>>=HashMap::new();
    let mut used=std::collections::HashSet::new();
    let mut names=std::collections::HashSet::new();
    let mut used_descriptors=std::collections::HashSet::new();
    for stmt in &program.root_stmts {
        if execution && matches!(stmt,Stmt::Expr{..}|Stmt::Show{..}) {continue}
        let Stmt::Definition{name,value,span,type_ref,reaction:None,..}=stmt else {
            let span=match stmt {Stmt::Mutate{span,..}|Stmt::Expr{span,..}|Stmt::Definition{span,..}
                |Stmt::Show{span,..}|Stmt::DerivedBinding{span,..}|Stmt::SumDefinition{span,..}=>*span,
                _=>Span::new(0,source.len())};
            return Err(failure(span,"E_PROOF_ROOT_UNSUPPORTED","정적 구조·차림새 정의 외의 실행 문장을 생략할 수 없습니다"));
        };
        if matches!(name.as_str(),"바른수"|"짜임") || callable_env.contains_key(name.as_str()) || !names.insert(name.as_str()) {
            return Err(failure(*span,"E_PROOF_BINDER_DUPLICATE","구조·차림새·함수의 정적 정의가 충돌합니다"));
        }
        if !matches!(type_ref,TypeRef::Infer) {return Err(failure(*span,"E_PROOF_STRUCTURE_TYPE","이 정적 구조 흐름은 원문 descriptor와 구조의 추론된 씨만 지원합니다"))}
        if matches!(&value.kind,ExprKind::Pack{..}) {descriptors.insert(name,(stmt,value));continue}
        let ExprKind::Call{..}=&value.kind else {return Err(failure(value.span,"E_PROOF_STRUCTURE_DESCRIPTOR","정적 차림새로 생성한 구조 정의가 필요합니다"))};
        let law=resolve_structure_declaration(program,&context,stmt,value,&callable_env,&descriptors,&mut used_descriptors)?;
        used.insert(law.owner.operation.id);used.insert(law.owner.dependency.id);
        laws.insert(name,law);
    }
    if let Some((_,(_,expr)))=descriptors.iter().find(|(name,_)|!used_descriptors.contains(**name)) {
        return Err(failure(expr.span,"E_PROOF_STRUCTURE_DESCRIPTOR","사용하지 않은 정적 묶음을 검사에서 생략할 수 없습니다"));
    }
    let mut out=Vec::new();
    let mut completed:HashMap<NodeId,Arc<ResolvedTransfer>>=HashMap::new();
    for TopLevelItem::SeedDef(seed) in &program.items {
        if used.contains(&seed.id) {continue}
        // Execution-only entry candidates were checked by resolve_identity_execution.
        // Conditional proof verification does not omit these definitions.
        if execution && seed.seed_kind==SeedKind::Umjikssi {continue}
        if seed.seed_kind!=SeedKind::Semssi || !seed.modifiers.is_empty() || !seed.type_params.is_empty() {
            return Err(failure(seed.span,"E_PROOF_DEFINITION","구조 법칙 흐름의 조건부 셈씨가 아닙니다"));
        }
        let mut pin_names=std::collections::HashSet::new();
        if seed.params.iter().any(|p|p.pin_name==seed.canonical_name || !pin_names.insert(&p.pin_name)) {
            return Err(failure(seed.span,"E_PROOF_BINDER_DUPLICATE","입력과 결과의 실제 선언이 충돌합니다"));
        }
        let scope=Scope(Arc::new(DefinitionScope{source:context.clone()}));
        let mut assumptions=Vec::new();let mut proof_inputs=Vec::new();let mut numeric_inputs=Vec::new();
        let mut scoped_goals:HashMap<&str,Proposition>=HashMap::new();
        for pin in &seed.params {
            if pin.optional || pin.default_value.is_some() {return Err(failure(pin.span,"E_PROOF_ASSUMPTION","모든 입력은 명시 필수 입력이어야 합니다"))}
            if let TypeRef::Applied{name,args}=&pin.type_ref {
                if name=="보임" {
                    let [TypeRef::StructureLaw{structure,member}]=args.as_slice() else {return Err(failure(pin.span,"E_PROOF_GOAL","정확한 구조 법칙 목표가 필요합니다"))};
                    if member!="법칙" || seed.params.iter().any(|p|p.pin_name==*structure) {return Err(failure(pin.span,"E_PROOF_GOAL","가려지거나 미지원인 구조 법칙 목표입니다"))}
                    let law=laws.get(structure.as_str()).filter(|law| match &law.owner.declaration {Stmt::Definition{span,..}=>span.end<seed.span.start,_=>false})
                        .ok_or_else(||failure(pin.span,"E_PROOF_GOAL","현재 위치에서 해소된 구조 정의가 없습니다"))?;
                    let goal=scoped_goals.entry(structure).or_insert_with(||Proposition(Arc::new(PropositionDeclaration{scope:scope.clone(),parts:None,law:Some(law.clone()),coordinate:None}))).clone();
                    let assumption=Assumption(Arc::new(AssumptionDeclaration{scope:scope.clone(),goal}));
                    proof_inputs.push((pin,assumption.clone()));assumptions.push(assumption);continue;
                }
            }
            if pin.type_ref!=TypeRef::Named("바른수".into()) {return Err(failure(pin.span,"E_PROOF_STRUCTURE_TYPE","계산 입력은 바른수여야 합니다"))}
            numeric_inputs.push(pin);
        }
        let [(proof_pin,assumption)]=proof_inputs.as_slice() else {return Err(failure(seed.span,"E_PROOF_ASSUMPTION","유일하게 대응하는 필수 구조 법칙 가정이 필요합니다"))};
        let value=single_result(seed)?;
        let law=assumption.goal().0.law.as_ref().unwrap().clone();
        let mut application=None;
        let mut application_depth=0;
        let mut calculation=None;
        let consumption=if let Some(TypeRef::Applied{name,args})=&seed.result_type {
            if name!="보임" || !numeric_inputs.is_empty() || !matches!(args.as_slice(),[TypeRef::StructureLaw{structure,member}] if member=="법칙" && scoped_goals.get(structure.as_str())==Some(assumption.goal()))
                {
                return Err(failure(value.span,"E_PROOF_CONTEXT_MISMATCH","가정과 반환 법칙·증거가 일치하지 않습니다"));
            }
            match &value.kind {
                ExprKind::Var(n) if n==&proof_pin.pin_name => (),
                ExprKind::Call{args,..} => {
                    let ids=program.origin.authored_call_candidates.get(&value.id).ok_or_else(||failure(value.span,"E_PROOF_CALL_UNRESOLVED","실제 보임 호출 정의가 없습니다"))?;
                    let [id]=ids.as_slice() else {return Err(failure(value.span,"E_PROOF_CALL_AMBIGUOUS","호출 정의가 유일하지 않습니다"))};
                    let definition=completed.get(id).cloned().ok_or_else(||failure(value.span,"E_PROOF_CALL_UNSUPPORTED","앞서 검사한 비재귀 보임 정의가 필요합니다"))?;
                    let callee=callable_env.values().find(|d|d.id==*id).unwrap();
                    if definition.structure_consumption().is_some() || definition.calculation_application().is_some() || definition.assumptions.len()!=1 || callee.params.len()!=1
                        || seed.params.iter().any(|p|p.pin_name==callee.canonical_name)
                        || args.len()!=1 || args[0].resolved_pin.as_deref()!=Some(callee.params[0].pin_name.as_str())
                        || !matches!(&args[0].expr.kind,ExprKind::Var(n) if n==&proof_pin.pin_name)
                        || !definition.goal().structure_law().is_some_and(|formal|formal.same_structure(&law)) {
                        return Err(failure(value.span,"E_PROOF_CONTEXT_MISMATCH","호출의 실제 가정·법칙·핀 결속이 다릅니다"));
                    }
                    application_depth=definition.application_depth+1;
                    if application_depth>64 {return Err(failure(value.span,"E_PROOF_BUDGET","보임 호출 깊이 한도를 넘었습니다"))}
                    application=Some(ResolvedApplication{ substitution:vec![(definition.goal().clone(),assumption.goal().clone())],
                        definition,arguments:vec![assumption.clone()],result_goal:assumption.goal().clone() });
                },
                _=>return Err(failure(value.span,"E_PROOF_EVIDENCE","실제 가정 또는 검사된 보임 호출이 필요합니다")),
            }
            None
        }else{
            let [input]=numeric_inputs.as_slice() else {return Err(failure(seed.span,"E_PROOF_STRUCTURE_TYPE","계산은 바른수 입력 하나가 필요합니다"))};
            let ExprKind::Call{args,..}=&value.kind else {return Err(failure(value.span,"E_PROOF_RULE_UNSUPPORTED","실제 연산 호출 하나가 필요합니다"))};
            let ids=program.origin.authored_call_candidates.get(&value.id)
                .ok_or_else(||failure(value.span,"E_PROOF_CALL_UNRESOLVED","실제 계산 호출 정의가 없습니다"))?;
            let [id]=ids.as_slice() else {return Err(failure(value.span,"E_PROOF_CALL_AMBIGUOUS","계산 호출 정의가 유일하지 않습니다"))};
            if *id==law.owner.operation.id {
                if seed.result_type.as_ref()!=Some(&law.domain) || args.len()!=1
                    || !matches!(&args[0].expr.kind,ExprKind::Var(n) if n==&input.pin_name)
                    || args[0].resolved_pin.as_deref()!=Some(law.owner.operation.params[0].pin_name.as_str()) {
                    return Err(failure(value.span,"E_PROOF_STRUCTURE_CONSUMPTION","증거의 구조 멤버와 실제 호출·입력·결과 형식이 다릅니다"));
                }
                Some(ResolvedStructureConsumption{law:law.clone(),operation:law.owner.operation.clone(),call:value.clone(),input:(*input).clone()})
            } else {
                let definition=completed.get(id).cloned().ok_or_else(||failure(value.span,
                    "E_PROOF_STRUCTURE_CONSUMPTION","앞서 검사한 직접 계산 정의가 필요합니다"))?;
                let callee=callable_env.values().find(|d|d.id==*id).cloned()
                    .ok_or_else(||failure(value.span,"E_PROOF_CALL_UNRESOLVED","실제 계산 선언이 없습니다"))?;
                if seed.params.iter().any(|pin|pin.pin_name==callee.canonical_name) {
                    return Err(failure(value.span,"E_PROOF_CALL_SHADOWED","입력에 가려진 계산 호출입니다"));
                }
                let candidate=ResolvedCalculationApplication{definition,callee,caller:seed.clone(),call:value.clone(),
                    input:(*input).clone(),proof_input:(*proof_pin).clone(),argument:assumption.clone()};
                if !candidate.bindings_match() {
                    return Err(failure(value.span,"E_PROOF_STRUCTURE_CONSUMPTION","계산 호출의 구조·값 입력·필수 가정·결과 결속이 다릅니다"));
                }
                calculation=Some(candidate);
                None
            }
        };
        let transfer=ResolvedTransfer{scope,goal:assumption.goal().clone(),assumptions,evidence:ResolvedEvidence::Assumption(assumption.clone()),
            definition_span:seed.span,result_span:value.span,propositions:Vec::new(),application,application_depth,projection:Vec::new(),consumption,calculation,coordinate_calculation:None};
        completed.insert(seed.id,Arc::new(transfer.clone()));out.push(transfer);
    }
    if out.is_empty() {return Err(failure(Span::new(0,source.len()),"E_PROOF_DEFINITION","검사할 조건부 정의가 없습니다"))}
    Ok(out)
}


use ddonirang_symbolic::typed_math::{Term, TermType, Goal as MathGoal, NumericRole};

/// Public untrusted facts. Neither constructing these nor obtaining a reference
/// to the parsed program grants coordinate admission or checker success.
pub struct NumericSourceCandidate<'a> {
    pub program: &'a CanonProgram,
    pub expressions: Vec<(NodeId, TypeRef)>,
    pub operations: Vec<(NodeId, NumericRole)>,
}

#[derive(Debug)]
struct CoordinateOwner {
    source: Arc<SourceContext>,
    definitions: [SeedDef;3],
    bodies: [Term;3],
    input: Term,
    goals: Vec<MathGoal>,
    numeric: Vec<(Expr,NumericRole)>,
}
#[derive(Debug)]
pub struct ResolvedCoordinateGoal {
    owner: Arc<CoordinateOwner>,
    index: usize,
}
impl ResolvedCoordinateGoal {
    pub fn index(&self) -> usize { self.index }
    pub fn term(&self) -> &MathGoal { &self.owner.goals[self.index] }
    pub fn bodies(&self) -> &[Term;3] { &self.owner.bodies }
    pub fn input(&self) -> &Term { &self.owner.input }
    pub fn definitions(&self) -> &[SeedDef;3] { &self.owner.definitions }
    pub fn numeric_roles(&self) -> &[(Expr,NumericRole)] { &self.owner.numeric }
    pub fn belongs_to(&self, scope:&Scope) -> bool { Arc::ptr_eq(&self.owner.source,&scope.0.source) }
    pub fn same_structure(&self, other:&Self) -> bool { Arc::ptr_eq(&self.owner,&other.owner) }
    pub fn calculation_matches(&self, call:&Expr) -> bool {
        let ExprKind::Call{args,..}=&call.kind else {return false};
        args.len()==1 && args[0].resolved_pin.as_deref()==Some(self.owner.definitions[2].params[0].pin_name.as_str())
            && coordinate_literal(&args[0].expr).is_ok_and(|p|p==self.owner.input)
    }
}
fn coordinate_literal(expr:&Expr)->Result<Term,ParseError> {
    if let ExprKind::TupleConstructor{elements}=&expr.kind {
        if elements.len()==2 {return Ok(Term::Pair(Box::new(coordinate_literal(&elements[0])?),Box::new(coordinate_literal(&elements[1])?)))}
    }
    if let Some(value)=expr.numeric_literal_source.as_ref().filter(|n|!n.is_decimal()).and_then(|n|n.raw().parse::<i64>().ok()) {
        if matches!(&expr.kind,ExprKind::Literal(_)) {return Ok(Term::Integer(value))}
    }
    Err(failure(expr.span,"E_PROOF_COORDINATE_LITERAL","정확 정수 좌표 원문이 필요합니다"))
}
fn coordinate_term(expr:&Expr, formal:usize, definitions:&[SeedDef;3], program:&CanonProgram,
    numeric:&mut Vec<Expr>,depth:usize)->Result<Term,ParseError> {
    if depth>32 {return Err(failure(expr.span,"E_PROOF_BUDGET","좌표 항 깊이 한도를 넘었습니다"))}
    let recur=|e:&Expr,n:&mut Vec<Expr>|coordinate_term(e,formal,definitions,program,n,depth+1);
    let term=match &expr.kind {
        ExprKind::Var(name) if name==&definitions[formal].params[0].pin_name => Term::Parameter{definition:formal,pin:0},
        ExprKind::Literal(_) => coordinate_literal(expr)?,
        ExprKind::TupleConstructor{elements} if elements.len()==2 => Term::Pair(Box::new(recur(&elements[0],numeric)?),Box::new(recur(&elements[1],numeric)?)),
        ExprKind::MultiAxisSelection(_) => {
            let (base,index)=selection_base(expr)?;
            let [index]=index.as_slice() else {return Err(failure(expr.span,"E_PROOF_COORDINATE_TYPE","좌표 선택은 하나의 성분 번호여야 합니다"))};
            Term::Select{target:Box::new(recur(base,numeric)?),index:*index}
        },
        ExprKind::Infix{left,op,right} if matches!(op.as_str(),"+"|"-") => {
            let a=Box::new(recur(left,numeric)?); let b=Box::new(recur(right,numeric)?);
            numeric.push(expr.clone());
            if op=="+" {Term::Add(a,b)}else{Term::Subtract(a,b)}
        },
        ExprKind::Call{func,args} => {
            if func==&definitions[formal].params[0].pin_name {return Err(failure(expr.span,"E_PROOF_CALL_SHADOWED","좌표 호출이 입력에 가려졌습니다"))}
            let ids=program.origin.authored_call_candidates.get(&expr.id).ok_or_else(||failure(expr.span,"E_PROOF_CALL_UNRESOLVED","좌표 호출 정의가 해소되지 않았습니다"))?;
            let [id]=ids.as_slice() else {return Err(failure(expr.span,"E_PROOF_CALL_AMBIGUOUS","좌표 호출 정의가 유일하지 않습니다"))};
            let index=definitions.iter().position(|d|d.id==*id).ok_or_else(||failure(expr.span,"E_PROOF_COORDINATE_DEFINITION","다른 좌표 정의 호출입니다"))?;
            if args.len()!=1 || args[0].resolved_pin.as_deref()!=Some(definitions[index].params[0].pin_name.as_str()) {
                return Err(failure(expr.span,"E_PROOF_ARGUMENT_BINDING","좌표 호출의 실제 핀 대응이 다릅니다"));
            }
            Term::Apply{definition:index,argument:Box::new(recur(&args[0].expr,numeric)?)}
        },
        _=>return Err(failure(expr.span,"E_PROOF_COORDINATE_UNSUPPORTED","이 좌표 계약에서 지원하지 않는 항입니다")),
    };
    if term.term_type(&[(TermType::IntegerPair,TermType::IntegerPair);3]).is_none() {
        return Err(failure(expr.span,"E_PROOF_COORDINATE_TYPE","좌표 항의 입력·성분·결과 형식이 다릅니다"));
    }
    Ok(term)
}
fn coordinate_expected_bodies()->[Term;3] {
    let x=|d,i|Term::Select{target:Box::new(Term::Parameter{definition:d,pin:0}),index:i};
    [Term::Pair(Box::new(Term::Subtract(Box::new(Term::Integer(0)),Box::new(x(0,1)))),Box::new(x(0,0))),
     Term::Pair(Box::new(Term::Add(Box::new(x(1,0)),Box::new(Term::Integer(10)))),Box::new(Term::Add(Box::new(x(1,1)),Box::new(Term::Integer(2))))),
     Term::Apply{definition:1,argument:Box::new(Term::Apply{definition:0,argument:Box::new(Term::Parameter{definition:2,pin:0})})}]
}
fn coordinate_goals(input:&Term)->Vec<MathGoal> {
    let apply=|d,t|Term::Apply{definition:d,argument:Box::new(t)};
    let select=|t,i|Term::Select{target:Box::new(t),index:i};
    let rotation=apply(0,input.clone());
    let terms=[Term::Subtract(Box::new(Term::Integer(0)),Box::new(select(input.clone(),1))),
        Term::Add(Box::new(select(rotation.clone(),0)),Box::new(Term::Integer(10))),
        Term::Add(Box::new(select(rotation.clone(),1)),Box::new(Term::Integer(2)))];
    let mut goals=Vec::new();
    for occurrence in 0..6 {goals.push(MathGoal::Defined{occurrence,term:terms[occurrence%3].clone()});}
    goals.push(MathGoal::Defined{occurrence:6,term:apply(2,input.clone())});
    goals.push(MathGoal::Equal{left:apply(2,input.clone()),right:apply(1,rotation)});
    goals
}
fn resolve_coordinate_source(prepared:&PreparedProofSource,candidate:&NumericSourceCandidate<'_>)->Result<Vec<ResolvedTransfer>,ParseError> {
    let program=&prepared.program;
    let context=Arc::new(SourceContext{source:prepared.source.clone(),path:prepared.path.clone()});
    let q=TypeRef::Tuple(vec![TypeRef::Named("바른수".into());2]);
    let reject=|span,msg|failure(span,"E_PROOF_COORDINATE_ADMISSION",msg);
    if let Some(span)=program.origin.authored_jjaim_blocks.first(){return Err(reject(*span,"미해석 짜임 블록은 생략할 수 없습니다"))}
    let mut owners=HashMap::new();let mut used=std::collections::HashSet::new();
    for stmt in &program.root_stmts {
        let Stmt::Definition{name,value,type_ref:TypeRef::Infer,reaction:None,..}=stmt else {return Err(reject(Span::new(0,prepared.source.len()),"좌표 구조 정의 외의 실행 문장은 지원하지 않습니다"))};
        if owners.contains_key(name) || program.items.iter().any(|TopLevelItem::SeedDef(d)|d.canonical_name==*name) {
            return Err(reject(value.span,"좌표 구조의 선언이 중복되거나 가려졌습니다"));
        }
        let binding=program.origin.static_structure_calls.get(&value.id).filter(|b|b.contract==StaticStructureContract::FixedCoordinateCompositionV1)
            .ok_or_else(||reject(value.span,"공통 분석이 결속한 채택 좌표 도우미가 필요합니다"))?;
        let extra=binding.coordinate.as_ref().ok_or_else(||reject(value.span,"좌표 연산 결속이 없습니다"))?;
        let definitions=[binding.operation.clone(),extra.translation.clone(),extra.composition.clone()];
        for d in &definitions {
            if d.seed_kind!=SeedKind::Semssi || !d.modifiers.is_empty() || !d.type_params.is_empty() || d.proof_header.is_some()
                || d.params.len()!=1 || d.params[0].optional || d.params[0].default_value.is_some()
                || d.params[0].type_ref!=q || d.result_type.as_ref()!=Some(&q) || d.params[0].pin_name==d.canonical_name {
                return Err(reject(d.span,"좌표 연산은 명시된 필수 Q 입력·Q 결과의 순수 셈씨여야 합니다"));
            }
            used.insert(d.id);
        }
        if definitions[0].id==definitions[1].id || definitions[0].id==definitions[2].id || definitions[1].id==definitions[2].id {
            return Err(reject(value.span,"회전·이동·합성의 실제 정의가 서로 달라야 합니다"));
        }
        let input=coordinate_literal(&extra.input)?;
        if input!=Term::Pair(Box::new(Term::Integer(1)),Box::new(Term::Integer(0))) {return Err(reject(extra.input.span,"첫 좌표 계약의 입력은 (1,0)입니다"))}
        let mut numeric=Vec::new();
        let bodies=[coordinate_term(single_result(&definitions[0])?,0,&definitions,program,&mut numeric,0)?,
            coordinate_term(single_result(&definitions[1])?,1,&definitions,program,&mut numeric,0)?,
            coordinate_term(single_result(&definitions[2])?,2,&definitions,program,&mut numeric,0)?];
        for (index,(actual,expected)) in bodies.iter().zip(coordinate_expected_bodies()).enumerate() {
            if actual!=&expected {return Err(reject(single_result(&definitions[index])?.span,"고정 A·B·C의 성분·순서·최종 반환 경로가 다릅니다"));}
        }
        let mut roles=Vec::new();
        for expr in numeric {
            let ExprKind::Infix{left,op,right}=&expr.kind else {unreachable!()};
            let role=if op=="+" {NumericRole::CheckedDimensionlessIntegerAdd}else{NumericRole::CheckedDimensionlessIntegerSubtract};
            for e in [&expr,left.as_ref(),right.as_ref()] {
                let rows:Vec<_>=candidate.expressions.iter().filter(|(id,_)|*id==e.id).collect();
                if rows.len()!=1 || rows[0].1!=TypeRef::Named("바른수".into()) {
                    return Err(failure(e.span,"E_PROOF_NUMERIC_TYPE","실제 연산·피연산자의 유일한 바른수 분석이 필요합니다"));
                }
            }
            let rows:Vec<_>=candidate.operations.iter().filter(|(id,_)|*id==expr.id).collect();
            if rows.len()!=1 || rows[0].1!=role {return Err(failure(expr.span,"E_PROOF_NUMERIC_CONTRACT","실제 숫자 실행 분기·실패 계약이 일치하지 않습니다"));}
            roles.push((expr,role));
        }
        let goals=coordinate_goals(&input);
        owners.insert(name.clone(),Arc::new(CoordinateOwner{source:context.clone(),definitions,bodies,input,goals,numeric:roles}));
    }
    let mut transfers=Vec::new();
    for TopLevelItem::SeedDef(seed) in &program.items {
        if used.contains(&seed.id){continue}
        if seed.seed_kind!=SeedKind::Semssi || !seed.modifiers.is_empty() || !seed.type_params.is_empty() || seed.params.len()!=8
            || seed.proof_header.as_ref().is_some_and(|header|!header.propositions.is_empty()) {
            return Err(reject(seed.span,"좌표 소비 정의에는 여덟 필수 증거 입력이 필요합니다"));
        }
        let scope=Scope(Arc::new(DefinitionScope{source:context.clone()}));
        let mut assumptions=Vec::new();let mut names=std::collections::HashSet::new();let mut indices=std::collections::HashSet::new();
        let mut selected:Option<Arc<CoordinateOwner>>=None;
        for pin in &seed.params {
            let TypeRef::Applied{name,args}=&pin.type_ref else {return Err(reject(pin.span,"좌표 목표의 보임 입력이 필요합니다"))};
            let [TypeRef::StructureLaw{structure,member}]=args.as_slice() else {return Err(reject(pin.span,"좌표 목표 멤버가 필요합니다"))};
            if name!="보임" || pin.optional || pin.default_value.is_some() || pin.pin_name==seed.canonical_name || !names.insert(pin.pin_name.clone()) {
                return Err(reject(pin.span,"증거 입력의 이름·필수성·소유가 다릅니다"));
            }
            let owner=owners.get(structure).ok_or_else(||reject(pin.span,"해소되지 않은 좌표 구조입니다"))?;
            let index=COORDINATE_GOAL_MEMBERS.iter().position(|n|*n==member).ok_or_else(||reject(pin.span,"지원하지 않는 좌표 목표입니다"))?;
            if !indices.insert(index) || selected.as_ref().is_some_and(|s|!Arc::ptr_eq(s,owner)) {return Err(reject(pin.span,"목표가 중복되거나 서로 다른 구조의 가정입니다"));}
            selected=Some(owner.clone());
            let goal=Proposition(Arc::new(PropositionDeclaration{scope:scope.clone(),parts:None,law:None,
                coordinate:Some(Arc::new(ResolvedCoordinateGoal{owner:owner.clone(),index}))}));
            assumptions.push(Assumption(Arc::new(AssumptionDeclaration{scope:scope.clone(),goal})));
        }
        let owner=selected.ok_or_else(||reject(seed.span,"좌표 구조 가정이 없습니다"))?;
        let law_index=assumptions.iter().position(|a|a.goal().coordinate_goal().is_some_and(|g|g.index==7)).unwrap();
        let law=assumptions[law_index].clone();
        let value=single_result(seed)?;
        let coordinate_calculation=if seed.result_type.as_ref()==Some(&q) {
            let ExprKind::Call{args,func}=&value.kind else {return Err(reject(value.span,"계산 결과는 실제 C 호출이어야 합니다"))};
            if names.contains(func) || program.origin.authored_call_candidates.get(&value.id).map(Vec::as_slice)!=Some(&[owner.definitions[2].id][..])
                || args.len()!=1 || args[0].resolved_pin.as_deref()!=Some(owner.definitions[2].params[0].pin_name.as_str())
                || coordinate_literal(&args[0].expr)?!=owner.input {return Err(reject(value.span,"동일 C·입력·핀·최종 반환이 필요합니다"));}
            Some(value.clone())
        }else{
            let Some(TypeRef::Applied{name,args})=&seed.result_type else {return Err(reject(seed.span,"좌표 법칙 보임 결과가 필요합니다"))};
            let [TypeRef::StructureLaw{structure,member}]=args.as_slice() else {return Err(reject(seed.span,"좌표 법칙 목표가 필요합니다"))};
            if name!="보임" || member!=COORDINATE_GOAL_MEMBERS[7] || !owners.get(structure).is_some_and(|o|Arc::ptr_eq(o,&owner))
                || !matches!(&value.kind,ExprKind::Var(n) if n==&seed.params[law_index].pin_name) {return Err(reject(value.span,"실제 법칙 가정을 같은 법칙 목표에 전달해야 합니다"));}
            None
        };
        transfers.push(ResolvedTransfer{scope,goal:law.goal().clone(),assumptions,evidence:ResolvedEvidence::Assumption(law),
            definition_span:seed.span,result_span:value.span,propositions:Vec::new(),application:None,application_depth:0,projection:Vec::new(),
            consumption:None,calculation:None,coordinate_calculation});
    }
    if transfers.is_empty(){return Err(reject(Span::new(0,prepared.source.len()),"검사할 좌표 보임 정의가 없습니다"))}
    Ok(transfers)
}


/// Location in an actual definition signature, never an external type factory.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum SavedProofSignatureSlot { Input(usize), Result }
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum SavedProofTypeChild { Argument(usize), Tuple(usize), Conjunction(usize), FunctionInput(usize), FunctionResult, TensorElement }
#[derive(Debug)]
enum SavedProofBinding {
    Algebra { parameters:Vec<ParamPin>, equation:crate::algebra_authoring::AlgebraEquality },
    Structure(Arc<ResolvedStructureLaw>),
}
/// Non-serializable source-signature fact, NOT a checked Proof or saved handle.
/// All fields are private; only PreparedProofSource can produce a fact.
#[derive(Debug)]
pub struct VerifiedSavedProofTypeMaterial {
    owner:Arc<SourceContext>, definition:NodeId, slot:SavedProofSignatureSlot,
    children:Vec<SavedProofTypeChild>, text:String, binding:SavedProofBinding,
}
fn signature_error(span:Span,message:&str)->ParseError { failure(span,"E_SAVED_PROOF_SIGNATURE",message) }
fn signature_type<'a>(seed:&'a SeedDef,slot:&SavedProofSignatureSlot,children:&[SavedProofTypeChild])->Option<&'a TypeRef> {
    let mut ty=match slot {SavedProofSignatureSlot::Input(i)=>&seed.params.get(*i)?.type_ref,SavedProofSignatureSlot::Result=>seed.result_type.as_ref()?};
    for child in children {ty=match (child,ty) {
        (SavedProofTypeChild::Argument(i),TypeRef::Applied{args,..})=>args.get(*i)?,
        (SavedProofTypeChild::Tuple(i),TypeRef::Tuple(args))=>args.get(*i)?,
        (SavedProofTypeChild::Conjunction(i),TypeRef::PropositionConjunction(args))=>args.get(*i)?,
        (SavedProofTypeChild::FunctionInput(i),TypeRef::Function{params,..})=>params.get(*i)?,
        (SavedProofTypeChild::FunctionResult,TypeRef::Function{result,..})=>result,
        (SavedProofTypeChild::TensorElement,TypeRef::Tensor{element,..})=>element,
        _=>return None,
    };} Some(ty)
}
impl VerifiedSavedProofTypeMaterial {
    pub fn kind(&self)->&'static str {match self.binding {SavedProofBinding::Algebra{..}=>"algebra_equality",SavedProofBinding::Structure(_)=>"structure_law"}}
    pub fn description(&self)->&str {&self.text}
    pub fn slot(&self)->&SavedProofSignatureSlot {&self.slot}
    pub fn children(&self)->&[SavedProofTypeChild] {&self.children}
    pub fn definition_id(&self)->NodeId {self.definition}
    pub fn algebra_parameters(&self)->Option<&[ParamPin]> {match &self.binding {SavedProofBinding::Algebra{parameters,..}=>Some(parameters),_=>None}}
    pub fn algebra_goal(&self)->Option<&crate::algebra_authoring::AlgebraEquality> {match &self.binding {SavedProofBinding::Algebra{equation,..}=>Some(equation),_=>None}}
    pub fn structure_law(&self)->Option<&ResolvedStructureLaw> {match &self.binding {SavedProofBinding::Structure(law)=>Some(law),_=>None}}
    pub fn source_matches(&self,source:&str,path:&str)->bool {self.owner.source==source&&self.owner.path==path}
    pub fn definition<'a>(&self,prepared:&'a PreparedProofSource)->Result<&'a SeedDef,ParseError> {
        if !Arc::ptr_eq(&self.owner,&prepared.signature_owner) {return Err(signature_error(Span::new(0,prepared.source.len()),"다른 파싱 문맥의 signature 근거입니다"));}
        prepared.program.items.iter().map(|TopLevelItem::SeedDef(s)|s).find(|s|s.id==self.definition)
            .ok_or_else(||signature_error(Span::new(0,prepared.source.len()),"원문 정의가 없습니다"))
    }
    /// The consumer must use this same prepared program. Equal spelling, cloned
    /// AST/TypeRef or a fresh parse of identical bytes is not this occurrence.
    pub fn validate_occurrence(&self,prepared:&PreparedProofSource,seed:&SeedDef,kind:&TypeRef)->Result<(),ParseError> {
        let actual=self.definition(prepared)?;
        if !std::ptr::eq(actual,seed) || !signature_type(actual,&self.slot,&self.children).is_some_and(|t|std::ptr::eq(t,kind)) {
            return Err(signature_error(seed.span,"실제 정의·타입 occurrence가 일치하지 않습니다"));
        } Ok(())
    }
    pub fn type_ref<'a>(&self,prepared:&'a PreparedProofSource)->Result<&'a TypeRef,ParseError> {
        signature_type(self.definition(prepared)?,&self.slot,&self.children)
            .ok_or_else(||signature_error(Span::new(0,prepared.source.len()),"원문 타입 occurrence가 없습니다"))
    }
}

impl PreparedProofSource {
    /// Header admission on the COMPLETE existing parse; never reparse, remove
    /// roots, call a filtered proof resolver, or claim body/checker success.
    pub fn saved_proof_signatures(&self)->Result<Vec<VerifiedSavedProofTypeMaterial>,ParseError> {
        let mut facts=Vec::new();
        let mut structure_cache=HashMap::new();
        for TopLevelItem::SeedDef(seed) in &self.program.items {
            for (i,pin) in seed.params.iter().enumerate() {
                self.collect_saved_proof_types(seed,&pin.type_ref,SavedProofSignatureSlot::Input(i),&mut Vec::new(),&mut facts,&mut structure_cache)?;
            }
            if let Some(ty)=&seed.result_type {self.collect_saved_proof_types(seed,ty,SavedProofSignatureSlot::Result,&mut Vec::new(),&mut facts,&mut structure_cache)?;}
        } Ok(facts)
    }
    fn collect_saved_proof_types(&self,seed:&SeedDef,ty:&TypeRef,slot:SavedProofSignatureSlot,children:&mut Vec<SavedProofTypeChild>,facts:&mut Vec<VerifiedSavedProofTypeMaterial>,structure_cache:&mut HashMap<String,Arc<ResolvedStructureLaw>>)->Result<(),ParseError> {
        if children.len()>64||facts.len()>4096 {return Err(signature_error(seed.span,"signature 분석 한도를 넘었습니다"));}
        let binding=match ty {
            TypeRef::AlgebraEquality{..}=>{
                let (parameters,equation)=crate::algebra_authoring::saved_signature_equation(&self.program,seed,ty)?;
                Some(SavedProofBinding::Algebra{parameters,equation})
            },
            TypeRef::StructureLaw{structure,member}=>Some(SavedProofBinding::Structure(self.saved_structure_law(seed,structure,member,structure_cache)?)),
            _=>None,
        };
        if let Some(binding)=binding {
            facts.push(VerifiedSavedProofTypeMaterial{owner:self.signature_owner.clone(),definition:seed.id,slot,children:children.clone(),
                text:crate::normalizer::Normalizer::source_type_description(ty,self.program.source_epoch),binding});
            return Ok(());
        }
        let mut descend=|ty:&TypeRef,child:SavedProofTypeChild|->Result<(),ParseError> {
            children.push(child);let result=self.collect_saved_proof_types(seed,ty,slot.clone(),children,facts,structure_cache);children.pop();result
        };
        match ty {
            TypeRef::Applied{args,..}=>for (i,t) in args.iter().enumerate(){descend(t,SavedProofTypeChild::Argument(i))?;},
            TypeRef::Tuple(args)=>for (i,t) in args.iter().enumerate(){descend(t,SavedProofTypeChild::Tuple(i))?;},
            TypeRef::PropositionConjunction(args)=>for (i,t) in args.iter().enumerate(){descend(t,SavedProofTypeChild::Conjunction(i))?;},
            TypeRef::Function{params,result}=>{for(i,t)in params.iter().enumerate(){descend(t,SavedProofTypeChild::FunctionInput(i))?;}descend(result,SavedProofTypeChild::FunctionResult)?;},
            TypeRef::Tensor{element,..}=>descend(element,SavedProofTypeChild::TensorElement)?,
            _=>{},
        } Ok(())
    }
    fn saved_structure_law(&self,seed:&SeedDef,name:&str,member:&str,cache:&mut HashMap<String,Arc<ResolvedStructureLaw>>)->Result<Arc<ResolvedStructureLaw>,ParseError> {
        let program=&self.program;
        if member!="법칙" || seed.params.iter().any(|p|p.pin_name==name) || seed.type_params.iter().any(|p|p.name==name) {
            return Err(signature_error(seed.span,"가려지거나 미지원인 구조 법칙입니다"));
        }
        let mut callable_env=HashMap::new();
        for TopLevelItem::SeedDef(def) in &program.items {
            if matches!(def.canonical_name.as_str(),"바른수"|"짜임") || callable_env.insert(def.canonical_name.as_str(),Arc::new(def.clone())).is_some() {
                return Err(signature_error(def.span,"구조 의존 정의가 중복되거나 기본 씨를 가립니다"));
            }
        }
        let mut declarations=HashMap::new();let mut duplicates=std::collections::HashSet::new();
        // This is lexical declaration indexing on the original program, not a
        // filtered program fed to proof admission. Executable roots remain.
        for stmt in &program.root_stmts {
            if let Stmt::Definition{name,..}=stmt {if matches!(name.as_str(),"바른수"|"짜임"){return Err(signature_error(seed.span,"구조의 기본 씨가 다른 원문 선언에 가려졌습니다"));}if declarations.insert(name.as_str(),stmt).is_some(){duplicates.insert(name.as_str());}}
        }
        let stmt=*declarations.get(name).ok_or_else(||signature_error(seed.span,"현재 모듈에 해소된 구조 정의가 없습니다"))?;
        if duplicates.contains(name)||callable_env.contains_key(name){return Err(signature_error(seed.span,"구조 이름의 소유자가 중복됩니다"));}
        let Stmt::Definition{value,span,type_ref:TypeRef::Infer,reaction:None,..}=stmt else{return Err(signature_error(seed.span,"원문 정적 구조 선언이 필요합니다"))};
        if span.end>=seed.span.start{return Err(signature_error(*span,"signature 앞의 실제 구조 선언이 필요합니다"));}
        let mut descriptors=HashMap::new();
        for (key,decl) in &declarations {
            if let Stmt::Definition{value:expr,span:ds,type_ref:TypeRef::Infer,reaction:None,..}=decl {
                if ds.end<span.start && matches!(expr.kind,ExprKind::Pack{..}) && !duplicates.contains(key) && !callable_env.contains_key(key) {descriptors.insert(*key,(*decl,expr));}
            }
        }
        if let Some(law)=cache.get(name){return Ok(law.clone());}
        let law=resolve_structure_declaration(program,&self.signature_owner,stmt,value,&callable_env,&descriptors,&mut std::collections::HashSet::new())?;
        cache.insert(name.into(),law.clone());Ok(law)
    }
}
