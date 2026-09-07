use std::fs;
use std::path::Path;

use serde::Serialize;

const SKIP_WARNING_CODE: &str = "W_SKIP_NON_GAJI_DIR";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GajiScanResult {
    pub gaji_packages: usize,
    pub warnings: Vec<GajiScanWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GajiScanWarning {
    pub relative_path: String,
}

#[derive(Serialize)]
struct ScanWarningPayload<'a> {
    kind: &'static str,
    level: &'static str,
    code: &'static str,
    message: &'a str,
}

pub fn scan_gaji_packages(root: &Path) -> Result<GajiScanResult, String> {
    let gaji_root = root.join("gaji");
    let mut directories = Vec::new();

    if gaji_root.exists() {
        let entries = fs::read_dir(&gaji_root).map_err(|err| format!("E_SCAN_READ {err}"))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("E_SCAN_READ {err}"))?;
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
            }
        }
    }

    directories.sort_by(|left, right| directory_name(left).cmp(directory_name(right)));

    let mut gaji_packages = 0usize;
    let mut warnings = Vec::new();
    for path in directories {
        let name = directory_name(&path);
        if name.starts_with('.') {
            continue;
        }
        if path.join("gaji.toml").exists() {
            gaji_packages += 1;
        } else {
            warnings.push(GajiScanWarning {
                relative_path: format!("gaji/{name}"),
            });
        }
    }

    Ok(GajiScanResult {
        gaji_packages,
        warnings,
    })
}

pub fn render_gaji_scan(result: &GajiScanResult) -> Result<String, String> {
    let mut lines = Vec::with_capacity(result.warnings.len() + 2);
    for warning in &result.warnings {
        let message = format!(
            "{} 에 gaji.toml이 없어 기본 스캔에서 SKIP합니다.",
            warning.relative_path
        );
        let payload = ScanWarningPayload {
            kind: "scan",
            level: "warn",
            code: SKIP_WARNING_CODE,
            message: &message,
        };
        lines.push(
            serde_json::to_string(&payload).map_err(|err| format!("E_SCAN_SERIALIZE {err}"))?,
        );
    }
    lines.push(format!("scan_gaji_packages={}", result.gaji_packages));
    lines.push(format!("scan_warnings={}", result.warnings.len()));
    Ok(format!("{}\n", lines.join("\n")))
}

pub fn run_gaji_scan(root: &Path) -> Result<(), String> {
    let result = scan_gaji_packages(root)?;
    print!("{}", render_gaji_scan(&result)?);
    Ok(())
}

fn directory_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("ddn_scan_{name}_{}", std::process::id()))
    }

    #[test]
    fn scan_is_sorted_and_classifies_only_gaji_directories() {
        let root = test_root("sorted");
        let _ = fs::remove_dir_all(&root);
        for name in ["z_skip", "valid_b", ".hidden", "a_skip", "valid_a"] {
            fs::create_dir_all(root.join("gaji").join(name)).unwrap();
        }
        fs::write(root.join("gaji/valid_a/gaji.toml"), "[gaji]\n").unwrap();
        fs::write(root.join("gaji/valid_b/gaji.toml"), "[gaji]\n").unwrap();
        fs::write(root.join("gaji/not_a_directory"), "ignored\n").unwrap();

        let result = scan_gaji_packages(&root).unwrap();
        assert_eq!(result.gaji_packages, 2);
        assert_eq!(
            result.warnings,
            vec![
                GajiScanWarning {
                    relative_path: "gaji/a_skip".to_string(),
                },
                GajiScanWarning {
                    relative_path: "gaji/z_skip".to_string(),
                },
            ]
        );
        let rendered = render_gaji_scan(&result).unwrap();
        assert!(rendered.find("gaji/a_skip").unwrap() < rendered.find("gaji/z_skip").unwrap());
        assert!(rendered.ends_with("scan_gaji_packages=2\nscan_warnings=2\n"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn invalid_gaji_root_fails_closed() {
        let root = test_root("invalid_root");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("gaji"), "not a directory\n").unwrap();

        let error = scan_gaji_packages(&root).unwrap_err();
        assert!(error.starts_with("E_SCAN_READ "));

        fs::remove_dir_all(&root).unwrap();
    }
}
