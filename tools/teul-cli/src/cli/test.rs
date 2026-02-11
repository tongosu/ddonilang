use std::path::Path;

use ddonirang_core::realms::{MultiRealmManager, RealmStepInput, ThreadMode};
use hex::encode as hex_encode;
use serde::{Deserialize, Serialize};

use crate::cli::detjson::read_text;

#[derive(Debug, Deserialize)]
struct RealmsTestInput {
    master_seed: u64,
    realm_count: usize,
    steps: u64,
    step_batch: Vec<RealmsStepInput>,
}

#[derive(Debug, Deserialize)]
struct RealmsStepInput {
    realm_id: usize,
    delta: i64,
}

#[derive(Debug, Serialize)]
struct RealmsTestOutput {
    realm_count: usize,
    steps: u64,
    state_hashes: Vec<String>,
}

pub fn run_realms_test(path: &Path, threads: usize, out: Option<&Path>) -> Result<(), String> {
    let text = read_text(path)?;
    let input: RealmsTestInput =
        serde_json::from_str(&text).map_err(|err| format!("E_REALM_INPUT {}", err))?;

    if input.realm_count == 0 {
        return Err("E_REALM_INPUT realm_count must be > 0".to_string());
    }

    let thread_mode = if threads <= 1 {
        ThreadMode::Seq
    } else {
        ThreadMode::Rayon(threads)
    };

    let mut manager =
        MultiRealmManager::new(input.realm_count, input.master_seed, thread_mode);
    let batch: Vec<RealmStepInput> = input
        .step_batch
        .iter()
        .map(|item| RealmStepInput {
            realm_id: item.realm_id,
            delta: item.delta,
        })
        .collect();

    for _ in 0..input.steps {
        manager.step_batch(&batch)?;
    }

    let state_hashes = manager
        .state_hashes()
        .iter()
        .map(|hash| format!("blake3:{}", hex_encode(hash.as_bytes())))
        .collect::<Vec<_>>();

    let output = RealmsTestOutput {
        realm_count: input.realm_count,
        steps: input.steps,
        state_hashes,
    };

    let json = serde_json::to_string_pretty(&output)
        .map_err(|err| format!("E_REALM_OUTPUT {}", err))?;
    if let Some(out_path) = out {
        std::fs::write(out_path, format!("{json}\n")).map_err(|err| err.to_string())?;
    }
    println!("{json}");
    Ok(())
}
