//! 창작자가 기존 봉인 자료와 근거 artifact를 만드는 공통 제품 경계.
//!
//! 이 모듈의 `Draft` 구조는 Rust/CLI/WASM 제품 frontdoor 사이에서만 쓰는
//! 비정본 전송 구조다. 정본 산출물은 오직
//! `ddn.sealed_domain_dataset.v1`과 `ddn.domain_evidence_manifest.v1`이며,
//! 각 기존 owner의 seal/validate/canonical 함수를 그대로 사용한다.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt;

use blake3::hash as blake3_hash;
use ddonirang_core::{asset_handle_from_bundle_path, Fixed64};

use crate::corpus_csv::{
    parse_corpus_csv_bytes, CorpusCsvDiagnosticSeverity, CorpusCsvOptions, CorpusCsvStatus,
};
use crate::domain_evidence_manifest::{
    canonical_domain_evidence_manifest_json, parse_and_validate_domain_evidence_manifest,
    seal_domain_evidence_manifest, validate_domain_evidence_manifest, validate_evidence_sources,
    DomainEvidenceManifest, EvidenceActivity, EvidenceActivityKind, EvidenceCellStatus,
    EvidenceClaim, EvidenceDatasetRowTarget, EvidenceDatasetTarget, EvidenceManifestError,
    EvidenceManifestHashes, EvidenceRelation, EvidenceSource, EvidenceSubject, EvidenceTarget,
    DOMAIN_EVIDENCE_MANIFEST_SCHEMA,
};
use crate::gate0_registry::build_project_asset_registry_from_manifest;
use crate::sealed_domain_dataset::{
    canonical_sealed_domain_dataset_json, parse_and_validate_sealed_domain_dataset,
    seal_domain_dataset, validate_sealed_domain_dataset, DatasetCell, DatasetColumn, DatasetDtype,
    DatasetError, DatasetPayload, DatasetRow, DatasetScalar, DatasetSource, MissingReason,
    SealedDomainDataset, SEALED_DOMAIN_DATASET_SCHEMA,
};

pub const CORPUS_COLUMNS_TRANSPORT: &str = "seamgrim.internal.corpus_columns.v1";
pub const CORPUS_SOURCES_TRANSPORT: &str = "seamgrim.internal.corpus_sources.v1";
pub const CORPUS_EVIDENCE_TRANSPORT: &str = "seamgrim.internal.corpus_evidence.v1";
const CORPUS_SOURCES_HASH_PREFIX: &[u8] = b"seamgrim.corpus-sources.v1\0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusAuthoringError {
    pub code: String,
    pub path: String,
    pub message: String,
}

impl CorpusAuthoringError {
    fn new(code: impl Into<String>, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for CorpusAuthoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}: {}", self.code, self.path, self.message)
    }
}

impl std::error::Error for CorpusAuthoringError {}

impl From<DatasetError> for CorpusAuthoringError {
    fn from(value: DatasetError) -> Self {
        Self::new(value.code, value.path, value.message)
    }
}

impl From<EvidenceManifestError> for CorpusAuthoringError {
    fn from(value: EvidenceManifestError) -> Self {
        Self::new(value.code, value.path, value.message)
    }
}

