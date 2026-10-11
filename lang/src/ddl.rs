//! DDL Wave 1 frontdoor.
//!
//! This module accepts only the Owner-bound two-language, immutable-value
//! projection.  It is intentionally not a second general DDN parser and it
//! never creates an executable body.

use crate::ast::{
    DdlDocument, DdlExtension, ImmutablePublicValueProjection, LanguageSection, OmittedPayload,
    Span,
};
use crate::lexer::{Lexer, TokenKind};

const KOREAN: DdlRegistryEntry = DdlRegistryEntry {
    header: "#!한국어",
    profile_identity: "language-profile:ko",
    localized_name: "원주율",
    localized_type: "셈수",
    definition_identity: "definition:math.pi",
    type_identity: "type:number",
};

const JAPANESE: DdlRegistryEntry = DdlRegistryEntry {
    header: "#!日本語",
    profile_identity: "language-profile:ja",
    localized_name: "円周率",
    localized_type: "数値",
    definition_identity: "definition:math.pi",
    type_identity: "type:number",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DdlRegistryEntry {
    pub header: &'static str,
    pub profile_identity: &'static str,
    pub localized_name: &'static str,
    pub localized_type: &'static str,
    pub definition_identity: &'static str,
    pub type_identity: &'static str,
}

const REGISTRY: [DdlRegistryEntry; 2] = [KOREAN, JAPANESE];

/// Internal, version-bound registry context for the DDL Wave 1 parser.
///
/// This is deliberately a Rust-side product context, not a public DDL wire
/// or dynamic language-profile registry.  It lets the parser validate that
/// translated entries preserve their common definition and type identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlWave1Context {
    entries: Vec<DdlRegistryEntry>,
}

impl DdlWave1Context {
    pub fn standard() -> Self {
        Self {
            entries: REGISTRY.to_vec(),
        }
    }

    pub fn from_entries(entries: Vec<DdlRegistryEntry>) -> Self {
        Self { entries }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlError {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
}

impl std::fmt::Display for DdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DdlError {}

/// Parses the exact Wave 1 DDL surface.  `file_path` has extension-admission
/// authority only; it is never stored as a semantic identity.
pub fn parse_ddl_wave1(source: &str, file_path: &str) -> Result<DdlDocument, DdlError> {
    parse_ddl_wave1_with_context(source, file_path, &DdlWave1Context::standard())
}

/// Parses a Wave 1 document against an explicit internal registry context.
/// Callers must supply the fixed versioned context; source text never creates
/// profile or semantic identities dynamically.
pub fn parse_ddl_wave1_with_context(
    source: &str,
    file_path: &str,
    context: &DdlWave1Context,
) -> Result<DdlDocument, DdlError> {
    let extension = parse_extension(file_path)?;
    let headers = scan_headers(source)?;
    if headers.len() != 2 {
        return Err(error(
            "E_DDL_SECTION_COUNT",
            Span::new(0, source.len()),
            "DDL Wave 1은 정확히 두 LanguageSection이 필요합니다",
        ));
    }

    let mut sections = Vec::with_capacity(2);
    for (index, header) in headers.iter().enumerate() {
        let content_end = headers
            .get(index + 1)
            .map_or(source.len(), |next| next.start);
        let entry = registry_entry(&context.entries, header.text).ok_or_else(|| {
            error(
                if matches!(header.text, "#!qu" | "#!Quechua") {
                    "E_DDL_GENERIC_QUECHUA_FORBIDDEN"
                } else {
                    "E_DDL_PROFILE_UNKNOWN"
                },
                Span::new(header.start, header.end),
                "등록되지 않았거나 일반화된 LanguageProfile header입니다",
            )
        })?;
        if sections.iter().any(|section: &LanguageSection| {
            section.language_profile_identity == entry.profile_identity
        }) {
            return Err(error(
                "E_DDL_PROFILE_DUPLICATE",
                Span::new(header.start, header.end),
                "같은 LanguageProfileIdentity section을 두 번 쓸 수 없습니다",
            ));
        }
        let projection = parse_projection(&source[header.end..content_end], header.end, entry)?;
        sections.push(LanguageSection {
            language_profile_identity: entry.profile_identity.to_owned(),
            header_span: Span::new(header.start, header.end),
            content_span: Span::new(header.end, content_end),
            projection,
        });
    }

    let first = &sections[0].projection;
    let second = &sections[1].projection;
    if first.definition_identity != second.definition_identity {
        return Err(error(
            "E_DDL_DEFINITION_IDENTITY_MISMATCH",
            second.declaration_span,
            "두 localized projection은 같은 DefinitionIdentity를 가리켜야 합니다",
        ));
    }
    if first.type_identity != second.type_identity {
        return Err(error(
            "E_DDL_TYPE_IDENTITY_MISMATCH",
            second.declaration_span,
            "두 localized projection은 같은 TypeIdentity를 가리켜야 합니다",
        ));
    }

    Ok(DdlDocument {
        extension,
        source_span: Span::new(0, source.len()),
        sections,
    })
}

fn parse_extension(file_path: &str) -> Result<DdlExtension, DdlError> {
    if file_path.ends_with(".ddl") {
        Ok(DdlExtension::Ddl)
    } else if file_path.ends_with(".옮김") {
        Ok(DdlExtension::Olgim)
    } else {
        Err(error(
            "E_DDL_EXTENSION_UNSUPPORTED",
            Span::new(0, 0),
            "DDL Wave 1은 .ddl 또는 .옮김 extension만 허용합니다",
        ))
    }
}

fn registry_entry<'a>(
    entries: &'a [DdlRegistryEntry],
    header: &str,
) -> Option<&'a DdlRegistryEntry> {
    entries.iter().find(|entry| entry.header == header)
}

