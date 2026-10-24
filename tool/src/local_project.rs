// 패닉 감사 F5(가), 이 모듈과 분리한 자식 구현의 한정 경계:
// windows(2)가 만든 pair의 [0]/[1]은 항상 유효하다. 드라이브 접두사 검사는
// bytes.len()>=2의 단락 평가 뒤에만 바이트 둘을 읽는다.
// 닫힌 serde_json::Value/String 및 (String,String,usize) 차림을 자체 Vec writer로
// 직렬화하는 expect는 사용자 정의 Serialize나 파일 I/O를 호출하지 않는다.
// 임의 T:Serialize, 다른 자료형 직렬화, graph/map 생산자와 source-boundary 배열은
// 이 설명으로 안전 판정하지 않는다. JSON 생성 매크로·할당·재귀도 별도 감사 대상이다.
// 추가 F5 생산자 감사: 원문 토큰의 index/open/close는 같은 불변 token 차림의
// 길이를 검사한 반복 안에서만 읽는다. parse_work_root의 Modules 필드는 파서가
// 필수 존재와 형식을 확인한 뒤 같은 불변 root에서 읽는다.
// 모듈 추출 pending은 검증된 graph 모듈 이름에서만 늘어나고 remaining은 그 키의
// 부분집합이다. 포함할 경로도 같은 정규 원문 map의 closure에서만 공급한다.
// 이 경계의 산술·재귀 한도와 raw 머리 레코드 조회의 별도 오류를 혼동하지 않는다.
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::work_cost::{
    StandardResourceLimitsV1, WorkCostCounters, WorkCostMeter, WorkCostReceipt,
    E_WORK_COST_REGISTRY_MISSING_CANDIDATE,
};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::domain_evidence_manifest::{
    evidence_manifest_identity, parse_and_validate_domain_evidence_manifest, EvidenceCellStatus,
    EvidenceDatasetRowTarget, EvidenceDatasetTarget, EvidenceManifestIdentity,
};
use crate::gate0_registry::{
    build_project_asset_registry_from_manifest, ProjectAssetRegistry, SealedDatasetAssetIdentity,
};
use crate::host_resource::{self, HostResourceRequest, PreparedHostResources};
use crate::runtime_surface;
use crate::sealed_domain_dataset::{
    DatasetCell, DatasetSource, MissingReason, SealedDomainDataset, SEALED_DOMAIN_DATASET_SCHEMA,
};
use ddonirang_core::ResourceHandle;
use ddonirang_lang::{
    parse_module_directive_prefix, resolve_known_call_target, Body, Expr, ExprKind, Lexer,
    ModuleDirectiveContext, ParseMode, Span, Stmt, Token, TokenKind, TopLevelItem,
};

pub const VIRTUAL_PROJECT_SOURCE_SCHEMA: &str = "ddn.virtual_project_source.v1";
pub const VIRTUAL_PROJECT_MANIFEST_SCHEMA: &str = "ddn.virtual_project_manifest.v1";
pub const PROJECT_ROOT_DISCOVERY_REQUEST_SCHEMA: &str = "ddn.project_root_discovery_request.v1";
pub const PROJECT_ROOT_DISCOVERY_SCHEMA: &str = "ddn.project_root_discovery.v1";
pub const PROJECT_GRAPH_SCHEMA: &str = "ddn.local_project_graph.v1";
pub const PROJECT_SYMBOL_BOUNDARY_SCHEMA: &str = "ddn.local_project_symbol_boundary.v1";
pub const LOCAL_GAJI_LOCK_SCHEMA: &str = "ddn.local_gaji_lock.v1";
pub const LOCAL_PROJECT_RUN_SUMMARY_SCHEMA: &str = "ddn.local_project_run_summary.v1";
pub const LOCAL_PROJECT_RUN_SEMANTIC_IDENTITY: &str = "ddn.project-run.v1";
pub const PROJECT_EDIT_PLAN_SCHEMA: &str = "ddn.local_project_edit_plan.v1";
pub const PROJECT_EXECUTION_RECIPE_REQUEST_SCHEMA: &str =
    "ddn.local_project.execution_recipe_request.v1";
pub const GAJI_EXTRACTION_PLAN_SCHEMA: &str = "ddn.gaji_extraction_plan.v1";
pub const LOCAL_GAJI_ARTIFACT_SCHEMA: &str = "ddn.local_gaji_artifact.v1";
pub const LOCAL_GOTGAN_STORE_SCHEMA: &str = "ddn.local_gotgan_store.v1";
pub const GAJI_LIFECYCLE_PLAN_SCHEMA: &str = "ddn.gaji_lifecycle_plan.v1";
const GAJI_ARTIFACT_MAX_FILES: usize = 256;
const GAJI_ARTIFACT_MAX_SINGLE_FILE_BYTES: usize = 1024 * 1024;
const GAJI_ARTIFACT_MAX_TOTAL_BYTES: usize = 8 * 1024 * 1024;
const GAJI_ARTIFACT_MAX_MANIFEST_BYTES: usize = 256 * 1024;
pub const LOCAL_PROJECT_TREE_KIND: &str = "seamgrim.local_project_tree.v0";
pub const MAX_PROJECT_FILES: usize = 64;
pub const MAX_FILE_BYTES: usize = 256 * 1024;
pub const MAX_SERIALIZED_ARTIFACT_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_PROJECT_TOTAL_UTF8_BYTES: usize = 2 * 1024 * 1024;
pub const PROJECT_ROOT_FILE: &str = "뿌리.ddn";
pub const PROJECT_JSON_FILE: &str = "ddn.project.json";
const LEGACY_ROOT_FILES: &[&str] = &["프로젝트.ddn", "project.ddn"];

fn require_beta_project_feature(code: &'static str, message: &str) -> Result<(), LocalProjectError> {
    if cfg!(feature = "beta-project-features") {
        Ok(())
    } else {
        Err(LocalProjectError::new(code, message))
    }
}

#[derive(Debug, Clone)]
pub struct LocalProjectError {
    pub code: &'static str,
    pub message: String,
    pub path: Option<String>,
    pub source_span: Option<(usize, usize)>,
    pub phase: Option<&'static str>,
    pub cause: Option<Box<crate::preprocess::ExecutionDiagnostic>>,
    pub offending_field: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

impl LocalProjectError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: None,
            source_span: None,
            phase: None,
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        }
    }

    pub(crate) fn with_path(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path: Some(path.into()),
            source_span: None,
            phase: None,
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        }
    }

    fn with_path_span(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
        source_span: (usize, usize),
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path: Some(path.into()),
            source_span: Some(source_span),
            phase: None,
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        }
    }

    fn from_execution_diagnostic(err: crate::preprocess::ExecutionDiagnostic) -> Self {
        Self {
            code: err.code,
            message: err.message.clone(),
            path: Some(err.source_path.clone()),
            source_span: err.span,
            phase: Some(err.phase),
            cause: err.cause.clone(),
            offending_field: err.offending_field.clone(),
            expected: err.expected.clone(),
            actual: err.actual.clone(),
        }
    }

    fn execution_diagnostic_json(err: &crate::preprocess::ExecutionDiagnostic) -> JsonValue {
        json!({
            "code": err.code,
            "message": err.message,
            "source_path": err.source_path,
            "span": err.span.map(|(start, end)| json!({"start": start, "end": end})),
            "phase": err.phase,
            "cause": err.cause.as_deref().map(Self::execution_diagnostic_json),
            "offending_field": err.offending_field,
            "expected": err.expected,
            "actual": err.actual,
        })
    }

    pub fn to_json_value(&self) -> JsonValue {
        let mut value = json!({
            "schema": "ddn.local_project.error.v1",
            "code": self.code,
            "message": self.message,
            "path": self.path,
            "source_ref": self.path,
            "source_path": self.path,
            "severity": "fatal",
            "phase": self.phase,
            "cause": self.cause.as_deref().map(Self::execution_diagnostic_json),
            "offending_field": self.offending_field,
            "expected": self.expected,
            "actual": self.actual,
        });
        if let Some((start, end)) = self.source_span {
            if let Some(object) = value.as_object_mut() {
                object.insert(
                    "source_span".to_string(),
                    json!({"start": start, "end": end}),
                );
            }
        }
        value
    }
}

impl fmt::Display for LocalProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{}: {} ({})", self.code, self.message, path)
        } else {
            write!(f, "{}: {}", self.code, self.message)
        }
    }
}

