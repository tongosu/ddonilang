//! Definition, references and highlights share one rule: a name is navigable
//! only when the common producers bind it exactly in this document revision.
//!
//! - A top-level callable: name binding A1 `Exact` (declaration, result place,
//!   `…하기` call stem, value use).
//! - A parameter or local: the common lexical facts give `Final` and
//!   `exact_local_occurrences` supplies the complete name-token set.
//!
//! Candidates, unresolved or erroneous uses, other files and spelling matches
//! give no result rather than a wrong jump.
use super::document_store::DocumentSnapshot;
use super::position::{position_to_byte_offset, ByteRange, LspPosition, PositionEncoding};
use crate::name_binding::final_bindings::{self, BindingState, DefinitionKind as FactKind};
use crate::name_binding::token_ranges::analyze_name_tokens;
use crate::name_binding::{bind_document as bind_callables, OccurrenceRole, Resolution};

pub const SOURCE_PROFILE_V25: &str = "v25";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundKind {
    TopLevelCallable,
    Parameter,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundOccurrence {
    pub range: ByteRange,
    /// A write: the declaration or a callable's result place.
    pub write: bool,
}

/// One exactly bound name in one source revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundName {
    pub identity: String,
    pub name: String,
    pub kind: BoundKind,
    pub declaration: ByteRange,
    /// Every occurrence in this document, sorted, declaration included.
    pub occurrences: Vec<BoundOccurrence>,
}

/// Why a position is not navigable; used for user-facing refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotBound {
    NotAName,
    SourceNotAccepted,
    CallableNotExact,
    NotFinal,
    LocalNotComplete,
}

pub fn bound_name_at(source: &str, path: &str, offset: usize) -> Result<BoundName, NotBound> {
    let mut last = NotBound::NotAName;
    for probe in probe_offsets(source, offset) {
        match bound_name_exactly_at(source, path, probe) {
            Ok(bound) => return Ok(bound),
            Err(NotBound::NotAName) => {}
            Err(other) => last = other,
        }
    }
    Err(last)
}

/// The cursor may sit just after a name.
fn probe_offsets(source: &str, offset: usize) -> Vec<usize> {
    let mut offsets = vec![offset];
    if offset > 0 {
        let mut previous = offset - 1;
        while previous > 0 && !source.is_char_boundary(previous) {
            previous -= 1;
        }
        offsets.push(previous);
    }
    offsets
}

fn bound_name_exactly_at(source: &str, path: &str, offset: usize) -> Result<BoundName, NotBound> {
    let callables = bind_callables(source, path);
    if callables.outcome != crate::name_binding::BindingOutcome::Analyzed {
        return Err(NotBound::SourceNotAccepted);
    }
    if let Some((definition, _)) = callables.occurrence_at(offset) {
        if let Resolution::Exact(occurrences) = &definition.resolution {
            let range = |r: &crate::name_binding::ByteRange| ByteRange { start: r.start, end: r.end };
            let declaration = occurrences
                .iter()
                .find(|o| o.role == OccurrenceRole::Declaration)
                .map(|o| range(&o.name_range))
                .ok_or(NotBound::CallableNotExact)?;
            return Ok(BoundName {
                identity: format!("callable@{}:{}", declaration.start, declaration.end),
                name: definition.name.clone(),
                kind: BoundKind::TopLevelCallable,
                declaration,
                occurrences: occurrences
                    .iter()
                    .map(|o| BoundOccurrence {
                        range: range(&o.name_range),
                        write: matches!(o.role, OccurrenceRole::Declaration | OccurrenceRole::ResultWrite),
                    })
                    .collect(),
            });
        }
    }

    let report = final_bindings::bind_document(source, path);
    let Some(facts) = report.facts() else {
        return Err(NotBound::SourceNotAccepted);
    };
    let Some(name_use) = report.use_in_source(source, path, offset) else {
        // A callable name A1 could not resolve is a name, but not navigable.
        return if callables.definitions.iter().any(|d| {
            matches!(d.resolution, Resolution::Unsupported(_))
                && name_token_at(source, path, offset).is_some_and(|t| t.starts_with(d.name.as_str()))
        }) {
            Err(NotBound::CallableNotExact)
        } else {
            Err(NotBound::NotAName)
        };
    };
    let BindingState::Final(key) = name_use.state() else {
        return Err(NotBound::NotFinal);
    };
    let definition = facts.definition(*key).ok_or(NotBound::NotFinal)?;
    let kind = match definition.kind() {
        FactKind::Parameter => BoundKind::Parameter,
        FactKind::Local => BoundKind::Local,
        // Callables and result places are A1's; it did not resolve them exactly.
        FactKind::Callable | FactKind::ResultPlace => return Err(NotBound::CallableNotExact),
    };
    let tokens = analyze_name_tokens(source, path);
    let spans = report
        .exact_local_occurrences(source, path, *key, &tokens)
        .map_err(|_| NotBound::LocalNotComplete)?;
    let declaration = definition.name_span().ok_or(NotBound::LocalNotComplete)?;
    let declaration = ByteRange { start: declaration.start, end: declaration.end };
    Ok(BoundName {
        identity: format!("{}@{}:{}", if kind == BoundKind::Parameter { "parameter" } else { "local" }, declaration.start, declaration.end),
        name: definition.name().to_string(),
        kind,
        declaration,
        occurrences: spans
            .iter()
            .map(|s| {
                let range = ByteRange { start: s.start, end: s.end };
                BoundOccurrence { range, write: range == declaration }
            })
            .collect(),
    })
}

