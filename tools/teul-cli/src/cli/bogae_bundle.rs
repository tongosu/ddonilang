use std::fs;
use std::path::{Path, PathBuf};

use blake3;

pub struct BogaeBundleOptions<'a> {
    pub out_dir: &'a Path,
    pub mapping: Option<&'a Path>,
    pub scene: Option<&'a Path>,
    pub assets_dir: Option<&'a Path>,
}

pub fn run_bundle(options: BogaeBundleOptions<'_>) -> Result<(), String> {
    if options.mapping.is_none() && options.scene.is_none() {
        return Err("mapping 또는 scene 중 하나가 필요합니다.".to_string());
    }
    fs::create_dir_all(options.out_dir).map_err(|e| e.to_string())?;
    let assets_out = options.out_dir.join("assets");
    fs::create_dir_all(&assets_out).map_err(|e| e.to_string())?;

    let mapping_name = if let Some(mapping) = options.mapping {
        let target = options.out_dir.join("mapping.ddn");
        fs::copy(mapping, &target).map_err(|e| e.to_string())?;
        Some("mapping.ddn".to_string())
    } else {
        None
    };
    let scene_name = if let Some(scene) = options.scene {
        let target = options.out_dir.join("scene.json");
        fs::copy(scene, &target).map_err(|e| e.to_string())?;
        Some("scene.json".to_string())
    } else {
        None
    };

    let mut assets = Vec::new();
    if let Some(root) = options.assets_dir {
        if !root.exists() {
            return Err(format!("assets dir not found: {}", root.display()));
        }
        let mut files = Vec::new();
        collect_files(root, &mut files)?;
        files.sort_by(|a, b| normalize_rel_path(root, a).cmp(&normalize_rel_path(root, b)));
        for path in files {
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            let hash = blake3::hash(&bytes).to_hex().to_string();
            let id = format!("blake3:{}", hash);
            let ext = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_lowercase();
            let file_name = if ext.is_empty() {
                hash.clone()
            } else {
                format!("{}.{}", hash, ext)
            };
            let rel_source = normalize_rel_path(root, &path);
            let out_path = format!("assets/{}", file_name);
            let target = assets_out.join(&file_name);
            if !target.exists() {
                fs::write(&target, &bytes).map_err(|e| e.to_string())?;
            }
            assets.push(BundleAsset {
                id,
                path: out_path,
                source: rel_source,
            });
        }
    }

    let manifest = build_manifest_text(&BundleManifest {
        mapping: mapping_name,
        scene: scene_name,
        assets,
    });
    let manifest_path = options.out_dir.join("manifest.toml");
    fs::write(&manifest_path, &manifest).map_err(|e| e.to_string())?;
    let hash = blake3::hash(manifest.as_bytes());
    println!("bogae_bundle_hash=blake3:{}", hash.to_hex());
    Ok(())
}

struct BundleManifest {
    mapping: Option<String>,
    scene: Option<String>,
    assets: Vec<BundleAsset>,
}

struct BundleAsset {
    id: String,
    path: String,
    source: String,
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(root).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn normalize_rel_path(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

fn build_manifest_text(manifest: &BundleManifest) -> String {
    let mut out = String::new();
    out.push_str("version = \"bogae_bundle_v1\"\n");
    if let Some(mapping) = &manifest.mapping {
        out.push_str(&format!("mapping = \"{}\"\n", escape_toml(mapping)));
    }
    if let Some(scene) = &manifest.scene {
        out.push_str(&format!("scene = \"{}\"\n", escape_toml(scene)));
    }
    if !manifest.assets.is_empty() {
        out.push('\n');
    }
    for asset in &manifest.assets {
        out.push_str("[[asset]]\n");
        out.push_str(&format!("id = \"{}\"\n", escape_toml(&asset.id)));
        out.push_str(&format!("path = \"{}\"\n", escape_toml(&asset.path)));
        out.push_str(&format!("source = \"{}\"\n", escape_toml(&asset.source)));
        out.push('\n');
    }
    out
}

fn escape_toml(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}
