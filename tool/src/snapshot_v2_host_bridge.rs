//! RD-670 transport admission under the common Rust codec/transaction owner.
//! Raw request fields are comparison evidence, never semantic identity supply.
//! This gate does not publish a session, receipt or restored world.

use sha2::{Digest,Sha256};
use crate::snapshot_v2_durable_transaction::{TransactionBindingV1,
    SnapshotV2DurabilityFailureV1 as Failure};
use crate::unit_wire::{check_complete_state_frame_v1,CompleteStateFrameV1,CompleteStateValueTablesV1};

const DOMAIN:&[u8]=b"DDN-SNAPSHOT-V2-HOST-BRIDGE-REQUEST-V1\0";
const DOMAIN_V2:&[u8]=b"DDN-SNAPSHOT-V2-HOST-BRIDGE-REQUEST-V2\0";
// 2 MB complete state plus bounded identity fields and fixed transport fields.
// No adapter may relax the independently checked complete-state 2 MB ceiling.
const REQUEST_MAX:usize=2_030_000;
const REVISION_MAX:usize=1_000_000;
const REQUEST_V2_MAX:usize=REQUEST_MAX+REVISION_MAX+5;

#[derive(Debug)]
pub(crate) struct SnapshotV2RequestFrameV1<'a> {
    pub(crate) fields:[&'a [u8];13],
    pub(crate) snapshot:CompleteStateFrameV1<'a>,
    pub(crate) values:CompleteStateValueTablesV1,
    pub(crate) request_kind:u8,
    pub(crate) transaction_digest:[u8;32],
}

/// Byte/integrity-checked request only. Source/configuration/custody admission
/// must still precede prepared-session publication and every adapter call.
#[derive(Debug)]
pub(crate) struct SnapshotV2RequestFrameV2<'a> {
    pub(crate) request:SnapshotV2RequestFrameV1<'a>,
    pub(crate) revision_bytes:&'a [u8],
    pub(crate) revision:serde_json::Value,
}

/// Approved complete-state successor in the existing RD672 transport. This
/// candidate checks duplicate bytes only; it grants no adapter or resume right.
pub(crate) struct ResumableRequestCandidateV2<'a> {
    pub(crate) binding: TransactionBindingV1,
    pub(crate) layout: crate::unit_wire::ResumableCompleteStateLayoutV2<'a>,
    pub(crate) revision_bytes: &'a [u8],
    pub(crate) request_kind: u8,
    pub(crate) snapshot_bytes: &'a [u8],
}

impl ResumableRequestCandidateV2<'_> {
    pub(crate) fn encode_canonical(&self)->Result<Vec<u8>,Failure> {
        self.encode_canonical_with_catalog(None)
    }
    pub(crate) fn encode_canonical_with_catalog(&self,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<Vec<u8>,Failure> {
        let snapshot=self.layout.encode_canonical_with_catalog(catalog)?;
        if snapshot!=self.snapshot_bytes {return Err(Failure::SnapshotHashMismatch);}
        let b=&self.binding;let length=b.snapshot_length.to_be_bytes();let digest=b.transaction_digest()?;
        let atom=hash(self.layout.world.atom_evidence);let custody=hash(self.layout.world.subject_custody);
        let kind=[self.request_kind];
        let fields:[&[u8];14]=[&b.project,&b.expected_parent,&b.new_revision,&b.execution_profile,&b.pesi,
            &b.snapshot_sha256,&length,&b.revision_integrity,&atom,&custody,&digest,&snapshot,&kind,self.revision_bytes];
        let mut size=1usize;
        for field in fields {size=size.checked_add(5).and_then(|n|n.checked_add(field.len())).ok_or(Failure::LimitExceeded)?;}
        let total=DOMAIN_V2.len()+42+size;
        if total>REQUEST_V2_MAX {return Err(Failure::LimitExceeded);}
        let mut body=Vec::new();body.try_reserve_exact(size).map_err(|_|Failure::LimitExceeded)?;body.push(14);
        for (i,field) in fields.into_iter().enumerate() {
            body.push(i as u8+1);body.extend(u32::try_from(field.len()).map_err(|_|Failure::LimitExceeded)?.to_be_bytes());body.extend(field);
        }
        let mut out=Vec::new();out.try_reserve_exact(total).map_err(|_|Failure::LimitExceeded)?;
        out.extend(DOMAIN_V2);out.extend([0,2]);out.extend((body.len() as u64).to_be_bytes());out.extend(hash(&body));out.extend(body);
        check_resumable_request_candidate_v2_with_catalog(&out,&b.pesi,catalog)?;
        Ok(out)
    }
}

