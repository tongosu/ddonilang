// lang/src/lexer.rs
use crate::dialect::DialectConfig;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Integer(i64),
    Float(String),
    StringLit(String),
    BytesLit(Vec<u8>),
    Atom(String),
    Variable(String),
    Nuance(String),
    BranchRole(String),
    Pragma(String),
    AssertionBlock(String),
    TemplateBlock(String),
    FormulaBlock(String),
    StateMachineBlock(String),
    BogeaMadangBlock(String),
    BogeaJangmyeonBlock(String),
    JjaimBlock(String),
    KwImeumssi,
    KwUmjikssi,
    KwGallaessi,
    KwRelationssi,
    KwValueFunc,
    KwSam,
    KwHeureumssi,
    KwIeumssi,
    KwSemssi,
    KwManyak,
    KwIramyeon,
    KwIlttae,
    KwAniramyeon,
    KwBanbok,
    KwButeo,
    KwKkaji,
    KwDongan,
    KwDaehae,
    KwMeomchugi,
    KwGeonneottwigi,
    KwDollyeojwo,
    KwTolabogi,
    KwAllyeo,
    KwHaebogo,
    KwGoreugi,
    KwMajeumyeon,
    KwJeonjehae,
    KwBojanghago,
    KwHaeseo,
    KwNeuljikeobogo,
    KwBoyeojugi,
    KwBeat,
    KwReserve,
    Josa(String),
    At,
    Question,
    Bang,
    Tilde,
    Colon,
    Define,
    Equals,
    Arrow,
    RightArrow,
    TypeArrow,
    AsyncLeftArrow,
    AsyncRightArrow,
    RendezvousLeftArrow,
    PersistentDependencyArrow,
    /// Successor rendezvous `~~>` token. Explicit migration-evidence mode may
    /// reinterpret the same historical bytes as `HistoricalAsync`.
    DoubleArrow,
    Dot,
    DotDot,
    DotDotEq,
    Comma,
    Semicolon,
    Pipe,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Plus,
    PlusArrow,
    PlusEqual,
    Minus,
    MinusArrow,
    MinusEqual,
    Star,
    Slash,
    Percent,
    Caret,
    RelationEq,
    EqEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Not,
    Eof,
}

#[cfg(test)]
mod r20_korean_header_tests {
    use super::*;
    #[test]
    fn r20_korean_header_keeps_original_utf8_spans_and_default_lexer() {
        let raw="#!한국어\r\n// 日本語 위치\r\n값 <- 1.\r\n";
        let (mut lexer,header)=Lexer::with_korean_source_header(raw).unwrap();
        assert_eq!(header,Some("#!한국어"));let tokens=lexer.tokenize().unwrap();
        assert_eq!(tokens[0].span.start,raw.find("값").unwrap());
        assert_eq!(&raw[tokens[0].span.start..tokens[0].span.end],tokens[0].raw);
        assert_eq!(tokens.last().unwrap().span.end,raw.len());
        assert!(matches!(Lexer::new(raw).tokenize().unwrap()[0].kind,TokenKind::Pragma(_)));
    }
    #[test]
    fn r20_header_unknown_duplicate_and_nonfirst_are_not_silently_accepted() {
        for raw in ["#!日本語\n값 <- 1.","#!韓國語\n값 <- 1.","#!한국어 \n값 <- 1."] {
            assert!(Lexer::with_korean_source_header(raw).is_err());
        }
        for raw in [" #!한국어\n값 <- 1.","\n#!한국어\n값 <- 1.","#!한국어\n#!한국어\n값 <- 1."] {
            let (mut lexer,_)=Lexer::with_korean_source_header(raw).unwrap();
            let error=lexer.tokenize().unwrap_err();assert!(error.message.contains("HEADER_NOT_FIRST"));
            assert!(error.pos>0);
        }
    }
    #[test]
    fn r20_header_spelling_in_strings_and_comments_is_not_metadata() {
        let raw="// #!한국어\n글 <- \"#!한국어\".\n";
        let (mut lexer,header)=Lexer::with_korean_source_header(raw).unwrap();assert_eq!(header,None);
        let tokens=lexer.tokenize().unwrap();
        assert!(!tokens.iter().any(|t|matches!(t.kind,TokenKind::Pragma(_))));
        assert!(tokens.iter().any(|t|t.raw=="\"#!한국어\""));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub raw: String,
}

pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    pending: Option<Token>,
    dialect: DialectConfig,
    public_callable_projection: Option<crate::ddl::RegisteredSuccessorProfile>,
    public_callable_declaration: bool,
    first_callable_registered: bool,
    first_callable_projection: Option<crate::ddl::CheckedFirstCallableProfile>,
    korean_header_aware: bool,
    j1_profile: Option<crate::ddl::CheckedJ1Profile>,
    rejected_j1_word_end: Option<usize>,
}

