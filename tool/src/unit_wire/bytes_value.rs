//! Reused RD-680 saved Bytes transport owner from fixed 30f19f23.
use ddonirang_core::platform::ResourceValue;
use serde_json::Value as JsonValue;
#[cfg(test)] use serde_json::json;
#[cfg(test)] use ddonirang_core::platform::{RecordFieldType, ResourceMapEntry};
#[cfg(test)] use std::collections::BTreeMap;
#[cfg(test)] use ddonirang_core::Fixed64;
#[cfg(test)] use super::{resource_value_to_detjson, resource_value_from_detjson};

pub(super) const BYTES_FIELDS: [&str; 8] = [
    "frame_base64url", "frame_byte_length", "frame_sha256", "payload_byte_length",
    "payload_sha256", "resource_value_tag", "type", "version",
];
#[cfg(test)] const SNAPSHOT_MAX_BYTES: usize = 2_000_000;
const PAYLOAD_MAX_BYTES: u64 = 1_000_000;

fn invalid_bytes_snapshot() -> String {
    "E_RESOURCE_SNAPSHOT_INVALID: 바이트열 저장 필드·형식·순서가 올바르지 않습니다".into()
}

fn bytes_length_error() -> String {
    ddonirang_core::bytes::BytesError::Length.to_string()
}

fn bytes_hash(value: &JsonValue) -> Result<[u8; 32], String> {
    let text = value.as_str().ok_or_else(invalid_bytes_snapshot)?;
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(invalid_bytes_snapshot());
    }
    let mut hash = [0; 32];
    hex::decode_to_slice(text, &mut hash).map_err(|_| invalid_bytes_snapshot())?;
    Ok(hash)
}

fn base64url_digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'-' => Some(62),
        b'_' => Some(63),
        _ => None,
    }
}

pub(super) fn parse_bytes_v1(obj: &serde_json::Map<String, JsonValue>) -> Result<ResourceValue, String> {
    use ddonirang_core::bytes::{ByteSequence, BytesError};
    if obj.len() != BYTES_FIELDS.len() || !obj.keys().map(String::as_str).eq(BYTES_FIELDS) {
        return Err(invalid_bytes_snapshot());
    }
    if obj["type"].as_str() != Some("bytes_v1") || obj["resource_value_tag"].as_str() != Some("0c") {
        return Err(invalid_bytes_snapshot());
    }
    if obj["version"].as_u64() != Some(1) {
        return Err(BytesError::Version.to_string());
    }
    let payload_len = obj["payload_byte_length"].as_u64().ok_or_else(invalid_bytes_snapshot)?;
    let frame_len = obj["frame_byte_length"].as_u64().ok_or_else(invalid_bytes_snapshot)?;
    if payload_len > PAYLOAD_MAX_BYTES || payload_len.checked_add(20) != Some(frame_len) {
        return Err(bytes_length_error());
    }
    let encoded = obj["frame_base64url"].as_str().ok_or_else(invalid_bytes_snapshot)?;
    let encoded_len = frame_len.checked_mul(8).and_then(|n| n.checked_add(5)).map(|n| n / 6)
        .ok_or_else(bytes_length_error)?;
    if encoded.len() > 1_333_360 || encoded.len() as u64 != encoded_len {
        return Err(bytes_length_error());
    }
    // Alphabet and final unused bits prove canonical unpadded re-encoding
    // before allocating the decoded frame. No normalization/repair is allowed.
    if !encoded.bytes().all(|byte| base64url_digit(byte).is_some()) {
        return Err(invalid_bytes_snapshot());
    }
    let last = encoded.bytes().last().and_then(base64url_digit).ok_or_else(invalid_bytes_snapshot)?;
    if (encoded.len() % 4 == 2 && last & 15 != 0)
        || (encoded.len() % 4 == 3 && last & 3 != 0) || encoded.len() % 4 == 1 {
        return Err(invalid_bytes_snapshot());
    }
    let payload_hash = bytes_hash(&obj["payload_sha256"])?;
    let frame_hash = bytes_hash(&obj["frame_sha256"])?;
    let mut frame = Vec::with_capacity(frame_len as usize);
    let mut bits = 0u32;
    let mut count = 0;
    for byte in encoded.bytes() {
        bits = (bits << 6) | u32::from(base64url_digit(byte).ok_or_else(invalid_bytes_snapshot)?);
        count += 6;
        if count >= 8 {
            count -= 8;
            frame.push((bits >> count) as u8);
            bits &= (1 << count) - 1;
        }
    }
    if frame.len() != frame_len as usize { return Err(bytes_length_error()); }
    let value = ByteSequence::restore_verified(&frame, &payload_hash, &frame_hash)
        .map_err(|error| error.to_string())?;
    if value.len() as u64 != payload_len { return Err(bytes_length_error()); }
    Ok(ResourceValue::Bytes(value))
}

