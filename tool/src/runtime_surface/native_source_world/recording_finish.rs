//! Short-lived Native finish owner. It stays on the host controller thread;
//! request handles never construct a Run, root scope or Finished authority.
use crate::unit_wire::project_runtime_attachment as attachment;
use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,ReadmittedFirstSourceWorldV2};
use crate::unit_wire::resumable_complete_state::NativeRecordingProgressOwnerV1;

/// Explicit cold query/repair. Restores no Running VM/callback and executes no
/// input. The common Native owner must consume the full-custody permit first.
pub(crate) fn recover_native_logical_finish(artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,
    locator:&str,binding:&crate::logical_input_tape::LogicalInputBinding)->Result<String,String>{
    let reject=|cause|format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 원 기록 마치기 확인 ({cause:?})");
    let mut adapter=SqliteAdapterV1::open_source_revision(root).map_err(reject)?;
    let checked=attachment::check_bytes(artifact.as_bytes())?;
    let owned=attachment::restore_saved_runtime_payload(&checked)?;
    let source=attachment::check_saved_source_context(&checked,&owned,parent,revision)?;
    let admitted=adapter.readmit_current_source_world_source_context(locator,&source).map_err(reject)?;
    let context=attachment::check_saved_revision_context(&checked,&owned,parent,revision)?;
    let permit=admitted.replay_checkpoint_proof(&adapter,&context).map_err(reject)?
        .prepare_finish_recovery(&mut adapter,binding,&context).map_err(reject)?;
    let (c01,notice,original_failure)=permit.consume(&mut adapter,&context).map_err(reject)?.into_parts();
    let original=String::from_utf8(c01).map_err(|_|"E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 원 응답 글 인코딩 오류")?;
    // Original C01 remains an exact separate string, not a new receipt or a
    // host-reconstructed summary. Do not add notice/cause inside its two keys.
    Ok(serde_json::json!({"original_output_json_utf8":original,"notice":notice,"original_failure":original_failure}).to_string())
}

pub(crate) struct NativeLogicalFinishSessionV1 {
    adapter:SqliteAdapterV1,admitted:ReadmittedFirstSourceWorldV2,
    owner:NativeRecordingProgressOwnerV1,artifact:String,parent:Vec<u8>,revision:Vec<u8>,
    binding:crate::logical_input_tape::LogicalInputBinding,
    finished_packet:Option<String>,
    no_end:Option<crate::replay_evidence::NativeNoEndStoredV1>,
}

