//! Source-owned adapter to the existing private definedness producer/checker.
use ddonirang_lang::{conditional_algebra::*,ParseError,Span};
use ddonirang_symbolic::DeclaredAssumptionClosure;
use sha2::{Digest,Sha256};
use std::collections::BTreeSet;
#[derive(Clone,Debug)]
pub struct CheckedDefinition {definition:ResolvedDefinition,retained_sources:Vec<BoundTerm>,pub left:String,pub right:String,pub retained_conditions:Vec<String>,pub used_assumptions:Vec<usize>,pub checked_steps:usize}
impl CheckedDefinition {
 pub fn validate_source(&self,source:&str,path:&str)->Result<(),ParseError>{
  if self.definition.source_matches(source,path){Ok(())}else{Err(error(self.definition.declaration().span,"E_PROOF_SOURCE_CHANGED","조건부 근거의 원문 또는 판본이 바뀌었습니다"))}
 }
}
fn error(span:Span,code:&str,text:&str)->ParseError{ParseError{span,message:format!("{code}: {text}")}}
fn hash(s:&str)->String{format!("{:x}",Sha256::digest(s.as_bytes()))}
pub fn check_definition(context:&ResolvedDefinition)->Result<CheckedDefinition,ParseError>{
 check_definition_cached(context,&mut std::collections::BTreeMap::new())
}
fn check_definition_cached(context:&ResolvedDefinition,checked_definitions:&mut std::collections::BTreeMap<ddonirang_lang::NodeId,CheckedDefinition>)->Result<CheckedDefinition,ParseError>{
 if let Some(checked)=checked_definitions.get(&context.declaration().id){
  if !checked.definition.same_resolution(context){return Err(error(context.declaration().span,"E_PROOF_DEFINEDNESS_OWNER","같은 노드의 실제 보임 해소 문맥이 다릅니다"));}
  return Ok(checked.clone());
 }
 let mut declared=Vec::new();let mut declaration_slots=Vec::new();
 for(i,a)in context.assumptions().iter().enumerate(){if let Assumption::Nonzero(t)=a{
  let text=t.expression_in(context)?;
  declared.push(text.to_string());declaration_slots.push((i,ddonirang_symbolic::canon(text).map_err(|e|error(t.span(),"E_PROOF_DEFINEDNESS_ASSUMPTION",&e))?));
 }}
 let refs=declared.iter().map(String::as_str).collect::<Vec<_>>();
 let closure=DeclaredAssumptionClosure::from_declared_nonzero(&refs).map_err(|e|error(context.declaration().span,"E_PROOF_DEFINEDNESS_ASSUMPTION",&e))?;
 let snapshot=hash(context.source());let provenance=format!("{}:{}..{}",context.path(),context.declaration().span.start,context.declaration().span.end);
 let mut retained=BTreeSet::new();
 let mut normalize=|term:&BoundTerm|->Result<String,ParseError>{
  let text=term.expression_in(context)?;
  let request=crate::DefinednessCheckRequest::internal_candidate(&snapshot,text,closure.clone(),&snapshot,&provenance);
  let checked=crate::execute_definedness_checked_internal(&request).map_err(|e|error(term.span(),"E_PROOF_DEFINEDNESS_REQUIRED",&e))?;
  if !checked.report.verdict{return Err(error(term.span(),"E_PROOF_DEFINEDNESS_CHECK","독립 kernel 검사가 거부했습니다"));}
  retained.extend(checked.problem.transformation_obligations.nonzero_expressions().iter().cloned());
  Ok(checked.certificate.result)
 };
 #[derive(Clone,Debug,PartialEq,Eq)]enum Fact{Nonzero(String),Equality((String,String))}
 let mut carried_conditions=Vec::new();
 let mut inputs=Vec::new();
 for input in context.assumptions(){inputs.push(match input{
  Assumption::Nonzero(t)=>Fact::Nonzero(ddonirang_symbolic::canon(t.expression_in(context)?).map_err(|e|error(t.span(),"E_PROOF_DEFINEDNESS_ASSUMPTION",&e))?),
  Assumption::Equality(eq)=>Fact::Equality((normalize(&eq.left)?,normalize(&eq.right)?))});}
 let mut steps:Vec<(String,String)>=Vec::new();let mut used_evidence=BTreeSet::new();
 fn evidence(r:EvidenceRef,inputs:&[Fact],steps:&[(String,String)],span:Span,used:&mut BTreeSet<usize>)->Result<Fact,ParseError>{
  match r{
   EvidenceRef::Input(i)=>{let value=inputs.get(i).cloned().ok_or_else(||error(span,"E_PROOF_ASSUMPTION","가정 슬롯이 없습니다"))?;used.insert(i);Ok(value)},
   EvidenceRef::Step(i)=>steps.get(i).cloned().map(Fact::Equality).ok_or_else(||error(span,"E_PROOF_ASSUMPTION","앞선 검사 단계가 아닙니다")),
  }
 }
 for step in context.steps(){
  let pair=(normalize(&step.goal.left)?,normalize(&step.goal.right)?);
  match &step.rule{
   Rule::Rewrite{prior}=>match evidence(*prior,&inputs,&steps,step.span,&mut used_evidence)?{
    Fact::Equality(previous)if previous!=pair=>return Err(error(step.span,"E_PROOF_ALGEBRA_REWRITE","각 변은 원래 근거와 같은 조건부 식이어야 합니다")),
    Fact::Nonzero(_)if pair.0!=pair.1=>return Err(error(step.span,"E_PROOF_ALGEBRA_REWRITE","비영 가정만으로 다른 두 식의 등식을 발급하지 않습니다")),_=>{}
   },
   Rule::ApplyDefinition{definition,values,evidence:provided}=>{
    if !definition.source_matches(context.source(),context.path()){return Err(error(step.span,"E_PROOF_SOURCE_CHANGED","사용할 정리의 원문·판본이 다릅니다"));}
    let checked=check_definition_cached(definition,checked_definitions)?;
    // Every supplied value must itself be defined, including unused pins.
    // Normalize the original term so cancellation cannot erase its domain.
    for value in values{normalize(value)?;}
    if provided.len()!=definition.assumptions().len(){return Err(error(step.span,"E_PROOF_ASSUMPTION","정리의 필수 근거가 모두 필요합니다"));}
    for (required,reference)in definition.assumptions().iter().zip(provided){
     let required=match required{
      Assumption::Nonzero(t)=>{let term=definition.instantiate(t,context,values,step.span)?;
       Fact::Nonzero(ddonirang_symbolic::canon(term.expression_in(context)?).map_err(|e|error(step.span,"E_PROOF_DEFINEDNESS_ASSUMPTION",&e))?)},
      Assumption::Equality(eq)=>{let left=definition.instantiate(&eq.left,context,values,step.span)?;let right=definition.instantiate(&eq.right,context,values,step.span)?;
       Fact::Equality((normalize(&left)?,normalize(&right)?))},
     };
     if required!=evidence(*reference,&inputs,&steps,step.span,&mut used_evidence)?{return Err(error(step.span,"E_PROOF_DEFINEDNESS_OWNER","정리의 실제 값 입력과 명시 근거가 같은 소유 문맥에 결속되지 않습니다"));}
    }
    let left=definition.instantiate(&definition.goal().left,context,values,step.span)?;let right=definition.instantiate(&definition.goal().right,context,values,step.span)?;
    if pair!=(normalize(&left)?,normalize(&right)?){return Err(error(step.span,"E_PROOF_CONTEXT_MISMATCH","호출된 checked 정리의 결과와 단계 결과가 다릅니다"));}
    let mut typed_conditions=BTreeSet::new();
    for term in &checked.retained_sources{
     typed_conditions.insert(ddonirang_symbolic::canon(term.expression_in(definition)?).map_err(|e|error(step.span,"E_PROOF_DEFINEDNESS_ASSUMPTION",&e))?);
     let term=definition.instantiate(term,context,values,step.span)?;
     carried_conditions.push(ddonirang_symbolic::canon(term.expression_in(context)?).map_err(|e|error(step.span,"E_PROOF_DEFINEDNESS_ASSUMPTION",&e))?);
    }
    if typed_conditions!=checked.retained_conditions.iter().cloned().collect(){return Err(error(step.span,"E_PROOF_DEFINEDNESS_OWNER","원래 제외조건의 실제 원문 근거가 완결되지 않았습니다"));}
   },
  }
  steps.push(pair);
 }
 let Fact::Equality(result)=evidence(context.result(),&inputs,&steps,context.result_span(),&mut used_evidence)?else{return Err(error(context.result_span(),"E_PROOF_NONZERO_INPUT","비영 입력은 등식 결과가 아닙니다"));};
 let goal=(normalize(&context.goal().left)?,normalize(&context.goal().right)?);
 if result!=goal{return Err(error(context.result_span(),"E_PROOF_CONTEXT_MISMATCH","최종 조건부 등식과 반환 목표가 다릅니다"));}
 drop(normalize);
 retained.extend(carried_conditions);
 let mut retained_sources=Vec::new();
 for input in context.assumptions(){if let Assumption::Nonzero(term)=input{let normalized=ddonirang_symbolic::canon(term.expression_in(context)?).map_err(|e|error(term.span(),"E_PROOF_DEFINEDNESS_ASSUMPTION",&e))?;if retained.contains(&normalized){retained_sources.push(term.clone());}}}
 used_evidence.extend(declaration_slots.into_iter().filter_map(|(i,t)|retained.contains(&t).then_some(i)));
 let used_assumptions=used_evidence.into_iter().collect();
 let checked=CheckedDefinition{definition:context.clone(),retained_sources,left:result.0,right:result.1,retained_conditions:retained.into_iter().collect(),used_assumptions,checked_steps:steps.len()};
 checked_definitions.insert(context.declaration().id,checked.clone());Ok(checked)
}
