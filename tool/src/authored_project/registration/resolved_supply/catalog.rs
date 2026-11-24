//! Source-bound R3.2 catalog. JSON is transport only: readback reconstructs
//! every admitted source correspondence before returning this private owner.
use super::*;
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};

pub(in crate::authored_project::registration) const ROLE:&str="resolved-supply-catalog-r3.3";
pub(in crate::authored_project::registration) const PATH:&str=".ddn/registration/resolved-supply-catalog.r3.3.json";
const SCHEMA:&str="ddn.resolved-supply-catalog.r3.3";
const LIMIT:usize=1_000_000;
fn encode(catalog:&Catalog)->Result<Vec<u8>,String> {
    struct Bounded(Vec<u8>);
    impl std::io::Write for Bounded {
        fn write(&mut self,bytes:&[u8])->std::io::Result<usize> {
            if self.0.len().checked_add(bytes.len()).is_none_or(|length|length>LIMIT) {return Err(std::io::Error::other("catalog limit"));}
            self.0.try_reserve(bytes.len()).map_err(std::io::Error::other)?;
            self.0.extend_from_slice(bytes);Ok(bytes.len())
        }
        fn flush(&mut self)->std::io::Result<()> {Ok(())}
    }
    let mut writer=Bounded(Vec::new());serde_json::to_writer(&mut writer,catalog).map_err(|_|fail("등록 자료 크기·직렬화 한도"))?;Ok(writer.0)
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema:String,mode:String,scope:String,modules:Vec<String>,captures:Vec<Capture>,
    definitions:Vec<Definition>,branches:Vec<BranchRow>,pins:Vec<PinRow>,scopes:Vec<ScopeRow>,binders:Vec<BinderRow>,
    endpoints:Vec<RejectRow>,channels:Vec<RejectRow>,providers:Vec<RejectRow>,input_maps:Vec<RejectRow>,relations:Vec<RejectRow>,
}
/// These producers have not landed in this A vertical. A nonempty transport
/// row is refused, not deserialized into an arbitrary JSON capability.
#[derive(Debug,Serialize)]
enum RejectRow {}
impl<'de> Deserialize<'de> for RejectRow {
    fn deserialize<D:serde::Deserializer<'de>>(_:D)->Result<Self,D::Error> {
        Err(serde::de::Error::custom("E_RESOLVED_SUPPLY: 이 공급의 실제 제품 생산자가 아직 연결되지 않았습니다"))
    }
}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Capture {identity:String,module:String,content_sha256:String,byte_length:u64}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Definition {id:String,module:String,capture:String,start:u64,end:u64,kind:String,spelling:String,canonical:String,result_type:String,owner:String}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct BranchRow {id:String,template:String,spelling:String}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct PinRow {id:String,owner:String,spelling:String,#[serde(rename="type")]type_hex:String,required:bool,default_canon:String}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopeRow {id:String,definition:String,capture:String,start:u64,end:u64}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct BinderRow {id:String,scope:String,selected_pin:String,spelling:String,capture:String,start:u64,end:u64}

#[derive(Debug)]
pub(in crate::authored_project::registration) struct ResolvedCatalog {
    raw:Vec<u8>,
    legacy_ddl_start:bool,templates:SumRoleCandidates,patterns:UserPatternCandidates,
    selected:Option<SelectedPlainCallable>,
    public:Option<PublicCallableSelection>,
}
fn id(text:&str)->Result<[u8;32],String> {
    if text.len()!=64||!text.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {return Err(fail("등록 정체의 정본 표기가 아닙니다"));}
    let raw: [u8;32]=hex::decode(text).map_err(|_|fail("등록 정체 표기 오류"))?.try_into().map_err(|_|fail("등록 정체 길이 오류"))?;
    if raw==[0;32] {return Err(fail("빈 등록 정체입니다"));}Ok(raw)
}
fn raw(text:&str)->Result<Vec<u8>,String> {
    if text.len()>LIMIT*2||text.len()%2!=0||!text.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {return Err(fail("등록 바이트의 정본 표기가 아닙니다"));}
    hex::decode(text).map_err(|_|fail("등록 바이트 표기 오류"))
}
fn row_site(row:&Definition)->Result<Site,String> {Ok(Site{module:id(&row.module)?,capture:id(&row.capture)?,start:row.start,end:row.end})}
fn unique<'a,T>(rows:impl Iterator<Item=&'a T>)->Result<&'a T,String> {
    let mut rows=rows;let first=rows.next().ok_or_else(||fail("등록 자료의 실제 원문 대응이 없습니다"))?;
    if rows.next().is_some(){return Err(fail("등록 자료의 실제 원문 대응이 중복됩니다"));}Ok(first)
}
// Process-local checked source selection, never a wire field or identity issuer.
#[derive(Clone,Debug)]
struct SelectedPlainCallable {
    original:crate::authored_project::ProjectArtifact,
    path:String,
    site:Site,
}
impl SelectedPlainCallable {
    fn from_context(source:&Source,first:&crate::authored_project::FirstCallableAuthoredContext)->Result<Self,String> {
        let Source::Saved(owned)=source else {return Err(fail("선택 함수는 실제 저장 원본 owner가 필요합니다"));};
        if owned.project()!=first.original_artifact() {return Err(fail("선택 함수와 실제 원본 작품·판본이 다릅니다"));}
        let original=first.definition_source_span();
        let capture=source.captures().iter().find(|(path,_)|path==original.source_path)
            .ok_or_else(||fail("선택 함수의 실제 원문 capture가 없습니다"))?;
        if capture.1.bytes()!=original.source_text.as_bytes() {return Err(fail("선택 함수의 원문 capture가 낡았습니다"));}
        let module=module_raw(source,&capture.0)?.ok_or_else(||fail("선택 함수의 실제 소유 모듈이 없습니다"))?;
        if hex::encode(module)!=first.module_identity().as_str() {return Err(fail("선택 함수의 모듈 소유가 다릅니다"));}
        let selected=Self{original:owned.project().clone(),path:capture.0.clone(),site:Site{module,
            capture:*capture.1.identity(),start:original.span.start as u64,end:original.span.end as u64}};
        selected.context(source)?;
        Ok(selected)
    }
    fn context(&self,source:&Source)->Result<std::sync::Arc<DeclarationContext>,String> {
        let Source::Saved(owned)=source else {return Err(fail("선택 함수의 저장 owner가 없습니다"));};
        if owned.project()!=&self.original {return Err(fail("선택 함수의 원본·판본이 변경됐습니다"));}
        let (_,capture)=source.captures().iter().find(|(path,_)|path==&self.path)
            .ok_or_else(||fail("선택 함수의 원문이 누락됐습니다"))?;
        if capture.identity()!=&self.site.capture||module_raw(source,&self.path)?!=Some(self.site.module) {
            return Err(fail("선택 함수의 capture·모듈이 변경됐습니다"));
        }
        let program=source.parse_capture(capture.text(),&self.path)
            .map_err(|e|fail(&e.to_string()))?;
        let seed=unique(program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed)
            .filter(|seed|seed.span.start as u64==self.site.start&&seed.span.end as u64==self.site.end))?;
        // The existing FirstCallableAuthoredContext supplied the complete
        // public declaration; verify the actual raw selected declaration again.
        if seed.seed_kind!=ddonirang_lang::SeedKind::Semssi||seed.body.is_none()||seed.params.len()!=2
            ||seed.params.iter().any(|pin|pin.optional||pin.default_value.is_some()
                ||pin.type_ref!=ddonirang_lang::TypeRef::Named("나눔수".into()))
            ||seed.result_type!=Some(ddonirang_lang::TypeRef::Named("나눔수".into())) {
            return Err(fail("선택한 실제 셈씨·필수 두 핀·타입이 다릅니다"));
        }
        Ok(std::sync::Arc::new(DeclarationContext{site:self.site.clone(),name:Some(seed.canonical_name.clone()),kind:Some(seed.seed_kind.clone())}))
    }
}

/// Explicit process-local selection; transport bytes alone never select it.
#[derive(Clone,Debug)]
pub(in crate::authored_project::registration) struct PublicCallableSelection {
    original:crate::authored_project::ProjectArtifact,
    module:String,
}
impl PublicCallableSelection {
    fn new(source:&Source,module:&str)->Result<Self,String> {
        let Source::Saved(owned)=source else {return Err(fail("공개 함수의 저장 owner가 없습니다"));};
        let selected=Self{original:owned.project().clone(),module:module.into()};
        selected.contexts(source)?;
        Ok(selected)
    }
    fn contexts(&self,source:&Source)->Result<BTreeMap<Site,std::sync::Arc<DeclarationContext>>,String> {
        let Source::Saved(owned)=source else {return Err(fail("공개 함수의 저장 owner가 없습니다"));};
        if owned.project()!=&self.original {return Err(fail("공개 함수의 원본·판본이 변경됐습니다"));}
        let facts=source.public_callable_facts(&self.module)?;
        let mut rows=BTreeMap::new();
        for declaration in facts.declarations() {
            let crate::authored_project::public_surface::DeclarationNode::Callable(seed)=declaration.node() else {
                return Err("E_RESOLVED_PUBLIC_DECLARATION_ADAPTER_REQUIRED: 공개 비함수 선언의 owner 연결이 필요합니다".into());
            };
            if !matches!(seed.seed_kind,ddonirang_lang::SeedKind::Umjikssi|ddonirang_lang::SeedKind::Semssi|ddonirang_lang::SeedKind::ValueFunc)
                ||seed.body.is_none() {return Err("E_RESOLVED_CONTEXT_EFFECT_ADAPTER_REQUIRED: 이 공개 선언의 기존 callable owner가 없습니다".into());}
            let (_,capture)=source.captures().iter().find(|(path,_)|path==declaration.source_path())
                .ok_or_else(||fail("공개 선언의 실제 capture가 없습니다"))?;
            let start=declaration.declaration_offset().checked_add(seed.span.start).ok_or_else(||fail("공개 선언 위치 한도"))?;
            let end=declaration.declaration_offset().checked_add(seed.span.end).ok_or_else(||fail("공개 선언 위치 한도"))?;
            if capture.text().get(start..end).is_none(){return Err(fail("공개 선언 원문 범위 오류"));}
            let site=Site{module:id(facts.module().as_str())?,capture:*capture.identity(),start:start as u64,end:end as u64};
            let context=std::sync::Arc::new(DeclarationContext{site:site.clone(),name:Some(declaration.internal_name().into()),kind:Some(seed.seed_kind.clone())});
            if rows.insert(site,context).is_some(){return Err(fail("공개 선언 위치가 중복됩니다"));}
        }
        if rows.is_empty(){return Err(fail("선택한 공개 함수가 없습니다"));}
        Ok(rows)
    }
}

