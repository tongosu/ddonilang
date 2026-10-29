use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::net::IpAddr;

pub const DOMAIN_EVIDENCE_MANIFEST_SCHEMA: &str = "ddn.domain_evidence_manifest.v1";
pub const DOMAIN_EVIDENCE_INSPECTION_SCHEMA: &str = "ddn.domain_evidence_inspection.v1";
pub const DOMAIN_EVIDENCE_BINDING_PREFIX: &[u8] = b"ddn.domain-evidence.binding.v1\0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvidenceManifestError {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

impl EvidenceManifestError {
    fn new(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for EvidenceManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}: {}", self.code, self.path, self.message)
    }
}

impl std::error::Error for EvidenceManifestError {}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainEvidenceManifest {
    pub schema: String,
    pub subject: EvidenceSubject,
    pub sources: Vec<EvidenceSource>,
    pub claims: Vec<EvidenceClaim>,
    pub activities: Vec<EvidenceActivity>,
    pub hashes: EvidenceManifestHashes,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSubject {
    pub dataset_content_sha256: String,
    pub project_revision_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSource {
    pub source_ref: String,
    pub title: String,
    pub provider: String,
    pub canonical_identifier: String,
    pub published_or_version: String,
    pub retrieved_at: String,
    pub rights: EvidenceRights,
    pub payload_mode: EvidencePayloadMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRights {
    pub license_id: String,
    pub holder: String,
    pub reuse_mode: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidencePayloadMode {
    Reference,
    Embedded,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceClaim {
    pub target: EvidenceTarget,
    pub relation: EvidenceRelation,
    pub source_ref: String,
    pub source_locator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceTarget {
    pub dataset_content_sha256: String,
    pub row_source_ordinal: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    DirectSource,
    DerivedFrom,
    QuotedFrom,
    PrimarySource,
    Context,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceActivity {
    pub kind: EvidenceActivityKind,
    pub input_source_refs: Vec<String>,
    pub output_targets: Vec<EvidenceTarget>,
    pub recipe_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceActivityKind {
    Extract,
    Normalize,
    Calculate,
    Classify,
    Summarize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceManifestHashes {
    pub evidence_manifest_sha256: String,
    pub content_binding_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvidenceManifestIdentity {
    pub schema: &'static str,
    pub source_ref: String,
    pub dataset_content_sha256: String,
    pub evidence_manifest_sha256: String,
    pub content_binding_sha256: String,
    pub inspection: EvidenceInspectionProjection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvidenceInspectionProjection {
    pub schema: &'static str,
    pub sources: Vec<EvidenceSource>,
    pub claims: Vec<EvidenceClaim>,
    pub activities: Vec<EvidenceActivity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceDatasetTarget {
    pub dataset_content_sha256: String,
    pub project_revision_ref: Option<String>,
    pub column_names: Vec<String>,
    pub rows: Vec<EvidenceDatasetRowTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceDatasetRowTarget {
    pub source_ordinal: u64,
    pub cell_statuses: Vec<EvidenceCellStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceCellStatus {
    Present,
    Gap,
    Missing { reason: String },
    Pruned,
}

pub fn parse_and_validate_domain_evidence_manifest(
    input_json: &str,
    datasets: &[EvidenceDatasetTarget],
) -> Result<DomainEvidenceManifest, EvidenceManifestError> {
    let raw: JsonValue = serde_json::from_str(input_json).map_err(|err| {
        EvidenceManifestError::new(
            "E_EVIDENCE_MANIFEST_JSON",
            "$",
            format!("evidence manifest JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    scan_forbidden_material(&raw, "$")?;
    let manifest: DomainEvidenceManifest = serde_json::from_str(input_json).map_err(|err| {
        EvidenceManifestError::new(
            "E_EVIDENCE_MANIFEST_JSON",
            "$",
            format!("evidence manifest가 strict contract와 다릅니다: {err}"),
        )
    })?;
    validate_domain_evidence_manifest(&manifest, datasets)?;
    Ok(manifest)
}

pub fn validate_domain_evidence_manifest(
    manifest: &DomainEvidenceManifest,
    datasets: &[EvidenceDatasetTarget],
) -> Result<(), EvidenceManifestError> {
    validate_domain_evidence_manifest_internal(manifest, datasets, true)
}

/// 저작 중인 source record를 기존 manifest source contract로 검증한다.
/// 별도 source artifact schema를 만들지 않으며, 최종 봉인도 같은 validator를
/// 다시 통과한다.
pub fn validate_evidence_sources(sources: &[EvidenceSource]) -> Result<(), EvidenceManifestError> {
    let raw = serde_json::to_value(sources).map_err(|err| {
        EvidenceManifestError::new(
            "E_EVIDENCE_MANIFEST_JSON",
            "$.sources",
            format!("evidence source를 검증용 JSON으로 만들 수 없습니다: {err}"),
        )
    })?;
    scan_forbidden_material(&raw, "$.sources")?;
    validate_evidence_sources_internal(sources).map(|_| ())
}

pub fn seal_domain_evidence_manifest(
    mut manifest: DomainEvidenceManifest,
    datasets: &[EvidenceDatasetTarget],
) -> Result<DomainEvidenceManifest, EvidenceManifestError> {
    manifest.hashes.evidence_manifest_sha256 = "0".repeat(64);
    manifest.hashes.content_binding_sha256 = "0".repeat(64);
    validate_domain_evidence_manifest_internal(&manifest, datasets, false)?;
    manifest.hashes.evidence_manifest_sha256 = evidence_manifest_sha256(&manifest)?;
    manifest.hashes.content_binding_sha256 = evidence_content_binding_sha256(
        &manifest.subject.dataset_content_sha256,
        &manifest.hashes.evidence_manifest_sha256,
    )?;
    validate_domain_evidence_manifest(&manifest, datasets)?;
    Ok(manifest)
}

pub fn evidence_manifest_sha256(
    manifest: &DomainEvidenceManifest,
) -> Result<String, EvidenceManifestError> {
    let mut value = serde_json::to_value(manifest).map_err(|err| {
        EvidenceManifestError::new(
            "E_EVIDENCE_MANIFEST_SERIALIZE",
            "$",
            format!("evidence manifest hash 입력을 직렬화할 수 없습니다: {err}"),
        )
    })?;
    value
        .as_object_mut()
        .ok_or_else(|| {
            EvidenceManifestError::new(
                "E_EVIDENCE_MANIFEST_SERIALIZE",
                "$",
                "evidence manifest hash 입력이 object가 아닙니다",
            )
        })?
        .remove("hashes");
    Ok(sha256_hex(&canonical_json_bytes(&value)))
}

pub fn evidence_content_binding_sha256(
    dataset_content_sha256: &str,
    evidence_manifest_sha256: &str,
) -> Result<String, EvidenceManifestError> {
    validate_sha256_text(dataset_content_sha256, "$.subject.dataset_content_sha256")?;
    validate_sha256_text(
        evidence_manifest_sha256,
        "$.hashes.evidence_manifest_sha256",
    )?;
    let mut bytes = Vec::with_capacity(DOMAIN_EVIDENCE_BINDING_PREFIX.len() + 128);
    bytes.extend_from_slice(DOMAIN_EVIDENCE_BINDING_PREFIX);
    bytes.extend_from_slice(dataset_content_sha256.as_bytes());
    bytes.extend_from_slice(evidence_manifest_sha256.as_bytes());
    Ok(sha256_hex(&bytes))
}

pub fn canonical_domain_evidence_manifest_json(
    manifest: &DomainEvidenceManifest,
    datasets: &[EvidenceDatasetTarget],
) -> Result<String, EvidenceManifestError> {
    validate_domain_evidence_manifest(manifest, datasets)?;
    let value = serde_json::to_value(manifest).map_err(|err| {
        EvidenceManifestError::new(
            "E_EVIDENCE_MANIFEST_SERIALIZE",
            "$",
            format!("evidence manifest JSON 직렬화 실패: {err}"),
        )
    })?;
    String::from_utf8(canonical_json_bytes(&value)).map_err(|err| {
        EvidenceManifestError::new(
            "E_EVIDENCE_MANIFEST_SERIALIZE",
            "$",
            format!("evidence manifest canonical UTF-8 생성 실패: {err}"),
        )
    })
}

pub fn evidence_manifest_identity(
    source_ref: &str,
    manifest: &DomainEvidenceManifest,
    datasets: &[EvidenceDatasetTarget],
) -> Result<EvidenceManifestIdentity, EvidenceManifestError> {
    require_text(source_ref, "$context.source_ref")?;
    validate_domain_evidence_manifest(manifest, datasets)?;
    Ok(EvidenceManifestIdentity {
        schema: DOMAIN_EVIDENCE_MANIFEST_SCHEMA,
        source_ref: source_ref.to_string(),
        dataset_content_sha256: manifest.subject.dataset_content_sha256.clone(),
        evidence_manifest_sha256: manifest.hashes.evidence_manifest_sha256.clone(),
        content_binding_sha256: manifest.hashes.content_binding_sha256.clone(),
        inspection: EvidenceInspectionProjection {
            schema: DOMAIN_EVIDENCE_INSPECTION_SCHEMA,
            sources: manifest.sources.clone(),
            claims: manifest.claims.clone(),
            activities: manifest.activities.clone(),
        },
    })
}

fn validate_domain_evidence_manifest_internal(
    manifest: &DomainEvidenceManifest,
    datasets: &[EvidenceDatasetTarget],
    verify_hashes: bool,
) -> Result<(), EvidenceManifestError> {
    let raw = serde_json::to_value(manifest).map_err(|err| {
        EvidenceManifestError::new(
            "E_EVIDENCE_MANIFEST_SERIALIZE",
            "$",
            format!("evidence manifest 검증 값을 직렬화할 수 없습니다: {err}"),
        )
    })?;
    scan_forbidden_material(&raw, "$")?;

    if manifest.schema != DOMAIN_EVIDENCE_MANIFEST_SCHEMA {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_MANIFEST_SCHEMA",
            "$.schema",
            format!("schema는 {DOMAIN_EVIDENCE_MANIFEST_SCHEMA}여야 합니다"),
        ));
    }
    validate_sha256_text(
        &manifest.subject.dataset_content_sha256,
        "$.subject.dataset_content_sha256",
    )?;
    require_text(
        &manifest.subject.project_revision_ref,
        "$.subject.project_revision_ref",
    )?;
    validate_sha256_text(
        &manifest.hashes.evidence_manifest_sha256,
        "$.hashes.evidence_manifest_sha256",
    )?;
    validate_sha256_text(
        &manifest.hashes.content_binding_sha256,
        "$.hashes.content_binding_sha256",
    )?;

    let dataset_by_hash = build_dataset_index(datasets)?;
    let subject_dataset = dataset_by_hash
        .get(manifest.subject.dataset_content_sha256.as_str())
        .ok_or_else(|| {
            EvidenceManifestError::new(
                "E_EVIDENCE_DATASET_NOT_FOUND",
                "$.subject.dataset_content_sha256",
                "subject dataset content hash와 일치하는 검증 대상이 없습니다",
            )
        })?;
    if let Some(dataset_revision_ref) = &subject_dataset.project_revision_ref {
        if dataset_revision_ref != &manifest.subject.project_revision_ref {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_CROSS_RUN_SUBJECT",
                "$.subject.project_revision_ref",
                format!(
                    "manifest revision {}이 dataset revision {dataset_revision_ref}와 다릅니다",
                    manifest.subject.project_revision_ref
                ),
            ));
        }
    }

    let source_refs = validate_evidence_sources_internal(&manifest.sources)?;
    if manifest.claims.is_empty() {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_CLAIMS_EMPTY",
            "$.claims",
            "evidence claim은 하나 이상이어야 합니다",
        ));
    }

    for (index, activity) in manifest.activities.iter().enumerate() {
        let path = format!("$.activities[{index}]");
        if activity.input_source_refs.is_empty() {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_ACTIVITY_INPUTS_EMPTY",
                format!("{path}.input_source_refs"),
                "activity input source는 하나 이상이어야 합니다",
            ));
        }
        let mut activity_refs = HashSet::new();
        for (source_index, source_ref) in activity.input_source_refs.iter().enumerate() {
            let source_path = format!("{path}.input_source_refs[{source_index}]");
            validate_source_ref(source_ref, &source_refs, &source_path)?;
            if !activity_refs.insert(source_ref.as_str()) {
                return Err(EvidenceManifestError::new(
                    "E_EVIDENCE_ACTIVITY_DUPLICATE_SOURCE_REF",
                    source_path,
                    format!("activity input source_ref가 중복됩니다: {source_ref}"),
                ));
            }
        }
        if activity.output_targets.is_empty() {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_ACTIVITY_OUTPUTS_EMPTY",
                format!("{path}.output_targets"),
                "activity output target은 하나 이상이어야 합니다",
            ));
        }
        for (target_index, target) in activity.output_targets.iter().enumerate() {
            if target.dataset_content_sha256 != manifest.subject.dataset_content_sha256 {
                return Err(EvidenceManifestError::new(
                    "E_EVIDENCE_CROSS_RUN_TARGET",
                    format!("{path}.output_targets[{target_index}].dataset_content_sha256"),
                    "activity output은 다른 dataset/run target을 결합할 수 없습니다",
                ));
            }
            let status = validate_target(
                target,
                subject_dataset,
                &manifest.subject.dataset_content_sha256,
                &format!("{path}.output_targets[{target_index}]"),
            )?;
            if let Some(status) = status {
                if status != &EvidenceCellStatus::Present {
                    return Err(EvidenceManifestError::new(
                        "E_EVIDENCE_ACTIVITY_SPARSE_OUTPUT",
                        format!("{path}.output_targets[{target_index}]"),
                        format!(
                            "activity output은 present cell만 가리킬 수 있습니다: {}",
                            sparse_status_label(status)
                        ),
                    ));
                }
            }
        }
        validate_sha256_text(&activity.recipe_sha256, &format!("{path}.recipe_sha256"))?;
    }

    for (index, claim) in manifest.claims.iter().enumerate() {
        let path = format!("$.claims[{index}]");
        validate_source_ref(
            &claim.source_ref,
            &source_refs,
            &format!("{path}.source_ref"),
        )?;
        require_text(&claim.source_locator, format!("{path}.source_locator"))?;
        if let Some(note) = &claim.note {
            require_text(note, format!("{path}.note"))?;
        }
        let status = validate_target(
            &claim.target,
            subject_dataset,
            &manifest.subject.dataset_content_sha256,
            &format!("{path}.target"),
        )?;
        if claim.relation == EvidenceRelation::Context && claim.target.column_name.is_some() {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_CONTEXT_VALUE_PROMOTION",
                format!("{path}.target.column_name"),
                "context relation은 직접 cell 값 근거로 승격할 수 없습니다",
            ));
        }
        if let Some(status) = status {
            if status != &EvidenceCellStatus::Present {
                return Err(EvidenceManifestError::new(
                    "E_EVIDENCE_SPARSE_NON_PRESENT_CLAIM",
                    format!("{path}.target"),
                    format!(
                        "gap/missing/pruned cell을 present claim으로 인용할 수 없습니다: {}",
                        sparse_status_label(status)
                    ),
                ));
            }
        }
        if claim.relation == EvidenceRelation::DerivedFrom {
            let matching_activity = manifest.activities.iter().any(|activity| {
                activity
                    .output_targets
                    .iter()
                    .any(|target| target == &claim.target)
                    && activity
                        .input_source_refs
                        .iter()
                        .any(|source_ref| source_ref == &claim.source_ref)
            });
            if !matching_activity {
                return Err(EvidenceManifestError::new(
                    "E_EVIDENCE_DERIVED_ACTIVITY_REQUIRED",
                    path,
                    "derived_from claim에는 같은 target, input source, recipe hash의 activity가 필요합니다",
                ));
            }
        }
    }

    if verify_hashes {
        let expected_manifest_sha256 = evidence_manifest_sha256(manifest)?;
        if manifest.hashes.evidence_manifest_sha256 != expected_manifest_sha256 {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_MANIFEST_HASH_MISMATCH",
                "$.hashes.evidence_manifest_sha256",
                format!(
                    "evidence manifest hash 불일치: expected {expected_manifest_sha256}, actual {}",
                    manifest.hashes.evidence_manifest_sha256
                ),
            ));
        }
        let expected_binding_sha256 = evidence_content_binding_sha256(
            &manifest.subject.dataset_content_sha256,
            &expected_manifest_sha256,
        )?;
        if manifest.hashes.content_binding_sha256 != expected_binding_sha256 {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_CONTENT_BINDING_HASH_MISMATCH",
                "$.hashes.content_binding_sha256",
                format!(
                    "content binding hash 불일치: expected {expected_binding_sha256}, actual {}",
                    manifest.hashes.content_binding_sha256
                ),
            ));
        }
    }

    Ok(())
}

fn build_dataset_index<'a>(
    datasets: &'a [EvidenceDatasetTarget],
) -> Result<HashMap<&'a str, &'a EvidenceDatasetTarget>, EvidenceManifestError> {
    let mut index = HashMap::new();
    for (dataset_index, dataset) in datasets.iter().enumerate() {
        validate_sha256_text(
            &dataset.dataset_content_sha256,
            &format!("$context.datasets[{dataset_index}].dataset_content_sha256"),
        )?;
        if index
            .insert(dataset.dataset_content_sha256.as_str(), dataset)
            .is_some()
        {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_DUPLICATE_DATASET_CONTEXT",
                format!("$context.datasets[{dataset_index}].dataset_content_sha256"),
                "같은 dataset content hash가 검증 context에 중복됩니다",
            ));
        }
    }
    Ok(index)
}

