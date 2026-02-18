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
            model_id: default_model_id(),
            prompt_hash: default_prompt_hash(),
            schema_hash: schema_hash.unwrap_or_else(|| "UNKNOWN".to_string()),
            toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
            policy_hash: read_policy_hash(),
        }
    }
}

const DEFAULT_AI_MODEL_ID: &str = "ddn.slgi.default";
const DEFAULT_PROMPT_FINGERPRINT: &str = "slgi-block-v1";

fn default_model_id() -> String {
    std::env::var("DDN_AI_MODEL_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_AI_MODEL_ID.to_string())
}

fn default_prompt_hash() -> String {
    format!(
        "blake3:{}",
        blake3::hash(DEFAULT_PROMPT_FINGERPRINT.as_bytes()).to_hex()
    )
}

pub fn preprocess_source_for_parse(source: &str) -> Result<String, String> {
    let stripped = strip_slgi_blocks(source)?;
    if find_ai_call(&stripped).is_some() {
        return Err("AI_PREPROCESS_REQUIRED: `??()`/`??{}` 전처리가 필요합니다".to_string());
    }
    let no_comments = strip_line_comments(&stripped);
    let rewritten_hooks = rewrite_hook_every_to_seed(&no_comments);
    let rewritten = rewrite_legacy_shorthand(&rewritten_hooks);
    let expanded_colons = expand_colon_blocks(&rewritten);
    let no_brace_dot = normalize_closing_brace_dot(&expanded_colons);
    let no_unary = rewrite_unary_minus(&no_brace_dot);
    let wrapped = wrap_if_no_seed_def(&no_unary);
    Ok(wrapped)
}

/// 콜론 블록(`...:`)을 명시적 `{ ... }` 블록으로 확장
/// - 들여쓰기 기반으로 블록 종료를 감지
/// - `#`/`//` 주석 라인은 블록 개시/종료 판단에서 제외
fn expand_colon_blocks(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut stack: Vec<(usize, String)> = Vec::new();
    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line(chunk);
        let trimmed_end = line.trim_end();
        let trimmed_start = line.trim_start();
        if trimmed_end.is_empty() || trimmed_start.starts_with('#') || trimmed_start.starts_with("//") {
            out.push_str(line);
            out.push_str(newline);
            continue;
        }

        let indent_len = line.len().saturating_sub(trimmed_start.len());
        while let Some((indent, indent_str)) = stack.last() {
            if indent_len < *indent {
                out.push_str(indent_str);
                out.push('}');
                out.push_str(newline);
                stack.pop();
            } else {
                break;
            }
        }

        let is_colon_block = trimmed_end.ends_with(':')
            && !trimmed_end.contains('{')
            && !trimmed_start.starts_with('#');
        if is_colon_block {
            let mut line_no_colon = trimmed_end.to_string();
            line_no_colon.pop();
            out.push_str(&line_no_colon);
            out.push_str(" {");
            out.push_str(newline);
            let indent_str = line[..indent_len].to_string();
            stack.push((indent_len + 1, indent_str));
            continue;
        }

        out.push_str(line);
        out.push_str(newline);
    }
    while let Some((_, indent_str)) = stack.pop() {
        out.push_str(&indent_str);
        out.push_str("}\n");
    }
    out
}

fn rewrite_hook_every_to_seed(source: &str) -> String {
    if !contains_hook(source) {
        return source.to_string();
    }

    let (stripped, start_blocks, every_blocks) = extract_hook_blocks(source);
    if start_blocks.is_empty() && every_blocks.is_empty() {
        return stripped;
    }

    if !has_seed_def(&stripped) {
        let mut out = String::new();
        out.push_str("매마디:움직씨 = {\n");
        out.push_str(&render_hook_blocks("  ", &start_blocks, &every_blocks));
        out.push_str("}\n");
        return out;
    }

    if let Some((insert_at, indent)) = find_update_seed_insert(&stripped) {
        let hook_text = render_hook_blocks(&indent, &start_blocks, &every_blocks);
        if hook_text.is_empty() {
            return stripped;
        }
        let mut out = String::with_capacity(stripped.len() + hook_text.len() + 8);
        out.push_str(&stripped[..insert_at]);
        out.push_str(&hook_text);
        out.push_str(&stripped[insert_at..]);
        return out;
    }

    let mut out = stripped;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("매마디:움직씨 = {\n");
    out.push_str(&render_hook_blocks("  ", &start_blocks, &every_blocks));
    out.push_str("}\n");
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HookKind {
    Every,
    Start,
}

fn hook_start_kind(trimmed: &str) -> Option<HookKind> {
    let trimmed = trimmed.trim_start();
    if trimmed.starts_with("(시작)할때") {
        return Some(HookKind::Start);
    }
    if trimmed.starts_with("(매마디)마다")
        || trimmed.starts_with("(마디)마다")
        || trimmed.starts_with("마다")
    {
        return Some(HookKind::Every);
    }
    None
}

fn contains_hook(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let indent_len = line.len().saturating_sub(trimmed.len());
        if indent_len != 0 {
            continue;
        }
        if hook_start_kind(trimmed).is_some() {
            return true;
        }
    }
    false
}

