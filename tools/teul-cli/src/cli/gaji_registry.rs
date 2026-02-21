#![allow(dead_code)]

use std::cmp::{Ordering, Reverse};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::diag;
use blake3;
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub const VERIFY_DUPLICATE_RESOLUTION_POLICY: &str =
    "non_yanked_then_pin_score_then_normalized_entry_key";

#[derive(Clone, Debug)]
struct RegistryEntry {
    scope: String,
    name: String,
    version: String,
    archive_sha256: Option<String>,
    contract: Option<String>,
    detmath_seal_hash: Option<String>,
    min_runtime: Option<String>,
    dependencies: Option<Value>,
    download_url: Option<String>,
    published_at: Option<String>,
    summary: Option<String>,
    yanked: bool,
    yanked_at: Option<String>,
    yank_reason_code: Option<String>,
    yank_note: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct SnapshotMeta {
    snapshot_id: Option<String>,
    index_root_hash: Option<String>,
    trust_root_hash: Option<String>,
    trust_root_source: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ReadGuardOptions {
    pub frozen_lockfile: bool,
    pub expect_snapshot_id: Option<String>,
    pub expect_index_root_hash: Option<String>,
    pub expect_trust_root_hash: Option<String>,
    pub require_trust_root: bool,
}

impl ReadGuardOptions {
    fn is_enabled(&self) -> bool {
        self.frozen_lockfile
            || self.expect_snapshot_id.is_some()
            || self.expect_index_root_hash.is_some()
            || self.expect_trust_root_hash.is_some()
            || self.require_trust_root
    }
}

#[derive(Clone, Debug, Default)]
pub struct DownloadOptions {
    pub include_yanked: bool,
    pub cache_dir: Option<PathBuf>,
    pub offline: bool,
    pub allow_http: bool,
}

#[derive(Parser, Debug)]
#[command(name = "teul-cli gaji registry")]
struct RegistryCli {
    #[command(subcommand)]
    command: RegistryCommand,
}

#[derive(Subcommand, Debug)]
enum RegistryCommand {
    Versions {
        #[arg(long)]
        index: std::path::PathBuf,
        #[arg(long)]
        lock: Option<std::path::PathBuf>,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        name: String,
        #[arg(long = "include-yanked")]
        include_yanked: bool,
        #[arg(long = "frozen-lockfile")]
        frozen_lockfile: bool,
        #[arg(long = "expect-snapshot-id")]
        expect_snapshot_id: Option<String>,
        #[arg(long = "expect-index-root-hash")]
        expect_index_root_hash: Option<String>,
        #[arg(long = "expect-trust-root-hash")]
        expect_trust_root_hash: Option<String>,
        #[arg(long = "require-trust-root")]
        require_trust_root: bool,
    },
    Entry {
        #[arg(long)]
        index: std::path::PathBuf,
        #[arg(long)]
        lock: Option<std::path::PathBuf>,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        version: String,
        #[arg(long = "frozen-lockfile")]
        frozen_lockfile: bool,
        #[arg(long = "expect-snapshot-id")]
        expect_snapshot_id: Option<String>,
        #[arg(long = "expect-index-root-hash")]
        expect_index_root_hash: Option<String>,
        #[arg(long = "expect-trust-root-hash")]
        expect_trust_root_hash: Option<String>,
        #[arg(long = "require-trust-root")]
        require_trust_root: bool,
    },
    Search {
        #[arg(long)]
        index: std::path::PathBuf,
        #[arg(long)]
        lock: Option<std::path::PathBuf>,
        #[arg(long)]
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long = "include-yanked")]
        include_yanked: bool,
        #[arg(long = "frozen-lockfile")]
        frozen_lockfile: bool,
        #[arg(long = "expect-snapshot-id")]
        expect_snapshot_id: Option<String>,
        #[arg(long = "expect-index-root-hash")]
        expect_index_root_hash: Option<String>,
        #[arg(long = "expect-trust-root-hash")]
        expect_trust_root_hash: Option<String>,
        #[arg(long = "require-trust-root")]
        require_trust_root: bool,
    },
    Download {
        #[arg(long)]
        index: std::path::PathBuf,
        #[arg(long)]
        lock: Option<std::path::PathBuf>,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        version: String,
        #[arg(long)]
        out: std::path::PathBuf,
        #[arg(long = "include-yanked")]
        include_yanked: bool,
        #[arg(long = "cache-dir")]
        cache_dir: Option<std::path::PathBuf>,
        #[arg(long = "offline")]
        offline: bool,
        #[arg(long = "allow-http")]
        allow_http: bool,
        #[arg(long = "frozen-lockfile")]
        frozen_lockfile: bool,
        #[arg(long = "expect-snapshot-id")]
        expect_snapshot_id: Option<String>,
        #[arg(long = "expect-index-root-hash")]
        expect_index_root_hash: Option<String>,
        #[arg(long = "expect-trust-root-hash")]
        expect_trust_root_hash: Option<String>,
        #[arg(long = "require-trust-root")]
        require_trust_root: bool,
    },
    Publish {
        #[arg(long)]
        index: std::path::PathBuf,
        #[arg(long = "audit-log")]
        audit_log: Option<String>,
        #[arg(long = "auth-policy")]
        auth_policy: Option<std::path::PathBuf>,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        version: String,
        #[arg(long = "archive-sha256")]
        archive_sha256: String,
        #[arg(long)]
        contract: Option<String>,
        #[arg(long = "detmath-seal-hash")]
        detmath_seal_hash: Option<String>,
        #[arg(long = "min-runtime")]
        min_runtime: Option<String>,
        #[arg(long = "download-url")]
        download_url: Option<String>,
        #[arg(long)]
        summary: Option<String>,
        #[arg(long)]
        token: String,
        #[arg(long)]
        role: String,
        #[arg(long = "at")]
        at: Option<String>,
    },
    Yank {
        #[arg(long)]
        index: std::path::PathBuf,
        #[arg(long = "audit-log")]
        audit_log: Option<String>,
        #[arg(long = "auth-policy")]
        auth_policy: Option<std::path::PathBuf>,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        version: String,
        #[arg(long = "reason-code")]
        reason_code: String,
        #[arg(long)]
        note: Option<String>,
        #[arg(long)]
        token: String,
        #[arg(long)]
        role: String,
        #[arg(long = "at")]
        at: Option<String>,
    },
    Verify {
        #[arg(long)]
        index: std::path::PathBuf,
        #[arg(long)]
        lock: std::path::PathBuf,
        #[arg(long)]
        out: Option<std::path::PathBuf>,
        #[arg(long = "frozen-lockfile")]
        frozen_lockfile: bool,
        #[arg(long = "expect-snapshot-id")]
        expect_snapshot_id: Option<String>,
        #[arg(long = "expect-index-root-hash")]
        expect_index_root_hash: Option<String>,
        #[arg(long = "expect-trust-root-hash")]
        expect_trust_root_hash: Option<String>,
        #[arg(long = "require-trust-root")]
        require_trust_root: bool,
        #[arg(long = "deny-yanked-locked")]
        deny_yanked_locked: bool,
        #[arg(long = "verify-audit")]
        verify_audit: bool,
        #[arg(long = "audit-log")]
        audit_log: Option<std::path::PathBuf>,
        #[arg(long = "audit-out")]
        audit_out: Option<std::path::PathBuf>,
        #[arg(long = "expect-audit-last-hash")]
        expect_audit_last_hash: Option<String>,
    },
    AuditVerify {
        #[arg(long = "audit-log")]
        audit_log: std::path::PathBuf,
        #[arg(long)]
        out: Option<std::path::PathBuf>,
        #[arg(long = "expect-audit-last-hash")]
        expect_audit_last_hash: Option<String>,
    },
}

pub fn run_cli(args: &[String]) -> Result<(), String> {
    let mut argv: Vec<String> = vec!["teul-cli gaji registry".to_string()];
    argv.extend(args.iter().cloned());

    let cli = match RegistryCli::try_parse_from(argv) {
        Ok(v) => v,
        Err(err) => {
            if matches!(
                err.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                print!("{err}");
                return Ok(());
            }
            return Err(err.to_string());
        }
    };

    match cli.command {
        RegistryCommand::Versions {
            index,
            lock,
            scope,
            name,
            include_yanked,
            frozen_lockfile,
            expect_snapshot_id,
            expect_index_root_hash,
            expect_trust_root_hash,
            require_trust_root,
        } => {
            let guard = build_read_guard(
                lock.as_deref(),
                frozen_lockfile,
                expect_snapshot_id,
                expect_index_root_hash,
                expect_trust_root_hash,
                require_trust_root,
            )?;
            run_versions_with_guard(&index, &scope, &name, include_yanked, &guard)
        }
        RegistryCommand::Entry {
            index,
            lock,
            scope,
            name,
            version,
            frozen_lockfile,
            expect_snapshot_id,
            expect_index_root_hash,
            expect_trust_root_hash,
            require_trust_root,
        } => {
            let guard = build_read_guard(
                lock.as_deref(),
                frozen_lockfile,
                expect_snapshot_id,
                expect_index_root_hash,
                expect_trust_root_hash,
                require_trust_root,
            )?;
            run_entry_with_guard(&index, &scope, &name, &version, &guard)
        }
        RegistryCommand::Search {
            index,
            lock,
            query,
            limit,
            include_yanked,
            frozen_lockfile,
            expect_snapshot_id,
            expect_index_root_hash,
            expect_trust_root_hash,
            require_trust_root,
        } => {
            let guard = build_read_guard(
                lock.as_deref(),
                frozen_lockfile,
                expect_snapshot_id,
                expect_index_root_hash,
                expect_trust_root_hash,
                require_trust_root,
            )?;
            run_search_with_guard(&index, &query, limit, include_yanked, &guard)
        }
        RegistryCommand::Download {
            index,
            lock,
            scope,
            name,
            version,
            out,
            include_yanked,
            cache_dir,
            offline,
            allow_http,
            frozen_lockfile,
            expect_snapshot_id,
            expect_index_root_hash,
            expect_trust_root_hash,
            require_trust_root,
        } => {
            let guard = build_read_guard(
                lock.as_deref(),
                frozen_lockfile,
                expect_snapshot_id,
                expect_index_root_hash,
                expect_trust_root_hash,
                require_trust_root,
            )?;
            run_download_with_options(
                &index,
                &scope,
                &name,
                &version,
                &out,
                &guard,
                &DownloadOptions {
                    include_yanked,
                    cache_dir,
                    offline,
                    allow_http,
                },
            )
        }
        RegistryCommand::Publish {
            index,
            audit_log,
            auth_policy,
            scope,
            name,
            version,
            archive_sha256,
            contract,
            detmath_seal_hash,
            min_runtime,
            download_url,
            summary,
            token,
            role,
            at,
        } => {
            let options = PublishOptions {
                audit_log,
                scope,
                name,
                version,
                archive_sha256,
                contract,
                detmath_seal_hash,
                min_runtime,
                download_url,
                summary,
                token,
                role,
                at,
            };
            run_publish_with_auth_policy(&index, &options, auth_policy.as_deref())
        }
        RegistryCommand::Yank {
            index,
            audit_log,
            auth_policy,
            scope,
            name,
            version,
            reason_code,
            note,
            token,
            role,
            at,
        } => {
            let options = YankOptions {
                audit_log,
                scope,
                name,
                version,
                reason_code,
                note,
                token,
                role,
                at,
            };
            run_yank_with_auth_policy(&index, &options, auth_policy.as_deref())
        }
        RegistryCommand::Verify {
            index,
            lock,
            out,
            frozen_lockfile,
            expect_snapshot_id,
            expect_index_root_hash,
            expect_trust_root_hash,
            require_trust_root,
            deny_yanked_locked,
            verify_audit,
            audit_log,
            audit_out,
            expect_audit_last_hash,
        } => {
            let guard = build_read_guard(
                Some(lock.as_path()),
                frozen_lockfile,
                expect_snapshot_id,
                expect_index_root_hash,
                expect_trust_root_hash,
                require_trust_root,
            )?;
            let report = run_verify_with_guard(&index, &lock, &guard, deny_yanked_locked)?;
            let out_path = out.unwrap_or_else(|| lock.with_extension("verify.report.json"));
            write_verify_report(&out_path, &report)?;
            if verify_audit {
                let Some(audit_path) = audit_log.as_deref() else {
                    return Err(diag::build_diag(
                        "E_REG_AUDIT_VERIFY_LOG_REQUIRED",
                        "--verify-audit requires --audit-log",
                        None,
                        Some("add --audit-log <path>".to_string()),
                    ));
                };
                let audit_report = run_audit_verify(audit_path)?;
                ensure_expected_audit_last_hash(&audit_report, expect_audit_last_hash.as_deref())?;
                let out_path =
                    audit_out.unwrap_or_else(|| lock.with_extension("audit.verify.report.json"));
                write_audit_verify_report(&out_path, &audit_report)?;
            }
            Ok(())
        }
        RegistryCommand::AuditVerify {
            audit_log,
            out,
            expect_audit_last_hash,
        } => {
            let report = run_audit_verify(&audit_log)?;
            ensure_expected_audit_last_hash(&report, expect_audit_last_hash.as_deref())?;
            let out_path = out.unwrap_or_else(|| audit_log.with_extension("verify.report.json"));
            write_audit_verify_report(&out_path, &report)?;
            Ok(())
        }
    }
}

fn build_read_guard(
    lock_path: Option<&Path>,
    frozen_lockfile: bool,
    expect_snapshot_id: Option<String>,
    expect_index_root_hash: Option<String>,
    expect_trust_root_hash: Option<String>,
    require_trust_root: bool,
) -> Result<ReadGuardOptions, String> {
    let mut guard = ReadGuardOptions {
        frozen_lockfile,
        expect_snapshot_id,
        expect_index_root_hash,
        expect_trust_root_hash,
        require_trust_root,
    };

    if let Some(path) = lock_path {
        let lock = read_lock_guard_meta(path)?;
        if guard.expect_snapshot_id.is_none() {
            guard.expect_snapshot_id = lock.snapshot_id;
        }
        if guard.expect_index_root_hash.is_none() {
            guard.expect_index_root_hash = lock.index_root_hash;
        }
        if guard.expect_trust_root_hash.is_none() {
            guard.expect_trust_root_hash = lock.trust_root_hash;
        }
        if guard.frozen_lockfile
            && (guard.expect_snapshot_id.is_none() || guard.expect_index_root_hash.is_none())
        {
            return Err(diag::build_diag(
                "E_REG_SNAPSHOT_MISSING",
                "frozen-lockfile requires ddn.lock registry_snapshot(snapshot_id/index_root_hash)",
                None,
                Some(
                    "ddn.lock에 registry_snapshot.snapshot_id/index_root_hash를 채우세요."
                        .to_string(),
                ),
            ));
        }
    }

    Ok(guard)
}

#[derive(Default)]
struct LockGuardMeta {
    snapshot_id: Option<String>,
    index_root_hash: Option<String>,
    trust_root_hash: Option<String>,
}

