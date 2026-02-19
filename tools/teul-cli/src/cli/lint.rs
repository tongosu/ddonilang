use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::lang::dialect::DialectConfig;
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
    let dialect = DialectConfig::from_source(&source);
    let tokens = Lexer::tokenize(&source).map_err(|e| format!("{} lint 실패", e.code()))?;

    let lines: Vec<String> = source.lines().map(|line| line.to_string()).collect();
    let mut line_counts: HashMap<String, usize> = HashMap::new();
    for line in &lines {
        *line_counts.entry(line.clone()).or_insert(0) += 1;
    }

    let mut warnings: Vec<String> = Vec::new();
    warnings.extend(collect_i18n_warnings(&source));
    let mut by_line: BTreeMap<usize, Vec<Replacement>> = BTreeMap::new();
    for token in tokens {
        let TokenKind::Ident(name) = &token.kind else {
            continue;
        };
        if dialect.is_inactive_keyword(name) {
            warnings.push(format!(
                "DIALECT_TOKEN_NOT_ACTIVE line={} col={} token={}",
                token.span.start_line, token.span.start_col, name
            ));
        }
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

fn collect_i18n_warnings(source: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let active_tag = detect_active_dialect_tag(source);
    for (idx, line) in source.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim_start_matches(|ch| matches!(ch, ' ' | '\t' | '\r' | '\u{feff}'));
        if let Some(pragma) = parse_pragma_line(trimmed) {
            if is_setting_pragma_name(pragma) {
                warnings.push(format!(
                    "I18N101_SETTINGS_PRAGMA_BLOCK line={} pragma=#{} use=설정보개/보개/보임/슬기 블록",
                    line_no, pragma
                ));
            }
        }
        match active_tag.as_deref() {
            Some("ay") => {
                if line.contains("~xa") {
                    warnings.push(format!(
                        "I18N001_AMBIGUOUS_JOSA line={} token=~xa hint=핀고정 또는 ~xa1/~xa2 사용",
                        line_no
                    ));
                }
                if contains_ident_word(line, "janiwa") {
                    warnings.push(format!(
                        "I18N002_SYM3_REQUIRED line={} token=janiwa pair=none/not hint=sym3 표기 사용",
                        line_no
                    ));
                }
            }
            Some("qu") => {
                if contains_ident_word(line, "mana") {
                    warnings.push(format!(
                        "I18N002_SYM3_REQUIRED line={} token=mana pair=none/not hint=sym3 표기 사용",
                        line_no
                    ));
                }
            }
            _ => {}
        }
    }
    warnings
}

fn detect_active_dialect_tag(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim_start_matches(|ch| matches!(ch, ' ' | '\t' | '\r' | '\u{feff}'));
        if !trimmed.starts_with('#') {
            continue;
        }
        let rest = trimmed[1..].trim();
        if rest.starts_with("말씨:") {
            let tag = rest["말씨:".len()..].trim();
            if !tag.is_empty() {
                return Some(tag.to_ascii_lowercase());
            }
        }
        if rest.starts_with("사투리:") {
            let tag = rest["사투리:".len()..].trim();
            if !tag.is_empty() {
                return Some(tag.to_ascii_lowercase());
            }
        }
    }
    None
}

fn parse_pragma_line(trimmed_line: &str) -> Option<&str> {
    if !trimmed_line.starts_with('#') {
        return None;
    }
    let body = trimmed_line[1..].trim();
    if body.is_empty() {
        return None;
    }
    Some(body)
}

fn is_setting_pragma_name(pragma_body: &str) -> bool {
    let name = pragma_body
        .split(|ch: char| ch == '(' || ch == ':' || ch.is_whitespace())
        .next()
        .unwrap_or("");
    matches!(name, "그래프" | "조종" | "관찰" | "추적" | "설정" | "보개" | "슬기")
}

fn contains_ident_word(line: &str, word: &str) -> bool {
    let mut current = String::new();
    for ch in line.chars() {
        if ch == '_' || ch == '\'' || ch.is_alphanumeric() {
            current.push(ch);
            continue;
        }
        if current == word {
            return true;
        }
        current.clear();
    }
    current == word
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

#[cfg(test)]
mod tests {
    use super::{collect_i18n_warnings, contains_ident_word, detect_active_dialect_tag};

    #[test]
    fn detect_active_dialect_header() {
        let src = "#말씨: ay\n값 <- 1.\n";
        assert_eq!(detect_active_dialect_tag(src).as_deref(), Some("ay"));
    }

    #[test]
    fn i18n_warns_for_setting_pragmas() {
        let src = "#그래프(y축=바탕.x)\n값 <- 1.\n";
        let warnings = collect_i18n_warnings(src);
        assert!(warnings
            .iter()
            .any(|line| line.contains("I18N101_SETTINGS_PRAGMA_BLOCK")));
    }

    #[test]
    fn i18n_warns_for_ay_xa_and_janiwa() {
        let src = "#말씨: ay\n값~xa <- 1.\njaniwa 조건.\n";
        let warnings = collect_i18n_warnings(src);
        assert!(warnings.iter().any(|line| line.contains("I18N001_AMBIGUOUS_JOSA")));
        assert!(warnings.iter().any(|line| line.contains("I18N002_SYM3_REQUIRED")));
    }

    #[test]
    fn contains_ident_word_matches_whole_token() {
        assert!(contains_ident_word("mana 조건", "mana"));
        assert!(!contains_ident_word("imanager", "mana"));
    }
}
