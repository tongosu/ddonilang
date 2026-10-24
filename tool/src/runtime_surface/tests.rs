//! Existing tests; namespace and included source bytes preserved.
    #[test]
    fn authored_single_file_bogae_runs_without_a_runtime_storage_root() {
        use super::*;
        let source=include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha2_linear_composition.ddn");
        let request=json!({"source_profile":"v1-core-v25","module_name":"앱",
            "source_snapshot":{"entry_path":"lesson.ddn","files":[{"path":"lesson.ddn","text":source}]}});
        let original=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let registered=crate::authored_project::operate("register_bogae",&json!({
            "original_json":original,"module_name":"앱","source_path":"lesson.ddn",
            "composition_definition":"일차함수짜임","make_entry":true}).to_string()).unwrap();
        let stored=crate::authored_project::open(registered.as_bytes()).unwrap();
        assert!(stored.source_snapshot.root_path.is_none());
        let temporary=run_summary_with_bogae_entry(source,"lesson.ddn",Some(1),"v1-core-v25","일차함수짜임").unwrap();
        let actual=run_summary_with_authored_bogae(source,"lesson.ddn",Some(1),"v1-core-v25",&registered,None,None).unwrap();
        assert_eq!(actual["state_hash"],temporary["state_hash"]);
        assert_eq!(actual["bogae_projection"]["persistent_identity"],true);
        assert!(crate::project_revision_envelope_supply::admit_fresh_authored_project(registered.as_bytes())
            .unwrap_err().starts_with("E_PROJECT_RUNTIME_ROOT_REQUIRED:"),"execution must not invent storage custody");
    }
    #[cfg(not(target_arch="wasm32"))]
    #[test]
    fn general_native_actual_sqlite_save_cold_restore_and_stale_failure_preserves_old_copy() {
        use super::*;
        use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":"채비 { 값:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }."}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root=std::env::temp_dir().join(format!("ddn-general-native-{}-{nonce}",std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let saved=record_authored_general_project_native(&authored,2,7,&[],&[9],&root,true,None).unwrap();
        let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(saved.saved_artifact.as_bytes()).unwrap();
        let owned=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes).unwrap();
        let project=hex::decode(owned.project().authored_identity.logical_dependency_id.as_str()).unwrap();
        let store=SqliteAdapterV1::open_admitted(&root).unwrap();
        let before=store.reread_committed(&saved.transaction_digest,&project,&[9]).unwrap();
        drop(store); // release the single-writer root lease before cold reopen
        let resumed=resume_saved_general_project_artifact_native(
            &saved.saved_artifact,&[],&[9],&root,&saved.transaction_digest,2).unwrap();
        let uninterrupted=run_authored_general_input_prefix(&authored,4,7,None,None).unwrap();
        let retained=run_general_input_prefix_from_owned(owned.clone(),&authored,2,7,None,None,
            crate::ddn_runtime::default_age_target()).unwrap();
        let previous_work=retained.execution.runner.work_cost_receipt();
        let (_,mut continued,actual_inputs)=resume_saved_project_artifact_from_runner(
            &saved.saved_artifact,&[],&[9],2,Some((retained.execution.runner,crate::ddn_runtime::default_age_target()))).unwrap();
        let work=continued.runner.work_cost_receipt();
        assert!(work.counters.base_dispatches>previous_work.counters.base_dispatches);
        assert_eq!(work.counters.base_dispatches,uninterrupted.execution.runner.work_cost_receipt().counters.base_dispatches);
        assert_eq!(work.counters.committed_madi,4);
        assert_eq!(work.counters.sealed_input_bytes,previous_work.counters.sealed_input_bytes);
        assert_eq!(work.counters.call_depth_current,0);
        assert_eq!(continued.next_tick,4);
        assert_eq!(actual_inputs,uninterrupted.rows[2..]);
        assert_eq!(current_state_hash(continued.world.world()),uninterrupted.summary()["state_hash"]);
        // A continued carrier uses the actual retained world/runner, not a
        // newly admitted fresh source or a copied pre-continuation carrier.
        let second=continued.export_continued_artifact(&owned,&[9],&[10]).unwrap();
        let second_bytes=crate::unit_wire::project_runtime_attachment::check_bytes(second.as_bytes()).unwrap();
        let second_owned=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&second_bytes).unwrap();
        let second_context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context(
            &second_bytes,&second_owned,&[9],&[10]).unwrap();
        assert_eq!(second_context.pesi().as_raw(),crate::project_snapshot_identity::current::issue_saved_local_project(&owned).unwrap().pesi().as_raw());
        let after_second=resume_saved_project_artifact(&second,&[9],&[10],2).unwrap();
        let six=run_authored_general_input_prefix(&authored,6,7,None,None).unwrap();
        assert_eq!(after_second["next_tick_id"],6);
        assert_eq!(after_second["state_hash"],six.summary()["state_hash"]);
        assert_eq!(after_second["resources"],six.summary()["engine_state"]["resources"]);
        let original_runtime=continued.saved_runtime.as_ref().unwrap().clone();
        let unchanged_world=current_state_hash(continued.world.world());
        continued.next_tick+=1;
        assert!(continued.export_continued_artifact(&owned,&[9],&[10]).is_err());
        continued.next_tick-=1;
        assert_eq!(continued.export_continued_artifact(&owned,&[9],&[10]).unwrap(),second);
        continued.saved_runtime.as_mut().unwrap().value_mut()["state"]["snapshot_v2"]["resources"]["fixed64"]["값"]="999".into();
        assert!(continued.export_continued_artifact(&owned,&[9],&[10]).is_err());
        assert_eq!(current_state_hash(continued.world.world()),unchanged_world);
        continued.saved_runtime=Some(original_runtime);
        assert_eq!(continued.export_continued_artifact(&owned,&[9],&[10]).unwrap(),second);
        assert_eq!(resumed["next_tick_id"],4);
        assert_eq!(resumed["state_hash"],uninterrupted.summary()["state_hash"]);
        let input=crate::input_receipt::parse_receipt_snapshot(&resumed["frames"][0]["input_receipt"]).unwrap();
        assert_eq!(input.tick_id,2);assert_eq!(input.rng_seed,7^2);
        // A fresh stale project save may prepare a candidate, but cannot
        // replace the old head or return a partially published success.
        assert!(record_authored_general_project_native(&authored,1,7,&[],&[10],&root,false,None).is_err());
        let store=SqliteAdapterV1::open_admitted(&root).unwrap();
        let after=store.reread_committed(&saved.transaction_digest,&project,&[9]).unwrap();
        assert_eq!(after.payload,before.payload);
        assert_eq!(after.revision,before.revision);
        assert_eq!(after.head,before.head);
        drop(store);
        assert!(resume_saved_general_project_artifact_native(&saved.saved_artifact,&[],&[9],
            &root,&[0;32],2).is_err());
        let store=SqliteAdapterV1::open_admitted(&root).unwrap();
        let after_rejection=store.reread_committed(&saved.transaction_digest,&project,&[9]).unwrap();
        assert_eq!(after_rejection.payload,before.payload);
        assert_eq!(after_rejection.revision,before.revision);
        assert_eq!(after_rejection.head,before.head);
        drop(store);
        let second_native=resume_and_save_general_project_native(&saved.saved_artifact,&[],&[9],
            &root,&saved.transaction_digest,2,&[10],&saved.committed_receipt).unwrap();
        assert_eq!(second_native.summary["state_hash"],uninterrupted.summary()["state_hash"]);
        let native_six=resume_saved_general_project_artifact_native(&second_native.saved_artifact,&[9],&[10],
            &root,&second_native.transaction_digest,2).unwrap();
        assert_eq!(native_six["state_hash"],six.summary()["state_hash"]);
        assert_eq!(native_six["next_tick_id"],6);
        assert!(resume_and_save_general_project_native(&second_native.saved_artifact,&[9],&[10],
            &root,&second_native.transaction_digest,1,&[11],&saved.committed_receipt).is_err());
        let third_native=resume_and_save_general_project_native(&second_native.saved_artifact,&[9],&[10],
            &root,&second_native.transaction_digest,1,&[11],&second_native.committed_receipt).unwrap();
        let third_six=resume_saved_general_project_artifact_native(&third_native.saved_artifact,&[10],&[11],
            &root,&third_native.transaction_digest,1).unwrap();
        assert_eq!(third_six["state_hash"],six.summary()["state_hash"]);
        assert_eq!(third_six["resources"],six.summary()["engine_state"]["resources"]);
        // Keep this small owned test store as evidence; no shared path removal.
        eprintln!("general Native evidence store: {}",root.display());
    }

    /// Real source/capture/input admission and SQLite persistence, with a real
    /// forced child-process kill. This is a test-build Native execution proof,
    /// not a shipped CLI3, power-loss or recovery-record/receipt oracle.
    #[cfg(windows)]
    #[test]
    fn native_general_save_forced_process_kill_preserves_old_and_raw_new() {
        use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
        use crate::unit_wire::project_runtime_attachment as attachment;
        const ROOT: &str = "DDN_NATIVE_OWNER_TEST_ROOT";
        if let Some(root) = std::env::var_os(ROOT) {
            let root = std::path::PathBuf::from(root);
            let authored = std::fs::read_to_string(root.join("authored.fixture")).unwrap();
            let receipt = std::fs::read(root.join("old-receipt.fixture")).unwrap();
            record_authored_general_project_native(&authored,4,7,&[9],&[10],&root,false,Some(&receipt)).unwrap();
            std::fs::write(root.join("unexpected-success.fixture"), b"published").unwrap();
            panic!("interrupted transaction unexpectedly returned success");
        }
        // Other ddn_runtime unit tests temporarily change the process-global
        // default AGE. Their mutex is not shared with this module, and a new
        // child starts with its own default. Run the coordinator in its own
        // exact-test process so both producers have identical execution policy.
        // Never relax a digest assertion to mask different execution profiles.
        if std::env::var_os("DDN_NATIVE_OWNER_TEST_COORDINATOR").is_none() {
            let result=std::process::Command::new(std::env::current_exe().unwrap())
                .arg("--exact").arg("runtime_surface::tests::native_general_save_forced_process_kill_preserves_old_and_raw_new")
                .arg("--nocapture").env("DDN_NATIVE_OWNER_TEST_COORDINATOR","1").output().unwrap();
            eprintln!("{}",String::from_utf8_lossy(&result.stdout));
            assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
            return;
        }
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":"채비 { 값:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }."}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let four=run_authored_general_input_prefix(&authored,4,7,None,None).unwrap();
        let candidate=four.export_saved_artifact(&[9],&[10]).unwrap();
        let checked=attachment::check_bytes(candidate.as_bytes()).unwrap();
        let context=attachment::check_saved_revision_context(&checked,four.source_owner(),&[9],&[10]).unwrap();
        let raw=four.encode_checkpoint_request(&context,&[9],&[10]).unwrap();
        let admitted=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2(&raw,context.pesi().as_raw()).unwrap();
        let next_digest=admitted.binding.transaction_digest().unwrap();
        let project=admitted.binding.project.clone();
        for phase in ["before_prepared_commit","after_prepared_commit","after_head_commit","after_container_confirmation"] {
            let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let root=std::env::temp_dir().join(format!("ddn-native-forced-kill-{}-{nonce}",std::process::id()));
            std::fs::create_dir(&root).unwrap();
            let old=record_authored_general_project_native(&authored,2,7,&[],&[9],&root,true,None).unwrap();
            let adapter=SqliteAdapterV1::open_admitted(&root).unwrap();
            let before=adapter.reread_committed(&old.transaction_digest,&project,&[9]).unwrap();
            drop(adapter);
            // Test evidence only, never adapter recovery metadata or authority.
            std::fs::write(root.join("authored.fixture"),&authored).unwrap();
            std::fs::write(root.join("old-receipt.fixture"),&old.committed_receipt).unwrap();
            std::fs::write(root.join("expected-request.fixture"),&raw).unwrap();
            let marker=root.join("paused.fixture");
            let mut child=std::process::Command::new(std::env::current_exe().unwrap())
                .arg("--exact").arg("runtime_surface::tests::native_general_save_forced_process_kill_preserves_old_and_raw_new")
                .env(ROOT,&root).env("DDN_NATIVE_OWNER_TEST_PHASE",phase).env("DDN_NATIVE_OWNER_TEST_MARKER",&marker)
                .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().unwrap();
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(15);
            while !marker.exists() && std::time::Instant::now()<deadline && child.try_wait().unwrap().is_none() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let reached=marker.exists();
            // Always reap the test-owned child, including the timeout path.
            if child.try_wait().unwrap().is_none() { child.kill().unwrap(); }
            let status=child.wait().unwrap();
            assert!(reached,"child did not reach {phase}; {status}");
            assert!(!status.success());
            assert!(!root.join("unexpected-success.fixture").exists());
            let adapter=SqliteAdapterV1::open_admitted(&root).unwrap();
            let after=adapter.reread_committed(&old.transaction_digest,&project,&[9]).unwrap();
            assert_eq!(after.payload,before.payload,"old payload preserved: {phase}");
            assert_eq!(after.revision,before.revision,"old revision preserved: {phase}");
            if phase=="before_prepared_commit" || phase=="after_prepared_commit" {
                assert_eq!(after.head,before.head,"CAS has not occurred: {phase}");
                drop(adapter);
                let resumed=resume_saved_general_project_artifact_native(&old.saved_artifact,&[],&[9],&root,&old.transaction_digest,2).unwrap();
                assert_eq!(resumed["state_hash"],four.summary()["state_hash"]);
            } else {
                let next=adapter.reread_committed(&next_digest,&project,&[10]).unwrap();
                assert_eq!(next.head,next.revision);
                let head=next.head.as_ref().unwrap();
                if head.digest!=next_digest {
                    let actual=adapter.reread_prepared(&head.digest).unwrap().unwrap();
                    eprintln!("expected binding {:?}; actual binding {:?}; root {}",admitted.binding,actual.binding,root.display());
                    std::fs::write(root.join("actual-snapshot.fixture"),actual.snapshot).unwrap();
                }
                assert_eq!(head.digest,next_digest);
                drop(adapter);
                // NEW_UNCONFIRMED only. Do not call ordinary restore to bypass
                // RD-669's retained recovery record/token admission. Their
                // Native persisted producer is still an explicit open gate.
            }
            eprintln!("forced termination {phase}: {}",root.display());
        }
    }
    #[cfg(not(target_arch="wasm32"))]
    #[test]
    fn n1_rational_tuple_native_sqlite_save_reopen_continue_resave() {
        let body="채비 { 진행 := 0. }. 정확:나눔수 := 0.1. 실행:움직씨 := { 진행 <- 진행+1. 묶음값 <- (진행,정확,참). 묶음값[1] 보여주기. }.";
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/root.ddn")},
                {"path":"main.ddn","text":body}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root=std::env::temp_dir().join(format!("ddn-rational-native-{}-{nonce}",std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let saved=record_authored_general_project_native(&authored,3,7,&[],&[9],&root,true,None).unwrap();
        let resumed=resume_saved_general_project_artifact_native(&saved.saved_artifact,&[],&[9],&root,&saved.transaction_digest,4).unwrap();
        let continuous=run_authored_general_input_prefix(&authored,7,7,None,None).unwrap();
        assert_eq!(resumed["state_hash"],continuous.summary()["state_hash"]);
        assert_eq!(resumed["resources"],continuous.summary()["engine_state"]["resources"]);
        assert_eq!(resumed["next_tick_id"],7);
        let second=resume_and_save_general_project_native(&saved.saved_artifact,&[],&[9],&root,
            &saved.transaction_digest,4,&[10],&saved.committed_receipt).unwrap();
        let reopened=resume_saved_general_project_artifact_native(&second.saved_artifact,&[9],&[10],&root,&second.transaction_digest,1).unwrap();
        let eight=run_authored_general_input_prefix(&authored,8,7,None,None).unwrap();
        assert_eq!(reopened["state_hash"],eight.summary()["state_hash"]);
        assert_eq!(reopened["resources"],eight.summary()["engine_state"]["resources"]);
        eprintln!("Rational Native evidence store: {}",root.display());
    }

    #[cfg(windows)]
    #[test]
    fn m2_actual_runtime_frontdoor_save_cold_resume_and_refuse_bad_inputs() {
        use super::*;
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/root.ddn")},
                {"path":"main.ddn","text":"채비 { 진행:바른수 := 0. }.\n실행:움직씨 := { (매마디)마다 { 진행 <- 진행 + 1. 진행 보여주기. }. }.\n"}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
        let root=std::env::temp_dir().join(format!("ddn-m2-runtime-{}",hex::encode(nonce)));
        std::fs::create_dir(&root).unwrap();
        let prepared=root.with_extension("prepared");
        let saved=record_authored_general_project_custody_native(&authored,3,17,&[9],&root,Some(&prepared)).unwrap();
        assert_eq!(std::fs::read(prepared.join("request.bin")).unwrap(),saved.recovery_request);
        assert_eq!(std::fs::read_to_string(prepared.join("saved.json")).unwrap(),saved.checkpoint.saved_artifact);
        let resume=|raw:&[u8],token:&[u8;33]|resume_saved_general_project_custody_native(
            &saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,raw,token,4);
        let continued=resume(&saved.recovery_request,&saved.recovery_token).unwrap();
        let continuous=run_authored_general_input_prefix(&authored,7,17,None,None).unwrap();
        assert_eq!(continued["state_hash"],continuous.summary()["state_hash"]);
        assert_eq!(continued["resources"],continuous.summary()["engine_state"]["resources"]);
        assert_eq!(continued["next_tick_id"],7);
        let mut wrong_token=saved.recovery_token;wrong_token[32]^=1;
        assert!(resume(&saved.recovery_request,&wrong_token).is_err());
        let mut wrong_request=saved.recovery_request.clone();*wrong_request.last_mut().unwrap()^=1;
        assert!(resume(&wrong_request,&saved.recovery_token).is_err());
        assert!(record_authored_general_project_custody_native(&authored,3,17,&[9],&root,Some(&prepared)).is_err(),"existing root must not be re-enrolled");
        let unchanged=resume(&saved.recovery_request,&saved.recovery_token).unwrap();
        assert_eq!(unchanged["state_hash"],continued["state_hash"]);
        println!("M2_RUNTIME_FRONTDOOR_ROOT {}",root.display());
    }

    #[cfg(windows)]
    #[test]
    #[ignore="only an explicitly spawned interruption child enters this test"]
    fn m2_actual_runtime_interruption_child() {
        let directory=std::path::PathBuf::from(std::env::var_os("DDN_M2_RUNTIME_INTERRUPT_DIR").expect("exact own interruption directory"));
        if std::env::var("DDN_M2_RUNTIME_INTERRUPT_MODE").unwrap()=="initial" {
            let authored=std::fs::read_to_string(directory.join("authored.json")).unwrap();
            super::record_authored_general_project_custody_native(&authored,3,0,&[9],&directory.join("root"),Some(&directory.join("prepared"))).unwrap();
        }else{
            let artifact=std::fs::read_to_string(directory.join("saved.json")).unwrap();
            let raw=std::fs::read(directory.join("request.bin")).unwrap();
            let token:[u8;33]=std::fs::read(directory.join("token.bin")).unwrap().try_into().unwrap();
            let locator=std::fs::read_to_string(directory.join("locator.txt")).unwrap();
            super::resume_saved_general_project_custody_native(&artifact,&[9],&directory.join("root"),&locator,&raw,&token,4).unwrap();
        }
        std::fs::write(directory.join("unexpected-success.fixture"),b"returned").unwrap();
        panic!("owned interruption child unexpectedly returned success");
    }

    #[cfg(windows)]
    #[test]
    fn m2_actual_runtime_frontdoor_logical_tetris_fixed_seed() {
        use super::*;
        use crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1;
        let authored=include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/authored-project.json");
        let initial:crate::logical_input_tape::LogicalRecordRequest=serde_json::from_str(
            include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/initial-input.json")).unwrap();
        let next:crate::logical_input_tape::LogicalRecordRequest=serde_json::from_str(
            include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/next-input.json")).unwrap();
        let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
        let root=std::env::temp_dir().join(format!("ddn-m2-logical-{}",hex::encode(nonce)));std::fs::create_dir(&root).unwrap();
        let prepared=root.with_extension("prepared");
        let saved=record_authored_logical_project_custody_native(authored,&initial,&[9],&root,Some(&prepared)).unwrap();
        let restored=resume_saved_logical_project_custody_native(&saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,
            &saved.recovery_request,&saved.recovery_token,&next).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes()).unwrap();
        let mut continuous=CheckpointedLogicalRunV1::fresh(&owned,initial.binding.clone(),crate::ddn_runtime::default_age_target()).unwrap();
        for snapshot in initial.snapshots.iter().chain(next.snapshots.iter()) {continuous.step(snapshot).unwrap();}
        assert_eq!(restored,continuous.session().summary());
        let mut foreign=next.clone();foreign.binding.initial_rng^=1;
        assert!(resume_saved_logical_project_custody_native(&saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,
            &saved.recovery_request,&saved.recovery_token,&foreign).is_err());
        assert!(resume_saved_general_project_custody_native(&saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,
            &saved.recovery_request,&saved.recovery_token,4).is_err(),"logical history must not enter the general input supplier");
        let again=resume_saved_logical_project_custody_native(&saved.checkpoint.saved_artifact,&[9],&root,&saved.grant_locator,
            &saved.recovery_request,&saved.recovery_token,&next).unwrap();
        assert_eq!(again,restored);
        println!("M2_LOGICAL_RUNTIME_ROOT {}",root.display());
    }

    #[cfg(not(target_arch="wasm32"))]
    #[test]
    fn vc13_tuple_native_sqlite_save_cold_resume_and_resave() {
        use super::*;
        let request=json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/root.ddn")},
                {"path":"main.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/main.ddn")} ]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root=std::env::temp_dir().join(format!("ddn-tuple-native-{}-{nonce}",std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let saved=record_authored_general_project_native(&authored,3,7,&[],&[9],&root,true,None).unwrap();
        let resumed=resume_saved_general_project_artifact_native(&saved.saved_artifact,&[],&[9],&root,&saved.transaction_digest,4).unwrap();
        let uninterrupted=run_authored_general_input_prefix(&authored,7,7,None,None).unwrap();
        assert_eq!(resumed["state_hash"],uninterrupted.summary()["state_hash"]);
        assert_eq!(resumed["resources"],uninterrupted.summary()["engine_state"]["resources"]);
        assert_eq!(resumed["next_tick_id"],7);
        assert_eq!(resumed["resources"]["fixed64"]["진행"],"7");
        let second=resume_and_save_general_project_native(&saved.saved_artifact,&[],&[9],&root,
            &saved.transaction_digest,4,&[10],&saved.committed_receipt).unwrap();
        let reopened=resume_saved_general_project_artifact_native(&second.saved_artifact,&[9],&[10],&root,&second.transaction_digest,1).unwrap();
        let eight=run_authored_general_input_prefix(&authored,8,7,None,None).unwrap();
        assert_eq!(reopened["state_hash"],eight.summary()["state_hash"]);
        assert_eq!(reopened["resources"],eight.summary()["engine_state"]["resources"]);
        assert_eq!(reopened["resources"]["fixed64"]["진행"],"8");
        eprintln!("Tuple Native evidence store: {}",root.display());
    }

    #[cfg(not(target_arch="wasm32"))]
    #[test]
    fn vc13_declared_call_tuple_native_cold_resume_and_resave() {
        let body = include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/declared_call.ddn");
        let request = json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple/root.ddn")},
                {"path":"main.ddn","text":body}]}});
        let authored = crate::authored_project::operate("create", &request.to_string()).unwrap();
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let store=std::env::temp_dir().join(format!("ddn-declared-call-tuple-{}-{nonce}",std::process::id()));
        std::fs::create_dir(&store).unwrap();
        let saved=record_authored_general_project_native(&authored,3,7,&[],&[9],&store,true,None).unwrap();
        let resumed=resume_saved_general_project_artifact_native(&saved.saved_artifact,&[],&[9],&store,&saved.transaction_digest,4).unwrap();
        let continuous=run_authored_general_input_prefix(&authored,7,7,None,None).unwrap();
        assert_eq!(resumed["state_hash"],continuous.summary()["state_hash"]);
        assert_eq!(resumed["resources"],continuous.summary()["engine_state"]["resources"]);
        let second=resume_and_save_general_project_native(&saved.saved_artifact,&[],&[9],&store,
            &saved.transaction_digest,4,&[10],&saved.committed_receipt).unwrap();
        let reopened=resume_saved_general_project_artifact_native(&second.saved_artifact,&[9],&[10],&store,&second.transaction_digest,1).unwrap();
        let eight=run_authored_general_input_prefix(&authored,8,7,None,None).unwrap();
        assert_eq!(reopened["state_hash"],eight.summary()["state_hash"]);
        assert_eq!(reopened["resources"],eight.summary()["engine_state"]["resources"]);
        eprintln!("declared-call Tuple evidence store: {}",store.display());
    }

    #[test]
    fn general_input_supplier_uses_actual_runner_rows_and_never_publishes_failed_prefix() {
        use super::*;
        use crate::input_tape::{mask_to_bytes, InputEvent, InputEventKind};
        use crate::unit_wire::resumable_complete_state::{decode_general_processed_inputs,GeneralProcessedInputRow};
        let source = include_str!("../../../tools/teul-cli/tests/fixtures/cli_control_input_r1.ddn");
        let tape = InputTape { version:2,madi_hz:60,records:vec![
            InputRecord { madi:0,held_mask:mask_to_bytes(2),events:vec![InputEvent {sequence:0,key_bit:2,kind:InputEventKind::Press}] },
            InputRecord { madi:1,held_mask:mask_to_bytes(0),events:vec![InputEvent {sequence:1,key_bit:2,kind:InputEventKind::Release}] },
            InputRecord { madi:2,held_mask:mask_to_bytes(0),events:vec![] },
        ] };
        let profile = resolve_supported_source_profile(Some("v1-core-v25")).unwrap();
        let mut actual = Vec::new();
        let result = run_summary_with_general_input_supplier(source,"supplier.ddn",Some(3),None,None,None,
            Some(profile),Some(&tape),7,DEFAULT_UPDATE_NAME,None,false,None,None,None,None,Some(&mut actual),None,None,None).unwrap();
        let old = run_summary_from_source_with_authored_bogae(source,"supplier.ddn",Some(3),None,None,None,
            Some(profile),Some(&tape),7,DEFAULT_UPDATE_NAME,None,false,None,None,None,None).unwrap();
        assert_eq!(result,old,"old summaries, hash and work evidence are unchanged");
        assert_eq!(actual.len(),3);
        assert_eq!(actual.iter().map(|row|row.input.rng_seed).collect::<Vec<_>>(),vec![7,6,5]);
        for (row,record) in actual.iter().zip(&tape.records) {
            assert_eq!(row.edges,Some(record.runtime_edges()));
        }
        let bytes = crate::unit_wire::resumable_complete_state::encode_general_processed_inputs(&actual,0,7).unwrap();
        assert_eq!(decode_general_processed_inputs(&bytes,0,7).unwrap(),actual);
        let sentinel = GeneralProcessedInputRow { input:actual[0].input.clone(),edges:None };
        let mut published = vec![sentinel];
        let mut polling = |madi| if madi==1 { Err("E_SAM_LIVE_QUEUE_OVERFLOW 입력 실패".into()) }
            else { Ok(tape.records[madi as usize].clone()) };
        assert!(run_summary_with_general_input_supplier(source,"supplier.ddn",Some(3),None,None,None,
            Some(profile),None,7,DEFAULT_UPDATE_NAME,Some((60,&mut polling)),false,None,None,None,None,
            Some(&mut published),None,None,None).is_err());
        assert_eq!(published.len(),1);
        assert_eq!(published[0].edges,None,"first successful madi must not escape a later failed run");
        assert_eq!(published[0].input,actual[0].input);
    }

    #[test]
    fn general_input_supplier_binds_actual_named_project_source_and_xor_seed() {
        use super::*;
        let request = json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":"채비 { 값:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }."}]}});
        let artifact = crate::authored_project::operate("create",&request.to_string()).unwrap();
        let mut prefix = run_authored_general_input_prefix(&artifact,2,7,None,None).unwrap();
        prefix.check_reconstructed_prefix().unwrap();
        let descriptor=prefix.source_rebuild_descriptor_supply().unwrap();
        let reconstructed=crate::unit_wire::resumable_complete_state::rebuild_source_descriptor_candidate(
            &descriptor,prefix.source_owner()).unwrap();
        assert_eq!(reconstructed.successful_madi_count,2);
        assert_eq!(reconstructed.base_rng_seed,7);
        assert_eq!(reconstructed.execution_instance,prefix.execution.runner.execution_instance_owner_bytes());
        let receipt=prefix.execution.runner.work_cost_receipt();
        let work=crate::unit_wire::resumable_complete_state::encode_work_receipt_supply(&receipt).unwrap();
        crate::unit_wire::resumable_complete_state::check_replayed_work_supply(&work,&receipt).unwrap();
        assert_eq!(prefix.source_owner().project().save().unwrap(),artifact);
        let rows = crate::unit_wire::resumable_complete_state::decode_general_processed_inputs(&prefix.input_supply().unwrap(),0,7).unwrap();
        assert_eq!(rows.len(),2);
        assert!(rows.iter().all(|row|row.edges.is_none()));
        assert_eq!(rows.iter().map(|row|row.input.rng_seed).collect::<Vec<_>>(),vec![7,6]);
        let original_frame=prefix.encode_world_frame().unwrap();
        // An unused input can leave world/hash unchanged. Full actual input
        // custody must nevertheless reject it, then accept exact restoration.
        prefix.rows[0].input.pointer_x_i32=19;
        assert!(prefix.check_reconstructed_prefix().unwrap_err().contains("input"));
        assert_eq!(prefix.encode_world_frame().unwrap(),original_frame);
        prefix.rows[0].input.pointer_x_i32=0;
        prefix.check_reconstructed_prefix().unwrap();
        prefix.rows[0].edges=Some((0,0));
        assert!(prefix.check_reconstructed_prefix().unwrap_err().contains("input"));
        prefix.rows[0].edges=None;
        prefix.check_reconstructed_prefix().unwrap();
        // Direct snapshots are not rejected just because their origin differs.
        // Replay retains that origin and its sealed-input byte/work evidence.
        let snapshots=rows.iter().map(|row|row.input.clone()).collect::<Vec<_>>();
        let direct=run_authored_general_input_prefix(&artifact,2,7,Some(&snapshots),None).unwrap();
        direct.check_reconstructed_prefix().unwrap();
        use crate::input_tape::mask_to_bytes;
        let tape=InputTape {version:2,madi_hz:60,records:vec![
            InputRecord {madi:0,held_mask:mask_to_bytes(0),events:vec![]},
            InputRecord {madi:1,held_mask:mask_to_bytes(0),events:vec![]},
        ]};
        let recorded=run_authored_general_input_prefix(&artifact,2,7,None,Some(&tape)).unwrap();
        recorded.check_reconstructed_prefix().unwrap();
        assert!(recorded.rows.iter().all(|row|row.edges==Some((0,0))));
        assert_eq!(prefix.summary()["source_profile_identity"],"v1-core-v25");
        assert!(!prefix.summary()["state_hash"].as_str().unwrap().is_empty());
        let preparation = prefix.source_owner().sources().preparation();
        let public = run_summary_with_authored_bogae(&preparation.source,&preparation.target_path,
            Some(2),"v1-core-v25",&artifact,None,None).unwrap();
        assert_eq!(public["state_hash"],prefix.summary()["state_hash"]);
        let changed = rows.iter().map(|row| { let mut input=row.input.clone(); input.rng_seed=7; input }).collect::<Vec<_>>();
        assert!(run_authored_general_input_prefix(&artifact,2,7,Some(&changed),None).is_err());
        // Existing execution of those snapshots is not changed to XOR seeds.
        let profile = resolve_supported_source_profile(Some("v1-core-v25")).unwrap();
        let mut ordinary_inputs = Vec::new();
        let _ordinary = run_summary_with_general_input_supplier(&preparation.source,&preparation.target_path,
            Some(2),None,None,None,Some(profile),None,7,"실행",None,true,Some(&changed),None,None,
            Some(&artifact),Some(&mut ordinary_inputs),Some(prefix.source_owner()),None,None).unwrap();
        assert_eq!(ordinary_inputs.len(),changed.len());
        assert!(ordinary_inputs.iter().zip(&changed).all(|(row,input)|row.input==*input));
        let frame = prefix.encode_world_frame().unwrap();
        let issued = crate::project_snapshot_identity::current::issue_saved_local_project(prefix.source_owner()).unwrap();
        let checked_frame = crate::unit_wire::check_complete_state_frame_v1(&frame,issued.pesi().as_raw()).unwrap();
        assert_eq!(checked_frame.next_tick,2);
        let saved = prefix.export_saved_artifact(&[],&[9]).unwrap();
        let attachment = crate::unit_wire::project_runtime_attachment::check_bytes(saved.as_bytes()).unwrap();
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context(
            &attachment,prefix.source_owner(),&[],&[9]).unwrap();
        let mut supplies:[Vec<u8>;10]=std::array::from_fn(|_|Vec::new());
        use crate::unit_wire::resumable_complete_state as complete;
        supplies[0]=context.revision_envelope().to_vec();
        supplies[1]=complete::encode_source_capture_supply(prefix.source_owner().sources()).unwrap();
        supplies[2]=complete::encode_constituent_supply(prefix.source_owner()).unwrap();
        supplies[3]=complete::encode_execution_contract_supply(prefix.source_owner(),&prefix.execution.runner).unwrap();
        supplies[4]=prefix.input_supply().unwrap();
        supplies[9]=prefix.source_rebuild_descriptor_supply().unwrap();
        // This is only the static source/input gate; empty boundary supplies
        // cannot pass full request admission or authorize Native I/O.
        fn static_layout<'a>(supplies:&'a [Vec<u8>;10],frame:&'a [u8],pesi:&'a [u8;32])
            ->crate::unit_wire::ResumableCompleteStateLayoutV2<'a> {
            crate::unit_wire::ResumableCompleteStateLayoutV2 {
                world:crate::unit_wire::check_complete_state_frame_v1(frame,pesi).unwrap(),
                supplies:std::array::from_fn(|i|supplies[i].as_slice()),
            }
        }
        let static_pesi=*issued.pesi().as_raw();
        let layout=static_layout(&supplies,&frame,&static_pesi);
        assert_eq!(complete::rebuild_general_complete_state_source_candidate(&layout,&context).unwrap().successful_madi_count,2);
        assert!(complete::rebuild_complete_state_source_candidate(&layout,&context).is_err(),"general input must not enter logical admission");
        supplies[3]=complete::encode_logical_execution_contract_supply(prefix.source_owner(),&prefix.execution.runner).unwrap();
        assert!(complete::rebuild_general_complete_state_source_candidate(&static_layout(&supplies,&frame,&static_pesi),&context).is_err());
        supplies[3]=complete::encode_execution_contract_supply(prefix.source_owner(),&prefix.execution.runner).unwrap();
        assert!(complete::rebuild_general_complete_state_source_candidate(&static_layout(&supplies,&frame,&static_pesi),&context).is_ok());
        let continued = resume_saved_project_artifact(&saved,&[],&[9],2).unwrap();
        let uninterrupted = run_authored_general_input_prefix(&artifact,4,7,None,None).unwrap();
        assert_eq!(continued["next_tick_id"],4);
        assert_eq!(continued["state_hash"],uninterrupted.summary()["state_hash"]);
        let resumed_input = crate::input_receipt::parse_receipt_snapshot(&continued["frames"][0]["input_receipt"]).unwrap();
        assert_eq!(resumed_input.tick_id,2);
        assert_eq!(resumed_input.rng_seed,7^2);
    }
    #[test]
    fn deep_result_artifact_finalizer_accepts_exact_limit_and_cleans_rejection() {
        use super::*;
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let make_runner = |limit| {
                let program = DdnProgram::from_source("(매마디)마다 { }.", "deep-result.ddn").unwrap();
                DdnRunner::new(program, "매틱").with_standard_resource_limits(
                    StandardResourceLimitsV1 { result_artifact_bytes:limit, ..StandardResourceLimitsV1::CANDIDATE })
            };
            let make_summary = |runner: &DdnRunner| {
                let mut nested = JsonValue::Bool(true);
                for _ in 0..20_000 { nested = JsonValue::Array(vec![nested]); }
                crate::unit_wire::SnapshotJsonOwner::from_value(JsonValue::Object([
                    ("resources".into(), nested),
                    ("work_cost_receipt".into(), serde_json::to_value(runner.work_cost_receipt()).unwrap()),
                ].into_iter().collect()))
            };
            let mut zero = make_runner(0);
            let before = zero.work_cost_receipt().counters.result_artifact_bytes;
            let failure = finalize_run_summary_result_artifact(make_summary(&zero), &mut zero, "deep-result.ddn").unwrap_err();
            assert_eq!(failure.code, "E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE");
            assert_eq!(zero.work_cost_receipt().counters.result_artifact_bytes, before);
            // Iterate only the limit's decimal-width effect to find its exact fixed point.
            let mut exact = 100_000;
            for _ in 0..16 {
                let mut runner = make_runner(exact);
                let summary = make_summary(&runner);
                let finalized = crate::unit_wire::SnapshotJsonOwner::from_value(
                    finalize_run_summary_result_artifact(summary, &mut runner, "deep-result.ddn").unwrap());
                let actual = serialized_json_byte_count(&finalized).unwrap() as u64;
                assert_eq!(runner.work_cost_receipt().counters.result_artifact_bytes, actual);
                if actual == exact { break; }
                exact = actual;
            }
            let mut runner = make_runner(exact);
            let finalized = crate::unit_wire::SnapshotJsonOwner::from_value(
                finalize_run_summary_result_artifact(make_summary(&runner), &mut runner, "deep-result.ddn").unwrap());
            assert_eq!(serialized_json_byte_count(&finalized).unwrap() as u64, exact);
            let mut below = make_runner(exact - 1);
            assert!(finalize_run_summary_result_artifact(make_summary(&below), &mut below, "deep-result.ddn").is_err());
            drop(finalized);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn result_byte_count_and_resource_patch_handle_deep_values_without_buffering() {
        use super::*;
        for value in [json!(null), json!({"a":["한글\n😀", 1.25, false]}), json!({"type":"bytes_v1"})] {
            assert_eq!(serialized_json_byte_count(&value).unwrap(), serde_json::to_vec(&value).unwrap().len());
        }
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = ResourceValue::Bool(true);
            for _ in 0..20_000 { value = ResourceValue::List(vec![value]); }
            let mut patch = Patch::default();
            patch.ops.push(PatchOp::SetResourceValue { tag:"tree".into(), value });
            let payload = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_patch(&patch));
            let text = payload.compact_text();
            assert_eq!(serialized_json_byte_count(&payload).unwrap(), text.len());
            assert_eq!(payload[0]["op"], json!("set_resource_value"));
            let cloned = crate::unit_wire::SnapshotJsonOwner::clone_from_value(&payload);
            assert!(cloned.equals_json(&payload));
            let mut world = DetNuri::new();
            let PatchOp::SetResourceValue { value, .. } = &patch.ops[0] else { panic!() };
            world.world_mut().set_resource_value("tree".into(), value.clone());
            let before = world.world().state_hash();
            let response = crate::unit_wire::SnapshotJsonOwner::from_value(build_engine_state_json(&world, 1, json!({}), &patch));
            assert_eq!(serialized_json_byte_count(&response).unwrap(), response.compact_text().len());
            assert_eq!(world.world().state_hash(), before);
            drop(response); drop(world); drop(cloned); drop(payload); drop(patch);
        }).unwrap().join().unwrap();
        let value = ResourceValue::List(vec![ResourceValue::Bool(true), ResourceValue::String("한글".into())]);
        assert_eq!(resource_value_patch_json("tree", &value).to_string(), json!({
            "op":"set_resource_value", "tag":"tree", "value":value.canon_key(),
            "value_json":resource_value_to_json(&value)}).to_string());
    }

    #[test]
    fn display_projection_preserves_legacy_bytes_and_deep_tree_lifetime() {
        use super::*;
        use ddonirang_core::platform::ResourceMapEntry;
        fn old(value: &ResourceValue) -> JsonValue {
            if let Some(formula) = formula_from_resource_value(value) {
                let (dialect, raw) = formula_summary_parts(&formula);
                return json!({"dialect":dialect,"raw":raw});
            }
            match value {
                ResourceValue::List(items) => JsonValue::Array(items.iter().map(old).collect()),
                ResourceValue::Set(items) => JsonValue::Array(items.values().map(old).collect()),
                ResourceValue::Map(entries) => JsonValue::Object(entries.values().map(|entry|
                    (resource_key_to_string(&entry.key), old(&entry.value))).collect()),
                _ => resource_value_leaf_to_json(value),
            }
        }
        let formula = ResourceValue::Map([
            ("a".into(), ResourceMapEntry { key: ResourceValue::String("__ddn_kind".into()), value: ResourceValue::String("ddn.formula.v1".into()) }),
            ("b".into(), ResourceMapEntry { key: ResourceValue::String("raw".into()), value: ResourceValue::String("x + 1".into()) }),
            ("c".into(), ResourceMapEntry { key: ResourceValue::String("dialect".into()), value: ResourceValue::String("ascii".into()) }),
        ].into_iter().collect());
        assert!(formula_from_resource_value(&formula).is_some());
        let sample = ResourceValue::List(vec![ResourceValue::None, ResourceValue::Bool(true),
            ResourceValue::Fixed64(Fixed64::from_i64(-2)), ResourceValue::String("한글\n🙂".into()),
            ResourceValue::Set([("a".into(), ResourceValue::Bool(false))].into_iter().collect()), formula,
            ResourceValue::Map([
                ("a".into(), ResourceMapEntry { key: ResourceValue::String("same".into()), value: ResourceValue::List(vec![ResourceValue::Bool(false)]) }),
                ("b".into(), ResourceMapEntry { key: ResourceValue::String("same".into()), value: ResourceValue::Bool(true) }),
            ].into_iter().collect())]);
        let actual = crate::unit_wire::SnapshotJsonOwner::from_value(resource_value_to_json(&sample));
        assert_eq!(actual.compact_text(), old(&sample).to_string());
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = ResourceValue::Bool(true);
            for index in 0..20_000 {
                value = match index % 3 {
                    0 => ResourceValue::List(vec![value]),
                    1 => ResourceValue::Set([("x".into(), value)].into_iter().collect()),
                    _ => ResourceValue::Map([("x".into(), ResourceMapEntry { key: ResourceValue::String("x".into()), value })].into_iter().collect()),
                };
            }
            let projected = crate::unit_wire::SnapshotJsonOwner::from_value(resource_value_to_json(&value));
            let text = projected.compact_text();
            assert_eq!(text.matches("[" ).count(), 13_334);
            assert_eq!(text.matches("{\"x\":" ).count(), 6_666);
            // The second display key displaces a deep first value in the actual product map path.
            let collision = ResourceValue::Map([
                ("a".into(), ResourceMapEntry { key: ResourceValue::String("same".into()), value }),
                ("b".into(), ResourceMapEntry { key: ResourceValue::String("same".into()), value: ResourceValue::Bool(false) }),
            ].into_iter().collect());
            let displaced = crate::unit_wire::SnapshotJsonOwner::from_value(resource_value_to_json(&collision));
            assert_eq!(displaced.compact_text(), "{\"same\":false}");
            let mut world = DetNuri::new();
            world.world_mut().set_resource_value("tree".into(), collision);
            let before = world.world().state_hash();
            let resources = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_world_resources(world.world()));
            assert_eq!(resources["value_json"]["tree"]["same"], json!(false));
            assert!(!resources.compact_text().is_empty());
            assert_eq!(world.world().state_hash(), before);
            drop(resources); drop(world); drop(displaced); drop(projected);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn runtime_snapshot_builder_transfers_deep_resources_without_reencoding() {
        use super::*;
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = ResourceValue::Bool(true);
            for _ in 0..20_000 { value = ResourceValue::List(vec![value]); }
            let mut world = DetNuri::new();
            world.world_mut().set_resource_value("tree".into(), value);
            let before = world.world().state_hash();
            let snapshot = crate::unit_wire::SnapshotJsonOwner::from_value(runtime_state_snapshot(&world, vec![], 7));
            let text = snapshot.compact_text();
            assert_eq!(text.matches("\"type\":\"list\"").count(), 20_000);
            assert_eq!(snapshot["next_tick_id"], json!(7));
            assert_eq!(world.world().state_hash(), before);
            // RD-694: exercise the actual shared raw parse/restore path, not
            // just an internal projection that bypasses saved admission.
            let mut parsed = crate::unit_wire::parse_snapshot_json(&text).unwrap();
            let mut restored = DetNuri::new();
            crate::unit_wire::restore_world_resources_snapshot(
                restored.world_mut(), &parsed["resources"]).unwrap();
            assert_eq!(restored.world().state_hash(), before);
            let again = crate::unit_wire::SnapshotJsonOwner::from_value(
                runtime_state_snapshot(&restored, vec![], 7));
            assert_eq!(again.compact_text(), text);
            // A later invalid typed row must not publish the admitted deep
            // first row or replace a pre-existing normal world.
            parsed.value_mut()["resources"]["value_det"]["z"] =
                json!({"type":"bool","value":1});
            let preserved = restored.world().state_hash();
            assert!(crate::unit_wire::restore_world_resources_snapshot(
                restored.world_mut(), &parsed["resources"]).is_err());
            assert_eq!(restored.world().state_hash(), preserved);
            drop(again); drop(restored); drop(parsed);
            drop(snapshot); drop(world);
        }).unwrap().join().unwrap();
        let world = DetNuri::new();
        let expected = json!({"schema":"ddn.wasm_state_snapshot.v2",
            "resource_snapshot_schema":RESOURCE_SNAPSHOT_SCHEMA_V2,"unit_wire_schema":UNIT_VALUE_WIRE_SCHEMA_V2,
            "resources":serialize_world_resources_snapshot(world.world()),"param_overrides":{},
            "public_observation_frames":[],"next_tick_id":7});
        assert_eq!(runtime_state_snapshot(&world, vec![], 7).to_string(), expected.to_string());
    }
    // 이전 제품의 전체 JSON 재직렬화 루프. 실행이 끝난 동일 공개 후보에서
    // 영수증 초기값만 복원하여 최적화 경로와 완성 바이트/실패 영수증을 대조한다.
    fn receipt_full_scan_reference(mut summary: super::JsonValue,
        limits: super::StandardResourceLimitsV1, file_path: &str)
        -> Result<super::JsonValue, super::ExecutionDiagnostic> {
        use super::*;
        let mut counters: crate::work_cost::WorkCostCounters =
            serde_json::from_value(summary["work_cost_receipt"]["counters"].clone()).unwrap();
        counters.result_artifact_bytes = 0;
        let mut meter = crate::work_cost::WorkCostMeter::from_receipt_parts(
            counters, limits, summary["work_cost_receipt"]["rd206_resource_capability"].as_bool().unwrap());
        summary["work_cost_receipt"] = serde_json::to_value(meter.receipt()).unwrap();
        for _ in 0..16 {
            let bytes = serialized_json_byte_count(&summary).map_err(|error|
                run_summary_diagnostic("E_RESULT_ARTIFACT_SERIALIZE", error.to_string(), file_path, "resource_commit"))?;
            if meter.receipt().counters.result_artifact_bytes == bytes as u64 { return Ok(summary); }
            meter.set_result_artifact_bytes(bytes).map_err(|failure|
                work_cost_failure_diagnostic(failure, file_path, "resource_commit"))?;
            summary["work_cost_receipt"] = serde_json::to_value(meter.receipt()).unwrap();
        }
        Err(run_summary_diagnostic("E_RESULT_ARTIFACT_SIZE_FIXED_POINT",
            "결과 artifact byte 크기와 WorkCostV1 영수증이 수렴하지 않았습니다", file_path, "resource_commit"))
    }

    #[test]
    fn receipt_cache_matches_full_scan_source_1_2_40_and_exact_limit_plus_one() {
        use super::*;
        for source in ["(매마디)마다 { }.",
            "(매마디)마다 { 글 <- \"한글🙂\\n\\t\\\"\". 글 보여주기. }."] {
            for ticks in [1, 2, 40] {
                let file = "receipt-cache.ddn";
                let execute = |limits| run_summary_from_source_with_assets_diagnostic(
                    source, file, Some(ticks), None, Some(limits), None, None, None, 0);
                let actual = execute(StandardResourceLimitsV1::CANDIDATE).unwrap();
                let old = receipt_full_scan_reference(actual.clone(), StandardResourceLimitsV1::CANDIDATE, file).unwrap();
                assert_eq!(serde_json::to_vec(&actual).unwrap(), serde_json::to_vec(&old).unwrap());
                assert_eq!(hash_json_string(&actual), hash_json_string(&old));
                // limit의 자릿수도 영수증 바이트에 포함된다. 원 루프로 정확한 경계를 구한다.
                let mut exact = serialized_json_byte_count(&actual).unwrap() as u64;
                for _ in 0..16 {
                    let limits = StandardResourceLimitsV1 { result_artifact_bytes: exact,
                        ..StandardResourceLimitsV1::CANDIDATE };
                    let bounded = receipt_full_scan_reference(actual.clone(), limits, file).unwrap();
                    let next = serialized_json_byte_count(&bounded).unwrap() as u64;
                    if next == exact { break; }
                    exact = next;
                }
                let limits = StandardResourceLimitsV1 { result_artifact_bytes: exact,
                    ..StandardResourceLimitsV1::CANDIDATE };
                let expected = receipt_full_scan_reference(actual.clone(), limits, file).unwrap();
                assert_eq!(serde_json::to_vec(&execute(limits).unwrap()).unwrap(), serde_json::to_vec(&expected).unwrap());
                let below = StandardResourceLimitsV1 { result_artifact_bytes: exact - 1, ..limits };
                let expected_failure = receipt_full_scan_reference(actual.clone(), below, file).unwrap_err();
                let actual_failure = execute(below).unwrap_err();
                assert_eq!(actual_failure.to_json_value(), expected_failure.to_json_value());
                assert_eq!(actual_failure.phase, "resource_commit");
            }
        }
    }

    #[test]
    fn receipt_cache_decomposition_handles_unicode_empty_values_and_checked_overflow() {
        use super::*;
        for receipt in [JsonValue::Null, json!({}), json!(""),
            json!({"counter": 0, "text": "한글🙂\"\\\n\u{0000}"})] {
            for samples in [1, 2, 40] {
                let mut summary = json!({"work_cost_receipt": receipt,
                    "outside": (0..samples).map(|i|json!({"i":i,"text":"공개 🙂\\n"})).collect::<Vec<_>>()});
                let outside = serialized_json_byte_count(&summary).unwrap()
                    .checked_sub(serialized_json_byte_count(&summary["work_cost_receipt"]).unwrap()).unwrap();
                for replacement in [json!({}), json!({"counter": 999999}), json!("새 영수증🙂\n")] {
                    summary["work_cost_receipt"] = replacement;
                    let receipt_bytes = serialized_json_byte_count(&summary["work_cost_receipt"]).unwrap();
                    assert_eq!(outside.checked_add(receipt_bytes).unwrap(), serde_json::to_vec(&summary).unwrap().len());
                }
            }
        }
        let outside = usize::MAX;
        let error = outside.checked_add(1).ok_or_else(|| serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::Other, "JSON byte count overflow"))).unwrap_err();
        let mut counter = JsonByteCounter { bytes: outside };
        let original_error = serde_json::to_writer(&mut counter, &json!("x")).unwrap_err();
        assert_eq!(error.to_string(), original_error.to_string());
        assert_eq!(error.is_io(), original_error.is_io());
        assert_eq!(counter.bytes, usize::MAX);
    }

    // 최적화 전 literal을 그대로 유지한 공개 바이트 oracle.
    fn engine_state_literal_reference(world: &super::DetNuri, tick_id: u64,
        input_json: super::JsonValue, patch: &super::Patch) -> super::JsonValue {
        use super::*;
        let committed_madi = tick_id.saturating_sub(1);
        let resources = serialize_world_resources(world.world());
        let (columns, row) = collect_columns_and_row(world.world());
        let view_meta = build_view_meta(world.world());
        let view_hash = hash_json_string(&view_meta);
        let patch_json = serialize_patch(patch);
        json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "tick_id": tick_id,
            "committed_madi": committed_madi,
            "state_hash": current_state_hash(world.world()),
            "input": input_json.clone(),
            "resources": resources.clone(),
            "channels": columns.clone(),
            "row": row.clone(),
            "streams": JsonValue::Object(Map::new()),
            "patch": patch_json.clone(),
            "state": {
                "tick_id": tick_id,
                "committed_madi": committed_madi,
                "input": input_json,
                "resources": resources,
                "channels": columns,
                "row": row,
                "patch": patch_json,
                "streams": JsonValue::Object(Map::new())
            },
            "view_meta": view_meta,
            "view_hash": view_hash
        })
    }

    fn assert_engine_state_literal_bytes(world: &super::DetNuri, tick: u64,
        input: super::JsonValue, patch: &super::Patch) {
        let before_hash = super::current_state_hash(world.world());
        let expected = engine_state_literal_reference(world, tick, input.clone(), patch);
        let actual = super::build_engine_state_json(world, tick, input, patch);
        assert_eq!(serde_json::to_vec(&actual).unwrap(), serde_json::to_vec(&expected).unwrap());
        assert_eq!(serde_json::to_string_pretty(&actual).unwrap(),
            serde_json::to_string_pretty(&expected).unwrap());
        assert_eq!(super::hash_json_string(&actual), super::hash_json_string(&expected));
        assert_eq!(super::current_state_hash(world.world()), before_hash);
        assert!(actual.as_object().unwrap().contains_key("patch"));
        assert!(actual["state"].as_object().unwrap().contains_key("patch"));
        assert!(actual["patch"].is_array());
        assert_eq!(actual["patch"], actual["state"]["patch"]);
        assert_eq!(actual["patch"].as_array().unwrap().len(), patch.ops.len());
    }

    #[test]
    fn engine_state_patch_move_matches_literal_nested_unicode_unit_shape_and_empty() {
        use super::*;
        let mut world = DetNuri::new();
        let nested = ResourceValue::List(vec![
            ResourceValue::String("한글🙂\"\\\n\t\u{0000}".into()),
            ResourceValue::List(vec![ResourceValue::None, ResourceValue::Bool(true)]),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2), ddonirang_core::Unit::Mole)),
        ]);
        let shape = ResourceValue::map_from_entries(vec![
            ddonirang_core::platform::ResourceMapEntry { key: ResourceValue::String("kind".into()), value: ResourceValue::String("polyline".into()) },
            ddonirang_core::platform::ResourceMapEntry { key: ResourceValue::String("points".into()), value: ResourceValue::List(vec![
                ResourceValue::List(vec![ResourceValue::Fixed64(Fixed64::ZERO),
                    ResourceValue::Fixed64(Fixed64::ONE)]),
            ]) },
        ]);
        let mut patch = Patch::default();
        patch.ops.push(PatchOp::SetResourceValue { tag: "중첩".into(), value: nested.clone() });
        patch.ops.push(PatchOp::SetResourceValue { tag: "그림".into(), value: shape });
        patch.ops.push(PatchOp::SetResourceJson { tag: "json".into(), json: "{\"문장\":\"🙂\\n\"}".into() });
        patch.ops.push(PatchOp::SetResourceValue { tag: "중첩".into(), value: nested });
        world.apply_patch(&patch, 0, &mut ddonirang_core::signals::VecSignalSink::default());
        assert_engine_state_literal_bytes(&world, 1, json!({"text":"입력🙂", "nested":[1,null]}), &patch);
        assert_engine_state_literal_bytes(&world, 0, JsonValue::Null, &Patch::default());
    }

    #[test]
    fn engine_state_patch_move_matches_literal_source_observations_and_receipt() {
        use super::*;
        let source = "채비 { 시간 <- 0. }. (매마디)마다 { 시간 <- 시간 + 1. (시간: 시간 * 1@s, 값: 시간) 내보임. 시간 보여주기. }.";
        let program = DdnProgram::from_source(source, "patch-move.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let mut world = DetNuri::new();
        let mut defaults = HashMap::new();
        seed_bogae_defaults(&mut defaults);
        for tick in 0..3 {
            let input = InputSnapshot { tick_id: tick, dt: Fixed64::ONE, keys_pressed: 0,
                last_key_name: String::new(), pointer_x_i32: 0, pointer_y_i32: 0,
                ai_injections: Vec::new(), net_events: Vec::new(),
                frame_source: InputSource::Person, rng_seed: tick };
            let output = runner.run_update(world.world(), &input, &defaults).unwrap();
            assert!(!output.public_observations.is_empty());
            world.apply_patch(&output.patch, tick, &mut ddonirang_core::signals::VecSignalSink::default());
            let receipt_before = serde_json::to_vec(&runner.work_cost_receipt()).unwrap();
            assert_engine_state_literal_bytes(&world, tick + 1,
                serialize_input_snapshot(&input, Some(0)), &output.patch);
            assert_eq!(serde_json::to_vec(&runner.work_cost_receipt()).unwrap(), receipt_before);
        }
    }

    #[test]
    fn json_byte_count_matches_allocating_serialization() {
        let fixtures = [
            json!("한글🙂\"\\\n\t\u{0000}"),
            json!({"중첩": [null, true, -12.5, {"escaped": "\r\n\\\""}]}),
            json!({"rows": (0..4096).map(|index| json!({
                "index": index, "text": "원 표본과 공개 필드 🙂", "values": [1, 2, 3]
            })).collect::<Vec<_>>()}),
        ];
        for fixture in fixtures {
            assert_eq!(super::serialized_json_byte_count(&fixture).unwrap(),
                serde_json::to_vec(&fixture).unwrap().len());
        }
        let summary = super::run_summary_from_supported_source_profile(
            "(매마디)마다 { a <- -0.08. a 보여주기. }.",
            "byte-count.ddn", Some(2), Some("v1-core-v25"),
        ).expect("public summary executes");
        let bytes = serde_json::to_vec(&summary).unwrap().len();
        assert_eq!(super::serialized_json_byte_count(&summary).unwrap(), bytes);
        assert_eq!(summary["work_cost_receipt"]["counters"]["result_artifact_bytes"], bytes);
    }

    #[test]
    fn summary_json_moves_preserve_all_public_duplicate_slots() {
        let summary = super::run_summary_from_supported_source_profile(
            "(매마디)마다 { 값 <- -0.08. (시간: 1@s, 값: 값) 보여주기. \"한글🙂\\n\" 보여주기. }.",
            "summary-move.ddn", Some(2), Some("v1-core-v25"),
        ).expect("public observations and output execute");
        let state = &summary["engine_state"];
        let mut literal_reference = summary.clone();
        for key in ["resources", "channels", "row", "observation_columns",
            "public_observation_trace", "public_observation_trace_hash",
            "public_observation_table_projection", "output_rows"] {
            assert!(!summary[key].is_null(), "{key} must be published");
            literal_reference[key] = serde_json::to_value(&state[key]).unwrap();
        }
        literal_reference["output_log_texts"] = serde_json::to_value(
            super::output_log_texts_from_state_json(state)).unwrap();
        assert_eq!(serde_json::to_vec(&summary).unwrap(),
            serde_json::to_vec(&literal_reference).unwrap());
        assert_eq!(serde_json::to_vec_pretty(&summary).unwrap(),
            serde_json::to_vec_pretty(&literal_reference).unwrap());
    }

    #[test]
    fn a3_p1_summary_projects_numbers_without_rewriting_world_output() {
        let summary = super::run_summary_from_supported_source_profile(
            "(매마디)마다 { 값 <- 20 - 9.8 * 0.1. 값 보여주기. \"19.019999999\" 보여주기. }.",
            "display-summary.ddn", Some(2), Some("v1-core-v25"),
        ).unwrap();
        let rows = summary["output_display"]["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0]["text"], "19.02");
        assert_eq!(rows[1]["text"], "19.019999999");
        assert_eq!(rows[2]["madi"], 1);
        assert_eq!(rows[2]["line_no"], 3);
        assert_eq!(summary["engine_state"]["output_display"], summary["output_display"]);
        assert_eq!(summary["output_display"]["world_hash_participation"], false);
        assert_ne!(summary["output_log_texts"][0], rows[0]["text"]);
        assert_eq!(summary["output_log_texts"][1], "19.019999999");
    }

    #[test]
    fn json_byte_counter_is_non_allocating_and_overflow_is_atomic() {
        use std::io::Write;
        assert_eq!(std::mem::size_of::<super::JsonByteCounter>(), std::mem::size_of::<usize>());
        let mut counter = super::JsonByteCounter { bytes: usize::MAX - 1 };
        counter.write_all(b"x").unwrap();
        assert_eq!(counter.bytes, usize::MAX);
        let error = serde_json::to_writer(&mut counter, &json!("x")).unwrap_err();
        assert!(error.is_io());
        assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::Other));
        assert_eq!(counter.bytes, usize::MAX);
        assert_eq!(counter.write(b"").unwrap(), 0);
        counter.flush().unwrap();
    }

    #[test]
    fn wo09_signed_fixed64_resource_display_matches_numeric_value() {
        let source = "(매마디)마다 { a <- -0.08. b <- -1.5. c <- -2. d <- 0.25. }.";
        let summary = super::run_summary_from_supported_source_profile(
            source, "wo09-display.ddn", Some(1), Some("v1-core-v25"),
        ).expect("canonical source executes");
        for (name, expected) in [("a", "-0.08"), ("b", "-1.5"), ("c", "-2"), ("d", "0.25")] {
            assert_eq!(summary["resources"]["fixed64"][name], expected, "{name}");
        }
    }

    use super::*;
    use ddonirang_core::KEY_D;

    const P1A_V1_CORE_PROFILE: &str = "v1-core-v25";

    #[test]
    fn wo03_explicit_profile_controls_arithmetic_without_token_inference() {
        for source in [
            "(1 / 0) 보여주기.",
            "값 <- (1 / 0). 값 보여주기.",
            "값 := (1 / 0). 값 보여주기.",
            "(매마디)마다 { (1 / 0) 보여주기. }.",
        ] {
            let error = run_summary_from_supported_source_profile(source, "wo03.ddn", Some(1), Some(P1A_V1_CORE_PROFILE))
                .expect_err("numeric failure is not a successful signal");
            assert!(error.contains("0으로 나눌 수 없습니다"), "{error}");
            assert!(run_summary_from_supported_source_profile(source, "wo03.ddn", Some(1), None).is_err());
        }
        for source in ["(1 / 0) 보여주기.", "값 <- (1 / 0). 값 보여주기."] {
            let error = run_summary_from_source(source, "legacy.ddn", Some(1)).expect_err("legacy numeric fault also fails");
            assert!(error.contains("0으로 나눌 수 없습니다"), "{error}");
        }
    }

    #[test]
    fn wo02_summary_entrypoints_reject_inferred_type_change_before_execution() {
        for source in [
            "값 := 1. 값 <- \"가\". 값 보여주기.",
            "값 <- 1. 값 <- \"가\". 값 보여주기.",
        ] {
            // D7: current sources bind their profile explicitly; the legacy
            // internal path is not authority for V25 validation semantics.
            for profile in [Some(P1A_V1_CORE_PROFILE)] {
                let error = run_summary_from_source_with_assets_diagnostic(
                    source, "wo02-rebind.ddn", Some(1), None, None, None,
                    profile.map(|_| resolve_supported_source_profile(profile).unwrap()),
                    None, 0,
                ).expect_err("all execution frontdoors must reject the same type change");
                assert_eq!(error.phase, "static_validation");
                assert!(error.message.contains("E_"));
            }
        }
    }

    #[test]
    fn wo02_summary_checks_even_skipped_logical_operand_before_execution() {
        for source in ["거짓 그리고 1 보여주기.", "참 또는 1 보여주기."] {
            let error = run_summary_from_source_diagnostic(source, "wo02-logical.ddn", Some(1))
                .expect_err("short circuit does not skip static type checks");
            assert_eq!(error.phase, "static_validation");
        }
    }

    #[test]
    fn shared_sha256_identity_preserves_product_artifact_contract() {
        assert_eq!(
            sha256_identity(b"ddn-artifact-v1"),
            "sha256:dd0db943346466681ea7255089423388fcf354f758febcd3d555a129d31aef16"
        );
    }

    #[test]
    fn explicit_age_target_is_per_invocation_and_fail_closed() {
        let source = "매틱:움직씨 = {\n  기계 <- 상태머신{\n    빨강, 초록 으로 이뤄짐.\n    빨강 으로 시작.\n    빨강 에서 초록 으로.\n  }.\n}\n";
        let before = crate::ddn_runtime::default_age_target();
        let error = run_summary_from_source_with_age_target_diagnostic(
            source,
            "explicit-age-target.ddn",
            Some(1),
            AgeTarget::Age0,
        )
        .expect_err("AGE0 must reject an AGE1 state-machine surface");
        assert_eq!(error.code, "E_AGE_NOT_AVAILABLE");
        assert!(error.message.contains("state_machine_literal"));
        assert_eq!(crate::ddn_runtime::default_age_target(), before);

        let summary = run_summary_from_source_with_age_target_diagnostic(
            source,
            "explicit-age-target.ddn",
            Some(1),
            AgeTarget::Age1,
        )
        .expect("AGE1 must accept the same state-machine source");
        assert_eq!(summary["age_target_identity"], "AGE1");
        assert_eq!(summary["execution_envelope"]["age_target_identity"], "AGE1");
        assert_eq!(crate::ddn_runtime::default_age_target(), before);
    }

    #[test]
    fn p1a_supported_profile_run_executes_v25_definition_and_rebinding() {
        let source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n";
        let summary = run_summary_from_supported_source_profile_diagnostic(
            source,
            "p1a-positive.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect("V1-Core run summary");
        assert!(
            summary["output_log_texts"]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|row| row.as_str() == Some("2"))),
            "summary={summary}"
        );
        assert_eq!(summary["source_profile_identity"], P1A_V1_CORE_PROFILE);
    }

    #[test]
    fn supported_profile_seed_is_sealed_and_changes_run_identity() {
        let source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 보여주기.\n}\n";
        let first = run_summary_from_supported_source_profile_with_seed(
            source,
            "seeded-v25.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
            7,
        )
        .expect("seeded V1-Core run");
        let repeated = run_summary_from_supported_source_profile_with_seed(
            source,
            "seeded-v25.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
            7,
        )
        .expect("repeated seeded V1-Core run");
        let changed = run_summary_from_supported_source_profile_with_seed(
            source,
            "seeded-v25.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
            8,
        )
        .expect("changed-seed V1-Core run");

        assert_eq!(first, repeated);
        assert_eq!(
            first["execution_envelope"]["input_snapshot"]["rng_base_seed"],
            7
        );
        assert_eq!(first["engine_state"]["state"]["input"]["rng_base_seed"], 7);
        assert_ne!(
            first["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"][0]["run_id"],
            changed["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"][0]
                ["run_id"]
        );
    }

    #[test]
    fn live_control_input_and_v2_replay_share_ddn_result_state_and_trace() {
        use crate::input_tape::{mask_to_bytes, InputEvent, InputEventKind};
        let source = include_str!("../../../tools/teul-cli/tests/fixtures/cli_control_input_r1.ddn");
        let tape = InputTape { version: 2, madi_hz: 60, records: vec![
            InputRecord { madi: 0, held_mask: mask_to_bytes(2), events: vec![InputEvent { sequence: 0, key_bit: 2, kind: InputEventKind::Press }] },
            InputRecord { madi: 1, held_mask: mask_to_bytes(0), events: vec![InputEvent { sequence: 1, key_bit: 2, kind: InputEventKind::Release }] },
            InputRecord { madi: 2, held_mask: mask_to_bytes(2), events: vec![InputEvent { sequence: 2, key_bit: 2, kind: InputEventKind::Press }] },
        ] };
        let live = run_summary_from_source_with_live_input(source, "control-input.ddn", Some(3), 0,
            Some(P1A_V1_CORE_PROFILE), 60, |madi| Ok(tape.records[madi as usize].clone())).expect("live DDN execution");
        let bytes = encode_input_tape(&tape).expect("V2 candidate");
        let replay_tape = crate::input_tape::decode_input_tape(&bytes).expect("V2 admission");
        let replay = run_summary_from_supported_source_profile_with_input_tape(source, "control-input.ddn", Some(3), Some(P1A_V1_CORE_PROFILE), &replay_tape).expect("replay");
        assert_eq!(live["output_log_texts"], json!(["참", "거짓", "참"]));
        assert_eq!(live, replay, "all result, input frames, state/trace hashes and receipts must match");
        let failure = run_summary_from_source_with_live_input(source, "control-input.ddn", Some(3), 0,
            Some(P1A_V1_CORE_PROFILE), 60, |madi| {
                if madi == 1 { Err("E_SAM_LIVE_QUEUE_OVERFLOW 입력 대기열 오류".into()) }
                else { Ok(tape.records[madi as usize].clone()) }
            }).expect_err("no partial summary after a later input failure");
        assert!(failure.contains("E_SAM_LIVE_QUEUE_OVERFLOW"));
        assert_eq!(serde_json::from_str::<JsonValue>(&failure).unwrap()["code"], "E_SAM_LIVE_QUEUE_OVERFLOW");
        assert!(!failure.contains("output_log_texts"));
        assert!(!failure.contains("state_hash"));
    }

    #[test]
    fn live_control_same_madi_edges_survive_shared_runtime_and_replay() {
        use crate::input_tape::{mask_to_bytes, InputEvent, InputEventKind};
        let source = "매마디:움직씨 := {\n    누름 <- (\"right\") 눌렸나.\n    순간 <- (\"right\") 막눌렸나.\n    누름 보여주기.\n    순간 보여주기.\n}\n";
        let tape = InputTape { version: 2, madi_hz: 60, records: vec![
            InputRecord { madi: 0, held_mask: mask_to_bytes(0), events: vec![
                InputEvent { sequence: 0, key_bit: 2, kind: InputEventKind::Press },
                InputEvent { sequence: 1, key_bit: 2, kind: InputEventKind::Release },
            ] },
            InputRecord { madi: 1, held_mask: mask_to_bytes(0), events: Vec::new() },
        ] };
        let live = run_summary_from_source_with_live_input(source, "transient.ddn", Some(2), 0, Some(P1A_V1_CORE_PROFILE), 60,
            |madi| Ok(tape.records[madi as usize].clone())).unwrap();
        let replay = run_summary_from_supported_source_profile_with_input_tape(source, "transient.ddn", Some(2), Some(P1A_V1_CORE_PROFILE), &tape).unwrap();
        assert_eq!(live["output_log_texts"], json!(["거짓", "참", "거짓", "거짓"]));
        assert_eq!(live, replay);
    }

    #[test]
    fn supported_profile_run_replays_shared_input_tape_without_private_cli_runtime() {
        let source = "매마디:움직씨 := {\n    1 보여주기.\n}\n";
        let tape = InputTape {
            version: 1,
            madi_hz: 60,
            records: vec![
                crate::input_tape::InputRecord {
                    madi: 0,
                    held_mask: crate::input_tape::mask_to_bytes(0),
                    events: Vec::new(),
                },
                crate::input_tape::InputRecord {
                    madi: 1,
                    held_mask: crate::input_tape::mask_to_bytes(1 << 1),
                    events: Vec::new(),
                },
                crate::input_tape::InputRecord {
                    madi: 2,
                    held_mask: crate::input_tape::mask_to_bytes(1 << 1),
                    events: Vec::new(),
                },
            ],
        };

        let first = run_summary_from_supported_source_profile_with_input_tape(
            source,
            "supported-input-tape.ddn",
            Some(3),
            Some(P1A_V1_CORE_PROFILE),
            &tape,
        )
        .expect("supported V25 input-tape run");
        let repeated = run_summary_from_supported_source_profile_with_input_tape(
            source,
            "supported-input-tape.ddn",
            Some(3),
            Some(P1A_V1_CORE_PROFILE),
            &tape,
        )
        .expect("deterministic repeated input-tape run");

        assert_eq!(first, repeated);
        assert_eq!(first["ticks_run"], 3);
        assert_eq!(first["output_log_texts"], json!(["1", "1", "1"]));
        assert_eq!(first["source_profile_identity"], P1A_V1_CORE_PROFILE);
        let frames = first["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"]
            .as_array()
            .expect("committed observation frames");
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0]["input_snapshot"]["keys_pressed"], 0);
        assert_eq!(frames[1]["input_snapshot"]["keys_pressed"], KEY_D);
        assert_eq!(frames[2]["input_snapshot"]["keys_pressed"], KEY_D);
        assert!(frames[0]["run_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("run:")));

        let mut changed_tape = tape.clone();
        changed_tape.records[1].held_mask = crate::input_tape::mask_to_bytes(0);
        let changed = run_summary_from_supported_source_profile_with_input_tape(
            source,
            "supported-input-tape.ddn",
            Some(3),
            Some(P1A_V1_CORE_PROFILE),
            &changed_tape,
        )
        .expect("changed input-tape run");
        assert_ne!(
            frames[0]["run_id"],
            changed["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"][0]
                ["run_id"]
        );
    }

    #[test]
    fn shared_input_tape_fails_closed_before_partial_execution() {
        let source = "매마디:움직씨 := {\n    1 보여주기.\n}\n";
        let tape = InputTape {
            version: 1,
            madi_hz: 60,
            records: vec![crate::input_tape::InputRecord {
                madi: 1,
                held_mask: crate::input_tape::mask_to_bytes(0),
                events: Vec::new(),
            }],
        };

        let error = run_summary_from_supported_source_profile_with_input_tape(
            source,
            "invalid-input-tape.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
            &tape,
        )
        .expect_err("out-of-order tape must fail closed");
        let diagnostic: JsonValue = serde_json::from_str(&error).expect("structured diagnostic");
        assert_eq!(diagnostic["code"], "E_SAM_RECORD_ORDER_MISMATCH");
        assert!(diagnostic.get("summary").is_none());
    }

    #[test]
    fn c1_review_source_diagnostics_preserve_code_and_location() {
        for (source, code) in [
            ("\u{feff}값 <- 1.", "E_SOURCE_UTF8_BOM_FORBIDDEN"),
            ("만약 참 이라면 { 1 보여주기. }.", "E_V25_IF_SURFACE"),
        ] {
            let error = run_summary_from_supported_source_profile_diagnostic(
                source, "review.ddn", Some(1), Some(P1A_V1_CORE_PROFILE),
            ).expect_err("invalid source must be rejected without masking its code");
            assert_eq!(error.code, code);
            assert!(error.message.contains(code));
        }
        let source = include_str!("../../../solutions/seamgrim_ui_mvp/alpha1/잘못된입력/03_함수표_인자누락.ddn");
        let error = run_summary_from_supported_source_profile_diagnostic(
            source, "missing-pin.ddn", Some(1), Some(P1A_V1_CORE_PROFILE),
        ).expect_err("missing pin must preserve its original location");
        assert_eq!(error.code, "E_CALL_REQUIRED_PIN_MISSING");
        assert!(error.message.contains("(4:1)"), "{}", error.message);
        assert!(error.span.is_some());
    }

    #[test]
    fn p1a_supported_profile_type_mismatch_preserves_shared_identity() {
        let source = "매마디:움직씨 := {\n    값:수 := \"글\".\n}\n";
        let error = run_summary_from_supported_source_profile_diagnostic(
            source,
            "p1a-type-mismatch.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect_err("type mismatch must fail closed");
        assert_eq!(error.code, "E_RUNTIME_TYPE_MISMATCH");
        assert_eq!(error.offending_field.as_deref(), Some("값"));
        assert_eq!(error.expected.as_deref(), Some("셈수"));
        assert_eq!(error.actual.as_deref(), Some("글"));
    }

    #[test]
    fn supported_profile_canon_is_explicit_and_roundtrips_without_default_inference() {
        let source = "매마디:움직씨 := {\n    글:글 := \"첫\\줄둘\\따옴인용\\따옴\".\n}\n";
        let first =
            canonicalize_shared_source_with_supported_profile(source, Some(P1A_V1_CORE_PROFILE))
                .expect("supported V25 canon");
        let second = canonicalize_shared_source_with_supported_profile(
            &first.ddn,
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect("supported V25 canon reparse");
        assert_eq!(first.ddn, second.ddn);
        assert!(first.ddn.contains("\\n"));
        assert!(first.ddn.contains("\\\"인용\\\""));

        // D7 supersedes token-driven current-default selection. A legacy
        // internal canon path must not silently reinterpret V25 definitions.
        assert!(canonicalize_shared_source(source).is_err());
        let omitted = match canonicalize_shared_source_with_supported_profile(source, None) {
            Ok(_) => panic!("profile is required"),
            Err(error) => error,
        };
        assert!(omitted.starts_with("E_SOURCE_PROFILE_REQUIRED:"));
        let unknown = match canonicalize_shared_source_with_supported_profile(
            source,
            Some("unknown-profile"),
        ) {
            Ok(_) => panic!("unknown profile must fail closed"),
            Err(error) => error,
        };
        assert!(unknown.starts_with("E_SOURCE_PROFILE_UNSUPPORTED:"));
    }

    #[test]
    fn p1a_supported_profile_run_fails_closed_for_profile_and_v25_boundaries() {
        let positive = "매마디:움직씨 := {\n    값:수 := 1.\n}\n";
        let omitted = run_summary_from_supported_source_profile_diagnostic(
            positive,
            "p1a-omitted.ddn",
            Some(1),
            None,
        )
        .expect_err("omitted profile");
        assert_eq!(omitted.code, "E_SOURCE_PROFILE_REQUIRED");

        let unknown = run_summary_from_supported_source_profile_diagnostic(
            positive,
            "p1a-unknown.ddn",
            Some(1),
            Some("unknown-profile"),
        )
        .expect_err("unknown profile");
        assert_eq!(unknown.code, "E_SOURCE_PROFILE_UNSUPPORTED");

        let legacy = run_summary_from_supported_source_profile_diagnostic(
            "매마디:움직씨 = {\n}\n",
            "p1a-legacy.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect_err("legacy definition");
        assert_eq!(legacy.code, "E_V25_DEFINITION_MIGRATION_REQUIRED");

        let recursive = "매마디:움직씨 := {\n    () 매마디하기.\n}\n";
        let first = run_summary_from_supported_source_profile_diagnostic(
            recursive,
            "p1a-recursive.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect_err("direct recursion");
        let repeated = run_summary_from_supported_source_profile_diagnostic(
            recursive,
            "p1a-recursive.ddn",
            Some(1),
            Some(P1A_V1_CORE_PROFILE),
        )
        .expect_err("deterministic direct recursion");
        assert_eq!(first, repeated);
        assert_eq!(first.code, "E_V1_CORE_RECURSION_UNSUPPORTED");
    }

    #[test]
    fn migration_evidence_is_explicit_tagged_and_non_executable() {
        let source = "검사:움직씨 = { 값 ~~> 대상. }";
        let artifact = historical_async_migration_evidence_json(source, "history/input.ddn")
            .expect("migration artifact");
        let value: JsonValue = serde_json::from_str(&artifact).expect("artifact json");
        assert_eq!(value["parser_mode"], "HistoricalMigrationEvidence");
        assert_eq!(value["product_execution_authorized"], false);
        assert_eq!(
            value["artifacts"][0]["historical_identity"],
            "MessageTransfer::HistoricalAsync"
        );
        assert_eq!(
            value["artifacts"][0]["successor_replacement"],
            "값 ~> 대상."
        );
        assert!(value["artifacts"][0]["source_span"]["start"].is_u64());
        assert!(value["artifacts"][0]["source_span"]["end"].is_u64());
        let err = match DdnProgram::from_source_with_mode(
            source,
            "history/input.ddn",
            ParseMode::HistoricalMigrationEvidence,
        ) {
            Ok(_) => panic!("migration mode must not execute"),
            Err(err) => err,
        };
        assert!(err.contains("E_HISTORICAL_ASYNC_MIGRATION_EXECUTION_FORBIDDEN"));
    }

    fn envelope_mode(source: &str) -> String {
        execution_envelope_from_source(source, Some(1))
            .unwrap_or_else(|err| panic!("execution envelope for {source:?}: {err}"))
            .get("execution_mode_decision")
            .and_then(|decision| decision.get("mode"))
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn execution_envelope_carries_shared_parsed_mode_decision() {
        assert_eq!(envelope_mode("1 보여주기."), "oneshot");
        assert_eq!(envelope_mode("(시작)할때 { 1 보여주기. }."), "oneshot");
        assert_eq!(envelope_mode("(매마디)마다 { 1 보여주기. }."), "live");
        assert_eq!(envelope_mode("(4마디)마다 { 1 보여주기. }."), "live");
        let envelope =
            execution_envelope_from_source("1 보여주기.", Some(1)).expect("versioned envelope");
        assert_eq!(
            envelope["resource_snapshot_schema"],
            RESOURCE_SNAPSHOT_SCHEMA_V2
        );
        assert_eq!(envelope["unit_wire_schema"], UNIT_VALUE_WIRE_SCHEMA_V2);
    }

    #[test]
    fn shared_cli_runtime_resources_emit_the_same_unit_wire_v2() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "물질량@mol".to_string(),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2),
                ddonirang_core::Unit::Mole,
            )),
        );
        let resources = serialize_world_resources(world.world());
        assert_eq!(resources["schema"], RESOURCE_SNAPSHOT_SCHEMA_V2);
        assert_eq!(resources["unit_wire_schema"], UNIT_VALUE_WIRE_SCHEMA_V2);
        assert_eq!(
            resources["value_det"]["물질량@mol"]["dimension"]["exponents"],
            json!([0, 0, 0, 0, 0, 0, 0, 0, 1])
        );
    }

    #[test]
    fn public_observation_trace_keeps_ddn_order_source_refs_and_committed_rows() {
        let source = r#"
채비 {
  시간 <- 0.
  각도 <- 15.
  각속도 <- 2.
}.

(매마디)마다 {
  시간 <- 시간 + 1.
  각도 <- 각도 + 각속도.
  (시간: 시간 * 1@s, 각도: 각도 * 1@rad, 각속도: 각속도) 내보임.
}.
"#;
        let summary = run_summary_from_source(source, "진자.ddn", Some(2)).expect("run summary");
        let trace = &summary["public_observation_trace"];
        assert_eq!(
            trace["columns"]
                .as_array()
                .expect("columns")
                .iter()
                .map(|column| column["identity"]["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec!["시간", "각도", "각속도"]
        );
        assert_eq!(
            trace["samples"]
                .as_array()
                .expect("samples")
                .iter()
                .map(|sample| sample["committed_madi"].as_u64().unwrap_or(u64::MAX))
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(trace["samples"][1]["cells"][0]["value_text"], json!("2@s"));
        assert_eq!(
            trace["samples"][1]["cells"][1]["value_text"],
            json!("19@rad")
        );
        for column in trace["columns"].as_array().expect("columns") {
            let source_ref = column["identity"]["source_ref"]
                .as_str()
                .expect("source ref");
            assert!(source_ref.starts_with("진자.ddn#bytes="));
        }
        assert_eq!(summary["output_rows"], trace["samples"]);
        assert!(summary["engine_state"]
            .get("observation_manifest")
            .is_none());
        assert!(!trace.to_string().contains("project.obs."));
        assert!(!trace.to_string().contains("output_"));
        let receipt = &summary["work_cost_receipt"];
        assert_eq!(receipt["schema"], "ddonirang.work_cost.v1-candidate");
        assert_eq!(receipt["counters"]["committed_madi"], 2);
        assert_eq!(receipt["counters"]["public_observation_samples"], 6);
        assert_eq!(receipt["counters"]["sealed_input_bytes"], source.len());
        assert_eq!(receipt["counters"]["replay_capsule_bytes"], 0);
        assert_eq!(receipt["slice_limits_enforced"], false);
        assert_eq!(receipt["continuation_present"], false);
        assert_eq!(receipt["rd206_resource_capability"], false);
        assert_eq!(
            receipt["counters"]["result_artifact_bytes"],
            serde_json::to_vec(&summary).expect("summary bytes").len()
        );
        assert!(receipt["counters"]["work_cost"]
            .as_u64()
            .is_some_and(|value| value > 0));
    }

    #[test]
    fn host_preflight_fault_is_distinct_and_returns_no_partial_summary() {
        struct ResetHostPreflightFault;
        impl Drop for ResetHostPreflightFault {
            fn drop(&mut self) {
                set_force_host_preflight_rejection(false);
            }
        }

        let source = "매틱:움직씨 = { (값: 1) 내보임. }\n";
        set_force_host_preflight_rejection(true);
        let _reset = ResetHostPreflightFault;
        let error = run_summary_from_source_diagnostic(source, "host_preflight.ddn", Some(1))
            .expect_err("fault injection must reject before execution");
        assert_eq!(
            error.code,
            crate::work_cost::E_STANDARD_RESOURCE_PREFLIGHT_REJECTED_CANDIDATE
        );
        assert_eq!(error.phase, "resource_preflight");
        assert!(error
            .message
            .contains("observation_frames.try_reserve_exact:fault"));
        assert!(error.message.contains("work_cost_receipt="));
    }

    #[test]
    fn result_artifact_boundary_plus_one_returns_no_partial_summary() {
        let source = "매틱:움직씨 = { (값: 1) 내보임. }\n";
        let limits = StandardResourceLimitsV1 {
            result_artifact_bytes: 1,
            ..StandardResourceLimitsV1::CANDIDATE
        };
        let error = run_summary_from_source_with_assets_diagnostic(
            source,
            "result_artifact_limit.ddn",
            Some(1),
            None,
            Some(limits),
            None,
            None,
            None,
            0,
        )
        .expect_err("one-byte result limit must reject the whole summary");
        assert_eq!(
            error.code,
            crate::work_cost::E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE
        );
        assert_eq!(error.phase, "resource_commit");
        assert!(error.message.contains("result_artifact_bytes"));
        assert!(error.message.contains("work_cost_receipt="));
    }

    #[test]
    fn bounded_model_run_reaches_shared_result_table_surface() {
        let source = r#"
정밀한방법 <- (방식: #정밀).
계산 = 모형 {
  입력:나눔수@m :: (들임, 내줌).
  결과:나눔수@m :: (그자리, 내줌).
  결과::구함.
  입력 + 1@m =:= 결과.
}.
실행결과 <- (대상=계산, 들임=(입력: 2@m), 방법=정밀한방법, 내줌=(결과, 입력)) 풀기.
(답: 실행결과.결과, 원본: 실행결과.입력) 내보임.
"#;
        let summary = run_summary_from_source(source, "app/model.ddn", Some(1))
            .expect("bounded model summary");
        assert_eq!(
            summary["runtime_schema"],
            crate::model_runtime::MODEL_EXECUTION_SCHEMA
        );
        assert_eq!(summary["model_execution"]["status"], "completed");
        assert_eq!(summary["final_row"]["결과"], "3@m");
        assert_eq!(summary["final_row"]["입력"], "2@m");
        assert_eq!(
            summary["public_observation_trace"]["columns"][0]["identity"]["name"],
            "답"
        );
        assert!(
            summary["public_observation_trace"]["columns"][0]["identity"]["source_ref"]
                .as_str()
                .is_some_and(|value| value.starts_with("app/model.ddn#bytes="))
        );
        assert_eq!(
            summary["public_observation_table_projection"]["presentation_state"],
            "ready"
        );
        assert_eq!(summary["ticks_run"], 1);
    }

    #[test]
    fn bounded_model_prepared_state_is_rust_owned_and_empty_until_commit() {
        let source = r#"
정밀한방법 <- (방식: #정밀).
계산 = 모형 {
  입력:나눔수@m :: (들임, 내줌).
  결과:나눔수@m :: (그자리, 내줌).
  결과::구함.
  입력 + 1@m =:= 결과.
}.
실행결과 <- (대상=계산, 들임=(입력: 2@m), 방법=정밀한방법, 내줌=(결과, 입력)) 풀기.
(답: 실행결과.결과, 원본: 실행결과.입력) 내보임.
"#;
        let prepared = model_prepared_state_from_source(source, "app/model.ddn")
            .expect("bounded model prepared state");
        let completed = run_summary_from_source(source, "app/model.ddn", Some(1))
            .expect("bounded model completed state");
        assert_eq!(prepared["schema"], ENGINE_RESPONSE_SCHEMA);
        assert_eq!(prepared["tick_id"], 0);
        assert_eq!(
            prepared["state"]["snapshot_v2"]["schema"],
            "ddn.wasm_state_snapshot.v2"
        );
        assert_eq!(
            prepared["state"]["snapshot_v2"]["resources"]["schema"],
            crate::unit_wire::RESOURCE_SNAPSHOT_SCHEMA_V2
        );
        assert_eq!(completed["engine_state"]["schema"], ENGINE_RESPONSE_SCHEMA);
        assert_eq!(
            completed["engine_state"]["state"]["snapshot_v2"]["next_tick_id"],
            1
        );
        assert_eq!(
            completed["engine_state"]["state"]["snapshot_v2"]["public_observation_frames"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            prepared["state_hash"],
            completed["model_execution"]["pre_state_hash"]
        );
        assert_eq!(
            prepared["public_observation_trace"]["samples"]
                .as_array()
                .map(Vec::len),
            Some(0)
        );
        assert_eq!(
            prepared["model_execution_envelope"]["execution_mode_decision"]["mode"],
            "oneshot"
        );
        assert_eq!(
            prepared["model_execution_envelope"]["backend_descriptor"]["backend_id"],
            "ddn.exact_acyclic_relation.v1"
        );
    }

    #[test]
    fn bounded_model_summary_rejects_non_oneshot_tick_request() {
        let error = run_summary_from_source("값 = 모형 { }.", "model.ddn", Some(2))
            .expect_err("model backend is one-shot");
        assert!(error.contains("E_MODEL_ONESHOT_TICKS_CANDIDATE"));
    }

    #[test]
    fn execution_contract_diagnostic_remains_structured_through_run_summary() {
        let source = crate::self_contained_test_sources::RD206_INNER_EQUALS;
        let error = run_summary_from_source_diagnostic(source, "project/entry.ddn", Some(1))
            .expect_err("inner equals must fail before execution");
        assert_eq!(
            error.code,
            "E_MODEL_EXECUTION_CONTRACT_FIELD_DELIMITER_CANDIDATE"
        );
        assert_eq!(error.source_path, "project/entry.ddn");
        assert!(error.span.is_some_and(|(start, end)| start < end));
        assert_eq!(error.phase, "model_frontdoor_candidate");
        assert!(error.cause.is_some());
        assert_eq!(error.offending_field.as_deref(), Some("결과"));
        assert_eq!(error.expected.as_deref(), Some(":"));
        assert!(error
            .actual
            .as_deref()
            .is_some_and(|value| value.contains('=')));
    }