fn name_token_at(source: &str, path: &str, offset: usize) -> Option<String> {
    analyze_name_tokens(source, path)
        .token_at(offset)
        .map(|t| t.spelling.clone())
}

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

/// The exact bound name at a position of an open `.ddn` snapshot.
pub fn bound_name_in_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<Result<BoundName, NotBound>, DefinitionError> {
    if snapshot.language_id != "ddn" {
        return Ok(Err(NotBound::NotAName));
    }
    let source = snapshot.text().map_err(|_| DefinitionError::InvalidUtf8)?;
    let offset = position_to_byte_offset(source, position, encoding)
        .map_err(DefinitionError::InvalidPosition)?;
    Ok(bound_name_at(source, &snapshot.uri, offset))
}

pub fn definition_snapshot(
    snapshot: &DocumentSnapshot,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<Option<DefinitionResult>, DefinitionError> {
    let Ok(bound) = bound_name_in_snapshot(snapshot, position, encoding)? else {
        return Ok(None);
    };
    Ok(Some(DefinitionResult {
        document_version: snapshot.version,
        snapshot_identity: snapshot.identity.as_str().to_string(),
        position,
        identity: bound.identity,
        range: bound.declaration,
        source_profile: SOURCE_PROFILE_V25,
    }))
}

// ---------------------------------------------------------------------------
// Project documents.
//
// A document belongs to a project only when an open `.ddn` document is a work
// root (`… 얽음.`) whose declared source set contains it. There is no parent
// folder search and no file-name convention. The project is read from disk,
// every open document of it replaces its disk text, and the common
// `bind_project` binds that exact text. A member document never falls back to
// the single-document path: a failed project binding gives no result.
// ---------------------------------------------------------------------------

use crate::local_project::{build_virtual_project_source_from_disk, VirtualProjectSource};
use crate::name_binding::final_bindings::{
    bind_project, DefinitionOwnerScope, ProjectBindingIssue, ProjectBindingReport, ProjectBindingTarget,
    ProjectNameSite,
};

/// A name site in a project file, with that file's text for position conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteLocation {
    pub uri: String,
    pub path: String,
    pub source: String,
    pub range: ByteRange,
}

pub enum ProjectLookup {
    /// No open work root declares this document: the single-document rules apply.
    NotMember,
    /// More than one open work root declares it.
    Ambiguous,
    Member(Box<ProjectView>),
}

/// One request's view of the project containing the requested document.
pub struct ProjectView {
    raw: String,
    project_uri: String,
    generation: u64,
    root_dir_uri: String,
    open_uris: std::collections::BTreeMap<String, String>,
    source: VirtualProjectSource,
    pub document_path: String,
    pub binding: Result<ProjectBindingReport, ProjectBindingIssue>,
}