/// Raw admission retains duplicate/order evidence before serde_json::Value
/// would discard it. Non-Bytes objects keep their previous parser behavior.
#[cfg(test)]
fn parse_resource_snapshot_json(text: &str) -> Result<JsonValue, String> {
    if text.len() > SNAPSHOT_MAX_BYTES { return Err(bytes_length_error()); }
    serde_json::from_str::<SnapshotJson>(text).map(|value| value.0).map_err(|error| error.to_string())
}

#[cfg(test)] struct SnapshotJson(JsonValue);

pub(crate) fn serialize_resource_snapshot_json(payload: &JsonValue) -> String {
    let mut bytes = Vec::new();
    let contains_bytes = write_compact_json(&mut bytes, payload).expect("JSON Vec writer");
    // Bytes snapshot transport alone adds RD-680's terminal LF. Ordinary
    // engine responses and result-byte accounting use compact JSON without it.
    if contains_bytes { bytes.push(b'\n'); }
    String::from_utf8(bytes).expect("JSON is UTF-8")
}

pub(crate) fn write_compact_json<W: std::io::Write>(writer: &mut W, payload: &JsonValue)
    -> Result<bool, serde_json::Error> {
    // Preserve serde_json's compact scalar encoding and map iteration order,
    // without recursively visiting the snapshot's structural containers.
    enum Job<'a> { Value(&'a JsonValue), Key(&'a str), Text(&'static str) }
    let mut jobs = vec![Job::Value(payload)];
    let mut contains_bytes = false;
    while let Some(job) = jobs.pop() {
        match job {
            Job::Text(token) => writer.write_all(token.as_bytes()).map_err(serde_json::Error::io)?,
            Job::Key(key) => serde_json::to_writer(&mut *writer, key)?,
            Job::Value(JsonValue::Object(obj)) => {
                contains_bytes |= obj.get("type").and_then(JsonValue::as_str) == Some("bytes_v1");
                writer.write_all(b"{").map_err(serde_json::Error::io)?;
                jobs.push(Job::Text("}"));
                for (index, (key, value)) in obj.iter().enumerate().rev() {
                    jobs.push(Job::Value(value));
                    jobs.push(Job::Text(":"));
                    jobs.push(Job::Key(key));
                    if index != 0 { jobs.push(Job::Text(",")); }
                }
            }
            Job::Value(JsonValue::Array(items)) => {
                writer.write_all(b"[").map_err(serde_json::Error::io)?;
                jobs.push(Job::Text("]"));
                for (index, value) in items.iter().enumerate().rev() {
                    jobs.push(Job::Value(value));
                    if index != 0 { jobs.push(Job::Text(",")); }
                }
            }
            Job::Value(value) => serde_json::to_writer(&mut *writer, value)?,
        }
    }
    Ok(contains_bytes)
}

