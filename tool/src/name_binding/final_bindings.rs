//! Read-only editor projection of the common language owner's lexical facts.
//! This module has no independent name resolver and issues no edit permission.
use crate::ddn_runtime::{resolve_supported_source_profile, DdnProgram};
use crate::source_analysis::{analyze_source, SourceAnalysisRequest};
pub use ddonirang_lang::name_resolution::{
    BindingState, DefinitionKey, DefinitionKind, DefinitionOwnerScope, Issue, NameDefinition, NameResolutionFacts,
    NameUse,
};
use ddonirang_lang::ParseMode;
/// A complete occurrence census is still not permission to choose a new name.
/// The LSP consumer keeps its capture/collision/current-version guards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalCoverageIssue {
    StaleSource,
    Unavailable,
    ForeignDefinition,
    NotLocal,
    MissingDeclaration,
    UnresolvedOccurrence,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalBindingOutcome {
    Available,
    SourceRejected,
    CommonFactsUnavailable,
}
#[derive(Debug, Clone)]
pub struct FinalBindingReport {
    outcome: FinalBindingOutcome,
    facts: Option<NameResolutionFacts>,
}
impl FinalBindingReport {
    /// Reuse the existing original-lexer token supply to refuse holes (e.g.
    /// a captured use or a named pin whose selected callable is not supplied).
    /// Another binding's same-spelling identifier is deliberately left alone.
    pub fn exact_local_occurrences(
        &self,
        source: &str,
        path: &str,
        key: DefinitionKey,
        tokens: &super::token_ranges::NameTokenReport,
    ) -> Result<Vec<ddonirang_lang::Span>, LocalCoverageIssue> {
        let facts = self.facts.as_ref().ok_or(LocalCoverageIssue::Unavailable)?;
        if !facts.is_for_source(source, path) || !tokens.is_for_source(source) {
            return Err(LocalCoverageIssue::StaleSource);
        }
        if tokens.outcome() != &super::token_ranges::NameTokenOutcome::Available {
            return Err(LocalCoverageIssue::Unavailable);
        }
        let definition = facts
            .definition(key)
            .ok_or(LocalCoverageIssue::ForeignDefinition)?;
        if !matches!(
            definition.kind(),
            DefinitionKind::Parameter | DefinitionKind::Local
        ) {
            return Err(LocalCoverageIssue::NotLocal);
        }
        if definition.owner_scope() != DefinitionOwnerScope::Lexical {
            return Err(LocalCoverageIssue::NotLocal);
        }
        let declaration = definition
            .name_span()
            .ok_or(LocalCoverageIssue::MissingDeclaration)?;
        let mut spans = Vec::new();
        for token in tokens
            .tokens()
            .iter()
            .filter(|t| t.spelling == definition.name())
        {
            let occurrence = facts
                .use_at(token.range.start)
                .ok_or(LocalCoverageIssue::UnresolvedOccurrence)?;
            match occurrence.state() {
                BindingState::Final(other) if *other == key => {
                    let span = occurrence.name_span();
                    if span.start != token.range.start || span.end != token.range.end {
                        return Err(LocalCoverageIssue::UnresolvedOccurrence);
                    }
                    spans.push(span);
                }
                BindingState::Final(_) => {}
                _ => return Err(LocalCoverageIssue::UnresolvedOccurrence),
            }
        }
        if !spans.contains(&declaration) {
            return Err(LocalCoverageIssue::MissingDeclaration);
        }
        spans.sort_by_key(|s| (s.start, s.end));
        spans.dedup();
        Ok(spans)
    }
    pub fn outcome(&self) -> FinalBindingOutcome {
        self.outcome
    }
    pub fn facts(&self) -> Option<&NameResolutionFacts> {
        self.facts.as_ref()
    }
    /// A stale source/document cannot supply even an otherwise final result.
    pub fn use_in_source(&self, source: &str, path: &str, byte_offset: usize) -> Option<&NameUse> {
        let facts = self.facts.as_ref()?;
        if !facts.is_for_source(source, path) {
            return None;
        }
        facts.use_at(byte_offset)
    }
}
pub fn bind_document(source: &str, path: &str) -> FinalBindingReport {
    let mut report = FinalBindingReport {
        outcome: FinalBindingOutcome::SourceRejected,
        facts: None,
    };
    let checked = analyze_source(SourceAnalysisRequest {
        source_text: source.into(),
        path: path.into(),
        analysis_generation: 0,
        source_profile_identity: "v1-core-v25".into(),
    });
    if checked.outcome != "ACCEPTED" {
        return report;
    }
    report.outcome = FinalBindingOutcome::CommonFactsUnavailable;
    let Ok(profile) = resolve_supported_source_profile(Some("v1-core-v25")) else {
        return report;
    };
    let Ok(program) = DdnProgram::from_supported_source_profile_with_mode(
        source,
        path,
        ParseMode::Strict,
        profile,
    ) else {
        return report;
    };
    let Some(facts) = program.canonical_program().origin.name_resolution() else {
        return report;
    };
    if !facts.is_for_source(source, path) {
        return report;
    }
    report.facts = Some(facts.clone());
    report.outcome = FinalBindingOutcome::Available;
    report
}

