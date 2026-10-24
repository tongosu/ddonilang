use super::position::{ByteRange, LspPosition, PositionEncoding};
use super::symbol::{resolve_symbol, SymbolError};
use super::DocumentSnapshot;

pub use super::symbol::SymbolError as DefinitionError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionResult {
    pub document_version: i64,
    pub snapshot_identity: String,
    pub position: LspPosition,
    pub identity: String,
    pub range: ByteRange,
    pub source_profile: &'static str,
}

pub fn definition_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<Option<DefinitionResult>, DefinitionError> {
    let Some(symbol) =
        resolve_symbol(snapshot, position, encoding).map_err(|error| match error {
            SymbolError::InvalidUtf8 => DefinitionError::InvalidUtf8,
            SymbolError::InvalidPosition(error) => DefinitionError::InvalidPosition(error),
        })?
    else {
        return Ok(None);
    };
    let Some(range) = symbol.definition_range else {
        return Ok(None);
    };
    Ok(Some(DefinitionResult {
        document_version: symbol.document_version,
        snapshot_identity: symbol.snapshot_identity,
        position: symbol.position,
        identity: symbol.identity,
        range,
        source_profile: symbol.source_profile,
    }))
}
