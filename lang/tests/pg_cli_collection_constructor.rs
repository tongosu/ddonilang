use ddonirang_lang::{
    canonicalize, normalize, parse_v25, ExprKind, NormalizationLevel, Stmt, TopLevelItem, TypeRef,
};

const SOURCE: &str = r#"검산:움직씨 := {
    목록:(수)차림 := 차림[1, 2, 3].
    목록 보여주기.
}
"#;

#[test]
fn pg_cli_collection_constructor_is_not_a_postfix_selection_call() {
    let program = parse_v25(SOURCE, "collection_constructor.ddn").expect("current source parses");
    let TopLevelItem::SeedDef(seed) = &program.items[0];
    let body = seed.body.as_ref().expect("callable body");
    let Stmt::Definition { value, .. } = &body.stmts[0] else {
        panic!("first statement must be the typed definition");
    };
    let ExprKind::CollectionConstructor { elements } = &value.kind else {
        panic!("차림[...] must have its own AST category");
    };
    assert_eq!(elements.len(), 3);
    assert!(matches!(elements[0].kind, ExprKind::Literal(_)));
    assert!(!matches!(value.kind, ExprKind::Call { .. }));
    let Stmt::Definition { type_ref, .. } = &body.stmts[0] else {
        unreachable!();
    };
    assert!(matches!(type_ref, TypeRef::Applied { name, args } if name == "차림" && args.len() == 1));
}

#[test]
fn pg_cli_collection_constructor_canon_is_stable_and_preserves_order() {
    let mut program = parse_v25(SOURCE, "collection_constructor.ddn").expect("parse");
    canonicalize(&mut program).expect("canonicalize");
    let once = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&once, "collection_constructor.ddn").expect("reparse");
    canonicalize(&mut reparsed).expect("recanonicalize");
    let twice = normalize(&reparsed, NormalizationLevel::N1);

    assert_eq!(once, twice);
    assert!(once.contains("차림[1, 2, 3]"));
}

#[test]
fn pg_cli_collection_constructor_rejects_empty_and_trailing_forms() {
    for source in [
        "검산:움직씨 := { 목록 <- 차림[]. }\n",
        "검산:움직씨 := { 목록 <- 차림[1,]. }\n",
    ] {
        let error = parse_v25(source, "collection_constructor_negative.ddn")
            .expect_err("type-undetermined or malformed constructor must fail closed");
        assert!(error.code().starts_with("E_") || error.message.starts_with("E_"));
    }
}
