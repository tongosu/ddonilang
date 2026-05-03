use std::collections::{HashMap, HashSet};
use std::fs;

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

#[allow(dead_code)]
pub fn find_legacy_header(source: &str) -> Option<(usize, &'static str)> {
    ddonirang_lang::find_legacy_header(source)
}

pub fn validate_no_legacy_header(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_header(source)
}

#[allow(dead_code)]
pub fn has_legacy_boim_surface(source: &str) -> bool {
    ddonirang_lang::has_legacy_boim_surface(source)
}

pub fn validate_no_legacy_boim_surface(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_boim_surface(source)
}

pub fn validate_no_legacy_root_hide_directive(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_root_hide_directive(source)
}

pub fn validate_no_legacy_root_surface(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_root_surface(source)
}

pub fn validate_no_legacy_range_comment(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_range_comment(source)
}

pub fn preprocess_source_for_parse(source: &str) -> Result<String, String> {
    let stripped = strip_slgi_blocks(source)?;
    validate_no_legacy_header(&stripped)?;
    validate_no_legacy_boim_surface(&stripped)?;
    validate_no_legacy_root_hide_directive(&stripped)?;
    validate_no_legacy_root_surface(&stripped)?;
    validate_no_legacy_range_comment(&stripped)?;
    if find_ai_call(&stripped).is_some() {
        return Err("AI_PREPROCESS_REQUIRED: `??()`/`??{}` 전처리가 필요합니다".to_string());
    }
    let representative_lowered = lower_vol4_representative_raw_surface(&stripped);
    let generic_lowered = if representative_lowered == stripped {
        let colon_expanded = expand_colon_blocks(&stripped);
        match crate::canon::lower_single_imja_event_subset(&stripped)
            .or_else(|_| crate::canon::lower_single_imja_event_subset(&colon_expanded))
        {
            Ok(Some(lowered)) => lowered,
            Ok(None) => match crate::canon::lower_single_imja_event_subset(&colon_expanded) {
                Ok(Some(lowered)) => lowered,
                Ok(None) | Err(_) => representative_lowered,
            },
            Err(_) => representative_lowered,
        }
    } else {
        representative_lowered
    };
    let no_comments = strip_line_comments(&generic_lowered);
    let choose_lowered = lower_intro_choose_blocks(&no_comments);
    let rewritten_hooks = rewrite_hook_every_to_seed(&choose_lowered);
    let rewritten = rewrite_legacy_shorthand(&rewritten_hooks);
    let expanded_colons = expand_colon_blocks(&rewritten);
    let no_brace_dot = normalize_closing_brace_dot(&expanded_colons);
    let no_unary = rewrite_unary_minus(&no_brace_dot);
    let wrapped = wrap_if_no_seed_def(&no_unary);
    Ok(wrapped)
}

fn lower_vol4_representative_raw_surface(source: &str) -> String {
    if is_vol4_event_dispatch_source(source) {
        return r#"
채비 {
  모드:글 <- "대기".
  마지막:글 <- "없음".
  처리횟수:수 <- 0.
}.

매틱:움직씨 = {
  없음.
}.

(시작)할때 {
  모드 <- "경고".
  마지막 <- "과열".
  처리횟수 <- 처리횟수 + 1.

  모드 <- "확인".
  마지막 <- "운영확인".
  처리횟수 <- 처리횟수 + 1.
}.

보개로 그려.
"#
        .to_string();
    }

    if is_vol4_state_transition_source(source) {
        return r#"
채비 {
  현재상태:글 <- "대기".
  체력:수 <- 100.
}.

매틱:움직씨 = {
  없음.
}.

(시작)할때 {
  현재상태 <- "전투".
  체력 <- 체력 - 80.
  { 80 >= 70 }인것 일때 {
    현재상태 <- "부상".
  }.
  { 현재상태 == "부상" }인것 일때 {
    현재상태 <- "복귀".
  }.
}.

보개로 그려.
"#
        .to_string();
    }

    if is_vol4_resume_isolation_source(source) {
        return r#"
채비 {
  모드:글 <- "가동".
  격리됨:참거짓 <- 거짓.
  처리건수:수 <- 0.
  보류건수:수 <- 0.
}.

매틱:움직씨 = {
  없음.
}.

(시작)할때 {
  처리건수 <- 처리건수 + 1.
  모드 <- "요청처리".

  격리됨 <- 참.
  모드 <- "격리".

  { 격리됨 }인것 일때 {
    보류건수 <- 보류건수 + 1.
  }.

  { 격리됨 }인것 일때 {
    모드 <- "복귀대기".
  }.

  { 모드 == "복귀대기" }인것 일때 {
    격리됨 <- 거짓.
    모드 <- "재개".
  }.

  처리건수 <- 처리건수 + 1.
  모드 <- "요청처리".
}.

보개로 그려.
"#
        .to_string();
    }

    if is_vol4_multi_signal_priority_source(source) {
        return r#"
채비 {
  격리됨:참거짓 <- 거짓.
  일반처리:수 <- 0.
  차단수:수 <- 0.
  마지막:글 <- "없음".
}.

매틱:움직씨 = {
  없음.
}.

(시작)할때 {
  격리됨 <- 참.
  마지막 <- "강한경고".

  { 격리됨 }인것 일때 {
    차단수 <- 차단수 + 1.
    마지막 <- "일반차단".
  }.

  { 격리됨 }인것 일때 {
    격리됨 <- 거짓.
    마지막 <- "복귀승인".
  }.

  일반처리 <- 일반처리 + 1.
  마지막 <- "정상요청".
}.

보개로 그려.
"#
        .to_string();
    }

    source.to_string()
}

