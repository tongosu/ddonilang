//! RD-686 physical byte gate. Passing this gate is NOT revision/PESI admission.
//! The caller must subsequently verify the raw envelope with its shared owner,
//! restore a candidate VM, and only then publish. No adapter may mint identity.
// F5-GA[IMMUTABLE-JSON]: envelope[key]는 serde_json::Value의 불변 조회다. 키/형식 불일치는
// Null을 돌려주며, 뒤의 명시적 형식/정체 검사에서 거부된다. IndexMut 증명은 아니다.

use serde::{de::{MapAccess, SeqAccess, Visitor}, Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::{SerializeMap, SerializeSeq};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt};

pub(crate) mod source_only;

pub(crate) const OUTER_MAX: usize = 2_000_000;
const ENVELOPE_MAX: usize = 1_000_000;
const ENCODED_ENVELOPE_MAX: usize = 1_333_334;
const TOP: [&str; 6] = ["schema", "project_snapshot", "runtime_attachment", "canonical_json", "content_sha256", "canonical_byte_length"];
const ATTACHMENT: [&str; 12] = ["schema", "source_profile_identity", "project_execution_snapshot_identity_hex", "revision_envelope_base64url", "revision_envelope_byte_length", "revision_envelope_sha256", "runtime_state_json_utf8", "runtime_state_byte_length", "runtime_state_sha256", "resource_snapshot_v2_json_utf8", "resource_snapshot_v2_byte_length", "resource_snapshot_v2_sha256"];

// Pair vectors preserve authored order and detect duplicates before collapse.
#[derive(Clone, Debug)]
enum Node { Null, Bool(bool), UInt(u64), Text(String), Array(Vec<Node>), Object(Vec<(String, Node)>) }
impl Serialize for Node {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::UInt(v) => serializer.serialize_u64(*v),
            Self::Text(v) => serializer.serialize_str(v),
            Self::Array(values) => { let mut seq = serializer.serialize_seq(Some(values.len()))?; for v in values { seq.serialize_element(v)?; } seq.end() },
            Self::Object(values) => { let mut map = serializer.serialize_map(Some(values.len()))?; for (k,v) in values { map.serialize_entry(k,v)?; } map.end() },
        }
    }
}
impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Ordered;
        impl<'de> Visitor<'de> for Ordered {
            type Value = Node;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("정본 프로젝트 저장 JSON") }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Node,E> { Ok(Node::Null) }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Node,E> { Ok(Node::Bool(v)) }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Node,E> { Ok(Node::UInt(v)) }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Node,E> { Ok(Node::Text(v.into())) }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Node,E> { Ok(Node::Text(v)) }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Node,A::Error> {
                let mut values = Vec::new(); while let Some(v) = seq.next_element()? { values.push(v); } Ok(Node::Array(values))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Node,A::Error> {
                let mut seen = BTreeSet::new(); let mut values = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !seen.insert(key.clone()) { return Err(serde::de::Error::custom("중복 프로젝트 저장 키")); }
                    values.push((key, map.next_value()?));
                }
                Ok(Node::Object(values))
            }
        }
        deserializer.deserialize_any(Ordered)
    }
}

/// Transport bytes only: intentionally no `Admitted` or publication capability.
#[derive(Debug)]
pub(crate) struct ByteCheckedAttachment {
    pub source_profile: String,
    pub pesi_hex: String,
    pub revision_envelope: Vec<u8>,
    pub runtime_state: String,
    pub resource_snapshot: String,
    pub project_snapshot_json: String,
}

/// The standalone resource closure and the runtime's embedded resource bytes
/// must describe the same snapshot. Independent SHA checks alone do not bind
/// them: two individually valid snapshots must not be combined into one run.
/// This remains a structural gate, not revision/PESI/VM admission.
pub(crate) fn check_runtime_resource_binding(runtime: &str, resources: &str) -> Result<(),String> {
    check_runtime_resource_binding_with_catalog(runtime,resources,None)
}
pub(crate) fn check_runtime_resource_binding_with_catalog(runtime:&str,resources:&str,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<(),String>{
    let runtime = super::parse_snapshot_json_with_catalog(runtime,catalog)?;
    let resources = super::parse_snapshot_json_with_catalog(resources,catalog)?;
    check_runtime_resource_projection(&runtime,&resources)?;
    super::resource_snapshot::check_resource_snapshot_structure_with_catalog(&resources,catalog)?;
    Ok(())
}
fn check_runtime_resource_projection(runtime:&serde_json::Value,resources:&serde_json::Value)->Result<(),String>{
    let snapshot = runtime.pointer("/state/snapshot_v2")
        .and_then(serde_json::Value::as_object)
        .ok_or("E_PROJECT_RUNTIME_STATE_SNAPSHOT: runtime snapshot_v2가 없습니다")?;
    if snapshot.get("schema").and_then(serde_json::Value::as_str) != Some("ddn.wasm_state_snapshot.v2")
        || snapshot.get("resource_snapshot_schema").and_then(serde_json::Value::as_str) != Some(super::RESOURCE_SNAPSHOT_SCHEMA_V2)
        || resources.get("schema").and_then(serde_json::Value::as_str) != Some(super::RESOURCE_SNAPSHOT_SCHEMA_V2) {
        return Err("E_PROJECT_RUNTIME_RESOURCE_SCHEMA: 자원 스냅샷 판이 다릅니다".into());
    }
    if !snapshot.get("resources").is_some_and(|value| super::SnapshotJsonOwner::clone_from_value(resources).equals_json(value)) {
        return Err("E_PROJECT_RUNTIME_RESOURCE_MISMATCH: 실행 상태와 자원 스냅샷이 다릅니다".into());
    }
    // Native and WASM both reject incomplete/malformed resource channels
    // before publishing a saved artifact or a source-bound restore candidate.
    Ok(())
}

fn check_runtime_resource_binding_with_frame(runtime:&str,resources:&str,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,frame:&super::CompleteStateFrameV1<'_>,
)->Result<(),String>{
    let invalid=|error|format!("E_VC04_COMPLETE_RESOURCE_BINDING: {error:?}");
    let contracts=super::complete_state_tables::decode_registered_atom_evidence(frame.atom_evidence).map_err(invalid)?;
    let custody=super::complete_state_tables::decode_saved_world_custody(frame.subject_custody,frame.pesi).map_err(invalid)?;
    let references=custody.as_ref().map(crate::subject_reference_owner::SubjectReferenceContextV1::new).transpose().map_err(invalid)?;
    let runtime=super::parse_snapshot_json_with_value_context(runtime,&contracts,references.as_ref(),catalog)?;
    let resources=super::parse_snapshot_json_with_value_context(resources,&contracts,references.as_ref(),catalog)?;
    check_runtime_resource_projection(&runtime,&resources)?;
    super::resource_snapshot::check_resource_snapshot_complete_pair(&resources,frame,catalog)?;
    if runtime.pointer("/state/snapshot_v2/next_tick_id").and_then(serde_json::Value::as_u64)!=Some(frame.next_tick){
        return Err("E_VC04_COMPLETE_RESOURCE_BINDING: binary·runtime 마디가 다릅니다".into());
    }
    Ok(())
}

pub(crate) fn check_saved_revision_context_with_frame<'a>(attachment:&'a ByteCheckedAttachment,
    owned:&'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,parent:&[u8],revision:&[u8],
    frame:super::CompleteStateFrameV1<'a>,
)->Result<SourceBoundSavedRuntimeAttachment<'a>,String>{
    let source=check_saved_source_context(attachment,owned,parent,revision)?;
    if frame.pesi!=source.pesi.as_raw(){return Err("E_VC04_COMPLETE_RESOURCE_BINDING: 원문·binary PESI가 다릅니다".into());}
    let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose()?;
    check_runtime_resource_binding_with_frame(&attachment.runtime_state,&attachment.resource_snapshot,catalog.as_ref(),&frame)?;
    Ok(SourceBoundSavedRuntimeAttachment{owned,attachment,pesi:source.pesi,world_frame:Some(frame)})
}

/// The caller supplies the expected identity from its shared owner, not from
/// the untrusted wrapper. This is still NOT candidate-VM/publication admission.
pub(crate) fn check_revision_context(
    attachment: &ByteCheckedAttachment,
    project: &[u8], expected_parent: &[u8], new_revision: &[u8],
    source_profile: &str, admitted_pesi: crate::project_snapshot_identity::Identity32,
    saved_source: &crate::project_revision_envelope_supply::VerifiedSavedSourceV1,
) -> Result<serde_json::Value,String> {
    use crate::project_revision_envelope::{verify_rd631_canonical_envelope_bytes, verify_rd631_envelope_structure, verify_self_empty_digest};
    use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
    if attachment.source_profile != source_profile || attachment.pesi_hex != admitted_pesi.to_hex() {
        return Err("E_PROJECT_RUNTIME_ATTACHMENT_CONTEXT: 실행 판·정체가 다릅니다".into());
    }
    let reject = |error: Failure| format!("E_PROJECT_RUNTIME_REVISION: {error:?}");
    let envelope = verify_rd631_canonical_envelope_bytes(&attachment.revision_envelope).map_err(reject)?;
    verify_rd631_envelope_structure(&envelope).map_err(reject)?;
    verify_self_empty_digest(&envelope,"revision_integrity_sha256",Failure::CustodyConflict).map_err(reject)?;
    verify_self_empty_digest(&envelope["source_manifest"],"canonical_manifest_sha256",Failure::CustodyConflict).map_err(reject)?;
    crate::project_revision_envelope_supply::verify_revision_identity_crosschecks(&envelope,project,expected_parent,new_revision).map_err(reject)?;
    // Bind the envelope manifest to exactly supplied raw source bytes. Never
    // accept a self-consistent forged manifest or reread ambient paths.
    let expected = saved_source.manifest();
    let manifest = &envelope["source_manifest"];
    let expected_files = serde_json::to_value(&expected.files).map_err(|e| e.to_string())?;
    if manifest["schema"] != expected.schema || manifest["kind"] != expected.kind
        || manifest["root_path"] != expected.root_path || manifest["entry_path"] != expected.entry_path
        || manifest["file_count"].as_u64() != Some(expected.file_count as u64)
        || manifest["total_utf8_bytes"].as_u64() != Some(expected.total_utf8_bytes as u64)
        || manifest["files"] != expected_files || manifest["project_graph_sha256"] != expected.project_graph_sha256
        || manifest["lock_sha256"] != serde_json::to_value(&expected.lock_sha256).map_err(|e| e.to_string())? {
        return Err("E_PROJECT_RUNTIME_REVISION_SOURCE: 기록의 원문 목록이 공급된 원문과 다릅니다".into());
    }
    use crate::project_revision_envelope_supply::{verify_rd631_identity_projection,IdentityFamily};
    let definition = verify_rd631_identity_projection(IdentityFamily::Definition,
        envelope["official_entry_definition_identity"].as_str().ok_or("E_PROJECT_RUNTIME_REVISION_DEFINITION")?).map_err(reject)?;
    if definition.as_bytes() != saved_source.official_entry_definition()
        .map_err(|_|"E_PROJECT_RUNTIME_REVISION_DEFINITION: 시작 정의 정체가 해소되지 않았습니다")?.as_bytes() {
        return Err("E_PROJECT_RUNTIME_REVISION_DEFINITION: 시작 정의 정체가 다릅니다".into());
    }
    Ok(envelope)
}

