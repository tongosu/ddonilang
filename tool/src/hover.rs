use super::position::{ByteRange, LspPosition, PositionEncoding};
use super::symbol::resolve_symbol;
use super::DocumentSnapshot;

pub use super::symbol::SymbolError as HoverError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverResult {
    pub document_version: i64,
    pub snapshot_identity: String,
    pub position: LspPosition,
    pub identity: String,
    pub kind: &'static str,
    pub signature: String,
    pub range: ByteRange,
    pub source_profile: &'static str,
}

pub fn hover_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<Option<HoverResult>, HoverError> {
    let Some(symbol) = resolve_symbol(snapshot, position, encoding)? else {
        return Ok(None);
    };
    Ok(Some(HoverResult {
        document_version: symbol.document_version,
        snapshot_identity: symbol.snapshot_identity,
        position: symbol.position,
        identity: symbol.identity,
        kind: symbol.kind,
        signature: symbol.signature,
        range: symbol.reference_range,
        source_profile: symbol.source_profile,
    }))
}
