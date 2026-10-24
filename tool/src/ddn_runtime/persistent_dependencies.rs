//! WO22 B2-09..12 provisional product graph. Source owns the relation;
//! resource values never manufacture an edge or an execution capability.
use super::*;

#[derive(Clone,Debug)]
pub(crate) struct DerivedBinding {
    pub id:NodeId,
    pub name:String,
    pub value:Expr,
    pub dependencies:BTreeSet<String>,
}
#[derive(Clone,Debug,Default)]
pub(crate) struct DerivedPlan {pub bindings:Vec<DerivedBinding>,pub declaration_ids:BTreeSet<NodeId>}

fn failure(code:&str,text:impl std::fmt::Display)->String {format!("{code}: {text}")}
fn physical_name(name:&str)->bool {name=="입력키"||name.starts_with("샘.키보드.")}

impl DerivedPlan {
    pub fn compile(program:&CanonProgram)->Result<Self,String> {
        let mut bindings=BTreeMap::new();
        for stmt in &program.root_stmts {
            if let Stmt::DerivedBinding{id,name,value,..}=stmt {
                if bindings.contains_key(name) {return Err(failure("E_DERIVED_DUPLICATE",format!("{name}에 파생 생산자가 둘입니다")));}
                if physical_name(name) {return Err(failure("E_DERIVED_TARGET","입력 자원은 파생 대상이 아닙니다"));}
                let mut dependencies=BTreeSet::new();
                pure_expr(program,value,&BTreeSet::new(),&mut dependencies,&mut BTreeSet::new())?;
                bindings.insert(name.clone(),DerivedBinding{id:*id,name:name.clone(),value:value.clone(),dependencies});
            }
        }
        if bindings.is_empty(){return Ok(Self::default());}
        let targets=bindings.keys().cloned().collect::<BTreeSet<_>>();
        let mut declaration_ids=BTreeSet::new();
        let mut setup=program.root_stmts.iter().collect::<Vec<_>>();
        for TopLevelItem::SeedDef(seed) in &program.items {
            if let Some(body)=&seed.body {setup.extend(body.stmts.iter().filter(|stmt|seed.is_relocated_top_level_setup(stmt)));}
        }
        let mut world_names=targets.clone();
        for stmt in &setup {
            match stmt {
                Stmt::Definition{name,..}=>{world_names.insert(name.clone());},
                Stmt::DeclBlock{items,..}=>world_names.extend(items.iter().map(|item|item.name.clone())),
                _=>{},
            }
        }
        crate::source_analysis::collect_world_bindings(&program.root_stmts,&BTreeSet::new(),&mut world_names);
        for TopLevelItem::SeedDef(seed) in &program.items {
            if seed.seed_kind==SeedKind::Umjikssi {if let Some(body)=&seed.body {
                let locals=seed.params.iter().map(|pin|pin.pin_name.clone()).collect();
                crate::source_analysis::collect_world_bindings(&body.stmts,&locals,&mut world_names);
            }}
        }
        for binding in bindings.values(){for name in &binding.dependencies {
            if !world_names.contains(name){return Err(failure("E_DERIVED_DEPENDENCY_REQUIRED",format!("{name}: 파생식은 원문이 결속한 세계 값만 읽습니다. 호스트의 암묵 값·입력은 사용할 수 없습니다")));}
        }}
        for stmt in setup {if let Stmt::DeclBlock{items,..}=stmt {for item in items.iter().filter(|item|targets.contains(&item.name)) {
            if item.value.is_some(){return Err(failure("E_DERIVED_PRODUCER_CONFLICT",format!("{}: 파생식과 별도 초기값을 함께 생산할 수 없습니다. 값 없는 씨 선언을 사용하세요",item.name)));}
            declaration_ids.insert(item.id);
        }}}
        check_producers(&program.root_stmts,&targets,&BTreeSet::new(),true)?;
        for TopLevelItem::SeedDef(seed) in &program.items {
            if let Some(body)=&seed.body {
                let mut locals=seed.params.iter().map(|pin|pin.pin_name.clone()).collect::<BTreeSet<_>>();
                if matches!(seed.seed_kind,SeedKind::Semssi|SeedKind::ValueFunc){locals.insert(seed.canonical_name.clone());}
                check_producers(&body.stmts,&targets,&locals,false)?;
            }
        }
        // Kahn order uses existing UTF-8 name order only to stabilize independent
        // pure evaluations. It is not a new public identity ordering contract.
        let mut ordered=Vec::new();let mut done=BTreeSet::new();
        while !bindings.is_empty() {
            let ready=bindings.iter().find(|(_,binding)|binding.dependencies.iter().all(|name|!targets.contains(name)||done.contains(name)))
                .map(|(name,_)|name.clone());
            let Some(name)=ready else {return Err(failure("E_DERIVED_CYCLE",format!("파생 관계에 순환이 있습니다: {}",bindings.keys().cloned().collect::<Vec<_>>().join(", "))));};
            done.insert(name.clone());ordered.push(bindings.remove(&name).unwrap());
        }
        Ok(Self{bindings:ordered,declaration_ids})
    }
    pub fn contains(&self,name:&str)->bool {self.bindings.iter().any(|binding|binding.name==name)}
}

