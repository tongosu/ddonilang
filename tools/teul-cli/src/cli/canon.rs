use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde_json::json;

use crate::canon::{self, CanonError};
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

#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum EmitKind {
    Ddn,
    FixitsJson,
    Both,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum BridgeKind {
    Age0Step01,
}

pub struct CanonArgs {
    pub emit: EmitKind,
    pub out_dir: Option<PathBuf>,
    pub bridge: Option<BridgeKind>,
    pub fixits_json: Option<PathBuf>,
    pub diag_jsonl: Option<PathBuf>,
    pub meta_out: Option<PathBuf>,
    pub check: bool,
}

pub fn run(path: &Path, args: CanonArgs) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| format!("E_CLI_READ {}", e))?;
    let fixits_json = build_fixits_json(&source, path);
    let bridge = matches!(args.bridge, Some(BridgeKind::Age0Step01));
    let result = canon::canonicalize(&source, bridge);

    match result {
        Ok(output) => {
            ensure_no_inplace_fixits(path, &args, &fixits_json)?;
            if args.check && !is_canon_match(&source, &output.ddn) {
                let err = CanonError::new(
                    "E_CANON_CHECK_MISMATCH",
                    format!("정본 불일치: {}", path.display()),
                );
                maybe_write_fixits(&fixits_json, &args.fixits_json)?;
                maybe_write_diag(&args.diag_jsonl, &diag_error_line(&err))?;
                return Err(err.to_string());
            }
            for warning in &output.warnings {
                eprintln!("warning: {}", warning);
            }
            maybe_write_fixits(&fixits_json, &args.fixits_json)?;
            maybe_write_diag(&args.diag_jsonl, &diag_ok_line())?;
            maybe_write_meta(&output.meta, &args.meta_out)?;
            write_emit(path, &args, &fixits_json, &output.ddn)?;
            Ok(())
        }
        Err(err) => {
            maybe_write_fixits(&fixits_json, &args.fixits_json)?;
            maybe_write_diag(&args.diag_jsonl, &diag_error_line(&err))?;
            Err(err.to_string())
        }
    }
}

fn ensure_no_inplace_fixits(
    path: &Path,
    args: &CanonArgs,
    fixits_json: &str,
) -> Result<(), String> {
    if !has_fixits(fixits_json) {
        return Ok(());
    }
    if !matches!(args.emit, EmitKind::Ddn) {
        return Ok(());
    }
    let Some(out_path) = args.out_dir.as_ref() else {
        return Ok(());
    };
    if is_same_path(path, out_path) {
        return Err(
            "E_CANON_FIXIT_INPLACE fix-it이 필요한 입력은 자동 반영할 수 없습니다.".to_string(),
        );
    }
    Ok(())
}

fn has_fixits(fixits_json: &str) -> bool {
    let trimmed = fixits_json.trim();
    !(trimmed.is_empty() || trimmed == "[]")
}

fn is_same_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn write_emit(path: &Path, args: &CanonArgs, fixits_json: &str, ddn: &str) -> Result<(), String> {
    match args.emit {
        EmitKind::Ddn => {
            if let Some(out_path) = args.out_dir.as_ref() {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("E_CLI_WRITE {}", e))?;
                }
                fs::write(out_path, ddn).map_err(|e| format!("E_CLI_WRITE {}", e))?;
            } else {
                print!("{}", ddn);
            }
            Ok(())
        }
        EmitKind::FixitsJson => {
            print!("{}", fixits_json);
            Ok(())
        }
        EmitKind::Both => {
            let out_dir = args
                .out_dir
                .as_ref()
                .ok_or_else(|| "E_CLI_MISSING_OUT 출력 디렉터리가 필요합니다.".to_string())?;
            fs::create_dir_all(out_dir).map_err(|e| format!("E_CLI_WRITE {}", e))?;
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| "E_CLI_BAD_PATH 파일 이름이 필요합니다.".to_string())?;
            let canon_path = out_dir.join(format!("{}.canon.ddn", stem));
            let fixits_path = out_dir.join(format!("{}.fixits.json", stem));
            fs::write(canon_path, ddn).map_err(|e| format!("E_CLI_WRITE {}", e))?;
            fs::write(fixits_path, fixits_json).map_err(|e| format!("E_CLI_WRITE {}", e))?;
            Ok(())
        }
    }
}

