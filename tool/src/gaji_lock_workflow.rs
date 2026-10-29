use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::artifact_output::write_text_artifact_atomic;

const MAX_GAJI_SCAN_DEPTH: usize = 16;

struct GajiMeta {
    id: String,
    version: String,
}

struct GajiPackage {
    id: String,
    version: String,
    path: String,
    hash: String,
    files: Vec<GajiFile>,
}

#[derive(Clone, Debug)]
pub struct GajiFile {
    pub path: String,
    pub bytes: u64,
    pub hash: String,
}

#[derive(Clone, Debug, Default)]
pub struct LockWriteOptions {
    pub snapshot_id: Option<String>,
    pub index_root_hash: Option<String>,
    pub trust_root_hash: Option<String>,
    pub trust_root_source: Option<String>,
    pub audit_last_hash: Option<String>,
}

impl LockWriteOptions {
    pub fn is_empty(&self) -> bool {
        self.snapshot_id.is_none()
            && self.index_root_hash.is_none()
            && self.trust_root_hash.is_none()
            && self.trust_root_source.is_none()
            && self.audit_last_hash.is_none()
    }
}

pub fn run_gaji_lock(root: &Path, out: &Path) -> Result<(), String> {
    run_gaji_lock_with_options(root, out, &LockWriteOptions::default())
}

pub fn run_gaji_lock_with_options(
    root: &Path,
    out: &Path,
    options: &LockWriteOptions,
) -> Result<(), String> {
    let gaji_root = root.join("gaji");
    if !gaji_root.exists() {
        return Err("E_GAJI_SCAN gaji/ 폴더가 없습니다.".to_string());
    }

    let mut packages = collect_packages(&gaji_root)?;
    packages.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.path.cmp(&b.path)));

    let lock_hash = lock_hash(&packages);
    let json_text = build_lock_json(&lock_hash, &packages, options)?;

    write_text_artifact_atomic(out, &json_text).map_err(|e| format!("E_GAJI_WRITE {}", e))?;
    println!("gaji_lock_written={}", out.display());
    println!("gaji_lock_hash={}", lock_hash);
    Ok(())
}

fn collect_packages(gaji_root: &Path) -> Result<Vec<GajiPackage>, String> {
    let mut packages = Vec::new();
    collect_packages_recursive(gaji_root, gaji_root, 0, &mut packages)?;
    Ok(packages)
}

fn collect_packages_recursive(
    gaji_root: &Path,
    current: &Path,
    depth: usize,
    packages: &mut Vec<GajiPackage>,
) -> Result<(), String> {
    if depth > MAX_GAJI_SCAN_DEPTH {
        return Err(format!(
            "E_GAJI_SCAN_DEPTH path={} max_depth={}",
            current.display(),
            MAX_GAJI_SCAN_DEPTH
        ));
    }
    let gaji_toml = current.join("gaji.toml");
    if depth > 0 && gaji_toml.exists() {
        let meta = parse_gaji_toml(&gaji_toml)?;
        let rel_path = rel_path(gaji_root, current)?;
        let mut files = collect_gaji_files(current)?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let hash = gaji_package_hash(&files);
        packages.push(GajiPackage {
            id: meta.id,
            version: meta.version,
            path: rel_path,
            hash,
            files,
        });
        return Ok(());
    }

    let entries = fs::read_dir(current).map_err(|e| format!("E_GAJI_SCAN {}", e))?;
    let mut dirs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("E_GAJI_SCAN {}", e))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("E_GAJI_SCAN {}", e))?;
        if !file_type.is_dir() || should_skip_gaji_dir(&path) {
            continue;
        }
        dirs.push(path);
    }
    dirs.sort();
    for path in dirs {
        collect_packages_recursive(gaji_root, &path, depth + 1, packages)?;
    }
    Ok(())
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

