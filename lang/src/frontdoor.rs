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

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ('가'..='힣').contains(&ch)
}

pub fn preprocess_frontdoor_source(input: &str) -> String {
    let stripped = strip_file_leading_setting_block(input);
    let range_spaced = normalize_attached_range_josa(&stripped);
    let pipeline_lowered = lower_collection_pipeline_sugar(&range_spaced);
    let connect_endpoint_lowered = lower_connect_endpoint_relation(&pipeline_lowered);
    rewrite_inline_maegim_fields(&connect_endpoint_lowered)
}

pub fn connect_endpoint_relation_seum_rows(source: &str) -> Option<Vec<String>> {
    let trimmed = source.trim();
    let body = trimmed.strip_suffix('.')?.trim_end();
    let rhs = if let Some((_target, expr)) = body.split_once("<-") {
        expr.trim()
    } else {
        body
    };
    let relation_set = parse_connect_endpoint_relation(rhs)?;
    format_connect_endpoint_relation_seum_rows(&relation_set)
}

pub fn owner_inner_seum_canon_rows(source: &str) -> Option<Vec<String>> {
    let prepared = preprocess_frontdoor_source(source);
    let program =
        crate::parse_with_mode(&prepared, "<owner-inner-seum>", crate::ParseMode::Strict).ok()?;
    let mut rows = Vec::new();
    for item in &program.items {
        let crate::TopLevelItem::SeedDef(seed) = item;
        if !matches!(&seed.seed_kind, crate::SeedKind::Named(name) if name == "임자") {
            continue;
        }
        let Some(body) = &seed.body else {
            continue;
        };
        collect_owner_inner_seum_rows_from_body(body, &mut rows);
    }
    if rows.is_empty() {
        None
    } else {
        Some(rows)
    }
}

pub fn owner_state_symbol_table_rows(source: &str) -> Option<Vec<String>> {
    let prepared = preprocess_frontdoor_source(source);
    let program =
        crate::parse_with_mode(&prepared, "<owner-state-symbol-table>", crate::ParseMode::Strict)
            .ok()?;
    let mut rows = Vec::new();
    for item in &program.items {
        let crate::TopLevelItem::SeedDef(seed) = item;
        if !matches!(&seed.seed_kind, crate::SeedKind::Named(name) if name == "임자") {
            continue;
        }
        let Some(body) = &seed.body else {
            continue;
        };
        collect_owner_state_symbol_rows_from_body(&prepared, &seed.canonical_name, body, &mut rows);
    }
    if rows.is_empty() {
        None
    } else {
        Some(rows)
    }
}

fn collect_owner_inner_seum_rows_from_body(body: &crate::Body, rows: &mut Vec<String>) {
    for stmt in &body.stmts {
        match stmt {
            crate::Stmt::Expr { expr, .. } => {
                if let crate::ExprKind::Assertion(assertion) = &expr.kind {
                    rows.push(assertion.canon.clone());
                }
            }
            crate::Stmt::Receive { body, .. } => {
                // Event bodies are reaction code, not owner-local model relation rows.
                let _ = body;
            }
            _ => {}
        }
    }
}

fn collect_owner_state_symbol_rows_from_body(
    source: &str,
    owner: &str,
    body: &crate::Body,
    rows: &mut Vec<String>,
) {
    for stmt in &body.stmts {
        match stmt {
            crate::Stmt::DeclBlock { items, .. } => {
                for item in items {
                    let value = item
                        .value
                        .as_ref()
                        .map(|expr| source_span_text(source, expr.span).unwrap_or("<expr>"))
                        .unwrap_or("<unset>");
                    let kind = match item.kind {
                        crate::DeclKind::Gureut => "state",
                        crate::DeclKind::Butbak => "constant",
                    };
                    rows.push(format!(
                        "owner={owner};symbol={};type={};kind={kind};initializer={value}",
                        item.name,
                        type_ref_label(&item.type_ref)
                    ));
                }
            }
            crate::Stmt::Receive { body, .. } => {
                // Event bodies are reaction code, not owner-local state declarations.
                let _ = body;
            }
            _ => {}
        }
    }
}

fn source_span_text(source: &str, span: crate::Span) -> Option<&str> {
    source.get(span.start..span.end).map(str::trim)
}

fn type_ref_label(type_ref: &crate::TypeRef) -> String {
    match type_ref {
        crate::TypeRef::Named(name) => name.clone(),
        crate::TypeRef::Applied { name, args } => {
            let args = args
                .iter()
                .map(type_ref_label)
                .collect::<Vec<_>>()
                .join(",");
            format!("{name}<{args}>")
        }
        crate::TypeRef::Infer => "_".to_string(),
    }
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
    let _ = source;
    false
}

pub fn validate_no_legacy_boim_surface(source: &str) -> Result<(), String> {
    let _ = source;
    Ok(())
}

