//! R3 source-owned role candidates, before canon or final Start.
//!
//! This first stage issues sum template/branch/pin candidates from the real
//! admitted source closure. It is NOT a recorded catalog, a runtime capability,
//! or permission to publish a source revision. Final catalog encoding, lexical
//! binder resolution and the protected closure consumer are separate stages.
use std::collections::{BTreeMap,BTreeSet};
use ddonirang_lang::{CanonProgram,Expr,ExprKind,SeedDef,Span,Stmt,TopLevelItem,TypeRef};
mod source;
pub(super) use source::Source;
mod typeproof;
mod math_callable;
pub(super) fn analyze_math_callable_source(
    source: &Source,
    nominals: &super::RegistrationDraft,
    selected_source_span: Span,
) -> Result<math_callable::SourceFacts, String> {
    math_callable::analyze(source, nominals, selected_source_span)
}
mod catalog;
pub(super) use catalog::{ResolvedCatalog,ROLE as RESOLVED_ROLE,PATH as RESOLVED_PATH};
pub(super) fn requires_resolved_program(program:&CanonProgram)->bool {
    let (declarations,matches)=scoped_sum_syntax(program);!declarations.is_empty()||!matches.is_empty()
}
pub(super) fn builtin_manifest_bytes()->Result<Vec<u8>,String> {
    Ok(builtin_result_ids()?.into_iter().chain(builtin_reference_failure_ids()?).flatten().collect())
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct DefinitionId([u8;32]);
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct BranchId([u8;32]);
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct PinId([u8;32]);
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct ScopeId([u8;32]);
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct BinderId([u8;32]);

#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct Site {module:[u8;32],capture:[u8;32],start:u64,end:u64}
#[derive(Debug,PartialEq,Eq)]
struct Template {
    id:DefinitionId,site:Site,lexical_scope:(u64,u64),spelling:String,branches:Vec<Branch>,
}
#[derive(Debug,PartialEq,Eq)]
struct Branch {id:BranchId,spelling:String,pins:Vec<Pin>}
#[derive(Debug,PartialEq,Eq)]
struct Pin {id:PinId,spelling:String,type_bytes:Vec<u8>}

/// A verified source is required even for a candidate. There is no serde
/// constructor, caller-supplied ID map, or final-Start prerequisite here.
#[derive(Debug)]
struct SumRoleCandidates {
    scope:String,templates:Vec<Template>,
}
struct PlannedTemplate {site:Site,lexical_scope:(u64,u64),spelling:String,branches:Vec<PlannedBranch>}
struct PlannedBranch {spelling:String,pins:Vec<String>}

fn fail(message:&str)->String {format!("E_RESOLVED_SUPPLY: {message}")}

// A necessary lower bound of the eventual JSON blob. The final encoder must
// still check its exact bytes; this gate only prevents unbounded candidate
// allocation/entropy work for a catalog that cannot possibly fit R3's budget.
fn charge_role_bytes(total:&mut usize,amount:usize)->Result<(),String> {
    *total=total.checked_add(amount).ok_or_else(||fail("등록 자료 크기 한도를 넘었습니다"))?;
    if *total>1_000_000 {return Err(fail("등록 자료 크기 한도를 넘었습니다"));}
    Ok(())
}

/// Raw bytes come only from the owning transaction's CSPRNG. A zero or
/// collision fails the candidate once; neither retry nor spelling derivation.
fn fresh(used:&mut BTreeSet<[u8;32]>,entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<[u8;32],String> {
    let mut raw=[0;32];entropy(&mut raw)?;
    if raw==[0;32]||!used.insert(raw) {return Err(fail("등록 정체가 비어 있거나 충돌했습니다. 재발급하지 않습니다"));}
    Ok(raw)
}

fn reserved(source:&Source,nominals:&super::RegistrationDraft)->Result<BTreeSet<[u8;32]>,String> {
    let mut used=BTreeSet::new();
    for schema in &nominals.schemas {
        used.extend([schema.definition,schema.name_type]);used.extend(schema.slots.iter().map(|pin|pin.id));
    }
    used.extend(nominals.registry);used.extend(nominals.atoms.iter().map(|atom|atom.id));
    used.extend(source.module_ids()?);
    used.extend(source.captures().iter().map(|(_,capture)|*capture.identity()));
    for id in builtin_result_ids()? {if !used.insert(id) {return Err(fail("기존 정체가 고정 맺음 manifest와 충돌합니다"));}}
    for id in builtin_reference_failure_ids()? {if !used.insert(id) {return Err(fail("기존 정체가 고정 가리킴안됨 manifest와 충돌합니다"));}}
    Ok(used)
}

/// Fixed reviewed builtin manifest, never names hashed into user identities.
/// Its five rows are reserved even when the current source uses no Result.
fn builtin_result_ids()->Result<[[u8;32];5],String> {
    let rows=[
        "1284a7d7f7a53bdcca0e0a60c76e182c0ad51e780af578d9cec2740de2d5c9e2",
        "8ab0d9f38bb046ed820f4c45ea59465f2d5cc4cc7a731327845ad8ad5cf4be5f",
        "ee4c15149e066f0040b45fc905594fb5171b09aa50ab06c6fc3e7df8e25aceb3",
        "8e2542d8bf29b172eeb59d598d1e9f5863aae9690ab19154aaf4429a56257265",
        "fea5107468cc1a74c531d629c5b914a4a4519368ba5678a2a941c303c3658fb8",
    ];
    let mut ids=[[0;32];5];
    for (target,row) in ids.iter_mut().zip(rows) {
        *target=hex::decode(row).map_err(|_|fail("고정 맺음 manifest 표기 오류"))?.try_into().map_err(|_|fail("고정 맺음 manifest 길이 오류"))?;
    }
    Ok(ids)
}
fn builtin_reference_failure_ids()->Result<[[u8;32];5],String> {
    let rows=[
        "dea705b0c7b76facbca1c7d1d4bcb40bdd61e37231935354ee88490e8c61d237",
        "f8acfd35dc5778f7a0a2003848f686b2e85cd31c8e34c5b2d2d23afe60cf9a8c",
        "047212203a7a2280c665dc44b6ae604b300eb1668db83b3587402cd364ba73f1",
        "4ecc6d871e62033be2688009bb5c2990908a54ec5635b2da3f4bd57bff9b2194",
        "6ce99c346919e257629f4abdde0e9b0af4c8362eac6b0e9dd5aa0ef2f6baf170",
    ];
    let mut ids=[[0;32];5];
    for (target,row) in ids.iter_mut().zip(rows) {*target=hex::decode(row).map_err(|_|fail("가리킴안됨 manifest 표기 오류"))?.try_into().map_err(|_|fail("가리킴안됨 manifest 길이 오류"))?;}
    Ok(ids)
}

fn module_raw(source:&Source,path:&str)->Result<Option<[u8;32]>,String> {
    source.module(path)
}

/// Scan the real AST without recursion or normalization. Every current
/// statement/expression variant is explicit: a new child-bearing AST kind
/// must not silently make nested declarations disappear from the catalog.
fn sum_declarations(program:&CanonProgram)->Vec<(&Stmt,Span)> {sum_syntax(program).0}
fn sum_syntax(program:&CanonProgram)->(Vec<(&Stmt,Span)>,Vec<&Stmt>) {
    let (declarations,matches)=scoped_sum_syntax(program);
    (declarations.into_iter().map(|(stmt,scope,_)|(stmt,scope)).collect(),matches.into_iter().map(|(stmt,_)|stmt).collect())
}
enum SyntaxJob<'a>{Body(&'a [Stmt],Span),Stmt(&'a Stmt,Span),Expr(&'a Expr,Span)}
fn scoped_sum_syntax(program:&CanonProgram)->(Vec<(&Stmt,Span,Option<&SeedDef>)>,Vec<(&Stmt,Option<&SeedDef>)>) {
    use SyntaxJob as Job;
    let (root_declarations,root_matches)=scan_sum_syntax(vec![Job::Body(&program.root_stmts,Span::new(0,program.origin.source.len()))]);
    let mut declarations=root_declarations.into_iter().map(|(stmt,scope)|(stmt,scope,None)).collect::<Vec<_>>();
    let mut matches=root_matches.into_iter().map(|stmt|(stmt,None)).collect::<Vec<_>>();
    for TopLevelItem::SeedDef(seed) in &program.items {
        let mut jobs=Vec::new();
        if let Some(body)=&seed.body {jobs.push(Job::Body(&body.stmts,body.span));}
        for pin in &seed.params {if let Some(value)=&pin.default_value {jobs.push(Job::Expr(value,seed.span));}}
        let (inner_declarations,inner_matches)=scan_sum_syntax(jobs);
        declarations.extend(inner_declarations.into_iter().map(|(stmt,scope)|(stmt,scope,Some(seed))));
        matches.extend(inner_matches.into_iter().map(|stmt|(stmt,Some(seed))));
    }
    declarations.sort_by_key(|(stmt,_,_)|match stmt {Stmt::SumDefinition{span,..}=>(span.start,span.end),_=>unreachable!()});
    matches.sort_by_key(|(stmt,_)|match stmt {Stmt::SumMatch{span,..}=>(span.start,span.end),_=>unreachable!()});
    (declarations,matches)
}
fn scan_sum_syntax(mut jobs:Vec<SyntaxJob<'_>>)->(Vec<(&Stmt,Span)>,Vec<&Stmt>) {
    use SyntaxJob as Job;
    let mut found=Vec::new();let mut matches=Vec::new();
    while let Some(job)=jobs.pop() {match job {
        Job::Body(stmts,scope)=>jobs.extend(stmts.iter().rev().map(|stmt|Job::Stmt(stmt,scope))),
        Job::Stmt(stmt,scope)=>match stmt {
            Stmt::SumDefinition{..}=>found.push((stmt,scope)),
            Stmt::SumMatch{subject,arms,default_body,..}=>{
                matches.push(stmt);
                jobs.push(Job::Expr(subject,scope));
                for arm in arms {jobs.push(Job::Body(&arm.body.stmts,arm.body.span));}
                if let Some(body)=default_body {jobs.push(Job::Body(&body.stmts,body.span));}
            },
            Stmt::DeclBlock{items,..}=>for item in items {if let Some(value)=&item.value {jobs.push(Job::Expr(value,scope));}},
            Stmt::Definition{value,..}|Stmt::DerivedBinding{value,..}|Stmt::Return{value,..}=>jobs.push(Job::Expr(value,scope)),
            Stmt::Mutate{target,value,..}=>{jobs.push(Job::Expr(target,scope));jobs.push(Job::Expr(value,scope));},
            Stmt::ResultFlow{producer,destination,..}=>{jobs.push(Job::Expr(producer,scope));jobs.push(Job::Expr(destination,scope));},
            Stmt::Expr{expr,..}|Stmt::Show{expr,..}|Stmt::Inspect{expr,..}=>jobs.push(Job::Expr(expr,scope)),
            Stmt::PublicObservation{fields,..}=>jobs.extend(fields.iter().map(|field|Job::Expr(&field.value,scope))),
            Stmt::Receive{sender,condition,body,..}=>{
                for expr in sender.iter().chain(condition.iter()) {jobs.push(Job::Expr(expr,scope));}
                jobs.push(Job::Body(&body.stmts,body.span));
            },
            Stmt::Send{sender,payload,receiver,..}=>{
                if let Some(sender)=sender {jobs.push(Job::Expr(sender,scope));}
                jobs.extend([Job::Expr(payload,scope),Job::Expr(receiver,scope)]);
            },
            Stmt::If{condition,then_body,else_body,..}=>{
                jobs.push(Job::Expr(condition,scope));jobs.push(Job::Body(&then_body.stmts,then_body.span));
                if let Some(body)=else_body {jobs.push(Job::Body(&body.stmts,body.span));}
            },
            Stmt::Try{action,body,..}=>{jobs.push(Job::Expr(action,scope));jobs.push(Job::Body(&body.stmts,body.span));},
            Stmt::Choose{branches,else_body,..}=>{
                jobs.push(Job::Body(&else_body.stmts,else_body.span));
                for branch in branches {jobs.push(Job::Expr(&branch.condition,scope));jobs.push(Job::Body(&branch.body.stmts,branch.body.span));}
            },
            Stmt::Repeat{body,..}|Stmt::Quantifier{body,..}|Stmt::BeatBlock{body,..}|Stmt::Hook{body,..}=>jobs.push(Job::Body(&body.stmts,body.span)),
            Stmt::While{condition,body,..}|Stmt::Guard{condition,body,..}|Stmt::HookWhenBecomes{condition,body,..}|Stmt::HookWhile{condition,body,..}=>{
                jobs.push(Job::Expr(condition,scope));jobs.push(Job::Body(&body.stmts,body.span));
            },
            Stmt::ForEach{iterable,body,..}=>{jobs.push(Job::Expr(iterable,scope));jobs.push(Job::Body(&body.stmts,body.span));},
            Stmt::Contract{condition,then_body,else_body,..}=>{
                jobs.push(Job::Expr(condition,scope));jobs.push(Job::Body(&else_body.stmts,else_body.span));
                if let Some(body)=then_body {jobs.push(Job::Body(&body.stmts,body.span));}
            },
            Stmt::MetaBlock{..}|Stmt::Pragma{..}|Stmt::Break{..}|Stmt::ContinueLoop{..}=>{},
        },
        Job::Expr(expr,scope)=>match &expr.kind {
            ExprKind::Tensor(tensor)=>jobs.push(Job::Expr(tensor.input(),scope)),
            ExprKind::MultiAxisSelection(selection)=>jobs.extend(selection.children().map(|expr|Job::Expr(expr,scope))),
            ExprKind::FieldAccess{target,..}=>jobs.push(Job::Expr(target,scope)),
            ExprKind::SeedLiteral{body,..}=>jobs.push(Job::Expr(body,scope)),
            ExprKind::Call{args,..}|ExprKind::ProductConstruction{fields:args,..}=>jobs.extend(args.iter().map(|arg|Job::Expr(&arg.expr,scope))),
            ExprKind::SumBranchValue{payload,..}=>jobs.extend(payload.iter().map(|arg|Job::Expr(&arg.value,scope))),
            ExprKind::CollectionConstructor{elements}|ExprKind::TupleConstructor{elements}|ExprKind::Pipe{stages:elements}=>jobs.extend(elements.iter().map(|expr|Job::Expr(expr,scope))),
            ExprKind::SelectionPlace{target,index}=>jobs.extend([Job::Expr(target,scope),Job::Expr(index,scope)]),
            ExprKind::Infix{left,right,..}=>jobs.extend([Job::Expr(left,scope),Job::Expr(right,scope)]),
            ExprKind::Suffix{value,..}=>jobs.push(Job::Expr(value,scope)),
            ExprKind::Thunk(body)=>jobs.push(Job::Body(&body.stmts,body.span)),
            ExprKind::Eval{thunk,..}=>jobs.push(Job::Expr(thunk,scope)),
            ExprKind::Pack{fields}|ExprKind::TemplateRender{inject:fields,..}|ExprKind::FormulaEval{inject:fields,..}=>jobs.extend(fields.iter().map(|(_,expr)|Job::Expr(expr,scope))),
            ExprKind::Nuance{expr,..}=>jobs.push(Job::Expr(expr,scope)),
            ExprKind::Literal(_)|ExprKind::Var(_)|ExprKind::FlowValue|ExprKind::Assertion(_)|ExprKind::StateMachine(_)|ExprKind::Formula(_)|ExprKind::Template(_)=>{},
        },
    }}
    found.sort_by_key(|(stmt,_)|match stmt {Stmt::SumDefinition{span,..}=>(span.start,span.end),_=>unreachable!()});
    matches.sort_by_key(|stmt|match stmt {Stmt::SumMatch{span,..}=>(span.start,span.end),_=>unreachable!()});
    (found,matches)
}

