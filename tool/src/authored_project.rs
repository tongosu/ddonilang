//! Alpha2 authored identity storage. This is admission, not composition activation.
//! Locators and source bytes never supply persistent identities.
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
pub(crate) mod registration;
mod source_save;
mod language_support;
mod joint_execution;
#[cfg(windows)]
pub(crate) use source_save::{check_public_edition_from_current, inspect_public_callable_original, prepare_public_callable_original_diagnostic, run_public_source_diagnostic};
#[cfg(windows)]
pub(crate) use source_save::logic_consumer::selected as selected_logic_dependency;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use source_save::logic_consumer::recorded_payload_is_selected as recorded_logic_payload_is_selected;
mod source_projection;
pub(crate) mod public_surface;
pub(crate) use source_save::prepare_authoring_source;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const SCHEMA: &str = "ddn.authored_project_artifact.v1";
pub const SCHEMA_V2: &str = "ddn.authored_project_artifact.v2";
pub const SCHEMA_V3:&str="ddn.authored_project_artifact.v3";
const MAX_BYTES: usize = 67_108_864;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub(crate) fn fresh() -> Result<Self, String> {
                let mut bytes = [0; 32];
                getrandom::fill(&mut bytes)
                    .map_err(|_| "E_AUTHORED_ID_RANDOM: 정체 생성에 실패했습니다".to_string())?;
                Ok(Self(hex::encode(bytes)))
            }
            fn valid(&self) -> bool {
                valid_id(&self.0)
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}
identity!(ProjectId);
identity!(ModuleId);
identity!(CompositionId);
identity!(BindingId);

