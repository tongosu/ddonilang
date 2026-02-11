use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use blake3;
use clap::ValueEnum;
use serde_json::{json, Value};

use crate::canon;

use crate::lang::lexer::Lexer;
use crate::lang::token::TokenKind;

#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum PreviewFormat {
    Diff,
    Json,
}

pub struct PatchPreview {
    pub file: String,
    pub anchor: String,
    pub kind: String,
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug)]
struct PatchFile {
    patch_version: Option<String>,
    changes: Vec<PatchChange>,
}

#[derive(Clone, Debug)]
struct PatchChange {
    kind: String,
    target: PatchTarget,
    before: Option<Vec<String>>,
    after: Vec<String>,
    reason: Option<String>,
}

#[derive(Clone, Debug)]
struct PatchTarget {
    file: String,
    anchor: String,
}

#[derive(Clone, Debug)]
struct PatchApproval {
    patch_hash: String,
    approved: bool,
}

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

#[derive(Clone, Debug)]
struct FileBuffer {
    lines: Vec<String>,
    line_ending: String,
    trailing_newline: bool,
}

pub fn run_preview(path: &Path, format: PreviewFormat) -> Result<(), String> {
    let patch = load_patch(path)?;
    let previews = build_preview(&patch)?;
    match format {
        PreviewFormat::Diff => {
            if let Some(version) = patch.patch_version.as_deref() {
                println!("# patch_version: {}", version);
            }
            print_preview_diff(&previews);
        }
        PreviewFormat::Json => print_preview_json(&previews, patch.patch_version.as_deref()),
    }
    Ok(())
}

pub fn run_propose(file: &Path, out: Option<&Path>) -> Result<(), String> {
    let source = fs::read_to_string(file).map_err(|e| format!("E_PATCH_READ {}", e))?;
    let tokens = Lexer::tokenize(&source).map_err(|e| format!("{} patch 실패", e.code()))?;

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
            warnings.push(format!("E_PATCH_LINE line={} out of range", line_idx + 1));
            continue;
        }
        let old_line = lines[line_idx].clone();
        if old_line.is_empty() {
            continue;
        }
        if old_line.contains('{') {
            warnings.push(format!(
                "E_PATCH_SKIP_BLOCK line={} contains '{{'",
                line_idx + 1
            ));
            continue;
        }
        if line_counts.get(&old_line).copied().unwrap_or(0) > 1 {
            warnings.push(format!(
                "E_PATCH_SKIP_AMBIGUOUS line={} anchor is not unique",
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
            "before": [change.old_line],
            "after": [change.new_line],
            "reason": change.reason,
        }));
    }

    let out_path = out
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("ddn.patch.json"));
    let patch_json = json!({
        "patch_version": "0.1-draft",
        "changes": changes,
    });
    let text = serde_json::to_string_pretty(&patch_json).map_err(|e| e.to_string())? + "\n";
    fs::write(&out_path, text).map_err(|e| format!("E_PATCH_WRITE {}", e))?;
    println!("patch_written={}", out_path.display());

    for warning in warnings {
        eprintln!("{}", warning);
    }
    Ok(())
}

pub fn run_approve(path: &Path, out: &Path, yes: bool, notes: Option<String>) -> Result<(), String> {
    let patch_bytes = fs::read(path).map_err(|e| format!("E_PATCH_READ {}", e))?;
    let patch_hash = patch_hash_string(&patch_bytes);

    if !yes {
        let mut stdout = io::stdout();
        stdout
            .write_all("ddn.patch.json 승인을 진행할까요? (yes/no): ".as_bytes())
            .map_err(|e| format!("E_PATCH_APPROVE {}", e))?;
        stdout.flush().map_err(|e| format!("E_PATCH_APPROVE {}", e))?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("E_PATCH_APPROVE {}", e))?;
        if input.trim().to_ascii_lowercase() != "yes" {
            return Err("E_PATCH_APPROVE_ABORT 승인 취소됨".to_string());
        }
    }

    let approval_json = json!({
        "patch_hash": patch_hash,
        "approved": true,
        "approved_by": "manual",
        "scope": "workspace",
        "notes": notes.unwrap_or_else(|| "preview 확인 후 승인".to_string()),
    });
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("E_PATCH_WRITE {}", e))?;
    }
    fs::write(out, serde_json::to_string_pretty(&approval_json).unwrap() + "\n")
        .map_err(|e| format!("E_PATCH_WRITE {}", e))?;
    Ok(())
}