/// `open` is every open document as (uri, language id, text).
pub fn project_lookup(snapshot: &DocumentSnapshot, open: &[(&str, &str, &str)]) -> ProjectLookup {
    if snapshot.language_id != "ddn" {
        return ProjectLookup::NotMember;
    }
    let Some(document_file) = file_uri_to_path(&snapshot.uri) else {
        return ProjectLookup::NotMember;
    };
    let mut found = Vec::new();
    for (root_uri, language_id, root_text) in open {
        if *language_id != "ddn" || ddonirang_lang::work_root::parse_work_root(root_text).is_err() {
            continue;
        }
        let Some(root_file) = file_uri_to_path(root_uri) else { continue };
        let Some((root_dir, _)) = root_file.rsplit_once('/') else { continue };
        let Some(document_path) = document_file
            .get(..root_dir.len())
            .filter(|prefix| same_path(prefix, root_dir))
            .and_then(|_| document_file.get(root_dir.len()..))
            .and_then(|rest| rest.strip_prefix('/'))
        else {
            continue;
        };
        // A root that cannot be read from disk declares nothing we can check.
        let Ok(mut source) = build_virtual_project_source_from_disk(std::path::Path::new(&root_file), None) else {
            continue;
        };
        if !source.files.iter().any(|f| f.role == "ddn_source" && same_path(&f.path, document_path)) {
            continue;
        }
        let root_dir_uri = match root_uri.rsplit_once('/') {
            Some((prefix, _)) => format!("{prefix}/"),
            None => continue,
        };
        let mut open_uris = std::collections::BTreeMap::new();
        for file in source.files.iter_mut() {
            let wanted = format!("{root_dir}/{}", file.path);
            let mut matching = open.iter().filter(|(uri, _, _)| {
                file_uri_to_path(uri).is_some_and(|p| same_path(&p, &wanted))
            });
            if let Some((uri, _, text)) = matching.next() {
                // URI spelling is not a document revision. Two open buffers
                // for this project file cannot choose one another's text or
                // issue edits under the requested buffer's version, even if
                // their current text happens to match. Keep both buffers;
                // refuse this project view until the ambiguity is removed.
                if matching.next().is_some() {
                    return ProjectLookup::Ambiguous;
                }
                file.utf8_text = (*text).to_string();
                open_uris.insert(file.path.clone(), (*uri).to_string());
            }
        }
        let document_path = source
            .files
            .iter()
            .find(|f| same_path(&f.path, document_path))
            .map(|f| f.path.clone())
            .unwrap_or_else(|| document_path.to_string());
        found.push((root_uri.to_string(), root_dir_uri, open_uris, source, document_path));
    }
    if found.len() > 1 {
        return ProjectLookup::Ambiguous;
    }
    let Some((project_uri, root_dir_uri, open_uris, source, document_path)) = found.pop() else {
        return ProjectLookup::NotMember;
    };
    let generation = snapshot.version.max(0) as u64;
    let raw = serde_json::to_string(&source).unwrap_or_default();
    let binding = bind_project(&raw, &project_uri, generation);
    ProjectLookup::Member(Box::new(ProjectView {
        raw,
        project_uri,
        generation,
        root_dir_uri,
        open_uris,
        source,
        document_path,
        binding,
    }))
}

impl ProjectView {
    fn report(&self) -> Result<&ProjectBindingReport, ProjectBindingIssue> {
        self.binding.as_ref().map_err(|issue| *issue)
    }

    /// The common binding at a byte offset of the requested document; the
    /// cursor may sit just after a name.
    pub fn target_at(&self, offset: usize) -> Result<ProjectBindingTarget, ProjectBindingIssue> {
        let report = self.report()?;
        let text = self.text(&self.document_path).ok_or(ProjectBindingIssue::SourceRejected)?;
        let mut last = ProjectBindingIssue::NonNamePosition;
        for probe in probe_offsets(text, offset) {
            match report.binding_at(&self.raw, &self.project_uri, self.generation, &self.document_path, probe) {
                Ok(target) => return Ok(target),
                Err(ProjectBindingIssue::NonNamePosition) => {}
                Err(other) => last = other,
            }
        }
        Err(last)
    }

    /// The complete occurrence census of a parameter or local. Callables and
    /// result places are refused: the producer census covers name tokens
    /// only, not `…하기` call stems.
    pub fn local_sites(&self, target: &ProjectBindingTarget) -> Result<Vec<ProjectNameSite>, ProjectBindingIssue> {
        if !matches!(target.kind(), FactKind::Parameter | FactKind::Local) {
            return Err(ProjectBindingIssue::IncompleteOccurrences);
        }
        self.report()?
            .exact_occurrences(&self.raw, &self.project_uri, self.generation, target)
    }

    pub fn location(&self, site: &ProjectNameSite) -> Option<SiteLocation> {
        let source = self.text(site.path())?.to_string();
        let range = ByteRange { start: site.span().start, end: site.span().end };
        if source.get(range.start..range.end) != Some(site.spelling()) {
            return None;
        }
        Some(SiteLocation { uri: self.uri_for(site.path()), path: site.path().to_string(), source, range })
    }

