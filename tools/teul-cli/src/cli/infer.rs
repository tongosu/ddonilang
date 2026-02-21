use std::fs;
use std::path::{Path, PathBuf};

use blake3;
use serde::Deserialize;

use super::detjson::write_text;
use super::paths;

#[derive(Deserialize)]
struct MlpModel {
    schema: Option<String>,
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    activation: Option<String>,
    weights_path: String,
}

#[derive(Deserialize)]
struct InferInput {
    schema: Option<String>,
    input: Vec<i64>,
}

pub fn run_mlp(model_path: &Path, input_path: &Path, out_dir: Option<&Path>) -> Result<(), String> {
    let model_bytes = fs::read(model_path)
        .map_err(|e| format!("E_INFER_MODEL_READ {} {}", model_path.display(), e))?;
    let model_text = String::from_utf8(model_bytes.clone())
        .map_err(|_| "E_INFER_MODEL_UTF8 모델 detjson은 UTF-8이어야 합니다".to_string())?;
    let model: MlpModel =
        serde_json::from_str(&model_text).map_err(|e| format!("E_INFER_MODEL_PARSE {}", e))?;
    if let Some(schema) = model.schema.as_deref() {
        if schema != "seulgi.mlp.v1" {
            return Err(format!("E_INFER_SCHEMA {}", schema));
        }
    }
    if model.input_size == 0 || model.hidden_size == 0 || model.output_size == 0 {
        return Err("E_INFER_DIM dims must be >= 1".to_string());
    }
    let activation = model.activation.as_deref().unwrap_or("relu");
    if activation != "relu" && activation != "linear" {
        return Err(format!("E_INFER_ACTIVATION {}", activation));
    }

    let weights_path = resolve_weights_path(model_path, &model.weights_path);
    let weights = fs::read(&weights_path)
        .map_err(|e| format!("E_INFER_WEIGHTS_READ {} {}", weights_path.display(), e))?;

    let input_text = fs::read_to_string(input_path)
        .map_err(|e| format!("E_INFER_INPUT_READ {} {}", input_path.display(), e))?;
    let input: InferInput =
        serde_json::from_str(&input_text).map_err(|e| format!("E_INFER_INPUT_PARSE {}", e))?;
    if let Some(schema) = input.schema.as_deref() {
        if schema != "seulgi.infer_input.v1" {
            return Err(format!("E_INFER_INPUT_SCHEMA {}", schema));
        }
    }
    if input.input.len() != model.input_size {
        return Err(format!(
            "E_INFER_INPUT_LEN input_size {} != {}",
            input.input.len(),
            model.input_size
        ));
    }

    let weights_i16 = parse_weights_i16(&weights, model.expected_weight_count())?;
    let output = run_inference(&model, activation, &input.input, &weights_i16);
    let model_hash = compute_model_hash(&model_bytes, &weights);
    let output_detjson = build_output(&model_hash, &output);

    let out_dir = resolve_out_dir(out_dir);
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    write_text(&out_dir.join("infer.output.detjson"), &output_detjson)?;

    println!("{}", output_detjson);
    Ok(())
}

impl MlpModel {
    fn expected_weight_count(&self) -> usize {
        self.hidden_size * self.input_size
            + self.hidden_size
            + self.output_size * self.hidden_size
            + self.output_size
    }
}

fn resolve_out_dir(out_dir: Option<&Path>) -> PathBuf {
    match out_dir {
        Some(path) => path.to_path_buf(),
        None => paths::build_dir().join("infer"),
    }
}

fn resolve_weights_path(model_path: &Path, weights_path: &str) -> PathBuf {
    let base = model_path.parent().unwrap_or_else(|| Path::new("."));
    base.join(weights_path)
}

fn parse_weights_i16(bytes: &[u8], expected: usize) -> Result<Vec<i16>, String> {
    if bytes.len() % 2 != 0 {
        return Err("E_INFER_WEIGHTS_LEN weights byte length must be even".to_string());
    }
    let count = bytes.len() / 2;
    if count != expected {
        return Err(format!(
            "E_INFER_WEIGHTS_COUNT expected {} got {}",
            expected, count
        ));
    }
    let mut out = Vec::with_capacity(count);
    for idx in 0..count {
        let lo = bytes[idx * 2];
        let hi = bytes[idx * 2 + 1];
        let value = i16::from_le_bytes([lo, hi]);
        out.push(value);
    }
    Ok(out)
}

fn run_inference(model: &MlpModel, activation: &str, input: &[i64], weights: &[i16]) -> Vec<i64> {
    let mut offset = 0usize;
    let mut hidden = vec![0i64; model.hidden_size];

    for h in 0..model.hidden_size {
        let mut acc: i64 = 0;
        for i in 0..model.input_size {
            let w = weights[offset] as i64;
            offset += 1;
            acc = acc.saturating_add(input[i].saturating_mul(w));
        }
        let bias = weights[offset] as i64;
        offset += 1;
        acc = acc.saturating_add(bias);
        hidden[h] = if activation == "relu" && acc < 0 {
            0
        } else {
            acc
        };
    }

    let mut output = vec![0i64; model.output_size];
    for o in 0..model.output_size {
        let mut acc: i64 = 0;
        for h in 0..model.hidden_size {
            let w = weights[offset] as i64;
            offset += 1;
            acc = acc.saturating_add(hidden[h].saturating_mul(w));
        }
        let bias = weights[offset] as i64;
        offset += 1;
        output[o] = acc.saturating_add(bias);
    }
    output
}

fn compute_model_hash(model_bytes: &[u8], weights: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(model_bytes);
    hasher.update(weights);
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn build_output(model_hash: &str, output: &[i64]) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"seulgi.infer_output.v1\",\"model_hash\":\"");
    out.push_str(model_hash);
    out.push_str("\",\"output\":[");
    for (idx, value) in output.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&value.to_string());
    }
    out.push_str("]}");
    out
}
