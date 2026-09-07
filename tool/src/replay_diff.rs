use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use crate::state_trace_wire::escape_json_string_contents as escape_json;

static REPLAY_DIFF_OUTPUT_NONCE: AtomicU64 = AtomicU64::new(0);

pub struct ReplayDiffOptions {
    pub a: PathBuf,
    pub b: PathBuf,
    pub out: PathBuf,
    pub write_summary: bool,
}

struct FrameInfo {
    state_hash: String,
    bogae_hash: Option<String>,
}

struct ManifestInfo {
    start_madi: u64,
    end_madi: u64,
    frames: BTreeMap<u64, FrameInfo>,
}

struct FirstDiverge {
    madi: u64,
    state_hash_a: Option<String>,
    state_hash_b: Option<String>,
    bogae_hash_a: Option<String>,
    bogae_hash_b: Option<String>,
}

pub fn run_diff(options: ReplayDiffOptions) -> Result<(), String> {
    let a_manifest = load_manifest(&options.a)?;
    let b_manifest = load_manifest(&options.b)?;

    let mut all_madis = BTreeMap::new();
    for madi in a_manifest.frames.keys() {
        all_madis.insert(*madi, ());
    }
    for madi in b_manifest.frames.keys() {
        all_madis.insert(*madi, ());
    }

    let mut first_diverge = None;
    for madi in all_madis.keys() {
        let a_frame = a_manifest.frames.get(madi);
        let b_frame = b_manifest.frames.get(madi);
        let state_a = a_frame.map(|frame| frame.state_hash.as_str());
        let state_b = b_frame.map(|frame| frame.state_hash.as_str());
        let bogae_a = a_frame.and_then(|frame| frame.bogae_hash.as_deref());
        let bogae_b = b_frame.and_then(|frame| frame.bogae_hash.as_deref());

        let state_diff = state_a != state_b;
        let bogae_diff = match (bogae_a, bogae_b) {
            (Some(left), Some(right)) => left != right,
            _ => false,
        };
        if state_diff || bogae_diff {
            first_diverge = Some(FirstDiverge {
                madi: *madi,
                state_hash_a: state_a.map(|value| value.to_string()),
                state_hash_b: state_b.map(|value| value.to_string()),
                bogae_hash_a: bogae_a.map(|value| value.to_string()),
                bogae_hash_b: bogae_b.map(|value| value.to_string()),
            });
            break;
        }
    }

    let equal = first_diverge.is_none();
    let detjson = build_detjson_report(&a_manifest, &b_manifest, equal, &first_diverge);
    let summary = options
        .write_summary
        .then(|| build_summary_text(equal, &first_diverge));
    write_replay_diff_outputs_atomic(&options.out, &detjson, summary.as_deref())
}

struct PreparedOutput {
    target: PathBuf,
    temp: PathBuf,
    backup: PathBuf,
    had_existing: bool,
}

