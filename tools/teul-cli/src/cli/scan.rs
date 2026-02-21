use std::fs;
use std::path::Path;

pub fn run(root: &Path) -> Result<(), String> {
    let gaji_root = root.join("gaji");
    let mut warnings = Vec::new();
    let mut gaji_packages = 0usize;

    if gaji_root.exists() {
        let entries = fs::read_dir(&gaji_root).map_err(|e| format!("E_SCAN_READ {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("E_SCAN_READ {}", e))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') {
                continue;
            }
            let gaji_toml = path.join("gaji.toml");
            if gaji_toml.exists() {
                gaji_packages += 1;
            } else {
                let rel = format!("gaji/{}", name);
                warnings.push(rel);
            }
        }
    }

    for rel in &warnings {
        print_warn(
            "W_SKIP_NON_GAJI_DIR",
            &format!("{} 에 gaji.toml이 없어 기본 스캔에서 SKIP합니다.", rel),
        );
    }

    println!("scan_gaji_packages={}", gaji_packages);
    println!("scan_warnings={}", warnings.len());
    Ok(())
}

fn print_warn(code: &str, message: &str) {
    println!(
        "{{\"kind\":\"scan\",\"level\":\"warn\",\"code\":\"{}\",\"message\":\"{}\"}}",
        code,
        json_escape(message)
    );
}

fn json_escape(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}