pub fn find_legacy_root_hide_directive(source: &str) -> Option<(usize, &'static str)> {
    for (line_no, raw) in source.lines().enumerate() {
        let line = raw.trim_start();
        if !line.starts_with('#') {
            continue;
        }
        let rest = line[1..].trim_start();
        if rest.starts_with("바탕숨김") {
            return Some((line_no + 1, "바탕숨김"));
        }
        if rest.starts_with("암묵살림") {
            return Some((line_no + 1, "암묵살림"));
        }
    }
    None
}

pub fn validate_no_legacy_root_hide_directive(source: &str) -> Result<(), String> {
    if let Some((line, key)) = find_legacy_root_hide_directive(source) {
        return Err(format!(
            "E_PRAGMA_REMOVED line={line} key={key} fix=`채비 {{ ... }}` 선언만 허용됩니다"
        ));
    }
    Ok(())
}

pub fn find_legacy_range_comment(source: &str) -> Option<(usize, &'static str)> {
    for (line_no, raw) in source.lines().enumerate() {
        if let Some(idx) = raw.find("//") {
            let comment = &raw[idx + 2..];
            if comment.contains("범위(") {
                return Some((line_no + 1, "// 범위(...)"));
            }
        }
    }
    None
}

pub fn validate_no_legacy_range_comment(source: &str) -> Result<(), String> {
    if let Some((line, _)) = find_legacy_range_comment(source) {
        return Err(format!(
            "E_LEGACY_RANGE_SYNTAX line={line} fix=`(값) 매김 {{ 범위: a..b. 간격: c. }}.`"
        ));
    }
    Ok(())
}

pub fn find_legacy_root_surface(source: &str) -> Option<(usize, &'static str)> {
    let prepared = preprocess_frontdoor_source(source);
    for (line_no, raw) in prepared.lines().enumerate() {
        let chars: Vec<char> = raw.chars().collect();
        let mut char_index = 0usize;
        while char_index < chars.len() {
            let ch = chars[char_index];
            if ch == '"' {
                char_index += 1;
                while char_index < chars.len() {
                    if chars[char_index] == '"' && chars[char_index.saturating_sub(1)] != '\\' {
                        char_index += 1;
                        break;
                    }
                    char_index += 1;
                }
                continue;
            }
            if ch == '/' && chars.get(char_index + 1) == Some(&'/') {
                break;
            }
            let rest = &raw[raw
                .char_indices()
                .nth(char_index)
                .map(|(idx, _)| idx)
                .unwrap_or(raw.len())..];
            if rest.starts_with("살림.") {
                let prev_ok = if char_index == 0 {
                    true
                } else {
                    !is_ident_char(chars[char_index - 1])
                };
                if prev_ok {
                    return Some((line_no + 1, "살림."));
                }
            }
            if rest.starts_with("바탕.") {
                let prev_ok = if char_index == 0 {
                    true
                } else {
                    !is_ident_char(chars[char_index - 1])
                };
                if prev_ok {
                    return Some((line_no + 1, "바탕."));
                }
            }
            char_index += 1;
        }
    }
    None
}