pub fn run_apply(path: &Path, approval: &Path, out: Option<&Path>, in_place: bool) -> Result<(), String> {
    let patch_bytes = fs::read(path).map_err(|e| format!("E_PATCH_READ {}", e))?;
    let patch_hash = patch_hash_string(&patch_bytes);
    let approval = load_approval(approval)?;
    validate_approval(&approval, &patch_hash)?;

    let patch = load_patch(path)?;
    let mut buffers = apply_patch_to_buffers(&patch)?;

    let (out_dir, apply_in_place) = resolve_apply_mode(out, in_place)?;

    for (path, buffer) in buffers.iter_mut() {
        let content = buffer_to_string(buffer);
        if Path::new(&path).extension().and_then(|s| s.to_str()) == Some("ddn") {
            ensure_canon(&content, &path)?;
        }
        let target_path = match out_dir.as_ref() {
            Some(dir) => {
                if Path::new(&path).is_absolute() {
                    return Err("E_PATCH_PATH out 모드에서는 상대 경로만 허용됩니다".to_string());
                }
                dir.join(path)
            }
            None => PathBuf::from(path),
        };
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("E_PATCH_WRITE {}", e))?;
        }
        if !apply_in_place && out_dir.is_none() {
            return Err("E_PATCH_MODE 적용 경로를 지정해야 합니다".to_string());
        }
        fs::write(target_path, content).map_err(|e| format!("E_PATCH_WRITE {}", e))?;
    }

    Ok(())
}

pub fn run_verify(
    path: &Path,
    approval: &Path,
    tests_root: Option<&Path>,
    walk: Option<&str>,
) -> Result<(), String> {
    let patch_bytes = fs::read(path).map_err(|e| format!("E_PATCH_READ {}", e))?;
    let patch_hash = patch_hash_string(&patch_bytes);
    let approval = load_approval(approval)?;
    validate_approval(&approval, &patch_hash)?;

    let root = match tests_root {
        Some(root) => root.to_path_buf(),
        None => {
            let default_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("golden");
            if !default_root.exists() {
                return Err("E_PATCH_VERIFY_TESTS --tests 경로가 필요합니다".to_string());
            }
            default_root
        }
    };
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("run_golden.py");
    let exe = std::env::current_exe().map_err(|e| format!("E_PATCH_VERIFY {}", e))?;
    let mut cmd = Command::new("python");
    cmd.arg(script)
        .arg("--root")
        .arg(root)
        .arg("--teul-cli")
        .arg(exe);
    if let Some(walk) = walk {
        cmd.arg("--walk").arg(walk);
    }
    let status = cmd.status().map_err(|e| format!("E_PATCH_VERIFY {}", e))?;
    if !status.success() {
        return Err("E_PATCH_VERIFY_FAIL tests 실패".to_string());
    }
    Ok(())
}

fn resolve_apply_mode(out: Option<&Path>, in_place: bool) -> Result<(Option<PathBuf>, bool), String> {
    if out.is_some() && in_place {
        return Err("E_PATCH_MODE --out과 --in-place는 함께 사용할 수 없습니다".to_string());
    }
    if out.is_none() && !in_place {
        return Ok((None, true));
    }
    Ok((out.map(|p| p.to_path_buf()), in_place || out.is_none()))
}

fn patch_hash_string(bytes: &[u8]) -> String {
    let digest = blake3::hash(bytes);
    format!("blake3:{}", digest.to_hex())
}

fn load_patch(path: &Path) -> Result<PatchFile, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("E_PATCH_READ {}", e))?;
    let json: Value = serde_json::from_str(&text).map_err(|e| format!("E_PATCH_JSON {}", e))?;
    let obj = json.as_object().ok_or_else(|| "E_PATCH_JSON root must be object".to_string())?;

    let patch_version = obj
        .get("patch_version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let changes = obj
        .get("changes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "E_PATCH_JSON changes must be array".to_string())?;
    let mut parsed_changes = Vec::new();
    for change in changes {
        parsed_changes.push(parse_change(change)?);
    }
    Ok(PatchFile {
        patch_version,
        changes: parsed_changes,
    })
}

fn parse_change(value: &Value) -> Result<PatchChange, String> {
    let obj = value.as_object().ok_or_else(|| "E_PATCH_JSON change must be object".to_string())?;
    let kind = obj
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "E_PATCH_JSON change.kind is required".to_string())?
        .to_string();
    let target = obj
        .get("target")
        .ok_or_else(|| "E_PATCH_JSON change.target is required".to_string())?;
    let target = parse_target(target)?;
    let before = obj
        .get("before")
        .map(|v| parse_lines(v, "before"))
        .transpose()?;
    let after = if let Some(value) = obj.get("after") {
        let parsed = parse_lines(value, "after")?;
        if let Some(code_value) = obj.get("code") {
            let code = parse_lines(code_value, "code")?;
            if code != parsed {
                return Err("E_PATCH_JSON change.after/code mismatch".to_string());
            }
        }
        parsed
    } else if let Some(code_value) = obj.get("code") {
        parse_lines(code_value, "code")?
    } else {
        return Err("E_PATCH_JSON change.after is required".to_string());
    };
    let reason = obj.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());
    Ok(PatchChange {
        kind,
        target,
        before,
        after,
        reason,
    })
}

