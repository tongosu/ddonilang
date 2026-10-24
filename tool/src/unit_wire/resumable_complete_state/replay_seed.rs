//! Capture only an original zero-input owner and its admitted source envelope.
//! Old checkpoints never invoke this as a backfill or a new root admission.
use super::*;
use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision;
fn rejected(cause:impl std::fmt::Debug)->String {format!("E_REPLAY_SEED_SUPPLY: {cause:?}")}

impl CheckpointedLogicalRunV1 {
    pub(crate) fn capture_initial_replay_seed(&mut self,source:&PreparedSourceOnlyRevision)->Result<(),String> {
        if self.replay_seed.is_some(){return Err("E_REPLAY_SEED_ALREADY_CAPTURED: 원 시작 정본은 한 번만 생산합니다".into());}
        let seed=crate::replay_evidence::ReplaySeedV1::capture(InitialLogicalReplaySeedBoundaryV1{run:self,source})?;
        self.replay_seed=Some(seed);Ok(())
    }
    pub(crate) fn original_replay_seed(&self)->Result<&crate::replay_evidence::ReplaySeedV1,String> {
        self.replay_seed.as_ref().ok_or_else(||"E_MISSING_REPLAY_SEED: 원 시작 정본을 보존한 기록이 아닙니다".into())
    }
}
impl InitialLogicalReplaySeedBoundaryV1<'_> {
    pub(crate) fn actual_payloads(self)->Result<[Vec<u8>;17],String> {
        let run=self.run;let owned=&run.provenance.source_owner;
        let (binding,frames,failure)=run.session.accepted_prefix_v3();
        if run.provenance.local_restored || !run.provenance.prior_durable_receipts.is_empty()
            || !frames.is_empty() || failure.is_some() || binding!=&run.provenance.initial_binding
            || run.session.resume_boundary_next_tick(owned.sources().entry_declaration().name())?!=0
            || crate::runtime_surface::current_state_hash(run.session.committed_world())!=binding.initial_world {
            return Err("E_REPLAY_SEED_INITIAL_BOUNDARY: 원 입력을 실행하기 전 경계에서만 캡처합니다".into());
        }
        run.session.borrow_result_boundary()?;
        let payload=crate::unit_wire::project_runtime_attachment::saved_runtime_project_payload_json(owned)?;
        if self.source.pesi()!=&run.provenance.pesi || self.source.payload()!=payload.as_bytes() {
            return Err("E_REPLAY_SEED_SOURCE_MISMATCH: 실제 원 실행과 원문 봉인이 다릅니다".into());
        }
        // The source-only owner already validates this RD631 envelope. Use its
        // exact original bytes, not a future world receipt/envelope or file name.
        let world_bytes=run.encode_world_frame().map_err(rejected)?;
        let world=super::super::complete_state_frame::check_complete_state_frame_v1(&world_bytes,&run.provenance.pesi).map_err(rejected)?;
        let mut fields=Vec::with_capacity(17);
        fields.extend(world.replay_seed_world_payloads().map_err(rejected)?);
        fields.push(self.source.envelope().to_vec());
        fields.push(encode_source_capture_supply(owned.sources()).map_err(rejected)?);
        fields.push(encode_constituent_supply(owned).map_err(rejected)?);
        fields.push(run.session.initial_execution_contract_supply(owned)?);
        fields.push(encode_processed_input_supply(&run.session).map_err(rejected)?);
        fields.push(run.logical_rng_supply().map_err(rejected)?);
        fields.push(run.execution_position_supply().map_err(rejected)?);
        fields.push(encode_replayed_work_supply(&run.session).map_err(rejected)?);
        fields.push(run.source_rebuild_descriptor_supply().map_err(rejected)?);
        fields.try_into().map_err(|_|"E_REPLAY_SEED_SUPPLY: 원 시작 필드 개수가 다릅니다".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,crate::logical_input_tape::LogicalInputBinding) {
        fixture_with(1)
    }
    fn fixture_with(increment:i32)->(crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,crate::logical_input_tape::LogicalInputBinding) {
        let request=serde_json::json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":format!("채비 {{ 값:수 := 0. }}. 실행:움직씨 := {{ (매마디)마다 {{ 값 <- 값 + {increment}. 값 보여주기. }}. }}.")}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes()).unwrap();
        let mut binding:crate::logical_input_tape::LogicalInputBinding=serde_json::from_str(
            include_str!("../../../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json")).unwrap();
        binding.source_revision=crate::runtime_surface::sha256_identity(owned.sources().preparation().source.as_bytes());
        (owned,binding)
    }
    #[test]
    fn b3_seed_is_captured_once_before_input_and_retains_actual_owner_fields() {
        use crate::replay_evidence::ReplaySeedCandidateV1;
        let (owned,binding)=fixture();
        let source=PreparedSourceOnlyRevision::prepare(&owned,&[],b"original-source-r1").unwrap();
        let mut run=CheckpointedLogicalRunV1::fresh(&owned,binding.clone(),crate::ddn_runtime::default_age_target()).unwrap();
        let before=run.session().summary();run.capture_initial_replay_seed(&source).unwrap();
        assert_eq!(run.session().summary(),before,"seed capture must not make a hidden madi");
        let bytes=run.original_replay_seed().unwrap().canonical_bytes().to_vec();
        let candidate=ReplaySeedCandidateV1::parse(&bytes).unwrap();
        assert_eq!(candidate.payload(1),Some(source.pesi().as_slice()));
        assert_eq!(candidate.payload(4),Some(0u64.to_be_bytes().as_slice()));
        assert_eq!(candidate.payload(9),Some(source.envelope()));
        assert_eq!(candidate.payload(14),Some(run.logical_rng_supply().unwrap().as_slice()));
        assert!(candidate.payload(17).is_none());assert!(candidate.payload(19).is_none());
        assert!(run.capture_initial_replay_seed(&source).is_err());
        let held=binding.initial_held.clone();
        let mut active=crate::replay_evidence::LogicalRecordingRunV1::begin(run).unwrap();
        active.step(&crate::logical_input_tape::LogicalSnapshot{madi:0,held:held.clone(),events:Vec::new()}).unwrap();
        assert_eq!(active.original_replay_seed().unwrap().canonical_bytes(),bytes);
        let progress=active.progress_evidence().unwrap();assert_eq!(progress.accepted_count(),1);
        assert_eq!(progress.input_evidence().first(),Some(&1));
        let authored=owned.project().save().unwrap();
        assert!(crate::input_tape_v3::replay_authored_logical(&authored,&progress.input_evidence()[1..],&binding)
            .err().unwrap().contains("옵션"));
        active.step(&crate::logical_input_tape::LogicalSnapshot{madi:1,held,events:Vec::new()}).unwrap();
        assert_eq!(active.progress_evidence().unwrap().accepted_count(),2,"progress export must not stop the actual owner");
        let mut damaged=bytes.clone();damaged[b"DDN-WO23-REPLAY-SEED-V1\0".len()+3]=2;
        assert!(ReplaySeedCandidateV1::parse(&damaged).is_err());
        assert!(ReplaySeedCandidateV1::parse(&bytes[..bytes.len()-1]).is_err());
    }
    #[test]
    fn b3_seed_cannot_be_backfilled_after_an_accepted_input_or_from_foreign_source() {
        let (owned,binding)=fixture();
        let source=PreparedSourceOnlyRevision::prepare(&owned,&[],b"original-source-r1").unwrap();
        let (foreign,_)=fixture_with(2);let wrong=PreparedSourceOnlyRevision::prepare(&foreign,&[],b"foreign-source-r1").unwrap();
        let mut run=CheckpointedLogicalRunV1::fresh(&owned,binding.clone(),crate::ddn_runtime::default_age_target()).unwrap();
        assert!(run.capture_initial_replay_seed(&wrong).is_err());assert!(run.original_replay_seed().is_err());
        run.step(&crate::logical_input_tape::LogicalSnapshot{madi:0,held:binding.initial_held.clone(),events:Vec::new()}).unwrap();
        let before=run.session().summary();assert!(run.capture_initial_replay_seed(&source).is_err());
        assert_eq!(run.session().summary(),before);assert!(run.original_replay_seed().is_err());
    }
}
