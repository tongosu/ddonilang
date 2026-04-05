use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

pub fn run_pack(input_dir: &Path, out_file: &Path) -> Result<(), String> {
    if !input_dir.is_dir() {
        return Err(format!(
            "E_UNIVERSE_PACK_INPUT_DIR input dir not found: {}",
            input_dir.display()
        ));
    }
    if let Some(parent) = out_file.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "E_UNIVERSE_PACK_OUT_DIR create out parent failed {} ({})",
                parent.display(),
                e
            )
        })?;
    }

    let files = collect_files(input_dir)?;
    let out_handle = File::create(out_file).map_err(|e| {
        format!(
            "E_UNIVERSE_PACK_OUT_CREATE create out file failed {} ({})",
            out_file.display(),
            e
        )
    })?;
    let mut zip = ZipWriter::new(out_handle);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(fixed_zip_datetime())
        .unix_permissions(0o644);

    for rel in &files {
        let rel_unix = path_to_unix(rel)?;
        let source = input_dir.join(rel);
        let bytes = fs::read(&source).map_err(|e| {
            format!(
                "E_UNIVERSE_PACK_READ read source failed {} ({})",
                source.display(),
                e
            )
        })?;
        zip.start_file(rel_unix, options)
            .map_err(|e| format!("E_UNIVERSE_PACK_ZIP_WRITE {}", e))?;
        zip.write_all(&bytes)
            .map_err(|e| format!("E_UNIVERSE_PACK_ZIP_WRITE {}", e))?;
    }
    zip.finish()
        .map_err(|e| format!("E_UNIVERSE_PACK_ZIP_FINISH {}", e))?;

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
    if out_dir.exists() {
        let mut iter = fs::read_dir(out_dir).map_err(|e| {
            format!(
                "E_UNIVERSE_UNPACK_OUT_READ read out dir failed {} ({})",
                out_dir.display(),
                e
            )
        })?;
        if iter.next().is_some() {
            return Err(format!(
                "E_UNIVERSE_UNPACK_OUT_NOT_EMPTY out dir must be empty: {}",
                out_dir.display()
            ));
        }
    } else {
        fs::create_dir_all(out_dir).map_err(|e| {
            format!(
                "E_UNIVERSE_UNPACK_OUT_CREATE create out dir failed {} ({})",
                out_dir.display(),
                e
            )
        })?;
    }

    let input_handle = File::open(input_file).map_err(|e| {
        format!(
            "E_UNIVERSE_UNPACK_INPUT_OPEN open input failed {} ({})",
            input_file.display(),
            e
        )
    })?;
    let mut archive =
        ZipArchive::new(input_handle).map_err(|e| format!("E_UNIVERSE_UNPACK_ZIP_OPEN {}", e))?;

    let mut file_count = 0usize;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("E_UNIVERSE_UNPACK_ZIP_ENTRY {}", e))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        validate_relative_zip_path(&name)?;
        let out_path = out_dir.join(Path::new(&name));
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "E_UNIVERSE_UNPACK_OUT_PARENT create parent failed {} ({})",
                    parent.display(),
                    e
                )
            })?;
        }
        let mut out_handle = File::create(&out_path).map_err(|e| {
            format!(
                "E_UNIVERSE_UNPACK_OUT_FILE create out file failed {} ({})",
                out_path.display(),
                e
            )
        })?;
        std::io::copy(&mut entry, &mut out_handle).map_err(|e| {
            format!(
                "E_UNIVERSE_UNPACK_COPY copy failed {} ({})",
                out_path.display(),
                e
            )
        })?;
        file_count += 1;
    }

    println!("universe_unpack_out={}", out_dir.display());
    println!("universe_unpack_files={}", file_count);
    Ok(())
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(root: &Path, current: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let mut entries = fs::read_dir(current)
            .map_err(|e| format!("E_UNIVERSE_PACK_READ_DIR {} ({})", current.display(), e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("E_UNIVERSE_PACK_READ_DIR {} ({})", current.display(), e))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out)?;
            } else if path.is_file() {
                let rel = path.strip_prefix(root).map_err(|e| {
                    format!(
                        "E_UNIVERSE_PACK_REL_PATH strip_prefix failed {} ({})",
                        path.display(),
                        e
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
            "E_UNIVERSE_PACK_PATH_INVALID invalid relative path: {}",
            text
        ));
    }
    Ok(text)
}

fn validate_relative_zip_path(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    for comp in p.components() {
        match comp {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) | Component::ParentDir => {
                return Err(format!(
                    "E_UNIVERSE_UNPACK_PATH_INVALID unsafe zip path: {}",
                    path
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
    let bytes = fs::read(path).map_err(|e| {
        format!(
            "E_UNIVERSE_HASH_READ read file failed {} ({})",
            path.display(),
            e
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("sha256:{}", hex::encode(hasher.finalize())))
}
