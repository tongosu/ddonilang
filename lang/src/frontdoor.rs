const LEGACY_HEADER_KEYS: &[&str] = &[
    "이름",
    "설명",
    "말씨",
    "사투리",
    "그래프",
    "필수보기",
    "required_views",
    "필수보개",
    "조종",
    "조절",
    "control",
];

pub fn preprocess_frontdoor_source(input: &str) -> String {
    let stripped = strip_file_leading_setting_block(input);
    rewrite_inline_maegim_fields(&stripped)
}

pub fn find_legacy_header(source: &str) -> Option<(usize, &'static str)> {
    for (line_no, raw) in source.lines().enumerate() {
        let line = raw.trim_start();
        if !line.starts_with('#') {
            continue;
        }
        let rest = line[1..].trim_start();
        for key in LEGACY_HEADER_KEYS {
            if header_key_matches(rest, key) {
                return Some((line_no + 1, *key));
            }
        }
    }
    None
}

pub fn validate_no_legacy_header(source: &str) -> Result<(), String> {
    if let Some((line, key)) = find_legacy_header(source) {
        return Err(format!(
            "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN line={line} key={key} use=설정{{}}/매김{{}}/설정.보개"
        ));
    }
    Ok(())
}

pub fn has_legacy_boim_surface(source: &str) -> bool {
    let prepared = preprocess_frontdoor_source(source);
    for line in prepared.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("보임") {
            continue;
        }
        let rest = trimmed["보임".len()..].trim_start();
        if rest.starts_with('{') || rest.starts_with(':') {
            return true;
        }
    }
    false
}

pub fn validate_no_legacy_boim_surface(source: &str) -> Result<(), String> {
    if has_legacy_boim_surface(source) {
        return Err(
            "E_CANON_LEGACY_BOIM_FORBIDDEN legacy `보임 {}` 표면은 금지되었습니다. `설정.보개`/정본 보개 표면으로 전환하세요."
                .to_string(),
        );
    }
    Ok(())
}

pub fn normalize_for_lang_parity(source: &str) -> String {
    let without_maegim = strip_decl_item_maegim_suffix_for_lang_parity(source);
    let without_hook_colon = strip_hook_colon_before_block_for_lang_parity(&without_maegim);
    rewrite_three_arg_range_bound_call_for_lang_parity(&without_hook_colon)
}

pub fn wrap_lang_parity_source(source: &str) -> String {
    format!("프론트도어_패리티_검사:움직씨 = {{\n{source}\n}}\n")
}

fn strip_file_leading_setting_block(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        match bytes[idx] {
            b' ' | b'\t' | b'\r' | b'\n' => idx += 1,
            0xEF if idx + 2 < bytes.len() && bytes[idx + 1] == 0xBB && bytes[idx + 2] == 0xBF => {
                idx += 3;
            }
            _ => break,
        }
    }

    let remain = &input[idx..];
    if !(remain.starts_with("설정")
        || remain.starts_with("  설정")
        || remain.starts_with("\t설정"))
    {
        return input.to_string();
    }

    let mut cursor = idx;
    cursor += "설정".len();
    while let Some(ch) = input[cursor..].chars().next() {
        if matches!(ch, ' ' | '\t' | '\r' | '\n') {
            cursor += ch.len_utf8();
        } else {
            break;
        }
    }
    if input[cursor..].starts_with(':') {
        cursor += 1;
        while let Some(ch) = input[cursor..].chars().next() {
            if matches!(ch, ' ' | '\t' | '\r' | '\n') {
                cursor += ch.len_utf8();
            } else {
                break;
            }
        }
    }
    if !input[cursor..].starts_with('{') {
        return input.to_string();
    }

    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let mut end = cursor;
    for (off, ch) in input[cursor..].char_indices() {
        end = cursor + off + ch.len_utf8();
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return input.to_string();
    }

    let mut tail = end;
    while let Some(ch) = input[tail..].chars().next() {
        if matches!(ch, ' ' | '\t' | '\r' | '\n') {
            tail += ch.len_utf8();
        } else {
            break;
        }
    }
    if input[tail..].starts_with('.') {
        tail += 1;
    }
    while let Some(ch) = input[tail..].chars().next() {
        if matches!(ch, ' ' | '\t' | '\r' | '\n') {
            tail += ch.len_utf8();
        } else {
            break;
        }
    }

    let mut out = String::new();
    out.push_str(&input[..idx]);
    out.push_str(&input[tail..]);
    out
}

