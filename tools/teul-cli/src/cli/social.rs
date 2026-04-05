use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use blake3;
use serde::{Deserialize, Serialize};

use super::detjson::write_text;
use super::paths;

const SOCIAL_WORLD_SCHEMA: &str = "ddn.social.world.v1";
const SOCIAL_REPORT_SCHEMA: &str = "ddn.social.report.v1";

#[derive(Deserialize)]
struct SocialWorldInput {
    schema: Option<String>,
    seed: Option<String>,
    steps: u64,
    max_conflicts: Option<u64>,
    agents: Vec<SocialAgentInput>,
    events: Option<Vec<SocialEventInput>>,
}

#[derive(Deserialize)]
struct SocialAgentInput {
    agent_id: u64,
    wealth: i64,
    trust: i64,
    culture: i64,
}

#[derive(Deserialize)]
struct SocialEventInput {
    step: u64,
    kind: String,
    from: Option<u64>,
    to: Option<u64>,
    a: Option<u64>,
    b: Option<u64>,
    amount: Option<i64>,
    intensity: Option<i64>,
    boost: Option<i64>,
}

#[derive(Clone)]
struct SocialAgentState {
    wealth: i64,
    trust: i64,
    culture: i64,
}

#[derive(Default)]
struct SocialEventCounters {
    redistribution: u64,
    conflict: u64,
    cooperation: u64,
}

#[derive(Serialize)]
struct SocialMetrics {
    total_wealth: i64,
    min_wealth: i64,
    max_wealth: i64,
    inequality_gap: i64,
    avg_trust: i64,
    trust_span: i64,
}

#[derive(Serialize)]
struct SocialEventStats {
    total_events: u64,
    redistribution: u64,
    conflict: u64,
    cooperation: u64,
}

#[derive(Serialize, Clone)]
struct SocialSourceProvenance {
    schema: &'static str,
    source_kind: &'static str,
    input_file: String,
    input_hash: String,
}

#[derive(Serialize)]
struct SocialReportSeed {
    schema: &'static str,
    source_hash: String,
    source_provenance: SocialSourceProvenance,
    input_schema: &'static str,
    steps: u64,
    agent_count: usize,
    metrics: SocialMetrics,
    event_stats: SocialEventStats,
    final_state_hash: String,
}

#[derive(Serialize)]
struct SocialReport {
    schema: &'static str,
    source_hash: String,
    source_provenance: SocialSourceProvenance,
    input_schema: &'static str,
    steps: u64,
    agent_count: usize,
    metrics: SocialMetrics,
    event_stats: SocialEventStats,
    final_state_hash: String,
    social_report_hash: String,
}

