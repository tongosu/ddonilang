//! Adopted explicit v2 dependency transport. This private checked owner is
//! not public B1 admission; the independent material gate remains external.
//! Restore has no entropy/issuer argument and the record cannot activate DDL.
use super::RecordedRegistrationBlob;
use crate::authored_project::{ProjectArtifact,PublicCallableProfileRecordV4,FirstCallableDdlSource};
use ddonirang_lang::{ddl::{CheckedJ1Profile,SuccessorRegistryOwner},parser::RegisteredLogicCallableContext};
use ddonirang_lang::stdlib::logic_callable_registration_r1 as logic;
use sha2::{Digest,Sha256};
use std::sync::Arc;

pub(crate) const ROLE:&str="logic-callable-dependency-r1";
pub(crate) const PATH:&str=".ddn/registration/logic-callable-dependency.r1.json";
const SCHEMA:&str="ddn.logic_callable_dependency.r1";
const MAX:usize=131_072;
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema:String,builtin_owner:String,builtin_version:u32,builtin_material_sha256:String,
    profile:PublicCallableProfileRecordV4,ddl_source:FirstCallableDdlSource,
    consumer_project_id:String,consumer_module_id:String,consumer_source_path:String,
}
pub(crate) struct CheckedLogicDependency { record:Record }
fn error(message:&str)->String{format!("E_LOGIC_DEPENDENCY: {message}")}
fn id(raw:&str)->Result<[u8;32],String>{
    if raw.len()!=64||!raw.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){
        return Err(error("정체 표기가 다릅니다"));
    }
    let bytes:[u8;32]=hex::decode(raw).map_err(|_|error("정체 바이트"))?.try_into().map_err(|_|error("정체 길이"))?;
    if bytes==[0;32]{return Err(error("빈 정체"));}Ok(bytes)
}
fn source_owner<'a>(project:&'a ProjectArtifact,path:&str,module:&str)->Result<&'a str,String>{
    if project.source_profile!=ddonirang_lang::ddl::J1_SOURCE_PROFILE
        ||!matches!(project.schema.as_str(),crate::authored_project::SCHEMA|crate::authored_project::SCHEMA_V2)||path.len()>1024||path.is_empty(){
        return Err(error("작품 판본·원문 프로필·경로가 다릅니다"));
    }
    let owners=project.authored_identity.modules.iter().filter(|m|m.sources.iter().any(|p|p==path)).collect::<Vec<_>>();
    let files=project.source_snapshot.files.iter().filter(|f|f.path==path).collect::<Vec<_>>();
    if owners.len()!=1||owners[0].id.as_str()!=module||files.len()!=1{
        return Err(error("소비 원문의 실제 소유자가 다릅니다"));
    }
    // The explicit provider cannot shadow an authored module.
    if project.authored_identity.modules.iter().any(|m|m.name=="logic"){
        return Err("E_LOGIC_PROVIDER_COLLISION".into());
    }
    Ok(&files[0].text)
}
impl CheckedLogicDependency {
    pub(crate) fn consumer_source_path(&self)->&str{&self.record.consumer_source_path}