impl SnapshotV2RequestFrameV2<'_> {
    /// Common Rust transport writer. The caller must retain the admitted
    /// identity/source/custody owners; serializing a request grants no I/O or
    /// transaction authority. Existing envelope bytes/identities are preserved.
    pub(crate) fn encode_canonical(&self)->Result<Vec<u8>,Failure> {
        let snapshot=self.request.snapshot.encode_canonical()?;
        if snapshot!=self.request.fields[11] {return Err(Failure::SnapshotHashMismatch);}
        let fields=self.request.fields.iter().copied().chain(std::iter::once(self.revision_bytes));
        let mut length=1usize;
        for field in fields.clone() {
            u32::try_from(field.len()).map_err(|_|Failure::LimitExceeded)?;
            length=length.checked_add(5).and_then(|n|n.checked_add(field.len())).ok_or(Failure::LimitExceeded)?;
        }
        let total=DOMAIN_V2.len().checked_add(42).and_then(|n|n.checked_add(length)).ok_or(Failure::LimitExceeded)?;
        if total>REQUEST_V2_MAX {return Err(Failure::LimitExceeded);}
        let mut body=Vec::new();body.try_reserve_exact(length).map_err(|_|Failure::LimitExceeded)?;
        body.push(14);
        for (index,field) in fields.enumerate() {
            body.push(index as u8+1);
            body.extend_from_slice(&(field.len() as u32).to_be_bytes());body.extend_from_slice(field);
        }
        let mut raw=Vec::new();raw.try_reserve_exact(total).map_err(|_|Failure::LimitExceeded)?;
        raw.extend_from_slice(DOMAIN_V2);raw.extend_from_slice(&[0,2]);
        raw.extend_from_slice(&(body.len() as u64).to_be_bytes());raw.extend_from_slice(&hash(&body));
        raw.extend_from_slice(&body);
        // Re-admit duplicated lengths/digests and the exact RD-631 envelope.
        check_snapshot_v2_request_frame_v2(&raw,self.request.snapshot.pesi)?;
        Ok(raw)
    }
}

struct Cursor<'a>{bytes:&'a [u8],offset:usize}
impl<'a> Cursor<'a>{
    fn take(&mut self,count:usize)->Result<&'a [u8],Failure>{
        let end=self.offset.checked_add(count).ok_or(Failure::LimitExceeded)?;
        let value=self.bytes.get(self.offset..end).ok_or(Failure::InvalidInput)?;
        self.offset=end;Ok(value)
    }
    fn field(&mut self,tag:u8)->Result<&'a [u8],Failure>{
        if self.take(1)?!=[tag]{return Err(Failure::InvalidInput);}
        let length=u32::from_be_bytes(self.take(4)?.try_into().map_err(|_|Failure::InvalidInput)?);
        let length=usize::try_from(length).map_err(|_|Failure::LimitExceeded)?;
        self.take(length)
    }
    fn finish(&self)->Result<(),Failure>{
        if self.offset==self.bytes.len(){Ok(())}else{Err(Failure::InvalidInput)}
    }
}
fn fixed32(bytes:&[u8])->Result<[u8;32],Failure>{bytes.try_into().map_err(|_|Failure::InvalidInput)}
fn hash(bytes:&[u8])->[u8;32]{Sha256::digest(bytes).into()}

/// RD-670's literal twelve-field COMMITTED payload. This is comparison
/// evidence only: parsing host bytes cannot confirm storage or issue provenance.
pub(crate) struct CommittedReceiptPayloadV1<'a> {
    fields: [&'a [u8]; 12],
}

impl CommittedReceiptPayloadV1<'_> {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn native_head(&self) -> Result<crate::snapshot_v2_durable_transaction::native::StoredHeadV1,Failure> {
        Ok(crate::snapshot_v2_durable_transaction::native::StoredHeadV1 {
            project:self.fields[1].to_vec(),revision:self.fields[3].to_vec(),
            integrity:fixed32(self.fields[8])?,digest:fixed32(self.fields[0])?,
        })
    }
    pub(crate) fn check_binding(&self, binding: &TransactionBindingV1,
        adapter_profile: &[u8]) -> Result<(), Failure> {
        let digest = binding.transaction_digest()?;
        let length = binding.snapshot_length.to_be_bytes();
        let expected: [&[u8]; 11] = [&digest, &binding.project, &binding.expected_parent,
            &binding.new_revision, &binding.pesi, &binding.execution_profile, &length,
            &binding.snapshot_sha256, &binding.revision_integrity,
            &binding.revision_integrity, adapter_profile];
        if self.fields[..11] != expected { return Err(Failure::CustodyConflict); }
        Ok(())
    }
}

