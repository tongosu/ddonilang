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
    #[allow(dead_code)]
    pub stripped: String,
    pub dup_keys: Vec<String>,
    #[allow(dead_code)]
    pub meta_lines: usize,
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
            let trimmed =
                line.trim_start_matches(|ch| matches!(ch, ' ' | '\t' | '\r' | '\u{feff}'));
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

pub fn file_meta_to_json(meta: &FileMeta) -> String {
    let mut entries = Vec::with_capacity(meta.entries.len());
    for entry in &meta.entries {
        let mut item = serde_json::Map::new();
        item.insert(
            "key".to_string(),
            serde_json::Value::String(entry.key.clone()),
        );
        item.insert(
            "value".to_string(),
            serde_json::Value::String(entry.value.clone()),
        );
        entries.push(serde_json::Value::Object(item));
    }
    let mut root = serde_json::Map::new();
    root.insert("entries".to_string(), serde_json::Value::Array(entries));
    let text = serde_json::to_string_pretty(&serde_json::Value::Object(root))
        .unwrap_or_else(|_| "{}".to_string());
    format!("{}\n", text)
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
