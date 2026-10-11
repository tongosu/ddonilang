use super::document_store::DocumentSnapshot;
use super::position::{ByteRange, LspPosition, PositionEncoding};
use super::symbol::{
    collect_document_symbols, collect_local_references, collect_local_references_for_identity,
    resolve_symbol, resolve_symbol_across_documents,
};
use ddonirang_lang::{parse_v25, Lexer, TokenKind, TopLevelItem};

pub use super::symbol::SymbolError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareRenameResolution {
    pub document_version: i64,
    pub snapshot_identity: String,
    pub identity: String,
    pub range: ByteRange,
    pub source_profile: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameResolution {
    pub document_version: i64,
    pub snapshot_identity: String,
    pub identity: String,
    pub ranges: Vec<ByteRange>,
    pub source_profile: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameDocumentResolution {
    pub uri: String,
    pub document_version: i64,
    pub snapshot_identity: String,
    pub ranges: Vec<ByteRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRenameResolution {
    pub identity: String,
    pub source_profile: &'static str,
    pub documents: Vec<RenameDocumentResolution>,
}

pub fn prepare_rename_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<Option<PrepareRenameResolution>, SymbolError> {
    let Some(symbol) = resolve_symbol(snapshot, position, encoding)? else {
        return Ok(None);
    };
    if !is_local_identity(&symbol.identity) {
        return Ok(None);
    }
    Ok(Some(PrepareRenameResolution {
        document_version: symbol.document_version,
        snapshot_identity: symbol.snapshot_identity,
        identity: symbol.identity,
        range: symbol.reference_range,
        source_profile: symbol.source_profile,
    }))
}

pub fn rename_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
    new_name: &str,
) -> Result<Option<RenameResolution>, SymbolError> {
    if !valid_identifier(new_name) {
        return Ok(None);
    }
    let Some(symbol) = resolve_symbol(snapshot, position, encoding)? else {
        return Ok(None);
    };
    if !is_local_identity(&symbol.identity) {
        return Ok(None);
    }
    let source = snapshot.text().map_err(|_| SymbolError::InvalidUtf8)?;
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(None),
    };
    let Some(old_name) = symbol.identity.strip_prefix("definition:local:") else {
        return Ok(None);
    };
    if program.items.iter().any(|item| {
        let TopLevelItem::SeedDef(seed) = item;
        seed.canonical_name == new_name && seed.canonical_name != old_name
    }) {
        return Ok(None);
    }
    let ranges = collect_local_references(snapshot, position, encoding, true)?;
    if ranges.is_empty() {
        return Ok(None);
    }
    Ok(Some(RenameResolution {
        document_version: symbol.document_version,
        snapshot_identity: symbol.snapshot_identity,
        identity: symbol.identity,
        ranges,
        source_profile: symbol.source_profile,
    }))
}

pub fn prepare_rename_across_documents(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
    documents: &[&DocumentSnapshot],
) -> Result<Option<PrepareRenameResolution>, SymbolError> {
    let Some(symbol) = resolve_symbol_across_documents(snapshot, position, encoding, documents)?
    else {
        return Ok(None);
    };
    if !is_local_identity(&symbol.identity) {
        return Ok(None);
    }
    Ok(Some(PrepareRenameResolution {
        document_version: symbol.document_version,
        snapshot_identity: symbol.snapshot_identity,
        identity: symbol.identity,
        range: symbol.reference_range,
        source_profile: symbol.source_profile,
    }))
}

pub fn rename_across_documents(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
    new_name: &str,
    documents: &[&DocumentSnapshot],
) -> Result<Option<WorkspaceRenameResolution>, SymbolError> {
    if !valid_identifier(new_name) {
        return Ok(None);
    }
    let Some(symbol) = resolve_symbol_across_documents(snapshot, position, encoding, documents)?
    else {
        return Ok(None);
    };
    if !is_local_identity(&symbol.identity) {
        return Ok(None);
    }
    if documents.iter().any(|document| {
        collect_document_symbols(document)
            .map(|symbols| {
                symbols.iter().any(|candidate| {
                    candidate.name == new_name && candidate.identity != symbol.identity
                })
            })
            .unwrap_or(true)
    }) {
        return Ok(None);
    }

    let mut changed_documents = Vec::new();
    for document in documents {
        let ranges = collect_local_references_for_identity(document, &symbol.identity, true)?;
        if ranges
            .iter()
            .any(|range| !document.byte_range_is_valid(*range))
        {
            return Ok(None);
        }
        if !ranges.is_empty() {
            changed_documents.push(RenameDocumentResolution {
                uri: document.uri.clone(),
                document_version: document.version,
                snapshot_identity: document.identity.as_str().to_string(),
                ranges,
            });
        }
    }
    if changed_documents.is_empty() {
        return Ok(None);
    }
    Ok(Some(WorkspaceRenameResolution {
        identity: symbol.identity,
        source_profile: symbol.source_profile,
        documents: changed_documents,
    }))
}

fn is_local_identity(identity: &str) -> bool {
    identity.starts_with("definition:local:")
}

fn valid_identifier(new_name: &str) -> bool {
    let Ok(tokens) = Lexer::new(new_name).tokenize() else {
        return false;
    };
    tokens.len() == 2
        && matches!(&tokens[0].kind, TokenKind::Ident(_) | TokenKind::Josa(_))
        && tokens[0].raw == new_name
}