pub(crate) fn check_committed_receipt_payload_v1(
    bytes: &[u8],
) -> Result<CommittedReceiptPayloadV1<'_>, Failure> {
    // Existing identity bounds, not a new receipt family or success authority.
    if bytes.len() > REQUEST_MAX { return Err(Failure::LimitExceeded); }
    let mut cursor = Cursor { bytes, offset: 0 };
    if cursor.take(1)? != [12] { return Err(Failure::InvalidInput); }
    let mut fields = [&[][..]; 12];
    for (index, field) in fields.iter_mut().enumerate() {
        *field = cursor.field(index as u8 + 1)?;
    }
    cursor.finish()?;
    for index in [1, 2, 3, 5] {
        if fields[index].len() > 4096 { return Err(Failure::LimitExceeded); }
        if index != 2 && fields[index].is_empty() { return Err(Failure::InvalidInput); }
    }
    if fields[10].is_empty() { return Err(Failure::InvalidInput); }
    for index in [0, 4, 7, 8, 9] { fixed32(fields[index])?; }
    let length = u64::from_be_bytes(fields[6].try_into().map_err(|_|Failure::InvalidInput)?);
    if length == 0 || fields[11] != [1] { return Err(Failure::InvalidInput); }
    // RD-669 R1.3: artifact identity is the existing revision integrity,
    // not a file path, adapter locator, or newly generated identity.
    if fields[9] != fields[8] { return Err(Failure::ArtifactMismatch); }
    let binding = TransactionBindingV1 {
        project: fields[1].to_vec(), expected_parent: fields[2].to_vec(),
        new_revision: fields[3].to_vec(), execution_profile: fields[5].to_vec(),
        pesi: fixed32(fields[4])?, snapshot_sha256: fixed32(fields[7])?,
        snapshot_length: length, revision_integrity: fixed32(fields[8])?,
    };
    if binding.transaction_digest()? != fixed32(fields[0])? {
        return Err(Failure::CustodyConflict);
    }
    Ok(CommittedReceiptPayloadV1 { fields })
}

/// Validate transport and all duplicated hash/length comparisons without
/// copying the snapshot or trusting host-precomputed checksums. Resource and
/// custody typed validation follows this borrowed gate before owner prepare.
pub(crate) fn check_snapshot_v2_request_frame_v1<'a>(
    raw:&'a [u8],admitted_pesi:&[u8;32],
)->Result<SnapshotV2RequestFrameV1<'a>,Failure>{
    // RD-672: historical V1 lacks the complete revision envelope. It never
    // initiates a product durability transaction or consults ambient state.
    let _=admitted_pesi;
    if raw.starts_with(DOMAIN){return Err(Failure::RevisionEnvelopeRequired);}
    Err(Failure::InvalidInput)
}

pub(crate) fn check_snapshot_v2_request_frame_v2<'a>(
    raw:&'a [u8],admitted_pesi:&[u8;32],
)->Result<SnapshotV2RequestFrameV2<'a>,Failure>{
    let (fields,revision_bytes)=decode_request_v2_fields(raw)?;
    let request=check_request_fields(fields,admitted_pesi)?;
    let revision=check_request_revision(fields,revision_bytes)?;
    Ok(SnapshotV2RequestFrameV2{request,revision_bytes,revision})
}

fn decode_request_v2_fields(raw:&[u8])->Result<([&[u8];13],&[u8]),Failure>{
    if raw.starts_with(DOMAIN){return Err(Failure::RevisionEnvelopeRequired);}
    if raw.len()>REQUEST_V2_MAX{return Err(Failure::LimitExceeded);}
    let mut outer=Cursor{bytes:raw,offset:0};
    if outer.take(DOMAIN_V2.len())?!=DOMAIN_V2||outer.take(2)?!=[0,2]{return Err(Failure::InvalidInput);}
    let length=u64::from_be_bytes(outer.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?);
    let length=usize::try_from(length).map_err(|_|Failure::LimitExceeded)?;
    if length>REQUEST_V2_MAX{return Err(Failure::LimitExceeded);}
    let digest=fixed32(outer.take(32)?)?;
    let body=outer.take(length)?;outer.finish()?;
    if hash(body)!=digest{return Err(Failure::InvalidInput);}
    let mut cursor=Cursor{bytes:body,offset:0};
    let count=cursor.take(1)?[0];
    if count==13{return Err(Failure::RevisionEnvelopeRequired);}
    if count!=14{return Err(Failure::InvalidInput);}
    let mut fields=[&[][..];13];
    for (index,field) in fields.iter_mut().enumerate(){*field=cursor.field(index as u8+1)?;}
    let revision_bytes=cursor.field(14)?;
    cursor.finish()?;
    if revision_bytes.is_empty(){return Err(Failure::RevisionEnvelopeRequired);}
    if revision_bytes.len()>REVISION_MAX{return Err(Failure::LimitExceeded);}
    Ok((fields,revision_bytes))
}

fn check_request_revision(fields:[&[u8];13],revision_bytes:&[u8])->Result<serde_json::Value,Failure>{
    use crate::project_revision_envelope::{verify_rd631_canonical_envelope_bytes,
        verify_rd631_envelope_structure,verify_self_empty_digest};
    let revision=verify_rd631_canonical_envelope_bytes(revision_bytes)?;
    verify_rd631_envelope_structure(&revision)?;
    let integrity=verify_self_empty_digest(&revision,"revision_integrity_sha256",Failure::RevisionCommitFailed)?;
    if integrity!=fixed32(fields[7])?{return Err(Failure::RevisionCommitFailed);}
    verify_self_empty_digest(&revision["source_manifest"],"canonical_manifest_sha256",Failure::CustodyConflict)?;
    crate::project_revision_envelope_supply::verify_revision_identity_crosschecks(
        &revision,fields[0],fields[1],fields[2])?;
    Ok(revision)
}

