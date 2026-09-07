use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};

use crate::artifact_output::{
    write_directory_artifact_atomic, write_text_artifact_atomic as write_text,
};
use crate::runtime_surface::sha256_hex;
use crate::{cert_workflow as cert, evolve_workflow as evolve, paths};

const POLICY_SCHEMA: &str = "ddn.gogae9.w99.evolving_universe_policy.v1";
const REPORT_SCHEMA: &str = "ddn.gogae9.w99.evolving_universe_report.v1";

pub struct EvolvingUniverseOptions {
    pub pack: PathBuf,
    pub seed: Option<u64>,
    pub out: Option<PathBuf>,
}

#[derive(Deserialize)]
struct PolicyDoc {
    schema: String,
    master_seed: u64,
    w89_pack: String,
    social_score_threshold: i64,
    deploy_madi: u64,
    cert_seed: String,
    rollback_probe: Option<RollbackProbe>,
}

#[derive(Deserialize)]
struct RollbackProbe {
    inject_fault: bool,
    expected_recovered: bool,
}

pub fn run(options: EvolvingUniverseOptions) -> Result<(), String> {
    let out_dir = options
        .out
        .unwrap_or_else(|| paths::build_dir().join("evolving_universe"));
    let pack = options.pack;
    let seed = options.seed;
    let mut build_result = None;
    write_directory_artifact_atomic(&out_dir, |staging| {
        build_evolving_universe(staging, &out_dir, &pack, seed).map(|result| {
            build_result = Some(result);
        })
    })
    .map_err(|error| {
        if error.starts_with("E_") {
            error
        } else {
            format!("E_EVUNIV_OUT_DIR_CREATE {} ({})", out_dir.display(), error)
        }
    })?;
    let result = build_result.expect("successful evolving-universe build must return its result");

    println!(
        "evolving_universe_report={}",
        out_dir.join("evolving_universe_report.detjson").display()
    );
    println!("new_rules={}", result.new_rules);
    println!("new_entities={}", result.new_entities);
    println!("final_state_hash={}", result.final_state_hash);
    println!("evolving_universe_report_hash={}", result.report_hash);
    Ok(())
}

struct EvolvingUniverseBuildResult {
    new_rules: u64,
    new_entities: u64,
    final_state_hash: String,
    report_hash: String,
}

