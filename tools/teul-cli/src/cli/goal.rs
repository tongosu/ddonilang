use std::fs;
use std::path::Path;

use ddonirang_core::seulgi::goal::{parse_dorok, target_state_detjson};
use ddonirang_core::seulgi::goap::{pick_best_action, ActionNode};
use serde_json::Value;

use super::detjson::write_text;

pub fn run_parse(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let raw = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let target = parse_dorok(&raw);
    let detjson = target_state_detjson(&target);
    if let Some(path) = out {
        write_text(path, &format!("{}\n", detjson))?;
    } else {
        println!("{}", detjson);
    }
    Ok(())
}

pub fn run_plan(actions: &Path, out: Option<&Path>) -> Result<(), String> {
    let raw = fs::read_to_string(actions).map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let list = value
        .get("actions")
        .ok_or_else(|| "E_GOAP_ACTIONS actions 필드 없음".to_string())?;
    let mut actions_vec = Vec::new();
    if let Value::Array(items) = list {
        for item in items {
            let action_id = item
                .get("action_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "E_GOAP_ACTION action_id 없음".to_string())?
                .to_string();
            let total_cost = item
                .get("total_cost")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| "E_GOAP_ACTION total_cost 없음".to_string())?;
            let steps = item
                .get("steps")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "E_GOAP_ACTION steps 없음".to_string())?;
            actions_vec.push(ActionNode {
                action_id,
                total_cost,
                steps: steps as u32,
            });
        }
    }

    let best = pick_best_action(actions_vec);
    let detjson = match best {
        Some(action) => {
            format!(
                "{{\"schema\":\"goap.plan.v1\",\"action_id\":\"{}\",\"total_cost\":{},\"steps\":{}}}",
                escape_json(&action.action_id),
                action.total_cost,
                action.steps
            )
        }
        None => "{\"schema\":\"goap.plan.v1\",\"action_id\":null}".to_string(),
    };

    if let Some(path) = out {
        write_text(path, &format!("{}\n", detjson))?;
    } else {
        println!("{}", detjson);
    }
    Ok(())
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