fn contexts(source:&Source,selected:Option<&SelectedPlainCallable>,public:Option<&PublicCallableSelection>)->Result<BTreeMap<Site,std::sync::Arc<DeclarationContext>>,String> {
    let preparation=source.preparation();
    let program=source.parse_prepared()
        .map_err(|e|fail(&e.to_string()))?;
    let mut output=BTreeMap::new();
    for (path,capture) in source.captures() {if let Some(module)=module_raw(source,path)? {
        let site=Site{module,capture:*capture.identity(),start:0,end:capture.bytes().len() as u64};
        output.insert(site.clone(),std::sync::Arc::new(DeclarationContext{site,name:None,kind:None}));
    }}
    let (declarations,matches)=scoped_sum_syntax(&program);
    for (span,seed) in declarations.into_iter().map(|(stmt,_,seed)|(match stmt {Stmt::SumDefinition{span,..}=>*span,_=>unreachable!()},seed))
        .chain(matches.into_iter().map(|(stmt,seed)|(match stmt {Stmt::SumMatch{span,..}=>*span,_=>unreachable!()},seed))) {
        let context=context_for(source,seed,span,&mut BTreeMap::new())?;
        output.insert(context.site.clone(),context);
    }
    if let Some(public)=public {
        if selected.is_some(){return Err(fail("첫 함수와 공개 함수 선택을 중복할 수 없습니다"));}
        for (site,context) in public.contexts(source)? {
            if let Some(old)=output.get(&site) {
                if old.name!=context.name||old.kind!=context.kind {return Err(fail("공개 함수와 기존 문맥 소유가 다릅니다"));}
            } else {output.insert(site,context);}
        }
    }
    if let Some(selected)=selected {
        let context=selected.context(source)?;
        if output.contains_key(&selected.site) {return Err(fail("선택 함수 문맥이 기존 등록 문맥과 중복됩니다"));}
        output.insert(selected.site.clone(),context);
    }
    Ok(output)
}
fn kind(context:&DeclarationContext)->Result<&'static str,String> {
    match &context.kind {
        None=>Ok("root-context"),
        // R3.1's ordinary callable context records the actual source header;
        // it is not the verified-PURE provider capability used by relations.
        // Semssi declarations must not gain that capability merely by kind.
        Some(ddonirang_lang::SeedKind::Umjikssi|ddonirang_lang::SeedKind::Semssi|ddonirang_lang::SeedKind::ValueFunc)=>Ok("callable-context"),
        _=>Err("E_RESOLVED_CONTEXT_EFFECT_ADAPTER_REQUIRED: 이 선언 문맥의 실제 씨·effect 검증 연결이 필요합니다".into()),
    }
}
fn callable_pin_names(source:&Source,planned:&BTreeMap<Site,std::sync::Arc<DeclarationContext>>)->Result<Vec<(Site,String)>,String> {
    let mut names=Vec::new();
    for (path,capture) in source.captures() {let Some(module)=module_raw(source,path)? else {continue};
        let program=source.parse_capture(capture.text(),path).map_err(|e|fail(&e.to_string()))?;
        for TopLevelItem::SeedDef(seed) in &program.items {
            let site=Site{module,capture:*capture.identity(),start:seed.span.start as u64,end:seed.span.end as u64};
            if planned.get(&site).is_some_and(|context|context.name.is_some()) {for pin in &seed.params {names.push((site.clone(),pin.pin_name.clone()));}}
        }
    }
    Ok(names)
}

impl ResolvedCatalog {
    pub(in crate::authored_project::registration) fn needed(source:&Source)->Result<bool,String> {
        let preparation=source.preparation();
        let program=source.parse_prepared().map_err(|e|fail(&e.to_string()))?;
        let (declarations,matches)=scoped_sum_syntax(&program);Ok(!declarations.is_empty()||!matches.is_empty())
    }
    pub(in crate::authored_project::registration) fn issue(source:&Source,nominals:&super::super::RegistrationDraft,
        entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        Self::issue_selected(source,nominals,None,entropy)
    }
    pub(in crate::authored_project::registration) fn validate_first_callable(source:&Source,first:&crate::authored_project::FirstCallableAuthoredContext)->Result<(),String> {
        SelectedPlainCallable::from_context(source,first).map(|_|())
    }
    pub(in crate::authored_project::registration) fn checked_first_callable_export(&self,source:&Source,
        owner:&super::super::RegistrationDraft,first:&crate::authored_project::FirstCallableAuthoredContext,
    )->Result<super::super::FirstCallableExportFacts,String>{
        let checked=self.recheck(source,owner)?;
        let selected=SelectedPlainCallable::from_context(source,first)?;
        if checked.selected.as_ref().map(|s|&s.site)!=Some(&selected.site){return Err(fail("공개 DDL의 선택 원본과 G1 소유가 다릅니다"));}
        let registry=ddonirang_lang::ddl::FIRST_CALLABLE_JA_REGISTRATION;
        let surface=ddonirang_lang::ddl::bind_first_callable_ja_declaration(first.compiler_context().original_public(),registry.language_profile_identity,registry.registration_revision)
            .map_err(|e|fail(&e.to_string()))?;
        let seed=surface.callable(registry.callable_name).ok_or_else(||fail("실제 공개 함수 분모가 없습니다"))?;
        // The approved product is this closed two-argument exact addition,
        // not every Semssi. Audit the actual common AST/body as well as the
        // rechecked type owner; no world read/write, hidden call or label-only
        // PURE admission is possible, and no relation-provider is issued.
        let body=seed.body.as_ref().ok_or_else(||fail("공개 함수의 실제 본문이 없습니다"))?;
        let expression=match body.stmts.as_slice(){
            [Stmt::Mutate{target,value,deferred:false,..}] if matches!(&target.kind,ddonirang_lang::ExprKind::Var(name) if name==&seed.canonical_name)=>value,
            [Stmt::Return{value,..}]=>value,
            _=>return Err(fail("E_DDL_FIRST_BODY_SCOPE: 승인된 두 인수의 정확 덧셈 본문만 공개합니다")),
        };
        let ddonirang_lang::ExprKind::Infix{left,op,right}=&expression.kind else{return Err(fail("E_DDL_FIRST_BODY_SCOPE: 실제 덧셈 본문이 아닙니다"));};
        if op!="+" || !matches!((&left.kind,&right.kind),(ddonirang_lang::ExprKind::Var(a),ddonirang_lang::ExprKind::Var(b))
            if (a==&seed.params[0].pin_name&&b==&seed.params[1].pin_name)||(a==&seed.params[1].pin_name&&b==&seed.params[0].pin_name)){
            return Err(fail("E_DDL_FIRST_BODY_SCOPE: 실제 두 핀 이외의 의존·연산입니다"));
        }
        // This rebuilds the actual shared static checker, including effects;
        // a Semssi label or a JSON kind alone never creates export admission.
        let catalog:Catalog=serde_json::from_slice(checked.bytes()).map_err(|_|fail("공개 등록 원자료가 다릅니다"))?;
        let def=unique(catalog.definitions.iter().filter(|d|row_site(d).ok().as_ref()==Some(&selected.site)&&d.kind=="callable-context"))?;
        if def.result_type.is_empty(){return Err(fail("결과의 실제 형식 증명이 없습니다"));}
        let mut pins=Vec::new();
        for param in &seed.params {
            let pin=unique(catalog.pins.iter().filter(|p|p.owner==def.id&&p.spelling==param.pin_name))?;
            if !pin.required||!pin.default_canon.is_empty()||pin.type_hex.is_empty(){return Err(fail("공개 필수 핀·형식·기본값이 다릅니다"));}
            pins.push((pin.spelling.clone(),pin.id.clone(),pin.type_hex.clone()));
        }
        if catalog.pins.iter().filter(|p|p.owner==def.id).count()!=2 {return Err(fail("공개 받는자리 분모가 두 개와 다릅니다"));}
        Ok(super::super::FirstCallableExportFacts{module:def.module.clone(),capture:def.capture.clone(),definition:def.id.clone(),
            pins:pins.try_into().map_err(|_|fail("공개 핀 분모"))?,result_type:def.result_type.clone()})
    }
    pub(in crate::authored_project::registration) fn checked_public_callable_export(&self,source:&Source,
        owner:&super::super::RegistrationDraft,module:&str,
        mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    )->Result<super::super::CheckedPublicCallableExport,String> {
        let checked=self.recheck(source,owner)?;
        let public=checked.public.as_ref().ok_or_else(||fail("명시한 전체 공개 함수 owner가 없습니다"))?;
        if public.module!=module {return Err(fail("공개 DDL의 실제 소유 모듈이 다릅니다"));}
        let contexts=public.contexts(source)?;
        let facts=source.public_callable_facts(module)?;
        let catalog:Catalog=serde_json::from_slice(checked.bytes()).map_err(|_|fail("공개 등록 원자료가 다릅니다"))?;
        let mut definitions=Vec::new();
        for declaration in facts.declarations() {
            let (seed,body_canon)=crate::authored_project::public_surface::check_ddl_callable_shape(declaration).map_err(|error| {
                if let Some(output)=diagnostic.as_deref_mut() {
                    if let Some((_,capture))=source.captures().iter().find(|(path,_)|path==declaration.source_path()) {
                        *output=error.source_diagnostic(declaration,capture.text());
                    }
                }
                error.to_string()
            })?;
            let (_,capture)=source.captures().iter().find(|(path,_)|path==declaration.source_path())
                .ok_or_else(||fail("공개 선언의 실제 원문이 없습니다"))?;
            let start=declaration.declaration_offset().checked_add(seed.span.start).ok_or_else(||fail("공개 원문 범위 한도"))?;
            let end=declaration.declaration_offset().checked_add(seed.span.end).ok_or_else(||fail("공개 원문 범위 한도"))?;
            let site=Site{module:id(module)?,capture:*capture.identity(),start:start as u64,end:end as u64};
            if !contexts.contains_key(&site){return Err(fail("공개 원문·capture·등록 문맥이 다릅니다"));}
            let row=unique(catalog.definitions.iter().filter(|row|row.kind=="callable-context"&&row_site(row).ok().as_ref()==Some(&site)))?;
            if row.result_type.is_empty(){return Err(fail("실제 결과 형식 증명이 없습니다"));}
            let mut pins=Vec::new();
            for param in &seed.params {
                let pin=unique(catalog.pins.iter().filter(|pin|pin.owner==row.id&&pin.spelling==param.pin_name))?;
                if !pin.required||!pin.default_canon.is_empty()||pin.type_hex.is_empty()||pin.type_hex!=row.result_type {
                    return Err(fail("실제 정의 소유 핀·필수성·정확 나눔수 형식이 다릅니다"));
                }
                pins.push(super::super::PublicCallablePinExport{name:param.pin_name.clone(),identity:pin.id.clone(),type_hex:pin.type_hex.clone()});
            }
            if catalog.pins.iter().filter(|pin|pin.owner==row.id).count()!=pins.len(){return Err(fail("전체 공개 핀 분모가 다릅니다"));}
            definitions.push(super::super::PublicCallableDefinitionExport{public_name:declaration.external_name().into(),
                internal_name:declaration.internal_name().into(),identity:row.id.clone(),capture:row.capture.clone(),source_path:declaration.source_path().into(),
                start:site.start,end:site.end,result_type:row.result_type.clone(),pins,closed_body_canon:body_canon});
        }
        if definitions.len()!=contexts.len(){return Err(fail("전체 공개 정의 분모가 다릅니다"));}
        definitions.sort_by(|a,b|a.identity.cmp(&b.identity));
        Ok(super::super::CheckedPublicCallableExport{module:module.into(),project_content_sha256:public.original.content_sha256.clone(),
            catalog_sha256:hex::encode(Sha256::digest(checked.bytes())),
            captures:catalog.captures.iter().map(|capture|(capture.module.clone(),capture.identity.clone(),capture.content_sha256.clone(),capture.byte_length)).collect(),definitions})
    }