pub(crate) fn check_resumable_request_candidate_v2<'a>(raw:&'a [u8],admitted_pesi:&[u8;32])
    ->Result<ResumableRequestCandidateV2<'a>,Failure>{
    check_resumable_request_candidate_v2_with_catalog(raw,admitted_pesi,None)
}
pub(crate) fn check_resumable_request_candidate_v2_with_catalog<'a>(raw:&'a [u8],admitted_pesi:&[u8;32],catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)
    ->Result<ResumableRequestCandidateV2<'a>,Failure>{
    let (fields,revision_bytes)=decode_request_v2_fields(raw)?;
    let binding=check_request_comparisons(fields,admitted_pesi)?;
    let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(
        fields[11],admitted_pesi)?;
    layout.world.decode_value_tables_with_catalog(catalog)?;
    if hash(layout.world.atom_evidence)!=fixed32(fields[8])?{return Err(Failure::AtomEvidenceInvalid);}
    if hash(layout.world.subject_custody)!=fixed32(fields[9])?{return Err(Failure::SubjectCustodyInvalid);}
    check_request_revision(fields,revision_bytes)?;
    // The source envelope within the capsule must be the exact transport
    // envelope, not a second independently plausible revision.
    if layout.supplies[0]!=revision_bytes{return Err(Failure::RevisionCommitFailed);}
    Ok(ResumableRequestCandidateV2{binding,layout,revision_bytes,request_kind:fields[12][0],snapshot_bytes:fields[11]})
}

/// Exact existing twelve-field receipt bytes. This is a candidate until the
/// transaction evidence owner confirms the fresh reread and checks binding.
pub(crate) fn encode_committed_receipt_candidate(
    binding:&TransactionBindingV1,adapter_profile:&[u8],
) -> Result<Vec<u8>,Failure> {
    let digest=binding.transaction_digest()?;let length=binding.snapshot_length.to_be_bytes();
    let fields:[&[u8];12]=[&digest,&binding.project,&binding.expected_parent,&binding.new_revision,
        &binding.pesi,&binding.execution_profile,&length,&binding.snapshot_sha256,
        &binding.revision_integrity,&binding.revision_integrity,adapter_profile,&[1]];
    let mut out=vec![12];
    for (i,field) in fields.iter().enumerate() {
        out.push(i as u8+1);
        out.extend(u32::try_from(field.len()).map_err(|_|Failure::LimitExceeded)?.to_be_bytes());
        out.extend(*field);
    }
    check_committed_receipt_payload_v1(&out)?.check_binding(binding,adapter_profile)?;
    Ok(out)
}

#[cfg(test)]
fn check_historical_request_bytes<'a>(raw:&'a [u8],admitted_pesi:&[u8;32])->Result<SnapshotV2RequestFrameV1<'a>,Failure>{
    if raw.len()>REQUEST_MAX{return Err(Failure::LimitExceeded);}
    let mut outer=Cursor{bytes:raw,offset:0};
    if outer.take(DOMAIN.len())?!=DOMAIN||outer.take(2)?!=[0,1]{return Err(Failure::InvalidInput);}
    let length=u64::from_be_bytes(outer.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?);
    let length=usize::try_from(length).map_err(|_|Failure::LimitExceeded)?;
    if length>REQUEST_MAX{return Err(Failure::LimitExceeded);}
    let digest=fixed32(outer.take(32)?)?;
    let body=outer.take(length)?;outer.finish()?;
    if hash(body)!=digest{return Err(Failure::InvalidInput);}
    let mut cursor=Cursor{bytes:body,offset:0};
    if cursor.take(1)?!=[13]{return Err(Failure::InvalidInput);}
    let mut fields=[&[][..];13];
    for (index,field) in fields.iter_mut().enumerate(){*field=cursor.field(index as u8+1)?;}
    cursor.finish()?;
    check_request_fields(fields,admitted_pesi)
}

fn check_request_fields<'a>(fields:[&'a [u8];13],admitted_pesi:&[u8;32])->Result<SnapshotV2RequestFrameV1<'a>,Failure>{
    let binding=check_request_comparisons(fields,admitted_pesi)?;
    let snapshot=check_complete_state_frame_v1(fields[11],admitted_pesi)?;
    if hash(snapshot.atom_evidence)!=fixed32(fields[8])?{return Err(Failure::AtomEvidenceInvalid);}
    if hash(snapshot.subject_custody)!=fixed32(fields[9])?{return Err(Failure::SubjectCustodyInvalid);}
    let transaction_digest=binding.transaction_digest()?;
    let values=snapshot.decode_value_tables()?;
    Ok(SnapshotV2RequestFrameV1{fields,snapshot,values,request_kind:fields[12][0],transaction_digest})
}

