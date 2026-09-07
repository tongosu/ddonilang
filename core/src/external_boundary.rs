//! Internal shared boundary for already-resolved dependency execution.
//!
//! These Rust names are implementation-only.  They deliberately assign no public
//! canonical name, CLI command, lock-file wire, or ecosystem-specific identity.

use crate::capability::{
    CapabilityAuthorizationRequest, CapabilityBinding, CapabilityOperation, CapabilityRef,
    ProcessLocalCapabilityStore,
};
use crate::commit_canon::domain_hash_text;

const INPUT_DOMAIN: &str = "ddn.external_boundary.input.v1-internal";
const OUTPUT_DOMAIN: &str = "ddn.external_boundary.output.v1-internal";
const RECEIPT_DOMAIN: &str = "ddn.external_boundary.receipt.v1-internal";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyNodeKind {
    DdnGaji,
    ExternalArtifact,
}

impl DependencyNodeKind {
    fn canonical_name(self) -> &'static str {
        match self {
            Self::DdnGaji => "DDN_GAJI",
            Self::ExternalArtifact => "EXTERNAL_ARTIFACT",
        }
    }
}

/// A dependency is resolved before this object is constructed.  There is exactly
/// one boundary identity slot; no provider-specific identity family is introduced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalDependencyNode {
    pub kind: DependencyNodeKind,
    pub artifact_identity: String,
    pub provenance_identity: String,
    pub provider_owner: String,
    pub callable_owner: String,
    pub invocation_identity: String,
    pub execution_contract: String,
    pub external_boundary_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalExternalInput {
    pub canonical_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalBoundaryRequest {
    pub dependency: ExternalDependencyNode,
    pub input: CanonicalExternalInput,
    pub capability: Option<CapabilityRef>,
    pub capability_binding: CapabilityBinding,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub max_calls: u64,
    pub call_ordinal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalBoundaryReceipt {
    pub dependency_kind: DependencyNodeKind,
    pub artifact_identity: String,
    pub provenance_identity: String,
    pub provider_owner: String,
    pub callable_owner: String,
    pub invocation_identity: String,
    pub execution_contract: String,
    pub external_boundary_identity: String,
    pub authorized_capability: CapabilityRef,
    pub canonical_input_hash: String,
    pub canonical_output_hash: String,
    pub call_ordinal: u64,
    pub receipt_seal: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalBoundaryResult {
    pub output: String,
    pub receipt: ExternalBoundaryReceipt,
    pub dispatch_count: u64,
    pub publication_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalBoundaryFailure {
    InvalidResolvedDependency,
    CapabilityDenied,
    InputLimitExceeded,
    OutputLimitExceeded,
    CallLimitExceeded,
    OrdinalMismatch,
    ProviderIdentityMismatch,
    ProviderFailure,
    ReceiptMismatch,
}

/// The common product interface admits only already-resolved data and a typed,
/// bounded execution context.  It has no resolver, network, filesystem, PATH,
/// environment, clock, random, or subprocess capability.
pub trait SealedBoundaryProvider {
    fn provider_owner(&self) -> &str;
    fn callable_owner(&self) -> &str;
    fn execute_sealed(&self, input: &CanonicalExternalInput) -> Result<String, ExternalBoundaryFailure>;
}

/// Deterministic internal provider used by the product boundary itself, not a
/// harness-only wrapper.  It binds the typed input to a stable transformed output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeterministicSealedProvider {
    provider_owner: String,
    callable_owner: String,
}

impl DeterministicSealedProvider {
    pub fn new(provider_owner: impl Into<String>, callable_owner: impl Into<String>) -> Self {
        Self {
            provider_owner: provider_owner.into(),
            callable_owner: callable_owner.into(),
        }
    }
}

impl SealedBoundaryProvider for DeterministicSealedProvider {
    fn provider_owner(&self) -> &str { &self.provider_owner }
    fn callable_owner(&self) -> &str { &self.callable_owner }

    fn execute_sealed(&self, input: &CanonicalExternalInput) -> Result<String, ExternalBoundaryFailure> {
        let mut output = String::from("sealed:");
        output.extend(input.canonical_text.chars().rev());
        Ok(output)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SealedExecutionLedger {
    next_ordinal: u64,
    published: Vec<ExternalBoundaryReceipt>,
}

impl SealedExecutionLedger {
    pub fn new() -> Self { Self { next_ordinal: 1, published: Vec::new() } }
    pub fn published(&self) -> &[ExternalBoundaryReceipt] { &self.published }

    pub fn execute(
        &mut self,
        capability_store: &ProcessLocalCapabilityStore,
        provider: &dyn SealedBoundaryProvider,
        request: &ExternalBoundaryRequest,
    ) -> Result<ExternalBoundaryResult, ExternalBoundaryFailure> {
        validate_dependency(&request.dependency)?;
        if request.call_ordinal != self.next_ordinal {
            return Err(ExternalBoundaryFailure::OrdinalMismatch);
        }
        if request.call_ordinal > request.max_calls {
            return Err(ExternalBoundaryFailure::CallLimitExceeded);
        }
        if request.input.canonical_text.as_bytes().len() > request.max_input_bytes {
            return Err(ExternalBoundaryFailure::InputLimitExceeded);
        }
        if provider.provider_owner() != request.dependency.provider_owner
            || provider.callable_owner() != request.dependency.callable_owner
        {
            return Err(ExternalBoundaryFailure::ProviderIdentityMismatch);
        }
        let authorization = capability_store.authorize(&CapabilityAuthorizationRequest {
            presented: request.capability,
            binding: request.capability_binding.clone(),
            operation: CapabilityOperation::ExternalBoundaryExecute {
                boundary_id: request.dependency.external_boundary_identity.clone(),
            },
        });
        let crate::capability::CapabilityAuthorization::Authorized { capability } = authorization else {
            return Err(ExternalBoundaryFailure::CapabilityDenied);
        };
        let output = provider.execute_sealed(&request.input)?;
        if output.as_bytes().len() > request.max_output_bytes {
            return Err(ExternalBoundaryFailure::OutputLimitExceeded);
        }
        let receipt = build_receipt(request, capability, &output);
        // The only publication happens after all validation and sealing succeeds.
        self.published.push(receipt.clone());
        self.next_ordinal += 1;
        Ok(ExternalBoundaryResult { output, receipt, dispatch_count: 1, publication_count: 1 })
    }
}

pub fn replay_sealed_external_execution(
    dependency: &ExternalDependencyNode,
    input: &CanonicalExternalInput,
    output: &str,
    receipt: &ExternalBoundaryReceipt,
) -> Result<(), ExternalBoundaryFailure> {
    validate_dependency(dependency)?;
    if receipt.dependency_kind != dependency.kind
        || receipt.artifact_identity != dependency.artifact_identity
        || receipt.provenance_identity != dependency.provenance_identity
        || receipt.provider_owner != dependency.provider_owner
        || receipt.callable_owner != dependency.callable_owner
        || receipt.invocation_identity != dependency.invocation_identity
        || receipt.execution_contract != dependency.execution_contract
        || receipt.external_boundary_identity != dependency.external_boundary_identity
        || receipt.canonical_input_hash != input_hash(input)
        || receipt.canonical_output_hash != output_hash(output)
    {
        return Err(ExternalBoundaryFailure::ReceiptMismatch);
    }
    let expected = receipt_seal(receipt);
    if expected != receipt.receipt_seal {
        return Err(ExternalBoundaryFailure::ReceiptMismatch);
    }
    Ok(())
}

fn validate_dependency(dependency: &ExternalDependencyNode) -> Result<(), ExternalBoundaryFailure> {
    let all = [
        &dependency.artifact_identity,
        &dependency.provenance_identity,
        &dependency.provider_owner,
        &dependency.callable_owner,
        &dependency.invocation_identity,
        &dependency.execution_contract,
        &dependency.external_boundary_identity,
    ];
    if all.iter().any(|value| value.is_empty()) {
        Err(ExternalBoundaryFailure::InvalidResolvedDependency)
    } else {
        Ok(())
    }
}

fn build_receipt(
    request: &ExternalBoundaryRequest,
    capability: CapabilityRef,
    output: &str,
) -> ExternalBoundaryReceipt {
    let mut receipt = ExternalBoundaryReceipt {
        dependency_kind: request.dependency.kind,
        artifact_identity: request.dependency.artifact_identity.clone(),
        provenance_identity: request.dependency.provenance_identity.clone(),
        provider_owner: request.dependency.provider_owner.clone(),
        callable_owner: request.dependency.callable_owner.clone(),
        invocation_identity: request.dependency.invocation_identity.clone(),
        execution_contract: request.dependency.execution_contract.clone(),
        external_boundary_identity: request.dependency.external_boundary_identity.clone(),
        authorized_capability: capability,
        canonical_input_hash: input_hash(&request.input),
        canonical_output_hash: output_hash(output),
        call_ordinal: request.call_ordinal,
        receipt_seal: String::new(),
    };
    receipt.receipt_seal = receipt_seal(&receipt);
    receipt
}

fn input_hash(input: &CanonicalExternalInput) -> String { domain_hash_text(INPUT_DOMAIN, &input.canonical_text) }
fn output_hash(output: &str) -> String { domain_hash_text(OUTPUT_DOMAIN, output) }

fn receipt_seal(receipt: &ExternalBoundaryReceipt) -> String {
    domain_hash_text(RECEIPT_DOMAIN, &format!(
        "kind={};artifact={};provenance={};provider={};callable={};invocation={};contract={};boundary={};capability={};input={};output={};ordinal={}",
        receipt.dependency_kind.canonical_name(), receipt.artifact_identity, receipt.provenance_identity,
        receipt.provider_owner, receipt.callable_owner, receipt.invocation_identity,
        receipt.execution_contract, receipt.external_boundary_identity, receipt.authorized_capability.raw(),
        receipt.canonical_input_hash, receipt.canonical_output_hash, receipt.call_ordinal,
    ))
}