fn validate_evidence_sources_internal<'a>(
    sources: &'a [EvidenceSource],
) -> Result<HashSet<&'a str>, EvidenceManifestError> {
    if sources.is_empty() {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_SOURCES_EMPTY",
            "$.sources",
            "evidence source는 하나 이상이어야 합니다",
        ));
    }
    let mut source_refs = HashSet::new();
    for (index, source) in sources.iter().enumerate() {
        let path = format!("$.sources[{index}]");
        require_text(&source.source_ref, format!("{path}.source_ref"))?;
        if !source_refs.insert(source.source_ref.as_str()) {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_DUPLICATE_SOURCE_REF",
                format!("{path}.source_ref"),
                format!("source_ref가 중복됩니다: {}", source.source_ref),
            ));
        }
        require_text(&source.title, format!("{path}.title"))?;
        require_text(&source.provider, format!("{path}.provider"))?;
        require_text(
            &source.canonical_identifier,
            format!("{path}.canonical_identifier"),
        )?;
        validate_canonical_identifier(
            &source.canonical_identifier,
            &format!("{path}.canonical_identifier"),
        )?;
        require_text(
            &source.published_or_version,
            format!("{path}.published_or_version"),
        )?;
        require_text(&source.retrieved_at, format!("{path}.retrieved_at"))?;
        require_text(
            &source.rights.license_id,
            format!("{path}.rights.license_id"),
        )?;
        require_text(&source.rights.holder, format!("{path}.rights.holder"))?;
        require_text(
            &source.rights.reuse_mode,
            format!("{path}.rights.reuse_mode"),
        )?;
        if let Some(payload_sha256) = &source.payload_sha256 {
            validate_sha256_text(payload_sha256, &format!("{path}.payload_sha256"))?;
        }
        if source.payload_mode == EvidencePayloadMode::Embedded && source.payload_sha256.is_none() {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_EMBEDDED_PAYLOAD_HASH_REQUIRED",
                format!("{path}.payload_sha256"),
                "embedded payload에는 payload_sha256이 필요합니다",
            ));
        }
    }
    Ok(source_refs)
}

