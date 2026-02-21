use std::fs;
use std::path::{Path, PathBuf};

use ddonirang_core::Fixed64;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use super::detjson::{sha256_hex, write_text};
use super::paths;

#[derive(Deserialize)]
struct ImitationConfig {
    schema: Option<String>,
    replay_path: String,
    train_seed: u64,
    max_epochs: u64,
    env_id: Option<String>,
}

struct ReplayStep {
    episode_id: u64,
    step_index: u64,
    observation: JsonValue,
    action: JsonValue,
    order: usize,
}

pub fn run_imitation(config_path: &Path, out_dir: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(config_path)
        .map_err(|e| format!("E_IMITATION_CONFIG_READ {} {}", config_path.display(), e))?;
    let config: ImitationConfig =
        serde_json::from_str(&text).map_err(|e| format!("E_IMITATION_CONFIG_PARSE {}", e))?;
    if let Some(schema) = config.schema.as_deref() {
        if schema != "seulgi.imitation_config.v1" {
            return Err(format!("E_IMITATION_SCHEMA {}", schema));
        }
    }

    if config.max_epochs == 0 {
        return Err("E_IMITATION_MAX_EPOCHS max_epochs는 1 이상이어야 합니다".to_string());
    }

    let replay_path = resolve_replay_path(config_path, &config.replay_path);
    let replay_text = fs::read_to_string(&replay_path)
        .map_err(|e| format!("E_IMITATION_REPLAY_READ {} {}", replay_path.display(), e))?;
    let (env_id, mut steps) = parse_replay_lines(&replay_text)?;

    if steps.is_empty() {
        return Err("E_IMITATION_EMPTY replay 입력에 step 기록이 없습니다".to_string());
    }

    let env_id = match (config.env_id.as_deref(), env_id.as_deref()) {
        (Some(config_env), Some(replay_env)) => {
            if config_env != replay_env {
                return Err(format!(
                    "E_IMITATION_ENV_MISMATCH config_env={} replay_env={}",
                    config_env, replay_env
                ));
            }
            config_env.to_string()
        }
        (Some(config_env), None) => config_env.to_string(),
        (None, Some(replay_env)) => replay_env.to_string(),
        (None, None) => "nurigym.unknown".to_string(),
    };

    steps.sort_by(|a, b| {
        (a.episode_id, a.step_index, a.order).cmp(&(b.episode_id, b.step_index, b.order))
    });

    let dataset_text = build_dataset_text(&env_id, &steps);
    let dataset_hash = format!("sha256:{}", sha256_hex(dataset_text.as_bytes()));

    let out_dir = resolve_out_dir(out_dir);
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    write_text(&out_dir.join("imitation.dataset.jsonl"), &dataset_text)?;
    write_text(
        &out_dir.join("dataset.sha256"),
        &format!("{}\n", dataset_hash),
    )?;

    let metrics = build_metrics_curve(
        &dataset_hash,
        config.train_seed,
        config.max_epochs,
        steps.len() as u64,
    )?;
    let metrics_text = metrics.join("\n") + "\n";
    write_text(&out_dir.join("metrics_curve.detjsonl"), &metrics_text)?;

    println!("dataset_hash={}", dataset_hash);
    for line in metrics {
        println!("{}", line);
    }
    Ok(())
}

fn resolve_out_dir(out_dir: Option<&Path>) -> PathBuf {
    match out_dir {
        Some(path) => path.to_path_buf(),
        None => paths::build_dir().join("imitation"),
    }
}

fn resolve_replay_path(config_path: &Path, replay_path: &str) -> PathBuf {
    let path = PathBuf::from(replay_path);
    if path.is_absolute() {
        return path;
    }
    match config_path.parent() {
        Some(parent) => parent.join(path),
        None => path,
    }
}

fn parse_replay_lines(text: &str) -> Result<(Option<String>, Vec<ReplayStep>), String> {
    let mut env_id: Option<String> = None;
    let mut steps: Vec<ReplayStep> = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: JsonValue = serde_json::from_str(trimmed)
            .map_err(|e| format!("E_IMITATION_REPLAY_PARSE {} {}", idx + 1, e))?;
        let schema = value.get("schema").and_then(|v| v.as_str()).unwrap_or("");
        match schema {
            "nurigym.episode.v1"
            | "imitation.episode.v1"
            | "nurigym.dataset.v1"
            | "imitation.dataset.v1" => {
                let line_env = value
                    .get("env_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| format!("E_IMITATION_REPLAY_ENV {} env_id 누락", idx + 1))?;
                note_env_id(&mut env_id, idx + 1, line_env)?;
            }
            "nurigym.step.v1" | "imitation.step.v1" => {
                let obj = value.as_object().ok_or_else(|| {
                    format!("E_IMITATION_REPLAY_STEP {} step이 객체가 아닙니다", idx + 1)
                })?;
                let episode_id = value_as_u64(obj.get("episode_id"), idx + 1, "episode_id")?;
                let step_index = match obj.get("step_index") {
                    Some(value) => value_as_u64(Some(value), idx + 1, "step_index")?,
                    None => value_as_u64(obj.get("madi"), idx + 1, "madi")?,
                };
                let observation = obj.get("observation").cloned().ok_or_else(|| {
                    format!("E_IMITATION_REPLAY_STEP {} observation 누락", idx + 1)
                })?;
                let action = obj
                    .get("action")
                    .cloned()
                    .ok_or_else(|| format!("E_IMITATION_REPLAY_STEP {} action 누락", idx + 1))?;

                steps.push(ReplayStep {
                    episode_id,
                    step_index,
                    observation,
                    action,
                    order: idx,
                });
            }
            _ => {
                return Err(format!(
                    "E_IMITATION_REPLAY_SCHEMA {} 지원하지 않는 schema: {}",
                    idx + 1,
                    schema
                ));
            }
        }
    }

    Ok((env_id, steps))
}

