//! ROW1/LIFECYCLE1 producer data. Pending is never a Ready/restore proof.
use sha2::{Digest,Sha256};
#[cfg(windows)] use super::ReplayCheckpointPlanDataV1;
#[cfg(windows)] use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
fn encode(domain:&[u8],fields:&[&[u8]])->Result<Vec<u8>,String> {
    let length=fields.iter().try_fold(domain.len()+3,|n,p|n.checked_add(9).and_then(|n|n.checked_add(p.len())))
        .filter(|n|*n<=16384).ok_or("E_REPLAY_CONTROL_LIMIT: 제어 자료 한도 초과")?;
    let mut out=Vec::new();out.try_reserve_exact(length).map_err(|_|"E_REPLAY_CONTROL_LIMIT: 메모리 부족")?;
    out.extend_from_slice(domain);out.extend_from_slice(&1u16.to_be_bytes());out.push(fields.len() as u8);
    for (i,field) in fields.iter().enumerate(){out.push(i as u8+1);out.extend_from_slice(&(field.len() as u64).to_be_bytes());out.extend_from_slice(field);}
    Ok(out)
}
#[cfg(windows)]
pub(crate) struct PlannedReplayControlRowV1 {
    pub(crate) first_tx:[u8;32],pub(crate) control_ordinal:u64,pub(crate) recording_epoch:u64,
    pub(crate) checkpoint_tx:[u8;32],pub(crate) recording_state:u8,pub(crate) previous_row:[u8;32],
    pub(crate) bundle_digest:[u8;32],pub(crate) bundle_length:u64,pub(crate) chunk_count:u64,
    pub(crate) custody_ciphertext:Vec<u8>,pub(crate) row_digest:[u8;32],
    custody_plain_digest:[u8;32],
}
#[cfg(windows)]
impl PlannedReplayControlRowV1 {
    pub(crate) fn from_plan(plan:&ReplayCheckpointPlanDataV1,adapter:&SqliteAdapterV1)->Result<Self,String> {
        if plan.bundle.is_empty() || plan.bundle.len()>134217728 || plan.accepted_count>i64::MAX as u64 {
            return Err("E_REPLAY_CONTROL_LIMIT: 묶음·마디 한도 초과".into());
        }
        if !matches!((plan.recording_state,plan.end_owner_kind),(0,2)|(1,0)|(1,1)|(2,2)|(3,2)) {
            return Err("E_REPLAY_CONTROL_KIND: 기록 상태와 종료 증거 종류가 다릅니다".into());
        }
        let plain=super::ReplayCustodyPlainV1::from_plan(plan,adapter)?;
        let custody_plain_digest=Sha256::digest(plain.canonical_bytes()).into();
        let cipher=plan.scope.protect_replay_custody(adapter,&plain).map_err(|e|format!("E_REPLAY_CUSTODY_PROTECT: {e:?}"))?;
        let returned=plan.scope.unprotect_replay_custody(adapter,&cipher).map_err(|e|format!("E_REPLAY_CUSTODY_READBACK: {e:?}"))?;
        if returned!=plain.canonical_bytes(){return Err("E_REPLAY_CUSTODY_READBACK: 원 보호 자료와 다릅니다".into());}
        let bundle_length=plan.bundle.len() as u64;
        let chunk_count=bundle_length.checked_add(999999).ok_or("E_REPLAY_CONTROL_LIMIT")?/1000000;
        if chunk_count==0 || chunk_count>135{return Err("E_REPLAY_CONTROL_LIMIT: 조각 수 한도 초과".into());}
        let mut row=Self{first_tx:plan.first_tx,control_ordinal:plan.control_ordinal,recording_epoch:plan.recording_epoch,checkpoint_tx:plan.checkpoint_tx,
            recording_state:plan.recording_state,previous_row:plan.previous_row,bundle_digest:plan.bundle_digest,bundle_length,chunk_count,
            custody_ciphertext:cipher,row_digest:[0;32],custody_plain_digest};
        row.row_digest=Sha256::digest(row.canonical_bytes()?).into();Ok(row)
    }
    pub(crate) fn canonical_bytes(&self)->Result<Vec<u8>,String> {
        if self.control_ordinal>i64::MAX as u64 || self.recording_epoch>i64::MAX as u64
            || self.recording_state>3 || self.bundle_length==0 || self.bundle_length>134217728
            || self.chunk_count==0 || self.chunk_count>135 {
            return Err("E_REPLAY_CONTROL_LIMIT: 제어 행 범위가 다릅니다".into());
        }
        control_row_canonical_bytes([
            &self.first_tx,&self.control_ordinal.to_be_bytes(),&self.recording_epoch.to_be_bytes(),&self.checkpoint_tx,
            &[self.recording_state],&self.previous_row,&self.bundle_digest,&self.bundle_length.to_be_bytes(),
            &self.chunk_count.to_be_bytes(),&self.custody_plain_digest])
    }
}
pub(crate) fn control_row_canonical_bytes(fields:[&[u8];10])->Result<Vec<u8>,String>{
    let lengths=[32,8,8,32,1,32,32,8,8,32];
    if fields.iter().zip(lengths).any(|(value,length)|value.len()!=length)
        || fields[4][0]>3 || [1,2,7,8].into_iter().any(|i|u64::from_be_bytes(fields[i].try_into().unwrap())>i64::MAX as u64)
        || u64::from_be_bytes(fields[7].try_into().unwrap())==0 || u64::from_be_bytes(fields[7].try_into().unwrap())>134217728
        || u64::from_be_bytes(fields[8].try_into().unwrap())==0 || u64::from_be_bytes(fields[8].try_into().unwrap())>135 {
        return Err("E_REPLAY_CONTROL_FORMAT: 제어 행의 범위·길이가 다릅니다".into());
    }
    encode(b"DDN-WO23-REPLAY-CONTROL-ROW-V1\0",&fields)
}
#[cfg(windows)]
pub(crate) struct PendingReplayLifecycleV1 {bytes:Vec<u8>,previous:Option<Vec<u8>>}
#[cfg(windows)]
pub(crate) struct ReadyReplayLifecycleV1 {bytes:Vec<u8>}
#[cfg(windows)]
impl PendingReplayLifecycleV1 {
    pub(crate) fn from_plan(plan:&ReplayCheckpointPlanDataV1,row:&PlannedReplayControlRowV1,adapter:&SqliteAdapterV1)->Result<Self,String> {
        let (_,nonce,fingerprint,generation)=plan.scope.identity_fields(adapter).map_err(|e|format!("E_REPLAY_LIFECYCLE_ROOT: {e:?}"))?;
        if row.first_tx!=plan.first_tx || row.checkpoint_tx!=plan.checkpoint_tx || row.control_ordinal!=plan.control_ordinal
            || row.recording_epoch!=plan.recording_epoch || row.recording_state!=plan.recording_state || row.previous_row!=plan.previous_row
            || row.bundle_digest!=plan.bundle_digest || row.bundle_length!=plan.bundle.len() as u64 {
            return Err("E_REPLAY_LIFECYCLE_PLAN: 첫 실제 제어 행과 다릅니다".into());
        }
        let receipt_digest:[u8;32]=Sha256::digest(&plan.prospective_receipt).into();
        let bytes=encode(b"DDN-WO23-REPLAY-LIFECYCLE-V1\0",&[
            &nonce,&fingerprint,&generation.to_be_bytes(),&plan.binding.pesi,&plan.first_tx,
            &plan.recording_epoch.to_be_bytes(),&plan.control_ordinal.to_be_bytes(),&[0],&[plan.recording_state],&plan.checkpoint_tx,&receipt_digest,
            &row.row_digest,&plan.previous_row,&plan.seed_digest,&plan.accepted_count.to_be_bytes(),&plan.result_digest,&[plan.end_owner_kind]])?;
        Ok(Self{bytes,previous:plan.previous_lifecycle.clone()})
    }
    pub(crate) fn canonical_bytes(&self)->&[u8] {&self.bytes}
    pub(crate) fn previous_bytes(&self)->Option<&[u8]>{self.previous.as_deref()}
    /// Raw R2 commitment candidate only. This does not issue Ready authority;
    /// verify_metadata_anchor/verify_committed still own that transition.
    pub(crate) fn ready_candidate_bytes(&self)->Result<Vec<u8>,String>{
        let mut raw=self.bytes.clone();
        let phase=lifecycle_field_offset(&raw,8).ok_or("E_REPLAY_LIFECYCLE_FORMAT")?;
        if raw.get(phase)!=Some(&0){return Err("E_REPLAY_LIFECYCLE_FORMAT".into());}
        raw[phase]=1;ReplayLifecycleCandidateV1::parse(&raw)?;Ok(raw)
    }
    pub(crate) fn first_tx(&self)->&[u8;32] {
        lifecycle_field(&self.bytes,5).unwrap().try_into().unwrap()
    }
    /// The actual retained world owner and same-transaction child rows must
    /// both revalidate before this private Pending can become Ready.
    pub(crate) fn verify_committed(self,plan:&ReplayCheckpointPlanDataV1,
        row:&PlannedReplayControlRowV1,adapter:&SqliteAdapterV1,
        retained:&crate::snapshot_v2_durable_transaction::native::RetainedFirstSourceWorldV2)
        ->Result<ReadyReplayLifecycleV1,String> {
        let actual={let _phase=crate::storage_observation::Span::start("replay.ready_retained_receipt");
            retained.replay_committed_receipt(adapter).map_err(|e|format!("E_REPLAY_RECEIPT_CURRENT: {e:?}"))?};
        if actual!=plan.prospective_receipt || self.first_tx()!=&plan.first_tx {
            return Err("E_REPLAY_RECEIPT_MISMATCH: 후보와 실제 영수증이 다릅니다".into());
        }
        {let _phase=crate::storage_observation::Span::start("replay.ready_archive_readback");
            adapter.check_replay_archive(plan,row).map_err(|e|format!("E_REPLAY_ARCHIVE_READBACK: {e:?}"))?;}
        let mut bytes=self.bytes;
        let phase=lifecycle_field_offset(&bytes,8).ok_or("E_REPLAY_LIFECYCLE_FORMAT")?;
        bytes[phase]=1;
        Ok(ReadyReplayLifecycleV1{bytes})
    }
    pub(crate) fn verify_metadata_anchor(self,plan:&ReplayCheckpointPlanDataV1,row:&PlannedReplayControlRowV1,
        adapter:&SqliteAdapterV1,actual:&crate::snapshot_v2_durable_transaction::native::ReadmittedFirstSourceWorldV2,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<ReadyReplayLifecycleV1,String>{
        let receipt=actual.current_receipt(adapter,context).map_err(|e|format!("E_REPLAY_METADATA_ANCHOR: {e:?}"))?;
        if receipt!=plan.prospective_receipt || actual.payload().binding!=plan.binding
            || actual.payload().digest!=plan.checkpoint_tx || self.first_tx()!=&plan.first_tx {
            return Err("E_REPLAY_METADATA_ANCHOR: 실제 기존 확정 거래가 다릅니다".into());
        }
        adapter.check_replay_archive(plan,row).map_err(|e|format!("E_REPLAY_METADATA_READBACK: {e:?}"))?;
        let mut bytes=self.bytes;let phase=lifecycle_field_offset(&bytes,8).ok_or("E_REPLAY_LIFECYCLE_FORMAT")?;bytes[phase]=1;
        Ok(ReadyReplayLifecycleV1{bytes})
    }
}
#[cfg(windows)]
impl ReadyReplayLifecycleV1 {
    pub(crate) fn canonical_bytes(&self)->&[u8]{&self.bytes}
    pub(crate) fn first_tx(&self)->&[u8;32]{lifecycle_field(&self.bytes,5).unwrap().try_into().unwrap()}
    pub(crate) fn expected_pending(&self)->Vec<u8>{
        let mut bytes=self.bytes.clone();let phase=lifecycle_field_offset(&bytes,8).unwrap();bytes[phase]=0;bytes
    }
}
fn lifecycle_field_offset(bytes:&[u8],tag:u8)->Option<usize>{
    let domain=b"DDN-WO23-REPLAY-LIFECYCLE-V1\0";
    if !bytes.starts_with(domain) || bytes.get(domain.len()..domain.len()+3)!=Some(&[0,1,17][..]){return None;}
    let mut offset=domain.len()+3;
    for expected in 1..=17u8 {
        let header=bytes.get(offset..offset.checked_add(9)?)?;
        if header[0]!=expected{return None;}
        let length=usize::try_from(u64::from_be_bytes(header[1..].try_into().ok()?)).ok()?;
        offset=offset.checked_add(9)?;
        bytes.get(offset..offset.checked_add(length)?)?;
        if expected==tag{return Some(offset);}
        offset=offset.checked_add(length)?;
    }None
}
fn lifecycle_field(bytes:&[u8],tag:u8)->Option<&[u8]>{
    let offset=lifecycle_field_offset(bytes,tag)?;
    let length=usize::try_from(u64::from_be_bytes(bytes.get(offset-8..offset)?.try_into().ok()?)).ok()?;
    bytes.get(offset..offset.checked_add(length)?)
}
/// Structural data only. Protected storage and an actual Native checkpoint
/// must independently admit this; decoding never issues a Run/restore permit.
pub(crate) struct ReplayLifecycleCandidateV1<'a>{fields:[&'a [u8];17]}
impl<'a> ReplayLifecycleCandidateV1<'a>{
    pub(crate) fn parse(bytes:&'a [u8])->Result<Self,String>{
        const LENGTHS:[usize;17]=[32,32,8,32,32,8,8,1,1,32,32,32,32,32,8,32,1];
        let domain=b"DDN-WO23-REPLAY-LIFECYCLE-V1\0";
        let exact=domain.len()+3+17*9+LENGTHS.iter().sum::<usize>();
        if bytes.len()!=exact{return Err("E_REPLAY_LIFECYCLE_FORMAT: 보호 기록 길이가 다릅니다".into());}
        let mut fields=[&[][..];17];
        for (index,length) in LENGTHS.into_iter().enumerate(){
            let value=lifecycle_field(bytes,index as u8+1).ok_or("E_REPLAY_LIFECYCLE_FORMAT: 필드가 다릅니다")?;
            if value.len()!=length{return Err("E_REPLAY_LIFECYCLE_FORMAT: 필드 길이가 다릅니다".into());}
            fields[index]=value;
        }
        if fields[7][0]>1 || fields[8][0]>3 || fields[16][0]>2
            || [2,5,6,14].into_iter().any(|i|u64::from_be_bytes(fields[i].try_into().unwrap())>i64::MAX as u64) {
            return Err("E_REPLAY_LIFECYCLE_FORMAT: 상태·순번 범위가 다릅니다".into());
        }
        Ok(Self{fields})
    }
    pub(crate) fn payload(&self,tag:u8)->Option<&'a [u8]>{tag.checked_sub(1).and_then(|i|self.fields.get(i as usize)).copied()}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn b3_lifecycle_data_rejects_extra_truncated_or_out_of_range_without_authority(){
        let zero=[0u8;32];let n=0u64.to_be_bytes();
        let fields:[&[u8];17]=[&zero,&zero,&n,&zero,&zero,&n,&n,&[0],&[0],&zero,&zero,&zero,&zero,&zero,&n,&zero,&[2]];
        let original=encode(b"DDN-WO23-REPLAY-LIFECYCLE-V1\0",&fields).unwrap();
        assert!(ReplayLifecycleCandidateV1::parse(&original).is_ok());
        let mut bad=original.clone();bad.push(0);assert!(ReplayLifecycleCandidateV1::parse(&bad).is_err());
        assert!(ReplayLifecycleCandidateV1::parse(&original[..original.len()-1]).is_err());
        let mut bad=original.clone();let phase=lifecycle_field_offset(&bad,8).unwrap();bad[phase]=2;
        assert!(ReplayLifecycleCandidateV1::parse(&bad).is_err());
        let mut bad=original.clone();let ordinal=lifecycle_field_offset(&bad,7).unwrap();bad[ordinal]=0x80;
        assert!(ReplayLifecycleCandidateV1::parse(&bad).is_err());
    }
}
