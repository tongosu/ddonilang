use std::fs;
use std::path::Path;

use ddonirang_core::fixed64::Fixed64;
use ddonirang_core::mix64;
use ddonirang_core::nurigym::cartpole::{run_episode as run_cartpole_episode, CartPoleConfig, CartPoleEnv};
use ddonirang_core::nurigym::gridmaze::{
    run_episode as run_gridmaze_episode,
    run_episode_with_layout as run_gridmaze_episode_with_layout,
    GridMazeConfig,
    GridMazeEnv,
    GridMazeLayout,
};
use ddonirang_core::nurigym::pendulum::{run_episode as run_pendulum_episode, PendulumConfig, PendulumEnv};
use ddonirang_core::nurigym::spec::{ActionSpec, ObservationSpec};
use serde::Deserialize;
use std::collections::HashMap;

use super::detjson::{read_text, sha256_hex, write_text};

#[derive(Debug, Deserialize)]
struct NuriGymRunInput {
    env_id: Option<String>,
    seed: u64,
    episode_id: u64,
    agent_id: Option<u64>,
    max_steps: Option<u64>,
    actions: Option<Vec<i64>>,
    agents: Option<Vec<NuriGymAgentInput>>,
    shared_env: Option<bool>,
    shared_env_mode: Option<String>,
    merge: Option<String>,
    noop_action: Option<i64>,
    reward_mode: Option<String>,
    reward_weights: Option<HashMap<String, f64>>,
    gridmaze_layouts: Option<Vec<GridMazeLayoutInput>>,
}

