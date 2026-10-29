use ddonirang_lang::{proof_authoring::prepare_source, ParseError};
use ddonirang_proof::algebra::{check_algebra_definition,CheckedAlgebraDefinition};
const LINEAR:&str="(미지수:나눔수~을, 근거:((미지수 + 3) =:= 7)보임~로)\n일차방정식보이다{일차방정식보이기}:(미지수 =:= 4)보임:셈씨 := {\n빼기 := (근거=근거, 항=3) 양변빼기.\n일차방정식보이다 <- (근거=빼기, 왼=미지수, 오른=4) 등식정리.\n}.";
fn check(source:&str)->Result<Vec<CheckedAlgebraDefinition>,ParseError> {
    prepare_source(source,"대수.ddn")?.resolve_algebra()?.iter().map(check_algebra_definition).collect()
}
#[test]
fn original_linear_and_pythagorean_conditional_sources() {
    for source in [LINEAR,include_str!("fixtures/pythagorean_algebra_r1.ddn")] {
        let checked=check(source).unwrap();assert_eq!(checked.len(),1);
        assert_eq!(checked[0].resolved().assumptions().len(),1);
        checked[0].validate_source(source,"대수.ddn").unwrap();
        assert!(checked[0].validate_source(&format!("{source}\n"),"대수.ddn").is_err());
        assert!(checked[0].validate_source(source,"다른.ddn").is_err());
        assert!(!checked[0].resolved().same_owner(check(source).unwrap()[0].resolved()));
    }
}
#[test]
fn exact_fraction_and_subtraction_of_an_absent_term_are_valid() {
    for source in [
        include_str!("../../work/astra-education/algebra-source-candidates/일차방정식_대수_R1_정상_정확분수.ddn"),
        include_str!("../../work/astra-education/algebra-source-candidates/일차방정식_대수_R1_정상_겉에없는항빼기.ddn"),
    ] {assert_eq!(check(source).unwrap().len(),1);}
}
#[test]
fn rewrite_errors_and_final_goal_errors_are_distinct() {
    let wrong=LINEAR.replace("오른=4","오른=5");
    let error=check(&wrong).unwrap_err();assert!(error.message.starts_with("E_PROOF_ALGEBRA_REWRITE:"));
    assert_eq!(&wrong[error.span.start..error.span.end],"5");
    let wrong=LINEAR.replace("(미지수 =:= 4)보임","(미지수 =:= 5)보임");
    let error=check(&wrong).unwrap_err();assert!(error.message.starts_with("E_PROOF_ALGEBRA_RESULT:"));
    assert_eq!(&wrong[error.span.start..error.span.end],"5");
}
#[test]
fn invalid_unused_steps_cannot_be_hidden_by_a_valid_result() {
    let source=LINEAR.replace("빼기 :=","잘못 := (근거=근거, 왼=미지수 + 3, 오른=8) 등식정리.\n빼기 :=");
    let error=check(&source).unwrap_err();assert!(error.message.starts_with("E_PROOF_ALGEBRA_REWRITE:"));
    assert_eq!(&source[error.span.start..error.span.end],"8");
}
#[test]
fn missing_assumption_and_success_flags_are_not_proofs() {
    let missing=LINEAR.replace("근거=근거","근거=없는근거");
    assert!(check(&missing).unwrap_err().message.starts_with("E_PROOF_ASSUMPTION:"));
    let forged=LINEAR.replace("(근거=빼기, 왼=미지수, 오른=4) 등식정리","참");
    let error=check(&forged).unwrap_err();
    assert!(error.message.starts_with("E_PROOF_ASSUMPTION:"),"{error:?}");
    assert_eq!(&forged[error.span.start..error.span.end],"참");
}
#[test]
fn equal_residuals_do_not_allow_rewriting_both_sides() {
    let source="(값:나눔수~을, 근거:(값 =:= 값)보임~로)\n잘못보이다{잘못보이기}:((값+1) =:= (값+1))보임:셈씨 := { 잘못보이다 <- (근거=근거, 왼=값+1, 오른=값+1) 등식정리. }.";
    assert!(check(source).unwrap_err().message.starts_with("E_PROOF_ALGEBRA_REWRITE:"));
}
#[test]
fn static_operations_are_not_global_builtin_fallbacks() {
    let outside="(근거=참, 항=3) 양변빼기.";
    assert!(prepare_source(outside,"범위밖.ddn").is_err());
    let shadow=LINEAR.replace("빼기 :=","양변빼기 := 근거.\n빼기 :=");
    assert!(check(&shadow).unwrap_err().message.starts_with("E_PROOF_CALL_SHADOWED:"));
    let missing=LINEAR.replace("근거=근거, 항=3","근거=근거");
    assert!(check(&missing).unwrap_err().message.starts_with("E_PROOF_ALGEBRA_PIN:"));
}

