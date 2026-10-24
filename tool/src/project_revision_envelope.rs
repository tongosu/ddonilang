//! RD-631 raw revision byte owner, selected from fixed c9ec1798 host ingress.
//! Structural/integrity checks alone are not source/PESI admission.
use std::collections::{BTreeMap, BTreeSet};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1;
const MAX_REVISION_ENVELOPE_BYTES: usize = 1_000_000;

pub(crate) fn verify_rd631_canonical_envelope_bytes(
    bytes: &[u8],
) -> Result<JsonValue, SnapshotV2DurabilityFailureV1> {
    use SnapshotV2DurabilityFailureV1::*;
    if bytes.is_empty()
        || bytes.len() > MAX_REVISION_ENVELOPE_BYTES
        || bytes.starts_with(&[0xef, 0xbb, 0xbf])
        || !bytes.ends_with(b"\n")
        || bytes.ends_with(b"\r\n")
        || bytes.get(bytes.len().saturating_sub(2)) == Some(&b'\n')
    {
        return Err(InvalidInput);
    }
    let json_bytes = &bytes[..bytes.len() - 1];
    let mut deserializer = serde_json::Deserializer::from_slice(json_bytes);
    let value = StrictJson::deserialize(&mut deserializer)
        .map_err(|_| InvalidInput)?
        .0;
    deserializer.end().map_err(|_| InvalidInput)?;
    let mut canonical = serde_json::to_vec(&value).map_err(|_| InvalidInput)?;
    canonical.push(b'\n');
    if canonical != bytes {
        return Err(InvalidInput);
    }
    Ok(value)
}

/// RD-631's closed record grammar, not an identity issuer or a source-blob
/// verifier. Identity values stay opaque here; the shared identity owners must
/// still admit them and compare their canonical bytes before durable I/O.
pub(crate) fn verify_rd631_envelope_structure(
    envelope: &JsonValue,
) -> Result<(), SnapshotV2DurabilityFailureV1> {
    use SnapshotV2DurabilityFailureV1::InvalidInput;
    exact_members(
        envelope,
        &[
            "schema",
            "project_identity",
            "history_revision_identity",
            "expected_parent_revision_identity",
            "source_epoch_identity",
            "project_semantic_configuration_identity",
            "official_entry_path",
            "official_entry_definition_identity",
            "source_manifest",
            "dependency_resolution_identity",
            "declared_resource_members",
            "revision_integrity_sha256",
        ],
    )?;
    if envelope["schema"] != "ddn.workbench.project_snapshot_revision.v1" {
        return Err(InvalidInput);
    }
    for key in [
        "project_identity",
        "history_revision_identity",
        "source_epoch_identity",
        "project_semantic_configuration_identity",
        "official_entry_definition_identity",
    ] {
        if envelope[key].is_null() {
            return Err(InvalidInput);
        }
    }
    let entry = exact_project_path(&envelope["official_entry_path"])?;
    let manifest = &envelope["source_manifest"];
    exact_members(
        manifest,
        &[
            "schema",
            "kind",
            "root_path",
            "entry_path",
            "file_count",
            "total_utf8_bytes",
            "files",
            "project_graph_sha256",
            "lock_sha256",
            "canonical_manifest_sha256",
        ],
    )?;
    if manifest["schema"] != "ddn.virtual_project_manifest.v1"
        || manifest["kind"] != "seamgrim.local_project_tree.v0"
        || exact_project_path(&manifest["entry_path"])? != entry
    {
        return Err(InvalidInput);
    }
    let root = exact_project_path(&manifest["root_path"])?;
    let files = manifest["files"].as_array().ok_or(InvalidInput)?;
    if manifest["file_count"].as_u64() != Some(files.len() as u64) {
        return Err(InvalidInput);
    }
    let mut previous = None;
    let mut folded_paths = BTreeSet::new();
    let mut total = 0u64;
    let mut has_root = false;
    let mut has_entry = false;
    for file in files {
        exact_members(file, &["path", "role", "byte_length", "sha256"])?;
        let path = exact_project_path(&file["path"])?;
        if previous.is_some_and(|before| before >= path)
            || !folded_paths.insert(path.to_ascii_lowercase())
        {
            return Err(InvalidInput);
        }
        previous = Some(path);
        let role = file["role"].as_str().ok_or(InvalidInput)?;
        if !matches!(
            role,
            "root" | "ddn_source" | "project_json" | "lock" | "gaji_manifest" | "metadata"
        ) {
            return Err(InvalidInput);
        }
        total = total
            .checked_add(file["byte_length"].as_u64().ok_or(InvalidInput)?)
            .ok_or(InvalidInput)?;
        digest32(&file["sha256"])?;
        has_root |= path == root && role == "root";
        has_entry |= path == entry && role == "ddn_source";
    }
    if !has_root || !has_entry || manifest["total_utf8_bytes"].as_u64() != Some(total) {
        return Err(InvalidInput);
    }
    digest32(&manifest["project_graph_sha256"])?;
    if !manifest["lock_sha256"].is_null() {
        digest32(&manifest["lock_sha256"])?;
    }
    let members = envelope["declared_resource_members"]
        .as_array()
        .ok_or(InvalidInput)?;
    let mut previous_resource_path = None;
    for member in members {
        exact_members(member, &["logical_path", "content_identity"])?;
        let path = exact_project_path(&member["logical_path"])?;
        // Equal paths require the content-identity owner's secondary ordering;
        // this structural gate does not invent an identity ordering or encoding.
        if previous_resource_path.is_some_and(|before| before > path)
            || member["content_identity"].is_null()
        {
            return Err(InvalidInput);
        }
        previous_resource_path = Some(path);
    }
    Ok(())
}

