//! Shared RD-659/RD-663 ProjectExecutionSnapshotIdentity issuer and verifier.
//!
//! This module owns canonical bytes. CLI, WASM and Workbench adapters may
//! transport its values, but must not reproduce or repair this encoding.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;

// RD-689 is the current forward physical owner. Legacy functions below are
// preserved predecessor code, not authority for current constituent bytes.
pub(crate) mod current;

const VERSION: u16 = 1;
const SOURCE_DOMAIN: &[u8] = b"DDN-SOURCE-LOGICAL-TREE-CONTENT-IDENTITY-V1\0";
const CONFIG_DOMAIN: &[u8] = b"DDN-PROJECT-SEMANTIC-CONFIGURATION-IDENTITY-V1\0";
const DEPENDENCY_DOMAIN: &[u8] = b"DDN-VERIFIED-DEPENDENCY-RESOLUTION-IDENTITY-V1\0";
const RESOURCE_DOMAIN: &[u8] = b"DDN-DECLARED-RESOURCE-CONTENT-CLOSURE-IDENTITY-V1\0";
const PESI_DOMAIN: &[u8] = b"DDN-PROJECT-EXECUTION-SNAPSHOT-IDENTITY-V1\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity32([u8; 32]);

impl Identity32 {
    pub fn from_raw(raw: [u8; 32]) -> Self {
        Self(raw)
    }

