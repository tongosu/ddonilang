//! Source-owned candidates for explicit nonzero conditional algebra.
//! Constructed only from the prepared common-parser program, never JSON or text assumptions.
use crate::{ast::*,ParseError};
use std::{collections::HashMap,sync::Arc};
#[derive(Clone,Debug)]
pub struct BoundTerm { owner:Arc<()>, expression:String, source_text:String, span:Span, syntax:BoundSyntax }
#[derive(Clone,Debug)]
enum BoundSyntax {Integer(i64),Parameter(usize),Binary(char,Box<BoundSyntax>,Box<BoundSyntax>)}
impl BoundSyntax {
 fn from_term(t:&AlgebraTermSyntax,names:&HashMap<String,usize>,budget:&mut usize,depth:usize)->Result<Self,ParseError>{
  if *budget==0{return Err(fail(t.span,"E_PROOF_BUDGET","조건부 수식 한도를 넘었습니다"));}*budget-=1;
  Ok(match &t.kind{
   AlgebraTermKind::Integer(v)=>Self::Integer(*v),
   AlgebraTermKind::Name(n)=>Self::Parameter(*names.get(n).ok_or_else(||fail(t.span,"E_PROOF_ALGEBRA_BINDER","같은 정의의 실제 나눔수 입력이 아닙니다"))?),
   AlgebraTermKind::DivideByInteger(a,b)=>Self::Binary('/',Box::new(Self::from_term(a,names,budget,depth+1)?),Box::new(Self::Integer(*b))),
   AlgebraTermKind::Add(a,b)|AlgebraTermKind::Subtract(a,b)|AlgebraTermKind::Multiply(a,b)|AlgebraTermKind::RationalDivide(a,b)=>{
    let op=match &t.kind{AlgebraTermKind::Add(..)=>'+',AlgebraTermKind::Subtract(..)=>'-',AlgebraTermKind::Multiply(..)=>'*',_=>'/'};
    Self::Binary(op,Box::new(Self::from_term(a,names,budget,depth+1)?),Box::new(Self::from_term(b,names,budget,depth+1)?))},
  })
 }
 fn render(&self,name:&impl Fn(usize)->String)->String{match self{Self::Integer(v)=>v.to_string(),Self::Parameter(i)=>name(*i),Self::Binary(op,a,b)=>format!("({}{}{})",a.render(name),op,b.render(name))}}
 fn substitute(&self,values:&[BoundSyntax],replace:bool,budget:&mut usize,depth:usize,span:Span)->Result<Self,ParseError>{
  if *budget==0||depth>64{return Err(fail(span,"E_PROOF_BUDGET","보임 인수 전개 한도를 넘었습니다"));}*budget-=1;
  Ok(match self{
   Self::Parameter(i)if replace=>values.get(*i).ok_or_else(||fail(span,"E_PROOF_ARGUMENT_BINDING","실제 값 받는자리의 인수가 없습니다"))?.substitute(values,false,budget,depth+1,span)?,
   Self::Binary(op,a,b)=>Self::Binary(*op,Box::new(a.substitute(values,replace,budget,depth+1,span)?),Box::new(b.substitute(values,replace,budget,depth+1,span)?)),
   other=>other.clone(),
  })
 }
}
impl BoundTerm {
 pub fn expression_in<'a>(&'a self,c:&ResolvedDefinition)->Result<&'a str,ParseError>{
  if !Arc::ptr_eq(&self.owner,&c.0.owner){return Err(fail(self.span,"E_PROOF_DEFINEDNESS_OWNER","수식과 가정의 실제 정의 소유자가 다릅니다"));}
  Ok(&self.expression)
 }
 pub fn span(&self)->Span{self.span}
 pub fn source_text(&self)->&str{&self.source_text}
}
#[derive(Clone,Debug)]
pub struct Equality {pub left:BoundTerm,pub right:BoundTerm}
#[derive(Clone,Debug)]
pub enum Assumption {Nonzero(BoundTerm),Equality(Equality)}
#[derive(Clone,Copy,Debug)]
pub enum EvidenceRef {Input(usize),Step(usize)}
#[derive(Clone,Debug)]
pub enum Rule {Rewrite{prior:EvidenceRef},ApplyDefinition{definition:ResolvedDefinition,values:Vec<BoundTerm>,evidence:Vec<EvidenceRef>}}
#[derive(Clone,Debug)]
pub struct Step {pub rule:Rule,pub goal:Equality,pub span:Span}
#[derive(Debug)]
struct Definition {source:String,path:String,seed:SeedDef,owner:Arc<()>,call_depth:usize,parameters:Vec<ParamPin>,assumptions:Vec<Assumption>,goal:Equality,steps:Vec<Step>,result:EvidenceRef,result_span:Span}
#[derive(Clone,Debug)]
pub struct ResolvedDefinition(Arc<Definition>);
impl ResolvedDefinition {
 pub fn source_matches(&self,s:&str,p:&str)->bool{self.0.source==s&&self.0.path==p}
 pub fn same_resolution(&self,other:&Self)->bool{Arc::ptr_eq(&self.0,&other.0)}
 pub fn declaration(&self)->&SeedDef{&self.0.seed}
 pub fn parameters(&self)->&[ParamPin]{&self.0.parameters}
 pub fn assumptions(&self)->&[Assumption]{&self.0.assumptions}
 pub fn goal(&self)->&Equality{&self.0.goal}
 pub fn steps(&self)->&[Step]{&self.0.steps}
 pub fn result(&self)->EvidenceRef{self.0.result}
 pub fn result_span(&self)->Span{self.0.result_span}
 pub fn source(&self)->&str{&self.0.source}
 pub fn path(&self)->&str{&self.0.path}
 pub fn instantiate(&self,term:&BoundTerm,consumer:&ResolvedDefinition,values:&[BoundTerm],span:Span)->Result<BoundTerm,ParseError>{
  term.expression_in(self)?;
  if !self.source_matches(consumer.source(),consumer.path())||values.len()!=self.parameters().len(){return Err(fail(span,"E_PROOF_SOURCE_CHANGED","정리의 원문·판본·입력 소유가 다릅니다"));}
  for value in values{value.expression_in(consumer)?;}
  let syntax=term.syntax.substitute(&values.iter().map(|v|v.syntax.clone()).collect::<Vec<_>>(),true,&mut 4096,0,span)?;
  Ok(BoundTerm{owner:consumer.0.owner.clone(),expression:syntax.render(&|i|format!("p{i}")),
   source_text:syntax.render(&|i|consumer.parameters().get(i).map(|p|p.pin_name.clone()).unwrap_or_default()),span,syntax})
 }
}
fn fail(span:Span,code:&str,message:&str)->ParseError{ParseError{span,message:format!("{code}: {message}")}}
fn rational(t:&AlgebraTermSyntax)->bool{match &t.kind{
 AlgebraTermKind::RationalDivide(..)=>true,
 AlgebraTermKind::Add(a,b)|AlgebraTermKind::Subtract(a,b)|AlgebraTermKind::Multiply(a,b)=>rational(a)||rational(b),
 AlgebraTermKind::DivideByInteger(a,_)=>rational(a),_=>false}}