/// Reuse the registered nominal/type owner, not a type-name byte encoding or
/// a runtime value. All branches are planned before any identity is issued.
fn plan(source:&Source,nominals:&super::RegistrationDraft)->Result<Vec<PlannedTemplate>,String> {
    source.verify_nominals(nominals)?;
    let mut programs=Vec::new();
    for (path,capture) in source.captures() {
        let Some(module)=module_raw(source,path)? else {continue};
        let program=ddonirang_lang::parse_v25_with_mode(capture.text(),path,ddonirang_lang::ParseMode::Strict)
            .map_err(|error|fail(&error.to_string()))?;
        programs.push((module,capture,program));
    }
    let mut result=Vec::new();let mut declarations=BTreeSet::new();let mut role_bytes=0;
    for (module,capture,program) in &programs {
        let module=*module;
        for (stmt,scope) in sum_declarations(program) {
            let Stmt::SumDefinition{name,branches,span,..}=stmt else {continue};
            if !declarations.insert((module,scope.start,scope.end,name.as_str().to_owned())) {return Err(fail("같은 지역 범위의 고름씨 선언이 중복됩니다"));}
            if span.start>span.end||span.end>capture.bytes().len() {return Err(fail("고름씨 선언의 원문 범위가 다릅니다"));}
            charge_role_bytes(&mut role_bytes,64usize.checked_add(name.len()).ok_or_else(||fail("선언 크기 한도"))?)?;
            let mut planned=Vec::new();let mut branch_names=BTreeSet::new();
            for branch in branches {
                if !branch_names.insert(branch.name.as_str()) {return Err(fail("고름씨 갈래가 중복됩니다"));}
                charge_role_bytes(&mut role_bytes,64usize.checked_add(branch.name.len()).ok_or_else(||fail("갈래 크기 한도"))?)?;
                let mut pins=Vec::new();let mut pin_names=BTreeSet::new();
                for pin in &branch.payload {
                    if !pin_names.insert(pin.binder.as_str()) {return Err(fail("갈래의 받는자리가 중복됩니다"));}
                    // R3.1 reserves all nominal references first. Closed type
                    // proofs are produced below from the actual linked source.
                    let lower=64usize.checked_add(pin.binder.len())
                        .ok_or_else(||fail("받는자리 크기 한도"))?;
                    charge_role_bytes(&mut role_bytes,lower)?;
                    pins.push(pin.binder.clone());
                }
                planned.push(PlannedBranch{spelling:branch.name.clone(),pins});
            }
            result.push(PlannedTemplate{site:Site{module,capture:*capture.identity(),start:span.start as u64,end:span.end as u64},lexical_scope:(scope.start as u64,scope.end as u64),
                spelling:name.clone(),branches:planned});
        }
    }
    Ok(result)
}

