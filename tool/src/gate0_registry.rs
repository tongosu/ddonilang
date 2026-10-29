use crate::sealed_domain_dataset::{
    apply_dataset_transform_recipe, derived_dataset_artifact_sha256,
    parse_and_validate_dataset_transform_recipe, parse_and_validate_sealed_domain_dataset,
    DerivedDatasetArtifact, SealedDomainDataset, DERIVED_DATASET_ARTIFACT_SCHEMA,
};
use blake3::hash;
use ddonirang_core::{set_unit_registry_symbols, unit_spec_from_symbol, ResourceHandle};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static ASSET_REGISTRY: OnceLock<ProjectAssetRegistry> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct ProjectAssetRegistry {
    bundle_id: String,
    manifest_sha256: String,
    entries: HashMap<String, AssetEntry>,
    paths_by_handle: HashMap<u64, String>,
}

#[derive(Debug, Clone)]
struct AssetEntry {
    handle: u64,
    blake3: String,
    mime: Option<String>,
    bytes: Vec<u8>,
    sealed_dataset: Option<SealedDatasetBindingRaw>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetManifest {
    version: String,
    bundle_id: String,
    hash_algo: String,
    entries: Vec<AssetEntryRaw>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetEntryRaw {
    path: String,
    handle: serde_json::Value,
    hash: String,
    size: u64,
    #[allow(dead_code)]
    mime: Option<String>,
    #[serde(default)]
    sealed_dataset: Option<SealedDatasetBindingRaw>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedDatasetBindingRaw {
    #[serde(default)]
    parent_envelope_path: Option<String>,
    #[serde(default)]
    recipe_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedSealedDataset {
    pub dataset: SealedDomainDataset,
    pub payload_bytes: Vec<u8>,
    pub asset_manifest_sha256: String,
    pub envelope_asset_blake3: String,
    pub envelope_sha256: String,
    pub payload_sha256: String,
    pub recipe_sha256: Option<String>,
    pub derived_artifact_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SealedDatasetAssetIdentity {
    pub path: String,
    pub handle: String,
    pub artifact_id: String,
    pub artifact_version: String,
    pub source: crate::sealed_domain_dataset::DatasetSource,
    pub asset_manifest_sha256: String,
    pub envelope_asset_blake3: String,
    pub envelope_sha256: String,
    pub content_sha256: String,
    pub schema_sha256: String,
    pub payload_sha256: String,
    pub recipe_sha256: Option<String>,
    pub derived_artifact_sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UnitsRegistry {
    version: Option<String>,
    units: Vec<String>,
}

pub(crate) const UNIT_REGISTRY_VERSION: &str = "v1";

pub fn ensure_gate0_registries() -> Result<(), String> {
    ensure_asset_registry_default()?;
    ensure_units_registry_default()?;
    Ok(())
}

pub fn resolve_asset_handle(path: &str) -> Result<ResourceHandle, String> {
    let Some(registry) = ASSET_REGISTRY.get() else {
        return Ok(ResourceHandle::from_path(path));
    };
    registry.resolve_handle(path)
}

pub fn open_sealed_dataset(handle: ResourceHandle) -> Result<OpenedSealedDataset, String> {
    let registry = ASSET_REGISTRY
        .get()
        .ok_or_else(|| "E_DATASET_ASSET_REGISTRY_MISSING: ddn.asset.json".to_string())?;
    registry.open_sealed_dataset(handle)
}

impl ProjectAssetRegistry {
    pub fn bundle_id(&self) -> &str {
        &self.bundle_id
    }

    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }

    pub fn sealed_dataset_identities(&self) -> Result<Vec<SealedDatasetAssetIdentity>, String> {
        let mut paths = self
            .entries
            .iter()
            .filter_map(|(path, entry)| {
                (entry.mime.as_deref() == Some("application/vnd.ddn.sealed-domain-dataset+json"))
                    .then(|| path.clone())
            })
            .collect::<Vec<_>>();
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                let entry = self
                    .entries
                    .get(&path)
                    .ok_or_else(|| format!("E_DATASET_OBJECT_MISSING: envelope asset {path}"))?;
                let handle = ResourceHandle::from_raw(entry.handle);
                let opened = self.open_sealed_dataset(handle)?;
                Ok(SealedDatasetAssetIdentity {
                    path,
                    handle: handle.to_hex(),
                    artifact_id: opened.dataset.artifact_id.clone(),
                    artifact_version: opened.dataset.artifact_version.clone(),
                    source: opened.dataset.source.clone(),
                    asset_manifest_sha256: opened.asset_manifest_sha256,
                    envelope_asset_blake3: opened.envelope_asset_blake3,
                    envelope_sha256: opened.envelope_sha256,
                    content_sha256: opened.dataset.content_sha256,
                    schema_sha256: opened.dataset.schema_sha256,
                    payload_sha256: opened.payload_sha256,
                    recipe_sha256: opened.recipe_sha256,
                    derived_artifact_sha256: opened.derived_artifact_sha256,
                })
            })
            .collect()
    }

    pub fn resolve_handle(&self, path: &str) -> Result<ResourceHandle, String> {
        let normalized = normalize_asset_path(path)?;
        let entry = self
            .entries
            .get(&normalized)
            .ok_or_else(|| format!("RESOURCE_NOT_FOUND: {normalized}"))?;
        Ok(ResourceHandle::from_raw(entry.handle))
    }

    pub fn sealed_dataset_path_for_handle(&self, handle: ResourceHandle) -> Result<String, String> {
        let envelope_path = self
            .paths_by_handle
            .get(&handle.raw())
            .ok_or_else(|| {
                format!(
                    "E_DATASET_HANDLE_UNVERIFIED: 자원핸들 {}은 project asset registry에 없습니다",
                    handle.to_hex()
                )
            })?
            .clone();
        self.open_sealed_dataset(handle)?;
        Ok(envelope_path)
    }

    pub fn open_sealed_dataset(
        &self,
        handle: ResourceHandle,
    ) -> Result<OpenedSealedDataset, String> {
        let envelope_path = self
            .paths_by_handle
            .get(&handle.raw())
            .ok_or_else(|| {
                format!(
                    "E_DATASET_HANDLE_UNVERIFIED: 자원핸들 {}은 project asset registry에 없습니다",
                    handle.to_hex()
                )
            })?
            .clone();
        let envelope_entry = self
            .entries
            .get(&envelope_path)
            .ok_or_else(|| format!("E_DATASET_OBJECT_MISSING: envelope asset {envelope_path}"))?;
        if envelope_entry.mime.as_deref() != Some("application/vnd.ddn.sealed-domain-dataset+json")
        {
            return Err(format!(
                "E_DATASET_ASSET_KIND: {envelope_path}는 sealed dataset envelope asset이 아닙니다"
            ));
        }
        let envelope_json = std::str::from_utf8(&envelope_entry.bytes)
            .map_err(|_| format!("E_DATASET_ENVELOPE_UTF8: {envelope_path}"))?;
        let preliminary: serde_json::Value = serde_json::from_str(envelope_json)
            .map_err(|err| format!("E_DATASET_JSON: {envelope_path}: {err}"))?;
        let payload_path = preliminary
            .get("payload")
            .and_then(|value| value.get("object_ref"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                format!("E_DATASET_OBJECT_REF: {envelope_path} payload.object_ref가 없습니다")
            })?;
        let payload_path = normalize_asset_path(payload_path)?;
        let payload_entry = self
            .entries
            .get(&payload_path)
            .ok_or_else(|| format!("E_DATASET_OBJECT_MISSING: payload asset {payload_path}"))?;
        let dataset = parse_and_validate_sealed_domain_dataset(envelope_json, &payload_entry.bytes)
            .map_err(|err| err.to_string())?;
        if dataset.payload.sha256 != sha256_hex(&payload_entry.bytes) {
            return Err(format!(
                "E_DATASET_PAYLOAD_HASH_MISMATCH: manifest payload {payload_path}"
            ));
        }

        let mut recipe_sha256 = None;
        let mut derived_artifact_sha256_value = None;
        if let Some(binding) = &envelope_entry.sealed_dataset {
            match (&binding.parent_envelope_path, &binding.recipe_path) {
                (None, None) => {}
                (Some(parent_path), Some(recipe_path)) => {
                    let parent_path = normalize_asset_path(parent_path)?;
                    let recipe_path = normalize_asset_path(recipe_path)?;
                    if parent_path == envelope_path {
                        return Err(
                            "E_DATASET_DERIVATION_CYCLE: derived envelope가 자신을 parent로 참조합니다"
                                .to_string(),
                        );
                    }
                    let parent_entry = self.entries.get(&parent_path).ok_or_else(|| {
                        format!("E_DATASET_OBJECT_MISSING: parent envelope asset {parent_path}")
                    })?;
                    let parent_json = std::str::from_utf8(&parent_entry.bytes)
                        .map_err(|_| format!("E_DATASET_ENVELOPE_UTF8: {parent_path}"))?;
                    let parent_value: serde_json::Value = serde_json::from_str(parent_json)
                        .map_err(|err| format!("E_DATASET_JSON: {parent_path}: {err}"))?;
                    let parent_payload_path = parent_value
                        .get("payload")
                        .and_then(|value| value.get("object_ref"))
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| {
                            format!(
                                "E_DATASET_OBJECT_REF: {parent_path} payload.object_ref가 없습니다"
                            )
                        })?;
                    let parent_payload_path = normalize_asset_path(parent_payload_path)?;
                    let parent_payload_entry =
                        self.entries.get(&parent_payload_path).ok_or_else(|| {
                            format!(
                                "E_DATASET_OBJECT_MISSING: parent payload asset {parent_payload_path}"
                            )
                        })?;
                    let parent = parse_and_validate_sealed_domain_dataset(
                        parent_json,
                        &parent_payload_entry.bytes,
                    )
                    .map_err(|err| err.to_string())?;
                    let recipe_entry = self.entries.get(&recipe_path).ok_or_else(|| {
                        format!("E_DATASET_OBJECT_MISSING: recipe asset {recipe_path}")
                    })?;
                    let recipe_json = std::str::from_utf8(&recipe_entry.bytes)
                        .map_err(|_| format!("E_DATASET_TRANSFORM_RECIPE_UTF8: {recipe_path}"))?;
                    let recipe = parse_and_validate_dataset_transform_recipe(recipe_json)
                        .map_err(|err| err.to_string())?;
                    let derived = apply_dataset_transform_recipe(
                        &parent,
                        &parent_payload_entry.bytes,
                        &recipe,
                    )
                    .map_err(|err| err.to_string())?;
                    if derived.dataset != dataset {
                        return Err(format!(
                            "E_DATASET_DERIVED_CONTENT_MISMATCH: {envelope_path}가 parent+recipe 결과와 다릅니다"
                        ));
                    }
                    if parent_payload_entry.bytes != payload_entry.bytes {
                        return Err(format!(
                            "E_DATASET_DERIVED_PAYLOAD_MISMATCH: {envelope_path} payload가 parent bytes와 다릅니다"
                        ));
                    }
                    recipe_sha256 = Some(derived.recipe_sha256.clone());
                    derived_artifact_sha256_value =
                        Some(derived_dataset_artifact_sha256(&DerivedDatasetArtifact {
                            schema: DERIVED_DATASET_ARTIFACT_SCHEMA.to_string(),
                            parent_content_sha256: parent.content_sha256,
                            recipe_sha256: derived.recipe_sha256,
                            dataset: dataset.clone(),
                            artifact_sha256: String::new(),
                        })
                        .map_err(|err| err.to_string())?);
                }
                _ => {
                    return Err(
                        "E_DATASET_DERIVATION_BINDING: parent_envelope_path와 recipe_path는 함께 있어야 합니다"
                            .to_string(),
                    )
                }
            }
        }
        Ok(OpenedSealedDataset {
            dataset,
            payload_bytes: payload_entry.bytes.clone(),
            asset_manifest_sha256: self.manifest_sha256.clone(),
            envelope_asset_blake3: envelope_entry.blake3.clone(),
            envelope_sha256: sha256_hex(&envelope_entry.bytes),
            payload_sha256: sha256_hex(&payload_entry.bytes),
            recipe_sha256,
            derived_artifact_sha256: derived_artifact_sha256_value,
        })
    }
}

fn ensure_asset_registry_default() -> Result<(), String> {
    let manifest_path = Path::new("ddn.asset.json");
    let manifest_path = if manifest_path.exists() {
        manifest_path
    } else {
        let legacy_path = Path::new("ddn.resource.json");
        if legacy_path.exists() {
            eprintln!(
                "경고: ddn.resource.json은 레거시 별칭입니다. ddn.asset.json으로 교체하세요."
            );
            legacy_path
        } else {
            return Err("RESOURCE_REGISTRY_MISSING: ddn.asset.json".to_string());
        }
    };
    let registry = load_asset_registry(manifest_path, true)?;
    ASSET_REGISTRY
        .set(registry)
        .map_err(|_| "쓸감 곳간은 한 번만 초기화할 수 있습니다".to_string())?;
    Ok(())
}

fn load_asset_registry(
    manifest_path: &Path,
    strict_hash: bool,
) -> Result<ProjectAssetRegistry, String> {
    if !manifest_path.exists() {
        return Err("RESOURCE_REGISTRY_MISSING: ddn.asset.json".to_string());
    }
    let raw = fs::read(manifest_path).map_err(|e| format!("ASSET_MANIFEST_INVALID: {e}"))?;
    let manifest_dir = manifest_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let manifest: AssetManifest =
        serde_json::from_slice(&raw).map_err(|e| format!("ASSET_MANIFEST_INVALID: {e}"))?;
    let mut files = HashMap::new();
    for entry in &manifest.entries {
        let normalized = normalize_asset_path(&entry.path)?;
        let full_path = manifest_dir.join(&normalized);
        let data = fs::read(&full_path)
            .map_err(|_| format!("RESOURCE_HASH_MISMATCH: missing {normalized}"))?;
        files.insert(normalized, data);
    }
    build_project_asset_registry_from_manifest(&raw, &files, strict_hash)
}

pub fn build_project_asset_registry_from_manifest(
    manifest_bytes: &[u8],
    files: &HashMap<String, Vec<u8>>,
    strict_hash: bool,
) -> Result<ProjectAssetRegistry, String> {
    let manifest: AssetManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|e| format!("ASSET_MANIFEST_INVALID: {e}"))?;
    if manifest.version != "v0" {
        return Err(format!(
            "ASSET_MANIFEST_INVALID: version {}",
            manifest.version
        ));
    }
    if manifest.bundle_id.trim().is_empty() {
        return Err("ASSET_MANIFEST_INVALID: bundle_id".to_string());
    }
    if manifest.hash_algo != "blake3" {
        return Err(format!(
            "ASSET_MANIFEST_INVALID: hash_algo {}",
            manifest.hash_algo
        ));
    }
    let mut entries = HashMap::new();
    let mut paths_by_handle = HashMap::new();
    let mut last_path = None::<String>;
    for entry in manifest.entries {
        let normalized = normalize_asset_path(&entry.path)?;
        if let Some(prev) = &last_path {
            if normalized < *prev {
                return Err("ASSET_MANIFEST_INVALID: entries not sorted".to_string());
            }
        }
        last_path = Some(normalized.clone());
        let handle = parse_handle(&entry.handle)?;
        let expected =
            ddonirang_core::asset_handle_from_bundle_path(&manifest.bundle_id, &normalized).raw();
        if handle != expected {
            return Err(format!(
                "ASSET_MANIFEST_INVALID: handle mismatch for {normalized}"
            ));
        }
        let data = files
            .get(&normalized)
            .ok_or_else(|| format!("RESOURCE_HASH_MISMATCH: missing {normalized}"))?;
        let actual_hash = hash(data).to_hex().to_string();
        if strict_hash && !hash_eq(&entry.hash, &actual_hash) {
            return Err(format!("RESOURCE_HASH_MISMATCH: {normalized}"));
        }
        if strict_hash && entry.size != data.len() as u64 {
            return Err(format!("RESOURCE_HASH_MISMATCH: {normalized}"));
        }
        if paths_by_handle.insert(handle, normalized.clone()).is_some() {
            return Err("ASSET_MANIFEST_INVALID: duplicate handle".to_string());
        }
        entries.insert(
            normalized,
            AssetEntry {
                handle,
                blake3: actual_hash,
                mime: entry.mime,
                bytes: data.clone(),
                sealed_dataset: entry.sealed_dataset,
            },
        );
    }
    Ok(ProjectAssetRegistry {
        bundle_id: manifest.bundle_id,
        manifest_sha256: sha256_hex(manifest_bytes),
        entries,
        paths_by_handle,
    })
}

pub(crate) fn load_unit_registry_symbols(path: &Path) -> Result<Vec<String>, String> {
    if !path.exists() {
        return Err(format!("UNIT_REGISTRY_MISSING: {}", path.display()));
    }
    let raw = fs::read(path).map_err(|e| format!("UNIT_REGISTRY_INVALID: {}", e))?;
    let registry: UnitsRegistry =
        serde_json::from_slice(&raw).map_err(|e| format!("UNIT_REGISTRY_INVALID: {}", e))?;
    if let Some(version) = &registry.version {
        if version != UNIT_REGISTRY_VERSION {
            return Err(format!("UNIT_REGISTRY_INVALID: version {}", version));
        }
    }
    let mut symbols = HashSet::new();
    for unit in registry.units {
        if !symbols.insert(unit.clone()) {
            return Err(format!("UNIT_REGISTRY_INVALID: duplicate {}", unit));
        }
        if unit_spec_from_symbol(&unit).is_none() {
            return Err(format!("UNIT_UNKNOWN: {}", unit));
        }
    }
    let mut symbols = symbols.into_iter().collect::<Vec<_>>();
    symbols.sort();
    Ok(symbols)
}

fn ensure_units_registry_default() -> Result<(), String> {
    let symbols = load_unit_registry_symbols(Path::new("ddn.units.json"))?;
    set_unit_registry_symbols(symbols.into_iter().collect())?;
    Ok(())
}

fn normalize_asset_path(path: &str) -> Result<String, String> {
    if path.is_empty() {
        return Err("ASSET_MANIFEST_INVALID: empty path".to_string());
    }
    if path.contains('\\') {
        return Err("ASSET_MANIFEST_INVALID: backslash".to_string());
    }
    let mut out = path.to_string();
    if out.starts_with("./") {
        out = out.trim_start_matches("./").to_string();
    }
    if out.starts_with('/') {
        return Err("ASSET_MANIFEST_INVALID: absolute path".to_string());
    }
    if out.ends_with('/') {
        return Err("ASSET_MANIFEST_INVALID: trailing slash".to_string());
    }
    let parts: Vec<&str> = out.split('/').collect();
    if parts.iter().any(|p| *p == "..") {
        return Err("ASSET_MANIFEST_INVALID: parent segment".to_string());
    }
    if parts.iter().any(|p| p.is_empty()) {
        return Err("ASSET_MANIFEST_INVALID: empty segment".to_string());
    }
    Ok(out)
}

fn parse_handle(value: &serde_json::Value) -> Result<u64, String> {
    match value {
        serde_json::Value::Number(num) => num
            .as_u64()
            .ok_or_else(|| "ASSET_MANIFEST_INVALID: handle".to_string()),
        serde_json::Value::String(s) => parse_handle_str(s),
        _ => Err("ASSET_MANIFEST_INVALID: handle".to_string()),
    }
}

fn parse_handle_str(value: &str) -> Result<u64, String> {
    let s = value.trim();
    if let Some(hex) = s.strip_prefix("0x") {
        return u64::from_str_radix(hex, 16)
            .map_err(|_| "ASSET_MANIFEST_INVALID: handle".to_string());
    }
    if s.len() == 16 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        return u64::from_str_radix(s, 16)
            .map_err(|_| "ASSET_MANIFEST_INVALID: handle".to_string());
    }
    s.parse::<u64>()
        .map_err(|_| "ASSET_MANIFEST_INVALID: handle".to_string())
}

