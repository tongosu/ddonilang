use std::collections::BTreeSet;

use crate::{Lexer, ParseError, ParseMode, Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleDirectiveKind {
    Import,
    Export,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDirectiveEntry {
    /// Existing local binding spelling; this is not a newly-created identity.
    pub local_name: String,
    /// Existing module or DefinitionIdentity spelling supplied by the product context.
    pub target: String,
}

/// Parsed request only. A header or this tuple never grants DDL activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleTranslationRequest {
    pub target_language: String,
    pub target_version: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDirective {
    pub kind: ModuleDirectiveKind,
    pub entries: Vec<ModuleDirectiveEntry>,
    pub translation: Option<ModuleTranslationRequest>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDirectiveDocument {
    pub source_header: Option<String>,
    pub imports: Option<ModuleDirective>,
    pub exports: Option<ModuleDirective>,
    /// A validated ordinary V25 declaration tail. Directives never enter it.
    pub declaration_source: String,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleDirectiveContext {
    pub known_import_targets: BTreeSet<String>,
    pub public_definition_targets: BTreeSet<String>,
}

impl ModuleDirectiveContext {
    pub fn with_known_imports<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.known_import_targets = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_public_definitions<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.public_definition_targets = values.into_iter().map(Into::into).collect();
        self
    }
}

pub fn parse_module_directive_document(
    source: &str,
    file_path: &str,
    context: &ModuleDirectiveContext,
) -> Result<ModuleDirectiveDocument, ParseError> {
    let document = parse_module_directive_prefix(source, file_path, context)?;
    if !document.declaration_source.is_empty() {
        crate::parse_v25_with_mode(&document.declaration_source, file_path, ParseMode::Strict)
            .map_err(|mut error| {
                let directive_bytes = source.len() - document.declaration_source.len();
                error.span = Span::new(
                    directive_bytes + error.span.start,
                    directive_bytes + error.span.end,
                );
                error
            })?;
    }
    Ok(document)
}

/// Parse only the typed module-directive prefix.  Consumers that own a later
/// link phase may validate the declaration tail after imported aliases have
/// been resolved; this function never lowers a directive into an ordinary
/// call or discards its typed metadata.
pub fn parse_module_directive_prefix(
    source: &str,
    _file_path: &str,
    context: &ModuleDirectiveContext,
) -> Result<ModuleDirectiveDocument, ParseError> {
    let tokens = Lexer::new(source).tokenize().map_err(|error| ParseError {
        span: Span::new(error.pos, error.pos + 1),
        message: error.message,
    })?;
    parse_directive_tokens(source,&tokens,context,None)
}

/// Lexical context is explicit and remains compilation-local. A successful
/// prefix parse is not registry, public-DDL or execution admission.
pub fn parse_module_directive_prefix_with_first_callable_context(
    source:&str,file_path:&str,context:&ModuleDirectiveContext,
    first:&crate::parser::RegisteredFirstCallableContext,
)->Result<ModuleDirectiveDocument,ParseError>{
    first_callable_module_tokens(source,file_path,context,first).map(|(document,_)|document)
}
pub(crate) fn first_callable_module_tokens(
    source:&str,_file_path:&str,context:&ModuleDirectiveContext,
    first:&crate::parser::RegisteredFirstCallableContext,
)->Result<(ModuleDirectiveDocument,Vec<Token>),ParseError>{
    let registry=crate::ddl::FIRST_CALLABLE_JA_REGISTRATION;
    let binding=crate::ddl::bind_first_callable_ja_declaration(first.original_public(),registry.language_profile_identity,registry.registration_revision)
        .map_err(|e|ParseError{span:e.span,message:e.to_string()})?;
    let japanese_header=source.lines().next()==Some("#!日本語");
    let source_header=japanese_header.then(||"#!日本語".to_string());
    let mut lexer=if japanese_header {
        Lexer::with_first_callable_registration(source,&binding)
    }else if let Some(profile)=first.activation.as_ref().map(|a|&a.profile).or(first.prefix_profile.as_ref()) {
        Lexer::with_first_callable_projection(source,&binding,profile)
    }else{return Err(ParseError{span:Span::new(0,0),message:"E_MULTILANG_HEADER_REQUIRED".into()});}
        .map_err(|e|prefix_lex_error(source,e))?;
    let mut tokens=Vec::new();
    // The existing common lexer supplies only complete module records. Body
    // lookahead uses a checkpoint; it never consumes/reclassifies body bytes.
    while lexer.prefix_token_start().1==Some('('){
        let mut probe=lexer.prefix_checkpoint();
        let mut record=Vec::new();let mut depth=0usize;
        loop {
            let token=match probe.next_common_prefix_token(){Ok(token)=>token,Err(_)=>break};
            match token.kind {TokenKind::LParen=>depth+=1,TokenKind::RParen=>{depth=match depth.checked_sub(1){Some(d)=>d,None=>break};},TokenKind::Eof=>break,_=>{}}
            record.push(token);
            if depth==0{break;}
        }
        // A parameter list/call belongs to the registered body, not to a
        // common-prefix vocabulary expansion. The existing parser decides it.
        if depth!=0 || !matches!(record.last().map(|t|&t.kind),Some(TokenKind::RParen)){break;}
        let keyword=match probe.next_common_prefix_token(){Ok(token)=>token,Err(_)=>break};
        if !matches!(&keyword.kind,TokenKind::Ident(name) if matches!(name.as_str(),"쓰임"|"드러냄"|"공개")){break;}
        record.push(keyword);
        record.push(probe.next_common_prefix_token().map_err(|e|prefix_lex_error(source,e))?);
        let mut checked=tokens.clone();checked.extend(record.iter().cloned());
        let end=record.last().unwrap().span.end;
        checked.push(Token{kind:TokenKind::Eof,span:crate::lexer::Span::new(end,end),raw:String::new()});
        // Exact grammar, target lookup, duplicate and retired-surface errors
        // remain owned by the existing module-directive parser.
        parse_directive_tokens(source,&checked,context,source_header.clone())?;
        tokens.extend(record);lexer=probe;
    }
    tokens.extend(lexer.tokenize().map_err(|e|prefix_lex_error(source,e))?);
    let document=parse_directive_tokens(source,&tokens,context,source_header.clone())?;
    Ok((document,tokens))
}
fn prefix_lex_error(source:&str,e:crate::lexer::LexError)->ParseError{
    ParseError{span:Span::new(e.pos,e.pos+source.get(e.pos..).and_then(|tail|tail.chars().next()).map_or(0,char::len_utf8)),message:e.message}

}
fn parse_directive_tokens(source:&str,tokens:&[Token],context:&ModuleDirectiveContext,source_header:Option<String>)->Result<ModuleDirectiveDocument,ParseError>{
    let mut cursor = DirectiveCursor::new(&tokens);
    let mut imports = None;
    let mut exports = None;

    loop {
        if cursor.is_eof() {
            break;
        }
        if !matches!(cursor.current().kind, TokenKind::LParen) {
            break;
        }
        // Parenthesized callable parameters belong to the declaration tail,
        // not to the preceding module directives.
        if cursor.starts_callable_declaration() {
            break;
        }
        let directive = cursor.parse_directive(context)?;
        match directive.kind {
            ModuleDirectiveKind::Import => {
                if imports.replace(directive).is_some() {
                    return Err(cursor.error(
                        "E_RD457_IMPORT_DUPLICATE",
                        "쓰임 directive는 module당 하나만 허용됩니다",
                    ));
                }
            }
            ModuleDirectiveKind::Export => {
                if exports.replace(directive).is_some() {
                    return Err(cursor.error(
                        "E_RD457_EXPORT_DUPLICATE",
                        "드러냄 directive는 module당 하나만 허용됩니다",
                    ));
                }
            }
        }
    }

    if imports.is_none() && exports.is_none() && !cursor.starts_callable_declaration() {
        if matches!(cursor.current().kind, TokenKind::Ident(ref name) if matches!(name.as_str(), "쓰임" | "드러냄" | "공개"))
            && matches!(cursor.peek().kind, TokenKind::LBrace)
        {
            return Err(cursor.error(
                "E_RD457_LEGACY_BRACE_DIRECTIVE",
                "brace형 module directive는 current V25 입력이 아닙니다",
            ));
        }
        return Err(cursor.error(
            "E_RD457_DIRECTIVE_EXPECTED",
            "parenthesized module directive가 필요합니다",
        ));
    }

    let declaration_start = cursor.current().span.start;
    let declaration_source = if cursor.is_eof() {
        String::new()
    } else {
        source[declaration_start..].to_string()
    };
    Ok(ModuleDirectiveDocument {
        source_header,
        imports,
        exports,
        declaration_source,
    })
}

pub fn canonicalize_module_directive_document(document: &ModuleDirectiveDocument) -> String {
    let mut lines = Vec::new();
    if let Some(header)=&document.source_header {lines.push(header.clone());}
    if let Some(imports) = &document.imports {
        lines.push(canonicalize_directive(imports));
    }
    if let Some(exports) = &document.exports {
        lines.push(canonicalize_directive(exports));
    }
    if !document.declaration_source.is_empty() {
        let declaration_source = document
            .declaration_source
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        lines.push(declaration_source.trim_end_matches('\n').to_string());
    }
    format!("{}\n", lines.join("\n"))
}

fn canonicalize_directive(directive: &ModuleDirective) -> String {
    let entries = directive
        .entries
        .iter()
        .map(|entry| format!("{}={}", entry.local_name, quote_if_needed(&entry.target)))
        .collect::<Vec<_>>()
        .join(", ");
    let entries=if let Some(request)=&directive.translation {
        format!("{entries}, 옮김=(대상말씨={}, 대상판본={})",serde_json::to_string(&request.target_language).unwrap(),serde_json::to_string(&request.target_version).unwrap())
    } else {entries};
    let keyword = match directive.kind {
        ModuleDirectiveKind::Import => "쓰임",
        ModuleDirectiveKind::Export => "드러냄",
    };
    format!("({entries}) {keyword}.")
}

fn quote_if_needed(value: &str) -> String {
    if value.starts_with("./") || value.contains('/') {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_directive_prefix_preserves_parameterized_callable() {
        let tail = "(값:수) 누적:움직씨 := { 합 <- 합 + 값. }.";
        let source = format!("(누적=누적) 드러냄.\n{tail}");
        let context = ModuleDirectiveContext::default().with_public_definitions(["누적"]);
        let parsed = parse_module_directive_prefix(&source, "lib.ddn", &context).unwrap();
        assert_eq!(parsed.declaration_source, tail);
        assert!(parsed.exports.is_some());
    }

    #[test]
    fn module_directive_prefix_preserves_callable_lexical_registration() {
        // RD-326 lexical forms belong to the definition, not an import record.
        let tail = "(값:수) 돕다{도우~도와}:수:셈씨 := { 돕다 <- 값 + 1. }.";
        for source in [tail.to_owned(), format!("(도움=돕다) 드러냄.\n{tail}")] {
            let context=ModuleDirectiveContext::default().with_public_definitions(["돕다"]);
            let prefix=parse_module_directive_prefix(&source,"lib.ddn",&context).unwrap();
            assert_eq!(prefix.declaration_source,tail);
            let program=crate::parse_v25(&prefix.declaration_source,"lib.ddn").unwrap();
            let crate::TopLevelItem::SeedDef(seed)=&program.items[0];
            assert_eq!(seed.alternate_forms,["도우","도와"]);
        }
    }

    #[test]
    fn module_directive_prefix_still_rejects_malformed_following_directive() {
        let context = ModuleDirectiveContext::default().with_public_definitions(["누적"]);
        let error = parse_module_directive_prefix(
            "(누적=누적) 드러냄. (앱:앱) 쓰임.", "lib.ddn", &context,
        ).unwrap_err();
        assert!(error.message.contains("E_RD457_ENTRY_EQUALS"));
    }
}

struct DirectiveCursor<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl<'a> DirectiveCursor<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, index: 0 }
    }

    fn current(&self) -> &'a Token {
        &self.tokens[self.index]
    }

    fn peek(&self) -> &'a Token {
        self.tokens
            .get(self.index + 1)
            .unwrap_or_else(|| self.current())
    }

    fn is_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    fn starts_callable_declaration(&self) -> bool {
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate().skip(self.index) {
            match &token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth = match depth.checked_sub(1) {
                        Some(value) => value,
                        None => return false,
                    };
                    if depth == 0 {
                        // Static type/proposition binders precede the callable
                        // parameters. The declaration parser owns validation;
                        // they are not another module-directive record.
                        if self.tokens.get(index + 1).is_some_and(|token|token.raw=="두고") {
                            return true;
                        }
                        if !matches!(self.tokens.get(index + 1).map(|t| &t.kind), Some(TokenKind::Ident(_))) {
                            return false;
                        }
                        let mut after_name = index + 2;
                        // RD-326: a named callable can register lexical forms
                        // before its colon. This is still the declaration tail,
                        // never an RD-457 record of module imports/exports.
                        // The ordinary parser owns validation of the forms.
                        if matches!(self.tokens.get(after_name).map(|t| &t.kind), Some(TokenKind::LBrace)) {
                            after_name += 1;
                            while let Some(token) = self.tokens.get(after_name) {
                                match token.kind {
                                    TokenKind::RBrace => { after_name += 1; break; }
                                    TokenKind::Eof | TokenKind::LBrace => return false,
                                    _ => after_name += 1,
                                }
                            }
                        }
                        return matches!(self.tokens.get(after_name).map(|t| &t.kind), Some(TokenKind::Colon));
                    }
                }
                TokenKind::Eof => return false,
                _ => {}
            }
        }
        false
    }

    fn advance(&mut self) {
        if !self.is_eof() {
            self.index += 1;
        }
    }

    fn error(&self, code: &str, detail: &str) -> ParseError {
        ParseError {
            span: Span::new(self.current().span.start, self.current().span.end),
            message: format!("{code}: {detail}"),
        }
    }

    fn expect(
        &mut self,
        expected: fn(&TokenKind) -> bool,
        code: &str,
        detail: &str,
    ) -> Result<Token, ParseError> {
        if !expected(&self.current().kind) {
            return Err(self.error(code, detail));
        }
        let token = self.current().clone();
        self.advance();
        Ok(token)
    }

    fn parse_translation_request(&mut self)->Result<ModuleTranslationRequest,ParseError>{
        let start=self.current().span.start;self.advance();
        let mut language=None;let mut version=None;
        loop {
            let key=self.expect(|k|matches!(k,TokenKind::Ident(_)),"E_DDL_ACTIVATION_FIELD","대상말씨·대상판본 필드가 필요합니다")?;
            let TokenKind::Ident(key)=key.kind else{unreachable!()};
            self.expect(|k|matches!(k,TokenKind::Equals),"E_DDL_ACTIVATION_EQUALS","필드에는 '='가 필요합니다")?;
            let value=self.expect(|k|matches!(k,TokenKind::StringLit(_)),"E_DDL_ACTIVATION_LITERAL","말씨와 정확 판본은 글 리터럴이어야 합니다")?;
            let TokenKind::StringLit(value)=value.kind else{unreachable!()};
            if value.is_empty(){return Err(self.error("E_DDL_ACTIVATION_EMPTY","빈 말씨·판본은 허용하지 않습니다"));}
            let slot=match key.as_str(){"대상말씨"=>&mut language,"대상판본"=>&mut version,_=>return Err(self.error("E_DDL_ACTIVATION_FIELD","알 수 없는 옮김 필드입니다"))};
            if slot.replace(value).is_some(){return Err(self.error("E_DDL_ACTIVATION_DUPLICATE","옮김 필드가 중복됩니다"));}
            if !matches!(self.current().kind,TokenKind::Comma){break;}self.advance();
        }
        let end=self.expect(|k|matches!(k,TokenKind::RParen),"E_DDL_ACTIVATION_SHAPE","옮김 자료는 ')'로 닫혀야 합니다")?.span.end;
        Ok(ModuleTranslationRequest{target_language:language.ok_or_else(||self.error("E_DDL_ACTIVATION_FIELD","대상말씨가 없습니다"))?,
            target_version:version.ok_or_else(||self.error("E_DDL_ACTIVATION_FIELD","대상판본이 없습니다"))?,span:Span::new(start,end)})
    }

    fn parse_directive(
        &mut self,
        context: &ModuleDirectiveContext,
    ) -> Result<ModuleDirective, ParseError> {
        let start = self
            .expect(
                |kind| matches!(kind, TokenKind::LParen),
                "E_RD457_DIRECTIVE_SHAPE",
                "directive는 '('로 시작해야 합니다",
            )?
            .span
            .start;
        let mut entries = Vec::new();
        let mut translation = None;
        loop {
            let local = self.expect(
                |kind| matches!(kind, TokenKind::Ident(_)),
                "E_RD457_ENTRY_NAME",
                "항목 이름이 필요합니다",
            )?;
            let local_name = match local.kind {
                TokenKind::Ident(name) => name,
                _ => unreachable!(),
            };
            self.expect(
                |kind| matches!(kind, TokenKind::Equals),
                "E_RD457_ENTRY_EQUALS",
                "항목에는 '='가 필요합니다",
            )?;
            if local_name == "옮김" && matches!(self.current().kind,TokenKind::LParen) {
                if translation.is_some(){return Err(self.error("E_DDL_ACTIVATION_DUPLICATE","옮김은 하나만 허용됩니다"));}
                translation=Some(self.parse_translation_request()?);
            } else {
            let target_token = self.expect(
                |kind| matches!(kind, TokenKind::Ident(_) | TokenKind::StringLit(_)),
                "E_RD457_ENTRY_TARGET",
                "항목 target은 existing identifier 또는 source reference여야 합니다",
            )?;
            let target = match target_token.kind {
                TokenKind::Ident(value) | TokenKind::StringLit(value) => value,
                _ => unreachable!(),
            };
            entries.push(ModuleDirectiveEntry { local_name, target });
            }
            if matches!(self.current().kind, TokenKind::Comma) {
                self.advance();
                continue;
            }
            break;
        }
        self.expect(
            |kind| matches!(kind, TokenKind::RParen),
            "E_RD457_DIRECTIVE_SHAPE",
            "directive는 ')'로 닫혀야 합니다",
        )?;
        let keyword = self.expect(
            |kind| matches!(kind, TokenKind::Ident(_)),
            "E_RD457_DIRECTIVE_KEYWORD",
            "쓰임 또는 드러냄 directive가 필요합니다",
        )?;
        let kind = match keyword.kind {
            TokenKind::Ident(name) if name == "쓰임" => ModuleDirectiveKind::Import,
            TokenKind::Ident(name) if name == "드러냄" => ModuleDirectiveKind::Export,
            TokenKind::Ident(name) if name == "공개" => {
                return Err(self.error(
                    "E_RD457_LEGACY_PUBLIC_DIRECTIVE",
                    "공개 directive는 current V25 입력이 아닙니다",
                ))
            }
            _ => {
                return Err(self.error(
                    "E_RD457_DIRECTIVE_KEYWORD",
                    "쓰임 또는 드러냄 directive가 필요합니다",
                ))
            }
        };
        let end = self
            .expect(
                |token| matches!(token, TokenKind::Dot),
                "E_RD457_DIRECTIVE_TERMINATOR",
                "directive에는 '.' terminator가 필요합니다",
            )?
            .span
            .end;

        if translation.is_some() && (kind != ModuleDirectiveKind::Import || entries.len()!=1){
            return Err(ParseError{span:Span::new(start,end),message:"E_DDL_ACTIVATION_BINDING_REQUIRED: 한정 옮김은 하나의 쓰임 모듈 결속에만 적용합니다".into()});
        }
        let registry = match kind {
            ModuleDirectiveKind::Import => &context.known_import_targets,
            ModuleDirectiveKind::Export => &context.public_definition_targets,
        };
        for entry in &entries {
            if !registry.contains(&entry.target) {
                let code = match kind {
                    ModuleDirectiveKind::Import => "E_RD457_IMPORT_TARGET_UNKNOWN",
                    ModuleDirectiveKind::Export => "E_RD457_EXPORT_TARGET_PRIVATE_OR_UNKNOWN",
                };
                return Err(ParseError {
                    span: Span::new(start, end),
                    message: format!("{code}: existing product registry에 없는 target입니다"),
                });
            }
        }
        Ok(ModuleDirective {
            kind,
            entries,
            translation,
            span: Span::new(start, end),
        })
    }
}

