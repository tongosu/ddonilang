//! Independent replay of source-owned conditional Q algebra steps.
//! Checked handles are private, non-serializable and retain the whole context.
use ddonirang_lang::{algebra_authoring::*, ParseError, Span, TypeRef};
use ddonirang_symbolic::algebra_term::{normalize, NormalizeError, NormalizedQ, QTerm, closed_reciprocal, ReciprocalError};

#[derive(Clone,Debug)]
pub struct CheckedAlgebraDefinition { context:ResolvedAlgebraDefinition }
impl CheckedAlgebraDefinition {
    pub fn resolved(&self)->&ResolvedAlgebraDefinition {&self.context}
    pub fn validate_source(&self,source:&str,path:&str)->Result<(),ParseError> {
        if !self.context.source_matches(source,path) {
            return Err(error(self.context.result_span(),"E_PROOF_STALE_SOURCE","검사한 원문 문맥이 변경되었습니다"));
        }
        check_algebra_definition(&self.context).map(|_|())
    }
}
fn error(span:Span,code:&str,message:&str)->ParseError {ParseError{span,message:format!("{code}: {message}")}}
fn normal(term:&QTerm,count:usize,span:Span)->Result<NormalizedQ,ParseError> {
    normalize(term,count).map_err(|e|match e {
        NormalizeError::BudgetExceeded=>error(span,"E_PROOF_BUDGET","정확 대수 검사 한도를 넘었습니다. 반례 판정이 아닙니다"),
        NormalizeError::UnboundParameter=>error(span,"E_PROOF_ALGEBRA_BINDER","항의 변수가 이 정의의 매개변수에 결속되지 않았습니다"),
        NormalizeError::ZeroDenominator=>error(span,"E_PROOF_ALGEBRA_DEFINEDNESS","분모가 0인 항은 검사할 수 없습니다"),
    })
}
fn validate(eq:&AlgebraEquality,count:usize)->Result<(),ParseError> {
    normal(&eq.left,count,eq.left_span)?;normal(&eq.right,count,eq.right_span)?;Ok(())
}
fn lookup<'a>(reference:AlgebraEvidenceRef,context:&'a ResolvedAlgebraDefinition,steps:&'a [AlgebraEquality],span:Span)->Result<&'a AlgebraEquality,ParseError> {
    match reference {
        AlgebraEvidenceRef::Assumption(i)=>context.assumptions().get(i),
        AlgebraEvidenceRef::Step(i)=>steps.get(i),
    }.ok_or_else(||error(span,"E_PROOF_ASSUMPTION","현재 가정이나 이미 검사한 단계에 속하지 않는 근거입니다"))
}
fn same_sides(from:&AlgebraEquality,to:&AlgebraEquality,count:usize,code:&str)->Result<(),ParseError> {
    if normal(&from.left,count,from.left_span)?!=normal(&to.left,count,to.left_span)? {
        return Err(error(to.left_span,code,"왼쪽 항이 앞선 증거의 왼쪽 항과 항등적으로 같지 않습니다"));
    }
    if normal(&from.right,count,from.right_span)?!=normal(&to.right,count,to.right_span)? {
        return Err(error(to.right_span,code,"오른쪽 항이 앞선 증거의 오른쪽 항과 항등적으로 같지 않습니다"));
    }
    Ok(())
}