fn check_request_comparisons(fields:[&[u8];13],admitted_pesi:&[u8;32])->Result<TransactionBindingV1,Failure>{
    for index in [0,1,2,3]{
        if fields[index].len()>4096{return Err(Failure::LimitExceeded);}
        if index!=1&&fields[index].is_empty(){return Err(Failure::InvalidInput);}
    }
    for index in [4,5,7,8,9,10]{fixed32(fields[index])?;}
    if fields[6].len()!=8||fields[12].len()!=1||!matches!(fields[12][0],1..=3){return Err(Failure::InvalidInput);}
    let snapshot_length=u64::from_be_bytes(fields[6].try_into().map_err(|_|Failure::InvalidInput)?);
    if snapshot_length!=fields[11].len() as u64||hash(fields[11])!=fixed32(fields[5])?{
        return Err(Failure::SnapshotHashMismatch);
    }
    if fields[4]!=admitted_pesi{return Err(Failure::ForeignPesi);}
    let binding=TransactionBindingV1{project:fields[0].to_vec(),expected_parent:fields[1].to_vec(),
        new_revision:fields[2].to_vec(),execution_profile:fields[3].to_vec(),
        pesi:*admitted_pesi,snapshot_sha256:fixed32(fields[5])?,snapshot_length,
        revision_integrity:fixed32(fields[7])?};
    let transaction_digest=binding.transaction_digest()?;
    if transaction_digest!=fixed32(fields[10])?{return Err(Failure::CustodyConflict);}
    Ok(binding)
}

#[cfg(test)]
mod tests{
    fn receipt_fixture(binding: &TransactionBindingV1, adapter: &[u8]) -> Vec<u8> {
        let digest = binding.transaction_digest().unwrap();
        let length = binding.snapshot_length.to_be_bytes();
        let fields: [&[u8]; 12] = [&digest, &binding.project, &binding.expected_parent,
            &binding.new_revision, &binding.pesi, &binding.execution_profile, &length,
            &binding.snapshot_sha256, &binding.revision_integrity,
            &binding.revision_integrity, adapter, &[1]];
        let mut payload = vec![12];
        for (index, field) in fields.iter().enumerate() {
            payload.push(index as u8 + 1);
            payload.extend_from_slice(&(field.len() as u32).to_be_bytes());
            payload.extend_from_slice(field);
        }
        payload
    }

