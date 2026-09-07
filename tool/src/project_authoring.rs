use std::collections::{BTreeMap, BTreeSet};

use ddonirang_lang::{
    has_model_surface, utf16_position, AtSuffix, Body, CanonProgram, DeclKind, Expr, ExprKind,
    Initialization, Lexer, Literal, ModelExpr, ModelProgram, ModelSpan, ParseMode, PrimaryRole,
    SeedKind, Span, Stmt, Token, TokenKind, TopLevelItem, TypeRef,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::local_project::{self, LocalProjectError, VirtualProjectSource};

pub const PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA: &str =
    "ddn.project_authoring_snapshot_request.v1";
pub const PROJECT_AUTHORING_SNAPSHOT_SCHEMA: &str = "ddn.project_authoring_snapshot.v1";
pub const PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA: &str = "ddn.project_authoring_cursor_request.v1";
pub const PROJECT_AUTHORING_NAVIGATION_SCHEMA: &str = "ddn.project_authoring_navigation.v1";
pub const PROJECT_AUTHORING_HOVER_SCHEMA: &str = "ddn.project_authoring_hover.v1";
pub const PROJECT_AUTHORING_COMPLETION_SCHEMA: &str = "ddn.project_authoring_completion.v1";
pub const PROJECT_AUTHORING_RENAME_REQUEST_SCHEMA: &str = "ddn.project_authoring_rename_request.v1";
pub const PROJECT_AUTHORING_RENAME_PLAN_SCHEMA: &str = "ddn.project_authoring_rename_plan.v1";
const FLAT_WRAPPER_PREFIX: &str = "매틱:움직씨 = {\n";
const FLAT_WRAPPER_SUFFIX: &str = "}\n";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAuthoringSnapshotRequest {
    pub schema: String,
    pub source: VirtualProjectSource,
    pub source_version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAuthoringCursorRequest {
    pub schema: String,
    pub source: VirtualProjectSource,
    pub source_version: String,
    pub expected_project_source_sha256: String,
    pub expected_snapshot_sha256: String,
    pub path: String,
    pub byte_offset: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAuthoringRenameRequest {
    pub schema: String,
    pub source: VirtualProjectSource,
    pub source_version: String,
    pub expected_project_source_sha256: String,
    pub expected_snapshot_sha256: String,
    pub path: String,
    pub byte_offset: usize,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectAuthoringSnapshot {
    pub schema: &'static str,
    pub source_version: String,
    pub project_source_sha256: String,
    pub project_graph_sha256: Option<String>,
    pub root_path: String,
    pub entry_path: String,
    pub files: Vec<AuthoringFileSnapshot>,
    pub imports: Vec<AuthoringImport>,
    pub exports: Vec<AuthoringExport>,
    pub scopes: Vec<AuthoringScope>,
    pub definitions: Vec<AuthoringDefinition>,
    pub references: Vec<AuthoringReference>,
    pub diagnostics: Vec<AuthoringDiagnostic>,
    pub snapshot_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringFileSnapshot {
    pub path: String,
    pub role: String,
    pub source_sha256: String,
    pub utf8_bytes: usize,
    pub parse_status: &'static str,
    pub parse_span_mode: &'static str,
    pub wrapper_prefix_bytes: usize,
    pub definition_count: usize,
    pub scope_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringSourceRange {
    pub start: usize,
    pub end: usize,
    pub start_line: usize,
    pub start_column_utf16: usize,
    pub end_line: usize,
    pub end_column_utf16: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringImport {
    pub id: String,
    pub importer_path: String,
    pub alias: String,
    pub source: String,
    pub resolved_path: String,
    pub alias_range: Option<AuthoringSourceRange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringExport {
    pub module_path: String,
    pub external_name: String,
    pub internal_name: String,
    pub name_range: Option<AuthoringSourceRange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringScope {
    pub id: String,
    pub path: String,
    pub parent_id: Option<String>,
    pub kind: &'static str,
    pub range: AuthoringSourceRange,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringDefinition {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub kind: &'static str,
    pub path: String,
    pub scope_id: String,
    pub range: AuthoringSourceRange,
    pub type_label: Option<String>,
    pub unit: Option<String>,
    pub value_label: Option<String>,
    pub constant: bool,
    pub explicit: bool,
    pub export_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringReference {
    pub name: String,
    pub normalized_name: String,
    pub path: String,
    pub scope_id: String,
    pub range: AuthoringSourceRange,
    pub definition_id: Option<String>,
    pub import_id: Option<String>,
    pub resolution: &'static str,
    pub write: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringDiagnostic {
    pub code: String,
    pub message: String,
    pub severity: &'static str,
    pub source_ref: Option<String>,
    pub source_range: Option<AuthoringSourceRange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectAuthoringNavigationResult {
    pub schema: &'static str,
    pub operation: &'static str,
    pub status: &'static str,
    pub source_version: String,
    pub project_source_sha256: String,
    pub snapshot_sha256: String,
    pub symbol_id: Option<String>,
    pub symbol_name: Option<String>,
    pub declaration: Option<AuthoringNavigationLocation>,
    pub references: Vec<AuthoringNavigationLocation>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringNavigationLocation {
    pub path: String,
    pub range: AuthoringSourceRange,
    pub name: String,
    pub kind: String,
    pub declaration: bool,
    pub write: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectAuthoringHoverResult {
    pub schema: &'static str,
    pub status: &'static str,
    pub source_version: String,
    pub project_source_sha256: String,
    pub snapshot_sha256: String,
    pub symbol_id: Option<String>,
    pub name: Option<String>,
    pub kind: Option<String>,
    pub type_label: Option<String>,
    pub unit: Option<String>,
    pub value_label: Option<String>,
    pub declaration: Option<AuthoringNavigationLocation>,
    pub import_source: Option<String>,
    pub evidence: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringCompletionItem {
    pub label: String,
    pub kind: String,
    pub origin: String,
    pub detail: String,
    pub insert_text: String,
    pub definition_id: Option<String>,
    pub conflict: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectAuthoringCompletionResult {
    pub schema: &'static str,
    pub status: &'static str,
    pub source_version: String,
    pub project_source_sha256: String,
    pub snapshot_sha256: String,
    pub path: String,
    pub replace_range: AuthoringSourceRange,
    pub prefix: String,
    pub items: Vec<AuthoringCompletionItem>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringRenameEdit {
    pub path: String,
    pub range: AuthoringSourceRange,
    pub old_text: String,
    pub new_text: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringRenameFilePatch {
    pub path: String,
    pub expected_source_sha256: String,
    pub edits: Vec<AuthoringRenameEdit>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectAuthoringRenamePlan {
    pub schema: &'static str,
    pub status: &'static str,
    pub source_version: String,
    pub project_source_sha256: String,
    pub snapshot_sha256: String,
    pub symbol_id: Option<String>,
    pub old_name: Option<String>,
    pub new_name: String,
    pub files: Vec<AuthoringRenameFilePatch>,
    pub affected_reference_count: usize,
    pub plan_sha256: String,
    pub message: String,
}

struct ParsedAuthoringFile {
    program: Option<CanonProgram>,
    tokens: Vec<Token>,
    parse_status: &'static str,
    parse_span_mode: &'static str,
    shift: usize,
    diagnostic: Option<AuthoringDiagnostic>,
}

#[derive(Debug, Clone)]
struct RawOccurrence {
    name: String,
    path: String,
    scope_id: String,
    start: usize,
    end: usize,
    write: bool,
}

pub fn build_project_authoring_snapshot_json(
    request_json: &str,
) -> Result<String, LocalProjectError> {
    let request: ProjectAuthoringSnapshotRequest =
        serde_json::from_str(request_json).map_err(|err| {
            LocalProjectError::new(
                "E_PROJECT_AUTHORING_REQUEST_JSON",
                format!("authoring snapshot 요청 JSON을 읽을 수 없습니다: {err}"),
            )
        })?;
    let snapshot = build_project_authoring_snapshot(request)?;
    serde_json::to_string(&snapshot)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|err| {
            LocalProjectError::new(
                "E_PROJECT_AUTHORING_SERIALIZE",
                format!("authoring snapshot을 직렬화할 수 없습니다: {err}"),
            )
        })
}

pub fn build_project_authoring_definition_json(
    request_json: &str,
) -> Result<String, LocalProjectError> {
    build_project_authoring_navigation_json(request_json, "definition")
}

pub fn build_project_authoring_references_json(
    request_json: &str,
) -> Result<String, LocalProjectError> {
    build_project_authoring_navigation_json(request_json, "references")
}

pub fn build_project_authoring_hover_json(request_json: &str) -> Result<String, LocalProjectError> {
    let request = parse_cursor_request(request_json)?;
    serialize_authoring_result(&build_project_authoring_hover(request)?, "hover")
}

pub fn build_project_authoring_completion_json(
    request_json: &str,
) -> Result<String, LocalProjectError> {
    let request = parse_cursor_request(request_json)?;
    serialize_authoring_result(&build_project_authoring_completion(request)?, "completion")
}

pub fn build_project_authoring_rename_preview_json(
    request_json: &str,
) -> Result<String, LocalProjectError> {
    let request: ProjectAuthoringRenameRequest =
        serde_json::from_str(request_json).map_err(|err| {
            LocalProjectError::new(
                "E_PROJECT_AUTHORING_RENAME_JSON",
                format!("authoring rename 요청 JSON을 읽을 수 없습니다: {err}"),
            )
        })?;
    serialize_authoring_result(
        &build_project_authoring_rename_preview(request)?,
        "rename plan",
    )
}

fn parse_cursor_request(
    request_json: &str,
) -> Result<ProjectAuthoringCursorRequest, LocalProjectError> {
    serde_json::from_str(request_json).map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_AUTHORING_CURSOR_JSON",
            format!("authoring cursor 요청 JSON을 읽을 수 없습니다: {err}"),
        )
    })
}

fn serialize_authoring_result<T: Serialize>(
    value: &T,
    label: &str,
) -> Result<String, LocalProjectError> {
    serde_json::to_string(value)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|err| {
            LocalProjectError::new(
                "E_PROJECT_AUTHORING_SERIALIZE",
                format!("authoring {label}을 직렬화할 수 없습니다: {err}"),
            )
        })
}

fn build_project_authoring_navigation_json(
    request_json: &str,
    operation: &'static str,
) -> Result<String, LocalProjectError> {
    let request = parse_cursor_request(request_json)?;
    let result = build_project_authoring_navigation(request, operation)?;
    serde_json::to_string(&result)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|err| {
            LocalProjectError::new(
                "E_PROJECT_AUTHORING_SERIALIZE",
                format!("authoring navigation을 직렬화할 수 없습니다: {err}"),
            )
        })
}

fn build_project_authoring_navigation(
    request: ProjectAuthoringCursorRequest,
    operation: &'static str,
) -> Result<ProjectAuthoringNavigationResult, LocalProjectError> {
    if request.schema != PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA {
        return Err(LocalProjectError::new(
            "E_PROJECT_AUTHORING_CURSOR_SCHEMA",
            format!("schema는 {PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA} 이어야 합니다"),
        ));
    }
    let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
        schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
        source: request.source,
        source_version: request.source_version,
    })?;
    if snapshot.project_source_sha256 != request.expected_project_source_sha256
        || snapshot.snapshot_sha256 != request.expected_snapshot_sha256
    {
        return Err(LocalProjectError::new(
            "E_PROJECT_AUTHORING_STALE",
            "cursor 요청의 project source/snapshot identity가 현재 Rust snapshot과 다릅니다",
        ));
    }
    let base = |status, message: String| ProjectAuthoringNavigationResult {
        schema: PROJECT_AUTHORING_NAVIGATION_SCHEMA,
        operation,
        status,
        source_version: snapshot.source_version.clone(),
        project_source_sha256: snapshot.project_source_sha256.clone(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        symbol_id: None,
        symbol_name: None,
        declaration: None,
        references: Vec::new(),
        message,
    };
    let definition_at_cursor = snapshot.definitions.iter().find(|definition| {
        definition.path == request.path
            && range_contains_offset(&definition.range, request.byte_offset)
    });
    let import_at_cursor = snapshot.imports.iter().find(|import| {
        import.importer_path == request.path
            && import
                .alias_range
                .as_ref()
                .is_some_and(|range| range_contains_offset(range, request.byte_offset))
    });
    let reference_at_cursor = snapshot
        .references
        .iter()
        .filter(|reference| {
            reference.path == request.path
                && range_contains_offset(&reference.range, request.byte_offset)
        })
        .min_by_key(|reference| reference.range.end - reference.range.start);

    if let Some(import) = import_at_cursor {
        let Some(range) = import.alias_range.clone() else {
            return Ok(base(
                "not_found",
                "쓰임 별칭의 정확한 선언 위치가 없습니다".to_string(),
            ));
        };
        let declaration = AuthoringNavigationLocation {
            path: import.importer_path.clone(),
            range,
            name: import.alias.clone(),
            kind: "import_alias".to_string(),
            declaration: true,
            write: false,
        };
        let references = if operation == "references" {
            snapshot
                .references
                .iter()
                .filter(|reference| reference.import_id.as_deref() == Some(import.id.as_str()))
                .map(reference_location)
                .collect()
        } else {
            Vec::new()
        };
        return Ok(ProjectAuthoringNavigationResult {
            symbol_id: Some(import.id.clone()),
            symbol_name: Some(import.alias.clone()),
            declaration: Some(declaration),
            references,
            status: "found",
            message: "Rust linker의 쓰임 별칭 선언을 찾았습니다".to_string(),
            ..base("found", String::new())
        });
    }

    let definition = if let Some(definition) = definition_at_cursor {
        Some(definition)
    } else if let Some(reference) = reference_at_cursor {
        if reference.resolution == "ambiguous" {
            return Ok(base(
                "ambiguous",
                "여러 lexical/export 후보가 있어 첫 정의를 임의 선택하지 않았습니다".to_string(),
            ));
        }
        reference.definition_id.as_ref().and_then(|id| {
            snapshot
                .definitions
                .iter()
                .find(|definition| &definition.id == id)
        })
    } else {
        None
    };
    let Some(definition) = definition else {
        return Ok(base(
            "not_found",
            "cursor에 AST로 결속된 DDN 정의가 없습니다".to_string(),
        ));
    };
    let declaration = definition_location(definition);
    let references = if operation == "references" {
        snapshot
            .references
            .iter()
            .filter(|reference| reference.definition_id.as_deref() == Some(definition.id.as_str()))
            .map(reference_location)
            .collect()
    } else {
        Vec::new()
    };
    Ok(ProjectAuthoringNavigationResult {
        symbol_id: Some(definition.id.clone()),
        symbol_name: Some(definition.name.clone()),
        declaration: Some(declaration),
        references,
        status: "found",
        message: "Rust AST lexical binding으로 정의를 찾았습니다".to_string(),
        ..base("found", String::new())
    })
}

fn range_contains_offset(range: &AuthoringSourceRange, offset: usize) -> bool {
    range.start <= offset
        && (offset < range.end || (range.start == range.end && offset == range.end))
}

fn definition_location(definition: &AuthoringDefinition) -> AuthoringNavigationLocation {
    AuthoringNavigationLocation {
        path: definition.path.clone(),
        range: definition.range.clone(),
        name: definition.name.clone(),
        kind: definition.kind.to_string(),
        declaration: true,
        write: false,
    }
}

fn reference_location(reference: &AuthoringReference) -> AuthoringNavigationLocation {
    AuthoringNavigationLocation {
        path: reference.path.clone(),
        range: reference.range.clone(),
        name: reference.name.clone(),
        kind: reference.resolution.to_string(),
        declaration: false,
        write: reference.write,
    }
}

fn checked_cursor_snapshot(
    request: &ProjectAuthoringCursorRequest,
) -> Result<ProjectAuthoringSnapshot, LocalProjectError> {
    if request.schema != PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA {
        return Err(LocalProjectError::new(
            "E_PROJECT_AUTHORING_CURSOR_SCHEMA",
            format!("schema는 {PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA} 이어야 합니다"),
        ));
    }
    let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
        schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
        source: request.source.clone(),
        source_version: request.source_version.clone(),
    })?;
    if snapshot.project_source_sha256 != request.expected_project_source_sha256
        || snapshot.snapshot_sha256 != request.expected_snapshot_sha256
    {
        return Err(LocalProjectError::new(
            "E_PROJECT_AUTHORING_STALE",
            "cursor 요청의 project source/snapshot identity가 현재 Rust snapshot과 다릅니다",
        ));
    }
    Ok(snapshot)
}

fn definition_for_cursor<'a>(
    snapshot: &'a ProjectAuthoringSnapshot,
    path: &str,
    byte_offset: usize,
) -> Option<&'a AuthoringDefinition> {
    if let Some(definition) = snapshot.definitions.iter().find(|definition| {
        definition.path == path && range_contains_offset(&definition.range, byte_offset)
    }) {
        return Some(definition);
    }
    snapshot
        .references
        .iter()
        .filter(|reference| {
            reference.path == path && range_contains_offset(&reference.range, byte_offset)
        })
        .min_by_key(|reference| reference.range.end - reference.range.start)
        .and_then(|reference| reference.definition_id.as_deref())
        .and_then(|id| {
            snapshot
                .definitions
                .iter()
                .find(|definition| definition.id == id)
        })
}

fn import_for_cursor<'a>(
    snapshot: &'a ProjectAuthoringSnapshot,
    path: &str,
    byte_offset: usize,
) -> Option<&'a AuthoringImport> {
    snapshot.imports.iter().find(|import| {
        import.importer_path == path
            && import
                .alias_range
                .as_ref()
                .is_some_and(|range| range_contains_offset(range, byte_offset))
    })
}

fn precise_method_atom_at_cursor(source: &str, byte_offset: usize) -> Option<ModelSpan> {
    let tokens = Lexer::new(source).tokenize().ok()?;
    let (index, token) = tokens.iter().enumerate().find(|(_, token)| {
        matches!(&token.kind, TokenKind::Atom(name) if name == "정밀")
            && token.span.start <= byte_offset
            && byte_offset <= token.span.end
    })?;
    if index < 2
        || !matches!(tokens[index - 1].kind, TokenKind::Colon)
        || !matches!(&tokens[index - 2].kind, TokenKind::Ident(name) if name == "방식")
    {
        return None;
    }
    Some(ModelSpan {
        start: token.span.start,
        end: token.span.end,
    })
}

fn build_project_authoring_hover(
    request: ProjectAuthoringCursorRequest,
) -> Result<ProjectAuthoringHoverResult, LocalProjectError> {
    let snapshot = checked_cursor_snapshot(&request)?;
    let base = |status, message: String| ProjectAuthoringHoverResult {
        schema: PROJECT_AUTHORING_HOVER_SCHEMA,
        status,
        source_version: snapshot.source_version.clone(),
        project_source_sha256: snapshot.project_source_sha256.clone(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        symbol_id: None,
        name: None,
        kind: None,
        type_label: None,
        unit: None,
        value_label: None,
        declaration: None,
        import_source: None,
        evidence: Vec::new(),
        message,
    };
    let source = project_source_text(&request.source, &request.path).ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_AUTHORING_SOURCE_REF",
            &request.path,
            "cursor 파일을 project source에서 찾지 못했습니다",
        )
    })?;
    if let Some(span) = precise_method_atom_at_cursor(source, request.byte_offset) {
        return Ok(ProjectAuthoringHoverResult {
            status: "found",
            symbol_id: Some(format!("solve-method:{}:{}", request.path, span.start)),
            name: Some("#정밀".to_string()),
            kind: Some("solve_method".to_string()),
            value_label: Some("#정밀".to_string()),
            evidence: vec!["Rust typed 풀이방법".to_string()],
            message: "정확한 비순환 관계를 엄밀한 수로 계산합니다".to_string(),
            ..base("found", String::new())
        });
    }
    if let Some(import) = import_for_cursor(&snapshot, &request.path, request.byte_offset) {
        let Some(range) = import.alias_range.clone() else {
            return Ok(base(
                "unavailable",
                "쓰임 별칭 위치 근거가 없습니다".to_string(),
            ));
        };
        return Ok(ProjectAuthoringHoverResult {
            status: "found",
            symbol_id: Some(import.id.clone()),
            name: Some(import.alias.clone()),
            kind: Some("import_alias".to_string()),
            declaration: Some(AuthoringNavigationLocation {
                path: import.importer_path.clone(),
                range,
                name: import.alias.clone(),
                kind: "import_alias".to_string(),
                declaration: true,
                write: false,
            }),
            import_source: Some(import.source.clone()),
            evidence: vec!["Rust project graph import".to_string()],
            message: "프로젝트 쓰임 선언에 기록된 정보입니다".to_string(),
            ..base("found", String::new())
        });
    }
    let Some(definition) = definition_for_cursor(&snapshot, &request.path, request.byte_offset)
    else {
        return Ok(base(
            "unavailable",
            "cursor에 AST로 결속된 선언 근거가 없습니다".to_string(),
        ));
    };
    let reference_import = snapshot.references.iter().find(|reference| {
        reference.path == request.path
            && range_contains_offset(&reference.range, request.byte_offset)
            && reference.definition_id.as_deref() == Some(definition.id.as_str())
    });
    let import_source = reference_import
        .and_then(|reference| reference.import_id.as_deref())
        .and_then(|id| snapshot.imports.iter().find(|import| import.id == id))
        .map(|import| import.source.clone());
    let mut evidence = vec!["Rust AST declaration".to_string()];
    if definition.type_label.is_some() {
        evidence.push("explicit type annotation".to_string());
    }
    if definition.unit.is_some() {
        evidence.push("explicit literal unit".to_string());
    }
    if definition.value_label.is_some() {
        evidence.push("direct AST initializer".to_string());
    }
    Ok(ProjectAuthoringHoverResult {
        status: "found",
        symbol_id: Some(definition.id.clone()),
        name: Some(definition.name.clone()),
        kind: Some(definition.kind.to_string()),
        type_label: definition.type_label.clone(),
        unit: definition.unit.clone(),
        value_label: definition.value_label.clone(),
        declaration: Some(definition_location(definition)),
        import_source,
        evidence,
        message: "Rust AST에 직접 기록된 정보만 표시합니다".to_string(),
        ..base("found", String::new())
    })
}

fn project_source_text<'a>(source: &'a VirtualProjectSource, path: &str) -> Option<&'a str> {
    source
        .files
        .iter()
        .find(|file| file.path.nfc().collect::<String>().replace('\\', "/") == path)
        .map(|file| file.utf8_text.as_str())
}

fn cursor_is_non_code(source: &str, byte_offset: usize) -> bool {
    if byte_offset > source.len() || !source.is_char_boundary(byte_offset) {
        return true;
    }
    if let Ok(tokens) = Lexer::new(source).tokenize() {
        if tokens.iter().any(|token| {
            token.span.start <= byte_offset
                && byte_offset < token.span.end
                && matches!(
                    token.kind,
                    TokenKind::StringLit(_)
                        | TokenKind::TemplateBlock(_)
                        | TokenKind::FormulaBlock(_)
                        | TokenKind::AssertionBlock(_)
                        | TokenKind::StateMachineBlock(_)
                        | TokenKind::BogeaMadangBlock(_)
                        | TokenKind::BogeaJangmyeonBlock(_)
                        | TokenKind::JjaimBlock(_)
                )
        }) {
            return true;
        }
    }
    let line_start = source[..byte_offset]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    source[line_start..byte_offset].find("//").is_some()
}

fn completion_prefix(source: &str, byte_offset: usize) -> (usize, String) {
    let mut start = byte_offset;
    for (index, ch) in source[..byte_offset].char_indices().rev() {
        if ch.is_alphanumeric()
            || matches!(ch, '_' | '.' | '?' | '!' | '\'' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ')
        {
            start = index;
        } else {
            break;
        }
    }
    (start, source[start..byte_offset].to_string())
}

fn scope_chain_at_cursor(
    snapshot: &ProjectAuthoringSnapshot,
    path: &str,
    byte_offset: usize,
) -> Vec<String> {
    let Some(mut scope) = snapshot
        .scopes
        .iter()
        .filter(|scope| scope.path == path && range_contains_offset(&scope.range, byte_offset))
        .min_by_key(|scope| scope.range.end.saturating_sub(scope.range.start))
    else {
        return Vec::new();
    };
    let mut chain = vec![scope.id.clone()];
    while let Some(parent) = scope.parent_id.as_deref() {
        chain.push(parent.to_string());
        let Some(next) = snapshot
            .scopes
            .iter()
            .find(|candidate| candidate.id == parent)
        else {
            break;
        };
        scope = next;
    }
    chain
}

fn model_double_colon_completion(source: &str, byte_offset: usize) -> bool {
    if !has_model_surface(source) {
        return false;
    }
    let before = &source[..byte_offset];
    let line_start = before.rfind(['\n', '\r']).map_or(0, |index| index + 1);
    let line = &before[line_start..];
    let Some(colon) = line.rfind("::") else {
        return false;
    };
    !line[colon + 2..].contains(['(', ')', '=', '.', ','])
}

fn model_output_completion(source: &str, byte_offset: usize) -> bool {
    if !has_model_surface(source) {
        return false;
    }
    let before = &source[..byte_offset];
    let Some(marker) = before.rfind("내줌") else {
        return false;
    };
    let tail = &before[marker + "내줌".len()..];
    let Some(open) = tail.find('(') else {
        return false;
    };
    let after_open = &tail[open + 1..];
    after_open.chars().filter(|ch| *ch == '(').count()
        == after_open.chars().filter(|ch| *ch == ')').count()
}

fn model_method_completion(source: &str, byte_offset: usize) -> Option<(usize, String)> {
    if !has_model_surface(source) {
        return None;
    }
    let before = &source[..byte_offset];
    let line_start = before.rfind(['\n', '\r']).map_or(0, |index| index + 1);
    let line = &before[line_start..];
    let marker = line.rfind("방식")?;
    let after_name = &line[marker + "방식".len()..];
    let colon = after_name.find(':')?;
    if !after_name[..colon].trim().is_empty() {
        return None;
    }
    let after_colon = &after_name[colon + 1..];
    let leading_space = after_colon.len() - after_colon.trim_start().len();
    let value = &after_colon[leading_space..];
    if value.chars().any(char::is_whitespace) {
        return None;
    }
    let partial = value.strip_prefix('#').unwrap_or(value);
    if !"정밀".starts_with(partial) {
        return None;
    }
    let value_start = line_start + marker + "방식".len() + colon + 1 + leading_space;
    Some((value_start, value.to_string()))
}

fn bounded_model_completion(
    snapshot: &ProjectAuthoringSnapshot,
    path: &str,
    source: &str,
    byte_offset: usize,
    filter_prefix: &str,
) -> Option<Vec<AuthoringCompletionItem>> {
    if model_double_colon_completion(source, byte_offset) {
        const PROPERTIES: &[(&str, &str)] = &[
            ("구함", "관계 한쪽의 계산 대상"),
            ("들임", "실행 때 받는 정확한 입력"),
            ("그자리", "acyclic 관계로 계산하는 값"),
            ("처음::고정", "원문에 고정한 정확한 초기값"),
            ("처음받음", "실행 때 초기값 변경 허용"),
            ("내줌", "실행별 출력 선택 허용"),
        ];
        return Some(
            PROPERTIES
                .iter()
                .filter(|(label, _)| label.starts_with(filter_prefix))
                .map(|(label, detail)| AuthoringCompletionItem {
                    label: (*label).to_string(),
                    kind: "model_property".to_string(),
                    origin: "Rust typed Model IR 제한 backend".to_string(),
                    detail: (*detail).to_string(),
                    insert_text: (*label).to_string(),
                    definition_id: None,
                    conflict: false,
                })
                .collect(),
        );
    }
    if model_output_completion(source, byte_offset) {
        return Some(
            snapshot
                .definitions
                .iter()
                .filter(|definition| {
                    definition.path == path
                        && matches!(
                            definition.kind,
                            "model_input" | "model_unknown" | "model_value"
                        )
                        && definition
                            .value_label
                            .as_deref()
                            .is_some_and(|label| label.split(" · ").any(|part| part == "내줌"))
                        && definition.name.starts_with(filter_prefix)
                })
                .map(|definition| AuthoringCompletionItem {
                    label: definition.name.clone(),
                    kind: "model_output".to_string(),
                    origin: definition.path.clone(),
                    detail: "::내줌 권한이 선언된 결과 이름".to_string(),
                    insert_text: definition.name.clone(),
                    definition_id: Some(definition.id.clone()),
                    conflict: false,
                })
                .collect(),
        );
    }
    None
}

fn build_project_authoring_completion(
    request: ProjectAuthoringCursorRequest,
) -> Result<ProjectAuthoringCompletionResult, LocalProjectError> {
    let snapshot = checked_cursor_snapshot(&request)?;
    let source = project_source_text(&request.source, &request.path).ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_AUTHORING_SOURCE_REF",
            &request.path,
            "cursor 파일을 project source에서 찾지 못했습니다",
        )
    })?;
    if request.byte_offset > source.len() || !source.is_char_boundary(request.byte_offset) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_AUTHORING_CURSOR_RANGE",
            &request.path,
            "cursor byte offset이 UTF-8 source 경계가 아닙니다",
        ));
    }
    let (prefix_start, prefix) = completion_prefix(source, request.byte_offset);
    let mut replace_start = prefix_start;
    let mut filter_prefix = prefix.as_str();
    let non_code = cursor_is_non_code(source, request.byte_offset);
    let replace_range = source_range(source, replace_start, request.byte_offset).unwrap();
    if non_code {
        return Ok(ProjectAuthoringCompletionResult {
            schema: PROJECT_AUTHORING_COMPLETION_SCHEMA,
            status: "unavailable",
            source_version: snapshot.source_version,
            project_source_sha256: snapshot.project_source_sha256,
            snapshot_sha256: snapshot.snapshot_sha256,
            path: request.path,
            replace_range,
            prefix,
            items: Vec::new(),
            message: "문자열·주석·opaque block에서는 코드 완성을 제안하지 않습니다".to_string(),
        });
    }

    if let Some((method_start, method_prefix)) =
        model_method_completion(source, request.byte_offset)
    {
        return Ok(ProjectAuthoringCompletionResult {
            schema: PROJECT_AUTHORING_COMPLETION_SCHEMA,
            status: "found",
            source_version: snapshot.source_version,
            project_source_sha256: snapshot.project_source_sha256,
            snapshot_sha256: snapshot.snapshot_sha256,
            path: request.path,
            replace_range: source_range(source, method_start, request.byte_offset).unwrap(),
            prefix: method_prefix,
            items: vec![AuthoringCompletionItem {
                label: "#정밀".to_string(),
                kind: "solve_method".to_string(),
                origin: "Rust typed 풀이방법".to_string(),
                detail: "정확한 비순환 관계를 엄밀한 수로 계산".to_string(),
                insert_text: "#정밀".to_string(),
                definition_id: None,
                conflict: false,
            }],
            message: "Rust typed 풀이방법 후보 1개를 찾았습니다".to_string(),
        });
    }

    if let Some(mut model_items) = bounded_model_completion(
        &snapshot,
        &request.path,
        source,
        request.byte_offset,
        filter_prefix,
    ) {
        model_items.sort_by(|left, right| left.label.cmp(&right.label));
        return Ok(ProjectAuthoringCompletionResult {
            schema: PROJECT_AUTHORING_COMPLETION_SCHEMA,
            status: "found",
            source_version: snapshot.source_version,
            project_source_sha256: snapshot.project_source_sha256,
            snapshot_sha256: snapshot.snapshot_sha256,
            path: request.path,
            replace_range,
            prefix,
            message: format!(
                "Rust typed Model IR 제한 범위에서 {}개 후보를 찾았습니다",
                model_items.len()
            ),
            items: model_items,
        });
    }

    let mut items = Vec::new();
    if let Some((field_parent, field_suffix)) = prefix.rsplit_once('.') {
        replace_start += field_parent.len() + 1;
        filter_prefix = field_suffix;
        let local_prefix = format!("{field_parent}.");
        let chain = scope_chain_at_cursor(&snapshot, &request.path, request.byte_offset);
        for definition in snapshot.definitions.iter().filter(|definition| {
            definition.path == request.path
                && definition.kind == "record_field"
                && chain.iter().any(|scope| scope == &definition.scope_id)
                && definition.name.starts_with(&local_prefix)
                && !definition.name[local_prefix.len()..].contains('.')
                && definition.name[local_prefix.len()..].starts_with(filter_prefix)
        }) {
            let field = &definition.name[local_prefix.len()..];
            items.push(AuthoringCompletionItem {
                label: field.to_string(),
                kind: "record_field".to_string(),
                origin: definition.path.clone(),
                detail: definition
                    .type_label
                    .clone()
                    .unwrap_or_else(|| "값꾸러미 필드".to_string()),
                insert_text: field.to_string(),
                definition_id: Some(definition.id.clone()),
                conflict: false,
            });
        }
        if items.is_empty() {
            if let Some(import) = (!field_parent.contains('.'))
                .then(|| {
                    snapshot.imports.iter().find(|import| {
                        import.importer_path == request.path && import.alias == field_parent
                    })
                })
                .flatten()
            {
                for export in snapshot.exports.iter().filter(|export| {
                    export.module_path == import.resolved_path
                        && export.external_name.starts_with(filter_prefix)
                }) {
                    items.push(AuthoringCompletionItem {
                        label: export.external_name.clone(),
                        kind: "imported_public_symbol".to_string(),
                        origin: import.source.clone(),
                        detail: format!("{}의 공개 심볼", import.alias),
                        insert_text: export.external_name.clone(),
                        definition_id: snapshot
                            .definitions
                            .iter()
                            .find(|definition| {
                                definition.path == export.module_path
                                    && definition.name == export.internal_name
                            })
                            .map(|definition| definition.id.clone()),
                        conflict: false,
                    });
                }
            }
        }
    } else {
        let chain = scope_chain_at_cursor(&snapshot, &request.path, request.byte_offset);
        let mut seen = BTreeSet::new();
        for scope_id in &chain {
            for definition in snapshot.definitions.iter().filter(|definition| {
                definition.path == request.path
                    && definition.scope_id == *scope_id
                    && (definition.range.start <= request.byte_offset
                        || matches!(
                            definition.kind,
                            "seed" | "function" | "value_function" | "relation"
                        ))
            }) {
                if definition.name.starts_with(filter_prefix)
                    && seen.insert(definition.normalized_name.clone())
                {
                    items.push(AuthoringCompletionItem {
                        label: definition.name.clone(),
                        kind: definition.kind.to_string(),
                        origin: definition.path.clone(),
                        detail: definition
                            .type_label
                            .clone()
                            .unwrap_or_else(|| "현재 lexical scope".to_string()),
                        insert_text: definition.name.clone(),
                        definition_id: Some(definition.id.clone()),
                        conflict: false,
                    });
                }
            }
        }
        for import in snapshot.imports.iter().filter(|import| {
            import.importer_path == request.path && import.alias.starts_with(filter_prefix)
        }) {
            items.push(AuthoringCompletionItem {
                label: import.alias.clone(),
                kind: "import_alias".to_string(),
                origin: import.source.clone(),
                detail: "프로젝트 쓰임 별칭".to_string(),
                insert_text: import.alias.clone(),
                definition_id: Some(import.id.clone()),
                conflict: seen.contains(&import.alias.nfc().collect::<String>()),
            });
        }
        const KEYWORDS: &[&str] = &[
            "만약",
            "이라면",
            "아니면",
            "되풀이",
            "동안",
            "멈추기",
            "건너뛰기",
            "돌려줘",
            "톺아보기",
            "해보고",
            "고르기",
            "보여주기",
            "덩이",
            "미루기",
        ];
        for keyword in KEYWORDS
            .iter()
            .filter(|keyword| keyword.starts_with(filter_prefix))
        {
            items.push(AuthoringCompletionItem {
                label: (*keyword).to_string(),
                kind: "keyword".to_string(),
                origin: "현행 Rust lexer".to_string(),
                detail: "현행 parser canonical keyword".to_string(),
                insert_text: (*keyword).to_string(),
                definition_id: None,
                conflict: false,
            });
        }
    }
    items.sort_by(|left, right| {
        left.label
            .cmp(&right.label)
            .then(left.kind.cmp(&right.kind))
            .then(left.origin.cmp(&right.origin))
    });
    let replace_range = source_range(source, replace_start, request.byte_offset).unwrap();
    Ok(ProjectAuthoringCompletionResult {
        schema: PROJECT_AUTHORING_COMPLETION_SCHEMA,
        status: "found",
        source_version: snapshot.source_version,
        project_source_sha256: snapshot.project_source_sha256,
        snapshot_sha256: snapshot.snapshot_sha256,
        path: request.path,
        replace_range,
        prefix,
        message: format!(
            "Rust scope/project graph에서 {}개 후보를 찾았습니다",
            items.len()
        ),
        items,
    })
}

