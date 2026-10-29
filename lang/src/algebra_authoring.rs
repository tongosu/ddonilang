//! Source-owned candidates for the adopted conditional Q equality flow.
//! Only PreparedProofSource can construct these facts from the shared parser.
use crate::{ast::*, ParseError};
use ddonirang_symbolic::algebra_term::QTerm;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone,Debug)]
pub struct AlgebraEquality { pub left:QTerm, pub right:QTerm, pub left_span:Span, pub right_span:Span }
#[derive(Clone,Copy,Debug)]
pub enum AlgebraEvidenceRef { Assumption(usize), Step(usize) }
#[derive(Clone,Debug)]
pub enum AlgebraRule {
    SubtractSame { prior:AlgebraEvidenceRef, term:QTerm },
    RewriteSides { prior:AlgebraEvidenceRef, goal:AlgebraEquality },
    DivideSameNonzero { prior:AlgebraEvidenceRef, coefficient:QTerm, coefficient_span:Span },
}
#[derive(Clone,Debug)]
pub struct AlgebraStep { pub rule:AlgebraRule, pub span:Span }
#[derive(Debug)]
struct SourceDefinition {
    source:String, path:String, declaration:SeedDef,
    parameters:Vec<ParamPin>, assumptions:Vec<AlgebraEquality>,
    goal:AlgebraEquality, steps:Vec<AlgebraStep>, result:AlgebraEvidenceRef, result_span:Span,
}
/// Immutable source provenance, not a checked verdict. No public AST constructor.
#[derive(Clone,Debug)]
pub struct ResolvedAlgebraDefinition(Arc<SourceDefinition>);
impl ResolvedAlgebraDefinition {
    pub fn same_owner(&self,other:&Self)->bool {Arc::ptr_eq(&self.0,&other.0)}
    pub fn source_matches(&self,source:&str,path:&str)->bool {self.0.source==source&&self.0.path==path}
    pub fn declaration(&self)->&SeedDef {&self.0.declaration}
    pub fn parameters(&self)->&[ParamPin] {&self.0.parameters}
    pub fn assumptions(&self)->&[AlgebraEquality] {&self.0.assumptions}
    pub fn goal(&self)->&AlgebraEquality {&self.0.goal}
    pub fn steps(&self)->&[AlgebraStep] {&self.0.steps}
    pub fn result(&self)->AlgebraEvidenceRef {self.0.result}
    pub fn result_span(&self)->Span {self.0.result_span}
}

fn error(span:Span,code:&str,message:&str)->ParseError {ParseError{span,message:format!("{code}: {message}")}}
pub(crate) fn is_goal(t:&TypeRef)->bool {
    matches!(t,TypeRef::Applied{name,args} if name=="보임"&&matches!(args.as_slice(),[TypeRef::AlgebraEquality{..}]))
}
fn lower(term:&AlgebraTermSyntax,parameters:&HashMap<String,usize>)->Result<QTerm,ParseError> {
    Ok(match &term.kind {
        AlgebraTermKind::Integer(n)=>QTerm::Integer(*n),
        AlgebraTermKind::Name(name)=>QTerm::Parameter(*parameters.get(name).ok_or_else(||error(term.span,"E_PROOF_ALGEBRA_BINDER","이 정의의 필수 나눔수 매개변수가 아닙니다"))?),
        AlgebraTermKind::Add(a,b)=>QTerm::Add(Box::new(lower(a,parameters)?),Box::new(lower(b,parameters)?)),
        AlgebraTermKind::Subtract(a,b)=>QTerm::Subtract(Box::new(lower(a,parameters)?),Box::new(lower(b,parameters)?)),
        AlgebraTermKind::Multiply(a,b)=>QTerm::Multiply(Box::new(lower(a,parameters)?),Box::new(lower(b,parameters)?)),
        AlgebraTermKind::DivideByInteger(a,b)=>QTerm::DivideByInteger(Box::new(lower(a,parameters)?),*b),
    })
}
fn goal(t:&TypeRef,parameters:&HashMap<String,usize>,span:Span)->Result<AlgebraEquality,ParseError> {
    if let TypeRef::Applied{name,args}=t {
        if let ("보임",[TypeRef::AlgebraEquality{left,right}])=(name.as_str(),args.as_slice()) {
            return Ok(AlgebraEquality{left:lower(left,parameters)?,right:lower(right,parameters)?,left_span:left.span,right_span:right.span});
        }
    }
    Err(error(span,"E_PROOF_ALGEBRA_GOAL","정확한 정적 등식 보임 목표가 필요합니다"))
}

fn stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::SumDefinition { span, .. }
        | Stmt::SumMatch { span, .. }
        | Stmt::DeclBlock { span, .. }
        | Stmt::Definition { span, .. }
        | Stmt::DerivedBinding { span, .. }
        | Stmt::Mutate { span, .. }
        | Stmt::ResultFlow { span, .. }
        | Stmt::Expr { span, .. }
        | Stmt::Receive { span, .. }
        | Stmt::Send { span, .. }
        | Stmt::Show { span, .. }
        | Stmt::Inspect { span, .. }
        | Stmt::PublicObservation { span, .. }
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
        | Stmt::HookWhile { span, .. }
        | Stmt::MetaBlock { span, .. }
        | Stmt::Pragma { span, .. } => *span,
    }
}

