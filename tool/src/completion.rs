use crate::lsp::{
    byte_range_to_lsp_range, position_to_byte_offset, ByteRange, DocumentSnapshot, DocumentStore,
    LspPosition, LspRange, PositionEncoding, PositionError,
};
use ddonirang_lang::stdlib::canonicalize_stdlib_alias;
use ddonirang_lang::{minimal_stdlib_sigs, parse_v25, Lexer, TokenKind, TopLevelItem};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const SHARED_COMPLETION_SCHEMA: &str = "ddonilang.completion.v1-candidate";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredCompletionRequest {
    pub schema: String,
    pub profile: String,
    pub document_identity: String,
    pub document_version: i64,
    pub source: String,
    pub position: LspPositionWire,
    pub encoding: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LspPositionWire {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LspRangeWire {
    pub start: LspPositionWire,
    pub end: LspPositionWire,
}

#[derive(Debug, Clone, Serialize)]
pub struct StructuredCompletionCandidate {
    pub label: String,
    pub insert_text: String,
    pub definition_identity: String,
    pub candidate_kind: &'static str,
    pub canonical_detail: String,
    pub replacement_range: LspRangeWire,
}

#[derive(Debug, Clone, Serialize)]
pub struct StructuredCompletionResponse {
    pub schema: &'static str,
    pub profile: String,
    pub document_identity: String,
    pub document_version: i64,
    pub snapshot_identity: String,
    pub position: LspPositionWire,
    pub candidates: Vec<StructuredCompletionCandidate>,
}

pub fn build_structured_completion_json(request_json: &str) -> Result<String, String> {
    let request: StructuredCompletionRequest = serde_json::from_str(request_json)
        .map_err(|error| format!("completion request JSON을 읽을 수 없습니다: {error}"))?;
    if request.schema != SHARED_COMPLETION_SCHEMA {
        return Err(format!("schema는 {SHARED_COMPLETION_SCHEMA} 이어야 합니다"));
    }
    if request.profile.is_empty() || request.document_identity.is_empty() {
        return Err("completion profile과 document_identity가 필요합니다".to_string());
    }
    let encoding = PositionEncoding::from_lsp_name(&request.encoding)
        .ok_or_else(|| format!("지원하지 않는 position encoding: {}", request.encoding))?;
    let position = LspPosition {
        line: request.position.line,
        character: request.position.character,
    };
    let mut store = DocumentStore::new();
    store
        .open(
            request.document_identity.clone(),
            "ddn",
            request.document_version,
            request.source.as_bytes().to_vec(),
        )
        .map_err(|error| error.to_string())?;
    let snapshot = store
        .get(&request.document_identity)
        .ok_or_else(|| "completion snapshot을 만들지 못했습니다".to_string())?;
    let result = complete_snapshot(snapshot, position, encoding).map_err(|error| error.to_string())?;
    let candidates = result
        .candidates
        .into_iter()
        .map(|candidate| {
            let replacement_range = byte_range_to_lsp_range(
                &request.source,
                candidate.replacement_range,
                encoding,
            )
            .map_err(|error| error.to_string())?;
            Ok(StructuredCompletionCandidate {
                label: candidate.label,
                insert_text: candidate.insert_text,
                definition_identity: candidate.definition_identity,
                candidate_kind: match candidate.candidate_kind {
                    CompletionCandidateKind::LocalDefinition => "local_definition",
                    CompletionCandidateKind::StdlibCallable => "stdlib_callable",
                },
                canonical_detail: candidate.canonical_detail,
                replacement_range: lsp_range_wire(replacement_range),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    serde_json::to_string(&StructuredCompletionResponse {
        schema: SHARED_COMPLETION_SCHEMA,
        profile: request.profile,
        document_identity: request.document_identity,
        document_version: result.document_version,
        snapshot_identity: result.snapshot_identity,
        position: request.position,
        candidates,
    })
    .map(|mut text| {
        text.push('\n');
        text
    })
    .map_err(|error| format!("completion response를 직렬화할 수 없습니다: {error}"))
}

fn lsp_range_wire(range: LspRange) -> LspRangeWire {
    LspRangeWire {
        start: LspPositionWire {
            line: range.start.line,
            character: range.start.character,
        },
        end: LspPositionWire {
            line: range.end.line,
            character: range.end.character,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompletionCandidateKind {
    LocalDefinition,
    StdlibCallable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    pub label: String,
    pub insert_text: String,
    pub definition_identity: String,
    pub candidate_kind: CompletionCandidateKind,
    pub canonical_detail: String,
    pub replacement_range: ByteRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionResult {
    pub document_version: i64,
    pub snapshot_identity: String,
    pub position: LspPosition,
    pub candidates: Vec<CompletionCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionError {
    InvalidUtf8,
    InvalidPosition(PositionError),
}

impl fmt::Display for CompletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 => f.write_str("completion source가 UTF-8이 아닙니다"),
            Self::InvalidPosition(error) => {
                write!(f, "completion position이 잘못되었습니다: {error}")
            }
        }
    }
}

impl std::error::Error for CompletionError {}

/// Computes the one deterministic candidate list shared by all consumers.
///
/// The current source must be a valid V25 document.  Incomplete, legacy, or
/// lexically ambiguous source is deliberately fail-closed rather than being
/// interpreted by a completion-specific grammar.
pub fn complete_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<CompletionResult, CompletionError> {
    let source = snapshot.text().map_err(|_| CompletionError::InvalidUtf8)?;
    let cursor = position_to_byte_offset(source, position, encoding)
        .map_err(CompletionError::InvalidPosition)?;
    let replacement_range = identifier_replacement_range(source, cursor);

    let empty = || CompletionResult {
        document_version: snapshot.version,
        snapshot_identity: snapshot.identity.as_str().to_string(),
        position,
        candidates: Vec::new(),
    };

    if snapshot.language_id != "ddn" || in_comment(source, cursor) {
        return Ok(empty());
    }

    let tokens = match Lexer::new(source).tokenize() {
        Ok(tokens) => tokens,
        Err(_) => return Ok(empty()),
    };
    if token_blocks_completion_context(&tokens, cursor)
        || in_type_schema_context(&tokens, cursor)
        || replacement_range.start > cursor
    {
        return Ok(empty());
    }

    let prefix = &source[replacement_range.start..cursor];
    if !prefix.chars().all(is_identifier_surface_char) {
        return Ok(empty());
    }

    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(empty()),
    };

    let mut candidates = Vec::new();
    let mut identities = BTreeSet::new();
    for item in program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if !seed.canonical_name.starts_with(prefix) {
            continue;
        }
        let candidate = CompletionCandidate {
            label: seed.canonical_name.clone(),
            insert_text: seed.canonical_name.clone(),
            definition_identity: format!("definition:local:{}", seed.canonical_name),
            candidate_kind: CompletionCandidateKind::LocalDefinition,
            canonical_detail: "local definition".to_string(),
            replacement_range,
        };
        if identities.insert(candidate.definition_identity.clone()) {
            candidates.push(candidate);
        }
    }

    for signature in minimal_stdlib_sigs() {
        // The registry contains compatibility spellings as well as current
        // spellings.  Only registered current names are offered here; an
        // alias is never synthesized by the completion engine.
        if canonicalize_stdlib_alias(signature.name) != signature.name
            || !signature.name.starts_with(prefix)
        {
            continue;
        }
        let identity = format!("definition:stdlib:{}", signature.name);
        if !identities.insert(identity.clone()) {
            continue;
        }
        candidates.push(CompletionCandidate {
            label: signature.name.to_string(),
            insert_text: signature.name.to_string(),
            definition_identity: identity,
            candidate_kind: CompletionCandidateKind::StdlibCallable,
            canonical_detail: format!("fn({}) -> {}", signature.params.join(", "), signature.ret),
            replacement_range,
        });
    }

    candidates.sort_by(|left, right| {
        left.candidate_kind
            .cmp(&right.candidate_kind)
            .then_with(|| left.label.as_bytes().cmp(right.label.as_bytes()))
            .then_with(|| {
                left.definition_identity
                    .as_bytes()
                    .cmp(right.definition_identity.as_bytes())
            })
    });

    Ok(CompletionResult {
        document_version: snapshot.version,
        snapshot_identity: snapshot.identity.as_str().to_string(),
        position,
        candidates,
    })
}

fn identifier_replacement_range(source: &str, cursor: usize) -> ByteRange {
    let mut start = cursor;
    while start > 0 {
        let Some((previous, ch)) = source[..start].char_indices().next_back() else {
            break;
        };
        if !is_identifier_surface_char(ch) {
            break;
        }
        start = previous;
    }
    ByteRange { start, end: cursor }
}

fn is_identifier_surface_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '.' | '?' | '!' | '\'')
}

fn token_blocks_completion_context(tokens: &[ddonirang_lang::Token], cursor: usize) -> bool {
    tokens.iter().any(|token| {
        token.span.start < cursor
            && cursor <= token.span.end
            && matches!(
                token.kind,
                TokenKind::StringLit(_) | TokenKind::Integer(_) | TokenKind::Float(_)
            )
    })
}

fn in_type_schema_context(tokens: &[ddonirang_lang::Token], cursor: usize) -> bool {
    let mut last_colon = None;
    for token in tokens {
        if token.span.start >= cursor {
            break;
        }
        match token.kind {
            TokenKind::Colon => last_colon = Some(token.span.end),
            TokenKind::Define | TokenKind::Equals | TokenKind::LBrace | TokenKind::RBrace => {
                last_colon = None
            }
            _ => {}
        }
    }
    last_colon
        .is_some_and(|colon_end| source_between_tokens_is_type_position(tokens, colon_end, cursor))
}

fn source_between_tokens_is_type_position(
    tokens: &[ddonirang_lang::Token],
    colon_end: usize,
    cursor: usize,
) -> bool {
    let mut saw_identifier = false;
    for token in tokens {
        if token.span.start < colon_end || token.span.start >= cursor {
            continue;
        }
        if matches!(
            token.kind,
            TokenKind::Ident(_) | TokenKind::LParen | TokenKind::RParen
        ) {
            saw_identifier = true;
            continue;
        }
        if matches!(token.kind, TokenKind::Comma | TokenKind::Arrow) {
            continue;
        }
        return false;
    }
    saw_identifier || tokens.iter().any(|token| token.span.end == colon_end)
}

fn in_comment(source: &str, cursor: usize) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;
    while index < cursor {
        let ch = source[index..].chars().next().unwrap();
        let width = ch.len_utf8();
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += width;
            continue;
        }
        if ch == '"' {
            in_string = true;
            index += width;
            continue;
        }
        if ch == '/' && bytes.get(index + 1) == Some(&b'/') {
            let line_end = source[index..]
                .find('\n')
                .map(|offset| index + offset)
                .unwrap_or(source.len());
            return cursor <= line_end;
        }
        index += width;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::DocumentStore;

    fn snapshot(source: &str) -> DocumentSnapshot {
        let mut store = DocumentStore::new();
        store
            .open(
                "file:///completion.ddn",
                "ddn",
                1,
                source.as_bytes().to_vec(),
            )
            .unwrap();
        store.get("file:///completion.ddn").unwrap().clone()
    }

    #[test]
    fn deterministic_local_and_stdlib_candidates_are_sorted() {
        let source = "첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { 첫함수. }";
        let document = snapshot(source);
        let result = complete_snapshot(
            &document,
            LspPosition {
                line: 1,
                character: 13,
            },
            PositionEncoding::Utf16,
        )
        .unwrap();
        assert_eq!(result.candidates[0].label, "첫함수");
        assert_eq!(
            result.candidates[0].candidate_kind,
            CompletionCandidateKind::LocalDefinition
        );
        let start = source.rfind("첫함수").unwrap();
        assert_eq!(
            result.candidates[0].replacement_range,
            ByteRange {
                start,
                end: start + "첫".len()
            }
        );
    }

    #[test]
    fn comments_strings_numbers_and_type_schema_fail_closed() {
        for (source, position) in [
            (
                "첫함수:셈씨 := { 1 돌려줘. }\n// 나",
                LspPosition {
                    line: 1,
                    character: 4,
                },
            ),
            (
                "첫함수:셈씨 := { \"나\" 보여주기. }",
                LspPosition {
                    line: 0,
                    character: 18,
                },
            ),
            (
                "첫함수:셈씨 := { 123 돌려줘. }",
                LspPosition {
                    line: 0,
                    character: 18,
                },
            ),
            (
                "첫함수:셈씨 := { 1 돌려줘. }\n나:",
                LspPosition {
                    line: 1,
                    character: 2,
                },
            ),
        ] {
            let document = snapshot(source);
            assert!(
                complete_snapshot(&document, position, PositionEncoding::Utf16)
                    .unwrap()
                    .candidates
                    .is_empty()
            );
        }
    }
}