pub fn validate_no_legacy_root_surface(source: &str) -> Result<(), String> {
    if let Some((line, key)) = find_legacy_root_surface(source) {
        let code = if key == "바탕." {
            "E_BATANG_PREFIX_REMOVED"
        } else {
            "E_SALIM_REMOVED"
        };
        return Err(format!(
            "{code} line={line} key={key} fix=`채비 {{ x:수 <- ... }}`에 선언한 뒤 `x`만 사용하세요"
        ));
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
    if !(remain.starts_with("설정") || remain.starts_with("  설정") || remain.starts_with("\t설정"))
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

fn normalize_attached_range_josa(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
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
        if ch.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i + 1 < chars.len() && chars[i] == '.' && chars[i + 1] == '.' {
                i += 2;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                for c in &chars[start..i] {
                    out.push(*c);
                }
                if i < chars.len() && is_attached_josa_start(chars[i]) {
                    out.push(' ');
                }
                continue;
            }
            for c in &chars[start..i] {
                out.push(*c);
            }
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
}

fn is_attached_josa_start(ch: char) -> bool {
    matches!(ch, '가'..='힣' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ')
}

fn lower_collection_pipeline_sugar(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for chunk in input.split_inclusive('\n') {
        let (line, newline) = split_line_frontdoor(chunk);
        if let Some(rewritten) = lower_collection_pipeline_line(line) {
            out.push_str(&rewritten);
        } else {
            out.push_str(line);
        }
        out.push_str(newline);
    }
    out
}

fn lower_collection_pipeline_line(line: &str) -> Option<String> {
    if !(line.contains("~을") || line.contains("~를")) || !line.contains("~로") {
        return None;
    }
    let trimmed_end = line.trim_end();
    let dot_suffix = trimmed_end.ends_with('.');
    let without_dot = if dot_suffix {
        trimmed_end[..trimmed_end.len().saturating_sub(1)].trim_end()
    } else {
        trimmed_end
    };
    let (lhs_prefix, rhs_source) = if let Some((lhs, rhs)) = without_dot.split_once("<-") {
        (format!("{}<- ", lhs.trim_end()), rhs.trim_start())
    } else {
        (String::new(), without_dot)
    };
    for func in ["거르기", "변환", "정렬", "합치기"] {
        let Some(head) = rhs_source.strip_suffix(func) else {
            continue;
        };
        let groups = parse_pipeline_groups(head.trim_end())?;
        let lowered = match (func, groups.as_slice()) {
            ("거르기" | "변환" | "정렬", [list, callable]) => {
                let list = strip_pipeline_suffix(list, &["~을", "~를"])?;
                let callable = strip_pipeline_suffix(callable, &["~로", "~으로"])?;
                format!("({list}, {callable}) {func}")
            }
            ("합치기", [list, initial, callable]) => {
                let list = strip_pipeline_suffix(list, &["~을", "~를"])?;
                let initial = strip_pipeline_suffix(initial, &["~부터"])?;
                let callable = strip_pipeline_suffix(callable, &["~로", "~으로"])?;
                format!("({list}, {initial}, {callable}) {func}")
            }
            _ => continue,
        };
        let prefix_len = line.len().saturating_sub(line.trim_start().len());
        let prefix = &line[..prefix_len];
        let mut rendered = String::new();
        rendered.push_str(prefix);
        rendered.push_str(&lhs_prefix);
        rendered.push_str(&lowered);
        if dot_suffix {
            rendered.push('.');
        }
        return Some(rendered);
    }
    None
}

fn parse_pipeline_groups(input: &str) -> Option<Vec<String>> {
    let chars: Vec<char> = input.chars().collect();
    let mut groups = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        if chars[i] != '(' {
            return None;
        }
        let start = i + 1;
        i += 1;
        let mut depth = 1usize;
        let mut in_string = false;
        let mut escape = false;
        while i < chars.len() {
            let ch = chars[i];
            if in_string {
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
            match ch {
                '"' => in_string = true,
                '(' | '[' | '{' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let inner: String = chars[start..i].iter().collect();
                        groups.push(inner.trim().to_string());
                        i += 1;
                        break;
                    }
                }
                ']' | '}' => depth = depth.saturating_sub(1),
                _ => {}
            }
            i += 1;
        }
        if depth != 0 {
            return None;
        }
    }
    if groups.is_empty() {
        None
    } else {
        Some(groups)
    }
}

fn strip_pipeline_suffix<'a>(value: &'a str, suffixes: &[&str]) -> Option<&'a str> {
    let trimmed = value.trim();
    for suffix in suffixes {
        if let Some(prefix) = trimmed.strip_suffix(suffix) {
            return Some(prefix.trim());
        }
    }
    None
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

fn lower_connect_endpoint_relation(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut pending = Vec::new();
    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line_frontdoor(chunk);
        if let Some(assignment) = parse_connect_endpoint_assignment_line(line, newline) {
            if pending
                .last()
                .is_some_and(|previous: &ConnectEndpointAssignmentLine<'_>| {
                    previous.target == assignment.target && previous.indent == assignment.indent
                })
            {
                pending.push(assignment);
            } else {
                flush_connect_endpoint_assignment_block(&mut out, &mut pending);
                pending.push(assignment);
            }
            continue;
        }
        flush_connect_endpoint_assignment_block(&mut out, &mut pending);
        if let Some(lowered) = lower_connect_endpoint_relation_line(line) {
            out.push_str(&lowered);
        } else {
            out.push_str(line);
        }
        out.push_str(newline);
    }
    flush_connect_endpoint_assignment_block(&mut out, &mut pending);
    if !source.ends_with('\n') && source.is_empty() {
        return source.to_string();
    }
    out
}

struct ConnectEndpointAssignmentLine<'a> {
    indent: &'a str,
    target: &'a str,
    relation: ConnectEndpointRelationSet<'a>,
    newline: &'a str,
}

fn parse_connect_endpoint_assignment_line<'a>(
    line: &'a str,
    newline: &'a str,
) -> Option<ConnectEndpointAssignmentLine<'a>> {
    let indent_len = line.len() - line.trim_start().len();
    let indent = &line[..indent_len];
    let trimmed = line.trim();
    let body = trimmed.strip_suffix('.')?.trim_end();
    let (target, rhs) = body.split_once("<-")?;
    let target = target.trim();
    if target.is_empty() {
        return None;
    }
    let relation = parse_connect_endpoint_relation(rhs.trim())?;
    Some(ConnectEndpointAssignmentLine {
        indent,
        target,
        relation,
        newline,
    })
}

