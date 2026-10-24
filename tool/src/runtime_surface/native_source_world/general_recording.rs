//! Opt-in ordinary first checkpoint with same-transaction replay children.
//! The legacy save route is retained; this route never invents normal End.
use super::*;
use crate::unit_wire::project_runtime_attachment as attachment;
/// Constructed here only after actual current Native admission and original
/// replay. This binds existing bytes; it is not a source-first-use permit.
pub(crate) struct GeneralOriginalSeedAdmissionV1{
    seed:Vec<u8>,source:attachment::source_only::PreparedSourceOnlyRevision,
}
impl GeneralOriginalSeedAdmissionV1{
    fn from_current(transport:&crate::snapshot_v2_durable_transaction::native::NativeReplayProgressTransportV1,
        replay:&crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1,
        context:&attachment::SourceBoundSavedRuntimeAttachment<'_>,
        adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1)->Result<Self,String>{
        transport.revalidate(adapter).map_err(|cause|format!("E_GENERAL_RECORDING_CURRENT: {cause:?}"))?;
        let bundle=crate::replay_evidence::ReplayBundleCandidateV1::parse(transport.bundle())?;
        let source=attachment::source_only::PreparedSourceOnlyRevision::from_artifact(
            std::str::from_utf8(bundle.payload(1).ok_or("E_GENERAL_REPLAY_SOURCE")?).map_err(|_|"E_GENERAL_REPLAY_SOURCE")?)?;
        if source.pesi()!=context.pesi().as_raw()||replay.source_pesi()!=source.pesi()
            ||bundle.payload(6)!=Some(replay.checked_result_bytes()){
            return Err("E_GENERAL_RECORDING_SOURCE_CHANGED".into());
        }
        let seed=bundle.payload(2).ok_or("E_GENERAL_REPLAY_SEED")?.to_vec();
        Ok(Self{seed,source})
    }
    pub(crate) fn check_seed_candidate(&self,candidate:&[u8])->Result<&[u8],String>{
        if candidate!=self.seed{return Err("E_GENERAL_REPLAY_SEED_CHANGED: 원 시작과 정상 복원의 최초 경계가 다릅니다".into());}
        Ok(&self.seed)
    }
    pub(crate) fn checked_seed_at_actual_zero(&self,
        owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
        runner:&DdnRunner,base:u64,age:AgeTarget)->Result<crate::replay_evidence::ReplaySeedV1,String>{
        let zero=crate::replay_evidence::ReplaySeedV1::capture_general(
            super::super::general_result::InitialGeneralReplaySeedBoundaryV1{owned,runner,source:&self.source,base,age})?;
        crate::replay_evidence::ReplaySeedV1::retain_general_original(self,zero.canonical_bytes())
    }
}