    /// Produce only from the actual retained compiler/source owner, after its
    /// existing source and registration preflight. No profile issuance here.
    pub(crate) fn from_source(source:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1,
        path:&str)->Result<Self,String>{
        if source.sources().linked_logic_path().is_some_and(|owner|owner!=path){
            return Err(error("연결된 소비 원문의 소유자가 다릅니다"));
        }
        let context=source.sources().logic_callable_context().ok_or_else(||error("검증된 논리 공급이 없습니다"))?;
        let activation=context.activation().ok_or_else(||error("명시 활성화가 없습니다"))?;
        if activation.target!="logic"{return Err(error("명시 builtin 선택자가 다릅니다"));}
        let project=source.project();
        let owners=project.authored_identity.modules.iter().filter(|m|m.sources.iter().any(|p|p==path)).collect::<Vec<_>>();
        if owners.len()!=1{return Err(error("원문 소유 모듈이 유일하지 않습니다"));}
        let profile=context.profile();let (_,digest)=logic::material(1).ok_or_else(||error("물리 판본이 없습니다"))?;
        let record=Record{schema:SCHEMA.into(),builtin_owner:hex::encode(context.builtin_owner()),builtin_version:1,
            builtin_material_sha256:digest.into(),profile:PublicCallableProfileRecordV4{
                identity:profile.identity().into(),version:profile.version().into(),registration_revision:profile.registration_revision(),
                content_utf8:std::str::from_utf8(profile.content()).map_err(|_|error("프로필 UTF8"))?.into(),content_sha256:profile.content_sha256().into()},
            ddl_source:FirstCallableDdlSource{path:context.ddl_path().into(),text:context.ddl_source().into()},
            consumer_project_id:project.authored_identity.logical_dependency_id.as_str().into(),
            consumer_module_id:owners[0].id.as_str().into(),consumer_source_path:path.into()};
        let checked=Self{record};checked.restore_context(project,&[])?;checked.bytes()?;Ok(checked)
    }
    pub(crate) fn bytes(&self)->Result<Vec<u8>,String>{
        let raw=serde_json::to_vec(&serde_json::to_value(&self.record).map_err(|_|error("record value"))?).map_err(|_|error("기록 인코딩"))?;
        if raw.len()>MAX{return Err(error("기록 한도"));}Ok(raw)
    }
    /// Strict nested duplicate/unknown/null rejection before any typed owner.
    pub(crate) fn read(raw:&[u8])->Result<Self,String>{
        if raw.is_empty()||raw.len()>MAX||raw.starts_with(&[0xef,0xbb,0xbf]){return Err(error("기록 한도·BOM"));}
        let strict:crate::authored_project::StrictValue=serde_json::from_slice(raw).map_err(|_|error("기록 JSON·중복 필드"))?;
        crate::authored_project::check_v3_json_depth(&strict.0).map_err(|_|error("기록 깊이 한도"))?;
        let record:Record=serde_json::from_value(strict.0).map_err(|_|error("기록 필드"))?;
        let checked=Self{record};
        if checked.bytes()?.as_slice()!=raw{return Err(error("기록 정본 바이트"));}Ok(checked)
    }
    pub(crate) fn into_blob(self)->Result<RecordedRegistrationBlob,String>{
        let raw=self.bytes()?;Ok(RecordedRegistrationBlob{logical_path:PATH.into(),role:Some(ROLE.into()),
            byte_length:raw.len() as u64,sha256:hex::encode(Sha256::digest(&raw)),raw_hex:hex::encode(raw)})
    }
    pub(crate) fn from_blob(row:&RecordedRegistrationBlob)->Result<Self,String>{
        if row.role.as_deref()!=Some(ROLE)||row.logical_path!=PATH||row.byte_length==0||row.byte_length>MAX as u64
            ||row.raw_hex.len()!=row.byte_length as usize*2||!row.raw_hex.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){
            return Err(error("의존 role·경로·길이"));
        }
        let raw=hex::decode(&row.raw_hex).map_err(|_|error("기록 바이트"))?;
        if hex::encode(Sha256::digest(&raw))!=row.sha256{return Err(error("기록 무결성"));}Self::read(&raw)
    }
    /// Exact physical/profile restore; source `使用…翻訳` is the sole
    /// activation channel. Names and paths never mint identities.
    pub(crate) fn restore_context(&self,project:&ProjectArtifact,reserved:&[[u8;32]])
        ->Result<Arc<RegisteredLogicCallableContext>,String>{self.restore_context_selected(project,reserved,None,"source_restore")}
    pub(crate) fn restore_context_diagnostic(&self,project:&ProjectArtifact,reserved:&[[u8;32]])
        ->Result<Arc<RegisteredLogicCallableContext>,crate::preprocess::ExecutionDiagnostic>{
        let mut diagnostic=None;
        self.restore_context_selected(project,reserved,Some(&mut diagnostic),"source_restore").map_err(|message|
            diagnostic.unwrap_or_else(||dependency_diagnostic(message,"source_restore")))
    }
    fn restore_context_selected(&self,project:&ProjectArtifact,reserved:&[[u8;32]],
        mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>, phase:&'static str)->Result<Arc<RegisteredLogicCallableContext>,String>{
        let r=&self.record;
        if r.schema!=SCHEMA||r.builtin_version!=1||r.profile.content_utf8.len()>65_536
            ||r.ddl_source.text.len()>16_384||r.ddl_source.path.len()>1024||r.ddl_source.path.is_empty()
            ||r.consumer_project_id!=project.authored_identity.logical_dependency_id.as_str(){return Err(error("판본·한도·작품 소유"));}
        id(&r.consumer_project_id)?;id(&r.consumer_module_id)?;
        let source=source_owner(project,&r.consumer_source_path,&r.consumer_module_id)?;
        let (raw,digest)=logic::material(r.builtin_version).ok_or_else(||error("물리 판본"))?;
        if r.builtin_material_sha256!=digest{return Err(error("물리자료 무결성"));}
        let mut reserved=reserved.to_vec();reserved.push(id(&r.consumer_project_id)?);
        for module in &project.authored_identity.modules{
            reserved.push(id(module.id.as_str())?);
            for definition in &module.compositions{reserved.push(id(definition.id.as_str())?);}
        }
        let material:serde_json::Value=serde_json::from_str(raw).map_err(|_|error("fixed material JSON"))?;
        let owner=id(material["owner"].as_str().ok_or_else(||error("fixed owner"))?)?;
        let rows=material["definitions"].as_array().ok_or_else(||error("fixed definitions"))?;
        let definitions=rows.iter().map(|row|{
            let definition=id(row["definition"].as_str().ok_or_else(||error("fixed definition"))?)?;
            logic::lookup(&owner,&definition,1,digest,&reserved)
        }).collect::<Result<Vec<_>,String>>()?;
        if definitions.len()!=4||definitions.iter().any(|d|hex::encode(d.owner())!=r.builtin_owner){return Err(error("builtin 소유자·공개 분모"));}
        let p=&r.profile;let mut registry=SuccessorRegistryOwner::default();
        let profile=registry.restore_logic_recorded_exact(&p.identity,&p.version,p.registration_revision,
            &p.content_utf8,&p.content_sha256,&definitions).map_err(|e|e.to_string())?;
        let syntax=CheckedJ1Profile::resolve(&project.source_profile).map_err(|e|e.to_string())?;
        let doc=ddonirang_lang::Parser::logic_callable_module_prefix(source,&r.consumer_source_path,syntax)
            .map_err(|e|keep_parse_failure(e,&r.consumer_source_path,&mut diagnostic))?;
        let import=doc.imports.ok_or_else(||error("원문 활성화가 없습니다"))?;
        if import.entries.len()!=1||import.entries[0].target!="logic"{return Err(error("원문 builtin 결속"));}
        let translation=import.translation.ok_or_else(||error("원문 翻訳가 없습니다"))?;
        if translation.target_language!=profile.canonical_name()||translation.target_version!=profile.version(){
            let message=error("원문과 기록의 말씨·판본이 다릅니다");
            if let Some(output)=diagnostic.as_deref_mut(){
                let mut failure=dependency_diagnostic(message.clone(),phase);
                failure.source_path=r.consumer_source_path.clone();
                failure.span=Some((translation.span.start,translation.span.end));
                *output=Some(failure);
            }
            return Err(message);
        }
        let context=Arc::new(RegisteredLogicCallableContext::from_exported_ddl(profile,definitions,
            &r.ddl_source.text,&r.ddl_source.path,syntax).map_err(|e|keep_parse_failure(e,&r.ddl_source.path,&mut diagnostic))?
            .with_module_activation(import.entries[0].clone()).map_err(|e|keep_parse_failure(e,&r.consumer_source_path,&mut diagnostic))?);
        // Same actual text/parser/byte spans; never synthesizes a directive.
        ddonirang_lang::Parser::parse_with_logic_callable_context(source,&r.consumer_source_path,syntax,context.clone())
            .map_err(|e|keep_parse_failure(e,&r.consumer_source_path,&mut diagnostic))?;
        Ok(context)
    }
}

