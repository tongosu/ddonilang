//! Actual execution producer -> exact retained request. No raw/JSON constructor.
use super::{native::StoredPayloadV1,SnapshotV2DurabilityFailureV1 as Failure,TransactionBindingV1};
use crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment;

/// A compilation/execution-owned request, not a root/start/receipt capability.
/// Fields and constructors from bytes are deliberately private to this owner.
pub(crate) struct ActualPreparedRequestV1 {
    raw:std::sync::Arc<[u8]>,
    payload:StoredPayloadV1,
}

/// Explicit expected bytes at cold entry. Integrity/source checks alone do
/// not prove a previous execution or issue a start/restore capability.
#[cfg(windows)]
pub(super) struct ExplicitRecoveryRequestV1 {
    raw:std::sync::Arc<[u8]>,
    payload:StoredPayloadV1,
}

#[cfg(windows)]
impl ExplicitRecoveryRequestV1 {
    pub(super) fn check(raw:&[u8],source:&SourceBoundSavedRuntimeAttachment<'_>)->Result<Self,Failure> {
        if raw.is_empty() || raw.len()>super::recovery_custody::REQUEST_MAX {return Err(Failure::LimitExceeded);}
        let payload=check_expected_request(raw,source)?;
        if !payload.binding.expected_parent.is_empty() {return Err(Failure::UnsupportedDurability);}
        Ok(Self{raw:raw.into(),payload})
    }
    pub(super) fn raw(&self)->&[u8] {&self.raw}
    pub(super) fn payload(&self)->&StoredPayloadV1 {&self.payload}
    pub(super) fn check_source(&self,source:&SourceBoundSavedRuntimeAttachment<'_>)->Result<(),Failure> {
        let project=hex::decode(source.owned().project().authored_identity.logical_dependency_id.as_str())
            .map_err(|_|Failure::CustodyConflict)?;
        if self.payload.binding.pesi!=*source.pesi().as_raw() || self.payload.envelope!=source.revision_envelope()
            || self.payload.binding.project!=project
            || self.payload.binding.execution_profile!=source.owned().project().source_profile.as_bytes() {
            return Err(Failure::CustodyConflict);
        }
        Ok(())
    }
}

fn check_expected_request(raw:&[u8],source:&SourceBoundSavedRuntimeAttachment<'_>)->Result<StoredPayloadV1,Failure> {
    // Bind R5 value admission to the actual retained source owner. Neither a
    // caller-provided schema map nor a JSON admission flag supplies it.
    let catalog=source.owned().registration().map(|registration|registration.saved_nominal_catalog(source.owned()))
        .transpose().map_err(|_|Failure::CustodyConflict)?;
    let checked=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2_with_catalog(raw,source.pesi().as_raw(),catalog.as_ref())?;
    if checked.request_kind!=1 || checked.revision_bytes!=source.revision_envelope()
        || checked.encode_canonical_with_catalog(catalog.as_ref())?!=raw {return Err(Failure::CustodyConflict);}
    let project=hex::decode(source.owned().project().authored_identity.logical_dependency_id.as_str())
        .map_err(|_|Failure::CustodyConflict)?;
    if checked.binding.project!=project
        || checked.binding.execution_profile!=source.owned().project().source_profile.as_bytes() {
        return Err(Failure::CustodyConflict);
    }
    Ok(StoredPayloadV1{digest:checked.binding.transaction_digest()?,binding:checked.binding,
        snapshot:checked.snapshot_bytes.to_vec(),envelope:checked.revision_bytes.to_vec()})
}

impl ActualPreparedRequestV1 {
    pub(crate) fn from_general_continuation(run:&crate::runtime_surface::VerifiedGeneralNativeRun,
        source:&SourceBoundSavedRuntimeAttachment<'_>,parent:&[u8],revision:&[u8])->Result<Self,String>{
        let raw=run.checkpoint_request(source,parent,revision)?;
        Self::from_actual_encoder(raw,source).map_err(|e|format!("E_PROJECT_RUNTIME_NATIVE: 실제 이어 저장 요청 결속 실패 ({e:?})"))
    }
    pub(crate) fn from_general(
        run:&crate::runtime_surface::GeneralSuccessfulInputPrefix,
        source:&SourceBoundSavedRuntimeAttachment<'_>,parent:&[u8],revision:&[u8],
    )->Result<Self,String> {
        let raw=run.encode_checkpoint_request(source,parent,revision)?;
        Self::from_actual_encoder(raw,source).map_err(|failure|format!("E_PROJECT_RUNTIME_NATIVE: 실제 요청 결속 실패 ({failure:?})"))
    }