impl<'a> Lexer<'a> {
    /// Internal mapped-source fragments only. Public source admission still
    /// requires with_j1_profile's exact byte-zero header.
    pub fn with_j1_linked_source(source:&'a str,profile:crate::ddl::CheckedJ1Profile)->Self{
        let mut lexer=Self::new(source);lexer.j1_profile=Some(profile);lexer
    }
    pub fn with_j1_profile(source:&'a str,profile:crate::ddl::CheckedJ1Profile)->Result<Self,LexError>{
        if source.lines().next()!=Some("#!日本語"){return Err(LexError::new(0,"E_J1_HEADER_PROFILE_MISMATCH"));}
        let mut lexer=Self::new(source);lexer.j1_profile=Some(profile);
        lexer.pos=source.find('\n').map_or(source.len(),|at|at+1);Ok(lexer)
    }

    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            pending: None,
            dialect: DialectConfig::from_source(source),
            public_callable_projection: None,public_callable_declaration:false,
            first_callable_registered: false,
            first_callable_projection: None,
            korean_header_aware: false,
            j1_profile: None,
            rejected_j1_word_end: None,
        }
    }
    /// Existing Korean source selection, not a new profile or execution grant.
    /// Consume only the exact byte-zero registered header with the ordinary
    /// cursor. Original source and all later token spans remain unchanged.
    pub fn with_korean_source_header(source: &'a str) -> Result<(Self, Option<&'static str>), LexError> {
        let mut lexer=Self::new(source);
        lexer.korean_header_aware=true;
        let first=source.lines().next().unwrap_or("");
        if first=="#!한국어" {
            let end=source.find('\n').map_or(source.len(),|end|end+1);
            while lexer.pos<end {lexer.advance();}
            Ok((lexer,Some("#!한국어")))
        } else if source.starts_with("#!") {
            Err(LexError::new(0,"E_MULTILANG_HEADER_UNSUPPORTED: 이 공통 원문 경로는 등록된 한국어 머리표만 받습니다"))
        } else {Ok((lexer,None))}
    }

    /// Explicit declaration-side context, never inferred from locale/header.
    /// The original binding must come from the common public-declaration path.
    pub fn with_first_callable_registration(
        source: &'a str,
        _original: &crate::ddl::FirstCallableJaDeclarationBinding<'_>,
    ) -> Result<Self, LexError> {
        let header = source.split('\n').next().unwrap_or("").trim_end_matches('\r');
        if header != "#!日本語" {
            return Err(LexError::new(0, "E_MULTILANG_HEADER_REQUIRED: 登録された #!日本語 が必要です"));
        }
        let mut lexer = Self::new(source);
        lexer.first_callable_registered = true;
        lexer.pos = source.find('\n').map_or(source.len(), |end| end + 1);
        Ok(lexer)
    }
    /// Existing Korean source grammar with only the exact registered foreign
    /// call surface added by a checked registry owner. A header is not authority.
    pub(crate) fn with_first_callable_projection(
        source:&'a str,_original:&crate::ddl::FirstCallableJaDeclarationBinding<'_>,
        profile:&crate::ddl::CheckedFirstCallableProfile,
    )->Result<Self,LexError>{
        let mut lexer=Self::new(source);
        lexer.first_callable_projection=Some(profile.clone());
        Ok(lexer)
    }
    pub(crate) fn with_public_callable_projection(source:&'a str,profile:&crate::ddl::RegisteredSuccessorProfile,declaration:bool)->Result<Self,LexError>{
        let mut lexer=Self::new(source);
        if declaration {
            if source.lines().next()!=Some("#!日本語"){return Err(LexError::new(0,"E_MULTILANG_HEADER_REQUIRED"));}
            lexer.pos=source.find('\n').map_or(source.len(),|end|end+1);
        }else if source.starts_with("#!"){
            return Err(LexError::new(0,"E_MULTILANG_SUCCESSOR_HEADER_SCOPE: 실행 틀은 기존 한국어 문법과 명시 옮김을 사용하세요"));
        }
        lexer.public_callable_projection=Some(profile.clone());lexer.public_callable_declaration=declaration;Ok(lexer)
    }

    #[doc(hidden)]
    pub fn tokenize_j1_source(
        source: &'a str,
        profile: crate::ddl::CheckedJ1Profile,
    ) -> Result<Vec<Token>, (LexError, Span)> {
        let scalar_end = |pos: usize| {
            pos + source.get(pos..).and_then(|tail| tail.chars().next()).map_or(0, char::len_utf8)
        };
        let mut lexer = Self::with_j1_profile(source, profile).map_err(|error| {
            let span = Span::new(error.pos, scalar_end(error.pos));
            (error, span)
        })?;
        lexer.tokenize().map_err(|error| {
            let end = lexer.rejected_j1_word_end.unwrap_or_else(|| scalar_end(error.pos));
            let span = Span::new(error.pos, end);
            (error, span)
        })
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        self.rejected_j1_word_end = None;
        // F5-GA[LEXER-EOF]: 성공 반환에는 항상 EOF 토큰이 붙는다. 토큰 오류는 ?로 반환되어 불완전한 벡터를 소비자에게 주지 않는다.
        let mut tokens = Vec::new();
        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }
            if self.peek_char() == Some('#') && self.is_line_directive_start() {
                if self.korean_header_aware && self.source[self.pos..].starts_with("#!") {
                    return Err(LexError::new(self.pos,"E_MULTILANG_HEADER_NOT_FIRST: 언어 머리표는 byte zero에서 한 번만 지정하세요"));
                }
                if self.first_callable_registered {
                    return Err(LexError::new(self.pos, "E_MULTILANG_DUPLICATE_HEADER: 先頭以外の言語ヘッダーは使えません"));
                }
                tokens.push(self.read_line_pragma());
                continue;
            }
            tokens.push(self.next_token()?);
        }
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(self.pos, self.pos),
            raw: String::new(),
        });
        Ok(tokens)
    }

    // Source-bound checkpoints for the common module-prefix owner. No caller
    // can assign a position; failed lookahead leaves the real cursor intact.
    pub(crate) fn prefix_checkpoint(&self)->Self {
        Self{source:self.source,pos:self.pos,pending:self.pending.clone(),korean_header_aware:self.korean_header_aware,j1_profile:self.j1_profile,
            rejected_j1_word_end:None,
            public_callable_projection:self.public_callable_projection.clone(),public_callable_declaration:self.public_callable_declaration,
            dialect:self.dialect.clone(),first_callable_registered:self.first_callable_registered,first_callable_projection:self.first_callable_projection.clone()}
    }
    pub(crate) fn prefix_token_start(&mut self)->(usize,Option<char>){
        self.skip_whitespace();(self.pos,self.peek_char())
    }
    pub(crate) fn next_common_prefix_token(&mut self)->Result<Token,LexError>{
        self.skip_whitespace();
        if self.is_eof(){return Ok(Token{kind:TokenKind::Eof,span:Span::new(self.pos,self.pos),raw:String::new()});}
        if self.peek_char()==Some('#') && self.is_line_directive_start(){
            if self.first_callable_registered{return Err(LexError::new(self.pos,"E_MULTILANG_DUPLICATE_HEADER: 先頭以外の言語ヘッダーは使えません"));}
            return Ok(self.read_line_pragma());
        }
        let public_projection=self.public_callable_projection.take();
        let public_declaration=self.public_callable_declaration;self.public_callable_declaration=false;
        let registered=self.first_callable_registered;
        let projection=self.first_callable_projection.take();
        self.first_callable_registered=false;
        let result=self.next_token();
        self.public_callable_projection=public_projection;self.public_callable_declaration=public_declaration;
        self.first_callable_registered=registered;
        self.first_callable_projection=projection;
        result
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        // Only the explicitly registered Japanese surface uses this rejection.
        // Fullwidth ASCII and CJK punctuation are syntax mistakes, not names.
        let is_fullwidth_syntax = |c: char| matches!(c,
            '\u{ff01}'..='\u{ff5e}' | '、' | '。' | '〈' | '〉' | '《' | '》'
            | '「' | '」' | '『' | '』' | '【' | '】' | '〔' | '〕'
            | '〖' | '〗' | '〘' | '〙' | '〚' | '〛' | '・');
        if let Some(profile)=self.j1_profile {
            let start=self.pos;let ch=self.peek_char().ok_or_else(||LexError::new(start,"EOF"))?;
            if is_fullwidth_syntax(ch){return Err(LexError::new(start,"E_MULTILANG_FULLWIDTH_SYNTAX: ASCII 구문 기호가 필요합니다"));}
            if ch=='#'&&self.source[start..].starts_with("#!"){return Err(LexError::new(start,"E_J1_DUPLICATE_HEADER"));}
            if !ch.is_ascii()&&!ch.is_whitespace()&&!matches!(ch,'가'..='힣'|'ㄱ'..='ㅎ'|'ㅏ'..='ㅣ'){
                while let Some(next)=self.peek_char(){
                    if is_fullwidth_syntax(next){return Err(LexError::new(self.pos,"E_MULTILANG_FULLWIDTH_SYNTAX: ASCII 구문 기호가 필요합니다"));}
                    if next.is_whitespace()||next.is_ascii()&&!(next.is_alphanumeric()||next=='_'){break;}
                    self.advance();
                }
                let raw=&self.source[start..self.pos];
                if !profile.admits_word(raw) {
                    self.rejected_j1_word_end = Some(self.pos);
                    return Err(LexError::new(start, "E_J1_UNREGISTERED_SURFACE"));
                }
                let kind=if matches!(raw,"に"|"を"){TokenKind::Josa(raw.into())}else{TokenKind::Ident(raw.into())};
                return Ok(Token{kind,span:Span::new(start,self.pos),raw:raw.into()});
            }
        }
        if let Some(profile)=self.public_callable_projection.clone(){
            let start=self.pos;let ch=self.peek_char().ok_or_else(||LexError::new(start,"EOF"))?;
            if is_fullwidth_syntax(ch){return Err(LexError::new(start,"E_MULTILANG_FULLWIDTH_SYNTAX: ASCII 숫자·구문 기호를 사용하세요"));}
            let foreign=!ch.is_ascii()&&!ch.is_whitespace()&&!matches!(ch,'가'..='힣'|'ㄱ'..='ㅎ'|'ㅏ'..='ㅣ');
            if foreign||self.public_callable_declaration&&(ch.is_alphabetic()||ch=='_'){
                while let Some(next)=self.peek_char(){
                    if is_fullwidth_syntax(next){return Err(LexError::new(self.pos,"E_MULTILANG_FULLWIDTH_SYNTAX: ASCII 숫자·구문 기호를 사용하세요"));}
                    if next.is_whitespace()||next.is_ascii()&&!(self.public_callable_declaration&&(next.is_alphanumeric()||next=='_')){break;}
                    self.advance();
                }
                let raw=&self.source[start..self.pos];
                let registered=profile.surfaces().iter().any(|row|row.surface==raw||row.pins.iter().any(|pin|pin.surface==raw));
                let kind=if raw=="関数"&&self.public_callable_declaration{TokenKind::KwSemssi}
                    else if matches!(raw,"に"|"を"){TokenKind::Josa(raw.into())}
                    else if raw=="有理数"||registered{TokenKind::Ident(raw.into())}
                    else{return Err(LexError::new(start,"E_MULTILANG_UNREGISTERED_SURFACE"));};
                return Ok(Token{kind,span:Span::new(start,self.pos),raw:raw.into()});
            }
        }

        if self.first_callable_projection.is_some() {
            let start=self.pos;
            let ch=self.peek_char().ok_or_else(||LexError::new(start,"EOF"))?;
            if is_fullwidth_syntax(ch) {
                return Err(LexError::new(start, &format!("E_MULTILANG_FULLWIDTH_SYNTAX: ASCII の数字・構文記号を使ってください (encoding=utf-8-byte byte={}..{})", start, start + ch.len_utf8())));
            }
            if !ch.is_ascii() && !ch.is_whitespace() && !matches!(ch,'가'..='힣'|'ㄱ'..='ㅎ'|'ㅏ'..='ㅣ') {
                while let Some(next)=self.peek_char(){
                    if is_fullwidth_syntax(next) {
                        return Err(LexError::new(self.pos, &format!("E_MULTILANG_FULLWIDTH_SYNTAX: ASCII の数字・構文記号を使ってください (encoding=utf-8-byte byte={}..{})", self.pos, self.pos + next.len_utf8())));
                    }
                    if next.is_ascii()||next.is_whitespace()||matches!(next,'가'..='힣'|'ㄱ'..='ㅎ'|'ㅏ'..='ㅣ'){break;}
                    self.advance();
                }
                let raw=&self.source[start..self.pos];
                let r=crate::ddl::FIRST_CALLABLE_JA_REGISTRATION;
                let kind=if r.pin_particles.contains(&raw){TokenKind::Josa(raw.into())}
                    else if raw==r.callable_name||raw==r.rational_type_name||r.pin_names.contains(&raw){TokenKind::Ident(raw.into())}
                    else{return Err(LexError::new(start,"E_MULTILANG_UNREGISTERED_SURFACE: この範囲では未登録の表面です"));};
                return Ok(Token{kind,span:Span::new(start,self.pos),raw:raw.into()});
            }
        }
        if self.first_callable_registered {
            let start = self.pos;
            let ch = self.peek_char().ok_or_else(|| LexError::new(start, "EOF"))?;
            if is_fullwidth_syntax(ch) {
                return Err(LexError::new(start, &format!("E_MULTILANG_FULLWIDTH_SYNTAX: ASCII の数字・構文記号を使ってください (encoding=utf-8-byte byte={}..{})", start, start + ch.len_utf8())));
            }
            if ch.is_alphabetic() || ch == '_' || (!ch.is_ascii() && !ch.is_whitespace()) {
                while let Some(next) = self.peek_char() {
                    if is_fullwidth_syntax(next) {
                        return Err(LexError::new(self.pos, &format!("E_MULTILANG_FULLWIDTH_SYNTAX: ASCII の数字・構文記号を使ってください (encoding=utf-8-byte byte={}..{})", self.pos, self.pos + next.len_utf8())));
                    }
                    if next.is_ascii() && !(next.is_alphanumeric() || next == '_') || next.is_whitespace() {
                        break;
                    }
                    self.advance();
                }
                let raw = &self.source[start..self.pos];
                let registry = crate::ddl::FIRST_CALLABLE_JA_REGISTRATION;
                let kind = if raw == registry.callable_kind_name {
                    TokenKind::KwSemssi
                } else if registry.pin_particles.contains(&raw) {
                    TokenKind::Josa(raw.to_owned())
                } else if raw == registry.callable_name || raw == registry.rational_type_name
                    || registry.pin_names.contains(&raw)
                {
                    TokenKind::Ident(raw.to_owned())
                } else {
                    return Err(LexError::new(start, "E_MULTILANG_UNREGISTERED_SURFACE: この範囲では未登録の表面です"));
                };
                return Ok(Token { kind, span: Span::new(start, self.pos), raw: raw.to_owned() });
            }
            if ch == '#' {
                return Err(LexError::new(start, "E_MULTILANG_DUPLICATE_HEADER: 先頭以外の言語ヘッダーは使えません"));
            }
        }
        if let Some(token) = self.pending.take() {
            return Ok(token);
        }
        let start = self.pos;
        if let Some(token) = self.try_read_sym3_token() {
            return Ok(token);
        }
        let ch = self
            .peek_char()
            .ok_or_else(|| LexError::new(self.pos, "EOF"))?;
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
            ':' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::Define
                } else {
                    TokenKind::Colon
                }
            }
            '=' => {
                self.advance();
                if self.peek_char() == Some(':') && self.peek_ahead(1) == Some('=') {
                    self.advance();
                    self.advance();
                    TokenKind::RelationEq
                } else if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::EqEq
                } else {
                    TokenKind::Equals
                }
            }
            '<' => {
                self.advance();
                match self.peek_char() {
                    Some('=') => {
                        self.advance();
                        TokenKind::LtEq
                    }
                    Some('-') => {
                        self.advance();
                        TokenKind::Arrow
                    }
                    Some('<') => {
                        self.advance();
                        if self.peek_char() == Some('-') {
                            self.advance();
                            TokenKind::PersistentDependencyArrow
                        } else {
                            return Err(LexError::new(start, "<<-"));
                        }
                    }
                    Some('~') => {
                        self.advance();
                        if self.peek_char() == Some('~') {
                            self.advance();
                            TokenKind::RendezvousLeftArrow
                        } else {
                            TokenKind::AsyncLeftArrow
                        }
                    }
                    _ => TokenKind::Lt,
                }
            }
            '>' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '@' => {
                self.advance();
                TokenKind::At
            }
            '~' => {
                if self.peek_ahead(1) == Some('~') && self.peek_ahead(2) == Some('>') {
                    self.advance();
                    self.advance();
                    self.advance();
                    TokenKind::DoubleArrow
                } else if self.peek_ahead(1) == Some('>') {
                    self.advance();
                    self.advance();
                    TokenKind::AsyncRightArrow
                } else {
                    return Ok(self.read_tilde_token());
                }
            }
            '!' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::NotEq
                } else {
                    TokenKind::Bang
                }
            }
            '&' => {
                self.advance();
                if self.peek_char() == Some('&') {
                    self.advance();
                    TokenKind::And
                } else {
                    return Err(LexError::new(start, "Gate0: '&'는 예약 기호입니다"));
                }
            }
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
            }
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
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            '(' => {
                self.advance();
                TokenKind::LParen
            }
            ')' => {
                self.advance();
                TokenKind::RParen
            }
            '{' => {
                self.advance();
                TokenKind::LBrace
            }
            '}' => {
                self.advance();
                TokenKind::RBrace
            }
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
            }
            '-' => {
                self.advance();
                if self.peek_char() == Some('-') && self.peek_ahead(1) == Some('>') {
                    self.advance();
                    self.advance();
                    TokenKind::TypeArrow
                } else if self.peek_char() == Some('>') {
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
            }
            '*' => {
                self.advance();
                TokenKind::Star
            }
            '/' => {
                self.advance();
                TokenKind::Slash
            }
            '%' => {
                self.advance();
                TokenKind::Percent
            }
            '^' => {
                self.advance();
                TokenKind::Caret
            }
            _ => {
                self.advance();
                return Err(LexError::new(start, "알 수 없는 문자"));
            }
        };
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
            raw: self.source[start..self.pos].to_string(),
        })
    }

    fn read_hangul(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if matches!(ch, '가'..='힣' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
            {
                self.advance();
            } else {
                break;
            }
        }
        let text = &self.source[start..self.pos];
        if text == "글무늬" {
            let checkpoint = self.pos;
            self.skip_inline_whitespace();
            if self.peek_char() == Some('{') {
                return self.read_template_block(start);
            }
            self.pos = checkpoint;
        }
        if text == "바이트열" {
            let checkpoint = self.pos;
            self.skip_inline_whitespace();
            if self.peek_char() == Some('{') {
                return self.read_bytes_literal(start);
            }
            self.pos = checkpoint;
        }
        if text == "수식" {
            let checkpoint = self.pos;
            self.skip_inline_whitespace();
            if self.peek_char() == Some('{') {
                return self.read_formula_block(start);
            }
            self.pos = checkpoint;
        }
        if text == "세움" || text == "세움씨" {
            let checkpoint = self.pos;
            self.skip_inline_whitespace();
            if self.peek_char() == Some('{') {
                return self.read_assertion_block(start);
            }
            self.pos = checkpoint;
        }
        if text == "상태머신" {
            let checkpoint = self.pos;
            self.skip_inline_whitespace();
            if self.peek_char() == Some('{') {
                return self.read_state_machine_block(start);
            }
            self.pos = checkpoint;
        }
        if text == "보개마당" {
            let checkpoint = self.pos;
            self.skip_inline_whitespace();
            if self.peek_char() == Some('{') {
                return self.read_bogae_madang_block(start, false);
            }
            self.pos = checkpoint;
        }
        if text == "보개장면" {
            let checkpoint = self.pos;
            self.skip_inline_whitespace();
            if self.peek_char() == Some('{') {
                return self.read_bogae_madang_block(start, true);
            }
            self.pos = checkpoint;
        }
        if text == "짜임" || text == "구성" {
            let checkpoint = self.pos;
            self.skip_inline_whitespace();
            if self.peek_char() == Some('{') {
                return self.read_jjaim_block(start);
            }
            self.pos = checkpoint;
        }
        let mut lexeme = text.to_string();
        if let Some(canon) = self.dialect.canonicalize_keyword(text) {
            lexeme = canon.to_string();
        }
        if lexeme == "입력키" || lexeme == "찾기" || lexeme == "글바꾸기" || lexeme == "충돌"
        {
            if let Some(suffix) = self.peek_char() {
                let can_use_suffix = matches!(suffix, '?' | '!')
                    && !(suffix == '?' && self.peek_ahead(1) == Some('?'))
                    && !self.peek_ahead(1).map(is_ident_continue).unwrap_or(false)
                    && !(lexeme == "찾기" && suffix == '!')
                    && !(lexeme == "글바꾸기" && suffix == '?')
                    && !(lexeme == "충돌" && suffix == '!');
                if can_use_suffix {
                    lexeme.push(suffix);
                    self.advance();
                }
            }
        }
        if let Some(kind) = keyword_kind_for(&lexeme) {
            return Ok(Token {
                kind,
                span: Span::new(start, self.pos),
                raw: lexeme,
            });
        }
        self.append_prime_derivative_suffix(&mut lexeme);

        let josa_list = [
            "에게서",
            "를",
            "을",
            "가",
            "이",
            "은",
            "는",
            "에게",
            "의",
            "로",
            "으로",
            "와",
            "과",
            "에",
            "에서",
            "부터",
            "까지",
            "마다",
            "도",
            "만",
            "뿐",
            "밖에",
            "처럼",
            "보다",
            "한테",
        ];
        if josa_list.iter().any(|j| *j == lexeme.as_str()) {
            return Ok(Token {
                kind: TokenKind::Josa(lexeme.clone()),
                span: Span::new(start, self.pos),
                raw: lexeme,
            });
        }
        let no_split = ["길이", "처음으로", "다음으로"];
        let has_underscore = lexeme.contains('_');
        let next_sig = self.peek_non_ws_char();
        let next_is_postfix_keyword = self.next_non_ws_starts_with("보여주기")
            || self.next_non_ws_starts_with("돌아보기")
            || self.next_non_ws_starts_with("돌려줘");
        if !has_underscore && !lexeme.contains('\'') && self.peek_char() != Some(':') {
            if matches!(
                next_sig,
                Some('<')
                    | Some('>')
                    | Some('=')
                    | Some(':')
                    | Some('.')
                    | Some('+')
                    | Some('-')
                    | Some('*')
                    | Some('/')
                    | Some('%')
                    | Some(')')
                    | Some(']')
                    | Some('}')
                    | Some(',')
                    | Some('|')
            ) || self.next_non_ws_starts_with("!=") || next_is_postfix_keyword
                || no_split.iter().any(|w| *w == lexeme.as_str())
            {
                return Ok(Token {
                    kind: TokenKind::Ident(lexeme.clone()),
                    span: Span::new(start, self.pos),
                    raw: lexeme,
                });
            }
            for j in josa_list {
                if lexeme.ends_with(j) && lexeme.chars().count() > 2 {
                    let sl = lexeme.len() - j.len();
                    self.pos = start + sl;
                    let n = lexeme[..sl].to_string();
                    return Ok(Token {
                        kind: TokenKind::Ident(n.clone()),
                        span: Span::new(start, self.pos),
                        raw: n,
                    });
                }
            }
        }
        Ok(Token {
            kind: TokenKind::Ident(lexeme.clone()),
            span: Span::new(start, self.pos),
            raw: lexeme,
        })
    }

    fn read_bytes_literal(&mut self, start: usize) -> Result<Token, LexError> {
        self.advance();
        self.skip_whitespace();
        let mut bytes = Vec::new();
        if self.peek_char() != Some('}') {
            loop {
                let item_start = self.pos;
                if self.peek_char() != Some('0') || self.peek_ahead(1) != Some('x') {
                    return Err(LexError::new(item_start, "E_BYTES_LITERAL_INVALID: 0x와 두 자리 16진수가 필요합니다"));
                }
                self.advance(); self.advance();
                let mut byte = 0u8;
                for _ in 0..2 {
                    let Some(digit) = self.peek_char().filter(|c| c.is_ascii_hexdigit()) else {
                        return Err(LexError::new(item_start, "E_BYTES_LITERAL_INVALID: 두 자리 16진수가 필요합니다"));
                    };
                    // F5-GA[BYTE-HEX]: 같은 digit을 ASCII 16진수로 걸렀으므로 변환은 Some이다. 두 자리 누적값도 255를 넘지 않는다.
                    byte = byte * 16 + digit.to_digit(16).unwrap() as u8;
                    self.advance();
                }
                bytes.push(byte);
                self.skip_whitespace();
                if self.peek_char() == Some('}') { break; }
                if self.peek_char() != Some(',') {
                    return Err(LexError::new(self.pos, "E_BYTES_LITERAL_INVALID: 쉼표 또는 닫는 중괄호가 필요합니다"));
                }
                self.advance(); self.skip_whitespace();
            }
        }
        if self.peek_char() != Some('}') {
            return Err(LexError::new(start, "E_BYTES_LITERAL_INVALID: 바이트열이 닫히지 않았습니다"));
        }
        self.advance();
        Ok(Token { kind: TokenKind::BytesLit(bytes), span: Span::new(start, self.pos), raw: self.source[start..self.pos].into() })
    }

    fn read_number(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.'
                && self
                    .peek_ahead(1)
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                self.advance();
            } else {
                break;
            }
        }
        let raw = &self.source[start..self.pos];
        let kind = if raw.contains('.') {
            TokenKind::Float(raw.to_string())
        } else {
            TokenKind::Integer(raw.parse().map_err(|_| {
                LexError::new(start, "E_INTEGER_LITERAL_RANGE: 정수 리터럴 범위 초과")
            })?)
        };
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
            raw: raw.to_string(),
        })
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        self.advance();
        let mut c = String::new();
        let mut closed = false;
        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                self.advance();
                closed = true;
                break;
            }
            if ch == '\\' {
                self.advance();
                c.push_str(&self.read_string_escape(start)?);
                continue;
            }
            c.push(ch);
            self.advance();
        }
        if !closed {
            return Err(LexError::new(start, "문자열이 닫히지 않았습니다"));
        }
        Ok(Token {
            kind: TokenKind::StringLit(c),
            span: Span::new(start, self.pos),
            raw: self.source[start..self.pos].to_string(),
        })
    }

    fn read_string_escape(&mut self, start: usize) -> Result<String, LexError> {
        if self.is_eof() {
            return Err(LexError::new(self.pos, "문자열 종료"));
        }
        for (surface, value) in [
            ("줄", "\n"),
            ("칸", "\t"),
            ("앞", "\r"),
            ("따옴", "\""),
            ("역빗금", "\\"),
        ] {
            if self.starts_with(surface) {
                self.advance_str(surface);
                return Ok(value.to_string());
            }
        }
        for surface in ["반전끝", "굵게끝", "반전", "굵게", "되돌림"] {
            if self.starts_with(surface) {
                self.advance_str(surface);
                return Ok(format!("\\{}", surface));
            }
        }
        for surface in ["색", "배경"] {
            if self.starts_with(surface) {
                let saved = self.pos;
                self.advance_str(surface);
                if self.peek_char() == Some('{') {
                    let body = self.read_rich_markup_braced_body(start, surface)?;
                    return Ok(format!("\\{}{{{}}}", surface, body));
                }
                self.pos = saved;
            }
        }
        match self.peek_char() {
            Some('n') => {
                self.advance();
                Ok("\n".to_string())
            }
            Some('r') => {
                self.advance();
                Ok("\r".to_string())
            }
            Some('t') => {
                self.advance();
                Ok("\t".to_string())
            }
            Some('"') => {
                self.advance();
                Ok("\"".to_string())
            }
            Some('\\') => {
                self.advance();
                Ok("\\".to_string())
            }
            Some(other) => Err(LexError::new(
                self.pos,
                &format!("알 수 없는 글 이스케이프: \\{}", other),
            )),
            None => Err(LexError::new(self.pos, "문자열 종료")),
        }
    }

    fn read_rich_markup_braced_body(
        &mut self,
        start: usize,
        surface: &str,
    ) -> Result<String, LexError> {
        if self.peek_char() != Some('{') {
            return Err(LexError::new(
                self.pos,
                &format!("콘솔 {} 표지는 '{{...}}'가 필요합니다", surface),
            ));
        }
        self.advance();
        let mut body = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == '}' {
                self.advance();
                return Ok(body);
            }
            if ch == '\n' || ch == '\r' {
                return Err(LexError::new(start, "콘솔 표지 본문이 닫히지 않았습니다"));
            }
            body.push(ch);
            self.advance();
        }
        Err(LexError::new(start, "콘솔 표지 본문이 닫히지 않았습니다"))
    }

    fn read_atom(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        self.advance();
        while let Some(ch) = self.peek_char() {
            if matches!(ch, '가'..='힣' | 'a'..='z' | '0'..='9' | '_') {
                self.advance();
            } else {
                break;
            }
        }
        Ok(Token {
            kind: TokenKind::Atom(self.source[start + 1..self.pos].to_string()),
            span: Span::new(start, self.pos),
            raw: self.source[start..self.pos].to_string(),
        })
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
        let start = self.pos;
        self.advance();
        while let Some(ch) = self.peek_char() {
            if matches!(ch, '가'..='힣' | 'a'..='z' | '0'..='9' | '_') {
                self.advance();
            } else {
                break;
            }
        }
        Ok(Token {
            kind: TokenKind::Variable(self.source[start + 1..self.pos].to_string()),
            span: Span::new(start, self.pos),
            raw: self.source[start..self.pos].to_string(),
        })
    }

    fn read_nuance(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        self.advance();
        let mut saw = false;
        while let Some(ch) = self.peek_char() {
            if matches!(ch, '가'..='힣' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ')
            {
                self.advance();
                saw = true;
            } else {
                break;
            }
        }
        if !saw {
            return Err(LexError::new(
                start,
                "Gate0: '$' 뒤에는 뉘앙스 토큰이 필요합니다",
            ));
        }
        let branch = self.source[start + 1..self.pos].to_string();
        if let Some(name) = branch.strip_suffix("이면").filter(|name| !name.is_empty()) {
            let tail_start = start + 1 + name.len();
            self.pending = Some(Token {
                kind: TokenKind::KwMajeumyeon,
                span: Span::new(tail_start, tail_start + "이면".len()),
                raw: "이면".to_string(),
            });
            return Ok(Token {
                kind: TokenKind::BranchRole(name.to_string()),
                span: Span::new(start, tail_start),
                raw: self.source[start..tail_start].to_string(),
            });
        }
        Ok(Token {
            kind: TokenKind::BranchRole(branch),
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

    fn read_bogae_madang_block(
        &mut self,
        start: usize,
        legacy_alias: bool,
    ) -> Result<Token, LexError> {
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
                        kind: if legacy_alias {
                            TokenKind::BogeaJangmyeonBlock(body)
                        } else {
                            TokenKind::BogeaMadangBlock(body)
                        },
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
        Err(LexError::new(start, "보개마당 블록이 닫히지 않았습니다"))
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

    fn read_assertion_block(&mut self, start: usize) -> Result<Token, LexError> {
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
                        kind: TokenKind::AssertionBlock(body),
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
        Err(LexError::new(start, "세움 블록이 닫히지 않았습니다"))
    }

    fn read_state_machine_block(&mut self, start: usize) -> Result<Token, LexError> {
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
                        kind: TokenKind::StateMachineBlock(body),
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
        Err(LexError::new(start, "상태머신 블록이 닫히지 않았습니다"))
    }

    fn read_jjaim_block(&mut self, start: usize) -> Result<Token, LexError> {
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
                        kind: TokenKind::JjaimBlock(body),
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
        Err(LexError::new(start, "짜임 블록이 닫히지 않았습니다"))
    }

    fn read_ascii(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if is_ident_continue(ch) {
                self.advance();
            } else {
                break;
            }
        }
        let text = &self.source[start..self.pos];
        let mut lexeme = text.to_string();
        if let Some(canon) = self.dialect.canonicalize_keyword(text) {
            lexeme = canon.to_string();
        }
        if lexeme == "짜임" {
            let checkpoint = self.pos;
            self.skip_inline_whitespace();
            if self.peek_char() == Some('{') {
                return self.read_jjaim_block(start);
            }
            self.pos = checkpoint;
        }
        if let Some(kind) = keyword_kind_for(&lexeme) {
            return Ok(Token {
                kind,
                span: Span::new(start, self.pos),
                raw: lexeme,
            });
        }
        self.append_prime_derivative_suffix(&mut lexeme);
        Ok(Token {
            kind: TokenKind::Ident(lexeme.clone()),
            span: Span::new(start, self.pos),
            raw: lexeme,
        })
    }

    fn append_prime_derivative_suffix(&mut self, lexeme: &mut String) {
        let mut count = 0usize;
        while count < 2 && self.peek_char() == Some('\'') {
            lexeme.push('\'');
            self.advance();
            count += 1;
        }
    }

    fn try_read_sym3_token(&mut self) -> Option<Token> {
        let start = self.pos;
        let symbol_tokens: Vec<String> = self.dialect.symbol_tokens().to_vec();
        for token in symbol_tokens.iter() {
            if self.starts_with(token) {
                if let Some(canon) = self.dialect.canonicalize_symbol(token) {
                    if let Some(kind) = keyword_kind_for(canon) {
                        self.advance_str(token);
                        return Some(Token {
                            kind,
                            span: Span::new(start, self.pos),
                            raw: token.to_string(),
                        });
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
            self.pending = Some(Token {
                kind: TokenKind::Josa(tail.clone()),
                span,
                raw: tail,
            });
        }
        Token {
            kind: TokenKind::Tilde,
            span: Span::new(start, start + 1),
            raw: "~".to_string(),
        }
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

    fn skip_inline_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if matches!(ch, ' ' | '\t' | '\r' | '\u{feff}') {
                self.advance();
            } else {
                break;
            }
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

    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }
    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.source[self.pos..].chars().nth(n)
    }
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
    fn next_non_ws_starts_with(&self, s: &str) -> bool {
        let mut idx = self.pos;
        while idx < self.source.len() {
            let Some(ch) = self.source[idx..].chars().next() else {
                return false;
            };
            if !ch.is_whitespace() {
                return self.source[idx..].starts_with(s);
            }
            idx += ch.len_utf8();
        }
        false
    }
    fn advance(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.pos += ch.len_utf8();
        }
    }
    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }
}

fn is_josa_tail_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}

fn is_ident_continue(ch: char) -> bool {
    matches!(
        ch,
        '가'..='힣' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'
    )
}

/// 편집기용 한국어 렉서 키워드의 정본 철자입니다. 별칭은 포함하지 않습니다.
///
/// 말씨 표의 `ko_canon`과 렉서 고유 키워드를 투영합니다. 같은 토큰 종류라도
/// 정본 역할이 다른 철자(예: `맞으면`, `이면`)는 각각 남깁니다.
/// 이 목록은 예약어·표준 함수 전체나 위치별 구문 수용 여부를 뜻하지 않습니다.
/// 소비자는 원문 문맥과 명시적 후보 채택 정책을 별도로 적용해야 합니다.
pub const CANONICAL_LEXER_KEYWORDS: &[&str] = &[
    "이름씨", "움직씨", "셈씨", "이음씨", "흐름씨", "갈래씨", "고름씨", "맞물림씨",
    "샘", "그리고", "또는", "만약", "이라면", "일때", "아니면", "되풀이", "동안",
    "대해", "멈추기", "건너뛰기", "되돌림", "톺아보기", "해보고", "고르기", "맞으면",
    "이면", "바탕으로", "다짐하고", "해서", "늘지켜보고", "보여주기", "덩이", "미루기",
];

fn keyword_kind_for(canon: &str) -> Option<TokenKind> {
    match canon {
        "이름씨" => Some(TokenKind::KwImeumssi),
        "움직씨" => Some(TokenKind::KwUmjikssi),
        "값함수" => None,
        "셈씨" => Some(TokenKind::KwSemssi),
        "이음씨" => Some(TokenKind::KwIeumssi),
        "흐름씨" => Some(TokenKind::KwHeureumssi),
        "갈래씨" | "고름씨" => Some(TokenKind::KwGallaessi),
        "관계씨" | "맞물림씨" => Some(TokenKind::KwRelationssi),
        "샘" => Some(TokenKind::KwSam),
        "그리고" => Some(TokenKind::And),
        "또는" => Some(TokenKind::Or),
        "만약" => Some(TokenKind::KwManyak),
        "이라면" => Some(TokenKind::KwIramyeon),
        "일때" => Some(TokenKind::KwIlttae),
        "아니면" | "아니라면" => Some(TokenKind::KwAniramyeon),
        "되풀이" | "반복" => Some(TokenKind::KwBanbok),
        "동안" => Some(TokenKind::KwDongan),
        "대해" => Some(TokenKind::KwDaehae),
        "멈추기" => Some(TokenKind::KwMeomchugi),
        "건너뛰기" => Some(TokenKind::KwGeonneottwigi),
        "되돌림" | "반환" | "돌려줘" => Some(TokenKind::KwDollyeojwo),
        "톺아보기" | "감사" => Some(TokenKind::KwTolabogi),
        "해보고" => Some(TokenKind::KwHaebogo),
        "고르기" => Some(TokenKind::KwGoreugi),
        "맞으면" | "이면" => Some(TokenKind::KwMajeumyeon),
        "바탕으로" | "전제하에" => Some(TokenKind::KwJeonjehae),
        "다짐하고" | "보장하고" => Some(TokenKind::KwBojanghago),
        "해서" => Some(TokenKind::KwHaeseo),
        "늘지켜보고" => Some(TokenKind::KwNeuljikeobogo),
        "보여주기" => Some(TokenKind::KwBoyeojugi),
        "덩이" => Some(TokenKind::KwBeat),
        "미루기" => Some(TokenKind::KwReserve),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub pos: usize,
    pub message: String,
}
impl LexError {
    fn new(pos: usize, message: &str) -> Self {
        Self {
            pos,
            message: message.to_string(),
        }
    }
}
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "렉서 오류: {}", self.message)
    }
}
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
    fn map_lookup_optional_suffix_is_lexed_as_ident() {
        let mut lexer = Lexer::new("값 <- ((\"a\", 1) 짝맞춤, \"a\") 찾기?.\n");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ident(ref name) if name == "찾기?")));
    }

    #[test]
    fn string_replace_strict_suffix_is_lexed_as_ident() {
        let mut lexer = Lexer::new("값 <- (\"가\", 0, \"나\") 글바꾸기!.\n");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ident(ref name) if name == "글바꾸기!")));
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
    fn function_type_arrow_uses_longest_match() {
        let mut lexer = Lexer::new("가씨 --> 나씨. 식 -> 자리.");
        let tokens = lexer.tokenize().expect("tokenize");
        let type_arrow = tokens
            .iter()
            .find(|token| matches!(token.kind, TokenKind::TypeArrow))
            .expect("type arrow");
        assert_eq!(type_arrow.raw, "-->");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::RightArrow)));
    }

    #[test]
    fn right_assignment_arrow_has_exact_two_byte_span() {
        let mut lexer = Lexer::new("3 -> 값.");
        let tokens = lexer.tokenize().expect("tokenize");
        let arrow = tokens
            .iter()
            .find(|token| matches!(token.kind, TokenKind::RightArrow))
            .expect("right assignment arrow");
        assert_eq!(arrow.raw, "->");
        assert_eq!(arrow.span, Span::new(2, 4));
    }

    #[test]
    fn assignment_arrow_token_family_keeps_longest_match_and_neighbors() {
        let mut lexer = Lexer::new("- > >= -<- +<- ~~> --> ->");
        let tokens = lexer.tokenize().expect("tokenize");
        let kinds: Vec<_> = tokens.iter().map(|token| &token.kind).collect();
        assert!(matches!(kinds[0], TokenKind::Minus));
        assert!(matches!(kinds[1], TokenKind::Gt));
        assert!(matches!(kinds[2], TokenKind::GtEq));
        assert!(matches!(kinds[3], TokenKind::MinusArrow));
        assert!(matches!(kinds[4], TokenKind::PlusArrow));
        assert!(matches!(kinds[5], TokenKind::DoubleArrow));
        assert!(matches!(kinds[6], TokenKind::TypeArrow));
        assert!(matches!(kinds[7], TokenKind::RightArrow));
        assert_eq!(tokens[6].raw, "-->");
        assert_eq!(tokens[7].raw, "->");
    }

    #[test]
    fn persistent_dependency_is_not_a_message_token() {
        let mut lexer = Lexer::new("왼 <<- 오른.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::PersistentDependencyArrow)));
        assert!(!tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::DoubleArrow)));
    }

    #[test]
    fn successor_delivery_arrows_have_distinct_tokens() {
        let mut lexer = Lexer::new("받는이 <~ 값. 값 ~> 받는이. 받는이 <~~ 값.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::AsyncLeftArrow)));
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::AsyncRightArrow)));
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::RendezvousLeftArrow)));
    }

    #[test]
    fn sender_filtered_typed_receive_tokens_preserve_exact_josa_boundary() {
        let tokens = Lexer::new("철수에게서 (받은수:수)를 받으면 { }.")
            .tokenize()
            .expect("tokenize");
        let kinds: Vec<_> = tokens.iter().map(|token| &token.kind).collect();
        assert!(matches!(&kinds[0], TokenKind::Ident(name) if name == "철수"));
        assert!(matches!(&kinds[1], TokenKind::Josa(josa) if josa == "에게서"));
        assert!(matches!(&kinds[2], TokenKind::LParen));
        assert!(matches!(&kinds[3], TokenKind::Ident(name) if name == "받은수"));
        assert!(matches!(&kinds[4], TokenKind::Colon));
        assert!(matches!(&kinds[5], TokenKind::Ident(name) if name == "수"));
        assert!(matches!(&kinds[6], TokenKind::RParen));
        assert!(matches!(&kinds[7], TokenKind::Josa(josa) if josa == "를"));
        assert!(matches!(&kinds[8], TokenKind::Ident(name) if name == "받으면"));
    }

    #[test]
    fn bogae_madang_block_allows_space_before_lbrace() {
        let mut lexer = Lexer::new("보개마당 { #자막(\"테스트\"). }.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::BogeaMadangBlock(_))));
    }

    #[test]
    fn bogae_jangmyeon_alias_allows_space_before_lbrace() {
        let mut lexer = Lexer::new("보개장면 { #자막(\"테스트\"). }.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::BogeaJangmyeonBlock(_))));
    }

    #[test]
    fn jjaim_block_allows_space_before_lbrace() {
        let mut lexer = Lexer::new("짜임 { 상태 { x <- 0. }. }.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::JjaimBlock(_))));
    }

    #[test]
    fn guseong_alias_block_allows_space_before_lbrace() {
        let mut lexer = Lexer::new("구성 { 상태 { x <- 0. }. }.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::JjaimBlock(_))));
    }

    #[test]
    fn relation_eq_is_lexed_as_single_token() {
        let mut lexer = Lexer::new("a <- ((#ascii) 수식{x}) =:= ((#ascii) 수식{1}).");
        let tokens = lexer.tokenize().expect("tokenize");
        let relation = tokens
            .iter()
            .find(|token| matches!(token.kind, TokenKind::RelationEq))
            .expect("relation eq token");
        assert_eq!(relation.raw, "=:=");
    }

    #[test]
    fn ident_ending_with_josa_is_not_split_before_arithmetic_operator() {
        let mut lexer = Lexer::new("vx <- 초기속도 * cos_a.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ident(ref name) if name == "초기속도")));
        assert!(!tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Josa(ref name) if name == "도")));
    }

    #[test]
    fn ident_ending_with_josa_is_not_split_before_closing_paren() {
        let mut lexer = Lexer::new("(초기속도).");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ident(ref name) if name == "초기속도")));
        assert!(!tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Josa(ref name) if name == "도")));
    }

    #[test]
    fn ident_ending_with_josa_is_not_split_before_comparison_operator() {
        for operator in [">", ">=", "<", "<=", "==", "!="] {
            let source = format!("{{ 변길이 {operator} 최대변길이 }}이면 {{}}.");
            let tokens = Lexer::new(&source).tokenize().expect("tokenize");
            assert!(tokens.iter().any(|token|
                matches!(&token.kind, TokenKind::Ident(name) if name == "변길이")),
                "비교 앞 이름의 조사 꼬리를 분리하면 안 됩니다: {operator}");
        }
    }

    #[test]
    fn mixed_ascii_korean_identifier_is_single_ident_token() {
        let mut lexer = Lexer::new("n목록 <- 1.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ident(ref name) if name == "n목록")));
    }

    #[test]
    fn prime_derivative_suffix_is_part_of_identifier() {
        let mut lexer = Lexer::new("위치' <- 위치''.");
        let tokens = lexer.tokenize().expect("tokenize");
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ident(ref name) if name == "위치'")));
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ident(ref name) if name == "위치''")));
    }

    #[test]
    fn third_prime_derivative_marker_is_rejected_by_lexer() {
        let mut lexer = Lexer::new("위치''' <- 1.");
        let err = lexer.tokenize().expect_err("third prime must fail");
        assert_eq!(err.message, "알 수 없는 문자");
    }
}