fn note_env_id(env_id: &mut Option<String>, line_no: usize, line_env: &str) -> Result<(), String> {
    match env_id.as_deref() {
        Some(existing) if existing != line_env => Err(format!(
            "E_IMITATION_REPLAY_ENV {} env_id 불일치: {} vs {}",
            line_no, existing, line_env
        )),
        Some(_) => Ok(()),
        None => {
            *env_id = Some(line_env.to_string());
            Ok(())
        }
    }
}

fn value_as_u64(value: Option<&JsonValue>, line_no: usize, field: &str) -> Result<u64, String> {
    let Some(value) = value else {
        return Err(format!(
            "E_IMITATION_REPLAY_STEP {} {} 누락",
            line_no, field
        ));
    };
    match value {
        JsonValue::Number(num) => num.as_u64().ok_or_else(|| {
            format!(
                "E_IMITATION_REPLAY_STEP {} {} 숫자 범위 오류",
                line_no, field
            )
        }),
        JsonValue::String(text) => text.trim().parse::<u64>().map_err(|_| {
            format!(
                "E_IMITATION_REPLAY_STEP {} {} 숫자 변환 실패",
                line_no, field
            )
        }),
        _ => Err(format!(
            "E_IMITATION_REPLAY_STEP {} {} 타입이 숫자가 아닙니다",
            line_no, field
        )),
    }
}

fn build_dataset_text(env_id: &str, steps: &[ReplayStep]) -> String {
    let mut lines = Vec::with_capacity(steps.len() + 1);
    lines.push(build_dataset_header(env_id, steps.len() as u64));
    for step in steps {
        lines.push(build_sample_line(step));
    }
    lines.join("\n") + "\n"
}

fn build_dataset_header(env_id: &str, count: u64) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"imitation.dataset.v1\",\"env_id\":\"");
    out.push_str(env_id);
    out.push_str("\",\"count\":");
    out.push_str(&count.to_string());
    out.push('}');
    out
}

fn build_sample_line(step: &ReplayStep) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"imitation.sample.v1\",\"episode_id\":");
    out.push_str(&step.episode_id.to_string());
    out.push_str(",\"step_index\":");
    out.push_str(&step.step_index.to_string());
    out.push_str(",\"observation\":");
    out.push_str(&step.observation.to_string());
    out.push_str(",\"action\":");
    out.push_str(&step.action.to_string());
    out.push('}');
    out
}

fn build_metrics_curve(
    dataset_hash: &str,
    train_seed: u64,
    max_epochs: u64,
    total: u64,
) -> Result<Vec<String>, String> {
    if total == 0 {
        return Err("E_IMITATION_EMPTY total=0".to_string());
    }

    let mix_seed = train_seed ^ hash_to_u64(dataset_hash);
    let base = (mix_seed % total).saturating_add(1);
    let mut step = base / max_epochs.max(1);
    if step == 0 {
        step = 1;
    }

    let mut out = Vec::with_capacity(max_epochs as usize);
    for epoch in 1..=max_epochs {
        let decay = step.saturating_mul(epoch.saturating_sub(1));
        let mismatch = if decay >= base { 0 } else { base - decay };
        let rate = fixed_ratio(mismatch, total)?;
        out.push(build_metric_line(epoch, mismatch, total, &rate.to_string()));
    }
    Ok(out)
}

fn build_metric_line(epoch: u64, mismatch: u64, total: u64, rate: &str) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"seulgi.imitation_metric.v1\",\"epoch\":");
    out.push_str(&epoch.to_string());
    out.push_str(",\"mismatch\":");
    out.push_str(&mismatch.to_string());
    out.push_str(",\"total\":");
    out.push_str(&total.to_string());
    out.push_str(",\"action_error_rate\":\"");
    out.push_str(rate);
    out.push_str("\"}");
    out
}

fn fixed_ratio(numer: u64, denom: u64) -> Result<Fixed64, String> {
    if denom == 0 {
        return Err("E_IMITATION_DIV_ZERO total=0".to_string());
    }
    let n = Fixed64::from_i64(numer as i64);
    let d = Fixed64::from_i64(denom as i64);
    n.try_div(d)
        .map_err(|_| "E_IMITATION_DIV_ZERO total=0".to_string())
}

fn hash_to_u64(input: &str) -> u64 {
    let digest = sha256_hex(input.as_bytes());
    u64::from_str_radix(&digest[..16], 16).unwrap_or(0)
}
