//! Host data is not a checkpoint authority. Execute accepted logical inputs in
//! the real Native owner and compare before changing Pending or the world head.
use super::*;
use crate::{runtime_surface as runtime,logical_input_tape::LogicalRecordRequest};

pub(super) fn recover_finish(authored:&str,input:&LogicalRecordRequest,
    retained:&BTreeMap<String,runtime::NativeFirstSourceWorldRetained>)->Result<String,String>{
    if !input.snapshots.is_empty(){return Err("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 복구 요청에는 실행 입력을 넣지 않습니다".into());}
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
    let project=owned.project().authored_identity.logical_dependency_id.as_str();
    if retained.contains_key(project){return Err("E_NATIVE_SOURCE_EXECUTION_PROFILE_CHANGED".into());}
    let path=root(project)?;let source:SourceLocator=read(&path.join("source.locator.json"))?;
    if source.project_hex!=project{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    let digest=runtime::native_source_current_digest(&path,&identity(project)?,&source.grant_locator)?;
    let saved:WorldLocator=read(&path.join(format!("world-{}.json",hex::encode(digest))))?;
    let checked=crate::unit_wire::project_runtime_attachment::check_bytes(saved.saved_artifact_json_utf8.as_bytes())?;
    let recorded=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&checked)?;
    if recorded.project().save()?!=authored{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    runtime::recover_native_logical_finish(&saved.saved_artifact_json_utf8,&identity(&saved.parent_hex)?,
        &identity(&saved.revision_hex)?,&path,&source.grant_locator,&input.binding)
}

pub(super) fn is_logical_artifact(artifact:&str)->Result<bool,String>{
    let checked=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes())?;
    crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&checked)?;
    let data:serde_json::Value=serde_json::from_str(&checked.runtime_state).map_err(|_|"E_NATIVE_SOURCE_ARTIFACT")?;
    Ok(data["input_profile"].as_str()==Some(crate::logical_input_tape::LOGICAL_INPUT_PROFILE))
}