/// 비정본 저작 입력. Dataset hash와 evidence hash를 사용자가 주입하지 못하게
/// 하고 기존 Rust seal owner가 결정하도록 한다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEvidenceDraft {
    pub project_revision_ref: String,
    pub sources: Vec<EvidenceSource>,
    pub claims: Vec<CorpusEvidenceClaimDraft>,
    pub activities: Vec<CorpusEvidenceActivityDraft>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEvidenceClaimDraft {
    pub target: CorpusEvidenceTargetDraft,
    pub relation: EvidenceRelation,
    pub source_ref: String,
    pub source_locator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEvidenceActivityDraft {
    pub kind: EvidenceActivityKind,
    pub input_source_refs: Vec<String>,
    pub output_targets: Vec<CorpusEvidenceTargetDraft>,
    pub recipe_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEvidenceTargetDraft {
    pub row_source_ordinal: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SealedCorpusArtifacts {
    pub dataset: SealedDomainDataset,
    pub evidence_manifest: DomainEvidenceManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalCorpusArtifacts {
    pub dataset_json: String,
    pub evidence_manifest_json: String,
    pub dataset_content_sha256: String,
    pub evidence_manifest_sha256: String,
    pub content_binding_sha256: String,
}

/// 비정본 열 저작 요청. `description`과 `missing_policy`는 저작 단계의
/// 명시적 결정이고, 기존 v1 dataset artifact에 존재하는 필드만 canonical
/// dataset으로 내려간다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusColumnsRequest {
    pub transport_kind: String,
    pub source_bytes: Vec<u8>,
    pub expected_source_bytes_sha256: String,
    pub csv_options: CorpusCsvOptions,
    pub artifact_id: String,
    pub artifact_version: String,
    pub project_revision_ref: String,
    pub source_ref: String,
    pub author_label: String,
    pub object_ref: String,
    pub media_type: String,
    pub ordered_columns: Vec<CorpusColumnDraft>,
    pub ordered_raw_rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusColumnDraft {
    pub source_index: usize,
    pub name: String,
    pub dtype: DatasetDtype,
    pub unit: Option<String>,
    pub missing_policy: CorpusMissingPolicy,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusMissingPolicy {
    Gap,
    SourceUnavailable,
    RecordDamaged,
    Pruned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusColumnsResult {
    pub transport_kind: &'static str,
    pub source_bytes_sha256: String,
    pub ordered_columns: Vec<CorpusColumnDraft>,
    pub row_count: usize,
    pub dataset_content_sha256: String,
    pub canonical_dataset_json: String,
    pub target_rows: Vec<CorpusTargetRowProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusTargetRowProjection {
    pub row_source_ordinal: u64,
    pub cells: Vec<CorpusTargetCellProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusTargetCellProjection {
    pub column_name: String,
    pub status: CorpusTargetCellStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusTargetCellStatus {
    Present,
    Gap,
    SourceUnavailable,
    RecordDamaged,
    CalculationFailed,
    Pruned,
}

/// 기존 `EvidenceSource` vector를 그대로 받는 비정본 저작 전송이다.
/// 최종 정본은 G5/G6에서 같은 순서로 `ddn.domain_evidence_manifest.v1`에
/// 들어가며, 이 transport 자체는 artifact schema가 아니다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusSourcesRequest {
    pub transport_kind: String,
    pub source_bytes: Vec<u8>,
    pub expected_source_bytes_sha256: String,
    pub canonical_dataset_json: String,
    pub expected_dataset_content_sha256: String,
    pub ordered_sources: Vec<EvidenceSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusSourcesResult {
    pub transport_kind: &'static str,
    pub source_bytes_sha256: String,
    pub dataset_content_sha256: String,
    pub source_records_sha256: String,
    pub ordered_sources: Vec<EvidenceSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEvidenceRequest {
    pub transport_kind: String,
    pub source_bytes: Vec<u8>,
    pub expected_source_bytes_sha256: String,
    pub canonical_dataset_json: String,
    pub expected_dataset_content_sha256: String,
    pub ordered_sources: Vec<EvidenceSource>,
    pub expected_source_records_sha256: String,
    pub project_revision_ref: String,
    pub claims: Vec<CorpusEvidenceClaimDraft>,
    pub activities: Vec<CorpusEvidenceActivityDraft>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusEvidenceResult {
    pub transport_kind: &'static str,
    pub source_bytes_sha256: String,
    pub dataset_content_sha256: String,
    pub source_records_sha256: String,
    pub evidence_manifest_sha256: String,
    pub content_binding_sha256: String,
    pub claim_count: usize,
    pub activity_count: usize,
    pub canonical_evidence_manifest_json: String,
}

pub const CORPUS_REVIEW_TRANSPORT: &str = "seamgrim.internal.corpus_review.v1";
pub const CORPUS_SEAL_TRANSPORT: &str = "seamgrim.internal.corpus_seal.v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusReviewRequest {
    pub transport_kind: String,
    pub source_bytes: Vec<u8>,
    pub csv_options: CorpusCsvOptions,
    pub expected_source_bytes_sha256: String,
    pub canonical_dataset_json: String,
    pub expected_dataset_content_sha256: String,
    pub ordered_sources: Vec<EvidenceSource>,
    pub expected_source_records_sha256: String,
    pub project_revision_ref: String,
    pub claims: Vec<CorpusEvidenceClaimDraft>,
    pub activities: Vec<CorpusEvidenceActivityDraft>,
    pub expected_evidence_manifest_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusReviewSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusReviewIssue {
    pub severity: CorpusReviewSeverity,
    pub step: String,
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CorpusSparseSummary {
    pub gap: usize,
    pub source_unavailable: usize,
    pub record_damaged: usize,
    pub calculation_failed: usize,
    pub pruned: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusReviewUnit {
    pub column_name: String,
    pub exact_unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusReviewResult {
    pub transport_kind: &'static str,
    pub ready_to_seal: bool,
    pub source_bytes_sha256: String,
    pub dataset_content_sha256: String,
    pub evidence_manifest_sha256: String,
    pub row_count: usize,
    pub column_count: usize,
    pub units: Vec<CorpusReviewUnit>,
    pub source_count: usize,
    pub claim_count: usize,
    pub activity_count: usize,
    pub sparse: CorpusSparseSummary,
    pub errors: Vec<CorpusReviewIssue>,
    pub warnings: Vec<CorpusReviewIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusSealRequest {
    pub transport_kind: String,
    pub source_bytes: Vec<u8>,
    pub expected_source_bytes_sha256: String,
    pub canonical_dataset_json: String,
    pub expected_dataset_content_sha256: String,
    pub canonical_evidence_manifest_json: String,
    pub expected_evidence_manifest_sha256: String,
    pub expected_content_binding_sha256: String,
    pub expected_project_revision_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusSealedProjectFile {
    pub path: String,
    pub role: String,
    pub utf8_text: String,
    pub byte_length: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusSealResult {
    pub transport_kind: &'static str,
    pub source_bytes_sha256: String,
    pub dataset_content_sha256: String,
    pub evidence_manifest_sha256: String,
    pub content_binding_sha256: String,
    pub asset_manifest_sha256: String,
    pub dataset_handle: String,
    pub dataset_envelope_path: String,
    pub dataset_payload_path: String,
    pub evidence_manifest_path: String,
    pub project_files: Vec<CorpusSealedProjectFile>,
}

#[derive(Debug, Serialize)]
struct CorpusAssetManifest {
    version: &'static str,
    bundle_id: String,
    hash_algo: &'static str,
    entries: Vec<CorpusAssetManifestEntry>,
}

#[derive(Debug, Serialize)]
struct CorpusAssetManifestEntry {
    path: String,
    handle: String,
    hash: String,
    size: usize,
    mime: String,
}

pub fn build_corpus_columns_json(request_json: &str) -> Result<String, CorpusAuthoringError> {
    let request: CorpusColumnsRequest = serde_json::from_str(request_json).map_err(|err| {
        CorpusAuthoringError::new(
            "E_CORPUS_COLUMNS_REQUEST_JSON",
            "$",
            format!("열 저작 요청 JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let result = build_corpus_columns(request)?;
    serde_json::to_string(&result)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|err| {
            CorpusAuthoringError::new(
                "E_CORPUS_COLUMNS_SERIALIZE",
                "$",
                format!("열 저작 결과를 직렬화할 수 없습니다: {err}"),
            )
        })
}

pub fn build_corpus_columns(
    request: CorpusColumnsRequest,
) -> Result<CorpusColumnsResult, CorpusAuthoringError> {
    if request.transport_kind != CORPUS_COLUMNS_TRANSPORT {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_COLUMNS_TRANSPORT",
            "$.transport_kind",
            format!("transport_kind는 {CORPUS_COLUMNS_TRANSPORT}이어야 합니다"),
        ));
    }
    let source_bytes_sha256 = hex::encode(Sha256::digest(&request.source_bytes));
    if request.expected_source_bytes_sha256 != source_bytes_sha256 {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_COLUMNS_STALE_SOURCE",
            "$.expected_source_bytes_sha256",
            format!(
                "CSV source hash가 Rust intake 결과와 다릅니다: expected {}, actual {source_bytes_sha256}",
                request.expected_source_bytes_sha256
            ),
        ));
    }
    let intake = parse_corpus_csv_bytes(&request.source_bytes, request.csv_options.clone())?;
    if intake.status != CorpusCsvStatus::Ready {
        let first = intake.diagnostics.first();
        return Err(CorpusAuthoringError::new(
            first
                .map(|diagnostic| diagnostic.code)
                .unwrap_or("E_CORPUS_COLUMNS_CSV_REJECTED"),
            "$.source_bytes",
            first
                .map(|diagnostic| diagnostic.message.clone())
                .unwrap_or_else(|| "CSV intake가 ready 상태가 아닙니다".to_string()),
        ));
    }
    if intake.ordered_raw_rows != request.ordered_raw_rows {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_COLUMNS_STALE_INTAKE",
            "$.ordered_raw_rows",
            "열 저작 raw rows가 같은 source bytes를 Rust에서 다시 해석한 결과와 다릅니다",
        ));
    }
    if request.ordered_columns.is_empty() {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_COLUMNS_EMPTY",
            "$.ordered_columns",
            "열은 하나 이상이어야 합니다",
        ));
    }
    let source_width = request
        .ordered_raw_rows
        .first()
        .map(Vec::len)
        .unwrap_or(request.ordered_columns.len());
    let mut names = HashSet::new();
    let mut source_indexes = HashSet::new();
    for (index, column) in request.ordered_columns.iter().enumerate() {
        let path = format!("$.ordered_columns[{index}]");
        if column.name.trim().is_empty() {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_COLUMN_NAME_EMPTY",
                format!("{path}.name"),
                "열 이름은 비어 있을 수 없습니다",
            ));
        }
        if !names.insert(column.name.as_str()) {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_COLUMN_NAME_DUPLICATE",
                format!("{path}.name"),
                format!("열 이름이 중복됩니다: {}", column.name),
            ));
        }
        if column.source_index >= source_width {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_COLUMN_SOURCE_INDEX",
                format!("{path}.source_index"),
                format!(
                    "source_index {}가 CSV 열 수 {source_width} 범위를 벗어납니다",
                    column.source_index
                ),
            ));
        }
        if !source_indexes.insert(column.source_index) {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_COLUMN_SOURCE_DUPLICATE",
                format!("{path}.source_index"),
                format!(
                    "같은 CSV 열을 두 번 사용할 수 없습니다: {}",
                    column.source_index
                ),
            ));
        }
        if column.description.trim().is_empty() {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_COLUMN_DESCRIPTION_EMPTY",
                format!("{path}.description"),
                "열 설명은 비어 있을 수 없습니다",
            ));
        }
        if let Some(unit) = &column.unit {
            if unit.is_empty() || unit.trim() != unit {
                return Err(CorpusAuthoringError::new(
                    "E_CORPUS_COLUMN_UNIT_TEXT",
                    format!("{path}.unit"),
                    "단위는 공백을 보정하지 않고 exact text로 입력해야 합니다",
                ));
            }
            if !matches!(column.dtype, DatasetDtype::Integer | DatasetDtype::Fixed64) {
                return Err(CorpusAuthoringError::new(
                    "E_CORPUS_COLUMN_UNIT_TYPE",
                    format!("{path}.unit"),
                    "typed unit은 integer 또는 fixed64 열에만 지정할 수 있습니다",
                ));
            }
        }
    }
    if request.ordered_columns.len() != source_width {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_COLUMNS_SOURCE_COVERAGE",
            "$.ordered_columns",
            format!(
                "CSV 열 {source_width}개를 모두 한 번씩 설정해야 합니다: 현재 {}개",
                request.ordered_columns.len()
            ),
        ));
    }

    let mut ordered_rows = Vec::with_capacity(request.ordered_raw_rows.len());
    for (row_index, raw_row) in request.ordered_raw_rows.iter().enumerate() {
        if raw_row.len() != source_width {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_COLUMNS_ROW_WIDTH",
                format!("$.ordered_raw_rows[{row_index}]"),
                format!(
                    "raw row cell 수 {}가 CSV 열 수 {source_width}와 다릅니다",
                    raw_row.len()
                ),
            ));
        }
        let mut cells = Vec::with_capacity(request.ordered_columns.len());
        for (column_index, column) in request.ordered_columns.iter().enumerate() {
            let raw = &raw_row[column.source_index];
            cells.push(raw_cell(
                raw,
                column,
                &format!(
                    "$.ordered_raw_rows[{row_index}][{}] -> $.ordered_columns[{column_index}]",
                    column.source_index
                ),
            )?);
        }
        ordered_rows.push(DatasetRow {
            source_ordinal: row_index as u64 + 1,
            cells,
        });
    }

    let ordered_columns: Vec<DatasetColumn> = request
        .ordered_columns
        .iter()
        .map(|column| DatasetColumn {
            name: column.name.clone(),
            dtype: column.dtype,
            unit: column.unit.clone(),
            source_locator: format!("csv:column={}", column.source_index + 1),
        })
        .collect();
    let dataset = seal_domain_dataset(
        SealedDomainDataset {
            schema: SEALED_DOMAIN_DATASET_SCHEMA.to_string(),
            artifact_id: request.artifact_id,
            artifact_version: request.artifact_version,
            source: DatasetSource::ProjectAuthored {
                project_revision_ref: request.project_revision_ref,
                source_ref: request.source_ref,
                author_label: request.author_label,
            },
            payload: DatasetPayload {
                object_ref: request.object_ref,
                media_type: request.media_type,
                byte_length: 0,
                sha256: String::new(),
            },
            ordered_columns,
            ordered_rows,
            content_sha256: String::new(),
            schema_sha256: String::new(),
        },
        &request.source_bytes,
    )?;
    Ok(CorpusColumnsResult {
        transport_kind: CORPUS_COLUMNS_TRANSPORT,
        source_bytes_sha256,
        ordered_columns: request.ordered_columns,
        row_count: dataset.ordered_rows.len(),
        dataset_content_sha256: dataset.content_sha256.clone(),
        canonical_dataset_json: canonical_sealed_domain_dataset_json(&dataset)?,
        target_rows: corpus_target_rows(&dataset),
    })
}

