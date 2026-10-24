//! Internal typed reaction transaction lifecycle; unchanged implementations.
use super::*;

/// Product-internal typed body for W1-C2 I1. This is neither public DDN
/// syntax nor a serialized or stable-wire representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalReactionProgram {
    pub operations: Vec<InternalReactionOperation>,
}

impl InternalReactionProgram {
    pub fn new(operations: Vec<InternalReactionOperation>) -> Self {
        Self { operations }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalReactionOperation {
    Set {
        key: String,
        value: String,
    },
    RequireSettledValue {
        key: String,
        expected: Option<String>,
    },
    StageSend(InternalDeliveryEnvelopeSemantic),
    StageEffect(InternalEffectIntent),
    EmitResult(String),
    EmitTrace(String),
    EmitReceipt(String),
    Fail {
        code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternalEffectIntent {
    pub identity: String,
}

impl InternalEffectIntent {
    pub fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionProviderPatchCandidate {
    pub participant_identity: String,
    pub writes: BTreeMap<String, String>,
    pub staged_outbox: Vec<InternalDeliveryEnvelopeSemantic>,
    pub result_candidates: Vec<String>,
    pub trace_candidates: Vec<String>,
    pub receipt_candidates: Vec<String>,
    pub effect_intents: Vec<InternalEffectIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionGroupCommitCandidate {
    pub writes: BTreeMap<String, String>,
    pub staged_outbox: Vec<InternalDeliveryEnvelopeSemantic>,
    pub result_candidates: Vec<String>,
    pub trace_candidates: Vec<String>,
    pub receipt_candidates: Vec<String>,
    pub effect_intents: Vec<InternalEffectIntent>,
    pub canonical_participants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalReactionFailure {
    RouteUnhandled,
    RouteAmbiguous {
        count: usize,
    },
    ProviderIdentity {
        detail: String,
    },
    BodyFailure {
        participant_identity: String,
        code: String,
    },
    PatchConflict {
        key: String,
        first_participant: String,
        second_participant: String,
    },
}

impl InternalReactionFailure {
    pub fn phase(&self) -> &'static str {
        match self {
            Self::RouteUnhandled | Self::RouteAmbiguous { .. } => "route",
            Self::ProviderIdentity { .. } => "provider_identity",
            Self::BodyFailure { .. } => "provider_body",
            Self::PatchConflict { .. } => "candidate_merge",
        }
    }

    pub fn product_local_candidate_code(&self) -> &'static str {
        match self {
            Self::RouteUnhandled => "E_ASYNC_ENDPOINT_UNHANDLED_CANDIDATE",
            Self::RouteAmbiguous { .. } => "E_ASYNC_ENDPOINT_AMBIGUOUS_CANDIDATE",
            Self::ProviderIdentity { .. } => "E_ASYNC_PROVIDER_IDENTITY_CANDIDATE",
            Self::BodyFailure { .. } => "E_ASYNC_PROVIDER_BODY_CANDIDATE",
            Self::PatchConflict { .. } => "E_ASYNC_PROVIDER_PATCH_CONFLICT_CANDIDATE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalReactionSuccess {
    pub route_identity: ddonirang_lang::RouteIdentity,
    pub execution_identity: ExecutionInstanceIdentity,
    pub occurrence_identity: DeliveryOccurrenceIdentity,
    pub provider_registration_identities: Vec<ProviderRegistrationIdentity>,
    pub settled_snapshot_digest: String,
    pub reaction_digest: String,
    pub committed_revision: u64,
    pub wait_receipts: Vec<LogicalWaitReceipt>,
    pub released_effect_identities: Vec<String>,
    pub external_release_failures: Vec<String>,
}

/// Prepared I1 reaction group. Preparing evaluates the consuming route and
/// every reached provider against one settled snapshot, but publishes no
/// state, outbox, result, trace, receipt, revision, or EffectIntent. The
/// actual DdnRunner uses this split so its outer delivery frame and the shared
/// typed group can commit as one product transaction.
#[derive(Debug, Clone)]
pub struct InternalReactionPrepared {
    pub(super) route_identity: ddonirang_lang::RouteIdentity,
    pub(super) execution_identity: ExecutionInstanceIdentity,
    pub(super) occurrence_identity: DeliveryOccurrenceIdentity,
    pub(super) provider_registration_identities: Vec<ProviderRegistrationIdentity>,
    pub(super) settled_snapshot_digest: String,
    pub(super) reaction_digest: String,
    pub(super) group: ReactionGroupCommitCandidate,
}

#[derive(Debug, Clone)]
pub struct InternalTypedReactionRuntime {
    pub(super) world: BTreeMap<String, String>,
    pub(super) revision: u64,
    pub(super) queue: LogicalAsyncQueue<InternalDeliveryEnvelopeSemantic>,
    pub(super) published_results: Vec<String>,
    pub(super) published_traces: Vec<String>,
    pub(super) published_receipts: Vec<String>,
    pub(super) released_effects: Vec<String>,
}

impl Default for InternalTypedReactionRuntime {
    fn default() -> Self {
        Self::with_world_and_queue_capacity(
            BTreeMap::new(),
            ddonirang_lang::DEFAULT_ASYNC_QUEUE_CAPACITY,
        )
    }
}

impl InternalTypedReactionRuntime {
    pub fn with_world_and_queue_capacity(
        world: BTreeMap<String, String>,
        queue_capacity: usize,
    ) -> Self {
        Self {
            world,
            revision: 0,
            queue: LogicalAsyncQueue::with_capacity(queue_capacity),
            published_results: Vec::new(),
            published_traces: Vec::new(),
            published_receipts: Vec::new(),
            released_effects: Vec::new(),
        }
    }

    pub fn world(&self) -> &BTreeMap<String, String> {
        &self.world
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn published_results(&self) -> &[String] {
        &self.published_results
    }
    pub fn published_traces(&self) -> &[String] {
        &self.published_traces
    }
    pub fn published_receipts(&self) -> &[String] {
        &self.published_receipts
    }
    pub fn released_effects(&self) -> &[String] {
        &self.released_effects
    }
    pub fn pending_delivery_count(&self) -> usize {
        self.queue.len()
    }
    pub fn waiting_delivery_count(&self) -> usize {
        self.queue.waiting_len()
    }
    pub fn pop_next_delivery(&mut self) -> Option<InternalDeliveryEnvelopeSemantic> {
        self.queue.pop_front()
    }
    pub fn drain_resumed_waits(&mut self) -> Vec<LogicalWaitReceipt> {
        self.queue.drain_resumed_waits()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare<I>(
        &self,
        routes: &[EndpointRoute<InternalReactionProgram>],
        providers: &ReactionProviderRegistry<InternalReactionProgram>,
        reached_registration_keys: I,
        envelope: &InternalDeliveryEnvelopeSemantic,
        execution_identity: ExecutionInstanceIdentity,
        occurrence_semantic_identity: impl Into<String>,
    ) -> Result<InternalReactionPrepared, InternalReactionFailure>
    where
        I: IntoIterator<Item = ProviderRegistrationIdentity>,
    {
        let occurrence = plan_internal_delivery_occurrence(
            routes,
            providers,
            reached_registration_keys,
            envelope,
            execution_identity,
            occurrence_semantic_identity,
        )
        .map_err(map_internal_occurrence_failure)?;

        let settled = self.world.clone();
        let settled_snapshot_digest = internal_world_digest(&settled);
        let provider_registration_identities = occurrence.provider_registration_identities();
        let mut candidates = Vec::with_capacity(provider_registration_identities.len() + 1);
        candidates.push(evaluate_internal_program(
            format!("route:{}", occurrence.route.identity.as_str()),
            &occurrence.route.value,
            &settled,
        )?);
        for provider in occurrence.providers() {
            candidates.push(evaluate_internal_program(
                format!("provider:{}", provider.registration_identity.as_str()),
                &provider.value,
                &settled,
            )?);
        }
        let group = merge_internal_candidates(candidates)?;
        let reaction_digest = internal_reaction_group_digest(&group);

        Ok(InternalReactionPrepared {
            route_identity: occurrence.route.identity.clone(),
            execution_identity: occurrence.execution_identity,
            occurrence_identity: occurrence.occurrence_identity,
            provider_registration_identities,
            settled_snapshot_digest,
            reaction_digest,
            group,
        })
    }

    pub fn commit_prepared<F>(
        &mut self,
        prepared: InternalReactionPrepared,
        mut release_effect: F,
    ) -> InternalReactionSuccess
    where
        F: FnMut(&InternalEffectIntent, &BTreeMap<String, String>, u64) -> Result<(), String>,
    {
        let InternalReactionPrepared {
            route_identity,
            execution_identity,
            occurrence_identity,
            provider_registration_identities,
            settled_snapshot_digest,
            reaction_digest,
            group,
        } = prepared;

        let mut committed_world = self.world.clone();
        committed_world.extend(group.writes.clone());
        let committed_revision = self
            .revision
            .checked_add(1)
            .expect("internal revision overflow");
        let mut wait_receipts = Vec::new();
        for (index, staged) in group.staged_outbox.iter().cloned().enumerate() {
            if let QueueAdmission::Waiting(receipt) = self.queue.enqueue_or_wait(
                "internal-reaction-outbox",
                &format!("{}:{}", occurrence_identity.as_str(), index),
                staged,
            ) {
                wait_receipts.push(receipt);
            }
        }

        self.world = committed_world;
        self.revision = committed_revision;
        self.published_results
            .extend(group.result_candidates.clone());
        self.published_traces.extend(group.trace_candidates.clone());
        self.published_receipts
            .extend(group.receipt_candidates.clone());

        let mut released_effect_identities = Vec::new();
        let mut external_release_failures = Vec::new();
        for effect in &group.effect_intents {
            match release_effect(effect, &self.world, self.revision) {
                Ok(()) => {
                    released_effect_identities.push(effect.identity.clone());
                    self.released_effects.push(effect.identity.clone());
                }
                Err(detail) => {
                    external_release_failures.push(format!("{}:{}", effect.identity, detail))
                }
            }
        }

        InternalReactionSuccess {
            route_identity,
            execution_identity,
            occurrence_identity,
            provider_registration_identities,
            settled_snapshot_digest,
            reaction_digest,
            committed_revision,
            wait_receipts,
            released_effect_identities,
            external_release_failures,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute<F, I>(
        &mut self,
        routes: &[EndpointRoute<InternalReactionProgram>],
        providers: &ReactionProviderRegistry<InternalReactionProgram>,
        reached_registration_keys: I,
        envelope: &InternalDeliveryEnvelopeSemantic,
        execution_identity: ExecutionInstanceIdentity,
        occurrence_semantic_identity: impl Into<String>,
        release_effect: F,
    ) -> Result<InternalReactionSuccess, InternalReactionFailure>
    where
        F: FnMut(&InternalEffectIntent, &BTreeMap<String, String>, u64) -> Result<(), String>,
        I: IntoIterator<Item = ProviderRegistrationIdentity>,
    {
        let prepared = self.prepare(
            routes,
            providers,
            reached_registration_keys,
            envelope,
            execution_identity,
            occurrence_semantic_identity,
        )?;
        Ok(self.commit_prepared(prepared, release_effect))
    }
}

pub(super) fn map_internal_occurrence_failure(
    failure: InternalDeliveryOccurrenceFailure,
) -> InternalReactionFailure {
    match failure {
        InternalDeliveryOccurrenceFailure::Endpoint(EndpointRouteFailure::Unhandled) => {
            InternalReactionFailure::RouteUnhandled
        }
        InternalDeliveryOccurrenceFailure::Endpoint(EndpointRouteFailure::Ambiguous { count }) => {
            InternalReactionFailure::RouteAmbiguous { count }
        }
        InternalDeliveryOccurrenceFailure::Provider(detail) => {
            InternalReactionFailure::ProviderIdentity {
                detail: format!("{detail:?}"),
            }
        }
    }
}

pub(super) fn internal_reaction_failure_to_eval(
    failure: InternalReactionFailure,
    span: ddonirang_lang::Span,
) -> EvalError {
    match failure {
        InternalReactionFailure::RouteUnhandled => EvalError::async_candidate(
            "E_ASYNC_ENDPOINT_UNHANDLED_CANDIDATE",
            "async_route",
            "matching route count is 0",
            span,
        ),
        InternalReactionFailure::RouteAmbiguous { count } => EvalError::async_candidate(
            "E_ASYNC_ENDPOINT_AMBIGUOUS_CANDIDATE",
            "async_route",
            format!("matching route count is {count}"),
            span,
        ),
        failure @ InternalReactionFailure::ProviderIdentity { .. } => EvalError::async_candidate(
            failure.product_local_candidate_code(),
            "async_provider",
            format!("internal provider identity failure: {failure:?}"),
            span,
        ),
        failure @ InternalReactionFailure::BodyFailure { .. } => EvalError::async_candidate(
            failure.product_local_candidate_code(),
            "async_provider_body",
            format!("internal provider body failure: {failure:?}"),
            span,
        ),
        failure @ InternalReactionFailure::PatchConflict { .. } => EvalError::async_candidate(
            failure.product_local_candidate_code(),
            "async_candidate_merge",
            format!("internal provider candidate merge failure: {failure:?}"),
            span,
        ),
    }
}

pub(super) fn evaluate_internal_program(
    participant_identity: String,
    program: &InternalReactionProgram,
    settled: &BTreeMap<String, String>,
) -> Result<ReactionProviderPatchCandidate, InternalReactionFailure> {
    let mut candidate = ReactionProviderPatchCandidate {
        participant_identity: participant_identity.clone(),
        writes: BTreeMap::new(),
        staged_outbox: Vec::new(),
        result_candidates: Vec::new(),
        trace_candidates: Vec::new(),
        receipt_candidates: Vec::new(),
        effect_intents: Vec::new(),
    };
    for operation in &program.operations {
        match operation {
            InternalReactionOperation::Set { key, value } => {
                candidate.writes.insert(key.clone(), value.clone());
            }
            InternalReactionOperation::RequireSettledValue { key, expected } => {
                if settled.get(key) != expected.as_ref() {
                    return Err(InternalReactionFailure::BodyFailure {
                        participant_identity,
                        code: "E_INTERNAL_SETTLED_SNAPSHOT_MISMATCH_CANDIDATE".into(),
                    });
                }
            }
            InternalReactionOperation::StageSend(value) => {
                candidate.staged_outbox.push(value.clone())
            }
            InternalReactionOperation::StageEffect(value) => {
                candidate.effect_intents.push(value.clone())
            }
            InternalReactionOperation::EmitResult(value) => {
                candidate.result_candidates.push(value.clone())
            }
            InternalReactionOperation::EmitTrace(value) => {
                candidate.trace_candidates.push(value.clone())
            }
            InternalReactionOperation::EmitReceipt(value) => {
                candidate.receipt_candidates.push(value.clone())
            }
            InternalReactionOperation::Fail { code } => {
                return Err(InternalReactionFailure::BodyFailure {
                    participant_identity,
                    code: code.clone(),
                });
            }
        }
    }
    Ok(candidate)
}

pub(super) fn merge_internal_candidates(
    mut candidates: Vec<ReactionProviderPatchCandidate>,
) -> Result<ReactionGroupCommitCandidate, InternalReactionFailure> {
    candidates.sort_by(|left, right| left.participant_identity.cmp(&right.participant_identity));
    let mut group = ReactionGroupCommitCandidate {
        writes: BTreeMap::new(),
        staged_outbox: Vec::new(),
        result_candidates: Vec::new(),
        trace_candidates: Vec::new(),
        receipt_candidates: Vec::new(),
        effect_intents: Vec::new(),
        canonical_participants: Vec::new(),
    };
    let mut write_owners = BTreeMap::<String, String>::new();
    for candidate in candidates {
        for (key, value) in &candidate.writes {
            if let Some(previous) = group.writes.get(key) {
                if previous != value {
                    return Err(InternalReactionFailure::PatchConflict {
                        key: key.clone(),
                        first_participant: write_owners[key].clone(),
                        second_participant: candidate.participant_identity.clone(),
                    });
                }
            } else {
                group.writes.insert(key.clone(), value.clone());
                write_owners.insert(key.clone(), candidate.participant_identity.clone());
            }
        }
        group.staged_outbox.extend(candidate.staged_outbox);
        group.result_candidates.extend(candidate.result_candidates);
        group.trace_candidates.extend(candidate.trace_candidates);
        group
            .receipt_candidates
            .extend(candidate.receipt_candidates);
        group.effect_intents.extend(candidate.effect_intents);
        group
            .canonical_participants
            .push(candidate.participant_identity);
    }
    Ok(group)
}

pub(super) fn internal_hash_text(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

pub(super) fn internal_world_digest(world: &BTreeMap<String, String>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ddonirang.internal-reaction-settled-snapshot.candidate");
    for (key, value) in world {
        internal_hash_text(&mut hasher, key);
        internal_hash_text(&mut hasher, value);
    }
    hasher.finalize().to_hex().to_string()
}

pub(super) fn internal_reaction_group_digest(group: &ReactionGroupCommitCandidate) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ddonirang.internal-reaction-group.candidate");
    for participant in &group.canonical_participants {
        internal_hash_text(&mut hasher, participant);
    }
    for (key, value) in &group.writes {
        internal_hash_text(&mut hasher, key);
        internal_hash_text(&mut hasher, value);
    }
    for value in &group.result_candidates {
        internal_hash_text(&mut hasher, value);
    }
    for value in &group.trace_candidates {
        internal_hash_text(&mut hasher, value);
    }
    for value in &group.receipt_candidates {
        internal_hash_text(&mut hasher, value);
    }
    for delivery in &group.staged_outbox {
        internal_hash_text(&mut hasher, &delivery.receiver_identity);
        internal_hash_text(&mut hasher, &delivery.message_type);
        internal_hash_text(&mut hasher, &delivery.sender_identity);
        internal_hash_text(&mut hasher, &delivery.payload_identity);
    }
    for effect in &group.effect_intents {
        internal_hash_text(&mut hasher, &effect.identity);
    }
    hasher.finalize().to_hex().to_string()
}

