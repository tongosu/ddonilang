use std::path::Path;

use ddonirang_core::{
    run_warp_bench, RealmStepInput, StepBatchSoA, WarpBackend, WarpBenchInput, WarpPolicy,
};
use serde::{Deserialize, Serialize};

use crate::cli::detjson::read_text;

#[derive(Debug, Deserialize)]
struct WarpBenchInputFile {
    master_seed: u64,
    realm_count: usize,
    steps: u64,
    step_batch: Vec<WarpStepInput>,
}

#[derive(Debug, Deserialize)]
struct WarpStepInput {
    realm_id: usize,
    delta: i64,
}

#[derive(Debug, Serialize)]
struct WarpBenchOutputView {
    cpu_ms: u64,
    gpu_ms: u64,
    speedup: f64,
    realm_count: usize,
    step_count: u64,
}

fn calc_speedup(cpu_ms: u64, gpu_ms: u64) -> f64 {
    let den = gpu_ms.max(1) as f64;
    cpu_ms as f64 / den
}

pub fn run_bench(
    path: &Path,
    backend: WarpBackend,
    policy: WarpPolicy,
    threads: usize,
    measure: bool,
    out: Option<&Path>,
) -> Result<(), String> {
    let text = read_text(path)?;
    let input: WarpBenchInputFile =
        serde_json::from_str(&text).map_err(|err| format!("E_WARP_INPUT {}", err))?;
    let batch_inputs = input
        .step_batch
        .iter()
        .map(|item| RealmStepInput {
            realm_id: item.realm_id,
            delta: item.delta,
        })
        .collect::<Vec<_>>();
    let bench_input = WarpBenchInput {
        master_seed: input.master_seed,
        realm_count: input.realm_count,
        steps: input.steps,
        step_batch: StepBatchSoA::from_inputs(&batch_inputs),
    };

    let output = run_warp_bench(bench_input, backend, policy, threads, measure)?;
    let view = WarpBenchOutputView {
        cpu_ms: output.cpu_ms,
        gpu_ms: output.gpu_ms,
        speedup: calc_speedup(output.cpu_ms, output.gpu_ms),
        realm_count: output.realm_count,
        step_count: output.step_count,
    };
    let json =
        serde_json::to_string_pretty(&view).map_err(|err| format!("E_WARP_OUTPUT {}", err))?;
    if let Some(out_path) = out {
        std::fs::write(out_path, format!("{json}\n")).map_err(|err| err.to_string())?;
    }
    println!("{json}");
    Ok(())
}