fn valid_id(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub path: String,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,
    pub entry_path: String,
    pub files: Vec<SourceFile>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredComposition {
    pub id: CompositionId,
    pub definition: String,
    pub binding_ids: Vec<BindingId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registration: Option<crate::authored_bogae::Registration>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredModule {
    pub id: ModuleId,
    pub name: String,
    pub head: String,
    pub sources: Vec<String>,
    pub compositions: Vec<AuthoredComposition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<crate::authored_bogae::BindingDefinition>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub module: ModuleId,
    pub composition: CompositionId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredIdentity {
    pub schema: String,
    pub logical_dependency_id: ProjectId,
    pub modules: Vec<AuthoredModule>,
    pub entry: Option<Entry>,
    pub clone_origin: Option<ProjectId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectArtifact {
    pub schema: String,
    pub source_profile: String,
    pub source_snapshot: SourceSnapshot,
    pub authored_identity: AuthoredIdentity,
    pub content_sha256: String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub language_binding:Option<FirstCallableLanguageBinding>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirstCallableDdlSource {pub path:String,pub text:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirstCallableLanguageBinding {
    pub profile_identity:String,pub profile_version:String,pub profile_content_sha256:String,
    pub registration_revision:u32,pub original_project_json_utf8:String,pub original_module_id:String,
    pub original_source_path:String,pub ddl_source:FirstCallableDdlSource,
    pub consumer_module_id:String,pub consumer_source_path:String,
}
/// Capability is reconstructed from the original recorded source/catalog; it
/// is never serialized as a proof flag and grants no Native transaction/run.
#[derive(Clone)]
pub struct FirstCallablePublicDdl {
    original:FirstCallableAuthoredContext,original_json:String,
    owner:std::sync::Arc<crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>,
    facts:registration::FirstCallableExportFacts,profile:ddonirang_lang::ddl::CheckedFirstCallableProfile,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCallableSurfaceSpec {
    pub public_name:String,pub surface:String,
    /// Original parameter name -> finite surface -> registered particle.
    pub pins:Vec<(String,String,String)>,
}
#[derive(Clone)]
pub struct PublicCallablePublicDdl {
    original:ProjectArtifact,module_head:String,
    source_owner:std::sync::Arc<crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1>,
    registration:std::sync::Arc<registration::RegistrationDraft>,
    facts:registration::CheckedPublicCallableExport,
    compiler:std::sync::Arc<ddonirang_lang::parser::RegisteredPublicCallableContext>,
}
#[derive(Clone)]
pub struct PublicCallableAuthoredContext {
    export:std::sync::Arc<PublicCallablePublicDdl>,consumer:ProjectArtifact,
    consumer_module:String,path:String,
    compiler:std::sync::Arc<ddonirang_lang::parser::RegisteredPublicCallableContext>,
}
/// Process-local recorded material only. This is deliberately not Serialize or
/// Deserialize: no existing source wire slot admits a successor registry record.
pub(crate) struct PublicCallableOriginalMaterial {
    original_json:String,module:String,start_identity:Vec<u8>,captures:String,
    resources:Vec<registration::RecordedRegistrationBlob>,
    profile:ddonirang_lang::ddl::RegisteredSuccessorProfile,ddl:String,ddl_path:String,
}
impl PublicCallablePublicDdl {
    pub fn profile(&self)->&ddonirang_lang::ddl::RegisteredSuccessorProfile{self.compiler.profile()}
    pub(crate) fn recorded_original_material(&self)->Result<PublicCallableOriginalMaterial,String>{
        self.recheck()?;
        let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(
            (*self.source_owner).clone(),registration::restore_precanon_public_callables_candidate(
                &self.source_owner,&self.facts.module,self.registration.recorded_blobs())?)?;
        Ok(PublicCallableOriginalMaterial{original_json:self.original.save()?,module:self.facts.module.clone(),
            start_identity:owned.definition_identity().as_bytes().to_vec(),
            captures:self.source_owner.sources().recorded_capture_catalog_json()?,resources:self.registration.recorded_blobs(),
            profile:self.profile().clone(),ddl:self.compiler.ddl_source().into(),ddl_path:self.compiler.ddl_path().into()})
    }
    /// Reconstruct the original capture/G1 owner from actual recorded bytes.
    /// The profile is still a live registry capability, NOT cold admission.
    #[cfg(test)]
    pub(crate) fn restore_recorded_original(material:PublicCallableOriginalMaterial)->Result<Self,String>{
        Self::restore_recorded_original_diagnostic(material,None)
    }

    fn restore_recorded_original_diagnostic(material:PublicCallableOriginalMaterial,
        mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    )->Result<Self,String>{
        use crate::project_revision_envelope_supply as supply;
        if material.original_json.len()>8_000_000||material.captures.len()>1_000_000||material.resources.len()>64
            ||material.ddl.len()>16_384||material.ddl_path.len()>4096{return Err("E_DDL_ORIGINAL_RECORD_LIMIT".into());}
        let original=open(material.original_json.as_bytes())?;
        if !matches!(original.schema.as_str(),SCHEMA|SCHEMA_V2){return Err("E_DDL_NESTED_ORIGINAL_FORBIDDEN".into());}
        let snapshot=&original.source_snapshot;
        let carrier=supply::CanonicalIdentityBytesV1::from_owner_bytes(&material.start_identity).map_err(|_|"E_DDL_ORIGINAL_START_CARRIER")?;
        let j1=if original.source_profile==ddonirang_lang::ddl::J1_SOURCE_PROFILE{Some(ddonirang_lang::ddl::CheckedJ1Profile::resolve(&original.source_profile).map_err(|e|e.to_string())?)}else{None};
        let roles=supply::saved_authored_source_roles_selected(snapshot,j1)?;
        let blobs=snapshot.files.iter().map(|file|Ok(supply::SavedSourceBlobV1{path:&file.path,
            role:roles.get(&file.path).ok_or("E_DDL_ORIGINAL_SOURCE_ROLE")?,bytes:file.text.as_bytes()})).collect::<Result<Vec<_>,String>>()?;
        let saved=supply::collect_saved_source_supply(snapshot.root_path.as_deref().ok_or("E_DDL_ORIGINAL_ROOT_REQUIRED")?,
            &snapshot.entry_path,&carrier,&blobs).map_err(|_|"E_DDL_ORIGINAL_SOURCE_SUPPLY")?;
        let sources=std::sync::Arc::new(match j1{Some(profile)=>supply::restore_saved_v25_sources_with_j1_profile(saved,&material.captures,profile),None=>supply::restore_saved_v25_sources(saved,&material.captures)}?);
        let owned=supply::admit_precanon_authored_source(sources,material.original_json.as_bytes())?;
        let registration=registration::restore_precanon_public_callables_candidate(&owned,&material.module,material.resources)?;
        let facts=registration.checked_public_callable_export(&owned,&material.module)?;
        if hex::encode(Sha256::digest(material.profile.content()))!=material.profile.content_sha256()
            ||material.profile.surfaces().len()!=facts.definitions.len(){return Err("E_DDL_PUBLIC_PROFILE_OWNER".into());}
        for def in &facts.definitions {
            let rows=material.profile.surfaces().iter().filter(|row|row.identity==def.identity).collect::<Vec<_>>();
            if rows.len()!=1||rows[0].pins.len()!=def.pins.len()||def.pins.iter().any(|pin|
                rows[0].pins.iter().filter(|row|row.identity==pin.identity).count()!=1){return Err("E_DDL_PUBLIC_PROFILE_OWNER".into());}
        }
        let checked=supply::finalize_registered_authored_source(owned.clone(),
            registration::restore_precanon_public_callables_candidate(&owned,&material.module,registration.recorded_blobs())?)?;
        if checked.definition_identity().as_bytes()!=material.start_identity.as_slice(){return Err("E_DDL_ORIGINAL_START_MISMATCH".into());}
        let module=original.authored_identity.modules.iter().find(|module|module.id.as_str()==material.module).ok_or("E_DDL_ORIGINAL_MODULE_REQUIRED")?;
        let public_targets=original.source_owned_public_facts(&material.module)?.public_targets();
        let mut bindings=Vec::new();let mut next_node=1u64;
        for path in &module.sources {
            let file=original.source_snapshot.files.iter().find(|file|&file.path==path).ok_or("E_DDL_ORIGINAL_SOURCE_REQUIRED")?;
            let (program,next,offset)=if original.source_profile==ddonirang_lang::ddl::J1_SOURCE_PROFILE{
                let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(&original.source_profile).map_err(|e|e.to_string())?;
                let (program,next)=ddonirang_lang::Parser::parse_j1_public_original(&file.text,path,next_node,syntax).map_err(|e|e.to_string())?;
                (program,next,0)
            }else{
            let front=ddonirang_lang::parse_module_directive_frontdoor_with_mode(&file.text,path,ddonirang_lang::ParseMode::Strict,
                &ddonirang_lang::ModuleDirectiveContext::default().with_public_definitions(public_targets.clone())).map_err(|e|e.to_string())?;
            let offset=file.text.len().checked_sub(front.directives.declaration_source.len()).ok_or("E_DDL_ORIGINAL_SPAN_REQUIRED")?;
            if file.text.get(offset..)!=Some(front.directives.declaration_source.as_str()){return Err("E_DDL_ORIGINAL_SPAN_REQUIRED".into());}
            let (program,next)=ddonirang_lang::parser::Parser::parse_public_original_tail(&front.directives.declaration_source,path,next_node).map_err(|e|e.to_string())?;                (program,next,offset)
            };next_node=next;
            for def in facts.definitions.iter().filter(|row|&row.source_path==path) {
                let seeds=program.items.iter().map(|ddonirang_lang::TopLevelItem::SeedDef(seed)|seed).filter(|seed|
                    seed.canonical_name==def.internal_name&&offset.checked_add(seed.span.start)==Some(def.start as usize)
                    &&offset.checked_add(seed.span.end)==Some(def.end as usize)).collect::<Vec<_>>();
                if seeds.len()!=1{return Err("E_DDL_PUBLIC_DEFINITION_OWNER".into());}
                bindings.push(ddonirang_lang::parser::PublicCallableBinding{definition:def.identity.clone(),
                    pin_identities:def.pins.iter().map(|pin|pin.identity.clone()).collect(),original:seeds[0].clone(),source_path:path.clone(),source_text:file.text.clone()});
            }
        }
        let compiler=ddonirang_lang::parser::RegisteredPublicCallableContext::from_public_and_ddl(material.profile,bindings,
            &material.ddl,&material.ddl_path,next_node)
            .map_err(|error|ddl_parse_failure(error,&material.ddl_path,&mut diagnostic))?;
        let export=Self{module_head:module.head.clone(),original,source_owner:std::sync::Arc::new(owned),
            registration:std::sync::Arc::new(registration),facts,compiler:std::sync::Arc::new(compiler)};
        export.recheck()?;Ok(export)
    }

    pub fn original_module_identity(&self)->&str{&self.facts.module}
    pub fn definition_identities(&self)->Vec<(&str,&str)>{self.facts.definitions.iter().map(|row|(row.public_name.as_str(),row.identity.as_str())).collect()}
    fn recheck(&self)->Result<(),String>{
        if self.original.content_sha256!=self.original.checksum()?||self.source_owner.project()!=&self.original
            ||self.registration.checked_public_callable_export(&self.source_owner,&self.facts.module)?!=self.facts{return Err("E_DDL_ORIGINAL_STALE".into());}Ok(())
    }
    pub fn activate_for_consumer(&self,consumer:&ProjectArtifact,module_id:&str,path:&str)->Result<PublicCallableAuthoredContext,String>{
        self.activate_for_consumer_diagnostic(consumer,module_id,path,None)
    }
    fn activate_for_consumer_diagnostic(&self,consumer:&ProjectArtifact,module_id:&str,path:&str,
        diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<PublicCallableAuthoredContext,String>{
        self.activate_for_consumer_selected(consumer,module_id,path,false,diagnostic)
            .map(|(context,_)|context)
    }
    pub(crate) fn activate_for_consumer_module(
        &self,consumer:&ProjectArtifact,module_id:&str,path:&str,
    )->Result<(PublicCallableAuthoredContext,ddonirang_lang::CanonProgram),String>{
        self.activate_for_consumer_selected(consumer,module_id,path,true,None)
    }
    fn activate_for_consumer_selected(&self,consumer:&ProjectArtifact,module_id:&str,path:&str,
        linked_module:bool,
        mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    )->Result<(PublicCallableAuthoredContext,ddonirang_lang::CanonProgram),String>{
        self.recheck()?;consumer.validate()?;
        if consumer.content_sha256!=consumer.checksum()?{return Err("E_DDL_CONSUMER_STALE".into());}
        let module=consumer.authored_identity.modules.iter().find(|row|row.id.as_str()==module_id).ok_or("E_DDL_CONSUMER_MODULE_REQUIRED")?;
        if !module.sources.iter().any(|source|source==path){return Err("E_DDL_CONSUMER_SOURCE_OWNER_REQUIRED".into());}
        if linked_module && (module.head!=path || consumer.source_snapshot.entry_path==path){
            return Err("E_DDL_CONSUMER_SOURCE_OWNER_REQUIRED".into());
        }
        let file=consumer.source_snapshot.files.iter().find(|file|file.path==path).ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?;
        let j1=if self.original.source_profile==ddonirang_lang::ddl::J1_SOURCE_PROFILE{Some(ddonirang_lang::ddl::CheckedJ1Profile::resolve(&self.original.source_profile).map_err(|e|e.to_string())?)}else{None};
        let consumer_j1=if consumer.source_profile==ddonirang_lang::ddl::J1_SOURCE_PROFILE{
            Some(ddonirang_lang::ddl::CheckedJ1Profile::resolve(&consumer.source_profile).map_err(|error|error.to_string())?)
        }else{None};
        let compiler=(*self.compiler).clone().with_consumer_j1(consumer_j1);
        let public_targets = if linked_module {
            let syntax = consumer_j1.ok_or("E_J1_PROFILE_REQUIRED")?;
            ddonirang_lang::Parser::j1_local_export_targets(&file.text, path, syntax)
                .map_err(|error| error.to_string())?
        } else {
            Default::default()
        };
        let request=crate::local_project::resolve_public_callable_module_translation_request_selected(&file.text,path,&snapshot_source_selected(&self.original.source_snapshot,j1)?,&self.module_head,&compiler,j1,public_targets)
            .map_err(|error|{
                let message=if error.code=="E_MULTILANG_SUCCESSOR_HEADER_SCOPE" {
                    error.message.clone()
                } else {
                    format!("{}: {}",error.code,error.message)
                };
                if error.code=="E_DDL_ACTIVATION_REQUIRED" || (matches!(error.code,"E_DDL_ACTIVATION_OWNER_OR_PROFILE"|"E_DDL_ORIGINAL_MODULE_MISMATCH"|"E_MULTILANG_SUCCESSOR_HEADER_SCOPE"|"E_DDL_ACTIVATION_PARSE")
                    && error.source_span.is_some_and(|(start,end)|start<end&&file.text.get(start..end).is_some())){
                    if let Some(output)=diagnostic.as_deref_mut(){
                        let mut located=error.into_execution_diagnostic();
                        located.message=message.clone();
                        *output=Some(located);
                    }
                }
                message
            })?;
        let compiler=std::sync::Arc::new(compiler.with_module_activation(request.binding).map_err(|error|error.to_string())?);
        // The common parser rechecks exact explicit activation and all calls;
        // the envelope or profile itself never activates a source program.
        let program=if linked_module {
            ddonirang_lang::parser::Parser::parse_with_public_callable_module_context(&file.text,path,compiler.clone())
        }else{
            ddonirang_lang::parser::Parser::parse_with_public_callable_context(&file.text,path,compiler.clone())
        }.map_err(|error|{
            // Use this actual failed parse's byte span, never a second analysis
            // or a freshly issued profile for diagnostic presentation.
            if error.span.start<error.span.end&&file.text.get(error.span.start..error.span.end).is_some(){
                let message=format!("{} [원문 {}:{}..{}]",error,path,error.span.start,error.span.end);
                let code=match error.message.as_str() {
                    "E_MULTILANG_CALL_UNREGISTERED"=>Some("E_MULTILANG_CALL_UNREGISTERED"),
                    "E_MULTILANG_PIN_UNREGISTERED"=>Some("E_MULTILANG_PIN_UNREGISTERED"),
                    "E_MULTILANG_PUBLIC_CALLER_SCOPE" if linked_module=>Some("E_MULTILANG_PUBLIC_CALLER_SCOPE"),
                    _=>match error.code() {
                        code @ ("E_CALL_REQUIRED_PIN_MISSING" | "E_PARSE_CALL_PIN_DUPLICATE"
                            | "E_CALL_POSITIONAL_ARGUMENT_EXCESS")=>Some(code),
                        _=>None,
                    },
                };
                if let (Some(code),Some(output))=(code,diagnostic) {
                    *output=Some(crate::preprocess::ExecutionDiagnostic{
                            code,message:message.clone(),source_path:path.into(),
                            span:Some((error.span.start,error.span.end)),phase:"lang_parse",cause:None,
                        offending_field:None,expected:None,actual:None,
                    });
                }
                message
            }else{
                let message=format!("{} [원문 {}: 위치 미확정]",error,path);
                if linked_module && error.message=="E_MULTILANG_PUBLIC_CALLER_SCOPE"{
                    if let Some(output)=diagnostic{
                        *output=Some(crate::preprocess::ExecutionDiagnostic{
                            code:"E_MULTILANG_PUBLIC_CALLER_SCOPE",message:message.clone(),source_path:path.into(),
                            span:None,phase:"lang_parse",cause:None,
                            offending_field:None,expected:None,actual:None,
                        });
                    }
                }
                message
            }
        })?;
        Ok((PublicCallableAuthoredContext{export:std::sync::Arc::new(self.clone()),consumer:consumer.clone(),consumer_module:module_id.into(),path:path.into(),compiler},program))
    }
}
impl PublicCallableAuthoredContext {
    pub(crate) fn linked_module_prefix(
        &self,
        project: &ProjectArtifact,
    ) -> Result<(&str, ddonirang_lang::ModuleDirectiveDocument), String> {
        self.export.recheck()?;
        if project != &self.consumer || project.source_snapshot.entry_path == self.path
            || !project.authored_identity.modules.iter().any(|module| {
                module.id.as_str() == self.consumer_module && module.head == self.path
            })
        {
            return Err("E_DDL_CONSUMER_SOURCE_OWNER_REQUIRED".into());
        }
        let file = project.source_snapshot.files.iter().find(|file| file.path == self.path)
            .ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?;
        let prefix = ddonirang_lang::Parser::public_callable_module_prefix(
            &file.text, &self.path, &self.compiler,
        ).map_err(|error| error.to_string())?;
        Ok((&self.path, prefix))
    }

    pub(crate) fn has_legacy_ddl_original(&self)->bool {self.export.registration.has_legacy_ddl_start()}
    pub fn compiler_context(&self)->&std::sync::Arc<ddonirang_lang::parser::RegisteredPublicCallableContext>{&self.compiler}
    pub fn original_module_identity(&self)->&str{self.export.original_module_identity()}
    pub fn definition_identities(&self)->Vec<(&str,&str)>{self.export.definition_identities()}
    pub(crate) fn original_source_profile(&self) -> &str {
        &self.export.original.source_profile
    }
    pub(crate) fn verify_runtime_activation(&self,source:&str,path:&str)->Result<(),String>{
        self.parse_verified_consumer(source, path).map(|_| ())
    }
    pub(crate) fn parse_verified_consumer(&self, source: &str, path: &str) -> Result<ddonirang_lang::CanonProgram, String> {
        self.export.recheck()?;
        if self.path!=path||!self.consumer.authored_identity.modules.iter().any(|module|module.id.as_str()==self.consumer_module&&module.sources.iter().any(|file|file==path))
            ||!self.consumer.source_snapshot.files.iter().any(|file|file.path==path&&file.text.as_bytes()==source.as_bytes()){
            return Err("E_DDL_CONSUMER_STALE".into());
        }
        ddonirang_lang::parser::Parser::parse_with_public_callable_context(source,path,self.compiler.clone())
            .map_err(|error|error.to_string())
    }
}
fn checked_public_callable_surfaces(
    facts:&registration::CheckedPublicCallableExport,surfaces:&[PublicCallableSurfaceSpec],
)->Result<Vec<ddonirang_lang::ddl::SuccessorCallableSurface>,String>{
    if surfaces.len()!=facts.definitions.len(){return Err("E_DDL_PUBLIC_COVERAGE".into());}
    facts.definitions.iter().map(|definition| {
        let rows=surfaces.iter().filter(|row|row.public_name==definition.public_name).collect::<Vec<_>>();
        if rows.len()!=1{return Err("E_DDL_PUBLIC_COVERAGE".into());}
        let row=rows[0];
        if row.pins.len()!=definition.pins.len(){return Err("E_DDL_PUBLIC_PIN_OWNER".into());}
        let pins=definition.pins.iter().map(|pin| {
            let matches=row.pins.iter().filter(|mapped|mapped.0==pin.name).collect::<Vec<_>>();
            if matches.len()!=1{return Err("E_DDL_PUBLIC_PIN_OWNER".into());}
            Ok(ddonirang_lang::ddl::SuccessorPinSurface{
                identity:pin.identity.clone(),surface:matches[0].1.clone(),particle:matches[0].2.clone(),
            })
        }).collect::<Result<Vec<_>,String>>()?;
        Ok(ddonirang_lang::ddl::SuccessorCallableSurface{
            identity:definition.identity.clone(),surface:row.surface.clone(),pins,
        })
    }).collect()
}

impl ProjectArtifact {
    /// Actual original-owner producer; no serde proof flags, profile strings
    /// or source path can bypass the existing public G1 owner/checker.
    pub fn produce_public_callable_ddl(&self,module_id:&str,surfaces:Vec<PublicCallableSurfaceSpec>,ddl_source:&str,ddl_path:&str)->Result<PublicCallablePublicDdl,String>{
        self.produce_public_callable_ddl_diagnostic(module_id,surfaces,ddl_source,ddl_path,None)
    }

    fn produce_public_callable_ddl_diagnostic(&self,module_id:&str,surfaces:Vec<PublicCallableSurfaceSpec>,
        ddl_source:&str,ddl_path:&str,mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    )->Result<PublicCallablePublicDdl,String>{
        self.validate()?;if self.content_sha256!=self.checksum()?{return Err("E_DDL_ORIGINAL_STALE".into());}
        if !matches!(self.schema.as_str(),SCHEMA|SCHEMA_V2){return Err("E_DDL_ORIGINAL_SCHEMA_UNSUPPORTED".into());}
        let module=self.authored_identity.modules.iter().find(|module|module.id.as_str()==module_id).ok_or("E_DDL_ORIGINAL_MODULE_REQUIRED")?;
        let owned=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(self.save()?.as_bytes())?;
        let mut entropy=|raw:&mut [u8;32]|getrandom::fill(raw).map_err(|_|"E_DDL_IDENTITY_ENTROPY".to_string());
        let registration=registration::prepare_public_callables_under_candidate_owner(&owned,module_id,None,&mut entropy)?;
        let facts=registration.checked_public_callable_export_diagnostic(&owned,module_id,diagnostic.as_deref_mut())?;
        let requested=checked_public_callable_surfaces(&facts,&surfaces)?.into_iter().map(|row|
            registration::PublicCallableSurfaceRequest{definition:row.identity,surface:row.surface,
                pins:row.pins.into_iter().map(|pin|(pin.identity,pin.surface,pin.particle)).collect()}
        ).collect();
        let mut registry=ddonirang_lang::ddl::SuccessorRegistryOwner::default();
        let (facts,profile)=registration.issue_public_callable_profile(&owned,module_id,requested,&mut registry,&mut entropy)?;
        registry.lookup(profile.identity(),profile.version(),profile.content_sha256(),profile.registration_revision()).map_err(|error|error.to_string())?;
        let public_targets=self.source_owned_public_facts(module_id)?.public_targets();
        let mut bindings=Vec::new();let mut next_node=1u64;
        for source_path in &module.sources {
            let file=self.source_snapshot.files.iter().find(|file|&file.path==source_path).ok_or("E_DDL_ORIGINAL_SOURCE_REQUIRED")?;
            let (program,next,offset)=if self.source_profile==ddonirang_lang::ddl::J1_SOURCE_PROFILE{
                let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(&self.source_profile).map_err(|e|e.to_string())?;
                let (program,next)=ddonirang_lang::Parser::parse_j1_public_original(&file.text,source_path,next_node,syntax).map_err(|e|e.to_string())?;
                (program,next,0)
            }else{
            let front=ddonirang_lang::parse_module_directive_frontdoor_with_mode(&file.text,source_path,ddonirang_lang::ParseMode::Strict,
                &ddonirang_lang::ModuleDirectiveContext::default().with_public_definitions(public_targets.clone())).map_err(|error|error.to_string())?;
            let offset=file.text.len().checked_sub(front.directives.declaration_source.len()).ok_or("E_DDL_ORIGINAL_SPAN_REQUIRED")?;
            if file.text.get(offset..)!=Some(front.directives.declaration_source.as_str()){return Err("E_DDL_ORIGINAL_SPAN_REQUIRED".into());}
            let (program,next)=ddonirang_lang::parser::Parser::parse_public_original_tail(&front.directives.declaration_source,source_path,next_node).map_err(|error|error.to_string())?;                (program,next,offset)
            };next_node=next;
            for definition in facts.definitions.iter().filter(|row|&row.source_path==source_path){
                let candidates=program.items.iter().map(|ddonirang_lang::TopLevelItem::SeedDef(seed)|seed).filter(|seed|
                    seed.canonical_name==definition.internal_name&&offset.checked_add(seed.span.start)==Some(definition.start as usize)&&offset.checked_add(seed.span.end)==Some(definition.end as usize)).collect::<Vec<_>>();
                if candidates.len()!=1{return Err("E_DDL_PUBLIC_DEFINITION_OWNER".into());}
                bindings.push(ddonirang_lang::parser::PublicCallableBinding{definition:definition.identity.clone(),pin_identities:definition.pins.iter().map(|pin|pin.identity.clone()).collect(),
                    original:candidates[0].clone(),source_path:source_path.clone(),source_text:file.text.clone()});
            }
        }
        let compiler=ddonirang_lang::parser::RegisteredPublicCallableContext::from_public_and_ddl(profile,bindings,ddl_source,ddl_path,next_node)
            .map_err(|error|ddl_parse_failure(error,ddl_path,&mut diagnostic))?;
        Ok(PublicCallablePublicDdl{original:self.clone(),module_head:module.head.clone(),source_owner:std::sync::Arc::new(owned),registration:std::sync::Arc::new(registration),facts,compiler:std::sync::Arc::new(compiler)})
    }
}


// Dedicated successor transport. These DTOs never confer execution authority.
pub(crate) const PUBLIC_ARTIFACT_V4:&str="ddn.authored_project_artifact.v4";
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PublicCallableProfileRecordV4 {
    identity:String,version:String,registration_revision:u32,content_utf8:String,content_sha256:String,
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PublicCallableOriginalRecordV4 {
    profile:PublicCallableProfileRecordV4,original_project_json_utf8:String,
    original_module_id:String,original_source_path:String,original_start_identity_hex:String,
    original_source_capture_catalog_json_utf8:String,
    original_registration_resources:Vec<registration::RecordedRegistrationBlob>,ddl_source:FirstCallableDdlSource,
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PublicCallableBindingV4 {
    profile:PublicCallableProfileRecordV4,original_project_json_utf8:String,
    original_module_id:String,original_source_path:String,original_start_identity_hex:String,
    original_source_capture_catalog_json_utf8:String,
    original_registration_resources:Vec<registration::RecordedRegistrationBlob>,ddl_source:FirstCallableDdlSource,
    consumer_module_id:String,consumer_source_path:String,
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PublicCallableProjectArtifactV4 {
    #[serde(skip)]
    verified_edition:bool,
    schema:String,pub(crate) source_profile:String,pub(crate) source_snapshot:SourceSnapshot,
    pub(crate) authored_identity:AuthoredIdentity,public_callable_binding:PublicCallableBindingV4,content_sha256:String,
}
fn public_json<T:Serialize>(value:&T)->Result<String,String>{
    serde_json::to_string(&serde_json::to_value(value).map_err(|_|"E_DDL_V4_ENCODING")?).map_err(|_|"E_DDL_V4_ENCODING".into())
}
fn public_read<T:serde::de::DeserializeOwned>(raw:&str)->Result<T,String>{
    if raw.len()>8_000_000||raw.starts_with('\u{feff}'){return Err("E_DDL_V4_LIMIT".into());}
    let value:StrictValue=serde_json::from_str(raw).map_err(|_|"E_DDL_V4_FIELDS")?;
    check_v3_json_depth(&value.0)?;
    serde_json::from_value(value.0).map_err(|_|"E_DDL_V4_FIELDS".into())
}
fn ddl_parse_failure(error:ddonirang_lang::ParseError,path:&str,
    diagnostic:&mut Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
)->String{
    if matches!(error.j1_source_code(),"E_MULTILANG_UNREGISTERED_SURFACE"|"E_MULTILANG_FULLWIDTH_SYNTAX"|"E_DDL_PUBLIC_PIN_OWNER"){
        registration::logic_dependency::keep_parse_failure(error,path,diagnostic)
    }else{
        error.to_string()
    }
}

impl PublicCallableOriginalRecordV4 {
    /// Validate a draft next edition without granting storage or activation.
    /// The original record supplies all captures and owner material.
    fn revise_surfaces(mut self,version:&str,surfaces:&[PublicCallableSurfaceSpec],
        ddl_source:FirstCallableDdlSource, diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    )->Result<Self,String>{
        let previous=self.restore()?;
        let mut owner=ddonirang_lang::ddl::SuccessorRegistryOwner::default();
        let profile=owner.restore_recorded_exact(&self.profile.identity,&self.profile.version,self.profile.registration_revision,
            &self.profile.content_utf8,&self.profile.content_sha256).map_err(|e|e.to_string())?;
        let next=owner.revise_public_surfaces(&profile,version,checked_public_callable_surfaces(&previous.facts,surfaces)?)
            .map_err(|e|e.to_string())?;
        self.profile.version=next.version().into();
        self.profile.registration_revision=next.registration_revision();
        self.profile.content_utf8=String::from_utf8(next.content().to_vec()).map_err(|_|"E_DDL_PROFILE_UTF8")?;
        self.profile.content_sha256=next.content_sha256().into();
        self.ddl_source=ddl_source;
        if self.restore_edition_draft_diagnostic(diagnostic)?.facts!=previous.facts{return Err("E_DDL_ORIGINAL_STALE".into());}
        Ok(self)
    }

    pub(crate) fn from_export(export:&PublicCallablePublicDdl,path:&str)->Result<Self,String>{
        let original=export.recorded_original_material()?;
        if !export.original.authored_identity.modules.iter().any(|m|m.id.as_str()==original.module&&m.sources.iter().any(|p|p==path)){
            return Err("E_DDL_ORIGINAL_SOURCE_OWNER_REQUIRED".into());
        }
        Ok(Self{profile:PublicCallableProfileRecordV4{identity:original.profile.identity().into(),version:original.profile.version().into(),
            registration_revision:original.profile.registration_revision(),content_utf8:String::from_utf8(original.profile.content().to_vec()).map_err(|_|"E_DDL_PROFILE_UTF8")?,content_sha256:original.profile.content_sha256().into()},
            original_project_json_utf8:original.original_json,original_module_id:original.module,original_source_path:path.into(),
            original_start_identity_hex:hex::encode(original.start_identity),original_source_capture_catalog_json_utf8:original.captures,
            original_registration_resources:original.resources,ddl_source:FirstCallableDdlSource{path:original.ddl_path,text:original.ddl}})
    }
    pub(crate) fn save(&self)->Result<String,String>{public_json(self)}
    pub(crate) fn open(raw:&str)->Result<Self,String>{let record:Self=public_read(raw)?;record.restore()?;Ok(record)}
    pub(crate) fn restore(&self)->Result<PublicCallablePublicDdl,String>{
        if self.profile.registration_revision!=1{return Err("E_DDL_EDITION_HISTORY_REQUIRED".into());}
        self.restore_edition_draft()
    }

    fn restore_edition_draft(&self)->Result<PublicCallablePublicDdl,String>{
        self.restore_edition_draft_diagnostic(None)
    }

    fn restore_edition_draft_diagnostic(&self,
        diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    )->Result<PublicCallablePublicDdl,String>{
        if self.original_project_json_utf8.len()>1_048_576||self.original_source_capture_catalog_json_utf8.len()>1_000_000
            ||self.original_registration_resources.len()>64||self.ddl_source.text.len()>16_384||self.original_start_identity_hex.len()>8192{
            return Err("E_DDL_ORIGINAL_RECORD_LIMIT".into());
        }
        if self.original_start_identity_hex.len()%2!=0||!self.original_start_identity_hex.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){
            return Err("E_DDL_ORIGINAL_START_CARRIER".into());
        }
        let original=open(self.original_project_json_utf8.as_bytes())?;
        if !matches!(original.schema.as_str(),SCHEMA|SCHEMA_V2)||!original.authored_identity.modules.iter().any(|m|
            m.id.as_str()==self.original_module_id&&m.sources.iter().any(|p|p==&self.original_source_path)){
            return Err("E_DDL_ORIGINAL_SOURCE_OWNER_REQUIRED".into());
        }
        let mut owner=ddonirang_lang::ddl::SuccessorRegistryOwner::default();
        let profile=owner.restore_recorded_exact(&self.profile.identity,&self.profile.version,self.profile.registration_revision,
            &self.profile.content_utf8,&self.profile.content_sha256).map_err(|e|e.to_string())?;
        // Exact DTO roundtrip copies raw catalog transport without trusting it;
        // R27 reconstructs every capture/Definition/Pin correspondence below.
        let resources=serde_json::from_value(serde_json::to_value(&self.original_registration_resources).map_err(|_|"E_DDL_V4_ENCODING")?).map_err(|_|"E_DDL_V4_FIELDS")?;
        PublicCallablePublicDdl::restore_recorded_original_diagnostic(PublicCallableOriginalMaterial{original_json:self.original_project_json_utf8.clone(),module:self.original_module_id.clone(),
            start_identity:hex::decode(&self.original_start_identity_hex).map_err(|_|"E_DDL_ORIGINAL_START_CARRIER")?,captures:self.original_source_capture_catalog_json_utf8.clone(),resources,
            profile,ddl:self.ddl_source.text.clone(),ddl_path:self.ddl_source.path.clone()},diagnostic)
    }
}
impl PublicCallableProjectArtifactV4 {
    fn checksum(&self)->Result<String,String>{let mut value=serde_json::to_value(self).map_err(|_|"E_DDL_V4_ENCODING")?;
        value.as_object_mut().ok_or("E_DDL_V4_FIELDS")?.remove("content_sha256");
        Ok(hex::encode(Sha256::digest(serde_json::to_vec(&value).map_err(|_|"E_DDL_V4_ENCODING")?)))}
    pub(crate) fn save(&self)->Result<String,String>{
        if self.schema!=PUBLIC_ARTIFACT_V4||self.checksum()?!=self.content_sha256{return Err("E_DDL_V4_CHECKSUM".into());}
        let raw=public_json(self)?;if raw.len()>8_000_000{return Err("E_DDL_V4_LIMIT".into());}Ok(raw)
    }
    pub(crate) fn open(raw:&str)->Result<std::sync::Arc<Self>,String>{
        let artifact:Self=public_read(raw)?;
        if artifact.save()?!=raw{return Err("E_DDL_V4_CANONICAL".into());}
        artifact.restore_context()?;Ok(std::sync::Arc::new(artifact))
    }
    /// In-memory source/identity view for the existing compiler/issuer only.
    /// The durable owner and bytes always remain the distinct v4 artifact.
    /// This view is never emitted/accepted as a legacy saved artifact.
    pub(crate) fn source_view(&self)->Result<ProjectArtifact,String>{
        let schema=match self.authored_identity.schema.as_str(){
            "ddn.authored_identity.v1"=>SCHEMA,"ddn.authored_identity.v2"=>SCHEMA_V2,
            _=>return Err("E_DDL_V4_IDENTITY_SCHEMA".into()),
        };
        let mut source=ProjectArtifact{schema:schema.into(),source_profile:self.source_profile.clone(),source_snapshot:self.source_snapshot.clone(),
            authored_identity:self.authored_identity.clone(),content_sha256:String::new(),language_binding:None};
        source.content_sha256=source.checksum()?;source.validate()?;Ok(source)
    }
    /// Tagged structural view for existing issuer consumers. It intentionally
    /// cannot be serialized by the legacy ProjectArtifact writer: unsupported
    /// old frontdoors must reject v4 rather than silently drop its binding.
    pub(crate) fn owner_view(&self)->Result<ProjectArtifact,String>{
        let mut view=self.source_view()?;view.schema=PUBLIC_ARTIFACT_V4.into();view.content_sha256=self.content_sha256.clone();Ok(view)
    }
    fn original_record(&self)->Result<PublicCallableOriginalRecordV4,String>{
        let mut value=serde_json::to_value(&self.public_callable_binding).map_err(|_|"E_DDL_V4_ENCODING")?;
        let fields=value.as_object_mut().ok_or("E_DDL_V4_FIELDS")?;fields.remove("consumer_module_id");fields.remove("consumer_source_path");
        serde_json::from_value(value).map_err(|_|"E_DDL_V4_FIELDS".into())
    }
    pub(crate) fn restore_context(&self)->Result<std::sync::Arc<PublicCallableAuthoredContext>,String>{
        if self.schema!=PUBLIC_ARTIFACT_V4||self.checksum()?!=self.content_sha256{return Err("E_DDL_V4_CHECKSUM".into());}
        let record=self.original_record()?;
        let export=if self.verified_edition {record.restore_edition_draft()?}else{record.restore()?};
        let consumer=self.source_view()?;
        Ok(std::sync::Arc::new(export.activate_for_consumer(&consumer,&self.public_callable_binding.consumer_module_id,&self.public_callable_binding.consumer_source_path)?))
    }
    pub(crate) fn create(record:PublicCallableOriginalRecordV4,consumer:&ProjectArtifact,path:&str,
        diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<std::sync::Arc<Self>,String>{
        consumer.validate()?;if !matches!(consumer.schema.as_str(),SCHEMA|SCHEMA_V2)||consumer.content_sha256!=consumer.checksum()?{
            return Err("E_DDL_CONSUMER_STALE".into());
        }
        let export=record.restore()?;let modules=consumer.authored_identity.modules.iter().filter(|m|m.sources.iter().any(|p|p==path)).collect::<Vec<_>>();
        if modules.len()!=1{return Err("E_DDL_CONSUMER_SOURCE_OWNER_REQUIRED".into());}
        export.activate_for_consumer_diagnostic(consumer,modules[0].id.as_str(),path,diagnostic)?;
        let mut binding=serde_json::to_value(record).map_err(|_|"E_DDL_V4_ENCODING")?;
        let fields=binding.as_object_mut().ok_or("E_DDL_V4_FIELDS")?;
        fields.insert("consumer_module_id".into(),modules[0].id.as_str().into());fields.insert("consumer_source_path".into(),path.into());
        let mut artifact=Self{verified_edition:false,schema:PUBLIC_ARTIFACT_V4.into(),source_profile:consumer.source_profile.clone(),source_snapshot:consumer.source_snapshot.clone(),authored_identity:consumer.authored_identity.clone(),
            public_callable_binding:serde_json::from_value(binding).map_err(|_|"E_DDL_V4_FIELDS")?,content_sha256:String::new()};
        artifact.content_sha256=artifact.checksum()?;artifact.restore_context()?;Ok(std::sync::Arc::new(artifact))
    }
    /// Replay the spelling edit from a separately verified parent. The raw
    /// child cannot grant itself permission to use the edition reader.
    pub(crate) fn open_spelling_successor(parent:&Self,raw:&str)->Result<std::sync::Arc<Self>,String>{
        let supplied:Self=public_read(raw)?;
        if supplied.save()?!=raw{return Err("E_DDL_V4_CANONICAL".into());}
        let before=parent.restore_context()?;
        let record=supplied.original_record()?;
        let restored=record.restore_edition_draft()?;
        if restored.facts!=before.export.facts{return Err("E_DDL_EDITION_ORIGINAL_CHANGED".into());}
        let surfaces=Self::edition_surfaces(&before,&restored)?;
        let text=supplied.source_snapshot.files.iter().find(|file|file.path==parent.consumer_path())
            .ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?.text.clone();
        let mut checked=parent.check_spelling_edition(&record.profile.version,&surfaces,record.ddl_source,text,None)?;
        if checked.save()?!=raw{return Err("E_DDL_EDITION_PARENT_MISMATCH".into());}
        checked.verified_edition=true;
        Ok(std::sync::Arc::new(checked))
    }

    fn edition_surfaces(before:&PublicCallableAuthoredContext, restored:&PublicCallablePublicDdl)
        ->Result<Vec<PublicCallableSurfaceSpec>,String>{
        before.export.facts.definitions.iter().map(|definition| {
            let surface=restored.profile().surfaces().iter().find(|row|row.identity==definition.identity)
                .ok_or("E_DDL_PUBLIC_COVERAGE")?;
            let pins=definition.pins.iter().map(|pin|{
                let row=surface.pins.iter().find(|row|row.identity==pin.identity).ok_or("E_DDL_PUBLIC_PIN_OWNER")?;
                Ok((pin.name.clone(),row.surface.clone(),row.particle.clone()))
            }).collect::<Result<Vec<_>,String>>()?;
            Ok(PublicCallableSurfaceSpec{public_name:definition.public_name.clone(),surface:surface.surface.clone(),pins})
        }).collect()
    }

    #[cfg(windows)]
    pub(crate) fn edition_authoring_view(&self)->Result<Value,String>{
        let context=self.restore_context()?;
        let source=self.source_snapshot.files.iter().find(|file|file.path==self.consumer_path())
            .ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?;
        let surfaces=Self::edition_surfaces(&context,&context.export)?;
        let original_files=context.export.original.source_snapshot.files.iter().map(|file|{
            serde_json::json!({
                "path":file.path,"bytes":file.text.len(),
                "sha256":hex::encode(Sha256::digest(file.text.as_bytes())),
            })
        }).collect::<Vec<_>>();
        Ok(serde_json::json!({
            "original_files":original_files,
            "edition":self.public_callable_binding.profile.version,
            "ddl_source":self.public_callable_binding.ddl_source,
            "consumer_path":source.path,"consumer_source_utf8":source.text,"surfaces":surfaces,
        }))
    }

    /// Build an edition from this exact parent. The caller must still prove
    /// that the parent is current in the selected durable store.
    fn check_spelling_edition(&self, version:&str, surfaces:&[PublicCallableSurfaceSpec],
        ddl:FirstCallableDdlSource, consumer_text:String,
        mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    )->Result<PublicCallableProjectArtifactV4,String>{
        let parent=self.restore_context()?;
        let record=self.original_record()?.revise_surfaces(version,surfaces,ddl,diagnostic.as_deref_mut())?;
        let export=record.restore_edition_draft()?;
        let path=self.consumer_path();
        let module=&self.public_callable_binding.consumer_module_id;
        let mut consumer=self.source_view()?;
        let file=consumer.source_snapshot.files.iter_mut().find(|file|file.path==path)
            .ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?;
        let previous=std::mem::replace(&mut file.text,consumer_text);
        consumer.content_sha256=consumer.checksum()?;
        let (_,after)=export.activate_for_consumer_selected(&consumer,module,path,false,diagnostic)?;
        let before=ddonirang_lang::parser::Parser::parse_with_public_callable_context(
            &previous,path,parent.compiler_context().clone(),
        ).map_err(|error|error.to_string())?;
        let level=ddonirang_lang::NormalizationLevel::N1;
        if ddonirang_lang::normalize(&before,level)!=ddonirang_lang::normalize(&after,level){
            return Err("E_DDL_EDITION_CONSUMER_MEANING_CHANGED".into());
        }
        let mut binding=serde_json::to_value(record).map_err(|_|"E_DDL_V4_ENCODING")?;
        let fields=binding.as_object_mut().ok_or("E_DDL_V4_FIELDS")?;
        fields.insert("consumer_module_id".into(),module.clone().into());
        fields.insert("consumer_source_path".into(),path.into());
        let mut draft=Self{verified_edition:false,schema:PUBLIC_ARTIFACT_V4.into(),source_profile:consumer.source_profile,
            source_snapshot:consumer.source_snapshot,authored_identity:consumer.authored_identity,
            public_callable_binding:serde_json::from_value(binding).map_err(|_|"E_DDL_V4_FIELDS")?,content_sha256:String::new()};
        draft.content_sha256=draft.checksum()?;
        Ok(draft)
    }

    pub(crate) fn dependency_material(&self)->Result<Value,String>{serde_json::to_value(&self.public_callable_binding).map_err(|_|"E_DDL_V4_ENCODING".into())}
    pub(crate) fn consumer_path(&self)->&str{&self.public_callable_binding.consumer_source_path}
}
impl fmt::Debug for PublicCallableAuthoredContext {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.debug_struct("PublicCallableAuthoredContext").field("path",&self.path).finish_non_exhaustive()}
}
impl fmt::Debug for PublicCallableProjectArtifactV4 {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.debug_struct("PublicCallableProjectArtifactV4").field("checksum",&self.content_sha256).finish_non_exhaustive()}
}

/// Existing project/module owners plus an explicitly supplied DDL compilation
/// context. It has no serde/wire representation or durable admission power.
#[derive(Clone)]
pub struct FirstCallableAuthoredContext {
    original: ProjectArtifact,
    module_index: usize,
    source_index: usize,
    declaration_offset: usize,
    compiler: std::sync::Arc<ddonirang_lang::parser::RegisteredFirstCallableContext>,
    public_export:Option<std::sync::Arc<FirstCallablePublicDdl>>,
    consumer_owner:Option<(std::sync::Arc<ProjectArtifact>,String,String)>,
}

/// UTF-8 byte, half-open range in the verified original artifact's raw file.
pub struct FirstCallableOriginalSourceSpan<'a> {
    pub source_path: &'a str,
    pub source_text: &'a str,
    pub span: ddonirang_lang::Span,
}

impl std::fmt::Debug for FirstCallableAuthoredContext {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        f.debug_struct("FirstCallableAuthoredContext").field("original_project",self.project_identity()).field("original_module",self.module_identity()).finish_non_exhaustive()
    }
}
impl FirstCallableAuthoredContext {
    pub(crate) fn has_legacy_ddl_original(&self)->bool {
        self.public_export.as_ref().and_then(|export|export.owner.registration())
            .is_some_and(|registration|registration.has_legacy_ddl_start())
    }
    pub(crate) fn checked_native_source<'a>(&'a self,source:&crate::local_project::VirtualProjectSource)->Result<(&'a str,&'a str),String>{
        let (consumer,_,path)=self.consumer_owner.as_ref().ok_or("E_DDL_CONSUMER_OWNER_REQUIRED")?;
        if consumer.schema!=SCHEMA_V3||consumer.source_snapshot.root_path.is_none()
            ||consumer.authored_identity.modules.len()!=1||consumer.authored_identity.modules[0].sources.as_slice()!=[path.clone()]{return Err("E_DDL_NATIVE_SINGLE_ENTRY_SCOPE".into());}
        let expected=snapshot_source(&consumer.source_snapshot)?;
        let members=|s:&crate::local_project::VirtualProjectSource|s.files.iter().map(|f|(f.path.clone(),(f.role.clone(),f.utf8_text.clone()))).collect::<BTreeMap<_,_>>();
        if expected.schema!=source.schema||expected.kind.as_deref().unwrap_or(crate::local_project::LOCAL_PROJECT_TREE_KIND)!=source.kind.as_deref().unwrap_or(crate::local_project::LOCAL_PROJECT_TREE_KIND)||expected.package_hash!=source.package_hash||expected.root_path!=source.root_path||expected.entry_path!=source.entry_path
            ||expected.files.len()!=source.files.len()||members(&expected)!=members(source){return Err("E_DDL_CONSUMER_STALE".into());}
        let text=&consumer.source_snapshot.files.iter().find(|file|&file.path==path).ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?.text;
        self.verify_runtime_activation(text,path)?;
        Ok((path,text))
    }
    /// Storage owns only declarations authored in the consumer. The imported
    /// function stays with its original recorded owner and is supplied to the
    /// runtime separately; it must not receive a consumer module identity.
    pub(crate) fn parse_consumer_owned_program(&self,source:&str,path:&str)->Result<ddonirang_lang::CanonProgram,String>{
        self.verify_runtime_activation(source,path)?;
        let mut program=ddonirang_lang::parser::Parser::parse_with_first_callable_context(source,path,self.compiler.clone()).map_err(|e|e.to_string())?;
        let imported=&self.compiler.original_public().declaration_program.as_ref().ok_or("E_DDL_ORIGINAL_DECLARATION_REQUIRED")?.items[0];
        let ddonirang_lang::TopLevelItem::SeedDef(imported)=imported;
        program.items.retain(|ddonirang_lang::TopLevelItem::SeedDef(seed)|seed.canonical_name!=imported.canonical_name);
        Ok(program)
    }
    pub(crate) fn saved_dependency_material(&self)->Result<Value,String>{
        let export=self.public_export.as_ref().ok_or("E_DDL_PUBLIC_EXPORT_REQUIRED")?;
        Ok(serde_json::json!({"module":export.facts.module,"capture":export.facts.capture,"definition":export.facts.definition,
            "pins":export.facts.pins,"result_type":export.facts.result_type,"profile":export.profile.identity(),"version":export.profile.version(),
            "profile_content_sha256":export.profile.content_sha256(),"registration_revision":export.profile.registration_revision()}))
    }

    /// Resolve a consumer's declared module locator against this exact original
    /// owner. This preflight grants neither profile nor exported-DDL admission.
    pub fn check_module_translation_request(&self,source:&str,path:&str)->Result<(),String>{
        self.original.validate()?;
        if self.original.content_sha256!=self.original.checksum()? {return Err("E_MULTILANG_ORIGINAL_CHECKSUM_STALE".into());}
        let module=&self.original.authored_identity.modules[self.module_index];
        let resolved=crate::local_project::resolve_first_callable_module_translation_request(
            source,path,&snapshot_source(&self.original.source_snapshot)?,&module.head,self.compiler_context()).map_err(|e|format!("{}: {}",e.code,e.message))?;
        if resolved.closure!=module.sources || resolved.head!=module.head {return Err("E_MULTILANG_ORIGINAL_SOURCE_OWNER_REQUIRED".into());}
        // Exact registry and exported DDL are not supplied by the R8 compiler
        // context. Never reinterpret its placeholder as durable authority.
        if let Some(export)=&self.public_export {
            let profile=ddonirang_lang::ddl::resolve_first_callable_profile_name(&resolved.request.target_language,&resolved.request.target_version).map_err(|e|e.to_string())?;
            if export.facts.module!=self.module_identity().as_str()||export.original.original!=self.original
                ||profile.identity()!=export.profile.identity()||profile.version()!=export.profile.version(){return Err("E_DDL_ACTIVATION_OWNER_OR_PROFILE".into());}
            return Ok(());
        }
        Err("E_DDL_PROFILE_AND_EXPORTED_OWNER_REQUIRED: 실제 말씨 정체·정확 판본과 원본 모듈의 공개 DDL 공급이 필요합니다".into())
    }
    /// Explicit original-owner export operation. Reading DDL text by itself
    /// never calls this G1 owner projection or creates public export authority.
    pub fn produce_public_ddl(&self,original_json:&str,original_source_artifact:&str)->Result<FirstCallablePublicDdl,String>{
        if self.public_export.is_some(){return Err("E_DDL_ORIGINAL_EXPORT_CONTEXT_REQUIRED".into());}
        if original_json.len()>1_048_576||open(original_json.as_bytes())?!=self.original {return Err("E_DDL_ORIGINAL_RECORD_MISMATCH".into());}
        let source=crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision::from_artifact_with_first_callable(original_source_artifact,self)?;
        let owner=source.recorded_execution_owner()?;
        if owner.project()!=&self.original{return Err("E_DDL_ORIGINAL_OWNER_MISMATCH".into());}
        let draft=owner.registration().ok_or("E_DDL_ORIGINAL_G1_CATALOG_REQUIRED")?;
        let facts=draft.checked_first_callable_export(&owner.precanon_view(),self)?;
        if facts.module!=self.module_identity().as_str(){return Err("E_DDL_EXPORTED_MODULE_OWNER_MISMATCH".into());}
        let profile=ddonirang_lang::ddl::issued_first_callable_profile().map_err(|e|e.to_string())?;
        if hex::encode(Sha256::digest(profile.content()))!=profile.content_sha256(){return Err("E_DDL_PROFILE_CONTENT_HASH".into());}
        Ok(FirstCallablePublicDdl{original:self.clone(),original_json:original_json.into(),owner:std::sync::Arc::new(owner),facts,profile})
    }
    pub(crate) fn verify_runtime_activation(&self,source:&str,path:&str)->Result<(),String>{
        if self.public_export.is_none(){return Ok(());}
        let (consumer,module_id,selected)=self.consumer_owner.as_ref().ok_or("E_DDL_CONSUMER_OWNER_REQUIRED")?;
        if selected!=path || !consumer.authored_identity.modules.iter().any(|m|m.id.as_str()==module_id&&m.sources.iter().any(|p|p==path))
            || !consumer.source_snapshot.files.iter().any(|f|f.path==path&&f.text.as_bytes()==source.as_bytes()){
            return Err("E_DDL_CONSUMER_STALE: 실행 원문과 확인된 소비 작품·원문이 다릅니다".into());
        }
        self.check_module_translation_request(source,path)
    }
    pub(crate) fn registration_record_for_consumer(&self,project:&ProjectArtifact)->Result<FirstCallableOriginalRegistration,String>{
        let (consumer,_,path)=self.consumer_owner.as_ref().ok_or("E_DDL_CONSUMER_OWNER_REQUIRED")?;
        if consumer.as_ref()!=project||project.schema!=SCHEMA_V3{return Err("E_DDL_CONSUMER_STALE".into());}
        let source=project.source_snapshot.files.iter().find(|f|&f.path==path).ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?;
        self.verify_runtime_activation(&source.text,path)?;
        self.public_export.as_ref().ok_or("E_DDL_PUBLIC_EXPORT_REQUIRED")?.original_registration_record()
    }
    pub fn project_identity(&self) -> &ProjectId { &self.original.authored_identity.logical_dependency_id }
    pub fn module_identity(&self) -> &ModuleId { &self.original.authored_identity.modules[self.module_index].id }
    pub fn original_artifact(&self) -> &ProjectArtifact { &self.original }
    pub fn compiler_context(&self) -> &std::sync::Arc<ddonirang_lang::parser::RegisteredFirstCallableContext> { &self.compiler }
    fn original_span(&self, span: ddonirang_lang::Span) -> FirstCallableOriginalSourceSpan<'_> {
        let source = &self.original.source_snapshot.files[self.source_index];
        FirstCallableOriginalSourceSpan { source_path: &source.path, source_text: &source.text,
            span: ddonirang_lang::Span::new(self.declaration_offset + span.start, self.declaration_offset + span.end) }
    }
    pub fn definition_source_span(&self) -> FirstCallableOriginalSourceSpan<'_> {
        let ddonirang_lang::TopLevelItem::SeedDef(seed) = &self.compiler.original_public().declaration_program.as_ref().unwrap().items[0];
        self.original_span(seed.span)
    }
    pub fn pin_source_span(&self, surface: &str) -> Option<FirstCallableOriginalSourceSpan<'_>> {
        let r = ddonirang_lang::ddl::FIRST_CALLABLE_JA_REGISTRATION;
        let binding = ddonirang_lang::ddl::bind_first_callable_ja_declaration(self.compiler.original_public(),
            r.language_profile_identity, r.registration_revision).ok()?;
        binding.pin(surface).map(|pin| self.original_span(pin.span))
    }
}

impl FirstCallablePublicDdl {
    pub fn attach_v3(&self,consumer:&ProjectArtifact,module_id:&str,path:&str)->Result<ProjectArtifact,String>{
        consumer.validate()?;
        if consumer.schema!=SCHEMA || consumer.authored_identity.schema!="ddn.authored_identity.v1"
            || consumer.authored_identity.entry.is_some()||consumer.authored_identity.modules.iter().any(|m|!m.compositions.is_empty()||!m.bindings.is_empty()){
            return Err("E_DDL_V3_CONSUMER_UNSUPPORTED: 첫 v3는 빈 보개 identity.v1 작품만 받습니다".into());
        }
        self.activate_for_consumer(consumer,module_id,path)?;
        let mut project=consumer.clone();project.schema=SCHEMA_V3.into();
        project.language_binding=Some(FirstCallableLanguageBinding{
            profile_identity:self.profile.identity().into(),profile_version:self.profile.version().into(),profile_content_sha256:self.profile.content_sha256().into(),
            registration_revision:self.profile.registration_revision(),original_project_json_utf8:self.original_json.clone(),original_module_id:self.facts.module.clone(),
            original_source_path:self.original.definition_source_span().source_path.into(),ddl_source:FirstCallableDdlSource{path:self.original.compiler.ddl_path().into(),text:self.original.compiler.ddl_source().into()},
            consumer_module_id:module_id.into(),consumer_source_path:path.into(),
        });
        project.content_sha256=project.checksum()?;project.validate()?;Ok(project)
    }
    pub(crate) fn original_registration_record(&self)->Result<FirstCallableOriginalRegistration,String>{
        Ok(FirstCallableOriginalRegistration{start_identity_hex:hex::encode(self.owner.definition_identity().as_bytes()),
            capture_catalog_json_utf8:self.owner.sources().recorded_capture_catalog_json()?,
            resources:self.owner.registration().ok_or("E_DDL_ORIGINAL_G1_CATALOG_REQUIRED")?.recorded_blobs()})
    }
    pub fn profile(&self)->&ddonirang_lang::ddl::CheckedFirstCallableProfile{&self.profile}
    pub fn definition_identity(&self)->&str{&self.facts.definition}
    pub fn pin_identities(&self)->[&str;2]{[&self.facts.pins[0].1,&self.facts.pins[1].1]}
    pub fn original_module_identity(&self)->&str{&self.facts.module}
    pub fn activate_for_consumer(&self,consumer:&ProjectArtifact,module_id:&str,path:&str)->Result<FirstCallableAuthoredContext,String>{
        consumer.validate()?;
        if consumer.content_sha256!=consumer.checksum()?{return Err("E_DDL_CONSUMER_CHECKSUM_STALE".into());}
        let module=consumer.authored_identity.modules.iter().find(|m|m.id.as_str()==module_id).ok_or("E_DDL_CONSUMER_MODULE_REQUIRED")?;
        if !module.sources.iter().any(|p|p==path){return Err("E_DDL_CONSUMER_SOURCE_OWNER_REQUIRED".into());}
        let file=consumer.source_snapshot.files.iter().find(|f|f.path==path).ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?;
        let resolved=crate::local_project::resolve_first_callable_module_translation_request(&file.text,path,
            &snapshot_source(&self.original.original.source_snapshot)?,&self.original.original.authored_identity.modules[self.original.module_index].head,
            &std::sync::Arc::new((**self.original.compiler_context()).clone().with_prefix_profile(self.profile.clone())))
            .map_err(|e|format!("{}: {}",e.code,e.message))?;
        let profile=ddonirang_lang::ddl::resolve_first_callable_profile_name(&resolved.request.target_language,&resolved.request.target_version).map_err(|e|e.to_string())?;
        if profile.identity()!=self.profile.identity()||profile.version()!=self.profile.version(){return Err("E_DDL_ACTIVATION_OWNER_OR_PROFILE".into());}
        let mut context=self.original.clone();
        context.compiler=std::sync::Arc::new((*context.compiler).clone().with_module_activation(profile,resolved.binding).map_err(|e|e.to_string())?);
        context.public_export=Some(std::sync::Arc::new(self.clone()));
        context.consumer_owner=Some((std::sync::Arc::new(consumer.clone()),module_id.into(),path.into()));
        context.check_module_translation_request(&file.text,path)?;
        Ok(context)
    }
}

#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FirstCallableOriginalRegistration {
    start_identity_hex:String,capture_catalog_json_utf8:String,
    resources:Vec<registration::RecordedRegistrationBlob>,
}
fn check_v3_json_depth(value:&Value)->Result<(),String>{
    let mut stack=vec![(value,1usize)];
    while let Some((value,depth))=stack.pop(){if depth>32{return Err("E_DDL_V3_DEPTH".into());}
        match value{Value::Array(items)=>stack.extend(items.iter().map(|v|(v,depth+1))),Value::Object(items)=>stack.extend(items.values().map(|v|(v,depth+1))),_=>{}}
    }Ok(())
}
fn restore_original_public_ddl(binding:&FirstCallableLanguageBinding,record:FirstCallableOriginalRegistration)->Result<FirstCallablePublicDdl,String>{
    use crate::project_revision_envelope_supply as supply;
    let original=open(binding.original_project_json_utf8.as_bytes())?;
    if !matches!(original.schema.as_str(),SCHEMA|SCHEMA_V2){return Err("E_DDL_NESTED_ORIGINAL_FORBIDDEN".into());}
    let first=original.first_callable_registration_context(&binding.original_module_id,&binding.original_source_path,&binding.ddl_source.text,&binding.ddl_source.path,
        ddonirang_lang::ddl::FIRST_CALLABLE_JA_REGISTRATION.language_profile_identity,binding.registration_revision)?;
    if record.start_identity_hex.len()>8192||record.capture_catalog_json_utf8.len()>1_000_000||record.resources.len()>64{return Err("E_DDL_ORIGINAL_RECORD_LIMIT".into());}
    let bytes=hex::decode(&record.start_identity_hex).map_err(|_|"E_DDL_ORIGINAL_START_CARRIER")?;
    let carrier=supply::CanonicalIdentityBytesV1::from_owner_bytes(&bytes).map_err(|_|"E_DDL_ORIGINAL_START_CARRIER")?;
    let snapshot=&original.source_snapshot;let root=snapshot.root_path.as_deref().ok_or("E_DDL_ORIGINAL_ROOT_REQUIRED")?;
    let roles=supply::saved_authored_source_roles(snapshot)?;
    let blobs=snapshot.files.iter().map(|f|Ok(supply::SavedSourceBlobV1{path:&f.path,role:roles.get(&f.path).ok_or("E_DDL_ORIGINAL_SOURCE_ROLE")?,bytes:f.text.as_bytes()})).collect::<Result<Vec<_>,String>>()?;
    let saved=supply::collect_saved_source_supply(root,&snapshot.entry_path,&carrier,&blobs).map_err(|_|"E_DDL_ORIGINAL_SOURCE_SUPPLY")?;
    let sources=std::sync::Arc::new(supply::restore_saved_v25_sources(saved,&record.capture_catalog_json_utf8)?);
    let source=supply::admit_precanon_authored_source(sources,binding.original_project_json_utf8.as_bytes())?;
    let draft=registration::restore_precanon_first_callable_candidate(&source,&first,record.resources)?;
    let facts=draft.checked_first_callable_export(&source,&first)?;
    let owner=supply::finalize_registered_authored_source(source,draft)?;
    if owner.definition_identity().as_bytes()!=bytes.as_slice(){return Err("E_DDL_ORIGINAL_START_MISMATCH".into());}
    let profile=ddonirang_lang::ddl::lookup_first_callable_profile(&binding.profile_identity,&binding.profile_version,&binding.profile_content_sha256,binding.registration_revision).map_err(|e|e.to_string())?;
    if hex::encode(Sha256::digest(profile.content()))!=profile.content_sha256(){return Err("E_DDL_PROFILE_CONTENT_HASH".into());}
    Ok(FirstCallablePublicDdl{original:first,original_json:binding.original_project_json_utf8.clone(),owner:std::sync::Arc::new(owner),facts,profile})
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateRequest {
    source_profile: String,
    source_snapshot: SourceSnapshot,
    module_name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateRequest {
    original_json: String,
    source_snapshot: SourceSnapshot,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegisterBogaeRequest {
    original_json: String,
    module_name: String,
    source_path: String,
    composition_definition: String,
    make_entry: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RenameRequest {
    original_json: String,
    owner_module_id: String,
    target_kind: String,
    target_id: String,
    new_name: String,
    source_snapshot: SourceSnapshot,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RelocateRequest {
    original_json: String,
    old_path: String,
    new_path: String,
    source_snapshot: SourceSnapshot,
}

fn fail(message: &str) -> String {
    format!("E_AUTHORED_PROJECT: {message}")
}
fn locator(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':', '\0'])
        && !path.starts_with('/')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn supports_authored_source_profile(profile: &str) -> bool {
    matches!(
        profile,
        "v1-core-v25" | "v1-core-v25-logical-alpha2" | ddonirang_lang::ddl::J1_SOURCE_PROFILE
    )
}

impl ProjectArtifact {
    fn validate(&self) -> Result<(), String> {self.validate_diagnostic(None)}
    fn validate_diagnostic(&self,diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>) -> Result<(), String> {
        if self.schema!=SCHEMA_V3 && self.language_binding.is_some(){return Err("E_DDL_BINDING_SCHEMA: 기존 판본은 언어 결속 필드를 받지 않습니다".into());}
        if !matches!(
            (self.schema.as_str(), self.authored_identity.schema.as_str()),
            (SCHEMA, "ddn.authored_identity.v1") | (SCHEMA_V2, "ddn.authored_identity.v2") | (SCHEMA_V3,"ddn.authored_identity.v1")
        ) {
            return Err(fail("지원하지 않는 저장 버전입니다"));
        }
        if !supports_authored_source_profile(&self.source_profile) {
            return Err(fail("문법 판을 명시해야 합니다"));
        }
        if self.schema==SCHEMA_V3 {
            if self.authored_identity.entry.is_some()||self.authored_identity.modules.iter().any(|m|!m.compositions.is_empty()||!m.bindings.is_empty()){
                return Err("E_DDL_V3_CONSUMER_UNSUPPORTED: 보개·v2 소비 작품의 attach/upgrade는 지원하지 않습니다".into());
            }
            self.validate_language_binding()?;
        }
        let identity = &self.authored_identity;
        if !identity.logical_dependency_id.valid()
            || identity.clone_origin.as_ref().is_some_and(|id| !id.valid())
        {
            return Err(fail("작품 정체가 올바르지 않습니다"));
        }
        let mut files = BTreeSet::new();
        for file in &self.source_snapshot.files {
            if !locator(&file.path)
                || !files.insert(&file.path)
                || file.text.starts_with('\u{feff}')
            {
                return Err(fail("원문 경로 중복·이탈 또는 BOM이 있습니다"));
            }
        }
        if !files.contains(&self.source_snapshot.entry_path) {
            return Err(fail("실행 대상 원문이 없습니다"));
        }
        let mut owned = BTreeSet::new();
        let mut modules = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut compositions = BTreeSet::new();
        for module in &identity.modules {
            if !module.id.valid()
                || !modules.insert(&module.id)
                || module.name.is_empty()
                || !names.insert(&module.name)
            {
                return Err(fail("모듈 정체 또는 작성 이름이 중복·누락되었습니다"));
            }
            if module.sources.is_empty() || !module.sources.contains(&module.head) {
                return Err(fail("모듈 머리는 소유 원문이어야 합니다"));
            }
            for path in &module.sources {
                if !files.contains(path) || !owned.insert(path) {
                    return Err(fail("원문 소유가 중복되거나 파일이 없습니다"));
                }
            }
            let mut bindings = BTreeSet::new();
            let mut definitions = BTreeSet::new();
            for composition in &module.compositions {
                if !composition.id.valid()
                    || !compositions.insert((&module.id, &composition.id))
                    || composition.definition.is_empty()
                    || !definitions.insert(&composition.definition)
                {
                    return Err(fail("짜임 정체 또는 정의 참조가 중복·누락되었습니다"));
                }
                for binding in &composition.binding_ids {
                    if !binding.valid() || (self.schema == SCHEMA && !bindings.insert(binding)) {
                        return Err(fail("작성 결속 정체가 중복·누락되었습니다"));
                    }
                }
            }
            crate::authored_bogae::validate_module_storage(self, module)?;
        }
        if identity.modules.is_empty() {
            return Err(fail("명시 작품에는 모듈 정체가 필요합니다"));
        }
        let mut expected_owned = files.clone();
        if let Some(root_path) = &self.source_snapshot.root_path {
            let graph = snapshot_graph_diagnostic(&self.source_snapshot, &self.source_profile,diagnostic_out)?;
            if owned.contains(root_path) {
                return Err(fail("작품 뿌리를 모듈 원문으로 소유할 수 없습니다"));
            }
            expected_owned.remove(root_path);
            // Package custody is sealed by source_snapshot and verified by the
            // project graph, but is not an invented executable module source.
            let syntax=if self.source_profile==ddonirang_lang::ddl::J1_SOURCE_PROFILE{
                Some(ddonirang_lang::ddl::CheckedJ1Profile::resolve(&self.source_profile).map_err(|e|e.to_string())?)
            }else{None};
            for file in snapshot_source_selected(&self.source_snapshot,syntax)?.files {
                if matches!(file.role.as_str(), "metadata" | "gaji_manifest" | "lock") {
                    expected_owned.remove(&file.path);
                }
            }
            if graph.module_paths.len() != identity.modules.len() {
                return Err(fail("작성 정체와 얽음의 모듈 수가 다릅니다"));
            }
            for module in &identity.modules {
                if graph.module_paths.get(&module.name) != Some(&module.head)
                    || graph
                        .module_sources
                        .get(&module.name)
                        .map(|paths| paths.iter().collect::<BTreeSet<_>>())
                        != Some(module.sources.iter().collect())
                {
                    return Err(fail(
                        "작성 모듈 소유와 얽음 선언이 다릅니다. 정체를 추정하지 않습니다",
                    ));
                }
            }
        } else if identity.modules.len() != 1 {
            return Err(fail("여러 모듈에는 명시 얽음 뿌리가 필요합니다"));
        }
        if owned != expected_owned {
            return Err(fail("소유 모듈이 없는 원문이 있습니다"));
        }
        if let Some(entry) = &identity.entry {
            if !compositions.contains(&(&entry.module, &entry.composition)) {
                return Err(fail("보개 시작 정체를 해소할 수 없습니다"));
            }
        }
        Ok(())
    }

    fn validate_language_binding(&self)->Result<(),String>{
        let binding=self.language_binding.as_ref().ok_or("E_DDL_V3_BINDING_REQUIRED")?;
        if binding.original_project_json_utf8.len()>1_048_576||binding.ddl_source.text.len()>16_384
            ||self.source_snapshot.files.len()>64||self.source_snapshot.files.iter().any(|f|f.text.len()>262_144)
            ||self.source_snapshot.files.iter().map(|f|f.text.len()).sum::<usize>()>2_097_152{return Err("E_DDL_V3_LIMIT".into());}
        let profile=ddonirang_lang::ddl::lookup_first_callable_profile(&binding.profile_identity,&binding.profile_version,&binding.profile_content_sha256,binding.registration_revision).map_err(|e|e.to_string())?;
        if hex::encode(Sha256::digest(profile.content()))!=profile.content_sha256(){return Err("E_DDL_PROFILE_CONTENT_HASH".into());}
        let inner:StrictValue=serde_json::from_str(&binding.original_project_json_utf8).map_err(|_|"E_DDL_ORIGINAL_FIELDS")?;
        check_v3_json_depth(&inner.0)?;
        if !matches!(inner.0.get("schema").and_then(Value::as_str),Some(SCHEMA)|Some(SCHEMA_V2))
            ||inner.0.as_object().is_some_and(|o|o.contains_key("language_binding")){return Err("E_DDL_NESTED_ORIGINAL_FORBIDDEN".into());}
        let original=open(binding.original_project_json_utf8.as_bytes())?;
        let first=original.first_callable_registration_context(&binding.original_module_id,&binding.original_source_path,&binding.ddl_source.text,&binding.ddl_source.path,
            ddonirang_lang::ddl::FIRST_CALLABLE_JA_REGISTRATION.language_profile_identity,binding.registration_revision)?;
        let module=self.authored_identity.modules.iter().find(|m|m.id.as_str()==binding.consumer_module_id).ok_or("E_DDL_CONSUMER_MODULE_REQUIRED")?;
        if !module.sources.iter().any(|p|p==&binding.consumer_source_path){return Err("E_DDL_CONSUMER_SOURCE_OWNER_REQUIRED".into());}
        let file=self.source_snapshot.files.iter().find(|f|f.path==binding.consumer_source_path).ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?;
        let resolved=crate::local_project::resolve_first_callable_module_translation_request(&file.text,&file.path,&snapshot_source(&original.source_snapshot)?,
            &original.authored_identity.modules[first.module_index].head,
            &std::sync::Arc::new((**first.compiler_context()).clone().with_prefix_profile(profile.clone()))).map_err(|e|format!("{}: {}",e.code,e.message))?;
        let actual=ddonirang_lang::ddl::resolve_first_callable_profile_name(&resolved.request.target_language,&resolved.request.target_version).map_err(|e|e.to_string())?;
        if actual.identity()!=profile.identity()||actual.version()!=profile.version(){return Err("E_DDL_ACTIVATION_OWNER_OR_PROFILE".into());}
        // Structural/source checking only; actual G1/public export admission
        // is rebuilt from the retained original registration record below.
        let compiler=std::sync::Arc::new((*first.compiler).clone().with_module_activation(profile,resolved.binding).map_err(|e|e.to_string())?);
        ddonirang_lang::parser::Parser::parse_with_first_callable_context(&file.text,&file.path,compiler)
            .map_err(|e|format!("{} source={} encoding=utf-8-byte byte={}..{}: {}",e.code(),file.path,e.span.start,e.span.end,e.message))?;
        Ok(())
    }
    pub(crate) fn restore_language_context(&self,record:FirstCallableOriginalRegistration)->Result<FirstCallableAuthoredContext,String>{
        self.validate()?;
        if self.schema!=SCHEMA_V3{return Err("E_DDL_V3_REQUIRED".into());}
        let binding=self.language_binding.as_ref().ok_or("E_DDL_V3_BINDING_REQUIRED")?;
        let exported=restore_original_public_ddl(binding,record)?;
        exported.activate_for_consumer(self,&binding.consumer_module_id,&binding.consumer_source_path)
    }

    fn checksum(&self) -> Result<String, String> {
        let mut value =
            serde_json::to_value(self).map_err(|_| fail("저장 값을 직렬화할 수 없습니다"))?;
        value.as_object_mut().unwrap().remove("content_sha256");
        let bytes = serde_json::to_vec(&value).map_err(|_| fail("정본 직렬화에 실패했습니다"))?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }

    pub fn save(&self) -> Result<String, String> {self.save_diagnostic(None)}
    fn save_diagnostic(&self,diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>) -> Result<String, String> {
        self.validate_diagnostic(diagnostic_out)?;
        let mut sealed = self.clone();
        sealed.content_sha256 = sealed.checksum()?;
        let bytes =
            serde_json::to_vec(&serde_json::to_value(sealed).map_err(|_| fail("직렬화 실패"))?)
                .map_err(|_| fail("직렬화 실패"))?;
        if self.schema==SCHEMA_V3&&bytes.len()>8_388_608{return Err("E_DDL_V3_LIMIT".into());}
        if bytes.len() > MAX_BYTES {
            return Err(fail("저장 크기 한도를 넘었습니다"));
        }
        String::from_utf8(bytes).map_err(|_| fail("UTF-8 직렬화 실패"))
    }

    /// Explicit creation only; callers must not invoke this while running a bare source.
    pub fn create(
        source_profile: String,
        source_snapshot: SourceSnapshot,
        module_name: String,
    ) -> Result<Self, String> {
        Self::create_diagnostic(source_profile,source_snapshot,module_name,None)
    }
    fn create_diagnostic(source_profile:String,source_snapshot:SourceSnapshot,module_name:String,
        mut diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>) -> Result<Self,String> {
        if !supports_authored_source_profile(&source_profile) {
            use crate::ddn_runtime::{E_SOURCE_PROFILE_REQUIRED, E_SOURCE_PROFILE_UNSUPPORTED};

            let (code, message) = if source_profile.is_empty() {
                (E_SOURCE_PROFILE_REQUIRED, "문법 판을 명시해야 합니다")
            } else {
                (E_SOURCE_PROFILE_UNSUPPORTED, "지원하지 않는 작성 원문 프로필입니다")
            };
            if let Some(out) = diagnostic_out.as_deref_mut() {
                *out = Some(crate::preprocess::ExecutionDiagnostic {
                    code,
                    message: message.into(),
                    source_path: String::new(),
                    span: None,
                    phase: "project_prepare",
                    cause: None,
                    offending_field: Some("source_profile".into()),
                    expected: None,
                    actual: Some(source_profile),
                });
            }
            return Err(format!("{code}: {message}"));
        }
        let modules = if source_snapshot.root_path.is_some() {
            if !module_name.is_empty() {
                return Err(fail(
                    "여러 모듈의 이름은 얽음에 선언합니다. 별도 모듈 이름을 덮어쓸 수 없습니다",
                ));
            }
            let graph = snapshot_graph_diagnostic(&source_snapshot, &source_profile,diagnostic_out.as_deref_mut())?;
            graph
                .module_paths
                .into_iter()
                .map(|(name, head)| {
                    Ok(AuthoredModule {
                        id: ModuleId::fresh()?,
                        sources: graph.module_sources[&name].clone(),
                        name,
                        head,
                        compositions: vec![],
                        bindings: vec![],
                    })
                })
                .collect::<Result<Vec<_>, String>>()?
        } else {
            vec![AuthoredModule {
                id: ModuleId::fresh()?,
                name: module_name,
                head: source_snapshot.entry_path.clone(),
                sources: source_snapshot
                    .files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect(),
                compositions: vec![],
                bindings: vec![],
            }]
        };
        let project = Self {
            schema: SCHEMA.into(),
            source_profile,
            authored_identity: AuthoredIdentity {
                schema: "ddn.authored_identity.v1".into(),
                logical_dependency_id: ProjectId::fresh()?,
                modules,
                entry: None,
                clone_origin: None,
            },
            source_snapshot,
            content_sha256: String::new(),
            language_binding:None,
        };
        project.validate_diagnostic(diagnostic_out)?;
        Ok(project)
    }

    /// The explicit authoring operation is the only composition/binding ID supplier.
    pub fn register_bogae(
        &self,
        module_name: &str,
        source_path: &str,
        composition_definition: &str,
        make_entry: bool,
    ) -> Result<Self, String> {
        self.validate()?;
        crate::authored_bogae::register(
            self,
            module_name,
            source_path,
            composition_definition,
            make_entry,
        )
    }

    /// Complete source-owned public declarations; no registration capability.
    pub(crate) fn source_owned_public_facts(&self, module_identity: &str) -> Result<public_surface::SourceOwnedPublicFacts,String> {
        public_surface::from_project(self,module_identity)
    }

    /// Actual linked source and graph start, separate from presentation entry.
    pub fn first_callable_registration_context(
        &self, module_identity: &str, source_path: &str, ddl_source: &str, ddl_path: &str,
        language_profile_identity: &str, registration_revision: u32,
    ) -> Result<FirstCallableAuthoredContext, String> {
        self.validate()?;
        if self.content_sha256 != self.checksum()? {
            return Err("E_MULTILANG_ORIGINAL_CHECKSUM_STALE: 원본 변경은 기존 작품 갱신·검증을 거쳐야 합니다".into());
        }
        crate::ddn_runtime::resolve_supported_source_profile(Some(&self.source_profile))
            .map_err(|e| format!("{}: {}", e.code, e.message))?;
        let module_index = self.authored_identity.modules.iter().position(|module| module.id.as_str() == module_identity)
            .ok_or("E_MULTILANG_ORIGINAL_MODULE_OWNER_REQUIRED")?;
        let module = &self.authored_identity.modules[module_index];
        if module.sources.as_slice() != [source_path] {
            return Err("E_MULTILANG_ORIGINAL_SOURCE_OWNER_REQUIRED: 검증된 모듈의 전체 원본이 필요합니다".into());
        }
        let (source_index, source) = self.source_snapshot.files.iter().enumerate().find(|(_,file)| file.path == source_path)
            .ok_or("E_MULTILANG_ORIGINAL_SOURCE_REQUIRED")?;
        // Rootless originals retain their existing closed first-function
        // contract; do not synthesize a Root to claim general public facts.
        let public_targets = if self.source_snapshot.root_path.is_some() {
            self.source_owned_public_facts(module_identity)?.public_targets()
        } else {
            ["총액합하다".to_string()].into_iter().collect::<BTreeSet<_>>()
        };
        let public = ddonirang_lang::parse_module_directive_frontdoor_with_mode(&source.text, source_path,
            ddonirang_lang::ParseMode::Strict,
            &ddonirang_lang::ModuleDirectiveContext::default().with_public_definitions(public_targets))
            .map_err(|e| e.to_string())?;
        let declaration_offset = source.text.len().checked_sub(public.directives.declaration_source.len())
            .filter(|offset| source.text.get(*offset..) == Some(public.directives.declaration_source.as_str()))
            .ok_or("E_MULTILANG_ORIGINAL_SPAN_SUPPLY_REQUIRED: 원본 바이트로의 실제 위치 공급이 필요합니다")?;
        let compiler = ddonirang_lang::parser::RegisteredFirstCallableContext::from_public_and_ddl(
            public, ddl_source, ddl_path, language_profile_identity, registration_revision,
        ).map_err(|e| e.to_string())?;
        Ok(FirstCallableAuthoredContext { original: self.clone(), module_index, source_index, declaration_offset,
            compiler: std::sync::Arc::new(compiler),public_export:None,consumer_owner:None })
    }

    pub fn execution_source(&self) -> Result<(String, String, Option<String>), String> {
        if self.source_snapshot.root_path.is_none() && self.source_snapshot.files.iter().any(|file|
            file.path == self.source_snapshot.entry_path && crate::ddn_runtime::first_callable_header_requires_context(&file.text)) {
            return Err("E_MULTILANG_DURABLE_CONTEXT_REQUIRED: 기존 작품 wire에는 일본어 DDL 활성 문맥 공급이 없습니다".into());
        }
        self.validate()?;
        if let Some(prepared) = self.authored_preparation()? {
            Ok((
                prepared.source,
                prepared.target_path,
                prepared.start_definition,
            ))
        } else {
            let source = self
                .source_snapshot
                .files
                .iter()
                .find(|file| file.path == self.source_snapshot.entry_path)
                .ok_or_else(|| fail("실행 원문이 없습니다"))?;
            Ok((source.text.clone(), source.path.clone(), None))
        }
    }

    pub(crate) fn authored_preparation(
        &self,
    ) -> Result<Option<crate::local_project::AuthoredPreparation>, String> {
        self.authored_preparation_diagnostic(None)
    }

    pub(crate) fn authored_preparation_diagnostic(
        &self,
        diagnostic: Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    ) -> Result<Option<crate::local_project::AuthoredPreparation>, String> {
        if self.source_snapshot.root_path.is_none() {
            return Ok(None);
        }
        let profile=crate::ddn_runtime::resolve_supported_source_profile(Some(&self.source_profile))
            .map_err(|error|fail(&error.message))?;
        let prepared=match profile {
            crate::ddn_runtime::SupportedSourceProfile::V1CoreV25JaBaseR1=>{
                let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(&self.source_profile).map_err(|error|fail(&error.to_string()))?;
                let source=snapshot_source_selected(&self.source_snapshot,Some(syntax))?;
                crate::local_project::prepare_authored_execution_with_j1_profile(source,syntax)
            },
            crate::ddn_runtime::SupportedSourceProfile::V1CoreV25=>crate::local_project::prepare_authored_execution(snapshot_source(&self.source_snapshot)?),
        };
        prepared.map(Some).map_err(|error| {
            let message = fail(&format!("{}: {}", error.code, error.message));
            if let Some(output) = diagnostic.filter(|output| output.is_none()) {
                if let Some(cause) = error.cause.filter(|cause| matches!(cause.code, "E_J1_ROLE_AMBIGUOUS" | "E_MODULE_MISSING" | "E_FIXED64_LITERAL_RANGE")) {
                    *output = Some(*cause);
                }
            }
            message
        })
    }

    fn finish_explicit_edit(mut self) -> Result<Self, String> {
        crate::authored_bogae::refresh_source_locks(&mut self)?;
        let reopened = open(self.save()?.as_bytes())?;
        let (source, path, _) = reopened.execution_source()?;
        if reopened.schema == SCHEMA_V2
            && reopened
                .authored_identity
                .modules
                .iter()
                .any(|m| !m.compositions.is_empty())
        {
            crate::authored_bogae::Activation::from_artifact(&reopened.save()?, &source, &path)?;
        }
        Ok(reopened)
    }

    /// Caller supplies the completed source edit and one explicit identity map.
    pub fn rename(
        &self,
        owner_module_id: &str,
        target_kind: &str,
        target_id: &str,
        new_name: &str,
        snapshot: SourceSnapshot,
    ) -> Result<Self, String> {
        self.validate()?;
        if self.schema != SCHEMA_V2 {
            return Err(fail("정의 이름 변경은 명시 v2 작성 결속이 필요합니다"));
        }
        let parsed = ddonirang_lang::parse_v25(&format!("{new_name} := 0."), "<rename>")
            .map_err(|_| fail("새 이름은 정확한 정의 이름 하나여야 합니다"))?;
        if !parsed.items.is_empty()
            || parsed.root_stmts.len() != 1
            || !matches!(&parsed.root_stmts[0],
            ddonirang_lang::Stmt::Definition{name,..} if name==new_name)
        {
            return Err(fail("새 이름은 정확한 정의 이름 하나여야 합니다"));
        }
        if snapshot.root_path != self.source_snapshot.root_path
            || snapshot.entry_path != self.source_snapshot.entry_path
            || snapshot
                .files
                .iter()
                .map(|f| &f.path)
                .collect::<BTreeSet<_>>()
                != self
                    .source_snapshot
                    .files
                    .iter()
                    .map(|f| &f.path)
                    .collect::<BTreeSet<_>>()
        {
            return Err(fail("이름 변경으로 파일 위치·모듈 소유를 바꿀 수 없습니다"));
        }
        let mut copy = self.clone();
        let module = copy
            .authored_identity
            .modules
            .iter_mut()
            .find(|m| m.id.as_str() == owner_module_id)
            .ok_or_else(|| fail("명시 소유 모듈 정체가 없습니다"))?;
        let (old_name, path) = match target_kind {
            "binding" => {
                let binding = module
                    .bindings
                    .iter_mut()
                    .find(|b| b.id.as_str() == target_id)
                    .ok_or_else(|| fail("소유 결속 정체가 없습니다"))?;
                let old = binding.definition.clone();
                binding.definition = new_name.into();
                (old, binding.source_path.clone())
            }
            "composition" => {
                let composition = module
                    .compositions
                    .iter_mut()
                    .find(|c| c.id.as_str() == target_id)
                    .ok_or_else(|| fail("소유 짜임 정체가 없습니다"))?;
                let old = composition.definition.clone();
                composition.definition = new_name.into();
                (
                    old,
                    composition
                        .registration
                        .as_ref()
                        .ok_or_else(|| fail("짜임 등록이 없습니다"))?
                        .source_path
                        .clone(),
                )
            }
            _ => {
                return Err(fail(
                    "이름 변경 대상은 binding 또는 composition이어야 합니다",
                ))
            }
        };
        if old_name == new_name {
            return Err(fail("새 이름이 원래 이름과 같습니다"));
        }
        copy.source_snapshot = snapshot;
        crate::authored_bogae::verify_explicit_rename(
            &copy,
            owner_module_id,
            &path,
            &old_name,
            new_name,
        )?;
        copy.finish_explicit_edit()
    }

    /// Explicit file mapping only; this never rewrites source text or ownership.
    pub fn relocate(
        &self,
        old_path: &str,
        new_path: &str,
        snapshot: SourceSnapshot,
    ) -> Result<Self, String> {
        self.validate()?;
        if old_path == new_path
            || !locator(old_path)
            || !locator(new_path)
            || !self
                .source_snapshot
                .files
                .iter()
                .any(|f| f.path == old_path)
            || self
                .source_snapshot
                .files
                .iter()
                .any(|f| f.path == new_path)
        {
            return Err(fail("명시 파일 이동 위치가 잘못되었거나 충돌합니다"));
        }
        let mapped = |path: &str| {
            if path == old_path {
                new_path.to_string()
            } else {
                path.to_string()
            }
        };
        if snapshot
            .files
            .iter()
            .map(|f| f.path.clone())
            .collect::<BTreeSet<_>>()
            != self
                .source_snapshot
                .files
                .iter()
                .map(|f| mapped(&f.path))
                .collect::<BTreeSet<_>>()
            || snapshot.entry_path != mapped(&self.source_snapshot.entry_path)
            || snapshot.root_path != self.source_snapshot.root_path.as_deref().map(mapped)
        {
            return Err(fail("파일 이동 요청과 완성 원문의 위치 mapping이 다릅니다"));
        }
        let mut copy = self.clone();
        copy.source_snapshot = snapshot;
        for module in &mut copy.authored_identity.modules {
            module.head = mapped(&module.head);
            for path in &mut module.sources {
                *path = mapped(path);
            }
            for binding in &mut module.bindings {
                binding.source_path = mapped(&binding.source_path);
            }
            for composition in &mut module.compositions {
                if let Some(registration) = &mut composition.registration {
                    registration.source_path = mapped(&registration.source_path);
                }
            }
        }
        copy.finish_explicit_edit()
    }

    /// Explicit clone is atomic: original is immutable even if entropy acquisition fails.
    pub fn duplicate(&self) -> Result<Self, String> {
        self.validate()?;
        let mut copy = self.clone();
        copy.authored_identity.clone_origin =
            Some(self.authored_identity.logical_dependency_id.clone());
        copy.authored_identity.logical_dependency_id = ProjectId::fresh()?;
        for module in &mut copy.authored_identity.modules {
            let old_module = module.id.clone();
            module.id = ModuleId::fresh()?;
            let mut binding_map = BTreeMap::new();
            for binding in &mut module.bindings {
                let old = binding.id.clone();
                let fresh = BindingId::fresh()?;
                binding_map.insert(old, fresh.clone());
                binding.id = fresh;
            }
            for composition in &mut module.compositions {
                let old_composition = composition.id.clone();
                composition.id = CompositionId::fresh()?;
                if let Some(registration) = &mut composition.registration {
                    registration.clone_origin = Some(old_composition.clone());
                }
                if let Some(entry) = &mut copy.authored_identity.entry {
                    if entry.module == old_module && entry.composition == old_composition {
                        entry.module = module.id.clone();
                        entry.composition = composition.id.clone();
                    }
                }
                for binding in &mut composition.binding_ids {
                    *binding = if copy.schema == SCHEMA_V2 {
                        binding_map
                            .get(binding)
                            .cloned()
                            .ok_or_else(|| fail("복제 결속을 해소할 수 없습니다"))?
                    } else {
                        BindingId::fresh()?
                    };
                }
            }
        }
        copy.content_sha256.clear();
        copy.validate()?;
        Ok(copy)
    }
}

fn snapshot_graph(
    snapshot: &SourceSnapshot,
    profile: &str,
) -> Result<crate::local_project::LocalProjectGraph, String> {
    snapshot_graph_diagnostic(snapshot,profile,None)
}
fn snapshot_graph_diagnostic(
    snapshot:&SourceSnapshot,profile:&str,
    mut diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
)->Result<crate::local_project::LocalProjectGraph,String>{
    if profile==ddonirang_lang::ddl::J1_SOURCE_PROFILE{
        let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(profile).map_err(|e|e.to_string())?;
        let source=snapshot_source_selected_diagnostic(snapshot,Some(syntax),diagnostic_out.as_deref_mut())?;
        return crate::local_project::build_project_graph_with_j1_profile(source,syntax)
            .map_err(|error|{
                let message=fail(&format!("{}: {}",error.code,error.message));
                if let Some(out)=diagnostic_out.as_deref_mut(){*out=Some(error.into_execution_diagnostic());}
                message
            });
    }
    let source = snapshot_source(snapshot)?;
    crate::local_project::project_source_profile(&source, Some(profile))
        .map_err(|error| fail(&format!("{}: {}", error.code, error.message)))?;
    crate::local_project::build_project_graph(source)
        .map_err(|error| fail(&format!("{}: {}", error.code, error.message)))
}

fn snapshot_source(
    snapshot: &SourceSnapshot,
) -> Result<crate::local_project::VirtualProjectSource, String> {
    snapshot_source_selected(snapshot,None)
}
pub(crate) fn snapshot_source_selected(snapshot:&SourceSnapshot,j1:Option<ddonirang_lang::ddl::CheckedJ1Profile>)->Result<crate::local_project::VirtualProjectSource,String>{
    snapshot_source_selected_diagnostic(snapshot,j1,None)
}
fn snapshot_source_selected_diagnostic(snapshot:&SourceSnapshot,j1:Option<ddonirang_lang::ddl::CheckedJ1Profile>,
    mut diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
)->Result<crate::local_project::VirtualProjectSource,String>{
    use crate::local_project::{self, VirtualProjectFileSource, VirtualProjectSource};
    let root = snapshot
        .root_path
        .as_ref()
        .ok_or_else(|| fail("명시 작품 뿌리가 없습니다"))?;
    if !locator(root) {
        return Err(fail("작품 뿌리 경로가 올바르지 않습니다"));
    }
    let root_text = snapshot.files.iter().find(|file| &file.path == root)
        .ok_or_else(|| fail("작품 뿌리 원문이 없습니다"))?;
    let metadata_roles=if let Some(profile)=j1{
        // The closed J1 root admits only epoch/modules/start, no metadata role.
        ddonirang_lang::work_root::parse_work_root_with_j1_profile(&root_text.text,profile).map_err(|error|{
            let message=fail(&format!("E_PROJECT_ROOT_PARSE: {}",error.message));
            if let Some(out)=diagnostic_out.as_deref_mut(){*out=Some(crate::preprocess::ExecutionDiagnostic{
                code:error.j1_source_code(),message:error.message,source_path:root.clone(),
                span:Some((error.span.start,error.span.end)),phase:"project_root",cause:None,
                offending_field:None,expected:None,actual:None});}
            message
        })?;
        BTreeMap::new()
    }else{local_project::v25_package_metadata_roles(&root_text.text).map_err(|e|fail(&format!("{}: {}",e.code,e.message)))?};
    let source = VirtualProjectSource {
        schema: local_project::VIRTUAL_PROJECT_SOURCE_SCHEMA.into(),
        kind: None,
        package_hash: None,
        root_path: root.clone(),
        entry_path: snapshot.entry_path.clone(),
        files: snapshot
            .files
            .iter()
            .map(|file| VirtualProjectFileSource {
                path: file.path.clone(),
                role: if &file.path == root { "root".into() } else {
                    metadata_roles.get(&file.path).cloned().unwrap_or_else(|| "ddn_source".into())
                },
                utf8_text: file.text.clone(),
            })
            .collect(),
    };
    if source.files.iter().any(|file| j1.is_none() && file.role == "ddn_source"
        && crate::ddn_runtime::first_callable_header_requires_context(&file.utf8_text)) {
        return Err("E_MULTILANG_DURABLE_CONTEXT_REQUIRED: DDL을 임의의 ddn_source 저장 역할로 바꿀 수 없습니다".into());
    }
    Ok(source)
}

// serde_json::Value normally silently accepts duplicate keys. Admission must not.
struct StrictValue(Value);
impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = StrictValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("중복 없는 정수 JSON")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut out = vec![];
                while let Some(v) = a.next_element::<StrictValue>()? {
                    out.push(v.0);
                }
                Ok(StrictValue(out.into()))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut out = serde_json::Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if out.contains_key(&k) {
                        return Err(de::Error::custom("중복 JSON 키"));
                    }
                    out.insert(k, a.next_value::<StrictValue>()?.0);
                }
                Ok(StrictValue(out.into()))
            }
        }
        d.deserialize_any(V)
    }
}

pub fn open(bytes: &[u8]) -> Result<ProjectArtifact, String> {open_with_diagnostic(bytes,None)}
pub(crate) fn open_with_diagnostic(bytes:&[u8],diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<ProjectArtifact,String>{
    if bytes.len() > MAX_BYTES || bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(fail("저장 크기 초과 또는 BOM입니다"));
    }
    let value: StrictValue =
        serde_json::from_slice(bytes).map_err(|_| fail("UTF-8·중복 키·수 표기를 확인하세요"))?;
    if value.0.as_object().is_some_and(|object|object.contains_key("language_binding"))
        &&value.0.get("schema").and_then(Value::as_str)!=Some(SCHEMA_V3){return Err("E_DDL_BINDING_SCHEMA: 구판은 null 포함 언어 필드를 받지 않습니다".into());}
    if value.0.get("schema").and_then(Value::as_str)==Some(SCHEMA_V3){
        if bytes.len()>8_388_608{return Err("E_DDL_V3_LIMIT".into());}check_v3_json_depth(&value.0)?;
    }
    let project: ProjectArtifact =
        serde_json::from_value(value.0).map_err(|_| fail("저장 필드가 올바르지 않습니다"))?;
    project.validate_diagnostic(diagnostic_out)?;
    if project.content_sha256 != project.checksum()? {
        return Err(fail("저장 내용 해시가 다릅니다"));
    }
    Ok(project)
}

/// Shared native/WASM entry point. Does not activate unvalidated authored views.
pub(crate) fn restore_first_callable_original_registration(
    owned:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1,
    first:&FirstCallableAuthoredContext,records:Vec<registration::RecordedRegistrationBlob>,
)->Result<registration::RegistrationDraft,String>{
    registration::restore_precanon_first_callable_candidate(owned,first,records)
}

/// Explicit, checked original-source context. Existing operations and request
/// schemas remain unchanged; no ambient registry or execution permission.
pub fn operate_with_first_callable_context(
    operation: &str, text: &str, first: &FirstCallableAuthoredContext,
) -> Result<String,String> {
    match operation {
        "source_prepare" => source_save::prepare_first_callable(text, first),
        "source_open" => source_save::open_first_callable(text, first),
        _ => Err("E_MULTILANG_OPERATION_CONTEXT: 이 연결은 원본 등록 저장·재열기만 받습니다".into()),
    }
}

/// Same-attempt presentation data, never a stored or executable capability.
#[derive(Debug)]
pub struct FirstCallableProductError {message:String,analysis:Option<crate::source_analysis::SourceAnalysisSnapshot>}
impl FirstCallableProductError {
    pub fn message(&self)->&str{&self.message}
    pub fn analysis(&self)->Option<&crate::source_analysis::SourceAnalysisSnapshot>{self.analysis.as_ref()}
}
impl From<String> for FirstCallableProductError{fn from(message:String)->Self{Self{message,analysis:None}}}
impl From<&str> for FirstCallableProductError{fn from(message:&str)->Self{Self{message:message.into(),analysis:None}}}
pub fn prepare_first_callable_source_with_analysis(text:&str)->Result<String,FirstCallableProductError>{source_save::prepare_first_callable_product_with_analysis(text)}

pub(crate) fn operate_with_logic_provider_private(operation:&str,text:&str)->Result<String,String>{
    source_save::logic_consumer::operate_private(operation,text)
}

/// Internal library/binary presentation bridge for existing CLI operations.
/// Same request, success JSON and original error text; no new transport API.
#[doc(hidden)]
pub fn operate_for_cli(operation: &str, text: &str)
    -> Result<String, (String, Option<crate::preprocess::ExecutionDiagnostic>)> {
    if matches!(operation,"create"|"update") {
        let mut diagnostic=None;
        return operate_authoring_diagnostic(operation,text,Some(&mut diagnostic))
            .map_err(|message|(message,diagnostic));
    }
    if operation=="public_callable_original_source_prepare" {
        let mut diagnostic=None;
        return source_save::prepare_public_callable_original_diagnostic(text,Some(&mut diagnostic))
            .map_err(|message|(message,diagnostic));
    }
    if operation=="public_callable_source_prepare" {
        let mut diagnostic=None;
        return source_save::prepare_public_callable_product(text,Some(&mut diagnostic))
            .map_err(|message|(message,diagnostic));
    }
    if matches!(operation,"public_callable_edition_update_native"|"public_callable_edition_reopen_native"|"public_callable_edition_run_native"){
        return source_save::operate_public_edition_native(operation,text).map_err(|message|(message,None));
    }
    if operation == "linked_callable_source_run" {
        return joint_execution::run(text).map_err(|error| (error.message.clone(), Some(error)));
    }
    let typed = match operation {
        "source_probe" | "source_prepare" => true,
        "public_callable_source_run" => source_save::logic_consumer::public_run_is_selected(text)
            .map_err(|message| (message, None))?,
        _ => false,
    };
    if typed {
        source_save::logic_consumer::operate_private_diagnostic(operation, text)
            .map_err(|error| (error.message.clone(), Some(error)))
    } else if operation == "public_callable_source_run" {
        let mut diagnostic = None;
        source_save::run_v4_public_callable_product(text, Some(&mut diagnostic))
            .map_err(|message| (message, diagnostic))
    } else { operate(operation, text).map_err(|message| (message, None)) }
}

// The original authoring DTOs and custody order, with optional same-attempt
// presentation data. No post-failure reanalysis, source rewriting or IO.
fn operate_authoring_diagnostic(operation:&str,text:&str,
    mut diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<String,String>{
    match operation {
        "create" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("작성 요청 JSON을 확인하세요"))?;
            let request: CreateRequest =
                serde_json::from_value(value.0).map_err(|_| fail("작성 요청 필드를 확인하세요"))?;
            ProjectArtifact::create_diagnostic(
                request.source_profile,
                request.source_snapshot,
                request.module_name,diagnostic_out.as_deref_mut(),
            )?
            .save_diagnostic(diagnostic_out)
        }
        "update" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("갱신 요청 JSON을 확인하세요"))?;
            let request: UpdateRequest =
                serde_json::from_value(value.0).map_err(|_| fail("갱신 요청 필드를 확인하세요"))?;
            // Verify the original before any edit; do not bless a tampered
            // identity registry by simply computing a new checksum for it.
            let mut project = open_with_diagnostic(request.original_json.as_bytes(),diagnostic_out.as_deref_mut())?;
            if project.source_snapshot.root_path != request.source_snapshot.root_path {
                return Err(fail("원문 갱신으로 작품 뿌리를 추가·제거·바꿀 수 없습니다"));
            }
            project.source_snapshot = request.source_snapshot;
            crate::authored_bogae::refresh_source_locks(&mut project)?;
            // Module ownership is retained. File moves/ownership changes must
            // use an explicit edit operation, never infer new IDs from paths.
            project.save_diagnostic(diagnostic_out)
        }
        _=>unreachable!("authoring dispatcher is private and closed"),
    }
}