#[derive(Debug, Clone, Copy)]
struct Header<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

/// Finds only raw top-level line-start headers.  Quoted, line-commented,
/// block-commented, and nested `#!` bytes are ordinary section content.
fn scan_headers(source: &str) -> Result<Vec<Header<'_>>, DdlError> {
    let mut headers = Vec::new();
    let mut offset = 0;
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escape = false;
    let mut in_block_comment = false;

    for line_with_end in source.split_inclusive('\n') {
        let line = line_with_end.strip_suffix('\n').unwrap_or(line_with_end);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let eligible = depth == 0 && !in_string && !in_block_comment;
        if eligible && line.starts_with("#!") {
            if offset != 0 && headers.is_empty() {
                return Err(error(
                    "E_DDL_FIRST_HEADER_NOT_BYTE_ZERO",
                    Span::new(offset, offset + line.len()),
                    "첫 DDL header는 raw byte zero에서 시작해야 합니다",
                ));
            }
            // Exact registry matching also rejects whitespace and trailing text.
            headers.push(Header {
                start: offset,
                end: offset + line.len(),
                text: line,
            });
        }
        advance_lexical_depth(
            line,
            &mut depth,
            &mut in_string,
            &mut escape,
            &mut in_block_comment,
        );
        offset += line_with_end.len();
    }

    if !source.is_empty() && !source.ends_with('\n') {
        // `split_inclusive` already included the final unterminated line.
    }
    if in_string || in_block_comment || depth != 0 {
        return Err(error(
            "E_DDL_SOURCE_UNBALANCED",
            Span::new(0, source.len()),
            "DDL section boundary를 판정할 수 없는 미닫힌 문자열·주석·블록이 있습니다",
        ));
    }
    if headers.first().map(|header| header.start) != Some(0) {
        return Err(error(
            "E_DDL_FIRST_HEADER_REQUIRED",
            Span::new(0, source.len().min(1)),
            "DDL 문서는 byte zero의 exact LanguageProfile header로 시작해야 합니다",
        ));
    }
    Ok(headers)
}