fn rewrite_inline_maegim_fields(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + input.len() / 20);
    for chunk in input.split_inclusive('\n') {
        let (line, newline) = split_line_frontdoor(chunk);
        if let Some(rewritten) = rewrite_single_line_maegim(line) {
            out.push_str(&rewritten);
            out.push_str(newline);
        } else {
            out.push_str(line);
            out.push_str(newline);
        }
    }
    out
}

fn rewrite_single_line_maegim(line: &str) -> Option<String> {
    let maegim_idx = line.find("매김 {")?;
    let brace_start = maegim_idx + "매김 ".len();
    let close_rel = line[brace_start..].find('}')?;
    let close_idx = brace_start + close_rel;
    let inside = line[brace_start + 1..close_idx].trim();
    if inside.is_empty() {
        return None;
    }

    let chars: Vec<char> = inside.chars().collect();
    let mut entries = Vec::new();
    let mut buf = String::new();
    for i in 0..chars.len() {
        let ch = chars[i];
        buf.push(ch);
        if ch != '.' {
            continue;
        }
        let prev = if i > 0 { Some(chars[i - 1]) } else { None };
        let next = chars.get(i + 1).copied();
        let is_range_dot = prev == Some('.') || next == Some('.');
        let is_decimal_dot =
            prev.is_some_and(|c| c.is_ascii_digit()) && next.is_some_and(|c| c.is_ascii_digit());
        if is_range_dot || is_decimal_dot {
            continue;
        }
        let item = buf.trim();
        if !item.is_empty() {
            entries.push(item.to_string());
        }
        buf.clear();
    }
    if !buf.trim().is_empty() {
        return None;
    }
    if entries.len() <= 1 {
        return None;
    }

    let indent_len = line.len() - line.trim_start().len();
    let indent = &line[..indent_len];
    let field_indent = format!("{indent}  ");
    let prefix = &line[..brace_start + 1];
    let suffix = &line[close_idx + 1..];

    let mut out = String::new();
    out.push_str(prefix.trim_end());
    out.push('\n');
    for entry in entries {
        out.push_str(&field_indent);
        out.push_str(entry.trim());
        out.push('\n');
    }
    out.push_str(indent);
    out.push('}');
    out.push_str(suffix);
    Some(out)
}

fn split_line_frontdoor(chunk: &str) -> (&str, &str) {
    if let Some(stripped) = chunk.strip_suffix('\n') {
        if let Some(stripped_cr) = stripped.strip_suffix('\r') {
            return (stripped_cr, "\r\n");
        }
        return (stripped, "\n");
    }
    (chunk, "")
}

