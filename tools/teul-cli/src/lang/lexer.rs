use crate::lang::span::Span;
use crate::lang::token::{Token, TokenKind};
use crate::core::fixed64::Fixed64;

#[derive(Debug)]
pub enum LexError {
    UnterminatedString { line: usize, col: usize },
    UnterminatedTemplate { line: usize, col: usize },
    BadEscape { line: usize, col: usize, ch: char },
    BadIdentStart { line: usize, col: usize },
    UnexpectedChar { line: usize, col: usize, ch: char },
}

impl LexError {
    pub fn code(&self) -> &'static str {
        match self {
            LexError::UnterminatedString { .. } => "E_LEX_UNTERM_STRING",
            LexError::UnterminatedTemplate { .. } => "E_LEX_UNTERM_TEMPLATE",
            LexError::BadEscape { .. } => "E_LEX_BAD_ESCAPE",
            LexError::BadIdentStart { .. } => "E_LEX_BAD_IDENT_START",
            LexError::UnexpectedChar { .. } => "E_LEX_UNEXPECTED_CHAR",
        }
    }
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        while !lexer.is_eof() {
            lexer.skip_whitespace_except_newline();

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

        tokens.push(Token::new(TokenKind::Eof, lexer.span_here()));
        Ok(tokens)
    }

    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn span_here(&self) -> Span {
        Span::new(self.line, self.col, self.line, self.col)
    }

    fn span_from(&self, start_line: usize, start_col: usize) -> Span {
        Span::new(start_line, start_col, self.line, self.col)
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

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace_except_newline(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\u{feff}' || ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
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
            self.advance();
        }
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

    fn next_token(&mut self) -> Result<Token, LexError> {
        let ch = self.peek().unwrap();
        match ch {
            '\n' => Ok(self.read_newline()),
            '"' => self.read_string(),
            '/' if self.peek_next() == Some('/') => {
                self.read_comment();
                self.next_token()
            }
            '+' if self.peek_next() == Some('<') && self.peek_n(2) == Some('-') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::PlusArrow,
                    self.span_from(start_line, start_col),
                ))
            }
            '+' if self.peek_next() == Some('=') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::PlusEqual,
                    self.span_from(start_line, start_col),
                ))
            }
            '+' => self.single_char(TokenKind::Plus),
            '-' if self.peek_next() == Some('<') && self.peek_n(2) == Some('-') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::MinusArrow,
                    self.span_from(start_line, start_col),
                ))
            }
            '-' if self.peek_next() == Some('=') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::MinusEqual,
                    self.span_from(start_line, start_col),
                ))
            }
            '-' => self.single_char(TokenKind::Minus),
            '*' => self.single_char(TokenKind::Star),
            '/' => self.single_char(TokenKind::Slash),
            '@' => self.single_char(TokenKind::At),
            '&' if self.peek_next() == Some('&') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(TokenKind::And, self.span_from(start_line, start_col)))
            }
            '|' if self.peek_next() == Some('|') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(TokenKind::Or, self.span_from(start_line, start_col)))
            }
            '|' => self.single_char(TokenKind::Pipe),
            '(' => self.single_char(TokenKind::LParen),
            ')' => self.single_char(TokenKind::RParen),
            '[' => self.single_char(TokenKind::LBracket),
            ']' => self.single_char(TokenKind::RBracket),
            '{' => self.single_char(TokenKind::LBrace),
            '}' => self.single_char(TokenKind::RBrace),
            ',' => self.single_char(TokenKind::Comma),
            '^' => self.single_char(TokenKind::Caret),
            '#' => self.read_hash_or_atom(),
            '=' if self.peek_next() == Some('=') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(TokenKind::EqEq, self.span_from(start_line, start_col)))
            }
            '!' if self.peek_next() == Some('=') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(TokenKind::NotEq, self.span_from(start_line, start_col)))
            }
            '<' if self.peek_next() == Some('=') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(TokenKind::Lte, self.span_from(start_line, start_col)))
            }
            '>' if self.peek_next() == Some('=') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(TokenKind::Gte, self.span_from(start_line, start_col)))
            }
            '<' if self.peek_next() == Some('-') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(TokenKind::Arrow, self.span_from(start_line, start_col)))
            }
            '<' => self.single_char(TokenKind::Lt),
            '>' => self.single_char(TokenKind::Gt),
            '=' => self.single_char(TokenKind::Equal),
            '.' => {
                let (start_line, start_col) = (self.line, self.col);
                if self.peek_next() == Some('.') {
                    self.advance();
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Ok(Token::new(
                            TokenKind::DotDotEq,
                            self.span_from(start_line, start_col),
                        ))
                    } else {
                        Ok(Token::new(
                            TokenKind::DotDot,
                            self.span_from(start_line, start_col),
                        ))
                    }
                } else {
                    self.advance();
                    Ok(Token::new(TokenKind::Dot, self.span_from(start_line, start_col)))
                }
            }
            ':' => self.single_char(TokenKind::Colon),
            '0'..='9' => self.read_number(),
            _ if is_ident_start(ch) => self.read_ident_or_keyword(),
            _ => Err(LexError::UnexpectedChar {
                line: self.line,
                col: self.col,
                ch,
            }),
        }
    }

    fn read_newline(&mut self) -> Token {
        let (start_line, start_col) = (self.line, self.col);
        self.advance();
        Token::new(TokenKind::Newline, self.span_from(start_line, start_col))
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        let (start_line, start_col) = (self.line, self.col);
        self.advance();

        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                return Ok(Token::new(
                    TokenKind::String(value),
                    self.span_from(start_line, start_col),
                ));
            }
            if ch == '\n' {
                return Err(LexError::UnterminatedString {
                    line: start_line,
                    col: start_col,
                });
            }
            if ch == '\\' {
                self.advance();
                let esc = self.peek().ok_or(LexError::UnterminatedString {
                    line: start_line,
                    col: start_col,
                })?;
                let mapped = match esc {
                    '"' => '"',
                    '\\' => '\\',
                    'n' => '\n',
                    't' => '\t',
                    _ => {
                        return Err(LexError::BadEscape {
                            line: self.line,
                            col: self.col,
                            ch: esc,
                        })
                    }
                };
                self.advance();
                value.push(mapped);
                continue;
            }
            self.advance();
            value.push(ch);
        }

        Err(LexError::UnterminatedString {
            line: start_line,
            col: start_col,
        })
    }

    fn read_number(&mut self) -> Result<Token, LexError> {
        let (start_line, start_col) = (self.line, self.col);
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            text.push('.');
            self.advance();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    text.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if let Some(next) = self.peek() {
            if is_ident_start(next) {
                return Err(LexError::BadIdentStart {
                    line: self.line,
                    col: self.col,
                });
            }
        }

        let value = Fixed64::parse_literal(&text).ok_or(LexError::UnexpectedChar {
            line: start_line,
            col: start_col,
            ch: '.',
        })?;

        Ok(Token::new(
            TokenKind::Number(value.raw()),
            self.span_from(start_line, start_col),
        ))
    }

    fn single_char(&mut self, kind: TokenKind) -> Result<Token, LexError> {
        let (start_line, start_col) = (self.line, self.col);
        self.advance();
        Ok(Token::new(kind, self.span_from(start_line, start_col)))
    }

    fn read_hash_or_atom(&mut self) -> Result<Token, LexError> {
        Ok(self.read_atom_literal())
    }

    fn read_atom_literal(&mut self) -> Token {
        let (start_line, start_col) = (self.line, self.col);
        self.advance();
        let mut text = String::from("#");

        while let Some(ch) = self.peek() {
            if is_atom_stop_char(ch) {
                break;
            }
            if ch == '.' || ch == '/' {
                let next = self.peek_next();
                if next.map_or(true, is_atom_stop_char) {
                    break;
                }
            }
            text.push(ch);
            self.advance();
        }

        Token::new(TokenKind::Atom(text), self.span_from(start_line, start_col))
    }

    fn read_ident_or_keyword(&mut self) -> Result<Token, LexError> {
        let (start_line, start_col) = (self.line, self.col);
        let mut ident = String::new();

        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if ident == "글무늬" && self.peek() == Some('{') {
            return self.read_template_block(start_line, start_col);
        }

        let kind = match ident.as_str() {
            "참" => TokenKind::True,
            "거짓" => TokenKind::False,
            "없음" => TokenKind::None,
            "살림" => TokenKind::Salim,
            "보여주기" => TokenKind::Boyeojugi,
            "채우기" => TokenKind::Chaewugi,
            "수식" => TokenKind::Susic,
            "시작" => TokenKind::Start,
            "매마디" => TokenKind::EveryMadi,
            "할때" => TokenKind::Halttae,
            "마다" => TokenKind::Mada,
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
            _ => TokenKind::Ident(ident),
        };

        Ok(Token::new(kind, self.span_from(start_line, start_col)))
    }

    fn read_template_block(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, LexError> {
        self.advance();
        let body = if self.peek() == Some('"') {
            let text = self.read_string_value(start_line, start_col)?;
            self.skip_template_ws();
            if self.peek() != Some('}') {
                return Err(LexError::UnterminatedTemplate {
                    line: start_line,
                    col: start_col,
                });
            }
            text
        } else {
            let raw = self.read_raw_template_body(start_line, start_col)?;
            normalize_raw_template(&raw)
        };
        if self.peek() != Some('}') {
            return Err(LexError::UnterminatedTemplate {
                line: start_line,
                col: start_col,
            });
        }
        self.advance();
        Ok(Token::new(
            TokenKind::Template(body),
            self.span_from(start_line, start_col),
        ))
    }

    fn read_string_value(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<String, LexError> {
        self.advance();
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                return Ok(value);
            }
            if ch == '\n' {
                return Err(LexError::UnterminatedString {
                    line: start_line,
                    col: start_col,
                });
            }
            if ch == '\\' {
                self.advance();
                let esc = self.peek().ok_or(LexError::UnterminatedString {
                    line: start_line,
                    col: start_col,
                })?;
                let mapped = match esc {
                    '"' => '"',
                    '\\' => '\\',
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    _ => {
                        return Err(LexError::BadEscape {
                            line: self.line,
                            col: self.col,
                            ch: esc,
                        })
                    }
                };
                self.advance();
                value.push(mapped);
                continue;
            }
            self.advance();
            value.push(ch);
        }
        Err(LexError::UnterminatedString {
            line: start_line,
            col: start_col,
        })
    }

    fn read_raw_template_body(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<String, LexError> {
        let mut depth = 1;
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if ch == '{' {
                if self.peek_next() == Some('{') {
                    out.push('{');
                    out.push('{');
                    self.advance();
                    self.advance();
                    continue;
                }
                depth += 1;
                out.push('{');
                self.advance();
                continue;
            }
            if ch == '}' {
                if self.peek_next() == Some('}') {
                    out.push('}');
                    out.push('}');
                    self.advance();
                    self.advance();
                    continue;
                }
                if depth == 1 {
                    return Ok(out);
                }
                depth -= 1;
                self.advance();
                out.push('}');
                continue;
            }
            out.push(ch);
            self.advance();
        }
        Err(LexError::UnterminatedTemplate {
            line: start_line,
            col: start_col,
        })
    }

    fn skip_template_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
}

