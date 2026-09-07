use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use ddonirang_lang::{DialectConfig, Lexer, TokenKind};
use serde_json::json;

use crate::source_analysis::{analyze_source, SourceAnalysisRequest};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintAnalysis {
    pub warnings: Vec<String>,
    pub patch_json: String,
}

pub fn analyze_lint_source(
    source: &str,
    file_label: &str,
    source_profile_identity: &str,
) -> Result<LintAnalysis, String> {
    let snapshot = analyze_source(SourceAnalysisRequest {
        source_text: source.to_string(),
        path: file_label.to_string(),
        analysis_generation: 0,
        source_profile_identity: source_profile_identity.to_string(),
    });
    if snapshot.outcome != "ACCEPTED" {
        let diagnostic = snapshot
            .diagnostics
            .first()
            .map(|row| format!("{} {}", row.code, row.message))
            .unwrap_or_else(|| "E_LINT_SOURCE_REJECTED 진단이 없습니다".to_string());
        return Err(diagnostic);
    }

    let dialect = DialectConfig::from_source(source);
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|error| format!("E_LINT_LEX {error}"))?;
    let lines: Vec<String> = source.lines().map(str::to_string).collect();
    let mut line_counts: HashMap<String, usize> = HashMap::new();
    for line in &lines {
        *line_counts.entry(line.clone()).or_insert(0) += 1;
    }

    let mut warnings = snapshot
        .diagnostics
        .iter()
        .map(|row| {
            format!(
                "{} start={} end={} {}",
                row.code, row.span.start, row.span.end, row.message
            )
        })
        .collect::<Vec<_>>();
    warnings.extend(collect_i18n_warnings(source));
    let mut by_line: BTreeMap<usize, Vec<Replacement>> = BTreeMap::new();
    for token in tokens {
        let (start_line, start_col) = line_column_for_byte_offset(source, token.span.start);
        if let TokenKind::Ident(name) = &token.kind {
            if dialect.is_inactive_keyword(name) {
                warnings.push(format!(
                    "DIALECT_TOKEN_NOT_ACTIVE line={} col={} token={}",
                    start_line, start_col, name
                ));
            }
        }
        let Some(lexeme) = source.get(token.span.start..token.span.end) else {
            continue;
        };
        let Some(term) = find_legacy_term(lexeme) else {
            continue;
        };
        let line_idx = start_line.saturating_sub(1);
        by_line.entry(line_idx).or_default().push(Replacement {
            start_col,
            len: term.input.chars().count(),
            old: term.input.to_string(),
            new: term.canonical.to_string(),
            code: term.code.to_string(),
        });
    }

    let mut changes = Vec::new();
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
        let Some(change) =
            apply_replacements(&old_line, &replacements, line_idx + 1, &mut warnings)
        else {
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

    let patch_json = serde_json::to_string_pretty(&json!({
        "patch_version": "0.1-draft",
        "changes": changes,
    }))
    .map_err(|error| format!("E_LINT_SERIALIZE {error}"))?
        + "\n";
    Ok(LintAnalysis {
        warnings,
        patch_json,
    })
}

fn line_column_for_byte_offset(source: &str, target: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    for (offset, ch) in source.char_indices() {
        if offset >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

pub fn write_lint_patch_atomic(path: &Path, text: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(format!("E_LINT_OUTPUT_PARENT {}", parent.display()));
    }
    if path.is_dir() {
        return Err(format!("E_LINT_OUTPUT_IS_DIRECTORY {}", path.display()));
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("E_LINT_OUTPUT_PATH {}", path.display()))?
        .to_string_lossy();
    let nonce = std::process::id();
    let temp = parent.join(format!(".{file_name}.ddn-lint-tmp-{nonce}"));
    let backup = parent.join(format!(".{file_name}.ddn-lint-backup-{nonce}"));
    if temp.exists() || backup.exists() {
        return Err(format!("E_LINT_OUTPUT_RESIDUE {}", path.display()));
    }
    fs::write(&temp, text.as_bytes())
        .map_err(|error| format!("E_LINT_OUTPUT_WRITE {} {error}", temp.display()))?;

    let had_existing = path.exists();
    if had_existing {
        if let Err(error) = fs::rename(path, &backup) {
            let _ = fs::remove_file(&temp);
            return Err(format!("E_LINT_OUTPUT_BACKUP {} {error}", path.display()));
        }
    }
    if let Err(error) = fs::rename(&temp, path) {
        if had_existing && backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temp);
        return Err(format!("E_LINT_OUTPUT_COMMIT {} {error}", path.display()));
    }
    if backup.exists() {
        fs::remove_file(&backup)
            .map_err(|error| format!("E_LINT_OUTPUT_CLEANUP {} {error}", backup.display()))?;
    }
    Ok(())
}