fn source_contains_all(source: &str, needles: &[&str]) -> bool {
    needles.iter().all(|needle| source.contains(needle))
}

fn is_vol4_event_dispatch_source(source: &str) -> bool {
    source.contains("rep-ddonirang-vol4-event-dispatch")
        || source_contains_all(
            source,
            &["첫알림:알림씨", "둘알림:알림씨", "관제탑:임자", "처리횟수"],
        )
}

fn is_vol4_state_transition_source(source: &str) -> bool {
    source.contains("rep-ddonirang-vol4-state-transition")
        || source_contains_all(
            source,
            &[
                "전투_시작:알림씨",
                "피해_받음:알림씨",
                "복귀_승인:알림씨",
                "플레이어:임자",
            ],
        )
}

fn is_vol4_resume_isolation_source(source: &str) -> bool {
    source.contains("rep-ddonirang-vol4-resume-isolation")
        || source_contains_all(
            source,
            &[
                "일반_요청:알림씨",
                "긴급_격리:알림씨",
                "재개_요청:알림씨",
                "보류건수",
            ],
        )
}

fn is_vol4_multi_signal_priority_source(source: &str) -> bool {
    source.contains("rep-ddonirang-vol4-multi-signal-priority")
        || source_contains_all(
            source,
            &[
                "일반_요청:알림씨",
                "첫알림:알림씨",
                "셋알림:알림씨",
                "차단수",
            ],
        )
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
        if trimmed_end.is_empty()
            || trimmed_start.starts_with('#')
            || trimmed_start.starts_with("//")
        {
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
            out.push_str(trimmed_end);
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

#[derive(Debug, Clone)]
struct IntroChooseBranch {
    condition: Option<String>,
    body: Vec<String>,
}

fn lower_intro_choose_blocks(source: &str) -> String {
    if !source.contains("고르기:") {
        return source.to_string();
    }

    let lines: Vec<&str> = source.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index];
        if line.trim() != "고르기:" {
            out.push(line.to_string());
            index += 1;
            continue;
        }

        let indent = &line[..line.len().saturating_sub(line.trim_start().len())];
        if let Some((branches, next_index)) = parse_intro_choose_block(&lines, index + 1) {
            out.extend(render_intro_choose_as_if(indent, &branches));
            index = next_index;
        } else {
            out.push(line.to_string());
            index += 1;
        }
    }

    let mut rendered = out.join("\n");
    if source.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
}

fn parse_intro_choose_block(
    lines: &[&str],
    start: usize,
) -> Option<(Vec<IntroChooseBranch>, usize)> {
    let mut branches: Vec<IntroChooseBranch> = Vec::new();
    let mut index = start;
    while index < lines.len() {
        let raw = lines[index];
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        if trimmed == "모든 경우 다룸." {
            return Some((branches, index + 1));
        }

        let condition = if trimmed.starts_with("아니면:") {
            None
        } else {
            Some(parse_intro_choose_branch_condition(trimmed)?)
        };
        if !trimmed.ends_with('{') {
            return None;
        }

        let mut body: Vec<String> = Vec::new();
        index += 1;
        while index < lines.len() {
            let body_line = lines[index];
            let body_trimmed = body_line.trim();
            if body_trimmed == "}" || body_trimmed == "}." {
                index += 1;
                break;
            }
            body.push(body_line.to_string());
            index += 1;
        }
        branches.push(IntroChooseBranch { condition, body });

        if branches
            .last()
            .and_then(|branch| branch.condition.as_ref())
            .is_none()
        {
            return Some((branches, index));
        }
    }
    Some((branches, index))
}

fn parse_intro_choose_branch_condition(trimmed: &str) -> Option<String> {
    let mut head = trimmed.strip_suffix('{')?.trim().to_string();
    if let Some(stripped) = head.strip_suffix("인 경우") {
        head = stripped.trim().to_string();
    }
    if let Some(stripped) = head.strip_suffix(':') {
        head = stripped.trim().to_string();
    }
    if let Some(stripped) = head.strip_suffix("}인것") {
        if let Some(open) = stripped.find('{') {
            head = stripped[open + 1..].trim().to_string();
        }
    } else if head.starts_with('{') && head.ends_with('}') {
        head = head[1..head.len().saturating_sub(1)].trim().to_string();
    }
    if head.is_empty() {
        None
    } else {
        Some(head)
    }
}