/// Typed source/configuration binding, before a VM can be restored. Revision
/// lineage still comes from the selected storage owner, not from this record.
#[derive(Debug)]
pub(crate) struct SourceBoundSavedRuntimeAttachment<'a> {
    owned: &'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    attachment: &'a ByteCheckedAttachment,
    pesi: crate::project_snapshot_identity::Identity32,
    world_frame:Option<super::CompleteStateFrameV1<'a>>,
}
impl<'a> SourceBoundSavedRuntimeAttachment<'a> {
    pub(crate) fn owned(&self) -> &'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1 { self.owned }
    pub(crate) fn runtime_state(&self) -> &str { &self.attachment.runtime_state }
    pub(crate) fn resource_snapshot(&self) -> &str { &self.attachment.resource_snapshot }
    pub(crate) fn revision_envelope(&self) -> &[u8] { &self.attachment.revision_envelope }
    pub(crate) fn pesi(&self) -> crate::project_snapshot_identity::Identity32 { self.pesi }
    pub(crate) fn world_frame(&self)->Option<super::CompleteStateFrameV1<'a>>{self.world_frame}
    pub(crate) fn parse_runtime(&self)->Result<super::SnapshotJsonOwner,String>{
        let catalog=self.owned.registration().map(|registration|registration.saved_nominal_catalog(self.owned)).transpose()?;
        let Some(frame)=self.world_frame else{return super::parse_snapshot_json_with_catalog(self.runtime_state(),catalog.as_ref());};
        let invalid=|error|format!("E_VC04_COMPLETE_RESOURCE_BINDING: {error:?}");
        let contracts=super::complete_state_tables::decode_registered_atom_evidence(frame.atom_evidence).map_err(invalid)?;
        let custody=super::complete_state_tables::decode_saved_world_custody(frame.subject_custody,frame.pesi).map_err(invalid)?;
        let references=custody.as_ref().map(crate::subject_reference_owner::SubjectReferenceContextV1::new).transpose().map_err(invalid)?;
        super::parse_snapshot_json_with_value_context(self.runtime_state(),&contracts,references.as_ref(),catalog.as_ref())
    }
    pub(crate) fn source_context(&self)->SourceEnvelopeBoundRuntimeAttachment<'a>{
        SourceEnvelopeBoundRuntimeAttachment{owned:self.owned,attachment:self.attachment,pesi:self.pesi}
    }
}

/// Source/capture/PESI binding only. It cannot restore resources or a VM and
/// carries no Native lease. Cold Native readmission precedes full value binding.
#[derive(Debug)]
pub(crate) struct SourceEnvelopeBoundRuntimeAttachment<'a>{
    owned:&'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    attachment:&'a ByteCheckedAttachment,pesi:crate::project_snapshot_identity::Identity32,
}
impl<'a> SourceEnvelopeBoundRuntimeAttachment<'a>{
    pub(crate) fn owned(&self)->&'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1{self.owned}
    pub(crate) fn revision_envelope(&self)->&[u8]{&self.attachment.revision_envelope}
    pub(crate) fn pesi(&self)->crate::project_snapshot_identity::Identity32{self.pesi}
}

// A3-P11: product-owned payload inside RD-686's project_snapshot slot. This
// does not add keys to the closed revision envelope or to its attachment.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedRuntimeProjectPayloadV1 {
    schema: String,
    authored_project_json_utf8: String,
    source_capture_catalog_json_utf8: String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    registration_resources:Option<Vec<crate::authored_project::registration::RecordedRegistrationBlob>>,
}

pub(crate) fn saved_runtime_project_payload_json(
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
) -> Result<String,String> {
    let payload=SavedRuntimeProjectPayloadV1 {
        schema:if owned.registration().is_some(){"ddn.saved_runtime_project.v2"}else{"ddn.saved_runtime_project.v1"}.into(),
        authored_project_json_utf8:owned.project().save()?,
        source_capture_catalog_json_utf8:owned.sources().recorded_capture_catalog_json()?,
        registration_resources:owned.registration().map(|draft|draft.recorded_blobs()),
    };
    serde_json::to_string(&payload).map_err(|_|"E_PROJECT_RUNTIME_PAYLOAD_ENCODING: 저장 작품을 만들 수 없습니다".into())
}

fn read_saved_runtime_payload(raw: &str) -> Result<SavedRuntimeProjectPayloadV1,String> {
    if raw.len()>OUTER_MAX { return Err("E_PROJECT_RUNTIME_PAYLOAD_LIMIT: 저장 작품 한도를 넘었습니다".into()); }
    let value=super::parse_snapshot_json(raw)?;
    let payload=<SavedRuntimeProjectPayloadV1 as serde::Deserialize>::deserialize(&*value)
        .map_err(|_|"E_PROJECT_RUNTIME_PAYLOAD_FIELDS: 저장 작품 필드가 다릅니다")?;
    if !matches!((payload.schema.as_str(),payload.registration_resources.is_some()),("ddn.saved_runtime_project.v1",false)|("ddn.saved_runtime_project.v2",true))
        || serde_json::to_string(&payload).map_err(|_|"E_PROJECT_RUNTIME_PAYLOAD_ENCODING")?!=raw {
        return Err("E_PROJECT_RUNTIME_PAYLOAD_CANONICAL: 저장 작품이 정본이 아닙니다".into());
    }
    Ok(payload)
}

/// Shared product writer, paired with the strict reader. Values describing
/// semantic ownership come from the admitted owner, never adapter-made hashes.
fn issue_saved_source_envelope(
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    parent: &[u8], revision: &[u8],
) -> Result<(crate::project_snapshot_identity::current::IssuedSavedLocalProjectIdentity,Vec<u8>),String> {
    use crate::project_revision_envelope_supply::{CanonicalIdentityBytesV1,
        issue_rd631_identity_projection,IdentityFamily};
    let issued=crate::project_snapshot_identity::current::issue_saved_local_project(owned)?;
    let projection=|family,raw:&[u8]| -> Result<String,String> {
        let value=CanonicalIdentityBytesV1::from_owner_bytes(raw)
            .map_err(|_|"E_PROJECT_RUNTIME_IDENTITY_LIMIT: 저장 정체 바이트가 올바르지 않습니다")?;
        issue_rd631_identity_projection(family,&value)
            .map_err(|_|"E_PROJECT_RUNTIME_IDENTITY_ENCODING: 저장 정체를 표시할 수 없습니다".into())
    };
    let project_raw=hex::decode(owned.project().authored_identity.logical_dependency_id.as_str())
        .map_err(|_|"E_PROJECT_RUNTIME_PROJECT_IDENTITY")?;
    let stamp=|value:&mut serde_json::Value,key:&str| -> Result<(),String> {
        value[key]="".into();
        let mut encoded=serde_json::to_vec(value).map_err(|_|"E_PROJECT_RUNTIME_REVISION_ENCODING")?;
        encoded.push(b'\n'); value[key]=digest(&encoded).into(); Ok(())
    };
    let mut manifest=serde_json::to_value(owned.sources().saved().manifest())
        .map_err(|_|"E_PROJECT_RUNTIME_MANIFEST_ENCODING")?;
    manifest.as_object_mut().ok_or("E_PROJECT_RUNTIME_MANIFEST_ENCODING")?.remove("limits");
    stamp(&mut manifest,"canonical_manifest_sha256")?;
    let mut envelope=serde_json::json!({
        "schema":"ddn.workbench.project_snapshot_revision.v1",
        "project_identity":projection(IdentityFamily::Project,&project_raw)?,
        "history_revision_identity":projection(IdentityFamily::HistoryRevision,revision)?,
        "expected_parent_revision_identity":if parent.is_empty(){None}else{Some(projection(IdentityFamily::HistoryRevision,parent)?)},
        "source_epoch_identity":projection(IdentityFamily::SourceEpoch,owned.sources().entry_capture().epoch_owner_bytes())?,
        "project_semantic_configuration_identity":projection(IdentityFamily::ProjectSemanticConfiguration,issued.constituents()[1].as_raw())?,
        "official_entry_path":owned.sources().saved().manifest().entry_path,
        "official_entry_definition_identity":projection(IdentityFamily::Definition,owned.definition_identity().as_bytes())?,
        "source_manifest":manifest,"dependency_resolution_identity":null,
        "declared_resource_members":registration_members(owned),"revision_integrity_sha256":"",
    });
    stamp(&mut envelope,"revision_integrity_sha256")?;
    let mut revision_bytes=serde_json::to_vec(&envelope).map_err(|_|"E_PROJECT_RUNTIME_REVISION_ENCODING")?;
    revision_bytes.push(b'\n');
    Ok((issued,revision_bytes))
}

pub(crate) fn encode_saved_runtime_artifact(
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    runtime: &str, resources: &str, parent: &[u8], revision: &[u8],
) -> Result<String,String> {
    encode_saved_runtime_artifact_with_frame(owned,runtime,resources,parent,revision,None)
}
pub(crate) fn encode_saved_runtime_artifact_with_frame(owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    runtime:&str,resources:&str,parent:&[u8],revision:&[u8],world_frame:Option<super::CompleteStateFrameV1<'_>>,
)->Result<String,String>{
    if runtime.len()>OUTER_MAX || resources.len()>OUTER_MAX {
        return Err("E_PROJECT_RUNTIME_ATTACHMENT_TOO_LARGE".into());
    }
    let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose()?;
    if let Some(frame)=world_frame{
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(owned)?;
        if frame.pesi!=issued.pesi().as_raw(){return Err("E_VC04_COMPLETE_RESOURCE_BINDING: 원문·binary PESI가 다릅니다".into());}
        check_runtime_resource_binding_with_frame(runtime,resources,catalog.as_ref(),&frame)?;
    }else{check_runtime_resource_binding_with_catalog(runtime,resources,catalog.as_ref())?;}
    let (issued,revision_bytes)=issue_saved_source_envelope(owned,parent,revision)?;
    let project:Node=serde_json::from_str(&saved_runtime_project_payload_json(owned)?)
        .map_err(|_|"E_PROJECT_RUNTIME_PAYLOAD_ENCODING")?;
    let attachment=Node::Object(ATTACHMENT.iter().zip([
        Node::Text("ddn.workbench.project_runtime_attachment.v1".into()),
        Node::Text(owned.project().source_profile.clone()),Node::Text(issued.pesi().to_hex()),
        Node::Text(encode_envelope(&revision_bytes)),Node::UInt(revision_bytes.len() as u64),Node::Text(digest(&revision_bytes)),
        Node::Text(runtime.into()),Node::UInt(runtime.len() as u64),Node::Text(digest(runtime.as_bytes())),
        Node::Text(resources.into()),Node::UInt(resources.len() as u64),Node::Text(digest(resources.as_bytes())),
    ]).map(|(key,value)|((*key).into(),value)).collect());
    let canonical=bytes(&Node::Object(vec![("project_snapshot".into(),project.clone()),("runtime_attachment".into(),attachment.clone())]))?;
    let result=bytes(&Node::Object(TOP.iter().zip([
        Node::Text("seamgrim.project_snapshot_artifact.v1".into()),project,attachment,
        Node::Text(String::from_utf8(canonical.clone()).map_err(|_|"E_PROJECT_RUNTIME_ARTIFACT_UTF8")?),
        Node::Text(digest(&canonical)),Node::UInt(canonical.len() as u64),
    ]).map(|(key,value)|((*key).into(),value)).collect()))?;
    let checked=check_bytes(&result)?;
    if let Some(frame)=world_frame{check_saved_revision_context_with_frame(&checked,owned,parent,revision,frame)?;}
    else{check_saved_revision_context(&checked,owned,parent,revision)?;}
    String::from_utf8(result).map_err(|_|"E_PROJECT_RUNTIME_ARTIFACT_UTF8".into())
}

