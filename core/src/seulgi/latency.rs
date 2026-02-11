#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LatencyMode {
    Fixed,
    Jitter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencyPolicy {
    pub l_madi: u64,
    pub mode: LatencyMode,
    pub seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencyEvent {
    pub madi: u64,
    pub deliver_madi: u64,
}

pub fn simulate(policy: LatencyPolicy, count: u64) -> Vec<LatencyEvent> {
    let mut events = Vec::with_capacity(count as usize);
    let mut rng = policy.seed;
    for madi in 0..count {
        let delay = match policy.mode {
            LatencyMode::Fixed => policy.l_madi,
            LatencyMode::Jitter => {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                let jitter = if policy.l_madi == 0 { 0 } else { rng % (policy.l_madi + 1) };
                policy.l_madi.saturating_add(jitter)
            }
        };
        events.push(LatencyEvent {
            madi,
            deliver_madi: madi + delay,
        });
    }
    events
}
