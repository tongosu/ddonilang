use crate::lsp::{
    byte_range_to_lsp_range, ByteRange, DocumentSnapshot, LspRange, PositionEncoding,
};
use ddonirang_lang::{parse_v25, validate_no_legacy_boim_surface};
use std::fmt;

pub const SOURCE_PROFILE_V25: &str = "v25";
pub const DIAGNOSTIC_SEVERITY_ERROR: u8 = 1;

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
    let mut diagnostics = Vec::new();

    if snapshot.language_id == "ddn" {
        if let Err(message) = validate_no_legacy_boim_surface(source) {
            let start = source.find("보임").unwrap_or(0);
            let range = diagnostic_range(source, start, start + "보임".len(), encoding)?;
            diagnostics.push(Diagnostic {
                code: message
                    .split(':')
                    .next()
                    .unwrap_or("E_BOIM_LEGACY_REMOVED")
                    .to_string(),
                severity: DIAGNOSTIC_SEVERITY_ERROR,
                message,
                range,
            });
        } else if let Err(error) = parse_v25(source, &snapshot.uri) {
            let range = diagnostic_range(source, error.span.start, error.span.end, encoding)?;
            diagnostics.push(Diagnostic {
                code: error.code().to_string(),
                severity: DIAGNOSTIC_SEVERITY_ERROR,
                message: error.message,
                range,
            });
        }
    }

    sort_diagnostics(&mut diagnostics);
    Ok(DiagnosticReport {
        document_version: snapshot.version,
        source_profile: SOURCE_PROFILE_V25,
        diagnostics,
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