    #[test]
    fn rd670_committed_payload_is_comparison_not_storage_confirmation() {
        use crate::snapshot_v2_durable_transaction::TransactionEvidenceV1;
        let snapshot = b"actual bytes";
        let binding = TransactionBindingV1 { project: b"project".to_vec(),
            expected_parent: Vec::new(), new_revision: b"revision".to_vec(),
            execution_profile: b"profile".to_vec(), pesi: [7;32],
            snapshot_sha256: hash(snapshot), snapshot_length: snapshot.len() as u64,
            revision_integrity: [8;32] };
        let payload = receipt_fixture(&binding, b"adapter");
        let parsed = check_committed_receipt_payload_v1(&payload).unwrap();
        parsed.check_binding(&binding, b"adapter").unwrap();
        assert!(parsed.check_binding(&binding, b"foreign-adapter").is_err());
        let mut evidence = TransactionEvidenceV1::prepared(binding.clone()).unwrap();
        // Valid self-consistent bytes never bypass the durable state graph.
        assert!(evidence.check_committed_receipt_evidence(&payload, b"adapter").is_err());
        evidence.synced().unwrap(); evidence.verified(snapshot, snapshot).unwrap();
        evidence.head_changed().unwrap();
        assert!(evidence.check_committed_receipt_evidence(&payload, b"adapter").is_err());
        evidence.container_confirmation(Ok(())).unwrap();
        evidence.committed_chain(&binding, snapshot, snapshot).unwrap();
        evidence.check_committed_receipt_evidence(&payload, b"adapter").unwrap();
        // Each comparison field is protected, including the separate adapter.
        for index in [0,1,3,4,5,6,7,8,9,10,11] {
            let mut bad = payload.clone();
            let offset = parsed.fields[index].as_ptr() as usize - payload.as_ptr() as usize;
            bad[offset] ^= 1;
            assert!(evidence.check_committed_receipt_evidence(&bad,b"adapter").is_err());
        }
        let mut foreign = binding.clone(); foreign.expected_parent = b"parent".to_vec();
        let foreign_payload = receipt_fixture(&foreign,b"adapter");
        assert!(check_committed_receipt_payload_v1(&foreign_payload).is_ok());
        assert!(evidence.check_committed_receipt_evidence(&foreign_payload,b"adapter").is_err());
        for prefix in 0..payload.len() {
            assert!(check_committed_receipt_payload_v1(&payload[..prefix]).is_err());
        }
        let mut suffix = payload.clone(); suffix.push(0);
        assert!(check_committed_receipt_payload_v1(&suffix).is_err());
        let mut reordered = payload; reordered[1] = 2;
        assert!(check_committed_receipt_payload_v1(&reordered).is_err());
    }
    use super::*;
    fn fixture()->Vec<u8>{
        fixture_with_resources(&[0u8;8])
    }
    fn fixture_with_resources(resources:&[u8])->Vec<u8>{
        let mut snapshot=b"DDN-SNAPSHOT-V2-COMPLETE-STATE-V1\0".to_vec();snapshot.extend([0,1,8]);
        let pesi=[7u8;32];let empty=[0u8;8];let tick=3u64.to_be_bytes();
        for (index,value) in [&pesi[..],resources,&empty,&tick,&empty,&[][..],&[][..],&[0][..]].iter().enumerate(){
            snapshot.push(index as u8+1);snapshot.extend((value.len() as u64).to_be_bytes());snapshot.extend(*value);
        }
        let binding=TransactionBindingV1{project:b"project".to_vec(),expected_parent:vec![],new_revision:b"revision".to_vec(),
            execution_profile:b"profile".to_vec(),pesi,snapshot_sha256:hash(&snapshot),snapshot_length:snapshot.len() as u64,
            revision_integrity:[8;32]};
        let length=binding.snapshot_length.to_be_bytes();let empty_digest=hash(&[]);let tx=binding.transaction_digest().unwrap();
        let fields=[&binding.project[..],&binding.expected_parent,&binding.new_revision,&binding.execution_profile,
            &pesi,&binding.snapshot_sha256,&length,&binding.revision_integrity,&empty_digest,&empty_digest,&tx,&snapshot,&[1][..]];
        let mut body=vec![13];for (index,value) in fields.iter().enumerate(){body.push(index as u8+1);body.extend((value.len() as u32).to_be_bytes());body.extend(*value);}
        wrap(&body)
    }
    fn wrap(body:&[u8])->Vec<u8>{
        let mut raw=DOMAIN.to_vec();raw.extend([0,1]);raw.extend((body.len() as u64).to_be_bytes());raw.extend(hash(body));raw.extend(body);raw
    }
    #[test]
    fn rd670_request_checks_all_comparison_fields_without_receipt(){
        let raw=fixture();let request=check_historical_request_bytes(&raw,&[7;32]).unwrap();
        assert_eq!(request.request_kind,1);assert_eq!(request.snapshot.next_tick,3);
        assert!(request.values.resources.is_empty());assert!(request.values.parameters.is_empty());
        assert_eq!(request.fields[0],b"project");assert_eq!(request.transaction_digest,fixed32(request.fields[10]).unwrap());
        assert!(matches!(check_historical_request_bytes(&raw,&[9;32]),Err(Failure::ForeignPesi)));
        let offset=DOMAIN.len()+2+8+32;
        for index in [5,8,9,10]{
            let mut body=raw[offset..].to_vec();let request=check_historical_request_bytes(&raw,&[7;32]).unwrap();
            let field_offset=request.fields[index].as_ptr() as usize-raw.as_ptr() as usize-offset;
            body[field_offset]^=1;assert!(check_historical_request_bytes(&wrap(&body),&[7;32]).is_err());
        }
        let mut bad=raw.clone();bad.push(0);assert!(check_historical_request_bytes(&bad,&[7;32]).is_err());
        let mut bad=raw;*bad.last_mut().unwrap()^=1;assert!(check_historical_request_bytes(&bad,&[7;32]).is_err());
    }
    #[test]
    fn rd670_request_rejects_invalid_value_even_with_recomputed_transport_hashes(){
        let mut table=1u64.to_be_bytes().to_vec();table.extend(1u32.to_be_bytes());table.push(b'a');
        table.extend(2u64.to_be_bytes());table.extend([1,1]);
        let raw=fixture_with_resources(&table);
        let request=check_historical_request_bytes(&raw,&[7;32]).unwrap();
        assert_eq!(request.values.resources["a"],ddonirang_core::platform::ResourceValue::Bool(true));
        *table.last_mut().unwrap()=2;
        // Fixture recomputes snapshot digest, transaction digest and transport
        // checksum: integrity matches, but typed Boolean still must fail.
        assert!(matches!(check_historical_request_bytes(&fixture_with_resources(&table),&[7;32]),Err(Failure::InvalidInput)));
    }

    #[test]
    fn rd672_historical_v1_never_opens_a_durability_session(){
        let raw=fixture();
        assert!(matches!(check_snapshot_v2_request_frame_v1(&raw,&[7;32]),Err(Failure::RevisionEnvelopeRequired)));
        assert!(matches!(check_snapshot_v2_request_frame_v2(&raw,&[7;32]),Err(Failure::RevisionEnvelopeRequired)));
    }

