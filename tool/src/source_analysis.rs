use ddonirang_lang::{Lexer, ParseMode, TokenKind};
use serde::{Deserialize, Serialize};

use crate::ddn_runtime::{
    resolve_supported_source_profile, DdnParseWarning, DdnProgram,
};

pub const SOURCE_ANALYSIS_SNAPSHOT_SCHEMA: &str = "ddn.source_analysis_snapshot.candidate.v1";
pub const STRICT_SOURCE_PROFILE_IDENTITY: &str = "strict";
pub const REPAIR_PROPOSAL_SCHEMA: &str = "ddn.source_analysis_repair_proposal.candidate.v1";
pub const REMOVE_SETUP_COLON_FIX_ID: &str = "FIX_SOURCE_REMOVE_SETUP_COLON";

#[derive(Clone, Debug, Deserialize)]
pub struct SourceAnalysisRequest {
    pub source_text: String,
    #[serde(default = "default_path")]
    pub path: String,
    pub analysis_generation: u64,
    #[serde(default = "default_profile")]
    pub source_profile_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisSpan {
    pub encoding: &'static str,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisOriginalSourceSpan {
    pub coordinate_space: &'static str,
    pub encoding: &'static str,
    pub half_open: bool,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisRepairProposal {
    pub schema: &'static str,
    pub fix_id: &'static str,
    pub source_revision_identity: String,
    pub analysis_generation: u64,
    pub diagnostic_identity: String,
    pub original_source_span: SourceAnalysisOriginalSourceSpan,
    pub expected_text: String,
    pub replacement_text: String,
    pub proposal_identity: String,
    pub explanation: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisDiagnostic {
    pub code: String,
    pub severity: &'static str,
    pub message: String,
    pub path: String,
    pub span: SourceAnalysisSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repair_proposal: Option<SourceAnalysisRepairProposal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisSnapshot {
    pub schema: &'static str,
    pub source_revision_identity: String,
    pub analysis_generation: u64,
    pub source_profile_identity: String,
    pub outcome: &'static str,
    pub diagnostics: Vec<SourceAnalysisDiagnostic>,
}

fn default_path() -> String {
    "lesson.ddn".to_string()
}

fn default_profile() -> String {
    String::new()
}

fn source_revision_identity(source: &str) -> String {
    format!("blake3:{}", blake3::hash(source.as_bytes()).to_hex())
}

fn deterministic_identity(domain: &str, parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&(part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    format!("{domain}:blake3:{}", hasher.finalize().to_hex())
}

fn raw_chaebi_header_colon_spans(source: &str) -> Option<Vec<(usize, usize)>> {
    let tokens = Lexer::new(source).tokenize().ok()?;
    let mut spans = Vec::new();
    for window in tokens.windows(3) {
        let is_chaebi = match &window[0].kind {
            TokenKind::Ident(name) | TokenKind::Josa(name) => name == "채비",
            _ => false,
        };
        if is_chaebi
            && matches!(window[1].kind, TokenKind::Colon)
            && matches!(window[2].kind, TokenKind::LBrace)
        {
            spans.push((window[1].span.start, window[1].span.end));
        }
    }
    Some(spans)
}

fn is_chaebi_header_warning(warning: &DdnParseWarning) -> bool {
    warning.code == "W_BLOCK_HEADER_COLON_DEPRECATED" && warning.message.contains("`채비:`")
}

fn authoritative_repair_proposal(
    source: &str,
    revision: &str,
    generation: u64,
    occurrence_index: usize,
    original_span: (usize, usize),
) -> Option<(String, SourceAnalysisRepairProposal)> {
    let (start, end) = original_span;
    if source.get(start..end)? != ":" {
        return None;
    }
    let generation_bytes = generation.to_be_bytes();
    let occurrence_bytes = (occurrence_index as u64).to_be_bytes();
    let start_bytes = (start as u64).to_be_bytes();
    let end_bytes = (end as u64).to_be_bytes();
    let diagnostic_identity = deterministic_identity(
        "ddn.source_analysis_diagnostic.candidate.v1",
        &[
            revision.as_bytes(),
            &generation_bytes,
            b"W_BLOCK_HEADER_COLON_DEPRECATED",
            &occurrence_bytes,
            &start_bytes,
            &end_bytes,
        ],
    );
    let proposal_identity = deterministic_identity(
        "ddn.source_analysis_repair_proposal.candidate.v1",
        &[
            revision.as_bytes(),
            diagnostic_identity.as_bytes(),
            &start_bytes,
            &end_bytes,
            b":",
            b"",
        ],
    );
    let proposal = SourceAnalysisRepairProposal {
        schema: REPAIR_PROPOSAL_SCHEMA,
        fix_id: REMOVE_SETUP_COLON_FIX_ID,
        source_revision_identity: revision.to_string(),
        analysis_generation: generation,
        diagnostic_identity: diagnostic_identity.clone(),
        original_source_span: SourceAnalysisOriginalSourceSpan {
            coordinate_space: "original-source",
            encoding: "utf-8-byte",
            half_open: true,
            start,
            end,
        },
        expected_text: ":".to_string(),
        replacement_text: String::new(),
        proposal_identity,
        explanation: "비권장 블록 헤더의 콜론을 제거합니다.",
    };
    Some((diagnostic_identity, proposal))
}

fn accepted_diagnostics(
    source: &str,
    path: &str,
    revision: &str,
    generation: u64,
    warnings: &[DdnParseWarning],
) -> Vec<SourceAnalysisDiagnostic> {
    let repairable_count = warnings
        .iter()
        .filter(|warning| is_chaebi_header_warning(warning))
        .count();
    let raw_spans =
        raw_chaebi_header_colon_spans(source).filter(|spans| spans.len() == repairable_count);
    let mut repairable_ordinal = 0usize;

    warnings
        .iter()
        .map(|warning| {
            let repair = if is_chaebi_header_warning(warning) {
                let ordinal = repairable_ordinal;
                repairable_ordinal += 1;
                raw_spans.as_ref().and_then(|spans| {
                    spans.get(ordinal).and_then(|span| {
                        authoritative_repair_proposal(source, revision, generation, ordinal, *span)
                    })
                })
            } else {
                None
            };
            let (diagnostic_identity, repair_proposal) = match repair {
                Some((identity, proposal)) => (Some(identity), Some(proposal)),
                None => (None, None),
            };
            SourceAnalysisDiagnostic {
                code: warning.code.clone(),
                severity: "warning",
                message: warning.message.clone(),
                path: path.to_string(),
                span: SourceAnalysisSpan {
                    encoding: "utf-8-byte",
                    start: warning.span_start,
                    end: warning.span_end,
                },
                diagnostic_identity,
                repair_proposal,
            }
        })
        .collect()
}

fn diagnostic_code(message: &str) -> String {
    let Some(start) = message.find("E_") else {
        return "E_PARSE".to_string();
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

fn byte_offset_for_line_column(source: &str, target_line: usize, target_column: usize) -> usize {
    let mut line = 1usize;
    let mut column = 1usize;
    for (offset, ch) in source.char_indices() {
        if line == target_line && column >= target_column {
            return offset;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    source.len()
}

fn formatted_error_span(source: &str, message: &str) -> SourceAnalysisSpan {
    let mut start = 0usize;
    if let Some(location_end) = message.find(")\n") {
        if let Some(location_start) = message[..location_end].rfind('(') {
            let location = &message[location_start + 1..location_end];
            if let Some((line, column)) = location.split_once(':') {
                if let (Ok(line), Ok(column)) = (line.parse::<usize>(), column.parse::<usize>()) {
                    start = byte_offset_for_line_column(source, line, column);
                }
            }
        }
    }
    let end = source[start..]
        .chars()
        .next()
        .map(|ch| start + ch.len_utf8())
        .unwrap_or(start);
    SourceAnalysisSpan {
        encoding: "utf-8-byte",
        start,
        end,
    }
}

pub fn analyze_source(request: SourceAnalysisRequest) -> SourceAnalysisSnapshot {
    let revision = source_revision_identity(&request.source_text);
    let program = if request.source_profile_identity == STRICT_SOURCE_PROFILE_IDENTITY {
        DdnProgram::from_source_with_mode(&request.source_text, &request.path, ParseMode::Strict)
    } else {
        let profile = match resolve_supported_source_profile(Some(&request.source_profile_identity)) {
            Ok(profile) => profile,
            Err(error) => {
                return SourceAnalysisSnapshot {
                    schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
                    source_revision_identity: revision,
                    analysis_generation: request.analysis_generation,
                    source_profile_identity: request.source_profile_identity,
                    outcome: "UNSUPPORTED",
                    diagnostics: vec![SourceAnalysisDiagnostic {
                        code: error.code.to_string(),
                        severity: "error",
                        message: error.message,
                        path: request.path,
                        span: SourceAnalysisSpan {
                            encoding: "utf-8-byte",
                            start: 0,
                            end: 0,
                        },
                        diagnostic_identity: None,
                        repair_proposal: None,
                    }],
                };
            }
        };
        DdnProgram::from_supported_source_profile_with_mode(
            &request.source_text,
            &request.path,
            ParseMode::Strict,
            profile,
        )
    };

    match program {
        Ok(program) => SourceAnalysisSnapshot {
            schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
            source_revision_identity: revision.clone(),
            analysis_generation: request.analysis_generation,
            source_profile_identity: request.source_profile_identity,
            outcome: "ACCEPTED",
            diagnostics: accepted_diagnostics(
                &request.source_text,
                &request.path,
                &revision,
                request.analysis_generation,
                program.parse_warnings(),
            ),
        },
        Err(message) => SourceAnalysisSnapshot {
            schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
            source_revision_identity: revision,
            analysis_generation: request.analysis_generation,
            source_profile_identity: request.source_profile_identity,
            outcome: "REJECTED",
            diagnostics: vec![SourceAnalysisDiagnostic {
                code: diagnostic_code(&message),
                severity: "error",
                span: formatted_error_span(&request.source_text, &message),
                message,
                path: request.path,
                diagnostic_identity: None,
                repair_proposal: None,
            }],
        },
    }
}

pub fn source_analysis_snapshot_json(request_json: &str) -> Result<String, String> {
    let request: SourceAnalysisRequest = serde_json::from_str(request_json)
        .map_err(|error| format!("E_SOURCE_ANALYSIS_REQUEST_INVALID {error}"))?;
    serde_json::to_string(&analyze_source(request))
        .map_err(|error| format!("E_SOURCE_ANALYSIS_SERIALIZE_FAILED {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(source: &str, generation: u64, profile: &str) -> SourceAnalysisRequest {
        SourceAnalysisRequest {
            source_text: source.to_string(),
            path: "lesson.ddn".to_string(),
            analysis_generation: generation,
            source_profile_identity: profile.to_string(),
        }
    }

    #[test]
    fn accepted_snapshot_preserves_identity_generation_and_utf8_warning_span() {
        let source = "매틱:움직씨 = {\n  채비: { 값:수 <- 0. }.\n}";
        let snapshot = analyze_source(request(source, 7, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_eq!(snapshot.outcome, "ACCEPTED");
        assert_eq!(snapshot.analysis_generation, 7);
        assert_eq!(snapshot.source_profile_identity, "strict");
        assert!(snapshot.source_revision_identity.starts_with("blake3:"));
        let warning = snapshot
            .diagnostics
            .iter()
            .find(|row| row.code == "W_BLOCK_HEADER_COLON_DEPRECATED")
            .expect("authoritative warning");
        assert_eq!(warning.severity, "warning");
        assert_eq!(warning.span.encoding, "utf-8-byte");
        assert!(warning.span.end > warning.span.start);
        let proposal = warning
            .repair_proposal
            .as_ref()
            .expect("authoritative repair proposal");
        let original_colon = source.find("채비:").unwrap() + "채비".len();
        assert_eq!(proposal.original_source_span.start, original_colon);
        assert_eq!(proposal.original_source_span.end, original_colon + 1);
        assert_eq!(
            proposal.original_source_span.coordinate_space,
            "original-source"
        );
        assert_eq!(proposal.expected_text, ":");
        assert_eq!(proposal.replacement_text, "");
        assert_eq!(
            warning.diagnostic_identity.as_deref(),
            Some(proposal.diagnostic_identity.as_str())
        );
    }

    fn proposal_spans(snapshot: &SourceAnalysisSnapshot) -> Vec<(usize, usize)> {
        snapshot
            .diagnostics
            .iter()
            .filter_map(|row| row.repair_proposal.as_ref())
            .map(|proposal| {
                (
                    proposal.original_source_span.start,
                    proposal.original_source_span.end,
                )
            })
            .collect()
    }

    #[test]
    fn authoritative_repair_proposals_match_original_a_to_h() {
        let cases: Vec<(&str, String, Vec<usize>)> = vec![
            (
                "A_ASCII",
                "매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "B_KOREAN_COMMENT",
                "// 앞한글\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "C_NON_BMP_COMMENT",
                "// 😀\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "D_COMBINING_COMMENT",
                "// e\u{301}\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "E_MULTIPLE",
                "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }"
                    .to_string(),
                Vec::new(),
            ),
            (
                "F_STRING_DECOY",
                "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"
                    .to_string(),
                Vec::new(),
            ),
            (
                "G_COMMENT_DECOY",
                "// 채비: { 주석 decoy\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"
                    .to_string(),
                Vec::new(),
            ),
            (
                "H_OPAQUE_DECOY",
                "매틱:움직씨 = { 채비 { 이름:글 <- (x=1) 글무늬{\"채비:\"}. }. 채비: { 값:수 <- 1. }. }"
                    .to_string(),
                Vec::new(),
            ),
        ];

        for (case_id, source, _) in cases {
            let expected = if case_id == "E_MULTIPLE" {
                source
                    .match_indices("채비:")
                    .map(|(start, _)| start + "채비".len())
                    .collect::<Vec<_>>()
            } else {
                vec![source.rfind("채비:").unwrap() + "채비".len()]
            };
            let snapshot = analyze_source(request(&source, 41, STRICT_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{case_id}");
            assert_eq!(
                proposal_spans(&snapshot),
                expected
                    .iter()
                    .map(|start| (*start, *start + 1))
                    .collect::<Vec<_>>(),
                "{case_id}"
            );
            for diagnostic in snapshot
                .diagnostics
                .iter()
                .filter(|row| row.code == "W_BLOCK_HEADER_COLON_DEPRECATED")
            {
                let proposal = diagnostic.repair_proposal.as_ref().expect(case_id);
                assert_eq!(
                    source.get(
                        proposal.original_source_span.start..proposal.original_source_span.end
                    ),
                    Some(":"),
                    "{case_id}"
                );
            }
        }
    }

    #[test]
    fn multiple_occurrence_identities_are_unique_and_deterministic() {
        let source = "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }";
        let first = analyze_source(request(source, 42, STRICT_SOURCE_PROFILE_IDENTITY));
        let repeated = analyze_source(request(source, 42, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_eq!(first, repeated);
        let diagnostics = first
            .diagnostics
            .iter()
            .filter(|row| row.repair_proposal.is_some())
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 2);
        assert_ne!(
            diagnostics[0].diagnostic_identity,
            diagnostics[1].diagnostic_identity
        );
        assert_ne!(
            diagnostics[0]
                .repair_proposal
                .as_ref()
                .unwrap()
                .proposal_identity,
            diagnostics[1]
                .repair_proposal
                .as_ref()
                .unwrap()
                .proposal_identity
        );
        let first_span = &diagnostics[0]
            .repair_proposal
            .as_ref()
            .unwrap()
            .original_source_span;
        let second_span = &diagnostics[1]
            .repair_proposal
            .as_ref()
            .unwrap()
            .original_source_span;
        assert_ne!(first_span, second_span);
        assert_eq!(source.get(first_span.start..first_span.end), Some(":"));
        assert_eq!(source.get(second_span.start..second_span.end), Some(":"));
        assert!(first_span.start < second_span.start);
        let next_generation = analyze_source(request(source, 43, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_ne!(
            diagnostics[0].diagnostic_identity,
            next_generation.diagnostics[0].diagnostic_identity
        );
    }

    #[test]
    fn string_decoy_is_not_mutated_and_reanalysis_advances_revision() {
        let source = "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }";
        let snapshot = analyze_source(request(source, 44, STRICT_SOURCE_PROFILE_IDENTITY));
        let proposal = snapshot.diagnostics[0].repair_proposal.as_ref().unwrap();
        let mut edited = source.to_string();
        edited.replace_range(
            proposal.original_source_span.start..proposal.original_source_span.end,
            &proposal.replacement_text,
        );
        assert!(edited.contains("제목: \"채비: {\""));
        assert!(edited.contains("매틱:움직씨 = { 채비 {"));
        assert!(!edited.contains("매틱:움직씨 = { 채비: {"));
        let reanalyzed = analyze_source(request(&edited, 45, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_ne!(
            snapshot.source_revision_identity,
            reanalyzed.source_revision_identity
        );
        assert!(!reanalyzed
            .diagnostics
            .iter()
            .any(|row| row.code == "W_BLOCK_HEADER_COLON_DEPRECATED"));
    }

    #[test]
    fn native_payload_matches_t4h_cross_projection_seal() {
        let source = "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }";
        let snapshot = analyze_source(request(source, 270, STRICT_SOURCE_PROFILE_IDENTITY));
        let diagnostic = &snapshot.diagnostics[0];
        let proposal = diagnostic.repair_proposal.as_ref().unwrap();
        assert_eq!(
            snapshot.source_revision_identity,
            "blake3:3159cfef3cdbeeeb74ed267fe9cae2a12944661234c0a942b38c1339057abaab"
        );
        assert_eq!(
            diagnostic.diagnostic_identity.as_deref(),
            Some("ddn.source_analysis_diagnostic.candidate.v1:blake3:9ac895408889cb0cd9b7b2aaf73a98eb72637242512f8204fbdc9649c6e79e1f")
        );
        assert_eq!(
            proposal.proposal_identity,
            "ddn.source_analysis_repair_proposal.candidate.v1:blake3:b36a7c471fdcb01548dd56902ec27e449faeaf9bac72da74a7c67c5a248f7158"
        );
        assert_eq!(proposal.original_source_span.start, 60);
        assert_eq!(proposal.original_source_span.end, 61);
        assert_eq!(proposal.expected_text, ":");
        assert_eq!(proposal.replacement_text, "");
    }

    #[test]
    fn association_mismatch_preserves_diagnostic_without_proposal() {
        let source = "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }";
        let warnings = vec![DdnParseWarning {
            code: "W_BLOCK_HEADER_COLON_DEPRECATED".to_string(),
            message: "블록 헤더의 `채비:` 표기는 예정된 비권장입니다.".to_string(),
            span_start: 0,
            span_end: 1,
        }];
        let diagnostics =
            accepted_diagnostics(source, "lesson.ddn", "blake3:mismatch", 46, &warnings);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].diagnostic_identity.is_none());
        assert!(diagnostics[0].repair_proposal.is_none());
    }

    #[test]
    fn rejected_snapshot_is_fail_closed() {
        let snapshot = analyze_source(request(
            "매틱:움직씨 = { 값:수 <- . }",
            8,
            STRICT_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(snapshot.diagnostics.len(), 1);
        assert_eq!(snapshot.diagnostics[0].severity, "error");
        assert_eq!(snapshot.diagnostics[0].span.encoding, "utf-8-byte");
    }

    #[test]
    fn unsupported_profile_is_not_downgraded() {
        let snapshot = analyze_source(request("값:수 <- 1.", 9, "unknown-profile"));
        assert_eq!(snapshot.outcome, "UNSUPPORTED");
        assert_eq!(snapshot.diagnostics[0].code, "E_SOURCE_PROFILE_UNSUPPORTED");
    }

    const P1A_V1_CORE_PROFILE: &str = "v1-core-v25";

    #[test]
    fn p1a_v1_core_profile_accepts_definition_and_rebinding() {
        let source = "계산:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n";
        let snapshot = analyze_source(request(source, 101, P1A_V1_CORE_PROFILE));
        assert_eq!(snapshot.outcome, "ACCEPTED");
        assert!(snapshot.diagnostics.is_empty());
    }

    #[test]
    fn p1a_v1_core_profile_reports_exact_legacy_definition_migration() {
        let source = "계산:움직씨 = {\n}\n";
        let snapshot = analyze_source(request(source, 102, P1A_V1_CORE_PROFILE));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(
            snapshot.diagnostics[0].code,
            "E_V25_DEFINITION_MIGRATION_REQUIRED"
        );
    }

    #[test]
    fn p1a_v1_core_profile_rejects_direct_and_mutual_recursion_deterministically() {
        let direct = "첫계산:움직씨 := {\n    () 첫계산하기.\n}\n";
        let mutual = "첫계산:움직씨 := {\n    () 둘계산하기.\n}\n둘계산:움직씨 := {\n    () 첫계산하기.\n}\n";
        for source in [direct, mutual] {
            let first = analyze_source(request(source, 103, P1A_V1_CORE_PROFILE));
            let repeated = analyze_source(request(source, 103, P1A_V1_CORE_PROFILE));
            assert_eq!(first, repeated);
            assert_eq!(first.outcome, "REJECTED");
            assert_eq!(
                first.diagnostics[0].code,
                "E_V1_CORE_RECURSION_UNSUPPORTED",
                "diagnostic={:?}",
                first.diagnostics
            );
        }
    }

    #[test]
    fn p1a_v1_core_profile_omission_fails_closed_without_default_or_retry() {
        let request_json = serde_json::json!({
            "source_text": "계산:움직씨 := {\n    값:수 := 1.\n}\n",
            "path": "omitted-profile.ddn",
            "analysis_generation": 104
        })
        .to_string();
        let response = source_analysis_snapshot_json(&request_json).expect("structured response");
        let snapshot: serde_json::Value =
            serde_json::from_str(&response).expect("snapshot response");
        assert_eq!(snapshot["outcome"], "UNSUPPORTED");
        assert_eq!(snapshot["diagnostics"][0]["code"], "E_SOURCE_PROFILE_REQUIRED");
    }
}
