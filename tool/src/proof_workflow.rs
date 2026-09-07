use std::fs;
use std::path::Path;

use crate::artifact_output::write_text_artifact_atomic;

pub fn run_verify(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let report = ddonirang_proof::verify_json_text(&text)?;
    write_report(&report, out)?;
    println!("proof_valid={} kind={}", report.valid, report.kind);
    Ok(())
}

pub fn run_replay(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let report = ddonirang_proof::verify_json_text(&text)?;
    write_report(&report, out)?;
    println!("proof_replay_valid={} kind={}", report.valid, report.kind);
    Ok(())
}

pub fn run_prove_symbolic_eq(lhs: &str, rhs: &str, out: Option<&Path>) -> Result<(), String> {
    let cert = ddonirang_symbolic::prove_equivalent(lhs, rhs)?;
    if !cert.equivalent {
        return Err("E_PROOF_EQUIVALENCE_NOT_PROVEN_INTERNAL".to_string());
    }
    let value = serde_json::to_value(&cert).map_err(|e| e.to_string())?;
    let report = ddonirang_proof::verify_value(&value)?;
    if !report.valid {
        return Err("E_PROOF_CHECK_FAILED_INTERNAL".to_string());
    }
    if let Some(path) = out {
        write_text_artifact_atomic(path, &ddonirang_symbolic::to_detjson(&cert)?)?;
    }
    println!(
        "proof_tactic_ok=true kind={} equivalent=true",
        report.kind
    );
    Ok(())
}

pub fn run_rewrite_chain(from: &str, to: &str, out: Option<&Path>) -> Result<(), String> {
    let proof = serde_json::json!({
        "schema": "ddn.proof.symbolic_rewrite.v1",
        "steps": [{"from": from, "to": to}]
    });
    let report = ddonirang_proof::verify_value(&proof)?;
    if !report.valid {
        return Err("E_PROOF_REWRITE_NOT_PROVEN_INTERNAL".to_string());
    }
    if let Some(path) = out {
        write_text_artifact_atomic(
            path,
            &serde_json::to_string_pretty(&proof).map_err(|e| e.to_string())?,
        )?;
    }
    println!("proof_rewrite_ok=true kind={}", report.kind);
    Ok(())
}

pub fn run_prove_relation_eq(
    first_lhs: &str,
    first_rhs: &str,
    second_lhs: &str,
    second_rhs: &str,
    out: Option<&Path>,
) -> Result<(), String> {
    let cert = ddonirang_symbolic::prove_relation_equivalent(
        first_lhs, first_rhs, second_lhs, second_rhs,
    )?;
    if !cert.equivalent {
        return Err("E_PROOF_RELATION_EQUIVALENCE_NOT_PROVEN_INTERNAL".to_string());
    }
    let value = serde_json::to_value(&cert).map_err(|e| e.to_string())?;
    let report = ddonirang_proof::verify_value(&value)?;
    if !report.valid {
        return Err("E_PROOF_CHECK_FAILED_INTERNAL".to_string());
    }
    if let Some(path) = out {
        write_text_artifact_atomic(path, &ddonirang_symbolic::to_detjson(&cert)?)?;
    }
    println!(
        "proof_relation_ok=true kind={} equivalent=true",
        report.kind
    );
    Ok(())
}

fn write_report(report: &ddonirang_proof::VerifyReport, out: Option<&Path>) -> Result<(), String> {
    if let Some(path) = out {
        write_text_artifact_atomic(path, &ddonirang_proof::to_detjson(report)?)?;
    }
    Ok(())
}