pub(crate) struct NativeGeneralRecordingCurrentV1{
    run:VerifiedGeneralNativeRun,
    transport:crate::snapshot_v2_durable_transaction::native::NativeReplayProgressTransportV1,
    admitted:crate::snapshot_v2_durable_transaction::native::ReadmittedFirstSourceWorldV2,
}
impl NativeGeneralRecordingCurrentV1{
    pub(crate) fn restore(artifact:&str,parent:&[u8],revision:&[u8],
        adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,locator:&str)->Result<Self,String>{
        let reject=|cause|format!("E_GENERAL_RECORDING_NORMAL_RESTORE: {cause:?}");
        let bytes=attachment::check_bytes(artifact.as_bytes())?;
        let owned=attachment::restore_saved_runtime_payload(&bytes)?;
        let source=attachment::check_saved_source_context(&bytes,&owned,parent,revision)?;
        let admitted=adapter.readmit_current_source_world_source_context(locator,&source).map_err(reject)?;
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&admitted.payload().snapshot,source.pesi().as_raw()).map_err(reject)?;
        let context=attachment::check_saved_revision_context_with_frame(&bytes,&owned,parent,revision,layout.world)?;
        let checkpoint=admitted.replay_checkpoint_proof(adapter,&context).map_err(reject)?;
        let transport=adapter.load_current_replay_progress_transport(checkpoint).map_err(reject)?;
        let replay=crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1::check_running(transport.bundle())?;
        let seed=GeneralOriginalSeedAdmissionV1::from_current(&transport,&replay,&context,adapter)?;
        let history=|index|admitted.historical_payload(adapter,index,&context).map_err(reject);
        // Actual Native normal reconstruction remains the permission owner.
        // The read-only staging kernel above cannot be extracted or adopted.
        let (run,expected)=rebuild_general_native_history_with_evidence(adapter,admitted.payload(),&context,Some(&history),
            Some(GeneralEvidenceSourceV1::Retained(&seed)))?;
        let actual=crate::replay_evidence::ResultPreimageV1::capture_general(run.borrow_general_result_boundary()?)?;
        let (contract,inputs,count,receipts)=run.borrow_general_result_boundary()?.transport_parts()?;
        let bundle=crate::replay_evidence::ReplayBundleCandidateV1::parse(transport.bundle())?;
        let expected_bytes=attachment::check_bytes(expected.as_bytes())?;
        let exact=attachment::check_saved_revision_context_with_frame(&expected_bytes,&owned,parent,revision,layout.world)?;
        if exact.runtime_state()!=context.runtime_state()||exact.resource_snapshot()!=context.resource_snapshot()
            ||exact.revision_envelope()!=context.revision_envelope()
            ||actual.canonical_bytes()!=replay.checked_result_bytes()||count!=replay.accepted_count()
            ||bundle.payload(3)!=Some(contract.as_slice())||bundle.payload(4)!=Some(inputs.as_slice())
            ||bundle.payload(8)!=Some(receipts.as_slice()){
            return Err("E_GENERAL_RECORDING_NORMAL_RESULT: 정상 복원과 원 기록 결과가 다릅니다".into());
        }
        transport.revalidate(adapter).map_err(reject)?;
        Ok(Self{run,transport,admitted})
    }
    pub(crate) fn result_identity(&self,adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1)->Result<String,String>{
        self.transport.revalidate(adapter).map_err(|cause|format!("E_GENERAL_RECORDING_CURRENT: {cause:?}"))?;
        crate::replay_evidence::ResultPreimageV1::capture_general(self.run.borrow_general_result_boundary()?).map(|value|value.digest_label())
    }
    /// Consume the normal owner: a failed staged continuation cannot later be
    /// reused as a successful old VM. Existing durable state is never reset.
    pub(crate) fn save_next(self,artifact:&str,parent:&[u8],revision:&[u8],next:&[u8],
        adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,ticks:u32,
        stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)
        ->Result<(String,crate::snapshot_v2_durable_transaction::native::RetainedFirstSourceWorldV2,JsonValue),String>{
        if ticks==0{return Err("E_GENERAL_RECORDING_CHECKPOINT_UNCHANGED: 새 마디 없이 새 세계 거래를 만들지 않습니다".into());}
        let reject=|cause|format!("E_GENERAL_RECORDING_NEXT: {cause:?}");
        self.transport.revalidate(adapter).map_err(reject)?;
        let bytes=attachment::check_bytes(artifact.as_bytes())?;
        let owned=attachment::restore_saved_runtime_payload(&bytes)?;
        let source=attachment::check_saved_source_context(&bytes,&owned,parent,revision)?;
        // Keep the exact pinned Native admission from restore. Re-pinning a
        // second owner would invalidate this live transport's custody lease.
        let admitted=self.admitted;
        let payload=admitted.payload().clone();
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&payload.snapshot,source.pesi().as_raw()).map_err(reject)?;
        let context=attachment::check_saved_revision_context_with_frame(&bytes,&owned,parent,revision,layout.world)?;
        let mut run=self.run;
        if crate::unit_wire::resumable_complete_state::encode_source_capture_supply(run.owned.sources()).map_err(reject)?
            !=crate::unit_wire::resumable_complete_state::encode_source_capture_supply(owned.sources()).map_err(reject)?{
            return Err("E_GENERAL_RECORDING_NEXT_SOURCE".into());
        }
        run.receipts.push(admitted.current_receipt(adapter,&context).map_err(reject)?);
        let result=run.execution.result.take();
        let (summary,execution,rows)=resume_saved_project_artifact_with_general_result(artifact,parent,revision,ticks,
            Some((run.execution.runner,run.age)),Some(layout.world),result)?;
        run.execution=execution;run.rows.extend(rows);
        self.transport.revalidate(adapter).map_err(reject)?;
        let artifact=run.execution.export_continued_artifact_native_complete(&owned,revision,next)?;
        let world=crate::unit_wire::resumable_complete_state::encode_owned_world_candidate_with_source(
            run.execution.world.world(),context.pesi().as_raw(),run.execution.next_tick,&owned).map_err(reject)?;
        let frame=crate::unit_wire::check_complete_state_frame_v1(&world,context.pesi().as_raw()).map_err(reject)?;
        let next_bytes=attachment::check_bytes(artifact.as_bytes())?;
        let next_context=attachment::check_saved_revision_context_with_frame(&next_bytes,&owned,revision,next,frame)?;
        let request=ActualPreparedRequestV1::from_general_continuation(&run,&next_context,revision,next)?;
        let plan=crate::replay_evidence::PlannedNextReplayCheckpointV1::from_general_prepare(
            &run,&self.transport,&request,&context,&next_context,adapter,&admitted)?;
        stage(&artifact,&request.payload().digest)?;
        let (row,pending)=plan.begin_pending(adapter)?;
        let start=admitted.start_next_source_world(adapter,&context,&next_context,request).map_err(reject)?;
        let prepared=adapter.write_next_source_world_prepared(start,&next_context).map_err(reject)?;
        let retained=prepared.commit_with_replay(adapter,&next_context,&plan,&row)?;
        if let Some(cause)=retained.outcome().failure(){return Err(format!("E_GENERAL_RECORDING_NEXT_NOT_COMMITTED: {cause:?}"));}
        plan.complete_ready(adapter,&row,pending,&retained,&next_context)?;
        Ok((artifact,retained,summary))
    }
}
/// Current protected archive admission and data-only original replay. This
/// does not expose a VM, a normal continuation owner or a successful End.
#[cfg(windows)]
pub(crate) fn check_current_general_recording(
    artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,locator:&str,
)->Result<crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1,String>{
    read_current_general_recording_content(artifact,parent,revision,root,locator).map(|checked|checked.comparison)
}
/// Original content only, never a Native normal restore capability. The
/// protected admission is rechecked before copying bytes out of its lease.
pub struct NativeGeneralReplayContentV1{
    bundle:Vec<u8>,comparison:crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1,
}
impl NativeGeneralReplayContentV1{
    pub fn bundle_bytes(&self)->&[u8]{&self.bundle}
    pub fn report_json(&self)->String{self.comparison.report_json()}
}
pub fn read_current_general_recording_content(
    artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,locator:&str,
)->Result<NativeGeneralReplayContentV1,String>{
    let reject=|cause|format!("E_GENERAL_RECORDING_COLD: {cause:?}");
    let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(root).map_err(reject)?;
    read_current_general_recording_content_with_adapter(artifact,parent,revision,&mut adapter,locator)
}
pub(crate) fn read_current_general_recording_content_with_adapter(
    artifact:&str,parent:&[u8],revision:&[u8],
    adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,locator:&str,
)->Result<NativeGeneralReplayContentV1,String>{
    let reject=|cause|format!("E_GENERAL_RECORDING_COLD: {cause:?}");
    let bytes=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&bytes)?;
    let source=attachment::check_saved_source_context(&bytes,&owned,parent,revision)?;
    let admitted=adapter.readmit_current_source_world_source_context(locator,&source).map_err(reject)?;
    let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&admitted.payload().snapshot,source.pesi().as_raw()).map_err(reject)?;
    let context=attachment::check_saved_revision_context_with_frame(&bytes,&owned,parent,revision,layout.world)?;
    let checkpoint=admitted.replay_checkpoint_proof(adapter,&context).map_err(reject)?;
    let transport=adapter.load_current_replay_progress_transport(checkpoint).map_err(reject)?;
    let replay=crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1::check_running(transport.bundle())?;
    if replay.source_pesi()!=source.pesi().as_raw(){return Err("E_GENERAL_RECORDING_COLD_SOURCE".into());}
    transport.revalidate(adapter).map_err(reject)?;
    let mut bundle=Vec::new();bundle.try_reserve_exact(transport.bundle().len()).map_err(|_|"E_GENERAL_RECORDING_CONTENT_LIMIT")?;
    bundle.extend_from_slice(transport.bundle());
    Ok(NativeGeneralReplayContentV1{bundle,comparison:replay})
}
pub struct NativeGeneralRecordingPrepared{
    inner:NativeFirstSourceWorldPrepared,plan:crate::replay_evidence::PlannedFirstReplayCheckpointV1,
}
impl NativeFirstSourceWorldExecution{
    pub fn prepare_recording(self,revision:&[u8])->Result<NativeGeneralRecordingPrepared,String>{
        let inner=self.prepare(revision)?;
        let checked=attachment::check_bytes(inner.artifact.as_bytes())?;
        let binding=inner.request.binding();let snapshot=&inner.request.payload().snapshot;
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(snapshot,&binding.pesi)
            .map_err(|cause|format!("E_GENERAL_RECORDING_FRAME: {cause:?}"))?;
        let context=attachment::check_saved_revision_context_with_frame(&checked,&inner.run.owned,
            &binding.expected_parent,&binding.new_revision,layout.world)?;
        let scope=inner.session.permit.replay_archive_scope(&inner.session.adapter)
            .map_err(|cause|format!("E_GENERAL_RECORDING_SCOPE: {cause:?}"))?;
        let plan=crate::replay_evidence::PlannedFirstReplayCheckpointV1::from_general_source_prepare(
            &inner.run,inner.session.permit.source(),&inner.request,&context,&inner.session.adapter,scope)?;
        Ok(NativeGeneralRecordingPrepared{inner,plan})
    }
}
impl NativeGeneralRecordingPrepared{
    pub fn stage_restore_cache(&self,stage:impl FnOnce(&str,&[u8;32])->Result<(),String>)->Result<(),String>{
        self.inner.stage_restore_cache(stage)
    }
    pub fn commit(self)->Result<NativeFirstSourceWorldRetained,String>{
        let Self{inner,plan}=self;
        let NativeFirstSourceWorldPrepared{mut session,run,artifact,request}=inner;
        let checked=attachment::check_bytes(artifact.as_bytes())?;
        let binding=request.binding().clone();let snapshot=request.payload().snapshot.clone();
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&snapshot,&binding.pesi)
            .map_err(|cause|format!("E_GENERAL_RECORDING_FRAME: {cause:?}"))?;
        let context=attachment::check_saved_revision_context_with_frame(&checked,&run.owned,
            &binding.expected_parent,&binding.new_revision,layout.world)?;
        let (row,pending)=plan.begin_pending(&mut session.adapter)?;
        let start=session.permit.start_first_world_from_saved_source(&session.adapter,&context,request)
            .map_err(|cause|format!("E_GENERAL_RECORDING_WORLD_PENDING: {cause:?}"))?;
        let prepared=session.adapter.write_first_source_world_prepared(start,&context)
            .map_err(|cause|format!("E_GENERAL_RECORDING_WORLD_PREPARED: {cause:?}"))?;
        let retained=prepared.commit_with_replay(&session.adapter,&context,&plan,&row)?;
        if let Some(cause)=retained.outcome().failure(){return Err(format!("E_GENERAL_RECORDING_WORLD_NOT_COMMITTED: {cause:?}"));}
        plan.complete_ready(&mut session.adapter,&row,pending,&retained,&context)?;
        Ok(NativeFirstSourceWorldRetained{_adapter:session.adapter,run,artifact,retained,latest_summary:None})
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use sha2::Digest;
    fn prepared()->(std::path::PathBuf,NativeSourceRevisionReadback,NativeGeneralRecordingPrepared,Vec<u8>,Vec<u8>){
        let (root,_,saved)=crate::runtime_surface::native_source_world::tests::fixture(
            "채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.");
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),
            &saved.revision_identity,&saved.revision_integrity).unwrap().execute(2,7,None,None).unwrap();
        let seed=execution.run.original_general_replay_seed().unwrap().canonical_bytes().to_vec();
        let result=crate::replay_evidence::ResultPreimageV1::capture_general(execution.run.borrow_general_result_boundary().unwrap()).unwrap().canonical_bytes().to_vec();
        let prepared=execution.prepare_recording(&[2]).unwrap();(root,saved,prepared,seed,result)
    }
    #[test]
    fn b3_general_actual_first_world_and_original_evidence_commit_together(){
        let (root,saved,prepared,seed,result)=prepared();
        let expected=prepared.inner.run.summary["state_hash"].clone();
        let retained=prepared.commit().unwrap();
        assert_eq!(retained.summary().unwrap()["state_hash"],expected);
        let artifact=retained.saved_runtime_artifact().unwrap().to_owned();
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let (count,state,ordinal):(i64,i64,i64)=db.query_row("SELECT count(*),recording_state,control_ordinal FROM wo23_replay_controls_v1",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
        assert_eq!((count,state,ordinal),(1,0,0),"saving a live boundary must not fabricate Finished");
        let mut statement=db.prepare("SELECT chunk_bytes FROM wo23_replay_chunks_v1 ORDER BY chunk_ordinal").unwrap();
        let bundle_bytes=statement.query_map([],|row|row.get::<_,Vec<u8>>(0)).unwrap().map(Result::unwrap).flatten().collect::<Vec<_>>();
        let bundle=crate::replay_evidence::ReplayBundleCandidateV1::parse(&bundle_bytes).unwrap();
        assert_eq!(bundle.payload(2),Some(seed.as_slice()));assert_eq!(bundle.payload(6),Some(result.as_slice()));
        assert_eq!(bundle.payload(3).unwrap()[0],3);assert_eq!(bundle.payload(4).unwrap()[0],2);
        assert_eq!(bundle.payload(8).unwrap()[..8],2u64.to_be_bytes());
        let replay=crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1::check_running(&bundle_bytes).unwrap();
        assert_eq!(replay.checked_result_bytes(),result);assert_eq!(replay.accepted_count(),2);
        assert_eq!(replay.source_pesi(),&saved.pesi);
        // Data-level mutations only; native custody/outer-checksum mutation
        // admission is a distinct gate and is not claimed by these checks.
        let parsed_seed=crate::replay_evidence::ReplaySeedCandidateV1::parse(bundle.payload(2).unwrap()).unwrap();
        let mode=parsed_seed.payload(14).unwrap().as_ptr() as usize-bundle_bytes.as_ptr() as usize+2;
        let mut changed=bundle_bytes.clone();changed[mode]=3;
        assert!(crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1::check_running(&changed).is_err());
        let state=bundle.payload(11).unwrap().as_ptr() as usize-bundle_bytes.as_ptr() as usize;
        let mut changed=bundle_bytes.clone();changed[state]=1;
        assert!(crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1::check_running(&changed).is_err());
        assert!(crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1::check_running(&bundle_bytes).is_ok());
        drop(statement);drop(db);drop(retained);
        let checked=check_current_general_recording(&artifact,&[1],&[2],&root,saved.grant_locator.as_deref().unwrap()).unwrap();
        assert_eq!(checked.checked_result_bytes(),result);
        assert_eq!(checked.accepted_count(),2);
        let data=read_current_general_recording_content(&artifact,&[1],&[2],&root,saved.grant_locator.as_deref().unwrap()).unwrap();
        assert_eq!(data.bundle_bytes(),bundle_bytes);
        let reported:serde_json::Value=serde_json::from_str(&data.report_json()).unwrap();
        assert_eq!(reported["accepted_count_decimal"],"2");assert_eq!(reported["status"],"RUNNING_RESULT_COMPARED");
        assert_eq!(reported["result_identity"],format!("sha256:{}",hex::encode(sha2::Sha256::digest(&result))));
        let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
        let normal_recording=NativeGeneralRecordingCurrentV1::restore(&artifact,&[1],&[2],&mut adapter,saved.grant_locator.as_deref().unwrap()).unwrap();
        assert_eq!(normal_recording.run.execution.next_tick,2);
        assert_eq!(normal_recording.result_identity(&adapter).unwrap(),format!("sha256:{}",hex::encode(sha2::Sha256::digest(&result))));
        drop(normal_recording);drop(adapter);
        let cold=resume_first_saved_source_world_native(&artifact,&[1],&[2],&root,saved.grant_locator.as_deref().unwrap(),0).unwrap();
        assert_eq!(cold["state_hash"],expected,"basic normal reopening is not ReplayVerified");
    }
    #[test]
    fn b3_general_retained_original_reader_then_save_and_cold_read(){
        let (root,saved,prepared,seed,_)=prepared();
        let locator=saved.grant_locator.as_deref().unwrap();
        let mut retained=prepared.commit().unwrap();
        let first=retained.recording_content(&[1],&[2],locator).unwrap();
        let first_report:serde_json::Value=serde_json::from_str(&first.report_json()).unwrap();
        assert_eq!(first_report["accepted_count_decimal"],"2");
        let summary=retained.save_continuation(&[1],&[2],&[3],locator,2,|_,_|Ok(())).unwrap();
        let second=retained.recording_content(&[2],&[3],locator).unwrap();
        let second_report:serde_json::Value=serde_json::from_str(&second.report_json()).unwrap();
        assert_eq!(second_report["accepted_count_decimal"],"4");
        assert_eq!(second_report["status"],"RUNNING_RESULT_COMPARED");
        let bundle=crate::replay_evidence::ReplayBundleCandidateV1::parse(second.bundle_bytes()).unwrap();
        assert_eq!(bundle.payload(2),Some(seed.as_slice()),"reader cannot replace the original start");
        let artifact=retained.saved_runtime_artifact().unwrap().to_owned();
        drop(retained);
        let cold=read_current_general_recording_content(&artifact,&[2],&[3],&root,locator).unwrap();
        assert_eq!(cold.bundle_bytes(),second.bundle_bytes());
        let resumed=resume_first_saved_source_world_native(&artifact,&[2],&[3],&root,locator,0).unwrap();
        for key in ["state_hash","resources"]{assert_eq!(resumed[key],summary[key]);}
    }
    #[test]
    fn b3_general_cache_failure_before_pending_preserves_source_head_and_has_no_child_success(){
        let (root,_,prepared,_,_)=prepared();
        assert!(prepared.stage_restore_cache(|_,_|Err("injected pre-Pending cache failure".into())).is_err());
        drop(prepared);
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let (kind,head):(i64,Vec<u8>)=db.query_row("SELECT payload_kind,history_revision_identity FROM vc04_project_heads",[],|row|Ok((row.get(0)?,row.get(1)?))).unwrap();
        assert_eq!((kind,head),(0,vec![1]));
        let tables:i64=db.query_row("SELECT count(*) FROM sqlite_schema WHERE name='wo23_replay_controls_v1'",[],|row|row.get(0)).unwrap();
        if tables!=0{assert_eq!(db.query_row("SELECT count(*) FROM wo23_replay_controls_v1",[],|row|row.get::<_,i64>(0)).unwrap(),0);}
    }
    #[test]
    fn b3_general_legacy_checkpoint_opens_but_cannot_backfill_original_result(){
        let (root,_,saved)=crate::runtime_surface::native_source_world::tests::fixture(
            "채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.");
        let prepared=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),
            &saved.revision_identity,&saved.revision_integrity).unwrap().execute(2,7,None,None).unwrap().prepare(&[2]).unwrap();
        // Deliberately take the pre-existing save route. A source or a normal
        // restore succeeding must not mint missing archive evidence later.
        let retained=prepared.write_prepared().unwrap().commit().unwrap();
        let artifact=retained.saved_runtime_artifact().unwrap().to_owned();
        let expected=retained.summary().unwrap()["state_hash"].clone();drop(retained);
        assert!(check_current_general_recording(&artifact,&[1],&[2],&root,saved.grant_locator.as_deref().unwrap()).is_err());
        let normal=resume_first_saved_source_world_native(&artifact,&[1],&[2],&root,saved.grant_locator.as_deref().unwrap(),0).unwrap();
        assert_eq!(normal["state_hash"],expected);
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let count:i64=db.query_row("SELECT count(*) FROM sqlite_schema WHERE name='wo23_replay_controls_v1'",[],|row|row.get(0)).unwrap();
        if count!=0{assert_eq!(db.query_row("SELECT count(*) FROM wo23_replay_controls_v1",[],|row|row.get::<_,i64>(0)).unwrap(),0);}
    }
    #[test]
    fn b3_general_original_replay_preserves_registered_subject_atom_and_fraction_tuple(){
        let source="명찰:이름씨 := { 이름:글. }. 병사:임자 := { 이름:글. }. 채비 { 이름표:(나눔수,명찰) := (1 / 3,(이름=\"처음\")인 명찰). 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 표적 <- (이름=\"철수\")인 병사. 이름표 보여주기. #표지 보여주기. }. }.";
        let (root,_,saved)=crate::runtime_surface::native_source_world::tests::fixture(source);
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),
            &saved.revision_identity,&saved.revision_integrity).unwrap().execute(2,7,None,None).unwrap();
        let expected=crate::replay_evidence::ResultPreimageV1::capture_general(execution.run.borrow_general_result_boundary().unwrap()).unwrap().canonical_bytes().to_vec();
        let retained=execution.prepare_recording(&[2]).unwrap().commit().unwrap();
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let mut statement=db.prepare("SELECT chunk_bytes FROM wo23_replay_chunks_v1 ORDER BY chunk_ordinal").unwrap();
        let bytes=statement.query_map([],|row|row.get::<_,Vec<u8>>(0)).unwrap().map(Result::unwrap).flatten().collect::<Vec<_>>();
        let replay=crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1::check_running(&bytes).unwrap();
        assert_eq!(replay.checked_result_bytes(),expected);assert_eq!(replay.source_pesi(),&saved.pesi);
        drop(statement);drop(db);drop(retained);
    }
    #[test]
    fn b3_general_readonly_preserves_atom_world_without_claiming_atom_output_type(){
        // Distinct coverage: atom world canon is already assigned; atom
        // output TypeIdentity above stays a preserved, blocked positive gate.
        let source="명찰:이름씨 := { 이름:글. }. 병사:임자 := { 이름:글. }. 채비 { 이름표:(나눔수,명찰) := (1 / 3,(이름=\"처음\")인 명찰). 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 표적 <- (이름=\"철수\")인 병사. 표지 <- #표지. 이름표 보여주기. }. }.";
        let (root,_,saved)=crate::runtime_surface::native_source_world::tests::fixture(source);
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),
            &saved.revision_identity,&saved.revision_integrity).unwrap().execute(2,7,None,None).unwrap();
        let expected=crate::replay_evidence::ResultPreimageV1::capture_general(execution.run.borrow_general_result_boundary().unwrap()).unwrap().canonical_bytes().to_vec();
        let retained=execution.prepare_recording(&[2]).unwrap().commit().unwrap();
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let mut statement=db.prepare("SELECT chunk_bytes FROM wo23_replay_chunks_v1 ORDER BY chunk_ordinal").unwrap();
        let bytes=statement.query_map([],|row|row.get::<_,Vec<u8>>(0)).unwrap().map(Result::unwrap).flatten().collect::<Vec<_>>();
        let replay=crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1::check_running(&bytes).unwrap();
        assert_eq!(replay.checked_result_bytes(),expected);assert_eq!(replay.source_pesi(),&saved.pesi);
        drop(statement);drop(db);drop(retained);
    }
    #[test]
    fn b3_general_cold_recording_three_plus_four_preserves_original_seed_and_exact_result(){
        let (root,_,saved)=crate::runtime_surface::native_source_world::tests::fixture(
            "병사:임자 := { 이름:글. }. 채비 { 횟수:바른수 := 0. 값:(나눔수,바른수) := (1 / 3,2). }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 표적 <- (이름=\"철수\")인 병사. 표지 <- #표지. 값 보여주기. 횟수 보여주기. }. }.");
        let locator=saved.grant_locator.as_deref().unwrap();
        let execution=begin_first_authored_source_native(&root,&saved.project_identity,locator,
            &saved.revision_identity,&saved.revision_integrity).unwrap().execute(3,7,None,None).unwrap();
        // Same original source/zero subject nonce and actual inputs; this
        // staging reference is not a second Native first-use permission.
        let reference=run_general_input_prefix_from_owned_with_subject(execution.run.owned.clone(),&execution.run.owned.project().save().unwrap(),
            7,7,None,None,execution.run.construction.age,execution.run.construction.subject_bootstrap.as_ref(),Some(execution.session.permit.source())).unwrap();
        let expected=crate::replay_evidence::ResultPreimageV1::capture_general(reference.borrow_general_result_boundary().unwrap()).unwrap();
        let seed=execution.run.original_general_replay_seed().unwrap().canonical_bytes().to_vec();
        let retained=execution.prepare_recording(&[2]).unwrap().commit().unwrap();
        let artifact=retained.saved_runtime_artifact().unwrap().to_owned();drop(retained);
        let mut adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
        let current=NativeGeneralRecordingCurrentV1::restore(&artifact,&[1],&[2],&mut adapter,locator).unwrap();
        let failure=current.save_next(&artifact,&[1],&[2],&[3],&mut adapter,4,|_,_|Err("injected cache failure before Pending".into())).err().unwrap();
        assert!(failure.contains("injected cache failure before Pending"),"unexpected earlier failure: {failure}");
        drop(adapter);
        let mut continued=None;
        let summary=save_continued_source_world_native(&artifact,&[1],&[2],&[3],&root,locator,4,|artifact,_|{continued=Some(artifact.to_owned());Ok(())}).unwrap();
        let continued=continued.unwrap();
        assert_eq!(summary["state_hash"],reference.summary["state_hash"]);
        let mut cold=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
        let actual=NativeGeneralRecordingCurrentV1::restore(&continued,&[2],&[3],&mut cold,locator).unwrap();
        assert_eq!(actual.run.execution.next_tick,7);
        assert_eq!(actual.run.original_general_replay_seed().unwrap().canonical_bytes(),seed);
        assert_eq!(actual.result_identity(&cold).unwrap(),expected.digest_label());
        let bundle=crate::replay_evidence::ReplayBundleCandidateV1::parse(actual.transport.bundle()).unwrap();
        assert_eq!(bundle.payload(8).unwrap()[..8],7u64.to_be_bytes());
        assert_eq!(bundle.payload(11).unwrap()[0],0,"re-saving Running must not fabricate normal End");
    }
}
