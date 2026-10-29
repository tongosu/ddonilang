//! Actual OS child termination, not a returned-error substitute. This first
//! matrix covers one source-parent world only, not CLI3/Chrome or power loss.
use super::*;
use crate::snapshot_v2_durable_transaction::native::crash_probe;
use std::{path::Path, process::{Child, Command, Stdio}, time::{Duration, Instant}};

struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); }
}

#[test]
#[ignore = "parent arms one private test-build kill stage"]
fn wo22_native_store_kill_child() {
    let case = std::path::PathBuf::from(std::env::var_os("DDN_WO22_CASE").expect("explicit case"));
    let stage = std::env::var("DDN_WO22_STAGE").expect("explicit stage");
    let source = "채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.";
    let (root, _, saved) = super::tests::fixture(source);
    let locator = saved.grant_locator.as_deref().unwrap();
    let execution = begin_first_authored_source_native(&root, &saved.project_identity, locator,
        &saved.revision_identity, &saved.revision_integrity).unwrap().execute(2, 53, None, None).unwrap();
    let prepared = execution.prepare(&[2]).unwrap();
    let db = rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let source_bytes: Vec<u8> = db.query_row("SELECT project_payload_bytes FROM vc04_prepared_payloads WHERE payload_kind=0", [], |r| r.get(0)).unwrap();
    drop(db);
    let metadata = json!({"root":root,"locator":locator,"artifact":prepared.artifact,
        "world_hash":prepared.run.summary["state_hash"],
        "source_hash":blake3::hash(&source_bytes).to_hex().to_string()});
    std::fs::write(case.join("case.json"), serde_json::to_vec(&metadata).unwrap()).unwrap();
    crash_probe::arm(&stage, case.join("reached"));
    let mut retained = prepared.write_prepared().unwrap().commit().unwrap();
    // The sixth point is after an actual checked receipt, not merely after
    // CAS or a JSON success flag. Nothing exposes a pre-commit world.
    let artifact = retained.saved_runtime_artifact().unwrap();
    let bytes = crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes()).unwrap();
    let owned = crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes).unwrap();
    let source = crate::unit_wire::project_runtime_attachment::check_saved_source_context(&bytes, &owned, &[1], &[2]).unwrap();
    let admitted = retained._adapter.readmit_first_source_world_source_context(locator, &source).unwrap();
    let frame = crate::unit_wire::check_resumable_complete_state_layout_v2(&admitted.payload().snapshot, source.pesi().as_raw()).unwrap();
    let context = crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(&bytes, &owned, &[1], &[2], frame.world).unwrap();
    let receipt = admitted.current_receipt(&retained._adapter, &context).unwrap();
    std::fs::write(case.join("receipt.bin"), receipt).unwrap();
    crash_probe::checkpoint("after_receipt");
    panic!("selected kill stage was not reached: {stage}");
}

#[test]
#[ignore = "parent invokes a distinct process after the producer was killed"]
fn wo22_native_store_reopen_child() {
    let case = std::path::PathBuf::from(std::env::var_os("DDN_WO22_CASE").unwrap());
    let stage = std::env::var("DDN_WO22_STAGE").unwrap();
    let metadata: JsonValue = serde_json::from_slice(&std::fs::read(case.join("case.json")).unwrap()).unwrap();
    let restored = resume_first_saved_source_world_native(metadata["artifact"].as_str().unwrap(), &[1], &[2],
        Path::new(metadata["root"].as_str().unwrap()), metadata["locator"].as_str().unwrap(), 0);
    let result = if stage == "after_receipt" {
        let restored = restored.unwrap(); assert_eq!(restored["state_hash"], metadata["world_hash"]);
        assert!(case.join("receipt.bin").is_file());
        json!({"outcome":"NORMAL_COMMITTED_WORLD","state_hash":restored["state_hash"]})
    } else {
        let error = restored.unwrap_err();
        assert!(error.contains("RecoveryRequired"), "explicit recovery cause missing at {stage}: {error}");
        assert!(!case.join("receipt.bin").exists());
        json!({"outcome":"RECOVERY_REQUIRED_NO_WORLD_PUBLISHED","error":error})
    };
    std::fs::write(case.join("reopened.json"), serde_json::to_vec(&result).unwrap()).unwrap();
}