fn advance_lexical_depth(
    line: &str,
    depth: &mut i32,
    in_string: &mut bool,
    escape: &mut bool,
    in_block_comment: &mut bool,
) {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if *in_block_comment {
            if bytes[index..].starts_with(b"*/") {
                *in_block_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if *in_string {
            if *escape {
                *escape = false;
            } else if bytes[index] == b'\\' {
                *escape = true;
            } else if bytes[index] == b'"' {
                *in_string = false;
            }
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"//") {
            break;
        }
        if bytes[index..].starts_with(b"/*") {
            *in_block_comment = true;
            index += 2;
            continue;
        }
        match bytes[index] {
            b'"' => *in_string = true,
            b'{' => *depth += 1,
            b'}' => *depth -= 1,
            _ => {}
        }
        index += 1;
    }
}

fn parse_projection(
    content: &str,
    content_start: usize,
    entry: &DdlRegistryEntry,
) -> Result<ImmutablePublicValueProjection, DdlError> {
    let declaration = content.trim();
    if declaration.is_empty() {
        return Err(error(
            "E_DDL_PROJECTION_MISSING",
            Span::new(content_start, content_start + content.len()),
            "각 LanguageSection에는 immutable public value projection 하나가 필요합니다",
        ));
    }
    if content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
        != 1
    {
        return Err(error(
            "E_DDL_PROJECTION_COUNT",
            Span::new(content_start, content_start + content.len()),
            "Wave 1 section에는 projection 하나만 허용합니다",
        ));
    }
    let leading = content.find(declaration).unwrap_or(0);
    let declaration_span = Span::new(
        content_start + leading,
        content_start + leading + declaration.len(),
    );
    let exact = format!(
        "{}:{} :={{...}}.",
        entry.localized_name, entry.localized_type
    );
    if declaration != exact {
        return Err(error(
            "E_DDL_OMITTED_VALUE_SHAPE",
            declaration_span,
            "DDL-only immutable value 표기는 exact `이름:타입 :={...}.`이어야 합니다",
        ));
    }
    validate_declaration_tokens(declaration, declaration_span, entry)?;
    Ok(ImmutablePublicValueProjection {
        definition_identity: entry.definition_identity.to_owned(),
        type_identity: entry.type_identity.to_owned(),
        localized_name: entry.localized_name.to_owned(),
        localized_type: entry.localized_type.to_owned(),
        declaration_span,
        omitted_payload: OmittedPayload::ValueInitializer,
    })
}

/// Reuses the shared lexer for the Korean declaration shell.  The existing
/// lexer has no Japanese identifier admission, so the Owner-bound Japanese
/// Wave 1 surface is accepted only through its exact registry literal; this
/// is deliberately not generalized parsing or a fallback language mode.
fn validate_declaration_tokens(
    source: &str,
    span: Span,
    entry: &DdlRegistryEntry,
) -> Result<(), DdlError> {
    if entry.profile_identity == JAPANESE.profile_identity {
        return Ok(());
    }
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|lex| error("E_DDL_DECLARATION_LEX", span, lex.message))?;
    let expected = [
        matches!(&tokens[0].kind, TokenKind::Ident(_)),
        matches!(tokens[1].kind, TokenKind::Colon),
        matches!(&tokens[2].kind, TokenKind::Ident(_)),
        matches!(tokens[3].kind, TokenKind::Define),
        matches!(tokens[4].kind, TokenKind::LBrace),
        matches!(tokens[5].kind, TokenKind::DotDot),
        matches!(tokens[6].kind, TokenKind::Dot),
        matches!(tokens[7].kind, TokenKind::RBrace),
        matches!(tokens[8].kind, TokenKind::Dot),
        matches!(tokens[9].kind, TokenKind::Eof),
    ];
    if tokens.len() != expected.len() || expected.iter().any(|valid| !valid) {
        return Err(error(
            "E_DDL_OMITTED_VALUE_TOKENS",
            span,
            "DDL omission은 exact ASCII `:={...}` token sequence여야 합니다",
        ));
    }
    Ok(())
}

fn error(code: &'static str, span: Span, message: impl Into<String>) -> DdlError {
    DdlError {
        code,
        span,
        message: message.into(),
    }
}
