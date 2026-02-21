use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use ddonirang_lang::{parse_with_mode, ParseMode, SeedKind, TypeRef};

use crate::preprocess::preprocess_source_for_parse;

#[derive(Debug, Serialize)]
pub struct DdnSchema {
    pub version: String,
    pub source_file: String,
    pub source_hash: String,
    pub units: Vec<String>,
    pub assets_manifest: AssetManifestSummary,
    pub pins: Vec<String>,
    pub types: Vec<String>,
    pub seeds: Vec<SeedSchema>,
}

#[derive(Debug, Serialize)]
pub struct AssetManifestSummary {
    pub version: String,
    pub bundle_id: String,
    pub hash_algo: String,
    pub entry_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SeedSchema {
    pub name: String,
    pub kind: String,
    pub params: Vec<SeedParamSchema>,
}

#[derive(Debug, Serialize)]
pub struct SeedParamSchema {
    pub name: String,
    pub type_ref: String,
    pub optional: bool,
    pub has_default: bool,
}

#[derive(Debug, serde::Deserialize)]
struct UnitsRegistry {
    version: Option<String>,
    units: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct AssetManifest {
    version: String,
    bundle_id: String,
    hash_algo: String,
    entries: Vec<serde_json::Value>,
}

pub fn build_schema(source: &str, file_path: &str) -> Result<DdnSchema, String> {
    let cleaned = preprocess_source_for_parse(source)?;
    let program = parse_with_mode(&cleaned, file_path, ParseMode::Strict)
        .map_err(|e| format!("스키마 파싱 실패: {}", e))?;

    let mut pins = BTreeSet::new();
    let mut types = BTreeSet::new();
    let mut seeds = Vec::new();

    for item in &program.items {
        let ddonirang_lang::TopLevelItem::SeedDef(seed) = item;
        let kind = seed_kind_name(&seed.seed_kind);
        let mut params = Vec::new();
        for param in &seed.params {
            pins.insert(param.pin_name.clone());
            collect_type_names(&param.type_ref, &mut types);
            params.push(SeedParamSchema {
                name: param.pin_name.clone(),
                type_ref: type_ref_name(&param.type_ref),
                optional: param.optional,
                has_default: param.default_value.is_some(),
            });
        }
        seeds.push(SeedSchema {
            name: seed.canonical_name.clone(),
            kind,
            params,
        });
    }

    seeds.sort_by(|a, b| a.name.cmp(&b.name));
    let units = load_units_registry()?;
    let assets_manifest = load_asset_manifest_summary()?;

    let source_hash = blake3::hash(cleaned.as_bytes()).to_hex().to_string();
    Ok(DdnSchema {
        version: "v18.0.12".to_string(),
        source_file: file_path.to_string(),
        source_hash,
        units,
        assets_manifest,
        pins: pins.into_iter().collect(),
        types: types.into_iter().collect(),
        seeds,
    })
}

pub fn write_schema(path: &str, schema: &DdnSchema) -> Result<String, String> {
    let json =
        serde_json::to_string_pretty(schema).map_err(|e| format!("SCHEMA_JSON_INVALID: {}", e))?;
    fs::write(path, json.as_bytes()).map_err(|e| format!("SCHEMA_WRITE_FAILED: {}", e))?;
    Ok(blake3::hash(json.as_bytes()).to_hex().to_string())
}

fn seed_kind_name(kind: &SeedKind) -> String {
    match kind {
        SeedKind::Imeumssi => "Imeumssi",
        SeedKind::Umjikssi => "Umjikssi",
        SeedKind::ValueFunc => "ValueFunc",
        SeedKind::Gallaessi => "Gallaessi",
        SeedKind::Relationssi => "Relationssi",
        SeedKind::Sam => "Sam",
        SeedKind::Heureumssi => "Heureumssi",
        SeedKind::Ieumssi => "Ieumssi",
        SeedKind::Semssi => "Semssi",
        SeedKind::Named(name) => name,
    }
    .to_string()
}

fn type_ref_name(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Applied { name, args } => {
            let mut out = String::new();
            out.push('(');
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    out.push(' ');
                }
                out.push_str(&type_ref_name(arg));
            }
            out.push_str(") ");
            out.push_str(name);
            out
        }
        TypeRef::Infer => "infer".to_string(),
    }
}

fn collect_type_names(type_ref: &TypeRef, types: &mut BTreeSet<String>) {
    match type_ref {
        TypeRef::Named(name) => {
            if name != "_" {
                types.insert(name.clone());
            }
        }
        TypeRef::Applied { name, args } => {
            types.insert(name.clone());
            for arg in args {
                collect_type_names(arg, types);
            }
        }
        TypeRef::Infer => {}
    }
}

fn load_units_registry() -> Result<Vec<String>, String> {
    let path = Path::new("ddn.units.json");
    if !path.exists() {
        return Err("SCHEMA_UNIT_REGISTRY_MISSING: ddn.units.json".to_string());
    }
    let raw =
        fs::read_to_string(path).map_err(|e| format!("SCHEMA_UNIT_REGISTRY_INVALID: {}", e))?;
    let registry: UnitsRegistry =
        serde_json::from_str(&raw).map_err(|e| format!("SCHEMA_UNIT_REGISTRY_INVALID: {}", e))?;
    if let Some(version) = &registry.version {
        if version != "v0" {
            return Err(format!("SCHEMA_UNIT_REGISTRY_INVALID: version {}", version));
        }
    }
    let mut units = registry.units;
    units.sort();
    units.dedup();
    Ok(units)
}

fn load_asset_manifest_summary() -> Result<AssetManifestSummary, String> {
    let path = Path::new("ddn.asset.json");
    if !path.exists() {
        return Err("SCHEMA_ASSET_MANIFEST_MISSING: ddn.asset.json".to_string());
    }
    let raw =
        fs::read_to_string(path).map_err(|e| format!("SCHEMA_ASSET_MANIFEST_INVALID: {}", e))?;
    let manifest: AssetManifest =
        serde_json::from_str(&raw).map_err(|e| format!("SCHEMA_ASSET_MANIFEST_INVALID: {}", e))?;
    Ok(AssetManifestSummary {
        version: manifest.version,
        bundle_id: manifest.bundle_id,
        hash_algo: manifest.hash_algo,
        entry_count: manifest.entries.len(),
    })
}
