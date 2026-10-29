//! Read-only projection for the Workbench. It supplies no execution, package
//! verification or storage authority; those consumers still admit the raw tree.
use crate::local_project::{VirtualProjectFileSource, VirtualProjectSource, VIRTUAL_PROJECT_SOURCE_SCHEMA};

pub(super) fn project(text: &str) -> Result<String, String> {
    let artifact = super::open(text.as_bytes())?;
    let snapshot = &artifact.source_snapshot;
    let roles = crate::project_revision_envelope_supply::saved_authored_source_roles(snapshot)?;
    let root_path = snapshot.root_path.as_ref()
        .ok_or("E_PROJECT_RUNTIME_ROOT_REQUIRED: 명시 얽음 뿌리가 필요합니다")?;
    let files = snapshot.files.iter().map(|file| {
        let role = roles.get(&file.path)
            .ok_or("E_PROJECT_RUNTIME_SOURCE_SUPPLY: 원자료 역할 공급이 없습니다")?;
        Ok(VirtualProjectFileSource {
            path: file.path.clone(), role: role.clone(), utf8_text: file.text.clone(),
        })
    }).collect::<Result<Vec<_>, String>>()?;
    let source = VirtualProjectSource {
        schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.into(),
        kind: Some("seamgrim.local_project_tree.v0".into()),
        package_hash: None,
        root_path: root_path.clone(), entry_path: snapshot.entry_path.clone(), files,
    };
    serde_json::to_string(&source)
        .map_err(|_| "E_PROJECT_RUNTIME_SOURCE_SUPPLY: 원자료 투영을 만들 수 없습니다".into())
}

#[cfg(all(test, feature = "v2-shared-modules"))]
mod tests {
    use super::*;
    const FIXTURE: &str = include_str!("../../tests/fixtures/v2_helper_source.json");

    #[test]
    fn preserves_all_raw_files_and_explicit_metadata_roles() {
        let original = crate::authored_project::open(FIXTURE.as_bytes()).unwrap();
        let source: VirtualProjectSource = serde_json::from_str(&project(FIXTURE).unwrap()).unwrap();
        assert_eq!(source.files.len(), 6);
        assert_eq!(source.root_path, "뿌리.ddn");
        assert_eq!(source.entry_path, "app/main.ddn");
        for (actual, raw) in source.files.iter().zip(&original.source_snapshot.files) {
            assert_eq!(actual.path, raw.path);
            assert_eq!(actual.utf8_text, raw.text);
        }
        let roles: std::collections::BTreeMap<_, _> = source.files.iter()
            .map(|file| (file.path.as_str(), file.role.as_str())).collect();
        assert_eq!(roles["뿌리.ddn"], "root");
        assert_eq!(roles["app/main.ddn"], "ddn_source");
        assert_eq!(roles["vendor/운동/ddn/lib/운동.ddn"], "ddn_source");
        assert_eq!(roles["vendor/운동/artifact.detjson"], "metadata");
        assert_eq!(roles["vendor/운동/gaji.toml"], "gaji_manifest");
        assert_eq!(roles["ddn.lock"], "lock");
    }

    #[test]
    fn shared_operation_is_deterministic_and_keeps_persistent_ids() {
        let before = crate::authored_project::open(FIXTURE.as_bytes()).unwrap().save().unwrap();
        let first = crate::authored_project::operate("source_project", FIXTURE).unwrap();
        let second = crate::authored_project::operate("source_project", FIXTURE).unwrap();
        assert_eq!(first, second);
        assert_eq!(crate::authored_project::open(FIXTURE.as_bytes()).unwrap().save().unwrap(), before);
        let expected: VirtualProjectSource = serde_json::from_str(&first).unwrap();
        assert!(expected.package_hash.is_none());
    }

    #[test]
    fn rejects_tampered_artifact_instead_of_blessing_its_checksum() {
        let mut value: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
        value["source_snapshot"]["files"][0]["text"] = serde_json::json!("값 := 999.");
        assert!(crate::authored_project::operate("source_project", &value.to_string()).is_err());
    }

    #[test]
    fn requires_explicit_root_without_changing_legacy_single_file_open() {
        let artifact = crate::authored_project::ProjectArtifact::create(
            "v1-core-v25".into(), crate::authored_project::SourceSnapshot {
                root_path: None, entry_path: "gaji.toml".into(),
                files: vec![crate::authored_project::SourceFile {path:"gaji.toml".into(),text:"값 := 1.".into()}],
            }, "앱".into()).unwrap().save().unwrap();
        assert!(crate::authored_project::operate("open", &artifact).is_ok());
        assert!(crate::authored_project::operate("source_project", &artifact).unwrap_err()
            .contains("E_PROJECT_RUNTIME_ROOT_REQUIRED"));
    }

    #[test]
    fn projection_never_supplies_lock_verification_authority() {
        let projected = project(FIXTURE).unwrap();
        let source: VirtualProjectSource = serde_json::from_str(&projected).unwrap();
        crate::local_project::build_project_prepare_link(source, "", true, true).unwrap();

        let mut artifact = crate::authored_project::open(FIXTURE.as_bytes()).unwrap();
        artifact.source_snapshot.files.iter_mut().find(|file|file.path=="ddn.lock")
            .unwrap().text.push_str("\n{}");
        let candidate = artifact.save().unwrap();
        let projected = project(&candidate).unwrap();
        let source: VirtualProjectSource = serde_json::from_str(&projected).unwrap();
        assert!(source.files.iter().find(|file|file.path=="ddn.lock").unwrap().utf8_text.ends_with("\n{}"));
        let error = crate::local_project::build_project_prepare_link(source,"",true,true).unwrap_err();
        assert_eq!(error.code,"E_LOCK_HASH_MISMATCH");
    }
}
