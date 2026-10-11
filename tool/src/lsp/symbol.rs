use crate::lsp::{
    position_to_byte_offset, ByteRange, DocumentSnapshot, LspPosition, PositionEncoding,
    PositionError,
};
use ddonirang_lang::stdlib::{canonicalize_stdlib_alias, minimal_stdlib_sigs};
use ddonirang_lang::{
    parse_v25, Lexer, SeedDef, SeedKind, Token, TokenKind, TopLevelItem, TypeRef,
};
use std::fmt;

pub const SOURCE_PROFILE_V25: &str = "v25";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolResolution {
    pub document_version: i64,
    pub snapshot_identity: String,
    pub position: LspPosition,
    pub identity: String,
    pub kind: &'static str,
    pub signature: String,
    pub reference_range: ByteRange,
    pub definition_range: Option<ByteRange>,
    pub source_profile: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbolResolution {
    pub uri: String,
    pub document_version: i64,
    pub snapshot_identity: String,
    pub identity: String,
    pub name: String,
    pub kind: u32,
    pub full_range: ByteRange,
    pub selection_range: ByteRange,
    pub source_profile: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDefinitionResolution {
    pub uri: String,
    pub document_version: i64,
    pub snapshot_identity: String,
    pub identity: String,
    pub name: String,
    pub range: ByteRange,
    pub source_profile: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallHierarchyCall {
    pub caller_identity: String,
    pub callee_name: String,
    pub range: ByteRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalDefinition {
    identity: String,
    signature: String,
    definition_range: Option<ByteRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolError {
    InvalidUtf8,
    InvalidPosition(PositionError),
}

impl fmt::Display for SymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 => f.write_str("symbol source가 UTF-8이 아닙니다"),
            Self::InvalidPosition(error) => write!(f, "symbol position이 잘못되었습니다: {error}"),
        }
    }
}

impl std::error::Error for SymbolError {}

pub fn resolve_symbol(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<Option<SymbolResolution>, SymbolError> {
    let source = snapshot.text().map_err(|_| SymbolError::InvalidUtf8)?;
    let cursor = position_to_byte_offset(source, position, encoding)
        .map_err(SymbolError::InvalidPosition)?;
    if snapshot.language_id != "ddn" {
        return Ok(None);
    }

    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|_| SymbolError::InvalidUtf8)?;
    let Some(token) = token_at(&tokens, cursor) else {
        return Ok(None);
    };
    let Some(raw_name) = token_name(token) else {
        return Ok(None);
    };
    if let Some(local) = local_definition(snapshot, raw_name)? {
        return Ok(Some(SymbolResolution {
            document_version: snapshot.version,
            snapshot_identity: snapshot.identity.as_str().to_string(),
            position,
            identity: local.identity,
            kind: "local_definition",
            signature: local.signature,
            reference_range: ByteRange {
                start: token.span.start,
                end: token.span.end,
            },
            definition_range: local.definition_range,
            source_profile: SOURCE_PROFILE_V25,
        }));
    }

    for signature in minimal_stdlib_sigs() {
        if canonicalize_stdlib_alias(signature.name) != signature.name || signature.name != raw_name
        {
            continue;
        }
        return Ok(Some(SymbolResolution {
            document_version: snapshot.version,
            snapshot_identity: snapshot.identity.as_str().to_string(),
            position,
            identity: format!("definition:stdlib:{}", signature.name),
            kind: "stdlib_callable",
            signature: format!(
                "{}({}) -> {}",
                signature.name,
                signature.params.join(", "),
                signature.ret
            ),
            reference_range: ByteRange {
                start: token.span.start,
                end: token.span.end,
            },
            definition_range: None,
            source_profile: SOURCE_PROFILE_V25,
        }));
    }

    Ok(None)
}

pub fn resolve_symbol_across_documents(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
    documents: &[&DocumentSnapshot],
) -> Result<Option<SymbolResolution>, SymbolError> {
    let source = snapshot.text().map_err(|_| SymbolError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(None);
    }
    let cursor = position_to_byte_offset(source, position, encoding)
        .map_err(SymbolError::InvalidPosition)?;
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|_| SymbolError::InvalidUtf8)?;
    let Some(token) = token_at(&tokens, cursor) else {
        return Ok(None);
    };
    let Some(raw_name) = token_name(token) else {
        return Ok(None);
    };
    let mut candidates = Vec::new();
    for document in documents {
        let Some(local) = local_definition(document, raw_name)? else {
            continue;
        };
        candidates.push(SymbolResolution {
            document_version: document.version,
            snapshot_identity: document.identity.as_str().to_string(),
            position,
            identity: local.identity,
            kind: "local_definition",
            signature: local.signature,
            reference_range: ByteRange {
                start: token.span.start,
                end: token.span.end,
            },
            definition_range: local.definition_range,
            source_profile: SOURCE_PROFILE_V25,
        });
    }
    if candidates.len() == 1 {
        Ok(candidates.pop())
    } else if candidates.len() > 1 {
        Ok(None)
    } else {
        resolve_symbol(snapshot, position, encoding)
    }
}

pub fn resolve_type_definition_across_documents(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
    documents: &[&DocumentSnapshot],
) -> Result<Option<TypeDefinitionResolution>, SymbolError> {
    let source = snapshot.text().map_err(|_| SymbolError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(None);
    }
    let cursor = position_to_byte_offset(source, position, encoding)
        .map_err(SymbolError::InvalidPosition)?;
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|_| SymbolError::InvalidUtf8)?;
    let Some((token_index, token)) = tokens
        .iter()
        .enumerate()
        .find(|(_, token)| token.span.start <= cursor && cursor < token.span.end)
    else {
        return Ok(None);
    };
    let Some(type_name) = token_name(token) else {
        return Ok(None);
    };
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(None),
    };
    if !is_type_reference_token(&tokens, token_index, &program, type_name) {
        return Ok(None);
    }

    let mut candidates = Vec::new();
    for document in documents {
        for (name, identity, range) in collect_local_type_definitions(document)? {
            if name == type_name {
                candidates.push(TypeDefinitionResolution {
                    uri: document.uri.clone(),
                    document_version: document.version,
                    snapshot_identity: document.identity.as_str().to_string(),
                    identity,
                    name,
                    range,
                    source_profile: SOURCE_PROFILE_V25,
                });
            }
        }
    }
    if candidates.len() == 1 {
        Ok(candidates.pop())
    } else {
        Ok(None)
    }
}

