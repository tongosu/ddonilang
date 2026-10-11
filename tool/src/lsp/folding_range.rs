use super::document_store::DocumentSnapshot;
use super::position::{byte_range_to_lsp_range, ByteRange, LspRange, PositionEncoding};
use ddonirang_lang::{parse_v25, Body, Stmt, TopLevelItem};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldingRange {
    pub range: LspRange,
    pub kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoldingRangeError {
    InvalidUtf8,
    InvalidRange(String),
}

impl fmt::Display for FoldingRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 => f.write_str("folding range source가 UTF-8이 아닙니다"),
            Self::InvalidRange(error) => write!(f, "folding range가 잘못되었습니다: {error}"),
        }
    }
}

impl std::error::Error for FoldingRangeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FoldingKind {
    Definition,
    CallableBody,
    Compound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Candidate {
    range: ByteRange,
    kind: FoldingKind,
}

pub fn folding_ranges_snapshot(
    snapshot: &DocumentSnapshot,
    encoding: PositionEncoding,
) -> Result<Vec<FoldingRange>, FoldingRangeError> {
    let source = snapshot
        .text()
        .map_err(|_| FoldingRangeError::InvalidUtf8)?;
    if snapshot.language_id != "ddn" {
        return Ok(Vec::new());
    }
    let program = match parse_v25(source, &snapshot.uri) {
        Ok(program) => program,
        Err(_) => return Ok(Vec::new()),
    };
    let mut candidates = Vec::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        candidates.push(Candidate {
            range: ByteRange {
                start: seed.span.start,
                end: seed.span.end,
            },
            kind: FoldingKind::Definition,
        });
        if let Some(body) = &seed.body {
            if !body.stmts.is_empty() {
                candidates.push(Candidate {
                    range: ByteRange {
                        start: body.span.start,
                        end: body.span.end,
                    },
                    kind: FoldingKind::CallableBody,
                });
                collect_nested_candidates(body, &mut candidates);
            }
        }
    }
    let candidates = normalize_candidates(candidates, source, encoding)?;
    Ok(candidates
        .into_iter()
        .map(|candidate| FoldingRange {
            range: candidate.range,
            kind: match candidate.kind {
                FoldingKind::Definition | FoldingKind::CallableBody | FoldingKind::Compound => {
                    "region"
                }
            },
        })
        .collect())
}

fn collect_nested_candidates(body: &Body, candidates: &mut Vec<Candidate>) {
    for statement in &body.stmts {
        match statement {
            Stmt::Receive { body, .. }
            | Stmt::Repeat { body, .. }
            | Stmt::BeatBlock { body, .. }
            | Stmt::Hook { body, .. } => {
                push_compound(statement, std::slice::from_ref(&body), candidates);
                collect_nested_candidates(body, candidates);
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                let mut bodies = vec![then_body];
                if let Some(else_body) = else_body {
                    bodies.push(else_body);
                }
                push_compound(statement, &bodies, candidates);
                for nested in bodies {
                    collect_nested_candidates(nested, candidates);
                }
            }
            Stmt::Try { body, .. }
            | Stmt::While { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::Guard { body, .. }
            | Stmt::HookWhenBecomes { body, .. }
            | Stmt::HookWhile { body, .. } => {
                push_compound(statement, std::slice::from_ref(&body), candidates);
                collect_nested_candidates(body, candidates);
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                let mut bodies = branches
                    .iter()
                    .map(|branch| &branch.body)
                    .collect::<Vec<_>>();
                bodies.push(else_body);
                push_compound(statement, &bodies, candidates);
                for nested in bodies {
                    collect_nested_candidates(nested, candidates);
                }
            }
            Stmt::Quantifier { body, .. } => {
                push_compound(statement, std::slice::from_ref(&body), candidates);
                collect_nested_candidates(body, candidates);
            }
            Stmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                let mut bodies = Vec::new();
                if let Some(then_body) = then_body {
                    bodies.push(then_body);
                }
                bodies.push(else_body);
                push_compound(statement, &bodies, candidates);
                for nested in bodies {
                    collect_nested_candidates(nested, candidates);
                }
            }
            Stmt::DeclBlock { .. }
            | Stmt::Definition { .. }
            | Stmt::Mutate { .. }
            | Stmt::ResultFlow { .. }
            | Stmt::Expr { .. }
            | Stmt::Send { .. }
            | Stmt::Show { .. }
            | Stmt::Inspect { .. }
            | Stmt::PublicObservation { .. }
            | Stmt::MetaBlock { .. }
            | Stmt::Pragma { .. }
            | Stmt::Return { .. }
            | Stmt::Break { .. }
            | Stmt::ContinueLoop { .. } => {}
        }
    }
}

