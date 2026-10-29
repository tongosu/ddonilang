//! Logical mode03 uses the same protected source-parent transaction as mode01.
//! Its actual snapshots, fixed seed and receipt lineage remain logical owners.
use super::*;
use crate::logical_input_tape::LogicalRecordRequest;
use crate::unit_wire::{project_runtime_attachment as attachment,resumable_complete_state::CheckpointedLogicalRunV1};

pub(crate) struct NativeLogicalSourceWorldExecution {
    session:NativeFirstSourceWorldSession,
    owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    run:crate::replay_evidence::LogicalRecordingRunV1,
}
pub(crate) struct NativeLogicalSourceWorldPrepared {
    execution:NativeLogicalSourceWorldExecution,
    artifact:String,
    request:ActualPreparedRequestV1,
    _replay_plan:crate::replay_evidence::PlannedFirstReplayCheckpointV1,
}

impl NativeFirstSourceWorldSession {
    pub(crate) fn execute_logical(self,input:&LogicalRecordRequest)->Result<NativeLogicalSourceWorldExecution,String>{
        self.permit.verify_before_world(&self.adapter).map_err(reject)?;
        let owned=self.permit.source().recorded_execution_owner()?;
        let mut run=CheckpointedLogicalRunV1::fresh(&owned,input.binding.clone(),crate::ddn_runtime::default_age_target()).map_err(reject)?;
        run.capture_initial_replay_seed(self.permit.source())?;
        let mut run=crate::replay_evidence::LogicalRecordingRunV1::begin(run)?;
        for snapshot in &input.snapshots {run.step(snapshot)?;}
        self.permit.verify_before_world(&self.adapter).map_err(reject)?;
        Ok(NativeLogicalSourceWorldExecution{session:self,owned,run})
    }
}
pub(super) fn reject(cause:impl std::fmt::Debug)->String {
    format!("E_SOURCE_LOGICAL_WORLD: 논리 입력 저장 결속 실패 ({cause:?})")
}
impl NativeLogicalSourceWorldExecution {
    pub(crate) fn check_result_identity(&self,expected:&str)->Result<(),String>{
        let actual=format!("sha256:{}",hex::encode(self.run.progress_evidence()?.result_digest()));
        if actual!=expected{return Err("E_NATIVE_SOURCE_SAVE_RESULT_MISMATCH: 원 형식 결과가 화면과 다릅니다. 세계를 저장하지 않았습니다".into());}
        Ok(())
    }
    pub(crate) fn check_state_hash(&self,expected:&str)->Result<(),String>{
        if self.run.session().summary()["state_hash"].as_str()!=Some(expected){
            return Err("E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH: 화면과 Native 논리 실행이 다릅니다. 성공을 게시하지 않습니다".into());
        }Ok(())
    }
    pub(crate) fn prepare(self,revision:&[u8])->Result<NativeLogicalSourceWorldPrepared,String>{
        let parent=self.session.permit.source().revision();
        self.session.permit.verify_before_world(&self.session.adapter).map_err(reject)?;
        let artifact={let _timing=crate::storage_observation::Span::start("first.serialize_attachment");self.run.export_saved_artifact(parent,revision).map_err(reject)?};
        let checked=attachment::check_bytes(artifact.as_bytes())?;
        let context=attachment::check_saved_revision_context(&checked,&self.owned,parent,revision)?;
        let produced={let _timing=crate::storage_observation::Span::start("first.serialize_hash_request");crate::snapshot_v2_durable_transaction::PreparedLogicalRequestBorrowV1::from_run(&self.run,&context,parent,revision).map_err(reject)?};
        let request=produced.request();
        {let _timing=crate::storage_observation::Span::start("first.verify_request");self.session.permit.check_actual_world_request(&self.session.adapter,&context,&request).map_err(reject)?;}
        let scope=self.session.permit.replay_archive_scope(&self.session.adapter).map_err(reject)?;
        let plan={let _timing=crate::storage_observation::Span::start("first.plan_replay");crate::replay_evidence::PlannedFirstReplayCheckpointV1::from_produced_source_prepare(
            &produced,self.session.permit.source(),&context,&self.session.adapter,scope)?};
        let request=produced.into_request();
        Ok(NativeLogicalSourceWorldPrepared{execution:self,artifact,request,_replay_plan:plan})
    }
}
impl NativeLogicalSourceWorldPrepared {
    pub(crate) fn stage_restore_cache(&self,stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)->Result<(),String>{
        self.execution.session.permit.verify_before_world(&self.execution.session.adapter).map_err(reject)?;
        stage(&self.artifact,&self.request.payload().digest)
    }
    /// No output or artifact is published until the actual Witness commit.
    pub(crate) fn commit(self)->Result<(String,JsonValue),String>{
        let Self{execution,artifact,request,_replay_plan}=self;
        let NativeLogicalSourceWorldExecution{mut session,owned,run}=execution;
        let checked=attachment::check_bytes(artifact.as_bytes())?;
        let binding=request.binding();
        let parent=binding.expected_parent.clone();let revision=binding.new_revision.clone();
        let context=attachment::check_saved_revision_context(&checked,&owned,&parent,&revision)?;
        #[cfg(feature="beta2-crash-probe")]
        crate::snapshot_v2_durable_transaction::native::crash_probe::capture_candidate(
            &artifact,&run.session().summary(),&JsonValue::Null)?;
        let (replay_row,replay_pending)={let _timing=crate::storage_observation::Span::start("first.replay_pending");_replay_plan.begin_pending(&mut session.adapter)?};
        let start={let _timing=crate::storage_observation::Span::start("first.start_protected_attempt");session.permit.start_first_world_from_saved_source(&session.adapter,&context,request).map_err(reject)?};
        let prepared={let _timing=crate::storage_observation::Span::start("first.write_prepared");session.adapter.write_first_source_world_prepared(start,&context).map_err(reject)?};
        let retained={let _timing=crate::storage_observation::Span::start("first.commit_verify_and_publish");prepared.commit_with_replay(&session.adapter,&context,&_replay_plan,&replay_row)?};
        if let Some(cause)=retained.outcome().failure(){return Err(format!("E_SOURCE_LOGICAL_WORLD_NOT_COMMITTED: {cause:?}"));}
        {let _timing=crate::storage_observation::Span::start("first.replay_ready");_replay_plan.complete_ready(&mut session.adapter,&replay_row,replay_pending,&retained,&context)?;}
        Ok((artifact,run.session().summary()))
    }
}