fn valid_rename_name(name: &str) -> bool {
    if name.trim() != name || name.is_empty() {
        return false;
    }
    let Ok(tokens) = Lexer::new(name).tokenize() else {
        return false;
    };
    tokens.len() == 2
        && tokens[0].span.start == 0
        && tokens[0].span.end == name.len()
        && matches!(tokens[0].kind, TokenKind::Ident(_))
}

fn rename_plan_base(
    snapshot: &ProjectAuthoringSnapshot,
    status: &'static str,
    new_name: String,
    message: String,
) -> ProjectAuthoringRenamePlan {
    ProjectAuthoringRenamePlan {
        schema: PROJECT_AUTHORING_RENAME_PLAN_SCHEMA,
        status,
        source_version: snapshot.source_version.clone(),
        project_source_sha256: snapshot.project_source_sha256.clone(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        symbol_id: None,
        old_name: None,
        new_name,
        files: Vec::new(),
        affected_reference_count: 0,
        plan_sha256: String::new(),
        message,
    }
}

fn rename_edit(
    source: &VirtualProjectSource,
    path: &str,
    range: AuthoringSourceRange,
    new_text: &str,
    kind: &str,
) -> Option<AuthoringRenameEdit> {
    let text = project_source_text(source, path)?;
    let old_text = text.get(range.start..range.end)?.to_string();
    Some(AuthoringRenameEdit {
        path: path.to_string(),
        range,
        old_text,
        new_text: new_text.to_string(),
        kind: kind.to_string(),
    })
}

fn build_project_authoring_rename_preview(
    request: ProjectAuthoringRenameRequest,
) -> Result<ProjectAuthoringRenamePlan, LocalProjectError> {
    if request.schema != PROJECT_AUTHORING_RENAME_REQUEST_SCHEMA {
        return Err(LocalProjectError::new(
            "E_PROJECT_AUTHORING_RENAME_SCHEMA",
            format!("schema는 {PROJECT_AUTHORING_RENAME_REQUEST_SCHEMA} 이어야 합니다"),
        ));
    }
    let cursor_request = ProjectAuthoringCursorRequest {
        schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
        source: request.source.clone(),
        source_version: request.source_version.clone(),
        expected_project_source_sha256: request.expected_project_source_sha256.clone(),
        expected_snapshot_sha256: request.expected_snapshot_sha256.clone(),
        path: request.path.clone(),
        byte_offset: request.byte_offset,
    };
    let snapshot = checked_cursor_snapshot(&cursor_request)?;
    if !valid_rename_name(&request.new_name) {
        return Ok(rename_plan_base(
            &snapshot,
            "unavailable",
            request.new_name,
            "새 이름은 현행 Rust lexer가 하나의 식별자로 읽어야 합니다".to_string(),
        ));
    }

    let mut edits = Vec::new();
    let symbol_id;
    let old_name;
    let mut affected_reference_count = 0;
    if let Some(import) = import_for_cursor(&snapshot, &request.path, request.byte_offset) {
        if import.alias == request.new_name {
            return Ok(rename_plan_base(
                &snapshot,
                "unavailable",
                request.new_name,
                "새 이름이 현재 쓰임 별칭과 같습니다".to_string(),
            ));
        }
        let normalized = request.new_name.nfc().collect::<String>();
        if snapshot.imports.iter().any(|candidate| {
            candidate.id != import.id
                && candidate.importer_path == import.importer_path
                && candidate.alias.nfc().collect::<String>() == normalized
        }) {
            return Ok(rename_plan_base(
                &snapshot,
                "conflict",
                request.new_name,
                "같은 파일의 쓰임 별칭과 충돌하여 어떤 파일도 바꾸지 않았습니다".to_string(),
            ));
        }
        let Some(range) = import.alias_range.clone() else {
            return Ok(rename_plan_base(
                &snapshot,
                "unavailable",
                request.new_name,
                "쓰임 별칭의 정확한 source range가 없습니다".to_string(),
            ));
        };
        if let Some(edit) = rename_edit(
            &request.source,
            &import.importer_path,
            range,
            &request.new_name,
            "declaration",
        ) {
            edits.push(edit);
        }
        for reference in snapshot
            .references
            .iter()
            .filter(|reference| reference.import_id.as_deref() == Some(import.id.as_str()))
        {
            let text = project_source_text(&request.source, &reference.path).unwrap_or("");
            let end = reference.range.start.saturating_add(import.alias.len());
            let Some(range) = source_range(text, reference.range.start, end) else {
                continue;
            };
            if let Some(edit) = rename_edit(
                &request.source,
                &reference.path,
                range,
                &request.new_name,
                "reference",
            ) {
                edits.push(edit);
                affected_reference_count += 1;
            }
        }
        symbol_id = Some(import.id.clone());
        old_name = Some(import.alias.clone());
    } else {
        let Some(definition) = definition_for_cursor(&snapshot, &request.path, request.byte_offset)
        else {
            return Ok(rename_plan_base(
                &snapshot,
                "unavailable",
                request.new_name,
                "cursor에 AST로 결속된 이름 바꾸기 대상이 없습니다".to_string(),
            ));
        };
        if definition.name == request.new_name {
            return Ok(rename_plan_base(
                &snapshot,
                "unavailable",
                request.new_name,
                "새 이름이 현재 이름과 같습니다".to_string(),
            ));
        }
        let normalized = request.new_name.nfc().collect::<String>();
        if snapshot.definitions.iter().any(|candidate| {
            candidate.id != definition.id
                && candidate.path == definition.path
                && candidate.scope_id == definition.scope_id
                && candidate.normalized_name == normalized
        }) {
            return Ok(rename_plan_base(
                &snapshot,
                "conflict",
                request.new_name,
                "같은 lexical scope의 정의와 충돌하여 어떤 파일도 바꾸지 않았습니다".to_string(),
            ));
        }
        let export_rows = snapshot
            .exports
            .iter()
            .filter(|export| {
                export.module_path == definition.path && export.internal_name == definition.name
            })
            .collect::<Vec<_>>();
        if export_rows
            .iter()
            .any(|export| export.external_name != definition.name || export.name_range.is_none())
        {
            return Ok(rename_plan_base(
                &snapshot,
                "unavailable",
                request.new_name,
                "외부명과 내부명이 다른 공개 mapping은 이번 bounded rename에서 바꾸지 않습니다"
                    .to_string(),
            ));
        }
        if !export_rows.is_empty()
            && snapshot.exports.iter().any(|export| {
                export.module_path == definition.path
                    && export.external_name.nfc().collect::<String>() == normalized
                    && export.internal_name != definition.name
            })
        {
            return Ok(rename_plan_base(
                &snapshot,
                "conflict",
                request.new_name,
                "같은 module의 공개 이름과 충돌하여 어떤 파일도 바꾸지 않았습니다".to_string(),
            ));
        }
        if let Some(edit) = rename_edit(
            &request.source,
            &definition.path,
            definition.range.clone(),
            &request.new_name,
            "declaration",
        ) {
            edits.push(edit);
        }
        for export in export_rows {
            if let Some(edit) = export.name_range.clone().and_then(|range| {
                rename_edit(
                    &request.source,
                    &export.module_path,
                    range,
                    &request.new_name,
                    "export",
                )
            }) {
                edits.push(edit);
            }
        }
        for reference in snapshot
            .references
            .iter()
            .filter(|reference| reference.definition_id.as_deref() == Some(definition.id.as_str()))
        {
            let text = project_source_text(&request.source, &reference.path).unwrap_or("");
            let range = if reference.name.contains('.') {
                let start = reference.range.end.saturating_sub(definition.name.len());
                source_range(text, start, reference.range.end)
            } else {
                Some(reference.range.clone())
            };
            if let Some(edit) = range.and_then(|range| {
                rename_edit(
                    &request.source,
                    &reference.path,
                    range,
                    &request.new_name,
                    "reference",
                )
            }) {
                edits.push(edit);
                affected_reference_count += 1;
            }
        }
        symbol_id = Some(definition.id.clone());
        old_name = Some(definition.name.clone());
    }

    edits.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.range.start.cmp(&right.range.start))
            .then(left.range.end.cmp(&right.range.end))
    });
    edits.dedup_by(|left, right| {
        left.path == right.path
            && left.range.start == right.range.start
            && left.range.end == right.range.end
    });
    let mut grouped: BTreeMap<String, Vec<AuthoringRenameEdit>> = BTreeMap::new();
    for edit in edits {
        grouped.entry(edit.path.clone()).or_default().push(edit);
    }
    let mut files = Vec::new();
    for (path, edits) in grouped {
        let source_text = project_source_text(&request.source, &path).unwrap_or("");
        files.push(AuthoringRenameFilePatch {
            path,
            expected_source_sha256: sha256_hex(source_text.as_bytes()),
            edits,
        });
    }
    let plan_material = serde_json::to_vec(&(
        &snapshot.project_source_sha256,
        &snapshot.snapshot_sha256,
        &symbol_id,
        &old_name,
        &request.new_name,
        &files,
    ))
    .map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_AUTHORING_SERIALIZE",
            format!("rename plan hash 입력을 직렬화할 수 없습니다: {err}"),
        )
    })?;
    Ok(ProjectAuthoringRenamePlan {
        schema: PROJECT_AUTHORING_RENAME_PLAN_SCHEMA,
        status: "ready",
        source_version: snapshot.source_version,
        project_source_sha256: snapshot.project_source_sha256,
        snapshot_sha256: snapshot.snapshot_sha256,
        symbol_id,
        old_name,
        new_name: request.new_name,
        files,
        affected_reference_count,
        plan_sha256: sha256_hex(&plan_material),
        message: "Rust AST binding으로 만든 원자 적용용 이름 바꾸기 계획입니다".to_string(),
    })
}

