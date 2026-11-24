//! Read-only language evidence from a fully restored source revision.
//! Counts describe verified scoped evidence, never a whole-project translation promise.
use super::StrictValue;
use crate::ddn_runtime::{resolve_supported_source_profile, SupportedSourceProfile};
use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as Source;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    source_artifact_utf8: String,
    project_hex: String,
    revision_hex: String,
}

pub(super) fn inspect(raw: &str) -> Result<String, String> {
    if raw.len() > 16_000_000 {
        return Err("E_LANGUAGE_SUPPORT_REQUEST_LIMIT".into());
    }
    let strict: StrictValue = serde_json::from_str(raw)
        .map_err(|_| "E_LANGUAGE_SUPPORT_REQUEST_FIELDS")?;
    let request: Request = serde_json::from_value(strict.0)
        .map_err(|_| "E_LANGUAGE_SUPPORT_REQUEST_FIELDS")?;
    // Existing reader rechecks identity, exact registry versions, activation,
    // captures and the revision seal. It issues no identities and never runs.
    let source = restore(&request.source_artifact_utf8)?;
    if request.project_hex != hex::encode(source.project())
        || request.revision_hex != hex::encode(source.revision())
    {
        return Err("E_LANGUAGE_SUPPORT_SELECTION_CHANGED".into());
    }
    let owned = source.owned();
    let project = owned.project();
    let profile = resolve_supported_source_profile(Some(&project.source_profile))
        .map_err(|error| error.message)?;
    let name = match profile {
        SupportedSourceProfile::V1CoreV25 => "한국어",
        SupportedSourceProfile::V1CoreV25JaBaseR1 => "日本語",
    };
    let mut evidence = vec![json!({
        "canonical_name": name,
        "scope": "source_syntax",
        "source_profile": project.source_profile,
    })];
    let sources = owned.sources();
    if let Some(artifact) = sources.public_callable_artifact() {
        let context = sources.public_callable_context()
            .ok_or("E_LANGUAGE_SUPPORT_CALLABLE_CONTEXT")?;
        let binding = artifact.dependency_material()?;
        evidence.push(callable_evidence(context.compiler.profile(), &binding));
    }
    if let Some(context) = sources.first_callable_context() {
        if let Some(binding) = &project.language_binding {
            let profile = context.public_export.as_ref()
                .ok_or("E_LANGUAGE_SUPPORT_FIRST_CONTEXT")?.profile();
            evidence.push(json!({
                "canonical_name": profile.canonical_name(),
                "scope": "module_public_callables",
                "profile_identity": profile.identity(),
                "profile_version": profile.version(),
                "registration_revision": profile.registration_revision(),
                "profile_content_sha256": profile.content_sha256(),
                "target_owner_kind": "module",
                "target_owner_identity": binding.original_module_id,
                "consumer_module_id": binding.consumer_module_id,
                "consumer_source_path": binding.consumer_source_path,
            }));
        }
    }
    if let Some(context) = sources.logic_callable_context() {
        let profile = context.profile();
        let path = &project.source_snapshot.entry_path;
        let owners = project.authored_identity.modules.iter()
            .filter(|module| module.sources.contains(path)).collect::<Vec<_>>();
        if owners.len() != 1 || context.activation().is_none() {
            return Err("E_LANGUAGE_SUPPORT_LOGIC_OWNER".into());
        }
        evidence.push(json!({
            "canonical_name": profile.canonical_name(),
            "scope": "module_public_callables",
            "profile_identity": profile.identity(),
            "profile_version": profile.version(),
            "registration_revision": profile.registration_revision(),
            "profile_content_sha256": profile.content_sha256(),
            "target_owner_kind": "builtin",
            "target_owner_identity": hex::encode(context.builtin_owner()),
            "consumer_module_id": owners[0].id.as_str(),
            "consumer_source_path": path,
        }));
    }
    let names = evidence.iter().filter_map(|row| row["canonical_name"].as_str())
        .collect::<BTreeSet<_>>();
    serde_json::to_string(&json!({
        "project_hex": request.project_hex,
        "revision_hex": request.revision_hex,
        "source_profile": project.source_profile,
        "project_content_sha256": project.content_sha256,
        "source_artifact_sha256": hex::encode(Sha256::digest(request.source_artifact_utf8.as_bytes())),
        "verified_languages": names,
        "verified_language_count": names.len(),
        "complete": false,
        "total_language_count": Value::Null,
        "evidence": evidence,
    })).map_err(|_| "E_LANGUAGE_SUPPORT_ENCODING".into())
}

