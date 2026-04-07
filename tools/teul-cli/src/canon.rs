use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use crate::file_meta::{format_file_meta, split_file_meta, FileMeta};
use serde::Serialize;

#[derive(Debug)]
pub struct CanonError {
    code: &'static str,
    message: String,
}

impl CanonError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl std::fmt::Display for CanonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code, self.message)
    }
}

impl std::error::Error for CanonError {}

fn has_root_hide_directive(input: &str) -> bool {
    for line in input.lines() {
        let trimmed = line.trim_start_matches(|ch| matches!(ch, ' ' | '\t' | '\r' | '\u{feff}'));
        if !trimmed.starts_with('#') {
            continue;
        }
        let rest = trimmed[1..].trim_start_matches(|ch| matches!(ch, ' ' | '\t'));
        if rest.starts_with("바탕숨김") || rest.starts_with("암묵살림") {
            return true;
        }
    }
    false
}

fn is_bridge_alias_type_name(type_name: &str) -> bool {
    matches!(
        type_name,
        "글"
            | "수"
            | "셈수"
            | "fixed64"
            | "sim_num"
            | "참거짓"
            | "논"
            | "bool"
            | "boolean"
    )
}

fn bridge_alias_matches_literal(type_name: &str, literal: &Literal) -> bool {
    match type_name {
        "글" => matches!(literal, Literal::Str(_)),
        "수" | "셈수" | "fixed64" | "sim_num" => matches!(literal, Literal::Num(_)),
        "참거짓" | "논" | "bool" | "boolean" => matches!(literal, Literal::Bool(_)),
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Ident(String),
    Atom(String),
    Template(String),
    Formula(String),
    BogaeMadangBlock(String),
    BogaeJangmyeonBlock(String),
    ExecPolicyBlock(String),
    JjaimBlock(String),
    GuseongBlock(String),
    String(String),
    Number(String),
    True,
    False,
    None,
    Show,
    Inspect,
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
    Plus,
    PlusEqual,
    PlusArrow,
    Minus,
    MinusEqual,
    MinusArrow,
    Star,
    Slash,
    Percent,
    And,
    Or,
    Comma,
    EqEq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    DotDot,
    DotDotEq,
    Dot,
    Arrow,
    SignalArrow,
    Colon,
    Equals,
    Pipe,
    At,
    Question,
    Prompt,
    PromptBlock(String),
    Bang,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
}

struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    _input: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            _input: input,
        }
    }

    fn tokenize(input: &'a str) -> Result<Vec<Token>, CanonError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        while !lexer.is_eof() {
            lexer.skip_whitespace();
            if lexer.is_eof() {
                break;
            }
            if lexer.peek() == Some('#') && lexer.is_line_directive_start() {
                lexer.read_comment();
                continue;
            }
            if lexer.peek() == Some('/') && lexer.peek_next() == Some('/') {
                lexer.read_comment();
                continue;
            }
            let token = lexer.next_token()?;
            tokens.push(token);
        }
        tokens.push(Token {
            kind: TokenKind::Eof,
        });
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, CanonError> {
        self.skip_whitespace();
        let ch = match self.peek() {
            Some(ch) => ch,
            None => {
                return Ok(Token {
                    kind: TokenKind::Eof,
                })
            }
        };

        if ch == '~' && self.peek_next() == Some('~') && self.peek_n(2) == Some('>') {
            self.bump();
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::SignalArrow,
            });
        }

        if ch == '\n' || ch == '\r' {
            self.consume_newline();
            return Ok(Token {
                kind: TokenKind::Newline,
            });
        }

        if ch == '.' && self.peek_next() == Some('.') && self.peek_n(2) == Some('=') {
            self.bump();
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::DotDotEq,
            });
        }
        if ch == '.' && self.peek_next() == Some('.') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::DotDot,
            });
        }
        if ch == '.' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Dot,
            });
        }
        if ch == '+' && self.peek_next() == Some('<') && self.peek_n(2) == Some('-') {
            self.bump();
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::PlusArrow,
            });
        }
        if ch == '+' && self.peek_next() == Some('=') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::PlusEqual,
            });
        }
        if ch == '+' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Plus,
            });
        }
        if ch == '-' && self.peek_next() == Some('<') && self.peek_n(2) == Some('-') {
            self.bump();
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::MinusArrow,
            });
        }
        if ch == '-' && self.peek_next() == Some('=') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::MinusEqual,
            });
        }
        if ch == '-' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Minus,
            });
        }
        if ch == '*' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Star,
            });
        }
        if ch == '/' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Slash,
            });
        }
        if ch == '%' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Percent,
            });
        }
        if ch == '&' && self.peek_next() == Some('&') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::And,
            });
        }
        if ch == '|' && self.peek_next() == Some('|') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::Or,
            });
        }
        if ch == '|' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Pipe,
            });
        }
        if ch == '?' && self.peek_next() == Some('?') {
            self.bump();
            self.bump();
            if self.peek() == Some('{') {
                let body = self.lex_brace_block()?;
                return Ok(Token {
                    kind: TokenKind::PromptBlock(body),
                });
            }
            return Ok(Token {
                kind: TokenKind::Prompt,
            });
        }
        if ch == '?' {
            if !self.peek_next().map(is_ident_continue).unwrap_or(false) {
                self.bump();
                return Ok(Token {
                    kind: TokenKind::Question,
                });
            }
        }
        if ch == '!' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Bang,
            });
        }
        if ch == '@' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::At,
            });
        }
        if ch == ',' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Comma,
            });
        }
        if ch == ':' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Colon,
            });
        }
        if ch == '[' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::LBracket,
            });
        }
        if ch == ']' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::RBracket,
            });
        }
        if ch == '{' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::LBrace,
            });
        }
        if ch == '}' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::RBrace,
            });
        }
        if ch == '=' {
            if self.peek_next() == Some('=') {
                self.bump();
                self.bump();
                return Ok(Token {
                    kind: TokenKind::EqEq,
                });
            }
            self.bump();
            return Ok(Token {
                kind: TokenKind::Equals,
            });
        }
        if ch == '!' && self.peek_next() == Some('=') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::NotEq,
            });
        }
        if ch == '(' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::LParen,
            });
        }
        if ch == ')' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::RParen,
            });
        }
        if ch == '<' && self.peek_next() == Some('-') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::Arrow,
            });
        }
        if ch == '<' && self.peek_next() == Some('=') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::Lte,
            });
        }
        if ch == '>' && self.peek_next() == Some('=') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::Gte,
            });
        }
        if ch == '<' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Lt,
            });
        }
        if ch == '>' {
            self.bump();
            return Ok(Token {
                kind: TokenKind::Gt,
            });
        }
        if ch == '#' {
            let text = self.lex_atom()?;
            return Ok(Token {
                kind: TokenKind::Atom(text),
            });
        }
        if ch == '"' {
            let text = self.lex_string()?;
            return Ok(Token {
                kind: TokenKind::String(text),
            });
        }
        if ch.is_ascii_digit() {
            let number = self.lex_number()?;
            return Ok(Token {
                kind: TokenKind::Number(number),
            });
        }
        if is_ident_start(ch) {
            let ident = self.lex_ident();
            if ident == "글무늬"
                || ident == "수식"
                || ident == "보개마당"
                || ident == "보개장면"
                || ident == "실행정책"
                || ident == "짜임"
                || ident == "구성"
            {
                let checkpoint = self.pos;
                self.skip_inline_whitespace();
                if self.peek() == Some('{') {
                    let body = self.lex_brace_block()?;
                    let kind = match ident.as_str() {
                        "글무늬" => TokenKind::Template(body),
                        "수식" => TokenKind::Formula(body),
                        "보개마당" => TokenKind::BogaeMadangBlock(body),
                        "보개장면" => TokenKind::BogaeJangmyeonBlock(body),
                        "실행정책" => TokenKind::ExecPolicyBlock(body),
                        "짜임" => TokenKind::JjaimBlock(body),
                        _ => TokenKind::GuseongBlock(body),
                    };
                    return Ok(Token { kind });
                }
                self.pos = checkpoint;
            }
            let kind = match ident.as_str() {
                "보여주기" => TokenKind::Show,
                "톺아보기" | "감사" => TokenKind::Inspect,
                "일때" => TokenKind::Ilttae,
                "아니면" => TokenKind::Aniramyeon,
                "아니고" => TokenKind::Anigo,
                "맞으면" => TokenKind::Majeumyeon,
                "바탕으로" => TokenKind::Jeonjehae,
                "전제하에" => TokenKind::Jeonjehae,
                "다짐하고" => TokenKind::Bojanghago,
                "보장하고" => TokenKind::Bojanghago,
                "고르기" => TokenKind::Goreugi,
                "되풀이" | "반복" => TokenKind::Repeat,
                "동안" => TokenKind::During,
                "대해" => TokenKind::Daehae,
                "멈추기" => TokenKind::Break,
                "그리고" => TokenKind::And,
                "또는" => TokenKind::Or,
                "참" => TokenKind::True,
                "거짓" => TokenKind::False,
                "없음" => TokenKind::None,
                _ => TokenKind::Ident(ident),
            };
            return Ok(Token { kind });
        }

        Err(CanonError::new(
            "E_CANON_BAD_CHAR",
            format!("알 수 없는 문자: {}", ch),
        ))
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\u{feff}' || ch == ' ' || ch == '\t' || ch == '\r' {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn skip_inline_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\u{feff}' || ch == ' ' || ch == '\t' || ch == '\r' {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn read_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.bump();
        }
    }

    fn consume_newline(&mut self) {
        if self.peek() == Some('\r') {
            self.bump();
            if self.peek() == Some('\n') {
                self.bump();
            }
            return;
        }
        if self.peek() == Some('\n') {
            self.bump();
        }
    }

    fn lex_ident(&mut self) -> String {
        let mut buf = String::new();
        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                buf.push(ch);
                self.bump();
            } else {
                break;
            }
        }
        buf
    }

    fn lex_number(&mut self) -> Result<String, CanonError> {
        let mut buf = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                buf.push(ch);
                self.bump();
            } else {
                break;
            }
        }
        if self.peek() == Some('.')
            && self
                .peek_next()
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(false)
        {
            buf.push('.');
            self.bump();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    buf.push(ch);
                    self.bump();
                } else {
                    break;
                }
            }
        }
        if buf.is_empty() {
            return Err(CanonError::new(
                "E_CANON_BAD_NUMBER",
                "숫자 파싱 실패: 빈 숫자입니다.",
            ));
        }
        Ok(buf)
    }

    fn lex_atom(&mut self) -> Result<String, CanonError> {
        if self.peek() != Some('#') {
            return Err(CanonError::new(
                "E_CANON_BAD_ATOM",
                "원자 리터럴이 아닙니다.",
            ));
        }
        self.bump();
        let mut buf = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || matches!(ch, ',' | ':' | '(' | ')' | '[' | ']' | '{' | '}') {
                break;
            }
            if ch == '.' {
                if matches!(
                    self.peek_next(),
                    None | Some(' ') | Some('\t') | Some('\r') | Some('\n')
                ) {
                    break;
                }
            }
            buf.push(ch);
            self.bump();
        }
        if buf.is_empty() {
            return Err(CanonError::new(
                "E_CANON_BAD_ATOM",
                "원자 이름이 필요합니다.",
            ));
        }
        Ok(buf)
    }

    fn lex_string(&mut self) -> Result<String, CanonError> {
        let mut buf = String::new();
        if self.peek() != Some('"') {
            return Err(CanonError::new("E_CANON_BAD_STRING", "문자열이 아닙니다."));
        }
        self.bump();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.bump();
                return Ok(buf);
            }
            if ch == '\\' {
                self.bump();
                let escaped = match self.peek() {
                    Some('"') => '"',
                    Some('\\') => '\\',
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some(other) => {
                        return Err(CanonError::new(
                            "E_CANON_BAD_ESCAPE",
                            format!("알 수 없는 이스케이프: {}", other),
                        ))
                    }
                    None => {
                        return Err(CanonError::new(
                            "E_CANON_UNTERM_STRING",
                            "문자열이 끝나지 않았습니다.",
                        ))
                    }
                };
                self.bump();
                buf.push(escaped);
                continue;
            }
            if ch == '\r' {
                self.bump();
                if self.peek() == Some('\n') {
                    self.bump();
                }
                buf.push('\n');
                continue;
            }
            if ch == '\n' {
                self.bump();
                buf.push('\n');
                continue;
            }
            buf.push(ch);
            self.bump();
        }
        Err(CanonError::new(
            "E_CANON_UNTERM_STRING",
            "문자열이 끝나지 않았습니다.",
        ))
    }

    fn lex_brace_block(&mut self) -> Result<String, CanonError> {
        if self.peek() != Some('{') {
            return Err(CanonError::new(
                "E_CANON_BAD_BLOCK",
                "중괄호 블록이 필요합니다.",
            ));
        }
        let mut depth = 0usize;
        let mut buf = String::new();
        while let Some(ch) = self.peek() {
            if ch == '{' {
                depth += 1;
                self.bump();
                if depth > 1 {
                    buf.push('{');
                }
                continue;
            }
            if ch == '}' {
                depth -= 1;
                self.bump();
                if depth == 0 {
                    return Ok(buf);
                }
                buf.push('}');
                continue;
            }
            buf.push(ch);
            self.bump();
        }
        Err(CanonError::new(
            "E_CANON_UNTERM_BLOCK",
            "중괄호 블록이 끝나지 않았습니다.",
        ))
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn peek_n(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn is_line_directive_start(&self) -> bool {
        let mut idx = self.pos;
        while idx > 0 {
            let ch = self.chars[idx - 1];
            if ch == '\n' {
                return true;
            }
            if !matches!(ch, ' ' | '\t' | '\r' | '\u{feff}') {
                return false;
            }
            idx -= 1;
        }
        true
    }

    fn bump(&mut self) {
        self.pos += 1;
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch == '?' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch == '?' || ch == '~' || ch.is_alphanumeric()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclKind {
    Gureut,
    Butbak,
}

#[derive(Debug, Clone)]
struct DeclItem {
    name: String,
    kind: DeclKind,
    type_name: String,
    value: Option<Expr>,
    maegim: Option<MaegimSpec>,
}

#[derive(Debug, Clone)]
struct MaegimSpec {
    fields: Vec<Binding>,
}

#[derive(Debug, Clone)]
enum SurfaceStmt {
    RootDecl {
        items: Vec<DeclItem>,
    },
    Decl {
        name: String,
        type_name: String,
        value: Expr,
    },
    SeedDef {
        name: String,
        kind: String,
        params: Vec<Param>,
        body: Vec<SurfaceStmt>,
    },
    Assign {
        target: Path,
        value: Expr,
    },
    Show {
        value: Expr,
    },
    Inspect {
        value: Expr,
    },
    BogaeMadangBlock {
        body: String,
    },
    JjaimBlock {
        body: String,
    },
    BogaeDraw,
    If {
        condition: Condition,
        then_body: Vec<SurfaceStmt>,
        else_body: Option<Vec<SurfaceStmt>>,
    },
    Contract {
        kind: ContractKind,
        mode: ContractMode,
        condition: Condition,
        then_body: Option<Vec<SurfaceStmt>>,
        else_body: Vec<SurfaceStmt>,
    },
    Choose {
        branches: Vec<SurfaceChooseBranch>,
        else_body: Vec<SurfaceStmt>,
    },
    PromptChoose {
        branches: Vec<SurfaceChooseBranch>,
        else_body: Option<Vec<SurfaceStmt>>,
    },
    PromptAfter {
        value: Expr,
        body: Vec<SurfaceStmt>,
    },
    PromptCondition {
        condition: Condition,
        body: Vec<SurfaceStmt>,
    },
    PromptBlock {
        body: Vec<SurfaceStmt>,
    },
    ExecPolicyBlock {
        body: String,
    },
    Repeat {
        body: Vec<SurfaceStmt>,
    },
    While {
        condition: Condition,
        body: Vec<SurfaceStmt>,
    },
    ForEach {
        item: String,
        iterable: Expr,
        body: Vec<SurfaceStmt>,
    },
    Hook {
        name: String,
        suffix: HookSuffix,
        body: Vec<SurfaceStmt>,
    },
    Receive {
        kind: Option<String>,
        binding: Option<String>,
        condition: Option<Expr>,
        body: Vec<SurfaceStmt>,
    },
    EventReact {
        kind: String,
        body: Vec<SurfaceStmt>,
    },
    OpenBlock {
        body: Vec<SurfaceStmt>,
    },
    Return {
        value: Expr,
    },
    Send {
        sender: Option<Expr>,
        payload: Expr,
        receiver: Expr,
    },
    Expr {
        value: Expr,
    },
    Break,
}

#[derive(Debug, Clone)]
enum Stmt {
    RootDecl {
        items: Vec<DeclItem>,
    },
    Assign {
        target: Path,
        value: Expr,
    },
    SeedDef {
        name: String,
        kind: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    Show {
        value: Expr,
    },
    Inspect {
        value: Expr,
    },
    BogaeMadangBlock {
        body: String,
    },
    JjaimBlock {
        body: String,
    },
    BogaeDraw,
    If {
        condition: Condition,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    Contract {
        kind: ContractKind,
        mode: ContractMode,
        condition: Condition,
        then_body: Option<Vec<Stmt>>,
        else_body: Vec<Stmt>,
    },
    Choose {
        branches: Vec<ChooseBranch>,
        else_body: Vec<Stmt>,
    },
    PromptChoose {
        branches: Vec<ChooseBranch>,
        else_body: Option<Vec<Stmt>>,
    },
    PromptAfter {
        value: Expr,
        body: Vec<Stmt>,
    },
    PromptCondition {
        condition: Condition,
        body: Vec<Stmt>,
    },
    PromptBlock {
        body: Vec<Stmt>,
    },
    ExecPolicyBlock {
        body: String,
    },
    Repeat {
        body: Vec<Stmt>,
    },
    While {
        condition: Condition,
        body: Vec<Stmt>,
    },
    ForEach {
        item: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Hook {
        name: String,
        suffix: HookSuffix,
        body: Vec<Stmt>,
    },
    Receive {
        kind: Option<String>,
        binding: Option<String>,
        condition: Option<Expr>,
        body: Vec<Stmt>,
    },
    EventReact {
        kind: String,
        body: Vec<Stmt>,
    },
    OpenBlock {
        body: Vec<Stmt>,
    },
    Return {
        value: Expr,
    },
    Send {
        sender: Option<Expr>,
        payload: Expr,
        receiver: Expr,
    },
    Expr {
        value: Expr,
    },
    Break,
}

#[derive(Debug, Clone, Copy)]
enum HookSuffix {
    Halttae,
    Mada,
}

#[derive(Debug, Clone)]
enum Expr {
    Literal(Literal),
    Resource(String),
    Template {
        body: String,
    },
    TemplateApply {
        bindings: Vec<Binding>,
        body: String,
    },
    Formula {
        tag: Option<String>,
        body: String,
    },
    PromptExpr {
        expr: Box<Expr>,
    },
    PromptBlock {
        body: String,
    },
    Path(Path),
    FieldAccess {
        target: Box<Expr>,
        field: String,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    CallIn {
        name: String,
        bindings: Vec<Binding>,
    },
    Pack {
        bindings: Vec<Binding>,
    },
    Unit {
        value: Box<Expr>,
        unit: String,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Pipe {
        left: Box<Expr>,
        kind: PipeKind,
        right: Box<Expr>,
    },
    SeedLiteral {
        param: String,
        body: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    value: Expr,
}

#[derive(Debug, Clone)]
struct Param {
    name: String,
    type_name: Option<String>,
    default: Option<Expr>,
}

#[derive(Debug, Clone)]
struct Condition {
    expr: Expr,
    style: ConditionStyle,
    negated: bool,
}

#[derive(Debug, Clone, Copy)]
enum ConditionStyle {
    Plain,
    Thunk,
}

#[derive(Debug, Clone, Copy)]
enum ContractKind {
    Pre,
    Post,
}

#[derive(Debug, Clone, Copy)]
enum ContractMode {
    Abort,
    Alert,
}

#[derive(Debug, Clone)]
struct ChooseBranch {
    condition: Condition,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct SurfaceChooseBranch {
    condition: Condition,
    body: Vec<SurfaceStmt>,
}

#[derive(Debug, Clone, Copy)]
enum UnaryOp {
    Neg,
}

#[derive(Debug, Clone, Copy)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone, Copy)]
enum PipeKind {
    Haseo,
    Hago,
}

#[derive(Debug, Clone)]
enum Literal {
    Str(String),
    Num(String),
    Bool(bool),
    None,
    Atom(String),
}

#[derive(Debug, Clone)]
struct Path {
    segments: Vec<String>,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    bridge: bool,
    deprecated_block_header_colon_count: usize,
    seed_kind_stack: Vec<String>,
}

impl Parser {
    fn new(tokens: Vec<Token>, bridge: bool) -> Self {
        Self {
            tokens,
            pos: 0,
            bridge,
            deprecated_block_header_colon_count: 0,
            seed_kind_stack: Vec::new(),
        }
    }

    fn deprecated_block_header_colon_count(&self) -> usize {
        self.deprecated_block_header_colon_count
    }

    fn unexpected_token_error(&self, context: &str) -> CanonError {
        CanonError::new(
            "E_CANON_UNEXPECTED_TOKEN",
            format!(
                "{}; at_token_index={} found={:?}",
                context,
                self.pos,
                self.peek_kind()
            ),
        )
    }

    fn parse_program(&mut self) -> Result<Vec<SurfaceStmt>, CanonError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_separators();
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                break;
            }
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }
        Ok(stmts)
    }

    fn in_imja_seed_body(&self) -> bool {
        self.seed_kind_stack
            .last()
            .is_some_and(|kind| kind == "임자")
    }

    fn parse_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        if self.is_seed_def_start() {
            return self.parse_seed_def();
        }

        if let Some(kind) = self.peek_decl_block_kind() {
            return self.parse_decl_block(kind);
        }

        if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
            if self.peek_n_is(1, |k| matches!(k, TokenKind::Colon)) {
                return self.parse_prompt_choose_stmt();
            }
            if self.peek_n_is(1, |k| matches!(k, TokenKind::LBrace)) {
                return self.parse_prompt_block_stmt();
            }
        }

        if self.peek_is_ident_colon() {
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let type_name = self.expect_ident()?;
            self.expect(TokenKind::Equals)?;
            let value = self.parse_expr()?;
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Decl {
                name,
                type_name,
                value,
            });
        }

        if self.is_open_block_start() {
            return self.parse_open_block_stmt();
        }

        if self.peek_is(|k| matches!(k, TokenKind::Repeat)) {
            return self.parse_repeat_stmt();
        }

        if self.peek_is(|k| matches!(k, TokenKind::Break)) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Break);
        }

        if self.peek_is(|k| matches!(k, TokenKind::Goreugi)) {
            return self.parse_choose_stmt();
        }

        if self.is_hook_start() {
            return self.parse_hook_stmt();
        }

        if self.is_foreach_start() {
            return self.parse_foreach_stmt();
        }

        if self.peek_is(|k| {
            matches!(
                k,
                TokenKind::BogaeMadangBlock(_) | TokenKind::BogaeJangmyeonBlock(_)
            )
        }) {
            return self.parse_bogae_madang_block_stmt();
        }
        if self.peek_is(|k| matches!(k, TokenKind::ExecPolicyBlock(_))) {
            return self.parse_exec_policy_block_stmt();
        }
        if self.peek_is(|k| matches!(k, TokenKind::JjaimBlock(_) | TokenKind::GuseongBlock(_))) {
            return self.parse_jjaim_block_stmt();
        }

        if self.is_bogae_draw_stmt() {
            self.advance();
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::BogaeDraw);
        }

        if self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            let condition = self.parse_condition_expr(false)?;
            if self.peek_is(|k| matches!(k, TokenKind::Ilttae)) {
                return self.parse_if_stmt(condition);
            }
            if self.peek_is(|k| matches!(k, TokenKind::During)) {
                return self.parse_while_stmt(condition);
            }
            if self.peek_is(|k| matches!(k, TokenKind::Jeonjehae | TokenKind::Bojanghago)) {
                return self.parse_contract_stmt(condition);
            }
            if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
                return self.parse_prompt_condition_stmt(condition);
            }
            return Err(CanonError::new(
                "E_CANON_EXPECTED_ILTTTAE",
                "일때 또는 동안/바탕으로/다짐하고가 필요합니다.",
            ));
        }

        if let Some(stmt) = self.try_parse_event_react_stmt()? {
            return Ok(stmt);
        }

        if let Some(stmt) = self.try_parse_receive_stmt()? {
            return Ok(stmt);
        }

        let expr = self.parse_expr()?;
        let mut sender = None;
        let mut payload = expr;
        if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "의")) {
            self.advance();
            sender = Some(payload);
            payload = self.parse_expr()?;
        }
        if self.peek_is(|k| matches!(k, TokenKind::SignalArrow)) {
            self.advance();
            let receiver = self.parse_expr()?;
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Send {
                sender,
                payload,
                receiver,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Ilttae)) {
            return self.parse_if_stmt(Condition {
                expr: payload,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Jeonjehae | TokenKind::Bojanghago)) {
            return self.parse_contract_stmt(Condition {
                expr: payload,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
            return self.parse_prompt_after_stmt(payload);
        }
        if self.peek_is(|k| matches!(k, TokenKind::PlusEqual | TokenKind::MinusEqual)) {
            return Err(CanonError::new(
                "E_CANON_UNSUPPORTED_COMPOUND_UPDATE",
                "+=/-=는 미지원입니다. +<-/ -<-를 사용하세요.",
            ));
        }

        if self.peek_is(|k| {
            matches!(
                k,
                TokenKind::Arrow | TokenKind::PlusArrow | TokenKind::MinusArrow
            )
        }) {
            let op = match self.peek_kind() {
                TokenKind::PlusArrow => Some(BinaryOp::Add),
                TokenKind::MinusArrow => Some(BinaryOp::Sub),
                _ => None,
            };
            self.advance();
            let mut consumed_dot = false;
            let value = if self.peek_is(|k| matches!(k, TokenKind::Dot)) {
                self.advance();
                consumed_dot = true;
                Expr::Literal(Literal::None)
            } else {
                self.parse_expr()?
            };

            let stmt = if let Some(op) = op {
                let Expr::Path(path) = payload else {
                    return Err(CanonError::new(
                        "E_CANON_UNSUPPORTED_COMPOUND_TARGET",
                        "복합 갱신(+<-, -<-)은 이름 대상만 허용됩니다.",
                    ));
                };
                let left = Expr::Path(path.clone());
                let value_expr = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(value),
                };
                SurfaceStmt::Assign {
                    target: path,
                    value: value_expr,
                }
            } else {
                match payload {
                    Expr::Call { name, args } if name == "차림.값" && args.len() == 2 => {
                        let target = match &args[0] {
                            Expr::Path(path)
                                if path.segments.len() == 2
                                    && matches!(path.segments[0].as_str(), "살림" | "바탕") =>
                            {
                                path.clone()
                            }
                            _ => {
                                return Err(CanonError::new(
                                    "E_CANON_EXPECTED_TARGET",
                                    "대상이 필요합니다.",
                                ))
                            }
                        };
                        let index_expr = args[1].clone();
                        let value_call = Expr::Call {
                            name: "차림.바꾼값".to_string(),
                            args: vec![Expr::Path(target.clone()), index_expr, value],
                        };
                        SurfaceStmt::Assign {
                            target,
                            value: value_call,
                        }
                    }
                    Expr::FieldAccess { target, field } => {
                        let Expr::Path(path) = *target else {
                            return Err(CanonError::new(
                                "E_CANON_EXPECTED_TARGET",
                                "점 대입 대상은 이름만 허용됩니다.",
                            ));
                        };
                        let key_expr = Expr::Literal(Literal::Str(field));
                        let value_call = Expr::Call {
                            name: "짝맞춤.바꾼값".to_string(),
                            args: vec![Expr::Path(path.clone()), key_expr, value],
                        };
                        SurfaceStmt::Assign {
                            target: path,
                            value: value_call,
                        }
                    }
                    Expr::Path(path) => {
                        if let Some((target, key_segments)) = Self::split_map_dot_target(&path) {
                            let value_call = Self::build_map_dot_write_expr(
                                Expr::Path(target.clone()),
                                &key_segments,
                                value,
                            );
                            SurfaceStmt::Assign {
                                target,
                                value: value_call,
                            }
                        } else {
                            SurfaceStmt::Assign {
                                target: path,
                                value,
                            }
                        }
                    }
                    _ => {
                        return Err(CanonError::new(
                            "E_CANON_EXPECTED_TARGET",
                            "대상이 필요합니다.",
                        ))
                    }
                }
            };

            if !consumed_dot {
                self.consume_terminator()?;
            } else {
                self.skip_newlines();
            }
            return Ok(stmt);
        }
        if self.peek_is(|k| matches!(k, TokenKind::Show)) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Show { value: payload });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Inspect)) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Inspect { value: payload });
        }
        if self.peek_is(|k| {
            matches!(
                k,
                TokenKind::Ident(name)
                    if name == "되돌림" || name == "반환" || name == "돌려줘"
            )
        }) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Return { value: payload });
        }
        if matches!(payload, Expr::Call { .. } | Expr::CallIn { .. }) {
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Expr { value: payload });
        }
        Err(self.unexpected_token_error("예상하지 못한 토큰입니다"))
    }

    fn try_parse_event_react_stmt(&mut self) -> Result<Option<SurfaceStmt>, CanonError> {
        if let Some(alias) = self.detect_forbidden_event_alias() {
            return Err(CanonError::new(
                "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
                format!(
                    "비정본 이벤트 문형 금지({}) — 정본은 \"KIND\"라는 알림이 오면 {{ ... }}.",
                    alias
                ),
            ));
        }

        let TokenKind::String(kind) = self.peek_kind() else {
            return Ok(None);
        };
        if !self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(text) if text == "라는")) {
            return Ok(None);
        }
        if !self.peek_n_is(
            2,
            |k| matches!(k, TokenKind::Ident(text) if is_event_noun_canonical(text)),
        ) {
            return Ok(None);
        }
        if !self.peek_n_is(3, |k| matches!(k, TokenKind::Ident(text) if text == "오면")) {
            return Ok(None);
        }

        self.advance(); // "KIND"
        self.advance(); // 라는
        self.advance(); // 알림/알림이
        self.advance(); // 오면
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(Some(SurfaceStmt::EventReact { kind, body }))
    }

    fn try_parse_receive_stmt(&mut self) -> Result<Option<SurfaceStmt>, CanonError> {
        let checkpoint = self.pos;
        let mut binding = None;
        let mut condition = None;

        if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            if !self.has_receive_binding_head() {
                return Ok(None);
            }
            self.advance();
            let name = self.expect_ident()?;
            binding = Some(name);
            if !self.peek_is(|k| matches!(k, TokenKind::RParen)) {
                condition = Some(self.parse_expr()?);
            }
            self.expect(TokenKind::RParen)?;
            if !self.peek_is(|k| matches!(k, TokenKind::Ident(text) if text == "인")) {
                self.pos = checkpoint;
                return Ok(None);
            }
            self.advance();
        }

        let Some(kind) = self.peek_receive_kind_name() else {
            self.pos = checkpoint;
            return Ok(None);
        };
        self.advance();
        if !self.peek_is(|k| matches!(k, TokenKind::Ident(text) if text == "받으면")) {
            self.pos = checkpoint;
            return Ok(None);
        }
        if !self.in_imja_seed_body() {
            return Err(CanonError::new(
                "E_CANON_RECEIVE_OUTSIDE_IMJA",
                "`받으면` 훅은 `임자` 본문 안에서만 사용할 수 있습니다.",
            ));
        }
        self.advance();
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(Some(SurfaceStmt::Receive {
            kind,
            binding,
            condition,
            body,
        }))
    }

    fn has_receive_binding_head(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            return false;
        }
        let mut depth = 0usize;
        let mut index = self.pos;
        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return self.tokens.get(index + 1).is_some_and(
                            |next| matches!(&next.kind, TokenKind::Ident(text) if text == "인"),
                        );
                    }
                }
                _ => {}
            }
            index += 1;
        }
        false
    }

    fn peek_receive_kind_name(&self) -> Option<Option<String>> {
        let TokenKind::Ident(text) = self.peek_kind() else {
            return None;
        };
        let kind = strip_receive_object_particle(&text)?;
        if kind == "알림" {
            Some(None)
        } else {
            Some(Some(kind))
        }
    }

    fn detect_forbidden_event_alias(&self) -> Option<&'static str> {
        match self.peek_kind() {
            TokenKind::Ident(ref text) if is_event_noun_any(text) => {
                if self.peek_n_is(1, |k| matches!(k, TokenKind::String(_))) {
                    return Some("prefix_form");
                }
            }
            TokenKind::String(_) => {
                if self.peek_n_is(1, |k| matches!(k, TokenKind::Ilttae))
                    || (self
                        .peek_n_is(1, |k| matches!(k, TokenKind::Ident(text) if text == "일때"))
                        && self.peek_n_is(2, |k| matches!(k, TokenKind::Colon)))
                {
                    return Some("ilttae_form");
                }
                if self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(text) if text == "라는"))
                    && self.peek_n_is(
                        2,
                        |k| matches!(k, TokenKind::Ident(text) if is_event_noun_alias(text)),
                    )
                {
                    return Some("noun_alias");
                }
            }
            _ => {}
        }
        None
    }

    fn peek_decl_block_kind(&self) -> Option<DeclKind> {
        if !self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "채비")) {
            return None;
        }
        if self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace | TokenKind::Colon)) {
            return Some(DeclKind::Gureut);
        }
        None
    }

    fn parse_decl_block(&mut self, _kind: DeclKind) -> Result<SurfaceStmt, CanonError> {
        let keyword = self.expect_ident()?;
        if keyword != "채비" {
            return Err(CanonError::new(
                "E_CANON_DECL_HEADER_ONLY_CHAEVI",
                "선언 블록 머릿말은 `채비 {`만 사용합니다",
            ));
        }
        self.consume_optional_block_header_colon();
        self.expect(TokenKind::LBrace)?;

        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_RBRACE",
                    "닫는 중괄호가 필요합니다.",
                ));
            }

            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let type_name = self.expect_ident()?;

            let mut value = None;
            let mut maegim = None;
            let mut kind = DeclKind::Gureut;
            if self.peek_is(|k| matches!(k, TokenKind::Arrow)) {
                self.advance();
                let (expr, item_maegim) = self.parse_decl_item_value_and_maegim()?;
                value = Some(expr);
                maegim = item_maegim;
            } else if self.peek_is(|k| matches!(k, TokenKind::Equals)) {
                kind = DeclKind::Butbak;
                self.advance();
                let (expr, item_maegim) = self.parse_decl_item_value_and_maegim()?;
                value = Some(expr);
                maegim = item_maegim;
            }

            self.consume_terminator()?;
            items.push(DeclItem {
                name,
                kind,
                type_name,
                value,
                maegim,
            });
        }

        self.expect(TokenKind::RBrace)?;
        self.consume_terminator()?;
        Ok(SurfaceStmt::RootDecl { items })
    }

    fn parse_decl_item_value_and_maegim(
        &mut self,
    ) -> Result<(Expr, Option<MaegimSpec>), CanonError> {
        if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            let checkpoint = self.pos;
            if let Ok(grouped) = self.try_parse_grouped_expr_for_maegim() {
                if self.peek_maegim_keyword() {
                    let maegim = self.parse_maegim_spec()?;
                    return Ok((grouped, Some(maegim)));
                }
            }
            self.pos = checkpoint;
        }

        let expr = self.parse_expr()?;
        if self.peek_maegim_keyword() {
            return Err(CanonError::new(
                "E_CANON_MAEGIM_GROUPED_VALUE_REQUIRED",
                "매김은 `(<식>) 매김 { ... }` 형태만 허용됩니다.",
            ));
        }
        Ok((expr, None))
    }

    fn try_parse_grouped_expr_for_maegim(&mut self) -> Result<Expr, CanonError> {
        self.expect(TokenKind::LParen)?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        Ok(expr)
    }

    fn peek_maegim_keyword(&self) -> bool {
        self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "조건" || name == "매김"))
    }

    fn is_supported_maegim_nested_section(name: &str) -> bool {
        matches!(name, "가늠" | "갈래")
    }

    fn is_supported_maegim_nested_field(section: &str, field: &str) -> bool {
        match section {
            "가늠" => matches!(field, "범위" | "간격" | "분할수"),
            "갈래" => matches!(field, "가만히" | "벗남다룸"),
            _ => false,
        }
    }

    fn parse_maegim_spec(&mut self) -> Result<MaegimSpec, CanonError> {
        let keyword = self.expect_ident()?;
        if keyword != "조건" && keyword != "매김" {
            return Err(CanonError::new(
                "E_CANON_BAD_MAEGIM",
                "매김 블록은 `조건 {` 또는 `매김 {`만 허용됩니다.",
            ));
        }
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        let mut has_step = false;
        let mut has_split_count = false;
        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_RBRACE",
                    "닫는 중괄호가 필요합니다.",
                ));
            }
            let name = self.expect_ident()?;
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
                if !Self::is_supported_maegim_nested_section(&name) {
                    return Err(CanonError::new(
                        "E_CANON_MAEGIM_NESTED_SECTION_UNSUPPORTED",
                        format!(
                            "`매김` 중첩 섹션은 `가늠`/`갈래`만 허용됩니다: {}",
                            name
                        ),
                    ));
                }
                self.expect(TokenKind::LBrace)?;
                loop {
                    self.skip_newlines();
                    if self.peek_is(|k| matches!(k, TokenKind::RBrace)) {
                        break;
                    }
                    if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                        return Err(CanonError::new(
                            "E_CANON_EXPECTED_RBRACE",
                            "닫는 중괄호가 필요합니다.",
                        ));
                    }
                    let nested_name = self.expect_ident()?;
                    self.expect(TokenKind::Colon)?;
                    let value = self.parse_expr()?;
                    if !Self::is_supported_maegim_nested_field(&name, &nested_name) {
                        return Err(CanonError::new(
                            "E_CANON_MAEGIM_NESTED_FIELD_UNSUPPORTED",
                            format!(
                                "`매김` 섹션 `{}`에서 지원하지 않는 항목입니다: {}",
                                name, nested_name
                            ),
                        ));
                    }
                    if nested_name == "간격" {
                        has_step = true;
                    } else if nested_name == "분할수" {
                        has_split_count = true;
                    }
                    if has_step && has_split_count {
                        return Err(CanonError::new(
                            "E_CANON_MAEGIM_STEP_SPLIT_CONFLICT",
                            "`매김`에서 `간격`과 `분할수`는 동시에 사용할 수 없습니다.",
                        ));
                    }
                    self.consume_terminator()?;
                    fields.push(Binding {
                        name: format!("{}.{}", name, nested_name),
                        value,
                    });
                }
                self.expect(TokenKind::RBrace)?;
                if self.peek_is(|k| matches!(k, TokenKind::Dot | TokenKind::Newline)) {
                    self.consume_terminator()?;
                } else if !self.peek_is(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
                    return Err(CanonError::new(
                        "E_CANON_EXPECTED_TERMINATOR",
                        "문장 끝 구분자(점 또는 줄바꿈)가 필요합니다.",
                    ));
                }
            } else {
                self.expect(TokenKind::Colon)?;
                let value = self.parse_expr()?;
                if name == "간격" {
                    has_step = true;
                } else if name == "분할수" {
                    has_split_count = true;
                }
                if has_step && has_split_count {
                    return Err(CanonError::new(
                        "E_CANON_MAEGIM_STEP_SPLIT_CONFLICT",
                        "`매김`에서 `간격`과 `분할수`는 동시에 사용할 수 없습니다.",
                    ));
                }
                self.consume_terminator()?;
                fields.push(Binding { name, value });
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(MaegimSpec { fields })
    }

    fn is_hook_start(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            return false;
        }
        let mut idx = self.pos + 1;
        let number_head = self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Number(_)))
            .unwrap_or(false)
            && self
                .tokens
                .get(idx + 1)
                .map(|t| matches!(t.kind, TokenKind::Ident(_)))
                .unwrap_or(false);
        let head_is_ident = self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Ident(_)))
            .unwrap_or(false);
        if head_is_ident {
            idx += 1;
        } else if number_head {
            idx += 2;
        } else {
            return false;
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::RParen))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        let Some(Token { kind }) = self.tokens.get(idx) else {
            return false;
        };
        let suffix_ok = if number_head {
            matches!(kind, TokenKind::Ident(_))
        } else {
            matches!(kind, TokenKind::Ident(name) if name == "할때" || name == "마다")
        };
        if !suffix_ok {
            return false;
        }
        idx += 1;
        self.hook_header_has_block_start(idx)
    }

    fn hook_header_has_block_start(&self, mut idx: usize) -> bool {
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        if self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Colon))
            .unwrap_or(false)
        {
            idx += 1;
            while self
                .tokens
                .get(idx)
                .map(|t| matches!(t.kind, TokenKind::Newline))
                .unwrap_or(false)
            {
                idx += 1;
            }
        }
        self.tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::LBrace))
            .unwrap_or(false)
    }

    fn parse_hook_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::LParen)?;
        let mut number_head = false;
        let mut name = if self.peek_is(|k| matches!(k, TokenKind::Number(_))) {
            number_head = true;
            let raw = match self.advance().kind {
                TokenKind::Number(value) => value,
                _ => unreachable!("number branch must consume number token"),
            };
            let interval = raw.parse::<u64>().ok().filter(|value| *value > 0).ok_or_else(|| {
                CanonError::new(
                    "E_CANON_HOOK_EVERY_N_MADI_INTERVAL_INVALID",
                    "(N마디)마다에서 N은 양의 정수여야 합니다.",
                )
            })?;
            let unit = match self.advance().kind {
                TokenKind::Ident(text) => text,
                other => {
                    return Err(CanonError::new(
                        "E_CANON_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED",
                        format!("(N마디)마다에서 단위는 `마디`만 허용됩니다: {other:?}"),
                    ))
                }
            };
            if unit != "마디" {
                return Err(CanonError::new(
                    "E_CANON_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED",
                    format!("(N마디)마다에서 단위는 `마디`만 허용됩니다: {unit}"),
                ));
            }
            format!("{}마디", interval)
        } else {
            self.expect_ident()?
        };
        self.expect(TokenKind::RParen)?;
        let raw_suffix = self.expect_ident()?;
        let suffix = match raw_suffix.as_str() {
            "할때" if number_head => {
                return Err(CanonError::new(
                    "E_CANON_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED",
                    format!("(N마디) 훅 접미는 `마다`만 허용됩니다: {raw_suffix}"),
                ))
            }
            "할때" => HookSuffix::Halttae,
            "마다" => HookSuffix::Mada,
            _ if number_head => {
                return Err(CanonError::new(
                    "E_CANON_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED",
                    format!("(N마디) 훅 접미는 `마다`만 허용됩니다: {raw_suffix}"),
                ))
            }
            _ => {
                return Err(CanonError::new(
                    "E_CANON_HOOK_SUFFIX",
                    "훅 접미는 할때/마다만 허용됩니다.",
                ))
            }
        };
        name = Self::canonicalize_hook_name(name, suffix);
        self.skip_newlines();
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            return Err(CanonError::new(
                "E_BLOCK_HEADER_COLON_FORBIDDEN",
                "훅 블록 헤더의 `:` 표기는 허용되지 않습니다. `(시작)할때 { ... }`처럼 사용하세요.",
            ));
        }
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Hook { name, suffix, body })
    }

    fn canonicalize_hook_name(name: String, suffix: HookSuffix) -> String {
        match (name.as_str(), suffix) {
            ("처음", HookSuffix::Halttae) => "시작".to_string(),
            ("매틱", HookSuffix::Mada) => "매마디".to_string(),
            _ => name,
        }
    }

    fn is_open_block_start(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::Ident(name) if is_open_block_keyword(name))) {
            return false;
        }
        self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace))
            || (self.peek_next_non_newline_is(|k| matches!(k, TokenKind::Colon))
                && self.peek_n_is(2, |k| matches!(k, TokenKind::LBrace)))
    }

    fn parse_open_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let name = self.expect_ident()?;
        if !is_open_block_keyword(&name) {
            return Err(CanonError::new(
                "E_CANON_OPEN_BLOCK",
                "너머 블록은 '너머'로 시작해야 합니다.",
            ));
        }
        if name != "너머" {
            return Err(CanonError::new(
                "E_EFFECT_SURFACE_ALIAS_FORBIDDEN",
                format!("비정본 너머 문형 금지({}) — 정본은 너머 {{ ... }}.", name),
            ));
        }
        self.consume_optional_block_header_colon();
        if !self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(CanonError::new(
                "E_CANON_EXPECTED_LBRACE",
                "너머 블록에는 '{'가 필요합니다.",
            ));
        }
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::OpenBlock { body })
    }

    fn is_seed_def_start(&self) -> bool {
        let mut idx = self.pos;
        if self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::LParen))
            == Some(true)
        {
            let mut depth = 1usize;
            idx += 1;
            while idx < self.tokens.len() && depth > 0 {
                match self.tokens[idx].kind {
                    TokenKind::LParen => depth += 1,
                    TokenKind::RParen => depth -= 1,
                    _ => {}
                }
                idx += 1;
            }
            if depth != 0 {
                return false;
            }
            while self
                .tokens
                .get(idx)
                .map(|t| matches!(t.kind, TokenKind::Newline))
                .unwrap_or(false)
            {
                idx += 1;
            }
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Ident(_)))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Colon))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Ident(_)))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Equals))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        self.tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::LBrace))
            .unwrap_or(false)
    }

    fn parse_seed_def(&mut self) -> Result<SurfaceStmt, CanonError> {
        let params = if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            self.parse_params()?
        } else {
            Vec::new()
        };
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let kind = self.expect_ident()?;
        self.expect(TokenKind::Equals)?;
        self.seed_kind_stack.push(kind.clone());
        let body = self.parse_block();
        self.seed_kind_stack.pop();
        let body = body?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::SeedDef {
            name,
            kind,
            params,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, CanonError> {
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if self.peek_is(|k| matches!(k, TokenKind::RParen)) {
            self.advance();
            return Ok(params);
        }
        loop {
            let name = self.expect_ident()?;
            let mut type_name = None;
            let mut default = None;
            if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
                self.advance();
                type_name = Some(self.expect_ident()?);
            }
            if self.peek_is(|k| matches!(k, TokenKind::Equals)) {
                self.advance();
                default = Some(self.parse_expr()?);
            }
            params.push(Param {
                name,
                type_name,
                default,
            });
            if self.peek_is(|k| matches!(k, TokenKind::Comma)) {
                self.advance();
                continue;
            }
            break;
        }
        self.expect(TokenKind::RParen)?;
        Ok(params)
    }

    fn expect_string(&mut self) -> Result<String, CanonError> {
        if let Some(TokenKind::String(value)) = self.peek() {
            self.advance();
            return Ok(value.clone());
        }
        Err(CanonError::new(
            "E_CANON_EXPECTED_STRING",
            "문자열이 필요합니다.",
        ))
    }

    fn parse_repeat_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.advance();
        self.consume_optional_block_header_colon();
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Repeat { body })
    }

    fn parse_while_stmt(&mut self, condition: Condition) -> Result<SurfaceStmt, CanonError> {
        self.advance();
        self.consume_optional_block_header_colon();
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::While { condition, body })
    }

    fn is_foreach_start(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            return false;
        }
        let mut idx = self.pos + 1;
        let Some(first) = self.tokens.get(idx) else {
            return false;
        };
        if !matches!(first.kind, TokenKind::Ident(_)) {
            return false;
        }
        idx += 1;
        if let Some(token) = self.tokens.get(idx) {
            if matches!(token.kind, TokenKind::Colon) {
                idx += 1;
                if !self
                    .tokens
                    .get(idx)
                    .map(|tok| matches!(tok.kind, TokenKind::Ident(_)))
                    .unwrap_or(false)
                {
                    return false;
                }
                idx += 1;
            }
        }
        if !self
            .tokens
            .get(idx)
            .map(|tok| matches!(tok.kind, TokenKind::RParen))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        let mut depth = 0usize;
        while let Some(token) = self.tokens.get(idx) {
            match &token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                TokenKind::Daehae if depth == 0 => {
                    return self
                        .tokens
                        .get(idx + 1)
                        .map(|tok| matches!(tok.kind, TokenKind::LBrace | TokenKind::Colon))
                        .unwrap_or(false);
                }
                TokenKind::Ident(name) if depth == 0 && name == "모두" => {
                    return self
                        .tokens
                        .get(idx + 1)
                        .map(|tok| matches!(tok.kind, TokenKind::LBrace))
                        .unwrap_or(false);
                }
                TokenKind::Dot => {
                    if !self
                        .tokens
                        .get(idx + 1)
                        .map(|tok| matches!(tok.kind, TokenKind::Ident(_)))
                        .unwrap_or(false)
                    {
                        break;
                    }
                    idx += 1;
                }
                TokenKind::Newline | TokenKind::RBrace | TokenKind::Eof => break,
                _ => {}
            }
            idx += 1;
        }
        false
    }

    fn parse_foreach_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let item = self.parse_foreach_var()?;
        let mut iterable = self.parse_expr()?;
        let is_modu = self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "모두"));
        if is_modu {
            self.advance();
        } else {
            self.expect(TokenKind::Daehae)?;
            if let Expr::Path(path) = &mut iterable {
                if let Some(last) = path.segments.last_mut() {
                    if let Some(trimmed) = last.strip_suffix('에') {
                        if !trimmed.is_empty() {
                            *last = trimmed.to_string();
                        }
                    }
                }
            }
            self.consume_optional_block_header_colon();
        }
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::ForEach {
            item,
            iterable,
            body,
        })
    }

    fn parse_foreach_var(&mut self) -> Result<String, CanonError> {
        self.expect(TokenKind::LParen)?;
        let name = self.expect_ident()?;
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.expect_ident()?;
        }
        self.expect(TokenKind::RParen)?;
        Ok(name)
    }

    fn parse_expr(&mut self) -> Result<Expr, CanonError> {
        self.parse_pipe()
    }

    fn consume_optional_block_header_colon(&mut self) -> bool {
        self.skip_newlines();
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
            self.deprecated_block_header_colon_count += 1;
            return true;
        }
        false
    }

    fn parse_pipe(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_logical_or()?;
        loop {
            let kind = match self.peek_ident_text() {
                Some(text) if text == "해서" => Some(PipeKind::Haseo),
                Some(text) if text == "하고" => Some(PipeKind::Hago),
                _ => None,
            };
            let Some(kind) = kind else { break };
            self.advance();
            let right = self.parse_logical_or()?;
            expr = Expr::Pipe {
                left: Box::new(expr),
                kind,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_logical_and()?;
        loop {
            let op = match self.peek() {
                Some(TokenKind::Or) => Some(BinaryOp::Or),
                Some(TokenKind::Ident(name)) if name == "또는" => Some(BinaryOp::Or),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                Some(TokenKind::And) => Some(BinaryOp::And),
                Some(TokenKind::Ident(name)) if name == "그리고" => Some(BinaryOp::And),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_range()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::EqEq => Some(BinaryOp::Eq),
                TokenKind::NotEq => Some(BinaryOp::NotEq),
                TokenKind::Lt => Some(BinaryOp::Lt),
                TokenKind::Lte => Some(BinaryOp::Lte),
                TokenKind::Gt => Some(BinaryOp::Gt),
                TokenKind::Gte => Some(BinaryOp::Gte),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_range()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_range(&mut self) -> Result<Expr, CanonError> {
        let left = self.parse_additive()?;
        let inclusive = match self.peek_kind() {
            TokenKind::DotDot => false,
            TokenKind::DotDotEq => true,
            _ => return Ok(left),
        };
        self.advance();
        let right = self.parse_additive()?;
        let flag = Expr::Literal(Literal::Num(if inclusive { "1" } else { "0" }.to_string()));
        Ok(Expr::Call {
            name: "표준.범위".to_string(),
            args: vec![left, right, flag],
        })
    }

    fn parse_additive(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => Some(BinaryOp::Add),
                TokenKind::Minus => Some(BinaryOp::Sub),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => Some(BinaryOp::Mul),
                TokenKind::Slash => Some(BinaryOp::Div),
                TokenKind::Percent => Some(BinaryOp::Mod),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, CanonError> {
        if self.peek_is(|k| matches!(k, TokenKind::Minus)) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Plus)) {
            self.advance();
            return self.parse_unary();
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.peek_is(|k| matches!(k, TokenKind::At)) {
                self.advance();
                let unit = self.expect_ident()?;
                expr = Expr::Unit {
                    value: Box::new(expr),
                    unit,
                };
                continue;
            }
            if self.peek_is(|k| matches!(k, TokenKind::LBracket)) {
                self.advance();
                let index_expr = self.parse_expr()?;
                self.expect(TokenKind::RBracket)?;
                expr = Expr::Call {
                    name: "차림.값".to_string(),
                    args: vec![expr, index_expr],
                };
                continue;
            }
            if let Some(field) = self.try_parse_dot_segment()? {
                expr = Expr::FieldAccess {
                    target: Box::new(expr),
                    field,
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, CanonError> {
        if let Some(TokenKind::PromptBlock(body)) = self.peek() {
            self.advance();
            return Ok(Expr::PromptBlock { body });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
            self.advance();
            if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expr::PromptExpr {
                    expr: Box::new(expr),
                });
            }
            return Err(CanonError::new(
                "E_CANON_EXPECTED_EXPR",
                "?? 뒤에는 (표현식) 형태가 필요합니다.",
            ));
        }
        if let Some(TokenKind::Atom(text)) = self.peek() {
            self.advance();
            return Ok(Expr::Literal(Literal::Atom(text.clone())));
        }
        if let Some(TokenKind::Template(body)) = self.peek() {
            self.advance();
            return Ok(Expr::Template { body: body.clone() });
        }
        if let Some(TokenKind::Formula(body)) = self.peek() {
            self.advance();
            return Ok(Expr::Formula {
                tag: None,
                body: body.clone(),
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_n_is(1, |k| matches!(k, TokenKind::Atom(_)))
            && self.peek_n_is(2, |k| matches!(k, TokenKind::RParen))
            && self.peek_n_is(3, |k| matches!(k, TokenKind::Formula(_)))
        {
            self.expect(TokenKind::LParen)?;
            let tag = if let Some(TokenKind::Atom(text)) = self.peek() {
                self.advance();
                text.clone()
            } else {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_ATOM",
                    "수식 태그가 필요합니다.",
                ));
            };
            self.expect(TokenKind::RParen)?;
            let body = if let Some(TokenKind::Formula(text)) = self.peek() {
                self.advance();
                text.clone()
            } else {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_FORMULA",
                    "수식 블록이 필요합니다.",
                ));
            };
            return Ok(Expr::Formula {
                tag: Some(tag),
                body,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::At)) {
            self.advance();
            let path = self.expect_string()?;
            return Ok(Expr::Resource(path));
        }
        if self.peek_is(|k| matches!(k, TokenKind::LBracket)) {
            self.advance();
            let mut args = Vec::new();
            if !self.peek_is(|k| matches!(k, TokenKind::RBracket)) {
                loop {
                    let expr = self.parse_expr()?;
                    args.push(expr);
                    if self.peek_is(|k| matches!(k, TokenKind::Comma)) {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }
            self.expect(TokenKind::RBracket)?;
            return self.build_charim_literal(args);
        }

        if self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            self.advance();
            let param = self.expect_ident()?;
            self.expect(TokenKind::Pipe)?;
            let body = self.parse_expr()?;
            self.expect(TokenKind::RBrace)?;
            return Ok(Expr::SeedLiteral {
                param,
                body: Box::new(body),
            });
        }

        if self.peek_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(_)))
            && self.peek_n_is(2, |k| matches!(k, TokenKind::Equals | TokenKind::Colon))
        {
            let bindings = self.parse_bindings()?;
            if let Some(text) = self.peek_ident_text() {
                if text == "인" {
                    self.advance();
                    let name = self.parse_call_name()?;
                    return Ok(Expr::CallIn { name, bindings });
                }
            }
            if let Some(TokenKind::Template(body)) = self.peek() {
                self.advance();
                return Ok(Expr::TemplateApply {
                    bindings,
                    body: body.clone(),
                });
            }
            if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name != "의")) {
                let name = self.parse_call_name()?;
                return Ok(Expr::Call {
                    name,
                    args: vec![Expr::Pack { bindings }],
                });
            }
            return Ok(Expr::Pack { bindings });
        }

        if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            self.advance();
            let mut args = Vec::new();
            if !self.peek_is(|k| matches!(k, TokenKind::RParen)) {
                loop {
                    let expr = self.parse_expr()?;
                    args.push(expr);
                    if self.peek_is(|k| matches!(k, TokenKind::Comma)) {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name != "의")) {
                let name = self.parse_call_name()?;
                return Ok(Expr::Call { name, args });
            }
            if args.len() == 1 {
                return Ok(args.into_iter().next().unwrap());
            }
            return Err(CanonError::new(
                "E_CANON_EXPECTED_EXPR",
                "호출 이름이 필요합니다.",
            ));
        }

        if self.bridge {
            if let Some(prefix) = self.peek_ident_text() {
                if is_bridge_alias_type_name(&prefix) {
                    self.advance();
                    let literal = self.parse_literal()?;
                    let ok = bridge_alias_matches_literal(&prefix, &literal);
                    if !ok {
                        return Err(CanonError::new(
                            "E_CANON_BAD_ALIAS",
                            "접두 타입이 맞지 않습니다.",
                        ));
                    }
                    return Ok(Expr::Literal(literal));
                }
                if self.peek_next_is_literal() {
                    return Err(CanonError::new(
                        "E_CANON_BAD_ALIAS",
                        format!("알 수 없는 별칭입니다: {}", prefix),
                    ));
                }
            }
        }

        match self.peek_kind() {
            TokenKind::String(text) => {
                self.advance();
                Ok(Expr::Literal(Literal::Str(text)))
            }
            TokenKind::Number(value) => {
                self.advance();
                Ok(Expr::Literal(Literal::Num(value)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            TokenKind::None => {
                self.advance();
                Ok(Expr::Literal(Literal::None))
            }
            TokenKind::Ident(_) => {
                let path = self.parse_path()?;
                Ok(Expr::Path(path))
            }
            _ => Err(CanonError::new(
                "E_CANON_EXPECTED_EXPR",
                "표현식이 필요합니다.",
            )),
        }
    }

    fn parse_bindings(&mut self) -> Result<Vec<Binding>, CanonError> {
        self.expect(TokenKind::LParen)?;
        let mut bindings = Vec::new();
        if !self.peek_is(|k| matches!(k, TokenKind::RParen)) {
            loop {
                let name = self.expect_ident()?;
                if !self.peek_is(|k| matches!(k, TokenKind::Equals | TokenKind::Colon)) {
                    return Err(CanonError::new(
                        "E_CANON_EXPECTED_EQUALS",
                        "'=' 또는 ':'가 필요합니다.",
                    ));
                }
                self.advance();
                let value = self.parse_expr()?;
                bindings.push(Binding { name, value });
                if self.peek_is(|k| matches!(k, TokenKind::Comma)) {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(bindings)
    }

    fn build_charim_literal(&self, args: Vec<Expr>) -> Result<Expr, CanonError> {
        let has_inner = args
            .iter()
            .any(|arg| matches!(arg, Expr::Call { name, .. } if name == "차림"));
        if !has_inner {
            return Ok(Expr::Call {
                name: "차림".to_string(),
                args,
            });
        }

        let mut rows: Vec<Vec<Expr>> = Vec::new();
        for arg in args {
            match arg {
                Expr::Call { name, args } if name == "차림" => {
                    if args
                        .iter()
                        .any(|item| matches!(item, Expr::Call { name, .. } if name == "차림"))
                    {
                        return Err(CanonError::new(
                            "E_CANON_TENSOR_SHAPE",
                            "중첩 차림은 같은 길이의 2차 차림이어야 합니다.",
                        ));
                    }
                    rows.push(args);
                }
                _ => {
                    return Err(CanonError::new(
                        "E_CANON_TENSOR_SHAPE",
                        "중첩 차림은 같은 길이의 2차 차림이어야 합니다.",
                    ))
                }
            }
        }

        let row_count = rows.len();
        let col_count = rows.first().map(|row| row.len()).unwrap_or(0);
        for row in &rows {
            if row.len() != col_count {
                return Err(CanonError::new(
                    "E_CANON_TENSOR_SHAPE",
                    "중첩 차림은 같은 길이의 2차 차림이어야 합니다.",
                ));
            }
        }

        let mut flat = Vec::new();
        for row in rows {
            flat.extend(row);
        }

        let shape = Expr::Call {
            name: "차림".to_string(),
            args: vec![
                Expr::Literal(Literal::Num(row_count.to_string())),
                Expr::Literal(Literal::Num(col_count.to_string())),
            ],
        };
        let data = Expr::Call {
            name: "차림".to_string(),
            args: flat,
        };
        let layout = Expr::Literal(Literal::Str("가로먼저".to_string()));
        let bindings = vec![
            Binding {
                name: "형상".to_string(),
                value: shape,
            },
            Binding {
                name: "자료".to_string(),
                value: data,
            },
            Binding {
                name: "배치".to_string(),
                value: layout,
            },
        ];
        Ok(Expr::Pack { bindings })
    }

    fn parse_literal(&mut self) -> Result<Literal, CanonError> {
        match self.peek_kind() {
            TokenKind::String(text) => {
                self.advance();
                Ok(Literal::Str(text))
            }
            TokenKind::Number(value) => {
                self.advance();
                Ok(Literal::Num(value))
            }
            TokenKind::True => {
                self.advance();
                Ok(Literal::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Literal::Bool(false))
            }
            TokenKind::None => {
                self.advance();
                Ok(Literal::None)
            }
            _ => Err(CanonError::new(
                "E_CANON_EXPECTED_LITERAL",
                "리터럴이 필요합니다.",
            )),
        }
    }

    fn parse_if_stmt(&mut self, condition: Condition) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Ilttae)?;
        let then_body = self.parse_block()?;
        self.skip_newlines();
        if self.peek_is(|k| matches!(k, TokenKind::Anigo)) {
            self.advance();
            self.skip_newlines();
            let else_condition = self.parse_condition_expr(false)?;
            self.expect(TokenKind::Ilttae)?;
            let stmt = self.parse_if_stmt(else_condition)?;
            return Ok(SurfaceStmt::If {
                condition,
                then_body,
                else_body: Some(vec![stmt]),
            });
        }
        let else_body = if self.peek_is(|k| matches!(k, TokenKind::Aniramyeon)) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        self.consume_optional_terminator();
        Ok(SurfaceStmt::If {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_contract_stmt(&mut self, condition: Condition) -> Result<SurfaceStmt, CanonError> {
        let kind = if self.peek_is(|k| matches!(k, TokenKind::Jeonjehae)) {
            self.advance();
            ContractKind::Pre
        } else {
            self.advance();
            ContractKind::Post
        };
        let mode = self.parse_contract_mode()?;
        self.skip_newlines();
        if !self.peek_is(|k| matches!(k, TokenKind::Aniramyeon)) {
            return Err(CanonError::new(
                "E_CANON_CONTRACT_NEEDS_ELSE",
                "계약에는 아니면 절이 필요합니다.",
            ));
        }
        self.advance();
        let else_body = self.parse_block()?;
        self.skip_newlines();
        let then_body = if self.peek_is(|k| matches!(k, TokenKind::Majeumyeon)) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Contract {
            kind,
            mode,
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_choose_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Goreugi)?;
        self.expect(TokenKind::Colon)?;
        let mut branches = Vec::new();
        let mut else_body: Option<Vec<SurfaceStmt>> = None;

        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::Aniramyeon)) {
                self.advance();
                self.expect(TokenKind::Colon)?;
                else_body = Some(self.parse_block()?);
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                break;
            }
            let condition = self.parse_condition_expr(true)?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            branches.push(SurfaceChooseBranch { condition, body });
        }

        let Some(else_body) = else_body else {
            return Err(CanonError::new(
                "E_CANON_CHOOSE_NEEDS_ELSE",
                "고르기에는 아니면 절이 필요합니다.",
            ));
        };
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Choose {
            branches,
            else_body,
        })
    }

    fn parse_prompt_choose_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Prompt)?;
        self.expect(TokenKind::Colon)?;
        let mut branches = Vec::new();
        let mut else_body: Option<Vec<SurfaceStmt>> = None;

        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
                self.advance();
                if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
                    self.advance();
                    else_body = Some(self.parse_block()?);
                    break;
                }
                return Err(CanonError::new(
                    "E_CANON_PROMPT_ELSE",
                    "??: 뒤에는 :가 필요합니다.",
                ));
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof | TokenKind::RBrace)) {
                break;
            }
            let condition = self.parse_condition_expr(true)?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            branches.push(SurfaceChooseBranch { condition, body });
        }

        self.consume_optional_terminator();
        Ok(SurfaceStmt::PromptChoose {
            branches,
            else_body,
        })
    }

    fn parse_prompt_after_stmt(&mut self, value: Expr) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Prompt)?;
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
        }
        self.skip_newlines();
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::PromptAfter { value, body })
    }

    fn parse_prompt_condition_stmt(
        &mut self,
        condition: Condition,
    ) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Prompt)?;
        self.skip_newlines();
        if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
            self.advance();
            if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
                self.advance();
            }
            self.skip_newlines();
        } else if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::PromptCondition { condition, body })
    }

    fn parse_prompt_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Prompt)?;
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
        }
        self.skip_newlines();
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::PromptBlock { body })
    }

    fn parse_contract_mode(&mut self) -> Result<ContractMode, CanonError> {
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            return Ok(ContractMode::Abort);
        }
        self.expect(TokenKind::LParen)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::RParen)?;
        match name.as_str() {
            "알림" => Ok(ContractMode::Alert),
            "물림" => Ok(ContractMode::Abort),
            _ => Err(CanonError::new(
                "E_CANON_CONTRACT_MODE",
                "계약 모드는 알림 또는 물림이어야 합니다.",
            )),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<SurfaceStmt>, CanonError> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_RBRACE",
                    "닫는 중괄호가 필요합니다.",
                ));
            }
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn parse_condition_expr(&mut self, require_thunk: bool) -> Result<Condition, CanonError> {
        if !self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            if require_thunk {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_THUNK_CONDITION",
                    "조건은 { ... }인것 형태여야 합니다.",
                ));
            }
            let expr = self.parse_expr()?;
            return Ok(Condition {
                expr,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        self.expect(TokenKind::LBrace)?;
        self.skip_newlines();
        let expr = self.parse_expr()?;
        if self.peek_is(|k| matches!(k, TokenKind::Dot)) {
            self.advance();
        }
        self.skip_newlines();
        self.expect(TokenKind::RBrace)?;
        self.skip_newlines();
        let name = self.expect_ident()?;
        let negated = match name.as_str() {
            "인것" => false,
            "아닌것" => true,
            _ => {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_THUNK_SUFFIX",
                    "조건은 { ... }인것/아닌것 형태여야 합니다.",
                ))
            }
        };
        Ok(Condition {
            expr,
            style: ConditionStyle::Thunk,
            negated,
        })
    }

    fn parse_path(&mut self) -> Result<Path, CanonError> {
        let first = self.expect_ident()?;
        let mut segments = vec![first];
        loop {
            let Some(segment) = self.try_parse_dot_segment()? else {
                break;
            };
            segments.push(segment);
        }
        Ok(Path { segments })
    }

    fn try_parse_dot_segment(&mut self) -> Result<Option<String>, CanonError> {
        if !self.peek_is(|k| matches!(k, TokenKind::Dot)) {
            return Ok(None);
        }
        if self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(_))) {
            self.advance();
            return Ok(Some(self.expect_ident()?));
        }
        let Some(next) = self
            .tokens
            .get(self.pos + 1)
            .map(|token| token.kind.clone())
        else {
            return Ok(None);
        };
        let TokenKind::Number(raw) = next else {
            return Ok(None);
        };
        if !raw.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(CanonError::new(
                "E_CANON_TUPLE_INDEX_INVALID",
                "점 접근 인덱스는 정수(.0/.1 등)만 허용됩니다.",
            ));
        }
        self.advance();
        self.advance();
        Ok(Some(raw))
    }

    // Step B/C scope: allow one-level + nested map write.
    // - allowed: 살림.공.x, 바탕.공.x, 공.x, 공.속도.x, 살림.공.속도.x
    fn split_map_dot_target(path: &Path) -> Option<(Path, Vec<String>)> {
        if path.segments.len() < 2 {
            return None;
        }
        let has_root = matches!(
            path.segments.first().map(|seg| seg.as_str()),
            Some("살림" | "바탕" | "샘")
        );
        if has_root {
            if path.segments.len() < 3 {
                return None;
            }
            let target = Path {
                segments: vec![path.segments[0].clone(), path.segments[1].clone()],
            };
            return Some((target, path.segments[2..].to_vec()));
        }
        let target = Path {
            segments: vec![path.segments[0].clone()],
        };
        Some((target, path.segments[1..].to_vec()))
    }

    fn build_map_dot_write_expr(map_expr: Expr, key_segments: &[String], value: Expr) -> Expr {
        debug_assert!(!key_segments.is_empty());
        let key_expr = Expr::Literal(Literal::Str(key_segments[0].clone()));
        if key_segments.len() == 1 {
            return Expr::Call {
                name: "짝맞춤.바꾼값".to_string(),
                args: vec![map_expr, key_expr, value],
            };
        }
        let child_map_expr = Expr::Call {
            name: "짝맞춤.필수값".to_string(),
            args: vec![map_expr.clone(), key_expr.clone()],
        };
        let child_value = Self::build_map_dot_write_expr(child_map_expr, &key_segments[1..], value);
        Expr::Call {
            name: "짝맞춤.바꾼값".to_string(),
            args: vec![map_expr, key_expr, child_value],
        }
    }

    fn parse_call_name(&mut self) -> Result<String, CanonError> {
        let mut name = self.expect_ident()?;
        loop {
            if self.peek_is(|k| matches!(k, TokenKind::Dot))
                && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(_)))
            {
                self.advance();
                let ident = self.expect_ident()?;
                name.push('.');
                name.push_str(&ident);
                continue;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Ident(_))) {
                if self.peek_is(
                    |k| matches!(k, TokenKind::Ident(name) if name == "해서" || name == "하고"),
                ) {
                    break;
                }
                let ident = self.expect_ident()?;
                name.push(' ');
                name.push_str(&ident);
                continue;
            }
            break;
        }
        Ok(name)
    }

    fn consume_terminator(&mut self) -> Result<(), CanonError> {
        if self.peek_is(|k| matches!(k, TokenKind::Dot | TokenKind::Question | TokenKind::Bang)) {
            self.advance();
            self.skip_newlines();
            return Ok(());
        }
        if self.peek_is(|k| matches!(k, TokenKind::Newline)) {
            self.skip_newlines();
            return Ok(());
        }
        if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
            return Ok(());
        }
        Err(CanonError::new(
            "E_CANON_EXPECTED_TERMINATOR",
            "문장 끝에 마침표가 필요합니다.",
        ))
    }

    fn consume_optional_terminator(&mut self) {
        if self.peek_is(|k| matches!(k, TokenKind::Dot | TokenKind::Question | TokenKind::Bang)) {
            self.advance();
        }
        self.skip_newlines();
    }

    fn skip_separators(&mut self) {
        loop {
            if self.peek_is(|k| {
                matches!(
                    k,
                    TokenKind::Newline | TokenKind::Dot | TokenKind::Question | TokenKind::Bang
                )
            }) {
                self.advance();
                continue;
            }
            break;
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek_is(|k| matches!(k, TokenKind::Newline)) {
            self.advance();
        }
    }

    fn is_bogae_draw_stmt(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Ident(ref name) if name == "보개로")
            && self.peek_n_is(
                1,
                |k| matches!(k, TokenKind::Ident(ref name) if name == "그려"),
            )
    }

    fn parse_bogae_madang_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let token = self.advance();
        let body = match token.kind {
            TokenKind::BogaeMadangBlock(body) => body,
            TokenKind::BogaeJangmyeonBlock(body) => body,
            _ => {
                return Err(self.unexpected_token_error("보개마당/보개장면 블록 파싱 실패"))
            }
        };
        self.consume_optional_terminator();
        Ok(SurfaceStmt::BogaeMadangBlock { body })
    }

    fn parse_jjaim_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let token = self.advance();
        let body = match token.kind {
            TokenKind::JjaimBlock(body) => body,
            TokenKind::GuseongBlock(body) => body,
            _ => {
                return Err(self.unexpected_token_error("짜임/구성 블록 파싱 실패"))
            }
        };
        validate_jjaim_block_body(&body)?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::JjaimBlock { body })
    }

    fn parse_exec_policy_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let token = self.advance();
        let body = match token.kind {
            TokenKind::ExecPolicyBlock(body) => body,
            _ => {
                return Err(self.unexpected_token_error("너머 블록 파싱 실패"))
            }
        };
        self.consume_optional_terminator();
        Ok(SurfaceStmt::ExecPolicyBlock { body })
    }

    fn expect_ident(&mut self) -> Result<String, CanonError> {
        match self.peek_kind() {
            TokenKind::Ident(text) => {
                self.advance();
                Ok(text)
            }
            _ => Err(CanonError::new(
                "E_CANON_EXPECTED_IDENT",
                "이름이 필요합니다.",
            )),
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), CanonError> {
        let current = self.peek_kind();
        if std::mem::discriminant(&current) == std::mem::discriminant(&expected) {
            self.advance();
            return Ok(());
        }
        Err(self.unexpected_token_error("토큰 기대값 불일치"))
    }

    fn peek_ident_text(&self) -> Option<String> {
        match self.peek_kind() {
            TokenKind::Ident(text) => Some(text),
            _ => None,
        }
    }

    fn peek_next_is_literal(&self) -> bool {
        self.peek_n_is(1, |k| {
            matches!(
                k,
                TokenKind::String(_)
                    | TokenKind::Number(_)
                    | TokenKind::True
                    | TokenKind::False
                    | TokenKind::None
            )
        })
    }

    fn peek_is_ident_colon(&self) -> bool {
        self.peek_is(|k| matches!(k, TokenKind::Ident(_)))
            && self.peek_n_is(1, |k| matches!(k, TokenKind::Colon))
    }

    fn peek_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|token| token.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn peek(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|token| token.kind.clone())
    }

    fn peek_is(&self, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        f(&self.peek_kind())
    }

    fn peek_n_is(&self, n: usize, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        self.tokens
            .get(self.pos + n)
            .map(|token| f(&token.kind))
            .unwrap_or(false)
    }

    fn peek_next_non_newline_is(&self, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        let mut idx = self.pos + 1;
        while let Some(token) = self.tokens.get(idx) {
            if !matches!(token.kind, TokenKind::Newline) {
                return f(&token.kind);
            }
            idx += 1;
        }
        false
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }
}

