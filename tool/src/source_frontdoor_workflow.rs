use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::artifact_output::write_text_artifact_atomic;
use crate::lint::{analyze_lint_source, write_lint_patch_atomic};
use crate::preprocess::preprocess_source_for_parse;
use crate::runtime_surface::canonicalize_shared_source_with_supported_profile;
use crate::source_analysis::{analyze_source, SourceAnalysisRequest};
use ddonirang_lang::{Body, Expr, ExprKind, Literal, ParseMode, Stmt, TopLevelItem};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFrontdoorError {
    message: String,
    exit_code: i32,
}

impl SourceFrontdoorError {
    pub fn runtime(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 1,
        }
    }

    pub fn input(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 2,
        }
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SourceFrontdoorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SourceFrontdoorError {}

impl std::ops::Deref for SourceFrontdoorError {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintWorkflowOutput {
    pub stdout: String,
    pub warnings: Vec<String>,
    pub patch_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacyInternalCheckArgs {
    pub emit_schema: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckTypeKind {
    Str,
    Num,
    Bool,
    None,
    Unknown,
}

impl CheckTypeKind {
    fn name(self) -> Option<&'static str> {
        match self {
            Self::Str => Some("글"),
            Self::Num => Some("수"),
            Self::Bool => Some("참거짓"),
            Self::None => Some("없음"),
            Self::Unknown => None,
        }
    }

    fn is_known(self) -> bool {
        self != Self::Unknown
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckSchemaEntry {
    name: String,
    type_name: String,
}

pub fn run_check_or_build_supported_profile(
    file: &Path,
    source_profile: &str,
) -> Result<(String, i32), SourceFrontdoorError> {
    let source = read_source(file, "E_CLI_SOURCE_READ", "E_CLI_SOURCE_UTF8", false)?;
    let snapshot = analyze_source(SourceAnalysisRequest {
        source_text: source,
        path: file.display().to_string(),
        analysis_generation: 0,
        source_profile_identity: source_profile.to_string(),
    });
    let exit_code = if snapshot.outcome == "ACCEPTED" { 0 } else { 1 };
    let output = serde_json::to_string_pretty(&snapshot)
        .map(|text| format!("{text}\n"))
        .map_err(|error| SourceFrontdoorError::runtime(format!("E_CLI_JSON_SERIALIZE {error}")))?;
    Ok((output, exit_code))
}

/// Historical/internal check and build custody routed through the shared
/// language parser and shared artifact writer. This is not a public profile or
/// a compatibility alias; it exists only so the two shared commands do not
/// retain a private teul parser/type/schema fallback.
pub fn run_check_or_build_legacy_internal_custody(
    file: &Path,
    args: LegacyInternalCheckArgs,
) -> Result<(), String> {
    let source = fs::read_to_string(file).map_err(|error| error.to_string())?;
    let prepared = preprocess_source_for_parse(&source)?;
    let prepared = ddonirang_lang::preprocess_frontdoor_source(&prepared);
    let program =
        ddonirang_lang::parse_with_mode(&prepared, &file.display().to_string(), ParseMode::Strict)
            .map_err(|error| {
                format!(
                    "{}: {} ({}#bytes={}:{})",
                    error.code(),
                    error.message,
                    file.display(),
                    error.span.start,
                    error.span.end
                )
            })?;

    let mut symbols = BTreeMap::<String, CheckTypeKind>::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            collect_check_symbols(body, &mut symbols)?;
        }
    }

    if args.emit_schema {
        let entries = symbols
            .iter()
            .filter_map(|(name, kind)| {
                kind.name().map(|type_name| CheckSchemaEntry {
                    name: name.clone(),
                    type_name: type_name.to_string(),
                })
            })
            .collect::<Vec<_>>();
        write_check_schema(file, &entries)?;
    }
    Ok(())
}

fn collect_check_symbols(
    body: &Body,
    symbols: &mut BTreeMap<String, CheckTypeKind>,
) -> Result<(), String> {
    for stmt in &body.stmts {
        let Stmt::Mutate { target, value, .. } = stmt else {
            continue;
        };
        let Some(name) = check_target_name(target) else {
            continue;
        };
        let value_type = check_expr_type(value, symbols);
        if let Some(existing) = symbols.get(&name).copied() {
            if existing.is_known() && value_type.is_known() && existing != value_type {
                return Err(format!(
                    "E_CHECK_TYPE_MISMATCH {} {} -> {}",
                    name,
                    existing.name().unwrap_or("알수없음"),
                    value_type.name().unwrap_or("알수없음")
                ));
            }
            return Err(format!("E_CHECK_DUPLICATE_SYMBOL {name}"));
        }
        symbols.insert(name, value_type);
    }
    Ok(())
}

fn check_target_name(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Var(name) => Some(name.clone()),
        ExprKind::FieldAccess { target, field, .. } => {
            check_target_name(target).map(|prefix| format!("{prefix}.{field}"))
        }
        _ => None,
    }
}

fn check_expr_type(expr: &Expr, symbols: &BTreeMap<String, CheckTypeKind>) -> CheckTypeKind {
    match &expr.kind {
        ExprKind::Literal(literal) => match literal {
            Literal::String(_) | Literal::Atom(_) => CheckTypeKind::Str,
            Literal::Int(_) | Literal::Fixed64(_) => CheckTypeKind::Num,
            Literal::Bool(_) => CheckTypeKind::Bool,
            Literal::None => CheckTypeKind::None,
            Literal::Regex(_) | Literal::Resource(_) => CheckTypeKind::Unknown,
        },
        ExprKind::Var(name) => symbols.get(name).copied().unwrap_or(CheckTypeKind::Unknown),
        ExprKind::Infix { op, .. }
            if matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") =>
        {
            CheckTypeKind::Bool
        }
        ExprKind::Infix { .. } | ExprKind::Call { .. } | ExprKind::FormulaEval { .. } => {
            CheckTypeKind::Num
        }
        ExprKind::TemplateRender { .. } => CheckTypeKind::Str,
        ExprKind::Nuance { expr, .. } | ExprKind::Suffix { value: expr, .. } => {
            check_expr_type(expr, symbols)
        }
        _ => CheckTypeKind::Unknown,
    }
}

fn write_check_schema(source_path: &Path, entries: &[CheckSchemaEntry]) -> Result<(), String> {
    let mut output = String::from("{\n  \"version\": 0,\n  \"symbols\": [\n");
    for (index, entry) in entries.iter().enumerate() {
        output.push_str("    { \"name\": \"");
        output.push_str(&crate::state_trace_wire::escape_json_string_contents(
            &entry.name,
        ));
        output.push_str("\", \"type\": \"");
        output.push_str(&crate::state_trace_wire::escape_json_string_contents(
            &entry.type_name,
        ));
        output.push_str("\" }");
        if index + 1 < entries.len() {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("  ]\n}\n");
    write_text_artifact_atomic(&check_schema_path(source_path), &output)
}

pub fn check_schema_path(source_path: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    source_path.to_string_lossy().hash(&mut hasher);
    let digest = hasher.finish();
    let stem = source_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("ddn");
    let safe_stem = stem
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    crate::paths::build_dir()
        .join("check_schema")
        .join(format!("{safe_stem}.{digest:016x}.ddn.schema.json"))
}

pub fn run_canon_supported_profile(
    file: &Path,
    source_profile: &str,
    out: Option<&Path>,
    check: bool,
) -> Result<String, SourceFrontdoorError> {
    let source = read_source(file, "E_CLI_SOURCE_READ", "E_CLI_SOURCE_UTF8", false)?;
    let canonical =
        canonicalize_shared_source_with_supported_profile(&source, Some(source_profile))
            .map_err(SourceFrontdoorError::runtime)?;
    let mut ddn = canonical.ddn;
    if !ddn.ends_with('\n') {
        ddn.push('\n');
    }
    if check && !canon_matches(&source, &ddn) {
        return Err(SourceFrontdoorError::runtime(format!(
            "E_CANON_CHECK_MISMATCH 정본 불일치: {}",
            file.display()
        )));
    }
    if let Some(path) = out {
        write_text_artifact_atomic(path, &ddn)
            .map_err(|error| SourceFrontdoorError::runtime(format!("E_CLI_WRITE {error}")))?;
        Ok(String::new())
    } else {
        Ok(ddn)
    }
}

pub fn run_lint_supported_profile(
    file: &Path,
    source_profile: &str,
    suggest_patch: bool,
    out: Option<&Path>,
) -> Result<LintWorkflowOutput, SourceFrontdoorError> {
    let source = read_source(file, "E_LINT_READ", "E_LINT_READ_UTF8", true)?;
    let analysis = analyze_lint_source(&source, &file.to_string_lossy(), source_profile)
        .map_err(SourceFrontdoorError::input)?;
    let patch_path = suggest_patch.then(|| {
        out.map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("ddn.patch.json"))
    });
    let stdout = if let Some(path) = patch_path.as_ref() {
        write_lint_patch_atomic(path, &analysis.patch_json)
            .map_err(SourceFrontdoorError::runtime)?;
        format!("patch_written={}\n", path.display())
    } else {
        String::new()
    };
    Ok(LintWorkflowOutput {
        stdout,
        warnings: analysis.warnings,
        patch_path,
    })
}

fn read_source(
    file: &Path,
    read_code: &str,
    utf8_code: &str,
    input_error: bool,
) -> Result<String, SourceFrontdoorError> {
    let build_error = |message: String| {
        if input_error {
            SourceFrontdoorError::input(message)
        } else {
            SourceFrontdoorError::runtime(message)
        }
    };
    let bytes = fs::read(file)
        .map_err(|error| build_error(format!("{read_code} {} {error}", file.display())))?;
    String::from_utf8(bytes)
        .map_err(|error| build_error(format!("{utf8_code} {} {error}", file.display())))
}

fn canon_matches(source: &str, canonical: &str) -> bool {
    canonical.trim_end() == source.trim_end()
}

#[cfg(test)]
mod tests {
    use super::{
        check_schema_path, run_check_or_build_legacy_internal_custody, LegacyInternalCheckArgs,
    };
    use std::fs;
    use std::path::Path;

    struct TemporarySource(std::path::PathBuf);

    impl Drop for TemporarySource {
        fn drop(&mut self) {
            if let Some(directory) = self.0.parent() {
                let _ = fs::remove_dir_all(directory);
            }
        }
    }

    fn temporary_source(name: &str, source: &str) -> TemporarySource {
        let directory =
            std::env::temp_dir().join(format!("ddn-shared-check-{}-{}", std::process::id(), name));
        fs::create_dir_all(&directory).expect("create test directory");
        let path = directory.join("main.ddn");
        fs::write(&path, source).expect("write test source");
        TemporarySource(path)
    }

    #[test]
    fn legacy_internal_check_uses_shared_parser_and_rejects_duplicates() {
        let path = temporary_source("duplicate", "나이 <- 10.\n나이 <- 11.\n");
        let error = run_check_or_build_legacy_internal_custody(
            &path.0,
            LegacyInternalCheckArgs { emit_schema: false },
        )
        .expect_err("duplicate must fail closed");
        assert!(error.contains("E_CHECK_DUPLICATE_SYMBOL 나이"));
    }

    #[test]
    fn legacy_internal_check_rejects_type_flip() {
        let path = temporary_source("type-flip", "x <- 1.\nx <- \"글\".\n");
        let error = run_check_or_build_legacy_internal_custody(
            &path.0,
            LegacyInternalCheckArgs { emit_schema: false },
        )
        .expect_err("type flip must fail closed");
        assert!(error.contains("E_CHECK_TYPE_MISMATCH x 수 -> 글"));
    }

    #[test]
    fn schema_path_is_redirected_to_build_cache_zone() {
        let source = Path::new("solutions/seamgrim_ui_mvp/lessons/foo/lesson.ddn");
        let output = check_schema_path(source);
        let output = output.to_string_lossy().replace('\\', "/");
        assert!(output.contains("/check_schema/"));
        assert!(output.ends_with(".ddn.schema.json"));
        assert!(!output.contains("/lessons/foo/ddn.schema.json"));
    }
}
