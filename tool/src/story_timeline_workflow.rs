use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_json::{Map, Value};

use crate::artifact_output::write_text_artifact_atomic;

const AUDIT_MAGIC: &[u8; 4] = b"DDNI";
const AUDIT_VERSION: u16 = 1;
const AUDIT_HEADER_REMAINDER_BYTES: usize = 28;

pub fn run_story_make(geoul_dir: &Path, out_path: &Path) -> Result<(), String> {
    let frame_count = geoul_frame_count(geoul_dir)?;
    let t1 = if frame_count == 0 { 0 } else { frame_count - 1 };
    let summary = format!("frames={frame_count}");
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str("  \"version\": 1,\n");
    output.push_str(&format!("  \"summary\": \"{summary}\",\n"));
    output.push_str("  \"scenes\": [\n");
    output.push_str(&format!(
        "    {{\"t0\": 0, \"t1\": {t1}, \"kind\": \"summary\", \"text\": \"{summary}\"}}\n"
    ));
    output.push_str("  ],\n");
    output.push_str("  \"suggested_intents\": [\n");
    if frame_count > 0 {
        output.push_str(&format!(
            "    {{\"agent_id\": 1, \"recv_seq\": 1, \"intent\": {{\"kind\": \"말하기\", \"text\": \"{summary}\"}}}}\n"
        ));
    }
    output.push_str("  ]\n");
    output.push_str("}\n");

    write_text_artifact_atomic(out_path, &output)?;
    println!("story_written={}", out_path.display());
    Ok(())
}

pub fn run_timeline_make(
    geoul_dir: &Path,
    story_path: &Path,
    out_path: &Path,
) -> Result<(), String> {
    let frame_count = geoul_frame_count(geoul_dir)?;
    let story_text = std::fs::read_to_string(story_path).map_err(|error| error.to_string())?;
    let story_json: Value = serde_json::from_str(&story_text).map_err(|error| error.to_string())?;
    let scenes = story_json
        .get("scenes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut items = Vec::with_capacity(scenes.len());
    for scene in scenes {
        let Some(object) = scene.as_object() else {
            continue;
        };
        let t0 = object.get("t0").cloned().unwrap_or(Value::Number(0.into()));
        let t1 = object.get("t1").cloned().unwrap_or(Value::Number(0.into()));
        let kind = object
            .get("kind")
            .cloned()
            .unwrap_or(Value::String("summary".to_string()));
        let text = object
            .get("text")
            .cloned()
            .unwrap_or(Value::String(String::new()));
        let mut item = Map::new();
        item.insert("t0".to_string(), t0);
        item.insert("t1".to_string(), t1);
        item.insert("kind".to_string(), kind);
        item.insert("text".to_string(), text);
        items.push(Value::Object(item));
    }

    let mut root = Map::new();
    root.insert("version".to_string(), Value::Number(1.into()));
    root.insert("frames".to_string(), Value::Number(frame_count.into()));
    root.insert("items".to_string(), Value::Array(items));
    let output = serde_json::to_string_pretty(&Value::Object(root))
        .map_err(|error| format!("E_TIMELINE_JSON {error}"))?
        + "\n";

    write_text_artifact_atomic(out_path, &output)?;
    println!("timeline_written={}", out_path.display());
    Ok(())
}

fn geoul_frame_count(geoul_dir: &Path) -> Result<u64, String> {
    let audit_path = geoul_dir.join("audit.ddni");
    let index_path = geoul_dir.join("audit.idx");
    let mut audit = File::open(audit_path).map_err(|error| error.to_string())?;
    let mut magic = [0_u8; 4];
    audit
        .read_exact(&mut magic)
        .map_err(|error| error.to_string())?;
    if &magic != AUDIT_MAGIC {
        return Err("audit.ddni magic 불일치".to_string());
    }
    let mut header = [0_u8; AUDIT_HEADER_REMAINDER_BYTES];
    audit
        .read_exact(&mut header)
        .map_err(|error| error.to_string())?;
    let version = u16::from_le_bytes([header[0], header[1]]);
    if version != AUDIT_VERSION {
        return Err(format!("audit.ddni version 불일치: {version}"));
    }

    let mut index = File::open(index_path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    index
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() % 8 != 0 {
        return Err("audit.idx 길이가 8의 배수가 아닙니다".to_string());
    }
    u64::try_from(bytes.len() / 8).map_err(|_| "audit.idx frame count 범위 오류".to_string())
}