pub fn build_project_authoring_snapshot(
    request: ProjectAuthoringSnapshotRequest,
) -> Result<ProjectAuthoringSnapshot, LocalProjectError> {
    if request.schema != PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA {
        return Err(LocalProjectError::new(
            "E_PROJECT_AUTHORING_SCHEMA",
            format!("schema는 {PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA} 이어야 합니다"),
        ));
    }
    if request.source_version.trim().is_empty() {
        return Err(LocalProjectError::new(
            "E_PROJECT_AUTHORING_SOURCE_VERSION",
            "source_version은 비어 있을 수 없습니다",
        ));
    }

    let normalized = local_project::normalize_virtual_project_source(request.source.clone())?;
    let project_source_sha256 = local_project::virtual_project_source_sha256(&request.source)?;
    let mut diagnostics = Vec::new();
    let graph = match local_project::build_project_graph(request.source.clone()) {
        Ok(graph) => Some(graph),
        Err(err) => {
            diagnostics.push(diagnostic_from_local_error(&err, &request.source));
            None
        }
    };
    if graph.is_some() {
        if let Err(err) =
            local_project::validate_project_module_boundary_for_authoring(request.source.clone())
        {
            diagnostics.push(diagnostic_from_local_error(&err, &request.source));
        }
    }

    let mut source_by_path = BTreeMap::new();
    let mut role_by_path = BTreeMap::new();
    for file in &request.source.files {
        let path = file.path.nfc().collect::<String>().replace('\\', "/");
        source_by_path.insert(path.clone(), file.utf8_text.clone());
        role_by_path.insert(path, file.role.clone());
    }

    let mut imports = Vec::new();
    let mut exports = Vec::new();
    for (path, source) in &source_by_path {
        if !is_ddn_source(
            path,
            role_by_path.get(path).map(String::as_str).unwrap_or(""),
        ) {
            continue;
        }
        match local_project::parse_imports(source, path) {
            Ok(rows) => {
                for row in rows {
                    let range =
                        local_project::find_named_block_key_spans(source, "쓰임", &row.alias)
                            .into_iter()
                            .next()
                            .and_then(|span| source_range(source, span.0, span.1));
                    match local_project::resolve_import_source_path(path, &row.source) {
                        Ok(resolved_path) => {
                            let start = range.as_ref().map(|range| range.start).unwrap_or(0);
                            imports.push(AuthoringImport {
                                id: format!(
                                    "import:{}:{}:{}",
                                    path,
                                    start,
                                    sha256_hex(row.alias.as_bytes())
                                ),
                                importer_path: path.clone(),
                                alias: row.alias,
                                source: row.source,
                                resolved_path,
                                alias_range: range,
                            })
                        }
                        Err(err) => {
                            diagnostics.push(diagnostic_from_local_error(&err, &request.source))
                        }
                    }
                }
            }
            Err(err) => diagnostics.push(diagnostic_from_local_error(&err, &request.source)),
        }
        match local_project::parse_exports(source, path) {
            Ok(rows) => {
                for row in rows {
                    let span = local_project::find_named_block_key_spans(
                        source,
                        "드러냄",
                        &row.external_name,
                    )
                    .into_iter()
                    .next()
                    .or_else(|| {
                        local_project::find_named_block_key_spans(
                            source,
                            "공개",
                            &row.external_name,
                        )
                        .into_iter()
                        .next()
                    });
                    exports.push(AuthoringExport {
                        module_path: path.clone(),
                        external_name: row.external_name,
                        internal_name: row.internal_name,
                        name_range: span.and_then(|span| source_range(source, span.0, span.1)),
                    });
                }
            }
            Err(err) => diagnostics.push(diagnostic_from_local_error(&err, &request.source)),
        }
    }
    imports.sort_by(|a, b| {
        a.importer_path
            .cmp(&b.importer_path)
            .then(a.alias.cmp(&b.alias))
            .then(a.resolved_path.cmp(&b.resolved_path))
    });
    exports.sort_by(|a, b| {
        a.module_path
            .cmp(&b.module_path)
            .then(a.external_name.cmp(&b.external_name))
            .then(a.internal_name.cmp(&b.internal_name))
    });

    let mut files = Vec::new();
    let mut scopes = Vec::new();
    let mut definitions = Vec::new();
    let mut raw_occurrences = Vec::new();
    for normalized_file in &normalized.files {
        let path = &normalized_file.path;
        let source = source_by_path.get(path).map(String::as_str).unwrap_or("");
        let role = role_by_path
            .get(path)
            .map(String::as_str)
            .unwrap_or("metadata");
        let before_scope_count = scopes.len();
        let before_definition_count = definitions.len();
        if !is_ddn_source(path, role) || path == &normalized.root_path {
            files.push(AuthoringFileSnapshot {
                path: path.clone(),
                role: role.to_string(),
                source_sha256: normalized_file.sha256.clone(),
                utf8_bytes: normalized_file.byte_length,
                parse_status: "metadata_only",
                parse_span_mode: "original_utf8_bytes",
                wrapper_prefix_bytes: 0,
                definition_count: 0,
                scope_count: 0,
            });
            continue;
        }

        if has_model_surface(source) {
            // Module metadata belongs to the project graph/linker boundary, not
            // the linked model frontdoor.  The shared mask preserves every
            // original byte offset, so typed-model diagnostics and authoring
            // navigation remain file/source exact after project validation.
            let model_source = local_project::mask_module_metadata_blocks(source)?;
            match ddonirang_lang::parse_model_program_with_source_file(&model_source, path) {
                Ok(program) => collect_model_authoring_file(
                    path,
                    source,
                    &program,
                    &mut scopes,
                    &mut definitions,
                    &mut raw_occurrences,
                    &mut diagnostics,
                ),
                Err(err) => diagnostics.push(AuthoringDiagnostic {
                    code: err.candidate_code.to_string(),
                    message: err.message,
                    severity: "error",
                    source_ref: Some(path.clone()),
                    source_range: model_source_range(source, err.span),
                }),
            }
            files.push(AuthoringFileSnapshot {
                path: path.clone(),
                role: role.to_string(),
                source_sha256: normalized_file.sha256.clone(),
                utf8_bytes: normalized_file.byte_length,
                parse_status: "model_typed_ir",
                parse_span_mode: "original_utf8_bytes",
                wrapper_prefix_bytes: 0,
                definition_count: definitions.len() - before_definition_count,
                scope_count: scopes.len() - before_scope_count,
            });
            continue;
        }

        let parsed = parse_authoring_file(source, path)?;
        if let Some(diagnostic) = parsed.diagnostic.clone() {
            diagnostics.push(diagnostic);
        }
        if let Some(program) = &parsed.program {
            let mut collector = DefinitionCollector::new(
                path,
                source,
                &parsed.tokens,
                parsed.shift,
                &exports,
                &mut scopes,
                &mut definitions,
                &mut raw_occurrences,
            );
            collector.collect(program, parsed.parse_status == "wrapped");
        }
        files.push(AuthoringFileSnapshot {
            path: path.clone(),
            role: role.to_string(),
            source_sha256: normalized_file.sha256.clone(),
            utf8_bytes: normalized_file.byte_length,
            parse_status: parsed.parse_status,
            parse_span_mode: parsed.parse_span_mode,
            wrapper_prefix_bytes: parsed.shift,
            definition_count: definitions.len() - before_definition_count,
            scope_count: scopes.len() - before_scope_count,
        });
    }

    definitions.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.range.start.cmp(&b.range.start))
            .then(a.name.cmp(&b.name))
    });
    scopes.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.range.start.cmp(&b.range.start))
            .then(a.id.cmp(&b.id))
    });
    let mut references = resolve_references(
        &raw_occurrences,
        &source_by_path,
        &scopes,
        &definitions,
        &imports,
        &exports,
    );
    references.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.range.start.cmp(&b.range.start))
            .then(a.name.cmp(&b.name))
    });
    diagnostics.sort_by(|a, b| {
        a.source_ref
            .cmp(&b.source_ref)
            .then(
                a.source_range
                    .as_ref()
                    .map(|range| range.start)
                    .cmp(&b.source_range.as_ref().map(|range| range.start)),
            )
            .then(a.code.cmp(&b.code))
    });

    let mut snapshot = ProjectAuthoringSnapshot {
        schema: PROJECT_AUTHORING_SNAPSHOT_SCHEMA,
        source_version: request.source_version,
        project_source_sha256,
        project_graph_sha256: graph.map(|row| row.project_graph_sha256),
        root_path: normalized.root_path,
        entry_path: normalized.entry_path,
        files,
        imports,
        exports,
        scopes,
        definitions,
        references,
        diagnostics,
        snapshot_sha256: String::new(),
    };
    snapshot.snapshot_sha256 = stable_snapshot_sha256(&snapshot)?;
    Ok(snapshot)
}