fn collect_local_type_definitions(
    snapshot: &DocumentSnapshot,
) -> Result<Vec<(String, String, ByteRange)>, SymbolError> {
    let source = snapshot.text().map_err(|_| SymbolError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(Vec::new());
    }
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|_| SymbolError::InvalidUtf8)?;
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(Vec::new()),
    };
    Ok(program
        .items
        .iter()
        .filter_map(|item| {
            let TopLevelItem::SeedDef(seed) = item;
            if !matches!(&seed.seed_kind, SeedKind::Imeumssi) {
                return None;
            }
            declaration_range(&tokens, seed).map(|range| {
                (
                    seed.canonical_name.clone(),
                    format!("type:local:{}", seed.canonical_name),
                    range,
                )
            })
        })
        .collect())
}

fn is_type_reference_token(
    tokens: &[Token],
    token_index: usize,
    program: &ddonirang_lang::CanonProgram,
    type_name: &str,
) -> bool {
    if token_name(&tokens[token_index]) != Some(type_name) {
        return false;
    }
    if token_is_seed_kind(tokens, token_index, program) {
        return false;
    }
    let mut paren_depth = 0usize;
    for index in (0..token_index).rev() {
        match &tokens[index].kind {
            TokenKind::RParen => paren_depth += 1,
            TokenKind::LParen if paren_depth > 0 => paren_depth -= 1,
            TokenKind::Colon if paren_depth == 0 => return true,
            TokenKind::Comma if paren_depth == 0 => return false,
            TokenKind::Define
            | TokenKind::Equals
            | TokenKind::Dot
            | TokenKind::LBrace
            | TokenKind::RBrace
            | TokenKind::Semicolon
                if paren_depth == 0 => return false,
            _ => {}
        }
    }
    false
}

fn token_is_seed_kind(
    tokens: &[Token],
    token_index: usize,
    program: &ddonirang_lang::CanonProgram,
) -> bool {
    program.items.iter().any(|item| {
        let TopLevelItem::SeedDef(seed) = item;
        let Some(name_index) = tokens.iter().position(|token| {
            token.span.start >= seed.span.start
                && token.span.end <= seed.span.end
                && token.raw == seed.canonical_name
                && token_name(token).is_some()
        }) else {
            return false;
        };
        let Some(colon_index) = tokens
            .iter()
            .enumerate()
            .skip(name_index + 1)
            .find_map(|(index, token)| {
                (index < tokens.len()
                    && matches!(&token.kind, TokenKind::Colon)
                    && token.span.start < seed.span.end)
                    .then_some(index)
            })
        else {
            return false;
        };
        let Some(define_index) = tokens
            .iter()
            .enumerate()
            .skip(colon_index + 1)
            .find_map(|(index, token)| {
                (token.span.start < seed.span.end
                    && matches!(&token.kind, TokenKind::Define | TokenKind::Equals))
                    .then_some(index)
            })
        else {
            return false;
        };
        token_index > colon_index && token_index < define_index
    })
}

