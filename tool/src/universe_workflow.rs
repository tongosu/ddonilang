use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::artifact_output::{
    write_artifact_set_atomic, write_binary_artifact_atomic, ArtifactPayload,
};

pub fn run_pack(input_dir: &Path, out_file: &Path) -> Result<(), String> {
    if !input_dir.is_dir() {
        return Err(format!(
            "E_UNIVERSE_PACK_INPUT_DIR input dir not found: {}",
            input_dir.display()
        ));
    }

    let files = collect_files(input_dir)?;
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(fixed_zip_datetime())
        .unix_permissions(0o644);

    for rel in &files {
        let rel_unix = path_to_unix(rel)?;
        let source = input_dir.join(rel);
        let bytes = fs::read(&source).map_err(|error| {
            format!(
                "E_UNIVERSE_PACK_READ read source failed {} ({})",
                source.display(),
                error
            )
        })?;
        zip.start_file(rel_unix, options)
            .map_err(|error| format!("E_UNIVERSE_PACK_ZIP_WRITE {error}"))?;
        zip.write_all(&bytes)
            .map_err(|error| format!("E_UNIVERSE_PACK_ZIP_WRITE {error}"))?;
    }
    let bytes = zip
        .finish()
        .map_err(|error| format!("E_UNIVERSE_PACK_ZIP_FINISH {error}"))?
        .into_inner();
    write_binary_artifact_atomic(out_file, &bytes).map_err(|error| {
        format!(
            "E_UNIVERSE_PACK_ATOMIC_PUBLISH {} ({error})",
            out_file.display()
        )
    })?;

    let pack_hash = sha256_file(out_file)?;
    println!("universe_pack_out={}", out_file.display());
    println!("universe_pack_files={}", files.len());
    println!("universe_pack_hash={}", pack_hash);
    Ok(())
}

pub fn run_unpack(input_file: &Path, out_dir: &Path) -> Result<(), String> {
    if !input_file.is_file() {
        return Err(format!(
            "E_UNIVERSE_UNPACK_INPUT_FILE input file not found: {}",
            input_file.display()
        ));
    }
    require_empty_output_dir(out_dir)?;

    let input_handle = File::open(input_file).map_err(|error| {
        format!(
            "E_UNIVERSE_UNPACK_INPUT_OPEN open input failed {} ({})",
            input_file.display(),
            error
        )
    })?;
    let mut archive = ZipArchive::new(input_handle)
        .map_err(|error| format!("E_UNIVERSE_UNPACK_ZIP_OPEN {error}"))?;

    let mut prepared = Vec::<(PathBuf, Vec<u8>)>::new();
    let mut targets = HashSet::new();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("E_UNIVERSE_UNPACK_ZIP_ENTRY {error}"))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        validate_relative_zip_path(&name)?;
        let out_path = out_dir.join(Path::new(&name));
        if !targets.insert(out_path.clone()) {
            return Err(format!(
                "E_UNIVERSE_UNPACK_DUPLICATE_PATH duplicate archive path: {name}"
            ));
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|error| {
            format!("E_UNIVERSE_UNPACK_READ read archive entry failed {name} ({error})")
        })?;
        prepared.push((out_path, bytes));
    }

    if prepared.is_empty() {
        fs::create_dir_all(out_dir).map_err(|error| {
            format!(
                "E_UNIVERSE_UNPACK_OUT_CREATE create out dir failed {} ({error})",
                out_dir.display()
            )
        })?;
    } else {
        let payloads = prepared
            .iter()
            .map(|(target, bytes)| ArtifactPayload {
                target: target.as_path(),
                bytes: bytes.as_slice(),
            })
            .collect::<Vec<_>>();
        write_artifact_set_atomic(&payloads).map_err(|error| {
            format!(
                "E_UNIVERSE_UNPACK_ATOMIC_PUBLISH {} ({error})",
                out_dir.display()
            )
        })?;
    }

    println!("universe_unpack_out={}", out_dir.display());
    println!("universe_unpack_files={}", prepared.len());
    Ok(())
}

fn require_empty_output_dir(out_dir: &Path) -> Result<(), String> {
    if !out_dir.exists() {
        return Ok(());
    }
    if !out_dir.is_dir() {
        return Err(format!(
            "E_UNIVERSE_UNPACK_OUT_NOT_DIR out path must be a directory: {}",
            out_dir.display()
        ));
    }
    let mut iter = fs::read_dir(out_dir).map_err(|error| {
        format!(
            "E_UNIVERSE_UNPACK_OUT_READ read out dir failed {} ({error})",
            out_dir.display()
        )
    })?;
    if iter.next().is_some() {
        return Err(format!(
            "E_UNIVERSE_UNPACK_OUT_NOT_EMPTY out dir must be empty: {}",
            out_dir.display()
        ));
    }
    Ok(())
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(root: &Path, current: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let mut entries = fs::read_dir(current)
            .map_err(|error| format!("E_UNIVERSE_PACK_READ_DIR {} ({error})", current.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("E_UNIVERSE_PACK_READ_DIR {} ({error})", current.display()))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out)?;
            } else if path.is_file() {
                let rel = path.strip_prefix(root).map_err(|error| {
                    format!(
                        "E_UNIVERSE_PACK_REL_PATH strip_prefix failed {} ({error})",
                        path.display()
                    )
                })?;
                out.push(rel.to_path_buf());
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    walk(root, root, &mut files)?;
    files.sort_by_key(|path| path_to_unix(path).unwrap_or_default());
    Ok(files)
}

fn path_to_unix(path: &Path) -> Result<String, String> {
    let text = path.to_string_lossy().replace('\\', "/");
    if text.starts_with('/') || text.contains("../") || text.contains("/..") {
        return Err(format!(
            "E_UNIVERSE_PACK_PATH_INVALID invalid relative path: {text}"
        ));
    }
    Ok(text)
}

fn validate_relative_zip_path(path: &str) -> Result<(), String> {
    let value = Path::new(path);
    for component in value.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) | Component::ParentDir => {
                return Err(format!(
                    "E_UNIVERSE_UNPACK_PATH_INVALID unsafe zip path: {path}"
                ));
            }
        }
    }
    Ok(())
}

fn fixed_zip_datetime() -> DateTime {
    DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).unwrap_or_default()
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "E_UNIVERSE_HASH_READ read file failed {} ({error})",
            path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("sha256:{}", hex::encode(hasher.finalize())))
}
