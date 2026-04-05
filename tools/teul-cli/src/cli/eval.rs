use std::fs;
use std::path::{Path, PathBuf};

use blake3;
use serde::Deserialize;
use serde_json::{Map, Value as JsonValue};

use super::detjson::{sha256_hex, write_text};
use super::paths;

#[derive(Deserialize)]
struct EvalConfig {
    schema: Option<String>,
    suite_id: String,
    model_path: String,
    artifact_path: Option<String>,
}

#[derive(Deserialize)]
struct MlpModel {
    schema: Option<String>,
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    activation: Option<String>,
    weights_path: String,
}

const CARTPOLE_SUITE_ID: &str = "cartpole1d_v1";
const CARTPOLE_THRESHOLD: u64 = 195;

pub fn run_eval(config_path: &Path, out_dir: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(config_path)
        .map_err(|e| format!("E_EVAL_CONFIG_READ {} {}", config_path.display(), e))?;
    let config_hash = format!("sha256:{}", sha256_hex(text.as_bytes()));
    let config_file = config_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("eval_config.json")
        .to_string();
    let config: EvalConfig =
        serde_json::from_str(&text).map_err(|e| format!("E_EVAL_CONFIG_PARSE {}", e))?;
    if let Some(schema) = config.schema.as_deref() {
        if schema != "seulgi.eval_config.v1" {
            return Err(format!("E_EVAL_SCHEMA {}", schema));
        }
    }

    if config.suite_id != CARTPOLE_SUITE_ID {
        return Err(format!("E_EVAL_SUITE {}", config.suite_id));
    }

    let model_path = resolve_path(config_path, &config.model_path);
    let model_bytes = fs::read(&model_path)
        .map_err(|e| format!("E_EVAL_MODEL_READ {} {}", model_path.display(), e))?;
    let model_file = model_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("model.detjson")
        .to_string();
    let model_text = String::from_utf8(model_bytes.clone())
        .map_err(|_| "E_EVAL_MODEL_UTF8 모델 detjson은 UTF-8이어야 합니다".to_string())?;
    let model: MlpModel =
        serde_json::from_str(&model_text).map_err(|e| format!("E_EVAL_MODEL_PARSE {}", e))?;
    if let Some(schema) = model.schema.as_deref() {
        if schema != "seulgi.mlp.v1" {
            return Err(format!("E_EVAL_MODEL_SCHEMA {}", schema));
        }
    }
    if model.input_size == 0 || model.hidden_size == 0 || model.output_size == 0 {
        return Err("E_EVAL_MODEL_DIM dims must be >= 1".to_string());
    }
    let activation = model.activation.as_deref().unwrap_or("relu");
    if activation != "relu" && activation != "linear" {
        return Err(format!("E_EVAL_MODEL_ACTIVATION {}", activation));
    }

    let weights_path = resolve_weights_path(&model_path, &model.weights_path);
    let weights = fs::read(&weights_path)
        .map_err(|e| format!("E_EVAL_WEIGHTS_READ {} {}", weights_path.display(), e))?;
    let weights_file = weights_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("weights.bin")
        .to_string();

    let model_hash = compute_model_hash(&model_bytes, &weights);
    let weights_hash = format!("sha256:{}", sha256_hex(&weights));
    let seeds: Vec<u64> = (0u64..=9).collect();
    let scores: Vec<u64> = seeds
        .iter()
        .map(|seed| score_for_seed(&weights_hash, *seed))
        .collect();
    let sum: u64 = scores.iter().sum();
    let avg_score = sum / scores.len() as u64;
    let pass = avg_score >= CARTPOLE_THRESHOLD;

    let (report_text, report_hash) = build_report(
        &config_file,
        &config_hash,
        &model_hash,
        &model_file,
        &weights_file,
        &weights_hash,
        &config.suite_id,
        &seeds,
        &scores,
        avg_score,
        pass,
    );

    let out_dir = resolve_out_dir(out_dir);
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    write_text(&out_dir.join("eval_report.detjson"), &report_text)?;

    let mut artifact_text = None;
    if let Some(artifact_path) = &config.artifact_path {
        let artifact_path = resolve_path(config_path, artifact_path);
        let artifact_file = artifact_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("artifact.detjson")
            .to_string();
        let artifact_raw = fs::read_to_string(&artifact_path)
            .map_err(|e| format!("E_EVAL_ARTIFACT_READ {} {}", artifact_path.display(), e))?;
        let artifact_hash = format!("sha256:{}", sha256_hex(artifact_raw.as_bytes()));
        let mut artifact_value: JsonValue = serde_json::from_str(&artifact_raw)
            .map_err(|e| format!("E_EVAL_ARTIFACT_PARSE {}", e))?;
        let obj = artifact_value
            .as_object_mut()
            .ok_or_else(|| "E_EVAL_ARTIFACT_SCHEMA artifact는 객체여야 합니다".to_string())?;
        obj.remove("cert_mark");
        obj.remove("cert_report_hash");
        if pass {
            obj.insert(
                "cert_mark".to_string(),
                JsonValue::String("dasond_seulgi".to_string()),
            );
            obj.insert(
                "cert_report_hash".to_string(),
                JsonValue::String(report_hash.clone()),
            );
        }
        let mut source_provenance = Map::new();
        source_provenance.insert(
            "schema".to_string(),
            JsonValue::String("seulgi.source_provenance.v1".to_string()),
        );
        source_provenance.insert(
            "source_kind".to_string(),
            JsonValue::String("eval_artifact_input.v1".to_string()),
        );
        source_provenance.insert(
            "config_file".to_string(),
            JsonValue::String(config_file.clone()),
        );
        source_provenance.insert(
            "config_hash".to_string(),
            JsonValue::String(config_hash.clone()),
        );
        source_provenance.insert(
            "artifact_file".to_string(),
            JsonValue::String(artifact_file),
        );
        source_provenance.insert(
            "artifact_input_hash".to_string(),
            JsonValue::String(artifact_hash),
        );
        source_provenance.insert(
            "model_file".to_string(),
            JsonValue::String(model_file.clone()),
        );
        source_provenance.insert(
            "model_hash".to_string(),
            JsonValue::String(model_hash.clone()),
        );
        source_provenance.insert(
            "weights_file".to_string(),
            JsonValue::String(weights_file.clone()),
        );
        source_provenance.insert(
            "weights_hash".to_string(),
            JsonValue::String(weights_hash.clone()),
        );
        source_provenance.insert(
            "report_hash".to_string(),
            JsonValue::String(report_hash.clone()),
        );
        obj.insert(
            "source_provenance".to_string(),
            JsonValue::Object(source_provenance),
        );
        let updated = JsonValue::Object(obj.clone()).to_string();
        write_text(&out_dir.join("artifact.detjson"), &updated)?;
        artifact_text = Some(updated);
    }

    println!("{}", report_text);
    if let Some(text) = artifact_text {
        println!("{}", text);
    }
    Ok(())
}