    pub(in crate::authored_project::registration) fn issue_first_callable(source:&Source,nominals:&super::super::RegistrationDraft,
        first:&crate::authored_project::FirstCallableAuthoredContext,entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        let selected=SelectedPlainCallable::from_context(source,first)?;
        Self::issue_selected(source,nominals,Some(selected),entropy)
    }
    pub(in crate::authored_project::registration) fn select_public_callables(source:&Source,module:&str)->Result<PublicCallableSelection,String> {
        PublicCallableSelection::new(source,module)
    }
    pub(in crate::authored_project::registration) fn issue_public_callables(source:&Source,nominals:&super::super::RegistrationDraft,
        public:PublicCallableSelection,entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        Self::issue_selected_scopes(source,nominals,None,Some(public),entropy)
    }
    fn issue_selected(source:&Source,nominals:&super::super::RegistrationDraft,selected:Option<SelectedPlainCallable>,
        entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        Self::issue_selected_scopes(source,nominals,selected,None,entropy)
    }
    fn issue_selected_scopes(source:&Source,nominals:&super::super::RegistrationDraft,selected:Option<SelectedPlainCallable>,
        public:Option<PublicCallableSelection>,entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        if let Some(public)=&public {public.contexts(source)?;}
        if let Some(selected)=&selected {selected.context(source)?;}
        let mut templates=SumRoleCandidates::reserve(source,nominals,entropy)?;
        let planned=contexts(source,selected.as_ref(),public.as_ref())?;
        let mut used=reserved(source,nominals)?;
        for row in &templates.templates {used.insert(row.id.0);for branch in &row.branches {used.insert(branch.id.0);used.extend(branch.pins.iter().map(|pin|pin.id.0));}}
        let mut context_ids=BTreeMap::new();
        for (site,context) in &planned {
            kind(context)?;
            if !context_ids.contains_key(site) {context_ids.insert(site.clone(),DefinitionId(fresh(&mut used,entropy)?));}
        }
        let mut callable_pins=BTreeMap::new();
        for key in callable_pin_names(source,&planned)? {
            if callable_pins.insert(key,PinId(fresh(&mut used,entropy)?)).is_some(){return Err(fail("callable의 받는자리 선언이 중복됩니다"));}
        }
        let reserved_pins=callable_pins.values().map(|id|id.0).collect::<Vec<_>>();
        let patterns=UserPatternCandidates::issue_with_contexts(source,nominals,&templates,Some(&context_ids),&reserved_pins,entropy)?;
        // Every necessary role is reserved before closed type proof/canon.
        templates.bind_types(source,nominals)?;
        let transport=Self::project(source,nominals,&templates,&patterns,&context_ids,&callable_pins,selected.as_ref(),public.as_ref(),None)?;
        let raw=encode(&transport)?;
        if raw.len()>LIMIT {return Err(fail("등록 자료 크기 한도입니다"));}
        Ok(Self{raw,templates,patterns,selected,public,legacy_ddl_start:false})
    }
    /// Rebind one checked source revision without minting or spelling-derived
    /// identity transfer. Existing IDs are unchanged; current role/type/canon
    /// verification is repeated before the new data can be consumed.
    pub(in crate::authored_project::registration) fn rebind_nominal_revision(
        old_source:&Source,old_nominals:&super::super::RegistrationDraft,
        source:&Source,nominals:&super::super::RegistrationDraft,bytes:&[u8],
    )->Result<Self,String>{
        if old_source.mode()!="nominal-source"||source.mode()!="nominal-source"||old_source.scope()!=source.scope()
            ||old_source.captures().len()!=1||source.captures().len()!=1||old_source.module_ids()?!=source.module_ids()? {
            return Err(fail("명목 원문 판의 소유 경계가 다릅니다"));
        }
        Self::restore(old_source,old_nominals,bytes)?;
        let old=&old_source.captures()[0].1;let current=&source.captures()[0].1;
        let a=old.bytes();let b=current.bytes();let mut prefix=a.iter().zip(b).take_while(|(a,b)|a==b).count();
        while !old.text().is_char_boundary(prefix)||!current.text().is_char_boundary(prefix){prefix-=1;}
        let mut suffix=a[prefix..].iter().rev().zip(b[prefix..].iter().rev()).take_while(|(a,b)|a==b).count();
        while !old.text().is_char_boundary(a.len()-suffix)||!current.text().is_char_boundary(b.len()-suffix){suffix-=1;}
        let old_end=a.len()-suffix;let new_end=b.len()-suffix;
        let position=|offset:u64|->Result<u64,String>{let offset=usize::try_from(offset).map_err(|_|fail("원문 위치 한도"))?;
            if offset<=prefix{return Ok(offset as u64)}
            if offset>=old_end{return Ok((new_end+offset-old_end) as u64)}
            Err(fail("등록 선언·받는자리 내부 편집의 소유 이행을 증명하지 못했습니다. 편집 원문은 보존했습니다"))};
        let mut catalog:Catalog=serde_json::from_slice(bytes).map_err(|_|fail("이전 등록 원자료 오류"))?;
        let old_id=hex::encode(old.identity());let current_id=hex::encode(current.identity());
        for row in &mut catalog.definitions {if row.capture!=old_id{return Err(fail("다른 원문 선언입니다"));}row.start=position(row.start)?;row.end=position(row.end)?;row.capture=current_id.clone();}
        for row in &mut catalog.scopes {if row.capture!=old_id{return Err(fail("다른 원문 범위입니다"));}row.start=position(row.start)?;row.end=position(row.end)?;row.capture=current_id.clone();}
        for row in &mut catalog.binders {if row.capture!=old_id{return Err(fail("다른 원문 받는자리입니다"));}row.start=position(row.start)?;row.end=position(row.end)?;row.capture=current_id.clone();}
        if catalog.captures.len()!=1{return Err(fail("원문 판 개수가 다릅니다"));}
        catalog.captures[0].identity=current_id.clone();catalog.captures[0].content_sha256=hex::encode(Sha256::digest(b));catalog.captures[0].byte_length=b.len() as u64;
        let preparation=source.preparation();let mut program=source.parse_capture(current.text(),&source.captures()[0].0).map_err(|e|fail(&e.to_string()))?;
        let mut order=BTreeMap::new();
        for stmt in sum_syntax(&program).1 {let Stmt::SumMatch{arms,..}=stmt else{unreachable!()};for arm in arms {
            let scope=unique(catalog.scopes.iter().filter(|row|row.capture==current_id&&row.start==arm.span.start as u64&&row.end==arm.span.end as u64))?;
            for binder in &arm.binder {let selected=unique(catalog.binders.iter().filter(|row|row.scope==scope.id&&row.capture==current_id&&row.start==binder.span.start as u64&&row.end==binder.span.end as u64&&row.spelling==binder.pin))?;order.insert(binder.id,id(&selected.selected_pin)?);}
        }}
        ddonirang_lang::canonicalize(&mut program).map_err(|e|fail(&e.to_string()))?;
        for row in &mut catalog.definitions {
            if row.kind=="root-context" {row.canonical=ddonirang_lang::normalizer::normalize_with_registered_sum_pin_order(&program,ddonirang_lang::NormalizationLevel::N1,&order)?;}
            else if row.kind!="sum-template" {
                let seed=unique(program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed).filter(|seed|seed.span.start as u64==row.start&&seed.span.end as u64==row.end))?;
                row.canonical=ddonirang_lang::normalizer::normalize_registered_fragment(&program,ddonirang_lang::normalizer::RegisteredCanonFragment::Definition(seed),&order)?;
            }
        }
        Self::restore(source,nominals,&encode(&catalog)?)
    }
    pub(in crate::authored_project::registration) fn bytes(&self)->&[u8] {&self.raw}
    pub(in crate::authored_project::registration) fn has_legacy_ddl_start(&self)->bool {self.legacy_ddl_start}
    pub(in crate::authored_project::registration) fn checked_canon(&self,source:&Source,nominals:&super::super::RegistrationDraft)->Result<String,String> {
        self.patterns.checked_canon(source,nominals,&self.templates,self.legacy_ddl_start)
    }
    pub(in crate::authored_project::registration) fn bind_program(&self,source:&Source,program:&mut CanonProgram)->Result<(),String> {
        let order=self.patterns.pin_order(source,program)?;
        program.origin.registered_sum_pin_order=Some(order);
        ddonirang_lang::canonicalize(program).map_err(|error|fail(&error.to_string()))?;
        Ok(())
    }
    fn project(source:&Source,nominals:&super::super::RegistrationDraft,templates:&SumRoleCandidates,
        patterns:&UserPatternCandidates,context_ids:&BTreeMap<Site,DefinitionId>,callable_pins:&BTreeMap<(Site,String),PinId>,
        selected:Option<&SelectedPlainCallable>,public:Option<&PublicCallableSelection>,legacy_ddl_root:Option<&str>)->Result<Catalog,String> {
        templates.verify(source,nominals)?;patterns.verify(source,nominals,templates)?;
        let planned=contexts(source,selected,public)?;
        if planned.len()!=context_ids.len(){return Err(fail("원문 문맥 집합이 다릅니다"));}
        let expected_pins=callable_pin_names(source,&planned)?;
        if expected_pins.len()!=callable_pins.len()||expected_pins.iter().any(|key|!callable_pins.contains_key(key)) {return Err(fail("callable 받는자리 집합이 다릅니다"));}
        let callable_sites=planned.iter().filter(|(_,context)|context.name.is_some()).map(|(site,_)|site.clone()).collect();
        let callable_types=typeproof::StaticTypeOwner::from_source(source,nominals,&templates.templates)?.callable_proofs(source,&callable_sites)?;
        let mut catalog=Catalog{schema:SCHEMA.into(),mode:source.mode().into(),scope:templates.scope.clone(),modules:Vec::new(),captures:Vec::new(),
            definitions:Vec::new(),branches:Vec::new(),pins:Vec::new(),scopes:Vec::new(),binders:Vec::new(),
            endpoints:Vec::new(),channels:Vec::new(),providers:Vec::new(),input_maps:Vec::new(),relations:Vec::new()};
        let mut used=reserved(source,nominals)?;let mut canonical_bytes=0;
        for id in context_ids.values() {if !used.insert(id.0)||id.0==[0;32] {return Err(fail("선언 문맥 정체가 충돌합니다"));}}
        for id in callable_pins.values() {if !used.insert(id.0)||id.0==[0;32] {return Err(fail("callable 받는자리 정체가 충돌합니다"));}}
        for row in &templates.templates {if !used.insert(row.id.0){return Err(fail("고름씨와 문맥 정체가 충돌합니다"));}for branch in &row.branches {
            if !used.insert(branch.id.0){return Err(fail("갈래 정체가 충돌합니다"));}for pin in &branch.pins {if !used.insert(pin.id.0){return Err(fail("받는자리 정체가 충돌합니다"));}}
        }}
        for scope in &patterns.scopes {if !used.insert(scope.id.0){return Err(fail("범위 정체가 충돌합니다"));}for binder in &scope.binders {if !used.insert(binder.id.0){return Err(fail("지역 받는 이름 정체가 충돌합니다"));}}}
        for (path,capture) in source.captures() {let Some(module)=module_raw(source,path)? else {continue};
            catalog.modules.push(hex::encode(module));
            catalog.captures.push(Capture{identity:hex::encode(capture.identity()),module:hex::encode(module),content_sha256:hex::encode(Sha256::digest(capture.bytes())),byte_length:capture.bytes().len() as u64});
            let mut program=source.parse_capture(capture.text(),path).map_err(|e|fail(&e.to_string()))?;
            let mut order=BTreeMap::new();let mut binder_definitions=BTreeMap::new();let mut binder_spans=BTreeMap::new();
            for stmt in sum_syntax(&program).1 {let Stmt::SumMatch{arms,..}=stmt else {unreachable!()};for arm in arms {
                let site=Site{module,capture:*capture.identity(),start:arm.span.start as u64,end:arm.span.end as u64};
                let scope=unique(patterns.scopes.iter().filter(|scope|scope.site==site))?;
                for binder in &arm.binder {let selected=unique(scope.binders.iter().filter(|row|row.site.start==binder.span.start as u64&&row.site.end==binder.span.end as u64&&row.spelling==binder.pin))?;order.insert(binder.id,selected.selected_pin.0);binder_definitions.insert(binder.id,scope.definition);binder_spans.insert(binder.id,binder.span);}
            }}
            ddonirang_lang::canonicalize(&mut program).map_err(|e|fail(&e.to_string()))?;
            let root_site=Site{module,capture:*capture.identity(),start:0,end:capture.bytes().len() as u64};
            let root_id=*context_ids.get(&root_site).ok_or_else(||fail("실제 원문 뿌리 문맥이 없습니다"))?;
            let canonical=if legacy_ddl_root==Some(path.as_str()) {
                ddonirang_lang::normalizer::normalize_legacy_saved_ddl_root(&program,&order)?
            } else {
                ddonirang_lang::normalizer::normalize_with_registered_sum_pin_order(&program,ddonirang_lang::NormalizationLevel::N1,&order)?
            };
            charge_role_bytes(&mut canonical_bytes,canonical.len())?;
            catalog.definitions.push(Definition{id:hex::encode(root_id.0),module:hex::encode(module),capture:hex::encode(capture.identity()),start:0,end:root_site.end,
                kind:"root-context".into(),spelling:String::new(),canonical,result_type:String::new(),owner:String::new()});
            for (site,context) in planned.iter().filter(|(site,context)|site.capture==*capture.identity()&&context.name.is_some()) {
                let seed=unique(program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed).filter(|seed|seed.span.start as u64==site.start&&seed.span.end as u64==site.end))?;
                let (types,result)=callable_types.get(site).ok_or_else(||fail("callable의 실제 정적 형식 증명이 없습니다"))?;
                let fragment_order=order.iter().filter(|(node,_)|binder_definitions.get(node)==Some(&context_ids[site])).map(|(node,id)|(*node,*id)).collect();
                let canonical=ddonirang_lang::normalizer::normalize_registered_fragment(&program,ddonirang_lang::normalizer::RegisteredCanonFragment::Definition(seed),&fragment_order)?;
                charge_role_bytes(&mut canonical_bytes,canonical.len())?;
                catalog.definitions.push(Definition{id:hex::encode(context_ids[site].0),module:hex::encode(module),capture:hex::encode(capture.identity()),start:site.start,end:site.end,
                    kind:kind(context)?.into(),spelling:seed.canonical_name.clone(),canonical,result_type:result.as_ref().map(|proof|proof.encode().map(hex::encode)).transpose()?.unwrap_or_default(),owner:hex::encode(root_id.0)});
                for pin in &seed.params {
                    let key=(site.clone(),pin.pin_name.clone());
                    let default_canon=if let Some(expression)=&pin.default_value {
                        let selected=order.iter().filter(|(node,_)|binder_spans.get(node).is_some_and(|span|expression.span.start<=span.start&&span.end<=expression.span.end)).map(|(node,id)|(*node,*id)).collect();
                        ddonirang_lang::normalizer::normalize_registered_fragment(&program,ddonirang_lang::normalizer::RegisteredCanonFragment::Expression(expression),&selected)?
                    } else {String::new()};
                    catalog.pins.push(PinRow{id:hex::encode(callable_pins[&key].0),owner:hex::encode(context_ids[site].0),spelling:pin.pin_name.clone(),
                        type_hex:hex::encode(types.get(&pin.pin_name).ok_or_else(||fail("실제 callable pin 형식이 없습니다"))?.encode()?),required:!pin.optional&&pin.default_value.is_none(),default_canon});
                }
            }
            for (statement,_,seed) in scoped_sum_syntax(&program).0 {
                let Stmt::SumDefinition{span,..}=statement else {unreachable!()};
                let site=Site{module,capture:*capture.identity(),start:span.start as u64,end:span.end as u64};
                let row=unique(templates.templates.iter().filter(|row|row.site==site))?;
                let owner=if let Some(seed)=seed.filter(|seed|!seed.is_relocated_top_level_setup(statement)) {
                    *context_ids.get(&Site{module,capture:*capture.identity(),start:seed.span.start as u64,end:seed.span.end as u64}).ok_or_else(||fail("고름씨의 실제 선언 문맥이 없습니다"))?
                } else {root_id};
                let canonical=ddonirang_lang::normalizer::normalize_registered_fragment(&program,ddonirang_lang::normalizer::RegisteredCanonFragment::Statement(statement),&BTreeMap::new())?;
                charge_role_bytes(&mut canonical_bytes,canonical.len())?;
                catalog.definitions.push(Definition{id:hex::encode(row.id.0),module:hex::encode(module),capture:hex::encode(capture.identity()),start:site.start,end:site.end,
                    kind:"sum-template".into(),spelling:row.spelling.clone(),canonical,result_type:String::new(),owner:hex::encode(owner.0)});
                for branch in &row.branches {catalog.branches.push(BranchRow{id:hex::encode(branch.id.0),template:hex::encode(row.id.0),spelling:branch.spelling.clone()});
                    for pin in &branch.pins {catalog.pins.push(PinRow{id:hex::encode(pin.id.0),owner:hex::encode(branch.id.0),spelling:pin.spelling.clone(),type_hex:hex::encode(&pin.type_bytes),required:true,default_canon:String::new()});}
                }
            }
        }
        for scope in &patterns.scopes {
            catalog.scopes.push(ScopeRow{id:hex::encode(scope.id.0),definition:hex::encode(scope.definition.0),capture:hex::encode(scope.site.capture),start:scope.site.start,end:scope.site.end});
            for binder in &scope.binders {catalog.binders.push(BinderRow{id:hex::encode(binder.id.0),scope:hex::encode(scope.id.0),selected_pin:hex::encode(binder.selected_pin.0),spelling:binder.spelling.clone(),capture:hex::encode(binder.site.capture),start:binder.site.start,end:binder.site.end});}
        }
        catalog.modules.sort();catalog.modules.dedup();catalog.captures.sort_by(|a,b|(&a.module,&a.identity).cmp(&(&b.module,&b.identity)));
        catalog.definitions.sort_by(|a,b|a.id.cmp(&b.id));catalog.branches.sort_by(|a,b|a.id.cmp(&b.id));catalog.pins.sort_by(|a,b|a.id.cmp(&b.id));
        catalog.scopes.sort_by(|a,b|a.id.cmp(&b.id));catalog.binders.sort_by(|a,b|a.id.cmp(&b.id));
        Ok(catalog)
    }
    pub(in crate::authored_project::registration) fn restore(source:&Source,nominals:&super::super::RegistrationDraft,bytes:&[u8])->Result<Self,String> {
        Self::restore_selected(source,nominals,bytes,None)
    }
    pub(in crate::authored_project::registration) fn restore_first_callable(source:&Source,nominals:&super::super::RegistrationDraft,
        first:&crate::authored_project::FirstCallableAuthoredContext,bytes:&[u8])->Result<Self,String> {
        Self::restore_selected(source,nominals,bytes,Some(SelectedPlainCallable::from_context(source,first)?))
    }
    pub(in crate::authored_project::registration) fn recheck(&self,source:&Source,nominals:&super::super::RegistrationDraft)->Result<Self,String> {
        Self::restore_selected_scopes(source,nominals,self.bytes(),self.selected.clone(),self.public.clone())
    }
    pub(in crate::authored_project::registration) fn restore_public_callables(source:&Source,nominals:&super::super::RegistrationDraft,
        bytes:&[u8],public:PublicCallableSelection)->Result<Self,String> {
        Self::restore_selected_scopes(source,nominals,bytes,None,Some(public))
    }
    fn restore_selected(source:&Source,nominals:&super::super::RegistrationDraft,bytes:&[u8],selected:Option<SelectedPlainCallable>)->Result<Self,String> {
        Self::restore_selected_scopes(source,nominals,bytes,selected,None)
    }
    fn restore_selected_scopes(source:&Source,nominals:&super::super::RegistrationDraft,bytes:&[u8],selected:Option<SelectedPlainCallable>,
        public:Option<PublicCallableSelection>)->Result<Self,String> {
        if let Some(public)=&public {public.contexts(source)?;}
        if let Some(selected)=&selected {selected.context(source)?;}
        if bytes.len()>LIMIT||bytes.starts_with(&[0xef,0xbb,0xbf]) {return Err(fail("등록 자료 크기·BOM 오류"));}
        let catalog:Catalog=serde_json::from_slice(bytes).map_err(|_|fail("등록 자료의 닫힌 형식이 다릅니다"))?;
        if catalog.schema!=SCHEMA||catalog.mode!=source.mode()||catalog.scope!=source.scope()
            ||encode(&catalog)?!=bytes {return Err(fail("등록 자료 버전·작품·정본 바이트가 다릅니다"));}
        let mut templates=Vec::new();
        for planned in plan(source,nominals)? {
            let row=unique(catalog.definitions.iter().filter(|row|row.kind=="sum-template"&&row_site(row).ok().as_ref()==Some(&planned.site)))?;
            let mut branches=Vec::new();
            for branch in planned.branches {
                let saved=unique(catalog.branches.iter().filter(|saved|saved.template==row.id&&saved.spelling==branch.spelling))?;
                let mut pins=Vec::new();
                for spelling in branch.pins {let pin=unique(catalog.pins.iter().filter(|pin|pin.owner==saved.id&&pin.spelling==spelling))?;
                    pins.push(Pin{id:PinId(id(&pin.id)?),spelling,type_bytes:raw(&pin.type_hex)?});}
                pins.sort_by_key(|pin|pin.id);branches.push(Branch{id:BranchId(id(&saved.id)?),spelling:branch.spelling,pins});
            }
            branches.sort_by_key(|branch|branch.id);templates.push(Template{id:DefinitionId(id(&row.id)?),site:planned.site,lexical_scope:planned.lexical_scope,spelling:planned.spelling,branches});
        }
        templates.sort_by_key(|row|row.id);let templates=SumRoleCandidates{scope:catalog.scope.clone(),templates};templates.verify(source,nominals)?;
        let planned_contexts=contexts(source,selected.as_ref(),public.as_ref())?;let mut context_ids=BTreeMap::new();
        for (site,context) in &planned_contexts {
            let row=unique(catalog.definitions.iter().filter(|row|row.kind==kind(context).unwrap_or("")&&row_site(row).ok().as_ref()==Some(site)))?;
            context_ids.insert(site.clone(),DefinitionId(id(&row.id)?));
        }
        let mut callable_pins=BTreeMap::new();
        for key in callable_pin_names(source,&planned_contexts)? {
            let row=unique(catalog.pins.iter().filter(|row|row.owner==hex::encode(context_ids[&key.0].0)&&row.spelling==key.1))?;
            callable_pins.insert(key,PinId(id(&row.id)?));
        }
        let mut pattern_contexts=BTreeMap::new();let mut scopes=Vec::new();
        for planned in plan_user_patterns(source,nominals,&templates,true)? {
            let definition=*context_ids.get(&planned.context.site).ok_or_else(||fail("기록된 실제 callable 문맥이 없습니다"))?;
            pattern_contexts.insert(definition,planned.context.clone());
            let row=unique(catalog.scopes.iter().filter(|row|row.definition==hex::encode(definition.0)&&row.capture==hex::encode(planned.site.capture)&&row.start==planned.site.start&&row.end==planned.site.end))?;
            let mut binders=Vec::new();
            for binder in planned.binders {
                let saved=unique(catalog.binders.iter().filter(|saved|saved.scope==row.id&&saved.start==binder.site.start&&saved.end==binder.site.end&&saved.spelling==binder.spelling))?;
                if id(&saved.selected_pin)?!=binder.pin.0{return Err(fail("기록된 지역 이름의 선택 pin이 다릅니다"));}
                binders.push(BinderCandidate{id:BinderId(id(&saved.id)?),site:binder.site,spelling:binder.spelling,selected_pin:binder.pin,template:binder.template,branch:binder.branch,owner_raw:binder.owner_raw});
            }
            binders.sort_by_key(|row|row.id);scopes.push(ScopeCandidate{id:ScopeId(id(&row.id)?),site:planned.site,definition,binders});
        }
        scopes.sort_by_key(|row|row.id);
        let patterns=UserPatternCandidates{contexts:pattern_contexts.into_iter().map(|(id,declaration)|ContextCandidate{id,declaration}).collect(),scopes};
        let expected=Self::project(source,nominals,&templates,&patterns,&context_ids,&callable_pins,selected.as_ref(),public.as_ref(),None)?;
        let legacy_ddl_start=encode(&expected)?!=bytes;
        if legacy_ddl_start {
            // These selected registration owners restore the embedded
            // original project, before the consumer activation exists.
            // Recompute only the old entry root rendering and still require
            // exact equality of the complete, source-owned catalog.
            let legacy_path=match source {
                Source::Saved(saved) if (selected.is_some()||public.is_some())
                    && saved.sources().saved().j1_profile().is_none()
                    && saved.sources().logic_callable_context().is_none() => {
                    Some(saved.project().source_snapshot.entry_path.as_str())
                }
                _=>None,
            };
            let mismatch=||fail("등록 자료가 실제 원문·씨·소유자·정본과 다릅니다");
            let path=legacy_path.ok_or_else(mismatch)?;
            let legacy=Self::project(source,nominals,&templates,&patterns,&context_ids,&callable_pins,selected.as_ref(),public.as_ref(),Some(path))?;
            if encode(&legacy)?!=bytes {return Err(mismatch());}
        }
        Ok(Self{raw:bytes.into(),templates,patterns,selected,public,legacy_ddl_start})
    }
}

