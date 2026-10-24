//! Four non-overlapping actual child kill boundaries, exact R1.1 §7.
//! These test-only fixtures are not repair permits or production fault hooks.
use super::*;
use std::{process::{Command,Stdio},time::{Instant,Duration}};

#[test]
#[ignore="parent owns this test child and exact fixture"]
fn authoring_kill_child(){
    let case=std::path::PathBuf::from(std::env::var_os("DDN_A_CASE").unwrap());
    let stage=std::env::var("DDN_A_STAGE").unwrap();
    let (root,original,saved)=super::tests::fixture("채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.");
    let locator=saved.grant_locator.as_deref().unwrap();
    let run=begin_first_authored_source_native(&root,&saved.project_identity,locator,&saved.revision_identity,&saved.revision_integrity).unwrap()
        .execute(1,0,None,None).unwrap().prepare(&[2]).unwrap().write_prepared().unwrap().commit().unwrap();
    let old=run.saved_runtime_artifact().unwrap().to_owned();drop(run);
    let selected=native_authoring_selection(&root,&saved.project_identity,locator,&original).unwrap();
    let parsed:crate::authored_project::ProjectArtifact=serde_json::from_str(&original).unwrap();let mut snapshot=parsed.source_snapshot;
    snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text.push_str("\n// 새 실행\n");
    let edited=crate::authored_project::operate("update",&json!({"original_json":original,"source_snapshot":snapshot}).to_string()).unwrap();
    let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let old_snapshot:Vec<u8>=db.query_row("SELECT canonical_snapshot_v2_bytes FROM vc04_prepared_payloads WHERE new_history_revision_identity=?1",[&[2u8][..]],|r|r.get(0)).unwrap();drop(db);
    std::fs::write(case.join("case.json"),serde_json::to_vec(&json!({"root":root,"project":saved.project_identity,"locator":locator,
        "original":original,"edited":edited,"old_artifact":old,"old_snapshot_hex":hex::encode(old_snapshot)})).unwrap()).unwrap();
    if stage=="authoring_after_source_cas"{crate::snapshot_v2_durable_transaction::native::crash_probe::arm(&stage,case.join("reached"));}
    save_changed_authored_source_native(&edited,&original,&[3],&root,locator,&selected).unwrap();
    if stage=="authoring_invalid_transition"{
        let adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&root).unwrap();
        adapter.invalidate_source_transition_for_test(locator).unwrap();drop(adapter);
        crate::snapshot_v2_durable_transaction::native::crash_probe::arm(&stage,case.join("reached"));
        crate::snapshot_v2_durable_transaction::native::crash_probe::checkpoint(&stage);unreachable!();
    }
    let ready=native_authoring_selection(&root,&saved.project_identity,locator,&edited).unwrap();
    crate::snapshot_v2_durable_transaction::native::crash_probe::arm(&stage,case.join("reached"));
    begin_new_authored_source_native(&root,&saved.project_identity,locator,&edited,&ready).unwrap().execute(1,0,None,None).unwrap()
        .prepare(&[4]).unwrap().write_prepared().unwrap().commit().unwrap();
    panic!("requested boundary was not reached");
}

#[test]
#[ignore="fresh child reads only the parent's exact killed fixture"]
fn authoring_reopen_child(){
    let case=std::path::PathBuf::from(std::env::var_os("DDN_A_CASE").unwrap());
    let v:JsonValue=serde_json::from_slice(&std::fs::read(case.join("case.json")).unwrap()).unwrap();
    let project:Vec<u8>=serde_json::from_value(v["project"].clone()).unwrap();let root=std::path::Path::new(v["root"].as_str().unwrap());
    let locator=v["locator"].as_str().unwrap();let edited=v["edited"].as_str().unwrap();
    let result=native_authoring_selection(root,&project,locator,edited);
    assert!(result.is_err(),"an uncertain or damaged transition must not admit a source/Run");
    let old=resume_first_saved_source_world_native(v["old_artifact"].as_str().unwrap(),&[1],&[2],root,locator,0);
    assert!(old.is_err(),"must not silently select the old world after uncertain authoring");
    std::fs::write(case.join("cold.json"),serde_json::to_vec(&json!({"source_error":result.err(),"old_world_error":old.err(),"success_published":false})).unwrap()).unwrap();
}

