//! Process-local capability authorization for product-core execution boundaries.
//!
//! These values are runtime authority, not stable asset or resource identities. They are
//! intentionally absent from canonical world state and wire formats.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CapabilityRef(u64);

impl CapabilityRef {
    pub fn from_process_local(raw: u64) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityBinding {
    pub owner_id: String,
    pub world_id: String,
    pub process_id: u64,
    pub generation: u64,
}

impl CapabilityBinding {
    pub fn new(
        owner_id: impl Into<String>,
        world_id: impl Into<String>,
        process_id: u64,
        generation: u64,
    ) -> Self {
        Self {
            owner_id: owner_id.into(),
            world_id: world_id.into(),
            process_id,
            generation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityScope {
    ResourceHandleWrite { tag: String },
    /// Internal-only authority for one already sealed external-boundary contract.
    /// This is deliberately not a resource handle or a public callable name.
    ExternalBoundaryExecute { boundary_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityValue {
    pub reference: CapabilityRef,
    pub binding: CapabilityBinding,
    pub scopes: BTreeSet<CapabilityScope>,
    pub valid: bool,
}

impl CapabilityValue {
    pub fn new(
        reference: CapabilityRef,
        binding: CapabilityBinding,
        scopes: impl IntoIterator<Item = CapabilityScope>,
    ) -> Self {
        Self {
            reference,
            binding,
            scopes: scopes.into_iter().collect(),
            valid: true,
        }
    }

    pub fn with_validity(mut self, valid: bool) -> Self {
        self.valid = valid;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityOperation {
    ResourceHandleWrite { tags: Vec<String> },
    ExternalBoundaryExecute { boundary_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityAuthorizationRequest {
    pub presented: Option<CapabilityRef>,
    pub binding: CapabilityBinding,
    pub operation: CapabilityOperation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityAuthorization {
    Authorized { capability: CapabilityRef },
    MissingCapability,
    WrongCapability,
    WrongOwner,
    WrongWorld,
    WrongProcess,
    GenerationMismatch,
    InactiveCapability,
    ScopeDenied,
}

impl CapabilityAuthorization {
    pub fn is_authorized(&self) -> bool {
        matches!(self, Self::Authorized { .. })
    }
}

/// Runtime-only capability custody for exactly one host process.
#[derive(Clone, Debug)]
pub struct ProcessLocalCapabilityStore {
    process_id: u64,
    values: BTreeMap<CapabilityRef, CapabilityValue>,
}

impl ProcessLocalCapabilityStore {
    pub fn new(process_id: u64) -> Self {
        Self {
            process_id,
            values: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, capability: CapabilityValue) -> bool {
        if capability.binding.process_id != self.process_id {
            return false;
        }
        self.values
            .insert(capability.reference, capability)
            .is_none()
    }

    pub fn authorize(
        &self,
        request: &CapabilityAuthorizationRequest,
    ) -> CapabilityAuthorization {
        let Some(reference) = request.presented else {
            return CapabilityAuthorization::MissingCapability;
        };
        let Some(capability) = self.values.get(&reference) else {
            return CapabilityAuthorization::WrongCapability;
        };
        if request.binding.process_id != self.process_id
            || capability.binding.process_id != request.binding.process_id
        {
            return CapabilityAuthorization::WrongProcess;
        }
        if capability.binding.owner_id != request.binding.owner_id {
            return CapabilityAuthorization::WrongOwner;
        }
        if capability.binding.world_id != request.binding.world_id {
            return CapabilityAuthorization::WrongWorld;
        }
        if capability.binding.generation != request.binding.generation {
            return CapabilityAuthorization::GenerationMismatch;
        }
        if !capability.valid {
            return CapabilityAuthorization::InactiveCapability;
        }
        if !operation_is_allowed(capability, &request.operation) {
            return CapabilityAuthorization::ScopeDenied;
        }
        CapabilityAuthorization::Authorized {
            capability: reference,
        }
    }
}

fn operation_is_allowed(capability: &CapabilityValue, operation: &CapabilityOperation) -> bool {
    match operation {
        CapabilityOperation::ResourceHandleWrite { tags } => tags.iter().all(|tag| {
            capability
                .scopes
                .contains(&CapabilityScope::ResourceHandleWrite { tag: tag.clone() })
        }),
        CapabilityOperation::ExternalBoundaryExecute { boundary_id } => capability
            .scopes
            .contains(&CapabilityScope::ExternalBoundaryExecute {
                boundary_id: boundary_id.clone(),
            }),
    }
}
