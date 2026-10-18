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
    raw:Vec<u8>,templates:SumRoleCandidates,patterns:UserPatternCandidates,
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
fn contexts(source:&Source)->Result<BTreeMap<Site,std::sync::Arc<DeclarationContext>>,String> {
    let preparation=source.preparation();
    let program=ddonirang_lang::parse_v25_with_mode(&preparation.source,&preparation.target_path,ddonirang_lang::ParseMode::Strict)
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
        let program=ddonirang_lang::parse_v25_with_mode(capture.text(),path,ddonirang_lang::ParseMode::Strict).map_err(|e|fail(&e.to_string()))?;
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
        let program=ddonirang_lang::parse_v25_with_mode(&preparation.source,&preparation.target_path,ddonirang_lang::ParseMode::Strict).map_err(|e|fail(&e.to_string()))?;
        let (declarations,matches)=scoped_sum_syntax(&program);Ok(!declarations.is_empty()||!matches.is_empty())
    }
    pub(in crate::authored_project::registration) fn issue(source:&Source,nominals:&super::super::RegistrationDraft,
        entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String> {
        let mut templates=SumRoleCandidates::reserve(source,nominals,entropy)?;
        let planned=contexts(source)?;
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
        let transport=Self::project(source,nominals,&templates,&patterns,&context_ids,&callable_pins)?;
        let raw=encode(&transport)?;
        if raw.len()>LIMIT {return Err(fail("등록 자료 크기 한도입니다"));}
        Ok(Self{raw,templates,patterns})
    }
    pub(in crate::authored_project::registration) fn bytes(&self)->&[u8] {&self.raw}
    pub(in crate::authored_project::registration) fn checked_canon(&self,source:&Source,nominals:&super::super::RegistrationDraft)->Result<String,String> {
        self.patterns.checked_canon(source,nominals,&self.templates)
    }
    pub(in crate::authored_project::registration) fn bind_program(&self,source:&Source,program:&mut CanonProgram)->Result<(),String> {
        let order=self.patterns.pin_order(source,program)?;
        program.origin.registered_sum_pin_order=Some(order);
        ddonirang_lang::canonicalize(program).map_err(|error|fail(&error.to_string()))?;
        Ok(())
    }
    fn project(source:&Source,nominals:&super::super::RegistrationDraft,templates:&SumRoleCandidates,
        patterns:&UserPatternCandidates,context_ids:&BTreeMap<Site,DefinitionId>,callable_pins:&BTreeMap<(Site,String),PinId>)->Result<Catalog,String> {
        templates.verify(source,nominals)?;patterns.verify(source,nominals,templates)?;
        let planned=contexts(source)?;
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
            let mut program=ddonirang_lang::parse_v25_with_mode(capture.text(),path,ddonirang_lang::ParseMode::Strict).map_err(|e|fail(&e.to_string()))?;
            let mut order=BTreeMap::new();let mut binder_definitions=BTreeMap::new();let mut binder_spans=BTreeMap::new();
            for stmt in sum_syntax(&program).1 {let Stmt::SumMatch{arms,..}=stmt else {unreachable!()};for arm in arms {
                let site=Site{module,capture:*capture.identity(),start:arm.span.start as u64,end:arm.span.end as u64};
                let scope=unique(patterns.scopes.iter().filter(|scope|scope.site==site))?;
                for binder in &arm.binder {let selected=unique(scope.binders.iter().filter(|row|row.site.start==binder.span.start as u64&&row.site.end==binder.span.end as u64&&row.spelling==binder.pin))?;order.insert(binder.id,selected.selected_pin.0);binder_definitions.insert(binder.id,scope.definition);binder_spans.insert(binder.id,binder.span);}
            }}
            ddonirang_lang::canonicalize(&mut program).map_err(|e|fail(&e.to_string()))?;
            let root_site=Site{module,capture:*capture.identity(),start:0,end:capture.bytes().len() as u64};
            let root_id=*context_ids.get(&root_site).ok_or_else(||fail("실제 원문 뿌리 문맥이 없습니다"))?;
            let canonical=ddonirang_lang::normalizer::normalize_with_registered_sum_pin_order(&program,ddonirang_lang::NormalizationLevel::N1,&order)?;
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
        let planned_contexts=contexts(source)?;let mut context_ids=BTreeMap::new();
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
        let expected=Self::project(source,nominals,&templates,&patterns,&context_ids,&callable_pins)?;
        if encode(&expected)?!=bytes {return Err(fail("등록 자료가 실제 원문·씨·소유자·정본과 다릅니다"));}
        Ok(Self{raw:bytes.into(),templates,patterns})
    }
}