/// A browser-restored logical run is deliberately not accepted here. Only
/// Native source/Witness re-admission can supply checkpoint provenance.
pub(crate) fn resume_logical_source_world(
    artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,locator:&str,
    input:&LogicalRecordRequest,
    source_owner:Option<&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>,
)->Result<JsonValue,String>{
    resume_logical_source_world_with_result(artifact,parent,revision,root,locator,input,source_owner,None).map(|v|v.0)
}

pub(crate) fn resume_logical_source_world_with_result(
    artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,locator:&str,
    input:&LogicalRecordRequest,
    source_owner:Option<&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>,
    result_for_count:Option<usize>,
)->Result<(JsonValue,Option<Result<JsonValue,String>>),String>{
    use crate::storage_observation::Span;
    let _total=Span::start("logical.resume_total");
    let mut adapter={let _phase=Span::start("logical.db_open");
        crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root).map_err(reject)?};
    let checked={let _phase=Span::start("logical.attachment_check");attachment::check_bytes(artifact.as_bytes())?};
    // The caller may supply only a real typed owner retained in this request.
    // Exact artifact/source binding and fresh Native admission below still run.
    let restored;
    let owned=match source_owner{Some(owner)=>owner,None=>{
        let _phase=Span::start("logical.attachment_restore");
        restored=attachment::restore_saved_runtime_payload(&checked)?;&restored
    }};
    let source={let _phase=Span::start("logical.source_context");attachment::check_saved_source_context(&checked,owned,parent,revision)?};
    let admitted={let _phase=Span::start("logical.source_readmit");adapter.readmit_current_source_world_source_context(locator,&source).map_err(reject)?};
    let context={let _phase=Span::start("logical.revision_context");attachment::check_saved_revision_context(&checked,owned,parent,revision)?};
    let mut run={let _phase=Span::start("logical.checkpoint_restore");
        CheckpointedLogicalRunV1::restore_native_source_checkpoint(&adapter,&admitted,&context,&input.binding).map_err(reject)?};
    {let _phase=Span::start("logical.input_suffix");for snapshot in &input.snapshots {run.step(snapshot)?;}}
    let summary={let _phase=Span::start("logical.summary");run.session().summary()};
    // Only the empty resave suffix uses this result. Preserve the host's
    // prefix/state/error ordering by returning a deferred fallible result.
    let result=if result_for_count.and_then(|n|u64::try_from(n).ok())==summary["madi_count"].as_u64() && result_for_count.is_some(){
        let _phase=Span::start("logical.resave_result_session_reuse");
        Some((||{
            let owner=crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1::from_restored_checkpoint(
                &adapter,&admitted,&context,&input.binding,run)?;
            Ok(serde_json::json!({"result_identity":owner.result_identity(&adapter)?}))
        })())
    }else{None};
    Ok((summary,result))
}