impl super::RegistrationDraft {
    pub(crate) fn attach_logic_dependency(&mut self,source:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1,path:&str)->Result<(),String>{
        self.verify_precanon_binding(source)?;
        if self.artifacts.iter().any(|b|b.role==Some(ROLE)||b.path.eq_ignore_ascii_case(PATH))
            ||source.project().source_snapshot.files.iter().any(|f|f.path.eq_ignore_ascii_case(PATH)){
            return Err(error("의존기록 경로·role 중복"));
        }
        let row=CheckedLogicDependency::from_source(source,path)?.into_blob()?;
        let raw=hex::decode(&row.raw_hex).map_err(|_|error("의존 바이트"))?;
        if self.artifacts.len()>=257||self.artifacts.iter().map(|b|b.bytes.len()).sum::<usize>()+raw.len()>2_000_000{return Err(error("등록자료 전체 한도"));}
        self.artifacts.push(super::RegistrationBlob{path:PATH.into(),role:Some(ROLE),sha:Sha256::digest(&raw).into(),bytes:raw});
        self.artifacts.sort_by(|a,b|a.path.cmp(&b.path));self.verify_logic_dependency(source)
    }
    pub(crate) fn verify_logic_dependency(&self,source:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1)->Result<(),String>{
        self.verify_precanon_binding(source)?;
        let rows=self.artifacts.iter().filter(|b|b.role==Some(ROLE)).collect::<Vec<_>>();
        let Some(actual)=source.sources().logic_callable_context()else{
            return if rows.is_empty(){Ok(())}else{Err(error("원문 공급 없는 의존기록"))};
        };
        if rows.len()!=1||rows[0].path!=PATH{return Err("E_LOGIC_STORAGE_CONTEXT_REQUIRED: 명시 의존기록 한 건이 필요합니다".into());}
        let mut reserved=Vec::new();
        if let Some(registry)=self.registry{reserved.push(registry);}
        for schema in &self.schemas{reserved.extend([schema.module,schema.definition,schema.name_type]);for pin in &schema.slots{reserved.push(pin.id);}}
        reserved.extend(self.atoms.iter().map(|a|a.id));
        let recorded=CheckedLogicDependency::read(&rows[0].bytes)?;
        if source.sources().linked_logic_path().is_some_and(|path|path!=recorded.consumer_source_path()){
            return Err(error("연결된 소비 원문의 소유자가 다릅니다"));
        }
        let restored=recorded.restore_context(source.project(),&reserved)?;
        if restored.profile().identity()!=actual.profile().identity()
            ||restored.profile().content()!=actual.profile().content()
            ||restored.ddl_source()!=actual.ddl_source()||restored.ddl_path()!=actual.ddl_path()
            ||restored.activation()!=actual.activation(){return Err(error("현재 공급과 기록이 다릅니다"));}
        Ok(())
    }
}

