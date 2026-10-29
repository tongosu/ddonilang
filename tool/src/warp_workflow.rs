use std::path::Path;

pub use ddonirang_core::{WarpBackend, WarpPolicy};
use ddonirang_core::{run_warp_bench, RealmStepInput, StepBatchSoA, WarpBenchInput};
use serde::{Deserialize, Serialize};

use crate::artifact_output::write_text_artifact_atomic;

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

pub fn run_warp_bench_workflow(
    path: &Path,
    backend: WarpBackend,
    policy: WarpPolicy,
    threads: usize,
    measure: bool,
    out: Option<&Path>,
) -> Result<String, String> {
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let input: WarpBenchInputFile =
        serde_json::from_str(&text).map_err(|error| format!("E_WARP_INPUT {error}"))?;
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
        speedup: calculate_speedup(output.cpu_ms, output.gpu_ms),
        realm_count: output.realm_count,
        step_count: output.step_count,
    };
    let json =
        serde_json::to_string_pretty(&view).map_err(|error| format!("E_WARP_OUTPUT {error}"))?;
    let rendered = format!("{json}\n");
    if let Some(out_path) = out {
        write_text_artifact_atomic(out_path, &rendered)
            .map_err(|error| format!("E_WARP_OUTPUT_WRITE {} {error}", out_path.display()))?;
    }
    Ok(rendered)
}

fn calculate_speedup(cpu_ms: u64, gpu_ms: u64) -> f64 {
    cpu_ms as f64 / gpu_ms.max(1) as f64
}