pub fn run_simulate(input: &Path, out_dir: Option<&Path>) -> Result<(), String> {
    let input_bytes =
        fs::read(input).map_err(|e| format!("E_SOCIAL_INPUT_READ {} ({})", input.display(), e))?;
    let text = String::from_utf8(input_bytes.clone())
        .map_err(|_| format!("E_SOCIAL_INPUT_UTF8 {}", input.display()))?;
    let doc: SocialWorldInput =
        serde_json::from_str(&text).map_err(|e| format!("E_SOCIAL_INPUT_PARSE {}", e))?;

    if let Some(schema) = doc.schema.as_deref() {
        if schema != SOCIAL_WORLD_SCHEMA {
            return Err(format!("E_SOCIAL_SCHEMA {}", schema));
        }
    }
    if doc.steps == 0 {
        return Err("E_SOCIAL_STEPS_INVALID steps must be >= 1".to_string());
    }
    if doc.agents.is_empty() {
        return Err("E_SOCIAL_AGENTS_EMPTY agents must not be empty".to_string());
    }

    let seed = parse_seed(doc.seed.as_deref().unwrap_or("0x0"))?;
    let mut agents = build_agents(&doc.agents)?;
    let events = doc.events.unwrap_or_default();
    let max_conflicts = doc.max_conflicts.unwrap_or(10_000);
    let mut counters = SocialEventCounters::default();

    let mut ordered_events: Vec<(u64, usize)> = Vec::with_capacity(events.len());
    for (idx, event) in events.iter().enumerate() {
        if event.step == 0 || event.step > doc.steps {
            return Err(format!(
                "E_SOCIAL_EVENT_STEP id={} step={} steps={}",
                idx, event.step, doc.steps
            ));
        }
        let kind = event.kind.trim();
        if kind.is_empty() {
            return Err(format!("E_SOCIAL_EVENT_KIND_EMPTY id={}", idx));
        }
        ordered_events.push((event.step, idx));
    }
    ordered_events.sort_by_key(|(step, idx)| (*step, *idx));

    for step in 1..=doc.steps {
        apply_step_drift(&mut agents, seed, step);
        for (_, idx) in ordered_events
            .iter()
            .filter(|(event_step, _)| *event_step == step)
        {
            let event = &events[*idx];
            apply_event(event, *idx, &mut agents, &mut counters, max_conflicts)?;
        }
    }

    let final_state_hash = compute_state_hash(&agents);
    let metrics = build_metrics(&agents);
    let event_stats = SocialEventStats {
        total_events: counters.redistribution + counters.conflict + counters.cooperation,
        redistribution: counters.redistribution,
        conflict: counters.conflict,
        cooperation: counters.cooperation,
    };
    let source_hash = format!("sha256:{}", super::detjson::sha256_hex(&input_bytes));
    let source_provenance = SocialSourceProvenance {
        schema: "ddn.social.source_provenance.v1",
        source_kind: "social_world.v1",
        input_file: input.to_string_lossy().replace('\\', "/"),
        input_hash: source_hash.clone(),
    };

    let seed_report = SocialReportSeed {
        schema: SOCIAL_REPORT_SCHEMA,
        source_hash: source_hash.clone(),
        source_provenance: source_provenance.clone(),
        input_schema: SOCIAL_WORLD_SCHEMA,
        steps: doc.steps,
        agent_count: agents.len(),
        metrics,
        event_stats,
        final_state_hash: final_state_hash.clone(),
    };
    let seed_text = serde_json::to_string(&seed_report)
        .map_err(|e| format!("E_SOCIAL_REPORT_SERIALIZE {}", e))?;
    let social_report_hash = format!("blake3:{}", blake3::hash(seed_text.as_bytes()).to_hex());

    let report = SocialReport {
        schema: SOCIAL_REPORT_SCHEMA,
        source_hash,
        source_provenance,
        input_schema: SOCIAL_WORLD_SCHEMA,
        steps: seed_report.steps,
        agent_count: seed_report.agent_count,
        metrics: seed_report.metrics,
        event_stats: seed_report.event_stats,
        final_state_hash: final_state_hash.clone(),
        social_report_hash: social_report_hash.clone(),
    };
    let report_text =
        serde_json::to_string(&report).map_err(|e| format!("E_SOCIAL_REPORT_SERIALIZE {}", e))?;

    let out_root = resolve_out_dir(out_dir);
    fs::create_dir_all(&out_root)
        .map_err(|e| format!("E_SOCIAL_OUT_DIR_CREATE {} ({})", out_root.display(), e))?;
    let out_file = out_root.join("social_report.detjson");
    write_text(&out_file, &report_text)?;

    println!("social_report_out={}", out_file.display());
    println!("social_report_hash={}", social_report_hash);
    println!("final_state_hash={}", final_state_hash);
    Ok(())
}

fn resolve_out_dir(out_dir: Option<&Path>) -> PathBuf {
    match out_dir {
        Some(path) => path.to_path_buf(),
        None => paths::build_dir().join("social"),
    }
}

fn build_agents(inputs: &[SocialAgentInput]) -> Result<BTreeMap<u64, SocialAgentState>, String> {
    let mut out: BTreeMap<u64, SocialAgentState> = BTreeMap::new();
    for agent in inputs {
        if out.contains_key(&agent.agent_id) {
            return Err(format!("E_SOCIAL_AGENT_DUPLICATE {}", agent.agent_id));
        }
        out.insert(
            agent.agent_id,
            SocialAgentState {
                wealth: agent.wealth,
                trust: clamp_i64(agent.trust, -100, 100),
                culture: agent.culture,
            },
        );
    }
    Ok(out)
}