/// A name's original source location, projected by the actual common linker
/// and lexer. No saved identity, access right or replacement text is issued.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectNameSite {
    path: String,
    span: ddonirang_lang::Span,
    spelling: String,
}
impl ProjectNameSite {
    pub fn path(&self) -> &str { &self.path }
    pub fn span(&self) -> ddonirang_lang::Span { self.span }
    pub fn spelling(&self) -> &str { &self.spelling }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectBindingIssue {
    SourceRejected,
    CommonFactsUnavailable,
    StaleProject,
    NonNamePosition,
    NotFinal,
    OriginalNameUnavailable,
    ImportNotAdmitted,
    IncompleteOccurrences,
}

/// Private construction ties the existing Final key to one exact editor
/// snapshot. URI/generation are caller context selectors, never permissions.
#[derive(Debug, Clone)]
pub struct ProjectBindingTarget {
    context: [u8; 32],
    key: DefinitionKey,
    kind: DefinitionKind,
    owner_scope: DefinitionOwnerScope,
    definition: ProjectNameSite,
    usage: ProjectNameSite,
}
impl ProjectBindingTarget {
    pub fn definition(&self) -> &ProjectNameSite { &self.definition }
    pub fn usage(&self) -> &ProjectNameSite { &self.usage }
    pub fn kind(&self) -> DefinitionKind { self.kind }
    pub fn owner_scope(&self) -> DefinitionOwnerScope { self.owner_scope }
}

#[derive(Debug)]
struct ProjectSourceTokens {
    source: String,
    tokens: Vec<ddonirang_lang::Token>,
}

#[derive(Debug)]
pub struct ProjectBindingReport {
    context: [u8; 32],
    prepared: crate::local_project::AuthoredPreparation,
    files: std::collections::BTreeMap<String, ProjectSourceTokens>,
    bindings: FinalBindingReport,
}

fn project_context(source: &str, uri: &str, generation: u64) -> [u8; 32] {
    let mut hash = blake3::Hasher::new();
    hash.update(&(uri.len() as u64).to_be_bytes());
    hash.update(uri.as_bytes());
    hash.update(&generation.to_be_bytes());
    hash.update(&(source.len() as u64).to_be_bytes());
    hash.update(source.as_bytes());
    *hash.finalize().as_bytes()
}

/// Reuses the checked graph/linker with symbol-value execution disabled, then
/// consumes the SAME linked source's common lexical facts. There is no second
/// resolver. A missing Final, module permission or precise token stays missing.
pub fn bind_project(
    source: &str, project_uri: &str, generation: u64,
) -> Result<ProjectBindingReport, ProjectBindingIssue> {
    if source.len() > 67_108_864 { return Err(ProjectBindingIssue::SourceRejected); }
    let input: crate::local_project::VirtualProjectSource = serde_json::from_str(source)
        .map_err(|_| ProjectBindingIssue::SourceRejected)?;
    let prepared = crate::local_project::prepare_authored_execution(input.clone())
        .map_err(|_| ProjectBindingIssue::SourceRejected)?;
    let bindings = bind_document(&prepared.source, &prepared.target_path);
    if bindings.outcome() != FinalBindingOutcome::Available {
        return Err(ProjectBindingIssue::CommonFactsUnavailable);
    }
    let mut files = std::collections::BTreeMap::new();
    for file in input.files.into_iter().filter(|f| f.role == "ddn_source") {
        let tokens = ddonirang_lang::Lexer::new(&file.utf8_text).tokenize()
            .map_err(|_| ProjectBindingIssue::OriginalNameUnavailable)?;
        if files.insert(file.path, ProjectSourceTokens { source: file.utf8_text, tokens }).is_some() {
            return Err(ProjectBindingIssue::SourceRejected);
        }
    }
    Ok(ProjectBindingReport {
        context: project_context(source, project_uri, generation), prepared, files, bindings,
    })
}

impl ProjectBindingReport {
    pub fn is_for_project(&self, source: &str, uri: &str, generation: u64) -> bool {
        self.context == project_context(source, uri, generation)
    }

