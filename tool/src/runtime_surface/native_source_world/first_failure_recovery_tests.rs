use super::*;
use super::tests::fixture;

#[test]
fn first_failure_recovery_reads_actual_aborted_source_and_requires_fresh_explicit_selection(){
    let(root,artifact,saved)=fixture("명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 값 <- 1 / 0. }.");
    let locator=saved.grant_locator.as_deref().unwrap();
    let session=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap();
    assert!(session.execute(1,53,None,None).is_err());
    assert!(begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).is_err());
    let(read,selection,recorded)=read_failed_first_source_native(&root,&saved.project_identity,locator).unwrap();
    assert_eq!(read.project_payload_utf8,saved.project_payload_utf8);
    assert_eq!(selection.kind,2);assert_eq!(read.revision_identity,saved.revision_identity);
    let(_,same_selection,same_recorded)=read_failed_first_source_native(&root,&saved.project_identity,locator).unwrap();
    assert_eq!(same_selection,selection);assert_eq!(same_recorded,recorded,"read must not append an attempt");
    let mut stale=selection.clone();stale.key[0]^=1;
    assert!(begin_retry_first_source_native(&root,&saved.project_identity,locator,&artifact,&stale).is_err());
    assert_eq!(read_failed_first_source_native(&root,&saved.project_identity,locator).unwrap().1,selection);
    let retry=begin_retry_first_source_native(&root,&saved.project_identity,locator,&artifact,&selection).unwrap();
    assert!(retry.execute(1,53,None,None).is_err());
    let(_,new_selection,new_recorded)=read_failed_first_source_native(&root,&saved.project_identity,locator).unwrap();
    assert_ne!(new_selection.key,selection.key);assert_eq!(new_recorded,recorded);
    assert!(begin_retry_first_source_native(&root,&saved.project_identity,locator,&artifact,&selection).is_err());
}

#[test]
fn first_failure_recovery_commits_same_identity_and_then_refuses_recovery_of_a_world(){
    let(root,artifact,saved)=fixture("실행:움직씨 := { 7 보여주기. }.");
    let locator=saved.grant_locator.as_deref().unwrap();
    // An interrupted pre-world session is closed by releasing the actual
    // exclusive owner. No persisted caller flag or fabricated failure is used.
    drop(begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap());
    let(_,selection,_)=read_failed_first_source_native(&root,&saved.project_identity,locator).unwrap();
    let retry=begin_retry_first_source_native(&root,&saved.project_identity,locator,&artifact,&selection).unwrap();
    let run=retry.execute(1,0,None,None).unwrap().prepare(&[2]).unwrap().write_prepared().unwrap().commit().unwrap();
    assert!(run.retained.outcome().failure().is_none());
    assert_eq!(run.summary().unwrap()["output_log_texts"],serde_json::json!(["7"]));
    let world=run.saved_runtime_artifact().unwrap().to_owned();drop(run);
    assert!(read_failed_first_source_native(&root,&saved.project_identity,locator).is_err());
    assert!(begin_retry_first_source_native(&root,&saved.project_identity,locator,&artifact,&selection).is_err());
    // Ordinary committed-world restore remains the real source/world path.
    let result=resume_first_saved_source_world_native(&world,&saved.revision_identity,&[2],&root,locator,1).unwrap();
    assert!(result.is_object());
}

#[test]
fn first_failure_recovery_refuses_pending_custody_and_unstarted_sources(){
    let(root,artifact,saved)=fixture("실행:움직씨 := { 7 보여주기. }.");
    let locator=saved.grant_locator.as_deref().unwrap();
    assert!(read_failed_first_source_native(&root,&saved.project_identity,locator).is_err());
    let prepared=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap()
        .execute(1,0,None,None).unwrap().prepare(&[2]).unwrap().write_prepared().unwrap();
    drop(prepared);
    assert!(read_failed_first_source_native(&root,&saved.project_identity,locator).is_err());
    let selection=NativeAuthoringSelection{project:saved.project_identity.clone(),revision:saved.revision_identity.clone(),integrity:saved.revision_integrity,key:[0;32],kind:2};
    assert!(begin_retry_first_source_native(&root,&saved.project_identity,locator,&artifact,&selection).is_err());
}
