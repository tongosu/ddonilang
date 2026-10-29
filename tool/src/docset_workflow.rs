use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value as JsonValue};
use sha2::{Digest, Sha256};

use crate::artifact_output::write_directory_artifact_atomic;
use crate::runtime_surface::sha256_identity;
use crate::state_trace_wire::LEGACY_RUNTIME_SSOT_VERSION;

const DOCSET_SCHEMA: &str = "malmoi.docset.v1";

struct DocSource {
    path: &'static str,
}

const DOCSET_SOURCES: &[DocSource] = &[
    DocSource {
        path: "docs/guides/GRAMMAR_COMPLETE_REFERENCE.md",
    },
    DocSource {
        path: "docs/guides/GLOSSARY.md",
    },
    DocSource {
        path: "docs/guides/README.md",
    },
    DocSource {
        path: "docs/guides/QUICKSTART.md",
    },
    DocSource {
        path: "docs/guides/QUICK_REFERENCE.md",
    },
    DocSource {
        path: "docs/guides/ERRORS.md",
    },
    DocSource {
        path: "docs/guides/BEGINNER_GUIDE.md",
    },
    DocSource {
        path: "docs/guides/INTERMEDIATE_GUIDE.md",
    },
    DocSource {
        path: "docs/guides/ADVANCED_GUIDE.md",
    },
    DocSource {
        path: "docs/guides/CONTRIBUTING.md",
    },
    DocSource {
        path: "docs/guides/TESTING.md",
    },
    DocSource {
        path: "docs/guides/TETRIS_RENDER_ALIGNMENT.md",
    },
    DocSource {
        path: "docs/guides/LEARNING_RESOURCES_INDEX.md",
    },
    DocSource {
        path: "docs/EXAMPLES/README.md",
    },
];

#[derive(Serialize)]
struct DocsetEntry {
    id: String,
    title: String,
    source_path: String,
    content_path: String,
    sha256: String,
}

#[derive(Serialize)]
struct DocsetMeta {
    schema: String,
    ssot_version: String,
    source_hash: String,
    source_provenance: JsonValue,
    entries: Vec<DocsetEntry>,
}

pub struct DocBuildOptions {
    pub out: PathBuf,
}

pub struct DocsetBuildResult {
    pub docset_hash: String,
}

pub fn run_build(options: DocBuildOptions) -> Result<(), String> {
    let result = build_docset_atomic(&options.out)?;
    println!("docset_out={}", options.out.display());
    println!("docset_hash={}", result.docset_hash);
    Ok(())
}

pub fn build_docset_atomic(out_dir: &Path) -> Result<DocsetBuildResult, String> {
    let mut result = None;
    write_directory_artifact_atomic(out_dir, |staging| {
        let built = build_docset(staging)?;
        write_text(
            &staging.join("docset_hash.txt"),
            &format!("{}\n", built.docset_hash),
        )?;
        result = Some(built);
        Ok(())
    })?;
    result.ok_or_else(|| "E_DOCSET_BUILD_RESULT_MISSING".to_string())
}

fn build_docset(out_dir: &Path) -> Result<DocsetBuildResult, String> {
    let entries_dir = out_dir.join("entries");
    fs::create_dir_all(&entries_dir)
        .map_err(|error| format!("E_DOCSET_OUT_DIR {} ({error})", entries_dir.display()))?;

    let mut entries = Vec::new();
    let mut written_files = Vec::new();
    for source in DOCSET_SOURCES {
        let source_path = Path::new(source.path);
        let text = fs::read_to_string(source_path)
            .map_err(|error| format!("E_DOCSET_SOURCE_READ {} ({error})", source_path.display()))?;
        let title = extract_title(&text, source.path);
        let rel_path = normalize_rel_path(source_path);
        let id = stable_key_from_path(&rel_path);
        let entry_file = format!("{id}.md");
        let entry_path = entries_dir.join(&entry_file);
        write_text(&entry_path, &text)?;
        let entry_hash = sha256_identity(text.as_bytes());
        entries.push(DocsetEntry {
            id,
            title,
            source_path: rel_path,
            content_path: format!("entries/{entry_file}"),
            sha256: entry_hash,
        });
        written_files.push(entry_path);
    }

    let source_provenance = build_docset_source_provenance(&entries);
    let source_provenance_text = serde_json::to_string(&source_provenance)
        .map_err(|error| format!("E_DOCSET_SOURCE_PROVENANCE_JSON {error}"))?;
    let source_hash = sha256_identity(source_provenance_text.as_bytes());
    let meta = DocsetMeta {
        schema: DOCSET_SCHEMA.to_string(),
        ssot_version: LEGACY_RUNTIME_SSOT_VERSION.to_string(),
        source_hash,
        source_provenance,
        entries,
    };
    let meta_text = serde_json::to_string_pretty(&meta)
        .map_err(|error| format!("E_DOCSET_META_JSON {error}"))?
        + "\n";
    let meta_path = out_dir.join("docset.json");
    write_text(&meta_path, &meta_text)?;
    written_files.push(meta_path);

    Ok(DocsetBuildResult {
        docset_hash: compute_docset_hash(out_dir, &written_files)?,
    })
}

fn compute_docset_hash(out_dir: &Path, files: &[PathBuf]) -> Result<String, String> {
    let mut entries = files
        .iter()
        .map(|path| {
            let rel = path
                .strip_prefix(out_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            (rel, path.to_path_buf())
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hasher = Sha256::new();
    for (rel, path) in entries {
        let bytes =
            fs::read(&path).map_err(|error| format!("E_DOCSET_HASH_READ {rel} ({error})"))?;
        hasher.update(rel.as_bytes());
        hasher.update(b"\0");
        hasher.update(bytes);
    }
    Ok(format!("sha256:{}", hex::encode(hasher.finalize())))
}

fn extract_title(text: &str, fallback: &str) -> String {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            let title = title.trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }
    Path::new(fallback)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(fallback)
        .to_string()
}

fn build_docset_source_provenance(entries: &[DocsetEntry]) -> JsonValue {
    let files = entries
        .iter()
        .map(|entry| {
            let mut file = Map::new();
            file.insert(
                "path".to_string(),
                JsonValue::String(entry.source_path.clone()),
            );
            file.insert(
                "sha256".to_string(),
                JsonValue::String(entry.sha256.clone()),
            );
            JsonValue::Object(file)
        })
        .collect();
    let mut map = Map::new();
    map.insert(
        "schema".to_string(),
        JsonValue::String("malmoi.docset_source_provenance.v1".to_string()),
    );
    map.insert(
        "source_kind".to_string(),
        JsonValue::String("docset_sources.v1".to_string()),
    );
    map.insert(
        "file_count".to_string(),
        JsonValue::Number(serde_json::Number::from(entries.len())),
    );
    map.insert("files".to_string(), JsonValue::Array(files));
    JsonValue::Object(map)
}

fn normalize_rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn stable_key_from_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len() + 8);
    out.push_str("doc_");
    for ch in path.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("E_DOCSET_WRITE {} ({error})", parent.display()))?;
    }
    fs::write(path, text).map_err(|error| format!("E_DOCSET_WRITE {} ({error})", path.display()))
}
