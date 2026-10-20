//! RD-669: shared private transaction authority, independent of storage mechanics.
//! No callback, transport status, or visible head is a committed receipt.

use sha2::{Digest, Sha256};

mod recovery_custody;
#[cfg(not(target_arch="wasm32"))]
mod retained_attempt;
#[cfg(not(target_arch="wasm32"))]
pub(crate) use retained_attempt::ActualPreparedRequestV1;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod native;

#[cfg(not(target_arch = "wasm32"))]
mod native_adapter_sealed { pub trait Sealed {} }

/// RD-671 mechanics only. A successful method is not a receipt or a state
/// transition; the shared owner below still checks the ordered evidence.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) trait SnapshotV2DurabilityAdapterV1: native_adapter_sealed::Sealed {
    type PreparedHandle;
    fn capability(&self) -> Result<&'static str, SnapshotV2DurabilityFailureV1>;
    fn write_prepared(&self, payload: &native::StoredPayloadV1) -> Result<Self::PreparedHandle, SnapshotV2DurabilityFailureV1>;
    fn sync_prepared(&self, handle: &mut Self::PreparedHandle) -> Result<(), SnapshotV2DurabilityFailureV1>;
    fn reread_prepared(&self, digest: &[u8;32]) -> Result<Option<native::StoredPayloadV1>, SnapshotV2DurabilityFailureV1>;
    fn commit_verified(&self, payload: &native::StoredPayloadV1, expected: Option<&native::StoredHeadV1>) -> Result<(), SnapshotV2DurabilityFailureV1>;
    fn confirm_container_durable(&self) -> Result<(), SnapshotV2DurabilityFailureV1>;
    fn reread_committed(&self, digest: &[u8;32], project: &[u8], revision: &[u8]) -> Result<native::RawChainV1, SnapshotV2DurabilityFailureV1>;
    fn abort(&self, handle: &mut Self::PreparedHandle) -> Result<(), SnapshotV2DurabilityFailureV1>;
    fn inspect_recovery(&self, digest: &[u8;32], project: &[u8], revision: &[u8]) -> Result<native::RawChainV1, SnapshotV2DurabilityFailureV1>;
}

/// Private in-process failure information, not a new wire field/state/identity.
/// None means admission failed before a transaction was prepared. Preserve the
/// mechanical cause separately from the owner's recovery disposition.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeTransactionFailureV1 {
    pub state: Option<SnapshotV2TransactionStateV1>,
    pub cause: SnapshotV2DurabilityFailureV1,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<SnapshotV2DurabilityFailureV1> for NativeTransactionFailureV1 {
    fn from(cause: SnapshotV2DurabilityFailureV1) -> Self { Self { state: None, cause } }
}

