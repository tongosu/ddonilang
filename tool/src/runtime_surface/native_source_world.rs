//! Actual saved source -> once-permit VM execution -> FRAME2 request.
//! These live owners are not Committed/restore/receipt authority.
use super::*;
use crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1;
mod logical;
pub(crate) mod general_recording;
pub use general_recording::NativeGeneralRecordingPrepared;
pub use general_recording::{NativeGeneralReplayContentV1,read_current_general_recording_content};
mod recording_finish;
pub(crate) use recording_finish::NativeLogicalFinishSessionV1;
mod restart;
pub use restart::{begin_new_authored_source_native,NativeRestartSourceWorldSession,NativeRestartSourceWorldExecution,NativeRestartSourceWorldPrepared};
pub(crate) use logical::{resume_logical_source_world,save_continued_logical_source_world,verify_native_logical_current_receipt};

/// Consumes the real Native session. Not Clone, serde, or constructible from
/// a successful JSON response; failed VM execution never resets Started.
pub struct NativeFirstSourceWorldExecution {
    session:NativeFirstSourceWorldSession,
    run:GeneralSuccessfulInputPrefix,
}
pub struct NativeFirstSourceWorldPrepared {
    session:NativeFirstSourceWorldSession,
    run:GeneralSuccessfulInputPrefix,
    artifact:String,
    request:ActualPreparedRequestV1,
}
pub struct NativeFirstSourceWorldDurablyPrepared {
    adapter:crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    run:GeneralSuccessfulInputPrefix,
    artifact:String,
    prepared:crate::snapshot_v2_durable_transaction::native::PreparedFirstSourceWorldV2,
}
pub struct NativeFirstSourceWorldRetained {
    _adapter:crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    run:GeneralSuccessfulInputPrefix,
    artifact:String,
    retained:crate::snapshot_v2_durable_transaction::native::RetainedFirstSourceWorldV2,
    latest_summary:Option<JsonValue>,
}
impl NativeFirstSourceWorldDurablyPrepared {
    pub fn commit(self)->Result<NativeFirstSourceWorldRetained,String>{
        let Self{adapter,run,artifact,prepared}=self;
        let checked=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes())?;
        let b=prepared.request().binding();
        let snapshot=prepared.request().payload().snapshot.clone();
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&snapshot,&b.pesi)
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_FRAME: {cause:?}"))?;
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(&checked,&run.owned,&b.expected_parent,&b.new_revision,layout.world)?;
        let retained=prepared.commit(&adapter,&context)?;
        Ok(NativeFirstSourceWorldRetained{_adapter:adapter,run,artifact,retained,latest_summary:None})
    }
}
impl NativeFirstSourceWorldRetained {
    pub(crate) fn authoring_selection(&mut self,project:&[u8],locator:&str,artifact:&str)->Result<NativeAuthoringSelection,String>{
        native_authoring_selection_from_adapter(&mut self._adapter,project,locator,artifact)
    }
    pub(crate) fn current_authoring_source(&mut self,project:&[u8],locator:&str)->Result<Option<(NativeSourceRevisionReadback,NativeAuthoringSelection)>,String>{
        current_authoring_source_with_adapter(&mut self._adapter,project,locator)
    }
    pub(crate) fn save_changed_source(&mut self,artifact:&str,original:&str,revision:&[u8],locator:&str,selection:&NativeAuthoringSelection)->Result<NativeSourceRevisionReadback,String>{
        save_changed_authored_source_with_adapter(&mut self._adapter,artifact,original,revision,locator,selection)
    }
    pub(crate) fn browser_snapshot(&mut self,parent:&[u8],revision:&[u8],locator:&str)->Result<Vec<u8>,String>{
        self.saved_runtime_artifact()?;
        native_source_browser_snapshot_with_adapter(&self.artifact,parent,revision,&mut self._adapter,locator)
    }
    /// Uses the very same retained Native owner. This is a continuation result,
    /// not a new durable revision or a second first-use permit.
    pub fn resume_checkpoint(&mut self,parent:&[u8],revision:&[u8],locator:&str,ticks:u32)->Result<JsonValue,String>{
        self.saved_runtime_artifact()?;
        resume_first_saved_source_world_with_adapter(&self.artifact,parent,revision,&mut self._adapter,locator,ticks)
    }
    pub fn saved_runtime_artifact(&self)->Result<&str,String>{
        if let Some(failure)=self.retained.outcome().failure(){return Err(format!("E_SOURCE_FIRST_WORLD_NOT_COMMITTED: {failure:?}"));}
        Ok(&self.artifact)
    }
    pub fn summary(&self)->Result<&JsonValue,String>{self.saved_runtime_artifact()?;Ok(self.latest_summary.as_ref().unwrap_or(&self.run.summary))}
    pub(crate) fn save_continuation(&mut self,parent:&[u8],revision:&[u8],next:&[u8],locator:&str,ticks:u32,
        stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)->Result<JsonValue,String>{
        self.saved_runtime_artifact()?;
        let (artifact,retained,summary)=save_continued_source_world_with_adapter(&self.artifact,parent,revision,next,&mut self._adapter,locator,ticks,stage)?;
        self.artifact=artifact;self.retained=retained;
        self.latest_summary=Some(crate::unit_wire::SnapshotJsonOwner::clone_from_value(&summary).into_parent_value());
        Ok(summary)
    }
    pub(crate) fn current_digest(&mut self,project:&[u8],locator:&str)->Result<[u8;32],String>{
        self._adapter.current_source_world_digest(locator,project).map_err(|cause|format!("E_SOURCE_WORLD_CURRENT: {cause:?}"))
    }
    pub(crate) fn recording_content(&mut self,parent:&[u8],revision:&[u8],locator:&str)->Result<NativeGeneralReplayContentV1,String>{
        self.saved_runtime_artifact()?;
        general_recording::read_current_general_recording_content_with_adapter(&self.artifact,parent,revision,&mut self._adapter,locator)
    }
    /// CLI/host projection only; neither this text nor its locator is a lease.
    pub fn report_json(&self)->Result<String,String>{
        let artifact=self.saved_runtime_artifact()?;
        let mut report=crate::unit_wire::SnapshotJsonOwner::from_value(json!({
            "saved_artifact_json_utf8":artifact,"summary":null}));
        report.value_mut()["summary"]=crate::unit_wire::SnapshotJsonOwner::clone_from_value(self.summary()?).into_parent_value();
        Ok(report.compact_text())
    }
}
pub(crate) fn native_source_current_digest(root:&std::path::Path,project:&[u8],locator:&str)->Result<[u8;32],String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root).map_err(|cause|format!("E_SOURCE_WORLD_CURRENT: {cause:?}"))?;
    adapter.current_source_world_digest(locator,project).map_err(|cause|format!("E_SOURCE_WORLD_CURRENT: {cause:?}"))
}
pub fn save_continued_source_world_native(artifact:&str,parent:&[u8],revision:&[u8],next:&[u8],root:&std::path::Path,locator:&str,ticks:u32,
    stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)->Result<JsonValue,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root).map_err(|cause|format!("E_SOURCE_WORLD_COLD_OPEN: {cause:?}"))?;
    save_continued_source_world_with_adapter(artifact,parent,revision,next,&mut adapter,locator,ticks,stage).map(|(_,_,summary)|summary)
}
fn save_continued_source_world_with_adapter(artifact:&str,parent:&[u8],revision:&[u8],next:&[u8],
    adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,locator:&str,ticks:u32,
    stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)
    ->Result<(String,crate::snapshot_v2_durable_transaction::native::RetainedFirstSourceWorldV2,JsonValue),String>{
    use crate::unit_wire::{project_runtime_attachment as attachment,resumable_complete_state as complete};
    let reject=|cause|format!("E_SOURCE_WORLD_NEXT: {cause:?}");
    let bytes=attachment::check_bytes(artifact.as_bytes())?;let owned=attachment::restore_saved_runtime_payload(&bytes)?;
    let source=attachment::check_saved_source_context(&bytes,&owned,parent,revision)?;
    let admitted=adapter.readmit_current_source_world_source_context(locator,&source).map_err(reject)?;
    let payload=admitted.payload().clone();
    let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&payload.snapshot,source.pesi().as_raw()).map_err(reject)?;
    let context=attachment::check_saved_revision_context_with_frame(&bytes,&owned,parent,revision,layout.world)?;
    let progress=admitted.replay_checkpoint_proof(adapter,&context).map_err(reject)?;
    let has_progress=progress.replay_progress_present(adapter).map_err(reject)?;
    drop(progress);
    if has_progress{
        drop(admitted);
        let current=general_recording::NativeGeneralRecordingCurrentV1::restore(artifact,parent,revision,adapter,locator)?;
        return current.save_next(artifact,parent,revision,next,adapter,ticks,stage);
    }
    let history=|index|admitted.historical_payload(adapter,index,&context).map_err(reject);
    let (mut run,_)=rebuild_general_native_history_with_source(adapter,&payload,&context,Some(&history))?;
    run.receipts.push(admitted.current_receipt(adapter,&context).map_err(reject)?);
    let (summary,execution,rows)=resume_saved_project_artifact_from_runner_with_frame(artifact,parent,revision,ticks,
        Some((run.execution.runner,run.age)),Some(layout.world))?;
    run.execution=execution;run.rows.extend(rows);
    let artifact=run.execution.export_continued_artifact_native_complete(&owned,revision,next)?;
    let world=complete::encode_owned_world_candidate_with_source(run.execution.world.world(),context.pesi().as_raw(),run.execution.next_tick,&owned).map_err(reject)?;
    let frame=crate::unit_wire::check_complete_state_frame_v1(&world,context.pesi().as_raw()).map_err(reject)?;
    let next_bytes=attachment::check_bytes(artifact.as_bytes())?;
    let next_context=attachment::check_saved_revision_context_with_frame(&next_bytes,&owned,revision,next,frame)?;
    let request=ActualPreparedRequestV1::from_general_continuation(&run,&next_context,revision,next)?;
    #[cfg(feature="beta2-crash-probe")]
    crate::snapshot_v2_durable_transaction::native::crash_probe::capture_candidate(&artifact,&summary,&JsonValue::Null)?;
    // Cache bytes are data, never a published checkpoint. Stage them before
    // changing Pending/head so a cache failure leaves the old Witness intact.
    stage(&artifact,&request.payload().digest)?;
    let start=admitted.start_next_source_world(adapter,&context,&next_context,request).map_err(reject)?;
    let prepared=adapter.write_next_source_world_prepared(start,&next_context).map_err(reject)?;
    let retained=prepared.commit(adapter,&next_context)?;
    if let Some(failure)=retained.outcome().failure(){return Err(format!("E_SOURCE_WORLD_NEXT_NOT_COMMITTED: {failure:?}"));}
    let reopened=resume_first_saved_source_world_with_adapter(&artifact,revision,next,adapter,locator,0)?;
    if reopened["state_hash"]!=summary["state_hash"]||reopened["next_tick_id"]!=summary["next_tick_id"]{
        return Err("E_SOURCE_WORLD_NEXT_READBACK: 다시 읽은 상태가 다릅니다".into());
    }
    Ok((artifact,retained,summary))
}
pub(crate) fn native_source_browser_snapshot(artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,locator:&str)->Result<Vec<u8>,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root)
        .map_err(|cause|format!("E_SOURCE_WORLD_COLD_OPEN: {cause:?}"))?;
    native_source_browser_snapshot_with_adapter(artifact,parent,revision,&mut adapter,locator)
}
fn native_source_browser_snapshot_with_adapter(artifact:&str,parent:&[u8],revision:&[u8],
    adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,locator:&str)->Result<Vec<u8>,String>{
    use crate::unit_wire::project_runtime_attachment as attachment;
    let checked=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&checked)?;
    let source=attachment::check_saved_source_context(&checked,&owned,parent,revision)?;
    let actual=adapter.readmit_current_source_world_source_context(locator,&source)
        .map_err(|cause|format!("E_SOURCE_WORLD_COLD_ADMISSION: 복구 필요 — 저장 자료를 검증하지 못했습니다. 기존 저장본을 보존했습니다. cause={cause:?}"))?;
    let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&actual.payload().snapshot,source.pesi().as_raw())
        .map_err(|cause|format!("E_SOURCE_WORLD_COLD_FRAME: {cause:?}"))?;
    attachment::check_saved_revision_context_with_frame(&checked,&owned,parent,revision,layout.world)?;
    Ok(actual.payload().snapshot.clone())
}
/// Cold resume of the first source-parent W only. Native protected Witness,
/// full source ancestry and world/custody bytes precede any VM reconstruction.
pub fn resume_first_saved_source_world_native(artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,locator:&str,ticks:u32)
    ->Result<JsonValue,String>{
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root)
        .map_err(|cause|format!("E_SOURCE_WORLD_COLD_OPEN: {cause:?}"))?;
    resume_first_saved_source_world_with_adapter(artifact,parent,revision,&mut adapter,locator,ticks)
}
fn resume_first_saved_source_world_with_adapter(artifact:&str,parent:&[u8],revision:&[u8],
    adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,locator:&str,ticks:u32)->Result<JsonValue,String>{
    use crate::unit_wire::project_runtime_attachment as attachment;
    let checked=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&checked)?;
    let source=attachment::check_saved_source_context(&checked,&owned,parent,revision)?;
    let admitted=adapter.readmit_current_source_world_source_context(locator,&source)
        .map_err(|cause|format!("E_SOURCE_WORLD_COLD_ADMISSION: 복구 필요 — 저장 자료를 검증하지 못했습니다. 기존 저장본을 보존했습니다. cause={cause:?}"))?;
    let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&admitted.payload().snapshot,source.pesi().as_raw())
        .map_err(|cause|format!("E_SOURCE_WORLD_COLD_FRAME: {cause:?}"))?;
    let context=attachment::check_saved_revision_context_with_frame(&checked,&owned,parent,revision,layout.world)?;
    let history=|index|admitted.historical_payload(adapter,index,&context).map_err(|cause|format!("E_SOURCE_WORLD_HISTORY: {cause:?}"));
    let (verified,expected)=rebuild_general_native_history_with_source(&adapter,admitted.payload(),&context,Some(&history))?;
    let expected_bytes=attachment::check_bytes(expected.as_bytes())?;
    let exact=attachment::check_saved_revision_context_with_frame(&expected_bytes,&owned,parent,revision,layout.world)?;
    if exact.runtime_state()!=context.runtime_state()||exact.resource_snapshot()!=context.resource_snapshot()
        ||exact.revision_envelope()!=context.revision_envelope(){return Err("E_SOURCE_WORLD_COLD_STATE: 실제 입력·세계 복원이 다릅니다".into());}
    let (result,_,_)=resume_saved_project_artifact_from_runner_with_frame(artifact,parent,revision,ticks,Some((verified.execution.runner,verified.age)),Some(layout.world))?;
    Ok(result)
}
impl NativeFirstSourceWorldSession {
    pub fn execute(self,ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,input_tape:Option<&InputTape>)
        ->Result<NativeFirstSourceWorldExecution,String>{
        self.permit.verify_before_world(&self.adapter)
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_BEFORE_VM: {cause:?}"))?;
        let owned=self.permit.source().recorded_execution_owner()?;
        let artifact=owned.project().save()?;
        let subject_bootstrap=crate::unit_wire::subject_execution::SubjectExecutionStage::fresh_for_permitted_source(&owned)?;
        let run=run_general_input_prefix_from_owned_with_subject(owned,&artifact,ticks,base_seed,input_snapshots,input_tape,
            crate::ddn_runtime::default_age_target(),subject_bootstrap.as_ref(),Some(self.permit.source()))?;
        self.permit.verify_before_world(&self.adapter)
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_AFTER_VM: {cause:?}"))?;
        Ok(NativeFirstSourceWorldExecution{session:self,run})
    }
}
impl NativeFirstSourceWorldExecution {
    pub fn prepare(self,revision:&[u8])->Result<NativeFirstSourceWorldPrepared,String>{
        let parent=self.session.permit.source().revision();
        self.session.permit.verify_before_world(&self.session.adapter)
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_PREPARE: {cause:?}"))?;
        // The JSON projection must bind to the same complete binary world.
        // This stages values only; the actual Native permit and transaction
        // below remain mandatory, and default JSON export still rejects any
        // Subject lifetime whose saved runtime context is not connected yet.
        let encoded=self.run.encode_world_frame().map_err(|cause|format!("E_SOURCE_FIRST_WORLD_FRAME: {cause:?}"))?;
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(&self.run.owned)?;
        let frame=crate::unit_wire::check_complete_state_frame_v1(&encoded,issued.pesi().as_raw())
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_FRAME: {cause:?}"))?;
        let catalog=self.run.owned.registration().map(|registration|registration.saved_nominal_catalog(&self.run.owned)).transpose()?;
        let plan=self.run.execution.runner.saved_tuple_restore_plan();
        let projection=self.run.summary.pointer("/engine_state/state/snapshot_v2/resources")
            .ok_or("E_SOURCE_FIRST_WORLD_FRAME: 자원 투영이 없습니다")?;
        let staged=crate::unit_wire::stage_resource_snapshot_with_complete_frame(
            projection,&frame,plan.as_ref(),catalog.as_ref())?;
        if staged.state_hash()!=self.run.execution.world.state_hash(){
            return Err("E_SOURCE_FIRST_WORLD_FRAME: 실행 세계와 저장 공급이 다릅니다".into());
        }
        let artifact=self.run.export_saved_artifact_native_complete(parent,revision)?;
        let checked=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes())?;
        let source=crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(
            &checked,&self.run.owned,parent,revision,frame)?;
        let request=ActualPreparedRequestV1::from_general(&self.run,&source,parent,revision)?;
        self.session.permit.check_actual_world_request(&self.session.adapter,&source,&request)
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_BINDING: {cause:?}"))?;
        Ok(NativeFirstSourceWorldPrepared{session:self.session,run:self.run,artifact,request})
    }
}
impl NativeFirstSourceWorldPrepared {
    /// Untrusted host restore material, never a success publication or permit.
    /// Stage before the first SQL attempt just like subsequent-W caches.
    /// Reopening still requires the protected Pending/Witness and exact frame.
    pub fn stage_restore_cache(&self,stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)->Result<(),String>{
        self.session.permit.verify_before_world(&self.session.adapter)
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_CACHE_CURRENT: {cause:?}"))?;
        stage(&self.artifact,&self.request.payload().digest)
    }
    /// Candidate inspection is test-private; the public Native pipeline must
    /// not expose world output before the retained Committed/Witness gate.
    #[cfg(test)]
    fn snapshot_bytes(&self)->&[u8] {&self.request.payload().snapshot}
    pub fn write_prepared(self)->Result<NativeFirstSourceWorldDurablyPrepared,String>{
        let Self{session,run,artifact,request}=self;
        #[cfg(feature="beta2-crash-probe")]
        if std::env::var_os("DDN_B2_CRASH_CAPTURE").is_some() {
            // A development-only independent continuous reference, from the
            // same recorded source/initial subject owner and actual seed.
            // No observed prefix is converted into a logical input plan.
            if run.construction.snapshots.is_some()||run.construction.tape.is_some(){
                return Err("E_B2_PROBE_REFERENCE: 이 시험 경로의 명시 입력 기준은 아직 연결되지 않았습니다".into());
            }
            let ticks=run.construction.requested_ticks.checked_add(1).ok_or("E_B2_PROBE_REFERENCE: 마디 한도")?;
            let reference=run_general_input_prefix_from_owned_with_subject(run.owned.clone(),&run.owned.project().save()?,
                ticks,run.base_seed,None,None,run.construction.age,run.construction.subject_bootstrap.as_ref(),None)?;
            crate::snapshot_v2_durable_transaction::native::crash_probe::capture_candidate(&artifact,&run.summary,&reference.summary)?;
        }
        let checked=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes())?;
        let b=request.binding();
        let snapshot=request.payload().snapshot.clone();
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&snapshot,&b.pesi)
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_FRAME: {cause:?}"))?;
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(
            &checked,&run.owned,&b.expected_parent,&b.new_revision,layout.world)?;
        let start=session.permit.start_first_world_from_saved_source(&session.adapter,&context,request)
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_PENDING: {cause:?}"))?;
        let prepared=session.adapter.write_first_source_world_prepared(start,&context)
            .map_err(|cause|format!("E_SOURCE_FIRST_WORLD_PREPARED: {cause:?}"))?;
        Ok(NativeFirstSourceWorldDurablyPrepared{adapter:session.adapter,run,artifact,prepared})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
    pub(super) fn fixture(source:&str)->(std::path::PathBuf,String,NativeSourceRevisionReadback){
        let mut nonce=[0;32];getrandom::fill(&mut nonce).unwrap();
        let root=std::env::temp_dir().join(format!("ddn-r5-first-world-{}",hex::encode(nonce)));
        std::fs::create_dir(&root).unwrap();
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{
            root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
                SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.".into()},
                SourceFile{path:"main.ddn".into(),text:source.into()}]},String::new()).unwrap();
        let artifact=project.save().unwrap();
        let saved=save_initial_authored_source_native(&artifact,&[1],&root).unwrap();
        (root,artifact,saved)
    }
    #[test]
    fn wo22_first_restore_cache_never_grants_success_before_commit(){
        let (root,_,saved)=fixture("채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.");
        let locator=saved.grant_locator.as_deref().unwrap();
        let prepared=begin_first_authored_source_native(&root,&saved.project_identity,locator,
            &saved.revision_identity,&saved.revision_integrity).unwrap().execute(1,0,None,None).unwrap().prepare(&[2]).unwrap();
        let mut observed=None;
        prepared.stage_restore_cache(|artifact,digest|{
            assert!(!artifact.is_empty());observed=Some((artifact.to_owned(),*digest));Ok(())
        }).unwrap();
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let (kind,head):(i64,Vec<u8>)=db.query_row("SELECT payload_kind,history_revision_identity FROM vc04_project_heads",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!((kind,head),(0,vec![1]));
        assert_eq!(db.query_row("SELECT count(*) FROM vc04_recovery_custody_v1",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        drop(db);
        let retained=prepared.write_prepared().unwrap().commit().unwrap();
        assert_eq!(retained.saved_runtime_artifact().unwrap(),observed.as_ref().unwrap().0);
        let expected=retained.summary().unwrap()["state_hash"].clone();drop(retained);
        let cold=resume_first_saved_source_world_native(&observed.unwrap().0,&[1],&[2],&root,locator,0).unwrap();
        assert_eq!(cold["state_hash"],expected);
    }
    #[test]
    fn vc04_subject_and_fraction_tuple_continued_candidate_preserves_actual_world(){
        use crate::unit_wire::{project_runtime_attachment as attachment,resumable_complete_state as complete};
        let source="명찰:이름씨 := { 이름:글. }. 병사:임자 := { 이름:글. }. 채비 { 이름표:(나눔수,명찰) := (1 / 3,(이름=\"처음\")인 명찰). 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 표적 <- (이름=\"철수\")인 병사. 이름표 보여주기. }. }.";
        let (root,_,saved)=fixture(source);let locator=saved.grant_locator.as_deref().unwrap();
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap()
            .execute(1,0,None,None).unwrap();
        let baseline=run_general_input_prefix_from_owned_with_subject(execution.run.owned.clone(),&execution.run.owned.project().save().unwrap(),2,0,None,None,
            crate::ddn_runtime::default_age_target(),execution.run.construction.subject_bootstrap.as_ref(),None).unwrap();
        let mut retained=execution.prepare(&[2]).unwrap().write_prepared().unwrap().commit().unwrap();
        let artifact=retained.saved_runtime_artifact().unwrap().to_owned();
        let bytes=attachment::check_bytes(artifact.as_bytes()).unwrap();
        let owned=attachment::restore_saved_runtime_payload(&bytes).unwrap();
        let source_context=attachment::check_saved_source_context(&bytes,&owned,&[1],&[2]).unwrap();
        let admitted=retained._adapter.readmit_first_source_world_source_context(locator,&source_context).unwrap();
        let admitted_snapshot=admitted.payload().snapshot.clone();
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&admitted_snapshot,source_context.pesi().as_raw()).unwrap();
        let context=attachment::check_saved_revision_context_with_frame(&bytes,&owned,&[1],&[2],layout.world).unwrap();
        let history=|index|admitted.historical_payload(&retained._adapter,index,&context).map_err(|cause|format!("{cause:?}"));
        let (mut run,_)=rebuild_general_native_history_with_source(&retained._adapter,admitted.payload(),&context,Some(&history)).unwrap();
        run.receipts.push(admitted.current_receipt(&retained._adapter,&context).unwrap());
        let (_,execution,rows)=resume_saved_project_artifact_from_runner_with_frame(&artifact,&[1],&[2],1,
            Some((run.execution.runner,run.age)),Some(layout.world)).unwrap();
        run.execution=execution;run.rows.extend(rows);
        assert_eq!(run.execution.world.state_hash(),baseline.execution.world.state_hash());
        assert!(run.execution.export_continued_artifact(&run.owned,&[2],&[3]).unwrap_err().contains("SUBJECT_RUNTIME_CONTEXT_REQUIRED"));
        let next=run.execution.export_continued_artifact_native_complete(&run.owned,&[2],&[3]).unwrap();
        let pesi=crate::project_snapshot_identity::current::issue_saved_local_project(&run.owned).unwrap();
        let world=complete::encode_owned_world_candidate_with_source(run.execution.world.world(),pesi.pesi().as_raw(),run.execution.next_tick,&run.owned).unwrap();
        let frame=crate::unit_wire::check_complete_state_frame_v1(&world,pesi.pesi().as_raw()).unwrap();
        let next_bytes=attachment::check_bytes(next.as_bytes()).unwrap();
        let next_context=attachment::check_saved_revision_context_with_frame(&next_bytes,&run.owned,&[2],&[3],frame).unwrap();
        let raw=run.checkpoint_request(&next_context,&[2],&[3]).unwrap();
        let catalog=run.owned.registration().unwrap().saved_nominal_catalog(&run.owned).unwrap();
        let checked=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2_with_catalog(&raw,pesi.pesi().as_raw(),Some(&catalog)).unwrap();
        assert_eq!(checked.layout.world.next_tick,2);
        let values=checked.layout.world.decode_value_tables_with_catalog(Some(&catalog)).unwrap();
        assert_eq!(values.subject_custody.unwrap().next_subject_ordinal,2);assert_eq!(values.subjects.len(),2);
        assert_eq!(checked.binding.expected_parent,[2]);assert_eq!(checked.binding.new_revision,[3]);
        // Candidate construction is not a durable T2 or a new first permit.
        assert_eq!(retained.saved_runtime_artifact().unwrap(),artifact);
        retained.resume_checkpoint(&[1],&[2],locator,0).unwrap();
        // Only after the read-only candidate checks above, reserve a distinct
        // actual stateful epoch. No first-use permit is requested again.
        // The cold read above replaced the adapter's pinned live admission.
        // Acquire a fresh owner rather than reusing the earlier stale lease.
        let admitted=retained._adapter.readmit_first_source_world_source_context(locator,&source_context).unwrap();
        let actual=crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1::from_general_continuation(&run,&next_context,&[2],&[3]).unwrap();
        let start=admitted.start_next_source_world(&retained._adapter,&context,&next_context,actual).unwrap();
        let prepared=retained._adapter.write_next_source_world_prepared(start,&next_context).unwrap();
        let next_retained=prepared.commit(&retained._adapter,&next_context).unwrap();
        assert!(next_retained.outcome().failure().is_none());
        assert!(retained.resume_checkpoint(&[1],&[2],locator,0).is_err(),"epoch2 must reject stale first-W restore, never reset epoch1");
        let reopened=resume_first_saved_source_world_with_adapter(&next,&[2],&[3],&mut retained._adapter,locator,0).unwrap();
        assert_eq!(reopened["state_hash"],format!("blake3:{}",run.execution.world.state_hash().to_hex()));
    }
    #[test]
    fn m2_next_source_checkpoint_cas_failures_preserve_prior_rows_and_block_cold_publication(){
        use crate::snapshot_v2_durable_transaction::{native,SnapshotV2TransactionStateV1 as State};
        let raw="채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.";
        for (phase,state,cause,head,expected_rows) in [(1,State::Aborted,"AtomicCommitFailed",2u8,6),
            (2,State::RecoveryRequired,"ContainerDurabilityFailed",3,6),(3,State::RecoveryRequired,"ContainerDurabilityFailed",3,5)]{
            let (root,_,saved)=fixture(raw);let locator=saved.grant_locator.as_deref().unwrap();
            let execution=begin_first_authored_source_native(&root,&saved.project_identity,locator,&[1],&saved.revision_integrity).unwrap().execute(1,0,None,None).unwrap();
            let mut retained=execution.prepare(&[2]).unwrap().write_prepared().unwrap().commit().unwrap();
            let old_artifact=retained.artifact.clone();let old_summary=retained.summary().unwrap().clone();
            let db_path=root.join("vc04_snapshot_v2.sqlite3");
            let read_old=||{
                let db=rusqlite::Connection::open_with_flags(&db_path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
                let mut query=db.prepare("SELECT transaction_digest,journal_ordinal,recovery_record_bytes,row_sha256 FROM vc04_recovery_custody_v1 ORDER BY transaction_digest,journal_ordinal").unwrap();
                query.query_map([],|r|Ok((r.get::<_,Vec<u8>>(0)?,r.get::<_,i64>(1)?,r.get::<_,Vec<u8>>(2)?,r.get::<_,Vec<u8>>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
            };
            let previous=read_old();assert_eq!(previous.len(),3);
            let candidate=std::cell::RefCell::new(None);
            native::inject_source_world_fault(phase);
            let result=retained.save_continuation(&[1],&[2],&[3],locator,1,|artifact,digest|{*candidate.borrow_mut()=Some((artifact.to_owned(),*digest));Ok(())});
            native::inject_source_world_fault(0);
            let error=result.unwrap_err();assert!(error.contains(cause)&&error.contains(&format!("{state:?}")),"{error}");
            if phase==3 {assert!(error.contains("retention=WriteFailed"),"secondary failure must remain separate: {error}");}
            assert_eq!(retained.artifact,old_artifact);assert_eq!(retained.summary().unwrap(),&old_summary);
            let rows=read_old();assert_eq!(rows.len(),expected_rows);for row in &previous {assert!(rows.contains(row),"prior committed record changed");}
            let (next,digest)=candidate.into_inner().unwrap();assert!(rows.iter().any(|r|r.0==digest));
            let db=rusqlite::Connection::open_with_flags(&db_path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
            let actual_head:Vec<u8>=db.query_row("SELECT history_revision_identity FROM vc04_project_heads",[],|r|r.get(0)).unwrap();
            assert_eq!(actual_head,[head]);drop(db);drop(retained);
            // Test input data only: no registry/permit/Witness is created by
            // this file. The child must use the real cold Native admission.
            let cold_input=root.join("m2-next-source-cold-test-input.json");
            let input=json!({"root":root,"locator":locator,"old":old_artifact,"next":next});
            use std::io::Write;
            let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(&cold_input).unwrap();
            file.write_all(serde_json::to_string(&input).unwrap().as_bytes()).unwrap();file.sync_all().unwrap();drop(file);
            let child=std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact","runtime_surface::native_source_world::tests::m2_next_source_failed_cold_child","--ignored","--nocapture"])
                .env("DDN_M2_NEXT_SOURCE_COLD_TEST_INPUT",&cold_input).output().unwrap();
            assert!(child.status.success(),"fresh process cold admission: {}\n{}",String::from_utf8_lossy(&child.stdout),String::from_utf8_lossy(&child.stderr));
            println!("M2_NEXT_SOURCE_COLD_CHILD phase={phase} {}",String::from_utf8_lossy(&child.stdout));
            // Fresh adapter admission must not pick the older successful W1
            // or publish the failed W2. Stateful retry needs its exact owner.
            assert!(resume_first_saved_source_world_native(&old_artifact,&[1],&[2],&root,locator,0).is_err());
            assert!(resume_first_saved_source_world_native(&next,&[2],&[3],&root,locator,0).is_err());
            assert_eq!(read_old(),rows,"failed cold admission must not heal or rewrite the prefix");
            println!("M2_NEXT_SOURCE_FAULT phase={phase} state={state:?} cause={cause} head={head} old_rows=3 retained_rows={expected_rows} success_published=false root={}",root.display());
        }
    }
    #[test]
    #[ignore = "parent creates actual failed W2 and invokes a fresh process"]
    fn m2_next_source_failed_cold_child(){
        let path=std::env::var("DDN_M2_NEXT_SOURCE_COLD_TEST_INPUT").unwrap();
        let input:JsonValue=serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let root=std::path::Path::new(input["root"].as_str().unwrap());let locator=input["locator"].as_str().unwrap();
        for (key,parent,revision) in [("old",1u8,2u8),("next",2,3)]{
            let result=resume_first_saved_source_world_native(input[key].as_str().unwrap(),&[parent],&[revision],root,locator,0);
            let error=result.unwrap_err();assert!(error.contains("E_SOURCE_WORLD_COLD_ADMISSION"),"{error}");
            println!("M2_COLD_REJECTED {key}: {error}");
        }
    }
    #[test]
    fn vc04_actual_repeated_source_checkpoint_preserves_world_and_cache_failure(){
        let raw="병사:임자 := { 이름:글. }. 채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 표적 <- (이름=\"철수\")인 병사. 횟수 보여주기. }. }.";
        let (root,_,saved)=fixture(raw);let locator=saved.grant_locator.as_deref().unwrap();
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap().execute(1,0,None,None).unwrap();
        let baseline=run_general_input_prefix_from_owned_with_subject(execution.run.owned.clone(),&execution.run.owned.project().save().unwrap(),3,0,None,None,
            crate::ddn_runtime::default_age_target(),execution.run.construction.subject_bootstrap.as_ref(),None).unwrap();
        let mut retained=execution.prepare(&[2]).unwrap().write_prepared().unwrap().commit().unwrap();
        let original=retained.artifact.clone();
        assert!(retained.save_continuation(&[1],&[2],&[3],locator,1,|_,_|Err("TEST_CACHE_WRITE_FAILED".into())).is_err());
        assert_eq!(retained.artifact,original);
        retained.resume_checkpoint(&[1],&[2],locator,0).unwrap();
        retained.save_continuation(&[1],&[2],&[3],locator,1,|_,_|Ok(())).unwrap();
        retained.save_continuation(&[2],&[3],&[4],locator,1,|_,_|Ok(())).unwrap();
        let artifact=retained.artifact.clone();drop(retained);
        let result=resume_first_saved_source_world_native(&artifact,&[3],&[4],&root,locator,0).unwrap();
        assert_eq!(result["state_hash"],format!("blake3:{}",baseline.execution.world.state_hash().to_hex()));
        assert_eq!(result["next_tick_id"],3);
        assert!(resume_first_saved_source_world_native(&original,&[1],&[2],&root,locator,0).is_err());
    }
    #[test]
    fn vc04_registered_nominal_constructor_uses_actual_saved_declaration_identity(){
        let source="명찰:이름씨 := { 이름:글. }. 표찰:이름씨 := { 이름:글. }. 보관함:이름씨 := { 안:명찰. }. 병사:임자 := { 이름표:명찰. }. 대상함:이름씨 := { 대상:병사. }. 실행:움직씨 := { 첫:명찰 := (이름=\"철수\")인 명찰. 둘:표찰 := (이름=\"철수\")인 표찰. 담은것:보관함 := (안=첫)인 보관함. 첫 보여주기. 둘 보여주기. 담은것 보여주기. (첫 == 둘) 보여주기. }.";
        let (root,_,saved)=fixture(source);
        let locator=saved.grant_locator.as_deref().unwrap();
        let session=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap();
        let execution=session.execute(1,53,None,None).unwrap();
        let rendered=execution.run.summary.to_string();
        assert!(rendered.contains("거짓"),"different NameType identities must not collapse: {rendered}");
        let raw=execution.run.summary["output_log_texts"].as_array().unwrap().iter().take(3).map(|value| {
            let text=value.as_str().unwrap().strip_prefix("nominal-carrier-v1:").unwrap();
            text.to_owned()
        }).collect::<Vec<_>>();
        let owned=&execution.run.owned;
        let catalog=owned.registration().unwrap().saved_nominal_catalog(owned).unwrap();
        crate::unit_wire::check_actual_r5_nominal_catalog_vectors(&catalog,&raw,execution.session.permit.source().pesi());
        println!("VC04_NOMINAL_CONSTRUCTION_NATIVE_ROOT={}",root.display());
    }

    #[test]
    fn vc04_actual_subject_stage_hash_rollback_and_tombstone(){
        use ddonirang_core::{platform::{ResourceValue,SubjectWorldStateV1},nominal::NominalSlotTableV1};
        let raw="병사:임자 := { 이름:글. }. 채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 표적 <- (이름=\"철수\")인 병사. 복제 <- 표적. { 횟수 == 2 }이면 { (1 / 0) 보여주기. }. 표적 보여주기. }. }.";
        let (root,_,saved)=fixture(raw);
        let session=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),
            &saved.revision_identity,&saved.revision_integrity).unwrap();
        let mut execution=session.execute(1,53,None,None).unwrap();
        let world=execution.run.execution.world.world();
        let snapshot=world.subject_world().unwrap().clone();
        let before_hash=world.state_hash();
        let actual_target=world.get_resource_value("표적").unwrap();
        let ResourceValue::SubjectHandleV1(handle)=&actual_target else {panic!("actual raw Subject did not produce a handle")};
        assert_eq!(world.get_resource_value("표적"),world.get_resource_value("복제"));
        let owned=&execution.run.owned;
        let catalog=owned.registration().unwrap().saved_nominal_catalog(owned).unwrap();
        let schema=catalog.schema(&handle.name_type_identity).unwrap();
        let slots=schema.slots.iter().map(|slot|(slot.identity,slot.kind.clone(),ResourceValue::String("영희".into()))).collect::<Vec<_>>();
        let replacement=std::sync::Arc::new(NominalSlotTableV1::from_slots(schema.name_type,&slots,2_000_000,1_000_000).unwrap());
        let bootstrap=execution.run.construction.subject_bootstrap.as_ref().unwrap();
        let mut staged=bootstrap.stage_committed(&snapshot).unwrap();
        let reference=staged.acquire(&handle).unwrap();
        let crate::unit_wire::subject_execution::SubjectReadOutcome::Snapshot(frozen)=staged.read(&reference,&handle.name_type_identity).unwrap() else {panic!("LIVE read")};
        assert_eq!(frozen.slot(&schema.slots[0].identity),Some(&ResourceValue::String("철수".into())));
        staged.replace_payload(&handle,replacement.clone()).unwrap();
        let crate::unit_wire::subject_execution::SubjectReadOutcome::Snapshot(next)=staged.read(&reference,&handle.name_type_identity).unwrap() else {panic!("LIVE read after update")};
        assert_eq!(next.slot(&schema.slots[0].identity),Some(&ResourceValue::String("영희".into())));
        assert_eq!(frozen.slot(&schema.slots[0].identity),Some(&ResourceValue::String("철수".into())),"immutable captured slot after update");
        let mut changed=world.clone();changed.set_subject_world(staged.committed_snapshot().unwrap());
        assert_ne!(changed.state_hash(),before_hash,"same handles must not hide actual slot changes");
        assert_eq!(changed.get_resource_value("표적"),world.get_resource_value("표적"));
        assert_eq!(changed.get_resource_value("복제"),world.get_resource_value("복제"));
        assert_ne!(ddonirang_core::commit_canon::canonical_patch_ops_hash(&ddonirang_core::platform::Patch{
            origin:ddonirang_core::platform::Origin::system("ddn"),ops:vec![ddonirang_core::platform::PatchOp::SetSubjectWorld{value:snapshot.clone()}]}),
            ddonirang_core::commit_canon::canonical_patch_ops_hash(&ddonirang_core::platform::Patch{
            origin:ddonirang_core::platform::Origin::system("ddn"),ops:vec![ddonirang_core::platform::PatchOp::SetSubjectWorld{value:changed.subject_world().unwrap().clone()}]}));
        // This exercises the existing Rust reference owner only. The missing
        // source acquire/read callable is not supplied by this test.
        let reference=ddonirang_core::platform::IdentityReferenceValueV1{profile_version:1,
            target_type_identity:handle.name_type_identity.to_vec(),owner_space_identity:handle.world_identity.to_vec(),
            target_identity:handle.subject_identity.to_vec(),generation:0};
        staged.retire(&reference).unwrap();
        let dead=staged.committed_snapshot().unwrap();
        assert_eq!(dead.subject_values,0u64.to_be_bytes());
        assert!(staged.retire(&reference).is_err());
        assert_eq!(dead,staged.committed_snapshot().unwrap());
        assert!(staged.replace_payload(&handle,replacement.clone()).is_err());
        let next=staged.create(replacement).unwrap();assert_ne!(next.subject_identity,handle.subject_identity);
        let after=staged.committed_snapshot().unwrap();
        assert_ne!(after.subject_custody,dead.subject_custody);
        // The second raw tick creates a Subject and then fails. Neither the
        // real world, resources nor its next ordinal can escape that failure.
        let mut input=execution.run.rows[0].input.clone();input.tick_id=1;input.rng_seed=52;
        assert!(execution.run.execution.runner.run_update(world,&input,&std::collections::HashMap::new()).is_err());
        assert_eq!(world.state_hash(),before_hash);assert_eq!(world.subject_world(),Some(&snapshot));
        assert_eq!(bootstrap.stage_committed(&snapshot).unwrap().committed_snapshot().unwrap(),snapshot);
        let mut guarded=ddonirang_core::platform::DetNuri::new();*guarded.world_mut()=world.clone();
        guarded.apply_patch(&ddonirang_core::platform::Patch{origin:ddonirang_core::platform::Origin::Entity(ddonirang_core::platform::EntityId(0)),
            ops:vec![ddonirang_core::platform::PatchOp::SetSubjectWorld{value:changed.subject_world().unwrap().clone()},
                ddonirang_core::platform::PatchOp::GuardViolation{entity:ddonirang_core::platform::EntityId(0),rule_id:"실패 검사".into()}]},
            1,&mut ddonirang_core::signals::VecSignalSink::default());
        // Existing Core guard markers remain observable. Only the rejected
        // entity's writes are discarded; unrelated system patches are not.
        assert_eq!(guarded.world().subject_world(),Some(&snapshot),"discard the violator's Subject write");
        assert_eq!(guarded.world().get_resource_value("표적"),world.get_resource_value("표적"));
        // The resource-only codec must refuse, not silently erase this lane.
        let pesi=crate::project_snapshot_identity::current::issue_saved_local_project(owned).unwrap();
        assert!(crate::unit_wire::resumable_complete_state::encode_resource_only_world_candidate(world,pesi.pesi().as_raw(),1).is_err());
        let encoded=execution.run.encode_world_frame().unwrap();
        let frame=crate::unit_wire::check_complete_state_frame_v1(&encoded,pesi.pesi().as_raw()).unwrap();
        let tables=frame.decode_value_tables_with_catalog(Some(&catalog)).unwrap();
        assert_eq!(frame.subjects,snapshot.subject_values);assert_eq!(frame.subject_custody,snapshot.subject_custody);
        assert_eq!(tables.subjects.len(),1);assert_eq!(tables.subject_custody.as_ref().unwrap().next_subject_ordinal,1);
        assert_eq!(tables.resources.get("표적"),world.get_resource_value("표적").as_ref());
        assert_eq!(tables.resources.get("복제"),tables.resources.get("표적"));
        let projection=serialize_world_resources_snapshot(world);
        let restored=crate::unit_wire::stage_resource_snapshot_with_complete_frame(
            &projection,&frame,None,Some(&catalog)).unwrap();
        assert_eq!(restored.state_hash(),world.state_hash());
        assert_eq!(restored.subject_world(),world.subject_world());
        let mut missing_alias=projection.clone();
        missing_alias["value_det"].as_object_mut().unwrap().remove("복제");
        assert!(crate::unit_wire::stage_resource_snapshot_with_complete_frame(
            &missing_alias,&frame,None,Some(&catalog)).is_err());
        assert_eq!(world.state_hash(),before_hash,"failed pair cannot mutate the actual source world");
        let descriptor=execution.run.source_rebuild_descriptor_supply().unwrap();
        let rebuilt=crate::unit_wire::resumable_complete_state::rebuild_source_descriptor_candidate(&descriptor,owned).unwrap();
        assert_eq!(rebuilt.runner.saved_initial_subject_world().unwrap(),bootstrap.committed_snapshot().map(Some).unwrap());
        let recorded_bootstrap=crate::unit_wire::subject_execution::SubjectExecutionStage::from_recorded_initial(owned,&rebuilt.initial_world).unwrap();
        let authored=owned.project().save().unwrap();
        let replayed=run_general_input_prefix_from_owned_with_subject(owned.clone(),&authored,1,53,None,None,
            crate::ddn_runtime::default_age_target(),recorded_bootstrap.as_ref(),None).unwrap();
        assert_eq!(replayed.execution.world.state_hash(),before_hash,"binary initial custody must preserve actual World/Subject IDs");
        assert_eq!(replayed.encode_world_frame().unwrap(),encoded);
        let empty=[0;8];
        let initial_pesi=pesi.pesi();
        let no_custody=crate::unit_wire::CompleteStateFrameV1{
            pesi:initial_pesi.as_raw(),resources:&empty,parameters:&empty,next_tick:0,subjects:&empty,atom_evidence:&[],subject_custody:&[]};
        assert!(crate::unit_wire::subject_execution::SubjectExecutionStage::from_recorded_initial(owned,&no_custody).unwrap().is_none());
        let unrelated_initial=crate::unit_wire::CompleteStateFrameV1{next_tick:7,..no_custody};
        assert!(crate::unit_wire::subject_execution::SubjectExecutionStage::from_recorded_initial(owned,&unrelated_initial).unwrap().is_none(),
            "Subject bootstrap must not impose tick zero on an unrelated existing profile");
        assert!(run_general_input_prefix_from_owned_with_subject(owned.clone(),&authored,1,53,None,None,
            crate::ddn_runtime::default_age_target(),None,None).is_err(),"missing initial custody cannot trigger fresh identity repair");
        let not_initial=crate::unit_wire::CompleteStateFrameV1{
            pesi:initial_pesi.as_raw(),resources:&empty,parameters:&empty,next_tick:0,
            subjects:&snapshot.subject_values,atom_evidence:&[],subject_custody:&snapshot.subject_custody};
        assert!(crate::unit_wire::subject_execution::SubjectExecutionStage::from_recorded_initial(owned,&not_initial).is_err());
        // Full runtime/artifact admission still needs the actual binary
        // Subject context; encoding a frame alone must not publish success.
        assert!(execution.run.export_saved_artifact(&[1],&[2]).is_err());
        println!("VC04_ACTUAL_SUBJECT_STAGE_PASS root={} hash_before={} hash_after={} source_birth=true slot_update_and_retire=Rust_owner_only source_acquire_read=NOT_RUN",root.display(),before_hash.to_hex(),changed.state_hash().to_hex());
        let _:SubjectWorldStateV1=snapshot;
    }
    #[test]
    fn vc04_subject_json_only_refuses_native_complete_prepare_preserves_custody(){
        let (root,_,saved)=fixture("병사:임자 := { 이름:글. }. 실행:움직씨 := { 7 보여주기. }.");
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),
            &saved.revision_identity,&saved.revision_integrity).unwrap().execute(1,53,None,None).unwrap();
        assert!(execution.run.execution.world.world().subject_world().is_some());
        assert!(execution.run.encode_world_frame().is_ok());
        assert!(execution.run.export_saved_artifact(&[1],&[2]).unwrap_err().contains("E_VC04_SUBJECT_RUNTIME_CONTEXT_REQUIRED"));
        // R5 Native source-first may now supply the actual binary lifetime.
        // Only the old JSON-only export remains refused; no new JSON fields.
        let prepared=execution.prepare(&[2]).unwrap();
        let frame=crate::unit_wire::check_resumable_complete_state_layout_v2(
            &prepared.request.payload().snapshot,&prepared.request.binding().pesi).unwrap();
        let catalog=prepared.run.owned.registration().unwrap().saved_nominal_catalog(&prepared.run.owned).unwrap();
        let tables=frame.world.decode_value_tables_with_catalog(Some(&catalog)).unwrap();
        assert!(tables.subject_custody.is_some());assert!(tables.subjects.is_empty());
    }
    #[test]
    fn vc04_subject_reference_native_actual_cold(){
        let source=r#"병사:임자 := { 이름:글. }.
채비 { 횟수:바른수 := 0. }.
실행:움직씨 := { (매마디)마다 {
    횟수 <- 횟수 + 1.
    { 횟수 == 2 }이면 {
        이전표적:(병사)가리킴 := 표적.
        이전:(병사,가리킴안됨)맺음 := 이전표적 읽기.
        이전에 따라 {
            (병사,가리킴안됨)맺음.$잘됨(값)이면 { 값.이름 보여주기. }.
            (병사,가리킴안됨)맺음.$안됨(이유)이면 { "이전참조실패" 보여주기. }.
        }.
        (대상=이전표적) 폐기하기.
        폐기뒤:(병사,가리킴안됨)맺음 := 이전표적 읽기.
        폐기뒤에 따라 {
            (병사,가리킴안됨)맺음.$잘됨(값)이면 { "폐기실패" 보여주기. }.
            (병사,가리킴안됨)맺음.$안됨(이유)이면 { "폐기확인" 보여주기. }.
        }.
    }.
    철수:병사 := (이름="철수")인 병사.
    보관 <- 철수.
    표적 <- 철수 가리키기.
    읽은것:(병사,가리킴안됨)맺음 := 표적 읽기.
    읽은것에 따라 {
        (병사,가리킴안됨)맺음.$잘됨(값)이면 { 값.이름 보여주기. }.
        (병사,가리킴안됨)맺음.$안됨(이유)이면 { }.
    }.
}. }."#;
        let (root,_,saved)=fixture(source);let locator=saved.grant_locator.as_deref().unwrap();
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap()
            .execute(1,53,None,None).unwrap();
        let owned=execution.run.owned.clone();let authored=owned.project().save().unwrap();
        let baseline=run_general_input_prefix_from_owned_with_subject(owned,&authored,2,53,None,None,
            crate::ddn_runtime::default_age_target(),execution.run.construction.subject_bootstrap.as_ref(),None).unwrap();
        let before_resources=execution.run.summary["engine_state"]["resources"].clone();
        let before_hash=execution.run.summary["state_hash"].clone();
        let retained=execution.prepare(&[2]).unwrap().write_prepared().unwrap().commit().unwrap();
        let artifact=retained.saved_runtime_artifact().unwrap().to_owned();
        std::fs::write(root.join("saved-artifact.json"),&artifact).unwrap();
        std::fs::write(root.join("grant-locator.txt"),locator).unwrap();drop(retained);
        let zero=resume_first_saved_source_world_native(&artifact,&[1],&[2],&root,locator,0).unwrap();
        assert_eq!(zero["resources"],before_resources);assert_eq!(zero["state_hash"],before_hash);
        let child=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","runtime_surface::native_source_world::tests::vc04_first_world_cold_process_entry","--ignored","--nocapture"])
            .env("DDN_R5_FIRST_WORLD_COLD_ROOT",&root).status().unwrap();assert!(child.success());
        let cold:JsonValue=serde_json::from_slice(&std::fs::read(root.join("cold-result.json")).unwrap()).unwrap();
        assert_eq!(cold["state_hash"],baseline.summary["state_hash"]);
        assert_eq!(cold["resources"],baseline.summary["engine_state"]["resources"]);
        assert_eq!(cold["next_tick_id"],2);
        assert_eq!(cold["output_display"]["texts"],json!(["철수","철수","폐기확인","철수"]));
        println!("VC04_SUBJECT_REFERENCE_NATIVE_COLD_PASS root={} hash={}",root.display(),cold["state_hash"]);
    }
    #[test]
    fn vc04_actual_source_acquire_then_retire_keeps_tombstone_and_ordinal(){
        let source="병사:임자 := { 이름:글. }. 실행:움직씨 := { 철수:병사 := (이름=\"철수\")인 병사. 첫:(병사)가리킴 := 철수 가리키기. 표적:(병사)가리킴 := 철수 가리키기. (첫 == 표적) 보여주기. (대상=표적) 폐기하기. }.";
        let (root,_,saved)=fixture(source);
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),&saved.revision_identity,&saved.revision_integrity).unwrap()
            .execute(1,53,None,None).unwrap();
        let frame=execution.run.encode_world_frame().unwrap();
        let catalog=execution.run.owned.registration().unwrap().saved_nominal_catalog(&execution.run.owned).unwrap();
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(&execution.run.owned).unwrap();
        let tables=crate::unit_wire::check_complete_state_frame_v1(&frame,issued.pesi().as_raw()).unwrap().decode_value_tables_with_catalog(Some(&catalog)).unwrap();
        let custody=tables.subject_custody.unwrap();
        assert_eq!(custody.next_subject_ordinal,1);assert_eq!(custody.rows.len(),1);assert!(!custody.rows[0].live);
        assert!(tables.subjects.is_empty());
        assert!(execution.run.summary["output_log_texts"].as_array().unwrap().iter().any(|value|value=="참"));
    }
    #[test]
    fn vc04_tombstone_only_native_actual_cold_never_reuses_ordinal(){
        let source="병사:임자 := { 이름:글. }. 채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 철수:병사 := (이름=\"철수\")인 병사. 표적:(병사)가리킴 := 철수 가리키기. (대상=표적) 폐기하기. \"폐기확인\" 보여주기. }. }.";
        let (root,_,saved)=fixture(source);let locator=saved.grant_locator.as_deref().unwrap();
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap()
            .execute(1,53,None,None).unwrap();
        let owned=execution.run.owned.clone();let authored=owned.project().save().unwrap();
        let baseline=run_general_input_prefix_from_owned_with_subject(owned,&authored,2,53,None,None,
            crate::ddn_runtime::default_age_target(),execution.run.construction.subject_bootstrap.as_ref(),None).unwrap();
        let baseline_bytes=baseline.encode_world_frame().unwrap();
        let pesi=crate::project_snapshot_identity::current::issue_saved_local_project(&baseline.owned).unwrap();
        let baseline_catalog=baseline.owned.registration().unwrap().saved_nominal_catalog(&baseline.owned).unwrap();
        let baseline_tables=crate::unit_wire::check_complete_state_frame_v1(&baseline_bytes,pesi.pesi().as_raw()).unwrap()
            .decode_value_tables_with_catalog(Some(&baseline_catalog)).unwrap();
        assert!(baseline_tables.subjects.is_empty());let custody=baseline_tables.subject_custody.unwrap();
        assert_eq!(custody.next_subject_ordinal,2);assert_eq!(custody.rows.len(),2);
        assert!(custody.rows.iter().all(|row|!row.live));
        let prepared=execution.prepare(&[2]).unwrap();
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&prepared.request.payload().snapshot,&prepared.request.binding().pesi).unwrap();
        let catalog=prepared.run.owned.registration().unwrap().saved_nominal_catalog(&prepared.run.owned).unwrap();
        let tables=layout.world.decode_value_tables_with_catalog(Some(&catalog)).unwrap();
        assert!(tables.subjects.is_empty());let first=tables.subject_custody.unwrap();
        assert_eq!(first.next_subject_ordinal,1);assert_eq!(first.rows.len(),1);assert!(!first.rows[0].live);
        let retained=prepared.write_prepared().unwrap().commit().unwrap();
        std::fs::write(root.join("saved-artifact.json"),retained.saved_runtime_artifact().unwrap()).unwrap();
        std::fs::write(root.join("grant-locator.txt"),locator).unwrap();drop(retained);
        let child=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","runtime_surface::native_source_world::tests::vc04_first_world_cold_process_entry","--ignored","--nocapture"])
            .env("DDN_R5_FIRST_WORLD_COLD_ROOT",&root).status().unwrap();assert!(child.success());
        let cold:JsonValue=serde_json::from_slice(&std::fs::read(root.join("cold-result.json")).unwrap()).unwrap();
        assert_eq!(cold["state_hash"],baseline.summary["state_hash"]);
        assert_eq!(cold["resources"],baseline.summary["engine_state"]["resources"]);
        assert_eq!(cold["next_tick_id"],2);assert_eq!(cold["output_display"]["texts"],json!(["폐기확인","폐기확인"]));
        println!("VC04_TOMBSTONE_NATIVE_COLD_PASS root={} hash={}",root.display(),cold["state_hash"]);
    }
    #[test]
    fn vc04_source_double_retire_and_stale_acquire_never_publish_world(){
        for tail in ["(대상=표적) 폐기하기.","철수 가리키기 보여주기."] {
            let source=format!("병사:임자 := {{ 이름:글. }}. 실행:움직씨 := {{ 철수:병사 := (이름=\"철수\")인 병사. 표적:(병사)가리킴 := 철수 가리키기. (대상=표적) 폐기하기. {tail} }}.");
            let (root,_,saved)=fixture(&source);
            let session=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),&saved.revision_identity,&saved.revision_integrity).unwrap();
            assert!(session.execute(1,53,None,None).is_err());
            let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
            let worlds:i64=db.query_row("SELECT count(*) FROM vc04_prepared_payloads WHERE payload_kind=1",[],|row|row.get(0)).unwrap();
            assert_eq!(worlds,0,"failed source must not publish a partial world");
        }
    }
    #[test]
    fn vc04_actual_read_snapshot_survives_retirement_and_reports_broken(){
        let source="병사:임자 := { 이름:글. }. 실행:움직씨 := { 철수:병사 := (이름=\"철수\")인 병사. 표적:(병사)가리킴 := 철수 가리키기. 읽은것:(병사,가리킴안됨)맺음 := 표적 읽기. (대상=표적) 폐기하기. 읽은것에 따라 { (병사,가리킴안됨)맺음.$잘됨(값)이면 { 이름복사 <- 값.이름. 값.이름 보여주기. }. (병사,가리킴안됨)맺음.$안됨(이유)이면 { \"실패\" 보여주기. }. }. 이름복사 보여주기. 표적 읽기 보여주기. }.";
        let (root,_,saved)=fixture(source);
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),&saved.revision_identity,&saved.revision_integrity).unwrap()
            .execute(1,53,None,None).unwrap();
        let output=execution.run.summary["output_log_texts"].as_array().unwrap();
        assert_eq!(output[0],"철수");
        assert_eq!(output[1],"철수","explicit scalar projection remains a normal persistent value");
        assert!(output[2].as_str().unwrap().contains("끊김"));
    }
    #[test]
    fn vc04_nominal_world_save_cold_resume_matches_continuous(){
        let source="명찰:이름씨 := { 이름:글. }. 채비 { 이름표:명찰 := (이름=\"처음\")인 명찰. 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. { 횟수 == 1 }이면 { 이름표 <- (이름=\"첫마디\")인 명찰. }아니면 { 이름표 <- (이름=\"다음마디\")인 명찰. }. 이름표 보여주기. }. }.";
        assert_nominal_world_actual_cold(source);
    }
    #[test]
    fn r3_1_registered_sum_native_actual_cold_uses_the_same_catalog_and_world() {
        let source="명찰:이름씨 := { 이름:글. }. 선택:고름씨 := { $쌍(가로:바른수,세로:바른수). }. 채비 { 이름표:명찰 := (이름=\"처음\")인 명찰. 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 선택.$쌍(세로=횟수,가로=횟수)에 따라 { 선택.$쌍(가로,세로)이면 { (가로+세로) 보여주기. }. }. 이름표 보여주기. }. }.";
        assert_nominal_world_actual_cold(source);
    }
    #[test]
    fn r3_3_optional_source_proof_native_actual_cold_preserves_registered_identity() {
        let source="명찰:이름씨 := { 이름:글. }. 채비 { 이름표:명찰 := (이름=\"처음\")인 명찰. 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 자료 <- ((수)있없,글)맺음.$잘됨(값=없음). 자료에 따라 { ((수)있없,글)맺음.$잘됨(값)이면 { (값 == 없음) 보여주기. }. ((수)있없,글)맺음.$안됨(이유)이면 { 이유 보여주기. }. }. 이름표 보여주기. }. }.";
        // Reuse the actual Native first-world -> SQLite -> new-process cold
        // owner, compared with uninterrupted execution from the same owner.
        // The Optional remains a local value/None, not a new stored Sum tag.
        assert_nominal_world_actual_cold(source);
    }
    #[test]
    fn vc04_outer_tuple_rational_nominal_actual_cold(){
        let source="명찰:이름씨 := { 이름:글. }. 채비 { 이름표:(나눔수,명찰) := (1 / 3,(이름=\"처음\")인 명찰). 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 이름표 보여주기. }. }.";
        assert_nominal_world_actual_cold_with_carrier(source,"tuple_v1");
    }
    #[test]
    fn vc04_registered_source_atom_native_actual_cold(){
        let source="명찰:이름씨 := { 이름:글. }. 채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 이름표 <- #빨강. 이름표 보여주기. \"#빨강\" 보여주기. }. }.";
        assert_nominal_world_actual_cold_with_carrier(source,"registered_atom_v1");
    }
    #[test]
    fn vc04_local_atom_and_standard_side_share_spelling_not_owner(){
        let source="명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 이름표 <- #오른쪽. 기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 곁 := (보기=(() 표준보기.결과표),자리=#오른쪽) 표준보개.곁. (관계들=차림[기본,곁]) 표준보개.짜임. }.";
        let (root,_,saved)=fixture(source);
        let session=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),&saved.revision_identity,&saved.revision_integrity).unwrap();
        let owned=session.permit.source().recorded_execution_owner().unwrap();
        let descriptor=crate::registered_atom_contract::source_binding::Candidate::parse(owned.registration().unwrap().descriptor()).unwrap();
        assert_eq!(descriptor.bindings.len(),1,"standard policy occurrence is not a local registration");
        assert_eq!(descriptor.registries.len(),1);assert_eq!(descriptor.registries[0].entries.len(),1);
        let execution=session.execute(1,53,None,None).unwrap();
        assert!(matches!(execution.run.execution.world.world().get_resource_value("이름표"),Some(ddonirang_core::platform::ResourceValue::RegisteredAtom(_))));
    }
    #[test]
    fn vc04_snapshot_rational_projection_preserves_exact_source_kind(){
        let source="병사:임자 := { 비율:나눔수. }. 실행:움직씨 := { 철수:병사 := (비율=1 / 3)인 병사. 표적:(병사)가리킴 := 철수 가리키기. 읽은것:(병사,가리킴안됨)맺음 := 표적 읽기. 읽은것에 따라 { (병사,가리킴안됨)맺음.$잘됨(값)이면 { 비율복사:나눔수 := 값.비율. (비율복사 == 1 / 3) 보여주기. 비율복사 보여주기. }. (병사,가리킴안됨)맺음.$안됨(이유)이면 { }. }. }.";
        let (root,_,saved)=fixture(source);
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),&saved.revision_identity,&saved.revision_integrity).unwrap()
            .execute(1,53,None,None).unwrap();
        assert_eq!(execution.run.summary["output_log_texts"],json!(["참","1/3"]));
        // Local projection is covered here. Nested persistent numeric writes
        // remain the existing straight-line saved-type supplier's boundary.
    }
    #[test]
    fn vc04_tuple_rational_nominal_world_actual_cold(){
        let source="명찰:이름씨 := { 이름:글. }. 쌍명찰:이름씨 := { 짝:(나눔수,명찰). }. 채비 { 이름표:쌍명찰 := (짝=(1 / 3,(이름=\"처음\")인 명찰))인 쌍명찰. 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 이름표 <- (짝=(1 / 3,(이름=\"다음마디\")인 명찰))인 쌍명찰. 이름표 보여주기. }. }.";
        assert_nominal_world_actual_cold(source);
    }
    #[test]
    fn vc04_tuple_nominal_source_rejects_wrong_arity_order_and_identity(){
        for tuple in ["(1 / 3,)","((이름=\"가\")인 명찰,1 / 3)","(1 / 3,(이름=\"가\")인 다른명찰)"] {
            let source=format!("명찰:이름씨 := {{ 이름:글. }}. 다른명찰:이름씨 := {{ 이름:글. }}. 쌍명찰:이름씨 := {{ 짝:(나눔수,명찰). }}. 실행:움직씨 := {{ 값 <- (짝={tuple})인 쌍명찰. 값 보여주기. }}.");
            let (root,_,saved)=fixture(&source);
            let session=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),&saved.revision_identity,&saved.revision_integrity).unwrap();
            assert!(session.execute(1,53,None,None).is_err(),"invalid slot admitted: {tuple}");
        }
    }
    fn assert_nominal_world_actual_cold(source:&str){
        assert_nominal_world_actual_cold_with_carrier(source,"vc04_canonical_carrier");
    }
    fn assert_nominal_world_actual_cold_with_carrier(source:&str,carrier:&str){
        let (root,_,saved)=fixture(source);let locator=saved.grant_locator.as_deref().unwrap();
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap()
            .execute(1,53,None,None).unwrap();
        let owned=execution.run.owned.clone();let authored=owned.project().save().unwrap();
        let baseline=run_general_input_prefix_from_owned(owned.clone(),&authored,2,53,None,None,crate::ddn_runtime::default_age_target()).unwrap();
        let retained=execution.prepare(&[2]).unwrap().write_prepared().unwrap().commit().unwrap();
        let artifact=retained.saved_runtime_artifact().unwrap().to_owned();
        if carrier=="tuple_v1" {
            let checked=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes()).unwrap();
            assert!(crate::unit_wire::parse_snapshot_json(&checked.resource_snapshot).is_err(),
                "opaque Tuple bytes must not mint the missing nominal catalog");
            let catalog=owned.registration().unwrap().saved_nominal_catalog(&owned).unwrap();
            crate::unit_wire::parse_snapshot_json_with_catalog(&checked.resource_snapshot,Some(&catalog)).unwrap();
            assert!(crate::unit_wire::project_runtime_attachment::check_runtime_resource_binding(
                &checked.runtime_state,&checked.resource_snapshot).is_err());
        }
        assert_ne!(retained.summary().unwrap()["state_hash"],baseline.summary["state_hash"]);
        std::fs::write(root.join("saved-artifact.json"),&artifact).unwrap();
        std::fs::write(root.join("grant-locator.txt"),locator).unwrap();drop(retained);
        let child=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","runtime_surface::native_source_world::tests::vc04_first_world_cold_process_entry","--ignored","--nocapture"])
            .env("DDN_R5_FIRST_WORLD_COLD_ROOT",&root).status().unwrap();assert!(child.success());
        let cold:JsonValue=serde_json::from_slice(&std::fs::read(root.join("cold-result.json")).unwrap()).unwrap();
        assert_eq!(cold["state_hash"],baseline.summary["state_hash"]);
        assert_eq!(cold["resources"],baseline.summary["engine_state"]["resources"]);
        assert_eq!(cold["resources"]["fixed64"]["횟수"],"2");assert_eq!(cold["next_tick_id"],2);
        assert!(cold["resources"].to_string().contains(carrier));
        if carrier=="registered_atom_v1" {
            assert_eq!(cold["output_display"]["texts"].as_array().unwrap().last(),Some(&json!("#빨강")),
                "text spelling must not be promoted into an atom");
        }
        println!("VC04_NOMINAL_WORLD_ACTUAL_COLD_PASS root={} hash={} next_tick=2",root.display(),cold["state_hash"]);
    }
    #[test]
    fn vc04_first_world_actual_saved_owner_vm_and_frame_keep_source_parent(){
        let (root,_,saved)=fixture("명찰:이름씨 := { 이름:글. }. 채비 { 값:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }.");
        let locator=saved.grant_locator.as_deref().unwrap();
        let session=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap();
        let before=session.permit.source().payload().to_vec();
        let execution=session.execute(1,53,None,None).unwrap();
        assert_eq!(crate::unit_wire::project_runtime_attachment::saved_runtime_project_payload_json(&execution.run.owned).unwrap().as_bytes(),before);
        let prepared=execution.prepare(&[2]).unwrap();
        let baseline_owned=prepared.run.owned.clone();
        let authored=baseline_owned.project().save().unwrap();
        let baseline=run_general_input_prefix_from_owned(baseline_owned,&authored,2,53,None,None,crate::ddn_runtime::default_age_target()).unwrap();
        assert_eq!(prepared.request.binding().expected_parent,[1]);
        assert_eq!(prepared.request.binding().new_revision,[2]);
        assert!(!prepared.snapshot_bytes().is_empty());
        assert_eq!(prepared.run.execution.next_tick,1);
        assert_eq!(prepared.run.summary["engine_state"]["resources"]["fixed64"]["값"],"1");
        assert_eq!(baseline.summary["engine_state"]["resources"]["fixed64"]["값"],"2");
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let worlds:i64=db.query_row("SELECT count(*) FROM vc04_prepared_payloads WHERE payload_kind=1",[],|r|r.get(0)).unwrap();
        assert_eq!(worlds,0,"candidate/frame generation alone never publishes a world");
        drop(db);
        let persisted=prepared.write_prepared().unwrap();
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let (worlds,custody,head):(i64,i64,Vec<u8>)=db.query_row("SELECT (SELECT count(*) FROM vc04_prepared_payloads WHERE payload_kind=1),(SELECT count(*) FROM vc04_recovery_custody_v1),history_revision_identity FROM vc04_project_heads",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!((worlds,custody),(1,1));assert_eq!(head,[1]);
        assert!(!persisted.artifact.is_empty());
        drop(db);
        let committed=persisted.commit().unwrap();
        assert!(committed.saved_runtime_artifact().unwrap().contains("runtime_attachment"));
        assert!(committed.summary().is_ok());
        std::fs::write(root.join("saved-artifact.json"),committed.saved_runtime_artifact().unwrap()).unwrap();
        std::fs::write(root.join("grant-locator.txt"),locator).unwrap();
        drop(committed);
        // Two non-overlapping risks: custody-prefix loss and retained source
        // corruption. Mutate only this random test DB, then restore exact BLOBs.
        let artifact=std::fs::read_to_string(root.join("saved-artifact.json")).unwrap();
        for target in ["custody","source"] {
            let db=rusqlite::Connection::open(root.join("vc04_snapshot_v2.sqlite3")).unwrap();
            let (select,update)=if target=="custody"{
                ("SELECT row_sha256 FROM vc04_recovery_custody_v1 WHERE journal_ordinal=1","UPDATE vc04_recovery_custody_v1 SET row_sha256=?1 WHERE journal_ordinal=1")
            }else{
                ("SELECT project_payload_bytes FROM vc04_prepared_payloads WHERE payload_kind=0","UPDATE vc04_prepared_payloads SET project_payload_bytes=?1 WHERE payload_kind=0")
            };
            let original:Vec<u8>=db.query_row(select,[],|r|r.get(0)).unwrap();
            let mut changed=original.clone();changed[0]^=1;
            db.execute(update,rusqlite::params![changed]).unwrap();drop(db);
            assert!(resume_first_saved_source_world_native(&artifact,&[1],&[2],&root,locator,1).is_err(),"{target}: RED");
            let db=rusqlite::Connection::open(root.join("vc04_snapshot_v2.sqlite3")).unwrap();
            db.execute(update,rusqlite::params![original]).unwrap();drop(db);
            let restored=resume_first_saved_source_world_native(&artifact,&[1],&[2],&root,locator,1).unwrap();
            assert_eq!(restored["state_hash"],baseline.summary["state_hash"],"{target}: exact restore GREEN");
        }
        let child=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","runtime_surface::native_source_world::tests::vc04_first_world_cold_process_entry","--ignored","--nocapture"])
            .env("DDN_R5_FIRST_WORLD_COLD_ROOT",&root).status().unwrap();
        assert!(child.success());
        let cold:JsonValue=serde_json::from_slice(&std::fs::read(root.join("cold-result.json")).unwrap()).unwrap();
        assert_eq!(cold["state_hash"],baseline.summary["state_hash"]);
        assert_eq!(cold["resources"],baseline.summary["engine_state"]["resources"]);
        assert_eq!(cold["next_tick_id"],2);
        assert_eq!(cold["resources"]["fixed64"]["값"],"2","the saved counter must continue, not restart or stay constant");
        println!("VC04_FIRST_WORLD_NATIVE_COLD_PASS root={} hash={} next_tick=2",root.display(),cold["state_hash"]);
        assert!(begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).is_err());
    }
    #[test]
    #[ignore = "parent starts this in an actual new process against its own random saved root"]
    fn vc04_first_world_cold_process_entry(){
        let root=std::path::PathBuf::from(std::env::var_os("DDN_R5_FIRST_WORLD_COLD_ROOT").unwrap());
        let artifact=std::fs::read_to_string(root.join("saved-artifact.json")).unwrap();
        let locator=std::fs::read_to_string(root.join("grant-locator.txt")).unwrap();
        if std::env::var_os("DDN_R5_LEGACY_ATOM_STRING_REJECT").is_some(){
            // This probe consumes an actual pre-fix source-first saved root,
            // not a lowered source or a fabricated producer/permit. Outer,
            // source registration and protected Native custody still admit.
            use crate::unit_wire::project_runtime_attachment as attachment;
            let checked=attachment::check_bytes(artifact.as_bytes()).unwrap();
            let owned=attachment::restore_saved_runtime_payload(&checked).unwrap();
            assert!(!owned.sources().preparation().source.is_empty());
            let source=attachment::check_saved_source_context(&checked,&owned,&[1],&[2]).unwrap();
            let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
            let admitted=adapter.readmit_first_source_world_source_context(&locator,&source).unwrap();
            let frame=crate::unit_wire::check_resumable_complete_state_layout_v2(&admitted.payload().snapshot,source.pesi().as_raw()).unwrap();
            let catalog=owned.registration().unwrap().saved_nominal_catalog(&owned).unwrap();
            let tables=frame.world.decode_value_tables_with_catalog(Some(&catalog)).unwrap();
            assert_eq!(tables.resources.get("이름표"),Some(&ddonirang_core::platform::ResourceValue::String("빨강".into())));
            drop(admitted);drop(adapter);
            let head=||{
                let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
                db.query_row("SELECT history_revision_identity FROM vc04_project_heads",[],|row|row.get::<_,Vec<u8>>(0)).unwrap()
            };
            let before=head();
            let error=resume_first_saved_source_world_native(&artifact,&[1],&[2],&root,&locator,1).unwrap_err();
            assert!(error.starts_with("E_VC04_COMPLETE_RESOURCE_BINDING:"),"{error}");
            assert_eq!(head(),before);
            assert_eq!(std::fs::read_to_string(root.join("saved-artifact.json")).unwrap(),artifact);
            println!("VC04_ACTUAL_LEGACY_ATOM_STRING_REJECT root={} reason={error}",root.display());
            return;
        }
        let result=resume_first_saved_source_world_native(&artifact,&[1],&[2],&root,&locator,1).unwrap();
        std::fs::write(root.join("cold-result.json"),serde_json::to_vec(&result).unwrap()).unwrap();
    }
    #[test]
    fn vc04_first_world_vm_failure_does_not_undo_consumed_first_use(){
        let (root,_,saved)=fixture("명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 값 <- 1 / 0. }.");
        let locator=saved.grant_locator.as_deref().unwrap();
        let session=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap();
        assert!(session.execute(1,53,None,None).is_err());
        assert!(begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).is_err());
    }
    #[test]
    fn vc04_first_world_cas_failure_preserves_original_disposition_and_no_publication(){
        use crate::snapshot_v2_durable_transaction::{native,SnapshotV2TransactionStateV1 as State,SnapshotV2DurabilityFailureV1 as Failure};
        for (phase,state,cause,expected_head) in [(1,State::Aborted,Failure::AtomicCommitFailed,1u8),(2,State::RecoveryRequired,Failure::ContainerDurabilityFailed,2)]{
            let (root,_,saved)=fixture("명찰:이름씨 := { 이름:글. }. 채비 { 이름표:명찰 := (이름=\"철수\")인 명찰. }. 실행:움직씨 := { 이름표 보여주기. }.");
            let locator=saved.grant_locator.as_deref().unwrap();
            let session=begin_first_authored_source_native(&root,&saved.project_identity,locator,&[1],&saved.revision_integrity).unwrap();
            let prepared=session.execute(1,53,None,None).unwrap().prepare(&[2]).unwrap().write_prepared().unwrap();
            native::inject_source_world_fault(phase);
            let retained=prepared.commit();native::inject_source_world_fault(0);
            let retained=retained.unwrap();
            let failure=retained.retained.outcome().failure().unwrap();
            assert_eq!(failure.state,Some(state));assert_eq!(failure.cause,cause);
            assert!(retained.saved_runtime_artifact().is_err());assert!(retained.summary().is_err());
            let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
            let (head,count):(Vec<u8>,i64)=db.query_row("SELECT history_revision_identity,(SELECT count(*) FROM vc04_recovery_custody_v1) FROM vc04_project_heads",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
            assert_eq!(head,[expected_head]);assert_eq!(count,3);
            let artifact=retained.artifact.clone();drop(db);drop(retained);
            assert!(resume_first_saved_source_world_native(&artifact,&[1],&[2],&root,locator,1).is_err());
        }
    }
}

#[cfg(test)]
mod wo22_crash_tests;
#[cfg(all(test,windows))]
mod authoring_crash_tests;