#[test]
fn wo22_six_actual_process_kills_preserve_source_and_never_publish_partial_world() {
    let mut nonce = [0u8; 16]; getrandom::fill(&mut nonce).unwrap();
    let directory = std::env::temp_dir().join(format!("ddn-wo22-kill-{}", hex::encode(nonce)));
    std::fs::create_dir(&directory).unwrap();
    let mut rows = Vec::new();
    for (stage, head, custody) in [("before_prepared", 1u8, 0i64), ("after_prepared", 1, 1),
        ("before_cas", 1, 2), ("after_cas", 2, 2), ("after_confirmation", 2, 2), ("after_receipt", 2, 3)] {
        let case = directory.join(stage); std::fs::create_dir(&case).unwrap();
        let log = std::fs::File::create(case.join("child.log")).unwrap();
        let mut child = OwnedChild(Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "runtime_surface::native_source_world::wo22_crash_tests::wo22_native_store_kill_child", "--ignored", "--nocapture"])
            .env("DDN_WO22_CASE", &case).env("DDN_WO22_STAGE", stage)
            .stdout(Stdio::from(log.try_clone().unwrap())).stderr(Stdio::from(log)).spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(20);
        while !case.join("reached").is_file() {
            assert!(child.0.try_wait().unwrap().is_none(), "child exited before {stage}; see {}", case.display());
            assert!(Instant::now() < deadline, "kill rendezvous timeout at {stage}");
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(std::fs::read_to_string(case.join("reached")).unwrap(), stage);
        let pid = child.0.id(); child.0.kill().unwrap();
        let status = child.0.wait().unwrap(); assert!(!status.success(), "normal exit is not a forced kill");
        let metadata: JsonValue = serde_json::from_slice(&std::fs::read(case.join("case.json")).unwrap()).unwrap();
        let root = Path::new(metadata["root"].as_str().unwrap());
        let db = rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        assert_eq!(db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0)).unwrap(), "ok");
        let actual_head: Vec<u8> = db.query_row("SELECT history_revision_identity FROM vc04_project_heads", [], |r| r.get(0)).unwrap();
        assert_eq!(actual_head, [head], "head at {stage}");
        let source_bytes: Vec<u8> = db.query_row("SELECT project_payload_bytes FROM vc04_prepared_payloads WHERE payload_kind=0", [], |r| r.get(0)).unwrap();
        assert_eq!(blake3::hash(&source_bytes).to_hex().as_str(), metadata["source_hash"].as_str().unwrap());
        let actual_custody: i64 = db.query_row("SELECT count(*) FROM vc04_recovery_custody_v1", [], |r| r.get(0)).unwrap();
        assert_eq!(actual_custody, custody, "custody at {stage}"); drop(db);
        let reopened = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "runtime_surface::native_source_world::wo22_crash_tests::wo22_native_store_reopen_child", "--ignored", "--nocapture"])
            .env("DDN_WO22_CASE", &case).env("DDN_WO22_STAGE", stage).output().unwrap();
        std::fs::write(case.join("reopened.stdout"), &reopened.stdout).unwrap();
        std::fs::write(case.join("reopened.stderr"), &reopened.stderr).unwrap();
        assert!(reopened.status.success(), "fresh process failed at {stage}: {}", String::from_utf8_lossy(&reopened.stdout));
        let cold: JsonValue = serde_json::from_slice(&std::fs::read(case.join("reopened.json")).unwrap()).unwrap();
        rows.push(json!({"stage":stage,"pid":pid,"actual_kill":true,"source_preserved":true,
            "fresh_process":true,"head":head,"custody_rows":custody,"cold":cold}));
    }
    let report = json!({"claim":"SIX_TEST_BINARY_PROCESS_KILLS_ONE_NATIVE_SOURCE_PARENT_ONLY",
        "cli3": "NOT_RUN", "chrome":"NOT_RUN", "power_loss":"NOT_RUN", "rows":rows});
    std::fs::write(directory.join("B2_KILL_MATRIX.partial.json"), serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("WO22_KILL_MATRIX {}", directory.join("B2_KILL_MATRIX.partial.json").display());
}