/// Reconstruct source ownership from the bytes in the verified transport. The
/// returned owner is still not publication authority: lineage and actual PESI
/// must pass check_saved_revision_context before constructing a candidate VM.
pub(crate) fn restore_saved_runtime_payload(
    attachment: &ByteCheckedAttachment,
) -> Result<crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,String> {
    restore_recorded_project_payload(&attachment.project_snapshot_json,&attachment.revision_envelope)
}

fn restore_recorded_project_payload(
    project_snapshot_json:&str, revision_envelope:&[u8],
) -> Result<crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,String> {
    use crate::project_revision_envelope_supply::{collect_saved_source_supply,
        restore_saved_v25_sources,admit_saved_authored_project,SavedSourceBlobV1,
        admit_precanon_authored_source,finalize_registered_authored_source,
        verify_rd631_identity_projection,IdentityFamily};
    let payload=read_saved_runtime_payload(project_snapshot_json)?;
    let project=crate::authored_project::open(payload.authored_project_json_utf8.as_bytes())?;
    let snapshot=&project.source_snapshot;
    let root=snapshot.root_path.as_deref()
        .ok_or("E_PROJECT_RUNTIME_ROOT_REQUIRED: 실행 복원에는 명시 얽음 뿌리가 필요합니다")?;
    let envelope_text=std::str::from_utf8(revision_envelope)
        .map_err(|_|"E_PROJECT_RUNTIME_REVISION_UTF8")?;
    let envelope=super::parse_snapshot_json(envelope_text)?;
    let declaration=envelope["official_entry_definition_identity"].as_str()
        .ok_or("E_PROJECT_RUNTIME_REVISION_DEFINITION: 시작 정의 정체가 없습니다")?;
    // This untrusted carrier is only structural supply. The actual declaration
    // issuer below independently resolves it; revision admission compares both.
    let carrier=verify_rd631_identity_projection(IdentityFamily::Definition,declaration)
        .map_err(|_|"E_PROJECT_RUNTIME_REVISION_DEFINITION: 시작 정의 표기가 다릅니다")?;
    let blobs:Vec<_>=snapshot.files.iter().map(|file|SavedSourceBlobV1 {
        path:&file.path,role:if file.path==root {"root"} else {"ddn_source"},bytes:file.text.as_bytes(),
    }).collect();
    let saved=collect_saved_source_supply(root,&snapshot.entry_path,&carrier,&blobs)
        .map_err(|_|"E_PROJECT_RUNTIME_PAYLOAD_SOURCE: 저장 원문 공급이 다릅니다")?;
    let sources=std::sync::Arc::new(restore_saved_v25_sources(saved,&payload.source_capture_catalog_json_utf8)?);
    if let Some(resources)=payload.registration_resources {
        let source=admit_precanon_authored_source(sources,payload.authored_project_json_utf8.as_bytes())?;
        let draft=crate::authored_project::registration::restore_precanon_recorded_candidate(&source,resources)?;
        finalize_registered_authored_source(source,draft)
    } else {admit_saved_authored_project(sources,payload.authored_project_json_utf8.as_bytes())}
}

fn registration_members(owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1)->serde_json::Value {
    serde_json::Value::Array(owned.registration().into_iter().flat_map(|draft|draft.artifacts()).map(|blob|
        serde_json::json!({"logical_path":blob.path(),"content_identity":blob.sha()})).collect())
}

pub(crate) fn check_saved_revision_context<'a>(
    attachment: &'a ByteCheckedAttachment,
    owned: &'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    expected_parent: &[u8], new_revision: &[u8],
) -> Result<SourceBoundSavedRuntimeAttachment<'a>,String> {
    let source=check_saved_source_context(attachment,owned,expected_parent,new_revision)?;
    let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose()?;
    check_runtime_resource_binding_with_catalog(&attachment.runtime_state,&attachment.resource_snapshot,catalog.as_ref())?;
    Ok(SourceBoundSavedRuntimeAttachment {owned,attachment,pesi:source.pesi,world_frame:None})
}
pub(crate) fn check_saved_source_context<'a>(attachment:&'a ByteCheckedAttachment,
    owned:&'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    expected_parent:&[u8],new_revision:&[u8],
)->Result<SourceEnvelopeBoundRuntimeAttachment<'a>,String>{
    use crate::project_revision_envelope_supply::{verify_rd631_identity_projection,IdentityFamily};
    let payload=read_saved_runtime_payload(&attachment.project_snapshot_json)?;
    if payload.authored_project_json_utf8!=owned.project().save()?
        || payload.source_capture_catalog_json_utf8!=owned.sources().recorded_capture_catalog_json()? {
        return Err("E_PROJECT_RUNTIME_PAYLOAD_OWNER: 저장 작품과 검증된 원문 소유자가 다릅니다".into());
    }
    let recorded=payload.registration_resources.as_ref().map(serde_json::to_vec).transpose().map_err(|_|"E_PROJECT_REGISTRATION_ENCODING")?;
    let expected=owned.registration().map(|draft|serde_json::to_vec(&draft.recorded_blobs())).transpose().map_err(|_|"E_PROJECT_REGISTRATION_ENCODING")?;
    if recorded!=expected{return Err("E_PROJECT_REGISTRATION_OWNER: 등록 자료가 소유자와 다릅니다".into());}
    let issued=crate::project_snapshot_identity::current::issue_saved_local_project(owned)?;
    let project=hex::decode(owned.project().authored_identity.logical_dependency_id.as_str())
        .map_err(|_|"E_PROJECT_RUNTIME_PROJECT_IDENTITY: 작품 정체 바이트를 해소할 수 없습니다")?;
    let supply=owned.resolved_revision_supply();
    let envelope=check_revision_context(attachment,&project,expected_parent,new_revision,
        &owned.project().source_profile,issued.pesi(),&supply)?;
    let check_projection=|key:&str,family,expected:&[u8]| -> Result<(),String> {
        let text=envelope[key].as_str().ok_or_else(||format!("E_PROJECT_RUNTIME_REVISION_FIELD: {key}"))?;
        let actual=verify_rd631_identity_projection(family,text)
            .map_err(|_|format!("E_PROJECT_RUNTIME_REVISION_FIELD: {key}"))?;
        if actual.as_bytes()!=expected {
            return Err(format!("E_PROJECT_RUNTIME_REVISION_CONTEXT: {key} 정체가 다릅니다"));
        }
        Ok(())
    };
    check_projection("source_epoch_identity",IdentityFamily::SourceEpoch,
        owned.sources().entry_capture().epoch_owner_bytes())?;
    check_projection("project_semantic_configuration_identity",IdentityFamily::ProjectSemanticConfiguration,
        issued.constituents()[1].as_raw())?;
    // The local-only owner above proved that the external dependency set is
    // empty. RD-631 permits null for no dependency; a supplied identity must
    // nevertheless match the actual empty graph, never a caller's graph.
    if !envelope["dependency_resolution_identity"].is_null() {
        check_projection("dependency_resolution_identity",IdentityFamily::DependencyResolution,
            issued.constituents()[2].as_raw())?;
    }
    if envelope["declared_resource_members"]!=registration_members(owned) {
        return Err("E_PROJECT_RUNTIME_REVISION_RESOURCE: 선언 자료 목록이 검증된 공급과 다릅니다".into());
    }
    Ok(SourceEnvelopeBoundRuntimeAttachment {owned,attachment,pesi:issued.pesi()})
}

fn bytes(node: &Node) -> Result<Vec<u8>, String> { serde_json::to_vec(node).map_err(|e| e.to_string()) }
fn fields<'a>(node: &'a Node, keys: &[&str]) -> Result<&'a [(String,Node)], String> {
    let Node::Object(values) = node else { return Err("E_PROJECT_RUNTIME_ARTIFACT_FIELD_COUNT: 객체가 아닙니다".into()); };
    if values.len() != keys.len() || values.iter().zip(keys).any(|((key,_),expected)| key != expected) {
        return Err("E_PROJECT_RUNTIME_ARTIFACT_FIELD_COUNT: 저장 필드 순서·개수가 다릅니다".into());
    }
    Ok(values)
}
fn text(node: &Node) -> Result<&str,String> { if let Node::Text(v) = node { Ok(v) } else { Err("E_PROJECT_RUNTIME_ARTIFACT_TYPE: 글이 필요합니다".into()) } }
fn uint(node: &Node) -> Result<u64,String> { if let Node::UInt(v) = node { Ok(*v) } else { Err("E_PROJECT_RUNTIME_ARTIFACT_TYPE: 부호 없는 정수가 필요합니다".into()) } }
fn lower_hex(value: &str) -> bool { value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) }
fn digest(data: &[u8]) -> String { hex::encode(Sha256::digest(data)) }
fn check_digest(data: &[u8], length: &Node, hash: &Node) -> Result<(),String> {
    let hash = text(hash)?;
    if uint(length)? != data.len() as u64 || !lower_hex(hash) || digest(data) != hash {
        return Err("E_PROJECT_RUNTIME_ARTIFACT_INTEGRITY: 저장 길이·해시가 다릅니다".into());
    }
    Ok(())
}
fn decode_envelope(encoded: &str) -> Result<Vec<u8>,String> {
    if encoded.len() > ENCODED_ENVELOPE_MAX { return Err("E_REVISION_ENVELOPE_ENCODING_TOO_LARGE".into()); }
    if encoded.is_empty() || encoded.len() % 4 == 1 { return Err("E_PROJECT_RUNTIME_ENVELOPE_ENCODING".into()); }
    let capacity = encoded.len().checked_mul(3).and_then(|n| n.checked_div(4)).ok_or("E_PROJECT_RUNTIME_ENVELOPE_ENCODING")?;
    if capacity > ENVELOPE_MAX { return Err("E_REVISION_ENVELOPE_TOO_LARGE".into()); }
    let mut result = Vec::with_capacity(capacity);
    let mut bits = 0u32; let mut count = 0u32;
    for b in encoded.bytes() {
        let v = match b { b'A'..=b'Z' => b-b'A', b'a'..=b'z' => b-b'a'+26, b'0'..=b'9' => b-b'0'+52, b'-' => 62, b'_' => 63, _ => return Err("E_PROJECT_RUNTIME_ENVELOPE_ENCODING".into()) };
        bits = (bits << 6) | u32::from(v); count += 6;
        if count >= 8 { count -= 8; result.push((bits >> count) as u8); }
        bits &= (1 << count) - 1;
    }
    if bits != 0 || result.len() > ENVELOPE_MAX { return Err("E_PROJECT_RUNTIME_ENVELOPE_ENCODING".into()); }
    Ok(result)
}

fn encode_envelope(raw: &[u8]) -> String {
    let alphabet=b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    // F5-GA[BASE64-INDEX]: 두 인덱스는 &63으로 0..64 안이며 alphabet은 정확히 64바이트다.
    let mut output=String::new(); let mut bits=0u32; let mut count=0u32;
    for byte in raw {
        bits=(bits<<8)|u32::from(*byte); count+=8;
        while count>=6 { count-=6; output.push(alphabet[((bits>>count)&63) as usize] as char); }
        bits&=(1<<count)-1;
    }
    if count>0 { output.push(alphabet[((bits<<(6-count))&63) as usize] as char); }
    output
}