fn hook_line_rest_and_depth(line: &str) -> Option<(String, i32)> {
    let idx = line.find('{')?;
    let rest = line[idx + 1..].to_string();
    let mut depth = 1;
    depth += brace_diff(&rest);
    Some((rest, depth))
}

fn extract_hook_blocks(source: &str) -> (String, Vec<String>, Vec<String>) {
    let mut out = String::with_capacity(source.len());
    let mut start_blocks = Vec::new();
    let mut every_blocks = Vec::new();

    let mut in_hook = false;
    let mut hook_depth: i32 = 0;
    let mut hook_kind: HookKind = HookKind::Every;
    let mut hook_buf = String::new();

    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line(chunk);
        let trimmed = line.trim_start();
        let indent_len = line.len().saturating_sub(trimmed.len());

        if !in_hook {
            if indent_len == 0 {
                if let Some(kind) = hook_start_kind(trimmed) {
                    in_hook = true;
                    hook_kind = kind;
                    hook_buf.clear();
                    if let Some((rest, depth)) = hook_line_rest_and_depth(line) {
                        hook_depth = depth;
                        if !rest.trim().is_empty() {
                            hook_buf.push_str(rest.trim_end());
                            hook_buf.push_str(newline);
                        }
                        if hook_depth <= 0 {
                            store_hook_block(&mut start_blocks, &mut every_blocks, hook_kind, &hook_buf);
                            in_hook = false;
                            hook_depth = 0;
                            hook_buf.clear();
                        }
                    } else {
                        hook_depth = 1;
                    }
                    continue;
                }
            }
            out.push_str(line);
            out.push_str(newline);
            continue;
        }

        let diff = brace_diff(line);
        hook_depth += diff;
        let trimmed_end = trimmed.trim_end();
        let is_closing = trimmed_end == "}" || trimmed_end == "}.";
        if hook_depth <= 0 && is_closing {
            let before = trimmed_end.split('}').next().unwrap_or("");
            if !before.trim().is_empty() {
                hook_buf.push_str(before.trim_end());
                hook_buf.push_str(newline);
            }
            store_hook_block(&mut start_blocks, &mut every_blocks, hook_kind, &hook_buf);
            in_hook = false;
            hook_depth = 0;
            hook_buf.clear();
            continue;
        }
        hook_buf.push_str(line);
        hook_buf.push_str(newline);
        if hook_depth <= 0 {
            store_hook_block(&mut start_blocks, &mut every_blocks, hook_kind, &hook_buf);
            in_hook = false;
            hook_depth = 0;
            hook_buf.clear();
        }
    }

    if in_hook && !hook_buf.is_empty() {
        store_hook_block(&mut start_blocks, &mut every_blocks, hook_kind, &hook_buf);
    }

    (out, start_blocks, every_blocks)
}

fn store_hook_block(
    start_blocks: &mut Vec<String>,
    every_blocks: &mut Vec<String>,
    kind: HookKind,
    block: &str,
) {
    if block.trim().is_empty() {
        return;
    }
    let entry = block.to_string();
    match kind {
        HookKind::Start => start_blocks.push(entry),
        HookKind::Every => every_blocks.push(entry),
    }
}

fn render_hook_blocks(indent: &str, start_blocks: &[String], every_blocks: &[String]) -> String {
    let mut out = String::new();
    if !start_blocks.is_empty() {
        out.push_str(indent);
        out.push_str("(__wasm_start_once == 0) 일때 {");
        out.push('\n');
        for block in start_blocks {
            for line in block.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                out.push_str(indent);
                out.push_str("  ");
                out.push_str(trimmed);
                out.push('\n');
            }
        }
        out.push_str(indent);
        out.push_str("  __wasm_start_once <- 1.");
        out.push('\n');
        out.push_str(indent);
        out.push_str("}.");
        out.push('\n');
    }
    for block in every_blocks {
        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            out.push_str(indent);
            out.push_str(trimmed);
            out.push('\n');
        }
    }
    out
}