pub(crate) fn resolve(program:&CanonProgram,source:&str,path:&str)->Result<Vec<ResolvedAlgebraDefinition>,ParseError> {
    if !program.root_stmts.is_empty()||!program.origin.authored_jjaim_blocks.is_empty() {
        let span = program.root_stmts.iter().map(stmt_span)
            .chain(program.origin.authored_jjaim_blocks.iter().copied())
            .min_by_key(|span| (span.start, span.end)).expect("unsupported source exists");
        return Err(error(span,"E_PROOF_ROOT_UNSUPPORTED","대수 정의 검사에서 실행 문장이나 미해석 짜임을 무시할 수 없습니다"));
    }
    if program.items.is_empty(){return Err(error(Span::new(0,source.len()),"E_PROOF_DEFINITION","검사할 대수 정의가 없습니다"));}
    if program.items.iter().any(|TopLevelItem::SeedDef(s)|s.canonical_name=="나눔수") {
        return Err(error(Span::new(0,source.len()),"E_PROOF_ALGEBRA_TYPE","기본 나눔수가 다른 선언에 가려졌습니다"));
    }
    let mut resolved=Vec::new();
    for TopLevelItem::SeedDef(seed) in &program.items {
        if seed.seed_kind!=SeedKind::Semssi||!seed.modifiers.is_empty()||!seed.type_params.is_empty()
            ||seed.proof_header.as_ref().is_some_and(|h|!h.propositions.is_empty()) {
            return Err(error(seed.span,"E_PROOF_DEFINITION","첫 대수 흐름은 명시된 필수 입력의 셈씨 정의만 지원합니다"));
        }
        let mut parameters=Vec::new();let mut names=HashMap::new();let mut all_names=std::collections::HashSet::new();
        all_names.insert(seed.canonical_name.clone());
        for pin in &seed.params {
            if pin.optional||pin.default_value.is_some()||!all_names.insert(pin.pin_name.clone()) {
                return Err(error(pin.span,"E_PROOF_ALGEBRA_BINDER","대수 입력은 중복 없는 필수 매개변수여야 합니다"));
            }
            if pin.type_ref==TypeRef::Named("나눔수".into()) {
                names.insert(pin.pin_name.clone(),parameters.len());parameters.push(pin.clone());
            } else if !is_goal(&pin.type_ref) {
                return Err(error(pin.span,"E_PROOF_ALGEBRA_TYPE","나눔수 또는 명시된 등식 보임 입력이 필요합니다"));
            }
        }
        let mut assumptions=Vec::new();let mut locals=HashMap::new();
        for pin in &seed.params {
            if is_goal(&pin.type_ref) {
                let equation=goal(&pin.type_ref,&names,pin.span)?;
                locals.insert(pin.pin_name.clone(),AlgebraEvidenceRef::Assumption(assumptions.len()));assumptions.push(equation);
            }
        }
        if assumptions.is_empty(){return Err(error(seed.span,"E_PROOF_ASSUMPTION","첫 대수 흐름은 명시된 등식 가정이 필요합니다"));}
        let expected=goal(seed.result_type.as_ref().ok_or_else(||error(seed.span,"E_PROOF_RESULT","반환 목표가 필요합니다"))?,&names,seed.span)?;
        let body=seed.body.as_ref().ok_or_else(||error(seed.span,"E_PROOF_RESULT","보임 본문이 필요합니다"))?;
        if body.stmts.len()>256{return Err(error(body.span,"E_PROOF_BUDGET","대수 단계 수 한도를 넘었습니다"));}
        let mut steps=Vec::new();let mut result=None;
        for (index,stmt) in body.stmts.iter().enumerate() {
            let (value,local)=match stmt {
                Stmt::Definition{name,type_ref:TypeRef::Infer,value,reaction:None,..}=>{
                    if !all_names.insert(name.clone()){return Err(error(value.span,"E_PROOF_ALGEBRA_BINDER","가정·매개변수·지역 단계 이름은 서로 구별되어야 합니다"));}
                    (value,Some(name))
                }
                Stmt::Mutate{target,value,deferred:false,..} if matches!(&target.kind,ExprKind::Var(n) if n==&seed.canonical_name)=>{
                    if index+1!=body.stmts.len(){return Err(error(target.span,"E_PROOF_RESULT","최종 결과 기록은 마지막 단계여야 합니다"));}
                    (value,None)
                }
                _=>return Err(error(stmt_span(stmt),"E_PROOF_RULE_UNSUPPORTED","지역 증명 단계와 마지막 결과 기록만 지원합니다")),
            };
            let reference=if let ExprKind::Var(name)=&value.kind {
                *locals.get(name).ok_or_else(||error(value.span,"E_PROOF_ASSUMPTION","현재 정의의 가정이나 앞선 증명 단계가 아닙니다"))?
            } else {
                let ExprKind::Call{args,..}=&value.kind else {return Err(error(value.span,"E_PROOF_RULE_UNSUPPORTED","성공값 대신 정적 증명 조작이 필요합니다"));};
                let binding=program.origin.algebra_operation(value.id).filter(|b|b.definition==seed.id)
                    .ok_or_else(||error(value.span,"E_PROOF_CALL_UNRESOLVED","이 정의에 결속된 정적 증명 조작이 아닙니다"))?;
                let field=|name:&str|args.iter().find(|a|a.resolved_pin.as_deref()==Some(name)).map(|a|&a.expr)
                    .ok_or_else(||error(value.span,"E_PROOF_ALGEBRA_PIN","필수 받는자리가 없습니다"));
                let prior=field("근거")?;
                let ExprKind::Var(name)=&prior.kind else {return Err(error(prior.span,"E_PROOF_ASSUMPTION","명시된 가정 또는 앞선 단계 이름이 필요합니다"));};
                let prior=*locals.get(name).ok_or_else(||error(prior.span,"E_PROOF_ASSUMPTION","다른 문맥이나 뒤의 단계를 근거로 사용할 수 없습니다"))?;
                let rule=match binding.operation {
                    StaticAlgebraOperation::SubtractSame=>AlgebraRule::SubtractSame{prior,term:lower(&crate::algebra_syntax::term(field("항")?)?,&names)?},
                    StaticAlgebraOperation::DivideSameNonzero=>{
                        let coefficient=field("계수")?;
                        AlgebraRule::DivideSameNonzero{prior,coefficient:lower(&crate::algebra_syntax::term(coefficient)?,&names)?,coefficient_span:coefficient.span}
                    }
                    StaticAlgebraOperation::RewriteSides=>{
                        let left=field("왼")?;let right=field("오른")?;
                        AlgebraRule::RewriteSides{prior,goal:AlgebraEquality{left:lower(&crate::algebra_syntax::term(left)?,&names)?,right:lower(&crate::algebra_syntax::term(right)?,&names)?,left_span:left.span,right_span:right.span}}
                    }
                };
                let reference=AlgebraEvidenceRef::Step(steps.len());steps.push(AlgebraStep{rule,span:value.span});reference
            };
            if let Some(name)=local {locals.insert(name.clone(),reference);}else{result=Some((reference,value.span));}
        }
        let (result,result_span)=result.ok_or_else(||error(body.span,"E_PROOF_RESULT","최종 결과가 없습니다"))?;
        resolved.push(ResolvedAlgebraDefinition(Arc::new(SourceDefinition{source:source.into(),path:path.into(),declaration:seed.clone(),parameters,assumptions,goal:expected,steps,result,result_span})));
    }
    Ok(resolved)
}


