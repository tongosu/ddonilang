//! A subsequent candidate can only borrow the actual restored recording and
//! current Native world witness. It cannot replace the original first_tx/SEED.
use sha2::{Digest,Sha256};
use crate::snapshot_v2_durable_transaction::{ActualPreparedRequestV1,native::{SqliteAdapterV1,ReadmittedFirstSourceWorldV2}};
use crate::unit_wire::{resumable_complete_state::NativeRecordingProgressOwnerV1,
    project_runtime_attachment::SourceBoundSavedRuntimeAttachment};
pub(crate) struct PlannedNextReplayCheckpointV1 {data:super::ReplayCheckpointPlanDataV1}
impl std::ops::Deref for PlannedNextReplayCheckpointV1 {
    type Target=super::ReplayCheckpointPlanDataV1;
    fn deref(&self)->&Self::Target{&self.data}
}
fn rejected(cause:impl std::fmt::Debug)->String{format!("E_REPLAY_NEXT_PREPARE: {cause:?}")}
impl PlannedNextReplayCheckpointV1 {
    pub(crate) fn from_general_prepare(run:&crate::runtime_surface::VerifiedGeneralNativeRun,
        transport:&crate::snapshot_v2_durable_transaction::native::NativeReplayProgressTransportV1,
        request:&ActualPreparedRequestV1,previous:&SourceBoundSavedRuntimeAttachment<'_>,next:&SourceBoundSavedRuntimeAttachment<'_>,
        adapter:&SqliteAdapterV1,admitted:&ReadmittedFirstSourceWorldV2)->Result<Self,String>{
        transport.revalidate(adapter).map_err(rejected)?;
        let binding=request.binding();
        let expected=ActualPreparedRequestV1::from_general_continuation(run,next,&binding.expected_parent,&binding.new_revision)?;
        if expected.raw()!=request.raw()||binding.expected_parent!=admitted.payload().binding.new_revision
            ||binding.project!=admitted.payload().binding.project||binding.pesi!=admitted.payload().binding.pesi{
            return Err("E_REPLAY_GENERAL_NEXT_BINDING".into());
        }
        let proof=admitted.replay_checkpoint_proof(adapter,previous).map_err(rejected)?;
        let (scope,lifecycle)=proof.into_checkpoint_plan_scope(adapter).map_err(rejected)?;
        if lifecycle!=transport.checked_lifecycle(adapter).map_err(rejected)?{return Err("E_REPLAY_GENERAL_NEXT_CURRENT".into());}
        let parsed=super::ReplayLifecycleCandidateV1::parse(&lifecycle)?;
        let fixed=|tag|->Result<[u8;32],String>{parsed.payload(tag).ok_or("E_REPLAY_GENERAL_NEXT_LIFECYCLE")?.try_into().map_err(rejected)};
        let integer=|tag|->Result<u64,String>{Ok(u64::from_be_bytes(parsed.payload(tag).ok_or("E_REPLAY_GENERAL_NEXT_LIFECYCLE")?.try_into().map_err(rejected)?))};
        let first_tx=fixed(5)?;let previous_row=fixed(12)?;
        let control_ordinal=integer(7)?.checked_add(1).filter(|v|*v<=i64::MAX as u64).ok_or("E_REPLAY_CONTROL_LIMIT")?;
        let recording_epoch=integer(6)?;
        let old=super::ReplayBundleCandidateV1::parse(transport.bundle())?;
        let seed=old.payload(2).ok_or("E_REPLAY_GENERAL_NEXT_SEED")?;
        let progress=super::GeneralProgressEvidenceV1::capture(run.borrow_general_result_boundary()?)?;
        if run.original_general_replay_seed()?.canonical_bytes()!=seed
            ||<[u8;32]>::from(Sha256::digest(seed))!=fixed(14)?
            ||progress.accepted_count<integer(15)?||old.payload(3)!=Some(progress.input_contract.as_slice()){
            return Err("E_REPLAY_GENERAL_NEXT_PREFIX: 원 시작·입력 계약을 교체하거나 이력을 줄일 수 없습니다".into());
        }
        let checkpoint_tx=binding.transaction_digest().map_err(rejected)?;
        let profile=adapter.capability().map_err(rejected)?;
        let prospective_receipt=crate::snapshot_v2_host_bridge::encode_committed_receipt_candidate(binding,profile.as_bytes()).map_err(rejected)?;
        let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&progress.accepted_count.to_be_bytes());
        let mut receipt=vec![1];receipt.extend_from_slice(&prospective_receipt);
        let mut transaction=vec![1];transaction.extend_from_slice(&checkpoint_tx);
        let mut control=vec![0];control.extend_from_slice(&recording_epoch.to_be_bytes());control.extend_from_slice(&control_ordinal.to_be_bytes());
        control.extend_from_slice(&first_tx);control.extend_from_slice(&checkpoint_tx);control.extend_from_slice(&previous_row);
        let result_digest=*progress.result.digest_bytes();let seed_digest=Sha256::digest(seed).into();
        let bundle=super::bundle::encode_planned([old.payload(1).ok_or("E_REPLAY_GENERAL_NEXT_SOURCE")?,seed,&progress.input_contract,&progress.input_evidence,
            &range,progress.result.canonical_bytes(),&result_digest,&progress.actual_receipts,&receipt,&transaction,&control])?;
        let bundle_digest=Sha256::digest(&bundle).into();
        Ok(Self{data:super::ReplayCheckpointPlanDataV1{scope,binding:binding.clone(),prospective_receipt,first_tx,checkpoint_tx,
            control_ordinal,recording_epoch,previous_row,previous_lifecycle:Some(lifecycle),bundle,bundle_digest,reopen:None,
            recording_state:0,end_owner_kind:2,result_digest,seed_digest,accepted_count:progress.accepted_count}})
    }
    pub(crate) fn from_native_prepare(owner:&NativeRecordingProgressOwnerV1,request:&ActualPreparedRequestV1,
        previous:&SourceBoundSavedRuntimeAttachment<'_>,next:&SourceBoundSavedRuntimeAttachment<'_>,
        adapter:&SqliteAdapterV1,admitted:&ReadmittedFirstSourceWorldV2)->Result<Self,String>{
        let run=owner.checkpoint_run(adapter)?;
        run.require_native_checkpoint_provenance().map_err(rejected)?;
        let binding=request.binding();
        let expected=run.encode_checkpoint_request(&run.encode_world_frame().map_err(rejected)?,next,
            &binding.expected_parent,&binding.new_revision).map_err(rejected)?;
        if expected.as_slice()!=request.raw() || binding.expected_parent!=admitted.payload().binding.new_revision
            || binding.project!=admitted.payload().binding.project || binding.pesi!=admitted.payload().binding.pesi {
            return Err("E_REPLAY_NEXT_PREPARE: 실제 이전 저장·원문·다음 저장 계획이 다릅니다".into());
        }
        let proof=admitted.replay_checkpoint_proof(adapter,previous).map_err(rejected)?;
        let (scope,lifecycle)=proof.into_checkpoint_plan_scope(adapter).map_err(rejected)?;
        if lifecycle!=owner.checkpoint_lifecycle(adapter)? {return Err("E_REPLAY_NEXT_PREPARE: 다른 기록 생산자의 저장 경계입니다".into());}
        let parsed=super::ReplayLifecycleCandidateV1::parse(&lifecycle)?;
        let fixed=|tag|->Result<[u8;32],String>{parsed.payload(tag).ok_or("E_REPLAY_NEXT_PREPARE")?.try_into().map_err(rejected)};
        let integer=|tag|->Result<u64,String>{Ok(u64::from_be_bytes(parsed.payload(tag).ok_or("E_REPLAY_NEXT_PREPARE")?.try_into().map_err(rejected)?))};
        let first_tx=fixed(5)?;let previous_row=fixed(12)?;
        let control_ordinal=integer(7)?.checked_add(1).filter(|v|*v<=i64::MAX as u64).ok_or("E_REPLAY_CONTROL_LIMIT: 제어 순번 초과")?;
        let recording_epoch=integer(6)?;
        let old=super::ReplayBundleCandidateV1::parse(owner.previous_bundle(adapter)?)?;
        let source=old.payload(1).ok_or("E_REPLAY_NEXT_SOURCE")?;
        let seed=run.original_replay_seed()?.canonical_bytes();
        if old.payload(2)!=Some(seed) || <[u8;32]>::from(Sha256::digest(seed))!=fixed(14)? {
            return Err("E_REPLAY_NEXT_SEED: 원 시작 정본은 교체할 수 없습니다".into());
        }
        let progress=owner.progress_evidence(adapter)?;
        if progress.accepted_count<integer(15)? || old.payload(3)!=Some(progress.input_contract.as_slice()) {
            return Err("E_REPLAY_NEXT_PREFIX: 원 입력 계약·처리 이력이 후퇴했습니다".into());
        }
        let checkpoint_tx=binding.transaction_digest().map_err(rejected)?;
        let profile=adapter.capability().map_err(rejected)?;
        let prospective_receipt=crate::snapshot_v2_host_bridge::encode_committed_receipt_candidate(binding,profile.as_bytes()).map_err(rejected)?;
        let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&progress.accepted_count.to_be_bytes());
        let mut receipt=vec![1];receipt.extend_from_slice(&prospective_receipt);
        let mut transaction=vec![1];transaction.extend_from_slice(&checkpoint_tx);
        let mut control=vec![0];control.extend_from_slice(&recording_epoch.to_be_bytes());control.extend_from_slice(&control_ordinal.to_be_bytes());
        control.extend_from_slice(&first_tx);control.extend_from_slice(&checkpoint_tx);control.extend_from_slice(&previous_row);
        let result_digest=*progress.result.digest_bytes();let seed_digest=Sha256::digest(seed).into();
        let bundle=super::bundle::encode_planned([source,seed,&progress.input_contract,&progress.input_evidence,
            &range,progress.result.canonical_bytes(),&result_digest,&progress.actual_receipts,&receipt,&transaction,&control])?;
        let bundle_digest=Sha256::digest(&bundle).into();
        Ok(Self{data:super::ReplayCheckpointPlanDataV1{scope,binding:binding.clone(),prospective_receipt,first_tx,checkpoint_tx,
            control_ordinal,recording_epoch,previous_row,previous_lifecycle:Some(lifecycle),bundle,bundle_digest,reopen:None,
            recording_state:0,end_owner_kind:2,result_digest,seed_digest,accepted_count:progress.accepted_count}})
    }
}
