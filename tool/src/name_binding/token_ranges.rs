//! A2-1 syntax ranges for one exact, accepted source snapshot.
//!
//! These ranges are not definition bindings, rename permission, saved identity,
//! executable admission or proof evidence. A1 remains responsible for its own
//! existing Exact/Unsupported decision. In particular, lexical `...하기` is a
//! whole token here; do not invent a callee stem or a shadowing rule from it.
use super::{declaration_token, ByteRange, Facts};
use crate::ddn_runtime::{resolve_supported_source_profile, DdnProgram};
use crate::source_analysis::{analyze_source, SourceAnalysisRequest};
use ddonirang_lang::{Lexer, NodeId, ParseMode, Span, Token, TokenKind, TopLevelItem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameTokenOutcome {
    Available,
    SourceRejected,
    Unavailable(NameTokenIssue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameTokenIssue {
    ProductAnalysisUnavailable,
    OriginSourceMismatch,
    InvalidLexerRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationSyntaxKind {
    Seed,
    ParameterPin,
    StatementBinder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationRangeIssue {
    InvalidSyntaxRange,
    NoUniqueHeaderToken,
    NoLeadingNameToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclarationNameRange {
    Located(ByteRange),
    Unavailable(DeclarationRangeIssue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNameToken {
    /// Exact shared-lexer spelling, not normalized or stripped of a call tail.
    pub spelling: String,
    /// Half-open UTF-8 bytes of this whole identifier token.
    pub range: ByteRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxDeclaration {
    pub kind: DeclarationSyntaxKind,
    pub name: String,
    /// Existing AST syntax span; never replaced by the finer name span.
    pub syntax_range: ByteRange,
    /// Compiler-local anchor only. It is not a resolved or saved identity.
    pub node_id: Option<NodeId>,
    pub name_range: DeclarationNameRange,
}

/// No wire/Serialize and no public constructor. Each record is produced from
/// the actual common frontdoor and lexer of the same source bytes.
#[derive(Debug, Clone)]
pub struct NameTokenReport {
    source_revision_identity: String,
    source_digest: [u8; 32],
    outcome: NameTokenOutcome,
    tokens: Vec<SyntaxNameToken>,
    declarations: Vec<SyntaxDeclaration>,
}

impl NameTokenReport {
    pub fn source_revision_identity(&self) -> &str { &self.source_revision_identity }
    pub fn outcome(&self) -> &NameTokenOutcome { &self.outcome }
    pub fn tokens(&self) -> &[SyntaxNameToken] { &self.tokens }
    pub fn declarations(&self) -> &[SyntaxDeclaration] { &self.declarations }
    pub fn is_for_source(&self, source: &str) -> bool {
        self.source_digest == *blake3::hash(source.as_bytes()).as_bytes()
    }

    /// UTF-8 byte offset, half-open and on a character boundary. Consumers
    /// must also check the current source revision before using this result.
    pub fn token_at(&self, offset: usize) -> Option<&SyntaxNameToken> {
        self.tokens.iter().find(|token| {
            token.range.start <= offset && offset < token.range.end
                && token.spelling.is_char_boundary(offset - token.range.start)
        })
    }
}

fn valid_span(source: &str, span: Span) -> bool {
    span.start <= span.end && source.get(span.start..span.end).is_some()
}

fn leading_token(name: &str, span: Span, tokens: &[Token]) -> DeclarationNameRange {
    // The AST owns the declaration start. Do not search its body/default for
    // a later same-spelling token when this position cannot be established.
    let token = tokens.iter().find(|token| token.span.start == span.start
        && token.span.end <= span.end && token.raw == name
        && matches!(token.kind, TokenKind::Ident(_)));
    match token {
        Some(token) => DeclarationNameRange::Located(ByteRange {
            start: token.span.start, end: token.span.end,
        }),
        None => DeclarationNameRange::Unavailable(DeclarationRangeIssue::NoLeadingNameToken),
    }
}

pub fn analyze_name_tokens(source: &str, path: &str) -> NameTokenReport {
    let checked = analyze_source(SourceAnalysisRequest {
        source_text: source.to_string(), path: path.to_string(), analysis_generation: 0,
        source_profile_identity: "v1-core-v25".to_string(),
    });
    let mut report = NameTokenReport {
        source_revision_identity: checked.source_revision_identity,
        source_digest: *blake3::hash(source.as_bytes()).as_bytes(),
        outcome: NameTokenOutcome::SourceRejected,
        tokens: Vec::new(), declarations: Vec::new(),
    };
    if checked.outcome != "ACCEPTED" { return report; }
    report.outcome = NameTokenOutcome::Unavailable(NameTokenIssue::ProductAnalysisUnavailable);
    let Ok(profile) = resolve_supported_source_profile(Some("v1-core-v25")) else { return report; };
    let Ok(program) = DdnProgram::from_supported_source_profile_with_mode(
        source, path, ParseMode::Strict, profile,
    ) else { return report; };
    let canon = program.canonical_program();
    if canon.origin.source != source {
        report.outcome = NameTokenOutcome::Unavailable(NameTokenIssue::OriginSourceMismatch);
        return report;
    }
    let Ok(tokens) = Lexer::new(source).tokenize() else { return report; };
    let name_tokens: Vec<_> = tokens.iter().filter(|t| matches!(t.kind, TokenKind::Ident(_))).collect();
    let mut previous_end = 0;
    for token in &name_tokens {
        if !valid_span(source, Span::new(token.span.start, token.span.end)) || token.span.start == token.span.end
            || token.span.start < previous_end
            || source.get(token.span.start..token.span.end) != Some(token.raw.as_str()) {
            report.outcome = NameTokenOutcome::Unavailable(NameTokenIssue::InvalidLexerRange);
            return report;
        }
        previous_end = token.span.end;
    }
    report.tokens = name_tokens.iter().map(|token| SyntaxNameToken {
        spelling: token.raw.clone(), range: ByteRange {start:token.span.start,end:token.span.end},
    }).collect();
    let mut facts = Facts::default();
    for item in &canon.items {
        let TopLevelItem::SeedDef(seed) = item;
        let name_range = if !valid_span(source, seed.span) {
            DeclarationNameRange::Unavailable(DeclarationRangeIssue::InvalidSyntaxRange)
        } else {
            declaration_token(seed, &tokens).map(|token| DeclarationNameRange::Located(ByteRange {
                start:token.span.start,end:token.span.end,
            })).unwrap_or(DeclarationNameRange::Unavailable(DeclarationRangeIssue::NoUniqueHeaderToken))
        };
        report.declarations.push(SyntaxDeclaration {
            kind:DeclarationSyntaxKind::Seed,name:seed.canonical_name.clone(),
            syntax_range:ByteRange {start:seed.span.start,end:seed.span.end},node_id:Some(seed.id),name_range,
        });
        for param in &seed.params {
            let name_range = if valid_span(source,param.span) {
                leading_token(&param.pin_name,param.span,&tokens)
            } else {DeclarationNameRange::Unavailable(DeclarationRangeIssue::InvalidSyntaxRange)};
            report.declarations.push(SyntaxDeclaration {
                kind:DeclarationSyntaxKind::ParameterPin,name:param.pin_name.clone(),
                syntax_range:ByteRange {start:param.span.start,end:param.span.end},node_id:Some(param.id),name_range,
            });
            if let Some(value) = &param.default_value { facts.expr(value); }
        }
        if let Some(body) = &seed.body { facts.body(body); }
    }
    for statement in &canon.root_stmts { facts.stmt(statement); }
    for (name,span) in facts.binders {
        let name_range = if valid_span(source,span) {leading_token(&name,span,&tokens)}
            else {DeclarationNameRange::Unavailable(DeclarationRangeIssue::InvalidSyntaxRange)};
        report.declarations.push(SyntaxDeclaration {
            kind:DeclarationSyntaxKind::StatementBinder,name,
            syntax_range:ByteRange {start:span.start,end:span.end},node_id:None,name_range,
        });
    }
    report.outcome = NameTokenOutcome::Available;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_same_spelling_search_inside_a_binder_span() {
        let source="앞 := 이름.";
        let tokens=Lexer::new(source).tokenize().unwrap();
        assert_eq!(leading_token("이름",Span::new(0,source.len()),&tokens),
            DeclarationNameRange::Unavailable(DeclarationRangeIssue::NoLeadingNameToken));
    }
    #[test]
    fn malformed_utf8_or_out_of_source_syntax_span_is_not_a_name_range() {
        let source="이름 := 1.";
        assert!(!valid_span(source,Span::new(1,3)));
        assert!(!valid_span(source,Span::new(0,source.len()+1)));
        assert!(!valid_span(source,Span::new(5,1)));
    }
}
