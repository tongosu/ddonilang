//! Shared V1 declared-host-resource authority.
//!
//! A host locator is deliberately an adapter-local detail.  The replay and
//! receipt identities below contain only the declared logical identity and the
//! sealed UTF-8 content; replay never receives a locator or touches the host.

use std::collections::BTreeSet;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};

#[cfg(not(target_arch = "wasm32"))]
use crate::artifact_output::write_text_artifact_atomic;

pub const HOST_RESOURCE_MANIFEST_SCHEMA: &str = "ddn.host_resource_manifest.v1";
pub const HOST_RESOURCE_REQUEST_SCHEMA: &str = "ddn.host_resource_request.v1";
pub const HOST_RESOURCE_TRANSCRIPT_SCHEMA: &str = "ddn.host_resource_transcript.v1";
pub const HOST_RESOURCE_RECEIPT_SCHEMA: &str = "ddn.host_resource_receipt.v1";
pub const HOST_RESOURCE_BUNDLE_SCHEMA: &str = "ddn.host_resource_bundle.v1";
pub const V1_TEXT_MEDIA_TYPE: &str = "text/plain; charset=utf-8";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeclaredHostResource {
    pub logical_resource_id: String,
    /// Adapter-local only.  It is intentionally excluded from all identities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
    pub sha256: String,
    pub byte_length: u64,
    pub media_type: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HostResourceManifest {
    schema: String,
    resources: Vec<DeclaredHostResource>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostResourcePayload {
    pub logical_resource_id: String,
    pub sha256: String,
    pub byte_length: u64,
    pub media_type: String,
    pub utf8_text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostResourceTranscript {
    pub schema: String,
    pub resources: Vec<HostResourcePayload>,
    pub transcript_identity: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostResourceReceipt {
    pub schema: String,
    pub transcript_sha256: String,
    pub resource_count: u64,
    pub receipt_identity: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostResourceRequest {
    pub schema: String,
    pub mode: String,
    #[serde(default)]
    pub resources: Vec<HostResourcePayload>,
    #[serde(default)]
    pub transcript: Option<HostResourceTranscript>,
}

#[derive(Clone, Debug)]
pub struct PreparedHostResources {
    pub transcript: HostResourceTranscript,
    pub receipt: HostResourceReceipt,
}

pub fn parse_request_json(input: &str) -> Result<HostResourceRequest, String> {
    serde_json::from_str(input).map_err(|error| format!("E_HOST_RESOURCE_REQUEST_JSON {error}"))
}

pub fn prepare_request(request: &HostResourceRequest) -> Result<PreparedHostResources, String> {
    if request.schema != HOST_RESOURCE_REQUEST_SCHEMA {
        return Err(format!("E_HOST_RESOURCE_REQUEST_SCHEMA {}", request.schema));
    }
    match request.mode.as_str() {
        "record" => {
            if request.transcript.is_some() {
                return Err("E_HOST_RESOURCE_RECORD_TRANSCRIPT_FORBIDDEN".to_string());
            }
            prepare_transcript(new_transcript(request.resources.clone())?)
        }
        "replay" => {
            if !request.resources.is_empty() {
                return Err("E_HOST_RESOURCE_REPLAY_PAYLOAD_FORBIDDEN".to_string());
            }
            let transcript = request
                .transcript
                .clone()
                .ok_or_else(|| "E_HOST_RESOURCE_REPLAY_TRANSCRIPT_MISSING".to_string())?;
            prepare_transcript(transcript)
        }
        "deny" => {
            if !request.resources.is_empty() || request.transcript.is_some() {
                return Err("E_HOST_RESOURCE_DENY_INPUT_FORBIDDEN".to_string());
            }
            prepare_transcript(new_transcript(Vec::new())?)
        }
        _ => Err(format!("E_HOST_RESOURCE_MODE {}", request.mode)),
    }
}

pub fn transcript_json(prepared: &PreparedHostResources) -> Result<String, String> {
    canonical_json(&prepared.transcript)
}

pub fn bundle_json(prepared: &PreparedHostResources) -> Result<String, String> {
    canonical_json(&json!({
        "schema": HOST_RESOURCE_BUNDLE_SCHEMA,
        "transcript": prepared.transcript,
        "receipt": prepared.receipt,
    }))
}

pub fn prepare_replay_from_transcript_json(input: &str) -> Result<PreparedHostResources, String> {
    let transcript: HostResourceTranscript = serde_json::from_str(input)
        .map_err(|error| format!("E_HOST_RESOURCE_TRANSCRIPT_JSON {error}"))?;
    prepare_request(&HostResourceRequest {
        schema: HOST_RESOURCE_REQUEST_SCHEMA.to_string(),
        mode: "replay".to_string(),
        resources: Vec::new(),
        transcript: Some(transcript),
    })
}

pub fn prepare_replay_from_bundle_json(input: &str) -> Result<PreparedHostResources, String> {
    let value: JsonValue = serde_json::from_str(input)
        .map_err(|error| format!("E_HOST_RESOURCE_BUNDLE_JSON {error}"))?;
    if value.get("schema").and_then(JsonValue::as_str) != Some(HOST_RESOURCE_BUNDLE_SCHEMA) {
        return Err("E_HOST_RESOURCE_BUNDLE_SCHEMA".to_string());
    }
    let transcript: HostResourceTranscript = serde_json::from_value(
        value.get("transcript").cloned().unwrap_or(JsonValue::Null),
    )
    .map_err(|error| format!("E_HOST_RESOURCE_BUNDLE_TRANSCRIPT {error}"))?;
    let declared_receipt: HostResourceReceipt = serde_json::from_value(
        value.get("receipt").cloned().unwrap_or(JsonValue::Null),
    )
    .map_err(|error| format!("E_HOST_RESOURCE_BUNDLE_RECEIPT {error}"))?;
    let prepared = prepare_replay_from_transcript_json(&canonical_json(&transcript)?)?;
    if prepared.receipt != declared_receipt {
        return Err("E_HOST_RESOURCE_BUNDLE_RECEIPT_MISMATCH".to_string());
    }
    Ok(prepared)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn prepare_record_from_manifest_path(path: &Path) -> Result<PreparedHostResources, String> {
    let manifest_text = fs::read_to_string(path)
        .map_err(|error| format!("E_HOST_RESOURCE_MANIFEST_READ {} {error}", path.display()))?;
    let manifest: HostResourceManifest = serde_json::from_str(&manifest_text)
        .map_err(|error| format!("E_HOST_RESOURCE_MANIFEST_JSON {error}"))?;
    if manifest.schema != HOST_RESOURCE_MANIFEST_SCHEMA {
        return Err(format!("E_HOST_RESOURCE_MANIFEST_SCHEMA {}", manifest.schema));
    }
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let mut payloads = Vec::with_capacity(manifest.resources.len());
    let mut ids = BTreeSet::new();
    for declared in manifest.resources {
        validate_declared(&declared, &mut ids)?;
        let locator = declared
            .locator
            .as_deref()
            .ok_or_else(|| format!("E_HOST_RESOURCE_LOCATOR_MISSING {}", declared.logical_resource_id))?;
        let resource_path = resolve_relative_regular_file(root, locator)?;
        let bytes = fs::read(&resource_path).map_err(|error| {
            format!("E_HOST_RESOURCE_READ {} {error}", resource_path.display())
        })?;
        let utf8_text = String::from_utf8(bytes.clone()).map_err(|_| {
            format!("E_HOST_RESOURCE_UTF8 {}", declared.logical_resource_id)
        })?;
        let actual_sha256 = sha256_hex(&bytes);
        if actual_sha256 != declared.sha256 {
            return Err(format!(
                "E_HOST_RESOURCE_HASH_MISMATCH {} expected={} actual={}",
                declared.logical_resource_id, declared.sha256, actual_sha256
            ));
        }
        if bytes.len() as u64 != declared.byte_length {
            return Err(format!(
                "E_HOST_RESOURCE_LENGTH_MISMATCH {} expected={} actual={}",
                declared.logical_resource_id,
                declared.byte_length,
                bytes.len()
            ));
        }
        payloads.push(HostResourcePayload {
            logical_resource_id: declared.logical_resource_id,
            sha256: declared.sha256,
            byte_length: declared.byte_length,
            media_type: declared.media_type,
            utf8_text,
        });
    }
    prepare_request(&HostResourceRequest {
        schema: HOST_RESOURCE_REQUEST_SCHEMA.to_string(),
        mode: "record".to_string(),
        resources: payloads,
        transcript: None,
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_replay_artifact(path: &Path, bundle: bool) -> Result<PreparedHostResources, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("E_HOST_RESOURCE_REPLAY_READ {} {error}", path.display()))?;
    if bundle {
        prepare_replay_from_bundle_json(&text)
    } else {
        prepare_replay_from_transcript_json(&text)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_transcript_atomic(path: &Path, prepared: &PreparedHostResources) -> Result<(), String> {
    write_text_artifact_atomic(path, &format!("{}\n", transcript_json(prepared)?))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_bundle_atomic(path: &Path, prepared: &PreparedHostResources) -> Result<(), String> {
    write_text_artifact_atomic(path, &format!("{}\n", bundle_json(prepared)?))
}

fn prepare_transcript(transcript: HostResourceTranscript) -> Result<PreparedHostResources, String> {
    if transcript.schema != HOST_RESOURCE_TRANSCRIPT_SCHEMA {
        return Err(format!("E_HOST_RESOURCE_TRANSCRIPT_SCHEMA {}", transcript.schema));
    }
    let mut ids = BTreeSet::new();
    for payload in &transcript.resources {
        validate_payload(payload, &mut ids)?;
    }
    let expected_identity = transcript_identity(&transcript.schema, &transcript.resources)?;
    if transcript.transcript_identity != expected_identity {
        return Err("E_HOST_RESOURCE_TRANSCRIPT_IDENTITY_MISMATCH".to_string());
    }
    let transcript = HostResourceTranscript {
        schema: transcript.schema,
        resources: transcript.resources,
        transcript_identity: expected_identity,
    };
    let resource_count = transcript.resources.len() as u64;
    let transcript_sha256 = transcript.transcript_identity.clone();
    let receipt_identity = sha256_hex(
        canonical_json(&json!({
            "schema": HOST_RESOURCE_RECEIPT_SCHEMA,
            "transcript_sha256": transcript_sha256,
            "resource_count": transcript.resources.len(),
        }))?
        .as_bytes(),
    );
    Ok(PreparedHostResources {
        transcript,
        receipt: HostResourceReceipt {
            schema: HOST_RESOURCE_RECEIPT_SCHEMA.to_string(),
            transcript_sha256,
            resource_count,
            receipt_identity,
        },
    })
}

fn new_transcript(resources: Vec<HostResourcePayload>) -> Result<HostResourceTranscript, String> {
    let schema = HOST_RESOURCE_TRANSCRIPT_SCHEMA.to_string();
    let transcript_identity = transcript_identity(&schema, &resources)?;
    Ok(HostResourceTranscript {
        schema,
        resources,
        transcript_identity,
    })
}

fn transcript_identity(
    schema: &str,
    resources: &[HostResourcePayload],
) -> Result<String, String> {
    Ok(sha256_hex(
        canonical_json(&json!({
            "schema": schema,
            "resources": resources,
        }))?
        .as_bytes(),
    ))
}

fn validate_declared(declared: &DeclaredHostResource, ids: &mut BTreeSet<String>) -> Result<(), String> {
    if declared.logical_resource_id.trim().is_empty()
        || !ids.insert(declared.logical_resource_id.clone())
    {
        return Err("E_HOST_RESOURCE_ID_DUPLICATE_OR_EMPTY".to_string());
    }
    if declared.media_type != V1_TEXT_MEDIA_TYPE {
        return Err(format!(
            "E_HOST_RESOURCE_MEDIA_TYPE {} {}",
            declared.logical_resource_id, declared.media_type
        ));
    }
    validate_sha256(&declared.sha256, &declared.logical_resource_id)
}

fn validate_payload(payload: &HostResourcePayload, ids: &mut BTreeSet<String>) -> Result<(), String> {
    let declared = DeclaredHostResource {
        logical_resource_id: payload.logical_resource_id.clone(),
        locator: None,
        sha256: payload.sha256.clone(),
        byte_length: payload.byte_length,
        media_type: payload.media_type.clone(),
    };
    validate_declared(&declared, ids)?;
    let bytes = payload.utf8_text.as_bytes();
    if bytes.len() as u64 != payload.byte_length {
        return Err(format!("E_HOST_RESOURCE_LENGTH_MISMATCH {}", payload.logical_resource_id));
    }
    if sha256_hex(bytes) != payload.sha256 {
        return Err(format!("E_HOST_RESOURCE_HASH_MISMATCH {}", payload.logical_resource_id));
    }
    Ok(())
}

fn validate_sha256(value: &str, id: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("E_HOST_RESOURCE_SHA256 {}", id));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_relative_regular_file(root: &Path, locator: &str) -> Result<PathBuf, String> {
    let relative = Path::new(locator);
    if locator.trim().is_empty()
        || relative.is_absolute()
        || locator.contains("://")
        || locator.starts_with("file:")
        || locator.contains(['*', '?', '[', ']'])
        || relative.components().any(|component| {
            matches!(
                component,
                Component::CurDir | Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("E_HOST_RESOURCE_LOCATOR_FORBIDDEN {locator}"));
    }
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("E_HOST_RESOURCE_LOCATOR_READ {} {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("E_HOST_RESOURCE_NOT_REGULAR_FILE {}", path.display()));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(format!("E_HOST_RESOURCE_REPARSE_FORBIDDEN {}", path.display()));
        }
    }
    Ok(path)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| format!("E_HOST_RESOURCE_SERIALIZE {error}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(id: &str, text: &str) -> HostResourcePayload {
        HostResourcePayload {
            logical_resource_id: id.to_string(),
            sha256: sha256_hex(text.as_bytes()),
            byte_length: text.len() as u64,
            media_type: V1_TEXT_MEDIA_TYPE.to_string(),
            utf8_text: text.to_string(),
        }
    }

    #[test]
    fn record_and_replay_have_one_locator_free_identity() {
        let recorded = prepare_request(&HostResourceRequest {
            schema: HOST_RESOURCE_REQUEST_SCHEMA.to_string(),
            mode: "record".to_string(),
            resources: vec![payload("caption", "안녕하세요\n")],
            transcript: None,
        })
        .expect("record");
        let replayed = prepare_replay_from_transcript_json(&transcript_json(&recorded).unwrap())
            .expect("replay");
        assert_eq!(recorded.receipt, replayed.receipt);
        assert_eq!(recorded.transcript, replayed.transcript);
    }

    #[test]
    fn replay_rejects_duplicate_and_tampered_payloads() {
        let one = payload("caption", "a");
        let duplicate = HostResourceTranscript {
            schema: HOST_RESOURCE_TRANSCRIPT_SCHEMA.to_string(),
            resources: vec![one.clone(), one],
            transcript_identity: String::new(),
        };
        assert!(prepare_replay_from_transcript_json(&canonical_json(&duplicate).unwrap())
            .unwrap_err()
            .contains("E_HOST_RESOURCE_ID_DUPLICATE_OR_EMPTY"));
        let mut tampered = payload("caption", "a");
        tampered.utf8_text = "b".to_string();
        let transcript = HostResourceTranscript {
            schema: HOST_RESOURCE_TRANSCRIPT_SCHEMA.to_string(),
            resources: vec![tampered],
            transcript_identity: String::new(),
        };
        assert!(prepare_replay_from_transcript_json(&canonical_json(&transcript).unwrap())
            .unwrap_err()
            .contains("E_HOST_RESOURCE_HASH_MISMATCH"));
    }

    #[test]
    fn replay_rejects_order_identity_tamper() {
        let prepared = prepare_request(&HostResourceRequest {
            schema: HOST_RESOURCE_REQUEST_SCHEMA.to_string(),
            mode: "record".to_string(),
            resources: vec![payload("one", "1"), payload("two", "2")],
            transcript: None,
        })
        .expect("record");
        let mut reordered = prepared.transcript.clone();
        reordered.resources.reverse();
        assert!(prepare_replay_from_transcript_json(&canonical_json(&reordered).unwrap())
            .unwrap_err()
            .contains("E_HOST_RESOURCE_TRANSCRIPT_IDENTITY_MISMATCH"));
    }
}
