//! 봉인 corpus 저작을 위한 공통 CSV exact-byte intake.
//!
//! 이 모듈은 CSV를 정본 artifact로 정의하지 않는다. 입력과 출력의
//! `seamgrim.internal.*` transport는 CLI/WASM/Workbench 사이의 비정본 제품
//! 요청이며, 후속 저작 단계가 기존 sealed dataset builder를 호출한다.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

use crate::corpus_authoring::CorpusAuthoringError;

pub const CORPUS_CSV_INTAKE_TRANSPORT: &str = "seamgrim.internal.corpus_csv_intake.v1";
const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
const MAX_CELL_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusCsvIntakeRequest {
    pub transport_kind: String,
    pub source_bytes: Vec<u8>,
    pub options: CorpusCsvOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusCsvOptions {
    pub encoding: String,
    pub delimiter: String,
    pub header: bool,
    pub quote_rule: CorpusCsvQuoteRule,
    pub newline_policy: CorpusCsvNewlinePolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusCsvQuoteRule {
    DoubleQuote,
    Literal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusCsvNewlinePolicy {
    Lf,
    CrLf,
    Consistent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusCsvStatus {
    Ready,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusCsvDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusCsvSourceSpan {
    pub byte_start: usize,
    pub byte_end: usize,
    pub line: usize,
    pub column_byte: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusCsvDiagnostic {
    pub code: &'static str,
    pub severity: CorpusCsvDiagnosticSeverity,
    pub message: String,
    pub source_span: CorpusCsvSourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusCsvIntakeResult {
    pub transport_kind: &'static str,
    pub status: CorpusCsvStatus,
    pub options: CorpusCsvOptions,
    pub source_bytes_sha256: String,
    pub source_byte_length: usize,
    pub source_column_count: usize,
    pub ordered_column_names: Vec<String>,
    pub ordered_raw_rows: Vec<Vec<String>>,
    pub diagnostics: Vec<CorpusCsvDiagnostic>,
}

#[derive(Debug, Clone)]
struct ParsedCell {
    text: String,
    start: usize,
    end: usize,
}

pub fn build_corpus_csv_intake_json(request_json: &str) -> Result<String, CorpusAuthoringError> {
    let request: CorpusCsvIntakeRequest =
        serde_json::from_str(request_json).map_err(|err| CorpusAuthoringError {
            code: "E_CORPUS_CSV_REQUEST_JSON".to_string(),
            path: "$".to_string(),
            message: format!("CSV intake 요청 JSON을 읽을 수 없습니다: {err}"),
        })?;
    if request.transport_kind != CORPUS_CSV_INTAKE_TRANSPORT {
        return Err(CorpusAuthoringError {
            code: "E_CORPUS_CSV_TRANSPORT".to_string(),
            path: "$.transport_kind".to_string(),
            message: format!("transport_kind는 {CORPUS_CSV_INTAKE_TRANSPORT}이어야 합니다"),
        });
    }
    let result = parse_corpus_csv_bytes(&request.source_bytes, request.options)?;
    serde_json::to_string(&result)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|err| CorpusAuthoringError {
            code: "E_CORPUS_CSV_SERIALIZE".to_string(),
            path: "$".to_string(),
            message: format!("CSV intake 결과를 직렬화할 수 없습니다: {err}"),
        })
}

pub fn parse_corpus_csv_bytes(
    source_bytes: &[u8],
    options: CorpusCsvOptions,
) -> Result<CorpusCsvIntakeResult, CorpusAuthoringError> {
    validate_options(&options)?;
    let source_bytes_sha256 = hex::encode(Sha256::digest(source_bytes));
    let mut diagnostics = Vec::new();
    if source_bytes.len() > MAX_SOURCE_BYTES {
        diagnostics.push(diagnostic(
            source_bytes,
            "E_CORPUS_CSV_SOURCE_TOO_LARGE",
            CorpusCsvDiagnosticSeverity::Error,
            format!(
                "CSV 원본은 {MAX_SOURCE_BYTES} bytes를 넘을 수 없습니다: {}",
                source_bytes.len()
            ),
            MAX_SOURCE_BYTES.min(source_bytes.len()),
            source_bytes.len(),
        ));
        return Ok(result(
            options,
            source_bytes_sha256,
            source_bytes.len(),
            diagnostics,
        ));
    }
    if source_bytes.starts_with(&[0xef, 0xbb, 0xbf])
        || source_bytes.starts_with(&[0xff, 0xfe])
        || source_bytes.starts_with(&[0xfe, 0xff])
    {
        diagnostics.push(diagnostic(
            source_bytes,
            "E_CORPUS_CSV_BOM_UNSUPPORTED",
            CorpusCsvDiagnosticSeverity::Error,
            "BOM이 있는 CSV는 받지 않습니다. encoding declaration과 exact bytes를 다시 확인하세요",
            0,
            source_bytes.len().min(3),
        ));
        return Ok(result(
            options,
            source_bytes_sha256,
            source_bytes.len(),
            diagnostics,
        ));
    }
    if let Err(error) = std::str::from_utf8(source_bytes) {
        let start = error.valid_up_to();
        let end = start + error.error_len().unwrap_or(1);
        diagnostics.push(diagnostic(
            source_bytes,
            "E_CORPUS_CSV_INVALID_UTF8",
            CorpusCsvDiagnosticSeverity::Error,
            "encoding=utf-8 선언과 원본 bytes가 일치하지 않습니다",
            start,
            end.min(source_bytes.len()),
        ));
        return Ok(result(
            options,
            source_bytes_sha256,
            source_bytes.len(),
            diagnostics,
        ));
    }

    validate_newlines(source_bytes, options.newline_policy, &mut diagnostics);
    let delimiter = options.delimiter.as_bytes()[0];
    let records = parse_records(
        source_bytes,
        delimiter,
        options.quote_rule,
        &mut diagnostics,
    );
    let mut records = match records {
        Some(records) => records,
        None => {
            return Ok(result(
                options,
                source_bytes_sha256,
                source_bytes.len(),
                diagnostics,
            ));
        }
    };
    if records.is_empty() {
        diagnostics.push(diagnostic(
            source_bytes,
            "E_CORPUS_CSV_EMPTY",
            CorpusCsvDiagnosticSeverity::Error,
            "CSV에는 하나 이상의 record가 있어야 합니다",
            0,
            0,
        ));
        return Ok(result(
            options,
            source_bytes_sha256,
            source_bytes.len(),
            diagnostics,
        ));
    }

    let header = if options.header {
        let header = records.remove(0);
        validate_header(source_bytes, &header, &mut diagnostics);
        header
    } else {
        Vec::new()
    };
    let expected_width = if options.header {
        header.len()
    } else {
        records.first().map(Vec::len).unwrap_or(0)
    };
    for (row_index, row) in records.iter().enumerate() {
        if row.len() != expected_width {
            let (start, end) = row
                .first()
                .map(|cell| (cell.start, cell.end))
                .unwrap_or((source_bytes.len(), source_bytes.len()));
            diagnostics.push(diagnostic(
                source_bytes,
                "E_CORPUS_CSV_ROW_WIDTH",
                CorpusCsvDiagnosticSeverity::Error,
                format!(
                    "data row {}의 cell 수 {}가 선언된 열 수 {expected_width}와 다릅니다",
                    row_index + 1,
                    row.len()
                ),
                start,
                end,
            ));
        }
        for cell in row {
            if matches!(
                cell.text.as_bytes().first(),
                Some(b'=') | Some(b'+') | Some(b'-') | Some(b'@')
            ) {
                diagnostics.push(diagnostic(
                    source_bytes,
                    "W_CORPUS_CSV_FORMULA_LIKE_TEXT",
                    CorpusCsvDiagnosticSeverity::Warning,
                    "formula-like leading text를 실행하거나 변환하지 않고 raw text로 보존했습니다",
                    cell.start,
                    cell.end,
                ));
            }
        }
    }

    let ordered_column_names = header.into_iter().map(|cell| cell.text).collect();
    let ordered_raw_rows = records
        .into_iter()
        .map(|row| row.into_iter().map(|cell| cell.text).collect())
        .collect();
    Ok(CorpusCsvIntakeResult {
        transport_kind: CORPUS_CSV_INTAKE_TRANSPORT,
        status: status_for(&diagnostics),
        options,
        source_bytes_sha256,
        source_byte_length: source_bytes.len(),
        source_column_count: expected_width,
        ordered_column_names,
        ordered_raw_rows,
        diagnostics,
    })
}

fn validate_options(options: &CorpusCsvOptions) -> Result<(), CorpusAuthoringError> {
    if options.encoding != "utf-8" {
        return Err(CorpusAuthoringError {
            code: "E_CORPUS_CSV_ENCODING".to_string(),
            path: "$.options.encoding".to_string(),
            message: "encoding은 exact text `utf-8`만 지원합니다".to_string(),
        });
    }
    let bytes = options.delimiter.as_bytes();
    if bytes.len() != 1 || !bytes[0].is_ascii() || matches!(bytes[0], b'\r' | b'\n' | b'"') {
        return Err(CorpusAuthoringError {
            code: "E_CORPUS_CSV_DELIMITER".to_string(),
            path: "$.options.delimiter".to_string(),
            message: "delimiter는 quote/newline이 아닌 ASCII 1 byte여야 합니다".to_string(),
        });
    }
    Ok(())
}

fn validate_newlines(
    bytes: &[u8],
    policy: CorpusCsvNewlinePolicy,
    diagnostics: &mut Vec<CorpusCsvDiagnostic>,
) {
    let mut lf = Vec::new();
    let mut crlf = Vec::new();
    let mut bare_cr = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                crlf.push(index);
                index += 2;
            }
            b'\r' => {
                bare_cr.push(index);
                index += 1;
            }
            b'\n' => {
                lf.push(index);
                index += 1;
            }
            _ => index += 1,
        }
    }
    for index in bare_cr {
        diagnostics.push(diagnostic(
            bytes,
            "E_CORPUS_CSV_BARE_CR",
            CorpusCsvDiagnosticSeverity::Error,
            "단독 CR newline은 지원하지 않습니다",
            index,
            index + 1,
        ));
    }
    let mismatch = match policy {
        CorpusCsvNewlinePolicy::Lf => crlf.first().copied(),
        CorpusCsvNewlinePolicy::CrLf => lf.first().copied(),
        CorpusCsvNewlinePolicy::Consistent if !lf.is_empty() && !crlf.is_empty() => {
            Some(lf[0].max(crlf[0]))
        }
        CorpusCsvNewlinePolicy::Consistent => None,
    };
    if let Some(index) = mismatch {
        diagnostics.push(diagnostic(
            bytes,
            "E_CORPUS_CSV_NEWLINE_POLICY",
            CorpusCsvDiagnosticSeverity::Error,
            "원본 newline bytes가 선언한 정책과 일치하지 않거나 LF/CRLF가 섞였습니다",
            index,
            (index + 2).min(bytes.len()),
        ));
    }
}

fn parse_records(
    bytes: &[u8],
    delimiter: u8,
    quote_rule: CorpusCsvQuoteRule,
    diagnostics: &mut Vec<CorpusCsvDiagnostic>,
) -> Option<Vec<Vec<ParsedCell>>> {
    let mut records = Vec::new();
    let mut row = Vec::new();
    let mut cell = Vec::new();
    let mut cell_start = 0;
    let mut quote_start = 0;
    let mut in_quotes = false;
    let mut after_quote = false;
    let mut record_touched = false;
    let mut index = 0;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_quotes {
            if byte == b'"' {
                if bytes.get(index + 1) == Some(&b'"') {
                    cell.push(b'"');
                    index += 2;
                    continue;
                }
                in_quotes = false;
                after_quote = true;
                index += 1;
                continue;
            }
            cell.push(byte);
            if cell.len() > MAX_CELL_BYTES {
                diagnostics.push(diagnostic(
                    bytes,
                    "E_CORPUS_CSV_CELL_TOO_LONG",
                    CorpusCsvDiagnosticSeverity::Error,
                    format!("cell은 {MAX_CELL_BYTES} bytes를 넘을 수 없습니다"),
                    cell_start,
                    index + 1,
                ));
                return None;
            }
            index += 1;
            continue;
        }
        if after_quote {
            if byte == delimiter {
                push_cell(&mut row, &mut cell, cell_start, index);
                record_touched = true;
                after_quote = false;
                index += 1;
                cell_start = index;
                continue;
            }
            if byte == b'\n' || byte == b'\r' {
                push_cell(&mut row, &mut cell, cell_start, index);
                let width = if byte == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
                records.push(std::mem::take(&mut row));
                record_touched = false;
                after_quote = false;
                index += width;
                cell_start = index;
                continue;
            }
            diagnostics.push(diagnostic(
                bytes,
                "E_CORPUS_CSV_AFTER_QUOTE",
                CorpusCsvDiagnosticSeverity::Error,
                "닫힌 quote 뒤에는 delimiter, newline, EOF만 올 수 있습니다",
                index,
                index + 1,
            ));
            return None;
        }
        if quote_rule == CorpusCsvQuoteRule::DoubleQuote && byte == b'"' {
            if cell.is_empty() && index == cell_start {
                in_quotes = true;
                quote_start = index;
                record_touched = true;
                index += 1;
                continue;
            }
            diagnostics.push(diagnostic(
                bytes,
                "E_CORPUS_CSV_QUOTE_POSITION",
                CorpusCsvDiagnosticSeverity::Error,
                "double_quote 규칙의 quote는 cell 시작에서만 열 수 있습니다",
                index,
                index + 1,
            ));
            return None;
        }
        if byte == delimiter {
            push_cell(&mut row, &mut cell, cell_start, index);
            record_touched = true;
            index += 1;
            cell_start = index;
            continue;
        }
        if byte == b'\n' || byte == b'\r' {
            push_cell(&mut row, &mut cell, cell_start, index);
            let width = if byte == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                2
            } else {
                1
            };
            records.push(std::mem::take(&mut row));
            record_touched = false;
            index += width;
            cell_start = index;
            continue;
        }
        cell.push(byte);
        record_touched = true;
        if cell.len() > MAX_CELL_BYTES {
            diagnostics.push(diagnostic(
                bytes,
                "E_CORPUS_CSV_CELL_TOO_LONG",
                CorpusCsvDiagnosticSeverity::Error,
                format!("cell은 {MAX_CELL_BYTES} bytes를 넘을 수 없습니다"),
                cell_start,
                index + 1,
            ));
            return None;
        }
        index += 1;
    }
    if in_quotes {
        diagnostics.push(diagnostic(
            bytes,
            "E_CORPUS_CSV_UNCLOSED_QUOTE",
            CorpusCsvDiagnosticSeverity::Error,
            "열린 quote가 EOF 전에 닫히지 않았습니다",
            quote_start,
            bytes.len(),
        ));
        return None;
    }
    if after_quote || record_touched || !cell.is_empty() || !row.is_empty() {
        push_cell(&mut row, &mut cell, cell_start, bytes.len());
        records.push(row);
    }
    Some(records)
}

fn push_cell(row: &mut Vec<ParsedCell>, bytes: &mut Vec<u8>, start: usize, end: usize) {
    let text = String::from_utf8(std::mem::take(bytes))
        .expect("전체 source UTF-8 검증 뒤 cell bytes도 UTF-8이다");
    row.push(ParsedCell { text, start, end });
}

fn validate_header(
    bytes: &[u8],
    header: &[ParsedCell],
    diagnostics: &mut Vec<CorpusCsvDiagnostic>,
) {
    let mut names = HashSet::new();
    for cell in header {
        if cell.text.trim().is_empty() {
            diagnostics.push(diagnostic(
                bytes,
                "E_CORPUS_CSV_HEADER_EMPTY",
                CorpusCsvDiagnosticSeverity::Error,
                "header 이름은 비어 있을 수 없습니다",
                cell.start,
                cell.end,
            ));
        } else if !names.insert(cell.text.as_str()) {
            diagnostics.push(diagnostic(
                bytes,
                "E_CORPUS_CSV_HEADER_DUPLICATE",
                CorpusCsvDiagnosticSeverity::Error,
                format!("header 이름이 중복됩니다: {}", cell.text),
                cell.start,
                cell.end,
            ));
        }
    }
}

fn result(
    options: CorpusCsvOptions,
    source_bytes_sha256: String,
    source_byte_length: usize,
    diagnostics: Vec<CorpusCsvDiagnostic>,
) -> CorpusCsvIntakeResult {
    CorpusCsvIntakeResult {
        transport_kind: CORPUS_CSV_INTAKE_TRANSPORT,
        status: status_for(&diagnostics),
        options,
        source_bytes_sha256,
        source_byte_length,
        source_column_count: 0,
        ordered_column_names: Vec::new(),
        ordered_raw_rows: Vec::new(),
        diagnostics,
    }
}

fn status_for(diagnostics: &[CorpusCsvDiagnostic]) -> CorpusCsvStatus {
    if diagnostics
        .iter()
        .any(|item| item.severity == CorpusCsvDiagnosticSeverity::Error)
    {
        CorpusCsvStatus::Rejected
    } else {
        CorpusCsvStatus::Ready
    }
}

fn diagnostic(
    bytes: &[u8],
    code: &'static str,
    severity: CorpusCsvDiagnosticSeverity,
    message: impl Into<String>,
    start: usize,
    end: usize,
) -> CorpusCsvDiagnostic {
    CorpusCsvDiagnostic {
        code,
        severity,
        message: message.into(),
        source_span: span(bytes, start, end),
    }
}

fn span(bytes: &[u8], start: usize, end: usize) -> CorpusCsvSourceSpan {
    let clamped = start.min(bytes.len());
    let mut line = 1;
    let mut column_byte = 1;
    let mut index = 0;
    while index < clamped {
        if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
            line += 1;
            column_byte = 1;
            index += 2;
        } else if matches!(bytes[index], b'\r' | b'\n') {
            line += 1;
            column_byte = 1;
            index += 1;
        } else {
            column_byte += 1;
            index += 1;
        }
    }
    CorpusCsvSourceSpan {
        byte_start: clamped,
        byte_end: end.min(bytes.len()).max(clamped),
        line,
        column_byte,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> CorpusCsvOptions {
        CorpusCsvOptions {
            encoding: "utf-8".to_string(),
            delimiter: ",".to_string(),
            header: true,
            quote_rule: CorpusCsvQuoteRule::DoubleQuote,
            newline_policy: CorpusCsvNewlinePolicy::Consistent,
        }
    }

    fn parse(bytes: &[u8]) -> CorpusCsvIntakeResult {
        parse_corpus_csv_bytes(bytes, options()).expect("structural request")
    }

    fn codes(result: &CorpusCsvIntakeResult) -> Vec<&'static str> {
        result.diagnostics.iter().map(|item| item.code).collect()
    }

    #[test]
    fn exact_order_raw_text_quote_and_hash_are_preserved() {
        let bytes =
            "이름,값,메모\r\n물,2.0,\"a,b\"\r\n불,=SUM(A1),\"quote\"\"text\"\r\n".as_bytes();
        let result = parse(bytes);
        assert_eq!(result.status, CorpusCsvStatus::Ready);
        assert_eq!(result.ordered_column_names, ["이름", "값", "메모"]);
        assert_eq!(
            result.ordered_raw_rows,
            [["물", "2.0", "a,b"], ["불", "=SUM(A1)", "quote\"text"]]
        );
        assert_eq!(
            result.source_bytes_sha256,
            hex::encode(Sha256::digest(bytes))
        );
        assert_eq!(codes(&result), ["W_CORPUS_CSV_FORMULA_LIKE_TEXT"]);
    }

    #[test]
    fn invalid_utf8_and_bom_are_rejected_with_byte_spans() {
        let invalid = parse(&[b'a', b',', 0xff]);
        assert_eq!(invalid.status, CorpusCsvStatus::Rejected);
        assert_eq!(codes(&invalid), ["E_CORPUS_CSV_INVALID_UTF8"]);
        assert_eq!(invalid.diagnostics[0].source_span.byte_start, 2);

        let bom = parse(b"\xef\xbb\xbfname,value\nx,1\n");
        assert_eq!(codes(&bom), ["E_CORPUS_CSV_BOM_UNSUPPORTED"]);
    }

    #[test]
    fn mixed_newline_unclosed_quote_duplicate_and_empty_header_reject() {
        assert!(codes(&parse(b"a,b\r\n1,2\n")).contains(&"E_CORPUS_CSV_NEWLINE_POLICY"));
        assert_eq!(
            codes(&parse(b"a,b\n1,\"open\n")),
            ["E_CORPUS_CSV_UNCLOSED_QUOTE"]
        );
        assert_eq!(
            codes(&parse(b"a,a\n1,2\n")),
            ["E_CORPUS_CSV_HEADER_DUPLICATE"]
        );
        assert_eq!(codes(&parse(b"a, \n1,2\n")), ["E_CORPUS_CSV_HEADER_EMPTY"]);
    }

    #[test]
    fn row_width_mismatch_rejects_without_truncation() {
        let result = parse(b"a,b\n1,2,3\n");
        assert_eq!(result.status, CorpusCsvStatus::Rejected);
        assert_eq!(codes(&result), ["E_CORPUS_CSV_ROW_WIDTH"]);
        assert_eq!(result.ordered_raw_rows, [["1", "2", "3"]]);
    }

    #[test]
    fn headerless_input_never_synthesizes_semantic_column_names() {
        let mut no_header = options();
        no_header.header = false;
        let result = parse_corpus_csv_bytes(b"1,2\n3,4\n", no_header).expect("parse");
        assert_eq!(result.status, CorpusCsvStatus::Ready);
        assert!(result.ordered_column_names.is_empty());
        assert_eq!(result.source_column_count, 2);
    }

    #[test]
    fn very_long_cell_is_rejected() {
        let mut bytes = b"name\n".to_vec();
        bytes.extend(std::iter::repeat_n(b'x', MAX_CELL_BYTES + 1));
        let result = parse(&bytes);
        assert_eq!(codes(&result), ["E_CORPUS_CSV_CELL_TOO_LONG"]);
    }
}