#[cfg(test)]
mod failed_input_tests{
    use super::*;
    #[test]
    fn b3_native_failed_input_requires_actual_runtime_failure_and_preserves_world_anchor(){
        for must_fail in [true,false]{
            let source=if must_fail{
                "채비 { 횟수:셈수 := 0.0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1.0. 값 <- 6.0 / (2.0 - 횟수). 값 보여주기. }. }."
            }else{"실행:움직씨 := { (매마디)마다 { 6.0 보여주기. }. }."};
            let (root,_,saved)=super::super::tests::fixture(source);
            let locator=saved.grant_locator.as_deref().unwrap();
            let initial=crate::runtime_surface::begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap();
            let owned=initial.permit.source().recorded_execution_owner().unwrap();
            let mut input:crate::logical_input_tape::LogicalRecordRequest=serde_json::from_str(include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/initial-input.json")).unwrap();
            input.binding.source_revision=crate::runtime_surface::sha256_identity(owned.sources().preparation().source.as_bytes());input.snapshots.truncate(1);
            let prepared=initial.execute_logical(&input).unwrap().prepare(&[2]).unwrap();
            prepared.stage_restore_cache(|_,_|Ok(())).unwrap();
            let (artifact,before)=prepared.commit().unwrap();
            let reuse_checked=attachment::check_bytes(artifact.as_bytes()).unwrap();
            let reuse_owned=attachment::restore_saved_runtime_payload(&reuse_checked).unwrap();
            let reuse_session=NativeLogicalFinishSessionV1::open(&artifact,&[1],&[2],&root,locator,&input.binding,Some(&reuse_owned)).unwrap();
            let reused_result=reuse_session.result_identity().unwrap();drop(reuse_session);
            let empty=crate::logical_input_tape::LogicalRecordRequest{binding:input.binding.clone(),snapshots:vec![]};
            let (same_summary,same_result)=super::super::logical::resume_logical_source_world_with_result(
                &artifact,&[1],&[2],&root,locator,&empty,Some(&reuse_owned),Some(1)).unwrap();
            assert_eq!(same_summary,before,"consumed actual restore must preserve the complete summary");
            assert_eq!(same_result.unwrap().unwrap(),reused_result,"fresh protected progress/result admission must match standalone open");
            let mut session=NativeLogicalFinishSessionV1::open(&artifact,&[1],&[2],&root,locator,&input.binding,None).unwrap();
            assert_eq!(session.result_identity().unwrap(),reused_result,"request-local source reuse changes no result or authority");
            let bytes=attachment::check_bytes(artifact.as_bytes()).unwrap();
            let owned=attachment::restore_saved_runtime_payload(&bytes).unwrap();
            let context=attachment::check_saved_revision_context(&bytes,&owned,&[1],&[2]).unwrap();
            let receipt=session.admitted.current_receipt(&session.adapter,&context).unwrap();
            let head=session.adapter.selected_authoring_head(&session.admitted.payload().binding.project).unwrap();
            let result=session.owner.result_identity(&session.adapter).unwrap();
            let attempted=crate::logical_input_tape::LogicalSnapshot{madi:1,events:vec![],held:input.snapshots[0].held.clone()};
            let outcome=session.fail_at_input(&input,&attempted,&result);
            if must_fail{
                let actual=outcome.unwrap();assert_eq!(actual["outcome"],"ABORTED");
                assert!(actual["summary"]["failure"].as_str().unwrap().contains("0으로 나눌 수 없습니다"));
                for key in ["state_hash","resources","frames","input_receipts","madi_count"]{assert_eq!(actual["summary"][key],before[key],"failed Native madi changed {key}");}
                assert_eq!(session.stop().unwrap(),actual);assert!(session.owner.prepare_finish(&session.adapter,None).is_err());
            }else{
                assert!(outcome.err().unwrap().contains("E_NATIVE_FAILURE_NOT_REPRODUCED"));
                assert!(session.no_end.is_none(),"host failure claim must not publish a NoEnd row");
            }
            assert_eq!(session.admitted.current_receipt(&session.adapter,&context).unwrap(),receipt);
            assert_eq!(session.adapter.selected_authoring_head(&session.admitted.payload().binding.project).unwrap(),head);
        }
    }
}
enum Admission {CurrentProgress,ExplicitNewRecord}
impl NativeLogicalFinishSessionV1 {
    pub(crate) fn open(artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,
        locator:&str,binding:&crate::logical_input_tape::LogicalInputBinding,
        source_owner:Option<&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>)->Result<Self,String>{
        Self::open_with_admission(artifact,parent,revision,root,locator,binding,Admission::CurrentProgress,None,source_owner)
    }
    pub(crate) fn open_new_record(artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,
        locator:&str,binding:&crate::logical_input_tape::LogicalInputBinding,selection:&crate::runtime_surface::NativeAuthoringSelection,
        source_owner:Option<&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>)->Result<Self,String>{
        Self::open_with_admission(artifact,parent,revision,root,locator,binding,Admission::ExplicitNewRecord,Some(selection),source_owner)
    }
    fn open_with_admission(artifact:&str,parent:&[u8],revision:&[u8],root:&std::path::Path,
        locator:&str,binding:&crate::logical_input_tape::LogicalInputBinding,admission:Admission,
        selection:Option<&crate::runtime_surface::NativeAuthoringSelection>,
        source_owner:Option<&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>)->Result<Self,String>{
        let mut adapter=SqliteAdapterV1::open_source_revision(root).map_err(super::logical::reject)?;
        let checked=attachment::check_bytes(artifact.as_bytes())?;
        // Only reuse an actual typed source owner from this synchronous host
        // request. The original source/capture/registration/envelope binding
        // below still binds it to these exact bytes; this is no ambient cache
        // and no Native world/recording permission is reused or manufactured.
        let restored;
        let owned=match source_owner{Some(owner)=>owner,None=>{
            let _phase=crate::storage_observation::Span::start("logical.finish_saved_source_admission");
            restored=attachment::restore_saved_runtime_payload(&checked)?;&restored
        }};
        let source=attachment::check_saved_source_context(&checked,owned,parent,revision)?;
        let admitted=adapter.readmit_current_source_world_source_context(locator,&source).map_err(super::logical::reject)?;
        let context=attachment::check_saved_revision_context(&checked,owned,parent,revision)?;
        if let Admission::ExplicitNewRecord=admission{
            let expected=selection.ok_or("E_NEW_RECORD_SELECTION_REQUIRED")?;
            let current=adapter.selected_authoring_head(&admitted.payload().binding.project).map_err(super::logical::reject)?;
            if current.project!=expected.project || current.revision!=expected.revision || current.integrity!=expected.integrity
                || current.key!=expected.key || current.kind!=expected.kind{
                return Err("E_NEW_RECORD_SELECTION_CHANGED: 선택한 저장 시점이 바뀌었습니다. 다른 시점으로 대체하지 않습니다".into());
            }
        }
        let owner=match admission{
            Admission::CurrentProgress=>NativeRecordingProgressOwnerV1::restore(&adapter,&admitted,&context,binding)?,
            Admission::ExplicitNewRecord=>NativeRecordingProgressOwnerV1::start_new_record(&mut adapter,&admitted,&context,binding)?,
        };
        Ok(Self{adapter,admitted,owner,artifact:artifact.into(),parent:parent.into(),revision:revision.into(),binding:binding.clone(),finished_packet:None,no_end:None})
    }
    pub(crate) fn register_frontend(&mut self)->Result<serde_json::Value,String>{
        self.owner.register_finish_frontend(&self.adapter)
    }
    pub(crate) fn result_identity(&self)->Result<serde_json::Value,String>{
        Ok(serde_json::json!({"result_identity":self.owner.result_identity(&self.adapter)?}))
    }
    pub(crate) fn finish(&mut self,packet:&str)->Result<serde_json::Value,String>{
        let output=if let Some(previous)=&self.finished_packet {
            if previous!=packet{return Err("E_NATIVE_FINISH_PENDING_CHANGED: 종료한 입력 queue가 다릅니다".into());}
            self.owner.finished_output(&self.adapter)?
        }else{
            let checked=attachment::check_bytes(self.artifact.as_bytes())?;
            let owned=attachment::restore_saved_runtime_payload(&checked)?;
            let context=attachment::check_saved_revision_context(&checked,&owned,&self.parent,&self.revision)?;
            let candidate=self.owner.prepare_finish(&self.adapter,Some(packet))
                .map_err(|cause|format!("E_NATIVE_FINISH_BEFORE_PENDING: {cause}"))?;
            let plan=crate::replay_evidence::NativeFinishCheckpointPlanV1::from_saved_candidate(
                &mut self.owner,&candidate,&self.adapter,&self.admitted,&context)
                .map_err(|cause|format!("E_NATIVE_FINISH_BEFORE_PENDING: {cause}"))?;
            let output=plan.commit(&mut self.owner,&mut self.adapter,&context,candidate)?;
            self.finished_packet=Some(packet.into());output
        };
        // Keep the public recording operation's exact two-key response.
        Ok(crate::input_tape_v3::finished_output_value(&output.0,&output.1))
    }
    pub(crate) fn preview(&mut self,packet:&str)->Result<serde_json::Value,String>{
        let candidate=self.owner.prepare_finish(&self.adapter,Some(packet))
            .map_err(|cause|format!("E_NATIVE_FINISH_BEFORE_PENDING: {cause}"))?;
        Ok(crate::input_tape_v3::finished_output_value(candidate.tape_bytes(),candidate.summary()))
    }
    pub(crate) fn notice(&self)->Result<serde_json::Value,String>{self.owner.finish_notice(&self.adapter)}
    pub(crate) fn verify_receipt(&self)->Result<serde_json::Value,String>{
        self.owner.finished_output(&self.adapter)?;
        let checked=attachment::check_bytes(self.artifact.as_bytes())?;
        let owned=attachment::restore_saved_runtime_payload(&checked)?;
        let context=attachment::check_saved_revision_context(&checked,&owned,&self.parent,&self.revision)?;
        super::logical::verify_admitted_logical_receipt(&self.adapter,&self.admitted,&context,&self.binding)
    }
    pub(crate) fn stop(&mut self)->Result<serde_json::Value,String>{
        if let Some(stored)=&self.no_end{return stored.output(&self.adapter);}
        let checked=attachment::check_bytes(self.artifact.as_bytes())?;
        let owned=attachment::restore_saved_runtime_payload(&checked)?;
        let context=attachment::check_saved_revision_context(&checked,&owned,&self.parent,&self.revision)?;
        let plan=crate::replay_evidence::NativeNoEndCheckpointPlanV1::from_actual_stop(
            &mut self.owner,&self.adapter,&self.admitted,&context)?;
        let stored=plan.commit(&mut self.owner,&mut self.adapter,&context)?;
        self.no_end=Some(stored);
        self.no_end.as_ref().unwrap().output(&self.adapter)
    }
    pub(crate) fn fail_at_input(&mut self,prefix:&crate::logical_input_tape::LogicalRecordRequest,
        failed:&crate::logical_input_tape::LogicalSnapshot,expected_prefix_result:&str)->Result<serde_json::Value,String>{
        if self.finished_packet.is_some()||self.no_end.is_some(){return Err("E_NATIVE_FAILURE_TERMINAL: 끝난 생산자에 실패를 덧붙이지 않습니다".into());}
        self.owner.reproduce_failed_input(&self.adapter,prefix,failed,expected_prefix_result)?;
        let output=self.stop()?;
        if output.get("outcome").and_then(serde_json::Value::as_str)!=Some("ABORTED"){
            return Err("E_NATIVE_FAILURE_OWNER_REQUIRED: 실제 실패 기록이 아닙니다".into());
        }
        Ok(output)
    }
}
