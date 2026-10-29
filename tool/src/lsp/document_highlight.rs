use super::definition::{bound_name_in_snapshot, ProjectView, SOURCE_PROFILE_V25};
use super::document_store::DocumentSnapshot;
use super::position::{ByteRange, LspPosition, PositionEncoding};
use super::symbol::SymbolError;

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

/// Highlights of the exactly bound name: write (3) for the declaration and a
/// callable's result place, read (2) otherwise. Nothing when not bound exactly.
pub fn document_highlights_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<Option<DocumentHighlightResolution>, SymbolError> {
    let bound = bound_name_in_snapshot(snapshot, position, encoding)?;
    let Ok(bound) = bound else {
        return Ok(None);
    };
    Ok(Some(DocumentHighlightResolution {
        document_version: snapshot.version,
        snapshot_identity: snapshot.identity.as_str().to_string(),
        identity: bound.identity,
        highlights: bound
            .occurrences
            .iter()
            .map(|o| DocumentHighlightRange { range: o.range, kind: if o.write { 3 } else { 2 } })
            .collect(),
        source_profile: SOURCE_PROFILE_V25,
    }))
}

/// Project member: the sites of a completely censused parameter or local in
/// the requested document; nothing otherwise.
pub fn project_document_highlights(
    view: &ProjectView,
    snapshot: &DocumentSnapshot,
    offset: usize,
) -> Option<DocumentHighlightResolution> {
    let target = view.target_at(offset).ok()?;
    let sites = view.local_sites(&target).ok()?;
    let highlights = sites
        .iter()
        .filter(|site| site.path() == view.document_path)
        .map(|site| DocumentHighlightRange {
            range: ByteRange { start: site.span().start, end: site.span().end },
            kind: if site == target.definition() { 3 } else { 2 },
        })
        .collect();
    Some(DocumentHighlightResolution {
        document_version: snapshot.version,
        snapshot_identity: snapshot.identity.as_str().to_string(),
        identity: ProjectView::identity(target.definition()),
        highlights,
        source_profile: SOURCE_PROFILE_V25,
    })
}
