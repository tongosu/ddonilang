//! Minimal product-core vertical slice for deterministic atomic patch batches.
//!
//! `EngineLoop` reaches this boundary through `LegacySinglePatchEnvelope`, while the
//! legacy `Nuri::apply_patch` API and the remaining tool/WASM/eco frontdoors are still
//! open migration edges. Capability authority also remains open.

use std::collections::{BTreeMap, BTreeSet};

use crate::capability::{
    CapabilityAuthorization, CapabilityAuthorizationRequest, CapabilityBinding,
    CapabilityOperation, CapabilityRef, ProcessLocalCapabilityStore,
};
use crate::commit_canon::{
    candidate_disposition_name, canonical_commit_receipt_hash, canonical_patch_ops_hash,
    canonical_signal_list_hash, canonicalize_candidate_set, compare_patch_candidates,
    normalize_nfc_text, CanonicalCommitReceiptRecord,
};
use crate::platform::{ComponentTag, EntityId, NuriWorld, Origin, Patch, PatchOp, StateHash};
use crate::signals::{ArithmeticFaultKind, DiagEvent, FaultContext, Signal, SignalSink, TickId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatchCandidateStatus {
    Ok,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchCandidate {
    pub candidate_id: String,
    pub madi: TickId,
    pub phase: u32,
    pub origin_kind: String,
    pub origin_id: String,
    pub task_group_id: String,
    pub task_id: String,
    pub local_candidate_seq: u64,
    pub status: PatchCandidateStatus,
    pub patch: Patch,
}

/// Typed interpretation of the canonical candidate origin envelope.
///
/// The strings remain part of the canonical bytes. This enum is always derived from
/// them, so it cannot become a second, hidden semantic input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateOrigin {
    Entity(EntityId),
    System(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AtomicCommitInputKind {
    LegacySinglePatchCompat,
    CandidateBatch,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CandidateIdentityScope {
    PatchContentAndMadiCompat,
    PatchContentMadiAndExecutionBinding,
    DeclaredCandidateEnvelope,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtomicCommitInputRecord {
    pub kind: AtomicCommitInputKind,
    pub identity_scope: CandidateIdentityScope,
    pub madi: TickId,
    pub candidate_ids: Vec<String>,
    pub execution_binding_digest: Option<String>,
}

/// Compatibility boundary for an existing producer that yields exactly one `Patch`.
///
/// Its candidate identity covers only the patch content, canonical origin, and madi.
/// It must not be presented as source/task admission identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacySinglePatchEnvelope {
    pub candidate: PatchCandidate,
    pub execution_binding_digest: Option<String>,
}

impl LegacySinglePatchEnvelope {
    pub fn new(madi: TickId, patch: Patch) -> Self {
        let ops_hash = canonical_patch_ops_hash(&patch);
        let mut candidate = PatchCandidate::new("", madi, patch);
        candidate.candidate_id = format!(
            "legacy-single-patch:{madi}:{}:{}:{ops_hash}",
            candidate.origin_kind, candidate.origin_id
        );
        candidate.task_group_id = "legacy-single-patch".to_string();
        candidate.task_id = "legacy-single-patch".to_string();
        Self {
            candidate,
            execution_binding_digest: None,
        }
    }

    pub fn new_with_execution_binding(
        madi: TickId,
        patch: Patch,
        execution_binding_digest: impl Into<String>,
    ) -> Self {
        let execution_binding_digest = normalize_nfc_text(&execution_binding_digest.into());
        let ops_hash = canonical_patch_ops_hash(&patch);
        let mut candidate = PatchCandidate::new("", madi, patch);
        candidate.candidate_id = format!(
            "legacy-single-patch-bound:{madi}:{}:{}:{ops_hash}:{execution_binding_digest}",
            candidate.origin_kind, candidate.origin_id
        );
        candidate.task_group_id = "legacy-single-patch-bound".to_string();
        candidate.task_id = "legacy-single-patch-bound".to_string();
        Self {
            candidate,
            execution_binding_digest: Some(execution_binding_digest),
        }
    }

    pub fn input_record(&self) -> AtomicCommitInputRecord {
        AtomicCommitInputRecord {
            kind: AtomicCommitInputKind::LegacySinglePatchCompat,
            identity_scope: if self.execution_binding_digest.is_some() {
                CandidateIdentityScope::PatchContentMadiAndExecutionBinding
            } else {
                CandidateIdentityScope::PatchContentAndMadiCompat
            },
            madi: self.candidate.madi,
            candidate_ids: vec![self.candidate.candidate_id.clone()],
            execution_binding_digest: self.execution_binding_digest.clone(),
        }
    }

    pub fn commit(&self, world: &mut NuriWorld, sink: &mut dyn SignalSink) -> AtomicCommitReceipt {
        commit_patch_candidates_atomically(
            world,
            std::slice::from_ref(&self.candidate),
            self.candidate.madi,
            sink,
        )
    }
}

/// A producer-declared candidate batch. Unlike the legacy envelope, this type never
/// synthesizes or rewrites candidate identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateBatch {
    pub madi: TickId,
    pub candidates: Vec<PatchCandidate>,
}

impl CandidateBatch {
    pub fn new(madi: TickId, candidates: Vec<PatchCandidate>) -> Self {
        Self { madi, candidates }
    }

    pub fn input_record(&self) -> AtomicCommitInputRecord {
        let mut ordered = self.candidates.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| compare_patch_candidates(left, right));
        AtomicCommitInputRecord {
            kind: AtomicCommitInputKind::CandidateBatch,
            identity_scope: CandidateIdentityScope::DeclaredCandidateEnvelope,
            madi: self.madi,
            candidate_ids: ordered
                .into_iter()
                .map(|candidate| candidate.candidate_id.clone())
                .collect(),
            execution_binding_digest: None,
        }
    }

    pub fn commit(&self, world: &mut NuriWorld, sink: &mut dyn SignalSink) -> AtomicCommitReceipt {
        commit_patch_candidates_atomically(world, &self.candidates, self.madi, sink)
    }
}

impl PatchCandidate {
    pub fn new(candidate_id: impl Into<String>, madi: TickId, patch: Patch) -> Self {
        let (origin_kind, origin_id) = match &patch.origin {
            Origin::Entity(entity) => ("iyagi".to_string(), entity.0.to_string()),
            Origin::System(name) => ("system".to_string(), format!("#system:{name}")),
        };
        Self {
            candidate_id: candidate_id.into(),
            madi,
            phase: 20,
            origin_kind,
            origin_id,
            task_group_id: "default".to_string(),
            task_id: "default".to_string(),
            local_candidate_seq: 0,
            status: PatchCandidateStatus::Ok,
            patch,
        }
    }

    pub fn canonical_origin(&self) -> Option<CandidateOrigin> {
        let kind = normalize_nfc_text(&self.origin_kind);
        let id = normalize_nfc_text(&self.origin_id);
        match kind.as_str() {
            "iyagi" if !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()) => id
                .parse::<u64>()
                .ok()
                .map(EntityId)
                .map(CandidateOrigin::Entity),
            "system" if id.starts_with("#system:") && id.len() > "#system:".len() => {
                Some(CandidateOrigin::System(id))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogicalPatchKey {
    Component { entity: EntityId, tag: String },
    Resource { tag: String },
}

impl LogicalPatchKey {
    pub fn canonical_name(&self) -> String {
        match self {
            Self::Component { entity, tag } => {
                format!("component/{}/{}", entity.0, normalize_nfc_text(tag))
            }
            Self::Resource { tag } => format!("resource/{}", normalize_nfc_text(tag)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceStorageKind {
    Json,
    Fixed64,
    Handle,
    Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchEffects {
    pub reads: Vec<LogicalPatchKey>,
    pub writes: Vec<LogicalPatchKey>,
    pub resource_writes: Vec<(String, ResourceStorageKind)>,
    pub domain_emission_count: usize,
    pub diagnostic_intent_count: usize,
    pub may_arithmetic_fail: bool,
    pub has_guard_violation: bool,
    pub guard_violation_entities: Vec<EntityId>,
    pub requires_capability_preflight: bool,
}

pub fn analyze_patch_effects(patch: &Patch) -> PatchEffects {
    let mut reads = BTreeSet::new();
    let mut writes = BTreeSet::new();
    let mut resource_writes = BTreeSet::new();
    let mut domain_emission_count = 0;
    let mut diagnostic_intent_count = 0;
    let mut may_arithmetic_fail = false;
    let mut has_guard_violation = false;
    let mut guard_violation_entities = BTreeSet::new();
    let mut requires_capability_preflight = false;

    for op in &patch.ops {
        match op {
            PatchOp::SetComponentJson { entity, tag, .. }
            | PatchOp::RemoveComponent { entity, tag } => {
                writes.insert(LogicalPatchKey::Component {
                    entity: *entity,
                    tag: normalize_nfc_text(&tag.0),
                });
            }
            PatchOp::SetResourceJson { tag, .. } => {
                let tag = normalize_nfc_text(tag);
                writes.insert(LogicalPatchKey::Resource { tag: tag.clone() });
                resource_writes.insert((tag, ResourceStorageKind::Json));
            }
            PatchOp::SetResourceFixed64 { tag, .. } => {
                let tag = normalize_nfc_text(tag);
                writes.insert(LogicalPatchKey::Resource { tag: tag.clone() });
                resource_writes.insert((tag, ResourceStorageKind::Fixed64));
            }
            PatchOp::SetResourceHandle { tag, .. } => {
                let tag = normalize_nfc_text(tag);
                writes.insert(LogicalPatchKey::Resource { tag: tag.clone() });
                resource_writes.insert((tag, ResourceStorageKind::Handle));
                requires_capability_preflight = true;
            }
            PatchOp::SetResourceValue { tag, .. } => {
                let tag = normalize_nfc_text(tag);
                writes.insert(LogicalPatchKey::Resource { tag: tag.clone() });
                resource_writes.insert((tag, ResourceStorageKind::Value));
            }
            PatchOp::DivAssignResourceFixed64 { tag, .. } => {
                let tag = normalize_nfc_text(tag);
                let key = LogicalPatchKey::Resource { tag: tag.clone() };
                reads.insert(key.clone());
                writes.insert(key);
                resource_writes.insert((tag, ResourceStorageKind::Fixed64));
                may_arithmetic_fail = true;
            }
            PatchOp::EmitSignal { signal, .. } => match signal {
                Signal::Alrim { .. } => domain_emission_count += 1,
                Signal::ArithmeticFault { .. } | Signal::Diag { .. } => {
                    diagnostic_intent_count += 1;
                }
            },
            PatchOp::GuardViolation { entity, .. } => {
                diagnostic_intent_count += 1;
                has_guard_violation = true;
                guard_violation_entities.insert(*entity);
            }
        }
    }

    PatchEffects {
        reads: reads.into_iter().collect(),
        writes: writes.into_iter().collect(),
        resource_writes: resource_writes.into_iter().collect(),
        domain_emission_count,
        diagnostic_intent_count,
        may_arithmetic_fail,
        has_guard_violation,
        guard_violation_entities: guard_violation_entities.into_iter().collect(),
        requires_capability_preflight,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AtomicCommitStatus {
    Committed,
    Rejected,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CandidateDisposition {
    Committed,
    Rejected,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateDecision {
    pub candidate_id: String,
    pub disposition: CandidateDisposition,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AtomicCommitError {
    PatchSchema,
    DuplicateCandidate,
    TaskGroupFailed,
    ResourceNameTypeConflict,
    CapabilityPreflightUnavailable,
    CapabilityPreflightDenied,
    ArithmeticFault,
    GuardViolation,
    WriteConflict,
}

impl AtomicCommitError {
    pub fn code(self) -> &'static str {
        match self {
            Self::PatchSchema => "E_PATCH_SCHEMA",
            Self::DuplicateCandidate => "E_PATCH_DUPLICATE",
            Self::TaskGroupFailed => "E_TASK_GROUP_FAILED",
            Self::ResourceNameTypeConflict => "E_RESOURCE_NAME_TYPE_CONFLICT",
            Self::CapabilityPreflightUnavailable => "E_CAPABILITY_PREFLIGHT_UNAVAILABLE",
            Self::CapabilityPreflightDenied => "E_CAPABILITY_PREFLIGHT_DENIED",
            Self::ArithmeticFault => "E_ARITH_FAULT",
            Self::GuardViolation => "E_GUARD_VIOLATION",
            Self::WriteConflict => "E_MADI_WRITE_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtomicCommitReceipt {
    pub madi: TickId,
    pub commit_id: String,
    pub initial_state_hash: StateHash,
    pub candidate_set_hash: String,
    pub ordered_candidate_ids: Vec<String>,
    pub candidate_decisions: Vec<CandidateDecision>,
    pub status: AtomicCommitStatus,
    pub error: Option<AtomicCommitError>,
    pub conflict_keys: Vec<String>,
    pub committed_state_hash: StateHash,
    /// Half-open range `[start, end)` allocated within this batch.
    pub diagnostic_sequence_range: Option<(u64, u64)>,
    pub released_domain_emissions: usize,
    pub emission_digest: String,
    pub failure_consequence_ids: Vec<String>,
    pub receipt_hash: String,
    /// True only when the candidate set, released emissions, and receipt were canonically hashed.
    pub canonical_hashes_verified: bool,
}

impl AtomicCommitReceipt {
    pub fn canonical_record(&self) -> CanonicalCommitReceiptRecord {
        CanonicalCommitReceiptRecord {
            candidate_dispositions: self
                .candidate_decisions
                .iter()
                .map(|decision| {
                    (
                        decision.candidate_id.clone(),
                        candidate_disposition_name(decision.disposition).to_string(),
                    )
                })
                .collect(),
            candidate_set_hash: self.candidate_set_hash.clone(),
            commit_id: self.commit_id.clone(),
            committed_state_hash: state_hash_label(self.committed_state_hash),
            conflict_keys: self.conflict_keys.clone(),
            diagnostic_sequence_range: self.diagnostic_sequence_range,
            emission_digest: self.emission_digest.clone(),
            error_code: self.error.map(|error| error.code().to_string()),
            failure_consequence_ids: self.failure_consequence_ids.clone(),
            initial_state_hash: state_hash_label(self.initial_state_hash),
            madi: self.madi,
            ordered_candidate_ids: self.ordered_candidate_ids.clone(),
            status: match self.status {
                AtomicCommitStatus::Committed => "committed",
                AtomicCommitStatus::Rejected => "rejected",
            }
            .to_string(),
        }
    }
}

enum BufferedEmission {
    Domain(Signal),
    Fault(Signal),
    Diagnostic(DiagEvent),
}

struct StagedBatch {
    world: NuriWorld,
    emissions: Vec<BufferedEmission>,
    surviving_writes: Vec<(usize, LogicalPatchKey)>,
    surviving_resource_writes: Vec<(String, ResourceStorageKind)>,
    arithmetic_consequence_ids: Vec<String>,
}

#[derive(Copy, Clone)]
struct CommitAttempt {
    madi: TickId,
    initial_state_hash: StateHash,
}

fn state_hash_label(hash: StateHash) -> String {
    format!("blake3:{}", hash.to_hex())
}

fn candidate_entity_origin(candidate: &PatchCandidate) -> Option<EntityId> {
    match candidate.canonical_origin() {
        Some(CandidateOrigin::Entity(entity)) => Some(entity),
        Some(CandidateOrigin::System(_)) | None => None,
    }
}

fn candidate_is_guard_violator(
    candidate: &PatchCandidate,
    guard_violators: &BTreeSet<EntityId>,
) -> bool {
    candidate_entity_origin(candidate).is_some_and(|entity| guard_violators.contains(&entity))
}

fn existing_resource_kinds(world: &NuriWorld) -> BTreeMap<String, BTreeSet<ResourceStorageKind>> {
    let mut kinds: BTreeMap<String, BTreeSet<ResourceStorageKind>> = BTreeMap::new();
    for (tag, _) in world.resource_json_entries() {
        kinds
            .entry(tag)
            .or_default()
            .insert(ResourceStorageKind::Json);
    }
    for (tag, _) in world.resource_fixed64_entries() {
        kinds
            .entry(tag)
            .or_default()
            .insert(ResourceStorageKind::Fixed64);
    }
    for (tag, _) in world.resource_handle_entries() {
        kinds
            .entry(tag)
            .or_default()
            .insert(ResourceStorageKind::Handle);
    }
    for (tag, _) in world.resource_value_entries() {
        kinds
            .entry(tag)
            .or_default()
            .insert(ResourceStorageKind::Value);
    }
    kinds
}

fn resource_type_conflicts(
    world: &NuriWorld,
    resource_writes: &[(String, ResourceStorageKind)],
) -> Vec<String> {
    let mut kinds = existing_resource_kinds(world);
    let mut touched = BTreeSet::new();
    for (tag, kind) in resource_writes {
        touched.insert(tag.clone());
        kinds.entry(tag.clone()).or_default().insert(*kind);
    }
    touched
        .into_iter()
        .filter(|tag| kinds.get(tag).is_some_and(|items| items.len() > 1))
        .map(|tag| format!("resource/{tag}"))
        .collect()
}

fn write_conflicts(writes: &[(usize, LogicalPatchKey)]) -> Vec<String> {
    let mut owners: BTreeMap<LogicalPatchKey, usize> = BTreeMap::new();
    let mut conflicts = BTreeSet::new();
    for (candidate_index, key) in writes {
        if owners
            .insert(key.clone(), *candidate_index)
            .is_some_and(|previous| previous != *candidate_index)
        {
            conflicts.insert(key.canonical_name());
        }
    }
    conflicts.into_iter().collect()
}

fn fault_diag(
    madi: TickId,
    origin: String,
    tag: &str,
    kind: &ArithmeticFaultKind,
    ctx: &FaultContext,
) -> DiagEvent {
    let (reason, sub_reason) = match kind {
        ArithmeticFaultKind::DimensionMismatch { .. } => {
            ("UNIT_MISMATCH", Some("DIM_MISMATCH".to_string()))
        }
        ArithmeticFaultKind::DivByZero => ("ARITH_FAULT", Some("DIV0".to_string())),
    };
    DiagEvent {
        madi,
        seq: 0,
        fault_id: reason.to_string(),
        rule_id: String::new(),
        reason: reason.to_string(),
        sub_reason,
        mode: None,
        contract_kind: None,
        origin,
        targets: vec![format!("resource:{tag}")],
        sam_hash: None,
        source_span: ctx.source_span.clone(),
        expr: ctx.expr.clone(),
        message: None,
    }
}

fn decision_diag(madi: TickId, error: AtomicCommitError, targets: Vec<String>) -> DiagEvent {
    DiagEvent {
        madi,
        seq: 0,
        fault_id: error.code().to_string(),
        rule_id: String::new(),
        reason: error.code().to_string(),
        sub_reason: None,
        mode: None,
        contract_kind: Some("atomic_patch_batch_v0".to_string()),
        origin: "#system:atomic-commit-v0".to_string(),
        targets,
        sam_hash: None,
        source_span: None,
        expr: None,
        message: None,
    }
}

fn arithmetic_consequence_id(
    candidate: &PatchCandidate,
    op_index: usize,
    kind: &ArithmeticFaultKind,
) -> String {
    let kind = match kind {
        ArithmeticFaultKind::DivByZero => "div0",
        ArithmeticFaultKind::DimensionMismatch { .. } => "dim_mismatch",
    };
    format!(
        "arith:{}:{op_index}:{kind}",
        normalize_nfc_text(&candidate.candidate_id)
    )
}

fn stage_batch(
    initial: &NuriWorld,
    ordered: &[&PatchCandidate],
    madi: TickId,
    guard_violators: &BTreeSet<EntityId>,
) -> StagedBatch {
    let mut evaluation_world = initial.clone();
    let mut world = initial.clone();
    let mut emissions = Vec::new();
    let mut surviving_writes = Vec::new();
    let mut surviving_resource_writes = Vec::new();
    let mut arithmetic_consequence_ids = Vec::new();

    for (candidate_index, candidate) in ordered.iter().enumerate() {
        let discard_candidate_effects = candidate_is_guard_violator(candidate, guard_violators);
        for (op_index, op) in candidate.patch.ops.iter().enumerate() {
            match op {
                PatchOp::SetComponentJson { entity, tag, json } => {
                    evaluation_world.set_component_json(*entity, tag.clone(), json.clone());
                    if !discard_candidate_effects {
                        world.set_component_json(*entity, tag.clone(), json.clone());
                        surviving_writes.push((
                            candidate_index,
                            LogicalPatchKey::Component {
                                entity: *entity,
                                tag: normalize_nfc_text(&tag.0),
                            },
                        ));
                    }
                }
                PatchOp::RemoveComponent { entity, tag } => {
                    evaluation_world.remove_component(*entity, tag);
                    if !discard_candidate_effects {
                        world.remove_component(*entity, tag);
                        surviving_writes.push((
                            candidate_index,
                            LogicalPatchKey::Component {
                                entity: *entity,
                                tag: normalize_nfc_text(&tag.0),
                            },
                        ));
                    }
                }
                PatchOp::SetResourceJson { tag, json } => {
                    evaluation_world.set_resource_json(tag.clone(), json.clone());
                    if !discard_candidate_effects {
                        world.set_resource_json(tag.clone(), json.clone());
                        let tag = normalize_nfc_text(tag);
                        surviving_writes.push((
                            candidate_index,
                            LogicalPatchKey::Resource { tag: tag.clone() },
                        ));
                        surviving_resource_writes.push((tag, ResourceStorageKind::Json));
                    }
                }
                PatchOp::SetResourceFixed64 { tag, value } => {
                    evaluation_world.set_resource_fixed64(tag.clone(), *value);
                    if !discard_candidate_effects {
                        world.set_resource_fixed64(tag.clone(), *value);
                        let tag = normalize_nfc_text(tag);
                        surviving_writes.push((
                            candidate_index,
                            LogicalPatchKey::Resource { tag: tag.clone() },
                        ));
                        surviving_resource_writes.push((tag, ResourceStorageKind::Fixed64));
                    }
                }
                PatchOp::SetResourceHandle { tag, handle } => {
                    evaluation_world.set_resource_handle(tag.clone(), *handle);
                    if !discard_candidate_effects {
                        world.set_resource_handle(tag.clone(), *handle);
                        let tag = normalize_nfc_text(tag);
                        surviving_writes.push((
                            candidate_index,
                            LogicalPatchKey::Resource { tag: tag.clone() },
                        ));
                        surviving_resource_writes.push((tag, ResourceStorageKind::Handle));
                    }
                }
                PatchOp::SetResourceValue { tag, value } => {
                    evaluation_world.set_resource_value(tag.clone(), value.clone());
                    if !discard_candidate_effects {
                        world.set_resource_value(tag.clone(), value.clone());
                        let tag = normalize_nfc_text(tag);
                        surviving_writes.push((
                            candidate_index,
                            LogicalPatchKey::Resource { tag: tag.clone() },
                        ));
                        surviving_resource_writes.push((tag, ResourceStorageKind::Value));
                    }
                }
                PatchOp::DivAssignResourceFixed64 {
                    tag,
                    rhs,
                    tick_id,
                    location,
                    source_span,
                    expr,
                } => {
                    let current = evaluation_world
                        .get_resource_fixed64(tag)
                        .unwrap_or(crate::Fixed64::ZERO);
                    let ctx = FaultContext {
                        tick_id: *tick_id,
                        location,
                        source_span: source_span.clone(),
                        expr: expr.clone(),
                    };
                    match current.try_div(*rhs) {
                        Ok(next) => {
                            evaluation_world.set_resource_fixed64(tag.clone(), next);
                            if !discard_candidate_effects {
                                world.set_resource_fixed64(tag.clone(), next);
                                let tag = normalize_nfc_text(tag);
                                surviving_writes.push((
                                    candidate_index,
                                    LogicalPatchKey::Resource { tag: tag.clone() },
                                ));
                                surviving_resource_writes.push((tag, ResourceStorageKind::Fixed64));
                            }
                        }
                        Err(kind) => {
                            emissions.push(BufferedEmission::Fault(Signal::ArithmeticFault {
                                ctx: ctx.clone(),
                                kind: kind.clone(),
                            }));
                            emissions.push(BufferedEmission::Diagnostic(fault_diag(
                                madi,
                                candidate.origin_id.clone(),
                                tag,
                                &kind,
                                &ctx,
                            )));
                            arithmetic_consequence_ids
                                .push(arithmetic_consequence_id(candidate, op_index, &kind));
                        }
                    }
                }
                PatchOp::EmitSignal { signal, targets } => match signal {
                    Signal::Alrim { .. } => {
                        if !discard_candidate_effects {
                            emissions.push(BufferedEmission::Domain(signal.clone()));
                        }
                    }
                    Signal::ArithmeticFault { ctx, kind } => {
                        emissions.push(BufferedEmission::Fault(signal.clone()));
                        let tag = targets.first().map(String::as_str).unwrap_or("unknown");
                        emissions.push(BufferedEmission::Diagnostic(fault_diag(
                            madi,
                            candidate.origin_id.clone(),
                            tag,
                            kind,
                            ctx,
                        )));
                        arithmetic_consequence_ids
                            .push(arithmetic_consequence_id(candidate, op_index, kind));
                    }
                    Signal::Diag { event } => {
                        let mut event = event.clone();
                        event.madi = madi;
                        if event.targets.is_empty() {
                            event.targets = if targets.is_empty() {
                                vec!["unknown".to_string()]
                            } else {
                                targets.clone()
                            };
                        }
                        emissions.push(BufferedEmission::Diagnostic(event));
                    }
                },
                PatchOp::GuardViolation { entity, rule_id } => {
                    emissions.push(BufferedEmission::Diagnostic(DiagEvent {
                        madi,
                        seq: 0,
                        fault_id: "GUARD_VIOLATION".to_string(),
                        rule_id: rule_id.clone(),
                        reason: "GUARD_VIOLATION".to_string(),
                        sub_reason: None,
                        mode: None,
                        contract_kind: Some("atomic_patch_batch_v0".to_string()),
                        origin: format!("entity:{}", entity.0),
                        targets: vec![format!("entity:{}", entity.0)],
                        sam_hash: None,
                        source_span: None,
                        expr: None,
                        message: None,
                    }));
                }
            }
        }
    }

    for entity in guard_violators {
        world.set_component_json(
            *entity,
            ComponentTag("#규칙위반".to_string()),
            "참".to_string(),
        );
        world.set_component_json(*entity, ComponentTag("#휴면".to_string()), "참".to_string());
    }

    StagedBatch {
        world,
        emissions,
        surviving_writes,
        surviving_resource_writes,
        arithmetic_consequence_ids,
    }
}

fn release_emissions(
    emissions: Vec<BufferedEmission>,
    commit: bool,
    madi: TickId,
    sink: &mut dyn SignalSink,
) -> (u64, usize, String) {
    let mut diag_seq = 0u64;
    let mut domain_count = 0usize;
    let mut released = Vec::new();
    for emission in emissions {
        match emission {
            BufferedEmission::Domain(signal) if commit => {
                released.push(signal.clone());
                sink.emit(signal);
                domain_count += 1;
            }
            BufferedEmission::Fault(signal) => {
                released.push(signal.clone());
                sink.emit(signal);
            }
            BufferedEmission::Diagnostic(mut event) => {
                event.madi = madi;
                event.seq = diag_seq;
                diag_seq += 1;
                let signal = Signal::Diag { event };
                released.push(signal.clone());
                sink.emit(signal);
            }
            BufferedEmission::Domain(_) => {}
        }
    }
    let emission_digest = canonical_signal_list_hash(&released);
    (diag_seq, domain_count, emission_digest)
}

fn decisions(
    ordered: &[&PatchCandidate],
    committed: bool,
    guard_violators: &BTreeSet<EntityId>,
) -> Vec<CandidateDecision> {
    ordered
        .iter()
        .map(|candidate| CandidateDecision {
            candidate_id: candidate.candidate_id.clone(),
            disposition: match candidate.status {
                PatchCandidateStatus::Failed => CandidateDisposition::Failed,
                PatchCandidateStatus::Cancelled => CandidateDisposition::Cancelled,
                PatchCandidateStatus::Ok
                    if committed && candidate_is_guard_violator(candidate, guard_violators) =>
                {
                    CandidateDisposition::Rejected
                }
                PatchCandidateStatus::Ok if committed => CandidateDisposition::Committed,
                PatchCandidateStatus::Ok => CandidateDisposition::Rejected,
            },
        })
        .collect()
}

fn rejected_receipt(
    attempt: CommitAttempt,
    candidate_set_hash: &str,
    ordered: &[&PatchCandidate],
    error: AtomicCommitError,
    conflict_keys: Vec<String>,
    mut emissions: Vec<BufferedEmission>,
    failure_consequence_ids: Vec<String>,
    sink: &mut dyn SignalSink,
) -> AtomicCommitReceipt {
    let CommitAttempt {
        madi,
        initial_state_hash,
    } = attempt;
    if emissions.is_empty() {
        let targets = if conflict_keys.is_empty() {
            ordered
                .iter()
                .map(|candidate| candidate.candidate_id.clone())
                .collect()
        } else {
            conflict_keys.clone()
        };
        emissions.push(BufferedEmission::Diagnostic(decision_diag(
            madi, error, targets,
        )));
    }
    let (diag_count, _, emission_digest) = release_emissions(emissions, false, madi, sink);
    finalize_receipt(AtomicCommitReceipt {
        madi,
        commit_id: format!("commit-{madi}"),
        initial_state_hash,
        candidate_set_hash: candidate_set_hash.to_string(),
        ordered_candidate_ids: ordered
            .iter()
            .map(|candidate| candidate.candidate_id.clone())
            .collect(),
        candidate_decisions: decisions(ordered, false, &BTreeSet::new()),
        status: AtomicCommitStatus::Rejected,
        error: Some(error),
        conflict_keys,
        committed_state_hash: initial_state_hash,
        diagnostic_sequence_range: (diag_count > 0).then_some((0, diag_count)),
        released_domain_emissions: 0,
        emission_digest,
        failure_consequence_ids,
        receipt_hash: String::new(),
        canonical_hashes_verified: true,
    })
}

fn finalize_receipt(mut receipt: AtomicCommitReceipt) -> AtomicCommitReceipt {
    receipt.receipt_hash = canonical_commit_receipt_hash(&receipt.canonical_record());
    receipt
}

/// Bounded internal seam proving that authorization precedes external-like dispatch.
/// It is not an external-provider API or a stable boundary identity.
pub trait AuthorizedCapabilityDispatch {
    fn dispatch_authorized(&mut self, request: &CapabilityAuthorizationRequest);
}

pub struct AtomicCapabilityPreflight<'a> {
    pub store: &'a ProcessLocalCapabilityStore,
    pub binding: CapabilityBinding,
    pub presented: Option<CapabilityRef>,
    pub dispatch: &'a mut dyn AuthorizedCapabilityDispatch,
}

impl<'a> AtomicCapabilityPreflight<'a> {
    pub fn new(
        store: &'a ProcessLocalCapabilityStore,
        binding: CapabilityBinding,
        presented: Option<CapabilityRef>,
        dispatch: &'a mut dyn AuthorizedCapabilityDispatch,
    ) -> Self {
        Self {
            store,
            binding,
            presented,
            dispatch,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtomicCommitExecution {
    pub receipt: AtomicCommitReceipt,
    pub capability_authorization: Option<CapabilityAuthorization>,
}

fn commit_execution(
    receipt: AtomicCommitReceipt,
    capability_authorization: Option<CapabilityAuthorization>,
) -> AtomicCommitExecution {
    AtomicCommitExecution {
        receipt,
        capability_authorization,
    }
}

pub fn commit_patch_candidates_atomically(
    world: &mut NuriWorld,
    candidates: &[PatchCandidate],
    madi: TickId,
    sink: &mut dyn SignalSink,
) -> AtomicCommitReceipt {
    commit_patch_candidates_atomically_inner(world, candidates, madi, sink, None).receipt
}

pub fn commit_patch_candidates_atomically_with_capability_preflight(
    world: &mut NuriWorld,
    candidates: &[PatchCandidate],
    madi: TickId,
    sink: &mut dyn SignalSink,
    preflight: &mut AtomicCapabilityPreflight<'_>,
) -> AtomicCommitExecution {
    commit_patch_candidates_atomically_inner(world, candidates, madi, sink, Some(preflight))
}

fn commit_patch_candidates_atomically_inner(
    world: &mut NuriWorld,
    candidates: &[PatchCandidate],
    madi: TickId,
    sink: &mut dyn SignalSink,
    mut preflight: Option<&mut AtomicCapabilityPreflight<'_>>,
) -> AtomicCommitExecution {
    let initial_state_hash = world.state_hash();
    let attempt = CommitAttempt {
        madi,
        initial_state_hash,
    };
    let canonical_candidate_set = canonicalize_candidate_set(candidates, madi);
    let mut ordered = candidates.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| compare_patch_candidates(left, right));

    if ordered
        .iter()
        .any(|candidate| candidate.madi != madi || candidate.canonical_origin().is_none())
    {
        return commit_execution(rejected_receipt(
            attempt,
            &canonical_candidate_set.hash,
            &ordered,
            AtomicCommitError::PatchSchema,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            sink,
        ), None);
    }

    let mut ids = BTreeSet::new();
    if ordered
        .iter()
        .any(|candidate| !ids.insert(normalize_nfc_text(&candidate.candidate_id)))
    {
        return commit_execution(rejected_receipt(
            attempt,
            &canonical_candidate_set.hash,
            &ordered,
            AtomicCommitError::DuplicateCandidate,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            sink,
        ), None);
    }

    if ordered
        .iter()
        .any(|candidate| candidate.status != PatchCandidateStatus::Ok)
    {
        return commit_execution(rejected_receipt(
            attempt,
            &canonical_candidate_set.hash,
            &ordered,
            AtomicCommitError::TaskGroupFailed,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            sink,
        ), None);
    }

    let effects = ordered
        .iter()
        .map(|candidate| analyze_patch_effects(&candidate.patch))
        .collect::<Vec<_>>();
    let guard_violators = effects
        .iter()
        .flat_map(|effect| effect.guard_violation_entities.iter().copied())
        .collect::<BTreeSet<_>>();
    let committable_effects = ordered
        .iter()
        .zip(&effects)
        .filter(|(candidate, _)| !candidate_is_guard_violator(candidate, &guard_violators))
        .map(|(_, effect)| effect.clone())
        .collect::<Vec<_>>();

    let required_handle_tags = committable_effects
        .iter()
        .flat_map(|effect| effect.resource_writes.iter())
        .filter_map(|(tag, kind)| {
            (*kind == ResourceStorageKind::Handle).then_some(tag.clone())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut capability_authorization = None;
    let mut authorized_request = None;
    if !required_handle_tags.is_empty() {
        let Some(capability_preflight) = preflight.as_deref_mut() else {
            return commit_execution(
                rejected_receipt(
                    attempt,
                    &canonical_candidate_set.hash,
                    &ordered,
                    AtomicCommitError::CapabilityPreflightUnavailable,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    sink,
                ),
                Some(CapabilityAuthorization::MissingCapability),
            );
        };
        let request = CapabilityAuthorizationRequest {
            presented: capability_preflight.presented,
            binding: capability_preflight.binding.clone(),
            operation: CapabilityOperation::ResourceHandleWrite {
                tags: required_handle_tags,
            },
        };
        let authorization = capability_preflight.store.authorize(&request);
        if !authorization.is_authorized() {
            let error = if authorization == CapabilityAuthorization::MissingCapability {
                AtomicCommitError::CapabilityPreflightUnavailable
            } else {
                AtomicCommitError::CapabilityPreflightDenied
            };
            return commit_execution(
                rejected_receipt(
                    attempt,
                    &canonical_candidate_set.hash,
                    &ordered,
                    error,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    sink,
                ),
                Some(authorization),
            );
        }
        capability_authorization = Some(authorization);
        authorized_request = Some(request);
    }

    let mut staged = stage_batch(world, &ordered, madi, &guard_violators);
    let type_conflicts = resource_type_conflicts(world, &staged.surviving_resource_writes);
    if !type_conflicts.is_empty() {
        staged
            .emissions
            .push(BufferedEmission::Diagnostic(decision_diag(
                madi,
                AtomicCommitError::ResourceNameTypeConflict,
                type_conflicts.clone(),
            )));
        return commit_execution(rejected_receipt(
            attempt,
            &canonical_candidate_set.hash,
            &ordered,
            AtomicCommitError::ResourceNameTypeConflict,
            type_conflicts,
            staged.emissions,
            staged.arithmetic_consequence_ids,
            sink,
        ), capability_authorization);
    }

    let conflicts = write_conflicts(&staged.surviving_writes);
    if !conflicts.is_empty() {
        staged
            .emissions
            .push(BufferedEmission::Diagnostic(decision_diag(
                madi,
                AtomicCommitError::WriteConflict,
                conflicts.clone(),
            )));
        return commit_execution(rejected_receipt(
            attempt,
            &canonical_candidate_set.hash,
            &ordered,
            AtomicCommitError::WriteConflict,
            conflicts,
            staged.emissions,
            staged.arithmetic_consequence_ids,
            sink,
        ), capability_authorization);
    }

    let mut failure_consequence_ids = staged.arithmetic_consequence_ids.clone();
    failure_consequence_ids.sort();
    failure_consequence_ids.dedup();
    failure_consequence_ids.extend(
        guard_violators
            .iter()
            .map(|entity| format!("guard:{}:isolation", entity.0)),
    );
    if let Some(request) = authorized_request.as_ref() {
        preflight
            .as_deref_mut()
            .expect("authorized request requires a preflight context")
            .dispatch
            .dispatch_authorized(request);
    }
    *world = staged.world;
    let committed_state_hash = world.state_hash();
    let (diag_count, domain_count, emission_digest) =
        release_emissions(staged.emissions, true, madi, sink);
    commit_execution(finalize_receipt(AtomicCommitReceipt {
        madi,
        commit_id: format!("commit-{madi}"),
        initial_state_hash,
        candidate_set_hash: canonical_candidate_set.hash,
        ordered_candidate_ids: ordered
            .iter()
            .map(|candidate| candidate.candidate_id.clone())
            .collect(),
        candidate_decisions: decisions(&ordered, true, &guard_violators),
        status: AtomicCommitStatus::Committed,
        error: None,
        conflict_keys: Vec::new(),
        committed_state_hash,
        diagnostic_sequence_range: (diag_count > 0).then_some((0, diag_count)),
        released_domain_emissions: domain_count,
        emission_digest,
        failure_consequence_ids,
        receipt_hash: String::new(),
        canonical_hashes_verified: true,
    }), capability_authorization)
}