#[cfg(test)]
mod r15_fullwidth_tests {
    use super::*;

    fn check(source: &str, projected: bool) -> Result<Vec<Token>, LexError> {
        let original = "(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를) 총액합하다{총액합하기}:나눔수:셈씨 := { 총액합하다 <- 금액 + 세액. }.\n";
        let public = crate::frontdoor::parse_module_directive_frontdoor_with_mode(
            original, "세금.ddn", crate::ParseMode::Strict,
            &crate::ModuleDirectiveContext::default().with_public_definitions(["총액합하다"]),
        ).unwrap();
        let binding = crate::ddl::bind_first_callable_ja_declaration(&public, "language-profile:ja", 1).unwrap();
        if projected {
            let profile = crate::ddl::issued_first_callable_profile().unwrap();
            Lexer::with_first_callable_projection(source, &binding, &profile).unwrap().tokenize()
        } else {
            Lexer::with_first_callable_registration(source, &binding).unwrap().tokenize()
        }
    }

    #[test]
    fn registered_syntax_mistakes_report_exact_utf8_position_in_both_paths() {
        for projected in [false, true] {
            for (text, bad) in [("合計する。", '。'), ("有理数／有理数", '／'),
                ("金額、税額", '、'), ("に）", '）'), ("合＝計する", '＝'),
                ("(１００,1)", '１'), ("(100，1)", '，'), ("（100,1)", '（'),
                ("金額：有理数", '：'), ("合計する！", '！')] {
                let source = format!("#!日本語\n{text}");
                let err = check(&source, projected).unwrap_err();
                assert_eq!(err.pos, source.find(bad).unwrap(), "{projected}: {source}");
                assert!(err.message.starts_with("E_MULTILANG_FULLWIDTH_SYNTAX:"), "{err:?}");
                assert!(err.message.contains("ASCII"));
                assert!(err.message.contains(&format!("byte={}..{}", err.pos, err.pos + bad.len_utf8())));
            }
        }
    }