    fn v2(envelope:&[u8],integrity:[u8;32])->Vec<u8>{
        let raw=fixture();let request=check_historical_request_bytes(&raw,&[7;32]).unwrap();
        let binding=TransactionBindingV1{project:b"project".to_vec(),expected_parent:vec![],new_revision:b"revision".to_vec(),
            execution_profile:b"profile".to_vec(),pesi:[7;32],snapshot_sha256:fixed32(request.fields[5]).unwrap(),
            snapshot_length:request.fields[11].len() as u64,revision_integrity:integrity};
        let tx=binding.transaction_digest().unwrap();
        let mut fields=request.fields;fields[7]=&integrity;fields[10]=&tx;
        let mut body=vec![14];
        for (index,value) in fields.iter().chain([envelope].iter()).enumerate(){
            body.push(index as u8+1);body.extend((value.len() as u32).to_be_bytes());body.extend(*value);
        }
        let mut out=DOMAIN_V2.to_vec();out.extend([0,2]);out.extend((body.len() as u64).to_be_bytes());out.extend(hash(&body));out.extend(body);out
    }

    fn envelope()->(Vec<u8>,[u8;32]){
        use crate::project_revision_envelope_supply::{CanonicalIdentityBytesV1,issue_rd631_identity_projection,IdentityFamily};
        let projection=|family,bytes:&[u8]|issue_rd631_identity_projection(family,&CanonicalIdentityBytesV1::from_owner_bytes(bytes).unwrap()).unwrap();
        let mut manifest=serde_json::json!({"schema":"ddn.virtual_project_manifest.v1","kind":"seamgrim.local_project_tree.v0",
            "root_path":"root.ddn","entry_path":"app.ddn","file_count":2,"total_utf8_bytes":0,
            "files":[{"path":"app.ddn","role":"ddn_source","byte_length":0,"sha256":hex::encode(hash(&[]))},
                {"path":"root.ddn","role":"root","byte_length":0,"sha256":hex::encode(hash(&[]))}],
            "project_graph_sha256":hex::encode([0;32]),"lock_sha256":null,"canonical_manifest_sha256":""});
        let stamp=|value:&mut serde_json::Value,key:&str|{
            value[key]="".into();let mut bytes=serde_json::to_vec(value).unwrap();bytes.push(b'\n');
            let digest=hash(&bytes);value[key]=hex::encode(digest).into();digest
        };
        stamp(&mut manifest,"canonical_manifest_sha256");
        let mut envelope=serde_json::json!({"schema":"ddn.workbench.project_snapshot_revision.v1",
            "project_identity":projection(IdentityFamily::Project,b"project"),
            "history_revision_identity":projection(IdentityFamily::HistoryRevision,b"revision"),"expected_parent_revision_identity":null,
            "source_epoch_identity":projection(IdentityFamily::SourceEpoch,b"epoch"),
            "project_semantic_configuration_identity":projection(IdentityFamily::ProjectSemanticConfiguration,b"config"),
            "official_entry_path":"app.ddn","official_entry_definition_identity":projection(IdentityFamily::Definition,b"definition"),
            "source_manifest":manifest,"dependency_resolution_identity":null,"declared_resource_members":[],"revision_integrity_sha256":""});
        let integrity=stamp(&mut envelope,"revision_integrity_sha256");
        let mut bytes=serde_json::to_vec(&envelope).unwrap();bytes.push(b'\n');(bytes,integrity)
    }

    fn resumable_request_fixture(envelope:&[u8],integrity:[u8;32],embedded:&[u8])->Vec<u8>{
        let old=v2(envelope,integrity);
        let (fields,_)=decode_request_v2_fields(&old).unwrap();
        let old_snapshot=check_complete_state_frame_v1(fields[11],&[7;32]).unwrap();
        let tick=old_snapshot.next_tick.to_be_bytes();
        let mut sections:[Vec<u8>;19]=std::array::from_fn(|_|vec![1]);
        for (index,field) in [&old_snapshot.pesi[..],old_snapshot.resources,old_snapshot.parameters,
            &tick,old_snapshot.subjects,old_snapshot.atom_evidence,old_snapshot.subject_custody,&[0][..]]
            .into_iter().enumerate(){sections[index]=field.to_vec();}
        sections[8]=embedded.to_vec();sections[18].clear();
        let encode=|sections:&[Vec<u8>;19]|{
            let mut bytes=b"DDN-SNAPSHOT-V2-COMPLETE-STATE-V2\0".to_vec();bytes.extend([0,2,19]);
            for (index,field) in sections.iter().enumerate(){bytes.push(index as u8+1);
                bytes.extend((field.len() as u64).to_be_bytes());bytes.extend(field);}
            bytes
        };
        let empty=encode(&sections);let mut digest=Sha256::new();
        digest.update(b"DDN-RESUMABLE-COMPLETE-STATE-INTEGRITY-V2\0");digest.update([0,2]);
        digest.update((empty.len() as u64).to_be_bytes());digest.update(empty);
        sections[18]=digest.finalize().to_vec();let snapshot=encode(&sections);
        let mut binding=check_request_comparisons(fields,&[7;32]).unwrap();
        binding.snapshot_sha256=hash(&snapshot);binding.snapshot_length=snapshot.len() as u64;
        let tx=binding.transaction_digest().unwrap();let length=binding.snapshot_length.to_be_bytes();
        let mut fields=fields;fields[5]=&binding.snapshot_sha256;fields[6]=&length;fields[10]=&tx;fields[11]=&snapshot;
        let mut body=vec![14];
        for (index,field) in fields.iter().copied().chain([envelope]).enumerate(){
            body.push(index as u8+1);body.extend((field.len() as u32).to_be_bytes());body.extend(field);
        }
        let mut bytes=DOMAIN_V2.to_vec();bytes.extend([0,2]);bytes.extend((body.len() as u64).to_be_bytes());
        bytes.extend(hash(&body));bytes.extend(body);bytes
    }

