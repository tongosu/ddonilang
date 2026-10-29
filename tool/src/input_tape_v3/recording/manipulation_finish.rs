//! The same end issuer and codec, with the actual manipulation owner.
use super::*;
impl RecordingControllerV1 {
    pub(in crate::input_tape_v3) fn for_fresh_manipulation(session:&mut manipulation::ManipulationSession)->Result<Self,String> {
        let (_,frames,failure)=session.accepted_prefix_v3();
        if !frames.is_empty() || failure.is_some(){return Err(error("원래 새 실행에서만 기록 생산자를 결속합니다"));}
        // ManipulationSession has one source-admitted constructor and no
        // saved/reconstruction constructor. Never deserialize this generation.
        Ok(Self{generation:session.recording_generation(),state:RecordingStateV1::Running,frontend:None})
    }
    pub(in crate::input_tape_v3) fn check_manipulation_owner(&self,session:&manipulation::ManipulationSession)->Result<(),String> {
        if !self.generation.matches(&session.recording_generation()) {return Err(error("다른 조작 실행 세대의 기록 생산자입니다"));}
        Ok(())
    }
    pub(in crate::input_tape_v3) fn register_manipulation_frontend(&mut self,session:&mut manipulation::ManipulationSession)
        ->Result<serde_json::Value,String> {
        self.ensure_running()?;self.check_manipulation_owner(session)?;
        if self.frontend.is_some() || session.accepted_prefix_v3().2.is_some(){return Err(error("살아 있는 기록에 실제 입력 생산자를 한 번만 결속합니다"));}
        let count=session.accepted_prefix_v3().1.iter().try_fold(0u64,|n,f|n.checked_add(f.input.events.len() as u64))
            .ok_or_else(||error("사건 합계 초과"))?;
        let owner=PendingSupplyOwnerV1::issue()?;let response=owner.registration(count);
        self.frontend=Some(owner);Ok(response)
    }
    pub(in crate::input_tape_v3) fn prepare_manipulation_finish(&self,session:&mut manipulation::ManipulationSession,packet:Option<&str>)
        ->Result<PreparedDirectFinishV1,String> {
        self.ensure_running()?;self.check_manipulation_owner(session)?;
        session.check_live_recording_end()?;
        let (binding,accepted,failure)=session.accepted_prefix_v3();
        if failure.is_some(){return Err(error("실패한 조작 기록을 정상 마칠 수 없습니다"));}
        let frames=accepted.iter().map(|frame|Frame::from_input(
            Snapshot::from(&frame.input),&frame.accepted_world_digest)).collect::<Result<Vec<_>,_>>()?;
        // Existing active-transaction restrictions are checked by that typed
        // owner's Complete codec. A pending End cannot be executed to pass it.
        self.prepare_accepted_finish(ExpectedBinding::Manipulation(binding),frames,session.summary(),packet)
    }
    pub(in crate::input_tape_v3) fn commit_manipulation_finish(&mut self,session:&mut manipulation::ManipulationSession,
        candidate:PreparedDirectFinishV1)->Result<(),String> {
        let current=self.prepare_manipulation_finish(session,candidate.pending_packet.as_deref())?;
        self.commit_candidate(current,candidate)
    }
}
