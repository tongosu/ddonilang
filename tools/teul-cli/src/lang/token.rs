use crate::lang::span::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    String(String),
    Template(String),
    Number(i64),
    Atom(String),
    True,
    False,
    None,
    Salim,
    Boyeojugi,
    Chaewugi,
    Susic,
    Start,
    EveryMadi,
    Halttae,
    Mada,
    Ilttae,
    Aniramyeon,
    Anigo,
    Majeumyeon,
    Jeonjehae,
    Bojanghago,
    Goreugi,
    Repeat,
    During,
    Daehae,
    Break,
    Ident(String),
    Plus,
    PlusEqual,
    PlusArrow,
    Minus,
    MinusEqual,
    MinusArrow,
    Star,
    Slash,
    At,
    And,
    Or,
    EqEq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    Equal,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Caret,
    Dot,
    DotDot,
    DotDotEq,
    Colon,
    Arrow,
    Pipe,
    Newline,
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
