use std::fs;
use std::path::Path;

use serde_json::{Map, Number, Value as JsonValue};

use super::detjson::{sha256_hex, write_text};

pub struct DotbogiCaseOptions<'a> {
    pub input: &'a Path,
    pub out: Option<&'a Path>,
    pub after_state_out: Option<&'a Path>,
    pub report_out: Option<&'a Path>,
}

pub fn run_case(options: DotbogiCaseOptions<'_>) -> Result<(), String> {
    let text = fs::read_to_string(options.input)
        .map_err(|e| format!("E_DOTBOGI_CASE_READ {} {}", options.input.display(), e))?;
    let doc: JsonValue =
        serde_json::from_str(&text).map_err(|e| format!("E_DOTBOGI_CASE_PARSE {}", e))?;
    let root = doc
        .as_object()
        .ok_or_else(|| "E_DOTBOGI_CASE_SCHEMA root는 object여야 합니다".to_string())?;
    if root.get("schema").and_then(|v| v.as_str()) != Some("ddn.dotbogi.case.v1") {
        return Err("E_DOTBOGI_CASE_SCHEMA schema=ddn.dotbogi.case.v1 이어야 합니다".to_string());
    }

    let input = root
        .get("input")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "E_DOTBOGI_CASE_INPUT input object가 필요합니다".to_string())?;
    if input.get("schema").and_then(|v| v.as_str()) != Some("dotbogi.input.v1") {
        return Err(
            "E_DOTBOGI_CASE_INPUT_SCHEMA input.schema=dotbogi.input.v1 이어야 합니다".to_string(),
        );
    }

    let dotbogi = root
        .get("dotbogi")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "E_DOTBOGI_CASE_DOTBOGI dotbogi object가 필요합니다".to_string())?;
    let view_meta = dotbogi
        .get("view_meta")
        .cloned()
        .unwrap_or(JsonValue::Object(Map::new()));
    if !view_meta.is_object() {
        return Err("E_DOTBOGI_CASE_VIEW_META dotbogi.view_meta는 object여야 합니다".to_string());
    }
    let events = dotbogi
        .get("events")
        .cloned()
        .unwrap_or(JsonValue::Array(Vec::new()));
    if !events.is_array() {
        return Err("E_DOTBOGI_CASE_EVENTS dotbogi.events는 list여야 합니다".to_string());
    }

    if is_forbidden_state_write(dotbogi.get("state_write")) {
        return Err("E_DOTBOGI_STATE_WRITE_FORBIDDEN dotbogi state write is forbidden".to_string());
    }

    let output = JsonValue::Object(Map::from_iter(vec![
        (
            "schema".to_string(),
            JsonValue::String("dotbogi.output.v1".to_string()),
        ),
        ("view_meta".to_string(), view_meta.clone()),
        ("events".to_string(), events.clone()),
    ]));
    let output_hash = hash_sha256(&output)?;
    let view_meta_hash = hash_sha256(&view_meta)?;

    let mut report_map = Map::new();
    report_map.insert(
        "schema".to_string(),
        JsonValue::String("ddn.dotbogi.case.report.v1".to_string()),
    );
    report_map.insert("output".to_string(), output.clone());
    report_map.insert(
        "output_hash".to_string(),
        JsonValue::String(output_hash.clone()),
    );
    report_map.insert(
        "view_meta_hash".to_string(),
        JsonValue::String(view_meta_hash.clone()),
    );

    let mut after_state: Option<JsonValue> = None;
    if let Some(roundtrip) = root.get("roundtrip").and_then(|v| v.as_object()) {
        let state = input
            .get("state")
            .cloned()
            .ok_or_else(|| "E_DOTBOGI_CASE_STATE input.state가 필요합니다".to_string())?;
        let mut state_mut = state;
        apply_roundtrip(roundtrip, &events, &mut state_mut)?;
        let after_state_hash = hash_sha256(&state_mut)?;
        report_map.insert("after_state".to_string(), state_mut.clone());
        report_map.insert(
            "after_state_hash".to_string(),
            JsonValue::String(after_state_hash.clone()),
        );
        after_state = Some(state_mut);
    }

    let report = JsonValue::Object(report_map);

    if let Some(path) = options.out {
        let text = serde_json::to_string_pretty(&output)
            .map_err(|e| format!("E_DOTBOGI_OUTPUT_SERIALIZE {}", e))?;
        write_text(path, &(text + "\n"))?;
    }
    if let (Some(path), Some(state)) = (options.after_state_out, after_state.as_ref()) {
        let text = serde_json::to_string_pretty(state)
            .map_err(|e| format!("E_DOTBOGI_AFTER_STATE_SERIALIZE {}", e))?;
        write_text(path, &(text + "\n"))?;
    }
    if let Some(path) = options.report_out {
        let text = serde_json::to_string_pretty(&report)
            .map_err(|e| format!("E_DOTBOGI_REPORT_SERIALIZE {}", e))?;
        write_text(path, &(text + "\n"))?;
    }

    println!("dotbogi_output_hash={}", output_hash);
    println!("dotbogi_view_meta_hash={}", view_meta_hash);
    if let Some(value) = report.get("after_state_hash").and_then(|v| v.as_str()) {
        println!("dotbogi_after_state_hash={}", value);
    }
    Ok(())
}