fn model_source_range(source: &str, span: ModelSpan) -> Option<AuthoringSourceRange> {
    source_range(source, span.start, span.end.min(source.len()))
}

fn model_role_label(role: PrimaryRole) -> &'static str {
    match role {
        PrimaryRole::Axis => "축",
        PrimaryRole::Continuous => "이어감",
        PrimaryRole::Discrete => "띄엄",
        PrimaryRole::Algebraic => "그자리",
        PrimaryRole::Input => "들임",
    }
}

fn model_initialization_label(initialization: &Initialization) -> &'static str {
    match initialization {
        Initialization::None => "초기값 없음",
        Initialization::Fixed(_) => "처음 고정",
        Initialization::Guess(_) => "처음 추정",
        Initialization::Computed => "관계로 계산",
    }
}

fn is_model_name_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '.' | '?' | '!' | '\'')
}

fn model_name_spans(source: &str, within: ModelSpan, name: &str) -> Vec<(usize, usize)> {
    if name.is_empty() || within.start > within.end || within.end > source.len() {
        return Vec::new();
    }
    if name.contains('.') {
        let parts = name.split('.').collect::<Vec<_>>();
        let width = parts.len().saturating_mul(2).saturating_sub(1);
        let Ok(tokens) = Lexer::new(&source[within.start..within.end]).tokenize() else {
            return Vec::new();
        };
        if width == 0 || tokens.len() < width {
            return Vec::new();
        }
        let mut spans = Vec::new();
        for index in 0..=tokens.len() - width {
            let window = &tokens[index..index + width];
            let matches = window.iter().enumerate().all(|(offset, token)| {
                if offset % 2 == 0 {
                    token_name(token) == Some(parts[offset / 2])
                } else {
                    matches!(token.kind, TokenKind::Dot)
                }
            });
            if !matches
                || index
                    .checked_sub(1)
                    .is_some_and(|before| matches!(tokens[before].kind, TokenKind::Dot))
                || tokens
                    .get(index + width)
                    .is_some_and(|after| matches!(after.kind, TokenKind::Dot))
            {
                continue;
            }
            spans.push((
                within.start + window[0].span.start,
                within.start + window[width - 1].span.end,
            ));
        }
        return spans;
    }
    source[within.start..within.end]
        .match_indices(name)
        .filter_map(|(relative, _)| {
            let start = within.start + relative;
            let end = start + name.len();
            if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
                return None;
            }
            let before = source[..start].chars().next_back();
            let after = source[end..].chars().next();
            if before.is_some_and(is_model_name_char) || after.is_some_and(is_model_name_char) {
                return None;
            }
            Some((start, end))
        })
        .collect()
}

fn push_model_definition(
    definitions: &mut Vec<AuthoringDefinition>,
    path: &str,
    source: &str,
    scope_id: &str,
    name: &str,
    kind: &'static str,
    span: ModelSpan,
    type_label: Option<String>,
    unit: Option<String>,
    value_label: Option<String>,
) {
    let Some((start, end)) = model_name_spans(source, span, name).into_iter().next() else {
        return;
    };
    let Some(range) = source_range(source, start, end) else {
        return;
    };
    let normalized_name = name.nfc().collect::<String>();
    definitions.push(AuthoringDefinition {
        id: format!(
            "def:{path}:{start}:{kind}:{}",
            sha256_hex(normalized_name.as_bytes())
        ),
        name: name.to_string(),
        normalized_name,
        kind,
        path: path.to_string(),
        scope_id: scope_id.to_string(),
        range,
        type_label,
        unit,
        value_label,
        constant: false,
        explicit: true,
        export_names: Vec::new(),
    });
}

