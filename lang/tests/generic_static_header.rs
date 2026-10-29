use ddonirang_lang::{parse_v25, TopLevelItem};

const RAW: &str = "(값씨:씨자리) 두고\n(값:값씨~을)\n그대로:값씨:셈씨 := { 그대로 <- 값. }.";

#[test]
fn static_type_binder_is_separate_from_runtime_pin() {
    let program = parse_v25(RAW, "제네릭.ddn").unwrap();
    let TopLevelItem::SeedDef(seed) = &program.items[0];
    assert_eq!(seed.type_params.len(), 1);
    assert_eq!(seed.type_params[0].name, "값씨");
    assert_eq!(seed.params.len(), 1);
    assert_ne!(seed.type_params[0].id, seed.params[0].id);
    assert!(seed.proof_header.is_none());
    let span = seed.type_params[0].span;
    assert_eq!(&RAW[span.start..span.end], "값씨:씨자리");
    let canonical = ddonirang_lang::normalizer::normalize(
        &program, ddonirang_lang::normalizer::NormalizationLevel::N1);
    let reparsed = parse_v25(&canonical, "정본.ddn").unwrap();
    let TopLevelItem::SeedDef(roundtrip) = &reparsed.items[0];
    assert_eq!(roundtrip.type_params[0].name, "값씨");
    assert_eq!(roundtrip.params.len(), 1);
    assert!(roundtrip.proof_header.is_none());
}

#[test]
fn same_spelling_in_distinct_declarations_has_distinct_binders() {
    let raw = format!("{RAW}\n{}", RAW.replace("그대로", "다시"));
    let program = parse_v25(&raw, "두정의.ddn").unwrap();
    let TopLevelItem::SeedDef(first) = &program.items[0];
    let TopLevelItem::SeedDef(second) = &program.items[1];
    assert_eq!(first.type_params[0].name, second.type_params[0].name);
    assert_ne!(first.type_params[0].id, second.type_params[0].id);
}

#[test]
fn duplicate_or_unimplemented_mixed_binders_are_rejected() {
    for header in ["값씨:씨자리, 값씨:씨자리", "값씨:씨자리, P:세움"] {
        assert!(parse_v25(&RAW.replace("값씨:씨자리", header), "거부.ddn").is_err());
    }
}
