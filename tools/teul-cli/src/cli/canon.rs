use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde_json::json;

use crate::canon::{self, CanonError};
use crate::cli::frontdoor_input::{
    prepare_frontdoor_canon_input, validate_no_legacy_frontdoor_surface,
};
use crate::lang::lexer::Lexer;
use crate::lang::parser::{ParseError, Parser};
use crate::lang::span::Span;
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
    GuseongFlatJson,
    AlrimPlanJson,
    ExecPolicyMapJson,
    MaegimControlJson,
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
    validate_no_legacy_frontdoor_surface(&source)?;
    if let Some(alias_kind) = detect_forbidden_event_surface_alias(&source) {
        let err = CanonError::new(
            "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
            format!(
                "비정본 이벤트 문형 금지({alias_kind}) — 정본은 \"KIND\"라는 알림이 오면 {{ ... }}."
            ),
        );
        return Err(err.to_string());
    }
    let fixits_json = build_fixits_json(&source, path);
    let bridge = matches!(args.bridge, Some(BridgeKind::Age0Step01));
    let legacy_block_header_colon_count = detect_legacy_block_header_colon_count(&source);

    if matches!(args.emit, EmitKind::Ddn) {
        let (ddn, meta, mut warnings) = canonicalize_ddn_strict(&source, path, bridge)?;
        append_block_header_warning_if_needed(&mut warnings, legacy_block_header_colon_count);
        ensure_no_inplace_fixits(path, &args, &fixits_json)?;
        if args.check && !is_canon_match(&source, &ddn) {
            let err = CanonError::new(
                "E_CANON_CHECK_MISMATCH",
                format!("정본 불일치: {}", path.display()),
            );
            maybe_write_fixits(&fixits_json, &args.fixits_json)?;
            maybe_write_diag(&args.diag_jsonl, &diag_error_line(&err))?;
            return Err(err.to_string());
        }
        for warning in &warnings {
            eprintln!("warning: {}", warning);
        }
        maybe_write_fixits(&fixits_json, &args.fixits_json)?;
        maybe_write_diag(&args.diag_jsonl, &diag_ok_line())?;
        maybe_write_meta(&meta, &args.meta_out)?;
        write_emit(
            path,
            &args,
            &fixits_json,
            &ddn,
            "{}\n",
            "{}\n",
            "{}\n",
            "{}\n",
        )?;
        return Ok(());
    }

    if matches!(args.emit, EmitKind::ExecPolicyMapJson) && !canon::has_exec_policy_surface(&source)
    {
        let (ddn, meta, mut warnings) = canonicalize_ddn_strict(&source, path, bridge)?;
        append_block_header_warning_if_needed(&mut warnings, legacy_block_header_colon_count);
        ensure_no_inplace_fixits(path, &args, &fixits_json)?;
        if args.check && !is_canon_match(&source, &ddn) {
            let err = CanonError::new(
                "E_CANON_CHECK_MISMATCH",
                format!("정본 불일치: {}", path.display()),
            );
            maybe_write_fixits(&fixits_json, &args.fixits_json)?;
            maybe_write_diag(&args.diag_jsonl, &diag_error_line(&err))?;
            return Err(err.to_string());
        }
        for warning in &warnings {
            eprintln!("warning: {}", warning);
        }
        maybe_write_fixits(&fixits_json, &args.fixits_json)?;
        maybe_write_diag(&args.diag_jsonl, &diag_ok_line())?;
        maybe_write_meta(&meta, &args.meta_out)?;
        write_emit(
            path,
            &args,
            &fixits_json,
            &ddn,
            "{}\n",
            "{}\n",
            "{}\n",
            "{}\n",
        )?;
        return Ok(());
    }

    let input = prepare_frontdoor_canon_input(&source);
    let result = canon::canonicalize(&input.prepared, bridge);

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
            write_emit(
                path,
                &args,
                &fixits_json,
                &output.ddn,
                &output.guseong_flat_json,
                &output.alrim_plan_json,
                &output.exec_policy_map_json,
                &output.maegim_control_json,
            )?;
            Ok(())
        }
        Err(err) => {
            maybe_write_fixits(&fixits_json, &args.fixits_json)?;
            maybe_write_diag(&args.diag_jsonl, &diag_error_line(&err))?;
            Err(err.to_string())
        }
    }
}