#[cfg(test)]
mod g2_translation_request_tests {
 use super::*;
 const RAW:&str="(세금=\"./세금\", 옮김=(대상말씨=\"日本語\", 대상판본=\"first-callable-ja-r1\")) 쓰임.\n";
 fn ctx()->ModuleDirectiveContext{ModuleDirectiveContext::default().with_known_imports(["./세금","다른","옮김"]).with_public_definitions(["./세금"])}
 #[test] fn g2_literals_utf8_and_roundtrip(){
  let doc=parse_module_directive_prefix(RAW,"利用.ddn",&ctx()).unwrap();let request=doc.imports.as_ref().unwrap().translation.as_ref().unwrap();
  assert_eq!(request.target_language,"日本語");assert_eq!(request.target_version,"first-callable-ja-r1");assert_eq!(&RAW[request.span.start..request.span.end],"(대상말씨=\"日本語\", 대상판본=\"first-callable-ja-r1\")");
  assert_eq!(canonicalize_module_directive_document(&doc),RAW);
 }
 #[test] fn g2_duplicate_nonliteral_and_ambiguous_owner_refuse(){
  for raw in [RAW.replace("대상판본=\"first-callable-ja-r1\"","대상말씨=\"日本語\""),RAW.replace("대상말씨=\"日本語\"","대상말씨=日本語"),RAW.replace(" 쓰임."," 드러냄."),RAW.replace("세금=\"./세금\",","세금=\"./세금\", 다른=다른,"),RAW.replace("대상판본=\"first-callable-ja-r1\"","다른=\"x\""),RAW.replace("대상판본=\"first-callable-ja-r1\"","대상판본=\"\""),RAW.replace(") 쓰임.",", 옮김=(대상말씨=\"日本語\", 대상판본=\"x\")) 쓰임.")]{assert!(parse_module_directive_prefix(&raw,"利用.ddn",&ctx()).is_err(),"{raw}");}
  assert!(parse_module_directive_prefix("(옮김=옮김) 쓰임.","old.ddn",&ctx()).unwrap().imports.unwrap().translation.is_none());
 }
}