/// Header-only producer helper on the actual shared parse occurrence. It does
/// not inspect proof steps or issue a verdict. The caller retains that owner.
pub(crate) fn saved_signature_equation(program:&CanonProgram,seed:&SeedDef,kind:&TypeRef)->Result<(Vec<ParamPin>,AlgebraEquality),ParseError> {
    let TypeRef::AlgebraEquality{left,right}=kind else{return Err(error(seed.span,"E_PROOF_ALGEBRA_TYPE","실제 등식 목표가 필요합니다"))};
    let shadow=program.items.iter().any(|TopLevelItem::SeedDef(s)|s.canonical_name=="나눔수")
        || program.root_stmts.iter().any(|s|matches!(s,Stmt::Definition{name,..} if name=="나눔수"))
        || seed.type_params.iter().any(|p|p.name=="나눔수") || seed.params.iter().any(|p|p.pin_name=="나눔수");
    if shadow{return Err(error(seed.span,"E_PROOF_ALGEBRA_TYPE","기본 나눔수가 다른 선언에 가려졌습니다"));}
    let mut parameters=Vec::new();let mut names=HashMap::new();let mut seen=std::collections::HashSet::new();
    seen.insert(seed.canonical_name.as_str());
    for pin in &seed.params {
        if !seen.insert(pin.pin_name.as_str()){return Err(error(pin.span,"E_PROOF_ALGEBRA_BINDER","대수 입력 이름이 중복되었습니다"));}
        if pin.type_ref==TypeRef::Named("나눔수".into()) {
            if pin.optional||pin.default_value.is_some(){return Err(error(pin.span,"E_PROOF_ALGEBRA_BINDER","목표 변수는 필수 나눔수 입력이어야 합니다"));}
            names.insert(pin.pin_name.clone(),parameters.len());parameters.push(pin.clone());
        }
    }
    let equation=AlgebraEquality{left:lower(left,&names)?,right:lower(right,&names)?,left_span:left.span,right_span:right.span};
    Ok((parameters,equation))
}
