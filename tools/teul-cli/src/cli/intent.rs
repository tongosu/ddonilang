use std::fs;
use std::path::{Path, PathBuf};

use ddonirang_core::fixed64::Fixed64;
use ddonirang_core::platform::SeulgiIntent;
use ddonirang_core::seulgi::intent::{intent_bundle_detjson, IntentRecord};
use serde_json::Value;

use super::detjson::write_text;

pub fn run_inspect(geoul: &Path, madi: Option<u64>, agent: Option<u64>, out: Option<&Path>) -> Result<(), String> {
    let jsonl_path = geoul.join("intent.jsonl");
    if !jsonl_path.exists() {
        return Err("E_INTENT_MISSING intent.jsonl이 없습니다".to_string());
    }
    let records = parse_intent_jsonl(&jsonl_path, madi, agent)?;
    let detjson = intent_bundle_detjson(&records);
    if let Some(path) = out {
        write_text(path, &detjson)?;
    } else {
        println!("{}", detjson);
    }
    Ok(())
}

pub fn run_mock(
    input: &Path,
    out: &Path,
    agent_id: u64,
    madi: u64,
    recv_seq: u64,
) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let mut line = String::new();
    line.push_str("{\"accepted_madi\":");
    line.push_str(&madi.to_string());
    line.push_str(",\"target_madi\":");
    line.push_str(&madi.to_string());
    line.push_str(",\"agent_id\":");
    line.push_str(&agent_id.to_string());
    line.push_str(",\"recv_seq\":");
    line.push_str(&recv_seq.to_string());
    line.push_str(",\"kind\":\"Say\",\"text\":\"");
    line.push_str(&escape_json(text.trim()));
    line.push_str("\"}");
    write_text(out, &format!("{}\n", line))?;
    Ok(())
}

pub fn run_merge(inputs: &[PathBuf], madi: Option<u64>, agent: Option<u64>, out: Option<&Path>) -> Result<(), String> {
    let mut records = Vec::new();
    for path in inputs.iter() {
        let mut subset = parse_intent_jsonl(path.as_path(), madi, agent)?;
        records.append(&mut subset);
    }
    let detjson = intent_bundle_detjson(&records);
    if let Some(path) = out {
        write_text(path, &detjson)?;
    } else {
        println!("{}", detjson);
    }
    Ok(())
}

fn parse_intent_jsonl(path: &Path, madi: Option<u64>, agent: Option<u64>) -> Result<Vec<IntentRecord>, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut records = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .map_err(|e| format!("E_INTENT_JSON line {}: {}", idx + 1, e))?;
        let accepted_madi = value_as_u64_opt(&value, "accepted_madi")?
            .or(value_as_u64_opt(&value, "madi")?)
            .ok_or_else(|| "E_INTENT_FIELD accepted_madi 없음".to_string())?;
        let target_madi = value_as_u64_opt(&value, "target_madi")?.unwrap_or(accepted_madi);
        if let Some(expect) = madi {
            if accepted_madi != expect {
                continue;
            }
        }
        let line_agent = value_as_u64(&value, "agent_id")?;
        if let Some(expect) = agent {
            if line_agent != expect {
                continue;
            }
        }
        let recv_seq = value_as_u64(&value, "recv_seq")?;
        let kind = value_as_str(&value, "kind")?;
        let intent = parse_intent(kind, &value)?;
        records.push(IntentRecord {
            agent_id: line_agent,
            recv_seq,
            accepted_madi,
            target_madi,
            intent,
        });
    }
    Ok(records)
}

fn parse_intent(kind: &str, value: &Value) -> Result<SeulgiIntent, String> {
    let normalized = kind.trim();
    match normalized {
        "None" | "none" => Ok(SeulgiIntent::None),
        "MoveTo" | "move_to" | "moveto" => {
            let x = value_as_fixed64(value, "x")?;
            let y = value_as_fixed64(value, "y")?;
            Ok(SeulgiIntent::MoveTo { x, y })
        }
        "Attack" | "attack" => {
            let target_id = value_as_u64(value, "target_id")?;
            Ok(SeulgiIntent::Attack { target_id })
        }
        "Say" | "say" | "speak" => {
            let text = value_as_str(value, "text")?.to_string();
            Ok(SeulgiIntent::Say { text })
        }
        other => Err(format!("E_INTENT_KIND 알 수 없는 intent kind: {}", other)),
    }
}

