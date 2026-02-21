use std::fs;
use std::path::{Path, PathBuf};

use blake3;
use serde_json::{json, Value};

use super::{diag, gaji_registry};

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

struct LockPackage {
    id: String,
    version: String,
    path: String,
    hash: String,
    yanked: bool,
    archive_sha256: Option<String>,
    download_url: Option<String>,
    dependencies: Option<Value>,
    contract: Option<String>,
    min_runtime: Option<String>,
    detmath_seal_hash: Option<String>,
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
    fn is_empty(&self) -> bool {
        self.snapshot_id.is_none()
            && self.index_root_hash.is_none()
            && self.trust_root_hash.is_none()
            && self.trust_root_source.is_none()
            && self.audit_last_hash.is_none()
    }
}

#[derive(Clone, Debug, Default)]
pub struct FrozenLockOptions {
    pub frozen_lockfile: bool,
    pub expect_snapshot_id: Option<String>,
    pub expect_index_root_hash: Option<String>,
    pub expect_trust_root_hash: Option<String>,
    pub require_trust_root: bool,
    pub deny_yanked_locked: bool,
    pub registry_index: Option<PathBuf>,
    pub verify_registry: bool,
    pub registry_verify_out: Option<PathBuf>,
    pub registry_audit_log: Option<PathBuf>,
    pub verify_registry_audit: bool,
    pub registry_audit_verify_out: Option<PathBuf>,
    pub expect_audit_last_hash: Option<String>,
    pub strict_registry: bool,
}

impl FrozenLockOptions {
    fn is_enabled(&self) -> bool {
        self.frozen_lockfile
            || self.expect_snapshot_id.is_some()
            || self.expect_index_root_hash.is_some()
            || self.expect_trust_root_hash.is_some()
            || self.require_trust_root
            || self.verify_registry_audit
            || self.expect_audit_last_hash.is_some()
            || self.strict_registry
    }
}

pub fn run_lock(root: &Path, out: &Path) -> Result<(), String> {
    run_lock_with_options(root, out, &LockWriteOptions::default())
}