impl std::error::Error for LocalProjectError {}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirtualProjectSource {
    pub schema: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_hash: Option<String>,
    pub root_path: String,
    pub entry_path: String,
    pub files: Vec<VirtualProjectFileSource>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirtualProjectFileSource {
    pub path: String,
    pub role: String,
    pub utf8_text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectEditPlanRequest {
    pub schema: String,
    pub source: VirtualProjectSource,
    pub operation: ProjectEditOperation,
    pub expected_source_sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectEditOperation {
    RenameFile {
        from_path: String,
        to_path: String,
    },
    AddFile {
        path: String,
        utf8_text: String,
        #[serde(default = "default_ddn_source_role")]
        role: String,
    },
    AddModule {
        path: String,
        module_name: String,
    },
    DuplicateFile {
        from_path: String,
        to_path: String,
    },
    DeleteFile {
        path: String,
    },
    SetEntry {
        path: String,
    },
    BindSealedDataset {
        binding_name: String,
        dataset_handle: String,
    },
}

fn default_ddn_source_role() -> String {
    "ddn_source".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectEditPatch {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub before: String,
    pub after: String,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectEditPlan {
    pub schema: &'static str,
    pub operation: &'static str,
    pub before_source_sha256: String,
    pub after_source_sha256: String,
    pub affected_files: Vec<String>,
    pub affected_symbols: Vec<String>,
    pub affected_references: Vec<String>,
    pub patches: Vec<ProjectEditPatch>,
    pub diagnostics: Vec<ProjectDiagnostic>,
    pub project_source: VirtualProjectSource,
    pub atomic_apply: bool,
    pub lock_stale: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectExecutionRecipeRequest {
    pub schema: String,
    pub prepare_link_sha256: String,
    pub recipe_version: u64,
    pub setup: JsonValue,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectExecutionRecipeIdentity {
    pub schema: &'static str,
    pub prepare_link_sha256: String,
    pub recipe_version: u64,
    pub setup: JsonValue,
    pub execution_recipe_sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GajiExtractionPlanRequest {
    pub schema: String,
    pub source: VirtualProjectSource,
    pub selected_modules: Vec<String>,
    pub gaji_id: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_local_gotgan_profile")]
    pub profile: String,
    #[serde(default)]
    pub origin_project_ref: Option<String>,
    #[serde(default)]
    pub origin_gaji: Option<String>,
    #[serde(default = "default_gaji_relation")]
    pub relation: String,
}

fn default_local_gotgan_profile() -> String {
    "내".to_string()
}

fn default_gaji_relation() -> String {
    "extracted".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GajiPublicSymbol {
    pub module_path: String,
    pub external_name: String,
    pub internal_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GajiExtractionFile {
    pub path: String,
    pub byte_length: usize,
    pub sha256: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GajiExtractionPlan {
    pub schema: &'static str,
    pub gaji_id: String,
    pub version: String,
    pub profile: String,
    pub origin_project_ref: Option<String>,
    pub source_project_sha256: String,
    pub selected_modules: Vec<String>,
    pub included_modules: Vec<String>,
    pub excluded_files: Vec<String>,
    pub public_symbols: Vec<GajiPublicSymbol>,
    pub external_gaji_dependencies: BTreeMap<String, String>,
    pub files: Vec<GajiExtractionFile>,
    pub content_sha256: String,
    pub diagnostics: Vec<ProjectDiagnostic>,
    pub publishable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalGajiArtifactFile {
    pub path: String,
    pub utf8_text: String,
    pub byte_length: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalGajiArtifactProvenance {
    pub origin_project_ref: Option<String>,
    pub origin_gaji: Option<String>,
    pub relation: String,
    pub extraction_modules: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalGajiArtifact {
    pub schema: String,
    pub gaji_id: String,
    pub version: String,
    pub profile: String,
    pub description: String,
    pub tags: Vec<String>,
    pub manifest_path: String,
    pub content_sha256: String,
    pub package_sha256: String,
    pub public_symbols: Vec<GajiPublicSymbol>,
    pub dependencies: BTreeMap<String, String>,
    pub provenance: LocalGajiArtifactProvenance,
    pub files: Vec<LocalGajiArtifactFile>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalGotganStore {
    pub schema: String,
    pub artifacts: Vec<LocalGajiArtifact>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalGotganPublishRequest {
    pub schema: String,
    pub store: LocalGotganStore,
    pub artifact: LocalGajiArtifact,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalGotganSearchRequest {
    pub schema: String,
    pub store: LocalGotganStore,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub profiles: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalGotganSearchRow {
    pub profile: String,
    pub gaji_id: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub public_symbols: Vec<String>,
    pub content_sha256: String,
    pub package_sha256: String,
    pub origin_project_ref: Option<String>,
    pub relation: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GajiLifecyclePlanRequest {
    pub schema: String,
    pub source: VirtualProjectSource,
    pub expected_source_sha256: String,
    #[serde(default)]
    pub expected_lock_sha256: Option<String>,
    pub operation: GajiLifecycleOperation,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GajiLifecycleOperation {
    Install {
        alias: String,
        artifact: LocalGajiArtifact,
    },
    Update {
        alias: String,
        artifact: LocalGajiArtifact,
    },
    Remove {
        alias: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct GajiLifecyclePlan {
    pub schema: &'static str,
    pub operation: &'static str,
    pub alias: String,
    pub before_source_sha256: String,
    pub after_source_sha256: String,
    pub before_lock_sha256: Option<String>,
    pub after_lock_sha256: Option<String>,
    pub affected_files: Vec<String>,
    pub version_diff: Option<GajiVersionDiff>,
    pub diagnostics: Vec<ProjectDiagnostic>,
    pub project_source: VirtualProjectSource,
    pub atomic_apply: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GajiVersionDiff {
    pub from_version: Option<String>,
    pub to_version: Option<String>,
    pub added_files: Vec<String>,
    pub removed_files: Vec<String>,
    pub changed_files: Vec<String>,
    pub added_public_symbols: Vec<String>,
    pub removed_public_symbols: Vec<String>,
    pub dependency_changed: bool,
    pub risky: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedProjectManifest {
    pub schema: &'static str,
    pub kind: &'static str,
    pub root_path: String,
    pub entry_path: String,
    pub file_count: usize,
    pub total_utf8_bytes: usize,
    pub files: Vec<NormalizedProjectFile>,
    pub project_graph_sha256: String,
    pub lock_sha256: Option<String>,
    pub canonical_manifest_sha256: String,
    pub limits: ProjectLimits,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedProjectFile {
    pub path: String,
    pub role: String,
    pub byte_length: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectLimits {
    pub max_files: usize,
    pub max_file_bytes: usize,
    pub max_serialized_artifact_bytes: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRootDiscoveryRequest {
    pub schema: String,
    pub start_path: String,
    #[serde(default)]
    pub explicit_project_path: Option<String>,
    pub files: Vec<ProjectRootDiscoveryFile>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRootDiscoveryFile {
    pub path: String,
    #[serde(default)]
    pub utf8_text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectRootDiscoveryResult {
    pub schema: &'static str,
    pub root_kind: &'static str,
    pub root_marker_path: Option<String>,
    pub project_json_path: Option<String>,
    pub used_fallback: bool,
    pub diagnostics: Vec<ProjectDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalProjectGraph {
    pub schema: &'static str,
    pub project_name: String,
    pub root_path: String,
    pub entry_path: String,
    pub targets: BTreeMap<String, String>,
    pub module_paths: BTreeMap<String, String>,
    pub gaji_aliases: BTreeMap<String, String>,
    pub module_sources: BTreeMap<String, Vec<String>>,
    pub start_definition: Option<String>,
    pub file_count: usize,
    pub files: Vec<NormalizedProjectFile>,
    pub project_graph_sha256: String,
    pub diagnostics: Vec<ProjectDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalProjectSymbolBoundary {
    pub schema: &'static str,
    pub entry_path: String,
    pub module_graph_order: Vec<String>,
    pub module_graph_sha256: String,
    pub imports: Vec<ResolvedImport>,
    pub exports: Vec<ResolvedExport>,
    pub resolved_symbols: Vec<ResolvedSymbol>,
    pub output_rows: Vec<String>,
    pub symbol_boundary_sha256: String,
    pub diagnostics: Vec<ProjectDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedImport {
    pub importer_path: String,
    pub alias: String,
    pub source: String,
    pub resolved_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedExport {
    pub module_path: String,
    pub external_name: String,
    pub internal_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedSymbol {
    pub alias: String,
    pub symbol: String,
    pub module_path: String,
    pub internal_name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalGajiLock {
    pub schema: &'static str,
    pub lock_format: &'static str,
    pub root_path: String,
    pub entry_path: String,
    pub project_graph_sha256: String,
    pub packages: Vec<LocalGajiPin>,
    pub lock_sha256: String,
    pub diagnostics: Vec<ProjectDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalGajiPin {
    pub id: String,
    pub version: String,
    pub path: String,
    pub direct: bool,
    pub dependency_of: Vec<String>,
    pub dependencies: BTreeMap<String, String>,
    pub manifest_path: String,
    pub manifest_sha256: String,
    pub package_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalProjectRunSummary {
    pub schema: &'static str,
    pub frontdoor: &'static str,
    pub root_path: String,
    pub entry_path: String,
    pub target: String,
    pub target_path: String,
    pub file_count: usize,
    pub project_graph_sha256: String,
    pub lock_sha256: Option<String>,
    pub symbol_boundary_sha256: String,
    pub output_rows: Vec<JsonValue>,
    pub runtime_output_log_texts: Vec<String>,
    pub runtime_final_row: JsonValue,
    pub runtime_state_hash: String,
    pub runtime_engine_state: JsonValue,
    pub runtime_execution_envelope: JsonValue,
    pub work_cost_receipt: WorkCostReceipt,
    pub execution_preparation: JsonValue,
    pub project_observation_trace: JsonValue,
    pub project_observation_trace_hash: String,
    pub project_observation_table_projection: JsonValue,
    pub sealed_dataset_assets: Vec<SealedDatasetAssetIdentity>,
    pub domain_evidence_manifests: Vec<EvidenceManifestIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_resource_transcript: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_resource_receipt: Option<JsonValue>,
    pub linked_execution_source: String,
    pub linked_execution_canonical_ddn: String,
    pub linked_execution_source_sha256: String,
    pub linked_execution_canonical_sha256: String,
    pub bounded_state_sha256: String,
    pub graph_inspected: bool,
    pub lock_inspected: bool,
    pub summary_sha256: String,
    pub diagnostics: Vec<ProjectDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalProjectPrepareLink {
    pub source_profile_identity: &'static str,
    pub schema: &'static str,
    pub frontdoor: &'static str,
    pub root_path: String,
    pub entry_path: String,
    pub target: String,
    pub target_path: String,
    pub file_count: usize,
    pub normalized_project: NormalizedProjectManifest,
    pub project_graph: LocalProjectGraph,
    pub graph_inspected: bool,
    pub lock_required: bool,
    pub lock_inspected: bool,
    pub verified_lock: Option<LocalGajiLock>,
    pub linked_execution_source: String,
    pub linked_execution_source_sha256: String,
    pub runtime_execution_envelope: JsonValue,
    pub symbol_boundary: LocalProjectSymbolBoundary,
    pub sealed_dataset_assets: Vec<SealedDatasetAssetIdentity>,
    pub domain_evidence_manifests: Vec<EvidenceManifestIdentity>,
    pub diagnostics: Vec<ProjectDiagnostic>,
    pub state_hash_formula_change: bool,
    pub prepare_link_sha256: String,
}

#[derive(Debug, Clone)]
struct PendingFile {
    path: String,
    role: String,
    byte_len: usize,
    sha256: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedImport {
    pub(crate) alias: String,
    pub(crate) source: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedExport {
    pub(crate) external_name: String,
    pub(crate) internal_name: String,
}

#[derive(Debug, Clone)]
struct AliasShowRequest {
    alias: String,
    symbol: String,
    source_span: Option<(usize, usize)>,
}

#[derive(Debug, Clone)]
struct ModuleInfo {
    path: String,
    imports: Vec<ResolvedImport>,
    exports: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct SymbolValue {
    module_path: String,
    internal_name: String,
}

#[derive(Debug, Clone)]
struct LinkedProject {
    symbol_boundary: LocalProjectSymbolBoundary,
    execution_source: String,
    execution_source_map: Vec<LinkedSourceSegment>,
    start_definition: Option<String>,
}

#[derive(Debug, Clone)]
struct LinkedSourceSegment {
    generated_start: usize,
    generated_end: usize,
    source_path: String,
    source_boundaries: Vec<usize>,
}

#[derive(Debug, Clone)]
struct MappedSourceText {
    text: String,
    source_boundaries: Vec<usize>,
}

impl LinkedProject {
    fn remap_execution_diagnostic(
        &self,
        err: crate::preprocess::ExecutionDiagnostic,
    ) -> crate::preprocess::ExecutionDiagnostic {
        remap_execution_diagnostic(&self.execution_source_map, err)
    }
}

fn remap_execution_diagnostic(
    execution_source_map: &[LinkedSourceSegment],
    err: crate::preprocess::ExecutionDiagnostic,
) -> crate::preprocess::ExecutionDiagnostic {
    let Some((start, end)) = err.span else {
        return err;
    };
    let Some(segment) = execution_source_map
        .iter()
        .find(|segment| start >= segment.generated_start && end <= segment.generated_end)
    else {
        return err;
    };
    let relative_start = start - segment.generated_start;
    let relative_end = end - segment.generated_start;
    let Some(source_start) = segment.source_boundaries.get(relative_start).copied() else {
        return err;
    };
    let Some(source_end) = segment.source_boundaries.get(relative_end).copied() else {
        return err;
    };
    err.remap_source(segment.source_path.clone(), (source_start, source_end))
}

#[derive(Debug, Clone)]
struct ParsedGajiManifest {
    id: String,
    version: String,
    dependencies: BTreeMap<String, String>,
    manifest_path: String,
    manifest_sha256: String,
    package_sha256: String,
}

pub fn normalize_virtual_project_source_json(
    input_json: &str,
) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let manifest = normalize_virtual_project_source(source)?;
    Ok(to_canonical_json_line(&manifest))
}

pub fn build_project_edit_plan_json(input_json: &str) -> Result<String, LocalProjectError> {
    let request: ProjectEditPlanRequest = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_EDIT_REQUEST_JSON",
            format!("edit-plan 요청 JSON이 올바르지 않습니다: {err}"),
        )
    })?;
    if request.schema != PROJECT_EDIT_PLAN_SCHEMA {
        return Err(LocalProjectError::new(
            "E_PROJECT_EDIT_SCHEMA",
            format!("schema는 {PROJECT_EDIT_PLAN_SCHEMA} 이어야 합니다"),
        ));
    }
    let plan = build_project_edit_plan(request)?;
    Ok(to_canonical_json_line(&plan))
}

pub fn build_project_execution_recipe_json(input_json: &str) -> Result<String, LocalProjectError> {
    let request: ProjectExecutionRecipeRequest =
        serde_json::from_str(input_json).map_err(|err| {
            LocalProjectError::new(
                "E_PROJECT_EXECUTION_RECIPE_JSON",
                format!("실행 recipe 요청 JSON이 올바르지 않습니다: {err}"),
            )
        })?;
    let identity = build_project_execution_recipe(request)?;
    Ok(to_canonical_json_line(&identity))
}

pub fn build_project_execution_recipe(
    request: ProjectExecutionRecipeRequest,
) -> Result<ProjectExecutionRecipeIdentity, LocalProjectError> {
    require_beta_project_feature("E_ALPHA_DATASET_UNSUPPORTED", "자료셋과 실행 자료 연결은 베타에서 지원할 예정입니다")?;
    if request.schema != PROJECT_EXECUTION_RECIPE_REQUEST_SCHEMA {
        return Err(LocalProjectError::new(
            "E_PROJECT_EXECUTION_RECIPE_SCHEMA",
            format!("schema는 {PROJECT_EXECUTION_RECIPE_REQUEST_SCHEMA} 이어야 합니다"),
        ));
    }
    if request.prepare_link_sha256.len() != 64
        || !request
            .prepare_link_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(LocalProjectError::new(
            "E_PROJECT_EXECUTION_RECIPE_PREPARE_LINK",
            "실행 recipe에는 검증된 prepare/link sha256가 필요합니다",
        ));
    }
    if !request.setup.is_object() {
        return Err(LocalProjectError::new(
            "E_PROJECT_EXECUTION_RECIPE_SETUP",
            "실행 recipe setup은 JSON object여야 합니다",
        ));
    }
    let hash_input = json!({
        "schema": "ddn.local_project.execution_recipe.hash_input.v1",
        "prepare_link_sha256": request.prepare_link_sha256,
        "recipe_version": request.recipe_version,
        "setup": request.setup,
    });
    let execution_recipe_sha256 = sha256_hex(
        serde_json::to_string(&hash_input)
            .expect("recipe hash input")
            .as_bytes(),
    );
    Ok(ProjectExecutionRecipeIdentity {
        schema: "ddn.local_project.execution_recipe.v1",
        prepare_link_sha256: hash_input["prepare_link_sha256"]
            .as_str()
            .expect("validated prepare link")
            .to_string(),
        recipe_version: request.recipe_version,
        setup: hash_input["setup"].clone(),
        execution_recipe_sha256,
    })
}

mod gaji_lifecycle;
use gaji_lifecycle::*;
pub use gaji_lifecycle::{build_gaji_extraction_plan_json, build_local_gaji_artifact_json, verify_local_gaji_artifact_json, publish_local_gotgan_json, search_local_gotgan_json, build_gaji_lifecycle_plan_json, build_gaji_lifecycle_plan, publish_local_gotgan, search_local_gotgan, build_local_gaji_artifact, verify_local_gaji_artifact, build_gaji_extraction_plan};

pub fn virtual_project_source_sha256_json(input_json: &str) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON",
            format!("virtual project source JSON이 올바르지 않습니다: {err}"),
        )
    })?;
    virtual_project_source_sha256(&source)
}

pub fn virtual_project_source_sha256(
    source: &VirtualProjectSource,
) -> Result<String, LocalProjectError> {
    normalize_virtual_project_source(source.clone())?;
    let mut files = source.files.clone();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let canonical = serde_json::to_vec(&json!({
        "schema": source.schema,
        "kind": source.kind,
        "root_path": source.root_path,
        "entry_path": source.entry_path,
        "files": files,
    }))
    .expect("virtual project hash input serialization");
    Ok(sha256_hex(&canonical))
}

mod source_edits;
use source_edits::*;
pub use source_edits::{build_project_edit_plan, project_source_profile};

pub fn discover_virtual_project_root_json(input_json: &str) -> Result<String, LocalProjectError> {
    let request: ProjectRootDiscoveryRequest = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_DISCOVERY_JSON_INVALID",
            format!("project root discovery JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let result = discover_virtual_project_root(request)?;
    Ok(to_canonical_json_line(&result))
}

pub fn discover_disk_project_root_json(
    start: &Path,
    explicit_project: Option<&Path>,
) -> Result<String, LocalProjectError> {
    let result = discover_disk_project_root(start, explicit_project)?;
    Ok(to_canonical_json_line(&result))
}

pub fn build_project_graph_json(input_json: &str) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let graph = build_project_graph(source)?;
    Ok(to_canonical_json_line(&graph))
}

pub fn build_project_symbol_boundary_json(input_json: &str) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let symbols = build_project_symbol_boundary(source)?;
    Ok(to_canonical_json_line(&symbols))
}

pub fn build_project_gaji_lock_json(input_json: &str) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let lock = build_project_gaji_lock(source, false)?;
    Ok(to_canonical_json_line(&lock))
}

pub fn verify_project_gaji_lock_json(input_json: &str) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let lock = build_project_gaji_lock(source, true)?;
    Ok(to_canonical_json_line(&lock))
}

pub fn build_project_run_summary_json(
    input_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let summary = build_project_run_summary(source, target, inspect_graph, inspect_lock)?;
    Ok(to_canonical_json_line(&summary))
}

pub fn build_project_run_summary_with_supported_profile_json(
    input_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let summary = build_project_run_summary_with_supported_profile(
        source,
        target,
        inspect_graph,
        inspect_lock,
    )?;
    Ok(to_canonical_json_line(&summary))
}

pub fn build_project_run_summary_with_supported_profile_host_resources_json(
    input_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
    host_resource_request_json: &str,
) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let request = host_resource::parse_request_json(host_resource_request_json)
        .map_err(|error| LocalProjectError::new("E_HOST_RESOURCE_REQUEST", error))?;
    let summary = build_project_run_summary_with_supported_profile_and_host_resources(
        source,
        target,
        inspect_graph,
        inspect_lock,
        &request,
    )?;
    Ok(to_canonical_json_line(&summary))
}

pub fn build_project_run_summary_with_supported_profile_prepared_host_resources_json(
    input_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
    prepared: &PreparedHostResources,
) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let summary = build_project_run_summary_with_supported_profile_and_prepared_host_resources(
        source,
        target,
        inspect_graph,
        inspect_lock,
        prepared,
    )?;
    Ok(to_canonical_json_line(&summary))
}

pub fn write_project_run_artifact_atomic(path: &Path, text: &str) -> Result<(), LocalProjectError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        LocalProjectError::with_path(
            "E_PROJECT_RUN_WRITE",
            parent.display().to_string(),
            format!("출력 폴더를 만들 수 없습니다: {error}"),
        )
    })?;
    if path.is_dir() {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_RUN_WRITE",
            path.display().to_string(),
            "출력 경로가 폴더입니다",
        ));
    }
    let file_name = path.file_name().ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_RUN_WRITE",
            path.display().to_string(),
            "출력 파일 이름이 없습니다",
        )
    })?;
    let file_name = file_name.to_string_lossy();
    let nonce = std::process::id();
    let temp = parent.join(format!(".{file_name}.ddn-project-run-tmp-{nonce}"));
    let backup = parent.join(format!(".{file_name}.ddn-project-run-backup-{nonce}"));
    if temp.exists() || backup.exists() {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_RUN_WRITE",
            path.display().to_string(),
            "이전 atomic 출력 잔여가 있습니다",
        ));
    }
    fs::write(&temp, text.as_bytes()).map_err(|error| {
        LocalProjectError::with_path(
            "E_PROJECT_RUN_WRITE",
            temp.display().to_string(),
            format!("임시 출력 파일을 쓸 수 없습니다: {error}"),
        )
    })?;

    let had_existing = path.exists();
    if had_existing {
        if let Err(error) = fs::rename(path, &backup) {
            let _ = fs::remove_file(&temp);
            return Err(LocalProjectError::with_path(
                "E_PROJECT_RUN_WRITE",
                path.display().to_string(),
                format!("기존 출력을 보존할 수 없습니다: {error}"),
            ));
        }
    }
    if let Err(error) = fs::rename(&temp, path) {
        if had_existing && backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temp);
        return Err(LocalProjectError::with_path(
            "E_PROJECT_RUN_WRITE",
            path.display().to_string(),
            format!("최종 출력을 atomic replacement할 수 없습니다: {error}"),
        ));
    }
    if backup.exists() {
        fs::remove_file(&backup).map_err(|error| {
            LocalProjectError::with_path(
                "E_PROJECT_RUN_WRITE",
                backup.display().to_string(),
                format!("교체 후 백업을 정리할 수 없습니다: {error}"),
            )
        })?;
    }
    Ok(())
}

pub fn build_project_prepare_link_json(
    input_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<String, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    let prepared = build_project_prepare_link(source, target, inspect_graph, inspect_lock)?;
    Ok(to_canonical_json_line(&prepared))
}

pub fn build_disk_project_run_summary_json(
    start: &Path,
    explicit_project: Option<&Path>,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<String, LocalProjectError> {
    let source = build_virtual_project_source_from_disk(start, explicit_project)?;
    let summary = build_project_run_summary(source, target, inspect_graph, inspect_lock)?;
    Ok(to_canonical_json_line(&summary))
}

pub fn build_disk_project_run_summary_with_supported_profile_json(
    start: &Path,
    explicit_project: Option<&Path>,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<String, LocalProjectError> {
    let source = build_virtual_project_source_from_disk(start, explicit_project)?;
    let summary = build_project_run_summary_with_supported_profile(
        source,
        target,
        inspect_graph,
        inspect_lock,
    )?;
    Ok(to_canonical_json_line(&summary))
}

pub fn build_disk_project_run_summary_with_supported_profile_prepared_host_resources_json(
    start: &Path,
    explicit_project: Option<&Path>,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
    prepared: &PreparedHostResources,
) -> Result<String, LocalProjectError> {
    let source = build_virtual_project_source_from_disk(start, explicit_project)?;
    let summary = build_project_run_summary_with_supported_profile_and_prepared_host_resources(
        source,
        target,
        inspect_graph,
        inspect_lock,
        prepared,
    )?;
    Ok(to_canonical_json_line(&summary))
}

pub fn build_project_graph(
    source: VirtualProjectSource,
) -> Result<LocalProjectGraph, LocalProjectError> {
    if source.files.iter().any(|file| matches!(normalize_role(&file.role).as_str(), "root" | "ddn_source")
        && ddonirang_lang::has_model_surface(&file.utf8_text)) {
        require_beta_project_feature("E_ALPHA_MODEL_PROJECT_UNSUPPORTED", "정식 작품의 모형 연결은 알파에서 지원하지 않습니다. 한 파일 모형 실행과는 별도 기능입니다")?;
    }
    project_source_profile(&source, None)?;
    let normalized_manifest = normalize_virtual_project_source(source.clone())?;
    let root_path = normalized_manifest.root_path.clone();
    let entry_from_source = normalized_manifest.entry_path.clone();
    let mut text_by_path = BTreeMap::new();
    let mut role_by_path = BTreeMap::new();
    for file in source.files {
        let path = normalize_project_path(&file.path)?;
        role_by_path.insert(path.clone(), normalize_role(&file.role));
        text_by_path.insert(path, file.utf8_text);
    }
    let root_text = text_by_path.get(&root_path).ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_ROOT_MISSING",
            root_path.clone(),
            "project graph를 만들 root 파일이 없습니다",
        )
    })?;
    let directive = ddonirang_lang::work_root::parse_work_root(root_text).map_err(|error| {
        LocalProjectError::with_path_span("E_PROJECT_ROOT_PARSE", &root_path,
            error.message, (error.span.start, error.span.end))
    })?;
    for field in [ddonirang_lang::work_root::RootField::ExecutionContract,
        ddonirang_lang::work_root::RootField::View, ddonirang_lang::work_root::RootField::Exports] {
        if directive.field(field).is_some() {
            return Err(LocalProjectError::with_path("E_PROJECT_CONFIGURATION_UNSUPPORTED", &root_path,
                "이 알파 실행기는 명시 실행계약·보개·가지 내놓음을 아직 결속하지 못합니다. 값을 무시하고 실행하지 않습니다"));
        }
    }
    let declarations = directive.typed_modules().map_err(|error| {
        LocalProjectError::with_path_span("E_PROJECT_MODULE_DECLARATION", &root_path,
            error.message, (error.span.start, error.span.end))
    })?;
    let start = directive.typed_start().map_err(|error| {
        LocalProjectError::with_path_span("E_PROJECT_START_DECLARATION", &root_path,
            error.message, (error.span.start, error.span.end))
    })?;
    let mut module_paths = BTreeMap::new();
    let mut module_sources = BTreeMap::new();
    let mut owned_paths = BTreeSet::new();
    for module in declarations {
        let resolve = |raw: &str| {
            let relative = normalize_project_reference(raw)?;
            let base = parent_dir(&root_path);
            normalize_project_path(&if base.is_empty() { relative } else { format!("{base}/{relative}") })
        };
        let head = resolve(&module.head)?;
        let mut closure = Vec::new();
        for raw in module.sources {
            let path = resolve(raw.trim_end_matches('/'))?;
            let expanded = if text_by_path.contains_key(&path) { vec![path.clone()] } else {
                let prefix = format!("{path}/");
                let matches = role_by_path.iter().filter(|(candidate, role)|
                    candidate.starts_with(&prefix) && role.as_str() == "ddn_source")
                    .map(|(candidate, _)| candidate.clone()).collect::<Vec<_>>();
                if matches.is_empty() {
                    return Err(LocalProjectError::with_path("E_PROJECT_MODULE_SOURCE_MISSING", &path,
                        "선언한 원문 파일 또는 폴더 안의 원문이 입력 집합에 없습니다"));
                }
                matches
            };
            for path in expanded {
                if closure.contains(&path) {
                    return Err(LocalProjectError::with_path("E_PROJECT_MODULE_SOURCE_DUPLICATE", &path,
                        "모듈 원문 집합이 겹치거나 중복 지정되어 있습니다"));
                }
                closure.push(path);
            }
        }
        if !closure.contains(&head) {
            return Err(LocalProjectError::with_path("E_PROJECT_HEAD_OUTSIDE_SOURCE_CLOSURE", &head,
                "모듈 머리는 명시한 원문 집합에 포함되어야 합니다"));
        }
        closure.sort();
        for path in &closure {
            if path == &root_path || !text_by_path.contains_key(path) {
                return Err(LocalProjectError::with_path("E_PROJECT_MODULE_SOURCE_MISSING", path,
                    "모듈 머리와 원문은 닫힌 입력 집합의 원문 파일이어야 합니다"));
            }
            if !owned_paths.insert(path.clone()) {
                return Err(LocalProjectError::with_path("E_PROJECT_MODULE_SOURCE_MULTIPLE_OWNERS", path,
                    "같은 원문 파일을 둘 이상의 모듈이 소유할 수 없습니다"));
            }
        }
        module_paths.insert(module.name.clone(), head);
        module_sources.insert(module.name, closure);
    }
    for (path, role) in &role_by_path {
        if path != &root_path && role == "ddn_source" && !owned_paths.contains(path) {
            return Err(LocalProjectError::with_path("E_PROJECT_UNDECLARED_SOURCE", path,
                "입력 원문이 얽음의 모듬에 선언되어 있지 않습니다"));
        }
    }
    let (entry_path, start_definition) = match start {
        Some(start) => {
            let head = module_paths.get(&start.module).ok_or_else(|| {
                LocalProjectError::with_path("E_PROJECT_START_MODULE_MISSING", &root_path,
                    "시작 모듈이 모듬에 선언되어 있지 않습니다")
            })?;
            (head.clone(), Some(start.definition))
        }
        None => (String::new(), None),
    };
    if !entry_path.is_empty() && entry_path != entry_from_source {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ENTRY_CONFLICT",
            root_path.clone(),
            format!("호출의 entry({entry_from_source})와 얽음의 시작 머리({entry_path})가 다릅니다"),
        ));
    }
    if !entry_path.is_empty() && !text_by_path.contains_key(&entry_path) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ENTRY_MISSING",
            entry_path.clone(),
            "프로젝트 첫글 파일이 없습니다",
        ));
    }
    let project_name = root_path.clone(); // Display only; no source authority.
    let mut targets = BTreeMap::new();
    if let Some(definition) = &start_definition { targets.insert(definition.clone(), entry_path.clone()); }
    let gaji_aliases = BTreeMap::new();
    let files = normalized_manifest.files.clone();
    let hash_input = serde_json::to_string(&json!({
        "schema": "ddn.local_project_graph.hash_input.v1",
        "project_name": project_name,
        "root_path": root_path,
        "entry_path": entry_path,
        "targets": targets,
        "module_paths": module_paths,
        "gaji_aliases": gaji_aliases,
        "module_sources": module_sources,
        "start_definition": start_definition,
        "files": files,
    }))
    .expect("graph hash input serialization");
    let project_graph_sha256 = sha256_hex(hash_input.as_bytes());
    Ok(LocalProjectGraph {
        schema: PROJECT_GRAPH_SCHEMA,
        project_name,
        root_path,
        entry_path,
        targets,
        module_paths,
        gaji_aliases,
        module_sources,
        start_definition,
        file_count: files.len(),
        files,
        project_graph_sha256,
        diagnostics: Vec::new(),
    })
}

pub fn build_project_symbol_boundary(
    source: VirtualProjectSource,
) -> Result<LocalProjectSymbolBoundary, LocalProjectError> {
    build_linked_project(source, None).map(|linked| linked.symbol_boundary)
}

pub(crate) fn validate_project_module_boundary_for_authoring(
    source: VirtualProjectSource,
) -> Result<(), LocalProjectError> {
    build_linked_project_with_options(source, None, false, None).map(|_| ())
}

/// Bounded authored preparation reuses the product linker without execution.
/// Its byte map is private provenance, not a new public source-map wire.
#[derive(Debug)]
pub(crate) struct AuthoredPreparation {
    pub source: String,
    pub target_path: String,
    pub start_definition: Option<String>,
    source_map: Vec<LinkedSourceSegment>,
    imported_definitions: BTreeSet<(String,String,String)>,
}

impl AuthoredPreparation {
    pub fn permits_import(&self,importer_head:&str,owner_head:&str,name:&str)->bool {
        self.imported_definitions.contains(&(importer_head.into(),owner_head.into(),name.into()))
    }
    pub fn original_span(&self, span: Span) -> Result<(String, Span), String> {
        let segment = self.source_map.iter().find(|segment|
            span.start >= segment.generated_start && span.end <= segment.generated_end)
            .ok_or_else(|| "E_AUTHORED_BOGAE_SOURCE_MAP: 연결 정의의 원래 소유 원문을 해소할 수 없습니다".to_string())?;
        let start = segment.source_boundaries.get(span.start-segment.generated_start).copied();
        let end = segment.source_boundaries.get(span.end-segment.generated_start).copied();
        match (start,end) {
            (Some(start),Some(end)) if start <= end => Ok((segment.source_path.clone(),Span::new(start,end))),
            _ => Err("E_AUTHORED_BOGAE_SOURCE_MAP: 연결 정의의 원래 byte 범위를 해소할 수 없습니다".into()),
        }
    }
}

pub(crate) fn prepare_authored_execution(source: VirtualProjectSource)
    -> Result<AuthoredPreparation,LocalProjectError> {
    let linked = build_linked_project_with_options(source,None,false,None)?;
    let mut imported_definitions=BTreeSet::new();
    for symbol in &linked.symbol_boundary.resolved_symbols {
        for import in &linked.symbol_boundary.imports {
            if import.alias==symbol.alias && import.resolved_path==symbol.module_path {
                imported_definitions.insert((import.importer_path.clone(),symbol.module_path.clone(),symbol.internal_name.clone()));
            }
        }
    }
    // The linker has just checked this exact source's graph. Its metadata is
    // data, not execution admission; do not build the same graph a second time.
    Ok(AuthoredPreparation { source:linked.execution_source,target_path:linked.symbol_boundary.entry_path,
        start_definition:linked.start_definition,source_map:linked.execution_source_map,imported_definitions })
}

mod project_linking;
use project_linking::*;

#[derive(Debug, Clone)]
struct LinkedExecutionSource {
    text: String,
    source_map: Vec<LinkedSourceSegment>,
}

mod source_resolution;
use source_resolution::*;
pub use source_resolution::{build_project_gaji_lock, build_project_prepare_link, build_project_run_summary, build_project_run_summary_with_supported_profile, build_project_run_summary_with_supported_profile_and_host_resources, build_project_run_summary_with_supported_profile_and_prepared_host_resources, discover_virtual_project_root, discover_disk_project_root, build_virtual_project_source_from_disk, normalize_virtual_project_source, normalize_project_path};
pub(crate) use source_resolution::{mask_module_metadata_blocks, project_asset_registry_from_source, project_asset_registry_from_json, find_named_block_key_spans};

fn collect_gaji_dependencies(
    path: &str,
    text_by_path: &BTreeMap<String, String>,
    manifests: &mut BTreeMap<String, ParsedGajiManifest>,
    stack: &mut Vec<String>,
) -> Result<(), LocalProjectError> {
    let manifest = load_gaji_manifest_for_path(path, text_by_path)?;
    let id = manifest.id.clone();
    if stack.iter().any(|item| item == &id) {
        return Err(LocalProjectError::with_path(
            "E_GAJI_DEP_CYCLE",
            manifest.manifest_path,
            format!("가지 의존성 순환이 있습니다: {}", stack.join(" -> ")),
        ));
    }
    let dependencies = manifest.dependencies.clone();
    manifests.entry(id.clone()).or_insert(manifest);
    stack.push(id.clone());
    for (dep_id, pinned_version) in dependencies {
        if pinned_version.trim().is_empty() || pinned_version == "*" {
            return Err(LocalProjectError::with_path(
                "E_GAJI_DEP_UNPINNED",
                gaji_manifest_path_for_id(&dep_id),
                format!("{id}의 dependency {dep_id}가 exact version pin이 아닙니다"),
            ));
        }
        let dep_path = gaji_vendor_path_for_id(&dep_id);
        let dep_manifest = load_gaji_manifest_for_path(&dep_path, text_by_path).map_err(|err| {
            if err.code == "E_GAJI_MANIFEST_MISSING" {
                LocalProjectError::with_path(
                    "E_GAJI_TRANSITIVE_MISSING",
                    gaji_manifest_path_for_id(&dep_id),
                    format!("{id}가 요구하는 transitive 가지 {dep_id}가 없습니다"),
                )
            } else {
                err
            }
        })?;
        if dep_manifest.version != pinned_version {
            return Err(LocalProjectError::with_path(
                "E_GAJI_VERSION_PIN_MISMATCH",
                dep_manifest.manifest_path.clone(),
                format!(
                    "{id}가 {dep_id}@{pinned_version}을 요구하지만 실제 버전은 {}입니다",
                    dep_manifest.version
                ),
            ));
        }
        collect_gaji_dependencies(&dep_path, text_by_path, manifests, stack)?;
    }
    stack.pop();
    Ok(())
}

fn parse_gaji_manifest_text(
    manifest_path: &str,
    text: &str,
) -> Result<ParsedGajiManifest, LocalProjectError> {
    let mut id = None;
    let mut version = None;
    let mut dependencies = BTreeMap::new();
    let mut in_dependencies = false;
    for raw_line in text.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim().trim_matches('"');
        let value = raw_value.trim().trim_end_matches(',').trim();
        if in_dependencies {
            let Some(version_value) = strip_toml_string(value) else {
                return Err(LocalProjectError::with_path(
                    "E_GAJI_DEP_UNPINNED",
                    manifest_path,
                    format!("dependency {key}는 exact string version pin이어야 합니다"),
                ));
            };
            dependencies.insert(key.to_string(), version_value.to_string());
            continue;
        }
        match key {
            "id" => {
                if let Some(value) = strip_toml_string(value) {
                    id = Some(value.to_string());
                }
            }
            "version" => {
                if let Some(value) = strip_toml_string(value) {
                    version = Some(value.to_string());
                }
            }
            "dependencies" => {
                parse_inline_dependency_table(manifest_path, value, &mut dependencies)?;
            }
            _ => {}
        }
    }
    let id = id.ok_or_else(|| {
        LocalProjectError::with_path(
            "E_GAJI_ID_MISSING",
            manifest_path,
            "gaji.toml에 id가 없습니다",
        )
    })?;
    let version = version.ok_or_else(|| {
        LocalProjectError::with_path(
            "E_GAJI_VERSION_MISSING",
            manifest_path,
            "gaji.toml에 version이 없습니다",
        )
    })?;
    Ok(ParsedGajiManifest {
        id,
        version,
        dependencies,
        manifest_path: manifest_path.to_string(),
        manifest_sha256: sha256_hex(text.as_bytes()),
        package_sha256: String::new(),
    })
}

fn parse_inline_dependency_table(
    manifest_path: &str,
    value: &str,
    out: &mut BTreeMap<String, String>,
) -> Result<(), LocalProjectError> {
    let trimmed = value.trim();
    if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return Err(LocalProjectError::with_path(
            "E_GAJI_DEP_UNPINNED",
            manifest_path,
            "dependencies는 { \"id\" = \"version\" } exact pin table이어야 합니다",
        ));
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    for raw_pair in inner.split(',') {
        let pair = raw_pair.trim();
        if pair.is_empty() {
            continue;
        }
        let Some((raw_key, raw_value)) = pair.split_once('=') else {
            return Err(LocalProjectError::with_path(
                "E_GAJI_DEP_UNPINNED",
                manifest_path,
                format!("dependency pin 형식이 아닙니다: {pair}"),
            ));
        };
        let key = raw_key.trim().trim_matches('"');
        let Some(version) = strip_toml_string(raw_value.trim()) else {
            return Err(LocalProjectError::with_path(
                "E_GAJI_DEP_UNPINNED",
                manifest_path,
                format!("dependency {key}가 exact string version pin이 아닙니다"),
            ));
        };
        out.insert(key.to_string(), version.to_string());
    }
    Ok(())
}

fn strip_toml_string(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        Some(&trimmed[1..trimmed.len() - 1])
    } else {
        None
    }
}

fn gaji_id_from_vendor_path(path: &str) -> Result<&str, LocalProjectError> {
    let Some(id) = path.strip_prefix("vendor/") else {
        return Err(LocalProjectError::with_path(
            "E_GAJI_VENDOR_PATH",
            path,
            "local 가지는 vendor/<id> 경로에 있어야 합니다",
        ));
    };
    if id.is_empty() || id.contains('/') {
        return Err(LocalProjectError::with_path(
            "E_GAJI_VENDOR_PATH",
            path,
            "Wave 1 local 가지 id는 vendor/<id> 한 단계로 고정합니다",
        ));
    }
    Ok(id)
}

fn gaji_vendor_path_for_id(id: &str) -> String {
    format!("vendor/{id}")
}

fn gaji_manifest_path_for_id(id: &str) -> String {
    format!("vendor/{id}/gaji.toml")
}

fn gaji_package_sha256(prefix: &str, text_by_path: &BTreeMap<String, String>) -> String {
    let mut input = String::new();
    let package_prefix = format!("{prefix}/");
    for (path, text) in text_by_path {
        if path == &format!("{prefix}/gaji.toml") || path.starts_with(&package_prefix) {
            input.push_str(path);
            input.push('\t');
            input.push_str(&sha256_hex(text.as_bytes()));
            input.push('\n');
        }
    }
    sha256_hex(input.as_bytes())
}

#[derive(Debug, Clone)]
struct CurrentModuleDirectives {
    imports: Vec<ParsedImport>,
    exports: Vec<ParsedExport>,
}

fn parse_unlinked_work_module(
    source: &str, path: &str,
) -> Result<ddonirang_lang::CanonProgram, ddonirang_lang::ParseError> {
    let tokens = Lexer::new(source).tokenize().map_err(|error| ddonirang_lang::ParseError {
        span: Span::new(error.pos, error.pos + 1), message: error.message,
    })?;
    ddonirang_lang::Parser::new_v25_with_mode(tokens, ParseMode::Strict)
        .parse_unlinked_module(source.to_string(), path.to_string())
}

pub(crate) fn resolve_work_import_source_path(
    source: &VirtualProjectSource, module_name: &str,
) -> Result<String, LocalProjectError> {
    build_project_graph(source.clone())?.module_paths.get(module_name).cloned()
        .ok_or_else(|| LocalProjectError::new("E_MODULE_MISSING",
            format!("모듬에 선언하지 않은 모듈입니다: {module_name}")))
}

fn is_model_recipe_dependency(source: &VirtualProjectSource, graph: &LocalProjectGraph,
    target: &str) -> Result<bool, LocalProjectError> {
    for (module, paths) in &graph.module_sources {
        if !paths.iter().any(|path| source.files.iter().any(|file|
            file.path == *path && ddonirang_lang::has_model_surface(&file.utf8_text))) { continue; }
        let head = &graph.module_paths[module];
        // 그래프 경로는 정규화되지만 여기서는 원래 파일 레코드를 읽는다.
        // 비정규 머리 경로로 원문을 찾지 못해도 패닉 대신 기존 오류를 반환한다.
        let text = &source.files.iter().find(|file| file.path == *head)
            .ok_or_else(|| LocalProjectError::with_path(
                "E_PROJECT_MODULE_SOURCE_MISSING", head,
                "정규 모듈 머리와 일치하는 원문 파일이 없습니다"))?.utf8_text;
        let tokens = Lexer::new(text).tokenize().map_err(|error|
            LocalProjectError::with_path("E_PROJECT_MODULE_PARSE", head, error.message))?;
        // This pass only identifies an explicit named dependency. The normal
        // directive pass below still validates the true public definitions.
        let candidates = tokens.iter().filter_map(|token| match &token.kind {
            TokenKind::Ident(name) | TokenKind::Josa(name) => Some(name.clone()), _ => None });
        let context = ModuleDirectiveContext::default()
            .with_known_imports(graph.module_paths.keys().cloned()).with_public_definitions(candidates);
        if let Ok(prefix) = parse_module_directive_prefix(text, head, &context) {
            if prefix.imports.iter().flat_map(|directive| &directive.entries).any(|entry| entry.target == target) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub(crate) fn resolve_work_project_module_directives(
    source: &VirtualProjectSource, path: &str,
) -> Result<ProjectModuleDirectives, LocalProjectError> {
    let graph = build_project_graph(source.clone())?;
    let path = normalize_project_path(path)?;
    let (module, closure) = graph.module_sources.iter().find(|(_, paths)| paths.contains(&path))
        .ok_or_else(|| LocalProjectError::with_path("E_PROJECT_UNDECLARED_SOURCE", &path,
            "모듈에 선언하지 않은 원문입니다"))?;
    let texts = source.files.iter().map(|file| Ok((normalize_project_path(&file.path)?, &file.utf8_text)))
        .collect::<Result<BTreeMap<_, _>, LocalProjectError>>()?;
    let mut definitions = BTreeSet::new();
    for source_path in closure {
        let masked = mask_module_metadata_blocks(texts[source_path])?;
        if masked.trim().is_empty() { continue; }
        if ddonirang_lang::has_model_surface(&masked) {
            let model = ddonirang_lang::parse_model_program_with_source_file(&masked, source_path)
                .map_err(|error| LocalProjectError::with_path_span(error.candidate_code, source_path,
                    error.message, (error.span.start, error.span.end)))?;
            definitions.extend(model.ir.definitions.into_iter().map(|definition| definition.name));
            continue;
        }
        if is_model_recipe_dependency(source, &graph, module)? {
            let recipes = ddonirang_lang::parse_model_recipe_library_with_source_file(&masked, source_path)
                .map_err(|error| LocalProjectError::with_path_span(error.candidate_code, source_path,
                    error.message, (error.span.start, error.span.end)))?;
            definitions.extend(recipes.into_iter().map(|recipe| recipe.name));
            continue;
        }
        let program = parse_unlinked_work_module(&masked, source_path)
            .map_err(|error| LocalProjectError::with_path_span(
                if error.code() == "E_PARSE" { "E_PROJECT_MODULE_PARSE" } else { error.code() }, source_path,
                error.message, (error.span.start, error.span.end)))?;
        for item in program.items {
            let TopLevelItem::SeedDef(seed) = item;
            definitions.insert(seed.canonical_name);
            // The parser stores top-level declaration blocks in the synthetic
            // tick seed. Only those blocks (outside the authored seed span)
            // contribute module definitions, never ordinary callable locals.
            if let Some(body) = seed.body {
                for statement in body.stmts {
                    if let Stmt::DeclBlock { items, span, .. } = statement {
                        if span.start <= seed.span.start {
                            definitions.extend(items.into_iter().map(|item| item.name));
                        }
                    }
                }
            }
        }
        for statement in program.root_stmts {
            if let Stmt::Definition { name, .. } | Stmt::DerivedBinding {name,..} = statement { definitions.insert(name); }
            else if let Stmt::DeclBlock { items, .. } = statement {
                definitions.extend(items.into_iter().map(|item| item.name));
            }
        }
    }
    let head = &graph.module_paths[module];
    let head_tokens = Lexer::new(texts[head]).tokenize().map_err(|error|
        LocalProjectError::with_path("E_PROJECT_MODULE_PARSE", head, error.message))?;
    let mut aliases = BTreeSet::new();
    let mut segment_start = 0;
    for (index, token) in head_tokens.iter().enumerate() {
        if matches!(&token.kind, TokenKind::Ident(name) | TokenKind::Josa(name) if name == "쓰임") {
            for pair in head_tokens[segment_start..index].windows(2) {
                if matches!(pair[1].kind, TokenKind::Equals) {
                    if let TokenKind::Ident(name) | TokenKind::Josa(name) = &pair[0].kind {
                        if !aliases.insert(name.clone()) {
                            return Err(LocalProjectError::with_path_span("E_IMPORT_ALIAS_DUPLICATE", head,
                                format!("쓰임 이름이 중복됩니다: {name}"), (pair[0].span.start, pair[0].span.end)));
                        }
                    }
                }
            }
            segment_start = index + 1;
        }
    }
    if !matches!(head_tokens.first().map(|token| &token.kind), Some(TokenKind::LParen)) {
        return Ok(ProjectModuleDirectives { imports: Vec::new(), exports: Vec::new(), typed_current: true });
    }
    let context = ModuleDirectiveContext::default()
        .with_known_imports(graph.module_paths.keys().cloned())
        .with_public_definitions(definitions);
    let parsed = parse_module_directive_prefix(texts[head], head, &context)
        .map_err(|error| {
            let code = if error.message.contains("E_RD457_IMPORT_TARGET_UNKNOWN") { "E_MODULE_MISSING" }
                else if error.message.contains("DUPLICATE") { "E_IMPORT_ALIAS_DUPLICATE" }
                else { "E_CURRENT_MODULE_DIRECTIVE" };
            let span = if matches!(code, "E_MODULE_MISSING" | "E_IMPORT_ALIAS_DUPLICATE") {
                head_tokens.windows(2).filter(|pair|
                    pair[0].span.start >= error.span.start && pair[0].span.end <= error.span.end
                    && matches!(pair[1].kind, TokenKind::Equals)
                    && matches!(pair[0].kind, TokenKind::Ident(_) | TokenKind::Josa(_)))
                    .last().map(|pair| (pair[0].span.start, pair[0].span.end))
                    .unwrap_or((error.span.start, error.span.end))
            } else { (error.span.start, error.span.end) };
            LocalProjectError::with_path_span(code, head, error.message, span)
        })?;
    Ok(ProjectModuleDirectives {
        imports: parsed.imports.into_iter().flat_map(|directive| directive.entries)
            .map(|entry| ParsedImport { alias: entry.local_name, source: entry.target }).collect(),
        exports: parsed.exports.into_iter().flat_map(|directive| directive.entries)
            .map(|entry| ParsedExport { external_name: entry.local_name, internal_name: entry.target }).collect(),
        typed_current: true,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectModuleDirectives {
    pub imports: Vec<ParsedImport>,
    pub exports: Vec<ParsedExport>,
    pub typed_current: bool,
}

/// The current supported Project frontdoor must reject retired L11 brace
/// declarations before graph/link processing can interpret them as metadata.
/// This token-level check shares the parser's lexical boundary: strings and
/// comments have no `Ident + LBrace` token pair, so they remain ordinary data.
fn validate_current_project_sources_before_link(
    source: &VirtualProjectSource,
) -> Result<(), LocalProjectError> {
    for file in &source.files {
        if file.role != "ddn_source" {
            continue;
        }
        reject_source_bom(&file.utf8_text, Some(&file.path))?;
        let tokens = Lexer::new(&file.utf8_text).tokenize().map_err(|error| {
            LocalProjectError::with_path(
                "E_PROJECT_ENTRY_PARSE",
                file.path.clone(),
                format!("current module source 토큰화 실패: {}", error.message),
            )
        })?;
        for pair in tokens.windows(2) {
            let legacy_name = match &pair[0].kind {
                TokenKind::Ident(name) | TokenKind::Josa(name) => {
                    matches!(name.as_str(), "쓰임" | "드러냄" | "공개")
                }
                _ => false,
            };
            if legacy_name && matches!(pair[1].kind, TokenKind::LBrace) {
                return Err(LocalProjectError::with_path_span(
                    "E_V25_DEFINITION_MIGRATION_REQUIRED",
                    file.path.clone(),
                    "V25에서는 이전 brace형 L11 선언을 사용할 수 없습니다. `(<항목>=value) 쓰임.` 또는 `(<항목>=value) 드러냄.`처럼 현재 typed directive를 사용하세요",
                    (pair[0].span.start, pair[1].span.end),
                ));
            }
        }
    }
    Ok(())
}

fn current_module_directives(
    text: &str,
    path: &str,
    text_by_path: &BTreeMap<String, String>,
) -> Result<Option<CurrentModuleDirectives>, LocalProjectError> {
    reject_source_bom(text, Some(path))?;
    // A current module directive always begins with its parenthesized typed
    // surface.  Other project custody (including historical brace inputs)
    // stays on its existing explicit parser path; it is never a fallback for
    // a current typed document.
    let tokens = Lexer::new(text).tokenize().map_err(|error| {
        LocalProjectError::with_path(
            "E_PROJECT_ENTRY_PARSE",
            path,
            format!("current module source 토큰화 실패: {}", error.message),
        )
    })?;
    if !matches!(
        tokens.first().map(|token| &token.kind),
        Some(TokenKind::LParen)
    ) {
        return Ok(None);
    }

    let mut known_import_targets = BTreeSet::new();
    for candidate_path in text_by_path.keys() {
        let stem = candidate_path
            .strip_suffix(".ddn")
            .unwrap_or(candidate_path);
        known_import_targets.insert(stem.to_string());
        known_import_targets.insert(candidate_path.to_string());
        known_import_targets.insert(format!("./{stem}"));
        known_import_targets.insert(format!("./{candidate_path}"));
    }

    // The shared typed parser owns directive syntax and spans.  Its registry
    // preflight receives lexer-derived candidates only so the parser can read
    // the declaration tail; actual export authority is then checked against
    // the typed declaration AST below, rather than by a local string parser.
    let mut lexical_candidates = BTreeSet::new();
    for token in tokens {
        if let TokenKind::Ident(name) | TokenKind::Josa(name) = token.kind {
            lexical_candidates.insert(name);
        }
    }
    let context = ModuleDirectiveContext::default()
        .with_known_imports(known_import_targets)
        .with_public_definitions(lexical_candidates);
    let parsed = parse_module_directive_prefix(text, path, &context).map_err(|error| {
        LocalProjectError::with_path_span(
            "E_CURRENT_MODULE_DIRECTIVE",
            path,
            error.message,
            (error.span.start, error.span.end),
        )
    })?;

    let declared_names = if parsed.exports.is_some() {
        let program = ddonirang_lang::parse_v25_with_mode(&parsed.declaration_source, path, ParseMode::Strict)
            .map_err(|error| {
                LocalProjectError::with_path_span(
                    "E_CURRENT_MODULE_DECLARATION",
                    path,
                    error.message,
                    (error.span.start, error.span.end),
                )
            })?;
        program.items
            .iter()
            .map(|item| match item {
                TopLevelItem::SeedDef(seed) => seed.canonical_name.clone(),
            })
            .chain(program.root_stmts.iter().filter_map(|stmt| match stmt {
                Stmt::Definition { name, .. } => Some(name.clone()),
                // Binding/rebinding is not a new exported definition.
                _ => None,
            }))
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let imports = parsed
        .imports
        .as_ref()
        .map(|directive| {
            directive
                .entries
                .iter()
                .map(|entry| ParsedImport {
                    alias: entry.local_name.clone(),
                    source: entry.target.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    let exports = parsed
        .exports
        .as_ref()
        .map(|directive| {
            directive
                .entries
                .iter()
                .map(|entry| {
                    if !declared_names.contains(&entry.target) {
                        return Err(LocalProjectError::with_path(
                            "E_RD457_EXPORT_TARGET_PRIVATE_OR_UNKNOWN",
                            path,
                            format!(
                                "{}는 이 module의 typed DefinitionIdentity가 아닙니다",
                                entry.target
                            ),
                        ));
                    }
                    Ok(ParsedExport {
                        external_name: entry.local_name.clone(),
                        internal_name: entry.target.clone(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(Some(CurrentModuleDirectives { imports, exports }))
}

/// Resolve a module's import/export metadata through the current typed
/// directive owner when the source begins with the current parenthesized
/// surface. Historical block custody remains explicit and never becomes a
/// fallback for a typed document.
pub(crate) fn resolve_project_module_directives(
    text: &str,
    path: &str,
    text_by_path: &BTreeMap<String, String>,
) -> Result<ProjectModuleDirectives, LocalProjectError> {
    match current_module_directives(text, path, text_by_path)? {
        Some(current) => Ok(ProjectModuleDirectives {
            imports: current.imports,
            exports: current.exports,
            typed_current: true,
        }),
        None => Ok(ProjectModuleDirectives {
            imports: parse_imports(text, path)?,
            exports: parse_exports(text, path)?,
            typed_current: false,
        }),
    }
}

pub(crate) fn parse_imports(
    text: &str,
    path: &str,
) -> Result<Vec<ParsedImport>, LocalProjectError> {
    let pairs = parse_string_pair_list_from_named_block(text, "쓰임")?;
    let mut imports = Vec::new();
    for (alias, source) in pairs {
        if alias.contains('.') || alias.trim().is_empty() {
            return Err(LocalProjectError::with_path(
                "E_IMPORT_ALIAS_INVALID",
                path,
                format!("지원하지 않는 쓰임 alias입니다: {alias}"),
            ));
        }
        imports.push(ParsedImport { alias, source });
    }
    Ok(imports)
}

pub(crate) fn parse_exports(
    text: &str,
    path: &str,
) -> Result<Vec<ParsedExport>, LocalProjectError> {
    let Some(block) =
        extract_named_block(text, "드러냄").or_else(|| extract_named_block(text, "공개"))
    else {
        return Ok(Vec::new());
    };
    let mut exports = Vec::new();
    for raw_line in block.lines() {
        let line = raw_line.trim().trim_end_matches('.');
        if line.is_empty() {
            continue;
        }
        if let Some((external, internal)) = line.split_once(':') {
            let external_name = external.trim().to_string();
            let internal_name = internal.trim().to_string();
            if external_name.is_empty() || internal_name.is_empty() {
                return Err(LocalProjectError::with_path(
                    "E_EXPORT_UNSUPPORTED",
                    path,
                    "드러냄 항목은 이름 또는 외부: 내부 형식이어야 합니다",
                ));
            }
            exports.push(ParsedExport {
                external_name,
                internal_name,
            });
        } else {
            let name = line.trim().to_string();
            if name.is_empty() {
                continue;
            }
            exports.push(ParsedExport {
                external_name: name.clone(),
                internal_name: name,
            });
        }
    }
    Ok(exports)
}

fn collect_alias_show_requests_from_ast(
    text: &str,
    path: &str,
) -> Result<Vec<AliasShowRequest>, LocalProjectError> {
    let masked = mask_module_metadata_blocks(text)?;
    if masked.trim().is_empty() {
        return Ok(Vec::new());
    }
    if ddonirang_lang::has_model_surface(&masked) {
        runtime_surface::execution_envelope_from_source_with_profile(
            &masked, path, Some(1),
            Some(crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY),
        )
            .map_err(LocalProjectError::from_execution_diagnostic)?;
        return Ok(Vec::new());
    }
    // Import projection discovery must not execute call binding before the
    // linker has supplied imported callable definitions. The shared lexer is
    // sufficient for the exact `alias.symbol 보여주기` surface and preserves
    // original byte spans for later linker diagnostics.
    collect_alias_show_requests_from_tokens(&masked)
}

fn collect_alias_show_requests_from_tokens(
    text: &str,
) -> Result<Vec<AliasShowRequest>, LocalProjectError> {
    let tokens = Lexer::new(text).tokenize().map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_ENTRY_PARSE",
            format!("entry DDN 토큰화 실패: {}", err.message),
        )
    })?;
    let mut requests = Vec::new();
    for window in tokens.windows(4) {
        let [left, dot, right, show] = window else {
            continue;
        };
        if !matches!(dot.kind, TokenKind::Dot) || !matches!(show.kind, TokenKind::KwBoyeojugi) {
            continue;
        }
        let Some(alias) = token_name(left) else {
            continue;
        };
        let Some(symbol) = token_name(right) else {
            continue;
        };
        if !is_whitespace_only(text, left.span.end, dot.span.start)
            || !is_whitespace_only(text, dot.span.end, right.span.start)
            || !is_whitespace_only(text, right.span.end, show.span.start)
            || text[left.span.end..show.span.start].contains('\n')
        {
            continue;
        }
        requests.push(AliasShowRequest {
            alias: alias.to_string(),
            symbol: symbol.to_string(),
            source_span: Some((left.span.start, right.span.end)),
        });
    }
    Ok(requests)
}

fn collect_alias_show_requests_from_body(
    body: &Body,
    requests: &mut Vec<AliasShowRequest>,
) -> Result<(), LocalProjectError> {
    for stmt in &body.stmts {
        match stmt {
            Stmt::SumDefinition { .. } => {}
            Stmt::SumMatch { arms, default_body, .. } => {
                for arm in arms {
                    collect_alias_show_requests_from_body(&arm.body, requests)?;
                }
                if let Some(body) = default_body {
                    collect_alias_show_requests_from_body(body, requests)?;
                }
            }
            Stmt::Show { expr, .. } => {
                if let Some(request) = alias_show_request_from_expr(expr)? {
                    requests.push(request);
                }
            }
            Stmt::Receive { body, .. }
            | Stmt::Try { body, .. }
            | Stmt::Repeat { body, .. }
            | Stmt::While { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::Quantifier { body, .. }
            | Stmt::Guard { body, .. }
            | Stmt::BeatBlock { body, .. }
            | Stmt::Hook { body, .. }
            | Stmt::HookWhenBecomes { body, .. }
            | Stmt::HookWhile { body, .. } => {
                collect_alias_show_requests_from_body(body, requests)?;
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_alias_show_requests_from_body(then_body, requests)?;
                if let Some(else_body) = else_body {
                    collect_alias_show_requests_from_body(else_body, requests)?;
                }
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_alias_show_requests_from_body(&branch.body, requests)?;
                }
                collect_alias_show_requests_from_body(else_body, requests)?;
            }
            Stmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                if let Some(then_body) = then_body {
                    collect_alias_show_requests_from_body(then_body, requests)?;
                }
                collect_alias_show_requests_from_body(else_body, requests)?;
            }
            Stmt::DeclBlock { .. }
            | Stmt::Definition { .. }
            | Stmt::DerivedBinding {..}
            | Stmt::Mutate { .. }
            | Stmt::ResultFlow { .. }
            | Stmt::Expr { .. }
            | Stmt::Send { .. }
            | Stmt::Inspect { .. }
            | Stmt::MetaBlock { .. }
            | Stmt::PublicObservation { .. }
            | Stmt::Pragma { .. }
            | Stmt::Return { .. }
            | Stmt::Break { .. }
            | Stmt::ContinueLoop { .. } => {}
        }
    }
    Ok(())
}

fn collect_alias_symbol_requests_from_tokens(
    text: &str,
) -> Result<Vec<AliasShowRequest>, LocalProjectError> {
    let masked = mask_module_metadata_blocks(text)?;
    let tokens = Lexer::new(&masked).tokenize().map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_ENTRY_PARSE",
            format!("entry DDN 토큰화 실패: {}", err.message),
        )
    })?;
    let mut requests = Vec::new();
    for window in tokens.windows(3) {
        let [left, dot, right] = window else {
            continue;
        };
        if !matches!(dot.kind, TokenKind::Dot) {
            continue;
        }
        let Some(alias) = token_name(left) else {
            continue;
        };
        let Some(symbol) = token_name(right) else {
            continue;
        };
        if !is_whitespace_only(&masked, left.span.end, dot.span.start)
            || !is_whitespace_only(&masked, dot.span.end, right.span.start)
        {
            continue;
        }
        if !requests
            .iter()
            .any(|item: &AliasShowRequest| item.alias == alias && item.symbol == symbol)
        {
            requests.push(AliasShowRequest {
                alias: alias.to_string(),
                symbol: symbol.to_string(),
                source_span: Some((left.span.start, right.span.end)),
            });
        }
    }
    Ok(requests)
}

fn alias_show_request_from_expr(
    expr: &Expr,
) -> Result<Option<AliasShowRequest>, LocalProjectError> {
    let ExprKind::FieldAccess { target, field, .. } = &expr.kind else {
        return Ok(None);
    };
    match &target.kind {
        ExprKind::Var(alias) => Ok(Some(AliasShowRequest {
            alias: alias.clone(),
            symbol: field.clone(),
            source_span: Some((expr.span.start, expr.span.end)),
        })),
        ExprKind::FieldAccess { .. } => Err(LocalProjectError::new(
            "E_PROJECT_SYMBOL_UNSUPPORTED",
            "중첩 alias-qualified 보여주기 대상은 Wave 1에서 지원하지 않습니다",
        )),
        _ => Ok(None),
    }
}

fn has_top_level_show_effect(text: &str) -> bool {
    text.lines()
        .map(str::trim)
        .any(|line| line.trim_end_matches('.').ends_with("보여주기"))
}

fn collect_module_info(
    path: &str,
    entry_path: &str,
    text_by_path: &BTreeMap<String, String>,
    modules: &mut BTreeMap<String, ModuleInfo>,
    stack: &mut Vec<String>,
) -> Result<(), LocalProjectError> {
    if modules.contains_key(path) {
        return Ok(());
    }
    if stack.iter().any(|item| item == path) {
        return Err(LocalProjectError::with_path(
            "E_IMPORT_CYCLE",
            path,
            format!("모듈 순환이 있습니다: {}", stack.join(" -> ")),
        ));
    }
    let text = text_by_path.get(path).ok_or_else(|| {
        LocalProjectError::with_path("E_MODULE_MISSING", path, "모듈 파일이 없습니다")
    })?;
    if path != entry_path && has_top_level_show_effect(text) {
        return Err(LocalProjectError::with_path(
            "E_IMPORTED_TOP_LEVEL_EFFECT_UNSUPPORTED",
            path,
            "imported module의 top-level 보여주기 효과는 Wave 1에서 지원하지 않습니다",
        ));
    }
    stack.push(path.to_string());
    let directives = resolve_project_module_directives(text, path, text_by_path)?;
    let parsed_imports = directives.imports;
    let mut seen_aliases = BTreeSet::new();
    let mut imports = Vec::new();
    for import in parsed_imports {
        if !seen_aliases.insert(import.alias.clone()) {
            let message = format!("중복 쓰임 alias입니다: {}", import.alias);
            return Err(find_named_block_key_spans(text, "쓰임", &import.alias)
                .into_iter()
                .last()
                .map(|span| {
                    LocalProjectError::with_path_span(
                        "E_IMPORT_ALIAS_DUPLICATE",
                        path,
                        message.clone(),
                        span,
                    )
                })
                .unwrap_or_else(|| {
                    LocalProjectError::with_path("E_IMPORT_ALIAS_DUPLICATE", path, message)
                }));
        }
        let resolved_path = resolve_import_source_path(path, &import.source)?;
        if !text_by_path.contains_key(&resolved_path) {
            let message = format!(
                "쓰임 alias {}의 모듈을 찾을 수 없습니다: {}",
                import.alias, resolved_path
            );
            return Err(find_named_block_key_spans(text, "쓰임", &import.alias)
                .into_iter()
                .last()
                .map(|span| {
                    LocalProjectError::with_path_span(
                        "E_MODULE_MISSING",
                        path,
                        message.clone(),
                        span,
                    )
                })
                .unwrap_or_else(|| {
                    LocalProjectError::with_path("E_MODULE_MISSING", path, message)
                }));
        }
        imports.push(ResolvedImport {
            importer_path: path.to_string(),
            alias: import.alias,
            source: import.source,
            resolved_path,
        });
    }
    imports.sort_by(|a, b| {
        a.alias
            .cmp(&b.alias)
            .then(a.resolved_path.cmp(&b.resolved_path))
    });
    let mut exports = BTreeMap::new();
    let parsed_exports = directives.exports;
    for export in parsed_exports {
        if exports
            .insert(export.external_name.clone(), export.internal_name.clone())
            .is_some()
        {
            let message = format!("중복 드러냄 이름입니다: {}", export.external_name);
            return Err(
                find_named_block_key_spans(text, "드러냄", &export.external_name)
                    .into_iter()
                    .last()
                    .or_else(|| {
                        find_named_block_key_spans(text, "공개", &export.external_name)
                            .into_iter()
                            .last()
                    })
                    .map(|span| {
                        LocalProjectError::with_path_span(
                            "E_EXPORT_DUPLICATE",
                            path,
                            message.clone(),
                            span,
                        )
                    })
                    .unwrap_or_else(|| {
                        LocalProjectError::with_path("E_EXPORT_DUPLICATE", path, message)
                    }),
            );
        }
    }
    modules.insert(
        path.to_string(),
        ModuleInfo {
            path: path.to_string(),
            imports: imports.clone(),
            exports,
        },
    );
    for import in imports {
        collect_module_info(
            &import.resolved_path,
            entry_path,
            text_by_path,
            modules,
            stack,
        )?;
    }
    stack.pop();
    Ok(())
}

fn resolve_imported_symbol_surface(
    modules: &BTreeMap<String, ModuleInfo>,
    module_path: &str,
    authored_symbol: &str,
    source_span: Option<(usize, usize)>,
) -> Result<(SymbolValue, String), LocalProjectError> {
    let module = modules.get(module_path).ok_or_else(|| {
        LocalProjectError::with_path("E_MODULE_MISSING", module_path, "모듈을 찾지 못했습니다")
    })?;
    let known_exports = module.exports.keys().cloned().collect::<HashSet<_>>();
    let span = source_span
        .map(|(start, end)| Span::new(start, end))
        .unwrap_or_else(|| Span::new(0, authored_symbol.len()));
    let (external_stem, canonical_call) =
        resolve_known_call_target(authored_symbol, &known_exports, span)
            .unwrap_or_else(|_| (authored_symbol.to_string(), authored_symbol.to_string()));
    let resolved = resolve_exported_symbol(modules, module_path, &external_stem, &mut Vec::new())?;
    let suffix = canonical_call
        .strip_prefix(&external_stem)
        .unwrap_or_default();
    let rewritten = format!("{}{}", resolved.internal_name, suffix);
    Ok((resolved, rewritten))
}

fn resolve_exported_symbol(
    modules: &BTreeMap<String, ModuleInfo>,
    module_path: &str,
    external_name: &str,
    stack: &mut Vec<String>,
) -> Result<SymbolValue, LocalProjectError> {
    let key = format!("{module_path}::{external_name}");
    if stack.iter().any(|item| item == &key) {
        return Err(LocalProjectError::with_path(
            "E_IMPORT_CYCLE",
            module_path,
            format!("re-export 순환이 있습니다: {}", stack.join(" -> ")),
        ));
    }
    stack.push(key);
    let module = modules.get(module_path).ok_or_else(|| {
        LocalProjectError::with_path("E_MODULE_MISSING", module_path, "모듈을 찾지 못했습니다")
    })?;
    let internal = module.exports.get(external_name).ok_or_else(|| {
        LocalProjectError::with_path(
            "E_SYMBOL_NOT_EXPORTED",
            module_path,
            format!("{external_name}는 드러냄 목록에 없습니다"),
        )
    })?;
    let result = if let Some((alias, symbol)) = split_alias_symbol(internal) {
        let import = module
            .imports
            .iter()
            .find(|item| item.alias == alias)
            .ok_or_else(|| {
                LocalProjectError::with_path(
                    "E_ALIAS_UNKNOWN",
                    module_path,
                    format!("re-export 내부 alias를 찾지 못했습니다: {alias}"),
                )
            })?;
        resolve_exported_symbol(modules, &import.resolved_path, symbol, stack)?
    } else {
        if !is_supported_internal_symbol_name(internal) {
            return Err(LocalProjectError::with_path(
                "E_SYMBOL_NOT_FOUND",
                module_path,
                format!("내부 심볼 {internal}를 Wave 1 linked 실행으로 확인할 수 없습니다"),
            ));
        }
        SymbolValue {
            module_path: module_path.to_string(),
            internal_name: internal.clone(),
        }
    };
    stack.pop();
    Ok(result)
}

fn is_supported_internal_symbol_name(name: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty()
        && trimmed == name
        && !trimmed.contains('.')
        && trimmed.chars().all(is_identifier_char)
}

fn split_alias_symbol(text: &str) -> Option<(&str, &str)> {
    let (alias, symbol) = text.split_once('.')?;
    if alias.is_empty() || symbol.is_empty() || symbol.contains('.') {
        return None;
    }
    Some((alias, symbol))
}

pub(crate) fn resolve_import_source_path(
    importer_path: &str,
    source: &str,
) -> Result<String, LocalProjectError> {
    let normalized_ref = normalize_project_reference(source)?;
    let _ = importer_path;
    if normalized_ref.ends_with(".ddn") {
        Ok(normalized_ref)
    } else {
        Ok(format!("{normalized_ref}.ddn"))
    }
}

fn virtual_parent_dirs(files: &BTreeSet<String>, start_path: &str) -> Vec<String> {
    let mut start_dir = if files.contains(start_path) {
        parent_dir(start_path)
    } else {
        start_path.to_string()
    };
    let prefix = format!("{start_path}/");
    if files.iter().any(|path| path.starts_with(&prefix)) {
        start_dir = start_path.to_string();
    }
    let mut dirs = Vec::new();
    let mut current = start_dir;
    loop {
        dirs.push(current.clone());
        if current.is_empty() {
            break;
        }
        current = parent_dir(&current);
    }
    dirs
}

fn root_result(
    root_marker_path: String,
    project_json_path: Option<String>,
) -> ProjectRootDiscoveryResult {
    let used_fallback = root_marker_path.is_empty() && project_json_path.is_some();
    ProjectRootDiscoveryResult {
        schema: PROJECT_ROOT_DISCOVERY_SCHEMA,
        root_kind: if used_fallback {
            "project_json_fallback"
        } else {
            "root_ddn"
        },
        root_marker_path: if root_marker_path.is_empty() {
            None
        } else {
            Some(root_marker_path)
        },
        project_json_path,
        used_fallback,
        diagnostics: Vec::new(),
    }
}

fn has_file(files: &BTreeSet<String>, dir: &str, name: &str) -> bool {
    files.contains(&join_project_path(dir, name))
}

fn legacy_in_dir<'a>(files: &BTreeSet<String>, dir: &str) -> Option<&'a str> {
    for legacy in LEGACY_ROOT_FILES {
        if files.contains(&join_project_path(dir, legacy)) {
            return Some(*legacy);
        }
    }
    None
}

fn join_project_path(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    }
}

fn parent_dir(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

fn path_file_name(path: &str) -> Option<&str> {
    path.rsplit('/').next()
}

fn graph_hash_input(root_path: &str, entry_path: &str, files: &[PendingFile]) -> String {
    let mut input = String::new();
    input.push_str("ddn.local_project.graph_hash.v1\n");
    input.push_str("root=");
    input.push_str(root_path);
    input.push('\n');
    input.push_str("entry=");
    input.push_str(entry_path);
    input.push('\n');
    for file in files {
        input.push_str(&file.path);
        input.push('\t');
        input.push_str(&file.role);
        input.push('\t');
        input.push_str(&file.sha256);
        input.push('\n');
    }
    input
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn to_canonical_json_line<T: Serialize>(value: &T) -> String {
    let mut text = serde_json::to_string(value).expect("manifest serialization must succeed");
    text.push('\n');
    text
}

#[cfg(test)]
mod tests;
