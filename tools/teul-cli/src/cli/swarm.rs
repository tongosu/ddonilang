use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use blake3;
use serde::Deserialize;

use super::detjson::write_text;
use super::paths;

#[derive(Deserialize)]
struct SwarmCollisionInput {
    schema: Option<String>,
    agents: Vec<AgentInput>,
    actions: Option<Vec<ActionInput>>,
}

#[derive(Deserialize, Clone)]
struct AgentInput {
    agent_id: u64,
    x: i64,
    y: i64,
}

#[derive(Deserialize)]
struct ActionInput {
    agent_id: u64,
    dx: i64,
    dy: i64,
}

#[derive(Clone)]
struct AgentState {
    agent_id: u64,
    x: i64,
    y: i64,
}

#[derive(Clone)]
struct CollisionPair {
    a: u64,
    b: u64,
    x: i64,
    y: i64,
}

pub fn run_collision(input: &Path, out_dir: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(input)
        .map_err(|e| format!("E_SWARM_INPUT_READ {} {}", input.display(), e))?;
    let input: SwarmCollisionInput =
        serde_json::from_str(&text).map_err(|e| format!("E_SWARM_INPUT_PARSE {}", e))?;
    if let Some(schema) = input.schema.as_deref() {
        if schema != "swarm.collision_check.v1" {
            return Err(format!("E_SWARM_SCHEMA {}", schema));
        }
    }

    let agents = build_agents(&input.agents)?;
    let actions = build_actions(input.actions.unwrap_or_default())?;
    let mut updated = apply_actions(&agents, &actions)?;
    updated.sort_by_key(|agent| agent.agent_id);

    let collisions = detect_collisions(&updated);
    let state_hash = compute_state_hash(&updated);
    let log = build_log(&updated, &collisions, &state_hash);

    let out_dir = resolve_out_dir(out_dir);
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    write_text(&out_dir.join("swarm.collision.detjson"), &log)?;

    println!("{}", log);
    Ok(())
}

fn resolve_out_dir(out_dir: Option<&Path>) -> PathBuf {
    match out_dir {
        Some(path) => path.to_path_buf(),
        None => paths::build_dir().join("swarm"),
    }
}

fn build_agents(inputs: &[AgentInput]) -> Result<Vec<AgentState>, String> {
    if inputs.is_empty() {
        return Err("E_SWARM_EMPTY agents가 비었습니다".to_string());
    }
    let mut seen = BTreeSet::new();
    let mut agents = Vec::with_capacity(inputs.len());
    for agent in inputs {
        if !seen.insert(agent.agent_id) {
            return Err(format!("E_SWARM_AGENT_DUPLICATE {}", agent.agent_id));
        }
        agents.push(AgentState {
            agent_id: agent.agent_id,
            x: agent.x,
            y: agent.y,
        });
    }
    Ok(agents)
}

fn build_actions(inputs: Vec<ActionInput>) -> Result<BTreeMap<u64, (i64, i64)>, String> {
    let mut map = BTreeMap::new();
    for action in inputs {
        if map.contains_key(&action.agent_id) {
            return Err(format!("E_SWARM_ACTION_DUPLICATE {}", action.agent_id));
        }
        map.insert(action.agent_id, (action.dx, action.dy));
    }
    Ok(map)
}

fn apply_actions(
    agents: &[AgentState],
    actions: &BTreeMap<u64, (i64, i64)>,
) -> Result<Vec<AgentState>, String> {
    let mut updated = agents.to_vec();
    updated.sort_by_key(|agent| agent.agent_id);

    for agent in updated.iter_mut() {
        if let Some((dx, dy)) = actions.get(&agent.agent_id) {
            agent.x = agent.x.saturating_add(*dx);
            agent.y = agent.y.saturating_add(*dy);
        }
    }

    for action_id in actions.keys() {
        if !updated.iter().any(|agent| &agent.agent_id == action_id) {
            return Err(format!("E_SWARM_ACTION_UNKNOWN {}", action_id));
        }
    }

    Ok(updated)
}

fn detect_collisions(agents: &[AgentState]) -> Vec<CollisionPair> {
    let mut collisions = Vec::new();
    for i in 0..agents.len() {
        for j in (i + 1)..agents.len() {
            let a = &agents[i];
            let b = &agents[j];
            if a.x == b.x && a.y == b.y {
                let (min_id, max_id) = if a.agent_id <= b.agent_id {
                    (a.agent_id, b.agent_id)
                } else {
                    (b.agent_id, a.agent_id)
                };
                collisions.push(CollisionPair {
                    a: min_id,
                    b: max_id,
                    x: a.x,
                    y: a.y,
                });
            }
        }
    }
    collisions.sort_by_key(|pair| (pair.a, pair.b));
    collisions
}

fn compute_state_hash(agents: &[AgentState]) -> String {
    let mut parts = Vec::with_capacity(agents.len());
    for agent in agents {
        parts.push(format!("{}:{}:{}", agent.agent_id, agent.x, agent.y));
    }
    let joined = parts.join("|");
    let hash = blake3::hash(joined.as_bytes());
    format!("blake3:{}", hash.to_hex())
}

fn build_log(agents: &[AgentState], collisions: &[CollisionPair], state_hash: &str) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"swarm.collision_log.v1\"");
    out.push_str(",\"agent_count\":");
    out.push_str(&agents.len().to_string());
    out.push_str(",\"state_hash\":\"");
    out.push_str(state_hash);
    out.push_str("\",\"collisions\":[");
    for (idx, pair) in collisions.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str("{\"a\":");
        out.push_str(&pair.a.to_string());
        out.push_str(",\"b\":");
        out.push_str(&pair.b.to_string());
        out.push_str(",\"x\":");
        out.push_str(&pair.x.to_string());
        out.push_str(",\"y\":");
        out.push_str(&pair.y.to_string());
        out.push('}');
    }
    out.push_str("]}");
    out
}