fn read_lock_guard_meta(path: &Path) -> Result<LockGuardMeta, String> {
    let value = read_lock_json(path)?;

    let snapshot = value.get("registry_snapshot");
    let trust_root = value.get("trust_root");

    Ok(LockGuardMeta {
        snapshot_id: snapshot
            .and_then(|v| v.get("snapshot_id"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        index_root_hash: snapshot
            .and_then(|v| v.get("index_root_hash"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        trust_root_hash: trust_root
            .and_then(|v| v.get("hash"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
    })
}

fn read_lock_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| {
        diag::build_diag(
            "E_REG_LOCK_READ",
            &format!("path={} {}", path.display(), e),
            None,
            Some("ddn.lock 파일 경로/권한을 확인하세요.".to_string()),
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|e| {
        diag::build_diag(
            "E_REG_LOCK_PARSE",
            &format!("path={} {}", path.display(), e),
            Some("ddn.lock JSON 문법 오류".to_string()),
            Some("ddn.lock JSON을 정정하세요.".to_string()),
        )
    })?;
    let schema = value
        .get("schema_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if schema != "v1" {
        return Err(diag::build_diag(
            "E_REG_LOCK_SCHEMA",
            &format!("schema_version={} (need v1)", schema),
            None,
            Some("ddn.lock schema_version을 v1로 맞추세요.".to_string()),
        ));
    }
    Ok(value)
}

#[derive(Clone, Debug)]
struct LockVerifyPackage {
    id: String,
    version: String,
    yanked: bool,
    archive_sha256: Option<String>,
    download_url: Option<String>,
    dependencies: Option<Value>,
    contract: Option<String>,
    min_runtime: Option<String>,
    detmath_seal_hash: Option<String>,
}

fn read_lock_verify_packages(path: &Path) -> Result<Vec<LockVerifyPackage>, String> {
    let root = read_lock_json(path)?;
    let rows = root
        .get("packages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            diag::build_diag(
                "E_REG_LOCK_PACKAGES",
                "packages 배열이 없습니다.",
                None,
                Some("ddn.lock에 packages[]를 채우세요.".to_string()),
            )
        })?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id = req_str(row, "id")?.to_string();
        let version = req_str(row, "version")?.to_string();
        let yanked = row.get("yanked").and_then(|v| v.as_bool()).unwrap_or(false);
        let archive_sha256 = row
            .get("archive_sha256")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let download_url = row
            .get("download_url")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let dependencies = row.get("dependencies").cloned();
        let contract = row
            .get("contract")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let min_runtime = row
            .get("min_runtime")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let detmath_seal_hash = row
            .get("detmath_seal_hash")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        out.push(LockVerifyPackage {
            id,
            version,
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

pub fn run_verify_with_guard(
    index_path: &Path,
    lock_path: &Path,
    guard: &ReadGuardOptions,
    deny_yanked_locked: bool,
) -> Result<VerifyReport, String> {
    let entries = load_entries_with_guard(index_path, guard)?;
    let lock_packages = read_lock_verify_packages(lock_path)?;
    let mut matched = 0usize;
    let mut yanked_lock = 0usize;
    let mut yanked_index = 0usize;

    for pkg in &lock_packages {
        if pkg.version.trim().is_empty() {
            return Err(diag::build_diag(
                "E_REG_INDEX_FIELD",
                &format!("version 누락 id={}", pkg.id),
                None,
                Some("ddn.lock packages[].version에 비어있지 않은 버전을 적으세요.".to_string()),
            ));
        }
        if pkg.version.trim() != pkg.version {
            return Err(diag::build_diag(
                "E_REG_INDEX_FIELD",
                &format!("version 공백 포함 id={} version={}", pkg.id, pkg.version),
                None,
                Some("ddn.lock packages[].version의 앞뒤 공백을 제거하세요.".to_string()),
            ));
        }

        let Some((scope, name)) = pkg.id.split_once('/') else {
            return Err(diag::build_diag(
                "E_REG_LOCK_PACKAGE_ID_INVALID",
                &format!("id={}", pkg.id),
                Some("lock package id must be scope/name".to_string()),
                Some("ddn.lock packages[].id를 '<scope>/<name>' 형식으로 고치세요.".to_string()),
            ));
        };
        if scope.trim().is_empty() || name.trim().is_empty() {
            return Err(diag::build_diag(
                "E_REG_LOCK_PACKAGE_ID_INVALID",
                &format!("id={}", pkg.id),
                Some("scope/name must be non-empty".to_string()),
                Some("ddn.lock packages[].id에서 '/' 앞뒤를 비우지 마세요.".to_string()),
            ));
        }
        if scope.trim() != scope || name.trim() != name {
            return Err(diag::build_diag(
                "E_REG_LOCK_PACKAGE_ID_INVALID",
                &format!("id={}", pkg.id),
                Some("scope/name must not contain surrounding spaces".to_string()),
                Some("ddn.lock packages[].id에서 '/' 앞뒤 공백을 제거하세요.".to_string()),
            ));
        }
        if scope.chars().any(|c| c.is_whitespace()) || name.chars().any(|c| c.is_whitespace()) {
            return Err(diag::build_diag(
                "E_REG_LOCK_PACKAGE_ID_INVALID",
                &format!("id={}", pkg.id),
                Some("scope/name must not contain whitespace".to_string()),
                Some(
                    "ddn.lock packages[].id에서 공백문자(띄어쓰기/탭/개행)를 제거하세요."
                        .to_string(),
                ),
            ));
        }
        if name.contains('/') {
            return Err(diag::build_diag(
                "E_REG_LOCK_PACKAGE_ID_INVALID",
                &format!("id={}", pkg.id),
                Some("id contains extra '/'".to_string()),
                Some("ddn.lock packages[].id에 '/'는 한 번만 쓰세요.".to_string()),
            ));
        }

        let Some(entry) = entries
            .iter()
            .filter(|e| e.scope == scope && e.name == name && e.version == pkg.version)
            // same pin duplicates can exist in dirty/merged snapshots.
            // Prefer non-yanked and the entry that matches more pinned metadata.
            .min_by_key(|e| verify_duplicate_entry_rank(e, pkg))
        else {
            return Err(diag::build_diag(
                "E_REG_INDEX_NOT_FOUND",
                &format!("scope={} name={} version={}", scope, name, pkg.version),
                Some("lock pin not found in registry index snapshot".to_string()),
                Some(
                    "lock/index snapshot을 같은 기준으로 갱신하거나 pin 버전을 정정하세요."
                        .to_string(),
                ),
            ));
        };
        matched += 1;

        if pkg.yanked {
            yanked_lock += 1;
        }
        if entry.yanked {
            yanked_index += 1;
        }

        if deny_yanked_locked && (pkg.yanked || entry.yanked) {
            return Err(diag::build_diag(
                "E_REG_YANKED_LOCKED",
                &format!("id={} version={}", pkg.id, pkg.version),
                None,
                Some(
                    "잠금 해소를 갱신하거나 --deny-yanked-locked 설정을 재검토하세요.".to_string(),
                ),
            ));
        }

        if let Some(expected) = pkg.archive_sha256.as_deref() {
            let actual = entry.archive_sha256.as_deref().unwrap_or("<missing>");
            if actual != expected {
                return Err(diag::build_diag(
                    "E_REG_ARCHIVE_SHA256_MISMATCH",
                    &format!(
                        "id={} version={} expected={} actual={}",
                        pkg.id, pkg.version, expected, actual
                    ),
                    None,
                    Some(
                        "registry index의 archive_sha256 또는 lock pin을 다시 맞추세요."
                            .to_string(),
                    ),
                ));
            }
        }
        if let Some(expected) = pkg.download_url.as_deref() {
            let actual = entry.download_url.as_deref().unwrap_or("<missing>");
            if actual != expected {
                return Err(diag::build_diag(
                    "E_REG_DOWNLOAD_URL_MISMATCH",
                    &format!(
                        "id={} version={} expected={} actual={}",
                        pkg.id, pkg.version, expected, actual
                    ),
                    None,
                    Some(
                        "registry index의 download_url 또는 lock pin을 다시 맞추세요.".to_string(),
                    ),
                ));
            }
        }
        if let Some(expected) = pkg.dependencies.as_ref() {
            let actual = entry.dependencies.as_ref().unwrap_or(&Value::Null);
            if normalized_json_text(expected)? != normalized_json_text(actual)? {
                return Err(diag::build_diag(
                    "E_REG_DEPENDENCIES_MISMATCH",
                    &format!("id={} version={}", pkg.id, pkg.version),
                    None,
                    Some("lock의 dependencies와 index의 dependencies를 동기화하세요.".to_string()),
                ));
            }
        }

        if let Some(expected) = pkg.contract.as_deref() {
            let actual = entry.contract.as_deref().unwrap_or("<missing>");
            if actual != expected {
                return Err(diag::build_diag(
                    "E_REG_CONTRACT_MISMATCH",
                    &format!(
                        "id={} version={} expected={} actual={}",
                        pkg.id, pkg.version, expected, actual
                    ),
                    None,
                    Some("lock의 contract pin과 index 값을 일치시키세요.".to_string()),
                ));
            }
        }
        if let Some(expected) = pkg.min_runtime.as_deref() {
            let actual = entry.min_runtime.as_deref().unwrap_or("<missing>");
            if actual != expected {
                return Err(diag::build_diag(
                    "E_REG_MIN_RUNTIME_MISMATCH",
                    &format!(
                        "id={} version={} expected={} actual={}",
                        pkg.id, pkg.version, expected, actual
                    ),
                    None,
                    Some("lock의 min_runtime pin과 index 값을 일치시키세요.".to_string()),
                ));
            }
        }
        if let Some(expected) = pkg.detmath_seal_hash.as_deref() {
            let actual = entry.detmath_seal_hash.as_deref().unwrap_or("<missing>");
            if actual != expected {
                return Err(diag::build_diag(
                    "E_REG_DETMATH_SEAL_MISMATCH",
                    &format!(
                        "id={} version={} expected={} actual={}",
                        pkg.id, pkg.version, expected, actual
                    ),
                    None,
                    Some("lock의 detmath_seal_hash pin과 index 값을 일치시키세요.".to_string()),
                ));
            }
        }
    }

    println!("registry_verify_ok=1");
    println!("registry_verify_packages={}", lock_packages.len());
    println!("registry_verify_matched={}", matched);
    println!("registry_verify_yanked_lock={}", yanked_lock);
    println!("registry_verify_yanked_index={}", yanked_index);
    println!(
        "registry_verify_duplicate_resolution_policy={}",
        VERIFY_DUPLICATE_RESOLUTION_POLICY
    );
    Ok(VerifyReport {
        index_path: index_path.display().to_string(),
        lock_path: lock_path.display().to_string(),
        packages: lock_packages.len(),
        matched,
        yanked_lock,
        yanked_index,
        duplicate_resolution_policy: VERIFY_DUPLICATE_RESOLUTION_POLICY,
    })
}

#[derive(Clone, Debug)]
pub struct VerifyReport {
    index_path: String,
    lock_path: String,
    packages: usize,
    matched: usize,
    yanked_lock: usize,
    yanked_index: usize,
    duplicate_resolution_policy: &'static str,
}

pub fn write_verify_report(path: &Path, report: &VerifyReport) -> Result<(), String> {
    let root = json!({
        "schema": "ddn.registry.verify_report.v1",
        "ok": true,
        "index_path": report.index_path,
        "lock_path": report.lock_path,
        "packages": report.packages,
        "matched": report.matched,
        "yanked_lock": report.yanked_lock,
        "yanked_index": report.yanked_index,
        "duplicate_resolution_policy": report.duplicate_resolution_policy,
    });
    let text = serde_json::to_string_pretty(&root).map_err(|e| {
        diag::build_diag(
            "E_REG_JSON",
            &format!("verify report serialize failed: {}", e),
            None,
            Some("report payload를 점검하세요.".to_string()),
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            diag::build_diag(
                "E_REG_REPORT_WRITE",
                &format!("path={} {}", path.display(), e),
                None,
                Some("report 출력 경로를 확인하세요.".to_string()),
            )
        })?;
    }
    fs::write(path, text).map_err(|e| {
        diag::build_diag(
            "E_REG_REPORT_WRITE",
            &format!("path={} {}", path.display(), e),
            None,
            Some("report 파일 쓰기 권한/경로를 확인하세요.".to_string()),
        )
    })
}

#[derive(Clone, Debug)]
pub struct AuditVerifyReport {
    audit_log_path: String,
    rows: usize,
    last_hash: Option<String>,
}

impl AuditVerifyReport {
    pub fn last_hash(&self) -> Option<&str> {
        self.last_hash.as_deref()
    }
}

pub fn run_audit_verify(audit_log_path: &Path) -> Result<AuditVerifyReport, String> {
    let text = fs::read_to_string(audit_log_path).map_err(|e| {
        diag::build_diag(
            "E_REG_AUDIT_READ",
            &format!("path={} {}", audit_log_path.display(), e),
            None,
            Some("감사로그 파일 경로/권한을 확인하세요.".to_string()),
        )
    })?;
    let mut rows = 0usize;
    let mut prev_hash: Option<String> = None;

    for (line_idx, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(line).map_err(|e| {
            diag::build_diag(
                "E_REG_AUDIT_PARSE",
                &format!(
                    "path={} line={} {}",
                    audit_log_path.display(),
                    line_idx + 1,
                    e
                ),
                Some("감사로그 줄 JSON 파싱 실패".to_string()),
                Some("손상된 줄을 수정하거나 감사로그를 다시 생성하세요.".to_string()),
            )
        })?;
        let body = row.get("body").ok_or_else(|| {
            diag::build_diag(
                "E_REG_AUDIT_ROW_FIELD",
                &format!(
                    "path={} line={} body 누락",
                    audit_log_path.display(),
                    line_idx + 1
                ),
                None,
                Some("각 줄에 body 필드를 포함하세요.".to_string()),
            )
        })?;
        let schema = body.get("schema").and_then(|v| v.as_str()).unwrap_or("");
        if schema != "ddn.registry.audit.v1" {
            return Err(diag::build_diag(
                "E_REG_AUDIT_SCHEMA",
                &format!(
                    "path={} line={} schema={} (need ddn.registry.audit.v1)",
                    audit_log_path.display(),
                    line_idx + 1,
                    schema
                ),
                None,
                Some("감사로그 body.schema를 ddn.registry.audit.v1로 맞추세요.".to_string()),
            ));
        }
        let row_hash = row
            .get("row_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                diag::build_diag(
                    "E_REG_AUDIT_ROW_FIELD",
                    &format!(
                        "path={} line={} row_hash 누락",
                        audit_log_path.display(),
                        line_idx + 1
                    ),
                    None,
                    Some("각 줄에 row_hash 필드를 포함하세요.".to_string()),
                )
            })?;
        let body_text = serde_json::to_string(body).map_err(|e| {
            diag::build_diag(
                "E_REG_JSON",
                &format!(
                    "audit verify body serialize failed path={} line={} {}",
                    audit_log_path.display(),
                    line_idx + 1,
                    e
                ),
                None,
                Some("감사로그 body JSON을 점검하세요.".to_string()),
            )
        })?;
        let expected_hash = format!("blake3:{}", blake3::hash(body_text.as_bytes()).to_hex());
        if row_hash != expected_hash {
            return Err(diag::build_diag(
                "E_REG_AUDIT_ROW_HASH_MISMATCH",
                &format!(
                    "path={} line={} expected={} actual={}",
                    audit_log_path.display(),
                    line_idx + 1,
                    expected_hash,
                    row_hash
                ),
                None,
                Some("해당 줄 body/row_hash를 재생성하세요.".to_string()),
            ));
        }

        let declared_prev = body
            .get("prev_hash")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        if rows == 0 {
            if declared_prev.is_some() {
                return Err(diag::build_diag(
                    "E_REG_AUDIT_CHAIN_BROKEN",
                    &format!(
                        "path={} line={} expected_prev=<none> actual_prev={}",
                        audit_log_path.display(),
                        line_idx + 1,
                        declared_prev.as_deref().unwrap_or("<none>")
                    ),
                    None,
                    Some("감사로그 체인을 처음 줄부터 다시 생성하세요.".to_string()),
                ));
            }
        } else if declared_prev.as_deref() != prev_hash.as_deref() {
            return Err(diag::build_diag(
                "E_REG_AUDIT_CHAIN_BROKEN",
                &format!(
                    "path={} line={} expected_prev={} actual_prev={}",
                    audit_log_path.display(),
                    line_idx + 1,
                    prev_hash.as_deref().unwrap_or("<none>"),
                    declared_prev.as_deref().unwrap_or("<none>")
                ),
                None,
                Some("감사로그 체인을 끊긴 지점부터 복구하거나 다시 생성하세요.".to_string()),
            ));
        }

        prev_hash = Some(row_hash.to_string());
        rows += 1;
    }

    if rows == 0 {
        return Err(diag::build_diag(
            "E_REG_AUDIT_EMPTY",
            &format!(
                "path={} 감사로그가 비어 있습니다.",
                audit_log_path.display()
            ),
            None,
            Some("감사 이벤트를 기록한 뒤 다시 검증하세요.".to_string()),
        ));
    }

    println!("audit_verify_ok=1");
    println!("audit_verify_rows={}", rows);
    println!(
        "audit_verify_last_hash={}",
        prev_hash.as_deref().unwrap_or("<none>")
    );
    Ok(AuditVerifyReport {
        audit_log_path: audit_log_path.display().to_string(),
        rows,
        last_hash: prev_hash,
    })
}

pub fn write_audit_verify_report(path: &Path, report: &AuditVerifyReport) -> Result<(), String> {
    let root = json!({
        "schema": "ddn.registry.audit_verify_report.v1",
        "ok": true,
        "audit_log_path": report.audit_log_path,
        "rows": report.rows,
        "last_hash": report.last_hash,
    });
    let text = serde_json::to_string_pretty(&root).map_err(|e| {
        diag::build_diag(
            "E_REG_JSON",
            &format!("audit verify report serialize failed: {}", e),
            None,
            Some("audit verify report payload를 점검하세요.".to_string()),
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            diag::build_diag(
                "E_REG_REPORT_WRITE",
                &format!("path={} {}", path.display(), e),
                None,
                Some("report 출력 경로를 확인하세요.".to_string()),
            )
        })?;
    }
    fs::write(path, text).map_err(|e| {
        diag::build_diag(
            "E_REG_REPORT_WRITE",
            &format!("path={} {}", path.display(), e),
            None,
            Some("report 파일 쓰기 권한/경로를 확인하세요.".to_string()),
        )
    })
}

fn ensure_expected_audit_last_hash(
    report: &AuditVerifyReport,
    expected: Option<&str>,
) -> Result<(), String> {
    let Some(expected_hash) = expected else {
        return Ok(());
    };
    let actual = report.last_hash().unwrap_or("<none>");
    if actual != expected_hash {
        return Err(diag::build_diag(
            "E_REG_AUDIT_LAST_HASH_MISMATCH",
            &format!("expected={} actual={}", expected_hash, actual),
            Some("audit log last_hash does not match expected pin".to_string()),
            Some("update expected hash to current audit last_hash".to_string()),
        ));
    }
    Ok(())
}

fn normalized_json_text(value: &Value) -> Result<String, String> {
    let normalized = normalize_json_value(value);
    serde_json::to_string(&normalized).map_err(|e| {
        diag::build_diag(
            "E_REG_JSON",
            &format!("normalized json serialize failed: {}", e),
            None,
            Some("의존성 JSON 구조를 점검하세요.".to_string()),
        )
    })
}

fn normalize_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for key in keys {
                let next = map.get(key).unwrap_or(&Value::Null);
                out.insert(key.clone(), normalize_json_value(next));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(normalize_json_value).collect()),
        _ => value.clone(),
    }
}

pub fn run_versions(
    index_path: &Path,
    scope: &str,
    name: &str,
    include_yanked: bool,
) -> Result<(), String> {
    run_versions_with_guard(
        index_path,
        scope,
        name,
        include_yanked,
        &ReadGuardOptions::default(),
    )
}

pub fn run_versions_with_guard(
    index_path: &Path,
    scope: &str,
    name: &str,
    include_yanked: bool,
    guard: &ReadGuardOptions,
) -> Result<(), String> {
    let entries = load_entries_with_guard(index_path, guard)?;
    let payload = build_versions_response(&entries, scope, name, include_yanked)?;
    let text = serde_json::to_string_pretty(&payload).map_err(|e| {
        diag::build_diag(
            "E_REG_JSON",
            &format!("versions response serialize failed: {}", e),
            None,
            Some("versions 응답 payload를 점검하세요.".to_string()),
        )
    })?;
    println!("{}", text);
    Ok(())
}

pub fn run_entry(index_path: &Path, scope: &str, name: &str, version: &str) -> Result<(), String> {
    run_entry_with_guard(
        index_path,
        scope,
        name,
        version,
        &ReadGuardOptions::default(),
    )
}

pub fn run_entry_with_guard(
    index_path: &Path,
    scope: &str,
    name: &str,
    version: &str,
    guard: &ReadGuardOptions,
) -> Result<(), String> {
    let entries = load_entries_with_guard(index_path, guard)?;
    let payload = build_entry_response(&entries, scope, name, version)?;
    let text = serde_json::to_string_pretty(&payload).map_err(|e| {
        diag::build_diag(
            "E_REG_JSON",
            &format!("entry response serialize failed: {}", e),
            None,
            Some("entry 응답 payload를 점검하세요.".to_string()),
        )
    })?;
    println!("{}", text);
    Ok(())
}

pub fn run_search(
    index_path: &Path,
    query: &str,
    limit: usize,
    include_yanked: bool,
) -> Result<(), String> {
    run_search_with_guard(
        index_path,
        query,
        limit,
        include_yanked,
        &ReadGuardOptions::default(),
    )
}

pub fn run_search_with_guard(
    index_path: &Path,
    query: &str,
    limit: usize,
    include_yanked: bool,
    guard: &ReadGuardOptions,
) -> Result<(), String> {
    let entries = load_entries_with_guard(index_path, guard)?;
    let payload = build_search_response(&entries, query, limit, include_yanked)?;
    let text = serde_json::to_string_pretty(&payload).map_err(|e| {
        diag::build_diag(
            "E_REG_JSON",
            &format!("search response serialize failed: {}", e),
            None,
            Some("search 응답 payload를 점검하세요.".to_string()),
        )
    })?;
    println!("{}", text);
    Ok(())
}

pub fn run_download(
    index_path: &Path,
    scope: &str,
    name: &str,
    version: &str,
    out: &Path,
    include_yanked: bool,
) -> Result<(), String> {
    run_download_with_options(
        index_path,
        scope,
        name,
        version,
        out,
        &ReadGuardOptions::default(),
        &DownloadOptions {
            include_yanked,
            ..DownloadOptions::default()
        },
    )
}

pub fn run_download_with_guard(
    index_path: &Path,
    scope: &str,
    name: &str,
    version: &str,
    out: &Path,
    include_yanked: bool,
    guard: &ReadGuardOptions,
) -> Result<(), String> {
    run_download_with_options(
        index_path,
        scope,
        name,
        version,
        out,
        guard,
        &DownloadOptions {
            include_yanked,
            ..DownloadOptions::default()
        },
    )
}

pub fn run_download_with_options(
    index_path: &Path,
    scope: &str,
    name: &str,
    version: &str,
    out: &Path,
    guard: &ReadGuardOptions,
    options: &DownloadOptions,
) -> Result<(), String> {
    let entries = load_entries_with_guard(index_path, guard)?;
    let Some(entry) = select_entry(entries.as_slice(), scope, name, version) else {
        return Err(diag::build_diag(
            "E_REG_INDEX_NOT_FOUND",
            &format!("scope={} name={} version={}", scope, name, version),
            None,
            Some("요청 version이 인덱스에 존재하는지 확인하세요.".to_string()),
        ));
    };
    if entry.yanked && !options.include_yanked {
        return Err(diag::build_diag(
            "E_REG_YANKED_LOCKED",
            &format!("scope={} name={} version={}", scope, name, version),
            None,
            Some("--include-yanked를 명시하거나 yanked가 아닌 버전을 고르세요.".to_string()),
        ));
    }
    let Some(download_url) = entry.download_url.as_deref() else {
        return Err(diag::build_diag(
            "E_REG_DOWNLOAD_URL_REQUIRED",
            &format!("scope={} name={} version={}", scope, name, version),
            None,
            Some("index entry에 download_url을 추가하세요.".to_string()),
        ));
    };
    let Some(expected_sha256) = entry.archive_sha256.as_deref() else {
        return Err(diag::build_diag(
            "E_REG_ARCHIVE_SHA256_REQUIRED",
            &format!("scope={} name={} version={}", scope, name, version),
            None,
            Some("index entry에 archive_sha256을 채우세요.".to_string()),
        ));
    };
    let (bytes, source_label) =
        fetch_download_bytes(index_path, download_url, expected_sha256, options)?;
    verify_archive_sha256(scope, name, version, expected_sha256, &bytes)?;

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            diag::build_diag(
                "E_REG_DOWNLOAD_WRITE",
                &format!("path={} {}", out.display(), e),
                None,
                Some("download 산출물 경로를 확인하세요.".to_string()),
            )
        })?;
    }
    fs::write(out, &bytes).map_err(|e| {
        diag::build_diag(
            "E_REG_DOWNLOAD_WRITE",
            &format!("path={} {}", out.display(), e),
            None,
            Some("download 산출물 파일 쓰기 권한/경로를 확인하세요.".to_string()),
        )
    })?;

    let actual_sha256 = sha256_hex_prefixed(&bytes);
    println!("registry_download_ok={}/{}@{}", scope, name, version);
    println!("registry_download_source={}", source_label);
    println!("registry_download_out={}", out.display());
    println!("registry_download_archive_sha256={}", actual_sha256);
    Ok(())
}