fn canonicalize_ddn_strict(
    source: &str,
    _path: &Path,
    bridge: bool,
) -> Result<(String, crate::file_meta::FileMeta, Vec<String>), String> {
    validate_no_legacy_frontdoor_surface(source)?;
    let input = prepare_frontdoor_canon_input(source);
    let output = canon::canonicalize(&input.prepared, bridge).map_err(|err| err.to_string())?;
    Ok((output.ddn, output.meta, output.warnings))
}

fn detect_legacy_block_header_colon_count(source: &str) -> usize {
    source
        .lines()
        .filter(|line| is_legacy_block_header_colon_line(line))
        .count()
}

fn is_legacy_block_header_colon_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
        return false;
    }
    let Some(colon_pos) = trimmed.find(':') else {
        return false;
    };
    let head = trimmed[..colon_pos].trim();
    if head.is_empty() {
        return false;
    }
    let tail = trimmed[colon_pos + 1..].trim_start();
    if !tail.starts_with('{') {
        return false;
    }
    // 타입 선언/대입/경로 표면은 제외하고, 헤더형 `키워드: {`만 감지한다.
    if head.contains('=') || head.contains("<-") || head.contains('.') {
        return false;
    }
    true
}

fn append_block_header_warning_if_needed(warnings: &mut Vec<String>, count: usize) {
    if count == 0 {
        return;
    }
    if warnings
        .iter()
        .any(|w| w.contains("W_BLOCK_HEADER_COLON_DEPRECATED"))
    {
        return;
    }
    warnings.push(format!(
        "W_BLOCK_HEADER_COLON_DEPRECATED 블록 헤더의 `키워드:` 표기는 예정된 비권장입니다. `키워드 {{` 표기로 전환하세요 (count={count})"
    ));
}

fn detect_forbidden_event_surface_alias(source: &str) -> Option<&'static str> {
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("알림 \"")
            && (trimmed.contains("\"가 오면") || trimmed.contains("\"이 오면"))
        {
            return Some("prefix_form");
        }
        if trimmed.starts_with('"') && (trimmed.contains(" 일때") || trimmed.contains(" 일때:"))
        {
            return Some("ilttae_form");
        }
        if trimmed.contains("라는 소식") && trimmed.contains("오면") {
            return Some("noun_alias");
        }
        if trimmed.contains("라는 알람") && trimmed.contains("오면") {
            return Some("noun_alias");
        }
    }
    None
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

fn write_emit(
    path: &Path,
    args: &CanonArgs,
    fixits_json: &str,
    ddn: &str,
    guseong_flat_json: &str,
    alrim_plan_json: &str,
    exec_policy_map_json: &str,
    maegim_control_json: &str,
) -> Result<(), String> {
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
        EmitKind::GuseongFlatJson => {
            if let Some(out_path) = args.out_dir.as_ref() {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("E_CLI_WRITE {}", e))?;
                }
                fs::write(out_path, guseong_flat_json).map_err(|e| format!("E_CLI_WRITE {}", e))?;
            } else {
                print!("{}", guseong_flat_json);
            }
            Ok(())
        }
        EmitKind::AlrimPlanJson => {
            if let Some(out_path) = args.out_dir.as_ref() {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("E_CLI_WRITE {}", e))?;
                }
                fs::write(out_path, alrim_plan_json).map_err(|e| format!("E_CLI_WRITE {}", e))?;
            } else {
                print!("{}", alrim_plan_json);
            }
            Ok(())
        }
        EmitKind::ExecPolicyMapJson => {
            if let Some(out_path) = args.out_dir.as_ref() {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("E_CLI_WRITE {}", e))?;
                }
                fs::write(out_path, exec_policy_map_json)
                    .map_err(|e| format!("E_CLI_WRITE {}", e))?;
            } else {
                print!("{}", exec_policy_map_json);
            }
            Ok(())
        }
        EmitKind::MaegimControlJson => {
            if let Some(out_path) = args.out_dir.as_ref() {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("E_CLI_WRITE {}", e))?;
                }
                fs::write(out_path, maegim_control_json)
                    .map_err(|e| format!("E_CLI_WRITE {}", e))?;
            } else {
                print!("{}", maegim_control_json);
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
    suggestion_kind: &'static str,
    old: Option<String>,
    new: Option<String>,
    note: Option<String>,
}

