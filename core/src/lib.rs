pub mod alrim;
pub mod atomic_commit;
pub mod capability;
pub mod commit_canon;
pub mod engine;
pub mod external_boundary;
pub mod fixed64;
pub mod gogae3;
pub mod input;
pub mod nurigym;
pub mod platform;
pub mod realms;
pub mod resource;
pub mod sam;
pub mod seulgi;
pub mod signals;
pub mod units;
pub mod warp;

pub use alrim::{
    AlrimHandler, AlrimLogEntry, AlrimLogger, AlrimLoop, VecAlrimLogger, ALRIM_MAX_PASSES,
};
pub use atomic_commit::{
    analyze_patch_effects, commit_patch_candidates_atomically,
    commit_patch_candidates_atomically_with_capability_preflight, AtomicCapabilityPreflight,
    AtomicCommitError, AtomicCommitExecution, AtomicCommitInputKind, AtomicCommitInputRecord,
    AtomicCommitReceipt, AtomicCommitStatus, AuthorizedCapabilityDispatch, CandidateBatch,
    CandidateDecision, CandidateDisposition, CandidateIdentityScope, CandidateOrigin,
    LegacySinglePatchEnvelope, LogicalPatchKey, PatchCandidate, PatchCandidateStatus, PatchEffects,
    ResourceStorageKind,
};
pub use capability::{
    CapabilityAuthorization, CapabilityAuthorizationRequest, CapabilityBinding,
    CapabilityOperation, CapabilityRef, CapabilityScope, CapabilityValue,
    ProcessLocalCapabilityStore,
};
pub use commit_canon::{
    candidate_disposition_name, canonical_commit_receipt_hash, canonical_commit_receipt_text,
    canonical_execution_binding_hash, canonical_execution_binding_text,
    canonical_input_snapshot_hash, canonical_input_snapshot_text, canonical_patch_ops_hash,
    canonical_patch_ops_text, canonical_signal_list_hash, canonical_signal_list_text,
    canonical_signal_text, canonicalize_candidate, canonicalize_candidate_set, domain_hash_text,
    CanonicalCandidate, CanonicalCandidateSet, CanonicalCommitReceiptRecord,
    CanonicalExecutionBindingRecord, COMMIT_RECEIPT_DOMAIN, EMISSION_BUFFER_DOMAIN,
    EXECUTION_BINDING_DOMAIN, INPUT_SNAPSHOT_DOMAIN, PATCH_CANDIDATE_DOMAIN,
    PATCH_CANDIDATE_SET_DOMAIN, PATCH_OPS_DOMAIN,
};
pub use engine::EngineLoop;
pub use external_boundary::{
    replay_sealed_external_execution, CanonicalExternalInput, DependencyNodeKind,
    DeterministicSealedProvider, ExternalBoundaryFailure, ExternalBoundaryReceipt,
    ExternalBoundaryRequest, ExternalBoundaryResult, ExternalDependencyNode,
    SealedBoundaryProvider, SealedExecutionLedger,
};
pub use fixed64::Fixed64;
pub use input::{is_key_just_pressed, is_key_pressed, key_bit_from_name};
pub use nurigym::spec::{ActionSpec, ObservationSpec};
pub use platform::{
    Bogae, ComponentTag, DetSam, EntityId, Geoul, InMemoryGeoul, InputSnapshot, InputSource, Iyagi,
    Nuri, NuriWorld, Patch, PatchOp, ResourceMapEntry, ResourceValue, Sam, Seulgi, SeulgiContext,
    SeulgiIntent, SeulgiPacket, StateHash, TickFrame, KEY_A, KEY_D, KEY_S, KEY_W,
};
pub use realms::{mix64, MultiRealmManager, Realm, RealmStepInput, RealmStepOutput, ThreadMode};
pub use resource::{asset_handle_from_bundle_path, ResourceHandle};
pub use seulgi::latency::{LatencyEvent, LatencyMode, LatencyPolicy};
pub use seulgi::safety::{SafetyDecision, SafetyMode, SafetyRule};
pub use seulgi::{goal, intent};
pub use signals::{
    ArithmeticFaultKind, ExprTrace, FaultContext, Signal, SignalSink, SourceSpan, TickId,
    VecSignalSink,
};
pub use units::{
    base_unit_symbol_for_dim, canonical_unit_symbol, is_known_unit, resource_tag_with_unit,
    set_unit_registry_symbols, unit_spec_from_symbol, Unit, UnitDim, UnitError, UnitSpec,
    UnitValue,
};
pub use warp::{
    run_warp_bench, StepBatchSoA, WarpBackend, WarpBenchInput, WarpBenchOutput, WarpPolicy,
};

#[cfg(test)]
mod tests;
