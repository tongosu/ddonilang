use std::fs;
use std::path::Path;

use crate::artifact_output::write_text_artifacts_atomic;
#[cfg(not(target_arch = "wasm32"))]
use crate::geoul_bundle::{audit_hash, GeoulBundleReader};
#[cfg(not(target_arch = "wasm32"))]
use crate::geoul_snapshot::decode_input_snapshot;
use ddonirang_core::nurigym::spec::{ActionSpec, ObservationSpec};
use hex::encode as hex_encode;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

pub fn run_export_from_source(
    source: &DatasetExportSource,
    format: &str,
    out_dir: &Path,
    env_id: &str,
) -> Result<(), String> {
    match format {
        "nurigym_v0" => run_export_v0(source, out_dir, env_id),
        "nurigym_v1" => run_export_v1(source, out_dir, env_id),
        other => Err(format!("E_DATASET_FORMAT 지원하지 않는 format: {}", other)),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_export(
    geoul_dir: &Path,
    format: &str,
    out_dir: &Path,
    env_id: &str,
) -> Result<(), String> {
    if !matches!(format, "nurigym_v0" | "nurigym_v1") {
        return Err(format!("E_DATASET_FORMAT 지원하지 않는 format: {}", format));
    }

    let mut reader = GeoulBundleReader::open(geoul_dir)?;
    let frame_count = reader.frame_count();
    if frame_count == 0 {
        return Err("E_DATASET_EMPTY geoul 로그에 프레임이 없습니다".to_string());
    }
    let audit_hash = audit_hash(&geoul_dir.join("audit.ddni"))?;
    let provenance = read_geoul_manifest_provenance(geoul_dir, &audit_hash)?;

    let mut state_hashes = Vec::with_capacity(frame_count as usize);
    let mut rng_seed = 0u64;
    for idx in 0..frame_count {
        let frame = reader.read_frame(idx)?;
        state_hashes.push(format!("blake3:{}", hex_encode(frame.header.state_hash)));
        if idx == 0 {
            if let Ok(snapshot) = decode_input_snapshot(&frame.snapshot_detbin) {
                rng_seed = snapshot.rng_seed;
            }
        }
    }

    run_export_from_source(
        &DatasetExportSource {
            provenance,
            state_hashes,
            rng_seed,
        },
        format,
        out_dir,
        env_id,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatasetExportSource {
    pub provenance: DatasetProvenance,
    pub state_hashes: Vec<String>,
    pub rng_seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatasetProvenance {
    pub audit_hash: String,
    pub entry_file: String,
    pub entry_hash: String,
    pub age_target_source: String,
    pub age_target_value: String,
}

fn run_export_v0(source: &DatasetExportSource, out_dir: &Path, env_id: &str) -> Result<(), String> {
    if source.state_hashes.is_empty() {
        return Err("E_DATASET_EMPTY geoul 로그에 프레임이 없습니다".to_string());
    }

    let obs_spec = ObservationSpec::default_k64();
    let action_spec = ActionSpec::empty();
    let obs_text = obs_spec.to_detjson();
    let action_text = action_spec.to_detjson();
    let obs_hash = format!("sha256:{}", sha256_hex(obs_text.as_bytes()));
    let action_hash = format!("sha256:{}", sha256_hex(action_text.as_bytes()));

    let frame_count = source.state_hashes.len() as u64;
    let provenance = &source.provenance;

    let episode_header = build_episode_header(
        env_id,
        1,
        source.rng_seed,
        0,
        frame_count.saturating_sub(1),
        &obs_hash,
        &action_hash,
    );
    let episode_text = format!("{}\n", episode_header);

    let dataset_header = build_dataset_header(
        env_id,
        frame_count as u64,
        &provenance.audit_hash,
        &provenance.entry_file,
        &provenance.entry_hash,
        &provenance.age_target_source,
        &provenance.age_target_value,
    );
    let mut dataset_lines = Vec::with_capacity(frame_count as usize + 1);
    dataset_lines.push(dataset_header);

    for idx in 0..frame_count as usize {
        let madi = idx as u64;
        let obs_hash = &source.state_hashes[idx];
        let next_hash = if idx + 1 < source.state_hashes.len() {
            &source.state_hashes[idx + 1]
        } else {
            &source.state_hashes[idx]
        };
        let done = idx + 1 == source.state_hashes.len();
        let line = build_step_record(madi, obs_hash, next_hash, obs_spec.slot_count, done);
        dataset_lines.push(line);
    }

    let dataset_text = dataset_lines.join("\n") + "\n";
    let dataset_hash = format!("sha256:{}", sha256_hex(dataset_text.as_bytes()));
    let dataset_hash_text = format!("{}\n", dataset_hash);
    let obs_path = out_dir.join("obs_spec.detjson");
    let action_path = out_dir.join("action_spec.detjson");
    let episode_path = out_dir.join("nurigym.episode.jsonl");
    let dataset_path = out_dir.join("nurigym.dataset.jsonl");
    let hash_path = out_dir.join("dataset.sha256");
    write_text_artifacts_atomic(&[
        (obs_path.as_path(), obs_text.as_str()),
        (action_path.as_path(), action_text.as_str()),
        (episode_path.as_path(), episode_text.as_str()),
        (dataset_path.as_path(), dataset_text.as_str()),
        (hash_path.as_path(), dataset_hash_text.as_str()),
    ])
    .map_err(|error| format!("E_DATASET_ARTIFACT_SET_WRITE {error}"))?;
    println!("dataset_hash={}", dataset_hash);
    Ok(())
}

fn run_export_v1(source: &DatasetExportSource, out_dir: &Path, env_id: &str) -> Result<(), String> {
    if source.state_hashes.is_empty() {
        return Err("E_DATASET_EMPTY geoul 로그에 프레임이 없습니다".to_string());
    }

    let (obs_spec, action_spec) = canonical_specs_for_env(env_id);
    let obs_text = obs_spec.to_detjson();
    let action_text = action_spec.to_detjson();
    let obs_hash = format!("sha256:{}", sha256_hex(obs_text.as_bytes()));
    let action_hash = format!("sha256:{}", sha256_hex(action_text.as_bytes()));
    let provenance = &source.provenance;
    let source_hash = provenance.audit_hash.clone();

    let dataset_header = build_dataset_header_v1(
        env_id,
        &source_hash,
        &provenance.age_target_value,
        source.rng_seed,
        &obs_spec,
        &obs_hash,
        &action_spec,
        &action_hash,
        &provenance.entry_file,
        &provenance.entry_hash,
        &provenance.age_target_source,
    );
    let dataset_header_text = Value::Object(dataset_header).to_string();
    let dataset_header_file_text = dataset_header_text.clone() + "\n";

    let episode_text = build_episode_file_v1(
        env_id,
        source.rng_seed,
        &source_hash,
        &obs_hash,
        &action_hash,
        obs_spec.slot_count,
        &source.state_hashes,
    );

    let dataset_hash_source = format!("{}\n{}", dataset_header_text, episode_text);
    let dataset_hash = format!("sha256:{}", sha256_hex(dataset_hash_source.as_bytes()));
    let dataset_hash_text = format!("{}\n", dataset_hash);
    let obs_path = out_dir.join("obs_spec.detjson");
    let action_path = out_dir.join("action_spec.detjson");
    let header_path = out_dir.join("dataset_header.detjson");
    let episode_path = out_dir.join("episode_000001.detjsonl");
    let hash_path = out_dir.join("dataset_hash.txt");
    write_text_artifacts_atomic(&[
        (obs_path.as_path(), obs_text.as_str()),
        (action_path.as_path(), action_text.as_str()),
        (header_path.as_path(), dataset_header_file_text.as_str()),
        (episode_path.as_path(), episode_text.as_str()),
        (hash_path.as_path(), dataset_hash_text.as_str()),
    ])
    .map_err(|error| format!("E_DATASET_ARTIFACT_SET_WRITE {error}"))?;
    println!("dataset_hash={}", dataset_hash);
    Ok(())
}

fn build_episode_header(
    env_id: &str,
    episode_id: u64,
    seed: u64,
    madi_start: u64,
    madi_end: u64,
    obs_spec_hash: &str,
    action_spec_hash: &str,
) -> String {
    let mut map = Map::new();
    map.insert(
        "schema".to_string(),
        Value::String("nurigym.episode.v1".to_string()),
    );
    map.insert("env_id".to_string(), Value::String(env_id.to_string()));
    map.insert("episode_id".to_string(), Value::Number(episode_id.into()));
    map.insert("seed".to_string(), Value::Number(seed.into()));
    map.insert("madi_start".to_string(), Value::Number(madi_start.into()));
    map.insert("madi_end".to_string(), Value::Number(madi_end.into()));
    map.insert(
        "obs_spec_hash".to_string(),
        Value::String(obs_spec_hash.to_string()),
    );
    map.insert(
        "action_spec_hash".to_string(),
        Value::String(action_spec_hash.to_string()),
    );
    Value::Object(map).to_string()
}

fn build_dataset_header_v1(
    env_id: &str,
    source_hash: &str,
    age_target: &str,
    seed: u64,
    obs_spec: &ObservationSpec,
    obs_hash: &str,
    action_spec: &ActionSpec,
    action_hash: &str,
    entry_file: &str,
    entry_hash: &str,
    age_target_source: &str,
) -> Map<String, Value> {
    let mut action_space = Map::new();
    action_space.insert(
        "schema".to_string(),
        Value::String("nurigym.action_space.v1".to_string()),
    );
    action_space.insert(
        "spec_hash".to_string(),
        Value::String(action_hash.to_string()),
    );
    action_space.insert(
        "actions".to_string(),
        Value::Array(
            action_spec
                .actions
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        ),
    );

    let mut observation_space = Map::new();
    observation_space.insert(
        "schema".to_string(),
        Value::String("nurigym.observation_space.v1".to_string()),
    );
    observation_space.insert("spec_hash".to_string(), Value::String(obs_hash.to_string()));
    observation_space.insert(
        "slot_count".to_string(),
        Value::Number(obs_spec.slot_count.into()),
    );

    let mut source_provenance = Map::new();
    source_provenance.insert(
        "schema".to_string(),
        Value::String("nurigym.source_provenance.v1".to_string()),
    );
    source_provenance.insert(
        "audit_hash".to_string(),
        Value::String(source_hash.to_string()),
    );
    source_provenance.insert(
        "entry_file".to_string(),
        Value::String(entry_file.to_string()),
    );
    source_provenance.insert(
        "entry_hash".to_string(),
        Value::String(entry_hash.to_string()),
    );
    source_provenance.insert(
        "age_target_source".to_string(),
        Value::String(age_target_source.to_string()),
    );

    let mut map = Map::new();
    map.insert(
        "schema".to_string(),
        Value::String("nurigym.dataset_header.v1".to_string()),
    );
    map.insert(
        "version".to_string(),
        Value::String("nurigym_v1".to_string()),
    );
    map.insert("env_id".to_string(), Value::String(env_id.to_string()));
    map.insert(
        "source_hash".to_string(),
        Value::String(source_hash.to_string()),
    );
    map.insert(
        "age_target".to_string(),
        Value::String(age_target.to_string()),
    );
    map.insert("seed".to_string(), Value::Number(seed.into()));
    map.insert("action_space".to_string(), Value::Object(action_space));
    map.insert(
        "observation_space".to_string(),
        Value::Object(observation_space),
    );
    map.insert(
        "source_provenance".to_string(),
        Value::Object(source_provenance),
    );
    map
}

pub fn read_geoul_manifest_provenance(
    geoul_dir: &Path,
    fallback_audit_hash: &str,
) -> Result<DatasetProvenance, String> {
    let manifest_path = geoul_dir.join("manifest.detjson");
    if !manifest_path.exists() {
        return Ok(DatasetProvenance {
            audit_hash: fallback_audit_hash.to_string(),
            entry_file: String::new(),
            entry_hash: String::new(),
            age_target_source: "unknown".to_string(),
            age_target_value: "unknown".to_string(),
        });
    }
    let text = fs::read_to_string(&manifest_path).map_err(|e| {
        format!(
            "E_DATASET_GEOUL_MANIFEST_READ {} ({})",
            manifest_path.display(),
            e
        )
    })?;
    let root: Value = serde_json::from_str(&text).map_err(|e| {
        format!(
            "E_DATASET_GEOUL_MANIFEST_JSON {} ({})",
            manifest_path.display(),
            e
        )
    })?;
    let obj = root.as_object().ok_or_else(|| {
        format!(
            "E_DATASET_GEOUL_MANIFEST_SCHEMA {} 객체가 필요합니다",
            manifest_path.display()
        )
    })?;
    let manifest_audit_hash = obj
        .get("audit_hash")
        .and_then(Value::as_str)
        .unwrap_or(fallback_audit_hash);
    if manifest_audit_hash != fallback_audit_hash {
        return Err(format!(
            "E_DATASET_GEOUL_MANIFEST_AUDIT_HASH {} manifest={} actual={}",
            manifest_path.display(),
            manifest_audit_hash,
            fallback_audit_hash
        ));
    }
    Ok(DatasetProvenance {
        audit_hash: manifest_audit_hash.to_string(),
        entry_file: obj
            .get("entry_file")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        entry_hash: obj
            .get("entry_hash")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        age_target_source: obj
            .get("age_target_source")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        age_target_value: obj
            .get("age_target_value")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex_encode(digest)
}

fn build_dataset_header(
    env_id: &str,
    count: u64,
    source_hash: &str,
    entry_file: &str,
    entry_hash: &str,
    age_target_source: &str,
    age_target_value: &str,
) -> String {
    let mut map = Map::new();
    map.insert(
        "schema".to_string(),
        Value::String("nurigym.dataset.v1".to_string()),
    );
    map.insert("env_id".to_string(), Value::String(env_id.to_string()));
    map.insert(
        "source_hash".to_string(),
        Value::String(source_hash.to_string()),
    );
    let mut source_provenance = Map::new();
    source_provenance.insert(
        "schema".to_string(),
        Value::String("nurigym.source_provenance.v1".to_string()),
    );
    source_provenance.insert(
        "audit_hash".to_string(),
        Value::String(source_hash.to_string()),
    );
    source_provenance.insert(
        "entry_file".to_string(),
        Value::String(entry_file.to_string()),
    );
    source_provenance.insert(
        "entry_hash".to_string(),
        Value::String(entry_hash.to_string()),
    );
    source_provenance.insert(
        "age_target_source".to_string(),
        Value::String(age_target_source.to_string()),
    );
    source_provenance.insert(
        "age_target_value".to_string(),
        Value::String(age_target_value.to_string()),
    );
    map.insert(
        "source_provenance".to_string(),
        Value::Object(source_provenance),
    );
    map.insert("count".to_string(), Value::Number(count.into()));
    Value::Object(map).to_string()
}

fn build_episode_file_v1(
    env_id: &str,
    seed: u64,
    source_hash: &str,
    obs_hash: &str,
    action_hash: &str,
    slot_count: u32,
    state_hashes: &[String],
) -> String {
    let mut lines = Vec::with_capacity(state_hashes.len() + 1);
    let mut episode_header = Map::new();
    episode_header.insert(
        "schema".to_string(),
        Value::String("nurigym.episode.v1".to_string()),
    );
    episode_header.insert("env_id".to_string(), Value::String(env_id.to_string()));
    episode_header.insert("episode_id".to_string(), Value::Number(1u64.into()));
    episode_header.insert("seed".to_string(), Value::Number(seed.into()));
    episode_header.insert(
        "step_count".to_string(),
        Value::Number((state_hashes.len() as u64).into()),
    );
    episode_header.insert(
        "source_hash".to_string(),
        Value::String(source_hash.to_string()),
    );
    episode_header.insert(
        "obs_spec_hash".to_string(),
        Value::String(obs_hash.to_string()),
    );
    episode_header.insert(
        "action_spec_hash".to_string(),
        Value::String(action_hash.to_string()),
    );
    lines.push(Value::Object(episode_header).to_string());

    for (idx, state_hash) in state_hashes.iter().enumerate() {
        let terminal = idx + 1 == state_hashes.len();
        let observation = build_obs_record_v1(state_hash, slot_count);
        let observation_text = Value::Object(observation.clone()).to_string();
        let obs_hash = format!("sha256:{}", sha256_hex(observation_text.as_bytes()));

        let mut action = Map::new();
        action.insert(
            "schema".to_string(),
            Value::String("seulgi.intent.v1".to_string()),
        );
        action.insert("kind".to_string(), Value::String("none".to_string()));

        let mut step = Map::new();
        step.insert(
            "schema".to_string(),
            Value::String("nurigym.step.v1".to_string()),
        );
        step.insert("episode_id".to_string(), Value::Number(1u64.into()));
        step.insert("step".to_string(), Value::Number((idx as u64).into()));
        step.insert("action".to_string(), Value::Object(action));
        step.insert("reward".to_string(), Value::Number(0.into()));
        step.insert("terminal".to_string(), Value::Bool(terminal));
        step.insert(
            "state_hash".to_string(),
            Value::String(state_hash.to_string()),
        );
        step.insert("obs_hash".to_string(), Value::String(obs_hash));
        step.insert("observation".to_string(), Value::Object(observation));
        lines.push(Value::Object(step).to_string());
    }

    lines.join("\n") + "\n"
}

fn build_obs_record_v1(state_hash: &str, slot_count: u32) -> Map<String, Value> {
    let mut obs = Map::new();
    obs.insert(
        "schema".to_string(),
        Value::String("nurigym.obs.v1".to_string()),
    );
    obs.insert(
        "state_hash".to_string(),
        Value::String(state_hash.to_string()),
    );
    obs.insert("slot_count".to_string(), Value::Number(slot_count.into()));
    obs
}

fn build_step_record(
    madi: u64,
    obs_hash: &str,
    next_hash: &str,
    slot_count: u32,
    done: bool,
) -> String {
    let mut obs = Map::new();
    obs.insert(
        "schema".to_string(),
        Value::String("nurigym.obs.v1".to_string()),
    );
    obs.insert(
        "state_hash".to_string(),
        Value::String(obs_hash.to_string()),
    );
    obs.insert("slot_count".to_string(), Value::Number(slot_count.into()));

    let mut next_obs = Map::new();
    next_obs.insert(
        "schema".to_string(),
        Value::String("nurigym.obs.v1".to_string()),
    );
    next_obs.insert(
        "state_hash".to_string(),
        Value::String(next_hash.to_string()),
    );
    next_obs.insert("slot_count".to_string(), Value::Number(slot_count.into()));

    let mut action = Map::new();
    action.insert(
        "schema".to_string(),
        Value::String("seulgi.intent.v1".to_string()),
    );
    action.insert("kind".to_string(), Value::String("none".to_string()));

    let mut map = Map::new();
    map.insert(
        "schema".to_string(),
        Value::String("nurigym.step.v1".to_string()),
    );
    map.insert("episode_id".to_string(), Value::Number(1u64.into()));
    map.insert("agent_id".to_string(), Value::Number(0u64.into()));
    map.insert("madi".to_string(), Value::Number(madi.into()));
    map.insert("observation".to_string(), Value::Object(obs));
    map.insert("action".to_string(), Value::Object(action));
    map.insert("reward".to_string(), Value::Number(0.into()));
    map.insert("next_observation".to_string(), Value::Object(next_obs));
    map.insert("done".to_string(), Value::Bool(done));
    Value::Object(map).to_string()
}

fn canonical_specs_for_env(env_id: &str) -> (ObservationSpec, ActionSpec) {
    match env_id {
        "nurigym.cartpole1d" => (
            ObservationSpec { slot_count: 4 },
            ActionSpec {
                actions: vec!["left".to_string(), "right".to_string()],
            },
        ),
        "nurigym.pendulum1d" => (
            ObservationSpec { slot_count: 2 },
            ActionSpec {
                actions: vec!["left".to_string(), "right".to_string()],
            },
        ),
        "nurigym.gridmaze2d" => (
            ObservationSpec { slot_count: 2 },
            ActionSpec {
                actions: vec![
                    "left".to_string(),
                    "right".to_string(),
                    "down".to_string(),
                    "up".to_string(),
                ],
            },
        ),
        _ => (ObservationSpec::default_k64(), ActionSpec::empty()),
    }
}

#[cfg(test)]
mod tests {
    use super::{run_export_from_source, DatasetExportSource, DatasetProvenance};
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "ddn_dataset_workflow_{label}_{}_{}",
            process::id(),
            nonce
        ))
    }

    fn source() -> DatasetExportSource {
        DatasetExportSource {
            provenance: DatasetProvenance {
                audit_hash: "blake3:source".to_string(),
                entry_file: "entry.ddn".to_string(),
                entry_hash: "sha256:entry".to_string(),
                age_target_source: "manifest".to_string(),
                age_target_value: "AGE5".to_string(),
            },
            state_hashes: vec!["blake3:state0".to_string(), "blake3:state1".to_string()],
            rng_seed: 17,
        }
    }

    fn snapshot(dir: &PathBuf) -> Vec<(String, Vec<u8>)> {
        let mut files = fs::read_dir(dir)
            .expect("read dataset output")
            .map(|entry| entry.expect("dataset entry").path())
            .collect::<Vec<_>>();
        files.sort();
        files
            .into_iter()
            .map(|path| {
                (
                    path.file_name()
                        .expect("dataset file name")
                        .to_string_lossy()
                        .into_owned(),
                    fs::read(path).expect("read dataset artifact"),
                )
            })
            .collect()
    }

    #[test]
    fn shared_dataset_exports_are_deterministic() {
        for (format, expected_names) in [
            (
                "nurigym_v0",
                vec![
                    "action_spec.detjson",
                    "dataset.sha256",
                    "nurigym.dataset.jsonl",
                    "nurigym.episode.jsonl",
                    "obs_spec.detjson",
                ],
            ),
            (
                "nurigym_v1",
                vec![
                    "action_spec.detjson",
                    "dataset_hash.txt",
                    "dataset_header.detjson",
                    "episode_000001.detjsonl",
                    "obs_spec.detjson",
                ],
            ),
        ] {
            let first = test_dir("deterministic_first");
            let second = test_dir("deterministic_second");
            run_export_from_source(&source(), format, &first, "nurigym.gridmaze2d")
                .expect("first dataset export");
            run_export_from_source(&source(), format, &second, "nurigym.gridmaze2d")
                .expect("second dataset export");

            let first_snapshot = snapshot(&first);
            let second_snapshot = snapshot(&second);
            assert_eq!(first_snapshot, second_snapshot);
            assert_eq!(
                first_snapshot
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>(),
                expected_names
            );

            fs::remove_dir_all(first).expect("remove first dataset output");
            fs::remove_dir_all(second).expect("remove second dataset output");
        }
    }

    #[test]
    fn invalid_target_preserves_the_complete_dataset_predecessor() {
        let out = test_dir("atomic_preflight");
        fs::create_dir_all(&out).expect("create dataset output");
        let obs = out.join("obs_spec.detjson");
        let invalid_action = out.join("action_spec.detjson");
        fs::write(&obs, "old-observation\n").expect("write predecessor");
        fs::create_dir(&invalid_action).expect("create invalid action target");

        let error = run_export_from_source(&source(), "nurigym_v0", &out, "nurigym.gridmaze2d")
            .expect_err("directory target must fail closed");
        assert!(error.contains("E_DATASET_ARTIFACT_SET_WRITE"), "{error}");
        assert!(
            error.contains("E_SHARED_ARTIFACT_OUTPUT_TARGET_NOT_FILE"),
            "{error}"
        );
        assert_eq!(
            fs::read_to_string(&obs).expect("read predecessor"),
            "old-observation\n"
        );
        assert_eq!(fs::read_dir(&out).expect("read output").count(), 2);

        fs::remove_dir_all(out).expect("remove dataset output");
    }
}