fn value_as_u64_opt(value: &Value, key: &str) -> Result<Option<u64>, String> {
    let Some(target) = value.get(key) else {
        return Ok(None);
    };
    match target {
        Value::Number(num) => num
            .as_u64()
            .ok_or_else(|| format!("E_INTENT_FIELD {} 숫자 아님", key))
            .map(Some),
        Value::String(text) => text
            .parse::<u64>()
            .map(Some)
            .map_err(|_| format!("E_INTENT_FIELD {} 숫자 변환 실패", key)),
        _ => Err(format!("E_INTENT_FIELD {} 형식 오류", key)),
    }
}

fn value_as_u64(value: &Value, key: &str) -> Result<u64, String> {
    let target = value.get(key).ok_or_else(|| format!("E_INTENT_FIELD {} 없음", key))?;
    match target {
        Value::Number(num) => num.as_u64().ok_or_else(|| format!("E_INTENT_FIELD {} 숫자 아님", key)),
        Value::String(text) => text.parse::<u64>().map_err(|_| format!("E_INTENT_FIELD {} 숫자 변환 실패", key)),
        _ => Err(format!("E_INTENT_FIELD {} 형식 오류", key)),
    }
}

fn value_as_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    let target = value.get(key).ok_or_else(|| format!("E_INTENT_FIELD {} 없음", key))?;
    match target {
        Value::String(text) => Ok(text.as_str()),
        _ => Err(format!("E_INTENT_FIELD {} 형식 오류", key)),
    }
}

fn value_as_fixed64(value: &Value, key: &str) -> Result<Fixed64, String> {
    let target = value.get(key).ok_or_else(|| format!("E_INTENT_FIELD {} 없음", key))?;
    match target {
        Value::Number(num) => parse_fixed64_string(&num.to_string()),
        Value::String(text) => parse_fixed64_string(text),
        _ => Err(format!("E_INTENT_FIELD {} 형식 오류", key)),
    }
}

fn parse_fixed64_string(input: &str) -> Result<Fixed64, String> {
    let text = input.trim();
    if text.is_empty() {
        return Err("E_INTENT_FIXED64 빈 문자열".to_string());
    }
    let mut sign = 1i128;
    let mut raw_text = text;
    if let Some(rest) = raw_text.strip_prefix('-') {
        sign = -1;
        raw_text = rest;
    } else if let Some(rest) = raw_text.strip_prefix('+') {
        raw_text = rest;
    }
    let mut parts = raw_text.splitn(2, '.');
    let int_part = parts.next().unwrap_or("");
    let frac_part = parts.next().unwrap_or("");

    let int_value = if int_part.is_empty() {
        0i128
    } else {
        if !int_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("E_INTENT_FIXED64 정수부 형식 오류: {}", input));
        }
        int_part
            .parse::<i128>()
            .map_err(|_| format!("E_INTENT_FIXED64 정수부 변환 실패: {}", input))?
    };

    let frac_value = if frac_part.is_empty() {
        0i128
    } else {
        if !frac_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("E_INTENT_FIXED64 소수부 형식 오류: {}", input));
        }
        frac_part
            .parse::<i128>()
            .map_err(|_| format!("E_INTENT_FIXED64 소수부 변환 실패: {}", input))?
    };

    let scale = 10i128.pow(frac_part.len() as u32);
    let frac_raw = if frac_part.is_empty() {
        0i128
    } else {
        (frac_value * (1i128 << 32)) / scale
    };

    let raw = (int_value << 32) + frac_raw;
    let signed = raw.saturating_mul(sign);
    let clamped = clamp_i128_to_i64(signed);
    Ok(Fixed64::from_raw_i64(clamped))
}

fn clamp_i128_to_i64(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
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