    pub(crate) fn from_logical(
        run:&crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1,
        source:&SourceBoundSavedRuntimeAttachment<'_>,parent:&[u8],revision:&[u8],
    )->Result<Self,Failure> {
        run.require_native_checkpoint_provenance()?;
        let world=run.encode_world_frame()?;
        Self::from_actual_encoder(run.encode_checkpoint_request(&world,source,parent,revision)?,source)
    }

    fn from_actual_encoder(raw:Vec<u8>,source:&SourceBoundSavedRuntimeAttachment<'_>)->Result<Self,Failure> {
        let payload=check_expected_request(&raw,source)?;
        Ok(Self{raw:raw.into(),payload})
    }

    pub(crate) fn raw(&self)->&[u8] {&self.raw}
    pub(super) fn shared_raw(&self)->std::sync::Arc<[u8]> {self.raw.clone()}
    pub(crate) fn binding(&self)->&TransactionBindingV1 {&self.payload.binding}
    pub(crate) fn payload(&self)->&StoredPayloadV1 {&self.payload}
    pub(crate) fn check_source(&self,source:&SourceBoundSavedRuntimeAttachment<'_>)->Result<(),Failure> {
        let catalog=source.owned().registration().map(|registration|registration.saved_nominal_catalog(source.owned()))
            .transpose().map_err(|_|Failure::CustodyConflict)?;
        let checked=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2_with_catalog(&self.raw,source.pesi().as_raw(),catalog.as_ref())?;
        let project=hex::decode(source.owned().project().authored_identity.logical_dependency_id.as_str()).map_err(|_|Failure::CustodyConflict)?;
        if checked.request_kind!=1 || checked.binding!=self.payload.binding
            || checked.snapshot_bytes!=self.payload.snapshot || checked.revision_bytes!=source.revision_envelope()
            || checked.revision_bytes!=self.payload.envelope || checked.binding.project!=project {
            return Err(Failure::CustodyConflict);
        }
        Ok(())
    }
}

/// Owns one consumed Pending-start capability and the still-uncommitted SQL
/// write. Dropping it rolls back SQLite but deliberately never clears Pending.
#[cfg(windows)]
pub(crate) struct InitialCustodyWriteV1 {
    start:super::native::DurableAttemptStartV1,
    row:super::recovery_custody::InitialCustodyRowV1,
    handle:super::native::PreparedWriteV1,
}

/// Mechanical durable prepared bytes only, not Verified/Committed evidence.
#[cfg(windows)]
pub(crate) struct InitialCustodyPreparedV1 {
    _start:super::native::DurableAttemptStartV1,
    _row:super::recovery_custody::InitialCustodyRowV1,
}

