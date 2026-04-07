// ddonirang-lang/src/lib.rs
// 또니랑 언어 코어 라이브러리
//
// Phase 1 완료:
// - Lexer: 한국어 토큰화
// - Parser: SOV 어순 재귀 하강 파서
// - Normalizer: N1 레벨 정본화
// - AST: 비손실 정본 구조

pub mod age_gate;
pub mod ast;
pub mod canonicalizer;
pub mod dialect;
pub mod frontdoor;
pub mod lexer;
pub mod normalizer;
pub mod parser;
pub mod runtime;
pub mod stdlib;
pub mod surface;
pub mod term_map;

pub use age_gate::{age_not_available_error, AgeTarget};
pub use ast::*;
pub use canonicalizer::{canonicalize, CanonicalizeReport, LintWarning};
pub use dialect::DialectConfig;
pub use frontdoor::preprocess_frontdoor_source;
pub use lexer::{LexError, Lexer, Token, TokenKind};
pub use normalizer::{normalize, NormalizationLevel, Normalizer};
pub use parser::{ParseError, ParseMode, Parser};
pub use runtime::{
    input_just_pressed, input_pressed, list_add, list_len, list_new, list_nth, list_remove,
    list_set, string_concat, string_join, string_len, string_split, InputState, RuntimeError,
    Value,
};
pub use stdlib::{
    canonicalize_type_alias, input_function_sigs, list_function_sigs, minimal_stdlib_sigs,
    string_function_sigs, FunctionSig,
};
pub use surface::{surface_form, SurfaceError};

/// 편리 함수: 소스 → AST
pub fn parse(source: &str, file_path: &str) -> Result<CanonProgram, ParseError> {
    parse_with_mode(source, file_path, ParseMode::Strict)
}

/// 편리 함수: frontdoor 입력 표면 전처리 후 AST
pub fn parse_frontdoor_with_mode(
    source: &str,
    file_path: &str,
    mode: ParseMode,
) -> Result<CanonProgram, ParseError> {
    let prepared = preprocess_frontdoor_source(source);
    parse_with_mode(&prepared, file_path, mode)
}

/// 편리 함수: 소스 → AST (모드 지정)
pub fn parse_with_mode(
    source: &str,
    file_path: &str,
    mode: ParseMode,
) -> Result<CanonProgram, ParseError> {
    let tokens = Lexer::new(source).tokenize().map_err(|e| ParseError {
        span: crate::ast::Span {
            start: e.pos,
            end: e.pos + 1,
        },
        message: e.message,
    })?;

    let mut parser = Parser::new_with_mode(tokens, mode);
    parser.parse_program(source.to_string(), file_path.to_string())
}

/// 편리 함수: 소스 → 정본화
pub fn parse_and_normalize(
    source: &str,
    file_path: &str,
    level: NormalizationLevel,
) -> Result<String, ParseError> {
    let mut program = parse(source, file_path)?;
    let _report = canonicalize(&mut program)?;
    Ok(normalize(&program, level))
}