fn push_model_expr_references(
    occurrences: &mut Vec<RawOccurrence>,
    path: &str,
    source: &str,
    scope_id: &str,
    expr: &ModelExpr,
) {
    for name in &expr.references {
        for (start, end) in model_name_spans(source, expr.span, name) {
            occurrences.push(RawOccurrence {
                name: name.clone(),
                path: path.to_string(),
                scope_id: scope_id.to_string(),
                start,
                end,
                write: false,
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_model_authoring_file(
    path: &str,
    source: &str,
    program: &ModelProgram,
    scopes: &mut Vec<AuthoringScope>,
    definitions: &mut Vec<AuthoringDefinition>,
    occurrences: &mut Vec<RawOccurrence>,
    diagnostics: &mut Vec<AuthoringDiagnostic>,
) {
    let file_scope_id = format!("scope:{path}:model-file");
    if let Some(range) = source_range(source, 0, source.len()) {
        scopes.push(AuthoringScope {
            id: file_scope_id.clone(),
            path: path.to_string(),
            parent_id: None,
            kind: "model_file",
            range,
        });
    }

    for definition in &program.ir.definitions {
        let model_scope_id = format!("scope:{path}:model:{}", definition.span.start);
        if let Some(range) = model_source_range(source, definition.body_span) {
            scopes.push(AuthoringScope {
                id: model_scope_id.clone(),
                path: path.to_string(),
                parent_id: Some(file_scope_id.clone()),
                kind: "model_definition",
                range,
            });
        }
        push_model_definition(
            definitions,
            path,
            source,
            &file_scope_id,
            &definition.name,
            "model_definition",
            definition.header_span,
            Some("모형값".to_string()),
            None,
            None,
        );
        for declaration in &definition.declarations {
            let kind = if declaration.unknown {
                "model_unknown"
            } else if declaration.role == PrimaryRole::Input {
                "model_input"
            } else {
                "model_value"
            };
            let mut roles = vec![model_role_label(declaration.role).to_string()];
            if declaration.unknown {
                roles.push("구함".to_string());
            }
            if declaration.exported {
                roles.push("내줌".to_string());
            }
            if declaration.accepts_initial_override {
                roles.push("처음받음".to_string());
            }
            roles.push(model_initialization_label(&declaration.initialization).to_string());
            push_model_definition(
                definitions,
                path,
                source,
                &model_scope_id,
                &declaration.name,
                kind,
                declaration.identity.span,
                Some(declaration.model_type.name.clone()),
                declaration.model_type.unit.clone(),
                Some(roles.join(" · ")),
            );
            match &declaration.initialization {
                Initialization::Fixed(expr) | Initialization::Guess(expr) => {
                    push_model_expr_references(occurrences, path, source, &model_scope_id, expr);
                }
                Initialization::None | Initialization::Computed => {}
            }
        }
        for relation in &definition.relations {
            push_model_expr_references(occurrences, path, source, &model_scope_id, &relation.left);
            push_model_expr_references(occurrences, path, source, &model_scope_id, &relation.right);
        }
    }

    for request in &program.ir.requests {
        let model_scope_id = program
            .ir
            .definitions
            .iter()
            .find(|definition| definition.name == request.binding_name)
            .map(|definition| format!("scope:{path}:model:{}", definition.span.start))
            .or_else(|| {
                program
                    .ir
                    .definitions
                    .first()
                    .map(|definition| format!("scope:{path}:model:{}", definition.span.start))
            })
            .unwrap_or_else(|| file_scope_id.clone());
        for pin in &request.pins {
            push_model_expr_references(occurrences, path, source, &model_scope_id, &pin.value);
        }
        for field in request
            .input_bindings
            .iter()
            .chain(request.initial_overrides.iter())
        {
            push_model_expr_references(occurrences, path, source, &model_scope_id, &field.value);
        }
        if let Some(end) = &request.end {
            push_model_expr_references(occurrences, path, source, &model_scope_id, end);
        }
        for name in &request.output_selection {
            for (start, end) in model_name_spans(source, request.span, name) {
                occurrences.push(RawOccurrence {
                    name: name.clone(),
                    path: path.to_string(),
                    scope_id: model_scope_id.clone(),
                    start,
                    end,
                    write: false,
                });
            }
        }
    }

    for view in &program.ir.views {
        for field in &view.fields {
            let scope_id = program
                .ir
                .definitions
                .first()
                .map(|definition| format!("scope:{path}:model:{}", definition.span.start))
                .unwrap_or_else(|| file_scope_id.clone());
            push_model_expr_references(occurrences, path, source, &scope_id, &field.value);
        }
    }

    diagnostics.extend(
        program
            .ir
            .diagnostics
            .iter()
            .map(|diagnostic| AuthoringDiagnostic {
                code: diagnostic.candidate_code.to_string(),
                message: diagnostic.message.clone(),
                severity: diagnostic.severity,
                source_ref: Some(path.to_string()),
                source_range: model_source_range(source, diagnostic.provenance),
            }),
    );
}

fn parse_authoring_file(
    source: &str,
    path: &str,
) -> Result<ParsedAuthoringFile, LocalProjectError> {
    let masked = local_project::mask_module_metadata_blocks(source)?;
    let original_tokens = Lexer::new(&masked).tokenize();
    if masked.trim().is_empty() {
        return Ok(ParsedAuthoringFile {
            program: None,
            tokens: original_tokens.unwrap_or_default(),
            parse_status: "metadata_only",
            parse_span_mode: "original_utf8_bytes",
            shift: 0,
            diagnostic: None,
        });
    }

    match ddonirang_lang::parse_with_mode(&masked, path, ParseMode::Strict) {
        Ok(program) => Ok(ParsedAuthoringFile {
            program: Some(program),
            tokens: original_tokens.unwrap_or_default(),
            parse_status: "direct",
            parse_span_mode: "original_utf8_bytes",
            shift: 0,
            diagnostic: None,
        }),
        Err(direct_error) => {
            let mut wrapped = String::with_capacity(
                FLAT_WRAPPER_PREFIX.len() + masked.len() + FLAT_WRAPPER_SUFFIX.len() + 1,
            );
            wrapped.push_str(FLAT_WRAPPER_PREFIX);
            wrapped.push_str(&masked);
            if !masked.ends_with('\n') {
                wrapped.push('\n');
            }
            wrapped.push_str(FLAT_WRAPPER_SUFFIX);
            match ddonirang_lang::parse_with_mode(&wrapped, path, ParseMode::Strict) {
                Ok(program) => {
                    let tokens = Lexer::new(&wrapped)
                        .tokenize()
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|mut token| {
                            if token.span.start < FLAT_WRAPPER_PREFIX.len()
                                || token.span.end < FLAT_WRAPPER_PREFIX.len()
                            {
                                return None;
                            }
                            let start = token.span.start - FLAT_WRAPPER_PREFIX.len();
                            let end = token.span.end - FLAT_WRAPPER_PREFIX.len();
                            if start > end || end > source.len() {
                                return None;
                            }
                            token.span.start = start;
                            token.span.end = end;
                            Some(token)
                        })
                        .collect();
                    Ok(ParsedAuthoringFile {
                        program: Some(program),
                        tokens,
                        parse_status: "wrapped",
                        parse_span_mode: "wrapper_shift_reversed",
                        shift: FLAT_WRAPPER_PREFIX.len(),
                        diagnostic: None,
                    })
                }
                Err(wrapped_error) => {
                    let mapped =
                        map_span(wrapped_error.span, FLAT_WRAPPER_PREFIX.len(), source.len());
                    let chosen = mapped.unwrap_or(direct_error.span);
                    Ok(ParsedAuthoringFile {
                        program: None,
                        tokens: original_tokens.unwrap_or_default(),
                        parse_status: "unavailable",
                        parse_span_mode: "original_utf8_bytes",
                        shift: 0,
                        diagnostic: Some(AuthoringDiagnostic {
                            code: "E_PROJECT_AUTHORING_PARSE_UNAVAILABLE".to_string(),
                            message: format!(
                                "정확한 원문 span을 보존하는 strict/direct 또는 bounded wrapper parse가 실패했습니다: {}",
                                wrapped_error.message
                            ),
                            severity: "error",
                            source_ref: Some(path.to_string()),
                            source_range: source_range(source, chosen.start, chosen.end.min(source.len())),
                        }),
                    })
                }
            }
        }
    }
}

struct DefinitionCollector<'a> {
    path: &'a str,
    source: &'a str,
    tokens: &'a [Token],
    shift: usize,
    exports: &'a [AuthoringExport],
    scopes: &'a mut Vec<AuthoringScope>,
    definitions: &'a mut Vec<AuthoringDefinition>,
    occurrences: &'a mut Vec<RawOccurrence>,
    implicit_globals: BTreeSet<String>,
    scope_serial: usize,
}

impl<'a> DefinitionCollector<'a> {
    fn new(
        path: &'a str,
        source: &'a str,
        tokens: &'a [Token],
        shift: usize,
        exports: &'a [AuthoringExport],
        scopes: &'a mut Vec<AuthoringScope>,
        definitions: &'a mut Vec<AuthoringDefinition>,
        occurrences: &'a mut Vec<RawOccurrence>,
    ) -> Self {
        Self {
            path,
            source,
            tokens,
            shift,
            exports,
            scopes,
            definitions,
            occurrences,
            implicit_globals: BTreeSet::new(),
            scope_serial: 0,
        }
    }

    fn collect(&mut self, program: &CanonProgram, wrapped: bool) {
        let file_scope = self.add_scope(None, "file", 0, self.source.len());
        if wrapped {
            for item in &program.items {
                let TopLevelItem::SeedDef(seed) = item;
                if let Some(body) = &seed.body {
                    self.collect_body(body, &file_scope, false);
                }
            }
            return;
        }
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            let Some(seed_span) = self.map(seed.span) else {
                continue;
            };
            if let Some(name_span) = self.name_span(&seed.canonical_name, seed.span, false) {
                self.add_definition(
                    &seed.canonical_name,
                    seed_kind_label(&seed.seed_kind),
                    &file_scope,
                    name_span,
                    None,
                    None,
                    false,
                    true,
                );
            }
            let seed_scope =
                self.add_scope(Some(file_scope.clone()), "seed", seed_span.0, seed_span.1);
            for param in &seed.params {
                if let Some(default_value) = &param.default_value {
                    self.collect_expr(default_value, &seed_scope, false);
                }
                if let Some(span) = self.name_span(&param.pin_name, param.span, false) {
                    self.add_definition(
                        &param.pin_name,
                        "parameter",
                        &seed_scope,
                        span,
                        Some(type_ref_label(&param.type_ref)),
                        param.default_value.as_ref().and_then(explicit_unit),
                        false,
                        true,
                    );
                    if let Some(value) = &param.default_value {
                        self.set_last_value_label(value);
                    }
                }
            }
            if let Some(body) = &seed.body {
                self.collect_body(body, &seed_scope, false);
            }
        }
    }

    fn collect_body(&mut self, body: &Body, parent_scope: &str, nested: bool) {
        let scope_id = if nested {
            self.map(body.span)
                .map(|span| self.add_scope(Some(parent_scope.to_string()), "body", span.0, span.1))
                .unwrap_or_else(|| parent_scope.to_string())
        } else {
            parent_scope.to_string()
        };
        for stmt in &body.stmts {
            match stmt {
                Stmt::DeclBlock { items, .. } => {
                    for item in items {
                        if let Some(value) = &item.value {
                            self.collect_expr(value, &scope_id, false);
                        }
                        if let Some(span) = self.name_span(&item.name, item.span, false) {
                            self.add_definition(
                                &item.name,
                                match item.kind {
                                    DeclKind::Gureut => "variable",
                                    DeclKind::Butbak => "constant",
                                },
                                &scope_id,
                                span,
                                Some(type_ref_label(&item.type_ref)),
                                item.value.as_ref().and_then(explicit_unit),
                                matches!(item.kind, DeclKind::Butbak),
                                true,
                            );
                            if let Some(value) = &item.value {
                                self.set_last_value_label(value);
                            }
                        }
                    }
                }
                Stmt::Definition {
                    span,
                    name,
                    type_ref,
                    value,
                    ..
                } => {
                    self.collect_expr(value, &scope_id, false);
                    if let Some(name_span) = self.name_span(name, *span, false) {
                        self.add_definition(
                            name,
                            "definition",
                            &scope_id,
                            name_span,
                            Some(type_ref_label(type_ref)),
                            explicit_unit(value),
                            true,
                            true,
                        );
                        self.set_last_value_label(value);
                    }
                }
                Stmt::Mutate { target, value, .. } => {
                    let mut created = false;
                    if let ExprKind::Var(name) = &target.kind {
                        let normalized = name.nfc().collect::<String>();
                        if !self.definition_visible(&normalized, &scope_id)
                            && self.implicit_globals.insert(normalized)
                        {
                            created = true;
                            if let Some(span) = self.map(target.span) {
                                let file_scope_id = self.file_scope_id().to_string();
                                self.add_definition(
                                    name,
                                    "variable",
                                    &file_scope_id,
                                    span,
                                    None,
                                    explicit_unit(value),
                                    false,
                                    false,
                                );
                                self.set_last_value_label(value);
                            }
                        }
                    }
                    if !created {
                        self.collect_expr(target, &scope_id, true);
                    }
                    self.collect_expr(value, &scope_id, false);
                    if let ExprKind::Var(name) = &target.kind {
                        self.collect_record_field_definitions(name, value, &scope_id);
                    }
                }
                Stmt::Receive {
                    span,
                    binding,
                    condition,
                    body,
                    ..
                } => {
                    if let Some(condition) = condition {
                        self.collect_expr(condition, &scope_id, false);
                    }
                    let child = self.child_scope_for_body(&scope_id, body, "receive");
                    if let Some(name) = binding {
                        if let Some(span) = self.name_span(name, *span, false) {
                            self.add_definition(
                                name, "binding", &child, span, None, None, false, true,
                            );
                        }
                    }
                    self.collect_body(body, &child, false);
                }
                Stmt::ForEach {
                    span,
                    item,
                    item_type,
                    iterable,
                    body,
                    ..
                } => {
                    self.collect_expr(iterable, &scope_id, false);
                    let child = self.child_scope_for_body(&scope_id, body, "foreach");
                    if let Some(span) = self.name_span(item, *span, false) {
                        self.add_definition(
                            item,
                            "iteration_variable",
                            &child,
                            span,
                            item_type.as_ref().map(type_ref_label),
                            None,
                            false,
                            true,
                        );
                    }
                    self.collect_body(body, &child, false);
                }
                Stmt::Quantifier {
                    span,
                    variable,
                    domain,
                    body,
                    ..
                } => {
                    let child = self.child_scope_for_body(&scope_id, body, "quantifier");
                    if let Some(span) = self.name_span(variable, *span, false) {
                        self.add_definition(
                            variable,
                            "quantifier_variable",
                            &child,
                            span,
                            Some(type_ref_label(domain)),
                            None,
                            false,
                            true,
                        );
                    }
                    self.collect_body(body, &child, false);
                }
                Stmt::Try { action, body, .. } => {
                    self.collect_expr(action, &scope_id, false);
                    self.collect_body(body, &scope_id, true);
                }
                Stmt::While {
                    condition, body, ..
                }
                | Stmt::Guard {
                    condition, body, ..
                }
                | Stmt::HookWhenBecomes {
                    condition, body, ..
                }
                | Stmt::HookWhile {
                    condition, body, ..
                } => {
                    self.collect_expr(condition, &scope_id, false);
                    self.collect_body(body, &scope_id, true);
                }
                Stmt::Repeat { body, .. }
                | Stmt::BeatBlock { body, .. }
                | Stmt::Hook { body, .. } => self.collect_body(body, &scope_id, true),
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.collect_expr(condition, &scope_id, false);
                    self.collect_body(then_body, &scope_id, true);
                    if let Some(body) = else_body {
                        self.collect_body(body, &scope_id, true);
                    }
                }
                Stmt::Choose {
                    branches,
                    else_body,
                    ..
                } => {
                    for branch in branches {
                        self.collect_expr(&branch.condition, &scope_id, false);
                        self.collect_body(&branch.body, &scope_id, true);
                    }
                    self.collect_body(else_body, &scope_id, true);
                }
                Stmt::Contract {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.collect_expr(condition, &scope_id, false);
                    if let Some(body) = then_body {
                        self.collect_body(body, &scope_id, true);
                    }
                    self.collect_body(else_body, &scope_id, true);
                }
                Stmt::Expr { expr, .. } => self.collect_expr(expr, &scope_id, false),
                Stmt::Show { expr, .. }
                | Stmt::Inspect { expr, .. }
                | Stmt::Return { value: expr, .. } => self.collect_expr(expr, &scope_id, false),
                Stmt::PublicObservation { span, fields, .. } => {
                    let observation_scope = self
                        .map(*span)
                        .map(|span| {
                            self.add_scope(
                                Some(scope_id.clone()),
                                "public_observation",
                                span.0,
                                span.1,
                            )
                        })
                        .unwrap_or_else(|| scope_id.clone());
                    for field in fields {
                        let shorthand = field.name_span == field.value.span
                            && matches!(
                                &field.value.kind,
                                ExprKind::Var(name) if name == &field.name
                            );
                        if !shorthand {
                            if let Some(field_span) = self.map(field.name_span) {
                                self.add_definition(
                                    &field.name,
                                    "public_observation_field",
                                    &observation_scope,
                                    field_span,
                                    None,
                                    explicit_unit(&field.value),
                                    false,
                                    true,
                                );
                                self.set_last_value_label(&field.value);
                            }
                        }
                        self.collect_expr(&field.value, &scope_id, false);
                    }
                }
                Stmt::Send {
                    sender,
                    payload,
                    receiver,
                    ..
                } => {
                    if let Some(sender) = sender {
                        self.collect_expr(sender, &scope_id, false);
                    }
                    self.collect_expr(payload, &scope_id, false);
                    self.collect_expr(receiver, &scope_id, false);
                }
                Stmt::Break { .. }
                | Stmt::ContinueLoop { .. }
                | Stmt::MetaBlock { .. }
                | Stmt::Pragma { .. } => {}
            }
        }
    }

    fn collect_expr(&mut self, expr: &Expr, scope_id: &str, write: bool) {
        match &expr.kind {
            ExprKind::Thunk(body) => self.collect_body(body, scope_id, true),
            ExprKind::SeedLiteral { body, .. }
            | ExprKind::Nuance { expr: body, .. }
            | ExprKind::Suffix { value: body, .. } => self.collect_expr(body, scope_id, write),
            ExprKind::Var(name) => {
                if let Some(span) = self.map(expr.span) {
                    self.record_occurrence(name, scope_id, span, write);
                }
            }
            ExprKind::FieldAccess { .. } => {
                if let Some(path) = field_access_path(expr) {
                    if let Some(span) = self.map(expr.span) {
                        self.record_occurrence(&path, scope_id, span, write);
                    }
                } else if let ExprKind::FieldAccess { target, .. } = &expr.kind {
                    self.collect_expr(target, scope_id, write);
                }
            }
            ExprKind::Call { args, func } => {
                for arg in args {
                    self.collect_expr(&arg.expr, scope_id, false);
                }
                let span = if func.contains('.') {
                    self.qualified_name_span(func, expr.span)
                } else {
                    self.name_span(func, expr.span, true)
                };
                if let Some(span) = span {
                    self.record_occurrence(func, scope_id, span, false);
                }
            }
            ExprKind::Infix { left, right, .. } => {
                self.collect_expr(left, scope_id, false);
                self.collect_expr(right, scope_id, false);
            }
            ExprKind::Eval { thunk, .. } => self.collect_expr(thunk, scope_id, false),
            ExprKind::Pipe { stages } => {
                for stage in stages {
                    self.collect_expr(stage, scope_id, false);
                }
            }
            ExprKind::Pack { fields } => {
                for (_, value) in fields {
                    self.collect_expr(value, scope_id, false);
                }
            }
            ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
                for (_, value) in inject {
                    self.collect_expr(value, scope_id, false);
                }
            }
            ExprKind::Literal(_)
            | ExprKind::FlowValue
            | ExprKind::Assertion(_)
            | ExprKind::StateMachine(_)
            | ExprKind::Formula(_)
            | ExprKind::Template(_) => {}
        }
    }

    fn add_scope(
        &mut self,
        parent_id: Option<String>,
        kind: &'static str,
        start: usize,
        end: usize,
    ) -> String {
        self.scope_serial += 1;
        let id = format!("scope:{}:{}:{}", self.path, start, self.scope_serial);
        if let Some(range) = source_range(self.source, start, end) {
            self.scopes.push(AuthoringScope {
                id: id.clone(),
                path: self.path.to_string(),
                parent_id,
                kind,
                range,
            });
        }
        id
    }

    fn collect_record_field_definitions(&mut self, root: &str, value: &Expr, scope_id: &str) {
        let ExprKind::Pack { fields } = &value.kind else {
            return;
        };
        for (field, field_value) in fields {
            let Some(span) = self.record_field_name_span(field, value.span, field_value.span) else {
                continue;
            };
            let path = format!("{root}.{field}");
            self.add_definition(
                &path,
                "record_field",
                scope_id,
                span,
                Some(record_field_type_label(field_value)),
                explicit_unit(field_value),
                false,
                true,
            );
            self.set_last_value_label(field_value);
            self.collect_record_field_definitions(&path, field_value, scope_id);
        }
    }

    fn record_field_name_span(
        &self,
        name: &str,
        pack_span: Span,
        value_span: Span,
    ) -> Option<(usize, usize)> {
        let pack = self.map(pack_span)?;
        let value = self.map(value_span)?;
        self.tokens
            .iter()
            .filter(|token| {
                token.span.start >= pack.0
                    && token.span.end <= value.0
                    && token_name(token).is_some_and(|candidate| candidate == name)
            })
            .last()
            .map(|token| (token.span.start, token.span.end))
    }

    fn child_scope_for_body(&mut self, parent: &str, body: &Body, kind: &'static str) -> String {
        self.map(body.span)
            .map(|span| self.add_scope(Some(parent.to_string()), kind, span.0, span.1))
            .unwrap_or_else(|| parent.to_string())
    }

    fn file_scope_id(&self) -> &str {
        self.scopes
            .iter()
            .find(|scope| scope.path == self.path && scope.kind == "file")
            .map(|scope| scope.id.as_str())
            .unwrap_or("")
    }

    fn definition_visible(&self, normalized_name: &str, scope_id: &str) -> bool {
        let mut current = Some(scope_id);
        while let Some(id) = current {
            if self.definitions.iter().any(|definition| {
                definition.path == self.path
                    && definition.scope_id == id
                    && definition.normalized_name == normalized_name
            }) {
                return true;
            }
            current = self
                .scopes
                .iter()
                .find(|scope| scope.id == id)
                .and_then(|scope| scope.parent_id.as_deref());
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn add_definition(
        &mut self,
        name: &str,
        kind: &'static str,
        scope_id: &str,
        span: (usize, usize),
        type_label: Option<String>,
        unit: Option<String>,
        constant: bool,
        explicit: bool,
    ) {
        let Some(range) = source_range(self.source, span.0, span.1) else {
            return;
        };
        let normalized_name = name.nfc().collect::<String>();
        let export_names = self
            .exports
            .iter()
            .filter(|row| row.module_path == self.path && row.internal_name == name)
            .map(|row| row.external_name.clone())
            .collect::<Vec<_>>();
        let id = format!(
            "def:{}:{}:{}:{}",
            self.path,
            span.0,
            kind,
            sha256_hex(normalized_name.as_bytes())
        );
        self.definitions.push(AuthoringDefinition {
            id,
            name: name.to_string(),
            normalized_name,
            kind,
            path: self.path.to_string(),
            scope_id: scope_id.to_string(),
            range,
            type_label,
            unit,
            value_label: None,
            constant,
            explicit,
            export_names,
        });
    }

    fn set_last_value_label(&mut self, expr: &Expr) {
        let Some((start, end)) = self.map(expr.span) else {
            return;
        };
        let Some(raw) = self.source.get(start..end) else {
            return;
        };
        let compact = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if compact.is_empty() || compact.chars().count() > 120 {
            return;
        }
        if let Some(definition) = self.definitions.last_mut() {
            definition.value_label = Some(compact);
        }
    }

    fn map(&self, span: Span) -> Option<(usize, usize)> {
        map_span(span, self.shift, self.source.len()).map(|span| (span.start, span.end))
    }

    fn name_span(&self, name: &str, node_span: Span, last: bool) -> Option<(usize, usize)> {
        let node = self.map(node_span)?;
        let mut matches = self.tokens.iter().filter(|token| {
            token.span.start >= node.0
                && token.span.end <= node.1
                && token_name(token).is_some_and(|candidate| candidate == name)
        });
        let token = if last { matches.last() } else { matches.next() }?;
        Some((token.span.start, token.span.end))
    }

    fn qualified_name_span(&self, name: &str, node_span: Span) -> Option<(usize, usize)> {
        let (alias, symbol) = name.split_once('.')?;
        if symbol.contains('.') {
            return None;
        }
        let node = self.map(node_span)?;
        self.tokens.windows(3).rev().find_map(|window| {
            let [left, dot, right] = window else {
                return None;
            };
            if left.span.start < node.0
                || right.span.end > node.1
                || token_name(left) != Some(alias)
                || !matches!(dot.kind, TokenKind::Dot)
                || token_name(right) != Some(symbol)
            {
                return None;
            }
            Some((left.span.start, right.span.end))
        })
    }

    fn record_occurrence(&mut self, name: &str, scope_id: &str, span: (usize, usize), write: bool) {
        self.occurrences.push(RawOccurrence {
            name: name.to_string(),
            path: self.path.to_string(),
            scope_id: scope_id.to_string(),
            start: span.0,
            end: span.1,
            write,
        });
    }
}

fn token_name(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(name) | TokenKind::Josa(name) => Some(name),
        _ => None,
    }
}

fn map_span(span: Span, shift: usize, source_len: usize) -> Option<Span> {
    if span.start < shift || span.end < shift {
        return None;
    }
    let start = span.start - shift;
    let end = span.end - shift;
    if start > end || end > source_len {
        return None;
    }
    Some(Span { start, end })
}

fn source_range(source: &str, start: usize, end: usize) -> Option<AuthoringSourceRange> {
    if start > end
        || end > source.len()
        || !source.is_char_boundary(start)
        || !source.is_char_boundary(end)
    {
        return None;
    }
    let (start_line, start_column_utf16) = line_column_utf16(source, start)?;
    let (end_line, end_column_utf16) = line_column_utf16(source, end)?;
    Some(AuthoringSourceRange {
        start,
        end,
        start_line,
        start_column_utf16,
        end_line,
        end_column_utf16,
    })
}

fn line_column_utf16(source: &str, byte_offset: usize) -> Option<(usize, usize)> {
    let position = utf16_position(source, byte_offset)?;
    Some((position.line as usize + 1, position.character as usize + 1))
}

fn type_ref_label(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Applied { name, args } => format!(
            "({}){}",
            args.iter()
                .map(type_ref_label)
                .collect::<Vec<_>>()
                .join(", "),
            name
        ),
        TypeRef::Function { params, result } => format!(
            "({}) --> {}",
            params
                .iter()
                .map(type_ref_label)
                .collect::<Vec<_>>()
                .join(", "),
            type_ref_label(result)
        ),
        TypeRef::Infer => "_".to_string(),
    }
}