fn parse_seed(raw: &str) -> Result<u64, String> {
    let trimmed = raw.trim();
    if let Some(hex) = trimmed.strip_prefix("0x") {
        return u64::from_str_radix(hex, 16).map_err(|e| format!("E_SOCIAL_SEED_PARSE {}", e));
    }
    trimmed
        .parse::<u64>()
        .map_err(|e| format!("E_SOCIAL_SEED_PARSE {}", e))
}

fn apply_step_drift(agents: &mut BTreeMap<u64, SocialAgentState>, seed: u64, step: u64) {
    let ids: Vec<u64> = agents.keys().copied().collect();
    for agent_id in ids {
        if let Some(agent) = agents.get_mut(&agent_id) {
            let delta = deterministic_delta(seed, step, agent_id);
            agent.wealth = agent.wealth.saturating_add(delta);
            agent.trust = clamp_i64(agent.trust.saturating_add(delta), -100, 100);
        }
    }
}

fn apply_event(
    event: &SocialEventInput,
    idx: usize,
    agents: &mut BTreeMap<u64, SocialAgentState>,
    counters: &mut SocialEventCounters,
    max_conflicts: u64,
) -> Result<(), String> {
    match event.kind.trim() {
        "redistribute" => {
            let from_id = event
                .from
                .ok_or_else(|| format!("E_SOCIAL_EVENT_FIELD id={} missing=from", idx))?;
            let to_id = event
                .to
                .ok_or_else(|| format!("E_SOCIAL_EVENT_FIELD id={} missing=to", idx))?;
            let amount = event
                .amount
                .ok_or_else(|| format!("E_SOCIAL_EVENT_FIELD id={} missing=amount", idx))?;
            if amount <= 0 {
                return Err(format!("E_SOCIAL_EVENT_VALUE id={} amount={}", idx, amount));
            }

            let mut from = agents
                .get(&from_id)
                .cloned()
                .ok_or_else(|| format!("E_SOCIAL_AGENT_UNKNOWN id={} agent={}", idx, from_id))?;
            let mut to = agents
                .get(&to_id)
                .cloned()
                .ok_or_else(|| format!("E_SOCIAL_AGENT_UNKNOWN id={} agent={}", idx, to_id))?;

            let transferable = from.wealth.max(0).min(amount);
            from.wealth = from.wealth.saturating_sub(transferable);
            to.wealth = to.wealth.saturating_add(transferable);

            agents.insert(from_id, from);
            agents.insert(to_id, to);
            counters.redistribution = counters.redistribution.saturating_add(1);
            Ok(())
        }
        "conflict" => {
            let a_id = event
                .a
                .ok_or_else(|| format!("E_SOCIAL_EVENT_FIELD id={} missing=a", idx))?;
            let b_id = event
                .b
                .ok_or_else(|| format!("E_SOCIAL_EVENT_FIELD id={} missing=b", idx))?;
            let intensity = event
                .intensity
                .ok_or_else(|| format!("E_SOCIAL_EVENT_FIELD id={} missing=intensity", idx))?;
            if intensity <= 0 {
                return Err(format!(
                    "E_SOCIAL_EVENT_VALUE id={} intensity={}",
                    idx, intensity
                ));
            }

            apply_pair_update(agents, idx, a_id, b_id, |left, right| {
                left.trust = clamp_i64(left.trust.saturating_sub(intensity), -100, 100);
                left.wealth = left.wealth.saturating_sub(intensity / 2);
                if let Some(target) = right {
                    target.trust = clamp_i64(target.trust.saturating_sub(intensity), -100, 100);
                    target.wealth = target.wealth.saturating_sub(intensity / 2);
                }
            })?;

            counters.conflict = counters.conflict.saturating_add(1);
            if counters.conflict > max_conflicts {
                return Err(format!(
                    "E_SOCIAL_EXPLOSION conflicts={} max_conflicts={}",
                    counters.conflict, max_conflicts
                ));
            }
            Ok(())
        }
        "cooperate" => {
            let a_id = event
                .a
                .ok_or_else(|| format!("E_SOCIAL_EVENT_FIELD id={} missing=a", idx))?;
            let b_id = event
                .b
                .ok_or_else(|| format!("E_SOCIAL_EVENT_FIELD id={} missing=b", idx))?;
            let boost = event
                .boost
                .ok_or_else(|| format!("E_SOCIAL_EVENT_FIELD id={} missing=boost", idx))?;
            if boost <= 0 {
                return Err(format!("E_SOCIAL_EVENT_VALUE id={} boost={}", idx, boost));
            }

            apply_pair_update(agents, idx, a_id, b_id, |left, right| {
                left.trust = clamp_i64(left.trust.saturating_add(boost), -100, 100);
                left.wealth = left.wealth.saturating_add(boost / 2);
                if let Some(target) = right {
                    target.trust = clamp_i64(target.trust.saturating_add(boost), -100, 100);
                    target.wealth = target.wealth.saturating_add(boost / 2);
                }
            })?;

            counters.cooperation = counters.cooperation.saturating_add(1);
            Ok(())
        }
        kind => Err(format!("E_SOCIAL_EVENT_KIND id={} kind={}", idx, kind)),
    }
}

