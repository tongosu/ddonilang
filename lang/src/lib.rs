// ddonirang-lang/src/lib.rs
// 또니랑 언어 코어 라이브러리
//
// Phase 1 완료:
// - Lexer: 한국어 토큰화
// - Parser: SOV 어순 재귀 하강 파서
// - Normalizer: N1 레벨 정본화
// - AST: 비손실 정본 구조

pub mod ast;
pub mod canonicalizer;
pub mod dialect;
pub mod lexer;
pub mod parser;
pub mod normalizer;
pub mod stdlib;
pub mod runtime;
pub mod surface;
pub mod term_map;

pub use ast::*;
pub use canonicalizer::{canonicalize, CanonicalizeReport, LintWarning};
pub use dialect::DialectConfig;
pub use lexer::{Lexer, Token, TokenKind, LexError};
pub use parser::{Parser, ParseError, ParseMode};
pub use normalizer::{Normalizer, NormalizationLevel, normalize};
pub use stdlib::{FunctionSig, input_function_sigs, list_function_sigs, minimal_stdlib_sigs, string_function_sigs};
pub use runtime::{
    InputState, RuntimeError, Value, input_just_pressed, input_pressed, list_add, list_len,
    list_new, list_nth, list_remove, list_set, string_concat, string_join, string_len, string_split,
};
pub use surface::{surface_form, SurfaceError};

/// 편리 함수: 소스 → AST
pub fn parse(source: &str, file_path: &str) -> Result<CanonProgram, ParseError> {
    parse_with_mode(source, file_path, ParseMode::Strict)
}

/// 편리 함수: 소스 → AST (모드 지정)
pub fn parse_with_mode(source: &str, file_path: &str, mode: ParseMode) -> Result<CanonProgram, ParseError> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|e| ParseError {
            span: crate::ast::Span { start: e.pos, end: e.pos + 1 },
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
        assert!(normalized.contains("(10, 1) 더하기"));
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
        assert!(normalized.contains("(10, 없음) 더하기"));
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
        assert!(normalized.contains("(10, 5) 더하기"));
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
        assert!(normalized.contains("(10, 7) 더하기"));
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
        assert!(normalized.contains("(3, 1, 2) 합"));
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
        assert!(normalized.contains("(1~을, 3~가) 이동"));
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
        assert!(err.message.contains("모호합니다"));
        assert!(err.message.contains("핀=값"));
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
        assert!(normalized.contains("(2, 도구=1) 이동"));
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
        assert!(normalized.contains("값 <- (대상=보드, i=2) 차림.값."));
    }

    #[test]
    fn test_index_assign_normalizes_to_charim_set() {
        let source = r#"
테스트:셈씨 = {
    보드[2] <- 3.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("보드 <- (대상=보드, i=2, 값=3) 차림.바꾼값."));
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
        assert!(normalized.contains("보드 <- (대상=보드, i=2, 값=3) 차림.바꾼값."));
    }

    #[test]
    fn test_suffix_chain_unit_pin_josa() {
        let source = r#"
(거리:수~에서) 이동:셈씨 = {
    거리 돌려줘.
}

테스트:셈씨 = {
    (거리=100@m~에서) 이동.
}
"#;
        let args = first_call_args(source);
        assert_eq!(args.len(), 1);
        let arg = &args[0];
        assert_eq!(arg.josa.as_deref(), Some("에서"));
        assert_eq!(arg.resolved_pin.as_deref(), Some("거리"));
        assert!(matches!(arg.binding_reason, BindingReason::UserFixed));
        match &arg.expr.kind {
            ExprKind::Suffix { at: AtSuffix::Unit(unit), .. } => {
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
    (배경=@"그림/주인공.png"~으로) 보기.
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
    (거리=100~에서@m) 이동.
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
    (거리=100@m~에서~부터) 이동.
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
    assert!(err.message.contains("핀 고정은 핀=값"));
    }

    #[test]
    fn test_suffix_chain_fixed_pin_resolves_ambiguity() {
        let source = r#"
(시작:수~에서, 끝:수~에서?) 이동:셈씨 = {
    시작 돌려줘.
}

테스트:셈씨 = {
    (시작=100@m~에서) 이동.
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
    (끝=100@m~에서) 이동.
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
    fn test_setting_block_without_colon_is_error() {
        let source = r#"
테스트:셈씨 = {
    설정 { 화면: "기본". }.
    1 돌려줘.
}
"#;
        let _err = parse(source, "test.ddoni").expect_err("setting block without colon");
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

}