/// Observation of this replay only; it is not a checked proof handle.
#[derive(Debug)]
pub struct AlgebraInspection {
    pub checked_steps:Vec<AlgebraEquality>,
    pub reciprocals:Vec<Option<QTerm>>,
    pub failed_step:Option<usize>,
    pub error:Option<ParseError>,
}
pub fn inspect_algebra_definition(context:&ResolvedAlgebraDefinition)->AlgebraInspection {
    let mut view=AlgebraInspection{checked_steps:Vec::new(),reciprocals:Vec::new(),failed_step:None,error:None};
    if let Err(e)=replay(context,&mut view){view.error=Some(e);}
    view
}
pub fn check_algebra_definition(context:&ResolvedAlgebraDefinition)->Result<CheckedAlgebraDefinition,ParseError> {
    let view=inspect_algebra_definition(context);
    if let Some(e)=view.error{return Err(e);}
    Ok(CheckedAlgebraDefinition{context:context.clone()})
}
fn replay(context:&ResolvedAlgebraDefinition,view:&mut AlgebraInspection)->Result<(),ParseError> {
    let span=context.declaration().span;
    if context.assumptions().is_empty(){return Err(error(span,"E_PROOF_ASSUMPTION","명시된 등식 가정이 필요합니다"));}
    if context.parameters().iter().any(|p|p.type_ref!=TypeRef::Named("나눔수".into())||p.optional||p.default_value.is_some()) {
        return Err(error(span,"E_PROOF_ALGEBRA_TYPE","실제 필수 나눔수 입력만 허용합니다"));
    }
    if context.steps().len()>256 {return Err(error(span,"E_PROOF_BUDGET","대수 단계 수 한도를 넘었습니다"));}
    let count=context.parameters().len();
    for assumption in context.assumptions(){validate(assumption,count)?;}
    validate(context.goal(),count)?;
    let mut checked=Vec::new();
    // Check every authored step, including unused ones. A valid final normal
    // form must not hide an earlier invalid inference.
    for (index,step) in context.steps().iter().enumerate() {
        view.failed_step=Some(index);
        let mut reciprocal=None;
        let next=match &step.rule {
            AlgebraRule::SubtractSame{prior,term}=>{
                let prior=lookup(*prior,context,&checked,step.span)?;
                normal(term,count,step.span)?;
                AlgebraEquality{
                    left:QTerm::Subtract(Box::new(prior.left.clone()),Box::new(term.clone())),
                    right:QTerm::Subtract(Box::new(prior.right.clone()),Box::new(term.clone())),
                    left_span:step.span,right_span:step.span,
                }
            }
            AlgebraRule::DivideSameNonzero{prior,coefficient,coefficient_span}=>{
                let prior=lookup(*prior,context,&checked,step.span)?;
                let r=closed_reciprocal(coefficient).map_err(|e|match e {
                    ReciprocalError::NotClosed=>error(*coefficient_span,"E_PROOF_ALGEBRA_COEFFICIENT","계수는 변수를 포함하지 않는 정확 상수여야 합니다"),
                    ReciprocalError::Zero=>error(*coefficient_span,"E_PROOF_ALGEBRA_DIVIDE_ZERO","양변을 0으로 나눌 수 없습니다"),
                    ReciprocalError::Arithmetic(NormalizeError::BudgetExceeded)=>error(*coefficient_span,"E_PROOF_BUDGET","정확 역수 구성 한도를 넘었습니다. 반례 판정이 아닙니다"),
                    ReciprocalError::Arithmetic(_)=>error(*coefficient_span,"E_PROOF_ALGEBRA_DEFINEDNESS","정의되지 않은 정확 계수입니다"),
                })?;
                let product=QTerm::Multiply(Box::new(coefficient.clone()),Box::new(r.clone()));
                if normal(&product,count,*coefficient_span)?!=normal(&QTerm::Integer(1),count,*coefficient_span)? {
                    return Err(error(*coefficient_span,"E_PROOF_ALGEBRA_RECIPROCAL","계수와 역수의 곱이 1이 아닙니다"));
                }
                let next=AlgebraEquality{
                    left:QTerm::Multiply(Box::new(r.clone()),Box::new(prior.left.clone())),
                    right:QTerm::Multiply(Box::new(r.clone()),Box::new(prior.right.clone())),
                    left_span:step.span,right_span:step.span,
                };
                // Replay the same-operation relation independently of the candidate result.
                let expected=AlgebraEquality{
                    left:QTerm::Multiply(Box::new(r.clone()),Box::new(prior.left.clone())),
                    right:QTerm::Multiply(Box::new(r.clone()),Box::new(prior.right.clone())),
                    left_span:step.span,right_span:step.span,
                };
                same_sides(&next,&expected,count,"E_PROOF_ALGEBRA_RECIPROCAL")?;
                reciprocal=Some(r);next
            }
            AlgebraRule::RewriteSides{prior,goal}=>{
                let prior=lookup(*prior,context,&checked,step.span)?;
                same_sides(prior,goal,count,"E_PROOF_ALGEBRA_REWRITE")?;goal.clone()
            }
        };
        validate(&next,count)?;view.checked_steps.push(next.clone());view.reciprocals.push(reciprocal);checked.push(next);
    }
    view.failed_step=None;
    let result=lookup(context.result(),context,&checked,context.result_span())?;
    same_sides(result,context.goal(),count,"E_PROOF_ALGEBRA_RESULT")?;
    Ok(())
}