    pub fn as_raw(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        hex::encode(self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssuedIdentity {
    pub canonical_payload: Vec<u8>,
    pub envelope: Vec<u8>,
    pub identity: Identity32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotIdentityError {
    ConstituentMissing(&'static str),
    ConstituentMalformed(&'static str),
    ConstituentDuplicate(&'static str),
    ConstituentStale(&'static str),
    ConstituentMismatch(&'static str),
    ResourceDeclarationMissing(String),
    ResourceLogicalPathConflict(String),
    ResourceRootEscape(String),
    ResourceContentMismatch(String),
    DependencyResolutionUnverified,
    DependencyResolutionMismatch(&'static str),
}

impl std::fmt::Display for SnapshotIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConstituentMissing(name) => {
                write!(f, "PROJECT_SNAPSHOT_CONSTITUENT_MISSING: {name}")
            }
            Self::ConstituentMalformed(name) => {
                write!(f, "PROJECT_SNAPSHOT_CONSTITUENT_MALFORMED: {name}")
            }
            Self::ConstituentDuplicate(name) => {
                write!(f, "PROJECT_SNAPSHOT_CONSTITUENT_DUPLICATE: {name}")
            }
            Self::ConstituentStale(name) => {
                write!(f, "PROJECT_SNAPSHOT_CONSTITUENT_STALE: {name}")
            }
            Self::ConstituentMismatch(name) => {
                write!(f, "PROJECT_SNAPSHOT_CONSTITUENT_MISMATCH: {name}")
            }
            Self::ResourceDeclarationMissing(path) => {
                write!(f, "PROJECT_RESOURCE_DECLARATION_MISSING: {path}")
            }
            Self::ResourceLogicalPathConflict(path) => {
                write!(f, "PROJECT_RESOURCE_LOGICAL_PATH_CONFLICT: {path}")
            }
            Self::ResourceRootEscape(path) => {
                write!(f, "PROJECT_RESOURCE_ROOT_ESCAPE: {path}")
            }
            Self::ResourceContentMismatch(path) => {
                write!(f, "PROJECT_RESOURCE_CONTENT_MISMATCH: {path}")
            }
            Self::DependencyResolutionUnverified => {
                f.write_str("PROJECT_DEPENDENCY_RESOLUTION_UNVERIFIED")
            }
            Self::DependencyResolutionMismatch(reason) => {
                write!(f, "PROJECT_DEPENDENCY_RESOLUTION_MISMATCH: {reason}")
            }
        }
    }
}

impl std::error::Error for SnapshotIdentityError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLogicalTreeEntry {
    pub logical_path: String,
    pub source_snapshot_identity: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalTextMapEntry {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSemanticConfigurationInput {
    pub source_epoch_identity: Vec<u8>,
    pub official_entry_logical_path: String,
    pub target_map: Vec<CanonicalTextMapEntry>,
    pub module_path_map: Vec<CanonicalTextMapEntry>,
    pub dependency_requirements: Vec<String>,
    pub declared_resource_membership: Vec<String>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyNodeKind {
    DdnGaji = 0x01,
    ExternalArtifact = 0x02,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedDependencyNode {
    pub kind: DependencyNodeKind,
    pub node_identity: Vec<u8>,
    pub content_identity: Vec<u8>,
    pub dependencies: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedDependencyResolutionInput {
    pub graph_verified: bool,
    pub nodes: Vec<VerifiedDependencyNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredResourceContent {
    pub logical_path: String,
    pub content: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectExecutionSnapshotConstituents {
    pub source_logical_tree: Identity32,
    pub semantic_configuration: Identity32,
    pub dependency_resolution: Identity32,
    pub resource_content_closure: Identity32,
}

fn push_u32_len(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), SnapshotIdentityError> {
    let length = u32::try_from(bytes.len())
        .map_err(|_| SnapshotIdentityError::ConstituentMalformed("length_u32be"))?;
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

fn issue_enveloped(
    domain: &[u8],
    payload: Vec<u8>,
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    let payload_length = u32::try_from(payload.len())
        .map_err(|_| SnapshotIdentityError::ConstituentMalformed("payload_length_u32be"))?;
    let mut envelope = Vec::with_capacity(domain.len() + 6 + payload.len());
    envelope.extend_from_slice(domain);
    envelope.extend_from_slice(&VERSION.to_be_bytes());
    envelope.extend_from_slice(&payload_length.to_be_bytes());
    envelope.extend_from_slice(&payload);
    let identity = Identity32::from_raw(Sha256::digest(&envelope).into());
    Ok(IssuedIdentity {
        canonical_payload: payload,
        envelope,
        identity,
    })
}

fn verify_identity(
    issued: IssuedIdentity,
    expected: Identity32,
    owner: &'static str,
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    if issued.identity != expected {
        return Err(SnapshotIdentityError::ConstituentMismatch(owner));
    }
    Ok(issued)
}

fn validate_nfc_text(value: &str, owner: &'static str) -> Result<(), SnapshotIdentityError> {
    if value.is_empty() || value.nfc().ne(value.chars()) {
        return Err(SnapshotIdentityError::ConstituentMalformed(owner));
    }
    Ok(())
}

fn ascii_fold(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .iter()
        .map(|byte| byte.to_ascii_lowercase())
        .collect()
}

fn validate_logical_path(path: &str) -> Result<(), SnapshotIdentityError> {
    validate_nfc_text(path, "logical_path")?;
    if path.contains('\\')
        || path.contains('\0')
        || path.starts_with('/')
        || path.contains("://")
        || path.as_bytes().get(1) == Some(&b':')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(SnapshotIdentityError::ResourceRootEscape(path.to_string()));
    }
    Ok(())
}

fn canonical_paths(paths: &[String]) -> Result<Vec<String>, SnapshotIdentityError> {
    let mut rows = paths.to_vec();
    for path in &rows {
        validate_logical_path(path)?;
    }
    rows.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    let mut exact = BTreeSet::new();
    let mut folded = BTreeSet::new();
    for path in &rows {
        if !exact.insert(path.as_bytes().to_vec()) || !folded.insert(ascii_fold(path)) {
            return Err(SnapshotIdentityError::ResourceLogicalPathConflict(
                path.clone(),
            ));
        }
    }
    Ok(rows)
}

pub fn issue_source_logical_tree_content_identity_v1(
    entries: &[SourceLogicalTreeEntry],
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    let mut rows = entries.to_vec();
    for row in &rows {
        validate_logical_path(&row.logical_path)?;
        if row.source_snapshot_identity.is_empty() {
            return Err(SnapshotIdentityError::ConstituentMissing(
                "SourceSnapshotIdentity",
            ));
        }
    }
    rows.sort_by(|left, right| {
        left.logical_path
            .as_bytes()
            .cmp(right.logical_path.as_bytes())
    });
    let paths = rows
        .iter()
        .map(|row| row.logical_path.clone())
        .collect::<Vec<_>>();
    canonical_paths(&paths)?;
    let count = u32::try_from(rows.len())
        .map_err(|_| SnapshotIdentityError::ConstituentMalformed("source_entry_count"))?;
    let mut payload = Vec::new();
    payload.extend_from_slice(&count.to_be_bytes());
    for row in rows {
        push_u32_len(&mut payload, row.logical_path.as_bytes())?;
        push_u32_len(&mut payload, &row.source_snapshot_identity)?;
    }
    issue_enveloped(SOURCE_DOMAIN, payload)
}

pub fn verify_source_logical_tree_content_identity_v1(
    entries: &[SourceLogicalTreeEntry],
    expected: Identity32,
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    verify_identity(
        issue_source_logical_tree_content_identity_v1(entries)?,
        expected,
        "source_logical_tree_content_identity",
    )
}

fn encode_text_map(
    entries: &[CanonicalTextMapEntry],
    owner: &'static str,
) -> Result<Vec<u8>, SnapshotIdentityError> {
    let mut rows = entries.to_vec();
    for row in &rows {
        validate_nfc_text(&row.key, owner)?;
        validate_nfc_text(&row.value, owner)?;
    }
    rows.sort_by(|left, right| left.key.as_bytes().cmp(right.key.as_bytes()));
    let mut previous: Option<&str> = None;
    let mut out = Vec::new();
    out.extend_from_slice(
        &u32::try_from(rows.len())
            .map_err(|_| SnapshotIdentityError::ConstituentMalformed(owner))?
            .to_be_bytes(),
    );
    for row in &rows {
        if previous == Some(row.key.as_str()) {
            return Err(SnapshotIdentityError::ConstituentDuplicate(owner));
        }
        previous = Some(&row.key);
        push_u32_len(&mut out, row.key.as_bytes())?;
        push_u32_len(&mut out, row.value.as_bytes())?;
    }
    Ok(out)
}

fn encode_text_set(
    values: &[String],
    owner: &'static str,
    paths: bool,
) -> Result<Vec<u8>, SnapshotIdentityError> {
    let mut rows = if paths {
        canonical_paths(values)?
    } else {
        let mut rows = values.to_vec();
        for value in &rows {
            validate_nfc_text(value, owner)?;
        }
        rows.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        if rows.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(SnapshotIdentityError::ConstituentDuplicate(owner));
        }
        rows
    };
    let mut out = Vec::new();
    out.extend_from_slice(
        &u32::try_from(rows.len())
            .map_err(|_| SnapshotIdentityError::ConstituentMalformed(owner))?
            .to_be_bytes(),
    );
    for row in rows.drain(..) {
        push_u32_len(&mut out, row.as_bytes())?;
    }
    Ok(out)
}

fn push_config_field(
    payload: &mut Vec<u8>,
    tag: u8,
    bytes: &[u8],
) -> Result<(), SnapshotIdentityError> {
    payload.push(tag);
    push_u32_len(payload, bytes)
}

pub fn issue_project_semantic_configuration_identity_v1(
    input: &ProjectSemanticConfigurationInput,
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    if input.source_epoch_identity.is_empty() {
        return Err(SnapshotIdentityError::ConstituentMissing(
            "SourceEpochIdentity",
        ));
    }
    validate_logical_path(&input.official_entry_logical_path)?;
    let target_map = encode_text_map(&input.target_map, "target_map")?;
    let module_path_map = encode_text_map(&input.module_path_map, "module_path_map")?;
    let dependency_requirements = encode_text_set(
        &input.dependency_requirements,
        "dependency_requirements",
        false,
    )?;
    let declared_resources = encode_text_set(
        &input.declared_resource_membership,
        "declared_resource_membership",
        true,
    )?;
    let mut payload = Vec::new();
    push_config_field(&mut payload, 0x01, &input.source_epoch_identity)?;
    push_config_field(
        &mut payload,
        0x02,
        input.official_entry_logical_path.as_bytes(),
    )?;
    push_config_field(&mut payload, 0x03, &target_map)?;
    push_config_field(&mut payload, 0x04, &module_path_map)?;
    push_config_field(&mut payload, 0x05, &dependency_requirements)?;
    push_config_field(&mut payload, 0x06, &declared_resources)?;
    issue_enveloped(CONFIG_DOMAIN, payload)
}

pub fn verify_project_semantic_configuration_identity_v1(
    input: &ProjectSemanticConfigurationInput,
    expected: Identity32,
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    verify_identity(
        issue_project_semantic_configuration_identity_v1(input)?,
        expected,
        "ProjectSemanticConfigurationIdentity",
    )
}

fn dependency_cycle_visit(
    index: usize,
    nodes: &[VerifiedDependencyNode],
    positions: &BTreeMap<Vec<u8>, usize>,
    state: &mut [u8],
) -> Result<(), SnapshotIdentityError> {
    if state[index] == 1 {
        return Err(SnapshotIdentityError::DependencyResolutionMismatch(
            "dependency_cycle",
        ));
    }
    if state[index] == 2 {
        return Ok(());
    }
    state[index] = 1;
    for dependency in &nodes[index].dependencies {
        let next = positions.get(dependency).ok_or(
            SnapshotIdentityError::DependencyResolutionMismatch("missing_referenced_node"),
        )?;
        dependency_cycle_visit(*next, nodes, positions, state)?;
    }
    state[index] = 2;
    Ok(())
}

pub fn issue_verified_dependency_resolution_identity_v1(
    input: &VerifiedDependencyResolutionInput,
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    if !input.graph_verified {
        return Err(SnapshotIdentityError::DependencyResolutionUnverified);
    }
    let mut nodes = input.nodes.clone();
    for node in &mut nodes {
        if node.node_identity.is_empty() || node.content_identity.is_empty() {
            return Err(SnapshotIdentityError::DependencyResolutionMismatch(
                "empty_node_or_content_identity",
            ));
        }
        node.dependencies.sort();
        if node.dependencies.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(SnapshotIdentityError::ConstituentDuplicate(
                "dependency_identity",
            ));
        }
    }
    nodes.sort_by(|left, right| left.node_identity.cmp(&right.node_identity));
    if nodes
        .windows(2)
        .any(|pair| pair[0].node_identity == pair[1].node_identity)
    {
        return Err(SnapshotIdentityError::ConstituentDuplicate(
            "dependency_node_identity",
        ));
    }
    let positions = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.node_identity.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let mut state = vec![0; nodes.len()];
    for index in 0..nodes.len() {
        dependency_cycle_visit(index, &nodes, &positions, &mut state)?;
    }
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(nodes.len())
            .map_err(|_| SnapshotIdentityError::DependencyResolutionMismatch("node_count"))?
            .to_be_bytes(),
    );
    for node in nodes {
        payload.push(node.kind as u8);
        push_u32_len(&mut payload, &node.node_identity)?;
        push_u32_len(&mut payload, &node.content_identity)?;
        payload.extend_from_slice(
            &u32::try_from(node.dependencies.len())
                .map_err(|_| {
                    SnapshotIdentityError::DependencyResolutionMismatch("dependency_count")
                })?
                .to_be_bytes(),
        );
        for dependency in node.dependencies {
            push_u32_len(&mut payload, &dependency)?;
        }
    }
    issue_enveloped(DEPENDENCY_DOMAIN, payload)
}

pub fn verify_verified_dependency_resolution_identity_v1(
    input: &VerifiedDependencyResolutionInput,
    expected: Identity32,
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    verify_identity(
        issue_verified_dependency_resolution_identity_v1(input)?,
        expected,
        "verified_dependency_resolution_identity",
    )
}

pub fn issue_declared_resource_content_closure_identity_v1(
    declared_membership: &[String],
    resources: &[DeclaredResourceContent],
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    let declared = canonical_paths(declared_membership)?;
    let mut contents = resources.to_vec();
    let content_paths = contents
        .iter()
        .map(|resource| resource.logical_path.clone())
        .collect::<Vec<_>>();
    canonical_paths(&content_paths)?;
    contents.sort_by(|left, right| {
        left.logical_path
            .as_bytes()
            .cmp(right.logical_path.as_bytes())
    });
    let declared_set = declared.iter().cloned().collect::<BTreeSet<_>>();
    let content_set = contents
        .iter()
        .map(|resource| resource.logical_path.clone())
        .collect::<BTreeSet<_>>();
    if let Some(path) = declared_set.difference(&content_set).next() {
        return Err(SnapshotIdentityError::ResourceDeclarationMissing(
            path.clone(),
        ));
    }
    if let Some(path) = content_set.difference(&declared_set).next() {
        return Err(SnapshotIdentityError::ResourceContentMismatch(path.clone()));
    }
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(contents.len())
            .map_err(|_| SnapshotIdentityError::ConstituentMalformed("resource_count"))?
            .to_be_bytes(),
    );
    for resource in contents {
        push_u32_len(&mut payload, resource.logical_path.as_bytes())?;
        payload.extend_from_slice(
            &u64::try_from(resource.content.len())
                .map_err(|_| {
                    SnapshotIdentityError::ResourceContentMismatch(resource.logical_path.clone())
                })?
                .to_be_bytes(),
        );
        payload.extend_from_slice(&Sha256::digest(&resource.content));
    }
    issue_enveloped(RESOURCE_DOMAIN, payload)
}

pub fn verify_declared_resource_content_closure_identity_v1(
    declared_membership: &[String],
    resources: &[DeclaredResourceContent],
    expected: Identity32,
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    verify_identity(
        issue_declared_resource_content_closure_identity_v1(declared_membership, resources)?,
        expected,
        "declared_resource_content_closure_identity",
    )
}

pub fn issue_project_execution_snapshot_identity_v1(
    constituents: ProjectExecutionSnapshotConstituents,
) -> IssuedIdentity {
    let mut preimage = Vec::with_capacity(PESI_DOMAIN.len() + 3 + 4 * 37);
    preimage.extend_from_slice(PESI_DOMAIN);
    preimage.extend_from_slice(&VERSION.to_be_bytes());
    preimage.push(0x04);
    for (tag, identity) in [
        (0x01, constituents.source_logical_tree),
        (0x02, constituents.semantic_configuration),
        (0x03, constituents.dependency_resolution),
        (0x04, constituents.resource_content_closure),
    ] {
        preimage.push(tag);
        preimage.extend_from_slice(&32u32.to_be_bytes());
        preimage.extend_from_slice(identity.as_raw());
    }
    IssuedIdentity {
        canonical_payload: preimage[PESI_DOMAIN.len() + 3..].to_vec(),
        identity: Identity32::from_raw(Sha256::digest(&preimage).into()),
        envelope: preimage,
    }
}

pub fn verify_project_execution_snapshot_identity_v1(
    constituents: ProjectExecutionSnapshotConstituents,
    expected: Identity32,
) -> Result<IssuedIdentity, SnapshotIdentityError> {
    verify_identity(
        issue_project_execution_snapshot_identity_v1(constituents),
        expected,
        "ProjectExecutionSnapshotIdentity",
    )
}

pub fn verify_project_execution_snapshot_preimage_v1(
    preimage: &[u8],
    expected: Identity32,
) -> Result<ProjectExecutionSnapshotConstituents, SnapshotIdentityError> {
    let prefix = PESI_DOMAIN.len();
    if preimage.get(..prefix) != Some(PESI_DOMAIN)
        || preimage.get(prefix..prefix + 2) != Some(VERSION.to_be_bytes().as_slice())
    {
        return Err(SnapshotIdentityError::ConstituentMalformed("PESI_header"));
    }
    if preimage.get(prefix + 2) != Some(&0x04) {
        return Err(SnapshotIdentityError::ConstituentMissing(
            "PESI_field_count",
        ));
    }
    let mut cursor = prefix + 3;
    let mut values = [[0u8; 32]; 4];
    for tag in 1u8..=4 {
        if preimage.get(cursor) != Some(&tag) {
            return Err(
                if preimage
                    .get(cursor)
                    .copied()
                    .is_some_and(|actual| actual < tag)
                {
                    SnapshotIdentityError::ConstituentDuplicate("PESI_tag")
                } else {
                    SnapshotIdentityError::ConstituentMalformed("PESI_tag_order")
                },
            );
        }
        cursor += 1;
        let length: [u8; 4] = preimage
            .get(cursor..cursor + 4)
            .ok_or(SnapshotIdentityError::ConstituentMalformed("PESI_length"))?
            .try_into()
            .map_err(|_| SnapshotIdentityError::ConstituentMalformed("PESI_length"))?;
        cursor += 4;
        if u32::from_be_bytes(length) != 32 {
            return Err(SnapshotIdentityError::ConstituentMalformed(
                "PESI_identity_length",
            ));
        }
        values[(tag - 1) as usize] = preimage
            .get(cursor..cursor + 32)
            .ok_or(SnapshotIdentityError::ConstituentMissing("PESI_identity"))?
            .try_into()
            .map_err(|_| SnapshotIdentityError::ConstituentMalformed("PESI_identity"))?;
        cursor += 32;
    }
    if cursor != preimage.len() {
        return Err(SnapshotIdentityError::ConstituentMalformed("PESI_suffix"));
    }
    let actual = Identity32::from_raw(Sha256::digest(preimage).into());
    if actual != expected {
        return Err(SnapshotIdentityError::ConstituentMismatch(
            "ProjectExecutionSnapshotIdentity",
        ));
    }
    Ok(ProjectExecutionSnapshotConstituents {
        source_logical_tree: Identity32::from_raw(values[0]),
        semantic_configuration: Identity32::from_raw(values[1]),
        dependency_resolution: Identity32::from_raw(values[2]),
        resource_content_closure: Identity32::from_raw(values[3]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(byte: u8) -> Identity32 {
        Identity32::from_raw([byte; 32])
    }

    fn baseline() -> ProjectExecutionSnapshotConstituents {
        ProjectExecutionSnapshotConstituents {
            source_logical_tree: identity(0x11),
            semantic_configuration: identity(0x22),
            dependency_resolution: identity(0x33),
            resource_content_closure: identity(0x44),
        }
    }

    #[test]
    fn exact_pesi_01_through_06_vectors_match() {
        let cases = [
            (
                baseline(),
                "7edaba625c27e30f633fdc15ea27222c72a3e6112b6f0819335d1a4244b45fb2",
            ),
            (
                ProjectExecutionSnapshotConstituents {
                    source_logical_tree: identity(0x12),
                    ..baseline()
                },
                "b0c13bc2b0fc8e4c2c6058413ca468acb100f838ed7cd2d631fc1838f97652e1",
            ),
            (
                ProjectExecutionSnapshotConstituents {
                    semantic_configuration: identity(0x23),
                    ..baseline()
                },
                "2593c09bba82edc76b36e0ca401dc15bccc79d13bd4798f75da77e128cd16375",
            ),
            (
                ProjectExecutionSnapshotConstituents {
                    dependency_resolution: identity(0x34),
                    ..baseline()
                },
                "1330ec2960a169f4b6e73e62917022ea90dfab67d0e0a888ebad1da4202244fa",
            ),
            (
                ProjectExecutionSnapshotConstituents {
                    resource_content_closure: identity(0x45),
                    ..baseline()
                },
                "aaf3ec3f97405031018d44a3716c5d6900f2ad826d736fc71b9b2521e157aa9d",
            ),
        ];
        for (constituents, expected) in cases {
            let issued = issue_project_execution_snapshot_identity_v1(constituents);
            assert_eq!(issued.envelope.len(), 194);
            assert_eq!(issued.identity.to_hex(), expected);
            assert_eq!(
                verify_project_execution_snapshot_preimage_v1(&issued.envelope, issued.identity),
                Ok(constituents)
            );
        }
    }

    #[test]
    fn exact_pesi_08_and_09_malformed_vectors_fail_closed() {
        let issued = issue_project_execution_snapshot_identity_v1(baseline());
        let mut missing = issued.envelope.clone();
        let field_size = 37;
        missing[PESI_DOMAIN.len() + 2] = 3;
        missing.truncate(missing.len() - field_size);
        assert!(matches!(
            verify_project_execution_snapshot_preimage_v1(&missing, issued.identity),
            Err(SnapshotIdentityError::ConstituentMissing(
                "PESI_field_count"
            ))
        ));
        let mut duplicate = issued.envelope.clone();
        let fourth_tag = PESI_DOMAIN.len() + 3 + field_size * 3;
        duplicate[fourth_tag] = 0x03;
        assert!(matches!(
            verify_project_execution_snapshot_preimage_v1(&duplicate, issued.identity),
            Err(SnapshotIdentityError::ConstituentDuplicate("PESI_tag"))
        ));
    }

    #[test]
    fn source_identity_is_order_independent_and_rejects_path_collisions() {
        let a = SourceLogicalTreeEntry {
            logical_path: "src/a.ddn".into(),
            source_snapshot_identity: vec![1; 32],
        };
        let b = SourceLogicalTreeEntry {
            logical_path: "src/b.ddn".into(),
            source_snapshot_identity: vec![2; 32],
        };
        assert_eq!(
            issue_source_logical_tree_content_identity_v1(&[a.clone(), b.clone()])
                .unwrap()
                .identity,
            issue_source_logical_tree_content_identity_v1(&[b, a.clone()])
                .unwrap()
                .identity
        );
        let folded = SourceLogicalTreeEntry {
            logical_path: "SRC/A.ddn".into(),
            source_snapshot_identity: vec![3; 32],
        };
        assert!(matches!(
            issue_source_logical_tree_content_identity_v1(&[a, folded]),
            Err(SnapshotIdentityError::ResourceLogicalPathConflict(_))
        ));
    }

    #[test]
    fn dependency_empty_graph_is_not_null_and_unverified_or_cyclic_fails() {
        let empty =
            issue_verified_dependency_resolution_identity_v1(&VerifiedDependencyResolutionInput {
                graph_verified: true,
                nodes: vec![],
            })
            .unwrap();
        assert_eq!(empty.canonical_payload, 0u32.to_be_bytes());
        assert_ne!(empty.identity.as_raw(), &[0; 32]);
        assert_eq!(
            issue_verified_dependency_resolution_identity_v1(&VerifiedDependencyResolutionInput {
                graph_verified: false,
                nodes: vec![],
            }),
            Err(SnapshotIdentityError::DependencyResolutionUnverified)
        );
        let node = VerifiedDependencyNode {
            kind: DependencyNodeKind::DdnGaji,
            node_identity: vec![1],
            content_identity: vec![2],
            dependencies: vec![vec![1]],
        };
        assert!(matches!(
            issue_verified_dependency_resolution_identity_v1(&VerifiedDependencyResolutionInput {
                graph_verified: true,
                nodes: vec![node],
            }),
            Err(SnapshotIdentityError::DependencyResolutionMismatch(
                "dependency_cycle"
            ))
        ));
    }

    #[test]
    fn resource_identity_requires_exact_declared_membership_and_content() {
        let declared = vec!["assets/a.bin".to_string(), "assets/b.bin".to_string()];
        let resources = vec![
            DeclaredResourceContent {
                logical_path: "assets/b.bin".into(),
                content: vec![0xff],
            },
            DeclaredResourceContent {
                logical_path: "assets/a.bin".into(),
                content: vec![0x00],
            },
        ];
        let left =
            issue_declared_resource_content_closure_identity_v1(&declared, &resources).unwrap();
        let right = issue_declared_resource_content_closure_identity_v1(
            &[declared[1].clone(), declared[0].clone()],
            &[resources[1].clone(), resources[0].clone()],
        )
        .unwrap();
        assert_eq!(left.identity, right.identity);
        assert!(matches!(
            issue_declared_resource_content_closure_identity_v1(&declared, &resources[..1]),
            Err(SnapshotIdentityError::ResourceDeclarationMissing(_))
        ));
    }

    #[test]
    fn configuration_is_six_fields_and_rejects_hidden_normalization() {
        let input = ProjectSemanticConfigurationInput {
            source_epoch_identity: vec![7; 32],
            official_entry_logical_path: "src/main.ddn".into(),
            target_map: vec![CanonicalTextMapEntry {
                key: "default".into(),
                value: "main".into(),
            }],
            module_path_map: vec![],
            dependency_requirements: vec![],
            declared_resource_membership: vec![],
        };
        let issued = issue_project_semantic_configuration_identity_v1(&input).unwrap();
        assert_eq!(issued.canonical_payload[0], 0x01);
        assert!(issued.canonical_payload.contains(&0x06));
        let mut noncanonical = input;
        noncanonical.target_map[0].value = "가".into();
        assert!(matches!(
            issue_project_semantic_configuration_identity_v1(&noncanonical),
            Err(SnapshotIdentityError::ConstituentMalformed("target_map"))
        ));
    }
}