fn conditional_type(t:&TypeRef)->bool{match t{
 TypeRef::AlgebraNonzero{..}=>true,TypeRef::AlgebraEquality{left,right}=>rational(left)||rational(right),
 TypeRef::Applied{args,..}=>args.iter().any(conditional_type),_=>false}}
pub(crate) fn is_definition(seed:&SeedDef)->bool{seed.params.iter().any(|p|conditional_type(&p.type_ref))||seed.result_type.as_ref().is_some_and(conditional_type)}
pub(crate) fn has_contract(program:&CanonProgram)->bool{program.items.iter().any(|TopLevelItem::SeedDef(s)|is_definition(s))}
fn proof_type(t:&TypeRef)->Option<&TypeRef>{match t{TypeRef::Applied{name,args}if name=="보임"&&args.len()==1=>Some(&args[0]),_=>None}}
fn bind(t:&AlgebraTermSyntax,names:&HashMap<String,usize>,owner:&Arc<()>)->Result<BoundTerm,ParseError>{
 let syntax=BoundSyntax::from_term(t,names,&mut 4096,0)?;
 Ok(BoundTerm{owner:owner.clone(),expression:syntax.render(&|i|format!("p{i}")),source_text:t.source_text(),span:t.span,syntax})
}
fn equality(t:&TypeRef,n:&HashMap<String,usize>,o:&Arc<()>,span:Span)->Result<Equality,ParseError>{
 let Some(TypeRef::AlgebraEquality{left,right})=proof_type(t)else{return Err(fail(span,"E_PROOF_NONZERO_INPUT","비영은 명시 입력으로만 사용합니다. 결과에는 등식 보임이 필요합니다"));};
 Ok(Equality{left:bind(left,n,o)?,right:bind(right,n,o)?})
}
pub(crate) fn resolve(program:&CanonProgram,source:&str,path:&str,project:bool)->Result<Vec<ResolvedDefinition>,ParseError>{
 if !program.root_stmts.is_empty()||!program.origin.authored_jjaim_blocks.is_empty(){return Err(fail(Span::new(0,source.len()),"E_PROOF_ROOT_UNSUPPORTED","조건부 검사에서 실행 문장을 무시하지 않습니다"));}
 if program.items.len()>256{return Err(fail(Span::new(0,source.len()),"E_PROOF_BUDGET","정의 한도를 넘었습니다"));}
 if program.items.iter().any(|TopLevelItem::SeedDef(s)|s.canonical_name=="나눔수"){return Err(fail(Span::new(0,source.len()),"E_PROOF_ALGEBRA_TYPE","기본 나눔수가 다른 선언에 가려졌습니다"));}
 let mut out:Vec<ResolvedDefinition>=Vec::new();
 for TopLevelItem::SeedDef(seed)in &program.items{
  if !is_definition(seed){
   if !project && !seed.result_type.as_ref().is_some_and(crate::algebra_authoring::is_goal){return Err(fail(seed.span,"E_PROOF_DEFINITION","검사하지 않은 일반 정의를 조건부 수용에 포함하지 않습니다"));}
   continue;
  }
  if let Some(pin)=seed.params.iter().find(|p|p.pin_name=="나눔수"){return Err(fail(pin.span,"E_PROOF_ALGEBRA_TYPE","기본 나눔수가 입력 이름에 가려졌습니다"));}
  if seed.seed_kind!=SeedKind::Semssi||!seed.modifiers.is_empty()||!seed.type_params.is_empty()||seed.proof_header.as_ref().is_some_and(|h|!h.propositions.is_empty()){return Err(fail(seed.span,"E_PROOF_DEFINITION","명시된 필수 입력의 조건부 셈씨가 필요합니다"));}
  let owner=Arc::new(());let mut names=HashMap::new();let mut parameters=Vec::new();let mut all=std::collections::HashSet::new();all.insert(seed.canonical_name.clone());
  for pin in &seed.params{
   if pin.optional||pin.default_value.is_some()||!all.insert(pin.pin_name.clone()){return Err(fail(pin.span,"E_PROOF_ALGEBRA_BINDER","중복 없는 필수 입력이 필요합니다"));}
   if pin.type_ref==TypeRef::Named("나눔수".into()){names.insert(pin.pin_name.clone(),parameters.len());parameters.push(pin.clone());}
   else if !matches!(proof_type(&pin.type_ref),Some(TypeRef::AlgebraNonzero{..}|TypeRef::AlgebraEquality{..})){return Err(fail(pin.span,"E_PROOF_NONZERO_INPUT","나눔수와 명시 비영/등식 증거만 허용합니다. Bool은 증거가 아닙니다"));}
  }
  let mut assumptions=Vec::new();let mut locals=HashMap::new();
  for pin in &seed.params{
   let input=match proof_type(&pin.type_ref){
    Some(TypeRef::AlgebraNonzero{term})=>Some(Assumption::Nonzero(bind(term,&names,&owner)?)),
    Some(TypeRef::AlgebraEquality{..})=>Some(Assumption::Equality(equality(&pin.type_ref,&names,&owner,pin.span)?)),_=>None};
   if let Some(input)=input{locals.insert(pin.pin_name.clone(),EvidenceRef::Input(assumptions.len()));assumptions.push(input);}
  }
  let goal=equality(seed.result_type.as_ref().ok_or_else(||fail(seed.span,"E_PROOF_RESULT","등식 결과가 필요합니다"))?,&names,&owner,seed.span)?;
  let body=seed.body.as_ref().ok_or_else(||fail(seed.span,"E_PROOF_RESULT","조건부 본문이 필요합니다"))?;
  if body.stmts.len()>256{return Err(fail(body.span,"E_PROOF_BUDGET","단계 한도를 넘었습니다"));}
  let mut steps=Vec::new();let mut result=None;let mut result_span=body.span;
  for(i,stmt)in body.stmts.iter().enumerate(){
   let(value,local)=match stmt{
    Stmt::Definition{name,type_ref:TypeRef::Infer,value,reaction:None,..}=>{if !all.insert(name.clone()){return Err(fail(value.span,"E_PROOF_ALGEBRA_BINDER","단계 이름이 중복됩니다"));}(value,Some(name))},
    Stmt::Mutate{target,value,deferred:false,..}if matches!(&target.kind,ExprKind::Var(n)if n==&seed.canonical_name)&&i+1==body.stmts.len()=>(value,None),
    _=>return Err(fail(body.span,"E_PROOF_RULE_UNSUPPORTED","지역 검사 단계와 마지막 결과 기록만 허용합니다"))};
   let reference=if let ExprKind::Var(n)=&value.kind{*locals.get(n).ok_or_else(||fail(value.span,"E_PROOF_ASSUMPTION","같은 정의의 앞선 근거가 아닙니다"))?}else{
    let ExprKind::Call{args,..}=&value.kind else{return Err(fail(value.span,"E_PROOF_NONZERO_INPUT","Bool·비교값을 증거로 사용할 수 없습니다"));};
    if program.origin.algebra_operation(value.id).is_some_and(|b|b.definition==seed.id&&b.operation==StaticAlgebraOperation::RewriteSides){
    let field=|n:&str|args.iter().find(|a|a.resolved_pin.as_deref()==Some(n)).map(|a|&a.expr).ok_or_else(||fail(value.span,"E_PROOF_ALGEBRA_PIN","필수 입력이 없습니다"));
    let prior=field("근거")?;let ExprKind::Var(n)=&prior.kind else{return Err(fail(prior.span,"E_PROOF_DEFINEDNESS_OWNER","같은 정의의 명시 증거 입력이나 앞선 단계가 필요합니다"));};
    let prior=*locals.get(n).ok_or_else(||fail(prior.span,"E_PROOF_DEFINEDNESS_OWNER","다른 소유자의 증거는 사용할 수 없습니다"))?;
    let left=crate::algebra_syntax::term(field("왼")?)?;let right=crate::algebra_syntax::term(field("오른")?)?;
    steps.push(Step{rule:Rule::Rewrite{prior},goal:Equality{left:bind(&left,&names,&owner)?,right:bind(&right,&names,&owner)?},span:value.span});
    }else{
     let ExprKind::Call{func,..}=&value.kind else{unreachable!()};
     if program.origin.algebra_operation(value.id).is_some(){return Err(fail(value.span,"E_PROOF_RULE_UNSUPPORTED","조건부 경계에서 미지원 대수 조작입니다"));}
     if all.contains(func){return Err(fail(value.span,"E_PROOF_CALL_SHADOWED","지역 선언에 가려진 보임 호출입니다"));}
     let candidates=program.origin.authored_call_candidates.get(&value.id).ok_or_else(||fail(value.span,"E_PROOF_CALL_UNRESOLVED","실제 보임 정의가 해소되지 않았습니다"))?;
     let [id]=candidates.as_slice()else{return Err(fail(value.span,"E_PROOF_CALL_AMBIGUOUS","보임 정의 소유가 유일하지 않습니다"));};
     let definition=out.iter().find(|d|d.declaration().id==*id).cloned().ok_or_else(||fail(value.span,"E_PROOF_CALL_UNSUPPORTED","앞선 비재귀 조건부 보임 정의가 필요합니다"))?;
     if definition.declaration().callable_surface_names().iter().any(|n|all.contains(n)){return Err(fail(value.span,"E_PROOF_CALL_SHADOWED","같은 실제 정의의 호출 표면이 가려졌습니다"));}
     if definition.0.call_depth>=64{return Err(fail(value.span,"E_PROOF_BUDGET","보임 호출 깊이를 넘었습니다"));}
     let pins=&definition.declaration().params;
     if args.len()!=pins.len(){return Err(fail(value.span,"E_PROOF_ARGUMENT_COUNT","필수 값과 명시 근거를 모두 공급하세요"));}
     let mut seen=std::collections::HashSet::new();
     for arg in args{
      let name=arg.resolved_pin.as_deref().ok_or_else(||fail(arg.span,"E_PROOF_ARGUMENT_BINDING","공통 해소자의 실제 받는자리 결속이 필요합니다"))?;
      if !pins.iter().any(|p|p.pin_name==name)||!seen.insert(name){return Err(fail(arg.span,"E_PROOF_ARGUMENT_BINDING","다른 정의 또는 중복된 받는자리입니다"));}
     }
     let mut values=Vec::new();let mut evidence=Vec::new();
     for pin in pins{
      let arg=args.iter().find(|a|a.resolved_pin.as_deref()==Some(pin.pin_name.as_str())).ok_or_else(||fail(value.span,"E_PROOF_ARGUMENT_BINDING","필수 받는자리가 없습니다"))?;
      if pin.type_ref==TypeRef::Named("나눔수".into()){values.push(bind(&crate::algebra_syntax::term(&arg.expr)?,&names,&owner)?);}
      else{let ExprKind::Var(n)=&arg.expr.kind else{return Err(fail(arg.expr.span,"E_PROOF_DEFINEDNESS_OWNER","같은 정의의 명시 근거나 앞선 단계가 필요합니다. Bool은 근거가 아닙니다"));};
       evidence.push(*locals.get(n).ok_or_else(||fail(arg.expr.span,"E_PROOF_DEFINEDNESS_OWNER","다른 소유자의 근거입니다"))?);}
     }
     // The checked call's goal is instantiated only after its actual value
     // pins are sealed. Terms remain bound to this consumer's source owner.
     let substitute=|term:&BoundTerm|->Result<BoundTerm,ParseError>{
      term.expression_in(&definition)?;let syntax=term.syntax.substitute(&values.iter().map(|v|v.syntax.clone()).collect::<Vec<_>>(),true,&mut 4096,0,value.span)?;
      Ok(BoundTerm{owner:owner.clone(),expression:syntax.render(&|i|format!("p{i}")),source_text:syntax.render(&|i|parameters.get(i).map(|p|p.pin_name.clone()).unwrap_or_default()),span:value.span,syntax})};
     let goal=Equality{left:substitute(&definition.goal().left)?,right:substitute(&definition.goal().right)?};
     steps.push(Step{rule:Rule::ApplyDefinition{definition,values,evidence},goal,span:value.span});
    }
    EvidenceRef::Step(steps.len()-1)
   };
   if let Some(n)=local{locals.insert(n.clone(),reference);}else{result=Some(reference);result_span=value.span;}
  }
  let call_depth=steps.iter().filter_map(|s|match &s.rule{Rule::ApplyDefinition{definition,..}=>Some(definition.0.call_depth+1),_=>None}).max().unwrap_or(0);
  out.push(ResolvedDefinition(Arc::new(Definition{source:source.into(),path:path.into(),seed:seed.clone(),owner,call_depth,parameters,assumptions,goal,steps,result:result.ok_or_else(||fail(body.span,"E_PROOF_RESULT","마지막 결과가 없습니다"))?,result_span})));
 }
 Ok(out)
}