#[derive(Debug, Deserialize)]
struct NuriGymAgentInput {
    agent_id: u64,
    actions: Vec<i64>,
    max_steps: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct GridMazeLayoutInput {
    width: i64,
    height: i64,
    goal: [i64; 2],
    obstacles: Option<Vec<[i64; 2]>>,
}

#[derive(Clone, Debug)]
struct AgentRun {
    agent_id: u64,
    actions: Vec<i64>,
    max_steps: Option<u64>,
}

#[derive(Clone, Debug)]
struct AgentCursor {
    agent_id: u64,
    actions: Vec<i64>,
    limit: usize,
    index: usize,
}

#[derive(Clone, Copy, Debug)]
enum SharedEnvMode {
    RoundRobin,
    Sync,
}

#[derive(Clone, Copy, Debug)]
enum SharedMerge {
    SumClamp,
    Majority,
    Priority,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RewardMode {
    Individual,
    Shared,
    Weighted,
}

struct RewardWeights {
    map: HashMap<u64, Fixed64>,
    sum: Fixed64,
    hash: Option<String>,
}

impl SharedEnvMode {
    fn as_option(&self) -> Option<&'static str> {
        match self {
            SharedEnvMode::RoundRobin => Some("round_robin"),
            SharedEnvMode::Sync => Some("sync"),
        }
    }
}

#[derive(Clone, Debug)]
struct StepRecord {
    agent_id: u64,
    observation: Vec<Fixed64>,
    action: i64,
    reward: Fixed64,
    next_observation: Vec<Fixed64>,
    done: bool,
}

pub fn run_spec(from: &Path, out_dir: &Path, slots: Option<u32>) -> Result<(), String> {
    let _ = fs::read_to_string(from).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;

    let obs = if let Some(count) = slots {
        ObservationSpec { slot_count: count }
    } else {
        ObservationSpec::default_k64()
    };
    let action = ActionSpec::empty();

    let obs_text = obs.to_detjson();
    let action_text = action.to_detjson();

    let obs_path = out_dir.join("obs_spec.detjson");
    let action_path = out_dir.join("action_spec.detjson");

    write_text(&obs_path, &format!("{}\n", obs_text))?;
    write_text(&action_path, &format!("{}\n", action_text))?;

    let obs_hash = sha256_hex(obs_text.as_bytes());
    let action_hash = sha256_hex(action_text.as_bytes());
    println!("obs_spec_hash=sha256:{}", obs_hash);
    println!("action_spec_hash=sha256:{}", action_hash);
    Ok(())
}

pub fn run_view(spec: &Path) -> Result<(), String> {
    let text = fs::read_to_string(spec).map_err(|e| e.to_string())?;
    let hash = sha256_hex(text.as_bytes());
    println!("spec_hash=sha256:{}", hash);
    println!("{}", text.trim());
    Ok(())
}

pub fn run_episode_file(input_path: &Path, out_dir: &Path) -> Result<(), String> {
    let text = read_text(input_path)?;
    let input: NuriGymRunInput =
        serde_json::from_str(&text).map_err(|err| format!("E_NURIGYM_INPUT {}", err))?;

    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;

    let env_id = input
        .env_id
        .clone()
        .unwrap_or_else(|| "nurigym.cartpole1d".to_string());
    let shared_env = input.shared_env.unwrap_or(false);
    let shared_env_mode = parse_shared_env_mode(&input);
    let shared_merge = parse_shared_merge(&input);
    let reward_mode = parse_reward_mode(&input);

    let agents = collect_agents(&input)?;
    let reward_weights = build_reward_weights(&input, &agents)?;
    let gridmaze_layout = select_gridmaze_layout(input.seed, input.gridmaze_layouts.as_deref())?;

    let (obs_slot_count, action_spec) = match env_id.as_str() {
        "nurigym.cartpole1d" => (4u32, ActionSpec { actions: vec!["left".to_string(), "right".to_string()] }),
        "nurigym.pendulum1d" => (2u32, ActionSpec { actions: vec!["left".to_string(), "right".to_string()] }),
        "nurigym.gridmaze2d" => (2u32, ActionSpec { actions: vec![
            "left".to_string(),
            "right".to_string(),
            "down".to_string(),
            "up".to_string(),
        ] }),
        other => {
            return Err(format!("E_NURIGYM_ENV unknown env_id={}", other));
        }
    };
    let step_records = if shared_env {
        run_shared_env(
            &env_id,
            input.seed,
            &agents,
            input.max_steps,
            shared_env_mode,
            shared_merge,
            input.noop_action,
            reward_mode,
            &reward_weights,
            gridmaze_layout.clone(),
        )?
    } else {
        run_independent_env(
            &env_id,
            input.seed,
            &agents,
            input.max_steps,
            gridmaze_layout.clone(),
        )?
    };

    let obs_spec = ObservationSpec { slot_count: obs_slot_count };
    let obs_text = obs_spec.to_detjson();
    let action_text = action_spec.to_detjson();
    let obs_hash = format!("sha256:{}", sha256_hex(obs_text.as_bytes()));
    let action_hash = format!("sha256:{}", sha256_hex(action_text.as_bytes()));

    write_text(&out_dir.join("obs_spec.detjson"), &format!("{}\n", obs_text))?;
    write_text(&out_dir.join("action_spec.detjson"), &format!("{}\n", action_text))?;

    let count = step_records.len() as u64;
    let episode_header = build_episode_header(
        &env_id,
        input.episode_id,
        input.seed,
        0,
        count.saturating_sub(1),
        &obs_hash,
        &action_hash,
    );
    write_text(&out_dir.join("nurigym.episode.jsonl"), &format!("{}\n", episode_header))?;

    let agent_ids = collect_agent_ids(&step_records);
    let dataset_header = build_dataset_header(
        &env_id,
        count,
        &agent_ids,
        shared_env,
        shared_env_mode.as_option(),
        if shared_env { Some(reward_mode) } else { None },
        reward_weights.hash.as_deref(),
    );
    let mut dataset_lines = Vec::with_capacity(step_records.len() + 1);
    dataset_lines.push(dataset_header);

    for (idx, step) in step_records.iter().enumerate() {
        let line = build_step_record(
            input.episode_id,
            step.agent_id,
            idx as u64,
            step,
            obs_slot_count,
        );
        dataset_lines.push(line);
    }

    let dataset_text = dataset_lines.join("\n") + "\n";
    write_text(&out_dir.join("nurigym.dataset.jsonl"), &dataset_text)?;

    let dataset_hash = format!("sha256:{}", sha256_hex(dataset_text.as_bytes()));
    write_text(&out_dir.join("dataset.sha256"), &format!("{}\n", dataset_hash))?;

    println!("dataset_hash={}", dataset_hash);
    Ok(())
}

fn collect_agents(input: &NuriGymRunInput) -> Result<Vec<AgentRun>, String> {
    if let Some(list) = &input.agents {
        if list.is_empty() {
            return Err("E_NURIGYM_AGENTS agents must not be empty".to_string());
        }
        let mut out = Vec::with_capacity(list.len());
        for agent in list {
            if agent.actions.is_empty() {
                return Err("E_NURIGYM_ACTIONS actions must not be empty".to_string());
            }
            out.push(AgentRun {
                agent_id: agent.agent_id,
                actions: agent.actions.clone(),
                max_steps: agent.max_steps,
            });
        }
        return Ok(out);
    }

    let actions = input
        .actions
        .as_ref()
        .ok_or_else(|| "E_NURIGYM_ACTIONS actions must not be empty".to_string())?;
    if actions.is_empty() {
        return Err("E_NURIGYM_ACTIONS actions must not be empty".to_string());
    }
    Ok(vec![AgentRun {
        agent_id: input.agent_id.unwrap_or(0),
        actions: actions.clone(),
        max_steps: input.max_steps,
    }])
}

fn collect_agent_ids(steps: &[StepRecord]) -> Vec<u64> {
    let mut ids: Vec<u64> = steps.iter().map(|step| step.agent_id).collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn run_independent_env(
    env_id: &str,
    seed: u64,
    agents: &[AgentRun],
    default_max: Option<u64>,
    gridmaze_layout: Option<GridMazeLayout>,
) -> Result<Vec<StepRecord>, String> {
    let mut step_records = Vec::new();
    for agent in agents.iter() {
        let agent_seed = if agents.len() == 1 {
            seed
        } else {
            mix64(seed, agent.agent_id)
        };
        let steps = match env_id {
            "nurigym.cartpole1d" => run_cartpole(agent_seed, agent, default_max)?,
            "nurigym.pendulum1d" => run_pendulum(agent_seed, agent, default_max)?,
            "nurigym.gridmaze2d" => run_gridmaze_with_layout(agent_seed, agent, default_max, gridmaze_layout.clone())?,
            other => return Err(format!("E_NURIGYM_ENV unknown env_id={}", other)),
        };
        step_records.extend(steps);
    }
    Ok(step_records)
}

fn run_shared_env(
    env_id: &str,
    seed: u64,
    agents: &[AgentRun],
    max_steps: Option<u64>,
    shared_env_mode: SharedEnvMode,
    shared_merge: SharedMerge,
    noop_action: Option<i64>,
    reward_mode: RewardMode,
    reward_weights: &RewardWeights,
    gridmaze_layout: Option<GridMazeLayout>,
) -> Result<Vec<StepRecord>, String> {
    match (env_id, shared_env_mode) {
        ("nurigym.cartpole1d", SharedEnvMode::RoundRobin) => run_shared_cartpole(
            seed,
            agents,
            max_steps,
            reward_mode,
            reward_weights,
            noop_action,
        ),
        ("nurigym.cartpole1d", SharedEnvMode::Sync) => run_shared_cartpole_sync(
            seed,
            agents,
            max_steps,
            shared_merge,
            noop_action,
            reward_mode,
            reward_weights,
        ),
        ("nurigym.pendulum1d", SharedEnvMode::RoundRobin) => run_shared_pendulum(
            seed,
            agents,
            max_steps,
            reward_mode,
            reward_weights,
            noop_action,
        ),
        ("nurigym.pendulum1d", SharedEnvMode::Sync) => run_shared_pendulum_sync(
            seed,
            agents,
            max_steps,
            shared_merge,
            noop_action,
            reward_mode,
            reward_weights,
        ),
        ("nurigym.gridmaze2d", SharedEnvMode::RoundRobin) => run_shared_gridmaze(
            seed,
            agents,
            max_steps,
            gridmaze_layout,
            reward_mode,
            reward_weights,
            noop_action,
        ),
        ("nurigym.gridmaze2d", SharedEnvMode::Sync) => run_shared_gridmaze_sync(
            seed,
            agents,
            max_steps,
            shared_merge,
            noop_action,
            gridmaze_layout,
            reward_mode,
            reward_weights,
        ),
        _ => Err(format!("E_NURIGYM_ENV unknown env_id={}", env_id)),
    }
}

fn run_cartpole(seed: u64, agent: &AgentRun, default_max: Option<u64>) -> Result<Vec<StepRecord>, String> {
    let mut config = CartPoleConfig::default_v1();
    if let Some(max_steps) = agent.max_steps.or(default_max) {
        config.max_steps = max_steps;
    }
    let steps = run_cartpole_episode(seed, &agent.actions, config)?;
    Ok(steps
        .into_iter()
        .map(|step| StepRecord {
            agent_id: agent.agent_id,
            observation: step.observation.to_vec(),
            action: step.action,
            reward: step.reward,
            next_observation: step.next_observation.to_vec(),
            done: step.done,
        })
        .collect())
}

fn run_pendulum(seed: u64, agent: &AgentRun, default_max: Option<u64>) -> Result<Vec<StepRecord>, String> {
    let steps = run_pendulum_episode(seed, &agent.actions, agent.max_steps.or(default_max))?;
    Ok(steps
        .into_iter()
        .map(|step| StepRecord {
            agent_id: agent.agent_id,
            observation: step.observation.to_vec(),
            action: step.action,
            reward: step.reward,
            next_observation: step.next_observation.to_vec(),
            done: step.done,
        })
        .collect())
}

fn run_gridmaze_with_layout(
    seed: u64,
    agent: &AgentRun,
    default_max: Option<u64>,
    layout: Option<GridMazeLayout>,
) -> Result<Vec<StepRecord>, String> {
    let steps = if let Some(layout) = layout {
        run_gridmaze_episode_with_layout(seed, &agent.actions, agent.max_steps.or(default_max), layout)?
    } else {
        run_gridmaze_episode(seed, &agent.actions, agent.max_steps.or(default_max))?
    };
    Ok(steps
        .into_iter()
        .map(|step| StepRecord {
            agent_id: agent.agent_id,
            observation: step.observation.to_vec(),
            action: step.action,
            reward: step.reward,
            next_observation: step.next_observation.to_vec(),
            done: step.done,
        })
        .collect())
}

fn run_shared_cartpole(
    seed: u64,
    agents: &[AgentRun],
    max_steps: Option<u64>,
    reward_mode: RewardMode,
    reward_weights: &RewardWeights,
    noop_action: Option<i64>,
) -> Result<Vec<StepRecord>, String> {
    let mut cursors = build_agent_cursors(agents);
    let total_actions = total_action_capacity(&cursors);

    let mut config = CartPoleConfig::default_v1();
    if let Some(limit) = max_steps {
        config.max_steps = if limit == 0 { total_actions } else { limit };
    }
    if total_actions > 0 && config.max_steps > total_actions {
        config.max_steps = total_actions;
    }
    if config.max_steps == 0 {
        return Ok(Vec::new());
    }

    let mut env = CartPoleEnv::new(seed, config.clone());
    let mut step_records = Vec::new();
    let mut step_count = 0u64;
    let noop = noop_action.unwrap_or(0);

    'outer: loop {
        let mut progressed = false;
        for cursor in cursors.iter_mut() {
            if step_count >= config.max_steps {
                break 'outer;
            }
            if env.is_done() {
                break 'outer;
            }
            if cursor.index >= cursor.limit {
                continue;
            }
            let action = cursor.actions[cursor.index];
            cursor.index += 1;
            let mut step = env.step(action)?;
            let last_step = step_count + 1 >= config.max_steps;
            if last_step && !step.done {
                step.done = true;
                step.reward = Fixed64::ZERO;
            }
            if reward_mode == RewardMode::Individual {
                step_records.push(StepRecord {
                    agent_id: cursor.agent_id,
                    observation: step.observation.to_vec(),
                    action: step.action,
                    reward: step.reward,
                    next_observation: step.next_observation.to_vec(),
                    done: step.done,
                });
            } else {
                let rewards = distribute_rewards(
                    reward_mode,
                    step.reward,
                    agents,
                    Some(cursor.agent_id),
                    &[],
                    noop,
                    reward_weights,
                );
                for (idx, agent) in agents.iter().enumerate() {
                    let action_value = if agent.agent_id == cursor.agent_id {
                        step.action
                    } else {
                        noop
                    };
                    step_records.push(StepRecord {
                        agent_id: agent.agent_id,
                        observation: step.observation.to_vec(),
                        action: action_value,
                        reward: rewards[idx],
                        next_observation: step.next_observation.to_vec(),
                        done: step.done,
                    });
                }
            }
            step_count += 1;
            progressed = true;
            if step.done {
                break 'outer;
            }
        }
        if !progressed {
            break;
        }
    }

    Ok(step_records)
}

fn run_shared_pendulum(
    seed: u64,
    agents: &[AgentRun],
    max_steps: Option<u64>,
    reward_mode: RewardMode,
    reward_weights: &RewardWeights,
    noop_action: Option<i64>,
) -> Result<Vec<StepRecord>, String> {
    let mut cursors = build_agent_cursors(agents);
    let total_actions = total_action_capacity(&cursors);

    let mut config = PendulumConfig::default_v1();
    if let Some(limit) = max_steps {
        config.max_steps = if limit == 0 { total_actions } else { limit };
    }
    if total_actions > 0 && config.max_steps > total_actions {
        config.max_steps = total_actions;
    }
    if config.max_steps == 0 {
        return Ok(Vec::new());
    }

    let mut env = PendulumEnv::new(seed, config.clone());
    let mut step_records = Vec::new();
    let mut step_count = 0u64;
    let noop = noop_action.unwrap_or(0);

    'outer: loop {
        let mut progressed = false;
        for cursor in cursors.iter_mut() {
            if step_count >= config.max_steps {
                break 'outer;
            }
            if env.is_done() {
                break 'outer;
            }
            if cursor.index >= cursor.limit {
                continue;
            }
            let action = cursor.actions[cursor.index];
            cursor.index += 1;
            let mut step = env.step(action)?;
            let last_step = step_count + 1 >= config.max_steps;
            if last_step && !step.done {
                step.done = true;
                step.reward = Fixed64::ZERO;
            }
            if reward_mode == RewardMode::Individual {
                step_records.push(StepRecord {
                    agent_id: cursor.agent_id,
                    observation: step.observation.to_vec(),
                    action: step.action,
                    reward: step.reward,
                    next_observation: step.next_observation.to_vec(),
                    done: step.done,
                });
            } else {
                let rewards = distribute_rewards(
                    reward_mode,
                    step.reward,
                    agents,
                    Some(cursor.agent_id),
                    &[],
                    noop,
                    reward_weights,
                );
                for (idx, agent) in agents.iter().enumerate() {
                    let action_value = if agent.agent_id == cursor.agent_id {
                        step.action
                    } else {
                        noop
                    };
                    step_records.push(StepRecord {
                        agent_id: agent.agent_id,
                        observation: step.observation.to_vec(),
                        action: action_value,
                        reward: rewards[idx],
                        next_observation: step.next_observation.to_vec(),
                        done: step.done,
                    });
                }
            }
            step_count += 1;
            progressed = true;
            if step.done {
                break 'outer;
            }
        }
        if !progressed {
            break;
        }
    }