fn apply_pair_update<F>(
    agents: &mut BTreeMap<u64, SocialAgentState>,
    idx: usize,
    a_id: u64,
    b_id: u64,
    mut f: F,
) -> Result<(), String>
where
    F: FnMut(&mut SocialAgentState, Option<&mut SocialAgentState>),
{
    let mut left = agents
        .get(&a_id)
        .cloned()
        .ok_or_else(|| format!("E_SOCIAL_AGENT_UNKNOWN id={} agent={}", idx, a_id))?;
    if a_id == b_id {
        f(&mut left, None);
        agents.insert(a_id, left);
        return Ok(());
    }
    let mut right = agents
        .get(&b_id)
        .cloned()
        .ok_or_else(|| format!("E_SOCIAL_AGENT_UNKNOWN id={} agent={}", idx, b_id))?;
    f(&mut left, Some(&mut right));
    agents.insert(a_id, left);
    agents.insert(b_id, right);
    Ok(())
}

fn build_metrics(agents: &BTreeMap<u64, SocialAgentState>) -> SocialMetrics {
    let mut total_wealth = 0i64;
    let mut min_wealth = i64::MAX;
    let mut max_wealth = i64::MIN;
    let mut total_trust = 0i64;
    let mut min_trust = i64::MAX;
    let mut max_trust = i64::MIN;

    for state in agents.values() {
        total_wealth = total_wealth.saturating_add(state.wealth);
        total_trust = total_trust.saturating_add(state.trust);
        min_wealth = min_wealth.min(state.wealth);
        max_wealth = max_wealth.max(state.wealth);
        min_trust = min_trust.min(state.trust);
        max_trust = max_trust.max(state.trust);
    }

    let count = agents.len() as i64;
    let avg_trust = if count > 0 { total_trust / count } else { 0 };
    SocialMetrics {
        total_wealth,
        min_wealth,
        max_wealth,
        inequality_gap: max_wealth.saturating_sub(min_wealth),
        avg_trust,
        trust_span: max_trust.saturating_sub(min_trust),
    }
}

fn compute_state_hash(agents: &BTreeMap<u64, SocialAgentState>) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(agents.len());
    for (agent_id, state) in agents {
        parts.push(format!(
            "{}:{}:{}:{}",
            agent_id, state.wealth, state.trust, state.culture
        ));
    }
    let joined = parts.join("|");
    format!("blake3:{}", blake3::hash(joined.as_bytes()).to_hex())
}

fn deterministic_delta(seed: u64, step: u64, agent_id: u64) -> i64 {
    let mixed = mix64(seed, step, agent_id);
    (mixed % 3) as i64 - 1
}

fn mix64(seed: u64, step: u64, agent_id: u64) -> u64 {
    let mut x = seed
        ^ step.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ agent_id.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    x ^= x >> 33;
    x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    x ^= x >> 33;
    x = x.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    x ^= x >> 33;
    x
}

fn clamp_i64(value: i64, lo: i64, hi: i64) -> i64 {
    value.max(lo).min(hi)
}
