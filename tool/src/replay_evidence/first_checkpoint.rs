//! Non-authoritative bootstrap plan from actual SourceGrant2 prepare. A
//! candidate receipt is not Committed/Ready/Finished or a restore capability.
use sha2::{Digest,Sha256};
use crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1;
use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,ReplayArchiveRootScopeV1};
use crate::unit_wire::project_runtime_attachment::{SourceBoundSavedRuntimeAttachment,source_only::PreparedSourceOnlyRevision};
use super::LogicalRecordingRunV1;
pub(crate) struct PlannedFirstReplayCheckpointV1 {
    data:super::ReplayCheckpointPlanDataV1,
}
impl std::ops::Deref for PlannedFirstReplayCheckpointV1 {
    type Target=super::ReplayCheckpointPlanDataV1;
    fn deref(&self)->&Self::Target{&self.data}
}
fn rejected(cause:impl std::fmt::Debug)->String {format!("E_REPLAY_FIRST_PREPARE: {cause:?}")}
impl PlannedFirstReplayCheckpointV1 {
    pub(crate) fn from_general_source_prepare(run:&crate::runtime_surface::GeneralSuccessfulInputPrefix,
        source:&PreparedSourceOnlyRevision,request:&ActualPreparedRequestV1,context:&SourceBoundSavedRuntimeAttachment<'_>,
        adapter:&SqliteAdapterV1,scope:ReplayArchiveRootScopeV1)->Result<Self,String>{
        let binding=request.binding();
        if run.encode_checkpoint_request(context,&binding.expected_parent,&binding.new_revision)?.as_slice()!=request.raw()
            ||binding.expected_parent!=source.revision()||&binding.pesi!=source.pesi(){
            return Err("E_REPLAY_GENERAL_FIRST: 실제 일반 실행·저장 계획·원문이 다릅니다".into());
        }
        let (project_hash,_,_,_)=scope.identity_fields(adapter).map_err(rejected)?;
        if project_hash!=<[u8;32]>::from(Sha256::digest(&binding.project)){return Err("E_REPLAY_GENERAL_FIRST_ROOT".into());}
        let profile=adapter.capability().map_err(rejected)?.to_owned();
        let prospective_receipt=crate::snapshot_v2_host_bridge::encode_committed_receipt_candidate(binding,profile.as_bytes()).map_err(rejected)?;
        let first_tx=binding.transaction_digest().map_err(rejected)?;
        let progress=super::GeneralProgressEvidenceV1::capture(run.borrow_general_result_boundary()?)?;
        let seed=run.original_general_replay_seed()?.canonical_bytes();
        if super::ReplaySeedCandidateV1::parse(seed)?.payload(9)!=Some(source.envelope()){
            return Err("E_REPLAY_GENERAL_FIRST_SEED: 원 시작과 원문 봉인이 다릅니다".into());
        }
        let source_bytes=source.artifact()?.into_bytes();
        let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&progress.accepted_count.to_be_bytes());
        let mut receipt=vec![1];receipt.extend_from_slice(&prospective_receipt);
        let mut transaction=vec![1];transaction.extend_from_slice(&first_tx);
        // A save boundary is Running, never a substitute for a live end proof.
        let mut control=vec![0];control.extend_from_slice(&0u64.to_be_bytes());control.extend_from_slice(&0u64.to_be_bytes());
        control.extend_from_slice(&first_tx);control.extend_from_slice(&first_tx);control.extend_from_slice(&[0;32]);
        let result_digest=*progress.result.digest_bytes();
        let bundle=super::bundle::encode_planned([&source_bytes,seed,&progress.input_contract,&progress.input_evidence,
            &range,progress.result.canonical_bytes(),&result_digest,&progress.actual_receipts,&receipt,&transaction,&control])?;
        Ok(Self{data:super::ReplayCheckpointPlanDataV1{scope,binding:binding.clone(),prospective_receipt,first_tx,
            checkpoint_tx:first_tx,control_ordinal:0,recording_epoch:0,previous_row:[0;32],previous_lifecycle:None,
            bundle_digest:Sha256::digest(&bundle).into(),seed_digest:Sha256::digest(seed).into(),bundle,reopen:None,
            recording_state:0,end_owner_kind:2,result_digest,accepted_count:progress.accepted_count}})
    }
    pub(crate) fn from_source_prepare(run:&LogicalRecordingRunV1,source:&PreparedSourceOnlyRevision,
        request:&ActualPreparedRequestV1,context:&SourceBoundSavedRuntimeAttachment<'_>,
        adapter:&SqliteAdapterV1,scope:ReplayArchiveRootScopeV1)->Result<Self,String> {
        run.require_native_checkpoint_provenance().map_err(rejected)?;
        if run.checkpoint_predecessor_supply().map_err(rejected)?!=[0] {
            return Err("E_REPLAY_FIRST_PREPARE: 첫 실제 저장 계획만 초기 기록을 만듭니다".into());
        }
        let binding=request.binding();
        let world={let _timing=crate::storage_observation::Span::start("first.replay_world_frame_encode");run.encode_world_frame().map_err(rejected)?};
        let expected={let _timing=crate::storage_observation::Span::start("first.replay_request_encode");run.encode_checkpoint_request(&world,context,
            &binding.expected_parent,&binding.new_revision).map_err(rejected)?};
        if expected.as_slice()!=request.raw() || binding.expected_parent!=source.revision() || &binding.pesi!=source.pesi() {
            return Err("E_REPLAY_FIRST_PREPARE: 원 실행·실제 저장 계획·원문이 다릅니다".into());
        }
        Self::finish_source_prepare(run,source,request,adapter,scope)
    }
    /// Reuse only bytes from the actual encoder and its immutable run borrow.
    /// Source/revision binding and every Native/root/replay check remain live.
    pub(crate) fn from_produced_source_prepare(
        produced:&crate::snapshot_v2_durable_transaction::PreparedLogicalRequestBorrowV1<'_>,
        source:&PreparedSourceOnlyRevision,context:&SourceBoundSavedRuntimeAttachment<'_>,
        adapter:&SqliteAdapterV1,scope:ReplayArchiveRootScopeV1)->Result<Self,String>{
        let run=produced.run();let request=produced.request();
        run.require_native_checkpoint_provenance().map_err(rejected)?;
        if run.checkpoint_predecessor_supply().map_err(rejected)?!=[0]{
            return Err("E_REPLAY_FIRST_PREPARE: 첫 실제 저장 계획만 초기 기록을 만듭니다".into());
        }
        request.check_source(context).map_err(rejected)?;
        let binding=request.binding();
        if binding.expected_parent!=source.revision()||&binding.pesi!=source.pesi(){
            return Err("E_REPLAY_FIRST_PREPARE: 원 실행·실제 저장 계획·원문이 다릅니다".into());
        }
        Self::finish_source_prepare(run,source,request,adapter,scope)
    }
    fn finish_source_prepare(run:&LogicalRecordingRunV1,source:&PreparedSourceOnlyRevision,
        request:&ActualPreparedRequestV1,adapter:&SqliteAdapterV1,scope:ReplayArchiveRootScopeV1)->Result<Self,String>{
        let binding=request.binding();
        let (project_hash,_,_,_)=scope.identity_fields(adapter).map_err(rejected)?;
        if project_hash!=<[u8;32]>::from(Sha256::digest(&binding.project)) {
            return Err("E_REPLAY_FIRST_PREPARE: 실제 Root의 작품이 다릅니다".into());
        }
        let adapter_profile=adapter.capability().map_err(rejected)?.to_owned();
        let prospective_receipt=crate::snapshot_v2_host_bridge::encode_committed_receipt_candidate(binding,adapter_profile.as_bytes()).map_err(rejected)?;
        let first_tx=binding.transaction_digest().map_err(rejected)?;
        let progress={let _timing=crate::storage_observation::Span::start("first.replay_progress_evidence");run.progress_evidence()?};
        let source_bytes=source.artifact()?.into_bytes();
        let seed=run.original_replay_seed()?.canonical_bytes();
        let parsed_seed=super::ReplaySeedCandidateV1::parse(seed)?;
        if parsed_seed.payload(9)!=Some(source.envelope()) {return Err("E_REPLAY_FIRST_PREPARE: 원 SEED와 원문 봉인이 다릅니다".into());}
        let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&progress.accepted_count.to_be_bytes());
        let mut receipt=vec![1];receipt.extend_from_slice(&prospective_receipt);
        let mut transaction=vec![1];transaction.extend_from_slice(&first_tx);
        let mut control=vec![0];control.extend_from_slice(&0u64.to_be_bytes());control.extend_from_slice(&0u64.to_be_bytes());
        control.extend_from_slice(&first_tx);control.extend_from_slice(&first_tx);control.extend_from_slice(&[0;32]);
        let result_digest=*progress.result.digest_bytes();
        let bundle=super::bundle::encode_planned([&source_bytes,seed,&progress.input_contract,&progress.input_evidence,
            &range,progress.result.canonical_bytes(),&result_digest,&progress.actual_receipts,&receipt,&transaction,&control])?;
        let bundle_digest=Sha256::digest(&bundle).into();let seed_digest=Sha256::digest(seed).into();
        Ok(Self{data:super::ReplayCheckpointPlanDataV1{scope,binding:binding.clone(),prospective_receipt,first_tx,
            checkpoint_tx:first_tx,control_ordinal:0,recording_epoch:0,previous_row:[0;32],previous_lifecycle:None,bundle,bundle_digest,reopen:None,
            recording_state:0,end_owner_kind:2,result_digest,seed_digest,accepted_count:progress.accepted_count}})
    }
}