    Ok(step_records)
}

fn run_shared_gridmaze(
    seed: u64,
    agents: &[AgentRun],
    max_steps: Option<u64>,
    layout: Option<GridMazeLayout>,
    reward_mode: RewardMode,
    reward_weights: &RewardWeights,
    noop_action: Option<i64>,
) -> Result<Vec<StepRecord>, String> {
    let mut cursors = build_agent_cursors(agents);
    let total_actions = total_action_capacity(&cursors);

    let mut config = GridMazeConfig::default_v1();
    if let Some(limit) = max_steps {
        config.max_steps = if limit == 0 { total_actions } else { limit };
    }
    if total_actions > 0 && config.max_steps > total_actions {
        config.max_steps = total_actions;
    }
    if config.max_steps == 0 {
        return Ok(Vec::new());
    }

    let mut env = if let Some(layout) = layout {
        GridMazeEnv::new_with_layout(seed, config.clone(), layout)
    } else {
        GridMazeEnv::new(seed, config.clone())
    };
    let mut step_records = Vec::new();
    let mut step_count = 0u64;
    let noop = noop_action.unwrap_or(0);

    'outer: loop {
        let mut progressed = false;
        for cursor in cursors.iter_mut() {
            if step_count >= config.max_steps {
                break 'outer;
            }
            if env.is_done() {
                break 'outer;
            }
            if cursor.index >= cursor.limit {
                continue;
            }
            let action = cursor.actions[cursor.index];
            cursor.index += 1;
            let mut step = env.step(action)?;
            let last_step = step_count + 1 >= config.max_steps;
            if last_step && !step.done {
                step.done = true;
                step.reward = Fixed64::ZERO;
            }
            if reward_mode == RewardMode::Individual {
                step_records.push(StepRecord {
                    agent_id: cursor.agent_id,
                    observation: step.observation.to_vec(),
                    action: step.action,
                    reward: step.reward,
                    next_observation: step.next_observation.to_vec(),
                    done: step.done,
                });
            } else {
                let rewards = distribute_rewards(
                    reward_mode,
                    step.reward,
                    agents,
                    Some(cursor.agent_id),
                    &[],
                    noop,
                    reward_weights,
                );
                for (idx, agent) in agents.iter().enumerate() {
                    let action_value = if agent.agent_id == cursor.agent_id {
                        step.action
                    } else {
                        noop
                    };
                    step_records.push(StepRecord {
                        agent_id: agent.agent_id,
                        observation: step.observation.to_vec(),
                        action: action_value,
                        reward: rewards[idx],
                        next_observation: step.next_observation.to_vec(),
                        done: step.done,
                    });
                }
            }
            step_count += 1;
            progressed = true;
            if step.done {
                break 'outer;
            }
        }
        if !progressed {
            break;
        }
    }

    Ok(step_records)
}