pub fn collect_gaji_files(root: &Path) -> Result<Vec<GajiFile>, String> {
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
            if should_skip_gaji_dir(&path) {
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

pub fn should_skip_gaji_dir(path: &Path) -> bool {
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

pub fn gaji_package_hash(files: &[GajiFile]) -> String {
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

fn build_lock_json(
    lock_hash: &str,
    packages: &[GajiPackage],
    options: &LockWriteOptions,
) -> Result<String, String> {
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
    let mut root = serde_json::Map::new();
    root.insert(
        "schema_version".to_string(),
        Value::String("v1".to_string()),
    );
    root.insert(
        "lock_hash".to_string(),
        Value::String(lock_hash.to_string()),
    );
    root.insert("packages".to_string(), Value::Array(pkg_values));

    append_lock_meta_fields(&mut root, options)?;
    serde_json::to_string_pretty(&root).map_err(|e| format!("E_GAJI_JSON {}", e))
}

fn append_lock_meta_fields(
    root: &mut serde_json::Map<String, Value>,
    options: &LockWriteOptions,
) -> Result<(), String> {
    if options.snapshot_id.is_some() || options.index_root_hash.is_some() {
        let Some(snapshot_id) = options.snapshot_id.as_ref() else {
            return Err(
                "E_GAJI_LOCK_META snapshot_id/index_root_hash는 함께 지정해야 합니다.".to_string(),
            );
        };
        let Some(index_root_hash) = options.index_root_hash.as_ref() else {
            return Err(
                "E_GAJI_LOCK_META snapshot_id/index_root_hash는 함께 지정해야 합니다.".to_string(),
            );
        };
        root.insert(
            "registry_snapshot".to_string(),
            json!({
                "snapshot_id": snapshot_id,
                "index_root_hash": index_root_hash,
            }),
        );
    }

    if options.trust_root_hash.is_some() || options.trust_root_source.is_some() {
        let Some(hash) = options.trust_root_hash.as_ref() else {
            return Err(
                "E_GAJI_LOCK_META trust_root_hash/trust_root_source는 함께 지정해야 합니다."
                    .to_string(),
            );
        };
        let Some(source) = options.trust_root_source.as_ref() else {
            return Err(
                "E_GAJI_LOCK_META trust_root_hash/trust_root_source는 함께 지정해야 합니다."
                    .to_string(),
            );
        };
        if !matches!(source.as_str(), "registry" | "mirror" | "airgap") {
            return Err(format!(
                "E_GAJI_LOCK_META trust_root_source={} (need registry|mirror|airgap)",
                source
            ));
        }
        root.insert(
            "trust_root".to_string(),
            json!({
                "hash": hash,
                "source": source,
            }),
        );
    }
    if let Some(last_hash) = options.audit_last_hash.as_deref() {
        root.insert(
            "registry_audit".to_string(),
            json!({
                "last_hash": last_hash,
            }),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> std::path::PathBuf {
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("ddn_gaji_lock_{label}_{}_{}", process::id(), nonce))
    }

    #[test]
    fn lock_is_deterministic_and_preserves_registry_meta() {
        let root = test_dir("identity");
        let package = root.join("gaji").join("sample");
        fs::create_dir_all(package.join("ddn")).expect("create package");
        fs::write(
            package.join("gaji.toml"),
            "id = \"sample\"\nversion = \"1.0.0\"\n",
        )
        .expect("write metadata");
        fs::write(package.join("ddn").join("main.ddn"), "보임 값 = 1\n").expect("write source");
        let first = root.join("first.lock");
        let second = root.join("second.lock");
        let options = LockWriteOptions {
            snapshot_id: Some("snapshot-1".to_string()),
            index_root_hash: Some("sha256:index".to_string()),
            trust_root_hash: Some("sha256:trust".to_string()),
            trust_root_source: Some("registry".to_string()),
            audit_last_hash: Some("sha256:audit".to_string()),
        };

        run_gaji_lock_with_options(&root, &first, &options).expect("first lock");
        run_gaji_lock_with_options(&root, &second, &options).expect("second lock");
        assert_eq!(
            fs::read(&first).expect("first bytes"),
            fs::read(&second).expect("second bytes")
        );
        let lock: Value =
            serde_json::from_slice(&fs::read(&first).expect("lock bytes")).expect("lock json");
        assert_eq!(lock["registry_snapshot"]["snapshot_id"], "snapshot-1");
        assert_eq!(lock["trust_root"]["source"], "registry");
        assert_eq!(lock["registry_audit"]["last_hash"], "sha256:audit");
        fs::remove_dir_all(&root).expect("remove root");
    }
}