#[cfg(test)]
mod g1_plain_callable_tests {
    use super::*;
    use crate::authored_project::{self,FirstCallableAuthoredContext,ProjectArtifact};
    use crate::authored_project::registration as registration;
    const ORIGINAL:&str = r###"{"authored_identity":{"clone_origin":null,"entry":null,"logical_dependency_id":"fe6b4154262752919656f5a6f7e3629b3e81758594783b9efca19a1f0ce9ed52","modules":[{"compositions":[],"head":"세금.ddn","id":"0a9e9104a2f6a6434c2bd0dd41a721a7dacc85125c791fe47aed1deb2f4f2068","name":"세금","sources":["세금.ddn"]},{"compositions":[],"head":"app.ddn","id":"36612e8bcf97c580cb38d1c3173b719c4acbf892a937bbe97e9df9f947521c8c","name":"앱","sources":["app.ddn"]}],"schema":"ddn.authored_identity.v1"},"content_sha256":"60783cb50f9b5e604197407d2c7241c990e67171ee5608e2b10593d9b7ee1659","schema":"ddn.authored_project_artifact.v1","source_profile":"v1-core-v25","source_snapshot":{"entry_path":"app.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\"), (이름=세금, 머리=\"세금.ddn\", 원문=\"세금.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n"},{"path":"app.ddn","text":"(세금=세금) 쓰임.\n실행:움직씨 := {\n    ((금액=(100, 1) 나눔수, 세액=(10, 1) 나눔수) 세금.총액합하다) 보여주기.\n}.\n"},{"path":"세금.ddn","text":"(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를)\n총액합하다{총액합하기}:나눔수:셈씨 := {\n    총액합하다 <- 금액 + 세액.\n}.\n"}],"root_path":"root.ddn"}}"###;
    const DDL:&str = r###"#!日本語
(金額:有理数~に, 税額:有理数~を)
合計する:有理数:関数 :={...}.
"###;
    fn project()->ProjectArtifact {authored_project::open(ORIGINAL.as_bytes()).unwrap()}
    fn source(project:&ProjectArtifact)->crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1 {
        crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(project.save().unwrap().as_bytes()).unwrap()
    }
    fn context(project:&ProjectArtifact)->FirstCallableAuthoredContext {
        let module=project.authored_identity.modules.iter().find(|m|m.head=="세금.ddn").unwrap();
        project.first_callable_registration_context(module.id.as_str(),"세금.ddn",DDL,"세금.일본어.ddl","language-profile:ja",1).unwrap()
    }
    fn next(count:&mut u64,id:&mut [u8;32])->Result<(),String> {*count+=1;id.fill(0);id[0]=211;id[24..].copy_from_slice(&count.to_be_bytes());Ok(())}
    fn pair(project:&ProjectArtifact,count:&mut u64)->(registration::RegistrationDraft,crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1) {
        let owned=source(project);
        let draft=registration::prepare_first_callable_under_candidate_owner(&owned,&context(project),None,&mut |id|next(count,id)).unwrap();
        (draft,owned)
    }
    fn prepare(project:&ProjectArtifact,count:&mut u64)->registration::RegistrationDraft {pair(project,count).0}
    fn catalog(draft:&registration::RegistrationDraft)->serde_json::Value {
        serde_json::from_slice(draft.artifacts().iter().find(|b|b.role()==Some(ROLE)).unwrap().bytes()).unwrap()
    }
    fn reseal(draft:&registration::RegistrationDraft,mutate:impl FnOnce(&mut serde_json::Value))->Vec<registration::RecordedRegistrationBlob> {
        let mut records=draft.recorded_blobs();let row=records.iter_mut().find(|r|r.role.as_deref()==Some(ROLE)).unwrap();
        let mut value:serde_json::Value=serde_json::from_slice(&hex::decode(&row.raw_hex).unwrap()).unwrap();mutate(&mut value);
        // Same production transport field order; change a typed row, then
        // reseal its transport so the semantic verifier must detect it.
        let value:Catalog=serde_json::from_value(value).unwrap();let raw=encode(&value).unwrap();
        row.byte_length=raw.len() as u64;row.sha256=hex::encode(Sha256::digest(&raw));row.raw_hex=hex::encode(raw);records
    }
    #[test]
    fn g1_plain_actual_two_pin_issuer_and_legacy_unchanged() {
        let project=project();let owned=source(&project);let mut old_count=0;
        let old=registration::prepare_precanon_under_candidate_owner(&owned,None,&mut |id|next(&mut old_count,id)).unwrap();
        assert!(!ResolvedCatalog::needed(&Source::saved(&owned)).unwrap());assert!(old.resolved.is_none());
        let bytes=serde_json::to_vec(&old.recorded_blobs()).unwrap();let mut count=0;let draft=prepare(&project,&mut count);
        assert!(count>0);let json=catalog(&draft);assert_eq!(json["schema"],SCHEMA);
        let callable=json["definitions"].as_array().unwrap().iter().find(|r|r["kind"]=="callable-context").unwrap();
        assert_eq!(callable["spelling"],"총액합하다");assert!(!callable["result_type"].as_str().unwrap().is_empty());
        assert_eq!(callable["module"],context(&project).module_identity().as_str());
        let pins=json["pins"].as_array().unwrap();assert_eq!(pins.len(),2);
        let names=pins.iter().map(|r|r["spelling"].as_str().unwrap()).collect::<BTreeSet<_>>();assert_eq!(names,BTreeSet::from(["금액","세액"]));
        for pin in pins {assert_eq!(pin["owner"],callable["id"]);assert_eq!(pin["required"],true);assert_eq!(pin["default_canon"],"");assert!(!pin["type"].as_str().unwrap().is_empty());}
        assert_ne!(pins[0]["id"],pins[1]["id"]);assert!(json["providers"].as_array().unwrap().is_empty());
        let mut again_count=0;let again=registration::prepare_precanon_under_candidate_owner(&owned,None,&mut |id|next(&mut again_count,id)).unwrap();
        assert_eq!(old_count,again_count);assert_eq!(serde_json::to_vec(&again.recorded_blobs()).unwrap(),bytes);
        assert!(registration::restore_precanon_recorded_candidate(&owned,draft.recorded_blobs()).is_err(),"legacy read must not silently opt into selected G1");
    }
    #[test]
    fn g1_plain_cold_recorded_reopen_keeps_actual_ids_without_entropy() {
        let project=project();let mut count=0;let (draft,fresh)=pair(&project,&mut count);let before=serde_json::to_vec(&draft.recorded_blobs()).unwrap();
        // Cold restore consumes recorded capture identities, never fresh capture ordinals.
        let captures=fresh.sources().recorded_capture_catalog_json().unwrap();
        let artifact=project.save().unwrap();
        let original=crate::project_revision_envelope_supply::admit_saved_authored_project(fresh.sources().clone(),artifact.as_bytes()).unwrap();
        let restored_sources=crate::project_revision_envelope_supply::restore_saved_v25_sources(original.resolved_revision_supply(),&captures).unwrap();
        let reopened=authored_project::open(artifact.as_bytes()).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_precanon_authored_source(std::sync::Arc::new(restored_sources),reopened.save().unwrap().as_bytes()).unwrap();
        let records=serde_json::from_slice(&before).unwrap();
        let restored=registration::restore_precanon_first_callable_candidate(&owned,&context(&reopened),records).unwrap();
        assert_eq!(serde_json::to_vec(&restored.recorded_blobs()).unwrap(),before);
        let material=restored.resolved_start_material(&owned).unwrap().unwrap();assert_eq!(material.1,draft.resolved.as_ref().unwrap().bytes());
        let mut reuse_count=0;let reused=registration::prepare_first_callable_under_candidate_owner(&owned,&context(&reopened),Some(&restored),&mut |id|next(&mut reuse_count,id)).unwrap();
        assert_eq!(reuse_count,0);assert_eq!(serde_json::to_vec(&reused.recorded_blobs()).unwrap(),before);
    }
    #[test]
    fn g1_plain_missing_forged_pin_owner_and_type_refuse_after_transport_reseal() {
        let project=project();let first=context(&project);let mut count=0;let (draft,owned)=pair(&project,&mut count);
        for records in [reseal(&draft,|j|{j["pins"].as_array_mut().unwrap().pop();}),
            reseal(&draft,|j|{j["pins"][0]["owner"]=j["definitions"].as_array().unwrap().iter().find(|r|r["kind"]=="root-context").unwrap()["id"].clone();}),
            reseal(&draft,|j|{let other=project.authored_identity.modules.iter().find(|m|m.head=="app.ddn").unwrap().id.as_str();
                j["definitions"].as_array_mut().unwrap().iter_mut().find(|r|r["kind"]=="callable-context").unwrap()["module"]=serde_json::json!(other);}),
            reseal(&draft,|j|{j["pins"][0]["type"]=serde_json::json!("00");}),
            reseal(&draft,|j|{j["pins"][1]["id"]=j["pins"][0]["id"].clone();})] {
            assert!(registration::restore_precanon_first_callable_candidate(&owned,&first,records).is_err());
        }
        let mut missing=draft.recorded_blobs();missing.retain(|r|r.role.as_deref()!=Some(ROLE));
        assert!(registration::restore_precanon_first_callable_candidate(&owned,&first,missing).is_err());
    }
    #[test]
    fn g1_plain_same_spelling_other_actual_module_and_stale_source_refuse() {
        let first=project();let original_context=context(&first);let mut count=0;let draft=prepare(&first,&mut count);
        let other=first.duplicate().unwrap();let other=authored_project::open(other.save().unwrap().as_bytes()).unwrap();
        assert_ne!(context(&other).module_identity(),original_context.module_identity());
        let other_source=source(&other);let mut invoked=0;
        assert!(registration::prepare_first_callable_under_candidate_owner(&other_source,&original_context,None,&mut |id|next(&mut invoked,id)).is_err());assert_eq!(invoked,0);
        let mut other_count=100;let other_draft=prepare(&other,&mut other_count);assert_ne!(catalog(&draft)["pins"],catalog(&other_draft)["pins"]);
        assert!(registration::restore_precanon_first_callable_candidate(&other_source,&context(&other),draft.recorded_blobs()).is_err());
        let mut changed=first.clone();changed.source_snapshot.files.iter_mut().find(|f|f.path=="세금.ddn").unwrap().text.push_str(" ");
        let changed=authored_project::open(changed.save().unwrap().as_bytes()).unwrap();let changed_source=source(&changed);
        assert!(registration::prepare_first_callable_under_candidate_owner(&changed_source,&original_context,None,&mut |id|next(&mut invoked,id)).is_err());
        assert!(registration::restore_precanon_first_callable_candidate(&changed_source,&context(&changed),draft.recorded_blobs()).is_err());assert_eq!(invoked,0);
    }
}