// resolve_type already validates encoded container identities. Its nominal
// active-set argument is supplemented with actual current schema declarations,
// not used as a label-based authority for retained/inactive rows.
fn validate_closed_type(kind:&TypeRef,module:[u8;32],schemas:&[super::Schema],active:&BTreeSet<([u8;32],&str)>)->Result<(),String> {
    let mut pending=vec![kind];
    while let Some(kind)=pending.pop() {match kind {
        TypeRef::Named(name) if matches!(name.as_str(),"바른수"|"셈수"|"나눔수"|"참거짓"|"글")=>{},
        TypeRef::Named(name) if active.contains(&(module,name.as_str()))&&schemas.iter().any(|row|row.module==module&&row.label==*name)=>{},
        TypeRef::Tuple(items)=>pending.extend(items),
        TypeRef::Applied{name,args} if matches!(name.as_str(),"차림"|"모음"|"짝맞춤")=>pending.extend(args),
        _=>return Err(fail("고름씨 받는자리의 기존 닫힌 씨 공급을 확인할 수 없습니다")),
    }}
    Ok(())
}

impl SumRoleCandidates {
    fn issue(source:&Source,nominals:&super::RegistrationDraft,
        entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        let mut reserved=Self::reserve(source,nominals,entropy)?;
        reserved.bind_types(source,nominals)?;
        Ok(reserved)
    }
    fn reserve(source:&Source,nominals:&super::RegistrationDraft,
        entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        let planned=plan(source,nominals)?;
        // Sharing collision rejection never shares typed constructors.
        let mut used=reserved(source,nominals)?;
        let mut templates=Vec::new();
        for row in planned {
            let id=DefinitionId(fresh(&mut used,entropy)?);let mut branches=Vec::new();
            for branch in row.branches {
                let id=BranchId(fresh(&mut used,entropy)?);let mut pins=Vec::new();
                for spelling in branch.pins {pins.push(Pin{id:PinId(fresh(&mut used,entropy)?),spelling,type_bytes:Vec::new()});}
                pins.sort_by_key(|pin|pin.id);branches.push(Branch{id,spelling:branch.spelling,pins});
            }
            branches.sort_by_key(|branch|branch.id);
            templates.push(Template{id,site:row.site,lexical_scope:row.lexical_scope,spelling:row.spelling,branches});
        }
        templates.sort_by_key(|row|row.id);
        Ok(Self{scope:source.scope().into(),templates})
    }
    fn bind_types(&mut self,source:&Source,nominals:&super::RegistrationDraft)->Result<(),String> {
        let proofs=typeproof::StaticTypeOwner::from_source(source,nominals,&self.templates)?.pin_proofs(source)?;
        let mut type_bytes_total=0;
        for template in &mut self.templates {for branch in &mut template.branches {for pin in &mut branch.pins {
            let proof=proofs.get(&(template.site.clone(),branch.spelling.clone(),pin.spelling.clone()))
                .ok_or_else(||fail("실제 원문의 받는자리 형식 증명이 없습니다"))?;
            pin.type_bytes=proof.encode()?;
            charge_role_bytes(&mut type_bytes_total,pin.type_bytes.len().checked_mul(2).ok_or_else(||fail("형식 증명 크기 한도"))?)?;
        }}}
        Ok(())
    }
    /// Correspondence only; no ID minting or permission to deserialize a
    /// catalog. Actual recorded-catalog parser/PESI binding follows this stage.
    fn verify(&self,source:&Source,nominals:&super::RegistrationDraft)->Result<(),String> {
        self.verify_structure(source,nominals)?;
        let proofs=typeproof::StaticTypeOwner::from_source(source,nominals,&self.templates)?.pin_proofs(source)?;
        for template in &self.templates {for branch in &template.branches {for pin in &branch.pins {
            proofs.get(&(template.site.clone(),branch.spelling.clone(),pin.spelling.clone()))
                .ok_or_else(||fail("받는자리의 실제 형식 증명이 없습니다"))?.verify_recorded(&pin.type_bytes)?;
        }}}
        Ok(())
    }
    fn verify_structure(&self,source:&Source,nominals:&super::RegistrationDraft)->Result<(),String> {
        if self.scope!=source.scope() {return Err(fail("다른 실행 소유자의 고름씨 정체입니다"));}
        let planned=plan(source,nominals)?;
        if planned.len()!=self.templates.len() {return Err(fail("고름씨 선언 집합이 다릅니다"));}
        let mut used=reserved(source,nominals)?;let mut sites=BTreeSet::new();
        for template in &self.templates {
            if template.id.0==[0;32]||!used.insert(template.id.0) {return Err(fail("고름씨 정체가 중복됩니다"));}
            if !sites.insert((&template.site,template.lexical_scope,&template.spelling)) {return Err(fail("고름씨 원문 대응이 중복됩니다"));}
            let row=planned.iter().find(|row|row.site==template.site&&row.lexical_scope==template.lexical_scope&&row.spelling==template.spelling)
                .ok_or_else(||fail("고름씨 원문·모듈·봉인이 다릅니다"))?;
            if row.branches.len()!=template.branches.len() {return Err(fail("갈래 집합이 다릅니다"));}
            let mut branch_names=BTreeSet::new();
            for branch in &template.branches {
                if branch.id.0==[0;32]||!used.insert(branch.id.0) {return Err(fail("갈래 정체가 중복됩니다"));}
                if !branch_names.insert(&branch.spelling) {return Err(fail("갈래 대응이 중복됩니다"));}
                let expected=row.branches.iter().find(|old|old.spelling==branch.spelling).ok_or_else(||fail("다른 소유자의 갈래입니다"))?;
                if expected.pins.len()!=branch.pins.len() {return Err(fail("갈래 받는자리 집합이 다릅니다"));}
                let mut pin_names=BTreeSet::new();
                for pin in &branch.pins {
                    if pin.id.0==[0;32]||!used.insert(pin.id.0)||!pin_names.insert(&pin.spelling) {return Err(fail("받는자리 정체·이름이 중복됩니다"));}
                    if !expected.pins.contains(&pin.spelling) {return Err(fail("받는자리 소유자가 다릅니다"));}
                }
            }
        }
        Ok(())
    }
}

