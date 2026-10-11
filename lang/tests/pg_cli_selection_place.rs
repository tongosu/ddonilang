use ddonirang_lang::{canonicalize, normalize, parse_v25, ExprKind, NormalizationLevel, Stmt, TopLevelItem};

const SOURCE: &str = r#"검산:움직씨 := {
    목록:(수)차림 := 차림[1, 2, 3].
    목록[1] <- 9.
    목록 보여주기.
}
"#;

#[test]
fn pg_cli_selection_place_is_not_a_value_selection_or_definition() {
    let program = parse_v25(SOURCE, "selection_place.ddn").expect("current source parses");
    let TopLevelItem::SeedDef(seed) = &program.items[0];
    let body = seed.body.as_ref().expect("callable body");
    let Stmt::Mutate { target, value, .. } = &body.stmts[1] else {
        panic!("second statement must be a mutation");
    };
    let ExprKind::SelectionPlace { target: root, index } = &target.kind else {
        panic!("indexed <- target must retain SelectionPlace");
    };
    assert!(matches!(root.kind, ExprKind::Var(ref name) if name == "목록"));
    assert!(matches!(index.kind, ExprKind::Literal(_)));
    assert!(matches!(value.kind, ExprKind::Literal(_)));
}

#[test]
fn pg_cli_selection_place_canon_is_stable_and_preserves_source_order() {
    let mut program = parse_v25(SOURCE, "selection_place.ddn").expect("parse");
    canonicalize(&mut program).expect("canonicalize");
    let once = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&once, "selection_place.ddn").expect("reparse");
    canonicalize(&mut reparsed).expect("recanonicalize");
    let twice = normalize(&reparsed, NormalizationLevel::N1);

    assert_eq!(once, twice);
    assert!(once.contains("목록[1] <- 9."));
}

#[test]
fn pg_cli_selection_place_rejects_temporary_targets() {
    let error = parse_v25(
        "검산:움직씨 := { 차림[1, 2][0] <- 9. }\n",
        "selection_place_temporary.ddn",
    )
    .expect_err("temporary selections are not writable roots");
    assert!(error.code().starts_with("E_") || error.message.starts_with("E_"));
}
