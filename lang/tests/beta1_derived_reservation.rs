//! Beta2 WO22 F6 supersedes the Beta1 reservation only for root one-way
//! declarations. Preserve the distinct AST and unsupported-scope rejection.
use ddonirang_lang::{parse_v25_with_mode,ParseMode,Stmt};

#[test]
fn beta2_activates_only_the_approved_root_one_way_reservation() {
    let source="처음 <- 1. 결과 <<- 처음 + 1.";
    let program=parse_v25_with_mode(source,"root-derived.ddn",ParseMode::Strict).unwrap();
    assert!(matches!(&program.root_stmts[1],Stmt::DerivedBinding{name,..} if name=="결과"));
    for (source,code) in [
        ("실행:움직씨 := { 처음 <- 1. 결과 <<- 처음 + 1. }.","E_DERIVED_SCOPE"),
        ("결과 <<->> 처음.","E_BIDIRECTIONAL_BINDING_UNSUPPORTED"),
    ] {
        let error=parse_v25_with_mode(source,"reserved-derived.ddn",ParseMode::Strict).unwrap_err();
        assert!(error.message.contains(code),"{error}");
        assert!(!source[error.span.start..error.span.end].is_empty());
    }
}

#[test]
fn beta1_reservation_does_not_reclassify_existing_assignment() {
    let source="실행:움직씨 := { 값 <- 1. 값 <- 값 + 1. 값 보여주기. }.";
    assert!(parse_v25_with_mode(source,"assignment.ddn",ParseMode::Strict).is_ok());
}