fn validate_source_ref(
    source_ref: &str,
    source_refs: &HashSet<&str>,
    path: &str,
) -> Result<(), EvidenceManifestError> {
    require_text(source_ref, path)?;
    if !source_refs.contains(source_ref) {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_SOURCE_REF_NOT_FOUND",
            path,
            format!("등록되지 않은 source_ref입니다: {source_ref}"),
        ));
    }
    Ok(())
}

fn validate_target<'a>(
    target: &EvidenceTarget,
    subject_dataset: &'a EvidenceDatasetTarget,
    subject_dataset_hash: &str,
    path: &str,
) -> Result<Option<&'a EvidenceCellStatus>, EvidenceManifestError> {
    validate_sha256_text(
        &target.dataset_content_sha256,
        &format!("{path}.dataset_content_sha256"),
    )?;
    if target.dataset_content_sha256 != subject_dataset_hash {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_CROSS_DATASET_TARGET",
            format!("{path}.dataset_content_sha256"),
            "claim/activity target은 subject dataset을 벗어날 수 없습니다",
        ));
    }
    let row = subject_dataset
        .rows
        .iter()
        .find(|row| row.source_ordinal == target.row_source_ordinal)
        .ok_or_else(|| {
            EvidenceManifestError::new(
                "E_EVIDENCE_ROW_NOT_FOUND",
                format!("{path}.row_source_ordinal"),
                format!(
                    "dataset에 source ordinal {} 행이 없습니다",
                    target.row_source_ordinal
                ),
            )
        })?;
    if let Some(column_name) = &target.column_name {
        require_text(column_name, format!("{path}.column_name"))?;
        let column_index = subject_dataset
            .column_names
            .iter()
            .position(|name| name == column_name)
            .ok_or_else(|| {
                EvidenceManifestError::new(
                    "E_EVIDENCE_COLUMN_NOT_FOUND",
                    format!("{path}.column_name"),
                    format!("dataset에 {column_name} 열이 없습니다"),
                )
            })?;
        let status = row.cell_statuses.get(column_index).ok_or_else(|| {
            EvidenceManifestError::new(
                "E_EVIDENCE_DATASET_CONTEXT_WIDTH",
                format!("{path}.column_name"),
                "dataset evidence context의 cell status 폭이 column 수와 다릅니다",
            )
        })?;
        return Ok(Some(status));
    }
    Ok(None)
}

