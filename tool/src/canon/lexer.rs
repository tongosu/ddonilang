//! Surface canonicalization lexer; token types and method bodies unchanged.
use super::*;

impl<'a> Lexer<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            _input: input,
        }
    }

    pub(super) fn tokenize(input: &'a str) -> Result<Vec<Token>, CanonError> {
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

    pub(super) fn next_token(&mut self) -> Result<Token, CanonError> {
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
        if ch == '-' && self.peek_next() == Some('-') && self.peek_n(2) == Some('>') {
            self.bump();
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::TypeArrow,
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
        if ch == ':' && self.peek_next() == Some('=') {
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::Define,
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
        if ch == '=' && self.peek_next() == Some(':') && self.peek_n(2) == Some('=') {
            self.bump();
            self.bump();
            self.bump();
            return Ok(Token {
                kind: TokenKind::RelationEq,
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
                || ident == "모양"
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
                        "모양" => TokenKind::GuseongBlock(body),
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
                "만약" => TokenKind::Manyak,
                "이라면" => TokenKind::Iramyeon,
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
                "건너뛰기" => TokenKind::ContinueLoop,
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

    pub(super) fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\u{feff}' || ch == ' ' || ch == '\t' || ch == '\r' {
                self.bump();
            } else {
                break;
            }
        }
    }

    pub(super) fn skip_inline_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\u{feff}' || ch == ' ' || ch == '\t' || ch == '\r' {
                self.bump();
            } else {
                break;
            }
        }
    }

    pub(super) fn read_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.bump();
        }
    }

    pub(super) fn consume_newline(&mut self) {
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

    pub(super) fn lex_ident(&mut self) -> String {
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

    pub(super) fn lex_number(&mut self) -> Result<String, CanonError> {
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

    pub(super) fn lex_atom(&mut self) -> Result<String, CanonError> {
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

    pub(super) fn lex_string(&mut self) -> Result<String, CanonError> {
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
                buf.push_str(&self.lex_text_escape()?);
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

    pub(super) fn lex_text_escape(&mut self) -> Result<String, CanonError> {
        if self.peek().is_none() {
            return Err(CanonError::new(
                "E_CANON_UNTERM_STRING",
                "문자열이 끝나지 않았습니다.",
            ));
        }
        for (surface, value) in [
            ("줄", "\n"),
            ("칸", "\t"),
            ("앞", "\r"),
            ("따옴", "\""),
            ("역빗금", "\\"),
        ] {
            if self.match_str(surface) {
                self.advance_n(surface.chars().count());
                return Ok(value.to_string());
            }
        }
        for surface in ["반전끝", "굵게끝", "반전", "굵게", "되돌림"] {
            if self.match_str(surface) {
                self.advance_n(surface.chars().count());
                return Ok(format!("\\{}", surface));
            }
        }
        for surface in ["색", "배경"] {
            if self.match_str(surface) {
                let saved = self.pos;
                self.advance_n(surface.chars().count());
                if self.peek() == Some('{') {
                    let body = self.lex_rich_markup_body()?;
                    return Ok(format!("\\{}{{{}}}", surface, body));
                }
                self.pos = saved;
            }
        }
        let escaped = match self.peek() {
            Some('"') => "\"",
            Some('\\') => "\\",
            Some('n') => "\n",
            Some('r') => "\r",
            Some('t') => "\t",
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
        Ok(escaped.to_string())
    }

    pub(super) fn lex_rich_markup_body(&mut self) -> Result<String, CanonError> {
        if self.peek() != Some('{') {
            return Err(CanonError::new(
                "E_CANON_BAD_ESCAPE",
                "콘솔 표지는 {...} 본문이 필요합니다.",
            ));
        }
        self.bump();
        let mut body = String::new();
        while let Some(ch) = self.peek() {
            if ch == '}' {
                self.bump();
                return Ok(body);
            }
            if ch == '\r' || ch == '\n' {
                return Err(CanonError::new(
                    "E_CANON_BAD_ESCAPE",
                    "콘솔 표지 본문이 닫히지 않았습니다.",
                ));
            }
            body.push(ch);
            self.bump();
        }
        Err(CanonError::new(
            "E_CANON_BAD_ESCAPE",
            "콘솔 표지 본문이 닫히지 않았습니다.",
        ))
    }

    pub(super) fn lex_brace_block(&mut self) -> Result<String, CanonError> {
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

    pub(super) fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    pub(super) fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    pub(super) fn peek_n(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    pub(super) fn match_str(&self, token: &str) -> bool {
        for (idx, ch) in token.chars().enumerate() {
            if self.peek_n(idx) != Some(ch) {
                return false;
            }
        }
        true
    }

    pub(super) fn advance_n(&mut self, count: usize) {
        for _ in 0..count {
            if self.is_eof() {
                break;
            }
            self.bump();
        }
    }

    pub(super) fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    pub(super) fn is_line_directive_start(&self) -> bool {
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

    pub(super) fn bump(&mut self) {
        self.pos += 1;
    }
}

