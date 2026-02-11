use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const DTM_MAGIC: &[u8; 4] = b"DDTM";
const DTM_VERSION: u16 = 1;
const MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
struct DetMathManifest {
    version: u32,
    entries: Vec<DetMathEntry>,
}

#[derive(Debug, Deserialize)]
struct DetMathEntry {
    path: String,
    sha256: String,
}

pub fn default_manifest_path() -> PathBuf {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"));
    base.join("assets").join("detmath").join("dtm_manifest.json")
}

pub fn ensure_detmath_assets() -> Result<(), String> {
    ensure_detmath_assets_at(&default_manifest_path())
}

pub fn ensure_detmath_assets_at(manifest_path: &Path) -> Result<(), String> {
    let manifest = load_manifest(manifest_path)?;
    if manifest.entries.is_empty() {
        return Err("DetMath LUT 항목이 비어 있습니다.".to_string());
    }
    let manifest_dir = manifest_path
        .parent()
        .ok_or_else(|| "DetMath 매니페스트 경로를 확인할 수 없습니다.".to_string())?;
    for entry in &manifest.entries {
        let dtm_path = if Path::new(&entry.path).is_absolute() {
            PathBuf::from(&entry.path)
        } else {
            manifest_dir.join(&entry.path)
        };
        verify_dtm(&dtm_path, &entry.sha256)?;
    }
    Ok(())
}

fn load_manifest(path: &Path) -> Result<DetMathManifest, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("DetMath 매니페스트를 읽을 수 없습니다: {path:?} ({e})"))?;
    let manifest: DetMathManifest = serde_json::from_str(&text)
        .map_err(|e| format!("DetMath 매니페스트 파싱 실패: {e}"))?;
    if manifest.version != MANIFEST_VERSION {
        return Err(format!(
            "DetMath 매니페스트 버전 불일치: {} (expected {})",
            manifest.version, MANIFEST_VERSION
        ));
    }
    Ok(manifest)
}

fn verify_dtm(path: &Path, expected_sha256: &str) -> Result<(), String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("DetMath LUT 파일을 읽을 수 없습니다: {path:?} ({e})"))?;
    verify_dtm_header(path, &bytes)?;
    let actual = sha256_hex(&bytes);
    let expected = expected_sha256.trim().to_ascii_lowercase();
    if actual != expected {
        return Err(format!(
            "DetMath LUT 해시 불일치: {path:?} (expected {expected}, actual {actual})"
        ));
    }
    Ok(())
}

fn verify_dtm_header(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 16 {
        return Err(format!("DetMath LUT 헤더 길이 부족: {path:?}"));
    }
    if &bytes[0..4] != DTM_MAGIC {
        return Err(format!("DetMath LUT 매직 불일치: {path:?}"));
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != DTM_VERSION {
        return Err(format!(
            "DetMath LUT 버전 불일치: {path:?} (expected {DTM_VERSION}, actual {version})"
        ));
    }
    let entry_count = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let frac_bits = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    if frac_bits != 32 {
        return Err(format!(
            "DetMath LUT frac_bits 불일치: {path:?} (expected 32, actual {frac_bits})"
        ));
    }
    let expected_len = 16usize + (entry_count as usize) * 8;
    if bytes.len() != expected_len {
        return Err(format!(
            "DetMath LUT 길이 불일치: {path:?} (expected {expected_len}, actual {})",
            bytes.len()
        ));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detmath_manifest_missing_entry_reports_error() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("ddn_detmath_missing_{nanos}"));
        fs::create_dir_all(&base).expect("mkdir");
        let manifest_path = base.join("dtm_manifest.json");
        let manifest = r#"{"version":1,"entries":[{"path":"missing.dtm","sha256":"00"}]}"#;
        fs::write(&manifest_path, manifest).expect("write manifest");

        let err = ensure_detmath_assets_at(&manifest_path).unwrap_err();
        assert!(err.contains("DetMath LUT"));

        let _ = fs::remove_dir_all(&base);
    }
}
