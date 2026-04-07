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
            let (line, col, snippet) = locate_span(&parity_source, primary_err.span.start);
            let blocker = classify_lang_parser_gap(&snippet, &primary_err.to_string());
            Err(format!(
                "E_FRONTDOOR_LANG_PARSER_GAP lang_code={} blocked_by={} line={} col={} near={} detail={} wrapped_code={} wrapped_detail={}",
                primary_err.code(),
                blocker,
                line,
                col,
                snippet,
                primary_err,
                wrapped_err.code(),
                wrapped_err
            ))
        }
    }
}

fn normalize_for_lang_parity(source: &str) -> String {
    let without_maegim = strip_decl_item_maegim_suffix_for_lang_parity(source);
    strip_hook_colon_before_block_for_lang_parity(&without_maegim)
}

fn wrap_lang_parity_source(source: &str) -> String {
    format!("프론트도어_패리티_검사:움직씨 = {{\n{source}\n}}\n")
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

fn strip_decl_item_maegim_suffix_for_lang_parity(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    let mut in_string = false;
    let mut escape = false;
    while i < chars.len() {
        let ch = chars[i];
        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            i += 1;
            continue;
        }

        if (keyword_at(&chars, i, "매김") || keyword_at(&chars, i, "조건"))
            && prev_non_ws_char(&chars, i) == Some(')')
        {
            let mut j = i + 2; // Korean chars are one char each in Vec<char>
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && chars[j] == ':' {
                j += 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
            }
            if j < chars.len() && chars[j] == '{' {
                let mut depth = 1usize;
                j += 1;
                while j < chars.len() && depth > 0 {
                    match chars[j] {
                        '{' => depth += 1,
                        '}' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                // Optional trailing spaces after removed block.
                while j < chars.len() && chars[j].is_whitespace() && chars[j] != '\n' {
                    j += 1;
                }
                i = j;
                continue;
            }
        }

        out.push(ch);
        i += 1;
    }
    out
}

fn keyword_at(chars: &[char], idx: usize, keyword: &str) -> bool {
    let mut j = idx;
    for kc in keyword.chars() {
        if chars.get(j).copied() != Some(kc) {
            return false;
        }
        j += 1;
    }
    let prev_ok = idx == 0
        || chars
            .get(idx - 1)
            .copied()
            .is_some_and(|c| c.is_whitespace() || matches!(c, '{' | '}' | '(' | ')' | '.' | ':'));
    let next_ok = j >= chars.len()
        || chars
            .get(j)
            .copied()
            .is_some_and(|c| c.is_whitespace() || matches!(c, '{' | ':' | '.'));
    prev_ok && next_ok
}

fn prev_non_ws_char(chars: &[char], idx: usize) -> Option<char> {
    if idx == 0 {
        return None;
    }
    let mut j = idx;
    while j > 0 {
        j -= 1;
        let c = chars[j];
        if !c.is_whitespace() {
            return Some(c);
        }
    }
    None
}

fn strip_hook_colon_before_block_for_lang_parity(source: &str) -> String {
    const KEYWORDS: &[&str] = &["할때", "마다", "될때", "동안"];
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    let mut in_string = false;
    let mut escape = false;
    while i < chars.len() {
        let ch = chars[i];
        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            i += 1;
            continue;
        }

        let mut removed_colon = false;
        for kw in KEYWORDS {
            if !keyword_at(&chars, i, kw) {
                continue;
            }
            let kw_len = kw.chars().count();
            let kw_end = i + kw_len;
            let mut j = kw_end;
            while j < chars.len() && chars[j].is_whitespace() && chars[j] != '\n' {
                j += 1;
            }
            if j < chars.len() && chars[j] == ':' {
                let mut k = j + 1;
                while k < chars.len() && chars[k].is_whitespace() && chars[k] != '\n' {
                    k += 1;
                }
                if k < chars.len() && chars[k] == '{' {
                    for c in &chars[i..j] {
                        out.push(*c);
                    }
                    i = j + 1; // drop colon only
                    removed_colon = true;
                    break;
                }
            }
        }
        if removed_colon {
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
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
        parse_program_for_runtime, strip_decl_item_maegim_suffix_for_lang_parity,
        validate_lang_frontdoor_parity, FrontdoorParseFailure,
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
    fn strip_decl_item_maegim_suffix_removes_inline_block() {
        let src = "채비 { g: 수 <- (9.8) 매김 { 범위(0..10). 간격(1). }. }.";
        let out = strip_decl_item_maegim_suffix_for_lang_parity(src);
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
}
