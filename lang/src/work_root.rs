//! RD-457/523/524 static work-root bootstrap. No I/O or runtime parsing.
use crate::{Lexer, ParseError, Span, Token, TokenKind};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RootField {
    Gaji,
    Version,
    SourceEpoch,
    Modules,
    Exports,
    Start,
    ExecutionContract,
    View,
}

impl RootField {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "가지" => Self::Gaji,
            "판본" => Self::Version,
            "글판" => Self::SourceEpoch,
            "모듬" => Self::Modules,
            "내놓음" => Self::Exports,
            "시작" => Self::Start,
            "실행계약" => Self::ExecutionContract,
            "보개" => Self::View,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValueKind {
    Text(String),
    Name(String),
    TokenTree,
}

#[derive(Debug, Clone)]
pub struct FieldValue {
    pub kind: FieldValueKind,
    pub span: Span,
    /// Exact shared-lexer tokens, including balanced nested value delimiters.
    /// Interpreting module/schema contents belongs to the subsequent typed phase.
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct RootPin {
    pub field: RootField,
    pub span: Span,
    pub value: FieldValue,
}

#[derive(Debug, Clone)]
pub struct RootDirective {
    pub span: Span,
    pub pins: Vec<RootPin>,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDeclaration {
    pub name: String,
    pub head: String,
    pub sources: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartReference {
    pub module: String,
    pub definition: String,
    pub span: Span,
}

impl RootDirective {
    pub fn field(&self, field: RootField) -> Option<&FieldValue> {
        self.pins
            .iter()
            .find(|pin| pin.field == field)
            .map(|pin| &pin.value)
    }

    pub fn typed_modules(&self) -> Result<Vec<ModuleDeclaration>, ParseError> {
        let value = self.field(RootField::Modules).ok_or_else(|| ParseError {
            span: self.span,
            message: "E_WORK_ROOT_MODULES_REQUIRED: 모듬이 필요합니다".into(),
        })?;
        let entries = list_entries(&value.tokens, &self.source)?;
        let mut modules = Vec::new();
        let mut names = BTreeSet::new();
        for entry in entries {
            let fields = record_fields(entry, &self.source, &["이름", "머리", "원문"])?;
            let module_name = bare_name(fields[0], &self.source)?;
            if !names.insert(module_name.clone()) {
                return Err(error(
                    &entry[0],
                    "E_WORK_ROOT_DUPLICATE_MODULE: 모듈 이름이 중복됩니다",
                ));
            }
            let head = path_text(fields[1])?;
            let sources = if fields[2].len() == 1 {
                vec![path_text(fields[2])?]
            } else {
                list_entries(fields[2], &self.source)?
                    .into_iter()
                    .map(path_text)
                    .collect::<Result<Vec<_>, _>>()?
            };
            modules.push(ModuleDeclaration {
                name: module_name,
                head,
                sources,
                span: Span::new(entry[0].span.start, entry.last().unwrap().span.end),
            });
        }
        Ok(modules)
    }

    pub fn typed_start(&self) -> Result<Option<StartReference>, ParseError> {
        let Some(value) = self.field(RootField::Start) else {
            return Ok(None);
        };
        let fields = record_fields(&value.tokens, &self.source, &["모듈", "정의"])?;
        Ok(Some(StartReference {
            module: bare_name(fields[0], &self.source)?,
            definition: bare_name(fields[1], &self.source)?,
            span: value.span,
        }))
    }
}

fn authored<'a>(token: &Token, source: &'a str) -> &'a str {
    &source[token.span.start..token.span.end]
}

fn bare_name(tokens: &[Token], source: &str) -> Result<String, ParseError> {
    if tokens.len() == 1 && name(&tokens[0]).is_some() {
        Ok(authored(&tokens[0], source).to_string())
    } else {
        Err(error(
            &tokens[0],
            "E_WORK_ROOT_BARE_NAME: 이름은 따옴표나 점 경로 없이 하나의 식별자로 작성하세요",
        ))
    }
}

fn path_text(tokens: &[Token]) -> Result<String, ParseError> {
    if tokens.len() == 1 {
        if let TokenKind::StringLit(value) = &tokens[0].kind {
            if !value.is_empty() {
                return Ok(value.clone());
            }
        }
    }
    Err(error(
        &tokens[0],
        "E_WORK_ROOT_PATH: 경로는 비어 있지 않은 문자열이어야 합니다",
    ))
}

fn comma_parts(tokens: &[Token]) -> Result<Vec<&[Token]>, ParseError> {
    if tokens.is_empty() {
        return Ok(Vec::new());
    }
    let mut parts = Vec::new();
    let mut begin = 0;
    let mut depth = 0usize;
    for (pos, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| error(token, "E_WORK_ROOT_DELIMITER: 잘못된 괄호"))?;
            }
            TokenKind::Comma if depth == 0 => {
                if begin == pos {
                    return Err(error(
                        token,
                        "E_WORK_ROOT_VALUE: 빈 목록 항목은 허용하지 않습니다",
                    ));
                }
                parts.push(&tokens[begin..pos]);
                begin = pos + 1;
            }
            _ => {}
        }
    }
    if begin == tokens.len() {
        return Err(error(
            tokens.last().unwrap(),
            "E_WORK_ROOT_VALUE: 마지막 쉼표 뒤의 항목이 없습니다",
        ));
    }
    parts.push(&tokens[begin..]);
    Ok(parts)
}

