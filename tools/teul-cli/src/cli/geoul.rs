use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::sam_snapshot::apply_snapshot;
use crate::core::geoul::{
    audit_hash, decode_input_snapshot, geoul_state_hash_bytes, GeoulBundleReader, InputSnapshotV1,
};
use crate::core::state::Key;
use crate::core::value::Value;
use crate::core::State;
use crate::lang::lexer::Lexer;
use crate::lang::parser::Parser;
use crate::runtime::{Evaluator, RuntimeError};
use serde::Deserialize;
use serde_json::Value as JsonValue;

pub fn run_geoul_hash(dir: &Path) -> Result<(), String> {
    let hash = audit_hash(&dir.join("audit.ddni"))?;
    println!("audit_hash={}", hash);
    Ok(())
}

pub fn run_geoul_seek(dir: &Path, madi: u64) -> Result<(), String> {
    let mut reader = GeoulBundleReader::open(dir)?;
    let frame = reader.read_frame_header(madi)?;
    println!("state_hash=blake3:{}", hex32(&frame.state_hash));
    Ok(())
}

pub fn run_geoul_query(
    dir: &Path,
    madi: u64,
    key: &str,
    entry_override: Option<&Path>,
) -> Result<(), String> {
    let entry_path = resolve_entry_path(dir, entry_override)?;
    let source = std::fs::read_to_string(&entry_path)
        .map_err(|err| format!("E_GEOUL_ENTRY_READ {} {}", entry_path.display(), err))?;
    let snapshots = load_snapshots(dir, madi)?;
    let key = parse_query_key(key)?;

    let tokens = Lexer::tokenize(&source).map_err(|err| format!("E_GEOUL_LEX {:?}", err))?;
    let default_root = Parser::default_root_for_source(&source);
    let program = Parser::parse_with_default_root(tokens, default_root)
        .map_err(|err| format!("E_GEOUL_PARSE {:?}", err))?;
    let evaluator = Evaluator::with_state(State::new());

    let mut value_out: Option<String> = None;
    let mut hash_out: Option<[u8; 32]> = None;
    let mut before_tick = |tick: u64, state: &mut State| -> Result<(), RuntimeError> {
        if let Some(snapshot) = snapshots.get(tick as usize) {
            apply_snapshot(state, snapshot);
        }
        Ok(())
    };
    let mut on_tick = |tick: u64, state: &State, _tick_requested: bool| {
        if tick == madi {
            value_out = Some(value_canon(state, &key));
            hash_out = Some(geoul_state_hash_bytes(state));
        }
    };
    evaluator
        .run_with_ticks_observe_and_inject(&program, madi + 1, &mut before_tick, &mut on_tick)
        .map_err(|err| format!("E_GEOUL_RUNTIME {:?}", err))?;

    let value = value_out.unwrap_or_else(|| "없음".to_string());
    let hash = hash_out
        .map(|bytes| format!("blake3:{}", hex32(&bytes)))
        .unwrap_or_else(|| "blake3:".to_string());

    println!("madi={}", madi);
    println!("key={}", key_label(&key));
    println!("value={}", value);
    println!("state_hash={}", hash);
    Ok(())
}

pub fn run_geoul_backtrace(
    dir: &Path,
    key: &str,
    from: u64,
    to: u64,
    entry_override: Option<&Path>,
) -> Result<(), String> {
    if from > to {
        return Err(format!("E_GEOUL_RANGE from={} to={}", from, to));
    }
    let entry_path = resolve_entry_path(dir, entry_override)?;
    let source = std::fs::read_to_string(&entry_path)
        .map_err(|err| format!("E_GEOUL_ENTRY_READ {} {}", entry_path.display(), err))?;
    let snapshots = load_snapshots(dir, to)?;
    let key = parse_query_key(key)?;

    let tokens = Lexer::tokenize(&source).map_err(|err| format!("E_GEOUL_LEX {:?}", err))?;
    let default_root = Parser::default_root_for_source(&source);
    let program = Parser::parse_with_default_root(tokens, default_root)
        .map_err(|err| format!("E_GEOUL_PARSE {:?}", err))?;
    let evaluator = Evaluator::with_state(State::new());

    let changes: RefCell<Vec<(u64, String)>> = RefCell::new(Vec::new());
    let last_value: RefCell<Option<String>> = RefCell::new(None);
    let mut before_tick = |tick: u64, state: &mut State| -> Result<(), RuntimeError> {
        if let Some(snapshot) = snapshots.get(tick as usize) {
            apply_snapshot(state, snapshot);
        }
        Ok(())
    };
    let mut on_tick = |tick: u64, state: &State, _tick_requested: bool| {
        if tick < from || tick > to {
            return;
        }
        let value = value_canon(state, &key);
        let mut last = last_value.borrow_mut();
        let changed = match last.as_ref() {
            None => true,
            Some(prev) => prev != &value,
        };
        if changed {
            changes.borrow_mut().push((tick, value.clone()));
            *last = Some(value);
        }
    };
    evaluator
        .run_with_ticks_observe_and_inject(&program, to + 1, &mut before_tick, &mut on_tick)
        .map_err(|err| format!("E_GEOUL_RUNTIME {:?}", err))?;

    let changes = changes.into_inner();
    println!("key={}", key_label(&key));
    println!("from={}", from);
    println!("to={}", to);
    println!("change_count={}", changes.len());
    for (madi, value) in changes {
        println!("change_madi={} value={}", madi, value);
    }
    Ok(())
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{:02x}", b);
    }
    out
}

