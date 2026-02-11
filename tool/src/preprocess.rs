use std::fs;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct FileMeta {
    pub entries: Vec<FileMetaEntry>,
}

#[derive(Debug, Clone)]
pub struct FileMetaEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct FileMetaParse {
    pub meta: FileMeta,
    pub stripped: String,
    pub dup_keys: Vec<String>,
    pub meta_lines: usize,
}

#[derive(Debug, Clone)]
pub struct AiMeta {
    pub model_id: String,
    pub prompt_hash: String,
    pub schema_hash: String,
    pub toolchain_version: String,
    pub policy_hash: Option<String>,
}

impl AiMeta {
    pub fn default_with_schema(schema_hash: Option<String>) -> Self {
        Self {
            model_id: "TODO".to_string(),
            prompt_hash: "TODO".to_string(),
            schema_hash: schema_hash.unwrap_or_else(|| "UNKNOWN".to_string()),
            toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
            policy_hash: read_policy_hash(),
        }
    }
}

pub fn preprocess_source_for_parse(source: &str) -> Result<String, String> {
    let stripped = strip_slgi_blocks(source)?;
    if find_ai_call(&stripped).is_some() {
        return Err("AI_PREPROCESS_REQUIRED: `??()`/`??{}` 전처리가 필요합니다".to_string());
    }
    Ok(stripped)
}

pub fn split_file_meta(source: &str) -> FileMetaParse {
    let mut entries: Vec<FileMetaEntry> = Vec::new();
    let mut entry_pos: HashMap<String, usize> = HashMap::new();
    let mut dup_keys: HashSet<String> = HashSet::new();
    let mut out = String::with_capacity(source.len());
    let mut in_header = true;
    let mut meta_lines = 0usize;

    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line(chunk);
        if in_header {
            let trimmed = line
                .trim_start_matches(|ch| matches!(ch, ' ' | '\t' | '\r' | '\u{feff}'));
            if let Some((key, value)) = parse_meta_line(trimmed) {
                if let Some(pos) = entry_pos.get(&key).copied() {
                    entries[pos].value = value;
                    dup_keys.insert(key);
                } else {
                    entry_pos.insert(key.clone(), entries.len());
                    entries.push(FileMetaEntry { key, value });
                }
                out.push_str(newline);
                meta_lines += 1;
                continue;
            }
            if trimmed.is_empty() || trimmed.starts_with('#') {
                in_header = false;
            } else {
                in_header = false;
            }
        }
        out.push_str(chunk);
    }

    let mut dup_keys: Vec<String> = dup_keys.into_iter().collect();
    dup_keys.sort();
    FileMetaParse {
        meta: FileMeta { entries },
        stripped: out,
        dup_keys,
        meta_lines,
    }
}

pub fn format_file_meta(meta: &FileMeta) -> String {
    if meta.entries.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for entry in &meta.entries {
        out.push('#');
        out.push_str(entry.key.trim());
        out.push(':');
        if !entry.value.is_empty() {
            out.push(' ');
            out.push_str(entry.value.trim());
        }
        out.push('\n');
    }
    out
}

fn parse_meta_line(line: &str) -> Option<(String, String)> {
    if !line.starts_with('#') {
        return None;
    }
    let rest = line[1..].trim_start();
    let (key, value) = rest.split_once(':')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key.to_string(), value.trim().to_string()))
}

fn split_line(chunk: &str) -> (&str, &str) {
    if let Some(stripped) = chunk.strip_suffix('\n') {
        if let Some(stripped_cr) = stripped.strip_suffix('\r') {
            return (stripped_cr, "\r\n");
        }
        return (stripped, "\n");
    }
    (chunk, "")
}

pub fn preprocess_ai_calls(source: &str, meta: &AiMeta) -> Result<String, String> {
    let mut out = String::new();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escape = false;
    while i < source.len() {
        if !in_string && source[i..].starts_with("!!{") {
            let end = find_slgi_block_end(source, i)?;
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }

        let ch = source[i..].chars().next().unwrap();
        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += ch.len_utf8();
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        if source[i..].starts_with("??{") {
            let (end, body) = parse_ai_card_block(source, i)?;
            let intent = format!("글무늬{{{}}}", body);
            out.push_str("없음");
            out.push('\n');
            out.push_str(&format_slgi_block(meta, intent.trim()));
            i = end;
            continue;
        }

        if source[i..].starts_with("??(") {
            let (end, args) = parse_ai_call_args(source, i)?;
            let intent = args.trim();
            out.push_str("없음");
            out.push('\n');
            out.push_str(&format_slgi_block(meta, intent));
            i = end;
            continue;
        }

        out.push(ch);
        i += ch.len_utf8();
    }
    Ok(out)
}