fn restore(artifact: &str) -> Result<Source, String> {
    let selection = json!({"source_artifact_utf8": artifact}).to_string();
    if super::source_save::logic_consumer::public_run_is_selected(&selection)? {
        Source::from_artifact_logic_private(artifact)
    } else {
        Source::from_artifact(artifact)
    }
}

fn callable_evidence(profile: &ddonirang_lang::ddl::RegisteredSuccessorProfile,
    binding: &Value) -> Value {
    json!({
        "canonical_name": profile.canonical_name(),
        "scope": "module_public_callables",
        "profile_identity": profile.identity(),
        "profile_version": profile.version(),
        "registration_revision": profile.registration_revision(),
        "profile_content_sha256": profile.content_sha256(),
        "target_owner_kind": "module",
        "target_owner_identity": binding["original_module_id"],
        "consumer_module_id": binding["consumer_module_id"],
        "consumer_source_path": binding["consumer_source_path"],
    })
}

#[cfg(test)]
mod project_language_support_tests {
    use super::*;
    const B1: &str = include_str!("../../tests/fixtures/multilang_language_support_b1.json");
    const V4: &str = include_str!("../../tests/fixtures/multilang_language_support_v4.json");

    fn request(artifact: &str) -> Value {
        let source = restore(artifact).unwrap();
        json!({"source_artifact_utf8": artifact,
            "project_hex": hex::encode(source.project()),
            "revision_hex": hex::encode(source.revision())})
    }

    #[test]
    fn project_language_support_reuses_exact_v4_and_b1_owners_without_execution() {
        for (artifact, expected) in [(V4, 2), (B1, 1)] {
            let input = request(artifact).to_string();
            let result = super::super::operate("source_language_support", &input).unwrap();
            println!("LANGUAGE_SUPPORT_REAL {}", result);
            assert_eq!(inspect(&input).unwrap(), result);
            let row: Value = serde_json::from_str(&result).unwrap();
            assert_eq!(row["verified_language_count"], expected);
            assert_eq!(row["complete"], false);
            assert!(row["total_language_count"].is_null());
            assert_eq!(row["evidence"].as_array().unwrap().len(), 2);
            assert!(row.get("output_log_texts").is_none());
            let callable = &row["evidence"][1];
            assert_eq!(callable["canonical_name"], "日本語");
            assert!(callable["profile_identity"].as_str().unwrap().starts_with("language-profile:"));
            assert_eq!(callable["consumer_module_id"].as_str().unwrap().len(), 64);
        }
    }

    #[test]
    fn project_language_support_rejects_stale_revision_foreign_project_and_unknown_fields() {
        for field in ["project_hex", "revision_hex"] {
            let mut input = request(V4);
            input[field] = json!("00".repeat(32));
            assert_eq!(inspect(&input.to_string()).unwrap_err(), "E_LANGUAGE_SUPPORT_SELECTION_CHANGED");
        }
        let mut input = request(B1);
        input["trusted"] = json!(true);
        assert_eq!(inspect(&input.to_string()).unwrap_err(), "E_LANGUAGE_SUPPORT_REQUEST_FIELDS");
        assert!(inspect("{\"project_hex\":\"a\",\"project_hex\":\"b\"}").is_err());
    }

    #[test]
    fn project_language_support_plain_korean_has_no_invented_ddl_language() {
        use super::super::{ProjectArtifact, SourceFile, SourceSnapshot};
        let snapshot = SourceSnapshot {
            entry_path: "main.ddn".into(), root_path: Some("root.ddn".into()),
            files: vec![SourceFile {path: "root.ddn".into(), text:
                "(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n".into()},
                SourceFile {path: "main.ddn".into(), text: "실행:움직씨 := { 0 보여주기. }.\n".into()}],
        };
        let project = ProjectArtifact::create("v1-core-v25".into(), snapshot, String::new()).unwrap();
        let owned = crate::project_revision_envelope_supply::admit_fresh_authored_project(
            project.save().unwrap().as_bytes()).unwrap();
        let artifact = Source::prepare(&owned, &[], &[1]).unwrap().artifact().unwrap();
        let row: Value = serde_json::from_str(&inspect(&request(&artifact).to_string()).unwrap()).unwrap();
        assert_eq!(row["verified_languages"], json!(["한국어"]));
        assert_eq!(row["evidence"].as_array().unwrap().len(), 1);
        assert!(row["total_language_count"].is_null());
    }

    #[test]
    fn project_language_support_rejects_modified_registered_artifact() {
        let mut input = request(V4);
        let modified = V4.replacen("language-profile:", "language-profile:00", 1);
        assert_ne!(modified, V4);
        input["source_artifact_utf8"] = json!(modified);
        assert!(inspect(&input.to_string()).is_err());
    }
}
