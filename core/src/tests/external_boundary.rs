use crate::{
    replay_sealed_external_execution, CanonicalExternalInput, CapabilityBinding, CapabilityRef,
    CapabilityScope, CapabilityValue, DependencyNodeKind, DeterministicSealedProvider,
    ExternalBoundaryFailure, ExternalBoundaryRequest, ExternalDependencyNode,
    ProcessLocalCapabilityStore, SealedExecutionLedger,
};
use std::cell::Cell;

struct CountingProvider {
    calls: Cell<u64>,
    fail: bool,
}

impl CountingProvider {
    fn sealed() -> Self { Self { calls: Cell::new(0), fail: false } }
    fn failing() -> Self { Self { calls: Cell::new(0), fail: true } }
}

impl crate::SealedBoundaryProvider for CountingProvider {
    fn provider_owner(&self) -> &str { "provider:sealed" }
    fn callable_owner(&self) -> &str { "callable:transform" }

    fn execute_sealed(
        &self,
        input: &CanonicalExternalInput,
    ) -> Result<String, ExternalBoundaryFailure> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            Err(ExternalBoundaryFailure::ProviderFailure)
        } else {
            Ok(format!("counted:{}", input.canonical_text))
        }
    }
}

fn binding() -> CapabilityBinding { CapabilityBinding::new("owner-a", "world-a", 7, 3) }

fn dependency(kind: DependencyNodeKind) -> ExternalDependencyNode {
    ExternalDependencyNode {
        kind,
        artifact_identity: "artifact:sealed-01".to_string(),
        provenance_identity: "provenance:sealed-01".to_string(),
        provider_owner: "provider:sealed".to_string(),
        callable_owner: "callable:transform".to_string(),
        invocation_identity: "invocation:one".to_string(),
        execution_contract: "contract:sealed-v1".to_string(),
        external_boundary_identity: "boundary:one".to_string(),
    }
}

fn store() -> ProcessLocalCapabilityStore {
    let mut store = ProcessLocalCapabilityStore::new(7);
    assert!(store.insert(CapabilityValue::new(
        CapabilityRef::from_process_local(41),
        binding(),
        [CapabilityScope::ExternalBoundaryExecute { boundary_id: "boundary:one".to_string() }],
    )));
    store
}

fn request(kind: DependencyNodeKind, ordinal: u64) -> ExternalBoundaryRequest {
    ExternalBoundaryRequest {
        dependency: dependency(kind),
        input: CanonicalExternalInput { canonical_text: "입력:42".to_string() },
        capability: Some(CapabilityRef::from_process_local(41)),
        capability_binding: binding(),
        max_input_bytes: 64,
        max_output_bytes: 64,
        max_calls: 2,
        call_ordinal: ordinal,
    }
}

#[test]
fn ebc_p01_p02_both_typed_node_kinds_execute_through_one_boundary() {
    for kind in [DependencyNodeKind::DdnGaji, DependencyNodeKind::ExternalArtifact] {
        let mut ledger = SealedExecutionLedger::new();
        let result = ledger.execute(
            &store(),
            &DeterministicSealedProvider::new("provider:sealed", "callable:transform"),
            &request(kind, 1),
        ).unwrap();
        assert_eq!(result.output, "sealed:24:력입");
        assert_eq!(result.dispatch_count, 1);
        assert_eq!(result.publication_count, 1);
        assert_eq!(ledger.published(), std::slice::from_ref(&result.receipt));
        assert_eq!(replay_sealed_external_execution(&request(kind, 1).dependency, &request(kind, 1).input, &result.output, &result.receipt), Ok(()));
    }
}

#[test]
fn ebc_rejections_are_pre_dispatch_and_pre_publication() {
    let provider = CountingProvider::sealed();
    let mut ledger = SealedExecutionLedger::new();
    let mut missing = request(DependencyNodeKind::ExternalArtifact, 1);
    missing.capability = None;
    assert_eq!(ledger.execute(&store(), &provider, &missing), Err(ExternalBoundaryFailure::CapabilityDenied));
    assert!(ledger.published().is_empty());

    let mut wrong_provider = request(DependencyNodeKind::ExternalArtifact, 1);
    wrong_provider.dependency.provider_owner = "provider:other".to_string();
    assert_eq!(ledger.execute(&store(), &provider, &wrong_provider), Err(ExternalBoundaryFailure::ProviderIdentityMismatch));
    assert!(ledger.published().is_empty());

    let mut oversized = request(DependencyNodeKind::ExternalArtifact, 1);
    oversized.max_input_bytes = 1;
    assert_eq!(ledger.execute(&store(), &provider, &oversized), Err(ExternalBoundaryFailure::InputLimitExceeded));
    assert!(ledger.published().is_empty());
    assert_eq!(provider.calls.get(), 0, "all rejections must precede provider dispatch");

    let mut wrong_scope = request(DependencyNodeKind::ExternalArtifact, 1);
    wrong_scope.dependency.external_boundary_identity = "boundary:other".to_string();
    assert_eq!(ledger.execute(&store(), &provider, &wrong_scope), Err(ExternalBoundaryFailure::CapabilityDenied));
    assert!(ledger.published().is_empty());
    assert_eq!(provider.calls.get(), 0);
}

#[test]
fn ebc_provider_failure_is_pre_publication_and_does_not_advance_the_ordinal() {
    let provider = CountingProvider::failing();
    let mut ledger = SealedExecutionLedger::new();
    let request = request(DependencyNodeKind::ExternalArtifact, 1);
    assert_eq!(ledger.execute(&store(), &provider, &request), Err(ExternalBoundaryFailure::ProviderFailure));
    assert_eq!(provider.calls.get(), 1);
    assert!(ledger.published().is_empty());
    assert_eq!(ledger.execute(&store(), &provider, &request), Err(ExternalBoundaryFailure::ProviderFailure));
    assert_eq!(provider.calls.get(), 2, "failure must not consume a call ordinal");
    assert!(ledger.published().is_empty());
}

#[test]
fn ebc_receipt_tamper_and_replay_do_not_dispatch_or_publish() {
    let provider = DeterministicSealedProvider::new("provider:sealed", "callable:transform");
    let mut ledger = SealedExecutionLedger::new();
    let request = request(DependencyNodeKind::ExternalArtifact, 1);
    let result = ledger.execute(&store(), &provider, &request).unwrap();
    let published_before = ledger.published().len();

    assert_eq!(replay_sealed_external_execution(&request.dependency, &request.input, "tampered", &result.receipt), Err(ExternalBoundaryFailure::ReceiptMismatch));
    let mut tampered = result.receipt.clone();
    tampered.call_ordinal = 2;
    assert_eq!(replay_sealed_external_execution(&request.dependency, &request.input, &result.output, &tampered), Err(ExternalBoundaryFailure::ReceiptMismatch));
    assert_eq!(ledger.published().len(), published_before);
}