fn list_entries<'a>(tokens: &'a [Token], source: &str) -> Result<Vec<&'a [Token]>, ParseError> {
    if tokens.len() < 3
        || authored(&tokens[0], source) != "차림"
        || !matches!(tokens[1].kind, TokenKind::LBracket)
        || !matches!(tokens.last().unwrap().kind, TokenKind::RBracket)
    {
        return Err(error(
            &tokens[0],
            "E_WORK_ROOT_LIST: 목록은 차림[...]으로 작성하세요",
        ));
    }
    let entries = comma_parts(&tokens[2..tokens.len() - 1])?;
    if entries.is_empty() {
        return Err(error(
            &tokens[0],
            "E_WORK_ROOT_LIST_EMPTY: 목록은 비어 있을 수 없습니다",
        ));
    }
    Ok(entries)
}

fn record_fields<'a>(
    tokens: &'a [Token],
    source: &str,
    required: &[&str],
) -> Result<Vec<&'a [Token]>, ParseError> {
    if tokens.len() < 2
        || !matches!(tokens[0].kind, TokenKind::LParen)
        || !matches!(tokens.last().unwrap().kind, TokenKind::RParen)
    {
        return Err(error(
            &tokens[0],
            "E_WORK_ROOT_RECORD: 항목 묶음은 (항목=값, ...)으로 작성하세요",
        ));
    }
    let mut fields = vec![None; required.len()];
    for part in comma_parts(&tokens[1..tokens.len() - 1])? {
        let Some(index) = required
            .iter()
            .position(|key| *key == authored(&part[0], source))
        else {
            return Err(error(
                &part[0],
                "E_WORK_ROOT_RECORD_FIELD: 허용되지 않은 항목입니다",
            ));
        };
        if fields[index].is_some() {
            return Err(error(
                &part[0],
                "E_WORK_ROOT_RECORD_DUPLICATE: 항목이 중복됩니다",
            ));
        }
        if part.len() < 3 || !matches!(part[1].kind, TokenKind::Equals) {
            return Err(error(
                &part[0],
                "E_WORK_ROOT_RECORD_VALUE: 항목=값이 필요합니다",
            ));
        }
        fields[index] = Some(&part[2..]);
    }
    fields
        .into_iter()
        .map(|field| {
            field.ok_or_else(|| {
                error(
                    &tokens[0],
                    "E_WORK_ROOT_RECORD_MISSING: 필수 항목이 없습니다",
                )
            })
        })
        .collect()
}

fn error(token: &Token, message: &str) -> ParseError {
    ParseError {
        span: Span::new(token.span.start, token.span.end),
        message: message.into(),
    }
}

fn name(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(value) | TokenKind::Josa(value) => Some(value),
        _ => None,
    }
}

