//! RD-631 source-only candidate and exact recorded readback. No VM, world,
//! transaction receipt, first-use authority, or fresh identities on restore.
use crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1 as Project;

#[derive(Clone)]
pub(crate) struct PreparedSourceOnlyRevision {
    owned:Project, recorded:Option<Project>, project:Vec<u8>, parent:Vec<u8>, revision:Vec<u8>,
    integrity:[u8;32], pesi:[u8;32], payload:Vec<u8>, envelope:Vec<u8>,
}
// Product-owned source carrier for the approved v3 browser record. Neither
// parsing nor this schema name is source-save/first-use authority.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceArtifact {
    schema:String,project_hex:String,parent_hex:String,revision_hex:String,
    integrity_hex:String,profile:String,pesi_hex:String,
    project_payload_utf8:String,revision_envelope_base64url:String,
}
fn hex_bytes(text:&str,max:usize,empty:bool)->Result<Vec<u8>,String> {
    if (!empty&&text.is_empty())||text.len()>max*2||text.len()%2!=0
        ||!text.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {return Err("E_SOURCE_ARTIFACT_HEX".into());}
    hex::decode(text).map_err(|_|"E_SOURCE_ARTIFACT_HEX".into())
}
impl PreparedSourceOnlyRevision {
    pub(crate) fn artifact(&self)->Result<String,String> {
        let encoded=serde_json::to_string(&SourceArtifact{schema:"ddn.source_only_project_artifact.v1".into(),
            project_hex:hex::encode(&self.project),parent_hex:hex::encode(&self.parent),revision_hex:hex::encode(&self.revision),
            integrity_hex:hex::encode(self.integrity),profile:self.owned.project().source_profile.clone(),pesi_hex:hex::encode(self.pesi),
            project_payload_utf8:std::str::from_utf8(&self.payload).map_err(|_|"E_SOURCE_ARTIFACT_UTF8")?.into(),
            revision_envelope_base64url:super::encode_envelope(&self.envelope)}).map_err(|_|"E_SOURCE_ARTIFACT_ENCODING")?;
        if (self.owned.project().schema==crate::authored_project::SCHEMA_V3||self.owned.sources().public_callable_artifact().is_some())&&encoded.len()>8_000_000{return Err("E_SOURCE_ARTIFACT_LIMIT".into());}
        Ok(encoded)
    }
    pub(crate) fn from_artifact(raw:&str)->Result<Self,String> {Self::from_artifact_selected(raw,None,false)}
    pub(crate) fn from_artifact_with_first_callable(raw:&str,first:&crate::authored_project::FirstCallableAuthoredContext)->Result<Self,String>{
        Self::from_artifact_selected(raw,Some(first),false)
    }
    pub(crate) fn from_artifact_logic_private(raw:&str)->Result<Self,String>{Self::from_artifact_selected(raw,None,true)}
    fn from_artifact_selected(raw:&str,first:Option<&crate::authored_project::FirstCallableAuthoredContext>,logic_private:bool)->Result<Self,String>{
        if raw.len()>8_000_000||raw.starts_with('\u{feff}') {return Err("E_SOURCE_ARTIFACT_LIMIT".into());}
        let record:SourceArtifact=serde_json::from_str(raw).map_err(|_|"E_SOURCE_ARTIFACT_FIELDS")?;
        if record.schema!="ddn.source_only_project_artifact.v1"||serde_json::to_string(&record).map_err(|_|"E_SOURCE_ARTIFACT_ENCODING")?!=raw {
            return Err("E_SOURCE_ARTIFACT_CANONICAL".into());
        }
        let project=hex_bytes(&record.project_hex,32,false)?;let parent=hex_bytes(&record.parent_hex,4096,true)?;
        let revision=hex_bytes(&record.revision_hex,4096,false)?;
        let integrity:[u8;32]=hex_bytes(&record.integrity_hex,32,false)?.try_into().map_err(|_|"E_SOURCE_ARTIFACT_INTEGRITY")?;
        let pesi:[u8;32]=hex_bytes(&record.pesi_hex,32,false)?.try_into().map_err(|_|"E_SOURCE_ARTIFACT_PESI")?;
        let envelope=super::decode_envelope(&record.revision_envelope_base64url)?;
        Self::restore_selected(&project,&parent,&revision,&integrity,record.profile.as_bytes(),&pesi,record.project_payload_utf8.as_bytes(),&envelope,first,logic_private)
    }
    pub(crate) fn prepare(owned:&Project,parent:&[u8],revision:&[u8])->Result<Self,String> {Self::prepare_selected(owned,parent,revision,false)}
    pub(crate) fn prepare_logic_private(owned:&Project,parent:&[u8],revision:&[u8])->Result<Self,String>{Self::prepare_selected(owned,parent,revision,true)}
    fn prepare_selected(owned:&Project,parent:&[u8],revision:&[u8],logic_private:bool)->Result<Self,String>{
        if logic_private{
            if !parent.is_empty(){return Err("E_SOURCE_SAVE_PARENT: B1 첫 저장은 빈 부모만 허용합니다".into());}
            owned.registration().ok_or("E_LOGIC_STORAGE_CONTEXT_REQUIRED")?.verify_production_logic_dependency(&owned.precanon_view())?;
        }
        if revision.is_empty()||revision.len()>4096||parent.len()>4096||parent==revision {
            return Err("E_SOURCE_REVISION_IDENTITY: 저장 판·부모 정체가 올바르지 않습니다".into());
        }
        if owned.registration().is_none() {
            let mut needs_registration=!owned.sources().atom_value_occurrences().is_empty();
            for (path,capture) in owned.sources().captures() {
                if !owned.project().authored_identity.modules.iter().any(|module|module.sources.iter().any(|source|source==path)){continue;}
                let parsed=if let Some(public)=owned.sources().public_callable_context(){
                    ddonirang_lang::parser::Parser::parse_with_public_callable_context(capture.text(),path,public.compiler_context().clone()).map_err(|e|e.to_string())?
                }else{match owned.sources().first_callable_context(){
                    Some(first)=>first.parse_consumer_owned_program(capture.text(),path)?,
                    None=>ddonirang_lang::parse_v25_with_mode(capture.text(),path,ddonirang_lang::ParseMode::Strict).map_err(|e|e.to_string())?,
                }};
                needs_registration|=parsed.items.iter().any(|ddonirang_lang::TopLevelItem::SeedDef(seed)|matches!(seed.seed_kind,ddonirang_lang::SeedKind::Schema(_)));
            }
            if needs_registration{return Err("E_VC04_REGISTRATION_REQUIRED: 명목·원자 등록 자료가 없는 원문은 저장 입장할 수 없습니다".into());}
        }
        let (issued,envelope)=super::issue_saved_source_envelope(owned,parent,revision)?;
        let payload=if logic_private{super::saved_logic_payload_private(owned)?}else if owned.sources().public_callable_artifact().is_some(){
            super::saved_public_callable_payload_json(owned)?
        }else{match owned.sources().first_callable_context(){
            Some(first)=>super::saved_runtime_project_payload_json_with_first_callable(owned,first)?,
            None=>super::saved_runtime_project_payload_json(owned)?,
        }}.into_bytes();
        if payload.is_empty()||payload.len()>super::OUTER_MAX||envelope.len()>super::ENVELOPE_MAX {
            return Err("E_SOURCE_REVISION_LIMIT: 원문 저장 한도를 넘었습니다".into());
        }
        let value=crate::project_revision_envelope::verify_rd631_canonical_envelope_bytes(&envelope)
            .map_err(|e|format!("E_SOURCE_REVISION_ENVELOPE: {e:?}"))?;
        let integrity=crate::project_revision_envelope::verify_self_empty_digest(&value,"revision_integrity_sha256",
            crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1::CustodyConflict)
            .map_err(|e|format!("E_SOURCE_REVISION_INTEGRITY: {e:?}"))?;
        let project=hex::decode(owned.project().authored_identity.logical_dependency_id.as_str())
            .map_err(|_|"E_SOURCE_REVISION_PROJECT")?;
        if project.len()!=32{return Err("E_SOURCE_REVISION_PROJECT".into());}
        Ok(Self {owned:owned.clone(),recorded:None,project,parent:parent.to_vec(),revision:revision.to_vec(),
            integrity,pesi:*issued.pesi().as_raw(),payload,envelope})
    }
    /// The caller supplies the selected revision/head context separately from
    /// the untrusted bytes. Re-encoding restores recorded source captures and
    /// registered IDs, never re-seals or invokes the registration CSPRNG.
    pub(crate) fn restore(project:&[u8],parent:&[u8],revision:&[u8],integrity:&[u8;32],
        profile:&[u8],pesi:&[u8;32],payload:&[u8],envelope:&[u8])->Result<Self,String> {
        Self::restore_selected(project,parent,revision,integrity,profile,pesi,payload,envelope,None,false)
    }
    pub(crate) fn restore_logic_private(project:&[u8],parent:&[u8],revision:&[u8],integrity:&[u8;32],profile:&[u8],pesi:&[u8;32],payload:&[u8],envelope:&[u8])->Result<Self,String>{
        Self::restore_selected(project,parent,revision,integrity,profile,pesi,payload,envelope,None,true)
    }
    /// A successor is decoded only with the actual verified previous source
    /// row. The general artifact reader never supplies this authority.
    pub(crate) fn restore_public_edition(previous:&Self,revision:&[u8],integrity:&[u8;32],
        profile:&[u8],pesi:&[u8;32],payload:&[u8],envelope:&[u8],
    )->Result<Self,String>{
        if payload.is_empty()||payload.len()>super::OUTER_MAX||envelope.is_empty()||envelope.len()>super::ENVELOPE_MAX{
            return Err("E_SOURCE_REVISION_LIMIT".into());
        }
        let parent=previous.owned.sources().public_callable_artifact().ok_or("E_DDL_V4_RECORDED_CONTEXT_REQUIRED")?;
        let text=std::str::from_utf8(payload).map_err(|_|"E_SOURCE_REVISION_UTF8")?;
        let owned=super::restore_public_callable_payload_successor(text,envelope,parent)?;
        Self::check_recorded(owned,previous.project(),previous.revision(),revision,integrity,profile,pesi,payload,envelope,false)
    }