fn strip_decl_item_maegim_suffix_for_lang_parity(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
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

        if (keyword_at(&chars, i, "매김") || keyword_at(&chars, i, "조건"))
            && prev_non_ws_char(&chars, i) == Some(')')
        {
            let mut j = i + 2;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && chars[j] == ':' {
                j += 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
            }
            if j < chars.len() && chars[j] == '{' {
                let mut depth = 1usize;
                j += 1;
                while j < chars.len() && depth > 0 {
                    match chars[j] {
                        '{' => depth += 1,
                        '}' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                while j < chars.len() && chars[j].is_whitespace() && chars[j] != '\n' {
                    j += 1;
                }
                i = j;
                continue;
            }
        }

        out.push(ch);
        i += 1;
    }
    out
}

fn keyword_at(chars: &[char], idx: usize, keyword: &str) -> bool {
    let mut j = idx;
    for kc in keyword.chars() {
        if chars.get(j).copied() != Some(kc) {
            return false;
        }
        j += 1;
    }
    let prev_ok = idx == 0
        || chars
            .get(idx - 1)
            .copied()
            .is_some_and(|c| c.is_whitespace() || matches!(c, '{' | '}' | '(' | ')' | '.' | ':'));
    let next_ok = j >= chars.len()
        || chars
            .get(j)
            .copied()
            .is_some_and(|c| c.is_whitespace() || matches!(c, '{' | ':' | '.'));
    prev_ok && next_ok
}

fn prev_non_ws_char(chars: &[char], idx: usize) -> Option<char> {
    if idx == 0 {
        return None;
    }
    let mut j = idx;
    while j > 0 {
        j -= 1;
        let c = chars[j];
        if !c.is_whitespace() {
            return Some(c);
        }
    }
    None
}

fn strip_hook_colon_before_block_for_lang_parity(source: &str) -> String {
    const KEYWORDS: &[&str] = &["할때", "마다", "될때", "동안"];
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
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

        let mut removed_colon = false;
        for kw in KEYWORDS {
            if !keyword_at(&chars, i, kw) {
                continue;
            }
            let kw_len = kw.chars().count();
            let kw_end = i + kw_len;
            let mut j = kw_end;
            while j < chars.len() && chars[j].is_whitespace() && chars[j] != '\n' {
                j += 1;
            }
            if j < chars.len() && chars[j] == ':' {
                let mut k = j + 1;
                while k < chars.len() && chars[k].is_whitespace() && chars[k] != '\n' {
                    k += 1;
                }
                if k < chars.len() && chars[k] == '{' {
                    for c in &chars[i..j] {
                        out.push(*c);
                    }
                    i = j + 1;
                    removed_colon = true;
                    break;
                }
            }
        }
        if removed_colon {
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
}

fn rewrite_three_arg_range_bound_call_for_lang_parity(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line_frontdoor(chunk);
        if let Some(rewritten) = rewrite_single_line_three_arg_range(line) {
            out.push_str(&rewritten);
            out.push_str(newline);
        } else {
            out.push_str(line);
            out.push_str(newline);
        }
    }
    out
}

fn rewrite_single_line_three_arg_range(line: &str) -> Option<String> {
    let assign_idx = line.find("<-")?;
    let tail = &line[assign_idx + 2..];
    let open_rel = tail.find('(')?;
    let open_idx = assign_idx + 2 + open_rel;
    let mut depth = 0usize;
    let mut close_idx = None;
    for (off, ch) in line[open_idx..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close_idx = Some(open_idx + off);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_idx = close_idx?;
    let after = &line[close_idx + 1..];
    let ws_len = after.chars().take_while(|c| c.is_whitespace()).count();
    let after_trim = &after[after
        .char_indices()
        .nth(ws_len)
        .map(|(i, _)| i)
        .unwrap_or(after.len())..];
    if !after_trim.starts_with("범위") {
        return None;
    }
    let inside = &line[open_idx + 1..close_idx];
    let parts = split_top_level_commas(inside);
    if parts.len() != 3 {
        return None;
    }
    let start = parts[0].trim();
    let end = parts[1].trim();
    if start.is_empty() || end.is_empty() {
        return None;
    }
    let keyword_bytes = "범위".len();
    let keyword_start = line[close_idx + 1..]
        .find("범위")
        .map(|off| close_idx + 1 + off)?;
    let after_keyword = &line[keyword_start + keyword_bytes..];
    Some(format!(
        "{}({} .. {}){}",
        &line[..open_idx],
        start,
        end,
        after_keyword
    ))
}

fn split_top_level_commas(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (idx, ch) in input.char_indices() {
        match ch {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(input[start..idx].to_string());
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(input[start..].to_string());
    parts
}

fn header_key_matches(rest: &str, key: &str) -> bool {
    if key.is_ascii() {
        let lower = rest.to_ascii_lowercase();
        let key_lower = key.to_ascii_lowercase();
        if !lower.starts_with(&key_lower) {
            return false;
        }
        return lower[key_lower.len()..].trim_start().starts_with(':');
    }
    if !rest.starts_with(key) {
        return false;
    }
    rest[key.len()..].trim_start().starts_with(':')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_legacy_header_detects_hash_header() {
        let source = "#이름: 예제\n(매마디)마다 { n <- 1. }.";
        let detected = find_legacy_header(source).expect("legacy header");
        assert_eq!(detected.0, 1);
        assert_eq!(detected.1, "이름");
    }

    #[test]
    fn validate_no_legacy_header_accepts_canonical_surface() {
        let source = "설정 { 문서 { 이름: \"ok\". }. }.\n(매마디)마다 { n <- 1. }.";
        validate_no_legacy_header(source).expect("must pass");
    }

    #[test]
    fn validate_no_legacy_boim_surface_detects_block() {
        let source = "보임 { x: 1. }.";
        let err = validate_no_legacy_boim_surface(source).expect_err("must fail");
        assert!(err.contains("E_CANON_LEGACY_BOIM_FORBIDDEN"));
    }

    #[test]
    fn normalize_for_lang_parity_removes_decl_item_maegim_suffix() {
        let src = "채비 { g: 수 <- (9.8) 매김 { 범위(0..10). 간격(1). }. }.";
        let out = normalize_for_lang_parity(src);
        assert!(!out.contains(" 매김 {"));
    }

    #[test]
    fn normalize_for_lang_parity_rewrites_three_arg_range() {
        let src = "경로 <- (0, 1, 2) 범위.";
        let out = normalize_for_lang_parity(src);
        assert!(out.contains("(0 .. 1)"));
    }

    #[test]
    fn wrap_lang_parity_source_wraps_seed() {
        let wrapped = wrap_lang_parity_source("x <- 1.");
        assert!(wrapped.contains("프론트도어_패리티_검사:움직씨"));
        assert!(wrapped.contains("x <- 1."));
    }
}
