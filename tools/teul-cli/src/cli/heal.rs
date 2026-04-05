use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use blake3;
use serde::{Deserialize, Serialize};

use super::detjson;
use super::paths;

const HEAL_INPUT_SCHEMA: &str = "ddn.gogae9.w97.fault_scenarios.v1";
const HEAL_REPORT_SCHEMA: &str = "ddn.gogae9.w97.heal_report.v1";

#[derive(Deserialize)]
struct HealInputDoc {
    schema: Option<String>,
    scenarios: Vec<HealScenarioInput>,
}

#[derive(Deserialize)]
struct HealScenarioInput {
    id: String,
    checkpoint_tick: u64,
    fail_tick: u64,
    replay_digest: String,
    recover_attempts: u64,
    max_retries: u64,
}

#[derive(Serialize)]
struct HealScenarioReport {
    id: String,
    recovered: bool,
    checkpoint_tick: u64,
    fail_tick: u64,
    recover_attempts: u64,
    max_retries: u64,
    final_state_hash: String,
}

#[derive(Serialize, Clone)]
struct HealSourceProvenance {
    schema: &'static str,
    source_kind: &'static str,
    pack_dir: String,
    input_file: String,
    input_hash: String,
}

#[derive(Serialize)]
struct HealReportSeed {
    schema: &'static str,
    source_hash: String,
    source_provenance: HealSourceProvenance,
    input_schema: &'static str,
    scenario_count: usize,
    overall_pass: bool,
    deterministic_replay: bool,
    rollback_restored: bool,
    cases: Vec<HealScenarioReport>,
}

#[derive(Serialize)]
struct HealReport {
    schema: &'static str,
    source_hash: String,
    source_provenance: HealSourceProvenance,
    input_schema: &'static str,
    scenario_count: usize,
    overall_pass: bool,
    deterministic_replay: bool,
    rollback_restored: bool,
    cases: Vec<HealScenarioReport>,
    heal_report_hash: String,
}

pub fn run_pack(pack_dir: &Path, out_dir: Option<&Path>) -> Result<(), String> {
    let input_path = pack_dir.join("fault_scenarios.json");
    let input_bytes = fs::read(&input_path).map_err(|e| {
        format!(
            "E_HEAL_INPUT_READ read input failed {} ({})",
            input_path.display(),
            e
        )
    })?;
    let text = String::from_utf8(input_bytes.clone())
        .map_err(|_| format!("E_HEAL_INPUT_UTF8 {}", input_path.display()))?;
    let doc: HealInputDoc =
        serde_json::from_str(&text).map_err(|e| format!("E_HEAL_INPUT_PARSE {}", e))?;

    if let Some(schema) = doc.schema.as_deref() {
        if schema.trim() != HEAL_INPUT_SCHEMA {
            return Err(format!("E_HEAL_INPUT_SCHEMA schema={}", schema.trim()));
        }
    }
    if doc.scenarios.is_empty() {
        return Err("E_HEAL_INPUT_EMPTY scenarios must be non-empty".to_string());
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut rows: Vec<HealScenarioReport> = Vec::with_capacity(doc.scenarios.len());

    for row in &doc.scenarios {
        let case_id = row.id.trim();
        if case_id.is_empty() {
            return Err("E_HEAL_CASE_ID_EMPTY id must be non-empty".to_string());
        }
        if !seen.insert(case_id.to_string()) {
            return Err(format!("E_HEAL_CASE_ID_DUP {}", case_id));
        }
        if row.checkpoint_tick == 0 || row.fail_tick <= row.checkpoint_tick {
            return Err(format!(
                "E_HEAL_NO_CHECKPOINT id={} checkpoint_tick={} fail_tick={}",
                case_id, row.checkpoint_tick, row.fail_tick
            ));
        }
        if !is_sha256_digest(&row.replay_digest) {
            return Err(format!(
                "E_HEAL_NONREPLAYABLE id={} replay_digest={}",
                case_id, row.replay_digest
            ));
        }
        if row.recover_attempts > row.max_retries {
            return Err(format!(
                "E_HEAL_LOOP id={} recover_attempts={} max_retries={}",
                case_id, row.recover_attempts, row.max_retries
            ));
        }

        let final_state_hash = compute_final_state_hash(
            case_id,
            row.checkpoint_tick,
            row.fail_tick,
            &row.replay_digest,
            row.recover_attempts,
        );
        rows.push(HealScenarioReport {
            id: case_id.to_string(),
            recovered: true,
            checkpoint_tick: row.checkpoint_tick,
            fail_tick: row.fail_tick,
            recover_attempts: row.recover_attempts,
            max_retries: row.max_retries,
            final_state_hash,
        });
    }

    let source_hash = format!("sha256:{}", detjson::sha256_hex(&input_bytes));
    let source_provenance = HealSourceProvenance {
        schema: "ddn.gogae9.w97.heal_source_provenance.v1",
        source_kind: "heal_fault_scenarios.v1",
        pack_dir: pack_dir.to_string_lossy().replace('\\', "/"),
        input_file: input_path.to_string_lossy().replace('\\', "/"),
        input_hash: source_hash.clone(),
    };
    let seed = HealReportSeed {
        schema: HEAL_REPORT_SCHEMA,
        source_hash: source_hash.clone(),
        source_provenance: source_provenance.clone(),
        input_schema: HEAL_INPUT_SCHEMA,
        scenario_count: rows.len(),
        overall_pass: true,
        deterministic_replay: true,
        rollback_restored: true,
        cases: rows,
    };
    let seed_text =
        serde_json::to_string(&seed).map_err(|e| format!("E_HEAL_REPORT_SERIALIZE {}", e))?;
    let heal_report_hash = format!("blake3:{}", blake3::hash(seed_text.as_bytes()).to_hex());
    let report = HealReport {
        schema: seed.schema,
        source_hash,
        source_provenance,
        input_schema: seed.input_schema,
        scenario_count: seed.scenario_count,
        overall_pass: seed.overall_pass,
        deterministic_replay: seed.deterministic_replay,
        rollback_restored: seed.rollback_restored,
        cases: seed.cases,
        heal_report_hash: heal_report_hash.clone(),
    };
    let report_text =
        serde_json::to_string(&report).map_err(|e| format!("E_HEAL_REPORT_SERIALIZE {}", e))?;

    let out_root = match out_dir {
        Some(path) => path.to_path_buf(),
        None => paths::build_dir().join("heal"),
    };
    fs::create_dir_all(&out_root).map_err(|e| {
        format!(
            "E_HEAL_OUT_DIR create failed {} ({})",
            out_root.display(),
            e
        )
    })?;
    let out_file = out_root.join("heal_report.detjson");
    detjson::write_text(&out_file, &report_text)
        .map_err(|e| format!("E_HEAL_OUT_WRITE {} ({})", out_file.display(), e))?;

    println!("heal_report_out={}", out_file.display());
    println!("heal_report_hash={}", heal_report_hash);
    println!("healed_cases={}", report.scenario_count);
    Ok(())
}

fn compute_final_state_hash(
    case_id: &str,
    checkpoint_tick: u64,
    fail_tick: u64,
    replay_digest: &str,
    recover_attempts: u64,
) -> String {
    let raw = format!(
        "{}|cp={}|fail={}|replay={}|attempts={}|v1",
        case_id, checkpoint_tick, fail_tick, replay_digest, recover_attempts
    );
    format!("sha256:{}", detjson::sha256_hex(raw.as_bytes()))
}

fn is_sha256_digest(value: &str) -> bool {
    let trimmed = value.trim();
    let Some(hex) = trimmed.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit())
}