#[derive(Clone, Debug)]
pub struct PublishOptions {
    pub audit_log: Option<String>,
    pub scope: String,
    pub name: String,
    pub version: String,
    pub archive_sha256: String,
    pub contract: Option<String>,
    pub detmath_seal_hash: Option<String>,
    pub min_runtime: Option<String>,
    pub download_url: Option<String>,
    pub summary: Option<String>,
    pub token: String,
    pub role: String,
    pub at: Option<String>,
}

#[derive(Clone, Debug)]
pub struct YankOptions {
    pub audit_log: Option<String>,
    pub scope: String,
    pub name: String,
    pub version: String,
    pub reason_code: String,
    pub note: Option<String>,
    pub token: String,
    pub role: String,
    pub at: Option<String>,
}

pub fn run_publish(index_path: &Path, options: &PublishOptions) -> Result<(), String> {
    run_publish_with_auth_policy(index_path, options, None)
}

pub fn run_publish_with_auth_policy(
    index_path: &Path,
    options: &PublishOptions,
    auth_policy: Option<&Path>,
) -> Result<(), String> {
    let package_id = format!("{}/{}@{}", options.scope, options.name, options.version);
    if let Some(path) = auth_policy {
        if let Err(err) = validate_auth_policy(path, &options.token, &options.role, &options.scope)
        {
            append_audit(
                index_path,
                options.audit_log.as_deref(),
                "publish",
                &package_id,
                &options.role,
                &options.token,
                false,
                Some(error_code_from(&err)),
                options.at.as_deref(),
            )?;
            return Err(err);
        }
    }
    if let Err(err) = ensure_role(
        &options.token,
        &options.role,
        &["publisher", "scope_admin", "registry_admin"],
    ) {
        append_audit(
            index_path,
            options.audit_log.as_deref(),
            "publish",
            &package_id,
            &options.role,
            &options.token,
            false,
            Some(error_code_from(&err)),
            options.at.as_deref(),
        )?;
        return Err(err);
    }
    let (mut entries, mut meta) = if index_path.exists() {
        load_entries_and_meta(index_path)?
    } else {
        (Vec::new(), SnapshotMeta::default())
    };
    if entries
        .iter()
        .any(|e| e.scope == options.scope && e.name == options.name && e.version == options.version)
    {
        append_audit(
            index_path,
            options.audit_log.as_deref(),
            "publish",
            &format!("{}/{}@{}", options.scope, options.name, options.version),
            &options.role,
            &options.token,
            false,
            Some("E_REG_IMMUTABLE_EXISTS"),
            options.at.as_deref(),
        )?;
        return Err(diag::build_diag(
            "E_REG_IMMUTABLE_EXISTS",
            &format!(
                "scope={} name={} version={}",
                options.scope, options.name, options.version
            ),
            None,
            Some("기존 버전을 수정하지 말고 새 버전을 발행하세요.".to_string()),
        ));
    }
    entries.push(RegistryEntry {
        scope: options.scope.clone(),
        name: options.name.clone(),
        version: options.version.clone(),
        archive_sha256: Some(options.archive_sha256.clone()),
        contract: options.contract.clone(),
        detmath_seal_hash: options.detmath_seal_hash.clone(),
        min_runtime: options.min_runtime.clone(),
        dependencies: Some(json!({})),
        download_url: options.download_url.clone(),
        published_at: Some(now_or(options.at.as_deref())),
        summary: options.summary.clone(),
        yanked: false,
        yanked_at: None,
        yank_reason_code: None,
        yank_note: None,
    });
    meta.snapshot_id = Some(now_or(options.at.as_deref()));
    meta.index_root_hash = None;
    write_snapshot_with_meta(index_path, &entries, &meta)?;
    append_audit(
        index_path,
        options.audit_log.as_deref(),
        "publish",
        &format!("{}/{}@{}", options.scope, options.name, options.version),
        &options.role,
        &options.token,
        true,
        None,
        options.at.as_deref(),
    )?;
    println!(
        "registry_publish_ok={}/{}@{}",
        options.scope, options.name, options.version
    );
    Ok(())
}

pub fn run_yank(index_path: &Path, options: &YankOptions) -> Result<(), String> {
    run_yank_with_auth_policy(index_path, options, None)
}

pub fn run_yank_with_auth_policy(
    index_path: &Path,
    options: &YankOptions,
    auth_policy: Option<&Path>,
) -> Result<(), String> {
    let package_id = format!("{}/{}@{}", options.scope, options.name, options.version);
    if let Some(path) = auth_policy {
        if let Err(err) = validate_auth_policy(path, &options.token, &options.role, &options.scope)
        {
            append_audit(
                index_path,
                options.audit_log.as_deref(),
                "yank",
                &package_id,
                &options.role,
                &options.token,
                false,
                Some(error_code_from(&err)),
                options.at.as_deref(),
            )?;
            return Err(err);
        }
    }
    if let Err(err) = ensure_role(
        &options.token,
        &options.role,
        &["scope_admin", "registry_admin"],
    ) {
        append_audit(
            index_path,
            options.audit_log.as_deref(),
            "yank",
            &package_id,
            &options.role,
            &options.token,
            false,
            Some(error_code_from(&err)),
            options.at.as_deref(),
        )?;
        return Err(err);
    }
    let (mut entries, mut meta) = load_entries_and_meta(index_path)?;
    let mut found = false;
    for entry in &mut entries {
        if entry.scope == options.scope
            && entry.name == options.name
            && entry.version == options.version
        {
            entry.yanked = true;
            entry.yanked_at = Some(now_or(options.at.as_deref()));
            entry.yank_reason_code = Some(options.reason_code.clone());
            entry.yank_note = options.note.clone();
            found = true;
            break;
        }
    }
    if !found {
        append_audit(
            index_path,
            options.audit_log.as_deref(),
            "yank",
            &format!("{}/{}@{}", options.scope, options.name, options.version),
            &options.role,
            &options.token,
            false,
            Some("E_REG_INDEX_NOT_FOUND"),
            options.at.as_deref(),
        )?;
        return Err(diag::build_diag(
            "E_REG_INDEX_NOT_FOUND",
            &format!(
                "scope={} name={} version={}",
                options.scope, options.name, options.version
            ),
            None,
            Some("yank 대상 scope/name/version이 인덱스에 있는지 확인하세요.".to_string()),
        ));
    }
    meta.snapshot_id = Some(now_or(options.at.as_deref()));
    meta.index_root_hash = None;
    write_snapshot_with_meta(index_path, &entries, &meta)?;
    append_audit(
        index_path,
        options.audit_log.as_deref(),
        "yank",
        &format!("{}/{}@{}", options.scope, options.name, options.version),
        &options.role,
        &options.token,
        true,
        None,
        options.at.as_deref(),
    )?;
    println!(
        "registry_yank_ok={}/{}@{}",
        options.scope, options.name, options.version
    );
    Ok(())
}

fn load_entries(index_path: &Path) -> Result<Vec<RegistryEntry>, String> {
    load_entries_with_guard(index_path, &ReadGuardOptions::default())
}

fn load_entries_and_meta(index_path: &Path) -> Result<(Vec<RegistryEntry>, SnapshotMeta), String> {
    let text = fs::read_to_string(index_path).map_err(|e| {
        diag::build_diag(
            "E_REG_INDEX_READ",
            &format!("path={} {}", index_path.display(), e),
            None,
            Some("registry index 파일 경로/권한을 확인하세요.".to_string()),
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|e| {
        diag::build_diag(
            "E_REG_INDEX_PARSE",
            &format!("path={} {}", index_path.display(), e),
            Some("registry index JSON 파싱 실패".to_string()),
            Some("registry index JSON을 정정하세요.".to_string()),
        )
    })?;
    let entries = load_entries_from_value(&value)?;
    let meta = snapshot_meta_from_root(&value);
    Ok((entries, meta))
}

fn load_entries_with_guard(
    index_path: &Path,
    guard: &ReadGuardOptions,
) -> Result<Vec<RegistryEntry>, String> {
    let text = fs::read_to_string(index_path).map_err(|e| {
        diag::build_diag(
            "E_REG_INDEX_READ",
            &format!("path={} {}", index_path.display(), e),
            None,
            Some("registry index 파일 경로/권한을 확인하세요.".to_string()),
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|e| {
        diag::build_diag(
            "E_REG_INDEX_PARSE",
            &format!("path={} {}", index_path.display(), e),
            Some("registry index JSON 파싱 실패".to_string()),
            Some("registry index JSON을 정정하세요.".to_string()),
        )
    })?;
    validate_index_read_guard(&value, guard)?;
    load_entries_from_value(&value)
}

fn snapshot_meta_from_root(root: &Value) -> SnapshotMeta {
    let snapshot = root.get("registry_snapshot");
    let trust_root = root.get("trust_root");
    SnapshotMeta {
        snapshot_id: root
            .get("snapshot_id")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .or_else(|| {
                snapshot
                    .and_then(|v| v.get("snapshot_id"))
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
            }),
        index_root_hash: root
            .get("index_root_hash")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .or_else(|| {
                snapshot
                    .and_then(|v| v.get("index_root_hash"))
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
            }),
        trust_root_hash: trust_root
            .and_then(|v| v.get("hash"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        trust_root_source: trust_root
            .and_then(|v| v.get("source"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
    }
}

fn validate_index_read_guard(root: &Value, guard: &ReadGuardOptions) -> Result<(), String> {
    if !guard.is_enabled() {
        return Ok(());
    }

    let snapshot = root.get("registry_snapshot");
    let snapshot_id = root
        .get("snapshot_id")
        .and_then(|v| v.as_str())
        .or_else(|| {
            snapshot
                .and_then(|v| v.get("snapshot_id"))
                .and_then(|v| v.as_str())
        });
    let index_root_hash = root
        .get("index_root_hash")
        .and_then(|v| v.as_str())
        .or_else(|| {
            snapshot
                .and_then(|v| v.get("index_root_hash"))
                .and_then(|v| v.as_str())
        });

    if guard.frozen_lockfile && (snapshot_id.is_none() || index_root_hash.is_none()) {
        return Err(diag::build_diag(
            "E_REG_SNAPSHOT_MISSING",
            "frozen-lockfile requires registry_snapshot(snapshot_id/index_root_hash)",
            None,
            Some("index에 snapshot_id/index_root_hash를 포함시키세요.".to_string()),
        ));
    }

    if let Some(expected) = guard.expect_snapshot_id.as_deref() {
        let Some(actual) = snapshot_id else {
            return Err(diag::build_diag(
                "E_REG_SNAPSHOT_MISSING",
                "registry_snapshot.snapshot_id is missing",
                None,
                Some("index snapshot_id를 채우세요.".to_string()),
            ));
        };
        if actual != expected {
            return Err(diag::build_diag(
                "E_REG_SNAPSHOT_MISMATCH",
                &format!("expected={} actual={}", expected, actual),
                None,
                Some("요구 snapshot_id와 index snapshot_id를 일치시키세요.".to_string()),
            ));
        }
    }

    if let Some(expected) = guard.expect_index_root_hash.as_deref() {
        let Some(actual) = index_root_hash else {
            return Err(diag::build_diag(
                "E_REG_INDEX_ROOT_HASH_MISMATCH",
                "expected=<given> actual=<missing>",
                None,
                Some("index_root_hash를 index에 포함시키세요.".to_string()),
            ));
        };
        if actual != expected {
            return Err(diag::build_diag(
                "E_REG_INDEX_ROOT_HASH_MISMATCH",
                &format!("expected={} actual={}", expected, actual),
                None,
                Some("요구 index_root_hash와 실제 값을 일치시키세요.".to_string()),
            ));
        }
    }

    let trust_root_hash = root
        .get("trust_root")
        .and_then(|v| v.get("hash"))
        .and_then(|v| v.as_str());

    if guard.require_trust_root && trust_root_hash.is_none() {
        return Err(diag::build_diag(
            "E_REG_TRUST_ROOT_INVALID",
            "trust_root.hash is missing",
            None,
            Some("index에 trust_root.hash를 포함시키세요.".to_string()),
        ));
    }
    if let Some(expected) = guard.expect_trust_root_hash.as_deref() {
        let Some(actual) = trust_root_hash else {
            return Err(diag::build_diag(
                "E_REG_TRUST_ROOT_INVALID",
                "trust_root.hash is missing",
                None,
                Some("index에 trust_root.hash를 포함시키세요.".to_string()),
            ));
        };
        if actual != expected {
            return Err(diag::build_diag(
                "E_REG_TRUST_ROOT_INVALID",
                &format!("expected={} actual={}", expected, actual),
                None,
                Some("요구 trust_root_hash와 index trust_root.hash를 일치시키세요.".to_string()),
            ));
        }
    }

    Ok(())
}

fn load_entries_from_value(root: &Value) -> Result<Vec<RegistryEntry>, String> {
    let schema = root.get("schema").and_then(|v| v.as_str()).unwrap_or("");
    match schema {
        "ddn.registry.snapshot.v1" => {
            let rows = root
                .get("entries")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    diag::build_diag(
                        "E_REG_INDEX_SCHEMA",
                        "entries 배열이 없습니다.",
                        None,
                        Some("snapshot 인덱스에 entries[]를 추가하세요.".to_string()),
                    )
                })?;
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                out.push(parse_index_entry(row)?);
            }
            Ok(out)
        }
        "ddn.registry.package_versions.v1" => {
            let scope = req_str(root, "scope")?;
            let name = req_str(root, "name")?;
            let rows = root
                .get("versions")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    diag::build_diag(
                        "E_REG_INDEX_SCHEMA",
                        "versions 배열이 없습니다.",
                        None,
                        Some("package_versions 인덱스에 versions[]를 추가하세요.".to_string()),
                    )
                })?;
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                out.push(parse_version_entry(row, scope, name)?);
            }
            Ok(out)
        }
        "ddn.registry.index_entry.v1" => Ok(vec![parse_index_entry(root)?]),
        _ => {
            if let Some(rows) = root.get("entries").and_then(|v| v.as_array()) {
                let mut out = Vec::with_capacity(rows.len());
                for row in rows {
                    out.push(parse_index_entry(row)?);
                }
                return Ok(out);
            }
            Err(diag::build_diag(
                "E_REG_INDEX_SCHEMA",
                &format!(
                    "schema={} (need ddn.registry.snapshot.v1|ddn.registry.package_versions.v1|ddn.registry.index_entry.v1)",
                    schema
                ),
                None,
                Some("인덱스 schema를 지원 형식으로 맞추세요.".to_string()),
            ))
        }
    }
}

fn parse_index_entry(value: &Value) -> Result<RegistryEntry, String> {
    Ok(RegistryEntry {
        scope: req_str(value, "scope")?.to_string(),
        name: req_str(value, "name")?.to_string(),
        version: req_str(value, "version")?.to_string(),
        archive_sha256: opt_str(value, "archive_sha256"),
        contract: opt_str(value, "contract"),
        detmath_seal_hash: opt_str(value, "detmath_seal_hash"),
        min_runtime: opt_str(value, "min_runtime"),
        dependencies: value.get("dependencies").cloned(),
        download_url: opt_str(value, "download_url"),
        published_at: opt_str(value, "published_at"),
        summary: opt_str(value, "summary").or_else(|| opt_str(value, "description")),
        yanked: value
            .get("yanked")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        yanked_at: opt_str(value, "yanked_at"),
        yank_reason_code: opt_str(value, "yank_reason_code"),
        yank_note: opt_str(value, "yank_note"),
    })
}

fn parse_version_entry(value: &Value, scope: &str, name: &str) -> Result<RegistryEntry, String> {
    Ok(RegistryEntry {
        scope: scope.to_string(),
        name: name.to_string(),
        version: req_str(value, "version")?.to_string(),
        archive_sha256: opt_str(value, "archive_sha256"),
        contract: opt_str(value, "contract"),
        detmath_seal_hash: opt_str(value, "detmath_seal_hash"),
        min_runtime: opt_str(value, "min_runtime"),
        dependencies: value.get("dependencies").cloned(),
        download_url: opt_str(value, "download_url"),
        published_at: opt_str(value, "published_at"),
        summary: opt_str(value, "summary").or_else(|| opt_str(value, "description")),
        yanked: value
            .get("yanked")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        yanked_at: opt_str(value, "yanked_at"),
        yank_reason_code: opt_str(value, "yank_reason_code"),
        yank_note: opt_str(value, "yank_note"),
    })
}

fn build_versions_response(
    entries: &[RegistryEntry],
    scope: &str,
    name: &str,
    include_yanked: bool,
) -> Result<Value, String> {
    let mut by_version: BTreeMap<String, &RegistryEntry> = BTreeMap::new();
    for entry in entries
        .iter()
        .filter(|e| e.scope == scope && e.name == name)
        .filter(|e| include_yanked || !e.yanked)
    {
        match by_version.get(&entry.version) {
            Some(prev) => {
                if duplicate_entry_rank(entry) < duplicate_entry_rank(prev) {
                    by_version.insert(entry.version.clone(), entry);
                }
            }
            None => {
                by_version.insert(entry.version.clone(), entry);
            }
        }
    }
    let mut versions: Vec<&RegistryEntry> = by_version.values().copied().collect();
    versions.sort_by(|a, b| compare_versions_desc(&a.version, &b.version));
    if versions.is_empty() {
        return Err(diag::build_diag(
            "E_REG_INDEX_NOT_FOUND",
            &format!(
                "scope={} name={} include_yanked={}",
                scope,
                name,
                if include_yanked { 1 } else { 0 }
            ),
            None,
            Some(
                "해당 scope/name이 인덱스에 있는지 확인하거나 include-yanked 조건을 조정하세요."
                    .to_string(),
            ),
        ));
    }
    let rows: Vec<Value> = versions.iter().map(|entry| version_json(entry)).collect();
    Ok(json!({
        "schema": "ddn.registry.package_versions.v1",
        "scope": scope,
        "name": name,
        "versions": rows
    }))
}

fn build_entry_response(
    entries: &[RegistryEntry],
    scope: &str,
    name: &str,
    version: &str,
) -> Result<Value, String> {
    let Some(entry) = select_entry(entries, scope, name, version) else {
        return Err(diag::build_diag(
            "E_REG_INDEX_NOT_FOUND",
            &format!("scope={} name={} version={}", scope, name, version),
            None,
            Some("요청 version이 인덱스에 존재하는지 확인하세요.".to_string()),
        ));
    };
    Ok(index_entry_json(entry))
}

fn select_entry<'a>(
    entries: &'a [RegistryEntry],
    scope: &str,
    name: &str,
    version: &str,
) -> Option<&'a RegistryEntry> {
    entries
        .iter()
        .filter(|e| e.scope == scope && e.name == name && e.version == version)
        .min_by_key(|e| duplicate_entry_rank(e))
}