pub fn collect_document_symbols(
    snapshot: &DocumentSnapshot,
) -> Result<Vec<DocumentSymbolResolution>, SymbolError> {
    let source = snapshot.text().map_err(|_| SymbolError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(Vec::new());
    }
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|_| SymbolError::InvalidUtf8)?;
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(Vec::new()),
    };
    let mut symbols = Vec::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        let Some(selection_range) = declaration_range(&tokens, seed) else {
            continue;
        };
        symbols.push(DocumentSymbolResolution {
            uri: snapshot.uri.clone(),
            document_version: snapshot.version,
            snapshot_identity: snapshot.identity.as_str().to_string(),
            identity: format!("definition:local:{}", seed.canonical_name),
            name: seed.canonical_name.clone(),
            kind: lsp_symbol_kind(&seed.seed_kind),
            full_range: ByteRange {
                start: seed.span.start,
                end: seed.span.end,
            },
            selection_range,
            source_profile: SOURCE_PROFILE_V25,
        });
    }
    Ok(symbols)
}

pub fn collect_local_references(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
    include_declaration: bool,
) -> Result<Vec<ByteRange>, SymbolError> {
    let Some(resolution) = resolve_symbol(snapshot, position, encoding)? else {
        return Ok(Vec::new());
    };
    if !resolution.identity.starts_with("definition:local:") {
        return Ok(Vec::new());
    }
    collect_local_references_for_identity(snapshot, &resolution.identity, include_declaration)
}

pub fn collect_local_references_for_identity(
    snapshot: &DocumentSnapshot,
    identity: &str,
    include_declaration: bool,
) -> Result<Vec<ByteRange>, SymbolError> {
    let Some(name) = identity.strip_prefix("definition:local:") else {
        return Ok(Vec::new());
    };
    let source = snapshot.text().map_err(|_| SymbolError::InvalidUtf8)?;
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|_| SymbolError::InvalidUtf8)?;
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(Vec::new()),
    };
    let declaration = program.items.iter().find_map(|item| {
        let TopLevelItem::SeedDef(seed) = item;
        (seed.canonical_name == name).then(|| declaration_range(&tokens, seed))
    });
    let declaration = declaration.flatten();
    Ok(tokens
        .iter()
        .filter(|token| token_name(token) == Some(name))
        .map(|token| ByteRange {
            start: token.span.start,
            end: token.span.end,
        })
        .filter(|range| include_declaration || Some(*range) != declaration)
        .collect())
}

pub fn collect_call_hierarchy_calls(
    snapshot: &DocumentSnapshot,
) -> Result<Vec<CallHierarchyCall>, SymbolError> {
    let source = snapshot.text().map_err(|_| SymbolError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(Vec::new());
    }
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|_| SymbolError::InvalidUtf8)?;
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(Vec::new()),
    };
    let mut calls = Vec::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        let Some(body) = &seed.body else {
            continue;
        };
        walk_body(
            body,
            &format!("definition:local:{}", seed.canonical_name),
            &tokens,
            &mut calls,
        );
    }
    calls.sort_by_key(|call| (call.range.start, call.range.end));
    Ok(calls)
}

pub fn call_hierarchy_call_at_position(
    snapshot: &DocumentSnapshot,
    cursor: usize,
) -> Result<Option<CallHierarchyCall>, SymbolError> {
    Ok(collect_call_hierarchy_calls(snapshot)?
        .into_iter()
        .find(|call| call.range.start <= cursor && cursor < call.range.end))
}

fn walk_body(
    body: &ddonirang_lang::Body,
    caller_identity: &str,
    tokens: &[Token],
    calls: &mut Vec<CallHierarchyCall>,
) {
    for statement in &body.stmts {
        walk_stmt(statement, caller_identity, tokens, calls);
    }
}

