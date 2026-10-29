use ddonirang_lang::{parse_v25_with_mode,ParseMode,PiecewiseCondition,NormalizationLevel};

const SOURCE:&str="요금 (사용량:나눔수~을 --> 나눔수) := 조건별 {\n정의역: 0 <= 사용량 <= 6.\n10 * 사용량 : 0 <= 사용량 <= 3.\n30 + 20 * (사용량 - 3) : 3 < 사용량 <= 6.\n}.\n";
fn parse(s:&str)->Result<ddonirang_lang::CanonProgram,ddonirang_lang::ParseError>{
    parse_v25_with_mode(s,"요금.ddn",ParseMode::Strict)
}
#[test]
fn approved_fee_source_keeps_domain_rows_and_chain_without_bool_lowering(){
    let p=parse(SOURCE).unwrap();let body=p.origin.piecewise_definitions.values().next().unwrap();
    assert_eq!(body.rows.len(),2);
    let PiecewiseCondition::Relations{terms,relations}=&body.domain else{panic!("domain")};
    assert_eq!(terms.len(),3);assert_eq!(relations.as_slice(),["<=","<="]);
    let canonical=ddonirang_lang::normalize(&p,NormalizationLevel::N1);
    let again=parse(&canonical).unwrap();assert_eq!(again.origin.piecewise_definitions.len(),1);
    assert_eq!(canonical,ddonirang_lang::normalize(&again,NormalizationLevel::N1));
    assert!(canonical.contains("정의역:"));assert!(canonical.contains("3 < 사용량 <= 6"));
}
#[test]
fn original_row_order_is_not_rewritten_as_priority(){
    let reversed=SOURCE.replace("10 * 사용량 : 0 <= 사용량 <= 3.\n30 + 20 * (사용량 - 3) : 3 < 사용량 <= 6.","30 + 20 * (사용량 - 3) : 3 < 사용량 <= 6.\n10 * 사용량 : 0 <= 사용량 <= 3.");
    let p=parse(&reversed).unwrap();assert_eq!(p.origin.piecewise_definitions.values().next().unwrap().rows.len(),2);
    assert!(ddonirang_lang::normalize(&p,NormalizationLevel::N1).contains("30 + 20 * (사용량 - 3) : 3 < 사용량 <= 6.\n10 * 사용량"));
}
#[test]
fn explicit_empty_is_preserved_and_not_synthesized_from_a_contradiction(){
    let empty="빈요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역: 비어있음. }.";
    let p=parse(empty).unwrap();assert!(matches!(p.origin.piecewise_definitions.values().next().unwrap().domain,PiecewiseCondition::ExplicitEmpty(_)));
    let ordinary="모순요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역: (0 <= 사용량 <= 1) 그리고 (2 <= 사용량 <= 3). }.";
    let p=parse(ordinary).unwrap();assert!(matches!(p.origin.piecewise_definitions.values().next().unwrap().domain,PiecewiseCondition::And(..)));
    // Syntax preservation is not admission of the contradictory domain.
}
#[test]
fn missing_duplicate_domain_and_empty_piece_are_rejected(){
    assert!(parse(&SOURCE.replace("정의역: 0 <= 사용량 <= 6.\n","")).is_err());
    assert!(parse(&SOURCE.replace("정의역: 0 <= 사용량 <= 6.","정의역: 0 <= 사용량 <= 6. 정의역: 0 <= 사용량 <= 6.")).is_err());
    assert!(parse(&SOURCE.replace("0 <= 사용량 <= 3.","비어있음.")).is_err());
    assert!(parse("빈요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역: 비어있음. 1 : 사용량 =:= 0. }.").is_err());
}
#[test]
fn mixed_chain_and_bare_bool_do_not_acquire_mathematical_condition_meaning(){
    assert!(parse(&SOURCE.replace("0 <= 사용량 <= 6","0 <= 사용량 > 6")).is_err());
    assert!(parse(&SOURCE.replace("0 <= 사용량 <= 6","참")).is_err());
}

#[test]
fn ordinary_identifier_named_piecewise_is_not_reserved_outside_the_body_surface(){
    let p=parse("그대로하다 (조건별:바른수 --> 바른수) := 조건별.").unwrap();
    assert!(p.origin.piecewise_definitions.is_empty());
}