fn build_fixits_json(source: &str, path: &Path) -> String {
    let prepared = ddonirang_lang::preprocess_frontdoor_source(source);
    let tokens = match Lexer::tokenize(&prepared) {
        Ok(tokens) => tokens,
        Err(_) => return "[]\n".to_string(),
    };
    let file_label = path.to_string_lossy().to_string();
    let mut entries: Vec<FixitEntry> = Vec::new();

    collect_legacy_term_fixits(&tokens, &file_label, &mut entries);
    collect_surface_fixits(source, &file_label, &mut entries);
    if let Err(err) = Parser::parse_with_default_root(tokens, "살림") {
        if let Some(entry) = build_parse_fixit(source, &file_label, &err) {
            entries.push(entry);
        }
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
            let mut suggestion = json!({
                "kind": entry.suggestion_kind,
            });
            if let Some(old) = entry.old {
                suggestion["old"] = json!(old);
            }
            if let Some(new) = entry.new {
                suggestion["new"] = json!(new);
            }
            if let Some(note) = entry.note {
                suggestion["note"] = json!(note);
            }
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
                "suggestion": suggestion
            })
        })
        .collect();

    let text = serde_json::to_string_pretty(&json_entries).unwrap_or_else(|_| "[]".to_string());
    format!("{}\n", text)
}

fn collect_legacy_term_fixits(
    tokens: &[crate::lang::token::Token],
    file_label: &str,
    entries: &mut Vec<FixitEntry>,
) {
    for token in tokens {
        let TokenKind::Ident(name) = &token.kind else {
            continue;
        };
        let Some(term) = find_legacy_term(name.as_str()) else {
            continue;
        };
        let span = token.span;
        entries.push(FixitEntry {
            file: file_label.to_string(),
            start_line: span.start_line,
            start_col: span.start_col,
            end_line: span.end_line,
            end_col: normalize_end_col(span),
            code: term.code.to_string(),
            message: format!("레거시 용어를 정본으로 교체하세요: {}", term.canonical),
            suggestion_kind: "replace",
            old: Some(term.input.to_string()),
            new: Some(term.canonical.to_string()),
            note: None,
        });
    }
}

fn collect_surface_fixits(source: &str, file_label: &str, entries: &mut Vec<FixitEntry>) {
    const DEPRECATED_HEADER_KEYWORDS: &[&str] = &[
        "채비", "설정", "보개", "모양", "슬기", "반복", "동안", "대해",
    ];
    const ALIAS_REWRITES: &[(&str, &str, &str, &str)] = &[
        (
            "보개장면",
            "보개마당",
            "W_BOGAE_MADANG_ALIAS_DEPRECATED",
            "`보개장면`은 별칭입니다. 정본 키워드 `보개마당`을 사용하세요",
        ),
        (
            "구성",
            "짜임",
            "W_JJAIM_ALIAS_DEPRECATED",
            "`구성`은 별칭입니다. 정본 키워드 `짜임`을 사용하세요",
        ),
    ];

    for (line_idx, line) in source.lines().enumerate() {
        let line_no = line_idx + 1;
        let trimmed = line.trim_start();
        let leading_cols = line.chars().count().saturating_sub(trimmed.chars().count());
        for keyword in DEPRECATED_HEADER_KEYWORDS {
            let needle = format!("{keyword}:");
            if !trimmed.starts_with(&needle) {
                continue;
            }
            let colon_col = leading_cols + keyword.chars().count() + 1;
            entries.push(FixitEntry {
                file: file_label.to_string(),
                start_line: line_no,
                start_col: colon_col,
                end_line: line_no,
                end_col: colon_col,
                code: "W_BLOCK_HEADER_COLON_DEPRECATED".to_string(),
                message: format!("블록 헤더의 `{keyword}:` 표기를 `{keyword} {{`로 바꾸세요"),
                suggestion_kind: "delete",
                old: Some(":".to_string()),
                new: None,
                note: Some("블록 헤더에서는 콜론을 제거하고 공백 뒤 `{`를 사용하세요.".to_string()),
            });
            break;
        }

        for (legacy, canonical, code, message) in ALIAS_REWRITES {
            for column in find_standalone_keyword_columns(line, legacy) {
                entries.push(FixitEntry {
                    file: file_label.to_string(),
                    start_line: line_no,
                    start_col: column,
                    end_line: line_no,
                    end_col: column + legacy.chars().count().saturating_sub(1),
                    code: (*code).to_string(),
                    message: (*message).to_string(),
                    suggestion_kind: "replace",
                    old: Some((*legacy).to_string()),
                    new: Some((*canonical).to_string()),
                    note: None,
                });
            }
        }
    }
}