#[cfg(test)]
mod r21_public_callable_tests {
    use super::*;
    use crate::authored_project::{self,ProjectArtifact,SourceFile,SourceSnapshot};
    use crate::authored_project::registration;
    use crate::project_revision_envelope_supply as supply;

    fn project()->ProjectArtifact {
        let snapshot=SourceSnapshot{entry_path:"main.ddn".into(),root_path:Some("root.ddn".into()),files:vec![
            SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\"),(이름=모음,머리=\"모음.ddn\",원문=차림[\"모음.ddn\",\"추가.ddn\"])], 시작=(모듈=앱,정의=실행)) 얽음.\n".into()},
            SourceFile{path:"main.ddn".into(),text:"(모음=모음) 쓰임.\n실행:움직씨 := {\n((왼값=(2,1) 나눔수, 오른값=(3,1) 나눔수) 모음.첫째) 보여주기.\n((값=(4,1) 나눔수) 모음.둘째) 보여주기.\n}.\n".into()},
            SourceFile{path:"모음.ddn".into(),text:"#!한국어\n// 日本語 위치 보존\n(첫째=더하다, 둘째=두배하다) 드러냄.\n(왼값:나눔수~에, 오른값:나눔수~을) 더하다:나눔수:셈씨 := { 더하다 <- 왼값 + 오른값. }.\n(숨은값:나눔수~을) 숨기다:나눔수:셈씨 := { 숨기다 <- 숨은값. }.\n".into()},
            SourceFile{path:"추가.ddn".into(),text:"(값:나눔수~을) 두배하다:나눔수:셈씨 := { 두배하다 <- 값 + 값. }.\n".into()},
        ]};
        let made=ProjectArtifact::create("v1-core-v25".into(),snapshot,String::new()).unwrap();
        authored_project::open(made.save().unwrap().as_bytes()).unwrap()
    }
    fn owner(project:&ProjectArtifact)->String {project.authored_identity.modules.iter().find(|m|m.name=="모음").unwrap().id.as_str().into()}
    fn source(project:&ProjectArtifact)->supply::PreCanonAuthoredSourceV1 {
        supply::admit_fresh_precanon_authored_source(project.save().unwrap().as_bytes()).unwrap()
    }
    fn next(count:&mut u64,id:&mut [u8;32])->Result<(),String>{*count+=1;id.fill(0);id[0]=213;id[24..].copy_from_slice(&count.to_be_bytes());Ok(())}
    fn prepare(owned:&supply::PreCanonAuthoredSourceV1,module:&str,count:&mut u64)->registration::RegistrationDraft {
        registration::prepare_public_callables_under_candidate_owner(owned,module,None,&mut |id|next(count,id)).unwrap()
    }
    fn transport(draft:&registration::RegistrationDraft)->Catalog {
        serde_json::from_slice(draft.resolved.as_ref().unwrap().bytes()).unwrap()
    }
    fn reseal(draft:&registration::RegistrationDraft,mutate:impl FnOnce(&mut Catalog))->Vec<registration::RecordedRegistrationBlob> {
        let mut records=draft.recorded_blobs();let row=records.iter_mut().find(|r|r.role.as_deref()==Some(ROLE)).unwrap();
        let mut catalog:Catalog=serde_json::from_slice(&hex::decode(&row.raw_hex).unwrap()).unwrap();
        mutate(&mut catalog);let bytes=encode(&catalog).unwrap();
        row.byte_length=bytes.len() as u64;row.sha256=hex::encode(Sha256::digest(&bytes));row.raw_hex=hex::encode(bytes);records
    }
    #[test]
    fn r21_explicit_public_owner_issues_f_g_and_preserves_default_old_bytes() {
        let project=project();let owned=source(&project);let view=Source::saved(&owned);let module=owner(&project);
        assert!(!ResolvedCatalog::needed(&view).unwrap());
        let mut old_count=0;let old=registration::prepare_precanon_under_candidate_owner(&owned,None,&mut |id|next(&mut old_count,id)).unwrap();
        let old_bytes=serde_json::to_vec(&old.recorded_blobs()).unwrap();assert!(old.resolved.is_none());
        let mut count=0;let draft=prepare(&owned,&module,&mut count);assert!(count>0);
        let catalog=transport(&draft);
        let rows=catalog.definitions.iter().filter(|r|r.kind=="callable-context").collect::<Vec<_>>();
        assert_eq!(rows.len(),2);assert_eq!(rows.iter().map(|r|r.spelling.as_str()).collect::<BTreeSet<_>>(),BTreeSet::from(["더하다","두배하다"]));
        assert_ne!(rows[0].id,rows[1].id);assert!(rows.iter().all(|r|r.module==module&&!r.result_type.is_empty()));
        assert_eq!(catalog.pins.len(),3);assert!(catalog.providers.is_empty());
        for row in rows {
            let pin_count=if row.spelling=="더하다"{2}else{1};
            assert_eq!(catalog.pins.iter().filter(|p|p.owner==row.id).count(),pin_count);
            let capture=owned.sources().captures().iter().find(|(_,c)|hex::encode(c.identity())==row.capture).unwrap();
            assert!(capture.1.text().get(row.start as usize..row.end as usize).unwrap().contains(&row.spelling));
        }
        assert_eq!(catalog.pins.iter().map(|p|p.id.as_str()).collect::<BTreeSet<_>>().len(),3);
        assert!(catalog.pins.iter().all(|p|p.required&&!p.type_hex.is_empty()));
        let restored_old=registration::restore_precanon_recorded_candidate(&owned,serde_json::from_slice(&old_bytes).unwrap()).unwrap();
        assert_eq!(serde_json::to_vec(&restored_old.recorded_blobs()).unwrap(),old_bytes);
        assert!(registration::restore_precanon_recorded_candidate(&owned,draft.recorded_blobs()).is_err());
        let mut again_count=0;let again=registration::prepare_precanon_under_candidate_owner(&owned,None,&mut |id|next(&mut again_count,id)).unwrap();
        assert_eq!(old_count,again_count);assert_eq!(serde_json::to_vec(&again.recorded_blobs()).unwrap(),old_bytes);
    }
    #[test]
    fn r21_public_recorded_cold_restore_keeps_ids_and_uses_no_entropy() {
        let project=project();let fresh=source(&project);let module=owner(&project);let mut count=0;let draft=prepare(&fresh,&module,&mut count);
        let before=serde_json::to_vec(&draft.recorded_blobs()).unwrap();
        let captures=fresh.sources().recorded_capture_catalog_json().unwrap();let artifact=project.save().unwrap();
        let original=supply::admit_saved_authored_project(fresh.sources().clone(),artifact.as_bytes()).unwrap();
        let restored_sources=supply::restore_saved_v25_sources(original.resolved_revision_supply(),&captures).unwrap();
        let owned=supply::admit_precanon_authored_source(std::sync::Arc::new(restored_sources),artifact.as_bytes()).unwrap();
        let restored=registration::restore_precanon_public_callables_candidate(&owned,&module,serde_json::from_slice(&before).unwrap()).unwrap();
        assert_eq!(serde_json::to_vec(&restored.recorded_blobs()).unwrap(),before);
        let mut reuse_count=0;let again=registration::prepare_public_callables_under_candidate_owner(&owned,&module,Some(&restored),&mut |id|next(&mut reuse_count,id)).unwrap();
        assert_eq!(reuse_count,0);assert_eq!(serde_json::to_vec(&again.recorded_blobs()).unwrap(),before);
        assert_eq!(restored.resolved_start_material(&owned).unwrap().unwrap().1,draft.resolved.as_ref().unwrap().bytes());
    }
    #[test]
    fn r21_public_owner_refuses_forged_pin_type_foreign_and_stale_source() {
        let project=project();let owned=source(&project);let module=owner(&project);let mut count=0;let draft=prepare(&owned,&module,&mut count);
        for records in [reseal(&draft,|c|{c.pins[0].owner=c.definitions.iter().find(|r|r.kind=="root-context").unwrap().id.clone();}),
            reseal(&draft,|c|{c.pins[0].type_hex="00".into();})] {
            assert!(registration::restore_precanon_public_callables_candidate(&owned,&module,records).is_err());
        }
        let mut invoked=0;
        assert!(registration::prepare_public_callables_under_candidate_owner(&owned,&"ff".repeat(32),None,&mut |id|next(&mut invoked,id)).is_err());assert_eq!(invoked,0);
        let other=project.duplicate().unwrap();let other=authored_project::open(other.save().unwrap().as_bytes()).unwrap();let other_source=source(&other);
        assert!(registration::restore_precanon_public_callables_candidate(&other_source,&owner(&other),draft.recorded_blobs()).is_err());
        let mut changed=project.clone();changed.source_snapshot.files.iter_mut().find(|f|f.path=="추가.ddn").unwrap().text.push_str("// 판본 변경\n");
        let changed=authored_project::open(changed.save().unwrap().as_bytes()).unwrap();
        assert!(registration::restore_precanon_public_callables_candidate(&source(&changed),&owner(&changed),draft.recorded_blobs()).is_err());
    }
}