fn is_allowed_jjaim_header(raw: &str) -> bool {
    let mut header = raw.trim();
    if let Some(stripped) = header.strip_suffix(':') {
        header = stripped.trim_end();
    }
    if header.is_empty() {
        return true;
    }
    let fixed = [
        "입력",
        "상태",
        "출력",
        "수식",
        "모양",
        "보개",
        "보개마당",
        "채비",
        "씨앗",
        "고르기",
        "반복",
        "되풀이",
    ];
    if fixed
        .iter()
        .any(|k| header == *k || header.starts_with(&format!("{} ", k)))
    {
        return true;
    }
    header.contains("마다")
        || header.contains("할때")
        || header.contains("일때")
        || header.contains("오면")
        || header.contains("대해")
        || header.contains("동안")
}

fn validate_jjaim_block_body(body: &str) -> Result<(), CanonError> {
    let mut depth: i32 = 0;
    for raw_line in body.lines() {
        let line = raw_line.trim();
        if depth == 0 && !line.is_empty() && !line.starts_with("//") {
            if let Some(open_idx) = line.find('{') {
                let head = line[..open_idx].trim();
                if !head.is_empty() && !is_allowed_jjaim_header(head) {
                    return Err(CanonError::new(
                        "E_JJAIM_SUBBLOCK_INVALID",
                        format!(
                            "짜임 상위 항목은 입력/상태/출력/수식/모양/매마디·조건·이벤트 계열만 허용됩니다: {}",
                            head
                        ),
                    ));
                }
            }
        }
        for ch in raw_line.chars() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
            }
        }
    }
    Ok(())
}

