//! Actual Native finish producer only. A decoded header is never this owner.
use super::*;
use crate::input_tape_v3::PreparedNativeLogicalFinishV1;
use crate::replay_evidence::{PlannedReplayControlRowV1,PendingReplayLifecycleV1};
use crate::replay_evidence::finish_recovery::{encode_header,RecordHeaderV1,RecordKindV1,CHUNK_BYTES};

/// Owns the one actual randomized ROW/Pending target created for this finish.
/// Not Clone, Deserialize, from-path, Ready, or a cold recovery permit.
pub(crate) struct NativeFinishRecoveryIntentV1 {
    pub(super) row:PlannedReplayControlRowV1,
    pending:Option<PendingReplayLifecycleV1>,
    header:Vec<u8>,payload:Vec<u8>,record_digest:[u8;32],
}
impl NativeFinishRecoveryIntentV1 {
    pub(crate) fn header_bytes(&self)->&[u8]{&self.header}
    pub(crate) fn payload_bytes(&self)->&[u8]{&self.payload}
    pub(crate) fn record_digest(&self)->&[u8;32]{&self.record_digest}
    pub(crate) fn row(&self)->&PlannedReplayControlRowV1{&self.row}
    pub(crate) fn pending(&self)->Option<&PendingReplayLifecycleV1>{self.pending.as_ref()}
    pub(super) fn take_pending(&mut self)->Result<PendingReplayLifecycleV1,String>{
        self.pending.take().ok_or_else(||"E_NATIVE_FINISH_PENDING_CONSUMED: 원 종료 저장 대상은 한 번만 소비합니다".into())
    }
}

impl NativeFinishCheckpointPlanV1 {
    pub(super) fn retain_original_failure(&self,adapter:&mut SqliteAdapterV1,
        intent:&NativeFinishRecoveryIntentV1,
        witness:&crate::snapshot_v2_durable_transaction::native::DurableFinishIntentWitnessV1,
        cause:&crate::snapshot_v2_durable_transaction::native::CapturedReplayMetadataFailureV1)->String{
        use crate::snapshot_v2_durable_transaction::native::ReplayLifecycleWriteFailureV1;
        let describe=|error|match error{
            ReplayLifecycleWriteFailureV1::Before(cause)=>format!("Before({cause:?})"),
            ReplayLifecycleWriteFailureV1::Uncertain(cause)=>format!("Uncertain({cause:?})"),
        };
        let retained=(||->Result<(),String>{
            let pending=self.data.scope.persist_finish_failure_capture(adapter,intent,witness,cause).map_err(describe)?;
            adapter.append_finish_recovery_failure(&self.data.scope,intent,cause,&pending)
                .map_err(|error|format!("SQL({error:?})"))?;
            self.data.scope.persist_finish_failure_witness(adapter,intent,cause,&pending).map_err(describe)
        })();
        match retained{
            Ok(())=>cause.original_message().to_owned(),
            Err(storage)=>format!("{}; finish_recovery={storage}",cause.original_message()),
        }
    }

    /// Called by this actual plan owner before Pending. It performs no SQL
    /// writes, lifecycle write, Finished publication or world advance. Persistence and
    /// witness gates must finish before consuming its original row/pending.
    pub(super) fn prepare_recovery_intent(&self,adapter:&SqliteAdapterV1,
        candidate:&PreparedNativeLogicalFinishV1)->Result<NativeFinishRecoveryIntentV1,String>{
        let data=&self.data;
        if data.control_ordinal==0 || data.recording_state!=1 || data.end_owner_kind>1
            || data.previous_lifecycle.is_none() || data.reopen.is_some()
            || !self.generation.matches(&candidate.generation())
            || self.closed_count!=candidate.closed_count()
            || self.tape_hash!=<[u8;32]>::from(Sha256::digest(candidate.tape_bytes())) {
            return Err("E_NATIVE_FINISH_STALE_CANDIDATE: 복구용 원래 종료 후보가 다릅니다".into());
        }
        // The original producer bounds the response before a Pending write.
        let payload=candidate.recovery_payload_bytes()?;
        let notice=candidate.recovery_notice_payload();
        let (project,nonce,fingerprint,generation)=data.scope.identity_fields(adapter)
            .map_err(|cause|format!("E_NATIVE_FINISH_BEFORE_PENDING: 원 저장소 확인 ({cause:?})"))?;
        let row=PlannedReplayControlRowV1::from_plan(data,adapter)?;
        let pending=PendingReplayLifecycleV1::from_plan(data,&row,adapter)?;
        let ready=pending.ready_candidate_bytes()?;
        let row_bytes=row.canonical_bytes()?;
        let previous=data.previous_lifecycle.as_deref().ok_or("E_NATIVE_FINISH_CURRENT")?;
        let generation=generation.to_be_bytes();let epoch=data.recording_epoch.to_be_bytes();
        let ordinal=data.control_ordinal.to_be_bytes();let count=data.accepted_count.to_be_bytes();
        let kind=[data.end_owner_kind];let payload_digest:[u8;32]=Sha256::digest(&payload).into();
        let length=(payload.len() as u64).to_be_bytes();
        let chunks=u32::try_from(payload.len().div_ceil(CHUNK_BYTES))
            .map_err(|_|"E_NATIVE_FINISH_PAYLOAD_LIMIT")?.to_be_bytes();
        let header=encode_header(RecordKindV1::Intent,&[
            &project,&nonce,&fingerprint,&generation,&data.first_tx,&epoch,&ordinal,&data.checkpoint_tx,
            previous,pending.canonical_bytes(),&ready,&row_bytes,&data.result_digest,&data.seed_digest,
            &count,&kind,&notice,&payload_digest,&length,&chunks])?;
        let record_digest=RecordHeaderV1::parse(&header)?.check_payload(payload.chunks(CHUNK_BYTES))?;
        Ok(NativeFinishRecoveryIntentV1{row,pending:Some(pending),header,payload,record_digest})
    }
}