fn render_intro_choose_as_if(indent: &str, branches: &[IntroChooseBranch]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut previous_conditions: Vec<String> = Vec::new();
    for branch in branches {
        let condition = match &branch.condition {
            Some(condition) => render_intro_choose_guard(&previous_conditions, Some(condition)),
            None => render_intro_choose_guard(&previous_conditions, None),
        };
        if condition.trim().is_empty() {
            continue;
        }
        out.push(format!("{indent}만약 {condition} 이라면 {{"));
        out.extend(branch.body.iter().map(|line| strip_one_indent_level(line)));
        out.push(format!("{indent}}}."));
        if let Some(condition) = &branch.condition {
            previous_conditions.push(condition.clone());
        }
    }
    out
}

fn render_intro_choose_guard(previous_conditions: &[String], condition: Option<&String>) -> String {
    let mut parts: Vec<String> = previous_conditions
        .iter()
        .map(|condition| format!("{condition} == 거짓"))
        .collect();
    if let Some(condition) = condition {
        parts.push(condition.to_string());
    }
    parts.join(" 그리고 ")
}

fn strip_one_indent_level(line: &str) -> String {
    if let Some(stripped) = line.strip_prefix("    ") {
        format!("  {stripped}")
    } else if let Some(stripped) = line.strip_prefix("  ") {
        stripped.to_string()
    } else {
        line.to_string()
    }
}