fn normalize_raw_template(input: &str) -> String {
    let mut text = input.replace("\r\n", "\n").replace('\r', "\n");
    if text.starts_with('\n') {
        text.remove(0);
    }
    if text.ends_with('\n') {
        text.pop();
    }
    text = dedent_text(&text);
    join_lines_with_backslash(&text)
}

fn dedent_text(input: &str) -> String {
    let lines: Vec<&str> = input.split('\n').collect();
    let mut min_indent: Option<usize> = None;
    for line in &lines {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|ch| *ch == ' ' || *ch == '\t').count();
        min_indent = Some(match min_indent {
            Some(value) => value.min(indent),
            None => indent,
        });
    }
    let Some(min_indent) = min_indent else {
        return input.to_string();
    };
    let mut out = String::new();
    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let mut skipped = 0;
        for ch in line.chars() {
            if skipped < min_indent && (ch == ' ' || ch == '\t') {
                skipped += 1;
                continue;
            }
            out.push(ch);
        }
    }
    out
}

fn join_lines_with_backslash(input: &str) -> String {
    let mut out = String::new();
    let mut lines = input.split('\n').peekable();
    while let Some(line) = lines.next() {
        if line.ends_with('\\') {
            let trimmed = &line[..line.len() - 1];
            out.push_str(trimmed);
            continue;
        }
        out.push_str(line);
        if lines.peek().is_some() {
            out.push('\n');
        }
    }
    out
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic() || is_hangul(ch)
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}

fn is_atom_stop_char(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            '(' | ')' | '[' | ']' | '{' | '}' | ',' | '+' | '-' | '*' | '=' | '<' | '>' | '|' | '&'
        )
}

fn is_hangul(ch: char) -> bool {
    matches!(
        ch,
        '\u{AC00}'..='\u{D7AF}' | '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}'
    )
}