fn build_parse_fixit(source: &str, file_label: &str, err: &ParseError) -> Option<FixitEntry> {
    let span = parse_error_span(err);
    let (suggestion_kind, old, new, note) = match err {
        ParseError::ExpectedRParen { .. } => (
            "insert",
            None,
            Some(")".to_string()),
            Some("열린 괄호를 닫아 식을 완결하세요.".to_string()),
        ),
        ParseError::ExpectedRBrace { .. } => (
            "insert",
            None,
            Some("}".to_string()),
            Some("열린 블록을 닫아 문장을 완결하세요.".to_string()),
        ),
        ParseError::MaegimRequiresGroupedValue { .. } => {
            let replacement = build_grouped_maegim_replacement(source, span)?;
            (
                "replace",
                Some(replacement.0),
                Some(replacement.1),
                Some("매김 값은 괄호로 감싸서 적어야 합니다.".to_string()),
            )
        }
        ParseError::BlockHeaderColonForbidden { .. } => (
            "delete",
            Some(":".to_string()),
            None,
            Some("레거시 선언 헤더의 콜론을 제거하고 `채비 {` 형태로 전환하세요.".to_string()),
        ),
        ParseError::EventSurfaceAliasForbidden { .. } => (
            "replace",
            Some("오면".to_string()),
            Some("받으면".to_string()),
            Some("이벤트 반응은 `받으면 (...) { ... }` 형태로 옮기세요.".to_string()),
        ),
        ParseError::EffectSurfaceAliasForbidden { .. } => (
            "replace",
            None,
            Some("너머".to_string()),
            Some("열림/효과/바깥 별칭 대신 정본 블록 `너머 { ... }`를 사용하세요.".to_string()),
        ),
        ParseError::RootHideUndeclared { name, .. } if name == "제" => (
            "replace",
            Some("제".to_string()),
            None,
            Some("`제`는 임자 안에서만 씁니다. 임자 밖에서는 대상 이름을 직접 적으세요.".to_string()),
        ),
        ParseError::RootHideUndeclared { name, .. } => (
            "replace",
            Some(name.clone()),
            None,
            Some("채비에 선언된 이름만 쓸 수 있습니다. 대상을 먼저 선언하거나 bare key를 사용하세요.".to_string()),
        ),
        ParseError::ExpectedExpr { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some("여기에는 값/경로/호출식이 와야 합니다.".to_string()),
        ),
        ParseError::ExpectedPath { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some("여기에는 `x` 또는 `공.x` 같은 이름/경로식이 와야 합니다.".to_string()),
        ),
        ParseError::ExpectedTarget { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some("대입 대상은 이름 또는 경로여야 합니다.".to_string()),
        ),
        ParseError::UnsupportedCompoundTarget { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some("복합 갱신 대상은 단순 이름/경로로 줄이세요.".to_string()),
        ),
        ParseError::UnexpectedToken { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some("토큰 순서를 다시 확인하고, 문장을 `.` 또는 블록 `}`로 닫으세요.".to_string()),
        ),
        ParseError::ExpectedUnit { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some("여기에는 유효한 단위식이 와야 합니다. 예: `m`, `s`, `kg`.".to_string()),
        ),
        ParseError::ReceiveOutsideImja { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some("`받으면` 블록은 `임자` 안으로 옮기세요.".to_string()),
        ),
        ParseError::DeferredAssignOutsideBeat { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some("미루기 대입은 `덩이 { ... }` 안에서만 허용됩니다.".to_string()),
        ),
        ParseError::ChaebiInLoop { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some("`채비 {}`는 루프/훅/순회 body 안에서 사용할 수 없습니다.".to_string()),
        ),
        ParseError::LifecycleNameDuplicate {
            name, first_span, ..
        } => (
            "replace",
            Some(name.clone()),
            Some(propose_lifecycle_rename_candidate(source, name)),
            Some(format!(
                "같은 lifecycle 이름이 이미 {}:{}에 선언되었습니다. 이름을 바꿔 중복을 해소하세요.",
                first_span.start_line, first_span.start_col
            )),
        ),
        ParseError::QuantifierMutationForbidden { .. }
        | ParseError::QuantifierShowForbidden { .. }
        | ParseError::QuantifierIoForbidden { .. }
        | ParseError::ImmediateProofMutationForbidden { .. }
        | ParseError::ImmediateProofShowForbidden { .. }
        | ParseError::ImmediateProofIoForbidden { .. }
        | ParseError::CaseCompletionRequired { .. }
        | ParseError::CaseElseNotLast { .. }
        | ParseError::CompatEqualDisabled { .. }
        | ParseError::InvalidTensor { .. }
        | ParseError::ImportAliasDuplicate { .. }
        | ParseError::ImportAliasReserved { .. }
        | ParseError::ImportPathInvalid { .. }
        | ParseError::ImportVersionConflict { .. }
        | ParseError::ExportBlockDuplicate { .. }
        | ParseError::MaegimStepSplitConflict { .. }
        | ParseError::MaegimNestedSectionUnsupported { .. }
        | ParseError::MaegimNestedFieldUnsupported { .. }
        | ParseError::HookEveryNMadiIntervalInvalid { .. }
        | ParseError::HookEveryNMadiUnitUnsupported { .. }
        | ParseError::HookEveryNMadiSuffixUnsupported { .. }
        | ParseError::TagDuplicateInContext { .. }
        | ParseError::CompatMaticEntryDisabled { .. } => (
            "replace",
            span_snippet(source, span),
            None,
            Some(err.code().to_string()),
        ),
    };

    Some(FixitEntry {
        file: file_label.to_string(),
        start_line: span.start_line,
        start_col: span.start_col,
        end_line: span.end_line,
        end_col: normalize_end_col(span),
        code: err.code().to_string(),
        message: format!("{} fix-it 제안", err.code()),
        suggestion_kind,
        old,
        new,
        note,
    })
}

