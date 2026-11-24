//! Typed model-to-RK4 adapter. Public recipe admission is a separate boundary.
//! Reads real parsed owners/units/derivatives; does not rewrite source or issue identities.
use super::*;
use ddonirang_core::solver_rk4;
use ddonirang_core::signals::ArithmeticFaultKind;
use ddonirang_lang::model::LoweredRelationNode;

#[derive(Debug, Clone)]
struct Quantity { value:Fixed64, dimension:UnitDim }
#[derive(Debug)]
pub(super) struct TimeSample {pub step:u64,pub time:Fixed64,pub environment:BTreeMap<String,ExactQuantity>}
#[derive(Debug)]
pub(super) struct TimeRun {
    pub outcome:solver_rk4::Outcome,
    pub samples:Vec<TimeSample>,
    pub terminal:BTreeMap<String,ExactQuantity>,
    pub definition_identity:String,
    pub request_identity:String,
    pub source_hash:String,
    pub axis_owner:String,
    pub state_owners:Vec<String>,
}
pub(super) struct TimeModel<'a> {
    program:&'a ModelProgram,
    definition:&'a ModelDefinition,
    request:&'a RunRequest,
    axis:&'a ModelDeclaration,
    states:Vec<&'a ModelDeclaration>,
    algebraic:Vec<(&'a str,&'a LoweredRelationNode)>,
    derivatives:Vec<&'a LoweredRelationNode>,
    base:BTreeMap<String,Quantity>,
    initial:Vec<Fixed64>,
    context:&'a ReceiptContext,
}
fn time_error(message:impl Into<String>,span:ModelSpan,c:&ReceiptContext)->ModelExecutionError {
    failure("E_MODEL_TIME_SHAPE_UNSUPPORTED","model_time_binding",message,span,c)
}
fn checked(value:Result<Fixed64,ArithmeticFaultKind>,span:ModelSpan,c:&ReceiptContext)->Result<Fixed64,ModelExecutionError>{
    value.map_err(|fault|match fault {
        ArithmeticFaultKind::DivByZero=>arithmetic_failure("E_MODEL_DIVISION_BY_ZERO_CANDIDATE","0으로 나눌 수 없습니다",span,c),
        _=>arithmetic_failure("E_MODEL_ARITHMETIC_OVERFLOW_CANDIDATE","Fixed64 연산 범위를 벗어났습니다",span,c),
    })
}
fn dimension(left:UnitDim,right:UnitDim,subtract:bool,span:ModelSpan,c:&ReceiptContext)->Result<UnitDim,ModelExecutionError>{
    let mut values=[0;UnitDim::AXIS_COUNT];
    for (index,(a,b)) in left.exponents().into_iter().zip(right.exponents()).enumerate(){
        values[index]=if subtract{a.checked_sub(b)}else{a.checked_add(b)}
            .ok_or_else(||time_error("단위 지수 범위를 벗어났습니다",span,c))?;
    }
    Ok(UnitDim::from_exponents(values))
}
fn admit(value:BigRational,span:ModelSpan,c:&ReceiptContext)->Result<Fixed64,ModelExecutionError>{
    let scaled=value*BigInt::from(Fixed64::ONE_RAW);
    let numerator=scaled.numer().abs();let denominator=scaled.denom();
    let mut quotient=&numerator/denominator;let remainder=&numerator%denominator;
    let twice=&remainder*2;
    if twice>*denominator || (twice==*denominator && (&quotient%2)!=BigInt::zero()){quotient+=1;}
    if scaled.is_negative(){quotient=-quotient;}
    quotient.to_i64().map(Fixed64::from_raw_i64).ok_or_else(||arithmetic_failure(
        "E_MODEL_SOLVER_CONVERSION_UNREPRESENTABLE","정확한 입력을 Solver Fixed64로 변환할 수 없습니다",span,c))
}
fn evaluate(node:&ModelExprNode,env:&BTreeMap<String,Quantity>,span:ModelSpan,c:&ReceiptContext)->Result<Quantity,ModelExecutionError>{
    charge_model_dispatch(c,span)?;
    match node {
        ModelExprNode::Number{surface,unit,dimension}=>{
            let value=parse_decimal_rational(surface).map_err(|e|time_error(e,span,c))?;
            let scale=unit_scale(unit.as_deref()).map_err(|e|time_error(e,span,c))?;
            Ok(Quantity{value:admit(value*scale,span,c)?,dimension:*dimension})
        }
        ModelExprNode::Path(path)=>env.get(path).cloned().ok_or_else(||time_error(format!("현재 모형에서 준비되지 않은 값: {path}"),span,c)),
        ModelExprNode::Group(inner)=>evaluate(inner,env,span,c),
        ModelExprNode::Unary{operator,operand}=>{
            let mut q=evaluate(operand,env,span,c)?;
            match operator.as_str(){"+"=>{},"-"=>q.value=checked(Fixed64::ZERO.checked_sub(q.value),span,c)?,_=>return Err(time_error("지원하지 않는 단항 연산",span,c))};Ok(q)
        }
        ModelExprNode::Binary{operator,left,right}=>{
            let a=evaluate(left,env,span,c)?;let b=evaluate(right,env,span,c)?;
            let (value,dim)=match operator.as_str(){
                "+"|"-"=>{
                    if a.dimension!=b.dimension{return Err(arithmetic_failure("E_MODEL_UNIT_MISMATCH_CANDIDATE","덧셈·뺄셈 단위가 다릅니다",span,c));}
                    (checked(if operator=="+"{a.value.checked_add(b.value)}else{a.value.checked_sub(b.value)},span,c)?,a.dimension)
                }
                "*"=>(checked(a.value.checked_mul(b.value),span,c)?,dimension(a.dimension,b.dimension,false,span,c)?),
                "/"=>(checked(a.value.try_div(b.value),span,c)?,dimension(a.dimension,b.dimension,true,span,c)?),
                "^"=>{
                    if b.dimension!=UnitDim::NONE || b.value.raw_i64()%Fixed64::ONE_RAW!=0{return Err(time_error("지수는 단위 없는 정수여야 합니다",span,c));}
                    let exponent=b.value.raw_i64()/Fixed64::ONE_RAW;
                    if exponent.unsigned_abs()>16{return Err(time_error("이 시간 모형의 정수 지수 한도를 벗어났습니다",span,c));}
                    let mut value=Fixed64::ONE;let mut dim=UnitDim::NONE;
                    for _ in 0..exponent.unsigned_abs(){value=checked(value.checked_mul(a.value),span,c)?;dim=dimension(dim,a.dimension,false,span,c)?;}
                    if exponent<0{value=checked(Fixed64::ONE.try_div(value),span,c)?;dim=dimension(UnitDim::NONE,dim,true,span,c)?;}
                    (value,dim)
                }
                _=>return Err(time_error("이 시간 모형에서 지원하지 않는 연산",span,c)),
            };Ok(Quantity{value,dimension:dim})
        }
        _=>Err(time_error("시간 미분식은 해소된 값과 산술 관계여야 합니다",span,c)),
    }
}
fn expression(expr:&ModelExpr,env:&BTreeMap<String,Quantity>,c:&ReceiptContext)->Result<Quantity,ModelExecutionError>{
    evaluate(expr.node.as_ref().ok_or_else(||time_error("실제 typed 식이 없습니다",expr.span,c))?,env,expr.span,c)
}
fn validate_domain(q:&Quantity,decl:&ModelDeclaration,c:&ReceiptContext)->Result<(),ModelExecutionError>{
    let quantity=ExactQuantity{base_value:BigRational::new(BigInt::from(q.value.raw_i64()),BigInt::from(Fixed64::ONE_RAW)),dimension:q.dimension,numeric_kind:NumericKind::FixedQ32,
        authored_scale:unit_scale(decl.model_type.unit.as_deref()).map_err(|e|time_error(e,decl.identity.span,c))?,authored_unit:decl.model_type.unit.clone()};
    validate_exact_type_value(decl,&quantity,decl.identity.span,c)
}
fn exact_value(expr:&ModelExpr,decl:&ModelDeclaration,env:&BTreeMap<String,ExactQuantity>,c:&ReceiptContext)->Result<ExactQuantity,ModelExecutionError>{
    let node=expr.node.as_ref().ok_or_else(||time_error("입력의 typed 식이 없습니다",expr.span,c))?;
    let mut q=eval_node(node,env,Some(NumericKind::Exact),expr.span,c)?;
    if q.dimension!=decl.model_type.dimension.unwrap_or(UnitDim::NONE){return Err(arithmetic_failure("E_MODEL_UNIT_MISMATCH_CANDIDATE","입력·초기조건의 단위가 선언과 다릅니다",expr.span,c));}
    numeric_kind(decl).map_err(|e|time_error(e,expr.span,c))?;
    validate_exact_type_value(decl,&q,expr.span,c)?;
    q.authored_scale=unit_scale(decl.model_type.unit.as_deref()).map_err(|e|time_error(e,expr.span,c))?;
    q.authored_unit=decl.model_type.unit.clone();Ok(q)
}
impl<'a> TimeModel<'a> {
    pub(super) fn prepare(program:&'a ModelProgram,request:&'a RunRequest,c:&'a ReceiptContext)->Result<Self,ModelExecutionError>{
        if !program.ir.requests.iter().any(|r|r.identity==request.identity){return Err(time_error("다른 모형 원문의 실행 요청입니다",request.span,c));}
        if let Some(d)=program.ir.diagnostics.first(){
            // The common source analysis already identified this unit failure.
            // Keep its code, phase and source range; only unsupported shapes use time_error.
            if d.candidate_code == "E_MODEL_UNIT_MISMATCH_CANDIDATE" {
                return Err(failure(d.candidate_code,d.phase,d.message.clone(),d.provenance,c));
            }
            return Err(time_error(d.message.clone(),d.provenance,c));
        }
        let definition=program.ir.definitions.iter().find(|d|Some(d.name.as_str())==request.instance_binding.as_deref())
            .ok_or_else(||time_error("요청이 실제 모형 정의에 결속되지 않았습니다",request.span,c))?;
        if !definition.events.is_empty() || !definition.connections.is_empty(){return Err(time_error("사건·충돌·연결 모형은 이 시간 실행 범위 밖입니다",definition.span,c));}
        let axes=definition.declarations.iter().filter(|d|d.role==PrimaryRole::Axis).collect::<Vec<_>>();
        if axes.len()!=1 || axes[0].model_type.dimension!=Some(UnitDim::TIME){return Err(time_error("명시한 시간 차원의 축 하나가 필요합니다",definition.span,c));}
        let axis=axes[0];let states=definition.declarations.iter().filter(|d|d.role==PrimaryRole::Continuous).collect::<Vec<_>>();
        if states.is_empty() || definition.declarations.iter().any(|d|d.role==PrimaryRole::Discrete){return Err(time_error("직접 1차 미분의 이어감 상태가 필요합니다",definition.span,c));}
        let declarations=definition.declarations.iter().map(|d|(d.name.as_str(),d)).collect::<BTreeMap<_,_>>();
        let mut exact=BTreeMap::new();
        for binding in &program.ir.exact_prelude_bindings{
            let node=binding.value.node.as_ref().ok_or_else(||time_error("자료 값의 typed 식이 없습니다",binding.span,c))?;
            exact.insert(binding.name.clone(),eval_node(node,&exact,Some(NumericKind::Exact),binding.span,c)?);
        }
        for decl in &definition.declarations{
            match &decl.initialization{
                Initialization::Fixed(expr)=>{exact.insert(decl.name.clone(),exact_value(expr,decl,&exact,c)?);}
                Initialization::Guess(_)|Initialization::Computed=>return Err(time_error("어림·계산 초기조건은 이 직접 시간 모형에서 지원하지 않습니다",decl.identity.span,c)),
                Initialization::None=>{},
            }
        }
        for field in &request.input_bindings{
            let decl=declarations.get(field.name.as_str()).ok_or_else(||time_error("다른 소유자의 들임입니다",field.span,c))?;
            if decl.role!=PrimaryRole::Input || decl.unknown{return Err(time_error("이 선언은 들임 권한이 없습니다",field.span,c));}
            exact.insert(field.name.clone(),exact_value(&field.value,decl,&exact,c)?);
        }
        for field in &request.initial_overrides{
            let decl=declarations.get(field.name.as_str()).ok_or_else(||time_error("다른 소유자의 처음 값입니다",field.span,c))?;
            if !decl.accepts_initial_override || decl.unknown{return Err(time_error("이 선언은 처음받음 권한이 없습니다",field.span,c));}
            exact.insert(field.name.clone(),exact_value(&field.value,decl,&exact,c)?);
        }
        let mut base=BTreeMap::new();
        for decl in &definition.declarations{
            if matches!(decl.role,PrimaryRole::Axis|PrimaryRole::Continuous|PrimaryRole::Input){
                let q=exact.get(&decl.name).ok_or_else(||time_error(format!("초기 값이 없는 선언: {}",decl.name),decl.identity.span,c))?;
                base.insert(decl.name.clone(),Quantity{value:admit(q.base_value.clone(),decl.identity.span,c)?,dimension:q.dimension});
            }
        }
        let mut remaining=BTreeMap::new();let mut rules=BTreeMap::new();
        for relation in &definition.relations{
            let lowered=relation.lowered.as_ref().ok_or_else(||time_error("직접 해소되지 않은 미분·대수 관계입니다",relation.identity.span,c))?;
            match &lowered.target{
                LoweredRelationTarget::Derivative(id)=>{
                    if id.order!=1 || id.axis.as_deref()!=Some(axis.name.as_str()) || !states.iter().any(|d|d.name==id.base){return Err(time_error("현재 축·이어감 소유자의 직접 1차 미분이어야 합니다",relation.identity.span,c));}
                    if rules.insert(id.base.as_str(),lowered).is_some(){return Err(time_error("같은 상태에 미분 관계가 중복됐습니다",relation.identity.span,c));}
                }
                LoweredRelationTarget::Algebraic(name)=>{
                    let decl=declarations.get(name.as_str()).ok_or_else(||time_error("대수 target의 선언 소유자가 없습니다",relation.identity.span,c))?;
                    if decl.role!=PrimaryRole::Algebraic || !decl.unknown || remaining.insert(name.as_str(),lowered).is_some(){return Err(time_error("대수 관계 소유·중복을 확인하세요",relation.identity.span,c));}
                }
            }
        }
        let mut known=base.keys().map(String::as_str).collect::<BTreeSet<_>>();let mut algebraic=Vec::new();
        while !remaining.is_empty(){
            let ready=remaining.iter().find(|(_,r)|r.dependencies.iter().all(|d|known.contains(d.as_str()))).map(|(n,_)|*n)
                .ok_or_else(||time_error("대수 관계가 순환하거나 현재 모형 밖의 값을 참조합니다",definition.span,c))?;
            let rule=remaining.remove(ready).expect("selected remaining target");known.insert(ready);algebraic.push((ready,rule));
        }
        let derivatives=states.iter().map(|d|rules.remove(d.name.as_str()).ok_or_else(||time_error(format!("미분 관계가 없는 상태: {}",d.name),d.identity.span,c))).collect::<Result<Vec<_>,_>>()?;
        let initial=states.iter().map(|d|base[&d.name].value).collect();
        Ok(Self{program,definition,request,axis,states,algebraic,derivatives,base,initial,context:c})
    }
    pub(super) fn initial_time(&self)->Fixed64{self.base[&self.axis.name].value}
    fn environment(&self,time:Fixed64,state:&[Fixed64])->Result<BTreeMap<String,Quantity>,ModelExecutionError>{
        if state.len()!=self.states.len(){return Err(time_error("상태 벡터 길이가 모형 소유자 목록과 다릅니다",self.request.span,self.context));}
        let mut env=self.base.clone();env.get_mut(&self.axis.name).expect("prepared axis").value=time;
        validate_domain(&env[&self.axis.name],self.axis,self.context)?;
        for (decl,value) in self.states.iter().zip(state){let slot=env.get_mut(&decl.name).expect("prepared state");slot.value=*value;validate_domain(slot,decl,self.context)?;}
        for (name,rule) in &self.algebraic{
            let q=expression(&rule.expression,&env,self.context)?;
            let decl=self.definition.declarations.iter().find(|d|d.name==*name).expect("prepared algebraic owner");
            if q.dimension!=decl.model_type.dimension.unwrap_or(UnitDim::NONE){return Err(arithmetic_failure("E_MODEL_UNIT_MISMATCH_CANDIDATE","대수 관계의 계산 단위가 선언과 다릅니다",rule.provenance,self.context));}
            validate_domain(&q,decl,self.context)?;
            env.insert((*name).into(),q);
        }Ok(env)
    }
    fn derivatives(&self,time:Fixed64,state:&[Fixed64])->Result<Vec<Fixed64>,ModelExecutionError>{
        self.context.work_cost.borrow_mut().charge_solver_evaluations(1).map_err(|e|work_cost_failure(e,self.request.span,self.context))?;
        let env=self.environment(time,state)?;
        self.states.iter().zip(&self.derivatives).map(|(decl,rule)|{
            let q=expression(&rule.expression,&env,self.context)?;
            let required=dimension(decl.model_type.dimension.unwrap_or(UnitDim::NONE),self.axis.model_type.dimension.unwrap(),true,rule.provenance,self.context)?;
            if q.dimension!=required{return Err(arithmetic_failure("E_MODEL_UNIT_MISMATCH_CANDIDATE","미분 결과의 단위가 상태/시간과 다릅니다",rule.provenance,self.context));}Ok(q.value)
        }).collect()
    }
    fn output_environment(&self,time:Fixed64,state:&[Fixed64])->Result<BTreeMap<String,ExactQuantity>,ModelExecutionError>{
        let env=self.environment(time,state)?;let mut out=BTreeMap::new();
        for decl in &self.definition.declarations{
            if let Some(q)=env.get(&decl.name){out.insert(decl.name.clone(),ExactQuantity{
                base_value:BigRational::new(BigInt::from(q.value.raw_i64()),BigInt::from(Fixed64::ONE_RAW)),dimension:q.dimension,numeric_kind:NumericKind::FixedQ32,
                authored_scale:unit_scale(decl.model_type.unit.as_deref()).map_err(|e|time_error(e,decl.identity.span,self.context))?,authored_unit:decl.model_type.unit.clone(),
            });}
        }Ok(out)
    }
    pub(super) fn run_typed_rk4(&self,step:Fixed64,count:u64,evaluation_ceiling:u64,observation_steps:&[u64])->Result<TimeRun,ModelExecutionError>{
        if observation_steps.windows(2).any(|w|w[0]>=w[1]) || observation_steps.iter().any(|i|*i>count){return Err(time_error("관찰 계산 단계는 중복 없이 증가하며 실제 계산 범위 안이어야 합니다",self.request.span,self.context));}
        let mut raw_samples=Vec::new();let mut next=0;
        let result=solver_rk4::integrate(solver_rk4::Request{initial_time:self.initial_time(),initial_state:&self.initial,step,step_count:count,evaluation_ceiling},
            |t,y|self.derivatives(t,y),|i,t,y|{if observation_steps.get(next)==Some(&i){raw_samples.push((i,t,y.to_vec()));next+=1;}})
            .map_err(|error|match error{
                solver_rk4::Failure::Callback{step,stage,mut cause}=>{cause.cause=Some(json!({"solver_step":step,"solver_stage":format!("{stage:?}"),"inner":cause.cause}));cause},
                solver_rk4::Failure::Arithmetic{fault,..}=>checked(Err(fault),self.request.span,self.context).unwrap_err(),
                solver_rk4::Failure::EvaluationCeiling{required,provided}=>time_error(format!("solver.evaluation_ceiling_exhausted: 필요{required}, 공급{provided}"),self.request.span,self.context),
                _=>time_error("Solver의 상태 길이·간격·단계 입력을 확인하세요",self.request.span,self.context),
            })?;
        let terminal=self.output_environment(result.time,&result.state)?;
        let samples=raw_samples.into_iter().map(|(step,time,state)|Ok(TimeSample{step,time,environment:self.output_environment(time,&state)?})).collect::<Result<Vec<_>,ModelExecutionError>>()?;
        Ok(TimeRun{outcome:result,samples,terminal,definition_identity:self.definition.identity.clone(),request_identity:self.request.identity.clone(),source_hash:self.program.source_hash.clone(),axis_owner:self.axis.identity.stable_id.clone(),state_owners:self.states.iter().map(|d|d.identity.stable_id.clone()).collect()})
    }
}
#[cfg(test)]mod tests{
 use super::*;
 const SOURCE:&str=include_str!("../../tests/fixtures/model_time_r1/용수철_시간_선언.또니");
 fn context()->ReceiptContext{ReceiptContext{source_path:"용수철_시간_선언.또니".into(),..Default::default()}}
 #[test]fn actual_parsed_spring_drives_typed_rk4_with_owned_units_samples_and_counts(){
  let program=ddonirang_lang::parse_model_program_with_source_file(SOURCE,"용수철_시간_선언.또니").unwrap();let c=context();let plan=TimeModel::prepare(&program,&program.ir.requests[0],&c).unwrap();
  let observations=(0..=8).map(|i|i*8).collect::<Vec<_>>();let run=plan.run_typed_rk4(Fixed64::parse_decimal("0.015625").unwrap(),64,256,&observations).unwrap();
  assert_eq!(run.outcome.time,Fixed64::ONE);assert_eq!(run.samples.len(),9);assert_eq!(run.samples[1].time,Fixed64::parse_decimal("0.125").unwrap());assert_eq!(run.samples[1].step,8);
  assert!((run.terminal["위치"].base_value.to_f64().unwrap()-2f64.cos()).abs()<0.000001);assert!((run.terminal["속도"].base_value.to_f64().unwrap()+2.0*2f64.sin()).abs()<0.000001);
  assert_eq!(run.samples[0].environment["힘"].base_value,BigRational::from_integer((-4).into()));
  assert_eq!(run.definition_identity,program.ir.definitions[0].identity);assert_eq!(run.request_identity,program.ir.requests[0].identity);assert_eq!(run.source_hash,program.source_hash);assert_eq!(run.state_owners.len(),2);assert_eq!(run.axis_owner,program.ir.definitions[0].declarations[0].identity.stable_id);
  assert_eq!(c.work_cost.borrow().receipt().counters.solver_evaluations,256);
 }
 #[test]fn one_changed_mass_changes_motion_not_the_model_ownership(){
  let source=SOURCE.replace("질량: 1@kg","질량: 4@kg");assert_ne!(source,SOURCE);let program=ddonirang_lang::parse_model_program(&source).unwrap();let c=context();let plan=TimeModel::prepare(&program,&program.ir.requests[0],&c).unwrap();let run=plan.run_typed_rk4(Fixed64::parse_decimal("0.015625").unwrap(),64,256,&[0,64]).unwrap();
  assert!((run.terminal["위치"].base_value.to_f64().unwrap()-1f64.cos()).abs()<0.000001);assert!((run.terminal["속도"].base_value.to_f64().unwrap()+1f64.sin()).abs()<0.000001);assert_eq!(run.samples[0].environment["가속도"].base_value,BigRational::from_integer((-1).into()));
 }
 #[test]fn budget_and_wrong_sample_selection_fail_before_solver_callbacks(){
  let program=ddonirang_lang::parse_model_program(SOURCE).unwrap();let c=context();let plan=TimeModel::prepare(&program,&program.ir.requests[0],&c).unwrap();
  assert!(plan.run_typed_rk4(Fixed64::ONE,2,7,&[0,2]).is_err());assert_eq!(c.work_cost.borrow().receipt().counters.solver_evaluations,0);assert!(plan.run_typed_rk4(Fixed64::ONE,2,8,&[1,0]).is_err());assert_eq!(c.work_cost.borrow().receipt().counters.solver_evaluations,0);
 }
 #[test]fn missing_initial_and_wrong_units_are_not_guessed(){
  let missing=SOURCE.replace("처음::고정=1@m, ","");let p=ddonirang_lang::parse_model_program(&missing).unwrap();let c=context();assert!(TimeModel::prepare(&p,&p.ir.requests[0],&c).is_err());
  let wrong=SOURCE.replace("위치' =:= 속도","위치' =:= 시간");let p=ddonirang_lang::parse_model_program(&wrong).unwrap();assert!(TimeModel::prepare(&p,&p.ir.requests[0],&c).is_err());
 }
 #[test]fn solver_admission_rounds_once_and_preserves_conversion_failure(){
  let c=context();let span=ModelSpan{start:0,end:1};assert_eq!(admit(BigRational::new(1.into(),3.into()),span,&c).unwrap().raw_i64(),1431655765);
  assert_eq!(admit(BigRational::new(1.into(),BigInt::from(1u64<<33)),span,&c).unwrap().raw_i64(),0);
  assert_eq!(admit(BigRational::new((-3).into(),BigInt::from(1u64<<33)),span,&c).unwrap().raw_i64(),-2);
  assert_eq!(admit(BigRational::from_integer(BigInt::from(i64::MAX)),span,&c).unwrap_err().candidate_code,"E_MODEL_SOLVER_CONVERSION_UNREPRESENTABLE");
 }
}

