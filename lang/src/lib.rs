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
pub mod currentline;
pub mod delivery;
pub mod dialect;
pub mod execution_mode;
pub mod frontdoor;
pub mod lexer;
pub mod migration;
pub mod model;
pub mod normalizer;
pub mod parser;
pub mod runtime;
pub mod source_coordinate;
pub mod stdlib;
pub mod surface;
pub mod term_map;

pub use age_gate::{age_not_available_error, AgeTarget};
pub use ast::*;
pub use canonicalizer::{canonicalize, CanonicalizeReport, LintWarning};
pub use currentline::{apply_currentline_cell, apply_currentline_cell_v25, CurrentLineResult};
pub use delivery::{
    canonical_provider_occurrences, plan_delivery_occurrence, plan_internal_delivery_occurrence,
    select_exactly_one_endpoint, AsyncSend, DeliveryOccurrence, DeliveryOccurrenceFailure,
    DeliveryOccurrenceIdentity, EndpointRoute, EndpointRouteFailure, EndpointRouteSemantic,
    EnqueueOutcome, ExecutionInstanceIdentity, HistoricalAsyncSend,
    InternalDeliveryEnvelopeSemantic, InternalDeliveryOccurrence,
    InternalDeliveryOccurrenceFailure, InternalProviderIdentityFailure, LogicalAsyncQueue,
    LogicalDeliveryPhase, LogicalWaitEdge, LogicalWaitGraph, LogicalWaitReceipt, MessageTransfer,
    PersistentDependencyId, ProviderDeclarationIdentity, ProviderIdentity, ProviderIdentityFailure,
    ProviderOwnerIdentity, ProviderRegistrationIdentity, QueueAdmission, ReactionProvider,
    ReactionProviderDescriptor, ReactionProviderPredicate, ReactionProviderRegistry,
    RendezvousSend, RouteIdentity, SenderIdentityPredicate, StagedDeliveryOutbox, WaitDomain,
    WaitNodeIdentity, WaitProgress, DEFAULT_ASYNC_QUEUE_CAPACITY,
};
pub use dialect::DialectConfig;
pub use execution_mode::{decide_execution_mode, ExecutionMode, ExecutionModeDecision};
pub use frontdoor::{
    connect_endpoint_relation_seum_rows, find_legacy_header, find_legacy_range_comment,
    find_legacy_root_hide_directive, find_legacy_root_surface, has_legacy_boim_surface,
    has_natural_foreach_surface, has_right_assignment_arrow, has_simple_assignment_arrow,
    normalize_for_lang_parity, owner_inner_seum_canon_rows, owner_state_symbol_table_rows,
    preprocess_frontdoor_source, validate_no_legacy_boim_surface, validate_no_legacy_header,
    validate_no_legacy_range_comment, validate_no_legacy_root_hide_directive,
    validate_no_legacy_root_surface, wrap_lang_parity_source,
};
pub use lexer::{LexError, Lexer, Token, TokenKind};
pub use migration::{
    historical_async_migration_evidence, HistoricalAsyncMigrationEvidence,
    HISTORICAL_ASYNC_MIGRATION_DIAGNOSTIC_CODE,
};
pub use model::{
    fixed_default_precise_recipe, has_model_surface, model_ir_candidate_json, parse_model_program,
    parse_model_program_with_source_file, validate_model_capabilities, validate_model_ir_bindings,
    AuthoredIdentity, ConnectionClause, ConnectionMode, DerivativeIdentity, ExactPreludeBinding,
    ExecutionContractProfile, ExecutionContractProvenance, ExecutionEvidencePolicy,
    ExecutionExternalIngressPolicy, ExecutionPerformancePolicy, ExecutionReplayPolicy,
    ExecutionReproScope, ExecutionResourcePolicy, ExecutionResultContract, Initialization,
    LoweredRelationNode, LoweredRelationTarget, ModelBodyItem, ModelConnection, ModelDeclaration,
    ModelDefinition, ModelDiagnostic, ModelEvent, ModelEventKind, ModelExpr, ModelExprKind,
    ModelExprNode, ModelInstance, ModelIr, ModelParseError, ModelProgram, ModelReset, ModelSpan,
    ModelType, ModelValue, ModelView, PrimaryRole, RecordField, RunPin, RunRequest,
    SolveMethodProvenance, SolveMethodValue, SolveRecipe, TypedExecutionContractField,
    EXACT_ACYCLIC_RELATION_BACKEND_ID, EXECUTION_CONTRACT_FIELD_EVIDENCE,
    EXECUTION_CONTRACT_FIELD_EXTERNAL_INGRESS, EXECUTION_CONTRACT_FIELD_PERFORMANCE,
    EXECUTION_CONTRACT_FIELD_REPLAY, EXECUTION_CONTRACT_FIELD_REPRO_SCOPE,
    EXECUTION_CONTRACT_FIELD_RESOURCE, EXECUTION_CONTRACT_FIELD_RESULT,
    EXECUTION_CONTRACT_SCHEMA_CANDIDATE, FIXED_DEFAULT_PRECISE_RECIPE_NAME,
    MODEL_IR_SCHEMA_CANDIDATE, STANDARD_REPRO_PROFILE_IDENTITY,
};
pub use normalizer::{normalize, normalize_expression_identity, NormalizationLevel, Normalizer};
pub use parser::{resolve_known_call_target, ParseError, ParseMode, Parser};
pub use runtime::{
    input_just_pressed, input_pressed, list_add, list_len, list_new, list_nth, list_remove,
    list_set, string_concat, string_join, string_len, string_split, InputState, RuntimeError,
    Value,
};
pub use source_coordinate::{utf16_position, Utf16Position};
pub use stdlib::{
    canonicalize_type_alias, input_function_sigs, list_function_sigs, minimal_stdlib_sigs,
    string_function_sigs, FunctionSig,
};
pub use surface::{surface_form, SurfaceError};