fn parse_error_span(err: &ParseError) -> Span {
    match err {
        ParseError::UnexpectedToken { span, .. }
        | ParseError::ExpectedExpr { span }
        | ParseError::ExpectedPath { span }
        | ParseError::ExpectedTarget { span }
        | ParseError::RootHideUndeclared { span, .. }
        | ParseError::UnsupportedCompoundTarget { span }
        | ParseError::ExpectedRParen { span }
        | ParseError::ExpectedRBrace { span }
        | ParseError::ExpectedUnit { span }
        | ParseError::InvalidTensor { span }
        | ParseError::CompatEqualDisabled { span }
        | ParseError::CompatMaticEntryDisabled { span }
        | ParseError::BlockHeaderColonForbidden { span }
        | ParseError::EventSurfaceAliasForbidden { span }
        | ParseError::EffectSurfaceAliasForbidden { span }
        | ParseError::ImportAliasDuplicate { span }
        | ParseError::ImportAliasReserved { span }
        | ParseError::ImportPathInvalid { span }
        | ParseError::ImportVersionConflict { span }
        | ParseError::ExportBlockDuplicate { span }
        | ParseError::LifecycleNameDuplicate { span, .. }
        | ParseError::ReceiveOutsideImja { span }
        | ParseError::MaegimRequiresGroupedValue { span }
        | ParseError::MaegimStepSplitConflict { span }
        | ParseError::MaegimNestedSectionUnsupported { span, .. }
        | ParseError::MaegimNestedFieldUnsupported { span, .. }
        | ParseError::HookEveryNMadiIntervalInvalid { span }
        | ParseError::HookEveryNMadiUnitUnsupported { span, .. }
        | ParseError::HookEveryNMadiSuffixUnsupported { span, .. }
        | ParseError::DeferredAssignOutsideBeat { span }
        | ParseError::QuantifierMutationForbidden { span }
        | ParseError::QuantifierShowForbidden { span }
        | ParseError::QuantifierIoForbidden { span }
        | ParseError::ImmediateProofMutationForbidden { span }
        | ParseError::ImmediateProofShowForbidden { span }
        | ParseError::ImmediateProofIoForbidden { span }
        | ParseError::CaseCompletionRequired { span }
        | ParseError::CaseElseNotLast { span }
        | ParseError::TagDuplicateInContext { span, .. }
        | ParseError::ChaebiInLoop { span } => *span,
    }
}

