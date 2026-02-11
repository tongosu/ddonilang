use crate::fixed64::Fixed64;

#[derive(Clone, Debug)]
pub struct PendulumConfig {
    pub dt: Fixed64,
    pub torque: Fixed64,
    pub gravity: Fixed64,
    pub damping: Fixed64,
    pub angle_limit: Fixed64,
    pub max_steps: u64,
}

#[derive(Clone, Debug)]
pub struct PendulumState {
    pub theta: Fixed64,
    pub omega: Fixed64,
}

#[derive(Clone, Debug)]
pub struct PendulumEnv {
    state: PendulumState,
    config: PendulumConfig,
    done: bool,
}

#[derive(Clone, Debug)]
pub struct PendulumStep {
    pub observation: [Fixed64; 2],
    pub action: i64,
    pub reward: Fixed64,
    pub next_observation: [Fixed64; 2],
    pub done: bool,
}

impl PendulumConfig {
    pub fn default_v1() -> Self {
        Self {
            dt: fixed_ratio(1, 50),
            torque: Fixed64::ONE,
            gravity: Fixed64::ONE,
            damping: fixed_ratio(1, 10),
            angle_limit: fixed_ratio(157, 100),
            max_steps: 200,
        }
    }
}

impl PendulumState {
    pub fn seeded(seed: u64) -> Self {
        let base = splitmix64(seed);
        let theta = seed_to_fixed(base, 0);
        let omega = seed_to_fixed(base, 16);
        Self { theta, omega }
    }

    pub fn observation(&self) -> [Fixed64; 2] {
        [self.theta, self.omega]
    }
}

impl PendulumEnv {
    pub fn new(seed: u64, config: PendulumConfig) -> Self {
        Self {
            state: PendulumState::seeded(seed),
            config,
            done: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.done || is_done(&self.state, &self.config)
    }

    pub fn step(&mut self, action: i64) -> Result<PendulumStep, String> {
        if self.is_done() {
            return Err("E_NURIGYM_DONE pendulum already done".to_string());
        }
        let action = normalize_action(action)?;
        let obs = self.state.observation();
        apply_step(&mut self.state, action, &self.config);
        let next_obs = self.state.observation();
        let done_after = is_done(&self.state, &self.config);
        let reward = if done_after { Fixed64::ZERO } else { Fixed64::ONE };
        self.done = done_after;
        Ok(PendulumStep {
            observation: obs,
            action,
            reward,
            next_observation: next_obs,
            done: done_after,
        })
    }
}

pub fn run_episode(seed: u64, actions: &[i64], max_steps: Option<u64>) -> Result<Vec<PendulumStep>, String> {
    if actions.is_empty() {
        return Err("E_NURIGYM_ACTIONS actions must not be empty".to_string());
    }

    let mut config = PendulumConfig::default_v1();
    if let Some(limit) = max_steps {
        config.max_steps = limit;
    }

    let mut env = PendulumEnv::new(seed, config.clone());
    let mut steps = Vec::new();

    for (idx, raw_action) in actions.iter().enumerate() {
        if idx as u64 >= config.max_steps {
            break;
        }
        if env.is_done() {
            break;
        }
        let mut step = env.step(*raw_action)?;
        let last_step = (idx as u64 + 1) >= config.max_steps;
        if last_step && !step.done {
            step.done = true;
            step.reward = Fixed64::ZERO;
        }
        steps.push(step);
        if steps.last().map(|s| s.done).unwrap_or(false) {
            break;
        }
    }

    Ok(steps)
}

fn normalize_action(action: i64) -> Result<i64, String> {
    match action {
        -1 | 1 => Ok(action),
        _ => Err(format!("E_NURIGYM_ACTION_INVALID action={} (expected -1 or 1)", action)),
    }
}

fn apply_step(state: &mut PendulumState, action: i64, config: &PendulumConfig) {
    let act = Fixed64::from_i64(action);
    let torque = act.saturating_mul(config.torque);
    let accel = torque
        .saturating_sub(state.theta.saturating_mul(config.gravity))
        .saturating_sub(state.omega.saturating_mul(config.damping));
    state.omega = state.omega + accel.saturating_mul(config.dt);
    state.theta = state.theta + state.omega.saturating_mul(config.dt);
}

fn is_done(state: &PendulumState, config: &PendulumConfig) -> bool {
    fixed_abs(state.theta).raw_i64() > config.angle_limit.raw_i64()
}

fn fixed_abs(value: Fixed64) -> Fixed64 {
    if value.raw_i64() < 0 {
        Fixed64::from_raw_i64(value.raw_i64().saturating_neg())
    } else {
        value
    }
}

fn seed_to_fixed(seed: u64, shift: u32) -> Fixed64 {
    let bits = ((seed >> shift) & 0xFFFF) as i64;
    let centered = bits - 0x8000;
    let raw = centered << 16;
    Fixed64::from_raw_i64(raw)
}

fn fixed_ratio(num: i64, den: i64) -> Fixed64 {
    if den == 0 {
        return Fixed64::ZERO;
    }
    let raw = (num as i128)
        .saturating_mul(Fixed64::ONE_RAW as i128)
        .saturating_div(den as i128);
    if raw > i64::MAX as i128 {
        Fixed64::MAX
    } else if raw < i64::MIN as i128 {
        Fixed64::MIN
    } else {
        Fixed64::from_raw_i64(raw as i64)
    }
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