fn explicit_unit(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Suffix {
            at: AtSuffix::Unit(unit),
            ..
        } => Some(unit.clone()),
        _ => None,
    }
}

fn record_field_type_label(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Pack { .. } => "값꾸러미".to_string(),
        ExprKind::Literal(Literal::Int(_) | Literal::Fixed64(_)) => "수".to_string(),
        ExprKind::Literal(Literal::String(_)) => "글".to_string(),
        ExprKind::Literal(Literal::Bool(_)) => "참거짓".to_string(),
        _ => "필드값".to_string(),
    }
}

fn field_access_path(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Var(name) => Some(name.clone()),
        ExprKind::FieldAccess { target, field, .. } => {
            Some(format!("{}.{}", field_access_path(target)?, field))
        }
        _ => None,
    }
}

fn seed_kind_label(kind: &SeedKind) -> &'static str {
    match kind {
        SeedKind::Imeumssi => "name_seed",
        SeedKind::Umjikssi => "action_seed",
        SeedKind::ValueFunc | SeedKind::Semssi => "value_seed",
        SeedKind::Gallaessi => "branch_seed",
        SeedKind::Relationssi => "relation_seed",
        SeedKind::Sam => "sample_seed",
        SeedKind::Heureumssi => "flow_seed",
        SeedKind::Ieumssi => "link_seed",
        SeedKind::Named(_) => "named_seed",
    }
}

fn is_ddn_source(path: &str, role: &str) -> bool {
    path.ends_with(".ddn") && matches!(role, "ddn_source" | "entry" | "root")
}

fn diagnostic_from_local_error(
    err: &LocalProjectError,
    source: &VirtualProjectSource,
) -> AuthoringDiagnostic {
    let source_text = err.path.as_ref().and_then(|path| {
        source
            .files
            .iter()
            .find(|file| &file.path == path)
            .map(|file| file.utf8_text.as_str())
    });
    AuthoringDiagnostic {
        code: err.code.to_string(),
        message: err.message.clone(),
        severity: "error",
        source_ref: err.path.clone(),
        source_range: err
            .source_span
            .and_then(|span| source_text.and_then(|text| source_range(text, span.0, span.1))),
    }
}

fn resolve_references(
    occurrences: &[RawOccurrence],
    source_by_path: &BTreeMap<String, String>,
    scopes: &[AuthoringScope],
    definitions: &[AuthoringDefinition],
    imports: &[AuthoringImport],
    exports: &[AuthoringExport],
) -> Vec<AuthoringReference> {
    occurrences
        .iter()
        .filter_map(|occurrence| {
            let source = source_by_path.get(&occurrence.path)?;
            let range = source_range(source, occurrence.start, occurrence.end)?;
            let normalized_name = occurrence.name.nfc().collect::<String>();
            let (local_definition, local_resolution) =
                resolve_simple_reference(occurrence, &normalized_name, scopes, definitions);
            let (definition_id, import_id, resolution) = if local_definition.is_some()
                || !occurrence.name.contains('.')
            {
                (local_definition, None, local_resolution)
            } else if let Some((alias, symbol)) = occurrence.name.split_once('.') {
                resolve_qualified_reference(
                    &occurrence.path,
                    alias,
                    symbol,
                    imports,
                    exports,
                    scopes,
                    definitions,
                )
            } else {
                (None, None, "unresolved")
            };
            Some(AuthoringReference {
                name: occurrence.name.clone(),
                normalized_name,
                path: occurrence.path.clone(),
                scope_id: occurrence.scope_id.clone(),
                range,
                definition_id,
                import_id,
                resolution,
                write: occurrence.write,
            })
        })
        .collect()
}

fn resolve_simple_reference(
    occurrence: &RawOccurrence,
    normalized_name: &str,
    scopes: &[AuthoringScope],
    definitions: &[AuthoringDefinition],
) -> (Option<String>, &'static str) {
    let ancestors = scope_ancestors(&occurrence.scope_id, scopes);
    let mut candidates = definitions
        .iter()
        .filter(|definition| {
            definition.path == occurrence.path
                && definition.normalized_name == normalized_name
                && ancestors.iter().any(|scope| scope == &definition.scope_id)
                && (definition.range.start <= occurrence.start
                    || definition.kind.ends_with("_seed"))
        })
        .map(|definition| (scope_depth(&definition.scope_id, scopes), definition))
        .collect::<Vec<_>>();
    let Some(max_depth) = candidates.iter().map(|(depth, _)| *depth).max() else {
        return (None, "unresolved");
    };
    candidates.retain(|(depth, _)| *depth == max_depth);
    candidates.sort_by(|(_, a), (_, b)| a.id.cmp(&b.id));
    if candidates.len() != 1 {
        return (None, "ambiguous");
    }
    (Some(candidates[0].1.id.clone()), "local")
}

fn resolve_qualified_reference(
    importer_path: &str,
    alias: &str,
    symbol: &str,
    imports: &[AuthoringImport],
    exports: &[AuthoringExport],
    scopes: &[AuthoringScope],
    definitions: &[AuthoringDefinition],
) -> (Option<String>, Option<String>, &'static str) {
    let matching_imports = imports
        .iter()
        .filter(|row| row.importer_path == importer_path && row.alias == alias)
        .collect::<Vec<_>>();
    if matching_imports.len() != 1 {
        return (
            None,
            None,
            if matching_imports.is_empty() {
                "unresolved"
            } else {
                "ambiguous"
            },
        );
    }
    let import = matching_imports[0];
    let mut visited = BTreeSet::new();
    match resolve_export_definition(
        &import.resolved_path,
        symbol,
        imports,
        exports,
        scopes,
        definitions,
        &mut visited,
    ) {
        ExportResolution::One(id) => (Some(id), Some(import.id.clone()), "imported_qualified"),
        ExportResolution::Missing => (None, Some(import.id.clone()), "unresolved"),
        ExportResolution::Ambiguous => (None, Some(import.id.clone()), "ambiguous"),
    }
}

enum ExportResolution {
    One(String),
    Missing,
    Ambiguous,
}

#[allow(clippy::too_many_arguments)]
fn resolve_export_definition(
    module_path: &str,
    external_name: &str,
    imports: &[AuthoringImport],
    exports: &[AuthoringExport],
    _scopes: &[AuthoringScope],
    definitions: &[AuthoringDefinition],
    visited: &mut BTreeSet<String>,
) -> ExportResolution {
    let visit_key = format!("{module_path}::{external_name}");
    if !visited.insert(visit_key) {
        return ExportResolution::Ambiguous;
    }
    let matching_exports = exports
        .iter()
        .filter(|row| row.module_path == module_path && row.external_name == external_name)
        .collect::<Vec<_>>();
    if matching_exports.len() != 1 {
        return if matching_exports.is_empty() {
            ExportResolution::Missing
        } else {
            ExportResolution::Ambiguous
        };
    }
    let internal_name = &matching_exports[0].internal_name;
    if let Some((alias, symbol)) = internal_name.split_once('.') {
        let matching_imports = imports
            .iter()
            .filter(|row| row.importer_path == module_path && row.alias == alias)
            .collect::<Vec<_>>();
        if matching_imports.len() != 1 {
            return if matching_imports.is_empty() {
                ExportResolution::Missing
            } else {
                ExportResolution::Ambiguous
            };
        }
        return resolve_export_definition(
            &matching_imports[0].resolved_path,
            symbol,
            imports,
            exports,
            _scopes,
            definitions,
            visited,
        );
    }
    let matches = definitions
        .iter()
        .filter(|definition| definition.path == module_path && definition.name == *internal_name)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [definition] => ExportResolution::One(definition.id.clone()),
        [] => ExportResolution::Missing,
        _ => ExportResolution::Ambiguous,
    }
}

fn scope_ancestors(scope_id: &str, scopes: &[AuthoringScope]) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = Some(scope_id);
    let mut seen = BTreeSet::new();
    while let Some(id) = current {
        if !seen.insert(id.to_string()) {
            break;
        }
        result.push(id.to_string());
        current = scopes
            .iter()
            .find(|scope| scope.id == id)
            .and_then(|scope| scope.parent_id.as_deref());
    }
    result
}

fn scope_depth(scope_id: &str, scopes: &[AuthoringScope]) -> usize {
    scope_ancestors(scope_id, scopes).len()
}

fn stable_snapshot_sha256(
    snapshot: &ProjectAuthoringSnapshot,
) -> Result<String, LocalProjectError> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema": snapshot.schema,
        "source_version": snapshot.source_version,
        "project_source_sha256": snapshot.project_source_sha256,
        "project_graph_sha256": snapshot.project_graph_sha256,
        "root_path": snapshot.root_path,
        "entry_path": snapshot.entry_path,
        "files": snapshot.files,
        "imports": snapshot.imports,
        "exports": snapshot.exports,
        "scopes": snapshot.scopes,
        "definitions": snapshot.definitions,
        "references": snapshot.references,
        "diagnostics": snapshot.diagnostics,
    }))
    .map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_AUTHORING_HASH",
            format!("authoring snapshot hash 입력을 만들 수 없습니다: {err}"),
        )
    })?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_project::{
        VirtualProjectFileSource, LOCAL_PROJECT_TREE_KIND, VIRTUAL_PROJECT_SOURCE_SCHEMA,
    };

    fn project(files: Vec<(&str, &str, &str)>, entry: &str) -> VirtualProjectSource {
        VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: entry.to_string(),
            files: files
                .into_iter()
                .map(|(path, role, utf8_text)| VirtualProjectFileSource {
                    path: path.to_string(),
                    role: role.to_string(),
                    utf8_text: utf8_text.to_string(),
                })
                .collect(),
        }
    }

    fn root(entry: &str) -> String {
        format!("프로젝트 {{\n  이름: \"저작검사\".\n  첫글: \"{entry}\".\n}}\n")
    }

    fn model_source() -> &'static str {
        r#"정밀한방법 <- (방식: #정밀).

비례계산 = 모형 {
  입력:나눔수 :: (들임, 내줌).
  배수:나눔수 :: (들임).
  결과:나눔수 :: (그자리, 내줌).
  결과::구함.
  결과 =:= 입력 * 배수.
}.

계산 <- (
  대상=비례계산,
  들임=(입력: 3, 배수: 2),
  방법=정밀한방법,
  내줌=(결과, 입력)
) 풀기.

(결과: 계산.결과, 입력: 계산.입력) 보임.
"#
    }

    fn exact_method_module() -> &'static str {
        r#"드러냄 {
  정밀한방법.
}

