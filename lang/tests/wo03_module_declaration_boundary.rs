use ddonirang_lang::{parse_v25, Lexer, ParseMode, Parser};

#[test]
fn statement_dot_before_eof_or_closing_body_is_not_field_access() {
    for source in ["원본 := 1. 값 := 원본.", "실행:움직씨 := {원본 := 1. 값 := 원본.}"] {
        parse_v25(source, "boundary.ddn").expect("statement terminator");
    }
}

#[test]
fn unlinked_declaration_scan_does_not_admit_an_executable_unknown_call() {
    let source = "실행:움직씨 := { () 외부.계산하기. }.";
    let tokens = Lexer::new(source).tokenize().unwrap();
    Parser::new_v25_with_mode(tokens, ParseMode::Strict)
        .parse_unlinked_module(source.into(), "module.ddn".into())
        .expect("declaration scan defers imported calls");
    let error = parse_v25(source, "module.ddn").unwrap_err();
    assert!(error.message.contains("E_CALL_TAIL_NO_SEED"));
}
