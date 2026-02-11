use crate::fixed64::Fixed64;

#[derive(Clone, Debug)]
pub struct CartPoleConfig {
    pub dt: Fixed64,
    pub force: Fixed64,
    pub friction: Fixed64,
    pub gravity: Fixed64,
    pub angle_stiffness: Fixed64,
    pub angle_damping: Fixed64,
    pub position_limit: Fixed64,
    pub angle_limit: Fixed64,
    pub max_steps: u64,
}

#[derive(Clone, Debug)]
pub struct CartPoleState {
    pub x: Fixed64,
    pub v: Fixed64,
    pub theta: Fixed64,
    pub omega: Fixed64,
}

#[derive(Clone, Debug)]
pub struct CartPoleEnv {
    state: CartPoleState,
    config: CartPoleConfig,
    done: bool,
}

#[derive(Clone, Debug)]
pub struct CartPoleStep {
    pub observation: [Fixed64; 4],
    pub action: i64,
    pub reward: Fixed64,
    pub next_observation: [Fixed64; 4],
    pub done: bool,
}

impl CartPoleConfig {
    pub fn default_v1() -> Self {
        Self {
            dt: fixed_ratio(1, 50),
            force: Fixed64::from_i64(10),
            friction: fixed_ratio(1, 100),
            gravity: fixed_ratio(49, 5),
            angle_stiffness: Fixed64::ONE,
            angle_damping: fixed_ratio(1, 10),
            position_limit: fixed_ratio(12, 5),
            angle_limit: fixed_ratio(209, 1000),
            max_steps: 200,
        }
    }
}

impl CartPoleState {
    pub fn seeded(seed: u64) -> Self {
        let base = splitmix64(seed);
        let x = seed_to_fixed(base, 0);
        let v = seed_to_fixed(base, 16);
        let theta = seed_to_fixed(base, 32);
        let omega = seed_to_fixed(base, 48);
        Self { x, v, theta, omega }
    }

    pub fn observation(&self) -> [Fixed64; 4] {
        [self.x, self.v, self.theta, self.omega]
    }
}

impl CartPoleEnv {
    pub fn new(seed: u64, config: CartPoleConfig) -> Self {
        Self {
            state: CartPoleState::seeded(seed),
            config,
            done: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.done || is_done(&self.state, &self.config)
    }

    pub fn step(&mut self, action: i64) -> Result<CartPoleStep, String> {
        if self.is_done() {
            return Err("E_NURIGYM_DONE cartpole already done".to_string());
        }
        let action = normalize_action(action)?;
        let obs = self.state.observation();
        apply_step(&mut self.state, action, &self.config);
        let next_obs = self.state.observation();
        let done_after = is_done(&self.state, &self.config);
        let reward = if done_after { Fixed64::ZERO } else { Fixed64::ONE };
        self.done = done_after;
        Ok(CartPoleStep {
            observation: obs,
            action,
            reward,
            next_observation: next_obs,
            done: done_after,
        })
    }
}

pub fn run_episode(
    seed: u64,
    actions: &[i64],
    mut config: CartPoleConfig,
) -> Result<Vec<CartPoleStep>, String> {
    if actions.is_empty() {
        return Err("E_NURIGYM_ACTIONS actions must not be empty".to_string());
    }

    if config.max_steps == 0 {
        config.max_steps = actions.len() as u64;
    }

    let mut env = CartPoleEnv::new(seed, config.clone());
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

fn apply_step(state: &mut CartPoleState, action: i64, config: &CartPoleConfig) {
    let act = Fixed64::from_i64(action);

    let accel = act.saturating_mul(config.force)
        .saturating_sub(state.v.saturating_mul(config.friction));
    state.v = state.v + accel.saturating_mul(config.dt);
    state.x = state.x + state.v.saturating_mul(config.dt);

    let ang_accel = act
        .saturating_mul(config.force)
        .saturating_mul(config.angle_stiffness)
        .saturating_sub(state.theta.saturating_mul(config.gravity))
        .saturating_sub(state.omega.saturating_mul(config.angle_damping));
    state.omega = state.omega + ang_accel.saturating_mul(config.dt);
    state.theta = state.theta + state.omega.saturating_mul(config.dt);
}

fn is_done(state: &CartPoleState, config: &CartPoleConfig) -> bool {
    let x_abs = fixed_abs(state.x);
    let theta_abs = fixed_abs(state.theta);
    x_abs.raw_i64() > config.position_limit.raw_i64() || theta_abs.raw_i64() > config.angle_limit.raw_i64()
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