fn is_forbidden_state_write(value: Option<&JsonValue>) -> bool {
    let Some(value) = value else {
        return false;
    };
    match value {
        JsonValue::Null => false,
        JsonValue::Bool(false) => false,
        JsonValue::Object(map) if map.is_empty() => false,
        JsonValue::Array(items) if items.is_empty() => false,
        _ => true,
    }
}

fn hash_sha256(value: &JsonValue) -> Result<String, String> {
    let canonical = canonical_json_text(value)?;
    Ok(format!("sha256:{}", sha256_hex(canonical.as_bytes())))
}

fn canonical_json_text(value: &JsonValue) -> Result<String, String> {
    let normalized = canonicalize_json(value);
    serde_json::to_string(&normalized).map_err(|e| format!("E_DOTBOGI_CANON_JSON {}", e))
}

fn canonicalize_json(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            let mut out = Map::new();
            for key in keys {
                if let Some(item) = map.get(&key) {
                    out.insert(key, canonicalize_json(item));
                }
            }
            JsonValue::Object(out)
        }
        JsonValue::Array(items) => JsonValue::Array(items.iter().map(canonicalize_json).collect()),
        _ => value.clone(),
    }
}

fn apply_roundtrip(
    roundtrip: &Map<String, JsonValue>,
    events: &JsonValue,
    state: &mut JsonValue,
) -> Result<(), String> {
    if !state.is_object() {
        return Err("E_DOTBOGI_CASE_STATE input.state는 object여야 합니다".to_string());
    }
    let rules = roundtrip
        .get("event_rules")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            "E_DOTBOGI_CASE_ROUNDTRIP roundtrip.event_rules는 list여야 합니다".to_string()
        })?;

    let mut rule_map: std::collections::BTreeMap<String, Vec<JsonValue>> =
        std::collections::BTreeMap::new();
    for (idx, rule) in rules.iter().enumerate() {
        let rule_obj = rule.as_object().ok_or_else(|| {
            format!(
                "E_DOTBOGI_CASE_ROUNDTRIP roundtrip.event_rules[{}]는 object여야 합니다",
                idx
            )
        })?;
        let event_type = rule_obj
            .get("event_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                format!(
                    "E_DOTBOGI_CASE_ROUNDTRIP roundtrip.event_rules[{}].event_type가 필요합니다",
                    idx
                )
            })?
            .to_string();
        let ops = rule_obj
            .get("ops")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                format!(
                    "E_DOTBOGI_CASE_ROUNDTRIP roundtrip.event_rules[{}].ops는 list여야 합니다",
                    idx
                )
            })?
            .to_vec();
        rule_map.insert(event_type, ops);
    }

    for (event_idx, event) in events.as_array().into_iter().flatten().enumerate() {
        let Some(event_obj) = event.as_object() else {
            continue;
        };
        let Some(event_type) = event_obj.get("type").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(ops) = rule_map.get(event_type) else {
            continue;
        };
        apply_ops(state, ops, event_idx, event_type)?;
    }
    Ok(())
}