fn find_update_seed_insert(source: &str) -> Option<(usize, String)> {
    let names = ["매마디:움직씨", "매틱:움직씨"];
    for name in names {
        if let Some(idx) = source.find(name) {
            let rest = &source[idx + name.len()..];
            let brace_rel = rest.find('{')?;
            let brace_idx = idx + name.len() + brace_rel;
            if let Some(close_idx) = find_matching_brace(source, brace_idx) {
                let indent = infer_seed_indent(source, brace_idx);
                return Some((close_idx, indent));
            }
        }
    }
    None
}

fn find_matching_brace(source: &str, open_idx: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if *bytes.get(open_idx)? != b'{' {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open_idx;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn infer_seed_indent(source: &str, brace_idx: usize) -> String {
    let slice = &source[brace_idx..];
    if let Some(pos) = slice.find('\n') {
        let after = &slice[pos + 1..];
        let mut indent = String::new();
        for ch in after.chars() {
            if ch == ' ' || ch == '\t' {
                indent.push(ch);
            } else {
                break;
            }
        }
        if !indent.is_empty() {
            return indent;
        }
    }
    "  ".to_string()
}

fn brace_diff(line: &str) -> i32 {
    let mut diff = 0i32;
    for ch in line.chars() {
        if ch == '{' {
            diff += 1;
        } else if ch == '}' {
            diff -= 1;
        }
    }
    diff
}

fn rewrite_legacy_shorthand(source: &str) -> String {
    // teul-cli 호환: "보개로 그려." 문장을 Gate0 파서가 먹을 수 있게 정규화한다.
    // 현재 wasm/런타임 경로에서는 보개 draw 호출이 별도 문장으로 처리되지 않으므로 no-op으로 둔다.
    let mut out = String::new();
    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line(chunk);
        let trimmed = line.trim();
        if trimmed == "보개로 그려." {
            let indent_len = line.len().saturating_sub(line.trim_start().len());
            let indent = &line[..indent_len];
            out.push_str(indent);
            out.push_str("없음.");
            out.push_str(newline);
            continue;
        }
        out.push_str(chunk);
    }
    out
}

/// teul-cli 호환: `//` 줄 주석 제거 (문자열 내부 제외)
/// lang/ 파서는 `//` 주석을 지원하지 않으므로 전처리 단계에서 제거한다.
fn strip_line_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line(chunk);
        let bytes = line.as_bytes();
        let mut in_string = false;
        let mut escape = false;
        let mut comment_pos: Option<usize> = None;
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if in_string {
                if escape {
                    escape = false;
                } else if b == b'\\' {
                    escape = true;
                } else if b == b'"' {
                    in_string = false;
                }
                i += 1;
            } else if b == b'"' {
                in_string = true;
                i += 1;
            } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                comment_pos = Some(i);
                break;
            } else {
                i += 1;
            }
        }
        match comment_pos {
            Some(pos) => {
                let before = &line[..pos];
                out.push_str(before.trim_end());
            }
            None => out.push_str(line),
        }
        out.push_str(newline);
    }
    out
}

/// teul-cli 호환: 줄 전체가 `}.`인 경우 `}` 로 정규화
/// lang/ 파서는 복합문(일때, 동안 등) 닫는 `}` 뒤에 `.`을 기대하지 않는다.
fn normalize_closing_brace_dot(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line(chunk);
        let trimmed = line.trim();
        if trimmed == "}." {
            let indent_len = line.len().saturating_sub(line.trim_start().len());
            out.push_str(&line[..indent_len]);
            out.push('}');
        } else {
            out.push_str(line);
        }
        out.push_str(newline);
    }
    out
}

/// teul-cli 호환: 단항 마이너스 `-숫자` → `(0 - 숫자)` 변환
/// lang/ 파서는 단항 마이너스(`-5`, `-3.14`)를 지원하지 않으므로 이항 연산으로 풀어쓴다.
fn rewrite_unary_minus(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    let mut in_string = false;
    let mut escape = false;
    while i < chars.len() {
        let ch = chars[i];
        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            i += 1;
            continue;
        }
        if ch == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            if is_unary_minus_context(&out) {
                let num_start = i + 1;
                let mut num_end = num_start;
                let mut has_dot = false;
                while num_end < chars.len() {
                    if chars[num_end].is_ascii_digit() {
                        num_end += 1;
                    } else if chars[num_end] == '.' && !has_dot
                        && num_end + 1 < chars.len()
                        && chars[num_end + 1].is_ascii_digit()
                    {
                        has_dot = true;
                        num_end += 1;
                    } else {
                        break;
                    }
                }
                let number: String = chars[num_start..num_end].iter().collect();
                out.push_str("(0 - ");
                out.push_str(&number);
                out.push(')');
                i = num_end;
                continue;
            }
        }
        out.push(ch);
        i += 1;
    }
    out
}

