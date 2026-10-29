//! Explicit new execution after an admitted edited-source save. No FirstUse
//! reset, old VM transplant, or Run permit reconstructed from saved JSON.
use super::*;
pub struct NativeRestartSourceWorldSession {
    adapter:crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    permit:crate::snapshot_v2_durable_transaction::native::NewSourceRunPermitV1,
}
pub struct NativeRestartSourceWorldExecution {session:NativeRestartSourceWorldSession,run:GeneralSuccessfulInputPrefix}
pub struct NativeRestartSourceWorldPrepared {session:NativeRestartSourceWorldSession,run:GeneralSuccessfulInputPrefix,artifact:String,request:ActualPreparedRequestV1}

pub fn begin_new_authored_source_native(root:&std::path::Path,project:&[u8],locator:&str,artifact:&str,selection:&NativeAuthoringSelection)
    ->Result<NativeRestartSourceWorldSession,String>{
    use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,AuthoringHeadV1};
    if selection.project!=project||selection.kind!=0{return Err("E_SOURCE_NEW_RUN_SELECTION".into());}
    let mut adapter=SqliteAdapterV1::open_source_revision(root).map_err(|e|format!("E_SOURCE_NEW_RUN_OPEN: {e:?}"))?;
    let head=AuthoringHeadV1{project:selection.project.clone(),revision:selection.revision.clone(),integrity:selection.integrity,key:selection.key,kind:selection.kind};
    let base=adapter.admit_authoring_base(locator,&head,artifact).map_err(|e|format!("E_SOURCE_NEW_RUN_ADMISSION: {e:?}"))?;
    let permit=base.consume_new_run(&adapter).map_err(|e|format!("E_SOURCE_NEW_RUN_CONSUME: {e:?}"))?;
    Ok(NativeRestartSourceWorldSession{adapter,permit})
}
impl NativeRestartSourceWorldSession {
    pub(crate) fn execute_logical(self,input:&crate::logical_input_tape::LogicalRecordRequest)->Result<NativeRestartLogicalExecution,String>{
        self.permit.verify_before_world(&self.adapter).map_err(|e|format!("E_SOURCE_NEW_RUN_BEFORE_VM: {e:?}"))?;
        let owned=self.permit.source().recorded_execution_owner()?;
        let mut run=crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1::fresh(&owned,input.binding.clone(),crate::ddn_runtime::default_age_target())
            .map_err(|e|format!("E_SOURCE_NEW_LOGICAL_RUN: {e:?}"))?;
        run.capture_initial_replay_seed(self.permit.source())?;
        let mut run=crate::replay_evidence::LogicalRecordingRunV1::begin(run)?;
        for snapshot in &input.snapshots{run.step(snapshot)?;}
        self.permit.verify_before_world(&self.adapter).map_err(|e|format!("E_SOURCE_NEW_RUN_AFTER_VM: {e:?}"))?;
        Ok(NativeRestartLogicalExecution{session:self,owned,run})
    }
    pub fn execute(self,ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,input_tape:Option<&InputTape>)->Result<NativeRestartSourceWorldExecution,String>{
        self.permit.verify_before_world(&self.adapter).map_err(|e|format!("E_SOURCE_NEW_RUN_BEFORE_VM: {e:?}"))?;
        let owned=self.permit.source().recorded_execution_owner()?;let artifact=owned.project().save()?;
        let subject=crate::unit_wire::subject_execution::SubjectExecutionStage::fresh_for_permitted_source(&owned)?;
        let run=run_general_input_prefix_from_owned_with_subject(owned,&artifact,ticks,base_seed,input_snapshots,input_tape,crate::ddn_runtime::default_age_target(),subject.as_ref(),Some(self.permit.source()))?;
        self.permit.verify_before_world(&self.adapter).map_err(|e|format!("E_SOURCE_NEW_RUN_AFTER_VM: {e:?}"))?;
        Ok(NativeRestartSourceWorldExecution{session:self,run})
    }
}

pub(crate) struct NativeRestartLogicalExecution {
    session:NativeRestartSourceWorldSession,
    owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    run:crate::replay_evidence::LogicalRecordingRunV1,
}
pub(crate) struct NativeRestartLogicalPrepared {execution:NativeRestartLogicalExecution,artifact:String,request:ActualPreparedRequestV1,
    _replay_plan:crate::replay_evidence::PlannedFirstReplayCheckpointV1}
