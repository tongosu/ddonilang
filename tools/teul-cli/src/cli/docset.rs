use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::{paths, run};
use crate::core::hash::SSOT_VERSION;
use crate::core::{hash, State};

const DOCSET_SCHEMA: &str = "malmoi.docset.v1";
const EXAMPLES_SCHEMA: &str = "malmoi.examples_manifest.v1";

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
    entries: Vec<DocsetEntry>,
}

#[derive(Deserialize)]
struct ExamplesManifest {
    schema: String,
    examples: Vec<ExampleSpec>,
}

#[derive(Deserialize)]
struct ExampleSpec {
    id: Option<String>,
    path: String,
    ticks: Option<u64>,
    seed: Option<u64>,
    state_hash: String,
}

pub struct DocBuildOptions {
    pub out: PathBuf,
}

pub struct DocVerifyOptions {
    pub pack: PathBuf,
    pub out: Option<PathBuf>,
}

pub fn run_build(opts: DocBuildOptions) -> Result<(), String> {
    let result = build_docset(&opts.out)?;
    let hash_path = opts.out.join("docset_hash.txt");
    write_text(&hash_path, &format!("{}\n", result.docset_hash))?;
    println!("docset_out={}", opts.out.display());
    println!("docset_hash={}", result.docset_hash);
    Ok(())
}

pub fn run_verify(opts: DocVerifyOptions) -> Result<(), String> {
    let pack = &opts.pack;
    let golden_hash_path = pack.join("golden_docset_hash.txt");
    let golden_hash = fs::read_to_string(&golden_hash_path)
        .map_err(|e| format!("E_DOCSET_GOLDEN_READ {} ({})", golden_hash_path.display(), e))?;
    let golden_hash = golden_hash.trim();
    if golden_hash.is_empty() {
        return Err("E_DOCSET_GOLDEN_EMPTY".to_string());
    }

    let out_dir = opts
        .out
        .unwrap_or_else(|| paths::build_dir().join("malmoi_docset"));
    let result = build_docset(&out_dir)?;
    if result.docset_hash != golden_hash {
        return Err(format!(
            "E_DOCSET_NONDETERMINISM expected={} got={}",
            golden_hash, result.docset_hash
        ));
    }

    let manifest_path = pack.join("examples_manifest.json");
    let manifest_text = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("E_DOCTEST_MANIFEST_READ {} ({})", manifest_path.display(), e))?;
    let manifest: ExamplesManifest = serde_json::from_str(&manifest_text)
        .map_err(|e| format!("E_DOCTEST_MANIFEST_PARSE {e}"))?;
    if manifest.schema != EXAMPLES_SCHEMA {
        return Err(format!("E_DOCTEST_MANIFEST_SCHEMA {}", manifest.schema));
    }

    let mut passed = 0u64;
    for (idx, example) in manifest.examples.iter().enumerate() {
        let entry_id = example
            .id
            .clone()
            .unwrap_or_else(|| format!("example-{}", idx + 1));
        let ticks = example.ticks.unwrap_or(1);
        let seed = example.seed.unwrap_or(0);
        let path = Path::new(&example.path);
        let source = fs::read_to_string(path).map_err(|e| {
            format!("E_DOCTEST_SOURCE_READ {} ({})", path.display(), e)
        })?;
        let output = run::run_source_with_state_seed_ticks(&source, State::new(), seed, ticks)
            .map_err(|err| format!("E_DOCTEST_FAIL {} {}", entry_id, err.format(&example.path)))?;
        let state_hash = hash::state_hash(&output.state);
        if state_hash != example.state_hash {
            return Err(format!(
                "E_DOCTEST_FAIL {} expected={} got={}",
                entry_id, example.state_hash, state_hash
            ));
        }
        passed += 1;
    }

    println!("docset_hash={}", result.docset_hash);
    println!("docset_verify=ok");
    println!("doctest_passed={}", passed);
    Ok(())
}

struct DocsetBuildResult {
    docset_hash: String,
}

fn build_docset(out_dir: &Path) -> Result<DocsetBuildResult, String> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("E_DOCSET_OUT_DIR {} ({})", out_dir.display(), e))?;
    let entries_dir = out_dir.join("entries");
    fs::create_dir_all(&entries_dir)
        .map_err(|e| format!("E_DOCSET_OUT_DIR {} ({})", entries_dir.display(), e))?;

    let mut entries: Vec<DocsetEntry> = Vec::new();
    let mut written_files: Vec<PathBuf> = Vec::new();

    for source in DOCSET_SOURCES {
        let source_path = Path::new(source.path);
        let text = fs::read_to_string(source_path).map_err(|e| {
            format!("E_DOCSET_SOURCE_READ {} ({})", source_path.display(), e)
        })?;
        let title = extract_title(&text, source.path);
        let rel_path = normalize_rel_path(source_path);
        let id = stable_key_from_path(&rel_path);
        let entry_file = format!("{}.md", id);
        let entry_path = entries_dir.join(&entry_file);
        write_text(&entry_path, &text)?;
        let entry_hash = sha256_text(&text);
        entries.push(DocsetEntry {
            id,
            title,
            source_path: rel_path,
            content_path: format!("entries/{}", entry_file),
            sha256: entry_hash,
        });
        written_files.push(entry_path);
    }

    let meta = DocsetMeta {
        schema: DOCSET_SCHEMA.to_string(),
        ssot_version: SSOT_VERSION.to_string(),
        entries,
    };
    let meta_text =
        serde_json::to_string_pretty(&meta).map_err(|e| format!("E_DOCSET_META_JSON {}", e))?
            + "\n";
    let meta_path = out_dir.join("docset.json");
    write_text(&meta_path, &meta_text)?;
    written_files.push(meta_path);

    let docset_hash = compute_docset_hash(out_dir, &written_files)?;
    Ok(DocsetBuildResult { docset_hash })
}

fn compute_docset_hash(out_dir: &Path, files: &[PathBuf]) -> Result<String, String> {
    let mut entries: Vec<(String, PathBuf)> = files
        .iter()
        .map(|path| {
            let rel = path
                .strip_prefix(out_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            (rel, path.to_path_buf())
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hasher = Sha256::new();
    for (rel, path) in entries {
        let bytes = fs::read(&path).map_err(|e| format!("E_DOCSET_HASH_READ {} ({})", rel, e))?;
        hasher.update(rel.as_bytes());
        hasher.update(b"\0");
        hasher.update(&bytes);
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
        .and_then(|s| s.to_str())
        .unwrap_or(fallback)
        .to_string()
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

fn sha256_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("E_DOCSET_WRITE {} ({})", parent.display(), e))?;
    }
    fs::write(path, text)
        .map_err(|e| format!("E_DOCSET_WRITE {} ({})", path.display(), e))?;
    Ok(())
}
