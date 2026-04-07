use crate::cli::frontdoor_input::prepare_frontdoor_runtime_source;
use crate::lang::ast::Program;
use crate::lang::lexer::{LexError, Lexer};
use crate::lang::parser::{ParseError, ParseMode, Parser};

#[derive(Debug)]
pub enum FrontdoorParseFailure {
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
    let prepared = prepare_frontdoor_runtime_source(source);
    let tokens = Lexer::tokenize(&prepared).map_err(FrontdoorParseFailure::Lex)?;
    let default_root = Parser::default_root_for_source(&prepared);
    let program = Parser::parse_with_default_root_mode(tokens, default_root, parse_mode)
        .map_err(FrontdoorParseFailure::Parse)?;
    Ok((program, prepared))
}
