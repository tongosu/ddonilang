use serde_json::{json, Map, Value};

/// Bounded product-side primary-view decision.
///
/// This is intentionally conservative: only runtime-emitted view artifacts are
/// candidates. Project, lesson, observation-name, and output-row heuristics do
/// not participate in the decision.
pub const PRIMARY_VIEW_DECISION_SCHEMA: &str = "seamgrim.primary_view_decision.v1";

const RUNTIME_FAMILIES: &[&str] = &["space2d", "graph", "grid2d"];

pub fn decide(
    view_meta: &Value,
    source_bytes_sha256: &str,
    canonical_ddn_sha256: &str,
    state_hash: &str,
    view_hash: &str,
) -> Value {
    let mut candidates = Vec::new();
    for family in RUNTIME_FAMILIES {
        if view_meta.get(*family).is_some() {
            candidates.push((*family).to_string());
        }
    }
    // Keep explicit stack metadata useful for future runtime families while
    // still requiring an artifact key above. A stack entry alone is metadata,
    // not evidence that the runtime emitted the artifact.
    if candidates.is_empty() {
        if let Some(primary) = view_meta.pointer("/primary/family").and_then(Value::as_str) {
            if RUNTIME_FAMILIES.contains(&primary) && view_meta.get(primary).is_some() {
                candidates.push(primary.to_string());
            }
        }
    }
    candidates.sort();
    candidates.dedup();

    let (status, diagnostic_code, primary) = match candidates.as_slice() {
        [] => ("absent", Some("E_PRIMARY_VIEW_ABSENT"), Value::Null),
        [family] => (
            "selected",
            None,
            json!({
                "family": family,
                "source_ref": format!("/view_meta/{family}"),
                "runtime_artifact_kind": format!("view_meta.{family}"),
            }),
        ),
        _ => ("ambiguous", Some("E_PRIMARY_VIEW_AMBIGUOUS"), Value::Null),
    };

    let mut result = Map::new();
    result.insert("schema".to_string(), json!(PRIMARY_VIEW_DECISION_SCHEMA));
    result.insert("bounded_candidate".to_string(), json!(true));
    result.insert("status".to_string(), json!(status));
    result.insert("primary".to_string(), primary);
    result.insert("available_families".to_string(), json!(candidates));
    result.insert(
        "source_bytes_sha256".to_string(),
        json!(source_bytes_sha256),
    );
    result.insert(
        "canonical_ddn_sha256".to_string(),
        json!(canonical_ddn_sha256),
    );
    result.insert("state_hash".to_string(), json!(state_hash));
    result.insert("view_hash".to_string(), json!(view_hash));
    result.insert("state_hash_participates".to_string(), json!(false));
    if let Some(code) = diagnostic_code {
        result.insert(
            "diagnostic".to_string(),
            json!({"code": code, "severity": "error", "source": "runtime_view_meta"}),
        );
    } else {
        result.insert("diagnostic".to_string(), Value::Null);
    }
    Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::decide;
    use serde_json::json;

    fn d(meta: serde_json::Value) -> serde_json::Value {
        decide(&meta, "src", "canon", "state", "view")
    }

    #[test]
    fn selects_single_runtime_artifact() {
        let result = d(json!({"space2d": {"schema": "seamgrim.space2d.v0"}}));
        assert_eq!(result["status"], "selected");
        assert_eq!(result["primary"]["family"], "space2d");
        assert_eq!(result["primary"]["source_ref"], "/view_meta/space2d");
        assert!(result["diagnostic"].is_null());
    }

    #[test]
    fn refuses_multiple_runtime_artifacts() {
        let result = d(json!({"space2d": {}, "graph": {}}));
        assert_eq!(result["status"], "ambiguous");
        assert_eq!(result["diagnostic"]["code"], "E_PRIMARY_VIEW_AMBIGUOUS");
        assert!(result["primary"].is_null());
    }

    #[test]
    fn reports_absent_without_heuristic_fallback() {
        let result = d(json!({
            "public_observation_trace": {"schema": "ddn.public_observation_trace.v1"},
            "project_id": "projectile"
        }));
        assert_eq!(result["status"], "absent");
        assert_eq!(result["diagnostic"]["code"], "E_PRIMARY_VIEW_ABSENT");
    }

    #[test]
    fn project_identity_does_not_change_decision() {
        let a = d(json!({"space2d": {}, "project_id": "a"}));
        let b = d(json!({"space2d": {}, "project_id": "b"}));
        assert_eq!(a["status"], b["status"]);
        assert_eq!(a["primary"]["family"], b["primary"]["family"]);
    }
}
