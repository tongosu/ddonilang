use std::fs;
use std::path::Path;

use crate::core::geoul::{decode_input_snapshot, GeoulBundleReader};
use ddonirang_core::nurigym::spec::{ActionSpec, ObservationSpec};
use hex::encode as hex_encode;
use serde_json::{Map, Value};

use super::detjson::{sha256_hex, write_text};

pub fn run_export(geoul_dir: &Path, format: &str, out_dir: &Path, env_id: &str) -> Result<(), String> {
    if format != "nurigym_v0" {
        return Err(format!("E_DATASET_FORMAT 지원하지 않는 format: {}", format));
    }
    fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;

    let obs_spec = ObservationSpec::default_k64();
    let action_spec = ActionSpec::empty();
    let obs_text = obs_spec.to_detjson();
    let action_text = action_spec.to_detjson();
    let obs_hash = format!("sha256:{}", sha256_hex(obs_text.as_bytes()));
    let action_hash = format!("sha256:{}", sha256_hex(action_text.as_bytes()));

    write_text(&out_dir.join("obs_spec.detjson"), &obs_text)?;
    write_text(&out_dir.join("action_spec.detjson"), &action_text)?;

    let mut reader = GeoulBundleReader::open(geoul_dir)?;
    let frame_count = reader.frame_count();
    if frame_count == 0 {
        return Err("E_DATASET_EMPTY geoul 로그에 프레임이 없습니다".to_string());
    }

    let mut state_hashes = Vec::new();
    let mut rng_seed = 0u64;
    for idx in 0..frame_count {
        let frame = reader.read_frame(idx)?;
        let hash = format!("blake3:{}", hex_encode(frame.header.state_hash));
        state_hashes.push(hash);
        if idx == 0 {
            if let Ok(snapshot) = decode_input_snapshot(&frame.snapshot_detbin) {
                rng_seed = snapshot.rng_seed;
            }
        }
    }

    let episode_header = build_episode_header(
        env_id,
        1,
        rng_seed,
        0,
        frame_count.saturating_sub(1),
        &obs_hash,
        &action_hash,
    );
    write_text(&out_dir.join("nurigym.episode.jsonl"), &format!("{}\n", episode_header))?;

    let dataset_header = build_dataset_header(env_id, frame_count as u64);
    let mut dataset_lines = Vec::with_capacity(frame_count as usize + 1);
    dataset_lines.push(dataset_header);

    for idx in 0..frame_count as usize {
        let madi = idx as u64;
        let obs_hash = &state_hashes[idx];
        let next_hash = if idx + 1 < state_hashes.len() {
            &state_hashes[idx + 1]
        } else {
            &state_hashes[idx]
        };
        let done = idx + 1 == state_hashes.len();
        let line = build_step_record(madi, obs_hash, next_hash, obs_spec.slot_count, done);
        dataset_lines.push(line);
    }

    let dataset_text = dataset_lines.join("\n") + "\n";
    let dataset_path = out_dir.join("nurigym.dataset.jsonl");
    write_text(&dataset_path, &dataset_text)?;

    let dataset_hash = format!("sha256:{}", sha256_hex(dataset_text.as_bytes()));
    write_text(&out_dir.join("dataset.sha256"), &format!("{}\n", dataset_hash))?;
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
    map.insert("schema".to_string(), Value::String("nurigym.episode.v1".to_string()));
    map.insert("env_id".to_string(), Value::String(env_id.to_string()));
    map.insert("episode_id".to_string(), Value::Number(episode_id.into()));
    map.insert("seed".to_string(), Value::Number(seed.into()));
    map.insert("madi_start".to_string(), Value::Number(madi_start.into()));
    map.insert("madi_end".to_string(), Value::Number(madi_end.into()));
    map.insert("obs_spec_hash".to_string(), Value::String(obs_spec_hash.to_string()));
    map.insert("action_spec_hash".to_string(), Value::String(action_spec_hash.to_string()));
    Value::Object(map).to_string()
}

fn build_dataset_header(env_id: &str, count: u64) -> String {
    let mut map = Map::new();
    map.insert("schema".to_string(), Value::String("nurigym.dataset.v1".to_string()));
    map.insert("env_id".to_string(), Value::String(env_id.to_string()));
    map.insert("count".to_string(), Value::Number(count.into()));
    Value::Object(map).to_string()
}

fn build_step_record(madi: u64, obs_hash: &str, next_hash: &str, slot_count: u32, done: bool) -> String {
    let mut obs = Map::new();
    obs.insert("schema".to_string(), Value::String("nurigym.obs.v1".to_string()));
    obs.insert("state_hash".to_string(), Value::String(obs_hash.to_string()));
    obs.insert("slot_count".to_string(), Value::Number(slot_count.into()));

    let mut next_obs = Map::new();
    next_obs.insert("schema".to_string(), Value::String("nurigym.obs.v1".to_string()));
    next_obs.insert("state_hash".to_string(), Value::String(next_hash.to_string()));
    next_obs.insert("slot_count".to_string(), Value::Number(slot_count.into()));

    let mut action = Map::new();
    action.insert("schema".to_string(), Value::String("seulgi.intent.v1".to_string()));
    action.insert("kind".to_string(), Value::String("none".to_string()));

    let mut map = Map::new();
    map.insert("schema".to_string(), Value::String("nurigym.step.v1".to_string()));
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