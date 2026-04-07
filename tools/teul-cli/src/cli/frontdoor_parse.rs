use crate::cli::frontdoor_input::{
    prepare_frontdoor_runtime_source, validate_no_legacy_frontdoor_surface,
};
use crate::lang::ast::Program;
use crate::lang::lexer::{LexError, Lexer};
use crate::lang::parser::{ParseError, ParseMode, Parser};

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
    Ok((program, prepared))
}

#[cfg(test)]
mod tests {
    use super::{parse_program_for_runtime, FrontdoorParseFailure};

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
}
