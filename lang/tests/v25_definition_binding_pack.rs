use ddonirang_lang::{
    canonicalize, normalize, parse, parse_v25, parse_v25_and_normalize, DeclKind,
    NormalizationLevel, SourceEpoch, Stmt, TopLevelItem,
};

const PUBLIC_SAFE_POSITIVE: &str = r#"검산:움직씨 := {
    합계:수 := 4.
    합계 <- 7.
    합계 보여주기.
}
"#;
const PUBLIC_SAFE_OLD_EQUALS: &str = "검산:움직씨 = {\n}\n";

#[test]
fn v25_braced_callable_terminator_belongs_to_definition_not_root_statement() {
    let first = "검산:움직씨 := { 값:수 := 1. }.";
    let source = format!("{first}\n다음:움직씨 := {{ 값:수 := 2. }}.\n");
    let mut program = parse_v25(&source, "종결자.ddn").expect("current dotted definition");
    assert_eq!(program.items.len(), 2);
    assert!(program.root_stmts.is_empty());
    let TopLevelItem::SeedDef(seed) = &program.items[0];
    assert_eq!(seed.span.end, first.len());
    assert_eq!(&source[seed.body.as_ref().unwrap().span.end..seed.span.end], ".");
    canonicalize(&mut program).unwrap();
    let canonical = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&canonical, "정본.ddn").unwrap();
    canonicalize(&mut reparsed).unwrap();
    assert_eq!(canonical, normalize(&reparsed, NormalizationLevel::N1));
    assert_eq!(reparsed.items.len(), 2);
    assert!(reparsed.root_stmts.is_empty());
    assert!(parse_v25("검산:움직씨 := {}", "기존.ddn").is_ok());
    for source in ["검산:움직씨 := {}..", "검산:움직씨 := {}. .", "검산:움직씨 = {}."] {
        assert!(parse_v25(source, "invalid.ddn").is_err());
    }
}
const PUBLIC_SAFE_BINDING_AS_DEFINITION: &str = r#"검산:움직씨 := {
    채비: { 합계:수 <- 4. }.
}
"#;

fn statement_categories(source: &str) -> (usize, usize) {
    let program = parse_v25(source, "pc01.ddn").expect("V25 source must parse");
    assert_eq!(program.source_epoch, SourceEpoch::V25);
    let TopLevelItem::SeedDef(seed) = &program.items[0];
    let body = seed.body.as_ref().expect("seed body");
    let definitions = body
        .stmts
        .iter()
        .filter(|stmt| matches!(stmt, Stmt::Definition { .. }))
        .count();
    let bindings = body
        .stmts
        .iter()
        .filter(|stmt| matches!(stmt, Stmt::Mutate { .. }))
        .count();
    (definitions, bindings)
}

#[test]
fn v25_definition_binding_pack_matches_oracle() {
    assert_eq!(statement_categories(PUBLIC_SAFE_POSITIVE), (1, 1));
    for (case, source, diagnostic) in [
        (
            "old-equals.ddn",
            PUBLIC_SAFE_OLD_EQUALS,
            "E_V25_DEFINITION_MIGRATION_REQUIRED",
        ),
        (
            "binding-as-definition.ddn",
            PUBLIC_SAFE_BINDING_AS_DEFINITION,
            "E_V25_DEFINITION_BINDING_CATEGORY",
        ),
    ] {
        let error = parse_v25(source, case).expect_err("case must fail closed");
        assert!(
            error.message.contains(diagnostic),
            "unexpected diagnostic for {case}: {}",
            error.message
        );
    }
}

#[test]
fn v25_parse_canon_parse_preserves_categories_and_symbols() {
    let before = statement_categories(PUBLIC_SAFE_POSITIVE);
    let mut program = parse_v25(PUBLIC_SAFE_POSITIVE, "positive.ddn").expect("parse");
    canonicalize(&mut program).expect("canonicalize");
    let canonical = normalize(&program, NormalizationLevel::N1);

    assert!(canonical.contains("검산:움직씨 :="));
    assert!(canonical.contains("합계:수 := 4."));
    assert!(canonical.contains("합계 <- 7."));
    assert!(!canonical.contains("합계:수 <- 4."));
    assert_eq!(statement_categories(&canonical), before);
}

#[test]
fn pre_v25_reader_is_explicitly_separate_from_v25_reader() {
    let legacy = parse(PUBLIC_SAFE_OLD_EQUALS, "legacy.ddn").expect("historical pre-V25 reader");
    assert_eq!(legacy.source_epoch, SourceEpoch::PreV25);

    let error =
        parse_v25(PUBLIC_SAFE_OLD_EQUALS, "v25.ddn").expect_err("V25 must reject silent alias");
    assert!(error
        .message
        .contains("E_V25_DEFINITION_MIGRATION_REQUIRED"));
}

