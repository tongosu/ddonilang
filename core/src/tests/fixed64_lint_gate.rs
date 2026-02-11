use std::fs;
use std::path::Path;

const ALLOW_MARKER: &str = "FIXED64_LINT_ALLOW";

#[test]
fn fixed64_lint_gate_no_float_in_core() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();
    scan_dir(&root, &mut violations);

    if !violations.is_empty() {
        let mut message = String::from("Fixed64 Lint Gate violation:\n");
        for line in violations {
            message.push_str(&line);
            message.push('\n');
        }
        panic!("{message}");
    }
}

fn scan_dir(dir: &Path, violations: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, violations);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            scan_file(&path, violations);
        }
    }
}

fn scan_file(path: &Path, violations: &mut Vec<String>) {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "fixed64_lint_gate.rs")
        .unwrap_or(false)
    {
        return;
    }
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    for (idx, line) in content.lines().enumerate() {
        if line.contains(ALLOW_MARKER) {
            continue;
        }
        if contains_token(line, "f32") || contains_token(line, "f64") {
            violations.push(format!(
                "{}:{}: {}",
                path.display(),
                idx + 1,
                line.trim_end()
            ));
        }
    }
}

fn contains_token(line: &str, token: &str) -> bool {
    let mut offset = 0usize;
    while let Some(pos) = line[offset..].find(token) {
        let idx = offset + pos;
        let before = line[..idx].chars().last();
        let after = line[idx + token.len()..].chars().next();
        let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
        if !before.map_or(false, is_ident) && !after.map_or(false, is_ident) {
            return true;
        }
        offset = idx + token.len();
    }
    false
}
