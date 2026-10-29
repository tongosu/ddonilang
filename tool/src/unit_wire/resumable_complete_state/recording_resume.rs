//! Native cold recording join. Original seed/output data alone is not a Run
//! permit: actual normal restoration and fresh protected progress are required.
use super::*;
use crate::replay_evidence::{ExecutionGenerationV1,LogicalRecordingRunV1};
use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,ReadmittedFirstSourceWorldV2,NativeReplayProgressTransportV1};
use sha2::{Digest,Sha256};

/// Only the dual Native normal-restore/terminal-prefix constructor below can
/// mint this. It is not a host choice, decoded frame or ReplayOnly permission.
pub(crate) struct NewRecordAdmissionProofV1 {
    root:([u8;32],[u8;32],[u8;32],u64),binding:crate::snapshot_v2_durable_transaction::TransactionBindingV1,
    previous_hash:[u8;32],first_tx:[u8;32],checkpoint_tx:[u8;32],previous_row:[u8;32],
    epoch:u64,ordinal:u64,count:u64,seed_hash:[u8;32],result_hash:[u8;32],receipt_hash:[u8;32],
}
impl NewRecordAdmissionProofV1 {
    pub(crate) fn check_plan(&self,plan:&crate::replay_evidence::ReplayCheckpointPlanDataV1,adapter:&SqliteAdapterV1)->Result<(),String>{
        let previous=plan.previous_lifecycle.as_ref().ok_or("E_NEW_RECORD_PREVIOUS")?;
        if plan.scope.identity_fields(adapter).map_err(|e|format!("E_NEW_RECORD_ROOT: {e:?}"))?!=self.root
            || plan.binding!=self.binding || plan.recording_state!=0 || plan.end_owner_kind!=2
            || plan.first_tx!=self.first_tx || plan.checkpoint_tx!=self.checkpoint_tx || plan.previous_row!=self.previous_row
            || plan.recording_epoch!=self.epoch || plan.control_ordinal!=self.ordinal || plan.accepted_count!=self.count
            || plan.seed_digest!=self.seed_hash || plan.result_digest!=self.result_hash
            || <[u8;32]>::from(Sha256::digest(previous))!=self.previous_hash
            || <[u8;32]>::from(Sha256::digest(&plan.prospective_receipt))!=self.receipt_hash {
            return Err("E_NEW_RECORD_ADMISSION: 실제 정상 복원·원 종료 기록과 결속되지 않았습니다".into());
        }Ok(())
    }
}