    fn restore_selected(project:&[u8],parent:&[u8],revision:&[u8],integrity:&[u8;32],
        profile:&[u8],pesi:&[u8;32],payload:&[u8],envelope:&[u8],
        first:Option<&crate::authored_project::FirstCallableAuthoredContext>,logic_private:bool)->Result<Self,String>{
        if payload.is_empty()||payload.len()>super::OUTER_MAX||envelope.is_empty()||envelope.len()>super::ENVELOPE_MAX {
            return Err("E_SOURCE_REVISION_LIMIT".into());
        }
        let text=std::str::from_utf8(payload).map_err(|_|"E_SOURCE_REVISION_UTF8")?;
        let owned=if logic_private{super::restore_logic_payload_private(text)?}else{match first {
            Some(first)=>super::restore_recorded_first_callable_project_payload(text,envelope,first)?,
            None=>super::restore_recorded_project_payload(text,envelope)?,
        }};
        Self::check_recorded(owned,project,parent,revision,integrity,profile,pesi,payload,envelope,logic_private)
    }
    fn check_recorded(mut owned:Project,project:&[u8],parent:&[u8],revision:&[u8],integrity:&[u8;32],
        profile:&[u8],pesi:&[u8;32],payload:&[u8],envelope:&[u8],logic_private:bool,
    )->Result<Self,String>{
        let mut checked=Self::prepare_selected(&owned,parent,revision,logic_private)?;
        if checked.project!=project||checked.profile()!=profile||checked.payload!=payload {
            return Err("E_SOURCE_REVISION_READBACK: 저장 원문·등록 자료·봉인·실행 정체가 다릅니다".into());
        }
        let matches_seal=|candidate:&Self|candidate.integrity==*integrity
            &&candidate.pesi==*pesi&&candidate.envelope==envelope;
        if !matches_seal(&checked) {
            let mismatch=||"E_SOURCE_REVISION_READBACK: 저장 원문·등록 자료·봉인·실행 정체가 다릅니다".to_string();
            // Only the old initial DDL source revision is eligible. Neither
            // the recorded definition nor its seal supplies identity authority.
            if !parent.is_empty()||logic_private{return Err(mismatch());}
            let legacy=owned.with_legacy_ddl_consumer_start()?.ok_or_else(mismatch)?;
            let candidate=Self::prepare_selected(&legacy,parent,revision,false)?;
            if !matches_seal(&candidate)||candidate.project!=project||candidate.profile()!=profile
                ||candidate.payload!=payload{return Err(mismatch());}
            checked=candidate;
            owned=legacy;
        }
        // Only the complete restore/readback above can retain this typed owner.
        // It contains recorded captures and IDs, never fresh registration.
        checked.recorded=Some(owned);
        Ok(checked)
    }
    pub(crate) fn owned(&self)->&Project {&self.owned}
    /// Reconstruct only recorded ownership, including capture ordinals and
    /// registration IDs. No fresh admission, entropy, or first-use permission.
    pub(crate) fn recorded_execution_owner(&self)->Result<Project,String> {
        if let Some(recorded)=&self.recorded {return Ok(recorded.clone());}
        if self.owned.sources().logic_callable_context().is_some(){return Ok(Self::restore_logic_private(&self.project,&self.parent,&self.revision,&self.integrity,
            self.profile(),&self.pesi,&self.payload,&self.envelope)?.owned);}
        Ok(Self::restore(&self.project,&self.parent,&self.revision,&self.integrity,
            self.profile(),&self.pesi,&self.payload,&self.envelope)?.owned)
    }
    pub(crate) fn project(&self)->&[u8] {&self.project}
    pub(crate) fn parent(&self)->&[u8] {&self.parent}
    pub(crate) fn revision(&self)->&[u8] {&self.revision}
    pub(crate) fn integrity(&self)->&[u8;32] {&self.integrity}
    pub(crate) fn pesi(&self)->&[u8;32] {&self.pesi}
    pub(crate) fn profile(&self)->&[u8] {self.owned.project().source_profile.as_bytes()}
    pub(crate) fn payload(&self)->&[u8] {&self.payload}
    pub(crate) fn envelope(&self)->&[u8] {&self.envelope}
}