fn normalize_end_col(span: Span) -> usize {
    if span.end_line == span.start_line && span.end_col > 0 {
        span.end_col.saturating_sub(1)
    } else {
        span.end_col
    }
}

fn span_snippet(source: &str, span: Span) -> Option<String> {
    if span.start_line != span.end_line {
        return None;
    }
    let line = source.lines().nth(span.start_line.saturating_sub(1))?;
    let chars: Vec<char> = line.chars().collect();
    let start = span.start_col.saturating_sub(1);
    let end_inclusive = normalize_end_col(span);
    if start >= chars.len() || end_inclusive == 0 {
        return None;
    }
    let end_exclusive = end_inclusive
        .saturating_sub(1)
        .min(chars.len().saturating_sub(1))
        .saturating_add(1);
    Some(chars[start..end_exclusive].iter().collect())
}

fn build_grouped_maegim_replacement(source: &str, span: Span) -> Option<(String, String)> {
    let line = source.lines().nth(span.start_line.saturating_sub(1))?;
    let maegim_idx = line.find("매김")?;
    let equals_idx = line[..maegim_idx].rfind('=')?;
    let value = line[(equals_idx + 1)..maegim_idx].trim();
    if value.is_empty() {
        return None;
    }
    Some((value.to_string(), format!("({value})")))
}

fn find_standalone_keyword_columns(line: &str, needle: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for (byte_idx, _) in line.match_indices(needle) {
        let before = line[..byte_idx].chars().next_back();
        let after = line[(byte_idx + needle.len())..].chars().next();
        if before.is_some_and(is_ident_like) || after.is_some_and(is_ident_like) {
            continue;
        }
        let col = line[..byte_idx].chars().count() + 1;
        out.push(col);
    }
    out
}

fn source_contains_identifier_name(source: &str, name: &str) -> bool {
    match Lexer::tokenize(source) {
        Ok(tokens) => tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Ident(text) if text == name)),
        Err(_) => source.contains(name),
    }
}

fn propose_lifecycle_rename_candidate(source: &str, base: &str) -> String {
    for suffix in 2..=10_000 {
        let candidate = format!("{base}{suffix}");
        if !source_contains_identifier_name(source, &candidate) {
            return candidate;
        }
    }
    format!("{base}_renamed")
}

fn is_ident_like(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '.')
}

