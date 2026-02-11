use std::fs;
use std::path::Path;

use blake3;
use serde_json::{json, Value};

struct GajiMeta {
    id: String,
    version: String,
}

struct GajiFile {
    path: String,
    bytes: u64,
    hash: String,
}

struct GajiPackage {
    id: String,
    version: String,
    path: String,
    hash: String,
    files: Vec<GajiFile>,
}

pub fn run_lock(root: &Path, out: &Path) -> Result<(), String> {
    let gaji_root = root.join("gaji");
    if !gaji_root.exists() {
        return Err("E_GAJI_SCAN gaji/ 폴더가 없습니다.".to_string());
    }

    let mut packages = collect_packages(&gaji_root)?;
    packages.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.path.cmp(&b.path)));

    let lock_hash = lock_hash(&packages);
    let json_text = build_lock_json(&lock_hash, &packages)?;

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("E_GAJI_WRITE {}", e))?;
    }
    fs::write(out, json_text).map_err(|e| format!("E_GAJI_WRITE {}", e))?;
    println!("gaji_lock_written={}", out.display());
    println!("gaji_lock_hash={}", lock_hash);
    Ok(())
}

fn collect_packages(gaji_root: &Path) -> Result<Vec<GajiPackage>, String> {
    let mut packages = Vec::new();
    let entries = fs::read_dir(gaji_root).map_err(|e| format!("E_GAJI_SCAN {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("E_GAJI_SCAN {}", e))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let gaji_toml = path.join("gaji.toml");
        if !gaji_toml.exists() {
            continue;
        }
        let meta = parse_gaji_toml(&gaji_toml)?;
        let rel_path = rel_path(gaji_root, &path)?;
        let mut files = collect_files(&path)?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let hash = package_hash(&files);
        packages.push(GajiPackage {
            id: meta.id,
            version: meta.version,
            path: rel_path,
            hash,
            files,
        });
    }
    Ok(packages)
}

fn parse_gaji_toml(path: &Path) -> Result<GajiMeta, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("E_GAJI_READ {}", e))?;
    let mut id = None;
    let mut name = None;
    let mut version = None;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() || line.starts_with('[') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let mut value = value.trim().trim_end_matches(',').trim().to_string();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value = value[1..value.len() - 1].to_string();
        }
        match key {
            "id" => id = Some(value),
            "name" => name = Some(value),
            "version" => version = Some(value),
            _ => {}
        }
    }
    let dir_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let id = id.or(name).unwrap_or_else(|| format!("gaji/{}", dir_name));
    let Some(version) = version else {
        return Err(format!(
            "E_GAJI_TOML_VERSION version이 없습니다: {}",
            path.display()
        ));
    };
    Ok(GajiMeta { id, version })
}

fn collect_files(root: &Path) -> Result<Vec<GajiFile>, String> {
    let mut out = Vec::new();
    visit_dir(root, root, &mut out)?;
    Ok(out)
}

fn visit_dir(root: &Path, current: &Path, out: &mut Vec<GajiFile>) -> Result<(), String> {
    let entries = fs::read_dir(current).map_err(|e| format!("E_GAJI_SCAN {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("E_GAJI_SCAN {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            visit_dir(root, &path, out)?;
        } else {
            let rel = rel_path(root, &path)?;
            let bytes = fs::read(&path).map_err(|e| format!("E_GAJI_READ {}", e))?;
            let hash = format!("blake3:{}", blake3::hash(&bytes).to_hex());
            out.push(GajiFile {
                path: rel,
                bytes: bytes.len() as u64,
                hash,
            });
        }
    }
    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".git" | "target" | "build" | "out" | "dist" | "node_modules" | ".cargo"
    )
}

fn rel_path(root: &Path, path: &Path) -> Result<String, String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| format!("E_GAJI_PATH {}", path.display()))?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn package_hash(files: &[GajiFile]) -> String {
    let mut hasher = blake3::Hasher::new();
    for file in files {
        hasher.update(file.path.as_bytes());
        hasher.update(&[0]);
        hasher.update(file.hash.as_bytes());
        hasher.update(&[0]);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn lock_hash(packages: &[GajiPackage]) -> String {
    let mut hasher = blake3::Hasher::new();
    for pkg in packages {
        hasher.update(pkg.id.as_bytes());
        hasher.update(&[0]);
        hasher.update(pkg.version.as_bytes());
        hasher.update(&[0]);
        hasher.update(pkg.hash.as_bytes());
        hasher.update(&[0]);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn build_lock_json(lock_hash: &str, packages: &[GajiPackage]) -> Result<String, String> {
    let mut pkg_values: Vec<Value> = Vec::new();
    for pkg in packages {
        let files: Vec<Value> = pkg
            .files
            .iter()
            .map(|f| {
                json!({
                    "path": f.path,
                    "bytes": f.bytes,
                    "hash": f.hash,
                })
            })
            .collect();
        pkg_values.push(json!({
            "id": pkg.id,
            "version": pkg.version,
            "path": pkg.path,
            "hash": pkg.hash,
            "files": files,
        }));
    }
    let root = json!({
        "schema_version": "v1",
        "lock_hash": lock_hash,
        "packages": pkg_values,
    });
    serde_json::to_string_pretty(&root).map_err(|e| format!("E_GAJI_JSON {}", e))
}
