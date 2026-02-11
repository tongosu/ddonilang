use blake3::hash;
use ddonirang_core::{set_unit_registry_symbols, unit_spec_from_symbol, ResourceHandle};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static ASSET_REGISTRY: OnceLock<AssetRegistry> = OnceLock::new();

#[derive(Debug, Clone)]
struct AssetRegistry {
    entries: HashMap<String, AssetEntry>,
}

#[derive(Debug, Clone)]
struct AssetEntry {
    handle: u64,
}

#[derive(Debug, Deserialize)]
struct AssetManifest {
    version: String,
    bundle_id: String,
    hash_algo: String,
    entries: Vec<AssetEntryRaw>,
}

#[derive(Debug, Deserialize)]
struct AssetEntryRaw {
    path: String,
    handle: serde_json::Value,
    hash: String,
    size: u64,
    #[allow(dead_code)]
    mime: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UnitsRegistry {
    version: Option<String>,
    units: Vec<String>,
}

pub fn ensure_gate0_registries() -> Result<(), String> {
    ensure_asset_registry_default()?;
    ensure_units_registry_default()?;
    Ok(())
}

pub fn resolve_asset_handle(path: &str) -> Result<ResourceHandle, String> {
    let Some(registry) = ASSET_REGISTRY.get() else {
        return Ok(ResourceHandle::from_path(path));
    };
    let normalized = normalize_asset_path(path)?;
    let entry = registry
        .entries
        .get(&normalized)
        .ok_or_else(|| format!("RESOURCE_NOT_FOUND: {}", normalized))?;
    Ok(ResourceHandle::from_raw(entry.handle))
}

fn ensure_asset_registry_default() -> Result<(), String> {
    let manifest_path = Path::new("ddn.asset.json");
    let manifest_path = if manifest_path.exists() {
        manifest_path
    } else {
        let legacy_path = Path::new("ddn.resource.json");
        if legacy_path.exists() {
            eprintln!("경고: ddn.resource.json은 레거시 별칭입니다. ddn.asset.json으로 교체하세요.");
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

fn load_asset_registry(manifest_path: &Path, strict_hash: bool) -> Result<AssetRegistry, String> {
    if !manifest_path.exists() {
        return Err("RESOURCE_REGISTRY_MISSING: ddn.asset.json".to_string());
    }
    let raw = fs::read_to_string(manifest_path)
        .map_err(|e| format!("ASSET_MANIFEST_INVALID: {}", e))?;
    let manifest: AssetManifest =
        serde_json::from_str(&raw).map_err(|e| format!("ASSET_MANIFEST_INVALID: {}", e))?;

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

    let manifest_dir = manifest_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let mut entries = HashMap::new();
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
        let expected = ddonirang_core::asset_handle_from_bundle_path(
            &manifest.bundle_id,
            &normalized,
        )
        .raw();
        if handle != expected {
            return Err(format!(
                "ASSET_MANIFEST_INVALID: handle mismatch for {}",
                normalized
            ));
        }
        if strict_hash {
            let full_path = manifest_dir.join(&normalized);
            let data = fs::read(&full_path).map_err(|_| {
                format!("RESOURCE_HASH_MISMATCH: missing {}", normalized)
            })?;
            let actual_hash = hash(&data).to_hex().to_string();
            if !hash_eq(&entry.hash, &actual_hash) {
                return Err(format!(
                    "RESOURCE_HASH_MISMATCH: {}",
                    normalized
                ));
            }
            let actual_size = data.len() as u64;
            if entry.size != actual_size {
                return Err(format!(
                    "RESOURCE_HASH_MISMATCH: {}",
                    normalized
                ));
            }
        }
        entries.insert(normalized, AssetEntry { handle });
    }

    Ok(AssetRegistry { entries })
}

fn ensure_units_registry_default() -> Result<(), String> {
    let path = Path::new("ddn.units.json");
    if !path.exists() {
        return Err("UNIT_REGISTRY_MISSING: ddn.units.json".to_string());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("UNIT_REGISTRY_INVALID: {}", e))?;
    let registry: UnitsRegistry =
        serde_json::from_str(&raw).map_err(|e| format!("UNIT_REGISTRY_INVALID: {}", e))?;
    if let Some(version) = &registry.version {
        if version != "v0" {
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
    set_unit_registry_symbols(symbols)?;
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
