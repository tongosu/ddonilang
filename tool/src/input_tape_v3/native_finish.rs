//! Native prepares without publishing Complete. Only its actual durable Ready
//! continuation may consume this candidate into the common finished cache.
use super::*;
pub(crate) struct PreparedNativeLogicalFinishV1 {inner:recording::PreparedDirectFinishV1}
impl PreparedNativeLogicalFinishV1 {
    pub(crate) fn tape_bytes(&self)->&[u8]{self.inner.planned_artifact_parts().0}
    pub(crate) fn summary(&self)->&serde_json::Value{self.inner.planned_artifact_parts().1}
    pub(crate) fn closed_count(&self)->u64{self.inner.closed_count()}
    pub(crate) fn generation(&self)->crate::replay_evidence::ExecutionGenerationV1{self.inner.generation()}
    pub(crate) fn recovery_payload_bytes(&self)->Result<Vec<u8>,String>{self.inner.recovery_payload_bytes()}
    pub(crate) fn recovery_notice_payload(&self)->Vec<u8>{self.inner.recovery_notice_payload()}
}
impl LiveRecording {
    pub(crate) fn native_finished_output(&self)->Result<(Vec<u8>,serde_json::Value),String>{
        self.completion.as_ref().ok_or_else(||error("MissingReplayProgress"))?.finished()
    }
    pub(crate) fn prepare_native_logical_finish(&mut self,session:&mut logical::LogicalInputSession,
        pending:Option<&str>)->Result<PreparedNativeLogicalFinishV1,String>{
        self.ensure_running()?;
        let owner=self.completion.as_ref().ok_or_else(||error("MissingReplayProgress: 실제 기록 생산자가 없습니다"))?;
        let inner=match pending {
            Some(packet)=>owner.prepare_frontend_finish(session,packet)?,
            None=>owner.prepare_direct_finish(session)?,
        };
        owner.validate_direct_candidate(session,&inner)?;
        Ok(PreparedNativeLogicalFinishV1{inner})
    }
    pub(crate) fn validate_native_logical_finish(&self,session:&mut logical::LogicalInputSession,
        candidate:&PreparedNativeLogicalFinishV1)->Result<(),String>{
        self.ensure_running()?;
        self.completion.as_ref().ok_or_else(||error("MissingReplayProgress: 실제 기록 생산자가 없습니다"))?
            .validate_direct_candidate(session,&candidate.inner)
    }
    pub(crate) fn publish_native_logical_finish_after_ready(&mut self,session:&mut logical::LogicalInputSession,
        candidate:PreparedNativeLogicalFinishV1,proof:&crate::replay_evidence::NativeFinishedRecordingReadyV1,
        adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1)->Result<(Vec<u8>,serde_json::Value),String>{
        self.ensure_running()?;
        proof.check_candidate(adapter,session,&candidate)?;
        let owner=self.completion.as_mut().ok_or_else(||error("MissingReplayProgress: 실제 기록 생산자가 없습니다"))?;
        owner.commit_direct_finish(session,candidate.inner)?;
        owner.finished()
    }
}
