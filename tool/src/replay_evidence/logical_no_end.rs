//! Actual stop/failure evidence, never a normal-end issuer. No host enum,
//! decoded tape or successful summary can construct this owned value.
use super::{ExecutionGenerationV1,ResultPreimageV1};
use crate::{input_tape_v3::LiveRecording,logical_input_tape::LogicalInputSession};

pub(crate) struct LogicalNoEndEvidenceV1 {
    pub(super) generation:ExecutionGenerationV1,pub(super) state:u8,
    pub(super) contract:Vec<u8>,pub(super) tape:Vec<u8>,pub(super) summary:serde_json::Value,
    pub(super) accepted_count:u64,pub(super) receipts:Vec<u8>,pub(super) result:ResultPreimageV1,
}
impl LogicalNoEndEvidenceV1 {
    pub(super) fn capture(recording:&mut LiveRecording,session:&LogicalInputSession)->Result<Self,String>{
        recording.check_live_logical_owner(session)?;
        let (binding,frames,failure)=session.accepted_prefix_v3();
        let accepted_count=u64::try_from(frames.len()).map_err(|_|"E_NO_END_COUNT")?;
        if session.committed_input_receipts().len()!=frames.len(){return Err("E_NO_END_RECEIPTS: 확정 입력 영수증 수가 다릅니다".into());}
        let contract=crate::input_tape_v3::logical_contract_bytes(binding)?;
        let receipts=super::actual_receipt_array(session.committed_input_receipts())?;
        let result=ResultPreimageV1::capture(session.borrow_result_boundary()?)?;
        let generation=session.recording_generation();let state=if failure.is_some(){3}else{2};
        let (tape,summary)=recording.stop_logical(session)?;
        Ok(Self{generation,state,contract,tape,summary,accepted_count,receipts,result})
    }
    pub(crate) fn check_session(&self,session:&LogicalInputSession)->Result<(),String>{
        let (_,frames,failure)=session.accepted_prefix_v3();
        if !self.generation.matches(&session.recording_generation()) || frames.len() as u64!=self.accepted_count
            || self.state!=(if failure.is_some(){3}else{2})
            || ResultPreimageV1::capture(session.borrow_result_boundary()?)?.canonical_bytes()!=self.result.canonical_bytes(){
            return Err("E_NO_END_OWNER: 실제 중단·실패 생산자가 다릅니다".into());
        }Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logical_input_tape as logical;
    use ddonirang_core::Nuri;
    use sha2::{Digest,Sha256};
    use std::collections::BTreeMap;
    #[test]
    fn b3_no_end_belongs_to_actual_stopped_or_failed_producer_never_complete(){
        let source="(매마디)마다 { 1 보여주기. }.";
        let binding=logical::LogicalInputBinding{
            source_profile:logical::LOGICAL_INPUT_PROFILE.into(),input_schema:"test.input@1".into(),channel_registry:"test.channels@1".into(),
            source_revision:format!("sha256:{}",hex::encode(Sha256::digest(source.as_bytes()))),
            initial_world:crate::runtime_surface::current_state_hash(ddonirang_core::platform::DetNuri::new().world()),
            initial_rng:7,recording_policy:"test.record@1".into(),integrity_policy:"test.seal@1".into(),madi_hz:60,dt_raw:4294967296,
            channels:BTreeMap::from([("왼쪽".into(),logical::ChannelKind::Control)]),initial_held:BTreeMap::from([("왼쪽".into(),false)])};
        for failed in [false,true]{
            let mut session=logical::LogicalInputSession::new(source,binding.clone()).unwrap();
            let mut recording=LiveRecording::for_logical(&mut session).unwrap();
            session.step(&logical::LogicalSnapshot{madi:0,events:vec![],held:binding.initial_held.clone()}).unwrap();
            if failed{session.reject_input("E_LOGICAL_INPUT_LOST: actual test producer loss".into());}
            let before=session.summary();
            let evidence=LogicalNoEndEvidenceV1::capture(&mut recording,&session).unwrap();
            assert_eq!(evidence.state,if failed{3}else{2});assert_eq!(evidence.accepted_count,1);
            assert_eq!(evidence.summary,before);assert_eq!(session.summary(),before);
            evidence.check_session(&session).unwrap();
            assert!(recording.finish_logical(&mut session,None).is_err());
            assert!(crate::input_tape_v3::replay(source,&evidence.tape,
                crate::input_tape_v3::ExpectedBinding::Logical(&binding)).is_err());
            let foreign=logical::LogicalInputSession::new(source,binding.clone()).unwrap();
            assert!(evidence.check_session(&foreign).is_err());
        }
    }
}
