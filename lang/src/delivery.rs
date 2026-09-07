//! v24.45.1 successor delivery identities and logical queue contract.
//!
//! This module is deliberately free of executor, thread, and declaration-order
//! concepts. Native, WASM, and CLI adapters must share these discriminants and
//! cardinality rules instead of recreating delivery semantics locally.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::num::NonZeroUsize;
use unicode_normalization::UnicodeNormalization;

pub const DEFAULT_ASYNC_QUEUE_CAPACITY: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AsyncSend;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RendezvousSend;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RendezvousReceive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HistoricalAsyncSend {
    _migration_evidence_only: (),
}

/// A statement carries exactly one transfer identity. Historical `~~>` is
/// represented explicitly and is never inferred to be successor async.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageTransfer {
    Async(AsyncSend),
    Rendezvous(RendezvousSend),
    HistoricalAsync(HistoricalAsyncSend),
}

impl MessageTransfer {
    pub const ASYNC: Self = Self::Async(AsyncSend);
    pub const RENDEZVOUS: Self = Self::Rendezvous(RendezvousSend);
    /// Explicit migration-evidence constructor. Product parsers must not call this.
    pub const fn historical_async_for_migration() -> Self {
        Self::HistoricalAsync(HistoricalAsyncSend {
            _migration_evidence_only: (),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RouteIdentity(String);

impl RouteIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RouteIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SenderIdentityPredicate {
    AnySender,
    Exact(String),
}

impl SenderIdentityPredicate {
    pub fn exact(identity: impl Into<String>) -> Self {
        Self::Exact(canonical_text(&identity.into()))
    }

    pub fn canonical_material(&self) -> String {
        match self {
            Self::AnySender => "ANY_SENDER".to_string(),
            Self::Exact(identity) => format!("EXACT:{}:{}", identity.len(), identity),
        }
    }

    pub fn matches(&self, sender_identity: &str) -> bool {
        match self {
            Self::AnySender => true,
            Self::Exact(expected) => expected == &canonical_text(sender_identity),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointRouteSemantic {
    pub receiver_identity: String,
    pub message_type: String,
    pub sender_predicate: SenderIdentityPredicate,
    pub payload_predicate: String,
    pub owner_identity: String,
}

impl EndpointRouteSemantic {
    pub fn new(
        receiver_identity: impl Into<String>,
        message_type: impl Into<String>,
        sender_predicate: SenderIdentityPredicate,
        payload_predicate: impl Into<String>,
        owner_identity: impl Into<String>,
    ) -> Self {
        Self {
            receiver_identity: canonical_text(&receiver_identity.into()),
            message_type: canonical_text(&message_type.into()),
            sender_predicate,
            payload_predicate: canonical_text(&payload_predicate.into()),
            owner_identity: canonical_text(&owner_identity.into()),
        }
    }

    pub fn route_identity(&self) -> RouteIdentity {
        RouteIdentity(semantic_digest(
            "ddonirang.endpoint-route-identity.v1",
            &[
                ("receiver", &self.receiver_identity),
                ("message-type", &self.message_type),
                ("sender", &self.sender_predicate.canonical_material()),
                ("payload", &self.payload_predicate),
                ("owner", &self.owner_identity),
            ],
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointRoute<T> {
    pub identity: RouteIdentity,
    pub semantic: EndpointRouteSemantic,
    pub value: T,
}

impl<T> EndpointRoute<T> {
    pub fn new(semantic: EndpointRouteSemantic, value: T) -> Self {
        let identity = semantic.route_identity();
        Self {
            identity,
            semantic,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderIdentity(String);

impl ProviderIdentity {
    pub fn derive(
        module_owner_identity: impl Into<String>,
        declaration_identity: impl Into<String>,
    ) -> Self {
        let module_owner_identity = canonical_text(&module_owner_identity.into());
        let declaration_identity = canonical_text(&declaration_identity.into());
        Self(semantic_digest(
            "ddonirang.reaction-provider-identity.v1",
            &[
                ("module-owner", &module_owner_identity),
                ("declaration", &declaration_identity),
            ],
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionProvider<T> {
    pub identity: ProviderIdentity,
    pub value: T,
}

impl<T> ReactionProvider<T> {
    pub fn new(
        module_owner_identity: impl Into<String>,
        declaration_identity: impl Into<String>,
        value: T,
    ) -> Self {
        Self {
            identity: ProviderIdentity::derive(module_owner_identity, declaration_identity),
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderIdentityFailure {
    DuplicateRegistration { identity: ProviderIdentity },
    UnknownProvider { identity: ProviderIdentity },
}

macro_rules! opaque_internal_identity {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

opaque_internal_identity!(ProviderDeclarationIdentity);
opaque_internal_identity!(ProviderOwnerIdentity);
opaque_internal_identity!(ProviderRegistrationIdentity);
opaque_internal_identity!(ExecutionInstanceIdentity);
opaque_internal_identity!(DeliveryOccurrenceIdentity);

impl ProviderDeclarationIdentity {
    /// Internal semantic declaration identity. Source spans and declaration
    /// order are provenance only; a body semantic change is identity-bearing.
    pub fn derive(
        module_identity: impl Into<String>,
        declaration_semantic_identity: impl Into<String>,
        body_semantic_identity: impl Into<String>,
    ) -> Self {
        let module_identity = canonical_text(&module_identity.into());
        let declaration_semantic_identity = canonical_text(&declaration_semantic_identity.into());
        let body_semantic_identity = canonical_text(&body_semantic_identity.into());
        Self(semantic_digest(
            "ddonirang.internal-provider-declaration.candidate",
            &[
                ("module", &module_identity),
                ("declaration", &declaration_semantic_identity),
                ("body", &body_semantic_identity),
            ],
        ))
    }
}

impl ProviderOwnerIdentity {
    pub fn derive(
        owner_space_identity: impl Into<String>,
        owner_instance_identity: impl Into<String>,
    ) -> Self {
        let owner_space_identity = canonical_text(&owner_space_identity.into());
        let owner_instance_identity = canonical_text(&owner_instance_identity.into());
        Self(semantic_digest(
            "ddonirang.internal-provider-owner.candidate",
            &[
                ("owner-space", &owner_space_identity),
                ("owner-instance", &owner_instance_identity),
            ],
        ))
    }
}

impl ProviderRegistrationIdentity {
    pub fn derive(
        declaration: &ProviderDeclarationIdentity,
        owner: &ProviderOwnerIdentity,
        registration_semantic_identity: impl Into<String>,
    ) -> Self {
        let registration_semantic_identity = canonical_text(&registration_semantic_identity.into());
        Self(semantic_digest(
            "ddonirang.internal-provider-registration.candidate",
            &[
                ("declaration", declaration.as_str()),
                ("owner", owner.as_str()),
                ("registration", &registration_semantic_identity),
            ],
        ))
    }
}

impl ExecutionInstanceIdentity {
    pub fn derive(execution_semantic_identity: impl Into<String>) -> Self {
        let execution_semantic_identity = canonical_text(&execution_semantic_identity.into());
        Self(semantic_digest(
            "ddonirang.internal-execution-instance.candidate",
            &[("execution", &execution_semantic_identity)],
        ))
    }
}

impl DeliveryOccurrenceIdentity {
    pub fn derive(
        execution: &ExecutionInstanceIdentity,
        occurrence_semantic_identity: impl Into<String>,
    ) -> Self {
        let occurrence_semantic_identity = canonical_text(&occurrence_semantic_identity.into());
        Self(semantic_digest(
            "ddonirang.internal-delivery-occurrence.candidate",
            &[
                ("execution", execution.as_str()),
                ("occurrence", &occurrence_semantic_identity),
            ],
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalDeliveryEnvelopeSemantic {
    pub receiver_identity: String,
    pub message_type: String,
    pub sender_identity: String,
    pub payload_identity: String,
}

impl InternalDeliveryEnvelopeSemantic {
    pub fn new(
        receiver_identity: impl Into<String>,
        message_type: impl Into<String>,
        sender_identity: impl Into<String>,
        payload_identity: impl Into<String>,
    ) -> Self {
        Self {
            receiver_identity: canonical_text(&receiver_identity.into()),
            message_type: canonical_text(&message_type.into()),
            sender_identity: canonical_text(&sender_identity.into()),
            payload_identity: canonical_text(&payload_identity.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionProviderPredicate {
    pub receiver_identity: String,
    pub message_type: String,
    pub sender_predicate: SenderIdentityPredicate,
    pub payload_predicate: String,
}

impl ReactionProviderPredicate {
    pub fn new(
        receiver_identity: impl Into<String>,
        message_type: impl Into<String>,
        sender_predicate: SenderIdentityPredicate,
        payload_predicate: impl Into<String>,
    ) -> Self {
        Self {
            receiver_identity: canonical_text(&receiver_identity.into()),
            message_type: canonical_text(&message_type.into()),
            sender_predicate,
            payload_predicate: canonical_text(&payload_predicate.into()),
        }
    }

    pub fn matches(&self, envelope: &InternalDeliveryEnvelopeSemantic) -> bool {
        self.receiver_identity == envelope.receiver_identity
            && self.message_type == envelope.message_type
            && self.sender_predicate.matches(&envelope.sender_identity)
            && (self.payload_predicate == "ANY_PAYLOAD"
                || self.payload_predicate == envelope.payload_identity)
    }
}

/// Product-internal typed descriptor. Its five identity axes remain distinct
/// fields and no serialization or public declaration surface is provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionProviderDescriptor<T> {
    pub provider_identity: ProviderIdentity,
    pub declaration_identity: ProviderDeclarationIdentity,
    pub owner_identity: ProviderOwnerIdentity,
    pub registration_identity: ProviderRegistrationIdentity,
    pub predicate: ReactionProviderPredicate,
    pub value: T,
}

impl<T> ReactionProviderDescriptor<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        module_identity: impl Into<String>,
        declaration_semantic_identity: impl Into<String>,
        body_semantic_identity: impl Into<String>,
        owner_space_identity: impl Into<String>,
        owner_instance_identity: impl Into<String>,
        registration_semantic_identity: impl Into<String>,
        predicate: ReactionProviderPredicate,
        value: T,
    ) -> Self {
        let module_identity = canonical_text(&module_identity.into());
        let declaration_semantic_identity = canonical_text(&declaration_semantic_identity.into());
        let body_semantic_identity = canonical_text(&body_semantic_identity.into());
        let declaration_identity = ProviderDeclarationIdentity::derive(
            &module_identity,
            &declaration_semantic_identity,
            &body_semantic_identity,
        );
        let owner_identity =
            ProviderOwnerIdentity::derive(owner_space_identity, owner_instance_identity);
        let registration_identity = ProviderRegistrationIdentity::derive(
            &declaration_identity,
            &owner_identity,
            registration_semantic_identity,
        );
        let provider_identity = ProviderIdentity::derive(
            module_identity,
            format!(
                "{}\u{001f}{}",
                declaration_semantic_identity, body_semantic_identity
            ),
        );
        Self {
            provider_identity,
            declaration_identity,
            owner_identity,
            registration_identity,
            predicate,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalProviderIdentityFailure {
    DuplicateDefinition {
        registration_identity: ProviderRegistrationIdentity,
    },
    UnknownRegistration {
        registration_identity: ProviderRegistrationIdentity,
    },
}

/// Internal W1-C1 registry. Public declaration syntax and stable wire remain
/// outside this type; adapters register only already-typed provider identities.
#[derive(Debug, Clone)]
pub struct ReactionProviderRegistry<T> {
    providers: BTreeMap<ProviderIdentity, ReactionProvider<T>>,
    internal_descriptors: BTreeMap<ProviderRegistrationIdentity, ReactionProviderDescriptor<T>>,
}

impl<T> Default for ReactionProviderRegistry<T> {
    fn default() -> Self {
        Self {
            providers: BTreeMap::new(),
            internal_descriptors: BTreeMap::new(),
        }
    }
}

impl<T> ReactionProviderRegistry<T> {
    pub fn register(
        &mut self,
        provider: ReactionProvider<T>,
    ) -> Result<(), ProviderIdentityFailure> {
        if self.providers.contains_key(&provider.identity) {
            return Err(ProviderIdentityFailure::DuplicateRegistration {
                identity: provider.identity,
            });
        }
        self.providers.insert(provider.identity.clone(), provider);
        Ok(())
    }

    pub fn resolve_occurrence<I>(
        &self,
        reached_provider_keys: I,
    ) -> Result<Vec<&ReactionProvider<T>>, ProviderIdentityFailure>
    where
        I: IntoIterator<Item = ProviderIdentity>,
    {
        canonical_provider_occurrences(reached_provider_keys)
            .into_iter()
            .map(|identity| {
                self.providers
                    .get(&identity)
                    .ok_or(ProviderIdentityFailure::UnknownProvider { identity })
            })
            .collect()
    }

    pub fn register_descriptor(
        &mut self,
        descriptor: ReactionProviderDescriptor<T>,
    ) -> Result<(), InternalProviderIdentityFailure> {
        if self
            .internal_descriptors
            .contains_key(&descriptor.registration_identity)
        {
            return Err(InternalProviderIdentityFailure::DuplicateDefinition {
                registration_identity: descriptor.registration_identity,
            });
        }
        self.internal_descriptors
            .insert(descriptor.registration_identity.clone(), descriptor);
        Ok(())
    }

    pub fn internal_len(&self) -> usize {
        self.internal_descriptors.len()
    }

    pub fn matching_registration_identities(
        &self,
        envelope: &InternalDeliveryEnvelopeSemantic,
    ) -> Vec<ProviderRegistrationIdentity> {
        self.internal_descriptors
            .values()
            .filter(|descriptor| descriptor.predicate.matches(envelope))
            .map(|descriptor| descriptor.registration_identity.clone())
            .collect()
    }

    pub fn resolve_internal_occurrence<I>(
        &self,
        reached_registration_keys: I,
        envelope: &InternalDeliveryEnvelopeSemantic,
    ) -> Result<Vec<&ReactionProviderDescriptor<T>>, InternalProviderIdentityFailure>
    where
        I: IntoIterator<Item = ProviderRegistrationIdentity>,
    {
        reached_registration_keys
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|registration_identity| {
                self.internal_descriptors.get(&registration_identity).ok_or(
                    InternalProviderIdentityFailure::UnknownRegistration {
                        registration_identity,
                    },
                )
            })
            .filter_map(|result| match result {
                Ok(descriptor) if descriptor.predicate.matches(envelope) => Some(Ok(descriptor)),
                Ok(_) => None,
                Err(failure) => Some(Err(failure)),
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalDeliveryOccurrenceFailure {
    Endpoint(EndpointRouteFailure),
    Provider(InternalProviderIdentityFailure),
}

pub struct InternalDeliveryOccurrence<'a, R, P> {
    pub route: &'a EndpointRoute<R>,
    pub execution_identity: ExecutionInstanceIdentity,
    pub occurrence_identity: DeliveryOccurrenceIdentity,
    providers: Vec<&'a ReactionProviderDescriptor<P>>,
}

impl<'a, R, P> InternalDeliveryOccurrence<'a, R, P> {
    pub fn provider_registration_identities(&self) -> Vec<ProviderRegistrationIdentity> {
        self.providers
            .iter()
            .map(|provider| provider.registration_identity.clone())
            .collect()
    }

    pub fn fan_out(&self, mut observe: impl FnMut(&ReactionProviderDescriptor<P>)) {
        for provider in &self.providers {
            observe(provider);
        }
    }

    pub fn providers(&self) -> &[&'a ReactionProviderDescriptor<P>] {
        &self.providers
    }
}

#[allow(clippy::too_many_arguments)]
pub fn plan_internal_delivery_occurrence<'a, R, P, I>(
    routes: &'a [EndpointRoute<R>],
    providers: &'a ReactionProviderRegistry<P>,
    reached_registration_keys: I,
    envelope: &InternalDeliveryEnvelopeSemantic,
    execution_identity: ExecutionInstanceIdentity,
    occurrence_semantic_identity: impl Into<String>,
) -> Result<InternalDeliveryOccurrence<'a, R, P>, InternalDeliveryOccurrenceFailure>
where
    I: IntoIterator<Item = ProviderRegistrationIdentity>,
{
    // The route cardinality gate is deliberately first. Provider resolution
    // must not run after unhandled or ambiguous consuming delivery.
    let route =
        select_exactly_one_endpoint(routes).map_err(InternalDeliveryOccurrenceFailure::Endpoint)?;
    let occurrence_identity =
        DeliveryOccurrenceIdentity::derive(&execution_identity, occurrence_semantic_identity);
    let providers = providers
        .resolve_internal_occurrence(reached_registration_keys, envelope)
        .map_err(InternalDeliveryOccurrenceFailure::Provider)?;
    Ok(InternalDeliveryOccurrence {
        route,
        execution_identity,
        occurrence_identity,
        providers,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryOccurrenceFailure {
    Endpoint(EndpointRouteFailure),
    Provider(ProviderIdentityFailure),
}

/// A single logical delivery occurrence. Endpoint cardinality is decided
/// before provider resolution, so unhandled/ambiguous delivery cannot execute
/// or even resolve providers.
pub struct DeliveryOccurrence<'a, R, P> {
    pub route: &'a EndpointRoute<R>,
    providers: Vec<&'a ReactionProvider<P>>,
}

impl<'a, R, P> DeliveryOccurrence<'a, R, P> {
    pub fn provider_identities(&self) -> Vec<ProviderIdentity> {
        self.providers
            .iter()
            .map(|provider| provider.identity.clone())
            .collect()
    }

    /// Canonical iteration is a deterministic internal observation form only;
    /// it does not expose provider physical execution order as language meaning.
    pub fn fan_out(&self, mut observe: impl FnMut(&ReactionProvider<P>)) {
        for provider in &self.providers {
            observe(provider);
        }
    }
}

pub fn plan_delivery_occurrence<'a, R, P, I>(
    routes: &'a [EndpointRoute<R>],
    providers: &'a ReactionProviderRegistry<P>,
    reached_provider_keys: I,
) -> Result<DeliveryOccurrence<'a, R, P>, DeliveryOccurrenceFailure>
where
    I: IntoIterator<Item = ProviderIdentity>,
{
    let route = select_exactly_one_endpoint(routes).map_err(DeliveryOccurrenceFailure::Endpoint)?;
    let providers = providers
        .resolve_occurrence(reached_provider_keys)
        .map_err(DeliveryOccurrenceFailure::Provider)?;
    Ok(DeliveryOccurrence { route, providers })
}

/// One delivery occurrence owns one provider-key set. Repeated diamond paths
/// collapse by semantic ProviderIdentity; distinct providers remain. The
/// returned canonical ordering is for deterministic trace/verification only,
/// not a language guarantee about physical execution order.
pub fn canonical_provider_occurrences<I>(identities: I) -> Vec<ProviderIdentity>
where
    I: IntoIterator<Item = ProviderIdentity>,
{
    identities
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn canonical_text(value: &str) -> String {
    value.nfc().collect()
}

fn semantic_digest(domain: &str, components: &[(&str, &str)]) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_component(&mut hasher, "domain", domain);
    for (label, value) in components {
        hash_component(&mut hasher, label, value);
    }
    hasher.finalize().to_hex().to_string()
}

fn hash_component(hasher: &mut blake3::Hasher, label: &str, value: &str) {
    hasher.update(&(label.len() as u64).to_le_bytes());
    hasher.update(label.as_bytes());
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersistentDependencyId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersistentDependency(pub PersistentDependencyId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointRouteFailure {
    Unhandled,
    Ambiguous { count: usize },
}

/// Selects a consuming endpoint by cardinality only. The caller must build the
/// candidate set from semantic predicates; source or worker order is irrelevant.
pub fn select_exactly_one_endpoint<T>(candidates: &[T]) -> Result<&T, EndpointRouteFailure> {
    match candidates {
        [only] => Ok(only),
        [] => Err(EndpointRouteFailure::Unhandled),
        many => Err(EndpointRouteFailure::Ambiguous { count: many.len() }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnqueueOutcome<E> {
    Enqueued,
    /// Capacity is full. Ownership is returned to the adapter so it can create
    /// a logical WAIT edge; silently dropping or converting this to success is
    /// forbidden.
    Wait(E),
}

/// Product-local logical phases used by deterministic W1-D~F evidence. These
/// are not a public wire or a promise about worker scheduling order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogicalDeliveryPhase {
    Admit,
    Wait,
    Resume,
    Start,
    Settle,
    Terminal,
    CleanupJoin,
    Preflight,
    Publish,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WaitNodeIdentity(String);

impl WaitNodeIdentity {
    pub fn derive(
        queue_identity: &str,
        operation_identity: &str,
        logical_admission_ordinal: u64,
    ) -> Self {
        let ordinal = logical_admission_ordinal.to_string();
        Self(semantic_digest(
            "ddonirang.logical-wait-node.v1",
            &[
                ("queue", &canonical_text(queue_identity)),
                ("operation", &canonical_text(operation_identity)),
                ("logical-admission-ordinal", &ordinal),
            ],
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WaitNodeIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WaitDomain {
    InternalSealed,
    ExternalSealedProvider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalWaitReceipt {
    pub node: WaitNodeIdentity,
    pub domain: WaitDomain,
    pub phase: LogicalDeliveryPhase,
    pub logical_admission_ordinal: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalWaitEdge {
    pub waiter: WaitNodeIdentity,
    pub satisfier_identity: String,
    pub domain: WaitDomain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitProgress {
    ProgressAvailable,
    ClosedInternalDeadlock,
    OpenExternalWait,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogicalWaitGraph {
    pub edges: Vec<LogicalWaitEdge>,
}

impl LogicalWaitGraph {
    pub fn classify(&self, enabled_internal_work: usize) -> WaitProgress {
        if enabled_internal_work > 0 || self.edges.is_empty() {
            return WaitProgress::ProgressAvailable;
        }
        if self
            .edges
            .iter()
            .any(|edge| edge.domain == WaitDomain::ExternalSealedProvider)
        {
            WaitProgress::OpenExternalWait
        } else {
            WaitProgress::ClosedInternalDeadlock
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueAdmission {
    Enqueued,
    /// The message remains owned by the queue and will be promoted in FIFO
    /// order when a slot is released. WAIT is therefore neither success nor a
    /// dropped message nor an immediate hard failure.
    Waiting(LogicalWaitReceipt),
}

#[derive(Debug, Clone)]
struct WaitingEnvelope<E> {
    envelope: E,
    receipt: LogicalWaitReceipt,
}

#[derive(Debug, Clone)]
pub struct LogicalAsyncQueue<E> {
    capacity: NonZeroUsize,
    pending: VecDeque<E>,
    waiting: VecDeque<WaitingEnvelope<E>>,
    resumed_waits: VecDeque<LogicalWaitReceipt>,
    next_logical_admission_ordinal: u64,
    draining: bool,
}

impl<E> Default for LogicalAsyncQueue<E> {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_ASYNC_QUEUE_CAPACITY)
    }
}

impl<E> LogicalAsyncQueue<E> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity: NonZeroUsize::new(capacity)
                .expect("logical async queue capacity must be non-zero"),
            pending: VecDeque::new(),
            waiting: VecDeque::new(),
            resumed_waits: VecDeque::new(),
            next_logical_admission_ordinal: 0,
            draining: false,
        }
    }

    pub fn enqueue(&mut self, envelope: E) -> EnqueueOutcome<E> {
        if self.pending.len() == self.capacity.get() {
            return EnqueueOutcome::Wait(envelope);
        }
        self.pending.push_back(envelope);
        EnqueueOutcome::Enqueued
    }

    /// Admits an async envelope without losing ownership at the bounded-full
    /// boundary. A full queue creates a sealed internal WAIT node; releasing a
    /// slot promotes the oldest waiter before later arrivals.
    pub fn enqueue_or_wait(
        &mut self,
        queue_identity: &str,
        operation_identity: &str,
        envelope: E,
    ) -> QueueAdmission {
        let logical_admission_ordinal = self.next_logical_admission_ordinal;
        self.next_logical_admission_ordinal = self
            .next_logical_admission_ordinal
            .checked_add(1)
            .expect("logical async admission ordinal overflow");
        if self.pending.len() < self.capacity.get() && self.waiting.is_empty() {
            self.pending.push_back(envelope);
            return QueueAdmission::Enqueued;
        }
        let receipt = LogicalWaitReceipt {
            node: WaitNodeIdentity::derive(
                queue_identity,
                operation_identity,
                logical_admission_ordinal,
            ),
            domain: WaitDomain::InternalSealed,
            phase: LogicalDeliveryPhase::Wait,
            logical_admission_ordinal,
        };
        self.waiting.push_back(WaitingEnvelope {
            envelope,
            receipt: receipt.clone(),
        });
        QueueAdmission::Waiting(receipt)
    }

    /// Returns false for nested dispatch. The nested sender has already queued
    /// at the tail and must not enter a handler directly.
    pub fn begin_drain(&mut self) -> bool {
        if self.draining {
            return false;
        }
        self.draining = true;
        true
    }

    pub fn pop_front(&mut self) -> Option<E> {
        let next = self.pending.pop_front();
        if next.is_some() {
            if let Some(waiter) = self.waiting.pop_front() {
                self.pending.push_back(waiter.envelope);
                self.resumed_waits.push_back(LogicalWaitReceipt {
                    phase: LogicalDeliveryPhase::Resume,
                    ..waiter.receipt
                });
            }
        }
        next
    }

    pub fn finish_drain(&mut self) {
        self.draining = false;
    }

    pub fn clear(&mut self) {
        self.pending.clear();
        self.waiting.clear();
        self.resumed_waits.clear();
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.waiting.is_empty()
    }

    pub fn waiting_len(&self) -> usize {
        self.waiting.len()
    }

    pub fn drain_resumed_waits(&mut self) -> Vec<LogicalWaitReceipt> {
        self.resumed_waits.drain(..).collect()
    }

    pub fn wait_graph(&self, queue_identity: &str) -> LogicalWaitGraph {
        LogicalWaitGraph {
            edges: self
                .waiting
                .iter()
                .map(|waiter| LogicalWaitEdge {
                    waiter: waiter.receipt.node.clone(),
                    satisfier_identity: format!("queue-space:{}", canonical_text(queue_identity)),
                    domain: waiter.receipt.domain,
                })
                .collect(),
        }
    }

    /// Authored cancellation removes exactly one stable waiter. It cannot
    /// cancel an already-published pending envelope or another occurrence.
    pub fn cancel_waiter(&mut self, identity: &WaitNodeIdentity) -> bool {
        let Some(index) = self
            .waiting
            .iter()
            .position(|waiter| &waiter.receipt.node == identity)
        else {
            return false;
        };
        self.waiting.remove(index);
        true
    }
}

/// Reaction-local outbox. Nested frames commit into their parent; only the
/// outermost successful frame yields messages for queue publication. Aborting
/// a frame discards exactly that frame and never leaks a partial delivery.
#[derive(Debug, Clone)]
pub struct StagedDeliveryOutbox<E> {
    frames: Vec<Vec<E>>,
}

impl<E> Default for StagedDeliveryOutbox<E> {
    fn default() -> Self {
        Self { frames: Vec::new() }
    }
}

impl<E> StagedDeliveryOutbox<E> {
    pub fn begin_frame(&mut self) {
        self.frames.push(Vec::new());
    }

    pub fn is_staging(&self) -> bool {
        !self.frames.is_empty()
    }

    pub fn stage(&mut self, envelope: E) {
        self.frames
            .last_mut()
            .expect("staged delivery requires an active outbox frame")
            .push(envelope);
    }

    pub fn commit_frame(&mut self) -> Vec<E> {
        let committed = self
            .frames
            .pop()
            .expect("outbox frame stack underflow on commit");
        if let Some(parent) = self.frames.last_mut() {
            parent.extend(committed);
            Vec::new()
        } else {
            committed
        }
    }

    pub fn abort_frame(&mut self) {
        self.frames
            .pop()
            .expect("outbox frame stack underflow on abort");
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_families_are_distinct() {
        assert_ne!(MessageTransfer::ASYNC, MessageTransfer::RENDEZVOUS);
        assert_ne!(
            MessageTransfer::ASYNC,
            MessageTransfer::historical_async_for_migration()
        );
        let _receive = RendezvousReceive;
        let route = EndpointRoute::new(
            EndpointRouteSemantic::new(
                "관제탑",
                "첫알림",
                SenderIdentityPredicate::AnySender,
                "ANY_PAYLOAD",
                "기본::관제탑",
            ),
            (),
        );
        let provider = ReactionProvider::new("기본::관제탑", "기록", ());
        assert_ne!(route.identity.as_str(), provider.identity.as_str());
        let _dependency = PersistentDependency(PersistentDependencyId(3));
    }

    #[test]
    fn route_identity_is_semantic_nfc_and_not_source_order() {
        let a = EndpointRouteSemantic::new(
            "관제탑",
            "첫알림",
            SenderIdentityPredicate::exact("철수"),
            "값>0",
            "기본::관제탑",
        );
        let b = EndpointRouteSemantic::new(
            "관제탑",
            "첫알림",
            SenderIdentityPredicate::exact("철수"),
            "값>0",
            "기본::관제탑",
        );
        assert_eq!(a.route_identity(), b.route_identity());

        let composed = EndpointRouteSemantic::new(
            "가\u{0301}",
            "첫알림",
            SenderIdentityPredicate::AnySender,
            "ANY_PAYLOAD",
            "기본",
        );
        let nfc = EndpointRouteSemantic::new(
            "가\u{0301}".nfc().collect::<String>(),
            "첫알림",
            SenderIdentityPredicate::AnySender,
            "ANY_PAYLOAD",
            "기본",
        );
        assert_eq!(composed.route_identity(), nfc.route_identity());

        let other_sender = EndpointRouteSemantic::new(
            "관제탑",
            "첫알림",
            SenderIdentityPredicate::exact("영희"),
            "값>0",
            "기본::관제탑",
        );
        assert_ne!(a.route_identity(), other_sender.route_identity());
    }

    #[test]
    fn equal_consuming_routes_remain_ambiguous() {
        let semantic = EndpointRouteSemantic::new(
            "관제탑",
            "첫알림",
            SenderIdentityPredicate::AnySender,
            "ANY_PAYLOAD",
            "기본::관제탑",
        );
        let candidates = vec![
            EndpointRoute::new(semantic.clone(), "a"),
            EndpointRoute::new(semantic, "b"),
        ];
        assert_eq!(candidates[0].identity, candidates[1].identity);
        assert_eq!(
            select_exactly_one_endpoint(&candidates),
            Err(EndpointRouteFailure::Ambiguous { count: 2 })
        );
    }

    #[test]
    fn provider_diamond_dedup_is_per_occurrence_and_canonical() {
        let a = ProviderIdentity::derive("기본::관제탑", "a");
        let b = ProviderIdentity::derive("기본::관제탑", "b");
        let first = canonical_provider_occurrences(vec![b.clone(), a.clone(), a.clone()]);
        assert_eq!(first.len(), 2);
        assert!(first[0] < first[1]);

        let nested = canonical_provider_occurrences(vec![a]);
        assert_eq!(nested.len(), 1, "nested occurrence owns a fresh dedup set");
    }

    #[test]
    fn delivery_occurrence_gates_provider_resolution_and_fans_out_distinct_keys() {
        let route = EndpointRoute::new(
            EndpointRouteSemantic::new(
                "관제탑",
                "수",
                SenderIdentityPredicate::AnySender,
                "ANY_PAYLOAD",
                "기본::관제탑",
            ),
            "consume",
        );
        let mut registry = ReactionProviderRegistry::default();
        let a = ReactionProvider::new("기본::관제탑", "관찰-a", "a");
        let b = ReactionProvider::new("기본::관제탑", "관찰-b", "b");
        let a_key = a.identity.clone();
        let b_key = b.identity.clone();
        registry.register(a).expect("register a");
        registry.register(b).expect("register b");

        let occurrence = plan_delivery_occurrence(
            std::slice::from_ref(&route),
            &registry,
            vec![b_key, a_key.clone(), a_key],
        )
        .expect("one route and valid provider keys");
        assert_eq!(occurrence.route.value, "consume");
        assert_eq!(occurrence.provider_identities().len(), 2);
        let mut observed = Vec::new();
        occurrence.fan_out(|provider| observed.push(provider.value));
        assert_eq!(observed.len(), 2);
    }

    #[test]
    fn route_failure_precedes_provider_identity_failure() {
        let registry = ReactionProviderRegistry::<()>::default();
        let unknown = ProviderIdentity::derive("기본::관제탑", "missing");
        assert!(matches!(
            plan_delivery_occurrence::<(), (), _>(&[], &registry, vec![unknown.clone()]),
            Err(DeliveryOccurrenceFailure::Endpoint(
                EndpointRouteFailure::Unhandled
            ))
        ));

        let semantic = EndpointRouteSemantic::new(
            "관제탑",
            "수",
            SenderIdentityPredicate::AnySender,
            "ANY_PAYLOAD",
            "기본::관제탑",
        );
        let routes = vec![
            EndpointRoute::new(semantic.clone(), ()),
            EndpointRoute::new(semantic, ()),
        ];
        assert!(matches!(
            plan_delivery_occurrence(&routes, &registry, vec![unknown]),
            Err(DeliveryOccurrenceFailure::Endpoint(
                EndpointRouteFailure::Ambiguous { count: 2 }
            ))
        ));
    }

    #[test]
    fn provider_registration_and_resolution_fail_closed() {
        let mut registry = ReactionProviderRegistry::default();
        let provider = ReactionProvider::new("기본::관제탑", "관찰", ());
        let key = provider.identity.clone();
        registry.register(provider).expect("first registration");
        assert!(matches!(
            registry.register(ReactionProvider::new("기본::관제탑", "관찰", ())),
            Err(ProviderIdentityFailure::DuplicateRegistration { .. })
        ));
        assert!(matches!(
            registry.resolve_occurrence(vec![ProviderIdentity::derive("기본::관제탑", "missing")]),
            Err(ProviderIdentityFailure::UnknownProvider { .. })
        ));
        assert_eq!(registry.resolve_occurrence(vec![key]).unwrap().len(), 1);
    }

    #[test]
    fn internal_provider_identity_axes_remain_distinct() {
        let declaration_a = ProviderDeclarationIdentity::derive("기본", "관찰", "body-a");
        let declaration_a_again = ProviderDeclarationIdentity::derive("기본", "관찰", "body-a");
        let declaration_body_changed =
            ProviderDeclarationIdentity::derive("기본", "관찰", "body-b");
        assert_eq!(declaration_a, declaration_a_again);
        assert_ne!(declaration_a, declaration_body_changed);

        let owner_a = ProviderOwnerIdentity::derive("기본::관제탑", "owner-a");
        let owner_b = ProviderOwnerIdentity::derive("기본::관제탑", "owner-b");
        assert_ne!(owner_a, owner_b);

        let registration_a =
            ProviderRegistrationIdentity::derive(&declaration_a, &owner_a, "registration-a");
        let registration_b =
            ProviderRegistrationIdentity::derive(&declaration_a, &owner_a, "registration-b");
        assert_ne!(registration_a, registration_b);

        let execution_a = ExecutionInstanceIdentity::derive("execution-a");
        let execution_b = ExecutionInstanceIdentity::derive("execution-b");
        assert_ne!(execution_a, execution_b);
        assert_ne!(
            DeliveryOccurrenceIdentity::derive(&execution_a, "delivery-1"),
            DeliveryOccurrenceIdentity::derive(&execution_a, "delivery-2")
        );
        assert_ne!(
            DeliveryOccurrenceIdentity::derive(&execution_a, "delivery-1"),
            DeliveryOccurrenceIdentity::derive(&execution_b, "delivery-1")
        );
    }

    #[test]
    fn internal_provider_identity_ignores_physical_provenance_and_worker_order() {
        let first = ReactionProviderDescriptor::new(
            "기본",
            "관찰",
            "body-a",
            "기본::관제탑",
            "owner-a",
            "registration-a",
            ReactionProviderPredicate::new(
                "관제탑",
                "첫알림",
                SenderIdentityPredicate::AnySender,
                "ANY_PAYLOAD",
            ),
            "source-span-1-worker-8",
        );
        let second = ReactionProviderDescriptor::new(
            "기본",
            "관찰",
            "body-a",
            "기본::관제탑",
            "owner-a",
            "registration-a",
            ReactionProviderPredicate::new(
                "관제탑",
                "첫알림",
                SenderIdentityPredicate::AnySender,
                "ANY_PAYLOAD",
            ),
            "source-span-99-worker-1",
        );
        assert_eq!(first.declaration_identity, second.declaration_identity);
        assert_eq!(first.owner_identity, second.owner_identity);
        assert_eq!(first.registration_identity, second.registration_identity);
    }

    #[test]
    fn endpoint_route_is_cardinality_only() {
        assert_eq!(
            select_exactly_one_endpoint::<u8>(&[]),
            Err(EndpointRouteFailure::Unhandled)
        );
        assert_eq!(select_exactly_one_endpoint(&[7]), Ok(&7));
        assert_eq!(
            select_exactly_one_endpoint(&[7, 8]),
            Err(EndpointRouteFailure::Ambiguous { count: 2 })
        );
    }

    #[test]
    fn queue_is_fifo_non_reentrant_and_waits_when_full() {
        let mut queue = LogicalAsyncQueue::with_capacity(2);
        assert_eq!(queue.enqueue(1), EnqueueOutcome::Enqueued);
        assert_eq!(queue.enqueue(2), EnqueueOutcome::Enqueued);
        assert_eq!(queue.enqueue(3), EnqueueOutcome::Wait(3));
        assert!(queue.begin_drain());
        assert!(!queue.begin_drain());
        assert_eq!(queue.pop_front(), Some(1));
        assert_eq!(queue.pop_front(), Some(2));
        queue.finish_drain();
        assert!(queue.is_empty());
    }

    #[test]
    fn bounded_full_wait_is_fifo_and_resumes_without_drop() {
        let mut queue = LogicalAsyncQueue::with_capacity(2);
        assert_eq!(
            queue.enqueue_or_wait("async", "op-1", 1),
            QueueAdmission::Enqueued
        );
        assert_eq!(
            queue.enqueue_or_wait("async", "op-2", 2),
            QueueAdmission::Enqueued
        );
        let wait = queue.enqueue_or_wait("async", "op-3", 3);
        assert!(matches!(wait, QueueAdmission::Waiting(_)));
        assert_eq!(queue.waiting_len(), 1);
        assert_eq!(queue.pop_front(), Some(1));
        assert_eq!(queue.pop_front(), Some(2));
        assert_eq!(queue.pop_front(), Some(3));
        let resumed = queue.drain_resumed_waits();
        assert_eq!(resumed.len(), 1);
        assert_eq!(resumed[0].phase, LogicalDeliveryPhase::Resume);
        assert!(queue.is_empty());
    }

    #[test]
    fn wait_nodes_are_occurrence_unique_and_graph_classifies_progress() {
        let mut queue = LogicalAsyncQueue::with_capacity(1);
        assert_eq!(
            queue.enqueue_or_wait("async", "same-op", 1),
            QueueAdmission::Enqueued
        );
        let QueueAdmission::Waiting(first) = queue.enqueue_or_wait("async", "same-op", 2) else {
            panic!("first waiter");
        };
        let QueueAdmission::Waiting(second) = queue.enqueue_or_wait("async", "same-op", 3) else {
            panic!("second waiter");
        };
        assert_ne!(first.node, second.node);
        assert_eq!(
            queue.wait_graph("async").classify(queue.len()),
            WaitProgress::ProgressAvailable
        );
        assert!(queue.cancel_waiter(&second.node));
        assert!(!queue.cancel_waiter(&second.node));
        assert_eq!(queue.waiting_len(), 1);
    }

    #[test]
    fn sealed_wait_graph_distinguishes_closed_deadlock_from_open_external() {
        let waiter = WaitNodeIdentity::derive("q", "op", 0);
        let closed = LogicalWaitGraph {
            edges: vec![LogicalWaitEdge {
                waiter: waiter.clone(),
                satisfier_identity: "internal-slot".to_string(),
                domain: WaitDomain::InternalSealed,
            }],
        };
        assert_eq!(closed.classify(0), WaitProgress::ClosedInternalDeadlock);
        let open = LogicalWaitGraph {
            edges: vec![LogicalWaitEdge {
                waiter,
                satisfier_identity: "sealed-provider".to_string(),
                domain: WaitDomain::ExternalSealedProvider,
            }],
        };
        assert_eq!(open.classify(0), WaitProgress::OpenExternalWait);
    }

    #[test]
    fn staged_outbox_commits_nested_fifo_and_aborts_without_leak() {
        let mut outbox = StagedDeliveryOutbox::default();
        outbox.begin_frame();
        outbox.stage(1);
        outbox.begin_frame();
        outbox.stage(2);
        assert!(outbox.commit_frame().is_empty());
        outbox.begin_frame();
        outbox.stage(99);
        outbox.abort_frame();
        outbox.stage(3);
        assert_eq!(outbox.commit_frame(), vec![1, 2, 3]);
        assert_eq!(outbox.depth(), 0);
    }
}