정밀한방법 <- (방식: #정밀).
"#
    }

    fn multifile_model_entry(import_block: &str, method_path: &str, prefix: &str) -> String {
        format!(
            r#"{import_block}
{prefix}다중파일비례 = 모형 {{
  입력:나눔수 :: (들임, 내줌).
  배수:나눔수 :: (들임, 내줌).
  결과:나눔수 :: (그자리, 내줌).
  결과::구함.
  결과 =:= 입력 * 배수.
}}.

계산 <- (
  대상=다중파일비례,
  들임=(입력: 5, 배수: 2),
  방법={method_path},
  내줌=(결과, 입력, 배수)
) 풀기.

(결과: 계산.결과, 입력: 계산.입력, 배수: 계산.배수) 보임.
"#
        )
    }

    fn multifile_model_project(entry: &str, module: &str) -> VirtualProjectSource {
        project(
            vec![
                ("뿌리.ddn", "root", &root("app/main.ddn")),
                ("app/main.ddn", "ddn_source", entry),
                ("lib/method.ddn", "ddn_source", module),
            ],
            "app/main.ddn",
        )
    }

    fn authoring_snapshot(
        source: VirtualProjectSource,
        source_version: &str,
    ) -> ProjectAuthoringSnapshot {
        build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source,
            source_version: source_version.to_string(),
        })
        .expect("project authoring snapshot")
    }

    fn diagnostic<'a>(
        snapshot: &'a ProjectAuthoringSnapshot,
        code: &str,
    ) -> &'a AuthoringDiagnostic {
        snapshot
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == code)
            .unwrap_or_else(|| panic!("missing diagnostic {code}: {:?}", snapshot.diagnostics))
    }

    #[test]
    fn model_typed_ir_authoring_links_definitions_references_and_bounded_completions() {
        let text = model_source();
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("lesson.ddn")),
                ("lesson.ddn", "ddn_source", text),
            ],
            "lesson.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "model-edit-1".to_string(),
        })
        .expect("model snapshot");
        assert_eq!(
            snapshot
                .files
                .iter()
                .find(|file| file.path == "lesson.ddn")
                .expect("model file")
                .parse_status,
            "model_typed_ir"
        );
        assert!(snapshot.definitions.iter().any(|definition| {
            definition.name == "비례계산" && definition.kind == "model_definition"
        }));
        let result_definition = snapshot
            .definitions
            .iter()
            .find(|definition| definition.name == "결과")
            .expect("result definition");
        assert_eq!(result_definition.kind, "model_unknown");
        assert_eq!(result_definition.type_label.as_deref(), Some("나눔수"));
        assert!(result_definition
            .value_label
            .as_deref()
            .is_some_and(|label| label.contains("구함") && label.contains("내줌")));
        assert!(snapshot.references.iter().any(|reference| {
            reference.name == "결과"
                && reference.resolution == "local"
                && reference.definition_id.as_deref() == Some(result_definition.id.as_str())
        }));

        let property_offset = text.find("결과::구함").expect("property") + "결과::구".len();
        let property = build_project_authoring_completion(ProjectAuthoringCursorRequest {
            schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "model-edit-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "lesson.ddn".to_string(),
            byte_offset: property_offset,
        })
        .expect("property completion");
        assert_eq!(
            property
                .items
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            ["구함"]
        );

        let output_offset = text.find("내줌=(결과").expect("output") + "내줌=(결".len();
        let output = build_project_authoring_completion(ProjectAuthoringCursorRequest {
            schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "model-edit-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "lesson.ddn".to_string(),
            byte_offset: output_offset,
        })
        .expect("output completion");
        assert_eq!(
            output
                .items
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            ["결과"]
        );
        assert!(!output.items.iter().any(|item| item.label == "배수"));

        let method_offset = text.find("#정밀").expect("method") + "#정".len();
        let method_completion = build_project_authoring_completion(ProjectAuthoringCursorRequest {
            schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "model-edit-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "lesson.ddn".to_string(),
            byte_offset: method_offset,
        })
        .expect("method completion");
        assert_eq!(method_completion.items.len(), 1);
        assert_eq!(method_completion.items[0].insert_text, "#정밀");
        assert!(!serde_json::to_string(&method_completion)
            .unwrap()
            .contains("ddn.exact_acyclic_relation.v1"));

        let method_hover = build_project_authoring_hover(ProjectAuthoringCursorRequest {
            schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
            source,
            source_version: "model-edit-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256,
            expected_snapshot_sha256: snapshot.snapshot_sha256,
            path: "lesson.ddn".to_string(),
            byte_offset: method_offset,
        })
        .expect("method hover");
        assert_eq!(method_hover.kind.as_deref(), Some("solve_method"));
        assert_eq!(method_hover.value_label.as_deref(), Some("#정밀"));
        assert!(!serde_json::to_string(&method_hover)
            .unwrap()
            .contains("ddn.exact_acyclic_relation.v1"));
    }

    #[test]
    fn model_cycle_diagnostic_keeps_exact_source_range() {
        let text = crate::self_contained_test_sources::MODEL_RELATION_CYCLE;
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: project(
                vec![
                    ("뿌리.ddn", "root", &root("cycle.ddn")),
                    ("cycle.ddn", "ddn_source", text),
                ],
                "cycle.ddn",
            ),
            source_version: "cycle-edit-1".to_string(),
        })
        .expect("cycle snapshot");
        let diagnostic = snapshot
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "E_MODEL_RELATION_CYCLE_CANDIDATE")
            .expect("cycle diagnostic");
        assert_eq!(diagnostic.source_ref.as_deref(), Some("cycle.ddn"));
        let range = diagnostic
            .source_range
            .as_ref()
            .expect("exact source range");
        assert!(range.start < range.end);
        assert!(range.end <= text.len());
    }

    #[test]
    fn snapshot_is_stable_and_collects_flat_ast_definitions_without_runtime() {
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                ("main.ddn", "ddn_source", "값 <- 1.\n값 보여주기.\n"),
            ],
            "main.ddn",
        );
        let request = ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "edit-1".to_string(),
        };
        let first = build_project_authoring_snapshot(request).expect("snapshot");
        let second = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source,
            source_version: "edit-1".to_string(),
        })
        .expect("same snapshot");
        assert_eq!(first.snapshot_sha256, second.snapshot_sha256);
        assert_eq!(
            first
                .files
                .iter()
                .find(|file| file.path == "main.ddn")
                .expect("main")
                .parse_status,
            "wrapped"
        );
        assert_eq!(first.definitions.len(), 1);
        assert_eq!(first.definitions[0].name, "값");
        assert!(!first.definitions[0].explicit);
        assert!(first.definitions[0].range.start < first.definitions[0].range.end);
        assert!(first.references.iter().any(|reference| {
            reference.name == "값"
                && reference.resolution == "local"
                && reference.definition_id.as_deref() == Some(first.definitions[0].id.as_str())
        }));
        let json = serde_json::to_string(&first).unwrap();
        let decoded: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded["snapshot_sha256"], first.snapshot_sha256);
    }

    #[test]
    fn snapshot_masks_metadata_with_offsets_and_links_import_export_rows() {
        let root_text = root("main.ddn");
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root_text),
                (
                    "main.ddn",
                    "ddn_source",
                    "쓰임 { 도구: \"lib\". }\n결과 <- 도구.값.\n",
                ),
                (
                    "lib.ddn",
                    "ddn_source",
                    "드러냄 { 값. }\n채비 {\n  값: 수 <- 3.\n}.\n",
                ),
            ],
            "main.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source,
            source_version: "edit-2".to_string(),
        })
        .expect("snapshot");
        assert_eq!(snapshot.imports.len(), 1);
        assert_eq!(snapshot.imports[0].resolved_path, "lib.ddn");
        assert_eq!(snapshot.exports.len(), 1);
        assert!(snapshot
            .definitions
            .iter()
            .any(|row| row.path == "lib.ddn" && row.export_names == ["값"]));
        let imported = snapshot
            .references
            .iter()
            .find(|row| row.name == "도구.값")
            .expect("qualified reference");
        assert_eq!(imported.resolution, "imported_qualified");
        assert!(imported.definition_id.is_some());
        assert_eq!(
            imported.import_id.as_deref(),
            Some(snapshot.imports[0].id.as_str())
        );
    }

    #[test]
    fn rd203_record_fields_share_definition_hover_and_completion_binding() {
        let text = "사람 <- (이름: \"또니\", 주소: (도시: \"서울\")).\n선택 <- 사람.주소.도시.\n";
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                ("main.ddn", "ddn_source", text),
            ],
            "main.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "rd203-authoring-1".to_string(),
        })
        .expect("snapshot");
        let city_definition = snapshot
            .definitions
            .iter()
            .find(|definition| definition.name == "사람.주소.도시")
            .expect("nested record field definition");
        assert_eq!(
            &text[city_definition.range.start..city_definition.range.end],
            "도시"
        );
        let city_reference = snapshot
            .references
            .iter()
            .find(|reference| reference.name == "사람.주소.도시")
            .expect("nested field reference");
        assert_eq!(city_reference.resolution, "local");
        assert_eq!(
            city_reference.definition_id.as_deref(),
            Some(city_definition.id.as_str())
        );

        let cursor = city_reference.range.end;
        let hover = build_project_authoring_hover(ProjectAuthoringCursorRequest {
            schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "rd203-authoring-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "main.ddn".to_string(),
            byte_offset: city_reference.range.end - 1,
        })
        .expect("field hover");
        assert_eq!(hover.status, "found");
        assert_eq!(hover.kind.as_deref(), Some("record_field"));

        let completion = build_project_authoring_completion(ProjectAuthoringCursorRequest {
            schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
            source,
            source_version: "rd203-authoring-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256,
            expected_snapshot_sha256: snapshot.snapshot_sha256,
            path: "main.ddn".to_string(),
            byte_offset: cursor,
        })
        .expect("field completion");
        assert!(completion.items.iter().any(|item| {
            item.kind == "record_field" && item.label == "도시" && item.insert_text == "도시"
        }));
    }

    #[test]
    fn byte_and_utf16_positions_cover_korean_combining_emoji_crlf_and_bom() {
        let cases = [
            ("앞줄\n값", 7, 10, 2, 1),
            ("e\u{301}", 0, 3, 1, 1),
            ("글🙂", 3, 7, 1, 2),
            ("앞줄\r\n값", 8, 11, 2, 1),
            ("\u{feff}값", 3, 6, 1, 2),
        ];
        for (source, start, end, line, column) in cases {
            let range = source_range(source, start, end).expect("valid range");
            assert_eq!((range.start_line, range.start_column_utf16), (line, column));
        }
        assert!(source_range("🙂", 1, 4).is_none());
    }

    #[test]
    fn strings_and_comments_do_not_create_definitions() {
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                (
                    "main.ddn",
                    "ddn_source",
                    "실제 <- 1.\n글 <- \"가짜\".\n// 주석가짜 <- 2.\n",
                ),
            ],
            "main.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source,
            source_version: "edit-3".to_string(),
        })
        .expect("snapshot");
        let names = snapshot
            .definitions
            .iter()
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["실제", "글"]);
    }

    #[test]
    fn parse_failure_is_a_file_diagnostic_and_other_project_state_never_leaks() {
        let invalid = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                ("main.ddn", "ddn_source", "망가짐 <- .\n"),
            ],
            "main.ddn",
        );
        let failed = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: invalid,
            source_version: "invalid-1".to_string(),
        })
        .expect("bounded snapshot with diagnostic");
        assert_eq!(
            failed
                .files
                .iter()
                .find(|file| file.path == "main.ddn")
                .expect("main")
                .parse_status,
            "unavailable"
        );
        assert!(failed
            .diagnostics
            .iter()
            .any(|row| row.code == "E_PROJECT_AUTHORING_PARSE_UNAVAILABLE"));

        let clean = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                ("main.ddn", "ddn_source", "새값 <- 2@m.\n"),
            ],
            "main.ddn",
        );
        let clean = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: clean,
            source_version: "clean-1".to_string(),
        })
        .expect("clean snapshot");
        assert_eq!(clean.definitions.len(), 1);
        assert_eq!(clean.definitions[0].name, "새값");
        assert_eq!(clean.definitions[0].unit.as_deref(), Some("m"));
        assert!(!clean
            .definitions
            .iter()
            .any(|definition| definition.name == "망가짐"));
    }

    #[test]
    fn bom_is_masked_for_parse_but_definition_range_stays_in_original_bytes() {
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                ("main.ddn", "ddn_source", "\u{feff}값 <- 1.\r\n"),
            ],
            "main.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source,
            source_version: "bom-1".to_string(),
        })
        .expect("snapshot");
        assert_eq!(snapshot.definitions[0].range.start, 3);
        assert_eq!(snapshot.definitions[0].range.start_column_utf16, 2);
    }

    #[test]
    fn same_name_references_bind_to_their_own_lexical_scope() {
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                (
                    "main.ddn",
                    "ddn_source",
                    "(값:수) 첫째:셈씨 = { 값 돌려줘. }\n(값:수) 둘째:셈씨 = { 값 돌려줘. }\n",
                ),
            ],
            "main.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source,
            source_version: "scope-1".to_string(),
        })
        .expect("snapshot");
        let definitions = snapshot
            .definitions
            .iter()
            .filter(|definition| definition.name == "값")
            .collect::<Vec<_>>();
        let references = snapshot
            .references
            .iter()
            .filter(|reference| reference.name == "값")
            .collect::<Vec<_>>();
        assert_eq!(definitions.len(), 2);
        assert_eq!(references.len(), 2);
        assert_ne!(references[0].definition_id, references[1].definition_id);
        assert!(references
            .iter()
            .all(|reference| reference.resolution == "local"));
    }

    #[test]
    fn definition_and_references_use_bound_ast_and_reject_stale_identity() {
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                (
                    "main.ddn",
                    "ddn_source",
                    "쓰임 { 도구: \"lib\". }\n결과 <- 도구.값.\n도구.값 보여주기.\n글 <- \"도구.값\".\n// 도구.값\n",
                ),
                (
                    "lib.ddn",
                    "ddn_source",
                    "드러냄 { 값. }\n값 <- 3.\n값 보여주기.\n",
                ),
            ],
            "main.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "query-1".to_string(),
        })
        .expect("snapshot");
        let qualified = snapshot
            .references
            .iter()
            .find(|reference| reference.name == "도구.값")
            .expect("qualified reference");
        assert_eq!(
            snapshot
                .references
                .iter()
                .filter(|reference| reference.name == "도구.값")
                .count(),
            2,
            "string/comment text must not become references"
        );
        let definition = build_project_authoring_navigation(
            ProjectAuthoringCursorRequest {
                schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
                source: source.clone(),
                source_version: "query-1".to_string(),
                expected_project_source_sha256: snapshot.project_source_sha256.clone(),
                expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
                path: "main.ddn".to_string(),
                byte_offset: qualified.range.start,
            },
            "definition",
        )
        .expect("definition");
        assert_eq!(definition.status, "found");
        assert_eq!(definition.declaration.as_ref().unwrap().path, "lib.ddn");
        assert_eq!(definition.declaration.as_ref().unwrap().name, "값");

        let declaration = snapshot
            .definitions
            .iter()
            .find(|definition| definition.path == "lib.ddn" && definition.name == "값")
            .expect("lib definition");
        let references = build_project_authoring_navigation(
            ProjectAuthoringCursorRequest {
                schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
                source: source.clone(),
                source_version: "query-1".to_string(),
                expected_project_source_sha256: snapshot.project_source_sha256.clone(),
                expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
                path: "lib.ddn".to_string(),
                byte_offset: declaration.range.start,
            },
            "references",
        )
        .expect("references");
        assert_eq!(references.status, "found");
        assert!(references
            .references
            .iter()
            .any(|location| location.path == "main.ddn" && location.name == "도구.값"));
        assert!(references
            .references
            .iter()
            .any(|location| location.path == "lib.ddn" && location.name == "값"));

        let stale = build_project_authoring_navigation(
            ProjectAuthoringCursorRequest {
                schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
                source,
                source_version: "query-1".to_string(),
                expected_project_source_sha256: "0".repeat(64),
                expected_snapshot_sha256: snapshot.snapshot_sha256,
                path: "main.ddn".to_string(),
                byte_offset: qualified.range.start,
            },
            "definition",
        )
        .expect_err("stale identity must fail");
        assert_eq!(stale.code, "E_PROJECT_AUTHORING_STALE");
    }

    #[test]
    fn hover_completion_and_rename_share_snapshot_binding_and_fail_closed() {
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                (
                    "main.ddn",
                    "ddn_source",
                    "쓰임 { 도구: \"lib\". }\n결과 <- 도구.값.\n글 <- \"도구.값\".\n// 도구.값\n",
                ),
                (
                    "lib.ddn",
                    "ddn_source",
                    "드러냄 { 값. }\n값 <- 3@m.\n값 보여주기.\n새값 <- 4.\n",
                ),
            ],
            "main.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "assist-1".to_string(),
        })
        .expect("snapshot");
        let definition = snapshot
            .definitions
            .iter()
            .find(|row| row.path == "lib.ddn" && row.name == "값")
            .expect("definition");
        let hover = build_project_authoring_hover(ProjectAuthoringCursorRequest {
            schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "assist-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "lib.ddn".to_string(),
            byte_offset: definition.range.start,
        })
        .expect("hover");
        assert_eq!(hover.status, "found");
        assert_eq!(hover.unit.as_deref(), Some("m"));
        assert_eq!(hover.value_label.as_deref(), Some("3@m"));
        assert!(hover
            .evidence
            .iter()
            .any(|row| row == "direct AST initializer"));

        let qualified = snapshot
            .references
            .iter()
            .find(|row| row.path == "main.ddn" && row.name == "도구.값")
            .expect("qualified reference");
        let completion = build_project_authoring_completion(ProjectAuthoringCursorRequest {
            schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "assist-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "main.ddn".to_string(),
            byte_offset: qualified.range.start + "도구.".len(),
        })
        .expect("completion");
        assert_eq!(completion.status, "found");
        assert!(completion.items.iter().any(|row| {
            row.label == "값" && row.kind == "imported_public_symbol" && row.origin == "lib"
        }));
        assert!(!completion.items.iter().any(|row| {
            matches!(row.label.as_str(), "보임" | "겹보기" | "곁보기" | "돋보기")
        }));

        let comment_offset = project_source_text(&source, "main.ddn")
            .unwrap()
            .rfind("도구.값")
            .unwrap();
        let comment_completion =
            build_project_authoring_completion(ProjectAuthoringCursorRequest {
                schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
                source: source.clone(),
                source_version: "assist-1".to_string(),
                expected_project_source_sha256: snapshot.project_source_sha256.clone(),
                expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
                path: "main.ddn".to_string(),
                byte_offset: comment_offset + "도구".len(),
            })
            .expect("comment completion");
        assert_eq!(comment_completion.status, "unavailable");
        assert!(comment_completion.items.is_empty());

        let plan = build_project_authoring_rename_preview(ProjectAuthoringRenameRequest {
            schema: PROJECT_AUTHORING_RENAME_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "assist-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "lib.ddn".to_string(),
            byte_offset: definition.range.start,
            new_name: "바뀐값".to_string(),
        })
        .expect("rename plan");
        assert_eq!(plan.status, "ready");
        assert_eq!(plan.affected_reference_count, 2);
        assert_eq!(plan.files.len(), 2);
        assert!(plan.files.iter().flat_map(|file| &file.edits).all(|edit| {
            !edit.old_text.contains("도구.값") && !edit.old_text.contains("//")
        }));
        assert!(plan
            .files
            .iter()
            .flat_map(|file| &file.edits)
            .any(|edit| edit.kind == "export"));

        let conflict = build_project_authoring_rename_preview(ProjectAuthoringRenameRequest {
            schema: PROJECT_AUTHORING_RENAME_REQUEST_SCHEMA.to_string(),
            source,
            source_version: "assist-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256,
            expected_snapshot_sha256: snapshot.snapshot_sha256,
            path: "lib.ddn".to_string(),
            byte_offset: definition.range.start,
            new_name: "새값".to_string(),
        })
        .expect("conflict plan");
        assert_eq!(conflict.status, "conflict");
        assert!(conflict.files.is_empty(), "conflict must be no-change");
    }

    #[test]
    fn public_observation_fields_use_exact_names_for_hover_references_and_atomic_rename() {
        let main = r#"채비 {
  시간: 수 <- 1.
  각도: 수 <- 15.
}.
(표시시간: 시간, 표시각도: 각도) 보임.
(각도) 보임.
"#;
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("main.ddn")),
                ("main.ddn", "ddn_source", main),
            ],
            "main.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "boim-authoring-1".to_string(),
        })
        .expect("public observation authoring snapshot");
        let field = snapshot
            .definitions
            .iter()
            .find(|row| row.name == "표시각도")
            .expect("public observation field definition");
        assert_eq!(field.kind, "public_observation_field");
        assert_eq!(&main[field.range.start..field.range.end], "표시각도");
        assert!(!field.id.contains("project.obs."));
        assert!(!field.id.contains("output_"));

        let hover = build_project_authoring_hover(ProjectAuthoringCursorRequest {
            schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "boim-authoring-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "main.ddn".to_string(),
            byte_offset: field.range.start,
        })
        .expect("public observation hover");
        assert_eq!(hover.status, "found");
        assert_eq!(hover.name.as_deref(), Some("표시각도"));
        assert_eq!(hover.kind.as_deref(), Some("public_observation_field"));
        assert_eq!(hover.value_label.as_deref(), Some("각도"));

        let references = build_project_authoring_navigation(
            ProjectAuthoringCursorRequest {
                schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
                source: source.clone(),
                source_version: "boim-authoring-1".to_string(),
                expected_project_source_sha256: snapshot.project_source_sha256.clone(),
                expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
                path: "main.ddn".to_string(),
                byte_offset: field.range.start,
            },
            "references",
        )
        .expect("public observation references");
        assert_eq!(references.status, "found");
        assert!(references.references.is_empty());

        let plan = build_project_authoring_rename_preview(ProjectAuthoringRenameRequest {
            schema: PROJECT_AUTHORING_RENAME_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "boim-authoring-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "main.ddn".to_string(),
            byte_offset: field.range.start,
            new_name: "방향".to_string(),
        })
        .expect("public observation rename");
        assert_eq!(plan.status, "ready");
        assert_eq!(plan.old_name.as_deref(), Some("표시각도"));
        assert_eq!(plan.affected_reference_count, 0);
        let edits = plan
            .files
            .iter()
            .flat_map(|file| &file.edits)
            .collect::<Vec<_>>();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].old_text, "표시각도");
        assert_eq!(edits[0].new_text, "방향");

        let conflict = build_project_authoring_rename_preview(ProjectAuthoringRenameRequest {
            schema: PROJECT_AUTHORING_RENAME_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "boim-authoring-1".to_string(),
            expected_project_source_sha256: snapshot.project_source_sha256.clone(),
            expected_snapshot_sha256: snapshot.snapshot_sha256.clone(),
            path: "main.ddn".to_string(),
            byte_offset: field.range.start,
            new_name: "표시시간".to_string(),
        })
        .expect("same observation conflict");
        assert_eq!(conflict.status, "conflict");
        assert!(conflict.files.is_empty());

        let stale = build_project_authoring_rename_preview(ProjectAuthoringRenameRequest {
            schema: PROJECT_AUTHORING_RENAME_REQUEST_SCHEMA.to_string(),
            source,
            source_version: "boim-authoring-1".to_string(),
            expected_project_source_sha256: "0".repeat(64),
            expected_snapshot_sha256: snapshot.snapshot_sha256,
            path: "main.ddn".to_string(),
            byte_offset: field.range.start,
            new_name: "방향".to_string(),
        })
        .expect_err("stale rename must fail closed");
        assert_eq!(stale.code, "E_PROJECT_AUTHORING_STALE");
    }

    #[test]
    fn multifile_public_observation_value_resolves_imported_alias_definition() {
        let main = r#"쓰임 {
  운동: "./lib/운동".
}

