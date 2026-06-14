use serde_json::json;

use crate::cli::detjson::sha256_hex;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DultraReplayArtifactSeed {
    pub solver_id: String,
    pub solver_version: String,
    pub backend: String,
    pub build_fingerprint: String,
    pub configuration_hash: String,
    pub initial_state_hash: String,
    pub input_sequence_hash: String,
    pub model_source_hash: String,
    pub input_snapshot_hash: String,
    pub time_horizon: String,
}

#[allow(dead_code)]
pub fn build_dultra_replay_seed_artifact(seed: &DultraReplayArtifactSeed) -> Result<String, String> {
    if seed.solver_id.trim().is_empty() {
        return Err("E_DULTRA_REPLAY_SEED solver_id is required".to_string());
    }
    if seed.configuration_hash.trim().is_empty() {
        return Err("E_DULTRA_REPLAY_SEED configuration_hash is required".to_string());
    }

    let solver_identity = json!({
        "solver_id": seed.solver_id.as_str(),
        "solver_version": seed.solver_version.as_str(),
        "backend": seed.backend.as_str(),
        "build_fingerprint": seed.build_fingerprint.as_str(),
        "configuration_hash": seed.configuration_hash.as_str()
    });
    let initial_context = json!({
        "initial_state_hash": seed.initial_state_hash.as_str(),
        "input_sequence_hash": seed.input_sequence_hash.as_str(),
        "model_source_hash": seed.model_source_hash.as_str(),
        "time_horizon": seed.time_horizon.as_str()
    });
    let input_sequence = json!({
        "ordered_events": [],
        "deterministic_indexes": true,
        "input_snapshot_hash": seed.input_snapshot_hash.as_str()
    });
    let step_trace = json!({
        "adaptive_step_choices": [],
        "solver_trace_summary": "seed_artifact_no_runtime_trace",
        "accepted_step_count": 0,
        "rejected_step_count": 0
    });
    let normalization_metadata = json!({
        "external_result_normalization": "not_runtime_landed",
        "unit_policy": "not_runtime_landed",
        "precision_policy": "not_runtime_landed",
        "serialization_policy": "detjson_seed"
    });
    let failure_diag = json!({
        "failure_state": "not_run",
        "diag_code": "DULTRA_REPLAY_ARTIFACT_SEED_ONLY",
        "warnings": [],
        "replay_confidence": "seed_artifact_only"
    });
    let claim_boundary = json!({
        "dultra_replay_scope": "artifact_writer_seed_only",
        "dstrict_truth_claim": false,
        "current_line_support_claim": false,
        "runtime_recorded_replay_implementation_landed": false
    });

    let section_bundle = json!({
        "solver_identity": solver_identity,
        "initial_context": initial_context,
        "input_sequence": input_sequence,
        "step_trace": step_trace,
        "normalization_metadata": normalization_metadata,
        "failure_diag": failure_diag,
        "claim_boundary": claim_boundary
    });
    let section_hash = format!("sha256:{}", sha256_hex(section_bundle.to_string().as_bytes()));
    let artifact = json!({
        "schema": "ddn.dultra_replay.detjson.v1",
        "kind": "dultra_replay_seed_artifact",
        "writer": {
            "id": "teul_cli.dultra_replay.seed_writer",
            "runtime_landed": false,
            "verifier_landed": false
        },
        "section_hash": section_hash,
        "sections": section_bundle
    });
    serde_json::to_string_pretty(&artifact)
        .map(|text| format!("{text}\n"))
        .map_err(|err| format!("E_DULTRA_REPLAY_SEED_SERIALIZE {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value as JsonValue;

    fn sample_seed() -> DultraReplayArtifactSeed {
        DultraReplayArtifactSeed {
            solver_id: "dultra.preview.external_ode_stub".to_string(),
            solver_version: "0.0-seed".to_string(),
            backend: "external_stub".to_string(),
            build_fingerprint: "not-runtime-landed".to_string(),
            configuration_hash: "sha256:config".to_string(),
            initial_state_hash: "sha256:initial".to_string(),
            input_sequence_hash: "sha256:input-sequence".to_string(),
            model_source_hash: "sha256:model-source".to_string(),
            input_snapshot_hash: "sha256:input-snapshot".to_string(),
            time_horizon: "seed-only".to_string(),
        }
    }

    #[test]
    fn dultra_replay_seed_artifact_has_required_sections_and_false_claims() {
        let text = build_dultra_replay_seed_artifact(&sample_seed()).expect("artifact");
        let doc: JsonValue = serde_json::from_str(&text).expect("json");
        assert_eq!(doc["schema"], "ddn.dultra_replay.detjson.v1");
        assert_eq!(doc["kind"], "dultra_replay_seed_artifact");
        assert_eq!(doc["writer"]["runtime_landed"], false);
        assert_eq!(doc["writer"]["verifier_landed"], false);
        let sections = doc["sections"].as_object().expect("sections");
        for key in [
            "solver_identity",
            "initial_context",
            "input_sequence",
            "step_trace",
            "normalization_metadata",
            "failure_diag",
            "claim_boundary",
        ] {
            assert!(sections.contains_key(key), "missing section {key}");
        }
        assert_eq!(doc["sections"]["claim_boundary"]["dstrict_truth_claim"], false);
        assert_eq!(
            doc["sections"]["claim_boundary"]["runtime_recorded_replay_implementation_landed"],
            false
        );
    }
}