// Only provider-owned deterministic, value-only intrinsics are admitted here.
// Unknown calls are never guessed pure from their Korean spelling.
fn pure_intrinsic(name:&str)->bool {
    matches!(ddonirang_lang::stdlib::canonicalize_stdlib_alias(name),
        "abs"|"sqrt"|"powi"|"min"|"max"|"sin"|"cos"|"tan"|"asin"|"acos"|"atan"|"atan2"|
        "floor"|"ceil"|"round"|"바닥"|"천장"|"반올림"|"셈수"|"바른수"|"나눔수"|"길이"|"차림.값"|"차림.길이")
}
fn pure_expr(program:&CanonProgram,expr:&Expr,locals:&BTreeSet<String>,deps:&mut BTreeSet<String>,seen:&mut BTreeSet<NodeId>)->Result<(),String> {
    let mut children=Vec::new();
    match &expr.kind {
        ExprKind::Literal(_)=>{},
        ExprKind::Var(name)=>{if physical_name(name){return Err(failure("E_DERIVED_EFFECT","파생식에서 입력을 읽을 수 없습니다"));}if !locals.contains(name){deps.insert(name.clone());}},
        ExprKind::Call{func,args}=>{
            children.extend(args.iter().map(|arg|&arg.expr));
            let builtin=crate::work_cost::builtin_work_cost_rule(ddonirang_lang::stdlib::canonicalize_stdlib_alias(func))
                .map_err(|_|failure("E_DERIVED_EFFECT","파생식의 표준 공급자 소유권을 확인할 수 없습니다"))?.is_some();
            let candidates=program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed)
                .filter(|seed|seed.callable_surface_names().iter().any(|name|name==func)).collect::<Vec<_>>();
            if builtin || candidates.is_empty() {
                if !pure_intrinsic(func){return Err(failure("E_DERIVED_EFFECT",format!("{func}: 순수식 공급이 확인되지 않은 호출·입력·난수는 파생식에서 지원하지 않습니다")));}
            }else{
                let ids=program.origin.registered_call_candidates.get(&expr.id);
                let resolved=candidates.into_iter().filter(|seed|ids.is_none_or(|ids|ids.contains(&seed.id))).collect::<Vec<_>>();
                if resolved.len()!=1 {return Err(failure("E_DERIVED_CALLABLE_AMBIGUOUS",format!("{func}: 파생식의 호출 소유자를 하나로 확인할 수 없습니다")));}
                let seed=resolved[0];
                if !matches!(seed.seed_kind,SeedKind::Semssi|SeedKind::ValueFunc){return Err(failure("E_DERIVED_EFFECT",format!("{func}: 파생식에서는 셈씨만 부를 수 있습니다")));}
                if seen.insert(seed.id) {
                    let mut function_locals=seed.params.iter().map(|pin|pin.pin_name.clone()).collect::<BTreeSet<_>>();
                    function_locals.insert(seed.canonical_name.clone());
                    let body=seed.body.as_ref().ok_or_else(||failure("E_DERIVED_EFFECT","파생식 셈씨의 몸통이 없습니다"))?;
                    pure_body(program,&body.stmts,&mut function_locals,deps,seen)?;
                }
            }
        },
        ExprKind::Infix{left,right,..}=>{children.push(left.as_ref());children.push(right.as_ref());},
        ExprKind::FieldAccess{target,..}|ExprKind::Suffix{value:target,..}|ExprKind::Nuance{expr:target,..}=>children.push(target),
        ExprKind::CollectionConstructor{elements}|ExprKind::TupleConstructor{elements}|ExprKind::Pipe{stages:elements}=>children.extend(elements),
        ExprKind::SumBranchValue{payload,..}=>children.extend(payload.iter().map(|arg|&arg.value)),
        ExprKind::ProductConstruction{category,fields,..}=>{
            if *category==Some(ddonirang_lang::SchemaCategory::Subject){return Err(failure("E_DERIVED_EFFECT","파생식에서 대상을 생성할 수 없습니다"));}
            children.extend(fields.iter().map(|arg|&arg.expr));
        },
        ExprKind::Pack{fields}|ExprKind::TemplateRender{inject:fields,..}|ExprKind::FormulaEval{inject:fields,..}=>children.extend(fields.iter().map(|(_,value)|value)),
        ExprKind::Tensor(value)=>children.push(value.input()),
        ExprKind::SelectionPlace{target,index}=>{children.push(target);children.push(index);},
        ExprKind::MultiAxisSelection(selection)=>children.extend(selection.children()),
        ExprKind::Template(_)|ExprKind::Formula(_)=>{},
        // A thunk/lambda is not executed by declaring it. Do not infer a future
        // invocation's effect or depend on ambient evaluator/context injection.
        _=>return Err(failure("E_DERIVED_EFFECT","이 실행식·지연식은 파생 순수식으로 지원하지 않습니다")),
    }
    for child in children {pure_expr(program,child,locals,deps,seen)?;}
    Ok(())
}
fn pure_body(program:&CanonProgram,stmts:&[Stmt],locals:&mut BTreeSet<String>,deps:&mut BTreeSet<String>,seen:&mut BTreeSet<NodeId>)->Result<(),String> {
    for stmt in stmts {match stmt {
        Stmt::Definition{name,value,..}=>{pure_expr(program,value,locals,deps,seen)?;locals.insert(name.clone());},
        Stmt::Return{value,..}|Stmt::Expr{expr:value,..}=>pure_expr(program,value,locals,deps,seen)?,
        Stmt::DeclBlock{items,..}=>for item in items {if let Some(value)=&item.value{pure_expr(program,value,locals,deps,seen)?;}locals.insert(item.name.clone());},
        Stmt::Mutate{target,value,..}=>{
            if !matches!(&target.kind,ExprKind::Var(name) if locals.contains(name)){return Err(failure("E_DERIVED_EFFECT","파생식 셈씨는 세계를 다시 쓸 수 없습니다"));}
            pure_expr(program,value,locals,deps,seen)?;
        },
        Stmt::If{condition,then_body,else_body,..}=>{
            pure_expr(program,condition,locals,deps,seen)?;
            pure_body(program,&then_body.stmts,&mut locals.clone(),deps,seen)?;
            if let Some(body)=else_body{pure_body(program,&body.stmts,&mut locals.clone(),deps,seen)?;}
        },
        _=>return Err(failure("E_DERIVED_EFFECT","파생식 셈씨에 지원되지 않는 효과·제어 문장이 있습니다")),
    }}Ok(())
}
fn target_root(mut expr:&Expr)->Option<&str> {
    loop{match &expr.kind {
        ExprKind::Var(name)=>return Some(name),
        ExprKind::FieldAccess{target,..}|ExprKind::SelectionPlace{target,..}=>expr=target,
        ExprKind::MultiAxisSelection(selection)=>expr=&selection.target,
        _=>return None,
    }}
}
fn check_producers(stmts:&[Stmt],targets:&BTreeSet<String>,inherited:&BTreeSet<String>,root:bool)->Result<(),String> {
    let mut locals=inherited.clone();
    for stmt in stmts {
        match stmt {
            Stmt::DerivedBinding{..} if !root=>return Err(failure("E_DERIVED_SCOPE","파생 관계는 맨 바깥에만 선언합니다")),
            Stmt::Definition{name,..}=>{
                if root&&targets.contains(name){return Err(failure("E_DERIVED_PRODUCER_CONFLICT",format!("{name}: 고정 정의와 파생 생산자를 함께 둘 수 없습니다")));}
                if !root{locals.insert(name.clone());}
            },
            Stmt::DeclBlock{items,..} if root=>for item in items {
                if targets.contains(&item.name)&&item.value.is_some(){return Err(failure("E_DERIVED_PRODUCER_CONFLICT",format!("{}: 파생식과 별도 초기값을 함께 생산할 수 없습니다. 값 없는 씨 선언을 사용하세요",item.name)));}
            },
            Stmt::DeclBlock{items,..} if !root=>{locals.extend(items.iter().map(|item|item.name.clone()));},
            Stmt::Mutate{target,..}|Stmt::ResultFlow{destination:target,..}=>{
                if let Some(name)=target_root(target){if targets.contains(name)&&!locals.contains(name){return Err(failure("E_DERIVED_DIRECT_WRITE",format!("{name}: 파생 대상에 직접 값을 넣을 수 없습니다")));}}
            },
            _=>{},
        }
        for body in nested_bodies(stmt){check_producers(&body.stmts,targets,&locals,false)?;}
    }Ok(())
}
fn nested_bodies(stmt:&Stmt)->Vec<&Body> {
    match stmt {
        Stmt::SumMatch{arms,default_body,..}=>arms.iter().map(|arm|&arm.body).chain(default_body.iter()).collect(),
        Stmt::If{then_body,else_body,..}=>std::iter::once(then_body).chain(else_body.iter()).collect(),
        Stmt::Choose{branches,else_body,..}=>branches.iter().map(|branch|&branch.body).chain(std::iter::once(else_body)).collect(),
        Stmt::Receive{body,..}|Stmt::Try{body,..}|Stmt::Repeat{body,..}|Stmt::While{body,..}|Stmt::ForEach{body,..}|Stmt::Quantifier{body,..}|Stmt::Guard{body,..}|Stmt::BeatBlock{body,..}|Stmt::Hook{body,..}|Stmt::HookWhenBecomes{body,..}|Stmt::HookWhile{body,..}=>vec![body],
        Stmt::Contract{then_body,else_body,..}=>then_body.iter().chain(std::iter::once(else_body)).collect(),
        _=>Vec::new(),
    }
}

