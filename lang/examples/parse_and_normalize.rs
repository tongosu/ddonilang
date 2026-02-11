// lang/examples/parse_and_normalize.rs
use ddonirang_lang::{lexer::Lexer, parser::Parser, normalizer::{Normalizer, NormalizationLevel}};

fn main() {
    let source = "(x:수) 증가:셈씨 = { x + 1 돌려줘. }";
    println!("--- 원본 --- \n{}", source);

    let tokens = Lexer::new(source).tokenize().expect("토큰화 실패");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program(source.to_string(), "example.ddoni".to_string()).expect("파싱 실패");

    let mut normalizer = Normalizer::new(NormalizationLevel::N1);
    let result = normalizer.normalize_program(&program);

    println!("\n--- 정본화 (N1) ---\n{}", result);
}
