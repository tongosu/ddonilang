//! Owner scope cut: preserve the approved operator, never silently lower it
//! to assignment or claim that a persistent relation has been installed.
use ddonirang_lang::{parse_v25_with_mode,ParseMode};

#[test]
fn beta1_reserved_derived_operator_is_explicitly_rejected_before_a_program_is_returned() {
    for source in [
        "처음 <- 1. 결과 <<- 처음 + 1.",
        "실행:움직씨 := { 처음 <- 1. 결과 <<- 처음 + 1. }.",
    ] {
        let error=parse_v25_with_mode(source,"reserved-derived.ddn",ParseMode::Strict).unwrap_err();
        assert!(error.message.contains("E_PERSISTENT_DEPENDENCY_PRODUCT_OPEN"),"{error}");
        assert!(error.message.contains("아직 지원하지 않습니다"),"{error}");
        assert!(error.message.contains("베타2"),"{error}");
        assert_eq!(&source[error.span.start..error.span.end],"<<-");
    }
}

#[test]
fn beta1_reservation_does_not_reclassify_existing_assignment() {
    let source="실행:움직씨 := { 값 <- 1. 값 <- 값 + 1. 값 보여주기. }.";
    assert!(parse_v25_with_mode(source,"assignment.ddn",ParseMode::Strict).is_ok());
}