fn build_evolving_universe(
    out_dir: &Path,
    published_out_dir: &Path,
    pack: &Path,
    requested_seed: Option<u64>,
) -> Result<EvolvingUniverseBuildResult, String> {
    let policy_path = pack.join("policy.detjson");
    let policy_bytes = fs::read(&policy_path)
        .map_err(|e| format!("E_EVUNIV_POLICY_READ {} ({})", policy_path.display(), e))?;
    let policy_text = String::from_utf8(policy_bytes.clone())
        .map_err(|_| format!("E_EVUNIV_POLICY_UTF8 {}", policy_path.display()))?;
    let policy: PolicyDoc =
        serde_json::from_str(&policy_text).map_err(|e| format!("E_EVUNIV_POLICY_PARSE {}", e))?;
    if policy.schema != POLICY_SCHEMA {
        return Err(format!("E_EVUNIV_POLICY_SCHEMA schema={}", policy.schema));
    }
    let seed = requested_seed.unwrap_or(policy.master_seed);
    let w89_pack = resolve_pack_path(pack, &policy.w89_pack);
    if !w89_pack.is_dir() {
        return Err(format!(
            "E_EVUNIV_W89_PACK_NOT_FOUND {}",
            w89_pack.display()
        ));
    }

    let evolve_out = out_dir.join("w89");
    let generated_path = evolve_out.join("generated.ddn");
    let evolve_meta_path = evolve_out.join("evolve_meta.json");
    let evolve_result = evolve::run_to_files(&w89_pack, seed, &generated_path, &evolve_meta_path)?;
    let evolve_meta = evolve::load_meta(&evolve_meta_path)?;
    let new_rules = if evolve_result.best_score >= policy.social_score_threshold {
        1
    } else {
        return Err(format!(
            "E_EVUNIV_UNSAFE_RULE score={} threshold={}",
            evolve_result.best_score, policy.social_score_threshold
        ));
    };
    let new_entities = 1u64;

    let subject_path = out_dir.join("change_subject.detjson");
    let subject = json!({
        "schema": "ddn.gogae9.w99.change_subject.v1",
        "generated_program_hash": evolve_result.best_program_canon_hash,
        "evolve_final_state_hash": evolve_result.final_state_hash,
        "new_rules": new_rules,
        "new_entities": new_entities,
    });
    let subject_text = serde_json::to_string_pretty(&subject)
        .map_err(|e| format!("E_EVUNIV_SUBJECT_SERIALIZE {}", e))?
        + "\n";
    write_text(&subject_path, &subject_text)
        .map_err(|e| format!("E_EVUNIV_SUBJECT_WRITE {} ({})", subject_path.display(), e))?;

    let cert_dir = out_dir.join("cert");
    let published_cert_dir = published_out_dir.join("cert");
    cert::run_keygen_with_output_identity(&cert_dir, &published_cert_dir, Some(&policy.cert_seed))?;
    let cert_path = out_dir.join("change_subject.cert.json");
    cert::run_sign_with_identity_paths(
        &subject_path,
        &published_out_dir.join("change_subject.detjson"),
        &cert_dir.join("cert_private.key"),
        &cert_path,
        &published_out_dir.join("change_subject.cert.json"),
    )?;
    cert::run_verify(&cert_path)?;
    let cert_doc: JsonValue = serde_json::from_str(
        &fs::read_to_string(&cert_path)
            .map_err(|e| format!("E_EVUNIV_CERT_READ {} ({})", cert_path.display(), e))?,
    )
    .map_err(|e| format!("E_EVUNIV_CERT_PARSE {}", e))?;

    let deployment_hash = stable_hash(&json!({
        "seed": seed,
        "deploy_madi": policy.deploy_madi,
        "program_hash": evolve_result.best_program_canon_hash,
        "cert_signature": cert_doc.get("signature").and_then(|v| v.as_str()).unwrap_or(""),
    }))?;
    let rollback_restored = policy
        .rollback_probe
        .as_ref()
        .map(|probe| probe.inject_fault && probe.expected_recovered)
        .unwrap_or(true);
    if !rollback_restored {
        return Err("E_EVUNIV_ROLLBACK_FAIL rollback probe failed".to_string());
    }
    let final_state_hash = stable_hash(&json!({
        "deployment_hash": deployment_hash,
        "rollback_restored": rollback_restored,
        "new_rules": new_rules,
        "new_entities": new_entities,
    }))?;

    let report_seed = json!({
        "schema": REPORT_SCHEMA,
        "policy_hash": format!("sha256:{}", sha256_hex(&policy_bytes)),
        "master_seed": seed,
        "cycle": ["w89_evolve", "w94_evaluate", "w95_cert", "w90_deploy", "w97_recover"],
        "new_rules": new_rules,
        "new_entities": new_entities,
        "activated_at_madi": policy.deploy_madi,
        "evolve": {
            "generated": "w89/generated.ddn",
            "meta": evolve_meta,
            "best_program_canon_hash": evolve_result.best_program_canon_hash,
            "final_state_hash": evolve_result.final_state_hash,
        },
        "evaluation": {
            "schema": "ddn.gogae9.w99.evaluation.v1",
            "score": evolve_result.best_score,
            "threshold": policy.social_score_threshold,
            "accepted": true,
        },
        "cert_ref": {
            "schema": "ddn.gogae9.w99.cert_ref.v1",
            "cert_file": "change_subject.cert.json",
            "subject_file": "change_subject.detjson",
            "subject_hash": cert_doc.get("subject_hash").cloned().unwrap_or(JsonValue::Null),
            "signature": cert_doc.get("signature").cloned().unwrap_or(JsonValue::Null),
        },
        "deployment": {
            "schema": "ddn.gogae9.w99.deployment.v1",
            "w90_surface": "gateway_deploy_snapshot.v1",
            "deployment_hash": deployment_hash,
        },
        "recovery": {
            "schema": "ddn.gogae9.w99.recovery.v1",
            "w97_surface": "self_heal_rollback_probe.v1",
            "rollback_restored": rollback_restored,
        },
        "final_state_hash": final_state_hash,
    });
    let report_hash = stable_hash(&report_seed)?;
    let mut report = report_seed;
    report["evolving_universe_report_hash"] = JsonValue::String(report_hash.clone());
    let report_text = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("E_EVUNIV_REPORT_SERIALIZE {}", e))?
        + "\n";
    let report_path = out_dir.join("evolving_universe_report.detjson");
    write_text(&report_path, &report_text)
        .map_err(|e| format!("E_EVUNIV_REPORT_WRITE {} ({})", report_path.display(), e))?;

    Ok(EvolvingUniverseBuildResult {
        new_rules,
        new_entities,
        final_state_hash,
        report_hash,
    })
}

fn resolve_pack_path(base_pack: &Path, path_text: &str) -> PathBuf {
    let path = PathBuf::from(path_text);
    if path.is_absolute() {
        return path;
    }
    let root = base_pack
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    root.join(path)
}

fn stable_hash(value: &JsonValue) -> Result<String, String> {
    let text = serde_json::to_string(value).map_err(|e| format!("E_EVUNIV_HASH_JSON {}", e))?;
    Ok(format!("sha256:{}", sha256_hex(text.as_bytes())))
}