pub fn operate(operation: &str, text: &str) -> Result<String, String> {
    match operation {
        "verified_dependency_tool" => crate::dependency_tool::operate(text),
        "observe_native_source" => crate::native_source_observation::operate(text),
        // Private Workbench transport operations, not DDN callables or new
        // CLI command names. Source candidates grant no execution authority.
        "public_callable_original_source_prepare"=>source_save::prepare_public_callable_original(text),
        "public_callable_source_prepare"=>source_save::prepare_public_callable_product(text,None),
        "public_callable_source_run"=>source_save::run_public_callable_product(text),
        "public_callable_source_save_native"=>source_save::save_public_callable_native(text),
        "public_callable_source_reopen_native"=>source_save::reopen_public_callable_native(text),
        "first_callable_original_source_prepare"=>source_save::prepare_first_callable_original(text),
        "first_callable_source_prepare"=>source_save::prepare_first_callable_product(text),
        "first_callable_source_run"=>source_save::run_first_callable_product(text),
        "first_callable_source_save_native"=>source_save::save_first_callable_native(text),
        "first_callable_source_reopen_native"=>source_save::reopen_first_callable_native(text),
        "source_prepare"=>source_save::prepare(text),
        "source_open"=>source_save::open(text),
        "source_probe"=>source_save::logic_consumer::probe(text),
        "linked_callable_source_run" => joint_execution::run(text).map_err(|error| error.message),
        "source_project"=>source_projection::project(text),
        "source_language_support" => language_support::inspect(text),
        "create" => operate_authoring_diagnostic(operation,text,None),
        "open" => open(text.as_bytes())?.save(),
        "clone" => open(text.as_bytes())?.duplicate()?.save(),
        "update" => operate_authoring_diagnostic(operation,text,None),
        "register_bogae" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("보개 등록 요청 JSON을 확인하세요"))?;
            let request: RegisterBogaeRequest = serde_json::from_value(value.0)
                .map_err(|_| fail("보개 등록 요청 필드를 확인하세요"))?;
            open(request.original_json.as_bytes())?
                .register_bogae(
                    &request.module_name,
                    &request.source_path,
                    &request.composition_definition,
                    request.make_entry,
                )?
                .save()
        }
        "rename" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("이름 변경 요청 JSON을 확인하세요"))?;
            let request: RenameRequest = serde_json::from_value(value.0)
                .map_err(|_| fail("이름 변경 요청 필드를 확인하세요"))?;
            open(request.original_json.as_bytes())?
                .rename(
                    &request.owner_module_id,
                    &request.target_kind,
                    &request.target_id,
                    &request.new_name,
                    request.source_snapshot,
                )?
                .save()
        }
        "relocate" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("파일 이동 요청 JSON을 확인하세요"))?;
            let request: RelocateRequest = serde_json::from_value(value.0)
                .map_err(|_| fail("파일 이동 요청 필드를 확인하세요"))?;
            open(request.original_json.as_bytes())?
                .relocate(
                    &request.old_path,
                    &request.new_path,
                    request.source_snapshot,
                )?
                .save()
        }
        _ => Err(fail("지원하지 않는 작성 작업입니다")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> ProjectArtifact {
        ProjectArtifact::create(
            "v1-core-v25".into(),
            SourceSnapshot {
                root_path: None,
                entry_path: "main.ddn".into(),
                files: vec![SourceFile {
                    path: "main.ddn".into(),
                    text: "값 := 1.".into(),
                }],
            },
            "앱".into(),
        )
        .unwrap()
    }
    #[test]
    fn update_preserves_identity_and_original_bytes() {
        let original = fixture();
        let bytes = original.save().unwrap();
        let mut snapshot = original.source_snapshot.clone();
        snapshot.files[0].text = "값 := 2.".into();
        let request = serde_json::json!({"original_json": bytes, "source_snapshot": snapshot});
        let updated = open(operate("update", &request.to_string()).unwrap().as_bytes()).unwrap();
        assert_eq!(updated.authored_identity, original.authored_identity);
        assert_eq!(updated.source_profile, original.source_profile);
        assert_eq!(updated.source_snapshot, snapshot);
        assert_ne!(
            updated.content_sha256,
            open(bytes.as_bytes()).unwrap().content_sha256
        );
        assert_eq!(
            open(bytes.as_bytes()).unwrap().source_snapshot,
            original.source_snapshot
        );
    }
    #[test]
    fn multi_module_root_is_separate_and_ids_survive_edit() {
        let snapshot = SourceSnapshot {
            root_path: Some("work.ddn".into()), entry_path: "app/main.ddn".into(),
            files: vec![
                SourceFile {path:"work.ddn".into(), text:r#"(글판="v25", 모듬=차림[(이름=앱, 머리="app/main.ddn", 원문="app/main.ddn"),(이름=힘, 머리="lib/힘.ddn", 원문="lib/힘.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음."#.into()},
                SourceFile {path:"app/main.ddn".into(),text:"실행:움직씨 := { 1 보여주기. }.".into()},
                SourceFile {path:"lib/힘.ddn".into(),text:"힘 := 2.".into()},
            ],
        };
        let original =
            ProjectArtifact::create("v1-core-v25".into(), snapshot.clone(), String::new()).unwrap();
        assert_eq!(original.authored_identity.modules.len(), 2);
        assert!(original
            .authored_identity
            .modules
            .iter()
            .all(|module| !module.sources.contains(&"work.ddn".to_string())));
        let bytes = original.save().unwrap();
        let mut edited = snapshot;
        edited.files[2].text = "힘 := 3.".into();
        let request = serde_json::json!({"original_json":bytes,"source_snapshot":edited});
        let updated = open(operate("update", &request.to_string()).unwrap().as_bytes()).unwrap();
        assert_eq!(updated.authored_identity, original.authored_identity);
        let mut bad = updated.clone();
        bad.source_snapshot.root_path = None;
        bad.source_snapshot
            .files
            .retain(|file| file.path != "work.ddn");
        assert!(bad.save().is_err());
        let request =
            serde_json::json!({"original_json":bytes,"source_snapshot":bad.source_snapshot});
        assert!(operate("update", &request.to_string()).is_err());
        let mut bad = updated.clone();
        bad.authored_identity.modules[0].head = "lib/힘.ddn".into();
        assert!(bad.save().is_err());
        let mut bad = updated;
        bad.source_snapshot.files.push(SourceFile {
            path: "extra.ddn".into(),
            text: "3 보여주기.".into(),
        });
        assert!(bad.save().is_err());
    }
    #[test]
    fn update_rejects_tampered_original_and_implicit_ownership_change() {
        let original = fixture();
        let bytes = original.save().unwrap();
        let tampered = bytes.replace("값 := 1.", "값 := 99.");
        let request = serde_json::json!({"original_json": tampered, "source_snapshot": original.source_snapshot});
        assert!(operate("update", &request.to_string()).is_err());
        let mut snapshot = original.source_snapshot.clone();
        snapshot.files.push(SourceFile {
            path: "unowned.ddn".into(),
            text: "1 보여주기.".into(),
        });
        let request = serde_json::json!({"original_json": bytes, "source_snapshot": snapshot});
        assert!(operate("update", &request.to_string()).is_err());
    }
    #[test]
    fn save_reopen_move_rename_preserve_ids() {
        let mut p = fixture();
        let ids = p.authored_identity.clone();
        assert_eq!(
            open(p.save().unwrap().as_bytes())
                .unwrap()
                .authored_identity,
            ids
        );
        p.source_snapshot.entry_path = "app/new.ddn".into();
        p.source_snapshot.files[0].path = "app/new.ddn".into();
        p.authored_identity.modules[0].head = "app/new.ddn".into();
        p.authored_identity.modules[0].sources = vec!["app/new.ddn".into()];
        p.authored_identity.modules[0].name = "새이름".into();
        let reopened = open(p.save().unwrap().as_bytes()).unwrap();
        assert_eq!(
            reopened.authored_identity.logical_dependency_id,
            ids.logical_dependency_id
        );
        assert_eq!(reopened.authored_identity.modules[0].id, ids.modules[0].id);
    }
    #[test]
    fn clone_references_and_origin_are_explicit() {
        let mut p = fixture();
        let c = CompositionId::fresh().unwrap();
        let m = p.authored_identity.modules[0].id.clone();
        p.authored_identity.modules[0]
            .compositions
            .push(AuthoredComposition {
                id: c.clone(),
                definition: "보기".into(),
                binding_ids: vec![BindingId::fresh().unwrap()],
                registration: None,
            });
        p.authored_identity.entry = Some(Entry {
            module: m,
            composition: c,
        });
        let original = p.clone();
        let copy = open(p.duplicate().unwrap().save().unwrap().as_bytes()).unwrap();
        assert_eq!(p, original);
        assert_ne!(
            p.authored_identity.modules[0].id,
            copy.authored_identity.modules[0].id
        );
        assert_ne!(
            p.authored_identity.modules[0].compositions[0].id,
            copy.authored_identity.modules[0].compositions[0].id
        );
        assert_ne!(
            p.authored_identity.modules[0].compositions[0].binding_ids,
            copy.authored_identity.modules[0].compositions[0].binding_ids
        );
        assert_ne!(
            p.authored_identity.logical_dependency_id,
            copy.authored_identity.logical_dependency_id
        );
        assert_eq!(
            copy.authored_identity.clone_origin,
            Some(p.authored_identity.logical_dependency_id.clone())
        );
        assert_eq!(
            copy.authored_identity.entry.as_ref().unwrap().module,
            copy.authored_identity.modules[0].id
        );
        assert_eq!(
            copy.authored_identity.entry.as_ref().unwrap().composition,
            copy.authored_identity.modules[0].compositions[0].id
        );
    }
    #[test]
    fn rejects_resealed_unowned_source() {
        let mut p = fixture();
        p.source_snapshot.files.push(SourceFile {
            path: "extra.ddn".into(),
            text: "값 := 2.".into(),
        });
        p.content_sha256 = p.checksum().unwrap();
        let bytes = serde_json::to_vec(&p).unwrap();
        assert!(open(&bytes).unwrap_err().contains("소유 모듈이 없는"));
        assert!(p.save().is_err());
    }
    #[test]
    fn rejects_corruption_duplicate_unknown_and_unowned_source() {
        let p = fixture();
        let sealed = p.save().unwrap();
        assert!(open(sealed.replace("값 := 1.", "값 := 2.").as_bytes()).is_err());
        assert!(open(format!("{{\"schema\":\"bad\",{}", &sealed[1..]).as_bytes()).is_err());
        assert!(open(format!("{{\"unknown\":1,{}", &sealed[1..]).as_bytes()).is_err());
        assert!(open(format!("{{\"unknown\":1.5,{}", &sealed[1..]).as_bytes()).is_err());
        let mut bad = p.clone();
        bad.authored_identity
            .modules
            .push(bad.authored_identity.modules[0].clone());
        assert!(bad.save().is_err());
        let mut bad = p;
        bad.authored_identity.modules[0].head = "../main.ddn".into();
        assert!(bad.save().is_err());
    }
}

#[cfg(test)]
mod g2_original_module_preflight_tests {
use super::*;
    const ORIGINAL:&str = r###"{"authored_identity":{"clone_origin":null,"entry":null,"logical_dependency_id":"fe6b4154262752919656f5a6f7e3629b3e81758594783b9efca19a1f0ce9ed52","modules":[{"compositions":[],"head":"세금.ddn","id":"0a9e9104a2f6a6434c2bd0dd41a721a7dacc85125c791fe47aed1deb2f4f2068","name":"세금","sources":["세금.ddn"]},{"compositions":[],"head":"app.ddn","id":"36612e8bcf97c580cb38d1c3173b719c4acbf892a937bbe97e9df9f947521c8c","name":"앱","sources":["app.ddn"]}],"schema":"ddn.authored_identity.v1"},"content_sha256":"60783cb50f9b5e604197407d2c7241c990e67171ee5608e2b10593d9b7ee1659","schema":"ddn.authored_project_artifact.v1","source_profile":"v1-core-v25","source_snapshot":{"entry_path":"app.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\"), (이름=세금, 머리=\"세금.ddn\", 원문=\"세금.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n"},{"path":"app.ddn","text":"(세금=세금) 쓰임.\n실행:움직씨 := {\n    ((금액=(100, 1) 나눔수, 세액=(10, 1) 나눔수) 세금.총액합하다) 보여주기.\n}.\n"},{"path":"세금.ddn","text":"(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를)\n총액합하다{총액합하기}:나눔수:셈씨 := {\n    총액합하다 <- 금액 + 세액.\n}.\n"}],"root_path":"root.ddn"}}"###;
    const DDL:&str = r###"#!日本語
(金額:有理数~に, 税額:有理数~を)
合計する:有理数:関数 :={...}.
"###;

 #[test] fn g2_actual_locator_and_missing_registry_are_separate(){
  let project=open(ORIGINAL.as_bytes()).unwrap();let m=project.authored_identity.modules.iter().find(|m|m.name=="세금").unwrap();
  let first=project.first_callable_registration_context(m.id.as_str(),"세금.ddn",DDL,"세금.ddl","language-profile:ja",1).unwrap();
  let raw="#!日本語\n(세금=\"./세금\", 옮김=(대상말씨=\"日本語\", 대상판본=\"first-callable-ja-r1\")) 쓰임.\n金額 <- (金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する.\n";
  let err=first.check_module_translation_request(raw,"利用.ddn").unwrap_err();assert!(err.starts_with("E_DDL_PROFILE_AND_EXPORTED_OWNER_REQUIRED"),"{err}");
  let virtual_source=snapshot_source(&project.source_snapshot).unwrap();let resolved=crate::local_project::resolve_first_callable_module_translation_request(raw,"利用.ddn",&virtual_source,&m.head,first.compiler_context()).unwrap();assert_eq!(resolved.head,m.head);assert_eq!(resolved.closure,m.sources);
  let wrong=first.check_module_translation_request(&raw.replace("./세금","./app"),"利用.ddn").unwrap_err();assert!(wrong.starts_with("E_DDL_ORIGINAL_MODULE_MISMATCH"),"{wrong}");
  let missing=first.check_module_translation_request(&raw.replace(", 옮김=(대상말씨=\"日本語\", 대상판본=\"first-callable-ja-r1\")",""),"利用.ddn").unwrap_err();assert!(missing.starts_with("E_DDL_ACTIVATION_REQUIRED"),"{missing}");
  let context=ddonirang_lang::ModuleDirectiveContext::default().with_known_imports(["./세금"]);let doc=ddonirang_lang::module_directive::parse_module_directive_prefix_with_first_callable_context(raw,"利用.ddn",&context,first.compiler_context()).unwrap();assert!(ddonirang_lang::canonicalize_module_directive_document(&doc).starts_with("#!日本語\n"));
  let texts=project.source_snapshot.files.iter().map(|f|(f.path.clone(),f.text.clone())).collect::<std::collections::BTreeMap<_,_>>();let prefix=raw.strip_prefix("#!日本語\n").unwrap().split("\n金額").next().unwrap();let rejected=crate::local_project::resolve_project_module_directives(prefix,"利用.ddn",&texts).unwrap_err();assert_eq!(rejected.code,"E_DDL_ACTIVATION_SUPPLY_REQUIRED");
  assert!(ddonirang_lang::ddl::FIRST_CALLABLE_PROFILE_MANIFEST.persistent_identity.is_none());assert!(ddonirang_lang::ddl::FIRST_CALLABLE_PROFILE_MANIFEST.exact_version.is_none());
 }
}

