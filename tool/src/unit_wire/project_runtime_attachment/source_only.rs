//! RD-631 source-only candidate and exact recorded readback. No VM, world,
//! transaction receipt, first-use authority, or fresh identities on restore.
use crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1 as Project;

pub(crate) struct PreparedSourceOnlyRevision {
    owned:Project, project:Vec<u8>, parent:Vec<u8>, revision:Vec<u8>,
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
        serde_json::to_string(&SourceArtifact{schema:"ddn.source_only_project_artifact.v1".into(),
            project_hex:hex::encode(&self.project),parent_hex:hex::encode(&self.parent),revision_hex:hex::encode(&self.revision),
            integrity_hex:hex::encode(self.integrity),profile:self.owned.project().source_profile.clone(),pesi_hex:hex::encode(self.pesi),
            project_payload_utf8:std::str::from_utf8(&self.payload).map_err(|_|"E_SOURCE_ARTIFACT_UTF8")?.into(),
            revision_envelope_base64url:super::encode_envelope(&self.envelope)}).map_err(|_|"E_SOURCE_ARTIFACT_ENCODING".into())
    }
    pub(crate) fn from_artifact(raw:&str)->Result<Self,String> {
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
        Self::restore(&project,&parent,&revision,&integrity,record.profile.as_bytes(),&pesi,record.project_payload_utf8.as_bytes(),&envelope)
    }
    pub(crate) fn prepare(owned:&Project,parent:&[u8],revision:&[u8])->Result<Self,String> {
        if revision.is_empty()||revision.len()>4096||parent.len()>4096||parent==revision {
            return Err("E_SOURCE_REVISION_IDENTITY: 저장 판·부모 정체가 올바르지 않습니다".into());
        }
        if owned.registration().is_none() {
            let mut needs_registration=!owned.sources().atom_value_occurrences().is_empty();
            for (path,capture) in owned.sources().captures() {
                if !owned.project().authored_identity.modules.iter().any(|module|module.sources.iter().any(|source|source==path)){continue;}
                let parsed=ddonirang_lang::parse_v25_with_mode(capture.text(),path,ddonirang_lang::ParseMode::Strict)
                    .map_err(|e|e.to_string())?;
                needs_registration|=parsed.items.iter().any(|ddonirang_lang::TopLevelItem::SeedDef(seed)|matches!(seed.seed_kind,ddonirang_lang::SeedKind::Schema(_)));
            }
            if needs_registration{return Err("E_VC04_REGISTRATION_REQUIRED: 명목·원자 등록 자료가 없는 원문은 저장 입장할 수 없습니다".into());}
        }
        let (issued,envelope)=super::issue_saved_source_envelope(owned,parent,revision)?;
        let payload=super::saved_runtime_project_payload_json(owned)?.into_bytes();
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
        Ok(Self {owned:owned.clone(),project,parent:parent.to_vec(),revision:revision.to_vec(),
            integrity,pesi:*issued.pesi().as_raw(),payload,envelope})
    }
    /// The caller supplies the selected revision/head context separately from
    /// the untrusted bytes. Re-encoding restores recorded source captures and
    /// registered IDs, never re-seals or invokes the registration CSPRNG.
    pub(crate) fn restore(project:&[u8],parent:&[u8],revision:&[u8],integrity:&[u8;32],
        profile:&[u8],pesi:&[u8;32],payload:&[u8],envelope:&[u8])->Result<Self,String> {
        if payload.is_empty()||payload.len()>super::OUTER_MAX||envelope.is_empty()||envelope.len()>super::ENVELOPE_MAX {
            return Err("E_SOURCE_REVISION_LIMIT".into());
        }
        let text=std::str::from_utf8(payload).map_err(|_|"E_SOURCE_REVISION_UTF8")?;
        let owned=super::restore_recorded_project_payload(text,envelope)?;
        let checked=Self::prepare(&owned,parent,revision)?;
        if checked.project!=project||checked.integrity!=*integrity||checked.pesi!=*pesi
            ||checked.profile()!=profile||checked.payload!=payload||checked.envelope!=envelope {
            return Err("E_SOURCE_REVISION_READBACK: 저장 원문·등록 자료·봉인·실행 정체가 다릅니다".into());
        }
        Ok(checked)
    }
    pub(crate) fn owned(&self)->&Project {&self.owned}
    /// Reconstruct only recorded ownership, including capture ordinals and
    /// registration IDs. No fresh admission, entropy, or first-use permission.
    pub(crate) fn recorded_execution_owner(&self)->Result<Project,String> {
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