fn exact_members(value: &JsonValue, names: &[&str]) -> Result<(), SnapshotV2DurabilityFailureV1> {
    use SnapshotV2DurabilityFailureV1::InvalidInput;
    let object = value.as_object().ok_or(InvalidInput)?;
    if object.len() != names.len() || names.iter().any(|name| !object.contains_key(*name)) {
        return Err(InvalidInput);
    }
    Ok(())
}

fn exact_project_path(value: &JsonValue) -> Result<&str, SnapshotV2DurabilityFailureV1> {
    use SnapshotV2DurabilityFailureV1::InvalidInput;
    let path = value.as_str().ok_or(InvalidInput)?;
    if crate::local_project::normalize_project_path(path).map_err(|_| InvalidInput)? != path {
        return Err(InvalidInput);
    }
    Ok(path)
}

fn digest32(value: &JsonValue) -> Result<[u8; 32], SnapshotV2DurabilityFailureV1> {
    use SnapshotV2DurabilityFailureV1::InvalidInput;
    let text = value.as_str().ok_or(InvalidInput)?;
    if text.len() != 64
        || !text
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(InvalidInput);
    }
    let mut digest = [0; 32];
    hex::decode_to_slice(text, &mut digest).map_err(|_| InvalidInput)?;
    Ok(digest)
}

pub(crate) fn verify_self_empty_digest(
    value: &JsonValue,
    field: &str,
    mismatch: SnapshotV2DurabilityFailureV1,
) -> Result<[u8; 32], SnapshotV2DurabilityFailureV1> {
    use SnapshotV2DurabilityFailureV1::InvalidInput;
    let claimed = digest32(&value[field])?;
    let mut empty = value.clone();
    *empty.get_mut(field).ok_or(InvalidInput)? = JsonValue::String(String::new());
    let mut bytes = serde_json::to_vec(&empty).map_err(|_| InvalidInput)?;
    bytes.push(b'\n');
    if Sha256::digest(&bytes).as_slice() != claimed {
        return Err(mismatch);
    }
    Ok(claimed)
}

struct StrictJson(JsonValue);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rd631_raw_bytes_never_repair_or_collapse() {
        assert_eq!(verify_rd631_canonical_envelope_bytes(b"{\"a\":1,\"b\":2}\n").unwrap()["a"],1);
        for bad in [b"{}".as_slice(),b"{}\r\n",b"{}\n\n",b" { }\n",b"{\"b\":2,\"a\":1}\n",b"{\"a\":1,\"a\":2}\n",b"{\"a\":1.0}\n",b"\xef\xbb\xbf{}\n",b"{}\ntrailing"] {
            assert!(verify_rd631_canonical_envelope_bytes(bad).is_err(),"{bad:?}");
        }
    }
    #[test]
    fn rd631_self_empty_digest_is_exact_and_not_an_identity_issuer() {
        let mut value = serde_json::json!({"a":1,"integrity":""});
        let mut raw = serde_json::to_vec(&value).unwrap(); raw.push(b'\n');
        value["integrity"] = hex::encode(Sha256::digest(raw)).into();
        verify_self_empty_digest(&value,"integrity",SnapshotV2DurabilityFailureV1::CustodyConflict).unwrap();
        value["a"] = 2.into();
        assert!(verify_self_empty_digest(&value,"integrity",SnapshotV2DurabilityFailureV1::CustodyConflict).is_err());
    }
    #[test]
    fn rd631_byte_valid_transport_fixture_is_not_a_revision() {
        let fixture = verify_rd631_canonical_envelope_bytes(b"{\"project\":\"p\",\"schema\":\"rd631.fixture.transport-only\"}\n").unwrap();
        assert!(verify_rd631_envelope_structure(&fixture).is_err());
    }
}

impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictJson;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("canonical integer-only JSON")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictJson(JsonValue::Null))
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                self.visit_unit()
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictJson(JsonValue::Bool(value)))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictJson(JsonValue::Number(value.into())))
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictJson(JsonValue::Number(value.into())))
            }
            fn visit_f64<E: serde::de::Error>(self, _value: f64) -> Result<Self::Value, E> {
                Err(E::custom("non-integer JSON number"))
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StrictJson(JsonValue::String(value.to_owned())))
            }
            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictJson(JsonValue::String(value)))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(StrictJson(value)) = seq.next_element()? {
                    values.push(value);
                }
                Ok(StrictJson(JsonValue::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = BTreeMap::new();
                while let Some((key, StrictJson(value))) = map.next_entry::<String, StrictJson>()? {
                    if values.insert(key, value).is_some() {
                        return Err(serde::de::Error::custom("duplicate JSON member"));
                    }
                }
                Ok(StrictJson(JsonValue::Object(values.into_iter().collect())))
            }
        }
        deserializer.deserialize_any(StrictVisitor)
    }
}