fn run_shared_cartpole_sync(
    seed: u64,
    agents: &[AgentRun],
    max_steps: Option<u64>,
    merge: SharedMerge,
    noop_action: Option<i64>,
    reward_mode: RewardMode,
    reward_weights: &RewardWeights,
) -> Result<Vec<StepRecord>, String> {
    let mut config = CartPoleConfig::default_v1();
    let limits = agent_limits(agents);
    let mut max_ticks = limits.iter().copied().max().unwrap_or(0) as u64;
    if let Some(limit) = max_steps {
        max_ticks = if limit == 0 { max_ticks } else { limit.min(max_ticks) };
    }
    config.max_steps = max_ticks;
    if config.max_steps == 0 {
        return Ok(Vec::new());
    }

    let allowed = allowed_actions("nurigym.cartpole1d");
    let noop = noop_action.unwrap_or(0);
    let mut env = CartPoleEnv::new(seed, config.clone());
    run_shared_sync_loop(
        &mut env,
        agents,
        &limits,
        config.max_steps,
        merge,
        noop,
        allowed,
        reward_mode,
        reward_weights,
    )
}

fn run_shared_pendulum_sync(
    seed: u64,
    agents: &[AgentRun],
    max_steps: Option<u64>,
    merge: SharedMerge,
    noop_action: Option<i64>,
    reward_mode: RewardMode,
    reward_weights: &RewardWeights,
) -> Result<Vec<StepRecord>, String> {
    let mut config = PendulumConfig::default_v1();
    let limits = agent_limits(agents);
    let mut max_ticks = limits.iter().copied().max().unwrap_or(0) as u64;
    if let Some(limit) = max_steps {
        max_ticks = if limit == 0 { max_ticks } else { limit.min(max_ticks) };
    }
    config.max_steps = max_ticks;
    if config.max_steps == 0 {
        return Ok(Vec::new());
    }

    let allowed = allowed_actions("nurigym.pendulum1d");
    let noop = noop_action.unwrap_or(0);
    let mut env = PendulumEnv::new(seed, config.clone());
    run_shared_sync_loop(
        &mut env,
        agents,
        &limits,
        config.max_steps,
        merge,
        noop,
        allowed,
        reward_mode,
        reward_weights,
    )
}