fn parse_lines(value: &Value, field: &str) -> Result<Vec<String>, String> {
    let arr = value
        .as_array()
        .ok_or_else(|| format!("E_PATCH_JSON change.{field} must be array"))?;
    arr.iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| format!("E_PATCH_JSON change.{field} items must be strings"))
                .map(|s| s.to_string())
        })
        .collect::<Result<Vec<String>, String>>()
}

fn parse_target(value: &Value) -> Result<PatchTarget, String> {
    let obj = value.as_object().ok_or_else(|| "E_PATCH_JSON target must be object".to_string())?;
    let file = obj
        .get("file")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "E_PATCH_JSON target.file is required".to_string())?
        .to_string();
    let anchor = obj
        .get("anchor")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "E_PATCH_JSON target.anchor is required".to_string())?
        .to_string();
    Ok(PatchTarget { file, anchor })
}

fn build_preview(patch: &PatchFile) -> Result<Vec<PatchPreview>, String> {
    let mut buffers = read_patch_files(patch)?;
    let mut previews = Vec::new();
    for change in &patch.changes {
        if change.kind != "replace_block" {
            return Err(format!("E_PATCH_KIND unsupported kind: {}", change.kind));
        }
        let buffer = buffers
            .get_mut(&change.target.file)
            .ok_or_else(|| "E_PATCH_READ target file not loaded".to_string())?;
        let (start, end) = find_block_range(&buffer.lines, &change.target.anchor)?;
        let old_lines = buffer.lines[start..=end].to_vec();
        if let Some(before) = &change.before {
            if before != &old_lines {
                return Err("E_PATCH_BEFORE_MISMATCH before block does not match".to_string());
            }
        }
        let new_lines = change.after.clone();
        replace_lines(&mut buffer.lines, start, end, &new_lines);
        previews.push(PatchPreview {
            file: change.target.file.clone(),
            anchor: change.target.anchor.clone(),
            kind: change.kind.clone(),
            old_lines,
            new_lines,
            reason: change.reason.clone(),
        });
    }
    Ok(previews)
}

fn print_preview_diff(previews: &[PatchPreview]) {
    for preview in previews {
        println!("--- {}", preview.file);
        println!("+++ {}", preview.file);
        println!("@@ {} ({})", preview.anchor, preview.kind);
        if let Some(reason) = preview.reason.as_ref() {
            println!("# reason: {}", reason);
        }
        for line in &preview.old_lines {
            println!("-{}", line);
        }
        for line in &preview.new_lines {
            println!("+{}", line);
        }
    }
}

fn print_preview_json(previews: &[PatchPreview], patch_version: Option<&str>) {
    let changes = previews
        .iter()
        .map(|p| {
            json!({
                "file": p.file,
                "anchor": p.anchor,
                "kind": p.kind,
                "old_lines": p.old_lines,
                "new_lines": p.new_lines,
                "reason": p.reason,
            })
        })
        .collect::<Vec<_>>();
    let payload = json!({
        "patch_version": patch_version,
        "changes": changes,
    });
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
}

fn load_approval(path: &Path) -> Result<PatchApproval, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("E_PATCH_READ {}", e))?;
    let json: Value = serde_json::from_str(&text).map_err(|e| format!("E_PATCH_JSON {}", e))?;
    let obj = json.as_object().ok_or_else(|| "E_PATCH_JSON approval must be object".to_string())?;
    let patch_hash = obj
        .get("patch_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "E_PATCH_JSON approval.patch_hash missing".to_string())?
        .to_string();
    let approved = obj
        .get("approved")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "E_PATCH_JSON approval.approved missing".to_string())?;
    Ok(PatchApproval { patch_hash, approved })
}

