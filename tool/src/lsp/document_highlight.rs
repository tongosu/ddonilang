use super::document_store::DocumentSnapshot;
use super::position::{ByteRange, LspPosition, PositionEncoding};
use super::symbol::{collect_local_references, resolve_symbol, SymbolError};

pub use super::symbol::SymbolError as DocumentHighlightError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentHighlightResolution {
    pub document_version: i64,
    pub snapshot_identity: String,
    pub identity: String,
    pub highlights: Vec<DocumentHighlightRange>,
    pub source_profile: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentHighlightRange {
    pub range: ByteRange,
    pub kind: u32,
}

pub fn document_highlights_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<Option<DocumentHighlightResolution>, SymbolError> {
    let Some(symbol) = resolve_symbol(snapshot, position, encoding)? else {
        return Ok(None);
    };
    if !symbol.identity.starts_with("definition:local:") {
        return Ok(None);
    }
    let declaration = symbol.definition_range;
    let ranges = collect_local_references(snapshot, position, encoding, true)?;
    Ok(Some(DocumentHighlightResolution {
        document_version: symbol.document_version,
        snapshot_identity: symbol.snapshot_identity,
        identity: symbol.identity,
        highlights: ranges
            .into_iter()
            .map(|range| DocumentHighlightRange {
                kind: if Some(range) == declaration { 3 } else { 2 },
                range,
            })
            .collect(),
        source_profile: symbol.source_profile,
    }))
}