fn run_shared_gridmaze_sync(
    seed: u64,
    agents: &[AgentRun],
    max_steps: Option<u64>,
    merge: SharedMerge,
    noop_action: Option<i64>,
    layout: Option<GridMazeLayout>,
    reward_mode: RewardMode,
    reward_weights: &RewardWeights,
) -> Result<Vec<StepRecord>, String> {
    let mut config = GridMazeConfig::default_v1();
    let limits = agent_limits(agents);
    let mut max_ticks = limits.iter().copied().max().unwrap_or(0) as u64;
    if let Some(limit) = max_steps {
        max_ticks = if limit == 0 { max_ticks } else { limit.min(max_ticks) };
    }
    config.max_steps = max_ticks;
    if config.max_steps == 0 {
        return Ok(Vec::new());
    }

    let allowed = allowed_actions("nurigym.gridmaze2d");
    let noop = noop_action.unwrap_or(0);
    let mut env = if let Some(layout) = layout {
        GridMazeEnv::new_with_layout(seed, config.clone(), layout)
    } else {
        GridMazeEnv::new(seed, config.clone())
    };
    run_shared_sync_loop(
        &mut env,
        agents,
        &limits,
        config.max_steps,
        merge,
        noop,
        allowed,
        reward_mode,
        reward_weights,
    )
}

