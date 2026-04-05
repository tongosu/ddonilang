use serde_json::{Map, Number, Value as JsonValue};
use std::fs;
use std::path::Path;

use super::detjson::{sha256_hex, write_text};

pub fn run_validate(input: &Path) -> Result<(), String> {
    let (normalized, canonical_text) = load_and_normalize(input)?;
    let hash = format!("sha256:{}", sha256_hex(canonical_text.as_bytes()));
    println!("tensor_validate=ok");
    println!("tensor_kind={}", normalized.kind);
    println!("tensor_shape={}", shape_to_text(&normalized.shape));
    println!("tensor_hash={}", hash);
    Ok(())
}

pub fn run_hash(input: &Path) -> Result<(), String> {
    let (normalized, canonical_text) = load_and_normalize(input)?;
    let hash = format!("sha256:{}", sha256_hex(canonical_text.as_bytes()));
    println!("tensor_kind={}", normalized.kind);
    println!("tensor_shape={}", shape_to_text(&normalized.shape));
    println!("tensor_hash={}", hash);
    Ok(())
}

pub fn run_canon(input: &Path, out: &Path) -> Result<(), String> {
    let (normalized, canonical_text) = load_and_normalize(input)?;
    let hash = format!("sha256:{}", sha256_hex(canonical_text.as_bytes()));
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("E_TENSOR_V0_OUT_DIR_CREATE {} ({})", parent.display(), e))?;
    }
    write_text(out, &(canonical_text + "\n"))
        .map_err(|e| format!("E_TENSOR_V0_OUT_WRITE {} ({})", out.display(), e))?;
    println!("tensor_out={}", out.display());
    println!("tensor_kind={}", normalized.kind);
    println!("tensor_shape={}", shape_to_text(&normalized.shape));
    println!("tensor_hash={}", hash);
    Ok(())
}

#[derive(Debug)]
struct TensorNormalized {
    kind: String,
    shape: Vec<u64>,
    canonical: JsonValue,
}

fn load_and_normalize(input: &Path) -> Result<(TensorNormalized, String), String> {
    let source = fs::read_to_string(input)
        .map_err(|e| format!("E_TENSOR_V0_INPUT_READ {} ({})", input.display(), e))?;
    let parsed: JsonValue = serde_json::from_str(&source)
        .map_err(|e| format!("E_TENSOR_V0_INPUT_PARSE {} ({})", input.display(), e))?;
    let normalized = normalize_tensor_v0(parsed)?;
    let canonical_text = canonical_json_text(&normalized.canonical)?;
    Ok((normalized, canonical_text))
}

fn normalize_tensor_v0(value: JsonValue) -> Result<TensorNormalized, String> {
    let root = value
        .as_object()
        .ok_or_else(|| "E_TENSOR_V0_ROOT_TYPE".to_string())?;
    let schema = root
        .get("schema")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if schema.trim() != "tensor.v0" {
        return Err("E_TENSOR_V0_SCHEMA".to_string());
    }
    let kind = root
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if kind != "dense" && kind != "sparse" {
        return Err("E_TENSOR_V0_KIND".to_string());
    }
    let dtype = root
        .get("dtype")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "E_TENSOR_V0_DTYPE".to_string())?
        .to_string();
    let shape = parse_shape(root.get("shape"))?;
    let mut canonical = Map::new();
    canonical.insert(
        "schema".to_string(),
        JsonValue::String("tensor.v0".to_string()),
    );
    canonical.insert("kind".to_string(), JsonValue::String(kind.to_string()));
    canonical.insert("dtype".to_string(), JsonValue::String(dtype));
    canonical.insert(
        "shape".to_string(),
        JsonValue::Array(
            shape
                .iter()
                .map(|value| JsonValue::Number(Number::from(*value)))
                .collect(),
        ),
    );
    if kind == "dense" {
        let data = parse_dense_data(root, &shape)?;
        canonical.insert("data".to_string(), JsonValue::Array(data));
    } else {
        let items = parse_sparse_items(root, &shape)?;
        canonical.insert("items".to_string(), JsonValue::Array(items));
    }
    Ok(TensorNormalized {
        kind: kind.to_string(),
        shape,
        canonical: JsonValue::Object(canonical),
    })
}

fn parse_shape(value: Option<&JsonValue>) -> Result<Vec<u64>, String> {
    let shape = value
        .and_then(|v| v.as_array())
        .ok_or_else(|| "E_TENSOR_V0_SHAPE".to_string())?;
    if shape.is_empty() {
        return Err("E_TENSOR_V0_SHAPE".to_string());
    }
    let mut out = Vec::with_capacity(shape.len());
    for dim in shape {
        let raw = dim
            .as_u64()
            .ok_or_else(|| "E_TENSOR_V0_SHAPE".to_string())?;
        if raw == 0 {
            return Err("E_TENSOR_V0_SHAPE".to_string());
        }
        out.push(raw);
    }
    Ok(out)
}