pub(crate) fn write_pretty_json<W: std::io::Write>(writer: &mut W, payload: &JsonValue)
    -> Result<(), serde_json::Error> {
    enum Job<'a> { Value(&'a JsonValue, usize), Key(&'a str), Text(&'static [u8]), Indent(usize) }
    let mut jobs = vec![Job::Value(payload, 0)];
    while let Some(job) = jobs.pop() {
        match job {
            Job::Text(text) => writer.write_all(text).map_err(serde_json::Error::io)?,
            Job::Indent(depth) => {
                let spaces = [b' '; 128];
                let mut remaining = depth.checked_mul(2).ok_or_else(|| serde_json::Error::io(
                    std::io::Error::new(std::io::ErrorKind::Other, "JSON indent overflow")))?;
                while remaining != 0 {
                    let count = remaining.min(spaces.len());
                    writer.write_all(&spaces[..count]).map_err(serde_json::Error::io)?;
                    remaining -= count;
                }
            }
            Job::Key(key) => serde_json::to_writer(&mut *writer, key)?,
            Job::Value(JsonValue::Array(items), depth) if !items.is_empty() => {
                writer.write_all(b"[\n").map_err(serde_json::Error::io)?;
                jobs.push(Job::Text(b"]")); jobs.push(Job::Indent(depth)); jobs.push(Job::Text(b"\n"));
                for (index, value) in items.iter().enumerate().rev() {
                    jobs.push(Job::Value(value, depth + 1)); jobs.push(Job::Indent(depth + 1));
                    if index != 0 { jobs.push(Job::Text(b",\n")); }
                }
            }
            Job::Value(value @ JsonValue::Object(items), _) if matches!(items.get("type").and_then(JsonValue::as_str),Some("tuple_v1" | "rational_v1")) => {
                // tuple_v1 is an exact ASCII JSON leaf, even in a human pretty
                // response. Ordinary non-Tuple formatting remains unchanged.
                serde_json::to_writer(&mut *writer, value)?;
            }
            Job::Value(JsonValue::Object(items), depth) if !items.is_empty() => {
                writer.write_all(b"{\n").map_err(serde_json::Error::io)?;
                jobs.push(Job::Text(b"}")); jobs.push(Job::Indent(depth)); jobs.push(Job::Text(b"\n"));
                for (index, (key, value)) in items.iter().enumerate().rev() {
                    jobs.push(Job::Value(value, depth + 1)); jobs.push(Job::Text(b": "));
                    jobs.push(Job::Key(key)); jobs.push(Job::Indent(depth + 1));
                    if index != 0 { jobs.push(Job::Text(b",\n")); }
                }
            }
            // Only scalars and empty containers reach serde's formatter.
            Job::Value(value, _) => serde_json::to_writer(&mut *writer, value)?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod saved_bytes_transport_tests {
    use super::*;
    use ddonirang_core::bytes::ByteSequence;
    use sha2::{Digest, Sha256};

    #[test]
    fn pretty_sink_matches_serde_bytes_and_handles_deep_failure_cleanup() {
        for value in [json!(null), json!(true), json!(1.25), json!("한글\n😀"), json!([]), json!({}),
            json!({"a":[[], {}, {"x": [false, "\u{0000}"]}], "b": {"type":"bytes_v1"}})] {
            let mut bytes = Vec::new();
            write_pretty_json(&mut bytes, &value).unwrap();
            assert_eq!(bytes, serde_json::to_vec_pretty(&value).unwrap());
        }
        struct Limit(usize);
        impl std::io::Write for Limit {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if self.0 == 0 { return Err(std::io::Error::new(std::io::ErrorKind::Other, "sink full")); }
                let count = bytes.len().min(self.0).min(3);
                self.0 -= count; Ok(count)
            }
            fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
        }
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut root = JsonValue::Bool(true);
            for _ in 0..20_000 { root = JsonValue::Array(vec![root]); }
            let owner = super::super::SnapshotJsonOwner::from_value(root);
            assert!(write_pretty_json(&mut Limit(65_536), &owner).unwrap_err().is_io());
            drop(owner);
            let mut root = JsonValue::Bool(true);
            for _ in 0..512 { root = JsonValue::Array(vec![root]); }
            let output = crate::runtime_surface::RuntimeJsonOutput::new(root);
            let text = output.pretty_text().unwrap();
            assert_eq!(text.matches("[\n").count(), 512);
            assert_eq!(text.matches(']').count(), 512);
            assert!(text.contains("true"));
            assert!(!text.ends_with('\n'));
            drop(output);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn compact_sink_preserves_bytes_without_lf_and_propagates_short_write_failure() {
        struct BoundedSink { bytes: Vec<u8>, limit: usize }
        impl std::io::Write for BoundedSink {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if self.bytes.len() == self.limit {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "sink full"));
                }
                let count = bytes.len().min(self.limit - self.bytes.len()).min(3);
                self.bytes.extend_from_slice(&bytes[..count]);
                Ok(count)
            }
            fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
        }
        let value = json!({"a": ["한글\n😀", 1.25, null, false], "b": {"type":"bytes_v1"}});
        let expected = serde_json::to_vec(&value).unwrap();
        for limit in 0..=expected.len() {
            let mut sink = BoundedSink { bytes: vec![], limit };
            let result = write_compact_json(&mut sink, &value);
            assert_eq!(sink.bytes, expected[..limit]);
            if limit == expected.len() { assert!(result.unwrap()); }
            else { assert!(result.unwrap_err().is_io()); }
        }
        assert_eq!(serialize_resource_snapshot_json(&value).as_bytes(), [expected.as_slice(), b"\n"].concat());
    }

    #[test]
    fn snapshot_iterative_writer_preserves_compact_json_and_bytes_terminal_lf() {
        for value in [JsonValue::Null, json!(true), json!(-17), json!(1.25),
            json!("한글\n\"\\\t😀"), json!([]), json!({}),
            json!({"z": [null, true, {"q": "\u{0000}"}], "a": [1, -2, "3"]})] {
            assert_eq!(serialize_resource_snapshot_json(&value), value.to_string());
        }
        let value = json!({"outer": [{"type": "bytes_v1"}], "text": "bytes_v1"});
        assert_eq!(serialize_resource_snapshot_json(&value), value.to_string() + "\n");
        assert_eq!(serialize_resource_snapshot_json(&json!({"text": "bytes_v1"})), "{\"text\":\"bytes_v1\"}");
    }

    #[test]
    fn snapshot_iterative_writer_handles_deep_containers_on_small_stack() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = leaf();
            for _ in 0..20_000 { value = JsonValue::Array(vec![value]); }
            let serialized = serialize_resource_snapshot_json(&value);
            // serde_json::Value's own Drop is outside this writer's contract.
            // Detach fixture containers iteratively before asserting.
            let mut pending = vec![value];
            while let Some(mut item) = pending.pop() {
                match &mut item {
                    JsonValue::Array(items) => pending.extend(std::mem::take(items)),
                    JsonValue::Object(items) => pending.extend(std::mem::take(items).into_iter().map(|(_, value)| value)),
                    _ => {},
                }
            }
            let mut expected = "[".repeat(20_000);
            expected.push_str(&leaf().to_string());
            expected.push_str(&"]".repeat(20_000));
            expected.push('\n');
            assert_eq!(serialized, expected);
        }).unwrap().join().unwrap();
    }

    fn leaf() -> JsonValue {
        resource_value_to_detjson(&ResourceValue::Bytes(ByteSequence::new(vec![0, 255])))
    }

    #[test]
    fn fixed_transport_vectors_preserve_type_payload_and_both_hash_domains() {
        let cases: &[(&[u8], usize, &str)] = &[
            (&[], 319, "eb42853bdc49c6b7417871e6c70b87916579c1f5021c9482505a4109935c5859"),
            (&[0], 320, "0a8790c1340ba9a0a9e7486b6b8b819116eb80018934de3b052091851470291d"),
            (&[0, 255], 322, "9a1f11b39d8557b0388535429ba58f64bcef8af8848d21994b29cdf4406341e8"),
            ("한글😀".as_bytes(), 333, "f6045303297dde7882dae9e931359adb821684dcc86e1e1160f053cdf498ff40"),
        ];
        for (payload, length, hash) in cases {
            let value = ResourceValue::Bytes(ByteSequence::new(*payload));
            let json = resource_value_to_detjson(&value);
            let raw = serialize_resource_snapshot_json(&json);
            assert_eq!(raw.len(), *length);
            assert_eq!(hex::encode(Sha256::digest(raw.as_bytes())), *hash);
            assert_eq!(resource_value_from_detjson(&parse_resource_snapshot_json(&raw).unwrap()).unwrap(), value);
            let mut canonical = Vec::new();
            value.encode_canon(&mut canonical);
            assert_eq!(canonical[0], 0x0c);
            assert_eq!(&canonical[1..], ByteSequence::new(*payload).storage_frame());
        }
    }

    #[test]
    fn raw_bytes_leaf_rejects_duplicate_missing_unknown_and_reordered_keys() {
        let raw = leaf().to_string();
        for bad in [
            raw.replacen("{", "{\"version\":1,", 1),
            raw.replacen("{", "{\"extra\":0,", 1),
            raw.replace(",\"version\":1", ""),
            raw.replace("\"type\":\"bytes_v1\"", "\"type\":\"bytes_v1\",\"type\":\"string\""),
        ] {
            assert!(parse_resource_snapshot_json(&bad).is_err(), "accepted: {bad}");
        }
        // Valid JSON with reversed fields must not be silently sorted/admitted.
        let reversed = leaf().as_object().unwrap().iter().rev()
            .map(|(k, v)| format!("{}:{v}", serde_json::to_string(k).unwrap()))
            .collect::<Vec<_>>().join(",");
        assert!(parse_resource_snapshot_json(&format!("{{{reversed}}}")).is_err());
    }

    #[test]
    fn bytes_transport_rejects_wrong_scalar_tag_version_lengths_and_hashes() {
        for (field, bad) in [
            ("version", json!("1")), ("version", json!(1.0)), ("version", json!(2)),
            ("resource_value_tag", json!("0C")), ("resource_value_tag", json!("0a")),
            ("frame_byte_length", json!(22.0)), ("payload_byte_length", json!(-1)),
            ("payload_byte_length", json!(3)), ("frame_byte_length", json!(23)),
            ("payload_sha256", json!("0".repeat(64))), ("frame_sha256", json!("0".repeat(64))),
            ("frame_sha256", json!(leaf()["frame_sha256"].as_str().unwrap().to_uppercase())),
            ("payload_sha256", leaf()["frame_sha256"].clone()),
            ("frame_base64url", json!([0, 255])),
        ] {
            let mut value = leaf(); value[field] = bad;
            assert!(resource_value_from_detjson(&value).is_err(), "accepted: {value}");
            assert!(parse_resource_snapshot_json(&value.to_string()).is_err());
        }
    }

    #[test]
    fn bytes_transport_rejects_noncanonical_base64_before_frame_materialization() {
        let encoded = leaf()["frame_base64url"].as_str().unwrap().to_owned();
        for bad in [format!("{encoded}="), format!(" {encoded}"), encoded.replace('_', "/"),
            encoded[..encoded.len()-1].to_owned() + "x", "A".repeat(1_333_361)] {
            let mut value = leaf(); value["frame_base64url"] = json!(bad);
            assert!(resource_value_from_detjson(&value).is_err());
        }
        let maximum = ResourceValue::Bytes(ByteSequence::new(vec![255; 1_000_000]));
        let json = resource_value_to_detjson(&maximum);
        assert_eq!(json["frame_base64url"].as_str().unwrap().len(), 1_333_360);
        assert_eq!(resource_value_from_detjson(&json).unwrap(), maximum);
        let too_large = ResourceValue::Bytes(ByteSequence::new(vec![0; 1_000_001]));
        assert!(resource_value_from_detjson(&resource_value_to_detjson(&too_large)).is_err());
        assert!(parse_resource_snapshot_json(&" ".repeat(SNAPSHOT_MAX_BYTES + 1)).is_err());
    }

    #[test]
    fn nested_bytes_use_one_verifier_without_string_list_coercion() {
        let bytes = ResourceValue::Bytes(ByteSequence::new(vec![0, 255]));
        let wrappers = [
            ResourceValue::List(vec![bytes.clone()]),
            ResourceValue::set_from_values(vec![bytes.clone()]),
            ResourceValue::map_from_entries(vec![ResourceMapEntry { key: bytes.clone(), value: bytes.clone() }]),
            ResourceValue::Record {
                field_types: BTreeMap::from([("자료".into(), RecordFieldType::Named { name: "바이트열".into() })]),
                fields: BTreeMap::from([("자료".into(), bytes.clone())]),
            },
        ];
        for value in wrappers {
            let raw = serialize_resource_snapshot_json(&resource_value_to_detjson(&value));
            assert_eq!(resource_value_from_detjson(&parse_resource_snapshot_json(&raw).unwrap()).unwrap(), value);
            let corrupt = raw.replace("\"resource_value_tag\":\"0c\"", "\"resource_value_tag\":\"0a\"");
            assert!(parse_resource_snapshot_json(&corrupt).is_err());
        }
        for value in [ResourceValue::String("바이트열{0x00, 0xff}".into()),
            ResourceValue::List(vec![ResourceValue::Fixed64(Fixed64::ZERO)])] {
            assert_ne!(value, bytes);
            assert_eq!(resource_value_from_detjson(&resource_value_to_detjson(&value)).unwrap(), value);
        }
        // No new compatibility policy for legacy non-Bytes JSON.
        assert_eq!(parse_resource_snapshot_json("{\"x\":1,\"x\":2}").unwrap(), json!({"x":2}));
    }
}