fn find_legacy_term(name: &str) -> Option<&'static LegacyTerm> {
    LEGACY_TERMS.iter().find(|term| term.input == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str, ext: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("teul_cli_canon_{name}_{nonce}.{ext}"));
        path
    }

    fn write_temp_ddn(source: &str, name: &str) -> std::path::PathBuf {
        let path = unique_temp_path(name, "ddn");
        fs::write(&path, source).expect("write source");
        path
    }

    fn run_emit_and_read(source: &str, emit: EmitKind, name: &str) -> String {
        let src = write_temp_ddn(source, name);
        let out = unique_temp_path(name, "json");
        let args = CanonArgs {
            emit,
            out_dir: Some(out.clone()),
            bridge: None,
            fixits_json: None,
            diag_jsonl: None,
            meta_out: None,
            check: false,
        };
        run(&src, args).expect("run canon");
        let text = fs::read_to_string(&out).expect("read out");
        let _ = fs::remove_file(src);
        let _ = fs::remove_file(out);
        text
    }

    fn run_expect_error(source: &str, emit: EmitKind, name: &str) -> String {
        let src = write_temp_ddn(source, name);
        let args = CanonArgs {
            emit,
            out_dir: None,
            bridge: None,
            fixits_json: None,
            diag_jsonl: None,
            meta_out: None,
            check: false,
        };
        let err = run(&src, args).expect_err("canon must fail");
        let _ = fs::remove_file(src);
        err
    }

    #[test]
    fn lifecycle_fixit_candidate_uses_next_available_suffix() {
        let source = r#"
연습판 = 판 {
}.
연습판2 = 판 {
}.
연습판3 = 판 {
}.
"#;
        let candidate = propose_lifecycle_rename_candidate(source, "연습판");
        assert_eq!(candidate, "연습판4");
    }

    #[test]
    fn lifecycle_fixit_candidate_defaults_to_suffix_two() {
        let source = r#"
연습판 = 판 {
}.
"#;
        let candidate = propose_lifecycle_rename_candidate(source, "연습판");
        assert_eq!(candidate, "연습판2");
    }

    #[test]
    fn run_exec_policy_emit_without_surface_uses_empty_fast_path() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let out = run_emit_and_read(
            source,
            EmitKind::ExecPolicyMapJson,
            "exec_policy_emit_empty_fast_path",
        );
        assert_eq!(out.trim(), "{}");
    }

    #[test]
    fn run_exec_policy_emit_with_surface_writes_policy_map_schema() {
        let source = r#"
너머 {
  실행모드: 일반.
  효과정책: 허용.
}.
"#;
        let out = run_emit_and_read(
            source,
            EmitKind::ExecPolicyMapJson,
            "exec_policy_emit_with_surface",
        );
        assert!(out.contains("\"schema\": \"ddn.exec_policy_effect_map.v1\""));
        assert!(out.contains("\"map\": {}"));
        assert_ne!(out.trim(), "{}");
    }

    #[test]
    fn run_guseong_emit_writes_flatten_schema() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let out = run_emit_and_read(source, EmitKind::GuseongFlatJson, "guseong_emit_schema");
        assert!(out.contains("\"schema\": \"ddn.guseong_flatten_plan.v1\""));
    }

    #[test]
    fn run_maegim_emit_writes_control_schema() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
  }.
}.
"#;
        let out = run_emit_and_read(source, EmitKind::MaegimControlJson, "maegim_emit_schema");
        assert!(out.contains("\"schema\": \"ddn.maegim_control_plan.v1\""));
        assert!(out.contains("\"name\": \"g\""));
    }

    #[test]
    fn run_ddn_rejects_forbidden_event_surface_alias() {
        let source = r#"
"jump"라는 소식이 오면 {
  x <- 1.
}.
"#;
        let err = run_expect_error(source, EmitKind::Ddn, "event_alias_forbidden");
        assert!(err.contains("E_EVENT_SURFACE_ALIAS_FORBIDDEN"));
    }

    #[test]
    fn canon_rejects_legacy_hash_header_for_all_emit_kinds() {
        let source = "#이름: 금지\n(매마디)마다 { 살림.n <- 1. }.";
        let emits = [
            EmitKind::Ddn,
            EmitKind::GuseongFlatJson,
            EmitKind::AlrimPlanJson,
            EmitKind::ExecPolicyMapJson,
            EmitKind::MaegimControlJson,
            EmitKind::FixitsJson,
            EmitKind::Both,
        ];
        for emit in emits {
            let err = run_expect_error(source, emit, "legacy_header_all_emit");
            assert!(
                err.contains("E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN"),
                "emit={emit:?} err={err}"
            );
        }
    }

    #[test]
    fn canon_accepts_boim_surface_for_ddn_emit() {
        let source = "보임 { x: 1. }.";
        let text = run_emit_and_read(source, EmitKind::Ddn, "boim_ddn_emit");
        assert!(text.contains("보임"), "text={text:?}");
    }
}
