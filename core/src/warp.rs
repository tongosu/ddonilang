use crate::realms::{MultiRealmManager, RealmStepInput, ThreadMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WarpBackend {
    Off,
    Cpu,
    Gpu,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WarpPolicy {
    Strict,
    Fast,
}

#[derive(Clone, Debug)]
pub struct StepBatchSoA {
    pub realm_ids: Vec<usize>,
    pub deltas: Vec<i64>,
}

impl StepBatchSoA {
    pub fn from_inputs(inputs: &[RealmStepInput]) -> Self {
        Self {
            realm_ids: inputs.iter().map(|item| item.realm_id).collect(),
            deltas: inputs.iter().map(|item| item.delta).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.realm_ids.len()
    }

    pub fn to_inputs(&self) -> Result<Vec<RealmStepInput>, String> {
        if self.realm_ids.len() != self.deltas.len() {
            return Err("E_WARP_SOA_LENGTH realm_ids and deltas length mismatch".to_string());
        }
        Ok(self
            .realm_ids
            .iter()
            .zip(self.deltas.iter())
            .map(|(realm_id, delta)| RealmStepInput {
                realm_id: *realm_id,
                delta: *delta,
            })
            .collect())
    }
}

#[derive(Clone, Debug)]
pub struct WarpBenchInput {
    pub master_seed: u64,
    pub realm_count: usize,
    pub steps: u64,
    pub step_batch: StepBatchSoA,
}

#[derive(Clone, Debug)]
pub struct WarpBenchOutput {
    pub cpu_ms: u64,
    pub gpu_ms: u64,
    pub realm_count: usize,
    pub step_count: u64,
}

fn estimate_ms(realm_count: usize, steps: u64, divisor: u64) -> u64 {
    let base = (realm_count as u64).saturating_mul(steps.max(1));
    let div = divisor.max(1);
    let value = base / div;
    value.max(1)
}

fn run_steps(
    realm_count: usize,
    master_seed: u64,
    steps: u64,
    batch: &StepBatchSoA,
    thread_mode: ThreadMode,
) -> Result<(), String> {
    let mut manager = MultiRealmManager::new(realm_count, master_seed, thread_mode);
    let inputs = batch.to_inputs()?;
    for _ in 0..steps {
        manager.step_batch(&inputs)?;
    }
    Ok(())
}

pub fn run_warp_bench(
    input: WarpBenchInput,
    backend: WarpBackend,
    policy: WarpPolicy,
    threads: usize,
    measure: bool,
) -> Result<WarpBenchOutput, String> {
    if input.realm_count == 0 {
        return Err("E_WARP_INPUT realm_count must be > 0".to_string());
    }

    let mut cpu_ms = estimate_ms(input.realm_count, input.steps, 1);
    let use_gpu = matches!(backend, WarpBackend::Gpu) && matches!(policy, WarpPolicy::Fast);
    let mut gpu_ms = if use_gpu {
        estimate_ms(input.realm_count, input.steps, 4)
    } else {
        cpu_ms
    };

    if measure {
        let start = std::time::Instant::now();
        run_steps(
            input.realm_count,
            input.master_seed,
            input.steps,
            &input.step_batch,
            ThreadMode::Seq,
        )?;
        let elapsed = start.elapsed();
        cpu_ms = elapsed.as_millis().max(1) as u64;
    } else {
        run_steps(
            input.realm_count,
            input.master_seed,
            input.steps,
            &input.step_batch,
            ThreadMode::Seq,
        )?;
    }

    if use_gpu {
        let thread_mode = if threads <= 1 {
            ThreadMode::Seq
        } else {
            ThreadMode::Rayon(threads)
        };
        if measure {
            let start = std::time::Instant::now();
            run_steps(
                input.realm_count,
                input.master_seed,
                input.steps,
                &input.step_batch,
                thread_mode,
            )?;
            let elapsed = start.elapsed();
            gpu_ms = elapsed.as_millis().max(1) as u64;
        } else {
            run_steps(
                input.realm_count,
                input.master_seed,
                input.steps,
                &input.step_batch,
                thread_mode,
            )?;
        }
    } else if measure {
        gpu_ms = cpu_ms;
    }

    Ok(WarpBenchOutput {
        cpu_ms,
        gpu_ms,
        realm_count: input.realm_count,
        step_count: input.steps,
    })
}