fn collect_i18n_warnings(source: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let active_tag = detect_active_dialect_tag(source);
    for (index, line) in source.lines().enumerate() {
        let line_no = index + 1;
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
            Some("qu") if contains_ident_word(line, "mana") => warnings.push(format!(
                "I18N002_SYM3_REQUIRED line={} token=mana pair=none/not hint=sym3 표기 사용",
                line_no
            )),
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
        for prefix in ["말씨:", "사투리:"] {
            if let Some(tag) = rest.strip_prefix(prefix).map(str::trim) {
                if !tag.is_empty() {
                    return Some(tag.to_ascii_lowercase());
                }
            }
        }
    }
    None
}

fn parse_pragma_line(trimmed_line: &str) -> Option<&str> {
    trimmed_line
        .strip_prefix('#')
        .map(str::trim)
        .filter(|body| !body.is_empty())
}

fn is_setting_pragma_name(pragma_body: &str) -> bool {
    let name = pragma_body
        .split(|ch: char| ch == '(' || ch == ':' || ch.is_whitespace())
        .next()
        .unwrap_or("");
    matches!(
        name,
        "그래프" | "조종" | "관찰" | "추적" | "설정" | "보개" | "슬기"
    )
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
    let mut chars: Vec<char> = line.chars().collect();
    let mut sorted = replacements.to_vec();
    sorted.sort_by(|a, b| b.start_col.cmp(&a.start_col));
    let mut reasons = Vec::new();
    for replacement in sorted {
        if replacement.start_col == 0 {
            warnings.push(format!("E_LINT_COL line={} col=0", line_no));
            continue;
        }
        let start = replacement.start_col - 1;
        let end = start + replacement.len;
        if end > chars.len() || start >= chars.len() {
            warnings.push(format!(
                "E_LINT_RANGE line={} col={} len={}",
                line_no, replacement.start_col, replacement.len
            ));
            continue;
        }
        let current: String = chars[start..end].iter().collect();
        if current != replacement.old {
            warnings.push(format!(
                "E_LINT_MISMATCH line={} col={} expected='{}' got='{}'",
                line_no, replacement.start_col, replacement.old, current
            ));
            continue;
        }
        chars.splice(start..end, replacement.new.chars());
        reasons.push(format!(
            "{}:{}->{}",
            replacement.code, replacement.old, replacement.new
        ));
    }
    let new_line: String = chars.into_iter().collect();
    if new_line == line {
        return None;
    }
    Some(LineChange {
        old_line: line.to_string(),
        new_line,
        reason: format!("TERM-LINT-01: {}", reasons.join(", ")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_v25_lint_uses_shared_profile_and_term_mapping() {
        let source = "매마디:움직씨 := {\n  채비 {\n    변수:수 := 1.\n  }.\n}\n";
        let analysis =
            analyze_lint_source(source, "lesson.ddn", "v1-core-v25").expect("supported lint");
        assert!(analysis.patch_json.contains("TERM-WARN-001:변수->이름"));
        assert!(analysis.patch_json.contains("이름:수 := 1."));
    }

    #[test]
    fn supported_lint_fails_closed_for_omitted_or_unknown_profile() {
        let source = "매마디:움직씨 := {\n}\n";
        assert!(analyze_lint_source(source, "lesson.ddn", "")
            .expect_err("missing profile")
            .starts_with("E_SOURCE_PROFILE_REQUIRED"));
        assert!(analyze_lint_source(source, "lesson.ddn", "future-profile")
            .expect_err("unknown profile")
            .starts_with("E_SOURCE_PROFILE_UNSUPPORTED"));
    }

    #[test]
    fn legacy_internal_custody_preserves_patch_shape() {
        let source = "변수 <- 1.\n함수 <- 2.\n클래스 <- 3.\n이벤트 <- 4.\n";
        let analysis =
            analyze_lint_source(source, "main.ddn", "strict").expect("legacy custody lint");
        for marker in [
            "TERM-WARN-001",
            "TERM-WARN-002",
            "TERM-WARN-003",
            "TERM-WARN-004",
        ] {
            assert!(
                analysis.patch_json.contains(marker),
                "missing {marker} in {}",
                analysis.patch_json
            );
        }
    }

    #[test]
    fn i18n_warnings_remain_deterministic() {
        let source = "#말씨: ay\n#그래프(y축=바탕.x)\n값~xa <- 1.\njaniwa 조건.\n";
        let warnings = collect_i18n_warnings(source);
        assert!(warnings
            .iter()
            .any(|line| line.contains("I18N101_SETTINGS_PRAGMA_BLOCK")));
        assert!(warnings
            .iter()
            .any(|line| line.contains("I18N001_AMBIGUOUS_JOSA")));
        assert!(warnings
            .iter()
            .any(|line| line.contains("I18N002_SYM3_REQUIRED")));
    }
}