pub(crate) struct CurrentRecordingProgressProofV1 {
    generation:ExecutionGenerationV1,accepted_count:u64,seed_hash:[u8;32],
}
impl CurrentRecordingProgressProofV1 {
    pub(crate) fn check_session(&self,session:&crate::logical_input_tape::LogicalInputSession)->Result<(),String>{
        let (_,frames,failure)=session.accepted_prefix_v3();
        if !self.generation.matches(&session.recording_generation()) || failure.is_some()
            || frames.len() as u64!=self.accepted_count {
            return Err("E_NATIVE_RECORDING_PROGRESS_OWNER: 실제 복원 생산자나 마디가 다릅니다".into());
        }Ok(())
    }
    pub(crate) fn check_seed(&self,bytes:&[u8])->Result<(),String>{
        if <[u8;32]>::from(Sha256::digest(bytes))!=self.seed_hash{return Err("E_NATIVE_RECORDING_PROGRESS_SEED: 원 시작 자료가 다릅니다".into());}
        Ok(())
    }
}
/// Normal continuation owner after consuming BOTH native admissions, not a
/// conversion/export operation on the read-only replay VM.
pub(crate) struct NativeRecordingProgressOwnerV1 {
    run:LogicalRecordingRunV1,transport:NativeReplayProgressTransportV1,blocked:Option<String>,
    finished:Option<crate::replay_evidence::NativeFinishedRecordingReadyV1>,
}
impl NativeRecordingProgressOwnerV1 {
    pub(crate) fn start_new_record(adapter:&mut SqliteAdapterV1,admitted:&ReadmittedFirstSourceWorldV2,
        context:&super::super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        binding:&crate::logical_input_tape::LogicalInputBinding)->Result<Self,String>{
        let mut normal=CheckpointedLogicalRunV1::restore_native_source_checkpoint(adapter,admitted,context,binding)
            .map_err(|e|format!("E_NEW_RECORD_NORMAL_RESTORE: {e:?}"))?;
        normal.require_native_checkpoint_provenance().map_err(|e|format!("E_NEW_RECORD_PROVENANCE: {e:?}"))?;
        let checkpoint=admitted.replay_checkpoint_proof(adapter,context).map_err(|e|format!("E_NEW_RECORD_ROOT: {e:?}"))?;
        let terminal=adapter.load_current_replay_terminal_transport(checkpoint).map_err(|e|format!("E_NEW_RECORD_TERMINAL_REQUIRED: {e:?}"))?;
        let count=terminal.anchor_count();
        if normal.session.accepted_prefix_v3().1.len() as u64!=count{return Err("E_NEW_RECORD_ANCHOR: 실제 저장 마디가 다릅니다".into());}
        let compared=super::readonly_replay::ReadOnlyNewRecordPrefixV1::check(terminal.bundle(),binding,count)?;
        compared.check_normal_candidate(&normal.session,context.owned().sources().entry_declaration().name())?;
        let old=crate::replay_evidence::ReplayBundleCandidateV1::parse(terminal.bundle())?;
        let seed=old.payload(2).ok_or("E_NEW_RECORD_SEED")?.to_vec();
        let progress_proof=CurrentRecordingProgressProofV1{generation:normal.session.recording_generation(),accepted_count:count,
            seed_hash:Sha256::digest(&seed).into()};
        normal.replay_seed=Some(crate::replay_evidence::ReplaySeedV1::retain_native_progress(&seed,&progress_proof)?);
        normal.session.bind_native_recording_progress(&progress_proof)?;
        let result=crate::replay_evidence::ResultPreimageV1::capture(normal.session.borrow_result_boundary()?)?;
        if result.canonical_bytes()!=compared.checked_result_bytes(){return Err("E_NEW_RECORD_RESULT: 저장 경계의 형식 있는 결과가 다릅니다".into());}
        let life=crate::replay_evidence::ReplayLifecycleCandidateV1::parse(terminal.checked_lifecycle(adapter).map_err(|e|format!("E_NEW_RECORD_CURRENT: {e:?}"))?)?;
        let integer=|tag|->Result<u64,String>{Ok(u64::from_be_bytes(life.payload(tag).ok_or("E_NEW_RECORD_CURRENT")?.try_into().map_err(|_|"E_NEW_RECORD_CURRENT")?))};
        let epoch=integer(6)?.checked_add(1).filter(|v|*v<=i64::MAX as u64).ok_or("E_NEW_RECORD_EPOCH_LIMIT")?;
        let ordinal=integer(7)?.checked_add(1).filter(|v|*v<=i64::MAX as u64).ok_or("E_NEW_RECORD_ORDINAL_LIMIT")?;
        let proof=NewRecordAdmissionProofV1{root:terminal.scope_identity(adapter).map_err(|e|format!("E_NEW_RECORD_ROOT: {e:?}"))?,
            binding:admitted.payload().binding.clone(),previous_hash:Sha256::digest(terminal.checked_lifecycle(adapter).map_err(|e|format!("E_NEW_RECORD_CURRENT: {e:?}"))?).into(),
            first_tx:life.payload(5).ok_or("E_NEW_RECORD_CURRENT")?.try_into().map_err(|_|"E_NEW_RECORD_CURRENT")?,
            checkpoint_tx:admitted.payload().digest,previous_row:life.payload(12).ok_or("E_NEW_RECORD_CURRENT")?.try_into().map_err(|_|"E_NEW_RECORD_CURRENT")?,
            epoch,ordinal,count,seed_hash:Sha256::digest(&seed).into(),result_hash:*result.digest_bytes(),
            receipt_hash:Sha256::digest(admitted.current_receipt(adapter,context).map_err(|e|format!("E_NEW_RECORD_RECEIPT: {e:?}"))?).into()};
        let staged=LogicalRecordingRunV1::from_native_progress(normal,&progress_proof)?;
        let data=crate::replay_evidence::ReplayCheckpointPlanDataV1::from_new_record(&staged,terminal,proof,adapter,admitted,context)?;
        let (row,pending)=data.begin_pending(adapter).map_err(String::from)?;
        adapter.commit_replay_metadata(&data,&row)?;
        let ready=data.complete_metadata_ready(adapter,&row,pending,context)?;
        ready.checked_lifecycle(adapter).map_err(|e|format!("E_NEW_RECORD_READY: {e:?}"))?;
        drop(staged);
        // Re-enter only through the actual normal checkpoint + newly current
        // Running proof. No VM is extracted from either read-only comparison.
        Self::restore(adapter,admitted,context,binding)
    }
    pub(crate) fn checkpoint_run(&self,adapter:&SqliteAdapterV1)->Result<&LogicalRecordingRunV1,String>{
        if let Some(cause)=&self.blocked{return Err(cause.clone());}
        self.transport.revalidate(adapter).map_err(|e|format!("E_NATIVE_RECORDING_CURRENT: {e:?}"))?;
        Ok(&self.run)
    }
    pub(crate) fn checkpoint_lifecycle(&self,adapter:&SqliteAdapterV1)->Result<&[u8],String>{
        self.transport.checked_lifecycle(adapter).map_err(|e|format!("E_NATIVE_RECORDING_CURRENT: {e:?}"))
    }
    pub(crate) fn previous_bundle(&self,adapter:&SqliteAdapterV1)->Result<&[u8],String>{
        self.checkpoint_run(adapter)?;Ok(self.transport.bundle())
    }
    pub(crate) fn restore(adapter:&SqliteAdapterV1,admitted:&ReadmittedFirstSourceWorldV2,
        context:&super::super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        binding:&crate::logical_input_tape::LogicalInputBinding)->Result<Self,String>{
        let normal=CheckpointedLogicalRunV1::restore_native_source_checkpoint(adapter,admitted,context,binding)
            .map_err(|e|format!("E_NATIVE_RECORDING_NORMAL_RESTORE: {e:?}"))?;
        Self::from_restored_checkpoint(adapter,admitted,context,binding,normal)
    }
    /// Consume the actual normal restore from this synchronous request, never
    /// a summary, cached permit or reconstructed VM. All protected replay,
    /// provenance, current-row and exact result checks below still run.
    pub(crate) fn from_restored_checkpoint(adapter:&SqliteAdapterV1,admitted:&ReadmittedFirstSourceWorldV2,
        context:&super::super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        binding:&crate::logical_input_tape::LogicalInputBinding,mut normal:CheckpointedLogicalRunV1)->Result<Self,String>{
        normal.require_native_checkpoint_provenance().map_err(|e|format!("E_NATIVE_RECORDING_PROVENANCE: {e:?}"))?;
        let checkpoint=admitted.replay_checkpoint_proof(adapter,context).map_err(|e|format!("E_NATIVE_RECORDING_ROOT: {e:?}"))?;
        let transport=adapter.load_current_replay_progress_transport(checkpoint).map_err(|e|format!("E_MISSING_REPLAY_PROGRESS: {e:?}"))?;
        let replay=ReadOnlyLogicalReplayV1::check_running_prefix(transport.bundle(),binding)?;
        replay.check_normal_recording_candidate(&normal.session,context.owned().sources().entry_declaration().name())?;
        let bundle=crate::replay_evidence::ReplayBundleCandidateV1::parse(transport.bundle())?;
        let seed=bundle.payload(2).ok_or("E_MISSING_REPLAY_SEED")?;
        let count=u64::try_from(normal.session.accepted_prefix_v3().1.len()).map_err(|_|"E_NATIVE_RECORDING_COUNT")?;
        let proof=CurrentRecordingProgressProofV1{generation:normal.session.recording_generation(),accepted_count:count,
            seed_hash:Sha256::digest(seed).into()};
        transport.revalidate(adapter).map_err(|e|format!("E_NATIVE_RECORDING_CURRENT: {e:?}"))?;
        normal.replay_seed=Some(crate::replay_evidence::ReplaySeedV1::retain_native_progress(seed,&proof)?);
        normal.session.bind_native_recording_progress(&proof)?;
        let result=crate::replay_evidence::ResultPreimageV1::capture(normal.session.borrow_result_boundary()?)?;
        if result.canonical_bytes()!=replay.checked_result_bytes(){return Err("E_NATIVE_RECORDING_RESULT: 정상 복원과 원 기록 결과가 다릅니다".into());}
        let run=LogicalRecordingRunV1::from_native_progress(normal,&proof)?;
        Ok(Self{run,transport,blocked:None,finished:None})
    }
    pub(crate) fn advance(&mut self,adapter:&SqliteAdapterV1,inputs:&[crate::logical_input_tape::LogicalSnapshot])
        ->Result<serde_json::Value,String>{
        if self.finished.is_some(){return Err("E_NATIVE_RECORDING_FINISHED: 기록이 정상 종료됐습니다".into());}
        if let Some(cause)=&self.blocked{return Err(cause.clone());}
        self.transport.revalidate(adapter).map_err(|e|format!("E_NATIVE_RECORDING_CURRENT: {e:?}"))?;
        for input in inputs{self.run.step(input)?;}
        if let Err(cause)=self.transport.revalidate(adapter){
            let cause=format!("E_NATIVE_RECORDING_EVIDENCE_BLOCKED: {cause:?}");
            self.blocked=Some(cause.clone());return Err(cause);
        }
        Ok(self.run.session().summary())
    }
    pub(crate) fn summary(&self)->Result<serde_json::Value,String>{
        if let Some(cause)=&self.blocked{return Err(cause.clone());}
        Ok(self.run.session().summary())
    }
    /// Reproduce a supplied failed attempt in this actual normal Native
    /// owner. Caller error text, decoded NoEnd and outcome flags are ignored.
    /// Accepted RAM suffixes may advance; no world/checkpoint is saved here.
    pub(crate) fn reproduce_failed_input(&mut self,adapter:&SqliteAdapterV1,
        prefix:&crate::logical_input_tape::LogicalRecordRequest,failed:&crate::logical_input_tape::LogicalSnapshot,
        expected_prefix_result:&str)->Result<(),String>{
        self.checkpoint_run(adapter)?;
        let (binding,frames,failure)=self.run.session().accepted_prefix_v3();
        let saved_count=frames.len();
        if failure.is_some()||binding!=&prefix.binding||prefix.snapshots.len()<saved_count
            ||failed.madi!=prefix.snapshots.len() as u64
            ||!frames.iter().zip(&prefix.snapshots).all(|(actual,supplied)|actual.input==*supplied){
            return Err("E_NATIVE_FAILURE_PREFIX: 현재 저장 경계·원 입력·실패 시점이 다릅니다".into());
        }
        self.advance(adapter,&prefix.snapshots[saved_count..])?;
        if self.result_identity(adapter)?!=expected_prefix_result{
            let cause="E_NATIVE_FAILURE_PREFIX_RESULT: 실패 전 원 값·형식의 결과가 다릅니다".to_owned();
            self.block_evidence(cause.clone());return Err(cause);
        }
        if self.advance(adapter,std::slice::from_ref(failed)).is_ok(){
            let cause="E_NATIVE_FAILURE_NOT_REPRODUCED: Native에서 같은 입력이 실패하지 않았습니다".to_owned();
            self.block_evidence(cause.clone());return Err(cause);
        }
        let (_,accepted,failure)=self.run.session().accepted_prefix_v3();
        if failure.is_none()||accepted.len()!=prefix.snapshots.len(){
            let cause="E_NATIVE_FAILURE_OWNER_REQUIRED: 실제 실패 생산자를 확인하지 못했습니다".to_owned();
            self.block_evidence(cause.clone());return Err(cause);
        }
        self.transport.revalidate(adapter).map_err(|cause|format!("E_NATIVE_RECORDING_CURRENT: {cause:?}"))?;
        Ok(())
    }
    pub(crate) fn result_identity(&self,adapter:&SqliteAdapterV1)->Result<String,String>{
        if self.finished.is_some(){
            self.finished_output(adapter)?;
            return crate::replay_evidence::ResultPreimageV1::capture(self.run.session().borrow_result_boundary()?).map(|v|v.digest_label());
        }
        self.progress_evidence(adapter).map(|evidence|format!("sha256:{}",hex::encode(evidence.result_digest())))
    }
    pub(crate) fn progress_evidence(&self,adapter:&SqliteAdapterV1)->Result<crate::replay_evidence::LogicalProgressEvidenceV1,String>{
        if let Some(cause)=&self.blocked{return Err(cause.clone());}
        self.transport.revalidate(adapter).map_err(|e|format!("E_NATIVE_RECORDING_CURRENT: {e:?}"))?;
        self.run.progress_evidence()
    }
    pub(crate) fn register_finish_frontend(&mut self,adapter:&SqliteAdapterV1)->Result<serde_json::Value,String>{
        self.checkpoint_run(adapter)?;self.run.register_native_finish_frontend()
    }
    pub(crate) fn prepare_finish(&mut self,adapter:&SqliteAdapterV1,pending:Option<&str>)
        ->Result<crate::input_tape_v3::PreparedNativeLogicalFinishV1,String>{
        self.checkpoint_run(adapter)?;
        let life=crate::replay_evidence::ReplayLifecycleCandidateV1::parse(self.transport.checked_lifecycle(adapter).map_err(|e|format!("E_NATIVE_FINISH_CURRENT: {e:?}"))?)?;
        let saved_count=u64::from_be_bytes(life.payload(15).ok_or("E_NATIVE_FINISH_CURRENT")?.try_into().map_err(|_|"E_NATIVE_FINISH_CURRENT")?);
        if self.run.session().accepted_prefix_v3().1.len() as u64!=saved_count {
            return Err("E_NATIVE_FINISH_CHECKPOINT_REQUIRED: 현재 마디를 먼저 실제로 저장해야 합니다".into());
        }
        self.run.prepare_native_finish(pending)
    }
    pub(crate) fn validate_finish(&mut self,adapter:&SqliteAdapterV1,candidate:&crate::input_tape_v3::PreparedNativeLogicalFinishV1)->Result<(),String>{
        self.checkpoint_run(adapter)?;self.run.validate_native_finish(candidate)
    }
    pub(crate) fn block_evidence(&mut self,cause:String){self.blocked=Some(cause);}
    pub(crate) fn publish_finish(&mut self,adapter:&SqliteAdapterV1,candidate:crate::input_tape_v3::PreparedNativeLogicalFinishV1,
        proof:crate::replay_evidence::NativeFinishedRecordingReadyV1)->Result<(Vec<u8>,serde_json::Value),String>{
        let output=self.run.publish_native_finish(candidate,&proof,adapter)?;
        self.finished=Some(proof);self.blocked=None;Ok(output)
    }
    pub(crate) fn finished_output(&self,adapter:&SqliteAdapterV1)->Result<(Vec<u8>,serde_json::Value),String>{
        let proof=self.finished.as_ref().ok_or("E_NATIVE_FINISH_NOT_READY")?;
        let output=self.run.native_finished_output()?;
        proof.check_cached(adapter,self.run.session(),&output.0)?;Ok(output)
    }
    pub(crate) fn finish_notice(&self,adapter:&SqliteAdapterV1)->Result<serde_json::Value,String>{
        self.finished_output(adapter)?;self.run.native_finish_notice()
    }
    pub(crate) fn stop_no_end(&mut self,adapter:&SqliteAdapterV1)->Result<crate::replay_evidence::LogicalNoEndEvidenceV1,String>{
        self.checkpoint_run(adapter)?;
        let evidence=self.run.stop_no_end()?;
        self.blocked=Some("E_NATIVE_RECORDING_NO_END: 실제 중단·실패 기록이며 정상 완료가 아닙니다".into());
        Ok(evidence)
    }
    pub(crate) fn validate_no_end(&self,adapter:&SqliteAdapterV1,evidence:&crate::replay_evidence::LogicalNoEndEvidenceV1)->Result<(),String>{
        if self.finished.is_some(){return Err("E_NATIVE_RECORDING_FINISHED".into());}
        self.transport.revalidate(adapter).map_err(|e|format!("E_NATIVE_NO_END_CURRENT: {e:?}"))?;
        evidence.check_session(self.run.session())
    }
}
