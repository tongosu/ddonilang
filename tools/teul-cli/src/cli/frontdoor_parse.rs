use crate::cli::frontdoor_input::{
    prepare_frontdoor_runtime_source, validate_no_legacy_frontdoor_surface,
};
use crate::lang::ast::Program;
use crate::lang::lexer::{LexError, Lexer};
use crate::lang::parser::{ParseError, ParseMode, Parser};
use ddonirang_lang::{parse_with_mode as lang_parse_with_mode, ParseMode as LangParseMode};

#[derive(Debug)]
pub enum FrontdoorParseFailure {
    Guard(String),
    Lex(LexError),
    Parse(ParseError),
}

pub fn parse_program_for_runtime(source: &str) -> Result<(Program, String), FrontdoorParseFailure> {
    parse_program_for_runtime_with_mode(source, ParseMode::Strict)
}

pub fn parse_program_for_runtime_with_mode(
    source: &str,
    parse_mode: ParseMode,
) -> Result<(Program, String), FrontdoorParseFailure> {
    validate_no_legacy_frontdoor_surface(source).map_err(FrontdoorParseFailure::Guard)?;
    let prepared = prepare_frontdoor_runtime_source(source);
    let tokens = Lexer::tokenize(&prepared).map_err(FrontdoorParseFailure::Lex)?;
    let default_root = Parser::default_root_for_source(&prepared);
    let program = Parser::parse_with_default_root_mode(tokens, default_root, parse_mode)
        .map_err(FrontdoorParseFailure::Parse)?;
    if should_enforce_lang_frontdoor_parity() {
        validate_lang_frontdoor_parity(&prepared).map_err(FrontdoorParseFailure::Guard)?;
    }
    Ok((program, prepared))
}

fn should_enforce_lang_frontdoor_parity() -> bool {
    std::env::var("DDN_LANG_FRONTDOOR_PARITY_ENFORCE")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn validate_lang_frontdoor_parity(prepared_source: &str) -> Result<(), String> {
    lang_parse_with_mode(
        prepared_source,
        "<teul-frontdoor-acceptance>",
        LangParseMode::Strict,
    )
    .map(|_| ())
    .map_err(|err| {
        let (line, col, snippet) = locate_span(prepared_source, err.span.start);
        format!(
            "E_FRONTDOOR_LANG_PARSER_GAP lang_code={} line={} col={} near={} detail={}",
            err.code(),
            line,
            col,
            snippet,
            err
        )
    })
}

fn locate_span(source: &str, byte_pos: usize) -> (usize, usize, String) {
    let clamped = byte_pos.min(source.len());
    let mut line_no = 1usize;
    let mut line_start = 0usize;
    for (idx, ch) in source.char_indices() {
        if idx >= clamped {
            break;
        }
        if ch == '\n' {
            line_no += 1;
            line_start = idx + ch.len_utf8();
        }
    }
    let line_end = source[line_start..]
        .find('\n')
        .map(|off| line_start + off)
        .unwrap_or(source.len());
    let line_slice = &source[line_start..line_end];
    let col = source[line_start..clamped].chars().count() + 1;
    let snippet = line_slice.trim().chars().take(120).collect::<String>();
    (line_no, col, snippet)
}

#[cfg(test)]
mod tests {
    use super::{parse_program_for_runtime, validate_lang_frontdoor_parity, FrontdoorParseFailure};

    #[test]
    fn parse_runtime_rejects_legacy_hash_header() {
        let source = "#이름: 금지\n(매마디)마다 { n <- 1. }.";
        let err = parse_program_for_runtime(source).expect_err("must reject");
        match err {
            FrontdoorParseFailure::Guard(message) => {
                assert!(message.contains("E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN"));
            }
            other => panic!("unexpected error variant: {:?}", other),
        }
    }

    #[test]
    fn parse_runtime_rejects_legacy_boim_surface() {
        let source = "보임 { x: 1. }.";
        let err = parse_program_for_runtime(source).expect_err("must reject");
        match err {
            FrontdoorParseFailure::Guard(message) => {
                assert!(message.contains("E_CANON_LEGACY_BOIM_FORBIDDEN"));
            }
            other => panic!("unexpected error variant: {:?}", other),
        }
    }

    #[test]
    fn validate_lang_frontdoor_parity_reports_gap_code_on_parse_error() {
        let err = validate_lang_frontdoor_parity("x <- .").expect_err("must fail");
        assert!(err.starts_with("E_FRONTDOOR_LANG_PARSER_GAP"));
        assert!(err.contains("line=1"));
        assert!(err.contains("col="));
        assert!(err.contains("near=x <- ."));
    }
}