#[cfg(test)]
mod g2_prefix_boundary_tests {
use super::*;
const ORIGINAL:&str="(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를)\n총액합하다{총액합하기}:나눔수:셈씨 := {\n    총액합하다 <- 금액 + 세액.\n}.\n";
const DDL:&str="#!日本語\n(金額:有理数~に, 税額:有理数~を)\n合計する:有理数:関数 :={...}.\n";

const PREFIX:&str="(세금=\"./세금\", 옮김=(대상말씨=\"日本語\", 대상판본=\"first-callable-ja-r1\")) 쓰임.\n";
const BODY:&str="金額 <- (金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する.\n";
fn first()->crate::parser::RegisteredFirstCallableContext{
 let original=crate::parse_module_directive_frontdoor_with_mode(ORIGINAL,"세금.ddn",ParseMode::Strict,&ModuleDirectiveContext::default().with_public_definitions(["총액합하다"])).unwrap();
 crate::parser::RegisteredFirstCallableContext::from_public_and_ddl(original,DDL,"세금.ddl","language-profile:ja",1).unwrap()
}
fn ctx()->ModuleDirectiveContext{ModuleDirectiveContext::default().with_known_imports(["./세금"])}
#[test] fn r13_same_japanese_prefix_reaches_exact_registered_body(){
 let first=first();let raw=format!("#!日本語\n{PREFIX}{BODY}");let doc=parse_module_directive_prefix_with_first_callable_context(&raw,"利用.ddn",&ctx(),&first).unwrap();
 assert_eq!(doc.declaration_source,BODY);assert_eq!(doc.source_header.as_deref(),Some("#!日本語"));
 let request=doc.imports.as_ref().unwrap().translation.as_ref().unwrap();assert_eq!(&raw[request.span.start..request.span.end],"(대상말씨=\"日本語\", 대상판본=\"first-callable-ja-r1\")");
 let canon=canonicalize_module_directive_document(&doc);assert_eq!(canon,raw);
 assert_eq!(parse_module_directive_prefix_with_first_callable_context(&canon,"正本.ddn",&ctx(),&first).unwrap(),doc);
 assert!(Lexer::new(&raw).tokenize().is_err());let reg=crate::ddl::FIRST_CALLABLE_JA_REGISTRATION;let binding=crate::ddl::bind_first_callable_ja_declaration(first.original_public(),reg.language_profile_identity,reg.registration_revision).unwrap();
 assert!(Lexer::with_first_callable_registration(&raw,&binding).unwrap().tokenize().is_err());
}
#[test] fn r13_comments_crlf_and_utf8_positions_stay_in_original_source(){
 let first=first();let body=BODY.replace('\n',"\r\n");let raw=format!("#!日本語\r\n// 한글・日本語 prefix\r\n{}// 本文 앞 주석\r\n{body}",PREFIX.replace('\n',"\r\n"));
 let doc=parse_module_directive_prefix_with_first_callable_context(&raw,"利用.ddn",&ctx(),&first).unwrap();assert_eq!(doc.declaration_source,body);
 let d=doc.imports.unwrap();assert_eq!(&raw[d.span.start..d.span.end],PREFIX.trim());assert!(raw.is_char_boundary(d.span.start)&&raw.is_char_boundary(d.span.end));
 let bad=raw.replace("金額 <-","未知 <-");let e=parse_module_directive_prefix_with_first_callable_context(&bad,"利用.ddn",&ctx(),&first).unwrap_err();assert!(e.message.contains("E_MULTILANG_UNREGISTERED_SURFACE"));assert_eq!(e.span.start,bad.find("未知").unwrap());assert_eq!(e.span.end-e.span.start,'未'.len_utf8());
}
#[test] fn r13_duplicate_malformed_and_repeated_header_still_refuse(){
 let first=first();for raw in [format!("#!日本語\n{PREFIX}{PREFIX}{BODY}"),format!("#!日本語\n{}\n{BODY}",PREFIX.trim_end().trim_end_matches('.')),format!("#!日本語\n{PREFIX}#!日本語\n{BODY}"),format!("#!日本語\n{}{BODY}",PREFIX.replace("./세금","./다른모듈"))]{assert!(parse_module_directive_prefix_with_first_callable_context(&raw,"利用.ddn",&ctx(),&first).is_err(),"{raw}");}
 let raw=format!("#!日本語\n{PREFIX}{PREFIX}{BODY}");let e=parse_module_directive_prefix_with_first_callable_context(&raw,"利用.ddn",&ctx(),&first).unwrap_err();assert!(e.message.contains("E_RD457_IMPORT_DUPLICATE"),"{}",e.message);
}
#[test] fn r13_callable_parentheses_are_a_body_boundary(){
 let first=first();let body="(金額:有理数~に, 税額:有理数~を) 合計する:有理数:関数 := { 合計する <- 金額 + 税額. }.\n";let raw=format!("#!日本語\n{PREFIX}{body}");
 let doc=parse_module_directive_prefix_with_first_callable_context(&raw,"利用.ddn",&ctx(),&first).unwrap();assert_eq!(doc.declaration_source,body);assert_eq!(doc.imports.unwrap().entries.len(),1);
}
}
