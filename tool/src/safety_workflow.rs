use std::fs;
use std::path::Path;

use ddonirang_core::seulgi::safety::{check, SafetyMode, SafetyRule};
use serde_json::Value;

use crate::artifact_output::{require_file_artifact_target, write_text_artifact_atomic};
use crate::state_trace_wire::escape_json_string_contents as escape_json;

pub fn run_check(rules: &Path, intent: &Path, out: Option<&Path>) -> Result<(), String> {
    let rule_text = fs::read_to_string(rules).map_err(|e| e.to_string())?;
    let intent_text = fs::read_to_string(intent).map_err(|e| e.to_string())?;
    let rule_value: Value = serde_json::from_str(&rule_text).map_err(|e| e.to_string())?;
    let intent_value: Value = serde_json::from_str(&intent_text).map_err(|e| e.to_string())?;

    let mode_str = rule_value
        .get("mode")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "E_SAFETY_RULE mode 없음".to_string())?;
    let mode = match mode_str {
        "allowlist" => SafetyMode::AllowList,
        "denylist" => SafetyMode::DenyList,
        other => return Err(format!("E_SAFETY_RULE mode 오류: {}", other)),
    };
    let intents = match rule_value.get("intents") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    };

    let rule = SafetyRule { mode, intents };
    let kind = intent_value
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "E_SAFETY_INTENT kind 없음".to_string())?;

    let decision = check(&rule, kind);
    let detjson = format!(
        "{{\"schema\":\"seulgi.safety_check.v1\",\"allowed\":{},\"reason\":\"{}\"}}",
        if decision.allowed { "true" } else { "false" },
        escape_json(&decision.reason)
    );

    if let Some(path) = out {
        require_file_artifact_target(path)?;
        write_text_artifact_atomic(path, &detjson)?;
    } else {
        println!("{}", detjson);
    }

    Ok(())
}