/// Actual declaration context, not a guessed privilege. R3.1 maps ordinary
/// named Umjikssi to callable-context, never to root/PURE/direct-write.
#[derive(Clone,Debug,PartialEq,Eq)]
struct DeclarationContext {site:Site,name:Option<String>,kind:Option<ddonirang_lang::SeedKind>}
#[derive(Debug)]
struct ContextCandidate {id:DefinitionId,declaration:std::sync::Arc<DeclarationContext>}
#[derive(Debug)]
struct ScopeCandidate {id:ScopeId,site:Site,definition:DefinitionId,binders:Vec<BinderCandidate>}
#[derive(Debug)]
struct BinderCandidate {id:BinderId,site:Site,spelling:String,selected_pin:PinId,template:DefinitionId,branch:BranchId,owner_raw:std::sync::Arc<[u8]>}
#[derive(Debug)]
struct UserPatternCandidates {contexts:Vec<ContextCandidate>,scopes:Vec<ScopeCandidate>}
struct PlannedPattern {site:Site,context:std::sync::Arc<DeclarationContext>,binders:Vec<PlannedBinder>}
struct PlannedBinder {site:Site,spelling:String,pin:PinId,template:DefinitionId,branch:BranchId,owner_raw:std::sync::Arc<[u8]>}

fn original_site(source:&Source,span:Span)->Result<Site,String> {
    let (path,span)=source.original_span(span)?;
    let module=module_raw(source,&path)?.ok_or_else(||fail("해소 원문의 실제 모듈이 없습니다"))?;
    let capture=&source.captures().iter().find(|(found,_)|found==&path)
        .ok_or_else(||fail("해소 원문의 실제 봉인이 없습니다"))?.1;
    if span.start>span.end||span.end>capture.bytes().len() {return Err(fail("해소 원문의 범위가 다릅니다"));}
    Ok(Site{module,capture:*capture.identity(),start:span.start as u64,end:span.end as u64})
}

fn context_for(source:&Source,seed:Option<&SeedDef>,span:Span,
    cache:&mut BTreeMap<Site,std::sync::Arc<DeclarationContext>>)->Result<std::sync::Arc<DeclarationContext>,String> {
    if let Some(seed)=seed {
        let relocated=seed.body.as_ref().is_some_and(|body|seed.span==body.span&&body.stmts.iter().all(|stmt|seed.is_relocated_top_level_setup(stmt)));
        if !relocated {
            let site=original_site(source,seed.span)?;
            if let Some(context)=cache.get(&site) {return Ok(context.clone());}
            // Retain the actual source declaration header, not the linked
            // spelling used only by the compiler's import lookup.
            let capture=source.captures().iter().find(|(_,capture)|capture.identity()==&site.capture)
                .ok_or_else(||fail("선언 문맥의 원 봉인이 없습니다"))?;
            let original=ddonirang_lang::parse_v25_with_mode(capture.1.text(),&capture.0,ddonirang_lang::ParseMode::Strict)
                .map_err(|error|fail(&error.to_string()))?;
            let mut declarations=original.items.iter().filter_map(|TopLevelItem::SeedDef(seed)|
                (seed.span.start as u64==site.start&&seed.span.end as u64==site.end).then_some(seed));
            let declaration=declarations.next().ok_or_else(||fail("원문 소유자의 실제 정의를 찾을 수 없습니다"))?;
            if declarations.next().is_some() {return Err(fail("원문 소유자 정의가 중복됩니다"));}
            let context=std::sync::Arc::new(DeclarationContext{site:site.clone(),name:Some(declaration.canonical_name.clone()),kind:Some(declaration.seed_kind.clone())});
            cache.insert(site,context.clone());return Ok(context);
        }
    }
    let mut site=original_site(source,span)?;
    let capture=source.captures().iter().find(|(_,capture)|capture.identity()==&site.capture)
        .ok_or_else(||fail("최상위 문맥의 원 봉인이 없습니다"))?;
    site.start=0;site.end=capture.1.bytes().len() as u64;
    if let Some(context)=cache.get(&site) {return Ok(context.clone());}
    let context=std::sync::Arc::new(DeclarationContext{site:site.clone(),name:None,kind:None});
    cache.insert(site,context.clone());Ok(context)
}