const DEFINITION_MIGRATION_DIAGNOSTIC: &str = "E_V25_DEFINITION_MIGRATION_REQUIRED";

fn local_source(statements: &str) -> String {
    format!("계산:움직씨 := {{\n{statements}\n}}\n")
}

fn assert_migration_at_equals(source: &str) {
    let error =
        parse_v25(source, "typed_local_equals.ddn").expect_err("legacy equals must fail closed");
    assert_eq!(error.code(), DEFINITION_MIGRATION_DIAGNOSTIC);
    assert!(error.message.starts_with(DEFINITION_MIGRATION_DIAGNOSTIC));
    assert!(
        error.span.start < error.span.end,
        "diagnostic span must be present"
    );
}

#[test]
fn v25_typed_local_legacy_equals_targeted_matrix() {
    // typed-local, untyped-local, and top-level old Definition surfaces.
    assert_migration_at_equals(&local_source("    값:수 = 1."));
    assert_migration_at_equals(&local_source("    값 = 1."));
    assert_migration_at_equals("계산:움직씨 = {\n}\n");

    // Valid Definition and Binding remain distinct, including redefinition.
    let valid = local_source("    값:수 := 1.\n    값 <- 2.");
    assert_eq!(statement_categories(&valid), (1, 1));
    let redefined = local_source("    값:수 := 1.\n    값:수 := 2.");
    assert_eq!(statement_categories(&redefined), (2, 0));

    // `채비` constant `=` is DeclKind::Butbak, not a legacy Definition.
    let butbak = parse_v25(
        &local_source("    채비 {\n        상수:수 = 1.\n    }."),
        "butbak.ddn",
    )
    .expect("Butbak equals must preserve its separate meaning");
    let TopLevelItem::SeedDef(seed) = &butbak.items[0];
    let body = seed.body.as_ref().expect("seed body");
    let Stmt::DeclBlock { items, .. } = &body.stmts[0] else {
        panic!("expected declaration block");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind, DeclKind::Butbak);

    // Full type parsing precedes the Equals decision; no fixed-token lookahead.
    assert_migration_at_equals(&local_source("    표:(글, 수)짝맞춤 = 1."));
    assert_migration_at_equals(&local_source("    변환:(수, 글) --> 수 = 1."));

    // A malformed annotation remains a type error and is not relabeled.
    let malformed = parse_v25(&local_source("    값:(수, 글) = 1."), "malformed_type.ddn")
        .expect_err("malformed type must fail closed");
    assert_ne!(malformed.code(), DEFINITION_MIGRATION_DIAGNOSTIC);
    assert!(!malformed.message.contains(DEFINITION_MIGRATION_DIAGNOSTIC));
}

#[test]
fn v25_current_frontdoor_rejects_legacy_l11_brace_blocks_without_lexical_false_positives() {
    for source in [
        "쓰임 { 운동: \"./lib/운동\". }",
        "드러냄 { 값. }",
        "공개 { 값. }",
    ] {
        let error = parse_v25(source, "legacy_l11.ddn")
            .expect_err("current V25 frontdoor must reject a legacy brace block");
        assert_eq!(error.code(), DEFINITION_MIGRATION_DIAGNOSTIC, "{source}");
    }

    let source = local_source("    글값:글 := \"쓰임 { 드러냄 { 공개 {\".\n    값:수 := 1.");
    parse_v25(&source, "literal_and_comment.ddn")
        .expect("string text must not trigger legacy brace rejection");

    let comment_source = local_source(
        "    값:수 := 1. // 쓰임 { 드러냄 { 공개 { 는 주석 텍스트입니다\n    다음값:수 := 값.",
    );
    parse_v25(&comment_source, "comment_only_legacy_text.ddn")
        .expect("comment text must not trigger legacy brace rejection");
}

#[test]
fn v25_root_binding_rebinding_keeps_statement_category_and_canonical_bytes() {
    let source = "값 <- 0.\n값 <- 값 + 1.\n값 보여주기.\n";
    let program = parse_v25(source, "root_binding.ddn").expect("root bindings parse");

    assert!(program.items.is_empty());
    assert_eq!(program.root_stmts.len(), 3);
    assert!(matches!(program.root_stmts[0], Stmt::Mutate { .. }));
    assert!(matches!(program.root_stmts[1], Stmt::Mutate { .. }));
    assert!(matches!(program.root_stmts[2], Stmt::Show { .. }));

    let once = parse_v25_and_normalize(source, "root_binding.ddn", NormalizationLevel::N1)
        .expect("root binding canonicalization");
    let twice = parse_v25_and_normalize(&once, "root_binding.ddn", NormalizationLevel::N1)
        .expect("canonical root binding reparses");
    assert_eq!(once, twice);
    assert!(once.contains("값 <- 0."));
    assert!(!once.contains("값:수 :="));
}