fn build_agent_cursors(agents: &[AgentRun]) -> Vec<AgentCursor> {
    agents
        .iter()
        .map(|agent| {
            let limit = effective_limit(agent.actions.len(), agent.max_steps);
            AgentCursor {
                agent_id: agent.agent_id,
                actions: agent.actions.clone(),
                limit,
                index: 0,
            }
        })
        .collect()
}

fn total_action_capacity(cursors: &[AgentCursor]) -> u64 {
    cursors.iter().map(|cursor| cursor.limit as u64).sum()
}

fn effective_limit(actions_len: usize, max_steps: Option<u64>) -> usize {
    match max_steps {
        Some(0) => actions_len,
        Some(limit) => {
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            actions_len.min(limit)
        }
        None => actions_len,
    }
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
    let mut out = String::new();
    out.push_str("{\"schema\":\"nurigym.episode.v1\",\"env_id\":\"");
    out.push_str(env_id);
    out.push_str("\",\"episode_id\":");
    out.push_str(&episode_id.to_string());
    out.push_str(",\"seed\":");
    out.push_str(&seed.to_string());
    out.push_str(",\"madi_start\":");
    out.push_str(&madi_start.to_string());
    out.push_str(",\"madi_end\":");
    out.push_str(&madi_end.to_string());
    out.push_str(",\"obs_spec_hash\":\"");
    out.push_str(obs_spec_hash);
    out.push_str("\",\"action_spec_hash\":\"");
    out.push_str(action_spec_hash);
    out.push_str("\"}");
    out
}

fn build_dataset_header(
    env_id: &str,
    count: u64,
    agent_ids: &[u64],
    shared_env: bool,
    shared_env_mode: Option<&str>,
    reward_mode: Option<RewardMode>,
    reward_weights_hash: Option<&str>,
) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"nurigym.dataset.v1\",\"env_id\":\"");
    out.push_str(env_id);
    out.push_str("\",\"shared_env\":");
    out.push_str(if shared_env { "true" } else { "false" });
    if shared_env {
        if let Some(mode) = shared_env_mode {
            out.push_str(",\"shared_env_mode\":\"");
            out.push_str(mode);
            out.push('"');
        }
        if let Some(mode) = reward_mode {
            out.push_str(",\"reward_mode\":\"");
            out.push_str(match mode {
                RewardMode::Individual => "individual",
                RewardMode::Shared => "shared",
                RewardMode::Weighted => "weighted",
            });
            out.push('"');
        }
        if let Some(hash) = reward_weights_hash {
            out.push_str(",\"reward_weights_hash\":\"");
            out.push_str(hash);
            out.push('"');
        }
    }
    out.push_str(",\"count\":");
    out.push_str(&count.to_string());
    out.push_str(",\"agent_count\":");
    out.push_str(&agent_ids.len().to_string());
    out.push_str(",\"agent_ids\":[");
    for (idx, agent_id) in agent_ids.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&agent_id.to_string());
    }
    out.push_str("]}");
    out
}

fn build_step_record(episode_id: u64, agent_id: u64, madi: u64, step: &StepRecord, slot_count: u32) -> String {
    let (obs_text, _) = build_observation(&step.observation, slot_count);
    let (next_text, _) = build_observation(&step.next_observation, slot_count);

    let mut out = String::new();
    out.push_str("{\"schema\":\"nurigym.step.v1\",\"episode_id\":");
    out.push_str(&episode_id.to_string());
    out.push_str(",\"agent_id\":");
    out.push_str(&agent_id.to_string());
    out.push_str(",\"madi\":");
    out.push_str(&madi.to_string());
    out.push_str(",\"observation\":");
    out.push_str(&obs_text);
    out.push_str(",\"action\":");
    out.push_str(&build_action(step.action));
    out.push_str(",\"reward\":");
    out.push_str(&step.reward.to_string());
    out.push_str(",\"next_observation\":");
    out.push_str(&next_text);
    out.push_str(",\"done\":");
    out.push_str(if step.done { "true" } else { "false" });
    out.push('}');
    out
}

fn build_observation(values: &[Fixed64], slot_count: u32) -> (String, String) {
    let mut base = String::new();
    base.push_str("{\"schema\":\"nurigym.obs.v1\",\"slot_count\":");
    base.push_str(&slot_count.to_string());
    base.push_str(",\"values\":[");
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            base.push(',');
        }
        base.push_str(&value.to_string());
    }
    base.push_str("]}");

    let hash = format!("sha256:{}", sha256_hex(base.as_bytes()));

    let mut out = String::new();
    out.push_str("{\"schema\":\"nurigym.obs.v1\",\"slot_count\":");
    out.push_str(&slot_count.to_string());
    out.push_str(",\"values\":[");
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&value.to_string());
    }
    out.push_str("],\"state_hash\":\"");
    out.push_str(&hash);
    out.push_str("\"}");
    (out, hash)
}

fn build_action(action: i64) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"nurigym.action.v1\",\"value\":");
    out.push_str(&action.to_string());
    out.push('}');
    out
}

fn parse_shared_env_mode(input: &NuriGymRunInput) -> SharedEnvMode {
    match input
        .shared_env_mode
        .as_deref()
        .unwrap_or("round_robin")
    {
        "sync" => SharedEnvMode::Sync,
        _ => SharedEnvMode::RoundRobin,
    }
}