/// 편리 함수: 소스 → AST
pub fn parse(source: &str, file_path: &str) -> Result<CanonProgram, ParseError> {
    parse_with_mode(source, file_path, ParseMode::SuccessorDefault)
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

    validate_rd203_forbidden_field_surfaces(&tokens)?;

    let mut parser = Parser::new_with_mode(tokens, mode);
    parser.parse_program(source.to_string(), file_path.to_string())
}

/// v25 source epoch: Definition `:=` and Binding/Rebinding `<-` are distinct.
pub fn parse_v25(source: &str, file_path: &str) -> Result<CanonProgram, ParseError> {
    parse_v25_with_mode(source, file_path, ParseMode::Strict)
}

pub fn parse_v25_with_mode(
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
    let mut parser = Parser::new_v25_with_mode(tokens, mode);
    parser.parse_program(source.to_string(), file_path.to_string())
}

fn validate_rd203_forbidden_field_surfaces(tokens: &[Token]) -> Result<(), ParseError> {
    for (index, token) in tokens.iter().enumerate() {
        if matches!(&token.kind, TokenKind::Josa(josa) if josa == "의") {
            if possessive_is_existing_message_sender(tokens, index) {
                continue;
            }
            return Err(ParseError {
                span: Span::new(token.span.start, token.span.end),
                message: "E_FIELD_POSSESSIVE_SURFACE_FORBIDDEN: field 접근은 붙여 쓴 `A.B`만 허용합니다; `A의 B`와 `~의` pin 별칭은 지원하지 않습니다".to_string(),
            });
        }
    }
    Ok(())
}

