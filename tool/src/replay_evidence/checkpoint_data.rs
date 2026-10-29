//! Shared data borrowed from actual first/next checkpoint plan owners.
//! Constructors remain in those owners; this is never decoded as authority.
use crate::snapshot_v2_durable_transaction::{TransactionBindingV1,native::{SqliteAdapterV1,ReplayArchiveRootScopeV1}};
use sha2::{Digest,Sha256};
pub(crate) struct ReplayCheckpointPlanDataV1 {
    pub(crate) scope:ReplayArchiveRootScopeV1,pub(crate) binding:TransactionBindingV1,
    pub(crate) prospective_receipt:Vec<u8>,pub(crate) first_tx:[u8;32],pub(crate) checkpoint_tx:[u8;32],
    pub(crate) control_ordinal:u64,pub(crate) recording_epoch:u64,pub(crate) previous_row:[u8;32],
    pub(crate) recording_state:u8,pub(crate) end_owner_kind:u8,
    pub(crate) previous_lifecycle:Option<Vec<u8>>,pub(crate) bundle:Vec<u8>,pub(crate) bundle_digest:[u8;32],
    pub(crate) result_digest:[u8;32],pub(crate) seed_digest:[u8;32],pub(crate) accepted_count:u64,
    pub(crate) reopen:Option<crate::unit_wire::resumable_complete_state::NewRecordAdmissionProofV1>,
}
pub(crate) enum ReplayPendingFailureV1 {Before(String),Uncertain(String)}
impl From<ReplayPendingFailureV1> for String {
    fn from(value:ReplayPendingFailureV1)->Self{match value{ReplayPendingFailureV1::Before(cause)|ReplayPendingFailureV1::Uncertain(cause)=>cause}}
}
impl ReplayCheckpointPlanDataV1 {
    pub(crate) fn from_new_record(run:&super::LogicalRecordingRunV1,
        terminal:crate::snapshot_v2_durable_transaction::native::NativeReplayTerminalTransportV1,
        proof:crate::unit_wire::resumable_complete_state::NewRecordAdmissionProofV1,adapter:&SqliteAdapterV1,
        admitted:&crate::snapshot_v2_durable_transaction::native::ReadmittedFirstSourceWorldV2,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)->Result<Self,String>{
        let progress=run.progress_evidence()?;
        let seed=run.original_replay_seed()?.canonical_bytes();
        let (scope,lifecycle,old_bundle)=terminal.into_plan_parts(adapter).map_err(|e|format!("E_NEW_RECORD_CURRENT: {e:?}"))?;
        let old=super::ReplayBundleCandidateV1::parse(&old_bundle)?;
        if old.payload(2)!=Some(seed){return Err("E_NEW_RECORD_SEED: 원 시작 자료가 바뀌었습니다".into());}
        let source=old.payload(1).ok_or("E_NEW_RECORD_SOURCE")?;
        let life=super::ReplayLifecycleCandidateV1::parse(&lifecycle)?;
        let integer=|tag|->Result<u64,String>{Ok(u64::from_be_bytes(life.payload(tag).ok_or("E_NEW_RECORD_CURRENT")?.try_into().map_err(|_|"E_NEW_RECORD_CURRENT")?))};
        let first_tx:[u8;32]=life.payload(5).ok_or("E_NEW_RECORD_CURRENT")?.try_into().map_err(|_|"E_NEW_RECORD_CURRENT")?;
        let previous_row:[u8;32]=life.payload(12).ok_or("E_NEW_RECORD_CURRENT")?.try_into().map_err(|_|"E_NEW_RECORD_CURRENT")?;
        let recording_epoch=integer(6)?.checked_add(1).filter(|v|*v<=i64::MAX as u64).ok_or("E_NEW_RECORD_EPOCH_LIMIT")?;
        let control_ordinal=integer(7)?.checked_add(1).filter(|v|*v<=i64::MAX as u64).ok_or("E_NEW_RECORD_ORDINAL_LIMIT")?;
        let checkpoint_tx=admitted.payload().digest;let binding=admitted.payload().binding.clone();
        let receipt=admitted.current_receipt(adapter,context).map_err(|e|format!("E_NEW_RECORD_RECEIPT: {e:?}"))?;
        let mut durable=vec![1];durable.extend_from_slice(&receipt);
        let mut transaction=vec![1];transaction.extend_from_slice(&checkpoint_tx);
        let mut control=vec![0];control.extend_from_slice(&recording_epoch.to_be_bytes());control.extend_from_slice(&control_ordinal.to_be_bytes());
        control.extend_from_slice(&first_tx);control.extend_from_slice(&checkpoint_tx);control.extend_from_slice(&previous_row);
        let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&progress.accepted_count.to_be_bytes());
        let result_digest=*progress.result.digest_bytes();let seed_digest=Sha256::digest(seed).into();
        let bundle=super::bundle::encode_planned([source,seed,&progress.input_contract,&progress.input_evidence,&range,
            progress.result.canonical_bytes(),&result_digest,&progress.actual_receipts,&durable,&transaction,&control])?;
        let data=Self{scope,binding,prospective_receipt:receipt,first_tx,checkpoint_tx,control_ordinal,recording_epoch,
            previous_row,recording_state:0,end_owner_kind:2,previous_lifecycle:Some(lifecycle),
            bundle_digest:Sha256::digest(&bundle).into(),bundle,result_digest,seed_digest,accepted_count:progress.accepted_count,reopen:Some(proof)};
        // F5-GA[CHECKPOINT_REOPEN]: data는 바로 위에서 reopen:Some(proof)로 만든 불변 지역값이며 그 사이 대입/호출이 없다. 검증 성공이나 재실행 권한을 미리 부여한다는 뜻은 아니다.
        data.reopen.as_ref().unwrap().check_plan(&data,adapter)?;
        Ok(data)
    }
    pub(crate) fn complete_metadata_ready(&self,adapter:&mut SqliteAdapterV1,row:&super::PlannedReplayControlRowV1,
        pending:super::PendingReplayLifecycleV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<crate::snapshot_v2_durable_transaction::native::ProtectedReplayReadyV1,String>{
        let actual=adapter.revalidate_replay_scope_current_world(&self.scope,context)
            .map_err(|e|format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 원 확정 거래 재검증 ({e:?})"))?;
        let ready=pending.verify_metadata_anchor(self,row,adapter,&actual,context)
            .map_err(|e|format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: {e}"))?;
        self.scope.write_replay_ready(adapter,&ready)
            .map_err(|e|format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 종료 Ready 보호 확인 ({e:?})"))
    }
    pub(crate) fn begin_pending(&self,adapter:&mut SqliteAdapterV1)
        ->Result<(super::PlannedReplayControlRowV1,super::PendingReplayLifecycleV1),ReplayPendingFailureV1>{
        use ReplayPendingFailureV1::{Before,Uncertain};
        use crate::snapshot_v2_durable_transaction::native::ReplayLifecycleWriteFailureV1;
        if let Some(proof)=&self.reopen{proof.check_plan(self,adapter).map_err(Before)?;}
        adapter.provision_replay_archive(&self.scope).map_err(|e|Before(format!("E_REPLAY_SCHEMA: {e:?}")))?;
        let row=super::PlannedReplayControlRowV1::from_plan(self,adapter).map_err(Before)?;
        let pending=super::PendingReplayLifecycleV1::from_plan(self,&row,adapter).map_err(Before)?;
        self.scope.write_replay_pending(adapter,&pending).map_err(|e|match e{
            ReplayLifecycleWriteFailureV1::Before(cause)=>Before(format!("E_REPLAY_PENDING_NOT_WRITTEN: {cause:?}")),
            ReplayLifecycleWriteFailureV1::Uncertain(cause)=>Uncertain(format!("E_REPLAY_EVIDENCE_BLOCKED: Pending 보호 기록 불확실 ({cause:?})")),
        })?;
        Ok((row,pending))
    }
    pub(crate) fn complete_ready(&self,adapter:&mut SqliteAdapterV1,row:&super::PlannedReplayControlRowV1,
        pending:super::PendingReplayLifecycleV1,
        retained:&crate::snapshot_v2_durable_transaction::native::RetainedFirstSourceWorldV2,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)->Result<(),String>{
        use crate::storage_observation::Span;
        let _total=Span::start("replay.ready_total");
        // Re-admit the actual committed current root and complete mixed
        // history before Ready, not only this transaction's local SQL chain.
        let full_check=(||{
            let current={let _phase=Span::start("replay.ready_fresh_history");
                adapter.revalidate_replay_scope_current_world(&self.scope,context)?};
            let actual={let _phase=Span::start("replay.ready_current_receipt");
                current.current_receipt(adapter,context)?};
            if actual!=self.prospective_receipt || current.payload().binding!=self.binding {
                return Err(crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1::CustodyConflict);
            }Ok(())
        })();
        full_check.map_err(|e|format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 원 확정 거래 전체 이력 재확인 실패 ({e:?})"))?;
        let ready={let _phase=Span::start("replay.ready_committed_and_archive");
            pending.verify_committed(self,row,adapter,retained)
                .map_err(|e|format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 원 세계 거래는 확정됐으나 재생 기록 확인이 끝나지 않았습니다 ({e})"))?};
        let _write=Span::start("replay.ready_protected_write");
        self.scope.write_replay_ready(adapter,&ready)
            .map(|_|())
            .map_err(|e|format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: Ready 보호 기록 불확실 ({e:?})"))
    }
}
