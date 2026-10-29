use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::Deserialize;
use serde_json::Value as JsonValue;

use sha2::{Digest, Sha256};

use crate::artifact_output::{write_artifact_set_atomic, ArtifactPayload};

const TRAINING_SSOT_VERSION: &str = "20.6.6";

#[derive(Deserialize)]
struct TrainConfig {
    schema: Option<String>,
    model_id: String,
    dataset_hash: String,
    recipe_hash: String,
    ssot_bundle_hash: String,
    train_seed: u64,
    target_score: u64,
    max_epochs: u64,
    weights_len: Option<usize>,
}

struct TrainOutcome {
    epoch: u64,
    score: u64,
    pass: bool,
}

pub fn run_train(config_path: &Path, out_dir: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(config_path)
        .map_err(|e| format!("E_TRAIN_CONFIG_READ {} {}", config_path.display(), e))?;
    let value: JsonValue =
        serde_json::from_str(&text).map_err(|e| format!("E_TRAIN_CONFIG_PARSE {}", e))?;
    let config_hash = format!("sha256:{}", sha256_hex(value.to_string().as_bytes()));
    let config_file = config_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("train_config.json")
        .to_string();

    let config: TrainConfig =
        serde_json::from_value(value).map_err(|e| format!("E_TRAIN_CONFIG_SCHEMA {}", e))?;
    if let Some(schema) = config.schema.as_deref() {
        if schema != "seulgi.train_config.v1" {
            return Err(format!("E_TRAIN_SCHEMA {}", schema));
        }
    }

    if config.max_epochs == 0 {
        return Err("E_TRAIN_MAX_EPOCHS max_epochs는 1 이상이어야 합니다".to_string());
    }

    let run_id = build_run_id(&config_hash, config.train_seed);
    let mix_seed =
        config.train_seed ^ hash_to_u64(&format!("{}:{}", config.dataset_hash, config.recipe_hash));
    let (base, step) = derive_curve(mix_seed);

    let outcome = run_toy_training(base, step, config.target_score, config.max_epochs);
    let report_detjson = build_train_report(
        &run_id,
        &outcome,
        config.target_score,
        &config_file,
        &config_hash,
        &config.dataset_hash,
        &config.recipe_hash,
        &config.ssot_bundle_hash,
    );
    let eval_report_hash = format!("sha256:{}", sha256_hex(report_detjson.as_bytes()));

    let mut artifact_detjson = None;
    let mut weights = Vec::new();
    if outcome.pass {
        let len = config.weights_len.unwrap_or(16);
        weights = build_weights(mix_seed, outcome.epoch, len);
        let weights_hash = format!("sha256:{}", sha256_hex(&weights));
        artifact_detjson = Some(build_artifact(
            &config.model_id,
            &run_id,
            &config_file,
            &config.ssot_bundle_hash,
            &config_hash,
            &eval_report_hash,
            &weights_hash,
            &config.dataset_hash,
            &config.recipe_hash,
        ));
    }

    let out_dir = resolve_out_dir(out_dir);
    let report_path = out_dir.join("train.report.detjson");
    let artifact_path = out_dir.join("artifact.detjson");
    let weights_path = out_dir.join("weights.bin");
    let mut outputs = vec![ArtifactPayload {
        target: &report_path,
        bytes: report_detjson.as_bytes(),
    }];
    if let Some(artifact) = artifact_detjson.as_deref() {
        outputs.push(ArtifactPayload {
            target: &artifact_path,
            bytes: artifact.as_bytes(),
        });
        outputs.push(ArtifactPayload {
            target: &weights_path,
            bytes: &weights,
        });
    }
    write_artifact_set_atomic(&outputs)?;

    println!("{}", report_detjson);
    if let Some(artifact) = artifact_detjson {
        println!("{}", artifact);
    }

    if !outcome.pass {
        return Err("E_TRAIN_THRESHOLD 임계값을 만족하지 못했습니다".to_string());
    }
    Ok(())
}

fn resolve_out_dir(out_dir: Option<&Path>) -> PathBuf {
    match out_dir {
        Some(path) => path.to_path_buf(),
        None => configured_build_dir().join("train"),
    }
}

fn configured_build_dir() -> PathBuf {
    if let Some(configured) = env::var_os("DDN_BUILD_DIR").filter(|value| !value.is_empty()) {
        return PathBuf::from(configured);
    }
    env::var_os("LOCALAPPDATA")
        .or_else(|| env::var_os("XDG_CACHE_HOME"))
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(env::temp_dir)
        .join("ddonirang")
        .join("build")
}

fn build_run_id(config_hash: &str, seed: u64) -> String {
    let input = format!("{}:{}", config_hash, seed);
    let digest = sha256_hex(input.as_bytes());
    format!("train_{}", &digest[..8])
}

fn hash_to_u64(input: &str) -> u64 {
    let digest = sha256_hex(input.as_bytes());
    u64::from_str_radix(&digest[..16], 16).unwrap_or(0)
}

fn derive_curve(seed: u64) -> (u64, u64) {
    let base = 100 + (seed % 200);
    let step = 5 + ((seed / 200) % 25);
    (base, step)
}

