use std::fs;
use std::path::{Path, PathBuf};

use blake3;

use crate::artifact_output::write_text_artifact_atomic;
use crate::state_trace_wire::escape_json_string_contents as escape_json;

pub fn run_manifest(root: &Path, out: Option<&Path>) -> Result<(), String> {
    if !root.exists() {
        return Err(format!("asset root not found: {}", root.display()));
    }
    if !root.is_dir() {
        return Err(format!("asset root is not a directory: {}", root.display()));
    }

    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    files.sort_by(|a, b| {
        let a_rel = normalize_rel_path(root, a);
        let b_rel = normalize_rel_path(root, b);
        a_rel.cmp(&b_rel)
    });

    let mut entries = Vec::new();
    for path in files {
        let bytes = fs::read(&path).map_err(|e| format!("{}: {}", path.display(), e))?;
        let id = format!("blake3:{}", blake3::hash(&bytes).to_hex());
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let rel_path = normalize_rel_path(root, &path);
        entries.push(AssetEntry {
            id,
            name,
            path: rel_path,
        });
    }

    let manifest = build_manifest_text(&entries);
    let manifest_hash = format!("blake3:{}", blake3::hash(manifest.as_bytes()).to_hex());
    if let Some(path) = out {
        require_file_target(path)?;
        write_text_artifact_atomic(path, &manifest)?;
    }
    println!("asset_manifest_hash={}", manifest_hash);
    Ok(())
}
fn require_file_target(path: &Path) -> Result<(), String> {
    if path.exists() && !path.is_file() {
        return Err(format!(
            "E_ASSET_MANIFEST_OUTPUT_TARGET_NOT_FILE {}",
            path.display()
        ));
    }
    Ok(())
}

struct AssetEntry {
    id: String,
    name: String,
    path: String,
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(root).map_err(|e| format!("{}: {}", root.display(), e))?;
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
    let text = rel.to_string_lossy().replace('\\', "/");
    text.trim_start_matches("./").to_string()
}

fn build_manifest_text(entries: &[AssetEntry]) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"kind\": \"bogae_asset_manifest_v1\",\n");
    out.push_str("  \"assets\": [\n");
    for (idx, entry) in entries.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"id\": \"{}\",\n", escape_json(&entry.id)));
        out.push_str(&format!(
            "      \"path\": \"{}\",\n",
            escape_json(&entry.path)
        ));
        out.push_str(&format!(
            "      \"name\": \"{}\"\n",
            escape_json(&entry.name)
        ));
        out.push_str("    }");
        if idx + 1 != entries.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}
