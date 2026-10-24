//! WO23 R1.5 common result/replay evidence owners.
//! Evaluator candidates are not a committed result or a completion proof.
mod typed_output;
pub(crate) use typed_output::StagedResultEmissionsV1;
mod emission_sink;
pub(crate) use emission_sink::ResultEmissionSinkV1;
pub(crate) use emission_sink::PreparedOutputMadiV1;
pub(crate) use emission_sink::ExecutionGenerationV1;
mod result_preimage;
pub(crate) use result_preimage::ResultPreimageV1;
mod seed;
pub(crate) use seed::{ReplaySeedV1,ReplaySeedCandidateV1};
mod receipt_json;
pub(crate) use receipt_json::actual_receipt_array;
pub(crate) use receipt_json::canonical_transport_json;
pub(crate) use receipt_json::decode_receipt_array;
mod logical_progress;
pub(crate) use logical_progress::{LogicalRecordingRunV1,LogicalProgressEvidenceV1};
mod general_progress;
pub(crate) use general_progress::GeneralProgressEvidenceV1;
mod logical_no_end;
pub(crate) use logical_no_end::LogicalNoEndEvidenceV1;
mod bundle;
pub(crate) mod finish_recovery;
#[cfg(windows)]
mod first_checkpoint;
#[cfg(windows)]
mod checkpoint_data;
#[cfg(windows)]
pub(crate) use checkpoint_data::{ReplayCheckpointPlanDataV1,ReplayPendingFailureV1};
#[cfg(windows)]
mod next_checkpoint;
#[cfg(windows)]
pub(crate) use next_checkpoint::PlannedNextReplayCheckpointV1;
#[cfg(windows)]
mod native_finish;
#[cfg(windows)]
pub(crate) use native_finish::{NativeFinishedRecordingReadyV1,NativeFinishCheckpointPlanV1,NativeFinishRecoveryIntentV1};
#[cfg(windows)]
mod native_no_end;
#[cfg(windows)]
pub(crate) use native_no_end::{NativeNoEndCheckpointPlanV1,NativeNoEndStoredV1};
#[cfg(windows)]
pub(crate) use first_checkpoint::PlannedFirstReplayCheckpointV1;
mod custody;
pub(crate) use custody::{ReplayCustodyPlainV1,ReplayCustodyCandidateV1};
mod control;
pub(crate) use control::ReplayLifecycleCandidateV1;
pub(crate) use control::control_row_canonical_bytes;
pub(crate) use bundle::ReplayBundleCandidateV1;
#[cfg(windows)]
pub(crate) use control::{PlannedReplayControlRowV1,PendingReplayLifecycleV1,ReadyReplayLifecycleV1};
#[cfg(test)]
mod readonly_tests;