/// A follow-up logical W retains the exact previous input and receipt prefix.
/// It cannot reset to INITIAL or replace those inputs with general snapshots.
pub(crate) fn save_continued_logical_source_world(
    artifact:&str,parent:&[u8],revision:&[u8],next:&[u8],root:&std::path::Path,locator:&str,
    input:&LogicalRecordRequest,expected_state:Option<&str>,expected_result:Option<&str>,stage:impl FnOnce(&str,&[u8;32])->Result<(),String>,
)->Result<(String,JsonValue),String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root).map_err(reject)?;
    let checked=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&checked)?;
    let source=attachment::check_saved_source_context(&checked,&owned,parent,revision)?;
    let admitted=adapter.readmit_current_source_world_source_context(locator,&source).map_err(reject)?;
    let context=attachment::check_saved_revision_context(&checked,&owned,parent,revision)?;
    let progress_presence=admitted.replay_checkpoint_proof(&adapter,&context).map_err(reject)?;
    let has_progress=progress_presence.replay_progress_present(&adapter).map_err(reject)?;
    drop(progress_presence);
    if has_progress {
        return save_recording_continued_logical(adapter,admitted,&context,revision,next,root,locator,input,expected_state,expected_result,stage);
    }
    if expected_result.is_some()&&expected_state.is_none(){return Err("E_MISSING_REPLAY_SEED: 원 결과 증거 없는 저장본을 새 증거로 바꾸지 않습니다".into());}
    let mut run=CheckpointedLogicalRunV1::restore_native_source_checkpoint(&adapter,&admitted,&context,&input.binding).map_err(reject)?;
    for snapshot in &input.snapshots {run.step(snapshot)?;}
    if let Some(expected)=expected_state{
        if run.session().summary()["state_hash"].as_str()!=Some(expected){return Err("E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH".into());}
    }
    if expected_result.is_some(){return Err("E_MISSING_REPLAY_SEED: 원 결과 증거 없는 저장본을 새 증거로 바꾸지 않습니다".into());}
    let artifact=run.export_saved_artifact(revision,next).map_err(reject)?;
    let next_checked=attachment::check_bytes(artifact.as_bytes())?;
    let next_context=attachment::check_saved_revision_context(&next_checked,&owned,revision,next)?;
    let request=ActualPreparedRequestV1::from_logical(&run,&next_context,revision,next).map_err(reject)?;
    #[cfg(feature="beta2-crash-probe")]
    crate::snapshot_v2_durable_transaction::native::crash_probe::capture_candidate(
        &artifact,&run.session().summary(),&JsonValue::Null)?;
    stage(&artifact,&request.payload().digest)?;
    let start=admitted.start_next_source_world(&mut adapter,&context,&next_context,request).map_err(reject)?;
    let prepared=adapter.write_next_source_world_prepared(start,&next_context).map_err(reject)?;
    let retained=prepared.commit(&adapter,&next_context)?;
    if let Some(cause)=retained.outcome().failure(){return Err(format!("E_SOURCE_LOGICAL_WORLD_NOT_COMMITTED: {cause:?}"));}
    let summary=run.session().summary();
    drop(retained);drop(adapter);
    let empty=LogicalRecordRequest{binding:input.binding.clone(),snapshots:vec![]};
    let reopened=resume_logical_source_world(&artifact,revision,next,root,locator,&empty,None)?;
    if reopened!=summary{return Err("E_SOURCE_LOGICAL_WORLD_READBACK: 다시 읽은 실행 결과가 다릅니다".into());}
    Ok((artifact,summary))
}