fn parse_dense_data(
    root: &Map<String, JsonValue>,
    shape: &[u64],
) -> Result<Vec<JsonValue>, String> {
    let data = root
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "E_TENSOR_V0_DENSE_DATA_TYPE".to_string())?;
    let expected_len = tensor_item_count(shape)?;
    if data.len() != expected_len {
        return Err("E_TENSOR_V0_DENSE_DATA_LEN".to_string());
    }
    let mut out = Vec::with_capacity(data.len());
    for value in data {
        let raw = value
            .as_str()
            .ok_or_else(|| "E_TENSOR_V0_DENSE_VALUE_TYPE".to_string())?;
        if !is_i64_raw(raw) {
            return Err("E_TENSOR_V0_DENSE_VALUE_TYPE".to_string());
        }
        out.push(JsonValue::String(raw.to_string()));
    }
    Ok(out)
}

fn parse_sparse_items(
    root: &Map<String, JsonValue>,
    shape: &[u64],
) -> Result<Vec<JsonValue>, String> {
    let items = root
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "E_TENSOR_V0_SPARSE_ITEMS_TYPE".to_string())?;
    let rank = shape.len();
    let mut out: Vec<JsonValue> = Vec::with_capacity(items.len());
    let mut previous: Option<Vec<u64>> = None;
    for item in items {
        let row = item
            .as_array()
            .ok_or_else(|| "E_TENSOR_V0_SPARSE_ITEM_TYPE".to_string())?;
        if row.len() != 2 {
            return Err("E_TENSOR_V0_SPARSE_ITEM_TYPE".to_string());
        }
        let index = row[0]
            .as_array()
            .ok_or_else(|| "E_TENSOR_V0_SPARSE_INDEX_TYPE".to_string())?;
        if index.len() != rank {
            return Err("E_TENSOR_V0_SPARSE_INDEX_RANK".to_string());
        }
        let mut point: Vec<u64> = Vec::with_capacity(rank);
        for (axis, value) in index.iter().enumerate() {
            let coord = value
                .as_u64()
                .ok_or_else(|| "E_TENSOR_V0_SPARSE_INDEX_TYPE".to_string())?;
            if coord >= shape[axis] {
                return Err("E_TENSOR_V0_SPARSE_INDEX_OOB".to_string());
            }
            point.push(coord);
        }
        let raw = row
            .get(1)
            .and_then(|v| v.as_str())
            .ok_or_else(|| "E_TENSOR_V0_SPARSE_VALUE_TYPE".to_string())?;
        if !is_i64_raw(raw) {
            return Err("E_TENSOR_V0_SPARSE_VALUE_TYPE".to_string());
        }
        if let Some(prev) = previous.as_ref() {
            if *prev == point {
                return Err("E_TENSOR_V0_SPARSE_DUP".to_string());
            }
            if point < *prev {
                return Err("E_TENSOR_V0_SPARSE_ORDER".to_string());
            }
        }
        previous = Some(point.clone());
        out.push(JsonValue::Array(vec![
            JsonValue::Array(
                point
                    .iter()
                    .map(|value| JsonValue::Number(Number::from(*value)))
                    .collect(),
            ),
            JsonValue::String(raw.to_string()),
        ]));
    }
    Ok(out)
}

fn tensor_item_count(shape: &[u64]) -> Result<usize, String> {
    let mut total: u128 = 1;
    for dim in shape {
        total = total
            .checked_mul(*dim as u128)
            .ok_or_else(|| "E_TENSOR_V0_SHAPE".to_string())?;
        if total > usize::MAX as u128 {
            return Err("E_TENSOR_V0_SHAPE".to_string());
        }
    }
    Ok(total as usize)
}

fn is_i64_raw(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let text = value.as_bytes();
    let (start, rest) = if text[0] == b'-' {
        if text.len() == 1 {
            return false;
        }
        (1usize, &text[1..])
    } else {
        (0usize, text)
    };
    if start == 0 && value == "+" {
        return false;
    }
    rest.iter().all(|ch| ch.is_ascii_digit())
}

fn canonical_json_text(value: &JsonValue) -> Result<String, String> {
    let normalized = canonicalize_json(value);
    serde_json::to_string(&normalized).map_err(|e| format!("E_TENSOR_V0_CANON_JSON {}", e))
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

fn shape_to_text(shape: &[u64]) -> String {
    shape
        .iter()
        .map(|dim| dim.to_string())
        .collect::<Vec<_>>()
        .join("x")
}
