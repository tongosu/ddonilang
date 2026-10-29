//! Private R4 host grant. Neither its locator nor these decoded bytes grant
//! source, transaction or recovery authority. Pinning/caller admission is a
//! separate producer; no public constructor or persisted semantic ID exists.
use super::Failure;
mod first_use;
pub(super) mod source_transition;

const MAGIC: &[u8] = b"DDN-NATIVE-ROOT-GRANT-V1\0";
const LENGTHS: [usize; 10] = [32, 32, 32, 32, 24, 16, 24, 16, 1, 8];
const PLAINTEXT_BYTES: usize = 295;
const MAX_CIPHERTEXT_BYTES: usize = 16_384;
const SLOT_MAGIC: &[u8] = b"DDN-NATIVE-ROOT-SLOT-V1\0";
const ATTEMPT_MAGIC: &[u8] = b"DDN-NATIVE-RECOVERY-ATTEMPT-V1\0";
const ATTEMPT_LENGTHS: [usize;8] = [32,8,8,32,32,32,1,32];

/// R5.1 private host-store bytes, not a transaction state or receipt.
/// Decoding this record never issues a start/witness/recovery capability.
#[derive(Clone,Debug,PartialEq,Eq)]
struct AttemptRecord {
    nonce: [u8;32],
    generation: u64,
    epoch: u64,
    transaction: [u8;32],
    request_hash: [u8;32],
    token_hash: [u8;32],
    mode: AttemptMode,
    record_hash: [u8;32],
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum AttemptMode { Pending, RecordWitness }

impl AttemptRecord {
    fn validate(&self) -> Result<(),Failure> {
        if self.nonce==[0;32] || self.epoch==0
            || (self.mode==AttemptMode::Pending)!=(self.record_hash==[0;32]) {
            return Err(Failure::CorruptRecoveryRecord);
        }
        Ok(())
    }

    fn encode(&self) -> Result<Vec<u8>,Failure> {
        self.validate()?;
        let generation=self.generation.to_be_bytes();let epoch=self.epoch.to_be_bytes();
        let mode=[match self.mode {AttemptMode::Pending=>0,AttemptMode::RecordWitness=>1}];
        let fields:[&[u8];8]=[&self.nonce,&generation,&epoch,&self.transaction,&self.request_hash,&self.token_hash,&mode,&self.record_hash];
        let mut bytes=Vec::new();bytes.try_reserve_exact(251).map_err(|_|Failure::LimitExceeded)?;
        bytes.extend(ATTEMPT_MAGIC);bytes.extend([0,1,8]);
        for (i,field) in fields.into_iter().enumerate() {bytes.push(i as u8+1);bytes.extend((field.len() as u32).to_be_bytes());bytes.extend(field);}
        Ok(bytes)
    }

    fn decode(bytes:&[u8]) -> Result<Self,Failure> {
        if bytes.len()!=251 || !bytes.starts_with(ATTEMPT_MAGIC)
            || bytes.get(ATTEMPT_MAGIC.len()..ATTEMPT_MAGIC.len()+3)!=Some(&[0,1,8]) {
            return Err(Failure::CorruptRecoveryRecord);
        }
        let mut cursor=ATTEMPT_MAGIC.len()+3;
        let mut fields:[&[u8];8]=[&[];8];
        for (i,length) in ATTEMPT_LENGTHS.into_iter().enumerate() {
            if bytes.get(cursor)!=Some(&(i as u8+1)) {return Err(Failure::CorruptRecoveryRecord);}
            let raw=bytes.get(cursor+1..cursor+5).ok_or(Failure::CorruptRecoveryRecord)?;
            if u32::from_be_bytes(raw.try_into().map_err(|_|Failure::CorruptRecoveryRecord)?) as usize!=length {
                return Err(Failure::CorruptRecoveryRecord);
            }
            cursor+=5;fields[i]=bytes.get(cursor..cursor+length).ok_or(Failure::CorruptRecoveryRecord)?;cursor+=length;
        }
        if cursor!=bytes.len() {return Err(Failure::CorruptRecoveryRecord);}
        let value=Self {nonce:fields[0].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?,
            generation:u64::from_be_bytes(fields[1].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?),
            epoch:u64::from_be_bytes(fields[2].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?),
            transaction:fields[3].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?,
            request_hash:fields[4].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?,
            token_hash:fields[5].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?,
            mode:match fields[6] {[0]=>AttemptMode::Pending,[1]=>AttemptMode::RecordWitness,_=>return Err(Failure::CorruptRecoveryRecord)},
            record_hash:fields[7].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?};
        value.validate()?;Ok(value)
    }