fn plan_user_patterns(source:&Source,nominals:&super::RegistrationDraft,templates:&SumRoleCandidates,typed:bool)->Result<Vec<PlannedPattern>,String> {
    let preparation=source.preparation();
    let program=ddonirang_lang::parse_v25_with_mode(&preparation.source,&preparation.target_path,ddonirang_lang::ParseMode::Strict)
        .map_err(|error|fail(&error.to_string()))?;
    // Names select declarations in the actual linked resolver output. IDs
    // still come from matching the declaration's module/capture/source site.
    let mut owners=BTreeMap::new();
    let (declarations,matches)=scoped_sum_syntax(&program);
    for (stmt,_,_) in declarations {
        let Stmt::SumDefinition{name,span,..}=stmt else {unreachable!()};
        let site=original_site(source,*span)?;
        let mut matching=templates.templates.iter().filter(|row|row.site==site);
        let template=matching.next().ok_or_else(||fail("해소된 고름씨의 실제 등록 후보가 없습니다"))?;
        if matching.next().is_some()||owners.insert(name.as_str(),template).is_some() {return Err(fail("해소된 고름씨 소유자가 중복됩니다"));}
    }
    let type_owner=typeproof::StaticTypeOwner::from_source(source,nominals,&templates.templates)?;
    let mut result=Vec::new();let mut contexts=BTreeMap::new();let mut role_bytes=0;
    for (stmt,seed) in matches {
        let Stmt::SumMatch{arms,..}=stmt else {unreachable!()};
        for arm in arms {
            let (template_id,branch_id,pin_ids,owner_raw)=if let Some(kind)=&arm.applied_owner {
                let applied=ddonirang_lang::sum_owner::resolve_applied_sum_owner(kind,arm.span)?;
                let owner_raw=if typed {type_owner.resolve(&applied.type_ref,arm.span)?.sum_owner_raw()?}else{std::sync::Arc::<[u8]>::from(&[][..])};
                let branch=applied.branches.iter().find(|branch|branch.name==arm.branch).ok_or_else(||fail("적용 맺음의 실제 갈래가 없습니다"))?;
                let fixed=builtin_result_ids()?;
                let (branch_id,pin_id)=match branch.name.as_str(){"잘됨"=>(fixed[1],fixed[3]),"안됨"=>(fixed[2],fixed[4]),_=>return Err(fail("고정 맺음 manifest와 실제 resolver가 다릅니다"))};
                let pins=branch.payload.iter().map(|pin|(pin.binder.clone(),PinId(pin_id))).collect::<BTreeMap<_,_>>();
                (DefinitionId(fixed[0]),BranchId(branch_id),pins,owner_raw)
            } else if let Some(template)=owners.get(arm.owner.as_str()) {
                let branch=template.branches.iter().find(|branch|branch.spelling==arm.branch).ok_or_else(||fail("다른 소유자의 갈래입니다"))?;
                let owner_raw=if typed {type_owner.resolve(&TypeRef::Named(arm.owner.clone()),arm.span)?.sum_owner_raw()?}else{std::sync::Arc::<[u8]>::from(&[][..])};
                (template.id,branch.id,branch.pins.iter().map(|pin|(pin.spelling.clone(),pin.id)).collect(),owner_raw)
            } else {
                let builtin=ddonirang_lang::sum_owner::resolve_named_builtin_sum_owner(&arm.owner).ok_or_else(||fail("pattern 소유자의 실제 등록 후보가 없습니다"))?;
                let branch=builtin.branches.iter().find(|branch|branch.name==arm.branch).ok_or_else(||fail("가리킴안됨의 실제 갈래가 없습니다"))?;
                if !branch.payload.is_empty()||!arm.binder.is_empty(){return Err(fail("가리킴안됨 갈래에는 받는자리가 없습니다"));}
                let fixed=builtin_reference_failure_ids()?;
                let branch_id=match branch.name.as_str(){"끊김"=>fixed[1],"세대다름"=>fixed[2],"누리다름"=>fixed[3],"씨다름"=>fixed[4],_=>return Err(fail("가리킴안됨 manifest와 실제 resolver가 다릅니다"))};
                let owner_raw=if typed {type_owner.resolve(&builtin.type_ref,arm.span)?.sum_owner_raw()?}else{std::sync::Arc::<[u8]>::from(&[][..])};
                (DefinitionId(fixed[0]),BranchId(branch_id),BTreeMap::new(),owner_raw)
            };
            let site=original_site(source,arm.span)?;let context=context_for(source,seed,arm.span,&mut contexts)?;
            charge_role_bytes(&mut role_bytes,64)?;
            let mut binders=Vec::new();let mut names=BTreeSet::new();
            for binder in &arm.binder {
                if !names.insert(binder.pin.as_str()) {return Err(fail("같은 pattern의 받는 이름이 중복됩니다"));}
                let pin=*pin_ids.get(&binder.pin).ok_or_else(||fail("갈래에 없는 받는자리입니다"))?;
                charge_role_bytes(&mut role_bytes,64usize.checked_add(binder.pin.len()).ok_or_else(||fail("받는 이름 크기 한도"))?)?;
                let binder_site=original_site(source,binder.span)?;
                if binder_site.module!=site.module||binder_site.capture!=site.capture||binder_site.start<site.start||binder_site.end>site.end {
                    return Err(fail("받는 이름이 실제 arm의 원문 밖에 있습니다"));
                }
                binders.push(PlannedBinder{site:binder_site,spelling:binder.pin.clone(),pin,template:template_id,branch:branch_id,owner_raw:owner_raw.clone()});
            }
            result.push(PlannedPattern{site,context,binders});
        }
    }
    Ok(result)
}

