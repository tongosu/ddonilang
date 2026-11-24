//! Actual retained source/Bool-type supply for B1's private execution consumer.
//! Not DDL activation, a resolved call target, a storage record or public admission.
use super::{fail,reserved,typeproof::StaticTypeOwner,Source};
use crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1;
use ddonirang_lang::{Expr,Span};
use ddonirang_lang::stdlib::logic_callable_registration_r1 as logic;

/// Complete builtin export, not an authored-module impersonation or activation.
/// Type bytes come only from the actual source's checked primitive Bool owner.
/// No Deserialize/raw constructor: a signature list or manifest alone is insufficient.
pub(crate) struct CheckedLogicExport {
    owner:[u8;32],
    version:u32,
    material_digest:&'static str,
    bool_type:Vec<u8>,
    definitions:Vec<logic::CheckedLogicDefinition>,
}
/// Borrowed typed export rows; not a SeedDef/body or a registered source call.
pub(crate) struct CheckedLogicCallableExport<'a>{
    descriptor:&'a logic::CheckedLogicDefinition,
    result_type_proof:&'a [u8],
    pins:Vec<CheckedLogicPinExport<'a>>,
}
pub(crate) struct CheckedLogicPinExport<'a>{
    identity:&'a [u8;32],
    role:logic::LogicPinRole,
    original_name:&'static str,
    surface:&'static str,
    type_proof:&'a [u8],
}
impl CheckedLogicCallableExport<'_>{
    pub(crate) fn descriptor(&self)->&logic::CheckedLogicDefinition{self.descriptor}
    pub(crate) fn result_type_proof(&self)->&[u8]{self.result_type_proof}
    pub(crate) fn pins(&self)->&[CheckedLogicPinExport<'_>]{&self.pins}
}
impl CheckedLogicPinExport<'_>{
    pub(crate) fn identity(&self)->&[u8;32]{self.identity}
    pub(crate) fn role(&self)->logic::LogicPinRole{self.role}
    pub(crate) fn original_name(&self)->&str{self.original_name}
    pub(crate) fn surface(&self)->&str{self.surface}
    pub(crate) fn type_proof(&self)->&[u8]{self.type_proof}
}
/// Actual common-parser DDL projections bound to this complete builtin export.
/// Not an executable context: explicit module/profile activation is still required.
pub(crate) struct CheckedLogicDdl<'a>{
    export:&'a CheckedLogicExport,
    source:String,path:String,
    definitions:Vec<CheckedLogicDdlDefinition>,
}
pub(crate) struct CheckedLogicDdlDefinition{
    identity:[u8;32],projection:ddonirang_lang::SeedDef,
    pins:Vec<(ddonirang_lang::NodeId,[u8;32])>,
}
/// Owner-produced builtin export. This is not a source module identity,
/// source-call binding or permission to activate without an import record.
pub(crate) struct CheckedLogicModuleExport<'a>{
    ddl:CheckedLogicDdl<'a>,profile:ddonirang_lang::ddl::RegisteredSuccessorProfile,
}
/// Profile selection from the actual common module prefix, not call admission.
/// The builtin module provider and source/Bool/call binder must still validate
/// the full actual project. This token is not accepted by the runtime consumer.
pub(crate) struct PendingLogicImport<'a,'b>{
    exported:&'a CheckedLogicModuleExport<'b>,
    source:String,path:String,alias:String,request_span:Span,declaration_start:usize,
}
impl PendingLogicImport<'_,'_>{
    pub(crate) fn alias(&self)->&str{&self.alias}
    pub(crate) fn owner(&self)->&[u8;32]{self.exported.owner()}
    pub(crate) fn profile(&self)->&ddonirang_lang::ddl::RegisteredSuccessorProfile{self.exported.profile()}
    pub(crate) fn request_span(&self)->Span{self.request_span}
    pub(crate) fn declaration_start(&self)->usize{self.declaration_start}
    pub(crate) fn verify_source(&self,source:&str,path:&str)->Result<(),String>{
        if self.source!=source||self.path!=path{return Err(fail("E_LOGIC_IMPORT_SOURCE_MISMATCH"));}Ok(())
    }
    /// Common compiler output only. The actual project provider/source/type
    /// admission is mandatory before this can become an executable program.
    pub(crate) fn parse_program(&self,syntax:ddonirang_lang::ddl::CheckedJ1Profile)
        ->Result<ddonirang_lang::CanonProgram,ddonirang_lang::ParseError>{
        let ddl=&self.exported.ddl;
        let context=ddonirang_lang::parser::RegisteredLogicCallableContext::from_exported_ddl(
            self.exported.profile.clone(),ddl.export.definitions.clone(),&ddl.source,&ddl.path,syntax)?
            .with_module_activation(ddonirang_lang::ModuleDirectiveEntry{local_name:self.alias.clone(),target:"logic".into()})?;
        ddonirang_lang::Parser::parse_with_logic_callable_context(&self.source,&self.path,syntax,std::sync::Arc::new(context))
    }
}
impl<'a> CheckedLogicDdl<'a>{
    pub(crate) fn export(&self)->&CheckedLogicExport{self.export}
    pub(crate) fn source(&self)->&str{&self.source}
    pub(crate) fn path(&self)->&str{&self.path}
    pub(crate) fn definitions(&self)->&[CheckedLogicDdlDefinition]{&self.definitions}
    pub(crate) fn issue_exported_profile(self,registry:&mut ddonirang_lang::ddl::SuccessorRegistryOwner,
        entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>,
    )->Result<CheckedLogicModuleExport<'a>,String>{
        let profile=registry.issue_logic(&self.export.definitions,entropy).map_err(|e|e.to_string())?;
        Ok(CheckedLogicModuleExport{ddl:self,profile})
    }
    pub(crate) fn restore_exported_profile(self,registry:&mut ddonirang_lang::ddl::SuccessorRegistryOwner,
        identity:&str,version:&str,revision:u32,content:&str,digest:&str,
    )->Result<CheckedLogicModuleExport<'a>,String>{
        let profile=registry.restore_logic_recorded_exact(identity,version,revision,content,digest,&self.export.definitions)
            .map_err(|e|e.to_string())?;
        Ok(CheckedLogicModuleExport{ddl:self,profile})
    }
}
impl CheckedLogicModuleExport<'_>{
    pub(crate) fn owner(&self)->&[u8;32]{self.ddl.export.owner()}
    pub(crate) fn profile(&self)->&ddonirang_lang::ddl::RegisteredSuccessorProfile{&self.profile}
    pub(crate) fn resolve_profile(&self,registry:&ddonirang_lang::ddl::SuccessorRegistryOwner,name:&str,version:&str)
        ->Result<ddonirang_lang::ddl::RegisteredSuccessorProfile,String>{
        registry.resolve_exported(std::slice::from_ref(&self.profile),name,version).map_err(|e|e.to_string())
    }
    pub(crate) fn select_import_request<'a>(&'a self,registry:&ddonirang_lang::ddl::SuccessorRegistryOwner,
        source:&str,path:&str,syntax:ddonirang_lang::ddl::CheckedJ1Profile,
    )->Result<PendingLogicImport<'a,'_>,ddonirang_lang::ParseError>{
        let err=|span,message:&str|ddonirang_lang::ParseError{span,message:message.into()};
        // The checked fixed material declares library="logic". This is a
        // private provider selector, not an invented path/source module ID or
        // an alias with implicit authority. Public graph admission is separate.
        let context=ddonirang_lang::ModuleDirectiveContext::default().with_known_imports(["logic"]);
        let document=ddonirang_lang::module_directive::parse_module_directive_prefix_with_j1_profile(source,path,&context,syntax)?;
        let imports=document.imports.ok_or_else(||err(Span::new(0,0),"E_DDL_ACTIVATION_REQUIRED"))?;
        let request=imports.translation.ok_or_else(||err(imports.span,"E_DDL_ACTIVATION_REQUIRED"))?;
        if imports.entries.len()!=1||imports.entries[0].target!="logic"||document.exports.is_some(){
            return Err(err(imports.span,"E_LOGIC_IMPORT_MODULE_REQUIRED"));
        }
        self.resolve_profile(registry,&request.target_language,&request.target_version)
            .map_err(|_|err(request.span,"E_DDL_PROFILE_NAME_OR_VERSION"))?;
        let declaration_start=source.len().checked_sub(document.declaration_source.len())
            .filter(|start|source.get(*start..)==Some(document.declaration_source.as_str()))
            .ok_or_else(||err(imports.span,"E_LOGIC_IMPORT_SOURCE_RANGE"))?;
        Ok(PendingLogicImport{exported:self,source:source.into(),path:path.into(),alias:imports.entries[0].local_name.clone(),request_span:request.span,declaration_start})
    }
}
impl CheckedLogicDdlDefinition{
    pub(crate) fn identity(&self)->&[u8;32]{&self.identity}
    pub(crate) fn projection(&self)->&ddonirang_lang::SeedDef{&self.projection}
    pub(crate) fn pins(&self)->&[(ddonirang_lang::NodeId,[u8;32])]{&self.pins}
}
impl CheckedLogicExport {
    pub(crate) fn owner(&self)->&[u8;32]{&self.owner}
    pub(crate) fn version(&self)->u32{self.version}
    pub(crate) fn material_digest(&self)->&str{self.material_digest}
    pub(crate) fn check_ddl(&self,source:&str,path:&str,syntax:ddonirang_lang::ddl::CheckedJ1Profile)
        ->Result<CheckedLogicDdl<'_>,ddonirang_lang::ParseError>{
        use ddonirang_lang::{ParseError,TypeRef,SeedKind};
        let err=|span,message:&str|ParseError{span,message:message.into()};
        let declarations=ddonirang_lang::Parser::parse_j1_ddl_declarations(source,path,syntax)?;
        if declarations.len()!=self.definitions.len(){return Err(err(Span::new(0,source.len()),"E_LOGIC_DDL_PUBLIC_DENOMINATOR"));}
        let mut seen=std::collections::BTreeSet::new();let mut definitions=Vec::new();
        for projection in declarations {
            let row=self.resolve_surface(&self.owner,self.version,self.material_digest,&projection.canonical_name)
                .map_err(|_|err(projection.span,"E_LOGIC_DDL_EXPORTED_DEFINITION"))?;
            let identity=*row.descriptor().definition();
            if !seen.insert(identity){return Err(err(projection.span,"E_LOGIC_DDL_DUPLICATE_DEFINITION"));}
            if projection.seed_kind!=SeedKind::Semssi||projection.body.is_some()||!projection.alternate_forms.is_empty()
                ||!projection.type_params.is_empty()||projection.proof_header.is_some()||!projection.modifiers.is_empty()
                ||projection.result_type!=Some(TypeRef::Named("真偽".into())){
                return Err(err(projection.span,"E_LOGIC_DDL_DECLARATION_SCOPE"));
            }
            if projection.params.len()!=row.pins().len(){return Err(err(projection.span,"E_LOGIC_DDL_PIN_DENOMINATOR"));}
            let mut pinned=std::collections::BTreeSet::new();let mut pins=Vec::new();
            for pin in &projection.params {
                let found=row.pins().iter().filter(|p|p.surface()==pin.pin_name).collect::<Vec<_>>();
                if found.len()!=1{return Err(err(pin.span,"E_LOGIC_DDL_PIN_OWNER"));}
                let actual=found[0];
                // DDL projections preserve the registered surface TypeRef,
                // as the existing rational DDL path does for 有理数. Its
                // meaning is supplied by the actual checked Bool export,
                // not inferred from this spelling or a fabricated type ID.
                if !pinned.insert(*actual.identity())||pin.type_ref!=TypeRef::Named("真偽".into())
                    ||pin.optional||pin.default_value.is_some()||!pin.josa_list.is_empty(){
                    return Err(err(pin.span,"E_LOGIC_DDL_PIN_TYPE_OR_SCOPE"));
                }
                pins.push((pin.id,*actual.identity()));
            }
            definitions.push(CheckedLogicDdlDefinition{identity,projection,pins});
        }
        Ok(CheckedLogicDdl{export:self,source:source.into(),path:path.into(),definitions})
    }
    /// Qualified export lookup for a DDL producer. The caller must separately
    /// prove the source's explicit import/translation activation; this lookup
    /// never admits a token or executes a source call by surface spelling.
    pub(crate) fn resolve_surface(&self,owner:&[u8;32],version:u32,digest:&str,surface:&str)
        ->Result<CheckedLogicCallableExport<'_>,String>{
        if owner!=&self.owner||version!=self.version||digest!=self.material_digest{
            return Err(fail("E_LOGIC_EXPORT_EXACT_REQUIRED"));
        }
        let mut found=self.definitions().into_iter().filter(|row|row.descriptor.surface()==surface);
        let row=found.next().ok_or_else(||fail("E_LOGIC_EXPORTED_SURFACE_REQUIRED"))?;
        if found.next().is_some(){return Err(fail("E_LOGIC_EXPORTED_SURFACE_AMBIGUOUS"));}
        Ok(row)
    }
    /// Complete denominator and checked input/result types for DDL producers.
    /// Intrinsic resources are empty; argument effects remain governed by the
    /// descriptor's evaluation policy, never an unconditional whole-call PURE.
    pub(crate) fn definitions(&self)->Vec<CheckedLogicCallableExport<'_>>{
        use logic::LogicPinRole as P;
        self.definitions.iter().map(|descriptor|{
            let pins=[(P::Left,"왼쪽","左"),(P::Right,"오른쪽","右"),(P::Value,"값","値")]
                .into_iter().filter_map(|(role,original_name,surface)|descriptor.pin(role)
                    .map(|identity|CheckedLogicPinExport{identity,role,original_name,surface,type_proof:&self.bool_type})).collect();
            CheckedLogicCallableExport{descriptor,result_type_proof:&self.bool_type,pins}
        }).collect()
    }
    fn from_checked(
        definitions:Vec<logic::CheckedLogicDefinition>,bool_type:Vec<u8>,
        material_digest:&'static str,
    )->Result<Self,String>{
        use logic::{LogicEvaluationPolicy as E,LogicPinRole as P};
        let signatures=ddonirang_lang::stdlib::logic_function_sigs();
        if definitions.len()!=4||signatures.len()!=4||bool_type.is_empty(){return Err(fail("E_LOGIC_PUBLIC_DENOMINATOR"));}
        let owner=*definitions[0].owner();
        let mut ids=std::collections::BTreeSet::new();let mut pins=std::collections::BTreeSet::new();
        for definition in &definitions {
            if definition.owner()!=&owner||definition.version()!=1||!ids.insert(*definition.definition()) {
                return Err(fail("E_LOGIC_EXPORT_OWNER_OR_VERSION"));
            }
            let found=signatures.iter().filter(|s|s.name==definition.name()).collect::<Vec<_>>();
            if found.len()!=1||found[0].ret!="참거짓"{return Err(fail("E_LOGIC_BUILTIN_SIGNATURE_DRIFT"));}
            let expected=match definition.evaluation_policy(){
                E::AndShortCircuit|E::OrShortCircuit=>vec![(P::Left,"왼쪽"),(P::Right,"오른쪽")],
                E::UnaryEager=>vec![(P::Value,"값")],
            };
            if found[0].params!=expected.iter().map(|(_,name)|*name).collect::<Vec<_>>().as_slice(){
                return Err(fail("E_LOGIC_BUILTIN_SIGNATURE_DRIFT"));
            }
            for (role,_) in expected {
                let pin=definition.pin(role).ok_or_else(||fail("E_LOGIC_EXPORT_PIN_OWNER"))?;
                if !pins.insert(*pin){return Err(fail("E_LOGIC_EXPORT_PIN_OWNER"));}
            }
        }
        if pins.len()!=6||signatures.iter().any(|s|definitions.iter().filter(|d|d.name()==s.name).count()!=1){
            return Err(fail("E_LOGIC_PUBLIC_DENOMINATOR"));
        }
        Ok(Self{owner,version:1,material_digest,bool_type,definitions})
    }
    /// Exact immutable lookup; paths, aliases and consumer module IDs are not owners.
    fn resolve(&self,owner:&[u8;32],version:u32,digest:&str,definition:&[u8;32])
        ->Result<&logic::CheckedLogicDefinition,String>{
        if owner!=&self.owner||version!=self.version||digest!=self.material_digest{
            return Err(fail("E_LOGIC_EXPORT_EXACT_REQUIRED"));
        }
        self.definitions.iter().find(|d|d.definition()==definition).ok_or_else(||fail("E_LOGIC_DEFINITION_OWNER"))
    }
}

