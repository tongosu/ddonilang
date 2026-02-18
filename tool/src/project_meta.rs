use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const SSOT_VERSION: &str = "20.6.33";
const SSOT_FILE_VERSION: &str = "v20.6.33";

const SSOT_BUNDLE_FILES: [&str; 12] = [
    "SSOT_MASTER",
    "SSOT_TERMS",
    "SSOT_DECISIONS",
    "SSOT_LANG",
    "SSOT_TOOLCHAIN",
    "SSOT_PLATFORM",
    "SSOT_DEMOS",
    "SSOT_PLANS",
    "SSOT_ROADMAP_CATALOG",
    "SSOT_OPEN_ISSUES",
    "SSOT_INDEX",
    "GATE0_IMPLEMENTATION_CHECKLIST",
];

#[derive(Debug, Deserialize)]
struct ProjectMeta {
    #[serde(default)]
    name: Option<String>,
    ssot_requires: String,
    #[serde(default)]
    ssot_bundle_hash: Option<String>,
    #[serde(default)]
    age_target: Option<String>,
    det_tier: String,
    #[serde(default)]
    trace_tier: Option<String>,
    openness: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DetTier {
    Strict,
    Fast,
    Ultra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Openness {
    Closed,
    Open,
}

#[derive(Debug)]
pub struct ProjectPolicy {
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub ssot_requires: String,
    #[allow(dead_code)]
    pub ssot_bundle_hash: Option<String>,
    pub age_target: AgeTarget,
    pub det_tier: DetTier,
    #[allow(dead_code)]
    pub trace_tier: Option<String>,
    pub openness: Openness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgeTarget {
    Age0,
    Age1,
    Age2,
    Age3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureGate {
    ClosedCore,
    AiTooling,
    #[allow(dead_code)]
    OpenMode,
}

impl DetTier {
    fn parse(input: &str) -> Result<Self, String> {
        match input {
            "D-STRICT" => Ok(Self::Strict),
            "D-FAST" => Ok(Self::Fast),
            "D-ULTRA" => Ok(Self::Ultra),
            _ => Err(format!("invalid det_tier: {input}")),
        }
    }
}

impl Openness {
    fn parse(input: &str) -> Result<Self, String> {
        match input {
            "closed" => Ok(Self::Closed),
            "open" => Ok(Self::Open),
            _ => Err(format!("invalid openness: {input}")),
        }
    }
}

impl AgeTarget {
    fn parse(input: &str) -> Result<Self, String> {
        match input {
            "AGE0" => Ok(Self::Age0),
            "AGE1" => Ok(Self::Age1),
            "AGE2" => Ok(Self::Age2),
            "AGE3" => Ok(Self::Age3),
            _ => Err(format!("invalid age_target: {input}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            AgeTarget::Age0 => "AGE0",
            AgeTarget::Age1 => "AGE1",
            AgeTarget::Age2 => "AGE2",
            AgeTarget::Age3 => "AGE3",
        }
    }
}

impl ProjectPolicy {
    pub fn require_gate(&self, min_det: DetTier, requires_open: bool) -> Result<(), String> {
        if self.det_tier < min_det {
            return Err(format!(
                "det_tier 위반: 프로젝트={:?}, 요구={:?}",
                self.det_tier, min_det
            ));
        }
        if requires_open && self.openness != Openness::Open {
            return Err("openness violation: open features require openness=open".to_string());
        }
        Ok(())
    }

    pub fn require_feature(&self, feature: FeatureGate) -> Result<(), String> {
        if self.age_target < feature.min_age() {
            return Err(format!(
                "E_AGE_NOT_AVAILABLE 요청 기능은 현재 AGE에서 사용할 수 없습니다: {} (need {}, current {})",
                feature.label(),
                feature.min_age().as_str(),
                self.age_target.as_str(),
            ));
        }
        self.require_gate(feature.min_det(), feature.requires_open())
    }
}

pub fn load_project_policy(unsafe_compat: bool) -> Result<ProjectPolicy, String> {
    let path = find_project_meta()?;
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read project meta: {} ({})", path.display(), e))?;
    let meta: ProjectMeta = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse project meta: {} ({})", path.display(), e))?;

    if meta.ssot_requires != SSOT_VERSION && !unsafe_compat {
        return Err(format!(
            "SSOT version mismatch: project={} toolchain={}",
            meta.ssot_requires, SSOT_VERSION
        ));
    }

    let bundle_hash = if let Some(hash) = &meta.ssot_bundle_hash {
        let actual = compute_ssot_bundle_hash()?;
        if hash != &actual && !unsafe_compat {
            return Err(format!(
                "SSOT bundle hash mismatch: project={} actual={}",
                hash, actual
            ));
        }
        Some(hash.clone())
    } else {
        None
    };

    let det_tier = DetTier::parse(&meta.det_tier)?;
    let openness = Openness::parse(&meta.openness)?;
    let age_target = meta
        .age_target
        .as_deref()
        .map(AgeTarget::parse)
        .transpose()?
        .unwrap_or(AgeTarget::Age1);

    enforce_age_policy(age_target, det_tier, openness)?;

    Ok(ProjectPolicy {
        name: meta.name,
        ssot_requires: meta.ssot_requires,
        ssot_bundle_hash: bundle_hash,
        age_target,
        det_tier,
        trace_tier: meta.trace_tier,
        openness,
    })
}

impl FeatureGate {
    pub const fn label(self) -> &'static str {
        match self {
            FeatureGate::ClosedCore => "closed_core",
            FeatureGate::AiTooling => "ai_tooling",
            FeatureGate::OpenMode => "open_mode",
        }
    }

    pub const fn min_age(self) -> AgeTarget {
        match self {
            FeatureGate::ClosedCore => AgeTarget::Age0,
            FeatureGate::AiTooling => AgeTarget::Age0,
            FeatureGate::OpenMode => AgeTarget::Age2,
        }
    }

    pub const fn min_det(self) -> DetTier {
        match self {
            FeatureGate::ClosedCore => DetTier::Strict,
            FeatureGate::AiTooling => DetTier::Strict,
            FeatureGate::OpenMode => DetTier::Strict,
        }
    }

    pub const fn requires_open(self) -> bool {
        matches!(self, FeatureGate::OpenMode)
    }
}

fn enforce_age_policy(
    age_target: AgeTarget,
    det_tier: DetTier,
    openness: Openness,
) -> Result<(), String> {
    if age_target == AgeTarget::Age0 {
        if det_tier != DetTier::Strict {
            return Err("AGE0 requires det_tier=D-STRICT".to_string());
        }
        if openness != Openness::Closed {
            return Err("AGE0 requires openness=closed".to_string());
        }
    }
    if age_target == AgeTarget::Age1 && openness == Openness::Open {
        return Err("AGE1 forbids openness=open".to_string());
    }
    Ok(())
}

fn find_project_meta() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("failed to get current directory: {e}"))?;
    let local = cwd.join("ddn.project.json");
    if local.exists() {
        return Ok(local);
    }
    let root = workspace_root()?;
    let root_meta = root.join("ddn.project.json");
    if root_meta.exists() {
        return Ok(root_meta);
    }
    Err("ddn.project.json not found".to_string())
}

fn workspace_root() -> Result<PathBuf, String> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "workspace root not found".to_string())
}

fn compute_ssot_bundle_hash() -> Result<String, String> {
    let root = workspace_root()?;
    let base = ssot_base_dir(&root);
    let mut files = Vec::new();
    for base_name in SSOT_BUNDLE_FILES.iter() {
        let name = format!("{base_name}_{SSOT_FILE_VERSION}.md");
        let path = base.join(&name);
        let bytes = fs::read(&path)
            .map_err(|e| format!("failed to read SSOT file: {} ({})", path.display(), e))?;
        if bytes.contains(&b'\r') {
            return Err(format!("SSOT file contains CRLF: {name}"));
        }
        std::str::from_utf8(&bytes)
            .map_err(|_| format!("SSOT file is not UTF-8: {name}"))?;
        files.push((name, bytes));
    }

    let mut hasher = Sha256::new();
    for (name, bytes) in files {
        hasher.update(format!("FILENAME {}\n", name).as_bytes());
        hasher.update(format!("BYTES {}\n", bytes.len()).as_bytes());
        hasher.update(&bytes);
        hasher.update(b"\n");
    }
    let digest = hasher.finalize();
    Ok(format!("sha256:{}", hex::encode(digest)))
}

fn ssot_base_dir(root: &Path) -> PathBuf {
    let preferred = root.join("docs").join("ssot").join("ssot");
    if preferred.exists() {
        return preferred;
    }
    root.join("docs").join("ssot")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(age_target: AgeTarget, det_tier: DetTier, openness: Openness) -> ProjectPolicy {
        ProjectPolicy {
            name: None,
            ssot_requires: SSOT_VERSION.to_string(),
            ssot_bundle_hash: None,
            age_target,
            det_tier,
            trace_tier: None,
            openness,
        }
    }

    #[test]
    fn require_feature_reports_age_not_available_code() {
        let project = policy(AgeTarget::Age0, DetTier::Strict, Openness::Closed);
        let err = project
            .require_feature(FeatureGate::OpenMode)
            .expect_err("age gate should fail");
        assert!(err.contains("E_AGE_NOT_AVAILABLE"));
        assert!(err.contains("need AGE2"));
        assert!(err.contains("current AGE0"));
    }

    #[test]
    fn age0_requires_strict_and_closed() {
        assert!(enforce_age_policy(AgeTarget::Age0, DetTier::Strict, Openness::Closed).is_ok());

        let det_err = enforce_age_policy(AgeTarget::Age0, DetTier::Fast, Openness::Closed)
            .expect_err("AGE0 must reject non-strict det_tier");
        assert!(det_err.contains("AGE0 requires det_tier=D-STRICT"));

        let openness_err = enforce_age_policy(AgeTarget::Age0, DetTier::Strict, Openness::Open)
            .expect_err("AGE0 must reject openness=open");
        assert!(openness_err.contains("AGE0 requires openness=closed"));
    }

    #[test]
    fn age1_forbids_open_mode() {
        let err = enforce_age_policy(AgeTarget::Age1, DetTier::Strict, Openness::Open)
            .expect_err("AGE1 should reject openness=open");
        assert!(err.contains("AGE1 forbids openness=open"));
    }
}