#[test]
fn authoring_actual_four_kills_and_fresh_process_rejection(){
    let mut nonce=[0;16];getrandom::fill(&mut nonce).unwrap();
    let out=std::env::temp_dir().join(format!("ddn-authoring-four-kills-{}",hex::encode(nonce)));std::fs::create_dir(&out).unwrap();let mut rows=vec![];
    for stage in ["authoring_after_source_cas","authoring_after_consume","after_cas","authoring_invalid_transition"]{
        let case=out.join(stage);std::fs::create_dir(&case).unwrap();let log=std::fs::File::create(case.join("child.log")).unwrap();
        let mut child=Command::new(std::env::current_exe().unwrap()).args(["--exact","runtime_surface::native_source_world::authoring_crash_tests::authoring_kill_child","--ignored","--nocapture"])
            .env("DDN_A_CASE",&case).env("DDN_A_STAGE",stage).stdout(Stdio::from(log.try_clone().unwrap())).stderr(Stdio::from(log)).spawn().unwrap();
        let deadline=Instant::now()+Duration::from_secs(50);
        while !case.join("reached").is_file(){
            if child.try_wait().unwrap().is_some(){panic!("child exited before boundary: {}",case.display());}
            if Instant::now()>deadline{let _=child.kill();let _=child.wait();panic!("kill boundary timeout: {}",case.display());}
            std::thread::sleep(Duration::from_millis(30));
        }
        let pid=child.id();child.kill().unwrap();let status=child.wait().unwrap();assert!(!status.success());
        let v:JsonValue=serde_json::from_slice(&std::fs::read(case.join("case.json")).unwrap()).unwrap();let root=std::path::Path::new(v["root"].as_str().unwrap());
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        assert_eq!(db.query_row("PRAGMA integrity_check",[],|r|r.get::<_,String>(0)).unwrap(),"ok");
        let head:Vec<u8>=db.query_row("SELECT history_revision_identity FROM vc04_project_heads",[],|r|r.get(0)).unwrap();
        assert_eq!(head,if stage=="after_cas"{vec![4]}else{vec![3]});
        let old:Vec<u8>=db.query_row("SELECT canonical_snapshot_v2_bytes FROM vc04_prepared_payloads WHERE new_history_revision_identity=?1",[&[2u8][..]],|r|r.get(0)).unwrap();
        assert_eq!(hex::encode(old),v["old_snapshot_hex"].as_str().unwrap());drop(db);
        let cold=Command::new(std::env::current_exe().unwrap()).args(["--exact","runtime_surface::native_source_world::authoring_crash_tests::authoring_reopen_child","--ignored","--nocapture"])
            .env("DDN_A_CASE",&case).output().unwrap();std::fs::write(case.join("cold.stdout"),&cold.stdout).unwrap();std::fs::write(case.join("cold.stderr"),&cold.stderr).unwrap();
        assert!(cold.status.success(),"cold refusal failed: {}",case.display());
        let observation:JsonValue=serde_json::from_slice(&std::fs::read(case.join("cold.json")).unwrap()).unwrap();
        rows.push(json!({"stage":stage,"pid":pid,"actual_kill":true,"old_payload_unchanged":true,"cold":observation}));
    }
    std::fs::write(out.join("RESULT.json"),serde_json::to_vec_pretty(&json!({"claim":"FOUR_TEST_BINARY_AUTHORING_KILLS_NOT_POWER_LOSS_OR_CLI3","rows":rows})).unwrap()).unwrap();
    println!("AUTHORING_FOUR_KILLS_REPORT={}",out.display());
}