/// 편리 함수: frontdoor 입력 표면 전처리 후 정본화
pub fn parse_frontdoor_and_normalize(
    source: &str,
    file_path: &str,
    level: NormalizationLevel,
) -> Result<String, ParseError> {
    let prepared = preprocess_frontdoor_source(source);
    parse_and_normalize(&prepared, file_path, level)
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn first_call_args(source: &str) -> Vec<ArgBinding> {
        let program = parse(source, "test.ddoni").unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        let stmt = body.stmts.first().expect("stmt");
        let expr = match stmt {
            Stmt::Expr { expr, .. } => expr,
            Stmt::Return { value, .. } => value,
            _ => panic!("call expr expected"),
        };
        match &expr.kind {
            ExprKind::Call { args, .. } => args.clone(),
            _ => panic!("call expr expected"),
        }
    }

    #[test]
    fn dialect_header_is_rejected() {
        let source = r#"
#말씨: en
검사:셈씨 = {
    (1 < 2) if {
        1 돌려줘.
    } else {
        2 돌려줘.
    }
}
"#;
        let err = parse(source, "test.ddoni").expect_err("pragma header rejected");
        assert!(err.message.contains("길잡이말"));
    }

    #[test]
    fn test_full_pipeline() {
        let source = "나이 : 수 = 10";
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();

        assert_eq!(normalized.trim(), "나이:수 = 10");
    }

    #[test]
    fn test_complex_function() {
        let source = r#"
(x:수, y:수) 더하:셈씨 = {
    x + y 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();

        assert_eq!(program.items.len(), 1);

        let TopLevelItem::SeedDef(seed) = &program.items[0];
        assert_eq!(seed.canonical_name, "더하");
        assert_eq!(seed.params.len(), 2);
    }

    #[test]
    fn test_korean_call() {
        let source = r#"
(x:수) 테스트:셈씨 = {
    (x) 증가.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();

        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("(x) 증가"));
    }

    #[test]
    fn test_call_tail_equivalence_shortens() {
        let source = r#"
(대상:수) 회복:움직씨 = {
    대상 <- 1.
}

연습:움직씨 = {
    (1) 회복하기.
    (1) 회복기.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "연습")
            .expect("연습 seed");
        let body = seed.body.as_ref().expect("연습 body");
        let funcs: Vec<String> = body
            .stmts
            .iter()
            .map(|stmt| match stmt {
                Stmt::Expr { expr, .. } => match &expr.kind {
                    ExprKind::Call { func, .. } => func.clone(),
                    _ => panic!("call expr expected"),
                },
                _ => panic!("expr stmt expected"),
            })
            .collect();
        assert_eq!(funcs, vec!["회복기".to_string(), "회복기".to_string()]);
    }

    #[test]
    fn test_seed_name_conflict_ha_is_error() {
        let source = r#"
회복:움직씨 = { }
회복하:움직씨 = { }
"#;
        let err = parse(source, "test.ddoni").expect_err("seed conflict");
        assert!(err.message.contains("E_SEED_NAME_CONFLICT_HA"));
    }

    #[test]
    fn test_term_lint_fatal_is_error() {
        let source = r#"
자산:셈씨 = {
    1 돌려줘.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("fatal term");
        assert!(err.message.contains("TERM-LINT-01"));
        assert!(err.message.contains("TERM-FATAL-001"));
    }

    #[test]
    fn test_name_lint_reserved_word_is_error() {
        let source = r#"
마디:셈씨 = {
    1 돌려줘.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("reserved word");
        assert!(err.message.contains("NAME-LINT-01"));
        assert!(err.message.contains("예약어"));
    }

    #[test]
    fn test_name_lint_josa_only_is_error() {
        let source = r#"
이:셈씨 = {
    1 돌려줘.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("josa only");
        assert!(err.message.contains("NAME-LINT-01"));
        assert!(err.message.contains("조사"));
    }

    #[test]
    fn test_term_lint_legacy_warns_without_rewrite() {
        let source = r#"
변수:셈씨 = {
    1 돌려줘.
}
"#;
        let mut program = parse(source, "test.ddoni").unwrap();
        let report = canonicalize(&mut program).unwrap();
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        assert_eq!(seed.canonical_name, "변수");
        assert!(report.warnings.iter().any(|w| w.code == "TERM-WARN-001"));
    }

    #[test]
    fn test_bogae_jangmyeon_alias_is_rejected() {
        let source = r#"
테스트:움직씨 = {
    보개장면 {
        #자막("테스트").
    }.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("legacy alias");
        assert!(err.message.contains("보개장면"));
        assert!(err.message.contains("보개마당"));
    }

    #[test]
    fn test_block_header_colon_warns_in_canonicalize_report() {
        let source = r#"
테스트:움직씨 = {
    채비: { 점수:수 <- 0. }.
    반복: { 멈추기. }.
}
"#;
        let mut program = parse(source, "test.ddoni").unwrap();
        let report = canonicalize(&mut program).unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w.code == "W_BLOCK_HEADER_COLON_DEPRECATED"));
    }

    #[test]
    fn test_block_header_no_colon_has_no_deprecation_warning() {
        let source = r#"
테스트:움직씨 = {
    채비 { 점수:수 <- 0. }.
    반복 { 멈추기. }.
}
"#;
        let mut program = parse(source, "test.ddoni").unwrap();
        let report = canonicalize(&mut program).unwrap();
        assert!(!report
            .warnings
            .iter()
            .any(|w| w.code == "W_BLOCK_HEADER_COLON_DEPRECATED"));
    }

    #[test]
    fn test_beat_block_header_colon_warns_in_canonicalize_report() {
        let source = r#"
테스트:움직씨 = {
    덩이: {
        살림.x <- 1 미루기.
    }.
}
"#;
        let mut program = parse(source, "test.ddoni").unwrap();
        let report = canonicalize(&mut program).unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w.code == "W_BLOCK_HEADER_COLON_DEPRECATED"));
    }

    #[test]
    fn test_bundle_alias_block_header_colon_is_rejected() {
        let source = r#"
테스트:움직씨 = {
    묶음: {
        살림.x <- 1 미루기.
    }.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("bundle alias must fail");
        assert_eq!(err.code(), "E_PARSE");
    }

    #[test]
    fn test_bogae_madang_with_space_roundtrip_parseable() {
        let source = r#"
테스트:움직씨 = {
    보개마당 {
        #자막("테스트").
    }.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();
        let normalized = normalize(&program, NormalizationLevel::N1);
        let reparsed = parse(&normalized, "test_roundtrip.ddoni");
        assert!(reparsed.is_ok());
    }

    #[test]
    fn test_guseong_alias_normalizes_to_jjaim_block() {
        let source = r#"
테스트:움직씨 = {
    구성 {
        상태 { theta <- 0.8. }.
    }.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();
        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("짜임 {"));
        assert!(!normalized.contains("구성 {"));
    }

    #[test]
    fn test_jjaim_block_roundtrip_parseable() {
        let source = r#"
테스트:움직씨 = {
    짜임 {
        상태 { theta <- 0.8. }.
        출력 { 끝점 <- (0.0, 0.0). }.
    }.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();
        let normalized = normalize(&program, NormalizationLevel::N1);
        let reparsed = parse(&normalized, "test_roundtrip.ddoni");
        assert!(reparsed.is_ok());
    }

    #[test]
    fn test_string_literal_parsing() {
        let source = r#"
인사:셈씨 = {
    "안녕" 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();

        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().unwrap();
        match &body.stmts[0] {
            Stmt::Return { value, .. } => match &value.kind {
                ExprKind::Literal(Literal::String(s)) => {
                    assert_eq!(s, "안녕");
                }
                _ => panic!("Expected string literal"),
            },
            _ => panic!("Expected Return"),
        }
    }

    #[test]
    fn test_mutation() {
        let source = r#"
(대상:플레이어) 회복:움직씨 = {
    대상.HP <- 대상.HP + 10.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();

        let TopLevelItem::SeedDef(seed) = &program.items[0];
        assert!(matches!(seed.seed_kind, SeedKind::Umjikssi));

        let body = seed.body.as_ref().unwrap();
        assert_eq!(body.stmts.len(), 1);

        match &body.stmts[0] {
            Stmt::Mutate { .. } => {}
            _ => panic!("Expected Mutate"),
        }
    }

    #[test]
    fn test_if_statement_parsing() {
        let source = r#"
(x:수) 판정:셈씨 = {
    (x < 0) 일때 {
        "음수" 돌려줘.
    } 아니면 {
        "양수" 돌려줘.
    }
}
"#;
        let program = parse(source, "test.ddoni").unwrap();

        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().unwrap();
        assert!(matches!(body.stmts[0], Stmt::If { .. }));
    }

    #[test]
    fn test_logical_ops_parse() {
        let source = r#"
테스트:셈씨 = {
    값 <- (1 < 2) && (2 < 3).
    값2 <- (1 < 2) || (2 < 3).
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("&&"));
        assert!(normalized.contains("||"));
    }

    #[test]
    fn test_logical_ops_korean_parse() {
        let source = r#"
테스트:셈씨 = {
    값 <- (1 < 2) 그리고 (2 < 3).
    값2 <- (1 < 2) 또는 (2 < 3).
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("그리고"));
        assert!(normalized.contains("또는"));
    }

    #[test]
    fn test_not_suffix_normalizes() {
        let source = r#"
테스트:셈씨 = {
    값 <- (1 < 2) 아님.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("아님"));
    }

    #[test]
    fn test_return_alias_normalizes_to_doedollim() {
        let source = r#"
(값:수) 테스트:셈씨 = {
    1 돌려줘.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("1 되돌림."));
        assert!(!normalized.contains("1 돌려줘."));
    }

    #[test]
    fn test_repeat_alias_normalizes_to_doepuli() {
        let source = r#"
테스트:움직씨 = {
    반복: { 멈추기. }.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("되풀이 {"));
        assert!(!normalized.contains("반복 {"));
    }

    #[test]
    fn test_audit_alias_normalizes_to_tolabogi() {
        let source = r#"
테스트:움직씨 = {
    값 감사.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("값 톺아보기."));
        assert!(!normalized.contains("값 감사."));
    }

    #[test]
    fn test_default_param_requires_trailing() {
        let source = r#"
(x:수=1, y:수) 테스트:셈씨 = {
    y 돌려줘.
}
"#;
        assert!(parse(source, "test.ddoni").is_err());
    }

    #[test]
    fn test_default_param_injection() {
        let source = r#"
(x:수, y:수=1) 더하:셈씨 = {
    x + y 돌려줘.
}

테스트:셈씨 = {
    (10) 더하기.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("10:x 1:y 더하기"));
    }

    #[test]
    fn test_optional_param_injection() {
        let source = r#"
(x:수, y:수?) 더하:셈씨 = {
    y 돌려줘.
}

테스트:셈씨 = {
    (10) 더하기.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("10:x 없음:y 더하기"));
    }

    #[test]
    fn test_optional_default_injection() {
        let source = r#"
(x:수, y:수?=5) 더하:셈씨 = {
    x + y 돌려줘.
}

테스트:셈씨 = {
    (10) 더하기.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("10:x 5:y 더하기"));
    }

    #[test]
    fn test_optional_default_value_overrides() {
        let source = r#"
(x:수, y:수?=5) 더하:셈씨 = {
    x + y 돌려줘.
}

테스트:셈씨 = {
    (10, 7) 더하기.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("10:x 7:y 더하기"));
    }

    #[test]
    fn test_multiple_optional_defaults_are_deterministic() {
        let source = r#"
(x:수, y:수?=1, z:수?=2) 합:셈씨 = {
    x + y + z 돌려줘.
}

테스트:셈씨 = {
    (3) 합.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("3:x 1:y 2:z 합"));
    }

    #[test]
    fn test_josa_binding_orders_args() {
        let source = r#"
(대상:수~을~를, 주체:수~이~가) 이동:셈씨 = {
    대상 돌려줘.
}

        테스트:셈씨 = {
    (3가, 1을) 이동.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("1~을 3~이 이동"));
    }

    #[test]
    fn test_josa_ambiguity_requires_fix() {
        let source = r#"
(대상:수~을, 도구:수~을) 이동:셈씨 = {
    대상 돌려줘.
}

테스트:셈씨 = {
    (1을) 이동.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("expected ambiguity error");
        assert_eq!(err.code(), "E_PARSE_CALL_JOSA_AMBIGUOUS");
        assert!(err.message.contains("모호합니다"));
        assert!(err.message.contains("값:핀"));
    }

    #[test]
    fn test_mood_inferred_from_suffix() {
        let source = r#"
테스트:셈씨 = {
    (1) 묻니.
    (1) 하자.
    (1) 해라.
    (1) 좋구나.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();

        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().unwrap();
        let moods: Vec<Mood> = body
            .stmts
            .iter()
            .map(|stmt| match stmt {
                Stmt::Expr { mood, .. } => mood.clone(),
                _ => panic!("Expected Expr"),
            })
            .collect();

        assert_eq!(
            moods,
            vec![
                Mood::Interrogative,
                Mood::Suggestive,
                Mood::Imperative,
                Mood::Exclamative,
            ]
        );
    }

    #[test]
    fn test_fixed_pin_binding() {
        let source = r#"
(대상:수~을, 도구:수~을) 이동:셈씨 = {
    도구 돌려줘.
}

테스트:셈씨 = {
    (도구=1, 2) 이동.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("2:대상 1:도구 이동"));
    }

    #[test]
    fn test_bare_josa_call_parses() {
        let source = r#"
(대상:수~을~를, 주체:수~이~가) 이동:셈씨 = {
    대상 돌려줘.
}

테스트:셈씨 = {
    1~을 3~이 이동.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("1~을 3~이 이동"));
    }

    #[test]
    fn test_bare_pin_call_parses() {
        let source = r#"
(x:수, y:수) 더하:셈씨 = {
    x + y 돌려줘.
}

테스트:셈씨 = {
    10:x 1:y 더하기.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("10:x 1:y 더하기"));
    }

    #[test]
    fn test_canonicalize_normalizes_alias_josa_to_primary_form() {
        let source = r#"
(왼:수~을~를, 오른:수~에) 더하:셈씨 = {
    왼 + 오른 돌려줘.
}

증명:셈씨 = {
    (3를, 1에) 더하.
}
"#;
        let mut program = parse(source, "test.ddoni").unwrap();
        canonicalize(&mut program).unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "증명")
            .expect("증명 seed");
        let body = seed.body.as_ref().expect("증명 body");
        let expr = match body.stmts.first().expect("stmt") {
            Stmt::Expr { expr, .. } => expr,
            other => panic!("expr stmt expected: {other:?}"),
        };
        let ExprKind::Call { args, .. } = &expr.kind else {
            panic!("call expected");
        };
        assert_eq!(args[0].josa.as_deref(), Some("을"));
        assert_eq!(args[1].josa.as_deref(), Some("에"));
    }

    #[test]
    fn test_duplicate_role_josa_rejected_deterministically() {
        let source = r#"
(왼:수~을~를, 오른:수~에) 더하:셈씨 = {
    왼 + 오른 돌려줘.
}

증명:셈씨 = {
    (3을, 1을) 더하.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("expected duplicate pin error");
        assert_eq!(err.code(), "E_PARSE_CALL_PIN_DUPLICATE");
        assert!(err.message.contains("핀 '왼'에 인자가 중복되었습니다"));
    }

    #[test]
    fn test_explicit_particle_reorder_is_success_without_conflict_warning() {
        let source = r#"
(왼:수~을~를, 오른:수~에) 더하:셈씨 = {
    왼 + 오른 돌려줘.
}

증명:셈씨 = {
    (1을, 3에) 더하.
}
"#;
        let mut program = parse(source, "test.ddoni").unwrap();
        let report = canonicalize(&mut program).unwrap();
        assert!(!report.warnings.iter().any(|w| w.code.contains("CONFLICT")));
        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("1~을 3~에 더하"));
    }

    #[test]
    fn test_tailed_call_keeps_alias_josa_normalization_without_warning() {
        let source = r#"
(왼:수~을~를, 오른:수~에) 더하:셈씨 = {
    왼 + 오른 돌려줘.
}

증명:셈씨 = {
    (3를, 1에) 더하기.
}
"#;
        let mut program = parse(source, "test.ddoni").unwrap();
        let report = canonicalize(&mut program).unwrap();
        assert!(report.warnings.is_empty());
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "증명")
            .expect("증명 seed");
        let body = seed.body.as_ref().expect("증명 body");
        let expr = match body.stmts.first().expect("stmt") {
            Stmt::Expr { expr, .. } => expr,
            other => panic!("expr stmt expected: {other:?}"),
        };
        let ExprKind::Call { args, .. } = &expr.kind else {
            panic!("call expected");
        };
        assert_eq!(args[0].josa.as_deref(), Some("을"));
        assert_eq!(args[1].josa.as_deref(), Some("에"));
        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("3~을 1~에 더하기"));
    }

    #[test]
    fn test_fixed_pin_ambiguity_emits_conflict_warning() {
        let source = r#"
(시작:수~에서, 끝:수~에서?) 이동:셈씨 = {
    시작 돌려줘.
}

증명:셈씨 = {
    (100@m:시작~에서) 이동하기.
}
"#;
        let mut program = parse(source, "test.ddoni").unwrap();
        let report = canonicalize(&mut program).unwrap();
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.warnings[0].code, "W_CALL_JOSA_CONFLICT_FIXED");
        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("100@m:처음 없음:끝 이동기"));
    }

    #[test]
    fn test_unit_suffix_expression() {
        let source = r#"
테스트:셈씨 = {
    거리 <- 10@m.
    거리 돌려줘.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("거리 <- 10@m."));
    }

    #[test]
    fn test_unit_dimension_mismatch_is_error() {
        let source = r#"
테스트:셈씨 = {
    값 <- 10@m + 2@s.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("dimension mismatch");
        assert!(err.message.contains("단위 차원이 다릅니다"));
    }

    #[test]
    fn test_currency_unit_mismatch_is_error() {
        let source = r#"
테스트:셈씨 = {
    값 <- 1@KRW + 2@USD.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("currency mismatch");
        assert!(err.message.contains("단위 차원이 다릅니다"));
    }

    #[test]
    fn test_speed_unit_addition_is_ok() {
        let source = r#"
테스트:셈씨 = {
    값 <- 10@kmh + 1@mps.
}
"#;
        assert!(parse(source, "test.ddoni").is_ok());
    }

    #[test]
    fn test_resource_literal_expression() {
        let source = r#"
테스트:셈씨 = {
    그림 <- @"그림/주인공.png".
    그림 돌려줘.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("그림 <- @\"그림/주인공.png\"."));
    }

    #[test]
    fn test_list_literal_normalizes_to_charim_call() {
        let source = r#"
테스트:셈씨 = {
    목록 <- [1, 2, 3].
    빈 <- [].
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("목록 <- (1, 2, 3) 차림."));
        assert!(normalized.contains("빈 <- () 차림."));
    }

    #[test]
    fn test_index_sugar_normalizes_to_charim_value() {
        let source = r#"
테스트:셈씨 = {
    값 <- 보드[2].
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("값 <- 보드:대상 2:i 차림.값."));
    }

    #[test]
    fn test_index_assign_normalizes_to_charim_set() {
        let source = r#"
테스트:셈씨 = {
    보드[2] <- 3.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("보드 <- 보드:대상 2:i 3:값 차림.바꾼값."));
    }

    #[test]
    fn test_right_assign_normalizes_to_left_assign() {
        let source = r#"
테스트:셈씨 = {
    점수 <- 0.
    점수 + 1 -> 점수.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("점수 <- 점수 + 1."));
    }

    #[test]
    fn test_right_assign_index_normalizes_to_charim_set() {
        let source = r#"
테스트:셈씨 = {
    3 -> 보드[2].
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("보드 <- 보드:대상 2:i 3:값 차림.바꾼값."));
    }

    #[test]
    fn test_suffix_chain_unit_pin_josa() {
        let source = r#"
(거리:수~에서) 이동:셈씨 = {
    거리 돌려줘.
}

테스트:셈씨 = {
    (100@m:거리~에서) 이동.
}
"#;
        let args = first_call_args(source);
        assert_eq!(args.len(), 1);
        let arg = &args[0];
        assert_eq!(arg.josa.as_deref(), Some("에서"));
        assert_eq!(arg.resolved_pin.as_deref(), Some("거리"));
        assert!(matches!(arg.binding_reason, BindingReason::UserFixed));
        match &arg.expr.kind {
            ExprKind::Suffix {
                at: AtSuffix::Unit(unit),
                ..
            } => {
                assert_eq!(unit, "m");
            }
            _ => panic!("unit suffix expected"),
        }
    }

    #[test]
    fn test_suffix_chain_unit_josa_binding() {
        let source = r#"
(거리:수~에서) 이동:셈씨 = {
    거리 돌려줘.
}

테스트:셈씨 = {
    (100@m~에서) 이동.
}
"#;
        let args = first_call_args(source);
        let arg = &args[0];
        assert_eq!(arg.josa.as_deref(), Some("에서"));
        assert_eq!(arg.resolved_pin.as_deref(), Some("거리"));
        assert!(matches!(arg.binding_reason, BindingReason::Dictionary));
    }

    #[test]
    fn test_suffix_chain_asset_pin_josa() {
        let source = r#"
(배경:그림~으로) 보:셈씨 = {
    배경 돌려줘.
}

테스트:셈씨 = {
    (@"그림/주인공.png":배경~으로) 보기.
}
"#;
        let args = first_call_args(source);
        let arg = &args[0];
        assert_eq!(arg.josa.as_deref(), Some("으로"));
        assert_eq!(arg.resolved_pin.as_deref(), Some("배경"));
        assert!(matches!(arg.binding_reason, BindingReason::UserFixed));
        match &arg.expr.kind {
            ExprKind::Literal(Literal::Resource(path)) => {
                assert_eq!(path, "그림/주인공.png");
            }
            _ => panic!("resource literal expected"),
        }
    }

    #[test]
    fn test_suffix_chain_order_invalid() {
        let source = r#"
(거리:수~에서) 이동:셈씨 = {
    거리 돌려줘.
}

테스트:셈씨 = {
    (100~에서:거리@m) 이동.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("order error");
        assert!(err.message.contains("접미 순서는 값@단위/자원:핀~조사"));
    }

    #[test]
    fn test_suffix_chain_duplicate_tilde() {
        let source = r#"
(거리:수~에서) 이동:셈씨 = {
    거리 돌려줘.
}

테스트:셈씨 = {
    (100@m:거리~에서~부터) 이동.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("tilde error");
        assert!(err.message.contains("맨 끝"));
    }

    #[test]
    fn test_suffix_chain_duplicate_at() {
        let source = r#"
(거리:수~에서) 이동:셈씨 = {
    거리 돌려줘.
}

테스트:셈씨 = {
    (거리=100@m@m) 이동.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("@ error");
        assert!(err.message.contains("@"));
        assert!(err.message.contains("1회만 허용"));
    }

    #[test]
    fn test_suffix_chain_unit_unknown_suggests_pin() {
        let source = r#"
(대상:수~를) 먹어:셈씨 = {
    대상 돌려줘.
}

테스트:셈씨 = {
    (사과@대상~를) 먹어.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("unit unknown");
        assert!(err.message.contains("핀 고정은 값:핀"));
    }

    #[test]
    fn test_suffix_chain_fixed_pin_resolves_ambiguity() {
        let source = r#"
(시작:수~에서, 끝:수~에서?) 이동:셈씨 = {
    시작 돌려줘.
}

테스트:셈씨 = {
    (100@m:시작~에서) 이동.
}
"#;
        let args = first_call_args(source);
        let arg = &args[0];
        assert!(
            matches!(arg.resolved_pin.as_deref(), Some("시작") | Some("처음")),
            "unexpected resolved pin: {:?}",
            arg.resolved_pin
        );
        assert!(matches!(arg.binding_reason, BindingReason::UserFixed));
    }

    #[test]
    fn test_suffix_chain_fixed_pin_resolves_ambiguity_other_pin() {
        let source = r#"
(시작:수~에서?, 끝:수~에서) 이동:셈씨 = {
    끝 돌려줘.
}

테스트:셈씨 = {
    (100@m:끝~에서) 이동.
}
"#;
        let args = first_call_args(source);
        let arg = args
            .iter()
            .find(|arg| arg.resolved_pin.as_deref() == Some("끝"))
            .expect("끝 arg");
        assert!(matches!(arg.binding_reason, BindingReason::UserFixed));
    }

    #[test]
    fn test_pipe_injects_flow_arg() {
        let source = concat!(
            "(value:\u{c218}~\u{c744}) Make:\u{c148}\u{c528} = {\n",
            "    value \u{b3cc}\u{b824}\u{c918}.\n",
            "}\n",
            "\n",
            "(lhs:\u{c218}~\u{c744}, rhs:\u{c218}~\u{c5d0}) Add:\u{c148}\u{c528} = {\n",
            "    lhs + rhs \u{b3cc}\u{b824}\u{c918}.\n",
            "}\n",
            "\n",
            "Test:\u{c148}\u{c528} = {\n",
            "    (10) Make \u{d574}\u{c11c} (rhs=2) Add.\n",
            "}\n",
        );
        let program = parse(source, "test.ddoni").unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("Test body");
        let stmt = body.stmts.first().expect("stmt");
        let expr = match stmt {
            Stmt::Expr { expr, .. } => expr,
            _ => panic!("expr stmt expected"),
        };
        let ExprKind::Pipe { stages } = &expr.kind else {
            panic!("pipe expected");
        };
        let stage = stages.get(1).expect("second stage");
        let ExprKind::Call { args, .. } = &stage.kind else {
            panic!("call expected");
        };
        let injected = args
            .iter()
            .find(|arg| matches!(arg.binding_reason, BindingReason::FlowInjected))
            .expect("flow injected arg");
        assert_eq!(injected.resolved_pin.as_deref(), Some("lhs"));
        assert!(matches!(injected.expr.kind, ExprKind::FlowValue));
    }

    #[test]
    fn test_pipe_rejects_non_call_stage() {
        let source = concat!(
            "(value:\u{c218}~\u{c744}) Make:\u{c148}\u{c528} = {\n",
            "    value \u{b3cc}\u{b824}\u{c918}.\n",
            "}\n",
            "\n",
            "Test:\u{c148}\u{c528} = {\n",
            "    (10) Make \u{d574}\u{c11c} 1 + 2.\n",
            "}\n",
        );
        let err = parse(source, "test.ddoni").expect_err("pipe call only");
        assert!(err.message.contains("PIPE-CALL-ONLY-01"));
    }

    #[test]
    fn test_thunk_eval_markers_parse() {
        let source = concat!(
            "Test:\u{c148}\u{c528} = {\n",
            "    { 1 }\u{d55c}\u{ac83} \u{b3cc}\u{b824}\u{c918}.\n",
            "}\n",
        );
        let program = parse(source, "test.ddoni").unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("Test body");
        let stmt = body.stmts.first().expect("stmt");
        let expr = match stmt {
            Stmt::Return { value, .. } => value,
            _ => panic!("return expected"),
        };
        match &expr.kind {
            ExprKind::Eval { mode, .. } => assert!(matches!(mode, ThunkEvalMode::Value)),
            _ => panic!("eval expected"),
        }
    }

    #[test]
    fn test_eval_bool_rejects_mutation() {
        let source = concat!(
            "Test:\u{c148}\u{c528} = {\n",
            "    { x <- 1. }\u{c778}\u{ac83}.\n",
            "}\n",
        );
        let err = parse(source, "test.ddoni").expect_err("mutation in eval bool");
        assert!(err.message.contains("\u{c778}\u{ac83}"));
    }

    #[test]
    fn test_choose_and_contract_parse() {
        let source = concat!(
            "Test:\u{c148}\u{c528} = {\n",
            "    \u{ace0}\u{b974}\u{ae30}:\n",
            "      { 1 }\u{c778}\u{ac83}: { 2. }\n",
            "      \u{c544}\u{b2c8}\u{ba74}: { 3. }\n",
            "    { 1 }\u{c778}\u{ac83} \u{c804}\u{c81c}\u{d558}\u{c5d0}\n",
            "      \u{c544}\u{b2c8}\u{ba74} { 4. }\n",
            "}\n",
        );
        let program = parse(source, "test.ddoni").unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("Test body");
        assert!(matches!(body.stmts[0], Stmt::Choose { .. }));
        assert!(matches!(body.stmts[1], Stmt::Contract { .. }));
    }

    #[test]
    fn test_contract_alert_mode_normalizes() {
        let source = r#"
Test:셈씨 = {
    { 1 }인것 바탕으로(알림)
      아니면 { 2. }
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("바탕으로(알림)"));
    }

    #[test]
    fn test_contract_keywords_batang_and_dajim_parse() {
        let source = concat!(
            "Test:\u{c148}\u{c528} = {\n",
            "    { 1 }\u{c778}\u{ac83} \u{bc14}\u{d0d5}\u{c73c}\u{b85c}\n",
            "      \u{c544}\u{b2c8}\u{ba74} { 2. }\n",
            "    { 1 }\u{c778}\u{ac83} \u{b2e4}\u{c9d0}\u{d558}\u{ace0}\n",
            "      \u{c544}\u{b2c8}\u{ba74} { 3. }\n",
            "}\n",
        );
        let program = parse(source, "test.ddoni").unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("Test body");
        assert!(matches!(body.stmts[0], Stmt::Contract { .. }));
        assert!(matches!(body.stmts[1], Stmt::Contract { .. }));
    }

    #[test]
    fn test_template_injection_prefix_parses() {
        let source = r#"
Test:셈씨 = {
    (id=1) 글무늬{"ID={id}"}.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("Test body");
        let stmt = body.stmts.first().expect("stmt");
        let expr = match stmt {
            Stmt::Expr { expr, .. } => expr,
            _ => panic!("expr expected"),
        };
        match &expr.kind {
            ExprKind::TemplateRender { inject, .. } => {
                assert_eq!(inject.len(), 1);
                assert_eq!(inject[0].0, "id");
            }
            _ => panic!("template render expected"),
        }
    }

    #[test]
    fn test_formula_injection_prefix_parses() {
        let source = r#"
Test:셈씨 = {
    (x=6) (#ascii) 수식{ y = 2*x + 3/2 }.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("Test body");
        let stmt = body.stmts.first().expect("stmt");
        let expr = match stmt {
            Stmt::Expr { expr, .. } => expr,
            _ => panic!("expr expected"),
        };
        match &expr.kind {
            ExprKind::FormulaEval { inject, .. } => {
                assert_eq!(inject.len(), 1);
                assert_eq!(inject[0].0, "x");
            }
            _ => panic!("formula eval expected"),
        }
    }

    #[test]
    fn test_injection_in_form_parses() {
        let source = r#"
Test:셈씨 = {
    직선 <- (#ascii) 수식{ y = 2*x + 3/2 }.
    (x=6)인 직선 풀기.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("Test body");
        let stmt = body.stmts.get(1).expect("second stmt");
        let expr = match stmt {
            Stmt::Expr { expr, .. } => expr,
            _ => panic!("expr expected"),
        };
        let ExprKind::Call { func, args } = &expr.kind else {
            panic!("call expected");
        };
        assert_eq!(func, "풀기");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].resolved_pin.as_deref(), Some("식"));
        assert_eq!(args[1].resolved_pin.as_deref(), Some("주입"));
    }

    #[test]
    fn test_regex_literal_parses() {
        let source = r#"
Test:셈씨 = {
    패턴 <- 정규식{"^[A-Z]{2}[0-9]+$", "i"}.
}
"#;
        let program = parse(source, "test.ddoni").expect("regex parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("Test body");
        let stmt = body.stmts.first().expect("stmt");
        let Stmt::Mutate { value, .. } = stmt else {
            panic!("mutate expected");
        };
        let ExprKind::Literal(Literal::Regex(regex)) = &value.kind else {
            panic!("regex literal expected");
        };
        assert_eq!(regex.pattern, "^[A-Z]{2}[0-9]+$");
        assert_eq!(regex.flags, "i");
    }

    #[test]
    fn test_regex_literal_normalizes_flag_order() {
        let source = r#"
Test:셈씨 = {
    패턴 <- 정규식{"a.b", "si"}.
}
"#;
        let program = parse(source, "test.ddoni").expect("regex parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("Test body");
        let stmt = body.stmts.first().expect("stmt");
        let Stmt::Mutate { value, .. } = stmt else {
            panic!("mutate expected");
        };
        let ExprKind::Literal(Literal::Regex(regex)) = &value.kind else {
            panic!("regex literal expected");
        };
        assert_eq!(regex.flags, "is");
        let normalized =
            parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).expect("normalize");
        assert!(normalized.contains("정규식{\"a.b\", \"is\"}"));
    }

    #[test]
    fn test_regex_literal_requires_attached_block() {
        let source = r#"
Test:셈씨 = {
    패턴 <- 정규식 {"^[A-Z]{2}[0-9]+$"}.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("regex block spacing");
        assert!(err.message.contains("정규식{"));
    }

    #[test]
    fn test_state_machine_literal_parses() {
        let source = r#"
Test:셈씨 = {
    기계 <- 상태머신{
        빨강, 초록, 노랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        초록 에서 노랑 으로.
        노랑 에서 빨강 으로.
        바뀔때마다 전이_안전 살피기.
    }.
}
"#;
        let program = parse(source, "state_machine.ddoni").expect("state machine parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("body");
        let stmt = body.stmts.first().expect("stmt");
        let Stmt::Mutate { value, .. } = stmt else {
            panic!("mutate expected");
        };
        let ExprKind::StateMachine(machine) = &value.kind else {
            panic!("state machine literal expected");
        };
        assert_eq!(machine.initial, "빨강");
        assert_eq!(machine.states, vec!["빨강", "초록", "노랑"]);
        assert_eq!(machine.transitions.len(), 3);
        assert_eq!(machine.on_transition_checks, vec!["전이_안전"]);
    }

    #[test]
    fn test_state_machine_literal_lexes_following_dot_and_stmt() {
        let source = r#"
Test:셈씨 = {
    기계 <- 상태머신{
        빨강, 초록 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
    }.
    현재 <- (기계) 처음으로.
}
"#;
        let tokens = Lexer::new(source).tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::StateMachineBlock(_))));
        let state_machine_idx = tokens
            .iter()
            .position(|token| matches!(token.kind, TokenKind::StateMachineBlock(_)))
            .expect("state machine block token");
        assert!(matches!(tokens[state_machine_idx + 1].kind, TokenKind::Dot));
        assert!(tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Ident(name) if name == "현재")));
    }

    #[test]
    fn test_state_machine_literal_allows_following_statements() {
        let source = r#"
Test:셈씨 = {
    기계 <- 상태머신{
        빨강, 초록, 노랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        초록 에서 노랑 으로.
        노랑 에서 빨강 으로.
        바뀔때마다 전이_안전 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program = parse(source, "state_machine.ddoni").expect("parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("body");
        assert_eq!(body.stmts.len(), 3);
    }

    #[test]
    fn test_state_machine_literal_parses_guard_and_action() {
        let source = r#"
Test:셈씨 = {
    기계 <- 상태머신{
        빨강, 초록, 파랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로 걸러서 전이_조건 하고 기록.
        빨강 에서 파랑 으로.
    }.
}
"#;
        let program = parse(source, "state_machine_guard_action.ddoni").expect("parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("body");
        let stmt = body.stmts.first().expect("stmt");
        let Stmt::Mutate { value, .. } = stmt else {
            panic!("mutate expected");
        };
        let ExprKind::StateMachine(machine) = &value.kind else {
            panic!("state machine literal expected");
        };
        assert_eq!(machine.transitions.len(), 2);
        assert_eq!(machine.transitions[0].from, "빨강");
        assert_eq!(machine.transitions[0].to, "초록");
        assert_eq!(
            machine.transitions[0].guard_name.as_deref(),
            Some("전이_조건")
        );
        assert_eq!(machine.transitions[0].action_name.as_deref(), Some("기록"));
        assert_eq!(machine.transitions[1].guard_name, None);
        assert_eq!(machine.transitions[1].action_name, None);
    }

    #[test]
    fn test_assertion_literal_parses() {
        let source = r#"
Test:셈씨 = {
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
}
"#;
        let program = parse(source, "assertion.ddoni").expect("assertion parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("body");
        let stmt = body.stmts.first().expect("stmt");
        let Stmt::Mutate { value, .. } = stmt else {
            panic!("mutate expected");
        };
        let ExprKind::Assertion(assertion) = &value.kind else {
            panic!("assertion literal expected");
        };
        assert!(assertion.body_source.contains("{ 거리 > 0 }인것"));
        assert!(assertion.canon.starts_with("세움{"));
        assert!(assertion.canon.contains("거리 > 0"));
    }

    #[test]
    fn test_assertion_check_call_parses() {
        let source = r#"
Test:셈씨 = {
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=3)인 검사 살피기.
}
"#;
        let program = parse(source, "assertion_call.ddoni").expect("assertion call parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "Test")
            .expect("Test seed");
        let body = seed.body.as_ref().expect("body");
        let stmt = &body.stmts[1];
        let Stmt::Mutate { value, .. } = stmt else {
            panic!("mutate expected");
        };
        let ExprKind::Call { func, args } = &value.kind else {
            panic!("call expected");
        };
        assert_eq!(func, "살피기");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].resolved_pin.as_deref(), Some("세움"));
        assert_eq!(args[1].resolved_pin.as_deref(), Some("값들"));
        assert!(matches!(args[0].expr.kind, ExprKind::Var(_)));
    }

    #[test]
    fn test_template_pipe_fill_is_rejected() {
        let source = r#"
Test:셈씨 = {
    글무늬{"ID={id}"} 해서 (id=1) 채우기.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("pipe template fill");
        assert!(err.message.contains("글무늬{...} 해서"));
    }

    #[test]
    fn test_decl_block_parses() {
        let source = r#"
테스트:셈씨 = {
    채비: { 점수:수 <- 0. }.
    점수 <- 점수 + 1.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().unwrap();
        match &body.stmts[0] {
            Stmt::DeclBlock { items, .. } => {
                assert_eq!(items.len(), 1);
                assert!(matches!(items[0].kind, DeclKind::Gureut));
            }
            _ => panic!("decl block expected"),
        }
    }

    #[test]
    fn test_decl_block_without_colon_parses() {
        let source = r#"
테스트:셈씨 = {
    채비 { 점수:수 <- 0. }.
    점수 <- 점수 + 1.
}
"#;
        let program = parse(source, "test.ddoni").unwrap();
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().unwrap();
        match &body.stmts[0] {
            Stmt::DeclBlock { items, .. } => {
                assert_eq!(items.len(), 1);
                assert!(matches!(items[0].kind, DeclKind::Gureut));
            }
            _ => panic!("decl block expected"),
        }
    }

    #[test]
    fn test_root_hide_pragma_is_rejected() {
        let source = r#"
테스트:셈씨 = {
    #바탕숨김.
    값 <- 1.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("pragma rejected");
        assert!(err.message.contains("길잡이말"));
    }

    #[test]
    fn test_pragma_stmt_rejected_inside_seed() {
        let source = r#"
테스트:셈씨 = {
    #그래프(y축=살림.x)
    살림.x <- 1.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("pragma rejected");
        assert!(err.message.contains("길잡이말"));
    }

    #[test]
    fn test_top_level_pragma_is_rejected() {
        let source = r#"
#가져오기 누리/기본
테스트:셈씨 = {
    1 돌려줘.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("top-level pragma rejected");
        assert!(err.message.contains("길잡이말"));
    }

    #[test]
    fn test_setting_block_parses_with_colon() {
        let source = r#"
테스트:셈씨 = {
    설정: { 화면: "기본". }.
    1 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("setting block parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        assert!(matches!(body.stmts[0], Stmt::MetaBlock { .. }));
    }

    #[test]
    fn test_setting_block_without_colon_parses() {
        let source = r#"
테스트:셈씨 = {
    설정 { 화면: "기본". }.
    1 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("setting block without colon parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        assert!(matches!(body.stmts[0], Stmt::MetaBlock { .. }));
    }

    #[test]
    fn test_bogae_block_without_colon_parses() {
        let source = r#"
테스트:셈씨 = {
    보개 { 선(0, 0, 1, 1). }.
    1 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("bogae block parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        match &body.stmts[0] {
            Stmt::MetaBlock { kind, entries, .. } => {
                assert!(matches!(kind, MetaBlockKind::Bogae));
                assert_eq!(entries.len(), 1);
            }
            other => panic!("meta block expected, got {other:?}"),
        }
    }

    #[test]
    fn test_moyang_block_with_colon_parses_as_bogae_meta() {
        let source = r#"
테스트:셈씨 = {
    모양: { 점(0, 0, 크기=0.1). }.
    1 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("moyang block parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        match &body.stmts[0] {
            Stmt::MetaBlock { kind, entries, .. } => {
                assert!(matches!(kind, MetaBlockKind::Bogae));
                assert_eq!(entries.len(), 1);
                assert!(entries[0].contains("점"));
            }
            other => panic!("meta block expected, got {other:?}"),
        }
    }

    #[test]
    fn test_setting_bogae_alias_is_rejected() {
        let source = r#"
테스트:셈씨 = {
    설정보개: { y축: 값. }.
    1 돌려줘.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("legacy bogae alias rejected");
        assert!(err.message.contains("문장 종결") || err.message.contains("설정/보개/슬기"));
    }

    #[test]
    fn test_boim_block_without_colon_is_rejected() {
        let source = r#"
테스트:셈씨 = {
    보임 { y축: 값. }.
    1 돌려줘.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("legacy boim alias rejected");
        assert!(err.message.contains("문장 종결") || err.message.contains("설정/보개/슬기"));
    }

    #[test]
    fn test_foreach_block_without_colon_parses() {
        let source = r#"
테스트:셈씨 = {
    (x) 값목록에 대해 {
        x 보여주기.
    }.
}
"#;
        let program = parse(source, "test.ddoni").expect("foreach without colon parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        assert!(matches!(body.stmts[0], Stmt::ForEach { .. }));
    }

    #[test]
    fn test_quantifier_statements_parse_and_normalize() {
        let source = r#"
증명:셈씨 = {
    n 이 자연수 낱낱에 대해 {
        없음.
    }.
    x 가 실수 중 하나가 {
        없음.
    }.
    y 가 정수 중 딱 하나가 {
        없음.
    }.
}
"#;
        let program = parse(source, "quantifier.ddoni").expect("quantifier parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "증명")
            .expect("증명 seed");
        let body = seed.body.as_ref().expect("증명 body");
        assert!(matches!(
            body.stmts[0],
            Stmt::Quantifier {
                kind: QuantifierKind::ForAll,
                ..
            }
        ));
        assert!(matches!(
            body.stmts[1],
            Stmt::Quantifier {
                kind: QuantifierKind::Exists,
                ..
            }
        ));
        assert!(matches!(
            body.stmts[2],
            Stmt::Quantifier {
                kind: QuantifierKind::ExistsUnique,
                ..
            }
        ));
        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("n 이 자연수 낱낱에 대해 {"));
        assert!(normalized.contains("x 이 실수 중 하나가 {"));
        assert!(normalized.contains("y 이 정수 중 딱 하나가 {"));
    }

    #[test]
    fn test_quantifier_rejects_mutation_in_body() {
        let source = r#"
증명:셈씨 = {
    n 이 자연수 낱낱에 대해 {
        값 <- 1.
    }.
}
"#;
        let err = parse(source, "quantifier_mutation.ddoni").expect_err("quantifier mutation");
        assert!(err
            .message
            .contains("양화 블록 안에서는 '<-'를 사용할 수 없습니다"));
    }

    #[test]
    fn test_quantifier_rejects_show_in_body() {
        let source = r#"
증명:셈씨 = {
    n 이 자연수 중 하나가 {
        n 보여주기.
    }.
}
"#;
        let err = parse(source, "quantifier_show.ddoni").expect_err("quantifier show");
        assert!(err
            .message
            .contains("양화 블록 안에서는 '보여주기'를 사용할 수 없습니다"));
    }

    #[test]
    fn test_legacy_ilmukssi_is_rejected() {
        let source = r#"
테스트:일묶음씨 = {
    1 돌려줘.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("legacy ilmukssi rejected");
        assert!(err.message.contains("일묶음씨"));
        assert!(err.message.contains("갈래씨"));
    }

    #[test]
    fn test_legacy_valuefunc_is_rejected() {
        let source = r#"
테스트:값함수 = {
    1 돌려줘.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("legacy valuefunc rejected");
        assert!(err.message.contains("값함수"));
        assert!(err.message.contains("셈씨"));
    }

    #[test]
    fn test_legacy_decl_headers_are_rejected() {
        let source = r#"
테스트:셈씨 = {
    붙박이마련: { 파이:수 = 3. }.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("legacy decl header");
        assert!(err.message.contains("채비"));
    }

    #[test]
    fn test_tilde_josa_keeps_rparen_for_typed_pin() {
        let source = r#"
(값:_~을) 통과:셈씨 = {
    값 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("typed pin parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "통과")
            .expect("통과 seed");
        assert_eq!(seed.params.len(), 1);
        assert!(matches!(seed.params[0].type_ref, TypeRef::Infer));
        assert_eq!(seed.params[0].josa_list, vec!["을".to_string()]);
    }

    #[test]
    fn test_tilde_josa_with_unit_type_parses() {
        let source = r#"
(거리:(m)수~을) 이동:셈씨 = {
    거리 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("unit typed pin parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "이동")
            .expect("이동 seed");
        assert_eq!(seed.params.len(), 1);
        match &seed.params[0].type_ref {
            TypeRef::Applied { name, args } => {
                assert_eq!(name, "수");
                assert_eq!(args.len(), 1);
            }
            other => panic!("unexpected type ref: {other:?}"),
        }
        assert_eq!(seed.params[0].josa_list, vec!["을".to_string()]);
    }

    #[test]
    fn test_typed_pin_with_numeric_sized_variants_parses() {
        let source = r#"
(x:셈수2, y:바른수4) 통과:셈씨 = {
    x 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("sized variant typed pin parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "통과")
            .expect("통과 seed");
        assert_eq!(seed.params.len(), 2);
        match &seed.params[0].type_ref {
            TypeRef::Named(name) => assert_eq!(name, "셈수2"),
            other => panic!("unexpected type ref: {other:?}"),
        }
        match &seed.params[1].type_ref {
            TypeRef::Named(name) => assert_eq!(name, "바른수4"),
            other => panic!("unexpected type ref: {other:?}"),
        }
    }

    #[test]
    fn test_numeric_sized_variant_types_canonicalize_to_base_names() {
        let source = r#"
테스트:움직씨 = {
    채비 {
        값:셈수2 <- 1.5.
        정:바른수4 <- 7.
    }.
}
"#;
        let mut program = parse(source, "test.ddoni").expect("parse");
        let _report = canonicalize(&mut program).expect("canonicalize");
        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("값:셈수 <- 1.5."));
        assert!(normalized.contains("정:바른수 <- 7."));
        assert!(!normalized.contains("셈수2"));
        assert!(!normalized.contains("바른수4"));
    }

    #[test]
    fn test_numeric_english_alias_types_canonicalize_to_base_names() {
        let source = r#"
테스트:움직씨 = {
    채비 {
        a:fixed64 <- 1.5.
        b:int64 <- 7.
        c:bigint <- ("9") 큰바른수.
        d:rational <- (1, 3) 나눔수.
        e:factorized <- (12) 곱수.
    }.
}
"#;
        let mut program = parse(source, "test.ddoni").expect("parse");
        let _report = canonicalize(&mut program).expect("canonicalize");
        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("a:셈수 <- 1.5."));
        assert!(normalized.contains("b:바른수 <- 7."));
        assert!(normalized.contains("c:큰바른수 <- (\"9\") 큰바른수."));
        assert!(normalized.contains("d:나눔수 <- (1, 3) 나눔수."));
        assert!(normalized.contains("e:곱수 <- (12) 곱수."));
        assert!(!normalized.contains("fixed64"));
        assert!(!normalized.contains("int64"));
        assert!(!normalized.contains("bigint"));
        assert!(!normalized.contains("rational"));
        assert!(!normalized.contains("factorized"));
    }

    #[test]
    fn test_bool_and_collection_alias_types_canonicalize_to_base_names() {
        let source = r#"
(판단:boolean, 열:list, 집:set, 사전:map, 꾸러미:pack) 통과:셈씨 = {
    1 돌려줘.
}
"#;
        let mut program = parse(source, "test.ddoni").expect("parse");
        let _report = canonicalize(&mut program).expect("canonicalize");
        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("참거짓"));
        assert!(normalized.contains("차림"));
        assert!(normalized.contains("모음"));
        assert!(normalized.contains("짝맞춤"));
        assert!(normalized.contains("묶음"));
        assert!(!normalized.contains("boolean"));
        assert!(!normalized.contains("list"));
        assert!(!normalized.contains("set"));
        assert!(!normalized.contains("map"));
        assert!(!normalized.contains("pack"));
    }

    #[test]
    fn test_beat_block_parses_deferred_assignments() {
        let source = r#"
테스트:움직씨 = {
    덩이 {
        살림.x <- 1 미루기.
        살림.y <- (살림.x + 1).
    }.
}
"#;
        let program = parse(source, "test.ddoni").expect("beat block parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let seed_body = seed.body.as_ref().expect("테스트 body");
        let Some(Stmt::BeatBlock { body, .. }) = seed_body.stmts.first() else {
            panic!("beat block expected");
        };
        let Some(Stmt::Mutate { deferred, .. }) = body.stmts.first() else {
            panic!("first mutate expected");
        };
        assert!(*deferred);
        let Some(Stmt::Mutate { deferred, .. }) = body.stmts.get(1) else {
            panic!("second mutate expected");
        };
        assert!(!*deferred);
    }

    #[test]
    fn test_legacy_beat_keyword_is_rejected() {
        let source = r#"
테스트:움직씨 = {
    박자 {
        살림.x <- 1 미루기.
        살림.y <- (살림.x + 1).
    }.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("legacy beat keyword must fail");
        assert_eq!(err.code(), "E_PARSE");
    }

    #[test]
    fn test_deferred_assignment_outside_beat_is_rejected() {
        let source = r#"
테스트:움직씨 = {
    살림.x <- 1 미루기.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("deferred assignment outside beat");
        assert_eq!(err.code(), "E_PARSE_DEFERRED_ASSIGN_OUTSIDE_BEAT");
        assert!(err.message.contains("미루기"));
        assert!(err.message.contains("덩이"));
    }

    #[test]
    fn test_hook_every_madi_parses() {
        let source = r#"
테스트:움직씨 = {
    (매마디)마다 {
        살림.x <- 1.
    }.
}
"#;
        let program = parse(source, "test.ddoni").expect("hook parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        let Some(Stmt::Hook { kind, .. }) = body.stmts.first() else {
            panic!("hook expected");
        };
        assert!(matches!(kind, HookKind::EveryMadi));
    }

    #[test]
    fn test_hook_every_n_madi_parses() {
        let source = r#"
테스트:움직씨 = {
    (3마디)마다 {
        살림.x <- 1.
    }.
}
"#;
        let program = parse(source, "test.ddoni").expect("hook parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        let Some(Stmt::Hook { kind, .. }) = body.stmts.first() else {
            panic!("hook expected");
        };
        assert!(matches!(kind, HookKind::EveryNMadi(3)));
    }

    #[test]
    fn test_hook_every_n_madi_zero_is_rejected() {
        let source = r#"
테스트:움직씨 = {
    (0마디)마다 {
        살림.x <- 1.
    }.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("hook interval zero");
        assert!(err.message.contains("양의 정수"));
    }

    #[test]
    fn test_hook_every_madi_colon_is_rejected() {
        let source = r#"
테스트:움직씨 = {
    (매마디)마다: {
        살림.x <- 1.
    }.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("hook colon");
        assert!(err.message.contains("':' 없이"));
    }

    #[test]
    fn test_hook_start_and_end_parse() {
        let source = r#"
테스트:움직씨 = {
    (시작)할때 {
        살림.x <- 1.
    }.
    (끝)할때 {
        살림.y <- 2.
    }.
}
"#;
        let program = parse(source, "test.ddoni").expect("hook start/end parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        let Some(Stmt::Hook { kind, .. }) = body.stmts.first() else {
            panic!("start hook expected");
        };
        assert!(matches!(kind, HookKind::Start));
        let Some(Stmt::Hook { kind, .. }) = body.stmts.get(1) else {
            panic!("end hook expected");
        };
        assert!(matches!(kind, HookKind::End));
    }

    #[test]
    fn test_hook_start_alias_choeum_normalizes_to_sijak() {
        let source = r#"
테스트:움직씨 = {
    (처음)할때 {
        살림.x <- 1.
    }.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1)
            .expect("normalize start alias");
        assert!(normalized.contains("(시작)할때 {"));
        assert!(!normalized.contains("(처음)할때 {"));
    }

    #[test]
    fn test_hook_condition_becomes_parses() {
        let source = r#"
테스트:움직씨 = {
    (살림.x > 0)이 될때 {
        살림.y <- 1.
    }.
}
"#;
        let program = parse(source, "test.ddoni").expect("condition hook parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        assert!(matches!(body.stmts.first(), Some(Stmt::HookWhenBecomes { .. })));
    }

    #[test]
    fn test_hook_condition_while_parses() {
        let source = r#"
테스트:움직씨 = {
    (살림.x > 0)인 동안 {
        살림.y <- 1.
    }.
}
"#;
        let program = parse(source, "test.ddoni").expect("condition while hook parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        assert!(matches!(body.stmts.first(), Some(Stmt::HookWhile { .. })));
    }
}