fn flush_connect_endpoint_assignment_block(
    out: &mut String,
    pending: &mut Vec<ConnectEndpointAssignmentLine<'_>>,
) {
    if pending.is_empty() {
        return;
    }
    if pending.len() == 1 {
        let assignment = pending.pop().expect("pending assignment");
        out.push_str(assignment.indent);
        out.push_str(assignment.target);
        out.push_str(" <- ");
        out.push_str(&format_connect_endpoint_pack(&assignment.relation));
        out.push('.');
        out.push_str(assignment.newline);
        return;
    }

    let first = &pending[0];
    let relations = pending
        .iter()
        .map(|assignment| format_connect_endpoint_pack(&assignment.relation))
        .collect::<Vec<_>>()
        .join(", ");
    let pack = format!(
        "(__이음관계종류: \"endpoint_statement_set\", 대상: \"{}\", 개수: {}, 이음들: ({}) 차림)",
        escape_frontdoor_string(first.target),
        pending.len(),
        relations
    );
    out.push_str(first.indent);
    out.push_str(first.target);
    out.push_str(" <- ");
    out.push_str(&pack);
    out.push('.');
    out.push_str(
        pending
            .last()
            .map(|assignment| assignment.newline)
            .unwrap_or(""),
    );
    pending.clear();
}

fn lower_connect_endpoint_relation_line(line: &str) -> Option<String> {
    let indent_len = line.len() - line.trim_start().len();
    let indent = &line[..indent_len];
    let trimmed = line.trim();
    let body = trimmed.strip_suffix('.')?.trim_end();
    let (prefix, rhs) = if let Some((target, expr)) = body.split_once("<-") {
        (Some(target.trim()), expr.trim())
    } else {
        (None, body)
    };
    let relation = parse_connect_endpoint_relation(rhs)?;
    let pack = format_connect_endpoint_pack(&relation);
    Some(match prefix {
        Some(target) => format!("{indent}{target} <- {pack}."),
        None => format!("{indent}{pack}."),
    })
}

struct ConnectEndpointRelation<'a> {
    left: &'a str,
    right: &'a str,
    channel: &'a str,
    rule: ConnectEndpointRule,
    carried_property: Option<&'a str>,
    carrier_rule: Option<ConnectEndpointRule>,
}

struct ConnectEndpointRelationSet<'a> {
    left: &'a str,
    right: &'a str,
    relations: Vec<ConnectEndpointRelation<'a>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConnectEndpointRule {
    Equality,
    Flow,
    ReverseFlow,
    CarriedProperty,
}

fn parse_connect_endpoint_relation(input: &str) -> Option<ConnectEndpointRelationSet<'_>> {
    let rest = input.strip_suffix("잇기")?.trim_end();
    let inner_start = rest.rfind('(')?;
    let inner = rest[inner_start + 1..].trim().strip_suffix(')')?.trim();
    let endpoints_part = rest[..inner_start].trim_end();
    let endpoints_part = endpoints_part
        .strip_suffix("을")
        .or_else(|| endpoints_part.strip_suffix("를"))?
        .trim_end();
    let (left, right) = split_connect_endpoints(endpoints_part)?;
    if !left.contains('.') || !right.contains('.') {
        return None;
    }
    let mut relations = Vec::new();
    for segment in inner.split(',') {
        relations.push(parse_connect_endpoint_inner_rule(
            left,
            right,
            segment.trim(),
        )?);
    }
    if relations.is_empty() {
        return None;
    }
    resolve_connect_carried_properties(&mut relations)?;
    Some(ConnectEndpointRelationSet {
        left,
        right,
        relations,
    })
}

fn parse_connect_endpoint_inner_rule<'a>(
    left: &'a str,
    right: &'a str,
    inner: &'a str,
) -> Option<ConnectEndpointRelation<'a>> {
    let (channel, rule) = if let Some(channel) = inner
        .strip_suffix("은 같게")
        .or_else(|| inner.strip_suffix("는 같게"))
    {
        (channel.trim(), ConnectEndpointRule::Equality)
    } else if let Some(channel) = inner
        .strip_suffix("은 흐르게")
        .or_else(|| inner.strip_suffix("는 흐르게"))
    {
        (channel.trim(), ConnectEndpointRule::Flow)
    } else if let Some(channel) = inner
        .strip_suffix("은 거슬러 흐르게")
        .or_else(|| inner.strip_suffix("는 거슬러 흐르게"))
    {
        (channel.trim(), ConnectEndpointRule::ReverseFlow)
    } else if let Some((_property, carrier_channel)) = parse_connect_carried_property_inner(inner) {
        (carrier_channel, ConnectEndpointRule::CarriedProperty)
    } else {
        return None;
    };
    if channel.is_empty() || channel.contains(char::is_whitespace) {
        return None;
    }
    let carried_property = if let ConnectEndpointRule::CarriedProperty = rule {
        let (property, _) = parse_connect_carried_property_inner(inner)?;
        if property.is_empty() || property.contains(char::is_whitespace) {
            return None;
        }
        Some(property)
    } else {
        None
    };
    Some(ConnectEndpointRelation {
        left,
        right,
        channel,
        rule,
        carried_property,
        carrier_rule: None,
    })
}

