use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::lang::lexer::Lexer;
use crate::lang::token::TokenKind;

struct LegacyTerm {
    input: &'static str,
    canonical: &'static str,
    code: &'static str,
}

const LEGACY_TERMS: &[LegacyTerm] = &[
    LegacyTerm {
        input: "변수",
        canonical: "이름",
        code: "TERM-WARN-001",
    },
    LegacyTerm {
        input: "함수",
        canonical: "움직씨",
        code: "TERM-WARN-002",
    },
    LegacyTerm {
        input: "클래스",
        canonical: "이름씨",
        code: "TERM-WARN-003",
    },
    LegacyTerm {
        input: "이벤트",
        canonical: "알림씨",
        code: "TERM-WARN-004",
    },
];

#[derive(Clone)]
struct Replacement {
    start_col: usize,
    len: usize,
    old: String,
    new: String,
    code: String,
}

struct LineChange {
    old_line: String,
    new_line: String,
    reason: String,
}

pub fn run(file: &Path, suggest_patch: bool, out: Option<&Path>) -> Result<(), String> {
    let source = fs::read_to_string(file).map_err(|e| format!("E_LINT_READ {}", e))?;
    let tokens = Lexer::tokenize(&source).map_err(|e| format!("{} lint 실패", e.code()))?;

    let lines: Vec<String> = source.lines().map(|line| line.to_string()).collect();
    let mut line_counts: HashMap<String, usize> = HashMap::new();
    for line in &lines {
        *line_counts.entry(line.clone()).or_insert(0) += 1;
    }

    let mut by_line: BTreeMap<usize, Vec<Replacement>> = BTreeMap::new();
    for token in tokens {
        let TokenKind::Ident(name) = &token.kind else {
            continue;
        };
        let Some(term) = find_legacy_term(name.as_str()) else {
            continue;
        };
        let line_idx = token.span.start_line.saturating_sub(1);
        let start_col = token.span.start_col;
        let len = name.chars().count();
        by_line
            .entry(line_idx)
            .or_default()
            .push(Replacement {
                start_col,
                len,
                old: term.input.to_string(),
                new: term.canonical.to_string(),
                code: term.code.to_string(),
            });
    }

    let mut changes: Vec<serde_json::Value> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let file_label = file.to_string_lossy().to_string();

    for (line_idx, replacements) in by_line {
        if line_idx >= lines.len() {
            warnings.push(format!("E_LINT_LINE line={} out of range", line_idx + 1));
            continue;
        }
        let old_line = lines[line_idx].clone();
        if old_line.is_empty() {
            continue;
        }
        if old_line.contains('{') {
            warnings.push(format!(
                "E_LINT_SKIP_BLOCK line={} contains '{{'",
                line_idx + 1
            ));
            continue;
        }
        if line_counts.get(&old_line).copied().unwrap_or(0) > 1 {
            warnings.push(format!(
                "E_LINT_SKIP_AMBIGUOUS line={} anchor is not unique",
                line_idx + 1
            ));
            continue;
        }
        let Some(change) = apply_replacements(
            &old_line,
            &replacements,
            line_idx + 1,
            &mut warnings,
        ) else {
            continue;
        };
        changes.push(json!({
            "kind": "replace_block",
            "target": {
                "file": file_label,
                "anchor": change.old_line,
            },
            "code": [change.new_line],
            "reason": change.reason,
        }));
    }

    if suggest_patch {
        let out_path = out
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("ddn.patch.json"));
        let patch_json = json!({
            "patch_version": "0.1-draft",
            "changes": changes,
        });
        let text = serde_json::to_string_pretty(&patch_json).map_err(|e| e.to_string())? + "\n";
        fs::write(&out_path, text).map_err(|e| format!("E_LINT_WRITE {}", e))?;
        println!("patch_written={}", out_path.display());
    }

    for warning in warnings {
        eprintln!("{}", warning);
    }
    Ok(())
}

fn find_legacy_term(name: &str) -> Option<&'static LegacyTerm> {
    LEGACY_TERMS.iter().find(|term| term.input == name)
}

fn apply_replacements(
    line: &str,
    replacements: &[Replacement],
    line_no: usize,
    warnings: &mut Vec<String>,
) -> Option<LineChange> {
    if replacements.is_empty() {
        return None;
    }
    let mut chars: Vec<char> = line.chars().collect();
    let mut sorted = replacements.to_vec();
    sorted.sort_by(|a, b| b.start_col.cmp(&a.start_col));
    let mut reasons = Vec::new();

    for rep in sorted {
        if rep.start_col == 0 {
            warnings.push(format!("E_LINT_COL line={} col=0", line_no));
            continue;
        }
        let start_idx = rep.start_col - 1;
        let end_idx = start_idx + rep.len;
        if end_idx > chars.len() || start_idx >= chars.len() {
            warnings.push(format!(
                "E_LINT_RANGE line={} col={} len={}",
                line_no, rep.start_col, rep.len
            ));
            continue;
        }
        let current: String = chars[start_idx..end_idx].iter().collect();
        if current != rep.old {
            warnings.push(format!(
                "E_LINT_MISMATCH line={} col={} expected='{}' got='{}'",
                line_no, rep.start_col, rep.old, current
            ));
            continue;
        }
        chars.splice(start_idx..end_idx, rep.new.chars());
        reasons.push(format!("{}:{}->{}", rep.code, rep.old, rep.new));
    }

    let new_line: String = chars.into_iter().collect();
    if new_line == line {
        return None;
    }
    let reason = format!("TERM-LINT-01: {}", reasons.join(", "));
    Some(LineChange {
        old_line: line.to_string(),
        new_line,
        reason,
    })
}
