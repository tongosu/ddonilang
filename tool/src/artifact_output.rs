use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

static ARTIFACT_OUTPUT_NONCE: AtomicU64 = AtomicU64::new(0);

pub fn require_file_artifact_target(path: &Path) -> Result<(), String> {
    if path.exists() && !path.is_file() {
        return Err(format!(
            "E_SHARED_ARTIFACT_OUTPUT_TARGET_NOT_FILE {}",
            path.display()
        ));
    }
    Ok(())
}

pub fn write_text_artifact_atomic(path: &Path, text: &str) -> Result<(), String> {
    write_binary_artifact(path, text.as_bytes(), false)
}

pub fn write_binary_artifact_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    write_binary_artifact(path, bytes, false)
}

pub fn write_text_artifacts_atomic(outputs: &[(&Path, &str)]) -> Result<(), String> {
    write_text_artifacts(outputs, None)
}

pub struct ArtifactPayload<'a> {
    pub target: &'a Path,
    pub bytes: &'a [u8],
}

pub fn write_artifact_set_atomic(outputs: &[ArtifactPayload<'_>]) -> Result<(), String> {
    write_artifact_set(outputs, None)
}

/// Validate every file target before a product workflow starts executing.
///
/// This is deliberately side-effect free: it checks target kind, lexical
/// duplicate/overlap, and any already-present staging siblings without
/// creating parent directories or temporary files.  The writer repeats the
/// same checks immediately before publication as its final safety boundary.
pub fn preflight_artifact_targets(targets: &[&Path]) -> Result<(), String> {
    let mut unique_targets = HashMap::with_capacity(targets.len());
    for target in targets {
        ensure_unique_target(&mut unique_targets, target)?;
        require_file_artifact_target(target)?;
        if artifact_set_staging_exists(target)? {
            return Err(format!(
                "E_SHARED_ARTIFACT_OUTPUT_STAGING_CONFLICT {}",
                target.display()
            ));
        }
    }
    Ok(())
}

pub fn write_directory_artifact_atomic<F>(target: &Path, populate: F) -> Result<(), String>
where
    F: FnOnce(&Path) -> Result<(), String>,
{
    write_directory_artifact(target, populate, false)
}

struct PreparedArtifactSetOutput {
    target: PathBuf,
    temp: PathBuf,
    backup: PathBuf,
    had_existing: bool,
}

fn write_text_artifacts(
    outputs: &[(&Path, &str)],
    fail_before_publish_index: Option<usize>,
) -> Result<(), String> {
    let payloads = outputs
        .iter()
        .map(|(target, text)| ArtifactPayload {
            target,
            bytes: text.as_bytes(),
        })
        .collect::<Vec<_>>();
    write_artifact_set(&payloads, fail_before_publish_index)
}

fn write_artifact_set(
    outputs: &[ArtifactPayload<'_>],
    fail_before_publish_index: Option<usize>,
) -> Result<(), String> {
    let mut unique_targets = HashMap::with_capacity(outputs.len());
    for output in outputs {
        ensure_unique_target(&mut unique_targets, output.target)?;
        require_file_artifact_target(output.target)?;
    }

    let nonce = ARTIFACT_OUTPUT_NONCE.fetch_add(1, Ordering::Relaxed);
    let mut prepared = Vec::with_capacity(outputs.len());
    for (index, output) in outputs.iter().enumerate() {
        let target = output.target;
        if let Some(parent) = target
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            if let Err(error) = fs::create_dir_all(parent) {
                cleanup_artifact_set(&prepared);
                return Err(error.to_string());
            }
        }

        let temp = match artifact_set_sibling_path(target, "tmp", nonce, index) {
            Ok(path) => path,
            Err(error) => {
                cleanup_artifact_set(&prepared);
                return Err(error);
            }
        };
        let backup = match artifact_set_sibling_path(target, "backup", nonce, index) {
            Ok(path) => path,
            Err(error) => {
                cleanup_artifact_set(&prepared);
                return Err(error);
            }
        };
        if temp.exists() || backup.exists() {
            cleanup_artifact_set(&prepared);
            return Err(format!(
                "E_SHARED_ARTIFACT_OUTPUT_STAGING_CONFLICT {}",
                target.display()
            ));
        }

        let mut output = match OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(output) => output,
            Err(error) => {
                cleanup_artifact_set(&prepared);
                return Err(error.to_string());
            }
        };
        if let Err(error) = output
            .write_all(outputs[index].bytes)
            .and_then(|_| output.sync_all())
        {
            drop(output);
            let _ = fs::remove_file(&temp);
            cleanup_artifact_set(&prepared);
            return Err(error.to_string());
        }
        drop(output);
        prepared.push(PreparedArtifactSetOutput {
            target: (*target).to_path_buf(),
            temp,
            backup,
            had_existing: target.exists(),
        });
    }

    let mut backed_up = 0;
    for item in &prepared {
        if item.had_existing {
            if let Err(error) = fs::rename(&item.target, &item.backup) {
                rollback_artifact_set(&prepared, backed_up, 0);
                return Err(error.to_string());
            }
        }
        backed_up += 1;
    }

    let mut published = 0;
    for (index, item) in prepared.iter().enumerate() {
        if fail_before_publish_index == Some(index) {
            rollback_artifact_set(&prepared, backed_up, published);
            return Err(format!(
                "injected failure before artifact-set publish index={index}"
            ));
        }
        if let Err(error) = fs::rename(&item.temp, &item.target) {
            rollback_artifact_set(&prepared, backed_up, published);
            return Err(error.to_string());
        }
        published += 1;
    }

    for item in &prepared {
        if item.backup.exists() {
            fs::remove_file(&item.backup).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn ensure_unique_target(
    unique_targets: &mut HashMap<String, PathBuf>,
    target: &Path,
) -> Result<(), String> {
    let key = normalized_target_key(target);
    if let Some(previous) = unique_targets.insert(key, target.to_path_buf()) {
        let code = if previous == target {
            "E_SHARED_ARTIFACT_OUTPUT_DUPLICATE_TARGET"
        } else {
            "E_SHARED_ARTIFACT_OUTPUT_OVERLAPPING_TARGET"
        };
        return Err(format!("{code} {}", target.display()));
    }
    Ok(())
}

fn normalized_target_key(path: &Path) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    let mut key = normalized.to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    {
        key.make_ascii_lowercase();
    }
    key
}

fn artifact_set_staging_exists(target: &Path) -> Result<bool, String> {
    let Some(parent) = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(false);
    };
    if !parent.exists() {
        return Ok(false);
    }
    let Some(file_name) = target.file_name().and_then(|name| name.to_str()) else {
        return Err("artifact target needs a UTF-8 file name".to_string());
    };
    let prefix = format!(".{file_name}.ddn-artifact-set-");
    let entries = fs::read_dir(parent).map_err(|error| {
        format!(
            "E_SHARED_ARTIFACT_OUTPUT_STAGING_SCAN {} {error}",
            parent.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "E_SHARED_ARTIFACT_OUTPUT_STAGING_SCAN {} {error}",
                parent.display()
            )
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(&prefix) && (name.ends_with(".tmp") || name.ends_with(".backup")) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn artifact_set_sibling_path(
    target: &Path,
    suffix: &str,
    nonce: u64,
    index: usize,
) -> Result<PathBuf, String> {
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "artifact target needs a UTF-8 file name".to_string())?;
    let parent = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Ok(parent.join(format!(
        ".{file_name}.ddn-artifact-set-{}-{nonce}-{index}.{suffix}",
        process::id()
    )))
}

fn cleanup_artifact_set(prepared: &[PreparedArtifactSetOutput]) {
    for item in prepared {
        if item.temp.exists() {
            let _ = fs::remove_file(&item.temp);
        }
    }
}

fn rollback_artifact_set(
    prepared: &[PreparedArtifactSetOutput],
    backed_up: usize,
    published: usize,
) {
    for item in prepared.iter().take(published) {
        if item.target.exists() {
            let _ = fs::remove_file(&item.target);
        }
    }
    for item in prepared.iter().take(backed_up) {
        if item.had_existing && item.backup.exists() {
            let _ = fs::rename(&item.backup, &item.target);
        }
    }
    cleanup_artifact_set(prepared);
}

fn write_binary_artifact(
    path: &Path,
    bytes: &[u8],
    fail_before_replace: bool,
) -> Result<(), String> {
    require_file_artifact_target(path)?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let temp = artifact_temp_path(path)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| error.to_string())?;
    if let Err(error) = output.write_all(bytes).and_then(|_| output.sync_all()) {
        drop(output);
        let _ = fs::remove_file(&temp);
        return Err(error.to_string());
    }
    drop(output);

    if fail_before_replace {
        let _ = fs::remove_file(&temp);
        return Err("injected failure before atomic replacement".to_string());
    }

    if let Err(error) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(error.to_string());
    }
    Ok(())
}

fn artifact_temp_path(path: &Path) -> Result<PathBuf, String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "artifact target needs a UTF-8 file name".to_string())?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let nonce = ARTIFACT_OUTPUT_NONCE.fetch_add(1, Ordering::Relaxed);
    Ok(parent.join(format!(
        ".{file_name}.ddn-artifact-{}-{nonce}.tmp",
        process::id()
    )))
}

