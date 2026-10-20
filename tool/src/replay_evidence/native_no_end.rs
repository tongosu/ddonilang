//! Actual stopped/failed producer -> NoEnd metadata. This cannot supply a
//! normal-end proof, Native normal restore, or successful receipt replay.
use sha2::{Digest,Sha256};
use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,ReadmittedFirstSourceWorldV2,ProtectedReplayReadyV1};
use crate::unit_wire::{resumable_complete_state::NativeRecordingProgressOwnerV1,project_runtime_attachment::SourceBoundSavedRuntimeAttachment};

pub(crate) struct NativeNoEndCheckpointPlanV1 {
    data:super::ReplayCheckpointPlanDataV1,evidence:super::LogicalNoEndEvidenceV1,
}
pub(crate) struct NativeNoEndStoredV1 {
    protected:ProtectedReplayReadyV1,evidence:super::LogicalNoEndEvidenceV1,
}
impl NativeNoEndStoredV1 {
    pub(crate) fn output(&self,adapter:&SqliteAdapterV1)->Result<serde_json::Value,String>{
        let bytes=self.protected.checked_lifecycle(adapter).map_err(|e|format!("E_NATIVE_NO_END_READY: {e:?}"))?;
        let lifecycle=super::ReplayLifecycleCandidateV1::parse(bytes)?;
        if lifecycle.payload(8)!=Some(&[1][..]) || lifecycle.payload(9)!=Some(&[self.evidence.state][..])
            || lifecycle.payload(17)!=Some(&[2][..])
            || lifecycle.payload(15)!=Some(self.evidence.accepted_count.to_be_bytes().as_slice())
            || lifecycle.payload(16)!=Some(self.evidence.result.digest_bytes().as_slice()){
            return Err("E_NATIVE_NO_END_READY: 실제 중단·실패 보존 경계가 다릅니다".into());
        }
        Ok(serde_json::json!({"outcome":if self.evidence.state==3{"ABORTED"}else{"INCOMPLETE"},
            "tape_bytes":self.evidence.tape,"summary":self.evidence.summary}))
    }
}
impl NativeNoEndCheckpointPlanV1 {
    pub(crate) fn from_actual_stop(owner:&mut NativeRecordingProgressOwnerV1,adapter:&SqliteAdapterV1,
        admitted:&ReadmittedFirstSourceWorldV2,context:&SourceBoundSavedRuntimeAttachment<'_>)->Result<Self,String>{
        let lifecycle=owner.checkpoint_lifecycle(adapter)?.to_vec();
        let old_bundle=owner.previous_bundle(adapter)?.to_vec();
        let seed=owner.checkpoint_run(adapter)?.original_replay_seed()?.canonical_bytes().to_vec();
        let proof=admitted.replay_checkpoint_proof(adapter,context).map_err(|e|format!("E_NATIVE_NO_END_ANCHOR: {e:?}"))?;
        let (scope,actual_lifecycle)=proof.into_checkpoint_plan_scope(adapter).map_err(|e|format!("E_NATIVE_NO_END_CURRENT: {e:?}"))?;
        if lifecycle!=actual_lifecycle{return Err("E_NATIVE_NO_END_CURRENT: 기존 기록 경계가 바뀌었습니다".into());}
        let life=super::ReplayLifecycleCandidateV1::parse(&lifecycle)?;
        let integer=|tag|->Result<u64,String>{Ok(u64::from_be_bytes(life.payload(tag).ok_or("E_NATIVE_NO_END_CURRENT")?.try_into().map_err(|_|"E_NATIVE_NO_END_CURRENT")?))};
        let first_tx:[u8;32]=life.payload(5).ok_or("E_NATIVE_NO_END_CURRENT")?.try_into().map_err(|_|"E_NATIVE_NO_END_CURRENT")?;
        let previous_row:[u8;32]=life.payload(12).ok_or("E_NATIVE_NO_END_CURRENT")?.try_into().map_err(|_|"E_NATIVE_NO_END_CURRENT")?;
        let recording_epoch=integer(6)?;
        let control_ordinal=integer(7)?.checked_add(1).filter(|v|*v<=i64::MAX as u64).ok_or("E_REPLAY_CONTROL_LIMIT")?;
        let binding=admitted.payload().binding.clone();let checkpoint_tx=admitted.payload().digest;
        let receipt=admitted.current_receipt(adapter,context).map_err(|e|format!("E_NATIVE_NO_END_RECEIPT: {e:?}"))?;
        let old=super::ReplayBundleCandidateV1::parse(&old_bundle)?;
        if old.payload(2)!=Some(seed.as_slice()){return Err("E_NATIVE_NO_END_SEED: 원 시작 정본이 다릅니다".into());}
        let source=old.payload(1).ok_or("E_NATIVE_NO_END_SOURCE")?;
        // Stop is issued by the actual VM/controller, not a host status flag.
        let evidence=owner.stop_no_end(adapter)?;
        if evidence.accepted_count<integer(15)?{return Err("E_NATIVE_NO_END_PREFIX: 확정 기록보다 뒤로 갈 수 없습니다".into());}
        let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&evidence.accepted_count.to_be_bytes());
        let mut input=vec![1];input.extend_from_slice(&evidence.tape);
        let mut durable=vec![1];durable.extend_from_slice(&receipt);
        let mut transaction=vec![1];transaction.extend_from_slice(&checkpoint_tx);
        let mut control=vec![evidence.state];control.extend_from_slice(&recording_epoch.to_be_bytes());control.extend_from_slice(&control_ordinal.to_be_bytes());
        control.extend_from_slice(&first_tx);control.extend_from_slice(&checkpoint_tx);control.extend_from_slice(&previous_row);
        let result_digest=*evidence.result.digest_bytes();
        let bundle=super::bundle::encode_planned([source,&seed,&evidence.contract,&input,&range,
            evidence.result.canonical_bytes(),&result_digest,&evidence.receipts,&durable,&transaction,&control])?;
        let data=super::ReplayCheckpointPlanDataV1{scope,binding,prospective_receipt:receipt,first_tx,checkpoint_tx,
            control_ordinal,recording_epoch,previous_row,recording_state:evidence.state,end_owner_kind:2,
            previous_lifecycle:Some(lifecycle),bundle_digest:Sha256::digest(&bundle).into(),bundle,reopen:None,
            result_digest,seed_digest:Sha256::digest(&seed).into(),accepted_count:evidence.accepted_count};
        Ok(Self{data,evidence})
    }
    pub(crate) fn commit(self,owner:&mut NativeRecordingProgressOwnerV1,adapter:&mut SqliteAdapterV1,
        context:&SourceBoundSavedRuntimeAttachment<'_>)->Result<NativeNoEndStoredV1,String>{
        owner.validate_no_end(adapter,&self.evidence)?;
        let (row,pending)=match self.data.begin_pending(adapter){
            Ok(value)=>value,
            Err(super::ReplayPendingFailureV1::Before(cause))=>return Err(cause),
            Err(super::ReplayPendingFailureV1::Uncertain(cause))=>{owner.block_evidence(cause.clone());return Err(cause);},
        };
        let protected=(||{
            adapter.commit_replay_metadata(&self.data,&row)?;
            self.data.complete_metadata_ready(adapter,&row,pending,context)
        })().map_err(|cause|{owner.block_evidence(cause.clone());cause})?;
        Ok(NativeNoEndStoredV1{protected,evidence:self.evidence})
    }
}