fn hash_eq(expected: &str, actual: &str) -> bool {
    expected.trim().eq_ignore_ascii_case(actual.trim())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_in_unit_registry_matches_v1_runtime_contract() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .join("ddn.units.json");
        let raw = fs::read(&path).expect("checked-in unit registry");
        let registry: UnitsRegistry =
            serde_json::from_slice(&raw).expect("parse checked-in unit registry");

        assert_eq!(registry.version.as_deref(), Some(UNIT_REGISTRY_VERSION));
        load_unit_registry_symbols(&path).expect("checked-in unit registry must validate");
    }

    fn single_asset_manifest(bundle_id: &str, path: &str, handle: u64, bytes: &[u8]) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "version": "v0",
            "bundle_id": bundle_id,
            "hash_algo": "blake3",
            "entries": [{
                "path": path,
                "handle": format!("0x{handle:016x}"),
                "hash": blake3::hash(bytes).to_hex().to_string(),
                "size": bytes.len(),
                "mime": "application/vnd.ddn.sealed-domain-dataset+json"
            }]
        }))
        .expect("asset manifest")
    }

    #[test]
    fn project_asset_registry_rejects_forged_handle_and_missing_object() {
        let bundle_id = "ddn.w1.g12.asset-attacks";
        let path = "datasets/attack/envelope.json";
        let bytes = br#"{"schema":"ddn.sealed_domain_dataset.v1"}"#;
        let expected = ddonirang_core::asset_handle_from_bundle_path(bundle_id, path).raw();
        let files = HashMap::from([(path.to_string(), bytes.to_vec())]);

        let forged_manifest =
            single_asset_manifest(bundle_id, path, expected.wrapping_add(1), bytes);
        let forged_error =
            build_project_asset_registry_from_manifest(&forged_manifest, &files, true)
                .expect_err("forged resource handle must fail before registry construction");
        assert!(
            forged_error.contains("ASSET_MANIFEST_INVALID: handle mismatch"),
            "{forged_error}"
        );

        let valid_manifest = single_asset_manifest(bundle_id, path, expected, bytes);
        let missing_error =
            build_project_asset_registry_from_manifest(&valid_manifest, &HashMap::new(), true)
                .expect_err("manifest entry without object bytes must fail");
        assert_eq!(
            missing_error,
            format!("RESOURCE_HASH_MISMATCH: missing {path}")
        );
    }
}
