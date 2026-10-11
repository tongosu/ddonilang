use super::document_store::DocumentSnapshot;
use super::position::{ByteRange, LspPosition, PositionEncoding};
use super::symbol::collect_local_references;
pub use super::symbol::SymbolError;

pub fn references_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
    include_declaration: bool,
) -> Result<Vec<ByteRange>, SymbolError> {
    collect_local_references(snapshot, position, encoding, include_declaration)
}
