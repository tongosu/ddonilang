//! Private publication proof. It has no byte/JSON/default constructor.
use sha2::{Digest,Sha256};
use super::ExecutionGenerationV1;
use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,ProtectedReplayReadyV1};
use crate::unit_wire::{resumable_complete_state::NativeRecordingProgressOwnerV1,project_runtime_attachment::SourceBoundSavedRuntimeAttachment};
use crate::snapshot_v2_durable_transaction::native::ReadmittedFirstSourceWorldV2;
pub(crate) struct NativeFinishedRecordingReadyV1 {
    protected:ProtectedReplayReadyV1,generation:ExecutionGenerationV1,tape_hash:[u8;32],closed_count:u64,
}
impl NativeFinishedRecordingReadyV1 {
    pub(crate) fn check_cached(&self,adapter:&SqliteAdapterV1,session:&crate::logical_input_tape::LogicalInputSession,bytes:&[u8])->Result<(),String>{
        let raw=self.protected.checked_lifecycle(adapter).map_err(|e|format!("E_NATIVE_FINISH_READY: {e:?}"))?;
        let lifecycle=super::ReplayLifecycleCandidateV1::parse(raw)?;
        if lifecycle.payload(8)!=Some(&[1][..]) || lifecycle.payload(9)!=Some(&[1][..]) || lifecycle.payload(17)!=Some(&[1][..])
            || lifecycle.payload(15)!=Some(self.closed_count.to_be_bytes().as_slice())
            || !self.generation.matches(&session.recording_generation())
            || <[u8;32]>::from(Sha256::digest(bytes))!=self.tape_hash {return Err("E_NATIVE_FINISH_READY: 실제 확정 기록과 다릅니다".into());}
        Ok(())
    }
    pub(crate) fn check_candidate(&self,adapter:&SqliteAdapterV1,session:&crate::logical_input_tape::LogicalInputSession,
        candidate:&crate::input_tape_v3::PreparedNativeLogicalFinishV1)->Result<(),String>{
        let bytes=self.protected.checked_lifecycle(adapter).map_err(|e|format!("E_NATIVE_FINISH_READY: {e:?}"))?;
        let lifecycle=super::ReplayLifecycleCandidateV1::parse(bytes)?;
        if lifecycle.payload(8)!=Some(&[1][..]) || lifecycle.payload(9)!=Some(&[1][..])
            || lifecycle.payload(17)!=Some(&[1][..]) || lifecycle.payload(15)!=Some(self.closed_count.to_be_bytes().as_slice())
            || !self.generation.matches(&session.recording_generation()) || candidate.closed_count()!=self.closed_count
            || <[u8;32]>::from(Sha256::digest(candidate.tape_bytes()))!=self.tape_hash {
            return Err("E_NATIVE_FINISH_READY: 실제 확정된 종료 생산자·기록과 다릅니다".into());
        }Ok(())
    }
}

