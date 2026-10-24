//! Offline migration utility, never a runtime frontdoor or an input-tape reader.
//! Input epoch is mandatory for every row; parser success never selects it.
use ddonirang_lang::{Lexer, TokenKind};
use ddonirang_tool::ddn_runtime::DdnProgram;
use serde_json::{json, Value};
use std::io::{self, BufRead};

fn migrate(source: &str, path: &str, epoch: &str) -> Result<String, String> {
    if source.starts_with('\u{feff}') {
        return Err("E_SOURCE_UTF8_BOM_FORBIDDEN: BOM을 제거하지 않습니다".into());
    }
    match epoch {
        "v25" => {
            DdnProgram::from_v25_source(source, path)?;
            Ok(source.to_string())
        }
        "pre-v25" => {
            // Preserve all bytes except explicitly selected syntax token spans.
            // Reprinting a legacy AST with N1 loses decimals/parentheses and
            // controls; it must never be a source migration transport.
            let candidate = migrate_tokens(source)?;
            // Never emit an unadmitted guessed successor. Runtime intent is
            // still verified separately by the corpus execution/review stage.
            DdnProgram::from_v25_source(&candidate, path)?;
            Ok(candidate)
        }
        _ => Err("E_MIGRATION_INPUT_EPOCH_REQUIRED: 입력 글판을 명시하세요".into()),
    }
}

fn migrate_tokens(source: &str) -> Result<String, String> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|error| error.to_string())?;
    let mut edits: Vec<(usize, usize, &str)> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind == TokenKind::Equals
            && index > 0
            && matches!(
                tokens[index - 1].kind,
                TokenKind::KwUmjikssi
                    | TokenKind::KwValueFunc
                    | TokenKind::KwImeumssi
                    | TokenKind::KwGallaessi
                    | TokenKind::KwRelationssi
                    | TokenKind::KwSemssi
            )
        {
            edits.push((token.span.start, token.span.end, ":="));
        }
        if token.kind == TokenKind::KwManyak {
            let mut depth = 0i32;
            let end = (index + 1..tokens.len())
                .find(|&next| {
                    match tokens[next].kind {
                        TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
                        TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => depth -= 1,
                        _ => {}
                    }
                    depth == 0 && tokens[next].kind == TokenKind::KwIramyeon
                })
                .ok_or("E_MIGRATION_IF_AMBIGUOUS: 조건 끝을 유일하게 찾을 수 없습니다")?;
            edits.push((token.span.start, token.span.end, "{"));
            edits.push((tokens[end].span.start, tokens[end].span.end, "}이면"));
            if tokens
                .get(end + 1)
                .is_some_and(|next| next.kind == TokenKind::Colon)
            {
                edits.push((tokens[end + 1].span.start, tokens[end + 1].span.end, ""));
            }
        }
        if token.kind == TokenKind::RBrace {
            let next = tokens.get(index + 1);
            let after = tokens.get(index + 2);
            if next.is_some_and(|next| next.raw == "인것") {
                if after.is_some_and(|after| {
                    matches!(
                        after.kind,
                        TokenKind::KwIlttae | TokenKind::KwIramyeon | TokenKind::KwMajeumyeon
                    )
                }) {
                    let after = after.unwrap();
                    edits.push((next.unwrap().span.start, next.unwrap().span.end, ""));
                    edits.push((after.span.start, after.span.end, "이면"));
                    if tokens
                        .get(index + 3)
                        .is_some_and(|next| next.kind == TokenKind::Colon)
                    {
                        edits.push((tokens[index + 3].span.start, tokens[index + 3].span.end, ""));
                    }
                } else if after.is_some_and(|after| after.kind == TokenKind::KwDongan) {
                    edits.push((next.unwrap().span.start, next.unwrap().span.end, "인"));
                    if tokens
                        .get(index + 3)
                        .is_some_and(|next| next.kind == TokenKind::Colon)
                    {
                        edits.push((tokens[index + 3].span.start, tokens[index + 3].span.end, ""));
                    }
                }
            }
        }
        if token.raw == "채비" {
            let mut open = index + 1;
            if tokens
                .get(open)
                .is_some_and(|next| next.kind == TokenKind::Colon)
            {
                edits.push((tokens[open].span.start, tokens[open].span.end, ""));
                open += 1;
            }
            if tokens
                .get(open)
                .is_some_and(|next| next.kind == TokenKind::LBrace)
            {
                let mut depth = 1i32;
                for next in tokens.iter().skip(open + 1) {
                    match next.kind {
                        TokenKind::LBrace => depth += 1,
                        TokenKind::RBrace => depth -= 1,
                        TokenKind::Arrow if depth == 1 => {
                            edits.push((next.span.start, next.span.end, ":="))
                        }
                        _ => {}
                    }
                    if depth == 0 {
                        break;
                    }
                }
            }
        }
    }
    edits.sort_by_key(|edit| edit.0);
    if edits.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err("E_MIGRATION_EDIT_AMBIGUOUS: 겹치는 변환은 거부합니다".into());
    }
    let mut output = source.to_string();
    for (start, end, replacement) in edits.into_iter().rev() {
        output.replace_range(start..end, replacement);
    }
    Ok(output)
}

