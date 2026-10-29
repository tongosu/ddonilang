use ddonirang_tool::proof_workflow::verify_assumption_source;
const RAW: &str = "(P:세움) 두고\n(증거=증거:(P)보임~을)\n가정그대로쓰다 : (P)보임 : 셈씨 := { 가정그대로쓰다 <- 증거. }.";
#[test]
fn shared_product_workflow_checks_unmodified_source() {
    assert_eq!(verify_assumption_source(RAW, "가정.ddn").unwrap().len(), 1);
    for raw in [RAW.replace("<- 증거", "<- 참"), RAW.replace("<- 증거", "<- 없음"),
        RAW.replace("P:세움", "P:세움, Q:세움").replace("증거:(P)보임", "증거:(Q)보임")] {
        assert!(verify_assumption_source(&raw, "거부.ddn").is_err());
    }
}

#[test]
fn existing_verify_file_frontdoor_accepts_raw_and_rejects_out_without_writing() {
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let input = std::env::temp_dir().join(format!("ddn-proof-{stamp}.ddn"));
    let output = input.with_extension("out");
    std::fs::write(&input, RAW).unwrap();
    ddonirang_tool::proof_workflow::run_verify(&input, None).unwrap();
    assert!(ddonirang_tool::proof_workflow::run_verify(&input, Some(&output)).is_err());
    assert!(!output.exists());
    std::fs::write(&input, RAW.replace("<- 증거", "<- 빠진증거")).unwrap();
    let error = ddonirang_tool::proof_workflow::run_verify(&input, None).unwrap_err();
    assert!(error.contains("E_PROOF_ASSUMPTION"), "{error}");
    assert!(error.contains(input.to_str().unwrap()));
    std::fs::remove_file(input).unwrap();
}

const V4: &str = include_str!("fixtures/v4_linear_steps/정상.ddn");
#[test]
fn v4_observation_stops_at_first_bad_step_without_fabricating_success() {
    use ddonirang_tool::proof_workflow::observe_algebra_source as observe;
    let normal=observe(V4,"수업.ddn");
    assert_eq!(normal["status"],"accepted");
    assert_eq!(normal["unconditional_proof_issued"],false);
    let steps=normal["definitions"][0]["steps"].as_array().unwrap();
    assert_eq!(steps.len(),4);
    assert!(steps.iter().all(|s|s["status"]=="checked"));
    assert_eq!(steps[2]["reciprocal"]["denominator"],"3");
    let bad=V4.replace("오른=6","오른=5");
    let report=observe(&bad,"수업.ddn");
    assert_eq!(report["status"],"rejected");
    let steps=report["definitions"][0]["steps"].as_array().unwrap();
    assert_eq!(steps.iter().map(|s|s["status"].as_str().unwrap()).collect::<Vec<_>>(),vec!["checked","failed","not_checked","not_checked"]);
    assert!(steps[2]["equality"].is_null());
    assert!(steps[3]["reciprocal"].is_null());
    let a=report["diagnostic"]["span"]["start"].as_u64().unwrap() as usize;
    let b=report["diagnostic"]["span"]["end"].as_u64().unwrap() as usize;
    assert_eq!(&bad[a..b],"5");
    assert_ne!(normal["source_sha256"],report["source_sha256"]);
    let analysis=observe(&V4.replace("근거=정리","근거=없는근거"),"수업.ddn");
    assert_eq!(analysis["phase"],"analysis");
    assert_eq!(analysis["status"],"rejected");
    assert!(analysis["definitions"].as_array().unwrap().is_empty());
    if let Ok(out)=std::env::var("V4_RAW_OUTPUT") {
        std::fs::create_dir_all(&out).unwrap();
        for (name,value) in [("normal",normal),("intermediate",report),("analysis",analysis)] {
            std::fs::write(std::path::Path::new(&out).join(format!("{name}.json")),serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        }
    }
}

#[cfg(windows)]
#[test]
fn v4_native_source_save_then_new_process_reopens_and_explicitly_checks() {
    use ddonirang_tool::{authored_project::{ProjectArtifact,SourceSnapshot,SourceFile,open},runtime_surface};
    use ddonirang_tool::proof_workflow::observe_algebra_source;
    use serde_json::json;
    if let Ok(manifest)=std::env::var("V4_NATIVE_COLD_MANIFEST") {
        let data:serde_json::Value=serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
        let root=std::path::Path::new(data["root"].as_str().unwrap());
        let project:Vec<u8>=serde_json::from_value(data["project"].clone()).unwrap();
        let read=runtime_surface::read_authored_source_native(root,&project,data["locator"].as_str().unwrap()).unwrap();
        assert_eq!(read.project_payload_utf8,data["saved_artifact"].as_str().unwrap());
        let payload:serde_json::Value=serde_json::from_str(&read.project_payload_utf8).unwrap();
        let artifact=open(payload["authored_project_json_utf8"].as_str().unwrap().as_bytes()).unwrap();
        let file=artifact.source_snapshot.files.iter().find(|f|f.path=="수업.ddn").unwrap();
        assert_eq!(file.text,data["source"].as_str().unwrap());
        // No observation or CheckedAlgebraDefinition is restored. Only an explicit
        // fresh source replay below produces the diagnostic/step observation.
        let mut observation:Option<serde_json::Value>=None;
        assert!(observation.is_none());
        observation=Some(observe_algebra_source(&file.text,&file.path));
        let report=observation.unwrap();
        assert_eq!(report["status"],data["expected_status"]);
        assert_eq!(report["unconditional_proof_issued"],false);
        std::fs::write(format!("{manifest}.rechecked.json"),serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        return;
    }
    let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let base=std::env::temp_dir().join(format!("v4-native-{stamp}"));
    std::fs::create_dir_all(&base).unwrap();
    for (name,source,status) in [("normal",V4.to_owned(),"accepted"),("intermediate",V4.replace("오른=6","오른=5"),"rejected")] {
        let root=base.join(name);std::fs::create_dir_all(&root).unwrap();
        let graph="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\"),(이름=수업, 머리=\"수업.ddn\", 원문=\"수업.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.";
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:graph.into()},
            SourceFile{path:"main.ddn".into(),text:"실행:움직씨 := { }.".into()},
            SourceFile{path:"수업.ddn".into(),text:source.clone()}]},String::new()).unwrap().save().unwrap();
        let saved=runtime_surface::save_initial_authored_source_native(&artifact,b"v4-first",&root).unwrap();
        let manifest=base.join(format!("{name}.json"));
        std::fs::write(&manifest,serde_json::to_vec_pretty(&json!({"root":root,"project":saved.project_identity,"locator":saved.grant_locator.unwrap(),"saved_artifact":saved.project_payload_utf8,"source":source,"expected_status":status})).unwrap()).unwrap();
        let result=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","v4_native_source_save_then_new_process_reopens_and_explicitly_checks","--nocapture"])
            .env("V4_NATIVE_COLD_MANIFEST",&manifest).output().unwrap();
        std::fs::write(base.join(format!("{name}.child.stdout")),&result.stdout).unwrap();
        std::fs::write(base.join(format!("{name}.child.stderr")),&result.stderr).unwrap();
        assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
    }
    println!("V4_NATIVE_RAW={}",base.display());
}