impl UserPatternCandidates {
    /// Checked compiler projection from this exact source's selected pins.
    /// It is not a persisted catalog or a runtime admission. NodeId is only
    /// a newly rebuilt lookup cache; no stored identity is derived from it.
    fn checked_canon(&self,source:&Source,nominals:&super::RegistrationDraft,templates:&SumRoleCandidates)->Result<String,String> {
        self.verify(source,nominals,templates)?;
        let preparation=source.preparation();
        let mut program=ddonirang_lang::parse_v25_with_mode(&preparation.source,&preparation.target_path,ddonirang_lang::ParseMode::Strict)
            .map_err(|error|fail(&error.to_string()))?;
        let order=self.pin_order(source,&program)?;
        program.origin.registered_sum_pin_order=Some(order.clone());
        ddonirang_lang::canonicalize(&mut program).map_err(|error|fail(&error.to_string()))?;
        ddonirang_lang::normalizer::normalize_with_registered_sum_pin_order(
            &program,ddonirang_lang::NormalizationLevel::N1,&order)
    }
    fn pin_order(&self,source:&Source,program:&CanonProgram)->Result<BTreeMap<ddonirang_lang::NodeId,[u8;32]>,String> {
        if program.origin.source!=source.preparation().source {return Err(fail("등록 정본화와 실제 실행 원문이 다릅니다"));}
        let scopes=self.scopes.iter().map(|scope|(&scope.site,scope)).collect::<BTreeMap<_,_>>();
        let mut order=BTreeMap::new();
        for (stmt,_) in scoped_sum_syntax(&program).1 {
            let Stmt::SumMatch{arms,..}=stmt else {unreachable!()};
            for arm in arms {
                let site=original_site(source,arm.span)?;
                let scope=scopes.get(&site).ok_or_else(||fail("정본화할 실제 arm 범위가 없습니다"))?;
                for binder in &arm.binder {
                    let site=original_site(source,binder.span)?;
                    let selected=scope.binders.iter().find(|row|row.site==site&&row.spelling==binder.pin)
                        .ok_or_else(||fail("정본화할 받는자리 원문 대응이 없습니다"))?;
                    if order.insert(binder.id,selected.selected_pin.0).is_some() {return Err(fail("정본화 조회 키가 중복됩니다"));}
                }
            }
        }
        Ok(order)
    }
    fn issue(source:&Source,nominals:&super::RegistrationDraft,templates:&SumRoleCandidates,
        entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        Self::issue_with_contexts(source,nominals,templates,None,&[],entropy)
    }
    fn issue_with_contexts(source:&Source,nominals:&super::RegistrationDraft,templates:&SumRoleCandidates,
        provided:Option<&BTreeMap<Site,DefinitionId>>,extra_reserved:&[[u8;32]],entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        templates.verify_structure(source,nominals)?;
        let planned=plan_user_patterns(source,nominals,templates,false)?;
        let mut used=reserved(source,nominals)?;
        for template in &templates.templates {
            used.insert(template.id.0);
            for branch in &template.branches {used.insert(branch.id.0);used.extend(branch.pins.iter().map(|pin|pin.id.0));}
        }
        if let Some(provided)=provided {for id in provided.values() {if id.0==[0;32]||!used.insert(id.0){return Err(fail("예약된 문맥 정체가 충돌합니다"));}}}
        for id in extra_reserved {if *id==[0;32]||!used.insert(*id){return Err(fail("예약된 받는자리 정체가 충돌합니다"));}}
        let mut contexts=Vec::<ContextCandidate>::new();let mut context_ids=BTreeMap::new();let mut scopes=Vec::new();
        for row in planned {
            let definition=if let Some(id)=context_ids.get(&row.context.site) {*id}
                else {let id=match provided {Some(ids)=>*ids.get(&row.context.site).ok_or_else(||fail("실제 문맥의 예약 정체가 없습니다"))?,None=>DefinitionId(fresh(&mut used,entropy)?)};context_ids.insert(row.context.site.clone(),id);contexts.push(ContextCandidate{id,declaration:row.context});id};
            let id=ScopeId(fresh(&mut used,entropy)?);let mut binders=Vec::new();
            for binder in row.binders {binders.push(BinderCandidate{id:BinderId(fresh(&mut used,entropy)?),site:binder.site,
                spelling:binder.spelling,selected_pin:binder.pin,template:binder.template,branch:binder.branch,owner_raw:binder.owner_raw});}
            binders.sort_by_key(|binder|binder.id);
            scopes.push(ScopeCandidate{id,site:row.site,definition,binders});
        }
        contexts.sort_by_key(|context|context.id);scopes.sort_by_key(|scope|scope.id);
        let typed=plan_user_patterns(source,nominals,templates,true)?;
        for scope in &mut scopes {
            let plan=typed.iter().find(|row|row.site==scope.site).ok_or_else(||fail("예약 범위의 실제 형식 소유자가 없습니다"))?;
            for binder in &mut scope.binders {
                binder.owner_raw=plan.binders.iter().find(|row|row.site==binder.site&&row.pin==binder.selected_pin).ok_or_else(||fail("예약 받는자리의 닫힌 소유자 형식이 없습니다"))?.owner_raw.clone();
            }
        }
        Ok(Self{contexts,scopes})
    }
    fn verify(&self,source:&Source,nominals:&super::RegistrationDraft,templates:&SumRoleCandidates)->Result<(),String> {
        templates.verify(source,nominals)?;
        let planned=plan_user_patterns(source,nominals,templates,true)?;
        if planned.len()!=self.scopes.len() {return Err(fail("pattern 범위 집합이 다릅니다"));}
        let mut used=reserved(source,nominals)?;let mut scope_sites=BTreeSet::new();let mut referenced=BTreeSet::new();
        for template in &templates.templates {used.insert(template.id.0);for branch in &template.branches {used.insert(branch.id.0);used.extend(branch.pins.iter().map(|pin|pin.id.0));}}
        let mut context_sites=BTreeSet::new();
        for context in &self.contexts {if context.id.0==[0;32]||!used.insert(context.id.0)||!context_sites.insert(&context.declaration.site) {return Err(fail("선언 문맥 정체·대응이 중복됩니다"));}}
        for scope in &self.scopes {
            if scope.id.0==[0;32]||!used.insert(scope.id.0)||!scope_sites.insert(&scope.site) {return Err(fail("pattern 범위 정체·원문이 중복됩니다"));}
            let row=planned.iter().find(|row|row.site==scope.site).ok_or_else(||fail("외래 pattern 원문입니다"))?;
            let context=self.contexts.iter().find(|context|context.id==scope.definition).ok_or_else(||fail("pattern 소유 정의가 없습니다"))?;
            if context.declaration!=row.context||scope.binders.len()!=row.binders.len() {return Err(fail("pattern 소유자·받는 이름 집합이 다릅니다"));}
            referenced.insert(context.id);let mut names=BTreeSet::new();
            for binder in &scope.binders {
                if binder.id.0==[0;32]||!used.insert(binder.id.0)||!names.insert(&binder.spelling) {return Err(fail("지역 받는 이름 정체·대응이 중복됩니다"));}
                if !row.binders.iter().any(|expected|expected.site==binder.site&&expected.spelling==binder.spelling
                    &&expected.pin==binder.selected_pin&&expected.template==binder.template&&expected.branch==binder.branch&&expected.owner_raw==binder.owner_raw) {
                    return Err(fail("지역 받는 이름의 실제 소유 받는자리와 다릅니다"));
                }
            }
        }
        if referenced.len()!=self.contexts.len() {return Err(fail("해소되지 않은 여분의 문맥 정체가 있습니다"));}
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r3_3_optional_proof_uses_closed_source_child_and_rejects_other_framing() {
        let source=make_source_with_body("선택:고름씨 := { $값(내용:(셈수)있없). }. 실행:움직씨 := {}.");
        let nominal=make_nominal(&source);
        let issued=SumRoleCandidates::issue(&source,&nominal,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        let proofs=typeproof::StaticTypeOwner::from_source(&source,&nominal,&issued.templates).unwrap().pin_proofs(&source).unwrap();
        let proof=proofs.values().next().unwrap();let bytes=proof.encode().unwrap();let p=typeproof::DOMAIN.len();
        assert_eq!(bytes.len(),61);assert_eq!(&bytes[p..p+2],&[3,4]);
        assert_eq!(&bytes[p+2..p+10],&1u64.to_le_bytes());
        assert_eq!(&bytes[p+10..p+18],&24u64.to_le_bytes());
        assert_eq!(bytes[p+18],0);
        for (offset,value) in [(p+1,5),(p+2,0),(p+2,2),(p+10,23)] {
            let mut changed=bytes.clone();changed[offset]=value;assert!(proof.verify_recorded(&changed).is_err());
        }
        let mut trailer=bytes.clone();trailer.push(0);assert!(proof.verify_recorded(&trailer).is_err());
        assert!(crate::unit_wire::decode_recorded_field_type(&bytes).is_err(),"type proof is not a new saved value codec");
    }
    #[test]
    fn r3_2_fixed_builtin_manifest_preserves_prefix_and_reserves_all_ten_roles() {
        use sha2::{Digest,Sha256};
        let bytes=builtin_manifest_bytes().unwrap();assert_eq!(bytes.len(),320);
        assert_eq!(&bytes[..160],builtin_result_ids().unwrap().into_iter().flatten().collect::<Vec<_>>());
        assert_eq!(hex::encode(Sha256::digest(&bytes)),"4ac1760e171b874b2e96352f897b227900881d4d06ffc90ef1b47d13f6aa7f1d");
        let ids=bytes.chunks_exact(32).collect::<BTreeSet<_>>();assert_eq!(ids.len(),10);assert!(ids.iter().all(|id|**id!=[0;32]));
    }
    #[test]
    fn r3_2_user_reference_failure_spelling_does_not_acquire_builtin_identity() {
        let source=make_source_with_body("가리킴안됨:고름씨 := { $끊김. }. 선택:고름씨 := { $값(내용:가리킴안됨). }. 실행:움직씨 := {}.");
        let nominal=make_nominal(&source);
        let roles=SumRoleCandidates::issue(&source,&nominal,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        let actual=roles.templates.iter().find(|row|row.spelling=="가리킴안됨").unwrap().id.0;
        assert_ne!(actual,builtin_reference_failure_ids().unwrap()[0]);
        let selected=roles.templates.iter().find(|row|row.spelling=="선택").unwrap();
        let bytes=&selected.branches[0].pins[0].type_bytes;let prefix=typeproof::DOMAIN.len();
        assert_eq!(&bytes[prefix+10..prefix+42],actual.as_slice());
    }
    fn make_source()->Source {
        make_source_with_body("짝:고름씨 := { $있음(왼쪽:바른수, 오른쪽:글). $없음. }. 실행:움직씨 := { }.")
    }
    fn make_source_with_body(main:&str)->Source {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.";
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),
            files:vec![SourceFile{path:"root.ddn".into(),text:root.into()},SourceFile{path:"main.ddn".into(),text:main.into()}]},String::new()).unwrap();
        Source::saved(&crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(project.save().unwrap().as_bytes()).unwrap())
    }
    fn make_nominal(source:&Source)->super::super::RegistrationDraft {
        super::super::prepare_from_source_view(&super::super::SavedRegistrationSource::precanon(source.saved_owner()),None,&mut |raw|{
            getrandom::fill(raw).map_err(|_|fail("등록 난수 실패"))
        }).unwrap()
    }
    #[test]
    fn r3_actual_precanon_sum_roles_have_distinct_random_owners_and_exact_source_types() {
        let source=make_source();let nominal=make_nominal(&source);
        let issued=SumRoleCandidates::issue(&source,&nominal,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        issued.verify(&source,&nominal).unwrap();
        assert_eq!(issued.templates.len(),1);assert_eq!(issued.templates[0].branches.len(),2);
        let branch=issued.templates[0].branches.iter().find(|row|row.spelling=="있음").unwrap();
        assert_eq!(branch.pins.len(),2);
        for pin in &branch.pins {assert!(pin.type_bytes.starts_with(typeproof::DOMAIN));}
        let other=SumRoleCandidates::issue(&source,&nominal,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        assert_ne!(issued.templates[0].id,other.templates[0].id);
        let foreign=make_source();let foreign_nominal=make_nominal(&foreign);
        assert!(issued.verify(&foreign,&foreign_nominal).is_err());
    }
    #[test]
    fn r3_1_actual_source_sum_references_nested_types_and_default_result_keep_closed_proofs() {
        let source=make_source_with_body("안쪽:고름씨 := { $값(내용:바른수). }. 큰선택:고름씨 := { $값(대상:안쪽, 묶음:(안쪽, (글) 차림), 결과:(안쪽) 맺음, 별칭:수). }. 실행:움직씨 := {}.");
        let nominal=make_nominal(&source);
        let mut issued=SumRoleCandidates::issue(&source,&nominal,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        issued.verify(&source,&nominal).unwrap();
        let inner=issued.templates.iter().find(|row|row.spelling=="안쪽").unwrap().id.0;
        let outer=issued.templates.iter().find(|row|row.spelling=="큰선택").unwrap();
        let pins=&outer.branches[0].pins;
        let target=&pins.iter().find(|pin|pin.spelling=="대상").unwrap().type_bytes;
        let prefix=typeproof::DOMAIN.len();
        assert_eq!(target.len(),prefix+1+8+41);
        assert_eq!(target[prefix],1);assert_eq!(target[prefix+9],0x21);
        assert_eq!(&target[prefix+10..prefix+42],inner.as_slice());
        assert_eq!(&target[prefix+42..],0u64.to_le_bytes());
        let tuple=&pins.iter().find(|pin|pin.spelling=="묶음").unwrap().type_bytes;
        assert_eq!(tuple[prefix],2);
        let result=&pins.iter().find(|pin|pin.spelling=="결과").unwrap().type_bytes;
        assert_eq!(result[prefix],1);
        assert_eq!(&result[prefix+42..prefix+50],2u64.to_le_bytes());
        let expected=typeproof::StaticTypeOwner::from_source(&source,&nominal,&issued.templates).unwrap().pin_proofs(&source).unwrap();
        assert!(expected.values().any(|proof|proof.verify_recorded(target).is_ok()));
        // A bare template ID is not the 41-byte owner, nor a type proof.
        assert!(expected.values().all(|proof|proof.verify_recorded(&inner).is_err()));
        let branch=issued.templates.iter_mut().find(|row|row.spelling=="큰선택").unwrap().branches.first_mut().unwrap();
        let pin=branch.pins.iter_mut().find(|pin|pin.spelling=="대상").unwrap();
        pin.type_bytes[prefix+10]^=1;
        assert!(issued.verify(&source,&nominal).is_err(),"recomputed source owner must reject a substituted Sum identity");
    }
    #[test]
    fn r3_1_actual_source_function_and_tensor_type_structure_is_not_saved_value_admission() {
        let source=make_source_with_body("선택:고름씨 := { $값(계산:(바른수) --> 글, 행렬값:(원소씨=셈수, 모양=(2, 3)) 텐서씨). }. 실행:움직씨 := {}.");
        let nominal=make_nominal(&source);
        let issued=SumRoleCandidates::issue(&source,&nominal,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        issued.verify(&source,&nominal).unwrap();
        let pins=&issued.templates[0].branches[0].pins;let prefix=typeproof::DOMAIN.len();
        assert_eq!(pins.iter().find(|pin|pin.spelling=="계산").unwrap().type_bytes[prefix],4);
        assert_eq!(pins.iter().find(|pin|pin.spelling=="행렬값").unwrap().type_bytes[prefix],5);
        for pin in pins {assert!(crate::unit_wire::decode_recorded_field_type(&pin.type_bytes).is_err(),"T1 proof is not a saved value wire");}
    }
    #[test]
    fn r3_sum_candidate_zero_collision_and_foreign_pin_fail_without_redraw() {
        let source=make_source();let nominal=make_nominal(&source);let mut calls=0;
        assert!(SumRoleCandidates::issue(&source,&nominal,&mut |_|{calls+=1;Ok(())}).is_err());assert_eq!(calls,1);
        calls=0;assert!(SumRoleCandidates::issue(&source,&nominal,&mut |raw|{calls+=1;*raw=[7;32];Ok(())}).is_err());assert_eq!(calls,2);
        for fixed in builtin_result_ids().unwrap().into_iter().chain(builtin_reference_failure_ids().unwrap()) {
            calls=0;assert!(SumRoleCandidates::issue(&source,&nominal,&mut |raw|{calls+=1;*raw=fixed;Ok(())}).is_err());
            assert_eq!(calls,1,"fixed builtin roles reject collision without drawing a replacement");
        }
        let mut issued=SumRoleCandidates::issue(&source,&nominal,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        issued.verify(&source,&nominal).unwrap();
        let branch=issued.templates[0].branches.iter_mut().find(|row|!row.pins.is_empty()).unwrap();
        branch.pins[0].spelling="외래".into();assert!(issued.verify(&source,&nominal).is_err());
    }
    #[test]
    fn r3_nested_sum_declarations_keep_real_lexical_owners_and_preflight_type_failure() {
        let source=make_source_with_body("실행:움직씨 := { 첫선택:고름씨 := { $값(내용:(바른수,글)). }. }. 다음:움직씨 := { 둘선택:고름씨 := { $값(내용:바른수). }. }.");
        let nominal=make_nominal(&source);
        let issued=SumRoleCandidates::issue(&source,&nominal,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        issued.verify(&source,&nominal).unwrap();
        assert_eq!(issued.templates.len(),2);
        assert_ne!(issued.templates[0].lexical_scope,issued.templates[1].lexical_scope);
        assert_ne!(issued.templates[0].id,issued.templates[1].id);
        assert_ne!(issued.templates[0].branches[0].pins[0].type_bytes,issued.templates[1].branches[0].pins[0].type_bytes);
        let invalid=make_source_with_body("선택:고름씨 := { $값(내용:없는씨). }. 실행:움직씨 := {}.");
        let nominal=make_nominal(&invalid);let mut calls=0;
        assert!(SumRoleCandidates::issue(&invalid,&nominal,&mut |raw|{calls+=1;getrandom::fill(raw).map_err(|_|fail("난수 실패"))}).is_err());
        assert!(calls>0,"R3.1 reserves role IDs before resolving closed nominal references; no candidate is published on failure");
    }
    #[test]
    fn r3_user_pattern_scopes_keep_selected_pin_separate_from_local_binder_and_actual_definition() {
        let source=make_source_with_body("선택:고름씨 := { $쌍(가로:바른수,세로:바른수). }. 실행:움직씨 := { 자료 <- 선택.$쌍(세로=4,가로=3). 자료에 따라 { 선택.$쌍(세로,가로)이면 { (가로+세로) 보여주기. }. }. 자료에 따라 { 선택.$쌍(가로)이면 { 가로 보여주기. }. }. }. 다음:움직씨 := { 자료 <- 선택.$쌍(세로=2,가로=1). 자료에 따라 { 선택.$쌍(가로)이면 { 가로 보여주기. }. }. }.");
        let nominal=make_nominal(&source);
        let mut entropy=|raw:&mut [u8;32]|getrandom::fill(raw).map_err(|_|fail("난수 실패"));
        let templates=SumRoleCandidates::issue(&source,&nominal,&mut entropy).unwrap();
        let mut patterns=UserPatternCandidates::issue(&source,&nominal,&templates,&mut entropy).unwrap();
        patterns.verify(&source,&nominal,&templates).unwrap();
        assert_eq!(patterns.contexts.len(),2);assert_eq!(patterns.scopes.len(),3);
        assert!(patterns.contexts.iter().all(|context|context.declaration.kind==Some(ddonirang_lang::SeedKind::Umjikssi)));
        let names=patterns.contexts.iter().map(|context|context.declaration.name.as_deref().unwrap()).collect::<BTreeSet<_>>();
        assert_eq!(names,BTreeSet::from(["다음","실행"]));
        let selected=patterns.scopes.iter().flat_map(|scope|scope.binders.iter()).filter(|binder|binder.spelling=="가로").collect::<Vec<_>>();
        assert_eq!(selected.len(),3);assert_eq!(selected.iter().map(|binder|binder.selected_pin).collect::<BTreeSet<_>>().len(),1);
        assert_eq!(selected.iter().map(|binder|binder.id).collect::<BTreeSet<_>>().len(),3);
        assert!(selected.iter().all(|binder|binder.id.0!=binder.selected_pin.0&&binder.id.0!=binder.template.0&&binder.id.0!=binder.branch.0));
        let binder=&mut patterns.scopes[0].binders[0];let original=binder.selected_pin;
        binder.selected_pin=PinId(binder.id.0);assert!(patterns.verify(&source,&nominal,&templates).is_err());
        patterns.scopes[0].binders[0].selected_pin=original;patterns.verify(&source,&nominal,&templates).unwrap();
    }
    #[test]
    fn r3_pattern_candidate_entropy_failure_preserves_original_template_owner() {
        let source=make_source_with_body("선택:고름씨 := { $값(내용:바른수). }. 실행:움직씨 := { 자료 <- 선택.$값(내용=3). 자료에 따라 { 선택.$값(내용)이면 { 내용 보여주기. }. }. }.");
        let nominal=make_nominal(&source);
        let templates=SumRoleCandidates::issue(&source,&nominal,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        let mut calls=0;
        assert!(UserPatternCandidates::issue(&source,&nominal,&templates,&mut |_|{calls+=1;Err(fail("의도한 난수 공급 실패"))}).is_err());
        assert_eq!(calls,1);templates.verify(&source,&nominal).unwrap();
    }
    #[test]
    fn r3_checked_canon_consumes_source_owned_pin_order_without_changing_construction_order() {
        let source=make_source_with_body("선택:고름씨 := { $쌍(가로:바른수,세로:바른수). }. 실행:움직씨 := { 자료 <- 선택.$쌍(세로=4,가로=3). 자료에 따라 { 선택.$쌍(가로,세로)이면 { (가로+세로) 보여주기. }. }. }.");
        let nominal=make_nominal(&source);let used=reserved(&source,&nominal).unwrap();
        // A deterministic entropy fixture makes raw order oppose spelling.
        // These fixture bytes are not evidence of production CSPRNG issuance.
        let available=(1u8..=255).map(|byte|[byte;32]).filter(|raw|!used.contains(raw)).collect::<Vec<_>>();
        let draws=[available[0],available[1],*available.last().unwrap(),available[2]];let mut at=0;
        let templates=SumRoleCandidates::issue(&source,&nominal,&mut |raw|{*raw=draws[at];at+=1;Ok(())}).unwrap();
        assert_eq!(at,4);
        let patterns=UserPatternCandidates::issue(&source,&nominal,&templates,&mut |raw|getrandom::fill(raw).map_err(|_|fail("난수 실패"))).unwrap();
        let captured=source.saved_owner().sources().recorded_capture_catalog_json().unwrap();
        let canon=patterns.checked_canon(&source,&nominal,&templates).unwrap();
        assert!(canon.contains("선택.$쌍(세로, 가로)이면"),"{canon}");
        assert!(canon.contains("선택.$쌍(세로=4, 가로=3)"),"construction evaluation order must not change: {canon}");
        assert_eq!(patterns.checked_canon(&source,&nominal,&templates).unwrap(),canon);
        assert_eq!(source.saved_owner().sources().recorded_capture_catalog_json().unwrap(),captured);
    }
    #[test]
    fn r3_checked_formatter_missing_duplicate_and_unused_cache_entries_return_no_candidate_string() {
        let source=make_source_with_body("선택:고름씨 := { $쌍(가로:바른수,세로:바른수). }. 실행:움직씨 := { 자료 <- 선택.$쌍(세로=4,가로=3). 자료에 따라 { 선택.$쌍(가로,세로)이면 { 가로 보여주기. }. }. }.");
        let preparation=source.preparation();
        let program=ddonirang_lang::parse_v25_with_mode(&preparation.source,&preparation.target_path,ddonirang_lang::ParseMode::Strict).unwrap();
        let raw=|order:&BTreeMap<_,_>|ddonirang_lang::normalizer::normalize_with_registered_sum_pin_order(&program,ddonirang_lang::NormalizationLevel::N1,order);
        assert!(raw(&BTreeMap::new()).unwrap_err().starts_with("E_SUM_PIN_IDENTITY_REQUIRED"));
        let binders=scoped_sum_syntax(&program).1.into_iter().flat_map(|(stmt,_)|match stmt {Stmt::SumMatch{arms,..}=>arms.iter().flat_map(|arm|arm.binder.iter()).collect::<Vec<_>>(),_=>unreachable!()}).collect::<Vec<_>>();
        let mut order=binders.iter().map(|binder|(binder.id,[1;32])).collect::<BTreeMap<_,_>>();
        assert!(raw(&order).unwrap_err().starts_with("E_SUM_PIN_IDENTITY_DUPLICATE"));
        order.insert(binders[1].id,[2;32]);assert!(raw(&order).is_ok());
        order.insert(u64::MAX,[3;32]);assert!(raw(&order).unwrap_err().starts_with("E_SUM_PIN_IDENTITY_UNUSED"));
    }
}
