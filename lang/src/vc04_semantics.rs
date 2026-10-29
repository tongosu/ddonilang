//! VC04 declaration-owned static effects and the command/value boundary.
//! No spelling/category inference, constant-folding exemption, or world state.
use crate::ast::*;
use crate::parser::ParseError;
use std::collections::{HashMap,HashSet,VecDeque};

fn reject(span:Span,code:&str,reason:&str)->ParseError {ParseError{span,message:format!("{code}: {reason}")}}
#[derive(Default)]
struct Effects {write:Option<Span>,read:Option<Span>,calls:Vec<(String,Span)>,policy_sides:HashMap<NodeId,&'static str>}

// A conservative execution-only qualifier, separate from public TypeRef T.
// A field projection is checked by the resolved runtime slot owner. General
// calls preserve an incoming qualifier; they cannot launder it by renaming T.
fn transient(expr:&Expr,aliases:&HashSet<&str>,program:&CanonProgram)->bool {
    let mut pending=vec![expr];
    while let Some(expr)=pending.pop(){
        if program.origin.subject_reference_calls.get(&expr.id).is_some_and(|binding|binding.definition=="VC04.DEF.ILGI.R1"){return true;}
        match &expr.kind {
            ExprKind::Var(name) if aliases.contains(name.as_str())=>return true,
            ExprKind::Call{func,args}=>{if aliases.contains(func.as_str()){return true;}pending.extend(args.iter().map(|arg|&arg.expr));},
            ExprKind::ProductConstruction{fields:args,..}=>pending.extend(args.iter().map(|arg|&arg.expr)),
            ExprKind::SumBranchValue{payload,..}=>pending.extend(payload.iter().map(|arg|&arg.value)),
            ExprKind::CollectionConstructor{elements}|ExprKind::TupleConstructor{elements}=>pending.extend(elements),
            ExprKind::Pack{fields}|ExprKind::TemplateRender{inject:fields,..}|ExprKind::FormulaEval{inject:fields,..}=>pending.extend(fields.iter().map(|(_,value)|value)),
            ExprKind::Infix{left,right,..}=>{pending.push(left);pending.push(right);},
            ExprKind::Pipe{stages}=>pending.extend(stages),
            ExprKind::SelectionPlace{target,index}=>{pending.push(target);pending.push(index);},
            ExprKind::MultiAxisSelection(selection)=>pending.extend(selection.children()),
            ExprKind::Nuance{expr,..}|ExprKind::Suffix{value:expr,..}=>pending.push(expr),
            ExprKind::Tensor(value)=>pending.push(value.input()),
            ExprKind::SeedLiteral{body,..}=>{if !aliases.is_empty(){return true;}pending.push(body);},
            ExprKind::FieldAccess{..}=>{},
            _=>{},
        }
    }
    false
}
fn transient_place<'a>(mut expr:&'a Expr,aliases:&HashSet<&str>)->bool {
    loop {match &expr.kind {
        ExprKind::Var(name)=>return aliases.contains(name.as_str()),
        ExprKind::FieldAccess{target,..}|ExprKind::SelectionPlace{target,..}=>expr=target,
        ExprKind::MultiAxisSelection(selection)=>expr=&selection.target,
        _=>return false,
    }}
}
enum Job<'a>{Body(&'a Body),Stmt(&'a Stmt),Expr(&'a Expr,bool),BindLocal(&'a str,&'a Expr),TransientBinder(&'a str),Restore(HashSet<&'a str>,HashSet<&'a str>)}
fn scan(program:&CanonProgram,body:Option<&Body>,root:&[Stmt],params:&[ParamPin])->Result<Effects,ParseError>{
    let types:HashMap<_,_>=params.iter().map(|pin|(pin.pin_name.as_str(),&pin.type_ref)).collect();
    let mut locals:HashSet<_>=params.iter().map(|pin|pin.pin_name.as_str()).collect();
    // Only statically certain non-reference values are tracked here. Unknown
    // calls/types remain the existing resolver's responsibility, never guessed.
    let mut non_references:HashSet<&str>=HashSet::new();
    let mut transient_values:HashSet<&str>=HashSet::new();
    let mut effects=Effects::default();let mut jobs=Vec::new();
    if let Some(body)=body {jobs.push(Job::Body(body));}else{jobs.extend(root.iter().rev().map(Job::Stmt));}
    while let Some(job)=jobs.pop(){match job {
        Job::TransientBinder(name)=>{locals.insert(name);transient_values.insert(name);},
        Job::BindLocal(name,value)=>{
            if transient(value,&transient_values,program){transient_values.insert(name);}else{transient_values.remove(name);}
            let non_reference=match &value.kind {
                ExprKind::Literal(_)|ExprKind::ProductConstruction{..}|ExprKind::TupleConstructor{..}|ExprKind::CollectionConstructor{..}=>true,
                ExprKind::Var(source)=>non_references.contains(source.as_str()),
                _=>false,
            };
            locals.insert(name);
            if non_reference{non_references.insert(name);}else{non_references.remove(name);}
        },
        Job::Restore(previous,previous_types)=>{
            locals=previous;
            // A nested branch may replace an outer value. Keep only facts
            // true both before and after it, not stale pre-branch guesses.
            non_references=previous_types.intersection(&non_references).copied().collect();
        },
        Job::Body(body)=>{jobs.push(Job::Restore(locals.clone(),non_references.clone()));jobs.extend(body.stmts.iter().rev().map(Job::Stmt));},
        Job::Stmt(stmt)=>match stmt {
            Stmt::DeclBlock{items,..}=>for item in items {if let Some(value)=&item.value {
                if transient(value,&transient_values,program){return Err(reject(value.span,"E_VC04_READ_SNAPSHOT_ESCAPE","채비에는 임자 읽기값을 보존할 수 없습니다"));}
                jobs.push(Job::Expr(value,false));
            }},
            Stmt::Definition{name,value,..}|Stmt::DerivedBinding{name,value,..}=>{
                if body.is_none()&&transient(value,&transient_values,program){return Err(reject(value.span,"E_VC04_READ_SNAPSHOT_ESCAPE","최상위 정의에 임자 읽기값을 보존할 수 없습니다"));}
                jobs.push(Job::BindLocal(name,value));jobs.push(Job::Expr(value,false));
            },
            Stmt::Return{value,..}=>{if transient(value,&transient_values,program){return Err(reject(value.span,"E_VC04_READ_SNAPSHOT_ESCAPE","임자 읽기값은 실행 활성화 밖으로 반환할 수 없습니다"));}jobs.push(Job::Expr(value,false));},
            Stmt::Mutate{target,value,..}=>{
                if transient_place(target,&transient_values){return Err(reject(target.span,"E_VC04_READ_ONLY_PLACE","임자 읽기값은 다시 쓸 수 있는 위치가 아닙니다"));}
                if transient(value,&transient_values,program)&&!matches!(&target.kind,ExprKind::Var(name) if locals.contains(name.as_str())) {
                    return Err(reject(value.span,"E_VC04_READ_SNAPSHOT_ESCAPE","임자 읽기값을 세계 상태나 지속 위치에 넣을 수 없습니다"));
                }
                if let ExprKind::Var(name)=&target.kind {jobs.push(Job::BindLocal(name,value));}else{jobs.push(Job::Expr(target,false));}jobs.push(Job::Expr(value,false));},
            Stmt::ResultFlow{producer,destination,..}=>{jobs.push(Job::Expr(producer,false));jobs.push(Job::Expr(destination,false));},
            Stmt::Expr{expr,..}=>jobs.push(Job::Expr(expr,true)),
            Stmt::Show{expr,..}|Stmt::Inspect{expr,..}=>jobs.push(Job::Expr(expr,false)),
            Stmt::Receive{sender,condition,body,..}=>{jobs.push(Job::Body(body));if let Some(e)=sender {jobs.push(Job::Expr(e,false));}if let Some(e)=condition {jobs.push(Job::Expr(e,false));}},
            Stmt::Send{sender,payload,receiver,..}=>{if let Some(e)=sender {jobs.push(Job::Expr(e,false));}jobs.push(Job::Expr(payload,false));jobs.push(Job::Expr(receiver,false));},
            Stmt::If{condition,then_body,else_body,..}=>{jobs.push(Job::Expr(condition,false));jobs.push(Job::Body(then_body));if let Some(body)=else_body {jobs.push(Job::Body(body));}},
            Stmt::Choose{branches,else_body,..}=>{jobs.push(Job::Body(else_body));for branch in branches {jobs.push(Job::Expr(&branch.condition,false));jobs.push(Job::Body(&branch.body));}},
            Stmt::Try{action,body,..}=>{jobs.push(Job::Expr(action,false));jobs.push(Job::Body(body));},
            Stmt::While{condition,body,..}|Stmt::Guard{condition,body,..}|Stmt::HookWhenBecomes{condition,body,..}|Stmt::HookWhile{condition,body,..}=>{jobs.push(Job::Expr(condition,false));jobs.push(Job::Body(body));},
            Stmt::Repeat{body,..}|Stmt::BeatBlock{body,..}|Stmt::ForEach{body,..}|Stmt::Quantifier{body,..}|Stmt::Hook{body,..}=>{
                jobs.push(Job::Body(body));if let Stmt::ForEach{iterable,..}=stmt {jobs.push(Job::Expr(iterable,false));}},
            Stmt::Contract{condition,then_body,else_body,..}=>{jobs.push(Job::Expr(condition,false));jobs.push(Job::Body(else_body));if let Some(body)=then_body {jobs.push(Job::Body(body));}},
            Stmt::SumMatch{subject,arms,default_body,..}=>{
                let qualified=transient(subject,&transient_values,program);
                jobs.push(Job::Expr(subject,false));
                for arm in arms {
                    jobs.push(Job::Body(&arm.body));
                    if qualified&&arm.branch=="잘됨" {
                        for binder in &arm.binder {jobs.push(Job::TransientBinder(&binder.pin));}
                    }
                }
                if let Some(body)=default_body {jobs.push(Job::Body(body));}
            },
            Stmt::PublicObservation{fields,..}=>{
                for field in fields {if transient(&field.value,&transient_values,program){return Err(reject(field.value.span,"E_VC04_READ_SNAPSHOT_ESCAPE","임자 읽기값을 관찰 이력에 저장할 수 없습니다"));}}
                jobs.extend(fields.iter().map(|field|Job::Expr(&field.value,false)));
            },
            Stmt::SumDefinition{..}|Stmt::MetaBlock{..}|Stmt::Pragma{..}|Stmt::Break{..}|Stmt::ContinueLoop{..}=>{},
        },
        Job::Expr(expr,command_position)=>{
            if let ExprKind::Call{func,args}=&expr.kind {
                let canonical=crate::stdlib::canonicalize_stdlib_alias(func);
                if canonical=="표준보개.곁"&&!locals.contains(func.as_str())&&!locals.contains(canonical)
                    &&!program.items.iter().any(|TopLevelItem::SeedDef(seed)|seed.callable_surface_names().iter().any(|name|name==func||name==canonical)) {
                    let signature=crate::stdlib::minimal_stdlib_sigs().into_iter().find(|signature|signature.name==canonical)
                        .ok_or_else(||reject(expr.span,"E_ATOM_POLICY_OWNER_REQUIRED","표준 곁의 기존 서명이 없습니다"))?;
                    let mut occupied=vec![false;signature.params.len()];let mut placement=None;
                    for arg in args {
                        let slot=if let Some(pin)=arg.resolved_pin.as_deref(){signature.params.iter().position(|name|*name==pin)}
                            else{occupied.iter().position(|value|!*value)}
                            .ok_or_else(||reject(arg.span,"E_CALL_PIN_UNKNOWN","표준 곁의 받는자리를 해소할 수 없습니다"))?;
                        if std::mem::replace(&mut occupied[slot],true){return Err(reject(arg.span,"E_CALL_PIN_DUPLICATE","표준 곁의 받는자리가 겹칩니다"));}
                        if signature.params[slot]=="자리"{placement=Some(&arg.expr);}
                    }
                    let place=placement.ok_or_else(||reject(expr.span,"E_CALL_REQUIRED_PIN_MISSING","표준 곁의 자리 받는자리가 없습니다"))?;
                    let ExprKind::Literal(Literal::Atom(label))=&place.kind else {
                        return Err(reject(place.span,"E_ATOM_POLICY_ROLE_REQUIRED","곁 자리에는 기존 위치 policy의 명시 #표기가 필요합니다"));
                    };
                    let side=crate::runtime::SidePlacement::from_authored(label)
                        .ok_or_else(||reject(place.span,"E_BOGAE_UNSUPPORTED","알 수 없는 곁 위치 policy입니다"))?;
                    effects.policy_sides.insert(place.id,side.authored_name());
                }
            }
            if matches!(expr.kind,ExprKind::SeedLiteral{..})&&!transient_values.is_empty(){return Err(reject(expr.span,"E_VC04_READ_SNAPSHOT_ESCAPE","임자 읽기값을 함수에 캡처할 수 없습니다"));}
            if let ExprKind::Call{func,args}=&expr.kind {
                if func!="차림.값"&&args.iter().any(|arg|transient(&arg.expr,&transient_values,program)) {
                    return Err(reject(expr.span,"E_VC04_READ_SNAPSHOT_ESCAPE","읽기값은 다른 호출이나 활성화로 넘기지 말고 필요한 붙이를 먼저 조회하세요"));
                }
            }
            if let Some(binding)=program.origin.subject_reference_calls.get(&expr.id){
                if !matches!((binding.definition,binding.receiver_pin),("VC04.DEF.GARIKIGI.R1","VC04.PIN.GARIKIGI.TARGET.R1")|("VC04.DEF.ILGI.R1","VC04.PIN.ILGI.REFERENCE.R1")) {
                    return Err(reject(expr.span,"E_VC04_REFERENCE_BINDING","가리키기의 정의·받는자리 정체가 다릅니다"));
                }
                if let ExprKind::Call{func,..}=&expr.kind {if locals.contains(func.as_str()){return Err(reject(expr.span,"E_VC04_REFERENCE_AMBIGUOUS","같은 이름의 지역 값과 등록 참조 동작이 경쟁합니다"));}}
                effects.read=Some(expr.span);
            }
            if let Some(binding)=program.origin.retirement_commands.get(&expr.id){
                if binding.definition!="VC04.DEF.PYEGIHAGI.R1"||binding.target_pin!="VC04.PIN.PYEGIHAGI.TARGET.R1"{return Err(reject(expr.span,"E_VC04_RETIRE_BINDING","폐기 정의·받는자리 정체가 다릅니다"));}
                if !command_position{return Err(reject(expr.span,"E_VC04_COMMAND_VALUE","폐기하기는 결과값 없는 명령입니다. 값 자리에 쓸 수 없습니다"));}
                if locals.contains("폐기하기"){return Err(reject(expr.span,"E_VC04_RETIRE_AMBIGUOUS","같은 이름의 지역 값과 등록 폐기 명령이 경쟁합니다"));}
                effects.write=Some(expr.span);
                if let ExprKind::Call{args,..}=&expr.kind {
                    let target=&args[0].expr;
                    let declared=if let ExprKind::Var(name)=&target.kind {types.get(name.as_str()).copied()}else{None};
                    if matches!(target.kind,ExprKind::Literal(_)|ExprKind::ProductConstruction{..}|ExprKind::TupleConstructor{..}|ExprKind::CollectionConstructor{..})
                        ||matches!(&target.kind,ExprKind::Var(name) if non_references.contains(name.as_str()))
                        ||declared.is_some_and(|ty|!matches!(ty,TypeRef::Applied{name,args} if name=="가리킴"&&args.len()==1&&matches!(&args[0],TypeRef::Named(owner) if program.items.iter().any(|TopLevelItem::SeedDef(seed)|seed.canonical_name==*owner&&seed.seed_kind==SeedKind::Schema(SchemaCategory::Subject))))) {
                        return Err(reject(target.span,"E_VC04_RETIRE_TARGET","폐기 대상에는 임자를 가리키는 가리킴이 필요합니다"));
                    }
                }
            }
            match &expr.kind {
                ExprKind::ProductConstruction{category,fields,..}=>{if *category==Some(SchemaCategory::Subject){effects.write=Some(expr.span);}jobs.extend(fields.iter().map(|arg|Job::Expr(&arg.expr,false)));},
                ExprKind::Call{func,args}=>{effects.calls.push((func.clone(),expr.span));jobs.extend(args.iter().map(|arg|Job::Expr(&arg.expr,false)));},
                ExprKind::SumBranchValue{payload,..}=>jobs.extend(payload.iter().map(|arg|Job::Expr(&arg.value,false))),
                ExprKind::Tensor(value)=>jobs.push(Job::Expr(value.input(),false)),
                ExprKind::MultiAxisSelection(value)=>jobs.extend(value.children().map(|e|Job::Expr(e,false))),
                ExprKind::FieldAccess{target,..}=>jobs.push(Job::Expr(target,false)),
                ExprKind::SeedLiteral{param,body,..}=>{
                    jobs.push(Job::Restore(locals.clone(),non_references.clone()));
                    locals.insert(param.as_str());
                    jobs.push(Job::Expr(body,false));
                },
                ExprKind::CollectionConstructor{elements}|ExprKind::TupleConstructor{elements}|ExprKind::Pipe{stages:elements}=>jobs.extend(elements.iter().map(|e|Job::Expr(e,false))),
                ExprKind::SelectionPlace{target,index}=>{jobs.push(Job::Expr(target,false));jobs.push(Job::Expr(index,false));},
                ExprKind::Infix{left,right,..}=>{jobs.push(Job::Expr(left,false));jobs.push(Job::Expr(right,false));},
                ExprKind::Suffix{value,..}=>jobs.push(Job::Expr(value,false)),
                ExprKind::Thunk(body)=>jobs.push(Job::Body(body)),
                ExprKind::Eval{thunk,..}=>jobs.push(Job::Expr(thunk,false)),
                ExprKind::Pack{fields}|ExprKind::TemplateRender{inject:fields,..}|ExprKind::FormulaEval{inject:fields,..}=>jobs.extend(fields.iter().map(|(_,e)|Job::Expr(e,false))),
                ExprKind::Nuance{expr,..}=>jobs.push(Job::Expr(expr,false)),
                ExprKind::Var(name)=>{
                    if name=="폐기하기"&&!locals.contains(name.as_str()){return Err(reject(expr.span,"E_VC04_RETIRE_PIN","폐기하기는 대상 받는자리 하나를 명시한 명령으로만 사용합니다"));}
                    // A bare name can be the existing niladic call surface;
                    // a parameter/local value with that name is not that call.
                    if !locals.contains(name.as_str())&&program.items.iter().any(|TopLevelItem::SeedDef(seed)|!seed.seed_kind.is_schema()&&seed.params.is_empty()&&seed.callable_surface_names().contains(name)) {
                        effects.calls.push((name.clone(),expr.span));
                    }
                },
                ExprKind::Literal(_)|ExprKind::FlowValue|ExprKind::Assertion(_)|ExprKind::StateMachine(_)|ExprKind::Formula(_)|ExprKind::Template(_)=>{},
            }
        }
    }}
    Ok(effects)
}
pub(crate) fn validate(program:&mut CanonProgram)->Result<(),ParseError>{
    if program.source_epoch!=SourceEpoch::V25{return Ok(());}
    let mut summaries=HashMap::new();let mut pure=Vec::new();let mut aliases=HashMap::new();
    for TopLevelItem::SeedDef(seed) in &program.items {
        if seed.seed_kind.is_schema(){continue;}
        for surface in seed.callable_surface_names(){aliases.insert(surface,seed.canonical_name.clone());}
        if matches!(seed.seed_kind,SeedKind::Semssi|SeedKind::ValueFunc) {pure.push(seed);}
        summaries.insert(seed.canonical_name.clone(),scan(program,seed.body.as_ref(),&[],&seed.params)?);
    }
    let root=scan(program,None,&program.root_stmts,&[])?;
    let mut reverse:HashMap<&str,Vec<&str>>=HashMap::new();let mut writes=HashSet::new();let mut ready=VecDeque::new();
    for (owner,effect) in &summaries {
        if effect.write.is_some(){writes.insert(owner.as_str());ready.push_back(owner.as_str());}
        for (call,_) in &effect.calls {if let Some(target)=aliases.get(call){reverse.entry(target).or_default().push(owner);}}
    }
    while let Some(target)=ready.pop_front(){if let Some(callers)=reverse.get(target){for caller in callers {if writes.insert(*caller){ready.push_back(*caller);}}}}
    for seed in &pure {if writes.contains(seed.canonical_name.as_str()){
        let effect=&summaries[&seed.canonical_name];
        let span=effect.write.or_else(||effect.calls.iter().find_map(|(call,span)|aliases.get(call).filter(|owner|writes.contains(owner.as_str())).map(|_|*span))).unwrap_or(seed.span);
        return Err(reject(span,"E_VC04_PURE_WORLD_WRITE","셈씨에서는 임자 생성·폐기를 직접 또는 간접으로 실행할 수 없습니다"));
    }}
    let mut reads=HashSet::new();let mut ready=VecDeque::new();
    for (owner,effect) in &summaries {if effect.read.is_some(){reads.insert(owner.as_str());ready.push_back(owner.as_str());}}
    while let Some(target)=ready.pop_front(){if let Some(callers)=reverse.get(target){for caller in callers {if reads.insert(*caller){ready.push_back(*caller);}}}}
    for seed in pure {if reads.contains(seed.canonical_name.as_str()){
        let effect=&summaries[&seed.canonical_name];
        let span=effect.read.or_else(||effect.calls.iter().find_map(|(call,span)|aliases.get(call).filter(|owner|reads.contains(owner.as_str())).map(|_|*span))).unwrap_or(seed.span);
        return Err(reject(span,"E_VC04_PURE_SUBJECT_READ","셈씨에는 가리키기·읽기의 읽기·권한 효과가 배정되지 않았습니다"));
    }}
    for (node,selector) in root.policy_sides.iter().chain(summaries.values().flat_map(|effect|effect.policy_sides.iter())) {
        let occurrence=program.origin.atom_value_occurrences.get_mut(node)
            .ok_or_else(||reject(Span{start:0,end:0},"E_ATOM_POLICY_OWNER_REQUIRED","위치 policy의 실제 원문 발생이 없습니다"))?;
        occurrence.role=AtomValueRole::StandardBogaeSide{selector};
    }
    Ok(())
}