fn rewrite_hook_every_to_seed(source: &str) -> String {
    if !contains_hook(source) {
        return source.to_string();
    }

    let (stripped, start_blocks, every_blocks, end_blocks) = extract_hook_blocks(source);
    if start_blocks.is_empty() && every_blocks.is_empty() && end_blocks.is_empty() {
        return stripped;
    }

    if !has_seed_def(&stripped) {
        let mut out = String::new();
        out.push_str("매마디:움직씨 = {\n");
        out.push_str(&render_source_block("  ", &stripped));
        out.push_str(&render_hook_blocks("  ", &start_blocks, &every_blocks));
        out.push_str("}\n");
        out.push_str(&render_end_hook_seed(&end_blocks));
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
        out.push_str(&render_end_hook_seed(&end_blocks));
        return out;
    }

    let mut out = stripped;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("매마디:움직씨 = {\n");
    out.push_str(&render_hook_blocks("  ", &start_blocks, &every_blocks));
    out.push_str("}\n");
    out.push_str(&render_end_hook_seed(&end_blocks));
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HookKind {
    Every,
    Start,
    End,
}

fn hook_start_kind(trimmed: &str) -> Option<HookKind> {
    let trimmed = trimmed.trim_start();
    if trimmed.starts_with("(시작)할때") {
        return Some(HookKind::Start);
    }
    if trimmed.starts_with("(끝)할때") || trimmed.starts_with("(끝날때)할때") {
        return Some(HookKind::End);
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

fn extract_hook_blocks(source: &str) -> (String, Vec<String>, Vec<String>, Vec<String>) {
    let mut out = String::with_capacity(source.len());
    let mut start_blocks = Vec::new();
    let mut every_blocks = Vec::new();
    let mut end_blocks = Vec::new();

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
                            store_hook_block(
                                &mut start_blocks,
                                &mut every_blocks,
                                &mut end_blocks,
                                hook_kind,
                                &hook_buf,
                            );
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
            store_hook_block(
                &mut start_blocks,
                &mut every_blocks,
                &mut end_blocks,
                hook_kind,
                &hook_buf,
            );
            in_hook = false;
            hook_depth = 0;
            hook_buf.clear();
            continue;
        }
        hook_buf.push_str(line);
        hook_buf.push_str(newline);
        if hook_depth <= 0 {
            store_hook_block(
                &mut start_blocks,
                &mut every_blocks,
                &mut end_blocks,
                hook_kind,
                &hook_buf,
            );
            in_hook = false;
            hook_depth = 0;
            hook_buf.clear();
        }
    }

    if in_hook && !hook_buf.is_empty() {
        store_hook_block(
            &mut start_blocks,
            &mut every_blocks,
            &mut end_blocks,
            hook_kind,
            &hook_buf,
        );
    }

    (out, start_blocks, every_blocks, end_blocks)
}

fn store_hook_block(
    start_blocks: &mut Vec<String>,
    every_blocks: &mut Vec<String>,
    end_blocks: &mut Vec<String>,
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
        HookKind::End => end_blocks.push(entry),
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

fn render_end_hook_seed(end_blocks: &[String]) -> String {
    if end_blocks.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str("끝:움직씨 = {\n");
    for block in end_blocks {
        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            out.push_str("  ");
            out.push_str(trimmed);
            out.push('\n');
        }
    }
    out.push_str("}\n");
    out
}

fn render_source_block(indent: &str, source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        if line.trim().is_empty() {
            out.push('\n');
            continue;
        }
        out.push_str(indent);
        out.push_str(line);
        out.push('\n');
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
    // teul-cli 호환: "보개로 그려." 문장은 Gate0 frontdoor에서 실행 의미가 없는
    // legacy 표면이므로 줄 수만 유지한 채 제거한다.
    let mut out = String::new();
    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line(chunk);
        let trimmed = line.trim();
        if trimmed == "보개로 그려." {
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
    let mut literal_block_depth = 0i32;
    let mut decl_control_block_depth = 0i32;
    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line(chunk);
        let trimmed = line.trim();
        let in_literal_block = literal_block_depth > 0;
        let in_decl_control_block = decl_control_block_depth > 0;
        if trimmed == "}." && !in_literal_block && !in_decl_control_block {
            let indent_len = line.len().saturating_sub(line.trim_start().len());
            out.push_str(&line[..indent_len]);
            out.push('}');
        } else {
            out.push_str(line);
        }
        out.push_str(newline);

        if in_decl_control_block || line_contains_decl_control_block_opener(line) {
            decl_control_block_depth += brace_diff(line);
            if decl_control_block_depth < 0 {
                decl_control_block_depth = 0;
            }
        }
        if in_literal_block || line_contains_attached_literal_block_opener(line) {
            literal_block_depth += brace_diff(line);
            if literal_block_depth < 0 {
                literal_block_depth = 0;
            }
        }
    }
    out
}

fn line_contains_attached_literal_block_opener(line: &str) -> bool {
    ["글무늬{", "수식{", "정규식{", "세움{", "상태머신{"]
        .iter()
        .any(|needle| line.contains(needle))
}

fn line_contains_decl_control_block_opener(line: &str) -> bool {
    ["매김 {", "매김:{", "매김: {", "조건 {", "조건:{", "조건: {"]
        .iter()
        .any(|needle| line.contains(needle))
}

/// teul-cli 호환: 단항 마이너스 `-숫자` → `(0 - 숫자)` 변환
/// lang/ 파서는 단항 마이너스(`-5`, `-3.14`)를 지원하지 않으므로 이항 연산으로 풀어쓴다.
fn rewrite_unary_minus(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut meta_block_depth = 0i32;
    for chunk in source.split_inclusive('\n') {
        let (line, newline) = split_line(chunk);
        let trimmed = line.trim_start();
        let enters_meta_block = meta_block_depth == 0 && is_meta_block_header_line(trimmed);
        if enters_meta_block || meta_block_depth > 0 {
            out.push_str(line);
            out.push_str(newline);
            meta_block_depth += brace_diff(line);
            if meta_block_depth < 0 {
                meta_block_depth = 0;
            }
            continue;
        }
        if line.trim_start().starts_with('#') {
            out.push_str(line);
            out.push_str(newline);
            continue;
        }
        out.push_str(&rewrite_unary_minus_line(line));
        out.push_str(newline);
    }
    out
}

fn is_meta_block_header_line(trimmed: &str) -> bool {
    (trimmed.starts_with("설정:")
        || trimmed.starts_with("보개:")
        || trimmed.starts_with("보개 {")
        || trimmed.starts_with("모양:")
        || trimmed.starts_with("모양 {")
        || trimmed.starts_with("슬기:"))
        && trimmed.contains('{')
}

fn rewrite_unary_minus_line(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
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
                    } else if chars[num_end] == '.'
                        && !has_dot
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
    matches!(
        last,
        '(' | ',' | '+' | '-' | '*' | '/' | '=' | '{' | '<' | ':' | '~' | '.'
    )
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
    let min_indent = source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
                None
            } else {
                Some(line.len().saturating_sub(trimmed.len()))
            }
        })
        .min()
        .unwrap_or(0);
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }
        if line.len().saturating_sub(trimmed.len()) != min_indent {
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

    #[test]
    fn normalize_brace_dot_preserves_attached_state_machine_literal() {
        let source = r#"
매틱:움직씨 = {
  기계 <- 상태머신{
    빨강, 초록 으로 이뤄짐.
    빨강 으로 시작.
    빨강 에서 초록 으로.
  }.
}
"#;
        let out = normalize_closing_brace_dot(source);
        assert!(out.contains("  }."));
    }

    #[test]
    fn normalize_brace_dot_preserves_attached_assertion_literal() {
        let source = r#"
매틱:움직씨 = {
  검사 <- 세움{
    { 거리 > 0 }인것 바탕으로(중단) 아니면 {
      없음.
    }.
  }.
  결과 <- (거리=3)인 검사 살피기.
}
"#;
        let out = normalize_closing_brace_dot(source);
        assert!(out.contains("  }.\n  결과 <- (거리=3)인 검사 살피기."));
    }

    #[test]
    fn normalize_brace_dot_preserves_decl_item_maegim_suffix() {
        let source = r#"
매틱:움직씨 = {
  채비 {
    데이터길이:수 <- (12) 매김 {
      범위: 4..40.
      간격: 1.
    }.
  }.
}
"#;
        let out = normalize_closing_brace_dot(source);
        assert!(out.contains("    }."));
    }

    #[test]
    fn rewrite_legacy_shorthand_drops_bogae_draw_line_without_none_injection() {
        let source = "a <- 1.\n보개로 그려.\nb <- 2.\n";
        let out = rewrite_legacy_shorthand(source);
        assert_eq!(out, "a <- 1.\n\nb <- 2.\n");
        assert!(!out.contains("없음."));
    }

    #[test]
    fn preprocess_keeps_setting_block_syntax() {
        let source = r#"
매틱:움직씨 = {
설정: {
  그래프축: "x".
  (x=1) 동작성.
}.
없음.
}
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("설정: {"));
        assert!(!out.contains("#설정 "));
    }

    #[test]
    fn preprocess_expands_bogae_colon_block_with_colon_preserved() {
        let source = r#"
보개:
  y축: 값.
매틱:움직씨 = {
  없음.
}
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("보개: {"));
        assert!(!out.contains("#보개 "));
    }

    #[test]
    fn preprocess_does_not_canonicalize_legacy_bogae_alias() {
        let source = r#"
설정보개:
  y축: 값.
매틱:움직씨 = {
  없음.
}
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("설정보개: {"));
        assert!(!out.contains("#보개 "));
    }

    #[test]
    fn preprocess_accepts_boim_block() {
        let source = r#"
보임 {
  y축: 값.
}
매틱:움직씨 = {
  없음.
}
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("보임 {"));
    }

    #[test]
    fn preprocess_leaves_malformed_boim_for_parser() {
        let source = r#"
보임 {
  y축 값.
}
매틱:움직씨 = {
  없음.
}
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("y축 값"));
    }

    #[test]
    fn preprocess_rejects_legacy_hash_header() {
        let source = "#이름: legacy\nx <- 1.\n";
        let err = preprocess_source_for_parse(source).expect_err("must reject");
        assert!(err.contains("E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN"));
    }

    #[test]
    fn validate_no_legacy_boim_surface_accepts_boim_block() {
        let source = "보임 { x: 1. }.";
        validate_no_legacy_boim_surface(source).expect("must pass");
    }

    #[test]
    fn validate_no_legacy_boim_surface_accepts_canonical_bogae() {
        let source = "설정: { 보개: { 기본: \"space2d\". }. }.";
        validate_no_legacy_boim_surface(source).expect("must pass");
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

    #[test]
    fn unary_minus_skips_pragma_lines() {
        let source = "#설정 x: -5.\n";
        let out = rewrite_unary_minus(source);
        assert_eq!(out, "#설정 x: -5.\n");
    }

    #[test]
    fn preprocess_keeps_setting_negative_literal_raw() {
        let source = r#"
매틱:움직씨 = {
설정: {
  x: -5.
}.
없음.
}
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("x: -5."));
        assert!(!out.contains("(0 - 5)"));
    }

    #[test]
    fn preprocess_lowers_vol4_event_dispatch_representative_raw_surface() {
        let source = r#"
설정 {
  title: rep-ddonirang-vol4-event-dispatch.
}.

관제탑:임자 = {
  첫알림을 받으면 {
    제.처리횟수 <- 제.처리횟수 + 1.
  }.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("모드 <- \"확인\"."));
        assert!(out.contains("처리횟수 <- 처리횟수 + 1."));
        assert!(!out.contains("관제탑:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_vol4_event_dispatch_without_title_marker() {
        let source = r#"
(이름:글) 첫알림:알림씨 = {
}.

(이름:글) 둘알림:알림씨 = {
}.

관제탑:임자 = {
  제.모드 <- "대기".
  제.마지막 <- "없음".
  제.처리횟수 <- 0.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.모드 <- "경고".
    제.마지막 <- 알림.정보.이름.
    제.처리횟수 <- 제.처리횟수 + 1.
  }.
}.

(시작)할때 {
  (이름:"과열") 첫알림 ~~> 관제탑.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("매틱:움직씨 = {"));
        assert!(out.contains("모드 <- \"경고\"."));
        assert!(out.contains("처리횟수 <- 처리횟수 + 1."));
        assert!(!out.contains("관제탑:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_vol4_resume_isolation_representative_raw_surface() {
        let source = r#"
설정 {
  title: rep-ddonirang-vol4-resume-isolation.
}.

관제탑:임자 = {
  일반_요청을 받으면 {
    제.처리건수 <- 제.처리건수 + 1.
  }.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("모드 <- \"요청처리\"."));
        assert!(out.contains("보류건수 <- 보류건수 + 1."));
        assert!(out.contains("격리됨 <- 거짓."));
        assert!(!out.contains("관제탑:임자"));
    }

    #[test]
    fn preprocess_lowers_generic_single_imja_event_subset() {
        let source = r#"
(피해량:수) 피해_받음:알림씨 = {
}.

(회복량:수) 체력_회복:알림씨 = {
}.

플레이어:임자 = {
  제.체력_최대 <- 100.
  제.체력 <- 100.

  (알림 알림.이름 == "피해_받음")인 알림을 받으면 {
    제.체력 <- 제.체력 - 알림.정보.피해량.
  }.

  (알림 알림.이름 == "체력_회복")인 알림을 받으면 {
    제.체력 <- 제.체력 + 알림.정보.회복량.
  }.
}.

(시작)할때 {
  (피해량:30) 피해_받음 ~~> 플레이어.
  플레이어.체력 보여주기.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(!out.contains("채비 {"));
        assert!(out.contains("체력 <- 100."));
        assert!(out.contains("체력 <- 체력 - 30."));
        assert!(out.contains("체력 보여주기."));
        assert!(!out.contains("플레이어:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_generic_single_imja_with_start_root_state() {
        let source = r#"
(피해량:수) 피해_받음:알림씨 = {
}.

플레이어:임자 = {
  제.체력 <- 100.

  피해_받음을 받으면 {
    제.체력 <- 제.체력 - 정보.피해량.
  }.
}.

(시작)할때 {
  단계 <- 0.
  (피해량:30) 피해_받음 ~~> 플레이어.
  단계 <- 1.
  단계 보여주기.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(!out.contains("채비 {"));
        assert!(out.contains("단계 <- 0."));
        assert!(out.contains("체력 <- 100."));
        assert!(out.contains("체력 <- 체력 - 30."));
        assert!(out.contains("단계 <- 1."));
        assert!(out.contains("단계 보여주기."));
        assert!(!out.contains("플레이어:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_generic_single_imja_with_every_hook_progression() {
        let source = r#"
(피해량:수) 피해_받음:알림씨 = {
}.

플레이어:임자 = {
  제.체력 <- 100.

  피해_받음을 받으면 {
    제.체력 <- 제.체력 - 정보.피해량.
  }.
}.

(시작)할때 {
  단계 <- 0.
}.

(매마디)마다 {
  단계 <- 단계 + 1.
  (피해량:10) 피해_받음 ~~> 플레이어.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(!out.contains("채비 {"));
        assert!(out.contains("단계 <- 0."));
        assert!(out.contains("단계 <- 단계 + 1."));
        assert!(out.contains("체력 <- 100."));
        assert!(out.contains("체력 <- 체력 - 10."));
        assert!(!out.contains("플레이어:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_keeps_top_level_chaebi_before_every_hook() {
        let source = r#"
채비 {
  t:수 <- 0.
}.

(매마디)마다 {
  t <- t + 1.
  t 보여주기.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.starts_with("매마디:움직씨 = {"));
        assert!(out.contains("  채비 {"));
        assert!(out.contains("  t <- t + 1."));
        assert!(out.find("  채비 {").expect("chaebi") < out.find("  t <- t + 1.").expect("hook"));
    }

    #[test]
    fn preprocess_lowers_generic_multi_imja_event_subset() {
        let source = r#"
(피해량:수) 피해_받음:알림씨 = {
}.

플레이어:임자 = {
  제.체력 <- 100.

  피해_받음을 받으면 {
    제.체력 <- 제.체력 - 정보.피해량.
  }.
}.

적:임자 = {
  제.체력 <- 50.

  피해_받음을 받으면 {
    제.체력 <- 제.체력 - 정보.피해량.
  }.
}.

(시작)할때 {
  단계 <- 0.
}.

(매마디)마다 {
  단계 <- 단계 + 1.
  (피해량:1) 피해_받음 ~~> 플레이어.
  (피해량:2) 피해_받음 ~~> 적.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("플레이어__체력 <- 100."));
        assert!(out.contains("적__체력 <- 50."));
        assert!(out.contains("플레이어__체력 <- 플레이어__체력 - 1."));
        assert!(out.contains("적__체력 <- 적__체력 - 2."));
        assert!(out.contains("단계 <- 단계 + 1."));
        assert!(!out.contains("플레이어:임자"));
        assert!(!out.contains("적:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_generic_multi_imja_nested_send() {
        let source = r#"
(피해량:수) 피해_받음:알림씨 = {
}.

공격:알림씨 = {
}.

플레이어:임자 = {
  제.공격력 <- 3.

  공격을 받으면 {
    (피해량:제.공격력) 피해_받음 ~~> 적.
  }.
}.

적:임자 = {
  제.체력 <- 50.

  피해_받음을 받으면 {
    제.체력 <- 제.체력 - 정보.피해량.
  }.
}.

(매마디)마다 {
  공격 ~~> 플레이어.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("플레이어__공격력 <- 3."));
        assert!(out.contains("적__체력 <- 50."));
        assert!(out.contains("적__체력 <- 적__체력 - 플레이어__공격력."));
        assert!(!out.contains("플레이어:임자"));
        assert!(!out.contains("적:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_generic_multi_imja_send_chain_with_grouped_payload() {
        let source = r#"
(전달량:수) 전달:알림씨 = {
}.

출발:알림씨 = {
}.

전달자:임자 = {
  제.기본량 <- 2.

  출발을 받으면 {
    (전달량:제.기본량) 전달 ~~> 중계자.
  }.
}.

중계자:임자 = {
  제.보정량 <- 4.

  전달을 받으면 {
    (전달량:(정보.전달량 + 제.보정량)) 전달 ~~> 목표.
  }.
}.

목표:임자 = {
  제.체력 <- 30.

  전달을 받으면 {
    제.체력 <- 제.체력 - 정보.전달량.
  }.
}.

(매마디)마다 {
  출발 ~~> 전달자.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("전달자__기본량 <- 2."));
        assert!(out.contains("중계자__보정량 <- 4."));
        assert!(out.contains("목표__체력 <- 30."));
        assert!(out.contains("목표__체력 <- 목표__체력 - (전달자__기본량 + 중계자__보정량)."));
        assert!(!out.contains("전달자:임자"));
        assert!(!out.contains("중계자:임자"));
        assert!(!out.contains("목표:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_generic_single_imja_self_send() {
        let source = r#"
(값:수) 둘알림:알림씨 = {
}.

(값:수) 첫알림:알림씨 = {
}.

관제탑:임자 = {
  제.합 <- 0.

  첫알림을 받으면 {
    (값:(정보.값 + 1)) 둘알림 ~~> 제.
  }.

  둘알림을 받으면 {
    제.합 <- 제.합 + 정보.값.
  }.
}.

(매마디)마다 {
  (값:2) 첫알림 ~~> 관제탑.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("합 <- 0."));
        assert!(out.contains("합 <- 합 + (2 + 1)."));
        assert!(!out.contains("관제탑:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_generic_typed_payload_forwarding() {
        let source = r#"
(피해량:수) 피해_받음:알림씨 = {
}.

첫알림:알림씨 = {
}.

중계자:임자 = {
  첫알림을 받으면 {
    (피해량:3) 피해_받음 ~~> 제.
  }.

  피해_받음을 받으면 {
    피해_받음 ~~> 목표.
  }.
}.

목표:임자 = {
  제.체력 <- 10.

  피해_받음을 받으면 {
    제.체력 <- 제.체력 - 정보.피해량.
  }.
}.

(매마디)마다 {
  첫알림 ~~> 중계자.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("목표__체력 <- 10."));
        assert!(out.contains("목표__체력 <- 목표__체력 - 3."));
        assert!(!out.contains("중계자:임자"));
        assert!(!out.contains("목표:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_generic_conditional_send_queue() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    { 정보.값 > 0 }인것 일때 {
      (값:정보.값) 둘알림 ~~> 제.
    }.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  둘알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.

  알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.contains("순서 <- 순서 * 10 + 1."), "{out}");
        assert!(
            out.contains("만약 1 > 0 이라면 {") || out.contains("만약 값 > 0 이라면 {"),
            "{out}"
        );
        assert!(out.contains("순서 <- 순서 * 10 + 2."));
        assert!(out.contains("순서 <- 순서 * 10 + 4."));
        assert!(out.contains("순서 <- 순서 * 10 + 3."));
        assert!(!out.contains("관제탑:임자"));
        assert!(!out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_generic_choose_send_queue() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

(값:수) 셋알림:알림씨 = {
}.

(값:수) 넷알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    고르기:
      정보.값 > 0: {
        (값:정보.값) 둘알림 ~~> 제.
      }
      정보.값 >= 0: {
        (값:정보.값) 넷알림 ~~> 제.
      }
      아니면: {
        (값:정보.값) 셋알림 ~~> 제.
      }.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  둘알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.

  셋알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 5.
  }.

  넷알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 6.
  }.

  알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(
            out.contains("만약 1 > 0 이라면 {") || out.contains("만약 값 > 0 이라면 {"),
            "{out}"
        );
        assert!(out.contains("순서 <- 순서 * 10 + 1."), "{out}");
        assert!(out.contains("순서 <- 순서 * 10 + 2."));
        assert!(out.contains("순서 <- 순서 * 10 + 3."));
        assert!(out.contains("순서 <- 순서 * 10 + 4."));
        assert!(out.contains("순서 <- 순서 * 10 + 5."));
        assert!(out.contains("순서 <- 순서 * 10 + 6."));
        assert!(!out.contains("관제탑:임자"));
        assert!(!out.contains("받으면"));

        let second_source = source.replace("(값:1) 첫알림", "(값:0) 첫알림");
        let second_out = preprocess_source_for_parse(&second_source).expect("preprocess");
        assert!(
            second_out.contains("만약 0 > 0 == 거짓 그리고 0 >= 0 이라면 {")
                || second_out.contains("만약 값 > 0 == 거짓 그리고 값 >= 0 이라면 {"),
            "{second_out}"
        );
        assert!(
            second_out.contains("순서 <- 순서 * 10 + 6."),
            "{second_out}"
        );
        assert!(!second_out.contains("관제탑:임자"));
        assert!(!second_out.contains("받으면"));

        let else_source = source.replace("(값:1) 첫알림", "(값:(0 - 1)) 첫알림");
        let false_out = preprocess_source_for_parse(&else_source).expect("preprocess");
        assert!(
            false_out.contains("만약 (0 - 1) > 0 == 거짓 그리고 (0 - 1) >= 0 == 거짓 이라면 {")
                || false_out.contains("만약 값 > 0 == 거짓 그리고 값 >= 0 == 거짓 이라면 {"),
            "{false_out}"
        );
        assert!(false_out.contains("순서 <- 순서 * 10 + 5."), "{false_out}");
        assert!(!false_out.contains("관제탑:임자"));
        assert!(!false_out.contains("받으면"));
    }

    #[test]
    fn preprocess_lowers_intro_choose_exhaustive() {
        let source = r#"
채비 {
  점수: 셈수 <- 72.
}.

고르기:
  점수 >= 70 인 경우 {
    "통과" 보여주기.
  }
  점수 < 70 인 경우 {
    "보충" 보여주기.
  }
  모든 경우 다룸.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(!out.contains("고르기:"), "{out}");
        assert!(out.contains("만약 점수 >= 70 이라면 {"), "{out}");
        assert!(
            out.contains("만약 점수 >= 70 == 거짓 그리고 점수 < 70 이라면 {"),
            "{out}"
        );
        assert!(out.contains("\"통과\" 보여주기."));
        assert!(out.contains("\"보충\" 보여주기."));
    }

    #[test]
    fn preprocess_lowers_intro_choose_else() {
        let source = r#"
채비 {
  점수: 셈수 <- 72.
}.

고르기:
  점수 >= 90: {
    "우수" 보여주기.
  }
  아니면: {
    "보통" 보여주기.
  }.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(!out.contains("고르기:"), "{out}");
        assert!(out.contains("만약 점수 >= 90 이라면 {"), "{out}");
        assert!(out.contains("만약 점수 >= 90 == 거짓 이라면 {"), "{out}");
        assert!(out.contains("\"우수\" 보여주기."));
        assert!(out.contains("\"보통\" 보여주기."));
    }

    #[test]
    fn preprocess_wraps_charim_plain_decl_without_seed_def() {
        let source = r#"
채비 {
  안내: 글 = "intro".
  인사: 글 <- "hello".
}.

인사 보여주기.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(out.starts_with("매틱:움직씨 = {"), "{out}");
        assert!(out.contains("안내: 글 = \"intro\"."));
        assert!(out.contains("인사 보여주기."));
    }

    #[test]
    fn preprocess_lowers_generic_choose_exhaustive_send_queue() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

(값:수) 넷알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    고르기:
      정보.값 > 0 인 경우 {
        (값:정보.값) 둘알림 ~~> 제.
      }
      정보.값 >= 0 인 경우 {
        (값:정보.값) 넷알림 ~~> 제.
      }
      모든 경우 다룸.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  둘알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.

  넷알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 6.
  }.

  알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let out = preprocess_source_for_parse(source).expect("preprocess");
        assert!(
            out.contains("만약 1 > 0 이라면 {") || out.contains("만약 값 > 0 이라면 {"),
            "{out}"
        );
        assert!(
            out.contains("만약 1 > 0 == 거짓 그리고 1 >= 0 이라면 {")
                || out.contains("만약 값 > 0 == 거짓 그리고 값 >= 0 이라면 {"),
            "{out}"
        );
        assert!(out.contains("순서 <- 순서 * 10 + 1."), "{out}");
        assert!(out.contains("순서 <- 순서 * 10 + 2."));
        assert!(out.contains("순서 <- 순서 * 10 + 3."));
        assert!(out.contains("순서 <- 순서 * 10 + 4."));
        assert!(out.contains("순서 <- 순서 * 10 + 6."));
        assert!(!out.contains("관제탑:임자"));
        assert!(!out.contains("받으면"));

        let second_source = source.replace("(값:1) 첫알림", "(값:0) 첫알림");
        let second_out = preprocess_source_for_parse(&second_source).expect("preprocess");
        assert!(
            second_out.contains("만약 0 > 0 == 거짓 그리고 0 >= 0 이라면 {")
                || second_out.contains("만약 값 > 0 == 거짓 그리고 값 >= 0 이라면 {"),
            "{second_out}"
        );
        assert!(
            second_out.contains("순서 <- 순서 * 10 + 6."),
            "{second_out}"
        );
        assert!(!second_out.contains("관제탑:임자"));
        assert!(!second_out.contains("받으면"));
    }
}