fn push_compound(statement: &Stmt, bodies: &[&Body], candidates: &mut Vec<Candidate>) {
    if bodies.iter().any(|body| !body.stmts.is_empty()) {
        let span = statement_span(statement);
        candidates.push(Candidate {
            range: ByteRange {
                start: span.start,
                end: span.end,
            },
            kind: FoldingKind::Compound,
        });
    }
}

fn statement_span(statement: &Stmt) -> ddonirang_lang::Span {
    match statement {
        Stmt::DeclBlock { span, .. }
        | Stmt::Definition { span, .. }
        | Stmt::Mutate { span, .. }
        | Stmt::ResultFlow { span, .. }
        | Stmt::Expr { span, .. }
        | Stmt::Receive { span, .. }
        | Stmt::Send { span, .. }
        | Stmt::Show { span, .. }
        | Stmt::Inspect { span, .. }
        | Stmt::PublicObservation { span, .. }
        | Stmt::MetaBlock { span, .. }
        | Stmt::Pragma { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::If { span, .. }
        | Stmt::Try { span, .. }
        | Stmt::Choose { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::While { span, .. }
        | Stmt::ForEach { span, .. }
        | Stmt::Quantifier { span, .. }
        | Stmt::Break { span, .. }
        | Stmt::ContinueLoop { span, .. }
        | Stmt::Contract { span, .. }
        | Stmt::Guard { span, .. }
        | Stmt::BeatBlock { span, .. }
        | Stmt::Hook { span, .. }
        | Stmt::HookWhenBecomes { span, .. }
        | Stmt::HookWhile { span, .. } => *span,
    }
}

fn normalize_candidates(
    candidates: Vec<Candidate>,
    source: &str,
    encoding: PositionEncoding,
) -> Result<Vec<NormalizedCandidate>, FoldingRangeError> {
    let mut normalized = candidates
        .into_iter()
        .filter_map(|candidate| {
            if candidate.range.start >= candidate.range.end {
                return None;
            }
            let Ok(range) = byte_range_to_lsp_range(source, candidate.range, encoding) else {
                return None;
            };
            (range.start.line < range.end.line).then_some(NormalizedCandidate {
                range,
                byte_range: candidate.range,
                kind: candidate.kind,
            })
        })
        .collect::<Vec<_>>();
    normalized.sort_by(|left, right| {
        left.byte_range
            .start
            .cmp(&right.byte_range.start)
            .then(right.byte_range.end.cmp(&left.byte_range.end))
    });
    normalized.dedup_by(|left, right| left.byte_range == right.byte_range);
    for pair in normalized.windows(2) {
        let left = pair[0].byte_range;
        let right = pair[1].byte_range;
        if right.start < left.end && right.end > left.end {
            return Err(FoldingRangeError::InvalidRange(
                "겹치는 folding range가 proper nesting이 아닙니다".into(),
            ));
        }
    }
    Ok(normalized)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedCandidate {
    range: LspRange,
    byte_range: ByteRange,
    kind: FoldingKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_ranges_are_outer_before_inner_and_source_ordered() {
        let source = "outer\ninner\nlast\n";
        let normalized = normalize_candidates(
            vec![
                Candidate {
                    range: ByteRange {
                        start: 6,
                        end: source.len() - 1,
                    },
                    kind: FoldingKind::Compound,
                },
                Candidate {
                    range: ByteRange {
                        start: 0,
                        end: source.len() - 1,
                    },
                    kind: FoldingKind::Definition,
                },
            ],
            source,
            PositionEncoding::Utf16,
        )
        .expect("nested ranges should normalize");
        assert_eq!(
            normalized
                .iter()
                .map(|candidate| candidate.byte_range)
                .collect::<Vec<_>>(),
            vec![
                ByteRange {
                    start: 0,
                    end: source.len() - 1,
                },
                ByteRange {
                    start: 6,
                    end: source.len() - 1,
                },
            ]
        );
    }
}
