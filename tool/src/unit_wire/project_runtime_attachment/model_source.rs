//! Explicit, world-free model source custody. No Root/Start/PESI is fabricated.
use sha2::{Digest,Sha256};
pub(crate) const SCHEMA:&str="ddn.model_source_only_artifact.v1";
const MAX:usize=2_000_000;
#[derive(Clone,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ModelSource {
    schema:String,pub(crate) project_hex:String,pub(crate) parent_hex:String,
    pub(crate) revision_hex:String,pub(crate) profile:String,
    pub(crate) authored_project_json_utf8:String,integrity_hex:String,
}
pub(crate) fn checked_project(raw:&str)->Result<Option<crate::authored_project::ProjectArtifact>,String>{
    let p=crate::authored_project::open(raw.as_bytes())?;
    if p.source_profile!=crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY
        ||p.source_snapshot.root_path.is_some(){return Ok(None);}
    if !p.source_snapshot.files.iter().any(|f|ddonirang_lang::has_model_surface(&f.text)){return Ok(None);}
    // This first custody kind covers the existing standalone model frontdoor.
    // Multiple files are never guessed, concatenated or silently discarded.
    if p.source_snapshot.files.len()!=1||p.authored_identity.modules.len()!=1
        ||p.authored_identity.modules[0].sources!=vec![p.source_snapshot.entry_path.clone()]
        ||p.source_snapshot.files[0].path!=p.source_snapshot.entry_path {
        return Err("E_MODEL_SOURCE_LAYOUT_UNSUPPORTED: 모형 단독 원문 한 파일을 지정하세요".into());
    }
    let f=&p.source_snapshot.files[0];
    let parsed=ddonirang_lang::parse_model_program_with_source_file(&f.text,&f.path).map_err(|e|e.to_string())?;
    if let Some(d)=parsed.ir.diagnostics.iter().find(|d|matches!(d.severity,"error"|"error_candidate")){
        return Err(format!("{} [{}..{}]: {}",d.candidate_code,d.provenance.start,d.provenance.end,d.message));
    }
    Ok(Some(p))
}
impl ModelSource {
    pub(crate) fn prepare(raw:&str,parent:&str,revision:&str)->Result<Self,String>{
        if raw.len()>MAX{return Err("E_MODEL_SOURCE_LIMIT".into());}
        let p=checked_project(raw)?.ok_or("E_MODEL_SOURCE_REQUIRED")?;
        let id=|x:&str|x.len()==32&&x.bytes().all(|c|c.is_ascii_digit()||(b'a'..=b'f').contains(&c));
        if !id(revision)||(!parent.is_empty()&&!id(parent))||parent==revision{return Err("E_MODEL_SOURCE_REVISION".into());}
        let mut result=Self{schema:SCHEMA.into(),project_hex:p.authored_identity.logical_dependency_id.as_str().to_owned(),
            parent_hex:parent.into(),revision_hex:revision.into(),profile:p.source_profile.clone(),
            authored_project_json_utf8:raw.into(),integrity_hex:String::new()};
        result.integrity_hex=hex::encode(Sha256::digest(result.encode()?.as_bytes()));
        if result.encode()?.len()>MAX{return Err("E_MODEL_SOURCE_LIMIT".into());}Ok(result)
    }
    pub(crate) fn encode(&self)->Result<String,String>{serde_json::to_string(self).map_err(|_|"E_MODEL_SOURCE_ENCODING".into())}
    pub(crate) fn decode(raw:&str)->Result<Self,String>{
        if raw.len()>MAX||raw.starts_with('\u{feff}'){return Err("E_MODEL_SOURCE_LIMIT".into());}
        let r:Self=serde_json::from_str(raw).map_err(|_|"E_MODEL_SOURCE_FIELDS")?;
        if r.schema!=SCHEMA||r.encode()?!=raw{return Err("E_MODEL_SOURCE_CANONICAL".into());}
        let checked=Self::prepare(&r.authored_project_json_utf8,&r.parent_hex,&r.revision_hex)?;
        if checked.encode()?!=raw{return Err("E_MODEL_SOURCE_INTEGRITY".into());}Ok(checked)
    }
    pub(crate) fn anchor(&self)->Result<Anchor,String>{Ok(Anchor{
        revision:hex::decode(&self.revision_hex).map_err(|_|"E_MODEL_SOURCE_REVISION")?.try_into().map_err(|_|"E_MODEL_SOURCE_REVISION")?,
        integrity:hex::decode(&self.integrity_hex).map_err(|_|"E_MODEL_SOURCE_INTEGRITY")?.try_into().map_err(|_|"E_MODEL_SOURCE_INTEGRITY")?})}
    pub(crate) fn project(&self)->Result<[u8;32],String>{hex::decode(&self.project_hex).map_err(|_|"E_MODEL_SOURCE_PROJECT")?.try_into().map_err(|_|"E_MODEL_SOURCE_PROJECT".into())}
}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Anchor {pub(crate) revision:[u8;16],pub(crate) integrity:[u8;32]}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) const SOURCE:&str="용수철 = 모형 {\r\n위치:수@m :: (들임, 내줌).\r\n상수:수@kg/s^2 :: (들임, 내줌).\r\n질량:수@kg :: (들임, 내줌).\r\n힘:수@kg*m/s^2 :: (그자리, 내줌).\r\n가속도:수@m/s^2 :: (그자리, 내줌).\r\n힘::구함.\r\n가속도::구함.\r\n힘 =:= -상수 * 위치.\r\n가속도 =:= 힘 / 질량.\r\n}.\r\n계산 <- (대상=용수철, 들임=(위치: -0.5@m, 상수: 6@kg/s^2, 질량: 3@kg), 내줌=(위치, 힘, 가속도)) 풀기.\r\n\n(위치: 계산.위치, 힘: 계산.힘, 가속도: 계산.가속도) 내보임.\n";
    pub(crate) fn artifact(text:&str)->String{
        crate::authored_project::operate("create",&serde_json::json!({"source_profile":"v1-core-v25","module_name":"관계",
            "source_snapshot":{"root_path":null,"entry_path":"관계표원문.ddn","files":[{"path":"관계표원문.ddn","text":text}]}}).to_string()).unwrap()
    }
    pub(crate) fn edited(raw:&str,text:&str)->String{
        let p=crate::authored_project::open(raw.as_bytes()).unwrap();
        crate::authored_project::operate("update",&serde_json::json!({"original_json":raw,"source_snapshot":{"root_path":null,
            "entry_path":p.source_snapshot.entry_path,"files":[{"path":"관계표원문.ddn","text":text}]}}).to_string()).unwrap()
    }
    #[test]
    fn model_source_codec_preserves_exact_project_and_old_reader_refuses(){
        let raw=artifact(SOURCE);let source=ModelSource::prepare(&raw,"",&"01".repeat(16)).unwrap();let saved=source.encode().unwrap();
        assert_eq!(ModelSource::decode(&saved).unwrap().authored_project_json_utf8,raw);
        let opened=crate::authored_project::operate("source_open",&serde_json::json!({"artifact_text":saved,"project_hex":source.project_hex,
            "parent_hex":"","revision_hex":source.revision_hex}).to_string()).unwrap();assert_eq!(opened,raw);
        assert!(super::super::source_only::PreparedSourceOnlyRevision::from_artifact(&saved).is_err());
        let changed=saved.replace("3@kg","4@kg");assert_ne!(changed,saved);assert!(ModelSource::decode(&changed).is_err());
        let ordinary=artifact("값 := 1. 값 보여주기.");assert!(checked_project(&ordinary).unwrap().is_none());
    }
}