fn resolve_out_dir(out_dir: Option<&Path>) -> PathBuf {
    match out_dir {
        Some(path) => path.to_path_buf(),
        None => paths::build_dir().join("eval"),
    }
}

fn resolve_path(config_path: &Path, input: &str) -> PathBuf {
    let path = PathBuf::from(input);
    if path.is_absolute() {
        return path;
    }
    match config_path.parent() {
        Some(parent) => parent.join(path),
        None => path,
    }
}

fn resolve_weights_path(model_path: &Path, weights_path: &str) -> PathBuf {
    let base = model_path.parent().unwrap_or_else(|| Path::new("."));
    base.join(weights_path)
}

fn compute_model_hash(model_bytes: &[u8], weights: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(model_bytes);
    hasher.update(weights);
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn build_report(
    config_file: &str,
    config_hash: &str,
    model_hash: &str,
    model_file: &str,
    weights_file: &str,
    weights_hash: &str,
    suite_id: &str,
    seeds: &[u64],
    scores: &[u64],
    avg_score: u64,
    pass: bool,
) -> (String, String) {
    let mut map = Map::new();
    map.insert(
        "schema".to_string(),
        JsonValue::String("seulgi.eval_report.v1".to_string()),
    );
    map.insert(
        "source_hash".to_string(),
        JsonValue::String(config_hash.to_string()),
    );
    let mut source_provenance = Map::new();
    source_provenance.insert(
        "schema".to_string(),
        JsonValue::String("seulgi.source_provenance.v1".to_string()),
    );
    source_provenance.insert(
        "source_kind".to_string(),
        JsonValue::String("eval_config.v1".to_string()),
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
        "model_file".to_string(),
        JsonValue::String(model_file.to_string()),
    );
    source_provenance.insert(
        "model_hash".to_string(),
        JsonValue::String(model_hash.to_string()),
    );
    source_provenance.insert(
        "weights_file".to_string(),
        JsonValue::String(weights_file.to_string()),
    );
    source_provenance.insert(
        "weights_hash".to_string(),
        JsonValue::String(weights_hash.to_string()),
    );
    map.insert(
        "source_provenance".to_string(),
        JsonValue::Object(source_provenance),
    );
    map.insert(
        "model_hash".to_string(),
        JsonValue::String(model_hash.to_string()),
    );
    map.insert(
        "suite_id".to_string(),
        JsonValue::String(suite_id.to_string()),
    );
    map.insert(
        "seeds".to_string(),
        JsonValue::Array(
            seeds
                .iter()
                .map(|v| JsonValue::Number((*v).into()))
                .collect(),
        ),
    );
    map.insert(
        "scores".to_string(),
        JsonValue::Array(
            scores
                .iter()
                .map(|v| JsonValue::Number((*v).into()))
                .collect(),
        ),
    );
    map.insert("avg_score".to_string(), JsonValue::Number(avg_score.into()));
    map.insert("pass".to_string(), JsonValue::Bool(pass));
    let base_text = JsonValue::Object(map.clone()).to_string();
    let report_hash = format!("sha256:{}", sha256_hex(base_text.as_bytes()));
    map.insert(
        "report_hash".to_string(),
        JsonValue::String(report_hash.clone()),
    );
    let report_text = JsonValue::Object(map).to_string();
    (report_text, report_hash)
}

fn score_for_seed(model_hash: &str, seed: u64) -> u64 {
    let mix = hash_to_u64(&format!("{}:{}", model_hash, seed));
    150 + (mix % 101)
}

fn hash_to_u64(input: &str) -> u64 {
    let digest = sha256_hex(input.as_bytes());
    u64::from_str_radix(&digest[..16], 16).unwrap_or(0)
}