/// The actual common loop's outcome, retained before its evidence is dropped.
/// Callers cannot construct a state/cause pair or deserialize this wrapper.
/// It is not source/root/custody admission or a persistence acknowledgement.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub(crate) struct NativeOwnerOutcomeV1 { inner: NativeOwnerOutcomeKindV1 }

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
enum NativeOwnerOutcomeKindV1 {
    Committed(TransactionEvidenceV1),
    Failed { evidence: TransactionEvidenceV1, failure: NativeTransactionFailureV1 },
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeOwnerOutcomeV1 {
    pub(crate) fn evidence(&self) -> &TransactionEvidenceV1 {
        match &self.inner { NativeOwnerOutcomeKindV1::Committed(evidence)
            | NativeOwnerOutcomeKindV1::Failed { evidence, .. } => evidence }
    }
    pub(crate) fn failure(&self) -> Option<&NativeTransactionFailureV1> {
        match &self.inner { NativeOwnerOutcomeKindV1::Committed(_) => None,
            NativeOwnerOutcomeKindV1::Failed { failure, .. } => Some(failure) }
    }
    fn into_result(self) -> Result<TransactionEvidenceV1, NativeTransactionFailureV1> {
        match self.inner { NativeOwnerOutcomeKindV1::Committed(evidence) => Ok(evidence),
            NativeOwnerOutcomeKindV1::Failed { failure, .. } => Err(failure) }
    }
}

/// Read-only admission of a retained terminal transaction. No new execution,
/// CAS/revision or source/world restore is issued by this private proof.
#[cfg(windows)]
pub(crate) struct ReadmittedCommittedV1 {
    _prior:native::AdmittedInitialAttemptV1,
    evidence:TransactionEvidenceV1,
}

#[cfg(windows)]
impl ReadmittedCommittedV1 {
    pub(crate) fn evidence(&self)->&TransactionEvidenceV1 {&self.evidence}
    pub(crate) fn payload(&self)->&native::StoredPayloadV1 {self._prior.expected_payload()}
}

/// A current admission failure must not relabel historical Committed as a
/// new RecoveryRequired transaction or replace its original recorded cause.
#[cfg(windows)]
pub(crate) struct ColdReadmissionBlockedV1 {
    pub(super) prior:native::AdmittedInitialAttemptV1,
    pub(super) failure:SnapshotV2DurabilityFailureV1,
}

#[cfg(windows)]
pub(crate) fn readmit_initial_committed_native(adapter:&native::SqliteAdapterV1,
    source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    prior:native::AdmittedInitialAttemptV1,
)->Result<ReadmittedCommittedV1,ColdReadmissionBlockedV1> {
    let admitted=(|| {
        prior.check_current(adapter,source)?;
        // Nonterminal recovery requires the next-epoch stateful barrier; do
        // not reuse this read-only terminal path as a recovery shortcut.
        if prior.historical_state()!=SnapshotV2TransactionStateV1::Committed {
            return Err(SnapshotV2DurabilityFailureV1::UnsupportedDurability);
        }
        adapter.confirm_container_durable()?;
        let expected=prior.expected_payload();
        let chain=adapter.reread_committed(&expected.digest,&expected.binding.project,&expected.binding.new_revision)?;
        checked_native_committed_payload(expected,&chain)?;
        prior.check_current(adapter,source)?;
        Ok(expected.binding.clone())
    })();
    match admitted {
        Ok(binding)=>Ok(ReadmittedCommittedV1{_prior:prior,evidence:TransactionEvidenceV1 {
            binding,state:SnapshotV2TransactionStateV1::Committed,
            // This describes the witnessed original transaction, not a CAS in
            // this process. The newly obtained confirmation was consumed by
            // the exact chain comparison above; it cannot be replayed later.
            head_changed:true,container_confirmed:false,
        }}),
        Err(failure)=>Err(ColdReadmissionBlockedV1{prior,failure}),
    }
}

/// Stateful recovery consumes a new acknowledged Pending epoch. It does not
/// rerun the source, replay inputs, invent a missing cause, or perform a CAS.
/// Fresh same-root confirmation precedes the exact committed-chain read.
#[cfg(windows)]
pub(crate) fn recover_initial_native_owner_with_custody(adapter:&native::SqliteAdapterV1,
    source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    start:native::RecoveryAttemptStartV1,
)->Result<retained_attempt::UnretainedRecoveryOutcomeV1,NativeTransactionFailureV1> {
    use SnapshotV2DurabilityFailureV1 as Failure;
    use SnapshotV2TransactionStateV1 as State;
    start.check_current(adapter,source)?;
    if start.prior_record().state()!=State::RecoveryRequired {return Err(Failure::CustodyConflict.into());}
    #[cfg(test)]
    pause_native_owner_test_process("after_recovery_pending_ack");
    let original_cause=start.prior_record().cause();
    let binding=start.expected_payload().binding.clone();
    let disposition=(|| {
        adapter.confirm_container_durable()?;
        let expected=start.expected_payload();
        let chain=adapter.reread_committed(&expected.digest,&binding.project,&binding.new_revision)?;
        let state=match checked_native_committed_payload(expected,&chain) {
            Ok(_)=>State::Committed,
            Err(_) if binding.expected_parent.is_empty() && chain.head.is_none() && chain.revision.is_none()
                && chain.payload.as_ref()==Some(expected)=>State::Aborted,
            Err(failure)=>return Err(failure),
        };
        start.check_current(adapter,source)?;
        Ok(state)
    })();
    let (state,attempt_failure)=match disposition {Ok(state)=>(state,None),Err(failure)=>(State::RecoveryRequired,Some(failure))};
    let evidence=TransactionEvidenceV1{binding,state,head_changed:state==State::Committed,container_confirmed:false};
    let inner=if state==State::Committed {NativeOwnerOutcomeKindV1::Committed(evidence)}else{
        NativeOwnerOutcomeKindV1::Failed{evidence,failure:NativeTransactionFailureV1{state:Some(state),cause:original_cause}}
    };
    Ok(retained_attempt::UnretainedRecoveryOutcomeV1{start,owner:NativeOwnerOutcomeV1{inner},attempt_failure})
}

#[cfg(not(target_arch="wasm32"))]
pub(crate) fn checked_native_committed_payload<'a>(expected:&native::StoredPayloadV1,reread:&'a native::RawChainV1)
    ->Result<&'a native::StoredPayloadV1,SnapshotV2DurabilityFailureV1> {
    use SnapshotV2DurabilityFailureV1 as Failure;
    if expected.digest!=expected.binding.transaction_digest()? {return Err(Failure::CustodyConflict);}
    let actual=reread.payload.as_ref().ok_or(Failure::ArtifactMismatch)?;
    if actual.binding!=expected.binding || actual.digest!=expected.digest {return Err(Failure::CustodyConflict);}
    if actual.snapshot!=expected.snapshot || actual.envelope!=expected.envelope {return Err(Failure::ArtifactMismatch);}
    let head=native::StoredHeadV1{project:expected.binding.project.clone(),revision:expected.binding.new_revision.clone(),
        integrity:expected.binding.revision_integrity,digest:expected.digest};
    if reread.head.as_ref()!=Some(&head) || reread.revision.as_ref()!=Some(&head) {return Err(Failure::CustodyConflict);}
    Ok(actual)
}