fn corpus_target_rows(dataset: &SealedDomainDataset) -> Vec<CorpusTargetRowProjection> {
    dataset
        .ordered_rows
        .iter()
        .map(|row| CorpusTargetRowProjection {
            row_source_ordinal: row.source_ordinal,
            cells: dataset
                .ordered_columns
                .iter()
                .zip(&row.cells)
                .map(|(column, cell)| CorpusTargetCellProjection {
                    column_name: column.name.clone(),
                    status: match cell {
                        DatasetCell::Present { .. } => CorpusTargetCellStatus::Present,
                        DatasetCell::Gap => CorpusTargetCellStatus::Gap,
                        DatasetCell::Missing {
                            reason: MissingReason::SourceUnavailable,
                        } => CorpusTargetCellStatus::SourceUnavailable,
                        DatasetCell::Missing {
                            reason: MissingReason::RecordDamaged,
                        } => CorpusTargetCellStatus::RecordDamaged,
                        DatasetCell::Missing {
                            reason: MissingReason::CalculationFailed,
                        } => CorpusTargetCellStatus::CalculationFailed,
                        DatasetCell::Pruned => CorpusTargetCellStatus::Pruned,
                    },
                })
                .collect(),
        })
        .collect()
}

pub fn build_corpus_sources_json(request_json: &str) -> Result<String, CorpusAuthoringError> {
    let request: CorpusSourcesRequest = serde_json::from_str(request_json).map_err(|err| {
        CorpusAuthoringError::new(
            "E_CORPUS_SOURCES_REQUEST_JSON",
            "$",
            format!("출처 저작 요청 JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let result = build_corpus_sources(request)?;
    serde_json::to_string(&result)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|err| {
            CorpusAuthoringError::new(
                "E_CORPUS_SOURCES_SERIALIZE",
                "$",
                format!("출처 저작 결과를 직렬화할 수 없습니다: {err}"),
            )
        })
}

pub fn build_corpus_sources(
    request: CorpusSourcesRequest,
) -> Result<CorpusSourcesResult, CorpusAuthoringError> {
    if request.transport_kind != CORPUS_SOURCES_TRANSPORT {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_SOURCES_TRANSPORT",
            "$.transport_kind",
            format!("transport_kind는 {CORPUS_SOURCES_TRANSPORT}이어야 합니다"),
        ));
    }
    let source_bytes_sha256 = hex::encode(Sha256::digest(&request.source_bytes));
    if request.expected_source_bytes_sha256 != source_bytes_sha256 {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_SOURCES_STALE_SOURCE",
            "$.expected_source_bytes_sha256",
            format!(
                "CSV source hash가 열 저작 결과와 다릅니다: expected {}, actual {source_bytes_sha256}",
                request.expected_source_bytes_sha256
            ),
        ));
    }
    let dataset = parse_and_validate_sealed_domain_dataset(
        &request.canonical_dataset_json,
        &request.source_bytes,
    )?;
    if dataset.content_sha256 != request.expected_dataset_content_sha256 {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_SOURCES_STALE_DATASET",
            "$.expected_dataset_content_sha256",
            format!(
                "dataset hash가 열 저작 결과와 다릅니다: expected {}, actual {}",
                request.expected_dataset_content_sha256, dataset.content_sha256
            ),
        ));
    }
    for (index, source) in request.ordered_sources.iter().enumerate() {
        if !source.canonical_identifier.starts_with("https://") {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_SOURCE_HTTPS_REQUIRED",
                format!("$.ordered_sources[{index}].canonical_identifier"),
                "W1 출처 record URI는 canonical HTTPS여야 합니다",
            ));
        }
        if !is_exact_date(&source.retrieved_at) {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_SOURCE_CHECKED_AT",
                format!("$.ordered_sources[{index}].retrieved_at"),
                "checked-at은 exact YYYY-MM-DD 날짜여야 합니다",
            ));
        }
    }
    validate_evidence_sources(&request.ordered_sources)?;
    let encoded = serde_json::to_vec(&request.ordered_sources).map_err(|err| {
        CorpusAuthoringError::new(
            "E_CORPUS_SOURCES_SERIALIZE",
            "$.ordered_sources",
            format!("출처 record를 identity bytes로 만들 수 없습니다: {err}"),
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(CORPUS_SOURCES_HASH_PREFIX);
    hasher.update(dataset.content_sha256.as_bytes());
    hasher.update([0]);
    hasher.update(encoded);
    Ok(CorpusSourcesResult {
        transport_kind: CORPUS_SOURCES_TRANSPORT,
        source_bytes_sha256,
        dataset_content_sha256: dataset.content_sha256,
        source_records_sha256: hex::encode(hasher.finalize()),
        ordered_sources: request.ordered_sources,
    })
}

pub fn build_corpus_evidence_json(request_json: &str) -> Result<String, CorpusAuthoringError> {
    let request: CorpusEvidenceRequest = serde_json::from_str(request_json).map_err(|err| {
        CorpusAuthoringError::new(
            "E_CORPUS_EVIDENCE_REQUEST_JSON",
            "$",
            format!("근거 결합 요청 JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let result = build_corpus_evidence(request)?;
    serde_json::to_string(&result)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|err| {
            CorpusAuthoringError::new(
                "E_CORPUS_EVIDENCE_SERIALIZE",
                "$",
                format!("근거 결합 결과를 직렬화할 수 없습니다: {err}"),
            )
        })
}

pub fn build_corpus_evidence(
    request: CorpusEvidenceRequest,
) -> Result<CorpusEvidenceResult, CorpusAuthoringError> {
    if request.transport_kind != CORPUS_EVIDENCE_TRANSPORT {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_EVIDENCE_TRANSPORT",
            "$.transport_kind",
            format!("transport_kind는 {CORPUS_EVIDENCE_TRANSPORT}이어야 합니다"),
        ));
    }
    for (index, claim) in request.claims.iter().enumerate() {
        if claim
            .note
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
        {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_CLAIM_TEXT_REQUIRED",
                format!("$.claims[{index}].note"),
                "claim text는 기존 evidence claim note에 명시해야 합니다",
            ));
        }
    }
    let source_result = build_corpus_sources(CorpusSourcesRequest {
        transport_kind: CORPUS_SOURCES_TRANSPORT.to_string(),
        source_bytes: request.source_bytes.clone(),
        expected_source_bytes_sha256: request.expected_source_bytes_sha256,
        canonical_dataset_json: request.canonical_dataset_json.clone(),
        expected_dataset_content_sha256: request.expected_dataset_content_sha256,
        ordered_sources: request.ordered_sources.clone(),
    })?;
    if source_result.source_records_sha256 != request.expected_source_records_sha256 {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_EVIDENCE_STALE_SOURCES",
            "$.expected_source_records_sha256",
            format!(
                "source records hash가 출처 단계와 다릅니다: expected {}, actual {}",
                request.expected_source_records_sha256, source_result.source_records_sha256
            ),
        ));
    }
    let dataset = parse_and_validate_sealed_domain_dataset(
        &request.canonical_dataset_json,
        &request.source_bytes,
    )?;
    let dataset_context = evidence_dataset_target_from_sealed_dataset(&dataset)?;
    let dataset_content_sha256 = dataset.content_sha256.clone();
    let target = |draft: CorpusEvidenceTargetDraft| EvidenceTarget {
        dataset_content_sha256: dataset_content_sha256.clone(),
        row_source_ordinal: draft.row_source_ordinal,
        column_name: draft.column_name,
    };
    let manifest = seal_domain_evidence_manifest(
        DomainEvidenceManifest {
            schema: DOMAIN_EVIDENCE_MANIFEST_SCHEMA.to_string(),
            subject: EvidenceSubject {
                dataset_content_sha256: dataset_content_sha256.clone(),
                project_revision_ref: request.project_revision_ref,
            },
            sources: request.ordered_sources,
            claims: request
                .claims
                .into_iter()
                .map(|claim| EvidenceClaim {
                    target: target(claim.target),
                    relation: claim.relation,
                    source_ref: claim.source_ref,
                    source_locator: claim.source_locator,
                    note: claim.note,
                })
                .collect(),
            activities: request
                .activities
                .into_iter()
                .map(|activity| EvidenceActivity {
                    kind: activity.kind,
                    input_source_refs: activity.input_source_refs,
                    output_targets: activity.output_targets.into_iter().map(&target).collect(),
                    recipe_sha256: activity.recipe_sha256,
                })
                .collect(),
            hashes: EvidenceManifestHashes {
                evidence_manifest_sha256: String::new(),
                content_binding_sha256: String::new(),
            },
        },
        &[dataset_context.clone()],
    )?;
    let canonical_evidence_manifest_json =
        canonical_domain_evidence_manifest_json(&manifest, &[dataset_context])?;
    Ok(CorpusEvidenceResult {
        transport_kind: CORPUS_EVIDENCE_TRANSPORT,
        source_bytes_sha256: source_result.source_bytes_sha256,
        dataset_content_sha256,
        source_records_sha256: source_result.source_records_sha256,
        evidence_manifest_sha256: manifest.hashes.evidence_manifest_sha256,
        content_binding_sha256: manifest.hashes.content_binding_sha256,
        claim_count: manifest.claims.len(),
        activity_count: manifest.activities.len(),
        canonical_evidence_manifest_json,
    })
}