fn strip_slgi_blocks(source: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escape = false;
    while i < source.len() {
        if !in_string && source[i..].starts_with("!!{") {
            let mut depth = 1usize;
            let mut j = i + 3;
            let mut inner_in_string = false;
            let mut inner_escape = false;
            while j < source.len() {
                let ch = source[j..].chars().next().unwrap();
                if ch == '\n' {
                    out.push('\n');
                }
                if inner_in_string {
                    if inner_escape {
                        inner_escape = false;
                    } else if ch == '\\' {
                        inner_escape = true;
                    } else if ch == '"' {
                        inner_in_string = false;
                    }
                } else {
                    match ch {
                        '"' => inner_in_string = true,
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                j += ch.len_utf8();
                                i = j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                j += ch.len_utf8();
            }
            if depth != 0 {
                return Err("AI_PREPROCESS_ERROR: `!!{}` 블록이 닫히지 않았습니다".to_string());
            }
            continue;
        }

        let ch = source[i..].chars().next().unwrap();
        out.push(ch);
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
        }
        i += ch.len_utf8();
    }
    Ok(out)
}

fn find_ai_call(source: &str) -> Option<usize> {
    let mut i = 0usize;
    let mut in_string = false;
    let mut escape = false;
    while i < source.len() {
        if !in_string && source[i..].starts_with("!!{") {
            if let Ok(end) = find_slgi_block_end(source, i) {
                i = end;
                continue;
            }
        }

        let ch = source[i..].chars().next().unwrap();
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += ch.len_utf8();
            continue;
        }

        if ch == '"' {
            in_string = true;
            i += ch.len_utf8();
            continue;
        }

        if source[i..].starts_with("??{") || source[i..].starts_with("??(") {
            return Some(i);
        }
        i += ch.len_utf8();
    }
    None
}

fn parse_ai_call_args(source: &str, start: usize) -> Result<(usize, String), String> {
    let mut i = start + 3;
    let mut depth = 1usize;
    let mut in_string = false;
    let mut escape = false;
    while i < source.len() {
        let ch = source[i..].chars().next().unwrap();
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else {
            match ch {
                '"' => in_string = true,
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        let args = source[start + 3..i].to_string();
                        i += ch.len_utf8();
                        return Ok((i, args));
                    }
                }
                _ => {}
            }
        }
        i += ch.len_utf8();
    }
    Err("AI_PREPROCESS_ERROR: `??()` 괄호가 닫히지 않았습니다".to_string())
}

fn parse_ai_card_block(source: &str, start: usize) -> Result<(usize, String), String> {
    let mut i = start + 3;
    let content_start = i;
    let mut depth = 1usize;
    let mut in_string = false;
    let mut escape = false;
    while i < source.len() {
        let ch = source[i..].chars().next().unwrap();
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else {
            match ch {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let body = source[content_start..i].to_string();
                        i += ch.len_utf8();
                        return Ok((i, body));
                    }
                }
                _ => {}
            }
        }
        i += ch.len_utf8();
    }
    Err("AI_PREPROCESS_ERROR: `??{}` 블록이 닫히지 않았습니다".to_string())
}

fn find_slgi_block_end(source: &str, start: usize) -> Result<usize, String> {
    let mut dummy = 0usize;
    find_slgi_block_end_with_newlines(source, start, &mut dummy)
}

fn find_slgi_block_end_with_newlines(
    source: &str,
    start: usize,
    newlines: &mut usize,
) -> Result<usize, String> {
    let mut i = start + 3;
    let mut depth = 1usize;
    let mut in_string = false;
    let mut escape = false;
    while i < source.len() {
        let ch = source[i..].chars().next().unwrap();
        if ch == '\n' {
            *newlines += 1;
        }
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else {
            match ch {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        i += ch.len_utf8();
                        return Ok(i);
                    }
                }
                _ => {}
            }
        }
        i += ch.len_utf8();
    }
    Err("AI_PREPROCESS_ERROR: `!!{}` 블록이 닫히지 않았습니다".to_string())
}

fn format_slgi_block(meta: &AiMeta, intent: &str) -> String {
    let mut block = String::new();
    block.push_str("!!{\n");
    block.push_str(&format!(
        "#슬기: {{ model_id: \"{}\", prompt_hash: \"{}\", schema_hash: \"{}\", toolchain_version: \"{}\"",
        meta.model_id, meta.prompt_hash, meta.schema_hash, meta.toolchain_version
    ));
    if let Some(policy_hash) = &meta.policy_hash {
        block.push_str(&format!(", policy_hash: \"{}\"", policy_hash));
    }
    block.push_str(" }\n");
    if !intent.is_empty() {
        block.push_str(&format!("#의도: {}\n", intent));
    }
    block.push_str("없음.\n");
    block.push_str("}\n");
    block
}

fn read_policy_hash() -> Option<String> {
    let path = std::path::Path::new("ddn.ai.policy.json");
    if !path.exists() {
        return None;
    }
    let data = fs::read(path).ok()?;
    Some(blake3::hash(&data).to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_slgi_blocks_keeps_newlines() {
        let source = "a\n!!{\n#meta\n}\nB";
        let stripped = strip_slgi_blocks(source).expect("strip");
        assert_eq!(stripped, "a\n\n\n\nB");
    }

    #[test]
    fn preprocess_source_rejects_ai_calls() {
        let source = "x <- ??(\"a\").";
        let err = preprocess_source_for_parse(source).unwrap_err();
        assert!(err.contains("AI_PREPROCESS_REQUIRED"));
    }

    #[test]
    fn preprocess_source_rejects_ai_cards() {
        let source = "x <- ??{hello}.";
        let err = preprocess_source_for_parse(source).unwrap_err();
        assert!(err.contains("AI_PREPROCESS_REQUIRED"));
    }

    #[test]
    fn preprocess_ai_calls_inserts_block() {
        let source = "x <- ??(\"a\").";
        let meta = AiMeta::default_with_schema(None);
        let out = preprocess_ai_calls(source, &meta).expect("rewrite");
        assert!(out.contains("없음"));
        assert!(out.contains("!!{"));
    }

    #[test]
    fn preprocess_ai_cards_inserts_block() {
        let source = "x <- ??{hello}.";
        let meta = AiMeta::default_with_schema(None);
        let out = preprocess_ai_calls(source, &meta).expect("rewrite");
        assert!(out.contains("글무늬{hello}"));
        assert!(out.contains("!!{"));
    }
}