fn possessive_is_existing_message_sender(tokens: &[Token], possessive_index: usize) -> bool {
    tokens
        .iter()
        .skip(possessive_index + 1)
        .take_while(|token| {
            !matches!(
                token.kind,
                TokenKind::Dot | TokenKind::Question | TokenKind::Bang | TokenKind::Eof
            )
        })
        .any(|token| {
            matches!(
                token.kind,
                TokenKind::AsyncRightArrow | TokenKind::DoubleArrow
            )
        })
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

pub fn parse_v25_and_normalize(
    source: &str,
    file_path: &str,
    level: NormalizationLevel,
) -> Result<String, ParseError> {
    let mut program = parse_v25(source, file_path)?;
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
    fn decimal_literal_keeps_q32_32_raw_value() {
        let program = parse("검사:셈씨 = { 3.14 돌려줘. }", "decimal.ddoni").expect("parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().expect("body");
        let Stmt::Return { value, .. } = &body.stmts[0] else {
            panic!("return expected");
        };
        let ExprKind::Literal(Literal::Fixed64(value)) = &value.kind else {
            panic!("fixed literal expected");
        };
        assert_eq!(value.raw_i64(), 13_486_197_309);
    }

    #[test]
    fn rd203_field_access_keeps_exact_segment_span() {
        let source = "매틱:움직씨 = { 사람 <- (이름: \"또니\"). 값 <- 사람.이름. }";
        let program = parse(source, "field.ddn").expect("A.B parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().expect("body");
        let Stmt::Mutate { value, .. } = &body.stmts[1] else {
            panic!("field assignment");
        };
        let ExprKind::FieldAccess {
            field, field_span, ..
        } = &value.kind
        else {
            panic!("field access");
        };
        assert_eq!(field, "이름");
        assert_eq!(&source[field_span.start..field_span.end], "이름");
    }

    #[test]
    fn rd203_missing_field_fails_at_field_segment() {
        let source = "매틱:움직씨 = { 사람 <- (이름: \"또니\"). 값 <- 사람.나이. }";
        let error = parse(source, "field.ddn").expect_err("missing field");
        assert_eq!(error.code(), "E_FIELD_MISSING");
        assert_eq!(&source[error.span.start..error.span.end], "나이");
    }

    #[test]
    fn rd203_non_record_field_target_fails_at_field_segment() {
        let source = "매틱:움직씨 = { 수 <- 10. 값 <- 수.이름. }";
        let error = parse(source, "field.ddn").expect_err("non-record field target");
        assert_eq!(error.code(), "E_FIELD_TARGET_TYPE");
        assert_eq!(&source[error.span.start..error.span.end], "이름");
    }

    #[test]
    fn rd203_possessive_and_tilde_ui_surfaces_are_rejected_by_shared_frontdoor() {
        for source in [
            "매틱:움직씨 = { 사람 <- (이름: \"또니\"). 사람의 이름. }",
            "(대상:묶음 ~의) 이름:움직씨 = { 대상.이름 돌아보기. }",
        ] {
            let error = parse(source, "field.ddn").expect_err("forbidden field surface");
            assert_eq!(error.code(), "E_FIELD_POSSESSIVE_SURFACE_FORBIDDEN");
            assert_eq!(&source[error.span.start..error.span.end], "의");
        }
    }

    #[test]
    fn existing_message_sender_possessive_reaches_typed_send_parser_only() {
        let source = "매틱:움직씨 = { (철수)의 7 ~> 관제탑. }";
        let program = parse(source, "sender.ddn").expect("typed sender message");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let Stmt::Send {
            sender, transfer, ..
        } = &seed.body.as_ref().expect("body").stmts[0]
        else {
            panic!("typed send statement expected")
        };
        assert!(sender.is_some());
        assert_eq!(*transfer, MessageTransfer::ASYNC);
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
    fn relation_eq_infix_parses_inside_seed_body() {
        let source = r#"
검사:셈씨 = {
    rel <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
}
"#;
        let program = parse(source, "relation_eq.ddoni").expect("parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().expect("body");
        let Stmt::Mutate { value, .. } = &body.stmts[0] else {
            panic!("mutate expected");
        };
        let ExprKind::Infix { op, .. } = &value.kind else {
            panic!("relation infix expected");
        };
        assert_eq!(op, "=:=");
    }

    #[test]
    fn prime_derivative_identifiers_parse_and_normalize() {
        let source = r#"
검사:셈씨 = {
    위치' <- 속도.
    위치'' 보여주기.
}
"#;
        let normalized =
            parse_and_normalize(source, "prime.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("위치' <- 속도"));
        assert!(normalized.contains("위치'' 보여주기"));
    }

    #[test]
    fn triple_prime_derivative_identifier_is_rejected() {
        let source = r#"
검사:셈씨 = {
    위치''' <- 1.
}
"#;
        let err = parse(source, "prime_bad.ddoni").expect_err("triple prime rejected");
        assert!(err.message.contains("알 수 없는 문자"));
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
    fn test_named_seed_kind_accepts_imja_in_kind_position() {
        let source = r#"
기계:임자 = {
}
"#;
        let program = parse(source, "test.ddoni").expect("seed kind imja");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        assert_eq!(seed.canonical_name, "기계");
        assert_eq!(seed.seed_kind, SeedKind::Named("임자".to_string()));
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
    fn test_manyak_if_parsing_and_normalization() {
        let source = r#"
(x:수) 판정:셈씨 = {
    만약 x < 0 이라면 {
        "음수" 돌려줘.
    }.
    아니면 {
        "양수" 돌려줘.
    }.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("만약 x < 0 이라면"));
        assert!(!normalized.contains(" 일때 "));
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
    fn imja_receive_and_signal_send_are_ast_nodes() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
    값 돌려줘.
}

관제탑:임자 = {
    첫알림을 받으면 {
        "받음" 보여주기.
    }.
}

테스트:움직씨 = {
    ((값=1) 첫알림) ~~> 관제탑.
}
"#;
        let program = parse(source, "test.ddoni").expect("parse");
        let TopLevelItem::SeedDef(imja) = &program.items[1];
        let body = imja.body.as_ref().expect("imja body");
        assert!(matches!(
            body.stmts.first(),
            Some(Stmt::Receive {
                kind: Some(kind),
                binding: None,
                condition: None,
                ..
            }) if kind == "첫알림"
        ));
        let TopLevelItem::SeedDef(test_seed) = &program.items[2];
        let body = test_seed.body.as_ref().expect("test body");
        assert!(matches!(
            body.stmts.first(),
            Some(Stmt::Send {
                transfer: MessageTransfer::Rendezvous(_),
                ..
            })
        ));
    }

    #[test]
    fn successor_async_direction_pair_shares_typed_identity_and_canon() {
        let source = r#"
(값:수) 첫알림:알림씨 = { 값 돌려줘. }
관제탑:임자 = { 첫알림을 받으면 { "받음" 보여주기. }. }
테스트:움직씨 = {
    관제탑 <~ ((값=1) 첫알림).
    ((값=2) 첫알림) ~> 관제탑.
}
"#;
        let program = parse(source, "successor_async_pair.ddoni").expect("parse");
        let TopLevelItem::SeedDef(test_seed) = &program.items[2];
        let body = test_seed.body.as_ref().expect("test body");
        assert_eq!(body.stmts.len(), 2);
        for stmt in &body.stmts {
            assert!(matches!(
                stmt,
                Stmt::Send {
                    transfer: MessageTransfer::Async(_),
                    ..
                }
            ));
        }
        let canonical = normalize(&program, NormalizationLevel::N1);
        assert_eq!(canonical.matches(" ~> ").count(), 2);
        assert!(!canonical.contains(" <~ "));
    }

    #[test]
    fn imja_owner_inner_seum_and_seongjil_parse_boundary() {
        let source = r#"
공:임자 = {
    성질 {
        위치: 수 <- 0.
        속도: 수 <- 0.
    }.
    세움 {
        위치' =:= 속도.
    }.
    힘가해짐을 받으면 {
        속도 <- 속도 + 힘.
    }.
}
"#;
        let program = parse(source, "owner_inner_seum.ddoni").expect("parse owner inner seum");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        assert_eq!(seed.canonical_name, "공");
        assert_eq!(seed.seed_kind, SeedKind::Named("임자".to_string()));
        let body = seed.body.as_ref().expect("owner body");
        assert!(matches!(body.stmts[0], Stmt::DeclBlock { .. }));
        let Stmt::Expr { expr, .. } = &body.stmts[1] else {
            panic!("owner-local seum expression expected");
        };
        let ExprKind::Assertion(assertion) = &expr.kind else {
            panic!("assertion expected");
        };
        assert_eq!(assertion.canon, "세움{\n    위치' =:= 속도.\n}");
        assert!(matches!(body.stmts[2], Stmt::Receive { .. }));

        let rows = owner_inner_seum_canon_rows(source).expect("owner inner seum rows");
        assert_eq!(rows, vec!["세움{\n    위치' =:= 속도.\n}"]);
    }

    #[test]
    fn imja_owner_state_symbol_table_rows_stay_owner_scoped() {
        let source = r#"
공:임자 = {
    성질 {
        위치: 수 <- 0.
        속도: 수 <- 0.
        이름: 글 = "공".
    }.
    세움 {
        위치' =:= 속도.
    }.
    힘가해짐을 받으면 {
        속도 <- 속도 + 힘.
    }.
}

바깥:움직씨 = {
    채비 {
        위치: 수 <- 9.
    }.
}
"#;
        let rows = owner_state_symbol_table_rows(source).expect("owner state rows");
        assert_eq!(
            rows,
            vec![
                "owner=공;symbol=위치;type=수;kind=state;initializer=0",
                "owner=공;symbol=속도;type=수;kind=state;initializer=0",
                "owner=공;symbol=이름;type=글;kind=constant;initializer=\"공\"",
            ]
        );
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
    fn test_amount_of_substance_unit_suffix_is_canonical_and_not_mass() {
        let source = r#"
테스트:셈씨 = {
    물질량 <- 2@mol.
    물질량 돌려줘.
}
"#;
        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1).unwrap();
        assert!(normalized.contains("물질량 <- 2@mol."));

        let mismatch = r#"
테스트:셈씨 = {
    값 <- 2@mol + 2@kg.
}
"#;
        let err = parse(mismatch, "test.ddoni").expect_err("mol must not alias mass");
        assert!(err.message.contains("단위 차원이 다릅니다"));
    }

    #[test]
    fn test_fixed64_literal_overflow_fails_closed_before_unit_suffix() {
        let source = r#"
테스트:셈씨 = {
    (입자수: 602214076000000000000000@mol) 보임.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("Fixed64 overflow must fail");
        assert!(
            err.message.contains("E_INTEGER_LITERAL_RANGE")
                || err.message.contains("E_FIXED64_LITERAL_RANGE")
        );

        let fixed64_overflow = r#"
테스트:셈씨 = {
    (물질량: 3000000000@mol) 보임.
}
"#;
        let err =
            parse(fixed64_overflow, "test.ddoni").expect_err("Fixed64 range overflow must fail");
        assert!(err.message.contains("E_FIXED64_LITERAL_RANGE"));
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
    fn test_right_assign_simple_surface_normalizes_to_left_assign() {
        let source = r#"
매마디:움직씨 = {
    값 <- 0.
    3 -> 값.
}
"#;
        let normalized =
            parse_and_normalize(source, "right_assign_simple.ddn", NormalizationLevel::N1)
                .expect("right assignment normalizes");
        assert!(normalized.contains("값 <- 3."));
        assert!(!normalized.contains("3 -> 값."));
    }

    #[test]
    fn test_bidirectional_assignment_rejects_literal_target_with_same_code_and_target_span() {
        for source in [
            "테스트:셈씨 = {\n  값 <- 0.\n  3 <- 값.\n}\n",
            "테스트:셈씨 = {\n  값 <- 0.\n  값 -> 3.\n}\n",
        ] {
            let err = parse(source, "literal_assignment_target.ddn")
                .expect_err("literal target must fail closed");
            assert_eq!(err.code(), "E_PARSE_EXPECTED_TARGET");
            assert_eq!(&source[err.span.start..err.span.end], "3");
        }
    }

    #[test]
    fn test_assignment_chains_are_not_implicitly_enabled() {
        for source in [
            "a:수 <- 1.\nb:수 <- 2.\nc:수 <- 3.\na -> b -> c.\n",
            "a:수 <- 1.\nb:수 <- 2.\nc:수 <- 3.\na -> b <- c.\n",
        ] {
            assert!(parse(source, "assignment_chain.ddn").is_err());
        }
    }

    #[test]
    fn test_spaced_assignment_arrows_are_not_recovered() {
        for source in ["값:수 <- 0.\n값 < - 3.\n", "값:수 <- 0.\n3 - > 값.\n"] {
            assert!(parse(source, "spaced_assignment_arrow.ddn").is_err());
        }
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
    fn test_symbolic_relation_assertion_literal_parses() {
        let source = r#"
Test:셈씨 = {
    검사 <- 세움{
        (#ascii) 수식{x + y} =:= (#ascii) 수식{5}
    }.
}
"#;
        let program = parse(source, "assertion_relation.ddoni").expect("assertion relation parse");
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
        assert_eq!(assertion.canon, "세움{수식관계: x + y =:= 5}");
    }

    #[test]
    fn test_seumssi_v1b_surface_alias_parses_to_canonical_seum() {
        let source = r#"
Test:셈씨 = {
    검사 <- 세움씨{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
}
"#;
        let program = parse(source, "seumssi_v1b.ddoni").expect("seumssi alias parse");
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
        assert!(!assertion.canon.starts_with("세움씨{"));
    }

    #[test]
    fn test_seumssi_v1b_relation_bridge_canon_matches_seum() {
        let source = r#"
Test:셈씨 = {
    검사 <- 세움씨{
        (#ascii) 수식{x + y} =:= (#ascii) 수식{5}
    }.
}
"#;
        let program = parse(source, "seumssi_relation_v1b.ddoni").expect("seumssi relation parse");
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
        assert_eq!(assertion.canon, "세움{수식관계: x + y =:= 5}");
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
    fn test_decl_block_in_repeat_body_is_rejected() {
        let source = r#"
테스트:합 = {
    반복 {
        채비 { 값:수 <- 0. }.
    }.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("decl block in repeat body");
        assert!(err.message.contains("E_CHAEBI_IN_LOOP"));
    }

    #[test]
    fn test_decl_block_in_hook_body_is_rejected() {
        let source = r#"
테스트:합 = {
    (처음)할때 {
        채비 { 값:수 <- 0. }.
    }.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("decl block in hook body");
        assert!(err.message.contains("E_CHAEBI_IN_LOOP"));
    }

    #[test]
    fn test_top_level_chaebi_reassign_warns() {
        let source = r#"
테스트:합 = {
    채비 { 값:수 <- 0. }.
    값 <- 1.
}
"#;
        let mut program = parse(source, "test.ddoni").expect("parse");
        let report = canonicalize(&mut program).expect("canonicalize");
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.code == "W_CHAEBI_REDUNDANT_TOP_REASSIGN"));
    }

    #[test]
    fn test_top_level_chaebi_derived_reassign_does_not_warn() {
        let source = r#"
테스트:합 = {
    채비 {
        점수:수 <- 72.
        통과함:참거짓 <- 거짓.
    }.
    통과함 <- 점수 >= 70.
    통과함 보여주기.
}
"#;
        let mut program = parse(source, "test.ddoni").expect("parse");
        let report = canonicalize(&mut program).expect("canonicalize");
        assert!(!report
            .warnings
            .iter()
            .any(|warning| warning.code == "W_CHAEBI_REDUNDANT_TOP_REASSIGN"));
    }

    #[test]
    fn test_decl_block_item_with_maegim_suffix_parses() {
        let source = r#"
테스트:셈씨 = {
    채비 {
        데이터길이:수 <- (12) 매김 {
            범위: 4..40.
            간격: 1.
        }.
    }.
}
"#;
        let program = parse(source, "test.ddoni").expect("decl block with 매김 parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().expect("body");
        match &body.stmts[0] {
            Stmt::DeclBlock { items, .. } => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].name, "데이터길이");
            }
            _ => panic!("decl block expected"),
        }
    }

    #[test]
    fn test_decl_block_item_with_condition_alias_suffix_parses() {
        let source = r#"
테스트:셈씨 = {
    채비 {
        데이터길이:수 <- (12) 조건 {
            범위: 4..40.
            간격: 1.
        }.
    }.
}
"#;
        let program = parse(source, "test.ddoni").expect("decl block with 조건 parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().expect("body");
        match &body.stmts[0] {
            Stmt::DeclBlock { items, .. } => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].name, "데이터길이");
            }
            _ => panic!("decl block expected"),
        }
    }

    #[test]
    fn test_decl_block_none_entry_reports_actionable_error() {
        let source = r#"
테스트:셈씨 = {
    채비 {
        없음.
    }.
}
"#;
        let err = parse(source, "test.ddoni").expect_err("decl block none entry must fail");
        assert!(err.message.contains("채비 항목은 `이름:타입 <- 값.` 형태"));
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
    fn test_public_observation_preserves_order_and_source_spans() {
        let source = r#"
테스트:셈씨 = {
    (시간: 1, 각도: 1 + (2 * 3), 각속도: 4) 보임.
    1 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("public observation parse");
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
            Stmt::PublicObservation { fields, .. } => {
                assert_eq!(
                    fields
                        .iter()
                        .map(|field| field.name.as_str())
                        .collect::<Vec<_>>(),
                    vec!["시간", "각도", "각속도"]
                );
                for field in fields {
                    assert_eq!(
                        &source[field.name_span.start..field.name_span.end],
                        field.name
                    );
                    assert!(field.span.start <= field.name_span.start);
                    assert!(field.span.end >= field.value.span.end);
                }
            }
            other => panic!("public observation expected, got {other:?}"),
        }
    }

    #[test]
    fn test_public_observation_preserves_authored_field_label_keyword_alias() {
        let source = r#"
테스트:셈씨 = {
    (날짜: "2099-01-01", 사건: "중립 fixture") 보임.
    1 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("public observation parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().expect("body");
        let Stmt::PublicObservation { fields, .. } = &body.stmts[0] else {
            panic!("public observation expected");
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            vec!["날짜", "사건"]
        );
        assert_eq!(
            &source[fields[1].name_span.start..fields[1].name_span.end],
            "사건"
        );

        let normalized = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1)
            .expect("canonicalization");
        assert!(normalized.contains("(날짜: \"2099-01-01\", 사건: \"중립 fixture\") 보임."));
        assert_eq!(
            parse_and_normalize(&normalized, "test.ddoni", NormalizationLevel::N1)
                .expect("second canonicalization"),
            normalized
        );
    }

    #[test]
    fn test_public_observation_unary_shorthand_builds_named_field() {
        let source = r#"
테스트:셈씨 = {
    (각도) 보임.
    1 돌려줘.
}
"#;
        let program = parse(source, "test.ddoni").expect("unary public observation parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        let body = seed.body.as_ref().expect("테스트 body");
        let Stmt::PublicObservation { fields, .. } = &body.stmts[0] else {
            panic!("public observation expected");
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "각도");
        assert!(matches!(&fields[0].value.kind, ExprKind::Var(name) if name == "각도"));
    }

    #[test]
    fn test_public_observation_canon_roundtrip_is_ordered_and_idempotent() {
        let source = r#"
테스트:셈씨 = {
    (시간: 1, 각도: 1 + (2 * 3), 각속도: 4) 보임.
    (각도) 보임.
    1 돌려줘.
}
"#;
        let once = parse_and_normalize(source, "test.ddoni", NormalizationLevel::N1)
            .expect("first canonicalization");
        assert!(once.contains("(시간: 1, 각도: 1 + 2 * 3, 각속도: 4) 보임."));
        assert!(once.contains("(각도: 각도) 보임."));
        assert!(!once.contains("보임 {"));
        assert!(!once.contains("project.obs."));
        assert!(!once.contains("output_"));

        let twice = parse_and_normalize(&once, "test.ddoni", NormalizationLevel::N1)
            .expect("second canonicalization");
        assert_eq!(once, twice);

        let reparsed = parse(&twice, "test.ddoni").expect("canonical output reparses");
        let TopLevelItem::SeedDef(seed) = &reparsed.items[0];
        let body = seed.body.as_ref().expect("body");
        let Stmt::PublicObservation { fields, .. } = &body.stmts[0] else {
            panic!("public observation expected");
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            vec!["시간", "각도", "각속도"]
        );
    }

    #[test]
    fn test_legacy_boim_block_has_dedicated_removal_error() {
        for legacy in ["보임 { y축: 값. }.", "보임: { y축: 값. }."] {
            let source = format!("테스트:셈씨 = {{\n    {legacy}\n    1 돌려줘.\n}}\n");
            let err = parse(&source, "test.ddoni").expect_err("legacy boim rejected");
            assert_eq!(err.code(), "E_BOIM_LEGACY_REMOVED");
        }
    }

    #[test]
    fn test_public_observation_rejects_invalid_field_forms() {
        let cases = [
            ("() 보임.", "E_BOIM_EMPTY_PACK"),
            ("(각도 = 1) 보임.", "E_BOIM_FIXED_BINDING_FORBIDDEN"),
            ("(각도: 1, 각도: 2) 보임.", "E_BOIM_DUPLICATE_FIELD"),
            ("(1 + 2) 보임.", "E_BOIM_FIELD_NAME_REQUIRED"),
            ("(각도, 시간: 1) 보임.", "E_BOIM_FIELD_NAME_REQUIRED"),
        ];
        for (surface, code) in cases {
            let source = format!("테스트:셈씨 = {{\n    {surface}\n    1 돌려줘.\n}}\n");
            let err = parse(&source, "test.ddoni").expect_err(surface);
            assert_eq!(err.code(), code, "{surface}: {}", err.message);
        }
    }

    #[test]
    fn test_public_observation_duplicate_diagnostic_names_exact_korean_span() {
        let source = "테스트:셈씨 = {\n    (각도: 1, 각도: 2) 보임.\n    1 돌려줘.\n}\n";
        let second = source.rfind("각도").expect("second field");
        let err = parse(source, "test.ddoni").expect_err("duplicate field");
        assert_eq!(err.code(), "E_BOIM_DUPLICATE_FIELD");
        assert!(err.message.contains("`각도`"));
        assert_eq!(
            (err.span.start, err.span.end),
            (second, second + "각도".len())
        );
        assert_eq!(&source[err.span.start..err.span.end], "각도");
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
    fn natural_iteration_lowers_to_explicit_foreach_and_injects_stable_pin() {
        let source = r#"
(값:수) 기록:움직씨 = {
    값 보여주기.
}
테스트:움직씨 = {
    채비 {
        목록:(수)차림 <- (1, 2, 3) 차림.
    }.
    목록마다 {
        () 기록하기.
    }.
}
"#;
        let program = parse(source, "natural_iteration.ddn").expect("natural iteration parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        let Stmt::ForEach {
            item,
            item_type,
            iterable,
            body: loop_body,
            ..
        } = &body.stmts[1]
        else {
            panic!("short mada must lower to ForEach");
        };
        assert_eq!(item, "목록");
        assert!(matches!(item_type, Some(TypeRef::Named(name)) if name == "수"));
        assert!(matches!(&iterable.kind, ExprKind::Var(name) if name == "목록"));
        let Stmt::Expr { expr, .. } = &loop_body.stmts[0] else {
            panic!("loop call");
        };
        let ExprKind::Call { args, func } = &expr.kind else {
            panic!("loop call expression");
        };
        assert_eq!(func, "기록기");
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].resolved_pin.as_deref(), Some("값"));
        assert_eq!(
            format!("{:?}", args[0].binding_reason),
            "ContextItemInjected"
        );
        assert!(matches!(&args[0].expr.kind, ExprKind::Var(name) if name == "목록"));

        let normalized = normalize(&program, NormalizationLevel::N1);
        assert!(normalized.contains("(목록:수) 목록에 대해"));
        assert!(normalized.contains("목록:값 기록기"));
        assert!(!normalized.contains("목록마다"));
    }

    #[test]
    fn natural_iteration_applies_defaults_before_current_item() {
        let source = r#"
(값:수, 배수:수=2) 기록:움직씨 = {
    값 보여주기.
}
테스트:움직씨 = {
    채비 { 목록:(수)차림 <- (1) 차림. }.
    목록마다 { () 기록하기. }.
}
"#;
        let normalized = parse_and_normalize(
            source,
            "natural_iteration_default.ddn",
            NormalizationLevel::N1,
        )
        .expect("default then current item");
        assert!(normalized.contains("목록:값"));
        assert!(normalized.contains("2:배수"));
    }

    #[test]
    fn natural_iteration_rejects_ambiguous_current_item_pins() {
        let source = r#"
(왼쪽:수, 오른쪽:수) 기록:움직씨 = { 없음. }
테스트:움직씨 = {
    채비 { 목록:(수)차림 <- (1) 차림. }.
    목록마다 { () 기록하기. }.
}
"#;
        let err = parse(source, "natural_iteration_ambiguous.ddn")
            .expect_err("two missing pins must fail closed");
        assert_eq!(err.code(), "E_CURRENT_ITEM_AMBIGUOUS");
        assert!(err.message.contains("왼쪽"));
        assert!(err.message.contains("오른쪽"));
    }

    #[test]
    fn natural_iteration_rejects_unsafe_current_item_type() {
        let source = r#"
(문장:글) 기록:움직씨 = { 없음. }
테스트:움직씨 = {
    채비 { 목록:(수)차림 <- (1) 차림. }.
    목록마다 { () 기록하기. }.
}
"#;
        let err = parse(source, "natural_iteration_type.ddn")
            .expect_err("mismatched current item type must fail closed");
        assert_eq!(err.code(), "E_CURRENT_ITEM_TYPE_MISMATCH");
        assert!(err.message.contains("문장"));
    }

    #[test]
    fn natural_iteration_rejects_function_pin_as_ownership_unsafe() {
        let source = r#"
(변환:수 --> 수) 기록:움직씨 = { 없음. }
테스트:움직씨 = {
    채비 { 목록:(수)차림 <- (1) 차림. }.
    목록마다 { () 기록하기. }.
}
"#;
        let err = parse(source, "natural_iteration_ownership.ddn")
            .expect_err("function-valued ownership must never be inferred");
        assert_eq!(err.code(), "E_CURRENT_ITEM_OWNERSHIP_UNSAFE");
        assert!(err.message.contains("변환"));
    }

    #[test]
    fn natural_iteration_preserves_explicit_pin_binding_before_context_item() {
        let source = r#"
(값:수) 기록:움직씨 = { 값 보여주기. }
테스트:움직씨 = {
    채비 { 목록:(수)차림 <- (1) 차림. }.
    목록마다 { (7:값) 기록하기. }.
}
"#;
        let normalized = parse_and_normalize(
            source,
            "natural_iteration_explicit_pin.ddn",
            NormalizationLevel::N1,
        )
        .expect("explicit pin binding has priority and needs no contextual injection");
        assert!(normalized.contains("7:값"));
        assert!(!normalized.contains("목록:값 기록기"));
    }

    #[test]
    fn natural_iteration_rejects_zero_missing_required_pin() {
        let source = r#"
(배수:수=2) 기록:움직씨 = { 없음. }
테스트:움직씨 = {
    채비 { 목록:(수)차림 <- (1) 차림. }.
    목록마다 { () 기록하기. }.
}
"#;
        let err = parse(source, "natural_iteration_zero_pin.ddn")
            .expect_err("zero missing required pins must fail closed");
        assert_eq!(err.code(), "E_CURRENT_ITEM_NO_REQUIRED_PIN");
    }

    #[test]
    fn nested_natural_iteration_does_not_fall_back_to_outer_item() {
        let source = r#"
(값:수) 기록:움직씨 = { 값 보여주기. }
테스트:움직씨 = {
    채비 {
        바깥들:(수)차림 <- (1) 차림.
        안쪽들:(글)차림 <- ("가") 차림.
    }.
    바깥들마다 {
        안쪽들마다 { () 기록하기. }.
    }.
}
"#;
        let err = parse(source, "nested_natural_iteration.ddn")
            .expect_err("inner mismatch must not select outer item");
        assert_eq!(err.code(), "E_CURRENT_ITEM_TYPE_MISMATCH");
    }

    #[test]
    fn missing_argument_outside_iteration_is_not_context_injected() {
        let source = r#"
(값:수) 기록:움직씨 = { 값 보여주기. }
테스트:움직씨 = { () 기록하기. }
"#;
        let err =
            parse(source, "outside_iteration.ddn").expect_err("no current item outside iteration");
        assert_eq!(err.code(), "E_CALL_REQUIRED_PIN_MISSING");
        assert!(err.message.contains("값"));
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
    fn function_type_arrow_parses_simple_multi_and_higher_order_types() {
        let source = r#"
(영항:() --> 된씨, 단항:가씨 --> 나씨, 이항:(가씨, 나씨) --> 다씨, 고차:(가씨 --> 나씨) --> 다씨, 돌림:가씨 --> (나씨 --> 다씨))
함수타입확인:셈씨 = { 0. }
"#;
        let program = parse(source, "function_type.ddn").expect("function type parse");
        let TopLevelItem::SeedDef(seed) = &program.items[0];
        assert_eq!(seed.params.len(), 5);

        let TypeRef::Function { params, result } = &seed.params[0].type_ref else {
            panic!("zero-parameter function type expected");
        };
        assert!(params.is_empty());
        assert!(matches!(result.as_ref(), TypeRef::Named(name) if name == "된씨"));

        let TypeRef::Function { params, result } = &seed.params[1].type_ref else {
            panic!("simple function type expected");
        };
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0], TypeRef::Named(name) if name == "가씨"));
        assert!(matches!(result.as_ref(), TypeRef::Named(name) if name == "나씨"));

        let TypeRef::Function { params, result } = &seed.params[2].type_ref else {
            panic!("multi-parameter function type expected");
        };
        assert_eq!(params.len(), 2);
        assert!(matches!(result.as_ref(), TypeRef::Named(name) if name == "다씨"));

        let TypeRef::Function { params, result } = &seed.params[3].type_ref else {
            panic!("higher-order function type expected");
        };
        assert!(matches!(&params[0], TypeRef::Function { .. }));
        assert!(matches!(result.as_ref(), TypeRef::Named(name) if name == "다씨"));

        let TypeRef::Function { params, result } = &seed.params[4].type_ref else {
            panic!("function-returning function type expected");
        };
        assert_eq!(params.len(), 1);
        assert!(matches!(result.as_ref(), TypeRef::Function { .. }));
    }

    #[test]
    fn function_type_arrow_normalizes_and_roundtrips_canonically() {
        let source = r#"
(영항:()-->된씨, 단항:가씨-->나씨, 이항:(가씨,나씨)-->다씨, 고차:(가씨-->나씨)-->다씨, 돌림:가씨-->(나씨-->다씨))
함수타입확인:셈씨 = { 0. }
"#;
        let canonical = parse_and_normalize(source, "function_type.ddn", NormalizationLevel::N1)
            .expect("function type normalize");
        assert!(canonical.contains("영항:() --> 된씨"));
        assert!(canonical.contains("단항:가씨 --> 나씨"));
        assert!(canonical.contains("이항:(가씨, 나씨) --> 다씨"));
        assert!(canonical.contains("고차:(가씨 --> 나씨) --> 다씨"));
        assert!(canonical.contains("돌림:가씨 --> (나씨 --> 다씨)"));
        parse(&canonical, "function_type.canon.ddn").expect("canonical roundtrip parse");
    }

    #[test]
    fn function_type_arrow_rejects_unparenthesized_chain() {
        let source = "(함수:가씨 --> 나씨 --> 다씨) 확인:셈씨 = { 0. }";
        let err = parse(source, "function_type_chain.ddn").expect_err("non-associative arrow");
        assert!(err
            .message
            .contains("E_FUNCTION_TYPE_ARROW_NON_ASSOCIATIVE"));
    }

    #[test]
    fn function_type_rejects_right_assignment_arrow_alias() {
        let source = "(함수:가씨 -> 나씨) 확인:셈씨 = { 0. }";
        let err = parse(source, "function_type_alias.ddn").expect_err("right arrow alias");
        assert!(err
            .message
            .contains("E_FUNCTION_TYPE_RIGHT_ARROW_FORBIDDEN"));
    }

    #[test]
    fn function_type_arrow_rejects_value_position() {
        let source = "확인:셈씨 = { 값 --> 다른값. }";
        let err = parse(source, "function_type_value.ddn").expect_err("value position");
        assert!(err.message.contains("E_FUNCTION_TYPE_ARROW_VALUE_POSITION"));
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
        assert!(matches!(
            body.stmts.first(),
            Some(Stmt::HookWhenBecomes { .. })
        ));
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

    #[test]
    fn test_continue_loop_parses() {
        let source = r#"
테스트:움직씨 = {
    건너뛰기.
}
"#;
        let program = parse(source, "test.ddoni").expect("continue parse");
        let seed = program
            .items
            .iter()
            .filter_map(|item| match item {
                TopLevelItem::SeedDef(seed) => Some(seed),
            })
            .find(|seed| seed.canonical_name == "테스트")
            .expect("테스트 seed");
        let body = seed.body.as_ref().expect("테스트 body");
        assert!(matches!(
            body.stmts.first(),
            Some(Stmt::ContinueLoop { .. })
        ));
    }
}