fn apply_ops(
    state: &mut JsonValue,
    ops: &[JsonValue],
    event_idx: usize,
    event_type: &str,
) -> Result<(), String> {
    for (op_idx, op) in ops.iter().enumerate() {
        let op_obj = op.as_object().ok_or_else(|| {
            format!(
                "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}]는 object여야 합니다",
                event_idx, event_type, op_idx
            )
        })?;
        let path = op_obj.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            format!(
                "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}].path가 필요합니다",
                event_idx, event_type, op_idx
            )
        })?;
        let mode = op_obj.get("op").and_then(|v| v.as_str()).ok_or_else(|| {
            format!(
                "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}].op가 필요합니다",
                event_idx, event_type, op_idx
            )
        })?;
        let segments: Vec<&str> = path.split('.').filter(|part| !part.is_empty()).collect();
        if segments.is_empty() {
            return Err(format!(
                "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}].path가 비었습니다",
                event_idx, event_type, op_idx
            ));
        }
        let mut cursor = &mut *state;
        for key in &segments[..segments.len() - 1] {
            let object = cursor.as_object_mut().ok_or_else(|| {
                format!(
                    "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}] 중간 경로가 object가 아닙니다: {}",
                    event_idx, event_type, op_idx, key
                )
            })?;
            cursor = object
                .entry((*key).to_string())
                .or_insert_with(|| JsonValue::Object(Map::new()));
            if !cursor.is_object() {
                return Err(format!(
                    "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}] 중간 경로가 object가 아닙니다: {}",
                    event_idx, event_type, op_idx, key
                ));
            }
        }
        let last = segments[segments.len() - 1];
        let object = cursor.as_object_mut().ok_or_else(|| {
            format!(
                "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}] 최종 부모 경로가 object가 아닙니다",
                event_idx, event_type, op_idx
            )
        })?;
        match mode {
            "set" => {
                let value = op_obj.get("value").cloned().unwrap_or(JsonValue::Null);
                object.insert(last.to_string(), value);
            }
            "add" => {
                let value = op_obj.get("value").ok_or_else(|| {
                    format!(
                        "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}].value가 필요합니다",
                        event_idx, event_type, op_idx
                    )
                })?;
                let current = object
                    .get(last)
                    .cloned()
                    .unwrap_or_else(|| JsonValue::Number(Number::from(0)));
                let sum = add_numbers(&current, value).ok_or_else(|| {
                    format!(
                        "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}] add는 숫자끼리만 허용됩니다",
                        event_idx, event_type, op_idx
                    )
                })?;
                object.insert(last.to_string(), sum);
            }
            _ => {
                return Err(format!(
                    "E_DOTBOGI_CASE_ROUNDTRIP event[{}:{}].ops[{}].op는 set|add만 허용됩니다",
                    event_idx, event_type, op_idx
                ));
            }
        }
    }
    Ok(())
}

