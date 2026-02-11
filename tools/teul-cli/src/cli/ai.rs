use std::fs;
use std::path::Path;

pub fn extract(in_path: &Path, out_path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(in_path).map_err(|e| e.to_string())?;
    let prompts = extract_prompts(&source);
    let mut out = String::new();
    out.push_str("{\"version\":0,\"holes\":[");
    for (idx, prompt) in prompts.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str("{\"id\":");
        out.push_str(&idx.to_string());
        out.push_str(",\"prompt\":\"");
        out.push_str(&escape_json(prompt));
        out.push_str("\"}");
    }
    out.push_str("],\"prompt\":\"");
    out.push_str(&escape_json(prompts.get(0).map(String::as_str).unwrap_or("")));
    out.push_str("\"}\n");
    fs::write(out_path, out).map_err(|e| e.to_string())
}

fn extract_prompts(source: &str) -> Vec<String> {
    let mut prompts = Vec::new();
    let mut i = 0;
    while let Some(pos) = source[i..].find("??(") {
        i += pos + 3;
        let mut chars = source[i..].chars().peekable();
        while matches!(chars.peek(), Some(ch) if ch.is_whitespace()) {
            chars.next();
            i += 1;
        }
        if chars.peek() != Some(&'"') {
            continue;
        }
        chars.next();
        i += 1;
        let mut prompt = String::new();
        let mut escaped = false;
        while let Some(ch) = chars.next() {
            i += ch.len_utf8();
            if escaped {
                let mapped = match ch {
                    'n' => '\n',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                };
                prompt.push(mapped);
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                break;
            }
            prompt.push(ch);
        }
        if !prompt.is_empty() {
            prompts.push(prompt);
        }
    }
    prompts
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}