pub struct CanonOutput {
    pub ddn: String,
    pub guseong_flat_json: String,
    pub alrim_plan_json: String,
    #[allow(dead_code)]
    pub block_editor_plan_json: String,
    pub exec_policy_map_json: String,
    pub maegim_control_json: String,
    pub meta: FileMeta,
    pub warnings: Vec<String>,
}

struct FrontdoorFallbackPlans {
    guseong_flat_json: String,
    alrim_plan_json: String,
    exec_policy_map_json: String,
    maegim_control_json: String,
}

pub fn preprocess_frontdoor_source(input: &str) -> String {
    ddonirang_lang::preprocess_frontdoor_source(input)
}

pub fn has_exec_policy_surface(input: &str) -> bool {
    let prepared = preprocess_frontdoor_source(input);
    // 빠른 음성 판정: 실행정책/너머 키워드 자체가 없으면 exec-policy 표면이 없다.
    if !prepared.contains("너머") && !prepared.contains("실행정책") {
        return false;
    }
    let Ok(tokens) = Lexer::tokenize(&prepared) else {
        // 토큰화가 실패해도 키워드 기반으로만 보수 판정한다.
        return prepared.contains("너머") || prepared.contains("실행정책");
    };

    let mut i = 0usize;
    while i < tokens.len() {
        match &tokens[i].kind {
            TokenKind::ExecPolicyBlock(_) => return true,
            TokenKind::Ident(name) if name == "너머" => {
                let mut j = i + 1;
                while j < tokens.len() && matches!(tokens[j].kind, TokenKind::Newline) {
                    j += 1;
                }
                if j < tokens.len() && matches!(tokens[j].kind, TokenKind::LBrace) {
                    return true;
                }
                if j < tokens.len() && matches!(tokens[j].kind, TokenKind::Colon) {
                    j += 1;
                    while j < tokens.len() && matches!(tokens[j].kind, TokenKind::Newline) {
                        j += 1;
                    }
                    if j < tokens.len() && matches!(tokens[j].kind, TokenKind::LBrace) {
                        return true;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

#[allow(dead_code)]
pub fn has_legacy_boim_surface(input: &str) -> bool {
    ddonirang_lang::has_legacy_boim_surface(input)
}

pub fn canonicalize(input: &str, bridge: bool) -> Result<CanonOutput, CanonError> {
    canonicalize_with_fallback_mode(input, bridge, true)
}

fn canonicalize_with_fallback_mode(
    input: &str,
    bridge: bool,
    allow_frontdoor_fallback: bool,
) -> Result<CanonOutput, CanonError> {
    let prepared = preprocess_frontdoor_source(input);
    if let Some((line, key)) = ddonirang_lang::find_legacy_header(&prepared) {
        return Err(CanonError::new(
            "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN",
            format!("line={line} key={key} use=설정{{}}/매김{{}}/설정.보개"),
        ));
    }
    if ddonirang_lang::has_legacy_boim_surface(&prepared) {
        return Err(CanonError::new(
            "E_CANON_LEGACY_BOIM_FORBIDDEN",
            "legacy `보임 {}` 표면은 금지되었습니다. `설정.보개`/정본 보개 표면으로 전환하세요.",
        ));
    }
    let meta_parse = split_file_meta(&prepared);
    let root_hide = has_root_hide_directive(&prepared);
    let normalized_frontdoor = ddonirang_lang::parse_frontdoor_and_normalize(
        &meta_parse.stripped,
        "<canon-frontdoor-prepared>",
        ddonirang_lang::NormalizationLevel::N1,
    )
    .ok();
    let parse_source = normalized_frontdoor
        .as_deref()
        .unwrap_or(&meta_parse.stripped);
    let default_root = "바탕";
    let tokens = Lexer::tokenize(parse_source)?;
    let legacy_guseong_alias_seen = tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::GuseongBlock(_)));
    let legacy_bogae_madang_alias_seen = tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::BogaeJangmyeonBlock(_)));
    let mut parser = Parser::new(tokens, bridge);
    let surface = match parser.parse_program() {
        Ok(surface) => surface,
        Err(primary_err) => {
            if !allow_frontdoor_fallback {
                return Err(primary_err);
            }
            let normalized = normalized_frontdoor.clone().or_else(|| {
                ddonirang_lang::parse_frontdoor_and_normalize(
                    &meta_parse.stripped,
                    "<canon-fallback-frontdoor>",
                    ddonirang_lang::NormalizationLevel::N1,
                )
                .ok()
            });

            if let Some(normalized_text) = normalized.as_ref() {
                if let Ok(mut strict_output) =
                    canonicalize_with_fallback_mode(normalized_text, bridge, false)
                {
                    let mut ddn = String::new();
                    ddn.push_str(&format_file_meta(&meta_parse.meta));
                    ddn.push_str(strict_output.ddn.trim_end());
                    ddn.push('\n');
                    strict_output.ddn = ddn;
                    strict_output.meta = meta_parse.meta.clone();
                    return Ok(strict_output);
                }
            }

            let fallback_source = normalized.as_deref().unwrap_or(&meta_parse.stripped);
            let fallback_plans = build_frontdoor_fallback_plans(fallback_source)?;
            let ddn_body = normalized.unwrap_or_else(|| preprocess_frontdoor_source(&meta_parse.stripped));
            let mut ddn = String::new();
            ddn.push_str(&format_file_meta(&meta_parse.meta));
            ddn.push_str(ddn_body.trim_end());
            ddn.push('\n');
            return Ok(CanonOutput {
                ddn,
                guseong_flat_json: fallback_plans.guseong_flat_json,
                alrim_plan_json: fallback_plans.alrim_plan_json,
                block_editor_plan_json: "{}\n".to_string(),
                exec_policy_map_json: fallback_plans.exec_policy_map_json,
                maegim_control_json: fallback_plans.maegim_control_json,
                meta: meta_parse.meta,
                warnings: Vec::new(),
            });
        }
    };
    let deprecated_block_header_colon_count = parser.deprecated_block_header_colon_count();
    let flatten_plan = build_guseong_flatten_plan_surface(&surface)?;
    let guseong_flat_json = flatten_plan.to_json_string()?;

    let mut declared = HashSet::new();
    for stmt in &surface {
        match stmt {
            SurfaceStmt::Decl { name, .. } => {
                declared.insert(name.clone());
            }
            SurfaceStmt::RootDecl { items, .. } => {
                for item in items {
                    if item.kind == DeclKind::Gureut {
                        declared.insert(item.name.clone());
                    }
                }
            }
            _ => {}
        }
    }

    let mut warnings = Vec::new();
    let mut canonical = Vec::new();
    for stmt in surface {
        match stmt {
            SurfaceStmt::RootDecl { items } => {
                let items = items
                    .into_iter()
                    .map(|item| DeclItem {
                        name: item.name,
                        kind: item.kind,
                        type_name: item.type_name,
                        value: item.value.map(|expr| {
                            canonicalize_expr(expr, &declared, bridge, default_root, root_hide)
                        }),
                        maegim: item.maegim.map(|spec| MaegimSpec {
                            fields: spec
                                .fields
                                .into_iter()
                                .map(|binding| Binding {
                                    name: binding.name,
                                    value: canonicalize_expr(
                                        binding.value,
                                        &declared,
                                        bridge,
                                        default_root,
                                        root_hide,
                                    ),
                                })
                                .collect(),
                        }),
                    })
                    .collect();
                canonical.push(Stmt::RootDecl { items });
            }
            SurfaceStmt::Decl {
                name,
                type_name,
                value,
            } => {
                if !is_bridge_alias_type_name(&type_name) {
                    return Err(CanonError::new(
                        "E_CANON_BAD_ALIAS",
                        format!("알 수 없는 별칭입니다: {}", type_name),
                    ));
                }
                let target = Path {
                    segments: vec![default_root.to_string(), name],
                };
                let value = canonicalize_expr(value, &declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Assign { target, value });
            }
            SurfaceStmt::Assign { target, value } => {
                if root_hide
                    && !matches!(
                        target.segments.first().map(|seg| seg.as_str()),
                        Some("살림" | "바탕" | "샘")
                    )
                {
                    if let Some(name) = target.segments.first() {
                        if !declared.contains(name) {
                            return Err(CanonError::new(
                                "E_CANON_ROOT_HIDE_UNDECLARED",
                                format!("바탕숨김에서 미등록 바탕칸 쓰기: {}", name),
                            ));
                        }
                    }
                }
                let target = canonicalize_path(target, &declared, bridge, default_root, root_hide);
                let value = canonicalize_expr(value, &declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Assign { target, value });
            }
            SurfaceStmt::SeedDef {
                name,
                kind,
                params,
                body,
            } => {
                let params = params
                    .into_iter()
                    .map(|param| Param {
                        name: param.name,
                        type_name: param.type_name,
                        default: param.default.map(|expr| {
                            canonicalize_expr(expr, &declared, bridge, default_root, root_hide)
                        }),
                    })
                    .collect();
                let body = canonicalize_body(body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::SeedDef {
                    name,
                    kind,
                    params,
                    body,
                });
            }
            SurfaceStmt::Send {
                sender,
                payload,
                receiver,
            } => {
                canonical.push(Stmt::Send {
                    sender: sender.map(|expr| {
                        canonicalize_expr(expr, &declared, bridge, default_root, root_hide)
                    }),
                    payload: canonicalize_expr(payload, &declared, bridge, default_root, root_hide),
                    receiver: canonicalize_expr(
                        receiver,
                        &declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                });
            }
            SurfaceStmt::Show { value } => {
                let value = canonicalize_expr(value, &declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Show { value });
            }
            SurfaceStmt::Inspect { value } => {
                let value = canonicalize_expr(value, &declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Inspect { value });
            }
            SurfaceStmt::BogaeMadangBlock { body, .. } => {
                canonical.push(Stmt::BogaeMadangBlock { body });
            }
            SurfaceStmt::JjaimBlock { body, .. } => {
                canonical.push(Stmt::JjaimBlock { body });
            }
            SurfaceStmt::BogaeDraw => {
                canonical.push(Stmt::BogaeDraw);
            }
            SurfaceStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition =
                    canonicalize_condition(condition, &declared, bridge, default_root, root_hide);
                let then_body =
                    canonicalize_body(then_body, &declared, bridge, default_root, root_hide)?;
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        &declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                canonical.push(Stmt::If {
                    condition,
                    then_body,
                    else_body,
                });
            }
            SurfaceStmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
            } => {
                let condition =
                    canonicalize_condition(condition, &declared, bridge, default_root, root_hide);
                let else_body =
                    canonicalize_body(else_body, &declared, bridge, default_root, root_hide)?;
                let then_body = match then_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        &declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                canonical.push(Stmt::Contract {
                    kind,
                    mode,
                    condition,
                    then_body,
                    else_body,
                });
            }
            SurfaceStmt::Repeat { body } => {
                let body = canonicalize_body(body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::Repeat { body });
            }
            SurfaceStmt::While { condition, body } => {
                let condition =
                    canonicalize_condition(condition, &declared, bridge, default_root, root_hide);
                let body = canonicalize_body(body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::While { condition, body });
            }
            SurfaceStmt::ForEach {
                item,
                iterable,
                body,
            } => {
                let iterable =
                    canonicalize_expr(iterable, &declared, bridge, default_root, root_hide);
                let body = canonicalize_body(body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::ForEach {
                    item,
                    iterable,
                    body,
                });
            }
            SurfaceStmt::Hook { name, suffix, body } => {
                let body = canonicalize_body(body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::Hook { name, suffix, body });
            }
            SurfaceStmt::Receive {
                kind,
                binding,
                condition,
                body,
            } => {
                canonical.push(Stmt::Receive {
                    kind,
                    binding,
                    condition: condition.map(|expr| {
                        canonicalize_expr(expr, &declared, bridge, default_root, root_hide)
                    }),
                    body: canonicalize_body(body, &declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::EventReact { kind, body } => {
                canonical.push(Stmt::EventReact {
                    kind,
                    body: canonicalize_body(body, &declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::OpenBlock { body } => {
                let body = canonicalize_body(body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::OpenBlock { body });
            }
            SurfaceStmt::Return { value } => {
                let value = canonicalize_expr(value, &declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Return { value });
            }
            SurfaceStmt::Expr { value } => {
                let value = canonicalize_expr(value, &declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Expr { value });
            }
            SurfaceStmt::Break => {
                canonical.push(Stmt::Break);
            }
            SurfaceStmt::PromptChoose {
                branches,
                else_body,
            } => {
                let mut mapped = Vec::new();
                for branch in branches {
                    mapped.push(ChooseBranch {
                        condition: canonicalize_condition(
                            branch.condition,
                            &declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                        body: canonicalize_body(
                            branch.body,
                            &declared,
                            bridge,
                            default_root,
                            root_hide,
                        )?,
                    });
                }
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        &declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                canonical.push(Stmt::PromptChoose {
                    branches: mapped,
                    else_body,
                });
            }
            SurfaceStmt::PromptAfter { value, body } => {
                canonical.push(Stmt::PromptAfter {
                    value: canonicalize_expr(value, &declared, bridge, default_root, root_hide),
                    body: canonicalize_body(body, &declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptCondition { condition, body } => {
                canonical.push(Stmt::PromptCondition {
                    condition: canonicalize_condition(
                        condition,
                        &declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    body: canonicalize_body(body, &declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptBlock { body } => {
                canonical.push(Stmt::PromptBlock {
                    body: canonicalize_body(body, &declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::ExecPolicyBlock { body } => {
                canonical.push(Stmt::ExecPolicyBlock { body });
            }
            SurfaceStmt::Choose {
                branches,
                else_body,
            } => {
                let mut mapped = Vec::new();
                for branch in branches {
                    mapped.push(ChooseBranch {
                        condition: canonicalize_condition(
                            branch.condition,
                            &declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                        body: canonicalize_body(
                            branch.body,
                            &declared,
                            bridge,
                            default_root,
                            root_hide,
                        )?,
                    });
                }
                let else_body =
                    canonicalize_body(else_body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::Choose {
                    branches: mapped,
                    else_body,
                });
            }
        }
    }

    let ddn = format_program(&canonical);
    let alrim_plan_json = build_alrim_event_plan(&canonical).to_json_string()?;
    let block_editor_plan_json = build_block_editor_plan(&canonical).to_json_string()?;
    let exec_policy_map_json = build_exec_policy_effect_map(&canonical).to_json_string()?;
    let maegim_control_json = build_maegim_control_plan(&canonical).to_json_string()?;
    if !meta_parse.dup_keys.is_empty() {
        warnings.push(format!("META_DUP_KEY {}", meta_parse.dup_keys.join(", ")));
    }
    if legacy_guseong_alias_seen {
        warnings.push(
            "W_JJAIM_ALIAS_DEPRECATED `구성`은 별칭입니다. 정본 키워드 `짜임`을 사용하세요"
                .to_string(),
        );
    }
    if legacy_bogae_madang_alias_seen {
        warnings.push(
            "W_BOGAE_MADANG_ALIAS_DEPRECATED `보개장면`은 별칭입니다. 정본 키워드 `보개마당`을 사용하세요"
                .to_string(),
        );
    }
    if deprecated_block_header_colon_count > 0 {
        warnings.push(format!(
            "W_BLOCK_HEADER_COLON_DEPRECATED 블록 헤더의 `키워드:` 표기는 예정된 비권장입니다. `키워드 {{` 표기로 전환하세요 (count={})",
            deprecated_block_header_colon_count
        ));
    }
    let mut output = String::new();
    output.push_str(&format_file_meta(&meta_parse.meta));
    output.push_str(&ddn);
    Ok(CanonOutput {
        ddn: output,
        guseong_flat_json,
        alrim_plan_json,
        block_editor_plan_json,
        exec_policy_map_json,
        maegim_control_json,
        meta: meta_parse.meta,
        warnings,
    })
}

#[derive(Serialize)]
struct FallbackGuseongEnvelope {
    schema: &'static str,
    groups: Vec<FallbackGuseongGroup>,
}

#[derive(Serialize)]
struct FallbackGuseongGroup {
    id: String,
}

#[derive(Serialize)]
struct FallbackAlrimEnvelope {
    schema: &'static str,
    plans: Vec<String>,
}

#[derive(Serialize)]
struct FallbackExecEnvelope {
    schema: &'static str,
    map: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct FallbackMaegimEnvelope {
    schema: &'static str,
    controls: Vec<FallbackMaegimControl>,
}

#[derive(Serialize)]
struct FallbackMaegimControl {
    name: String,
    decl_kind: &'static str,
    min_expr_canon: Option<String>,
    max_expr_canon: Option<String>,
    inclusive_end: bool,
    step_expr_canon: Option<String>,
    split_count_expr_canon: Option<String>,
}

fn build_frontdoor_fallback_plans(source: &str) -> Result<FrontdoorFallbackPlans, CanonError> {
    let guseong_flat_json = build_frontdoor_fallback_guseong_json(source)?;
    let alrim_plan_json = build_frontdoor_fallback_alrim_json()?;
    let exec_policy_map_json = build_frontdoor_fallback_exec_json()?;
    let maegim_control_json = build_frontdoor_fallback_maegim_json(source)?;
    Ok(FrontdoorFallbackPlans {
        guseong_flat_json,
        alrim_plan_json,
        exec_policy_map_json,
        maegim_control_json,
    })
}

fn build_frontdoor_fallback_guseong_json(source: &str) -> Result<String, CanonError> {
    let mut uniq = BTreeSet::new();
    let mut groups = Vec::new();
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if !line.starts_with("무리 <- ") {
            continue;
        }
        let Some(start) = line.find('"') else {
            continue;
        };
        let remain = &line[start + 1..];
        let Some(end) = remain.find('"') else {
            continue;
        };
        let id = remain[..end].trim();
        if id.is_empty() || !uniq.insert(id.to_string()) {
            continue;
        }
        groups.push(FallbackGuseongGroup { id: id.to_string() });
    }
    let envelope = FallbackGuseongEnvelope {
        schema: "ddn.guseong_flatten_plan.v1",
        groups,
    };
    serde_json::to_string_pretty(&envelope)
        .map(|text| format!("{}\n", text))
        .map_err(|err| {
            CanonError::new(
                "E_CANON_GUSEONG_FLAT_JSON",
                format!("frontdoor fallback guseong JSON 직렬화 실패: {}", err),
            )
        })
}

fn build_frontdoor_fallback_alrim_json() -> Result<String, CanonError> {
    let envelope = FallbackAlrimEnvelope {
        schema: "ddn.alrim_plan.v1",
        plans: Vec::new(),
    };
    serde_json::to_string_pretty(&envelope)
        .map(|text| format!("{}\n", text))
        .map_err(|err| {
            CanonError::new(
                "E_CANON_ALRIM_PLAN_JSON",
                format!("frontdoor fallback alrim JSON 직렬화 실패: {}", err),
            )
        })
}

fn build_frontdoor_fallback_exec_json() -> Result<String, CanonError> {
    let envelope = FallbackExecEnvelope {
        schema: "ddn.exec_policy_effect_map.v1",
        map: BTreeMap::new(),
    };
    serde_json::to_string_pretty(&envelope)
        .map(|text| format!("{}\n", text))
        .map_err(|err| {
            CanonError::new(
                "E_CANON_EXEC_POLICY_MAP_JSON",
                format!("frontdoor fallback exec-policy JSON 직렬화 실패: {}", err),
            )
        })
}

fn build_frontdoor_fallback_maegim_json(source: &str) -> Result<String, CanonError> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut idx = 0usize;
    let mut controls = Vec::new();
    while idx < lines.len() {
        let line = lines[idx].trim();
        if !line.contains("매김 {") {
            idx += 1;
            continue;
        }

        let Some(colon_pos) = line.find(':') else {
            idx += 1;
            continue;
        };
        let name = line[..colon_pos].trim();
        if name.is_empty() {
            idx += 1;
            continue;
        }

        let mut min_expr_canon = None;
        let mut max_expr_canon = None;
        let mut inclusive_end = false;
        let mut step_expr_canon = None;
        let mut split_count_expr_canon = None;

        idx += 1;
        while idx < lines.len() {
            let inner = lines[idx].trim();
            if inner.starts_with("}.") || inner == "}" {
                break;
            }
            if let Some(rest) = inner.strip_prefix("범위:") {
                let expr = rest.trim().trim_end_matches('.');
                if let Some((left, right, incl)) = parse_fallback_range(expr) {
                    min_expr_canon = Some(left);
                    max_expr_canon = Some(right);
                    inclusive_end = incl;
                }
            } else if let Some(rest) = inner.strip_prefix("간격:") {
                step_expr_canon = Some(rest.trim().trim_end_matches('.').to_string());
            } else if let Some(rest) = inner.strip_prefix("분할수:") {
                split_count_expr_canon = Some(rest.trim().trim_end_matches('.').to_string());
            }
            idx += 1;
        }

        controls.push(FallbackMaegimControl {
            name: name.to_string(),
            decl_kind: "gureut",
            min_expr_canon,
            max_expr_canon,
            inclusive_end,
            step_expr_canon,
            split_count_expr_canon,
        });
        idx += 1;
    }

    let envelope = FallbackMaegimEnvelope {
        schema: "ddn.maegim_control_plan.v1",
        controls,
    };
    serde_json::to_string_pretty(&envelope)
        .map(|text| format!("{}\n", text))
        .map_err(|err| {
            CanonError::new(
                "E_CANON_MAEGIM_CONTROL_PLAN_JSON",
                format!("frontdoor fallback maegim JSON 직렬화 실패: {}", err),
            )
        })
}

fn parse_fallback_range(input: &str) -> Option<(String, String, bool)> {
    if let Some((left, right)) = input.split_once("..=") {
        return Some((left.trim().to_string(), right.trim().to_string(), true));
    }
    if let Some((left, right)) = input.split_once("..") {
        return Some((left.trim().to_string(), right.trim().to_string(), false));
    }
    None
}

#[derive(Debug, Default, Serialize)]
struct GuseongFlattenPlan {
    topo_order: Vec<String>,
    instances: Vec<GuseongFlattenInstance>,
    links: Vec<GuseongFlattenLink>,
}

#[derive(Debug, Clone, Serialize)]
struct GuseongFlattenInstance {
    name: String,
    type_name: String,
}

#[derive(Debug, Clone, Serialize)]
struct GuseongFlattenLink {
    dst_instance: String,
    dst_port: String,
    src_instance: String,
    src_port: String,
}

#[derive(Debug, Serialize)]
struct GuseongFlattenPlanEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    plan: &'a GuseongFlattenPlan,
}

impl GuseongFlattenPlan {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = GuseongFlattenPlanEnvelope {
            schema: "ddn.guseong_flatten_plan.v1",
            plan: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_GUSEONG_FLATTEN_JSON",
                    format!("짜임 flatten JSON 직렬화 실패: {}", err),
                )
            })
    }
}

#[derive(Debug, Default, Serialize)]
struct AlrimEventPlan {
    handlers: Vec<AlrimEventHandler>,
}

#[derive(Debug, Clone, Serialize)]
struct AlrimEventHandler {
    order: u64,
    kind: String,
    scope: String,
    body_canon: String,
}

#[derive(Debug, Serialize)]
struct AlrimEventPlanEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    plan: &'a AlrimEventPlan,
}

impl AlrimEventPlan {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = AlrimEventPlanEnvelope {
            schema: "ddn.alrim_event_plan.v1",
            plan: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_ALRIM_EVENT_JSON",
                    format!("알림 이벤트 계획 JSON 직렬화 실패: {}", err),
                )
            })
    }
}

#[derive(Debug, Default, Serialize)]
struct MaegimControlPlan {
    controls: Vec<MaegimControlItem>,
}

#[derive(Debug, Serialize)]
struct MaegimControlPlanEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    plan: &'a MaegimControlPlan,
}

#[derive(Debug, Serialize)]
struct MaegimControlItem {
    name: String,
    decl_kind: String,
    type_name: String,
    init_expr_canon: String,
    fields: Vec<MaegimControlField>,
    range: Option<MaegimControlRange>,
    step_expr_canon: Option<String>,
    split_count_expr_canon: Option<String>,
}

#[derive(Debug, Default, Serialize)]
struct BlockEditorPlan {
    blocks: Vec<BlockEditorPlanNode>,
}

#[derive(Debug, Serialize)]
struct BlockEditorPlanEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    plan: &'a BlockEditorPlan,
}

impl BlockEditorPlan {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = BlockEditorPlanEnvelope {
            schema: "ddn.block_editor_plan.v1",
            plan: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_BLOCK_EDITOR_PLAN_JSON",
                    format!("block editor plan JSON 직렬화 실패: {}", err),
                )
            })
    }
}

#[derive(Debug, Default, Serialize)]
struct BlockEditorPlanNode {
    kind: String,
    fields: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    exprs: BTreeMap<String, BlockEditorExprNode>,
    inputs: BTreeMap<String, Vec<BlockEditorPlanNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_text: Option<String>,
}

#[derive(Debug, Serialize)]
struct BlockEditorExprNode {
    kind: String,
    text: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    fields: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    inputs: BTreeMap<String, Vec<BlockEditorExprNode>>,
}

fn build_block_editor_plan(stmts: &[Stmt]) -> BlockEditorPlan {
    let mut blocks = Vec::new();
    for stmt in stmts {
        blocks.push(build_block_editor_plan_stmt(stmt));
    }
    BlockEditorPlan { blocks }
}

fn build_block_editor_plan_stmt(stmt: &Stmt) -> BlockEditorPlanNode {
    match stmt {
        Stmt::RootDecl { items } => {
            let mut node = block_plan_node("charim_block");
            node.inputs.insert(
                "items".to_string(),
                items
                    .iter()
                    .map(build_block_editor_plan_decl_item)
                    .collect(),
            );
            node
        }
        Stmt::SeedDef {
            name,
            kind,
            params,
            body,
        } => {
            let mut node = block_plan_node("seed_def");
            node.fields.insert("name".to_string(), name.clone());
            node.fields.insert("kind".to_string(), kind.clone());
            node.fields
                .insert("params".to_string(), format_params(params));
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::BogaeMadangBlock { body } => {
            let mut node = block_plan_node("bogae_madang_block");
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Stmt::ExecPolicyBlock { body } => {
            let mut node = block_plan_node("exec_policy_block");
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Stmt::JjaimBlock { body } => {
            let mut node = block_plan_node("jjaim_block");
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Stmt::Expr { value } => {
            let mut node = block_plan_node("expr_stmt");
            node.fields.insert("expr".to_string(), format_expr(value));
            node.exprs
                .insert("expr".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::Assign { target, value } => {
            let mut node = block_plan_node("assign");
            node.fields
                .insert("target".to_string(), format_path(target));
            node.fields.insert("value".to_string(), format_expr(value));
            node.exprs
                .insert("value".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::Show { value } => {
            let mut node = block_plan_node("show");
            node.fields.insert("expr".to_string(), format_expr(value));
            node.exprs
                .insert("expr".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::Inspect { value } => {
            let mut node = block_plan_node("inspect_value");
            node.fields.insert("expr".to_string(), format_expr(value));
            node.exprs
                .insert("expr".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let kind = if else_body.is_some() {
                "if_else"
            } else {
                "if_then"
            };
            let mut node = block_plan_node(kind);
            node.fields
                .insert("cond".to_string(), format_condition(condition));
            node.exprs.insert(
                "cond".to_string(),
                build_block_editor_expr_node(&condition.expr),
            );
            node.inputs.insert(
                "then".to_string(),
                then_body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            if let Some(body) = else_body {
                node.inputs.insert(
                    "else".to_string(),
                    body.iter().map(build_block_editor_plan_stmt).collect(),
                );
            }
            node
        }
        Stmt::Repeat { body } => {
            let mut node = block_plan_node("repeat");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::While { condition, body } => {
            let mut node = block_plan_node("while_block");
            node.fields
                .insert("cond".to_string(), format_condition(condition));
            node.exprs.insert(
                "cond".to_string(),
                build_block_editor_expr_node(&condition.expr),
            );
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::ForEach {
            item,
            iterable,
            body,
        } => {
            let mut node = block_plan_node("for_each");
            node.fields.insert("item".to_string(), item.clone());
            node.fields
                .insert("iterable".to_string(), format_expr(iterable));
            node.exprs.insert(
                "iterable".to_string(),
                build_block_editor_expr_node(iterable),
            );
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Choose {
            branches,
            else_body,
        } => {
            let mut node = block_plan_node("choose_else");
            node.inputs.insert(
                "branches".to_string(),
                branches
                    .iter()
                    .map(build_block_editor_plan_choose_branch)
                    .collect(),
            );
            node.inputs.insert(
                "else".to_string(),
                else_body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::PromptChoose {
            branches,
            else_body,
        } => {
            let mut node = block_plan_node("prompt_choose");
            node.inputs.insert(
                "branches".to_string(),
                branches
                    .iter()
                    .map(build_block_editor_plan_choose_branch)
                    .collect(),
            );
            if let Some(body) = else_body {
                node.inputs.insert(
                    "else".to_string(),
                    body.iter().map(build_block_editor_plan_stmt).collect(),
                );
            }
            node
        }
        Stmt::PromptAfter { value, body } => {
            let mut node = block_plan_node("prompt_after");
            node.fields.insert("value".to_string(), format_expr(value));
            node.exprs
                .insert("value".to_string(), build_block_editor_expr_node(value));
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::PromptCondition { condition, body } => {
            let mut node = block_plan_node("prompt_condition");
            node.fields
                .insert("cond".to_string(), format_condition(condition));
            node.exprs.insert(
                "cond".to_string(),
                build_block_editor_expr_node(&condition.expr),
            );
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::PromptBlock { body } => {
            let mut node = block_plan_node("prompt_block");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Contract {
            kind,
            mode,
            condition,
            then_body,
            else_body,
        } => {
            let mut node = block_plan_node("contract_guard");
            node.fields
                .insert("cond".to_string(), format_condition(condition));
            node.exprs.insert(
                "cond".to_string(),
                build_block_editor_expr_node(&condition.expr),
            );
            node.fields.insert(
                "contract_kind".to_string(),
                match kind {
                    ContractKind::Pre => "pre".to_string(),
                    ContractKind::Post => "post".to_string(),
                },
            );
            node.fields.insert(
                "mode".to_string(),
                match mode {
                    ContractMode::Abort => "abort".to_string(),
                    ContractMode::Alert => "alert".to_string(),
                },
            );
            node.inputs.insert(
                "else".to_string(),
                else_body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            if let Some(body) = then_body {
                node.inputs.insert(
                    "then".to_string(),
                    body.iter().map(build_block_editor_plan_stmt).collect(),
                );
            }
            node
        }
        Stmt::OpenBlock { body } => {
            let mut node = block_plan_node("open_block");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Hook { name, suffix, body }
            if name == "시작" && matches!(suffix, HookSuffix::Halttae) =>
        {
            let mut node = block_plan_node("hook_start");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Hook { name, suffix, body }
            if name == "매마디" && matches!(suffix, HookSuffix::Mada) =>
        {
            let mut node = block_plan_node("hook_tick");
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Receive {
            kind,
            binding,
            condition,
            body,
        } => {
            let mut node = block_plan_node("receive_block");
            node.fields
                .insert("kind".to_string(), kind.clone().unwrap_or_default());
            node.fields
                .insert("binding".to_string(), binding.clone().unwrap_or_default());
            if let Some(condition) = condition {
                node.fields
                    .insert("condition".to_string(), format_expr(condition));
                node.exprs.insert(
                    "condition".to_string(),
                    build_block_editor_expr_node(condition),
                );
            } else {
                node.fields.insert("condition".to_string(), String::new());
            }
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::EventReact { kind, body } => {
            let mut node = block_plan_node("event_react");
            node.fields.insert("kind".to_string(), kind.clone());
            node.inputs.insert(
                "body".to_string(),
                body.iter().map(build_block_editor_plan_stmt).collect(),
            );
            node
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
        } => {
            let mut node = block_plan_node("send_signal");
            node.fields.insert(
                "sender".to_string(),
                sender.as_ref().map(format_expr).unwrap_or_default(),
            );
            node.fields
                .insert("payload".to_string(), format_expr(payload));
            node.fields
                .insert("receiver".to_string(), format_expr(receiver));
            if let Some(sender) = sender {
                node.exprs
                    .insert("sender".to_string(), build_block_editor_expr_node(sender));
            }
            node.exprs
                .insert("payload".to_string(), build_block_editor_expr_node(payload));
            node.exprs.insert(
                "receiver".to_string(),
                build_block_editor_expr_node(receiver),
            );
            node
        }
        Stmt::Return { value } => {
            let mut node = block_plan_node("return_value");
            node.fields.insert("value".to_string(), format_expr(value));
            node.exprs
                .insert("value".to_string(), build_block_editor_expr_node(value));
            node
        }
        Stmt::BogaeDraw => block_plan_node("bogae_draw"),
        Stmt::Break => block_plan_node("break_loop"),
        _ => raw_block_plan_node(format_program(&[stmt.clone()])),
    }
}

fn build_block_editor_plan_decl_item(item: &DeclItem) -> BlockEditorPlanNode {
    if let Some(maegim) = &item.maegim {
        let mut range = None;
        let mut step = None;
        let mut split_count = None;
        let mut unsupported = false;
        for field in &maegim.fields {
            let leaf = maegim_field_leaf_name(&field.name);
            match leaf {
                "범위" => {
                    range = Some(format_maegim_field_value(field));
                }
                "간격" => {
                    step = Some(format_expr(&field.value));
                }
                "분할수" => {
                    split_count = Some(format_expr(&field.value));
                }
                _ => {
                    unsupported = true;
                }
            }
        }
        if !unsupported && range.is_some() && (step.is_some() ^ split_count.is_some()) {
            let kind = if step.is_some() {
                "charim_item_const_step"
            } else {
                "charim_item_const_split"
            };
            let mut node = block_plan_node(kind);
            node.fields.insert("name".to_string(), item.name.clone());
            node.fields
                .insert("type_name".to_string(), item.type_name.clone());
            node.fields.insert(
                "value".to_string(),
                item.value
                    .as_ref()
                    .map(format_expr)
                    .unwrap_or_else(String::new),
            );
            if let Some(value) = item.value.as_ref() {
                node.exprs
                    .insert("value".to_string(), build_block_editor_expr_node(value));
            }
            node.fields
                .insert("range".to_string(), range.unwrap_or_default());
            if let Some(step) = step {
                node.fields.insert("step".to_string(), step);
            }
            if let Some(split_count) = split_count {
                node.fields.insert("split_count".to_string(), split_count);
            }
            return node;
        }
        return raw_block_plan_node(format_decl_item(item, 0));
    }

    if item.kind == DeclKind::Gureut {
        let mut node = block_plan_node("charim_item_var");
        node.fields.insert("name".to_string(), item.name.clone());
        node.fields
            .insert("type_name".to_string(), item.type_name.clone());
        node.fields.insert(
            "value".to_string(),
            item.value
                .as_ref()
                .map(format_expr)
                .unwrap_or_else(String::new),
        );
        if let Some(value) = item.value.as_ref() {
            node.exprs
                .insert("value".to_string(), build_block_editor_expr_node(value));
        }
        return node;
    }

    let mut node = block_plan_node("charim_item_plain");
    node.fields.insert("name".to_string(), item.name.clone());
    node.fields
        .insert("type_name".to_string(), item.type_name.clone());
    node.fields.insert(
        "value".to_string(),
        item.value
            .as_ref()
            .map(format_expr)
            .unwrap_or_else(String::new),
    );
    if let Some(value) = item.value.as_ref() {
        node.exprs
            .insert("value".to_string(), build_block_editor_expr_node(value));
    }
    node
}

fn build_block_editor_plan_choose_branch(branch: &ChooseBranch) -> BlockEditorPlanNode {
    let mut node = block_plan_node("choose_branch");
    node.fields
        .insert("cond".to_string(), format_condition(&branch.condition));
    node.exprs.insert(
        "cond".to_string(),
        build_block_editor_expr_node(&branch.condition.expr),
    );
    node.inputs.insert(
        "body".to_string(),
        branch
            .body
            .iter()
            .map(build_block_editor_plan_stmt)
            .collect(),
    );
    node
}

fn build_block_editor_binding_expr_node(binding: &Binding) -> BlockEditorExprNode {
    let mut node = block_editor_expr_node("binding", format_expr(&binding.value));
    node.fields.insert("name".to_string(), binding.name.clone());
    node.inputs.insert(
        "value".to_string(),
        vec![build_block_editor_expr_node(&binding.value)],
    );
    node
}

fn build_block_editor_expr_node(expr: &Expr) -> BlockEditorExprNode {
    match expr {
        Expr::Literal(value) => {
            let mut node = block_editor_expr_node("literal", format_expr(expr));
            node.fields.insert(
                "literal_kind".to_string(),
                match value {
                    Literal::Str(_) => "str".to_string(),
                    Literal::Num(_) => "num".to_string(),
                    Literal::Bool(_) => "bool".to_string(),
                    Literal::None => "none".to_string(),
                    Literal::Atom(_) => "atom".to_string(),
                },
            );
            node.fields
                .insert("value".to_string(), format_literal(value));
            node
        }
        Expr::Resource(name) => {
            let mut node = block_editor_expr_node("resource", format_expr(expr));
            node.fields.insert("name".to_string(), name.clone());
            node
        }
        Expr::Template { body } => {
            let mut node = block_editor_expr_node("template", format_expr(expr));
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Expr::TemplateApply { bindings, body } => {
            let mut node = block_editor_expr_node("template_apply", format_expr(expr));
            node.fields.insert("body".to_string(), body.clone());
            node.inputs.insert(
                "bindings".to_string(),
                bindings
                    .iter()
                    .map(build_block_editor_binding_expr_node)
                    .collect(),
            );
            node
        }
        Expr::Formula { tag, body } => {
            let mut node = block_editor_expr_node("formula", format_expr(expr));
            node.fields
                .insert("tag".to_string(), tag.clone().unwrap_or_default());
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Expr::PromptExpr { expr: inner } => {
            let mut node = block_editor_expr_node("prompt_expr", format_expr(expr));
            node.inputs.insert(
                "expr".to_string(),
                vec![build_block_editor_expr_node(inner)],
            );
            node
        }
        Expr::PromptBlock { body } => {
            let mut node = block_editor_expr_node("prompt_block_expr", format_expr(expr));
            node.fields.insert("body".to_string(), body.clone());
            node
        }
        Expr::Path(path) => {
            let mut node = block_editor_expr_node("path", format_expr(expr));
            node.fields.insert("path".to_string(), format_path(path));
            node.fields
                .insert("segments".to_string(), path.segments.join("/"));
            node
        }
        Expr::FieldAccess { target, field } => {
            let mut node = block_editor_expr_node("field_access", format_expr(expr));
            node.fields.insert("field".to_string(), field.clone());
            node.inputs.insert(
                "target".to_string(),
                vec![build_block_editor_expr_node(target)],
            );
            node
        }
        Expr::Call { name, args } => {
            let mut node = block_editor_expr_node("call", format_expr(expr));
            node.fields.insert("name".to_string(), name.clone());
            node.inputs.insert(
                "args".to_string(),
                args.iter().map(build_block_editor_expr_node).collect(),
            );
            node
        }
        Expr::CallIn { name, bindings } => {
            let mut node = block_editor_expr_node("call_in", format_expr(expr));
            node.fields.insert("name".to_string(), name.clone());
            node.inputs.insert(
                "bindings".to_string(),
                bindings
                    .iter()
                    .map(build_block_editor_binding_expr_node)
                    .collect(),
            );
            node
        }
        Expr::Pack { bindings } => {
            let mut node = block_editor_expr_node("pack", format_expr(expr));
            node.inputs.insert(
                "bindings".to_string(),
                bindings
                    .iter()
                    .map(build_block_editor_binding_expr_node)
                    .collect(),
            );
            node
        }
        Expr::Unit { value, unit } => {
            let mut node = block_editor_expr_node("unit", format_expr(expr));
            node.fields.insert("unit".to_string(), unit.clone());
            node.inputs.insert(
                "value".to_string(),
                vec![build_block_editor_expr_node(value)],
            );
            node
        }
        Expr::Unary { op, expr: inner } => {
            let mut node = block_editor_expr_node("unary", format_expr(expr));
            node.fields.insert(
                "op".to_string(),
                match op {
                    UnaryOp::Neg => "neg".to_string(),
                },
            );
            node.inputs.insert(
                "expr".to_string(),
                vec![build_block_editor_expr_node(inner)],
            );
            node
        }
        Expr::Binary { left, op, right } => {
            let mut node = block_editor_expr_node("binary", format_expr(expr));
            node.fields.insert(
                "op".to_string(),
                match op {
                    BinaryOp::Add => "add".to_string(),
                    BinaryOp::Sub => "sub".to_string(),
                    BinaryOp::Mul => "mul".to_string(),
                    BinaryOp::Div => "div".to_string(),
                    BinaryOp::Mod => "mod".to_string(),
                    BinaryOp::And => "and".to_string(),
                    BinaryOp::Or => "or".to_string(),
                    BinaryOp::Eq => "eq".to_string(),
                    BinaryOp::NotEq => "neq".to_string(),
                    BinaryOp::Lt => "lt".to_string(),
                    BinaryOp::Lte => "lte".to_string(),
                    BinaryOp::Gt => "gt".to_string(),
                    BinaryOp::Gte => "gte".to_string(),
                },
            );
            node.inputs
                .insert("left".to_string(), vec![build_block_editor_expr_node(left)]);
            node.inputs.insert(
                "right".to_string(),
                vec![build_block_editor_expr_node(right)],
            );
            node
        }
        Expr::Pipe { left, kind, right } => {
            let mut node = block_editor_expr_node("pipe", format_expr(expr));
            node.fields.insert(
                "pipe_kind".to_string(),
                match kind {
                    PipeKind::Haseo => "haseo".to_string(),
                    PipeKind::Hago => "hago".to_string(),
                },
            );
            node.inputs
                .insert("left".to_string(), vec![build_block_editor_expr_node(left)]);
            node.inputs.insert(
                "right".to_string(),
                vec![build_block_editor_expr_node(right)],
            );
            node
        }
        Expr::SeedLiteral { param, body } => {
            let mut node = block_editor_expr_node("seed_literal", format_expr(expr));
            node.fields.insert("param".to_string(), param.clone());
            node.inputs
                .insert("body".to_string(), vec![build_block_editor_expr_node(body)]);
            node
        }
    }
}

fn block_editor_expr_node(kind: &str, text: String) -> BlockEditorExprNode {
    BlockEditorExprNode {
        kind: kind.to_string(),
        text,
        fields: BTreeMap::new(),
        inputs: BTreeMap::new(),
    }
}

fn block_plan_node(kind: &str) -> BlockEditorPlanNode {
    BlockEditorPlanNode {
        kind: kind.to_string(),
        fields: BTreeMap::new(),
        exprs: BTreeMap::new(),
        inputs: BTreeMap::new(),
        raw_text: None,
    }
}

fn raw_block_plan_node(text: String) -> BlockEditorPlanNode {
    let mut node = block_plan_node("raw");
    node.raw_text = Some(text.trim().to_string());
    node
}

#[derive(Debug, Serialize)]
struct MaegimControlField {
    name: String,
    value_canon: String,
}

#[derive(Debug, Serialize)]
struct MaegimControlRange {
    min_expr_canon: String,
    max_expr_canon: String,
    inclusive_end: bool,
}

impl MaegimControlPlan {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = MaegimControlPlanEnvelope {
            schema: "ddn.maegim_control_plan.v1",
            plan: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_MAEGIM_CONTROL_JSON",
                    format!("매김 제어 계획 JSON 직렬화 실패: {}", err),
                )
            })
    }
}

#[derive(Debug, Serialize)]
struct ExecPolicyEffectMap {
    language_axis: ExecPolicyLanguageAxis,
    tool_axis: ExecPolicyToolAxis,
    selected_behavior_by_open_mode: Vec<ExecPolicyOpenModeBehavior>,
}

#[derive(Debug, Serialize)]
struct ExecPolicyLanguageAxis {
    block_count: u64,
    exec_mode_raw: Option<String>,
    effect_policy_raw: Option<String>,
    exec_mode_effective: String,
    effect_policy_effective: String,
    effect_policy_explicit: bool,
    strict_effect_ignored: bool,
    would_fail_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExecPolicyToolAxis {
    open_mode_values: Vec<String>,
    effective_open_mode_resolution_order: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ExecPolicyOpenModeBehavior {
    open_mode: String,
    effect_boundary_result: String,
}

#[derive(Debug, Serialize)]
struct ExecPolicyEffectMapEnvelope<'a> {
    schema: &'static str,
    #[serde(flatten)]
    map: &'a ExecPolicyEffectMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecMode {
    Strict,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectPolicy {
    Isolated,
    Allowed,
}

#[derive(Debug, Default)]
struct ExecPolicyFields {
    exec_mode_raw: Option<String>,
    effect_policy_raw: Option<String>,
}

impl ExecPolicyEffectMap {
    fn to_json_string(&self) -> Result<String, CanonError> {
        let envelope = ExecPolicyEffectMapEnvelope {
            schema: "ddn.exec_policy_effect_map.v1",
            map: self,
        };
        serde_json::to_string_pretty(&envelope)
            .map(|text| format!("{}\n", text))
            .map_err(|err| {
                CanonError::new(
                    "E_CANON_EXEC_POLICY_MAP_JSON",
                    format!("실행정책/효과 경계 매핑 JSON 직렬화 실패: {}", err),
                )
            })
    }
}

fn parse_exec_policy_fields(body: &str) -> ExecPolicyFields {
    let mut fields = ExecPolicyFields::default();
    for raw_line in body.lines() {
        let mut line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if let Some(stripped) = line.strip_suffix('.') {
            line = stripped.trim_end();
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().to_string();
        match key {
            "실행모드" => fields.exec_mode_raw = Some(value),
            "효과정책" => fields.effect_policy_raw = Some(value),
            _ => {}
        }
    }
    fields
}

fn parse_exec_mode_value(text: &str) -> Option<ExecMode> {
    match text.trim() {
        "엄밀" | "strict" | "STRICT" => Some(ExecMode::Strict),
        "일반" | "general" | "GENERAL" => Some(ExecMode::General),
        _ => None,
    }
}

fn parse_effect_policy_value(text: &str) -> Option<EffectPolicy> {
    match text.trim() {
        "격리" | "isolate" | "isolated" | "ISOLATE" | "ISOLATED" => Some(EffectPolicy::Isolated),
        "허용" | "allow" | "ALLOW" | "allowed" | "ALLOWED" => Some(EffectPolicy::Allowed),
        _ => None,
    }
}

fn exec_mode_label(mode: ExecMode) -> String {
    match mode {
        ExecMode::Strict => "엄밀".to_string(),
        ExecMode::General => "일반".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{canonicalize, has_exec_policy_surface};

    #[test]
    fn canon_accepts_signal_send_and_named_seed_kind() {
        let source = r#"
(온도:수, 풍속:수) 기상특보:알림씨 = {
}.
기상청:임자 = {
  (온도:1, 풍속:12@m/s) 기상특보 ~~> 관제탑.
  (기상청)의 (온도:2, 풍속:14@m/s) 기상특보 ~~> 관제탑.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("기상특보:알림씨 = {"));
        assert!(out
            .ddn
            .contains("(온도:1, 풍속:12@m / s) 기상특보 ~~> 관제탑."));
        assert!(out
            .ddn
            .contains("(기상청)의 (온도:2, 풍속:14@m / s) 기상특보 ~~> 관제탑."));
    }

    #[test]
    fn canon_accepts_receive_hooks_surface() {
        let source = r#"
관제탑:임자 = {
  기상특보를 받으면 {
    참 보여주기.
  }.

  (정보 정보.온도 > 40)인 기상특보를 받으면 {
    정보.온도 보여주기.
  }.

  알림을 받으면 {
    알림.이름 보여주기.
  }.

  (알림 알림.이름 == "기상특보")인 알림을 받으면 {
    알림.보낸이 보여주기.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("기상특보을 받으면 {") || out.ddn.contains("기상특보를 받으면 {"));
        assert!(out
            .ddn
            .contains("(정보 정보.온도 > 40)인 기상특보를 받으면 {"));
        assert!(out.ddn.contains("알림을 받으면 {"));
        assert!(out
            .ddn
            .contains("(알림 알림.이름 == \"기상특보\")인 알림을 받으면 {"));
    }

    #[test]
    fn canon_rejects_receive_hook_outside_imja() {
        let source = r#"
기상청:움직씨 = {
  알림을 받으면 {
    참 보여주기.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_RECEIVE_OUTSIDE_IMJA");
    }

    #[test]
    fn canon_accepts_every_n_madi_hook_surface() {
        let source = r#"
(16마디)마다 {
  "tick16" 보여주기.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("(16마디)마다 {"));
    }

    #[test]
    fn canon_rejects_every_n_madi_zero_interval() {
        let source = r#"
(0마디)마다 {
  "bad" 보여주기.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_HOOK_EVERY_N_MADI_INTERVAL_INVALID");
    }

    #[test]
    fn canon_rejects_every_n_madi_unsupported_unit() {
        let source = r#"
(3 foo)마다 {
  "bad" 보여주기.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED");
    }

    #[test]
    fn canon_rejects_every_n_madi_unsupported_suffix() {
        let source = r#"
(3마디)할때 {
  "bad" 보여주기.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED");
    }

    #[test]
    fn canon_normalizes_hook_start_alias_choeum_to_sijak() {
        let source = r#"
(처음)할때 {
  "boot" 보여주기.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("(시작)할때 {"));
        assert!(!out.ddn.contains("(처음)할때 {"));
    }

    #[test]
    fn canon_normalizes_hook_tick_alias_maetic_to_maemadi() {
        let source = r#"
(매틱)마다 {
  "tick" 보여주기.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("(매마디)마다 {"));
        assert!(!out.ddn.contains("(매틱)마다 {"));
    }

    #[test]
    fn canon_rejects_hook_colon_header_surface() {
        let source = r#"
(매마디)마다: {
  "tick" 보여주기.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_BLOCK_HEADER_COLON_FORBIDDEN");
    }

    #[test]
    fn canon_accepts_maegim_alias_and_normalizes_to_maegim() {
        let source = r#"
채비 {
  g:수 = (9.8) 조건 {
    범위: 1..20.
    간격: 0.1.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("g:수 = (9.8) 매김 {"));
        assert!(out.ddn.contains("범위: 1..20."));
        assert!(out.ddn.contains("간격: 0.1."));
    }

    #[test]
    fn canon_accepts_nested_maegim_sections_and_normalizes_canonical_sections() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      범위: 1..20.
      간격: 0.1.
    }.
    갈래 {
      가만히: 참.
      벗남다룸: "잘림".
    }.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("g:수 = (9.8) 매김 {"));
        assert!(out.ddn.contains("가늠 {"));
        assert!(out.ddn.contains("범위: 1..20."));
        assert!(out.ddn.contains("간격: 0.1."));
        assert!(out.ddn.contains("갈래 {"));
        assert!(out.ddn.contains("가만히: 참."));
        assert!(out.ddn.contains("벗남다룸: \"잘림\"."));
        assert!(!out.ddn.contains("가늠.범위:"));
        assert!(out
            .maegim_control_json
            .contains("\"step_expr_canon\": \"0.1\""));
    }

    #[test]
    fn canon_rejects_nested_maegim_step_and_split_count_together() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      간격: 0.1.
      분할수: 24.
    }.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_MAEGIM_STEP_SPLIT_CONFLICT");
    }

    #[test]
    fn canon_rejects_unknown_nested_maegim_section() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    실험 {
      범위: 1..20.
    }.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_MAEGIM_NESTED_SECTION_UNSUPPORTED");
    }

    #[test]
    fn canon_rejects_unknown_nested_maegim_field_in_ganeum() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      최소값: 1.
    }.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_MAEGIM_NESTED_FIELD_UNSUPPORTED");
    }

    #[test]
    fn canon_emits_maegim_control_plan_json() {
        let source = r#"
채비 {
  g:수 = (9.8) 조건 {
    범위: 1..20.
    간격: 0.1.
  }.
  theta0:수 <- (0.5) 매김 {
    범위: -1.2..1.2.
    분할수: 24.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .maegim_control_json
            .contains("\"schema\": \"ddn.maegim_control_plan.v1\""));
        assert!(out.maegim_control_json.contains("\"name\": \"g\""));
        assert!(out
            .maegim_control_json
            .contains("\"decl_kind\": \"butbak\""));
        assert!(out
            .maegim_control_json
            .contains("\"step_expr_canon\": \"0.1\""));
        assert!(out.maegim_control_json.contains("\"name\": \"theta0\""));
        assert!(out
            .maegim_control_json
            .contains("\"decl_kind\": \"gureut\""));
        assert!(out
            .maegim_control_json
            .contains("\"min_expr_canon\": \"-1.2\""));
        assert!(out
            .maegim_control_json
            .contains("\"max_expr_canon\": \"1.2\""));
        assert!(out
            .maegim_control_json
            .contains("\"split_count_expr_canon\": \"24\""));
    }

    #[test]
    fn canon_emits_maegim_control_plan_json_from_seed_body_decl_block() {
        let source = r#"
매틱:움직씨 = {
  채비 {
    g:수 = (9.8) 조건 {
      범위: 1..20.
      간격: 0.1.
    }.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .maegim_control_json
            .contains("\"schema\": \"ddn.maegim_control_plan.v1\""));
        assert!(out.maegim_control_json.contains("\"name\": \"g\""));
        assert!(out
            .maegim_control_json
            .contains("\"step_expr_canon\": \"0.1\""));
    }

    #[test]
    fn canon_emits_block_editor_plan_json_for_nested_maegim_sections() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      범위: 1..20.
      간격: 0.1.
    }.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"charim_item_const_step\""));
    }

    #[test]
    fn canon_emits_block_editor_plan_json() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
  }.
  t:변수 <- 0.
  안내:글 = "초기".
}.

(시작)할때 {
  안내 보여주기.
}.

(매마디)마다 {
  t <- t + 1.
  t < 3 일때 {
    t 보여주기.
  } 아니면 {
    g 보여주기.
  }.
}.

(x) x목록에 대해 {
  고르기:
    { x < 1 }인것: {
      "low" 보여주기.
    }
    아니면: {
      "high" 보여주기.
    }.
}.

되풀이 {
  "tick" 보여주기.
}.

너머 {
  "side" 보여주기.
}.

{ t < 2 }인것 동안 {
  t <- t + 1.
}.

{ t == 2 }인것 바탕으로 아니면 {
  "bad" 보여주기.
} 맞으면 {
  "ok" 보여주기.
}.

{ t >= 0 }인것 다짐하고(알림) 아니면 {
  "warn" 보여주기.
}.

?? {
  "prompt" 보여주기.
}

"answer" ??: {
  "after" 보여주기.
}

{ t >= 0 }인것 ??
?? {
  "cond" 보여주기.
}

??:
  { t < 1 }인것: {
    "choose-low" 보여주기.
  }
??: {
    "choose-else" 보여주기.
  }

(기준:수 = 1) 판정:움직씨 = {
  기준 되돌림.
}.

기상청:임자 = {
  기상특보를 받으면 {
    알림.이름 보여주기.
  }.

  (정보 정보.온도 > 40)인 기상특보를 받으면 {
    (기상청)의 (온도:정보.온도) 경보 ~~> 제.
  }.

  (알림 알림.이름 == "경보")인 알림을 받으면 {
    알림.정보.온도 보여주기.
  }.
}.

"tick"라는 알림이 오면 {
  "evt" 보여주기.
}

보개로 그려.

(매마디)마다 {
  t 톺아보기.
  되풀이 {
    t <- t + 1.
    { t > 0 }인것 일때 {
      멈추기.
    }.
  }.
}

실행정책 {
  실행모드: 일반.
  효과정책: 허용.
}.

짜임 {
  형식: 점_틀.
  입력 {
    시작점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    끝점: (수,수) <- (0.0, 0.0).
  }.
}.

테스트:움직씨 = {
  보개마당 {
    #자막("정본 테스트").
  }.
}.

() 증명.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .block_editor_plan_json
            .contains("\"schema\": \"ddn.block_editor_plan.v1\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"charim_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"charim_item_const_step\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"hook_start\""));
        assert!(out.block_editor_plan_json.contains("\"kind\": \"if_else\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"for_each\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"choose_else\""));
        assert!(out.block_editor_plan_json.contains("\"kind\": \"repeat\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"open_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"while_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"contract_guard\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"prompt_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"prompt_after\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"prompt_condition\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"prompt_choose\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"seed_def\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"receive_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"event_react\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"send_signal\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"return_value\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"inspect_value\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"bogae_draw\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"break_loop\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"bogae_madang_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"exec_policy_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"jjaim_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"expr_stmt\""));
        assert!(out.block_editor_plan_json.contains("\"exprs\": {"));
        assert!(out.block_editor_plan_json.contains("\"kind\": \"call\""));
        assert!(out.block_editor_plan_json.contains("\"kind\": \"binding\""));
    }

    #[test]
    fn canon_rejects_maegim_step_and_split_count_together() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
    분할수: 24.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_MAEGIM_STEP_SPLIT_CONFLICT");
    }

    #[test]
    fn canon_accepts_decl_value_prefix_call_with_parenthesized_args() {
        let source = r#"
채비 {
  목록:차림 <- (1, 2) 차림.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("목록:차림 <- (1, 2) 차림."));
    }

    #[test]
    fn canon_accepts_block_header_colon_with_foreach_and_prefix_call() {
        let source = r#"
채비: {
  목록:차림 <- (1, 2) 차림.
}.

매마디:움직씨 = {
  반복: {
    (x) 목록에 대해: {
      x 보여주기.
    }.
    멈추기.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("목록:차림 <- (1, 2) 차림."));
        assert!(out.ddn.contains("되풀이 {"));
        assert!(out.ddn.contains("(x) 목록에 대해 {"));
        assert!(out
            .warnings
            .iter()
            .any(|w| w.contains("W_BLOCK_HEADER_COLON_DEPRECATED")));
    }

    #[test]
    fn canon_normalizes_repeat_and_return_surface_to_canonical_keywords() {
        let source = r#"
테스트:셈씨 = {
  반복: {
    1 반환.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("되풀이 {"));
        assert!(out.ddn.contains("1 되돌림."));
        assert!(!out.ddn.contains("반복 {"));
        assert!(!out.ddn.contains("1 반환."));
    }

    #[test]
    fn canon_accepts_unary_plus_literals() {
        let source = r#"
채비 {
  x:수 <- +5.
  y:수 <- (+5).
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("x:수 <- 5."));
        assert!(out.ddn.contains("y:수 <- 5."));
    }

    #[test]
    fn canon_normalizes_audit_surface_to_canonical_keyword() {
        let source = r#"
테스트:셈씨 = {
  값 감사.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("값 톺아보기."));
        assert!(!out.ddn.contains("값 감사."));
    }

    #[test]
    fn canon_normalizes_legacy_bogae_jangmyeon_block() {
        let source = r#"
보개장면 {
  #자막("legacy").
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("보개마당 {"));
        assert!(!out.ddn.contains("보개장면 {"));
    }

    #[test]
    fn canon_frontdoor_accepts_file_leading_setting_block() {
        let source = r#"
설정 {
  문서 {
    이름: "프론트도어".
  }.
}.

채비 {
  x:수 <- 1.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("채비 {"));
        assert!(out.ddn.contains("x:수 <- 1."));
        assert!(!out.ddn.contains("설정 {"));
    }

    #[test]
    fn exec_policy_surface_detector_distinguishes_presence() {
        let no_policy = r#"
채비 {
  너머값:수 <- 1.
}.
"#;
        assert!(!has_exec_policy_surface(no_policy));

        let with_policy = r#"
너머 {
  실행모드: 엄밀.
  효과정책: 격리.
}.
"#;
        assert!(has_exec_policy_surface(with_policy));
    }

    #[test]
    fn canon_exec_policy_map_emits_strict_default_contract_without_surface() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .exec_policy_map_json
            .contains("\"schema\": \"ddn.exec_policy_effect_map.v1\""));
        assert!(out.exec_policy_map_json.contains("\"block_count\": 0"));
        assert!(out
            .exec_policy_map_json
            .contains("\"exec_mode_effective\": \"엄밀\""));
        assert!(out
            .exec_policy_map_json
            .contains("\"effect_policy_effective\": \"격리\""));
    }

    #[test]
    fn canon_rejects_legacy_hash_header_on_frontdoor() {
        let source = r#"
#이름: 금지
채비 {
  x:수 <- 1.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN");
    }

    #[test]
    fn canon_rejects_legacy_boim_surface_on_frontdoor() {
        let source = r#"
(매마디)마다 {
  보임 {
    x: 1.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_LEGACY_BOIM_FORBIDDEN");
    }
}

fn effect_policy_label(policy: EffectPolicy) -> String {
    match policy {
        EffectPolicy::Isolated => "격리".to_string(),
        EffectPolicy::Allowed => "허용".to_string(),
    }
}

fn build_maegim_control_plan(stmts: &[Stmt]) -> MaegimControlPlan {
    let mut controls = Vec::new();
    collect_maegim_controls_from_stmts(stmts, &mut controls);
    MaegimControlPlan { controls }
}

fn collect_maegim_controls_from_stmts(stmts: &[Stmt], controls: &mut Vec<MaegimControlItem>) {
    for stmt in stmts {
        match stmt {
            Stmt::RootDecl { items } => collect_maegim_controls_from_decl_items(items, controls),
            Stmt::SeedDef { body, .. }
            | Stmt::Repeat { body }
            | Stmt::While { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::Hook { body, .. }
            | Stmt::Receive { body, .. }
            | Stmt::EventReact { body, .. }
            | Stmt::OpenBlock { body }
            | Stmt::PromptAfter { body, .. }
            | Stmt::PromptCondition { body, .. }
            | Stmt::PromptBlock { body } => collect_maegim_controls_from_stmts(body, controls),
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_maegim_controls_from_stmts(then_body, controls);
                if let Some(else_body) = else_body {
                    collect_maegim_controls_from_stmts(else_body, controls);
                }
            }
            Stmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                if let Some(then_body) = then_body {
                    collect_maegim_controls_from_stmts(then_body, controls);
                }
                collect_maegim_controls_from_stmts(else_body, controls);
            }
            Stmt::Choose { branches, else_body } => {
                for branch in branches {
                    collect_maegim_controls_from_stmts(&branch.body, controls);
                }
                collect_maegim_controls_from_stmts(else_body, controls);
            }
            Stmt::PromptChoose { branches, else_body } => {
                for branch in branches {
                    collect_maegim_controls_from_stmts(&branch.body, controls);
                }
                if let Some(else_body) = else_body {
                    collect_maegim_controls_from_stmts(else_body, controls);
                }
            }
            _ => {}
        }
    }
}

fn collect_maegim_controls_from_decl_items(items: &[DeclItem], controls: &mut Vec<MaegimControlItem>) {
    for item in items {
        let Some(maegim) = &item.maegim else {
            continue;
        };
        let Some(value) = &item.value else {
            continue;
        };
        let fields = maegim
            .fields
            .iter()
            .map(|binding| MaegimControlField {
                name: binding.name.clone(),
                value_canon: format_maegim_field_value(binding),
            })
            .collect();
        let range = maegim
            .fields
            .iter()
            .find(|binding| maegim_field_leaf_name(&binding.name) == "범위")
            .and_then(|binding| parse_maegim_range_expr(&binding.value))
            .map(
                |(min_expr_canon, max_expr_canon, inclusive_end)| MaegimControlRange {
                    min_expr_canon,
                    max_expr_canon,
                    inclusive_end,
                },
            );
        let step_expr_canon = maegim
            .fields
            .iter()
            .find(|binding| maegim_field_leaf_name(&binding.name) == "간격")
            .map(|binding| format_expr(&binding.value));
        let split_count_expr_canon = maegim
            .fields
            .iter()
            .find(|binding| maegim_field_leaf_name(&binding.name) == "분할수")
            .map(|binding| format_expr(&binding.value));
        controls.push(MaegimControlItem {
            name: item.name.clone(),
            decl_kind: match item.kind {
                DeclKind::Gureut => "gureut".to_string(),
                DeclKind::Butbak => "butbak".to_string(),
            },
            type_name: item.type_name.clone(),
            init_expr_canon: format_expr(value),
            fields,
            range,
            step_expr_canon,
            split_count_expr_canon,
        });
    }
}

fn parse_maegim_range_expr(expr: &Expr) -> Option<(String, String, bool)> {
    let Expr::Call { name, args, .. } = expr else {
        return None;
    };
    if name != "표준.범위" || args.len() != 3 {
        return None;
    }
    let inclusive_end = matches!(&args[2], Expr::Literal(Literal::Num(value)) if value == "1");
    Some((format_expr(&args[0]), format_expr(&args[1]), inclusive_end))
}

fn build_exec_policy_effect_map(stmts: &[Stmt]) -> ExecPolicyEffectMap {
    let blocks: Vec<&str> = stmts
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::ExecPolicyBlock { body } => Some(body.as_str()),
            _ => None,
        })
        .collect();
    let block_count = blocks.len() as u64;
    let first_fields = blocks
        .first()
        .map(|body| parse_exec_policy_fields(body))
        .unwrap_or_default();

    let mode_parse = first_fields
        .exec_mode_raw
        .as_ref()
        .map(|raw| parse_exec_mode_value(raw));
    let effect_parse = first_fields
        .effect_policy_raw
        .as_ref()
        .map(|raw| parse_effect_policy_value(raw));

    let mut would_fail_code: Option<String> = None;
    if matches!(mode_parse, Some(None)) || matches!(effect_parse, Some(None)) {
        would_fail_code = Some("E_EXEC_ENUM_INVALID".to_string());
    } else if block_count > 1 {
        would_fail_code = Some("E_EXEC_POLICY_DUPLICATE".to_string());
    }

    let mode = mode_parse.flatten().unwrap_or(ExecMode::Strict);
    let effect_explicit = effect_parse.flatten();
    let mut effect = effect_explicit.unwrap_or(EffectPolicy::Isolated);
    let strict_effect_ignored =
        mode == ExecMode::Strict && matches!(effect_explicit, Some(EffectPolicy::Allowed));
    if mode == ExecMode::Strict {
        effect = EffectPolicy::Isolated;
    }

    let language_axis = ExecPolicyLanguageAxis {
        block_count,
        exec_mode_raw: first_fields.exec_mode_raw,
        effect_policy_raw: first_fields.effect_policy_raw,
        exec_mode_effective: exec_mode_label(mode),
        effect_policy_effective: effect_policy_label(effect),
        effect_policy_explicit: effect_explicit.is_some(),
        strict_effect_ignored,
        would_fail_code: would_fail_code.clone(),
    };

    let open_modes = ["deny", "record", "replay"];
    let selected_behavior_by_open_mode = open_modes
        .iter()
        .map(|open_mode| {
            let effect_boundary_result = if let Some(code) = &would_fail_code {
                format!("gate_error:{}", code)
            } else if mode == ExecMode::Strict {
                "compile_error:E_EFFECT_IN_STRICT_MODE".to_string()
            } else if effect == EffectPolicy::Isolated {
                "runtime_error:E_EFFECT_IN_ISOLATED_MODE".to_string()
            } else {
                match *open_mode {
                    "deny" => "runtime_error:E_OPEN_DENIED".to_string(),
                    "record" => "open_log:record".to_string(),
                    "replay" => "open_log:replay".to_string(),
                    _ => "unknown".to_string(),
                }
            };
            ExecPolicyOpenModeBehavior {
                open_mode: (*open_mode).to_string(),
                effect_boundary_result,
            }
        })
        .collect();

    ExecPolicyEffectMap {
        language_axis,
        tool_axis: ExecPolicyToolAxis {
            open_mode_values: vec![
                "deny".to_string(),
                "record".to_string(),
                "replay".to_string(),
            ],
            effective_open_mode_resolution_order: vec![
                "cli".to_string(),
                "open_policy".to_string(),
                "default_deny".to_string(),
            ],
        },
        selected_behavior_by_open_mode,
    }
}

fn build_alrim_event_plan(stmts: &[Stmt]) -> AlrimEventPlan {
    let mut handlers = Vec::new();
    collect_alrim_event_handlers(stmts, "root", &mut handlers);
    for (idx, handler) in handlers.iter_mut().enumerate() {
        handler.order = idx as u64;
    }
    AlrimEventPlan { handlers }
}

fn collect_alrim_event_handlers(stmts: &[Stmt], scope: &str, out: &mut Vec<AlrimEventHandler>) {
    for stmt in stmts {
        match stmt {
            Stmt::Receive { kind, body, .. } => {
                let handler_kind = kind.clone().unwrap_or_else(|| "알림".to_string());
                out.push(AlrimEventHandler {
                    order: 0,
                    kind: handler_kind.clone(),
                    scope: scope.to_string(),
                    body_canon: format_program(body),
                });
                let nested_scope = format!("{}/receive:{}", scope, handler_kind);
                collect_alrim_event_handlers(body, &nested_scope, out);
            }
            Stmt::EventReact { kind, body } => {
                out.push(AlrimEventHandler {
                    order: 0,
                    kind: kind.clone(),
                    scope: scope.to_string(),
                    body_canon: format_program(body),
                });
                let nested_scope = format!("{}/event:{}", scope, kind);
                collect_alrim_event_handlers(body, &nested_scope, out);
            }
            Stmt::SeedDef { name, body, .. } => {
                let next = format!("{}/seed:{}", scope, name);
                collect_alrim_event_handlers(body, &next, out);
            }
            Stmt::Hook { name, suffix, body } => {
                let suffix_text = match suffix {
                    HookSuffix::Halttae => "할때",
                    HookSuffix::Mada => "마다",
                };
                let next = format!("{}/hook:{}:{}", scope, name, suffix_text);
                collect_alrim_event_handlers(body, &next, out);
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_alrim_event_handlers(then_body, scope, out);
                if let Some(else_body) = else_body {
                    collect_alrim_event_handlers(else_body, scope, out);
                }
            }
            Stmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                if let Some(then_body) = then_body {
                    collect_alrim_event_handlers(then_body, scope, out);
                }
                collect_alrim_event_handlers(else_body, scope, out);
            }
            Stmt::Choose {
                branches,
                else_body,
            }
            | Stmt::PromptChoose {
                branches,
                else_body: Some(else_body),
            } => {
                for branch in branches {
                    collect_alrim_event_handlers(&branch.body, scope, out);
                }
                collect_alrim_event_handlers(else_body, scope, out);
            }
            Stmt::PromptChoose {
                branches,
                else_body: None,
            } => {
                for branch in branches {
                    collect_alrim_event_handlers(&branch.body, scope, out);
                }
            }
            Stmt::PromptAfter { body, .. }
            | Stmt::PromptCondition { body, .. }
            | Stmt::PromptBlock { body }
            | Stmt::Repeat { body }
            | Stmt::While { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::OpenBlock { body } => {
                collect_alrim_event_handlers(body, scope, out);
            }
            Stmt::RootDecl { .. }
            | Stmt::Assign { .. }
            | Stmt::Show { .. }
            | Stmt::Inspect { .. }
            | Stmt::Send { .. }
            | Stmt::BogaeMadangBlock { .. }
            | Stmt::ExecPolicyBlock { .. }
            | Stmt::JjaimBlock { .. }
            | Stmt::Return { .. }
            | Stmt::Expr { .. }
            | Stmt::BogaeDraw
            | Stmt::Break => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PortRef {
    instance: String,
    port: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct JjaimPortRegistry {
    input_ports: HashSet<String>,
    output_ports: HashSet<String>,
    input_port_types: HashMap<String, String>,
    output_port_types: HashMap<String, String>,
}

impl JjaimPortRegistry {
    fn has_any(&self) -> bool {
        !(self.input_ports.is_empty() && self.output_ports.is_empty())
    }
}

#[derive(Debug, Default, Clone)]
struct JjaimPortSchema {
    by_type: HashMap<String, JjaimPortRegistry>,
}

impl JjaimPortSchema {
    fn registry_for_type(&self, type_name: &str) -> Option<&JjaimPortRegistry> {
        self.by_type.get(type_name)
    }
}

fn build_guseong_flatten_plan_surface(
    stmts: &[SurfaceStmt],
) -> Result<GuseongFlattenPlan, CanonError> {
    let mut declared_instances: HashSet<String> = HashSet::new();
    let mut instance_types: HashMap<String, String> = HashMap::new();
    let mut links: Vec<(PortRef, PortRef)> = Vec::new();
    let mut edges: HashMap<String, HashSet<String>> = HashMap::new();
    let mut sink_sources: HashMap<PortRef, PortRef> = HashMap::new();
    let mut linked_instances: HashSet<String> = HashSet::new();
    let mut nodes: BTreeSet<String> = BTreeSet::new();

    collect_guseong_from_surface_stmts(
        stmts,
        &mut declared_instances,
        &mut instance_types,
        &mut links,
        &mut edges,
        &mut sink_sources,
        &mut linked_instances,
        &mut nodes,
        0,
    )?;

    let port_schema = collect_jjaim_port_schema(stmts, &instance_types)?;

    let mut undeclared: Vec<String> = linked_instances
        .difference(&declared_instances)
        .cloned()
        .collect();
    undeclared.sort();
    if !undeclared.is_empty() {
        return Err(CanonError::new(
            "E_GUSEONG_INSTANCE_UNDECLARED",
            format!(
                "짜임 연결에 선언되지 않은 인스턴스가 있습니다: {}",
                undeclared.join(", ")
            ),
        ));
    }

    validate_guseong_port_links(&links, &instance_types, &port_schema)?;

    let topo_order = topo_sort_or_cycle(&edges, &nodes)?;
    let mut instances: Vec<GuseongFlattenInstance> = declared_instances
        .iter()
        .map(|name| GuseongFlattenInstance {
            name: name.clone(),
            type_name: instance_types
                .get(name)
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".to_string()),
        })
        .collect();
    instances.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.type_name.cmp(&b.type_name))
    });

    let mut plan_links: Vec<GuseongFlattenLink> = links
        .iter()
        .map(|(dst, src)| GuseongFlattenLink {
            dst_instance: dst.instance.clone(),
            dst_port: dst.port.clone(),
            src_instance: src.instance.clone(),
            src_port: src.port.clone(),
        })
        .collect();
    plan_links.sort_by(|a, b| {
        a.dst_instance
            .cmp(&b.dst_instance)
            .then_with(|| a.dst_port.cmp(&b.dst_port))
            .then_with(|| a.src_instance.cmp(&b.src_instance))
            .then_with(|| a.src_port.cmp(&b.src_port))
    });

    Ok(GuseongFlattenPlan {
        topo_order,
        instances,
        links: plan_links,
    })
}

fn collect_guseong_from_surface_stmts(
    stmts: &[SurfaceStmt],
    declared_instances: &mut HashSet<String>,
    instance_types: &mut HashMap<String, String>,
    links: &mut Vec<(PortRef, PortRef)>,
    edges: &mut HashMap<String, HashSet<String>>,
    sink_sources: &mut HashMap<PortRef, PortRef>,
    linked_instances: &mut HashSet<String>,
    nodes: &mut BTreeSet<String>,
    link_scope_depth: usize,
) -> Result<(), CanonError> {
    for stmt in stmts {
        match stmt {
            SurfaceStmt::Assign { target, value } => {
                if let Some(instance_name) = extract_instance_decl_target(target) {
                    if matches!(value, Expr::CallIn { .. }) {
                        if !declared_instances.insert(instance_name.clone()) {
                            return Err(CanonError::new(
                                "E_GUSEONG_INSTANCE_DUP",
                                format!("짜임 인스턴스 이름 중복: {}", instance_name),
                            ));
                        }
                        if let Expr::CallIn { name, .. } = value {
                            instance_types.insert(instance_name.clone(), name.clone());
                        }
                        nodes.insert(instance_name);
                    }
                }

                let Some((dst, src)) = extract_link_from_assign(target, value) else {
                    continue;
                };
                if link_scope_depth > 0 {
                    return Err(CanonError::new(
                        "E_GUSEONG_LINK_TOPLEVEL_ONLY",
                        format!(
                            "짜임 연결은 최상위 문장에서만 허용됩니다: {}.{} <- {}.{}",
                            dst.instance, dst.port, src.instance, src.port
                        ),
                    ));
                }
                links.push((dst.clone(), src.clone()));

                nodes.insert(src.instance.clone());
                nodes.insert(dst.instance.clone());
                linked_instances.insert(src.instance.clone());
                linked_instances.insert(dst.instance.clone());
                edges
                    .entry(src.instance.clone())
                    .or_default()
                    .insert(dst.instance.clone());

                if let Some(existing_src) = sink_sources.get(&dst) {
                    if existing_src != &src {
                        return Err(CanonError::new(
                            "E_GUSEONG_PORT_DIR",
                            format!(
                                "입력 포트 다중 연결: {}.{} <- {}.{} / {}.{}",
                                dst.instance,
                                dst.port,
                                existing_src.instance,
                                existing_src.port,
                                src.instance,
                                src.port
                            ),
                        ));
                    }
                } else {
                    sink_sources.insert(dst, src);
                }
            }
            SurfaceStmt::SeedDef { body, .. } => collect_guseong_from_surface_stmts(
                body,
                declared_instances,
                instance_types,
                links,
                edges,
                sink_sources,
                linked_instances,
                nodes,
                link_scope_depth + 1,
            )?,
            SurfaceStmt::Receive { body, .. } => collect_guseong_from_surface_stmts(
                body,
                declared_instances,
                instance_types,
                links,
                edges,
                sink_sources,
                linked_instances,
                nodes,
                link_scope_depth + 1,
            )?,
            SurfaceStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_guseong_from_surface_stmts(
                    then_body,
                    declared_instances,
                    instance_types,
                    links,
                    edges,
                    sink_sources,
                    linked_instances,
                    nodes,
                    link_scope_depth + 1,
                )?;
                if let Some(body) = else_body {
                    collect_guseong_from_surface_stmts(
                        body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
            }
            SurfaceStmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                if let Some(body) = then_body {
                    collect_guseong_from_surface_stmts(
                        body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
                collect_guseong_from_surface_stmts(
                    else_body,
                    declared_instances,
                    instance_types,
                    links,
                    edges,
                    sink_sources,
                    linked_instances,
                    nodes,
                    link_scope_depth + 1,
                )?;
            }
            SurfaceStmt::Choose {
                branches,
                else_body,
            }
            | SurfaceStmt::PromptChoose {
                branches,
                else_body: Some(else_body),
            } => {
                for branch in branches {
                    collect_guseong_from_surface_stmts(
                        &branch.body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
                collect_guseong_from_surface_stmts(
                    else_body,
                    declared_instances,
                    instance_types,
                    links,
                    edges,
                    sink_sources,
                    linked_instances,
                    nodes,
                    link_scope_depth + 1,
                )?;
            }
            SurfaceStmt::PromptChoose {
                else_body: None,
                branches,
            } => {
                for branch in branches {
                    collect_guseong_from_surface_stmts(
                        &branch.body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
            }
            SurfaceStmt::PromptAfter { body, .. }
            | SurfaceStmt::PromptCondition { body, .. }
            | SurfaceStmt::PromptBlock { body }
            | SurfaceStmt::Repeat { body }
            | SurfaceStmt::While { body, .. }
            | SurfaceStmt::ForEach { body, .. }
            | SurfaceStmt::Hook { body, .. }
            | SurfaceStmt::EventReact { body, .. }
            | SurfaceStmt::OpenBlock { body } => collect_guseong_from_surface_stmts(
                body,
                declared_instances,
                instance_types,
                links,
                edges,
                sink_sources,
                linked_instances,
                nodes,
                link_scope_depth + 1,
            )?,
            SurfaceStmt::RootDecl { .. }
            | SurfaceStmt::Decl { .. }
            | SurfaceStmt::Send { .. }
            | SurfaceStmt::Show { .. }
            | SurfaceStmt::Inspect { .. }
            | SurfaceStmt::BogaeMadangBlock { .. }
            | SurfaceStmt::ExecPolicyBlock { .. }
            | SurfaceStmt::JjaimBlock { .. }
            | SurfaceStmt::BogaeDraw
            | SurfaceStmt::Return { .. }
            | SurfaceStmt::Expr { .. }
            | SurfaceStmt::Break => {}
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct NamedJjaimPortRegistry {
    type_name: Option<String>,
    registry: JjaimPortRegistry,
}

fn collect_jjaim_port_schema(
    stmts: &[SurfaceStmt],
    instance_types: &HashMap<String, String>,
) -> Result<JjaimPortSchema, CanonError> {
    let mut named: Vec<NamedJjaimPortRegistry> = Vec::new();
    collect_jjaim_port_blocks_from_surface(stmts, &mut named)?;

    let mut by_type: HashMap<String, JjaimPortRegistry> = HashMap::new();
    let mut unnamed_merged = JjaimPortRegistry::default();
    for block in named {
        if let Some(type_name) = block.type_name {
            if let Some(existing) = by_type.get(&type_name) {
                if existing != &block.registry {
                    return Err(CanonError::new(
                        "E_JJAIM_TYPE_SCHEMA_CONFLICT",
                        format!(
                            "같은 형식의 포트 스키마가 충돌합니다: {} (기존 입력/출력: {}/{}, 신규 입력/출력: {}/{})",
                            type_name,
                            sorted_ports_text(&existing.input_ports),
                            sorted_ports_text(&existing.output_ports),
                            sorted_ports_text(&block.registry.input_ports),
                            sorted_ports_text(&block.registry.output_ports)
                        ),
                    ));
                }
            } else {
                by_type.insert(type_name, block.registry);
            }
        } else {
            merge_port_registry(&mut unnamed_merged, &block.registry);
        }
    }

    if unnamed_merged.has_any() {
        let unique_types: HashSet<String> = instance_types.values().cloned().collect();
        if unique_types.len() > 1 {
            let mut types: Vec<String> = unique_types.into_iter().collect();
            types.sort();
            return Err(CanonError::new(
                "E_JJAIM_TYPE_TAG_REQUIRED",
                format!(
                    "형식 태그 없는 짜임 포트 스키마는 인스턴스 타입이 1종일 때만 허용됩니다. 현재 타입: {}",
                    types.join(", ")
                ),
            ));
        }
        if unique_types.len() == 1 {
            if let Some(type_name) = unique_types.into_iter().next() {
                let entry = by_type.entry(type_name).or_default();
                merge_port_registry(entry, &unnamed_merged);
            }
        }
    }

    Ok(JjaimPortSchema { by_type })
}

fn collect_jjaim_port_blocks_from_surface(
    stmts: &[SurfaceStmt],
    out: &mut Vec<NamedJjaimPortRegistry>,
) -> Result<(), CanonError> {
    for stmt in stmts {
        match stmt {
            SurfaceStmt::JjaimBlock { body, .. } => out.push(collect_ports_from_jjaim_body(body)?),
            SurfaceStmt::SeedDef { body, .. }
            | SurfaceStmt::PromptAfter { body, .. }
            | SurfaceStmt::PromptCondition { body, .. }
            | SurfaceStmt::PromptBlock { body }
            | SurfaceStmt::Repeat { body }
            | SurfaceStmt::While { body, .. }
            | SurfaceStmt::ForEach { body, .. }
            | SurfaceStmt::Hook { body, .. }
            | SurfaceStmt::Receive { body, .. }
            | SurfaceStmt::EventReact { body, .. }
            | SurfaceStmt::OpenBlock { body } => collect_jjaim_port_blocks_from_surface(body, out)?,
            SurfaceStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_jjaim_port_blocks_from_surface(then_body, out)?;
                if let Some(body) = else_body {
                    collect_jjaim_port_blocks_from_surface(body, out)?;
                }
            }
            SurfaceStmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                if let Some(body) = then_body {
                    collect_jjaim_port_blocks_from_surface(body, out)?;
                }
                collect_jjaim_port_blocks_from_surface(else_body, out)?;
            }
            SurfaceStmt::Choose {
                branches,
                else_body,
            }
            | SurfaceStmt::PromptChoose {
                branches,
                else_body: Some(else_body),
            } => {
                for branch in branches {
                    collect_jjaim_port_blocks_from_surface(&branch.body, out)?;
                }
                collect_jjaim_port_blocks_from_surface(else_body, out)?;
            }
            SurfaceStmt::PromptChoose {
                branches,
                else_body: None,
            } => {
                for branch in branches {
                    collect_jjaim_port_blocks_from_surface(&branch.body, out)?;
                }
            }
            SurfaceStmt::RootDecl { .. }
            | SurfaceStmt::Decl { .. }
            | SurfaceStmt::Assign { .. }
            | SurfaceStmt::Send { .. }
            | SurfaceStmt::Show { .. }
            | SurfaceStmt::Inspect { .. }
            | SurfaceStmt::BogaeMadangBlock { .. }
            | SurfaceStmt::ExecPolicyBlock { .. }
            | SurfaceStmt::BogaeDraw
            | SurfaceStmt::Return { .. }
            | SurfaceStmt::Expr { .. }
            | SurfaceStmt::Break => {}
        }
    }
    Ok(())
}

fn collect_ports_from_jjaim_body(body: &str) -> Result<NamedJjaimPortRegistry, CanonError> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Input,
        Output,
        Formula,
    }
    let mut depth: i32 = 0;
    let mut mode: Option<Mode> = None;
    let mut mode_depth: i32 = 0;
    let mut type_name: Option<String> = None;
    let mut registry = JjaimPortRegistry::default();
    for raw_line in body.lines() {
        let code = strip_line_comment(raw_line);
        let trimmed = code.trim();
        let line_start_depth = depth;

        if line_start_depth == 0 {
            if type_name.is_none() {
                type_name = extract_jjaim_type_tag(trimmed);
            }
            if let Some(open_idx) = trimmed.find('{') {
                let mut head = trimmed[..open_idx].trim();
                if let Some(stripped) = head.strip_suffix(':') {
                    head = stripped.trim_end();
                }
                if head == "입력" {
                    mode = Some(Mode::Input);
                    mode_depth = line_start_depth + 1;
                } else if head == "출력" {
                    mode = Some(Mode::Output);
                    mode_depth = line_start_depth + 1;
                } else if head == "수식" {
                    mode = Some(Mode::Formula);
                    mode_depth = line_start_depth + 1;
                }
            }
        }

        if let Some(active) = mode {
            if line_start_depth == mode_depth {
                match active {
                    Mode::Input | Mode::Output => {
                        if !(trimmed.is_empty() || trimmed == "}" || trimmed.starts_with("}")) {
                            if !contains_assign_operator(trimmed) {
                                return Err(CanonError::new(
                                    "E_JJAIM_PORT_DECL_INVALID",
                                    format!("포트 선언 형식 오류(대입식 필요): {}", trimmed),
                                ));
                            }
                            if has_type_colon_without_type(trimmed) {
                                return Err(CanonError::new(
                                    "E_JJAIM_PORT_TYPE_MISSING",
                                    format!("포트 타입 누락: {}", trimmed),
                                ));
                            }
                            if let Some((name, type_name)) = extract_decl_lhs_name_and_type(trimmed)
                            {
                                match active {
                                    Mode::Input => {
                                        if !registry.input_ports.insert(name.clone()) {
                                            return Err(CanonError::new(
                                                "E_JJAIM_PORT_DUP",
                                                format!("입력 포트 이름 중복: {}", name),
                                            ));
                                        }
                                        if let Some(type_name) = type_name {
                                            registry.input_port_types.insert(name, type_name);
                                        }
                                    }
                                    Mode::Output => {
                                        if !registry.output_ports.insert(name.clone()) {
                                            return Err(CanonError::new(
                                                "E_JJAIM_PORT_DUP",
                                                format!("출력 포트 이름 중복: {}", name),
                                            ));
                                        }
                                        if let Some(type_name) = type_name {
                                            registry.output_port_types.insert(name, type_name);
                                        }
                                    }
                                    Mode::Formula => {}
                                }
                            } else {
                                return Err(CanonError::new(
                                    "E_JJAIM_PORT_DECL_INVALID",
                                    format!("포트 선언에서 이름을 해석할 수 없습니다: {}", trimmed),
                                ));
                            }
                        }
                    }
                    Mode::Formula => {
                        // 수식 블록은 타입 명시된 항목만 외부 출력 포트로 간주한다.
                        if let Some((name, type_name)) =
                            extract_typed_decl_lhs_name_and_type(trimmed)
                        {
                            registry.output_ports.insert(name.clone());
                            registry.output_port_types.insert(name, type_name);
                        }
                    }
                }
            }
        }

        for ch in code.chars() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
            }
        }
        if mode.is_some() && depth < mode_depth {
            mode = None;
            mode_depth = 0;
        }
    }
    Ok(NamedJjaimPortRegistry {
        type_name,
        registry,
    })
}

fn merge_port_registry(dst: &mut JjaimPortRegistry, src: &JjaimPortRegistry) {
    dst.input_ports.extend(src.input_ports.iter().cloned());
    dst.output_ports.extend(src.output_ports.iter().cloned());
    for (name, type_name) in &src.input_port_types {
        dst.input_port_types
            .entry(name.clone())
            .or_insert_with(|| type_name.clone());
    }
    for (name, type_name) in &src.output_port_types {
        dst.output_port_types
            .entry(name.clone())
            .or_insert_with(|| type_name.clone());
    }
}

fn validate_guseong_port_links(
    links: &[(PortRef, PortRef)],
    instance_types: &HashMap<String, String>,
    port_schema: &JjaimPortSchema,
) -> Result<(), CanonError> {
    for (dst, src) in links {
        validate_tuple_port_projection(&dst.instance, &dst.port, "입력")?;
        validate_tuple_port_projection(&src.instance, &src.port, "출력")?;
        if let Some(dst_type) = instance_types.get(&dst.instance) {
            if let Some(registry) = port_schema.registry_for_type(dst_type) {
                let dst_base = base_port_name(&dst.port);
                if !registry.input_ports.contains(dst_base) {
                    return Err(CanonError::new(
                        "E_GUSEONG_INPUT_PORT_UNDECLARED",
                        format!(
                            "선언되지 않은 입력 포트입니다: {}.{} (타입 {}, 기대 입력 포트: {})",
                            dst.instance,
                            dst.port,
                            dst_type,
                            sorted_ports_text(&registry.input_ports)
                        ),
                    ));
                }
                validate_tuple_projection_type(
                    &dst.instance,
                    &dst.port,
                    "입력",
                    registry.input_port_types.get(dst_base),
                )?;
            }
        }
        if let Some(src_type) = instance_types.get(&src.instance) {
            if let Some(registry) = port_schema.registry_for_type(src_type) {
                let src_base = base_port_name(&src.port);
                if !registry.output_ports.contains(src_base) {
                    return Err(CanonError::new(
                        "E_GUSEONG_OUTPUT_PORT_UNDECLARED",
                        format!(
                            "선언되지 않은 출력 포트입니다: {}.{} (타입 {}, 기대 출력 포트: {})",
                            src.instance,
                            src.port,
                            src_type,
                            sorted_ports_text(&registry.output_ports)
                        ),
                    ));
                }
                validate_tuple_projection_type(
                    &src.instance,
                    &src.port,
                    "출력",
                    registry.output_port_types.get(src_base),
                )?;
            }
        }
    }
    Ok(())
}

fn validate_tuple_port_projection(
    instance: &str,
    port: &str,
    direction: &str,
) -> Result<(), CanonError> {
    let mut parts = port.split('.');
    let _base = parts.next().unwrap_or(port);
    for suffix in parts {
        if suffix != "0" && suffix != "1" {
            return Err(CanonError::new(
                "E_GUSEONG_TUPLE_INDEX_INVALID",
                format!(
                    "튜플 포트 접근은 .0/.1만 허용됩니다: {}.{} ({} 접근)",
                    instance, port, direction
                ),
            ));
        }
    }
    Ok(())
}

fn validate_tuple_projection_type(
    instance: &str,
    port: &str,
    direction: &str,
    declared_type: Option<&String>,
) -> Result<(), CanonError> {
    if !port.contains('.') {
        return Ok(());
    }
    let Some(type_name) = declared_type else {
        return Ok(());
    };
    if !is_tuple_type_name(type_name) {
        return Err(CanonError::new(
            "E_GUSEONG_TUPLE_ACCESS_ON_SCALAR",
            format!(
                "스칼라 포트에는 튜플 인덱스 접근을 사용할 수 없습니다: {}.{} ({} 포트 타입: {})",
                instance, port, direction, type_name
            ),
        ));
    }
    Ok(())
}

fn is_tuple_type_name(type_name: &str) -> bool {
    let trimmed = type_name.trim();
    trimmed.starts_with('(') && trimmed.ends_with(')') && trimmed.contains(',')
}

fn strip_line_comment(raw: &str) -> &str {
    raw.split_once("//").map(|(head, _)| head).unwrap_or(raw)
}

fn contains_assign_operator(line: &str) -> bool {
    line.contains("<-") || line.contains('=')
}

fn has_type_colon_without_type(line: &str) -> bool {
    let op_idx = line.find("<-").or_else(|| line.find('='));
    let Some(op_idx) = op_idx else {
        return false;
    };
    let lhs = line[..op_idx].trim();
    let Some((_, type_name)) = lhs.split_once(':') else {
        return false;
    };
    type_name.trim().is_empty()
}

fn extract_jjaim_type_tag(line: &str) -> Option<String> {
    let head = line.trim();
    let (key, rhs) = head.split_once(':')?;
    let key = key.trim();
    if key != "형식" && key != "짜임이름" {
        return None;
    }
    let rhs = rhs.trim().trim_end_matches('.').trim();
    if rhs.is_empty() {
        return None;
    }
    let mut out = String::new();
    for ch in rhs.chars() {
        if is_ident_continue(ch) {
            out.push(ch);
        } else {
            break;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn extract_decl_lhs_name_and_type(line: &str) -> Option<(String, Option<String>)> {
    if line.is_empty() || line.starts_with('{') || line.starts_with('}') {
        return None;
    }
    let op_idx = line.find("<-").or_else(|| line.find('='))?;
    let lhs = line[..op_idx].trim();
    if lhs.is_empty() {
        return None;
    }
    if let Some((name, type_name)) = lhs.split_once(':') {
        let parsed_name = parse_ident_head(name.trim())?;
        let type_name = type_name.trim();
        if type_name.is_empty() {
            return Some((parsed_name, None));
        }
        return Some((parsed_name, Some(type_name.to_string())));
    }
    parse_ident_head(lhs).map(|name| (name, None))
}

fn extract_typed_decl_lhs_name_and_type(line: &str) -> Option<(String, String)> {
    if line.is_empty() || line.starts_with('{') || line.starts_with('}') {
        return None;
    }
    let op_idx = line.find("<-").or_else(|| line.find('='))?;
    let lhs = line[..op_idx].trim();
    let (name, type_name) = lhs.split_once(':')?;
    let parsed_name = parse_ident_head(name.trim())?;
    let parsed_type = type_name.trim();
    if parsed_type.is_empty() {
        return None;
    }
    Some((parsed_name, parsed_type.to_string()))
}

fn parse_ident_head(raw: &str) -> Option<String> {
    if raw.is_empty() {
        return None;
    }
    let mut out = String::new();
    for ch in raw.chars() {
        if is_ident_continue(ch) {
            out.push(ch);
        } else {
            break;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn base_port_name(port: &str) -> &str {
    port.split('.').next().unwrap_or(port)
}

fn sorted_ports_text(ports: &HashSet<String>) -> String {
    let mut values: Vec<String> = ports.iter().cloned().collect();
    values.sort();
    if values.is_empty() {
        "(없음)".to_string()
    } else {
        values.join(", ")
    }
}

fn extract_instance_decl_target(path: &Path) -> Option<String> {
    let segments = normalize_root_segments(&path.segments);
    (segments.len() == 1).then(|| segments[0].clone())
}

fn extract_port_ref_from_path(path: &Path) -> Option<PortRef> {
    let segments = normalize_root_segments(&path.segments);
    if segments.len() < 2 {
        return None;
    }
    Some(PortRef {
        instance: segments[0].clone(),
        port: segments[1..].join("."),
    })
}

fn extract_port_ref_from_expr(expr: &Expr) -> Option<PortRef> {
    match expr {
        Expr::Path(path) => extract_port_ref_from_path(path),
        Expr::FieldAccess { target, field } => {
            let path = flatten_field_access(target, field)?;
            extract_port_ref_from_path(&path)
        }
        _ => None,
    }
}

fn extract_link_from_assign(target: &Path, value: &Expr) -> Option<(PortRef, PortRef)> {
    let direct_dst = extract_port_ref_from_path(target);
    let direct_src = extract_port_ref_from_expr(value);
    if let (Some(dst), Some(src)) = (direct_dst, direct_src) {
        return Some((dst, src));
    }

    let Expr::Call { name, args } = value else {
        return None;
    };
    if name != "짝맞춤.바꾼값" || args.len() != 3 {
        return None;
    }

    let Expr::Path(map_path) = &args[0] else {
        return None;
    };
    let map_instance = extract_instance_from_path(map_path)?;
    let Expr::Literal(Literal::Str(port_name)) = &args[1] else {
        return None;
    };
    let src = extract_port_ref_from_expr(&args[2])?;
    let dst = PortRef {
        instance: map_instance,
        port: port_name.clone(),
    };
    Some((dst, src))
}

fn extract_instance_from_path(path: &Path) -> Option<String> {
    let segments = normalize_root_segments(&path.segments);
    (!segments.is_empty()).then(|| segments[0].clone())
}

fn is_event_noun_canonical(text: &str) -> bool {
    matches!(text, "알림" | "알림이" | "알림가")
}

fn is_event_noun_alias(text: &str) -> bool {
    matches!(
        text,
        "소식" | "소식이" | "소식가" | "알람" | "알람이" | "알람가"
    )
}

fn is_event_noun_any(text: &str) -> bool {
    is_event_noun_canonical(text) || is_event_noun_alias(text)
}

fn strip_receive_object_particle(text: &str) -> Option<String> {
    text.strip_suffix('를')
        .or_else(|| text.strip_suffix('을'))
        .map(|raw| raw.to_string())
}

fn object_particle(text: &str) -> &'static str {
    let Some(last) = text.chars().last() else {
        return "를";
    };
    if !('가'..='힣').contains(&last) {
        return "를";
    }
    let code = last as u32 - '가' as u32;
    if code % 28 == 0 {
        "를"
    } else {
        "을"
    }
}

fn is_open_block_keyword(name: &str) -> bool {
    matches!(name, "열림" | "너머" | "효과" | "바깥")
}

fn flatten_field_access(target: &Expr, field: &str) -> Option<Path> {
    let mut segments = match target {
        Expr::Path(path) => path.segments.clone(),
        Expr::FieldAccess {
            target: inner,
            field: inner_field,
        } => {
            let inner_path = flatten_field_access(inner, inner_field)?;
            inner_path.segments
        }
        _ => return None,
    };
    segments.push(field.to_string());
    Some(Path { segments })
}

fn normalize_root_segments(segments: &[String]) -> Vec<String> {
    if segments.len() >= 2 && is_root_segment(&segments[0]) {
        return segments[1..].to_vec();
    }
    segments.to_vec()
}

fn is_root_segment(name: &str) -> bool {
    matches!(name, "바탕" | "살림" | "샘")
}

fn topo_sort_or_cycle(
    edges: &HashMap<String, HashSet<String>>,
    nodes: &BTreeSet<String>,
) -> Result<Vec<String>, CanonError> {
    let mut indegree: HashMap<String, usize> = nodes.iter().map(|n| (n.clone(), 0usize)).collect();
    for tos in edges.values() {
        for to in tos {
            *indegree.entry(to.clone()).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<String> = indegree
        .iter()
        .filter_map(|(name, deg)| (*deg == 0).then(|| name.clone()))
        .collect();
    let mut queue_vec: Vec<String> = queue.drain(..).collect();
    queue_vec.sort();
    queue = queue_vec.into_iter().collect();

    let mut out = Vec::new();
    let mut indegree_mut = indegree;
    while let Some(node) = queue.pop_front() {
        out.push(node.clone());
        if let Some(tos) = edges.get(&node) {
            let mut sorted_tos: Vec<String> = tos.iter().cloned().collect();
            sorted_tos.sort();
            for to in sorted_tos {
                if let Some(deg) = indegree_mut.get_mut(&to) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push_back(to);
                    }
                }
            }
        }
    }

    if out.len() == indegree_mut.len() {
        return Ok(out);
    }

    let cycle_nodes = detect_cycle_nodes(edges, nodes);
    let cycle_text = if cycle_nodes.is_empty() {
        "알 수 없는 순환".to_string()
    } else {
        cycle_nodes.join(" -> ")
    };
    Err(CanonError::new(
        "E_GUSEONG_LINK_CYCLE",
        format!("짜임 연결 순환: {}", cycle_text),
    ))
}

fn detect_cycle_nodes(
    edges: &HashMap<String, HashSet<String>>,
    nodes: &BTreeSet<String>,
) -> Vec<String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Temp,
        Perm,
    }
    fn dfs(
        node: &str,
        edges: &HashMap<String, HashSet<String>>,
        marks: &mut HashMap<String, Mark>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        marks.insert(node.to_string(), Mark::Temp);
        stack.push(node.to_string());
        if let Some(tos) = edges.get(node) {
            let mut nexts: Vec<String> = tos.iter().cloned().collect();
            nexts.sort();
            for to in nexts {
                match marks.get(&to).copied() {
                    Some(Mark::Temp) => {
                        if let Some(pos) = stack.iter().position(|s| s == &to) {
                            let mut cycle = stack[pos..].to_vec();
                            cycle.push(to);
                            return Some(cycle);
                        }
                    }
                    Some(Mark::Perm) => {}
                    None => {
                        if let Some(cycle) = dfs(&to, edges, marks, stack) {
                            return Some(cycle);
                        }
                    }
                }
            }
        }
        stack.pop();
        marks.insert(node.to_string(), Mark::Perm);
        None
    }

    let mut marks: HashMap<String, Mark> = HashMap::new();
    let mut stack = Vec::new();
    for node in nodes {
        if marks.contains_key(node) {
            continue;
        }
        if let Some(cycle) = dfs(node, edges, &mut marks, &mut stack) {
            return cycle;
        }
    }
    Vec::new()
}

fn canonicalize_expr(
    expr: Expr,
    declared: &HashSet<String>,
    bridge: bool,
    default_root: &str,
    root_hide: bool,
) -> Expr {
    match expr {
        Expr::Path(path) => Expr::Path(canonicalize_path(
            path,
            declared,
            bridge,
            default_root,
            root_hide,
        )),
        Expr::Literal(literal) => Expr::Literal(literal),
        Expr::Resource(path) => Expr::Resource(path),
        Expr::Template { body } => Expr::Template { body },
        Expr::PromptExpr { expr } => Expr::PromptExpr {
            expr: Box::new(canonicalize_expr(
                *expr,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
        Expr::PromptBlock { body } => Expr::PromptBlock { body },
        Expr::TemplateApply { bindings, body } => Expr::TemplateApply {
            bindings: bindings
                .into_iter()
                .map(|binding| Binding {
                    name: binding.name,
                    value: canonicalize_expr(
                        binding.value,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                })
                .collect(),
            body,
        },
        Expr::Formula { tag, body } => Expr::Formula { tag, body },
        Expr::Call { name, args } => Expr::Call {
            name: canonicalize_stdlib_alias(&name).to_string(),
            args: args
                .into_iter()
                .map(|arg| canonicalize_expr(arg, declared, bridge, default_root, root_hide))
                .collect(),
        },
        Expr::CallIn { name, bindings } => Expr::CallIn {
            name: canonicalize_stdlib_alias(&name).to_string(),
            bindings: bindings
                .into_iter()
                .map(|binding| Binding {
                    name: binding.name,
                    value: canonicalize_expr(
                        binding.value,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                })
                .collect(),
        },
        Expr::FieldAccess { target, field } => Expr::FieldAccess {
            target: Box::new(canonicalize_expr(
                *target,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
            field,
        },
        Expr::Pack { bindings } => Expr::Pack {
            bindings: bindings
                .into_iter()
                .map(|binding| Binding {
                    name: binding.name,
                    value: canonicalize_expr(
                        binding.value,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                })
                .collect(),
        },
        Expr::Unit { value, unit } => Expr::Unit {
            value: Box::new(canonicalize_expr(
                *value,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
            unit,
        },
        Expr::Unary { op, expr } => Expr::Unary {
            op,
            expr: Box::new(canonicalize_expr(
                *expr,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
        Expr::Binary { left, op, right } => Expr::Binary {
            left: Box::new(canonicalize_expr(
                *left,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
            op,
            right: Box::new(canonicalize_expr(
                *right,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
        Expr::Pipe { left, kind, right } => Expr::Pipe {
            left: Box::new(canonicalize_expr(
                *left,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
            kind,
            right: Box::new(canonicalize_expr(
                *right,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
        Expr::SeedLiteral { param, body } => Expr::SeedLiteral {
            param,
            body: Box::new(canonicalize_expr(
                *body,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
    }
}

fn canonicalize_stdlib_alias(name: &str) -> &str {
    ddonirang_lang::stdlib::canonicalize_stdlib_alias(name)
}

fn canonicalize_path(
    path: Path,
    declared: &HashSet<String>,
    bridge: bool,
    default_root: &str,
    root_hide: bool,
) -> Path {
    let mut segments = path.segments;
    let has_root = matches!(
        segments.first().map(|seg| seg.as_str()),
        Some("살림" | "바탕" | "샘")
    );
    if let Some(root) = segments.first_mut() {
        if root == "살림" {
            *root = "바탕".to_string();
        }
    }
    if !has_root {
        if root_hide {
            segments.insert(0, default_root.to_string());
        } else if bridge && segments.len() == 1 {
            let name = &segments[0];
            if !matches!(name.as_str(), "살림" | "바탕") && declared.contains(name) {
                segments.insert(0, default_root.to_string());
            }
        }
    }
    Path { segments }
}

fn canonicalize_condition(
    condition: Condition,
    declared: &HashSet<String>,
    bridge: bool,
    default_root: &str,
    root_hide: bool,
) -> Condition {
    Condition {
        expr: canonicalize_expr(condition.expr, declared, bridge, default_root, root_hide),
        style: condition.style,
        negated: condition.negated,
    }
}

fn canonicalize_body(
    body: Vec<SurfaceStmt>,
    declared: &HashSet<String>,
    bridge: bool,
    default_root: &str,
    root_hide: bool,
) -> Result<Vec<Stmt>, CanonError> {
    let mut out = Vec::new();
    for stmt in body {
        match stmt {
            SurfaceStmt::RootDecl { items } => {
                let items = items
                    .into_iter()
                    .map(|item| DeclItem {
                        name: item.name,
                        kind: item.kind,
                        type_name: item.type_name,
                        value: item.value.map(|expr| {
                            canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                        }),
                        maegim: item.maegim.map(|spec| MaegimSpec {
                            fields: spec
                                .fields
                                .into_iter()
                                .map(|binding| Binding {
                                    name: binding.name,
                                    value: canonicalize_expr(
                                        binding.value,
                                        declared,
                                        bridge,
                                        default_root,
                                        root_hide,
                                    ),
                                })
                                .collect(),
                        }),
                    })
                    .collect();
                out.push(Stmt::RootDecl { items });
            }
            SurfaceStmt::Decl {
                name,
                type_name,
                value,
            } => {
                if !is_bridge_alias_type_name(&type_name) {
                    return Err(CanonError::new(
                        "E_CANON_BAD_ALIAS",
                        format!("알 수 없는 별칭입니다: {}", type_name),
                    ));
                }
                let target = Path {
                    segments: vec![default_root.to_string(), name],
                };
                let value = canonicalize_expr(value, declared, bridge, default_root, root_hide);
                out.push(Stmt::Assign { target, value });
            }
            SurfaceStmt::SeedDef {
                name,
                kind,
                params,
                body,
            } => {
                let params = params
                    .into_iter()
                    .map(|param| Param {
                        name: param.name,
                        type_name: param.type_name,
                        default: param.default.map(|expr| {
                            canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                        }),
                    })
                    .collect();
                let body = canonicalize_body(body, declared, bridge, default_root, root_hide)?;
                out.push(Stmt::SeedDef {
                    name,
                    kind,
                    params,
                    body,
                });
            }
            SurfaceStmt::Send {
                sender,
                payload,
                receiver,
            } => {
                out.push(Stmt::Send {
                    sender: sender.map(|expr| {
                        canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                    }),
                    payload: canonicalize_expr(payload, declared, bridge, default_root, root_hide),
                    receiver: canonicalize_expr(
                        receiver,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                });
            }
            SurfaceStmt::Assign { target, value } => {
                if root_hide
                    && !matches!(
                        target.segments.first().map(|seg| seg.as_str()),
                        Some("살림" | "바탕" | "샘")
                    )
                {
                    if let Some(name) = target.segments.first() {
                        if !declared.contains(name) {
                            return Err(CanonError::new(
                                "E_CANON_ROOT_HIDE_UNDECLARED",
                                format!("바탕숨김에서 미등록 바탕칸 쓰기: {}", name),
                            ));
                        }
                    }
                }
                out.push(Stmt::Assign {
                    target: canonicalize_path(target, declared, bridge, default_root, root_hide),
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::Show { value } => {
                out.push(Stmt::Show {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::Inspect { value } => {
                out.push(Stmt::Inspect {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::BogaeMadangBlock { body, .. } => {
                out.push(Stmt::BogaeMadangBlock { body });
            }
            SurfaceStmt::JjaimBlock { body, .. } => {
                out.push(Stmt::JjaimBlock { body });
            }
            SurfaceStmt::BogaeDraw => {
                out.push(Stmt::BogaeDraw);
            }
            SurfaceStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let then_body =
                    canonicalize_body(then_body, declared, bridge, default_root, root_hide)?;
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                out.push(Stmt::If {
                    condition: canonicalize_condition(
                        condition,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    then_body,
                    else_body,
                });
            }
            SurfaceStmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
            } => {
                let else_body =
                    canonicalize_body(else_body, declared, bridge, default_root, root_hide)?;
                let then_body = match then_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                out.push(Stmt::Contract {
                    kind,
                    mode,
                    condition: canonicalize_condition(
                        condition,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    then_body,
                    else_body,
                });
            }
            SurfaceStmt::Choose {
                branches,
                else_body,
            } => {
                let mut mapped = Vec::new();
                for branch in branches {
                    mapped.push(ChooseBranch {
                        condition: canonicalize_condition(
                            branch.condition,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                        body: canonicalize_body(
                            branch.body,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        )?,
                    });
                }
                out.push(Stmt::Choose {
                    branches: mapped,
                    else_body: canonicalize_body(
                        else_body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?,
                });
            }
            SurfaceStmt::PromptChoose {
                branches,
                else_body,
            } => {
                let mut mapped = Vec::new();
                for branch in branches {
                    mapped.push(ChooseBranch {
                        condition: canonicalize_condition(
                            branch.condition,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                        body: canonicalize_body(
                            branch.body,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        )?,
                    });
                }
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                out.push(Stmt::PromptChoose {
                    branches: mapped,
                    else_body,
                });
            }
            SurfaceStmt::PromptAfter { value, body } => {
                out.push(Stmt::PromptAfter {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptCondition { condition, body } => {
                out.push(Stmt::PromptCondition {
                    condition: canonicalize_condition(
                        condition,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptBlock { body } => {
                out.push(Stmt::PromptBlock {
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::ExecPolicyBlock { body } => {
                out.push(Stmt::ExecPolicyBlock { body });
            }
            SurfaceStmt::Repeat { body } => {
                out.push(Stmt::Repeat {
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::While { condition, body } => {
                out.push(Stmt::While {
                    condition: canonicalize_condition(
                        condition,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::ForEach {
                item,
                iterable,
                body,
            } => {
                out.push(Stmt::ForEach {
                    item,
                    iterable: canonicalize_expr(
                        iterable,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::Hook { name, suffix, body } => {
                out.push(Stmt::Hook {
                    name,
                    suffix,
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::Receive {
                kind,
                binding,
                condition,
                body,
            } => {
                out.push(Stmt::Receive {
                    kind,
                    binding,
                    condition: condition.map(|expr| {
                        canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                    }),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::EventReact { kind, body } => {
                out.push(Stmt::EventReact {
                    kind,
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::OpenBlock { body } => {
                out.push(Stmt::OpenBlock {
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::Return { value } => {
                out.push(Stmt::Return {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::Expr { value } => {
                out.push(Stmt::Expr {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::Break => {
                out.push(Stmt::Break);
            }
        }
    }
    Ok(out)
}

fn format_program(stmts: &[Stmt]) -> String {
    let mut out = String::new();
    for stmt in stmts {
        format_stmt(stmt, 0, &mut out);
    }
    out
}

fn format_stmt(stmt: &Stmt, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    match stmt {
        Stmt::RootDecl { items, .. } => {
            out.push_str(&format!("{}채비 {{\n", pad));
            for item in items {
                out.push_str(&format_decl_item(item, indent + 1));
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Assign { target, value } => {
            out.push_str(&format!(
                "{}{} <- {}.\n",
                pad,
                format_path(target),
                format_expr(value)
            ));
        }
        Stmt::SeedDef {
            name,
            kind,
            params,
            body,
        } => {
            let params_text = format_params(params);
            out.push_str(&format!("{}{}{}:{} = {{\n", pad, params_text, name, kind));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Show { value } => {
            out.push_str(&format!("{}{} 보여주기.\n", pad, format_expr(value)));
        }
        Stmt::Inspect { value } => {
            out.push_str(&format!("{}{} 톺아보기.\n", pad, format_expr(value)));
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
        } => {
            if let Some(sender) = sender {
                out.push_str(&format!(
                    "{}({})의 {} ~~> {}.\n",
                    pad,
                    format_expr(sender),
                    format_expr(payload),
                    format_expr(receiver)
                ));
            } else {
                out.push_str(&format!(
                    "{}{} ~~> {}.\n",
                    pad,
                    format_expr(payload),
                    format_expr(receiver)
                ));
            }
        }
        Stmt::BogaeMadangBlock { body } => {
            out.push_str(&format!("{}보개마당 {{", pad));
            out.push_str(body);
            out.push_str("}.\n");
        }
        Stmt::ExecPolicyBlock { body } => {
            out.push_str(&format!("{}실행정책 {{", pad));
            out.push_str(body);
            out.push_str("}.\n");
        }
        Stmt::JjaimBlock { body } => {
            out.push_str(&format!("{}짜임 {{", pad));
            out.push_str(body);
            out.push_str("}.\n");
        }
        Stmt::Return { value } => {
            out.push_str(&format!("{}{} 되돌림.\n", pad, format_expr(value)));
        }
        Stmt::Expr { value } => {
            out.push_str(&format!("{}{}.\n", pad, format_expr(value)));
        }
        Stmt::BogaeDraw => {
            out.push_str(&format!("{}보개로 그려.\n", pad));
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            out.push_str(&format!("{}{} 일때 {{\n", pad, format_condition(condition)));
            for stmt in then_body {
                format_stmt(stmt, indent + 1, out);
            }
            if let Some(body) = else_body {
                out.push_str(&format!("{}}} 아니면 {{\n", pad));
                for stmt in body {
                    format_stmt(stmt, indent + 1, out);
                }
                out.push_str(&format!("{}}}.\n", pad));
            } else {
                out.push_str(&format!("{}}}.\n", pad));
            }
        }
        Stmt::Contract {
            kind,
            mode,
            condition,
            then_body,
            else_body,
        } => {
            let keyword = match kind {
                ContractKind::Pre => "바탕으로",
                ContractKind::Post => "다짐하고",
            };
            let mode_suffix = match mode {
                ContractMode::Alert => "(알림)",
                ContractMode::Abort => "",
            };
            out.push_str(&format!(
                "{}{} {}{} 아니면 {{\n",
                pad,
                format_condition(condition),
                keyword,
                mode_suffix
            ));
            for stmt in else_body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}", pad));
            if let Some(body) = then_body {
                out.push_str(" 맞으면 {\n");
                for stmt in body {
                    format_stmt(stmt, indent + 1, out);
                }
                out.push_str(&format!("{}}}", pad));
            }
            out.push_str(".\n");
        }
        Stmt::Choose {
            branches,
            else_body,
        } => {
            out.push_str(&format!("{}고르기:\n", pad));
            for branch in branches {
                out.push_str(&format!(
                    "{}  {}: {{\n",
                    pad,
                    format_condition(&branch.condition)
                ));
                for stmt in &branch.body {
                    format_stmt(stmt, indent + 2, out);
                }
                out.push_str(&format!("{}  }}\n", pad));
            }
            out.push_str(&format!("{}  아니면: {{\n", pad));
            for stmt in else_body {
                format_stmt(stmt, indent + 2, out);
            }
            out.push_str(&format!("{}  }}.\n", pad));
        }
        Stmt::PromptChoose {
            branches,
            else_body,
        } => {
            out.push_str(&format!("{}??:\n", pad));
            for branch in branches {
                out.push_str(&format!(
                    "{}  {}: {{\n",
                    pad,
                    format_condition(&branch.condition)
                ));
                for stmt in &branch.body {
                    format_stmt(stmt, indent + 2, out);
                }
                out.push_str(&format!("{}  }}\n", pad));
            }
            if let Some(else_body) = else_body {
                out.push_str(&format!("{}  ??: {{\n", pad));
                for stmt in else_body {
                    format_stmt(stmt, indent + 2, out);
                }
                out.push_str(&format!("{}  }}\n", pad));
            }
        }
        Stmt::PromptAfter { value, body } => {
            out.push_str(&format!("{}{} ??: {{\n", pad, format_expr(value)));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::PromptCondition { condition, body } => {
            out.push_str(&format!("{}{} ??\n", pad, format_condition(condition)));
            out.push_str(&format!("{}?? {{\n", pad));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::PromptBlock { body } => {
            out.push_str(&format!("{}?? {{\n", pad));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::Repeat { body } => {
            out.push_str(&format!("{}되풀이 {{\n", pad));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::While { condition, body } => {
            out.push_str(&format!("{}{} 동안 {{\n", pad, format_condition(condition)));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::ForEach {
            item,
            iterable,
            body,
        } => {
            out.push_str(&format!(
                "{}({}) {}에 대해 {{\n",
                pad,
                item,
                format_expr(iterable)
            ));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Hook { name, suffix, body } => {
            let suffix_text = match suffix {
                HookSuffix::Halttae => "할때",
                HookSuffix::Mada => "마다",
            };
            out.push_str(&format!("{}({}){} {{\n", pad, name, suffix_text));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Receive {
            kind,
            binding,
            condition,
            body,
        } => {
            out.push_str(&pad);
            if let Some(binding) = binding {
                out.push('(');
                out.push_str(binding);
                if let Some(condition) = condition {
                    out.push(' ');
                    out.push_str(&format_expr(condition));
                }
                out.push_str(")인 ");
            }
            if let Some(kind) = kind {
                out.push_str(kind);
                out.push_str(object_particle(kind));
            } else {
                out.push_str("알림을");
            }
            out.push_str(" 받으면 {\n");
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::EventReact { kind, body } => {
            out.push_str(&format!(
                "{}\"{}\"라는 알림이 오면 {{\n",
                pad,
                escape_string(kind)
            ));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::OpenBlock { body } => {
            out.push_str(&format!("{}너머 {{\n", pad));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Break => {
            out.push_str(&format!("{}멈추기.\n", pad));
        }
    }
}

fn format_decl_item(item: &DeclItem, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let mut out = String::new();
    out.push_str(&pad);
    out.push_str(&item.name);
    out.push(':');
    out.push_str(&item.type_name);
    if let Some(value) = &item.value {
        if item.kind == DeclKind::Butbak {
            out.push_str(" = ");
        } else {
            out.push_str(" <- ");
        }
        if let Some(maegim) = &item.maegim {
            out.push('(');
            out.push_str(&format_expr(value));
            out.push_str(") 매김 {\n");
            let mut emitted_ganeum = false;
            let mut emitted_gallae = false;
            for field in &maegim.fields {
                if field.name.starts_with("가늠.") {
                    if emitted_ganeum {
                        continue;
                    }
                    emitted_ganeum = true;
                    out.push_str(&pad);
                    out.push_str("  가늠 {\n");
                    for nested in &maegim.fields {
                        let Some(leaf) = nested.name.strip_prefix("가늠.") else {
                            continue;
                        };
                        out.push_str(&pad);
                        out.push_str("    ");
                        out.push_str(leaf);
                        out.push_str(": ");
                        out.push_str(&format_maegim_field_value(nested));
                        out.push_str(".\n");
                    }
                    out.push_str(&pad);
                    out.push_str("  }.\n");
                    continue;
                }
                if field.name.starts_with("갈래.") {
                    if emitted_gallae {
                        continue;
                    }
                    emitted_gallae = true;
                    out.push_str(&pad);
                    out.push_str("  갈래 {\n");
                    for nested in &maegim.fields {
                        let Some(leaf) = nested.name.strip_prefix("갈래.") else {
                            continue;
                        };
                        out.push_str(&pad);
                        out.push_str("    ");
                        out.push_str(leaf);
                        out.push_str(": ");
                        out.push_str(&format_maegim_field_value(nested));
                        out.push_str(".\n");
                    }
                    out.push_str(&pad);
                    out.push_str("  }.\n");
                    continue;
                }
                out.push_str(&pad);
                out.push_str("  ");
                out.push_str(&field.name);
                out.push_str(": ");
                out.push_str(&format_maegim_field_value(field));
                out.push_str(".\n");
            }
            out.push_str(&pad);
            out.push('}');
        } else {
            out.push_str(&format_expr(value));
        }
    }
    out.push_str(".\n");
    out
}

fn format_expr(expr: &Expr) -> String {
    format_expr_prec(expr, 0)
}

fn maegim_field_leaf_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

fn format_maegim_field_value(binding: &Binding) -> String {
    if maegim_field_leaf_name(&binding.name) == "범위" {
        if let Some(rendered) = try_format_range_expr(&binding.value) {
            return rendered;
        }
    }
    format_expr(&binding.value)
}

fn try_format_range_expr(expr: &Expr) -> Option<String> {
    let Expr::Call { name, args, .. } = expr else {
        return None;
    };
    if name != "표준.범위" || args.len() != 3 {
        return None;
    }
    let left = format_expr(&args[0]);
    let right = format_expr(&args[1]);
    let inclusive = matches!(&args[2], Expr::Literal(Literal::Num(value)) if value == "1");
    Some(format!(
        "{}{}{}",
        left,
        if inclusive { "..=" } else { ".." },
        right
    ))
}

fn format_params(params: &[Param]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push('(');
    for (idx, param) in params.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&param.name);
        if let Some(type_name) = &param.type_name {
            out.push(':');
            out.push_str(type_name);
        }
        if let Some(default) = &param.default {
            out.push_str(" = ");
            out.push_str(&format_expr(default));
        }
    }
    out.push(')');
    out.push(' ');
    out
}

fn format_expr_prec(expr: &Expr, parent_prec: u8) -> String {
    match expr {
        Expr::Literal(literal) => format_literal(literal),
        Expr::Resource(path) => format!("@\"{}\"", escape_string(path)),
        Expr::Template { body } => format!("글무늬{{{}}}", body),
        Expr::PromptExpr { expr } => format!("??({})", format_expr(expr)),
        Expr::PromptBlock { body } => format!("??{{{}}}", body),
        Expr::TemplateApply { bindings, body } => {
            let mut rendered = String::new();
            rendered.push('(');
            for (idx, binding) in bindings.iter().enumerate() {
                if idx > 0 {
                    rendered.push_str(", ");
                }
                rendered.push_str(&binding.name);
                rendered.push('=');
                rendered.push_str(&format_expr(&binding.value));
            }
            rendered.push(')');
            rendered.push(' ');
            rendered.push_str("글무늬");
            rendered.push('{');
            rendered.push_str(body);
            rendered.push('}');
            rendered
        }
        Expr::Formula { tag, body } => {
            let mut rendered = String::new();
            if let Some(tag) = tag {
                rendered.push('(');
                rendered.push('#');
                rendered.push_str(tag);
                rendered.push_str(") ");
            }
            rendered.push_str("수식");
            rendered.push('{');
            rendered.push_str(body);
            rendered.push('}');
            rendered
        }
        Expr::Path(path) => format_path(path),
        Expr::Call { name, args } => {
            let prec = 6;
            let rendered = if name == "차림.값" && args.len() == 2 {
                format!(
                    "(대상={}, i={}) {}",
                    format_expr(&args[0]),
                    format_expr(&args[1]),
                    name
                )
            } else if name == "차림.바꾼값" && args.len() == 3 {
                format!(
                    "(대상={}, i={}, 값={}) {}",
                    format_expr(&args[0]),
                    format_expr(&args[1]),
                    format_expr(&args[2]),
                    name
                )
            } else if args.len() == 1 {
                if let Expr::Pack { bindings } = &args[0] {
                    let mut rendered = String::new();
                    rendered.push('(');
                    for (idx, binding) in bindings.iter().enumerate() {
                        if idx > 0 {
                            rendered.push_str(", ");
                        }
                        rendered.push_str(&binding.name);
                        rendered.push(':');
                        rendered.push_str(&format_expr(&binding.value));
                    }
                    rendered.push(')');
                    rendered.push(' ');
                    rendered.push_str(name);
                    rendered
                } else {
                    let mut rendered = String::new();
                    rendered.push('(');
                    for (idx, arg) in args.iter().enumerate() {
                        if idx > 0 {
                            rendered.push_str(", ");
                        }
                        rendered.push_str(&format_expr(arg));
                    }
                    rendered.push(')');
                    rendered.push(' ');
                    rendered.push_str(name);
                    rendered
                }
            } else {
                let mut rendered = String::new();
                rendered.push('(');
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        rendered.push_str(", ");
                    }
                    rendered.push_str(&format_expr(arg));
                }
                rendered.push(')');
                rendered.push(' ');
                rendered.push_str(name);
                rendered
            };
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::CallIn { name, bindings } => {
            let prec = 6;
            let mut rendered = String::new();
            rendered.push('(');
            for (idx, binding) in bindings.iter().enumerate() {
                if idx > 0 {
                    rendered.push_str(", ");
                }
                rendered.push_str(&binding.name);
                rendered.push('=');
                rendered.push_str(&format_expr(&binding.value));
            }
            rendered.push(')');
            rendered.push_str("인 ");
            rendered.push_str(name);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::FieldAccess { target, field } => {
            let prec = 6;
            let rendered = format!("{}.{}", format_expr_prec(target, prec), field);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Pack { bindings } => {
            let prec = 6;
            let mut rendered = String::new();
            rendered.push('(');
            for (idx, binding) in bindings.iter().enumerate() {
                if idx > 0 {
                    rendered.push_str(", ");
                }
                rendered.push_str(&binding.name);
                rendered.push('=');
                rendered.push_str(&format_expr(&binding.value));
            }
            rendered.push(')');
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Unit { value, unit } => {
            let prec = 6;
            let rendered = format!("{}@{}", format_expr_prec(value, prec), unit);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Unary { op, expr } => {
            let prec = 5;
            let inner = format_expr_prec(expr, prec);
            let rendered = match op {
                UnaryOp::Neg => format!("-{}", inner),
            };
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Binary { left, op, right } => {
            let (prec, op_text) = match op {
                BinaryOp::Or => (0, "또는"),
                BinaryOp::And => (1, "그리고"),
                BinaryOp::Eq => (2, "=="),
                BinaryOp::NotEq => (2, "!="),
                BinaryOp::Lt => (2, "<"),
                BinaryOp::Lte => (2, "<="),
                BinaryOp::Gt => (2, ">"),
                BinaryOp::Gte => (2, ">="),
                BinaryOp::Add => (3, "+"),
                BinaryOp::Sub => (3, "-"),
                BinaryOp::Mul => (4, "*"),
                BinaryOp::Div => (4, "/"),
                BinaryOp::Mod => (4, "%"),
            };
            let left_text = format_expr_prec(left, prec);
            let right_text = format_expr_prec(right, prec);
            let rendered = format!("{} {} {}", left_text, op_text, right_text);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::Pipe { left, kind, right } => {
            let prec = 0;
            let op_text = match kind {
                PipeKind::Haseo => "해서",
                PipeKind::Hago => "하고",
            };
            let left_text = format_expr_prec(left, prec);
            let right_text = format_expr_prec(right, prec);
            let rendered = format!("{} {} {}", left_text, op_text, right_text);
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
        Expr::SeedLiteral { param, body } => {
            let prec = 6;
            let rendered = format!("{{{} | {}}}", param, format_expr(body));
            if prec < parent_prec {
                format!("({})", rendered)
            } else {
                rendered
            }
        }
    }
}

fn format_condition(condition: &Condition) -> String {
    let expr = format_expr(&condition.expr);
    match condition.style {
        ConditionStyle::Plain => expr,
        ConditionStyle::Thunk => {
            let suffix = if condition.negated {
                "아닌것"
            } else {
                "인것"
            };
            format!("{{ {} }}{}", expr, suffix)
        }
    }
}

fn format_literal(literal: &Literal) -> String {
    match literal {
        Literal::Str(text) => format!("\"{}\"", escape_string(text)),
        Literal::Num(value) => value.clone(),
        Literal::Bool(true) => "참".to_string(),
        Literal::Bool(false) => "거짓".to_string(),
        Literal::None => "없음".to_string(),
        Literal::Atom(text) => format!("#{}", text),
    }
}

fn format_path(path: &Path) -> String {
    path.segments.join(".")
}

fn escape_string(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}