fn sparse_status_label(status: &EvidenceCellStatus) -> String {
    match status {
        EvidenceCellStatus::Present => "present".to_string(),
        EvidenceCellStatus::Gap => "gap".to_string(),
        EvidenceCellStatus::Missing { reason } => format!("missing({reason})"),
        EvidenceCellStatus::Pruned => "pruned".to_string(),
    }
}

fn validate_sha256_text(value: &str, path: &str) -> Result<(), EvidenceManifestError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_SHA256",
            path,
            "SHA-256은 64자 lowercase hex여야 합니다",
        ));
    }
    Ok(())
}

fn require_text(value: &str, path: impl Into<String>) -> Result<(), EvidenceManifestError> {
    let path = path.into();
    if value.trim().is_empty() {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_REQUIRED_TEXT",
            path,
            "필수 글 값이 비어 있습니다",
        ));
    }
    Ok(())
}

fn scan_forbidden_material(value: &JsonValue, path: &str) -> Result<(), EvidenceManifestError> {
    match value {
        JsonValue::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{path}.{key}");
                let lowered = key.to_ascii_lowercase();
                if [
                    "capability",
                    "authorization",
                    "access_token",
                    "refresh_token",
                    "password",
                    "credential",
                    "private_key",
                    "server_path",
                ]
                .iter()
                .any(|token| lowered.contains(token))
                {
                    return Err(EvidenceManifestError::new(
                        "E_EVIDENCE_SECRET_FIELD",
                        child_path,
                        "secret, capability 또는 authority field를 evidence에 넣을 수 없습니다",
                    ));
                }
                scan_forbidden_material(child, &child_path)?;
            }
        }
        JsonValue::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                scan_forbidden_material(child, &format!("{path}[{index}]"))?;
            }
        }
        JsonValue::String(text) => {
            let lowered = text.to_ascii_lowercase();
            if lowered.contains("bearer ")
                || lowered.contains("access_token=")
                || lowered.contains("refresh_token=")
                || lowered.contains("capability=")
                || lowered.contains("-----begin private key-----")
            {
                return Err(EvidenceManifestError::new(
                    "E_EVIDENCE_SECRET_VALUE",
                    path,
                    "secret, capability 또는 authority 값을 evidence에 넣을 수 없습니다",
                ));
            }
            if (lowered.starts_with("http://")
                || lowered.starts_with("https://")
                || lowered.starts_with("doi:")
                || lowered.starts_with("ark:/"))
                && !path.ends_with(".canonical_identifier")
            {
                return Err(EvidenceManifestError::new(
                    "E_EVIDENCE_IDENTIFIER_FIELD",
                    path,
                    "공개 identifier는 canonical_identifier에서만 허용됩니다",
                ));
            }
            if looks_like_host_path(text) || lowered.starts_with("file://") {
                return Err(EvidenceManifestError::new(
                    "E_EVIDENCE_ABSOLUTE_PATH",
                    path,
                    "host absolute path를 evidence에 넣을 수 없습니다",
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn looks_like_host_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    (bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/'))
        || value.starts_with("\\\\")
        || ["/home/", "/Users/", "/tmp/", "/var/", "/etc/"]
            .iter()
            .any(|prefix| value.starts_with(prefix))
}

fn validate_canonical_identifier(value: &str, path: &str) -> Result<(), EvidenceManifestError> {
    let trimmed = value.trim();
    let lowered = trimmed.to_ascii_lowercase();
    if lowered.starts_with("http://") {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_IDENTIFIER_HTTPS_REQUIRED",
            path,
            "canonical_identifier의 web record는 HTTPS여야 합니다",
        ));
    }
    if lowered.starts_with("doi:") {
        let suffix = &trimmed[4..];
        if !suffix.starts_with("10.") || !suffix.contains('/') {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_IDENTIFIER_DOI",
                path,
                "DOI identifier는 doi:10.<registrant>/<suffix> 형식이어야 합니다",
            ));
        }
        return validate_identifier_suffix(suffix, path);
    }
    if lowered.starts_with("ark:/") {
        let suffix = &trimmed[5..];
        if suffix.trim_matches('/').is_empty() {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_IDENTIFIER_ARK",
                path,
                "ARK identifier가 비어 있습니다",
            ));
        }
        return validate_identifier_suffix(suffix, path);
    }
    let Some(rest) = trimmed.strip_prefix("https://") else {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_IDENTIFIER_SCHEME",
            path,
            "canonical_identifier는 HTTPS, DOI 또는 ARK stable record여야 합니다",
        ));
    };
    validate_identifier_suffix(rest, path)?;
    let (authority, record_path) = rest.split_once('/').unwrap_or((rest, ""));
    if authority.is_empty() || authority.contains('@') {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_IDENTIFIER_AUTHORITY",
            path,
            "canonical_identifier에 빈 host나 userinfo를 넣을 수 없습니다",
        ));
    }
    if authority.contains(':') || authority.starts_with('[') || authority.ends_with(']') {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_IDENTIFIER_AUTHORITY",
            path,
            "canonical_identifier에 explicit port나 bracket host를 넣을 수 없습니다",
        ));
    }
    let host = authority.trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty()
        || host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
    {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_IDENTIFIER_PRIVATE_HOST",
            path,
            "localhost 또는 local host는 canonical_identifier가 될 수 없습니다",
        ));
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        let non_public = match ip {
            IpAddr::V4(ip) => {
                ip.is_private()
                    || ip.is_loopback()
                    || ip.is_link_local()
                    || ip.is_unspecified()
                    || ip.is_multicast()
            }
            IpAddr::V6(ip) => {
                let first = ip.segments()[0];
                ip.is_loopback()
                    || ip.is_unspecified()
                    || ip.is_multicast()
                    || (first & 0xfe00) == 0xfc00
                    || (first & 0xffc0) == 0xfe80
            }
        };
        if non_public {
            return Err(EvidenceManifestError::new(
                "E_EVIDENCE_IDENTIFIER_PRIVATE_HOST",
                path,
                "private, loopback 또는 link-local 주소는 허용되지 않습니다",
            ));
        }
    }
    let segments = record_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    if segments.iter().any(|segment| {
        [
            "mutate", "mutation", "write", "upload", "delete", "token", "auth", "admin",
        ]
        .contains(&segment.as_str())
    }) {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_IDENTIFIER_MUTATION_ENDPOINT",
            path,
            "mutation/authority endpoint는 canonical record identifier가 될 수 없습니다",
        ));
    }
    Ok(())
}