pub(super) const BACKEND_ID:&str=ddonirang_lang::model::FIXED_STEP_RK4_MODEL_BACKEND_ID;
pub(super) const NUMERIC_POLICY:&str="DDN-SOLVER-FIXED64-V1@1";
pub(super) const ACCEPTED_CLASS:&str="pure_deterministic_first_order_ode_with_acyclic_algebraic_relations";
pub(super) fn backend_descriptor_json()->JsonValue{
    let mut descriptor=json!({"schema":MODEL_BACKEND_DESCRIPTOR_SCHEMA,"backend_id":BACKEND_ID,"version":1,
        "algorithm_identity":"stdlib.solver.ddn-solver-fixed64-v1.rk4-fixed-step@1",
        "numeric_profile_identity":"DDN-SOLVER-FIXED64-V1","numeric_profile_version":1,
        "arithmetic_policy_id":NUMERIC_POLICY,"accepted_model_class":ACCEPTED_CLASS,
        "capabilities":["first_order_derivative","acyclic_algebraic_relation","fixed_step_rk4","checked_fixed64_ties_to_even","explicit_initial_conditions","explicit_output","verified_identical_input_replay"],
        "automatic_method_selection":false,"silent_fallback":false,"world_continuation":false});
    descriptor["descriptor_hash"]=json!(hash_json(&descriptor));descriptor
}
struct TimeGrid {step:Fixed64,count:u64,stride:u64,observations:Vec<u64>}
fn time_grid(plan:&TimeModel<'_>,recipe:&SolveRecipe)->Result<TimeGrid,ModelExecutionError>{
    let c=plan.context;
    if recipe.fields.len()!=3 || recipe.fields.iter().any(|f|!matches!(f.name.as_str(),"방식"|"간격"|"내줌간격")){
        return Err(time_error("#고정RK4 방법에는 방식·간격·내줌간격 세 필드가 필요합니다",recipe.span,c));
    }
    let field=|name:&str|recipe.fields.iter().find(|f|f.name==name).ok_or_else(||time_error(format!("방법의 {name} 필드가 없습니다"),recipe.span,c));
    let step_expr=&field("간격")?.value;let obs_expr=&field("내줌간격")?.value;
    let end_expr=plan.request.end.as_ref().ok_or_else(||time_error("시간 풀기에는 끝= 시각이 필요합니다",plan.request.span,c))?;
    let constants=BTreeMap::new();let step=expression(step_expr,&constants,c)?;let interval=expression(obs_expr,&constants,c)?;let end=expression(end_expr,&constants,c)?;
    if [step.dimension,interval.dimension,end.dimension].iter().any(|d|*d!=UnitDim::TIME){return Err(arithmetic_failure("E_MODEL_UNIT_MISMATCH_CANDIDATE","계산·관찰 간격과 끝은 시간 단위여야 합니다",recipe.span,c));}
    let h=step.value.raw_i64();let observed=interval.value.raw_i64();let delta=(end.value.raw_i64() as i128)-(plan.initial_time().raw_i64() as i128);
    if h<=0 || observed<=0 || delta<0{return Err(time_error("계산·관찰 간격은 양수이며 끝은 시작보다 이르지 않아야 합니다",recipe.span,c));}
    for (mismatch,field,expr,actual) in [(delta%(h as i128)!=0,"끝",end_expr,end.value.raw_i64()),(observed%h!=0,"내줌간격",obs_expr,observed)]{
        if mismatch{let mut error=failure("E_MODEL_TIME_GRID_MISMATCH","model_time_grid","끝·관찰 시각이 실제 Fixed64 계산 격자에 없습니다. 끝을 줄이거나 보간하지 않았습니다",expr.span,c);
            error.offending_field=Some(field.into());error.expected=Some("Fixed64 계산 격자 위 시각·간격".into());error.actual=Some(actual.to_string());return Err(error);}
    }
    let count=u64::try_from(delta/(h as i128)).map_err(|_|time_error("계산 단계 수가 범위를 벗어났습니다",recipe.span,c))?;let stride=(observed/h) as u64;
    let expected=(count/stride).checked_add(1).and_then(|n|n.checked_add(u64::from(count%stride!=0))).ok_or_else(||time_error("표본 개수가 범위를 벗어났습니다",recipe.span,c))?;
    let column_count=plan.program.ir.views.iter().map(|v|v.fields.len() as u64).sum::<u64>();
    let predicted=expected.checked_mul(column_count).ok_or_else(||time_error("표본 크기가 범위를 벗어났습니다",recipe.span,c))?;
    if predicted>c.work_cost.borrow().receipt().limits.public_observation_samples{return Err(time_error("요청한 표본이 기존 표준 관찰 한도를 넘습니다",recipe.span,c));}
    let mut observations=Vec::new();observations.try_reserve(usize::try_from(expected).map_err(|_|time_error("표본 개수 범위를 벗어났습니다",recipe.span,c))?).map_err(|_|time_error("표본 영역을 확보할 수 없습니다",recipe.span,c))?;
    let mut index=0;loop{observations.push(index);if count-index<stride{break;}index+=stride;}
    if observations.last()!=Some(&count){observations.push(count);}
    Ok(TimeGrid{step:step.value,count,stride,observations})
}
pub(super) fn execute(program:&ModelProgram,request:&RunRequest,recipe:&SolveRecipe,mut c:ReceiptContext)->Result<JsonValue,ModelExecutionError>{
    c.backend=ModelBackendKind::FixedRk4;
    c.recipe_identity=Some(recipe.identity.clone());c.recipe_hash=Some(recipe.source_hash.clone());c.public_solve_method=Some("#고정RK4".into());
    let plan=TimeModel::prepare(program,request,&c)?;
    let definition:ModelDefinition=(*plan.definition).clone();
    let grid=time_grid(&plan,recipe)?;
    let evaluations=grid.count.checked_mul(4).ok_or_else(||time_error("Solver 평가 한도가 범위를 벗어났습니다",recipe.span,&c))?;
    // The model caller supplies the exact finite workload it requested; this
    // does not enforce or reinterpret the measurement-only slice counter.
    let run=plan.run_typed_rk4(grid.step,grid.count,evaluations,&grid.observations)?;
    let result_pack=build_result_pack(&definition,request,&run.terminal,&c)?;
    let mut samples=Vec::new();let mut observations=Vec::new();
    for sample in &run.samples{
        let pack=build_result_pack(&definition,request,&sample.environment,&c)?;
        let raw_fields=pack["ordered_fields"].as_array().expect("owned output fields").iter().map(|field|{
            let name=field["name"].as_str().expect("owned field name");
            let decl=definition.declarations.iter().find(|d|d.name==name).expect("selected output owner");
            let quantity=sample.environment.get(name).expect("selected sample output");
            let scaled=(&quantity.base_value/&quantity.authored_scale)*BigInt::from(Fixed64::ONE_RAW);
            if !scaled.denom().is_one(){return Err(arithmetic_failure("E_MODEL_TIME_OUTPUT_RAW_UNREPRESENTABLE","이 표시 단위의 출력 원자료를 정확한 Fixed64로 운반할 수 없습니다",decl.identity.span,&c));}
            let raw=scaled.numer().to_i64().ok_or_else(||arithmetic_failure("E_MODEL_TIME_OUTPUT_RAW_UNREPRESENTABLE","출력 원자료 범위를 벗어났습니다",decl.identity.span,&c))?;
            Ok(json!({"declaration_owner":decl.identity.stable_id,"name":name,"unit":field["unit"],"type":field["type"],"q32_raw_decimal":raw.to_string(),"value_text":field["value_text"]}))
        }).collect::<Result<Vec<_>,ModelExecutionError>>()?;
        let body=json!({"step":sample.step.to_string(),"time_raw":sample.time.raw_i64().to_string(),"time_text":format!("{}@s",sample.time.canonical_decimal_text()),"canonical_time_unit":"s","raw_fields":raw_fields,
            "axis_owner":run.axis_owner,"state_owners":run.state_owners,"definition_identity":run.definition_identity,"request_identity":run.request_identity,"source_hash":run.source_hash,"result_pack":pack});
        let model_state_hash=hash_json(&body);let mut emitted=build_public_observations(program,request,&pack,&c)?;
        for cell in &mut emitted{cell["model_sample"]=json!({"step":sample.step,"time_raw":sample.time.raw_i64().to_string(),"canonical_time_unit":"s","model_state_hash":model_state_hash,"axis_owner":run.axis_owner,"source_hash":run.source_hash});}
        observations.extend(emitted);let mut item=body;item["model_state_hash"]=json!(model_state_hash);samples.push(item);
    }
    let time=json!({"algorithm_identity":"stdlib.solver.ddn-solver-fixed64-v1.rk4-fixed-step@1","numeric_profile_identity":"DDN-SOLVER-FIXED64-V1","numeric_profile_version":1,
        "rounding_identity":"DDN-SOLVER-FIXED64-V1/ROUND-TIES-TO-EVEN-V1","initial_time_raw":plan.initial_time().raw_i64().to_string(),"end_time_raw":run.outcome.time.raw_i64().to_string(),
        "calculation_step_raw":grid.step.raw_i64().to_string(),"observation_stride_steps":grid.stride,"steps":run.outcome.steps.to_string(),"evaluations":run.outcome.evaluations.to_string(),"evaluation_ceiling":evaluations.to_string(),
        "sample_count":samples.len(),"time_unit":"s","samples_hash":hash_json(&json!(samples)),"endpoint_shortened":false,"interpolation":false,"terminal_fields_are_scalars":true,
        "definition_identity":run.definition_identity,"request_identity":run.request_identity,"source_hash":run.source_hash,"axis_owner":run.axis_owner,"state_owners":run.state_owners});
    let committed=json!({"schema":"ddn.model_committed_state.candidate.v1","bindings":[{"name":request.binding_name,"value":result_pack}],"temporal_samples":samples});
    let post=hash_json(&committed);
    let dependency_order=plan.algebraic.iter().map(|(n,_)|(*n).to_owned()).collect::<Vec<_>>();
    drop(plan);
    // Preparation borrows the context; all typed owner values are captured
    // before final receipt metadata is updated.
    let initial_map=request.initial_overrides.iter().map(|f|(f.name.clone(),f.value.clone())).collect();
    let instance=ModelInstance::candidate(definition.identity.clone(),format!("{}#{}",definition.name,request.binding_name),BTreeMap::new(),initial_map,request.span);
    c.instance_identity=Some(instance.identity);c.instance_binding_hash=Some(instance.source_hash);c.input_snapshot_hash=Some(input_snapshot_hash(request));c.unit_registry_hash=Some(unit_registry_hash(&definition)?);
    c.dependency_order_hash=Some(hash_json(&json!(dependency_order)));
    c.work_cost.borrow_mut().note_created_identity().map_err(|e|work_cost_failure(e,request.span,&c))?;
    c.work_cost.borrow_mut().note_public_observation_samples(observations.len()).map_err(|e|work_cost_failure(e,request.span,&c))?;
    let mut receipt=success_receipt(&c,&empty_state_hash(),&post,&hash_json(&result_pack));
    receipt.as_object_mut().expect("owned receipt").remove("receipt_hash");receipt["time_execution"]=time.clone();receipt=seal_receipt(receipt);
    let ir=ddonirang_lang::model_ir_candidate_json(program);
    let find=|key:&str,name:&str,value:&str|ir[key].as_array().and_then(|v|v.iter().find(|v|v[name].as_str()==Some(value))).cloned().unwrap_or(JsonValue::Null);
    Ok(json!({"schema":MODEL_EXECUTION_SCHEMA,"wire_status":"candidate_non_public","status":"completed","backend_descriptor":backend_descriptor_json(),"canonical_ddn":program.canonical_ddn,
        "model_ir_semantic_hash":program.ir.semantic_hash,"work_cost_receipt":c.work_cost.borrow().receipt(),"pre_state_hash":empty_state_hash(),"post_state_hash":post,"atomic_commit":"committed",
        "dependency_order":[],"result_binding":request.binding_name,"result_pack":result_pack,"public_observations":observations,"committed_state":committed,"time_execution":time,
        "execution_bindings":{"schema":"ddn.model_execution_bindings.candidate.v1","wire_status":"candidate_non_public","model_ir_schema":ir["schema"],"program_source_hash":program.source_hash,
            "canonical_program_hash":hash_bytes(program.canonical_ddn.as_bytes()),"model_ir_semantic_hash":ir["semantic_hash"],"model_definition_artifact":find("definitions","identity",&definition.identity),
            "solve_recipe":find("recipes","identity",&recipe.identity),"run_request":find("requests","identity",&request.identity)},"receipt":receipt}))
}

pub(super) fn capability_matrix_json(contract:&ExecutionContractProfile)->JsonValue{
    let mut value=super::execution_contract_capability_matrix_json(contract);
    if let Some(rows)=value["rows"].as_array_mut(){for row in rows{
        if row["field"]==EXECUTION_CONTRACT_FIELD_RESULT{row["evidence"]=json!("checked Fixed64 RK4 owner profile, terminal result and actual sample hashes");}
        if row["field"]==EXECUTION_CONTRACT_FIELD_PERFORMANCE{row["evidence"]=json!("fixed grid and explicit selected method; adaptive steps, hidden endpoint adjustment and fallback absent");}
    }}value
}