fn save_recording_continued_logical(mut adapter:crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    admitted:crate::snapshot_v2_durable_transaction::native::ReadmittedFirstSourceWorldV2,
    previous:&attachment::SourceBoundSavedRuntimeAttachment<'_>,revision:&[u8],next:&[u8],root:&std::path::Path,locator:&str,
    input:&LogicalRecordRequest,expected_state:Option<&str>,expected_result:Option<&str>,stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)->Result<(String,JsonValue),String>{
    let mut recording=match crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1::restore(
        &adapter,&admitted,previous,&input.binding){
        Ok(recording)=>recording,
        Err(cause)=>{
            // Preserve the former normal-restore/world-check error priority
            // only on this rejected path. It never stages or saves a value,
            // and cannot replace the failed protected recording admission.
            if let Some(expected)=expected_state {
                let mut candidate=CheckpointedLogicalRunV1::restore_native_source_checkpoint(
                    &adapter,&admitted,previous,&input.binding).map_err(reject)?;
                for snapshot in &input.snapshots {candidate.step(snapshot)?;}
                if candidate.session().summary()["state_hash"].as_str()!=Some(expected){return Err("E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH".into());}
            }
            return Err(cause);
        }
    };
    recording.advance(&adapter,&input.snapshots)?;
    // Both content checks precede artifact/cache staging and Pending. This
    // actual owner replaces only the redundant speculative suffix execution.
    let summary=recording.summary()?;
    if let Some(expected)=expected_state{
        if summary["state_hash"].as_str()!=Some(expected){return Err("E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH".into());}
    }
    if let Some(expected)=expected_result{
        if recording.result_identity(&adapter)?!=expected{return Err("E_NATIVE_SOURCE_SAVE_RESULT_MISMATCH: 원 형식 결과가 화면과 다릅니다. 기존 저장본을 보존했습니다".into());}
    }
    let artifact=recording.checkpoint_run(&adapter)?.export_saved_artifact(revision,next).map_err(reject)?;
    let checked=attachment::check_bytes(artifact.as_bytes())?;
    let next_context=attachment::check_saved_revision_context(&checked,previous.owned(),revision,next)?;
    let request=ActualPreparedRequestV1::from_logical(recording.checkpoint_run(&adapter)?,&next_context,revision,next).map_err(reject)?;
    let plan=crate::replay_evidence::PlannedNextReplayCheckpointV1::from_native_prepare(
        &recording,&request,previous,&next_context,&adapter,&admitted)?;
    #[cfg(feature="beta2-crash-probe")]
    crate::snapshot_v2_durable_transaction::native::crash_probe::capture_candidate(&artifact,&summary,&JsonValue::Null)?;
    stage(&artifact,&request.payload().digest)?;
    let (row,pending)=plan.begin_pending(&mut adapter)?;
    let start=admitted.start_next_source_world(&mut adapter,previous,&next_context,request).map_err(reject)?;
    let prepared=adapter.write_next_source_world_prepared(start,&next_context).map_err(reject)?;
    let retained=prepared.commit_with_replay(&adapter,&next_context,&plan,&row)?;
    if let Some(cause)=retained.outcome().failure(){return Err(format!("E_SOURCE_LOGICAL_WORLD_NOT_COMMITTED: {cause:?}"));}
    plan.complete_ready(&mut adapter,&row,pending,&retained,&next_context)?;
    drop(retained);drop(plan);drop(recording);drop(adapter);
    let cold=load_native_logical_replay_archive(&artifact,revision,next,root,locator,&input.binding)
        .map_err(|e|format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: {e}"))?;
    if cold.compare_running_prefix(&input.binding)?!=summary {
        return Err("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 새로 연 기록 결과가 원 실행과 다릅니다".into());
    }
    Ok((artifact,summary))
}

/// Native-backed child archive, deliberately without Deref, Run, save/export
/// or normal restore methods. Reading bytes is not ReplayVerified.
pub(crate) struct NativeLogicalReplayArchiveV1 {
    adapter:crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    transport:crate::snapshot_v2_durable_transaction::native::NativeReplayProgressTransportV1,
}
impl NativeLogicalReplayArchiveV1 {
    pub(crate) fn checked_bytes(&self)->Result<&[u8],String>{
        self.transport.revalidate(&self.adapter).map_err(|e|format!("E_REPLAY_CURRENT_ROOT: {e:?}"))?;
        Ok(self.transport.bundle())
    }
    pub(crate) fn compare_running_prefix(&self,binding:&crate::logical_input_tape::LogicalInputBinding)
        ->Result<JsonValue,String>{
        let replay=crate::unit_wire::resumable_complete_state::ReadOnlyLogicalReplayV1::check_running_prefix(
            self.checked_bytes()?,binding)?;
        self.transport.revalidate(&self.adapter).map_err(reject)?;
        Ok(replay.checked_progress_summary())
    }
}
pub(crate) fn load_native_logical_replay_archive(artifact:&str,parent:&[u8],revision:&[u8],
    root:&std::path::Path,locator:&str,binding:&crate::logical_input_tape::LogicalInputBinding)
    ->Result<NativeLogicalReplayArchiveV1,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root).map_err(reject)?;
    let checked=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&checked)?;
    let source=attachment::check_saved_source_context(&checked,&owned,parent,revision)?;
    let admitted=adapter.readmit_current_source_world_source_context(locator,&source).map_err(reject)?;
    let context=attachment::check_saved_revision_context(&checked,&owned,parent,revision)?;
    // Normal restoration is independently admitted, but its VM/capability is
    // not lent to the replay child. No input or initial seed is manufactured.
    let _normal=CheckpointedLogicalRunV1::restore_native_source_checkpoint(&adapter,&admitted,&context,binding).map_err(reject)?;
    let proof=admitted.replay_checkpoint_proof(&adapter,&context).map_err(reject)?;
    let transport=adapter.load_current_replay_progress_transport(proof)
        .map_err(|e|format!("E_MISSING_REPLAY_PROGRESS: Native 기록 자료를 확인하지 못했습니다 ({e:?})"))?;
    transport.revalidate(&adapter).map_err(reject)?;
    Ok(NativeLogicalReplayArchiveV1{adapter,transport})
}

