use std::collections::HashSet;

use crate::file_meta::{format_file_meta, split_file_meta, FileMeta};

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

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Ident(String),
    Atom(String),
    Template(String),
    Formula(String),
    String(String),
    Number(String),
    True,
    False,
    None,
    Show,
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
    Dot,
    Arrow,
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

        if ch == '\n' || ch == '\r' {
            self.consume_newline();
            return Ok(Token {
                kind: TokenKind::Newline,
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
            if !self
                .peek_next()
                .map(is_ident_continue)
                .unwrap_or(false)
            {
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
            if (ident == "글무늬" || ident == "수식") && self.peek() == Some('{') {
                let body = self.lex_brace_block()?;
                let kind = if ident == "글무늬" {
                    TokenKind::Template(body)
                } else {
                    TokenKind::Formula(body)
                };
                return Ok(Token { kind });
            }
            let kind = match ident.as_str() {
                "보여주기" => TokenKind::Show,
                "일때" => TokenKind::Ilttae,
                "아니면" => TokenKind::Aniramyeon,
                "아니고" => TokenKind::Anigo,
                "맞으면" => TokenKind::Majeumyeon,
                "바탕으로" => TokenKind::Jeonjehae,
                "전제하에" => TokenKind::Jeonjehae,
                "다짐하고" => TokenKind::Bojanghago,
                "보장하고" => TokenKind::Bojanghago,
                "고르기" => TokenKind::Goreugi,
                "반복" => TokenKind::Repeat,
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
        if self.peek() == Some('.') && self.peek_next().map(|ch| ch.is_ascii_digit()).unwrap_or(false) {
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
                if matches!(self.peek_next(), None | Some(' ') | Some('\t') | Some('\r') | Some('\n')) {
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
            return Err(CanonError::new(
                "E_CANON_BAD_STRING",
                "문자열이 아닙니다.",
            ));
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
    type_name: String,
    value: Option<Expr>,
}

#[derive(Debug, Clone)]
enum SurfaceStmt {
    RootDecl {
        kind: DeclKind,
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
    OpenBlock {
        body: Vec<SurfaceStmt>,
    },
    Return {
        value: Expr,
    },
    Expr {
        value: Expr,
    },
    Break,
}

#[derive(Debug, Clone)]
enum Stmt {
    RootDecl {
        kind: DeclKind,
        items: Vec<DeclItem>,
    },
    Assign { target: Path, value: Expr },
    SeedDef {
        name: String,
        kind: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    Show { value: Expr },
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
    OpenBlock {
        body: Vec<Stmt>,
    },
    Return {
        value: Expr,
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
    Template { body: String },
    TemplateApply { bindings: Vec<Binding>, body: String },
    Formula { tag: Option<String>, body: String },
    PromptExpr { expr: Box<Expr> },
    PromptBlock { body: String },
    Path(Path),
    FieldAccess { target: Box<Expr>, field: String },
    Call { name: String, args: Vec<Expr> },
    CallIn { name: String, bindings: Vec<Binding> },
    Pack { bindings: Vec<Binding> },
    Unit { value: Box<Expr>, unit: String },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
    Pipe { left: Box<Expr>, kind: PipeKind, right: Box<Expr> },
    SeedLiteral { param: String, body: Box<Expr> },
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
}

impl Parser {
    fn new(tokens: Vec<Token>, bridge: bool) -> Self {
        Self { tokens, pos: 0, bridge }
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

        let expr = self.parse_expr()?;
        if self.peek_is(|k| matches!(k, TokenKind::Ilttae)) {
            return self.parse_if_stmt(Condition {
                expr,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Jeonjehae | TokenKind::Bojanghago)) {
            return self.parse_contract_stmt(Condition {
                expr,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
            return self.parse_prompt_after_stmt(expr);
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
                let Expr::Path(path) = expr else {
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
                match expr {
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
                        if let Some((target, field)) = Self::split_map_dot_target(&path) {
                            let key_expr = Expr::Literal(Literal::Str(field));
                            let value_call = Expr::Call {
                                name: "짝맞춤.바꾼값".to_string(),
                                args: vec![Expr::Path(target.clone()), key_expr, value],
                            };
                            SurfaceStmt::Assign {
                                target,
                                value: value_call,
                            }
                        } else {
                            SurfaceStmt::Assign { target: path, value }
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
            return Ok(SurfaceStmt::Show { value: expr });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "돌려줘")) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Return { value: expr });
        }
        if matches!(expr, Expr::Call { .. } | Expr::CallIn { .. }) {
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Expr { value: expr });
        }
        Err(CanonError::new(
            "E_CANON_UNEXPECTED_TOKEN",
            "예상하지 못한 토큰입니다.",
        ))
    }

    fn peek_decl_block_kind(&self) -> Option<DeclKind> {
        if !self.peek_n_is(1, |k| matches!(k, TokenKind::Colon)) {
            return None;
        }
        match self.peek() {
            Some(TokenKind::Ident(name)) if name == "그릇채비" => Some(DeclKind::Gureut),
            Some(TokenKind::Ident(name)) if name == "바탕칸" => Some(DeclKind::Gureut),
            Some(TokenKind::Ident(name)) if name == "바탕칸표" => Some(DeclKind::Gureut),
            Some(TokenKind::Ident(name)) if name == "붙박이마련" => Some(DeclKind::Butbak),
            _ => None,
        }
    }

    fn parse_decl_block(&mut self, kind: DeclKind) -> Result<SurfaceStmt, CanonError> {
        self.advance(); // keyword
        self.expect(TokenKind::Colon)?;
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
            if self.peek_is(|k| matches!(k, TokenKind::Arrow)) {
                if kind == DeclKind::Butbak {
                    return Err(CanonError::new(
                        "E_CANON_BUTBAK_ARROW_FORBIDDEN",
                        "붙박이마련 항목은 '<-'가 아니라 '='를 사용합니다",
                    ));
                }
                self.advance();
                value = Some(self.parse_expr()?);
            } else if self.peek_is(|k| matches!(k, TokenKind::Equals)) {
                if kind == DeclKind::Gureut {
                    return Err(CanonError::new(
                        "E_CANON_GUREUT_EQUAL_FORBIDDEN",
                        "그릇채비 항목에서는 '='를 사용할 수 없습니다",
                    ));
                }
                self.advance();
                value = Some(self.parse_expr()?);
            }

            if kind == DeclKind::Butbak && value.is_none() {
                return Err(CanonError::new(
                    "E_CANON_CONST_MISSING_VALUE",
                    format!("붙박이마련에는 초기값이 필요합니다: {}", name),
                ));
            }

            self.consume_terminator()?;
            items.push(DeclItem {
                name,
                type_name,
                value,
            });
        }

        self.expect(TokenKind::RBrace)?;
        self.consume_terminator()?;
        Ok(SurfaceStmt::RootDecl { kind, items })
    }

    fn is_hook_start(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            return false;
        }
        let mut idx = self.pos + 1;
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Ident(_)))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
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
        let suffix_ok = matches!(
            kind,
            TokenKind::Ident(name) if name == "할때" || name == "마다"
        );
        if !suffix_ok {
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

    fn parse_hook_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::LParen)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::RParen)?;
        let suffix = match self.expect_ident()?.as_str() {
            "할때" => HookSuffix::Halttae,
            "마다" => HookSuffix::Mada,
            _ => {
                return Err(CanonError::new(
                    "E_CANON_HOOK_SUFFIX",
                    "훅 접미는 할때/마다만 허용됩니다.",
                ))
            }
        };
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Hook { name, suffix, body })
    }

    fn is_open_block_start(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "열림")) {
            return false;
        }
        self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace))
    }

    fn parse_open_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let name = self.expect_ident()?;
        if name != "열림" {
            return Err(CanonError::new(
                "E_CANON_OPEN_BLOCK",
                "열림 블록은 '열림'으로 시작해야 합니다.",
            ));
        }
        self.skip_newlines();
        if !self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(CanonError::new(
                "E_CANON_EXPECTED_LBRACE",
                "열림 블록에는 '{'가 필요합니다.",
            ));
        }
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::OpenBlock { body })
    }

    fn is_seed_def_start(&self) -> bool {
        let mut idx = self.pos;
        if self.tokens.get(idx).map(|t| matches!(t.kind, TokenKind::LParen)) == Some(true) {
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
        let body = self.parse_block()?;
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
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Repeat { body })
    }

    fn parse_while_stmt(&mut self, condition: Condition) -> Result<SurfaceStmt, CanonError> {
        self.advance();
        self.expect(TokenKind::Colon)?;
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
                        .map(|tok| matches!(tok.kind, TokenKind::Colon))
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
            self.expect(TokenKind::Colon)?;
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
        let mut expr = self.parse_additive()?;
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
            let right = self.parse_additive()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
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
            if self.peek_is(|k| matches!(k, TokenKind::Dot))
                && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(_)))
            {
                self.advance();
                let field = self.expect_ident()?;
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
            && self.peek_n_is(2, |k| matches!(k, TokenKind::Equals))
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
            if self.peek_is(|k| matches!(k, TokenKind::Ident(_))) {
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
            if self.peek_is(|k| matches!(k, TokenKind::Ident(_))) {
                let name = self.parse_call_name()?;
                return Ok(Expr::Call {
                    name,
                    args,
                });
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
                if prefix == "글" || prefix == "수" {
                    self.advance();
                    let literal = self.parse_literal()?;
                    let ok = match (&prefix[..], &literal) {
                        ("글", Literal::Str(_)) => true,
                        ("수", Literal::Num(_)) => true,
                        _ => false,
                    };
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
                self.expect(TokenKind::Equals)?;
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
                    if args.iter().any(|item| matches!(item, Expr::Call { name, .. } if name == "차림")) {
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
        Ok(SurfaceStmt::Choose { branches, else_body })
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
        Ok(SurfaceStmt::PromptChoose { branches, else_body })
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

    fn parse_prompt_condition_stmt(&mut self, condition: Condition) -> Result<SurfaceStmt, CanonError> {
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
            "중단" => Ok(ContractMode::Abort),
            _ => Err(CanonError::new(
                "E_CANON_CONTRACT_MODE",
                "계약 모드는 알림 또는 중단이어야 합니다.",
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
            if !self.peek_is(|k| matches!(k, TokenKind::Dot)) {
                break;
            }
            if !self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(_))) {
                break;
            }
            self.advance();
            let ident = self.expect_ident()?;
            segments.push(ident);
        }
        Ok(Path { segments })
    }

    // Step B scope: allow only one-level map write.
    // - allowed: 살림.공.x, 바탕.공.x, 공.x
    // - rejected: 공.속도.x, 살림.공.속도.x (Step C)
    fn split_map_dot_target(path: &Path) -> Option<(Path, String)> {
        if path.segments.len() < 2 {
            return None;
        }
        let has_root = matches!(
            path.segments.first().map(|seg| seg.as_str()),
            Some("살림" | "바탕" | "샘")
        );
        if has_root {
            if path.segments.len() != 3 {
                return None;
            }
            let field = path.segments[2].clone();
            let target = Path {
                segments: vec![path.segments[0].clone(), path.segments[1].clone()],
            };
            return Some((target, field));
        }
        if path.segments.len() != 2 {
            return None;
        }
        let field = path.segments[1].clone();
        let target = Path {
            segments: vec![path.segments[0].clone()],
        };
        Some((target, field))
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
                if self
                    .peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "해서" || name == "하고"))
                {
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
            && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(ref name) if name == "그려"))
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
        Err(CanonError::new(
            "E_CANON_UNEXPECTED_TOKEN",
            "예상하지 못한 토큰입니다.",
        ))
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

pub struct CanonOutput {
    pub ddn: String,
    pub meta: FileMeta,
    pub warnings: Vec<String>,
}

pub fn canonicalize(input: &str, bridge: bool) -> Result<CanonOutput, CanonError> {
    let meta_parse = split_file_meta(input);
    let root_hide = has_root_hide_directive(input);
    let default_root = "바탕";
    let tokens = Lexer::tokenize(input)?;
    let mut parser = Parser::new(tokens, bridge);
    let surface = parser.parse_program()?;

    let mut declared = HashSet::new();
    for stmt in &surface {
        match stmt {
            SurfaceStmt::Decl { name, .. } => {
                declared.insert(name.clone());
            }
            SurfaceStmt::RootDecl { kind, items } if *kind == DeclKind::Gureut => {
                for item in items {
                    declared.insert(item.name.clone());
                }
            }
            _ => {}
        }
    }

    let mut canonical = Vec::new();
    for stmt in surface {
        match stmt {
            SurfaceStmt::RootDecl { kind, items } => {
                let items = items
                    .into_iter()
                    .map(|item| DeclItem {
                        name: item.name,
                        type_name: item.type_name,
                        value: item
                            .value
                            .map(|expr| canonicalize_expr(expr, &declared, bridge, default_root, root_hide)),
                    })
                    .collect();
                canonical.push(Stmt::RootDecl { kind, items });
            }
            SurfaceStmt::Decl {
                name,
                type_name,
                value,
            } => {
                if type_name != "글" && type_name != "수" && type_name != "참거짓" {
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
                        default: param
                            .default
                            .map(|expr| canonicalize_expr(expr, &declared, bridge, default_root, root_hide)),
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
            SurfaceStmt::Show { value } => {
                let value = canonicalize_expr(value, &declared, bridge, default_root, root_hide);
                canonical.push(Stmt::Show { value });
            }
            SurfaceStmt::BogaeDraw => {
                canonical.push(Stmt::BogaeDraw);
            }
            SurfaceStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition = canonicalize_condition(condition, &declared, bridge, default_root, root_hide);
                let then_body = canonicalize_body(then_body, &declared, bridge, default_root, root_hide)?;
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(body, &declared, bridge, default_root, root_hide)?),
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
                let condition = canonicalize_condition(condition, &declared, bridge, default_root, root_hide);
                let else_body = canonicalize_body(else_body, &declared, bridge, default_root, root_hide)?;
                let then_body = match then_body {
                    Some(body) => Some(canonicalize_body(body, &declared, bridge, default_root, root_hide)?),
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
                let condition = canonicalize_condition(condition, &declared, bridge, default_root, root_hide);
                let body = canonicalize_body(body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::While { condition, body });
            }
            SurfaceStmt::ForEach { item, iterable, body } => {
                let iterable = canonicalize_expr(iterable, &declared, bridge, default_root, root_hide);
                let body = canonicalize_body(body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::ForEach { item, iterable, body });
            }
            SurfaceStmt::Hook { name, suffix, body } => {
                let body = canonicalize_body(body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::Hook { name, suffix, body });
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
            SurfaceStmt::PromptChoose { branches, else_body } => {
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
                        body: canonicalize_body(branch.body, &declared, bridge, default_root, root_hide)?,
                    });
                }
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(body, &declared, bridge, default_root, root_hide)?),
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
                    condition: canonicalize_condition(condition, &declared, bridge, default_root, root_hide),
                    body: canonicalize_body(body, &declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptBlock { body } => {
                canonical.push(Stmt::PromptBlock {
                    body: canonicalize_body(body, &declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::Choose { branches, else_body } => {
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
                        body: canonicalize_body(branch.body, &declared, bridge, default_root, root_hide)?,
                    });
                }
                let else_body = canonicalize_body(else_body, &declared, bridge, default_root, root_hide)?;
                canonical.push(Stmt::Choose {
                    branches: mapped,
                    else_body,
                });
            }
        }
    }

    let ddn = format_program(&canonical);
    let mut warnings = Vec::new();
    if !meta_parse.dup_keys.is_empty() {
        warnings.push(format!(
            "META_DUP_KEY {}",
            meta_parse.dup_keys.join(", ")
        ));
    }
    let mut output = String::new();
    output.push_str(&format_file_meta(&meta_parse.meta));
    output.push_str(&ddn);
    Ok(CanonOutput {
        ddn: output,
        meta: meta_parse.meta,
        warnings,
    })
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
            SurfaceStmt::RootDecl { kind, items } => {
                let items = items
                    .into_iter()
                    .map(|item| DeclItem {
                        name: item.name,
                        type_name: item.type_name,
                        value: item
                            .value
                            .map(|expr| canonicalize_expr(expr, declared, bridge, default_root, root_hide)),
                    })
                    .collect();
                out.push(Stmt::RootDecl { kind, items });
            }
            SurfaceStmt::Decl {
                name,
                type_name,
                value,
            } => {
                if type_name != "글" && type_name != "수" && type_name != "참거짓" {
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
                        default: param
                            .default
                            .map(|expr| canonicalize_expr(expr, declared, bridge, default_root, root_hide)),
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
            SurfaceStmt::BogaeDraw => {
                out.push(Stmt::BogaeDraw);
            }
            SurfaceStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let then_body = canonicalize_body(then_body, declared, bridge, default_root, root_hide)?;
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(body, declared, bridge, default_root, root_hide)?),
                    None => None,
                };
                out.push(Stmt::If {
                    condition: canonicalize_condition(condition, declared, bridge, default_root, root_hide),
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
                let else_body = canonicalize_body(else_body, declared, bridge, default_root, root_hide)?;
                let then_body = match then_body {
                    Some(body) => Some(canonicalize_body(body, declared, bridge, default_root, root_hide)?),
                    None => None,
                };
                out.push(Stmt::Contract {
                    kind,
                    mode,
                    condition: canonicalize_condition(condition, declared, bridge, default_root, root_hide),
                    then_body,
                    else_body,
                });
            }
            SurfaceStmt::Choose { branches, else_body } => {
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
                        body: canonicalize_body(branch.body, declared, bridge, default_root, root_hide)?,
                    });
                }
                out.push(Stmt::Choose {
                    branches: mapped,
                    else_body: canonicalize_body(else_body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptChoose { branches, else_body } => {
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
                        body: canonicalize_body(branch.body, declared, bridge, default_root, root_hide)?,
                    });
                }
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(body, declared, bridge, default_root, root_hide)?),
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
                    condition: canonicalize_condition(condition, declared, bridge, default_root, root_hide),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptBlock { body } => {
                out.push(Stmt::PromptBlock {
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::Repeat { body } => {
                out.push(Stmt::Repeat {
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::While { condition, body } => {
                out.push(Stmt::While {
                    condition: canonicalize_condition(condition, declared, bridge, default_root, root_hide),
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
                    iterable: canonicalize_expr(iterable, declared, bridge, default_root, root_hide),
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
        Stmt::RootDecl { kind, items } => {
            let keyword = match kind {
                DeclKind::Gureut => "그릇채비",
                DeclKind::Butbak => "붙박이마련",
            };
            out.push_str(&format!("{}{}: {{\n", pad, keyword));
            for item in items {
                out.push_str(&pad);
                out.push_str("  ");
                out.push_str(&item.name);
                out.push(':');
                out.push_str(&item.type_name);
                if let Some(value) = &item.value {
                    out.push_str(" = ");
                    out.push_str(&format_expr(value));
                }
                out.push_str(".\n");
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::Assign { target, value } => {
            out.push_str(&format!("{}{} <- {}.\n", pad, format_path(target), format_expr(value)));
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
        Stmt::Return { value } => {
            out.push_str(&format!("{}{} 돌려줘.\n", pad, format_expr(value)));
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
        Stmt::Choose { branches, else_body } => {
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
        Stmt::PromptChoose { branches, else_body } => {
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
            out.push_str(&format!("{}반복: {{\n", pad));
            for stmt in body {
                format_stmt(stmt, indent + 1, out);
            }
            out.push_str(&format!("{}}}.\n", pad));
        }
        Stmt::While { condition, body } => {
            out.push_str(&format!("{}{} 동안: {{\n", pad, format_condition(condition)));
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
                "{}({}) {}에 대해: {{\n",
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
        Stmt::OpenBlock { body } => {
            out.push_str(&format!("{}열림 {{\n", pad));
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

fn format_expr(expr: &Expr) -> String {
    format_expr_prec(expr, 0)
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
                        rendered.push('=');
                        rendered.push_str(&format_expr(&binding.value));
                    }
                    rendered.push(')');
                    rendered.push_str("인 ");
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
            let suffix = if condition.negated { "아닌것" } else { "인것" };
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