#[derive(Clone, Debug)]
enum DownloadSource {
    Local(PathBuf),
    Http(String),
}

fn fetch_download_bytes(
    index_path: &Path,
    download_url: &str,
    expected_sha256: &str,
    options: &DownloadOptions,
) -> Result<(Vec<u8>, String), String> {
    let cache_blob = match options.cache_dir.as_deref() {
        Some(cache_dir) => Some(cache_blob_path(cache_dir, expected_sha256)?),
        None => None,
    };

    if let Some(cache_path) = cache_blob.as_deref() {
        if cache_path.exists() {
            let bytes = fs::read(cache_path).map_err(|e| {
                diag::build_diag(
                    "E_REG_CACHE_READ",
                    &format!("cache_path={} {}", cache_path.display(), e),
                    None,
                    Some("cache 파일 권한/경로를 확인하세요.".to_string()),
                )
            })?;
            verify_expected_archive_sha256(expected_sha256, &bytes, "cache read")?;
            return Ok((bytes, format!("cache://{}", cache_path.display())));
        }
    }

    if options.offline {
        let detail = if let Some(cache_path) = cache_blob.as_deref() {
            format!(
                "download_url={} cache_path={} (miss)",
                download_url,
                cache_path.display()
            )
        } else {
            format!("download_url={} cache_dir=(none)", download_url)
        };
        return Err(diag::build_diag(
            "E_CACHE_UNAVAILABLE_OFFLINE",
            &detail,
            None,
            Some("offline 실행에는 --cache-dir <path>로 사전 캐시를 준비하세요.".to_string()),
        ));
    }

    let source = resolve_download_source(index_path, download_url)?;
    let (bytes, source_label) = match source {
        DownloadSource::Local(path) => {
            let bytes = fs::read(&path).map_err(|e| {
                diag::build_diag(
                    "E_REG_DOWNLOAD_READ",
                    &format!("download_url={} path={} {}", download_url, path.display(), e),
                    None,
                    Some("download_url 파일 경로/권한을 확인하세요.".to_string()),
                )
            })?;
            (bytes, format!("file://{}", path.display()))
        }
        DownloadSource::Http(url) => {
            if !options.allow_http {
                return Err(diag::build_diag(
                    "E_REG_DOWNLOAD_HTTP_DISABLED",
                    &format!("download_url={}", url),
                    None,
                    Some("HTTP(S) download를 허용하려면 --allow-http를 지정하세요.".to_string()),
                ));
            }
            let bytes = fetch_http_bytes(&url)?;
            (bytes, url)
        }
    };

    verify_expected_archive_sha256(expected_sha256, &bytes, "download fetch")?;
    if let Some(cache_path) = cache_blob.as_deref() {
        write_cache_blob(cache_path, &bytes)?;
    }
    Ok((bytes, source_label))
}

fn resolve_download_source(index_path: &Path, download_url: &str) -> Result<DownloadSource, String> {
    if let Some(raw) = download_url.strip_prefix("file://") {
        return Ok(DownloadSource::Local(resolve_local_download_path(
            index_path,
            normalize_file_url_path(raw),
        )));
    }
    if let Some((scheme, _)) = download_url.split_once("://") {
        if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") {
            return Ok(DownloadSource::Http(download_url.to_string()));
        }
        return Err(diag::build_diag(
            "E_REG_DOWNLOAD_URL_SCHEME",
            &format!("download_url={}", download_url),
            None,
            Some("download_url은 file://, 로컬 경로, http(s)://만 지원합니다.".to_string()),
        ));
    }
    Ok(DownloadSource::Local(resolve_local_download_path(
        index_path,
        download_url,
    )))
}

fn resolve_local_download_path(index_path: &Path, raw: &str) -> PathBuf {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        candidate
    } else {
        index_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(candidate)
    }
}

fn fetch_http_bytes(download_url: &str) -> Result<Vec<u8>, String> {
    let response = ureq::get(download_url).call().map_err(|e| {
        diag::build_diag(
            "E_REG_DOWNLOAD_HTTP_FAILED",
            &format!("download_url={} {}", download_url, e),
            None,
            Some("HTTP endpoint/네트워크 상태를 확인하세요.".to_string()),
        )
    })?;
    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(|e| {
        diag::build_diag(
            "E_REG_DOWNLOAD_HTTP_FAILED",
            &format!("download_url={} {}", download_url, e),
            None,
            Some("HTTP response read 실패 원인을 확인하세요.".to_string()),
        )
    })?;
    Ok(bytes)
}

fn write_cache_blob(cache_path: &Path, bytes: &[u8]) -> Result<(), String> {
    if cache_path.exists() {
        return Ok(());
    }
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            diag::build_diag(
                "E_REG_CACHE_WRITE",
                &format!("cache_path={} {}", cache_path.display(), e),
                None,
                Some("cache 디렉터리 생성 권한/경로를 확인하세요.".to_string()),
            )
        })?;
    }
    fs::write(cache_path, bytes).map_err(|e| {
        diag::build_diag(
            "E_REG_CACHE_WRITE",
            &format!("cache_path={} {}", cache_path.display(), e),
            None,
            Some("cache 파일 쓰기 권한/디스크 공간을 확인하세요.".to_string()),
        )
    })
}

fn cache_blob_path(cache_dir: &Path, expected_sha256: &str) -> Result<PathBuf, String> {
    let Some(hex) = expected_sha256.strip_prefix("sha256:") else {
        return Err(diag::build_diag(
            "E_REG_ARCHIVE_SHA256_INVALID",
            &format!("archive_sha256={}", expected_sha256),
            None,
            Some("archive_sha256 형식을 sha256:<64hex>로 맞추세요.".to_string()),
        ));
    };
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(diag::build_diag(
            "E_REG_ARCHIVE_SHA256_INVALID",
            &format!("archive_sha256={}", expected_sha256),
            None,
            Some("archive_sha256 형식을 sha256:<64hex>로 맞추세요.".to_string()),
        ));
    }
    Ok(cache_dir
        .join("blobs")
        .join("sha256")
        .join(hex.to_ascii_lowercase()))
}

fn verify_expected_archive_sha256(
    expected_sha256: &str,
    bytes: &[u8],
    source: &str,
) -> Result<(), String> {
    let actual_sha256 = sha256_hex_prefixed(bytes);
    if actual_sha256 != expected_sha256 {
        return Err(diag::build_diag(
            "E_REG_ARCHIVE_SHA256_MISMATCH",
            &format!(
                "expected={} actual={} source={}",
                expected_sha256, actual_sha256, source
            ),
            None,
            Some("registry index의 archive_sha256 또는 archive 내용을 다시 맞추세요.".to_string()),
        ));
    }
    Ok(())
}

fn normalize_file_url_path(raw: &str) -> &str {
    let bytes = raw.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'/' && bytes[2] == b':' {
        &raw[1..]
    } else {
        raw
    }
}

fn verify_archive_sha256(
    scope: &str,
    name: &str,
    version: &str,
    expected_sha256: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let actual_sha256 = sha256_hex_prefixed(bytes);
    if actual_sha256 != expected_sha256 {
        return Err(diag::build_diag(
            "E_REG_ARCHIVE_SHA256_MISMATCH",
            &format!(
                "scope={} name={} version={} expected={} actual={}",
                scope, name, version, expected_sha256, actual_sha256
            ),
            None,
            Some("registry index의 archive_sha256 또는 archive 내용을 다시 맞추세요.".to_string()),
        ));
    }
    Ok(())
}

fn sha256_hex_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", hex::encode(digest))
}

fn build_search_response(
    entries: &[RegistryEntry],
    query: &str,
    limit: usize,
    include_yanked: bool,
) -> Result<Value, String> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Err(diag::build_diag(
            "E_REG_SEARCH_QUERY",
            "query가 비어 있습니다.",
            None,
            Some("--query <text>를 지정하세요.".to_string()),
        ));
    }
    let mut latest: BTreeMap<(String, String), &RegistryEntry> = BTreeMap::new();
    for entry in entries
        .iter()
        .filter(|e| include_yanked || !e.yanked)
        .filter(|e| {
            format!("{}/{}", e.scope, e.name)
                .to_lowercase()
                .contains(&q)
        })
    {
        let key = (entry.scope.clone(), entry.name.clone());
        match latest.get(&key) {
            Some(prev) => match compare_versions_desc(&entry.version, &prev.version) {
                Ordering::Greater => {
                    continue;
                }
                Ordering::Equal => {
                    if duplicate_entry_rank(entry) < duplicate_entry_rank(prev) {
                        latest.insert(key, entry);
                    }
                }
                Ordering::Less => {
                    latest.insert(key, entry);
                }
            },
            None => {
                latest.insert(key, entry);
            }
        }
    }

    let mut rows: Vec<&RegistryEntry> = latest.values().copied().collect();
    rows.sort_by(|a, b| {
        a.scope
            .cmp(&b.scope)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| compare_versions_desc(&a.version, &b.version))
    });
    if limit > 0 && rows.len() > limit {
        rows.truncate(limit);
    }
    let items: Vec<Value> = rows
        .iter()
        .map(|entry| {
            json!({
                "scope": entry.scope,
                "name": entry.name,
                "latest_version": entry.version,
                "contract": entry.contract,
                "summary": entry.summary.clone().unwrap_or_else(|| format!("{}/{}", entry.scope, entry.name)),
                "yanked": entry.yanked
            })
        })
        .collect();
    Ok(json!({
        "schema": "ddn.registry.search_result.v1",
        "items": items
    }))
}

fn version_json(entry: &RegistryEntry) -> Value {
    json!({
        "version": entry.version,
        "archive_sha256": entry.archive_sha256,
        "contract": entry.contract,
        "detmath_seal_hash": entry.detmath_seal_hash,
        "min_runtime": entry.min_runtime,
        "dependencies": entry.dependencies,
        "download_url": entry.download_url,
        "published_at": entry.published_at,
        "yanked": entry.yanked,
        "yanked_at": entry.yanked_at,
        "yank_reason_code": entry.yank_reason_code,
        "yank_note": entry.yank_note
    })
}

fn index_entry_json(entry: &RegistryEntry) -> Value {
    json!({
        "schema": "ddn.registry.index_entry.v1",
        "scope": entry.scope,
        "name": entry.name,
        "version": entry.version,
        "archive_sha256": entry.archive_sha256,
        "contract": entry.contract,
        "detmath_seal_hash": entry.detmath_seal_hash,
        "min_runtime": entry.min_runtime,
        "dependencies": entry.dependencies,
        "download_url": entry.download_url,
        "published_at": entry.published_at,
        "summary": entry.summary,
        "yanked": entry.yanked,
        "yanked_at": entry.yanked_at,
        "yank_reason_code": entry.yank_reason_code,
        "yank_note": entry.yank_note
    })
}

fn duplicate_entry_rank(entry: &RegistryEntry) -> (bool, String) {
    // Full normalized JSON key makes duplicate resolution stable even when optional fields differ.
    let normalized = normalize_json_value(&index_entry_json(entry));
    let key = serde_json::to_string(&normalized).unwrap_or_else(|_| {
        format!(
            "{}|{}|{}|{}",
            entry.scope, entry.name, entry.version, entry.yanked
        )
    });
    (entry.yanked, key)
}

fn verify_pin_match_score(entry: &RegistryEntry, pkg: &LockVerifyPackage) -> usize {
    let mut score = 0usize;
    if let Some(expected) = pkg.archive_sha256.as_deref() {
        if entry.archive_sha256.as_deref() == Some(expected) {
            score += 1;
        }
    }
    if let Some(expected) = pkg.download_url.as_deref() {
        if entry.download_url.as_deref() == Some(expected) {
            score += 1;
        }
    }
    if let Some(expected) = pkg.dependencies.as_ref() {
        let actual = entry.dependencies.as_ref().unwrap_or(&Value::Null);
        if let (Ok(a), Ok(b)) = (normalized_json_text(expected), normalized_json_text(actual)) {
            if a == b {
                score += 1;
            }
        }
    }
    if let Some(expected) = pkg.contract.as_deref() {
        if entry.contract.as_deref() == Some(expected) {
            score += 1;
        }
    }
    if let Some(expected) = pkg.min_runtime.as_deref() {
        if entry.min_runtime.as_deref() == Some(expected) {
            score += 1;
        }
    }
    if let Some(expected) = pkg.detmath_seal_hash.as_deref() {
        if entry.detmath_seal_hash.as_deref() == Some(expected) {
            score += 1;
        }
    }
    score
}

fn verify_duplicate_entry_rank(
    entry: &RegistryEntry,
    pkg: &LockVerifyPackage,
) -> (bool, Reverse<usize>, String) {
    let (_, tie_key) = duplicate_entry_rank(entry);
    (
        entry.yanked,
        Reverse(verify_pin_match_score(entry, pkg)),
        tie_key,
    )
}

fn write_snapshot(index_path: &Path, entries: &[RegistryEntry]) -> Result<(), String> {
    write_snapshot_with_meta(index_path, entries, &SnapshotMeta::default())
}

fn write_snapshot_with_meta(
    index_path: &Path,
    entries: &[RegistryEntry],
    meta: &SnapshotMeta,
) -> Result<(), String> {
    let mut rows: Vec<Value> = entries.iter().map(index_entry_json).collect();
    rows.sort_by(|a, b| {
        let ascope = a.get("scope").and_then(|v| v.as_str()).unwrap_or("");
        let bscope = b.get("scope").and_then(|v| v.as_str()).unwrap_or("");
        let aname = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let bname = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let aver = a.get("version").and_then(|v| v.as_str()).unwrap_or("");
        let bver = b.get("version").and_then(|v| v.as_str()).unwrap_or("");
        ascope
            .cmp(bscope)
            .then_with(|| aname.cmp(bname))
            .then_with(|| compare_versions_desc(aver, bver))
    });
    let mut root = json!({
        "schema": "ddn.registry.snapshot.v1",
        "entries": rows
    });

    let snapshot_id = meta.snapshot_id.clone().unwrap_or_else(|| now_or(None));
    root["snapshot_id"] = Value::String(snapshot_id);

    let index_root_hash = match meta.index_root_hash.as_deref() {
        Some(value) => value.to_string(),
        None => {
            let entries_json = root.get("entries").cloned().unwrap_or_else(|| json!([]));
            let bytes = serde_json::to_vec(&entries_json).map_err(|e| {
                diag::build_diag(
                    "E_REG_JSON",
                    &format!("entries hash serialize failed: {}", e),
                    None,
                    Some("entries payload를 점검하세요.".to_string()),
                )
            })?;
            format!("blake3:{}", blake3::hash(&bytes).to_hex())
        }
    };
    root["index_root_hash"] = Value::String(index_root_hash);

    if let Some(hash) = meta.trust_root_hash.as_deref() {
        root["trust_root"] = json!({
            "hash": hash,
            "source": meta
                .trust_root_source
                .as_deref()
                .unwrap_or("registry")
        });
    }

    let text = serde_json::to_string_pretty(&root).map_err(|e| {
        diag::build_diag(
            "E_REG_JSON",
            &format!("index serialize failed: {}", e),
            None,
            Some("snapshot payload를 점검하세요.".to_string()),
        )
    })?;
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            diag::build_diag(
                "E_REG_INDEX_WRITE",
                &format!("path={} {}", index_path.display(), e),
                None,
                Some("index 출력 경로를 확인하세요.".to_string()),
            )
        })?;
    }
    fs::write(index_path, text).map_err(|e| {
        diag::build_diag(
            "E_REG_INDEX_WRITE",
            &format!("path={} {}", index_path.display(), e),
            None,
            Some("index 파일 쓰기 권한/경로를 확인하세요.".to_string()),
        )
    })?;
    Ok(())
}

fn error_code_from(err: &str) -> &str {
    err.split_whitespace().next().unwrap_or("E_REG_UNKNOWN")
}

fn validate_auth_policy(path: &Path, token: &str, role: &str, scope: &str) -> Result<(), String> {
    if token.trim().is_empty() {
        return Err(diag::build_diag(
            "E_REG_AUTH_REQUIRED",
            "token이 필요합니다.",
            None,
            Some("--token <value>를 지정하세요.".to_string()),
        ));
    }
    let text = fs::read_to_string(path).map_err(|e| {
        diag::build_diag(
            "E_REG_AUTH_POLICY_READ",
            &format!("path={} {}", path.display(), e),
            None,
            Some("auth policy 파일 경로/권한을 확인하세요.".to_string()),
        )
    })?;
    let root: Value = serde_json::from_str(&text).map_err(|e| {
        diag::build_diag(
            "E_REG_AUTH_POLICY_PARSE",
            &format!("path={} {}", path.display(), e),
            Some("auth policy JSON 파싱 실패".to_string()),
            Some("auth policy JSON을 정정하세요.".to_string()),
        )
    })?;
    let schema = root.get("schema").and_then(|v| v.as_str()).unwrap_or("");
    if schema != "ddn.registry.auth_policy.v1" {
        return Err(diag::build_diag(
            "E_REG_AUTH_POLICY_SCHEMA",
            &format!("schema={} (need ddn.registry.auth_policy.v1)", schema),
            None,
            Some("auth policy schema를 ddn.registry.auth_policy.v1로 맞추세요.".to_string()),
        ));
    }
    let rows = root
        .get("tokens")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            diag::build_diag(
                "E_REG_AUTH_POLICY_SCHEMA",
                "tokens 배열이 필요합니다.",
                None,
                Some("auth policy에 tokens[]를 추가하세요.".to_string()),
            )
        })?;
    let token_hash = format!("blake3:{}", blake3::hash(token.as_bytes()).to_hex());

    let mut token_matched = false;
    let mut scope_forbidden = false;
    for row in rows {
        let token_ok = row
            .get("token")
            .and_then(|v| v.as_str())
            .map(|v| v == token)
            .unwrap_or(false)
            || row
                .get("token_hash")
                .and_then(|v| v.as_str())
                .map(|v| v == token_hash)
                .unwrap_or(false);
        if !token_ok {
            continue;
        }
        token_matched = true;
        if !auth_role_matches(row, role) {
            continue;
        }
        if !auth_scope_matches(row, scope) {
            scope_forbidden = true;
            continue;
        }
        return Ok(());
    }
    if !token_matched {
        return Err(diag::build_diag(
            "E_REG_AUTH_TOKEN_UNKNOWN",
            "policy에 없는 token입니다.",
            None,
            Some("token 또는 token_hash를 auth policy tokens[]에 등록하세요.".to_string()),
        ));
    }
    if scope_forbidden {
        return Err(diag::build_diag(
            "E_REG_AUTH_SCOPE_FORBIDDEN",
            &format!("scope={} role={}", scope, role),
            None,
            Some("해당 role에 scope 권한(scopes)을 부여하세요.".to_string()),
        ));
    }
    Err(diag::build_diag(
        "E_REG_AUTH_ROLE_MISMATCH",
        &format!("role={}", role),
        None,
        Some("auth policy role/roles와 요청 role을 일치시키세요.".to_string()),
    ))
}

