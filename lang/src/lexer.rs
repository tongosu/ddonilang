// lang/src/lexer.rs
use std::fmt;
use crate::dialect::DialectConfig;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String), Integer(i64), Float(String), StringLit(String), Atom(String), Variable(String), Nuance(String),
    Pragma(String),
    TemplateBlock(String),
    FormulaBlock(String),
    KwImeumssi, KwUmjikssi, KwGallaessi, KwRelationssi, KwValueFunc, KwSam, KwHeureumssi, KwIeumssi, KwSemssi,
    KwIlttae, KwAniramyeon, KwBanbok, KwButeo, KwKkaji, KwDongan, KwDaehae, KwMeomchugi, KwDollyeojwo, KwAllyeo,
    KwHaebogo, KwGoreugi, KwMajeumyeon, KwJeonjehae, KwBojanghago, KwHaeseo, KwNeuljikeobogo,
    Josa(String), At, Question, Bang, Tilde, Colon, Equals, Arrow, RightArrow, DoubleArrow, Dot, DotDot, DotDotEq, Comma, Semicolon, Pipe,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket, Plus, PlusArrow, PlusEqual, Minus, MinusArrow, MinusEqual, Star, Slash, Percent, Caret,
    EqEq, NotEq, Lt, Gt, LtEq, GtEq, And, Or, Not, Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span { pub start: usize, pub end: usize }
impl Span { pub fn new(start: usize, end: usize) -> Self { Self { start, end } } }

#[derive(Debug, Clone)]
pub struct Token { pub kind: TokenKind, pub span: Span, pub raw: String }

pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    pending: Option<Token>,
    dialect: DialectConfig,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            pending: None,
            dialect: DialectConfig::from_source(source),
        }
    }
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() { break; }
            if self.peek_char() == Some('#') && self.is_line_directive_start() {
                tokens.push(self.read_line_pragma());
                continue;
            }
            tokens.push(self.next_token()?);
        }
        tokens.push(Token { kind: TokenKind::Eof, span: Span::new(self.pos, self.pos), raw: String::new() });
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        if let Some(token) = self.pending.take() {
            return Ok(token);
        }
        let start = self.pos;
        if let Some(token) = self.try_read_sym3_token() {
            return Ok(token);
        }
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
            '~' => {
                if self.peek_ahead(1) == Some('~') && self.peek_ahead(2) == Some('>') {
                    self.advance();
                    self.advance();
                    self.advance();
                    TokenKind::DoubleArrow
                } else {
                    return Ok(self.read_tilde_token());
                }
            },
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
                if self.peek_char() == Some('>') {
                    self.advance();
                    TokenKind::RightArrow
                } else if self.peek_char() == Some('<') && self.peek_ahead(1) == Some('-') {
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
            '%' => { self.advance(); TokenKind::Percent },
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
        let mut lexeme = text.to_string();
        if let Some(canon) = self.dialect.canonicalize_keyword(text) {
            lexeme = canon.to_string();
        }
        if let Some(kind) = keyword_kind_for(&lexeme) {
            return Ok(Token { kind, span: Span::new(start, self.pos), raw: lexeme });
        }

        let josa_list = [
            "를", "을", "가", "이", "은", "는", "에게", "의", "로", "으로", "와", "과", "에", "에서",
            "부터", "까지", "마다", "도", "만", "뿐", "밖에", "처럼", "보다", "한테",
        ];
        if josa_list.iter().any(|j| *j == lexeme.as_str()) {
            return Ok(Token { kind: TokenKind::Josa(lexeme.clone()), span: Span::new(start, self.pos), raw: lexeme });
        }
        let no_split = ["길이"];
        let has_underscore = lexeme.contains('_');
        let next_sig = self.peek_non_ws_char();
        if !has_underscore && self.peek_char() != Some(':') {
            if matches!(next_sig, Some('<') | Some('=') | Some(':') | Some('.')) || no_split.iter().any(|w| *w == lexeme.as_str()) {
                return Ok(Token { kind: TokenKind::Ident(lexeme.clone()), span: Span::new(start, self.pos), raw: lexeme });
            }
            for j in josa_list {
                if lexeme.ends_with(j) && lexeme.chars().count() > 2 {
                    let sl = lexeme.len() - j.len();
                    self.pos = start + sl;
                    let n = lexeme[..sl].to_string();
                    return Ok(Token { kind: TokenKind::Ident(n.clone()), span: Span::new(start, self.pos), raw: n });
                }
            }
        }
        Ok(Token { kind: TokenKind::Ident(lexeme.clone()), span: Span::new(start, self.pos), raw: lexeme })
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

    fn read_line_pragma(&mut self) -> Token {
        let start = self.pos;
        self.advance(); // '#'
        while let Some(ch) = self.peek_char() {
            if matches!(ch, ' ' | '\t') {
                self.advance();
            } else {
                break;
            }
        }
        let text_start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
        let text = self.source[text_start..self.pos].trim_end().to_string();
        Token {
            kind: TokenKind::Pragma(text),
            span: Span::new(start, self.pos),
            raw: self.source[start..self.pos].to_string(),
        }
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
        let text = &self.source[start..self.pos];
        let mut lexeme = text.to_string();
        if let Some(canon) = self.dialect.canonicalize_keyword(text) {
            lexeme = canon.to_string();
        }
        if let Some(kind) = keyword_kind_for(&lexeme) {
            return Ok(Token { kind, span: Span::new(start, self.pos), raw: lexeme });
        }
        Ok(Token { kind: TokenKind::Ident(lexeme.clone()), span: Span::new(start, self.pos), raw: lexeme })
    }

    fn try_read_sym3_token(&mut self) -> Option<Token> {
        let start = self.pos;
        let symbol_tokens: Vec<String> = self.dialect.symbol_tokens().to_vec();
        for token in symbol_tokens.iter() {
            if self.starts_with(token) {
                if let Some(canon) = self.dialect.canonicalize_symbol(token) {
                    if let Some(kind) = keyword_kind_for(canon) {
                        self.advance_str(token);
                        return Some(Token { kind, span: Span::new(start, self.pos), raw: token.to_string() });
                    }
                }
            }
        }
        None
    }

    fn read_tilde_token(&mut self) -> Token {
        let start = self.pos;
        self.advance();
        let tail_start = self.pos;
        let mut paren_depth = 0usize;
        while let Some(ch) = self.peek_char() {
            if ch == '(' {
                paren_depth += 1;
                self.advance();
                continue;
            }
            if ch == ')' {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
                self.advance();
                continue;
            }
            if is_josa_tail_char(ch) {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos > tail_start {
            let tail = self.source[tail_start..self.pos].to_string();
            let span = Span::new(tail_start, self.pos);
            self.pending = Some(Token { kind: TokenKind::Josa(tail.clone()), span, raw: tail });
        }
        Token { kind: TokenKind::Tilde, span: Span::new(start, start + 1), raw: "~".to_string() }
    }

    fn skip_whitespace(&mut self) {
        loop {
            while let Some(ch) = self.peek_char() {
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }
            if self.peek_char() == Some('/') && self.peek_ahead(1) == Some('/') {
                while let Some(ch) = self.peek_char() {
                    self.advance();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            break;
        }
    }

    fn is_line_directive_start(&self) -> bool {
        let line_start = self.source[..self.pos]
            .rfind('\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        self.source[line_start..self.pos]
            .chars()
            .all(|ch| matches!(ch, ' ' | '\t' | '\r' | '\u{feff}'))
    }

    fn starts_with(&self, s: &str) -> bool {
        self.source[self.pos..].starts_with(s)
    }

    fn advance_str(&mut self, s: &str) {
        self.pos += s.len();
    }

    fn peek_char(&self) -> Option<char> { self.source[self.pos..].chars().next() }
    fn peek_ahead(&self, n: usize) -> Option<char> { self.source[self.pos..].chars().nth(n) }
    fn peek_non_ws_char(&self) -> Option<char> {
        let mut idx = self.pos;
        while idx < self.source.len() {
            let ch = self.source[idx..].chars().next()?;
            if !ch.is_whitespace() {
                return Some(ch);
            }
            idx += ch.len_utf8();
        }
        None
    }
    fn advance(&mut self) { if let Some(ch) = self.peek_char() { self.pos += ch.len_utf8(); } }
    fn is_eof(&self) -> bool { self.pos >= self.source.len() }
}

fn is_josa_tail_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}

fn keyword_kind_for(canon: &str) -> Option<TokenKind> {
    match canon {
        "이름씨" => Some(TokenKind::KwImeumssi),
        "움직씨" => Some(TokenKind::KwUmjikssi),
        "값함수" => None,
        "셈씨" => Some(TokenKind::KwSemssi),
        "이음씨" => Some(TokenKind::KwIeumssi),
        "흐름씨" => Some(TokenKind::KwHeureumssi),
        "갈래씨" => Some(TokenKind::KwGallaessi),
        "관계씨" | "맞물림씨" => Some(TokenKind::KwRelationssi),
        "샘" => Some(TokenKind::KwSam),
        "그리고" => Some(TokenKind::And),
        "또는" => Some(TokenKind::Or),
        "일때" => Some(TokenKind::KwIlttae),
        "아니면" | "아니라면" => Some(TokenKind::KwAniramyeon),
        "되풀이" | "반복" => Some(TokenKind::KwBanbok),
        "동안" => Some(TokenKind::KwDongan),
        "대해" => Some(TokenKind::KwDaehae),
        "멈추기" => Some(TokenKind::KwMeomchugi),
        "되돌림" | "반환" | "돌려줘" => Some(TokenKind::KwDollyeojwo),
        "해보고" => Some(TokenKind::KwHaebogo),
        "고르기" => Some(TokenKind::KwGoreugi),
        "맞으면" | "이면" => Some(TokenKind::KwMajeumyeon),
        "바탕으로" | "전제하에" => Some(TokenKind::KwJeonjehae),
        "다짐하고" | "보장하고" => Some(TokenKind::KwBojanghago),
        "해서" => Some(TokenKind::KwHaeseo),
        "늘지켜보고" => Some(TokenKind::KwNeuljikeobogo),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct LexError { pub pos: usize, pub message: String }
impl LexError { fn new(pos: usize, message: &str) -> Self { Self { pos, message: message.to_string() } } }
impl fmt::Display for LexError { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "렉서 오류: {}", self.message) } }
impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_start_hash_becomes_pragma_token() {
        let mut lexer = Lexer::new("#그래프(y축=x)\n살림.x <- 1.\n");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(matches!(tokens[0].kind, TokenKind::Pragma(_)));
        match &tokens[0].kind {
            TokenKind::Pragma(text) => assert_eq!(text, "그래프(y축=x)"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn inline_hash_keeps_atom_token() {
        let mut lexer = Lexer::new("값 <- (#ascii) 수식{y=x}.");
        let tokens = lexer.tokenize().expect("tokenize");
        let has_ascii_atom = tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Atom(text) if text == "ascii"));
        assert!(has_ascii_atom, "expected #ascii atom token");
    }

    #[test]
    fn english_keyword_is_only_active_under_en_dialect() {
        let mut ko = Lexer::new("if 참.\n");
        let ko_tokens = ko.tokenize().expect("tokenize");
        assert!(matches!(ko_tokens[0].kind, TokenKind::Ident(ref name) if name == "if"));

        let mut en = Lexer::new("#말씨: en\nif 참.\n");
        let en_tokens = en.tokenize().expect("tokenize");
        assert!(matches!(en_tokens[1].kind, TokenKind::KwIlttae));
    }

    #[test]
    fn unsupported_dialect_keeps_english_keyword_as_ident() {
        let mut lexer = Lexer::new("#말씨: xx\nif 참.\n");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(matches!(tokens[1].kind, TokenKind::Ident(ref name) if name == "if"));
    }

    #[test]
    fn double_arrow_uses_tilde_form() {
        let mut lexer = Lexer::new("왼 ~~> 오른.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::DoubleArrow)));
        let arrow = tokens
            .iter()
            .find(|token| matches!(token.kind, TokenKind::DoubleArrow))
            .expect("double arrow");
        assert_eq!(arrow.raw, "~~>");
    }

    #[test]
    fn double_arrow_legacy_form_is_kept_for_compat() {
        let mut lexer = Lexer::new("왼 <<- 오른.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::DoubleArrow)));
    }
}