채비 {
  시간 <- 0.
  각도 <- 운동.시작각.
}.

(매마디)마다 {
  시간 <- 시간 + 1.
  각도 <- 각도 + 운동.증가량.
  (경과: 시간 * 1@s, 각도: 각도 * 1@rad, 증가량: 운동.증가량) 보임.
}.
"#;
        let source = project(
            vec![
                ("뿌리.ddn", "root", &root("app/main.ddn")),
                ("app/main.ddn", "ddn_source", main),
                (
                    "lib/운동.ddn",
                    "ddn_source",
                    "드러냄 {\n  시작각.\n  증가량.\n}\n\n시작각 <- 15.\n증가량 <- 2.\n",
                ),
            ],
            "app/main.ddn",
        );
        let snapshot = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: source.clone(),
            source_version: "boim-multifile-authoring-1".to_string(),
        })
        .expect("multifile public observation authoring snapshot");
        let occurrence_start = main.rfind("운동.증가량").expect("qualified value");
        let reference = snapshot
            .references
            .iter()
            .find(|row| {
                row.path == "app/main.ddn"
                    && row.name == "운동.증가량"
                    && row.range.start == occurrence_start
            })
            .expect("public observation imported reference");
        assert_eq!(
            reference.resolution, "imported_qualified",
            "imports={:?} exports={:?}",
            snapshot.imports, snapshot.exports
        );

        let definition = build_project_authoring_navigation(
            ProjectAuthoringCursorRequest {
                schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
                source,
                source_version: "boim-multifile-authoring-1".to_string(),
                expected_project_source_sha256: snapshot.project_source_sha256,
                expected_snapshot_sha256: snapshot.snapshot_sha256,
                path: "app/main.ddn".to_string(),
                byte_offset: occurrence_start + "운동".len(),
            },
            "definition",
        )
        .expect("definition");
        assert_eq!(definition.status, "found");
        assert_eq!(
            definition.declaration.expect("imported declaration").path,
            "lib/운동.ddn"
        );
    }

    #[test]
    fn w3_correction_valid_single_direct_and_alias_models_have_zero_diagnostics() {
        let single = authoring_snapshot(
            project(
                vec![
                    ("뿌리.ddn", "root", &root("lesson.ddn")),
                    ("lesson.ddn", "ddn_source", model_source()),
                ],
                "lesson.ddn",
            ),
            "w3-r1-valid-single",
        );
        assert!(single.diagnostics.is_empty(), "{:?}", single.diagnostics);

        for (case, alias) in [("direct", "방법"), ("alias", "비율")] {
            let entry = multifile_model_entry(
                &format!("쓰임 {{\n  {alias}: \"./lib/method\".\n}}\n"),
                &format!("{alias}.정밀한방법"),
                "",
            );
            let snapshot = authoring_snapshot(
                multifile_model_project(&entry, exact_method_module()),
                &format!("w3-r1-valid-{case}"),
            );
            assert!(
                snapshot.diagnostics.is_empty(),
                "{case}: {:?}",
                snapshot.diagnostics
            );
            let reference = snapshot
                .references
                .iter()
                .find(|reference| reference.name == format!("{alias}.정밀한방법"))
                .unwrap_or_else(|| panic!("{case}: imported recipe reference missing"));
            assert_eq!(reference.path, "app/main.ddn");
            assert_eq!(reference.resolution, "imported_qualified");
            assert!(reference.definition_id.is_some());
            assert!(reference.import_id.is_some());
        }
    }

    #[test]
    fn w3_correction_alias_whitespace_preserves_meaning_and_original_span() {
        let compact_entry = multifile_model_entry(
            "쓰임 {\n  비율: \"./lib/method\".\n}\n",
            "비율.정밀한방법",
            "",
        );
        let spaced_entry = multifile_model_entry(
            "쓰임 {\n  비율: \"./lib/method\".\n}\n",
            "비율 \n    . \n    정밀한방법",
            "",
        );
        let compact = authoring_snapshot(
            multifile_model_project(&compact_entry, exact_method_module()),
            "w3-r1-alias-compact",
        );
        let spaced = authoring_snapshot(
            multifile_model_project(&spaced_entry, exact_method_module()),
            "w3-r1-alias-spaced",
        );
        assert!(compact.diagnostics.is_empty());
        assert!(spaced.diagnostics.is_empty(), "{:?}", spaced.diagnostics);
        let compact_ref = compact
            .references
            .iter()
            .find(|reference| reference.name == "비율.정밀한방법")
            .expect("compact imported reference");
        let spaced_ref = spaced
            .references
            .iter()
            .find(|reference| reference.name == "비율.정밀한방법")
            .expect("spaced imported reference");
        assert_eq!(spaced_ref.resolution, "imported_qualified");
        assert_eq!(spaced_ref.definition_id, compact_ref.definition_id);
        assert_eq!(
            &spaced_entry[spaced_ref.range.start..spaced_ref.range.end],
            "비율 \n    . \n    정밀한방법"
        );
    }

    #[test]
    fn w3_correction_real_top_level_violation_remains_file_qualified() {
        let entry = multifile_model_entry(
            "쓰임 {\n  비율: \"./lib/method\".\n}\n",
            "비율.정밀한방법",
            "실제위반.\n",
        );
        let snapshot = authoring_snapshot(
            multifile_model_project(&entry, exact_method_module()),
            "w3-r1-real-top-level-error",
        );
        let error = diagnostic(&snapshot, "E_MODEL_TOP_LEVEL_CANDIDATE");
        assert_eq!(error.source_ref.as_deref(), Some("app/main.ddn"));
        let range = error.source_range.as_ref().expect("original source range");
        assert_eq!(&entry[range.start..range.end], "실제위반");
    }

    #[test]
    fn w3_correction_import_failures_keep_exact_codes_and_source_locations() {
        let missing_entry = multifile_model_entry(
            "쓰임 {\n  비율: \"./lib/missing\".\n}\n",
            "비율.정밀한방법",
            "",
        );
        let missing = authoring_snapshot(
            multifile_model_project(&missing_entry, exact_method_module()),
            "w3-r1-import-missing",
        );
        let missing_error = diagnostic(&missing, "E_MODULE_MISSING");
        assert_eq!(missing_error.source_ref.as_deref(), Some("app/main.ddn"));
        let missing_range = missing_error
            .source_range
            .as_ref()
            .expect("missing import source range");
        assert_eq!(
            &missing_entry[missing_range.start..missing_range.end],
            "비율"
        );

        let duplicate_entry = multifile_model_entry(
            "쓰임 {\n  비율: \"./lib/method\".\n  비율: \"./lib/method\".\n}\n",
            "비율.정밀한방법",
            "",
        );
        let duplicate = authoring_snapshot(
            multifile_model_project(&duplicate_entry, exact_method_module()),
            "w3-r1-import-duplicate",
        );
        let duplicate_error = diagnostic(&duplicate, "E_IMPORT_ALIAS_DUPLICATE");
        assert_eq!(duplicate_error.source_ref.as_deref(), Some("app/main.ddn"));
        let duplicate_range = duplicate_error
            .source_range
            .as_ref()
            .expect("duplicate alias source range");
        assert_eq!(
            &duplicate_entry[duplicate_range.start..duplicate_range.end],
            "비율"
        );

        let missing_symbol_entry = multifile_model_entry(
            "쓰임 {\n  비율: \"./lib/method\".\n}\n",
            "비율.없는방법",
            "",
        );
        let missing_symbol = authoring_snapshot(
            multifile_model_project(&missing_symbol_entry, exact_method_module()),
            "w3-r1-symbol-missing",
        );
        let symbol_error = diagnostic(&missing_symbol, "E_SYMBOL_NOT_EXPORTED");
        assert_eq!(symbol_error.source_ref.as_deref(), Some("app/main.ddn"));
        let symbol_range = symbol_error
            .source_range
            .as_ref()
            .expect("missing symbol source range");
        assert_eq!(
            &missing_symbol_entry[symbol_range.start..symbol_range.end],
            "비율.없는방법"
        );
    }

    #[test]
    fn w3_correction_stale_bad_entry_unrelated_error_and_failed_snapshot_fail_closed() {
        let entry = multifile_model_entry(
            "쓰임 {\n  비율: \"./lib/method\".\n}\n",
            "비율.정밀한방법",
            "",
        );
        let clean_source = multifile_model_project(&entry, exact_method_module());
        let clean = authoring_snapshot(clean_source.clone(), "w3-r1-clean-before-failure");
        assert!(clean.diagnostics.is_empty());

        let reference = clean
            .references
            .iter()
            .find(|reference| reference.name == "비율.정밀한방법")
            .expect("qualified reference");
        let stale = build_project_authoring_navigation(
            ProjectAuthoringCursorRequest {
                schema: PROJECT_AUTHORING_CURSOR_REQUEST_SCHEMA.to_string(),
                source: clean_source.clone(),
                source_version: "w3-r1-clean-before-failure".to_string(),
                expected_project_source_sha256: "0".repeat(64),
                expected_snapshot_sha256: clean.snapshot_sha256.clone(),
                path: "app/main.ddn".to_string(),
                byte_offset: reference.range.start,
            },
            "definition",
        )
        .expect_err("stale authoring identity must fail closed");
        assert_eq!(stale.code, "E_PROJECT_AUTHORING_STALE");

        let mut bad_entry = clean_source.clone();
        bad_entry.entry_path = "app/missing.ddn".to_string();
        let bad_entry_error = build_project_authoring_snapshot(ProjectAuthoringSnapshotRequest {
            schema: PROJECT_AUTHORING_SNAPSHOT_REQUEST_SCHEMA.to_string(),
            source: bad_entry,
            source_version: "w3-r1-bad-entry".to_string(),
        })
        .expect_err("missing entry must fail closed");
        assert_eq!(bad_entry_error.code, "E_PROJECT_ENTRY_MISSING");
        assert_eq!(bad_entry_error.path.as_deref(), Some("app/missing.ddn"));

        let mut unrelated = clean_source.clone();
        unrelated
            .files
            .push(local_project::VirtualProjectFileSource {
                path: "unused/bad.ddn".to_string(),
                role: "ddn_source".to_string(),
                utf8_text: "실제위반.\n나쁜모형 = 모형 {}.\n".to_string(),
            });
        let unrelated = authoring_snapshot(unrelated, "w3-r1-unrelated-error");
        let unrelated_error = diagnostic(&unrelated, "E_MODEL_TOP_LEVEL_CANDIDATE");
        assert_eq!(
            unrelated_error.source_ref.as_deref(),
            Some("unused/bad.ddn")
        );
        let unrelated_range = unrelated_error
            .source_range
            .as_ref()
            .expect("unrelated source range");
        assert_eq!(unrelated_range.start, 0);

        let failed_entry = multifile_model_entry(
            "쓰임 {\n  비율: \"./lib/method\".\n}\n",
            "비율.정밀한방법",
            "실제위반.\n",
        );
        let failed = authoring_snapshot(
            multifile_model_project(&failed_entry, exact_method_module()),
            "w3-r1-failed-after-clean",
        );
        assert_ne!(failed.snapshot_sha256, clean.snapshot_sha256);
        assert!(failed
            .definitions
            .iter()
            .all(|definition| definition.name != "다중파일비례"));
        assert!(diagnostic(&failed, "E_MODEL_TOP_LEVEL_CANDIDATE")
            .source_range
            .is_some());
    }
}
