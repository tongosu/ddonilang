use crate::fixed64::Fixed64;

#[derive(Clone, Debug)]
pub struct BanditConfig {
    pub max_steps: u64,
}

#[derive(Clone, Debug)]
pub struct BanditState {
    pub pulls: u64,
    pub preferred_action: i64,
}

#[derive(Clone, Debug)]
pub struct BanditEnv {
    state: BanditState,
    config: BanditConfig,
    done: bool,
}

#[derive(Clone, Debug)]
pub struct BanditStep {
    pub observation: [Fixed64; 2],
    pub action: i64,
    pub reward: Fixed64,
    pub next_observation: [Fixed64; 2],
    pub done: bool,
}

impl BanditConfig {
    pub fn default_v1() -> Self {
        Self { max_steps: 100 }
    }
}

impl BanditState {
    pub fn seeded(seed: u64) -> Self {
        let preferred_action = if splitmix64(seed) & 1 == 0 { -1 } else { 1 };
        Self {
            pulls: 0,
            preferred_action,
        }
    }

    pub fn observation(&self) -> [Fixed64; 2] {
        [
            Fixed64::from_i64(self.pulls as i64),
            Fixed64::from_i64(self.preferred_action),
        ]
    }
}

impl BanditEnv {
    pub fn new(seed: u64, config: BanditConfig) -> Self {
        Self {
            state: BanditState::seeded(seed),
            config,
            done: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.done || self.state.pulls >= self.config.max_steps
    }

    pub fn step(&mut self, action: i64) -> Result<BanditStep, String> {
        if self.is_done() {
            return Err("E_NURIGYM_DONE bandit already done".to_string());
        }
        let action = normalize_action(action)?;
        let obs = self.state.observation();
        let reward = if action == self.state.preferred_action {
            Fixed64::ONE
        } else {
            Fixed64::ZERO
        };
        self.state.pulls = self.state.pulls.saturating_add(1);
        let next_obs = self.state.observation();
        let done_after = self.state.pulls >= self.config.max_steps;
        self.done = done_after;
        Ok(BanditStep {
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
    max_steps: Option<u64>,
) -> Result<Vec<BanditStep>, String> {
    if actions.is_empty() {
        return Err("E_NURIGYM_ACTIONS actions must not be empty".to_string());
    }

    let mut config = BanditConfig::default_v1();
    if let Some(limit) = max_steps {
        config.max_steps = limit;
    }

    let mut env = BanditEnv::new(seed, config.clone());
    let mut steps = Vec::new();

    for (idx, raw_action) in actions.iter().enumerate() {
        if idx as u64 >= config.max_steps {
            break;
        }
        if env.is_done() {
            break;
        }
        let step = env.step(*raw_action)?;
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
        _ => Err(format!(
            "E_NURIGYM_ACTION_INVALID action={} (expected -1 or 1)",
            action
        )),
    }
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::{run_episode, BanditConfig, BanditEnv};
    use crate::fixed64::Fixed64;

    #[test]
    fn nurigym_bandit_rewards_preferred_action() {
        let env = BanditEnv::new(7, BanditConfig { max_steps: 3 });
        let preferred = env.state.preferred_action;
        let other = -preferred;

        let win = run_episode(7, &[preferred], Some(1)).expect("preferred action should run");
        assert_eq!(win.len(), 1);
        assert_eq!(win[0].reward, Fixed64::ONE);
        assert!(win[0].done);

        let lose = run_episode(7, &[other], Some(1)).expect("other action should run");
        assert_eq!(lose.len(), 1);
        assert_eq!(lose[0].reward, Fixed64::ZERO);
        assert!(lose[0].done);
    }

    #[test]
    fn nurigym_bandit_rejects_non_arm_action() {
        let err = run_episode(1, &[0], Some(1)).expect_err("zero is not a bandit arm");
        assert!(err.contains("E_NURIGYM_ACTION_INVALID"));
    }
}