fn maybe_write_fixits(fixits_json: &str, path: &Option<PathBuf>) -> Result<(), String> {
    if let Some(path) = path {
        fs::write(path, fixits_json).map_err(|e| format!("E_CLI_WRITE {}", e))?;
    }
    Ok(())
}

fn maybe_write_meta(
    meta: &crate::file_meta::FileMeta,
    path: &Option<PathBuf>,
) -> Result<(), String> {
    if let Some(path) = path {
        let text = crate::file_meta::file_meta_to_json(meta);
        fs::write(path, text).map_err(|e| format!("E_CLI_WRITE {}", e))?;
    }
    Ok(())
}

fn maybe_write_diag(path: &Option<PathBuf>, line: &str) -> Result<(), String> {
    if let Some(path) = path {
        fs::write(path, format!("{}\n", line)).map_err(|e| format!("E_CLI_WRITE {}", e))?;
    }
    Ok(())
}

fn is_canon_match(source: &str, canon: &str) -> bool {
    let mut normalized = canon.to_string();
    if !normalized.ends_with('\n') {
        normalized.push('\n');
    }
    normalized.trim_end() == source.trim_end()
}

fn diag_ok_line() -> String {
    format!(
        "{{\"kind\":\"canon\",\"level\":\"info\",\"code\":\"OK\",\"message\":\"{}\"}}",
        json_escape("정본화 완료")
    )
}

fn diag_error_line(err: &CanonError) -> String {
    format!(
        "{{\"kind\":\"canon\",\"level\":\"error\",\"code\":\"{}\",\"message\":\"{}\"}}",
        err.code(),
        json_escape(&err.to_string())
    )
}

fn json_escape(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

struct FixitEntry {
    file: String,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    code: String,
    message: String,
    old: String,
    new: String,
}

fn build_fixits_json(source: &str, path: &Path) -> String {
    let tokens = match Lexer::tokenize(source) {
        Ok(tokens) => tokens,
        Err(_) => return "[]\n".to_string(),
    };
    let file_label = path.to_string_lossy().to_string();
    let mut entries: Vec<FixitEntry> = Vec::new();

    for token in tokens {
        let TokenKind::Ident(name) = &token.kind else {
            continue;
        };
        let Some(term) = find_legacy_term(name.as_str()) else {
            continue;
        };
        let span = token.span;
        let end_col = if span.end_line == span.start_line && span.end_col > 0 {
            span.end_col.saturating_sub(1)
        } else {
            span.end_col
        };
        entries.push(FixitEntry {
            file: file_label.clone(),
            start_line: span.start_line,
            start_col: span.start_col,
            end_line: span.end_line,
            end_col,
            code: term.code.to_string(),
            message: format!("레거시 용어를 정본으로 교체하세요: {}", term.canonical),
            old: term.input.to_string(),
            new: term.canonical.to_string(),
        });
    }

    entries.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then_with(|| a.start_line.cmp(&b.start_line))
            .then_with(|| a.start_col.cmp(&b.start_col))
            .then_with(|| a.end_line.cmp(&b.end_line))
            .then_with(|| a.end_col.cmp(&b.end_col))
            .then_with(|| a.code.cmp(&b.code))
    });

    let json_entries: Vec<_> = entries
        .into_iter()
        .map(|entry| {
            json!({
                "code": entry.code,
                "message": entry.message,
                "span": {
                    "file": entry.file,
                    "start_line": entry.start_line,
                    "start_col": entry.start_col,
                    "end_line": entry.end_line,
                    "end_col": entry.end_col
                },
                "suggestion": {
                    "kind": "replace",
                    "old": entry.old,
                    "new": entry.new
                }
            })
        })
        .collect();

    let text = serde_json::to_string_pretty(&json_entries).unwrap_or_else(|_| "[]".to_string());
    format!("{}\n", text)
}

fn find_legacy_term(name: &str) -> Option<&'static LegacyTerm> {
    LEGACY_TERMS.iter().find(|term| term.input == name)
}