#[test]
#[ignore = "parent owns an actual second-transaction kill stage"]
fn wo22_next_store_kill_child() {
    let case=std::path::PathBuf::from(std::env::var_os("DDN_WO22_CASE").unwrap());
    let stage=std::env::var("DDN_WO22_STAGE").unwrap();
    let (root,_,saved)=super::tests::fixture("채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.");
    let locator=saved.grant_locator.as_deref().unwrap();
    let mut first=begin_first_authored_source_native(&root,&saved.project_identity,locator,&[1],&saved.revision_integrity)
        .unwrap().execute(2,53,None,None).unwrap().prepare(&[2]).unwrap().write_prepared().unwrap().commit().unwrap();
    let old=first.saved_runtime_artifact().unwrap().to_owned();
    let old_hash=first.summary().unwrap()["state_hash"].clone();
    let reference=first.resume_checkpoint(&[1],&[2],locator,1).unwrap();
    let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let prior:Vec<u8>=db.query_row("SELECT canonical_snapshot_v2_bytes FROM vc04_prepared_payloads WHERE payload_kind=1",[],|r|r.get(0)).unwrap();
    drop(db);
    let summary=first.save_continuation(&[1],&[2],&[3],locator,1,|candidate,digest|{
        let data=json!({"root":root,"locator":locator,"old":old,"next":candidate,"old_hash":old_hash,
            "next_hash":reference["state_hash"],"next_digest":hex::encode(digest),"prior_snapshot_hash":blake3::hash(&prior).to_hex().to_string()});
        std::fs::write(case.join("case.json"),serde_json::to_vec(&data).unwrap()).unwrap();
        crash_probe::arm(&stage,case.join("reached"));Ok(())
    }).unwrap();
    assert_eq!(summary["state_hash"],reference["state_hash"]);
    let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(first.artifact.as_bytes()).unwrap();
    let owned=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes).unwrap();
    let source=crate::unit_wire::project_runtime_attachment::check_saved_source_context(&bytes,&owned,&[2],&[3]).unwrap();
    let admitted=first._adapter.readmit_current_source_world_source_context(locator,&source).unwrap();
    let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&admitted.payload().snapshot,source.pesi().as_raw()).unwrap();
    let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(&bytes,&owned,&[2],&[3],layout.world).unwrap();
    let receipt=admitted.current_receipt(&first._adapter,&context).unwrap();
    std::fs::write(case.join("receipt.bin"),receipt).unwrap();
    crash_probe::checkpoint("after_receipt");
    panic!("second transaction kill not reached");
}

#[test]
#[ignore = "fresh cold observer, not a recovery authority producer"]
fn wo22_next_store_cold_child() {
    let case=std::path::PathBuf::from(std::env::var_os("DDN_WO22_CASE").unwrap());
    let data:JsonValue=serde_json::from_slice(&std::fs::read(case.join("case.json")).unwrap()).unwrap();
    let stage=std::env::var("DDN_WO22_STAGE").unwrap();
    let mut rows=Vec::new();
    for (key,parent,revision,expected) in [("old",1u8,2u8,"old_hash"),("next",2,3,"next_hash")] {
        let result=resume_first_saved_source_world_native(data[key].as_str().unwrap(),&[parent],&[revision],
            Path::new(data["root"].as_str().unwrap()),data["locator"].as_str().unwrap(),0);
        let row=match result {
            Ok(summary)=>{
                assert_eq!(summary["state_hash"],data[expected]);
                assert!((key=="old"&&matches!(stage.as_str(),"before_prepared"|"after_prepared"|"before_cas"))
                    ||(key=="next"&&stage=="after_receipt"),"unexpected world publication: {key}/{stage}");
                json!({"candidate":key,"outcome":"NORMAL_VERIFIED_WORLD","state_hash":summary["state_hash"]})
            },
            Err(error)=>json!({"candidate":key,"outcome":"REJECTED_NO_WORLD","error":error}),
        };rows.push(row);
    }
    if stage=="after_receipt"{assert_eq!(rows[1]["outcome"],"NORMAL_VERIFIED_WORLD");}
    let pre=matches!(stage.as_str(),"before_prepared"|"after_prepared"|"before_cas");
    let goal_match=if pre{rows[0]["outcome"]=="NORMAL_VERIFIED_WORLD"}else{true};
    std::fs::write(case.join("cold.json"),serde_json::to_vec_pretty(&json!({"rows":rows,
        "b2_04_pre_cas_prior_normal":if pre{Some(goal_match)}else{None},"claim":"OBSERVED_CONTRACT_OUTCOMES_NOT_WO22_COMPLETE"})).unwrap()).unwrap();
}