    #[test]
    fn registered_strings_and_ascii_tokens_keep_their_original_bytes() {
        for projected in [false, true] {
            let source = "#!日本語\n合計する (100,1) \"道／金額。＝１００\".";
            let tokens = check(source, projected).unwrap();
            assert!(tokens.iter().any(|t| matches!(&t.kind, TokenKind::StringLit(s) if s == "道／金額。＝１００")));
            assert!(tokens.iter().any(|t| t.raw == "合計する"));
            assert!(tokens.iter().all(|t| source.get(t.span.start..t.span.end) == Some(t.raw.as_str())));
            let unknown = check("#!日本語\n未知の名前", projected).unwrap_err();
            assert!(unknown.message.starts_with("E_MULTILANG_UNREGISTERED_SURFACE:"));
        }
        // The ordinary lexer remains the authority outside registered contexts.
        let plain = Lexer::new("合計する。").tokenize();
        assert!(!plain.err().is_some_and(|e| e.message.starts_with("E_MULTILANG_FULLWIDTH_SYNTAX:")));
    }
}

#[cfg(test)]
mod r31_j1_lexer_tests {
    use super::*;
    #[test] fn r31_j1_lex_is_explicit_and_keeps_literal_bytes_and_spans(){
        let profile=crate::ddl::CheckedJ1Profile::resolve(crate::ddl::J1_SOURCE_PROFILE).unwrap();
        let raw="#!日本語\r\n実行:手続き := { \"表示\" 表示. }.\r\n";
        let tokens=Lexer::with_j1_profile(raw,profile).unwrap().tokenize().unwrap();
        for token in &tokens{if !matches!(token.kind,TokenKind::Eof){assert_eq!(&raw[token.span.start..token.span.end],token.raw);}}
        assert!(matches!(tokens[0].kind,TokenKind::Ident(ref name) if name=="実行"));
        assert!(Lexer::with_j1_profile("実行:手続き := {}.",profile).is_err());
        assert!(crate::ddl::CheckedJ1Profile::resolve("v1-core-v25").is_err());
        let bad="#!日本語\n実行:手続き := { (金額） 表示. }.";
        let err=Lexer::with_j1_profile(bad,profile).unwrap().tokenize().unwrap_err();assert_eq!(err.pos,bad.find('）').unwrap());
        assert!(err.message.contains("FULLWIDTH"));
    }
}