    #[test]
    fn successor_request_reuses_transport_and_binds_exact_embedded_revision(){
        let (envelope,integrity)=envelope();
        let raw=resumable_request_fixture(&envelope,integrity,&envelope);
        let candidate=check_resumable_request_candidate_v2(&raw,&[7;32]).unwrap();
        assert_eq!(candidate.revision_bytes,envelope);assert_eq!(candidate.layout.world.next_tick,3);
        assert_eq!(candidate.binding.revision_integrity,integrity);assert_eq!(candidate.request_kind,1);
        // These one-byte supplies intentionally are not full resume admission.
        // Existing V1 snapshot success path must never accept the successor.
        assert!(check_snapshot_v2_request_frame_v2(&raw,&[7;32]).is_err());
        let old=v2(&envelope,integrity);
        assert!(check_resumable_request_candidate_v2(&old,&[7;32]).is_err());
        assert!(check_resumable_request_candidate_v2(&raw,&[8;32]).is_err());
        // Recompute ALL checksums but substitute the embedded source envelope.
        let foreign=resumable_request_fixture(&envelope,integrity,b"{}\n");
        assert!(matches!(check_resumable_request_candidate_v2(&foreign,&[7;32]),Err(Failure::RevisionCommitFailed)));
        let mut suffix=raw.clone();suffix.push(0);assert!(check_resumable_request_candidate_v2(&suffix,&[7;32]).is_err());
        let mut truncated=raw.clone();truncated.pop();assert!(check_resumable_request_candidate_v2(&truncated,&[7;32]).is_err());
        let mut damaged=raw;*damaged.last_mut().unwrap()^=1;
        assert!(check_resumable_request_candidate_v2(&damaged,&[7;32]).is_err());
    }

    #[test]
    fn rd672_v2_admits_exact_revision_bytes_without_receipt(){
        let (bytes,integrity)=envelope();let raw=v2(&bytes,integrity);
        let checked=check_snapshot_v2_request_frame_v2(&raw,&[7;32]).unwrap();
        assert_eq!(checked.encode_canonical().unwrap(),raw,"common Rust encoder must match independently assembled transport bytes");
        assert_eq!(checked.revision_bytes,bytes);assert_eq!(checked.request.snapshot.next_tick,3);
        assert_eq!(checked.revision["official_entry_path"],"app.ddn");
        // These checks are not semantic source/configuration admission.
        let mut bom=vec![0xef,0xbb,0xbf];bom.extend(&bytes);
        assert!(matches!(check_snapshot_v2_request_frame_v2(&v2(&bom,integrity),&[7;32]),Err(Failure::InvalidInput)));
        assert!(matches!(check_snapshot_v2_request_frame_v2(&v2(&bytes,[9;32]),&[7;32]),Err(Failure::RevisionCommitFailed)));
        let mut no_lf=bytes.clone();no_lf.pop();
        assert!(matches!(check_snapshot_v2_request_frame_v2(&v2(&no_lf,integrity),&[7;32]),Err(Failure::InvalidInput)));
        assert!(matches!(check_snapshot_v2_request_frame_v2(&v2(&[],integrity),&[7;32]),Err(Failure::RevisionEnvelopeRequired)));
        assert!(matches!(check_snapshot_v2_request_frame_v2(&v2(&vec![b' ';REVISION_MAX+1],integrity),&[7;32]),Err(Failure::LimitExceeded)));
    }
    #[test]
    fn rd672_v2_encoder_rejects_changed_constituents_instead_of_repairing_hashes(){
        let (bytes,integrity)=envelope();let raw=v2(&bytes,integrity);
        let mut checked=check_snapshot_v2_request_frame_v2(&raw,&[7;32]).unwrap();
        let changed=[9;32];checked.request.fields[5]=&changed;
        assert!(matches!(checked.encode_canonical(),Err(Failure::SnapshotHashMismatch)));
        let mut checked=check_snapshot_v2_request_frame_v2(&raw,&[7;32]).unwrap();
        checked.request.fields[10]=&changed;
        assert!(matches!(checked.encode_canonical(),Err(Failure::CustodyConflict)));
        let mut checked=check_snapshot_v2_request_frame_v2(&raw,&[7;32]).unwrap();
        checked.revision_bytes=&[];
        assert!(matches!(checked.encode_canonical(),Err(Failure::RevisionEnvelopeRequired)));
    }
}