#[test]
fn wo22_next_transaction_six_actual_kills_observe_prior_and_candidate() {
    let mut nonce=[0u8;16];getrandom::fill(&mut nonce).unwrap();
    let directory=std::env::temp_dir().join(format!("ddn-wo22-next-kill-{}",hex::encode(nonce)));
    std::fs::create_dir(&directory).unwrap();let mut rows=Vec::new();
    for (stage,head) in [("before_prepared",2u8),("after_prepared",2),("before_cas",2),
        ("after_cas",3),("after_confirmation",3),("after_receipt",3)]{
        let case=directory.join(stage);std::fs::create_dir(&case).unwrap();
        let log=std::fs::File::create(case.join("child.log")).unwrap();
        let mut child=OwnedChild(Command::new(std::env::current_exe().unwrap())
            .args(["--exact","runtime_surface::native_source_world::wo22_crash_tests::wo22_next_store_kill_child","--ignored","--nocapture"])
            .env("DDN_WO22_CASE",&case).env("DDN_WO22_STAGE",stage)
            .stdout(Stdio::from(log.try_clone().unwrap())).stderr(Stdio::from(log)).spawn().unwrap());
        let deadline=Instant::now()+Duration::from_secs(60);
        while !case.join("reached").is_file(){assert!(child.0.try_wait().unwrap().is_none(),"child failed at {}",case.display());
            assert!(Instant::now()<deadline,"second kill rendezvous timeout at {}",case.display());std::thread::sleep(Duration::from_millis(30));}
        assert_eq!(std::fs::read_to_string(case.join("reached")).unwrap(),stage);
        let pid=child.0.id();child.0.kill().unwrap();assert!(!child.0.wait().unwrap().success());
        let data:JsonValue=serde_json::from_slice(&std::fs::read(case.join("case.json")).unwrap()).unwrap();
        let db=rusqlite::Connection::open_with_flags(Path::new(data["root"].as_str().unwrap()).join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let actual:Vec<u8>=db.query_row("SELECT history_revision_identity FROM vc04_project_heads",[],|r|r.get(0)).unwrap();assert_eq!(actual,[head]);
        let snapshots:Vec<Vec<u8>>=db.prepare("SELECT canonical_snapshot_v2_bytes FROM vc04_prepared_payloads WHERE payload_kind=1").unwrap()
            .query_map([],|r|r.get(0)).unwrap().collect::<Result<_,_>>().unwrap();
        assert!(snapshots.iter().any(|v|blake3::hash(v).to_hex().as_str()==data["prior_snapshot_hash"].as_str().unwrap()));drop(db);
        let cold=Command::new(std::env::current_exe().unwrap())
            .args(["--exact","runtime_surface::native_source_world::wo22_crash_tests::wo22_next_store_cold_child","--ignored","--nocapture"])
            .env("DDN_WO22_CASE",&case).env("DDN_WO22_STAGE",stage).output().unwrap();
        std::fs::write(case.join("cold.stdout"),&cold.stdout).unwrap();std::fs::write(case.join("cold.stderr"),&cold.stderr).unwrap();
        assert!(cold.status.success(),"cold observer failed at {}: {}",case.display(),String::from_utf8_lossy(&cold.stdout));
        let observed:JsonValue=serde_json::from_slice(&std::fs::read(case.join("cold.json")).unwrap()).unwrap();
        rows.push(json!({"stage":stage,"pid":pid,"actual_kill":true,"head":head,"prior_snapshot_bytes_preserved":true,"cold":observed}));
        std::fs::write(directory.join("B2_NEXT_KILL_MATRIX.partial.json"),serde_json::to_vec_pretty(&json!({
            "claim":"TEST_BINARY_ACTUAL_SECOND_TRANSACTION_KILLS_OBSERVATION_NOT_B2_PASS","rows":rows})).unwrap()).unwrap();
    }
    println!("WO22_NEXT_KILL_MATRIX {}",directory.join("B2_NEXT_KILL_MATRIX.partial.json").display());
}
