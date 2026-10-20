use ddonirang_lang::{parse_v25,canonicalize,Stmt};
use ddonirang_lang::normalizer::{Normalizer,NormalizationLevel};

#[test]
fn derived_surface_preserves_distinct_ast_and_canonical_roundtrip() {
    let source="처음 <- 1. 결과 <<- 처음 + 1.";
    let mut program=parse_v25(source,"derived.ddn").unwrap();
    assert!(matches!(&program.root_stmts[0],Stmt::Mutate{..}));
    assert!(matches!(&program.root_stmts[1],Stmt::DerivedBinding{name,..} if name=="결과"));
    canonicalize(&mut program).unwrap();
    let canon=Normalizer::new(NormalizationLevel::N1).normalize_program(&program);
    assert!(canon.contains("결과 <<-"));
    let again=parse_v25(&canon,"derived-canon.ddn").unwrap();
    assert!(matches!(&again.root_stmts[1],Stmt::DerivedBinding{name,..} if name=="결과"));
}

#[test]
fn derived_surface_rejects_callable_local_and_selected_target() {
    for source in [
        "실행:움직씨 := { 처음 <- 1. 결과 <<- 처음 + 1. }.",
        "값:셈씨 := { 결과 <<- 2. }.",
        "대상.칸 <<- 1.",
        "값[0] <<- 1.",
        "(매마디)마다 { 결과 <<- 1.0. }.",
        "{ 참 }이면 { 결과 <<- 1.0. }.",
    ] {
        let error=parse_v25(source,"invalid-derived.ddn").unwrap_err();
        assert!(error.message.contains("E_DERIVED_SCOPE")||error.message.contains("E_DERIVED_TARGET"),"{source}: {error}");
    }
}
