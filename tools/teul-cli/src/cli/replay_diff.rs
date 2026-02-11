use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

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
    fs::create_dir_all(&options.out).map_err(|e| e.to_string())?;
    let detjson_path = options.out.join("diff.detjson");
    let detjson = build_detjson_report(&a_manifest, &b_manifest, equal, &first_diverge);
    fs::write(&detjson_path, detjson).map_err(|e| e.to_string())?;

    if options.write_summary {
        let summary_path = options.out.join("diff.txt");
        let summary = build_summary_text(equal, &first_diverge);
        fs::write(summary_path, summary).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn load_manifest(path: &Path) -> Result<ManifestInfo, String> {
    let manifest_path = if path.is_dir() {
        path.join("manifest.detjson")
    } else {
        path.to_path_buf()
    };
    let raw = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("E_REPLAY_DIFF_READ {}:1:1 {}", manifest_path.display(), e))?;
    let json: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("E_REPLAY_DIFF_MANIFEST {}:1:1 {}", manifest_path.display(), e))?;

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
    let frames_value = json.get("frames").and_then(|value| value.as_array()).ok_or_else(|| {
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
    out.push_str(&format!("  \"equal\": {},\n", if equal { "true" } else { "false" }));
    match first_diverge {
        Some(diverge) => {
            out.push_str(&format!(
                "  \"first_diverge_madi\": {},\n",
                diverge.madi
            ));
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
            push_optional_string(&mut out, "state_hash_a", diverge.state_hash_a.as_deref(), true);
            push_optional_string(&mut out, "state_hash_b", diverge.state_hash_b.as_deref(), true);
            push_optional_string(&mut out, "bogae_hash_a", diverge.bogae_hash_a.as_deref(), true);
            push_optional_string(&mut out, "bogae_hash_b", diverge.bogae_hash_b.as_deref(), false);
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

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

fn build_summary_text(equal: bool, first_diverge: &Option<FirstDiverge>) -> String {
    let mut out = String::new();
    out.push_str(&format!("equal: {}\n", if equal { "true" } else { "false" }));
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
