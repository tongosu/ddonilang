//! Alpha2 authored identity storage. This is admission, not composition activation.
//! Locators and source bytes never supply persistent identities.
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
pub(crate) mod registration;
mod source_save;
mod source_projection;
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

impl ProjectArtifact {
    fn validate(&self) -> Result<(), String> {
        if self.schema!=SCHEMA_V3 && self.language_binding.is_some(){return Err("E_DDL_BINDING_SCHEMA: 기존 판본은 언어 결속 필드를 받지 않습니다".into());}
        if !matches!(
            (self.schema.as_str(), self.authored_identity.schema.as_str()),
            (SCHEMA, "ddn.authored_identity.v1") | (SCHEMA_V2, "ddn.authored_identity.v2") | (SCHEMA_V3,"ddn.authored_identity.v1")
        ) {
            return Err(fail("지원하지 않는 저장 버전입니다"));
        }
        if !matches!(
            self.source_profile.as_str(),
            "v1-core-v25" | "v1-core-v25-logical-alpha2"
        ) {
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
            let graph = snapshot_graph(&self.source_snapshot, &self.source_profile)?;
            if owned.contains(root_path) {
                return Err(fail("작품 뿌리를 모듈 원문으로 소유할 수 없습니다"));
            }
            expected_owned.remove(root_path);
            // Package custody is sealed by source_snapshot and verified by the
            // project graph, but is not an invented executable module source.
            for file in snapshot_source(&self.source_snapshot)?.files {
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

    pub fn save(&self) -> Result<String, String> {
        self.validate()?;
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
        let modules = if source_snapshot.root_path.is_some() {
            if !module_name.is_empty() {
                return Err(fail(
                    "여러 모듈의 이름은 얽음에 선언합니다. 별도 모듈 이름을 덮어쓸 수 없습니다",
                ));
            }
            let graph = snapshot_graph(&source_snapshot, &source_profile)?;
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
        project.validate()?;
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
        let public = ddonirang_lang::parse_module_directive_frontdoor_with_mode(&source.text, source_path,
            ddonirang_lang::ParseMode::Strict,
            &ddonirang_lang::ModuleDirectiveContext::default().with_public_definitions(["총액합하다"]))
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
        if self.source_snapshot.root_path.is_none() {
            return Ok(None);
        }
        crate::local_project::prepare_authored_execution(snapshot_source(&self.source_snapshot)?)
            .map(Some)
            .map_err(|error| fail(&format!("{}: {}", error.code, error.message)))
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
    let source = snapshot_source(snapshot)?;
    crate::local_project::project_source_profile(&source, Some(profile))
        .map_err(|error| fail(&format!("{}: {}", error.code, error.message)))?;
    crate::local_project::build_project_graph(source)
        .map_err(|error| fail(&format!("{}: {}", error.code, error.message)))
}

fn snapshot_source(
    snapshot: &SourceSnapshot,
) -> Result<crate::local_project::VirtualProjectSource, String> {
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
    let metadata_roles = local_project::v25_package_metadata_roles(&root_text.text)
        .map_err(|e| fail(&format!("{}: {}", e.code, e.message)))?;
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
    if source.files.iter().any(|file| file.role == "ddn_source"
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

pub fn open(bytes: &[u8]) -> Result<ProjectArtifact, String> {
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
    project.validate()?;
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

pub fn operate(operation: &str, text: &str) -> Result<String, String> {
    match operation {
        // Private Workbench transport operations, not DDN callables or new
        // CLI command names. Source candidates grant no execution authority.
        "first_callable_source_prepare"=>source_save::prepare_first_callable_product(text),
        "first_callable_source_run"=>source_save::run_first_callable_product(text),
        "first_callable_source_save_native"=>source_save::save_first_callable_native(text),
        "first_callable_source_reopen_native"=>source_save::reopen_first_callable_native(text),
        "source_prepare"=>source_save::prepare(text),
        "source_open"=>source_save::open(text),
        "source_probe"=>source_save::requirements(text),
        "source_project"=>source_projection::project(text),
        "create" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("작성 요청 JSON을 확인하세요"))?;
            let request: CreateRequest =
                serde_json::from_value(value.0).map_err(|_| fail("작성 요청 필드를 확인하세요"))?;
            ProjectArtifact::create(
                request.source_profile,
                request.source_snapshot,
                request.module_name,
            )?
            .save()
        }
        "open" => open(text.as_bytes())?.save(),
        "clone" => open(text.as_bytes())?.duplicate()?.save(),
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
            let mut project = open(request.original_json.as_bytes())?;
            if project.source_snapshot.root_path != request.source_snapshot.root_path {
                return Err(fail("원문 갱신으로 작품 뿌리를 추가·제거·바꿀 수 없습니다"));
            }
            project.source_snapshot = request.source_snapshot;
            crate::authored_bogae::refresh_source_locks(&mut project)?;
            // Module ownership is retained. File moves/ownership changes must
            // use an explicit edit operation, never infer new IDs from paths.
            project.save()
        }
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
