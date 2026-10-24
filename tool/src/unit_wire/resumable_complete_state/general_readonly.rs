//! Original ordinary SEED/source replay. Only checked content survives this
//! constructor: no VM extraction, Deref, live step, save or normal permission.
use super::*;
use crate::replay_evidence::{ReplayBundleCandidateV1,ReplaySeedCandidateV1,ResultPreimageV1,ResultEmissionSinkV1};
use ddonirang_core::{Nuri,platform::DetNuri,signals::VecSignalSink};
pub(crate) struct ReadOnlyGeneralReplayV1{result:ResultPreimageV1,count:u64,pesi:[u8;32]}
pub(crate) struct GeneralReadOnlyBoundaryV1<'a>{
    owned:&'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    runner:&'a crate::ddn_runtime::DdnRunner,world:&'a DetNuri,outputs:&'a ResultEmissionSinkV1,
    rows:&'a [GeneralProcessedInputRow],base:u64,
}
impl GeneralReadOnlyBoundaryV1<'_>{
    pub(crate) fn content_parts(self)->Result<(Vec<u8>,Vec<u8>,Vec<u8>,u64),String>{
        let next=self.rows.len() as u64;
        if self.outputs.next_madi()!=next{return Err("E_GENERAL_REPLAY_OUTPUT_POSITION".into());}
        let boundary=encode_general_boundary_supplies(self.owned,self.runner,self.rows,self.base,next)
            .map_err(|cause|format!("E_GENERAL_REPLAY_RNG: {cause:?}"))?;
        Ok((crate::runtime_surface::result_world_canonical_bytes(self.world.world()),self.outputs.content_bytes()?,boundary[0].clone(),next))
    }
}
fn rejected(cause:impl std::fmt::Debug)->String{format!("E_GENERAL_ORIGINAL_REPLAY: {cause:?}")}
impl ReadOnlyGeneralReplayV1{
    pub(crate) fn checked_result_bytes(&self)->&[u8]{self.result.canonical_bytes()}
    pub(crate) fn accepted_count(&self)->u64{self.count}
    pub(crate) fn source_pesi(&self)->&[u8;32]{&self.pesi}
    pub(crate) fn report_json(&self)->String{
        serde_json::json!({"status":"RUNNING_RESULT_COMPARED","accepted_count_decimal":self.count.to_string(),
            "source_pesi":hex::encode(self.pesi),"result_identity":self.result.digest_label()}).to_string()
    }
    /// Running data comparison only. This never issues ReplayVerified/End.
    pub(crate) fn check_running(bytes:&[u8])->Result<Self,String>{
        let bundle=ReplayBundleCandidateV1::parse(bytes)?;
        if bundle.payload(11).and_then(|value|value.first())!=Some(&0){return Err("E_GENERAL_REPLAY_STATE: 진행 중인 기록만 비교합니다".into());}
        let source_raw=std::str::from_utf8(bundle.payload(1).ok_or("E_GENERAL_REPLAY_SOURCE")?).map_err(rejected)?;
        let source=super::super::project_runtime_attachment::source_only::PreparedSourceOnlyRevision::from_artifact(source_raw)?;
        let owned=source.recorded_execution_owner()?;
        let seed=ReplaySeedCandidateV1::parse(bundle.payload(2).ok_or("E_GENERAL_REPLAY_SEED")?)?;
        let field=|tag|seed.payload(tag).ok_or_else(||"E_GENERAL_REPLAY_SEED_FIELD".to_owned());
        if field(1)?!=source.pesi()||field(9)?!=source.envelope()||field(10)?!=encode_source_capture_supply(owned.sources()).map_err(rejected)?{
            return Err("E_GENERAL_REPLAY_SOURCE: 원문과 원 시작 봉인이 다릅니다".into());
        }
        check_constituent_supply(field(11)?,source.pesi(),&owned).map_err(rejected)?;
        let candidate=rebuild_source_descriptor_candidate(field(18)?,&owned).map_err(rejected)?;
        if candidate.initial_tick!=0||candidate.phase!=0||candidate.successful_madi_count!=0||!candidate.prior_durable_receipts.is_empty(){
            return Err("E_GENERAL_REPLAY_SEED: 최초 경계가 아닙니다".into());
        }
        for (index,original) in candidate.initial_world.replay_seed_world_payloads().map_err(rejected)?.iter().enumerate(){
            if field(index as u8+1)?!=original{return Err("E_GENERAL_REPLAY_INITIAL_WORLD".into());}
        }
        let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(&owned)).transpose()?;
        let initial=candidate.initial_world.decode_value_tables_with_catalog(catalog.as_ref()).map_err(rejected)?;
        if !initial.resources.is_empty()||!initial.parameters.is_empty()||!initial.subjects.is_empty()
            ||initial.subject_custody.as_ref().is_some_and(|custody|!custody.rows.is_empty()){
            return Err("E_UNSUPPORTED_GENERAL_INITIAL_STATE: 초기 값을 임의 재생성하지 않습니다".into());
        }
        let base=candidate.base_rng_seed;
        let mut runner=candidate.runner.with_bogae_activation();
        if owned.project().schema==crate::authored_project::SCHEMA_V2||owned.project().authored_identity.modules.iter().any(|module|!module.compositions.is_empty()){
            runner.configure_authored_bogae(&owned.project().save()?)?;
        }
        restore_general_original_work(field(16)?,&mut runner).map_err(rejected)?;
        let contract=encode_execution_contract_supply(&owned,&runner).map_err(rejected)?;
        let zero=encode_general_boundary_supplies(&owned,&runner,&[],base,0).map_err(rejected)?;
        if field(12)?!=contract||field(13)?!=encode_general_processed_inputs(&[],0,base).map_err(rejected)?
            ||field(14)?!=zero[0]||field(15)?!=zero[1]||field(16)?!=zero[2]{return Err("E_GENERAL_REPLAY_INITIAL_CONTRACT".into());}
        let mut input_contract=vec![3];input_contract.extend_from_slice(&contract);
        if bundle.payload(3)!=Some(input_contract.as_slice()){return Err("E_GENERAL_REPLAY_INPUT_CONTRACT".into());}
        let evidence=bundle.payload(4).ok_or("E_GENERAL_REPLAY_INPUTS")?;
        if evidence.first()!=Some(&2){return Err("E_GENERAL_REPLAY_INPUT_KIND".into());}
        let snapshots:serde_json::Value=serde_json::from_slice(&evidence[1..]).map_err(rejected)?;
        if crate::replay_evidence::canonical_transport_json(&snapshots)?!=evidence[1..]{return Err("E_GENERAL_REPLAY_NONCANONICAL_INPUTS".into());}
        let snapshots=snapshots.as_array().ok_or("E_GENERAL_REPLAY_INPUTS")?;
        let expected_receipts=crate::replay_evidence::decode_receipt_array(bundle.payload(8).ok_or("E_GENERAL_REPLAY_RECEIPTS")?,runner.work_cost_receipt().limits.slice_committed_madi)?;
        if snapshots.len()!=expected_receipts.len(){return Err("E_GENERAL_REPLAY_INPUT_COUNT".into());}
        let count=expected_receipts.len() as u64;
        let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&count.to_be_bytes());
        if bundle.payload(5)!=Some(range.as_slice()){return Err("E_GENERAL_REPLAY_RANGE".into());}
        let mut world=DetNuri::new();let mut defaults=std::collections::HashMap::new();
        crate::runtime_surface::seed_bogae_defaults(&mut defaults);
        let mut outputs=ResultEmissionSinkV1::default();let mut rows=Vec::new();let mut receipts=Vec::new();
        rows.try_reserve_exact(expected_receipts.len()).map_err(|_|"E_GENERAL_REPLAY_LIMIT")?;
        receipts.try_reserve_exact(expected_receipts.len()).map_err(|_|"E_GENERAL_REPLAY_LIMIT")?;
        for (index,(snapshot,receipt)) in snapshots.iter().zip(&expected_receipts).enumerate(){
            let input=crate::input_receipt::parse_receipt_snapshot(receipt)?;
            if crate::input_receipt::snapshot_value(&input)!=*snapshot||input.tick_id!=index as u64||input.rng_seed!=base^input.tick_id{
                return Err("E_GENERAL_REPLAY_INPUT_BINDING".into());
            }
            let output=runner.run_update_diagnostic_with_control_edges(world.world(),&input,&defaults,None,None)
                .map_err(|cause|format!("E_GENERAL_REPLAY_EXECUTION: {cause:?}"))?;
            let prepared=outputs.prepare_madi(output.result_emissions,input.tick_id)?;outputs.require_exact(&prepared)?;
            let actual=crate::input_receipt::receipt(&input,&output.input_reads);
            world.apply_patch(&output.patch,input.tick_id,&mut VecSignalSink::default());outputs.commit_madi(prepared)?;
            receipts.push(actual);rows.push(GeneralProcessedInputRow{input,edges:None});
        }
        let result=ResultPreimageV1::capture_general_readonly(GeneralReadOnlyBoundaryV1{owned:&owned,runner:&runner,world:&world,outputs:&outputs,rows:&rows,base})?;
        if bundle.payload(6)!=Some(result.canonical_bytes())||bundle.payload(7)!=Some(result.digest_bytes().as_slice()){
            return Err("E_GENERAL_REPLAY_RESULT_MISMATCH: 원 입력으로 계산한 정확 결과가 다릅니다".into());
        }
        if bundle.payload(8)!=Some(crate::replay_evidence::actual_receipt_array(&receipts)?.as_slice()){
            return Err("E_GENERAL_REPLAY_RECEIPTS: 실제 읽은 입력이 다릅니다".into());
        }
        // The staging kernel is dropped, not extracted or converted to a Run.
        Ok(Self{result,count,pesi:*source.pesi()})
    }
}
