pub fn preprocess_frontdoor_source(input: &str) -> String {
    let stripped = strip_file_leading_setting_block(input);
    rewrite_inline_maegim_fields(&stripped)
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