#[cfg(test)]
impl<'de> serde::Deserialize<'de> for SnapshotJson {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SnapshotVisitor;
        impl<'de> serde::de::Visitor<'de> for SnapshotVisitor {
            type Value = SnapshotJson;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("저장 JSON 값") }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> { Ok(SnapshotJson(JsonValue::Null)) }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> { Ok(SnapshotJson(v.into())) }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> { Ok(SnapshotJson(v.into())) }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> { Ok(SnapshotJson(v.into())) }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(v).map(|n| SnapshotJson(JsonValue::Number(n)))
                    .ok_or_else(|| E::custom(invalid_bytes_snapshot()))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> { Ok(SnapshotJson(v.into())) }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> { Ok(SnapshotJson(v.into())) }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<SnapshotJson>()? { values.push(value.0); }
                Ok(SnapshotJson(JsonValue::Array(values)))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut keys = Vec::new();
                let mut object = serde_json::Map::new();
                let mut bytes_leaf = false;
                while let Some((key, value)) = map.next_entry::<String, SnapshotJson>()? {
                    bytes_leaf |= key == "type" && value.0.as_str() == Some("bytes_v1");
                    keys.push(key.clone());
                    object.insert(key, value.0);
                }
                if bytes_leaf {
                    if !keys.iter().map(String::as_str).eq(BYTES_FIELDS) {
                        return Err(serde::de::Error::custom(invalid_bytes_snapshot()));
                    }
                    parse_bytes_v1(&object).map_err(serde::de::Error::custom)?;
                }
                Ok(SnapshotJson(JsonValue::Object(object)))
            }
        }
        deserializer.deserialize_any(SnapshotVisitor)
    }
}
