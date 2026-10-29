use std::fs;
use std::path::Path;

use crate::artifact_output::write_directory_artifact_atomic;
use crate::local_project::{build_local_gaji_artifact_json, LocalGajiArtifact};

#[derive(Debug)]
pub struct GajiSealResult {
    pub package_sha256: String,
}

pub fn run_gaji_seal(input: &Path, out: &Path) -> Result<GajiSealResult, String> {
    if out.exists() {
        return Err("E_GAJI_SEAL_IMMUTABLE 대상 가지 디렉터리가 이미 존재합니다".to_string());
    }
    let request = fs::read_to_string(input)
        .map_err(|error| format!("E_GAJI_SEAL_READ 가지 추출 요청을 읽을 수 없습니다: {error}"))?;
    let artifact_json =
        build_local_gaji_artifact_json(&request).map_err(|error| error.to_string())?;
    let artifact: LocalGajiArtifact = serde_json::from_str(&artifact_json).map_err(|error| {
        format!("E_GAJI_SEAL_ARTIFACT 공통 artifact를 읽을 수 없습니다: {error}")
    })?;

    seal_gaji_artifact(out, &artifact_json, &artifact, false)?;
    println!("gaji_sealed={}", out.display());
    println!("gaji_package_sha256={}", artifact.package_sha256);
    Ok(GajiSealResult {
        package_sha256: artifact.package_sha256,
    })
}

fn seal_gaji_artifact(
    out: &Path,
    artifact_json: &str,
    artifact: &LocalGajiArtifact,
    fail_after_first_file: bool,
) -> Result<(), String> {
    write_directory_artifact_atomic(out, |staging| {
        for (index, file) in artifact.files.iter().enumerate() {
            let target = staging.join(file.path.replace('/', std::path::MAIN_SEPARATOR_STR));
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|error| format!("E_GAJI_SEAL_DIR {error}"))?;
            }
            fs::write(&target, file.utf8_text.as_bytes())
                .map_err(|error| format!("E_GAJI_SEAL_WRITE {error}"))?;
            if fail_after_first_file && index == 0 {
                return Err("E_GAJI_SEAL_INJECTED_AFTER_FIRST_FILE".to_string());
            }
        }
        fs::write(staging.join("artifact.detjson"), artifact_json.as_bytes())
            .map_err(|error| format!("E_GAJI_SEAL_META {error}"))?;
        Ok(())
    })
    .map_err(|error| {
        if error.starts_with("E_GAJI_SEAL_") {
            error
        } else {
            format!("E_GAJI_SEAL_COMMIT {error}")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);
    const REQUEST: &str =
        include_str!("../../pack/gaji_creator_wave1/g2_extraction_request.detjson");

    fn test_dir(label: &str) -> std::path::PathBuf {
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("ddn_gaji_seal_{label}_{}_{}", process::id(), nonce))
    }

    #[test]
    fn seal_materializes_complete_artifact_and_preserves_immutability() {
        let root = test_dir("success");
        fs::create_dir_all(&root).expect("create root");
        let input = root.join("request.detjson");
        let out = root.join("sealed");
        fs::write(&input, REQUEST).expect("write request");

        let result = run_gaji_seal(&input, &out).expect("seal");
        assert_eq!(result.package_sha256.len(), 64);
        assert!(result
            .package_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit()));
        assert!(out.join("artifact.detjson").is_file());
        assert!(out.join("gaji.toml").is_file());
        let before = fs::read(out.join("artifact.detjson")).expect("read artifact");

        let error = run_gaji_seal(&input, &out).expect_err("existing seal must fail");
        assert!(error.contains("E_GAJI_SEAL_IMMUTABLE"));
        assert_eq!(
            fs::read(out.join("artifact.detjson")).expect("read preserved artifact"),
            before
        );
        fs::remove_dir_all(&root).expect("remove root");
    }

    #[test]
    fn population_failure_publishes_no_partial_directory_or_residue() {
        let root = test_dir("failure");
        fs::create_dir_all(&root).expect("create root");
        let out = root.join("sealed");
        let artifact_json = build_local_gaji_artifact_json(REQUEST).expect("artifact json");
        let artifact: LocalGajiArtifact =
            serde_json::from_str(&artifact_json).expect("artifact parse");

        let error = seal_gaji_artifact(&out, &artifact_json, &artifact, true)
            .expect_err("injected failure must fail");
        assert!(error.contains("E_GAJI_SEAL_INJECTED_AFTER_FIRST_FILE"));
        assert!(!out.exists());
        assert_eq!(fs::read_dir(&root).expect("read root").count(), 0);
        fs::remove_dir_all(&root).expect("remove root");
    }
}