#[cfg(test)]
mod r14_public_ddl_tests {
use super::*;
    const ORIGINAL:&str = r###"{"authored_identity":{"clone_origin":null,"entry":null,"logical_dependency_id":"fe6b4154262752919656f5a6f7e3629b3e81758594783b9efca19a1f0ce9ed52","modules":[{"compositions":[],"head":"세금.ddn","id":"0a9e9104a2f6a6434c2bd0dd41a721a7dacc85125c791fe47aed1deb2f4f2068","name":"세금","sources":["세금.ddn"]},{"compositions":[],"head":"app.ddn","id":"36612e8bcf97c580cb38d1c3173b719c4acbf892a937bbe97e9df9f947521c8c","name":"앱","sources":["app.ddn"]}],"schema":"ddn.authored_identity.v1"},"content_sha256":"60783cb50f9b5e604197407d2c7241c990e67171ee5608e2b10593d9b7ee1659","schema":"ddn.authored_project_artifact.v1","source_profile":"v1-core-v25","source_snapshot":{"entry_path":"app.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\"), (이름=세금, 머리=\"세금.ddn\", 원문=\"세금.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n"},{"path":"app.ddn","text":"(세금=세금) 쓰임.\n실행:움직씨 := {\n    ((금액=(100, 1) 나눔수, 세액=(10, 1) 나눔수) 세금.총액합하다) 보여주기.\n}.\n"},{"path":"세금.ddn","text":"(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를)\n총액합하다{총액합하기}:나눔수:셈씨 := {\n    총액합하다 <- 금액 + 세액.\n}.\n"}],"root_path":"root.ddn"}}"###;
    const DDL:&str = r###"#!日本語
(金額:有理数~に, 税額:有理数~を)
合計する:有理数:関数 :={...}.
"###;