/// 직전 텍스트의 마지막 비공백 문자를 기준으로 단항 마이너스 위치인지 판단한다.
fn is_unary_minus_context(preceding: &str) -> bool {
    let trimmed = preceding.trim_end();
    if trimmed.is_empty() {
        return true;
    }
    let last = trimmed.chars().last().unwrap();
    matches!(last, '(' | ',' | '+' | '-' | '*' | '/' | '=' | '{' | '<' | ':' | '~' | '.')
}

fn wrap_if_no_seed_def(source: &str) -> String {
    if has_seed_def(source) {
        return source.to_string();
    }
    let mut out = String::new();
    out.push_str("매틱:움직씨 = {\n");
    out.push_str(source);
    if !source.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

fn has_seed_def(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }
        let Some(colon_idx) = trimmed.find(':') else {
            continue;
        };
        let Some(eq_idx) = trimmed[colon_idx + 1..].find('=') else {
            continue;
        };
        let abs_eq = colon_idx + 1 + eq_idx;
        if colon_idx < abs_eq {
            return true;
        }
    }
    false
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

    #[test]
    fn ai_meta_default_removes_todo_placeholders() {
        let meta = AiMeta::default_with_schema(None);
        assert!(!meta.model_id.is_empty());
        assert_ne!(meta.model_id, "TODO");
        assert!(meta.prompt_hash.starts_with("blake3:"));
        assert_ne!(meta.prompt_hash, "TODO");
    }

    // --- strip_line_comments ---

    #[test]
    fn strip_line_comments_basic() {
        let source = "x <- 5. // 설명\ny <- 10.\n";
        let out = strip_line_comments(source);
        assert_eq!(out, "x <- 5.\ny <- 10.\n");
    }

    #[test]
    fn strip_line_comments_preserves_strings() {
        let source = "x <- \"hello // world\".\n";
        let out = strip_line_comments(source);
        assert_eq!(out, "x <- \"hello // world\".\n");
    }

    #[test]
    fn strip_line_comments_full_line() {
        let source = "// 전체 주석\nx <- 1.\n";
        let out = strip_line_comments(source);
        assert_eq!(out, "\nx <- 1.\n");
    }

    #[test]
    fn strip_line_comments_no_trailing_space() {
        let source = "x <- 5.   // 설명\n";
        let out = strip_line_comments(source);
        assert_eq!(out, "x <- 5.\n");
    }

    // --- normalize_closing_brace_dot ---

    #[test]
    fn normalize_brace_dot_standalone_line() {
        let source = "  }.\n";
        let out = normalize_closing_brace_dot(source);
        assert_eq!(out, "  }\n");
    }

    #[test]
    fn normalize_brace_dot_preserves_inline() {
        let source = "x <- { 5 }.\n";
        let out = normalize_closing_brace_dot(source);
        assert_eq!(out, "x <- { 5 }.\n");
    }

    #[test]
    fn normalize_brace_dot_no_dot() {
        let source = "  }\n";
        let out = normalize_closing_brace_dot(source);
        assert_eq!(out, "  }\n");
    }

    // --- rewrite_unary_minus ---

    #[test]
    fn unary_minus_simple() {
        let source = "x <- -5.\n";
        let out = rewrite_unary_minus(source);
        assert_eq!(out, "x <- (0 - 5).\n");
    }

    #[test]
    fn unary_minus_in_parens() {
        let source = "(-5)\n";
        let out = rewrite_unary_minus(source);
        assert_eq!(out, "((0 - 5))\n");
    }

    #[test]
    fn unary_minus_after_operator() {
        let source = "(20 + -2)\n";
        let out = rewrite_unary_minus(source);
        assert_eq!(out, "(20 + (0 - 2))\n");
    }

    #[test]
    fn binary_minus_preserved() {
        let source = "x - 5\n";
        let out = rewrite_unary_minus(source);
        assert_eq!(out, "x - 5\n");
    }

    #[test]
    fn unary_minus_decimal() {
        let source = "x <- -3.14.\n";
        let out = rewrite_unary_minus(source);
        assert_eq!(out, "x <- (0 - 3.14).\n");
    }

    #[test]
    fn unary_minus_preserves_string() {
        let source = "x <- \"-5\".\n";
        let out = rewrite_unary_minus(source);
        assert_eq!(out, "x <- \"-5\".\n");
    }

    #[test]
    fn unary_minus_after_assignment() {
        let source = "x <- -100.\n";
        let out = rewrite_unary_minus(source);
        assert!(out.contains("(0 - 100)"));
    }

    #[test]
    fn unary_minus_after_multiply() {
        let source = "(프레임수 * -2)\n";
        let out = rewrite_unary_minus(source);
        assert_eq!(out, "(프레임수 * (0 - 2))\n");
    }
}
