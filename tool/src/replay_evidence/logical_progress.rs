//! An actual native recording owner, not a replay-only or restore capability.
use super::ResultPreimageV1;
use crate::{input_tape_v3::LiveRecording,logical_input_tape::{LogicalInputSession,LogicalSnapshot}};
use crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1;
pub(crate) struct LogicalRecordingRunV1 {run:CheckpointedLogicalRunV1,recording:LiveRecording}
impl std::ops::Deref for LogicalRecordingRunV1 {
    type Target=CheckpointedLogicalRunV1;
    fn deref(&self)->&Self::Target {&self.run}
}
impl LogicalRecordingRunV1 {
    #[cfg(windows)]
    pub(crate) fn prepare_native_finish(&mut self,pending:Option<&str>)->Result<crate::input_tape_v3::PreparedNativeLogicalFinishV1,String>{
        self.recording.prepare_native_logical_finish(self.run.session_mut_for_recording(),pending)
    }
    #[cfg(windows)]
    pub(crate) fn validate_native_finish(&mut self,candidate:&crate::input_tape_v3::PreparedNativeLogicalFinishV1)->Result<(),String>{
        self.recording.validate_native_logical_finish(self.run.session_mut_for_recording(),candidate)
    }
    #[cfg(windows)]
    pub(crate) fn register_native_finish_frontend(&mut self)->Result<serde_json::Value,String>{
        self.recording.register_frontend_logical(self.run.session_mut_for_recording())
    }
    #[cfg(windows)]
    pub(crate) fn publish_native_finish(&mut self,candidate:crate::input_tape_v3::PreparedNativeLogicalFinishV1,
        proof:&super::NativeFinishedRecordingReadyV1,adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1)
        ->Result<(Vec<u8>,serde_json::Value),String>{
        self.recording.publish_native_logical_finish_after_ready(self.run.session_mut_for_recording(),candidate,proof,adapter)
    }
    #[cfg(windows)]
    pub(crate) fn native_finished_output(&self)->Result<(Vec<u8>,serde_json::Value),String>{self.recording.native_finished_output()}
    #[cfg(windows)]
    pub(crate) fn native_finish_notice(&self)->Result<serde_json::Value,String>{self.recording.finish_notice()}
    #[cfg(windows)]
    pub(crate) fn from_native_progress(mut run:CheckpointedLogicalRunV1,
        proof:&crate::unit_wire::resumable_complete_state::CurrentRecordingProgressProofV1)->Result<Self,String>{
        run.original_replay_seed()?;proof.check_session(run.session())?;
        let recording=LiveRecording::from_native_running_progress(run.session_mut_for_recording(),proof)?;
        Ok(Self{run,recording})
    }
    pub(crate) fn begin(mut run:CheckpointedLogicalRunV1)->Result<Self,String> {
        run.original_replay_seed()?;
        let recording=LiveRecording::for_logical(run.session_mut_for_recording())?;
        run.session_mut_for_recording().require_result_evidence_from_start()?;
        Ok(Self{run,recording})
    }
    pub(crate) fn step(&mut self,input:&LogicalSnapshot)->Result<serde_json::Value,String> {
        self.recording.ensure_running()?;self.run.step(input)
    }
    pub(crate) fn progress_evidence(&self)->Result<LogicalProgressEvidenceV1,String> {
        LogicalProgressEvidenceV1::capture(self.recording.borrow_running_logical(self.run.session())?)
    }
    pub(crate) fn recording_session(&self)->&LogicalInputSession {self.run.session()}
    pub(crate) fn stop_no_end(&mut self)->Result<super::LogicalNoEndEvidenceV1,String>{
        super::LogicalNoEndEvidenceV1::capture(&mut self.recording,self.run.session())
    }
}
pub(crate) struct LogicalProgressEvidenceV1 {
    pub(super) input_contract:Vec<u8>,pub(super) input_evidence:Vec<u8>,
    pub(super) accepted_count:u64,pub(super) actual_receipts:Vec<u8>,pub(super) result:ResultPreimageV1,
}
impl LogicalProgressEvidenceV1 {
    pub(crate) fn result_digest(&self)->&[u8;32]{self.result.digest_bytes()}
    pub(crate) fn accepted_count(&self)->u64 {self.accepted_count}
    pub(crate) fn input_evidence(&self)->&[u8] {&self.input_evidence}
    fn capture(boundary:crate::input_tape_v3::RunningLogicalRecordingBorrowV1<'_>)->Result<Self,String> {
        let (input_contract,input_evidence,accepted_count,receipts)=boundary.transport_parts()?;
        let actual_receipts=super::receipt_json::actual_receipt_array(receipts)?;
        let result=ResultPreimageV1::capture(boundary.result_boundary()?)?;
        Ok(Self{input_contract,input_evidence,accepted_count,actual_receipts,result})
    }
}