 fn context(project:&ProjectArtifact)->FirstCallableAuthoredContext{let module=project.authored_identity.modules.iter().find(|m|m.head=="세금.ddn").unwrap();project.first_callable_registration_context(module.id.as_str(),"세금.ddn",DDL,"세금.ddl","language-profile:ja",1).unwrap()}
 fn prepare(project:&ProjectArtifact,first:&FirstCallableAuthoredContext)->String{operate_with_first_callable_context("source_prepare",&serde_json::json!({"artifact_json_utf8":project.save().unwrap(),"expected_parent_hex":"","revision_hex":"05","previous_source_artifact_utf8":null}).to_string(),first).unwrap()}
 #[test]fn r14_public_export_uses_actual_g1_and_executes_same_japanese_source(){
  let original=open(ORIGINAL.as_bytes()).unwrap();let first=context(&original);let source=prepare(&original,&first);let export=first.produce_public_ddl(ORIGINAL,&source).unwrap();assert_ne!(export.pin_identities()[0],export.pin_identities()[1]);assert_eq!(export.original_module_identity(),first.module_identity().as_str());
  let again=first.produce_public_ddl(ORIGINAL,&source).unwrap();assert_eq!(again.definition_identity(),export.definition_identity());assert_eq!(again.pin_identities(),export.pin_identities());
  let raw=format!("#!日本語\n(세금=\"./세금\", 옮김=(대상말씨=\"日本語\", 대상판본=\"{}\")) 쓰임.\n金額 <- (金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する.\n",export.profile().version());
  let consumer=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:None,entry_path:"利用.ddn".into(),files:vec![SourceFile{path:"利用.ddn".into(),text:raw.clone()}]},"利用".into()).unwrap();let consumer=open(consumer.save().unwrap().as_bytes()).unwrap();
  let activated=export.activate_for_consumer(&consumer,consumer.authored_identity.modules[0].id.as_str(),"利用.ddn").unwrap();assert!(activated.public_export.is_some());
  let program=crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(&raw,"利用.ddn",std::sync::Arc::new(activated)).unwrap();
  let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
  let result=crate::ddn_runtime::DdnRunner::new(program,crate::runtime_surface::DEFAULT_UPDATE_NAME).run_update(&ddonirang_core::NuriWorld::new(),&input,&std::collections::HashMap::new()).unwrap();let ddonirang_lang::runtime::Value::Rational(value)=&result.resources["金額"] else{panic!("결과 형식")};assert_eq!(value.ratio().numer().to_string(),"110");assert_eq!(value.ratio().denom().to_string(),"1");
 }

 #[test]fn r14_v3_roundtrip_rebuilds_original_g1_and_refuses_forged_binding(){
  let original=open(ORIGINAL.as_bytes()).unwrap();let first=context(&original);let source=prepare(&original,&first);let export=first.produce_public_ddl(ORIGINAL,&source).unwrap();
  let raw=format!("#!日本語\n(세금=\"./세금\", 옮김=(대상말씨=\"日本語\", 대상판본=\"{}\")) 쓰임.\n金額 <- (金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する.\n",export.profile().version());
  let base=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:None,entry_path:"利用.ddn".into(),files:vec![SourceFile{path:"利用.ddn".into(),text:raw.clone()}]},"利用".into()).unwrap();let base=open(base.save().unwrap().as_bytes()).unwrap();let mid=base.authored_identity.modules[0].id.as_str();
  let v3=export.attach_v3(&base,mid,"利用.ddn").unwrap();let saved=v3.save().unwrap();let reopened=open(saved.as_bytes()).unwrap();assert_eq!(reopened.save().unwrap(),saved);assert_eq!(reopened.language_binding.as_ref().unwrap().original_project_json_utf8,ORIGINAL);
  let record=serde_json::to_string(&export.original_registration_record().unwrap()).unwrap();let restored=reopened.restore_language_context(serde_json::from_str(&record).unwrap()).unwrap();let actual=restored.public_export.as_ref().unwrap();assert_eq!(actual.definition_identity(),export.definition_identity());assert_eq!(actual.pin_identities(),export.pin_identities());assert_eq!(actual.facts.capture,export.facts.capture);
  crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(&raw,"利用.ddn",std::sync::Arc::new(restored.clone())).unwrap();assert!(crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(&raw.replace("(100,1)","(101,1)"),"利用.ddn",std::sync::Arc::new(restored)).is_err());
  for field in ["profile_identity","profile_version","profile_content_sha256","original_module_id","consumer_module_id"]{
   let mut value:Value=serde_json::from_str(&saved).unwrap();value["language_binding"][field]="forged".into();let mut sealed=value.clone();sealed.as_object_mut().unwrap().remove("content_sha256");value["content_sha256"]=hex::encode(Sha256::digest(serde_json::to_vec(&sealed).unwrap())).into();assert!(open(serde_json::to_vec(&value).unwrap().as_slice()).is_err(),"{field}");
  }
  let mut changed=reopened.clone();changed.source_snapshot.files[0].text=changed.source_snapshot.files[0].text.replace("옮김=","다른=");assert!(changed.save().is_err());
  let mut old:Value=serde_json::from_str(&base.save().unwrap()).unwrap();old["language_binding"]=Value::Null;assert!(open(serde_json::to_vec(&old).unwrap().as_slice()).is_err());
  let mut unknown:Value=serde_json::from_str(&saved).unwrap();unknown["language_binding"]["extra"]=true.into();assert!(open(serde_json::to_vec(&unknown).unwrap().as_slice()).is_err());
  let mut record_value:Value=serde_json::from_str(&record).unwrap();record_value["start_identity_hex"]="00".into();assert!(reopened.restore_language_context(serde_json::from_value(record_value).unwrap()).is_err());

  // The original v1 fixture must keep its exact canonical bytes. A v3
  // admission error must not become a migration or a synthetic Native start.
  assert_eq!(original.save().unwrap(),ORIGINAL);
  let duplicate=format!("{{\"schema\":\"{}\",{}",SCHEMA_V3,&saved[1..]);
  assert!(open(duplicate.as_bytes()).is_err());
  let mut too_large=reopened.clone();
  too_large.language_binding.as_mut().unwrap().ddl_source.text="a".repeat(16_385);
  assert_eq!(too_large.save().err().unwrap(),"E_DDL_V3_LIMIT");
  let mut nested=reopened.clone();
  nested.language_binding.as_mut().unwrap().original_project_json_utf8=saved.clone();
  assert_eq!(nested.save().err().unwrap(),"E_DDL_NESTED_ORIGINAL_FORBIDDEN");
  let mut deep=Value::Null;
  for _ in 0..33 {deep=Value::Array(vec![deep]);}
  assert_eq!(check_v3_json_depth(&deep).unwrap_err(),"E_DDL_V3_DEPTH");
  let native_error=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(saved.as_bytes()).err().unwrap();
  assert!(native_error.starts_with("E_PROJECT_RUNTIME_ROOT_REQUIRED"),"{native_error}");
  println!("R14_NATIVE_EXACT_REFUSAL={native_error}");
  assert!(operate_with_first_callable_context("source_prepare",&serde_json::json!({"artifact_json_utf8":saved,"expected_parent_hex":"","revision_hex":"09","previous_source_artifact_utf8":null}).to_string(),&first).is_err()); // No fake Native root or transplanted original owner.
 }

 fn native_fixture()->(ProjectArtifact,FirstCallableAuthoredContext){
  let original=open(ORIGINAL.as_bytes()).unwrap();let first=context(&original);let source=prepare(&original,&first);let export=first.produce_public_ddl(ORIGINAL,&source).unwrap();
  let raw=format!("(세금=\"./세금\", 옮김=(대상말씨=\"日本語\", 대상판본=\"{}\")) 쓰임.\n실행:움직씨 := {{ ((金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する) 보여주기. }}.\n",export.profile().version());
  let root="(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n";
  let base=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![SourceFile{path:"root.ddn".into(),text:root.into()},SourceFile{path:"main.ddn".into(),text:raw}]},String::new()).unwrap();
  let base=open(base.save().unwrap().as_bytes()).unwrap();
  let v3=export.attach_v3(&base,base.authored_identity.modules[0].id.as_str(),"main.ddn").unwrap();
  let v3=open(v3.save().unwrap().as_bytes()).unwrap();
  let first=v3.restore_language_context(export.original_registration_record().unwrap()).unwrap();
  (v3,first)
 }

 #[test]fn r14_source_candidate_cold_reopen_rebuilds_both_owners_without_new_ids(){
  use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as Source;
  let (v3,first)=native_fixture();
  let request=serde_json::json!({"artifact_json_utf8":v3.save().unwrap(),"expected_parent_hex":"","revision_hex":"09","previous_source_artifact_utf8":null}).to_string();
  let sealed=operate_with_first_callable_context("source_prepare",&request,&first).unwrap();
  let a=Source::from_artifact(&sealed).unwrap();let b=Source::from_artifact(&sealed).unwrap();
  assert_eq!(a.artifact().unwrap(),sealed);assert_eq!(b.artifact().unwrap(),sealed);assert_eq!(a.pesi(),b.pesi());
  assert_eq!(a.owned().definition_identity().as_bytes(),b.owned().definition_identity().as_bytes());
  let restored=a.owned().sources().first_callable_context().unwrap();
  assert_eq!(restored.saved_dependency_material().unwrap(),first.saved_dependency_material().unwrap());
  let reopened=operate("source_open",&serde_json::json!({"artifact_text":sealed,"project_hex":hex::encode(a.project()),"parent_hex":"","revision_hex":"09"}).to_string()).unwrap();
  assert_eq!(reopened,v3.save().unwrap());
  let raw=&v3.source_snapshot.files.iter().find(|f|f.path=="main.ddn").unwrap().text;
  crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(raw,"main.ddn",restored.clone()).unwrap();
  let original=open(ORIGINAL.as_bytes()).unwrap();let original_context=context(&original);let second_source=prepare(&original,&original_context);
  let second_export=original_context.produce_public_ddl(ORIGINAL,&second_source).unwrap();
  let second=second_export.activate_for_consumer(&v3,v3.authored_identity.modules[0].id.as_str(),"main.ddn").unwrap();
  assert_ne!(second.saved_dependency_material().unwrap(),first.saved_dependency_material().unwrap());
  let second_sealed=operate_with_first_callable_context("source_prepare",&request,&second).unwrap();
  let second_source=Source::from_artifact(&second_sealed).unwrap();
  assert_ne!(second_source.owned().definition_identity().as_bytes(),a.owned().definition_identity().as_bytes());

 }

 #[test]
 #[ignore="explicit child of r14_native_source_store_normal_close_fresh_process_reopen"]
 fn r14_native_cold_process_probe(){
  use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
  let root=std::path::PathBuf::from(std::env::var("DDN_R14_NATIVE_ROOT").unwrap());
  let expected:Value=serde_json::from_slice(&std::fs::read(root.join("expected.json")).unwrap()).unwrap();
  let project=hex::decode(expected["project"].as_str().unwrap()).unwrap();
  let integrity:[u8;32]=hex::decode(expected["integrity"].as_str().unwrap()).unwrap().try_into().unwrap();
  let adapter=SqliteAdapterV1::open_source_revision(&root).unwrap();
  let read=adapter.load_current_source_revision(&project,&[9],&integrity).unwrap();let source=read.source();
  assert_eq!(source.artifact().unwrap(),expected["source_artifact"].as_str().unwrap());
  assert_eq!(hex::encode(source.pesi()),expected["pesi"].as_str().unwrap());
  let owned=source.recorded_execution_owner().unwrap();
  assert_eq!(hex::encode(owned.definition_identity().as_bytes()),expected["consumer_start"].as_str().unwrap());
  let first=owned.sources().first_callable_context().unwrap();
  assert_eq!(first.saved_dependency_material().unwrap(),expected["original_dependency"]);
  let file=owned.project().source_snapshot.files.iter().find(|f|f.path=="main.ddn").unwrap();
  let program=crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(&file.text,&file.path,first.clone()).unwrap();
  let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
  let result=crate::ddn_runtime::DdnRunner::new(program,"실행").run_update(&ddonirang_core::NuriWorld::new(),&input,&std::collections::HashMap::new()).unwrap();
  assert_eq!(result.display_outputs[0].text,"110/1");
  println!("R14_NATIVE_SOURCE_COLD_REOPEN_OK pid={} output=110/1 pesi={} dependency={}",std::process::id(),hex::encode(source.pesi()),first.saved_dependency_material().unwrap());
 }
 #[test]fn r14_native_source_store_normal_close_fresh_process_reopen(){
  use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as Source;
  use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
  let (v3,first)=native_fixture();
  let sealed=operate_with_first_callable_context("source_prepare",&serde_json::json!({"artifact_json_utf8":v3.save().unwrap(),"expected_parent_hex":"","revision_hex":"09","previous_source_artifact_utf8":null}).to_string(),&first).unwrap();
  let sealed=if let Ok(path)=std::env::var("DDN_R14_PREPARED_CLI"){std::fs::read_to_string(path).unwrap()}else{sealed};
  let source=Source::from_artifact(&sealed).unwrap();let mut random=[0;16];getrandom::fill(&mut random).unwrap();
  let root=std::env::temp_dir().join(format!("NATIVE_{}",hex::encode(random)));
  std::fs::create_dir(&root).unwrap();
  let mut adapter=SqliteAdapterV1::provision_source_revision(&root).unwrap();
  let read=adapter.save_initial_source_revision(&source).unwrap();assert_eq!(read.source().payload(),source.payload());drop(read);
  assert!(adapter.load_current_source_revision(source.project(),&[9],&[0;32]).is_err());
  let expected=serde_json::json!({"project":hex::encode(source.project()),"integrity":hex::encode(source.integrity()),"pesi":hex::encode(source.pesi()),"source_artifact":sealed,
      "consumer_start":hex::encode(source.owned().definition_identity().as_bytes()),"original_dependency":source.owned().sources().first_callable_context().unwrap().saved_dependency_material().unwrap()});
  std::fs::write(root.join("expected.json"),serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
  drop(adapter); // Actual Native owner closes before a different process opens.
  let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","authored_project::r14_public_ddl_tests::r14_native_cold_process_probe","--ignored","--nocapture"]).env("DDN_R14_NATIVE_ROOT",&root).output().unwrap();
  std::fs::write(root.join("cold.stdout"),&output.stdout).unwrap();std::fs::write(root.join("cold.stderr"),&output.stderr).unwrap();
  assert!(output.status.success(),"{}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
  assert!(String::from_utf8_lossy(&output.stdout).contains("R14_NATIVE_SOURCE_COLD_REOPEN_OK"));
  println!("R14_NATIVE_SOURCE_RETAINED={}\n{}",root.display(),String::from_utf8_lossy(&output.stdout));
 }

 fn product_request()->Value{
  let original=open(ORIGINAL.as_bytes()).unwrap();let first=context(&original);let original_source=prepare(&original,&first);
  let (mut consumer,_)=native_fixture();consumer.schema=SCHEMA.into();consumer.language_binding=None;
  serde_json::json!({"original_project_json_utf8":ORIGINAL,"original_source_artifact_utf8":original_source,"original_source_path":"세금.ddn","ddl_source":{"path":"세금.ddl","text":DDL},
      "consumer_project_json_utf8":consumer.save().unwrap(),"consumer_source_path":"main.ddn","revision_hex":"09"})
 }
 #[test]fn r14_product_operations_prepare_reopen_run_and_reject_missing_pin(){
  let request=product_request();let sealed=operate("first_callable_source_prepare",&request.to_string()).unwrap();
  let run_request=serde_json::json!({"source_artifact_utf8":sealed});
  let result:Value=serde_json::from_str(&operate("first_callable_source_run",&run_request.to_string()).unwrap()).unwrap();
  assert_eq!(result["output_log_texts"],serde_json::json!(["110/1"]));assert_eq!(result["source_only"],true);
  let mut bad=request.clone();let mut consumer=open(request["consumer_project_json_utf8"].as_str().unwrap().as_bytes()).unwrap();
  let text=&mut consumer.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text;
  *text=text.replace(", 税額=(10,1) 有理数","");bad["consumer_project_json_utf8"]=consumer.save().unwrap().into();
  let error=operate("first_callable_source_prepare",&bad.to_string()).unwrap_err();println!("R14_PRODUCT_MISSING_PIN_REFUSAL={error}");
  assert!(error.contains("E_CALL_REQUIRED_PIN_MISSING"));assert!(error.contains("source=main.ddn encoding=utf-8-byte byte="));
  let detailed=prepare_first_callable_source_with_analysis(&bad.to_string()).unwrap_err();assert_eq!(detailed.message(),error);
  let analysis=detailed.analysis().unwrap();assert_eq!(analysis.outcome,"REJECTED");assert_eq!(analysis.analysis_generation,0);
  let diagnostic=analysis.diagnostics.iter().find(|d|d.code=="E_CALL_REQUIRED_PIN_MISSING").unwrap();
  assert_eq!(diagnostic.severity,"error");assert_eq!(diagnostic.path,"main.ddn");assert_eq!(diagnostic.span.encoding,"utf-8-byte");
  // Existing common analysis exposes a one-scalar location anchor; the raw
  // parser error retains the complete call range. Both must refer to the
  // same UTF-8 source start, without widening the shared snapshot contract.
  assert!(error.contains(&format!("byte={}..",diagnostic.span.start)));
  let text=&consumer.source_snapshot.files.iter().find(|f|f.path=="main.ddn").unwrap().text;
  assert_eq!(text[diagnostic.span.start..diagnostic.span.end].chars().count(),1);

  let malformed=prepare_first_callable_source_with_analysis("{}").unwrap_err();assert!(malformed.analysis().is_none());

  let out=std::env::temp_dir().join(format!("DDN_R14_CLI_INPUTS_{}", std::process::id()));std::fs::create_dir_all(&out).unwrap();
  for (name,value) in [("normal.json",request),("missing-pin.json",bad),("run.json",run_request)]{std::fs::write(out.join(name),serde_json::to_vec_pretty(&value).unwrap()).unwrap();}
  std::fs::write(out.join("source.json"),&sealed).unwrap();std::fs::write(out.join("missing-pin-error.txt"),&error).unwrap();
 }

 #[test]fn r14_native_product_operations_preserve_store_and_reject_wrong_context(){
  let sealed=operate("first_callable_source_prepare",&product_request().to_string()).unwrap();
  let mut random=[0;16];getrandom::fill(&mut random).unwrap();let root=std::env::temp_dir().join(format!("NATIVE_OPERATION_{}",hex::encode(random)));std::fs::create_dir(&root).unwrap();
  let request=serde_json::json!({"source_artifact_utf8":sealed,"storage_root":root}).to_string();
  let saved=operate("first_callable_source_save_native",&request).unwrap();assert_eq!(saved,sealed);
  assert!(operate("first_callable_source_save_native",&request).is_err());
  assert_eq!(operate("first_callable_source_reopen_native",&request).unwrap(),sealed);
  let other=operate("first_callable_source_prepare",&product_request().to_string()).unwrap();
  assert!(operate("first_callable_source_reopen_native",&serde_json::json!({"source_artifact_utf8":other,"storage_root":root}).to_string()).is_err());
  let missing=root.join("missing");assert!(operate("first_callable_source_reopen_native",&serde_json::json!({"source_artifact_utf8":sealed,"storage_root":missing}).to_string()).is_err());assert!(!missing.join("vc04_snapshot_v2.sqlite3").exists());
  let corrupt=root.join("corrupt");std::fs::create_dir(&corrupt).unwrap();std::fs::write(corrupt.join("vc04_snapshot_v2.sqlite3"),b"not a sqlite database").unwrap();
  assert!(operate("first_callable_source_reopen_native",&serde_json::json!({"source_artifact_utf8":sealed,"storage_root":corrupt}).to_string()).is_err());
  assert_eq!(operate("first_callable_source_reopen_native",&request).unwrap(),sealed);
  println!("R14_NATIVE_PRODUCT_STORE_RETAINED={}",root.display());
 }
 #[test]fn r14_actual_korean_start_executes_japanese_call_after_v3_codec_reopen(){
  let (v3,first)=native_fixture();let raw=&v3.source_snapshot.files.iter().find(|f|f.path=="main.ddn").unwrap().text;
  let first=std::sync::Arc::new(first);
  let admitted=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_first_callable(v3.save().unwrap().as_bytes(),first.clone()).unwrap();
  assert_eq!(admitted.sources().entry_declaration().name(),"실행");assert_eq!(admitted.sources().entry_declaration().path(),"main.ddn");
  let owned_program=admitted.sources().parse_prepared_program().unwrap();assert_eq!(owned_program.items.len(),1);
  assert!(!owned_program.items.iter().any(|ddonirang_lang::TopLevelItem::SeedDef(seed)|seed.canonical_name=="총액합하다"));
  let program=crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(raw,"main.ddn",first).unwrap();
  let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
  let result=crate::ddn_runtime::DdnRunner::new(program,"실행").run_update(&ddonirang_core::NuriWorld::new(),&input,&std::collections::HashMap::new()).unwrap();
  assert_eq!(result.display_outputs.len(),1);assert_eq!(result.display_outputs[0].text,"110/1");
  println!("R14_NATIVE_ENTRY_RUNTIME_OUTPUT={}",result.display_outputs[0].text);
 }
 #[test]fn r14_export_refuses_same_result_impostor_and_wrong_original_owner(){
  let original=open(ORIGINAL.as_bytes()).unwrap();let first=context(&original);let source=prepare(&original,&first);let other=original.duplicate().unwrap();assert!(context(&open(other.save().unwrap().as_bytes()).unwrap()).produce_public_ddl(&other.save().unwrap(),&source).is_err());
  let mut imposter=original.clone();let file=imposter.source_snapshot.files.iter_mut().find(|f|f.path=="세금.ddn").unwrap();file.text=file.text.replace("금액 + 세액","(110,1) 나눔수");let imposter=open(imposter.save().unwrap().as_bytes()).unwrap();let fake_context=context(&imposter);let fake_source=prepare(&imposter,&fake_context);let err=fake_context.produce_public_ddl(&imposter.save().unwrap(),&fake_source).err().unwrap();assert!(err.contains("E_DDL_FIRST_BODY_SCOPE"),"{err}");
 }
}

