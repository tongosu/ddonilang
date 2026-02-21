use std::fs;
use std::path::{Path, PathBuf};

use blake3;
use serde::Deserialize;
use serde_json::{Map, Value as JsonValue};

use super::detjson::{sha256_hex, write_text};
use super::paths;

#[derive(Deserialize)]
struct BundleManifest {
    schema: Option<String>,
    ssot_version: Option<String>,
    ssot_bundle_hash: Option<String>,
    toolchain_version: Option<String>,
    model_hash: Option<String>,
    artifact_hash: Option<String>,
    eval_report_hash: Option<String>,
    cert_mark: Option<String>,
}

#[derive(Deserialize)]
struct MlpModel {
    schema: Option<String>,
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    activation: Option<String>,
    weights_path: Option<String>,
}

#[derive(Deserialize)]
struct BundleInputs {
    schema: Option<String>,
    inputs: Vec<Vec<i64>>,
}

pub fn run_parity(
    bundle_in: &Path,
    inputs_path: &Path,
    out_dir: Option<&Path>,
    wasm_hash_path: Option<&Path>,
) -> Result<(), String> {
    let manifest = read_manifest(bundle_in)?;
    validate_manifest(&manifest)?;

    let model_path = bundle_in.join("model_mlp_v1.detjson");
    let model_bytes = fs::read(&model_path)
        .map_err(|e| format!("E_BUNDLE_MODEL_READ {} {}", model_path.display(), e))?;
    let model_text = String::from_utf8(model_bytes.clone())
        .map_err(|_| "E_BUNDLE_MODEL_UTF8 모델 detjson은 UTF-8이어야 합니다".to_string())?;
    let model: MlpModel =
        serde_json::from_str(&model_text).map_err(|e| format!("E_BUNDLE_MODEL_PARSE {}", e))?;
    if let Some(schema) = model.schema.as_deref() {
        if schema != "seulgi.mlp.v1" {
            return Err(format!("E_BUNDLE_MODEL_SCHEMA {}", schema));
        }
    }
    if model.input_size == 0 || model.hidden_size == 0 || model.output_size == 0 {
        return Err("E_BUNDLE_MODEL_DIM dims must be >= 1".to_string());
    }
    let activation = model.activation.as_deref().unwrap_or("relu");
    if activation != "relu" && activation != "linear" {
        return Err(format!("E_BUNDLE_MODEL_ACTIVATION {}", activation));
    }

    let weights_name = model
        .weights_path
        .clone()
        .unwrap_or_else(|| "weights_v1.detbin".to_string());
    let weights_path = bundle_in.join(weights_name);
    let weights = fs::read(&weights_path)
        .map_err(|e| format!("E_BUNDLE_WEIGHTS_READ {} {}", weights_path.display(), e))?;

    let model_hash = compute_model_hash(&model_bytes, &weights);
    if let Some(expected) = &manifest.model_hash {
        if expected != &model_hash {
            return Err(format!(
                "E_BUNDLE_MODEL_HASH expected={} got={}",
                expected, model_hash
            ));
        }
    }

    let eval_report_path = bundle_in.join("eval_report.detjson");
    if eval_report_path.exists() {
        let eval_report = fs::read_to_string(&eval_report_path).map_err(|e| {
            format!(
                "E_BUNDLE_EVAL_REPORT_READ {} {}",
                eval_report_path.display(),
                e
            )
        })?;
        let eval_hash = format!("sha256:{}", sha256_hex(eval_report.as_bytes()));
        if let Some(expected) = &manifest.eval_report_hash {
            if expected != &eval_hash {
                return Err(format!(
                    "E_BUNDLE_EVAL_REPORT_HASH expected={} got={}",
                    expected, eval_hash
                ));
            }
        }
    }

    let inputs_text = fs::read_to_string(inputs_path)
        .map_err(|e| format!("E_BUNDLE_INPUT_READ {} {}", inputs_path.display(), e))?;
    let inputs: BundleInputs =
        serde_json::from_str(&inputs_text).map_err(|e| format!("E_BUNDLE_INPUT_PARSE {}", e))?;
    if let Some(schema) = inputs.schema.as_deref() {
        if schema != "seulgi.bundle_inputs.v1" {
            return Err(format!("E_BUNDLE_INPUT_SCHEMA {}", schema));
        }
    }
    if inputs.inputs.is_empty() {
        return Err("E_BUNDLE_INPUT_EMPTY inputs가 비어 있습니다".to_string());
    }

    let weights_i16 = parse_weights_i16(&weights, model.expected_weight_count())?;
    let mut outputs: Vec<Vec<i64>> = Vec::with_capacity(inputs.inputs.len());
    for (idx, input) in inputs.inputs.iter().enumerate() {
        if input.len() != model.input_size {
            return Err(format!(
                "E_BUNDLE_INPUT_LEN input[{}] size {} != {}",
                idx,
                input.len(),
                model.input_size
            ));
        }
        outputs.push(run_inference(&model, activation, input, &weights_i16));
    }

    let outputs_text = build_outputs(&model_hash, &outputs);
    let outputs_hash = format!("sha256:{}", sha256_hex(outputs_text.as_bytes()));

    if let Some(path) = wasm_hash_path {
        let wasm_hash = read_hash_file(path)?;
        if wasm_hash != outputs_hash {
            return Err(format!(
                "E_BUNDLE_WASM_HASH expected={} got={}",
                wasm_hash, outputs_hash
            ));
        }
    }

    let out_dir = resolve_out_dir(out_dir);
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    write_text(&out_dir.join("outputs.detjson"), &outputs_text)?;
    write_text(
        &out_dir.join("outputs_hash.txt"),
        &format!("{}\n", outputs_hash),
    )?;

    println!("{}", outputs_text);
    println!("outputs_hash={}", outputs_hash);
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

fn read_manifest(bundle_in: &Path) -> Result<BundleManifest, String> {
    let path = bundle_in.join("manifest.detjson");
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("E_BUNDLE_MANIFEST_READ {} {}", path.display(), e))?;
    let manifest: BundleManifest =
        serde_json::from_str(&text).map_err(|e| format!("E_BUNDLE_MANIFEST_PARSE {}", e))?;
    if let Some(schema) = manifest.schema.as_deref() {
        if schema != "seulgi.bundle_manifest.v1" {
            return Err(format!("E_BUNDLE_MANIFEST_SCHEMA {}", schema));
        }
    }
    Ok(manifest)
}