fn parse_connect_carried_property_inner(input: &str) -> Option<(&str, &str)> {
    let base = input.strip_suffix("에 실리게")?.trim_end();
    for marker in ["이 ", "가 "] {
        if let Some((property, carrier_channel)) = base.split_once(marker) {
            let property = property.trim();
            let carrier_channel = carrier_channel.trim();
            if !property.is_empty() && !carrier_channel.is_empty() {
                return Some((property, carrier_channel));
            }
        }
    }
    None
}

fn resolve_connect_carried_properties(relations: &mut [ConnectEndpointRelation<'_>]) -> Option<()> {
    for idx in 0..relations.len() {
        if relations[idx].rule != ConnectEndpointRule::CarriedProperty {
            continue;
        }
        let carrier_rules = relations
            .iter()
            .filter(|relation| {
                relation.channel == relations[idx].channel
                    && matches!(
                        relation.rule,
                        ConnectEndpointRule::Flow | ConnectEndpointRule::ReverseFlow
                    )
            })
            .map(|relation| relation.rule)
            .collect::<Vec<_>>();
        let [carrier_rule] = carrier_rules.as_slice() else {
            return None;
        };
        relations[idx].carrier_rule = Some(*carrier_rule);
    }
    Some(())
}

fn split_connect_endpoints(input: &str) -> Option<(&str, &str)> {
    for marker in ["과 ", "와 "] {
        if let Some((left, right)) = input.split_once(marker) {
            let left = left.trim();
            let right = right.trim();
            if !left.is_empty() && !right.is_empty() {
                return Some((left, right));
            }
        }
    }
    None
}

fn format_connect_endpoint_pack(relation_set: &ConnectEndpointRelationSet<'_>) -> String {
    if relation_set.relations.len() == 1 {
        return format_connect_endpoint_relation_pack(&relation_set.relations[0]);
    }
    let relations = relation_set
        .relations
        .iter()
        .map(format_connect_endpoint_relation_pack)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "(__이음관계종류: \"endpoint_relation_set\", 왼쪽끝: \"{}\", 오른쪽끝: \"{}\", 관계들: ({}) 차림)",
        escape_frontdoor_string(relation_set.left),
        escape_frontdoor_string(relation_set.right),
        relations
    )
}

fn format_connect_endpoint_relation_pack(relation: &ConnectEndpointRelation<'_>) -> String {
    let left_path = format!("{}.{}", relation.left, relation.channel);
    let right_path = format!("{}.{}", relation.right, relation.channel);
    match relation.rule {
        ConnectEndpointRule::Equality => format!(
            "(__이음관계종류: \"endpoint_equality\", 왼쪽: \"{}\", 오른쪽: \"{}\", 규칙: \"같게\", 채널: \"{}\")",
            escape_frontdoor_string(&left_path),
            escape_frontdoor_string(&right_path),
            escape_frontdoor_string(relation.channel)
        ),
        ConnectEndpointRule::Flow | ConnectEndpointRule::ReverseFlow => {
            let (rule, direction) = match relation.rule {
                ConnectEndpointRule::Flow => ("흐르게", "왼쪽에서오른쪽"),
                ConnectEndpointRule::ReverseFlow => ("거슬러 흐르게", "오른쪽에서왼쪽"),
                ConnectEndpointRule::Equality | ConnectEndpointRule::CarriedProperty => {
                    unreachable!("non-flow handled elsewhere")
                }
            };
            format!(
                "(__이음관계종류: \"endpoint_flow\", 왼쪽: \"{}\", 오른쪽: \"{}\", 규칙: \"{}\", 채널: \"{}\", 부호규약: \"left_plus_right_zero\", 방향: \"{}\")",
                escape_frontdoor_string(&left_path),
                escape_frontdoor_string(&right_path),
                escape_frontdoor_string(rule),
                escape_frontdoor_string(relation.channel),
                escape_frontdoor_string(direction)
            )
        }
        ConnectEndpointRule::CarriedProperty => {
            let (carrier_rule, carrier_direction) = match relation
                .carrier_rule
                .expect("carried property carrier rule resolved")
            {
                ConnectEndpointRule::Flow => ("흐르게", "왼쪽에서오른쪽"),
                ConnectEndpointRule::ReverseFlow => ("거슬러 흐르게", "오른쪽에서왼쪽"),
                ConnectEndpointRule::Equality | ConnectEndpointRule::CarriedProperty => {
                    unreachable!("carried property carrier must be flow")
                }
            };
            format!(
                "(__이음관계종류: \"endpoint_carried_property\", 왼쪽운반자: \"{}\", 오른쪽운반자: \"{}\", 속성: \"{}\", 운반채널: \"{}\", 규칙: \"실리게\", 운반규칙: \"{}\", 운반방향: \"{}\")",
                escape_frontdoor_string(&left_path),
                escape_frontdoor_string(&right_path),
                escape_frontdoor_string(
                    relation
                        .carried_property
                        .expect("carried property value parsed")
                ),
                escape_frontdoor_string(relation.channel),
                escape_frontdoor_string(carrier_rule),
                escape_frontdoor_string(carrier_direction)
            )
        }
    }
}

