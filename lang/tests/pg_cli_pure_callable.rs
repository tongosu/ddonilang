use ddonirang_lang::{
    canonicalize, normalize, parse_v25, NormalizationLevel, SeedKind, Stmt,
    TopLevelItem, TypeRef,
};

const SOURCE: &str = include_str!("../../tool/tests/fixtures/pg_cli_pure_callable.ddn");

#[test]
fn pg_cli_rd406_single_expression_callable_parses() {
    let program = parse_v25(SOURCE, "pg_cli_pure_callable.ddn")
        .expect("RD-406 single-expression callable must parse through the shared frontdoor");
    let TopLevelItem::SeedDef(seed) = &program.items[0];
    assert_eq!(seed.canonical_name, "자리번호");
    assert_eq!(seed.seed_kind, SeedKind::ValueFunc);
    assert!(matches!(seed.result_type, Some(TypeRef::Named(ref name)) if name == "셈수"));
    let body = seed.body.as_ref().expect("single expression body");
    assert!(matches!(body.stmts.as_slice(), [Stmt::Return { .. }]));
}

#[test]
fn pg_cli_rd406_canon_is_stable_and_preserves_the_named_callable_surface() {
    let mut program = parse_v25(SOURCE, "pg_cli_pure_callable.ddn").expect("parse");
    canonicalize(&mut program).expect("canonicalize");
    let once = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&once, "pg_cli_pure_callable.ddn").expect("reparse");
    canonicalize(&mut reparsed).expect("recanonicalize");
    let twice = normalize(&reparsed, NormalizationLevel::N1);

    assert_eq!(once, twice);
    assert!(once.contains("자리번호 (가로:셈수, 세로:셈수 --> 셈수) := 세로 * 10 + 가로."));
}

#[test]
fn pg_cli_rd406_rejects_a_block_body_instead_of_lowering_it_to_an_ordinary_seed() {
    let source = "자리번호 (가로:셈수 --> 셈수) := { 가로. }\n";
    let error = parse_v25(source, "pg_cli_pure_callable_block.ddn")
        .expect_err("RD-406 requires exactly one expression body");
    assert!(error.message.contains("E_RD406_SINGLE_EXPRESSION_BODY_REQUIRED"));
}
