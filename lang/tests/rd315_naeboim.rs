use ddonirang_lang::{parse_v25, parse_v25_and_normalize, NormalizationLevel, Stmt};

#[test]
fn rd315_observation_uses_successor_surface_and_roundtrips() {
    let source = "t := 0. 값 := 1. (x축: t, 값: 값) 내보임.";
    let canon = parse_v25_and_normalize(source, "rd315.ddn", NormalizationLevel::N1)
        .expect("승인된 관찰 선언");
    assert!(canon.contains(") 내보임"), "{canon}");
    let again = parse_v25_and_normalize(&canon, "rd315.ddn", NormalizationLevel::N1)
        .expect("정본 왕복");
    assert_eq!(canon, again);
}

#[test]
fn rd315_predecessor_is_not_an_observation_alias() {
    let source = "(값: 1) 보임.";
    if let Ok(program) = parse_v25(source, "old.ddn") {
        assert!(!program.root_stmts.iter().any(|item| matches!(item, Stmt::PublicObservation { .. })));
    }
}

#[test]
fn rd315_observation_diagnostics_are_preserved() {
    for (source, code) in [
        ("() 내보임.", "E_BOIM_EMPTY_PACK"),
        ("(값: 1, 값: 2) 내보임.", "E_BOIM_DUPLICATE_FIELD"),
        ("(값=1) 내보임.", "E_BOIM_FIXED_BINDING_FORBIDDEN"),
    ] {
        let error = parse_v25(source, "invalid.ddn").expect_err("잘못된 관찰 선언");
        assert!(error.message.contains(code), "{error:?}");
        assert!(error.span.end > error.span.start);
    }
}