fn run_toy_training(base: u64, step: u64, target: u64, max_epochs: u64) -> TrainOutcome {
    let mut epoch = 1;
    let mut score = base;
    for idx in 1..=max_epochs {
        epoch = idx;
        score = base + step.saturating_mul(idx.saturating_sub(1));
        if score >= target {
            return TrainOutcome {
                epoch,
                score,
                pass: true,
            };
        }
    }
    TrainOutcome {
        epoch,
        score,
        pass: false,
    }
}

fn build_train_report(
    run_id: &str,
    outcome: &TrainOutcome,
    target: u64,
    config_file: &str,
    config_hash: &str,
    dataset_hash: &str,
    recipe_hash: &str,
    ssot_bundle_hash: &str,
) -> String {
    let mut map = serde_json::Map::new();
    map.insert(
        "schema".to_string(),
        JsonValue::String("seulgi.train_report.v1".to_string()),
    );
    map.insert(
        "source_hash".to_string(),
        JsonValue::String(config_hash.to_string()),
    );
    let mut source_provenance = serde_json::Map::new();
    source_provenance.insert(
        "schema".to_string(),
        JsonValue::String("seulgi.source_provenance.v1".to_string()),
    );
    source_provenance.insert(
        "source_kind".to_string(),
        JsonValue::String("train_config.v1".to_string()),
    );
    source_provenance.insert(
        "config_file".to_string(),
        JsonValue::String(config_file.to_string()),
    );
    source_provenance.insert(
        "config_hash".to_string(),
        JsonValue::String(config_hash.to_string()),
    );
    source_provenance.insert(
        "dataset_hash".to_string(),
        JsonValue::String(dataset_hash.to_string()),
    );
    source_provenance.insert(
        "recipe_hash".to_string(),
        JsonValue::String(recipe_hash.to_string()),
    );
    source_provenance.insert(
        "ssot_bundle_hash".to_string(),
        JsonValue::String(ssot_bundle_hash.to_string()),
    );
    map.insert(
        "source_provenance".to_string(),
        JsonValue::Object(source_provenance),
    );
    map.insert("run_id".to_string(), JsonValue::String(run_id.to_string()));
    map.insert("epoch".to_string(), JsonValue::Number(outcome.epoch.into()));
    map.insert("score".to_string(), JsonValue::Number(outcome.score.into()));
    map.insert("threshold".to_string(), JsonValue::Number(target.into()));
    map.insert("pass".to_string(), JsonValue::Bool(outcome.pass));
    JsonValue::Object(map).to_string()
}

fn build_artifact(
    model_id: &str,
    run_id: &str,
    config_file: &str,
    ssot_bundle_hash: &str,
    train_config_hash: &str,
    eval_report_hash: &str,
    weights_hash: &str,
    dataset_hash: &str,
    recipe_hash: &str,
) -> String {
    let mut map = serde_json::Map::new();
    map.insert(
        "schema".to_string(),
        JsonValue::String("seulgi.model_artifact.v1".to_string()),
    );
    map.insert(
        "model_id".to_string(),
        JsonValue::String(model_id.to_string()),
    );
    map.insert("run_id".to_string(), JsonValue::String(run_id.to_string()));
    map.insert(
        "ssot_version".to_string(),
        JsonValue::String(format!("v{}", TRAINING_SSOT_VERSION)),
    );
    map.insert(
        "ssot_bundle_hash".to_string(),
        JsonValue::String(ssot_bundle_hash.to_string()),
    );
    map.insert(
        "train_config_hash".to_string(),
        JsonValue::String(train_config_hash.to_string()),
    );
    map.insert(
        "eval_report_hash".to_string(),
        JsonValue::String(eval_report_hash.to_string()),
    );
    map.insert(
        "weights_hash".to_string(),
        JsonValue::String(weights_hash.to_string()),
    );
    let mut source_provenance = serde_json::Map::new();
    source_provenance.insert(
        "schema".to_string(),
        JsonValue::String("seulgi.source_provenance.v1".to_string()),
    );
    source_provenance.insert(
        "source_kind".to_string(),
        JsonValue::String("train_artifact.v1".to_string()),
    );
    source_provenance.insert(
        "config_file".to_string(),
        JsonValue::String(config_file.to_string()),
    );
    source_provenance.insert(
        "config_hash".to_string(),
        JsonValue::String(train_config_hash.to_string()),
    );
    source_provenance.insert(
        "train_report_hash".to_string(),
        JsonValue::String(eval_report_hash.to_string()),
    );
    source_provenance.insert(
        "dataset_hash".to_string(),
        JsonValue::String(dataset_hash.to_string()),
    );
    source_provenance.insert(
        "recipe_hash".to_string(),
        JsonValue::String(recipe_hash.to_string()),
    );
    map.insert(
        "source_provenance".to_string(),
        JsonValue::Object(source_provenance),
    );
    JsonValue::Object(map).to_string()
}

fn build_weights(seed: u64, epoch: u64, len: usize) -> Vec<u8> {
    let mut state = seed ^ epoch.wrapping_mul(0x9E3779B97F4A7C15);
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        out.push((state >> 32) as u8);
    }
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
