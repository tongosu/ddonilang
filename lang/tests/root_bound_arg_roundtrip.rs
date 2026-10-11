use ddonirang_lang::{canonicalize, normalize, parse_v25, NormalizationLevel};

fn assert_roundtrip(source: &str) {
    let mut program = parse_v25(source, "root_bound_arg.ddn").expect("parse original source");
    canonicalize(&mut program).expect("canonicalize original source");
    let first = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&first, "canonical.ddn")
        .unwrap_or_else(|error| panic!("canonical source must parse: {error:?}\n{first}"));
    canonicalize(&mut reparsed).expect("canonicalize canonical source");
    assert_eq!(first, normalize(&reparsed, NormalizationLevel::N1));
}

#[test]
fn root_and_body_short_foreach_keep_bound_values_unambiguous() {
    let source = "입력들:(수)차림 := 차림[1, 2, 3].\n입력들마다 { 입력들 보여주기. }.";
    assert_roundtrip(source);
    assert_roundtrip(&format!("검산:움직씨 := {{\n{source}\n}}"));
}

#[test]
fn root_and_body_selection_keep_bound_values_unambiguous() {
    let source = "값들 := 차림[1, 2].\n값들[5] 보여주기.";
    assert_roundtrip(source);
    assert_roundtrip(&format!("검산:움직씨 := {{\n{source}\n}}"));
}

#[test]
fn root_and_body_natural_calls_resolve_to_the_same_callable() {
    let declaration = "(사용량:수~을~를, 기본요금:수~에, 단가:수~로~으로) 요금계산:수:셈씨 := { 요금계산 <- 기본요금 + 사용량 * 단가. }.";
    let statements = "결과 := 120을 1000에 15로 요금계산하기. 결과 보여주기.";
    for source in [format!("{declaration}\n{statements}"), format!("{declaration}\n검산:움직씨 := {{ {statements} }}")] {
        assert_roundtrip(&source);
        let mut program = parse_v25(&source, "natural_call.ddn").unwrap();
        canonicalize(&mut program).unwrap();
        let text = normalize(&program, NormalizationLevel::N1);
        assert!(!text.contains("요금계산하기"), "call target must resolve: {text}");
    }
}

#[test]
fn root_and_body_implicit_current_item_preserve_typed_binding() {
    let declaration = "(값:수) 기록:움직씨 := { 값 보여주기. }.";
    let statements = "목록:(수)차림 := 차림[1, 2]. 목록마다 { () 기록하기. }.";
    assert_roundtrip(&format!("{declaration}\n{statements}"));
    assert_roundtrip(&format!("{declaration}\n검산:움직씨 := {{ {statements} }}"));
}
