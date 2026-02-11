use crate::fixed64::Fixed64;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetState {
    pub agent_id: u64,
    pub goal_id: u64,
    pub condition: GoalCondition,
    pub priority: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GoalCondition {
    LocationReached { x: Fixed64, y: Fixed64 },
    ObjectPickedUp { object_id: u64 },
    ObjectPlaced { object_id: u64, location_id: u64 },
    StateEquals { key: String, value: String },
    Custom { predicate: String },
}

pub fn parse_dorok(raw: &str) -> TargetState {
    let normalized = normalize_korean_text(raw);
    let text = normalize_goal_text(&normalized);
    let condition = parse_goal_condition(&text);
    TargetState {
        agent_id: 0,
        goal_id: hash_to_id(&format!("goal:{}", text)),
        condition,
        priority: 128,
    }
}

pub fn target_state_detjson(target: &TargetState) -> String {
    let mut out = String::new();
    out.push('{');
    push_kv_str(&mut out, "schema", "target_state.v1", true);
    push_kv_num(&mut out, "agent_id", target.agent_id, false);
    push_kv_num(&mut out, "goal_id", target.goal_id, false);
    push_kv_num(&mut out, "priority", target.priority as u64, false);
    out.push_str(",\"condition\":");
    out.push_str(&goal_condition_detjson(&target.condition));
    out.push('}');
    out
}

fn goal_condition_detjson(condition: &GoalCondition) -> String {
    let mut out = String::new();
    out.push('{');
    match condition {
        GoalCondition::LocationReached { x, y } => {
            push_kv_str(&mut out, "type", "LocationReached", true);
            push_kv_fixed(&mut out, "x", *x, false);
            push_kv_fixed(&mut out, "y", *y, false);
        }
        GoalCondition::ObjectPickedUp { object_id } => {
            push_kv_str(&mut out, "type", "ObjectPickedUp", true);
            push_kv_num(&mut out, "object_id", *object_id, false);
        }
        GoalCondition::ObjectPlaced {
            object_id,
            location_id,
        } => {
            push_kv_str(&mut out, "type", "ObjectPlaced", true);
            push_kv_num(&mut out, "object_id", *object_id, false);
            push_kv_num(&mut out, "location_id", *location_id, false);
        }
        GoalCondition::StateEquals { key, value } => {
            push_kv_str(&mut out, "type", "StateEquals", true);
            push_kv_str(&mut out, "key", key, false);
            push_kv_str(&mut out, "value", value, false);
        }
        GoalCondition::Custom { predicate } => {
            push_kv_str(&mut out, "type", "Custom", true);
            push_kv_str(&mut out, "predicate", predicate, false);
        }
    }
    out.push('}');
    out
}

fn normalize_korean_text(text: &str) -> String {
    text.trim().nfc().collect::<String>()
}

fn normalize_goal_text(text: &str) -> String {
    let line = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();
    let mut cleaned = line.to_string();
    loop {
        let next = cleaned.trim_end_matches([' ', '.', '!', '?']).to_string();
        if next.len() == cleaned.len() {
            break;
        }
        cleaned = next;
    }
    if cleaned.ends_with("도록") {
        cleaned = cleaned.trim_end_matches("도록").trim_end().to_string();
    }
    cleaned
}

fn parse_goal_condition(text: &str) -> GoalCondition {
    if let Some((key, value)) = parse_state_equals(text) {
        return GoalCondition::StateEquals { key, value };
    }
    if is_place_action(text) {
        let (object, rest) = split_object_and_rest(text);
        let location = extract_location(&rest).unwrap_or_default();
        return GoalCondition::ObjectPlaced {
            object_id: hash_to_id(&object),
            location_id: hash_to_id(&location),
        };
    }
    if is_pickup_action(text) {
        let (object, _) = split_object_and_rest(text);
        return GoalCondition::ObjectPickedUp {
            object_id: hash_to_id(&object),
        };
    }
    if is_move_action(text) {
        let coords = parse_coordinates(text).unwrap_or_else(|| {
            (
                Fixed64::from_i64(0),
                Fixed64::from_i64(0),
            )
        });
        return GoalCondition::LocationReached {
            x: coords.0,
            y: coords.1,
        };
    }
    GoalCondition::Custom {
        predicate: text.to_string(),
    }
}

fn parse_state_equals(text: &str) -> Option<(String, String)> {
    let mut parts = text.splitn(2, '=');
    let key = parts.next()?.trim();
    let value = parts.next()?.trim();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key.to_string(), value.to_string()))
}

fn is_pickup_action(text: &str) -> bool {
    ["줍", "집", "획득", "얻"]
        .iter()
        .any(|token| text.contains(token))
}

fn is_place_action(text: &str) -> bool {
    ["놓", "두", "배치", "설치"]
        .iter()
        .any(|token| text.contains(token))
}

fn is_move_action(text: &str) -> bool {
    text.contains("이동")
        || text.contains("도달")
        || (text.contains("가") && (text.contains("로") || text.contains("에")))
}

fn split_object_and_rest(text: &str) -> (String, String) {
    let idx = text
        .char_indices()
        .find(|(_, ch)| *ch == '을' || *ch == '를')
        .map(|(idx, ch)| (idx, ch.len_utf8()));
    if let Some((idx, len)) = idx {
        let object = text[..idx].trim().to_string();
        let rest = text[idx + len..].trim().to_string();
        return (object, rest);
    }
    let mut parts = text.split_whitespace();
    let object = parts.next().unwrap_or("").trim().to_string();
    let rest = parts.collect::<Vec<_>>().join(" ");
    (object, rest.trim().to_string())
}

