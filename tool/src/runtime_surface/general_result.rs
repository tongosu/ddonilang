//! Actual ordinary zero-start and typed output owners. No old-save backfill,
//! normal-end proof, JSON reconstruction or Native admission is granted here.
use super::*;
use crate::replay_evidence::{ReplaySeedV1,ResultEmissionSinkV1,PreparedOutputMadiV1};
use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision;
use crate::unit_wire::resumable_complete_state as complete;
pub(super) struct GeneralExecutionCollector<'a>{
    pub(super) destination:&'a mut Option<GeneralCompletedExecution>,
    pub(super) source:Option<GeneralEvidenceSourceV1<'a>>,
}
#[derive(Clone,Copy)]
pub(super) enum GeneralEvidenceSourceV1<'a>{
    Original(&'a PreparedSourceOnlyRevision),
    #[cfg(windows)]
    Retained(&'a super::native_source_world::general_recording::GeneralOriginalSeedAdmissionV1),
}
pub(super) struct GeneralResultCollectorV1{
    seed:ReplaySeedV1,outputs:ResultEmissionSinkV1,base:u64,age:AgeTarget,
    receipts:Vec<Vec<u8>>,receipt_bytes:usize,
}
pub(super) struct PreparedGeneralOutputMadiV1{output:PreparedOutputMadiV1,receipt:Vec<u8>,receipt_bytes:usize}
pub(crate) struct InitialGeneralReplaySeedBoundaryV1<'a>{
    pub(super) owned:&'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    pub(super) runner:&'a DdnRunner,pub(super) source:&'a PreparedSourceOnlyRevision,pub(super) base:u64,pub(super) age:AgeTarget,
}
impl InitialGeneralReplaySeedBoundaryV1<'_>{
    pub(crate) fn actual_payloads(self)->Result<[Vec<u8>;17],String>{
        let fail=|cause|format!("E_GENERAL_REPLAY_SEED: {cause:?}");
        // The common supplier checks actual zero work counters and entry,
        // rather than accepting a caller-provided 'fresh' success flag.
        let boundary=complete::encode_general_boundary_supplies(self.owned,self.runner,&[],self.base,0).map_err(fail)?;
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(self.owned)?;
        let payload=crate::unit_wire::project_runtime_attachment::saved_runtime_project_payload_json(self.owned)?;
        if self.source.pesi()!=issued.pesi().as_raw()||self.source.payload()!=payload.as_bytes(){
            return Err("E_GENERAL_REPLAY_SEED_SOURCE: 원 시작과 원문 봉인이 다릅니다".into());
        }
        let initial=complete::encode_general_original_initial_world(self.owned,self.runner).map_err(fail)?;
        let frame=crate::unit_wire::check_complete_state_frame_v1(&initial,issued.pesi().as_raw()).map_err(fail)?;
        let mut fields=Vec::with_capacity(17);fields.extend(frame.replay_seed_world_payloads().map_err(fail)?);
        fields.push(self.source.envelope().to_vec());
        fields.push(complete::encode_source_capture_supply(self.owned.sources()).map_err(fail)?);
        fields.push(complete::encode_constituent_supply(self.owned).map_err(fail)?);
        fields.push(complete::encode_execution_contract_supply(self.owned,self.runner).map_err(fail)?);
        fields.push(complete::encode_general_processed_inputs(&[],0,self.base).map_err(fail)?);
        fields.extend(boundary);
        fields.push(complete::encode_general_fresh_source_descriptor(self.owned,self.runner,self.age,self.base,0).map_err(fail)?);
        fields.try_into().map_err(|_|"E_GENERAL_REPLAY_SEED_FIELDS: 원 시작 필드 수가 다릅니다".into())
    }
}
impl GeneralResultCollectorV1{
    pub(super) fn from_source(owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
        runner:&DdnRunner,source:GeneralEvidenceSourceV1<'_>,base:u64,age:AgeTarget)->Result<Self,String>{
        match source{
            GeneralEvidenceSourceV1::Original(source)=>Self::begin(owned,runner,source,base,age),
            #[cfg(windows)]
            GeneralEvidenceSourceV1::Retained(proof)=>{
                let seed=proof.checked_seed_at_actual_zero(owned,runner,base,age)?;
                Ok(Self{seed,outputs:ResultEmissionSinkV1::default(),base,age,receipts:Vec::new(),receipt_bytes:8})
            }
        }
    }
    pub(super) fn begin(owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
        runner:&DdnRunner,source:&PreparedSourceOnlyRevision,base:u64,age:AgeTarget)->Result<Self,String>{
        let seed=ReplaySeedV1::capture_general(InitialGeneralReplaySeedBoundaryV1{owned,runner,source,base,age})?;
        Ok(Self{seed,outputs:ResultEmissionSinkV1::default(),base,age,receipts:Vec::new(),receipt_bytes:8})
    }
    pub(super) fn prepare(&mut self,staged:crate::replay_evidence::StagedResultEmissionsV1,madi:u64,receipt:&JsonValue)
        ->Result<PreparedGeneralOutputMadiV1,String>{
        let input=crate::input_receipt::parse_receipt_snapshot(receipt)?;
        if input.tick_id!=madi||self.receipts.len() as u64!=madi{return Err("E_GENERAL_RECEIPT_ORDER".into());}
        let receipt=crate::replay_evidence::canonical_transport_json(receipt)?;
        let receipt_bytes=self.receipt_bytes.checked_add(9).and_then(|n|n.checked_add(receipt.len()))
            .filter(|n|*n<=134_217_728).ok_or("E_REPLAY_RECEIPT_LIMIT")?;
        self.receipts.try_reserve(1).map_err(|_|"E_REPLAY_RECEIPT_LIMIT")?;
        Ok(PreparedGeneralOutputMadiV1{output:self.outputs.prepare_madi(staged,madi)?,receipt,receipt_bytes})
    }
    pub(super) fn commit(&mut self,prepared:PreparedGeneralOutputMadiV1)->Result<(),String>{
        self.outputs.commit_madi(prepared.output)?;self.receipt_bytes=prepared.receipt_bytes;
        self.receipts.push(prepared.receipt);Ok(())
    }
}
pub(crate) struct CommittedGeneralResultBoundaryV1<'a>{
    owned:&'a crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    execution:&'a GeneralCompletedExecution,rows:&'a [complete::GeneralProcessedInputRow],base:u64,age:AgeTarget,
}
impl GeneralSuccessfulInputPrefix{
    pub(crate) fn borrow_general_result_boundary(&self)->Result<CommittedGeneralResultBoundaryV1<'_>,String>{
        if self.execution.result.is_none(){return Err("E_MISSING_RESULT_EVIDENCE: 원 시작·형식 있는 출력을 보존한 일반 실행이 아닙니다".into());}
        Ok(CommittedGeneralResultBoundaryV1{owned:&self.owned,execution:&self.execution,rows:&self.rows,base:self.base_seed,age:self.construction.age})
    }
    pub(crate) fn original_general_replay_seed(&self)->Result<&ReplaySeedV1,String>{
        self.execution.result.as_ref().map(|result|&result.seed)
            .ok_or_else(||"E_MISSING_REPLAY_SEED: 일반 실행의 원 시작을 보존하지 않았습니다".into())
    }
}
#[cfg(not(target_arch="wasm32"))]
impl VerifiedGeneralNativeRun{
    pub(crate) fn original_general_replay_seed(&self)->Result<&ReplaySeedV1,String>{
        self.execution.result.as_ref().map(|value|&value.seed).ok_or_else(||"E_MISSING_REPLAY_SEED".into())
    }
    pub(crate) fn borrow_general_result_boundary(&self)->Result<CommittedGeneralResultBoundaryV1<'_>,String>{
        if self.execution.result.is_none(){return Err("E_MISSING_RESULT_EVIDENCE: 정상 복원의 원 결과 증거가 없습니다".into());}
        Ok(CommittedGeneralResultBoundaryV1{owned:&self.owned,execution:&self.execution,rows:&self.rows,base:self.base,age:self.age})
    }
}
impl CommittedGeneralResultBoundaryV1<'_>{
    pub(crate) fn transport_parts(&self)->Result<(Vec<u8>,Vec<u8>,u64,Vec<u8>),String>{
        let collector=self.execution.result.as_ref().ok_or("E_MISSING_RESULT_EVIDENCE")?;
        let seed=crate::replay_evidence::ReplaySeedCandidateV1::parse(collector.seed.canonical_bytes())?;
        let contract=complete::encode_execution_contract_supply(self.owned,&self.execution.runner)
            .map_err(|cause|format!("E_GENERAL_INPUT_CONTRACT: {cause:?}"))?;
        if seed.payload(12)!=Some(contract.as_slice())||collector.receipts.len()!=self.rows.len(){return Err("E_GENERAL_INPUT_CONTRACT_CHANGED".into());}
        let mut snapshots=Vec::new();snapshots.try_reserve_exact(self.rows.len()).map_err(|_|"E_REPLAY_RECEIPT_LIMIT")?;
        let mut receipts=Vec::new();receipts.try_reserve_exact(collector.receipt_bytes).map_err(|_|"E_REPLAY_RECEIPT_LIMIT")?;
        receipts.extend_from_slice(&(collector.receipts.len() as u64).to_be_bytes());
        for (row,bytes) in self.rows.iter().zip(&collector.receipts){
            // Recheck the actual retained receipt; this decoder never creates
            // a world, a typed output or a new receipt from display JSON.
            let receipt:JsonValue=serde_json::from_slice(bytes).map_err(|_|"E_GENERAL_RECEIPT_INVALID")?;
            let input=crate::input_receipt::parse_receipt_snapshot(&receipt)?;
            if row.edges.is_some()||crate::input_receipt::snapshot_value(&input)!=crate::input_receipt::snapshot_value(&row.input){
                return Err("E_UNSUPPORTED_GENERAL_INPUT_EVIDENCE: 별도 눌림·뗌 정보나 다른 전달값을 생략하지 않습니다".into());
            }
            snapshots.push(crate::input_receipt::snapshot_value(&row.input));
            receipts.push(1);receipts.extend_from_slice(&(bytes.len() as u64).to_be_bytes());receipts.extend_from_slice(bytes);
        }
        let mut input_contract=vec![3];input_contract.extend_from_slice(&contract);
        let mut evidence=vec![2];evidence.extend(crate::replay_evidence::canonical_transport_json(&JsonValue::Array(snapshots))?);
        Ok((input_contract,evidence,self.rows.len() as u64,receipts))
    }
    pub(crate) fn content_parts(self)->Result<(Vec<u8>,Vec<u8>,Vec<u8>,u64),String>{
        let execution=self.execution;
        let collector=execution.result.as_ref().ok_or("E_MISSING_RESULT_EVIDENCE")?;
        if collector.outputs.next_madi()!=execution.next_tick||collector.base!=self.base||collector.age!=self.age{
            return Err("E_GENERAL_RESULT_POSITION: 실제 일반 실행의 출력·난수 위치가 다릅니다".into());
        }
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(self.owned)?;
        let seed=crate::replay_evidence::ReplaySeedCandidateV1::parse(collector.seed.canonical_bytes())?;
        if seed.payload(1)!=Some(issued.pesi().as_raw().as_slice()){
            return Err("E_GENERAL_RESULT_SOURCE: 원 시작과 현재 일반 실행이 다릅니다".into());
        }
        let boundary=complete::encode_general_boundary_supplies(self.owned,&execution.runner,self.rows,self.base,execution.next_tick)
            .map_err(|cause|format!("E_GENERAL_RESULT_RNG: {cause:?}"))?;
        Ok((result_world_canonical_bytes(execution.world.world()),collector.outputs.content_bytes()?,boundary[0].clone(),execution.next_tick))
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn fixture(value:&str)->(crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,String,PreparedSourceOnlyRevision){
        let request=json!({"source_profile":"v1-core-v25","module_name":"","source_snapshot":{
            "root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":format!("채비 {{ 횟수:바른수 := 0. }}. 실행:움직씨 := {{ (매마디)마다 {{ 횟수 <- 횟수 + 1. {value} 보여주기. }}. }}.")}]}});
        let artifact=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(artifact.as_bytes()).unwrap();
        let source=PreparedSourceOnlyRevision::prepare(&owned,&[],b"original-general-source").unwrap();
        (owned,artifact,source)
    }
    fn run(owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,artifact:&str,
        source:Option<&PreparedSourceOnlyRevision>)->Result<GeneralSuccessfulInputPrefix,String>{
        run_general_input_prefix_from_owned_with_subject(owned,artifact,2,7,None,None,crate::ddn_runtime::default_age_target(),None,source)
    }
    #[test]
    fn b3_general_original_zero_seed_and_exact_outputs_are_retained_without_backfill(){
        let (owned,artifact,source)=fixture("1");
        let actual=run(owned.clone(),&artifact,Some(&source)).unwrap();
        let seed=crate::replay_evidence::ReplaySeedCandidateV1::parse(actual.original_general_replay_seed().unwrap().canonical_bytes()).unwrap();
        assert_eq!(seed.payload(4),Some(0u64.to_be_bytes().as_slice()));
        assert_eq!(seed.payload(9),Some(source.envelope()));
        assert_eq!(&seed.payload(14).unwrap()[..3],&[0,1,1],"ordinary original mode01 remains XOR-tick");
        let result=crate::replay_evidence::ResultPreimageV1::capture_general(actual.borrow_general_result_boundary().unwrap()).unwrap();
        assert!(result.canonical_bytes().starts_with(b"DDN-WO23-RESULT-V1\0"));
        let progress=crate::replay_evidence::GeneralProgressEvidenceV1::capture(actual.borrow_general_result_boundary().unwrap()).unwrap();
        assert_eq!(progress.accepted_count(),2);assert_eq!(progress.input_contract()[0],3);assert_eq!(progress.input_evidence()[0],2);
        assert_eq!(progress.result_canonical_bytes(),result.canonical_bytes());
        assert_eq!(u64::from_be_bytes(progress.actual_receipts()[..8].try_into().unwrap()),2);
        let snapshots:JsonValue=serde_json::from_slice(&progress.input_evidence()[1..]).unwrap();
        assert_eq!(snapshots,json!(actual.rows.iter().map(|row|crate::input_receipt::snapshot_value(&row.input)).collect::<Vec<_>>()));
        let outputs=actual.execution.result.as_ref().unwrap().outputs.content_bytes().unwrap();
        assert_eq!(u64::from_be_bytes(outputs[..8].try_into().unwrap()),2);
        assert_eq!(actual.rows[0].input.rng_seed,7);assert_eq!(actual.rows[1].input.rng_seed,6);
        let legacy=run(owned,&artifact,None).unwrap();
        assert_eq!(legacy.summary["state_hash"],actual.summary["state_hash"],"evidence capture must not change calculation");
        assert!(legacy.borrow_general_result_boundary().is_err());assert!(legacy.original_general_replay_seed().is_err());
        let before=actual.summary.pretty_text().unwrap();
        assert!(GeneralResultCollectorV1::begin(&actual.owned,&actual.execution.runner,&source,7,actual.construction.age).is_err());
        assert_eq!(actual.summary.pretty_text().unwrap(),before,"late seed refusal must not change the execution");
    }
    #[test]
    fn b3_general_output_type_comes_from_the_evaluator_not_the_rendered_number(){
        let (owned,artifact,source)=fixture("1");let integer=run(owned,&artifact,Some(&source)).unwrap();
        let (owned,artifact,source)=fixture("1.0");let decimal=run(owned,&artifact,Some(&source)).unwrap();
        assert_ne!(integer.execution.result.as_ref().unwrap().outputs.content_bytes().unwrap(),
            decimal.execution.result.as_ref().unwrap().outputs.content_bytes().unwrap());
    }
    #[test]
    fn b3_general_foreign_source_seed_is_rejected_before_the_world_runs(){
        let (owned,artifact,_)=fixture("1");let (_,_,foreign)=fixture("2");
        assert!(run(owned,&artifact,Some(&foreign)).err().unwrap().contains("E_GENERAL_REPLAY_SEED"));
    }
    #[test]
    fn b3_general_transport_rejects_edge_loss_and_tampered_actual_receipt(){
        let (owned,artifact,source)=fixture("1");let mut actual=run(owned,&artifact,Some(&source)).unwrap();
        actual.rows[0].edges=Some((0,0));
        assert!(actual.borrow_general_result_boundary().unwrap().transport_parts().err().unwrap().contains("E_UNSUPPORTED_GENERAL_INPUT_EVIDENCE"));
        actual.rows[0].edges=None;
        let receipt=&mut actual.execution.result.as_mut().unwrap().receipts[0];
        let marker=b"\"input_snapshot_sha256\":\"";
        let position=receipt.windows(marker.len()).position(|part|part==marker).unwrap()+marker.len();
        receipt[position]=if receipt[position]==b'0'{b'1'}else{b'0'};
        assert!(actual.borrow_general_result_boundary().unwrap().transport_parts().is_err(),"a changed receipt cannot supply accepted inputs");
    }
}
