//! Compilation-local source-owned function-call bindings. Not a public
//! FunctionArtifact/EffectAtom registry, Native grant or serialization format.
use std::{collections::{BTreeMap,BTreeSet},sync::Arc};
use ddonirang_lang::{CanonProgram,Expr,ExprKind,NodeId,SeedDef,TopLevelItem,TypeRef};

#[derive(Clone,Debug)]
pub(crate) struct SourceCall {
    call:NodeId,
    definition:Arc<SeedDef>,
    authored_pins:Vec<NodeId>,
    argument_nodes:Vec<(NodeId,NodeId)>,
    parameter_order:Vec<usize>,
    evaluation_order:Vec<usize>,
}
impl SourceCall {
    /// The source callable is selected independently of an expected callback.
    /// Only a unique, unshadowed actual declaration can supply these pins.
    pub(crate) fn expected_arguments(program:&CanonProgram,expr:&Expr,shadowed:bool)->Result<Option<Self>,String>{
        if shadowed||program.source_epoch!=ddonirang_lang::SourceEpoch::V25{return Ok(None);}
        let ExprKind::Call{func,..}=&expr.kind else{return Ok(None)};
        use ddonirang_lang::ordinary_callable::{resolve_scalar_math_callable,ScalarMathCallableOwner};
        let canonical=ddonirang_lang::stdlib::canonicalize_stdlib_alias(func);
        let declared=matches!(resolve_scalar_math_callable(program,func),Some(ScalarMathCallableOwner::Declared(_)));
        if !declared && !crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule|rule.is_none()){return Ok(None);}
        let mut candidates=program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed)
            .filter(|seed|seed.callable_surface_names().iter().any(|name|name==func));
        let Some(seed)=candidates.next()else{return Ok(None)};
        if candidates.next().is_some()||!seed.type_params.is_empty()
            ||!seed.params.iter().any(|pin|matches!(pin.type_ref,TypeRef::Function{..})){return Ok(None);}
        Self::bind(program,expr,seed.id).map(Some)
    }

    /// Select the actual parsed owner once; spelling is not used at execution.
    pub(crate) fn bind(program:&CanonProgram,expr:&Expr,definition:NodeId)->Result<Self,String>{
        let ExprKind::Call{args,func}=&expr.kind else{return Err("E_FUNCTION_CALL_SOURCE: 실제 호출 노드가 필요합니다".into())};
        let mut found=program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed).filter(|seed|seed.id==definition);
        let seed=found.next().ok_or("E_FUNCTION_CALL_OWNER: 실제 정의 owner가 없습니다")?;
        if found.next().is_some()||!seed.callable_surface_names().iter().any(|surface|surface==func){return Err("E_FUNCTION_CALL_OWNER: 정의 소유와 호출 표면이 다릅니다".into());}
        for candidates in [program.origin.authored_call_candidates.get(&expr.id),program.origin.registered_call_candidates.get(&expr.id)].into_iter().flatten(){
            if !candidates.contains(&seed.id){return Err("E_FUNCTION_CALL_OWNER: 원문 해소의 실제 정의와 다릅니다".into());}
        }
        let mut supplied=BTreeMap::new();let mut authored_pins=Vec::with_capacity(args.len());
        for (ordinal,arg) in args.iter().enumerate(){
            let pin=if let Some(name)=&arg.resolved_pin{seed.params.iter().find(|pin|&pin.pin_name==name)}
                else if arg.josa.is_none(){seed.params.get(ordinal)}else{None}
                .ok_or("E_FUNCTION_CALL_PIN_OWNER: 실제 받는자리의 유일 해소가 필요합니다")?;
            if supplied.insert(pin.id,ordinal).is_some(){return Err("E_FUNCTION_CALL_PIN_DUPLICATE: 같은 받는자리에 인자가 겹칩니다".into());}
            authored_pins.push(pin.id);
        }
        let parameter_order=seed.params.iter().map(|pin|supplied.get(&pin.id).copied()
            .ok_or_else(||"E_FUNCTION_CALL_REQUIRED_PIN: 해소된 실제 인자가 부족합니다".to_string())).collect::<Result<Vec<_>,_>>()?;
        if supplied.len()!=seed.params.len(){return Err("E_FUNCTION_CALL_PIN_OWNER: 다른 정의의 받는자리입니다".into());}
        Ok(Self{call:expr.id,definition:Arc::new(seed.clone()),authored_pins,argument_nodes:args.iter().map(|a|(a.id,a.expr.id)).collect(),parameter_order,evaluation_order:program.origin.argument_evaluation_order(expr.id,args)?})
    }
    pub(crate) fn same_binding(&self,other:&Self)->bool{self.call==other.call&&self.definition.id==other.definition.id&&self.authored_pins==other.authored_pins&&self.argument_nodes==other.argument_nodes&&self.parameter_order==other.parameter_order&&self.evaluation_order==other.evaluation_order}
    pub(crate) fn call(&self)->NodeId{self.call}
    pub(crate) fn definition(&self)->&SeedDef{&self.definition}
    pub(crate) fn authored_types(&self)->Result<Vec<&TypeRef>,String>{self.authored_pins.iter().map(|id|
        self.definition.params.iter().find(|pin|pin.id==*id).map(|pin|&pin.type_ref)
            .ok_or_else(||"E_FUNCTION_CALL_PIN_OWNER: 봉인한 받는자리가 정의에 없습니다".into())).collect()}
    pub(crate) fn parameter_order(&self)->&[usize]{&self.parameter_order}
    pub(crate) fn evaluation_order(&self)->&[usize]{&self.evaluation_order}
    pub(crate) fn verify(&self,expr:&Expr)->Result<(),String>{
        let ExprKind::Call{args,..}=&expr.kind else{return Err("E_FUNCTION_CALL_SOURCE: 실제 호출 노드가 다릅니다".into())};
        if expr.id!=self.call||args.iter().map(|a|(a.id,a.expr.id)).collect::<Vec<_>>()!=self.argument_nodes||args.len()!=self.authored_pins.len()||self.parameter_order.iter().copied().collect::<BTreeSet<_>>().len()!=args.len()||self.evaluation_order.iter().copied().collect::<BTreeSet<_>>().len()!=args.len()||self.evaluation_order.iter().any(|index|*index>=args.len()){
            return Err("E_FUNCTION_CALL_SOURCE: 봉인한 호출·받는자리 수가 다릅니다".into());
        }Ok(())
    }
}