fn validate_identifier_suffix(value: &str, path: &str) -> Result<(), EvidenceManifestError> {
    if value.contains('?') || value.contains('#') {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_IDENTIFIER_DECORATION",
            path,
            "canonical_identifier에 query나 fragment를 넣을 수 없습니다",
        ));
    }
    if value.contains('@') {
        return Err(EvidenceManifestError::new(
            "E_EVIDENCE_IDENTIFIER_AUTHORITY",
            path,
            "canonical_identifier에 userinfo를 넣을 수 없습니다",
        ));
    }
    Ok(())
}

fn canonical_json_bytes(value: &JsonValue) -> Vec<u8> {
    serde_json::to_vec(&canonical_json(value)).unwrap_or_default()
}

fn canonical_json(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            let mut out = serde_json::Map::new();
            for key in keys {
                out.insert(key.clone(), canonical_json(&map[key]));
            }
            JsonValue::Object(out)
        }
        JsonValue::Array(items) => JsonValue::Array(items.iter().map(canonical_json).collect()),
        other => other.clone(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const DATASET_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OTHER_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    fn dataset_context() -> Vec<EvidenceDatasetTarget> {
        vec![EvidenceDatasetTarget {
            dataset_content_sha256: DATASET_HASH.to_string(),
            project_revision_ref: Some("revision:chem-01".to_string()),
            column_names: vec!["물질".to_string(), "물질량".to_string()],
            rows: vec![
                EvidenceDatasetRowTarget {
                    source_ordinal: 7,
                    cell_statuses: vec![EvidenceCellStatus::Present, EvidenceCellStatus::Present],
                },
                EvidenceDatasetRowTarget {
                    source_ordinal: 2,
                    cell_statuses: vec![
                        EvidenceCellStatus::Gap,
                        EvidenceCellStatus::Missing {
                            reason: "#원천없음".to_string(),
                        },
                    ],
                },
                EvidenceDatasetRowTarget {
                    source_ordinal: 3,
                    cell_statuses: vec![EvidenceCellStatus::Pruned, EvidenceCellStatus::Present],
                },
            ],
        }]
    }

    fn manifest_value() -> JsonValue {
        let raw = json!({
            "schema": DOMAIN_EVIDENCE_MANIFEST_SCHEMA,
            "subject": {
                "dataset_content_sha256": DATASET_HASH,
                "project_revision_ref": "revision:chem-01"
            },
            "sources": [
                {
                    "source_ref": "source:first",
                    "title": "첫 출처",
                    "provider": "공급자",
                    "canonical_identifier": "https://example.org/source/first",
                    "published_or_version": "v1",
                    "retrieved_at": "2026-07-28",
                    "rights": {
                        "license_id": "CC-BY-4.0",
                        "holder": "첫 권리자",
                        "reuse_mode": "attribution"
                    },
                    "payload_mode": "reference"
                },
                {
                    "source_ref": "source:second",
                    "title": "둘째 출처",
                    "provider": "공급자",
                    "canonical_identifier": "https://example.org/source/second",
                    "published_or_version": "v2",
                    "retrieved_at": "2026-07-28",
                    "rights": {
                        "license_id": "LicenseRef-W1",
                        "holder": "둘째 권리자",
                        "reuse_mode": "sealed_embedded"
                    },
                    "payload_mode": "embedded",
                    "payload_sha256": OTHER_HASH
                }
            ],
            "claims": [
                {
                    "target": {
                        "dataset_content_sha256": DATASET_HASH,
                        "row_source_ordinal": 7,
                        "column_name": "물질량"
                    },
                    "relation": "direct_source",
                    "source_ref": "source:first",
                    "source_locator": "table:row=1;column=amount"
                },
                {
                    "target": {
                        "dataset_content_sha256": DATASET_HASH,
                        "row_source_ordinal": 2
                    },
                    "relation": "context",
                    "source_ref": "source:second",
                    "source_locator": "section:2",
                    "note": "ordered second claim"
                }
            ],
            "activities": [
                {
                    "kind": "extract",
                    "input_source_refs": ["source:second", "source:first"],
                    "output_targets": [
                        {
                            "dataset_content_sha256": DATASET_HASH,
                            "row_source_ordinal": 7,
                            "column_name": "물질"
                        }
                    ],
                    "recipe_sha256": OTHER_HASH
                }
            ],
            "hashes": {
                "evidence_manifest_sha256": ZERO_HASH,
                "content_binding_sha256": ZERO_HASH
            }
        });
        let manifest: DomainEvidenceManifest =
            serde_json::from_value(raw).expect("typed manifest fixture");
        serde_json::to_value(
            seal_domain_evidence_manifest(manifest, &dataset_context()).expect("seal manifest"),
        )
        .expect("sealed manifest value")
    }

    fn parse(value: &JsonValue) -> Result<DomainEvidenceManifest, EvidenceManifestError> {
        parse_and_validate_domain_evidence_manifest(
            &serde_json::to_string(value).expect("serialize"),
            &dataset_context(),
        )
    }

    fn reseal_value(value: &JsonValue) -> JsonValue {
        let manifest: DomainEvidenceManifest =
            serde_json::from_value(value.clone()).expect("typed manifest");
        serde_json::to_value(
            seal_domain_evidence_manifest(manifest, &dataset_context()).expect("reseal"),
        )
        .expect("resealed value")
    }

    #[test]
    fn ordered_manifest_preserves_source_claim_activity_order() {
        let manifest = parse(&manifest_value()).expect("valid ordered manifest");
        assert_eq!(manifest.sources[0].source_ref, "source:first");
        assert_eq!(manifest.sources[1].source_ref, "source:second");
        assert_eq!(manifest.claims[0].target.row_source_ordinal, 7);
        assert_eq!(manifest.claims[1].target.row_source_ordinal, 2);
        assert_eq!(
            manifest.activities[0].input_source_refs,
            vec!["source:second", "source:first"]
        );
    }

    #[test]
    fn strict_manifest_rejects_unknown_duplicate_and_missing_source_refs() {
        let mut unknown = manifest_value();
        unknown["claims"][0]["synthetic_meaning_id"] = json!("project.obs.0");
        assert_eq!(
            parse(&unknown).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_JSON"
        );

        let mut duplicate = manifest_value();
        duplicate["sources"][1]["source_ref"] = json!("source:first");
        assert_eq!(
            parse(&duplicate).unwrap_err().code,
            "E_EVIDENCE_DUPLICATE_SOURCE_REF"
        );

        let mut missing = manifest_value();
        missing["claims"][0]["source_ref"] = json!("source:missing");
        assert_eq!(
            parse(&missing).unwrap_err().code,
            "E_EVIDENCE_SOURCE_REF_NOT_FOUND"
        );
    }

    #[test]
    fn strict_manifest_rejects_missing_or_cross_dataset_targets() {
        let mut row = manifest_value();
        row["claims"][0]["target"]["row_source_ordinal"] = json!(999);
        assert_eq!(parse(&row).unwrap_err().code, "E_EVIDENCE_ROW_NOT_FOUND");

        let mut column = manifest_value();
        column["claims"][0]["target"]["column_name"] = json!("없는열");
        assert_eq!(
            parse(&column).unwrap_err().code,
            "E_EVIDENCE_COLUMN_NOT_FOUND"
        );

        let mut cross = manifest_value();
        cross["claims"][0]["target"]["dataset_content_sha256"] = json!(OTHER_HASH);
        assert_eq!(
            parse(&cross).unwrap_err().code,
            "E_EVIDENCE_CROSS_DATASET_TARGET"
        );
    }

    #[test]
    fn strict_manifest_rejects_invalid_enums_and_embedded_payload_without_hash() {
        let mut relation = manifest_value();
        relation["claims"][0]["relation"] = json!("supports");
        assert_eq!(
            parse(&relation).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_JSON"
        );

        let mut activity = manifest_value();
        activity["activities"][0]["kind"] = json!("infer");
        assert_eq!(
            parse(&activity).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_JSON"
        );

        let mut payload_mode = manifest_value();
        payload_mode["sources"][0]["payload_mode"] = json!("network");
        assert_eq!(
            parse(&payload_mode).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_JSON"
        );

        let mut embedded = manifest_value();
        embedded["sources"][1]
            .as_object_mut()
            .expect("source")
            .remove("payload_sha256");
        assert_eq!(
            parse(&embedded).unwrap_err().code,
            "E_EVIDENCE_EMBEDDED_PAYLOAD_HASH_REQUIRED"
        );

        let mut embedded_without_rights = manifest_value();
        embedded_without_rights["sources"][1]
            .as_object_mut()
            .expect("source")
            .remove("rights");
        assert_eq!(
            parse(&embedded_without_rights).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_JSON"
        );
    }

    #[test]
    fn sparse_gap_missing_and_pruned_cells_cannot_be_present_claims() {
        for (row_source_ordinal, column_name, expected_label) in [
            (2, "물질", "gap"),
            (2, "물질량", "missing(#원천없음)"),
            (3, "물질", "pruned"),
        ] {
            let mut value = manifest_value();
            value["claims"][0]["target"]["row_source_ordinal"] = json!(row_source_ordinal);
            value["claims"][0]["target"]["column_name"] = json!(column_name);
            let error = parse(&value).expect_err("sparse cell claim must fail");
            assert_eq!(error.code, "E_EVIDENCE_SPARSE_NON_PRESENT_CLAIM");
            assert!(error.message.contains(expected_label));
        }
    }

    #[test]
    fn context_cannot_be_promoted_to_direct_cell_value_evidence() {
        let mut value = manifest_value();
        value["claims"][1]["target"]["column_name"] = json!("물질량");
        let error = parse(&value).expect_err("context cell claim must fail");
        assert_eq!(error.code, "E_EVIDENCE_CONTEXT_VALUE_PROMOTION");
    }

    #[test]
    fn derived_claim_requires_matching_activity_input_target_and_recipe() {
        let mut missing_activity = manifest_value();
        missing_activity["claims"][0]["relation"] = json!("derived_from");
        let error = parse(&missing_activity).expect_err("unbound derived claim must fail");
        assert_eq!(error.code, "E_EVIDENCE_DERIVED_ACTIVITY_REQUIRED");

        let mut valid = missing_activity;
        valid["activities"][0]["output_targets"][0]["column_name"] = json!("물질량");
        valid = reseal_value(&valid);
        parse(&valid).expect("matching derived activity");

        valid["activities"][0]
            .as_object_mut()
            .expect("activity")
            .remove("recipe_sha256");
        assert_eq!(parse(&valid).unwrap_err().code, "E_EVIDENCE_MANIFEST_JSON");
    }

    #[test]
    fn quote_requires_locator_and_activity_rejects_cross_run_or_sparse_output() {
        let mut quote = manifest_value();
        quote["claims"][0]["relation"] = json!("quoted_from");
        quote["claims"][0]["source_locator"] = json!("");
        assert_eq!(parse(&quote).unwrap_err().code, "E_EVIDENCE_REQUIRED_TEXT");

        let mut cross_run = manifest_value();
        cross_run["activities"][0]["output_targets"][0]["dataset_content_sha256"] =
            json!(OTHER_HASH);
        assert_eq!(
            parse(&cross_run).unwrap_err().code,
            "E_EVIDENCE_CROSS_RUN_TARGET"
        );

        let mut sparse_output = manifest_value();
        sparse_output["activities"][0]["output_targets"][0]["row_source_ordinal"] = json!(2);
        assert_eq!(
            parse(&sparse_output).unwrap_err().code,
            "E_EVIDENCE_ACTIVITY_SPARSE_OUTPUT"
        );
    }

    #[test]
    fn cross_run_subject_is_rejected_and_ordered_multi_relation_target_is_allowed() {
        let mut cross_run = manifest_value();
        cross_run["subject"]["project_revision_ref"] = json!("revision:other-run");
        assert_eq!(
            parse(&cross_run).unwrap_err().code,
            "E_EVIDENCE_CROSS_RUN_SUBJECT"
        );

        let mut multi_relation = manifest_value();
        let shared_target = multi_relation["claims"][0]["target"].clone();
        multi_relation["claims"][1]["target"] = shared_target;
        multi_relation["claims"][1]["relation"] = json!("primary_source");
        multi_relation = reseal_value(&multi_relation);
        let parsed = parse(&multi_relation).expect("ordered multi relation target");
        assert_eq!(parsed.claims[0].relation, EvidenceRelation::DirectSource);
        assert_eq!(parsed.claims[1].relation, EvidenceRelation::PrimarySource);
        assert_eq!(parsed.claims[0].target, parsed.claims[1].target);
    }

    #[test]
    fn manifest_and_content_binding_hashes_are_deterministic_and_fail_closed() {
        let value = manifest_value();
        let parsed = parse(&value).expect("sealed manifest");
        assert_eq!(
            parsed.hashes.evidence_manifest_sha256,
            evidence_manifest_sha256(&parsed).expect("manifest hash")
        );
        assert_eq!(
            parsed.hashes.content_binding_sha256,
            evidence_content_binding_sha256(
                &parsed.subject.dataset_content_sha256,
                &parsed.hashes.evidence_manifest_sha256,
            )
            .expect("binding hash")
        );
        let canonical_first = canonical_domain_evidence_manifest_json(&parsed, &dataset_context())
            .expect("canonical first");
        let canonical_second = canonical_domain_evidence_manifest_json(&parsed, &dataset_context())
            .expect("canonical second");
        assert_eq!(canonical_first, canonical_second);

        let mut tampered = value.clone();
        tampered["claims"][0]["note"] = json!("hash 이후 변경");
        assert_eq!(
            parse(&tampered).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_HASH_MISMATCH"
        );

        let mut binding = value;
        binding["hashes"]["content_binding_sha256"] = json!(OTHER_HASH);
        assert_eq!(
            parse(&binding).unwrap_err().code,
            "E_EVIDENCE_CONTENT_BINDING_HASH_MISMATCH"
        );

        let mut identifier = manifest_value();
        identifier["sources"][0]["canonical_identifier"] =
            json!("https://example.org/stable/source-b");
        assert_eq!(
            parse(&identifier).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_HASH_MISMATCH"
        );

        let mut reordered = manifest_value();
        reordered["sources"]
            .as_array_mut()
            .expect("sources")
            .swap(0, 1);
        assert_eq!(
            parse(&reordered).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_HASH_MISMATCH"
        );

        let mut resealed_reorder = reordered;
        resealed_reorder = reseal_value(&resealed_reorder);
        let reordered_manifest = parse(&resealed_reorder).expect("explicit reordered manifest");
        assert_eq!(reordered_manifest.sources[0].source_ref, "source:second");
        assert_ne!(
            reordered_manifest.hashes.evidence_manifest_sha256,
            parsed.hashes.evidence_manifest_sha256
        );
        assert_ne!(
            reordered_manifest.hashes.content_binding_sha256,
            parsed.hashes.content_binding_sha256
        );

        let mut claims_reordered = manifest_value();
        claims_reordered["claims"]
            .as_array_mut()
            .expect("claims")
            .swap(0, 1);
        assert_eq!(
            parse(&claims_reordered).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_HASH_MISMATCH"
        );
        claims_reordered = reseal_value(&claims_reordered);
        let claims_reordered_manifest =
            parse(&claims_reordered).expect("explicit reordered claims");
        assert_eq!(
            claims_reordered_manifest.claims[0].source_ref,
            "source:second"
        );
        assert_ne!(
            claims_reordered_manifest.hashes.evidence_manifest_sha256,
            parsed.hashes.evidence_manifest_sha256
        );
        assert_ne!(
            claims_reordered_manifest.hashes.content_binding_sha256,
            parsed.hashes.content_binding_sha256
        );

        let mut activities_ordered = manifest_value();
        let mut second_activity = activities_ordered["activities"][0].clone();
        second_activity["kind"] = json!("normalize");
        second_activity["input_source_refs"] = json!(["source:first"]);
        second_activity["output_targets"][0]["column_name"] = json!("물질량");
        second_activity["recipe_sha256"] = json!(DATASET_HASH);
        activities_ordered["activities"]
            .as_array_mut()
            .expect("activities")
            .push(second_activity);
        activities_ordered = reseal_value(&activities_ordered);
        let activities_ordered_manifest =
            parse(&activities_ordered).expect("two ordered activities");

        let mut activities_reordered = activities_ordered;
        activities_reordered["activities"]
            .as_array_mut()
            .expect("activities")
            .swap(0, 1);
        assert_eq!(
            parse(&activities_reordered).unwrap_err().code,
            "E_EVIDENCE_MANIFEST_HASH_MISMATCH"
        );
        activities_reordered = reseal_value(&activities_reordered);
        let activities_reordered_manifest =
            parse(&activities_reordered).expect("explicit reordered activities");
        assert_eq!(
            activities_reordered_manifest.activities[0].kind,
            EvidenceActivityKind::Normalize
        );
        assert_ne!(
            activities_reordered_manifest
                .hashes
                .evidence_manifest_sha256,
            activities_ordered_manifest
                .hashes
                .evidence_manifest_sha256
        );
        assert_ne!(
            activities_reordered_manifest
                .hashes
                .content_binding_sha256,
            activities_ordered_manifest
                .hashes
                .content_binding_sha256
        );
    }

    #[test]
    fn canonical_identifier_is_the_only_restricted_public_identifier_surface() {
        let mut http = manifest_value();
        http["sources"][0]["canonical_identifier"] = json!("http://example.org/source");
        assert_eq!(
            parse(&http).unwrap_err().code,
            "E_EVIDENCE_IDENTIFIER_HTTPS_REQUIRED"
        );

        let mut decorated = manifest_value();
        decorated["sources"][0]["canonical_identifier"] =
            json!("https://example.org/source?token=none");
        assert_eq!(
            parse(&decorated).unwrap_err().code,
            "E_EVIDENCE_IDENTIFIER_DECORATION"
        );

        for invalid in [
            "https://localhost/source",
            "https://127.0.0.1/source",
            "https://169.254.1.1/source",
            "https://example.org/upload",
            "file:///tmp/source",
        ] {
            let mut value = manifest_value();
            value["sources"][0]["canonical_identifier"] = json!(invalid);
            assert!(parse(&value).is_err(), "{invalid} must fail");
        }

        for valid in [
            "https://example.org/stable/source",
            "doi:10.1000/example",
            "ark:/12345/example",
        ] {
            let mut value = manifest_value();
            value["sources"][0]["canonical_identifier"] = json!(valid);
            let sealed = reseal_value(&value);
            parse(&sealed).expect("allowed canonical identifier");
        }

        let mut wrong_field = manifest_value();
        wrong_field["claims"][0]["note"] = json!("https://example.org/not-allowed-here");
        assert_eq!(
            parse(&wrong_field).unwrap_err().code,
            "E_EVIDENCE_IDENTIFIER_FIELD"
        );
    }

    #[test]
    fn strict_manifest_rejects_secret_capability_and_host_path_material() {
        let mut secret = manifest_value();
        secret["sources"][0]["note_access_token"] = json!("secret");
        assert_eq!(parse(&secret).unwrap_err().code, "E_EVIDENCE_SECRET_FIELD");

        let mut capability = manifest_value();
        capability["sources"][0]["canonical_identifier"] =
            json!("https://example.org/source?capability=secret");
        assert_eq!(
            parse(&capability).unwrap_err().code,
            "E_EVIDENCE_SECRET_VALUE"
        );

        let mut path = manifest_value();
        path["sources"][0]["canonical_identifier"] = json!("C:\\private\\source.json");
        assert_eq!(parse(&path).unwrap_err().code, "E_EVIDENCE_ABSOLUTE_PATH");
    }
}