fn parse_shared_merge(input: &NuriGymRunInput) -> SharedMerge {
    match input.merge.as_deref().unwrap_or("sum_clamp") {
        "majority" => SharedMerge::Majority,
        "priority" => SharedMerge::Priority,
        _ => SharedMerge::SumClamp,
    }
}

fn select_gridmaze_layout(
    seed: u64,
    layouts: Option<&[GridMazeLayoutInput]>,
) -> Result<Option<GridMazeLayout>, String> {
    let Some(layouts) = layouts else {
        return Ok(None);
    };
    if layouts.is_empty() {
        return Err("E_NURIGYM_GRIDMAZE_LAYOUT gridmaze_layouts must not be empty".to_string());
    }
    let index = (seed as usize) % layouts.len();
    let layout = &layouts[index];
    let obstacles = layout
        .obstacles
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|pair| (pair[0], pair[1]))
        .collect();
    Ok(Some(GridMazeLayout {
        width: layout.width,
        height: layout.height,
        goal: (layout.goal[0], layout.goal[1]),
        obstacles,
    }))
}

fn agent_limits(agents: &[AgentRun]) -> Vec<usize> {
    agents
        .iter()
        .map(|agent| effective_limit(agent.actions.len(), agent.max_steps))
        .collect()
}

fn allowed_actions(env_id: &str) -> &'static [i64] {
    match env_id {
        "nurigym.gridmaze2d" => &[-2, -1, 1, 2],
        _ => &[-1, 1],
    }
}

fn validate_action(action: i64, allowed: &[i64], noop_action: i64) -> Result<(), String> {
    if allowed.contains(&action) || action == noop_action {
        Ok(())
    } else {
        Err(format!(
            "E_NURIGYM_ACTION_INVALID action={} (allowed={:?})",
            action, allowed
        ))
    }
}

fn merge_actions(
    merge: SharedMerge,
    actions: &[i64],
    noop_action: i64,
    allowed: &[i64],
    last_action: i64,
) -> i64 {
    match merge {
        SharedMerge::Priority => {
            for action in actions.iter().copied() {
                if action != noop_action {
                    return action;
                }
            }
            last_action
        }
        SharedMerge::Majority => {
            let mut counts = std::collections::HashMap::new();
            for action in actions.iter().copied() {
                if action == noop_action {
                    continue;
                }
                *counts.entry(action).or_insert(0usize) += 1;
            }
            let mut best = None;
            for (action, count) in counts {
                match best {
                    None => best = Some((action, count)),
                    Some((_, best_count)) if count > best_count => best = Some((action, count)),
                    _ => {}
                }
            }
            best.map(|(action, _)| action).unwrap_or(last_action)
        }
        SharedMerge::SumClamp => {
            let sum: i64 = actions.iter().copied().sum();
            let min = *allowed.iter().min().unwrap_or(&-1);
            let max = *allowed.iter().max().unwrap_or(&1);
            let mut merged = sum;
            if merged < min {
                merged = min;
            }
            if merged > max {
                merged = max;
            }
            if merged == 0 && !allowed.contains(&merged) {
                last_action
            } else if allowed.contains(&merged) {
                merged
            } else {
                last_action
            }
        }
    }
}

fn run_shared_sync_loop<E>(
    env: &mut E,
    agents: &[AgentRun],
    limits: &[usize],
    max_steps: u64,
    merge: SharedMerge,
    noop_action: i64,
    allowed: &[i64],
    reward_mode: RewardMode,
    reward_weights: &RewardWeights,
) -> Result<Vec<StepRecord>, String>
where
    E: SyncStepEnv,
{
    let mut step_records = Vec::new();
    let mut last_action = allowed.first().copied().unwrap_or(-1);

    for tick in 0..max_steps {
        if env.is_done() {
            break;
        }
        let mut actions = Vec::with_capacity(agents.len());
        for (idx, agent) in agents.iter().enumerate() {
            let limit = limits.get(idx).copied().unwrap_or(0);
            let action = if tick < limit as u64 {
                agent.actions[tick as usize]
            } else {
                noop_action
            };
            validate_action(action, allowed, noop_action)?;
            actions.push(action);
        }
        let merged_action = merge_actions(merge, &actions, noop_action, allowed, last_action);
        let mut step = env.step(merged_action)?;
        let last_tick = tick + 1 >= max_steps;
        if last_tick && !step.done {
            step.done = true;
            step.reward = Fixed64::ZERO;
        }
        if allowed.contains(&merged_action) {
            last_action = merged_action;
        }
        let rewards = distribute_rewards(
            reward_mode,
            step.reward,
            agents,
            None,
            &actions,
            noop_action,
            reward_weights,
        );
        for (idx, agent) in agents.iter().enumerate() {
            let action = actions.get(idx).copied().unwrap_or(noop_action);
            step_records.push(StepRecord {
                agent_id: agent.agent_id,
                observation: step.observation.to_vec(),
                action,
                reward: rewards.get(idx).copied().unwrap_or(step.reward),
                next_observation: step.next_observation.to_vec(),
                done: step.done,
            });
        }
        if step.done {
            break;
        }
    }

    Ok(step_records)
}