fn walk_stmt(
    statement: &ddonirang_lang::Stmt,
    caller_identity: &str,
    tokens: &[Token],
    calls: &mut Vec<CallHierarchyCall>,
) {
    use ddonirang_lang::Stmt;
    match statement {
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    walk_expr(value, caller_identity, tokens, calls);
                }
            }
        }
        Stmt::Definition { value, .. } => walk_expr(value, caller_identity, tokens, calls),
        Stmt::Mutate { target, value, .. } => {
            walk_expr(target, caller_identity, tokens, calls);
            walk_expr(value, caller_identity, tokens, calls);
        }
        Stmt::ResultFlow {
            producer,
            destination,
            ..
        } => {
            walk_expr(producer, caller_identity, tokens, calls);
            walk_expr(destination, caller_identity, tokens, calls);
        }
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            walk_expr(expr, caller_identity, tokens, calls)
        }
        Stmt::Receive {
            sender,
            condition,
            body,
            ..
        } => {
            if let Some(sender) = sender {
                walk_expr(sender, caller_identity, tokens, calls);
            }
            if let Some(condition) = condition {
                walk_expr(condition, caller_identity, tokens, calls);
            }
            walk_body(body, caller_identity, tokens, calls);
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            if let Some(sender) = sender {
                walk_expr(sender, caller_identity, tokens, calls);
            }
            walk_expr(payload, caller_identity, tokens, calls);
            walk_expr(receiver, caller_identity, tokens, calls);
        }
        Stmt::PublicObservation { fields, .. } => {
            for field in fields {
                walk_expr(&field.value, caller_identity, tokens, calls);
            }
        }
        Stmt::Return { value, .. } => walk_expr(value, caller_identity, tokens, calls),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            walk_expr(condition, caller_identity, tokens, calls);
            walk_body(then_body, caller_identity, tokens, calls);
            if let Some(else_body) = else_body {
                walk_body(else_body, caller_identity, tokens, calls);
            }
        }
        Stmt::Try { action, body, .. } => {
            walk_expr(action, caller_identity, tokens, calls);
            walk_body(body, caller_identity, tokens, calls);
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                walk_expr(&branch.condition, caller_identity, tokens, calls);
                walk_body(&branch.body, caller_identity, tokens, calls);
            }
            walk_body(else_body, caller_identity, tokens, calls);
        }
        Stmt::Repeat { body, .. }
        | Stmt::Quantifier { body, .. }
        | Stmt::BeatBlock { body, .. }
        | Stmt::Hook { body, .. } => walk_body(body, caller_identity, tokens, calls),
        Stmt::While {
            condition, body, ..
        }
        | Stmt::Guard {
            condition, body, ..
        }
        | Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => {
            walk_expr(condition, caller_identity, tokens, calls);
            walk_body(body, caller_identity, tokens, calls);
        }
        Stmt::ForEach { iterable, body, .. } => {
            walk_expr(iterable, caller_identity, tokens, calls);
            walk_body(body, caller_identity, tokens, calls);
        }
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            walk_expr(condition, caller_identity, tokens, calls);
            if let Some(then_body) = then_body {
                walk_body(then_body, caller_identity, tokens, calls);
            }
            walk_body(else_body, caller_identity, tokens, calls);
        }
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => {}
    }
}

fn walk_expr(
    expression: &ddonirang_lang::Expr,
    caller_identity: &str,
    tokens: &[Token],
    calls: &mut Vec<CallHierarchyCall>,
) {
    use ddonirang_lang::ExprKind;
    match &expression.kind {
        ExprKind::Call { args, func } => {
            if let Some(range) = call_name_range(tokens, expression.span, func) {
                calls.push(CallHierarchyCall {
                    caller_identity: caller_identity.to_string(),
                    callee_name: func.clone(),
                    range,
                });
            }
            for arg in args {
                walk_expr(&arg.expr, caller_identity, tokens, calls);
            }
        }
        ExprKind::CollectionConstructor { elements } => {
            for element in elements {
                walk_expr(element, caller_identity, tokens, calls);
            }
        }
        ExprKind::SelectionPlace { target, index } => {
            walk_expr(target, caller_identity, tokens, calls);
            walk_expr(index, caller_identity, tokens, calls);
        }
        ExprKind::FieldAccess { target, .. }
        | ExprKind::Suffix { value: target, .. }
        | ExprKind::Nuance { expr: target, .. } => {
            walk_expr(target, caller_identity, tokens, calls)
        }
        ExprKind::SeedLiteral { body, .. } => walk_expr(body, caller_identity, tokens, calls),
        ExprKind::Infix { left, right, .. } => {
            walk_expr(left, caller_identity, tokens, calls);
            walk_expr(right, caller_identity, tokens, calls);
        }
        ExprKind::Thunk(body) => walk_body(body, caller_identity, tokens, calls),
        ExprKind::Eval { thunk, .. } => walk_expr(thunk, caller_identity, tokens, calls),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                walk_expr(stage, caller_identity, tokens, calls);
            }
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                walk_expr(value, caller_identity, tokens, calls);
            }
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                walk_expr(value, caller_identity, tokens, calls);
            }
        }
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Assertion(_)
        | ExprKind::StateMachine(_)
        | ExprKind::Formula(_)
        | ExprKind::Template(_) => {}
    }
}