#[cfg(test)]
mod production_material_tool {
    use super::*;
    /// Explicit one-time material production, never part of ordinary tests.
    /// A create_new reservation precedes issuance; the caller retains output.
    /// This emits lexical profile material only, not physical review/admission.
    #[test]
    #[ignore = "explicit first issuance requires a fresh DDN_B1_PROFILE_OUTPUT path"]
    fn r31_logic_dependency_issue_production_profile_once(){
        use std::io::Write;
        let path=std::env::var_os("DDN_B1_PROFILE_OUTPUT").expect("explicit output path required");
        let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(path).expect("fresh output path required before issue");
        let (raw,digest)=logic::material(1).unwrap();let material:serde_json::Value=serde_json::from_str(raw).unwrap();
        let owner=id(material["owner"].as_str().unwrap()).unwrap();
        let definitions=material["definitions"].as_array().unwrap().iter().map(|row|
            logic::lookup(&owner,&id(row["definition"].as_str().unwrap()).unwrap(),1,digest,&[]).unwrap()).collect::<Vec<_>>();
        let mut registry=SuccessorRegistryOwner::default();let mut calls=0;
        let profile=registry.issue_logic(&definitions,&mut |bytes|{calls+=1;getrandom::fill(bytes).map_err(|_|"OS entropy unavailable".into())}).unwrap();
        let record=PublicCallableProfileRecordV4{identity:profile.identity().into(),version:profile.version().into(),registration_revision:profile.registration_revision(),content_utf8:std::str::from_utf8(profile.content()).unwrap().into(),content_sha256:profile.content_sha256().into()};
        let bytes=serde_json::to_vec(&serde_json::to_value(&record).unwrap()).unwrap();
        // Reconstruct using restore only; the first-production identity is
        // retained in material, never recomputed from a name or checksum.
        let restored=SuccessorRegistryOwner::default().restore_logic_recorded_exact(&record.identity,&record.version,record.registration_revision,&record.content_utf8,&record.content_sha256,&definitions).unwrap();
        assert_eq!(restored.identity(),profile.identity());
        file.write_all(&bytes).unwrap();file.sync_all().unwrap();
        println!("R31_B1_PROFILE_PRODUCTION_RAW {}",serde_json::json!({"profile":profile.identity(),"version":profile.version(),"content_sha256":profile.content_sha256(),"entropy_calls":calls,"definitions":4,"owned_pins":6,"builtin_reissued":0,"public_admission":false}));
    }
}

