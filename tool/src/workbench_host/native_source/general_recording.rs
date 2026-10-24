//! Protected saved content only. This never creates a continuation owner,
//! normal End, completed receipt, fresh source or replacement legacy evidence.
use super::*;
use crate::runtime_surface as runtime;

// Workbench physical response budget; common Native/CLI bundle limits remain
// unchanged. Large content is explicitly refused, never shortened or replaced.
const MAX_WORKBENCH_BUNDLE_BYTES:usize=8*1024*1024;

pub(super) fn content(authored:&str,retained:&mut BTreeMap<String,runtime::NativeFirstSourceWorldRetained>)->Result<String,String>{
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
    let project=owned.project().authored_identity.logical_dependency_id.as_str();
    let path=root(project)?;let source:SourceLocator=read(&path.join("source.locator.json"))?;
    if source.project_hex!=project{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    let digest=if let Some(owner)=retained.get_mut(project){owner.current_digest(&identity(project)?,&source.grant_locator)?}
        else{runtime::native_source_current_digest(&path,&identity(project)?,&source.grant_locator)?};
    let saved:WorldLocator=read(&path.join(format!("world-{}.json",hex::encode(digest))))?;
    let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(saved.saved_artifact_json_utf8.as_bytes())?;
    let recorded=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes)?;
    if recorded.project().save()?!=authored{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED: 현재 원문과 저장 원문이 다릅니다".into());}
    if super::logical::is_logical_artifact(&saved.saved_artifact_json_utf8)?{
        return Err("E_GENERAL_RECORDING_LOGICAL: 논리 입력의 완료 기록은 입력 기록에서 재현 확인하세요".into());
    }
    let parent=identity(&saved.parent_hex)?;let revision=identity(&saved.revision_hex)?;
    let checked=if let Some(owner)=retained.get_mut(project){owner.recording_content(&parent,&revision,&source.grant_locator)?}
        else{runtime::read_current_general_recording_content(&saved.saved_artifact_json_utf8,&parent,&revision,&path,&source.grant_locator)?};
    if checked.bundle_bytes().len()>MAX_WORKBENCH_BUNDLE_BYTES{
        return Err("E_GENERAL_RECORDING_TRANSPORT_LIMIT: 작업대의 원 기록 전송 한도(8MiB)를 넘었습니다. Native CLI에서 확인하세요".into());
    }
    let comparison:serde_json::Value=serde_json::from_str(&checked.report_json()).map_err(|_|"E_GENERAL_RECORDING_REPORT")?;
    Ok(serde_json::json!({"kind":"native_general_recording_content","authored_project_json_utf8":authored,
        "checkpoint_revision_hex":saved.revision_hex,"comparison":comparison,
        "recording_bundle_hex":hex::encode(checked.bundle_bytes())}).to_string())
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn b3_workbench_general_original_retained_and_cold_content(){
        let authored=crate::authored_project::operate("create",&serde_json::json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음."},
                {"path":"main.ddn","text":"채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }."}]}}).to_string()).unwrap();
        let host=NativeSourceController::start();
        let call=|host:&NativeSourceController,operation:&str,extra:serde_json::Value|{
            let mut request=extra;request["operation"]=operation.into();request["authored_project_json_utf8"]=authored.clone().into();
            host.call(request.to_string()).map(|text|serde_json::from_str::<serde_json::Value>(&text).unwrap())
        };
        call(&host,"save_source",serde_json::json!({})).unwrap();
        call(&host,"run",serde_json::json!({"ticks":2})).unwrap();
        let first=call(&host,"general_recording_content",serde_json::json!({})).unwrap();
        assert_eq!(first["comparison"]["accepted_count_decimal"],"2");
        let at_four=call(&host,"run_to",serde_json::json!({"next_tick":"4"})).unwrap();
        call(&host,"save_world",serde_json::json!({"next_tick":"4","state_hash":at_four["summary"]["state_hash"]})).unwrap();
        let current=call(&host,"general_recording_content",serde_json::json!({})).unwrap();
        assert_eq!(current["comparison"]["accepted_count_decimal"],"4");
        assert_eq!(current["comparison"]["status"],"RUNNING_RESULT_COMPARED");
        host.close_for_cold_test();
        let cold=call(&NativeSourceController::start(),"general_recording_content",serde_json::json!({})).unwrap();
        assert_eq!(cold,current,"cold content uses the actual unchanged Root, not a new world");
    }
}