#[cfg(test)]
mod recorded_owner_tests {
    use super::*;
    #[test]
    fn restored_source_reuses_only_the_actual_recorded_owner() {
        let authored=crate::authored_project::operate("create",&serde_json::json!({
            "source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음."},
                {"path":"main.ddn","text":"실행:움직씨 := { 0 보여주기. }."}]}}).to_string()).unwrap();
        let fresh=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes()).unwrap();
        let prepared=PreparedSourceOnlyRevision::prepare(&fresh,b"",b"source-test").unwrap();
        assert!(prepared.recorded.is_none());
        let raw=prepared.artifact().unwrap();
        let restored=PreparedSourceOnlyRevision::from_artifact(&raw).unwrap();
        let a=restored.recorded_execution_owner().unwrap();
        let b=restored.recorded_execution_owner().unwrap();
        assert!(std::sync::Arc::ptr_eq(a.sources(),b.sources()));
        assert_eq!(super::super::saved_runtime_project_payload_json(&a).unwrap(),
            super::super::saved_runtime_project_payload_json(&b).unwrap());
        assert_eq!(a.definition_identity().as_bytes(),b.definition_identity().as_bytes());
        assert_eq!(restored.artifact().unwrap(),raw);
        let mut wrong:serde_json::Value=serde_json::from_str(&raw).unwrap();
        wrong["pesi_hex"]=serde_json::Value::String("00".repeat(32));
        assert!(PreparedSourceOnlyRevision::from_artifact(&serde_json::to_string(&wrong).unwrap()).is_err());
    }
}