pub fn build_corpus_review_json(request_json: &str) -> Result<String, CorpusAuthoringError> {
    let request: CorpusReviewRequest = serde_json::from_str(request_json).map_err(|err| {
        CorpusAuthoringError::new(
            "E_CORPUS_REVIEW_REQUEST_JSON",
            "$",
            format!("검토 요청 JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let result = build_corpus_review(request)?;
    serde_json::to_string(&result)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|err| {
            CorpusAuthoringError::new(
                "E_CORPUS_REVIEW_SERIALIZE",
                "$",
                format!("검토 결과를 직렬화할 수 없습니다: {err}"),
            )
        })
}

pub fn build_corpus_review(
    request: CorpusReviewRequest,
) -> Result<CorpusReviewResult, CorpusAuthoringError> {
    if request.transport_kind != CORPUS_REVIEW_TRANSPORT {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_REVIEW_TRANSPORT",
            "$.transport_kind",
            format!("transport_kind는 {CORPUS_REVIEW_TRANSPORT}이어야 합니다"),
        ));
    }
    let source_bytes_sha256 = hex::encode(Sha256::digest(&request.source_bytes));
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    match parse_corpus_csv_bytes(&request.source_bytes, request.csv_options.clone()) {
        Ok(intake) => {
            for diagnostic in intake.diagnostics {
                let issue = CorpusReviewIssue {
                    severity: match diagnostic.severity {
                        CorpusCsvDiagnosticSeverity::Error => CorpusReviewSeverity::Error,
                        CorpusCsvDiagnosticSeverity::Warning => CorpusReviewSeverity::Warning,
                    },
                    step: "file".to_string(),
                    code: diagnostic.code.to_string(),
                    path: format!(
                        "byte {}:{}",
                        diagnostic.source_span.byte_start, diagnostic.source_span.byte_end
                    ),
                    message: diagnostic.message,
                };
                match issue.severity {
                    CorpusReviewSeverity::Error => errors.push(issue),
                    CorpusReviewSeverity::Warning => warnings.push(issue),
                }
            }
        }
        Err(error) => errors.push(review_issue(&error)),
    }

    let dataset = parse_and_validate_sealed_domain_dataset(
        &request.canonical_dataset_json,
        &request.source_bytes,
    );
    let (dataset_content_sha256, row_count, column_count, units, sparse) = match dataset {
        Ok(dataset) => {
            let mut sparse = CorpusSparseSummary::default();
            for row in &dataset.ordered_rows {
                for cell in &row.cells {
                    match cell {
                        DatasetCell::Present { .. } => {}
                        DatasetCell::Gap => sparse.gap += 1,
                        DatasetCell::Missing {
                            reason: MissingReason::SourceUnavailable,
                        } => sparse.source_unavailable += 1,
                        DatasetCell::Missing {
                            reason: MissingReason::RecordDamaged,
                        } => sparse.record_damaged += 1,
                        DatasetCell::Missing {
                            reason: MissingReason::CalculationFailed,
                        } => sparse.calculation_failed += 1,
                        DatasetCell::Pruned => sparse.pruned += 1,
                    }
                }
            }
            (
                dataset.content_sha256,
                dataset.ordered_rows.len(),
                dataset.ordered_columns.len(),
                dataset
                    .ordered_columns
                    .into_iter()
                    .map(|column| CorpusReviewUnit {
                        column_name: column.name,
                        exact_unit: column.unit,
                    })
                    .collect(),
                sparse,
            )
        }
        Err(error) => {
            errors.push(review_issue(&CorpusAuthoringError::from(error)));
            (
                String::new(),
                0,
                0,
                Vec::new(),
                CorpusSparseSummary::default(),
            )
        }
    };

    let evidence_request = CorpusEvidenceRequest {
        transport_kind: CORPUS_EVIDENCE_TRANSPORT.to_string(),
        source_bytes: request.source_bytes,
        expected_source_bytes_sha256: request.expected_source_bytes_sha256,
        canonical_dataset_json: request.canonical_dataset_json,
        expected_dataset_content_sha256: request.expected_dataset_content_sha256,
        ordered_sources: request.ordered_sources,
        expected_source_records_sha256: request.expected_source_records_sha256,
        project_revision_ref: request.project_revision_ref,
        claims: request.claims,
        activities: request.activities,
    };
    let source_count = evidence_request.ordered_sources.len();
    let claim_count = evidence_request.claims.len();
    let activity_count = evidence_request.activities.len();
    let evidence_manifest_sha256 = match build_corpus_evidence(evidence_request) {
        Ok(evidence) => {
            if evidence.evidence_manifest_sha256 != request.expected_evidence_manifest_sha256 {
                errors.push(CorpusReviewIssue {
                    severity: CorpusReviewSeverity::Error,
                    step: "evidence".to_string(),
                    code: "E_CORPUS_REVIEW_STALE_EVIDENCE".to_string(),
                    path: "$.expected_evidence_manifest_sha256".to_string(),
                    message: format!(
                        "evidence manifest hash가 근거 단계와 다릅니다: expected {}, actual {}",
                        request.expected_evidence_manifest_sha256,
                        evidence.evidence_manifest_sha256
                    ),
                });
            }
            evidence.evidence_manifest_sha256
        }
        Err(error) => {
            errors.push(review_issue(&error));
            String::new()
        }
    };
    Ok(CorpusReviewResult {
        transport_kind: CORPUS_REVIEW_TRANSPORT,
        ready_to_seal: errors.is_empty(),
        source_bytes_sha256,
        dataset_content_sha256,
        evidence_manifest_sha256,
        row_count,
        column_count,
        units,
        source_count,
        claim_count,
        activity_count,
        sparse,
        errors,
        warnings,
    })
}

pub fn build_corpus_seal_json(request_json: &str) -> Result<String, CorpusAuthoringError> {
    let request: CorpusSealRequest = serde_json::from_str(request_json).map_err(|err| {
        CorpusAuthoringError::new(
            "E_CORPUS_SEAL_REQUEST_JSON",
            "$",
            format!("봉인 요청 JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let result = build_corpus_seal(request)?;
    serde_json::to_string(&result)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|err| {
            CorpusAuthoringError::new(
                "E_CORPUS_SEAL_SERIALIZE",
                "$",
                format!("봉인 결과를 직렬화할 수 없습니다: {err}"),
            )
        })
}

pub fn build_corpus_seal(
    request: CorpusSealRequest,
) -> Result<CorpusSealResult, CorpusAuthoringError> {
    if request.transport_kind != CORPUS_SEAL_TRANSPORT {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_SEAL_TRANSPORT",
            "$.transport_kind",
            format!("transport_kind는 {CORPUS_SEAL_TRANSPORT}이어야 합니다"),
        ));
    }
    let source_bytes_sha256 = hex::encode(Sha256::digest(&request.source_bytes));
    if request.expected_source_bytes_sha256 != source_bytes_sha256 {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_SEAL_STALE_SOURCE",
            "$.expected_source_bytes_sha256",
            format!(
                "봉인할 source bytes hash가 검토 결과와 다릅니다: expected {}, actual {}",
                request.expected_source_bytes_sha256, source_bytes_sha256
            ),
        ));
    }
    let payload_text = std::str::from_utf8(&request.source_bytes).map_err(|_| {
        CorpusAuthoringError::new(
            "E_CORPUS_SEAL_PAYLOAD_UTF8",
            "$.source_bytes",
            "W1 project file payload는 검증된 UTF-8 CSV bytes여야 합니다",
        )
    })?;
    let canonical = parse_and_canonicalize_corpus_artifacts(
        &request.canonical_dataset_json,
        &request.source_bytes,
        &request.canonical_evidence_manifest_json,
    )?;
    if canonical.dataset_json != request.canonical_dataset_json {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_SEAL_DATASET_NOT_CANONICAL",
            "$.canonical_dataset_json",
            "dataset envelope bytes가 Rust canonical bytes와 다릅니다",
        ));
    }
    if canonical.evidence_manifest_json != request.canonical_evidence_manifest_json {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_SEAL_EVIDENCE_NOT_CANONICAL",
            "$.canonical_evidence_manifest_json",
            "evidence manifest bytes가 Rust canonical bytes와 다릅니다",
        ));
    }
    require_seal_hash(
        "E_CORPUS_SEAL_DATASET_STALE",
        "$.expected_dataset_content_sha256",
        &request.expected_dataset_content_sha256,
        &canonical.dataset_content_sha256,
    )?;
    require_seal_hash(
        "E_CORPUS_SEAL_EVIDENCE_STALE",
        "$.expected_evidence_manifest_sha256",
        &request.expected_evidence_manifest_sha256,
        &canonical.evidence_manifest_sha256,
    )?;
    require_seal_hash(
        "E_CORPUS_SEAL_BINDING_STALE",
        "$.expected_content_binding_sha256",
        &request.expected_content_binding_sha256,
        &canonical.content_binding_sha256,
    )?;
    let dataset =
        parse_and_validate_sealed_domain_dataset(&canonical.dataset_json, &request.source_bytes)?;
    match &dataset.source {
        DatasetSource::ProjectAuthored {
            project_revision_ref,
            ..
        } if project_revision_ref == &request.expected_project_revision_ref => {}
        DatasetSource::ProjectAuthored {
            project_revision_ref,
            ..
        } => {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_SEAL_PROJECT_REVISION_STALE",
                "$.expected_project_revision_ref",
                format!(
                    "dataset 저작 revision이 현재 봉인 대상과 다릅니다: expected {}, actual {}",
                    request.expected_project_revision_ref, project_revision_ref
                ),
            ));
        }
        DatasetSource::ExternalSnapshot { .. } => {
            return Err(CorpusAuthoringError::new(
                "E_CORPUS_SEAL_SOURCE_KIND",
                "$.canonical_dataset_json.source.kind",
                "Workbench corpus 저작 봉인은 project_authored dataset만 받습니다",
            ));
        }
    }

    let dataset_envelope_path = format!(
        "datasets/corpus/{}.envelope.json",
        canonical.dataset_content_sha256
    );
    let dataset_payload_path = dataset.payload.object_ref.clone();
    let evidence_manifest_path = format!(
        "evidence/corpus/{}.manifest.json",
        canonical.evidence_manifest_sha256
    );
    let bundle_id = format!("corpus:{}", canonical.dataset_content_sha256);
    let asset_material = [
        (
            dataset_envelope_path.clone(),
            canonical.dataset_json.as_bytes().to_vec(),
            "application/vnd.ddn.sealed-domain-dataset+json".to_string(),
        ),
        (
            evidence_manifest_path.clone(),
            canonical.evidence_manifest_json.as_bytes().to_vec(),
            "application/vnd.ddn.domain-evidence-manifest+json".to_string(),
        ),
        (
            dataset_payload_path.clone(),
            request.source_bytes.clone(),
            dataset.payload.media_type.clone(),
        ),
    ];
    let mut entries = asset_material
        .iter()
        .map(|(path, bytes, mime)| CorpusAssetManifestEntry {
            path: path.clone(),
            handle: asset_handle_from_bundle_path(&bundle_id, path).to_hex(),
            hash: blake3_hash(bytes).to_hex().to_string(),
            size: bytes.len(),
            mime: mime.clone(),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let asset_manifest_json = serde_json::to_string(&CorpusAssetManifest {
        version: "v0",
        bundle_id,
        hash_algo: "blake3",
        entries,
    })
    .map_err(|err| {
        CorpusAuthoringError::new(
            "E_CORPUS_SEAL_ASSET_MANIFEST",
            "$.asset_manifest",
            format!("asset manifest를 만들 수 없습니다: {err}"),
        )
    })?;
    let registry_files = asset_material
        .iter()
        .map(|(path, bytes, _)| (path.clone(), bytes.clone()))
        .collect::<HashMap<_, _>>();
    let registry = build_project_asset_registry_from_manifest(
        asset_manifest_json.as_bytes(),
        &registry_files,
        true,
    )
    .map_err(|err| {
        CorpusAuthoringError::new(
            "E_CORPUS_SEAL_ASSET_REGISTRY",
            "$.asset_manifest",
            format!("생성한 project asset registry가 검증되지 않았습니다: {err}"),
        )
    })?;
    let dataset_handle = registry
        .resolve_handle(&dataset_envelope_path)
        .map_err(|err| {
            CorpusAuthoringError::new(
                "E_CORPUS_SEAL_DATASET_HANDLE",
                "$.asset_manifest.entries",
                err,
            )
        })?
        .to_hex();
    registry
        .open_sealed_dataset(asset_handle_from_bundle_path(
            registry.bundle_id(),
            &dataset_envelope_path,
        ))
        .map_err(|err| {
            CorpusAuthoringError::new(
                "E_CORPUS_SEAL_DATASET_OPEN",
                "$.asset_manifest.entries",
                format!("생성한 자원핸들로 dataset을 다시 열 수 없습니다: {err}"),
            )
        })?;

    let project_files = vec![
        sealed_project_file("ddn.asset.json", "asset_manifest", asset_manifest_json),
        sealed_project_file(
            &dataset_envelope_path,
            "sealed_dataset_envelope",
            canonical.dataset_json,
        ),
        sealed_project_file(
            &evidence_manifest_path,
            "domain_evidence_manifest",
            canonical.evidence_manifest_json,
        ),
        sealed_project_file(
            &dataset_payload_path,
            "dataset_payload",
            payload_text.to_string(),
        ),
    ];
    Ok(CorpusSealResult {
        transport_kind: CORPUS_SEAL_TRANSPORT,
        source_bytes_sha256,
        dataset_content_sha256: canonical.dataset_content_sha256,
        evidence_manifest_sha256: canonical.evidence_manifest_sha256,
        content_binding_sha256: canonical.content_binding_sha256,
        asset_manifest_sha256: hex::encode(Sha256::digest(project_files[0].utf8_text.as_bytes())),
        dataset_handle,
        dataset_envelope_path,
        dataset_payload_path,
        evidence_manifest_path,
        project_files,
    })
}

fn require_seal_hash(
    code: &str,
    path: &str,
    expected: &str,
    actual: &str,
) -> Result<(), CorpusAuthoringError> {
    if expected == actual {
        return Ok(());
    }
    Err(CorpusAuthoringError::new(
        code,
        path,
        format!("봉인 identity가 검토 결과와 다릅니다: expected {expected}, actual {actual}"),
    ))
}

fn sealed_project_file(path: &str, role: &str, utf8_text: String) -> CorpusSealedProjectFile {
    CorpusSealedProjectFile {
        path: path.to_string(),
        role: role.to_string(),
        byte_length: utf8_text.len(),
        sha256: hex::encode(Sha256::digest(utf8_text.as_bytes())),
        utf8_text,
    }
}

fn review_issue(error: &CorpusAuthoringError) -> CorpusReviewIssue {
    let step = if error.code.contains("CSV") {
        "file"
    } else if error.code.contains("COLUMN") || error.code.contains("DATASET") {
        "columns"
    } else if error.code.contains("SOURCE") || error.path.contains("sources") {
        "source"
    } else {
        "evidence"
    };
    CorpusReviewIssue {
        severity: CorpusReviewSeverity::Error,
        step: step.to_string(),
        code: error.code.clone(),
        path: error.path.clone(),
        message: error.message.clone(),
    }
}

fn is_exact_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return false;
    }
    let year = value[0..4].parse::<u16>().ok();
    let month = value[5..7].parse::<u8>().ok();
    let day = value[8..10].parse::<u8>().ok();
    matches!(year, Some(1..=9999)) && matches!(month, Some(1..=12)) && matches!(day, Some(1..=31))
}

