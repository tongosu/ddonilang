use ddonirang_lang::{
    parse_v25 as parse, parse_v25_and_normalize as parse_and_normalize, NormalizationLevel, Stmt,
    TopLevelItem,
};

fn source(body: &str) -> String {
    format!("시험:움직씨 := {{\n{body}\n}}.")
}

#[test]
fn ordinary_and_hook_while_keep_distinct_ast_owners() {
    let program = parse(
        &source("{ 참 }인 동안 { 멈추기. }.\n(참)인 동안 { 1 보여주기. }."),
        "동안.ddn",
    )
    .unwrap();
    let TopLevelItem::SeedDef(def) = &program.items[0];
    let stmts = &def.body.as_ref().unwrap().stmts;
    assert!(matches!(stmts[0], Stmt::While { .. }));
    assert!(matches!(stmts[1], Stmt::HookWhile { .. }));
}

#[test]
fn legacy_and_colon_while_surfaces_are_rejected() {
    for body in [
        "{ 참 }인것 동안 { }.",
        "{ 참 }인것 동안: { }.",
        "{ 참 }인 동안: { }.",
    ] {
        let err = parse(&source(body), "옛반복.ddn").unwrap_err();
        assert!(err.message.contains("E_WHILE_SURFACE"), "{body}: {err:?}");
    }
}

#[test]
fn ordinary_while_canon_roundtrips_zero_multiple_and_nested_shapes() {
    for body in [
        "{ 거짓 }인 동안 { }.",
        "{ 참 }인 동안 { 멈추기. }.\n{ 거짓 }인 동안 { }.",
        "{ 참 }인 동안 { { 참 }인 동안 { 멈추기. }. 멈추기. }.",
    ] {
        let canonical =
            parse_and_normalize(&source(body), "정본.ddn", NormalizationLevel::N1).unwrap();
        let second = parse_and_normalize(&canonical, "정본.ddn", NormalizationLevel::N1).unwrap();
        assert_eq!(canonical, second, "{body}");
        assert!(canonical.contains("인 동안"));
        assert!(!canonical.contains("인것 동안"));
    }
}