fn resolve_entry_path(dir: &Path, entry_override: Option<&Path>) -> Result<PathBuf, String> {
    let entry_path = entry_override
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| dir.join("entry.ddn"));
    if !entry_path.exists() {
        return Err(format!("E_GEOUL_ENTRY_MISSING {}", entry_path.display()));
    }
    Ok(entry_path)
}

fn load_snapshots(dir: &Path, until: u64) -> Result<Vec<InputSnapshotV1>, String> {
    let mut reader = GeoulBundleReader::open(dir)?;
    let frame_count = reader.frame_count();
    if frame_count == 0 {
        return Err("E_GEOUL_EMPTY_LOG geoul 로그에 프레임이 없습니다".to_string());
    }
    if until >= frame_count {
        return Err(format!(
            "E_GEOUL_RANGE until={} max={}",
            until,
            frame_count.saturating_sub(1)
        ));
    }
    let mut snapshots = Vec::with_capacity((until + 1) as usize);
    for madi in 0..=until {
        let frame = reader.read_frame(madi)?;
        if frame.header.madi != madi {
            return Err(format!(
                "E_GEOUL_FRAME_MADI_MISMATCH expected={} got={}",
                madi, frame.header.madi
            ));
        }
        let snapshot = decode_input_snapshot(&frame.snapshot_detbin)?;
        if snapshot.madi != madi {
            return Err(format!(
                "E_GEOUL_SNAPSHOT_MADI_MISMATCH expected={} got={}",
                madi, snapshot.madi
            ));
        }
        snapshots.push(snapshot);
    }
    Ok(snapshots)
}