fn call_name_range(tokens: &[Token], span: ddonirang_lang::Span, func: &str) -> Option<ByteRange> {
    tokens
        .iter()
        .find(|token| {
            token.span.start >= span.start
                && token.span.end <= span.end
                && token.raw == func
                && token_name(token).is_some()
        })
        .map(|token| ByteRange {
            start: token.span.start,
            end: token.span.end,
        })
}

fn local_definition(
    snapshot: &DocumentSnapshot,
    name: &str,
) -> Result<Option<LocalDefinition>, SymbolError> {
    let source = snapshot.text().map_err(|_| SymbolError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(None);
    }
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|_| SymbolError::InvalidUtf8)?;
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(None),
    };
    let definitions = program
        .items
        .iter()
        .filter_map(|item| {
            let TopLevelItem::SeedDef(seed) = item;
            (seed.canonical_name == name).then(|| LocalDefinition {
                identity: format!("definition:local:{}", seed.canonical_name),
                signature: local_signature(seed),
                definition_range: declaration_range(&tokens, seed),
            })
        })
        .collect::<Vec<_>>();
    match definitions.as_slice() {
        [definition] => Ok(Some(definition.clone())),
        _ => Ok(None),
    }
}

fn token_at(tokens: &[Token], cursor: usize) -> Option<&Token> {
    tokens
        .iter()
        .find(|token| token.span.start <= cursor && cursor < token.span.end)
}

fn token_name(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(_) | TokenKind::Josa(_) | TokenKind::Variable(_) => {
            Some(token.raw.as_str())
        }
        _ => None,
    }
}

pub(crate) fn declaration_range(tokens: &[Token], seed: &SeedDef) -> Option<ByteRange> {
    let in_seed = tokens.iter().enumerate().filter(|(_, token)| {
        token.span.start >= seed.span.start && token.span.end <= seed.span.end
    });
    for (index, token) in in_seed {
        if token.raw != seed.canonical_name {
            continue;
        }
        let Some(next) = tokens.get(index + 1) else {
            continue;
        };
        if !matches!(&next.kind, TokenKind::Colon) {
            continue;
        }
        let has_definition_operator = tokens[index + 2..]
            .iter()
            .take_while(|candidate| candidate.span.start < seed.span.end)
            .any(|candidate| matches!(&candidate.kind, TokenKind::Define));
        if has_definition_operator {
            return Some(ByteRange {
                start: token.span.start,
                end: token.span.end,
            });
        }
    }
    None
}

fn lsp_symbol_kind(kind: &SeedKind) -> u32 {
    match kind {
        SeedKind::Imeumssi => 13,
        _ => 12,
    }
}

pub(crate) fn local_signature(seed: &SeedDef) -> String {
    let kind = seed_kind_surface(&seed.seed_kind);
    let params = if seed.params.is_empty() {
        String::new()
    } else {
        format!(
            "({})",
            seed.params
                .iter()
                .map(|param| format!("{}:{}", param.pin_name, type_surface(&param.type_ref)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!("{}{}:{} := …", seed.canonical_name, params, kind)
}

fn seed_kind_surface(kind: &SeedKind) -> &str {
    match kind {
        SeedKind::Imeumssi => "이름씨",
        SeedKind::Umjikssi => "움직씨",
        SeedKind::ValueFunc => "값함수",
        SeedKind::Gallaessi => "갈래씨",
        SeedKind::Relationssi => "관계씨",
        SeedKind::Sam => "삼",
        SeedKind::Heureumssi => "흐름씨",
        SeedKind::Ieumssi => "이음씨",
        SeedKind::Semssi => "셈씨",
        SeedKind::Named(name) => name.as_str(),
    }
}

pub(crate) fn type_surface(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Applied { name, args } => format!(
            "{}({})",
            name,
            args.iter().map(type_surface).collect::<Vec<_>>().join(", ")
        ),
        TypeRef::Function { params, result } => format!(
            "({}) -> {}",
            params
                .iter()
                .map(type_surface)
                .collect::<Vec<_>>()
                .join(", "),
            type_surface(result)
        ),
        TypeRef::Infer => "_".to_string(),
    }
}
