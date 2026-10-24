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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDirective {
    pub kind: ModuleDirectiveKind,
    pub entries: Vec<ModuleDirectiveEntry>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDirectiveDocument {
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
        imports,
        exports,
        declaration_source,
    })
}

pub fn canonicalize_module_directive_document(document: &ModuleDirectiveDocument) -> String {
    let mut lines = Vec::new();
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
            span: Span::new(start, end),
        })
    }
}