pub fn apply_registry_meta_from_index(
    options: &mut LockWriteOptions,
    index_path: &Path,
) -> Result<(), String> {
    let text = fs::read_to_string(index_path).map_err(|e| {
        diag::build_diag(
            "E_REG_INDEX_READ",
            &format!("path={} {}", index_path.display(), e),
            None,
            Some("registry index 파일 경로/권한을 확인하세요.".to_string()),
        )
    })?;
    let root: Value = serde_json::from_str(&text).map_err(|e| {
        diag::build_diag(
            "E_REG_INDEX_PARSE",
            &format!("path={} {}", index_path.display(), e),
            Some("registry index JSON 파싱 실패".to_string()),
            Some("registry index JSON을 정정하세요.".to_string()),
        )
    })?;

    let snapshot = root.get("registry_snapshot");
    let snapshot_id = root
        .get("snapshot_id")
        .and_then(|v| v.as_str())
        .or_else(|| {
            snapshot
                .and_then(|v| v.get("snapshot_id"))
                .and_then(|v| v.as_str())
        })
        .map(|v| v.to_string());
    let index_root_hash = root
        .get("index_root_hash")
        .and_then(|v| v.as_str())
        .or_else(|| {
            snapshot
                .and_then(|v| v.get("index_root_hash"))
                .and_then(|v| v.as_str())
        })
        .map(|v| v.to_string());

    if options.snapshot_id.is_none() {
        options.snapshot_id = snapshot_id;
    }
    if options.index_root_hash.is_none() {
        options.index_root_hash = index_root_hash;
    }
    if options.snapshot_id.is_none() || options.index_root_hash.is_none() {
        return Err(diag::build_diag(
            "E_REG_SNAPSHOT_MISSING",
            "registry index requires snapshot_id/index_root_hash",
            None,
            Some("index에 snapshot_id/index_root_hash를 모두 채우세요.".to_string()),
        ));
    }

    let trust_root = root.get("trust_root");
    if options.trust_root_hash.is_none() {
        options.trust_root_hash = trust_root
            .and_then(|v| v.get("hash"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
    }
    if options.trust_root_source.is_none() {
        options.trust_root_source = trust_root
            .and_then(|v| v.get("source"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
    }
    if options.trust_root_hash.is_some() && options.trust_root_source.is_none() {
        options.trust_root_source = Some("registry".to_string());
    }
    Ok(())
}

pub fn run_lock_with_options(
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

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("E_GAJI_WRITE {}", e))?;
    }
    fs::write(out, json_text).map_err(|e| format!("E_GAJI_WRITE {}", e))?;
    println!("gaji_lock_written={}", out.display());
    println!("gaji_lock_hash={}", lock_hash);
    Ok(())
}

#[allow(dead_code)]
pub fn run_install(root: &Path, lock_path: &Path, out: &Path) -> Result<(), String> {
    run_install_with_options(root, lock_path, out, &FrozenLockOptions::default())
}

pub fn run_install_with_options(
    root: &Path,
    lock_path: &Path,
    out: &Path,
    frozen: &FrozenLockOptions,
) -> Result<(), String> {
    let effective_frozen = normalize_strict_registry_options(frozen);
    let mut lock_created = false;
    if !lock_path.exists() {
        if effective_frozen.strict_registry && effective_frozen.registry_index.is_none() {
            return Err(diag::build_diag(
                "E_REG_VERIFY_INDEX_REQUIRED",
                "--strict-registry requires --registry-index",
                None,
                Some("add --registry-index <path>".to_string()),
            ));
        }
        if let Some(index_path) = effective_frozen.registry_index.as_deref() {
            let mut lock_options = LockWriteOptions::default();
            apply_registry_meta_from_index(&mut lock_options, index_path)?;
            run_lock_with_options(root, lock_path, &lock_options)?;
        } else {
            run_lock(root, lock_path)?;
        }
        lock_created = true;
    }
    run_vendor_with_options(root, lock_path, out, &effective_frozen)?;
    println!(
        "gaji_install_lock_created={}",
        if lock_created { 1 } else { 0 }
    );
    Ok(())
}

#[allow(dead_code)]
pub fn run_update(root: &Path, lock_path: &Path, out: &Path) -> Result<(), String> {
    run_update_with_options(
        root,
        lock_path,
        out,
        &LockWriteOptions::default(),
        &FrozenLockOptions::default(),
    )
}

pub fn run_update_with_options(
    root: &Path,
    lock_path: &Path,
    out: &Path,
    lock_options: &LockWriteOptions,
    frozen: &FrozenLockOptions,
) -> Result<(), String> {
    let effective_frozen = normalize_strict_registry_options(frozen);
    let before = read_lock_hash(lock_path)?;
    let mut effective = if lock_options.is_empty() {
        read_lock_write_options(lock_path)?
    } else {
        lock_options.clone()
    };
    if let Some(index_path) = effective_frozen.registry_index.as_deref() {
        apply_registry_meta_from_index(&mut effective, index_path)?;
    } else if effective_frozen.strict_registry {
        return Err(diag::build_diag(
            "E_REG_VERIFY_INDEX_REQUIRED",
            "--strict-registry requires --registry-index",
            None,
            Some("add --registry-index <path>".to_string()),
        ));
    }
    run_lock_with_options(root, lock_path, &effective)?;
    let after = read_lock_hash(lock_path)?;
    run_vendor_with_options(root, lock_path, out, &effective_frozen)?;
    let changed = match (before.as_deref(), after.as_deref()) {
        (Some(left), Some(right)) => left != right,
        (None, Some(_)) => true,
        (_, None) => true,
    };
    println!("gaji_update_changed={}", if changed { 1 } else { 0 });
    Ok(())
}

#[allow(dead_code)]
pub fn run_vendor(root: &Path, lock_path: &Path, out: &Path) -> Result<(), String> {
    run_vendor_with_options(root, lock_path, out, &FrozenLockOptions::default())
}

pub fn run_vendor_with_options(
    root: &Path,
    lock_path: &Path,
    out: &Path,
    frozen: &FrozenLockOptions,
) -> Result<(), String> {
    let mut effective_frozen = normalize_strict_registry_options(frozen);
    apply_default_verify_report_paths(&mut effective_frozen, out);
    let lock = read_lock_file(lock_path)?;
    validate_frozen_lock(&lock, &effective_frozen)?;
    maybe_verify_registry(lock_path, &lock, &effective_frozen)?;
    maybe_verify_registry_audit(&lock, &effective_frozen)?;
    let deny_yanked_locked =
        effective_frozen.deny_yanked_locked || effective_frozen.strict_registry;
    let packages = read_lock_packages(&lock)?;
    if out.exists() {
        fs::remove_dir_all(out).map_err(|e| format!("E_GAJI_VENDOR_CLEAN {}", e))?;
    }
    fs::create_dir_all(out).map_err(|e| format!("E_GAJI_VENDOR_WRITE {}", e))?;

    let mut copied = 0usize;
    for pkg in &packages {
        if pkg.yanked {
            if deny_yanked_locked {
                return Err(diag::build_diag(
                    "E_REG_YANKED_LOCKED",
                    &format!("id={} version={}", pkg.id, pkg.version),
                    None,
                    Some(
                        "잠금 해소를 갱신하거나 --deny-yanked-locked 설정을 재검토하세요."
                            .to_string(),
                    ),
                ));
            }
            eprintln!(
                "W_REG_YANKED_LOCKED id={} version={} (lock 재현 허용)",
                pkg.id, pkg.version
            );
        }
        let src = root.join("gaji").join(&pkg.path);
        if !src.exists() {
            return Err(format!(
                "E_GAJI_VENDOR_SRC 패키지 경로가 없습니다: id={} src={}",
                pkg.id,
                src.display()
            ));
        }
        let mut files = collect_files(&src)?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let actual_hash = package_hash(&files);
        if actual_hash != pkg.hash {
            return Err(format!(
                "E_GAJI_VENDOR_HASH_MISMATCH id={} expected={} actual={}",
                pkg.id, pkg.hash, actual_hash
            ));
        }
        let dst = out.join(&pkg.path);
        copy_dir_recursive(&src, &dst)?;
        copied += 1;
    }

    let index = build_vendor_index(&packages)?;
    fs::write(out.join("ddn.vendor.index.json"), index)
        .map_err(|e| format!("E_GAJI_VENDOR_WRITE {}", e))?;

    println!("gaji_vendor_out={}", out.display());
    println!("gaji_vendor_packages={}", copied);
    Ok(())
}

fn apply_default_verify_report_paths(options: &mut FrozenLockOptions, out: &Path) {
    let Some(base_dir) = out.parent() else {
        return;
    };
    if options.verify_registry && options.registry_verify_out.is_none() {
        options.registry_verify_out = Some(base_dir.join("registry.verify.json"));
    }
    let audit_verify_enabled = options.verify_registry_audit
        || (options.expect_audit_last_hash.is_some() && options.registry_audit_log.is_some());
    if audit_verify_enabled && options.registry_audit_verify_out.is_none() {
        options.registry_audit_verify_out = Some(base_dir.join("registry.audit.verify.json"));
    }
}

fn normalize_strict_registry_options(frozen: &FrozenLockOptions) -> FrozenLockOptions {
    let mut out = frozen.clone();
    if !out.strict_registry {
        return out;
    }
    out.frozen_lockfile = true;
    out.verify_registry = true;
    out.require_trust_root = true;
    out.deny_yanked_locked = true;
    if out.registry_audit_log.is_some() {
        out.verify_registry_audit = true;
    }
    out
}

fn maybe_verify_registry(
    lock_path: &Path,
    lock: &Value,
    frozen: &FrozenLockOptions,
) -> Result<(), String> {
    let strict_requires_verify = frozen.strict_registry && frozen.frozen_lockfile;
    let require_verify = frozen.verify_registry || strict_requires_verify;

    if !require_verify && frozen.registry_index.is_none() {
        return Ok(());
    }

    let Some(index_path) = frozen.registry_index.as_deref() else {
        return Err(diag::build_diag(
            "E_REG_VERIFY_INDEX_REQUIRED",
            "--verify-registry requires --registry-index",
            None,
            Some("add --registry-index <path>".to_string()),
        ));
    };

    let guard = build_registry_read_guard_from_lock(lock, frozen)?;
    let report = gaji_registry::run_verify_with_guard(
        index_path,
        lock_path,
        &guard,
        frozen.deny_yanked_locked,
    )?;
    if let Some(path) = frozen.registry_verify_out.as_deref() {
        gaji_registry::write_verify_report(path, &report)?;
    }
    Ok(())
}

fn maybe_verify_registry_audit(lock: &Value, frozen: &FrozenLockOptions) -> Result<(), String> {
    let expected_from_lock = lock
        .get("registry_audit")
        .and_then(|v| v.get("last_hash"))
        .and_then(|v| v.as_str());
    let expected_last_hash = frozen
        .expect_audit_last_hash
        .as_deref()
        .or(expected_from_lock);
    let strict_requires_last_hash_pin =
        frozen.strict_registry && frozen.registry_audit_log.is_some();
    if strict_requires_last_hash_pin && expected_last_hash.is_none() {
        let hint_last_hash = frozen
            .registry_audit_log
            .as_deref()
            .and_then(|path| gaji_registry::run_audit_verify(path).ok())
            .and_then(|report| report.last_hash().map(|v| v.to_string()))
            .map(|last| format!("hint_last_hash:{}", last));
        return Err(diag::build_diag(
            "E_REG_AUDIT_LAST_HASH_REQUIRED",
            "--strict-registry with --registry-audit-log requires registry_audit.last_hash in ddn.lock or --expect-audit-last-hash",
            hint_last_hash,
            Some("gaji lock --audit-last-hash <hash> OR gaji {install|update|vendor} --expect-audit-last-hash <hash>".to_string()),
        ));
    }

    let strict_requires_verify =
        frozen.strict_registry && frozen.frozen_lockfile && frozen.registry_audit_log.is_some();
    let expected_requires_verify =
        frozen.registry_audit_log.is_some() && expected_last_hash.is_some();
    let require_verify =
        frozen.verify_registry_audit || expected_requires_verify || strict_requires_verify;
    if !require_verify {
        return Ok(());
    }
    let Some(audit_log_path) = frozen.registry_audit_log.as_deref() else {
        return Err(diag::build_diag(
            "E_REG_AUDIT_VERIFY_LOG_REQUIRED",
            "--verify-registry-audit requires --registry-audit-log",
            None,
            Some("add --registry-audit-log <path>".to_string()),
        ));
    };
    let report = gaji_registry::run_audit_verify(audit_log_path)?;
    if let Some(expected) = expected_last_hash {
        let actual = report.last_hash().unwrap_or("<none>");
        if actual != expected {
            return Err(diag::build_diag(
                "E_REG_AUDIT_LAST_HASH_MISMATCH",
                &format!("expected={} actual={}", expected, actual),
                Some("audit_last_hash pin mismatch".to_string()),
                Some("update ddn.lock registry_audit.last_hash or pass --expect-audit-last-hash with current last hash".to_string()),
            ));
        }
    }
    if let Some(path) = frozen.registry_audit_verify_out.as_deref() {
        gaji_registry::write_audit_verify_report(path, &report)?;
    }
    Ok(())
}

fn build_registry_read_guard_from_lock(
    lock: &Value,
    frozen: &FrozenLockOptions,
) -> Result<gaji_registry::ReadGuardOptions, String> {
    let snapshot = lock.get("registry_snapshot");
    let trust_root = lock.get("trust_root");

    let expect_snapshot_id = frozen.expect_snapshot_id.clone().or_else(|| {
        snapshot
            .and_then(|v| v.get("snapshot_id"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
    });
    let expect_index_root_hash = frozen.expect_index_root_hash.clone().or_else(|| {
        snapshot
            .and_then(|v| v.get("index_root_hash"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
    });
    let expect_trust_root_hash = frozen.expect_trust_root_hash.clone().or_else(|| {
        trust_root
            .and_then(|v| v.get("hash"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
    });

    if frozen.frozen_lockfile && (expect_snapshot_id.is_none() || expect_index_root_hash.is_none())
    {
        return Err(diag::build_diag(
            "E_REG_SNAPSHOT_MISSING",
            "frozen-lockfile requires registry_snapshot(snapshot_id/index_root_hash)",
            None,
            Some(
                "ddn.lock에 registry_snapshot.snapshot_id/index_root_hash를 채우세요.".to_string(),
            ),
        ));
    }

    Ok(gaji_registry::ReadGuardOptions {
        frozen_lockfile: frozen.frozen_lockfile,
        expect_snapshot_id,
        expect_index_root_hash,
        expect_trust_root_hash,
        require_trust_root: frozen.require_trust_root,
    })
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

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("E_GAJI_VENDOR_WRITE {}", e))?;
    let entries = fs::read_dir(src).map_err(|e| format!("E_GAJI_VENDOR_READ {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("E_GAJI_VENDOR_READ {}", e))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            if should_skip_dir(&from) {
                continue;
            }
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("E_GAJI_VENDOR_WRITE {}", e))?;
            }
            fs::copy(&from, &to).map_err(|e| format!("E_GAJI_VENDOR_WRITE {}", e))?;
        }
    }
    Ok(())
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

fn read_lock_file(lock_path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(lock_path).map_err(|e| format!("E_GAJI_LOCK_READ {}", e))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|e| format!("E_GAJI_LOCK_PARSE {}", e))?;
    let schema = value
        .get("schema_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if schema != "v1" {
        return Err(format!(
            "E_GAJI_LOCK_SCHEMA schema_version={} (need v1)",
            schema
        ));
    }
    Ok(value)
}

fn read_lock_packages(value: &Value) -> Result<Vec<LockPackage>, String> {
    let packages = value
        .get("packages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "E_GAJI_LOCK_PACKAGES packages 배열이 없습니다.".to_string())?;
    let mut out = Vec::with_capacity(packages.len());
    for pkg in packages {
        let id = pkg
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "E_GAJI_LOCK_FIELD id 누락".to_string())?
            .to_string();
        let version = pkg
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "E_GAJI_LOCK_FIELD version 누락".to_string())?
            .to_string();
        let path = pkg
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "E_GAJI_LOCK_FIELD path 누락".to_string())?
            .to_string();
        let hash = pkg
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "E_GAJI_LOCK_FIELD hash 누락".to_string())?
            .to_string();
        let yanked = pkg.get("yanked").and_then(|v| v.as_bool()).unwrap_or(false);
        let archive_sha256 = pkg
            .get("archive_sha256")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let download_url = pkg
            .get("download_url")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let dependencies = pkg.get("dependencies").cloned();
        let contract = pkg
            .get("contract")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let min_runtime = pkg
            .get("min_runtime")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let detmath_seal_hash = pkg
            .get("detmath_seal_hash")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        out.push(LockPackage {
            id,
            version,
            path,
            hash,
            yanked,
            archive_sha256,
            download_url,
            dependencies,
            contract,
            min_runtime,
            detmath_seal_hash,
        });
    }
    Ok(out)
}

fn read_lock_hash(lock_path: &Path) -> Result<Option<String>, String> {
    if !lock_path.exists() {
        return Ok(None);
    }
    let value = read_lock_file(lock_path)?;
    Ok(value
        .get("lock_hash")
        .and_then(|v| v.as_str())
        .map(|text| text.to_string()))
}

fn read_lock_write_options(lock_path: &Path) -> Result<LockWriteOptions, String> {
    if !lock_path.exists() {
        return Ok(LockWriteOptions::default());
    }
    let value = read_lock_file(lock_path)?;

    let mut out = LockWriteOptions::default();
    if let Some(snapshot) = value.get("registry_snapshot") {
        out.snapshot_id = snapshot
            .get("snapshot_id")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        out.index_root_hash = snapshot
            .get("index_root_hash")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
    }
    if let Some(trust_root) = value.get("trust_root") {
        out.trust_root_hash = trust_root
            .get("hash")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        out.trust_root_source = trust_root
            .get("source")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
    }
    if let Some(registry_audit) = value.get("registry_audit") {
        out.audit_last_hash = registry_audit
            .get("last_hash")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
    }
    Ok(out)
}

fn validate_frozen_lock(lock: &Value, options: &FrozenLockOptions) -> Result<(), String> {
    if !options.is_enabled() {
        return Ok(());
    }

    let snapshot = lock.get("registry_snapshot");
    let snapshot_id = snapshot
        .and_then(|v| v.get("snapshot_id"))
        .and_then(|v| v.as_str());
    let index_root_hash = snapshot
        .and_then(|v| v.get("index_root_hash"))
        .and_then(|v| v.as_str());

    if options.frozen_lockfile && (snapshot_id.is_none() || index_root_hash.is_none()) {
        return Err(diag::build_diag(
            "E_REG_SNAPSHOT_MISSING",
            "frozen-lockfile requires registry_snapshot(snapshot_id/index_root_hash)",
            None,
            Some(
                "ddn.lock에 registry_snapshot.snapshot_id/index_root_hash를 채우세요.".to_string(),
            ),
        ));
    }

    if let Some(expected) = options.expect_snapshot_id.as_deref() {
        let Some(actual) = snapshot_id else {
            return Err(diag::build_diag(
                "E_REG_SNAPSHOT_MISSING",
                "registry_snapshot.snapshot_id is missing",
                None,
                Some("ddn.lock에 registry_snapshot.snapshot_id를 채우세요.".to_string()),
            ));
        };
        if actual != expected {
            return Err(diag::build_diag(
                "E_REG_SNAPSHOT_MISMATCH",
                &format!("expected={} actual={}", expected, actual),
                None,
                Some("요구 snapshot_id와 ddn.lock snapshot_id를 일치시키세요.".to_string()),
            ));
        }
    }

    if let Some(expected) = options.expect_index_root_hash.as_deref() {
        let Some(actual) = index_root_hash else {
            return Err(diag::build_diag(
                "E_REG_INDEX_ROOT_HASH_MISMATCH",
                "expected=<given> actual=<missing>",
                None,
                Some("ddn.lock에 registry_snapshot.index_root_hash를 채우세요.".to_string()),
            ));
        };
        if actual != expected {
            return Err(diag::build_diag(
                "E_REG_INDEX_ROOT_HASH_MISMATCH",
                &format!("expected={} actual={}", expected, actual),
                None,
                Some("요구 index_root_hash와 ddn.lock 값을 일치시키세요.".to_string()),
            ));
        }
    }

    let trust_root = lock.get("trust_root");
    let trust_root_hash = trust_root
        .and_then(|v| v.get("hash"))
        .and_then(|v| v.as_str());
    let require_trust_root = options.require_trust_root || options.strict_registry;
    if require_trust_root && trust_root_hash.is_none() {
        return Err(diag::build_diag(
            "E_REG_TRUST_ROOT_INVALID",
            "trust_root.hash is missing",
            None,
            Some("ddn.lock에 trust_root.hash를 채우세요.".to_string()),
        ));
    }
    if let Some(expected) = options.expect_trust_root_hash.as_deref() {
        let Some(actual) = trust_root_hash else {
            return Err(diag::build_diag(
                "E_REG_TRUST_ROOT_INVALID",
                "trust_root.hash is missing",
                None,
                Some("ddn.lock에 trust_root.hash를 채우세요.".to_string()),
            ));
        };
        if actual != expected {
            return Err(diag::build_diag(
                "E_REG_TRUST_ROOT_INVALID",
                &format!("expected={} actual={}", expected, actual),
                None,
                Some("요구 trust_root_hash와 ddn.lock 값을 일치시키세요.".to_string()),
            ));
        }
    }

    Ok(())
}

fn build_vendor_index(packages: &[LockPackage]) -> Result<String, String> {
    let mut rows = Vec::with_capacity(packages.len());
    for pkg in packages {
        rows.push(json!({
            "id": pkg.id,
            "version": pkg.version,
            "path": pkg.path,
            "hash": pkg.hash,
            "yanked": pkg.yanked,
            "archive_sha256": pkg.archive_sha256,
            "download_url": pkg.download_url,
            "dependencies": pkg.dependencies,
            "contract": pkg.contract,
            "min_runtime": pkg.min_runtime,
            "detmath_seal_hash": pkg.detmath_seal_hash,
        }));
    }
    let root = json!({
        "schema_version": "ddn.vendor.index.v1",
        "packages": rows,
    });
    serde_json::to_string_pretty(&root).map_err(|e| format!("E_GAJI_JSON {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ddn_gaji_{}_{}", name, stamp));
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    fn write_registry_index(path: &Path, snapshot_id: &str, index_root_hash: &str) {
        let root = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": snapshot_id,
            "index_root_hash": index_root_hash,
            "trust_root": {
                "hash": "sha256:trust",
                "source": "registry"
            },
            "entries": []
        });
        fs::write(path, serde_json::to_string_pretty(&root).expect("json")).expect("write index");
    }

    fn write_registry_index_with_entry(
        path: &Path,
        snapshot_id: &str,
        index_root_hash: &str,
        scope: &str,
        name: &str,
        version: &str,
    ) {
        let root = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": snapshot_id,
            "index_root_hash": index_root_hash,
            "trust_root": {
                "hash": "sha256:trust",
                "source": "registry"
            },
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": scope,
                "name": name,
                "version": version,
                "yanked": false
            }]
        });
        fs::write(path, serde_json::to_string_pretty(&root).expect("json")).expect("write index");
    }

    fn write_audit_log(path: &Path, rows: &[Value]) {
        let mut out = String::new();
        for row in rows {
            let line = serde_json::to_string(row).expect("row json");
            out.push_str(&line);
            out.push('\n');
        }
        fs::write(path, out).expect("write audit");
    }

    fn make_audit_row(prev_hash: Option<&str>, action: &str, package_id: &str) -> Value {
        let body = json!({
            "schema": "ddn.registry.audit.v1",
            "ts": "2026-02-20T00:00:00Z",
            "action": action,
            "package_id": package_id,
            "role": "publisher",
            "token_hash": "blake3:token",
            "allowed": true,
            "error_code": Value::Null,
            "prev_hash": prev_hash
        });
        let body_text = serde_json::to_string(&body).expect("body json");
        let row_hash = format!("blake3:{}", blake3::hash(body_text.as_bytes()).to_hex());
        json!({
            "body": body,
            "row_hash": row_hash
        })
    }

    #[test]
    fn run_lock_and_vendor_basic_flow() {
        let root = temp_dir("lock_vendor");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");
        assert!(lock_path.exists());

        let vendor_out = root.join("vendor").join("gaji");
        run_vendor(&root, &lock_path, &vendor_out).expect("vendor");
        assert!(vendor_out.join("demo").join("gaji.toml").exists());
        assert!(vendor_out.join("ddn.vendor.index.json").exists());
    }

    #[test]
    fn run_vendor_hash_mismatch_rejected() {
        let root = temp_dir("vendor_hash");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 2.\n").expect("tamper");

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor(&root, &lock_path, &vendor_out).expect_err("must fail");
        assert!(err.contains("E_GAJI_VENDOR_HASH_MISMATCH"));
    }

    #[test]
    fn run_install_bootstraps_lock_when_missing() {
        let root = temp_dir("install_bootstrap");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        let vendor_out = root.join("vendor").join("gaji");
        run_install(&root, &lock_path, &vendor_out).expect("install");

        assert!(lock_path.exists());
        assert!(vendor_out.join("demo").join("main.ddn").exists());
    }

    #[test]
    fn run_install_strict_registry_requires_index_before_bootstrap() {
        let root = temp_dir("install_strict_need_index");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        let vendor_out = root.join("vendor").join("gaji");
        let err = run_install_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict install requires index");
        assert!(err.contains("E_REG_VERIFY_INDEX_REQUIRED"));
        assert!(err.contains("fix="));
        assert!(!lock_path.exists());
    }

    #[test]
    fn run_install_strict_registry_bootstraps_lock_from_index() {
        let root = temp_dir("install_strict_bootstrap_index");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let index_path = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index_path,
            "2026-02-20T00:00:00Z#000100",
            "sha256:index_bootstrap",
            "표준",
            "역학",
            "0.1.0",
        );

        let lock_path = root.join("ddn.lock");
        let vendor_out = root.join("vendor").join("gaji");
        run_install_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect("strict install with index");

        let lock = read_lock_file(&lock_path).expect("read lock");
        let snapshot = lock.get("registry_snapshot").expect("snapshot");
        assert_eq!(
            snapshot.get("snapshot_id").and_then(|v| v.as_str()),
            Some("2026-02-20T00:00:00Z#000100")
        );
        assert_eq!(
            snapshot.get("index_root_hash").and_then(|v| v.as_str()),
            Some("sha256:index_bootstrap")
        );
    }

    #[test]
    fn run_install_strict_registry_with_invalid_index_json_fails() {
        let root = temp_dir("install_strict_invalid_index_json");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let index_path = root.join("registry.index.json");
        fs::write(&index_path, "{ invalid json").expect("write bad index");

        let lock_path = root.join("ddn.lock");
        let vendor_out = root.join("vendor").join("gaji");
        let err = run_install_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("invalid index json must fail");
        assert!(err.contains("E_REG_INDEX_PARSE"));
        assert!(err.contains("fix="));
        assert!(!lock_path.exists());
    }

    #[test]
    fn run_install_strict_registry_with_missing_index_file_fails() {
        let root = temp_dir("install_strict_missing_index_file");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let index_path = root.join("missing.registry.index.json");
        let lock_path = root.join("ddn.lock");
        let vendor_out = root.join("vendor").join("gaji");
        let err = run_install_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("missing index file must fail");
        assert!(err.contains("E_REG_INDEX_READ"));
        assert!(err.contains("fix="));
        assert!(!lock_path.exists());
    }

    #[test]
    fn run_install_strict_registry_with_index_missing_snapshot_meta_fails() {
        let root = temp_dir("install_strict_missing_snapshot_meta");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let index_path = root.join("registry.index.json");
        let index = json!({
            "schema": "ddn.registry.snapshot.v1",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "0.1.0",
                "yanked": false
            }]
        });
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index).expect("index json"),
        )
        .expect("write index");

        let lock_path = root.join("ddn.lock");
        let vendor_out = root.join("vendor").join("gaji");
        let err = run_install_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("missing snapshot metadata must fail");
        assert!(err.contains("E_REG_SNAPSHOT_MISSING"));
        assert!(err.contains("fix="));
        assert!(!lock_path.exists());
    }

    #[test]
    fn run_install_strict_registry_with_audit_log_requires_last_hash_pin_after_bootstrap() {
        let root = temp_dir("install_strict_audit_need_pin");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let index_path = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index_path,
            "2026-02-20T00:00:00Z#000101",
            "sha256:index_bootstrap_audit",
            "표준",
            "역학",
            "0.1.0",
        );

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        write_audit_log(&audit_path, &[row1, row2]);

        let lock_path = root.join("ddn.lock");
        let vendor_out = root.join("vendor").join("gaji");
        let err = run_install_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                registry_audit_log: Some(audit_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict install with audit must require last hash pin");
        assert!(err.contains("E_REG_AUDIT_LAST_HASH_REQUIRED"));
        assert!(err.contains("fix="));
        assert!(err.contains("hint="));
        assert!(lock_path.exists());
    }

    #[test]
    fn run_install_strict_registry_with_audit_log_accepts_cli_last_hash_pin_after_bootstrap() {
        let root = temp_dir("install_strict_audit_cli_pin");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let index_path = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index_path,
            "2026-02-20T00:00:00Z#000102",
            "sha256:index_bootstrap_audit_cli",
            "표준",
            "역학",
            "0.1.0",
        );

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        let expected_last_hash = row2
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row2 hash")
            .to_string();
        write_audit_log(&audit_path, &[row1, row2]);

        let lock_path = root.join("ddn.lock");
        let vendor_out = root.join("vendor").join("gaji");
        run_install_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                registry_audit_log: Some(audit_path),
                expect_audit_last_hash: Some(expected_last_hash),
                ..FrozenLockOptions::default()
            },
        )
        .expect("strict install with cli last hash pin");
        assert!(lock_path.exists());
        assert!(vendor_out.join("demo").join("gaji.toml").exists());
    }

    #[test]
    fn run_update_refreshes_lock_and_vendor() {
        let root = temp_dir("update_refresh");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");
        let before = fs::read_to_string(&lock_path).expect("read before");

        fs::write(pkg_dir.join("main.ddn"), "값 <- 2.\n").expect("update src");
        let vendor_out = root.join("vendor").join("gaji");
        run_update(&root, &lock_path, &vendor_out).expect("update");

        let after = fs::read_to_string(&lock_path).expect("read after");
        assert_ne!(before, after);
        let vendored_main =
            fs::read_to_string(vendor_out.join("demo").join("main.ddn")).expect("vendor main");
        assert!(vendored_main.contains("값 <- 2."));
    }

    #[test]
    fn run_update_strict_registry_applies_index_meta_to_existing_lock() {
        let root = temp_dir("update_strict_apply_index_meta");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock without registry meta");

        let index_path = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index_path,
            "2026-02-20T00:00:00Z#000200",
            "sha256:index_update",
            "표준",
            "역학",
            "0.1.0",
        );

        let vendor_out = root.join("vendor").join("gaji");
        run_update_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &LockWriteOptions::default(),
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect("strict update with index");

        let lock = read_lock_file(&lock_path).expect("read lock");
        let snapshot = lock.get("registry_snapshot").expect("snapshot");
        assert_eq!(
            snapshot.get("snapshot_id").and_then(|v| v.as_str()),
            Some("2026-02-20T00:00:00Z#000200")
        );
        assert_eq!(
            snapshot.get("index_root_hash").and_then(|v| v.as_str()),
            Some("sha256:index_update")
        );
    }

    #[test]
    fn run_update_strict_registry_with_missing_index_file_fails() {
        let root = temp_dir("update_strict_missing_index_file");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock without registry meta");

        let index_path = root.join("missing.registry.index.json");
        let vendor_out = root.join("vendor").join("gaji");
        let err = run_update_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &LockWriteOptions::default(),
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict update missing index file must fail");
        assert!(err.contains("E_REG_INDEX_READ"));
        assert!(err.contains("fix="));
    }

    #[test]
    fn run_update_strict_registry_with_invalid_index_json_fails() {
        let root = temp_dir("update_strict_invalid_index_json");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock without registry meta");

        let index_path = root.join("registry.index.json");
        fs::write(&index_path, "{ invalid json").expect("write bad index");

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_update_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &LockWriteOptions::default(),
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict update invalid index json must fail");
        assert!(err.contains("E_REG_INDEX_PARSE"));
        assert!(err.contains("fix="));
    }

    #[test]
    fn run_update_strict_registry_with_audit_log_uses_lock_last_hash_pin_and_fails_on_mismatch() {
        let root = temp_dir("update_strict_audit_lock_hash_mismatch");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("2026-02-20T00:00:00Z#000201".to_string()),
                index_root_hash: Some("sha256:index_update_audit".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: Some("blake3:not-match".to_string()),
            },
        )
        .expect("lock with audit pin");

        let index_path = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index_path,
            "2026-02-20T00:00:00Z#000201",
            "sha256:index_update_audit",
            "표준",
            "역학",
            "0.1.0",
        );

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        write_audit_log(&audit_path, &[row1, row2]);

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_update_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &LockWriteOptions::default(),
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                registry_audit_log: Some(audit_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict update should fail when lock audit hash mismatches log");
        assert!(err.contains("E_REG_AUDIT_LAST_HASH_MISMATCH"));
    }

    #[test]
    fn run_lock_writes_registry_snapshot_and_trust_root() {
        let root = temp_dir("lock_meta");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        let options = LockWriteOptions {
            snapshot_id: Some("2026-02-19T00:00:00Z#000001".to_string()),
            index_root_hash: Some("sha256:index".to_string()),
            trust_root_hash: Some("sha256:trust".to_string()),
            trust_root_source: Some("registry".to_string()),
            audit_last_hash: None,
        };
        run_lock_with_options(&root, &lock_path, &options).expect("lock with meta");

        let lock = read_lock_file(&lock_path).expect("read lock");
        let snapshot = lock.get("registry_snapshot").expect("registry_snapshot");
        assert_eq!(
            snapshot.get("snapshot_id").and_then(|v| v.as_str()),
            Some("2026-02-19T00:00:00Z#000001")
        );
        assert_eq!(
            snapshot.get("index_root_hash").and_then(|v| v.as_str()),
            Some("sha256:index")
        );
        let trust_root = lock.get("trust_root").expect("trust_root");
        assert_eq!(
            trust_root.get("hash").and_then(|v| v.as_str()),
            Some("sha256:trust")
        );
        assert_eq!(
            trust_root.get("source").and_then(|v| v.as_str()),
            Some("registry")
        );
    }

    #[test]
    fn run_lock_writes_registry_audit_last_hash() {
        let root = temp_dir("lock_audit_hash");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: None,
                index_root_hash: None,
                trust_root_hash: None,
                trust_root_source: None,
                audit_last_hash: Some("blake3:audit_last".to_string()),
            },
        )
        .expect("lock with audit hash");

        let lock = read_lock_file(&lock_path).expect("read lock");
        let registry_audit = lock.get("registry_audit").expect("registry_audit");
        assert_eq!(
            registry_audit.get("last_hash").and_then(|v| v.as_str()),
            Some("blake3:audit_last")
        );
    }

    #[test]
    fn apply_registry_meta_from_index_fills_lock_options() {
        let root = temp_dir("lock_index_meta_fill");
        let index_path = root.join("registry.index.json");
        write_registry_index(
            &index_path,
            "2026-02-20T00:00:00Z#000001",
            "sha256:index_from_registry",
        );

        let mut options = LockWriteOptions::default();
        apply_registry_meta_from_index(&mut options, &index_path).expect("apply index meta");
        assert_eq!(
            options.snapshot_id.as_deref(),
            Some("2026-02-20T00:00:00Z#000001")
        );
        assert_eq!(
            options.index_root_hash.as_deref(),
            Some("sha256:index_from_registry")
        );
        assert_eq!(options.trust_root_hash.as_deref(), Some("sha256:trust"));
        assert_eq!(options.trust_root_source.as_deref(), Some("registry"));
    }

    #[test]
    fn run_lock_with_registry_index_meta_writes_snapshot_and_trust() {
        let root = temp_dir("lock_index_meta_write");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let index_path = root.join("registry.index.json");
        write_registry_index(
            &index_path,
            "2026-02-20T00:00:00Z#000002",
            "sha256:index_from_registry_2",
        );

        let mut options = LockWriteOptions::default();
        apply_registry_meta_from_index(&mut options, &index_path).expect("apply index meta");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(&root, &lock_path, &options).expect("lock with index meta");
        let lock = read_lock_file(&lock_path).expect("read lock");
        let snapshot = lock.get("registry_snapshot").expect("snapshot");
        assert_eq!(
            snapshot.get("snapshot_id").and_then(|v| v.as_str()),
            Some("2026-02-20T00:00:00Z#000002")
        );
        assert_eq!(
            snapshot.get("index_root_hash").and_then(|v| v.as_str()),
            Some("sha256:index_from_registry_2")
        );
        let trust_root = lock.get("trust_root").expect("trust_root");
        assert_eq!(
            trust_root.get("hash").and_then(|v| v.as_str()),
            Some("sha256:trust")
        );
    }

    #[test]
    fn run_vendor_frozen_requires_snapshot_meta() {
        let root = temp_dir("vendor_frozen");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");
        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                frozen_lockfile: true,
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("frozen requires snapshot");
        assert!(err.contains("E_REG_SNAPSHOT_MISSING"));
    }

    #[test]
    fn run_update_preserves_existing_lock_meta() {
        let root = temp_dir("update_preserve_meta");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("2026-02-19T00:00:00Z#000001".to_string()),
                index_root_hash: Some("sha256:index".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("mirror".to_string()),
                audit_last_hash: Some("blake3:audit_last".to_string()),
            },
        )
        .expect("lock with meta");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 2.\n").expect("update src");

        let vendor_out = root.join("vendor").join("gaji");
        run_update(&root, &lock_path, &vendor_out).expect("update");
        let lock = read_lock_file(&lock_path).expect("read lock");
        let snapshot = lock.get("registry_snapshot").expect("registry_snapshot");
        assert_eq!(
            snapshot.get("snapshot_id").and_then(|v| v.as_str()),
            Some("2026-02-19T00:00:00Z#000001")
        );
        assert_eq!(
            snapshot.get("index_root_hash").and_then(|v| v.as_str()),
            Some("sha256:index")
        );
        let trust_root = lock.get("trust_root").expect("trust_root");
        assert_eq!(
            trust_root.get("hash").and_then(|v| v.as_str()),
            Some("sha256:trust")
        );
        assert_eq!(
            trust_root.get("source").and_then(|v| v.as_str()),
            Some("mirror")
        );
        let registry_audit = lock.get("registry_audit").expect("registry_audit");
        assert_eq!(
            registry_audit.get("last_hash").and_then(|v| v.as_str()),
            Some("blake3:audit_last")
        );
    }

    #[test]
    fn run_vendor_deny_yanked_locked_rejects() {
        let root = temp_dir("vendor_deny_yanked");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");
        let mut lock = read_lock_file(&lock_path).expect("read lock");
        lock.get_mut("packages")
            .and_then(|v| v.as_array_mut())
            .and_then(|rows| rows.get_mut(0))
            .and_then(|row| row.as_object_mut())
            .expect("package row")
            .insert("yanked".to_string(), Value::Bool(true));
        fs::write(
            &lock_path,
            serde_json::to_string_pretty(&lock).expect("lock json"),
        )
        .expect("write lock");

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                deny_yanked_locked: true,
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("must reject");
        assert!(err.contains("E_REG_YANKED_LOCKED"));
        assert!(err.contains("fix="));
    }

    #[test]
    fn run_vendor_verify_registry_requires_index() {
        let root = temp_dir("vendor_verify_need_index");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot");

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry: true,
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("index required");
        assert!(err.contains("E_REG_VERIFY_INDEX_REQUIRED"));
        assert!(err.contains("fix="));
    }

    #[test]
    fn run_vendor_strict_registry_requires_index_when_frozen() {
        let root = temp_dir("vendor_strict_registry_need_index");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot");

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                frozen_lockfile: true,
                strict_registry: true,
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict frozen requires registry index");
        assert!(err.contains("E_REG_VERIFY_INDEX_REQUIRED"));
        assert!(err.contains("fix="));
    }

    #[test]
    fn run_vendor_strict_registry_with_missing_index_file_fails() {
        let root = temp_dir("vendor_strict_missing_index_file");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-vendor-missing-index".to_string()),
                index_root_hash: Some("sha256:index-vendor-missing-index".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot+trust");

        let index_path = root.join("missing.registry.index.json");
        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict vendor missing index file must fail");
        assert!(err.contains("E_REG_INDEX_READ"));
        assert!(err.contains("fix="));
    }

    #[test]
    fn run_vendor_strict_registry_with_invalid_index_json_fails() {
        let root = temp_dir("vendor_strict_invalid_index_json");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-vendor-invalid-index".to_string()),
                index_root_hash: Some("sha256:index-vendor-invalid-index".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot+trust");

        let index_path = root.join("registry.index.json");
        fs::write(&index_path, "{ invalid json").expect("write bad index");

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict vendor invalid index json must fail");
        assert!(err.contains("E_REG_INDEX_PARSE"));
        assert!(err.contains("fix="));
    }

    #[test]
    fn run_vendor_strict_registry_with_index_passes_without_verify_flag() {
        let root = temp_dir("vendor_strict_registry_ok");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot+trust");

        let index_path = root.join("registry.index.json");
        let index = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "trust_root": { "hash": "sha256:trust", "source": "registry" },
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "0.1.0",
                "yanked": false
            }]
        });
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index).expect("index json"),
        )
        .expect("write index");

        let vendor_out = root.join("vendor").join("gaji");
        run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                frozen_lockfile: true,
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect("strict frozen verify pass");
        assert!(vendor_out.join("demo").join("gaji.toml").exists());
    }

    #[test]
    fn run_vendor_strict_registry_enforces_deny_yanked_locked() {
        let root = temp_dir("vendor_strict_registry_yanked");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot+trust");

        let mut lock = read_lock_file(&lock_path).expect("read lock");
        lock.get_mut("packages")
            .and_then(|v| v.as_array_mut())
            .and_then(|rows| rows.get_mut(0))
            .and_then(|row| row.as_object_mut())
            .expect("package row")
            .insert("yanked".to_string(), Value::Bool(true));
        fs::write(
            &lock_path,
            serde_json::to_string_pretty(&lock).expect("lock json"),
        )
        .expect("write lock");

        let index_path = root.join("registry.index.json");
        let index = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "trust_root": { "hash": "sha256:trust", "source": "registry" },
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "0.1.0",
                "yanked": false
            }]
        });
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index).expect("index json"),
        )
        .expect("write index");

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                frozen_lockfile: true,
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict must reject yanked lock");
        assert!(err.contains("E_REG_YANKED_LOCKED"));
        assert!(err.contains("fix="));
    }

    #[test]
    fn run_vendor_strict_registry_enforces_trust_root() {
        let root = temp_dir("vendor_strict_registry_trust");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: None,
                trust_root_source: None,
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot only");

        let index_path = root.join("registry.index.json");
        let index = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "0.1.0",
                "yanked": false
            }]
        });
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index).expect("index json"),
        )
        .expect("write index");

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                frozen_lockfile: true,
                strict_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict must require trust_root");
        assert!(err.contains("E_REG_TRUST_ROOT_INVALID"));
    }

    #[test]
    fn run_vendor_verify_registry_passes() {
        let root = temp_dir("vendor_verify_ok");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: None,
                trust_root_source: None,
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot");

        let index_path = root.join("registry.index.json");
        let index = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "0.1.0",
                "yanked": false
            }]
        });
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index).expect("index json"),
        )
        .expect("write index");

        let vendor_out = root.join("vendor").join("gaji");
        run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect("verify pass");
        assert!(vendor_out.join("demo").join("gaji.toml").exists());
    }

    #[test]
    fn run_vendor_verify_registry_snapshot_mismatch_fails() {
        let root = temp_dir("vendor_verify_snapshot_mismatch");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: None,
                trust_root_source: None,
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot");

        let index_path = root.join("registry.index.json");
        let index = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-99",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "0.1.0",
                "yanked": false
            }]
        });
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index).expect("index json"),
        )
        .expect("write index");

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("snapshot mismatch must fail");
        assert!(err.contains("E_REG_SNAPSHOT_MISMATCH"));
    }

    #[test]
    fn run_vendor_verify_registry_writes_report_json() {
        let root = temp_dir("vendor_verify_report");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: None,
                trust_root_source: None,
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot");

        let index_path = root.join("registry.index.json");
        let index = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "0.1.0",
                "yanked": false
            }]
        });
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index).expect("index json"),
        )
        .expect("write index");

        let vendor_out = root.join("vendor").join("gaji");
        let report_out = root.join("vendor").join("registry.verify.json");
        run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry: true,
                registry_index: Some(index_path),
                registry_verify_out: Some(report_out.clone()),
                ..FrozenLockOptions::default()
            },
        )
        .expect("verify pass");

        let report: Value =
            serde_json::from_str(&fs::read_to_string(report_out).expect("read verify report"))
                .expect("parse verify report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn run_vendor_verify_registry_writes_default_report_path() {
        let root = temp_dir("vendor_verify_report_default");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot+trust");

        let index_path = root.join("registry.index.json");
        let index = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "trust_root": { "hash": "sha256:trust", "source": "registry" },
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "0.1.0",
                "yanked": false
            }]
        });
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index).expect("index json"),
        )
        .expect("write index");

        let vendor_out = root.join("vendor").join("gaji");
        run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry: true,
                registry_index: Some(index_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect("verify pass");

        let default_report = root.join("vendor").join("registry.verify.json");
        assert!(default_report.exists());
    }

    #[test]
    fn run_vendor_verify_registry_audit_requires_log() {
        let root = temp_dir("vendor_verify_audit_need_log");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");
        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry_audit: true,
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("audit log is required");
        assert!(err.contains("E_REG_AUDIT_VERIFY_LOG_REQUIRED"));
    }

    #[test]
    fn normalize_strict_registry_options_enables_core_guards() {
        let base = FrozenLockOptions {
            strict_registry: true,
            ..FrozenLockOptions::default()
        };
        let normalized = normalize_strict_registry_options(&base);
        assert!(normalized.strict_registry);
        assert!(normalized.frozen_lockfile);
        assert!(normalized.verify_registry);
        assert!(normalized.require_trust_root);
        assert!(normalized.deny_yanked_locked);
    }

    #[test]
    fn normalize_strict_registry_options_auto_enables_audit_verify_when_log_exists() {
        let base = FrozenLockOptions {
            strict_registry: true,
            registry_audit_log: Some(PathBuf::from("audit.jsonl")),
            ..FrozenLockOptions::default()
        };
        let normalized = normalize_strict_registry_options(&base);
        assert!(normalized.verify_registry_audit);
        assert_eq!(
            normalized.registry_audit_log.as_deref(),
            Some(Path::new("audit.jsonl"))
        );
    }

    #[test]
    fn run_vendor_verify_registry_audit_passes() {
        let root = temp_dir("vendor_verify_audit_ok");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        write_audit_log(&audit_path, &[row1, row2]);

        let vendor_out = root.join("vendor").join("gaji");
        run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry_audit: true,
                registry_audit_log: Some(audit_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect("audit verify pass");
        assert!(vendor_out.join("demo").join("gaji.toml").exists());
    }

    #[test]
    fn run_vendor_verify_registry_audit_writes_report_json() {
        let root = temp_dir("vendor_verify_audit_report");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        write_audit_log(&audit_path, &[row1, row2]);

        let report_path = root.join("vendor").join("registry.audit.verify.json");
        let vendor_out = root.join("vendor").join("gaji");
        run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry_audit: true,
                registry_audit_log: Some(audit_path),
                registry_audit_verify_out: Some(report_path.clone()),
                ..FrozenLockOptions::default()
            },
        )
        .expect("audit verify pass");

        let report: Value = serde_json::from_str(
            &fs::read_to_string(report_path).expect("read audit verify report"),
        )
        .expect("parse audit verify report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.audit_verify_report.v1")
        );
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn run_vendor_verify_registry_audit_writes_default_report_path() {
        let root = temp_dir("vendor_verify_audit_report_default");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        write_audit_log(&audit_path, &[row1, row2]);

        let vendor_out = root.join("vendor").join("gaji");
        run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry_audit: true,
                registry_audit_log: Some(audit_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect("audit verify pass");

        let default_report = root.join("vendor").join("registry.audit.verify.json");
        assert!(default_report.exists());
    }

    #[test]
    fn run_vendor_verify_registry_audit_expect_last_hash_mismatch_fails() {
        let root = temp_dir("vendor_verify_audit_expect_last_hash_bad");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        write_audit_log(&audit_path, &[row1, row2]);

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry_audit: true,
                registry_audit_log: Some(audit_path),
                expect_audit_last_hash: Some("blake3:not-match".to_string()),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("must fail on expected last hash mismatch");
        assert!(err.contains("E_REG_AUDIT_LAST_HASH_MISMATCH"));
    }

    #[test]
    fn run_vendor_verify_registry_audit_uses_lock_last_hash_by_default() {
        let root = temp_dir("vendor_audit_lock_last_hash");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"demo/pkg\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: None,
                index_root_hash: None,
                trust_root_hash: None,
                trust_root_source: None,
                audit_last_hash: Some("blake3:not-match".to_string()),
            },
        )
        .expect("lock with audit hash");

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        write_audit_log(&audit_path, &[row1, row2]);

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                verify_registry_audit: true,
                registry_audit_log: Some(audit_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("lock hash mismatch must fail");
        assert!(err.contains("E_REG_AUDIT_LAST_HASH_MISMATCH"));
    }

    #[test]
    fn run_vendor_strict_registry_with_audit_log_requires_last_hash_pin() {
        let root = temp_dir("vendor_strict_audit_last_hash_required");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-need-pin".to_string()),
                index_root_hash: Some("sha256:index-need-pin".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot+trust only");

        let index_path = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index_path,
            "snap-need-pin",
            "sha256:index-need-pin",
            "표준",
            "역학",
            "0.1.0",
        );

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        let expected_hint_hash = row2
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row2 hash")
            .to_string();
        write_audit_log(&audit_path, &[row1, row2]);

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                frozen_lockfile: true,
                registry_index: Some(index_path),
                registry_audit_log: Some(audit_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict+audit must require lock/cli last hash pin");
        assert!(err.contains("E_REG_AUDIT_LAST_HASH_REQUIRED"));
        assert!(err.contains("fix="));
        assert!(err.contains("hint="));
        assert!(err.contains(&expected_hint_hash));
    }

    #[test]
    fn run_vendor_strict_registry_with_audit_log_accepts_cli_last_hash_pin() {
        let root = temp_dir("vendor_strict_audit_last_hash_cli");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-cli-pin".to_string()),
                index_root_hash: Some("sha256:index-cli-pin".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot+trust only");

        let index_path = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index_path,
            "snap-cli-pin",
            "sha256:index-cli-pin",
            "표준",
            "역학",
            "0.1.0",
        );

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let row1_hash = row1
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row1 hash")
            .to_string();
        let row2 = make_audit_row(Some(&row1_hash), "yank", "표준/역학@0.1.0");
        let expected_last_hash = row2
            .get("row_hash")
            .and_then(|v| v.as_str())
            .expect("row2 hash")
            .to_string();
        write_audit_log(&audit_path, &[row1, row2]);

        let vendor_out = root.join("vendor").join("gaji");
        run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                strict_registry: true,
                frozen_lockfile: true,
                registry_index: Some(index_path),
                registry_audit_log: Some(audit_path),
                expect_audit_last_hash: Some(expected_last_hash),
                ..FrozenLockOptions::default()
            },
        )
        .expect("strict+audit with cli pin");
        assert!(vendor_out.join("demo").join("gaji.toml").exists());
    }

    #[test]
    fn run_vendor_strict_registry_auto_verifies_audit_when_log_given() {
        let root = temp_dir("vendor_strict_auto_audit");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock_with_options(
            &root,
            &lock_path,
            &LockWriteOptions {
                snapshot_id: Some("snap-42".to_string()),
                index_root_hash: Some("sha256:abc".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: Some("blake3:strict-audit-pin".to_string()),
            },
        )
        .expect("lock with snapshot+trust");

        let index_path = root.join("registry.index.json");
        let index = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "trust_root": { "hash": "sha256:trust", "source": "registry" },
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "0.1.0",
                "yanked": false
            }]
        });
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index).expect("index json"),
        )
        .expect("write index");

        let audit_path = root.join("registry.audit.jsonl");
        let row1 = make_audit_row(None, "publish", "표준/역학@0.1.0");
        let mut row2 = make_audit_row(
            row1.get("row_hash").and_then(|v| v.as_str()),
            "yank",
            "표준/역학@0.1.0",
        );
        row2["row_hash"] = Value::String("blake3:tampered".to_string());
        write_audit_log(&audit_path, &[row1, row2]);

        let vendor_out = root.join("vendor").join("gaji");
        let err = run_vendor_with_options(
            &root,
            &lock_path,
            &vendor_out,
            &FrozenLockOptions {
                frozen_lockfile: true,
                strict_registry: true,
                registry_index: Some(index_path),
                registry_audit_log: Some(audit_path),
                ..FrozenLockOptions::default()
            },
        )
        .expect_err("strict+frozen should auto-verify audit and fail");
        assert!(err.contains("E_REG_AUDIT_ROW_HASH_MISMATCH"));
    }

    #[test]
    fn run_vendor_index_keeps_registry_meta_fields() {
        let root = temp_dir("vendor_index_meta");
        let pkg_dir = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg_dir).expect("pkg mkdir");
        fs::write(
            pkg_dir.join("gaji.toml"),
            "id = \"표준/역학\"\nversion = \"0.1.0\"\n",
        )
        .expect("write toml");
        fs::write(pkg_dir.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let lock_path = root.join("ddn.lock");
        run_lock(&root, &lock_path).expect("lock");

        let mut lock = read_lock_file(&lock_path).expect("read lock");
        let row = lock
            .get_mut("packages")
            .and_then(|v| v.as_array_mut())
            .and_then(|rows| rows.get_mut(0))
            .and_then(|v| v.as_object_mut())
            .expect("pkg row");
        row.insert(
            "archive_sha256".to_string(),
            Value::String("sha256:archive-a".to_string()),
        );
        row.insert(
            "download_url".to_string(),
            Value::String("https://registry/a".to_string()),
        );
        row.insert("dependencies".to_string(), json!({"표준/수학": "1.0.0"}));
        row.insert(
            "contract".to_string(),
            Value::String("D-STRICT".to_string()),
        );
        row.insert(
            "min_runtime".to_string(),
            Value::String("20.6.29".to_string()),
        );
        row.insert(
            "detmath_seal_hash".to_string(),
            Value::String("sha256:seal-a".to_string()),
        );
        fs::write(
            &lock_path,
            serde_json::to_string_pretty(&lock).expect("lock json"),
        )
        .expect("write lock");

        let vendor_out = root.join("vendor").join("gaji");
        run_vendor(&root, &lock_path, &vendor_out).expect("vendor");

        let index: Value = serde_json::from_str(
            &fs::read_to_string(vendor_out.join("ddn.vendor.index.json")).expect("read index"),
        )
        .expect("parse index");
        let pkg = index
            .get("packages")
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.first())
            .expect("pkg");
        assert_eq!(
            pkg.get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:archive-a")
        );
        assert_eq!(
            pkg.get("download_url").and_then(|v| v.as_str()),
            Some("https://registry/a")
        );
        assert_eq!(
            pkg.get("dependencies")
                .and_then(|v| v.get("표준/수학"))
                .and_then(|v| v.as_str()),
            Some("1.0.0")
        );
        assert_eq!(
            pkg.get("contract").and_then(|v| v.as_str()),
            Some("D-STRICT")
        );
        assert_eq!(
            pkg.get("min_runtime").and_then(|v| v.as_str()),
            Some("20.6.29")
        );
        assert_eq!(
            pkg.get("detmath_seal_hash").and_then(|v| v.as_str()),
            Some("sha256:seal-a")
        );
    }
}