fn validate_approval(approval: &PatchApproval, expected_hash: &str) -> Result<(), String> {
    if !approval.approved {
        return Err("E_PATCH_APPROVAL 승인되지 않은 패치입니다".to_string());
    }
    if approval.patch_hash != expected_hash {
        return Err("E_PATCH_APPROVAL patch_hash 불일치".to_string());
    }
    Ok(())
}

fn read_patch_files(patch: &PatchFile) -> Result<BTreeMap<String, FileBuffer>, String> {
    let mut buffers = BTreeMap::new();
    for change in &patch.changes {
        if buffers.contains_key(&change.target.file) {
            continue;
        }
        let path = PathBuf::from(&change.target.file);
        let buffer = read_file_buffer(&path)?;
        buffers.insert(change.target.file.clone(), buffer);
    }
    Ok(buffers)
}

fn read_file_buffer(path: &Path) -> Result<FileBuffer, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("E_PATCH_READ {}", e))?;
    let line_ending = if content.contains("\r\n") {
        "\r\n".to_string()
    } else {
        "\n".to_string()
    };
    let trailing_newline = content.ends_with('\n');
    let lines = content.lines().map(|line| line.to_string()).collect();
    Ok(FileBuffer {
        lines,
        line_ending,
        trailing_newline,
    })
}

fn apply_patch_to_buffers(patch: &PatchFile) -> Result<BTreeMap<String, FileBuffer>, String> {
    let mut buffers = read_patch_files(patch)?;
    for change in &patch.changes {
        if change.kind != "replace_block" {
            return Err(format!("E_PATCH_KIND unsupported kind: {}", change.kind));
        }
        let buffer = buffers
            .get_mut(&change.target.file)
            .ok_or_else(|| "E_PATCH_READ target file not loaded".to_string())?;
        let (start, end) = find_block_range(&buffer.lines, &change.target.anchor)?;
        if let Some(before) = &change.before {
            let current = &buffer.lines[start..=end];
            if current != before {
                return Err("E_PATCH_BEFORE_MISMATCH before block does not match".to_string());
            }
        }
        replace_lines(&mut buffer.lines, start, end, &change.after);
    }
    Ok(buffers)
}

fn find_block_range(lines: &[String], anchor: &str) -> Result<(usize, usize), String> {
    let matches: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| if line.contains(anchor) { Some(idx) } else { None })
        .collect();
    if matches.is_empty() {
        return Err(format!("E_PATCH_ANCHOR '{}' not found", anchor));
    }
    if matches.len() > 1 {
        return Err(format!("E_PATCH_ANCHOR '{}' is ambiguous", anchor));
    }
    let start = matches[0];
    let line = lines.get(start).cloned().unwrap_or_default();
    if !line.contains('{') {
        return Ok((start, start));
    }
    for idx in start + 1..lines.len() {
        if lines[idx].trim() == "}." {
            return Ok((start, idx));
        }
    }
    Err(format!("E_PATCH_BLOCK '{}' end not found", anchor))
}

fn replace_lines(lines: &mut Vec<String>, start: usize, end: usize, new_lines: &[String]) {
    let mut updated = Vec::new();
    updated.extend_from_slice(&lines[..start]);
    updated.extend_from_slice(new_lines);
    if end + 1 < lines.len() {
        updated.extend_from_slice(&lines[end + 1..]);
    }
    *lines = updated;
}

fn buffer_to_string(buffer: &FileBuffer) -> String {
    let mut out = buffer.lines.join(&buffer.line_ending);
    if buffer.trailing_newline {
        out.push_str(&buffer.line_ending);
    }
    out
}

fn ensure_canon(content: &str, path: &str) -> Result<(), String> {
    let output = canon::canonicalize(content, false)
        .map_err(|err| format!("E_PATCH_CANON_PARSE {}: {}", path, err))?;
    let mut normalized = output.ddn;
    if !normalized.ends_with('\n') {
        normalized.push('\n');
    }
    if normalized.trim_end() != content.trim_end() {
        return Err(format!("E_PATCH_CANON_MISMATCH {}", path));
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
            warnings.push(format!("E_PATCH_COL line={} col=0", line_no));
            continue;
        }
        let start_idx = rep.start_col - 1;
        let end_idx = start_idx + rep.len;
        if end_idx > chars.len() || start_idx >= chars.len() {
            warnings.push(format!(
                "E_PATCH_RANGE line={} col={} len={}",
                line_no, rep.start_col, rep.len
            ));
            continue;
        }
        let current: String = chars[start_idx..end_idx].iter().collect();
        if current != rep.old {
            warnings.push(format!(
                "E_PATCH_MISMATCH line={} col={} expected='{}' got='{}'",
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