impl<'a> EvalContext<'a> {
    pub(super) fn settle_derived(&mut self,locals:&mut HashMap<String,Value>)->Result<(),EvalError> {
        if self.program.derived_plan.bindings.is_empty(){return Ok(());}
        let plan=Arc::clone(&self.program.derived_plan);
        let snapshot=self.capture_frame_snapshot(locals);
        let result=(|| {
            for binding in &plan.bindings {
                for name in &binding.dependencies {
                    let staged=self.patch_ops.iter().any(|op|match op {
                        PatchOp::SetResourceFixed64{tag,..}|PatchOp::SetResourceHandle{tag,..}
                            |PatchOp::SetResourceValue{tag,..}=>tag==name,
                        _=>false,
                    });
                    if staged {
                        if !self.resources.contains_key(name){return Err(failure("E_DERIVED_DEPENDENCY_REQUIRED","현재 마디의 파생 입력 값이 없습니다").into());}
                    }else{
                        if !self.committed_resource_exists(name){return Err(failure("E_DERIVED_DEPENDENCY_REQUIRED",format!("{name}: 파생 입력은 실제 세계에 먼저 결속되어야 합니다. 호스트 기본값으로 대신하지 않습니다")).into());}
                        // A cached host projection is not the source-owned
                        // world slot, even when it uses the same visible name.
                        self.resources.remove(name);
                    }
                    let value=self.get_resource(name).ok_or_else(||failure("E_DERIVED_DEPENDENCY_REQUIRED","파생 입력의 세계 값을 읽을 수 없습니다"))?;
                    locals.insert(name.clone(),value);
                }
                let expected=self.binding_type_scopes.iter().rev().find_map(|scope|scope.get(&binding.name).map(|(kind,_)|kind.clone()));
                let value=self.eval_expr_expected(locals,&binding.value,expected.as_ref().unwrap_or(&TypeRef::Infer))?;
                if let Some(kind)=expected.as_ref(){self.check_value_type(&value,kind)
                    .map_err(|detail|type_mismatch_error(&binding.name,&detail.expected,&detail.actual))?;}
                // Only this graph owner bypasses the direct-write guard. Each
                // intermediate stays in this private frame, never a world patch
                // returned to the caller until every relation has succeeded.
                let saved_kind=self.tuple_type_plan.for_write(binding.id,&binding.name).cloned();
                self.set_resource_with_type_owner(&binding.name,value.clone(),saved_kind.as_ref())?;
                locals.insert(binding.name.clone(),value);
            }
            self.derived_initialized=true;Ok(())
        })();
        if result.is_err(){self.restore_frame_snapshot(locals,&snapshot);}
        result
    }
}

