//! Rename for one open document, driven only by the common binding producers
//! (see `definition::bound_name_at`): A1 `Exact` top-level callables and
//! `Final` parameters/locals with a complete name-token set.
//!
//! Anything not bound exactly, a new spelling that already occurs, or an edit
//! that would change any other binding is refused with the reason. There is no
//! spelling-based fallback.
use super::definition::{bound_name_at, BoundKind, BoundName, NotBound, ProjectView};
use crate::name_binding::final_bindings::{
    DefinitionKind as FactKind, ProjectBindingIssue, ProjectBindingTarget, ProjectNameSite,
};
use super::document_store::DocumentSnapshot;
use super::position::ByteRange;
use crate::name_binding::{
    bind_document, BindingOutcome, DefinitionKind, NameBindingReport, Resolution,
    UnsupportedReason,
};
use ddonirang_lang::term_map::{find_fatal_term, find_legacy_term, is_reserved_word};
use ddonirang_lang::{Lexer, TokenKind};

pub use super::symbol::SymbolError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameTarget {
    pub name: String,
    /// The occurrence under the cursor: exactly the name bytes.
    pub range: ByteRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePlan {
    pub name: String,
    pub new_name: String,
    /// Every occurrence's name bytes, in source order.
    pub ranges: Vec<ByteRange>,
}

/// User-facing reason; the server returns it as the request error message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameRefusal(pub String);

fn refuse(message: &str) -> RenameRefusal {
    RenameRefusal(format!("이름을 바꿀 수 없습니다: {message}"))
}

pub fn prepare_rename(
    snapshot: &DocumentSnapshot,
    offset: usize,
) -> Result<RenameTarget, RenameRefusal> {
    let (_, bound) = target(snapshot, offset)?;
    let range = bound
        .occurrences
        .iter()
        .map(|o| o.range)
        .find(|r| r.start <= offset && offset <= r.end)
        .unwrap_or(bound.declaration);
    Ok(RenameTarget { name: bound.name, range })
}

pub fn plan_rename(
    snapshot: &DocumentSnapshot,
    offset: usize,
    new_name: &str,
    other_open_sources: &[&str],
) -> Result<RenamePlan, RenameRefusal> {
    let (source, bound) = target(snapshot, offset)?;
    validate_new_name(source, &bound.name, new_name)?;
    // A callable name or a parameter (a public pin) may be used by another
    // file; multi-file binding is not supplied, so refuse rather than guess.
    if bound.kind != BoundKind::Local
        && other_open_sources
            .iter()
            .any(|other| identifier_tokens(other).iter().any(|raw| raw.starts_with(bound.name.as_str())))
    {
        return Err(refuse(
            "다른 열린 문서에도 같은 철자의 이름이 있습니다. 여러 파일에 걸친 결속은 아직 확인하지 못합니다.",
        ));
    }
    let ranges: Vec<ByteRange> = bound.occurrences.iter().map(|o| o.range).collect();
    let renamed = apply(source, &ranges, &bound.name, new_name);
    verify_after(source, &snapshot.uri, &bound, &renamed, new_name)?;
    Ok(RenamePlan {
        name: bound.name.clone(),
        new_name: new_name.to_string(),
        ranges,
    })
}

fn target(snapshot: &DocumentSnapshot, offset: usize) -> Result<(&str, BoundName), RenameRefusal> {
    if snapshot.language_id != "ddn" {
        return Err(refuse("또니랑 원문(.ddn) 문서가 아닙니다."));
    }
    let source = snapshot.text().map_err(|_| refuse("문서가 UTF-8이 아닙니다."))?;
    match bound_name_at(source, &snapshot.uri, offset) {
        Ok(bound) => Ok((source, bound)),
        Err(NotBound::SourceNotAccepted) => {
            Err(refuse("검사를 통과한 원문에서만 바꿀 수 있습니다. 먼저 오류를 고쳐 주세요."))
        }
        Err(NotBound::CallableNotExact) => Err(refuse(callable_reason(source, &snapshot.uri, offset))),
        Err(NotBound::NotFinal) => Err(refuse("이 이름이 가리키는 정의가 하나로 확정되지 않았습니다.")),
        Err(NotBound::LocalNotComplete) => Err(refuse("이 이름의 모든 사용을 확정하지 못했습니다.")),
        Err(NotBound::NotAName) => Err(refuse("이 위치에는 바꿀 수 있는 이름이 없습니다.")),
    }
}

/// The reason A1 gave for the callable under the cursor.
fn callable_reason(source: &str, path: &str, offset: usize) -> &'static str {
    let report = bind_document(source, path);
    let word = identifier_at(source, offset);
    let reason = report
        .definitions
        .iter()
        .filter(|d| d.kind == DefinitionKind::TopLevelCallable)
        .filter(|d| word.as_deref().is_some_and(|w| w.starts_with(d.name.as_str())))
        .find_map(|d| match &d.resolution {
            Resolution::Unsupported(sites) => sites.first().map(|s| s.reason.clone()),
            Resolution::Exact(_) => None,
        });
    match reason {
        Some(UnsupportedReason::SameSpellingLocalBinding | UnsupportedReason::SameSpellingParameter) => {
            "같은 철자의 지역 이름이나 매개변수가 있어 어느 쪽인지 확정할 수 없습니다."
        }
        Some(UnsupportedReason::CallTailParsedAsVariable) => {
            "인자 없는 `…하기` 문장 호출이 있어 이 함수의 사용인지 확정할 수 없습니다."
        }
        Some(UnsupportedReason::StdlibSameName) => "표준 함수와 같은 이름입니다.",
        Some(UnsupportedReason::RegisteredSurfaces) => "등록된 호출 표면이 있는 문서는 아직 지원하지 않습니다.",
        Some(UnsupportedReason::ResultPlaceRead) => "함수 본문에서 결과 자리를 읽는 경우는 아직 지원하지 않습니다.",
        Some(UnsupportedReason::DuplicateDefinition) => "같은 이름의 함수가 둘 이상입니다.",
        _ => "모든 사용을 확정하지 못했습니다.",
    }
}