/// The actual typed anonymous source root, before runtime values exist.
#[derive(Clone,Debug)]
pub(crate) struct FunctionRoot {
    pub body:Expr,
    pub parameter_names:Vec<String>,
    pub function_type:TypeRef,
}
#[derive(Clone,Debug,Default)]
pub(crate) struct FunctionGraphs(BTreeMap<NodeId,Arc<CheckedGraph>>);
#[derive(Clone,Debug)]
pub(crate) struct SourceCaptureBinding {pub scope:NodeId,pub declaration:NodeId,pub name:String,pub kind:TypeRef}
#[derive(Clone,Debug)]
pub(crate) struct CheckedGraph {
    root:NodeId,
    calls:BTreeMap<NodeId,SourceCall>,
    globals:BTreeMap<NodeId,SourceCaptureBinding>,
    callee_globals:BTreeMap<NodeId,Vec<SourceCaptureBinding>>,
}
impl FunctionGraphs {
    pub(crate) fn get(&self,root:NodeId)->Option<Arc<CheckedGraph>>{self.0.get(&root).cloned()}
    pub(crate) fn source_capture_owners(&self)->BTreeSet<NodeId>{self.0.values().flat_map(|graph|graph.globals.keys().copied()).collect()}
    fn seed_edges(seed:&SeedDef,plan:&crate::numeric_constraint_plan::NumericConstraintPlan)->Result<(BTreeSet<NodeId>,Vec<SourceCaptureBinding>),String>{
        // Empty effects are proved from actual closed source bodies. Neither
        // builtin spelling nor a work-cost row issues an EffectAtomRef.
        if !matches!(seed.seed_kind,ddonirang_lang::SeedKind::Semssi|ddonirang_lang::SeedKind::ValueFunc)
            || !seed.type_params.is_empty() || seed.proof_header.is_some()
            || !seed.modifiers.is_empty() || seed.result_type.is_none()
            || seed.params.iter().any(|pin|matches!(pin.type_ref,TypeRef::Infer)||pin.optional||pin.default_value.is_some()){
            return Err("E_FUNCTION_BODY_EFFECT: 실제 순수 원문 정의의 완결된 입력·결과·효과 공급이 필요합니다".into());
        }
        let body=seed.body.as_ref().ok_or("E_FUNCTION_CALL_GRAPH_UNSUPPORTED: 호출할 실제 원문 본문이 없습니다")?;
        let mut bound:std::collections::HashSet<_>=seed.params.iter().map(|p|p.pin_name.clone()).collect();
        bound.insert(seed.canonical_name.clone());
        let (free,edges)=crate::source_analysis::function_statement_capture_sites(&body.stmts,&bound)?;
        let mut captures=BTreeMap::new();
        for (site,name) in free{
            let (scope,declaration)=plan.source_capture_reads.get(&site).copied().flatten()
                .ok_or("E_FUNCTION_CALLEE_CAPTURE_SOURCE_REQUIRED: 자유변수 읽기의 실제 선언 소유가 없거나 문맥마다 다릅니다")?;
            let binding=plan.source_capture_bindings.get(&declaration).filter(|binding|binding.scope==scope&&binding.name==name)
                .ok_or("E_FUNCTION_CALLEE_CAPTURE_SOURCE_REQUIRED: 실제 해소된 바깥 선언·씨가 없거나 세계 값입니다")?;
            captures.insert(declaration,binding.clone());
        }
        Ok((edges,captures.into_values().collect()))
    }
    fn located(error:String,span:ddonirang_lang::Span)->String{format!("{error}; 원문 {}..{}",span.start,span.end)}
    fn call_error(program:&CanonProgram,id:NodeId,error:String)->String{
        program.origin.node_spans.get(&id).copied().map_or(error.clone(),|span|Self::located(error,span))
    }
    fn close(program:&CanonProgram,root:NodeId,mut pending:BTreeSet<NodeId>,initial:Option<(NodeId,Vec<SourceCaptureBinding>)>,plan:&crate::numeric_constraint_plan::NumericConstraintPlan)->Result<Arc<CheckedGraph>,String>{
        let mut calls=BTreeMap::new();let mut visited=BTreeSet::new();
        let mut globals=BTreeMap::new();let mut callee_globals=BTreeMap::new();
        if let Some((owner,bindings))=initial{
            for binding in &bindings{globals.insert(binding.declaration,binding.clone());}
            callee_globals.insert(owner,bindings);visited.insert(owner);
        }
        while let Some(call)=pending.pop_first(){
            if let Some(reason)=plan.source_call_rejections.get(&call){return Err(Self::call_error(program,call,reason.clone()));}
            let target=plan.source_calls.get(&call).ok_or_else(||
                Self::call_error(program,call,"E_FUNCTION_CALL_GRAPH_UNSUPPORTED: 실제 정의·받는자리·효과가 해소된 원문 호출 공급이 필요합니다".to_string()))?;
            calls.insert(call,target.clone());
            if visited.insert(target.definition().id){
                let (edges,bindings)=Self::seed_edges(target.definition(),plan).map_err(|e|Self::located(e,target.definition().span))?;
                for binding in &bindings{globals.insert(binding.declaration,binding.clone());}
                callee_globals.insert(target.definition().id,bindings);
                pending.extend(edges.into_iter().filter(|edge|!calls.contains_key(edge)));
            }
        }
        Ok(Arc::new(CheckedGraph{root,calls,globals,callee_globals}))
    }
    pub(crate) fn compile(program:&CanonProgram,plan:&crate::numeric_constraint_plan::NumericConstraintPlan)->Result<Self,String>{
        let mut graphs=BTreeMap::new();
        for (id,root) in &plan.function_roots {
            if root.body.id!=*id || !matches!(root.function_type,TypeRef::Function{..}) {
                return Err("E_FUNCTION_CALL_SOURCE: 함수 본문·기대 함수씨가 다릅니다".into());
            }
            let bound=root.parameter_names.iter().cloned().collect();
            let (_,pending)=crate::source_analysis::function_expression_inventory(&root.body,&bound).map_err(|e|Self::located(e,root.body.span))?;
            graphs.insert(*id,Self::close(program,*id,pending,None,plan)?);
        }
        for (id,seed) in &plan.named_function_roots{
            if seed.id!=*id{return Err("E_FUNCTION_CALL_OWNER: 함수 원문 정의가 다릅니다".into());}
            let (edges,bindings)=Self::seed_edges(seed,plan).map_err(|e|Self::located(e,seed.span))?;
            graphs.insert(*id,Self::close(program,*id,edges,Some((*id,bindings)),plan)?);
        }
        Ok(Self(graphs))
    }
}
impl CheckedGraph {
    pub(crate) fn root(&self)->NodeId{self.root}
    pub(crate) fn globals(&self)->impl Iterator<Item=&SourceCaptureBinding>{self.globals.values()}
    pub(crate) fn callee_globals(&self,definition:NodeId)->Option<&[SourceCaptureBinding]>{self.callee_globals.get(&definition).map(Vec::as_slice)}
    pub(crate) fn call(&self,id:NodeId)->Option<SourceCall>{self.calls.get(&id).cloned()}
}
