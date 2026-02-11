// lang/src/lexer.rs
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String), Integer(i64), Float(String), StringLit(String), Atom(String), Variable(String), Nuance(String),
    TemplateBlock(String),
    FormulaBlock(String),
    KwImeumssi, KwUmjikssi, KwGallaessi, KwRelationssi, KwValueFunc, KwSam, KwHeureumssi, KwIeumssi, KwSemssi,
    KwIlttae, KwAniramyeon, KwBanbok, KwButeo, KwKkaji, KwDongan, KwDaehae, KwMeomchugi, KwDollyeojwo, KwAllyeo,
    KwHaebogo, KwGoreugi, KwMajeumyeon, KwJeonjehae, KwBojanghago, KwHaeseo, KwNeuljikeobogo,
    Josa(String), At, Question, Bang, Tilde, Colon, Equals, Arrow, DoubleArrow, Dot, DotDot, DotDotEq, Comma, Semicolon, Pipe,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket, Plus, PlusArrow, PlusEqual, Minus, MinusArrow, MinusEqual, Star, Slash, Percent, Caret,
    EqEq, NotEq, Lt, Gt, LtEq, GtEq, And, Or, Not, Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span { pub start: usize, pub end: usize }
impl Span { pub fn new(start: usize, end: usize) -> Self { Self { start, end } } }

#[derive(Debug, Clone)]
pub struct Token { pub kind: TokenKind, pub span: Span, pub raw: String }