    fn original_name(&self, span: ddonirang_lang::Span, spelling: &str)
        -> Result<ProjectNameSite, ProjectBindingIssue>
    {
        use ddonirang_lang::TokenKind;
        let (path, coarse) = self.prepared.original_span(span)
            .map_err(|_| ProjectBindingIssue::OriginalNameUnavailable)?;
        let file = self.files.get(&path).ok_or(ProjectBindingIssue::OriginalNameUnavailable)?;
        let tokens = file.tokens.iter().filter(|t|
            t.span.start >= coarse.start && t.span.end <= coarse.end
                && !matches!(t.kind, TokenKind::Eof)).collect::<Vec<_>>();
        // The linker can map one generated identifier to alias.name. Only
        // actual complete shared-lexer tokens may refine that mapped range.
        let token = match tokens.as_slice() {
            [name] if matches!(name.kind, TokenKind::Ident(_)) => *name,
            [alias, dot, name] if matches!(alias.kind, TokenKind::Ident(_))
                && matches!(dot.kind, TokenKind::Dot)
                && matches!(name.kind, TokenKind::Ident(_)) => *name,
            _ => return Err(ProjectBindingIssue::OriginalNameUnavailable),
        };
        if tokens.first().map(|t| t.span.start) != Some(coarse.start)
            || tokens.last().map(|t| t.span.end) != Some(coarse.end)
            || token.raw != spelling
            || file.source.get(token.span.start..token.span.end) != Some(spelling) {
            return Err(ProjectBindingIssue::OriginalNameUnavailable);
        }
        Ok(ProjectNameSite { path, span: ddonirang_lang::Span::new(token.span.start, token.span.end),
            spelling: token.raw.clone() })
    }

    fn target(&self, usage: &NameUse) -> Result<ProjectBindingTarget, ProjectBindingIssue> {
        let BindingState::Final(key) = usage.state() else { return Err(ProjectBindingIssue::NotFinal); };
        let facts = self.bindings.facts().ok_or(ProjectBindingIssue::CommonFactsUnavailable)?;
        let definition = facts.definition(*key).ok_or(ProjectBindingIssue::NotFinal)?;
        let kind = definition.kind();
        let owner_scope = definition.owner_scope();
        let definition = self.original_name(
            definition.name_span().ok_or(ProjectBindingIssue::OriginalNameUnavailable)?, definition.name())?;
        let site = self.original_name(usage.name_span(), usage.spelling())?;
        if site.path != definition.path
            && !self.prepared.permits_import(&site.path, &definition.path, definition.spelling()) {
            return Err(ProjectBindingIssue::ImportNotAdmitted);
        }
        Ok(ProjectBindingTarget { context: self.context, key: *key,
            kind, owner_scope, definition, usage: site })
    }

    pub fn binding_at(&self, source: &str, uri: &str, generation: u64,
        path: &str, byte_offset: usize) -> Result<ProjectBindingTarget, ProjectBindingIssue>
    {
        if !self.is_for_project(source, uri, generation) { return Err(ProjectBindingIssue::StaleProject); }
        let facts = self.bindings.facts().ok_or(ProjectBindingIssue::CommonFactsUnavailable)?;
        for usage in facts.uses() {
            let Ok(site) = self.original_name(usage.name_span(), usage.spelling()) else { continue; };
            if site.path == path && site.span.start <= byte_offset && byte_offset < site.span.end
                && site.spelling.is_char_boundary(byte_offset - site.span.start) {
                return self.target(usage);
            }
        }
        Err(ProjectBindingIssue::NonNamePosition)
    }

    /// Complete EXACT token census only. Export aliases, named pins or other
    /// owner syntax not yet supplied by common facts make this fail closed.
    /// Even a complete census is not a workspace/file write permission.
    pub fn exact_occurrences(&self, source: &str, uri: &str, generation: u64,
        target: &ProjectBindingTarget) -> Result<Vec<ProjectNameSite>, ProjectBindingIssue>
    {
        if !self.is_for_project(source, uri, generation) || target.context != self.context {
            return Err(ProjectBindingIssue::StaleProject);
        }
        let facts = self.bindings.facts().ok_or(ProjectBindingIssue::CommonFactsUnavailable)?;
        let mut known = std::collections::BTreeSet::new();
        let mut sites = Vec::new();
        for usage in facts.uses().iter().filter(|u| u.spelling() == target.definition.spelling()) {
            let checked = self.target(usage).map_err(|_| ProjectBindingIssue::IncompleteOccurrences)?;
            known.insert((checked.usage.path.clone(), checked.usage.span.start, checked.usage.span.end));
            if checked.key == target.key { sites.push(checked.usage); }
        }
        for (path, file) in &self.files {
            for token in file.tokens.iter().filter(|t|
                matches!(t.kind, ddonirang_lang::TokenKind::Ident(_)) && t.raw == target.definition.spelling()) {
                if !known.contains(&(path.clone(), token.span.start, token.span.end)) {
                    return Err(ProjectBindingIssue::IncompleteOccurrences);
                }
            }
        }
        sites.sort_by(|a,b| (&a.path,a.span.start,a.span.end).cmp(&(&b.path,b.span.start,b.span.end)));
        sites.dedup();
        if !sites.contains(&target.definition) { return Err(ProjectBindingIssue::IncompleteOccurrences); }
        Ok(sites)
    }
}