    pub fn identity(site: &ProjectNameSite) -> String {
        format!("project:{}@{}:{}", site.path(), site.span().start, site.span().end)
    }

    pub fn text(&self, path: &str) -> Option<&str> {
        self.source.files.iter().find(|f| f.path == path).map(|f| f.utf8_text.as_str())
    }

    /// Every `.ddn` source of the project, as (path, text).
    pub fn sources(&self) -> impl Iterator<Item = (&str, &str)> {
        self.source
            .files
            .iter()
            .filter(|f| f.role == "ddn_source")
            .map(|f| (f.path.as_str(), f.utf8_text.as_str()))
    }

    /// Whether a project path lies inside a locked package (a folder holding
    /// a `gaji_manifest`): such files are not edited from the editor.
    pub fn is_dependency(&self, path: &str) -> bool {
        self.source.files.iter().filter(|f| f.role == "gaji_manifest").any(|manifest| {
            match manifest.path.rsplit_once('/') {
                Some((dir, _)) => path.starts_with(&format!("{dir}/")),
                None => true,
            }
        })
    }

    pub fn is_setup_owner(target: &ProjectBindingTarget) -> bool {
        target.owner_scope() == DefinitionOwnerScope::RelocatedTopLevelSetup
    }

    /// The same project with the requested document's text replaced, bound
    /// again by the common producer (the generation moves on).
    pub fn with_document_text(&self, text: &str) -> ProjectView {
        let mut source = self.source.clone();
        if let Some(file) = source.files.iter_mut().find(|f| f.path == self.document_path) {
            file.utf8_text = text.to_string();
        }
        let raw = serde_json::to_string(&source).unwrap_or_default();
        let generation = self.generation + 1;
        let binding = bind_project(&raw, &self.project_uri, generation);
        ProjectView {
            raw,
            project_uri: self.project_uri.clone(),
            generation,
            root_dir_uri: self.root_dir_uri.clone(),
            open_uris: self.open_uris.clone(),
            source,
            document_path: self.document_path.clone(),
            binding,
        }
    }

    /// Every name token of every source that the common producer binds, as
    /// (path, use start) -> (definition path, definition start).
    pub fn binding_map(&self) -> Option<std::collections::BTreeMap<(String, usize), (String, usize)>> {
        let report = self.report().ok()?;
        let mut map = std::collections::BTreeMap::new();
        for (path, text) in self.sources() {
            let tokens = ddonirang_lang::Lexer::new(text).tokenize().ok()?;
            for token in tokens.iter().filter(|t| matches!(t.kind, ddonirang_lang::TokenKind::Ident(_))) {
                if let Ok(target) =
                    report.binding_at(&self.raw, &self.project_uri, self.generation, path, token.span.start)
                {
                    map.insert(
                        (path.to_string(), token.span.start),
                        (target.definition().path().to_string(), target.definition().span().start),
                    );
                }
            }
        }
        Some(map)
    }

    fn uri_for(&self, path: &str) -> String {
        if let Some(uri) = self.open_uris.get(path) {
            return uri.clone();
        }
        format!("{}{}", self.root_dir_uri, encode_uri_path(path))
    }
}

fn same_path(left: &str, right: &str) -> bool {
    if cfg!(windows) {
        left.eq_ignore_ascii_case(right)
    } else {
        left == right
    }
}

/// `file:` URI to a path with `/` separators (`c:/x/y.ddn` or `/x/y.ddn`).
fn file_uri_to_path(uri: &str) -> Option<String> {
    let rest = uri.strip_prefix("file://")?;
    let rest = rest.split(['?', '#']).next()?;
    let mut bytes = Vec::with_capacity(rest.len());
    let raw = rest.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        if raw[index] == b'%' {
            let hex = std::str::from_utf8(raw.get(index + 1..index + 3)?).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            bytes.push(raw[index]);
            index += 1;
        }
    }
    let path = String::from_utf8(bytes).ok()?;
    // `file:///c:/x` -> `c:/x`; a UNC authority is not supported.
    let path = path.strip_prefix('/')?;
    if path.len() >= 2 && path.as_bytes()[1] == b':' {
        Some(path.replace('\\', "/"))
    } else if cfg!(windows) {
        None
    } else {
        Some(format!("/{path}"))
    }
}

fn encode_uri_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
