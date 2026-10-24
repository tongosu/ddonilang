//! R1.5 synchronous/direct and frontend normal-end owner. Native durable
//! Pending/Ready publication is separate and is not issued by this module.
use super::*;
use crate::replay_evidence::ExecutionGenerationV1;
use super::pending::{PendingSupplyOwnerV1,CheckedPendingAtFinishV1};
mod manipulation_finish;

/// Shared actual C01 response producer. Recovery serializes this exact value,
/// not host JSON or a reconstructed display summary.
pub(crate) fn finished_output_value(bytes:&[u8],summary:&serde_json::Value)->serde_json::Value {
    serde_json::json!({"tape_bytes":bytes,"summary":summary})
}

#[cfg(test)]
mod recovery_payload_tests {
    use super::*;
    #[test]
    fn wo25_finish_payload_limit_rejects_before_returning_partial_c01() {
        let summary=serde_json::json!({"output_log_texts":["한글 🧪"],"madi":3});
        let expected=finished_output_value(&[0,128,255],&summary).to_string().into_bytes();
        assert_eq!(encode_finish_payload(&[0,128,255],&summary,expected.len()).unwrap(),expected);
        assert!(encode_finish_payload(&[0,128,255],&summary,expected.len()-1).is_err());
        assert!(encode_finish_payload(&[0,128,255],&summary,0).is_err());
        let decoded:serde_json::Value=serde_json::from_slice(&expected).unwrap();
        assert_eq!(decoded.as_object().unwrap().len(),2);
        assert_eq!(decoded["summary"],summary);
    }
}

fn encode_finish_payload(bytes:&[u8],summary:&serde_json::Value,limit:usize)->Result<Vec<u8>,String> {
    struct Bounded {raw:Vec<u8>,limit:usize}
    impl std::io::Write for Bounded {
        fn write(&mut self,bytes:&[u8])->std::io::Result<usize>{
            let length=self.raw.len().checked_add(bytes.len()).filter(|length|*length<=self.limit)
                .ok_or_else(||std::io::Error::new(std::io::ErrorKind::InvalidData,"마치기 응답 크기 한도 초과"))?;
            self.raw.try_reserve(length-self.raw.len())
                .map_err(|_|std::io::Error::new(std::io::ErrorKind::OutOfMemory,"마치기 응답 메모리 부족"))?;
            self.raw.extend_from_slice(bytes);Ok(bytes.len())
        }
        fn flush(&mut self)->std::io::Result<()>{Ok(())}
    }
    let mut writer=Bounded{raw:Vec::new(),limit};
    serde_json::to_writer(&mut writer,&finished_output_value(bytes,summary))
        .map_err(|cause|format!("E_NATIVE_FINISH_PAYLOAD_LIMIT: 원래 마치기 응답을 보존할 수 없습니다 ({cause})"))?;
    Ok(writer.raw)
}

