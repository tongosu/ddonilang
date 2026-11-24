//! Explicit, world-free code source custody. No Root/Start/PESI is fabricated.
use sha2::{Digest,Sha256};
pub(crate) const SCHEMA:&str="ddn.code_source_only_artifact.v1";
const MAX:usize=2_000_000;
#[derive(Clone,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CodeSource {
    schema:String,pub(crate) project_hex:String,pub(crate) parent_hex:String,
    pub(crate) revision_hex:String,pub(crate) profile:String,
    pub(crate) authored_project_json_utf8:String,integrity_hex:String,
}
// Checked data custody is not current-language admission or execution permission.
// Native DB/parent/anchor/grant checks remain outside this structural decoder.
pub(crate) fn custody_project(raw:&str)->Result<Option<crate::authored_project::ProjectArtifact>,String>{
    let p=crate::authored_project::open(raw.as_bytes())?;
    if p.source_profile!=crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY
        ||p.source_snapshot.root_path.is_some(){return Ok(None);}
    if p.source_snapshot.files.iter().any(|f|ddonirang_lang::has_model_surface(&f.text)){return Ok(None);}
    if p.source_snapshot.files.len()!=1||p.authored_identity.modules.len()!=1
        ||p.authored_identity.modules[0].sources!=vec![p.source_snapshot.entry_path.clone()]
        ||p.source_snapshot.files[0].path!=p.source_snapshot.entry_path {
        return Err("E_CODE_SOURCE_LAYOUT_UNSUPPORTED: 일반 단독 원문 한 파일을 지정하세요".into());
    }
    Ok(Some(p))
}
pub(crate) fn checked_project(raw:&str)->Result<Option<crate::authored_project::ProjectArtifact>,String>{
    let Some(p)=custody_project(raw)? else{return Ok(None)};
    let required=crate::authored_project::operate("source_probe",&serde_json::json!({"artifact_json_utf8":raw}).to_string())?;
    if required!="false"{
        return Err("E_CODE_SOURCE_REGISTRATION_REQUIRED: 별도 등록이 필요한 원문은 기존 등록 저장 경로를 사용하세요".into());
    }
    Ok(Some(p))
}
impl CodeSource {
    pub(crate) fn prepare(raw:&str,parent:&str,revision:&str)->Result<Self,String>{
        if raw.len()>MAX{return Err("E_CODE_SOURCE_LIMIT".into());}
        let p=checked_project(raw)?.ok_or("E_CODE_SOURCE_REQUIRED")?;
        Self::from_project(p,raw,parent,revision)
    }
    fn from_project(p:crate::authored_project::ProjectArtifact,raw:&str,parent:&str,revision:&str)->Result<Self,String>{
        let id=|x:&str|x.len()==32&&x.bytes().all(|c|c.is_ascii_digit()||(b'a'..=b'f').contains(&c));
        if !id(revision)||(!parent.is_empty()&&!id(parent))||parent==revision{return Err("E_CODE_SOURCE_REVISION".into());}
        let mut result=Self{schema:SCHEMA.into(),project_hex:p.authored_identity.logical_dependency_id.as_str().to_owned(),
            parent_hex:parent.into(),revision_hex:revision.into(),profile:p.source_profile.clone(),
            authored_project_json_utf8:raw.into(),integrity_hex:String::new()};
        result.integrity_hex=hex::encode(Sha256::digest(result.encode()?.as_bytes()));
        if result.encode()?.len()>MAX{return Err("E_CODE_SOURCE_LIMIT".into());}Ok(result)
    }
    pub(crate) fn encode(&self)->Result<String,String>{serde_json::to_string(self).map_err(|_|"E_CODE_SOURCE_ENCODING".into())}
    pub(crate) fn decode(raw:&str)->Result<Self,String>{
        if raw.len()>MAX||raw.starts_with('\u{feff}'){return Err("E_CODE_SOURCE_LIMIT".into());}
        let r:Self=serde_json::from_str(raw).map_err(|_|"E_CODE_SOURCE_FIELDS")?;
        if r.schema!=SCHEMA||r.encode()?!=raw{return Err("E_CODE_SOURCE_CANONICAL".into());}
        // Rechecking today's grammar would make a valid historical revision
        // unreadable, and also break every newer head that retains it as parent.
        // Verify the same canonical fields/hash/identity/layout without granting
        // fresh admission. New writes still use checked_project in prepare.
        let p=custody_project(&r.authored_project_json_utf8)?.ok_or("E_CODE_SOURCE_REQUIRED")?;
        let checked=Self::from_project(p,&r.authored_project_json_utf8,&r.parent_hex,&r.revision_hex)?;
        if checked.encode()?!=raw{return Err("E_CODE_SOURCE_INTEGRITY".into());}Ok(checked)
    }
    pub(crate) fn anchor(&self)->Result<Anchor,String>{Ok(Anchor{
        revision:hex::decode(&self.revision_hex).map_err(|_|"E_CODE_SOURCE_REVISION")?.try_into().map_err(|_|"E_CODE_SOURCE_REVISION")?,
        integrity:hex::decode(&self.integrity_hex).map_err(|_|"E_CODE_SOURCE_INTEGRITY")?.try_into().map_err(|_|"E_CODE_SOURCE_INTEGRITY")?})}
    pub(crate) fn project(&self)->Result<[u8;32],String>{hex::decode(&self.project_hex).map_err(|_|"E_CODE_SOURCE_PROJECT")?.try_into().map_err(|_|"E_CODE_SOURCE_PROJECT".into())}
}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Anchor {pub(crate) revision:[u8;16],pub(crate) integrity:[u8;32]}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) const SOURCE:&str=include_str!("../../../tests/fixtures/rd714_native_course.또니");
    pub(crate) fn artifact(text:&str)->String {crate::authored_project::operate("create",&serde_json::json!({"source_profile":"v1-core-v25","module_name":"앱","source_snapshot":{"root_path":null,"entry_path":"두배와나누기.또니","files":[{"path":"두배와나누기.또니","text":text}]}}).to_string()).unwrap()}
    pub(crate) fn edited(raw:&str,text:&str)->String {crate::authored_project::operate("update",&serde_json::json!({"original_json":raw,"source_snapshot":{"root_path":null,"entry_path":"두배와나누기.또니","files":[{"path":"두배와나누기.또니","text":text}]}}).to_string()).unwrap()}

    #[test]
    fn code_source_legacy_custody_read_is_not_new_language_admission(){
        let raw=include_str!("../../../tests/fixtures/구판_미등록조사_원문보관.json");
        let read=CodeSource::decode(raw).unwrap();
        assert_eq!(read.encode().unwrap(),raw);
        assert!(read.authored_project_json_utf8.contains("3으로"));
        assert!(checked_project(&read.authored_project_json_utf8).is_err());
        assert!(CodeSource::prepare(&read.authored_project_json_utf8,&read.parent_hex,&read.revision_hex).is_err());
        assert!(CodeSource::decode(&raw.replace("3으로","4으로")).is_err());
        assert!(super::super::model_source::ModelSource::decode(raw).is_err());
        assert!(super::super::source_only::PreparedSourceOnlyRevision::from_artifact(raw).is_err());
    }
    #[test] fn code_source_exact_codec_no_world_or_model_alias(){let raw=artifact(SOURCE);let x=CodeSource::prepare(&raw,"",&"01".repeat(16)).unwrap();let bytes=x.encode().unwrap();assert_eq!(CodeSource::decode(&bytes).unwrap().authored_project_json_utf8,raw);assert!(super::super::model_source::ModelSource::decode(&bytes).is_err());assert!(super::super::source_only::PreparedSourceOnlyRevision::from_artifact(&bytes).is_err());assert!(CodeSource::decode(&bytes.replace("12.0","13.0")).is_err());}
}
