use ddonirang_lang::dialect::DialectConfig;
use ddonirang_lang::lexer::{Lexer, TokenKind, CANONICAL_LEXER_KEYWORDS};
use std::collections::HashSet;

fn is_keyword(kind: &TokenKind) -> bool {
    matches!(kind,
        TokenKind::KwImeumssi | TokenKind::KwUmjikssi | TokenKind::KwSemssi
        | TokenKind::KwIeumssi | TokenKind::KwHeureumssi | TokenKind::KwGallaessi
        | TokenKind::KwRelationssi | TokenKind::KwSam | TokenKind::And | TokenKind::Or
        | TokenKind::KwManyak | TokenKind::KwIramyeon | TokenKind::KwIlttae
        | TokenKind::KwAniramyeon | TokenKind::KwBanbok | TokenKind::KwDongan
        | TokenKind::KwDaehae | TokenKind::KwMeomchugi | TokenKind::KwGeonneottwigi
        | TokenKind::KwDollyeojwo | TokenKind::KwTolabogi | TokenKind::KwHaebogo
        | TokenKind::KwGoreugi | TokenKind::KwMajeumyeon | TokenKind::KwJeonjehae
        | TokenKind::KwBojanghago | TokenKind::KwHaeseo | TokenKind::KwNeuljikeobogo
        | TokenKind::KwBoyeojugi | TokenKind::KwBeat | TokenKind::KwReserve)
}

#[test]
fn published_spellings_are_unique_canonical_real_tokens_with_utf8_spans() {
    let dialect = DialectConfig::from_source("");
    let mut unique = HashSet::new();
    for &word in CANONICAL_LEXER_KEYWORDS {
        assert!(unique.insert(word), "duplicate: {word}");
        if let Some(canonical) = dialect.canonicalize_keyword(word) {
            assert_eq!(canonical, word, "alias published: {word}");
        }
        let tokens = Lexer::new(word).tokenize().unwrap();
        assert_eq!(tokens.len(), 2, "not one token: {word}");
        assert!(is_keyword(&tokens[0].kind), "not a lexer keyword: {word}");
        assert_eq!(tokens[0].raw, word);
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, word.len());
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }
}

#[test]
fn dialect_canonical_keyword_rows_are_not_omitted() {
    for row in include_str!("../src/dialect_keywords_v8.tsv").lines().skip(1) {
        let word = row.split('\t').nth(1).unwrap_or("").trim().trim_end_matches(':');
        if let Ok(tokens) = Lexer::new(word).tokenize() {
            if tokens.len() == 2 && is_keyword(&tokens[0].kind) && tokens[0].raw == word {
                assert!(CANONICAL_LEXER_KEYWORDS.contains(&word), "missing: {word}");
            }
        }
    }
}

#[test]
fn aliases_remain_lexable_without_becoming_completion_recommendations() {
    for (alias, canonical) in [
        ("반복", "되풀이"), ("반환", "되돌림"), ("관계씨", "맞물림씨"),
        ("감사", "톺아보기"), ("전제하에", "바탕으로"), ("보장하고", "다짐하고"),
        ("아니라면", "아니면"), ("돌려줘", "되돌림"),
    ] {
        assert!(!CANONICAL_LEXER_KEYWORDS.contains(&alias));
        let actual = Lexer::new(alias).tokenize().unwrap();
        let expected = Lexer::new(canonical).tokenize().unwrap();
        assert_eq!(actual[0].kind, expected[0].kind, "alias behavior changed: {alias}");
    }
    for removed in ["값함수", "일묶음씨", "유지하고", "검사할때"] {
        assert!(!CANONICAL_LEXER_KEYWORDS.contains(&removed));
    }
}

#[test]
fn external_consumer_can_supply_missing_show_keyword_without_stdlib_names() {
    let candidates: Vec<_> = CANONICAL_LEXER_KEYWORDS.iter().copied()
        .filter(|word| word.starts_with("보여")).collect();
    assert_eq!(candidates, ["보여주기"]);
    assert!(CANONICAL_LEXER_KEYWORDS.contains(&"맞으면"));
    assert!(CANONICAL_LEXER_KEYWORDS.contains(&"이면"));
    assert!(CANONICAL_LEXER_KEYWORDS.contains(&"갈래씨"));
    assert!(CANONICAL_LEXER_KEYWORDS.contains(&"고름씨"));
    assert!(!CANONICAL_LEXER_KEYWORDS.contains(&"합계"));
}
