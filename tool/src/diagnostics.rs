use crate::lsp::{
    byte_range_to_lsp_range, ByteRange, DocumentSnapshot, LspRange, PositionEncoding,
};
use crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY;
use crate::source_analysis::{analyze_source, SourceAnalysisRequest};
use std::fmt;

pub const SOURCE_PROFILE_V25: &str = "v25";
/// The editor checks open DDN text with the same shared analysis and profile
/// as `ddonilang 검사 <파일> --source-profile v1-core-v25`. It reports what that
/// analysis reports; an empty list is not a claim of further verification.
pub const ANALYSIS_PROFILE: &str = V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY;
pub const DIAGNOSTIC_SEVERITY_ERROR: u8 = 1;
pub const DIAGNOSTIC_SEVERITY_WARNING: u8 = 2;
pub const DIAGNOSTIC_SEVERITY_INFORMATION: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: u8,
    pub message: String,
    pub range: LspRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticReport {
    pub document_version: i64,
    pub source_profile: &'static str,
    pub diagnostics: Vec<Diagnostic>,
    /// Shared analysis outcome (`ACCEPTED`/`REJECTED`/`UNSUPPORTED`), or
    /// `NOT_ANALYZED` for an opened surface the editor does not check.
    pub outcome: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticError {
    InvalidUtf8,
    InvalidRange(String),
}

impl fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 => f.write_str("문서 bytes가 UTF-8이 아닙니다"),
            Self::InvalidRange(error) => write!(f, "diagnostic range가 잘못되었습니다: {error}"),
        }
    }
}

impl std::error::Error for DiagnosticError {}

pub fn analyze_snapshot(
    snapshot: &DocumentSnapshot,
    encoding: PositionEncoding,
) -> Result<DiagnosticReport, DiagnosticError> {
    let source = snapshot.text().map_err(|_| DiagnosticError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(DiagnosticReport {
            document_version: snapshot.version,
            source_profile: SOURCE_PROFILE_V25,
            diagnostics: Vec::new(),
            outcome: "NOT_ANALYZED",
        });
    }
    let request = SourceAnalysisRequest {
        source_text: source.to_string(),
        path: snapshot.uri.clone(),
        analysis_generation: u64::try_from(snapshot.version).unwrap_or(0),
        source_profile_identity: ANALYSIS_PROFILE.to_string(),
    };
    // A failure inside the shared analysis is reported on this exact version;
    // the previous version's diagnostics must not stay on screen as current.
    let Ok(analysis) = std::panic::catch_unwind(|| analyze_source(request)) else {
        return Ok(DiagnosticReport {
            document_version: snapshot.version,
            source_profile: SOURCE_PROFILE_V25,
            diagnostics: vec![Diagnostic {
                code: "E_LSP_ANALYSIS_FAILED".to_string(),
                severity: DIAGNOSTIC_SEVERITY_ERROR,
                message: "공통 분석기가 이 판을 분석하지 못했습니다. 오류가 없다는 뜻이 아닙니다."
                    .to_string(),
                range: diagnostic_range(source, 0, 0, encoding)?,
            }],
            outcome: "ANALYSIS_FAILED",
        });
    };
    let mut diagnostics = Vec::with_capacity(analysis.diagnostics.len());
    for diagnostic in analysis.diagnostics {
        let range = diagnostic_range(source, diagnostic.span.start, diagnostic.span.end, encoding)?;
        diagnostics.push(Diagnostic {
            code: diagnostic.code,
            severity: match diagnostic.severity {
                "error" => DIAGNOSTIC_SEVERITY_ERROR,
                "warning" => DIAGNOSTIC_SEVERITY_WARNING,
                _ => DIAGNOSTIC_SEVERITY_INFORMATION,
            },
            message: diagnostic.message,
            range,
        });
    }

    sort_diagnostics(&mut diagnostics);
    Ok(DiagnosticReport {
        document_version: snapshot.version,
        source_profile: SOURCE_PROFILE_V25,
        diagnostics,
        outcome: analysis.outcome,
    })
}

fn diagnostic_range(
    source: &str,
    start: usize,
    end: usize,
    encoding: PositionEncoding,
) -> Result<LspRange, DiagnosticError> {
    let start = previous_char_boundary(source, start.min(source.len()));
    let end = next_char_boundary(source, end.min(source.len())).max(start);
    byte_range_to_lsp_range(source, ByteRange { start, end }, encoding)
        .map_err(|error| DiagnosticError::InvalidRange(error.to_string()))
}

fn previous_char_boundary(source: &str, mut offset: usize) -> usize {
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn next_char_boundary(source: &str, mut offset: usize) -> usize {
    while offset < source.len() && !source.is_char_boundary(offset) {
        offset += 1;
    }
    offset
}

pub fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|left, right| {
        range_key(&left.range)
            .cmp(&range_key(&right.range))
            .then_with(|| left.code.cmp(&right.code))
    });
}

fn range_key(range: &LspRange) -> (u32, u32, u32, u32) {
    (
        range.start.line,
        range.start.character,
        range.end.line,
        range.end.character,
    )
}