/// Actual live-end candidate plus the current committed anchor. No world
/// request, revision, receipt or madi is manufactured for this metadata row.
pub(crate) struct NativeFinishCheckpointPlanV1 {
    data:super::ReplayCheckpointPlanDataV1,generation:ExecutionGenerationV1,tape_hash:[u8;32],closed_count:u64,
}
impl NativeFinishCheckpointPlanV1 {
    pub(crate) fn from_saved_candidate(owner:&mut NativeRecordingProgressOwnerV1,
        candidate:&crate::input_tape_v3::PreparedNativeLogicalFinishV1,adapter:&SqliteAdapterV1,
        admitted:&ReadmittedFirstSourceWorldV2,context:&SourceBoundSavedRuntimeAttachment<'_>)->Result<Self,String>{
        owner.validate_finish(adapter,candidate)?;
        let progress=owner.progress_evidence(adapter)?;
        let old_lifecycle=owner.checkpoint_lifecycle(adapter)?.to_vec();
        let proof=admitted.replay_checkpoint_proof(adapter,context).map_err(|e|format!("E_NATIVE_FINISH_ANCHOR: {e:?}"))?;
        let (scope,lifecycle)=proof.into_checkpoint_plan_scope(adapter).map_err(|e|format!("E_NATIVE_FINISH_ANCHOR: {e:?}"))?;
        if lifecycle!=old_lifecycle{return Err("E_NATIVE_FINISH_CURRENT: 다른 기록 경계입니다".into());}
        let life=super::ReplayLifecycleCandidateV1::parse(&lifecycle)?;
        let integer=|tag|->Result<u64,String>{Ok(u64::from_be_bytes(life.payload(tag).ok_or("E_NATIVE_FINISH_CURRENT")?.try_into().map_err(|_|"E_NATIVE_FINISH_CURRENT")?))};
        let first_tx:[u8;32]=life.payload(5).ok_or("E_NATIVE_FINISH_CURRENT")?.try_into().map_err(|_|"E_NATIVE_FINISH_CURRENT")?;
        let previous_row:[u8;32]=life.payload(12).ok_or("E_NATIVE_FINISH_CURRENT")?.try_into().map_err(|_|"E_NATIVE_FINISH_CURRENT")?;
        let closed_count=candidate.closed_count();
        if integer(15)?!=closed_count || progress.accepted_count!=closed_count || candidate.summary()!=&owner.summary()? {
            return Err("E_NATIVE_FINISH_CHECKPOINT_REQUIRED: 실제 저장된 마디에서만 정상 마칩니다".into());
        }
        let recording_epoch=integer(6)?;
        let control_ordinal=integer(7)?.checked_add(1).filter(|v|*v<=i64::MAX as u64).ok_or("E_REPLAY_CONTROL_LIMIT")?;
        let binding=admitted.payload().binding.clone();let checkpoint_tx=admitted.payload().digest;
        let receipt=admitted.current_receipt(adapter,context).map_err(|e|format!("E_NATIVE_FINISH_RECEIPT: {e:?}"))?;
        let old=super::ReplayBundleCandidateV1::parse(owner.previous_bundle(adapter)?)?;
        let source=old.payload(1).ok_or("E_NATIVE_FINISH_SOURCE")?;
        let seed=owner.checkpoint_run(adapter)?.original_replay_seed()?.canonical_bytes();
        if old.payload(2)!=Some(seed){return Err("E_NATIVE_FINISH_SEED: 원 시작 정본이 다릅니다".into());}
        let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&closed_count.to_be_bytes());
        let mut input=vec![1];input.extend_from_slice(candidate.tape_bytes());
        let mut durable=vec![1];durable.extend_from_slice(&receipt);
        let mut transaction=vec![1];transaction.extend_from_slice(&checkpoint_tx);
        let mut control=vec![1];control.extend_from_slice(&recording_epoch.to_be_bytes());control.extend_from_slice(&control_ordinal.to_be_bytes());
        control.extend_from_slice(&first_tx);control.extend_from_slice(&checkpoint_tx);control.extend_from_slice(&previous_row);
        let result_digest=*progress.result.digest_bytes();let seed_digest=Sha256::digest(seed).into();
        let bundle=super::bundle::encode_planned([source,seed,&progress.input_contract,&input,&range,
            progress.result.canonical_bytes(),&result_digest,&progress.actual_receipts,&durable,&transaction,&control])?;
        let bundle_digest=Sha256::digest(&bundle).into();
        let data=super::ReplayCheckpointPlanDataV1{scope,binding,prospective_receipt:receipt,first_tx,checkpoint_tx,
            control_ordinal,recording_epoch,previous_row,recording_state:1,end_owner_kind:1,previous_lifecycle:Some(lifecycle),reopen:None,
            bundle,bundle_digest,result_digest,seed_digest,accepted_count:closed_count};
        Ok(Self{data,generation:candidate.generation(),tape_hash:Sha256::digest(candidate.tape_bytes()).into(),closed_count})
    }
    pub(crate) fn commit(self,owner:&mut NativeRecordingProgressOwnerV1,adapter:&mut SqliteAdapterV1,
        context:&SourceBoundSavedRuntimeAttachment<'_>,candidate:crate::input_tape_v3::PreparedNativeLogicalFinishV1)
        ->Result<(Vec<u8>,serde_json::Value),String>{
        owner.validate_finish(adapter,&candidate)?;
        if !self.generation.matches(&candidate.generation()) || candidate.closed_count()!=self.closed_count
            || <[u8;32]>::from(Sha256::digest(candidate.tape_bytes()))!=self.tape_hash {
            return Err("E_NATIVE_FINISH_STALE_CANDIDATE: 종료 후보가 바뀌었습니다".into());
        }
        let (row,pending)=match self.data.begin_pending(adapter){
            Ok(value)=>value,
            Err(super::ReplayPendingFailureV1::Before(cause))=>return Err(format!("E_NATIVE_FINISH_BEFORE_PENDING: {cause}")),
            Err(super::ReplayPendingFailureV1::Uncertain(cause))=>{owner.block_evidence(cause.clone());return Err(cause);},
        };
        owner.block_evidence("E_NATIVE_FINISH_PENDING: 종료 기록의 내구 확인 중입니다".into());
        let committed=(||{
            adapter.commit_replay_metadata(&self.data,&row)?;
            self.data.complete_metadata_ready(adapter,&row,pending,context)
        })();
        let protected=match committed{Ok(value)=>value,Err(cause)=>{owner.block_evidence(cause.clone());return Err(cause);}};
        let proof=NativeFinishedRecordingReadyV1{protected,generation:self.generation,tape_hash:self.tape_hash,closed_count:self.closed_count};
        owner.publish_finish(adapter,candidate,proof)
    }
}
