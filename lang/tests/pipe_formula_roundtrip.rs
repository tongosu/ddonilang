use ddonirang_lang::{canonicalize, normalize, parse_v25, NormalizationLevel};

#[test]
fn formula_comparison_preserves_both_pipe_operands_after_canon() {
    let source = "결과 := (수식{x+3} 해서 (x=4) 풀기) == (수식{7} 해서 () 풀기). 결과 보여주기.";
    let mut program = parse_v25(source, "before.ddn").unwrap();
    canonicalize(&mut program).unwrap();
    let canonical = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&canonical, "after.ddn")
        .unwrap_or_else(|error| panic!("{error:?}\n{canonical}"));
    canonicalize(&mut reparsed).unwrap();
    assert_eq!(canonical, normalize(&reparsed, NormalizationLevel::N1));
}

#[test]
fn named_formula_pipe_reparses_without_an_authored_flow_identifier() {
    let source = "식 := 수식{x+3}. 결과 := 식 해서 (x=4) 풀기. 결과 보여주기.";
    let mut program = parse_v25(source, "before.ddn").unwrap();
    canonicalize(&mut program).unwrap();
    let canonical = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&canonical, "after.ddn")
        .unwrap_or_else(|error| panic!("{error:?}\n{canonical}"));
    canonicalize(&mut reparsed).unwrap();
    assert_eq!(canonical, normalize(&reparsed, NormalizationLevel::N1));
}

#[test]
fn supplied_input_pack_helper_roundtrips() {
    let source = include_str!("../../tool/tests/fixtures/student_equation_helper.ddn");
    let mut program = parse_v25(source, "helper.ddn").unwrap();
    canonicalize(&mut program).unwrap();
    let canonical = normalize(&program, NormalizationLevel::N1);
    assert!(!canonical.contains("흐름값"));
    let mut reparsed = parse_v25(&canonical, "after.ddn")
        .unwrap_or_else(|error| panic!("{error:?}\n{canonical}"));
    canonicalize(&mut reparsed).unwrap();
    assert_eq!(canonical, normalize(&reparsed, NormalizationLevel::N1));
}
