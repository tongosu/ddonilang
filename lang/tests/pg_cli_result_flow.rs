use ddonirang_lang::{canonicalize, normalize, parse_v25, ExprKind, NormalizationLevel, Stmt, TopLevelItem};

const SOURCE: &str = r#"검산:움직씨 := {
    판:(수)차림 := 차림[0, 0].
    1 -> 판[0].
    판 보여주기.
}
"#;

#[test]
fn result_flow_keeps_producer_and_selection_place_distinct_from_mutation() {
    let program = parse_v25(SOURCE, "result_flow.ddn").expect("current source parses");
    let TopLevelItem::SeedDef(seed) = &program.items[0];
    let body = seed.body.as_ref().expect("callable body");
    let Stmt::ResultFlow {
        producer,
        destination,
        ..
    } = &body.stmts[1]
    else {
        panic!("arrow statement must retain ResultFlow, not Mutate");
    };
    assert!(matches!(producer.kind, ExprKind::Literal(_)));
    let ExprKind::SelectionPlace { target, index } = &destination.kind else {
        panic!("result-flow destination must retain SelectionPlace");
    };
    assert!(matches!(target.kind, ExprKind::Var(ref name) if name == "판"));
    assert!(matches!(index.kind, ExprKind::Literal(_)));
}

#[test]
fn result_flow_canon_is_stable_and_does_not_rewrite_to_left_arrow() {
    let mut program = parse_v25(SOURCE, "result_flow.ddn").expect("parse");
    canonicalize(&mut program).expect("canonicalize");
    let once = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&once, "result_flow.ddn").expect("reparse");
    canonicalize(&mut reparsed).expect("recanonicalize");
    let twice = normalize(&reparsed, NormalizationLevel::N1);

    assert_eq!(once, twice);
    assert!(once.contains("1 -> 판[0]."));
    assert!(!once.contains("판[0] <- 1."));
    assert!(!once.contains('\r'));
}

#[test]
fn result_flow_rejects_non_selection_destinations() {
    let error = parse_v25(
        "검산:움직씨 := { 1 -> 판. }\n",
        "result_flow_not_place.ddn",
    )
    .expect_err("result-flow destination must be a writable selection place");
    assert!(error.message.contains("E_RESULT_FLOW_DESTINATION_PLACE_REQUIRED"));
}