fn raw_cell(
    raw: &str,
    column: &CorpusColumnDraft,
    path: &str,
) -> Result<DatasetCell, CorpusAuthoringError> {
    if raw.is_empty() {
        return Ok(match column.missing_policy {
            CorpusMissingPolicy::Gap => DatasetCell::Gap,
            CorpusMissingPolicy::SourceUnavailable => DatasetCell::Missing {
                reason: MissingReason::SourceUnavailable,
            },
            CorpusMissingPolicy::RecordDamaged => DatasetCell::Missing {
                reason: MissingReason::RecordDamaged,
            },
            CorpusMissingPolicy::Pruned => DatasetCell::Pruned,
        });
    }
    let value = match column.dtype {
        DatasetDtype::Integer => {
            let parsed = raw.parse::<i64>().map_err(|_| {
                CorpusAuthoringError::new(
                    "E_CORPUS_COLUMN_INTEGER",
                    path,
                    format!("integer 열의 raw text를 exact i64로 읽을 수 없습니다: {raw}"),
                )
            })?;
            if parsed.to_string() != raw {
                return Err(CorpusAuthoringError::new(
                    "E_CORPUS_COLUMN_INTEGER_CANON",
                    path,
                    format!("integer raw text를 묵시적으로 보정하지 않습니다: {raw}"),
                ));
            }
            DatasetScalar::Integer { value: parsed }
        }
        DatasetDtype::Fixed64 => {
            if Fixed64::parse_decimal_exact(raw).is_none() {
                return Err(CorpusAuthoringError::new(
                    "E_CORPUS_COLUMN_FIXED64",
                    path,
                    format!("fixed64 열의 raw text가 exact 범위/표기가 아닙니다: {raw}"),
                ));
            }
            DatasetScalar::Fixed64 {
                value: raw.to_string(),
            }
        }
        DatasetDtype::String => DatasetScalar::String {
            value: raw.to_string(),
        },
        DatasetDtype::Boolean => {
            let value = match raw {
                "true" => true,
                "false" => false,
                _ => {
                    return Err(CorpusAuthoringError::new(
                        "E_CORPUS_COLUMN_BOOLEAN",
                        path,
                        format!("boolean은 exact text true 또는 false여야 합니다: {raw}"),
                    ));
                }
            };
            DatasetScalar::Boolean { value }
        }
    };
    Ok(DatasetCell::Present { value })
}

pub fn seal_corpus_artifacts(
    dataset_draft: SealedDomainDataset,
    payload_bytes: &[u8],
    evidence_draft: CorpusEvidenceDraft,
) -> Result<SealedCorpusArtifacts, CorpusAuthoringError> {
    let dataset = seal_domain_dataset(dataset_draft, payload_bytes)?;
    let dataset_context = evidence_dataset_target_from_sealed_dataset(&dataset)?;
    let dataset_content_sha256 = dataset.content_sha256.clone();
    let target = |draft: CorpusEvidenceTargetDraft| EvidenceTarget {
        dataset_content_sha256: dataset_content_sha256.clone(),
        row_source_ordinal: draft.row_source_ordinal,
        column_name: draft.column_name,
    };
    let manifest = DomainEvidenceManifest {
        schema: DOMAIN_EVIDENCE_MANIFEST_SCHEMA.to_string(),
        subject: EvidenceSubject {
            dataset_content_sha256: dataset_content_sha256.clone(),
            project_revision_ref: evidence_draft.project_revision_ref,
        },
        sources: evidence_draft.sources,
        claims: evidence_draft
            .claims
            .into_iter()
            .map(|claim| EvidenceClaim {
                target: target(claim.target),
                relation: claim.relation,
                source_ref: claim.source_ref,
                source_locator: claim.source_locator,
                note: claim.note,
            })
            .collect(),
        activities: evidence_draft
            .activities
            .into_iter()
            .map(|activity| EvidenceActivity {
                kind: activity.kind,
                input_source_refs: activity.input_source_refs,
                output_targets: activity.output_targets.into_iter().map(&target).collect(),
                recipe_sha256: activity.recipe_sha256,
            })
            .collect(),
        hashes: EvidenceManifestHashes {
            evidence_manifest_sha256: String::new(),
            content_binding_sha256: String::new(),
        },
    };
    let evidence_manifest = seal_domain_evidence_manifest(manifest, &[dataset_context])?;
    Ok(SealedCorpusArtifacts {
        dataset,
        evidence_manifest,
    })
}