impl DdnRunner {
    pub(super) fn validate_derived_world(&self,world:&NuriWorld,required:bool)->Result<(),String> {
        let plan=&self.program.derived_plan;
        if plan.bindings.is_empty(){return Ok(());}
        let exists=|name:&str|world.get_resource_fixed64(name).is_some()||world.get_resource_value(name).is_some()||world.get_resource_handle(name).is_some();
        let count=plan.bindings.iter().filter(|binding|exists(&binding.name)).count();
        if count==0&&!required{return Ok(());}
        if count!=plan.bindings.len(){return Err(failure("E_DERIVED_RECOVERY_REQUIRED","복구 필요: 원문이 정한 파생값 일부 또는 전부가 저장본에 없습니다"));}
        let defaults=HashMap::new();
        let mut ctx=EvalContext::new(&self.program,world,&defaults,None,InputState::new(0,0),0,0,self.project_asset_registry.as_deref());
        ctx.tuple_type_plan=Rc::clone(&self.tuple_type_plan);
        let mut locals=HashMap::new();
        for binding in &plan.bindings {
            let previous=ctx.get_resource(&binding.name).ok_or_else(||failure("E_DERIVED_RECOVERY_REQUIRED","복구 필요: 파생 저장값을 읽을 수 없습니다"))?;
            let calculated=ctx.eval_expr(&mut locals,&binding.value).map_err(|error|failure("E_DERIVED_RECOVERY_REQUIRED",format!("복구 필요: 파생식을 다시 계산할 수 없습니다: {error}")))?;
            if previous!=calculated{return Err(failure("E_DERIVED_RECOVERY_REQUIRED",format!("복구 필요: {}의 저장값과 원문 파생식의 값이 다릅니다",binding.name)));}
            ctx.resources.insert(binding.name.clone(),calculated.clone());locals.insert(binding.name.clone(),calculated);
        }
        Ok(())
    }
    pub(crate) fn validate_restored_derived_world(&self,world:&NuriWorld)->Result<(),String> {
        self.validate_derived_world(world,true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::platform::{DetNuri,Nuri};
    fn input(tick:u64)->InputSnapshot {InputSnapshot {tick_id:tick,dt:Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),
        pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),frame_source:ddonirang_core::InputSource::Person,rng_seed:0}}
    fn runner(source:&str)->DdnRunner {DdnRunner::new(DdnProgram::from_v25_source(source,"derived.ddn").unwrap(),"매마디")}
    #[test]
    fn graph_rejects_duplicate_cycle_direct_producer_and_effects_before_execution() {
        for (source,code) in [
            ("앞값 <<- 1.0. 앞값 <<- 2.0.","E_DERIVED_DUPLICATE"),
            ("앞값 <<- 뒷값 + 1.0. 뒷값 <<- 앞값 + 1.0.","E_DERIVED_CYCLE"),
            ("원천 <- 1.0. 결과 <<- 원천 + 1.0. 결과 <- 9.0.","E_DERIVED_DIRECT_WRITE"),
            ("결과 := 1.0. 결과 <<- 2.0.","E_DERIVED_PRODUCER_CONFLICT"),
            ("결과 <<- 입력키.","E_DERIVED_EFFECT"),
            ("결과 <<- 프레임수.","E_DERIVED_DEPENDENCY_REQUIRED"),
            ("결과 <<- () 입력사상.사건들.","E_DERIVED_EFFECT"),
            ("숨김:글:셈씨 := { 숨김 <- 입력키. }. 결과 <<- () 숨김.","E_DERIVED_EFFECT"),
            ("도움:셈수:셈씨 := { 도움 <- 앞값. }. 앞값 <<- () 도움.","E_DERIVED_CYCLE"),
            ("채비 { 결과:셈수 := 0.0. }. 결과 <<- 2.0.","E_DERIVED_PRODUCER_CONFLICT"),
        ] {
            let error=DdnProgram::from_v25_source(source,"invalid-derived.ddn").err().expect("invalid relation");
            assert!(error.contains(code),"{source}: {error}");
        }
    }
    #[test]
    fn derived_graph_activates_in_topological_order_and_settles_after_each_madi() {
        let source="채비 { 원천:셈수 := 1.0. }. 최종 <<- 중간 + 1.0. 중간 <<- 원천 * 2.0. (매마디)마다 { 원천 <- 원천 + 1.0. }.";
        let mut runner=runner(source);let mut world=DetNuri::new();
        let first=runner.run_update(world.world(),&input(0),&HashMap::new()).unwrap();
        assert_eq!(first.resources["중간"],Value::Fixed64(Fixed64::from_i64(4)));
        assert_eq!(first.resources["최종"],Value::Fixed64(Fixed64::from_i64(5)));
        world.apply_patch(&first.patch,0,&mut ddonirang_core::signals::VecSignalSink::default());
        runner.validate_restored_derived_world(world.world()).unwrap();
        let next=runner.run_update(world.world(),&input(1),&HashMap::new()).unwrap();
        assert_eq!(next.resources["중간"],Value::Fixed64(Fixed64::from_i64(6)));
        assert_eq!(next.resources["최종"],Value::Fixed64(Fixed64::from_i64(7)));
    }
    #[test]
    fn failed_settle_never_publishes_a_partial_world_or_advances_consumed_keys() {
        let source="채비 { 원천:셈수 := 2.0. }. 결과 <<- 1.0 / 원천. (매마디)마다 { 원천 <- 원천 - 1.0. }.";
        let mut runner=runner(source);let mut world=DetNuri::new();
        let first=runner.run_update(world.world(),&input(0),&HashMap::new()).unwrap();
        world.apply_patch(&first.patch,0,&mut ddonirang_core::signals::VecSignalSink::default());
        let before=world.world().state_hash();let keys=runner.consumed_keys_pressed();
        let mut next=input(1);next.keys_pressed=1;
        assert!(runner.run_update(world.world(),&next,&HashMap::new()).is_err());
        assert_eq!(world.world().state_hash(),before);assert_eq!(runner.consumed_keys_pressed(),keys);
        assert_eq!(world.world().get_resource_fixed64("원천"),Some(Fixed64::from_i64(1)));
    }
    #[test]
    fn pure_callable_keeps_argument_and_hidden_world_dependencies() {
        let source="채비 { 원천:셈수 := 1.0. 가산:셈수 := 1.0. }. (값:셈수) 두배:셈수:셈씨 := { 두배 <- 값 * 2.0 + 가산. }. 결과 <<- (원천) 두배. (매마디)마다 { 원천 <- 원천 + 1.0. 가산 <- 가산 + 2.0. }.";
        let mut runner=runner(source);let mut world=DetNuri::new();
        let first=runner.run_update(world.world(),&input(0),&HashMap::new()).unwrap();
        assert_eq!(first.resources["결과"],Value::Fixed64(Fixed64::from_i64(7)));
        world.apply_patch(&first.patch,0,&mut ddonirang_core::signals::VecSignalSink::default());
        let next=runner.run_update(world.world(),&input(1),&HashMap::new()).unwrap();
        assert_eq!(next.resources["결과"],Value::Fixed64(Fixed64::from_i64(11)));
    }
    #[test]
    fn derived_tuple_uses_compiler_owned_positions_for_world_storage() {
        let source="채비 { 원천:셈수 := 1.0. }. 결과 <<- (원천, 1 / 3).";
        let mut runner=runner(source);let mut world=DetNuri::new();
        let first=runner.run_update(world.world(),&input(0),&HashMap::new()).unwrap();
        assert!(matches!(&first.resources["결과"],Value::Tuple(values) if matches!(&values[1],Value::Rational(_))));
        world.apply_patch(&first.patch,0,&mut ddonirang_core::signals::VecSignalSink::default());
        runner.validate_restored_derived_world(world.world()).unwrap();
    }
    #[test]
    fn derived_inputs_never_fall_back_to_host_defaults_or_cached_projections() {
        let program=DdnProgram::from_v25_source("채비 { 원천:셈수 := 1.0. }. 결과 <<- 원천 * 2.0.","derived.ddn").unwrap();
        let defaults=HashMap::from([("원천".into(),Value::Fixed64(Fixed64::from_i64(99)))]);
        let empty=NuriWorld::new();
        let mut ctx=EvalContext::new(&program,&empty,&defaults,None,InputState::new(0,0),0,0,None);
        ctx.resources.insert("원천".into(),Value::Fixed64(Fixed64::from_i64(99)));
        assert!(ctx.settle_derived(&mut HashMap::new()).unwrap_err().to_string().contains("E_DERIVED_DEPENDENCY_REQUIRED"));
        assert!(ctx.patch_ops.is_empty());
        let mut world=NuriWorld::new();world.set_resource_fixed64("원천".into(),Fixed64::from_i64(3));
        let mut ctx=EvalContext::new(&program,&world,&defaults,None,InputState::new(0,0),0,0,None);
        ctx.resources.insert("원천".into(),Value::Fixed64(Fixed64::from_i64(99)));
        ctx.settle_derived(&mut HashMap::new()).unwrap();
        assert_eq!(ctx.resources.get("결과"),Some(&Value::Fixed64(Fixed64::from_i64(6))));
    }
    #[test]
    fn restore_recomputes_from_source_and_rejects_wrong_or_missing_derived_values() {
        let source="채비 { 원천:셈수 := 1.0. }. 결과 <<- 원천 * 2.0.";
        let mut producer=runner(source);let mut world=DetNuri::new();
        let first=producer.run_update(world.world(),&input(0),&HashMap::new()).unwrap();
        world.apply_patch(&first.patch,0,&mut ddonirang_core::signals::VecSignalSink::default());
        let consumer=runner(source);consumer.validate_restored_derived_world(world.world()).unwrap();
        let mut wrong=world.world().clone();wrong.set_resource_fixed64("결과".into(),Fixed64::from_i64(99));let before=wrong.state_hash();
        assert!(consumer.validate_restored_derived_world(&wrong).unwrap_err().contains("E_DERIVED_RECOVERY_REQUIRED"));assert_eq!(wrong.state_hash(),before);
        assert!(consumer.validate_restored_derived_world(&NuriWorld::new()).unwrap_err().contains("E_DERIVED_RECOVERY_REQUIRED"));
    }
}
