//! Checked classical RK4, bound to DDN-SOLVER-FIXED64-V1@1.
//! Product-owned internal Rust carrier, not a source/API/registry identity issuer.
//! Callers stage observations and publish only after the whole solve succeeds.
use crate::{Fixed64, signals::ArithmeticFaultKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage { K1, K2, K3, K4, Update }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure<E> {
    InvalidArgument,
    EvaluationCeiling { required:u64, provided:u64 },
    Arithmetic { step:u64, stage:Stage, fault:ArithmeticFaultKind },
    Callback { step:u64, stage:Stage, cause:E },
    Shape { step:u64, stage:Stage, expected:usize, actual:usize },
}
pub struct Request<'a> {
    pub initial_time:Fixed64,
    pub initial_state:&'a [Fixed64],
    pub step:Fixed64,
    pub step_count:u64,
    pub evaluation_ceiling:u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {pub time:Fixed64,pub state:Vec<Fixed64>,pub steps:u64,pub evaluations:u64}

pub fn integrate<E>(request:Request<'_>,
    mut derivative:impl FnMut(Fixed64,&[Fixed64])->Result<Vec<Fixed64>,E>,
    mut observe:impl FnMut(u64,Fixed64,&[Fixed64]),
)->Result<Outcome,Failure<E>> {
    let Request{initial_time,initial_state,step:h,step_count,evaluation_ceiling}=request;
    if h==Fixed64::ZERO || initial_state.is_empty() {return Err(Failure::InvalidArgument);}
    let required=step_count.checked_mul(4).ok_or(Failure::InvalidArgument)?;
    if evaluation_ceiling<required {return Err(Failure::EvaluationCeiling{required,provided:evaluation_ceiling});}
    let mut time=initial_time;let mut state=initial_state.to_vec();
    let two=Fixed64::from_raw_i64(2*Fixed64::ONE_RAW);
    let six=Fixed64::from_raw_i64(6*Fixed64::ONE_RAW);
    observe(0,time,&state);
    for index in 0..step_count {
        let check=|value:Result<Fixed64,ArithmeticFaultKind>,stage|value.map_err(|fault|Failure::Arithmetic{step:index,stage,fault});
        let mut call=|t,y:&[Fixed64],stage|{
            let result=derivative(t,y).map_err(|cause|Failure::Callback{step:index,stage,cause})?;
            if result.len()!=state.len(){return Err(Failure::Shape{step:index,stage,expected:state.len(),actual:result.len()});}
            Ok(result)
        };
        let k1=call(time,&state,Stage::K1)?;
        let half=check(h.try_div(two),Stage::K2)?;
        let midpoint=check(time.checked_add(half),Stage::K2)?;
        let shifted=|k:&[Fixed64],divide_by_two:bool,stage|->Result<Vec<Fixed64>,Failure<E>>{
            state.iter().zip(k).map(|(y,k)|{
                let product=check(h.checked_mul(*k),stage)?;
                let delta=if divide_by_two{check(product.try_div(two),stage)?}else{product};
                check(y.checked_add(delta),stage)
            }).collect()
        };
        let y2=shifted(&k1,true,Stage::K2)?;
        let k2=call(midpoint,&y2,Stage::K2)?;
        let y3=shifted(&k2,true,Stage::K3)?;
        let k3=call(midpoint,&y3,Stage::K3)?;
        let endpoint=check(time.checked_add(h),Stage::K4)?;
        let y4=shifted(&k3,false,Stage::K4)?;
        let k4=call(endpoint,&y4,Stage::K4)?;
        // Explicit source-owner left-to-right accumulation. Never fuse/reassociate.
        let mut next=Vec::with_capacity(state.len());
        for component in 0..state.len(){
            let twice2=check(two.checked_mul(k2[component]),Stage::Update)?;
            let twice3=check(two.checked_mul(k3[component]),Stage::Update)?;
            let sum=check(k1[component].checked_add(twice2),Stage::Update)?;
            let sum=check(sum.checked_add(twice3),Stage::Update)?;
            let sum=check(sum.checked_add(k4[component]),Stage::Update)?;
            let scaled=check(h.checked_mul(sum),Stage::Update)?;
            let delta=check(scaled.try_div(six),Stage::Update)?;
            next.push(check(state[component].checked_add(delta),Stage::Update)?);
        }
        time=check(time.checked_add(h),Stage::Update)?;
        state=next;
        observe(index+1,time,&state);
    }
    Ok(Outcome{time,state,steps:step_count,evaluations:required})
}

#[cfg(test)]mod tests{
 use super::*;
 const MATERIAL:&str="STDLIB_SOLVER_NUMERIC_PROFILE_EXACT_MATERIAL_R1_1_20260908 / sha256:5084ea9452cb14d41340d820dd617e2e64c5a4b63c186377494f14385eba3a7c";
 fn request(y:&[Fixed64],h:Fixed64,n:u64)->Request<'_>{Request{initial_time:Fixed64::ZERO,initial_state:y,step:h,step_count:n,evaluation_ceiling:n*4}}
 #[test]fn rk_01_matches_recovered_numeric_owner_raw_and_stages(){
  let mut stages=Vec::new();let mut samples=Vec::new();let initial=[Fixed64::ONE];
  let result=integrate(request(&initial,Fixed64::from_raw_i64(429496730),1),|_,y|{stages.push(y[0].raw_i64());Ok::<_,()>(y.to_vec())},|i,t,y|samples.push((i,t.raw_i64(),y[0].raw_i64()))).unwrap();
  assert_eq!(stages,[4294967296,4509715661,4520453080,4747012604],"{MATERIAL}");
  assert_eq!((result.time.raw_i64(),result.state[0].raw_i64(),result.evaluations),(429496730,4746672586,4));assert_eq!(samples.len(),2);
 }
 #[test]fn rk_02_matches_recovered_numeric_owner_raw_and_callback_order(){
  let initial=[Fixed64::ZERO];let mut stages=Vec::new();
  let result=integrate(request(&initial,Fixed64::from_raw_i64(2147483648),2),|t,_|{stages.push(t.raw_i64());Ok::<_,()>(vec![t])},|_,_,_|{}).unwrap();
  assert_eq!(stages,[0,1073741824,1073741824,2147483648,2147483648,3221225472,3221225472,4294967296]);
  assert_eq!((result.time.raw_i64(),result.state[0].raw_i64(),result.evaluations),(4294967296,2147483648,8),"{MATERIAL}");
 }
 #[test]fn coupled_vector_uses_one_complete_state_per_stage(){
  let initial=[Fixed64::ONE,Fixed64::ZERO];let mut calls=Vec::new();
  let result=integrate(request(&initial,Fixed64::parse_decimal("0.5").unwrap(),1),|t,y|{calls.push((t.raw_i64(),y.iter().map(|v|v.raw_i64()).collect::<Vec<_>>()));Ok::<_,()>(vec![y[1],Fixed64::ZERO.checked_sub(y[0]).unwrap()])},|_,_,_|{}).unwrap();
  assert_eq!(calls.len(),4);assert_eq!(calls[1].1,[4294967296,-1073741824]);assert_eq!(calls[2].1,[4026531840,-1073741824]);assert_eq!(calls[3].1,[3758096384,-2013265920]);assert_eq!(result.evaluations,4);
 }
 #[test]fn budget_rejection_precedes_callbacks_and_zero_steps_do_not_evaluate(){
  let initial=[Fixed64::ONE];let mut r=request(&initial,Fixed64::ONE,2);r.evaluation_ceiling=7;
  let mut calls=0;let mut observations=0;let failed=integrate(r,|_,y|{calls+=1;Ok::<_,()>(y.to_vec())},|_,_,_|observations+=1);
  assert!(matches!(failed,Err(Failure::EvaluationCeiling{required:8,provided:7})));assert_eq!((calls,observations),(0,0));
  let result=integrate(request(&initial,Fixed64::ONE,0),|_,_|->Result<Vec<Fixed64>,()>{panic!("zero-step callback")},|_,_,_|{}).unwrap();assert_eq!(result.evaluations,0);assert_eq!(result.state,initial);
 }
 #[test]fn shape_callback_and_overflow_keep_failures_distinct(){
  let initial=[Fixed64::ONE];let wrong=integrate(request(&initial,Fixed64::ONE,1),|_,_|Ok::<_,()>(vec![]),|_,_,_|{});assert!(matches!(wrong,Err(Failure::Shape{stage:Stage::K1,..})));
  let failed=integrate(request(&initial,Fixed64::ONE,1),|_,_|Err::<Vec<Fixed64>,_>("domain"),|_,_,_|{});assert!(matches!(failed,Err(Failure::Callback{stage:Stage::K1,cause:"domain",..})));
  let high=[Fixed64::MAX];let overflow=integrate(request(&high,Fixed64::ONE,1),|_,y|Ok::<_,()>(y.to_vec()),|_,_,_|{});assert!(matches!(overflow,Err(Failure::Arithmetic{fault:ArithmeticFaultKind::Overflow,..})));
 }
}