pub(super) fn new_run(authored:&str,input:&LogicalRecordRequest,selection:&runtime::NativeAuthoringSelection,
    retained:&mut BTreeMap<String,runtime::NativeFirstSourceWorldRetained>)->Result<String,String>{
    if input.snapshots.len()!=1||input.snapshots[0].madi!=0||!input.snapshots[0].events.is_empty()
        ||input.snapshots[0].held!=input.binding.initial_held{return Err("E_NATIVE_SOURCE_NEW_LOGICAL_INITIAL_INPUT".into());}
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
    let project=owned.project().authored_identity.logical_dependency_id.as_str();let path=root(project)?;
    let source:SourceLocator=read(&path.join("source.locator.json"))?;
    if source.project_hex!=project{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    if retained.contains_key(project){return Err("E_SOURCE_NEW_RUN_NOT_READY".into());}
    let current=runtime::current_authoring_source_native(&path,&identity(project)?,&source.grant_locator)?.ok_or("E_SOURCE_NEW_RUN_NOT_READY")?;
    let payload:serde_json::Value=serde_json::from_str(&current.0.project_payload_utf8).map_err(|_|"E_NATIVE_SOURCE_SOURCE_PAYLOAD")?;
    if payload["authored_project_json_utf8"].as_str()!=Some(authored){return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    let next=revision_id()?;let parent=hex::encode(&current.0.revision_identity);
    let prepared=runtime::begin_new_authored_source_native(&path,&identity(project)?,&source.grant_locator,authored,selection)?.execute_logical(input)?.prepare(&next)?;
    prepared.stage_restore_cache(|artifact,digest|write_once(&path.join(format!("world-{}.json",hex::encode(digest))),
        &WorldLocator{parent_hex:parent.clone(),revision_hex:hex::encode(&next),saved_artifact_json_utf8:artifact.into()}))?;
    let (artifact,summary)=prepared.commit()?;
    let selected=runtime::native_authoring_selection(&path,&identity(project)?,&source.grant_locator,authored)?;
    Ok(serde_json::json!({"kind":"native_logical_world","authored_project_json_utf8":authored,"saved_artifact_json_utf8":artifact,
        "parent_hex":parent,"revision_hex":hex::encode(next),"summary":summary,"authoring_selection":selected}).to_string())
}

pub(super) fn dispatch(authored:&str,input:&LogicalRecordRequest,expected:Option<&str>,expected_result:Option<&str>,
    retained:&BTreeMap<String,runtime::NativeFirstSourceWorldRetained>)->Result<String,String>{
    let _total=crate::storage_observation::Span::start("logical.dispatch_total");
    let owned={let _phase=crate::storage_observation::Span::start("logical.authored_admission");
        crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?};
    let project=owned.project().authored_identity.logical_dependency_id.as_str();
    if retained.contains_key(project){return Err("E_NATIVE_SOURCE_EXECUTION_PROFILE_CHANGED".into());}
    if input.snapshots.len()>65536||expected.is_none()&&!input.snapshots.is_empty(){return Err("E_NATIVE_SOURCE_LOGICAL_INPUT_RANGE".into());}
    let path=root(project)?;
    let source:SourceLocator=read(&path.join("source.locator.json"))?;
    if source.project_hex!=project{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    let world_path=path.join("world.locator.json");
    let empty=LogicalRecordRequest{binding:input.binding.clone(),snapshots:vec![]};
    let (artifact,parent,next,summary)=if world_path.exists(){
        let digest=runtime::native_source_current_digest(&path,&identity(project)?,&source.grant_locator)?;
        let saved:WorldLocator=read(&path.join(format!("world-{}.json",hex::encode(digest))))?;
        let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(saved.saved_artifact_json_utf8.as_bytes())?;
        let recorded=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes)?;
        if recorded.project().save()?!=authored{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
        let parent=identity(&saved.parent_hex)?;let revision=identity(&saved.revision_hex)?;
        let (old,resave_result)=runtime::resume_logical_source_world_with_result(&saved.saved_artifact_json_utf8,&parent,&revision,&path,&source.grant_locator,&empty,Some(&recorded),expected_result.map(|_|input.snapshots.len()))?;
        let count=old["madi_count"].as_u64().and_then(|v|usize::try_from(v).ok()).ok_or("E_NATIVE_SOURCE_LOGICAL_COUNT")?;
        if let Some(expected)=expected {
            let frames=old["frames"].as_array().ok_or("E_NATIVE_SOURCE_LOGICAL_INPUTS")?;
            if frames.len()!=count||input.snapshots.len()<count{return Err("E_NATIVE_SOURCE_LOGICAL_PREFIX_CHANGED".into());}
            for (frame,snapshot) in frames.iter().zip(&input.snapshots) {
                let checked:crate::logical_input_tape::LogicalSnapshot=serde_json::from_value(frame["input"].clone()).map_err(|_|"E_NATIVE_SOURCE_LOGICAL_INPUTS")?;
                if &checked!=snapshot{return Err("E_NATIVE_SOURCE_LOGICAL_PREFIX_CHANGED".into());}
            }
            let suffix=LogicalRecordRequest{binding:input.binding.clone(),snapshots:input.snapshots[count..].to_vec()};
            // A nonempty recording suffix is executed by its actual protected
            // save owner below. Do not separately restore and execute it just
            // for the same state check. Legacy/empty-prefix paths stay intact.
            let candidate=if suffix.snapshots.is_empty()||expected_result.is_none(){
                // The exact empty suffix was already restored above for
                // this request's prefix check. With a result identity, the
                // protected recording owner below is still freshly opened;
                // final authoring-head admission also remains mandatory.
                // Reuse only the observation, never a permit or authority.
                let value=if suffix.snapshots.is_empty()&&expected_result.is_some(){old.clone()}else{
                    runtime::resume_logical_source_world(&saved.saved_artifact_json_utf8,&parent,&revision,&path,&source.grant_locator,&suffix,Some(&recorded))?
                };
                if value["state_hash"].as_str()!=Some(expected){return Err("E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH".into());}
                Some(value)
            }else{None};
            if suffix.snapshots.is_empty(){
                if let Some(expected)=expected_result{
                    let identity=resave_result.ok_or("E_NATIVE_SOURCE_SAVE_RESULT_MISMATCH: 실제 복원 결과가 없습니다")??;
                    if identity["result_identity"].as_str()!=Some(expected){return Err("E_NATIVE_SOURCE_SAVE_RESULT_MISMATCH: 원 형식 결과가 다릅니다. 기존 저장본을 보존했습니다".into());}
                }
                (saved.saved_artifact_json_utf8,saved.parent_hex,saved.revision_hex,candidate.ok_or("E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH")?)
            }else{
                let next=revision_id()?;
                let stage=|artifact:&str,digest:&[u8;32]|write_once(&path.join(format!("world-{}.json",hex::encode(digest))),
                    &WorldLocator{parent_hex:saved.revision_hex.clone(),revision_hex:hex::encode(&next),saved_artifact_json_utf8:artifact.into()});
                let (artifact,summary)={let _phase=crate::storage_observation::Span::start("logical.continued_save_call");
                    runtime::save_continued_logical_source_world(&saved.saved_artifact_json_utf8,&parent,&revision,&next,&path,&source.grant_locator,&suffix,expected_result.map(|_|expected),expected_result,stage)?};
                (artifact,saved.revision_hex,hex::encode(next),summary)
            }
        }else{(saved.saved_artifact_json_utf8,saved.parent_hex,saved.revision_hex,old)}
    }else{
        let expected=expected.ok_or("E_NATIVE_SOURCE_WORLD_REQUIRED: 논리 입력 저장본이 없습니다")?;
        if input.snapshots.is_empty(){return Err("E_NATIVE_SOURCE_LOGICAL_INPUT_RANGE".into());}
        let actual={let _phase=crate::storage_observation::Span::start("logical.first_save_read_source");
            runtime::read_authored_source_native(&path,&identity(project)?,&source.grant_locator)?};
        let payload:serde_json::Value=serde_json::from_str(&actual.project_payload_utf8).map_err(|_|"E_NATIVE_SOURCE_SOURCE_PAYLOAD")?;
        if payload["authored_project_json_utf8"].as_str()!=Some(authored){return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
        let session={let _phase=crate::storage_observation::Span::start("logical.first_save_begin");
            runtime::begin_first_authored_source_native(&path,&identity(project)?,&source.grant_locator,&actual.revision_identity,&actual.revision_integrity)?};
        let execution={let _phase=crate::storage_observation::Span::start("logical.first_save_execute");
            session.execute_logical(input)?};
        {let _phase=crate::storage_observation::Span::start("logical.first_save_compare");
            execution.check_state_hash(expected)?;
            if let Some(expected)=expected_result{execution.check_result_identity(expected)?;}}
        let next=revision_id()?;
        let prepared={let _phase=crate::storage_observation::Span::start("logical.first_save_prepare");
            execution.prepare(&next)?};
        {let _phase=crate::storage_observation::Span::start("logical.first_save_stage_cache");
            prepared.stage_restore_cache(|artifact,digest|{
                let cache=WorldLocator{parent_hex:hex::encode(&actual.revision_identity),revision_hex:hex::encode(&next),saved_artifact_json_utf8:artifact.into()};
                write_once(&path.join(format!("world-{}.json",hex::encode(digest))),&cache)?;write_once(&world_path,&cache)
            })?;}
        let (artifact,summary)={let _phase=crate::storage_observation::Span::start("logical.first_save_commit");
            prepared.commit()?};
        (artifact,hex::encode(actual.revision_identity),hex::encode(next),summary)
    };
    // The response is emitted only after Native readmission/commit. The UI
    // receives an input projection, not a Native permit or durable capsule.
    let selected={let _phase=crate::storage_observation::Span::start("logical.final_authoring_selection");
        runtime::native_authoring_selection(&path,&identity(project)?,&source.grant_locator,authored)?};
    Ok(serde_json::json!({"kind":"native_logical_world","authored_project_json_utf8":authored,"authoring_selection":selected,
        "saved_artifact_json_utf8":artifact,"parent_hex":parent,"revision_hex":next,"summary":summary}).to_string())
}
fn revision_id()->Result<Vec<u8>,String>{super::revision()}

pub(super) fn begin_finish(authored:&str,input:&LogicalRecordRequest,
    retained:&BTreeMap<String,runtime::NativeFirstSourceWorldRetained>)->Result<runtime::NativeLogicalFinishSessionV1,String>{
    open_recording(authored,input,retained,RecordingOpen::Current)
}
pub(super) fn new_record(authored:&str,input:&LogicalRecordRequest,
    retained:&BTreeMap<String,runtime::NativeFirstSourceWorldRetained>,selection:&runtime::NativeAuthoringSelection)->Result<String,String>{
    // The explicit operation has its own actual normal-restore admission.
    // Dropping this transient owner does not delete the old terminal rows.
    drop(open_recording(authored,input,retained,RecordingOpen::New(selection))?);
    dispatch(authored,input,None,None,retained)
}
enum RecordingOpen<'a>{Current,New(&'a runtime::NativeAuthoringSelection)}
fn open_recording(authored:&str,input:&LogicalRecordRequest,
    retained:&BTreeMap<String,runtime::NativeFirstSourceWorldRetained>,admission:RecordingOpen<'_>)->Result<runtime::NativeLogicalFinishSessionV1,String>{
    if !input.snapshots.is_empty(){return Err("E_NATIVE_FINISH_INPUT: 저장 경계를 먼저 확정하세요".into());}
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
    let project=owned.project().authored_identity.logical_dependency_id.as_str();
    if retained.contains_key(project){return Err("E_NATIVE_FINISH_WORLD_REQUIRED".into());}
    let path=root(project)?;let source:SourceLocator=read(&path.join("source.locator.json"))?;
    if source.project_hex!=project{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    let digest=runtime::native_source_current_digest(&path,&identity(project)?,&source.grant_locator)?;
    let saved:WorldLocator=read(&path.join(format!("world-{}.json",hex::encode(digest))))?;
    let checked=crate::unit_wire::project_runtime_attachment::check_bytes(saved.saved_artifact_json_utf8.as_bytes())?;
    let recorded=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&checked)?;
    if recorded.project().save()?!=authored{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    let parent=identity(&saved.parent_hex)?;let revision=identity(&saved.revision_hex)?;
    match admission{
        RecordingOpen::Current=>runtime::NativeLogicalFinishSessionV1::open(&saved.saved_artifact_json_utf8,&parent,
            &revision,&path,&source.grant_locator,&input.binding,Some(&recorded)),
        RecordingOpen::New(selection)=>runtime::NativeLogicalFinishSessionV1::open_new_record(&saved.saved_artifact_json_utf8,&parent,
            &revision,&path,&source.grant_locator,&input.binding,selection,Some(&recorded)),
    }
}

/// Read-only consumer. The locator selects bytes; only the Native receipt and
/// protected Finished row admit them. No normal restore or new Run is issued.
pub(super) fn verify_receipt(authored:&str,input:&LogicalRecordRequest,
    retained:&BTreeMap<String,runtime::NativeFirstSourceWorldRetained>)->Result<String,String>{
    if !input.snapshots.is_empty(){return Err("E_NATIVE_RECEIPT_INPUT: 재현 확인에 새 입력을 넣을 수 없습니다".into());}
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
    let project=owned.project().authored_identity.logical_dependency_id.as_str();
    if retained.contains_key(project){return Err("E_NATIVE_RECEIPT_NOT_FINISHED: 진행 중 실행은 완료 기록이 아닙니다".into());}
    let path=root(project)?;
    let source:SourceLocator=read(&path.join("source.locator.json"))?;
    if source.project_hex!=project{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    let digest=runtime::native_source_current_digest(&path,&identity(project)?,&source.grant_locator)?;
    let saved:WorldLocator=read(&path.join(format!("world-{}.json",hex::encode(digest))))?;
    let checked=crate::unit_wire::project_runtime_attachment::check_bytes(saved.saved_artifact_json_utf8.as_bytes())?;
    let recorded=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&checked)?;
    if recorded.project().save()?!=authored{
        return Err("E_UNSUPPORTED_HISTORICAL_REPLAY: 다른 원문의 기록으로 대체하지 않습니다".into());
    }
    runtime::verify_native_logical_current_receipt(&saved.saved_artifact_json_utf8,
        &identity(&saved.parent_hex)?,&identity(&saved.revision_hex)?,&path,&source.grant_locator,&input.binding)
        .map(|value|value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the exact existing task-owned Ready fixture and explicit observation flags"]
    fn wo25_existing_ready_query_compilation_observation(){
        let request_path=std::env::var("DDN_WO25_READY_FIXTURE").expect("explicit Ready fixture path required");
        let local=std::env::var("DDN_WO25_READY_LOCALAPPDATA").expect("explicit fixture owner location required");
        assert_eq!(std::env::var("DDN_WO25_TEST_COMPILE_OBSERVATION").as_deref(),Ok("1"));
        assert_eq!(std::env::var("DDN_PRIVATE_STORAGE_TIMING").as_deref(),Ok("1"));
        assert_eq!(std::env::var("LOCALAPPDATA").as_deref(),Ok(local.as_str()));
        assert!(std::fs::metadata(&request_path).unwrap().len()<=1_048_576);
        let request=std::fs::read_to_string(&request_path).unwrap();
        assert_eq!(runtime::sha256_hex(request.as_bytes()),
            "2888a09f0fa9bc5ed27e5e7ad49669abddd70adb694eddef5408b52130d619f4");
        let parsed:serde_json::Value=serde_json::from_str(&request).unwrap();
        assert_eq!(parsed["operation"],"logical_finish_recover");
        assert_eq!(parsed["input_request"]["snapshots"],serde_json::json!([]));
        let host=NativeSourceController::start();
        let response=host.call(request).unwrap();
        host.close_for_cold_test();
        let response:serde_json::Value=serde_json::from_str(&response).unwrap();
        assert_eq!(runtime::sha256_hex(response["original_output_json_utf8"].as_str().unwrap().as_bytes()),
            "66738e5024741a5b38238e365d897f6a60e5e27f7b154c44084e41f827547c4d");
        assert_eq!(response["notice"],serde_json::json!({"excluded_input_count":"1"}));
        assert!(response["original_failure"].is_null());
    }
    #[test]
    fn wo22_native_logical_controller_actual_cold_save_and_prefix_rejection(){
        controller_flow(include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/main.ddn"));
    }
    #[test]
    fn b3_native_controller_finish_current_owner_and_before_pending(){
        controller_flow("채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.");
    }
    #[test]
    fn b3_native_mixed_progress_corruption_and_wrong_world_observation(){
        controller_flow_inner("채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.",true,false);
    }
    #[test]
    fn wo25_native_finish_preserved_ready_failure_cold_repair_returns_original_without_rerun(){
        controller_flow_inner("채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.",false,true);
    }
    fn controller_flow(main_source:&str){
        controller_flow_inner(main_source,false,false);
    }
    fn controller_flow_inner(main_source:&str,mixed_failure:bool,ready_failure:bool){
        let authored=crate::authored_project::operate("create",&serde_json::json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":main_source}]}}).to_string()).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes()).unwrap();
        let mut input:LogicalRecordRequest=serde_json::from_str(include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/initial-input.json")).unwrap();
        input.binding.source_revision=runtime::sha256_identity(owned.sources().preparation().source.as_bytes());
        input.snapshots.truncate(2);
        let mut baseline=runtime::fresh_authored_logical_run(&authored,input.binding.clone()).unwrap();
        baseline.step(&input.snapshots[0]).unwrap();let first=baseline.session().summary();
        let first_result=crate::replay_evidence::ResultPreimageV1::capture(baseline.session().borrow_result_boundary().unwrap()).unwrap().digest_label();
        baseline.step(&input.snapshots[1]).unwrap();let second=baseline.session().summary();
        let second_result=crate::replay_evidence::ResultPreimageV1::capture(baseline.session().borrow_result_boundary().unwrap()).unwrap().digest_label();
        let host=NativeSourceController::start();
        host.call(serde_json::json!({"operation":"save_source","authored_project_json_utf8":authored}).to_string()).unwrap();
        let mut prefix=input.clone();prefix.snapshots.truncate(1);
        let call=|host:&NativeSourceController,operation:&str,input:&LogicalRecordRequest,hash:Option<&str>|{
            let mut request=serde_json::json!({"operation":operation,"authored_project_json_utf8":authored,"input_request":input});
            if let Some(hash)=hash{request["state_hash"]=hash.into();request["expected_result_identity"]=if input.snapshots.len()==1{first_result.clone()}else{second_result.clone()}.into();}
            host.call(request.to_string())
        };
        let saved:serde_json::Value=serde_json::from_str(&call(&host,"logical_save",&prefix,first["state_hash"].as_str()).unwrap()).unwrap();
        assert_eq!(saved["summary"],first);
        let empty=LogicalRecordRequest{binding:input.binding.clone(),snapshots:vec![]};
        // A new controller reconstructs from Native custody, not a live map.
        let cold=NativeSourceController::start();
        let loaded:serde_json::Value=serde_json::from_str(&call(&cold,"logical_load",&empty,None).unwrap()).unwrap();
        assert_eq!(loaded["summary"],first);
        if mixed_failure {
            let project=owned.project().authored_identity.logical_dependency_id.as_str();
            let path=root(project).unwrap();
            let db=rusqlite::Connection::open(path.join("vc04_snapshot_v2.sqlite3")).unwrap();
            let cipher:Vec<u8>=db.query_row("SELECT custody_ciphertext FROM wo23_replay_controls_v1 WHERE control_ordinal=0",[],|row|row.get(0)).unwrap();
            let mut damaged=cipher.clone();damaged[0]^=1;
            assert_eq!(db.execute("UPDATE wo23_replay_controls_v1 SET custody_ciphertext=?1 WHERE control_ordinal=0",[&damaged]).unwrap(),1);
            let rows_before:String=db.query_row("SELECT group_concat(hex(row_sha256),',') FROM wo23_replay_controls_v1",[],|row|row.get(0)).unwrap();
            let cache_before=std::fs::read_dir(&path).unwrap().map(|entry|entry.unwrap().path()).filter(|p|p.file_name().unwrap().to_string_lossy().starts_with("world-")).map(|p|(p.clone(),std::fs::read(p).unwrap())).collect::<Vec<_>>();
            let restore=call(&cold,"logical_load",&empty,None);
            let error=call(&cold,"logical_save",&input,Some("blake3:not-the-world")).unwrap_err();
            if let Err(ref restore_error)=restore {
                assert_eq!(&error,restore_error,"normal readmission rejects before either old or new suffix preflight");
            } else {
                assert_eq!(error,"E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH",
                    "the former normal/world check still wins on the rejected mixed path");
                let protected_error=call(&cold,"logical_save",&input,second["state_hash"].as_str()).unwrap_err();
                assert!(protected_error.starts_with("E_MISSING_REPLAY_PROGRESS:"),
                    "a correct world hash must not convert failed protected admission to success");
            }
            let rows_after:String=db.query_row("SELECT group_concat(hex(row_sha256),',') FROM wo23_replay_controls_v1",[],|row|row.get(0)).unwrap();
            assert_eq!(rows_after,rows_before,"no new replay row/Pending publication");
            assert_eq!(db.query_row("SELECT custody_ciphertext FROM wo23_replay_controls_v1 WHERE control_ordinal=0",[],|row|row.get::<_,Vec<u8>>(0)).unwrap(),damaged);
            for (file,bytes) in cache_before {assert_eq!(std::fs::read(file).unwrap(),bytes);}
            assert_eq!(db.execute("UPDATE wo23_replay_controls_v1 SET custody_ciphertext=?1 WHERE control_ordinal=0",[&cipher]).unwrap(),1);
            let restored:serde_json::Value=serde_json::from_str(&call(&cold,"logical_load",&empty,None).unwrap()).unwrap();
            assert_eq!(restored,loaded,"repairing only the test mutation recovers the exact original head");
            let normal_restore_rejected_before_suffix=restore.is_err();
            println!("MIXED_FAILURE_OBSERVATION={}",serde_json::json!({"normal_restore_error":restore.err(),"mixed_save_error":error,"normal_restore_rejected_before_suffix":normal_restore_rejected_before_suffix,"replay_rows_and_cache_unchanged":true,"original_head_restored":true}));
            return;
        }
        let source=host.call(serde_json::json!({"operation":"load","authored_project_json_utf8":authored}).to_string()).unwrap();
        assert_eq!(serde_json::from_str::<serde_json::Value>(&source).unwrap()["kind"],"native_logical_saved_source");
        let mut bad=input.clone();bad.snapshots[0].held.insert("왼쪽".into(),true);
        assert!(call(&cold,"logical_save",&bad,second["state_hash"].as_str()).unwrap_err().contains("PREFIX_CHANGED"));
        assert!(call(&cold,"logical_save",&input,Some("blake3:not-the-world")).unwrap_err().contains("MISMATCH"));
        let both=cold.call(serde_json::json!({"operation":"logical_save","authored_project_json_utf8":authored,
            "input_request":input,"state_hash":"blake3:not-the-world","expected_result_identity":format!("sha256:{}","0".repeat(64))}).to_string()).unwrap_err();
        assert_eq!(both,"E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH","world check keeps priority over result check");
        let refused=cold.call(serde_json::json!({"operation":"logical_save","authored_project_json_utf8":authored,
            "input_request":input,"state_hash":second["state_hash"],"expected_result_identity":format!("sha256:{}","0".repeat(64))}).to_string()).unwrap_err();
        assert!(refused.contains("SAVE_RESULT_MISMATCH"));
        let unchanged:serde_json::Value=serde_json::from_str(&call(&cold,"logical_load",&empty,None).unwrap()).unwrap();
        assert_eq!(unchanged["revision_hex"],saved["revision_hex"]);assert_eq!(unchanged["summary"],first);
        let saved:serde_json::Value=serde_json::from_str(&call(&cold,"logical_save",&input,second["state_hash"].as_str()).unwrap()).unwrap();
        assert_eq!(saved["summary"],second);
        let loaded:serde_json::Value=serde_json::from_str(&call(&NativeSourceController::start(),"logical_load",&empty,None).unwrap()).unwrap();
        assert_eq!(loaded["summary"],second);assert_eq!(loaded["revision_hex"],saved["revision_hex"]);
        // Author a new revision after a logical world; the fresh segment uses
        // a new source binding and starts with an actual initial empty input.
        let parsed:crate::authored_project::ProjectArtifact=serde_json::from_str(&authored).unwrap();
        let mut snapshot=parsed.source_snapshot.clone();snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text.push_str("\n// 새 실행 원문\n");
        let edited=crate::authored_project::operate("update",&serde_json::json!({"original_json":authored,"source_snapshot":snapshot}).to_string()).unwrap();
        let ack:serde_json::Value=serde_json::from_str(&cold.call(serde_json::json!({"operation":"save_source","authored_project_json_utf8":edited,
            "previous_authored_project_json_utf8":authored,"authoring_selection":loaded["authoring_selection"]}).to_string()).unwrap()).unwrap();
        let changed=crate::project_revision_envelope_supply::admit_fresh_authored_project(edited.as_bytes()).unwrap();
        let mut next=input.clone();next.binding.source_revision=runtime::sha256_identity(changed.sources().preparation().source.as_bytes());next.snapshots.truncate(1);
        let fresh:serde_json::Value=serde_json::from_str(&cold.call(serde_json::json!({"operation":"logical_new_run","authored_project_json_utf8":edited,
            "input_request":next,"authoring_selection":ack["authoring_selection"]}).to_string()).unwrap()).unwrap();
        assert_eq!(fresh["summary"]["madi_count"],1);
        let empty=LogicalRecordRequest{binding:next.binding.clone(),snapshots:vec![]};
        let loaded:serde_json::Value=serde_json::from_str(&cold.call(serde_json::json!({"operation":"logical_load","authored_project_json_utf8":edited,"input_request":empty}).to_string()).unwrap()).unwrap();
        assert_eq!(loaded["summary"],fresh["summary"]);
        assert!(cold.call(serde_json::json!({"operation":"logical_verify_receipt",
            "authored_project_json_utf8":edited,"input_request":empty}).to_string()).is_err(),
            "Running is not a completed receipt");
        let begin:serde_json::Value=serde_json::from_str(&cold.call(serde_json::json!({
            "operation":"logical_finish_begin","authored_project_json_utf8":edited,"input_request":empty
        }).to_string()).unwrap()).unwrap();
        let registration=&begin["registration"];
        assert_eq!(registration["accepted_event_count"],"0");
        let packet=serde_json::json!({"producer_generation":registration["producer_generation"],
            "first_unprocessed":"0","observed_until":"0","events":[]}).to_string();
        let mut foreign:serde_json::Value=serde_json::from_str(&packet).unwrap();
        foreign["producer_generation"]="foreign-producer".into();
        let refused=cold.call(serde_json::json!({"operation":"logical_finish_commit",
            "session_handle":begin["session_handle"],"pending_json_utf8":foreign.to_string()}).to_string()).unwrap_err();
        assert!(refused.starts_with("E_NATIVE_FINISH_BEFORE_PENDING:"));
        let finish=serde_json::json!({"operation":"logical_finish_commit","session_handle":begin["session_handle"],
            "pending_json_utf8":packet}).to_string();
        let original_failure;
        let complete:serde_json::Value=if ready_failure{
            let preview:serde_json::Value=serde_json::from_str(&cold.call(serde_json::json!({
                "operation":"logical_finish_prepare","session_handle":begin["session_handle"],"pending_json_utf8":packet
            }).to_string()).unwrap()).unwrap();
            cold.call(serde_json::json!({"operation":"_test_finish_ready_set_failure_once"}).to_string()).unwrap();
            let failure=cold.call(finish.clone()).unwrap_err();
            assert_eq!(failure,"E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 종료 Ready 보호 확인 (WriteFailed)","original cause must survive evidence retention unchanged");
            original_failure=Some(failure);preview
        }else{
            original_failure=None;serde_json::from_str(&cold.call(finish.clone()).unwrap()).unwrap()
        };
        assert_eq!(complete.as_object().unwrap().len(),2);
        assert_eq!(complete["summary"],fresh["summary"]);
        let notice:serde_json::Value=if ready_failure{
            assert!(cold.call(finish).is_err(),"failed Pending owner cannot publish a second finish");
            serde_json::json!({"excluded_input_count":"0"})
        }else{
            assert_eq!(serde_json::from_str::<serde_json::Value>(&cold.call(finish).unwrap()).unwrap(),complete);
            serde_json::from_str(&cold.call(serde_json::json!({
            "operation":"logical_finish_notice","session_handle":begin["session_handle"]}).to_string()).unwrap()).unwrap()
        };
        assert_eq!(notice["excluded_input_count"],"0");
        if !ready_failure{
            let same_owner_verified:serde_json::Value=serde_json::from_str(&cold.call(serde_json::json!({
            "operation":"logical_finish_verify","session_handle":begin["session_handle"]}).to_string()).unwrap()).unwrap();
            assert_eq!(same_owner_verified["status"],"REPLAY_VERIFIED");
            assert_eq!(same_owner_verified["summary"],complete["summary"]);
        }
        cold.call(serde_json::json!({"operation":"logical_finish_close","session_handle":begin["session_handle"]}).to_string()).unwrap();
        let storage_path=root(owned.project().authored_identity.logical_dependency_id.as_str()).unwrap();
        let stored_state=||{
            let db=rusqlite::Connection::open_with_flags(storage_path.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
            let head:String=db.query_row("SELECT group_concat(v,',') FROM (SELECT hex(history_revision_identity)||':'||hex(revision_integrity_sha256)||':'||hex(payload_key) AS v FROM vc04_project_heads ORDER BY project_identity)",[],|row|row.get(0)).unwrap();
            let controls:String=db.query_row("SELECT group_concat(v,',') FROM (SELECT hex(row_sha256) AS v FROM wo23_replay_controls_v1 ORDER BY execution_first_tx,control_ordinal)",[],|row|row.get(0)).unwrap();
            let records:String=db.query_row("SELECT group_concat(v,',') FROM (SELECT hex(record_sha256) AS v FROM wo25_finish_recovery_records_v1 ORDER BY execution_first_tx,control_ordinal,evidence_ordinal)",[],|row|row.get(0)).unwrap();
            (head,controls,records)
        };
        let before_recovery=stored_state();
        if ready_failure{
            assert_cold_recovery_tampering_rejected(&storage_path,&edited,&empty);
            assert_eq!(stored_state(),before_recovery,"restoring exact mutation bytes must preserve stored evidence");
        }
        let recovered_text=NativeSourceController::start().call(serde_json::json!({
            "operation":"logical_finish_recover","authored_project_json_utf8":edited,"input_request":empty
        }).to_string()).unwrap();
        let recovered:serde_json::Value=serde_json::from_str(&recovered_text).unwrap();
        assert_eq!(serde_json::from_str::<serde_json::Value>(recovered["original_output_json_utf8"].as_str().unwrap()).unwrap(),complete);
        let original_bytes=complete.to_string();
        assert_eq!(recovered["original_output_json_utf8"].as_str(),Some(original_bytes.as_str()),"original C01 bytes must be returned, not a rewritten summary");
        assert_eq!(stored_state(),before_recovery,"repair must not advance world head, replay ordinal or original evidence");
        assert_eq!(recovered["notice"],notice);
        assert_eq!(recovered["original_failure"],serde_json::to_value(&original_failure).unwrap());
        let mut wrong=empty.clone();wrong.binding.initial_rng^=1;
        assert!(NativeSourceController::start().call(serde_json::json!({
            "operation":"logical_finish_recover","authored_project_json_utf8":edited,"input_request":wrong
        }).to_string()).is_err());
        assert_eq!(NativeSourceController::start().call(serde_json::json!({
            "operation":"logical_finish_recover","authored_project_json_utf8":edited,"input_request":empty
        }).to_string()).unwrap(),recovered_text,"Ready query must return identical bytes without a new recording owner");
        assert_eq!(stored_state(),before_recovery,"repeated query/refusal must not write a second result");
        let verified:serde_json::Value=serde_json::from_str(&NativeSourceController::start().call(serde_json::json!({
            "operation":"logical_verify_receipt","authored_project_json_utf8":edited,"input_request":empty
        }).to_string()).unwrap()).unwrap();
        assert_eq!(verified["status"],"REPLAY_VERIFIED");
        assert_eq!(verified["summary"],complete["summary"]);
        let mut stale=loaded["authoring_selection"].clone();stale["key"]=serde_json::json!(vec![0u8;32]);
        assert!(cold.call(serde_json::json!({"operation":"logical_new_record","authored_project_json_utf8":edited,
            "input_request":empty,"authoring_selection":stale}).to_string()).unwrap_err().contains("SELECTION_CHANGED"));
        let restarted:serde_json::Value=serde_json::from_str(&cold.call(serde_json::json!({"operation":"logical_new_record",
            "authored_project_json_utf8":edited,"input_request":empty,"authoring_selection":loaded["authoring_selection"]}).to_string()).unwrap()).unwrap();
        assert_eq!(restarted["summary"],fresh["summary"]);
        assert_eq!(restarted["revision_hex"],fresh["revision_hex"]);
    }

    fn assert_cold_recovery_tampering_rejected(root:&std::path::Path,authored:&str,input:&LogicalRecordRequest){
        use crate::replay_evidence::finish_recovery::{RecordHeaderV1,RecordKindV1,encode_header};
        use sha2::{Digest,Sha256};
        let mut db=rusqlite::Connection::open(root.join("vc04_snapshot_v2.sqlite3")).unwrap();
        // This fixture has exactly one actual retained failure; never pick MAX/latest.
        let mut rows=db.prepare("SELECT execution_first_tx,control_ordinal,header_bytes,record_sha256,payload_length FROM wo25_finish_recovery_records_v1 WHERE evidence_ordinal=1").unwrap();
        let entries=rows.query_map([],|row|Ok((row.get::<_,Vec<u8>>(0)?,row.get::<_,i64>(1)?,row.get::<_,Vec<u8>>(2)?,row.get::<_,Vec<u8>>(3)?,row.get::<_,i64>(4)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        assert_eq!(entries.len(),1);drop(rows);
        let (first,ordinal,header,digest,length)=&entries[0];
        let intent:Vec<u8>=db.query_row("SELECT header_bytes FROM wo25_finish_recovery_records_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=0",rusqlite::params![first,ordinal],|row|row.get(0)).unwrap();
        let parsed=RecordHeaderV1::parse(&intent).unwrap();
        let nonce:&[u8;32]=parsed.field(2).unwrap().try_into().unwrap();
        let first_array:&[u8;32]=first.as_slice().try_into().unwrap();
        let protected=||crate::snapshot_v2_durable_transaction::native::test_finish_lifecycle_cipher_digest(nonce,first_array).unwrap();
        let before=protected();
        let recover=||NativeSourceController::start().call(serde_json::json!({"operation":"logical_finish_recover","authored_project_json_utf8":authored,"input_request":input}).to_string());
        let payload:Vec<u8>=db.query_row("SELECT chunk_bytes FROM wo25_finish_recovery_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=1 AND chunk_ordinal=0",rusqlite::params![first,ordinal],|row|row.get(0)).unwrap();
        let candidate=RecordHeaderV1::parse(header).unwrap();
        let mut fields=(1..=7).map(|tag|candidate.field(tag).unwrap().to_vec()).collect::<Vec<_>>();
        let forged=b"forged original failure";
        fields[4]=Sha256::digest(forged).to_vec();fields[5]=(forged.len() as u64).to_be_bytes().to_vec();
        let refs=fields.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let forged_header=encode_header(RecordKindV1::ActualFailure,&refs).unwrap();
        // All public hashes are internally valid; the protected stage2 witness is not.
        let forged_digest=RecordHeaderV1::parse(&forged_header).unwrap().check_payload([forged.as_slice()]).unwrap();
        let tx=db.transaction().unwrap();
        tx.execute("UPDATE wo25_finish_recovery_records_v1 SET header_bytes=?3,record_sha256=?4,payload_length=?5 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=1",rusqlite::params![first,ordinal,&forged_header,forged_digest.as_slice(),forged.len() as i64]).unwrap();
        tx.execute("UPDATE wo25_finish_recovery_chunks_v1 SET chunk_bytes=?3 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=1 AND chunk_ordinal=0",rusqlite::params![first,ordinal,forged.as_slice()]).unwrap();tx.commit().unwrap();
        assert!(recover().is_err(),"recomputed outer hashes cannot replace the authenticated failure");
        assert_eq!(protected(),before,"refusal must not publish protected Ready");
        let tx=db.transaction().unwrap();
        tx.execute("UPDATE wo25_finish_recovery_records_v1 SET header_bytes=?3,record_sha256=?4,payload_length=?5 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=1",rusqlite::params![first,ordinal,header,digest,length]).unwrap();
        tx.execute("UPDATE wo25_finish_recovery_chunks_v1 SET chunk_bytes=?3 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=1 AND chunk_ordinal=0",rusqlite::params![first,ordinal,&payload]).unwrap();tx.commit().unwrap();
        let cipher:Vec<u8>=db.query_row("SELECT custody_ciphertext FROM wo23_replay_controls_v1 WHERE execution_first_tx=?1 AND control_ordinal=0",[first],|row|row.get(0)).unwrap();
        let mut corrupt=cipher.clone();corrupt[0]^=1;
        db.execute("UPDATE wo23_replay_controls_v1 SET custody_ciphertext=?2 WHERE execution_first_tx=?1 AND control_ordinal=0",rusqlite::params![first,&corrupt]).unwrap();
        assert!(recover().is_err(),"valid current witness cannot bypass historical custody");
        assert_eq!(protected(),before,"archive refusal must not publish protected Ready");
        db.execute("UPDATE wo23_replay_controls_v1 SET custody_ciphertext=?2 WHERE execution_first_tx=?1 AND control_ordinal=0",rusqlite::params![first,&cipher]).unwrap();
    }
}