fn write_replay_diff_outputs_atomic(
    out_dir: &Path,
    detjson: &str,
    summary: Option<&str>,
) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|error| error.to_string())?;
    let mut outputs = vec![("diff.detjson", detjson)];
    if let Some(summary) = summary {
        outputs.push(("diff.txt", summary));
    }

    let nonce = REPLAY_DIFF_OUTPUT_NONCE.fetch_add(1, Ordering::Relaxed);
    let mut prepared: Vec<PreparedOutput> = Vec::with_capacity(outputs.len());
    for (index, (name, text)) in outputs.into_iter().enumerate() {
        let target = out_dir.join(name);
        if target.exists() && !target.is_file() {
            cleanup_prepared(&prepared);
            return Err(format!(
                "E_REPLAY_DIFF_OUTPUT_TARGET_NOT_FILE {}",
                target.display()
            ));
        }
        let temp = out_dir.join(format!(
            ".{name}.ddn-replay-diff-{}-{nonce}-{index}.tmp",
            process::id()
        ));
        let backup = out_dir.join(format!(
            ".{name}.ddn-replay-diff-{}-{nonce}-{index}.backup",
            process::id()
        ));
        if temp.exists() || backup.exists() {
            cleanup_prepared(&prepared);
            return Err(format!(
                "E_REPLAY_DIFF_OUTPUT_STAGING_CONFLICT {}",
                target.display()
            ));
        }

        let mut file = match OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(file) => file,
            Err(error) => {
                cleanup_prepared(&prepared);
                return Err(error.to_string());
            }
        };
        if let Err(error) = file
            .write_all(text.as_bytes())
            .and_then(|_| file.sync_all())
        {
            drop(file);
            let _ = fs::remove_file(&temp);
            cleanup_prepared(&prepared);
            return Err(error.to_string());
        }
        drop(file);
        prepared.push(PreparedOutput {
            had_existing: target.exists(),
            target,
            temp,
            backup,
        });
    }

    let mut backed_up = 0;
    for item in &prepared {
        if item.had_existing {
            if let Err(error) = fs::rename(&item.target, &item.backup) {
                rollback_outputs(&prepared, backed_up, 0);
                return Err(error.to_string());
            }
        }
        backed_up += 1;
    }

    let mut published = 0;
    for item in &prepared {
        if let Err(error) = fs::rename(&item.temp, &item.target) {
            rollback_outputs(&prepared, backed_up, published);
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

fn cleanup_prepared(prepared: &[PreparedOutput]) {
    for item in prepared {
        if item.temp.exists() {
            let _ = fs::remove_file(&item.temp);
        }
    }
}

fn rollback_outputs(prepared: &[PreparedOutput], backed_up: usize, published: usize) {
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
    cleanup_prepared(prepared);
}

fn load_manifest(path: &Path) -> Result<ManifestInfo, String> {
    let manifest_path = if path.is_dir() {
        path.join("manifest.detjson")
    } else {
        path.to_path_buf()
    };
    let raw = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("E_REPLAY_DIFF_READ {}:1:1 {}", manifest_path.display(), e))?;
    let json: Value = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "E_REPLAY_DIFF_MANIFEST {}:1:1 {}",
            manifest_path.display(),
            e
        )
    })?;

    let start_madi = json
        .get("start_madi")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| {
            format!(
                "E_REPLAY_DIFF_MANIFEST {}:1:1 start_madi 누락",
                manifest_path.display()
            )
        })?;
    let end_madi = json
        .get("end_madi")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| {
            format!(
                "E_REPLAY_DIFF_MANIFEST {}:1:1 end_madi 누락",
                manifest_path.display()
            )
        })?;
    let frames_value = json
        .get("frames")
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            format!(
                "E_REPLAY_DIFF_MANIFEST {}:1:1 frames 누락",
                manifest_path.display()
            )
        })?;

    let mut frames = BTreeMap::new();
    for frame in frames_value {
        let madi = frame
            .get("madi")
            .and_then(|value| value.as_u64())
            .ok_or_else(|| {
                format!(
                    "E_REPLAY_DIFF_MANIFEST {}:1:1 frame.madi 누락",
                    manifest_path.display()
                )
            })?;
        let state_hash = frame
            .get("state_hash")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                format!(
                    "E_REPLAY_DIFF_MANIFEST {}:1:1 frame.state_hash 누락",
                    manifest_path.display()
                )
            })?
            .to_string();
        let bogae_hash = frame
            .get("bogae_hash")
            .and_then(|value| value.as_str())
            .map(|text| text.to_string());
        if frames.contains_key(&madi) {
            return Err(format!(
                "E_REPLAY_DIFF_MANIFEST {}:1:1 frame.madi 중복: {}",
                manifest_path.display(),
                madi
            ));
        }
        frames.insert(
            madi,
            FrameInfo {
                state_hash,
                bogae_hash,
            },
        );
    }

    Ok(ManifestInfo {
        start_madi,
        end_madi,
        frames,
    })
}

fn build_detjson_report(
    a: &ManifestInfo,
    b: &ManifestInfo,
    equal: bool,
    first_diverge: &Option<FirstDiverge>,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"kind\": \"replay_diff_v1\",\n");
    out.push_str(&format!(
        "  \"equal\": {},\n",
        if equal { "true" } else { "false" }
    ));
    match first_diverge {
        Some(diverge) => {
            out.push_str(&format!("  \"first_diverge_madi\": {},\n", diverge.madi));
        }
        None => {
            out.push_str("  \"first_diverge_madi\": null,\n");
        }
    }
    out.push_str("  \"a\": {\n");
    out.push_str(&format!("    \"start_madi\": {},\n", a.start_madi));
    out.push_str(&format!("    \"end_madi\": {},\n", a.end_madi));
    out.push_str(&format!("    \"frame_count\": {}\n", a.frames.len()));
    out.push_str("  },\n");
    out.push_str("  \"b\": {\n");
    out.push_str(&format!("    \"start_madi\": {},\n", b.start_madi));
    out.push_str(&format!("    \"end_madi\": {},\n", b.end_madi));
    out.push_str(&format!("    \"frame_count\": {}\n", b.frames.len()));
    out.push_str("  },\n");
    match first_diverge {
        Some(diverge) => {
            out.push_str("  \"first_diverge\": {\n");
            out.push_str(&format!("    \"madi\": {},\n", diverge.madi));
            push_optional_string(
                &mut out,
                "state_hash_a",
                diverge.state_hash_a.as_deref(),
                true,
            );
            push_optional_string(
                &mut out,
                "state_hash_b",
                diverge.state_hash_b.as_deref(),
                true,
            );
            push_optional_string(
                &mut out,
                "bogae_hash_a",
                diverge.bogae_hash_a.as_deref(),
                true,
            );
            push_optional_string(
                &mut out,
                "bogae_hash_b",
                diverge.bogae_hash_b.as_deref(),
                false,
            );
            out.push_str("  }\n");
        }
        None => {
            out.push_str("  \"first_diverge\": null\n");
        }
    }
    out.push_str("}\n");
    out
}

