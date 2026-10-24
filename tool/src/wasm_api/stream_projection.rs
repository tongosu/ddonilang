//! WASM 스트림 표시의 기존 도우미. StreamMeta와 공개 VM의 소유 모듈은 유지한다.
use super::*;

pub(super) fn collect_stream_sidecars(
    world: &ddonirang_core::platform::NuriWorld,
) -> HashMap<String, StreamMeta> {
    let mut out: HashMap<String, StreamMeta> = HashMap::new();
    for (tag, value) in world.resource_fixed64_entries() {
        let Some((base, kind)) = parse_stream_sidecar_tag(&tag) else {
            continue;
        };
        let numeric = u64::try_from(value.raw_i64()).ok();
        out.entry(base).or_default().set_from_kind(kind, numeric);
    }
    for (tag, value) in world.resource_value_entries() {
        let Some((base, kind)) = parse_stream_sidecar_tag(&tag) else {
            continue;
        };
        let numeric = resource_value_to_u64(&value);
        out.entry(base).or_default().set_from_kind(kind, numeric);
    }
    out
}

pub(super) fn parse_stream_sidecar_tag(tag: &str) -> Option<(String, &'static str)> {
    const PATTERNS: [(&str, &str); 16] = [
        ("_stream_capacity", "capacity"),
        (".stream_capacity", "capacity"),
        ("_capacity", "capacity"),
        (".capacity", "capacity"),
        ("_head", "head"),
        (".head", "head"),
        ("_length", "len"),
        (".length", "len"),
        ("_len", "len"),
        (".len", "len"),
        ("_용량", "capacity"),
        (".용량", "capacity"),
        ("_머리", "head"),
        (".머리", "head"),
        ("_길이", "len"),
        (".길이", "len"),
    ];
    let lower = tag.to_ascii_lowercase();
    for (suffix, kind) in PATTERNS {
        let is_match = if suffix.is_ascii() {
            lower.ends_with(suffix)
        } else {
            tag.ends_with(suffix)
        };
        if !is_match {
            continue;
        }
        // "foo_stream_capacity"는 base를 "foo_stream"으로 해석한다.
        let raw_base = if suffix == "_stream_capacity" {
            tag.strip_suffix("_capacity").unwrap_or_default()
        } else if suffix == ".stream_capacity" {
            tag.strip_suffix(".capacity").unwrap_or_default()
        } else {
            tag.strip_suffix(suffix).unwrap_or_default()
        };
        let base = raw_base
            .trim_matches(|c| c == '.' || c == '_' || c == '/')
            .to_string();
        if base.is_empty() {
            continue;
        }
        return Some((base, kind));
    }
    None
}

pub(super) fn is_stream_resource(tag: &str, value: &ResourceValue, sidecar: Option<&StreamMeta>) -> bool {
    if sidecar.copied().unwrap_or_default().has_any() {
        return matches!(value, ResourceValue::List(_) | ResourceValue::Map(_));
    }
    let lower = tag.to_ascii_lowercase();
    if lower.contains("stream") || tag.contains("흐름") {
        return matches!(value, ResourceValue::List(_) | ResourceValue::Map(_));
    }
    let ResourceValue::Map(entries) = value else {
        return false;
    };
    let mut has_buffer = false;
    let mut has_meta = false;
    for entry in entries.values() {
        let key = resource_key_to_string(&entry.key);
        match key.as_str() {
            "buffer" | "버퍼" => has_buffer = true,
            "capacity" | "stream_capacity" | "용량" | "head" | "머리" | "len" | "length"
            | "길이" => has_meta = true,
            _ => {}
        }
    }
    has_buffer || has_meta
}

pub(super) fn extract_ring_buffer(value: &ResourceValue, sidecar: Option<&StreamMeta>) -> Option<JsonValue> {
    match value {
        ResourceValue::List(items) => {
            let side = sidecar.copied().unwrap_or_default();
            let mut buffer = items.iter().map(resource_value_to_json).collect::<Vec<_>>();
            let capacity = side
                .capacity
                .unwrap_or(items.len() as u64)
                .max(items.len() as u64);
            if buffer.len() < capacity as usize {
                buffer.resize(capacity as usize, JsonValue::Null);
            }
            let len = side
                .len
                .unwrap_or(items.len() as u64)
                .min(buffer.len() as u64);
            let head = if buffer.is_empty() {
                0
            } else {
                side.head
                    .unwrap_or_else(|| len.saturating_sub(1))
                    .min(buffer.len().saturating_sub(1) as u64)
            };
            Some(json!({
                "capacity": capacity,
                "head": head,
                "len": len,
                "buffer": JsonValue::Array(buffer),
            }))
        }
        ResourceValue::Map(entries) => {
            let side = sidecar.copied().unwrap_or_default();
            let mut buffer_json: Option<Vec<JsonValue>> = None;
            let mut capacity: Option<u64> = None;
            let mut head: Option<u64> = None;
            let mut len: Option<u64> = None;

            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                match key.as_str() {
                    "buffer" | "버퍼" => {
                        buffer_json = match &entry.value {
                            ResourceValue::List(items) => {
                                Some(items.iter().map(resource_value_to_json).collect::<Vec<_>>())
                            }
                            other => Some(vec![resource_value_to_json(other)]),
                        };
                    }
                    "capacity" | "stream_capacity" | "용량" => {
                        capacity = resource_value_to_u64(&entry.value);
                    }
                    "head" | "머리" => {
                        head = resource_value_to_u64(&entry.value);
                    }
                    "len" | "length" | "길이" => {
                        len = resource_value_to_u64(&entry.value);
                    }
                    _ => {}
                }
            }
            if capacity.is_none() {
                capacity = side.capacity;
            }
            if head.is_none() {
                head = side.head;
            }
            if len.is_none() {
                len = side.len;
            }

            let mut buffer = buffer_json.unwrap_or_default();
            let cap = capacity.unwrap_or(buffer.len() as u64);
            let cap_clamped = cap.max(buffer.len() as u64);
            if buffer.len() < cap_clamped as usize {
                buffer.resize(cap_clamped as usize, JsonValue::Null);
            }
            let raw_len = len.unwrap_or(buffer.len() as u64);
            let safe_len = raw_len.min(buffer.len() as u64);
            let safe_head = head
                .unwrap_or_else(|| safe_len.saturating_sub(1))
                .min(buffer.len().saturating_sub(1) as u64);

            Some(json!({
                "capacity": cap_clamped,
                "head": if buffer.is_empty() { 0 } else { safe_head },
                "len": safe_len,
                "buffer": buffer,
            }))
        }
        _ => None,
    }
}

pub(super) fn resource_value_to_u64(value: &ResourceValue) -> Option<u64> {
    match value {
        ResourceValue::Fixed64(v) => u64::try_from(v.raw_i64()).ok(),
        ResourceValue::Unit(v) => u64::try_from(v.value.raw_i64()).ok(),
        ResourceValue::Bool(v) => Some(if *v { 1 } else { 0 }),
        ResourceValue::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