const PROFILE_MATERIAL:&str=include_str!("logic_profile_r1.json");
const PROFILE_MATERIAL_SHA:&str="68a8c43c648a55240fd503e24508993c5b00ca11e3352a003c3d839af24f8ed6";
fn fixed_profile()->Result<PublicCallableProfileRecordV4,String>{
    if hex::encode(Sha256::digest(PROFILE_MATERIAL.as_bytes()))!=PROFILE_MATERIAL_SHA{return Err(error("고정 프로필 무결성"));}
    let strict:crate::authored_project::StrictValue=serde_json::from_str(PROFILE_MATERIAL).map_err(|_|error("고정 프로필 필드"))?;
    serde_json::from_value(strict.0).map_err(|_|error("고정 프로필 닫힌 필드"))
}
impl CheckedLogicDependency {
    /// Actual production lexical material, not the tests' fixture08 identity.
    /// Still private: physical verdict precedes public provider admission.
    fn for_project_record(project:&ProjectArtifact,path:&str,ddl_text:&str,ddl_path:&str)->Result<Self,String>{
        let owners=project.authored_identity.modules.iter().filter(|m|m.sources.iter().any(|p|p==path)).collect::<Vec<_>>();
        if owners.len()!=1{return Err(error("소비 모듈이 유일하지 않습니다"));}
        let (raw,digest)=logic::material(1).ok_or_else(||error("고정 builtin 판본"))?;
        let material:serde_json::Value=serde_json::from_str(raw).map_err(|_|error("고정 builtin 자료"))?;
        let checked=Self{record:Record{schema:SCHEMA.into(),builtin_owner:material["owner"].as_str().ok_or_else(||error("고정 builtin owner"))?.into(),builtin_version:1,builtin_material_sha256:digest.into(),profile:fixed_profile()?,
            ddl_source:FirstCallableDdlSource{path:ddl_path.into(),text:ddl_text.into()},consumer_project_id:project.authored_identity.logical_dependency_id.as_str().into(),consumer_module_id:owners[0].id.as_str().into(),consumer_source_path:path.into()}};
        checked.bytes()?;Ok(checked)
    }
    pub(crate) fn for_project(project:&ProjectArtifact,path:&str,ddl_text:&str,ddl_path:&str)->Result<Self,String>{
        let checked=Self::for_project_record(project,path,ddl_text,ddl_path)?;checked.restore_context(project,&[])?;Ok(checked)
    }
    pub(crate) fn for_project_diagnostic(project:&ProjectArtifact,path:&str,ddl_text:&str,ddl_path:&str)
        ->Result<(Self,Arc<RegisteredLogicCallableContext>),crate::preprocess::ExecutionDiagnostic>{
        let checked=Self::for_project_record(project,path,ddl_text,ddl_path).map_err(|message|dependency_diagnostic(message,"source_prepare"))?;
        let mut diagnostic=None;
        let context=checked.restore_context_selected(project,&[],Some(&mut diagnostic),"source_prepare").map_err(|message|
            diagnostic.unwrap_or_else(||dependency_diagnostic(message,"source_prepare")))?;
        Ok((checked,context))
    }
    pub(crate) fn verify_production_profile(&self)->Result<(),String>{
        let actual=serde_json::to_vec(&self.record.profile).map_err(|_|error("프로필 인코딩"))?;
        let fixed=serde_json::to_vec(&fixed_profile()?).map_err(|_|error("고정 프로필 인코딩"))?;
        if actual!=fixed{return Err(error("생산 프로필 정체·판본·내용 불일치"));}Ok(())
    }
}

impl super::RegistrationDraft {
    pub(crate) fn verify_production_logic_dependency(&self,source:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1)->Result<(),String>{
        self.verify_logic_dependency(source)?;
        let rows=self.artifacts.iter().filter(|row|row.role==Some(ROLE)).collect::<Vec<_>>();
        if rows.len()!=1{return Err(error("생산 의존기록 한 건이 필요합니다"));}
        CheckedLogicDependency::read(&rows[0].bytes)?.verify_production_profile()
    }
}

/// Direct conversion of the actual producer error. No formatted error report
/// parsing, post-failure analysis, alternate source or synthesized byte range.
pub(crate) fn keep_parse_failure(error:ddonirang_lang::ParseError,path:&str,
    output:&mut Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->String{
    let message=error.to_string();
    if let Some(output)=output.as_deref_mut(){
        if output.is_none(){*output=Some(crate::preprocess::ExecutionDiagnostic{
            code:error.j1_source_code(),message:message.clone(),source_path:path.into(),span:Some((error.span.start,error.span.end)),
            phase:"lang_parse",cause:None,offending_field:None,expected:None,actual:None});}
    }
    message
}
pub(crate) fn dependency_diagnostic(message:String,phase:&'static str)->crate::preprocess::ExecutionDiagnostic{
    crate::preprocess::ExecutionDiagnostic{code:"E_LOGIC_DEPENDENCY",message,source_path:String::new(),span:None,
        phase,cause:None,offending_field:None,expected:None,actual:None}
}
