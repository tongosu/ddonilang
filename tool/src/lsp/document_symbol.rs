use super::document_store::DocumentSnapshot;
pub use super::symbol::SymbolError;
use super::symbol::{collect_document_symbols, DocumentSymbolResolution};

pub fn document_symbols_snapshot(
    snapshot: &DocumentSnapshot,
) -> Result<Vec<DocumentSymbolResolution>, SymbolError> {
    collect_document_symbols(snapshot)
}
