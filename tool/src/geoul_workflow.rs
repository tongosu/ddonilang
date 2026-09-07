use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::artifact_output::write_text_artifact_atomic;
use crate::geoul_bundle::{audit_hash, GeoulBundleReader};
use crate::state_trace_wire::escape_json_string_contents as escape_json;

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

fn hex32(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
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
        write_text_artifact_atomic(out_path, &output)?;
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
