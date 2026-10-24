use super::*;
use crate::unit_wire::{resumable_complete_state::{CheckpointedLogicalRunV1,ReadOnlyLogicalReplayV1},
    project_runtime_attachment::source_only::PreparedSourceOnlyRevision};
fn fixture()->(crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,crate::logical_input_tape::LogicalInputBinding){
    let request=serde_json::json!({"source_profile":"v1-core-v25","module_name":"",
        "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
            {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
            {"path":"main.ddn","text":"채비 { 값:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }."}]}});
    let raw=crate::authored_project::operate("create",&request.to_string()).unwrap();
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(raw.as_bytes()).unwrap();
    let mut binding:crate::logical_input_tape::LogicalInputBinding=serde_json::from_str(include_str!(
        "../../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json")).unwrap();
    binding.source_revision=crate::runtime_surface::sha256_identity(owned.sources().preparation().source.as_bytes());
    (owned,binding)
}
#[test]
fn b3_readonly_original_seed_replays_progress_without_promoting_incomplete_or_changed_result(){
    let (owned,binding)=fixture();
    let source=PreparedSourceOnlyRevision::prepare(&owned,&[],&[1]).unwrap();
    let mut run=CheckpointedLogicalRunV1::fresh(&owned,binding.clone(),crate::ddn_runtime::default_age_target()).unwrap();
    run.capture_initial_replay_seed(&source).unwrap();
    let mut run=LogicalRecordingRunV1::begin(run).unwrap();
    for madi in 0..3 {run.step(&crate::logical_input_tape::LogicalSnapshot{madi,events:vec![],held:binding.initial_held.clone()}).unwrap();}
    let progress=run.progress_evidence().unwrap();
    let artifact=source.artifact().unwrap();let seed=run.original_replay_seed().unwrap().canonical_bytes();
    let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&3u64.to_be_bytes());
    let control=[0;113];
    let fields:[&[u8];11]=[artifact.as_bytes(),seed,&progress.input_contract,&progress.input_evidence,&range,
        progress.result.canonical_bytes(),progress.result.digest_bytes(),&progress.actual_receipts,&[0],&[0],&control];
    let raw=bundle::encode_planned(fields).unwrap();
    let replay=ReadOnlyLogicalReplayV1::check_running_prefix(&raw,&binding).unwrap();
    assert_eq!(replay.checked_progress_summary(),run.session().summary());
    assert_eq!(replay.result_digest(),progress.result.digest_bytes());
    assert!(crate::input_tape_v3::replay(owned.sources().preparation().source.as_str(),&progress.input_evidence[1..],
        crate::input_tape_v3::ExpectedBinding::Logical(&binding)).is_err(),"progress is still Incomplete in the public replay path");
    let mut changed_result=progress.result.canonical_bytes().to_vec();
    *changed_result.last_mut().unwrap()^=1;
    use sha2::{Digest,Sha256};
    let changed_hash:[u8;32]=Sha256::digest(&changed_result).into();
    let mut changed=fields;changed[5]=&changed_result;changed[6]=&changed_hash;
    let changed=bundle::encode_planned(changed).unwrap();
    assert!(ReadOnlyLogicalReplayV1::check_running_prefix(&changed,&binding).err().unwrap().contains("E_REPLAY_RESULT_MISMATCH"));
    let mut changed_binding=binding.clone();changed_binding.initial_rng^=1;
    assert!(ReadOnlyLogicalReplayV1::check_running_prefix(&raw,&changed_binding).is_err());
}