fn validate_manifest(manifest: &BundleManifest) -> Result<(), String> {
    let missing = [
        ("ssot_version", manifest.ssot_version.as_ref()),
        ("ssot_bundle_hash", manifest.ssot_bundle_hash.as_ref()),
        ("toolchain_version", manifest.toolchain_version.as_ref()),
        ("model_hash", manifest.model_hash.as_ref()),
        ("artifact_hash", manifest.artifact_hash.as_ref()),
        ("eval_report_hash", manifest.eval_report_hash.as_ref()),
    ]
    .iter()
    .filter_map(|(key, value)| if value.is_none() { Some(*key) } else { None })
    .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!("E_BUNDLE_MANIFEST_FIELDS {}", missing.join(",")));
    }
    if let Some(mark) = manifest.cert_mark.as_deref() {
        if mark != "dasond_seulgi" {
            return Err(format!("E_BUNDLE_CERT_MARK {}", mark));
        }
    }
    Ok(())
}

fn resolve_out_dir(out_dir: Option<&Path>) -> PathBuf {
    match out_dir {
        Some(path) => path.to_path_buf(),
        None => paths::build_dir().join("bundle_parity"),
    }
}

fn read_hash_file(path: &Path) -> Result<String, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("E_BUNDLE_WASM_HASH_READ {} {}", path.display(), e))?;
    let mut value = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("outputs_hash=") {
            value = Some(rest.trim().to_string());
            break;
        }
        if trimmed.starts_with("sha256:") {
            value = Some(trimmed.to_string());
            break;
        }
        value = Some(trimmed.to_string());
        break;
    }
    let Some(value) = value else {
        return Err(format!("E_BUNDLE_WASM_HASH_EMPTY {}", path.display()));
    };
    if !value.starts_with("sha256:") {
        return Err(format!(
            "E_BUNDLE_WASM_HASH_FORMAT {} {}",
            path.display(),
            value
        ));
    }
    Ok(value)
}

fn compute_model_hash(model_bytes: &[u8], weights: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(model_bytes);
    hasher.update(weights);
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn parse_weights_i16(bytes: &[u8], expected: usize) -> Result<Vec<i16>, String> {
    if bytes.len() % 2 != 0 {
        return Err("E_BUNDLE_WEIGHTS_LEN weights byte length must be even".to_string());
    }
    let count = bytes.len() / 2;
    if count != expected {
        return Err(format!(
            "E_BUNDLE_WEIGHTS_COUNT expected {} got {}",
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

fn build_outputs(model_hash: &str, outputs: &[Vec<i64>]) -> String {
    let mut map = Map::new();
    map.insert(
        "schema".to_string(),
        JsonValue::String("seulgi.bundle_outputs.v1".to_string()),
    );
    map.insert(
        "model_hash".to_string(),
        JsonValue::String(model_hash.to_string()),
    );
    let items = outputs
        .iter()
        .map(|row| JsonValue::Array(row.iter().map(|v| JsonValue::Number((*v).into())).collect()))
        .collect::<Vec<_>>();
    map.insert("outputs".to_string(), JsonValue::Array(items));
    JsonValue::Object(map).to_string()
}