pub struct Lexer<'a> { source: &'a str, pos: usize }

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self { Self { source, pos: 0 } }
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() { break; }
            tokens.push(self.next_token()?);
        }
        tokens.push(Token { kind: TokenKind::Eof, span: Span::new(self.pos, self.pos), raw: String::new() });
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let ch = self.peek_char().ok_or_else(|| LexError::new(self.pos, "EOF"))?;
        let kind = match ch {
            '0'..='9' => return self.read_number(),
            '"' => return self.read_string(),
            '#' => return self.read_atom(),
            '$' => return self.read_nuance(),
            '?' => {
                if self.peek_ahead(1)
                    .map(|c| matches!(c, '가'..='힣' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ'))
                    .unwrap_or(false)
                {
                    return self.read_variable();
                }
                self.advance();
                TokenKind::Question
            }
            '가'..='힣' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ' => return self.read_hangul(),
            'a'..='z' | 'A'..='Z' | '_' => return self.read_ascii(),
            ':' => { self.advance(); TokenKind::Colon },
            '=' => { self.advance(); if self.peek_char() == Some('=') { self.advance(); TokenKind::EqEq } else { TokenKind::Equals } },
            '<' => {
                self.advance();
                match self.peek_char() {
                    Some('=') => { self.advance(); TokenKind::LtEq },
                    Some('-') => { self.advance(); TokenKind::Arrow },
                    Some('<') => { self.advance(); if self.peek_char() == Some('-') { self.advance(); TokenKind::DoubleArrow } else { return Err(LexError::new(start, "<<-")); } },
                    _ => TokenKind::Lt,
                }
            },
            '>' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            },
            '@' => { self.advance(); TokenKind::At },
            '~' => { self.advance(); TokenKind::Tilde },
            '!' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::NotEq
                } else {
                    TokenKind::Bang
                }
            },
            '&' => {
                self.advance();
                if self.peek_char() == Some('&') {
                    self.advance();
                    TokenKind::And
                } else {
                    return Err(LexError::new(start, "Gate0: '&'는 예약 기호입니다"));
                }
            },
            '[' => {
                self.advance();
                TokenKind::LBracket
            }
            ']' => {
                self.advance();
                TokenKind::RBracket
            }
            '|' => {
                self.advance();
                if self.peek_char() == Some('|') {
                    self.advance();
                    TokenKind::Or
                } else {
                    TokenKind::Pipe
                }
            },
            '.' => {
                if self.peek_ahead(1) == Some('.') {
                    self.advance();
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::DotDotEq
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    self.advance();
                    TokenKind::Dot
                }
            },
            ',' => { self.advance(); TokenKind::Comma },
            '(' => { self.advance(); TokenKind::LParen },
            ')' => { self.advance(); TokenKind::RParen },
            '{' => { self.advance(); TokenKind::LBrace },
            '}' => { self.advance(); TokenKind::RBrace },
            '+' => {
                self.advance();
                if self.peek_char() == Some('<') && self.peek_ahead(1) == Some('-') {
                    self.advance();
                    self.advance();
                    TokenKind::PlusArrow
                } else if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::PlusEqual
                } else {
                    TokenKind::Plus
                }
            },
            '-' => {
                self.advance();
                if self.peek_char() == Some('<') && self.peek_ahead(1) == Some('-') {
                    self.advance();
                    self.advance();
                    TokenKind::MinusArrow
                } else if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::MinusEqual
                } else {
                    TokenKind::Minus
                }
            },
            '*' => { self.advance(); TokenKind::Star },
            '/' => { self.advance(); TokenKind::Slash },
            '^' => { self.advance(); TokenKind::Caret },
            _ => { self.advance(); return Err(LexError::new(start, "알 수 없는 문자")); }
        };
        Ok(Token { kind, span: Span::new(start, self.pos), raw: self.source[start..self.pos].to_string() })
    }

    fn read_hangul(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if matches!(ch, '가'..='힣' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') { self.advance(); } else { break; }
        }
        let text = &self.source[start..self.pos];
        if text == "글무늬" && self.peek_char() == Some('{') {
            return self.read_template_block(start);
        }
        if text == "수식" && self.peek_char() == Some('{') {
            return self.read_formula_block(start);
        }
        let josa_list = [
            "를", "을", "가", "이", "은", "는", "에게", "의", "로", "으로", "와", "과", "에", "에서",
            "부터", "까지", "마다", "도", "만", "뿐", "밖에", "처럼", "보다", "한테",
        ];
        let kind = match text {
            "이름씨" => TokenKind::KwImeumssi,
            "움직씨" => TokenKind::KwUmjikssi,
            "값함수" => TokenKind::KwValueFunc,
            "셈씨" => TokenKind::KwSemssi,
            "돌려줘" => TokenKind::KwDollyeojwo,
            "일때" => TokenKind::KwIlttae,
            "아니면" => TokenKind::KwAniramyeon,
            "아니라면" => TokenKind::KwAniramyeon,
            "반복" => TokenKind::KwBanbok,
            "동안" => TokenKind::KwDongan,
            "대해" => TokenKind::KwDaehae,
            "멈추기" => TokenKind::KwMeomchugi,
            "해보고" => TokenKind::KwHaebogo,
            "고르기" => TokenKind::KwGoreugi,
            "맞으면" => TokenKind::KwMajeumyeon,
            "바탕으로" => TokenKind::KwJeonjehae,
            "전제하에" => TokenKind::KwJeonjehae,
            "다짐하고" => TokenKind::KwBojanghago,
            "보장하고" => TokenKind::KwBojanghago,
            "해서" => TokenKind::KwHaeseo,
            "늘지켜보고" => TokenKind::KwNeuljikeobogo,
            "검사할때" => TokenKind::KwNeuljikeobogo,
            _ => {
                if josa_list.iter().any(|j| *j == text) {
                    return Ok(Token { kind: TokenKind::Josa(text.to_string()), span: Span::new(start, self.pos), raw: text.to_string() });
                }
                if self.peek_char() != Some(':') {
                    for j in josa_list {
                        if text.ends_with(j) && text.chars().count() > 2 {
                            let sl = text.len() - j.len();
                            self.pos = start + sl;
                            let n = text[..sl].to_string();
                            return Ok(Token { kind: TokenKind::Ident(n.clone()), span: Span::new(start, self.pos), raw: n });
                        }
                    }
                }
                TokenKind::Ident(text.to_string())
            }
        };
        Ok(Token { kind, span: Span::new(start, self.pos), raw: text.to_string() })
    }

    fn read_number(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() { self.advance(); }
            else if ch == '.' && self.peek_ahead(1).map(|c| c.is_ascii_digit()).unwrap_or(false) { self.advance(); }
            else { break; }
        }
        let raw = &self.source[start..self.pos];
        let kind = if raw.contains('.') { TokenKind::Float(raw.to_string()) } else { TokenKind::Integer(raw.parse().unwrap_or(0)) };
        Ok(Token { kind, span: Span::new(start, self.pos), raw: raw.to_string() })
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        let start = self.pos; self.advance();
        let mut c = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == '"' { self.advance(); break; }
            if ch == '\\' {
                self.advance();
                match self.peek_char() {
                    Some('n') => { c.push('\n'); self.advance(); }
                    Some('r') => { c.push('\r'); self.advance(); }
                    Some('t') => { c.push('\t'); self.advance(); }
                    Some('"') => { c.push('"'); self.advance(); }
                    Some('\\') => { c.push('\\'); self.advance(); }
                    Some(other) => { c.push(other); self.advance(); }
                    None => return Err(LexError::new(self.pos, "문자열 종료")),
                }
                continue;
            }
            c.push(ch);
            self.advance();
        }
        Ok(Token { kind: TokenKind::StringLit(c), span: Span::new(start, self.pos), raw: self.source[start..self.pos].to_string() })
    }

    fn read_atom(&mut self) -> Result<Token, LexError> {
        let start = self.pos; self.advance();
        while let Some(ch) = self.peek_char() { if matches!(ch, '가'..='힣' | 'a'..='z' | '0'..='9' | '_') { self.advance(); } else { break; } }
        Ok(Token { kind: TokenKind::Atom(self.source[start+1..self.pos].to_string()), span: Span::new(start, self.pos), raw: self.source[start..self.pos].to_string() })
    }

    fn read_variable(&mut self) -> Result<Token, LexError> {
        let start = self.pos; self.advance();
        while let Some(ch) = self.peek_char() { if matches!(ch, '가'..='힣' | 'a'..='z' | '0'..='9' | '_') { self.advance(); } else { break; } }
        Ok(Token { kind: TokenKind::Variable(self.source[start+1..self.pos].to_string()), span: Span::new(start, self.pos), raw: self.source[start..self.pos].to_string() })
    }

    fn read_nuance(&mut self) -> Result<Token, LexError> {
        let start = self.pos; self.advance();
        let mut saw = false;
        while let Some(ch) = self.peek_char() {
            if matches!(ch, '가'..='힣' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ') {
                self.advance();
                saw = true;
            } else {
                break;
            }
        }
        if !saw {
            return Err(LexError::new(start, "Gate0: '$' 뒤에는 뉘앙스 토큰이 필요합니다"));
        }
        Ok(Token {
            kind: TokenKind::Nuance(self.source[start + 1..self.pos].to_string()),
            span: Span::new(start, self.pos),
            raw: self.source[start..self.pos].to_string(),
        })
    }

    fn read_template_block(&mut self, start: usize) -> Result<Token, LexError> {
        self.advance(); // consume '{'
        let mut depth = 1usize;
        let mut body = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == '{' {
                if self.peek_ahead(1) == Some('{') {
                    body.push('{');
                    body.push('{');
                    self.advance();
                    self.advance();
                    continue;
                }
                depth += 1;
                body.push(ch);
                self.advance();
                continue;
            }
            if ch == '}' {
                if self.peek_ahead(1) == Some('}') {
                    body.push('}');
                    body.push('}');
                    self.advance();
                    self.advance();
                    continue;
                }
                depth = depth.saturating_sub(1);
                self.advance();
                if depth == 0 {
                    let end = self.pos;
                    return Ok(Token {
                        kind: TokenKind::TemplateBlock(body),
                        span: Span::new(start, end),
                        raw: self.source[start..end].to_string(),
                    });
                }
                body.push('}');
                continue;
            }
            body.push(ch);
            self.advance();
        }
        Err(LexError::new(start, "글무늬 블록이 닫히지 않았습니다"))
    }

    fn read_formula_block(&mut self, start: usize) -> Result<Token, LexError> {
        self.advance(); // consume '{'
        let mut depth = 1usize;
        let mut body = String::new();
        while let Some(ch) = self.peek_char() {
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
                    let end = self.pos;
                    return Ok(Token {
                        kind: TokenKind::FormulaBlock(body),
                        span: Span::new(start, end),
                        raw: self.source[start..end].to_string(),
                    });
                }
                body.push('}');
                continue;
            }
            body.push(ch);
            self.advance();
        }
        Err(LexError::new(start, "수식 블록이 닫히지 않았습니다"))
    }

    fn read_ascii(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() { if ch.is_ascii_alphanumeric() || ch == '_' { self.advance(); } else { break; } }
        Ok(Token { kind: TokenKind::Ident(self.source[start..self.pos].to_string()), span: Span::new(start, self.pos), raw: self.source[start..self.pos].to_string() })
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() { if ch.is_whitespace() { self.advance(); } else { break; } }
    }

    fn peek_char(&self) -> Option<char> { self.source[self.pos..].chars().next() }
    fn peek_ahead(&self, n: usize) -> Option<char> { self.source[self.pos..].chars().nth(n) }
    fn advance(&mut self) { if let Some(ch) = self.peek_char() { self.pos += ch.len_utf8(); } }
    fn is_eof(&self) -> bool { self.pos >= self.source.len() }
}

#[derive(Debug, Clone)]
pub struct LexError { pub pos: usize, pub message: String }
impl LexError { fn new(pos: usize, message: &str) -> Self { Self { pos, message: message.to_string() } } }
impl fmt::Display for LexError { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "렉서 오류: {}", self.message) } }
impl std::error::Error for LexError {}