#[cfg(test)]
mod r25_public_export_tests {
    use super::*;
    use crate::authored_project::{self,ProjectArtifact};
    use crate::authored_project::registration;
    use crate::project_revision_envelope_supply as supply;
    const ORIGINAL:&str=r#####"{"authored_identity":{"clone_origin":null,"entry":null,"logical_dependency_id":"d93448392868869dab65fa96a8a73c3869f07eaa540229b427d60ef336fcd9e9","modules":[{"compositions":[],"head":"main.ddn","id":"5143b023273c0ce951e9b7a410db70c3f1849c2492a0ee9c46eac577a1f2350b","name":"앱","sources":["main.ddn"]},{"compositions":[],"head":"함수.ddn","id":"a21d1b41d973933d2b3957e49fe998e3a98ee6d1871bfd79a707a94f6dd2ebb4","name":"함수","sources":["함수.ddn"]}],"schema":"ddn.authored_identity.v1"},"content_sha256":"724af7c43bb6492f703355101cbeb42371f05efd4d390ff0d6a884adbdd5dd53","schema":"ddn.authored_project_artifact.v1","source_profile":"v1-core-v25","source_snapshot":{"entry_path":"main.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\"),(이름=함수,머리=\"함수.ddn\",원문=\"함수.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n"},{"path":"main.ddn","text":"(함수=함수) 쓰임.\n실행:움직씨 := {\n    ((q=(3,1) 나눔수, p=(12,1) 나눔수) 함수.F) 보여주기.\n    ((r=(4,1) 나눔수) 함수.G) 보여주기.\n}.\n"},{"path":"함수.ddn","text":"(F=F, G=G) 드러냄.\n(p:나눔수~에, q:나눔수~을) F:나눔수:셈씨 := { F <- p + q. }.\n(r:나눔수~을) G:나눔수:셈씨 := { G <- r + r. }.\n"}],"root_path":"root.ddn"}}"#####;
    fn project()->ProjectArtifact {authored_project::open(ORIGINAL.as_bytes()).unwrap()}
    fn source(project:&ProjectArtifact)->supply::PreCanonAuthoredSourceV1 {
        supply::admit_fresh_precanon_authored_source(project.save().unwrap().as_bytes()).unwrap()
    }
    fn module(project:&ProjectArtifact)->String {project.authored_identity.modules.iter().find(|row|row.name=="함수").unwrap().id.as_str().into()}
    fn next(count:&mut u64,id:&mut [u8;32])->Result<(),String> {*count+=1;id.fill(0);id[0]=214;id[24..].copy_from_slice(&count.to_be_bytes());Ok(())}
    fn requested(facts:&registration::CheckedPublicCallableExport)->Vec<registration::PublicCallableSurfaceRequest> {
        facts.definitions.iter().map(|row|registration::PublicCallableSurfaceRequest{definition:row.identity.clone(),surface:row.public_name.clone(),
            pins:row.pins.iter().enumerate().map(|(i,pin)|(pin.identity.clone(),if i==0{"金額".into()}else{"税額".into()},if row.pins.len()>1&&i==0{"に".into()}else{"を".into()})).collect()}).collect()
    }
    #[test]
    fn r25_actual_full_public_f_g_profile_and_recorded_owner_recheck() {
        let project=project();let owned=source(&project);let module=module(&project);let mut calls=0;
        let draft=registration::prepare_public_callables_under_candidate_owner(&owned,&module,None,&mut |id|next(&mut calls,id)).unwrap();
        let facts=draft.checked_public_callable_export(&owned,&module).unwrap();
        assert_eq!(facts.definitions.len(),2);assert_eq!(facts.definitions.iter().map(|row|row.pins.len()).sum::<usize>(),3);
        assert_eq!(facts.definitions.iter().map(|row|row.public_name.as_str()).collect::<BTreeSet<_>>(),BTreeSet::from(["F","G"]));
        assert!(facts.definitions.iter().all(|row|!row.result_type.is_empty()&&row.closed_body_canon.starts_with("add(")&&row.source_path=="함수.ddn"));
        let mut registry=ddonirang_lang::ddl::SuccessorRegistryOwner::default();let mut profile_calls=0;
        let (export,profile)=draft.issue_public_callable_profile(&owned,&module,requested(&facts),&mut registry,&mut |raw|{profile_calls+=1;raw.fill(199);Ok(())}).unwrap();
        assert_eq!(profile_calls,1);assert_eq!(export,facts);assert_eq!(profile.surfaces().len(),2);
        let same_name=profile.surfaces().iter().flat_map(|row|row.pins.iter().filter(|pin|pin.surface=="金額").map(move |pin|(&row.identity,&pin.identity))).collect::<Vec<_>>();
        assert_eq!(same_name.len(),2);assert_ne!(same_name[0],same_name[1]);assert_ne!(same_name[0].1,same_name[1].1);
        assert!(registry.lookup(profile.identity(),"1",profile.content_sha256(),1).is_err());
        assert_eq!(registry.resolve_exported(&[profile.clone()],"日本語","2").unwrap().content(),profile.content());
        // Restore the real recorded captures and G1 catalog without entropy;
        // this is not Native persistence or restoration of a new wire format.
        let artifact=project.save().unwrap();let original=supply::admit_saved_authored_project(owned.sources().clone(),artifact.as_bytes()).unwrap();
        let restored_sources=supply::restore_saved_v25_sources(original.resolved_revision_supply(),&owned.sources().recorded_capture_catalog_json().unwrap()).unwrap();
        let cold=supply::admit_precanon_authored_source(std::sync::Arc::new(restored_sources),artifact.as_bytes()).unwrap();
        let restored=registration::restore_precanon_public_callables_candidate(&cold,&module,draft.recorded_blobs()).unwrap();
        let cold_facts=restored.checked_public_callable_export(&cold,&module).unwrap();assert_eq!(cold_facts,facts);
        let (_,same)=restored.issue_public_callable_profile(&cold,&module,requested(&cold_facts),&mut registry,&mut |_|panic!("same owner content cannot reissue")).unwrap();
        assert_eq!(same.identity(),profile.identity());
    }
    #[test]
    fn r25_public_coverage_cross_definition_pin_and_stale_refused_before_issuance() {
        let project=project();let owned=source(&project);let module=module(&project);let mut count=0;
        let draft=registration::prepare_public_callables_under_candidate_owner(&owned,&module,None,&mut |id|next(&mut count,id)).unwrap();
        let facts=draft.checked_public_callable_export(&owned,&module).unwrap();let all=requested(&facts);
        let mut omitted=all.clone();omitted.pop();
        let mut forged=all.clone();forged[0].pins[0].0=all[1].pins[0].0.clone();
        let mut extra=all.clone();let duplicate=extra[0].pins[0].clone();extra[0].pins.push(duplicate);
        let mut registry=ddonirang_lang::ddl::SuccessorRegistryOwner::default();
        for rows in [omitted,forged,extra] {
            assert!(draft.issue_public_callable_profile(&owned,&module,rows,&mut registry,&mut |_|panic!("bad coverage cannot issue")).is_err());
        }
        let mut changed=project.clone();changed.source_snapshot.files.iter_mut().find(|file|file.path=="함수.ddn").unwrap().text.push_str("// stale 日本語\n");
        let changed=authored_project::open(changed.save().unwrap().as_bytes()).unwrap();
        assert!(draft.checked_public_callable_export(&source(&changed),&module).is_err());
        assert!(draft.checked_public_callable_export(&owned,&"ff".repeat(32)).is_err());
    }
    #[test]
    fn r25_actual_body_dependency_scope_not_inferred_from_semssi_kind() {
        let project=project();let mut project=project.clone();
        let row=project.source_snapshot.files.iter_mut().find(|row|row.path=="함수.ddn").unwrap();
        row.text=row.text.replace("F <- p + q","F <- p - q");
        let project=authored_project::open(project.save().unwrap().as_bytes()).unwrap();let owned=source(&project);let module=module(&project);let mut count=0;
        let draft=registration::prepare_public_callables_under_candidate_owner(&owned,&module,None,&mut |id|next(&mut count,id)).unwrap();
        let failure=draft.checked_public_callable_export(&owned,&module).unwrap_err();assert!(failure.contains("E_DDL_PUBLIC_BODY_SCOPE"),"{failure}");
    }
}