pub fn validate_corpus_artifacts(
    dataset: &SealedDomainDataset,
    payload_bytes: &[u8],
    evidence_manifest: &DomainEvidenceManifest,
) -> Result<(), CorpusAuthoringError> {
    validate_sealed_domain_dataset(dataset, payload_bytes)?;
    let dataset_context = evidence_dataset_target_from_sealed_dataset(dataset)?;
    validate_domain_evidence_manifest(evidence_manifest, &[dataset_context])?;
    if evidence_manifest.subject.dataset_content_sha256 != dataset.content_sha256 {
        return Err(CorpusAuthoringError::new(
            "E_CORPUS_EVIDENCE_DATASET_MISMATCH",
            "$.evidence_manifest.subject.dataset_content_sha256",
            "evidence manifest가 같은 bundle의 dataset content hash를 가리키지 않습니다",
        ));
    }
    Ok(())
}

pub fn canonical_corpus_artifacts(
    artifacts: &SealedCorpusArtifacts,
    payload_bytes: &[u8],
) -> Result<CanonicalCorpusArtifacts, CorpusAuthoringError> {
    validate_corpus_artifacts(
        &artifacts.dataset,
        payload_bytes,
        &artifacts.evidence_manifest,
    )?;
    let dataset_context = evidence_dataset_target_from_sealed_dataset(&artifacts.dataset)?;
    Ok(CanonicalCorpusArtifacts {
        dataset_json: canonical_sealed_domain_dataset_json(&artifacts.dataset)?,
        evidence_manifest_json: canonical_domain_evidence_manifest_json(
            &artifacts.evidence_manifest,
            &[dataset_context],
        )?,
        dataset_content_sha256: artifacts.dataset.content_sha256.clone(),
        evidence_manifest_sha256: artifacts
            .evidence_manifest
            .hashes
            .evidence_manifest_sha256
            .clone(),
        content_binding_sha256: artifacts
            .evidence_manifest
            .hashes
            .content_binding_sha256
            .clone(),
    })
}

pub fn parse_and_canonicalize_corpus_artifacts(
    dataset_json: &str,
    payload_bytes: &[u8],
    evidence_manifest_json: &str,
) -> Result<CanonicalCorpusArtifacts, CorpusAuthoringError> {
    let dataset = parse_and_validate_sealed_domain_dataset(dataset_json, payload_bytes)?;
    let dataset_context = evidence_dataset_target_from_sealed_dataset(&dataset)?;
    let evidence_manifest =
        parse_and_validate_domain_evidence_manifest(evidence_manifest_json, &[dataset_context])?;
    canonical_corpus_artifacts(
        &SealedCorpusArtifacts {
            dataset,
            evidence_manifest,
        },
        payload_bytes,
    )
}