fn validate_new_name(source: &str, old_name: &str, new_name: &str) -> Result<(), RenameRefusal> {
    if new_name == old_name {
        return Err(refuse("새 이름이 지금 이름과 같습니다."));
    }
    let Ok(tokens) = Lexer::new(new_name).tokenize() else {
        return Err(refuse("새 이름이 이름 하나로 읽히지 않습니다."));
    };
    let single = tokens.len() == 2
        && matches!(&tokens[0].kind, TokenKind::Ident(_))
        && tokens[0].raw == new_name;
    if !single {
        return Err(refuse("새 이름이 이름 하나로 읽히지 않습니다."));
    }
    if find_fatal_term(new_name).is_some() || find_legacy_term(new_name).is_some() || is_reserved_word(new_name) {
        return Err(refuse("새 이름이 금지어·옛말·예약어입니다."));
    }
    // A spelling that already occurs could capture or be captured by another
    // binding (locals, parameters, call tails).
    if identifier_tokens(source).iter().any(|raw| raw.starts_with(new_name)) {
        return Err(refuse("새 이름(또는 그 앞부분)이 문서에 이미 있습니다."));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Project member documents (see `definition::project_lookup`). Only a
// parameter or local whose complete census the common `bind_project` supplies,
// all in the requested document, outside a locked package. The edit must keep
// every other binding of the project exactly as it was.
// ---------------------------------------------------------------------------

fn project_refusal(issue: ProjectBindingIssue) -> RenameRefusal {
    refuse(match issue {
        ProjectBindingIssue::SourceRejected => "검사를 통과한 작품에서만 바꿀 수 있습니다. 먼저 오류를 고쳐 주세요.",
        ProjectBindingIssue::CommonFactsUnavailable => "작품의 이름 결속을 얻지 못했습니다.",
        ProjectBindingIssue::StaleProject => "작품 판이 바뀌었습니다. 다시 시도해 주세요.",
        ProjectBindingIssue::NonNamePosition => "이 위치에는 바꿀 수 있는 이름이 없습니다.",
        ProjectBindingIssue::NotFinal => "이 이름이 가리키는 정의가 하나로 확정되지 않았습니다.",
        ProjectBindingIssue::OriginalNameUnavailable => "원문에서 이 이름의 정확한 위치를 확정하지 못했습니다.",
        ProjectBindingIssue::ImportNotAdmitted => "가져오기가 허가되지 않은 이름입니다.",
        ProjectBindingIssue::IncompleteOccurrences => "이 이름의 모든 사용을 확정하지 못했습니다.",
    })
}

fn project_target(
    view: &ProjectView,
    offset: usize,
) -> Result<(ProjectBindingTarget, Vec<ProjectNameSite>), RenameRefusal> {
    if view.is_dependency(&view.document_path) {
        return Err(refuse("잠긴 꾸러미(의존) 파일은 편집기에서 바꾸지 않습니다."));
    }
    let target = view.target_at(offset).map_err(project_refusal)?;
    if !matches!(target.kind(), FactKind::Parameter | FactKind::Local) {
        return Err(refuse("작품 안에서는 매개변수와 지역 이름만 바꿀 수 있습니다."));
    }
    let sites = view.local_sites(&target).map_err(project_refusal)?;
    if sites.iter().any(|site| site.path() != view.document_path) {
        return Err(refuse("다른 파일에도 사용이 있어 이 문서에서만 바꿀 수 없습니다."));
    }
    Ok((target, sites))
}

pub fn prepare_project_rename(view: &ProjectView, offset: usize) -> Result<RenameTarget, RenameRefusal> {
    let (target, sites) = project_target(view, offset)?;
    let range = sites
        .iter()
        .map(|site| ByteRange { start: site.span().start, end: site.span().end })
        .find(|r| r.start <= offset && offset <= r.end)
        .unwrap_or(ByteRange { start: target.usage().span().start, end: target.usage().span().end });
    Ok(RenameTarget { name: target.definition().spelling().to_string(), range })
}

pub fn plan_project_rename(view: &ProjectView, offset: usize, new_name: &str) -> Result<RenamePlan, RenameRefusal> {
    let (target, sites) = project_target(view, offset)?;
    let name = target.definition().spelling().to_string();
    let source = view.text(&view.document_path).ok_or_else(|| refuse("문서를 찾지 못했습니다."))?;
    validate_new_name(source, &name, new_name)?;
    if view
        .sources()
        .any(|(_, text)| identifier_tokens(text).iter().any(|raw| raw.starts_with(new_name)))
    {
        return Err(refuse("새 이름(또는 그 앞부분)이 작품의 다른 파일에 이미 있습니다."));
    }
    let ranges: Vec<ByteRange> =
        sites.iter().map(|site| ByteRange { start: site.span().start, end: site.span().end }).collect();
    let renamed = apply(source, &ranges, &name, new_name);
    let after = view.with_document_text(&renamed);
    if after.binding.is_err() {
        return Err(refuse("바꾼 작품이 검사를 통과하지 못합니다."));
    }
    let shift = new_name.len() as isize - name.len() as isize;
    let moved = |path: &str, at: usize| -> usize {
        if path != view.document_path {
            return at;
        }
        let before = ranges.iter().filter(|r| r.start < at).count() as isize;
        (at as isize + before * shift) as usize
    };
    let (Some(before), Some(now)) = (view.binding_map(), after.binding_map()) else {
        return Err(refuse("바꾸기 전후의 결속을 확인하지 못했습니다."));
    };
    let expected: std::collections::BTreeMap<_, _> = before
        .into_iter()
        .map(|((path, at), (def_path, def_at))| {
            let use_at = moved(&path, at);
            let def_at = moved(&def_path, def_at);
            ((path, use_at), (def_path, def_at))
        })
        .collect();
    if expected != now {
        return Err(refuse("바꾸면 다른 이름의 결속이 달라집니다."));
    }
    Ok(RenamePlan { name, new_name: new_name.to_string(), ranges })
}

/// The renamed text must stay accepted; the renamed name must bind exactly to
/// the same number of occurrences at its new declaration; and every other
/// top-level callable that was exact must stay exact with the same uses. (A
/// callable that was unsupported may become exact when a local stops sharing
/// its spelling; that is not a change of any existing binding.)
fn verify_after(
    source: &str,
    path: &str,
    bound: &BoundName,
    renamed: &str,
    new_name: &str,
) -> Result<(), RenameRefusal> {
    let before = bind_document(source, path);
    let after = bind_document(renamed, path);
    if after.outcome != BindingOutcome::Analyzed {
        return Err(refuse("바꾼 원문이 검사를 통과하지 못합니다."));
    }
    let shift = new_name.len() as isize - bound.name.len() as isize;
    let earlier = bound
        .occurrences
        .iter()
        .filter(|o| o.range.start < bound.declaration.start)
        .count() as isize;
    let new_declaration = (bound.declaration.start as isize + earlier * shift) as usize;
    let renamed_ok = matches!(
        bound_name_at(renamed, path, new_declaration),
        Ok(again) if again.kind == bound.kind
            && again.name == new_name
            && again.occurrences.len() == bound.occurrences.len()
    );
    let skip = if bound.kind == BoundKind::TopLevelCallable { (bound.name.as_str(), new_name) } else { ("", "") };
    let after_shape = shape(&after, skip.1);
    let kept = shape(&before, skip.0).into_iter().all(|(name, exact)| match exact {
        Some(count) => after_shape.iter().any(|(n, e)| *n == name && *e == Some(count)),
        None => true,
    });
    if !renamed_ok || !kept {
        return Err(refuse("바꾸면 다른 이름의 결속이 달라집니다."));
    }
    Ok(())
}

fn shape(report: &NameBindingReport, skip: &str) -> Vec<(String, Option<usize>)> {
    report
        .definitions
        .iter()
        .filter(|d| d.kind == DefinitionKind::TopLevelCallable && d.name != skip)
        .map(|d| {
            let exact = match &d.resolution {
                Resolution::Exact(o) => Some(o.len()),
                Resolution::Unsupported(_) => None,
            };
            (d.name.clone(), exact)
        })
        .collect()
}

fn apply(source: &str, ranges: &[ByteRange], old_name: &str, new_name: &str) -> String {
    let mut text = source.to_string();
    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|r| r.start);
    for range in sorted.iter().rev() {
        debug_assert_eq!(&source[range.start..range.end], old_name);
        text.replace_range(range.start..range.end, new_name);
    }
    text
}

fn identifier_tokens(source: &str) -> Vec<String> {
    Lexer::new(source)
        .tokenize()
        .map(|tokens| {
            tokens
                .into_iter()
                .filter(|t| matches!(t.kind, TokenKind::Ident(_)))
                .map(|t| t.raw)
                .collect()
        })
        .unwrap_or_default()
}

fn identifier_at(source: &str, offset: usize) -> Option<String> {
    let tokens = Lexer::new(source).tokenize().ok()?;
    tokens
        .into_iter()
        .find(|t| matches!(t.kind, TokenKind::Ident(_)) && t.span.start <= offset && offset <= t.span.end)
        .map(|t| t.raw)
}