/// Parse a designated formal root containing exactly one `(pins) 얽음.`.
/// Standalone zero-root source is handled by the caller; it is not a formal root.
/// No arbitrary expressions or calls are evaluated, and nested values retain
/// their token/span authority for later typed interpretation.
pub fn parse_work_root(source: &str) -> Result<RootDirective, ParseError> {
    let tokens = Lexer::new(source).tokenize().map_err(|err| ParseError {
        span: Span::new(err.pos, err.pos + 1),
        message: err.message,
    })?;
    let mut pos = 0;
    if !matches!(tokens[pos].kind, TokenKind::LParen) {
        return Err(error(&tokens[pos], "E_WORK_ROOT_SURFACE: 작품 뿌리는 '(항목=값, ...) 얽음.'으로 작성하세요. 프로젝트/얼개 옛 표면은 허용하지 않습니다"));
    }
    let start = tokens[pos].span.start;
    pos += 1;
    let mut pins = Vec::new();
    let mut seen = BTreeSet::new();
    while !matches!(tokens[pos].kind, TokenKind::RParen) {
        let pin_start = tokens[pos].span.start;
        // Field slots are exact spellings even where the shared lexer assigns
        // a keyword token (for example 시작); do not invent a second lexer.
        let field_spelling = &source[tokens[pos].span.start..tokens[pos].span.end];
        let Some(field) = RootField::from_name(field_spelling) else {
            return Err(error(&tokens[pos], &format!("E_WORK_ROOT_UNKNOWN_FIELD: 얽음은 가지/판본/글판/모듬/내놓음/시작/실행계약/보개만 허용합니다 (입력: {:?})", tokens[pos].raw)));
        };
        if !seen.insert(field) {
            return Err(error(
                &tokens[pos],
                "E_WORK_ROOT_DUPLICATE_FIELD: 뿌리 항목을 중복 지정할 수 없습니다",
            ));
        }
        pos += 1;
        if !matches!(tokens[pos].kind, TokenKind::Equals) {
            return Err(error(
                &tokens[pos],
                "E_WORK_ROOT_PIN: 항목 뒤에는 '='가 필요합니다",
            ));
        }
        pos += 1;
        let value_start = pos;
        let mut closes = Vec::new();
        loop {
            let token = &tokens[pos];
            if closes.is_empty() && matches!(token.kind, TokenKind::Comma | TokenKind::RParen) {
                break;
            }
            match token.kind {
                TokenKind::Eof => {
                    return Err(error(
                        token,
                        "E_WORK_ROOT_UNCLOSED: 뿌리의 괄호가 닫히지 않았습니다",
                    ))
                }
                TokenKind::LParen => closes.push(TokenKind::RParen),
                TokenKind::LBrace => closes.push(TokenKind::RBrace),
                TokenKind::LBracket => closes.push(TokenKind::RBracket),
                TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket => {
                    if closes.pop().as_ref() != Some(&token.kind) {
                        return Err(error(
                            token,
                            "E_WORK_ROOT_DELIMITER: 중첩 괄호의 종류가 맞지 않습니다",
                        ));
                    }
                }
                _ => {}
            }
            pos += 1;
        }
        if value_start == pos {
            return Err(error(
                &tokens[pos],
                "E_WORK_ROOT_VALUE: 항목 값이 비어 있습니다",
            ));
        }
        let value_tokens = tokens[value_start..pos].to_vec();
        let kind = if value_tokens.len() == 1 {
            match &value_tokens[0].kind {
                TokenKind::StringLit(value) => FieldValueKind::Text(value.clone()),
                TokenKind::Ident(value) | TokenKind::Josa(value) => {
                    FieldValueKind::Name(value.clone())
                }
                _ => FieldValueKind::TokenTree,
            }
        } else {
            FieldValueKind::TokenTree
        };
        let end = tokens[pos - 1].span.end;
        pins.push(RootPin {
            field,
            span: Span::new(pin_start, end),
            value: FieldValue {
                kind,
                span: Span::new(tokens[value_start].span.start, end),
                tokens: value_tokens,
            },
        });
        if matches!(tokens[pos].kind, TokenKind::Comma) {
            pos += 1;
            if matches!(tokens[pos].kind, TokenKind::RParen) {
                return Err(error(
                    &tokens[pos],
                    "E_WORK_ROOT_PIN: 마지막 쉼표 뒤의 항목이 없습니다",
                ));
            }
        }
    }
    pos += 1;
    if name(&tokens[pos]) != Some("얽음") {
        return Err(error(
            &tokens[pos],
            "E_WORK_ROOT_SURFACE: 현재 뿌리 지시어는 '얽음'입니다. 얼개는 이관용 옛 표면입니다",
        ));
    }
    pos += 1;
    if !matches!(tokens[pos].kind, TokenKind::Dot) {
        return Err(error(
            &tokens[pos],
            "E_WORK_ROOT_TERMINATOR: 얽음 뒤에 '.'이 필요합니다",
        ));
    }
    let end = tokens[pos].span.end;
    pos += 1;
    if !matches!(tokens[pos].kind, TokenKind::Eof) {
        return Err(error(
            &tokens[pos],
            "E_WORK_ROOT_EXTRA: 지정 뿌리에는 얽음 하나만 허용하며 실행 문장을 둘 수 없습니다",
        ));
    }
    let root = RootDirective {
        span: Span::new(start, end),
        pins,
        source: source.into(),
    };
    if !matches!(root.field(RootField::SourceEpoch).map(|value| &value.kind), Some(FieldValueKind::Text(value)) if value == "v25")
    {
        return Err(error(
            &tokens[0],
            "E_WORK_ROOT_SOURCE_EPOCH: 글판=\"v25\"를 명시해야 합니다",
        ));
    }
    if seen.contains(&RootField::Gaji) != seen.contains(&RootField::Version) {
        return Err(error(
            &tokens[0],
            "E_WORK_ROOT_IDENTITY_PAIR: 가지와 판본은 함께 지정해야 합니다",
        ));
    }
    if seen.contains(&RootField::Exports) && !seen.contains(&RootField::Gaji) {
        return Err(error(
            &tokens[0],
            "E_WORK_ROOT_EXPORTS_GAJI_ONLY: 내놓음은 가지 작품에서만 지정할 수 있습니다",
        ));
    }
    if let Some(value) = root.field(RootField::Gaji) {
        let scoped = match &value.kind {
            FieldValueKind::Text(value) | FieldValueKind::Name(value) => {
                let mut parts = value.split('.');
                matches!((parts.next(), parts.next(), parts.next()), (Some(a), Some(b), None) if !a.is_empty() && !b.is_empty())
            }
            FieldValueKind::TokenTree => {
                value.tokens.len() == 3
                    && name(&value.tokens[0]).is_some()
                    && matches!(value.tokens[1].kind, TokenKind::Dot)
                    && name(&value.tokens[2]).is_some()
            }
        };
        if !scoped {
            return Err(error(
                &tokens[0],
                "E_WORK_ROOT_GAJI_NAME: 선언 가지는 이름범위.가지이름이어야 합니다",
            ));
        }
    }
    let Some(modules) = root.field(RootField::Modules) else {
        return Err(error(
            &tokens[0],
            "E_WORK_ROOT_MODULES_REQUIRED: 정식 작품의 모듬은 비어 있지 않아야 합니다",
        ));
    };
    if matches!(&modules.kind, FieldValueKind::Text(value) if value.is_empty())
        || (modules.tokens.len() == 2
            && matches!(
                modules.tokens[0].kind,
                TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket
            ))
    {
        return Err(error(
            &tokens[0],
            "E_WORK_ROOT_MODULES_EMPTY: 정식 작품의 모듬은 비어 있지 않아야 합니다",
        ));
    }
    root.typed_modules()?;
    root.typed_start()?;
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_tokens_keep_nested_value_spans_without_runtime_parse() {
        let source = "// 뿌리\n(가지=공식.예제, 판본=\"0.1\", 글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.";
        let root = parse_work_root(source).unwrap();
        let modules = root.field(RootField::Modules).unwrap();
        assert_eq!(
            &source[modules.span.start..modules.span.end],
            "차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\")]"
        );
        assert_eq!(root.pins.len(), 5);
        assert_eq!(
            root.field(RootField::Version).unwrap().kind,
            FieldValueKind::Text("0.1".into())
        );
        assert_eq!(root.typed_modules().unwrap()[0].name, "앱");
        assert_eq!(root.typed_start().unwrap().unwrap().definition, "실행");
    }
    #[test]
    fn invalid_roots_fail_closed() {
        for source in [
            "프로젝트 { 판본: \"v25\". }.",
            "(모듬=[앱]) 얼개.",
            "(모듬=[앱], 별명=값) 얽음.",
            "(모듬=[앱], 모듬=[둘]) 얽음.",
            "(가지=공식.예제, 모듬=[앱]) 얽음.",
            "(판본=\"v25\", 모듬=[앱]) 얽음.",
            "(가지=예제, 판본=\"v25\", 모듬=[앱]) 얽음.",
            "(모듬=[]) 얽음.",
            "() 얽음.",
            "(모듬=[앱], 내놓음=[앱]) 얽음.",
            "(모듬=[앱)) 얽음.",
            "(모듬=[앱]) 얽음. 1 보여주기.",
            "(모듬=[앱]) 얽음. (모듬=[둘]) 얽음.",
            "(모듬=[앱],) 얽음.",
            "(모듬=) 얽음.",
        ] {
            assert!(parse_work_root(source).is_err(), "{source}");
        }
    }
    #[test]
    fn syntax_looking_strings_are_not_scanned_as_directives() {
        parse_work_root("(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app.ddn\", 원문=\"프로젝트 {}. (모듬=[]) 얼개.\")]) 얽음.").unwrap();
        assert!(parse_work_root("(모듬=[앱]) 얽음").is_err());
    }

    #[test]
    fn approved_typed_lists_names_and_source_epoch_are_exact() {
        let root = parse_work_root("(글판=\"v25\", 모듬=차림[(이름=시작, 머리=\"app.ddn\", 원문=차림[\"app.ddn\", \"lib/\"])], 시작=(모듈=시작, 정의=시작)) 얽음.").unwrap();
        let modules = root.typed_modules().unwrap();
        assert_eq!(
            modules[0].name, "시작",
            "shared lexer alias must not rewrite authored name"
        );
        assert_eq!(modules[0].sources, vec!["app.ddn", "lib/"]);
        assert_eq!(root.typed_start().unwrap().unwrap().definition, "시작");
        for modules in [
            "차림[]",
            "((이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\"))",
            "차림[(이름=\"앱\", 머리=\"app.ddn\", 원문=\"app.ddn\")]",
            "차림[(이름=앱, 머리=\"app.ddn\")]",
            "차림[(이름=앱, 머리=\"app.ddn\", 원문=차림[])]",
            "차림[(이름=앱, 머리=\"app.ddn\", 원문=(\"app.ddn\"))]",
            "차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\", 별명=앱)]",
            "차림[(이름=앱, 이름=둘, 머리=\"app.ddn\", 원문=\"app.ddn\")]",
        ] {
            assert!(
                parse_work_root(&format!("(글판=\"v25\", 모듬={modules}) 얽음.")).is_err(),
                "{modules}"
            );
        }
        let valid_modules = "차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\")]";
        for start in [
            "앱.실행",
            "(모듈=\"앱\", 정의=실행)",
            "(모듈=앱)",
            "(모듈=앱, 정의=실행, 정의=둘)",
        ] {
            assert!(
                parse_work_root(&format!(
                    "(글판=\"v25\", 모듬={valid_modules}, 시작={start}) 얽음."
                ))
                .is_err(),
                "{start}"
            );
        }
        for epoch in ["v25", "\"v24\"", "\"\""] {
            assert!(
                parse_work_root(&format!("(글판={epoch}, 모듬={valid_modules}) 얽음.")).is_err()
            );
        }
        assert!(parse_work_root(&format!("(모듬={valid_modules}) 얽음.")).is_err());
        assert!(
            parse_work_root(&format!("(글판=\"v25\", 모듬={valid_modules}) 얽음."))
                .unwrap()
                .typed_start()
                .unwrap()
                .is_none()
        );
    }
}
