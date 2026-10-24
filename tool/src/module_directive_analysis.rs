use ddonirang_lang::{
    parse_module_directive_frontdoor_with_mode, ModuleDirectiveContext, ModuleDirectiveEntry,
    ModuleDirectiveFrontdoorResult, ParseMode,
};
use serde::{Deserialize, Serialize};

use crate::ddn_runtime::resolve_supported_source_profile;
use crate::source_analysis::SourceAnalysisSpan;

/// Additive internal product carrier.  The established source-analysis v1
/// request and response remain byte-for-byte unchanged.
pub const MODULE_DIRECTIVE_ANALYSIS_SCHEMA: &str =
    "ddn.module_directive_analysis.candidate.v1";

#[derive(Clone, Debug, Deserialize)]
pub struct ModuleDirectiveAnalysisRequest {
    pub schema: String,
    pub source_text: String,
    #[serde(default = "default_path")]
    pub path: String,
    pub analysis_generation: u64,
    pub source_profile_identity: String,
    pub module_registry: ModuleDirectiveRegistryInput,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModuleDirectiveRegistryInput {
    pub known_import_targets: Vec<String>,
    pub public_definition_targets: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModuleDirectiveAnalysisEntry {
    pub local_name: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModuleDirectiveAnalysisMetadata {
    pub imports: Vec<ModuleDirectiveAnalysisEntry>,
    pub exports: Vec<ModuleDirectiveAnalysisEntry>,
    pub import_span: Option<SourceAnalysisSpan>,
    pub export_span: Option<SourceAnalysisSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModuleDirectiveAnalysisDiagnostic {
    pub code: String,
    pub severity: &'static str,
    pub message: String,
    pub path: String,
    pub span: SourceAnalysisSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModuleDirectiveAnalysisSnapshot {
    pub schema: &'static str,
    pub source_revision_identity: String,
    pub analysis_generation: u64,
    pub source_profile_identity: String,
    pub outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ModuleDirectiveAnalysisMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_source: Option<String>,
    pub diagnostics: Vec<ModuleDirectiveAnalysisDiagnostic>,
}

fn default_path() -> String {
    "lesson.ddn".to_string()
}

fn source_revision_identity(source: &str) -> String {
    format!("blake3:{}", blake3::hash(source.as_bytes()).to_hex())
}

fn diagnostic_code(message: &str) -> String {
    let Some(start) = message.find("E_") else {
        return "E_MODULE_DIRECTIVE_PARSE".to_string();
    };
    let tail = &message[start..];
    let end = tail
        .char_indices()
        .find_map(|(index, ch)| {
            if index > 0 && !(ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_') {
                Some(index)
            } else {
                None
            }
        })
        .unwrap_or(tail.len());
    tail[..end].to_string()
}

fn error_snapshot(
    request: &ModuleDirectiveAnalysisRequest,
    revision: String,
    code: impl Into<String>,
    message: impl Into<String>,
    start: usize,
    end: usize,
    outcome: &'static str,
) -> ModuleDirectiveAnalysisSnapshot {
    ModuleDirectiveAnalysisSnapshot {
        schema: MODULE_DIRECTIVE_ANALYSIS_SCHEMA,
        source_revision_identity: revision,
        analysis_generation: request.analysis_generation,
        source_profile_identity: request.source_profile_identity.clone(),
        outcome,
        metadata: None,
        canonical_source: None,
        diagnostics: vec![ModuleDirectiveAnalysisDiagnostic {
            code: code.into(),
            severity: "error",
            message: message.into(),
            path: request.path.clone(),
            span: SourceAnalysisSpan {
                encoding: "utf-8-byte",
                start,
                end,
            },
        }],
    }
}

fn entries(values: &[ModuleDirectiveEntry]) -> Vec<ModuleDirectiveAnalysisEntry> {
    values
        .iter()
        .map(|entry| ModuleDirectiveAnalysisEntry {
            local_name: entry.local_name.clone(),
            target: entry.target.clone(),
        })
        .collect()
}

fn metadata(result: &ModuleDirectiveFrontdoorResult) -> ModuleDirectiveAnalysisMetadata {
    let imports = result.directives.imports.as_ref();
    let exports = result.directives.exports.as_ref();
    ModuleDirectiveAnalysisMetadata {
        imports: imports.map(|directive| entries(&directive.entries)).unwrap_or_default(),
        exports: exports.map(|directive| entries(&directive.entries)).unwrap_or_default(),
        import_span: imports.map(|directive| SourceAnalysisSpan {
            encoding: "utf-8-byte",
            start: directive.span.start,
            end: directive.span.end,
        }),
        export_span: exports.map(|directive| SourceAnalysisSpan {
            encoding: "utf-8-byte",
            start: directive.span.start,
            end: directive.span.end,
        }),
    }
}

pub fn analyze_module_directive_source(
    request: ModuleDirectiveAnalysisRequest,
) -> ModuleDirectiveAnalysisSnapshot {
    let revision = source_revision_identity(&request.source_text);
    if request.schema != MODULE_DIRECTIVE_ANALYSIS_SCHEMA {
        return error_snapshot(
            &request,
            revision,
            "E_MODULE_DIRECTIVE_ANALYSIS_SCHEMA_UNSUPPORTED",
            "module directive analysis schema가 지원되지 않습니다",
            0,
            0,
            "UNSUPPORTED",
        );
    }
    if request.source_profile_identity.is_empty() {
        return error_snapshot(
            &request,
            revision,
            "E_SOURCE_PROFILE_REQUIRED",
            "source profile identity가 필요합니다",
            0,
            0,
            "UNSUPPORTED",
        );
    }
    if let Err(error) = resolve_supported_source_profile(Some(&request.source_profile_identity)) {
        return error_snapshot(
            &request,
            revision,
            error.code,
            error.message,
            0,
            0,
            "UNSUPPORTED",
        );
    }
    let context = ModuleDirectiveContext::default()
        .with_known_imports(request.module_registry.known_import_targets.iter().cloned())
        .with_public_definitions(request.module_registry.public_definition_targets.iter().cloned());
    match parse_module_directive_frontdoor_with_mode(
        &request.source_text,
        &request.path,
        ParseMode::Strict,
        &context,
    ) {
        Ok(result) => ModuleDirectiveAnalysisSnapshot {
            schema: MODULE_DIRECTIVE_ANALYSIS_SCHEMA,
            source_revision_identity: revision,
            analysis_generation: request.analysis_generation,
            source_profile_identity: request.source_profile_identity,
            outcome: "ACCEPTED",
            metadata: Some(metadata(&result)),
            canonical_source: Some(result.canonical_source),
            diagnostics: Vec::new(),
        },
        Err(error) => error_snapshot(
            &request,
            revision,
            diagnostic_code(&error.message),
            error.message,
            error.span.start,
            error.span.end,
            "REJECTED",
        ),
    }
}

pub fn module_directive_analysis_snapshot_json(request_json: &str) -> Result<String, String> {
    let request: ModuleDirectiveAnalysisRequest = serde_json::from_str(request_json)
        .map_err(|error| format!("E_MODULE_DIRECTIVE_ANALYSIS_REQUEST_INVALID {error}"))?;
    serde_json::to_string(&analyze_module_directive_source(request))
        .map_err(|error| format!("E_MODULE_DIRECTIVE_ANALYSIS_SERIALIZE_FAILED {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(source: &str) -> ModuleDirectiveAnalysisRequest {
        ModuleDirectiveAnalysisRequest {
            schema: MODULE_DIRECTIVE_ANALYSIS_SCHEMA.to_string(),
            source_text: source.to_string(),
            path: "운동😀.ddn".to_string(),
            analysis_generation: 17,
            source_profile_identity: "v1-core-v25".to_string(),
            module_registry: ModuleDirectiveRegistryInput {
                known_import_targets: vec!["./lib/😀".to_string()],
                public_definition_targets: vec!["중력".to_string()],
            },
        }
    }

    #[test]
    fn accepted_snapshot_preserves_metadata_canon_and_utf8_spans() {
        let source = "(운동=\"./lib/😀\") 쓰임.\n(속도=중력) 드러냄.\n계산:움직씨 := {\n  값:수 := 1.\n}\n";
        let snapshot = analyze_module_directive_source(request(source));
        assert_eq!(snapshot.outcome, "ACCEPTED");
        let metadata = snapshot.metadata.expect("metadata");
        assert_eq!(metadata.imports[0].target, "./lib/😀");
        assert_eq!(metadata.exports[0].target, "중력");
        let import_span = metadata.import_span.expect("import span");
        assert_eq!(&source[import_span.start..import_span.end], "(운동=\"./lib/😀\") 쓰임.");
        let export_span = metadata.export_span.expect("export span");
        assert_eq!(export_span.start, source.find("(속도=중력)").unwrap());
        assert!(snapshot.canonical_source.unwrap().contains("(운동=\"./lib/😀\") 쓰임."));
    }

    #[test]
    fn missing_registry_and_legacy_brace_are_fail_closed() {
        let mut missing = request("(운동=\"./lib/😀\") 쓰임.\n계산:움직씨 := { 값:수 := 1. }\n");
        missing.module_registry.known_import_targets.clear();
        let missing_snapshot = analyze_module_directive_source(missing);
        assert_eq!(missing_snapshot.outcome, "REJECTED");
        assert_eq!(missing_snapshot.diagnostics[0].code, "E_RD457_IMPORT_TARGET_UNKNOWN");

        let legacy = analyze_module_directive_source(request("쓰임 { 운동: \"./lib/😀\". }\n"));
        assert_eq!(legacy.outcome, "REJECTED");
        assert_eq!(legacy.diagnostics[0].code, "E_RD457_LEGACY_BRACE_DIRECTIVE");
        assert!(legacy.metadata.is_none());
        assert!(legacy.canonical_source.is_none());
    }

    #[test]
    fn workbench_physics_template_uses_the_same_typed_carrier_without_chaebi() {
        let source = "(힘=\"./lib/힘\") 쓰임.\n\n초기속도x:수 := 0.\n초기속도y:수 := 20.\n시간간격:수 := 0.1.\n중력가속도:수 := 힘.중력.\n속도:수 := 초기속도y - 중력가속도 * 시간간격.\n위치:수 := 초기속도x + 속도 * 시간간격.\n";
        assert!(!source.contains("채비 {"));
        let snapshot = analyze_module_directive_source(ModuleDirectiveAnalysisRequest {
            schema: MODULE_DIRECTIVE_ANALYSIS_SCHEMA.to_string(),
            source_text: source.to_string(),
            path: "app/main.ddn".to_string(),
            analysis_generation: 18,
            source_profile_identity: "v1-core-v25".to_string(),
            module_registry: ModuleDirectiveRegistryInput {
                known_import_targets: vec!["./lib/힘".to_string()],
                public_definition_targets: vec!["중력".to_string()],
            },
        });
        assert_eq!(snapshot.outcome, "ACCEPTED", "{:?}", snapshot.diagnostics);
        assert_eq!(snapshot.metadata.unwrap().imports[0].target, "./lib/힘");
    }
}
