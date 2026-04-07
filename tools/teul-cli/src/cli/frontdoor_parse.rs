use crate::cli::frontdoor_input::{
    prepare_frontdoor_runtime_source, validate_no_legacy_frontdoor_surface,
};
use crate::lang::ast::Program;
use crate::lang::lexer::{LexError, Lexer};
use crate::lang::parser::{ParseError, ParseMode, Parser};
use ddonirang_lang::{
    normalize_for_lang_parity as lang_normalize_for_parity, parse_with_mode as lang_parse_with_mode,
    wrap_lang_parity_source as lang_wrap_parity_source, ParseMode as LangParseMode,
};

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
    let parity_source = normalize_for_lang_parity(prepared_source);
    match lang_parse_with_mode(
        &parity_source,
        "<teul-frontdoor-acceptance>",
        LangParseMode::Strict,
    ) {
        Ok(_) => Ok(()),
        Err(primary_err) => {
            // lang parser is seed-oriented; preflight allows top-level runtime statements
            // by checking a wrapped synthetic seed body as a parity fallback.
            let wrapped = wrap_lang_parity_source(&parity_source);
            let wrapped_result = lang_parse_with_mode(
                &wrapped,
                "<teul-frontdoor-acceptance-wrapped>",
                LangParseMode::Strict,
            );
            if wrapped_result.is_ok() {
                return Ok(());
            }
            let wrapped_err = wrapped_result.expect_err("wrapped_result is already known as Err");
            let (wline, wcol, wsnippet) = locate_span(&wrapped, wrapped_err.span.start);
            let (line, col, snippet) = locate_span(&parity_source, primary_err.span.start);
            let blocker = classify_lang_parser_gap(&snippet, &primary_err.to_string());
            Err(format!(
                "E_FRONTDOOR_LANG_PARSER_GAP lang_code={} blocked_by={} line={} col={} near={} detail={} wrapped_code={} wrapped_line={} wrapped_col={} wrapped_near={} wrapped_detail={}",
                primary_err.code(),
                blocker,
                line,
                col,
                snippet,
                primary_err,
                wrapped_err.code(),
                wline,
                wcol,
                wsnippet,
                wrapped_err
            ))
        }
    }
}

fn normalize_for_lang_parity(source: &str) -> String {
    lang_normalize_for_parity(source)
}

fn wrap_lang_parity_source(source: &str) -> String {
    lang_wrap_parity_source(source)
}

fn classify_lang_parser_gap(snippet: &str, detail: &str) -> &'static str {
    if snippet.contains(" 매김 {")
        || snippet.contains(" 조건 {")
        || detail.contains("매김")
        || detail.contains("조건")
    {
        return "blocked_by_control_meta_relocation";
    }
    "blocked_by_frontdoor_parser_gap"
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
    use super::{
        lang_parse_with_mode, normalize_for_lang_parity, parse_program_for_runtime,
        validate_lang_frontdoor_parity, wrap_lang_parity_source, FrontdoorParseFailure,
        LangParseMode,
    };

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
        assert!(err.contains("blocked_by=blocked_by_frontdoor_parser_gap"));
        assert!(err.contains("line=1"));
        assert!(err.contains("col="));
        assert!(err.contains("near=x <- ."));
    }

    #[test]
    fn validate_lang_frontdoor_parity_accepts_control_meta_line_after_strip() {
        let source = "채비 { g: 수 <- (9.8) 매김 { 범위(0..10). }. }.";
        validate_lang_frontdoor_parity(source).expect("must pass after strip");
    }

    #[test]
    fn normalize_for_lang_parity_removes_inline_maegim_block() {
        let src = "채비 { g: 수 <- (9.8) 매김 { 범위(0..10). 간격(1). }. }.";
        let out = normalize_for_lang_parity(src);
        assert!(out.contains("g: 수 <- (9.8) ."));
        assert!(!out.contains("매김 {"));
    }

    #[test]
    fn validate_lang_frontdoor_parity_accepts_decl_item_after_strip() {
        let source = "채비 { g: 수 <- (9.8) 매김 { 범위(0..10). 간격(1). }. }.";
        validate_lang_frontdoor_parity(source).expect("must pass with preflight strip");
    }

    #[test]
    fn validate_lang_frontdoor_parity_accepts_top_level_hook_colon_variant() {
        let source = "(시작)할때: { n <- 1. }.";
        validate_lang_frontdoor_parity(source).expect("must pass with hook-colon strip");
    }

    #[test]
    fn validate_lang_frontdoor_parity_accepts_hook_with_mul_and_call() {
        let source = r#"
(시작)할때 {
  각도 <- 45.
  cos_a <- (각도) cos.
  vx <- 20 * cos_a.
}.
"#;
        validate_lang_frontdoor_parity(source).expect("must pass hook mul/call pattern");
    }

    #[test]
    fn validate_lang_frontdoor_parity_accepts_three_arg_bound_call_range_after_rewrite() {
        let source = "n목록 <- (n_min, n_max, dn) 범위.";
        validate_lang_frontdoor_parity(source).expect("must pass after range rewrite");
    }

    #[test]
    fn normalize_for_lang_parity_rewrites_three_arg_range_bound_call() {
        let source = "n목록 <- (n_min, n_max, dn) 범위.";
        let rewritten = normalize_for_lang_parity(source);
        assert_eq!(rewritten, "n목록 <- (n_min .. n_max).");
    }

    #[test]
    fn validate_lang_frontdoor_parity_accepts_simple_assignment() {
        validate_lang_frontdoor_parity("x <- 1.").expect("simple assignment must pass");
    }

    #[test]
    fn normalize_for_lang_parity_keeps_wave1_projectile_core_lines() {
        let source = include_str!("../../../../docs/ssot/pack/edu_phys_p001_05_projectile_xy/lesson.ddn");
        let parity = normalize_for_lang_parity(source);
        let wrapped = wrap_lang_parity_source(&parity);
        assert!(wrapped.contains("채비 {"));
        assert!(wrapped.contains("vx <- 초기속도 * cos_a."));
        assert!(!wrapped.contains("매김 {"));
    }

    #[test]
    fn validate_lang_frontdoor_parity_accepts_wave1_projectile_wrapped_source() {
        let source = include_str!("../../../../docs/ssot/pack/edu_phys_p001_05_projectile_xy/lesson.ddn");
        let parity = normalize_for_lang_parity(source);
        let wrapped = wrap_lang_parity_source(&parity);
        lang_parse_with_mode(&wrapped, "<wave1-projectile-parity>", LangParseMode::Strict)
            .expect("wrapped projectile parity source must parse in lang");
    }
}