#[cfg(windows)]
impl InitialCustodyPreparedV1 {
    pub(crate) fn revalidate_prepared(&self,adapter:&super::native::SqliteAdapterV1,source:&SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<(),Failure> {
        self._start.check_current(adapter,source)?;
        adapter.reread_initial_prepared_with_custody(self._start.request(),&self._row).map(|_|())
    }
}

#[cfg(windows)]
impl InitialCustodyWriteV1 {
    pub(crate) fn begin(adapter:&super::native::SqliteAdapterV1,source:&SourceBoundSavedRuntimeAttachment<'_>,
        start:super::native::DurableAttemptStartV1)->Result<Self,Failure> {
        start.check_current(adapter,source)?;
        let row=super::recovery_custody::InitialCustodyRowV1::from_actual_request(start.request())?;
        let handle=adapter.write_initial_prepared_with_custody(&start,source,&row)?;
        Ok(Self{start,row,handle})
    }
    pub(crate) fn sync_and_reread(mut self,adapter:&super::native::SqliteAdapterV1,source:&SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<InitialCustodyPreparedV1,Failure> {
        self.start.check_current(adapter,source)?;
        adapter.sync_prepared(&mut self.handle)?;
        self.start.check_current(adapter,source)?;
        adapter.reread_initial_prepared_with_custody(self.start.request(),&self.row)?;
        Ok(InitialCustodyPreparedV1{_start:self.start,_row:self.row})
    }
}

/// One actual owner execution. These fields cannot be deserialized or turned
/// into a receipt: terminal record/Witness retention is a subsequent gate.
#[cfg(windows)]
pub(crate) struct UnretainedCustodyOutcomeV1 {
    pub(super) owner:super::NativeOwnerOutcomeV1,
    pub(super) start:super::native::DurableAttemptStartV1,
    pub(super) initial:super::recovery_custody::InitialCustodyRowV1,
    // Keep the exact issued row even if its SQL COMMIT acknowledgement was
    // lost. This is not a claim that it persisted; retention must reread it.
    pub(super) issued_verified:Option<super::recovery_custody::VerifiedCustodyRowV1>,
    pub(super) append_completed:Option<VerifiedAppendCompletionV1>,
}

/// In-process provenance of this exact append/sync/reread return. Not a saved
/// proof or a substitute for future complete-prefix/Witness revalidation.
#[cfg(windows)]
pub(super) struct VerifiedAppendCompletionV1(());

/// Outcome of one actual stateful recovery loop. The original cause remains
/// in owner; this attempt's mechanical failure is separate and never rewrites it.
#[cfg(windows)]
pub(crate) struct UnretainedRecoveryOutcomeV1 {
    pub(super) start:super::native::RecoveryAttemptStartV1,
    pub(super) owner:super::NativeOwnerOutcomeV1,
    pub(super) attempt_failure:Option<Failure>,
}

#[cfg(windows)]
pub(super) struct SqlRetainedRecoveryV1 {
    actual:UnretainedRecoveryOutcomeV1,
    row:super::recovery_custody::RecoveryOutcomeCustodyRowV1,
}

#[cfg(windows)]
pub(crate) struct RetainedRecoveryOutcomeV1 {
    sql:SqlRetainedRecoveryV1,
    _witness:super::native::RetainedAttemptWitnessV1,
}

#[cfg(windows)]
pub(crate) struct RecoveryRetentionBlockedV1 {
    pub(super) actual:UnretainedRecoveryOutcomeV1,
    pub(super) retention_failure:Failure,
}

#[cfg(windows)]
impl RecoveryRetentionBlockedV1 {
    pub(crate) fn diagnostic(&self)->String {
        format!("state={:?}, original={:?}, attempt={:?}, retention={:?}",self.actual.owner.evidence().state(),
            self.actual.owner.failure(),self.actual.attempt_failure,self.retention_failure)
    }
}

#[cfg(windows)]
impl UnretainedRecoveryOutcomeV1 {
    pub(crate) fn retain(self,adapter:&super::native::SqliteAdapterV1,source:&SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<RetainedRecoveryOutcomeV1,RecoveryRetentionBlockedV1> {
        let row=match super::recovery_custody::RecoveryOutcomeCustodyRowV1::from_outcome(&self) {
            Ok(row)=>row,Err(retention_failure)=>return Err(RecoveryRetentionBlockedV1{actual:self,retention_failure}),
        };
        if let Err(retention_failure)=adapter.retain_recovery_outcome(&self,&row,source) {
            return Err(RecoveryRetentionBlockedV1{actual:self,retention_failure});
        }
        #[cfg(test)]
        super::pause_native_owner_test_process("after_recovery_outcome_custody");
        let sql=SqlRetainedRecoveryV1{actual:self,row};
        match sql.actual.start.retain_record_witness(adapter,source,&sql) {
            Ok(witness)=>{
                #[cfg(test)]
                super::pause_native_owner_test_process("after_recovery_witness");
                Ok(RetainedRecoveryOutcomeV1{sql,_witness:witness})
            },
            Err(retention_failure)=>Err(RecoveryRetentionBlockedV1{actual:sql.actual,retention_failure}),
        }
    }
}

#[cfg(windows)]
impl SqlRetainedRecoveryV1 {
    pub(super) fn actual(&self)->&UnretainedRecoveryOutcomeV1 {&self.actual}
    pub(super) fn row(&self)->&super::recovery_custody::RecoveryOutcomeCustodyRowV1 {&self.row}
    pub(super) fn revalidate(&self,adapter:&super::native::SqliteAdapterV1)->Result<(),Failure> {
        adapter.reread_recovery_outcome(&self.actual,&self.row)
    }
}

#[cfg(windows)]
impl RetainedRecoveryOutcomeV1 {
    pub(crate) fn owner(&self)->&super::NativeOwnerOutcomeV1 {&self.sql.actual.owner}
    pub(crate) fn attempt_failure(&self)->Option<Failure> {self.sql.actual.attempt_failure}
}

/// Constructed only after the exact terminal/failure row was committed and a
/// new SQLite read snapshot verified the payload and every preceding row.
#[cfg(windows)]
pub(super) struct SqlRetainedOutcomeV1 {
    actual:UnretainedCustodyOutcomeV1,
    row:super::recovery_custody::ActualOutcomeCustodyRowV1,
}

/// Failure of retention is separate from the original execution outcome.
/// In particular a Committed transaction is never rewritten as a failed one.
#[cfg(windows)]
pub(crate) struct RetentionBlockedV1 {
    pub(super) actual:UnretainedCustodyOutcomeV1,
    pub(super) retention_failure:Failure,
}

#[cfg(windows)]
impl RetentionBlockedV1 {
    pub(crate) fn diagnostic(&self)->String {
        format!("state={:?}, original={:?}, retention={:?}",self.actual.owner.evidence().state(),
            self.actual.owner.failure(),self.retention_failure)
    }
}

/// Actual owner + retained full SQL prefix + matching durable host Witness.
/// This still is not a public receipt or a cold-start restore capability.
#[cfg(windows)]
pub(crate) struct RetainedOutcomeV1 {
    sql:SqlRetainedOutcomeV1,
    _witness:super::native::RetainedAttemptWitnessV1,
}

#[cfg(windows)]
impl UnretainedCustodyOutcomeV1 {
    pub(crate) fn retain(self,adapter:&super::native::SqliteAdapterV1,
        source:&SourceBoundSavedRuntimeAttachment<'_>)->Result<RetainedOutcomeV1,RetentionBlockedV1> {
        let row=match super::recovery_custody::ActualOutcomeCustodyRowV1::from_outcome(&self) {
            Ok(row)=>row,Err(retention_failure)=>return Err(RetentionBlockedV1{actual:self,retention_failure}),
        };
        if let Err(retention_failure)=adapter.retain_initial_outcome(&self,&row,source) {
            return Err(RetentionBlockedV1{actual:self,retention_failure});
        }
        #[cfg(test)]
        super::pause_native_owner_test_process("after_outcome_custody");
        let sql=SqlRetainedOutcomeV1{actual:self,row};
        match sql.actual.start.retain_record_witness(adapter,source,&sql) {
            Ok(witness)=>{
                #[cfg(test)]
                super::pause_native_owner_test_process("after_record_witness");
                Ok(RetainedOutcomeV1{sql,_witness:witness})
            },
            Err(retention_failure)=>Err(RetentionBlockedV1{actual:sql.actual,retention_failure}),
        }
    }
}

#[cfg(windows)]
impl SqlRetainedOutcomeV1 {
    pub(super) fn actual(&self)->&UnretainedCustodyOutcomeV1 {&self.actual}
    pub(super) fn row(&self)->&super::recovery_custody::ActualOutcomeCustodyRowV1 {&self.row}
    pub(super) fn revalidate(&self,adapter:&super::native::SqliteAdapterV1)->Result<(),Failure> {
        adapter.reread_initial_outcome(&self.actual,&self.row)
    }
}

#[cfg(windows)]
impl RetainedOutcomeV1 {
    pub(crate) fn owner(&self)->&super::NativeOwnerOutcomeV1 {&self.sql.actual.owner}
}

#[cfg(windows)]
pub(super) struct NativeCustodySessionV1<'a,'s> {
    adapter:&'a super::native::SqliteAdapterV1,
    source:&'a SourceBoundSavedRuntimeAttachment<'s>,
    start:super::native::DurableAttemptStartV1,
    initial:super::recovery_custody::InitialCustodyRowV1,
    queue:std::cell::RefCell<Option<super::recovery_custody::VerifiedCustodyRowV1>>,
    consumed:std::cell::RefCell<Option<super::recovery_custody::VerifiedCustodyRowV1>>,
    append_completed:std::cell::Cell<Option<VerifiedAppendCompletionV1>>,
    started:std::cell::Cell<bool>,
}

#[cfg(windows)]
impl<'a,'s> NativeCustodySessionV1<'a,'s> {
    pub(super) fn new(adapter:&'a super::native::SqliteAdapterV1,source:&'a SourceBoundSavedRuntimeAttachment<'s>,
        start:super::native::DurableAttemptStartV1)->Result<Self,Failure> {
        start.check_current(adapter,source)?;
        let initial=super::recovery_custody::InitialCustodyRowV1::from_actual_request(start.request())?;
        Ok(Self{adapter,source,start,initial,queue:Default::default(),consumed:Default::default(),append_completed:Default::default(),started:std::cell::Cell::new(false)})
    }
    pub(super) fn payload(&self)->&StoredPayloadV1 {self.start.request().payload()}
    pub(super) fn queue_verified(&self,evidence:&super::TransactionEvidenceV1)->Result<(),Failure> {
        let mut queue=self.queue.try_borrow_mut().map_err(|_|Failure::CustodyConflict)?;
        if !self.started.get() || queue.is_some() {return Err(Failure::CustodyConflict);}
        self.start.check_current(self.adapter,self.source)?;
        *queue=Some(super::recovery_custody::VerifiedCustodyRowV1::from_verified(evidence,&self.initial,self.start.request())?);
        Ok(())
    }
    pub(super) fn into_unretained(self,owner:super::NativeOwnerOutcomeV1)->Result<UnretainedCustodyOutcomeV1,Failure> {
        if owner.evidence().binding!=*self.start.request().binding() {return Err(Failure::CustodyConflict);}
        Ok(UnretainedCustodyOutcomeV1{owner,start:self.start,initial:self.initial,issued_verified:self.consumed.into_inner(),append_completed:self.append_completed.into_inner()})
    }
}

#[cfg(windows)]
impl super::native_adapter_sealed::Sealed for NativeCustodySessionV1<'_,'_> {}

#[cfg(windows)]
impl super::SnapshotV2DurabilityAdapterV1 for NativeCustodySessionV1<'_,'_> {
    type PreparedHandle=super::native::PreparedWriteV1;
    fn capability(&self)->Result<&'static str,Failure> {
        self.start.check_current(self.adapter,self.source)?;self.adapter.capability()
    }
    fn write_prepared(&self,payload:&StoredPayloadV1)->Result<Self::PreparedHandle,Failure> {
        if payload!=self.payload() || self.started.replace(true) {return Err(Failure::CustodyConflict);}
        self.adapter.write_initial_prepared_with_custody(&self.start,self.source,&self.initial)
    }
    fn sync_prepared(&self,handle:&mut Self::PreparedHandle)->Result<(),Failure> {
        self.start.check_current(self.adapter,self.source)?;self.adapter.sync_prepared(handle)
    }
    fn reread_prepared(&self,digest:&[u8;32])->Result<Option<StoredPayloadV1>,Failure> {
        if digest!=self.initial.digest() {return Err(Failure::CustodyConflict);}
        self.start.check_current(self.adapter,self.source)?;
        self.adapter.reread_initial_prepared_with_custody(self.start.request(),&self.initial).map(Some)
    }
    fn commit_verified(&self,payload:&StoredPayloadV1,expected:Option<&super::native::StoredHeadV1>)->Result<(),Failure> {
        let mut consumed=self.consumed.try_borrow_mut().map_err(|_|Failure::CustodyConflict)?;
        if consumed.is_some() {return Err(Failure::CustodyConflict);}
        *consumed=Some(self.queue.try_borrow_mut().map_err(|_|Failure::CustodyConflict)?.take().ok_or(Failure::CustodyConflict)?);
        let verified=consumed.as_ref().ok_or(Failure::CustodyConflict)?;
        if payload!=self.payload() || expected.is_some() {return Err(Failure::CustodyConflict);}
        self.adapter.append_verified_initial_custody(&self.start,self.source,&self.initial,verified)?;
        self.append_completed.set(Some(VerifiedAppendCompletionV1(())));
        self.adapter.commit_initial_verified_with_custody(&self.start,self.source,&self.initial,verified,payload,expected)
    }
    fn confirm_container_durable(&self)->Result<(),Failure> {
        self.start.check_current(self.adapter,self.source)?;self.adapter.confirm_container_durable()
    }
    fn reread_committed(&self,digest:&[u8;32],project:&[u8],revision:&[u8])->Result<super::native::RawChainV1,Failure> {
        self.start.check_current(self.adapter,self.source)?;
        if digest!=self.initial.digest() || project!=self.payload().binding.project || revision!=self.payload().binding.new_revision {
            return Err(Failure::CustodyConflict);
        }
        self.adapter.reread_committed(digest,project,revision)
    }
    fn abort(&self,handle:&mut Self::PreparedHandle)->Result<(),Failure> {self.adapter.abort(handle)}
    fn inspect_recovery(&self,digest:&[u8;32],project:&[u8],revision:&[u8])->Result<super::native::RawChainV1,Failure> {
        self.start.check_current(self.adapter,self.source)?;self.adapter.inspect_recovery(digest,project,revision)
    }
}
