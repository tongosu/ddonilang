use crate::core::fixed64::Fixed64;
use crate::lang::dialect::DialectConfig;
use crate::lang::span::Span;
use crate::lang::token::{Token, TokenKind};

#[derive(Debug)]
pub enum LexError {
    UnterminatedString { line: usize, col: usize },
    UnterminatedTemplate { line: usize, col: usize },
    UnterminatedAssertion { line: usize, col: usize },
    BadEscape { line: usize, col: usize, ch: char },
    BadIdentStart { line: usize, col: usize },
    UnexpectedChar { line: usize, col: usize, ch: char },
}

impl LexError {
    pub fn code(&self) -> &'static str {
        match self {
            LexError::UnterminatedString { .. } => "E_LEX_UNTERM_STRING",
            LexError::UnterminatedTemplate { .. } => "E_LEX_UNTERM_TEMPLATE",
            LexError::UnterminatedAssertion { .. } => "E_LEX_UNTERM_ASSERTION",
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
    dialect: DialectConfig,
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
                let token = lexer.read_line_pragma();
                tokens.push(token);
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
        let dialect = DialectConfig::from_source(source);
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            dialect,
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

    fn match_str(&self, token: &str) -> bool {
        for (idx, ch) in token.chars().enumerate() {
            if self.peek_n(idx) != Some(ch) {
                return false;
            }
        }
        true
    }

    fn advance_n(&mut self, count: usize) {
        for _ in 0..count {
            if self.advance().is_none() {
                break;
            }
        }
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
        if let Some(token) = self.try_read_sym3_token() {
            return Ok(token);
        }
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
            '%' => self.single_char(TokenKind::Percent),
            '@' => self.single_char(TokenKind::At),
            '&' if self.peek_next() == Some('&') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::And,
                    self.span_from(start_line, start_col),
                ))
            }
            '|' if self.peek_next() == Some('|') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::Or,
                    self.span_from(start_line, start_col),
                ))
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
                Ok(Token::new(
                    TokenKind::EqEq,
                    self.span_from(start_line, start_col),
                ))
            }
            '!' if self.peek_next() == Some('=') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::NotEq,
                    self.span_from(start_line, start_col),
                ))
            }
            '<' if self.peek_next() == Some('=') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::Lte,
                    self.span_from(start_line, start_col),
                ))
            }
            '>' if self.peek_next() == Some('=') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::Gte,
                    self.span_from(start_line, start_col),
                ))
            }
            '<' if self.peek_next() == Some('-') => {
                let (start_line, start_col) = (self.line, self.col);
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::Arrow,
                    self.span_from(start_line, start_col),
                ))
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
                    Ok(Token::new(
                        TokenKind::Dot,
                        self.span_from(start_line, start_col),
                    ))
                }
            }
            ':' => self.single_char(TokenKind::Colon),
            '~' => self.read_josa(),
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
                let suffix_ident = self.peek_ident_text().unwrap_or_default();
                if suffix_ident == "마디" {
                    // `(N마디)마다` 주기 훅 표면을 위해 숫자 뒤 `마디` 접미를 허용한다.
                } else {
                    return Err(LexError::BadIdentStart {
                        line: self.line,
                        col: self.col,
                    });
                }
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

    fn peek_ident_text(&self) -> Option<String> {
        let mut idx = self.pos;
        let mut ident = String::new();
        while let Some(ch) = self.chars.get(idx).copied() {
            if is_ident_continue(ch) {
                ident.push(ch);
                idx += 1;
            } else {
                break;
            }
        }
        if ident.is_empty() {
            None
        } else {
            Some(ident)
        }
    }

    fn single_char(&mut self, kind: TokenKind) -> Result<Token, LexError> {
        let (start_line, start_col) = (self.line, self.col);
        self.advance();
        Ok(Token::new(kind, self.span_from(start_line, start_col)))
    }

    fn read_hash_or_atom(&mut self) -> Result<Token, LexError> {
        Ok(self.read_atom_literal())
    }

    fn read_line_pragma(&mut self) -> Token {
        let (start_line, start_col) = (self.line, self.col);
        self.advance(); // '#'
        while let Some(ch) = self.peek() {
            if matches!(ch, ' ' | '\t') {
                self.advance();
            } else {
                break;
            }
        }

        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            text.push(ch);
            self.advance();
        }

        let trimmed = text.trim_end().to_string();
        Token::new(
            TokenKind::Pragma(trimmed),
            self.span_from(start_line, start_col),
        )
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
        if ident == "세움" && self.peek() == Some('{') {
            return self.read_assertion_block(start_line, start_col);
        }

        if let Some(canon) = self.dialect.canonicalize(&ident) {
            ident = canon.to_string();
        }
        if ident == "입력키" || ident == "찾기" || ident == "글바꾸기" {
            if let Some(suffix) = self.peek() {
                let can_use_suffix = matches!(suffix, '?' | '!')
                    && !(suffix == '?' && self.peek_next() == Some('?'))
                    && !self.peek_next().map(is_ident_continue).unwrap_or(false)
                    && !(ident == "찾기" && suffix == '!')
                    && !(ident == "글바꾸기" && suffix == '?');
                if can_use_suffix {
                    ident.push(suffix);
                    self.advance();
                }
            }
        }

        let kind = self.classify_ident(ident);

        Ok(Token::new(kind, self.span_from(start_line, start_col)))
    }

    fn read_josa(&mut self) -> Result<Token, LexError> {
        let (start_line, start_col) = (self.line, self.col);
        let saved_pos = self.pos;
        let saved_line = self.line;
        let saved_col = self.col;
        let mut text = String::new();
        if let Some(ch) = self.advance() {
            text.push(ch);
        }
        // Phase 1: read ident-only chars (josa candidate)
        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        // Check if this is a known josa
        if let Some(role) = self.dialect.canonicalize_josa(&text) {
            let span = self.span_from(start_line, start_col);
            return Ok(Token::new(TokenKind::Josa(role.to_string()), span));
        }
        // Phase 2: not a josa — rewind and read full tilde alias (including parens/dash)
        self.pos = saved_pos;
        self.line = saved_line;
        self.col = saved_col;
        let mut full_text = String::new();
        if let Some(ch) = self.advance() {
            full_text.push(ch);
        }
        while let Some(ch) = self.peek() {
            if is_tilde_tail_char(ch) {
                full_text.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        Ok(Token::new(
            TokenKind::Ident(full_text),
            self.span_from(start_line, start_col),
        ))
    }

    fn classify_ident(&self, ident: String) -> TokenKind {
        match ident.as_str() {
            "참" => TokenKind::True,
            "거짓" => TokenKind::False,
            "없음" => TokenKind::None,
            "바탕" => TokenKind::Salim,
            "보여주기" => TokenKind::Boyeojugi,
            "톺아보기" | "감사" => TokenKind::Tolabogi,
            "채우기" => TokenKind::Chaewugi,
            "수식" => TokenKind::Susic,
            "처음" | "시작" => TokenKind::Start,
            "매마디" => TokenKind::EveryMadi,
            "할때" => TokenKind::Halttae,
            "마다" => TokenKind::Mada,
            "일때" => TokenKind::Ilttae,
            "아니면" => TokenKind::Aniramyeon,
            "아니고" => TokenKind::Anigo,
            "맞으면" => TokenKind::Majeumyeon,
            "바탕으로" => TokenKind::Jeonjehae,
            "다짐하고" => TokenKind::Bojanghago,
            "고르기" => TokenKind::Goreugi,
            "되풀이" | "반복" => TokenKind::Repeat,
            "동안" => TokenKind::During,
            "덩이" => TokenKind::Beat,
            "미루기" => TokenKind::Reserve,
            "대해" => TokenKind::Daehae,
            "멈추기" => TokenKind::Break,
            "그리고" => TokenKind::And,
            "또는" => TokenKind::Or,
            _ => TokenKind::Ident(ident),
        }
    }

    fn try_read_sym3_token(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_col = self.col;
        if self.match_str("~~>") {
            self.advance_n(3);
            return Some(Token::new(
                TokenKind::SignalArrow,
                self.span_from(start_line, start_col),
            ));
        }
        for token in DialectConfig::sym3_tokens() {
            if token.is_empty() {
                continue;
            }
            if !self.match_str(token) {
                continue;
            }
            let canon = self
                .dialect
                .canonicalize_symbol(token)
                .map(str::to_string)?;
            self.advance_n(token.chars().count());
            let kind = self.classify_ident(canon);
            return Some(Token::new(kind, self.span_from(start_line, start_col)));
        }
        None
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

    fn read_assertion_block(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, LexError> {
        self.advance();
        let mut depth = 1usize;
        let mut body = String::new();
        while let Some(ch) = self.peek() {
            if ch == '{' {
                depth += 1;
                body.push(ch);
                self.advance();
                continue;
            }
            if ch == '}' {
                depth = depth.saturating_sub(1);
                self.advance();
                if depth == 0 {
                    return Ok(Token::new(
                        TokenKind::Assertion(body),
                        self.span_from(start_line, start_col),
                    ));
                }
                body.push('}');
                continue;
            }
            body.push(ch);
            self.advance();
        }
        Err(LexError::UnterminatedAssertion {
            line: start_line,
            col: start_col,
        })
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
        let indent = line
            .chars()
            .take_while(|ch| *ch == ' ' || *ch == '\t')
            .count();
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
    ch == '_' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_numeric()
}

fn is_tilde_tail_char(ch: char) -> bool {
    is_ident_continue(ch) || ch == '(' || ch == ')' || ch == '-'
}

fn is_atom_stop_char(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            '(' | ')' | '[' | ']' | '{' | '}' | ',' | '+' | '-' | '*' | '=' | '<' | '>' | '|' | '&'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_start_hash_becomes_pragma_token() {
        let source = "#그래프(y축=x)\n살림.x <- 1.\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        assert!(!tokens.is_empty());
        match &tokens[0].kind {
            TokenKind::Pragma(text) => assert_eq!(text, "그래프(y축=x)"),
            other => panic!("expected pragma token, got: {:?}", other),
        }
    }

    #[test]
    fn inline_hash_keeps_atom_token() {
        let source = "살림.x <- (#ascii) 수식{x+1}.\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let has_ascii_atom = tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Atom(text) if text == "#ascii"));
        assert!(has_ascii_atom, "expected #ascii atom token");
    }

    #[test]
    fn english_keyword_is_only_active_under_en_dialect() {
        let ko_tokens = Lexer::tokenize("if 참.\n").expect("tokenize");
        assert!(ko_tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Ident(name) if name == "if")));
        assert!(!ko_tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ilttae)));

        let en_tokens = Lexer::tokenize("#말씨: en\nif 참.\n").expect("tokenize");
        assert!(en_tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ilttae)));
    }

    #[test]
    fn unsupported_dialect_keeps_english_keyword_as_ident() {
        let tokens = Lexer::tokenize("#말씨: xx\nif 참.\n").expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Ident(name) if name == "if")));
        assert!(!tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ilttae)));
    }

    #[test]
    fn input_key_suffixes_are_lexed_as_ident() {
        let tokens = Lexer::tokenize("값 <- () 입력키?.\n값2 <- () 입력키!.\n").expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Ident(name) if name == "입력키?")));
        assert!(tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Ident(name) if name == "입력키!")));
    }

    #[test]
    fn map_lookup_optional_suffix_is_lexed_as_ident() {
        let tokens =
            Lexer::tokenize("값 <- ((\"a\", 1) 짝맞춤, \"a\") 찾기?.\n").expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Ident(name) if name == "찾기?")));
    }

    #[test]
    fn string_replace_strict_suffix_is_lexed_as_ident() {
        let tokens = Lexer::tokenize("값 <- (\"가\", 0, \"나\") 글바꾸기!.\n").expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Ident(name) if name == "글바꾸기!")));
    }

    #[test]
    fn number_madi_suffix_is_lexed_for_periodic_hook() {
        let tokens = Lexer::tokenize("(3마디)마다 { 없음. }.\n").expect("tokenize");
        assert!(matches!(tokens.first().map(|token| &token.kind), Some(TokenKind::LParen)));
        assert!(matches!(tokens.get(1).map(|token| &token.kind), Some(TokenKind::Number(_))));
        assert!(matches!(
            tokens.get(2).map(|token| &token.kind),
            Some(TokenKind::Ident(name)) if name == "마디"
        ));
    }

    #[test]
    fn start_hook_aliases_lex_to_start_token() {
        let tokens = Lexer::tokenize("(시작)할때 { 없음. }.\n(처음)할때 { 없음. }.\n")
            .expect("tokenize");
        let start_count = tokens
            .iter()
            .filter(|token| matches!(token.kind, TokenKind::Start))
            .count();
        assert_eq!(start_count, 2);
    }

    #[test]
    fn signal_arrow_is_lexed_as_token() {
        let tokens = Lexer::tokenize("값 <- 참 ~~> 거짓.\n").expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::SignalArrow)));
    }
}