fn parse_query_key(input: &str) -> Result<Key, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("E_GEOUL_KEY_EMPTY key가 비었습니다".to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("살림.") {
        if rest.is_empty() {
            return Err("E_GEOUL_KEY_EMPTY 살림. 뒤가 비었습니다".to_string());
        }
        return Ok(Key::new(rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("바탕.") {
        if rest.is_empty() {
            return Err("E_GEOUL_KEY_EMPTY 바탕. 뒤가 비었습니다".to_string());
        }
        return Ok(Key::new(rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("샘.") {
        if rest.is_empty() {
            return Err("E_GEOUL_KEY_EMPTY 샘. 뒤가 비었습니다".to_string());
        }
        return Ok(Key::new(format!("샘.{}", rest)));
    }
    Ok(Key::new(trimmed.to_string()))
}

fn key_label(key: &Key) -> String {
    if key.as_str().starts_with("샘.") || key.as_str().starts_with("입력상태.") {
        key.as_str().to_string()
    } else {
        format!("살림.{}", key.as_str())
    }
}

fn value_canon(state: &State, key: &Key) -> String {
    match state.get(key) {
        Some(value) => value.canon(),
        None => Value::None.canon(),
    }
}

#[derive(Deserialize)]
struct GeoulRecordMakeSpec {
    schema: Option<String>,
    meta: GeoulRecordMeta,
    steps: Vec<GeoulRecordStepSpec>,
}

#[derive(Deserialize)]
struct GeoulRecordMeta {
    ssot_version: String,
    created_at: String,
    cmd: String,
}

#[derive(Deserialize)]
struct GeoulRecordStepSpec {
    step: u64,
    state_hash: String,
    inputs_ref: Option<String>,
}

struct GeoulRecordStep {
    step: u64,
    state_hash: String,
}

pub fn run_geoul_record_make(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(input)
        .map_err(|e| format!("E_GEOUL_RECORD_READ {} {}", input.display(), e))?;
    let spec: GeoulRecordMakeSpec =
        serde_json::from_str(&text).map_err(|e| format!("E_GEOUL_RECORD_PARSE {}", e))?;
    if let Some(schema) = spec.schema.as_deref() {
        if schema != "geoul.record.make.v0" {
            return Err(format!("E_GEOUL_RECORD_SCHEMA {}", schema));
        }
    }
    let header = build_record_header_line(&spec.meta);
    let mut lines = Vec::with_capacity(spec.steps.len() + 1);
    lines.push(header);
    for step in &spec.steps {
        lines.push(build_record_step_line(step));
    }
    let mut output = String::new();
    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            output.push('\n');
        }
        output.push_str(line);
    }
    output.push('\n');
    if let Some(out_path) = out {
        fs::write(out_path, output).map_err(|e| e.to_string())?;
    } else {
        print!("{}", output);
    }
    Ok(())
}

pub fn run_geoul_record_check(input: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input)
        .map_err(|e| format!("E_GEOUL_RECORD_READ {} {}", input.display(), e))?;
    let mut lines = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        lines.push((idx + 1, trimmed.to_string()));
    }
    if lines.is_empty() {
        return Err("E_GEOUL_RECORD_EMPTY geoul.record.v0 파일이 비었습니다".to_string());
    }
    let (header_line_no, header_line) = lines[0].clone();
    let header_value: JsonValue = serde_json::from_str(&header_line).map_err(|e| {
        format!(
            "E_GEOUL_RECORD_HEADER_PARSE {}:{} {}",
            input.display(),
            header_line_no,
            e
        )
    })?;
    let schema = header_value
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!(
                "E_GEOUL_RECORD_HEADER {}:{} schema 누락",
                input.display(),
                header_line_no
            )
        })?;
    if schema != "geoul.record.v0" {
        return Err(format!(
            "E_GEOUL_RECORD_SCHEMA {}:{} {}",
            input.display(),
            header_line_no,
            schema
        ));
    }
    let meta = header_value
        .get("meta")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            format!(
                "E_GEOUL_RECORD_HEADER {}:{} meta 누락",
                input.display(),
                header_line_no
            )
        })?;
    for key in ["ssot_version", "created_at", "cmd"] {
        let ok = meta.get(key).and_then(|v| v.as_str()).is_some();
        if !ok {
            return Err(format!(
                "E_GEOUL_RECORD_HEADER {}:{} meta.{} 누락",
                input.display(),
                header_line_no,
                key
            ));
        }
    }

    let mut steps = Vec::new();
    for (line_no, line) in lines.iter().skip(1) {
        let value: JsonValue = serde_json::from_str(line).map_err(|e| {
            format!(
                "E_GEOUL_RECORD_STEP_PARSE {}:{} {}",
                input.display(),
                line_no,
                e
            )
        })?;
        let kind = value.get("kind").and_then(|v| v.as_str()).ok_or_else(|| {
            format!(
                "E_GEOUL_RECORD_STEP {}:{} kind 누락",
                input.display(),
                line_no
            )
        })?;
        if kind != "step" {
            return Err(format!(
                "E_GEOUL_RECORD_STEP_KIND {}:{} {}",
                input.display(),
                line_no,
                kind
            ));
        }
        let step = value.get("step").and_then(|v| v.as_u64()).ok_or_else(|| {
            format!(
                "E_GEOUL_RECORD_STEP {}:{} step 누락",
                input.display(),
                line_no
            )
        })?;
        let state_hash = value
            .get("state_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                format!(
                    "E_GEOUL_RECORD_STEP {}:{} state_hash 누락",
                    input.display(),
                    line_no
                )
            })?
            .to_string();
        let inputs_ref = value
            .get("inputs_ref")
            .map(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or(None);
        if value.get("inputs_ref").is_some() && inputs_ref.is_none() {
            return Err(format!(
                "E_GEOUL_RECORD_STEP {}:{} inputs_ref 형식 오류",
                input.display(),
                line_no
            ));
        }
        steps.push(GeoulRecordStep { step, state_hash });
    }

    println!("schema=geoul.record.v0");
    println!("step_count={}", steps.len());
    if let (Some(first), Some(last)) = (steps.first(), steps.last()) {
        println!("first_step={}", first.step);
        println!("last_step={}", last.step);
        println!("first_state_hash={}", first.state_hash);
        println!("last_state_hash={}", last.state_hash);
    }
    Ok(())
}

fn build_record_header_line(meta: &GeoulRecordMeta) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"geoul.record.v0\",\"meta\":{");
    out.push_str("\"ssot_version\":\"");
    out.push_str(&escape_json(&meta.ssot_version));
    out.push_str("\",\"created_at\":\"");
    out.push_str(&escape_json(&meta.created_at));
    out.push_str("\",\"cmd\":\"");
    out.push_str(&escape_json(&meta.cmd));
    out.push_str("\"}}");
    out
}

fn build_record_step_line(step: &GeoulRecordStepSpec) -> String {
    let mut out = String::new();
    out.push_str("{\"kind\":\"step\",\"step\":");
    out.push_str(&step.step.to_string());
    out.push_str(",\"state_hash\":\"");
    out.push_str(&escape_json(&step.state_hash));
    out.push_str("\"");
    if let Some(inputs_ref) = step.inputs_ref.as_deref() {
        out.push_str(",\"inputs_ref\":\"");
        out.push_str(&escape_json(inputs_ref));
        out.push('"');
    }
    out.push('}');
    out
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}
