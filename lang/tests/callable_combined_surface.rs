use ddonirang_lang::{
    canonicalize, parse_v25, parse_v25_and_normalize, NormalizationLevel, TopLevelItem,
};

const SUCCESS: &str = r#"
(값:수) 돕다{도우}:수:셈씨 := { 돕다 <- 값 + 1. }.
계산:수:셈씨 := {
    중간:수 := (3) 돕다.
    계산 <- (중간) 도우.
}.
매마디:움직씨 := { 계산기 보여주기. }.
"#;

#[test]
fn explicit_alternate_is_one_definition_identity_and_roundtrips() {
    let program = parse_v25(SUCCESS, "combined.ddn").expect("parse explicit alternate");
    let seed = program
        .items
        .iter()
        .find_map(|item| match item {
            TopLevelItem::SeedDef(seed) if seed.canonical_name == "돕다" => Some(seed),
            _ => None,
        })
        .expect("돕다 definition");
    assert_eq!(seed.alternate_forms, ["도우"]);
    assert_eq!(seed.callable_surface_names(), ["돕다", "도우"]);

    let canonical = parse_v25_and_normalize(SUCCESS, "combined.ddn", NormalizationLevel::N1)
        .expect("canonicalize");
    assert!(canonical.contains("돕다{도우}:셈수:셈씨"));
    let reparsed =
        parse_v25_and_normalize(&canonical, "combined-canonical.ddn", NormalizationLevel::N1)
            .expect("reparse canonical");
    assert_eq!(canonical, reparsed);
}

#[test]
fn explicit_set_disables_hidden_suffix_but_legacy_owner_keeps_it() {
    let error = parse_v25(
        "먹{히히}:셈씨 := { 먹 <- 1. }. 계산:셈씨 := { 계산 <- () 먹기. }.",
        "explicit-hidden.ddn",
    )
    .expect_err("explicit set owner must not gain an unregistered suffix");
    assert!(
        error.message.contains("E_REGISTERED_FORM_REQUIRED"),
        "{}",
        error.message
    );

    parse_v25(
        "먹:셈씨 := { 먹 <- 1. }. 계산:셈씨 := { 계산 <- () 먹기. }.",
        "legacy-hidden.ddn",
    )
    .expect("legacy owner without an explicit set keeps the predecessor suffix");
}

#[test]
fn duplicate_explicit_surface_fails_closed() {
    let error = parse_v25(
        "(값:수) 돕다{도우~도우}:수:셈씨 := { 돕다 <- 값. }.",
        "duplicate.ddn",
    )
    .expect_err("duplicate surface must fail");
    assert!(
        error.message.contains("E_CALLABLE_SURFACE_DUPLICATE"),
        "{}",
        error.message
    );
}

#[test]
fn same_typed_surface_across_definitions_fails_closed() {
    let error = parse_v25(
        r#"
        (값:수) 돕다{도우}:수:셈씨 := { 돕다 <- 값. }.
        (값:수) 보태다{도우}:수:셈씨 := { 보태다 <- 값. }.
        "#,
        "ambiguous.ddn",
    )
    .expect_err("ambiguous exact surface must fail");
    assert!(
        error.message.contains("E_CALLABLE_SURFACE_AMBIGUOUS"),
        "{}",
        error.message
    );
}

#[test]
fn preferred_keeps_full_overload_family_and_alternate_keeps_exact_subset_provenance() {
    let definitions = r#"
        (값:셈수) 분류{숫자형}:글:셈씨 := { 분류 <- "숫자". }.
        (값:글) 분류:글:셈씨 := { 분류 <- "문자". }.
    "#;
    for (call, candidate_count) in [
        ("검산:움직씨 := { (값=7) 분류 보여주기. }.", 2usize),
        ("검산:움직씨 := { (값=7) 숫자형 보여주기. }.", 1usize),
    ] {
        let mut program = parse_v25(&format!("{definitions}{call}"), "overload.ddn")
            .expect("registered overload call");
        assert_eq!(
            program
                .origin
                .registered_call_candidates
                .values()
                .next()
                .expect("authored registered surface provenance")
                .len(),
            candidate_count
        );
        let before = program.origin.registered_call_candidates.clone();
        canonicalize(&mut program).expect("canonicalize registered call");
        assert_eq!(program.origin.registered_call_candidates, before);
    }
}