fn auth_role_matches(row: &Value, role: &str) -> bool {
    if row
        .get("role")
        .and_then(|v| v.as_str())
        .map(|v| v == role)
        .unwrap_or(false)
    {
        return true;
    }
    if let Some(roles) = row.get("roles").and_then(|v| v.as_array()) {
        return roles.iter().any(|v| v.as_str() == Some(role));
    }
    false
}

fn auth_scope_matches(row: &Value, scope: &str) -> bool {
    let Some(scopes_value) = row.get("scopes") else {
        return true;
    };
    if let Some(single) = scopes_value.as_str() {
        return single == "*" || single == scope;
    }
    if let Some(scopes) = scopes_value.as_array() {
        for item in scopes {
            if let Some(s) = item.as_str() {
                if s == "*" || s == scope {
                    return true;
                }
            }
        }
        return false;
    }
    false
}

fn ensure_role(token: &str, role: &str, allowed: &[&str]) -> Result<(), String> {
    if token.trim().is_empty() {
        return Err(diag::build_diag(
            "E_REG_AUTH_REQUIRED",
            "token이 필요합니다.",
            None,
            Some("--token <value>를 지정하세요.".to_string()),
        ));
    }
    if !allowed.contains(&role) {
        return Err(diag::build_diag(
            "E_REG_SCOPE_FORBIDDEN",
            &format!("role={} (need one of: {})", role, allowed.join(",")),
            None,
            Some("허용 role 중 하나로 --role 값을 바꾸세요.".to_string()),
        ));
    }
    Ok(())
}

fn append_audit(
    index_path: &Path,
    audit_log: Option<&str>,
    action: &str,
    package_id: &str,
    role: &str,
    token: &str,
    allowed: bool,
    error_code: Option<&str>,
    at: Option<&str>,
) -> Result<(), String> {
    let audit_path = if let Some(path) = audit_log {
        Path::new(path).to_path_buf()
    } else {
        index_path.with_extension("audit.jsonl")
    };
    if let Some(parent) = audit_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            diag::build_diag(
                "E_REG_AUDIT_WRITE",
                &format!("path={} {}", audit_path.display(), e),
                None,
                Some("audit 출력 경로를 확인하세요.".to_string()),
            )
        })?;
    }
    let prev_hash = read_last_audit_hash(&audit_path)?;
    let token_hash = format!("blake3:{}", blake3::hash(token.as_bytes()).to_hex());
    let ts = now_or(at);
    let body = json!({
        "schema": "ddn.registry.audit.v1",
        "ts": ts,
        "action": action,
        "package_id": package_id,
        "role": role,
        "token_hash": token_hash,
        "allowed": allowed,
        "error_code": error_code,
        "prev_hash": prev_hash
    });
    let body_text = serde_json::to_string(&body).map_err(|e| {
        diag::build_diag(
            "E_REG_JSON",
            &format!("audit body serialize failed: {}", e),
            None,
            Some("audit body payload를 점검하세요.".to_string()),
        )
    })?;
    let row_hash = format!("blake3:{}", blake3::hash(body_text.as_bytes()).to_hex());
    let row = json!({
        "body": body,
        "row_hash": row_hash
    });
    let line = serde_json::to_string(&row).map_err(|e| {
        diag::build_diag(
            "E_REG_JSON",
            &format!("audit row serialize failed: {}", e),
            None,
            Some("audit row payload를 점검하세요.".to_string()),
        )
    })?;
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&audit_path)
        .map_err(|e| {
            diag::build_diag(
                "E_REG_AUDIT_WRITE",
                &format!("path={} {}", audit_path.display(), e),
                None,
                Some("audit 파일 쓰기 권한/경로를 확인하세요.".to_string()),
            )
        })?;
    writeln!(file, "{}", line).map_err(|e| {
        diag::build_diag(
            "E_REG_AUDIT_WRITE",
            &format!("path={} {}", audit_path.display(), e),
            None,
            Some("audit 파일 쓰기 권한/경로를 확인하세요.".to_string()),
        )
    })?;
    Ok(())
}

fn read_last_audit_hash(path: &Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|e| {
        diag::build_diag(
            "E_REG_AUDIT_READ",
            &format!("path={} {}", path.display(), e),
            None,
            Some("감사로그 파일 경로/권한을 확인하세요.".to_string()),
        )
    })?;
    let mut last = None;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line).map_err(|e| {
            diag::build_diag(
                "E_REG_AUDIT_PARSE",
                &format!("path={} {}", path.display(), e),
                Some("감사로그 줄 JSON 파싱 실패".to_string()),
                Some("감사로그 JSON 줄을 정정하세요.".to_string()),
            )
        })?;
        last = value
            .get("row_hash")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
    }
    Ok(last)
}

fn now_or(at: Option<&str>) -> String {
    if let Some(value) = at {
        return value.to_string();
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("unix:{}", secs)
}

fn req_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
        diag::build_diag(
            "E_REG_INDEX_FIELD",
            &format!("{} 누락", key),
            None,
            Some(format!("인덱스 항목에 '{}' 필드를 추가하세요.", key)),
        )
    })
}

fn opt_str(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
}

