use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use blake3;
use serde_json::{Map, Value};

#[derive(Clone, Debug)]
struct Origin {
    file: String,
    line: usize,
    col: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FieldSchema {
    name: String,
    type_name: String,
    default: Option<String>,
}

#[derive(Clone, Debug)]
struct VariantSchema {
    tag: String,
    fields: Vec<FieldSchema>,
    origins: Vec<Origin>,
}

#[derive(Clone, Debug)]
struct VariantOutput {
    tag: String,
    fields: Vec<FieldSchema>,
    origins: Vec<Origin>,
    schema_hash: String,
    signal_id: u64,
}

pub fn run_registry(root: &Path, out_dir: &Path) -> Result<(), String> {
    let files = collect_ddn_files(root)?;
    let mut variants: HashMap<String, VariantSchema> = HashMap::new();
    let mut conflicts: Vec<(String, Vec<Origin>)> = Vec::new();

    for file in files {
        let text = fs::read_to_string(&file)
            .map_err(|e| format!("E_ALRIM_READ {} ({})", file.display(), e))?;
        let found = extract_variants(&text, &file)?;
        for mut variant in found {
            let entry = variants
                .entry(variant.tag.clone())
                .or_insert_with(|| VariantSchema {
                    tag: variant.tag.clone(),
                    fields: variant.fields.clone(),
                    origins: Vec::new(),
                });
            if entry.fields == variant.fields {
                entry.origins.append(&mut variant.origins);
            } else {
                let mut origins = entry.origins.clone();
                origins.extend(variant.origins.clone());
                conflicts.push((variant.tag.clone(), origins));
            }
        }
    }

    if !conflicts.is_empty() {
        let mut msg = String::from("E_ALRIM_VARIANT_CONFLICT 알림 태그 스키마 충돌\n");
        for (tag, origins) in conflicts {
            msg.push_str(&format!("- {}:\n", tag));
            for origin in origins {
                msg.push_str(&format!(
                    "  - {}:{}:{}\n",
                    origin.file, origin.line, origin.col
                ));
            }
        }
        return Err(msg);
    }

    let schema_version = "v1".to_string();
    let mut outputs: Vec<VariantOutput> = variants
        .into_values()
        .map(|mut v| {
            v.origins.sort_by(|a, b| {
                a.file
                    .cmp(&b.file)
                    .then_with(|| a.line.cmp(&b.line))
                    .then_with(|| a.col.cmp(&b.col))
            });
            let schema_hash = format!("blake3:{}", blake3::hash(&variant_detbin(&v)).to_hex());
            VariantOutput {
                tag: v.tag,
                fields: v.fields,
                origins: v.origins,
                schema_hash,
                signal_id: 0,
            }
        })
        .collect();

    outputs.sort_by(|a, b| {
        a.tag
            .as_bytes()
            .cmp(b.tag.as_bytes())
            .then_with(|| a.schema_hash.cmp(&b.schema_hash))
    });

    for (idx, item) in outputs.iter_mut().enumerate() {
        item.signal_id = idx as u64;
    }

    let registry_hash = format!(
        "blake3:{}",
        blake3::hash(&registry_detbin(&schema_version, &outputs)).to_hex()
    );

    let json_text = build_registry_json(&schema_version, &registry_hash, &outputs)?;

    fs::create_dir_all(out_dir).map_err(|e| format!("E_ALRIM_WRITE {}", e))?;
    let path = out_dir.join("alrim.registry.json");
    fs::write(&path, json_text).map_err(|e| format!("E_ALRIM_WRITE {}", e))?;
    println!("alrim_registry_written={}", path.display());
    println!("alrim_registry_hash={}", registry_hash);
    Ok(())
}

fn collect_ddn_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    visit_dir(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn visit_dir(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(root).map_err(|e| format!("E_ALRIM_SCAN {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("E_ALRIM_SCAN {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            visit_dir(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("ddn") {
            out.push(path);
        }
    }
    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".git"
            | "target"
            | "build"
            | "out"
            | "dist"
            | "docs"
            | "publish"
            | "node_modules"
            | ".cargo"
    )
}

fn extract_variants(source: &str, file: &Path) -> Result<Vec<VariantSchema>, String> {
    let mut variants = Vec::new();
    let mut idx = 0usize;
    let bytes = source.as_bytes();
    let mut in_string = false;
    let mut escape = false;
    while idx < bytes.len() {
        if in_string {
            let ch = source[idx..].chars().next().unwrap();
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            idx += ch.len_utf8();
            continue;
        }
        if bytes[idx] == b'/' && idx + 1 < bytes.len() && bytes[idx + 1] == b'/' {
            while idx < bytes.len() {
                let ch = source[idx..].chars().next().unwrap();
                idx += ch.len_utf8();
                if ch == '\n' {
                    break;
                }
            }
            continue;
        }
        if bytes[idx] == b'"' {
            in_string = true;
            idx += 1;
            continue;
        }
        if source[idx..].starts_with("알림목록:고름씨") {
            let mut cursor = idx + "알림목록:고름씨".len();
            while cursor < bytes.len() {
                let ch = source[cursor..].chars().next().unwrap();
                if !ch.is_whitespace() {
                    break;
                }
                cursor += ch.len_utf8();
            }
            if cursor >= bytes.len() || bytes[cursor] != b'=' {
                idx = cursor;
                continue;
            }
            cursor += 1;
            while cursor < bytes.len() {
                let ch = source[cursor..].chars().next().unwrap();
                if !ch.is_whitespace() {
                    break;
                }
                cursor += ch.len_utf8();
            }
            if cursor >= bytes.len() || bytes[cursor] != b'{' {
                idx = cursor;
                continue;
            }
            let block_start = cursor + 1;
            let block_end = find_matching_brace(source, cursor)?;
            let block = &source[block_start..block_end];
            let mut found = parse_block(block, source, file, block_start)?;
            variants.append(&mut found);
            idx = block_end + 1;
            continue;
        }
        let ch = source[idx..].chars().next().unwrap();
        idx += ch.len_utf8();
    }
    Ok(variants)
}

fn find_matching_brace(source: &str, start: usize) -> Result<usize, String> {
    let mut depth = 0usize;
    let mut idx = start;
    let mut in_string = false;
    let mut escape = false;
    while idx < source.len() {
        let ch = source[idx..].chars().next().unwrap();
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
        } else if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Ok(idx);
            }
        }
        idx += ch.len_utf8();
    }
    Err("E_ALRIM_PARSE brace not closed".to_string())
}

fn parse_block(
    block: &str,
    source: &str,
    file: &Path,
    base_offset: usize,
) -> Result<Vec<VariantSchema>, String> {
    let mut variants = Vec::new();
    let mut idx = 0usize;
    let bytes = block.as_bytes();
    let mut in_string = false;
    let mut escape = false;
    while idx < bytes.len() {
        if in_string {
            let ch = block[idx..].chars().next().unwrap();
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            idx += ch.len_utf8();
            continue;
        }
        if bytes[idx] == b'/' && idx + 1 < bytes.len() && bytes[idx + 1] == b'/' {
            while idx < bytes.len() {
                let ch = block[idx..].chars().next().unwrap();
                idx += ch.len_utf8();
                if ch == '\n' {
                    break;
                }
            }
            continue;
        }
        if bytes[idx] == b'"' {
            in_string = true;
            idx += 1;
            continue;
        }
        if bytes[idx] == b'#' {
            let tag_start = idx;
            idx += 1;
            let name_start = idx;
            while idx < bytes.len() {
                let ch = block[idx..].chars().next().unwrap();
                if !is_tag_char(ch) {
                    break;
                }
                idx += ch.len_utf8();
            }
            let tag = block[name_start..idx].trim();
            if tag.is_empty() {
                continue;
            }
            let mut fields = Vec::new();
            let mut lookahead = idx;
            while lookahead < bytes.len() {
                let ch = block[lookahead..].chars().next().unwrap();
                if !ch.is_whitespace() {
                    break;
                }
                lookahead += ch.len_utf8();
            }
            if lookahead < bytes.len() && bytes[lookahead] == b'(' {
                let (parsed, next_idx) = parse_fields(block, lookahead + 1)?;
                fields = parsed;
                idx = next_idx;
            }
            let origin = offset_to_origin(file, source, base_offset + tag_start);
            variants.push(VariantSchema {
                tag: format!("#{}", tag),
                fields,
                origins: vec![origin],
            });
            continue;
        }
        let ch = block[idx..].chars().next().unwrap();
        idx += ch.len_utf8();
    }
    Ok(variants)
}

fn parse_fields(block: &str, mut idx: usize) -> Result<(Vec<FieldSchema>, usize), String> {
    let mut fields = Vec::new();
    let bytes = block.as_bytes();
    loop {
        while idx < bytes.len() {
            let ch = block[idx..].chars().next().unwrap();
            if !ch.is_whitespace() {
                break;
            }
            idx += ch.len_utf8();
        }
        if idx >= bytes.len() {
            return Err("E_ALRIM_PARSE field list not closed".to_string());
        }
        if bytes[idx] == b')' {
            idx += 1;
            break;
        }
        let name_start = idx;
        while idx < bytes.len() {
            let ch = block[idx..].chars().next().unwrap();
            if !is_ident_char(ch) {
                break;
            }
            idx += ch.len_utf8();
        }
        let name = block[name_start..idx].trim().to_string();
        while idx < bytes.len() && block[idx..].chars().next().unwrap().is_whitespace() {
            idx += block[idx..].chars().next().unwrap().len_utf8();
        }
        if idx < bytes.len() && bytes[idx] == b':' {
            idx += 1;
        }
        while idx < bytes.len() && block[idx..].chars().next().unwrap().is_whitespace() {
            idx += block[idx..].chars().next().unwrap().len_utf8();
        }
        let type_start = idx;
        while idx < bytes.len() {
            let ch = block[idx..].chars().next().unwrap();
            if ch == ',' || ch == ')' || ch == '=' {
                break;
            }
            idx += ch.len_utf8();
        }
        let type_name = block[type_start..idx].trim().to_string();
        let mut default = None;
        if idx < bytes.len() && bytes[idx] == b'=' {
            idx += 1;
            while idx < bytes.len() {
                let ch = block[idx..].chars().next().unwrap();
                if ch.is_whitespace() {
                    idx += ch.len_utf8();
                } else {
                    break;
                }
            }
            let default_start = idx;
            while idx < bytes.len() {
                let ch = block[idx..].chars().next().unwrap();
                if ch == ',' || ch == ')' {
                    break;
                }
                idx += ch.len_utf8();
            }
            let default_val = block[default_start..idx].trim();
            if !default_val.is_empty() {
                default = Some(default_val.to_string());
            }
        }
        fields.push(FieldSchema {
            name,
            type_name,
            default,
        });
        while idx < bytes.len() && block[idx..].chars().next().unwrap().is_whitespace() {
            idx += block[idx..].chars().next().unwrap().len_utf8();
        }
        if idx < bytes.len() && bytes[idx] == b',' {
            idx += 1;
            continue;
        }
        if idx < bytes.len() && bytes[idx] == b')' {
            idx += 1;
            break;
        }
    }
    Ok((fields, idx))
}

fn offset_to_origin(file: &Path, source: &str, offset: usize) -> Origin {
    let mut line = 1usize;
    let mut col = 1usize;
    for (idx, ch) in source.char_indices() {
        if idx >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    Origin {
        file: file.to_string_lossy().to_string(),
        line,
        col,
    }
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || matches!(ch, '가'..='힣' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ')
}

fn is_tag_char(ch: char) -> bool {
    is_ident_char(ch)
}

fn encode_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn encode_str(out: &mut Vec<u8>, text: &str) {
    encode_u32(out, text.len() as u32);
    out.extend_from_slice(text.as_bytes());
}

fn variant_detbin(variant: &VariantSchema) -> Vec<u8> {
    let mut out = Vec::new();
    encode_str(&mut out, &variant.tag);
    encode_u32(&mut out, variant.fields.len() as u32);
    for field in &variant.fields {
        encode_str(&mut out, &field.name);
        encode_str(&mut out, &field.type_name);
        match &field.default {
            Some(default) => {
                out.push(1);
                encode_str(&mut out, default);
            }
            None => out.push(0),
        }
    }
    out
}

fn registry_detbin(schema_version: &str, variants: &[VariantOutput]) -> Vec<u8> {
    let mut out = Vec::new();
    encode_str(&mut out, schema_version);
    encode_u32(&mut out, variants.len() as u32);
    for variant in variants {
        encode_str(&mut out, &variant.tag);
        encode_u32(&mut out, variant.fields.len() as u32);
        for field in &variant.fields {
            encode_str(&mut out, &field.name);
            encode_str(&mut out, &field.type_name);
            match &field.default {
                Some(default) => {
                    out.push(1);
                    encode_str(&mut out, default);
                }
                None => out.push(0),
            }
        }
    }
    out
}

fn build_registry_json(
    schema_version: &str,
    registry_hash: &str,
    variants: &[VariantOutput],
) -> Result<String, String> {
    let mut root = Map::new();
    root.insert(
        "schema_version".to_string(),
        Value::String(schema_version.to_string()),
    );
    root.insert(
        "alrim_registry_hash".to_string(),
        Value::String(registry_hash.to_string()),
    );
    let mut items = Vec::new();
    for variant in variants {
        let mut entry = Map::new();
        entry.insert("tag".to_string(), Value::String(variant.tag.clone()));
        entry.insert(
            "signal_id".to_string(),
            Value::Number(serde_json::Number::from(variant.signal_id)),
        );
        entry.insert(
            "variant_schema_hash".to_string(),
            Value::String(variant.schema_hash.clone()),
        );
        let mut fields_json = Vec::new();
        for field in &variant.fields {
            let mut f = Map::new();
            f.insert("name".to_string(), Value::String(field.name.clone()));
            f.insert("type".to_string(), Value::String(field.type_name.clone()));
            if let Some(default) = &field.default {
                f.insert("default".to_string(), Value::String(default.clone()));
            }
            fields_json.push(Value::Object(f));
        }
        entry.insert("fields".to_string(), Value::Array(fields_json));

        let mut origins_json = Vec::new();
        for origin in &variant.origins {
            let mut o = Map::new();
            o.insert("file".to_string(), Value::String(origin.file.clone()));
            o.insert(
                "line".to_string(),
                Value::Number(serde_json::Number::from(origin.line as u64)),
            );
            o.insert(
                "col".to_string(),
                Value::Number(serde_json::Number::from(origin.col as u64)),
            );
            origins_json.push(Value::Object(o));
        }
        entry.insert("origins".to_string(), Value::Array(origins_json));
        items.push(Value::Object(entry));
    }
    root.insert("variants".to_string(), Value::Array(items));
    serde_json::to_string_pretty(&Value::Object(root))
        .map_err(|e| format!("E_ALRIM_JSON {}", e))
        .map(|text| format!("{}\n", text))
}