fn extract_location(text: &str) -> Option<String> {
    if let Some(idx) = text.find('에') {
        let location = text[..idx].trim();
        if !location.is_empty() {
            return Some(location.to_string());
        }
    }
    if let Some(idx) = text.find("으로") {
        let location = text[..idx].trim();
        if !location.is_empty() {
            return Some(location.to_string());
        }
    }
    if let Some(idx) = text.find('로') {
        let location = text[..idx].trim();
        if !location.is_empty() {
            return Some(location.to_string());
        }
    }
    None
}

fn parse_coordinates(text: &str) -> Option<(Fixed64, Fixed64)> {
    if let Some((x, y)) = parse_xy_fields(text) {
        return Some((x, y));
    }
    if let Some((x, y)) = parse_paren_pair(text) {
        return Some((x, y));
    }
    parse_simple_pair(text)
}

fn parse_xy_fields(text: &str) -> Option<(Fixed64, Fixed64)> {
    let x = extract_number_after_key(text, "x=")?;
    let y = extract_number_after_key(text, "y=")?;
    Some((x, y))
}

fn parse_paren_pair(text: &str) -> Option<(Fixed64, Fixed64)> {
    let start = text.find('(')?;
    let end = text[start..].find(')')? + start;
    let inner = &text[start + 1..end];
    parse_pair_from(inner)
}

fn parse_simple_pair(text: &str) -> Option<(Fixed64, Fixed64)> {
    for token in text.split_whitespace() {
        if let Some(pair) = parse_pair_from(token) {
            return Some(pair);
        }
    }
    None
}

fn parse_pair_from(text: &str) -> Option<(Fixed64, Fixed64)> {
    let mut parts = text.splitn(2, ',');
    let left = parts.next()?.trim();
    let right = parts.next()?.trim();
    if left.is_empty() || right.is_empty() {
        return None;
    }
    let x = parse_fixed64_string(left).ok()?;
    let y = parse_fixed64_string(right).ok()?;
    Some((x, y))
}

fn extract_number_after_key(text: &str, key: &str) -> Option<Fixed64> {
    let idx = text.find(key)?;
    let rest = &text[idx + key.len()..];
    let number = rest
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
        .collect::<String>();
    if number.is_empty() {
        return None;
    }
    parse_fixed64_string(&number).ok()
}

fn parse_fixed64_string(input: &str) -> Result<Fixed64, String> {
    let text = input.trim();
    if text.is_empty() {
        return Err("E_GOAL_FIXED64 빈 문자열".to_string());
    }
    let mut sign = 1i128;
    let mut raw_text = text;
    if let Some(rest) = raw_text.strip_prefix('-') {
        sign = -1;
        raw_text = rest;
    } else if let Some(rest) = raw_text.strip_prefix('+') {
        raw_text = rest;
    }
    let mut parts = raw_text.splitn(2, '.');
    let int_part = parts.next().unwrap_or("");
    let frac_part = parts.next().unwrap_or("");

    let int_value = if int_part.is_empty() {
        0i128
    } else {
        if !int_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("E_GOAL_FIXED64 정수부 형식 오류: {}", input));
        }
        int_part
            .parse::<i128>()
            .map_err(|_| format!("E_GOAL_FIXED64 정수부 변환 실패: {}", input))?
    };

    let frac_value = if frac_part.is_empty() {
        0i128
    } else {
        if !frac_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("E_GOAL_FIXED64 소수부 형식 오류: {}", input));
        }
        frac_part
            .parse::<i128>()
            .map_err(|_| format!("E_GOAL_FIXED64 소수부 변환 실패: {}", input))?
    };

    let scale = 10i128.pow(frac_part.len() as u32);
    let frac_raw = if frac_part.is_empty() {
        0i128
    } else {
        (frac_value * (1i128 << 32)) / scale
    };

    let raw = (int_value << 32) + frac_raw;
    let signed = raw.saturating_mul(sign);
    let clamped = clamp_i128_to_i64(signed);
    Ok(Fixed64::from_raw_i64(clamped))
}

fn clamp_i128_to_i64(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}

fn hash_to_id(text: &str) -> u64 {
    let normalized = normalize_korean_text(text);
    let bytes = blake3::hash(normalized.as_bytes());
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes.as_bytes()[..8]);
    u64::from_le_bytes(buf)
}

fn push_kv_str(out: &mut String, key: &str, value: &str, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":\"");
    out.push_str(&escape_json(value));
    out.push('"');
}

fn push_kv_num(out: &mut String, key: &str, value: u64, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(&value.to_string());
}

fn push_kv_fixed(out: &mut String, key: &str, value: Fixed64, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(&value.to_string());
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_goal_pickup() {
        let goal = parse_dorok("사과를 줍도록");
        assert!(matches!(goal.condition, GoalCondition::ObjectPickedUp { .. }));
    }

    #[test]
    fn parse_goal_place() {
        let goal = parse_dorok("사과를 상자에 놓도록");
        assert!(matches!(goal.condition, GoalCondition::ObjectPlaced { .. }));
    }

    #[test]
    fn parse_goal_move_with_coords() {
        let goal = parse_dorok("(1.5,2)로 이동하도록");
        assert!(matches!(goal.condition, GoalCondition::LocationReached { .. }));
    }

    #[test]
    fn parse_goal_state_equals() {
        let goal = parse_dorok("상태=준비");
        assert!(matches!(goal.condition, GoalCondition::StateEquals { .. }));
    }

    #[test]
    fn parse_goal_custom() {
        let goal = parse_dorok("알 수 없는 조건");
        assert!(matches!(goal.condition, GoalCondition::Custom { .. }));
    }
}