    /// This computes bytes only. The caller still needs actual checked request,
    /// root/source admission and durable Set/Flush/fresh read before starting.
    fn initial_pending(grant:&GrantRecord,transaction:[u8;32],exact_request:&[u8],exact_token:&[u8]) -> Result<Self,Failure> {
        use sha2::{Digest,Sha256};
        if grant.lifecycle!=Lifecycle::Active || exact_request.is_empty() || exact_request.len()>3_016_737
            || exact_token.len()!=33 || exact_token[0]!=1 || exact_token[1..]!=transaction {
            return Err(Failure::CustodyConflict);
        }
        let value=Self {nonce:grant.nonce,generation:grant.generation,epoch:1,transaction,
            request_hash:Sha256::digest(exact_request).into(),token_hash:Sha256::digest(exact_token).into(),
            mode:AttemptMode::Pending,record_hash:[0;32]};
        value.validate()?;Ok(value)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct GrantRecord {
    nonce: [u8; 32],
    sid_hash: [u8; 32],
    project_hash: [u8; 32],
    registration_pesi: [u8; 32],
    root_file: [u8; 24],
    root_object: [u8; 16],
    db_file: [u8; 24],
    db_object: [u8; 16],
    lifecycle: Lifecycle,
    generation: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lifecycle { Active, Revoked }

#[cfg(windows)]
pub(super) struct PinnedNativeRootV1 {
    root: std::fs::File,
    db: std::fs::File,
    root_file: [u8; 24],
    root_object: [u8; 16],
    db_file: [u8; 24],
    db_object: [u8; 16],
}

/// Issued only by the successful create_new/schema/capability branch of the
/// native owner. Its open objects prevent transferring proof via a path/BOOL.
#[cfg(windows)]
pub(super) struct FreshNativeContainerV1 {
    root: std::fs::File,
    db: std::fs::File,
    root_file: [u8; 24],
    db_file: [u8; 24],
}

fn encode_slot(nonce: &[u8; 32]) -> Result<Vec<u8>, Failure> {
    if nonce == &[0; 32] { return Err(Failure::InvalidInput); }
    let mut bytes = Vec::with_capacity(58);
    bytes.extend(SLOT_MAGIC); bytes.extend([0, 1]); bytes.extend(nonce);
    Ok(bytes)
}
fn decode_slot(bytes: &[u8]) -> Result<[u8; 32], Failure> {
    if bytes.len() != 58 || !bytes.starts_with(SLOT_MAGIC) || bytes.get(SLOT_MAGIC.len()..SLOT_MAGIC.len()+2) != Some(&[0,1]) {
        return Err(Failure::CorruptRecoveryRecord);
    }
    let nonce: [u8; 32] = bytes[SLOT_MAGIC.len()+2..].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?;
    if nonce == [0; 32] { return Err(Failure::CorruptRecoveryRecord); }
    Ok(nonce)
}

fn root_fingerprint(record: &GrantRecord) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let fields: [(u8, &[u8]); 6] = [(2,&record.sid_hash),(3,&record.project_hash),(5,&record.root_file),
        (6,&record.root_object),(7,&record.db_file),(8,&record.db_object)];
    let mut hash = Sha256::new();
    for (tag, value) in fields { hash.update([tag]); hash.update((value.len() as u32).to_be_bytes()); hash.update(value); }
    hash.finalize().into()
}

/// An in-process root grant, not a durable transaction or restore receipt.
/// Only the Windows producer below may construct it from a source-checked
/// caller and actual leased/pinned objects. Restart always requires reopening.
#[cfg(windows)]
pub(crate) struct AdmittedRootGrantV1 {
    record: GrantRecord,
    pinned: std::sync::Arc<PinnedNativeRootV1>,
    lease: std::sync::Arc<std::fs::File>,
}

#[cfg(windows)]
pub(crate) struct AdmittedSourceGrantV2 {
    record:first_use::Grant2,
    pinned:std::sync::Arc<PinnedNativeRootV1>,
    lease:std::sync::Arc<std::fs::File>,
}

/// Child evidence scope, not a new first-use/restore/world capability. Its
/// owned pin/lease permits the real world transaction to consume its permit.
#[cfg(windows)]
pub(crate) struct ReplayArchiveRootScopeV1 {
    record:GrantRecord,
    pinned:std::sync::Arc<PinnedNativeRootV1>,
    lease:std::sync::Arc<std::fs::File>,
}
/// Actual protected stage1, not SQL data or a cold recovery permit.
/// Constructed only by the existing Windows grant owner after flush/fresh.
#[cfg(windows)]
pub(crate) struct FinishFailureCaptureWitnessV1 {
    plain:Vec<u8>,intent_digest:[u8;32],failure_digest:[u8;32],
}
#[cfg(windows)]
pub(crate) struct DurableFinishIntentWitnessV1 {plain:Vec<u8>,intent_digest:[u8;32]}
/// One explicit cold query/repair, from actual current source/world admission.
/// No Clone/Deserialize/default/raw-ID constructor; does not recreate a VM.
#[cfg(windows)]
pub(crate) struct NativeFinishRecoveryPermitV1 {
    proof:VerifiedExecutionFirstCheckpointV1,
    binding:crate::logical_input_tape::LogicalInputBinding,
    checked:windows::CheckedColdFinishV1,
}
#[cfg(windows)]
pub(crate) struct RecoveredNativeFinishV1 {
    c01:Vec<u8>,notice:serde_json::Value,original_cause:Option<String>,
}
#[cfg(all(test,windows))]
pub(crate) fn test_arm_finish_ready_set_failure(){windows::test_arm_finish_ready_set_failure();}
#[cfg(all(test,windows))]
pub(crate) fn test_finish_lifecycle_cipher_digest(nonce:&[u8;32],first:&[u8;32])->Result<[u8;32],Failure>{
    windows::test_finish_lifecycle_cipher_digest(nonce,first)
}
#[cfg(windows)]
impl RecoveredNativeFinishV1 {
    pub(crate) fn into_parts(self)->(Vec<u8>,serde_json::Value,Option<String>){(self.c01,self.notice,self.original_cause)}
}
#[cfg(windows)]
impl Drop for FinishFailureCaptureWitnessV1 {
    fn drop(&mut self){self.plain.fill(0);self.intent_digest.fill(0);self.failure_digest.fill(0);}
}
/// Untrusted selection data; admission below compares every field to the
/// actual protected root and same SQLite head. Not a grant or receipt.
#[cfg(windows)]
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct AuthoringHeadV1 {
    pub project:Vec<u8>,pub revision:Vec<u8>,pub integrity:[u8;32],pub key:[u8;32],pub kind:u8,
}
#[cfg(windows)]
pub(crate) struct AdmittedAuthoringBaseV1 {
    grant:AdmittedSourceGrantV2,attempt:AttemptRecord,transition:Option<source_transition::Record>,
    head:AuthoringHeadV1,source:crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision,
    history:super::source_world::CheckedMixedRevisionHistory,
}
#[cfg(windows)]
pub(crate) struct PreparedAuthoringSourceV1 {
    base:AdmittedAuthoringBaseV1,source:crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision,
    prepared:source_transition::Record,
}
#[cfg(windows)]
pub(crate) struct DurablySavedAuthoringSourceV1 {
    grant:AdmittedSourceGrantV2,attempt:AttemptRecord,ready:source_transition::Record,
    source:crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision,
}
#[cfg(windows)]
pub(crate) struct NewSourceRunPermitV1 {
    grant:AdmittedSourceGrantV2,anchor_attempt:AttemptRecord,consumed:source_transition::Record,
    source:crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision,
}
#[cfg(windows)]
pub(crate) struct RestartSourceWorldAttemptV1 {
    permit:NewSourceRunPermitV1,attempt:AttemptRecord,
    request:crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1,
}
#[cfg(not(windows))]
pub(crate) enum AdmittedSourceGrantV2 {}
#[cfg(windows)]
pub(crate) struct FirstSourceWorldPermitV2 {
    grant:AdmittedSourceGrantV2,
    source:crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision,
}
#[cfg(windows)]
pub(crate) struct SourceWorldAttemptStartV2 {
    permit:FirstSourceWorldPermitV2,
    attempt:AttemptRecord,
    request:crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1,
}
/// A distinct stateful transaction in an already-created source world. It
/// owns the same Grant2 and a freshly flushed R5.1 epoch, not another first-use.
#[cfg(windows)]
pub(crate) struct NextSourceWorldAttemptStartV1 {
    grant:AdmittedSourceGrantV2,
    source:crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision,
    attempt:AttemptRecord,
    request:crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1,
    previous:super::StoredHeadV1,
    restart:Option<source_transition::Record>,
}
/// Fresh cold admission of an existing first-world witness, not first-use or
/// retry authority. Its source grant remains leased/pinned for the consumer.
#[cfg(windows)]
pub(crate) struct ReadmittedFirstSourceWorldV2 {
    _grant:AdmittedSourceGrantV2,
    _attempt:AttemptRecord,
    payload:super::StoredPayloadV1,
    history:Vec<[u8;32]>,
    restart:Option<RestartSourceHistoryV1>,
}
/// Issued only after the actual cold source/world owner has revalidated its
/// full current history. This child proof carries no Run/save capability.
#[cfg(windows)]
pub(crate) struct VerifiedExecutionFirstCheckpointV1 {
    pub(in crate::snapshot_v2_durable_transaction::native) scope:ReplayArchiveRootScopeV1,
    pub(in crate::snapshot_v2_durable_transaction::native) first_tx:[u8;32],
    pub(in crate::snapshot_v2_durable_transaction::native) checkpoint_tx:[u8;32],
    pub(in crate::snapshot_v2_durable_transaction::native) receipt:Vec<u8>,
    pub(in crate::snapshot_v2_durable_transaction::native) pesi:[u8;32],
    checkpoint_next_madi:u64,
}
#[cfg(windows)]
#[derive(Clone,Copy)]
pub(crate) enum ReplayArchiveReadKindV1 {Running,Finished,TerminalForNewRecord}
#[cfg(windows)]
impl ReplayArchiveReadKindV1 {
    pub(in crate::snapshot_v2_durable_transaction::native) fn accepts_state(self,state:u8)->bool{
        match self{Self::Running=>state==0,Self::Finished=>state==1,Self::TerminalForNewRecord=>(1..=3).contains(&state)}
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn accepts_end(self,state:u8,kind:u8)->bool{
        self.accepts_state(state)&&match state{0=>kind==2,1=>kind<=1,2|3=>kind==2,_=>false}
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn accepts_count(self,state:u8,count:u64,anchor:u64)->bool{
        self.accepts_state(state)&&if state<=1{count==anchor}else{count>=anchor}
    }
}
/// Physical write/flush/fresh-read proof, not a decoded Ready candidate.
#[cfg(windows)]
pub(crate) struct ProtectedReplayReadyV1 {scope:ReplayArchiveRootScopeV1,bytes:Vec<u8>,first_tx:[u8;32]}
#[cfg(windows)]
pub(crate) enum ReplayLifecycleWriteFailureV1 {Before(Failure),Uncertain(Failure)}
#[cfg(windows)]
impl ReplayLifecycleWriteFailureV1 {
    fn cause(self)->Failure{match self{Self::Before(cause)|Self::Uncertain(cause)=>cause}}
}
#[cfg(windows)]
struct RestartSourceHistoryV1 {
    source:crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision,
    record:source_transition::Record,
}
#[cfg(windows)]
#[derive(Debug)]
pub(crate) struct SourceGrantFailureV2 {pub(crate) phase:&'static str,pub(crate) cause:Failure}

/// Issued only after an actual producer and admitted root have durably written
/// and freshly re-read the Pending anchor. Not Clone/deserialize/receipt.
#[cfg(windows)]
pub(crate) struct DurableAttemptStartV1 {
    grant:AdmittedRootGrantV1,
    attempt:AttemptRecord,
    request:crate::snapshot_v2_durable_transaction::retained_attempt::ActualPreparedRequestV1,
}

/// Fresh host-store acknowledgement, not a decoded record/receipt. Only the
/// same live Pending owner consuming an SQL-retained actual outcome issues it.
#[cfg(windows)]
pub(crate) struct RetainedAttemptWitnessV1 { _attempt:AttemptRecord }

/// Root/source/Attempt and the complete INITIAL custody prefix were checked
/// in this process. Historical state/cause only: no current Committed proof,
/// successful world restore, next-epoch start or receipt is issued here.
#[cfg(windows)]
pub(crate) struct AdmittedInitialAttemptV1 {
    _grant:AdmittedRootGrantV1,
    _attempt:AttemptRecord,
    _expected:crate::snapshot_v2_durable_transaction::retained_attempt::ExplicitRecoveryRequestV1,
    record:crate::snapshot_v2_durable_transaction::recovery_custody::LoadedInitialCustodyV1,
}

/// A new durable Pending epoch carrying its immutable previous Witness and
/// exact admitted record. Neither bytes nor an ordinary DB open construct it.
#[cfg(windows)]
pub(crate) struct RecoveryAttemptStartV1 {
    prior:AdmittedInitialAttemptV1,
    pending:AttemptRecord,
}

#[cfg(windows)]
impl RecoveryAttemptStartV1 {
    pub(in crate::snapshot_v2_durable_transaction) fn expected_payload(&self)->&super::StoredPayloadV1 {self.prior._expected.payload()}
    pub(in crate::snapshot_v2_durable_transaction) fn expected_raw(&self)->&[u8] {self.prior._expected.raw()}
    pub(in crate::snapshot_v2_durable_transaction) fn expected_request(&self)->&crate::snapshot_v2_durable_transaction::retained_attempt::ExplicitRecoveryRequestV1 {&self.prior._expected}
    pub(in crate::snapshot_v2_durable_transaction) fn prior_record(&self)->&crate::snapshot_v2_durable_transaction::recovery_custody::LoadedInitialCustodyV1 {&self.prior.record}
    pub(in crate::snapshot_v2_durable_transaction) fn prior_hash(&self)->[u8;32] {self.prior._attempt.record_hash}
    pub(in crate::snapshot_v2_durable_transaction) fn prior_epoch(&self)->u64 {self.prior._attempt.epoch}
    pub(in crate::snapshot_v2_durable_transaction) fn epoch(&self)->u64 {self.pending.epoch}
}

#[cfg(windows)]
impl AdmittedInitialAttemptV1 {
    pub(crate) fn historical_state(&self)->crate::snapshot_v2_durable_transaction::SnapshotV2TransactionStateV1 {self.record.state()}
    pub(crate) fn original_cause(&self)->Failure {self.record.cause()}
    pub(in crate::snapshot_v2_durable_transaction) fn expected_payload(&self)->&super::StoredPayloadV1 {self._expected.payload()}
}

#[cfg(windows)]
struct RootGrantCandidateV1 {
    record: GrantRecord,
    pinned: std::sync::Arc<PinnedNativeRootV1>,
    lease: std::sync::Arc<std::fs::File>,
}

impl GrantRecord {
    fn decode(bytes: &[u8]) -> Result<Self, Failure> {
        if bytes.len() != PLAINTEXT_BYTES || !bytes.starts_with(MAGIC) {
            return Err(Failure::CorruptRecoveryRecord);
        }
        let mut cursor = MAGIC.len();
        if bytes.get(cursor..cursor + 3) != Some(&[0, 1, 10]) {
            return Err(Failure::CorruptRecoveryRecord);
        }
        cursor += 3;
        let mut fields: [&[u8]; 10] = [&[]; 10];
        for (index, expected_length) in LENGTHS.into_iter().enumerate() {
            if bytes.get(cursor) != Some(&(index as u8 + 1)) {
                return Err(Failure::CorruptRecoveryRecord);
            }
            let encoded_length = bytes.get(cursor + 1..cursor + 5).ok_or(Failure::CorruptRecoveryRecord)?;
            let length = u32::from_be_bytes(encoded_length.try_into().map_err(|_| Failure::CorruptRecoveryRecord)?) as usize;
            if length != expected_length { return Err(Failure::CorruptRecoveryRecord); }
            cursor += 5;
            fields[index] = bytes.get(cursor..cursor + length).ok_or(Failure::CorruptRecoveryRecord)?;
            cursor += length;
        }
        if cursor != bytes.len() || fields[0].iter().all(|byte| *byte == 0) {
            return Err(Failure::CorruptRecoveryRecord);
        }
        let lifecycle = match fields[8] {
            [1] => Lifecycle::Active,
            [2] => Lifecycle::Revoked,
            _ => return Err(Failure::CorruptRecoveryRecord),
        };
        // Field lengths were checked before any fixed-array copy.
        Ok(Self {
            nonce: fields[0].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?,
            sid_hash: fields[1].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?,
            project_hash: fields[2].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?,
            registration_pesi: fields[3].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?,
            root_file: fields[4].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?,
            root_object: fields[5].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?,
            db_file: fields[6].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?,
            db_object: fields[7].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?,
            lifecycle,
            generation: u64::from_be_bytes(fields[9].try_into().map_err(|_| Failure::CorruptRecoveryRecord)?),
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Failure> {
        let lifecycle = [match self.lifecycle { Lifecycle::Active => 1, Lifecycle::Revoked => 2 }];
        let generation = self.generation.to_be_bytes();
        let fields: [&[u8]; 10] = [&self.nonce, &self.sid_hash, &self.project_hash, &self.registration_pesi,
            &self.root_file, &self.root_object, &self.db_file, &self.db_object, &lifecycle, &generation];
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(PLAINTEXT_BYTES).map_err(|_| Failure::LimitExceeded)?;
        bytes.extend(MAGIC); bytes.extend([0, 1, 10]);
        for (index, field) in fields.into_iter().enumerate() {
            bytes.push(index as u8 + 1); bytes.extend((field.len() as u32).to_be_bytes()); bytes.extend(field);
        }
        Self::decode(&bytes)?;
        Ok(bytes)
    }

    fn revoked(&self) -> Result<Self, Failure> {
        if self.lifecycle != Lifecycle::Active { return Err(Failure::CustodyConflict); }
        let mut next = self.clone();
        next.generation = self.generation.checked_add(1).ok_or(Failure::LimitExceeded)?;
        next.lifecycle = Lifecycle::Revoked;
        Ok(next)
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};
    include!("root_grant/source_first_use_windows.rs");
    include!("root_grant/source_authoring_windows.rs");
    include!("root_grant/replay_windows.rs");
    include!("root_grant/finish_recovery_windows.rs");
    include!("root_grant/finish_recovery_cold_windows.rs");

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Namespace { Grant, RootSlot, Attempt, SourceTransition }
    #[cfg(test)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum IoStep { Set, Flush, FreshRead }
    #[cfg(test)]
    std::thread_local! { static IO_FAULT: std::cell::Cell<Option<(Namespace,IoStep)>> = const { std::cell::Cell::new(None) }; }
    #[cfg(test)]
    std::thread_local! { static LAST_REGISTRATION_NONCE: std::cell::Cell<Option<[u8;32]>> = const { std::cell::Cell::new(None) }; }
    #[cfg(test)]
    fn inject_io_failure(namespace: Namespace, step: IoStep) -> Result<(), Failure> {
        IO_FAULT.with(|point| {
            if point.get() != Some((namespace,step)) { return Ok(()); }
            point.set(None);
            Err(match step { IoStep::Set=>Failure::WriteFailed, IoStep::Flush=>Failure::SyncFailed, IoStep::FreshRead=>Failure::RereadFailed })
        })
    }
    impl Namespace {
        fn prefix(&self) -> &'static str { match self {
            Self::Grant | Self::Attempt | Self::SourceTransition => "Software\\DDonilang\\NativeRecoveryGrantV1\\",
            Self::RootSlot => "Software\\DDonilang\\NativeRecoveryRootSlotV1\\",
        } }
        fn value_name(self) -> &'static str { match self {Self::Attempt=>"Attempt",Self::SourceTransition=>"SourceTransition",_=>"Record"} }
    }
    type Handle = *mut c_void;
    #[repr(C)]
    struct Blob { length: u32, data: *mut u8 }
    #[link(name = "crypt32")]
    unsafe extern "system" {
        fn CryptProtectData(input: *const Blob, description: *const u16, entropy: *const Blob,
            reserved: *mut c_void, prompt: *mut c_void, flags: u32, output: *mut Blob) -> i32;
        fn CryptUnprotectData(input: *const Blob, description: *mut *mut u16, entropy: *const Blob,
            reserved: *mut c_void, prompt: *mut c_void, flags: u32, output: *mut Blob) -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LocalFree(memory: Handle) -> Handle;
        fn GetFileInformationByHandleEx(file: Handle, class: i32, information: *mut c_void, length: u32) -> i32;
        fn GetVolumeInformationByHandleW(file: Handle, volume_name: *mut u16, name_length: u32,
            serial: *mut u32, component_length: *mut u32, flags: *mut u32, filesystem: *mut u16, filesystem_length: u32) -> i32;
        fn DeviceIoControl(file: Handle, code: u32, input: *const c_void, input_length: u32,
            output: *mut c_void, output_length: u32, returned: *mut u32, overlapped: *mut c_void) -> i32;
        fn GetCurrentProcess() -> Handle;
        fn CloseHandle(handle: Handle) -> i32;
    }
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegCreateKeyExW(root: Handle, subkey: *const u16, reserved: u32, class: *mut u16,
            options: u32, access: u32, security: *const c_void, key: *mut Handle, disposition: *mut u32) -> i32;
        fn RegOpenKeyExW(root: Handle, subkey: *const u16, options: u32, access: u32, key: *mut Handle) -> i32;
        fn RegSetValueExW(key: Handle, name: *const u16, reserved: u32, kind: u32, data: *const u8, length: u32) -> i32;
        fn RegQueryValueExW(key: Handle, name: *const u16, reserved: *mut u32, kind: *mut u32, data: *mut u8, length: *mut u32) -> i32;
        fn RegFlushKey(key: Handle) -> i32;
        fn RegCloseKey(key: Handle) -> i32;
        fn OpenProcessToken(process: Handle, access: u32, token: *mut Handle) -> i32;
        fn GetTokenInformation(token: Handle, class: i32, information: *mut c_void, length: u32, needed: *mut u32) -> i32;
        fn IsValidSid(sid: *const c_void) -> i32;
        fn GetLengthSid(sid: *const c_void) -> u32;
    }

    struct Key(Handle);
    impl Drop for Key {
        fn drop(&mut self) {
            // SAFETY: only a successful registry open/create constructs Key.
            unsafe { RegCloseKey(self.0); }
        }
    }
    struct ProtectedOutput(Blob);
    impl Drop for ProtectedOutput {
        fn drop(&mut self) {
            if !self.0.data.is_null() {
                // SAFETY: DPAPI owns this writable LocalAlloc allocation of
                // exactly length bytes until LocalFree. Clear decrypted data.
                unsafe {
                    for offset in 0..self.0.length as usize { self.0.data.add(offset).write_volatile(0); }
                    LocalFree(self.0.data.cast());
                }
            }
        }
    }
    fn utf16(value: &str) -> Vec<u16> { value.encode_utf16().chain(Some(0)).collect() }
    fn hkcu() -> Handle { (0x8000_0001u32 as i32 as isize) as Handle }

    fn current_sid_hash() -> Result<[u8; 32], Failure> {
        use sha2::{Digest, Sha256};
        struct Token(Handle);
        impl Drop for Token { fn drop(&mut self) { unsafe { CloseHandle(self.0); } } }
        let mut token = null_mut();
        // The process pseudohandle is borrowed; only its new token is closed.
        if unsafe { OpenProcessToken(GetCurrentProcess(), 8, &mut token) } == 0 || token.is_null() {
            return Err(Failure::UnsupportedDurability);
        }
        let token = Token(token);
        let mut needed = 0;
        unsafe { GetTokenInformation(token.0, 1, null_mut(), 0, &mut needed); }
        if needed < std::mem::size_of::<Handle>() as u32 || needed > 65_536 { return Err(Failure::LimitExceeded); }
        // usize words retain TOKEN_USER pointer alignment, unlike Vec<u8>.
        let words = (needed as usize).div_ceil(std::mem::size_of::<usize>());
        let mut information = Vec::new();
        information.try_reserve_exact(words).map_err(|_| Failure::LimitExceeded)?;
        information.resize(words, 0usize);
        let capacity = information.len() * std::mem::size_of::<usize>();
        let mut actual = needed;
        if unsafe { GetTokenInformation(token.0, 1, information.as_mut_ptr().cast(), needed, &mut actual) } == 0 || actual > needed {
            return Err(Failure::CustodyConflict);
        }
        let start = information.as_ptr() as usize;
        let sid = information[0] as Handle;
        let offset = (sid as usize).checked_sub(start).ok_or(Failure::CustodyConflict)?;
        if offset.checked_add(8).filter(|end| *end <= capacity).is_none() { return Err(Failure::CustodyConflict); }
        if unsafe { IsValidSid(sid) } == 0 { return Err(Failure::CustodyConflict); }
        let length = unsafe { GetLengthSid(sid) } as usize;
        if !(8..=68).contains(&length) || offset.checked_add(length).filter(|end| *end <= capacity).is_none() {
            return Err(Failure::CustodyConflict);
        }
        // The token API's actual binary SID, not a display name/environment.
        Ok(Sha256::digest(unsafe { std::slice::from_raw_parts(sid.cast::<u8>(), length) }).into())
    }

    fn source_project_hash(source: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>) -> Result<[u8; 32], Failure> {
        use sha2::{Digest, Sha256};
        // Same admitted owner/bytes as check_saved_revision_context. This is
        // a private comparison digest, not a new project identity issuer.
        let bytes = hex::decode(source.owned().project().authored_identity.logical_dependency_id.as_str())
            .map_err(|_| Failure::CustodyConflict)?;
        if bytes.is_empty() || bytes.len() > 4096 { return Err(Failure::CustodyConflict); }
        Ok(Sha256::digest(&bytes).into())
    }

    #[repr(C)]
    struct FileIdInfo { volume_serial: u64, file_id: [u8; 16] }

    fn file_identity(file: Handle) -> Result<[u8; 24], Failure> {
        let mut information = FileIdInfo { volume_serial: 0, file_id: [0; 16] };
        // Cached Windows SDK bindings: FileIdInfo=18, CREATE_OR_GET_OBJECT_ID
        // =590016, GET_OBJECT_ID=589980. The OS handle remains owned by caller.
        if unsafe { GetFileInformationByHandleEx(file, 18, (&mut information as *mut FileIdInfo).cast(), 24) } == 0 {
            return Err(Failure::UnsupportedDurability);
        }
        let mut identity = [0u8; 24];
        identity[..8].copy_from_slice(&information.volume_serial.to_le_bytes());
        identity[8..].copy_from_slice(&information.file_id);
        Ok(identity)
    }

    fn physical_identity(file: Handle, registration: bool) -> Result<([u8; 24], [u8; 16]), Failure> {
        let identity = file_identity(file)?;
        let mut buffer = [0u8; 64];
        let mut returned = 0;
        let code = if registration { 590016 } else { 589980 };
        if unsafe { DeviceIoControl(file, code, null(), 0, buffer.as_mut_ptr().cast(), 64, &mut returned, null_mut()) } == 0 || returned != 64 {
            return Err(Failure::CustodyConflict);
        }
        let object: [u8; 16] = buffer[..16].try_into().map_err(|_| Failure::CustodyConflict)?;
        if object == [0; 16] { return Err(Failure::CustodyConflict); }
        Ok((identity, object))
    }

    fn open_pinned(path: &std::path::Path) -> Result<std::fs::File, Failure> {
        use std::os::windows::fs::OpenOptionsExt;
        super::super::reject_reparse(path)?;
        std::fs::OpenOptions::new().read(true).share_mode(3).custom_flags(0x02200000).open(path)
            .map_err(|_| Failure::UnsupportedDurability)
    }

    impl FreshNativeContainerV1 {
        /// Mechanical same-object check before source registration entropy.
        /// No ObjectId creation, host-store value, Grant or first-use proof.
        pub(in crate::snapshot_v2_durable_transaction::native) fn verify_source_connection(
            &self,path:&std::path::Path,connection:&rusqlite::Connection,
        )->Result<(),Failure> {
            use std::os::windows::io::AsRawHandle;
            let root=open_pinned(path.parent().ok_or(Failure::UnsupportedDurability)?)?;
            let db=open_pinned(path)?;
            let mut handle:Handle=null_mut();
            let status=unsafe{rusqlite::ffi::sqlite3_file_control(connection.handle(),b"main\0".as_ptr().cast(),29,(&mut handle as *mut Handle).cast())};
            if status!=rusqlite::ffi::SQLITE_OK||handle.is_null()
                ||file_identity(root.as_raw_handle())?!=self.root_file
                ||file_identity(db.as_raw_handle())?!=self.db_file
                ||file_identity(handle)?!=self.db_file
                ||file_identity(self.root.as_raw_handle())?!=self.root_file
                ||file_identity(self.db.as_raw_handle())?!=self.db_file {
                return Err(Failure::CustodyConflict);
            }
            Ok(())
        }
        pub(in crate::snapshot_v2_durable_transaction::native) fn capture_created(path: &std::path::Path) -> Result<Self, Failure> {
            use std::os::windows::io::AsRawHandle;
            let root = open_pinned(path.parent().ok_or(Failure::UnsupportedDurability)?)?;
            let db = open_pinned(path)?;
            Ok(Self { root_file: file_identity(root.as_raw_handle())?, db_file: file_identity(db.as_raw_handle())?, root, db })
        }
        fn verify_pinned(&self, pinned: &PinnedNativeRootV1) -> Result<(), Failure> {
            use std::os::windows::io::AsRawHandle;
            if self.root_file != pinned.root_file || self.db_file != pinned.db_file
                || file_identity(self.root.as_raw_handle())? != pinned.root_file || file_identity(self.db.as_raw_handle())? != pinned.db_file {
                return Err(Failure::CustodyConflict);
            }
            Ok(())
        }
    }

    impl PinnedNativeRootV1 {
        pub(in crate::snapshot_v2_durable_transaction::native) fn capture(path: &std::path::Path, registration: bool) -> Result<Self, Failure> {
            use std::os::windows::io::AsRawHandle;
            let root_path = path.parent().ok_or(Failure::UnsupportedDurability)?;
            super::super::reject_reparse(root_path)?;
            super::super::reject_reparse(path)?;
            // Share READ|WRITE, never DELETE: retain both actual objects, not
            // only path strings, for the entire admitted operation lifetime.
            let root = open_pinned(root_path)?;
            let db = open_pinned(path)?;
            let mut filesystem = [0u16; 32];
            if unsafe { GetVolumeInformationByHandleW(root.as_raw_handle(), null_mut(), 0, null_mut(), null_mut(), null_mut(), filesystem.as_mut_ptr(), 32) } == 0 {
                return Err(Failure::UnsupportedDurability);
            }
            if filesystem[..5] != [b'N' as u16, b'T' as u16, b'F' as u16, b'S' as u16, 0] {
                return Err(Failure::UnsupportedDurability);
            }
            let (root_file, root_object) = physical_identity(root.as_raw_handle(), registration)?;
            let (db_file, db_object) = physical_identity(db.as_raw_handle(), registration)?;
            Ok(Self { root, db, root_file, root_object, db_file, db_object })
        }

        pub(in crate::snapshot_v2_durable_transaction::native) fn verify_connection(&self, connection: &rusqlite::Connection) -> Result<(), Failure> {
            use std::os::windows::io::AsRawHandle;
            let mut handle: Handle = null_mut();
            // The bundled SQLite win VFS returns its actual borrowed HANDLE
            // for this connection. Never close it or reopen a same-name path.
            let status = unsafe { rusqlite::ffi::sqlite3_file_control(connection.handle(), b"main\0".as_ptr().cast(), 29, (&mut handle as *mut Handle).cast()) };
            if status != rusqlite::ffi::SQLITE_OK || handle.is_null() { return Err(Failure::UnsupportedDurability); }
            if physical_identity(handle, false)? != (self.db_file, self.db_object)
                || physical_identity(self.db.as_raw_handle(), false)? != (self.db_file, self.db_object)
                || physical_identity(self.root.as_raw_handle(), false)? != (self.root_file, self.root_object) {
                return Err(Failure::CustodyConflict);
            }
            Ok(())
        }
    }

    fn transform(input: &[u8], protect: bool, machine: bool) -> Result<Vec<u8>, Failure> {
        if input.is_empty() || input.len() > MAX_CIPHERTEXT_BYTES { return Err(Failure::LimitExceeded); }
        let input = Blob { length: input.len() as u32, data: input.as_ptr() as *mut u8 };
        let mut output = ProtectedOutput(Blob { length: 0, data: null_mut() });
        // SAFETY: input remains borrowed for this synchronous call; output is
        // owned by ProtectedOutput after either result. UI is always forbidden.
        let result = unsafe {
            if protect { CryptProtectData(&input, null(), null(), null_mut(), null_mut(), 1 | if machine { 4 } else { 0 }, &mut output.0) }
            else { CryptUnprotectData(&input, null_mut(), null(), null_mut(), null_mut(), 1, &mut output.0) }
        };
        if result == 0 { return Err(Failure::CustodyConflict); }
        let length = output.0.length as usize;
        if output.0.data.is_null() || length == 0 || length > MAX_CIPHERTEXT_BYTES { return Err(Failure::LimitExceeded); }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(|_| Failure::LimitExceeded)?;
        // SAFETY: DPAPI returned a valid length-byte allocation, retained by
        // output until after this copy; caller never borrows its allocation.
        bytes.extend_from_slice(unsafe { std::slice::from_raw_parts(output.0.data, length) });
        Ok(bytes)
    }

    fn protect(bytes: &[u8]) -> Result<Vec<u8>, Failure> {
        let mut inner = transform(bytes, true, false)?;
        let result = transform(&inner, true, true);
        inner.fill(0);
        result
    }
    fn unprotect(bytes: &[u8]) -> Result<Vec<u8>, Failure> {
        let mut inner = transform(bytes, false, false)?;
        let result = transform(&inner, false, false);
        inner.fill(0);
        result
    }

    /// Store mechanics only. No function returns an admitted root capability.
    /// A caller still needs its actual source owner, pinned root/DB and lease.
    struct WindowsRootGrantStoreV1;
    impl WindowsRootGrantStoreV1 {
        fn key(namespace: Namespace, locator: &[u8; 32], create: bool, require_new: bool) -> Result<Key, Failure> {
            if locator.iter().all(|byte| *byte == 0) { return Err(Failure::InvalidInput); }
            let path = utf16(&format!("{}{}", namespace.prefix(), hex::encode(locator)));
            let mut key = null_mut();
            let mut disposition = 0;
            // Fixed 64-bit registry view on every open; only this exact nonce
            // is addressed. No enumeration, deletion or ambient fallback.
            let status = unsafe {
                if create { RegCreateKeyExW(hkcu(), path.as_ptr(), 0, null_mut(), 0, 0x103, null(), &mut key, &mut disposition) }
                else { RegOpenKeyExW(hkcu(), path.as_ptr(), 0, 0x101, &mut key) }
            };
            if status != 0 || key.is_null() { return Err(Failure::CustodyConflict); }
            let key = Key(key);
            if require_new && disposition != 1 { return Err(Failure::ConflictingRetry); }
            Ok(key)
        }
        fn ensure_slot_absent(fingerprint: &[u8; 32]) -> Result<(), Failure> {
            let path = utf16(&format!("{}{}", Namespace::RootSlot.prefix(), hex::encode(fingerprint)));
            let mut key = null_mut();
            let status = unsafe { RegOpenKeyExW(hkcu(), path.as_ptr(), 0, 0x101, &mut key) };
            if status == 2 { return Ok(()); }
            if status == 0 && !key.is_null() { drop(Key(key)); return Err(Failure::ConflictingRetry); }
            Err(Failure::CustodyConflict)
        }
        fn read_plaintext(namespace: Namespace, locator: &[u8; 32]) -> Result<Vec<u8>, Failure> {
            let key = Self::key(namespace, locator, false, false)?;
            Self::read_plaintext_on_key(&key,namespace.value_name())
        }
        fn read_plaintext_on_key(key: &Key,value_name:&str) -> Result<Vec<u8>, Failure> {
            let _timing=crate::storage_observation::Span::start("protected.registry_read_unprotect");
            let name = utf16(value_name);
            let mut kind = 0;
            let mut length = 0;
            // Query the bound before allocating any registry data buffer.
            let status = unsafe { RegQueryValueExW(key.0, name.as_ptr(), null_mut(), &mut kind, null_mut(), &mut length) };
            if status != 0 || kind != 3 || length == 0 || length as usize > MAX_CIPHERTEXT_BYTES { return Err(Failure::CorruptRecoveryRecord); }
            let mut ciphertext = Vec::new();
            ciphertext.try_reserve_exact(length as usize).map_err(|_| Failure::LimitExceeded)?;
            ciphertext.resize(length as usize, 0);
            let expected_length = length;
            let status = unsafe { RegQueryValueExW(key.0, name.as_ptr(), null_mut(), &mut kind, ciphertext.as_mut_ptr(), &mut length) };
            if status != 0 || kind != 3 || length != expected_length { return Err(Failure::RereadFailed); }
            unprotect(&ciphertext)
        }
        fn read(nonce: &[u8; 32]) -> Result<GrantRecord, Failure> {
            let mut plaintext = Self::read_plaintext(Namespace::Grant, nonce)?;
            let record = GrantRecord::decode(&plaintext);
            plaintext.fill(0);
            let record = record?;
            if record.nonce != *nonce { return Err(Failure::CustodyConflict); }
            Ok(record)
        }
        fn write_verified(record: &GrantRecord, require_new: bool) -> Result<(), Failure> {
            let mut plaintext = record.encode()?;
            let result = Self::write_plaintext(Namespace::Grant, &record.nonce, &plaintext, require_new);
            plaintext.fill(0);
            result
        }
        fn write_plaintext(namespace: Namespace, locator: &[u8; 32], plaintext: &[u8], require_new: bool) -> Result<(), Failure> {
            let _timing=crate::storage_observation::Span::start("protected.protect_write_flush_requery");
            let ciphertext = protect(plaintext)?;
            let namespace_for_read = namespace;
            let key = if matches!(namespace,Namespace::Attempt|Namespace::SourceTransition) {
                if require_new {return Err(Failure::InvalidInput);}
                // Auxiliary records live under an existing grant. Never
                // create/heal a missing grant namespace to write either one.
                let path=utf16(&format!("{}{}",namespace.prefix(),hex::encode(locator)));
                let mut handle=null_mut();
                if unsafe {RegOpenKeyExW(hkcu(),path.as_ptr(),0,0x103,&mut handle)}!=0 || handle.is_null() {
                    return Err(Failure::CustodyConflict);
                }
                Key(handle)
            }else{Self::key(namespace, locator, true, require_new)?};
            let name = utf16(namespace.value_name());
            #[cfg(test)] inject_io_failure(namespace,IoStep::Set)?;
            if unsafe { RegSetValueExW(key.0, name.as_ptr(), 0, 3, ciphertext.as_ptr(), ciphertext.len() as u32) } != 0 {
                return Err(Failure::WriteFailed);
            }
            #[cfg(test)] inject_io_failure(namespace,IoStep::Flush)?;
            if unsafe { RegFlushKey(key.0) } != 0 { return Err(Failure::SyncFailed); }
            drop(key);
            // A new key handle and both decryptions are required. Set/Flush
            // alone must never publish successful registration/revocation.
            #[cfg(test)] inject_io_failure(namespace,IoStep::FreshRead)?;
            let mut reread = Self::read_plaintext(namespace_for_read, locator)?;
            let matches = reread == plaintext;
            reread.fill(0);
            if matches { Ok(()) } else { Err(Failure::RereadMismatch) }
        }
        fn read_slot(fingerprint: &[u8; 32]) -> Result<[u8; 32], Failure> {
            let mut bytes = Self::read_plaintext(Namespace::RootSlot, fingerprint)?;
            let nonce = decode_slot(&bytes); bytes.fill(0); nonce
        }
        fn write_slot_new(fingerprint: &[u8; 32], nonce: &[u8; 32]) -> Result<(), Failure> {
            let mut bytes = encode_slot(nonce)?;
            let result = Self::write_plaintext(Namespace::RootSlot, fingerprint, &bytes, true);
            bytes.fill(0); result
        }
        fn read_attempt(grant:&GrantRecord) -> Result<AttemptRecord,Failure> {
            let mut bytes=Self::read_plaintext(Namespace::Attempt,&grant.nonce)?;
            let decoded=AttemptRecord::decode(&bytes);bytes.fill(0);let attempt=decoded?;
            if grant.lifecycle!=Lifecycle::Active || attempt.nonce!=grant.nonce || attempt.generation!=grant.generation {
                return Err(Failure::CustodyConflict);
            }
            Ok(attempt)
        }

        /// Mechanical first-Pending storage only; no DB write/start capability.
        /// Higher epochs require separately admitted previous outcome custody.
        fn write_first_pending(grant:&GrantRecord,pending:&AttemptRecord) -> Result<(),Failure> {
            if Self::read(&grant.nonce)?!=*grant || grant.lifecycle!=Lifecycle::Active
                || pending.nonce!=grant.nonce || pending.generation!=grant.generation
                || pending.epoch!=1 || pending.mode!=AttemptMode::Pending {
                return Err(Failure::CustodyConflict);
            }
            pending.validate()?;
            let key=Self::key(Namespace::Grant,&grant.nonce,false,false)?;
            let name=utf16("Attempt");let mut kind=0;let mut length=0;
            let status=unsafe{RegQueryValueExW(key.0,name.as_ptr(),null_mut(),&mut kind,null_mut(),&mut length)};
            // Only an absent value permits the first epoch. Partial/corrupt/
            // prior Pending is never overwritten or inferred to have succeeded.
            if status!=2 {return Err(if status==0 {Failure::ConflictingRetry}else{Failure::CustodyConflict});}
            drop(key);
            let mut bytes=pending.encode()?;
            let written=Self::write_plaintext(Namespace::Attempt,&grant.nonce,&bytes,false);
            bytes.fill(0);written?;
            if Self::read_attempt(grant)?!=*pending || Self::read(&grant.nonce)?!=*grant {
                return Err(Failure::RereadMismatch);
            }
            Ok(())
        }
        fn write_next_pending(grant:&GrantRecord,previous:&AttemptRecord,pending:&AttemptRecord)->Result<(),Failure> {
            let mut expected=previous.clone();
            expected.epoch=expected.epoch.checked_add(1).ok_or(Failure::LimitExceeded)?;
            expected.mode=AttemptMode::Pending;expected.record_hash=[0;32];expected.validate()?;
            if previous.mode!=AttemptMode::RecordWitness || expected!=*pending
                || Self::read(&grant.nonce)?!=*grant || Self::read_attempt(grant)?!=*previous {
                return Err(Failure::CustodyConflict);
            }
            let mut bytes=pending.encode()?;
            let result=Self::write_plaintext(Namespace::Attempt,&grant.nonce,&bytes,false);
            bytes.fill(0);result?;
            if Self::read(&grant.nonce)?!=*grant || Self::read_attempt(grant)?!=*pending {return Err(Failure::RereadMismatch);}
            Ok(())
        }
        fn confirm_current(namespace: Namespace, locator: &[u8; 32], expected: &[u8]) -> Result<(), Failure> {
            // Explicit current confirmation: no SetValue/Create/rewrite. A
            // successful read must precede a new Flush on this actual handle.
            let key = Self::key(namespace, locator, false, false)?;
            let mut before = Self::read_plaintext_on_key(&key,namespace.value_name())?;
            let equal = before == expected; before.fill(0);
            if !equal { return Err(Failure::RereadMismatch); }
            #[cfg(test)] inject_io_failure(namespace,IoStep::Flush)?;
            if unsafe { RegFlushKey(key.0) } != 0 { return Err(Failure::SyncFailed); }
            drop(key);
            #[cfg(test)] inject_io_failure(namespace,IoStep::FreshRead)?;
            let mut after = Self::read_plaintext(namespace, locator)?;
            let equal = after == expected; after.fill(0);
            if equal { Ok(()) } else { Err(Failure::RereadMismatch) }
        }
    }

    impl super::super::SqliteAdapterV1 {
        pub(crate) fn register_saved_root_grant(&mut self, source: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>) -> Result<AdmittedRootGrantV1, Failure> {
            // Consume before any registration I/O. A failed/partial attempt
            // cannot regain freshness by opening an existing DB or changing
            // project/SID/fingerprint. Only provision can issue this token.
            let fresh = self.fresh_container.get_mut().take().ok_or(Failure::CustodyConflict)?;
            let project_hash = source_project_hash(source)?;
            let sid_hash = current_sid_hash()?;
            self.pin_objects_for_grant(true)?;
            let pinned = self.pinned_root.as_ref().ok_or(Failure::UnsupportedDurability)?.clone();
            fresh.verify_pinned(&pinned)?;
            let lease = self.root_lease.as_ref().ok_or(Failure::UnsupportedDurability)?.clone();
            let mut nonce = [0u8; 32];
            getrandom::fill(&mut nonce).map_err(|_| Failure::UnsupportedDurability)?;
            #[cfg(test)] LAST_REGISTRATION_NONCE.with(|last|last.set(Some(nonce)));
            let record = GrantRecord { nonce, sid_hash, project_hash, registration_pesi: *source.pesi().as_raw(),
                root_file: pinned.root_file, root_object: pinned.root_object, db_file: pinned.db_file, db_object: pinned.db_object,
                lifecycle: Lifecycle::Active, generation: 0 };
            let fingerprint = root_fingerprint(&record);
            WindowsRootGrantStoreV1::ensure_slot_absent(&fingerprint)?;
            WindowsRootGrantStoreV1::write_verified(&record, true)?;
            WindowsRootGrantStoreV1::write_slot_new(&fingerprint, &record.nonce)?;
            let admitted = AdmittedRootGrantV1 { record, pinned, lease };
            admitted.verify_current(self, source)?;
            Ok(admitted)
        }

        fn read_saved_root_grant_candidate(&mut self, locator: &str, source: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>) -> Result<RootGrantCandidateV1, Failure> {
            if locator.len() != 64 || !locator.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
                return Err(Failure::InvalidInput);
            }
            let raw = hex::decode(locator).map_err(|_| Failure::InvalidInput)?;
            let nonce: [u8; 32] = raw.try_into().map_err(|_| Failure::InvalidInput)?;
            let record = WindowsRootGrantStoreV1::read(&nonce)?;
            if record.lifecycle != Lifecycle::Active || record.project_hash != source_project_hash(source)? || record.sid_hash != current_sid_hash()? {
                return Err(Failure::CustodyConflict);
            }
            self.pin_objects_for_grant(false)?;
            let candidate = RootGrantCandidateV1 { record,
                pinned: self.pinned_root.as_ref().ok_or(Failure::UnsupportedDurability)?.clone(),
                lease: self.root_lease.as_ref().ok_or(Failure::UnsupportedDurability)?.clone() };
            verify_bound_root(&candidate.record,&candidate.pinned,&candidate.lease,self,source)?;
            Ok(candidate)
        }

        /// Ordinary cold read/open never produces an admitted capability.
        /// Retained for the original RED: its expectation remains rejection.
        pub(crate) fn reopen_saved_root_grant(&mut self, locator: &str, source: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>) -> Result<AdmittedRootGrantV1, Failure> {
            let _candidate=self.read_saved_root_grant_candidate(locator,source)?;
            Err(Failure::UnsupportedDurability)
        }

        pub(crate) fn confirm_saved_root_grant(&mut self, locator: &str, source: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>) -> Result<AdmittedRootGrantV1, Failure> {
            let candidate=self.read_saved_root_grant_candidate(locator,source)?;
            let mut grant_bytes=candidate.record.encode()?;
            let grant_confirmation=WindowsRootGrantStoreV1::confirm_current(Namespace::Grant,&candidate.record.nonce,&grant_bytes);
            grant_bytes.fill(0); grant_confirmation?;
            let mut slot_bytes=encode_slot(&candidate.record.nonce)?;
            let slot_confirmation=WindowsRootGrantStoreV1::confirm_current(Namespace::RootSlot,&root_fingerprint(&candidate.record),&slot_bytes);
            slot_bytes.fill(0); slot_confirmation?;
            verify_bound_root(&candidate.record,&candidate.pinned,&candidate.lease,self,source)?;
            // This is current confirmation, never historical registration ack
            // or transaction/cause/Witness/receipt admission.
            Ok(AdmittedRootGrantV1{record:candidate.record,pinned:candidate.pinned,lease:candidate.lease})
        }

        #[cfg(test)]
        pub(crate) fn test_root_grant_confirmation_fault(&self, name:&str) {
            let point=match name {
                "grant-flush"=>(Namespace::Grant,IoStep::Flush),
                "slot-flush"=>(Namespace::RootSlot,IoStep::Flush),
                "grant-read"=>(Namespace::Grant,IoStep::FreshRead),
                "slot-read"=>(Namespace::RootSlot,IoStep::FreshRead),
                _=>panic!("closed test fault set"),
            };
            IO_FAULT.with(|fault|fault.set(Some(point)));
        }

        pub(crate) fn revoke_saved_root_grant(&mut self, locator: &str, source: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>) -> Result<String, Failure> {
            let candidate=self.read_saved_root_grant_candidate(locator,source)?;
            verify_bound_root(&candidate.record,&candidate.pinned,&candidate.lease,self,source)?;
            WindowsRootGrantStoreV1::write_verified(&candidate.record.revoked()?,false)?;
            Ok(hex::encode(candidate.record.nonce))
        }
    }

    fn verify_bound_root(record: &GrantRecord, pinned: &std::sync::Arc<PinnedNativeRootV1>, lease: &std::sync::Arc<std::fs::File>, adapter: &super::super::SqliteAdapterV1, source: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>) -> Result<(), Failure> {
            if !adapter.root_lease.as_ref().is_some_and(|owner| std::sync::Arc::ptr_eq(owner, lease))
                || !adapter.pinned_root.as_ref().is_some_and(|owner| std::sync::Arc::ptr_eq(owner, pinned)) {
                return Err(Failure::CustodyConflict);
            }
            let current = WindowsRootGrantStoreV1::read(&record.nonce)?;
            if current != *record || current.lifecycle != Lifecycle::Active
                || current.sid_hash != current_sid_hash()? || current.project_hash != source_project_hash(source)?
                || (current.root_file, current.root_object) != (pinned.root_file, pinned.root_object)
                || (current.db_file, current.db_object) != (pinned.db_file, pinned.db_object) {
                return Err(Failure::CustodyConflict);
            }
            if WindowsRootGrantStoreV1::read_slot(&root_fingerprint(&current))? != current.nonce {
                return Err(Failure::CustodyConflict);
            }
            // registration_pesi records the registration's provenance; it is
            // not substituted for a later request's separately checked PESI.
            pinned.verify_connection(&adapter.connect()?)
    }

    impl AdmittedRootGrantV1 {
        pub(crate) fn locator(&self) -> String { hex::encode(self.record.nonce) }
        pub(crate) fn verify_current(&self, adapter: &super::super::SqliteAdapterV1, source: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>) -> Result<(), Failure> {
            verify_bound_root(&self.record,&self.pinned,&self.lease,adapter,source)
        }

        pub(crate) fn start_initial_attempt(self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
            request:crate::snapshot_v2_durable_transaction::retained_attempt::ActualPreparedRequestV1,
        )->Result<DurableAttemptStartV1,Failure> {
            if adapter.schema_profile!=super::super::NativeSchemaProfile::RecoveryCustody
                || !request.binding().expected_parent.is_empty() {return Err(Failure::UnsupportedDurability);}
            request.check_source(source)?;
            self.verify_current(adapter,source)?;
            let digest=request.binding().transaction_digest()?;
            let mut token=[0u8;33];token[0]=1;token[1..].copy_from_slice(&digest);
            let attempt=AttemptRecord::initial_pending(&self.record,digest,request.raw(),&token)?;
            WindowsRootGrantStoreV1::write_first_pending(&self.record,&attempt)?;
            self.verify_current(adapter,source)?;
            #[cfg(test)]
            crate::snapshot_v2_durable_transaction::pause_native_owner_test_process("after_pending_ack");
            Ok(DurableAttemptStartV1{grant:self,attempt,request})
        }

        pub(crate) fn load_initial_attempt(self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
            expected_raw:&[u8],expected_token:&[u8;33],
        )->Result<AdmittedInitialAttemptV1,Failure> {
            use sha2::{Digest,Sha256};
            use crate::snapshot_v2_durable_transaction::{retained_attempt::ExplicitRecoveryRequestV1,
                recovery_custody::InitialPrefixVerifierV1};
            if adapter.schema_profile!=super::super::NativeSchemaProfile::RecoveryCustody {return Err(Failure::UnsupportedDurability);}
            self.verify_current(adapter,source)?;
            let expected=ExplicitRecoveryRequestV1::check(expected_raw,source)?;
            let attempt=WindowsRootGrantStoreV1::read_attempt(&self.record)?;
            // INITIAL transaction and its bounded recovery attempts. Each
            // acknowledged epoch has exactly one retained outcome row.
            if attempt.epoch>63 {return Err(Failure::LimitExceeded);}
            if expected_token[0]!=1 || expected_token[1..]!=expected.payload().digest
                || attempt.transaction!=expected.payload().digest
                || attempt.request_hash!=<[u8;32]>::from(Sha256::digest(expected.raw()))
                || attempt.token_hash!=<[u8;32]>::from(Sha256::digest(expected_token)) {
                return Err(Failure::CustodyConflict);
            }
            // The host anchor is consulted BEFORE the DB prefix. Even a full
            // terminal SQL record cannot silently heal a Pending admission.
            if attempt.mode!=AttemptMode::RecordWitness {return Err(Failure::RecoveryRequired);}
            let mut encoded=attempt.encode()?;
            let confirmation=WindowsRootGrantStoreV1::confirm_current(Namespace::Attempt,&self.record.nonce,&encoded);
            encoded.fill(0);confirmation?;
            let mut verifier=InitialPrefixVerifierV1::new(&expected,attempt.record_hash,attempt.epoch);
            adapter.visit_initial_custody_rows(expected.payload(),|ordinal,previous,request,record,hash| {
                verifier.accept(ordinal,previous,request,record,hash)
            })?;
            let record=verifier.finish()?;
            self.verify_current(adapter,source)?;
            if WindowsRootGrantStoreV1::read_attempt(&self.record)?!=attempt {return Err(Failure::CustodyConflict);}
            Ok(AdmittedInitialAttemptV1{_grant:self,_attempt:attempt,_expected:expected,record})
        }

        pub(crate) fn revoke(&self, adapter: &super::super::SqliteAdapterV1, source: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>) -> Result<(), Failure> {
            self.verify_current(adapter, source)?;
            WindowsRootGrantStoreV1::write_verified(&self.record.revoked()?, false)
        }
    }

    impl AdmittedInitialAttemptV1 {
        pub(in crate::snapshot_v2_durable_transaction) fn check_current(&self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        )->Result<(),Failure> {
            self._expected.check_source(source)?;
            self._grant.verify_current(adapter,source)?;
            if WindowsRootGrantStoreV1::read_attempt(&self._grant.record)?!=self._attempt {return Err(Failure::CustodyConflict);}
            self.check_bound_custody(adapter,source)?;
            if WindowsRootGrantStoreV1::read_attempt(&self._grant.record)?!=self._attempt {return Err(Failure::CustodyConflict);}
            Ok(())
        }
        fn check_bound_custody(&self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        )->Result<(),Failure> {
            use crate::snapshot_v2_durable_transaction::recovery_custody::InitialPrefixVerifierV1;
            self._expected.check_source(source)?;self._grant.verify_current(adapter,source)?;
            let mut verifier=InitialPrefixVerifierV1::new(&self._expected,self._attempt.record_hash,self._attempt.epoch);
            adapter.visit_initial_custody_rows(self._expected.payload(),|ordinal,previous,request,record,hash| {
                verifier.accept(ordinal,previous,request,record,hash)
            })?;
            if verifier.finish()?.record()!=self.record.record() {return Err(Failure::CustodyConflict);}
            self._grant.verify_current(adapter,source)?;
            Ok(())
        }
        pub(crate) fn start_recovery(self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        )->Result<RecoveryAttemptStartV1,Failure> {
            self.check_current(adapter,source)?;
            if self.record.state()!=crate::snapshot_v2_durable_transaction::SnapshotV2TransactionStateV1::RecoveryRequired {
                return Err(Failure::UnsupportedDurability);
            }
            if self.record.ordinal()>=63 {return Err(Failure::LimitExceeded);}
            let mut pending=self._attempt.clone();pending.epoch=pending.epoch.checked_add(1).ok_or(Failure::LimitExceeded)?;
            pending.mode=AttemptMode::Pending;pending.record_hash=[0;32];
            WindowsRootGrantStoreV1::write_next_pending(&self._grant.record,&self._attempt,&pending)?;
            self._grant.verify_current(adapter,source)?;
            Ok(RecoveryAttemptStartV1{prior:self,pending})
        }
    }

    impl RecoveryAttemptStartV1 {
        pub(in crate::snapshot_v2_durable_transaction) fn check_anchor(&self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        )->Result<(),Failure> {
            self.prior._expected.check_source(source)?;self.prior._grant.verify_current(adapter,source)?;
            if WindowsRootGrantStoreV1::read_attempt(&self.prior._grant.record)?!=self.pending {return Err(Failure::CustodyConflict);}
            Ok(())
        }
        pub(in crate::snapshot_v2_durable_transaction) fn check_current(&self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        )->Result<(),Failure> {
            self.check_anchor(adapter,source)?;self.prior.check_bound_custody(adapter,source)?;self.check_anchor(adapter,source)
        }
        pub(in crate::snapshot_v2_durable_transaction) fn retain_record_witness(&self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
            sql:&crate::snapshot_v2_durable_transaction::retained_attempt::SqlRetainedRecoveryV1,
        )->Result<RetainedAttemptWitnessV1,Failure> {
            use sha2::{Digest,Sha256};
            if !std::ptr::eq(self,&sql.actual().start) {return Err(Failure::CustodyConflict);}
            self.check_anchor(adapter,source)?;sql.revalidate(adapter)?;
            let mut witness=self.pending.clone();witness.mode=AttemptMode::RecordWitness;
            witness.record_hash=Sha256::digest(sql.row().record()).into();witness.validate()?;
            self.check_anchor(adapter,source)?;
            let mut bytes=witness.encode()?;
            let result=WindowsRootGrantStoreV1::write_plaintext(Namespace::Attempt,&self.prior._grant.record.nonce,&bytes,false);
            bytes.fill(0);result?;
            self.prior._grant.verify_current(adapter,source)?;
            if WindowsRootGrantStoreV1::read_attempt(&self.prior._grant.record)?!=witness {return Err(Failure::RereadMismatch);}
            sql.revalidate(adapter)?;
            Ok(RetainedAttemptWitnessV1{_attempt:witness})
        }
    }

    impl DurableAttemptStartV1 {
        pub(crate) fn request(&self)->&crate::snapshot_v2_durable_transaction::retained_attempt::ActualPreparedRequestV1 {&self.request}
        pub(crate) fn check_current(&self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        )->Result<(),Failure> {
            if adapter.schema_profile!=super::super::NativeSchemaProfile::RecoveryCustody {return Err(Failure::UnsupportedDurability);}
            self.request.check_source(source)?;
            self.grant.verify_current(adapter,source)?;
            if self.attempt.mode!=AttemptMode::Pending || WindowsRootGrantStoreV1::read_attempt(&self.grant.record)?!=self.attempt {
                return Err(Failure::CustodyConflict);
            }
            Ok(())
        }

        pub(in crate::snapshot_v2_durable_transaction) fn retain_record_witness(&self,adapter:&super::super::SqliteAdapterV1,
            source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
            sql:&crate::snapshot_v2_durable_transaction::retained_attempt::SqlRetainedOutcomeV1,
        )->Result<RetainedAttemptWitnessV1,Failure> {
            use sha2::{Digest,Sha256};
            if !std::ptr::eq(self,&sql.actual().start) {return Err(Failure::CustodyConflict);}
            self.check_current(adapter,source)?;
            sql.revalidate(adapter)?;
            let mut witness=self.attempt.clone();
            witness.mode=AttemptMode::RecordWitness;
            witness.record_hash=Sha256::digest(sql.row().record()).into();
            witness.validate()?;
            // Final current grant/pinned-object/Pending check immediately
            // before Set. The existing exclusive root lease serializes owners.
            self.check_current(adapter,source)?;
            let mut bytes=witness.encode()?;
            let written=WindowsRootGrantStoreV1::write_plaintext(Namespace::Attempt,&self.grant.record.nonce,&bytes,false);
            bytes.fill(0);written?;
            self.grant.verify_current(adapter,source)?;
            if WindowsRootGrantStoreV1::read_attempt(&self.grant.record)?!=witness {return Err(Failure::RereadMismatch);}
            sql.revalidate(adapter)?;
            Ok(RetainedAttemptWitnessV1{_attempt:witness})
        }
    }

    #[test]
    fn m2_actual_outcome_retention_requires_full_sql_prefix_and_durable_witness() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::{self as owner,SnapshotV2TransactionStateV1 as State};
        use owner::retained_attempt::ActualPreparedRequestV1;
        use sha2::{Digest,Sha256};
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:vec![SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n".into()},
                SourceFile{path:"app/main.ddn".into(),text:"실행:움직씨 := { 1 보여주기. }.\n".into()}],
        },String::new()).unwrap().save().unwrap();
        let run=crate::runtime_surface::run_authored_general_input_prefix(&artifact,1,17,None,None).unwrap();
        let saved=run.export_saved_artifact(&[],&[2]).unwrap();
        let bytes=attachment::check_bytes(saved.as_bytes()).unwrap();
        let source=attachment::check_saved_revision_context(&bytes,run.source_owner(),&[],&[2]).unwrap();
        // Normal, SQL before INSERT/after COMMIT, host Set/Flush/fresh-read,
        // damaged earlier row, original failed Verified append, retained
        // pre/post-CAS failures, and a post-CAS failure plus host Set failure.
        for mode in 0..11 {
            let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
            let root=std::env::temp_dir().join(format!("ddn-m2-outcome-witness-{}",hex::encode(nonce)));
            std::fs::create_dir(&root).unwrap();
            let mut adapter=super::super::SqliteAdapterV1::provision_recovery_custody(&root).unwrap();
            let grant=adapter.register_saved_root_grant(&source).unwrap();
            let record=GrantRecord::decode(&grant.record.encode().unwrap()).unwrap();
            let request=ActualPreparedRequestV1::from_general(&run,&source,&[],&[2]).unwrap();
            let digest=request.payload().digest;
            let start=grant.start_initial_attempt(&adapter,&source,request).unwrap();
            let pending=start.attempt.clone();
            if mode==7 {super::super::FAIL_VERIFIED_CUSTODY_INSERT.with(|fault|fault.set(true));}
            if mode==8 {super::super::FAIL_CUSTODY_PREPARED_REREAD.with(|fault|fault.set(true));}
            if mode==9 || mode==10 {super::super::FAIL_CUSTODY_CONFIRM.with(|fault|fault.set(true));}
            let actual=owner::run_initial_native_owner_with_custody(&adapter,&source,start).unwrap();
            let original_state=actual.owner.evidence().state();
            let original_failure=actual.owner.failure().cloned();
            assert_eq!(original_state,match mode {7|8=>State::Aborted,9|10=>State::RecoveryRequired,_=>State::Committed});
            if mode==1 || mode==2 {super::super::FAIL_OUTCOME_CUSTODY.with(|fault|fault.set(Some(mode-1)));}
            if (3..=5).contains(&mode) {
                IO_FAULT.with(|fault|fault.set(Some((Namespace::Attempt,match mode {3=>IoStep::Set,4=>IoStep::Flush,_=>IoStep::FreshRead}))));
            }
            if mode==10 {IO_FAULT.with(|fault|fault.set(Some((Namespace::Attempt,IoStep::Set))));}
            if mode==6 {
                // Mutate only this random test-owned database; no file/key deletion.
                adapter.connect().unwrap().execute("UPDATE vc04_recovery_custody_v1 SET row_sha256=zeroblob(32) WHERE journal_ordinal=1",[]).unwrap();
            }
            match actual.retain(&adapter,&source) {
                Ok(retained)=>{
                    assert!(matches!(mode,0|8|9),"no incomplete acknowledgement may yield a retained capability");
                    assert_eq!(retained.owner().evidence().state(),original_state);
                    assert_eq!(retained.owner().failure(),original_failure.as_ref());
                }
                Err(blocked)=>{
                    assert_ne!(mode,0);
                    assert_eq!(blocked.actual.owner.evidence().state(),original_state);
                    assert_eq!(blocked.actual.owner.failure(),original_failure.as_ref(),"retention never replaces the primary cause/state");
                    let expected=match mode {1|10=>Failure::WriteFailed,2|5=>Failure::RereadFailed,
                        3=>Failure::WriteFailed,4=>Failure::SyncFailed,6|7=>Failure::CustodyConflict,_=>unreachable!()};
                    assert_eq!(blocked.retention_failure,expected);
                    if mode==7 {assert_eq!(blocked.actual.owner.failure().unwrap().cause,Failure::WriteFailed);}
                    if mode==10 {assert_eq!(blocked.actual.owner.failure().unwrap().cause,Failure::ContainerDurabilityFailed);}
                }
            }
            assert!(IO_FAULT.with(|fault|fault.get().is_none()));
            assert!(super::super::FAIL_OUTCOME_CUSTODY.with(|fault|fault.get().is_none()));
            let connection=adapter.connect().unwrap();
            let rows:i64=connection.query_row("SELECT count(*) FROM vc04_recovery_custody_v1",[],|row|row.get(0)).unwrap();
            assert_eq!(rows,match mode {1|6|8=>2,7=>1,_=>3});
            let head:i64=connection.query_row("SELECT count(*) FROM vc04_project_heads",[],|row|row.get(0)).unwrap();
            assert_eq!(head,if mode==7 || mode==8 {0}else{1});
            let after=WindowsRootGrantStoreV1::read_attempt(&record).unwrap();
            if matches!(mode,0|4|5|8|9) {
                assert_eq!(after.mode,AttemptMode::RecordWitness);
                let raw:Vec<u8>=connection.query_row("SELECT recovery_record_bytes FROM vc04_recovery_custody_v1 WHERE transaction_digest=?1 AND journal_ordinal=?2",rusqlite::params![digest.as_slice(),if mode==8 {1}else{2}],|row|row.get(0)).unwrap();
                assert_eq!(after.record_hash,<[u8;32]>::from(Sha256::digest(&raw)));
                let candidate=owner::recovery_custody::RecoveryRecordCandidateV1::decode(&raw).unwrap();
                assert_eq!(candidate.state(),original_state);
                assert_eq!(candidate.cause(),original_failure.as_ref().map(|failure|failure.cause).unwrap_or(Failure::Interrupted));
            }else{assert_eq!(after,pending,"failed SQL or host Set cannot publish a Witness");}
            println!("retained outcome/Witness mode{mode} root: {}",root.display());
        }
    }

    #[test]
    fn m2_actual_cold_initial_witness_load_preserves_cause_and_refuses_old_prefix() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::{self as owner,retained_attempt::ActualPreparedRequestV1};
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:vec![SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n".into()},
                SourceFile{path:"app/main.ddn".into(),text:"실행:움직씨 := { 1 보여주기. }.\n".into()}],
        },String::new()).unwrap().save().unwrap();
        let run=crate::runtime_surface::run_authored_general_input_prefix(&artifact,1,17,None,None).unwrap();
        let saved=run.export_saved_artifact(&[],&[2]).unwrap();
        let bytes=attachment::check_bytes(saved.as_bytes()).unwrap();
        let source=attachment::check_saved_revision_context(&bytes,run.source_owner(),&[],&[2]).unwrap();
        for mode in 0..7 {
            let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
            let directory=std::env::temp_dir().join(format!("ddn-m2-cold-witness-{}",hex::encode(nonce)));
            std::fs::create_dir(&directory).unwrap();
            let root=directory.join("root");std::fs::create_dir(&root).unwrap();
            let mut adapter=super::super::SqliteAdapterV1::provision_recovery_custody(&root).unwrap();
            let grant=adapter.register_saved_root_grant(&source).unwrap();
            let record=GrantRecord::decode(&grant.record.encode().unwrap()).unwrap();
            let request=ActualPreparedRequestV1::from_general(&run,&source,&[],&[2]).unwrap();
            let digest=request.payload().digest;
            let raw=request.raw().to_vec();
            let mut token=[0u8;33];token[0]=1;token[1..].copy_from_slice(&digest);
            let start=grant.start_initial_attempt(&adapter,&source,request).unwrap();
            if mode==1 {super::super::FAIL_CUSTODY_CONFIRM.with(|fault|fault.set(true));}
            let actual=owner::run_initial_native_owner_with_custody(&adapter,&source,start).unwrap();
            if mode==2 {IO_FAULT.with(|fault|fault.set(Some((Namespace::Attempt,IoStep::Set))));}
            match actual.retain(&adapter,&source) {
                Ok(retained)=>{assert_ne!(mode,2);drop(retained);}
                Err(blocked)=>{assert_eq!(mode,2);assert_eq!(blocked.retention_failure,Failure::WriteFailed);drop(blocked);}
            }
            if mode==3 {
                // Exact generated fixture only: preserve removed bytes first,
                // then simulate loss of its terminal row, leaving the old prefix.
                let connection=adapter.connect().unwrap();
                let terminal:Vec<u8>=connection.query_row("SELECT recovery_record_bytes FROM vc04_recovery_custody_v1 WHERE transaction_digest=?1 AND journal_ordinal=2",[digest.as_slice()],|row|row.get(0)).unwrap();
                std::fs::write(directory.join("lost-terminal-original.bin"),terminal).unwrap();
                assert_eq!(connection.execute("DELETE FROM vc04_recovery_custody_v1 WHERE transaction_digest=?1 AND journal_ordinal=2",[digest.as_slice()]).unwrap(),1);
            }
            if mode==4 {
                adapter.connect().unwrap().execute("UPDATE vc04_recovery_custody_v1 SET row_sha256=zeroblob(32) WHERE transaction_digest=?1 AND journal_ordinal=0",[digest.as_slice()]).unwrap();
            }
            if mode==5 {token[32]^=1;}
            if mode==6 {
                let mut attempt=WindowsRootGrantStoreV1::read_attempt(&record).unwrap();attempt.epoch=2;
                WindowsRootGrantStoreV1::write_plaintext(Namespace::Attempt,&record.nonce,&attempt.encode().unwrap(),false).unwrap();
            }
            std::fs::write(directory.join("artifact.json"),saved.as_bytes()).unwrap();
            std::fs::write(directory.join("request.bin"),raw).unwrap();
            std::fs::write(directory.join("token.bin"),token).unwrap();
            std::fs::write(directory.join("locator.txt"),hex::encode(record.nonce)).unwrap();
            // Neither an inherited Arc nor the writer's SQLite connection/
            // lease/outcome survives into the child's admission.
            drop(adapter);
            let child=std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact","snapshot_v2_durable_transaction::native::root_grant::windows::m2_actual_cold_witness_child","--ignored","--nocapture"])
                .env("DDN_M2_COLD_WITNESS_DIR",&directory).output().unwrap();
            assert!(child.status.success(),"cold child failed: {}\n{}",String::from_utf8_lossy(&child.stdout),String::from_utf8_lossy(&child.stderr));
            let report:serde_json::Value=serde_json::from_slice(&std::fs::read(directory.join("cold-result.json")).unwrap()).unwrap();
            let expected=match mode {
                0=>serde_json::json!({"state":4,"cause":21,"world_published":false}),
                1=>serde_json::json!({"state":6,"cause":18,"world_published":false}),
                2|3=>serde_json::json!({"error":"RecoveryRequired","world_published":false}),
                4|5=>serde_json::json!({"error":"CustodyConflict","world_published":false}),
                6=>serde_json::json!({"error":"CustodyConflict","world_published":false}),_=>unreachable!(),
            };
            assert_eq!(report,expected,"cold mode{mode}");
            println!("M2_COLD_CHILD mode{mode}: {}",String::from_utf8_lossy(&child.stdout));
            if mode==0 || mode==1 {
                let faults:&[&str]=if mode==0 {&["normal","confirm","reread","chain"]}else{&["stateful"]};
                for fault in faults {
                    let child=std::process::Command::new(std::env::current_exe().unwrap())
                        .args(["--exact","snapshot_v2_durable_transaction::native::root_grant::windows::m2_actual_cold_witness_child","--ignored","--nocapture"])
                        .env("DDN_M2_COLD_WITNESS_DIR",&directory).env("DDN_M2_COLD_READMIT",fault).output().unwrap();
                    assert!(child.status.success(),"readmit child: {}\n{}",String::from_utf8_lossy(&child.stdout),String::from_utf8_lossy(&child.stderr));
                    let result:serde_json::Value=serde_json::from_slice(&std::fs::read(directory.join(format!("readmit-{fault}.json"))).unwrap()).unwrap();
                    let expected=if *fault=="normal" {serde_json::json!({"state":4,"receipt_binding_checked":true,"world_published":false})}
                    else {serde_json::json!({"historical_state":if mode==1 {6}else{4},"original_cause":if mode==1 {18}else{21},
                        "error":match *fault {"confirm"=>"ContainerDurabilityFailed","reread"=>"RereadFailed","chain"=>"CustodyConflict",_=>"UnsupportedDurability"},"world_published":false})};
                    assert_eq!(result,expected,"readmission {fault}");
                    println!("M2_READMIT_CHILD {fault}: {}",String::from_utf8_lossy(&child.stdout));
                }
            }
            println!("retained cold Witness fixture: {}",directory.display());
        }
    }

    #[test]
    #[ignore = "only invoked by the exact parent with its own generated fixture"]
    fn m2_actual_cold_witness_child() {
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction as owner;
        let directory=std::path::PathBuf::from(std::env::var_os("DDN_M2_COLD_WITNESS_DIR").expect("exact parent fixture"));
        let artifact=std::fs::read(directory.join("artifact.json")).unwrap();
        let request=std::fs::read(directory.join("request.bin")).unwrap();
        let token: [u8;33]=std::fs::read(directory.join("token.bin")).unwrap().try_into().unwrap();
        let locator=std::fs::read_to_string(directory.join("locator.txt")).unwrap();
        let bytes=attachment::check_bytes(&artifact).unwrap();
        let owned=attachment::restore_saved_runtime_payload(&bytes).unwrap();
        let source=attachment::check_saved_revision_context(&bytes,&owned,&[],&[2]).unwrap();
        let mut adapter=super::super::SqliteAdapterV1::open_recovery_custody(&directory.join("root")).unwrap();
        let grant=adapter.confirm_saved_root_grant(&locator,&source).unwrap();
        if let Ok(phase)=std::env::var("DDN_M2_STATEFUL_PHASE") {
            assert!(["first","second","load"].contains(&phase.as_str()));
            let metadata=GrantRecord::decode(&grant.record.encode().unwrap()).unwrap();
            let fault=std::env::var("DDN_M2_STATEFUL_FAULT").unwrap_or_default();
            let loaded=grant.load_initial_attempt(&adapter,&source,&request,&token);
            let report=match loaded {
                Err(failure)=>serde_json::json!({"load_error":format!("{failure:?}"),"world_published":false}),
                Ok(prior) if phase=="load"=>serde_json::json!({"state":prior.historical_state() as u8,"primary":prior.original_cause() as u16,"world_published":false}),
                Ok(prior)=>{
                    let primary=prior.original_cause() as u16;
                    if fault=="reserve-flush" {
                        IO_FAULT.with(|point|point.set(Some((Namespace::Attempt,IoStep::Flush))));
                        super::super::FAIL_CUSTODY_CONFIRM.with(|point|point.set(true));
                    }
                    match prior.start_recovery(&adapter,&source) {
                        Err(failure)=>{
                            if fault=="reserve-flush" {assert!(super::super::FAIL_CUSTODY_CONFIRM.with(|point|point.replace(false)),"no recovery mechanics before a start acknowledgement");}
                            serde_json::json!({"start_error":format!("{failure:?}"),"primary":primary,"world_published":false})
                        },
                        Ok(start)=>{
                            if fault=="reread" {super::super::FAIL_CUSTODY_COMMITTED_REREAD.with(|point|point.set(true));}
                            let actual=owner::recover_initial_native_owner_with_custody(&adapter,&source,start).unwrap();
                            let state=actual.owner.evidence().state() as u8;
                            let mechanical=actual.attempt_failure.map(|failure|format!("{failure:?}"));
                            if fault=="append" {super::super::FAIL_OUTCOME_CUSTODY.with(|point|point.set(Some(0)));}
                            if fault=="witness" {IO_FAULT.with(|point|point.set(Some((Namespace::Attempt,IoStep::Set))));}
                            match actual.retain(&adapter,&source) {
                                Ok(retained)=>{
                                    assert_eq!(retained.owner().evidence().state() as u8,state);
                                    assert_eq!(retained.attempt_failure().map(|failure|format!("{failure:?}")),mechanical);
                                    serde_json::json!({"state":state,"primary":primary,"attempt_failure":mechanical,"retained":true,"world_published":false})
                                },
                                Err(blocked)=>{
                                    assert_eq!(blocked.actual.owner.evidence().state() as u8,state,"late retention must not change the actual outcome");
                                    serde_json::json!({"state":state,"primary":primary,"attempt_failure":mechanical,"retention_error":format!("{:?}",blocked.retention_failure),"world_published":false})
                                },
                            }
                        },
                    }
                },
            };
            let anchor=WindowsRootGrantStoreV1::read_attempt(&metadata).unwrap();
            let rows=adapter.connect().unwrap().query_row("SELECT count(*) FROM vc04_recovery_custody_v1",[],|row|row.get::<_,i64>(0)).unwrap();
            let result=serde_json::json!({"result":report,"epoch":anchor.epoch,"mode":format!("{:?}",anchor.mode),"rows":rows});
            std::fs::write(directory.join(format!("stateful-{phase}.json")),serde_json::to_vec(&result).unwrap()).unwrap();
            return;
        }
        if let Ok(fault)=std::env::var("DDN_M2_COLD_READMIT") {
            let loaded=grant.load_initial_attempt(&adapter,&source,&request,&token).unwrap();
            if fault=="confirm" || fault=="stateful" {super::super::FAIL_CUSTODY_CONFIRM.with(|point|point.set(true));}
            if fault=="confirm" || fault=="reread" {super::super::FAIL_CUSTODY_COMMITTED_REREAD.with(|point|point.set(true));}
            if fault=="chain" {
                // A mutually matching forged head/revision pair must still
                // fail against the original admitted payload. Defer the FK
                // only inside this test-owned corruption transaction; the
                // earlier single-row probe was rejected by SQLite itself.
                adapter.connect().unwrap().execute_batch("PRAGMA defer_foreign_keys=ON; BEGIN IMMEDIATE;
                    UPDATE vc04_project_revisions SET revision_integrity_sha256=zeroblob(32);
                    UPDATE vc04_project_heads SET revision_integrity_sha256=zeroblob(32); COMMIT;").unwrap();
            }
            let report=match owner::readmit_initial_committed_native(&adapter,&source,loaded) {
                Ok(admitted)=>{
                    let payload=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2(&request,source.pesi().as_raw()).unwrap();
                    let receipt=crate::snapshot_v2_host_bridge::encode_committed_receipt_candidate(&payload.binding,b"vc04.sqlite-durability.r1").unwrap();
                    admitted.evidence().check_committed_receipt_evidence(&receipt,b"vc04.sqlite-durability.r1").unwrap();
                    serde_json::json!({"state":admitted.evidence().state() as u8,"receipt_binding_checked":true,"world_published":false})
                },
                Err(blocked)=>serde_json::json!({"historical_state":blocked.prior.historical_state() as u8,
                    "original_cause":blocked.prior.original_cause() as u16,"error":format!("{:?}",blocked.failure),"world_published":false}),
            };
            if fault=="confirm" {assert!(super::super::FAIL_CUSTODY_COMMITTED_REREAD.with(|point|point.replace(false)),"failed fresh confirmation must not issue committed reread");}
            if fault=="stateful" {assert!(super::super::FAIL_CUSTODY_CONFIRM.with(|point|point.replace(false)),"RecoveryRequired cannot enter a read-only terminal shortcut");}
            if fault=="reread" {assert!(!super::super::FAIL_CUSTODY_COMMITTED_REREAD.with(|point|point.get()));}
            for table in ["vc04_project_heads","vc04_project_revisions"] {
                assert_eq!(adapter.connect().unwrap().query_row(&format!("SELECT count(*) FROM {table}"),[],|row|row.get::<_,i64>(0)).unwrap(),1,"read-only retry creates no new head/revision");
            }
            std::fs::write(directory.join(format!("readmit-{fault}.json")),serde_json::to_vec(&report).unwrap()).unwrap();
            return;
        }
        let report=match grant.load_initial_attempt(&adapter,&source,&request,&token) {
            Ok(loaded)=>serde_json::json!({"state":loaded.historical_state() as u8,"cause":loaded.original_cause() as u16,"world_published":false}),
            Err(failure)=>serde_json::json!({"error":format!("{failure:?}"),"world_published":false}),
        };
        std::fs::write(directory.join("cold-result.json"),serde_json::to_vec(&report).unwrap()).unwrap();
    }

    #[test]
    #[ignore="explicit process fixture issuer; never a production fault hook"]
    fn m2_actual_runtime_fault_fixture_child() {
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::{self as owner,retained_attempt::ActualPreparedRequestV1,SnapshotV2TransactionStateV1 as State};
        let directory=std::path::PathBuf::from(std::env::var_os("DDN_M2_RUNTIME_FIXTURE_DIR").expect("exact test-owned fixture directory"));
        let mode=std::env::var("DDN_M2_RUNTIME_FIXTURE_MODE").unwrap();
        assert!(["recover","pending","lost","aborted","inspect"].contains(&mode.as_str()));
        if mode=="inspect" {
            let bytes=std::fs::read(directory.join("saved.json")).unwrap();
            let checked=attachment::check_bytes(&bytes).unwrap();
            let owned=attachment::restore_saved_runtime_payload(&checked).unwrap();
            let source=attachment::check_saved_revision_context(&checked,&owned,&[],&[9]).unwrap();
            let mut adapter=super::super::SqliteAdapterV1::open_recovery_custody(&directory.join("root")).unwrap();
            let locator=std::fs::read_to_string(directory.join("locator.txt")).unwrap();
            let grant=adapter.confirm_saved_root_grant(&locator,&source).unwrap();
            let metadata=GrantRecord::decode(&grant.record.encode().unwrap()).unwrap();
            let token:[u8;33]=std::fs::read(directory.join("token.bin")).unwrap().try_into().unwrap();
            let raw=std::fs::read(directory.join("request.bin")).unwrap();
            let outcome=match grant.load_initial_attempt(&adapter,&source,&raw,&token) {
                Ok(loaded)=>serde_json::json!({"state":format!("{:?}",loaded.historical_state()),"original":format!("{:?}",loaded.original_cause())}),
                Err(failure)=>serde_json::json!({"rejected":format!("{failure:?}")}),
            };
            let anchor=WindowsRootGrantStoreV1::read_attempt(&metadata).unwrap();
            std::fs::write(directory.join("inspect.json"),serde_json::to_vec(&serde_json::json!({"outcome":outcome,"epoch":anchor.epoch,"mode":format!("{:?}",anchor.mode)})).unwrap()).unwrap();
            return;
        }
        let authored=std::fs::read_to_string(directory.join("authored.json")).unwrap();
        let run=crate::runtime_surface::run_authored_general_input_prefix(&authored,3,0,None,None).unwrap();
        let saved=run.export_saved_artifact(&[],&[9]).unwrap();
        let checked=attachment::check_bytes(saved.as_bytes()).unwrap();
        let source=attachment::check_saved_revision_context(&checked,run.source_owner(),&[],&[9]).unwrap();
        let root=directory.join("root");std::fs::create_dir(&root).unwrap();
        let mut adapter=super::super::SqliteAdapterV1::provision_recovery_custody(&root).unwrap();
        let grant=adapter.register_saved_root_grant(&source).unwrap();
        let request=ActualPreparedRequestV1::from_general(&run,&source,&[],&[9]).unwrap();
        let raw=request.raw().to_vec();let mut token=[0u8;33];token[0]=1;token[1..].copy_from_slice(&request.payload().digest);
        std::fs::write(directory.join("saved.json"),saved.as_bytes()).unwrap();
        std::fs::write(directory.join("request.bin"),raw).unwrap();
        std::fs::write(directory.join("token.bin"),token).unwrap();
        std::fs::write(directory.join("locator.txt"),grant.locator()).unwrap();
        let start=grant.start_initial_attempt(&adapter,&source,request).unwrap();
        match mode.as_str() {
            "recover"|"lost"=>super::super::FAIL_CUSTODY_CONFIRM.with(|fault|fault.set(true)),
            "aborted"=>super::super::FAIL_CUSTODY_SYNC_ACK.with(|fault|fault.set(true)),
            _=>{},
        }
        let actual=owner::run_initial_native_owner_with_custody(&adapter,&source,start).unwrap();
        let state=actual.owner.evidence().state();
        let original=actual.owner.failure().map(|failure|format!("{:?}",failure.cause));
        if mode=="pending" {IO_FAULT.with(|fault|fault.set(Some((Namespace::Attempt,IoStep::Set))));}
        let retention=match actual.retain(&adapter,&source) {
            Ok(_)=>{assert_ne!(mode,"pending");"retained".to_string()},
            Err(blocked)=>{assert_eq!(mode,"pending");assert_eq!(blocked.actual.owner.evidence().state(),State::Committed);blocked.diagnostic()},
        };
        if mode=="lost" {
            let connection=adapter.connect().unwrap();
            let raw:Vec<u8>=connection.query_row("SELECT recovery_record_bytes FROM vc04_recovery_custody_v1 WHERE journal_ordinal=2",[],|row|row.get(0)).unwrap();
            std::fs::write(directory.join("lost-record.bin"),raw).unwrap();
            assert_eq!(connection.execute("DELETE FROM vc04_recovery_custody_v1 WHERE journal_ordinal=2",[]).unwrap(),1);
        }
        std::fs::write(directory.join("FIXTURE.json"),serde_json::to_vec(&serde_json::json!({"mode":mode,"state":format!("{state:?}"),"original":original,"retention":retention,"world_published":false})).unwrap()).unwrap();
    }

    #[test]
    fn m2_actual_stateful_recovery_cold_epochs_preserve_original_cause() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::{self as owner,retained_attempt::ActualPreparedRequestV1,SnapshotV2TransactionStateV1 as State};
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:vec![SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n".into()},
                SourceFile{path:"app/main.ddn".into(),text:"실행:움직씨 := { 1 보여주기. }.\n".into()}],
        },String::new()).unwrap().save().unwrap();
        let run=crate::runtime_surface::run_authored_general_input_prefix(&artifact,1,17,None,None).unwrap();
        let saved=run.export_saved_artifact(&[],&[2]).unwrap();
        let bytes=attachment::check_bytes(saved.as_bytes()).unwrap();
        let source=attachment::check_saved_revision_context(&bytes,run.source_owner(),&[],&[2]).unwrap();
        for mode in 0..7 {
            let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
            let directory=std::env::temp_dir().join(format!("ddn-m2-stateful-{}",hex::encode(nonce)));
            std::fs::create_dir(&directory).unwrap();let root=directory.join("root");std::fs::create_dir(&root).unwrap();
            let mut adapter=super::super::SqliteAdapterV1::provision_recovery_custody(&root).unwrap();
            let grant=adapter.register_saved_root_grant(&source).unwrap();let locator=grant.locator();
            let request=ActualPreparedRequestV1::from_general(&run,&source,&[],&[2]).unwrap();
            let raw=request.raw().to_vec();let digest=request.payload().digest;
            let mut token=[0u8;33];token[0]=1;token[1..].copy_from_slice(&digest);
            let start=grant.start_initial_attempt(&adapter,&source,request).unwrap();
            if mode==1 {super::super::FAIL_CUSTODY_SYNC_ACK.with(|fault|fault.set(true));}
            else{super::super::FAIL_CUSTODY_CONFIRM.with(|fault|fault.set(true));}
            let actual=owner::run_initial_native_owner_with_custody(&adapter,&source,start).unwrap();
            assert_eq!(actual.owner.evidence().state(),State::RecoveryRequired);
            let original=if mode==1 {Failure::RecoveryRequired}else{Failure::ContainerDurabilityFailed};
            assert_eq!(actual.owner.failure().unwrap().cause,original);
            let retained=actual.retain(&adapter,&source).ok().expect("retain real original failure");drop(retained);
            std::fs::write(directory.join("artifact.json"),saved.as_bytes()).unwrap();std::fs::write(directory.join("request.bin"),raw).unwrap();
            std::fs::write(directory.join("token.bin"),token).unwrap();std::fs::write(directory.join("locator.txt"),&locator).unwrap();drop(adapter);
            let probe=|phase:&str,fault:&str|->serde_json::Value {
                let child=std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["--exact","snapshot_v2_durable_transaction::native::root_grant::windows::m2_actual_cold_witness_child","--ignored","--nocapture"])
                    .env("DDN_M2_COLD_WITNESS_DIR",&directory).env("DDN_M2_STATEFUL_PHASE",phase).env("DDN_M2_STATEFUL_FAULT",fault).output().unwrap();
                assert!(child.status.success(),"stateful child: {}\n{}",String::from_utf8_lossy(&child.stdout),String::from_utf8_lossy(&child.stderr));
                println!("M2_STATEFUL_CHILD mode{mode} phase{phase}: {}",String::from_utf8_lossy(&child.stdout));
                serde_json::from_slice(&std::fs::read(directory.join(format!("stateful-{phase}.json"))).unwrap()).unwrap()
            };
            let fault=match mode {2=>"reread",3=>"reserve-flush",4=>"append",5=>"witness",_=>""};
            let first=probe("first",fault);
            assert_eq!(first["epoch"],2);
            if mode==3 {
                assert_eq!(first["result"]["start_error"],"SyncFailed");assert_eq!(first["mode"],"Pending");assert_eq!(first["rows"],3);
            }else{
                assert_eq!(first["result"]["primary"],original as u16);
                assert_eq!(first["result"]["state"],match mode {1=>5,2=>6,_=>4});
                if mode==2 {assert_eq!(first["result"]["attempt_failure"],"RereadFailed");}
                if mode==4 || mode==5 {assert_eq!(first["result"]["retention_error"],"WriteFailed");assert_eq!(first["mode"],"Pending");}
                else{assert_eq!(first["result"]["retained"],true);assert_eq!(first["mode"],"RecordWitness");}
            }
            if mode==2 {
                let second=probe("second","");assert_eq!(second["epoch"],3);assert_eq!(second["result"]["state"],4);
                assert_eq!(second["result"]["primary"],original as u16);assert_eq!(second["result"]["retained"],true);
            }
            if mode==6 {
                let adapter=super::super::SqliteAdapterV1::open_recovery_custody(&root).unwrap();let connection=adapter.connect().unwrap();
                let raw:Vec<u8>=connection.query_row("SELECT recovery_record_bytes FROM vc04_recovery_custody_v1 WHERE journal_ordinal=3",[],|row|row.get(0)).unwrap();
                std::fs::write(directory.join("lost-recovery-original.bin"),raw).unwrap();
                assert_eq!(connection.execute("DELETE FROM vc04_recovery_custody_v1 WHERE journal_ordinal=3",[]).unwrap(),1);drop(connection);drop(adapter);
            }
            let loaded=probe("load","");
            if (3..=5).contains(&mode) {assert_eq!(loaded["result"]["load_error"],"RecoveryRequired");}
            else if mode==6 {assert_eq!(loaded["result"]["load_error"],"CustodyConflict");}
            else{assert_eq!(loaded["result"]["state"],if mode==1 {5}else{4});assert_eq!(loaded["result"]["primary"],original as u16);}
            assert_eq!(loaded["result"]["world_published"],false);
            println!("retained stateful recovery fixture: {}",directory.display());
        }
    }

    #[test]
    fn r4_root_grant_windows_storage_mechanics_only() {
        let mut record = GrantRecord::decode(super::tests::FIXTURE).unwrap();
        getrandom::fill(&mut record.nonce).expect("OS CSPRNG for test-owned nonce");
        let plaintext = record.encode().unwrap();
        let ciphertext = protect(&plaintext).expect("user then machine DPAPI");
        assert!(ciphertext.len() <= MAX_CIPHERTEXT_BYTES);
        assert_eq!(unprotect(&ciphertext).unwrap(), plaintext);
        let mut corrupt = ciphertext;
        let last = corrupt.len() - 1;
        corrupt[last] ^= 0x80;
        assert!(unprotect(&corrupt).is_err());
        // This exercises OS storage, not root registration: fixture object/SID
        // fields are not actual pinning and cannot issue an admitted capability.
        WindowsRootGrantStoreV1::write_verified(&record, true).expect("Set, Flush, fresh open and decrypt");
        assert!(WindowsRootGrantStoreV1::read(&record.nonce).unwrap() == record);
        let revoked = record.revoked().unwrap();
        WindowsRootGrantStoreV1::write_verified(&revoked, false).expect("persist revocation tombstone");
        assert!(WindowsRootGrantStoreV1::read(&record.nonce).unwrap() == revoked);
        // Keep the exact random task-owned tombstone; no key enumeration/delete.
        println!("test-owned revoked registry locator: {}", hex::encode(record.nonce));
    }

    #[test]
    fn m2_actual_producer_pending_start_precedes_sql_and_refuses_retry_after_partial_ack() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::retained_attempt::ActualPreparedRequestV1;
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:vec![SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n".into()},
                SourceFile{path:"app/main.ddn".into(),text:"실행:움직씨 := { 1 보여주기. }.\n".into()}],
        },String::new()).unwrap().save().unwrap();
        let run=crate::runtime_surface::run_authored_general_input_prefix(&artifact,1,17,None,None).unwrap();
        let saved=run.export_saved_artifact(&[],&[2]).unwrap();
        let saved_bytes=attachment::check_bytes(saved.as_bytes()).unwrap();
        let source=attachment::check_saved_revision_context(&saved_bytes,run.source_owner(),&[],&[2]).unwrap();
        for fault in [None,Some(IoStep::Set),Some(IoStep::Flush),Some(IoStep::FreshRead)] {
            let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
            let root=std::env::temp_dir().join(format!("ddn-m2-actual-start-{}",hex::encode(nonce)));
            std::fs::create_dir(&root).unwrap();
            let mut adapter=super::super::SqliteAdapterV1::provision_recovery_custody(&root).unwrap();
            let grant=adapter.register_saved_root_grant(&source).unwrap();
            let locator=grant.locator();
            let request=ActualPreparedRequestV1::from_general(&run,&source,&[],&[2]).unwrap();
            let digest=request.binding().transaction_digest().unwrap();
            IO_FAULT.with(|point|point.set(fault.map(|step|(Namespace::Attempt,step))));
            let start=grant.start_initial_attempt(&adapter,&source,request);
            assert!(IO_FAULT.with(|point|point.get().is_none()));
            if let Some(step)=fault {
                let error=start.err().expect("no start capability without all three host acknowledgements");
                assert_eq!(error,match step {IoStep::Set=>Failure::WriteFailed,IoStep::Flush=>Failure::SyncFailed,IoStep::FreshRead=>Failure::RereadFailed});
            }else{
                let start=start.unwrap();start.check_current(&adapter,&source).unwrap();
                assert_eq!(start.request().payload().digest,digest);
                assert_eq!(start.attempt.mode,AttemptMode::Pending);
                assert_eq!(start.attempt.epoch,1);
                // No SQL writer has consumed this capability yet.
                drop(start);
            }
            for table in ["vc04_prepared_payloads","vc04_project_revisions","vc04_project_heads","vc04_recovery_custody_v1"] {
                let count:i64=adapter.connect().unwrap().query_row(&format!("SELECT count(*) FROM {table}"),[],|row|row.get(0)).unwrap();
                assert_eq!(count,0,"SQL changed before a start capability was consumed");
            }
            if !matches!(fault,Some(IoStep::Set)) {
                let again=adapter.confirm_saved_root_grant(&locator,&source).unwrap();
                let request=ActualPreparedRequestV1::from_general(&run,&source,&[],&[2]).unwrap();
                assert!(matches!(again.start_initial_attempt(&adapter,&source,request),Err(Failure::ConflictingRetry)),"visible Pending cannot be cleared by a new start");
            }
            println!("retained actual producer Pending root: {}",root.display());
        }
    }

    #[test]
    fn m2_actual_payload_and_initial_custody_commit_or_rollback_together() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::retained_attempt::{ActualPreparedRequestV1,InitialCustodyWriteV1};
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:vec![SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n".into()},
                SourceFile{path:"app/main.ddn".into(),text:"실행:움직씨 := { 1 보여주기. }.\n".into()}],
        },String::new()).unwrap().save().unwrap();
        let run=crate::runtime_surface::run_authored_general_input_prefix(&artifact,1,17,None,None).unwrap();
        let saved=run.export_saved_artifact(&[],&[2]).unwrap();
        let bytes=attachment::check_bytes(saved.as_bytes()).unwrap();
        let source=attachment::check_saved_revision_context(&bytes,run.source_owner(),&[],&[2]).unwrap();
        for mode in [0,1,2] {
            let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
            let root=std::env::temp_dir().join(format!("ddn-m2-atomic-initial-{}",hex::encode(nonce)));
            std::fs::create_dir(&root).unwrap();
            let mut adapter=super::super::SqliteAdapterV1::provision_recovery_custody(&root).unwrap();
            let grant=adapter.register_saved_root_grant(&source).unwrap();
            let grant_record=GrantRecord::decode(&grant.record.encode().unwrap()).unwrap();
            let request=ActualPreparedRequestV1::from_general(&run,&source,&[],&[2]).unwrap();
            let payload=request.payload().clone();
            let start=grant.start_initial_attempt(&adapter,&source,request).unwrap();
            let pending=start.attempt.encode().unwrap();
            if mode==2 {super::super::FAIL_INITIAL_CUSTODY_INSERT.with(|fault|fault.set(true));}
            let write=InitialCustodyWriteV1::begin(&adapter,&source,start);
            if mode==2 {
                assert!(matches!(write,Err(Failure::WriteFailed)));
                assert!(!super::super::FAIL_INITIAL_CUSTODY_INSERT.with(|fault|fault.get()));
            }else{
                let write=write.unwrap();
                for table in ["vc04_prepared_payloads","vc04_recovery_custody_v1"] {
                    let count:i64=adapter.connect().unwrap().query_row(&format!("SELECT count(*) FROM {table}"),[],|row|row.get(0)).unwrap();
                    assert_eq!(count,0,"uncommitted half must not become visible");
                }
                if mode==1 {drop(write);}else{
                    let prepared=write.sync_and_reread(&adapter,&source).unwrap();
                    prepared.revalidate_prepared(&adapter,&source).unwrap();
                    assert_eq!(adapter.reread_prepared(&payload.digest).unwrap(),Some(payload.clone()));
                    let connection=adapter.connect().unwrap();
                    let count:i64=connection.query_row("SELECT count(*) FROM vc04_recovery_custody_v1",[],|row|row.get(0)).unwrap();
                    assert_eq!(count,1);
                    // Mutation is confined to this task-owned test container.
                    // It must not be ignored or repaired from the good payload.
                    connection.execute("UPDATE vc04_recovery_custody_v1 SET row_sha256=zeroblob(32)",[]).unwrap();
                    assert_eq!(prepared.revalidate_prepared(&adapter,&source),Err(Failure::RereadMismatch));
                    drop(prepared);
                }
            }
            for table in ["vc04_prepared_payloads","vc04_recovery_custody_v1"] {
                let count:i64=adapter.connect().unwrap().query_row(&format!("SELECT count(*) FROM {table}"),[],|row|row.get(0)).unwrap();
                assert_eq!(count,if mode==0 {1}else{0},"row0 and payload must commit/rollback together");
            }
            for table in ["vc04_project_revisions","vc04_project_heads"] {
                let count:i64=adapter.connect().unwrap().query_row(&format!("SELECT count(*) FROM {table}"),[],|row|row.get(0)).unwrap();
                assert_eq!(count,0,"prepared data is not a committed revision");
            }
            assert_eq!(WindowsRootGrantStoreV1::read_attempt(&grant_record).unwrap().encode().unwrap(),pending,
                "rollback/corruption must not clear Pending or mint a Witness");
            println!("retained atomic initial-custody root: {}",root.display());
        }
    }

    #[test]
    fn m2_actual_verified_queue_precedes_cas_and_unretained_outcome_cannot_issue_witness() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        use crate::unit_wire::project_runtime_attachment as attachment;
        use crate::snapshot_v2_durable_transaction::{self as owner,SnapshotV2TransactionStateV1 as State,SnapshotV2DurabilityAdapterV1 as Adapter};
        use owner::retained_attempt::{ActualPreparedRequestV1,NativeCustodySessionV1};
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:vec![SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n".into()},
                SourceFile{path:"app/main.ddn".into(),text:"실행:움직씨 := { 1 보여주기. }.\n".into()}],
        },String::new()).unwrap().save().unwrap();
        let run=crate::runtime_surface::run_authored_general_input_prefix(&artifact,1,17,None,None).unwrap();
        let saved=run.export_saved_artifact(&[],&[2]).unwrap();
        let bytes=attachment::check_bytes(saved.as_bytes()).unwrap();
        let source=attachment::check_saved_revision_context(&bytes,run.source_owner(),&[],&[2]).unwrap();
        for mode in [0,1,2] {
            let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
            let root=std::env::temp_dir().join(format!("ddn-m2-verified-owner-{}",hex::encode(nonce)));
            std::fs::create_dir(&root).unwrap();
            let mut adapter=super::super::SqliteAdapterV1::provision_recovery_custody(&root).unwrap();
            let grant=adapter.register_saved_root_grant(&source).unwrap();
            let record=GrantRecord::decode(&grant.record.encode().unwrap()).unwrap();
            let request=ActualPreparedRequestV1::from_general(&run,&source,&[],&[2]).unwrap();
            let payload=request.payload().clone();
            let start=grant.start_initial_attempt(&adapter,&source,request).unwrap();
            let pending=start.attempt.encode().unwrap();
            if mode==2 {
                let session=NativeCustodySessionV1::new(&adapter,&source,start).unwrap();
                let mut handle=Adapter::write_prepared(&session,&payload).unwrap();
                Adapter::sync_prepared(&session,&mut handle).unwrap();
                Adapter::reread_prepared(&session,&payload.digest).unwrap();
                // No common owner Verified evidence/queue was supplied.
                assert_eq!(Adapter::commit_verified(&session,&payload,None),Err(Failure::CustodyConflict));
                assert_eq!(Adapter::commit_verified(&session,&payload,None),Err(Failure::CustodyConflict));
            }else{
                if mode==1 {super::super::FAIL_VERIFIED_CUSTODY_INSERT.with(|fault|fault.set(true));}
                let outcome=owner::run_initial_native_owner_with_custody(&adapter,&source,start).unwrap();
                if mode==0 {
                    assert_eq!(outcome.owner.evidence().state(),State::Committed);
                    assert!(outcome.owner.failure().is_none());
                }else{
                    assert!(!super::super::FAIL_VERIFIED_CUSTODY_INSERT.with(|fault|fault.get()));
                    assert_eq!(outcome.owner.evidence().state(),State::Aborted);
                    assert_eq!(outcome.owner.failure().unwrap().cause,Failure::WriteFailed);
                }
                // Even actual Committed owner evidence is deliberately wrapped
                // in an unretained result. No record Witness is minted here.
                outcome.start.check_current(&adapter,&source).unwrap();
                outcome.initial.check_request(outcome.start.request()).unwrap();
                outcome.issued_verified.as_ref().expect("original issued bytes survive an append failure")
                    .check_initial(&outcome.initial,outcome.start.request()).unwrap();
                assert_eq!(outcome.append_completed.is_some(),mode==0,
                    "retention must distinguish acknowledged prefix from an uncertain append, not heal from a shorter prefix");
            }
            let connection=adapter.connect().unwrap();
            let count:i64=connection.query_row("SELECT count(*) FROM vc04_recovery_custody_v1",[],|row|row.get(0)).unwrap();
            assert_eq!(count,if mode==0 {2}else{1});
            for table in ["vc04_project_heads","vc04_project_revisions"] {
                let count:i64=connection.query_row(&format!("SELECT count(*) FROM {table}"),[],|row|row.get(0)).unwrap();
                assert_eq!(count,if mode==0 {1}else{0},"no CAS before a retained Verified row");
            }
            assert_eq!(WindowsRootGrantStoreV1::read_attempt(&record).unwrap().encode().unwrap(),pending,
                "unretained owner state must not be projected as a host Witness");
            println!("retained actual Verified-owner root: {}",root.display());
        }
    }

    #[test]
    fn r5_attempt_pending_io_failure_cannot_overwrite_visible_barrier() {
        for step in [IoStep::Set,IoStep::Flush,IoStep::FreshRead] {
            let mut grant=GrantRecord::decode(super::tests::FIXTURE).unwrap();
            getrandom::fill(&mut grant.nonce).unwrap();
            WindowsRootGrantStoreV1::write_verified(&grant,true).unwrap();
            let transaction=[31;32];let mut token=vec![1];token.extend(transaction);
            let pending=AttemptRecord::initial_pending(&grant,transaction,b"OS-store mechanics fixture only",&token).unwrap();
            IO_FAULT.with(|fault|fault.set(Some((Namespace::Attempt,step))));
            let error=WindowsRootGrantStoreV1::write_first_pending(&grant,&pending).unwrap_err();
            assert!(IO_FAULT.with(|fault|fault.get().is_none()));
            match step {
                IoStep::Set=>assert!(matches!(error,Failure::WriteFailed)),
                IoStep::Flush=>assert!(matches!(error,Failure::SyncFailed)),
                IoStep::FreshRead=>assert!(matches!(error,Failure::RereadFailed)),
            }
            if step==IoStep::Set {
                assert!(WindowsRootGrantStoreV1::read_attempt(&grant).is_err());
                WindowsRootGrantStoreV1::write_first_pending(&grant,&pending).unwrap();
            }
            assert_eq!(WindowsRootGrantStoreV1::read_attempt(&grant).unwrap(),pending);
            assert!(matches!(WindowsRootGrantStoreV1::write_first_pending(&grant,&pending),Err(Failure::ConflictingRetry)));
            let revoked=grant.revoked().unwrap();WindowsRootGrantStoreV1::write_verified(&revoked,false).unwrap();
            assert!(WindowsRootGrantStoreV1::read_attempt(&revoked).is_err());
            println!("R5 retained test-only Pending/tombstone locator {}",hex::encode(grant.nonce));
        }
        // Real DPAPI/registry I/O, but not checked request/start/Witness/cold
        // transaction recovery. No public issuer consumes this helper yet.
    }

    #[test]
    fn r4_root_grant_pins_actual_root_db_and_sqlite_connection() {
        let mut nonce = [0u8; 32];
        getrandom::fill(&mut nonce).unwrap();
        let root = std::env::temp_dir().join(format!("ddn-rd669-pin-{}", hex::encode(nonce)));
        std::fs::create_dir(&root).unwrap();
        let mut adapter = super::super::SqliteAdapterV1::provision(&root).unwrap();
        // No IDs have been manufactured for a cold reopen.
        assert!(PinnedNativeRootV1::capture(&adapter.path, false).is_err());
        adapter.pin_objects_for_grant(true).unwrap();
        let pinned = adapter.pinned_root.as_ref().unwrap();
        pinned.verify_connection(&adapter.connect().unwrap()).unwrap();
        assert!(std::fs::rename(&adapter.path, root.join("swapped.sqlite3")).is_err());
        let foreign = root.join("foreign.sqlite3");
        let connection = rusqlite::Connection::open(&foreign).unwrap();
        assert!(pinned.verify_connection(&connection).is_err());
        drop(connection);
        drop(adapter);
        let mut reopened = super::super::SqliteAdapterV1::open_admitted(&root).unwrap();
        reopened.pin_objects_for_grant(false).unwrap();
        reopened.pinned_root.as_ref().unwrap().verify_connection(&reopened.connect().unwrap()).unwrap();
        // Retain task-owned fixture files and ObjectIds; no deletion/rewrites.
        println!("test-owned retained pinned root: {}", root.display());
    }

    #[test]
    fn r4_root_grant_fresh_token_cannot_cross_physical_containers() {
        let mut nonce=[0u8;32]; getrandom::fill(&mut nonce).unwrap();
        let directory=std::env::temp_dir().join(format!("ddn-rd669-fresh-{}",hex::encode(nonce)));
        std::fs::create_dir(&directory).unwrap();
        let a=directory.join("a"); let b=directory.join("b");
        std::fs::create_dir(&a).unwrap(); std::fs::create_dir(&b).unwrap();
        let mut first=super::super::SqliteAdapterV1::provision(&a).unwrap();
        let mut second=super::super::SqliteAdapterV1::provision(&b).unwrap();
        let token=first.fresh_container.get_mut().take().unwrap();
        first.pin_objects_for_grant(true).unwrap(); second.pin_objects_for_grant(true).unwrap();
        assert!(token.verify_pinned(first.pinned_root.as_ref().unwrap()).is_ok());
        assert!(token.verify_pinned(second.pinned_root.as_ref().unwrap()).is_err());
        drop(token); drop(first);
        let reopened=super::super::SqliteAdapterV1::open_admitted(&a).unwrap();
        assert!(reopened.fresh_container.borrow().is_none(),"opening an empty DB does not mint freshness");
        println!("test-owned retained fresh token fixtures: {}",directory.display());
    }

    #[test]
    fn r4_root_grant_partial_slot_flush_never_admits() {
        use crate::unit_wire::project_runtime_attachment as attachment;
        let artifact=attachment::tests::root_grant_saved_fixture();
        let bytes=attachment::check_bytes(artifact.as_bytes()).unwrap();
        let owned=attachment::restore_saved_runtime_payload(&bytes).unwrap();
        let context=attachment::check_saved_revision_context(&bytes,&owned,&[],&[2]).unwrap();
        let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
        let directory=std::env::temp_dir().join(format!("ddn-rd669-slot-fault-{}",hex::encode(nonce)));
        std::fs::create_dir(&directory).unwrap();
        let root=directory.join("store");std::fs::create_dir(&root).unwrap();
        std::fs::write(directory.join("saved-project.json"),&artifact).unwrap();
        let mut adapter=super::super::SqliteAdapterV1::provision(&root).unwrap();
        // Real SetValue succeeds and becomes visible; simulate Flush failure
        // before an acknowledgement. No production fault switch exists.
        IO_FAULT.with(|point|point.set(Some((Namespace::RootSlot,IoStep::Flush))));
        let error=adapter.register_saved_root_grant(&context).err().expect("no capability after missing flush acknowledgement");
        IO_FAULT.with(|point|point.set(None));
        assert!(matches!(error,Failure::SyncFailed));
        let pinned=adapter.pinned_root.as_ref().unwrap();
        let observed=GrantRecord {nonce:[1;32],sid_hash:current_sid_hash().unwrap(),project_hash:source_project_hash(&context).unwrap(),
            registration_pesi:*context.pesi().as_raw(),root_file:pinned.root_file,root_object:pinned.root_object,
            db_file:pinned.db_file,db_object:pinned.db_object,lifecycle:Lifecycle::Active,generation:0};
        // Locator recovery here is privileged test observation, never a
        // product fallback or an admission proof.
        let locator=hex::encode(WindowsRootGrantStoreV1::read_slot(&root_fingerprint(&observed)).unwrap());
        drop(adapter);
        let mut reopened=super::super::SqliteAdapterV1::open_admitted(&root).unwrap();
        let admission=reopened.reopen_saved_root_grant(&locator,&context);
        let wrongly_admitted=admission.is_ok();
        if let Ok(grant)=admission { grant.revoke(&reopened,&context).unwrap(); }
        println!("R6_SLOT_FLUSH_FAILED_BUT_REOPEN_ADMITTED={wrongly_admitted}");
        println!("test-owned retained partial-slot fixture: {} / observed locator {locator}",directory.display());
        assert!(!wrongly_admitted,"partial registration must not be healed by visible grant+slot bytes");
        let confirmed=reopened.confirm_saved_root_grant(&locator,&context).expect("explicit current confirmation is a separate operation");
        assert!(matches!(error,Failure::SyncFailed),"the original registration error is not rewritten");
        confirmed.revoke(&reopened,&context).unwrap();
        println!("R7_EXPLICIT_CURRENT_CONFIRMATION_AFTER_REGISTRATION_ERROR=true");
    }

    #[test]
    fn r4_root_grant_all_partial_registration_windows_require_explicit_cold_confirmation() {
        use crate::unit_wire::project_runtime_attachment as attachment;
        let artifact=attachment::tests::root_grant_saved_fixture();
        let bytes=attachment::check_bytes(artifact.as_bytes()).unwrap();
        let owned=attachment::restore_saved_runtime_payload(&bytes).unwrap();
        let context=attachment::check_saved_revision_context(&bytes,&owned,&[],&[2]).unwrap();
        let mut nonce=[0u8;32];getrandom::fill(&mut nonce).unwrap();
        let directory=std::env::temp_dir().join(format!("ddn-rd669-six-windows-{}",hex::encode(nonce)));
        std::fs::create_dir(&directory).unwrap();
        let source=directory.join("saved-project.json");std::fs::write(&source,&artifact).unwrap();
        for (index,(namespace,step)) in [
            (Namespace::Grant,IoStep::Set),(Namespace::Grant,IoStep::Flush),(Namespace::Grant,IoStep::FreshRead),
            (Namespace::RootSlot,IoStep::Set),(Namespace::RootSlot,IoStep::Flush),(Namespace::RootSlot,IoStep::FreshRead),
        ].into_iter().enumerate() {
            let root=directory.join(format!("window-{index}"));std::fs::create_dir(&root).unwrap();
            let mut adapter=super::super::SqliteAdapterV1::provision(&root).unwrap();
            LAST_REGISTRATION_NONCE.with(|last|last.set(None));
            IO_FAULT.with(|fault|fault.set(Some((namespace,step))));
            let original=adapter.register_saved_root_grant(&context).err().expect("registration failure must not publish a grant");
            assert!(IO_FAULT.with(|fault|fault.get().is_none()),"actual IO boundary must be reached");
            match step {
                IoStep::Set=>assert!(matches!(original,Failure::WriteFailed)),
                IoStep::Flush=>assert!(matches!(original,Failure::SyncFailed)),
                IoStep::FreshRead=>assert!(matches!(original,Failure::RereadFailed)),
            }
            assert!(adapter.fresh_container.borrow().is_none(),"failed registration consumes the fresh token");
            let locator=hex::encode(LAST_REGISTRATION_NONCE.with(|last|last.get()).expect("test-only OS nonce observation"));
            drop(adapter);
            attachment::tests::root_grant_cold_probe(&root,&source,&locator,"read-denied");
            let complete_bytes=namespace==Namespace::RootSlot && step!=IoStep::Set;
            if complete_bytes {
                for fault in ["fault:grant-flush","fault:slot-flush","fault:grant-read","fault:slot-read"] {
                    attachment::tests::root_grant_cold_probe(&root,&source,&locator,fault);
                }
            }
            attachment::tests::root_grant_cold_probe(&root,&source,&locator,if complete_bytes {"active"}else{"reject"});
            // Current confirmation is separate: no earlier Err is rewritten,
            // no missing grant/slot is repaired, and no new nonce is generated.
            if complete_bytes {
                let mut adapter=super::super::SqliteAdapterV1::open_admitted(&root).unwrap();
                assert_eq!(adapter.revoke_saved_root_grant(&locator,&context).unwrap(),locator);
            }
            println!("R7_REGISTRATION_WINDOW index={index} original={original:?} explicit_current_allowed={complete_bytes}");
        }
        println!("R7 retained six-window fixtures: {}",directory.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) const FIXTURE: &[u8] = include_bytes!("root_grant_fixture.bin");

    #[test]
    fn r5_attempt_record_closed_bytes_and_missing_witness_are_not_success() {
        let grant=GrantRecord::decode(FIXTURE).unwrap();
        let transaction=[23;32];let mut token=vec![1];token.extend(transaction);
        let pending=AttemptRecord::initial_pending(&grant,transaction,b"codec-only request not admitted",&token).unwrap();
        let bytes=pending.encode().unwrap();assert_eq!(bytes.len(),251);
        assert_eq!(AttemptRecord::decode(&bytes).unwrap(),pending);
        assert_eq!(pending.epoch,1);assert_eq!(pending.mode,AttemptMode::Pending);
        for length in [0,1,250] {assert!(AttemptRecord::decode(&bytes[..length]).is_err());}
        let mut trailing=bytes.clone();trailing.push(0);assert!(AttemptRecord::decode(&trailing).is_err());
        let mut cursor=ATTEMPT_MAGIC.len()+3;
        for length in ATTEMPT_LENGTHS {
            let mut wrong_tag=bytes.clone();wrong_tag[cursor]^=0x40;assert!(AttemptRecord::decode(&wrong_tag).is_err());
            let mut wrong_length=bytes.clone();wrong_length[cursor+4]^=1;assert!(AttemptRecord::decode(&wrong_length).is_err());
            cursor+=5+length;
        }
        let mut bad=pending.clone();bad.epoch=0;assert!(bad.encode().is_err());
        bad=pending.clone();bad.record_hash=[1;32];assert!(bad.encode().is_err());
        bad=pending.clone();bad.mode=AttemptMode::RecordWitness;assert!(bad.encode().is_err());
        bad.record_hash=[1;32];assert_eq!(AttemptRecord::decode(&bad.encode().unwrap()).unwrap(),bad);
        let mut wrong_token=token.clone();wrong_token[32]^=1;
        assert!(AttemptRecord::initial_pending(&grant,transaction,b"request",&wrong_token).is_err());
        assert!(AttemptRecord::initial_pending(&grant.revoked().unwrap(),transaction,b"request",&token).is_err());
        // Neither codec-positive above is admitted transaction/source evidence.
    }

    #[test]
    fn r4_root_grant_canonical_vector_is_exact_and_revocation_is_monotone() {
        let record = GrantRecord::decode(FIXTURE).unwrap();
        assert_eq!(record.encode().unwrap(), FIXTURE);
        assert!(record.lifecycle == Lifecycle::Active);
        assert_eq!(record.generation, 0);
        let revoked = record.revoked().unwrap();
        assert!(revoked.lifecycle == Lifecycle::Revoked);
        assert_eq!(revoked.generation, 1);
        assert!(matches!(revoked.revoked(), Err(Failure::CustodyConflict)));
        let mut overflow = record;
        overflow.generation = u64::MAX;
        assert!(matches!(overflow.revoked(), Err(Failure::LimitExceeded)));
    }

    #[test]
    fn r4_root_grant_rejects_noncanonical_and_unbounded_inputs() {
        for length in [0, 1, 294] { assert!(GrantRecord::decode(&FIXTURE[..length]).is_err()); }
        let mut trailing = FIXTURE.to_vec(); trailing.push(0);
        assert!(GrantRecord::decode(&trailing).is_err());
        for offset in [0, MAGIC.len(), MAGIC.len() + 1, MAGIC.len() + 2, 28, 29, 30, 31, 32, 283] {
            let mut corrupt = FIXTURE.to_vec(); corrupt[offset] ^= 0xff;
            assert!(GrantRecord::decode(&corrupt).is_err(), "offset={offset}");
        }
        let mut zero_nonce = FIXTURE.to_vec(); zero_nonce[33..65].fill(0);
        assert!(GrantRecord::decode(&zero_nonce).is_err());
    }

    #[test]
    fn r4_root_grant_slot_and_fingerprint_match_reviewed_closed_layout() {
        let record=GrantRecord::decode(FIXTURE).unwrap();
        assert_eq!(hex::encode(root_fingerprint(&record)),"0448a3767f66fc2a995a2d7ead351b9c1cec59c85026af075bdaf119c6a9c756");
        let slot=encode_slot(&record.nonce).unwrap();
        assert_eq!(slot.len(),58);
        assert_eq!(decode_slot(&slot).unwrap(),record.nonce);
        for length in [0,1,57] { assert!(decode_slot(&slot[..length]).is_err()); }
        let mut trailing=slot.clone(); trailing.push(0); assert!(decode_slot(&trailing).is_err());
        for offset in [0,SLOT_MAGIC.len(),SLOT_MAGIC.len()+1] {
            let mut bad=slot.clone(); bad[offset]^=0xff; assert!(decode_slot(&bad).is_err());
        }
        let mut zero=slot; zero[SLOT_MAGIC.len()+2..].fill(0); assert!(decode_slot(&zero).is_err());
    }
}