fn main() {
    for line in io::stdin().lock().lines() {
        let result = line.map_err(|error| error.to_string()).and_then(|line| {
            let row: Value = serde_json::from_str(&line).map_err(|error| error.to_string())?;
            let path = row["path"].as_str().unwrap_or("<migration>");
            let source = row["source"]
                .as_str()
                .ok_or("E_MIGRATION_SOURCE_REQUIRED")?;
            Ok(
                match migrate(source, path, row["input_epoch"].as_str().unwrap_or("")) {
                    Ok(output) => json!({"path": path, "ok": true, "output": output}),
                    Err(error) => json!({"path": path, "ok": false, "error": error}),
                },
            )
        });
        println!(
            "{}",
            result.unwrap_or_else(|error| json!({"ok":false,"error":error}))
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_epoch_is_never_inferred() {
        assert!(migrate("값 <- 1.", "test", "")
            .unwrap_err()
            .contains("INPUT_EPOCH_REQUIRED"));
        assert!(migrate("값 <- 1.", "test", "unknown").is_err());
    }

    #[test]
    fn already_current_source_is_byte_preserved() {
        let source = "// 만약 참 이라면\n값 <- \"인것 일때\".\n값 보여주기.\n";
        assert_eq!(migrate(source, "test", "v25").unwrap(), source);
    }

    #[test]
    fn legacy_tokens_move_condition_without_rewriting_text_values() {
        let source = "매마디:움직씨 = { 만약 참 이라면 { \"만약\" 보여주기. } }";
        let output = migrate(source, "test", "pre-v25").unwrap();
        assert!(output.contains("이면"), "{output}");
        assert!(output.contains("\"만약\""));
        assert!(output.contains(":="));
        assert_eq!(migrate(source, "test", "pre-v25").unwrap(), output);
    }

    #[test]
    fn unknown_surfaces_and_bom_fail_without_guesses() {
        assert!(migrate("보임 { x: 1. }.", "test", "pre-v25").is_err());
        assert!(migrate("\u{feff}값 <- 1.", "test", "v25")
            .unwrap_err()
            .contains("BOM_FORBIDDEN"));
    }

    #[test]
    fn source_projection_preserves_q32_and_calculation_grouping() {
        let source = "매마디:움직씨 = { 값 <- 0.1. 결과 <- (1 + 2) * 3. 결과 보여주기. }";
        let output = migrate(source, "test", "pre-v25").unwrap();
        assert!(output.contains("값 <- 0.1."), "{output}");
        assert!(output.contains("(1 + 2) * 3"), "{output}");
    }

    #[test]
    fn comments_between_condition_tokens_are_preserved() {
        let source = "{ 참 }인것 // 의미 보존\n 일때 { 1 보여주기. }.";
        let output = migrate(source, "test", "pre-v25").unwrap();
        assert!(output.contains("// 의미 보존"));
    }
}
