//! Rebuild presentation-only typed selections from the already checked Native
//! source/initial-subject/input prefix. This emits no Run, receipt or storage.
use super::*;

pub(crate) fn projection_may_need_table_history(projection: Option<&JsonValue>) -> bool {
    // A saved plane placeholder can also be an ordinary empty scene. Use it
    // only to request the checked replay, never to authorize an XY selection.
    let placeholder=crate::scene_value::xy_projection_placeholder();
    let may_need_history=|view:&JsonValue|view["table_projection"].is_object() || view==&placeholder;
    projection.and_then(|p| p["candidates"].as_array()).into_iter().flatten().any(|candidate| {
        let artifact=&candidate["artifact"];
        may_need_history(&artifact["view"]) || artifact["relations"].as_array().into_iter().flatten()
            .any(|relation|may_need_history(&relation["view"]))
    })
}

pub(crate) fn rebuild_native_table_history(
    owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    rows:&[crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow],
    initial_tick:u64,base_seed:u64,age:AgeTarget,
    bootstrap:Option<&crate::unit_wire::subject_execution::SubjectExecutionStage>,
    expected_world:&ddonirang_core::platform::NuriWorld,
    expected_history:&crate::bogae_time::SceneHistory,
) -> Result<Option<crate::bogae_time::TableSelectionTimeline>,String> {
    // NativeSourceBinding already admits only the preserved basic-input route.
    // Never drop explicitly recorded control edges or guess an initial world.
    if initial_tick!=0 || rows.iter().any(|row|row.edges.is_some()) {
        return Err("E_NATIVE_SOURCE_TABLE_HISTORY_INPUT: 이 원문의 기록된 입력 경계를 재구성할 수 없습니다".into());
    }
    let ticks=u32::try_from(rows.len()).map_err(|_|"E_NATIVE_SOURCE_MADI_LIMIT")?;
    let inputs=rows.iter().map(|row|row.input.clone()).collect::<Vec<_>>();
    let artifact=owned.project().save()?;
    let replay=run_general_input_prefix_from_owned_with_subject(owned.clone(),&artifact,ticks,base_seed,
        Some(&inputs),None,age,bootstrap,None)?;
    if replay.execution.next_tick!=u64::from(ticks)
        || current_state_hash(replay.execution.world.world())!=current_state_hash(expected_world)
        || !replay.execution.runner.bogae_history_for_resume().same_execution_history(expected_history) {
        return Err("E_NATIVE_SOURCE_TABLE_HISTORY_MISMATCH: 실제 원문·입력으로 재구성한 세계나 보기 이력이 다릅니다".into());
    }
    Ok(replay.execution.runner.table_selection_history_for_rollback())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r86_empty_plane_requests_replay_but_only_checked_xy_restores_selection() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let prefix="채비 { 값:수 := 1. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. }. }. (거리:값 * 1@m,시각:값 * 1@s) 내보임.";
        for (body,xy) in [
            ("보기 := (가로=관찰.시각,세로=관찰.거리) 표준보기.좌표선.",true),
            ("보기 := (장면=(요소들=차림[(중심=차림[0,0],반지름=1) 표준장면.원][0..0],가로범위=차림[0,1],세로범위=차림[0,1]) 표준장면.평면) 표준보기.장면.",false),
        ] {
            let source=SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
                SourceFile{path:"root.ddn".into(),text:r#"(글판="v25",모듬=차림[(이름=앱,머리="main.ddn",원문="main.ddn")],시작=(모듈=앱,정의=실행)) 얽음."#.into()},
                SourceFile{path:"main.ddn".into(),text:format!("{prefix} {body}")}]};
            let artifact=ProjectArtifact::create("v1-core-v25".into(),source,String::new()).unwrap().save().unwrap();
            let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(artifact.as_bytes()).unwrap();
            let age=crate::ddn_runtime::default_age_target();
            let run=run_general_input_prefix_from_owned(owned.clone(),&artifact,2,0,None,None,age).unwrap();
            let history=run.execution.runner.bogae_history_for_resume();
            let projection=history.restored_projection(Some(1)).unwrap().unwrap();
            assert!(projection_may_need_table_history(Some(&projection)));
            assert!(projection["candidates"][0]["artifact"]["view"]["table_projection"].is_null());
            let timeline=rebuild_native_table_history(&owned,&run.rows,0,0,age,None,
                run.execution.world.world(),history).unwrap();
            assert_eq!(timeline.is_some(),xy);
        }
    }

    #[test]
    fn r85_native_projection_uses_committed_xy_and_fixed_table_before_step() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let source=SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:r#"(글판="v25",모듬=차림[(이름=앱,머리="main.ddn",원문="main.ddn")],시작=(모듈=앱,정의=실행)) 얽음."#.into()},
            SourceFile{path:"main.ddn".into(),text:"채비 { 값:수 := 1. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. }. }. (거리:값 * 1@m,시각:값 * 1@s,완료:거짓) 내보임. 보기 := (관계들=차림[(보기=(가로=관찰.시각,세로=관찰.거리) 표준보기.좌표선) 표준보개.으뜸,(보기=(열들=차림[관찰.거리]) 표준보기.결과표,자리=#오른쪽,시간=((번호=0) 표준시간.고정마디)) 표준보개.곁]) 표준보개.짜임.".into()}]};
        let artifact=ProjectArtifact::create("v1-core-v25".into(),source,String::new()).unwrap().save().unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(artifact.as_bytes()).unwrap();
        let age=crate::ddn_runtime::default_age_target();
        let run=run_general_input_prefix_from_owned(owned.clone(),&artifact,2,0,None,None,age).unwrap();
        let history=run.execution.runner.bogae_history_for_resume();
        let timeline=rebuild_native_table_history(&owned,&run.rows,0,0,age,None,
            run.execution.world.world(),history).unwrap().unwrap();
        let mut projection=history.restored_projection(Some(1)).unwrap().unwrap();
        let trace=&run.summary["public_observation_trace"];
        let frames=run.summary["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"].as_array().unwrap();
        let source=trace["source_identity"].as_str().unwrap();
        let revision=trace["revision_identity"].as_str().unwrap();
        crate::bogae_time::resolve_result_tables(&mut projection,frames,source,revision).unwrap();
        assert!(timeline.resolve_committed(2,&projection).is_err());
        let selections=timeline.resolve_committed(1,&projection).unwrap();
        let rebuilt=crate::bogae_time::resolve_typed_result_tables(&projection,frames,source,revision,&selections).unwrap();
        assert_eq!(rebuilt,run.summary["bogae_projection"]);
        let relations=&rebuilt["candidates"][0]["artifact"]["relations"];
        assert_eq!(relations[0]["view"]["table_projection"]["tables"][0]["rows"].as_array().unwrap().len(),2);
        let fixed=&relations[1]["view"]["table_projection"]["tables"][0];
        assert_eq!(fixed["columns"].as_array().unwrap().len(),1);
        assert_eq!(fixed["rows"][0]["cells"][0]["value_canon"],"2@m");
    }

    #[test]
    fn c3_native_display_rebuilds_typed_history_from_actual_prefix_and_rejects_other_history() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let source=SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\",모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")],시작=(모듈=앱,정의=실행)) 얽음.".into()},
            SourceFile{path:"main.ddn".into(),text:"채비 { 값:수 := 1. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. }. }. (거리:값 * 1@m,완료:거짓) 내보임. 보기 := (관계들=차림[(보기=(열들=차림[관찰.완료,관찰.거리]) 표준보기.결과표) 표준보개.으뜸,(보기=(열들=차림[관찰.거리]) 표준보기.결과표,자리=#오른쪽,시간=((번호=0) 표준시간.고정마디)) 표준보개.곁]) 표준보개.짜임.".into()}]};
        let artifact=ProjectArtifact::create("v1-core-v25".into(),source,String::new()).unwrap().save().unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(artifact.as_bytes()).unwrap();
        let age=crate::ddn_runtime::default_age_target();
        let run=run_general_input_prefix_from_owned(owned.clone(),&artifact,3,0,None,None,age).unwrap();
        assert!(rebuild_native_table_history(&owned,&run.rows,0,0,age,None,run.execution.world.world(),
            run.execution.runner.bogae_history_for_resume()).unwrap().is_some());
        assert!(rebuild_native_table_history(&owned,&run.rows,0,0,age,None,run.execution.world.world(),
            &crate::bogae_time::SceneHistory::default()).is_err());
        assert!(rebuild_native_table_history(&owned,&run.rows,1,0,age,None,run.execution.world.world(),
            run.execution.runner.bogae_history_for_resume()).is_err());
    }
}