fn compare_versions_desc(left: &str, right: &str) -> Ordering {
    match (parse_semver(left), parse_semver(right)) {
        (Some(a), Some(b)) => b.cmp(&a).then_with(|| right.cmp(left)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => right.cmp(left),
    }
}

fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    let clean = version.strip_prefix('v').unwrap_or(version);
    let mut parts = clean.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ddn_gaji_registry_{}_{}", name, stamp));
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    fn write_lock_meta(
        path: &Path,
        snapshot_id: Option<&str>,
        index_root_hash: Option<&str>,
        trust_root_hash: Option<&str>,
    ) {
        let mut root = json!({
            "schema_version": "v1",
            "packages": []
        });

        if snapshot_id.is_some() || index_root_hash.is_some() {
            root["registry_snapshot"] = json!({
                "snapshot_id": snapshot_id,
                "index_root_hash": index_root_hash,
            });
        }
        if let Some(hash) = trust_root_hash {
            root["trust_root"] = json!({
                "hash": hash,
                "source": "registry",
            });
        }

        fs::write(path, serde_json::to_string_pretty(&root).expect("json")).expect("write lock");
    }

    fn write_lock_with_packages(
        path: &Path,
        packages: Value,
        snapshot_id: Option<&str>,
        index_root_hash: Option<&str>,
    ) {
        let mut root = json!({
            "schema_version": "v1",
            "packages": packages
        });
        if snapshot_id.is_some() || index_root_hash.is_some() {
            root["registry_snapshot"] = json!({
                "snapshot_id": snapshot_id,
                "index_root_hash": index_root_hash,
            });
        }
        fs::write(path, serde_json::to_string_pretty(&root).expect("json")).expect("write lock");
    }

    fn assert_verify_report_contract(report: &Value, packages: u64, matched: u64) {
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            report.get("packages").and_then(|v| v.as_u64()),
            Some(packages)
        );
        assert_eq!(
            report.get("matched").and_then(|v| v.as_u64()),
            Some(matched)
        );
        assert_eq!(
            report
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    fn assert_audit_verify_report_contract(report: &Value, rows: u64) {
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.audit_verify_report.v1")
        );
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(report.get("rows").and_then(|v| v.as_u64()), Some(rows));
        assert!(report
            .get("last_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .starts_with("blake3:"));
    }

    fn assert_diag_with_fix(err: &str, code: &str) {
        assert!(err.contains(code), "missing code {code} in {err}");
        assert!(err.contains("fix="), "missing fix= in {err}");
    }

    fn assert_audit_last_hash_diag(err: &str) {
        assert_diag_with_fix(err, "E_REG_AUDIT_LAST_HASH_MISMATCH");
        assert!(err.contains("hint="), "missing hint= in {err}");
    }

    fn start_http_fixture(bytes: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind http fixture");
        let addr = listener.local_addr().expect("local addr");
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                bytes.len()
            );
            stream.write_all(headers.as_bytes()).expect("write headers");
            stream.write_all(bytes).expect("write body");
            stream.flush().expect("flush body");
        });
        format!("http://{}/archive.ddn.tar.gz", addr)
    }

    #[test]
    fn registry_diag_assertions_use_helpers_only() {
        let src = include_str!("gaji_registry.rs");
        assert!(
            !src.contains("assert!(err.contains(\"E_REG_"),
            "raw E_REG assertion found; use assert_diag_with_fix helper"
        );
    }

    fn sample_entries() -> Vec<RegistryEntry> {
        vec![
            RegistryEntry {
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: Some("sha256:a".to_string()),
                contract: Some("D-STRICT".to_string()),
                detmath_seal_hash: Some("sha256:seal".to_string()),
                min_runtime: Some("20.6.29".to_string()),
                dependencies: None,
                download_url: Some("https://registry/1".to_string()),
                published_at: Some("2026-02-19T00:00:00Z".to_string()),
                summary: Some("역학".to_string()),
                yanked: false,
                yanked_at: None,
                yank_reason_code: None,
                yank_note: None,
            },
            RegistryEntry {
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.31".to_string(),
                archive_sha256: Some("sha256:b".to_string()),
                contract: Some("D-STRICT".to_string()),
                detmath_seal_hash: Some("sha256:seal".to_string()),
                min_runtime: Some("20.6.29".to_string()),
                dependencies: None,
                download_url: Some("https://registry/2".to_string()),
                published_at: Some("2026-02-20T00:00:00Z".to_string()),
                summary: Some("역학".to_string()),
                yanked: true,
                yanked_at: Some("2026-02-21T00:00:00Z".to_string()),
                yank_reason_code: Some("broken".to_string()),
                yank_note: Some("test".to_string()),
            },
            RegistryEntry {
                scope: "나눔".to_string(),
                name: "파동".to_string(),
                version: "1.0.0".to_string(),
                archive_sha256: Some("sha256:c".to_string()),
                contract: Some("D-APPROX".to_string()),
                detmath_seal_hash: None,
                min_runtime: Some("20.6.29".to_string()),
                dependencies: None,
                download_url: Some("https://registry/3".to_string()),
                published_at: Some("2026-02-18T00:00:00Z".to_string()),
                summary: Some("파동".to_string()),
                yanked: false,
                yanked_at: None,
                yank_reason_code: None,
                yank_note: None,
            },
        ]
    }

    fn row_hash_for_body(body: &Value) -> String {
        let body_text = serde_json::to_string(body).expect("body json");
        format!("blake3:{}", blake3::hash(body_text.as_bytes()).to_hex())
    }

    fn read_audit_rows(path: &Path) -> Vec<Value> {
        let text = fs::read_to_string(path).expect("audit read");
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str::<Value>(line).expect("audit row parse"))
            .collect()
    }

    fn write_audit_rows(path: &Path, rows: &[Value]) {
        let mut lines = Vec::with_capacity(rows.len());
        for row in rows {
            lines.push(serde_json::to_string(row).expect("row json"));
        }
        fs::write(path, lines.join("\n") + "\n").expect("audit write");
    }

    fn write_valid_audit_log(path: &Path) {
        let body1 = json!({
            "schema": "ddn.registry.audit.v1",
            "ts": "2026-02-20T00:00:00Z",
            "action": "publish",
            "package_id": "표준/역학@20.6.30",
            "role": "publisher",
            "token_hash": "blake3:token1",
            "allowed": true,
            "error_code": Value::Null,
            "prev_hash": Value::Null
        });
        let row1 = json!({
            "body": body1.clone(),
            "row_hash": row_hash_for_body(&body1)
        });
        let body2 = json!({
            "schema": "ddn.registry.audit.v1",
            "ts": "2026-02-20T00:00:01Z",
            "action": "yank",
            "package_id": "표준/역학@20.6.30",
            "role": "scope_admin",
            "token_hash": "blake3:token2",
            "allowed": true,
            "error_code": Value::Null,
            "prev_hash": row1.get("row_hash").and_then(|v| v.as_str())
        });
        let row2 = json!({
            "body": body2.clone(),
            "row_hash": row_hash_for_body(&body2)
        });
        write_audit_rows(path, &[row1, row2]);
    }

    fn last_row_hash(path: &Path) -> String {
        let rows = read_audit_rows(path);
        rows.last()
            .and_then(|row| row.get("row_hash"))
            .and_then(|v| v.as_str())
            .expect("last row hash")
            .to_string()
    }

    fn merge_object_fields(dst: &mut Value, patch: &Value) {
        let dst_map = dst.as_object_mut().expect("dst object");
        let patch_map = patch.as_object().expect("patch object");
        for (k, v) in patch_map {
            dst_map.insert(k.clone(), v.clone());
        }
    }

    fn default_verify_index_entry() -> Value {
        json!({
            "schema": "ddn.registry.index_entry.v1",
            "scope": "표준",
            "name": "역학",
            "version": "20.6.30",
            "yanked": false
        })
    }

    fn default_verify_lock_package() -> Value {
        json!({
            "id": "표준/역학",
            "version": "20.6.30",
            "path": "x",
            "hash": "blake3:x",
            "yanked": false
        })
    }

    fn write_verify_fixture(
        index: &Path,
        lock: &Path,
        index_entry_patch: Value,
        lock_package_patch: Value,
    ) {
        let mut entry = default_verify_index_entry();
        merge_object_fields(&mut entry, &index_entry_patch);
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [entry]
        });
        fs::write(
            index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let mut pkg = default_verify_lock_package();
        merge_object_fields(&mut pkg, &lock_package_patch);
        write_lock_with_packages(lock, json!([pkg]), Some("snap-42"), Some("sha256:abc"));
    }

    fn verify_args(index: &Path, lock: &Path) -> Vec<String> {
        vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ]
    }

    fn write_auth_policy(path: &Path, rows: Value) {
        let root = json!({
            "schema": "ddn.registry.auth_policy.v1",
            "tokens": rows
        });
        fs::write(path, serde_json::to_string_pretty(&root).expect("json")).expect("policy write");
    }

    #[test]
    fn versions_excludes_yanked_by_default() {
        let out =
            build_versions_response(&sample_entries(), "표준", "역학", false).expect("versions");
        let rows = out
            .get("versions")
            .and_then(|v| v.as_array())
            .expect("versions array");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("version").and_then(|v| v.as_str()),
            Some("20.6.30")
        );
    }

    #[test]
    fn versions_include_yanked_deduplicates_same_version_prefers_non_yanked() {
        let entries = vec![
            RegistryEntry {
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: Some("sha256:a-y".to_string()),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                dependencies: None,
                download_url: None,
                published_at: None,
                summary: Some("역학".to_string()),
                yanked: true,
                yanked_at: Some("2026-02-20T00:00:00Z".to_string()),
                yank_reason_code: Some("policy".to_string()),
                yank_note: None,
            },
            RegistryEntry {
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: Some("sha256:a-n".to_string()),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                dependencies: None,
                download_url: None,
                published_at: None,
                summary: Some("역학".to_string()),
                yanked: false,
                yanked_at: None,
                yank_reason_code: None,
                yank_note: None,
            },
        ];
        let out = build_versions_response(&entries, "표준", "역학", true).expect("versions");
        let rows = out
            .get("versions")
            .and_then(|v| v.as_array())
            .expect("versions array");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("version").and_then(|v| v.as_str()),
            Some("20.6.30")
        );
        assert_eq!(rows[0].get("yanked").and_then(|v| v.as_bool()), Some(false));
        assert_eq!(
            rows[0].get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:a-n")
        );
    }

    #[test]
    fn versions_duplicate_same_version_same_state_is_order_independent() {
        let a = RegistryEntry {
            scope: "표준".to_string(),
            name: "역학".to_string(),
            version: "20.6.30".to_string(),
            archive_sha256: Some("sha256:a".to_string()),
            contract: None,
            detmath_seal_hash: None,
            min_runtime: None,
            dependencies: None,
            download_url: Some("https://registry/a".to_string()),
            published_at: Some("2026-02-19T00:00:00Z".to_string()),
            summary: Some("가".to_string()),
            yanked: false,
            yanked_at: None,
            yank_reason_code: None,
            yank_note: None,
        };
        let b = RegistryEntry {
            scope: "표준".to_string(),
            name: "역학".to_string(),
            version: "20.6.30".to_string(),
            archive_sha256: Some("sha256:b".to_string()),
            contract: None,
            detmath_seal_hash: None,
            min_runtime: None,
            dependencies: None,
            download_url: Some("https://registry/b".to_string()),
            published_at: Some("2026-02-19T00:00:00Z".to_string()),
            summary: Some("나".to_string()),
            yanked: false,
            yanked_at: None,
            yank_reason_code: None,
            yank_note: None,
        };

        let out_ab = build_versions_response(&vec![b.clone(), a.clone()], "표준", "역학", true)
            .expect("versions ab");
        let rows_ab = out_ab
            .get("versions")
            .and_then(|v| v.as_array())
            .expect("versions array");
        let out_ba = build_versions_response(&vec![a.clone(), b.clone()], "표준", "역학", true)
            .expect("versions ba");
        let rows_ba = out_ba
            .get("versions")
            .and_then(|v| v.as_array())
            .expect("versions array");

        assert_eq!(rows_ab.len(), 1);
        assert_eq!(rows_ba.len(), 1);
        assert_eq!(
            rows_ab[0].get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:a")
        );
        assert_eq!(
            rows_ba[0].get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:a")
        );
    }

    #[test]
    fn entry_lookup_returns_exact_version() {
        let out =
            build_entry_response(&sample_entries(), "표준", "역학", "20.6.31").expect("entry");
        assert_eq!(
            out.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.index_entry.v1")
        );
        assert_eq!(out.get("yanked").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn entry_lookup_prefers_non_yanked_when_duplicate_pin_exists() {
        let entries = vec![
            RegistryEntry {
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: Some("sha256:y".to_string()),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                dependencies: None,
                download_url: None,
                published_at: None,
                summary: Some("역학".to_string()),
                yanked: true,
                yanked_at: Some("2026-02-20T00:00:00Z".to_string()),
                yank_reason_code: Some("policy".to_string()),
                yank_note: None,
            },
            RegistryEntry {
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: Some("sha256:n".to_string()),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                dependencies: None,
                download_url: None,
                published_at: None,
                summary: Some("역학".to_string()),
                yanked: false,
                yanked_at: None,
                yank_reason_code: None,
                yank_note: None,
            },
        ];
        let out = build_entry_response(&entries, "표준", "역학", "20.6.30").expect("entry");
        assert_eq!(out.get("yanked").and_then(|v| v.as_bool()), Some(false));
        assert_eq!(
            out.get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:n")
        );
    }

    #[test]
    fn entry_lookup_same_state_duplicate_is_order_independent() {
        let a = RegistryEntry {
            scope: "표준".to_string(),
            name: "역학".to_string(),
            version: "20.6.30".to_string(),
            archive_sha256: Some("sha256:a".to_string()),
            contract: None,
            detmath_seal_hash: None,
            min_runtime: None,
            dependencies: None,
            download_url: Some("https://registry/a".to_string()),
            published_at: Some("2026-02-19T00:00:00Z".to_string()),
            summary: Some("가".to_string()),
            yanked: false,
            yanked_at: None,
            yank_reason_code: None,
            yank_note: None,
        };
        let b = RegistryEntry {
            scope: "표준".to_string(),
            name: "역학".to_string(),
            version: "20.6.30".to_string(),
            archive_sha256: Some("sha256:b".to_string()),
            contract: None,
            detmath_seal_hash: None,
            min_runtime: None,
            dependencies: None,
            download_url: Some("https://registry/b".to_string()),
            published_at: Some("2026-02-19T00:00:00Z".to_string()),
            summary: Some("나".to_string()),
            yanked: false,
            yanked_at: None,
            yank_reason_code: None,
            yank_note: None,
        };

        let out_ab = build_entry_response(&vec![b.clone(), a.clone()], "표준", "역학", "20.6.30")
            .expect("entry ab");
        let out_ba = build_entry_response(&vec![a.clone(), b.clone()], "표준", "역학", "20.6.30")
            .expect("entry ba");

        assert_eq!(
            out_ab.get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:a")
        );
        assert_eq!(
            out_ba.get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:a")
        );
    }

    #[test]
    fn search_builds_latest_items_with_limit() {
        let out = build_search_response(&sample_entries(), "학", 1, false).expect("search");
        let items = out.get("items").and_then(|v| v.as_array()).expect("items");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].get("latest_version").and_then(|v| v.as_str()),
            Some("20.6.30")
        );
    }

    #[test]
    fn search_includes_yanked_latest_when_include_yanked_true() {
        let out = build_search_response(&sample_entries(), "학", 10, true).expect("search");
        let items = out.get("items").and_then(|v| v.as_array()).expect("items");
        let row = items
            .iter()
            .find(|item| {
                item.get("scope").and_then(|v| v.as_str()) == Some("표준")
                    && item.get("name").and_then(|v| v.as_str()) == Some("역학")
            })
            .expect("표준/역학 row");
        assert_eq!(
            row.get("latest_version").and_then(|v| v.as_str()),
            Some("20.6.31")
        );
        assert_eq!(row.get("yanked").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn search_limit_zero_means_unbounded() {
        let mut entries = sample_entries();
        entries.push(RegistryEntry {
            scope: "표준".to_string(),
            name: "화학".to_string(),
            version: "1.0.0".to_string(),
            archive_sha256: Some("sha256:d".to_string()),
            contract: Some("D-STRICT".to_string()),
            detmath_seal_hash: None,
            min_runtime: Some("20.6.29".to_string()),
            dependencies: None,
            download_url: Some("https://registry/4".to_string()),
            published_at: Some("2026-02-18T00:00:00Z".to_string()),
            summary: Some("화학".to_string()),
            yanked: false,
            yanked_at: None,
            yank_reason_code: None,
            yank_note: None,
        });

        let out = build_search_response(&entries, "학", 0, false).expect("search");
        let items = out.get("items").and_then(|v| v.as_array()).expect("items");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn search_latest_version_prefers_higher_semver() {
        let entries = vec![
            RegistryEntry {
                scope: "표준".to_string(),
                name: "대수".to_string(),
                version: "1.2.0".to_string(),
                archive_sha256: None,
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                dependencies: None,
                download_url: None,
                published_at: None,
                summary: Some("대수".to_string()),
                yanked: false,
                yanked_at: None,
                yank_reason_code: None,
                yank_note: None,
            },
            RegistryEntry {
                scope: "표준".to_string(),
                name: "대수".to_string(),
                version: "1.10.0".to_string(),
                archive_sha256: None,
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                dependencies: None,
                download_url: None,
                published_at: None,
                summary: Some("대수".to_string()),
                yanked: false,
                yanked_at: None,
                yank_reason_code: None,
                yank_note: None,
            },
        ];
        let out = build_search_response(&entries, "대수", 10, false).expect("search");
        let items = out.get("items").and_then(|v| v.as_array()).expect("items");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].get("latest_version").and_then(|v| v.as_str()),
            Some("1.10.0")
        );
    }

    #[test]
    fn search_same_latest_version_prefers_non_yanked() {
        let entries = vec![
            RegistryEntry {
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: None,
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                dependencies: None,
                download_url: None,
                published_at: None,
                summary: Some("역학".to_string()),
                yanked: true,
                yanked_at: Some("2026-02-20T00:00:00Z".to_string()),
                yank_reason_code: Some("policy".to_string()),
                yank_note: None,
            },
            RegistryEntry {
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: None,
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                dependencies: None,
                download_url: None,
                published_at: None,
                summary: Some("역학".to_string()),
                yanked: false,
                yanked_at: None,
                yank_reason_code: None,
                yank_note: None,
            },
        ];
        let out = build_search_response(&entries, "역", 20, true).expect("search");
        let items = out.get("items").and_then(|v| v.as_array()).expect("items");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].get("latest_version").and_then(|v| v.as_str()),
            Some("20.6.30")
        );
        assert_eq!(
            items[0].get("yanked").and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn search_same_latest_version_same_state_is_order_independent() {
        let a = RegistryEntry {
            scope: "표준".to_string(),
            name: "역학".to_string(),
            version: "20.6.30".to_string(),
            archive_sha256: Some("sha256:a".to_string()),
            contract: None,
            detmath_seal_hash: None,
            min_runtime: None,
            dependencies: None,
            download_url: Some("https://registry/a".to_string()),
            published_at: Some("2026-02-19T00:00:00Z".to_string()),
            summary: Some("가".to_string()),
            yanked: false,
            yanked_at: None,
            yank_reason_code: None,
            yank_note: None,
        };
        let b = RegistryEntry {
            scope: "표준".to_string(),
            name: "역학".to_string(),
            version: "20.6.30".to_string(),
            archive_sha256: Some("sha256:b".to_string()),
            contract: None,
            detmath_seal_hash: None,
            min_runtime: None,
            dependencies: None,
            download_url: Some("https://registry/b".to_string()),
            published_at: Some("2026-02-19T00:00:00Z".to_string()),
            summary: Some("나".to_string()),
            yanked: false,
            yanked_at: None,
            yank_reason_code: None,
            yank_note: None,
        };

        let out_ab =
            build_search_response(&vec![b.clone(), a.clone()], "역", 20, true).expect("search ab");
        let items_ab = out_ab
            .get("items")
            .and_then(|v| v.as_array())
            .expect("items");
        let out_ba =
            build_search_response(&vec![a.clone(), b.clone()], "역", 20, true).expect("search ba");
        let items_ba = out_ba
            .get("items")
            .and_then(|v| v.as_array())
            .expect("items");

        assert_eq!(items_ab.len(), 1);
        assert_eq!(items_ba.len(), 1);
        assert_eq!(
            items_ab[0].get("summary").and_then(|v| v.as_str()),
            Some("가")
        );
        assert_eq!(
            items_ba[0].get("summary").and_then(|v| v.as_str()),
            Some("가")
        );
    }

    #[test]
    fn search_yanked_only_returns_empty_items_without_include_yanked() {
        let entries = vec![RegistryEntry {
            scope: "표준".to_string(),
            name: "열역학".to_string(),
            version: "1.0.0".to_string(),
            archive_sha256: None,
            contract: None,
            detmath_seal_hash: None,
            min_runtime: None,
            dependencies: None,
            download_url: None,
            published_at: None,
            summary: Some("열역학".to_string()),
            yanked: true,
            yanked_at: Some("2026-02-20T00:00:00Z".to_string()),
            yank_reason_code: Some("policy".to_string()),
            yank_note: None,
        }];
        let out = build_search_response(&entries, "열", 20, false).expect("search");
        let items = out.get("items").and_then(|v| v.as_array()).expect("items");
        assert!(items.is_empty());
    }

    #[test]
    fn search_yanked_only_returns_item_with_include_yanked() {
        let entries = vec![RegistryEntry {
            scope: "표준".to_string(),
            name: "열역학".to_string(),
            version: "1.0.0".to_string(),
            archive_sha256: None,
            contract: None,
            detmath_seal_hash: None,
            min_runtime: None,
            dependencies: None,
            download_url: None,
            published_at: None,
            summary: Some("열역학".to_string()),
            yanked: true,
            yanked_at: Some("2026-02-20T00:00:00Z".to_string()),
            yank_reason_code: Some("policy".to_string()),
            yank_note: None,
        }];
        let out = build_search_response(&entries, "열", 20, true).expect("search");
        let items = out.get("items").and_then(|v| v.as_array()).expect("items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].get("yanked").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn search_no_match_returns_empty_items() {
        let out =
            build_search_response(&sample_entries(), "없는꾸러미", 20, false).expect("search");
        let items = out.get("items").and_then(|v| v.as_array()).expect("items");
        assert!(items.is_empty());
    }

    #[test]
    fn snapshot_schema_loads_entries() {
        let root = json!({
            "schema": "ddn.registry.snapshot.v1",
            "entries": [
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30"
                }
            ]
        });
        let entries = load_entries_from_value(&root).expect("load");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].scope, "표준");
    }

    #[test]
    fn publish_rejects_same_version_overwrite() {
        let root = temp_dir("publish_immutable");
        let index = root.join("index.json");
        let options = PublishOptions {
            audit_log: None,
            scope: "표준".to_string(),
            name: "역학".to_string(),
            version: "20.6.30".to_string(),
            archive_sha256: "sha256:a".to_string(),
            contract: Some("D-STRICT".to_string()),
            detmath_seal_hash: None,
            min_runtime: None,
            download_url: None,
            summary: None,
            token: "token1".to_string(),
            role: "publisher".to_string(),
            at: Some("2026-02-19T00:00:00Z".to_string()),
        };
        run_publish(&index, &options).expect("first publish");
        let err = run_publish(&index, &options).expect_err("must reject overwrite");
        assert_diag_with_fix(&err, "E_REG_IMMUTABLE_EXISTS");
    }

    #[test]
    fn publish_writes_snapshot_meta_fields() {
        let root = temp_dir("publish_snapshot_meta");
        let index = root.join("index.json");
        let options = PublishOptions {
            audit_log: None,
            scope: "표준".to_string(),
            name: "역학".to_string(),
            version: "20.6.30".to_string(),
            archive_sha256: "sha256:a".to_string(),
            contract: Some("D-STRICT".to_string()),
            detmath_seal_hash: None,
            min_runtime: None,
            download_url: None,
            summary: None,
            token: "token1".to_string(),
            role: "publisher".to_string(),
            at: Some("2026-02-19T00:00:00Z".to_string()),
        };
        run_publish(&index, &options).expect("publish");

        let text = fs::read_to_string(index).expect("read index");
        let value: Value = serde_json::from_str(&text).expect("parse index");
        assert_eq!(
            value.get("snapshot_id").and_then(|v| v.as_str()),
            Some("2026-02-19T00:00:00Z")
        );
        let hash = value
            .get("index_root_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(hash.starts_with("blake3:"));
    }

    #[test]
    fn yank_requires_scope_admin_role() {
        let root = temp_dir("yank_role");
        let index = root.join("index.json");
        let publish = PublishOptions {
            audit_log: None,
            scope: "표준".to_string(),
            name: "역학".to_string(),
            version: "20.6.30".to_string(),
            archive_sha256: "sha256:a".to_string(),
            contract: Some("D-STRICT".to_string()),
            detmath_seal_hash: None,
            min_runtime: None,
            download_url: None,
            summary: None,
            token: "token1".to_string(),
            role: "publisher".to_string(),
            at: Some("2026-02-19T00:00:00Z".to_string()),
        };
        run_publish(&index, &publish).expect("publish");
        let err = run_yank(
            &index,
            &YankOptions {
                audit_log: None,
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                reason_code: "broken".to_string(),
                note: None,
                token: "token2".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect_err("must reject role");
        assert_diag_with_fix(&err, "E_REG_SCOPE_FORBIDDEN");
    }

    #[test]
    fn yank_missing_entry_fails_with_fix() {
        let root = temp_dir("yank_missing_entry");
        let index = root.join("index.json");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: None,
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let err = run_yank(
            &index,
            &YankOptions {
                audit_log: None,
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "99.9.99".to_string(),
                reason_code: "policy".to_string(),
                note: None,
                token: "token2".to_string(),
                role: "scope_admin".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect_err("missing entry must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");
    }

    #[test]
    fn publish_then_yank_writes_index_and_audit() {
        let root = temp_dir("publish_yank");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: Some("D-STRICT".to_string()),
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: Some("요약".to_string()),
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");
        run_yank(
            &index,
            &YankOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                reason_code: "policy".to_string(),
                note: Some("note".to_string()),
                token: "token2".to_string(),
                role: "scope_admin".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("yank");

        let entries = load_entries(&index).expect("load entries");
        assert_eq!(entries.len(), 1);
        assert!(entries[0].yanked);
        assert_eq!(entries[0].yank_reason_code.as_deref(), Some("policy"));

        let audit_text = fs::read_to_string(audit).expect("audit read");
        let lines: Vec<&str> = audit_text.lines().filter(|line| !line.is_empty()).collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn audit_verify_passes_for_publish_yank_log() {
        let root = temp_dir("audit_verify_ok");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: Some("D-STRICT".to_string()),
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");
        run_yank(
            &index,
            &YankOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                reason_code: "policy".to_string(),
                note: None,
                token: "token2".to_string(),
                role: "scope_admin".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("yank");

        let report = run_audit_verify(&audit).expect("audit verify");
        assert_eq!(report.rows, 2);
        assert!(report.last_hash.is_some());
    }

    #[test]
    fn audit_verify_detects_row_hash_tamper() {
        let root = temp_dir("audit_verify_row_hash_bad");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let mut rows = read_audit_rows(&audit);
        rows[0]["row_hash"] = Value::String("blake3:tampered".to_string());
        write_audit_rows(&audit, &rows);

        let err = run_audit_verify(&audit).expect_err("must fail row hash");
        assert_diag_with_fix(&err, "E_REG_AUDIT_ROW_HASH_MISMATCH");
    }

    #[test]
    fn audit_verify_detects_chain_break() {
        let root = temp_dir("audit_verify_chain_bad");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");
        run_yank(
            &index,
            &YankOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                reason_code: "policy".to_string(),
                note: None,
                token: "token2".to_string(),
                role: "scope_admin".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("yank");

        let mut rows = read_audit_rows(&audit);
        rows[1]["body"]["prev_hash"] = Value::String("blake3:broken".to_string());
        let second_body = rows[1].get("body").cloned().expect("body");
        rows[1]["row_hash"] = Value::String(row_hash_for_body(&second_body));
        write_audit_rows(&audit, &rows);

        let err = run_audit_verify(&audit).expect_err("must fail chain");
        assert_diag_with_fix(&err, "E_REG_AUDIT_CHAIN_BROKEN");
    }

    #[test]
    fn run_cli_versions_executes() {
        let root = temp_dir("cli_versions");
        let index = root.join("index.json");
        write_snapshot(&index, &sample_entries()).expect("write snapshot");

        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        run_cli(&args).expect("cli versions");
    }

    #[test]
    fn run_cli_versions_missing_index_file_fails() {
        let root = temp_dir("cli_versions_missing_index");
        let index = root.join("missing.index.json");
        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing index must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn run_cli_versions_invalid_index_json_fails() {
        let root = temp_dir("cli_versions_invalid_index_json");
        let index = root.join("index.json");
        fs::write(&index, "{ invalid json").expect("write bad index");
        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid index json must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_PARSE");
    }

    #[test]
    fn run_cli_versions_index_schema_mismatch_fails() {
        let root = temp_dir("cli_versions_index_schema_mismatch");
        let index = root.join("index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.unknown.v1"
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_cli(&args).expect_err("schema mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_SCHEMA");
    }

    #[test]
    fn run_cli_entry_missing_index_file_fails() {
        let root = temp_dir("cli_entry_missing_index");
        let index = root.join("missing.index.json");
        let args = vec![
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing index must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn run_cli_search_empty_query_fails() {
        let root = temp_dir("cli_search_empty_query");
        let index = root.join("index.json");
        write_snapshot(&index, &sample_entries()).expect("write snapshot");

        let args = vec![
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "   ".to_string(),
        ];
        let err = run_cli(&args).expect_err("empty query must fail");
        assert_diag_with_fix(&err, "E_REG_SEARCH_QUERY");
    }

    #[test]
    fn run_cli_publish_and_yank_executes() {
        let root = temp_dir("cli_publish_yank");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");

        let publish_args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_cli(&publish_args).expect("cli publish");

        let yank_args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        run_cli(&yank_args).expect("cli yank");
    }

    #[test]
    fn run_cli_publish_duplicate_writes_audit_denied() {
        let root = temp_dir("cli_publish_duplicate_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");

        let publish_args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_cli(&publish_args).expect("first publish");

        let err = run_cli(&publish_args).expect_err("duplicate publish must fail");
        assert_diag_with_fix(&err, "E_REG_IMMUTABLE_EXISTS");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_IMMUTABLE_EXISTS")
        );
    }

    #[test]
    fn run_cli_yank_missing_entry_fails_with_fix() {
        let root = temp_dir("cli_yank_missing_entry");
        let index = root.join("index.json");

        let publish_args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_cli(&publish_args).expect("cli publish");

        let yank_args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "99.9.99".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&yank_args).expect_err("missing yank target must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");
    }

    #[test]
    fn run_cli_yank_missing_entry_writes_audit_denied() {
        let root = temp_dir("cli_yank_missing_entry_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");

        let publish_args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_cli(&publish_args).expect("cli publish");

        let yank_args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "99.9.99".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&yank_args).expect_err("missing yank target must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_INDEX_NOT_FOUND")
        );
    }

    #[test]
    fn run_cli_publish_role_forbidden_writes_audit_denied() {
        let root = temp_dir("cli_publish_role_denied_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "viewer".to_string(),
        ];
        let err = run_cli(&args).expect_err("role forbidden");
        assert_diag_with_fix(&err, "E_REG_SCOPE_FORBIDDEN");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_SCOPE_FORBIDDEN")
        );
    }

    #[test]
    fn run_cli_yank_missing_token_writes_audit_denied() {
        let root = temp_dir("cli_yank_token_denied_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");

        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token_ok".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing token");
        assert_diag_with_fix(&err, "E_REG_AUTH_REQUIRED");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_REQUIRED")
        );
    }

    #[test]
    fn run_cli_publish_with_auth_policy_token_hash_passes() {
        let root = temp_dir("cli_publish_auth_hash");
        let index = root.join("index.json");
        let policy = root.join("auth_policy.json");
        let token = "token_hash_only";
        let token_hash = format!("blake3:{}", blake3::hash(token.as_bytes()).to_hex());
        write_auth_policy(
            &policy,
            json!([{
                "token_hash": token_hash,
                "roles": ["publisher"],
                "scopes": ["표준"]
            }]),
        );

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            token.to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_cli(&args).expect("publish with token_hash policy");

        let entries = load_entries(&index).expect("entries");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn run_cli_publish_with_auth_policy_missing_file_fails() {
        let root = temp_dir("cli_publish_auth_policy_missing");
        let index = root.join("index.json");
        let policy = root.join("missing_auth_policy.json");

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing auth policy must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_READ");
    }

    #[test]
    fn run_cli_publish_with_auth_policy_missing_file_writes_audit_denied() {
        let root = temp_dir("cli_publish_auth_policy_missing_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let policy = root.join("missing_auth_policy.json");

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing auth policy must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_READ");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_READ")
        );
    }

    #[test]
    fn run_cli_publish_with_auth_policy_invalid_json_fails() {
        let root = temp_dir("cli_publish_auth_policy_invalid_json");
        let index = root.join("index.json");
        let policy = root.join("auth_policy.json");
        fs::write(&policy, "{ invalid json").expect("write bad auth policy");

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid auth policy json must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_PARSE");
    }

    #[test]
    fn run_cli_yank_with_auth_policy_invalid_json_writes_audit_denied() {
        let root = temp_dir("cli_yank_auth_policy_invalid_json_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(&policy, "{ invalid json").expect("write bad auth policy");

        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid auth policy json must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_PARSE");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_PARSE")
        );
    }

    #[test]
    fn run_cli_yank_with_auth_policy_missing_file_writes_audit_denied() {
        let root = temp_dir("cli_yank_auth_policy_missing_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let policy = root.join("missing_auth_policy.json");

        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing auth policy must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_READ");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_READ")
        );
    }

    #[test]
    fn run_cli_yank_with_auth_policy_schema_mismatch_fails() {
        let root = temp_dir("cli_yank_auth_policy_schema_mismatch");
        let index = root.join("index.json");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v0",
                "tokens": []
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        run_publish(
            &index,
            &PublishOptions {
                audit_log: None,
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&args).expect_err("schema mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");
    }

    #[test]
    fn run_cli_yank_with_auth_policy_schema_mismatch_writes_audit_denied() {
        let root = temp_dir("cli_yank_auth_policy_schema_mismatch_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v0",
                "tokens": []
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&args).expect_err("schema mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_SCHEMA")
        );
    }

    #[test]
    fn run_cli_yank_with_auth_policy_missing_tokens_fails() {
        let root = temp_dir("cli_yank_auth_policy_missing_tokens");
        let index = root.join("index.json");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1"
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        run_publish(
            &index,
            &PublishOptions {
                audit_log: None,
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing tokens must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");
    }

    #[test]
    fn run_cli_publish_with_auth_policy_schema_mismatch_fails() {
        let root = temp_dir("cli_publish_auth_policy_schema_mismatch");
        let index = root.join("index.json");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v0",
                "tokens": []
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_cli(&args).expect_err("schema mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");
    }

    #[test]
    fn run_cli_publish_with_auth_policy_schema_mismatch_writes_audit_denied() {
        let root = temp_dir("cli_publish_auth_policy_schema_mismatch_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v0",
                "tokens": []
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_cli(&args).expect_err("schema mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_SCHEMA")
        );
    }

    #[test]
    fn run_cli_publish_with_auth_policy_missing_tokens_fails() {
        let root = temp_dir("cli_publish_auth_policy_missing_tokens");
        let index = root.join("index.json");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1"
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing tokens must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");
    }

    #[test]
    fn run_cli_publish_with_auth_policy_missing_tokens_writes_audit_denied() {
        let root = temp_dir("cli_publish_auth_policy_missing_tokens_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1"
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing tokens must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_SCHEMA")
        );
    }

    #[test]
    fn run_cli_yank_with_auth_policy_missing_tokens_writes_audit_denied() {
        let root = temp_dir("cli_yank_auth_policy_missing_tokens_audit");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1"
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&args).expect_err("missing tokens must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_SCHEMA")
        );
    }

    #[test]
    fn run_cli_publish_with_auth_policy_unknown_token_fails_and_audits() {
        let root = temp_dir("cli_publish_auth_unknown");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let policy = root.join("auth_policy.json");
        write_auth_policy(
            &policy,
            json!([{
                "token": "token1",
                "role": "publisher",
                "scopes": ["표준"]
            }]),
        );

        let args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "unknown".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_cli(&args).expect_err("must reject unknown token");
        assert_diag_with_fix(&err, "E_REG_AUTH_TOKEN_UNKNOWN");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_TOKEN_UNKNOWN")
        );
    }

    #[test]
    fn run_cli_yank_with_auth_policy_scope_forbidden_fails() {
        let root = temp_dir("cli_yank_auth_scope");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let policy = root.join("auth_policy.json");
        write_auth_policy(
            &policy,
            json!([
                {
                    "token": "token1",
                    "role": "publisher",
                    "scopes": ["표준"]
                },
                {
                    "token": "token2",
                    "role": "scope_admin",
                    "scopes": ["나눔"]
                }
            ]),
        );

        let publish_args = vec![
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_cli(&publish_args).expect("publish");

        let yank_args = vec![
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_cli(&yank_args).expect_err("scope must be denied");
        assert_diag_with_fix(&err, "E_REG_AUTH_SCOPE_FORBIDDEN");
    }

    #[test]
    fn run_cli_audit_verify_executes() {
        let root = temp_dir("cli_audit_verify");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("cli audit verify");
    }

    #[test]
    fn run_cli_audit_verify_writes_report_json() {
        let root = temp_dir("cli_audit_verify_report");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let out = root.join("audit.verify.json");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("cli audit verify");

        let report: Value = serde_json::from_str(&fs::read_to_string(out).expect("report read"))
            .expect("report parse");
        assert_audit_verify_report_contract(&report, 1);
    }

    #[test]
    fn run_cli_audit_verify_writes_default_report_json() {
        let root = temp_dir("cli_audit_verify_default_report");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("cli audit verify");

        let default_report = audit.with_extension("verify.report.json");
        let report: Value =
            serde_json::from_str(&fs::read_to_string(default_report).expect("report read"))
                .expect("report parse");
        assert_audit_verify_report_contract(&report, 1);
    }

    #[test]
    fn run_cli_audit_verify_expect_last_hash_mismatch_fails() {
        let root = temp_dir("cli_audit_verify_expect_last_hash_bad");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_cli(&args).expect_err("must fail on expected last hash mismatch");
        assert_audit_last_hash_diag(&err);
    }

    #[test]
    fn run_cli_audit_verify_expect_last_hash_mismatch_does_not_write_report() {
        let root = temp_dir("cli_audit_verify_expect_last_hash_bad_no_report");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let out = root.join("audit.verify.custom.json");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let args = vec![
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_cli(&args).expect_err("expected hash mismatch must fail");
        assert_audit_last_hash_diag(&err);
        assert!(
            !out.exists(),
            "audit verify report must not be written on mismatch failure"
        );
    }

    #[test]
    fn run_cli_audit_verify_out_parent_file_fails_with_report_write() {
        let root = temp_dir("cli_audit_verify_out_parent_file");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        let out_parent = root.join("blocked");
        let out = out_parent.join("audit.verify.report.json");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");
        fs::write(&out_parent, "file blocks directory").expect("write parent file");

        let args = vec![
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("audit verify out parent file must fail");
        assert_diag_with_fix(&err, "E_REG_REPORT_WRITE");
        assert!(!out.exists());
    }

    #[test]
    fn run_cli_audit_verify_expect_last_hash_recovery_passes() {
        let root = temp_dir("cli_audit_verify_expect_last_hash_recovery");
        let index = root.join("index.json");
        let audit = root.join("audit.jsonl");
        run_publish(
            &index,
            &PublishOptions {
                audit_log: Some(audit.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "20.6.30".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-19T00:00:00Z".to_string()),
            },
        )
        .expect("publish");

        let bad_args = vec![
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_cli(&bad_args).expect_err("must fail on bad hash");
        assert_audit_last_hash_diag(&err);

        let good_args = vec![
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            last_row_hash(&audit),
        ];
        run_cli(&good_args).expect("recovery must pass");
    }

    #[test]
    fn run_cli_versions_frozen_requires_snapshot_meta() {
        let root = temp_dir("cli_versions_frozen");
        let index = root.join("index.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--frozen-lockfile".to_string(),
        ];
        let err = run_cli(&args).expect_err("must fail without snapshot meta");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISSING");
    }

    #[test]
    fn run_cli_versions_guard_with_snapshot_and_trust_passes() {
        let root = temp_dir("cli_versions_guard_ok");
        let index = root.join("index.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "trust_root": {
                "hash": "sha256:trust",
                "source": "registry"
            },
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--expect-snapshot-id".to_string(),
            "snap-42".to_string(),
            "--expect-index-root-hash".to_string(),
            "sha256:abc".to_string(),
            "--require-trust-root".to_string(),
            "--expect-trust-root-hash".to_string(),
            "sha256:trust".to_string(),
        ];
        run_cli(&args).expect("guard pass");
    }

    #[test]
    fn run_cli_versions_guard_index_hash_mismatch_fails() {
        let root = temp_dir("cli_versions_guard_bad");
        let index = root.join("index.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--expect-index-root-hash".to_string(),
            "sha256:def".to_string(),
        ];
        let err = run_cli(&args).expect_err("mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_ROOT_HASH_MISMATCH");
    }

    #[test]
    fn run_cli_versions_guard_from_lock_passes() {
        let root = temp_dir("cli_versions_lock_ok");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "trust_root": {
                "hash": "sha256:trust",
                "source": "registry"
            },
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_meta(
            &lock,
            Some("snap-42"),
            Some("sha256:abc"),
            Some("sha256:trust"),
        );

        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--require-trust-root".to_string(),
        ];
        run_cli(&args).expect("lock guard pass");
    }

    #[test]
    fn run_cli_versions_guard_from_lock_mismatch_fails() {
        let root = temp_dir("cli_versions_lock_bad");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_meta(&lock, Some("snap-99"), Some("sha256:abc"), None);

        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_cli(&args).expect_err("lock mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISMATCH");
    }

    #[test]
    fn run_cli_versions_frozen_lock_requires_snapshot_meta_in_lock() {
        let root = temp_dir("cli_versions_lock_frozen");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_meta(&lock, None, None, None);

        let args = vec![
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--frozen-lockfile".to_string(),
        ];
        let err = run_cli(&args).expect_err("frozen lock must require snapshot pins");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISSING");
    }

    #[test]
    fn run_cli_verify_passes() {
        let root = temp_dir("cli_verify_ok");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("verify pass");
    }

    #[test]
    fn run_cli_verify_prefers_non_yanked_when_duplicate_pin_exists() {
        let root = temp_dir("cli_verify_prefers_non_yanked_duplicate");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": true
                },
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": false
                }
            ]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--deny-yanked-locked".to_string(),
        ];
        run_cli(&args).expect("must prefer non-yanked duplicate and pass");
    }

    #[test]
    fn run_cli_verify_duplicate_pin_reports_yanked_index_zero() {
        let root = temp_dir("cli_verify_dup_pin_report_counts");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": true
                },
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": false
                }
            ]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("verify should pass");

        let report_path = lock.with_extension("verify.report.json");
        let report_text = fs::read_to_string(&report_path).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(report.get("packages").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(report.get("matched").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(report.get("yanked_lock").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(report.get("yanked_index").and_then(|v| v.as_u64()), Some(0));
    }

    #[test]
    fn run_cli_verify_duplicate_pin_same_state_archive_pin_is_order_independent() {
        let root = temp_dir("cli_verify_dup_pin_same_state_archive_order_independent");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "archive_sha256": "sha256:b",
                    "download_url": "https://registry/b",
                    "summary": "나",
                    "yanked": false
                },
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "archive_sha256": "sha256:a",
                    "download_url": "https://registry/a",
                    "summary": "가",
                    "yanked": false
                }
            ]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "archive_sha256": "sha256:a",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("archive pin should match deterministic duplicate choice");
    }

    #[test]
    fn run_cli_verify_duplicate_pin_prefers_contract_matched_entry() {
        let root = temp_dir("cli_verify_dup_pin_prefers_contract_match");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "contract": "D-APPROX",
                    "yanked": false
                },
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "contract": "D-STRICT",
                    "yanked": false
                }
            ]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "contract": "D-STRICT",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("contract pin should prefer matched duplicate entry");
    }

    #[test]
    fn run_cli_verify_duplicate_pin_prefers_dependencies_matched_entry() {
        let root = temp_dir("cli_verify_dup_pin_prefers_dependencies_match");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "dependencies": {
                        "표준/벡터": "20.6.0"
                    },
                    "yanked": false
                },
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "dependencies": {
                        "표준/힘": "20.6.0",
                        "표준/벡터": "20.6.0"
                    },
                    "yanked": false
                }
            ]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "dependencies": {
                    "표준/벡터": "20.6.0",
                    "표준/힘": "20.6.0"
                },
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("dependencies pin should prefer matched duplicate entry");
    }

    #[test]
    fn run_cli_verify_duplicate_pin_prefers_higher_pin_match_score() {
        let root = temp_dir("cli_verify_dup_pin_prefers_higher_pin_match_score");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "contract": "D-STRICT",
                    "min_runtime": "v20.6.29",
                    "yanked": false
                },
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "contract": "D-STRICT",
                    "min_runtime": "v20.6.30",
                    "yanked": false
                }
            ]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "contract": "D-STRICT",
                "min_runtime": "v20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("verify should prefer duplicate entry with higher pin-match score");
    }

    #[test]
    fn run_cli_verify_duplicate_pin_prioritizes_non_yanked_over_higher_score() {
        let root = temp_dir("cli_verify_dup_pin_non_yanked_over_higher_score");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "contract": "D-STRICT",
                    "yanked": true
                },
                {
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "contract": "D-APPROX",
                    "yanked": false
                }
            ]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "contract": "D-STRICT",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("non-yanked must be selected before score");
        assert_diag_with_fix(&err, "E_REG_CONTRACT_MISMATCH");
    }

    #[test]
    fn run_cli_verify_duplicate_pin_score_tie_is_order_independent_for_diag() {
        let root = temp_dir("cli_verify_dup_pin_score_tie_diag_order_independent");
        let index_a = root.join("index_a.json");
        let index_b = root.join("index_b.json");
        let lock = root.join("ddn.lock");
        let entry_1 = json!({
            "schema": "ddn.registry.index_entry.v1",
            "scope": "표준",
            "name": "역학",
            "version": "20.6.30",
            "min_runtime": "v20.6.29",
            "yanked": false
        });
        let entry_2 = json!({
            "schema": "ddn.registry.index_entry.v1",
            "scope": "표준",
            "name": "역학",
            "version": "20.6.30",
            "min_runtime": "v20.6.30",
            "yanked": false
        });
        let snapshot_a = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [entry_1.clone(), entry_2.clone()]
        });
        let snapshot_b = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [entry_2, entry_1]
        });
        fs::write(
            &index_a,
            serde_json::to_string_pretty(&snapshot_a).expect("json"),
        )
        .expect("write");
        fs::write(
            &index_b,
            serde_json::to_string_pretty(&snapshot_b).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "min_runtime": "v99.0.0",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args_a = vec![
            "verify".to_string(),
            "--index".to_string(),
            index_a.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let args_b = vec![
            "verify".to_string(),
            "--index".to_string(),
            index_b.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err_a = run_cli(&args_a).expect_err("min_runtime mismatch expected");
        let err_b = run_cli(&args_b).expect_err("min_runtime mismatch expected");
        assert_eq!(
            err_a, err_b,
            "score-tie diagnostics must be order independent"
        );
    }

    #[test]
    fn run_cli_verify_empty_packages_writes_zero_counts_report() {
        let root = temp_dir("cli_verify_empty_packages_report");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(&lock, json!([]), Some("snap-42"), Some("sha256:abc"));

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("verify with empty packages should pass");

        let report_path = lock.with_extension("verify.report.json");
        let report_text = fs::read_to_string(&report_path).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(report.get("packages").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(report.get("matched").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(report.get("yanked_lock").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(report.get("yanked_index").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(
            report
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    #[test]
    fn run_cli_verify_lock_package_id_invalid_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_id");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid lock package id must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn run_cli_verify_lock_package_id_with_extra_slash_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_id_extra_slash");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학/추가",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid lock package id with extra slash must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn run_cli_verify_lock_package_id_with_empty_scope_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_id_empty_scope");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid lock package id with empty scope must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn run_cli_verify_lock_package_id_with_empty_name_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_id_empty_name");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid lock package id with empty name must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn run_cli_verify_lock_package_id_with_scope_space_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_id_scope_space");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": " 표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid lock package id with scope space must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn run_cli_verify_lock_package_id_with_name_space_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_id_name_space");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/ 역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid lock package id with name space must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn run_cli_verify_lock_package_id_with_scope_inner_tab_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_id_scope_inner_tab");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표\t준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("inner tab in scope must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn run_cli_verify_lock_package_id_with_name_inner_newline_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_id_name_inner_newline");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역\n학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("inner newline in name must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn run_cli_verify_lock_package_id_non_string_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_id_non_string");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": 123,
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("non-string id in lock package must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("id"));
    }

    #[test]
    fn run_cli_verify_lock_package_version_non_string_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_version_non_string");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": 20630,
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("non-string version in lock package must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn run_cli_verify_lock_package_version_empty_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_version_empty");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("empty version in lock package must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn run_cli_verify_lock_package_version_with_space_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_bad_version_space");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": " 20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("spaced version in lock package must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn run_cli_verify_lock_schema_invalid_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_schema_bad");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        let lock_json = json!({
            "schema_version": "v0",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid lock schema must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_SCHEMA");
    }

    #[test]
    fn run_cli_verify_lock_packages_not_array_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_packages_not_array");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": {}
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("non-array lock packages must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGES");
    }

    #[test]
    fn run_cli_verify_lock_packages_missing_fails_with_fix() {
        let root = temp_dir("cli_verify_lock_packages_missing");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        let lock_json = json!({
            "schema_version": "v1"
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("missing lock packages must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGES");
    }

    #[test]
    fn run_cli_verify_missing_index_entry_fails() {
        let root = temp_dir("cli_verify_missing");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("missing must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");
    }

    #[test]
    fn run_cli_verify_deny_yanked_locked_fails() {
        let root = temp_dir("cli_verify_yanked");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": true
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--deny-yanked-locked".to_string(),
        ];
        let err = run_cli(&args).expect_err("deny yanked must fail");
        assert_diag_with_fix(&err, "E_REG_YANKED_LOCKED");
    }

    #[test]
    fn run_cli_verify_writes_report_json() {
        let root = temp_dir("cli_verify_report");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let out = root.join("verify.report.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("verify with out");

        let report_text = fs::read_to_string(&out).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_verify_report_contract(&report, 1, 1);
    }

    #[test]
    fn run_cli_verify_out_parent_file_fails_with_report_write() {
        let root = temp_dir("cli_verify_out_parent_file");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let out_parent = root.join("blocked");
        let out = out_parent.join("verify.report.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        fs::write(&out_parent, "file blocks directory").expect("write parent file");

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("report out parent file must fail");
        assert_diag_with_fix(&err, "E_REG_REPORT_WRITE");
        assert!(!out.exists());
    }

    #[test]
    fn run_cli_verify_with_audit_verify_out_write_failure_skips_audit_step() {
        let root = temp_dir("cli_verify_with_audit_verify_out_write_fail");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let audit = root.join("audit.jsonl");
        let verify_out_parent = root.join("blocked_verify_out");
        let verify_out = verify_out_parent.join("verify.report.json");
        let audit_out = root.join("audit.verify.report.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        write_valid_audit_log(&audit);
        fs::write(&verify_out_parent, "file blocks directory").expect("write parent file");

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            verify_out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("verify out write failure must fail");
        assert_diag_with_fix(&err, "E_REG_REPORT_WRITE");
        assert!(!verify_out.exists());
        assert!(
            !audit_out.exists(),
            "audit step should be skipped when verify report write already failed"
        );
    }

    #[test]
    fn run_cli_verify_writes_default_report_json() {
        let root = temp_dir("cli_verify_report_default");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("verify default out");

        let default_report = lock.with_extension("verify.report.json");
        let report_text = fs::read_to_string(default_report).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_verify_report_contract(&report, 1, 1);
    }

    #[test]
    fn run_cli_verify_with_audit_requires_log() {
        let root = temp_dir("cli_verify_need_audit_log");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
        ];
        let err = run_cli(&args).expect_err("verify-audit needs audit log");
        assert_diag_with_fix(&err, "E_REG_AUDIT_VERIFY_LOG_REQUIRED");
    }

    #[test]
    fn run_cli_verify_with_audit_parse_failure_writes_only_verify_report() {
        let root = temp_dir("cli_verify_with_audit_parse_failure_reports");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let audit = root.join("audit.jsonl");
        let verify_out = root.join("verify.report.json");
        let audit_out = root.join("audit.verify.report.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        fs::write(&audit, "{ not-json").expect("write bad audit");

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            verify_out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("invalid audit json must fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_PARSE");

        assert!(
            verify_out.exists(),
            "verify report should be written before audit step fails"
        );
        assert!(
            !audit_out.exists(),
            "audit report must not be written on audit parse failure"
        );
        let verify_report_text = fs::read_to_string(&verify_out).expect("read verify report");
        let verify_report: Value =
            serde_json::from_str(&verify_report_text).expect("parse verify report");
        assert_verify_report_contract(&verify_report, 1, 1);
    }

    #[test]
    fn run_cli_verify_with_audit_verify_failure_writes_no_reports() {
        let root = temp_dir("cli_verify_with_audit_verify_failure_no_reports");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let audit = root.join("audit.jsonl");
        let verify_out = root.join("verify.report.json");
        let audit_out = root.join("audit.verify.report.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": []
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        write_valid_audit_log(&audit);

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            verify_out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("verify stage must fail before audit");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");

        assert!(
            !verify_out.exists(),
            "verify report must not be written when verify stage fails"
        );
        assert!(
            !audit_out.exists(),
            "audit report must not be written when verify stage fails"
        );
    }

    #[test]
    fn run_cli_verify_with_audit_writes_audit_report_json() {
        let root = temp_dir("cli_verify_with_audit_report");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let out = root.join("verify.report.json");
        let audit = root.join("audit.jsonl");
        let audit_out = root.join("audit.verify.report.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        write_valid_audit_log(&audit);

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("verify + audit");

        let report_text = fs::read_to_string(&out).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_verify_report_contract(&report, 1, 1);
        let audit_report_text = fs::read_to_string(&audit_out).expect("read audit report");
        let audit_report: Value =
            serde_json::from_str(&audit_report_text).expect("parse audit report");
        assert_audit_verify_report_contract(&audit_report, 2);
    }

    #[test]
    fn run_cli_verify_with_audit_expect_last_hash_mismatch_fails() {
        let root = temp_dir("cli_verify_with_audit_expect_last_hash_bad");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let audit = root.join("audit.jsonl");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        write_valid_audit_log(&audit);

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_cli(&args).expect_err("must fail on expected last hash mismatch");
        assert_audit_last_hash_diag(&err);
    }

    #[test]
    fn run_cli_verify_with_audit_expect_last_hash_mismatch_writes_only_verify_report() {
        let root = temp_dir("cli_verify_with_audit_expect_last_hash_bad_reports");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let audit = root.join("audit.jsonl");
        let verify_out = root.join("verify.report.json");
        let audit_out = root.join("audit.verify.report.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        write_valid_audit_log(&audit);

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            verify_out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_cli(&args).expect_err("must fail on expected hash mismatch");
        assert_audit_last_hash_diag(&err);
        assert!(
            verify_out.exists(),
            "verify report should be written before audit hash check"
        );
        assert!(
            !audit_out.exists(),
            "audit report must not be written on audit hash mismatch"
        );
        let verify_report_text = fs::read_to_string(&verify_out).expect("read verify report");
        let verify_report: Value =
            serde_json::from_str(&verify_report_text).expect("parse verify report");
        assert_verify_report_contract(&verify_report, 1, 1);
    }

    #[test]
    fn run_cli_verify_with_audit_out_parent_file_fails_with_report_write() {
        let root = temp_dir("cli_verify_with_audit_out_parent_file");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let audit = root.join("audit.jsonl");
        let verify_out = root.join("verify.report.json");
        let audit_out_parent = root.join("blocked_audit_out");
        let audit_out = audit_out_parent.join("audit.verify.report.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        write_valid_audit_log(&audit);
        fs::write(&audit_out_parent, "file blocks directory").expect("write parent file");

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            verify_out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("audit out parent file must fail");
        assert_diag_with_fix(&err, "E_REG_REPORT_WRITE");
        assert!(
            verify_out.exists(),
            "verify report should be written before audit report write"
        );
        assert!(!audit_out.exists());
        let verify_report_text = fs::read_to_string(&verify_out).expect("read verify report");
        let verify_report: Value =
            serde_json::from_str(&verify_report_text).expect("parse verify report");
        assert_verify_report_contract(&verify_report, 1, 1);
    }

    #[test]
    fn run_cli_verify_with_audit_expect_last_hash_recovery_passes() {
        let root = temp_dir("cli_verify_with_audit_expect_last_hash_recovery");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let audit = root.join("audit.jsonl");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        write_valid_audit_log(&audit);

        let bad_args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_cli(&bad_args).expect_err("must fail on expected hash mismatch");
        assert_audit_last_hash_diag(&err);

        let good_args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            last_row_hash(&audit),
        ];
        run_cli(&good_args).expect("recovery must pass");
    }

    #[test]
    fn run_cli_verify_with_audit_writes_default_audit_report_path() {
        let root = temp_dir("cli_verify_with_audit_default_out");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        let audit = root.join("audit.jsonl");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-42",
            "index_root_hash": "sha256:abc",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");
        write_lock_with_packages(
            &lock,
            json!([{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]),
            Some("snap-42"),
            Some("sha256:abc"),
        );
        write_valid_audit_log(&audit);

        let args = vec![
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("verify + audit default out");

        let default_out = lock.with_extension("audit.verify.report.json");
        assert!(default_out.exists());
        let verify_out = lock.with_extension("verify.report.json");
        assert!(verify_out.exists());
        let verify_report_text = fs::read_to_string(&verify_out).expect("read verify report");
        let verify_report: Value =
            serde_json::from_str(&verify_report_text).expect("parse verify report");
        assert_eq!(
            verify_report
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
        let audit_report_text = fs::read_to_string(&default_out).expect("read audit report");
        let audit_report: Value =
            serde_json::from_str(&audit_report_text).expect("parse audit report");
        assert_audit_verify_report_contract(&audit_report, 2);
    }

    #[test]
    fn run_cli_verify_contract_mismatch_fails() {
        let root = temp_dir("cli_verify_contract_mismatch");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        write_verify_fixture(
            &index,
            &lock,
            json!({
                "contract": "D-STRICT",
                "min_runtime": "20.6.29",
                "detmath_seal_hash": "sha256:seal-a"
            }),
            json!({
                "contract": "D-SEALED"
            }),
        );
        let args = verify_args(&index, &lock);
        let err = run_cli(&args).expect_err("contract mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_CONTRACT_MISMATCH");
    }

    #[test]
    fn run_cli_verify_min_runtime_mismatch_fails() {
        let root = temp_dir("cli_verify_min_runtime_mismatch");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        write_verify_fixture(
            &index,
            &lock,
            json!({
                "contract": "D-STRICT",
                "min_runtime": "20.6.29",
                "detmath_seal_hash": "sha256:seal-a"
            }),
            json!({
                "min_runtime": "20.6.31"
            }),
        );
        let args = verify_args(&index, &lock);
        let err = run_cli(&args).expect_err("min_runtime mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_MIN_RUNTIME_MISMATCH");
    }

    #[test]
    fn run_cli_verify_detmath_seal_mismatch_fails() {
        let root = temp_dir("cli_verify_detmath_seal_mismatch");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        write_verify_fixture(
            &index,
            &lock,
            json!({
                "contract": "D-STRICT",
                "min_runtime": "20.6.29",
                "detmath_seal_hash": "sha256:seal-a"
            }),
            json!({
                "detmath_seal_hash": "sha256:seal-b"
            }),
        );
        let args = verify_args(&index, &lock);
        let err = run_cli(&args).expect_err("detmath seal mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_DETMATH_SEAL_MISMATCH");
    }

    #[test]
    fn run_cli_verify_archive_sha256_mismatch_fails() {
        let root = temp_dir("cli_verify_archive_sha256_mismatch");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        write_verify_fixture(
            &index,
            &lock,
            json!({
                "archive_sha256": "sha256:archive-a"
            }),
            json!({
                "archive_sha256": "sha256:archive-b"
            }),
        );
        let args = verify_args(&index, &lock);
        let err = run_cli(&args).expect_err("archive sha mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_ARCHIVE_SHA256_MISMATCH");
    }

    #[test]
    fn run_cli_verify_archive_sha256_match_passes() {
        let root = temp_dir("cli_verify_archive_sha256_match");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        write_verify_fixture(
            &index,
            &lock,
            json!({
                "archive_sha256": "sha256:archive-a"
            }),
            json!({
                "archive_sha256": "sha256:archive-a"
            }),
        );
        let args = verify_args(&index, &lock);
        run_cli(&args).expect("archive sha match pass");
    }

    #[test]
    fn run_cli_verify_download_url_mismatch_fails() {
        let root = temp_dir("cli_verify_download_url_mismatch");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        write_verify_fixture(
            &index,
            &lock,
            json!({
                "download_url": "https://registry/a"
            }),
            json!({
                "download_url": "https://registry/b"
            }),
        );
        let args = verify_args(&index, &lock);
        let err = run_cli(&args).expect_err("download_url mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_DOWNLOAD_URL_MISMATCH");
    }

    #[test]
    fn run_cli_download_file_url_passes() {
        let root = temp_dir("cli_download_file_url_passes");
        let index = root.join("index.json");
        let source = root.join("archive").join("pkg.ddn.tar.gz");
        fs::create_dir_all(source.parent().expect("parent")).expect("mkdir");
        let bytes = b"archive-bytes-v1";
        fs::write(&source, bytes).expect("write source");
        let expected_sha = sha256_hex_prefixed(bytes);

        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-download-pass",
            "index_root_hash": "sha256:index-download-pass",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": expected_sha,
                "download_url": source.to_string_lossy(),
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("out").join("pkg.ddn.tar.gz");
        let args = vec![
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("download pass");

        let copied = fs::read(&out).expect("read copied");
        assert_eq!(copied, bytes);
    }

    #[test]
    fn run_cli_download_archive_sha256_mismatch_fails() {
        let root = temp_dir("cli_download_archive_sha_mismatch");
        let index = root.join("index.json");
        let source = root.join("archive").join("pkg.ddn.tar.gz");
        fs::create_dir_all(source.parent().expect("parent")).expect("mkdir");
        fs::write(&source, b"archive-bytes-v1").expect("write source");

        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-download-sha-mismatch",
            "index_root_hash": "sha256:index-download-sha-mismatch",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": "sha256:not-match",
                "download_url": source.to_string_lossy(),
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("out").join("pkg.ddn.tar.gz");
        let args = vec![
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("sha mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_ARCHIVE_SHA256_MISMATCH");
    }

    #[test]
    fn run_cli_download_http_requires_allow_http() {
        let root = temp_dir("cli_download_http_scheme");
        let index = root.join("index.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-download-http",
            "index_root_hash": "sha256:index-download-http",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": "sha256:any",
                "download_url": "https://registry.example/physics-20.6.30.ddn.tar.gz",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("out").join("pkg.ddn.tar.gz");
        let args = vec![
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_cli(&args).expect_err("http download must require explicit flag");
        assert_diag_with_fix(&err, "E_REG_DOWNLOAD_HTTP_DISABLED");
    }

    #[test]
    fn run_cli_download_http_allow_http_passes() {
        let root = temp_dir("cli_download_http_allowed");
        let index = root.join("index.json");
        let bytes = b"archive-http-v1";
        let expected_sha = sha256_hex_prefixed(bytes);
        let download_url = start_http_fixture(bytes);

        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-download-http-ok",
            "index_root_hash": "sha256:index-download-http-ok",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": expected_sha,
                "download_url": download_url,
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("out").join("pkg.ddn.tar.gz");
        let args = vec![
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--allow-http".to_string(),
        ];
        run_cli(&args).expect("http download should pass with allow flag");

        let copied = fs::read(&out).expect("read copied");
        assert_eq!(copied, bytes);
    }

    #[test]
    fn run_cli_download_offline_cache_hit_passes() {
        let root = temp_dir("cli_download_offline_cache_hit");
        let index = root.join("index.json");
        let cache_dir = root.join("cache");
        let bytes = b"archive-cache-hit-v1";
        let expected_sha = sha256_hex_prefixed(bytes);
        let cache_path = cache_dir
            .join("blobs")
            .join("sha256")
            .join(expected_sha.trim_start_matches("sha256:"));
        fs::create_dir_all(cache_path.parent().expect("cache parent")).expect("mkdir");
        fs::write(&cache_path, bytes).expect("write cache blob");

        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-download-offline-hit",
            "index_root_hash": "sha256:index-download-offline-hit",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": expected_sha,
                "download_url": "https://registry.example/physics-20.6.30.ddn.tar.gz",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("out").join("pkg.ddn.tar.gz");
        let args = vec![
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--cache-dir".to_string(),
            cache_dir.to_string_lossy().to_string(),
            "--offline".to_string(),
        ];
        run_cli(&args).expect("offline cache hit should pass");

        let copied = fs::read(&out).expect("read copied");
        assert_eq!(copied, bytes);
    }

    #[test]
    fn run_cli_download_offline_cache_miss_fails() {
        let root = temp_dir("cli_download_offline_cache_miss");
        let index = root.join("index.json");
        let cache_dir = root.join("cache");
        let expected_sha = sha256_hex_prefixed(b"archive-cache-miss-v1");

        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-download-offline-miss",
            "index_root_hash": "sha256:index-download-offline-miss",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": expected_sha.clone(),
                "download_url": "https://registry.example/physics-20.6.30.ddn.tar.gz",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("out").join("pkg.ddn.tar.gz");
        let args = vec![
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--cache-dir".to_string(),
            cache_dir.to_string_lossy().to_string(),
            "--offline".to_string(),
        ];
        let err = run_cli(&args).expect_err("offline cache miss must fail");
        assert_diag_with_fix(&err, "E_CACHE_UNAVAILABLE_OFFLINE");
    }

    #[test]
    fn run_cli_download_online_writes_cache_blob() {
        let root = temp_dir("cli_download_online_cache_write");
        let index = root.join("index.json");
        let source = root.join("archive").join("pkg.ddn.tar.gz");
        let cache_dir = root.join("cache");
        fs::create_dir_all(source.parent().expect("parent")).expect("mkdir");
        let bytes = b"archive-online-cache-v1";
        fs::write(&source, bytes).expect("write source");
        let expected_sha = sha256_hex_prefixed(bytes);

        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-download-online-cache",
            "index_root_hash": "sha256:index-download-online-cache",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": expected_sha.clone(),
                "download_url": source.to_string_lossy(),
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("out").join("pkg.ddn.tar.gz");
        let args = vec![
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--cache-dir".to_string(),
            cache_dir.to_string_lossy().to_string(),
        ];
        run_cli(&args).expect("online download should write cache");

        let cache_path = cache_dir
            .join("blobs")
            .join("sha256")
            .join(expected_sha.trim_start_matches("sha256:"));
        let cached = fs::read(cache_path).expect("read cache blob");
        assert_eq!(cached, bytes);
    }

    #[test]
    fn run_cli_download_yanked_requires_include_yanked() {
        let root = temp_dir("cli_download_yanked_guard");
        let index = root.join("index.json");
        let source = root.join("archive").join("pkg.ddn.tar.gz");
        fs::create_dir_all(source.parent().expect("parent")).expect("mkdir");
        let bytes = b"archive-bytes-v1";
        fs::write(&source, bytes).expect("write source");
        let expected_sha = sha256_hex_prefixed(bytes);

        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-download-yanked",
            "index_root_hash": "sha256:index-download-yanked",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": expected_sha,
                "download_url": source.to_string_lossy(),
                "yanked": true
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("out").join("pkg.ddn.tar.gz");
        let base_args = vec![
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_cli(&base_args).expect_err("yanked download must fail by default");
        assert_diag_with_fix(&err, "E_REG_YANKED_LOCKED");

        let mut include_args = base_args.clone();
        include_args.push("--include-yanked".to_string());
        run_cli(&include_args).expect("include-yanked should pass");
        let copied = fs::read(&out).expect("read copied");
        assert_eq!(copied, bytes);
    }

    #[test]
    fn run_cli_verify_dependencies_mismatch_fails() {
        let root = temp_dir("cli_verify_dependencies_mismatch");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        write_verify_fixture(
            &index,
            &lock,
            json!({
                "dependencies": {"표준/수학": "1.0.0"}
            }),
            json!({
                "dependencies": {"표준/수학": "2.0.0"}
            }),
        );
        let args = verify_args(&index, &lock);
        let err = run_cli(&args).expect_err("dependencies mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_DEPENDENCIES_MISMATCH");
    }

    #[test]
    fn run_cli_verify_dependencies_match_with_different_key_order() {
        let root = temp_dir("cli_verify_dependencies_order");
        let index = root.join("index.json");
        let lock = root.join("ddn.lock");
        write_verify_fixture(
            &index,
            &lock,
            json!({
                "dependencies": {"a": 1, "b": 2}
            }),
            json!({
                "dependencies": {"b": 2, "a": 1}
            }),
        );
        let args = verify_args(&index, &lock);
        run_cli(&args).expect("dependencies order-insensitive match pass");
    }

    #[test]
    fn error_code_from_extracts_first_token() {
        assert_eq!(
            error_code_from("E_REG_AUTH_POLICY_SCHEMA detail fix=..."),
            "E_REG_AUTH_POLICY_SCHEMA"
        );
    }

    #[test]
    fn error_code_from_empty_input_returns_unknown() {
        assert_eq!(error_code_from(""), "E_REG_UNKNOWN");
        assert_eq!(error_code_from("   "), "E_REG_UNKNOWN");
    }
}