#[cfg(test)]
mod isolated_j1_original_tests{
    use super::*;
    const CREATE:&str="{\"source_profile\":\"v1-core-v25-ja-base-r1\",\"module_name\":\"\",\"source_snapshot\":{\"root_path\":\"root.ddn\",\"entry_path\":\"main.ddn\",\"files\":[{\"path\":\"root.ddn\",\"text\":\"#!日本語\\n(版=\\\"v25\\\", モジュール群=列[\\n  (名前=アプリ, 見出し=\\\"main.ddn\\\", 原文=\\\"main.ddn\\\"),\\n  (名前=計算, 見出し=\\\"calc.ddn\\\", 原文=\\\"calc.ddn\\\")\\n], 開始=(モジュール=アプリ, 定義=実行)) 構成.\\n\"},{\"path\":\"calc.ddn\",\"text\":\"#!日本語\\n(F=合計, G=倍) 公開.\\n(金額:有理数~に, 税額:有理数~を) 合計:有理数:関数 := {\\n  合計 <- 金額 + 税額.\\n}.\\n(金額:有理数~を) 倍:有理数:関数 := {\\n  倍 <- 金額 + 金額.\\n}.\\n(金額:有理数~を) 内部:有理数:関数 := {\\n  内部 <- 金額 + 金額.\\n}.\\n\"},{\"path\":\"main.ddn\",\"text\":\"#!日本語\\n(計算=計算) 使用.\\n実行:手続き := {\\n  ((税額=(3,1) 有理数, 金額=(12,1) 有理数) 計算.F) 表示.\\n  ((金額=(4,1) 有理数) 計算.G) 表示.\\n}.\\n\"}]}}";
    #[test]fn isolated_j1_projection_and_snapshot_keep_actual_profile(){
        let project=operate("create",CREATE).unwrap();
        let artifact=open(project.as_bytes()).unwrap();
        let projected=operate("source_project",&project).unwrap();
        let source:crate::local_project::VirtualProjectSource=serde_json::from_str(&projected).unwrap();
        assert_eq!(source.root_path,"root.ddn");
        assert_eq!(source.files.len(),artifact.source_snapshot.files.len());
        for (actual,original) in source.files.iter().zip(&artifact.source_snapshot.files){
            assert_eq!(actual.path,original.path);assert_eq!(actual.utf8_text,original.text);
        }
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(project.as_bytes()).unwrap();
        assert!(owned.sources().saved().j1_profile().is_some());
        crate::project_snapshot_identity::current::issue_saved_project(&owned).unwrap();
        assert_eq!(operate("open",&project).unwrap(),project);
    }
    #[test]fn isolated_j1_actual_original_record_roundtrip(){
        let project=operate("create",CREATE).unwrap();
        let artifact=open(project.as_bytes()).unwrap();
        let (source,path,start)=artifact.execution_source().unwrap();
        assert_eq!(start.as_deref(),Some("実行"));
        let result=crate::runtime_surface::run_summary_with_authored_bogae(&source,&path,Some(1),&artifact.source_profile,&project,None,None).unwrap();
        assert_eq!(result["engine_state"]["output_display"]["texts"],serde_json::json!(["15/1","8/1"]));
        let request=serde_json::json!({"original_project_json_utf8":project,"original_source_path":"calc.ddn",
            "ddl_source":{"path":"関数.ddl","text":"#!日本語\n(金額:有理数~に, 税額:有理数~を) 合計:有理数:関数 :={...}.\n(金額:有理数~を) 倍:有理数:関数 :={...}.\n"},
            "surfaces":[{"public_name":"F","surface":"合計","pins":[["金額","金額","に"],["税額","税額","を"]]},
                {"public_name":"G","surface":"倍","pins":[["金額","金額","を"]]}],"revision_hex":"41"});
        let raw=operate("public_callable_original_source_prepare",&request.to_string()).unwrap();
        let record=PublicCallableOriginalRecordV4::open(&raw).unwrap();
        let restored=record.restore().unwrap();assert_eq!(restored.profile().identity(),record.profile.identity);
        assert_eq!(restored.profile().version(),record.profile.version);assert_eq!(record.save().unwrap(),raw);
        let call="(계산=\"./calc.ddn\", 옮김=(대상말씨=\"日本語\", 대상판본=\"2\")) 쓰임.\n실행:움직씨 := { ((税額=(3,1) 有理数, 金額=(12,1) 有理数) 合計) 보여주기. ((金額=(4,1) 有理数) 倍) 보여주기. }.\n";
        let consumer=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n".into()},SourceFile{path:"main.ddn".into(),text:call.into()}]},String::new()).unwrap();
        let consumer_source=operate("public_callable_source_prepare",&serde_json::json!({"original_record_json_utf8":raw,"consumer_project_json_utf8":consumer.save().unwrap(),"consumer_source_path":"main.ddn","revision_hex":"42"}).to_string()).unwrap();
        let result=operate("public_callable_source_run",&serde_json::json!({"source_artifact_utf8":consumer_source}).to_string()).unwrap();
        let result:serde_json::Value=serde_json::from_str(&result).unwrap();assert_eq!(result["output_log_texts"],serde_json::json!(["15/1","8/1"]));

    }
}