trait SyncStepEnv {
    fn is_done(&self) -> bool;
    fn step(&mut self, action: i64) -> Result<SyncStep, String>;
}

struct SyncStep {
    observation: Vec<Fixed64>,
    reward: Fixed64,
    next_observation: Vec<Fixed64>,
    done: bool,
}

impl SyncStepEnv for CartPoleEnv {
    fn is_done(&self) -> bool {
        self.is_done()
    }

    fn step(&mut self, action: i64) -> Result<SyncStep, String> {
        let step = self.step(action)?;
        Ok(SyncStep {
            observation: step.observation.to_vec(),
            reward: step.reward,
            next_observation: step.next_observation.to_vec(),
            done: step.done,
        })
    }
}

impl SyncStepEnv for PendulumEnv {
    fn is_done(&self) -> bool {
        self.is_done()
    }

    fn step(&mut self, action: i64) -> Result<SyncStep, String> {
        let step = self.step(action)?;
        Ok(SyncStep {
            observation: step.observation.to_vec(),
            reward: step.reward,
            next_observation: step.next_observation.to_vec(),
            done: step.done,
        })
    }
}

impl SyncStepEnv for GridMazeEnv {
    fn is_done(&self) -> bool {
        self.is_done()
    }

    fn step(&mut self, action: i64) -> Result<SyncStep, String> {
        let step = self.step(action)?;
        Ok(SyncStep {
            observation: step.observation.to_vec(),
            reward: step.reward,
            next_observation: step.next_observation.to_vec(),
            done: step.done,
        })
    }
}

fn parse_reward_mode(input: &NuriGymRunInput) -> RewardMode {
    match input.reward_mode.as_deref().unwrap_or("individual") {
        "shared" => RewardMode::Shared,
        "weighted" => RewardMode::Weighted,
        _ => RewardMode::Individual,
    }
}

fn build_reward_weights(
    input: &NuriGymRunInput,
    agents: &[AgentRun],
) -> Result<RewardWeights, String> {
    let mut map: HashMap<u64, Fixed64> = HashMap::new();
    let mut raw_pairs: Vec<(u64, i64)> = Vec::new();

    if let Some(weights) = &input.reward_weights {
        for (key, value) in weights {
            let agent_id: u64 = key
                .parse()
                .map_err(|_| "E_NURIGYM_REWARD_WEIGHTS key must be u64".to_string())?;
            let weight_fixed = Fixed64::from_f64_lossy(*value);
            if weight_fixed.raw_i64() <= 0 {
                return Err("E_NURIGYM_REWARD_WEIGHTS weight must be positive".to_string());
            }
            map.insert(agent_id, weight_fixed);
        }
    }

    for agent in agents {
        if !map.contains_key(&agent.agent_id) {
            map.insert(agent.agent_id, Fixed64::ONE);
        }
    }

    let mut sum = Fixed64::ZERO;
    for (agent_id, weight) in map.iter() {
        sum = sum.saturating_add(*weight);
        raw_pairs.push((*agent_id, weight.raw_i64()));
    }
    if sum.raw_i64() == 0 {
        return Err("E_NURIGYM_REWARD_WEIGHTS sum must be positive".to_string());
    }
    raw_pairs.sort_by_key(|pair| pair.0);
    let mut hash_text = String::new();
    for (idx, (agent_id, raw)) in raw_pairs.iter().enumerate() {
        if idx > 0 {
            hash_text.push(';');
        }
        hash_text.push_str(&agent_id.to_string());
        hash_text.push('=');
        hash_text.push_str(&raw.to_string());
    }
    let hash = if hash_text.is_empty() {
        None
    } else {
        Some(format!("sha256:{}", sha256_hex(hash_text.as_bytes())))
    };

    Ok(RewardWeights { map, sum, hash })
}

fn distribute_rewards(
    mode: RewardMode,
    reward: Fixed64,
    agents: &[AgentRun],
    acting_agent: Option<u64>,
    actions: &[i64],
    noop_action: i64,
    weights: &RewardWeights,
) -> Vec<Fixed64> {
    let mut rewards = vec![Fixed64::ZERO; agents.len()];
    match mode {
        RewardMode::Shared => {
            for slot in rewards.iter_mut() {
                *slot = reward;
            }
        }
        RewardMode::Individual => {
            if let Some(agent_id) = acting_agent {
                for (idx, agent) in agents.iter().enumerate() {
                    if agent.agent_id == agent_id {
                        rewards[idx] = reward;
                        break;
                    }
                }
            } else {
                for (idx, _) in agents.iter().enumerate() {
                    let action = actions.get(idx).copied().unwrap_or(noop_action);
                    if action != noop_action {
                        rewards[idx] = reward;
                    }
                }
            }
        }
        RewardMode::Weighted => {
            for (idx, agent) in agents.iter().enumerate() {
                let weight = weights
                    .map
                    .get(&agent.agent_id)
                    .copied()
                    .unwrap_or(Fixed64::ONE);
                let scaled = reward.saturating_mul(weight);
                let scaled = scaled.try_div(weights.sum).unwrap_or(Fixed64::ZERO);
                rewards[idx] = scaled;
            }
        }
    }
    rewards
}