#[test]
fn pythagorean_left_right_and_final_goal_fail_at_distinct_steps() {
    let source=include_str!("fixtures/pythagorean_algebra_r1.ddn");
    for (wrong,code,needle) in [
        (source.replace("왼=밑변 * 밑변 + 2 * 밑변 * 높이 + 높이 * 높이","왼=밑변 * 밑변 + 높이 * 높이"),"E_PROOF_ALGEBRA_REWRITE:","밑변 * 밑변 + 높이 * 높이"),
        (source.replace("오른=2 * 밑변 * 높이 + 빗변 * 빗변","오른=밑변 * 높이 + 빗변 * 빗변"),"E_PROOF_ALGEBRA_REWRITE:","밑변 * 높이 + 빗변 * 빗변"),
        (source.replace("항=2 * 밑변 * 높이","항=밑변 * 높이"),"E_PROOF_ALGEBRA_RESULT:","밑변 * 밑변 + 높이 * 높이"),
    ] {
        let error=check(&wrong).unwrap_err();assert!(error.message.starts_with(code),"{error:?}");
        assert!(wrong[error.span.start..error.span.end].contains(needle));
    }
}

#[test]
fn korean_emoji_and_crlf_preserve_bad_step_span() {
    let source=format!("// 한글 🍎\r\n{}",LINEAR.replace('\n',"\r\n").replace("오른=4","오른=5"));
    let error=check(&source).unwrap_err();
    assert!(error.message.starts_with("E_PROOF_ALGEBRA_REWRITE:"));
    assert_eq!(&source[error.span.start..error.span.end],"5");
}

const V4: &str = include_str!("../../tool/tests/fixtures/v4_linear_steps/정상.ddn");
#[test]
fn divide_closed_nonzero_coefficient_replays_and_preserves_owner() {
    for source in [V4.to_owned(), V4.replace("계수=3 / 2", "계수=(1 + 2) / 2"),
        V4.replace("(3 / 2)", "(0 - 3 / 2)").replace("계수=3 / 2", "계수=0 - 3 / 2").replace("미지수 =:= 4", "미지수 =:= (0 - 4)").replace("오른=4", "오른=0 - 4")] {
        let checked = check(&source).unwrap();
        checked[0].validate_source(&source, "대수.ddn").unwrap();
        assert!(checked[0].validate_source(&source, "다른.ddn").is_err());
        assert!(checked[0].validate_source(&(source.clone()+"\n"), "대수.ddn").is_err());
    }
}
#[test]
fn divide_zero_variable_and_wrong_intermediate_are_rejected() {
    for (coefficient,code) in [("2 - 2","E_PROOF_ALGEBRA_DIVIDE_ZERO"),
        ("미지수 - 미지수 + 1","E_PROOF_ALGEBRA_COEFFICIENT"),
        ("0 * 미지수 + 1","E_PROOF_ALGEBRA_COEFFICIENT"),
        ("1 / 0","E_PROOF_ALGEBRA_TERM")] {
        let source=V4.replace("계수=3 / 2", &format!("계수={coefficient}"));
        let error=check(&source).unwrap_err();
        assert!(error.message.starts_with(code),"{coefficient}: {error:?}");
    }
    let bad=V4.replace("오른=6", "오른=5");
    assert!(check(&bad).unwrap_err().message.starts_with("E_PROOF_ALGEBRA_REWRITE"));
    let wrong_divisor=V4.replace("계수=3 / 2", "계수=3");
    assert!(check(&wrong_divisor).unwrap_err().message.starts_with("E_PROOF_ALGEBRA_REWRITE"));
}
