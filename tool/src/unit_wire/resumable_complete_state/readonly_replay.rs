//! Staging VM from recorded source/initial SEED only. No Deref, extraction,
//! live step, checkpoint export, save or normal restoration capability.
use super::*;
use crate::replay_evidence::{ReplayBundleCandidateV1,ReplaySeedCandidateV1,ResultPreimageV1};
pub(crate) struct ReadOnlyLogicalReplayV1 {session:crate::logical_input_tape::LogicalInputSession,result:ResultPreimageV1}
pub(crate) struct ReadOnlyCompletedLogicalReplayV1 {inner:ReadOnlyLogicalReplayV1}
#[cfg(windows)]
pub(super) struct ReadOnlyNewRecordPrefixV1 {anchor:ReadOnlyLogicalReplayV1}
#[cfg(windows)]
impl ReadOnlyNewRecordPrefixV1 {
    pub(super) fn check(bytes:&[u8],expected:&crate::logical_input_tape::LogicalInputBinding,anchor:u64)->Result<Self,String>{
        let full=ReadOnlyLogicalReplayV1::check_prefix(bytes,expected,ArchivePrefixKind::Terminal)?;
        let prefix=ReadOnlyLogicalReplayV1::check_prefix_limit(bytes,expected,ArchivePrefixKind::Terminal,Some(anchor))?;
        let count=usize::try_from(anchor).map_err(rejected)?;
        let actual=crate::replay_evidence::actual_receipt_array(prefix.session.committed_input_receipts())?;
        let expected=crate::replay_evidence::actual_receipt_array(full.session.committed_input_receipts().get(..count).ok_or("E_NEW_RECORD_ANCHOR")?)?;
        if actual!=expected{return Err("E_NEW_RECORD_ANCHOR: 원 기록과 실제 저장 경계의 입력이 다릅니다".into());}
        Ok(Self{anchor:prefix})
    }
    pub(super) fn check_normal_candidate(&self,normal:&crate::logical_input_tape::LogicalInputSession,start:&str)->Result<(),String>{
        self.anchor.check_normal_recording_candidate(normal,start)
    }
    pub(super) fn checked_result_bytes(&self)->&[u8]{self.anchor.checked_result_bytes()}
}
impl ReadOnlyCompletedLogicalReplayV1 {
    pub(crate) fn check(bytes:&[u8],expected:&crate::logical_input_tape::LogicalInputBinding)->Result<Self,String>{
        ReadOnlyLogicalReplayV1::check_prefix(bytes,expected,ArchivePrefixKind::Completed).map(|inner|Self{inner})
    }
    pub(crate) fn summary(&self)->serde_json::Value{self.inner.checked_progress_summary()}
    pub(crate) fn result_digest(&self)->&[u8;32]{self.inner.result_digest()}
}
#[derive(Clone,Copy)]
enum ArchivePrefixKind {Running,Completed,Terminal}
fn rejected(cause:impl std::fmt::Debug)->String{format!("E_REPLAY_ORIGINAL_SEED: {cause:?}")}
impl ReadOnlyLogicalReplayV1 {
    pub(crate) fn check_running_prefix(bytes:&[u8],expected:&crate::logical_input_tape::LogicalInputBinding)
        ->Result<Self,String>{
        Self::check_prefix(bytes,expected,ArchivePrefixKind::Running)
    }
    fn check_prefix(bytes:&[u8],expected:&crate::logical_input_tape::LogicalInputBinding,kind:ArchivePrefixKind)->Result<Self,String>{
        Self::check_prefix_limit(bytes,expected,kind,None)
    }
    fn check_prefix_limit(bytes:&[u8],expected:&crate::logical_input_tape::LogicalInputBinding,kind:ArchivePrefixKind,
        anchor:Option<u64>)->Result<Self,String>{
        let bundle=ReplayBundleCandidateV1::parse(bytes)?;
        let state=bundle.payload(11).and_then(|control|control.first()).copied().ok_or("E_REPLAY_RECORDING_STATE")?;
        let valid=match kind{ArchivePrefixKind::Running=>state==0,ArchivePrefixKind::Completed=>state==1,
            ArchivePrefixKind::Terminal=>(1..=3).contains(&state)};
        if !valid{
            return Err("E_REPLAY_RECORDING_STATE: 요구한 기록 상태가 다릅니다".into());
        }
        let source_raw=std::str::from_utf8(bundle.payload(1).ok_or("E_REPLAY_SOURCE")?).map_err(rejected)?;
        let source=super::super::project_runtime_attachment::source_only::PreparedSourceOnlyRevision::from_artifact(source_raw)?;
        let owned=source.recorded_execution_owner()?;
        let seed=ReplaySeedCandidateV1::parse(bundle.payload(2).ok_or("E_REPLAY_SEED")?)?;
        let payload=|tag|seed.payload(tag).ok_or_else(||"E_REPLAY_SEED: 필드 누락".to_owned());
        if payload(9)?!=source.envelope() || payload(1)?!=source.pesi()
            || payload(10)?!=encode_source_capture_supply(owned.sources()).map_err(rejected)? {
            return Err("E_REPLAY_SOURCE: 원 시작과 원문 봉인이 다릅니다".into());
        }
        check_constituent_supply(payload(11)?,source.pesi(),&owned).map_err(rejected)?;
        let candidate=rebuild_source_descriptor_candidate(payload(18)?,&owned).map_err(rejected)?;
        if candidate.initial_tick!=0 || candidate.phase!=0 || candidate.successful_madi_count!=0
            || candidate.base_rng_seed!=expected.initial_rng || !candidate.prior_durable_receipts.is_empty() {
            return Err("E_REPLAY_SEED: 원 시작이 아닌 뒤 시점 자료입니다".into());
        }
        let original_world=candidate.initial_world.replay_seed_world_payloads().map_err(rejected)?;
        for (index,field) in original_world.iter().enumerate(){
            if payload(index as u8+1)?!=field{return Err("E_REPLAY_SEED: 원 초기 세계가 다릅니다".into());}
        }
        let initial=candidate.initial_world.decode_value_tables().map_err(rejected)?;
        if !initial.resources.is_empty() || !initial.parameters.is_empty() || !initial.subjects.is_empty()
            || initial.subject_custody.is_some() || !candidate.initial_world.atom_evidence.is_empty(){
            return Err("E_UNSUPPORTED_READONLY_INITIAL_STATE: 이 논리 입력의 원 초기 세계는 지원 범위 밖입니다".into());
        }
        if payload(12)?!=encode_logical_execution_contract_supply(&owned,&candidate.runner).map_err(rejected)? {
            return Err("E_REPLAY_CONTRACT: 원 실행 계약이 다릅니다".into());
        }
        // Existing descriptor reconstruction uses recorded nominal/Subject
        // context. It never calls the fresh Source/Root/Subject constructors.
        let mut session=crate::logical_input_tape::LogicalInputSession::from_saved_runner(&owned,expected.clone(),candidate.runner)?;
        session.bind_readonly_seed_result_start(ReadOnlyLogicalSeedAdmissionV1{_private:()})?;
        session.require_result_evidence_from_start()?;
        if payload(13)?!=encode_processed_input_supply(&session).map_err(rejected)?
            || payload(14)?!=encode_logical_accepted_rng_content(&session).map_err(rejected)? {
            return Err("E_REPLAY_SEED_INPUT: 원 시작 입력·난수 공급이 다릅니다".into());
        }
        let definition=owned.definition_identity().as_bytes();
        let boundary=decode_boundary_sections(payload(14)?,payload(15)?,payload(16)?,0,
            LOGICAL_FIXED_SEED_PROFILE,definition,definition,&session.reconstructed_work_cost()).map_err(rejected)?;
        if boundary.base_seed!=expected.initial_rng{return Err("E_REPLAY_SEED_RNG: 원 씨앗이 다릅니다".into());}
        let contract=bundle.payload(3).ok_or("E_REPLAY_INPUT")?;let evidence=bundle.payload(4).ok_or("E_REPLAY_INPUT")?;
        let frames=match kind {
            ArchivePrefixKind::Running=>crate::input_tape_v3::checked_running_logical_archive_prefix(contract,evidence,expected)?,
            ArchivePrefixKind::Completed=>crate::input_tape_v3::checked_completed_logical_archive_prefix(contract,evidence,expected)?,
            ArchivePrefixKind::Terminal=>crate::input_tape_v3::checked_terminal_logical_archive_prefix(contract,evidence,expected,state)?,
        };
        let count=u64::try_from(frames.len()).map_err(rejected)?;
        let mut range=0u64.to_be_bytes().to_vec();range.extend_from_slice(&count.to_be_bytes());
        if bundle.payload(5)!=Some(range.as_slice()){return Err("E_REPLAY_INPUT: 원 처리 범위가 다릅니다".into());}
        let replay_count=anchor.unwrap_or(count);
        if replay_count>count{return Err("E_NEW_RECORD_ANCHOR: 원 확정 입력보다 뒤 시점을 복원할 수 없습니다".into());}
        for frame in frames.iter().take(usize::try_from(replay_count).map_err(rejected)?){session.replay_v3_frame(frame)?;}
        let result=ResultPreimageV1::capture(session.borrow_result_boundary()?)?;
        if anchor.is_none()&&(bundle.payload(6)!=Some(result.canonical_bytes()) || bundle.payload(7)!=Some(result.digest_bytes().as_slice())) {
            return Err("E_REPLAY_RESULT_MISMATCH: 원 입력으로 재생한 결과가 다릅니다".into());
        }
        let receipts=crate::replay_evidence::actual_receipt_array(session.committed_input_receipts())?;
        if anchor.is_none()&&bundle.payload(8)!=Some(receipts.as_slice()){return Err("E_REPLAY_INPUT_RECEIPTS: 실제 처리 입력 영수증이 다릅니다".into());}
        Ok(Self{session,result})
    }
    /// Progress comparison only, not normal-end ReplayVerified/public success.
    pub(crate) fn checked_progress_summary(&self)->serde_json::Value{self.session.summary()}
    pub(crate) fn result_digest(&self)->&[u8;32]{self.result.digest_bytes()}
    #[cfg(windows)]
    pub(super) fn check_normal_recording_candidate(&self,normal:&crate::logical_input_tape::LogicalInputSession,start:&str)->Result<(),String>{
        normal.check_reconstructed_continuation(&self.session,start)?;
        if normal.accepted_output_content()?!=self.session.accepted_output_content()? {
            return Err("E_NATIVE_RECORDING_OUTPUTS: 정상 복원과 원 기록의 형식 있는 출력이 다릅니다".into());
        }Ok(())
    }
    #[cfg(windows)]
    pub(super) fn checked_result_bytes(&self)->&[u8]{self.result.canonical_bytes()}
}