impl NativeRestartLogicalExecution {
    pub(crate) fn prepare(self,revision:&[u8])->Result<NativeRestartLogicalPrepared,String>{
        let parent=self.session.permit.source().revision();
        self.session.permit.verify_before_world(&self.session.adapter).map_err(|e|format!("E_SOURCE_NEW_LOGICAL_PREPARE: {e:?}"))?;
        let artifact=self.run.export_saved_artifact(parent,revision).map_err(|e|format!("E_SOURCE_NEW_LOGICAL_ARTIFACT: {e:?}"))?;
        let checked=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes())?;
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context(&checked,&self.owned,parent,revision)?;
        let request=ActualPreparedRequestV1::from_logical(&self.run,&context,parent,revision).map_err(|e|format!("E_SOURCE_NEW_LOGICAL_REQUEST: {e:?}"))?;
        self.session.permit.check_actual_world_request(&self.session.adapter,&context,&request).map_err(|e|format!("E_SOURCE_NEW_LOGICAL_REQUEST: {e:?}"))?;
        let scope=self.session.permit.replay_archive_scope(&self.session.adapter).map_err(|e|format!("E_SOURCE_NEW_REPLAY_SCOPE: {e:?}"))?;
        let plan=crate::replay_evidence::PlannedFirstReplayCheckpointV1::from_source_prepare(
            &self.run,self.session.permit.source(),&request,&context,&self.session.adapter,scope)?;
        Ok(NativeRestartLogicalPrepared{execution:self,artifact,request,_replay_plan:plan})
    }
}
impl NativeRestartLogicalPrepared {
    pub(crate) fn stage_restore_cache(&self,stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)->Result<(),String>{
        self.execution.session.permit.verify_before_world(&self.execution.session.adapter).map_err(|e|format!("E_SOURCE_NEW_LOGICAL_CACHE: {e:?}"))?;
        stage(&self.artifact,&self.request.payload().digest)
    }
    pub(crate) fn commit(self)->Result<(String,JsonValue),String>{
        let Self{execution,artifact,request,_replay_plan}=self;let NativeRestartLogicalExecution{mut session,owned,run}=execution;
        let checked=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes())?;
        let b=request.binding();
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context(&checked,&owned,&b.expected_parent,&b.new_revision)?;
        let (replay_row,replay_pending)=_replay_plan.begin_pending(&mut session.adapter)?;
        let start=session.permit.start_world(&session.adapter,&context,request).map_err(|e|format!("E_SOURCE_NEW_LOGICAL_PENDING: {e:?}"))?;
        let prepared=session.adapter.write_restart_source_world_prepared(start,&context).map_err(|e|format!("E_SOURCE_NEW_LOGICAL_PREPARED: {e:?}"))?;
        let retained=prepared.commit_with_replay(&session.adapter,&context,&_replay_plan,&replay_row)?;
        if let Some(cause)=retained.outcome().failure(){return Err(format!("E_SOURCE_NEW_LOGICAL_NOT_COMMITTED: {cause:?}"));}
        _replay_plan.complete_ready(&mut session.adapter,&replay_row,replay_pending,&retained,&context)?;
        Ok((artifact,run.session().summary()))
    }
}
impl NativeRestartSourceWorldExecution {
    pub fn prepare(self,revision:&[u8])->Result<NativeRestartSourceWorldPrepared,String>{
        let parent=self.session.permit.source().revision();
        self.session.permit.verify_before_world(&self.session.adapter).map_err(|e|format!("E_SOURCE_NEW_RUN_PREPARE: {e:?}"))?;
        let encoded=self.run.encode_world_frame().map_err(|e|format!("E_SOURCE_NEW_RUN_FRAME: {e:?}"))?;
        let issued=crate::project_snapshot_identity::current::issue_saved_project(&self.run.owned)?;
        let frame=crate::unit_wire::check_complete_state_frame_v1(&encoded,issued.pesi().as_raw()).map_err(|e|format!("E_SOURCE_NEW_RUN_FRAME: {e:?}"))?;
        let catalog=self.run.owned.registration().map(|r|r.saved_nominal_catalog(&self.run.owned)).transpose()?;
        let plan=self.run.execution.runner.saved_tuple_restore_plan();
        let projection=self.run.summary.pointer("/engine_state/state/snapshot_v2/resources").ok_or("E_SOURCE_NEW_RUN_RESOURCES")?;
        let staged=crate::unit_wire::stage_resource_snapshot_with_complete_frame(projection,&frame,plan.as_ref(),catalog.as_ref())?;
        if staged.state_hash()!=self.run.execution.world.state_hash(){return Err("E_SOURCE_NEW_RUN_RESOURCE_HASH".into());}
        let artifact=self.run.export_saved_artifact_native_complete(parent,revision)?;
        let checked=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes())?;
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(&checked,&self.run.owned,parent,revision,frame)?;
        let request=ActualPreparedRequestV1::from_general(&self.run,&context,parent,revision)?;
        self.session.permit.check_actual_world_request(&self.session.adapter,&context,&request).map_err(|e|format!("E_SOURCE_NEW_RUN_REQUEST: {e:?}"))?;
        Ok(NativeRestartSourceWorldPrepared{session:self.session,run:self.run,artifact,request})
    }
}
impl NativeRestartSourceWorldPrepared {
    pub fn stage_restore_cache(&self,stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)->Result<(),String>{
        self.session.permit.verify_before_world(&self.session.adapter).map_err(|e|format!("E_SOURCE_NEW_RUN_CACHE: {e:?}"))?;
        stage(&self.artifact,&self.request.payload().digest)
    }
    pub fn write_prepared(self)->Result<NativeFirstSourceWorldDurablyPrepared,String>{
        let Self{session,run,artifact,request}=self;
        let checked=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes())?;
        let b=request.binding();let snapshot=request.payload().snapshot.clone();let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&snapshot,&b.pesi).map_err(|e|format!("E_SOURCE_NEW_RUN_FRAME: {e:?}"))?;
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(&checked,&run.owned,&b.expected_parent,&b.new_revision,layout.world)?;
        let start=session.permit.start_world(&session.adapter,&context,request).map_err(|e|format!("E_SOURCE_NEW_RUN_PENDING: {e:?}"))?;
        let prepared=session.adapter.write_restart_source_world_prepared(start,&context).map_err(|e|format!("E_SOURCE_NEW_RUN_PREPARED: {e:?}"))?;
        Ok(NativeFirstSourceWorldDurablyPrepared{adapter:session.adapter,run,artifact,prepared})
    }
}
