// Explicit cold owner. SQL/LC candidates alone never issue this admission.
pub(super) struct CheckedColdFinishV1 {
    headers:super::super::finish_recovery_store::FinishRecoveryHeadersV1,
    lifecycle:Vec<u8>,witness:Vec<u8>,ready:Vec<u8>,c01:Vec<u8>,
    notice:serde_json::Value,original_cause:Option<String>,
}
impl Drop for CheckedColdFinishV1 {fn drop(&mut self){self.witness.fill(0);}}

fn same_cold_anchor(a:&VerifiedExecutionFirstCheckpointV1,b:&VerifiedExecutionFirstCheckpointV1)->bool{
    a.first_tx==b.first_tx && a.checkpoint_tx==b.checkpoint_tx && a.pesi==b.pesi
        && a.receipt==b.receipt && a.anchor_madi()==b.anchor_madi()
}

fn check_cold_finish(adapter:&mut super::super::SqliteAdapterV1,proof:&VerifiedExecutionFirstCheckpointV1,
    binding:&crate::logical_input_tape::LogicalInputBinding,
    context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
    ->Result<CheckedColdFinishV1,Failure>{
    use crate::replay_evidence::{ReplayLifecycleCandidateV1,ReplayBundleCandidateV1,
        finish_recovery::{RecordHeaderV1,witness::WitnessCandidateV1}};
    use crate::storage_observation::Span;
    let _total=Span::start("finish_recovery.cold_check_total");
    let before=Span::start("finish_recovery.current_before");
    let actual=adapter.revalidate_replay_scope_current_world(&proof.scope,context)?;
    let current=actual.replay_checkpoint_proof(adapter,context)?;
    if !same_cold_anchor(proof,&current){return Err(Failure::CustodyConflict);}
    drop(before);
    let payload=Span::start("finish_recovery.protected_payload_and_archive");
    let lifecycle=proof.scope.read_replay_lifecycle(&proof.first_tx)?.ok_or(Failure::RecoveryRequired)?;
    let life=ReplayLifecycleCandidateV1::parse(&lifecycle).map_err(|_|Failure::CorruptRecoveryRecord)?;
    let pending=match life.payload(8){Some([0])=>true,Some([1])=>false,_=>return Err(Failure::CorruptRecoveryRecord)};
    if life.payload(9)!=Some(&[1][..]) || !life.payload(17).is_some_and(|kind|kind[0]<=1){return Err(Failure::RecoveryRequired);}
    let headers=adapter.read_finish_recovery_headers(proof,&lifecycle)?;
    let intent=RecordHeaderV1::parse(headers.intent().raw()).map_err(|_|Failure::CorruptRecoveryRecord)?;
    let witness=proof.scope.read_finish_witness(&intent)?.ok_or(Failure::RecoveryRequired)?;
    let protected=WitnessCandidateV1::parse(&witness).map_err(|_|Failure::CorruptRecoveryRecord)?;
    protected.check_intent(&intent,headers.intent().digest()).map_err(|_|Failure::CustodyConflict)?;
    let cause_authenticated=protected.stage()==2;
    if pending && !cause_authenticated{return Err(Failure::RecoveryRequired);}
    if cause_authenticated{
        let failure=headers.failure().ok_or(Failure::RecoveryRequired)?;
        if protected.field(9)!=Some(failure.digest().as_slice()){return Err(Failure::CustodyConflict);}
    }
    // Witness/header binding is checked before a whole claimed C01 allocation.
    let (c01,cause)=adapter.read_finish_recovery_payloads(proof,&headers)?;
    let ready=intent.field(11).ok_or(Failure::CorruptRecoveryRecord)?.to_vec();
    let bundle=adapter.check_replay_archive_candidate(proof,ReplayArchiveReadKindV1::Finished,&ready)?;
    let bundle=ReplayBundleCandidateV1::parse(&bundle).map_err(|_|Failure::CorruptRecoveryRecord)?;
    let notice=crate::input_tape_v3::check_original_finish_response(&c01,
        bundle.payload(3).ok_or(Failure::CorruptRecoveryRecord)?,bundle.payload(4).ok_or(Failure::CorruptRecoveryRecord)?,
        bundle.payload(8).ok_or(Failure::CorruptRecoveryRecord)?,binding,proof.anchor_madi(),
        intent.field(17).ok_or(Failure::CorruptRecoveryRecord)?).map_err(|_|Failure::ArtifactMismatch)?;
    let original_cause=if cause_authenticated{
        Some(String::from_utf8(cause.ok_or(Failure::RecoveryRequired)?).map_err(|_|Failure::CorruptRecoveryRecord)?)
    }else{None};
    drop(payload);
    let after=Span::start("finish_recovery.current_after");
    let mut after=proof.scope.read_finish_witness(&intent)?.ok_or(Failure::RecoveryRequired)?;
    let same=after==witness;after.fill(0);
    if !same || proof.scope.read_replay_lifecycle(&proof.first_tx)?.as_deref()!=Some(lifecycle.as_slice()){
        return Err(Failure::CustodyConflict);
    }
    let actual=adapter.revalidate_replay_scope_current_world(&proof.scope,context)?;
    if !same_cold_anchor(proof,&actual.replay_checkpoint_proof(adapter,context)?){return Err(Failure::CustodyConflict);}
    drop(after);
    Ok(CheckedColdFinishV1{headers,lifecycle,witness,ready,c01,notice,original_cause})
}

impl VerifiedExecutionFirstCheckpointV1 {
    pub(crate) fn prepare_finish_recovery(self,adapter:&mut super::super::SqliteAdapterV1,
        binding:&crate::logical_input_tape::LogicalInputBinding,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<NativeFinishRecoveryPermitV1,Failure>{
        let checked=check_cold_finish(adapter,&self,binding,context)?;
        Ok(NativeFinishRecoveryPermitV1{proof:self,binding:binding.clone(),checked})
    }
}

impl NativeFinishRecoveryPermitV1 {
    /// Consumes one explicit admission. No closure/runtime callback is stored
    /// or reissued; the result is the authenticated original C01 plus notice.
    pub(crate) fn consume(self,adapter:&mut super::super::SqliteAdapterV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<RecoveredNativeFinishV1,Failure>{
        let mut current=check_cold_finish(adapter,&self.proof,&self.binding,context)?;
        let same_headers=current.headers.same_bytes(&self.checked.headers);
        if !same_headers || current.witness!=self.checked.witness || current.c01!=self.checked.c01
            || current.notice!=self.checked.notice || current.ready!=self.checked.ready{
            return Err(Failure::CustodyConflict);
        }
        // Another identical explicit request may already have completed the
        // transition. Return its original result, never write backwards/re-run.
        if current.lifecycle!=current.ready{
            if self.checked.lifecycle==self.checked.ready || current.lifecycle!=self.checked.lifecycle{
                return Err(Failure::CustodyConflict);
            }
            self.proof.scope.write_replay_lifecycle(&self.proof.first_tx,&current.ready)
                .map_err(|error|match error{
                    ReplayLifecycleWriteFailureV1::Before(cause)=>cause,
                    ReplayLifecycleWriteFailureV1::Uncertain(_)=>Failure::RecoveryRequired,
                })?;
        }
        // Full source/world/custody and original response are checked again
        // after flush/fresh. Partial repair is never published as success.
        let mut verified=check_cold_finish(adapter,&self.proof,&self.binding,context)?;
        if verified.lifecycle!=verified.ready || !verified.headers.same_bytes(&current.headers)
            || verified.witness!=current.witness || verified.c01!=current.c01 || verified.notice!=current.notice{
            return Err(Failure::CustodyConflict);
        }
        Ok(RecoveredNativeFinishV1{c01:std::mem::take(&mut verified.c01),
            notice:std::mem::take(&mut verified.notice),original_cause:verified.original_cause.take()})
    }
}