fn write_directory_artifact<F>(
    target: &Path,
    populate: F,
    fail_before_publish: bool,
) -> Result<(), String>
where
    F: FnOnce(&Path) -> Result<(), String>,
{
    if target.exists() && !target.is_dir() {
        return Err(format!(
            "E_SHARED_ARTIFACT_OUTPUT_TARGET_NOT_DIRECTORY {}",
            target.display()
        ));
    }
    let parent = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "artifact target needs a UTF-8 directory name".to_string())?;
    let nonce = ARTIFACT_OUTPUT_NONCE.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".{file_name}.ddn-directory-artifact-{}-{nonce}.staging",
        process::id()
    ));
    let backup = parent.join(format!(
        ".{file_name}.ddn-directory-artifact-{}-{nonce}.backup",
        process::id()
    ));
    if staging.exists() || backup.exists() {
        return Err(format!(
            "E_SHARED_ARTIFACT_OUTPUT_STAGING_CONFLICT {}",
            target.display()
        ));
    }

    fs::create_dir(&staging).map_err(|error| error.to_string())?;
    if let Err(error) = populate(&staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let had_existing = target.exists();
    if had_existing {
        if let Err(error) = fs::rename(target, &backup) {
            let _ = fs::remove_dir_all(&staging);
            return Err(error.to_string());
        }
    }

    if fail_before_publish {
        if had_existing {
            let _ = fs::rename(&backup, target);
        }
        let _ = fs::remove_dir_all(&staging);
        return Err("injected failure before directory artifact publish".to_string());
    }

    if let Err(error) = fs::rename(&staging, target) {
        if had_existing {
            let _ = fs::rename(&backup, target);
        }
        let _ = fs::remove_dir_all(&staging);
        return Err(error.to_string());
    }

    if had_existing {
        if let Err(error) = fs::remove_dir_all(&backup) {
            let _ = fs::remove_dir_all(target);
            let _ = fs::rename(&backup, target);
            return Err(error.to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        preflight_artifact_targets, write_binary_artifact, write_directory_artifact,
        write_text_artifacts,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "ddn_artifact_output_{label}_{}_{}",
            process::id(),
            nonce
        ))
    }

    #[test]
    fn failure_before_replace_preserves_existing_artifact() {
        let dir = test_dir("preserve");
        fs::create_dir_all(&dir).expect("create test dir");
        let target = dir.join("result.detjson");
        fs::write(&target, "old\n").expect("write predecessor");

        let error = write_binary_artifact(&target, b"new\n", true)
            .expect_err("injected failure must fail closed");
        assert!(error.contains("injected failure"));
        assert_eq!(
            fs::read_to_string(&target).expect("read predecessor"),
            "old\n"
        );

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn success_replaces_existing_artifact_without_temp_residue() {
        let dir = test_dir("replace");
        fs::create_dir_all(&dir).expect("create test dir");
        let target = dir.join("result.detjson");
        fs::write(&target, "old\n").expect("write predecessor");

        write_binary_artifact(&target, b"new\n", false).expect("atomic replacement");
        assert_eq!(fs::read_to_string(&target).expect("read result"), "new\n");
        assert_eq!(fs::read_dir(&dir).expect("read test dir").count(), 1);

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn artifact_set_preflight_preserves_all_predecessors() {
        let dir = test_dir("set_preflight");
        fs::create_dir_all(&dir).expect("create test dir");
        let first = dir.join("first.detjson");
        let second = dir.join("second.detjson");
        fs::write(&first, "old-first\n").expect("write first predecessor");
        fs::create_dir(&second).expect("create invalid second target");

        let error =
            write_text_artifacts(&[(&first, "new-first\n"), (&second, "new-second\n")], None)
                .expect_err("directory target must fail before publication");
        assert!(
            error.contains("E_SHARED_ARTIFACT_OUTPUT_TARGET_NOT_FILE"),
            "{error}"
        );
        assert_eq!(
            fs::read_to_string(&first).expect("read first predecessor"),
            "old-first\n"
        );
        assert!(second.is_dir());

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn artifact_set_publish_failure_rolls_back_every_target() {
        let dir = test_dir("set_rollback");
        fs::create_dir_all(&dir).expect("create test dir");
        let first = dir.join("first.detjson");
        let second = dir.join("second.detjson");
        fs::write(&first, "old-first\n").expect("write first predecessor");
        fs::write(&second, "old-second\n").expect("write second predecessor");

        let error = write_text_artifacts(
            &[(&first, "new-first\n"), (&second, "new-second\n")],
            Some(1),
        )
        .expect_err("injected second publication failure must roll back");
        assert!(error.contains("injected failure"), "{error}");
        assert_eq!(
            fs::read_to_string(&first).expect("read first predecessor"),
            "old-first\n"
        );
        assert_eq!(
            fs::read_to_string(&second).expect("read second predecessor"),
            "old-second\n"
        );
        assert_eq!(fs::read_dir(&dir).expect("read test dir").count(), 2);

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn artifact_set_success_replaces_all_targets_without_residue() {
        let dir = test_dir("set_success");
        fs::create_dir_all(&dir).expect("create test dir");
        let first = dir.join("first.detjson");
        let second = dir.join("second.detjson");
        fs::write(&first, "old-first\n").expect("write first predecessor");

        write_text_artifacts(&[(&first, "new-first\n"), (&second, "new-second\n")], None)
            .expect("atomic artifact-set replacement");
        assert_eq!(
            fs::read_to_string(&first).expect("read first result"),
            "new-first\n"
        );
        assert_eq!(
            fs::read_to_string(&second).expect("read second result"),
            "new-second\n"
        );
        assert_eq!(fs::read_dir(&dir).expect("read test dir").count(), 2);

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn preflight_rejects_normalized_duplicate_targets_before_execution() {
        let dir = test_dir("normalized_duplicate");
        fs::create_dir_all(&dir).expect("create test dir");
        let first = dir.join("nested").join("result.detjson");
        let second = dir.join("nested").join(".").join("result.detjson");
        let error = preflight_artifact_targets(&[&first, &second])
            .expect_err("normalized duplicate must fail closed");
        assert!(
            error.contains("E_SHARED_ARTIFACT_OUTPUT_OVERLAPPING_TARGET")
                || error.contains("E_SHARED_ARTIFACT_OUTPUT_DUPLICATE_TARGET"),
            "{error}"
        );
        assert!(!first.exists());
        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn directory_artifact_populate_failure_preserves_predecessor() {
        let dir = test_dir("directory_populate_failure");
        let target = dir.join("vendor");
        fs::create_dir_all(&target).expect("create predecessor");
        fs::write(target.join("previous.txt"), "old\n").expect("write predecessor");

        let error = write_directory_artifact(
            &target,
            |staging| {
                fs::write(staging.join("partial.txt"), "partial\n")
                    .map_err(|error| error.to_string())?;
                Err("injected populate failure".to_string())
            },
            false,
        )
        .expect_err("populate failure must fail closed");
        assert!(error.contains("injected populate failure"), "{error}");
        assert_eq!(
            fs::read_to_string(target.join("previous.txt")).expect("read predecessor"),
            "old\n"
        );
        assert!(!target.join("partial.txt").exists());
        assert_eq!(fs::read_dir(&dir).expect("read test dir").count(), 1);

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn directory_artifact_publish_failure_restores_predecessor() {
        let dir = test_dir("directory_publish_failure");
        let target = dir.join("vendor");
        fs::create_dir_all(&target).expect("create predecessor");
        fs::write(target.join("previous.txt"), "old\n").expect("write predecessor");

        let error = write_directory_artifact(
            &target,
            |staging| {
                fs::write(staging.join("new.txt"), "new\n").map_err(|error| error.to_string())
            },
            true,
        )
        .expect_err("publish failure must fail closed");
        assert!(error.contains("injected failure"), "{error}");
        assert_eq!(
            fs::read_to_string(target.join("previous.txt")).expect("read predecessor"),
            "old\n"
        );
        assert!(!target.join("new.txt").exists());
        assert_eq!(fs::read_dir(&dir).expect("read test dir").count(), 1);

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn directory_artifact_success_replaces_predecessor_without_residue() {
        let dir = test_dir("directory_success");
        let target = dir.join("vendor");
        fs::create_dir_all(&target).expect("create predecessor");
        fs::write(target.join("previous.txt"), "old\n").expect("write predecessor");

        write_directory_artifact(
            &target,
            |staging| {
                fs::write(staging.join("new.txt"), "new\n").map_err(|error| error.to_string())
            },
            false,
        )
        .expect("directory replacement");
        assert!(!target.join("previous.txt").exists());
        assert_eq!(
            fs::read_to_string(target.join("new.txt")).expect("read result"),
            "new\n"
        );
        assert_eq!(fs::read_dir(&dir).expect("read test dir").count(), 1);

        fs::remove_dir_all(&dir).expect("remove test dir");
    }
}