/// Execute only the Native durability/evidence vertical. The caller must first
/// admit source/capture/input/custody with its existing live execution owner.
/// This function cannot construct/publish a receipt, restore a world or grant
/// source admission from an opaque payload or a successful adapter callback.
#[cfg(windows)]
pub(crate) fn run_initial_native_owner_with_custody(
    adapter:&native::SqliteAdapterV1,source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    start:native::DurableAttemptStartV1,
)->Result<retained_attempt::UnretainedCustodyOutcomeV1,NativeTransactionFailureV1> {
    let session=retained_attempt::NativeCustodySessionV1::new(adapter,source,start)?;
    let owner=run_native_owner_outcome_with_verified(&session,session.payload(),None,b"vc04.sqlite-durability.r1",
        |evidence|session.queue_verified(evidence))?;
    Ok(session.into_unretained(owner)?)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn commit_native_evidence<A: SnapshotV2DurabilityAdapterV1>(
    adapter: &A, payload: &native::StoredPayloadV1, expected: Option<&native::StoredHeadV1>, expected_profile: &[u8],
) -> Result<TransactionEvidenceV1, NativeTransactionFailureV1> {
    // Existing callers keep exactly the old Result facade and failure classes.
    // The retained-custody path will consume the same loop's sealed outcome.
    run_native_owner_outcome(adapter, payload, expected, expected_profile)?.into_result()
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn run_native_owner_outcome<A: SnapshotV2DurabilityAdapterV1>(
    adapter: &A, payload: &native::StoredPayloadV1, expected: Option<&native::StoredHeadV1>, expected_profile: &[u8],
) -> Result<NativeOwnerOutcomeV1, NativeTransactionFailureV1> {
    run_native_owner_outcome_with_verified(adapter,payload,expected,expected_profile,|_|Ok(()))
}

#[cfg(not(target_arch = "wasm32"))]
fn run_native_owner_outcome_with_verified<A:SnapshotV2DurabilityAdapterV1,F>(
    adapter:&A,payload:&native::StoredPayloadV1,expected:Option<&native::StoredHeadV1>,expected_profile:&[u8],
    queue_verified:F,
)->Result<NativeOwnerOutcomeV1,NativeTransactionFailureV1>
where F:FnOnce(&TransactionEvidenceV1)->Result<(),SnapshotV2DurabilityFailureV1> {
    use SnapshotV2DurabilityFailureV1 as Failure;
    use SnapshotV2TransactionStateV1 as State;
    if payload.digest != payload.binding.transaction_digest()?
        || payload.snapshot.len() as u64 != payload.binding.snapshot_length
        || <[u8;32]>::from(Sha256::digest(&payload.snapshot)) != payload.binding.snapshot_sha256 {
        return Err(Failure::SnapshotHashMismatch.into());
    }
    if payload.envelope.is_empty() || expected.map(|head|head.revision.as_slice()).unwrap_or(&[]) != payload.binding.expected_parent
        || expected.is_some_and(|head|head.project != payload.binding.project) {
        return Err(Failure::CustodyConflict.into());
    }
    let mut evidence = TransactionEvidenceV1::prepared(payload.binding.clone())?;
    let result = (|| {
        if adapter.capability()?.as_bytes() != expected_profile { return Err(Failure::UnsupportedDurability); }
        #[cfg(all(windows,feature="beta2-crash-probe"))]
        native::crash_probe::checkpoint("before_prepared");
        let mut handle = adapter.write_prepared(payload)?;
        #[cfg(test)]
        pause_native_owner_test_process("before_prepared_commit");
        if let Err(cause) = adapter.sync_prepared(&mut handle) {
            // A failed/uncertain COMMIT cannot be relabeled as a successful
            // rollback by this helper. The adapter's typed result is retained.
            if adapter.abort(&mut handle).is_err() { return Err(Failure::RecoveryRequired); }
            return Err(cause);
        }
        evidence.synced()?;
        #[cfg(all(windows,feature="beta2-crash-probe"))]
        native::crash_probe::checkpoint("after_prepared");
        #[cfg(test)]
        pause_native_owner_test_process("after_prepared_commit");
        verify_commit_native_from_synced(adapter,payload,expected,&mut evidence,queue_verified)
    })();
    match result {
        Ok(()) => Ok(NativeOwnerOutcomeV1 { inner: NativeOwnerOutcomeKindV1::Committed(evidence) }),
        Err(cause) => {
            // Reread errors after CAS must consume even a successful fresh
            // confirmation. An uncertain SQL COMMIT also requires recovery.
            evidence.container_confirmed = false;
            evidence.state = if evidence.head_changed || cause == Failure::RecoveryRequired {
                State::RecoveryRequired
            } else { State::Aborted };
            let failure = NativeTransactionFailureV1 { state: Some(evidence.state), cause };
            Ok(NativeOwnerOutcomeV1 { inner: NativeOwnerOutcomeKindV1::Failed { evidence, failure } })
        }
    }
}

/// Shared continuation of the same Native loop. A typed source-parent owner
/// may supply its actual durable Prepared proof, never a decoded phase tag.
#[cfg(not(target_arch="wasm32"))]
fn verify_commit_native_from_synced<A:SnapshotV2DurabilityAdapterV1,F>(adapter:&A,payload:&native::StoredPayloadV1,
    expected:Option<&native::StoredHeadV1>,evidence:&mut TransactionEvidenceV1,queue_verified:F)
    ->Result<(),SnapshotV2DurabilityFailureV1>
where F:FnOnce(&TransactionEvidenceV1)->Result<(),SnapshotV2DurabilityFailureV1>{
    use SnapshotV2DurabilityFailureV1 as Failure;
    if evidence.state()!=SnapshotV2TransactionStateV1::DurablyWritten||evidence.binding!=payload.binding{return Err(Failure::CustodyConflict);}
    let reread=adapter.reread_prepared(&payload.digest)?.ok_or(Failure::RereadFailed)?;
    if reread!=*payload{return Err(Failure::RereadMismatch);}
    evidence.verified(&payload.snapshot,&reread.snapshot)?;
    queue_verified(evidence)?;adapter.commit_verified(payload,expected)?;evidence.head_changed()?;
    #[cfg(test)]
    pause_native_owner_test_process("after_head_commit");
    evidence.container_confirmation(adapter.confirm_container_durable())?;
    #[cfg(all(windows,feature="beta2-crash-probe"))]
    native::crash_probe::checkpoint("after_confirmation");
    #[cfg(test)]
    pause_native_owner_test_process("after_container_confirmation");
    let chain=adapter.reread_committed(&payload.digest,&payload.binding.project,&payload.binding.new_revision)?;
    evidence.committed_native_chain(payload,&chain)
}

// Only the child of the exact forced-termination unit test sets these variables.
// No environment switch, extra file, wait or new CLI option exists in release.
#[cfg(all(test, not(target_arch = "wasm32")))]
fn pause_native_owner_test_process(phase: &str) {
    if std::env::var("DDN_NATIVE_OWNER_TEST_PHASE").ok().as_deref() != Some(phase) { return; }
    let Some(marker) = std::env::var_os("DDN_NATIVE_OWNER_TEST_MARKER") else { return; };
    std::fs::write(marker, phase).expect("owned process interruption marker");
    // The parent kills only its own child process, without running Rust Drop.
    // Bound the probe even if the parent itself fails or is interrupted.
    std::thread::sleep(std::time::Duration::from_secs(60));
    panic!("parent did not terminate the owned interruption probe");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub(crate) enum SnapshotV2DurabilityFailureV1 {
    InvalidInput = 1,
    LimitExceeded,
    SnapshotHashMismatch,
    AtomEvidenceInvalid,
    SubjectCustodyInvalid,
    PesiMismatch,
    ForeignPesi,
    StaleRevision,
    CustodyConflict,
    ConflictingRetry,
    UnsupportedDurability,
    WriteFailed,
    ShortWrite,
    SyncFailed,
    RereadFailed,
    RereadMismatch,
    AtomicCommitFailed,
    ContainerDurabilityFailed,
    RevisionCommitFailed,
    AdapterAborted,
    Interrupted,
    CorruptRecoveryRecord,
    RecoveryRequired,
    ArtifactMismatch,
    RevisionEnvelopeRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum SnapshotV2TransactionStateV1 {
    Prepared = 1,
    DurablyWritten,
    Verified,
    Committed,
    Aborted,
    RecoveryRequired,
}

impl SnapshotV2TransactionStateV1 {
    // Graph admission only. Evidence admission is performed separately by the owner.
    fn permits(self, next: Self) -> bool {
        use SnapshotV2TransactionStateV1::*;
        matches!(
            (self, next),
            (Prepared, DurablyWritten | Aborted | RecoveryRequired)
                | (DurablyWritten, Verified | Aborted | RecoveryRequired)
                | (Verified, Committed | Aborted | RecoveryRequired)
                | (Committed, Committed)
                | (Aborted, Aborted)
                | (RecoveryRequired, RecoveryRequired | Aborted | Committed)
        )
    }
}

/// Already supplied constituent bytes. This does not issue any semantic identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TransactionBindingV1 {
    pub project: Vec<u8>,
    pub expected_parent: Vec<u8>,
    pub new_revision: Vec<u8>,
    pub execution_profile: Vec<u8>,
    pub pesi: [u8; 32],
    pub snapshot_sha256: [u8; 32],
    pub snapshot_length: u64,
    pub revision_integrity: [u8; 32],
}

impl TransactionBindingV1 {
    fn preimage(&self) -> Result<Vec<u8>, SnapshotV2DurabilityFailureV1> {
        use SnapshotV2DurabilityFailureV1::*;
        if self.project.is_empty()
            || self.new_revision.is_empty()
            || self.execution_profile.is_empty()
            || self.snapshot_length == 0
        {
            return Err(InvalidInput);
        }
        let length = self.snapshot_length.to_be_bytes();
        let fields: [&[u8]; 8] = [
            &self.project,
            &self.expected_parent,
            &self.new_revision,
            &self.execution_profile,
            &self.pesi,
            &self.snapshot_sha256,
            &length,
            &self.revision_integrity,
        ];
        let mut total = b"DDN-VC04-SNAPSHOT-V2-DURABLE-TRANSACTION-V1\0".len() + 3;
        for field in fields {
            u32::try_from(field.len()).map_err(|_| LimitExceeded)?;
            total = total
                .checked_add(5)
                .and_then(|n| n.checked_add(field.len()))
                .ok_or(LimitExceeded)?;
        }
        let mut out = Vec::new();
        out.try_reserve_exact(total).map_err(|_| LimitExceeded)?;
        out.extend_from_slice(b"DDN-VC04-SNAPSHOT-V2-DURABLE-TRANSACTION-V1\0");
        out.extend_from_slice(&1u16.to_be_bytes());
        out.push(8);
        for (index, field) in fields.into_iter().enumerate() {
            out.push(index as u8 + 1);
            out.extend_from_slice(&(field.len() as u32).to_be_bytes());
            out.extend_from_slice(field);
        }
        Ok(out)
    }

    pub(crate) fn transaction_digest(&self) -> Result<[u8; 32], SnapshotV2DurabilityFailureV1> {
        Ok(Sha256::digest(self.preimage()?).into())
    }
}

/// Common-owner evidence gate. Storage adapters report mechanics; they cannot
/// promote a transaction by returning a generic success flag. This gate alone
/// is not source/custody admission and does not issue a durable receipt.
#[derive(Debug)]
pub(crate) struct TransactionEvidenceV1 {
    binding: TransactionBindingV1,
    state: SnapshotV2TransactionStateV1,
    container_confirmed: bool,
    head_changed: bool,
}

impl TransactionEvidenceV1 {
    pub(crate) fn prepared(binding: TransactionBindingV1) -> Result<Self, SnapshotV2DurabilityFailureV1> {
        binding.transaction_digest()?;
        Ok(Self { binding, state: SnapshotV2TransactionStateV1::Prepared, container_confirmed: false, head_changed: false })
    }

    pub(crate) fn state(&self) -> SnapshotV2TransactionStateV1 { self.state }

    /// Compare the RD670 payload only after actual committed-chain mechanics.
    /// This does not replace source/custody admission or issue a receipt/token.
    pub(crate) fn check_committed_receipt_evidence(&self, payload: &[u8],
        adapter_profile: &[u8]) -> Result<(), SnapshotV2DurabilityFailureV1> {
        self.require(SnapshotV2TransactionStateV1::Committed)?;
        crate::snapshot_v2_host_bridge::check_committed_receipt_payload_v1(payload)?
            .check_binding(&self.binding, adapter_profile)
    }

    fn require(&self, expected: SnapshotV2TransactionStateV1) -> Result<(), SnapshotV2DurabilityFailureV1> {
        if self.state == expected { Ok(()) } else { Err(SnapshotV2DurabilityFailureV1::RecoveryRequired) }
    }

    pub(crate) fn synced(&mut self) -> Result<(), SnapshotV2DurabilityFailureV1> {
        self.require(SnapshotV2TransactionStateV1::Prepared)?;
        self.state = SnapshotV2TransactionStateV1::DurablyWritten;
        Ok(())
    }

    fn check_snapshot(&self, bytes: &[u8]) -> bool {
        bytes.len() as u64 == self.binding.snapshot_length
            && <[u8; 32]>::from(Sha256::digest(bytes)) == self.binding.snapshot_sha256
    }

    /// Called only after the codec, PESI and revision owners have admitted the
    /// reread chain. Exact bytes are compared again here, not host checksums.
    pub(crate) fn verified(&mut self, expected: &[u8], reread: &[u8]) -> Result<(), SnapshotV2DurabilityFailureV1> {
        self.require(SnapshotV2TransactionStateV1::DurablyWritten)?;
        if expected != reread || !self.check_snapshot(expected) {
            self.state = SnapshotV2TransactionStateV1::Aborted;
            return Err(SnapshotV2DurabilityFailureV1::RereadMismatch);
        }
        self.state = SnapshotV2TransactionStateV1::Verified;
        Ok(())
    }

    /// CAS visibility is deliberately NOT Committed. After CAS the common
    /// owner must obtain fresh container durability and reread the full chain.
    pub(crate) fn head_changed(&mut self) -> Result<(), SnapshotV2DurabilityFailureV1> {
        self.require(SnapshotV2TransactionStateV1::Verified)?;
        self.head_changed = true;
        self.container_confirmed = false;
        Ok(())
    }

    pub(crate) fn container_confirmation(&mut self, result: Result<(), SnapshotV2DurabilityFailureV1>) -> Result<(), SnapshotV2DurabilityFailureV1> {
        if !self.head_changed || !matches!(self.state, SnapshotV2TransactionStateV1::Verified | SnapshotV2TransactionStateV1::RecoveryRequired) {
            return Err(SnapshotV2DurabilityFailureV1::RecoveryRequired);
        }
        // A later failed/interrupted confirmation invalidates any earlier one.
        self.container_confirmed = false;
        if let Err(error) = result {
            self.state = SnapshotV2TransactionStateV1::RecoveryRequired;
            return Err(error);
        }
        self.container_confirmed = true;
        Ok(())
    }

    /// Evidence-only final gate; callers must additionally prove the canonical
    /// revision/source/custody chain before publishing a receipt or a world.
    pub(crate) fn committed_chain(&mut self, actual: &TransactionBindingV1, expected: &[u8], reread: &[u8]) -> Result<(), SnapshotV2DurabilityFailureV1> {
        if !self.head_changed || !matches!(self.state, SnapshotV2TransactionStateV1::Verified | SnapshotV2TransactionStateV1::RecoveryRequired) {
            return Err(SnapshotV2DurabilityFailureV1::RecoveryRequired);
        }
        if !self.container_confirmed { return Err(SnapshotV2DurabilityFailureV1::RecoveryRequired); }
        self.container_confirmed = false;
        if actual != &self.binding {
            self.state = SnapshotV2TransactionStateV1::RecoveryRequired;
            return Err(SnapshotV2DurabilityFailureV1::CustodyConflict);
        }
        if expected != reread || !self.check_snapshot(expected) {
            self.state = SnapshotV2TransactionStateV1::RecoveryRequired;
            return Err(SnapshotV2DurabilityFailureV1::ArtifactMismatch);
        }
        self.state = SnapshotV2TransactionStateV1::Committed;
        Ok(())
    }

    /// Common-owner comparison of the complete Native reread, not just a
    /// matching snapshot or two mutually matching SQL heads. `expected` must
    /// remain the original source/custody-admitted payload. This evidence gate
    /// grants no source admission, resume authority or published receipt.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn committed_native_chain(&mut self, expected: &native::StoredPayloadV1,
        reread: &native::RawChainV1) -> Result<(), SnapshotV2DurabilityFailureV1> {
        use SnapshotV2DurabilityFailureV1 as Failure;
        if !self.head_changed || !self.container_confirmed
            || !matches!(self.state, SnapshotV2TransactionStateV1::Verified
                | SnapshotV2TransactionStateV1::RecoveryRequired) {
            return Err(Failure::RecoveryRequired);
        }
        let checked = (|| {
            if expected.binding != self.binding
                || expected.digest != self.binding.transaction_digest()? {
                return Err(Failure::CustodyConflict);
            }
            checked_native_committed_payload(expected,reread)
        })();
        match checked {
            Ok(actual) => self.committed_chain(&actual.binding, &expected.snapshot, &actual.snapshot),
            Err(error) => {
                // Every post-CAS failure consumes the fresh confirmation.
                // Recovery must confirm the same root anew before retrying.
                self.container_confirmed = false;
                self.state = SnapshotV2TransactionStateV1::RecoveryRequired;
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Dependency qualification only, not an RD-669 receipt/crash oracle.
    // SQLite is Native mechanics; browser builds must not depend on this crate.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_sqlite_dependency_strict_blob_and_foreign_key_checks() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection.execute_batch(
            "PRAGMA foreign_keys=ON;
             CREATE TABLE payload (digest BLOB PRIMARY KEY CHECK(length(digest)=32)) STRICT;
             CREATE TABLE revision (digest BLOB NOT NULL REFERENCES payload(digest)) STRICT;",
        ).unwrap();
        let digest = [7u8; 32];
        assert!(connection.execute("INSERT INTO payload VALUES (?1)", [&digest[..31]]).is_err());
        assert!(connection.execute("INSERT INTO payload VALUES (?1)", ["not-a-blob"]).is_err());
        assert!(connection.execute("INSERT INTO revision VALUES (?1)", [&digest[..]]).is_err());
        connection.execute("INSERT INTO payload VALUES (?1)", [&digest[..]]).unwrap();
        connection.execute("INSERT INTO revision VALUES (?1)", [&digest[..]]).unwrap();
        assert!(connection.execute("INSERT INTO payload VALUES (?1)", [&digest[..]]).is_err());
        let count: i64 = connection.query_row("SELECT count(*) FROM payload", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_sqlite_dependency_wal_full_commit_fresh_reread_and_rollback() {
        // Locator only; neither clock nor filename participates in a semantic identity.
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let directory = std::env::temp_dir().join(format!("ddn-sqlite-dependency-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("dependency-probe.sqlite3");
        let mut connection = rusqlite::Connection::open(&path).unwrap();
        let mode: String = connection.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0)).unwrap();
        assert_eq!(mode, "wal");
        connection.execute_batch("PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA read_uncommitted=OFF;
            CREATE TABLE payload (id INTEGER PRIMARY KEY, bytes BLOB NOT NULL) STRICT;").unwrap();
        for (pragma, expected) in [("synchronous", 2), ("foreign_keys", 1), ("read_uncommitted", 0)] {
            let value: i64 = connection.query_row(&format!("PRAGMA {pragma}"), [], |row| row.get(0)).unwrap();
            assert_eq!(value, expected);
        }
        let bytes = b"exact opaque bytes\0\xff";
        let transaction = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).unwrap();
        transaction.execute("INSERT INTO payload VALUES (1, ?1)", [&bytes[..]]).unwrap();
        transaction.commit().unwrap();
        let fresh = rusqlite::Connection::open(&path).unwrap();
        let reread: Vec<u8> = fresh.query_row("SELECT bytes FROM payload WHERE id=1", [], |row| row.get(0)).unwrap();
        assert_eq!(reread, bytes);
        drop(fresh);
        let transaction = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).unwrap();
        transaction.execute("INSERT INTO payload VALUES (2, ?1)", [&bytes[..]]).unwrap();
        transaction.rollback().unwrap();
        let (busy, frames, checkpointed): (i64, i64, i64) = connection.query_row(
            "PRAGMA wal_checkpoint(FULL)", [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
        assert_eq!(busy, 0);
        assert_eq!(frames, checkpointed);
        drop(connection);
        let fresh = rusqlite::Connection::open(&path).unwrap();
        let mode: String = fresh.query_row("PRAGMA journal_mode", [], |row| row.get(0)).unwrap();
        assert_eq!(mode, "wal");
        let count: i64 = fresh.query_row("SELECT count(*) FROM payload", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
        // Leave this exact test directory intact; do not manipulate WAL/SHM files.
    }

    fn hex32(s: &str) -> [u8; 32] {
        std::array::from_fn(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
    }

    fn binding() -> TransactionBindingV1 {
        TransactionBindingV1 {
            project: (0x80..=0x9f).collect(),
            expected_parent: (0xa0..=0xbf).collect(),
            new_revision: (0xc0..=0xdf).collect(),
            execution_profile: b"ddn.execution.profile.test.v1".to_vec(),
            pesi: hex32("9497bcb802baeab1610d3de5502568f892625391141ac3f5265b5522d635fad4"),
            snapshot_sha256: hex32(
                "9438e0e11ead728b23a04328c64deecd16ebf14a6f5b9b63aae73f0f0f00303f",
            ),
            snapshot_length: 614,
            revision_integrity: std::array::from_fn(|i| 0xe0 + i as u8),
        }
    }

    #[test]
    fn rd669_exact_corrected_transaction_vector() {
        let b = binding();
        assert_eq!(b.preimage().unwrap().len(), 316);
        assert_eq!(
            b.transaction_digest().unwrap(),
            hex32("fb6c08f374f5594b78c86e174e0777f488f27ff2a7c9ddc5e75b0e8cab55aed6")
        );
        assert_eq!(b.transaction_digest(), b.transaction_digest());
    }

    #[test]
    fn rd669_revision_changes_transaction_but_not_pesi() {
        let b = binding();
        let mut next = b.clone();
        next.new_revision[0] ^= 1;
        assert_ne!(b.transaction_digest(), next.transaction_digest());
        assert_eq!(b.pesi, next.pesi);
        next.expected_parent.clear(); // Only INITIAL parent may be empty.
        assert!(next.transaction_digest().is_ok());
        next.project.clear();
        assert_eq!(
            next.transaction_digest(),
            Err(SnapshotV2DurabilityFailureV1::InvalidInput)
        );
    }

    #[test]
    fn rd669_exact_state_graph_has_no_recovery_to_verified() {
        use SnapshotV2TransactionStateV1::*;
        let states = [
            Prepared,
            DurablyWritten,
            Verified,
            Committed,
            Aborted,
            RecoveryRequired,
        ];
        let allowed = [
            vec![1, 4, 5],
            vec![2, 4, 5],
            vec![3, 4, 5],
            vec![3],
            vec![4],
            vec![3, 4, 5],
        ];
        for (i, from) in states.iter().enumerate() {
            for (j, to) in states.iter().enumerate() {
                assert_eq!(
                    from.permits(*to),
                    allowed[i].contains(&j),
                    "{from:?} -> {to:?}"
                );
            }
        }
        assert_eq!(SnapshotV2DurabilityFailureV1::ArtifactMismatch as u16, 24);
        assert_eq!(
            SnapshotV2DurabilityFailureV1::RevisionEnvelopeRequired as u16,
            25
        );
    }

    fn evidence(bytes: &[u8]) -> TransactionEvidenceV1 {
        let mut b = binding();
        b.snapshot_length = bytes.len() as u64;
        b.snapshot_sha256 = Sha256::digest(bytes).into();
        TransactionEvidenceV1::prepared(b).unwrap()
    }

    #[test]
    fn rd669_cas_and_generic_success_are_not_committed() {
        let bytes = b"snapshot";
        let mut owner = evidence(bytes);
        assert_eq!(owner.head_changed(), Err(SnapshotV2DurabilityFailureV1::RecoveryRequired));
        owner.synced().unwrap();
        owner.verified(bytes, bytes).unwrap();
        owner.head_changed().unwrap();
        let b = owner.binding.clone();
        assert_eq!(owner.committed_chain(&b, bytes, bytes), Err(SnapshotV2DurabilityFailureV1::RecoveryRequired));
        owner.container_confirmation(Err(SnapshotV2DurabilityFailureV1::UnsupportedDurability)).unwrap_err();
        assert_eq!(owner.state(), SnapshotV2TransactionStateV1::RecoveryRequired);
        owner.container_confirmation(Ok(())).unwrap();
        owner.committed_chain(&b, bytes, bytes).unwrap();
        assert_eq!(owner.state(), SnapshotV2TransactionStateV1::Committed);
    }

    #[test]
    fn rd669_post_cas_corruption_requires_new_confirmation() {
        let bytes = b"snapshot";
        let mut owner = evidence(bytes);
        owner.synced().unwrap();
        owner.verified(bytes, bytes).unwrap();
        owner.head_changed().unwrap();
        owner.container_confirmation(Ok(())).unwrap();
        let b = owner.binding.clone();
        assert_eq!(owner.committed_chain(&b, bytes, b"damaged!"), Err(SnapshotV2DurabilityFailureV1::ArtifactMismatch));
        assert_eq!(owner.state(), SnapshotV2TransactionStateV1::RecoveryRequired);
        assert_eq!(owner.committed_chain(&b, bytes, bytes), Err(SnapshotV2DurabilityFailureV1::RecoveryRequired));
        owner.container_confirmation(Ok(())).unwrap();
        owner.container_confirmation(Err(SnapshotV2DurabilityFailureV1::Interrupted)).unwrap_err();
        assert_eq!(owner.committed_chain(&b, bytes, bytes), Err(SnapshotV2DurabilityFailureV1::RecoveryRequired));
    }

    #[test]
    fn rd669_pre_cas_corruption_aborts_without_changing_head() {
        let bytes = b"snapshot";
        let mut owner = evidence(bytes);
        owner.synced().unwrap();
        assert_eq!(owner.verified(bytes, b"damaged!"), Err(SnapshotV2DurabilityFailureV1::RereadMismatch));
        assert_eq!(owner.state(), SnapshotV2TransactionStateV1::Aborted);
        assert!(owner.head_changed().is_err());
    }
}