pub fn evidence_dataset_target_from_sealed_dataset(
    dataset: &SealedDomainDataset,
) -> Result<EvidenceDatasetTarget, CorpusAuthoringError> {
    let project_revision_ref = match &dataset.source {
        DatasetSource::ProjectAuthored {
            project_revision_ref,
            ..
        } => Some(project_revision_ref.clone()),
        DatasetSource::ExternalSnapshot { .. } => None,
    };
    let column_names: Vec<String> = dataset
        .ordered_columns
        .iter()
        .map(|column| column.name.clone())
        .collect();
    let rows = dataset
        .ordered_rows
        .iter()
        .map(|row| EvidenceDatasetRowTarget {
            source_ordinal: row.source_ordinal,
            cell_statuses: row
                .cells
                .iter()
                .map(|cell| match cell {
                    DatasetCell::Present { .. } => EvidenceCellStatus::Present,
                    DatasetCell::Gap => EvidenceCellStatus::Gap,
                    DatasetCell::Missing { reason } => EvidenceCellStatus::Missing {
                        reason: serde_json::to_value(reason)
                            .ok()
                            .and_then(|value| value.as_str().map(ToOwned::to_owned))
                            .unwrap_or_else(|| "#기록손상".to_string()),
                    },
                    DatasetCell::Pruned => EvidenceCellStatus::Pruned,
                })
                .collect(),
        })
        .collect();
    Ok(EvidenceDatasetTarget {
        dataset_content_sha256: dataset.content_sha256.clone(),
        project_revision_ref,
        column_names,
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_evidence_manifest::{EvidencePayloadMode, EvidenceRelation, EvidenceRights};
    use crate::sealed_domain_dataset::{
        DatasetCell, DatasetColumn, DatasetDtype, DatasetPayload, DatasetRow, DatasetScalar,
        DatasetSource, SealedDomainDataset, SEALED_DOMAIN_DATASET_SCHEMA,
    };

    fn dataset_draft() -> SealedDomainDataset {
        SealedDomainDataset {
            schema: SEALED_DOMAIN_DATASET_SCHEMA.to_string(),
            artifact_id: "corpus:g1-roundtrip".to_string(),
            artifact_version: "1".to_string(),
            source: DatasetSource::ProjectAuthored {
                project_revision_ref: "revision:g1".to_string(),
                source_ref: "source:g1-csv".to_string(),
                author_label: "G1 창작자".to_string(),
            },
            payload: DatasetPayload {
                object_ref: "objects/g1.csv".to_string(),
                media_type: "text/csv; charset=utf-8".to_string(),
                byte_length: 0,
                sha256: String::new(),
            },
            ordered_columns: vec![
                DatasetColumn {
                    name: "이름".to_string(),
                    dtype: DatasetDtype::String,
                    unit: None,
                    source_locator: "csv:column=1".to_string(),
                },
                DatasetColumn {
                    name: "값".to_string(),
                    dtype: DatasetDtype::Fixed64,
                    unit: Some("mol".to_string()),
                    source_locator: "csv:column=2".to_string(),
                },
            ],
            ordered_rows: vec![DatasetRow {
                source_ordinal: 1,
                cells: vec![
                    DatasetCell::Present {
                        value: DatasetScalar::String {
                            value: "물".to_string(),
                        },
                    },
                    DatasetCell::Present {
                        value: DatasetScalar::Fixed64 {
                            value: "2.0".to_string(),
                        },
                    },
                ],
            }],
            content_sha256: String::new(),
            schema_sha256: String::new(),
        }
    }

    fn evidence_draft() -> CorpusEvidenceDraft {
        CorpusEvidenceDraft {
            project_revision_ref: "revision:g1".to_string(),
            sources: vec![EvidenceSource {
                source_ref: "source:g1-csv".to_string(),
                title: "G1 CSV".to_string(),
                provider: "G1 제공자".to_string(),
                canonical_identifier: "https://example.org/g1/source".to_string(),
                published_or_version: "v1".to_string(),
                retrieved_at: "2026-07-28".to_string(),
                rights: EvidenceRights {
                    license_id: "CC-BY-4.0".to_string(),
                    holder: "G1 권리자".to_string(),
                    reuse_mode: "attribution".to_string(),
                },
                payload_mode: EvidencePayloadMode::Reference,
                payload_sha256: None,
            }],
            claims: vec![CorpusEvidenceClaimDraft {
                target: CorpusEvidenceTargetDraft {
                    row_source_ordinal: 1,
                    column_name: Some("값".to_string()),
                },
                relation: EvidenceRelation::DirectSource,
                source_ref: "source:g1-csv".to_string(),
                source_locator: "csv:row=2;column=2".to_string(),
                note: None,
            }],
            activities: vec![],
        }
    }

    fn columns_request() -> CorpusColumnsRequest {
        let source_bytes = b"name,amount\nwater,2.0\n".to_vec();
        CorpusColumnsRequest {
            transport_kind: CORPUS_COLUMNS_TRANSPORT.to_string(),
            expected_source_bytes_sha256: hex::encode(Sha256::digest(&source_bytes)),
            source_bytes,
            csv_options: CorpusCsvOptions {
                encoding: "utf-8".to_string(),
                delimiter: ",".to_string(),
                header: true,
                quote_rule: crate::corpus_csv::CorpusCsvQuoteRule::DoubleQuote,
                newline_policy: crate::corpus_csv::CorpusCsvNewlinePolicy::Consistent,
            },
            artifact_id: "corpus:g3-columns".to_string(),
            artifact_version: "1".to_string(),
            project_revision_ref: "revision:g3".to_string(),
            source_ref: "source:g3-csv".to_string(),
            author_label: "G3 창작자".to_string(),
            object_ref: "objects/g3.csv".to_string(),
            media_type: "text/csv; charset=utf-8".to_string(),
            ordered_columns: vec![
                CorpusColumnDraft {
                    source_index: 0,
                    name: "이름".to_string(),
                    dtype: DatasetDtype::String,
                    unit: None,
                    missing_policy: CorpusMissingPolicy::Gap,
                    description: "자료 행의 이름".to_string(),
                },
                CorpusColumnDraft {
                    source_index: 1,
                    name: "물질량".to_string(),
                    dtype: DatasetDtype::Fixed64,
                    unit: Some("mol".to_string()),
                    missing_policy: CorpusMissingPolicy::SourceUnavailable,
                    description: "자료에 명시된 exact mol 값".to_string(),
                },
            ],
            ordered_raw_rows: vec![vec!["water".to_string(), "2.0".to_string()]],
        }
    }

    fn sources_request() -> CorpusSourcesRequest {
        let columns_request = columns_request();
        let source_bytes = columns_request.source_bytes.clone();
        let columns = build_corpus_columns(columns_request).expect("columns");
        CorpusSourcesRequest {
            transport_kind: CORPUS_SOURCES_TRANSPORT.to_string(),
            expected_source_bytes_sha256: hex::encode(Sha256::digest(&source_bytes)),
            source_bytes,
            canonical_dataset_json: columns.canonical_dataset_json,
            expected_dataset_content_sha256: columns.dataset_content_sha256,
            ordered_sources: vec![
                EvidenceSource {
                    source_ref: "record:water-table".to_string(),
                    title: "물질량 표".to_string(),
                    provider: "공개 자료 제공자".to_string(),
                    canonical_identifier: "https://example.org/records/water-table".to_string(),
                    published_or_version: "2026-1".to_string(),
                    retrieved_at: "2026-07-28".to_string(),
                    rights: EvidenceRights {
                        license_id: "CC-BY-4.0".to_string(),
                        holder: "공개 자료 제공자".to_string(),
                        reuse_mode: "attribution".to_string(),
                    },
                    payload_mode: EvidencePayloadMode::Reference,
                    payload_sha256: None,
                },
                EvidenceSource {
                    source_ref: "record:method-note".to_string(),
                    title: "측정 방법 기록".to_string(),
                    provider: "연구 기록소".to_string(),
                    canonical_identifier: "https://example.net/records/method-note".to_string(),
                    published_or_version: "v1".to_string(),
                    retrieved_at: "2026-07-27".to_string(),
                    rights: EvidenceRights {
                        license_id: "CC0-1.0".to_string(),
                        holder: "연구 기록소".to_string(),
                        reuse_mode: "public_domain".to_string(),
                    },
                    payload_mode: EvidencePayloadMode::Reference,
                    payload_sha256: None,
                },
            ],
        }
    }

    fn evidence_request() -> CorpusEvidenceRequest {
        let source_request = sources_request();
        let source_result = build_corpus_sources(source_request.clone()).expect("sources");
        CorpusEvidenceRequest {
            transport_kind: CORPUS_EVIDENCE_TRANSPORT.to_string(),
            source_bytes: source_request.source_bytes,
            expected_source_bytes_sha256: source_request.expected_source_bytes_sha256,
            canonical_dataset_json: source_request.canonical_dataset_json,
            expected_dataset_content_sha256: source_request.expected_dataset_content_sha256,
            ordered_sources: source_request.ordered_sources,
            expected_source_records_sha256: source_result.source_records_sha256,
            project_revision_ref: "revision:g3".to_string(),
            claims: vec![CorpusEvidenceClaimDraft {
                target: CorpusEvidenceTargetDraft {
                    row_source_ordinal: 1,
                    column_name: Some("물질량".to_string()),
                },
                relation: EvidenceRelation::DirectSource,
                source_ref: "record:water-table".to_string(),
                source_locator: "csv:row=2;column=2".to_string(),
                note: Some("물의 물질량은 원본 표의 2.0 mol 값이다.".to_string()),
            }],
            activities: vec![],
        }
    }

    fn review_request() -> CorpusReviewRequest {
        let evidence_request = evidence_request();
        let evidence = build_corpus_evidence(evidence_request.clone()).expect("evidence");
        CorpusReviewRequest {
            transport_kind: CORPUS_REVIEW_TRANSPORT.to_string(),
            source_bytes: evidence_request.source_bytes,
            csv_options: columns_request().csv_options,
            expected_source_bytes_sha256: evidence_request.expected_source_bytes_sha256,
            canonical_dataset_json: evidence_request.canonical_dataset_json,
            expected_dataset_content_sha256: evidence_request.expected_dataset_content_sha256,
            ordered_sources: evidence_request.ordered_sources,
            expected_source_records_sha256: evidence_request.expected_source_records_sha256,
            project_revision_ref: evidence_request.project_revision_ref,
            claims: evidence_request.claims,
            activities: evidence_request.activities,
            expected_evidence_manifest_sha256: evidence.evidence_manifest_sha256,
        }
    }

    fn seal_request() -> CorpusSealRequest {
        let evidence_request = evidence_request();
        let evidence = build_corpus_evidence(evidence_request.clone()).expect("evidence");
        CorpusSealRequest {
            transport_kind: CORPUS_SEAL_TRANSPORT.to_string(),
            source_bytes: evidence_request.source_bytes,
            expected_source_bytes_sha256: evidence_request.expected_source_bytes_sha256,
            canonical_dataset_json: evidence_request.canonical_dataset_json,
            expected_dataset_content_sha256: evidence_request.expected_dataset_content_sha256,
            canonical_evidence_manifest_json: evidence.canonical_evidence_manifest_json,
            expected_evidence_manifest_sha256: evidence.evidence_manifest_sha256,
            expected_content_binding_sha256: evidence.content_binding_sha256,
            expected_project_revision_ref: evidence_request.project_revision_ref,
        }
    }

    #[test]
    fn existing_canonical_artifacts_roundtrip_with_identical_hashes() {
        let payload = b"\xEC\x9D\xB4\xEB\xA6\x84,\xEA\xB0\x92\n\xEB\xAC\xBC,2.0\n";
        let sealed =
            seal_corpus_artifacts(dataset_draft(), payload, evidence_draft()).expect("seal");
        let first = canonical_corpus_artifacts(&sealed, payload).expect("canonical first");
        let roundtrip = parse_and_canonicalize_corpus_artifacts(
            &first.dataset_json,
            payload,
            &first.evidence_manifest_json,
        )
        .expect("canonical roundtrip");
        assert_eq!(roundtrip, first);
    }

    #[test]
    fn builder_rejects_cross_revision_subject_before_sealing() {
        let payload = b"name,value\nwater,2.0\n";
        let mut evidence = evidence_draft();
        evidence.project_revision_ref = "revision:other".to_string();
        let error = seal_corpus_artifacts(dataset_draft(), payload, evidence).unwrap_err();
        assert_eq!(error.code, "E_EVIDENCE_CROSS_RUN_SUBJECT");
    }

    #[test]
    fn ordered_columns_change_hash_and_restoring_order_restores_hash() {
        let original_request = columns_request();
        let original = build_corpus_columns(original_request.clone()).expect("original");
        let mut reordered_request = original_request.clone();
        reordered_request.ordered_columns.swap(0, 1);
        let reordered = build_corpus_columns(reordered_request).expect("reordered");
        assert_ne!(
            reordered.dataset_content_sha256,
            original.dataset_content_sha256
        );
        let restored = build_corpus_columns(original_request).expect("restored");
        assert_eq!(
            restored.dataset_content_sha256,
            original.dataset_content_sha256
        );
        assert_eq!(
            restored.canonical_dataset_json,
            original.canonical_dataset_json
        );
    }

    #[test]
    fn column_builder_preserves_exact_unit_and_rejects_trim_duplicate_name_and_stale_source() {
        let mut exact_text = columns_request();
        exact_text.ordered_columns[1].unit = Some("MOL".to_string());
        assert_eq!(
            build_corpus_columns(exact_text)
                .expect("exact unit text")
                .ordered_columns[1]
                .unit,
            Some("MOL".to_string())
        );

        let mut padded = columns_request();
        padded.ordered_columns[1].unit = Some(" MOL".to_string());
        assert_eq!(
            build_corpus_columns(padded).unwrap_err().code,
            "E_CORPUS_COLUMN_UNIT_TEXT"
        );

        let mut duplicate = columns_request();
        duplicate.ordered_columns[1].name = "이름".to_string();
        assert_eq!(
            build_corpus_columns(duplicate).unwrap_err().code,
            "E_CORPUS_COLUMN_NAME_DUPLICATE"
        );

        let mut stale = columns_request();
        stale.source_bytes.push(b'!');
        assert_eq!(
            build_corpus_columns(stale).unwrap_err().code,
            "E_CORPUS_COLUMNS_STALE_SOURCE"
        );

        let mut tampered_rows = columns_request();
        tampered_rows.ordered_raw_rows[0][1] = "3.0".to_string();
        assert_eq!(
            build_corpus_columns(tampered_rows).unwrap_err().code,
            "E_CORPUS_COLUMNS_STALE_INTAKE"
        );
    }

    #[test]
    fn ordered_sources_and_one_character_uri_change_identity() {
        let original_request = sources_request();
        let original = build_corpus_sources(original_request.clone()).expect("original");

        let mut reordered_request = original_request.clone();
        reordered_request.ordered_sources.swap(0, 1);
        let reordered = build_corpus_sources(reordered_request).expect("reordered");
        assert_ne!(
            reordered.source_records_sha256,
            original.source_records_sha256
        );

        let restored = build_corpus_sources(original_request.clone()).expect("restored");
        assert_eq!(
            restored.source_records_sha256,
            original.source_records_sha256
        );

        let mut uri_tamper = original_request;
        uri_tamper.ordered_sources[0].canonical_identifier =
            "https://example.org/records/water-tablE".to_string();
        let tampered = build_corpus_sources(uri_tamper).expect("valid but distinct URI");
        assert_ne!(
            tampered.source_records_sha256,
            original.source_records_sha256
        );
    }

    #[test]
    fn source_builder_rejects_duplicate_private_mutation_rights_record_and_stale_input() {
        let mut duplicate = sources_request();
        duplicate.ordered_sources[1].source_ref = duplicate.ordered_sources[0].source_ref.clone();
        assert_eq!(
            build_corpus_sources(duplicate).unwrap_err().code,
            "E_EVIDENCE_DUPLICATE_SOURCE_REF"
        );

        let mut private = sources_request();
        private.ordered_sources[0].canonical_identifier =
            "https://127.0.0.1/records/water".to_string();
        assert_eq!(
            build_corpus_sources(private).unwrap_err().code,
            "E_EVIDENCE_IDENTIFIER_PRIVATE_HOST"
        );

        let mut mutation = sources_request();
        mutation.ordered_sources[0].canonical_identifier =
            "https://example.org/upload/water".to_string();
        assert_eq!(
            build_corpus_sources(mutation).unwrap_err().code,
            "E_EVIDENCE_IDENTIFIER_MUTATION_ENDPOINT"
        );

        let mut capability = sources_request();
        capability.ordered_sources[0].canonical_identifier =
            "https://example.org/records/water?capability=secret".to_string();
        assert_eq!(
            build_corpus_sources(capability).unwrap_err().code,
            "E_EVIDENCE_SECRET_VALUE"
        );

        let mut missing_rights = sources_request();
        missing_rights.ordered_sources[0].rights.license_id.clear();
        assert_eq!(
            build_corpus_sources(missing_rights).unwrap_err().code,
            "E_EVIDENCE_REQUIRED_TEXT"
        );

        let mut missing_record_id = sources_request();
        missing_record_id.ordered_sources[0].source_ref.clear();
        assert_eq!(
            build_corpus_sources(missing_record_id).unwrap_err().code,
            "E_EVIDENCE_REQUIRED_TEXT"
        );

        let mut bad_date = sources_request();
        bad_date.ordered_sources[0].retrieved_at = "2026/07/28".to_string();
        assert_eq!(
            build_corpus_sources(bad_date).unwrap_err().code,
            "E_CORPUS_SOURCE_CHECKED_AT"
        );

        let mut stale_dataset = sources_request();
        stale_dataset.expected_dataset_content_sha256 = "0".repeat(64);
        assert_eq!(
            build_corpus_sources(stale_dataset).unwrap_err().code,
            "E_CORPUS_SOURCES_STALE_DATASET"
        );

        let mut stale_source = sources_request();
        stale_source.source_bytes.push(b'!');
        assert_eq!(
            build_corpus_sources(stale_source).unwrap_err().code,
            "E_CORPUS_SOURCES_STALE_SOURCE"
        );
    }

    #[test]
    fn evidence_builder_seals_existing_claim_and_activity_contract() {
        let direct = build_corpus_evidence(evidence_request()).expect("direct claim");
        assert_eq!(direct.claim_count, 1);
        assert_eq!(direct.activity_count, 0);
        assert_eq!(direct.evidence_manifest_sha256.len(), 64);

        let mut derived = evidence_request();
        derived.claims[0].relation = EvidenceRelation::DerivedFrom;
        assert_eq!(
            build_corpus_evidence(derived.clone()).unwrap_err().code,
            "E_EVIDENCE_DERIVED_ACTIVITY_REQUIRED"
        );
        derived.activities.push(CorpusEvidenceActivityDraft {
            kind: EvidenceActivityKind::Calculate,
            input_source_refs: vec!["record:water-table".to_string()],
            output_targets: vec![CorpusEvidenceTargetDraft {
                row_source_ordinal: 1,
                column_name: Some("물질량".to_string()),
            }],
            recipe_sha256: "a".repeat(64),
        });
        let bound = build_corpus_evidence(derived).expect("derived activity");
        assert_eq!(bound.activity_count, 1);
    }

    #[test]
    fn evidence_builder_rejects_sparse_present_claim_missing_text_and_stale_sources() {
        let mut no_text = evidence_request();
        no_text.claims[0].note = None;
        assert_eq!(
            build_corpus_evidence(no_text).unwrap_err().code,
            "E_CORPUS_CLAIM_TEXT_REQUIRED"
        );

        let mut stale = evidence_request();
        stale.expected_source_records_sha256 = "0".repeat(64);
        assert_eq!(
            build_corpus_evidence(stale).unwrap_err().code,
            "E_CORPUS_EVIDENCE_STALE_SOURCES"
        );

        let mut sparse_columns = columns_request();
        sparse_columns.source_bytes = b"name,amount\nwater,2.0\nfire,\n".to_vec();
        sparse_columns.expected_source_bytes_sha256 =
            hex::encode(Sha256::digest(&sparse_columns.source_bytes));
        sparse_columns
            .ordered_raw_rows
            .push(vec!["fire".to_string(), "".to_string()]);
        let source_bytes = sparse_columns.source_bytes.clone();
        let sparse_result = build_corpus_columns(sparse_columns).expect("sparse columns");
        assert_eq!(
            sparse_result.target_rows[1].cells[1].status,
            CorpusTargetCellStatus::SourceUnavailable
        );
        let mut source_request = sources_request();
        source_request.source_bytes = source_bytes;
        source_request.expected_source_bytes_sha256 =
            hex::encode(Sha256::digest(&source_request.source_bytes));
        source_request.canonical_dataset_json = sparse_result.canonical_dataset_json;
        source_request.expected_dataset_content_sha256 = sparse_result.dataset_content_sha256;
        let source_result = build_corpus_sources(source_request.clone()).expect("sparse sources");
        let mut sparse_claim = evidence_request();
        sparse_claim.source_bytes = source_request.source_bytes;
        sparse_claim.expected_source_bytes_sha256 = source_request.expected_source_bytes_sha256;
        sparse_claim.canonical_dataset_json = source_request.canonical_dataset_json;
        sparse_claim.expected_dataset_content_sha256 =
            source_request.expected_dataset_content_sha256;
        sparse_claim.ordered_sources = source_request.ordered_sources;
        sparse_claim.expected_source_records_sha256 = source_result.source_records_sha256;
        sparse_claim.claims[0].target.row_source_ordinal = 2;
        assert_eq!(
            build_corpus_evidence(sparse_claim).unwrap_err().code,
            "E_EVIDENCE_SPARSE_NON_PRESENT_CLAIM"
        );
    }

    #[test]
    fn review_readiness_is_rust_owned_and_preserves_invalid_claims() {
        let ready = build_corpus_review(review_request()).expect("review");
        assert!(ready.ready_to_seal);
        assert_eq!(ready.row_count, 1);
        assert_eq!(ready.column_count, 2);
        assert_eq!(ready.source_count, 2);
        assert_eq!(ready.claim_count, 1);
        assert_eq!(ready.activity_count, 0);
        assert!(ready.errors.is_empty());

        let mut stale = review_request();
        stale.expected_evidence_manifest_sha256 = "0".repeat(64);
        let stale_result = build_corpus_review(stale).expect("stale review result");
        assert!(!stale_result.ready_to_seal);
        assert_eq!(
            stale_result.errors[0].code,
            "E_CORPUS_REVIEW_STALE_EVIDENCE"
        );

        let mut missing_source = review_request();
        missing_source.claims[0].source_ref = "record:not-registered".to_string();
        let invalid = build_corpus_review(missing_source).expect("invalid review result");
        assert!(!invalid.ready_to_seal);
        assert_eq!(invalid.claim_count, 1, "invalid claim must not be deleted");
        assert!(invalid
            .errors
            .iter()
            .any(|issue| issue.code == "E_EVIDENCE_SOURCE_REF_NOT_FOUND"));
    }

    #[test]
    fn seal_builds_verified_project_files_and_rejects_stale_revision() {
        let sealed = build_corpus_seal(seal_request()).expect("seal");
        assert_eq!(sealed.project_files.len(), 4);
        assert_eq!(sealed.project_files[0].path, "ddn.asset.json");
        assert_eq!(sealed.project_files[0].role, "asset_manifest");
        assert!(sealed
            .project_files
            .iter()
            .any(|file| file.role == "sealed_dataset_envelope"));
        assert!(sealed
            .project_files
            .iter()
            .any(|file| file.role == "dataset_payload"));
        assert!(sealed
            .project_files
            .iter()
            .any(|file| file.role == "domain_evidence_manifest"));
        assert_eq!(sealed.dataset_handle.len(), 16);
        assert_eq!(sealed.asset_manifest_sha256.len(), 64);

        let mut stale_revision = seal_request();
        stale_revision.expected_project_revision_ref = "revision:other".to_string();
        assert_eq!(
            build_corpus_seal(stale_revision).unwrap_err().code,
            "E_CORPUS_SEAL_PROJECT_REVISION_STALE"
        );

        let mut tampered_binding = seal_request();
        tampered_binding.expected_content_binding_sha256 = "0".repeat(64);
        assert_eq!(
            build_corpus_seal(tampered_binding).unwrap_err().code,
            "E_CORPUS_SEAL_BINDING_STALE"
        );
    }
}
