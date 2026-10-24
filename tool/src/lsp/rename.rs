//! Rename for one open document, driven only by the shared name binding A1.
//!
//! A name is renamed only when A1 resolves every occurrence of that top-level
//! callable exactly. Anything A1 marks unsupported, a new spelling that already
//! occurs, or an edit that would change any other binding is refused with the
//! reason. There is no spelling-based fallback.
use super::document_store::DocumentSnapshot;
use super::position::ByteRange;
use crate::name_binding::{
    bind_document, BindingOutcome, DefinitionKind, NameBindingReport, NameDefinition, Resolution,
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
    let report = analyzed(snapshot)?;
    target_at(&report, snapshot, offset).map(|(definition, range)| RenameTarget {
        name: definition.name.clone(),
        range,
    })
}

pub fn plan_rename(
    snapshot: &DocumentSnapshot,
    offset: usize,
    new_name: &str,
    other_open_sources: &[&str],
) -> Result<RenamePlan, RenameRefusal> {
    let report = analyzed(snapshot)?;
    let (definition, _) = target_at(&report, snapshot, offset)?;
    let Resolution::Exact(occurrences) = &definition.resolution else {
        return Err(refuse("모든 사용을 확정하지 못했습니다."));
    };
    let source = snapshot.text().map_err(|_| refuse("문서가 UTF-8이 아닙니다."))?;
    validate_new_name(source, &definition.name, new_name)?;
    if other_open_sources
        .iter()
        .any(|other| identifier_tokens(other).iter().any(|raw| raw.starts_with(definition.name.as_str())))
    {
        return Err(refuse(
            "다른 열린 문서에도 같은 철자의 이름이 있습니다. 여러 파일에 걸친 결속은 아직 확인하지 못합니다.",
        ));
    }

    let ranges: Vec<ByteRange> = occurrences
        .iter()
        .map(|o| ByteRange { start: o.name_range.start, end: o.name_range.end })
        .collect();
    let renamed = apply(source, &ranges, &definition.name, new_name);
    verify_after(&report, definition, occurrences.len(), &renamed, &snapshot.uri, new_name)?;
    Ok(RenamePlan {
        name: definition.name.clone(),
        new_name: new_name.to_string(),
        ranges,
    })
}

fn analyzed(snapshot: &DocumentSnapshot) -> Result<NameBindingReport, RenameRefusal> {
    if snapshot.language_id != "ddn" {
        return Err(refuse("또니랑 원문(.ddn) 문서가 아닙니다."));
    }
    let source = snapshot.text().map_err(|_| refuse("문서가 UTF-8이 아닙니다."))?;
    let report = bind_document(source, &snapshot.uri);
    if report.outcome != BindingOutcome::Analyzed {
        return Err(refuse("검사를 통과한 원문에서만 바꿀 수 있습니다. 먼저 오류를 고쳐 주세요."));
    }
    Ok(report)
}

/// The top-level callable whose name sits under the cursor. An unsupported
/// definition explains why instead of falling back to spelling.
fn target_at<'a>(
    report: &'a NameBindingReport,
    snapshot: &DocumentSnapshot,
    offset: usize,
) -> Result<(&'a NameDefinition, ByteRange), RenameRefusal> {
    if let Some((definition, occurrence)) = report.occurrence_at(offset) {
        if definition.kind == DefinitionKind::TopLevelCallable {
            return Ok((
                definition,
                ByteRange { start: occurrence.name_range.start, end: occurrence.name_range.end },
            ));
        }
    }
    let source = snapshot.text().map_err(|_| refuse("문서가 UTF-8이 아닙니다."))?;
    let word = identifier_at(source, offset);
    let unsupported = report.definitions.iter().find(|d| {
        d.kind == DefinitionKind::TopLevelCallable
            && word.as_deref().is_some_and(|w| w.starts_with(d.name.as_str()))
            && matches!(d.resolution, Resolution::Unsupported(_))
    });
    match unsupported.map(|d| &d.resolution) {
        Some(Resolution::Unsupported(sites)) => Err(refuse(reason_text(
            sites.first().map(|s| &s.reason),
        ))),
        _ => Err(refuse("지금은 최상위 함수 이름만 바꿀 수 있습니다.")),
    }
}

fn reason_text(reason: Option<&UnsupportedReason>) -> &'static str {
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
    // binding A1 does not enumerate (locals, parameters, call tails).
    if identifier_tokens(source).iter().any(|raw| raw.starts_with(new_name)) {
        return Err(refuse("새 이름(또는 그 앞부분)이 문서에 이미 있습니다."));
    }
    Ok(())
}

/// The renamed text must stay accepted, the renamed callable must resolve to
/// the same number of exact occurrences, and no other callable may change.
fn verify_after(
    before: &NameBindingReport,
    definition: &NameDefinition,
    count: usize,
    renamed: &str,
    path: &str,
    new_name: &str,
) -> Result<(), RenameRefusal> {
    let after = bind_document(renamed, path);
    if after.outcome != BindingOutcome::Analyzed {
        return Err(refuse("바꾼 원문이 검사를 통과하지 못합니다."));
    }
    let shape = |report: &NameBindingReport, skip: &str| -> Vec<(String, Option<usize>)> {
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
    };
    let renamed_ok = after.definitions.iter().any(|d| {
        d.kind == DefinitionKind::TopLevelCallable
            && d.name == new_name
            && matches!(&d.resolution, Resolution::Exact(o) if o.len() == count)
    });
    if !renamed_ok || shape(before, &definition.name) != shape(&after, new_name) {
        return Err(refuse("바꾸면 다른 이름의 결속이 달라집니다."));
    }
    Ok(())
}

fn apply(source: &str, ranges: &[ByteRange], old_name: &str, new_name: &str) -> String {
    let mut text = source.to_string();
    for range in ranges.iter().rev() {
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