pub(super) struct RecordingControllerV1 {
    generation:ExecutionGenerationV1,
    state:RecordingStateV1,
    frontend:Option<PendingSupplyOwnerV1>,
}
enum RecordingStateV1 {
    Running,
    Finished {bytes:Vec<u8>,summary:serde_json::Value,proof:RecordedEndProofV1,notice:Option<CheckedPendingAtFinishV1>},
    StoppedIncomplete,
    Aborted,
}
struct RecordedEndProofV1 {
    generation:ExecutionGenerationV1,
    accepted_prefix_digest:[u8;32],
    closed_count:u64,
}
/// Prepared is not Finished. Native may only publish it after the exact
/// same-DB Pending/commit/Ready protocol; this constructor grants no save right.
pub(super) struct PreparedDirectFinishV1 {
    bytes:Vec<u8>,summary:serde_json::Value,proof:RecordedEndProofV1,
    pending_packet:Option<String>,notice:Option<CheckedPendingAtFinishV1>,
}
impl PreparedDirectFinishV1 {
    pub(super) fn planned_artifact_parts(&self)->(&[u8],&serde_json::Value){(&self.bytes,&self.summary)}
    #[cfg(windows)]
    pub(super) fn closed_count(&self)->u64{self.proof.closed_count}
    #[cfg(windows)]
    pub(super) fn generation(&self)->ExecutionGenerationV1{self.proof.generation.clone()}
    pub(super) fn recovery_payload_bytes(&self)->Result<Vec<u8>,String>{
        encode_finish_payload(&self.bytes,&self.summary,536_870_912)
    }
    pub(super) fn recovery_notice_payload(&self)->Vec<u8>{
        // None is produced only by prepare_accepted_finish's exclusive
        // (None,None) branch. Missing/stale frontend supply cannot prepare it.
        self.notice.as_ref().map_or_else(||vec![0],CheckedPendingAtFinishV1::recovery_payload)
    }
}
impl RecordingControllerV1 {
    #[cfg(windows)]
    pub(super) fn for_native_running_progress(session:&mut logical::LogicalInputSession,
        proof:&crate::unit_wire::resumable_complete_state::CurrentRecordingProgressProofV1)->Result<Self,String>{
        proof.check_session(session)?;session.borrow_result_boundary()?;
        Ok(Self{generation:session.recording_generation(),state:RecordingStateV1::Running,frontend:None})
    }
    pub(super) fn for_fresh_logical(session:&mut logical::LogicalInputSession)->Result<Self,String> {
        let (_,frames,failure)=session.accepted_prefix_v3();
        if !frames.is_empty() || failure.is_some() {
            return Err(error("원래 새 실행에서만 기록 생산자를 결속합니다"));
        }
        // A reconstructed old checkpoint is not an original seed supplier.
        session.borrow_result_boundary()?;
        Ok(Self{generation:session.recording_generation(),state:RecordingStateV1::Running,frontend:None})
    }
    pub(super) fn ensure_running(&self)->Result<(),String> {
        if matches!(self.state,RecordingStateV1::Running){Ok(())}
        else{Err(error("기록이 종료되어 새 입력을 받지 않습니다"))}
    }
    pub(super) fn check_owner(&self,session:&logical::LogicalInputSession)->Result<(),String> {
        if !self.generation.matches(&session.recording_generation()) {
            return Err(error("다른 실행 세대의 기록 생산자입니다"));
        }
        Ok(())
    }
    pub(super) fn prepare_direct_finish(&self,session:&mut logical::LogicalInputSession)
        ->Result<PreparedDirectFinishV1,String> {
        self.prepare_finish(session,None)
    }
    pub(super) fn register_frontend(&mut self,session:&mut logical::LogicalInputSession)
        ->Result<serde_json::Value,String> {
        self.ensure_running()?;self.check_owner(session)?;
        if self.frontend.is_some() || session.accepted_prefix_v3().2.is_some() {
            return Err(error("살아 있는 기록에 실제 입력 생산자를 한 번만 결속합니다"));
        }
        let accepted_events=session.accepted_prefix_v3().1.iter().try_fold(0u64,|n,f|n.checked_add(f.input.events.len() as u64))
            .ok_or_else(||error("사건 합계 초과"))?;
        let owner=PendingSupplyOwnerV1::issue()?;let response=owner.registration(accepted_events);
        self.frontend=Some(owner);Ok(response)
    }
    pub(super) fn prepare_frontend_finish(&self,session:&mut logical::LogicalInputSession,packet:&str)
        ->Result<PreparedDirectFinishV1,String> {self.prepare_finish(session,Some(packet))}
    fn prepare_finish(&self,session:&mut logical::LogicalInputSession,packet:Option<&str>)
        ->Result<PreparedDirectFinishV1,String> {
        self.ensure_running()?;self.check_owner(session)?;
        // An exclusive borrow fences this synchronous direct owner. It has
        // no admitted-but-unprocessed queue; no host pending Boolean is read.
        let (binding,accepted,failure)=session.accepted_prefix_v3();
        if failure.is_some(){return Err(error("실패한 기록을 정상 마칠 수 없습니다"));}
        session.borrow_result_boundary()?;
        let frames=accepted.iter().map(|frame|Frame::from_input(
            Snapshot::from(&frame.input),&frame.accepted_world_digest)).collect::<Result<Vec<_>,_>>()?;
        self.prepare_accepted_finish(ExpectedBinding::Logical(binding),frames,session.summary(),packet)
    }
    fn prepare_accepted_finish(&self,expected:ExpectedBinding<'_>,frames:Vec<Frame>,summary:serde_json::Value,packet:Option<&str>)
        ->Result<PreparedDirectFinishV1,String> {
        let accepted_event_count=frames.iter().try_fold(0u64,|n,f|n.checked_add(f.input.events.len() as u64))
            .ok_or_else(||error("사건 합계 초과"))?;
        let notice=match (&self.frontend,packet) {
            (None,None)=>None,
            (Some(owner),Some(packet))=>Some(owner.check(packet,accepted_event_count,&expected)?),
            _=>return Err(error("MissingPendingSupply: 실제 입력 생산자와 queue 공급이 필요합니다")),
        };
        // This is the explicit live end issuer, NOT a fabricated finite plan.
        let tape=Tape{binding:expected.binding(),
            accepted_madi_count:frames.len() as u64,accepted_event_count,frames,
            termination:logical::Termination::Complete,failure_reason:None};
        let bytes=tape.encode(&expected)?;
        let proof=RecordedEndProofV1{generation:self.generation.clone(),
            accepted_prefix_digest:digest(DOMAIN,&bytes),closed_count:tape.accepted_madi_count};
        Ok(PreparedDirectFinishV1{bytes,summary,proof,notice,pending_packet:packet.map(str::to_owned)})
    }
    pub(super) fn commit_direct_finish(&mut self,session:&mut logical::LogicalInputSession,
        candidate:PreparedDirectFinishV1)->Result<(),String> {
        self.ensure_running()?;self.check_owner(session)?;
        // A step interleaved after preparation invalidates the candidate.
        let current=self.prepare_finish(session,candidate.pending_packet.as_deref())?;
        self.commit_candidate(current,candidate)
    }
    #[cfg(windows)]
    pub(super) fn validate_direct_candidate(&self,session:&mut logical::LogicalInputSession,candidate:&PreparedDirectFinishV1)->Result<(),String>{
        self.ensure_running()?;self.check_owner(session)?;
        let current=self.prepare_finish(session,candidate.pending_packet.as_deref())?;
        self.check_candidate_pair(&current,candidate)
    }
    fn commit_candidate(&mut self,current:PreparedDirectFinishV1,candidate:PreparedDirectFinishV1)->Result<(),String> {
        self.check_candidate_pair(&current,&candidate)?;
        self.state=RecordingStateV1::Finished{bytes:candidate.bytes,summary:candidate.summary,proof:candidate.proof,notice:candidate.notice};
        Ok(())
    }
    fn check_candidate_pair(&self,current:&PreparedDirectFinishV1,candidate:&PreparedDirectFinishV1)->Result<(),String>{
        let same_notice=match (&current.notice,&candidate.notice) {
            (None,None)=>true,(Some(a),Some(b))=>a.matches(b),_=>false,
        };
        if !self.generation.matches(&candidate.proof.generation)
            || current.proof.closed_count!=candidate.proof.closed_count
            || current.proof.accepted_prefix_digest!=candidate.proof.accepted_prefix_digest
            || current.bytes!=candidate.bytes || current.summary!=candidate.summary || !same_notice {
            return Err(error("마치기 후보 뒤에 실행 경계가 바뀌었습니다"));
        }
        Ok(())
    }
    pub(super) fn finished(&self)->Result<(Vec<u8>,serde_json::Value),String> {
        match &self.state {
            RecordingStateV1::Finished{bytes,summary,proof,..}=>{
                if !self.generation.matches(&proof.generation){return Err(error("종료 증거 세대 불일치"));}
                Ok((bytes.clone(),summary.clone()))
            },
            _=>Err(error("정상 마친 기록이 없습니다")),
        }
    }
    pub(super) fn finish_excluded_count(&self)->Result<u64,String> {
        match &self.state {
            RecordingStateV1::Finished{notice,..}=>Ok(notice.as_ref().map_or(0,CheckedPendingAtFinishV1::count)),
            _=>Err(error("정상 종료 전에는 입력 제외 성공을 게시하지 않습니다")),
        }
    }
    pub(super) fn stop_logical(&mut self,session:&mut logical::LogicalInputSession)
        ->Result<(Vec<u8>,serde_json::Value),String> {
        self.ensure_running()?;self.check_owner(session)?;
        let failed=session.accepted_prefix_v3().2.is_some();
        let output=LiveRecording::default().stop_logical(session)?;
        self.state=if failed{RecordingStateV1::Aborted}else{RecordingStateV1::StoppedIncomplete};
        Ok(output)
    }
}