fn add_numbers(lhs: &JsonValue, rhs: &JsonValue) -> Option<JsonValue> {
    if let (Some(a), Some(b)) = (lhs.as_i64(), rhs.as_i64()) {
        return Some(JsonValue::Number(Number::from(a.saturating_add(b))));
    }
    let a = lhs.as_f64()?;
    let b = rhs.as_f64()?;
    let sum = a + b;
    let number = Number::from_f64(sum)?;
    Some(JsonValue::Number(number))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ddn_dotbogi_{}_{}", name, stamp));
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    fn write_json(path: &Path, value: &JsonValue) {
        let text = serde_json::to_string_pretty(value).expect("json");
        fs::write(path, format!("{}\n", text)).expect("write");
    }

    #[test]
    fn forbidden_state_write_detection() {
        assert!(!is_forbidden_state_write(None));
        assert!(!is_forbidden_state_write(Some(&JsonValue::Null)));
        assert!(!is_forbidden_state_write(Some(&JsonValue::Bool(false))));
        assert!(!is_forbidden_state_write(Some(&JsonValue::Object(
            Map::new()
        ))));
        assert!(!is_forbidden_state_write(Some(&JsonValue::Array(
            Vec::new()
        ))));
        assert!(is_forbidden_state_write(Some(&JsonValue::Bool(true))));
        assert!(is_forbidden_state_write(Some(&serde_json::json!({"k": 1}))));
        assert!(is_forbidden_state_write(Some(&JsonValue::Array(vec![
            JsonValue::Null
        ]))));
    }

    #[test]
    fn run_case_generates_expected_roundtrip_report() {
        let dir = temp_dir("roundtrip");
        let input_path = dir.join("case.detjson");
        let output_path = dir.join("out.detjson");
        let after_state_path = dir.join("after_state.detjson");
        let report_path = dir.join("report.detjson");
        let doc = serde_json::json!({
            "schema": "ddn.dotbogi.case.v1",
            "input": {
                "schema": "dotbogi.input.v1",
                "state": {
                    "player": { "hp": 10, "potion": 1 }
                }
            },
            "dotbogi": {
                "view_meta": { "inventory": ["potion"] },
                "events": [{ "type": "아이템사용", "id": "potion" }]
            },
            "roundtrip": {
                "event_rules": [{
                    "event_type": "아이템사용",
                    "ops": [
                        { "op": "add", "path": "player.hp", "value": 5 },
                        { "op": "add", "path": "player.potion", "value": -1 }
                    ]
                }]
            }
        });
        write_json(&input_path, &doc);

        run_case(DotbogiCaseOptions {
            input: &input_path,
            out: Some(&output_path),
            after_state_out: Some(&after_state_path),
            report_out: Some(&report_path),
        })
        .expect("run_case");

        let report: JsonValue =
            serde_json::from_str(&fs::read_to_string(&report_path).expect("read report"))
                .expect("parse report");
        let output = report.get("output").cloned().expect("output");
        let after_state = report.get("after_state").cloned().expect("after_state");
        assert_eq!(
            output,
            serde_json::json!({
                "schema": "dotbogi.output.v1",
                "view_meta": { "inventory": ["potion"] },
                "events": [{ "type": "아이템사용", "id": "potion" }]
            })
        );
        assert_eq!(
            after_state,
            serde_json::json!({
                "player": { "hp": 15, "potion": 0 }
            })
        );

        let output_hash = report
            .get("output_hash")
            .and_then(|v| v.as_str())
            .expect("output hash");
        let view_meta_hash = report
            .get("view_meta_hash")
            .and_then(|v| v.as_str())
            .expect("view_meta hash");
        let after_state_hash = report
            .get("after_state_hash")
            .and_then(|v| v.as_str())
            .expect("after_state hash");

        assert_eq!(output_hash, hash_sha256(&output).expect("output hash calc"));
        assert_eq!(
            view_meta_hash,
            hash_sha256(output.get("view_meta").expect("view_meta")).expect("view_meta hash calc")
        );
        assert_eq!(
            after_state_hash,
            hash_sha256(&after_state).expect("after_state hash calc")
        );

        let output_doc: JsonValue =
            serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
                .expect("parse output");
        assert_eq!(output_doc, output);

        let after_state_doc: JsonValue =
            serde_json::from_str(&fs::read_to_string(&after_state_path).expect("read after state"))
                .expect("parse after state");
        assert_eq!(after_state_doc, after_state);
    }

    #[test]
    fn run_case_rejects_state_write() {
        let dir = temp_dir("write_forbidden");
        let input_path = dir.join("case.detjson");
        let doc = serde_json::json!({
            "schema": "ddn.dotbogi.case.v1",
            "input": {
                "schema": "dotbogi.input.v1",
                "state": { "hp": 1 }
            },
            "dotbogi": {
                "view_meta": {},
                "events": [],
                "state_write": { "hp": 999 }
            }
        });
        write_json(&input_path, &doc);
        let err = run_case(DotbogiCaseOptions {
            input: &input_path,
            out: None,
            after_state_out: None,
            report_out: None,
        })
        .expect_err("must fail");
        assert!(err.contains("E_DOTBOGI_STATE_WRITE_FORBIDDEN"));
    }
}