fn push_optional_string(out: &mut String, key: &str, value: Option<&str>, trailing: bool) {
    out.push_str(&format!("    \"{}\": ", key));
    match value {
        Some(text) => {
            out.push('"');
            out.push_str(&escape_json(text));
            out.push('"');
        }
        None => {
            out.push_str("null");
        }
    }
    if trailing {
        out.push_str(",\n");
    } else {
        out.push('\n');
    }
}

fn build_summary_text(equal: bool, first_diverge: &Option<FirstDiverge>) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "equal: {}\n",
        if equal { "true" } else { "false" }
    ));
    match first_diverge {
        Some(diverge) => {
            out.push_str(&format!("first_diverge_madi: {}\n", diverge.madi));
            if let (Some(a), Some(b)) = (&diverge.state_hash_a, &diverge.state_hash_b) {
                out.push_str(&format!("state_hash_a: {}\n", a));
                out.push_str(&format!("state_hash_b: {}\n", b));
            }
            if let (Some(a), Some(b)) = (&diverge.bogae_hash_a, &diverge.bogae_hash_b) {
                out.push_str(&format!("bogae_hash_a: {}\n", a));
                out.push_str(&format!("bogae_hash_b: {}\n", b));
            }
        }
        None => {
            out.push_str("first_diverge_madi: null\n");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{run_diff, ReplayDiffOptions};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "ddn_replay_diff_{label}_{}_{}",
            process::id(),
            nonce
        ))
    }

    fn write_manifest(dir: &Path, state_hash: &str) {
        fs::create_dir_all(dir).expect("create manifest dir");
        fs::write(
            dir.join("manifest.detjson"),
            format!(
                "{{\n  \"start_madi\": 0,\n  \"end_madi\": 0,\n  \"frames\": [{{\"madi\": 0, \"state_hash\": \"{state_hash}\", \"bogae_hash\": \"blake3:view\"}}]\n}}\n"
            ),
        )
        .expect("write manifest");
    }

    #[test]
    fn equal_replays_publish_one_atomic_output_set() {
        let root = test_dir("equal");
        let a = root.join("a");
        let b = root.join("b");
        let out = root.join("out");
        write_manifest(&a, "blake3:same");
        write_manifest(&b, "blake3:same");

        run_diff(ReplayDiffOptions {
            a,
            b,
            out: out.clone(),
            write_summary: true,
        })
        .expect("replay diff");

        assert!(fs::read_to_string(out.join("diff.detjson"))
            .expect("read detjson")
            .contains("\"equal\": true"));
        assert_eq!(
            fs::read_to_string(out.join("diff.txt")).expect("read summary"),
            "equal: true\nfirst_diverge_madi: null\n"
        );
        assert_eq!(fs::read_dir(&out).expect("read out").count(), 2);
        fs::remove_dir_all(root).expect("remove test dir");
    }

    #[test]
    fn blocked_summary_target_fails_before_any_result_publish() {
        let root = test_dir("blocked");
        let a = root.join("a");
        let b = root.join("b");
        let out = root.join("out");
        write_manifest(&a, "blake3:same");
        write_manifest(&b, "blake3:same");
        fs::create_dir_all(out.join("diff.txt")).expect("create blocking directory");

        let error = run_diff(ReplayDiffOptions {
            a,
            b,
            out: out.clone(),
            write_summary: true,
        })
        .expect_err("blocked summary must fail closed");

        assert!(error.contains("E_REPLAY_DIFF_OUTPUT_TARGET_NOT_FILE"));
        assert!(!out.join("diff.detjson").exists());
        assert_eq!(fs::read_dir(&out).expect("read out").count(), 1);
        fs::remove_dir_all(root).expect("remove test dir");
    }
}