fn format_connect_endpoint_relation_seum_rows(
    relation_set: &ConnectEndpointRelationSet<'_>,
) -> Option<Vec<String>> {
    let mut rows = Vec::with_capacity(relation_set.relations.len());
    for relation in &relation_set.relations {
        let left_path = format!("{}.{}", relation.left, relation.channel);
        let right_path = format!("{}.{}", relation.right, relation.channel);
        match relation.rule {
            ConnectEndpointRule::Equality => {
                rows.push(format!("{left_path} =:= {right_path}."));
            }
            ConnectEndpointRule::Flow | ConnectEndpointRule::ReverseFlow => {
                rows.push(format!("{left_path} + {right_path} =:= 0."));
            }
            ConnectEndpointRule::CarriedProperty => return None,
        }
    }
    Some(rows)
}

fn escape_frontdoor_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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
    fn validate_no_legacy_boim_surface_accepts_boim_block() {
        let source = "보임 { x: 1. }.";
        validate_no_legacy_boim_surface(source).expect("must pass");
        assert!(!has_legacy_boim_surface(source));
    }

    #[test]
    fn validate_no_legacy_root_surface_detects_salim_prefix() {
        let source = "살림.x <- 1.";
        let err = validate_no_legacy_root_surface(source).expect_err("must fail");
        assert!(err.contains("E_SALIM_REMOVED"));
    }

    #[test]
    fn validate_no_legacy_root_surface_detects_batang_prefix() {
        let source = "바탕.x <- 1.";
        let err = validate_no_legacy_root_surface(source).expect_err("must fail");
        assert!(err.contains("E_BATANG_PREFIX_REMOVED"));
    }

    #[test]
    fn validate_no_legacy_root_surface_ignores_batang_euro_phrase() {
        let source = "{ 값 > 0 }인것 바탕으로(알림) 아니면 { }.";
        validate_no_legacy_root_surface(source).expect("must pass");
    }

    #[test]
    fn validate_no_legacy_root_hide_directive_detects_pragma() {
        let source = "#바탕숨김.\nx <- 1.";
        let err = validate_no_legacy_root_hide_directive(source).expect_err("must fail");
        assert!(err.contains("E_PRAGMA_REMOVED"));
    }

    #[test]
    fn validate_no_legacy_range_comment_detects_comment() {
        let source = "g:수 <- 9.8. // 범위(1, 20, 0.1)";
        let err = validate_no_legacy_range_comment(source).expect_err("must fail");
        assert!(err.contains("E_LEGACY_RANGE_SYNTAX"));
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

    #[test]
    fn connect_endpoint_equal_lowers_to_endpoint_relation_pack() {
        let src = "이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(out.contains("__이음관계종류: \"endpoint_equality\""));
        assert!(out.contains("왼쪽: \"전지.양극.전압\""));
        assert!(out.contains("오른쪽: \"전구.왼핀.전압\""));
        assert!(out.contains("규칙: \"같게\""));
        assert!(out.contains("채널: \"전압\""));
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        crate::parse_frontdoor_with_mode(
            &wrapped,
            "connect_endpoint_equal.ddn",
            crate::ParseMode::Strict,
        )
        .expect("lowered endpoint equality must parse");
    }

    #[test]
    fn connect_endpoint_flow_lowers_to_endpoint_flow_pack() {
        let src = "이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(out.contains("__이음관계종류: \"endpoint_flow\""));
        assert!(out.contains("왼쪽: \"전지.양극.전류\""));
        assert!(out.contains("오른쪽: \"전구.왼핀.전류\""));
        assert!(out.contains("규칙: \"흐르게\""));
        assert!(out.contains("채널: \"전류\""));
        assert!(out.contains("부호규약: \"left_plus_right_zero\""));
        assert!(out.contains("방향: \"왼쪽에서오른쪽\""));
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        crate::parse_frontdoor_with_mode(
            &wrapped,
            "connect_endpoint_flow.ddn",
            crate::ParseMode::Strict,
        )
        .expect("lowered endpoint flow must parse");
    }

    #[test]
    fn connect_endpoint_reverse_flow_lowers_to_endpoint_flow_pack() {
        let src = "이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 거슬러 흐르게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(out.contains("__이음관계종류: \"endpoint_flow\""));
        assert!(out.contains("왼쪽: \"가계1.구매끝.돈\""));
        assert!(out.contains("오른쪽: \"장터.소매끝.돈\""));
        assert!(out.contains("규칙: \"거슬러 흐르게\""));
        assert!(out.contains("채널: \"돈\""));
        assert!(out.contains("부호규약: \"left_plus_right_zero\""));
        assert!(out.contains("방향: \"오른쪽에서왼쪽\""));
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        crate::parse_frontdoor_with_mode(
            &wrapped,
            "connect_endpoint_reverse_flow.ddn",
            crate::ParseMode::Strict,
        )
        .expect("lowered endpoint reverse flow must parse");
    }

    #[test]
    fn connect_endpoint_rejects_carried_property_surface() {
        assert_endpoint_connect_rejected(
            "이음관계 <- 전지.양극과 전구.왼핀을 (재화가 돈에 실리게) 잇기.",
        );
    }

    #[test]
    fn connect_endpoint_carried_property_forward_lowers_to_relation_set_pack() {
        let src = "이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(out.contains("__이음관계종류: \"endpoint_relation_set\""));
        assert!(out.contains("__이음관계종류: \"endpoint_flow\""));
        assert!(out.contains("채널: \"대출금\""));
        assert!(out.contains("방향: \"왼쪽에서오른쪽\""));
        assert!(out.contains("__이음관계종류: \"endpoint_carried_property\""));
        assert!(out.contains("왼쪽운반자: \"은행.대출창구.대출금\""));
        assert!(out.contains("오른쪽운반자: \"기업1.차입끝.대출금\""));
        assert!(out.contains("속성: \"위험\""));
        assert!(out.contains("운반채널: \"대출금\""));
        assert!(out.contains("규칙: \"실리게\""));
        assert!(out.contains("운반규칙: \"흐르게\""));
        assert!(out.contains("운반방향: \"왼쪽에서오른쪽\""));
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        crate::parse_frontdoor_with_mode(
            &wrapped,
            "connect_endpoint_carried_property_forward.ddn",
            crate::ParseMode::Strict,
        )
        .expect("lowered endpoint carried property must parse");
    }

    #[test]
    fn connect_endpoint_carried_property_reverse_lowers_to_relation_set_pack() {
        let src = "이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 거슬러 흐르게, 재화가 돈에 실리게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(out.contains("__이음관계종류: \"endpoint_relation_set\""));
        assert!(out.contains("규칙: \"거슬러 흐르게\""));
        assert!(out.contains("방향: \"오른쪽에서왼쪽\""));
        assert!(out.contains("__이음관계종류: \"endpoint_carried_property\""));
        assert!(out.contains("왼쪽운반자: \"가계1.구매끝.돈\""));
        assert!(out.contains("오른쪽운반자: \"장터.소매끝.돈\""));
        assert!(out.contains("속성: \"재화\""));
        assert!(out.contains("운반규칙: \"거슬러 흐르게\""));
        assert!(out.contains("운반방향: \"오른쪽에서왼쪽\""));
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        crate::parse_frontdoor_with_mode(
            &wrapped,
            "connect_endpoint_carried_property_reverse.ddn",
            crate::ParseMode::Strict,
        )
        .expect("lowered endpoint reverse carried property must parse");
    }

    #[test]
    fn connect_endpoint_multi_inner_lowers_to_relation_set_pack() {
        let src = "이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(out.contains("__이음관계종류: \"endpoint_relation_set\""));
        assert!(out.contains("왼쪽끝: \"전지.양극\""));
        assert!(out.contains("오른쪽끝: \"전구.왼핀\""));
        assert!(out.contains("관계들: ("));
        assert!(out.contains("__이음관계종류: \"endpoint_equality\""));
        assert!(out.contains("왼쪽: \"전지.양극.전압\""));
        assert!(out.contains("오른쪽: \"전구.왼핀.전압\""));
        assert!(out.contains("__이음관계종류: \"endpoint_flow\""));
        assert!(out.contains("왼쪽: \"전지.양극.전류\""));
        assert!(out.contains("오른쪽: \"전구.왼핀.전류\""));
        assert!(out.contains("방향: \"왼쪽에서오른쪽\""));
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        crate::parse_frontdoor_with_mode(
            &wrapped,
            "connect_endpoint_multi_inner.ddn",
            crate::ParseMode::Strict,
        )
        .expect("lowered endpoint multi inner relation set must parse");
    }

    #[test]
    fn connect_endpoint_multi_inner_exposes_seum_rows() {
        let src = "전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.";
        let rows = connect_endpoint_relation_seum_rows(src).expect("seum rows");
        assert_eq!(
            rows,
            vec![
                "전지.양극.전압 =:= 전구.왼핀.전압.",
                "전지.양극.전류 + 전구.왼핀.전류 =:= 0.",
            ]
        );
    }

    #[test]
    fn connect_endpoint_seum_rows_accept_assignment_surface() {
        let src = "이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.";
        let rows = connect_endpoint_relation_seum_rows(src).expect("seum rows");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "전지.양극.전압 =:= 전구.왼핀.전압.");
        assert_eq!(rows[1], "전지.양극.전류 + 전구.왼핀.전류 =:= 0.");
    }

    #[test]
    fn connect_block_surface_has_no_seum_rows() {
        assert!(connect_endpoint_relation_seum_rows("잇기 { 전압은 같게. }.").is_none());
    }

    #[test]
    fn connect_endpoint_multi_inner_econ_lowers_in_source_order() {
        let src = "이음관계 <- 가계1.구매끝과 장터.소매끝을 (체결값은 같게, 재화는 흐르게, 돈은 거슬러 흐르게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        let equality_idx = out.find("채널: \"체결값\"").expect("equality channel");
        let flow_idx = out.find("채널: \"재화\"").expect("flow channel");
        let reverse_idx = out.find("채널: \"돈\"").expect("reverse channel");
        assert!(equality_idx < flow_idx && flow_idx < reverse_idx);
        assert!(out.contains("__이음관계종류: \"endpoint_relation_set\""));
        assert!(out.contains("방향: \"왼쪽에서오른쪽\""));
        assert!(out.contains("방향: \"오른쪽에서왼쪽\""));
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        crate::parse_frontdoor_with_mode(
            &wrapped,
            "connect_endpoint_multi_inner_econ.ddn",
            crate::ParseMode::Strict,
        )
        .expect("lowered endpoint economic multi inner relation set must parse");
    }

    #[test]
    fn connect_endpoint_statement_append_same_pair_lowers_to_statement_set() {
        let src = "이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.\n이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(out.contains("__이음관계종류: \"endpoint_statement_set\""));
        assert!(out.contains("대상: \"이음관계\""));
        assert!(out.contains("개수: 2"));
        assert!(out.contains("__이음관계종류: \"endpoint_equality\""));
        assert!(out.contains("__이음관계종류: \"endpoint_flow\""));
        assert_eq!(out.matches("이음관계 <-").count(), 1);
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        let prepared = preprocess_frontdoor_source(&wrapped);
        crate::parse_frontdoor_with_mode(
            &prepared,
            "connect_endpoint_statement_append_same_pair.ddn",
            crate::ParseMode::Strict,
        )
        .expect("lowered endpoint statement append must parse");
    }

    #[test]
    fn connect_endpoint_statement_append_mixed_pair_lowers_to_statement_set() {
        let src = "이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.\n이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(out.contains("__이음관계종류: \"endpoint_statement_set\""));
        assert!(out.contains("전지.양극.전압"));
        assert!(out.contains("은행.대출창구.대출금"));
        assert!(out.contains("__이음관계종류: \"endpoint_relation_set\""));
        assert!(out.contains("__이음관계종류: \"endpoint_carried_property\""));
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        let prepared = preprocess_frontdoor_source(&wrapped);
        crate::parse_frontdoor_with_mode(
            &prepared,
            "connect_endpoint_statement_append_mixed_pair.ddn",
            crate::ParseMode::Strict,
        )
        .expect("lowered mixed endpoint statement append must parse");
    }

    #[test]
    fn connect_endpoint_rejects_statement_append_boundary_non_endpoint_breaks_block() {
        let src = "이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.\ndt <- 1.\n이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(!out.contains("__이음관계종류: \"endpoint_statement_set\""));
        assert_eq!(out.matches("이음관계 <-").count(), 2);
        assert!(out.contains("__이음관계종류: \"endpoint_equality\""));
        assert!(out.contains("__이음관계종류: \"endpoint_flow\""));
    }

    #[test]
    fn connect_endpoint_rejects_statement_append_boundary_other_target_breaks_block() {
        let src = "이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.\n다른이음 <- 전지.음극과 전구.오른핀을 (전류는 흐르게) 잇기.\n이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.";
        let out = preprocess_frontdoor_source(src);
        assert!(!out.contains("__이음관계종류: \"endpoint_statement_set\""));
        assert_eq!(out.matches("이음관계 <-").count(), 2);
        assert_eq!(out.matches("다른이음 <-").count(), 1);
    }

    #[test]
    fn connect_endpoint_rejects_unsupported_multi_inner_sentence_surface() {
        assert_endpoint_connect_rejected(
            "이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 재화가 돈에 실리게) 잇기.",
        );
    }

    #[test]
    fn connect_endpoint_rejects_duplicate_carrier_flow_surface() {
        assert_endpoint_connect_rejected(
            "이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 흐르게, 돈은 거슬러 흐르게, 재화가 돈에 실리게) 잇기.",
        );
    }

    #[test]
    fn connect_endpoint_rejects_empty_multi_inner_sentence_surface() {
        assert_endpoint_connect_rejected(
            "이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, ) 잇기.",
        );
    }

    #[test]
    fn connect_endpoint_rejects_whole_object_shorthand_surface() {
        assert_endpoint_connect_rejected("이음관계 <- 전구와 전지를 (전압은 같게) 잇기.");
    }

    fn assert_endpoint_connect_rejected(src: &str) {
        let out = preprocess_frontdoor_source(src);
        assert_eq!(out, src);
        let wrapped = format!("매틱:움직씨 = {{\n  {src}\n}}");
        let err = crate::parse_frontdoor_with_mode(
            &wrapped,
            "connect_endpoint_unsupported.ddn",
            crate::ParseMode::Strict,
        )
        .expect_err("unsupported endpoint connect is not V1C surface");
        assert!(!err.message.is_empty());
    }
}
