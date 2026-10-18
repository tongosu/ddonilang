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

#[test]
fn typed_postfix_selection_preserves_brackets_and_is_not_a_list_provider_call() {
    let source = "검산:움직씨 := { 자료 <- (\"칸\", 2) 짝맞춤. 값 <- 자료[\"칸\"]. 값 보여주기. }.";
    let mut program = parse_v25(source, "typed_selection.ddn").expect("parse");
    let TopLevelItem::SeedDef(seed) = &program.items[0];
    let Stmt::Mutate {value, ..} = &seed.body.as_ref().unwrap().stmts[1] else {panic!("value binding")};
    assert!(matches!(value.kind, ExprKind::MultiAxisSelection(_)));
    canonicalize(&mut program).expect("canonicalize");
    let once = normalize(&program, NormalizationLevel::N1);
    assert!(once.contains("자료[\"칸\"]"));
    assert!(!once.contains("차림.값"));
    let mut reparsed = parse_v25(&once, "typed_selection.ddn").expect("reparse");
    canonicalize(&mut reparsed).expect("recanonicalize");
    assert_eq!(once, normalize(&reparsed, NormalizationLevel::N1));
}

#[test]
fn rd352_runtime_list_range_keeps_authored_bounds_in_canonical_roundtrip() {
    let source="자료 <- 차림[10,20,30]. 처음 <- 1. 끝 <- 3. 조각 <- 자료[처음..끝]. 자료[처음..끝] <- 차림[7,8].";
    let mut ast=parse_v25(source,"runtime-range.ddn").expect("runtime list range parse");
    canonicalize(&mut ast).unwrap();
    let once=normalize(&ast,NormalizationLevel::N1);
    let mut again=parse_v25(&once,"runtime-range.ddn").expect("runtime range reparse");
    canonicalize(&mut again).unwrap();
    assert_eq!(once,normalize(&again,NormalizationLevel::N1));
    assert!(once.contains("처음") && once.contains("끝"));
}