/// Private fields/no Deserialize: material strings alone cannot create this scope.
pub(crate) struct CheckedLogicSourceScope {
    owner:PreCanonAuthoredSourceV1,
    site:Span,
    module:[u8;32],
    library:std::sync::Arc<CheckedLogicExport>,
}
impl CheckedLogicSourceScope {
    pub(crate) fn site(&self)->Span{self.site}
    pub(crate) fn verify_source_generation(&self,program:&crate::ddn_runtime::DdnProgram)->Result<(),String>{
        let sources=program.sealed_project_sources().ok_or_else(||fail("E_LOGIC_SOURCE_OWNER_REQUIRED"))?;
        if !std::ptr::eq(sources,self.owner.sources().as_ref())||program.canonical_program().origin.source!=self.owner.sources().preparation().source{
            return Err(fail("E_LOGIC_SOURCE_OWNER_MISMATCH"));
        }
        Ok(())
    }
    pub(crate) fn builtin_export(&self)->&CheckedLogicExport{&self.library}
    pub(crate) fn definition(&self,id:&[u8;32])->Result<&logic::CheckedLogicDefinition,String>{
        self.library.resolve(&self.library.owner,self.library.version,self.library.material_digest,id)
    }
    pub(crate) fn verify_program(&self,program:&crate::ddn_runtime::DdnProgram,expr:&Expr)->Result<(),String>{
        let sources=program.sealed_project_sources().ok_or_else(||fail("E_LOGIC_SOURCE_OWNER_REQUIRED"))?;
        // A retained process capability, never pointer-derived stable identity.
        // Different projects/generations with identical bytes must not borrow it.
        if !std::ptr::eq(sources,self.owner.sources().as_ref()) {
            return Err(fail("E_LOGIC_SOURCE_OWNER_MISMATCH"));
        }
        let actual=program.canonical_program();
        if actual.origin.source!=self.owner.sources().preparation().source
            ||expr.span!=self.site
            ||!super::actual_expression_sites(actual).iter().any(|(id,span)|id==&expr.id&&span==&self.site) {
            return Err(fail("E_LOGIC_SOURCE_SITE_MISMATCH"));
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn module(&self)->&[u8;32]{&self.module}
    #[cfg(test)]
    pub(crate) fn bool_type(&self)->&[u8]{&self.library.bool_type}
}

pub(super) fn prepare(
    owned:&PreCanonAuthoredSourceV1,nominals:&super::super::RegistrationDraft,site:Span,
)->Result<CheckedLogicSourceScope,String>{
    prepare_sites(owned,nominals,&[(None,site)])?.pop().map(|(_,scope)|scope)
        .ok_or_else(||fail("E_LOGIC_ACTUAL_SOURCE_SITE_REQUIRED"))
}
pub(super) fn prepare_batch(
    owned:&PreCanonAuthoredSourceV1,nominals:&super::super::RegistrationDraft,
    sites:&[(ddonirang_lang::NodeId,Span)],
)->Result<std::collections::BTreeMap<ddonirang_lang::NodeId,CheckedLogicSourceScope>,String>{
    let requested=sites.iter().map(|(node,span)|(Some(*node),*span)).collect::<Vec<_>>();
    Ok(prepare_sites(owned,nominals,&requested)?.into_iter().collect())
}
/// One actual source/type owner per batch. Every node/span and lexical Bool
/// proof is still checked separately; shared export bytes do not grant sites.
fn prepare_sites(
    owned:&PreCanonAuthoredSourceV1,nominals:&super::super::RegistrationDraft,
    sites:&[(Option<ddonirang_lang::NodeId>,Span)],
)->Result<Vec<(ddonirang_lang::NodeId,CheckedLogicSourceScope)>,String>{
    let source=Source::saved(owned);
    let types=StaticTypeOwner::from_source(&source,nominals,&[])?;
    let actual=super::actual_expression_sites(types.checked_program()).into_iter()
        .collect::<std::collections::BTreeMap<_,_>>();
    let mut selected=std::collections::BTreeSet::new();
    let mut requests=Vec::with_capacity(sites.len());
    for (node,site) in sites {
        let node=match node {
            Some(node) if actual.get(node)==Some(site)=>*node,
            Some(_)=>return Err(fail("E_LOGIC_ACTUAL_SOURCE_SITE_REQUIRED")),
            None=>*actual.iter().find(|(_,span)|*span==site)
                .map(|(id,_)|id).ok_or_else(||fail("E_LOGIC_ACTUAL_SOURCE_SITE_REQUIRED"))?,
        };
        if !selected.insert(node){return Err(fail("E_LOGIC_SOURCE_SITE_DUPLICATE"));}
        requests.push((node,*site));
    }
    let reserved=reserved(&source,nominals)?.into_iter().collect::<Vec<_>>();
    let (raw,digest)=logic::material(1).ok_or_else(||fail("E_LOGIC_EXACT_MATERIAL_REQUIRED"))?;
    let value:serde_json::Value=serde_json::from_str(raw).map_err(|_|fail("E_LOGIC_MATERIAL_JSON"))?;
    let id=|value:&serde_json::Value|->Result<[u8;32],String>{
        hex::decode(value.as_str().ok_or_else(||fail("E_LOGIC_MATERIAL_ID"))?)
            .map_err(|_|fail("E_LOGIC_MATERIAL_ID"))?.try_into().map_err(|_|fail("E_LOGIC_MATERIAL_ID"))
    };
    let owner=id(&value["owner"])?;
    let rows=value["definitions"].as_array().ok_or_else(||fail("E_LOGIC_PUBLIC_DENOMINATOR"))?;
    if rows.len()!=4{return Err(fail("E_LOGIC_PUBLIC_DENOMINATOR"));}
    let definitions=rows.iter().map(|row|logic::lookup(&owner,&id(&row["definition"])?,1,digest,&reserved))
        .collect::<Result<Vec<_>,String>>()?;
    let mut libraries=std::collections::BTreeMap::<Vec<u8>,std::sync::Arc<CheckedLogicExport>>::new();
    let mut out=Vec::with_capacity(requests.len());
    for (node,site) in requests {
        let proof=types.fulfill_logic_type_requirement(site,logic::LogicTypeRequirement::ExistingCheckedBool)?.encode()?;
        let (path,_)=source.original_span(site)?;
        let module=source.module(&path)?.ok_or_else(||fail("E_LOGIC_SOURCE_MODULE_REQUIRED"))?;
        let library=match libraries.get(&proof){Some(library)=>library.clone(),None=>{
            let library=std::sync::Arc::new(CheckedLogicExport::from_checked(definitions.clone(),proof.clone(),digest)?);
            libraries.insert(proof,library.clone());library
        }};
        out.push((node,CheckedLogicSourceScope{owner:owned.clone(),site,module,library}));
    }
    Ok(out)
}

#[cfg(test)]
mod r31_logic_source_scope_tests {
    use super::*;
    use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
    use crate::ddn_runtime::DdnProgram;
    fn fixture()->(PreCanonAuthoredSourceV1,super::super::super::RegistrationDraft,DdnProgram){
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.".into()},
            SourceFile{path:"main.ddn".into(),text:"실행:움직씨 := { 참 보여주기. }.".into()}
        ]},String::new()).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(project.save().unwrap().as_bytes()).unwrap();
        let draft=super::super::super::prepare_precanon_under_candidate_owner(&owned,None,&mut |_|Err("unexpected identity issue".into())).unwrap();
        let program=DdnProgram::from_saved_v25_project_with_mode(owned.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        (owned,draft,program)
    }
    fn expression(program:&DdnProgram)->&Expr{
        let ddonirang_lang::TopLevelItem::SeedDef(seed)=&program.canonical_program().items[0];
        let ddonirang_lang::Stmt::Show{expr,..}=&seed.body.as_ref().unwrap().stmts[0] else{panic!("actual expression")};expr
    }
    #[test]fn r31_logic_source_scope_actual_owner_bool_and_full_library(){
        let (owned,draft,program)=fixture();let expr=expression(&program);let scope=prepare(&owned,&draft,expr.span).unwrap();
        scope.verify_program(&program,expr).unwrap();assert_eq!(scope.library.definitions.len(),4);assert!(!scope.bool_type().is_empty());
        let count=scope.library.definitions.iter().flat_map(|d|[logic::LogicPinRole::Left,logic::LogicPinRole::Right,logic::LogicPinRole::Value].into_iter().filter_map(|role|d.pin(role))).count();
        assert_eq!(count,6);assert!(scope.definition(&[0;32]).unwrap_err().contains("E_LOGIC_DEFINITION_OWNER"));
        assert!(prepare(&owned,&draft,Span::new(0,0)).is_err());
    }
    #[test]fn r31_logic_source_scope_same_bytes_foreign_owner_refused(){
        let (owned,draft,program)=fixture();let (other,_,other_program)=fixture();
        assert_eq!(owned.project().source_snapshot.files[1].text,other.project().source_snapshot.files[1].text);
        let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        assert!(scope.verify_program(&other_program,expression(&other_program)).unwrap_err().contains("E_LOGIC_SOURCE_OWNER_MISMATCH"));
        let mut wrong_site=expression(&program).clone();wrong_site.span.end+=1;
        assert!(scope.verify_program(&program,&wrong_site).unwrap_err().contains("E_LOGIC_SOURCE_SITE_MISMATCH"));
    }
    #[test]fn r31_logic_source_scope_same_project_edited_generation_refused(){
        let (owned,draft,program)=fixture();let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        let mut edited=owned.project().clone();edited.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text="실행:움직씨 := { 거짓 보여주기. }.".into();
        let fresh=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(edited.save().unwrap().as_bytes()).unwrap();
        assert_eq!(owned.project().authored_identity.logical_dependency_id,fresh.project().authored_identity.logical_dependency_id);
        let new_program=DdnProgram::from_saved_v25_project_with_mode(fresh.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        assert!(scope.verify_program(&new_program,expression(&new_program)).unwrap_err().contains("E_LOGIC_SOURCE_OWNER_MISMATCH"));
    }
    #[test]fn r31_logic_source_scope_actual_j1_owner_and_existing_bool(){
        let project=ProjectArtifact::create("v1-core-v25-ja-base-r1".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:"#!日本語\n(版=\"v25\", モジュール群=列[(名前=アプリ, 見出し=\"main.ddn\", 原文=\"main.ddn\")], 開始=(モジュール=アプリ, 定義=実行)) 構成.\n".into()},
            SourceFile{path:"main.ddn".into(),text:"#!日本語\n実行:手続き := { 真 表示. }.\n".into()}
        ]},String::new()).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(project.save().unwrap().as_bytes()).unwrap();
        let draft=super::super::super::prepare_precanon_under_candidate_owner(&owned,None,&mut |_|Err("unexpected identity issue".into())).unwrap();
        let program=DdnProgram::from_saved_v25_project_with_mode(owned.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        let expr=expression(&program);let scope=prepare(&owned,&draft,expr.span).unwrap();
        scope.verify_program(&program,expr).unwrap();assert_eq!(scope.library.definitions.len(),4);assert!(!scope.bool_type().is_empty());
    }
    #[test]fn r31_logic_export_exact_owner_version_and_complete_denominator(){
        let (owned,draft,program)=fixture();let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        let library=&scope.library;let first=&library.definitions[0];
        assert_ne!(library.owner,*scope.module()); // builtin owner, never the consumer's module
        assert!(library.resolve(&library.owner,1,library.material_digest,first.definition()).is_ok());
        for (owner,version,digest) in [(*scope.module(),1,library.material_digest),(library.owner,2,library.material_digest),(library.owner,1,"wrong")] {
            assert!(library.resolve(&owner,version,digest,first.definition()).unwrap_err().contains("E_LOGIC_EXPORT_EXACT_REQUIRED"));
        }
        let mut partial=library.definitions.clone();partial.pop();
        assert!(CheckedLogicExport::from_checked(partial,library.bool_type.clone(),library.material_digest).is_err());
        let mut duplicate=library.definitions.clone();duplicate[3]=duplicate[0].clone();
        assert!(CheckedLogicExport::from_checked(duplicate,library.bool_type.clone(),library.material_digest).is_err());
        assert!(CheckedLogicExport::from_checked(library.definitions.clone(),Vec::new(),library.material_digest).is_err());
        let rows=scope.builtin_export().definitions();
        assert_eq!(rows.len(),4);assert_eq!(rows.iter().map(|row|row.pins.len()).sum::<usize>(),6);
        for row in rows {
            assert_eq!(row.result_type_proof,scope.bool_type());
            for pin in row.pins {
                assert_eq!(row.descriptor.pin(pin.role),Some(pin.identity));
                assert_eq!(pin.type_proof,row.result_type_proof);
                assert!(matches!((pin.original_name,pin.surface),("왼쪽","左")|("오른쪽","右")|("값","値")));
            }
        }
    }
    #[test]fn r31_logic_export_surface_lookup_stays_with_exact_builtin_owner(){
        let (owned,draft,program)=fixture();let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        let export=scope.builtin_export();let mut ids=std::collections::BTreeSet::new();
        for surface in ["かつ","または","否定する","否定である"] {
            let row=export.resolve_surface(export.owner(),export.version(),export.material_digest(),surface).unwrap();
            assert!(ids.insert(*row.descriptor().definition()));assert_eq!(row.result_type_proof(),scope.bool_type());
            for pin in row.pins(){
                assert_eq!(row.descriptor().pin(pin.role()),Some(pin.identity()));
                assert_eq!(pin.type_proof(),row.result_type_proof());
                assert!(matches!((pin.original_name(),pin.surface()),("왼쪽","左")|("오른쪽","右")|("값","値")));
            }
        }
        for surface in ["그리고","かつ ","カツ","未知"] {
            assert!(export.resolve_surface(export.owner(),1,export.material_digest(),surface).is_err());
        }
        assert!(export.resolve_surface(scope.module(),1,export.material_digest(),"かつ").is_err());
    }
    const DDL:&str="#!日本語\n(右:真偽, 左:真偽) かつ:真偽:関数 := { ... }.\n(左:真偽, 右:真偽) または:真偽:関数 := { ... }.\n(値:真偽) 否定する:真偽:関数 := { ... }.\n(値:真偽) 否定である:真偽:関数 := { ... }.\n";
    #[test]fn r31_logic_ddl_actual_common_parser_binds_all_definitions_and_owned_pins(){
        let (owned,draft,program)=fixture();let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let ddl=scope.builtin_export().check_ddl(DDL,"論理.ddl",syntax).unwrap();
        assert_eq!(ddl.source(),DDL);assert_eq!(ddl.path(),"論理.ddl");assert_eq!(ddl.export().owner(),scope.builtin_export().owner());
        assert_eq!(ddl.definitions().len(),4);assert_eq!(ddl.definitions().iter().map(|d|d.pins().len()).sum::<usize>(),6);
        let mut defs=std::collections::BTreeSet::new();let mut nodes=std::collections::HashSet::new();
        for binding in ddl.definitions(){
            let descriptor=scope.definition(binding.identity()).unwrap();assert!(defs.insert(*binding.identity()));
            let projection=binding.projection();assert_eq!(projection.canonical_name,descriptor.surface());
            assert!(DDL.get(projection.span.start..projection.span.end).unwrap().contains(descriptor.surface()));
            for (node,pin) in binding.pins(){assert!(nodes.insert(*node));assert!(projection.params.iter().any(|p|p.id==*node));
                assert!(!descriptor.argument_slots(binding.identity(),&[*pin]).is_ok()||projection.params.len()==1);
            }
            let pins=binding.pins().iter().map(|(_,id)|*id).collect::<Vec<_>>();
            let slots=descriptor.argument_slots(binding.identity(),&pins).unwrap();
            if descriptor.surface()=="かつ"{assert_eq!(slots,vec![1,0]);}
        }
    }
    #[test]fn r31_logic_ddl_rejects_changed_denominator_pin_type_body_and_preserves_utf8_span(){
        let (owned,draft,program)=fixture();let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        let export=scope.builtin_export();let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let missing=DDL.lines().take(4).collect::<Vec<_>>().join("\n");
        let variants=[missing,DDL.replacen("右:真偽","値:真偽",1),DDL.replacen("左:真偽","左:有理数",1),DDL.replacen("{ ... }","{ 真 表示. }",1)];
        for text in variants{assert!(export.check_ddl(&text,"論理.ddl",syntax).is_err(),"{text}");}
        let text=DDL.replacen("右:真偽","右:真偽）",1);let bad=text.find('）').unwrap();
        let error=export.check_ddl(&text,"論理.ddl",syntax).err().unwrap();
        assert!(error.message.starts_with("E_MULTILANG_FULLWIDTH_SYNTAX"));assert_eq!(error.span,Span::new(bad,bad+'）'.len_utf8()));
        assert!(export.check_ddl(DDL.trim_start_matches("#!日本語\n"),"論理.ddl",syntax).is_err());
    }
    #[test]fn r31_logic_export_profile_reuses_registry_issuer_and_restores_without_reissue(){
        use ddonirang_lang::ddl::{SuccessorRegistryOwner,SuccessorCallableSurface,SuccessorPinSurface};
        let (owned,draft,program)=fixture();let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let mut registry=SuccessorRegistryOwner::default();
        let old=registry.issue(vec![SuccessorCallableSurface{identity:format!("{:064x}",1),surface:"F".into(),pins:vec![SuccessorPinSurface{
            identity:format!("{:064x}",2),surface:"金額".into(),particle:"に".into()}]}],&mut |bytes|{bytes.fill(7);Ok(())}).unwrap();
        let old_bytes=old.content().to_vec();let mut attempts=0;
        let export=scope.builtin_export().check_ddl(DDL,"論理.ddl",syntax).unwrap().issue_exported_profile(&mut registry,&mut |bytes|{
            attempts+=1;bytes.fill(if attempts==1{7}else{8});Ok(())
        }).unwrap();
        assert_eq!(attempts,2);assert_eq!(export.owner(),scope.builtin_export().owner());
        let profile=export.profile();assert!(profile.is_logic_builtin());assert_eq!(profile.version(),"1");assert_ne!(profile.identity(),old.identity());
        assert!(ddonirang_lang::parser::RegisteredPublicCallableContext::from_public_and_ddl(profile.clone(),vec![],DDL,"論理.ddl",1)
            .unwrap_err().message.contains("E_DDL_PUBLIC_PROFILE_KIND"));
        assert_eq!(old.version(),"2");assert_eq!(old.content(),old_bytes);
        let value:serde_json::Value=serde_json::from_slice(profile.content()).unwrap();
        let old_value:serde_json::Value=serde_json::from_slice(old.content()).unwrap();
        assert_eq!(value.as_object().unwrap().keys().collect::<Vec<_>>(),old_value.as_object().unwrap().keys().collect::<Vec<_>>());
        assert_eq!(value["type"],"真偽");assert_eq!(profile.surfaces().len(),4);
        assert!(profile.surfaces().iter().flat_map(|d|&d.pins).all(|p|p.particle.is_empty()));
        assert!(export.resolve_profile(&registry,"日本語","1").is_ok());
        assert!(export.resolve_profile(&registry,"日本語","latest").is_err());
        assert!(export.resolve_profile(&registry,"ja","1").is_err());
        let again=scope.builtin_export().check_ddl(DDL,"論理.ddl",syntax).unwrap().issue_exported_profile(&mut registry,&mut |_|panic!("same content must not issue again")).unwrap();
        assert_eq!(again.profile().identity(),profile.identity());
        let text=std::str::from_utf8(profile.content()).unwrap();let mut fresh=SuccessorRegistryOwner::default();
        let restored=scope.builtin_export().check_ddl(DDL,"論理.ddl",syntax).unwrap()
            .restore_exported_profile(&mut fresh,profile.identity(),profile.version(),profile.registration_revision(),text,profile.content_sha256()).unwrap();
        assert_eq!(restored.profile().identity(),profile.identity());assert_eq!(restored.profile().content(),profile.content());
        assert!(fresh.restore_recorded_exact(profile.identity(),"1",1,text,profile.content_sha256()).is_err());
        assert!(registry.lookup(old.identity(),"2",old.content_sha256(),1).is_ok());
    }
    #[test]fn r31_logic_export_profile_rejects_partial_material_and_changed_record(){
        use ddonirang_lang::ddl::SuccessorRegistryOwner;
        let (owned,draft,program)=fixture();let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        let library=scope.builtin_export();let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let mut registry=SuccessorRegistryOwner::default();
        assert!(registry.issue_logic(&library.definitions[..3],&mut |_|panic!("invalid material cannot issue")).is_err());
        let export=library.check_ddl(DDL,"論理.ddl",syntax).unwrap().issue_exported_profile(&mut registry,&mut |bytes|{bytes.fill(8);Ok(())}).unwrap();
        let p=export.profile();let text=std::str::from_utf8(p.content()).unwrap();
        let mut fresh=SuccessorRegistryOwner::default();
        assert!(fresh.restore_logic_recorded_exact(p.identity(),"2",1,text,p.content_sha256(),&library.definitions).is_err());
        assert!(fresh.restore_logic_recorded_exact(p.identity(),"1",1,&text.replace("真偽","有理数"),p.content_sha256(),&library.definitions).is_err());
        assert!(fresh.restore_logic_recorded_exact(p.identity(),"1",1,&text.replace("\"particle\":\"\"","\"particle\":\"に\""),p.content_sha256(),&library.definitions).is_err());
        assert!(fresh.restore_logic_recorded_exact(p.identity(),"1",1,text,"wrong",&library.definitions).is_err());
    }
    #[test]fn r31_logic_export_selects_only_actual_explicit_module_profile_request(){
        let (owned,draft,program)=fixture();let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let mut registry=ddonirang_lang::ddl::SuccessorRegistryOwner::default();
        let export=scope.builtin_export().check_ddl(DDL,"論理.ddl",syntax).unwrap().issue_exported_profile(&mut registry,&mut |bytes|{bytes.fill(8);Ok(())}).unwrap();
        let prefix="(論理=logic, 翻訳=(対象言語=\"日本語\", 対象版=\"1\")) 使用.";
        let body="実行:手続き := { 真 表示. }.\r\n";
        let source=format!("#!日本語\r\n// 현재 prefix 원문\r\n{prefix}\r\n{body}");
        let selected=export.select_import_request(&registry,&source,"利用.ddn",syntax).unwrap();
        assert_eq!(selected.alias(),"論理");assert_eq!(selected.owner(),scope.builtin_export().owner());
        assert_eq!(selected.profile().identity(),export.profile().identity());
        assert_eq!(&source[selected.declaration_start()..],body);
        assert_eq!(&source[selected.request_span().start..selected.request_span().end],"(対象言語=\"日本語\", 対象版=\"1\")");
        selected.verify_source(&source,"利用.ddn").unwrap();
        assert!(selected.verify_source(&source.replace("真 表示","偽 表示"),"利用.ddn").is_err());
        assert!(selected.verify_source(&source,"別.ddn").is_err());
        for bad in [source.replace(prefix,""),source.replace(", 翻訳=(対象言語=\"日本語\", 対象版=\"1\")",""),source.replace("対象版=\"1\"","対象版=\"latest\""),source.replace("日本語\", 対象版","ja\", 対象版"),source.replace(prefix,&format!("{prefix}\r\n{prefix}"))]{
            assert!(export.select_import_request(&registry,&bad,"利用.ddn",syntax).is_err(),"{bad}");
        }
        // A profile in a different registry is not an exported registered result.
        assert!(export.select_import_request(&ddonirang_lang::ddl::SuccessorRegistryOwner::default(),&source,"利用.ddn",syntax).is_err());
    }
    const LOGIC_CALLS:&str="#!日本語\n(論理=logic, 翻訳=(対象言語=\"日本語\", 対象版=\"1\")) 使用.\n実行:手続き := {\n ((右=偽, 左=真) 論理.かつ) 表示.\n ((左=真, 右=偽) 論理.または) 表示.\n ((値=偽) 論理.否定する) 表示.\n ((値=真) 論理.否定である) 表示.\n}.\n";
    #[test]fn r31_logic_common_source_binds_actual_qualified_definitions_and_pins_without_fake_bodies(){
        let (owned,draft,program)=fixture();let scope=prepare(&owned,&draft,expression(&program).span).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let mut registry=ddonirang_lang::ddl::SuccessorRegistryOwner::default();
        let export=scope.builtin_export().check_ddl(DDL,"論理.ddl",syntax).unwrap().issue_exported_profile(&mut registry,&mut |bytes|{bytes.fill(8);Ok(())}).unwrap();
        let selected=export.select_import_request(&registry,LOGIC_CALLS,"利用.ddn",syntax).unwrap();
        let compiled=selected.parse_program(syntax).unwrap();assert_eq!(compiled.origin.source,LOGIC_CALLS);assert_eq!(compiled.items.len(),1);
        let ddonirang_lang::TopLevelItem::SeedDef(seed)=&compiled.items[0];assert_eq!(seed.canonical_name,"実行");
        let mut ids=std::collections::BTreeSet::new();
        for stmt in &seed.body.as_ref().unwrap().stmts{
            let ddonirang_lang::Stmt::Show{expr,..}=stmt else{panic!("actual Show")};
            let ddonirang_lang::ExprKind::Call{func,args}=&expr.kind else{panic!("actual qualified Call")};
            let binding=compiled.resolved_logic_call(expr).unwrap().unwrap();
            assert_eq!(func,&format!("論理.{}",binding.descriptor().surface()));assert!(ids.insert(*binding.descriptor().definition()));
            assert_eq!(&LOGIC_CALLS[binding.name_span().start..binding.name_span().end],func);
            assert_eq!(binding.descriptor().owner(),scope.builtin_export().owner());
            assert_eq!(binding.profile().identity(),export.profile().identity());
            assert!(args.iter().all(|a|matches!(a.expr.kind,ddonirang_lang::ExprKind::Literal(ddonirang_lang::Literal::Bool(_)))));
            let slots=binding.descriptor().argument_slots(binding.descriptor().definition(),binding.pin_identities()).unwrap();
            if binding.descriptor().surface()=="かつ"{assert_eq!(slots,vec![1,0]);}
            let mut changed=expr.clone();changed.span.end+=1;assert!(compiled.resolved_logic_call(&changed).is_err());
            let mut renamed=expr.clone();let ddonirang_lang::ExprKind::Call{args,..}=&mut renamed.kind else{unreachable!()};
            args[0].resolved_pin=Some("other".into());assert!(compiled.resolved_logic_call(&renamed).is_err());
        }
        assert_eq!(ids.len(),4);
        let mut canonical=compiled.clone();ddonirang_lang::canonicalize(&mut canonical).unwrap();
        let ddonirang_lang::TopLevelItem::SeedDef(seed)=&canonical.items[0];
        for stmt in &seed.body.as_ref().unwrap().stmts{
            let ddonirang_lang::Stmt::Show{expr,..}=stmt else{panic!("actual Show after common canon")};
            assert!(canonical.resolved_logic_call(expr).unwrap().is_some());
        }
        assert_eq!(canonical.origin.source,LOGIC_CALLS);
        for bad in [LOGIC_CALLS.replacen("右=偽, 左=真","左=真",1),LOGIC_CALLS.replacen("右=偽, 左=真","右=偽, 右=真",1),LOGIC_CALLS.replacen("論理.かつ","論理.未知",1),LOGIC_CALLS.replacen("右=偽, 左=真","偽, 真",1)]{
            assert!(export.select_import_request(&registry,&bad,"利用.ddn",syntax).and_then(|selected|selected.parse_program(syntax)).is_err(),"{bad}");
        }
        // Reuse the existing admitted parameter syntax, rather than inventing
        // a typed local declaration spelling for this counterexample.
        let shadowed=LOGIC_CALLS.replacen("実行:手続き := {","(論理:真偽) 実行:手続き := {",1);
        let error=export.select_import_request(&registry,&shadowed,"利用.ddn",syntax).and_then(|selected|selected.parse_program(syntax)).unwrap_err();
        assert!(error.message.contains("E_LOGIC_ALIAS_SHADOWED"),"{error}");
    }
    fn input()->ddonirang_core::InputSnapshot{ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:ddonirang_core::InputSource::Person,rng_seed:0}}
    fn actual_logic_source(source:&str,context:std::sync::Arc<ddonirang_lang::parser::RegisteredLogicCallableContext>)->Result<PreCanonAuthoredSourceV1,String>{
        actual_logic_source_named(source,context,"アプリ")
    }
    fn actual_logic_source_named(source:&str,context:std::sync::Arc<ddonirang_lang::parser::RegisteredLogicCallableContext>,module_name:&str)->Result<PreCanonAuthoredSourceV1,String>{
        let project=ProjectArtifact::create(ddonirang_lang::ddl::J1_SOURCE_PROFILE.into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:format!("#!日本語\n(版=\"v25\", モジュール群=列[(名前={module_name}, 見出し=\"main.ddn\", 原文=\"main.ddn\")], 開始=(モジュール={module_name}, 定義=実行)) 構成.\n")},
            SourceFile{path:"main.ddn".into(),text:source.into()},
        ]},String::new())?;
        crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic(project.save()?.as_bytes(),context)
    }
    fn logic_compiler_context()->std::sync::Arc<ddonirang_lang::parser::RegisteredLogicCallableContext>{
        // Lexical requirements precede the actual consumer source owner. No
        // dummy project supplies Bool authority; runtime admission below must
        // fulfill it from that actual project and its captures.
        let (raw,digest)=logic::material(1).unwrap();let value:serde_json::Value=serde_json::from_str(raw).unwrap();
        let id=|s:&str|->[u8;32]{hex::decode(s).unwrap().try_into().unwrap()};
        let owner=id(value["owner"].as_str().unwrap());
        let definitions=value["definitions"].as_array().unwrap().iter().map(|row|
            logic::lookup(&owner,&id(row["definition"].as_str().unwrap()),1,digest,&[]).unwrap()).collect::<Vec<_>>();
        let mut registry=ddonirang_lang::ddl::SuccessorRegistryOwner::default();
        let profile=registry.issue_logic(&definitions,&mut |bytes|{bytes.fill(8);Ok(())}).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        std::sync::Arc::new(ddonirang_lang::parser::RegisteredLogicCallableContext::from_exported_ddl(profile,definitions,DDL,"論理.ddl",syntax).unwrap()
            .with_module_activation(ddonirang_lang::ModuleDirectiveEntry{local_name:"論理".into(),target:"logic".into()}).unwrap())
    }
    fn run_actual_logic(source:&str)->Result<Vec<String>,String>{
        let actual=actual_logic_source(source,logic_compiler_context())?;
        let mut program=DdnProgram::from_saved_v25_project_with_mode(actual.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target())?;
        super::super::super::admit_logic_source_program(&actual,&mut program,&mut |_|Err("unexpected nominal issue".into()))?;
        let result=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new())
            .map(|result|result.display_outputs.iter().map(|row|row.text.clone()).collect::<Vec<_>>()).map_err(|e|e.to_string());
        log_actual_case(&actual,source,&result);result
    }
    fn log_actual_case(actual:&PreCanonAuthoredSourceV1,source:&str,outcome:&Result<Vec<String>,String>){
        let context=actual.sources().logic_callable_context().unwrap();
        println!("R31_B1_EXECUTION_RAW {}",serde_json::json!({
            "kind":"PRIVATE_ACTUAL_PROJECT_RUNNER_NOT_CLI_OR_NATIVE","source":source,"source_path":"main.ddn",
            "source_sha256":hex::encode(<sha2::Sha256 as sha2::Digest>::digest(source.as_bytes())),
            "actual_project":actual.project().authored_identity.logical_dependency_id.as_str(),"actual_consumer_module":actual.entry_module().id.as_str(),
            "builtin_owner":hex::encode(context.builtin_owner()),"profile_identity":context.profile().identity(),"profile_version":context.profile().version(),
            "captures":actual.sources().captures().iter().map(|(path,capture)|serde_json::json!({"path":path,"identity":hex::encode(capture.identity())})).collect::<Vec<_>>(),
            "outcome":match outcome{Ok(rows)=>serde_json::json!({"ok":true,"display_outputs":rows}),Err(error)=>serde_json::json!({"ok":false,"error":error})}
        }));
    }
    #[test]fn r31_logic_actual_project_capture_bool_admission_and_runner(){
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();
        assert_eq!(actual.sources().captures().iter().find(|(path,_)|path=="main.ddn").unwrap().1.text(),LOGIC_CALLS);
        let mut executable=DdnProgram::from_saved_v25_project_with_mode(actual.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        let run=|program:DdnProgram|crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new());
        let denied=run(executable.clone()).err().expect("no source/Bool admission");assert!(denied.to_string().contains("E_LOGIC_CALL_ADMISSION_REQUIRED"),"{denied}");
        super::super::super::admit_logic_source_program(&actual,&mut executable,&mut |_|Err("unexpected nominal issue".into())).unwrap();
        let result=run(executable).unwrap();
        let outputs=result.display_outputs.iter().map(|row|row.text.clone()).collect::<Vec<_>>();
        assert_eq!(outputs,vec!["거짓","참","참","거짓"]);
        // Preserve the runner's existing output/log resources. Only argument
        // and module-alias leakage is forbidden here, not all normal output.
        let leaked=["左","右","値","論理","logic"].into_iter().filter(|name|result.resources.contains_key(*name)).collect::<Vec<_>>();
        println!("R31_B1_RESOURCE_KEYS {}",serde_json::json!({"existing_resource_count":result.resources.len(),"forbidden_names_present":leaked}));
        assert!(leaked.is_empty(),"unexpected B1 names: {leaked:?}");
        log_actual_case(&actual,LOGIC_CALLS,&Ok(outputs));
        let storage_error=crate::project_revision_envelope_supply::admit_saved_authored_project(actual.sources().clone(),actual.project().save().unwrap().as_bytes()).err().unwrap();
        assert!(storage_error.contains("E_LOGIC_STORAGE_CONTEXT_REQUIRED"));
        println!("R31_B1_BOUNDARY_RAW {}",serde_json::json!({"without_admission":denied.to_string(),"unrecorded_storage":storage_error}));
    }
    #[test]fn r31_logic_actual_project_lazy_rhs_and_wrong_type(){
        let source=LOGIC_CALLS.replacen("実行:手続き", "위험하다:真偽:関数 := { 위험하다 <- 1 / 0 > 0. }.\n実行:手続き",1)
            .replacen("右=偽, 左=真","右=(() 위험하다), 左=偽",1)
            .replacen("左=真, 右=偽","左=真, 右=(() 위험하다)",1);
        assert_eq!(run_actual_logic(&source).unwrap(),vec!["거짓","참","참","거짓"]);
        let needed=source.replacen("左=偽","左=真",1);let failure=run_actual_logic(&needed).unwrap_err();
        assert!(failure.contains("0으로 나눌 수 없습니다"),"{failure}");
        let wrong=LOGIC_CALLS.replacen("右=偽","右=1",1);let failure=run_actual_logic(&wrong).unwrap_err();
        assert!(failure.contains("E_RUNTIME_CONDITION_TYPE"),"{failure}");
        let korean="실행:움직씨 := { 참 그리고 1 보여주기. }.";
        let baseline=DdnProgram::from_v25_source(korean,"ko.ddn").unwrap();
        let baseline_error=crate::ddn_runtime::DdnRunner::new(baseline,"실행").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).err().unwrap().to_string();
        assert!(baseline_error.contains("E_RUNTIME_CONDITION_TYPE"),"{baseline_error}");
        println!("R31_B1_KO_BASELINE_RAW {}",serde_json::json!({"source":korean,"phase":"run_update","error":baseline_error}));
        let effectful=source.replacen("위험하다:真偽:関数 := { 위험하다 <- 1 / 0 > 0. }.","위험하다:真偽:手続き := { 真 表示. 위험하다 <- 真. }.",1);
        assert_eq!(run_actual_logic(&effectful).unwrap(),vec!["거짓","참","참","거짓"]);
        let needed=effectful.replacen("左=偽","左=真",1);
        assert_eq!(run_actual_logic(&needed).unwrap(),vec!["참","참","참","참","거짓"]);
    }
    #[test]fn r31_logic_actual_project_rejects_foreign_and_stale_source_owners(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let other=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let mut program=DdnProgram::from_saved_v25_project_with_mode(actual.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        assert!(super::super::super::admit_logic_source_program(&other,&mut program,&mut |_|panic!("foreign source cannot reach issuer")).unwrap_err().contains("E_LOGIC_SOURCE_OWNER_MISMATCH"));
        let mut edited=actual.project().clone();edited.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=LOGIC_CALLS.replacen("右=偽","右=真",1);
        let fresh=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic(edited.save().unwrap().as_bytes(),context).unwrap();
        assert_eq!(actual.project().authored_identity.logical_dependency_id,fresh.project().authored_identity.logical_dependency_id);
        let mut fresh_program=DdnProgram::from_saved_v25_project_with_mode(fresh.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        assert!(super::super::super::admit_logic_source_program(&actual,&mut fresh_program,&mut |_|panic!("stale source cannot reach issuer")).unwrap_err().contains("E_LOGIC_SOURCE_OWNER_MISMATCH"));
        assert!(crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(actual.project().save().unwrap().as_bytes()).is_err());
    }
    #[test]fn r31_logic_actual_project_rejects_same_spelling_user_module(){
        let error=actual_logic_source_named(LOGIC_CALLS,logic_compiler_context(),"logic").err().unwrap();
        assert!(error.contains("E_LOGIC_PROVIDER_COLLISION"),"{error}");
        println!("R31_B1_PROVIDER_REFUSAL_RAW {}",serde_json::json!({"actual_user_module_name":"logic","error":error}));
    }
    fn restore_logic_context(old:&ddonirang_lang::parser::RegisteredLogicCallableContext)
        ->std::sync::Arc<ddonirang_lang::parser::RegisteredLogicCallableContext>{
        let (raw,digest)=logic::material(1).unwrap();let material:serde_json::Value=serde_json::from_str(raw).unwrap();
        let id=|s:&str|->[u8;32]{hex::decode(s).unwrap().try_into().unwrap()};
        let owner=id(material["owner"].as_str().unwrap());
        let definitions=material["definitions"].as_array().unwrap().iter().map(|row|
            logic::lookup(&owner,&id(row["definition"].as_str().unwrap()),1,digest,&[]).unwrap()).collect::<Vec<_>>();
        let mut registry=ddonirang_lang::ddl::SuccessorRegistryOwner::default();let profile=old.profile();
        let restored=registry.restore_logic_recorded_exact(profile.identity(),profile.version(),profile.registration_revision(),
            std::str::from_utf8(profile.content()).unwrap(),profile.content_sha256(),&definitions).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        std::sync::Arc::new(ddonirang_lang::parser::RegisteredLogicCallableContext::from_exported_ddl(
            restored,definitions,old.ddl_source(),old.ddl_path(),syntax).unwrap()
            .with_module_activation(old.activation().unwrap().clone()).unwrap())
    }
    fn replay_logic_captures(project:&ProjectArtifact,catalog:&str,context:std::sync::Arc<ddonirang_lang::parser::RegisteredLogicCallableContext>)
        ->Result<PreCanonAuthoredSourceV1,String>{
        crate::project_revision_envelope_supply::restore_precanon_authored_source_with_logic(
            project.save()?.as_bytes(),catalog,context)
    }
    #[test]fn r31_logic_recorded_capture_replay_and_registration_rebind_never_issue(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let draft=super::super::super::prepare_precanon_under_candidate_owner(&actual,None,&mut |_|panic!("no nominal issue expected")).unwrap();
        let records=draft.recorded_blobs();let recorded_json=serde_json::to_string(&records).unwrap();
        let catalog=actual.sources().recorded_capture_catalog_json().unwrap();
        let restored_context=restore_logic_context(&context);
        assert!(!std::sync::Arc::ptr_eq(&context,&restored_context));
        let restored=replay_logic_captures(actual.project(),&catalog,restored_context.clone()).unwrap();
        assert!(!std::sync::Arc::ptr_eq(actual.sources(),restored.sources()));
        assert_eq!(catalog,restored.sources().recorded_capture_catalog_json().unwrap());
        assert_eq!(actual.project().save().unwrap(),restored.project().save().unwrap());
        assert_eq!(context.profile().identity(),restored_context.profile().identity());
        assert_eq!(context.profile().content(),restored_context.profile().content());
        let retained=super::super::super::restore_precanon_recorded_candidate(&restored,records).unwrap();
        assert_eq!(recorded_json,serde_json::to_string(&retained.recorded_blobs()).unwrap());
        let mut program=DdnProgram::from_saved_v25_project_with_mode(restored.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        let calls=program.canonical_program().logic_call_sites();assert_eq!(calls.len(),4);
        super::super::super::bind_recorded_logic_source_program(&restored,&mut program,&retained).unwrap();
        // Replaying and binding above only prepares the program; this is the
        // first explicit runner operation and the only producer of output.
        let result=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).unwrap();
        let outputs=result.display_outputs.iter().map(|row|row.text.clone()).collect::<Vec<_>>();
        assert_eq!(outputs,vec!["거짓","참","참","거짓"]);
        println!("R31_B1_REPLAY_RAW {}",serde_json::json!({"kind":"PRIVATE_TYPED_REPLAY_NOT_NATIVE_OR_COLD",
            "catalog_equal":true,"registration_equal":true,"profile_identity":restored_context.profile().identity(),
            "project_identity":restored.project().authored_identity.logical_dependency_id.as_str(),
            "captures":restored.sources().captures().iter().map(|(path,capture)|serde_json::json!({"path":path,"identity":hex::encode(capture.identity())})).collect::<Vec<_>>(),
            "explicit_run_output":outputs,"entropy_argument_on_restore":false}));
    }
    #[test]fn r31_logic_recorded_capture_replay_rejects_stale_missing_and_foreign(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let catalog=actual.sources().recorded_capture_catalog_json().unwrap();
        let mut missing:serde_json::Value=serde_json::from_str(&catalog).unwrap();missing["records"].as_array_mut().unwrap().pop();
        assert!(replay_logic_captures(actual.project(),&missing.to_string(),restore_logic_context(&context)).is_err());
        let mut edited=actual.project().clone();edited.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=LOGIC_CALLS.replacen("右=偽","右=真",1);
        assert!(replay_logic_captures(&edited,&catalog,restore_logic_context(&context)).is_err());
        let other=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let foreign=super::super::super::prepare_precanon_under_candidate_owner(&other,None,&mut |_|panic!("no nominal issue expected")).unwrap();
        let restored=replay_logic_captures(actual.project(),&catalog,restore_logic_context(&context)).unwrap();
        let mut program=DdnProgram::from_saved_v25_project_with_mode(restored.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        assert!(super::super::super::bind_recorded_logic_source_program(&restored,&mut program,&foreign).is_err());
        let mut wrong:serde_json::Value=serde_json::from_str(&catalog).unwrap();wrong["records"][0]["snapshot_identity_hex"]=serde_json::Value::String("00".repeat(32));
        assert!(replay_logic_captures(actual.project(),&wrong.to_string(),restore_logic_context(&context)).is_err());
    }

    #[test]fn r31_logic_qualified_builtin_does_not_reserve_unrelated_user_spellings(){
        for source in [
            LOGIC_CALLS.replacen("実行:手続き", "아님:真偽:関数 := { 아님 <- 偽. }.\n実行:手続き",1),
            LOGIC_CALLS.replacen("実行:手続き := {", "実行:手続き := { 아님 := 偽.",1),
            LOGIC_CALLS.replacen("実行:手続き", "否定する:真偽:関数 := { 否定する <- 偽. }.\n実行:手続き",1),
        ] {
            assert_eq!(run_actual_logic(&source).unwrap(),vec!["거짓","참","참","거짓"]);
        }
    }
    #[test]fn r31_logic_same_japanese_spelling_selects_user_and_builtin_by_owner(){
        let source=LOGIC_CALLS.replacen("実行:手続き", "否定する:真偽:関数 := { 否定する <- 偽. }.\n実行:手続き",1)
            .replacen("実行:手続き := {", "実行:手続き := { (() 否定する) 表示.",1);
        assert_eq!(run_actual_logic(&source).unwrap(),vec!["거짓","거짓","참","참","거짓"]);
    }

    #[test]fn r31_logic_batch_reuses_checked_export_but_keeps_all_actual_sites(){
        let source=format!("#!日本語\n(論理=logic, 翻訳=(対象言語=\"日本語\", 対象版=\"1\")) 使用.\n実行:手続き := {{\n{}\n}}.\n",
            "((右=偽, 左=真) 論理.かつ) 表示.\n".repeat(64));
        let actual=actual_logic_source(&source,logic_compiler_context()).unwrap();
        let draft=super::super::super::prepare_precanon_under_candidate_owner(&actual,None,&mut |_|panic!("no nominal issue expected")).unwrap();
        let mut program=DdnProgram::from_saved_v25_project_with_mode(actual.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        let sites=program.canonical_program().logic_call_sites();assert_eq!(sites.len(),64);
        let scopes=prepare_batch(&actual,&draft,&sites).unwrap();assert_eq!(scopes.len(),64);
        let first=scopes.values().next().unwrap();
        assert!(scopes.values().all(|scope|std::sync::Arc::ptr_eq(&first.library,&scope.library)));
        for (node,span) in &sites{assert_eq!(scopes[node].site(),*span);scopes[node].verify_source_generation(&program).unwrap();}
        program.bind_logic_source_scopes(scopes).unwrap();
        let result=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).unwrap();
        assert_eq!(result.display_outputs.iter().map(|row|row.text.clone()).collect::<Vec<_>>(),vec!["거짓";64]);
        println!("R31_B1_BATCH_RAW {}",serde_json::json!({"kind":"PRIVATE_ACTUAL_SOURCE_BATCH_NOT_NATIVE","source_sha256":hex::encode(<sha2::Sha256 as sha2::Digest>::digest(source.as_bytes())),"call_count":64,"shared_exact_export":true,"explicit_output_count":64}));
    }
    #[test]fn r31_logic_batch_refuses_duplicate_forged_sites_and_partial_runtime_attachment(){
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();
        let draft=super::super::super::prepare_precanon_under_candidate_owner(&actual,None,&mut |_|panic!("no nominal issue expected")).unwrap();
        let mut program=DdnProgram::from_saved_v25_project_with_mode(actual.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        let sites=program.canonical_program().logic_call_sites();
        let duplicate=[sites[0],sites[0]];
        assert!(prepare_batch(&actual,&draft,&duplicate).err().unwrap().contains("E_LOGIC_SOURCE_SITE_DUPLICATE"));
        let wrong=[(sites[0].0,Span::new(0,1))];
        assert!(prepare_batch(&actual,&draft,&wrong).err().unwrap().contains("E_LOGIC_ACTUAL_SOURCE_SITE_REQUIRED"));
        let scopes=prepare_batch(&actual,&draft,&sites[..3]).unwrap();
        assert!(program.bind_logic_source_scopes(scopes).unwrap_err().contains("E_LOGIC_SOURCE_SCOPE_DENOMINATOR"));
    }

    #[test]fn r31_logic_preparation_preserves_same_failure_code_and_utf8_span(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        for (text,code,needle) in [
            (LOGIC_CALLS.replacen("論理.かつ","論理.unknown",1),"E_LOGIC_CALL_UNREGISTERED","論理.unknown"),
            (LOGIC_CALLS.replacen("右=偽, 左=真","左=真",1),"E_LOGIC_CALL_ARITY","論理.かつ"),
            // The common parser now supplies the actual explicit label token.
            (LOGIC_CALLS.replacen("右=偽","extra=偽",1),"E_LOGIC_CALL_PIN_OWNER","extra"),
        ]{
            let mut source=actual.sources().saved().source().clone();
            source.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().utf8_text=text.clone();
            let error=crate::local_project::prepare_logic_callable_authored_execution(source,syntax,context.clone()).unwrap_err();
            assert_eq!(error.code,code);assert_eq!(error.path.as_deref(),Some("main.ddn"));assert_eq!(error.phase,Some("lang_parse"));
            let (start,end)=error.source_span.unwrap();let slice=text.get(start..end).unwrap();
            assert_eq!(slice,needle,"{code}: actual source span");
            let json=error.to_json_value();assert_eq!(json["code"],code);assert_eq!(json["severity"],"fatal");
            assert_eq!(json["source_span"]["start"],start);assert_eq!(json["source_span"]["end"],end);
            println!("R31_B1_TYPED_DIAGNOSTIC_RAW {}",serde_json::json!({"kind":"PRIVATE_ACTUAL_PREPARATION_FAILURE_NOT_PUBLIC_CLI",
                "source_sha256":hex::encode(<sha2::Sha256 as sha2::Digest>::digest(text.as_bytes())),"diagnostic":json,"exact_slice":slice}));
        }
    }
    #[test]fn r31_logic_preparation_fullwidth_failure_keeps_exact_bad_character(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let text=LOGIC_CALLS.replacen("表示.","表示。",1);let at=text.find('。').unwrap();
        let mut source=actual.sources().saved().source().clone();source.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().utf8_text=text.clone();
        let error=crate::local_project::prepare_logic_callable_authored_execution(source,syntax,context).unwrap_err();
        assert_eq!(error.code,"E_MULTILANG_FULLWIDTH_SYNTAX");assert_eq!(error.source_span,Some((at,at+3)));
    }
    #[test]fn r31_logic_preparation_distinguishes_unregistered_j1_word_from_call_binding(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let text=LOGIC_CALLS.replacen("論理.かつ","論理.未知",1);let at=text.find("未知").unwrap();
        let mut source=actual.sources().saved().source().clone();source.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().utf8_text=text;
        let error=crate::local_project::prepare_logic_callable_authored_execution(source,syntax,context).unwrap_err();
        assert_eq!(error.code,"E_J1_UNREGISTERED_SURFACE");assert_eq!(error.source_span,Some((at,at+"未知".len())));assert_eq!(error.phase,Some("lang_lex"));
    }

    #[test]fn r31_logic_pin_error_points_at_exact_unicode_label_not_comment_or_value(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let text=LOGIC_CALLS.replacen("#!日本語\n","#!日本語\r\n// ((値 = 偽, 左=真) 論理.かつ) is a comment\r\n",1)
            .replacen("右=偽","値 = 偽",1);
        let at=text.rfind("値 = 偽").unwrap();
        let mut source=actual.sources().saved().source().clone();source.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().utf8_text=text.clone();
        let error=crate::local_project::prepare_logic_callable_authored_execution(source,syntax,context).unwrap_err();
        assert_eq!(error.code,"E_LOGIC_CALL_PIN_OWNER");assert_eq!(error.source_span,Some((at,at+3)));
        println!("R31_B1_PIN_LABEL_RAW {}",serde_json::json!({"kind":"PRIVATE_ACTUAL_SOURCE_FAILURE_NOT_PUBLIC_CLI",
            "source":text,"diagnostic":error.to_json_value(),"exact_label":"値"}));
    }
    #[test]fn r31_logic_duplicate_pin_error_points_at_second_authored_label(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).unwrap();
        let text=LOGIC_CALLS.replacen("右=偽","左=偽",1);let at=text.find("左=真").unwrap();
        let mut source=actual.sources().saved().source().clone();source.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().utf8_text=text.clone();
        let error=crate::local_project::prepare_logic_callable_authored_execution(source,syntax,context).unwrap_err();
        assert_eq!(error.code,"E_LOGIC_CALL_PIN_OWNER");assert_eq!(error.source_span,Some((at,at+3)));
    }

    #[test]fn r31_logic_bootstrap_retains_same_failure_pin_span_without_reanalysis(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let mut previous=None;
        for prefix in ["", "// 日本語の原文位置\r\n"]{
            let text=LOGIC_CALLS.replacen("#!日本語\n",&format!("#!日本語\n{prefix}"),1)
                .replacen("右=偽","extra=偽",1);
            let at=text.find("extra=偽").unwrap();
            let mut project=actual.project().clone();project.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=text.clone();
            let encoded=project.save().unwrap();
            let error=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic_diagnostic(encoded.as_bytes(),context.clone()).err().unwrap();
            assert_eq!(error.code,"E_LOGIC_CALL_PIN_OWNER");assert_eq!(error.source_path,"main.ddn");assert_eq!(error.phase,"lang_parse");assert_eq!(error.span,Some((at,at+5)));
            if let Some(old)=previous {assert_ne!(error.span,old,"edited generation cannot use an old failure span");}
            previous=Some(error.span);
            println!("R31_B1_BOOTSTRAP_DIAGNOSTIC_RAW {}",serde_json::json!({"kind":"PRIVATE_SAME_BOOTSTRAP_FAILURE_NOT_PUBLIC_CLI",
                "source":text,"source_sha256":hex::encode(<sha2::Sha256 as sha2::Digest>::digest(text.as_bytes())),
                "code":error.code,"path":error.source_path,"span":error.span,"phase":error.phase}));
        }
    }
    #[test]fn r31_logic_bootstrap_typed_success_and_untyped_input_failure_keep_boundaries(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();let encoded=actual.project().save().unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic_diagnostic(encoded.as_bytes(),context.clone()).unwrap();
        assert_eq!(owned.project().authored_identity.logical_dependency_id,actual.project().authored_identity.logical_dependency_id);
        let mut program=DdnProgram::from_saved_v25_project_with_mode(owned.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        super::super::super::admit_logic_source_program(&owned,&mut program,&mut |_|panic!("no nominal issue expected")).unwrap();
        let result=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).unwrap();
        assert_eq!(result.display_outputs.iter().map(|row|row.text.clone()).collect::<Vec<_>>(),vec!["거짓","참","참","거짓"]);
        let error=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic_diagnostic(b"{}",context).err().unwrap();
        assert_eq!(error.phase,"source_preparation");assert!(error.span.is_none());assert!(error.source_path.is_empty());
        // A generic earlier failure is not relabelled as a specific parser code.
        assert_eq!(error.code,"E_PARSE");
    }

    #[test]fn r31_logic_recorded_bootstrap_preserves_actual_failure_and_catalog_boundary(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let catalog=actual.sources().recorded_capture_catalog_json().unwrap();let artifact=actual.project().save().unwrap();
        let restored=crate::project_revision_envelope_supply::restore_precanon_authored_source_with_logic_diagnostic(
            artifact.as_bytes(),&catalog,restore_logic_context(&context)).unwrap();
        assert_eq!(restored.sources().recorded_capture_catalog_json().unwrap(),catalog);
        assert_eq!(restored.project().authored_identity.logical_dependency_id,actual.project().authored_identity.logical_dependency_id);
        let text=LOGIC_CALLS.replacen("右=偽","extra=偽",1);let at=text.find("extra=偽").unwrap();
        let mut edited=actual.project().clone();edited.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=text;
        let error=crate::project_revision_envelope_supply::restore_precanon_authored_source_with_logic_diagnostic(
            edited.save().unwrap().as_bytes(),&catalog,restore_logic_context(&context)).err().unwrap();
        // Preserve the established order: source preparation fails before
        // capture readback. Do not rerun a different analysis after that error.
        assert_eq!(error.code,"E_LOGIC_CALL_PIN_OWNER");assert_eq!(error.span,Some((at,at+5)));assert_eq!(error.source_path,"main.ddn");assert_eq!(error.phase,"lang_parse");
        let early=crate::project_revision_envelope_supply::restore_precanon_authored_source_with_logic_diagnostic(
            artifact.as_bytes(),"{}",restore_logic_context(&context)).err().unwrap();
        assert_eq!(early.phase,"source_restore");assert!(early.span.is_none());assert!(early.source_path.is_empty());
    }

    #[test]fn r31_logic_bootstrap_keeps_initial_artifact_lexer_and_root_diagnostics(){
        let context=logic_compiler_context();let actual=actual_logic_source(LOGIC_CALLS,context.clone()).unwrap();
        let catalog=actual.sources().recorded_capture_catalog_json().unwrap();
        for (path,from,to,bad,phase) in [("main.ddn","表示.","表示。",'。',"lang_lex"),("root.ddn","(版=","（版=",'（',"project_root")] {
            let mut edited=actual.project().clone();
            let file=edited.source_snapshot.files.iter_mut().find(|f|f.path==path).unwrap();file.text=file.text.replacen(from,to,1);
            let text=file.text.clone();let at=text.find(bad).unwrap();
            // Counterexample only: same real project owner and canonical outer
            // checksum, but an invalid authored source must still be refused.
            edited.content_sha256=edited.checksum().unwrap();let raw=serde_json::to_vec(&edited).unwrap();
            let fresh=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic_diagnostic(&raw,context.clone()).err().unwrap();
            let replay=crate::project_revision_envelope_supply::restore_precanon_authored_source_with_logic_diagnostic(&raw,&catalog,restore_logic_context(&context)).err().unwrap();
            for error in [fresh,replay]{
                assert_eq!(error.code,"E_MULTILANG_FULLWIDTH_SYNTAX");assert_eq!(error.source_path,path);assert_eq!(error.span,Some((at,at+3)));assert_eq!(error.phase,phase);
            }
        }
    }

    #[test]fn r31_logic_dependency_record_roundtrip_replays_actual_source_without_issue(){
        use crate::authored_project::registration::logic_dependency::CheckedLogicDependency as Dependency;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();
        let record=Dependency::from_source(&actual,"main.ddn").unwrap();
        let bytes=record.bytes().unwrap();let row=record.into_blob().unwrap();
        let read=Dependency::from_blob(&row).unwrap();assert_eq!(read.bytes().unwrap(),bytes);
        let context=read.restore_context(actual.project(),&[]).unwrap();
        assert_eq!(context.profile().identity(),actual.sources().logic_callable_context().unwrap().profile().identity());
        let catalog=actual.sources().recorded_capture_catalog_json().unwrap();
        let replay=replay_logic_captures(actual.project(),&catalog,context).unwrap();
        assert_eq!(replay.sources().recorded_capture_catalog_json().unwrap(),catalog);
        let mut program=DdnProgram::from_saved_v25_project_with_mode(replay.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        let draft=super::super::super::prepare_precanon_under_candidate_owner(&replay,None,&mut |_|panic!("no nominal issue")).unwrap();
        super::super::super::bind_recorded_logic_source_program(&replay,&mut program,&draft).unwrap();
        let result=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).unwrap();
        assert_eq!(result.display_outputs.iter().map(|row|row.text.as_str()).collect::<Vec<_>>(),vec!["거짓","참","참","거짓"]);
        println!("R31_B1_DEPENDENCY_REPLAY_RAW {}",serde_json::json!({"record_bytes":bytes.len(),"profile":replay.sources().logic_callable_context().unwrap().profile().identity(),"capture_bytes_equal":true,"result":[false,true,true,false],"public_admission":false}));
    }
    #[test]fn r31_logic_dependency_rejects_fields_duplicates_null_nested_and_limit(){
        use crate::authored_project::registration::logic_dependency::CheckedLogicDependency as Dependency;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();let raw=Dependency::from_source(&actual,"main.ddn").unwrap().bytes().unwrap();
        let value:serde_json::Value=serde_json::from_slice(&raw).unwrap();
        for key in ["schema","builtin_owner","builtin_version","builtin_material_sha256","profile","ddl_source","consumer_project_id","consumer_module_id","consumer_source_path"]{
            let mut bad=value.clone();bad.as_object_mut().unwrap().remove(key);assert!(Dependency::read(&serde_json::to_vec(&bad).unwrap()).is_err(),"missing {key}");
            let mut bad=value.clone();bad[key]=serde_json::Value::Null;assert!(Dependency::read(&serde_json::to_vec(&bad).unwrap()).is_err(),"null {key}");
        }
        let mut bad=value.clone();bad["trusted"]=true.into();assert!(Dependency::read(&serde_json::to_vec(&bad).unwrap()).is_err());
        let duplicate=format!("{{\"schema\":\"ddn.logic_callable_dependency.r1\",{}",std::str::from_utf8(&raw).unwrap().trim_start_matches('{'));
        assert!(Dependency::read(duplicate.as_bytes()).is_err());
        let mut bad=value.clone();bad["profile"]["schema"]="nested artifact".into();assert!(Dependency::read(&serde_json::to_vec(&bad).unwrap()).is_err());
        assert!(Dependency::read(&vec![b' ';131073]).is_err());
        let mut deep=serde_json::json!(0);for _ in 0..34{deep=serde_json::json!([deep]);}assert!(Dependency::read(&serde_json::to_vec(&deep).unwrap()).is_err());
    }
    #[test]fn r31_logic_dependency_rechecks_material_profile_ddl_and_actual_owner(){
        use crate::authored_project::registration::logic_dependency::CheckedLogicDependency as Dependency;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();let raw=Dependency::from_source(&actual,"main.ddn").unwrap().bytes().unwrap();
        let value:serde_json::Value=serde_json::from_slice(&raw).unwrap();
        for (key,bad_value) in [("builtin_owner",serde_json::json!("01".repeat(32))),("builtin_version",serde_json::json!(2)),("builtin_material_sha256",serde_json::json!("00".repeat(32))),("consumer_project_id",serde_json::json!("01".repeat(32))),("consumer_module_id",serde_json::json!("01".repeat(32))),("consumer_source_path",serde_json::json!("missing.ddn"))]{
            let mut bad=value.clone();bad[key]=bad_value;assert!(Dependency::read(&serde_json::to_vec(&bad).unwrap()).unwrap().restore_context(actual.project(),&[]).is_err(),"{key}");
        }
        for key in ["identity","version","content_utf8","content_sha256"]{
            let mut bad=value.clone();bad["profile"][key]="wrong".into();assert!(Dependency::read(&serde_json::to_vec(&bad).unwrap()).unwrap().restore_context(actual.project(),&[]).is_err(),"{key}");
        }
        let mut bad=value.clone();bad["ddl_source"]["text"]=DDL.replace("否定である","欠落").into();assert!(Dependency::read(&serde_json::to_vec(&bad).unwrap()).unwrap().restore_context(actual.project(),&[]).is_err());
        let owner=actual.sources().logic_callable_context().unwrap().builtin_owner();assert!(Dependency::read(&raw).unwrap().restore_context(actual.project(),&[*owner]).is_err());
    }
    #[test]fn r31_logic_dependency_record_never_activates_missing_or_changed_source(){
        use crate::authored_project::registration::logic_dependency::CheckedLogicDependency as Dependency;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();let record=Dependency::from_source(&actual,"main.ddn").unwrap();
        for source in [LOGIC_CALLS.replace(", 翻訳=(対象言語=\"日本語\", 対象版=\"1\")",""),LOGIC_CALLS.replace("対象版=\"1\"","対象版=\"2\""),LOGIC_CALLS.replace("論理=logic","論理=unregistered")]{
            let mut changed=actual.project().clone();changed.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=source;
            assert!(record.restore_context(&changed,&[]).is_err());
        }
    }
    #[test]fn r31_logic_dependency_blob_integrity_path_role_and_size_are_closed(){
        use crate::authored_project::registration::logic_dependency::CheckedLogicDependency as Dependency;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();let row=Dependency::from_source(&actual,"main.ddn").unwrap().into_blob().unwrap();
        let value=serde_json::to_value(row).unwrap();
        for (key,bad_value) in [("role",serde_json::json!("unknown")),("logical_path",serde_json::json!("other.json")),("byte_length",serde_json::json!(131073)),("sha256",serde_json::json!("00".repeat(32))),("raw_hex",serde_json::json!("GG"))]{
            let mut bad=value.clone();bad[key]=bad_value;let row=serde_json::from_value(bad).unwrap();assert!(Dependency::from_blob(&row).is_err(),"{key}");
        }
    }

    #[test]fn r31_logic_dependency_v2_private_payload_restores_registration_start_and_explicit_run(){
        use crate::unit_wire::project_runtime_attachment as wire;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();
        let mut draft=super::super::super::prepare_precanon_under_candidate_owner(&actual,None,&mut |_|panic!("no nominal issue")).unwrap();
        draft.attach_logic_dependency(&actual,"main.ddn").unwrap();
        let records=serde_json::to_vec(&draft.recorded_blobs()).unwrap();let captures=actual.sources().recorded_capture_catalog_json().unwrap();
        let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(actual,draft).unwrap();
        let start=owned.definition_identity().as_bytes().to_vec();
        assert!(wire::saved_runtime_project_payload_json(&owned).unwrap_err().contains("E_LOGIC_PHYSICAL_REVIEW_REQUIRED"));
        let raw=wire::saved_logic_payload_private(&owned).unwrap();let payload:serde_json::Value=serde_json::from_str(&raw).unwrap();
        assert_eq!(payload["schema"],"ddn.saved_runtime_project.v2");assert_eq!(payload.as_object().unwrap().len(),4);
        drop(owned);
        let restored=wire::restore_logic_payload_private(&raw).unwrap();
        assert_eq!(restored.definition_identity().as_bytes(),start.as_slice());
        assert_eq!(restored.sources().recorded_capture_catalog_json().unwrap(),captures);
        assert_eq!(serde_json::to_vec(&restored.registration().unwrap().recorded_blobs()).unwrap(),records);
        assert_eq!(wire::saved_logic_payload_private(&restored).unwrap(),raw);
        let source=restored.precanon_view();
        let mut program=DdnProgram::from_saved_v25_project_with_mode(source.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        super::super::super::bind_recorded_logic_source_program(&source,&mut program,restored.registration().unwrap()).unwrap();
        let outputs=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).unwrap().display_outputs;
        assert_eq!(outputs.iter().map(|o|o.text.as_str()).collect::<Vec<_>>(),["거짓","참","참","거짓"]);
        println!("R31_B1_DEPENDENCY_V2_PRIVATE_RAW {}",serde_json::json!({"registration_byte_equal":true,"capture_byte_equal":true,"start_identity_equal":true,"payload_byte_equal":true,"public_gate":"E_LOGIC_PHYSICAL_REVIEW_REQUIRED","db_cold":false}));
    }
    #[test]fn r31_logic_dependency_v2_private_rejects_missing_duplicate_and_mixed_carriers(){
        use crate::unit_wire::project_runtime_attachment as wire;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();
        let mut draft=super::super::super::prepare_precanon_under_candidate_owner(&actual,None,&mut |_|panic!("no nominal issue")).unwrap();
        draft.attach_logic_dependency(&actual,"main.ddn").unwrap();assert!(draft.attach_logic_dependency(&actual,"main.ddn").is_err());
        let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(actual,draft).unwrap();
        let raw=wire::saved_logic_payload_private(&owned).unwrap();let payload:serde_json::Value=serde_json::from_str(&raw).unwrap();
        // Preserve the existing payload field order, so negative rows reach
        // the actual carrier/owner checks rather than failing only JSON order.
        let encode=|p:serde_json::Value|{
            let rows:Vec<crate::authored_project::registration::RecordedRegistrationBlob>=serde_json::from_value(p["registration_resources"].clone()).unwrap();
            let resources=serde_json::to_string(&rows).unwrap();
            let mut raw=format!("{{\"schema\":{},\"authored_project_json_utf8\":{},\"source_capture_catalog_json_utf8\":{},\"registration_resources\":{}",p["schema"],p["authored_project_json_utf8"],p["source_capture_catalog_json_utf8"],resources);
            if let Some(first)=p.get("first_callable_original_registration"){raw.push_str(&format!(",\"first_callable_original_registration\":{first}"));}
            raw.push('}');raw
        };
        for schema in ["ddn.saved_runtime_project.v1","ddn.saved_runtime_project.v3","ddn.saved_runtime_project.v4"]{
            let mut p=payload.clone();p["schema"]=schema.into();assert!(wire::restore_logic_payload_private(&encode(p)).is_err());
        }
        let mut p=payload.clone();p["first_callable_original_registration"]=serde_json::json!({});assert!(wire::restore_logic_payload_private(&encode(p)).is_err());
        let mut p=payload.clone();p["registration_resources"]=serde_json::json!([]);assert_eq!(wire::restore_logic_payload_private(&encode(p)).err().unwrap(),"E_LOGIC_STORAGE_CONTEXT_REQUIRED");
        let mut p=payload.clone();let rows=p["registration_resources"].as_array_mut().unwrap();let dep=rows.iter().find(|r|r["role"]=="logic-callable-dependency-r1").unwrap().clone();rows.push(dep);assert_eq!(wire::restore_logic_payload_private(&encode(p)).err().unwrap(),"E_LOGIC_STORAGE_CONTEXT_REQUIRED");
    }

    #[test]fn r31_logic_dependency_production_profile_restores_fixed_identity_without_issuer(){
        use crate::authored_project::registration::logic_dependency::CheckedLogicDependency as Dependency;
        let original=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();
        let record=Dependency::for_project(original.project(),"main.ddn",DDL,"論理.ddl").unwrap();record.verify_production_profile().unwrap();
        let context=record.restore_context(original.project(),&[]).unwrap();
        assert_eq!(context.profile().identity(),"language-profile:d4465855dc978c9b1e6efdbc94832abc");
        assert_ne!(context.profile().identity(),original.sources().logic_callable_context().unwrap().profile().identity());
        let actual=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic(original.project().save().unwrap().as_bytes(),context).unwrap();
        let recorded=Dependency::from_source(&actual,"main.ddn").unwrap();assert_eq!(recorded.bytes().unwrap(),record.bytes().unwrap());
        assert!(Dependency::from_source(&original,"main.ddn").unwrap().verify_production_profile().is_err());
        let mut draft=super::super::super::prepare_precanon_under_candidate_owner(&actual,None,&mut |_|panic!("no nominal issue")).unwrap();draft.attach_logic_dependency(&actual,"main.ddn").unwrap();
        let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(actual,draft).unwrap();
        let payload=crate::unit_wire::project_runtime_attachment::saved_logic_payload_private(&owned).unwrap();
        let reopened=crate::unit_wire::project_runtime_attachment::restore_logic_payload_private(&payload).unwrap();
        assert_eq!(reopened.sources().logic_callable_context().unwrap().profile().identity(),"language-profile:d4465855dc978c9b1e6efdbc94832abc");
        println!("R31_B1_PRODUCTION_PROFILE_PRIVATE_RAW {}",serde_json::json!({"profile":reopened.sources().logic_callable_context().unwrap().profile().identity(),"fixture08":false,"restored_without_issue":true,"public_admission":false,"native_db":false}));
    }

    #[cfg(windows)]
    #[test]
    #[ignore="explicit child of r31_logic_native_private_separate_process_cold"]
    fn r31_logic_native_private_process_probe(){
        use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as Source;
        use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
        let base=std::path::PathBuf::from(std::env::var_os("DDN_B1_NATIVE_PROBE_DIR").unwrap());
        let root=base.join("store");let artifact=base.join("source-artifact.json");let mode=std::env::var("DDN_B1_NATIVE_PROBE_MODE").unwrap();
        if mode=="save"{
            use crate::authored_project::registration::logic_dependency::CheckedLogicDependency as Dependency;
            let initial=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();
            let record=Dependency::for_project(initial.project(),"main.ddn",DDL,"論理.ddl").unwrap();
            let context=record.restore_context(initial.project(),&[]).unwrap();
            let source=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic(initial.project().save().unwrap().as_bytes(),context).unwrap();
            let mut draft=super::super::super::prepare_precanon_under_candidate_owner(&source,None,&mut |_|panic!("no nominal issue")).unwrap();draft.attach_logic_dependency(&source,"main.ddn").unwrap();
            let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(source,draft).unwrap();
            assert!(Source::prepare(&owned,&[],b"b1-native-r1").is_err(),"public path stays gated");
            assert!(Source::prepare_logic_private(&owned,b"parent",b"b1-native-r1").is_err(),"updates excluded");
            let prepared=Source::prepare_logic_private(&owned,&[],b"b1-native-r1").unwrap();
            let expected=prepared.artifact().unwrap();let mut adapter=SqliteAdapterV1::provision_source_revision(&root).unwrap();
            let saved=adapter.save_initial_logic_source_revision_private(&prepared).unwrap_or_else(|_|panic!("private initial save failed"));
            assert_eq!(saved.source().artifact().unwrap(),expected);drop(saved);drop(adapter);
            std::fs::write(&artifact,&expected).unwrap();
            let original=serde_json::json!({"profile":owned.sources().logic_callable_context().unwrap().profile().identity(),"start_hex":hex::encode(owned.definition_identity().as_bytes()),"capture_catalog":owned.sources().recorded_capture_catalog_json().unwrap(),"registration":serde_json::to_value(owned.registration().unwrap().recorded_blobs()).unwrap(),"pesi":hex::encode(prepared.pesi())});
            std::fs::write(base.join("original-owner.json"),serde_json::to_vec(&original).unwrap()).unwrap();
            println!("R31_B1_NATIVE_PROCESS_RAW {}",serde_json::json!({"mode":mode,"pid":std::process::id(),"saved_artifact_bytes":expected.len(),"profile":original["profile"],"public_cli":false}));return;
        }
        let expected=std::fs::read_to_string(&artifact).unwrap();let selected=Source::from_artifact_logic_private(&expected).unwrap();
        assert!(Source::from_artifact(&expected).is_err(),"public reader does not admit private B1");
        let mut adapter=SqliteAdapterV1::open_source_revision(&root).unwrap();
        if mode=="duplicate"{
            assert!(adapter.save_initial_logic_source_revision_private(&selected).is_err(),"reopen never recreates fresh authority");
            println!("R31_B1_NATIVE_PROCESS_RAW {}",serde_json::json!({"mode":mode,"pid":std::process::id(),"duplicate_rejected":true}));return;
        }
        assert!(adapter.load_current_source_revision(selected.project(),selected.revision(),selected.integrity()).is_err(),"ordinary decode stays closed");
        assert!(adapter.load_current_logic_source_revision_private(&[1;32],selected.revision(),selected.integrity()).is_err(),"wrong project rejects");
        let read=adapter.load_current_logic_source_revision_private(selected.project(),selected.revision(),selected.integrity()).unwrap();
        assert_eq!(read.source().artifact().unwrap(),expected,"actual DB payload rather than selected memory artifact");
        let owned=read.source().recorded_execution_owner().unwrap();
        let original:serde_json::Value=serde_json::from_slice(&std::fs::read(base.join("original-owner.json")).unwrap()).unwrap();
        assert_eq!(original["profile"],owned.sources().logic_callable_context().unwrap().profile().identity());
        assert_eq!(original["start_hex"],hex::encode(owned.definition_identity().as_bytes()));
        assert_eq!(original["capture_catalog"],owned.sources().recorded_capture_catalog_json().unwrap());
        assert_eq!(original["registration"],serde_json::to_value(owned.registration().unwrap().recorded_blobs()).unwrap());
        assert_eq!(original["pesi"],hex::encode(read.source().pesi()));
        let mut outputs=Vec::<String>::new();let mut resolved_bindings=Vec::new();
        if mode=="run"{
            let source=owned.precanon_view();
            let mut program=DdnProgram::from_saved_v25_project_with_mode(source.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
            super::super::super::bind_recorded_logic_source_program(&source,&mut program,owned.registration().unwrap()).unwrap();
            let canonical=program.canonical_program();let mut definitions=std::collections::BTreeSet::new();let mut pins=std::collections::BTreeSet::new();
            for ddonirang_lang::TopLevelItem::SeedDef(seed) in &canonical.items{if let Some(body)=&seed.body{for statement in &body.stmts{
                if let ddonirang_lang::Stmt::Show{expr,..}=statement{
                    let binding=canonical.resolved_logic_call(expr).unwrap().unwrap();
                    definitions.insert(*binding.descriptor().definition());pins.extend(binding.pin_identities().iter().copied());
                    resolved_bindings.push(serde_json::json!({"owner":hex::encode(binding.descriptor().owner()),"definition":hex::encode(binding.descriptor().definition()),"pins":binding.pin_identities().iter().map(hex::encode).collect::<Vec<_>>(),"profile":binding.profile().identity(),"profile_version":binding.profile().version()}));
                }
            }}}
            assert_eq!(definitions.len(),4);assert_eq!(pins.len(),6);
            outputs=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).unwrap().display_outputs.into_iter().map(|o|o.text).collect();
            assert_eq!(outputs,["거짓","참","참","거짓"]);
        }else{assert_eq!(mode,"open");}
        println!("R31_B1_NATIVE_PROCESS_RAW {}",serde_json::json!({"mode":mode,"pid":std::process::id(),"profile":original["profile"],"actual_db_read":true,"owner_capture_registration_start_pesi_equal":true,"explicit_run":mode=="run","outputs":outputs,"resolved_bindings":resolved_bindings,"public_cli":false}));
    }
    #[cfg(windows)]
    #[test]fn r31_logic_native_private_separate_process_cold(){
        use sha2::{Digest,Sha256};
        let base=if let Some(base)=std::env::var_os("DDN_B1_NATIVE_EVIDENCE"){
            let base=std::path::PathBuf::from(base);std::fs::create_dir(&base).unwrap();base
        }else{
            let mut random=[0;16];getrandom::fill(&mut random).unwrap();let base=std::env::temp_dir().join(format!("ddn-b1-native-{}",hex::encode(random)));std::fs::create_dir(&base).unwrap();base
        };
        std::fs::create_dir(base.join("store")).unwrap();
        let probe="authored_project::registration::resolved_supply::logic_source::r31_logic_source_scope_tests::r31_logic_native_private_process_probe";
        let db=base.join("store/vc04_snapshot_v2.sqlite3");let mut saved_hash=None;let mut pids=std::collections::BTreeSet::new();
        for mode in ["save","open","run","duplicate"]{
            let child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact",probe,"--ignored","--nocapture"])
                .env("DDN_B1_NATIVE_PROBE_DIR",&base).env("DDN_B1_NATIVE_PROBE_MODE",mode).output().unwrap();
            std::fs::write(base.join(format!("{mode}.stdout")),&child.stdout).unwrap();std::fs::write(base.join(format!("{mode}.stderr")),&child.stderr).unwrap();
            assert!(child.status.success(),"mode={mode}\n{}\n{}",String::from_utf8_lossy(&child.stdout),String::from_utf8_lossy(&child.stderr));
            let stdout=String::from_utf8(child.stdout).unwrap();let raw=stdout.lines().find_map(|line|line.strip_prefix("R31_B1_NATIVE_PROCESS_RAW ")).unwrap();let result:serde_json::Value=serde_json::from_str(raw).unwrap();assert!(pids.insert(result["pid"].as_u64().unwrap()));
            let hash=hex::encode(Sha256::digest(std::fs::read(&db).unwrap()));
            if let Some(expected)=&saved_hash{assert_eq!(&hash,expected,"DB bytes changed after {mode}");}else{saved_hash=Some(hash.clone());}
            println!("R31_B1_NATIVE_COLD_RAW {}",serde_json::json!({"mode":mode,"exit":child.status.code(),"result":result,"db_sha256":hash,"normal_process_exit":true}));
        }
        // Preserve the successful DB before an intentional owned-fixture
        // corruption. Refusal must not write even the corrupted DB back.
        let good=std::fs::read(&db).unwrap();std::fs::write(base.join("normal-before-corruption.sqlite3"),&good).unwrap();
        let connection=rusqlite::Connection::open(&db).unwrap();
        let mut payload:Vec<u8>=connection.query_row("SELECT project_payload_bytes FROM vc04_prepared_payloads",[],|row|row.get(0)).unwrap();
        let at=payload.iter().position(|b|*b==b'd').unwrap();payload[at]=b'x';
        connection.execute("UPDATE vc04_prepared_payloads SET project_payload_bytes=?1",[payload]).unwrap();drop(connection);
        let corrupted=std::fs::read(&db).unwrap();
        let raw=std::fs::read_to_string(base.join("source-artifact.json")).unwrap();
        let selected=crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision::from_artifact_logic_private(&raw).unwrap();
        let adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&base.join("store")).unwrap();
        assert!(adapter.load_current_logic_source_revision_private(selected.project(),selected.revision(),selected.integrity()).is_err());drop(adapter);
        assert_eq!(std::fs::read(&db).unwrap(),corrupted,"refusal must preserve existing DB");
        println!("R31_B1_NATIVE_CORRUPTION_RAW {}",serde_json::json!({"actual_db_payload_corruption_rejected":true,"refusal_db_bytes_equal":true,"successful_db_backup":base.join("normal-before-corruption.sqlite3"),"public_cli":false}));
    }

    #[test]fn r31_logic_native_private_preflight_never_evaluates_user_arguments(){
        use crate::authored_project::registration::logic_dependency::CheckedLogicDependency as Dependency;
        use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as Source;
        let text=LOGIC_CALLS.replacen("実行:手続き", "위험하다:真偽:関数 := { 위험하다 <- 1 / 0 > 0. }.\n実行:手続き",1)
            .replacen("右=偽, 左=真","右=偽, 左=(() 위험하다)",1);
        let initial=actual_logic_source(&text,logic_compiler_context()).unwrap();let record=Dependency::for_project(initial.project(),"main.ddn",DDL,"論理.ddl").unwrap();
        let actual=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic(initial.project().save().unwrap().as_bytes(),record.restore_context(initial.project(),&[]).unwrap()).unwrap();
        let mut draft=super::super::super::prepare_precanon_under_candidate_owner(&actual,None,&mut |_|panic!("no nominal issue")).unwrap();draft.attach_logic_dependency(&actual,"main.ddn").unwrap();
        let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(actual,draft).unwrap();
        let artifact=Source::prepare_logic_private(&owned,&[],b"preflight-no-run").unwrap().artifact().unwrap();
        let read=Source::from_artifact_logic_private(&artifact).unwrap();assert_eq!(read.artifact().unwrap(),artifact);
        let owned=read.recorded_execution_owner().unwrap();let source=owned.precanon_view();
        let mut program=DdnProgram::from_saved_v25_project_with_mode(source.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        super::super::super::bind_recorded_logic_source_program(&source,&mut program,owned.registration().unwrap()).unwrap();
        let error=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).err().unwrap().to_string();
        assert!(error.contains("0으로 나눌 수 없습니다"),"{error}");
        println!("R31_B1_NATIVE_PREFLIGHT_NO_RUN_RAW {}",serde_json::json!({"prepare_and_restore_succeeded":true,"explicit_run_error":error,"source_bytes_preserved":true,"public_cli":false}));
    }

    #[test]
    fn r31_logic_non_entry_module_exports_only_its_own_callable() {
        use crate::authored_project::{ProjectArtifact, SourceFile, SourceSnapshot};
        use crate::authored_project::registration::logic_dependency::CheckedLogicDependency;
        use ddonirang_lang::{Parser, ddl::{CheckedJ1Profile, J1_SOURCE_PROFILE}};
        let root = "#!日本語
(版=\"v25\", モジュール群=列[(名前=アプリ, 見出し=\"main.ddn\", 原文=\"main.ddn\"), (名前=Logic, 見出し=\"Logic.ddn\", 原文=\"Logic.ddn\")], 開始=(モジュール=アプリ, 定義=実行)) 構成.
";
        let body = "#!日本語
(論理=logic, 翻訳=(対象言語=\"日本語\", 対象版=\"1\")) 使用.
(wrapper=flagValue) 公開.
flagValue:真偽:関数 := { flagValue <- ((右=偽, 左=真) 論理.かつ). }.
";
        let make = |body: &str| ProjectArtifact::create(
            J1_SOURCE_PROFILE.into(),
            SourceSnapshot {
                root_path: Some("root.ddn".into()), entry_path: "main.ddn".into(),
                files: [("root.ddn",root), ("main.ddn","#!日本語
(helper=Logic) 使用.
実行:手続き := { (() helper.wrapper) 表示. }.
"), ("Logic.ddn",body)]
                    .into_iter().map(|(path,text)|SourceFile { path:path.into(), text:text.into() }).collect(),
            }, String::new(),
        ).unwrap();
        let project = make(body);
        let (record, context) = CheckedLogicDependency::for_project_diagnostic(&project,"Logic.ddn",DDL,"論理.ddl").unwrap();
        let syntax = CheckedJ1Profile::resolve(J1_SOURCE_PROFILE).unwrap();
        let parsed = Parser::parse_with_logic_callable_context(body,"Logic.ddn",syntax,context.clone()).unwrap();
        assert_eq!(parsed.items.len(),1);
        assert_eq!(parsed.logic_call_sites().len(),1);
        let site = parsed.logic_call_sites()[0].1;
        assert!(body[site.start..site.end].contains("論理.かつ"));
        let source = crate::authored_project::snapshot_source_selected(&project.source_snapshot,Some(syntax)).unwrap();
        let (preparation, linked_program) = crate::local_project::link_checked_logic_module(
            source, "Logic.ddn", syntax, context,
        ).unwrap();
        assert!(preparation.permits_import("main.ddn", "Logic.ddn", "flagValue"));
        assert!(preparation.source.contains("(() flagValue) 表示"));
        let linked_sites = linked_program.logic_call_sites();
        assert_eq!(linked_sites.len(),1);
        assert_eq!(preparation.original_span(linked_sites[0].1).unwrap(),("Logic.ddn".into(),site));
        let owned = crate::project_revision_envelope_supply::admit_linked_logic_project(
            project.save().unwrap().as_bytes(),&record,
        ).unwrap();
        let mut draft = super::super::super::prepare_precanon_under_candidate_owner(
            &owned,None,&mut |_|panic!("no nominal issue expected"),
        ).unwrap();
        assert!(draft.attach_logic_dependency(&owned,"main.ddn").is_err());
        draft.attach_logic_dependency(&owned,"Logic.ddn").unwrap();
        let mut program = DdnProgram::from_saved_v25_project_with_mode(
            owned.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target(),
        ).unwrap();
        super::super::super::bind_recorded_logic_source_program(&owned,&mut program,&draft).unwrap();
        let result=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update(
            &ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new(),
        ).unwrap();
        assert_eq!(result.display_outputs.iter().map(|row|row.text.as_str()).collect::<Vec<_>>(),["거짓"]);
        println!("B1_LINKED_RUNTIME_RAW path=Logic.ddn start=実行 output=거짓");
        let wrapper = body.lines().last().unwrap();
        let foreign = wrapper.replace("flagValue", "foreignValue");
        let linked_source = format!("#!日本語\n{foreign}\n{wrapper}\n");
        let mut linked = Parser::parse_j1_v25_with_mode(&linked_source,"main.ddn",syntax,ddonirang_lang::ParseMode::Strict).unwrap();
        let start = linked_source.rfind(wrapper).unwrap();
        let source_start = body.rfind(wrapper).unwrap();
        let segment = ddonirang_lang::Span::new(start, start+wrapper.len());
        let mapped = |span: ddonirang_lang::Span| (span.start>=start&&span.end<=segment.end).then(||
            ddonirang_lang::Span::new(span.start-start+source_start,span.end-start+source_start));
        assert!(linked.carry_logic_module_calls(&parsed,segment,|_|None).is_err());
        assert!(linked.logic_call_sites().is_empty());
        linked.carry_logic_module_calls(&parsed,segment,mapped).unwrap();
        let linked_sites = linked.logic_call_sites();
        assert_eq!(linked_sites.len(),1);
        assert_eq!(mapped(linked_sites[0].1),Some(site));
        assert!(linked.carry_logic_module_calls(&parsed,segment,mapped).is_err());
        assert!(record.restore_context(&make(&body.replace("flagValue) 公開","missing) 公開")),&[]).is_err());
        for target in ["missing","かつ"] {
            let bad = body.replace("flagValue) 公開",&format!("{target}) 公開"));
            let error = CheckedLogicDependency::for_project_diagnostic(&make(&bad),"Logic.ddn",DDL,"論理.ddl")
                .err().expect("external or missing definition must be rejected");
            assert!(error.message.contains("E_RD457_EXPORT_TARGET_PRIVATE_OR_UNKNOWN"),"{}",error.message);
        }
    }

    #[test]fn r31_logic_consumer_existing_dto_prepare_open_run_checked_public(){
        use crate::authored_project as authored;
        let initial=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();let original=initial.project().save().unwrap();
        let invoke=authored::operate_with_logic_provider_private;
        let probe=serde_json::json!({"artifact_json_utf8":original}).to_string();
        assert_eq!(invoke("source_probe",&probe).unwrap(),"true");assert_eq!(authored::operate("source_probe",&probe).unwrap(),"true");
        let request=serde_json::json!({"artifact_json_utf8":original,"expected_parent_hex":"","revision_hex":"05","previous_source_artifact_utf8":null});
        let public_prepared=authored::operate("source_prepare",&request.to_string()).unwrap();
        assert!(!public_prepared.is_empty());
        let prepared=invoke("source_prepare",&request.to_string()).unwrap();let receipt:serde_json::Value=serde_json::from_str(&prepared).unwrap();
        let open=serde_json::json!({"artifact_text":prepared,"project_hex":receipt["project_hex"],"parent_hex":receipt["parent_hex"],"revision_hex":receipt["revision_hex"]});
        assert_eq!(invoke("source_open",&open.to_string()).unwrap(),original);
        assert_eq!(authored::operate("source_open",&open.to_string()).unwrap(),original);
        let run=serde_json::json!({"source_artifact_utf8":prepared}).to_string();let result:serde_json::Value=serde_json::from_str(&invoke("public_callable_source_run",&run).unwrap()).unwrap();
        assert_eq!(result["output_log_texts"],serde_json::json!(["거짓","참","참","거짓"]));assert!(result.get("original_binding").is_none());
        assert_eq!(serde_json::from_str::<serde_json::Value>(&authored::operate("public_callable_source_run",&run).unwrap()).unwrap(),result);
        let mut bad=request.clone();bad["trusted"]=true.into();assert!(invoke("source_prepare",&bad.to_string()).is_err());
        let mut update=request.clone();update["previous_source_artifact_utf8"]=prepared.into();assert!(invoke("source_prepare",&update.to_string()).unwrap_err().contains("E_SOURCE_SAVE_PARENT"));
        let mut wrong=open.clone();wrong["project_hex"]="01".repeat(32).into();assert!(invoke("source_open",&wrong.to_string()).unwrap_err().contains("E_SOURCE_SAVE_READBACK_CONTEXT"));
        println!("R31_B1_CONSUMER_DTO_RAW {}",serde_json::json!({"actual_existing_dto":true,"profile":"language-profile:d4465855dc978c9b1e6efdbc94832abc","output":result,"public_probe":"registration required only","public_prepare_open_run":"checked actual source/owner/profile"}));
    }
    #[test]fn r31_logic_consumer_non_builtin_ordinary_ko_operations_preserved(){
        use crate::authored_project::{self as authored,ProjectArtifact,SourceSnapshot,SourceFile};
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.".into()},SourceFile{path:"main.ddn".into(),text:"실행:움직씨 := { 1 보여주기. }.".into()}]},String::new()).unwrap();
        let raw=project.save().unwrap();let invoke=authored::operate_with_logic_provider_private;
        let probe=serde_json::json!({"artifact_json_utf8":raw}).to_string();assert_eq!(invoke("source_probe",&probe).unwrap(),"false");assert_eq!(authored::operate("source_probe",&probe).unwrap(),"false");
        let prepare=serde_json::json!({"artifact_json_utf8":raw,"expected_parent_hex":"","revision_hex":"05","previous_source_artifact_utf8":null}).to_string();assert_eq!(invoke("source_prepare",&prepare).unwrap(),"null");assert_eq!(authored::operate("source_prepare",&prepare).unwrap(),"null");
    }
    #[cfg(windows)]
    #[test]fn r31_logic_consumer_existing_native_json_transport_save_reopen(){
        use crate::authored_project as authored;use sha2::{Digest,Sha256};
        let initial=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();let original=initial.project().save().unwrap();let invoke=authored::operate;
        let prepare=serde_json::json!({"artifact_json_utf8":original,"expected_parent_hex":"","revision_hex":"05","previous_source_artifact_utf8":null}).to_string();let artifact=invoke("source_prepare",&prepare).unwrap();
        let mut random=[0;16];getrandom::fill(&mut random).unwrap();let root=std::env::temp_dir().join(format!("ddn-b1-consumer-{}",hex::encode(random)));std::fs::create_dir(&root).unwrap();
        let request=serde_json::json!({"source_artifact_utf8":artifact,"storage_root":root}).to_string();
        assert!(!root.join("vc04_snapshot_v2.sqlite3").exists());
        let stored=invoke("public_callable_source_save_native",&request).unwrap();assert_eq!(stored,artifact);
        let db=root.join("vc04_snapshot_v2.sqlite3");let before=std::fs::read(&db).unwrap();
        assert_eq!(invoke("public_callable_source_reopen_native",&request).unwrap(),artifact);
        let run=serde_json::json!({"source_artifact_utf8":stored}).to_string();let result:serde_json::Value=serde_json::from_str(&invoke("public_callable_source_run",&run).unwrap()).unwrap();assert_eq!(result["output_log_texts"],serde_json::json!(["거짓","참","참","거짓"]));
        assert_eq!(std::fs::read(&db).unwrap(),before);assert!(invoke("public_callable_source_save_native",&request).is_err());assert_eq!(std::fs::read(&db).unwrap(),before);
        assert_eq!(authored::operate("public_callable_source_reopen_native",&request).unwrap(),artifact);
        println!("R31_B1_CONSUMER_NATIVE_DTO_RAW {}",serde_json::json!({"existing_requests":true,"native_save_reopen":true,"db_sha256":hex::encode(Sha256::digest(before)),"db_preserved":true,"checked_public_native_transport":true,"separate_process":false}));
    }

    #[test]fn r31_logic_consumer_non_builtin_j1_exports_preserve_existing_probe(){
        use crate::authored_project::{self as authored,ProjectArtifact,SourceSnapshot,SourceFile};
        let root="#!日本語\n(版=\"v25\", モジュール群=列[(名前=アプリ, 見出し=\"main.ddn\", 原文=\"main.ddn\")], 開始=(モジュール=アプリ, 定義=実行)) 構成.\n";
        let body="#!日本語\n(F=F) 公開.\nF:有理数:関数 := { F <- (1,1) 有理数. }.\n実行:手続き := { 1 表示. }.\n";
        let project=ProjectArtifact::create(ddonirang_lang::ddl::J1_SOURCE_PROFILE.into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![SourceFile{path:"root.ddn".into(),text:root.into()},SourceFile{path:"main.ddn".into(),text:body.into()}]},String::new()).unwrap();
        let raw=project.save().unwrap();let probe=serde_json::json!({"artifact_json_utf8":raw}).to_string();
        let legacy=crate::authored_project::source_save::requirements(&probe);assert!(legacy.is_ok(),"legacy actual path: {legacy:?}");
        assert_eq!(authored::operate("source_probe",&probe),legacy);
    }

    #[test]fn r31_logic_consumer_diagnostic_actual_pin_span_tracks_unicode_and_edited_source(){
        use sha2::Digest;
        use crate::authored_project::{self as authored,registration::logic_dependency::CheckedLogicDependency as Dependency};
        let initial=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();let project=initial.project();
        let context=Dependency::for_project(project,"main.ddn",DDL,"論理.ddl").unwrap().restore_context(project,&[]).unwrap();
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(&project.source_profile).unwrap();let mut previous=None;
        for (prefix,label) in [("","extra"),("// 👣 新しい入力\r\n","余"),("// 👣👣\r\n\r\n","別")]{
            let text=LOGIC_CALLS.replacen("#!日本語\n",&format!("#!日本語\n{prefix}"),1).replacen("右=偽",&format!("{label}=偽"),1);
            let expected=ddonirang_lang::Parser::parse_with_logic_callable_context(&text,"main.ddn",syntax,context.clone()).unwrap_err();
            let mut edited=project.clone();edited.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=text.clone();edited.content_sha256=edited.checksum().unwrap();
            let raw=serde_json::to_string(&edited).unwrap();let request=serde_json::json!({"artifact_json_utf8":raw,"expected_parent_hex":"","revision_hex":"05","previous_source_artifact_utf8":null}).to_string();
            let error=authored::source_save::logic_consumer::operate_private_diagnostic("source_prepare",&request).unwrap_err();
            assert_eq!(error.code,expected.j1_source_code());assert_eq!(error.source_path,"main.ddn");assert_eq!(error.span,Some((expected.span.start,expected.span.end)));assert!(error.cause.is_none());
            let actual_slice=&text[expected.span.start..expected.span.end];assert_eq!(actual_slice,label);if let Some(old)=previous{assert_ne!(error.span,Some(old),"new input cannot keep old position");}previous=error.span;
            assert!(authored::operate_with_logic_provider_private("source_prepare",&request).unwrap_err().contains(&error.message),"legacy presentation wrapper retains actual producer message");
            println!("R31_B1_CONSUMER_TYPED_PIN_RAW {}",serde_json::json!({"source_sha256":hex::encode(sha2::Sha256::digest(text.as_bytes())),"diagnostic":error.to_json_value(),"exact_slice":actual_slice,"same_actual_parse_error":true,"post_failure_analysis":false}));
        }
    }
    #[test]fn r31_logic_consumer_diagnostic_keeps_initial_artifact_root_and_body_failures(){
        use crate::authored_project as authored;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();
        for (path,from,to,bad,phase) in [("main.ddn","表示.","表示。",'。',"lang_lex"),("root.ddn","(版=","（版=",'（',"project_root")]{
            let mut edited=actual.project().clone();let file=edited.source_snapshot.files.iter_mut().find(|f|f.path==path).unwrap();file.text=file.text.replacen(from,to,1);let at=file.text.find(bad).unwrap();edited.content_sha256=edited.checksum().unwrap();
            let raw=serde_json::to_string(&edited).unwrap();
            for operation in ["source_probe","source_prepare"]{
                let request=if operation=="source_probe"{serde_json::json!({"artifact_json_utf8":raw})}else{serde_json::json!({"artifact_json_utf8":raw,"expected_parent_hex":"","revision_hex":"05","previous_source_artifact_utf8":null})};
                let error=authored::source_save::logic_consumer::operate_private_diagnostic(operation,&request.to_string()).unwrap_err();
                assert_eq!(error.code,"E_MULTILANG_FULLWIDTH_SYNTAX");assert_eq!(error.source_path,path);assert_eq!(error.span,Some((at,at+3)));assert_eq!(error.phase,phase);
                println!("R31_B1_CONSUMER_TYPED_BOOTSTRAP_RAW {}",serde_json::json!({"operation":operation,"diagnostic":error.to_json_value(),"expected_actual_three_byte_character":bad}));
            }
        }
    }
    #[test]fn r31_logic_consumer_diagnostic_ddl_failure_uses_actual_ddl_path_and_record_replay(){
        use crate::authored_project::registration::logic_dependency::CheckedLogicDependency as Dependency;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();let ddl=DDL.replacen("右:真偽","右:有理数",1);
        let error=Dependency::for_project_diagnostic(actual.project(),"main.ddn",&ddl,"診断.ddl").err().unwrap();
        assert_eq!(error.source_path,"診断.ddl");let (start,end)=error.span.expect("actual DDL producer span");assert!(end>start&&end<=ddl.len());assert!(error.message.contains("E_LOGIC_DDL_PIN_TYPE_OR_SCOPE"),"{}",error.message);
        let record=Dependency::for_project(actual.project(),"main.ddn",DDL,"論理.ddl").unwrap();
        let mut edited=actual.project().clone();let file=edited.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap();file.text=file.text.replacen("右=偽","extra=偽",1);
        let replay=record.restore_context_diagnostic(&edited,&[]).err().unwrap();assert_eq!(replay.code,"E_LOGIC_CALL_PIN_OWNER");assert_eq!(replay.source_path,"main.ddn");assert!(replay.span.is_some());
        println!("R31_B1_CONSUMER_TYPED_DDL_REPLAY_RAW {}",serde_json::json!({"ddl":error.to_json_value(),"replay":replay.to_json_value(),"restore_has_no_issuer":true}));
    }
    #[test]fn r31_logic_consumer_diagnostic_transport_failures_never_invent_source_positions(){
        let invoke=crate::authored_project::source_save::logic_consumer::operate_private_diagnostic;
        for raw in ["{}","{\"artifact_json_utf8\":null}","{\"artifact_json_utf8\":\"{}\",\"trusted\":true}"]{
            let error=invoke("source_probe",raw).unwrap_err();assert!(error.span.is_none());assert!(error.source_path.is_empty());assert_eq!(error.phase,"source_prepare");
        }
        let error=invoke("public_callable_source_save_native","{}").unwrap_err();assert_eq!(error.code,"E_MULTILANG_OPERATION_CONTEXT");assert!(error.span.is_none());assert!(error.source_path.is_empty());
    }

    #[test]fn r31_logic_consumer_diagnostic_explicit_runtime_failure_is_same_actual_eval_error(){
        use crate::authored_project as authored;
        let body=LOGIC_CALLS.replacen("右=偽","右=1",1);let actual=actual_logic_source(&body,logic_compiler_context()).unwrap();
        let raw=actual.project().save().unwrap();let prepare=serde_json::json!({"artifact_json_utf8":raw,"expected_parent_hex":"","revision_hex":"05","previous_source_artifact_utf8":null}).to_string();
        let invoke=authored::source_save::logic_consumer::operate_private_diagnostic;
        let artifact=invoke("source_prepare",&prepare).unwrap();let request=serde_json::json!({"source_artifact_utf8":artifact}).to_string();
        let error=invoke("public_callable_source_run",&request).unwrap_err();
        assert_eq!(error.source_path,"main.ddn");assert_eq!(error.phase,"runtime_execution");assert!(error.message.contains("E_RUNTIME_CONDITION_TYPE"));
        // The shared typed evaluator is the expected authority. Do not infer
        // a more specific code by reverse-parsing the legacy human message.
        let source=crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision::from_artifact_logic_private(&artifact).unwrap();
        let owned=source.recorded_execution_owner().unwrap();let precanon=owned.precanon_view();let mut program=DdnProgram::from_saved_v25_project_with_mode(precanon.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        super::super::super::bind_recorded_logic_source_program(&precanon,&mut program,owned.registration().unwrap()).unwrap();
        let actual=crate::ddn_runtime::DdnRunner::new(program,"実行").run_update_diagnostic(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).err().unwrap();
        let expected=crate::runtime_surface::execution_error_diagnostic(actual,"main.ddn");assert_eq!(error,expected);
        assert_eq!(authored::operate_with_logic_provider_private("public_callable_source_run",&request).unwrap_err(),error.message);
        println!("R31_B1_CONSUMER_TYPED_RUNTIME_RAW {}",serde_json::json!({"diagnostic":error.to_json_value(),"actual_eval_projection_equal":true,"error_classification_not_promoted":true,"post_failure_analysis":false}));
    }
    #[test]fn r31_logic_consumer_diagnostic_valid_typed_requests_preserve_checked_public_result(){
        use crate::authored_project as authored;
        let actual=actual_logic_source(LOGIC_CALLS,logic_compiler_context()).unwrap();let raw=actual.project().save().unwrap();
        let invoke=authored::source_save::logic_consumer::operate_private_diagnostic;
        assert_eq!(invoke("source_probe",&serde_json::json!({"artifact_json_utf8":raw}).to_string()).unwrap(),"true");
        let prepare=serde_json::json!({"artifact_json_utf8":raw,"expected_parent_hex":"","revision_hex":"05","previous_source_artifact_utf8":null}).to_string();let artifact=invoke("source_prepare",&prepare).unwrap();
        let request=serde_json::json!({"source_artifact_utf8":artifact}).to_string();let result:serde_json::Value=serde_json::from_str(&invoke("public_callable_source_run",&request).unwrap()).unwrap();assert_eq!(result["output_log_texts"],serde_json::json!(["거짓","참","참","거짓"]));
        assert_eq!(serde_json::from_str::<serde_json::Value>(&authored::operate("public_callable_source_run",&request).unwrap()).unwrap(),result);
    }

}