/// RD-686 byte gates 1..8 only. Existing RD-631/663 gates remain mandatory.
pub(crate) fn check_bytes(raw: &[u8]) -> Result<ByteCheckedAttachment,String> {
    if raw.len() > OUTER_MAX { return Err("E_PROJECT_RUNTIME_ARTIFACT_TOO_LARGE".into()); }
    let raw_text = std::str::from_utf8(raw).map_err(|_| "E_PROJECT_RUNTIME_ARTIFACT_UTF8")?;
    if raw.starts_with(&[0xef,0xbb,0xbf]) || raw.first() != Some(&b'{') || raw.last() != Some(&b'}') { return Err("E_PROJECT_RUNTIME_ARTIFACT_FRAMING".into()); }
    let mut deserializer = serde_json::Deserializer::from_str(raw_text);
    let node = Node::deserialize(&mut deserializer).map_err(|e| format!("E_PROJECT_RUNTIME_ARTIFACT_JSON: {e}"))?;
    deserializer.end().map_err(|e| e.to_string())?;
    // Canonical re-encoding rejects whitespace, printable/surrogate escapes,
    // slash escapes, alternate integer forms, and noncanonical control escapes.
    if bytes(&node)? != raw { return Err("E_PROJECT_RUNTIME_ARTIFACT_CANONICAL".into()); }
    let top = fields(&node,&TOP)?;
    // F5-GA[FIXED-FIELDS]: fields가 TOP 6개와 ATTACHMENT 12개의 개수/순서를 먼저 확인한다.
    // 이후 불변 slice의 0..5/0..11 조회 동안 길이는 바뀌지 않는다. VM 입장 허가는 별도다.
    if text(&top[0].1)? != "seamgrim.project_snapshot_artifact.v1" { return Err("E_PROJECT_RUNTIME_ARTIFACT_SCHEMA".into()); }
    let attachment = fields(&top[2].1,&ATTACHMENT)?;
    if bytes(&top[2].1)?.len() > OUTER_MAX { return Err("E_PROJECT_RUNTIME_ATTACHMENT_TOO_LARGE".into()); }
    if text(&attachment[0].1)? != "ddn.workbench.project_runtime_attachment.v1" { return Err("E_PROJECT_RUNTIME_ATTACHMENT_SCHEMA".into()); }
    let pesi = text(&attachment[2].1)?;
    if !lower_hex(pesi) { return Err("E_PROJECT_RUNTIME_ATTACHMENT_PESI".into()); }
    let revision = decode_envelope(text(&attachment[3].1)?)?;
    check_digest(&revision,&attachment[4].1,&attachment[5].1)?;
    let runtime = text(&attachment[6].1)?;
    let resources = text(&attachment[9].1)?;
    if runtime.len() > OUTER_MAX || resources.len() > OUTER_MAX { return Err("E_RUNTIME_SNAPSHOT_TOO_LARGE".into()); }
    check_digest(runtime.as_bytes(),&attachment[7].1,&attachment[8].1)?;
    check_digest(resources.as_bytes(),&attachment[10].1,&attachment[11].1)?;
    let payload = Node::Object(vec![top[1].clone(),top[2].clone()]);
    let canonical = bytes(&payload)?;
    if text(&top[3].1)?.as_bytes() != canonical { return Err("E_PROJECT_RUNTIME_ARTIFACT_CANONICAL".into()); }
    check_digest(&canonical,&top[5].1,&top[4].1)?;
    Ok(ByteCheckedAttachment { source_profile: text(&attachment[1].1)?.into(), pesi_hex: pesi.into(), revision_envelope: revision, runtime_state: runtime.into(), resource_snapshot: resources.into(), project_snapshot_json: String::from_utf8(bytes(&top[1].1)?).map_err(|e| e.to_string())? })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[test]
    fn rd686_runtime_and_resource_snapshot_must_be_one_closure() {
        let resources=r#"{"schema":"ddn.resource_snapshot.v2","unit_wire_schema":"ddn.unit_value_wire.v2","components":[],"next_entity":0,"json":{},"handle":{},"value_det":{},"fixed64":{"값":"1"}}"#;
        let make_runtime=|resource:&str|format!(r#"{{"state":{{"snapshot_v2":{{"schema":"ddn.wasm_state_snapshot.v2","resource_snapshot_schema":"ddn.resource_snapshot.v2","resources":{resource}}}}}}}"#);
        let runtime=make_runtime(resources);
        check_runtime_resource_binding(&runtime,resources).unwrap();
        let foreign=resources.replace("\"1\"","\"2\"");
        assert!(check_runtime_resource_binding(&runtime,&foreign).unwrap_err().starts_with("E_PROJECT_RUNTIME_RESOURCE_MISMATCH:"));
        let duplicate=resources.replace("\"값\":\"1\"","\"값\":\"0\",\"값\":\"1\"");
        assert!(check_runtime_resource_binding(&make_runtime(&duplicate),&duplicate).is_err());
        assert!(check_runtime_resource_binding("{}",resources).is_err());
        let wrong_schema=resources.replace("snapshot.v2","snapshot.v1");
        assert!(check_runtime_resource_binding(&make_runtime(&wrong_schema),&wrong_schema).unwrap_err().starts_with("E_PROJECT_RUNTIME_RESOURCE_SCHEMA:"));
    }
    #[test]
    fn rd686_revision_context_binds_raw_source_and_rejects_stale_context() {
        use crate::project_revision_envelope_supply::{collect_saved_source_supply,SavedSourceBlobV1,CanonicalIdentityBytesV1,issue_rd631_identity_projection,IdentityFamily};
        use crate::project_snapshot_identity::Identity32;
        let definition = CanonicalIdentityBytesV1::from_owner_bytes(&[0x44]).unwrap();
        let blobs = [SavedSourceBlobV1 {path:"root.ddn",role:"root",bytes:b""},SavedSourceBlobV1 {path:"app/main.ddn",role:"ddn_source",bytes:b"1."}];
        let root = "(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let blobs = [SavedSourceBlobV1 {bytes:root.as_bytes(),..blobs[0]},blobs[1]];
        let saved = collect_saved_source_supply("root.ddn","app/main.ddn",&definition,&blobs).unwrap();
        let project = CanonicalIdentityBytesV1::from_owner_bytes(&[1]).unwrap();
        let revision = CanonicalIdentityBytesV1::from_owner_bytes(&[2]).unwrap();
        let projection = |family,id:&CanonicalIdentityBytesV1| issue_rd631_identity_projection(family,id).unwrap();
        let mut manifest = serde_json::to_value(saved.manifest()).unwrap();
        manifest.as_object_mut().unwrap().remove("limits");
        let stamp = |value:&mut serde_json::Value,key:&str| {
            value[key] = "".into(); let mut raw = serde_json::to_vec(value).unwrap(); raw.push(b'\n'); value[key] = digest(&raw).into();
        };
        stamp(&mut manifest,"canonical_manifest_sha256");
        let mut envelope = serde_json::json!({"schema":"ddn.workbench.project_snapshot_revision.v1",
            "project_identity":projection(IdentityFamily::Project,&project),
            "history_revision_identity":projection(IdentityFamily::HistoryRevision,&revision),"expected_parent_revision_identity":null,
            "source_epoch_identity":projection(IdentityFamily::SourceEpoch,&project),
            "project_semantic_configuration_identity":projection(IdentityFamily::ProjectSemanticConfiguration,&project),
            "official_entry_path":"app/main.ddn","official_entry_definition_identity":projection(IdentityFamily::Definition,&definition),
            "source_manifest":manifest,"dependency_resolution_identity":null,"declared_resource_members":[],"revision_integrity_sha256":""});
        stamp(&mut envelope,"revision_integrity_sha256");
        let mut raw = serde_json::to_vec(&envelope).unwrap(); raw.push(b'\n');
        let attachment = ByteCheckedAttachment { source_profile:"v1-core-v25".into(),pesi_hex:"00".repeat(32),revision_envelope:raw,runtime_state:"{}".into(),resource_snapshot:"{}".into(),project_snapshot_json:"{}".into() };
        let check = |project:&[u8],parent:&[u8],revision:&[u8],profile:&str,pesi:Identity32,supply:&crate::project_revision_envelope_supply::VerifiedSavedSourceV1|
            check_revision_context(&attachment,project,parent,revision,profile,pesi,supply);
        assert!(check(&[1],&[],&[2],"v1-core-v25",Identity32::from_raw([0;32]),&saved).is_ok());
        assert!(check(&[9],&[],&[2],"v1-core-v25",Identity32::from_raw([0;32]),&saved).is_err());
        assert!(check(&[1],&[0],&[2],"v1-core-v25",Identity32::from_raw([0;32]),&saved).is_err());
        assert!(check(&[1],&[],&[3],"v1-core-v25",Identity32::from_raw([0;32]),&saved).is_err());
        assert!(check(&[1],&[],&[2],"other",Identity32::from_raw([0;32]),&saved).is_err());
        assert!(check(&[1],&[],&[2],"v1-core-v25",Identity32::from_raw([1;32]),&saved).is_err());
        let changed = [blobs[0],SavedSourceBlobV1 {bytes:b"2.",..blobs[1]}];
        let changed = collect_saved_source_supply("root.ddn","app/main.ddn",&definition,&changed).unwrap();
        assert!(check(&[1],&[],&[2],"v1-core-v25",Identity32::from_raw([0;32]),&changed).is_err());
    }
    #[cfg(windows)]
    pub(crate) fn root_grant_saved_fixture() -> String {
        use crate::project_revision_envelope_supply::{collect_saved_source_supply,seal_saved_v25_sources,
            admit_saved_authored_project,SavedSourceBlobV1,CanonicalIdentityBytesV1};
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body="실행:움직씨 := { 1 보여주기. }.\n";
        let blobs=[SavedSourceBlobV1 {path:"root.ddn",role:"root",bytes:root.as_bytes()},
            SavedSourceBlobV1 {path:"app/main.ddn",role:"ddn_source",bytes:body.as_bytes()}];
        let definition=CanonicalIdentityBytesV1::from_owner_bytes(&[91;32]).unwrap();
        let sources=std::sync::Arc::new(seal_saved_v25_sources(
            collect_saved_source_supply("root.ddn","app/main.ddn",&definition,&blobs).unwrap()).unwrap());
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:blobs.iter().map(|blob|SourceFile {path:blob.path.into(),text:std::str::from_utf8(blob.bytes).unwrap().into()}).collect(),
        },String::new()).unwrap();
        let owned=admit_saved_authored_project(sources,project.save().unwrap().as_bytes()).unwrap();
        let resources=r#"{"schema":"ddn.resource_snapshot.v2","unit_wire_schema":"ddn.unit_value_wire.v2","components":[],"next_entity":0,"json":{},"handle":{},"value_det":{},"fixed64":{}}"#;
        let runtime=serde_json::json!({"state":{"snapshot_v2":{"schema":"ddn.wasm_state_snapshot.v2",
            "resource_snapshot_schema":"ddn.resource_snapshot.v2","resources":serde_json::from_str::<serde_json::Value>(resources).unwrap()}}}).to_string();
        encode_saved_runtime_artifact(&owned,&runtime,resources,&[],&[2]).unwrap()
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "Only the parent-owned test supplies an exact root/source/locator"]
    fn r4_root_grant_cold_process_probe() {
        let root=std::path::PathBuf::from(std::env::var_os("DDN_R4_TEST_ROOT").expect("explicit owned root"));
        let source=std::fs::read_to_string(std::env::var_os("DDN_R4_TEST_SOURCE").expect("explicit source")).unwrap();
        let attachment=check_bytes(source.as_bytes()).unwrap();
        let owned=restore_saved_runtime_payload(&attachment).unwrap();
        let context=check_saved_revision_context(&attachment,&owned,&[],&[2]).unwrap();
        let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_admitted(&root).unwrap();
        let locator=std::env::var("DDN_R4_TEST_LOCATOR").unwrap();
        let expected=std::env::var("DDN_R4_TEST_EXPECT").unwrap();
        let outcome=if expected=="read-denied" {
            adapter.reopen_saved_root_grant(&locator,&context)
        } else {
            if let Some(fault)=expected.strip_prefix("fault:") { adapter.test_root_grant_confirmation_fault(fault); }
            adapter.confirm_saved_root_grant(&locator,&context)
        };
        match expected.as_str() {
            "active" => { let grant=outcome.expect("fresh process must verify original target"); grant.verify_current(&adapter,&context).unwrap(); },
            "reject" | "read-denied" => assert!(outcome.is_err(),"read/foreign/revoked target must not admit"),
            name if name.starts_with("fault:") => {
                use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
                let error=outcome.err().expect("failed current confirmation must issue no capability");
                if name.ends_with("flush") { assert!(matches!(error,Failure::SyncFailed)); }
                else { assert!(matches!(error,Failure::RereadFailed)); }
            },
            _ => panic!("explicit test expectation"),
        }
        println!("R4_COLD_PROBE_COMPLETED");
    }

    #[cfg(windows)]
    pub(crate) fn root_grant_cold_probe(target:&std::path::Path, source:&std::path::Path, locator:&str, expected:&str) {
        let child=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","unit_wire::project_runtime_attachment::tests::r4_root_grant_cold_process_probe","--ignored","--nocapture"])
            .env("DDN_R4_TEST_ROOT",target).env("DDN_R4_TEST_SOURCE",source)
            .env("DDN_R4_TEST_LOCATOR",locator).env("DDN_R4_TEST_EXPECT",expected)
            .stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
        let pid=child.id();
        let output=child.wait_with_output().unwrap();
        println!("R7_COLD_CHILD pid={pid} expected={expected} status={}\n{}\n{}",output.status,
            String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
        assert!(output.status.success(),"cold child must exit normally with the exact expected result");
        assert!(String::from_utf8_lossy(&output.stdout).contains("R4_COLD_PROBE_COMPLETED"),"exact probe must run");
    }

    #[cfg(windows)]
    #[test]
    fn r4_root_grant_saved_caller_registration_cold_reopen_and_revocation() {
        use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
        let mut nonce=[0u8;32]; getrandom::fill(&mut nonce).unwrap();
        let directory=std::env::temp_dir().join(format!("ddn-rd669-source-{}",hex::encode(nonce)));
        std::fs::create_dir(&directory).unwrap();
        let root=directory.join("original"); std::fs::create_dir(&root).unwrap();
        let source=directory.join("saved-project.json");
        let original=root_grant_saved_fixture(); std::fs::write(&source,&original).unwrap();
        let attachment=check_bytes(original.as_bytes()).unwrap();
        let owned=restore_saved_runtime_payload(&attachment).unwrap();
        let context=check_saved_revision_context(&attachment,&owned,&[],&[2]).unwrap();
        let mut adapter=SqliteAdapterV1::provision(&root).unwrap();
        let grant=adapter.register_saved_root_grant(&context).unwrap();
        let locator=grant.locator();
        grant.verify_current(&adapter,&context).unwrap();
        assert!(adapter.register_saved_root_grant(&context).is_err(),"one provision cannot issue G2");
        drop(grant); drop(adapter);
        let probe=|target:&std::path::Path, source:&std::path::Path, expected:&str| root_grant_cold_probe(target,source,&locator,expected);
        probe(&root,&source,"active");
        let foreign=directory.join("copied"); std::fs::create_dir(&foreign).unwrap();
        for name in ["vc04_snapshot_v2.sqlite3","vc04_snapshot_v2.sqlite3-wal","vc04_snapshot_v2.sqlite3-shm"] {
            if root.join(name).exists() { std::fs::copy(root.join(name),foreign.join(name)).unwrap(); }
        }
        // The child first admits the ordinary SQLite schema/capability, then
        // rejects the original grant for these physically different objects.
        probe(&foreign,&source,"reject");
        let other_source=directory.join("other-project.json");
        std::fs::write(&other_source,root_grant_saved_fixture()).unwrap();
        probe(&root,&other_source,"reject");
        let mut adapter=SqliteAdapterV1::open_admitted(&root).unwrap();
        assert!(adapter.register_saved_root_grant(&context).is_err(),"existing empty container cannot be re-enrolled");
        let grant=adapter.confirm_saved_root_grant(&locator,&context).unwrap();
        grant.revoke(&adapter,&context).unwrap();
        assert!(grant.verify_current(&adapter,&context).is_err(),"stale in-process capability must reject");
        assert!(adapter.register_saved_root_grant(&context).is_err(),"revocation must not restore freshness");
        drop(grant); drop(adapter);
        probe(&root,&source,"reject");
        println!("R4 retained project fixture: {}",directory.display());
        println!("R4 retained revoked source-bound locator: {locator}");
    }

    #[test]
    fn rd686_saved_revision_uses_real_source_definition_configuration_owners() {
        // A3-P8/P9: real admitted owners, not expected bytes copied from input.
        use crate::project_revision_envelope_supply::{collect_saved_source_supply,seal_saved_v25_sources,
            admit_saved_authored_project,SavedSourceBlobV1,CanonicalIdentityBytesV1,
            issue_rd631_identity_projection,IdentityFamily};
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body="실행:움직씨 := { 1 보여주기. }.\n";
        let blobs=[SavedSourceBlobV1 {path:"root.ddn",role:"root",bytes:root.as_bytes()},
            SavedSourceBlobV1 {path:"app/main.ddn",role:"ddn_source",bytes:body.as_bytes()}];
        let opaque=CanonicalIdentityBytesV1::from_owner_bytes(&[91;32]).unwrap();
        let sources=std::sync::Arc::new(seal_saved_v25_sources(
            collect_saved_source_supply("root.ddn","app/main.ddn",&opaque,&blobs).unwrap()).unwrap());
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:blobs.iter().map(|b|SourceFile {path:b.path.into(),text:std::str::from_utf8(b.bytes).unwrap().into()}).collect(),
        },String::new()).unwrap();
        let owned=admit_saved_authored_project(sources,project.save().unwrap().as_bytes()).unwrap();
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(&owned).unwrap();
        let project_raw=hex::decode(project.authored_identity.logical_dependency_id.as_str()).unwrap();
        let projection=|family,raw:&[u8]|issue_rd631_identity_projection(family,
            &CanonicalIdentityBytesV1::from_owner_bytes(raw).unwrap()).unwrap();
        let stamp=|node:&mut serde_json::Value,key:&str| {
            node[key]="".into(); let mut raw=serde_json::to_vec(node).unwrap(); raw.push(b'\n'); node[key]=digest(&raw).into();
        };
        let mut manifest=serde_json::to_value(owned.sources().saved().manifest()).unwrap();
        manifest.as_object_mut().unwrap().remove("limits"); stamp(&mut manifest,"canonical_manifest_sha256");
        let mut envelope=serde_json::json!({"schema":"ddn.workbench.project_snapshot_revision.v1",
            "project_identity":projection(IdentityFamily::Project,&project_raw),
            "history_revision_identity":projection(IdentityFamily::HistoryRevision,&[2]),"expected_parent_revision_identity":null,
            "source_epoch_identity":projection(IdentityFamily::SourceEpoch,owned.sources().entry_capture().epoch_owner_bytes()),
            "project_semantic_configuration_identity":projection(IdentityFamily::ProjectSemanticConfiguration,issued.constituents()[1].as_raw()),
            "official_entry_path":"app/main.ddn",
            "official_entry_definition_identity":projection(IdentityFamily::Definition,owned.definition_identity().as_bytes()),
            "source_manifest":manifest,"dependency_resolution_identity":null,"declared_resource_members":[],"revision_integrity_sha256":""});
        let resources=r#"{"schema":"ddn.resource_snapshot.v2","unit_wire_schema":"ddn.unit_value_wire.v2","components":[],"next_entity":0,"json":{},"handle":{},"value_det":{},"fixed64":{"값":"1"}}"#;
        let runtime=serde_json::json!({"state":{"snapshot_v2":{"schema":"ddn.wasm_state_snapshot.v2",
            "resource_snapshot_schema":"ddn.resource_snapshot.v2",
            "resources":serde_json::from_str::<serde_json::Value>(resources).unwrap()}}}).to_string();
        let payload=saved_runtime_project_payload_json(&owned).unwrap();
        let saved_bytes=encode_saved_runtime_artifact(&owned,&runtime,resources,&[],&[2]).unwrap();
        let written=check_bytes(saved_bytes.as_bytes()).unwrap();
        let written_owner=restore_saved_runtime_payload(&written).unwrap();
        assert_eq!(check_saved_revision_context(&written,&written_owner,&[],&[2]).unwrap().pesi(),issued.pesi());
        let attachment=|node:&mut serde_json::Value| {
            stamp(node,"revision_integrity_sha256");
            let mut raw=serde_json::to_vec(node).unwrap(); raw.push(b'\n');
            ByteCheckedAttachment {source_profile:"v1-core-v25".into(),pesi_hex:issued.pesi().to_hex(),
                revision_envelope:raw,runtime_state:runtime.clone(),resource_snapshot:resources.into(),project_snapshot_json:payload.clone()}
        };
        let checked=attachment(&mut envelope);
        let context=check_saved_revision_context(&checked,&owned,&[],&[2]).unwrap();
        assert_eq!(context.pesi(),issued.pesi());
        assert!(std::ptr::eq(context.owned(),&owned));
        let check_supply = |bytes: &[u8]| super::super::resumable_complete_state::check_revision_envelope_supply(bytes, &context);
        assert!(check_supply(&checked.revision_envelope).is_ok());
        assert_eq!(context.revision_envelope().last(), Some(&b'\n'));
        assert!(check_supply(&checked.revision_envelope[..checked.revision_envelope.len()-1]).is_err());
        let mut corrupt_envelope = checked.revision_envelope.clone();
        corrupt_envelope[0] ^= 1;
        assert!(check_supply(&corrupt_envelope).is_err());
        let mut suffixed_envelope = checked.revision_envelope.clone();
        suffixed_envelope.push(b'\n');
        assert!(check_supply(&suffixed_envelope).is_err());
        // Static complete-state assembly is a staging prerequisite only. The
        // runtime/resource carrier above is not a durable continuation proof.
        let empty=[0;8];
        let initial=super::super::CompleteStateFrameV1 {
            pesi:issued.pesi().as_raw(),resources:&empty,parameters:&empty,next_tick:0,
            subjects:&empty,atom_evidence:&[],subject_custody:&[],
        }.encode_canonical().unwrap();
        let program=crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(
            owned.sources().clone(),ddonirang_lang::ParseMode::Strict,ddonirang_lang::AgeTarget::Age3,
        ).unwrap();
        let canonical=program.canonical_ddn();
        let mut runner=crate::ddn_runtime::DdnRunner::new(program,"실행");
        runner.configure_project_entry("실행").unwrap();
        let mut descriptor=vec![0,2];
        let add=|out:&mut Vec<u8>,bytes:&[u8]| {out.extend((bytes.len() as u32).to_be_bytes());out.extend(bytes);};
        add(&mut descriptor,b"ddn.resume.source-rebuild-descriptor.v1");add(&mut descriptor,b"v1-core-v25");
        descriptor.extend([3,1]);add(&mut descriptor,canonical.as_bytes());descriptor.push(1);descriptor.extend([0;24]);
        add(&mut descriptor,owned.definition_identity().as_bytes());add(&mut descriptor,owned.definition_identity().as_bytes());
        descriptor.push(1);add(&mut descriptor,owned.project().save().unwrap().as_bytes());add(&mut descriptor,&initial);
        descriptor.extend([0;16]);descriptor.push(0);descriptor.extend([0;8]);
        add(&mut descriptor,runner.execution_instance_owner_bytes());descriptor.extend([0;4]);
        use super::super::resumable_complete_state as complete;
        let supplies=[checked.revision_envelope.clone(),complete::encode_source_capture_supply(owned.sources()).unwrap(),
            complete::encode_constituent_supply(&owned).unwrap(),complete::encode_logical_execution_contract_supply(&owned,&runner).unwrap(),
            vec![1],vec![1],vec![1],vec![1],vec![1],descriptor];
        let assemble=|s:&[Vec<u8>;10],world_bytes:&[u8]| {
            let layout=super::super::complete_state_frame::ResumableCompleteStateLayoutV2 {
                world:super::super::check_complete_state_frame_v1(world_bytes,issued.pesi().as_raw()).unwrap(),
                supplies:std::array::from_fn(|i|s[i].as_slice()),
            };
            complete::rebuild_complete_state_source_candidate(&layout,&context).is_ok()
        };
        assert!(assemble(&supplies,&initial));
        for field in [0,1,2,3,9] {
            let mut bad=supplies.clone();bad[field][0]^=1;assert!(!assemble(&bad,&initial));
        }
        let different=super::super::CompleteStateFrameV1 {
            pesi:issued.pesi().as_raw(),resources:&empty,parameters:&empty,next_tick:1,
            subjects:&empty,atom_evidence:&[],subject_custody:&[],
        }.encode_canonical().unwrap();
        assert!(!assemble(&supplies,&different));
        // Actual fresh-run/request crosswalk, including both resource carriers.
        // Non-resource runtime state, RNG, transport, checkpoint and storage
        // admission remain separate. Test binding names are component fixtures.
        use crate::logical_input_tape::{LogicalInputBinding,LogicalSnapshot,LOGICAL_INPUT_PROFILE};
        use sha2::Digest;
        let binding=LogicalInputBinding {
            source_profile:LOGICAL_INPUT_PROFILE.into(),input_schema:"test.schema@1".into(),
            channel_registry:"test.channels@1".into(),
            source_revision:format!("sha256:{}",hex::encode(sha2::Sha256::digest(
                owned.sources().preparation().source.as_bytes()))),
            initial_world:crate::runtime_surface::current_state_hash(&ddonirang_core::platform::NuriWorld::new()),
            initial_rng:7,recording_policy:"test.recording@1".into(),integrity_policy:"test.integrity@1".into(),
            madi_hz:60,dt_raw:crate::input_receipt::alpha_host_dt().raw_i64(),
            channels:std::collections::BTreeMap::new(),initial_held:std::collections::BTreeMap::new(),
        };
        let mut active=complete::CheckpointedLogicalRunV1::fresh(&owned,binding.clone(),ddonirang_lang::AgeTarget::Age3).unwrap();
        let mut active_supplies=supplies.clone();
        active_supplies[4]=complete::encode_processed_input_supply(active.session()).unwrap();
        active_supplies[5]=active.logical_rng_supply().unwrap();
        active_supplies[6]=active.execution_position_supply().unwrap();
        active_supplies[7]=complete::encode_replayed_work_supply(active.session()).unwrap();
        active_supplies[8]=active.checkpoint_predecessor_supply().unwrap();
        active_supplies[9]=active.source_rebuild_descriptor_supply().unwrap();
        let transaction=crate::snapshot_v2_durable_transaction::TransactionBindingV1 {
            project:project_raw.clone(),expected_parent:vec![],new_revision:vec![2],
            execution_profile:b"v1-core-v25".to_vec(),pesi:*issued.pesi().as_raw(),
            snapshot_sha256:[0;32],snapshot_length:1,revision_integrity:[0;32],
        };
        let active_resources=resources.replace("\"값\":\"1\"","");
        let mut active_checked=attachment(&mut envelope);
        active_checked.resource_snapshot=active_resources.clone();
        // serde_json serialization may reorder the embedded resource object.
        let mut active_runtime:serde_json::Value=serde_json::from_str(&runtime).unwrap();
        active_runtime["state"]["snapshot_v2"]["resources"]=serde_json::from_str(&active_resources).unwrap();
        active_checked.runtime_state=active_runtime.to_string();
        let active_context=check_saved_revision_context(&active_checked,&owned,&[],&[2]).unwrap();
        let active_check=|run:&complete::CheckpointedLogicalRunV1,s:&[Vec<u8>;10],
                          transaction:crate::snapshot_v2_durable_transaction::TransactionBindingV1,
                          ctx:&SourceBoundSavedRuntimeAttachment<'_>,world_bytes:&[u8]| {
            let request=crate::snapshot_v2_host_bridge::ResumableRequestCandidateV2 {
                binding:transaction,revision_bytes:&checked.revision_envelope,request_kind:1,snapshot_bytes:world_bytes,
                layout:super::super::complete_state_frame::ResumableCompleteStateLayoutV2 {
                    world:super::super::check_complete_state_frame_v1(world_bytes,issued.pesi().as_raw()).unwrap(),
                    supplies:std::array::from_fn(|i|s[i].as_slice()),
                },
            };
            run.check_active_request_execution_candidate(&request,ctx).is_ok()
        };
        let before=active.session().summary();let tape_before=active.session().seal().unwrap();
        assert!(active_check(&active,&active_supplies,transaction.clone(),&active_context,&initial));
        // Both legacy carrier hashes can be valid and mutually consistent but
        // still describe a foreign world. Do not use them as runtime authority.
        assert!(!active_check(&active,&active_supplies,transaction.clone(),&context,&initial));
        for field in [4,7,8,9] {
            let mut bad=active_supplies.clone();bad[field][0]^=1;
            assert!(!active_check(&active,&bad,transaction.clone(),&active_context,&initial),"active field {field}");
        }
        let mut foreign=transaction.clone();foreign.project=vec![9];
        assert!(!active_check(&active,&active_supplies,foreign,&active_context,&initial));
        let mut foreign=transaction.clone();foreign.execution_profile=b"foreign".to_vec();
        assert!(!active_check(&active,&active_supplies,foreign,&active_context,&initial));
        assert_eq!(active.session().summary(),before);
        assert_eq!(active.session().seal().unwrap(),tape_before);
        assert_eq!(active.checkpoint_predecessor_supply().unwrap(),vec![0]);
        active.step(&LogicalSnapshot {madi:0,held:std::collections::BTreeMap::new(),events:vec![]}).unwrap();
        assert!(!active_check(&active,&active_supplies,transaction.clone(),&active_context,&initial));
        assert_eq!(active.checkpoint_predecessor_supply().unwrap(),vec![0]);
        // Independently calculated world after `1 보여주기`: one output row,
        // no named numeric resource. Never copy runtime output into a golden.
        use ddonirang_core::platform::{ResourceValue as V,ResourceMapEntry};
        let pairs=[("kind",V::String("output".into())),
            ("line_no",V::Fixed64(ddonirang_core::Fixed64::from_i64(1))),
            ("text",V::String("1".into())),("tick",V::Fixed64(ddonirang_core::Fixed64::ZERO))];
        let logs=V::List(vec![V::Map(pairs.into_iter().map(|(name,value)| {
            let key=V::String(name.into());(key.canon_key(),ResourceMapEntry {key,value})
        }).collect())]);
        let name="output_log";let mut leaf=vec![];logs.encode_canon(&mut leaf);
        let mut rows=1u64.to_be_bytes().to_vec();
        rows.extend((name.len() as u32).to_be_bytes());rows.extend(name.as_bytes());
        rows.extend((leaf.len() as u64).to_be_bytes());rows.extend(leaf);
        let current=super::super::CompleteStateFrameV1 {
            pesi:issued.pesi().as_raw(),resources:&rows,parameters:&empty,next_tick:1,
            subjects:&empty,atom_evidence:&[],subject_custody:&[],
        }.encode_canonical().unwrap();
        let mut current_resources:serde_json::Value=serde_json::from_str(&active_resources).unwrap();
        current_resources["value_det"]["output_log"]=crate::unit_wire::resource_value_to_detjson(&logs);
        let mut current_checked=attachment(&mut envelope);
        current_checked.resource_snapshot=current_resources.to_string();
        active_runtime["state"]["snapshot_v2"]["resources"]=current_resources.clone();
        current_checked.runtime_state=active_runtime.to_string();
        let current_context=check_saved_revision_context(&current_checked,&owned,&[],&[2]).unwrap();
        let mut current_supplies=active_supplies.clone();
        current_supplies[4]=complete::encode_processed_input_supply(active.session()).unwrap();
        current_supplies[5]=active.logical_rng_supply().unwrap();
        current_supplies[6]=active.execution_position_supply().unwrap();
        current_supplies[7]=complete::encode_replayed_work_supply(active.session()).unwrap();
        current_supplies[9]=active.source_rebuild_descriptor_supply().unwrap();
        assert!(active_check(&active,&current_supplies,transaction.clone(),&current_context,&current));
        let mut stale_position=current_supplies.clone();stale_position[6]=active_supplies[6].clone();
        assert!(!active_check(&active,&stale_position,transaction.clone(),&current_context,&current));
        assert!(!active_check(&active,&current_supplies,transaction.clone(),&active_context,&current));
        // A forged view-prefixed resource would evade the ordinary world hash.
        // Both valid JSON carriers carry it, but full-state crosswalk rejects.
        current_resources["fixed64"]["보개_위조"]=serde_json::json!({"raw_i64":"1"});
        let mut forged_checked=attachment(&mut envelope);
        forged_checked.resource_snapshot=current_resources.to_string();
        active_runtime["state"]["snapshot_v2"]["resources"]=current_resources.clone();
        forged_checked.runtime_state=active_runtime.to_string();
        let forged_context=check_saved_revision_context(&forged_checked,&owned,&[],&[2]).unwrap();
        let mut forged_world=ddonirang_core::platform::NuriWorld::new();
        super::super::restore_world_resources_snapshot(&mut forged_world,&current_resources).unwrap();
        assert_eq!(crate::runtime_surface::current_state_hash(&forged_world),
            crate::runtime_surface::current_state_hash(active.session().committed_world()));
        assert_ne!(forged_world.state_hash(),active.session().committed_world().state_hash());
        let before=active.session().summary();let tape_before=active.session().seal().unwrap();
        assert!(!active_check(&active,&current_supplies,transaction.clone(),&forged_context,&current));
        assert_eq!(active.session().summary(),before);
        assert_eq!(active.session().seal().unwrap(),tape_before);
        assert_eq!(active.checkpoint_predecessor_supply().unwrap(),vec![0]);
        #[cfg(not(target_arch="wasm32"))]
        {
            // Real nineteen-field capsule, common Rust transport, real durable
            // adapter. Outer checksums are recomputed on the semantic mutation.
            let layout=super::super::complete_state_frame::ResumableCompleteStateLayoutV2 {
                world:super::super::check_complete_state_frame_v1(&current,issued.pesi().as_raw()).unwrap(),
                supplies:std::array::from_fn(|i|current_supplies[i].as_slice()),
            };
            let snapshot=layout.encode_canonical().unwrap();
            let mut bound=transaction.clone();bound.snapshot_length=snapshot.len() as u64;
            bound.snapshot_sha256=sha2::Sha256::digest(&snapshot).into();
            let revision:serde_json::Value=serde_json::from_slice(&checked.revision_envelope).unwrap();
            bound.revision_integrity=hex::decode(revision["revision_integrity_sha256"].as_str().unwrap()).unwrap()
                .try_into().unwrap();
            let request=crate::snapshot_v2_host_bridge::ResumableRequestCandidateV2 {
                binding:bound.clone(),layout,revision_bytes:&checked.revision_envelope,request_kind:1,snapshot_bytes:&snapshot,
            };
            let raw=request.encode_canonical().unwrap();
            let live_world=active.encode_world_frame().unwrap();
            assert_eq!(live_world,current,"producer world must match the independently constructed golden");
            let live_artifact=active.export_saved_artifact(&[],&[2]).unwrap();
            let live_checked=check_bytes(live_artifact.as_bytes()).unwrap();
            let live_context=check_saved_revision_context(&live_checked,&owned,&[],&[2]).unwrap();
            let produced=active.encode_checkpoint_request(&live_world,&live_context,&[],&[2]).unwrap();
            assert_eq!(produced,raw,"live producer must retain the exact approved request bytes");
            assert!(active.encode_checkpoint_request(&current,&forged_context,&[],&[2]).is_err());
            assert!(active.encode_checkpoint_request(&current,&current_context,&[9],&[2]).is_err());
            assert!(active.encode_checkpoint_request(&current,&current_context,&[],&[9]).is_err());
            let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let root=std::env::temp_dir().join(format!("ddn-logical-native-checkpoint-{}-{nonce}",std::process::id()));
            std::fs::create_dir(&root).unwrap();
            let adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::provision(&root).unwrap();
            let mut tampered=current_supplies.clone();
            let progress=tampered[5].len()-16;tampered[5][progress..progress+8].copy_from_slice(&6u64.to_be_bytes());
            let bad_layout=super::super::complete_state_frame::ResumableCompleteStateLayoutV2 {
                world:super::super::check_complete_state_frame_v1(&current,issued.pesi().as_raw()).unwrap(),
                supplies:std::array::from_fn(|i|tampered[i].as_slice()),
            };
            let bad_snapshot=bad_layout.encode_canonical().unwrap();let mut bad_binding=bound.clone();
            bad_binding.snapshot_sha256=sha2::Sha256::digest(&bad_snapshot).into();bad_binding.snapshot_length=bad_snapshot.len() as u64;
            let bad_request=crate::snapshot_v2_host_bridge::ResumableRequestCandidateV2 {
                binding:bad_binding,layout:bad_layout,revision_bytes:&checked.revision_envelope,
                request_kind:1,snapshot_bytes:&bad_snapshot,
            }.encode_canonical().unwrap();
            assert!(active.save_native_checkpoint(&bad_request,&current_context,&adapter,None).is_err());
            assert_eq!(active.checkpoint_predecessor_supply().unwrap(),vec![0]);
            let receipt=active.save_native_checkpoint(&produced,&live_context,&adapter,None).unwrap();
            crate::snapshot_v2_host_bridge::check_committed_receipt_payload_v1(&receipt).unwrap();
            assert_eq!(active.checkpoint_predecessor_supply().unwrap()[0],1);
            let digest=bound.transaction_digest().unwrap();drop(adapter);
            let reopened_adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_admitted(&root).unwrap();
            let chain=reopened_adapter.reread_committed(&digest,&bound.project,&bound.new_revision).unwrap();
            let stored=chain.payload.unwrap();assert_eq!(stored.snapshot,snapshot);assert_eq!(chain.head,chain.revision);
            let mut rebuilt=complete::CheckpointedLogicalRunV1::restore_native_checkpoint(
                &reopened_adapter,&digest,&bound.new_revision,&current_context,&binding).unwrap();
            let mut foreign_binding=binding.clone();foreign_binding.initial_rng^=1;
            assert!(complete::CheckpointedLogicalRunV1::restore_native_checkpoint(
                &reopened_adapter,&digest,&bound.new_revision,&current_context,&foreign_binding).is_err());
            assert!(complete::CheckpointedLogicalRunV1::restore_native_checkpoint(
                &reopened_adapter,&digest,&bound.new_revision,&forged_context,&binding).is_err());
            assert!(complete::CheckpointedLogicalRunV1::restore_native_checkpoint(
                &reopened_adapter,&digest,&[9],&current_context,&binding).is_err());
            assert_eq!(rebuilt.checkpoint_predecessor_supply().unwrap(),active.checkpoint_predecessor_supply().unwrap());
            assert_eq!(rebuilt.source_rebuild_descriptor_supply().unwrap(),active.source_rebuild_descriptor_supply().unwrap());
            let next=LogicalSnapshot {madi:1,held:std::collections::BTreeMap::new(),events:vec![]};
            let artifact=encode_saved_runtime_artifact(&owned,&current_checked.runtime_state,
                &current_checked.resource_snapshot,&[],&[2]).unwrap();
            let artifact_checked=check_bytes(artifact.as_bytes()).unwrap();
            assert_eq!(artifact_checked.revision_envelope,current_checked.revision_envelope);
            let suffix=crate::logical_input_tape::LogicalRecordRequest {binding:binding.clone(),snapshots:vec![next.clone()]};
            // Release the actual exclusive Native adapter lease before the
            // frontdoor opens its own connection. Never remove a lock file.
            drop(reopened_adapter);
            let frontdoor=crate::runtime_surface::resume_saved_project_artifact_native(
                &artifact,&[],&[2],&root,&digest,&suffix).unwrap();
            let reopened_adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_admitted(&root).unwrap();
            assert_eq!(active.step(&next).unwrap(),rebuilt.step(&next).unwrap());
            assert_eq!(frontdoor,active.session().summary());
            assert_eq!(active.session().committed_world().state_hash(),rebuilt.session().committed_world().state_hash());
            // Reusing a stale pre-save request cannot erase the newly retained
            // durable predecessor or replace the known-good SQL head.
            assert!(active.save_native_checkpoint(&raw,&current_context,&reopened_adapter,None).is_err());
            let after=reopened_adapter.reread_committed(&digest,&bound.project,&bound.new_revision).unwrap();
            assert_eq!(after.payload.unwrap(),stored);assert_eq!(after.head,chain.head);
            // A NEW execution on a noninitial saved project is a first runtime
            // checkpoint: field11 must stay absent while project CAS uses rev2.
            let mut fresh_run=complete::CheckpointedLogicalRunV1::fresh(&owned,binding.clone(),ddonirang_lang::AgeTarget::Age3).unwrap();
            fresh_run.step(&LogicalSnapshot {madi:0,held:std::collections::BTreeMap::new(),events:vec![]}).unwrap();
            assert_eq!(fresh_run.checkpoint_predecessor_supply().unwrap(),vec![0]);
            let mut next_envelope:serde_json::Value=serde_json::from_slice(&checked.revision_envelope).unwrap();
            next_envelope["history_revision_identity"]=projection(IdentityFamily::HistoryRevision,&[3]).into();
            next_envelope["expected_parent_revision_identity"]=projection(IdentityFamily::HistoryRevision,&[2]).into();
            stamp(&mut next_envelope,"revision_integrity_sha256");
            let mut next_envelope_raw=serde_json::to_vec(&next_envelope).unwrap();next_envelope_raw.push(b'\n');
            let next_checked=ByteCheckedAttachment {
                source_profile:current_checked.source_profile.clone(),pesi_hex:current_checked.pesi_hex.clone(),
                revision_envelope:next_envelope_raw,runtime_state:current_checked.runtime_state.clone(),
                resource_snapshot:current_checked.resource_snapshot.clone(),project_snapshot_json:current_checked.project_snapshot_json.clone(),
            };
            let next_context=check_saved_revision_context(&next_checked,&owned,&[2],&[3]).unwrap();
            let mut next_supplies=current_supplies.clone();next_supplies[0]=next_checked.revision_envelope.clone();
            next_supplies[8]=fresh_run.checkpoint_predecessor_supply().unwrap();
            next_supplies[9]=fresh_run.source_rebuild_descriptor_supply().unwrap();
            let next_layout=super::super::complete_state_frame::ResumableCompleteStateLayoutV2 {
                world:super::super::check_complete_state_frame_v1(&current,issued.pesi().as_raw()).unwrap(),
                supplies:std::array::from_fn(|i|next_supplies[i].as_slice()),
            };
            let next_snapshot=next_layout.encode_canonical().unwrap();let mut next_bound=bound.clone();
            next_bound.expected_parent=vec![2];next_bound.new_revision=vec![3];
            next_bound.revision_integrity=hex::decode(next_envelope["revision_integrity_sha256"].as_str().unwrap()).unwrap().try_into().unwrap();
            next_bound.snapshot_sha256=sha2::Sha256::digest(&next_snapshot).into();next_bound.snapshot_length=next_snapshot.len() as u64;
            let next_raw=crate::snapshot_v2_host_bridge::ResumableRequestCandidateV2 {
                binding:next_bound.clone(),layout:next_layout,revision_bytes:&next_checked.revision_envelope,
                request_kind:1,snapshot_bytes:&next_snapshot,
            }.encode_canonical().unwrap();
            let expected_head=after.head.as_ref().unwrap();
            let produced_next=fresh_run.encode_checkpoint_request(&current,&next_context,&[2],&[3]).unwrap();
            assert_eq!(produced_next,next_raw,"fresh execution and project CAS remain distinct");
            assert!(fresh_run.save_native_checkpoint(&produced_next,&next_context,&reopened_adapter,None).is_err());
            fresh_run.save_native_checkpoint(&produced_next,&next_context,&reopened_adapter,Some(expected_head)).unwrap();
            let recovered_fresh=complete::CheckpointedLogicalRunV1::restore_native_checkpoint(
                &reopened_adapter,&next_bound.transaction_digest().unwrap(),&[3],&next_context,&binding).unwrap();
            assert_eq!(recovered_fresh.source_rebuild_descriptor_supply().unwrap(),fresh_run.source_rebuild_descriptor_supply().unwrap());
            // Preserve product-encoded artifacts for separate-process CLI
            // admission checks. They are not a test-only source translation.
            let cold_artifact=encode_saved_runtime_artifact(&owned,&next_checked.runtime_state,
                &next_checked.resource_snapshot,&[2],&[3]).unwrap();
            assert_eq!(check_bytes(cold_artifact.as_bytes()).unwrap().revision_envelope,next_checked.revision_envelope);
            fresh_run.step(&next).unwrap();
            std::fs::write(root.join("saved-project.json"),cold_artifact).unwrap();
            std::fs::write(root.join("next-input.json"),serde_json::to_vec(&suffix).unwrap()).unwrap();
            let failed_suffix=crate::logical_input_tape::LogicalRecordRequest {
                binding:binding.clone(),snapshots:vec![next.clone(),LogicalSnapshot{madi:99,..next.clone()}],
            };
            std::fs::write(root.join("failed-next-input.json"),serde_json::to_vec(&failed_suffix).unwrap()).unwrap();
            std::fs::write(root.join("continuous-expected.json"),serde_json::to_vec(&fresh_run.session().summary()).unwrap()).unwrap();
            std::fs::write(root.join("cold-resume-identity.json"),serde_json::to_vec(&serde_json::json!({
                "parent":"02","revision":"03","transaction":hex::encode(next_bound.transaction_digest().unwrap()),
                "madi":1,"source_status":"WIP_NOT_FIXED"
            })).unwrap()).unwrap();
            // The directory is retained as execution evidence, not deleted.
            eprintln!("NATIVE_LOGICAL_CHECKPOINT_EVIDENCE={}",root.display());
            let fresh_root=root.join("frontdoor-fresh");std::fs::create_dir(&fresh_root).unwrap();
            let initial_request=crate::logical_input_tape::LogicalRecordRequest {
                binding:binding.clone(),snapshots:vec![LogicalSnapshot{madi:0,..next.clone()}],
            };
            std::fs::write(root.join("authored-project.json"),owned.project().save().unwrap()).unwrap();
            std::fs::write(root.join("initial-input.json"),serde_json::to_vec(&initial_request).unwrap()).unwrap();
            let native=crate::runtime_surface::record_authored_logical_project_native(
                &owned.project().save().unwrap(),&initial_request,&[],&[2],&fresh_root,true,None).unwrap();
            assert_eq!(native.summary,serde_json::from_str::<serde_json::Value>(&live_artifact).unwrap()
                ["runtime_attachment"]["runtime_state_json_utf8"].as_str()
                .map(|text|serde_json::from_str::<serde_json::Value>(text).unwrap())
                .map(|mut value|{value.as_object_mut().unwrap().remove("schema");value.as_object_mut().unwrap().remove("state");value}).unwrap());
            let cold=crate::runtime_surface::resume_saved_project_artifact_native(
                &native.saved_artifact,&[],&[2],&fresh_root,&native.transaction_digest,&suffix).unwrap();
            assert_eq!(cold,frontdoor);
            assert!(crate::runtime_surface::record_authored_logical_project_native(
                &owned.project().save().unwrap(),&initial_request,&[],&[2],&fresh_root,true,None).is_err(),
                "explicit provisioning must never overwrite an existing store");
            assert_eq!(crate::runtime_surface::resume_saved_project_artifact_native(
                &native.saved_artifact,&[],&[2],&fresh_root,&native.transaction_digest,&suffix).unwrap(),cold);
            let continued=crate::runtime_surface::resume_and_save_logical_project_native(
                &native.saved_artifact,&[],&[2],&fresh_root,&native.transaction_digest,&suffix,
                &[3],&native.committed_receipt).unwrap();
            assert_eq!(continued.summary,cold);
            let no_input=crate::logical_input_tape::LogicalRecordRequest {binding:binding.clone(),snapshots:vec![]};
            assert_eq!(crate::runtime_surface::resume_saved_project_artifact_native(
                &continued.saved_artifact,&[2],&[3],&fresh_root,&continued.transaction_digest,&no_input).unwrap(),cold);
            assert!(crate::runtime_surface::resume_and_save_logical_project_native(
                &native.saved_artifact,&[],&[2],&fresh_root,&native.transaction_digest,&suffix,
                &[4],&native.committed_receipt).is_err(),"stale project receipt must not replace the continued checkpoint");
            assert_eq!(crate::runtime_surface::resume_saved_project_artifact_native(
                &continued.saved_artifact,&[2],&[3],&fresh_root,&continued.transaction_digest,&no_input).unwrap(),cold);
        }
        let reopened=restore_saved_runtime_payload(&checked).unwrap();
        let restored_context=check_saved_revision_context(&checked,&reopened,&[],&[2]).unwrap();
        assert_eq!(restored_context.pesi(),context.pesi());
        let mut wrong_payload=attachment(&mut envelope);
        wrong_payload.project_snapshot_json=payload.replace("1 보여주기","2 보여주기");
        assert!(restore_saved_runtime_payload(&wrong_payload).is_err());
        assert!(check_saved_revision_context(&wrong_payload,&owned,&[],&[2]).is_err());
        for bad in [format!(" {payload}"),payload.replacen("{","{\"schema\":\"duplicate\",",1),
            payload.replacen("{","{\"unknown\":true,",1)] {
            let mut damaged=attachment(&mut envelope);
            damaged.project_snapshot_json=bad;
            assert!(restore_saved_runtime_payload(&damaged).is_err());
            assert!(check_saved_revision_context(&damaged,&owned,&[],&[2]).is_err());
        }
        assert!(check_saved_revision_context(&checked,&owned,&[9],&[2]).is_err());
        assert!(check_saved_revision_context(&checked,&owned,&[],&[3]).is_err());
        for (key,family) in [("source_epoch_identity",IdentityFamily::SourceEpoch),
            ("project_semantic_configuration_identity",IdentityFamily::ProjectSemanticConfiguration),
            ("official_entry_definition_identity",IdentityFamily::Definition),
            ("dependency_resolution_identity",IdentityFamily::DependencyResolution)] {
            let mut foreign=envelope.clone(); foreign[key]=projection(family,&[91;32]).into();
            let checked=attachment(&mut foreign);
            assert!(check_saved_revision_context(&checked,&owned,&[],&[2]).is_err(),"{key}");
        }
        let mut foreign=attachment(&mut envelope);
        foreign.resource_snapshot=resources.replace("\"1\"","\"2\"");
        assert!(check_saved_revision_context(&foreign,&owned,&[],&[2]).unwrap_err()
            .starts_with("E_PROJECT_RUNTIME_RESOURCE_MISMATCH:"));
    }
    fn fixture() -> Vec<u8> {
        let envelope = b"{}\n";
        let attachment = Node::Object(ATTACHMENT.iter().zip([
            Node::Text("ddn.workbench.project_runtime_attachment.v1".into()), Node::Text("v1-core-v25".into()), Node::Text("00".repeat(32)),
            Node::Text("e30K".into()), Node::UInt(3), Node::Text(digest(envelope)),
            Node::Text("{}".into()), Node::UInt(2), Node::Text(digest(b"{}")),
            Node::Text("{}".into()), Node::UInt(2), Node::Text(digest(b"{}")),
        ]).map(|(k,v)| ((*k).into(),v)).collect());
        let project = Node::Object(vec![("schema".into(),Node::Text("seamgrim.project_snapshot.v0".into()))]);
        let payload = Node::Object(vec![("project_snapshot".into(),project.clone()),("runtime_attachment".into(),attachment.clone())]);
        let canonical = bytes(&payload).unwrap();
        bytes(&Node::Object(TOP.iter().zip([Node::Text("seamgrim.project_snapshot_artifact.v1".into()),project,attachment,Node::Text(String::from_utf8(canonical.clone()).unwrap()),Node::Text(digest(&canonical)),Node::UInt(canonical.len() as u64)]).map(|(k,v)| ((*k).into(),v)).collect())).unwrap()
    }
    fn encode_for_test(raw: &[u8]) -> String {
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut out = String::new(); let mut bits = 0u32; let mut count = 0u32;
        for b in raw { bits = (bits << 8) | u32::from(*b); count += 8;
            while count >= 6 { count -= 6; out.push(alphabet[((bits >> count) & 63) as usize] as char); }
            bits &= (1 << count) - 1;
        }
        if count > 0 { out.push(alphabet[(bits << (6-count)) as usize] as char); } out
    }
    #[test]
    fn rd686_independent_r11_fixed_vector_matches_exact_bytes() {
        // Immutable R1.1 vector identity, not a product-generated expected hash.
        let mut node: Node = serde_json::from_slice(&fixture()).unwrap();
        let Node::Object(top) = &mut node else { panic!() };
        let Node::Object(att) = &mut top[2].1 else { panic!() };
        let pesi = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let envelope = format!("{{\"schema\":\"rd631.fixture.transport-only\",\"project\":\"p\",\"parent_revision\":\"r0\",\"new_revision\":\"r1\",\"profile\":\"v1-core-v25\",\"pesi\":\"{pesi}\",\"runtime_snapshot_byte_length\":2}}\n");
        let resources = r#"{"schema":"ddn.resource_snapshot.v2","value_det":{"type":"tensor_v1","version":1}}"#;
        att[2].1 = Node::Text(pesi.into()); att[3].1 = Node::Text(encode_for_test(envelope.as_bytes()));
        att[4].1 = Node::UInt(envelope.len() as u64); att[5].1 = Node::Text(digest(envelope.as_bytes()));
        att[9].1 = Node::Text(resources.into()); att[10].1 = Node::UInt(resources.len() as u64); att[11].1 = Node::Text(digest(resources.as_bytes()));
        let payload = bytes(&Node::Object(vec![top[1].clone(),top[2].clone()])).unwrap();
        top[3].1 = Node::Text(String::from_utf8(payload.clone()).unwrap()); top[4].1 = Node::Text(digest(&payload)); top[5].1 = Node::UInt(payload.len() as u64);
        let raw = bytes(&node).unwrap();
        assert_eq!(raw.len(),2606);
        assert_eq!(digest(&raw),"6ced78cf01d25e3137d5af874413fbb8acf913426591684cc7e96c4625b25a1a");
        assert_eq!(check_bytes(&raw).unwrap().revision_envelope,envelope.as_bytes());
        // This transport-only fixture is deliberately NOT RD-631 admitted.
    }
    #[test]
    fn rd686_nested_duplicate_escapes_numbers_and_bounds_are_strict() {
        let raw = String::from_utf8(fixture()).unwrap();
        for bad in [raw.replacen("seamgrim.project_snapshot.v0",r"seamgrim.project_snapshot.v\u0030",1),raw.replacen("\"project_snapshot\":{","\"project_snapshot\":{\"schema\":null,",1),raw.replacen("\"runtime_state_byte_length\":2","\"runtime_state_byte_length\":-0",1),raw.replacen("\"runtime_state_byte_length\":2","\"runtime_state_byte_length\":2e0",1)] {
            assert!(check_bytes(bad.as_bytes()).is_err());
        }
        for n in [0,1,2,3,255,256,257,ENVELOPE_MAX] {
            let data = vec![0xab;n]; let encoded = encode_for_test(&data);
            if n == 0 { assert!(decode_envelope(&encoded).is_err()); }
            else { assert_eq!(decode_envelope(&encoded).unwrap(),data); }
        }
        assert!(decode_envelope(&encode_for_test(&vec![0xab;ENVELOPE_MAX+1])).is_err());
    }
    #[test]
    fn rd686_byte_gate_preserves_raw_envelope_without_semantic_admission() {
        let raw = fixture(); let checked = check_bytes(&raw).unwrap();
        assert_eq!(checked.revision_envelope,b"{}\n");
        assert_eq!(checked.source_profile,"v1-core-v25"); assert_eq!(checked.pesi_hex,"00".repeat(32));
        assert_eq!(checked.runtime_state,"{}"); assert_eq!(checked.resource_snapshot,"{}");
        assert_eq!(checked.project_snapshot_json,r#"{"schema":"seamgrim.project_snapshot.v0"}"#);
        assert_eq!(check_bytes(&raw).unwrap().revision_envelope,checked.revision_envelope);
    }
    #[test]
    fn rd686_raw_mutations_fail_before_candidate_or_publication() {
        let raw = String::from_utf8(fixture()).unwrap();
        for bad in [format!(" {raw}"),format!("{raw}\n"),format!("\u{feff}{raw}"),raw.replacen("{","{ ",1),raw.replacen("\"schema\":","\"schema\":null,\"schema\":",1),raw.replacen("e30K","e30K=",1),raw.replacen("\"runtime_state_byte_length\":2","\"runtime_state_byte_length\":3",1),raw.replacen("\"runtime_state_byte_length\":2","\"runtime_state_byte_length\":2.0",1),raw.replacen("v1-core-v25","v1-core-v24",1)] {
            assert!(check_bytes(bad.as_bytes()).is_err(),"accepted {bad}");
        }
        assert!(check_bytes(&vec![b' ';OUTER_MAX+1]).unwrap_err().contains("TOO_LARGE"));
        for invalid in ["e31", "e30=", "e30/", "A", ""] { assert!(decode_envelope(invalid).is_err()); }
    }
}
