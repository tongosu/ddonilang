use super::definition::{bound_name_in_snapshot, ProjectView, SiteLocation};
use super::document_store::DocumentSnapshot;
use super::position::{ByteRange, LspPosition, PositionEncoding};
pub use super::symbol::SymbolError;

/// Every occurrence of the exactly bound name in this document; empty when the
/// name is not bound exactly (no spelling-based guess, no other files).
pub fn references_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
    include_declaration: bool,
) -> Result<Vec<ByteRange>, SymbolError> {
    let bound = bound_name_in_snapshot(snapshot, position, encoding)?;
    let Ok(bound) = bound else {
        return Ok(Vec::new());
    };
    Ok(bound
        .occurrences
        .iter()
        .filter(|o| include_declaration || o.range != bound.declaration)
        .map(|o| o.range)
        .collect())
}

/// Project member: every site of a parameter or local whose complete census
/// the common project binding supplies; empty otherwise (callables included,
/// whose call stems the census does not cover).
pub fn project_references(
    view: &ProjectView,
    offset: usize,
    include_declaration: bool,
) -> Vec<SiteLocation> {
    let Ok(target) = view.target_at(offset) else {
        return Vec::new();
    };
    let Ok(sites) = view.local_sites(&target) else {
        return Vec::new();
    };
    let locations: Option<Vec<SiteLocation>> = sites
        .iter()
        .filter(|site| include_declaration || *site != target.definition())
        .map(|site| view.location(site))
        .collect();
    locations.unwrap_or_default()
}
