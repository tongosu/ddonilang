use ddonirang_lang::{canonicalize, normalize, parse_v25, NormalizationLevel};

// A3-P3 provisional lifecycle: formatting must not move world setup into a
// callable's local scope or manufacture a user-authored tick definition.
#[test]
fn top_level_setup_remains_top_level_after_canon() {
    for source in [
        "채비 { r:나눔수 := (6,9) 나눔수. }. r 보여주기.",
        "채비 { 값:수 := 2. }. 매마디:움직씨 := { 값 <- 값 + 1. }. 값 보여주기.",
    ] {
        let mut program = parse_v25(source, "채비.ddn").unwrap();
        canonicalize(&mut program).unwrap();
        let text = normalize(&program, NormalizationLevel::N1);
        assert!(text.starts_with("채비 {"), "{text}");
        let mut roundtrip = parse_v25(&text, "채비.ddn").unwrap();
        canonicalize(&mut roundtrip).unwrap();
        assert_eq!(text, normalize(&roundtrip, NormalizationLevel::N1));
    }
}

#[test]
fn callable_local_setup_is_not_lifted() {
    let mut program = parse_v25("실행:움직씨 := { 채비 { 숨김:수 := 2. }. 숨김 보여주기. }.", "지역.ddn").unwrap();
    canonicalize(&mut program).unwrap();
    let text = normalize(&program, NormalizationLevel::N1);
    assert!(text.starts_with("실행:움직씨"), "{text}");
    assert!(text.contains("    채비 {"));
}