/// Current completed receipt only. Historical selection must be handled by
/// an actual historical owner or explicitly refused, never current-source fallback.
pub(crate) fn verify_native_logical_current_receipt(artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,
    locator:&str,binding:&crate::logical_input_tape::LogicalInputBinding)->Result<JsonValue,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root).map_err(reject)?;
    let checked=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&checked)?;
    let source=attachment::check_saved_source_context(&checked,&owned,parent,revision)?;
    let admitted=adapter.readmit_current_source_world_source_context(locator,&source).map_err(reject)?;
    let context=attachment::check_saved_revision_context(&checked,&owned,parent,revision)?;
    verify_admitted_logical_receipt(&adapter,&admitted,&context,binding)
}
pub(super) fn verify_admitted_logical_receipt(
    adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    admitted:&crate::snapshot_v2_durable_transaction::native::ReadmittedFirstSourceWorldV2,
    context:&attachment::SourceBoundSavedRuntimeAttachment<'_>,
    binding:&crate::logical_input_tape::LogicalInputBinding)->Result<JsonValue,String>{
    let proof=admitted.replay_checkpoint_proof(&adapter,&context).map_err(reject)?;
    let archive=adapter.load_current_replay_finished_transport(proof)
        .map_err(|e|format!("E_REPLAY_COMPLETED_RECEIPT_REQUIRED: 완료 영수증을 확인하지 못했습니다 ({e:?})"))?;
    archive.revalidate(&adapter).map_err(reject)?;
    let replay=crate::unit_wire::resumable_complete_state::ReadOnlyCompletedLogicalReplayV1::check(archive.bundle(),binding)?;
    archive.revalidate(&adapter).map_err(reject)?;
    Ok(serde_json::json!({"status":"REPLAY_VERIFIED","message":"재현 확인",
        "result_identity":format!("sha256:{}",hex::encode(replay.result_digest())),"summary":replay.summary()}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn b4_request_local_resume_owner_matches_restore_and_rejects_other_source(){
        let make=|source:&str|{
            let (root,_,saved)=super::super::tests::fixture(source);
            let locator=saved.grant_locator.as_deref().unwrap().to_string();
            let session=begin_first_authored_source_native(&root,&saved.project_identity,&locator,&saved.revision_identity,&saved.revision_integrity).unwrap();
            let owned=session.permit.source().recorded_execution_owner().unwrap();
            let mut input:LogicalRecordRequest=serde_json::from_str(include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/initial-input.json")).unwrap();
            input.binding.source_revision=sha256_identity(owned.sources().preparation().source.as_bytes());input.snapshots.truncate(1);
            let prepared=session.execute_logical(&input).unwrap().prepare(&[2]).unwrap();
            prepared.stage_restore_cache(|_,_|Ok(())).unwrap();let (artifact,_)=prepared.commit().unwrap();
            (root,locator,artifact,LogicalRecordRequest{binding:input.binding,snapshots:vec![]})
        };
        let source="실행:움직씨 := { (매마디)마다 { 1 보여주기. }. }.";
        let (root,locator,artifact,input)=make(source);
        let checked=attachment::check_bytes(artifact.as_bytes()).unwrap();
        let owned=attachment::restore_saved_runtime_payload(&checked).unwrap();
        let expected=resume_logical_source_world(&artifact,&[1],&[2],&root,&locator,&input,None).unwrap();
        let actual=resume_logical_source_world(&artifact,&[1],&[2],&root,&locator,&input,Some(&owned)).unwrap();
        assert_eq!(actual,expected,"request-local reuse must preserve the entire result");
        let (_,_,other_artifact,_)=make("실행:움직씨 := { (매마디)마다 { 2 보여주기. }. }.");
        let other_checked=attachment::check_bytes(other_artifact.as_bytes()).unwrap();
        let other=attachment::restore_saved_runtime_payload(&other_checked).unwrap();
        let error=resume_logical_source_world(&artifact,&[1],&[2],&root,&locator,&input,Some(&other)).unwrap_err();
        assert!(error.starts_with("E_PROJECT_RUNTIME_PAYLOAD_OWNER:"),"{error}");
        assert_eq!(resume_logical_source_world(&artifact,&[1],&[2],&root,&locator,&input,Some(&owned)).unwrap(),expected);
    }
    #[test]
    fn b3_native_no_end_keeps_anchor_and_never_becomes_complete_or_running(){
        for failed in [false,true]{
            let source="채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.";
            let (root,_,saved)=super::super::tests::fixture(source);
            let locator=saved.grant_locator.as_deref().unwrap();
            let session=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap();
            let owned=session.permit.source().recorded_execution_owner().unwrap();
            let mut input:LogicalRecordRequest=serde_json::from_str(include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/initial-input.json")).unwrap();
            input.binding.source_revision=sha256_identity(owned.sources().preparation().source.as_bytes());
            input.snapshots.truncate(1);
            let prepared=session.execute_logical(&input).unwrap().prepare(&[2]).unwrap();
            prepared.stage_restore_cache(|_,_|Ok(())).unwrap();
            let (artifact,anchor_summary)=prepared.commit().unwrap();
            let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
            let checked=attachment::check_bytes(artifact.as_bytes()).unwrap();
            let owned=attachment::restore_saved_runtime_payload(&checked).unwrap();
            let source_context=attachment::check_saved_source_context(&checked,&owned,&[1],&[2]).unwrap();
            let admitted=adapter.readmit_current_source_world_source_context(locator,&source_context).unwrap();
            let context=attachment::check_saved_revision_context(&checked,&owned,&[1],&[2]).unwrap();
            let before_receipt=admitted.current_receipt(&adapter,&context).unwrap();
            let before_head=adapter.selected_authoring_head(&admitted.payload().binding.project).unwrap();
            let mut owner=crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1::restore(&adapter,&admitted,&context,&input.binding).unwrap();
            let next=crate::logical_input_tape::LogicalSnapshot{madi:1,events:vec![],held:input.snapshots[0].held.clone()};
            owner.advance(&adapter,std::slice::from_ref(&next)).unwrap();
            if failed{
                let mut prefix=input.clone();prefix.snapshots.push(next);
                let expected_result=owner.result_identity(&adapter).unwrap();
                let invalid=crate::logical_input_tape::LogicalSnapshot{madi:2,events:vec![],held:std::collections::BTreeMap::new()};
                owner.reproduce_failed_input(&adapter,&prefix,&invalid,&expected_result).unwrap();
            }
            let plan=crate::replay_evidence::NativeNoEndCheckpointPlanV1::from_actual_stop(&mut owner,&adapter,&admitted,&context).unwrap();
            let stored=plan.commit(&mut owner,&mut adapter,&context).unwrap();
            let output=stored.output(&adapter).unwrap();
            assert_eq!(output["outcome"],if failed{"ABORTED"}else{"INCOMPLETE"});
            assert_eq!(output["summary"]["madi_count"],2,"NoEnd RAM prefix is not the saved anchor K=1");
            assert!(owner.prepare_finish(&adapter,None).is_err());
            assert_eq!(admitted.current_receipt(&adapter,&context).unwrap(),before_receipt);
            assert_eq!(adapter.selected_authoring_head(&admitted.payload().binding.project).unwrap(),before_head);
            assert!(crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1::restore(&adapter,&admitted,&context,&input.binding).is_err());
            let completed_proof=admitted.replay_checkpoint_proof(&adapter,&context).unwrap();
            assert!(adapter.load_current_replay_finished_transport(completed_proof).is_err());
            let mut new_record=crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1::start_new_record(
                &mut adapter,&admitted,&context,&input.binding).unwrap();
            assert_eq!(new_record.summary().unwrap(),anchor_summary,"new recording starts at genuine saved K=1, not NoEnd RAM N=2");
            assert!(stored.output(&adapter).is_err(),"old terminal owner is stale, not revived");
            assert_eq!(admitted.current_receipt(&adapter,&context).unwrap(),before_receipt);
            assert_eq!(adapter.selected_authoring_head(&admitted.payload().binding.project).unwrap(),before_head);
            let life=crate::replay_evidence::ReplayLifecycleCandidateV1::parse(new_record.checkpoint_lifecycle(&adapter).unwrap()).unwrap();
            assert_eq!(life.payload(6),Some(1u64.to_be_bytes().as_slice()));
            assert_eq!(life.payload(7),Some(2u64.to_be_bytes().as_slice()));
            assert_eq!(life.payload(9),Some(&[0][..]));
            assert!(crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1::start_new_record(
                &mut adapter,&admitted,&context,&input.binding).is_err(),"Running cannot silently start another epoch");
            let next=crate::logical_input_tape::LogicalSnapshot{madi:1,events:vec![],held:input.snapshots[0].held.clone()};
            assert_eq!(new_record.advance(&adapter,&[next]).unwrap()["madi_count"],2);
            drop(new_record);
            drop(stored);drop(owner);drop(admitted);drop(adapter);
            assert!(verify_native_logical_current_receipt(&artifact,&[1],&[2],&root,locator,&input.binding).is_err());
        }
    }
    #[test]
    fn wo22_logical_source_parent_actual_tetris_cold_three_plus_four(){
        // Raw authored sample, actual input binding and real S-parent custody.
        let source=include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/main.ddn");
        let (root,_,saved)=super::super::tests::fixture(source);
        let locator=saved.grant_locator.as_deref().unwrap();
        let begin=||begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap();
        let session=begin();
        let owned=session.permit.source().recorded_execution_owner().unwrap();
        let mut initial:LogicalRecordRequest=serde_json::from_str(include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/initial-input.json")).unwrap();
        let mut suffix:LogicalRecordRequest=serde_json::from_str(include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/next-input.json")).unwrap();
        let mut binding=initial.binding.clone();
        binding.source_revision=sha256_identity(owned.sources().preparation().source.as_bytes());
        initial.binding=binding.clone();suffix.binding=binding.clone();
        let mut snapshots=initial.snapshots.clone();snapshots.extend(suffix.snapshots.clone());
        let mut baseline=CheckpointedLogicalRunV1::fresh(&owned,binding.clone(),crate::ddn_runtime::default_age_target()).unwrap();
        for input in &snapshots {baseline.step(input).unwrap();}
        let prepared=session.execute_logical(&initial).unwrap().prepare(&[2]).unwrap();
        let original_bundle=prepared._replay_plan.bundle.clone();
        let original_tx=prepared._replay_plan.first_tx;
        prepared.stage_restore_cache(|_,_|Ok(())).unwrap();
        let (artifact,saved_summary)=prepared.commit().unwrap();
        // Observe the actual committed child, not a planned receipt or mock
        // archive. The normal cold path below remains independently admitted.
        let archive=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
        archive.test_assert_first_replay_archive(&original_tx,&original_bundle);
        drop(archive);
        let cold_archive=load_native_logical_replay_archive(&artifact,&[1],&[2],&root,locator,&binding).unwrap();
        assert_eq!(cold_archive.checked_bytes().unwrap(),original_bundle);
        assert_eq!(cold_archive.compare_running_prefix(&binding).unwrap(),saved_summary);
        drop(cold_archive);
        // A read-only comparison alone cannot resume. The normal Native
        // restore and current protected progress are joined by a separate owner.
        let mut recording_adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
        let saved_checked=attachment::check_bytes(artifact.as_bytes()).unwrap();
        let saved_owned=attachment::restore_saved_runtime_payload(&saved_checked).unwrap();
        let saved_source=attachment::check_saved_source_context(&saved_checked,&saved_owned,&[1],&[2]).unwrap();
        let current=recording_adapter.readmit_current_source_world_source_context(locator,&saved_source).unwrap();
        let saved_context=attachment::check_saved_revision_context(&saved_checked,&saved_owned,&[1],&[2]).unwrap();
        let mut recording=crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1::restore(
            &recording_adapter,&current,&saved_context,&binding).unwrap();
        assert_eq!(recording.summary().unwrap(),saved_summary);
        recording.advance(&recording_adapter,&suffix.snapshots).unwrap();
        assert_eq!(recording.summary().unwrap(),baseline.session().summary());
        let continued_result=recording.progress_evidence(&recording_adapter).unwrap();
        let continuous_result=crate::replay_evidence::ResultPreimageV1::capture(baseline.session().borrow_result_boundary().unwrap()).unwrap();
        assert_eq!(continued_result.result_digest(),continuous_result.digest_bytes());
        drop(recording);drop(current);drop(recording_adapter);
        for row_digest in [false,true]{
            let mutate=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
            mutate.test_flip_first_replay_byte(&original_tx,row_digest);drop(mutate);
            let rejected=load_native_logical_replay_archive(&artifact,&[1],&[2],&root,locator,&binding).is_err();
            let restore=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
            restore.test_flip_first_replay_byte(&original_tx,row_digest);drop(restore);
            assert!(rejected,"untrusted row/chunk bytes cannot become a Native recording owner");
        }
        let cold=resume_logical_source_world(&artifact,&[1],&[2],&root,locator,&suffix,None).unwrap();
        assert_eq!(cold,baseline.session().summary());
        let mut bad=suffix.clone();bad.binding.initial_rng^=1;
        assert!(resume_logical_source_world(&artifact,&[1],&[2],&root,locator,&bad,None).is_err());
        // A cache write failure precedes Pending/head changes and publishes no
        // new revision. The old saved world is still the exact same prefix.
        let expected_summary=baseline.session().summary();
        for (expected_state,expected_result,code) in [
            ("blake3:not-the-world","sha256:not-the-result","E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH"),
            (expected_summary["state_hash"].as_str().unwrap(),"sha256:not-the-result","E_NATIVE_SOURCE_SAVE_RESULT_MISMATCH"),
        ]{
            let refused=save_continued_logical_source_world(&artifact,&[1],&[2],&[3],&root,locator,&suffix,
                Some(expected_state),Some(expected_result),|_,_|panic!("mismatch must precede cache/Pending")).unwrap_err();
            assert!(refused.starts_with(code),"{refused}");
        }
        let error=save_continued_logical_source_world(&artifact,&[1],&[2],&[3],&root,locator,&suffix,None,None,
            |_,_|Err("E_TEST_CACHE_WRITE".into())).unwrap_err();
        assert_eq!(error,"E_TEST_CACHE_WRITE");
        let empty=LogicalRecordRequest{binding:binding.clone(),snapshots:vec![]};
        assert_eq!(resume_logical_source_world(&artifact,&[1],&[2],&root,locator,&empty,None).unwrap()["madi_count"],3);
        let expected=baseline.session().summary();
        let result_label=continuous_result.digest_label();
        let (next,summary)=save_continued_logical_source_world(&artifact,&[1],&[2],&[3],&root,locator,&suffix,
            expected["state_hash"].as_str(),Some(&result_label),|_,_|Ok(())).unwrap();
        assert_eq!(summary,baseline.session().summary());
        assert!(resume_logical_source_world(&artifact,&[1],&[2],&root,locator,&empty,None).is_err(),"stale first-W cannot reset epoch");
        let last=crate::logical_input_tape::LogicalSnapshot{madi:7,events:vec![],held:binding.initial_held.clone()};
        baseline.step(&last).unwrap();
        let cold=resume_logical_source_world(&next,&[2],&[3],&root,locator,&LogicalRecordRequest{binding,snapshots:vec![last]},None).unwrap();
        assert_eq!(cold,baseline.session().summary());
        // Finish at the real saved boundary (7), not the unsaved diagnostic
        // continuation above. Metadata must not mint a world/revision/madi.
        let mut finish_adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
        let finish_checked=attachment::check_bytes(next.as_bytes()).unwrap();
        let finish_owned=attachment::restore_saved_runtime_payload(&finish_checked).unwrap();
        let finish_source=attachment::check_saved_source_context(&finish_checked,&finish_owned,&[2],&[3]).unwrap();
        let finish_current=finish_adapter.readmit_current_source_world_source_context(locator,&finish_source).unwrap();
        let finish_context=attachment::check_saved_revision_context(&finish_checked,&finish_owned,&[2],&[3]).unwrap();
        let before_receipt=finish_current.current_receipt(&finish_adapter,&finish_context).unwrap();
        let before_head=finish_adapter.selected_authoring_head(&finish_current.payload().binding.project).unwrap();
        let mut finisher=crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1::restore(
            &finish_adapter,&finish_current,&finish_context,&initial.binding).unwrap();
        let candidate=finisher.prepare_finish(&finish_adapter,None).unwrap();
        let plan=crate::replay_evidence::NativeFinishCheckpointPlanV1::from_saved_candidate(
            &mut finisher,&candidate,&finish_adapter,&finish_current,&finish_context).unwrap();
        let output=plan.commit(&mut finisher,&mut finish_adapter,&finish_context,candidate).unwrap();
        assert_eq!(output.1,summary);
        let complete=LogicalRecordRequest{binding:initial.binding.clone(),snapshots};
        let expected=crate::input_tape_v3::record_authored_logical(&owned.project().save().unwrap(),&complete).unwrap();
        assert_eq!(output,expected);
        assert_eq!(finisher.finished_output(&finish_adapter).unwrap(),output,"same owner returns the original completion");
        assert!(finisher.advance(&finish_adapter,&[]).is_err());
        assert_eq!(finish_current.current_receipt(&finish_adapter,&finish_context).unwrap(),before_receipt);
        assert_eq!(finish_adapter.selected_authoring_head(&finish_current.payload().binding.project).unwrap(),before_head);
        assert!(crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1::restore(
            &finish_adapter,&finish_current,&finish_context,&initial.binding).is_err(),"Finished cannot revive Running in the same epoch");
        drop(finisher);drop(finish_current);drop(finish_adapter);
        let verified=verify_native_logical_current_receipt(&next,&[2],&[3],&root,locator,&initial.binding).unwrap();
        assert_eq!(verified["status"],"REPLAY_VERIFIED");assert_eq!(verified["summary"],summary);
        assert_eq!(verified["result_identity"],continuous_result.digest_label());
        let mut wrong=initial.binding.clone();wrong.initial_rng^=1;
        assert!(verify_native_logical_current_receipt(&next,&[2],&[3],&root,locator,&wrong).is_err());
    }
}

#[cfg(all(test,windows))]
mod prepared_request_borrow_tests {
    use super::*;
    #[test]
    fn first_prepared_request_borrow_is_exact_and_rejects_other_revision() {
        let (root,_,saved)=super::super::tests::fixture("실행:움직씨 := { (매마디)마다 { 7 보여주기. }. }.");
        let locator=saved.grant_locator.as_deref().unwrap();
        let initial=crate::runtime_surface::begin_first_authored_source_native(&root,&saved.project_identity,locator,
            &saved.revision_identity,&saved.revision_integrity).unwrap();
        let owned=initial.permit.source().recorded_execution_owner().unwrap();
        let mut input:LogicalRecordRequest=serde_json::from_str(include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/initial-input.json")).unwrap();
        input.binding.source_revision=crate::runtime_surface::sha256_identity(owned.sources().preparation().source.as_bytes());
        input.snapshots.truncate(1);
        let execution=initial.execute_logical(&input).unwrap();
        let parent=execution.session.permit.source().revision();
        let artifact=execution.run.export_saved_artifact(parent,&[2]).unwrap();
        let checked=attachment::check_bytes(artifact.as_bytes()).unwrap();
        let context=attachment::check_saved_revision_context(&checked,&execution.owned,parent,&[2]).unwrap();
        let produced=crate::snapshot_v2_durable_transaction::PreparedLogicalRequestBorrowV1::from_run(&execution.run,&context,parent,&[2]).unwrap();
        let original=ActualPreparedRequestV1::from_logical(&execution.run,&context,parent,&[2]).unwrap();
        assert_eq!(produced.request().raw(),original.raw(),"actual encoder bytes must remain identical");
        let source=execution.session.permit.source();let adapter=&execution.session.adapter;
        crate::replay_evidence::PlannedFirstReplayCheckpointV1::from_source_prepare(&execution.run,source,&original,&context,adapter,
            execution.session.permit.replay_archive_scope(adapter).unwrap()).unwrap();
        crate::replay_evidence::PlannedFirstReplayCheckpointV1::from_produced_source_prepare(&produced,source,&context,adapter,
            execution.session.permit.replay_archive_scope(adapter).unwrap()).unwrap();
        let foreign=execution.run.export_saved_artifact(parent,&[3]).unwrap();
        let foreign_checked=attachment::check_bytes(foreign.as_bytes()).unwrap();
        let foreign_context=attachment::check_saved_revision_context(&foreign_checked,&execution.owned,parent,&[3]).unwrap();
        assert!(crate::replay_evidence::PlannedFirstReplayCheckpointV1::from_produced_source_prepare(&produced,source,&foreign_context,adapter,
            execution.session.permit.replay_archive_scope(adapter).unwrap()).is_err());
        assert_eq!(produced.into_request().raw(),original.raw());
    }
}
